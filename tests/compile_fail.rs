//! 编译期边界：结构体 `as`、JSON `?.`、`.` 访问 Option 等都必须编译失败。
//!
//! `tests/ui/*.stderr` 与当前工具链绑定；升级 rustc 后如提示基线过期，
//! 用 `TRYBUILD=overwrite cargo test --test compile_fail` 重新生成。

#[test]
fn compile_fail() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
