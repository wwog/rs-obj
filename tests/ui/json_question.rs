use obj::obj;
use serde_json::json;

fn main() {
    let res = json!({"a": 1});
    let _ = obj!(json: res?.a);
}
