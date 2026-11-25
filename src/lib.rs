//! 只做属性访问和结果提取的宏。
//!
//! 强类型数据（Rust 结构体）直接取字段，最终类型由字段类型决定；
//! 动态数据（JSON）显式声明数据后端，用 `as T` 指定最终类型。
//! 空值只认 `?.`，失败兜底只认 `??`。
//!
//! # 结构体
//!
//! ```
//! use obj::obj;
//!
//! struct Output {
//!     root: String,
//! }
//! struct Config {
//!     output: Output,
//! }
//! struct App {
//!     logger: Option<Config>,
//! }
//!
//! let app = App {
//!     logger: Some(Config {
//!         output: Output { root: "/tmp/app.log".into() },
//!     }),
//! };
//!
//! // 字段类型直接可见，不需要 `as`。
//! let root: Option<String> = obj!(app.logger?.output.root);
//! assert_eq!(root.as_deref(), Some("/tmp/app.log"));
//! ```
//!
//! 只有 `logger` 是 Option 时，`?.` 之后的所有普通字段都自动在 Option
//! 内部执行，整条链只保留一层 Option 包装。
//!
//! `??` 兜底整条访问链，默认值直接决定最终类型：
//!
//! ```
//! use obj::obj;
//!
//! struct Output {
//!     root: String,
//! }
//! struct Config {
//!     output: Output,
//! }
//! struct App {
//!     logger: Option<Config>,
//! }
//!
//! let app = App { logger: None };
//! let root: String = obj!(app.logger?.output.root ?? "/tmp/default.log".to_string());
//! assert_eq!(root, "/tmp/default.log");
//! ```
//!
//! # JSON
//!
//! JSON 用 `namespace:` 声明数据后端；`.` 和 `[]` 负责路径；
//! 缺失字段、数组越界、类型转换失败统一得到 `None`。
//!
//! ```
//! use obj::obj;
//! use serde_json::json;
//!
//! let res = json!({"data": [{"name": "hy4", "tags": ["a", "b"]}]});
//!
//! // 不写 as：Option<&serde_json::Value>
//! let name = obj!(json: res.data[0].name).and_then(|v| v.as_str());
//! assert_eq!(name, Some("hy4"));
//!
//! // 写 as T：只把最终节点反序列化成 T
//! let tags: Option<Vec<String>> = obj!(json: res.data[0].tags as Vec<String>);
//! assert_eq!(tags, Some(vec!["a".into(), "b".into()]));
//!
//! // 再加 ??：得到 T
//! let tags: Vec<String> = obj!(json: res.data[0].tags as Vec<String> ?? vec![]);
//! assert_eq!(tags, vec!["a".to_string(), "b".to_string()]);
//! ```
//!
//! # 语法表
//!
//! | 语法 | 适用对象 | 结果 |
//! | --- | --- | --- |
//! | `obj!(a.b.c)` | Rust 结构体 | 最终字段类型 |
//! | `obj!(a?.b.c)` | Rust 结构体 | `Option<T>` |
//! | `obj!(a?.b ?? default)` | Rust 结构体 | `T` |
//! | `obj!(json: a.b)` | JSON | `Option<&serde_json::Value>` |
//! | `obj!(json: a[0].b)` | JSON | `Option<&serde_json::Value>` |
//! | `obj!(json: a.b as T)` | JSON | `Option<T>` |
//! | `obj!(json: a.b as T ?? default)` | JSON | `T` |
//!
//! # 边界
//!
//! - 结构体使用 `as`：宏直接报错，最终类型由字段决定。
//! - JSON 使用 `?.`：宏直接报错，缺失节点本来就是 `None`。
//! - `.` 访问 Option、`?.` 访问普通结构体：由 rustc 报编译错误。
//! - 默认值类型不匹配：由 rustc 报编译错误。
//! - 结构体字段按值移动，不隐式 `clone`；需要引用时从 `&root` 或
//!   `as_ref()` 进入。
//! - `??` 展开为 `unwrap_or`，默认值表达式**总会**求值一次。
//! - 不提供隐式解包、自动类型转换或额外的失败处理策略。

pub use obj_macro::obj;

#[doc(hidden)]
pub mod __private {
    pub use serde_json;

    /// JSON 后端 `as T` 的落地点：把最终节点反序列化成 `T`，失败返回 `None`。
    pub fn from_value<T: serde::de::DeserializeOwned>(value: &serde_json::Value) -> Option<T> {
        serde_json::from_value(value.clone()).ok()
    }

    /// 纯类型断言：`?.` 的左侧必须是 Option。
    /// 放在这里是为了让误用 `?.` 的编译错误直接指向 Option 约束。
    pub fn require_option<T>(value: Option<T>) -> Option<T> {
        value
    }

    /// 纯类型断言：`json:` 的根必须是 `serde_json::Value`。
    pub fn require_value(value: &serde_json::Value) -> &serde_json::Value {
        value
    }
}
