//! 把 [`ObjInput`] 展开成 Rust 表达式。

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::{Expr, Member, Type};

use crate::ast::{Backend, Chain, ObjInput, StepKind};

pub fn expand(input: ObjInput) -> TokenStream {
    match input.backend {
        None => expand_struct(input.chain, input.default),
        Some(Backend::Json) => expand_json(input.chain, input.cast, input.default),
    }
}

/// 结构体：`.` 直接取字段；进入 Option 后，普通字段用 `map`，
/// 下一级仍是 Option 则用 `and_then`，最终保留 Option 包装。
fn expand_struct(chain: Chain, default: Option<Expr>) -> TokenStream {
    let Chain {
        root,
        root_optional,
        steps,
    } = chain;

    let value = format_ident!("__obj_v");
    let mut expr = quote!(#root);
    let mut in_option = root_optional.is_some();
    // 第一次进入 Option 前做一次类型断言，让 `?.` 用在普通结构体上时
    // rustc 报「expected Option」而不是误导性的「App 不是 Iterator」。
    let mut needs_check = root_optional.is_some();
    let mut index = 0;

    while index < steps.len() {
        if !in_option {
            // 当前是普通结构体：直接走字段，`?` 只是标记下一级是 Option。
            let member = field_member(&steps[index].kind);
            expr = quote!(#expr.#member);
            in_option = steps[index].optional.is_some();
            needs_check = in_option;
            index += 1;
            continue;
        }

        let checked = if needs_check {
            needs_check = false;
            quote!(::obj::__private::require_option(#expr))
        } else {
            expr
        };

        if steps[index].optional.is_some() {
            // 下一级本身也是 Option：展开一层，避免 Option<Option<T>>。
            let member = field_member(&steps[index].kind);
            expr = quote!(#checked.and_then(|#value| #value.#member));
            index += 1;
        } else {
            // 连续的普通字段合并进同一个闭包，只需要一次 map。
            let mut body = quote!(#value);
            while index < steps.len() && steps[index].optional.is_none() {
                let member = field_member(&steps[index].kind);
                body = quote!(#body.#member);
                index += 1;
            }
            expr = quote!(#checked.map(|#value| #body));
        }
    }

    // 链尾的 `?` 后面没有别的访问时，也要确认它确实是 Option。
    if needs_check {
        expr = quote!(::obj::__private::require_option(#expr));
    }

    if let Some(default) = default {
        expr = quote!(#expr.unwrap_or(#default));
    }

    expr
}

/// JSON：`get` 链，任何一级失败都会让整条链变成 None；
/// `as T` 只作用于最终节点；`??` 兜底整个表达式。
fn expand_json(chain: Chain, cast: Option<Type>, default: Option<Expr>) -> TokenStream {
    let Chain {
        root,
        root_optional: _,
        steps,
    } = chain;

    let value = format_ident!("__obj_v");
    let mut steps = steps.into_iter();

    // 第一跳直接调用 `get`，避免多包一层 `Some(&value)`；
    // `require_value` 把根表达式的类型固定成 `&Value`，错误更直接。
    let root = quote!(::obj::__private::require_value(&(#root)));
    let mut expr = match steps.next() {
        Some(step) => {
            let key = json_key(&step.kind);
            quote!(#root.get(#key))
        }
        None => quote!(::core::option::Option::Some(#root)),
    };

    for step in steps {
        let key = json_key(&step.kind);
        expr = quote!(#expr.and_then(|#value| #value.get(#key)));
    }

    if let Some(ty) = cast {
        expr = quote!(#expr.and_then(|#value| ::obj::__private::from_value::<#ty>(#value)));
    }

    if let Some(default) = default {
        expr = quote!(#expr.unwrap_or(#default));
    }

    expr
}

fn field_member(kind: &StepKind) -> &Member {
    match kind {
        StepKind::Field(member) => member,
        StepKind::Index(_) => unreachable!("结构体链中的索引已在解析阶段拒绝"),
    }
}

fn json_key(kind: &StepKind) -> TokenStream {
    match kind {
        StepKind::Field(Member::Named(ident)) => {
            // `r#type` 对应 JSON 里的 `"type"`。
            let key = syn::LitStr::new(&ident.unraw().to_string(), ident.span());
            quote!(#key)
        }
        // `.0` 在 JSON 里按数组下标处理。
        StepKind::Field(Member::Unnamed(index)) => quote!(#index as usize),
        StepKind::Index(expr) => quote!(#expr),
    }
}
