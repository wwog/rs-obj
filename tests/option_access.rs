//! `?.` 只用于 Option；整条链只要出现一次 `?.`，结果就保留 Option 包装。

use std::num::NonZeroUsize;

use obj::obj;

struct Output {
    root: Option<String>,
}

struct Config {
    output: Option<Output>,
    name: String,
}

struct Logger {
    config: Option<Config>,
}

struct App {
    logger: Option<Logger>,
}

fn full_app() -> App {
    App {
        logger: Some(Logger {
            config: Some(Config {
                output: Some(Output {
                    root: Some("/tmp/a.log".into()),
                }),
                name: "demo".into(),
            }),
        }),
    }
}

#[test]
fn every_level_is_an_option() {
    let app = full_app();
    let root: Option<String> = obj!(app.logger?.config?.output?.root?);
    assert_eq!(root.as_deref(), Some("/tmp/a.log"));
}

#[test]
fn one_question_mark_keeps_the_whole_chain_optional() {
    struct Inner {
        root: String,
    }
    struct Middle {
        output: Inner,
    }
    struct Outer {
        logger: Option<Middle>,
    }

    let outer = Outer {
        logger: Some(Middle {
            output: Inner {
                root: "/tmp/b.log".into(),
            },
        }),
    };

    let root: Option<String> = obj!(outer.logger?.output.root);
    assert_eq!(root.as_deref(), Some("/tmp/b.log"));
}

#[test]
fn none_propagates_to_the_end() {
    let app = App { logger: None };
    let root: Option<String> = obj!(app.logger?.config?.output?.root?);
    assert_eq!(root, None);

    let app = App {
        logger: Some(Logger { config: None }),
    };
    let name: Option<String> = obj!(app.logger?.config?.name);
    assert_eq!(name, None);
}

#[test]
fn an_optional_leaf_stays_flat() {
    struct Wrap {
        inner: Option<NonZeroUsize>,
    }

    let wrap = Wrap {
        inner: NonZeroUsize::new(7),
    };
    let inner: Option<NonZeroUsize> = obj!(wrap.inner?);
    assert_eq!(inner.map(NonZeroUsize::get), Some(7));
}

#[test]
fn option_after_option_is_flattened() {
    struct Wrap {
        inner: Option<NonZeroUsize>,
    }
    struct Outer {
        wrap: Option<Wrap>,
    }

    let outer = Outer {
        wrap: Some(Wrap {
            inner: NonZeroUsize::new(9),
        }),
    };
    let inner: Option<NonZeroUsize> = obj!(outer.wrap?.inner?);
    assert_eq!(inner.map(NonZeroUsize::get), Some(9));
}
