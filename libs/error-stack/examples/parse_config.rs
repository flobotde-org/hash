#![expect(clippy::print_stderr, unreachable_pub, clippy::use_debug)]
// This is the same example also used in `lib.rs`. When updating this, don't forget updating the doc
// example as well. This example is mainly used to generate the output shown in the documentation.

use core::{error::Error, fmt};
use std::{fs, path::Path};

use error_stack::{Report, ResultExt as _};

pub type Config = String;

#[derive(Debug)]
struct ParseConfigError;

impl ParseConfigError {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl fmt::Display for ParseConfigError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Could not parse configuration file")
    }
}

impl Error for ParseConfigError {}

struct Suggestion(&'static str);

fn parse_config(path: impl AsRef<Path>) -> Result<Config, Report<ParseConfigError>> {
    let path = path.as_ref();

    let content = fs::read_to_string(path)
        .change_context(ParseConfigError::new())
        .attach(Suggestion("use a file you can read next time!"))
        .attach_printable_lazy(|| format!("could not read file {}", path.display()))?;

    Ok(content)
}

fn main() {
    if let Err(report) = parse_config("config.json") {
        eprintln!("{report:?}");
        #[cfg(nightly)]
        for suggestion in report.request_ref::<Suggestion>() {
            eprintln!("Suggestion: {}", suggestion.0);
        }
    }
}
