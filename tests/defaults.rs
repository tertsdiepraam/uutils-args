use uutils_args::{Arguments, Initial, Options};

#[test]
fn true_default() {
    #[derive(Arguments)]
    enum Arg {
        #[option("--foo")]
        Foo,
    }

    #[derive(Initial)]
    struct Settings {
        #[field(default = true)]
        foo: bool,
    }

    impl Options for Settings {
        type Arg = Arg;
        fn apply(&mut self, Arg::Foo: Arg) {
            self.foo = false;
        }
    }

    assert!(Settings::parse(["test"]).foo);
    assert!(!Settings::parse(["test", "--foo"]).foo);
}

#[test]
fn env_var_string() {
    #[derive(Arguments)]
    enum Arg {
        #[option("--foo=MSG")]
        Foo(String),
    }

    #[derive(Initial)]
    struct Settings {
        #[field(env = "FOO")]
        foo: String,
    }

    impl Options for Settings {
        type Arg = Arg;
        fn apply(&mut self, Arg::Foo(x): Arg) {
            self.foo = x;
        }
    }

    std::env::set_var("FOO", "one");
    assert_eq!(Settings::parse(["test"]).foo, "one");

    std::env::set_var("FOO", "two");
    assert_eq!(Settings::parse(["test"]).foo, "two");

    std::env::remove_var("FOO");
    assert_eq!(Settings::parse(["test"]).foo, "");
}