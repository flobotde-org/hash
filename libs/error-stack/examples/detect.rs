#![expect(clippy::print_stdout, clippy::use_debug)]
// This example shows how you can use the `supports-color` and `supports-unicode` crates to
// automatically enable or disable the color mode and set the appropriate charset.
// Your default terminal should automatically show colored unicode output, to emulate a terminal
// that does not support color set your `$TERM` env-variable **temporarily** to "dumb", or set the
// env-variable `NO_COLOR=1`. To emulate no-unicode support set your `$TERM` variable
// **temporarily** to `linux`.

use core::fmt::{Display, Formatter};
use std::path::Path;

use error_stack::{
    Report,
    fmt::{Charset, ColorMode},
};

type Config = String;

#[derive(Debug)]
struct ParseConfigError;

impl Display for ParseConfigError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> core::fmt::Result {
        fmt.write_str("unable to parse config")
    }
}

impl core::error::Error for ParseConfigError {}

fn parse_config(path: impl AsRef<Path>) -> Result<Config, Report<ParseConfigError>> {
    _ = path.as_ref();

    /*
       usually you would actually do something here, we just error out, for a more complete example
       check out the other examples
    */

    Err(Report::new(ParseConfigError).attach_printable("unable to read configuration"))
}

fn main() {
    // error-stack only uses ANSI codes for colors
    let supports_color = supports_color::on_cached(supports_color::Stream::Stdout)
        .is_some_and(|level| level.has_basic);

    let color_mode = if supports_color {
        ColorMode::Color
    } else {
        ColorMode::None
    };

    let supports_unicode = supports_unicode::on(supports_unicode::Stream::Stdout);

    let charset = if supports_unicode {
        Charset::Utf8
    } else {
        Charset::Ascii
    };

    Report::set_color_mode(color_mode);
    Report::set_charset(charset);

    if let Err(err) = parse_config("config.json") {
        // if you would use `eprintln!` instead, you should check support on `Stream::Stderr`
        // instead.
        println!("{err:?}");
    }
}
