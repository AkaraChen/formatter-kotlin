//! Integration tests for the Kotlin formatter.
//!
//! Each test case supplies an `input` and an `expected` string. The harness:
//!   1. formats `input` and compares against `expected` (via pretty_assertions),
//!   2. formats the output a second time and confirms idempotence.
//!
//! Keeping both checks in one place means a regression in either direction
//! (correctness or stability) fails the test with a useful diff.

use formatter_kotlin::format;
use pretty_assertions::assert_eq;

fn check(name: &str, input: &str, expected: &str) {
    let actual = format(input).unwrap_or_else(|e| panic!("[{name}] format failed: {e}"));
    assert_eq!(expected, actual, "[{name}] formatter output did not match expected");

    let twice = format(&actual).unwrap_or_else(|e| panic!("[{name}] second format failed: {e}"));
    assert_eq!(actual, twice, "[{name}] formatter output is not idempotent");
}

// ---------------------------------------------------------------------------
// Top-level: package, imports, blank handling
// ---------------------------------------------------------------------------

#[test]
fn package_and_imports() {
    let input = "package com.example.app\nimport kotlin.math.PI\nimport kotlin.math.E\n";
    let expected = "package com.example.app\nimport kotlin.math.PI\nimport kotlin.math.E\n";
    check("package_and_imports", input, expected);
}

#[test]
fn blank_line_between_imports_and_body_is_preserved() {
    let input = "\
package demo

import kotlin.math.PI

fun main() {}
";
    let expected = "\
package demo

import kotlin.math.PI

fun main() {}
";
    check("blank_line_preserved", input, expected);
}

// ---------------------------------------------------------------------------
// Function declarations
// ---------------------------------------------------------------------------

#[test]
fn simple_function_without_body() {
    let input = "fun noop():Unit{}\n";
    let expected = "fun noop(): Unit {}\n";
    check("simple_function_without_body", input, expected);
}

#[test]
fn function_with_parameters_and_return_type() {
    let input = "fun add(a:Int,b:Int):Int{return a+b}\n";
    let expected = "\
fun add(a: Int, b: Int): Int {
    return a + b
}
";
    check("function_with_params", input, expected);
}

#[test]
fn function_with_multiple_statements() {
    let input = "\
fun greet(name:String){
val prefix=\"hi\"
println(\"$prefix, $name!\")
}
";
    let expected = "\
fun greet(name: String) {
    val prefix = \"hi\"
    println(\"$prefix, $name!\")
}
";
    check("function_multiple_statements", input, expected);
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

#[test]
fn val_with_type_annotation() {
    let input = "val answer:Int=42\n";
    let expected = "val answer: Int = 42\n";
    check("val_with_type", input, expected);
}

#[test]
fn var_without_type() {
    let input = "var counter=0\n";
    let expected = "var counter = 0\n";
    check("var_inferred", input, expected);
}

// ---------------------------------------------------------------------------
// Class / object / interface
// ---------------------------------------------------------------------------

#[test]
fn class_with_primary_constructor() {
    let input = "class Point(val x:Int,val y:Int)\n";
    let expected = "class Point(val x: Int, val y: Int)\n";
    check("class_primary_ctor", input, expected);
}

#[test]
fn class_with_body_and_members() {
    let input = "\
class Counter(private var n:Int=0){
fun inc(){n++}
fun value():Int=n
}
";
    let expected = "\
class Counter(private var n: Int = 0) {
    fun inc() {
        n++
    }
    fun value(): Int = n
}
";
    check("class_with_body", input, expected);
}

#[test]
fn class_with_inheritance() {
    let input = "class Dog(name:String):Animal(name),Barking{}\n";
    let expected = "class Dog(name: String) : Animal(name), Barking {}\n";
    check("class_inheritance", input, expected);
}

#[test]
fn object_declaration() {
    let input = "object Singleton{fun go(){println(\"hi\")}}\n";
    let expected = "\
object Singleton {
    fun go() {
        println(\"hi\")
    }
}
";
    check("object_declaration", input, expected);
}

#[test]
fn interface_declaration() {
    let input = "interface Foo{fun bar():Int}\n";
    let expected = "\
interface Foo {
    fun bar(): Int
}
";
    check("interface_declaration", input, expected);
}

// ---------------------------------------------------------------------------
// Enum
// ---------------------------------------------------------------------------

#[test]
fn enum_class_entries() {
    let input = "enum class Color{RED,GREEN,BLUE}\n";
    let expected = "\
enum class Color {
    RED,
    GREEN,
    BLUE
}
";
    check("enum_entries", input, expected);
}

// ---------------------------------------------------------------------------
// Control flow
// ---------------------------------------------------------------------------

#[test]
fn if_else_expression() {
    let input = "\
fun sign(n:Int):String{
if(n>0){return \"+\"}else{return \"-\"}
}
";
    let expected = "\
fun sign(n: Int): String {
    if (n > 0) {
        return \"+\"
    } else {
        return \"-\"
    }
}
";
    check("if_else", input, expected);
}

#[test]
fn when_expression() {
    let input = "\
fun label(n:Int):String=when{
n<0->\"neg\"
n==0->\"zero\"
else->\"pos\"
}
";
    let expected = "\
fun label(n: Int): String = when {
    n < 0 -> \"neg\"
    n == 0 -> \"zero\"
    else -> \"pos\"
}
";
    check("when_expression", input, expected);
}

#[test]
fn for_and_while_loops() {
    let input = "\
fun loops(){
for(i in 0..3){println(i)}
var j=0
while(j<3){j++}
do{j--}while(j>0)
}
";
    let expected = "\
fun loops() {
    for (i in 0..3) {
        println(i)
    }
    var j = 0
    while (j < 3) {
        j++
    }
    do {
        j--
    } while (j > 0)
}
";
    check("for_and_while", input, expected);
}

#[test]
fn try_catch_finally() {
    let input = "\
fun guarded(){
try{risky()}catch(e:Exception){println(e.message)}finally{cleanup()}
}
";
    let expected = "\
fun guarded() {
    try {
        risky()
    } catch (e: Exception) {
        println(e.message)
    } finally {
        cleanup()
    }
}
";
    check("try_catch_finally", input, expected);
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

#[test]
fn binary_operators_are_spaced() {
    let input = "val x=1+2*3-4/2%5\n";
    let expected = "val x = 1 + 2 * 3 - 4 / 2 % 5\n";
    check("binary_ops", input, expected);
}

#[test]
fn comparison_and_logical_operators() {
    let input = "val cond=a==b&&c!=d||!e\n";
    let expected = "val cond = a == b && c != d || !e\n";
    check("comparison_logical", input, expected);
}

#[test]
fn elvis_and_safe_calls_preserve_structure() {
    // We don't force spaces around `?:` here because the operator can be
    // used either way; but tight dotted access stays tight.
    let input = "val v = x?.y ?: defaultValue\n";
    let expected = "val v = x?.y ?: defaultValue\n";
    check("elvis", input, expected);
}

// ---------------------------------------------------------------------------
// Strings, comments
// ---------------------------------------------------------------------------

#[test]
fn string_content_is_preserved() {
    let input = "val s = \"  keep   me   as-is  \"\n";
    let expected = "val s = \"  keep   me   as-is  \"\n";
    check("string_preserved", input, expected);
}

#[test]
fn line_comment_preserved() {
    let input = "\
// header comment
fun main() {
    // inside fn
    println(\"ok\")
}
";
    let expected = "\
// header comment
fun main() {
    // inside fn
    println(\"ok\")
}
";
    check("line_comment", input, expected);
}

#[test]
fn block_comment_preserved_with_indent() {
    let input = "\
/* top-level
   doc
*/
fun main() {}
";
    let expected = "\
/* top-level
   doc
*/
fun main() {}
";
    check("block_comment", input, expected);
}

// ---------------------------------------------------------------------------
// Lambdas
// ---------------------------------------------------------------------------

#[test]
fn lambda_arrow_spacing() {
    let input = "val sq:(Int)->Int={x->x*x}\n";
    let expected = "val sq: (Int) -> Int = { x -> x * x }\n";
    check("lambda_arrow", input, expected);
}

// ---------------------------------------------------------------------------
// Idempotence on bulky, mixed input
// ---------------------------------------------------------------------------

#[test]
fn kitchen_sink_idempotent() {
    let input = "\
package com.example

import kotlin.math.sqrt

/**
 * A tiny geometry helper.
 */
class Point(val x: Double, val y: Double) {
    fun distanceTo(other: Point): Double {
        val dx = x - other.x
        val dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

fun classify(n: Int): String = when {
    n < 0 -> \"negative\"
    n == 0 -> \"zero\"
    else -> \"positive\"
}
";
    // The harness's second call already asserts idempotence; we just need
    // the first output to round-trip unchanged.
    let first = format(input).expect("format succeeds");
    let second = format(&first).expect("format succeeds again");
    assert_eq!(first, second, "kitchen-sink output was not idempotent");
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[test]
fn parsing_error_reports_failure() {
    // Completely un-parseable: stray braces and keywords.
    let input = "}}}} fun ( {{{ class";
    let result = format(input);
    assert!(
        result.is_err(),
        "expected a parse error, got Ok:\n{}",
        result.unwrap_or_default()
    );
}
