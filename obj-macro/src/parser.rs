//! 把宏输入解析成 [`ObjInput`]。
//!
//! ```text
//! [namespace ':'] path ['as' Type] ['??' Expr]
//! path := base ('?'? '.' field | '?'? '[' expr ']')*
//! ```
//!
//! 访问链直接交给 `syn` 解析：Rust 的 `?` 后缀与 `.field` 后缀天然
//! 构成 `Try` / `Field` 嵌套，正好对应「当前值是不是 Option」。
//! `??` 不是 Rust 语法，于是先在 token 层按顶层 `??` 切成两半。

use proc_macro2::{Spacing, Span, TokenStream, TokenTree};
use syn::{Expr, Type};

use crate::ast::{Backend, Chain, ObjInput, Step, StepKind};

pub fn parse(tokens: TokenStream) -> syn::Result<ObjInput> {
    let (head, default) = split_default(tokens)?;
    if head.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "obj! 需要一个访问表达式",
        ));
    }

    let (backend, path_tokens) = split_backend(head.into_iter().collect())?;
    if path_tokens.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "数据后端后面缺少访问表达式",
        ));
    }

    let expr = syn::parse2::<Expr>(path_tokens)?;
    let (chain, cast) = decompose(expr)?;

    let input = ObjInput {
        backend,
        chain,
        cast,
        default,
    };
    validate(&input)?;
    Ok(input)
}

/// 把顶层第一个 `??` 切开：左边是访问链，右边是默认值表达式。
fn split_default(tokens: TokenStream) -> syn::Result<(TokenStream, Option<Expr>)> {
    let mut head = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        // 两个连续的 `?` 才是默认值分隔符；`?.` 的下一个 token 是 `.`。
        // `??` 中间可以有空格，所以不能依赖 `Spacing::Joint`。
        if let TokenTree::Punct(punct) = &token
            && punct.as_char() == '?'
            && let Some(TokenTree::Punct(next)) = iter.peek()
            && next.as_char() == '?'
        {
            // 吃掉第二个 `?`，剩下的才是默认值表达式。
            iter.next();
            let default_tokens: TokenStream = iter.collect();
            if default_tokens.is_empty() {
                return Err(syn::Error::new(
                    punct.span(),
                    "`??` 后面需要一个默认值表达式",
                ));
            }
            let default = syn::parse2::<Expr>(default_tokens)?;
            return Ok((head.into_iter().collect(), Some(default)));
        }
        head.push(token);
    }

    Ok((head.into_iter().collect(), None))
}

/// 识别可选的 `namespace:` 前缀。单个 `:` 才是命名空间，`::` 是路径。
fn split_backend(tokens: Vec<TokenTree>) -> syn::Result<(Option<Backend>, TokenStream)> {
    if let [TokenTree::Ident(name), TokenTree::Punct(colon), ..] = tokens.as_slice()
        && colon.as_char() == ':'
        && colon.spacing() == Spacing::Alone
    {
        let backend = match name.to_string().as_str() {
            "json" => Backend::Json,
            other => {
                return Err(syn::Error::new(
                    name.span(),
                    format!("未知的数据后端 `{other}:`；V1 只支持 `json:`"),
                ));
            }
        };
        let rest = tokens[2..].iter().cloned().collect();
        return Ok((Some(backend), rest));
    }

    Ok((None, tokens.into_iter().collect()))
}

/// 剥掉 `as T`，再把表达式拆成「根 + 访问级」。
fn decompose(expr: Expr) -> syn::Result<(Chain, Option<Type>)> {
    let expr = peel(expr);
    let (expr, cast) = match expr {
        Expr::Cast(cast) => (peel(*cast.expr), Some(*cast.ty)),
        other => (other, None),
    };

    let mut builder = ChainBuilder::default();
    collect(&expr, &mut builder)?;
    Ok((builder.finish()?, cast))
}

/// 去掉不改变语义的括号，避免 `(a.b)` 被当成一个无法拆解的根。
fn peel(expr: Expr) -> Expr {
    match expr {
        Expr::Paren(paren) => peel(*paren.expr),
        Expr::Group(group) => peel(*group.expr),
        other => other,
    }
}

#[derive(Default)]
struct ChainBuilder {
    root: Option<Expr>,
    root_optional: Option<Span>,
    steps: Vec<Step>,
}

impl ChainBuilder {
    /// 把 `?` 标记到当前链的末端：没有访问级时落在根上。
    fn mark_optional(&mut self, span: Span) -> syn::Result<()> {
        match self.steps.last_mut() {
            Some(step) => {
                if step.optional.is_some() {
                    return Err(syn::Error::new(span, "重复的 `?`"));
                }
                step.optional = Some(span);
            }
            None => {
                if self.root_optional.is_some() {
                    return Err(syn::Error::new(span, "重复的 `?`"));
                }
                self.root_optional = Some(span);
            }
        }
        Ok(())
    }

    fn finish(self) -> syn::Result<Chain> {
        let root = self
            .root
            .ok_or_else(|| syn::Error::new(Span::call_site(), "obj! 需要一个访问根表达式"))?;
        Ok(Chain {
            root,
            root_optional: self.root_optional,
            steps: self.steps,
        })
    }
}

fn collect(expr: &Expr, chain: &mut ChainBuilder) -> syn::Result<()> {
    match expr {
        Expr::Field(field) => {
            collect(&field.base, chain)?;
            chain.steps.push(Step {
                kind: StepKind::Field(field.member.clone()),
                optional: None,
            });
        }
        Expr::Index(index) => {
            collect(&index.expr, chain)?;
            chain.steps.push(Step {
                kind: StepKind::Index((*index.index).clone()),
                optional: None,
            });
        }
        Expr::Try(try_expr) => {
            collect(&try_expr.expr, chain)?;
            chain.mark_optional(try_expr.question_token.span)?;
        }
        Expr::Paren(paren) => collect(&paren.expr, chain)?,
        Expr::Group(group) => collect(&group.expr, chain)?,
        other => {
            if chain.root.is_some() {
                return Err(syn::Error::new_spanned(other, "不支持的访问根表达式"));
            }
            chain.root = Some(other.clone());
        }
    }
    Ok(())
}

fn validate(input: &ObjInput) -> syn::Result<()> {
    match input.backend {
        None => validate_struct(input),
        Some(Backend::Json) => validate_json(input),
    }
}

/// 结构体：相信编译器。这里只拦下语法层面就说不通的东西。
fn validate_struct(input: &ObjInput) -> syn::Result<()> {
    if let Some(ty) = &input.cast {
        return Err(syn::Error::new_spanned(
            ty,
            "结构体访问不使用 `as`：最终类型由字段类型决定，例如 `obj!(app.logger.root)`",
        ));
    }

    for step in &input.chain.steps {
        if let StepKind::Index(index) = &step.kind {
            return Err(syn::Error::new_spanned(
                index,
                "结构体访问不支持索引 `[...]`；索引只用于动态后端，例如 `obj!(json: data[0])`",
            ));
        }
    }

    if input.chain.steps.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.chain.root,
            "obj! 需要至少一个字段访问，例如 `obj!(app.logger.root)`",
        ));
    }

    if let Some(default) = &input.default
        && !input.chain.is_optional()
    {
        return Err(syn::Error::new_spanned(
            default,
            "`??` 只能跟在包含 `?.` 的访问链后面",
        ));
    }

    Ok(())
}

/// JSON：缺失、越界、类型转换失败都收敛成 None，不需要 `?.`。
fn validate_json(input: &ObjInput) -> syn::Result<()> {
    let message = "JSON 访问不需要 `?.`：缺失节点会自动返回 None";

    if let Some(span) = input.chain.root_optional {
        return Err(syn::Error::new(span, message));
    }
    for step in &input.chain.steps {
        if let Some(span) = step.optional {
            return Err(syn::Error::new(span, message));
        }
    }

    Ok(())
}
