//! `??` 兜底整条访问链，默认值直接决定最终类型。

use obj::obj;
use serde_json::json;

struct Output {
    root: String,
}

struct Logger {
    output: Option<Output>,
}

struct App {
    logger: Option<Logger>,
}

#[test]
fn struct_default_is_used_when_any_level_is_missing() {
    let app = App { logger: None };
    let root: String = obj!(app.logger?.output?.root ?? "/tmp/default.log".to_string());
    assert_eq!(root, "/tmp/default.log");
}

#[test]
fn struct_value_wins_over_default() {
    let app = App {
        logger: Some(Logger {
            output: Some(Output {
                root: "/tmp/real.log".into(),
            }),
        }),
    };
    let root: String = obj!(app.logger?.output?.root ?? "/tmp/default.log".to_string());
    assert_eq!(root, "/tmp/real.log");
}

#[test]
fn json_cast_default_returns_the_value_type() {
    let missing = json!({"data": [{"name": "x"}]});
    let tags: Vec<String> = obj!(json: missing.data[0].tags as Vec<String> ?? vec![]);
    assert_eq!(tags, Vec::<String>::new());

    let present = json!({"data": [{"tags": ["a", "b"]}]});
    let tags: Vec<String> = obj!(json: present.data[0].tags as Vec<String> ?? vec![]);
    assert_eq!(tags, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn defaults_can_be_block_expressions() {
    let app = App { logger: None };
    let root: String = obj!(app.logger?.output?.root ?? {
        let mut path = String::from("/tmp/");
        path.push_str("fallback.log");
        path
    });
    assert_eq!(root, "/tmp/fallback.log");
}
