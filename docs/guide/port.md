# Porting from Clap

This chapter contains information about how common patterns in `clap` parsers can be ported to `uutils-args`.

More examples can be added here while we figure out more common patterns.

## Defaults

By default, the `clap` command roughly equivalent to a command from `uutils-args` looks like this (where everything with `...` is filled in automatically).

```rust,ignore
Command::new(...)
    .version(...)
    .override_usage(...)
    .about(...)
    .infer_long_args(true)
    .args_override_self(true)
    .disable_help_flag(true)
    .disable_version_flag(true)
    .arg(
        Arg::new("help")
            .long("help")
            .help("Print help information.")
            .action(ArgAction::Help),
    )
    .arg(
        Arg::new("version")
            .long("version")
            .help("Print version information.")
            .action(ArgAction::Version),
    )
```

Further differences are:

- Overrides are the default in `uutils-args`. There is no automatic conflict checking.
- Values can always start with hyphens.
- Long flags with optional arguments always require an equal sign.

## `ArgAction` equivalents

### `ArgAction::SetTrue`

```rust,ignore
let command = Command::new(...)
    .arg(
        Arg::new("a")
            .short('a')
            .action(ArgAction::SetTrue)
    );

let matches = command.get_matches();

let a = matches.get_flag("a");
```

```rust
use uutils_args::{Arguments, Options};

#[derive(Arguments)]
enum Arg {
    #[arg("-a")]
    A
}

#[derive(Default)]
struct Settings { a: bool }

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::A => self.a = true,
        }
    }
}

let a = Settings::default().parse(std::env::args_os()).a;
```

### `ArgAction::SetFalse`

```rust,ignore
let command = Command::new(...)
    .arg(
        Arg::new("a")
            .short('a')
            .action(ArgAction::SetFalse)
    );

let matches = command.get_matches();

let a = matches.get_flag("a");
```

```rust
use uutils_args::{Arguments, Options};

#[derive(Arguments)]
enum Arg {
    #[arg("-a")]
    A
}

struct Settings { a: bool }

impl Default for Settings {
    fn default() -> Self {
        Self { a: false }
    }
}

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::A => self.a = false,
        }
    }
}

let a = Settings::default().parse(std::env::args_os()).a;
```

### `ArgAction::Count`

```rust,ignore
let command = Command::new(...)
    .arg(
        Arg::new("a")
            .short('a')
            .action(ArgAction::Count)
    );

let matches = command.get_matches();

let a = matches.get_one("a").unwrap();
```

```rust
use uutils_args::{Arguments, Options};

#[derive(Arguments)]
enum Arg {
    #[arg("-a")]
    A
}

#[derive(Default)]
struct Settings { a: u8 }

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::A => self.a += 1,
        }
    }
}

let a = Settings::default().parse(std::env::args_os()).a;
```

### `ArgAction::Set`

```rust,ignore
let command = Command::new(...)
    .arg(
        Arg::new("a")
            .short('a')
            .action(ArgAction::Set)
            .value_name("VAL")
    );

let matches = command.get_matches();

let a = matches.get_one("a").unwrap();
```

```rust
use uutils_args::{Arguments, Options};
use std::ffi::OsString;

#[derive(Arguments)]
enum Arg {
    #[arg("-a VAL")]
    A(OsString)
}

#[derive(Default)]
struct Settings { a: OsString }

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::A(s) => self.a = s,
        }
    }
}

let a = Settings::default().parse(std::env::args_os()).a;
```

### `ArgAction::Append`

```rust,ignore
let command = Command::new(...)
    .arg(
        Arg::new("a")
            .short('a')
            .action(ArgAction::Append)
            .value_name("VAL")
    );

let matches = command.get_matches();

let a = matches.get_one("a").unwrap();
```

```rust
use uutils_args::{Arguments, Options};
use std::ffi::OsString;

#[derive(Arguments)]
enum Arg {
    #[arg("-a VAL")]
    A(OsString)
}

#[derive(Default)]
struct Settings { a: Vec<OsString> }

impl Options<Arg> for Settings {
    fn apply(&mut self, arg: Arg) {
        match arg {
            Arg::A(s) => self.a.push(s),
        }
    }
}

let a = Settings::default().parse(std::env::args_os()).a;
```
