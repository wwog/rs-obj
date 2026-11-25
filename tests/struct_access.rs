//! 强类型结构体：`.` 直接取字段，最终类型就是字段类型。

use obj::obj;

#[derive(Debug)]
struct Output {
    root: String,
    lines: usize,
}

#[derive(Debug)]
struct Logger {
    output: Output,
}

#[derive(Debug)]
struct App {
    logger: Logger,
}

fn sample_app() -> App {
    App {
        logger: Logger {
            output: Output {
                root: "/tmp/demo.log".into(),
                lines: 42,
            },
        },
    }
}

#[test]
fn returns_the_final_field_type() {
    let app = sample_app();
    let root: String = obj!(app.logger.output.root);
    assert_eq!(root, "/tmp/demo.log");

    let app = sample_app();
    let lines: usize = obj!(app.logger.output.lines);
    assert_eq!(lines, 42);
}

#[test]
fn supports_tuple_fields() {
    struct Pair(Output, usize);

    let pair = Pair(
        Output {
            root: "/tmp/pair.log".into(),
            lines: 3,
        },
        7,
    );

    let root: String = obj!(pair.0.root);
    assert_eq!(root, "/tmp/pair.log");

    let id: usize = obj!(pair.1);
    assert_eq!(id, 7);
}

#[test]
fn accepts_arbitrary_root_expressions() {
    let root: String = obj!(sample_app().logger.output.root);
    assert_eq!(root, "/tmp/demo.log");
}

#[test]
fn fields_are_moved_not_cloned() {
    // 不是 Clone 也能取出来：宏不会偷偷 clone。
    #[derive(Debug)]
    struct NoClone(String);

    struct Holder {
        value: NoClone,
    }

    let holder = Holder {
        value: NoClone("payload".into()),
    };
    let value: NoClone = obj!(holder.value);
    assert_eq!(value.0, "payload");
}
