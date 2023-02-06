use std::path::{Path, PathBuf};

use uutils_args::{Arguments, Initial, Options};

#[derive(Clone, Arguments)]
enum Arg {
    #[option("-d", "--directory")]
    Directory,

    #[option("-u", "--dry-run")]
    DryRun,

    #[option("-q", "--quiet")]
    Quiet,

    #[option("--suffix=SUFFIX")]
    Suffix(String),

    #[option("-t")]
    TreatAsTemplate,

    #[option("-p DIR", "--tmpdir[=DIR]", default = ".".into())]
    TmpDir(PathBuf),

    #[positional(0..=1)]
    Template(String),
}

#[derive(Default, Initial)]
struct Settings {
    directory: bool,
    dry_run: bool,
    quiet: bool,
    tmp_dir: Option<PathBuf>,
    suffix: Option<String>,
    treat_as_template: bool,
    template: String,
}

impl Options for Settings {
    type Arg = Arg;
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::Directory => self.directory = true,
            Arg::DryRun => self.dry_run = true,
            Arg::Quiet => self.quiet = true,
            Arg::Suffix(s) => self.suffix = Some(s),
            Arg::TreatAsTemplate => self.treat_as_template = true,
            Arg::TmpDir(dir) => self.tmp_dir = Some(dir),
            Arg::Template(s) => self.template = s,
        }
    }
}

#[test]
fn suffix() {
    let s = Settings::parse(["mktemp", "--suffix=hello"]);
    assert_eq!(s.suffix.unwrap(), "hello");

    let s = Settings::parse(["mktemp", "--suffix="]);
    assert_eq!(s.suffix.unwrap(), "");

    let s = Settings::parse(["mktemp", "--suffix="]);
    assert_eq!(s.suffix.unwrap(), "");

    let s = Settings::parse(["mktemp"]);
    assert_eq!(s.suffix, None);
}

#[test]
fn tmpdir() {
    let s = Settings::parse(["mktemp", "--tmpdir"]);
    assert_eq!(s.tmp_dir.unwrap(), Path::new("."));

    let s = Settings::parse(["mktemp", "--tmpdir="]);
    assert_eq!(s.tmp_dir.unwrap(), Path::new(""));

    let s = Settings::parse(["mktemp", "-p", "foo"]);
    assert_eq!(s.tmp_dir.unwrap(), Path::new("foo"));

    let s = Settings::parse(["mktemp", "-pfoo"]);
    assert_eq!(s.tmp_dir.unwrap(), Path::new("foo"));

    let s = Settings::parse(["mktemp", "-p", ""]);
    assert_eq!(s.tmp_dir.unwrap(), Path::new(""));

    assert!(Settings::try_parse(["mktemp", "-p"]).is_err());
}