//! Small CLI: reads Kotlin source from stdin (or a file path argument) and
//! writes the formatted result to stdout.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use formatter_kotlin::{FormatOptions, format_with};

fn read_source(arg: Option<&str>) -> io::Result<String> {
    match arg {
        Some(path) => std::fs::read_to_string(PathBuf::from(path)),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let source = match read_source(arg.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(io::stderr(), "failed to read input: {e}");
            return ExitCode::from(2);
        }
    };

    let opts = FormatOptions::default();
    match format_with(&source, &opts) {
        Ok(formatted) => {
            if let Err(e) = io::stdout().write_all(formatted.as_bytes()) {
                let _ = writeln!(io::stderr(), "failed to write output: {e}");
                return ExitCode::from(2);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            let _ = writeln!(io::stderr(), "format error: {e}");
            ExitCode::from(1)
        }
    }
}
