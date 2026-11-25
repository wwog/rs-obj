//! 动态 JSON：命名空间 + `.` / `[]` 访问 + `as T` 最终类型转换。

use obj::obj;
use serde_json::{Value, json};

fn res() -> Value {
    json!({
        "object": "list",
        "data": [
            {"name": "hy4", "tags": ["a", "b"], "meta": {"score": 9}},
            {"name": "other"}
        ],
        "weird-key": {"ok": true}
    })
}

#[test]
fn plain_path_returns_option_value() {
    let res = res();
    let name: Option<&Value> = obj!(json: res.data[0].name);
    assert_eq!(name.and_then(Value::as_str), Some("hy4"));
}

#[test]
fn index_accepts_expressions_and_strings() {
    let res = res();

    let i = 1;
    let name = obj!(json: res.data[i].name).and_then(Value::as_str);
    assert_eq!(name, Some("other"));

    let ok = obj!(json: res["weird-key"].ok).and_then(Value::as_bool);
    assert_eq!(ok, Some(true));
}

#[test]
fn missing_nodes_and_out_of_range_are_none() {
    let res = res();

    assert!(obj!(json: res.data[9]).is_none());
    assert!(obj!(json: res.data[0].missing).is_none());
    assert!(obj!(json: res.nope.deeper).is_none());
    assert!(obj!(json: res.data[0].name.wrong).is_none());
}

#[test]
fn cast_deserializes_only_the_final_node() {
    let res = res();

    let tags: Option<Vec<String>> = obj!(json: res.data[0].tags as Vec<String>);
    assert_eq!(tags, Some(vec!["a".into(), "b".into()]));

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Meta {
        score: u32,
    }

    let meta: Option<Meta> = obj!(json: res.data[0].meta as Meta);
    assert_eq!(meta, Some(Meta { score: 9 }));
}

#[test]
fn cast_failure_is_none() {
    let res = res();
    let score: Option<String> = obj!(json: res.data[0].meta.score as String);
    assert_eq!(score, None);
}

#[test]
fn raw_identifiers_map_to_plain_json_keys() {
    let res = json!({"type": "object"});
    let kind = obj!(json: res.r#type).and_then(Value::as_str);
    assert_eq!(kind, Some("object"));
}

#[test]
fn borrowed_roots_work() {
    let res = res();
    let res = &res;
    let name = obj!(json: res.data[0].name).and_then(Value::as_str);
    assert_eq!(name, Some("hy4"));
}
