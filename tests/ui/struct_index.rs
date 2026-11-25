use obj::obj;

struct App {
    items: Vec<String>,
}

fn main() {
    let app = App { items: vec![] };
    let _ = obj!(app[0]);
}
