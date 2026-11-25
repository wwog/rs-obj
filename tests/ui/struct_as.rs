use obj::obj;

struct App {
    name: String,
}

fn main() {
    let app = App { name: "x".into() };
    let _ = obj!(app.name as String);
}
