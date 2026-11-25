use obj::obj;

fn main() {
    let config = 1;
    let _ = obj!(toml: config.database);
}
