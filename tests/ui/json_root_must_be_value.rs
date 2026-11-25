use obj::obj;

fn main() {
    let text = String::from(r#"{"a": 1}"#);
    let _ = obj!(json: text.a);
}
