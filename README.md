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
