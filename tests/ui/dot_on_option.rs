use obj::obj;

struct Logger {
    name: String,
}

struct App {
    logger: Option<Logger>,
}

fn main() {
    let app = App {
        logger: Some(Logger { name: "x".into() }),
    };
    let _ = obj!(app.logger.name);
}
