//! Run ktfmt's test corpus through formatter-kotlin to measure how close we
//! are to being a drop-in ktfmt replacement.
//!
//! ktfmt (https://github.com/facebook/ktfmt) is vendored as a git submodule at
//! `third_party/ktfmt`. Its test suite (notably `FormatterTest.kt` and
//! `GoogleStyleFormatterKtTest.kt`) contains hundreds of `assertFormatted(s)`
//! calls where `s` is both input *and* expected output — i.e. ktfmt is
//! idempotent on `s`. That gives us a large corpus of known-good Kotlin where
//! the ktfmt-correct answer is simply `s` itself.
//!
//! Since the project's goal is to replace ktfmt, the bar here is output
//! parity. Each snippet falls into one of three buckets:
//!   - `parity`   — `format(s) == s`, i.e. we match ktfmt exactly
//!   - `mismatch` — we formatted without error but produced different output
//!   - `error`    — our pipeline refused the snippet outright
//!
//! The test asserts only that we extracted some snippets and that at least
//! one reaches parity. Concrete pass rates aren't gated, so the numbers can
//! grow as the formatter improves without churning this file.

use std::fs;
use std::path::{Path, PathBuf};

use formatter_kotlin::{FormatOptions, format_with};

fn ktfmt_format_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("third_party/ktfmt/core/src/test/java/com/facebook/ktfmt/format")
}

/// Extract every `assertFormatted(...)` snippet from a ktfmt test source file.
///
/// Two shapes are recognised:
///   1. `assertFormatted("literal\n")` — simple single-string argument.
///   2. `assertFormatted("""<body>""".trimMargin())` — the dominant pattern;
///      each body line is prefixed with `|` after the indentation.
///
/// The ktfmt test suite defines `private const val TQ = "\"\"\""` and splices
/// it into snippets via the Kotlin string-template forms `$TQ` / `${TQ}` so
/// each snippet can itself contain a triple-quoted string. We evaluate that
/// splice here — the compiler would do it at runtime, and without it the
/// snippet isn't valid Kotlin.
///
/// When `deduceMaxWidth = true` is passed, the first line of the snippet is a
/// row of `/` or `-` characters that encodes the desired wrap column — it
/// is not source code, so we strip it before returning.
fn extract_snippets(src: &str) -> Vec<String> {
    let marker = "assertFormatted(";
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(rel) = src[cursor..].find(marker) {
        let after = cursor + rel + marker.len();
        // Skip whitespace (spaces, newlines, and leading-margin pipes from
        // Kotlin source indentation) after the opening paren.
        let body_start = after + src[after..]
            .bytes()
            .take_while(|b| b.is_ascii_whitespace())
            .count();
        cursor = after;

        if src[body_start..].starts_with("\"\"\"") {
            let open = body_start + 3;
            if let Some(close_rel) = src[open..].find("\"\"\"") {
                let raw = &src[open..open + close_rel];
                // After `"""` there must be a `.trimMargin(` call for the
                // margin-stripping logic to apply; otherwise we treat the
                // body as already-literal and preserve it verbatim.
                let after_close = open + close_rel + 3;
                let rest = src[after_close..].trim_start();
                let snippet = if rest.starts_with(".trimMargin") {
                    strip_trim_margin(raw)
                } else {
                    raw.to_owned()
                };
                let snippet = strip_width_marker_line(&snippet);
                let snippet = expand_tq_templates(&snippet);
                if !snippet.trim().is_empty() {
                    out.push(snippet);
                }
                cursor = after_close;
                continue;
            }
        } else if src[body_start..].starts_with('"') {
            let open = body_start + 1;
            if let Some(lit) = parse_kotlin_string_literal(&src[open..]) {
                let lit = expand_tq_templates(&lit);
                if !lit.trim().is_empty() {
                    out.push(lit);
                }
            }
        }
    }
    out
}

/// Evaluate the `$TQ` / `${TQ}` Kotlin string-template references used by
/// ktfmt's test fixtures. `TQ` is always the literal `"""`; `\$` stays a
/// literal dollar sign and is left alone.
fn expand_tq_templates(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with("\\$") {
            out.push_str("\\$");
            i += 2;
        } else if rest.starts_with("${TQ}") {
            out.push_str("\"\"\"");
            i += "${TQ}".len();
        } else if rest.starts_with("$TQ") {
            out.push_str("\"\"\"");
            i += "$TQ".len();
        } else {
            let c = rest.chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// Reproduce `String.trimMargin("|")` from Kotlin: for each line, drop leading
/// whitespace and a single following `|`; lines without the marker are kept
/// unchanged. The first and last lines are dropped if blank.
fn strip_trim_margin(raw: &str) -> String {
    let mut lines: Vec<&str> = raw.lines().collect();
    if lines.first().map_or(false, |l| l.trim().is_empty()) {
        lines.remove(0);
    }
    if lines.last().map_or(false, |l| l.trim().is_empty()) {
        lines.pop();
    }
    lines
        .into_iter()
        .map(|line| {
            let trimmed = line.trim_start();
            trimmed.strip_prefix('|').unwrap_or(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// If the first line is a ktfmt width marker (>= 8 `/` or `-` chars), drop it.
fn strip_width_marker_line(s: &str) -> String {
    let (first, rest) = match s.split_once('\n') {
        Some(pair) => pair,
        None => return s.to_owned(),
    };
    let is_marker = first.len() >= 8 && first.chars().all(|c| c == '/' || c == '-');
    if is_marker {
        rest.to_owned()
    } else {
        s.to_owned()
    }
}

/// Parse a single-quoted Kotlin string literal, processing standard escapes,
/// returning its content. `src` must start immediately after the opening quote.
fn parse_kotlin_string_literal(src: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = src.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '\\' => out.push('\\'),
                '"' => out.push('"'),
                '\'' => out.push('\''),
                '$' => out.push('$'),
                // Unsupported escape: give up on this snippet.
                _ => return None,
            },
            '\n' => return None, // unterminated literal
            _ => out.push(c),
        }
    }
    None
}

#[derive(Default, Clone, Copy)]
struct Stats {
    total: usize,
    /// `format(s) == s` — exact parity with ktfmt on this snippet.
    parity: usize,
    /// Formatter produced output, but it differs from ktfmt's.
    mismatch: usize,
    /// Formatter refused the snippet.
    error: usize,
}

impl Stats {
    fn record(&mut self, outcome: Outcome) {
        self.total += 1;
        match outcome {
            Outcome::Parity => self.parity += 1,
            Outcome::Mismatch => self.mismatch += 1,
            Outcome::Error => self.error += 1,
        }
    }

    /// Parses cleanly — parity or mismatch, i.e. not an error.
    fn parse_ok(&self) -> usize {
        self.parity + self.mismatch
    }
}

#[derive(Clone, Copy)]
enum Outcome {
    Parity,
    Mismatch,
    Error,
}

fn run_one(snippet: &str) -> Outcome {
    let opts = FormatOptions {
        indent: "  ".to_owned(), // ktfmt uses 2-space indent
        skip_idempotence: true,
        tolerate_parsing_errors: false,
    };
    match format_with(snippet, &opts) {
        Ok(out) if out == snippet => Outcome::Parity,
        Ok(_) => Outcome::Mismatch,
        Err(_) => Outcome::Error,
    }
}

#[test]
fn ktfmt_corpus_coverage() {
    let dir = ktfmt_format_test_dir();
    if !dir.exists() {
        panic!(
            "ktfmt submodule not initialised at {}; run `git submodule update --init --recursive`.",
            dir.display()
        );
    }

    let mut overall = Stats::default();
    let mut first_error: Option<(PathBuf, String, String)> = None;
    let mut first_mismatch: Option<(PathBuf, String, String)> = None;
    let mut file_reports: Vec<(String, Stats)> = Vec::new();

    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("read ktfmt format test dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("kt"))
        .collect();
    paths.sort();

    for path in paths {
        let src = fs::read_to_string(&path).expect("read ktfmt test file");
        let snippets = extract_snippets(&src);
        let mut file_stats = Stats::default();
        for snippet in &snippets {
            let outcome = run_one(snippet);
            file_stats.record(outcome);
            match outcome {
                Outcome::Error if first_error.is_none() => {
                    let opts = FormatOptions {
                        indent: "  ".to_owned(),
                        skip_idempotence: true,
                        tolerate_parsing_errors: false,
                    };
                    let err = format_with(snippet, &opts).unwrap_err().to_string();
                    first_error = Some((path.clone(), snippet.clone(), err));
                }
                Outcome::Mismatch if first_mismatch.is_none() => {
                    let opts = FormatOptions {
                        indent: "  ".to_owned(),
                        skip_idempotence: true,
                        tolerate_parsing_errors: false,
                    };
                    let out = format_with(snippet, &opts).unwrap();
                    first_mismatch = Some((path.clone(), snippet.clone(), out));
                }
                _ => {}
            }
        }
        overall.total += file_stats.total;
        overall.parity += file_stats.parity;
        overall.mismatch += file_stats.mismatch;
        overall.error += file_stats.error;
        file_reports.push((filename_or_path(&path), file_stats));
    }

    eprintln!();
    eprintln!("=== ktfmt corpus parity ===");
    eprintln!(
        "  {:<40} {:>5}  {:>5}  {:>5}  {:>5}  {:>7}",
        "file", "total", "par", "mism", "err", "par%"
    );
    for (name, s) in &file_reports {
        let pct = percent(s.parity, s.total);
        eprintln!(
            "  {:<40} {:>5}  {:>5}  {:>5}  {:>5}  {:>6.1}%",
            name, s.total, s.parity, s.mismatch, s.error, pct
        );
    }
    let pct = percent(overall.parity, overall.total);
    let parse_pct = percent(overall.parse_ok(), overall.total);
    eprintln!(
        "  {:-<40} {:>5}  {:>5}  {:>5}  {:>5}  {:>6.1}%",
        "total ", overall.total, overall.parity, overall.mismatch, overall.error, pct
    );
    eprintln!(
        "  (parse-ok = parity + mismatch = {}/{} = {:.1}%)",
        overall.parse_ok(),
        overall.total,
        parse_pct,
    );

    if let Some((path, snippet, err)) = &first_error {
        eprintln!();
        eprintln!("first erroring snippet comes from {}:", filename_or_path(path));
        eprintln!("  error: {err}");
        eprintln!("  snippet (first 6 lines):");
        for line in snippet.lines().take(6) {
            eprintln!("  | {line}");
        }
    }
    if let Some((path, snippet, out)) = &first_mismatch {
        eprintln!();
        eprintln!("first mismatching snippet comes from {}:", filename_or_path(path));
        eprintln!("  input (first 6 lines):");
        for line in snippet.lines().take(6) {
            eprintln!("  | {line}");
        }
        eprintln!("  our output (first 6 lines):");
        for line in out.lines().take(6) {
            eprintln!("  | {line}");
        }
    }

    assert!(
        overall.total > 0,
        "extracted no snippets from ktfmt corpus — extractor is broken"
    );
    assert!(
        overall.parity > 0,
        "not a single ktfmt snippet reached output parity with our formatter"
    );
}

fn percent(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { 100.0 * (n as f64) / (d as f64) }
}

fn filename_or_path(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| p.display().to_string())
}
