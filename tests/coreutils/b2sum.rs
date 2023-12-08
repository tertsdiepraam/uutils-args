use std::path::{Path, PathBuf};
use uutils_args::{Arguments, Options};

#[derive(Clone, Arguments)]
enum Arg {
    #[arg("-b", "--binary")]
    Binary,

    #[arg("-c", "--check")]
    Check,

    #[arg("--tag")]
    Tag,

    #[arg("-t", "--text")]
    Text,

    #[arg("-q", "--quiet")]
    Quiet,

    #[arg("-s", "--status")]
    Status,

    #[arg("--strict")]
    Strict,

    #[arg("-w", "--warn")]
    Warn,

    #[arg("FILE", ..)]
    File(PathBuf),
}

#[derive(Default, Debug, PartialEq, Eq)]
enum CheckOutput {
    #[default]
    Warn,
    Quiet,
    Status,
}

#[derive(Default)]
struct Settings {
    binary: bool,
    check: bool,
    tag: bool,
    check_output: CheckOutput,
    strict: bool,
    files: Vec<PathBuf>,
}

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::Binary => self.binary = true,
            Arg::Check => self.check = true,
            Arg::Tag => self.tag = true,
            Arg::Text => self.binary = false,
            Arg::Quiet => self.check_output = CheckOutput::Quiet,
            Arg::Status => self.check_output = CheckOutput::Status,
            Arg::Strict => self.strict = true,
            Arg::Warn => self.check_output = CheckOutput::Warn,
            Arg::File(f) => self.files.push(f),
        }
    }
}

#[test]
fn binary() {
    assert!(!Settings::default().parse(["b2sum"]).binary);
    assert!(!Settings::default().parse(["b2sum", "--text"]).binary);
    assert!(!Settings::default().parse(["b2sum", "-t"]).binary);
    assert!(
        !Settings::default()
            .parse(["b2sum", "--binary", "--text"])
            .binary
    );
    assert!(!Settings::default().parse(["b2sum", "-b", "-t"]).binary);

    assert!(Settings::default().parse(["b2sum", "--binary"]).binary);
    assert!(Settings::default().parse(["b2sum", "-b"]).binary);
    assert!(
        Settings::default()
            .parse(["b2sum", "--text", "--binary"])
            .binary
    );
    assert!(Settings::default().parse(["b2sum", "-t", "-b"]).binary);
}

#[test]
fn check_output() {
    assert_eq!(
        Settings::default().parse(["b2sum", "--warn"]).check_output,
        CheckOutput::Warn
    );
    assert_eq!(
        Settings::default().parse(["b2sum", "--quiet"]).check_output,
        CheckOutput::Quiet
    );
    assert_eq!(
        Settings::default()
            .parse(["b2sum", "--status"])
            .check_output,
        CheckOutput::Status
    );
    assert_eq!(
        Settings::default()
            .parse(["b2sum", "--status", "--warn"])
            .check_output,
        CheckOutput::Warn
    );
    assert_eq!(
        Settings::default()
            .parse(["b2sum", "--status", "--warn"])
            .check_output,
        CheckOutput::Warn
    );

    assert_eq!(
        Settings::default()
            .parse(["b2sum", "--warn", "--quiet"])
            .check_output,
        CheckOutput::Quiet
    );

    assert_eq!(
        Settings::default()
            .parse(["b2sum", "--quiet", "--status"])
            .check_output,
        CheckOutput::Status
    );
}

#[test]
fn files() {
    assert_eq!(
        Settings::default().parse(["b2sum", "foo", "bar"]).files,
        vec![Path::new("foo"), Path::new("bar")]
    );
}
