//! Kotlin source formatter built on Topiary.
//!
//! The primary entry point is [`format`]. It parses Kotlin source with
//! `tree-sitter-kotlin-ng`, runs Topiary's formatting engine with the bundled
//! `queries/kotlin.scm` rule set, and returns the formatted output.

use std::io::Cursor;

use thiserror::Error;
use topiary_core::{Language, Operation, TopiaryQuery, formatter};

/// Errors returned by [`format`].
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("failed to build Kotlin language definition: {0}")]
    Language(String),
    #[error("formatting failed: {0}")]
    Formatter(String),
    #[error("formatted output was not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

const KOTLIN_QUERY: &str = include_str!("../queries/kotlin.scm");
const DEFAULT_INDENT: &str = "    ";

/// Formatting options.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Indentation string. Defaults to four spaces.
    pub indent: String,
    /// If `true`, Topiary skips its built-in idempotence check (format twice,
    /// compare). The default (`false`) enables the check, which catches query
    /// regressions but adds one extra parse+format pass per call.
    pub skip_idempotence: bool,
    /// If `true`, Topiary continues through parse errors by treating the
    /// erroring node as a leaf. Useful for partial / editor-buffer input.
    pub tolerate_parsing_errors: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: DEFAULT_INDENT.to_owned(),
            skip_idempotence: false,
            tolerate_parsing_errors: false,
        }
    }
}

/// Build the Topiary [`Language`] for Kotlin.
fn build_language(indent: String) -> Result<Language, FormatError> {
    let grammar =
        topiary_tree_sitter_facade::Language::from(tree_sitter_kotlin_ng::LANGUAGE);
    let query = TopiaryQuery::new(&grammar.clone().into(), KOTLIN_QUERY)
        .map_err(|e| FormatError::Language(e.to_string()))?;
    Ok(Language {
        name: "kotlin".to_owned(),
        query,
        grammar,
        indent: Some(indent),
    })
}

/// Format the given Kotlin source using the default options.
pub fn format(source: &str) -> Result<String, FormatError> {
    format_with(source, &FormatOptions::default())
}

/// Format Kotlin source with caller-supplied options.
pub fn format_with(source: &str, opts: &FormatOptions) -> Result<String, FormatError> {
    let language = build_language(opts.indent.clone())?;

    let mut input = Cursor::new(source.as_bytes());
    let mut output: Vec<u8> = Vec::with_capacity(source.len());

    formatter(
        &mut input,
        &mut output,
        &language,
        Operation::Format {
            skip_idempotence: opts.skip_idempotence,
            tolerate_parsing_errors: opts.tolerate_parsing_errors,
        },
    )
    .map_err(|e| FormatError::Formatter(e.to_string()))?;

    Ok(String::from_utf8(output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_compiles() {
        build_language(DEFAULT_INDENT.to_owned()).expect("kotlin.scm compiles against the grammar");
    }

    #[test]
    fn empty_file_is_blank_line() {
        let out = format("").expect("format empty");
        assert_eq!(out, "\n");
    }

    #[test]
    fn trailing_newline_is_added() {
        let out = format("val x = 1").expect("format");
        assert!(out.ends_with('\n'));
    }
}
