//! 引用：只在必要处借用，不产生隐式 clone。

use obj::obj;

struct Nested {
    value: u32,
}

struct Simple {
    name: String,
    nested: Nested,
}

#[test]
fn borrowed_root_plain_chain_can_be_borrowed_in_place() {
    let simple = Simple {
        name: "demo".into(),
        nested: Nested { value: 3 },
    };
    let app = &simple;

    // 宏本身按字段类型求值；要拿引用就在使用处借用这个访问结果。
    let name: &String = &obj!(app.name);
    let value: &u32 = &obj!(app.nested.value);

    assert_eq!(name, "demo");
    assert_eq!(*value, 3);
}

#[test]
fn reference_fields_flow_through_option_access() {
    struct Output {
        root: &'static str,
    }
    struct Logger<'a> {
        output: &'a Output,
    }
    struct App<'a> {
        logger: Option<&'a Logger<'a>>,
    }

    let output = Output {
        root: "/tmp/ref.log",
    };
    let logger = Logger { output: &output };
    let app = App {
        logger: Some(&logger),
    };

    let root: Option<&str> = obj!(app.logger?.output.root);
    assert_eq!(root, Some("/tmp/ref.log"));
}
