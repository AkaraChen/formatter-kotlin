//! Run ktfmt's test corpus through formatter-kotlin to measure how much
//! of the Kotlin language our formatter actually handles.
//!
//! ktfmt (https://github.com/facebook/ktfmt) is vendored as a git submodule at
//! `third_party/ktfmt`. Its test suite (notably `FormatterTest.kt` and
//! `GoogleStyleFormatterKtTest.kt`) contains ~1000 `assertFormatted(...)` calls
//! where the string argument is both the input and the expected output of a
//! ktfmt-idempotent format. That gives us a huge corpus of known-good,
//! syntactically interesting Kotlin snippets to run through our formatter.
//!
//! Since our goal is eventually to replace ktfmt, this test doesn't demand
//! byte-for-byte output parity with ktfmt (ktfmt does column-aware wrapping,
//! we do not). Instead, it measures syntactic coverage:
//!   - `format()` returns Ok  → we parsed and formatted the snippet
//!   - `format()` returns Err → the snippet breaks our pipeline
//!
//! The test emits a report to stderr with per-file and overall counts so it's
//! easy to see at a glance what fraction of ktfmt's corpus we already support.
//! It asserts only that we extracted some snippets and that at least one runs
//! through cleanly — concrete pass rates aren't gated, so the number can grow
//! as the formatter improves without churning this file.

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
                if !snippet.trim().is_empty() {
                    out.push(snippet);
                }
                cursor = after_close;
                continue;
            }
        } else if src[body_start..].starts_with('"') {
            let open = body_start + 1;
            if let Some(lit) = parse_kotlin_string_literal(&src[open..]) {
                if !lit.trim().is_empty() {
                    out.push(lit);
                }
            }
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

#[derive(Default)]
struct Stats {
    total: usize,
    ok: usize,
    err: usize,
}

impl Stats {
    fn record(&mut self, result: Result<String, impl std::fmt::Display>) -> bool {
        self.total += 1;
        match result {
            Ok(_) => {
                self.ok += 1;
                true
            }
            Err(_) => {
                self.err += 1;
                false
            }
        }
    }
}

fn run_one(snippet: &str) -> Result<String, formatter_kotlin::FormatError> {
    let opts = FormatOptions {
        indent: "  ".to_owned(), // ktfmt uses 2-space indent
        skip_idempotence: true,
        tolerate_parsing_errors: false,
    };
    format_with(snippet, &opts)
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
    let mut first_failure: Option<(PathBuf, String, String)> = None;
    let mut file_reports: Vec<(String, Stats, Vec<String>)> = Vec::new();

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
        let mut failing_samples: Vec<String> = Vec::new();
        for snippet in &snippets {
            let result = run_one(snippet);
            let ok = file_stats.record(result.as_ref().map(|s| s.clone()));
            if !ok {
                if failing_samples.len() < 3 {
                    let preview = snippet.lines().take(4).collect::<Vec<_>>().join("\n");
                    failing_samples.push(preview);
                }
                if first_failure.is_none() {
                    let err = run_one(snippet).unwrap_err().to_string();
                    first_failure = Some((path.clone(), snippet.clone(), err));
                }
            }
        }
        overall.total += file_stats.total;
        overall.ok += file_stats.ok;
        overall.err += file_stats.err;
        file_reports.push((
            filename_or_path(&path),
            file_stats,
            failing_samples,
        ));
    }

    eprintln!();
    eprintln!("=== ktfmt corpus coverage ===");
    for (name, s, _) in &file_reports {
        let pct = if s.total == 0 {
            0.0
        } else {
            100.0 * (s.ok as f64) / (s.total as f64)
        };
        eprintln!(
            "  {:<40} {:>4}/{:<4} ok  ({:5.1}%)",
            name, s.ok, s.total, pct
        );
    }
    let pct = if overall.total == 0 {
        0.0
    } else {
        100.0 * (overall.ok as f64) / (overall.total as f64)
    };
    eprintln!(
        "  {:-<40} {:>4}/{:<4} ok  ({:5.1}%)",
        "total ", overall.ok, overall.total, pct
    );

    if let Some((path, snippet, err)) = &first_failure {
        eprintln!();
        eprintln!("first failing snippet comes from {}:", filename_or_path(path));
        eprintln!("  error: {err}");
        eprintln!("  snippet (first 6 lines):");
        for line in snippet.lines().take(6) {
            eprintln!("  | {line}");
        }
    }

    assert!(
        overall.total > 0,
        "extracted no snippets from ktfmt corpus — extractor is broken"
    );
    assert!(
        overall.ok > 0,
        "not a single ktfmt snippet made it through our formatter"
    );
}

fn filename_or_path(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| p.display().to_string())
}
