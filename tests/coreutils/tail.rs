use std::{ffi::OsString, path::PathBuf};

use uutils_args::{Arguments, Initial, Options, Value};

#[derive(Arguments)]
enum DeprecatedArg {
    #[option("{N}")]
    Shorthand(Shorthand),
    #[positional]
    File(PathBuf),
}

impl Options<DeprecatedArg> for Settings {
    fn apply(&mut self, arg: DeprecatedArg) {
        match arg {
            DeprecatedArg::Shorthand(Shorthand { num, mode, follow }) => {
                self.number = num;
                self.mode = mode;
                self.follow = follow.then_some(FollowMode::Descriptor);
            }
            DeprecatedArg::File(file) => {
                self.inputs.push(file);
            }
        }
    }
}

struct Shorthand {
    num: SigNum,
    mode: Mode,
    follow: bool,
}

// This is not technically 100% compatible with GNU, because the shorthand can
// appear as any argument, not just the first.
impl Value for Shorthand {
    fn from_value(value: &std::ffi::OsStr) -> uutils_args::ValueResult<Self> {
        let s = String::from_value(value)?;

        let mut rest: &str = &s;

        let sig = if let Some(r) = rest.strip_prefix('-') {
            rest = r;
            SigNum::Negative
        } else if let Some(r) = rest.strip_prefix('+') {
            rest = r;
            SigNum::Positive
        } else {
            return Err("Invalid shorthand".into());
        };

        // Find and parse the number part of the string
        let end_num = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let num = rest[..end_num].parse().unwrap_or(10);
        rest = &rest[end_num..];

        let mode = if let Some(r) = rest.strip_prefix('l') {
            rest = r;
            Mode::Lines
        } else if let Some(r) = rest.strip_prefix('c') {
            rest = r;
            Mode::Bytes
        } else if let Some(r) = rest.strip_prefix('b') {
            rest = r;
            Mode::Blocks
        } else {
            Mode::Lines
        };

        let follow = if let Some(r) = rest.strip_prefix('f') {
            rest = r;
            true
        } else {
            false
        };

        if !rest.is_empty() {
            return Err("Invalid shorthand!".into());
        }

        Ok(Self {
            num: sig(num),
            mode,
            follow,
        })
    }
}

#[derive(Arguments)]
enum Arg {
    // TODO: Bytes and Lines should take a `SigNum`
    #[option("-c NUM", "--bytes=NUM")]
    Bytes(u64),

    #[option("-f", "--follow[=HOW]", default=FollowMode::Descriptor)]
    Follow(FollowMode),

    #[option("-F")]
    FollowRetry,

    #[option("--max-unchanged-stats=N")]
    MaxUnchangedStats(u32),

    #[option("-n NUM", "--lines=NUM")]
    Lines(u64),

    #[option("--pid=PID")]
    Pid(u64),

    #[option("-q", "--quiet", "--silent")]
    Quiet,

    #[option("--retry")]
    Retry,

    #[option("-s NUMBER", "--sleep-interval=NUMBER")]
    SleepInterval(u64),

    #[option("-v", "--verbose")]
    Verbose,

    #[option("-z", "--zero-terminated")]
    Zero,

    #[positional(..)]
    File(PathBuf),

    #[option("---presume-input-pipe", hidden)]
    PresumeInputPipe,
}

// We need both negative and positive 0
#[derive(Debug, PartialEq, Eq)]
enum SigNum {
    Positive(u64),
    Negative(u64),
}

impl Default for SigNum {
    fn default() -> Self {
        Self::Negative(10)
    }
}

#[derive(Value, Debug, PartialEq, Eq)]
enum FollowMode {
    #[value("descriptor")]
    Descriptor,
    #[value("name")]
    Name,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Mode {
    Bytes,
    #[default]
    Lines,
    Blocks,
}

#[derive(Initial)]
struct Settings {
    follow: Option<FollowMode>,
    max_unchanged_stats: u32,
    mode: Mode,
    number: SigNum,
    // Should be a dedicated PID type
    pid: u64,
    retry: bool,
    sleep_sec: u64,
    verbose: bool,
    presume_input_pipe: bool,
    inputs: Vec<PathBuf>,
    zero: bool,
}

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::Bytes(n) => {
                self.mode = Mode::Bytes;
                self.number = SigNum::Negative(n);
            }
            Arg::Lines(n) => {
                self.mode = Mode::Lines;
                self.number = SigNum::Negative(n);
            }
            Arg::Follow(mode) => self.follow = Some(mode),
            Arg::FollowRetry => {
                self.follow = Some(FollowMode::Name);
                self.retry = true;
            }
            Arg::MaxUnchangedStats(n) => self.max_unchanged_stats = n,
            Arg::Pid(pid) => self.pid = pid,
            Arg::Quiet => self.verbose = false,
            Arg::Retry => self.retry = true,
            Arg::SleepInterval(n) => self.sleep_sec = n,
            Arg::Verbose => self.verbose = true,
            Arg::Zero => self.zero = true,
            Arg::File(input) => self.inputs.push(input),
            Arg::PresumeInputPipe => self.presume_input_pipe = true,
        }
    }
}

fn parse_tail<I>(iter: I) -> Result<Settings, uutils_args::Error>
where
    I: IntoIterator + Clone + 'static,
    I::Item: Into<OsString>,
{
    <Settings as Options<DeprecatedArg>>::try_parse(iter.clone())
        .or_else(|_| <Settings as Options<Arg>>::try_parse(iter))
}
#[test]
fn shorthand() {
    let s = parse_tail(["tail", "-20", "somefile"]).unwrap();
    assert_eq!(s.number, SigNum::Negative(20));
    assert_eq!(s.mode, Mode::Lines);
    assert_eq!(s.follow, None);

    let s = parse_tail(["tail", "+20", "somefile"]).unwrap();
    assert_eq!(s.number, SigNum::Positive(20));
    assert_eq!(s.mode, Mode::Lines);
    assert_eq!(s.follow, None);

    let s = parse_tail(["tail", "-100cf", "somefile"]).unwrap();
    assert_eq!(s.number, SigNum::Negative(100));
    assert_eq!(s.mode, Mode::Bytes);
    assert_eq!(s.follow, Some(FollowMode::Descriptor));
}