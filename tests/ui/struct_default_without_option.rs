use obj::obj;

struct App {
    name: String,
}

fn main() {
    let app = App { name: "x".into() };
    let _: String = obj!(app.name ?? "fallback".to_string());
}
