# formatter-kotlin

A pragmatic Kotlin source formatter built on [Topiary](https://topiary.tweag.io/).
Parses Kotlin with [`tree-sitter-kotlin-ng`](https://crates.io/crates/tree-sitter-kotlin-ng)
and applies the tree-sitter-query rules in [`queries/kotlin.scm`](queries/kotlin.scm).

## What it formats

Covered:

- `package` / `import` headers
- classes, objects, interfaces, enum classes (primary constructors, inheritance,
  members)
- function declarations (parameters, return type, block or `=` body)
- property declarations (`val` / `var`, with or without type, default values)
- control flow: `if` / `else`, `when`, `for`, `while`, `do-while`, `try / catch / finally`
- binary / comparison / logical / elvis operators, lambda arrows, assignments
- lambda literals
- line and block comments (preserved as leaves)

Not in scope: annotation folding, KDoc reflow, import ordering, trailing-comma
policy, line-length-aware wrapping, expression chaining.

## Library usage

```rust
use formatter_kotlin::{format, format_with, FormatOptions};

let src = "fun add(a:Int,b:Int):Int{return a+b}";
let formatted = format(src).unwrap();
assert_eq!(
    formatted,
    "fun add(a: Int, b: Int): Int {\n    return a + b\n}\n"
);

// Custom indent / tolerate parse errors / skip idempotence check:
let opts = FormatOptions {
    indent: "  ".to_owned(),
    skip_idempotence: true,
    tolerate_parsing_errors: false,
};
let _ = format_with(src, &opts).unwrap();
```

## CLI

```
# stdin -> stdout
cat Main.kt | cargo run --quiet --bin formatter-kotlin

# file path -> stdout
cargo run --quiet --bin formatter-kotlin -- Main.kt
```

Exit codes: `0` on success, `1` on format error, `2` on I/O error.

## Tests

```
cargo test
```

There are unit tests in `src/lib.rs` (query-compilation, trivial inputs) and
26 integration tests in `tests/format.rs` covering every formatting construct
listed above. Each integration test checks both correctness and idempotence.

### ktfmt corpus coverage

This project's long-term goal is to fully replace
[ktfmt](https://github.com/facebook/ktfmt), so ktfmt is vendored as a git
submodule at `third_party/ktfmt` and its test suite is reused as a Kotlin
language-coverage benchmark:

```
git submodule update --init --recursive
cargo test --test ktfmt_corpus -- --nocapture
```

`tests/ktfmt_corpus.rs` scans ktfmt's JUnit tests for every
`assertFormatted(...)` snippet (~425 snippets across `FormatterTest.kt` and
`GoogleStyleFormatterKtTest.kt`), runs each one through `format_with`, and
prints a per-file and total pass rate to stderr. A snippet counts as "ok"
when our pipeline parses and formats it without error — output parity with
ktfmt is not required because ktfmt does column-aware wrapping and we do not.
The test only asserts that the corpus is non-empty and at least one snippet
runs through cleanly; concrete pass rates are reported, not gated, so the
number can climb as the formatter improves without churning the test.
