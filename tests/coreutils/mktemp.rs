use std::path::{Path, PathBuf};

use uutils_args::{Arguments, Options};

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

#[derive(Default, Options)]
#[arg_type(Arg)]
struct Settings {
    #[map(Arg::Directory => true)]
    directory: bool,

    #[map(Arg::DryRun => true)]
    dry_run: bool,

    #[map(Arg::Quiet => true)]
    quiet: bool,

    #[map(Arg::TmpDir(p) => Some(p))]
    tmp_dir: Option<PathBuf>,

    #[map(Arg::Suffix(s) => Some(s))]
    suffix: Option<String>,

    #[map(Arg::TreatAsTemplate => true)]
    treat_as_template: bool,

    #[set(Arg::Template)]
    template: String,
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
