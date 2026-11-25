//! `obj!` 宏的过程宏实现。
//!
//! 这个 crate 只负责「解析 + 展开」，不提供任何运行时能力。
//! 请依赖 [`obj`](https://crates.io/crates/obj) 使用宏（它会 re-export 本宏，
//! 并提供 JSON 反序列化的私有入口）。

mod ast;
mod expand;
mod parser;

use proc_macro::TokenStream;

/// TypeScript 风格的属性访问宏。
///
/// 完整语法、语义与边界见 `obj` crate 的文档。
#[proc_macro]
pub fn obj(input: TokenStream) -> TokenStream {
    match parser::parse(input.into()) {
        Ok(parsed) => expand::expand(parsed).into(),
        Err(error) => error.to_compile_error().into(),
    }
}
