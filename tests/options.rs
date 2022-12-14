use std::ffi::OsString;

use uutils_args::{Arguments, FromValue, Options};

#[test]
fn string_option() {
    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("--message=MSG")]
        Message(String),
    }

    #[derive(Default, Options)]
    #[arg_type(Arg)]
    struct Settings {
        #[set(Arg::Message)]
        message: String,
    }

    assert_eq!(
        Settings::parse(["test", "--message=hello"]).message,
        "hello"
    );
}

#[test]
fn enum_option() {
    #[derive(FromValue, Default, Debug, PartialEq, Eq, Clone)]
    enum Format {
        #[default]
        #[value]
        Foo,
        #[value]
        Bar,
        #[value]
        Baz,
    }

    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("--format=FORMAT")]
        Format(Format),
    }

    #[derive(Default, Options)]
    #[arg_type(Arg)]
    struct Settings {
        #[set(Arg::Format)]
        format: Format,
    }

    assert_eq!(
        Settings::parse(["test", "--format=bar"]).format,
        Format::Bar
    );

    assert_eq!(
        Settings::parse(["test", "--format", "baz"]).format,
        Format::Baz
    );
}

#[test]
fn enum_option_with_fields() {
    #[derive(FromValue, Default, Debug, PartialEq, Eq, Clone)]
    enum Indent {
        #[default]
        Tabs,
        #[value("thin", value = Self::Spaces(4))]
        #[value("wide", value = Self::Spaces(8))]
        Spaces(u8),
    }

    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("-i INDENT")]
        Indent(Indent),
    }

    #[derive(Default, Options)]
    #[arg_type(Arg)]
    struct Settings {
        #[set(Arg::Indent)]
        indent: Indent,
    }

    assert_eq!(
        Settings::parse(["test", "-i=thin"]).indent,
        Indent::Spaces(4)
    );
    assert_eq!(
        Settings::parse(["test", "-i=wide"]).indent,
        Indent::Spaces(8)
    );
}

#[test]
fn enum_with_complex_from_value() {
    #[derive(Default, Debug, PartialEq, Eq, Clone)]
    enum Indent {
        #[default]
        Tabs,
        Spaces(u8),
    }

    impl FromValue for Indent {
        fn from_value(option: &str, value: std::ffi::OsString) -> Result<Self, uutils_args::Error> {
            let value = String::from_value(option, value)?;
            if value == "tabs" {
                Ok(Self::Tabs)
            } else if let Ok(n) = value.parse() {
                Ok(Self::Spaces(n))
            } else {
                Err(uutils_args::Error::ParsingFailed {
                    option: option.to_string(),
                    value,
                    error: "Failure!".into(),
                })
            }
        }
    }

    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("-i INDENT")]
        Indent(Indent),
    }

    #[derive(Default, Options)]
    #[arg_type(Arg)]
    struct Settings {
        #[map(Arg::Indent(i) => i.clone())]
        indent: Indent,
    }

    assert_eq!(Settings::parse(["test", "-i=tabs"]).indent, Indent::Tabs);
    assert_eq!(Settings::parse(["test", "-i=4"]).indent, Indent::Spaces(4));
}

#[test]
fn color() {
    #[derive(Default, FromValue, Debug, PartialEq, Eq, Clone)]
    enum Color {
        #[value("yes", "always")]
        Always,
        #[default]
        #[value("auto")]
        Auto,
        #[value("no", "never")]
        Never,
    }

    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("--color[=WHEN]")]
        Color(Option<Color>),
    }

    #[derive(Default, Options)]
    #[arg_type(Arg)]
    struct Settings {
        #[map(
            Arg::Color(Some(c)) => c.clone(),
            Arg::Color(None) => Color::Always,
        )]
        color: Color,
    }

    assert_eq!(
        Settings::parse(["test", "--color=yes"]).color,
        Color::Always
    );
    assert_eq!(
        Settings::parse(["test", "--color=always"]).color,
        Color::Always
    );
    assert_eq!(Settings::parse(["test", "--color=no"]).color, Color::Never);
    assert_eq!(
        Settings::parse(["test", "--color=never"]).color,
        Color::Never
    );
    assert_eq!(Settings::parse(["test", "--color=auto"]).color, Color::Auto);
    assert_eq!(Settings::parse(["test", "--color"]).color, Color::Always)
}

#[test]
fn actions() {
    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("-m MESSAGE")]
        Message(String),
        #[option("--send")]
        Send,
        #[option("--receive")]
        Receive,
    }

    #[derive(Options, Default)]
    #[arg_type(Arg)]
    struct Settings {
        #[map(Arg::Message(m) => m.clone())]
        message1: String,

        #[set(Arg::Message)]
        message2: String,

        #[map(
            Arg::Send => true,
            Arg::Receive => false,
        )]
        send: bool,

        // Or map, true or false inside the collect
        #[collect(set(Arg::Message))]
        messages: Vec<String>,
    }

    let settings = Settings::parse(["test", "-m=Hello", "-m=World", "--send"]);
    assert_eq!(settings.messages, vec!["Hello", "World"]);
    assert_eq!(settings.message1, "World");
    assert_eq!(settings.message2, "World");
    assert!(settings.send);
}

#[test]
fn width() {
    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("-w WIDTH")]
        Width(u64),
    }

    #[derive(Options, Default)]
    #[arg_type(Arg)]
    struct Settings {
        #[map(
            Arg::Width(0) => None,
            Arg::Width(x) => Some(x),
        )]
        width: Option<u64>,
    }

    assert_eq!(Settings::parse(["test", "-w=0"]).width, None);
    assert_eq!(Settings::parse(["test", "-w=1"]).width, Some(1));
}

#[test]
fn integers() {
    #[derive(Arguments, Clone)]
    enum Arg {
        #[option("--u8=N")]
        U8(u8),
        #[option("--u16=N")]
        U16(u16),
        #[option("--u32=N")]
        U32(u32),
        #[option("--u64=N")]
        U64(u64),
        #[option("--u128=N")]
        U128(u128),
        #[option("--i8=N")]
        I8(i8),
        #[option("--i16=N")]
        I16(i16),
        #[option("--i32=N")]
        I32(i32),
        #[option("--i64=N")]
        I64(i64),
        #[option("--i128=N")]
        I128(i128),
    }

    #[derive(Options, Default)]
    #[arg_type(Arg)]
    struct Settings {
        #[map(
            Arg::U8(x) => x as i128,
            Arg::U16(x) => x as i128,
            Arg::U32(x) => x as i128,
            Arg::U64(x) => x as i128,
            Arg::U128(x) => x as i128,
            Arg::I8(x) => x as i128,
            Arg::I16(x) => x as i128,
            Arg::I32(x) => x as i128,
            Arg::I64(x) => x as i128,
            Arg::I128(x) => x,
        )]
        n: i128,
    }

    assert_eq!(Settings::parse(["test", "--u8=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u16=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u32=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u64=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--u128=5"]).n, 5);

    assert_eq!(Settings::parse(["test", "--i8=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i16=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i32=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i64=5"]).n, 5);
    assert_eq!(Settings::parse(["test", "--i128=5"]).n, 5);
}

#[test]
fn ls_classify() {
    #[derive(FromValue, Default, Clone, PartialEq, Eq, Debug)]
    enum When {
        #[value]
        Never,
        #[default]
        #[value]
        Auto,
        #[value]
        Always,
    }

    #[derive(Clone, Arguments)]
    enum Arg {
        #[option(
            "-F", "--classify[=WHEN]",
            default = When::Always,
        )]
        Classify(When),
    }

    #[derive(Options, Default)]
    #[arg_type(Arg)]
    struct Settings {
        #[set(Arg::Classify)]
        classify: When,
    }

    assert_eq!(Settings::parse(["test"]).classify, When::Auto);
    assert_eq!(
        Settings::parse(["test", "--classify=never"]).classify,
        When::Never,
    );
    assert_eq!(
        Settings::parse(["test", "--classify"]).classify,
        When::Always,
    );
    assert_eq!(Settings::parse(["test", "-F"]).classify, When::Always,);
    assert!(Settings::try_parse(["test", "-Falways"]).is_err());
}

#[test]
fn mktemp_tmpdir() {
    #[derive(Clone, Arguments)]
    enum Arg {
        #[option(
            "-p DIR", "--tmpdir[=DIR]",
            default = String::from("/tmp"),
        )]
        TmpDir(String),
    }

    #[derive(Default, Options)]
    #[arg_type(Arg)]
    struct Settings {
        #[map(Arg::TmpDir(dir) => Some(dir))]
        tmpdir: Option<String>,
    }

    let settings = Settings::parse(["test", "-p", "X"]);
    assert_eq!(settings.tmpdir.unwrap(), "X");

    let settings = Settings::parse(["test", "--tmpdir=X"]);
    assert_eq!(settings.tmpdir.unwrap(), "X");

    let settings = Settings::parse(["test", "--tmpdir"]);
    assert_eq!(settings.tmpdir.unwrap(), "/tmp");

    assert!(Settings::try_parse(["test", "-p"]).is_err());
}

#[test]
fn infer_value() {
    #[derive(FromValue, PartialEq, Eq, Debug)]
    enum Foo {
        #[value("long")]
        Long,
        #[value("link")]
        Link,
        #[value("deck")]
        Deck,
        #[value("desk")]
        Desk,
    }

    assert_eq!(
        Foo::from_value("--foo", OsString::from("lo")).unwrap(),
        Foo::Long
    );
    assert_eq!(
        Foo::from_value("--foo", OsString::from("dec")).unwrap(),
        Foo::Deck
    );

    Foo::from_value("--foo", OsString::from("l")).unwrap_err();
    Foo::from_value("--foo", OsString::from("de")).unwrap_err();
}
