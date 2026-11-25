use obj::obj;

struct Output {
    root: Option<String>,
}

struct App {
    output: Option<Output>,
}

fn main() {
    let app = App {
        output: Some(Output {
            root: Some("/tmp/a.log".into()),
        }),
    };
    let _: String = obj!(app.output?.root ?? 1);
}
