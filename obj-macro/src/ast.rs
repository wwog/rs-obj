//! `obj!` 的中间表示：数据后端、访问链、最终类型转换与默认值。

use proc_macro2::Span;
use syn::{Expr, Member, Type};

/// 动态数据后端。V1 只支持 JSON；`toml` / `yaml` 留给后续版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Json,
}

/// 宏的完整输入。
pub struct ObjInput {
    /// `None` 表示 Rust 结构体（强类型，直接取字段）。
    pub backend: Option<Backend>,
    /// 从根表达式开始的访问链。
    pub chain: Chain,
    /// 动态后端的 `as T`；结构体禁止使用。
    pub cast: Option<Type>,
    /// `?? default`，兜底整个访问链。
    pub default: Option<Expr>,
}

/// 一条访问链：根表达式 + 若干级字段/索引。
pub struct Chain {
    pub root: Expr,
    /// 根上的 `?` 位置（`root?`）。
    pub root_optional: Option<Span>,
    pub steps: Vec<Step>,
}

/// 一级访问。
pub struct Step {
    pub kind: StepKind,
    /// 本级结果上的 `?` 位置（`a.field?`）。
    pub optional: Option<Span>,
}

pub enum StepKind {
    /// `.name` 或 `.0`
    Field(Member),
    /// `[expr]`
    Index(Expr),
}

impl Chain {
    /// 整条链上是否出现过 `?`。
    pub fn is_optional(&self) -> bool {
        self.root_optional.is_some() || self.steps.iter().any(|step| step.optional.is_some())
    }
}
