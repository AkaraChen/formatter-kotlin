; Kotlin formatting rules for Topiary.
;
; Scope: package / imports, class / object / interface / function / property
; declarations, basic control flow, operators, modifiers, blocks and
; lambdas. Comments and strings are preserved as leaves.

; --- Leaves ----------------------------------------------------------------

[
  (string_literal)
  (multiline_string_literal)
  (character_literal)
  (line_comment)
  (block_comment)
] @leaf

; --- Blank line policy -----------------------------------------------------

[
  (import)
  (class_declaration)
  (object_declaration)
  (function_declaration)
  (property_declaration)
  (type_alias)
  (secondary_constructor)
  (anonymous_initializer)
  (companion_object)
  (enum_entry)
  (line_comment)
  (block_comment)
] @allow_blank_line_before

; --- Comments --------------------------------------------------------------

(line_comment) @append_hardline

[
  (line_comment)
  (block_comment)
] @prepend_input_softline

(
  (block_comment) @append_input_softline
  .
  ["," ";"]* @do_nothing
)

; --- Keywords needing surrounding space ------------------------------------

[
  "abstract"
  "actual"
  "as"
  "as?"
  "by"
  "catch"
  "class"
  "companion"
  "const"
  "constructor"
  "crossinline"
  "data"
  "do"
  "else"
  "enum"
  "expect"
  "external"
  "final"
  "finally"
  "for"
  "fun"
  "get"
  "if"
  "in"
  "infix"
  "init"
  "inline"
  "inner"
  "interface"
  "internal"
  "is"
  "lateinit"
  "noinline"
  "object"
  "open"
  "operator"
  "out"
  "override"
  "private"
  "protected"
  "public"
  "return"
  "sealed"
  "set"
  "suspend"
  "tailrec"
  "throw"
  "try"
  "typealias"
  "val"
  "value"
  "var"
  "vararg"
  "when"
  "where"
  "while"
] @prepend_space @append_space

"package" @append_space
"import" @append_space

; --- Assignment, arrows ----------------------------------------------------

(assignment
  "=" @prepend_space @append_space
)

(property_declaration
  "=" @prepend_space @append_space
)

(class_parameter
  "=" @prepend_space @append_space
)

(function_value_parameters
  "=" @prepend_space @append_space
)

(type_alias
  "=" @prepend_space @append_space
)

"->" @prepend_space @append_space

; --- Binary expression operators ------------------------------------------

(binary_expression
  [
    "+"
    "-"
    "*"
    "/"
    "%"
    "=="
    "!="
    "==="
    "!=="
    "<"
    ">"
    "<="
    ">="
    "&&"
    "||"
    "?:"
  ] @prepend_space @append_space
)

; --- Commas ----------------------------------------------------------------

"," @append_space

; --- Colons (context-dependent) -------------------------------------------

(class_declaration
  ":" @prepend_space @append_space
)

(object_declaration
  ":" @prepend_space @append_space
)

(function_declaration
  ":" @append_space
)

(secondary_constructor
  ":" @append_space
)

(companion_object
  ":" @prepend_space @append_space
)

(catch_block
  ":" @append_space
)

(parameter
  ":" @append_space
)

(class_parameter
  ":" @append_space
)

(variable_declaration
  ":" @append_space
)

(type_parameter
  ":" @prepend_space @append_space
)

(type_constraint
  ":" @prepend_space @append_space
)

; --- Headers: one per line ------------------------------------------------

(source_file
  (package_header) @append_hardline
)

(source_file
  (import) @append_hardline
)

; --- Top-level separation -------------------------------------------------

; Statement children of the file go on their own line.
(source_file
  (statement) @append_hardline
)

; --- Class body members ---------------------------------------------------

(class_body
  (class_member_declaration) @append_hardline
)

; Put each enum entry on its own line: the comma stays glued to the entry,
; newline goes after the comma.
(enum_class_body
  "," @append_hardline
)

; --- Block indentation ----------------------------------------------------

; Non-empty class / enum bodies force multi-line formatting. Empty `{}` is
; left untouched.
(class_body
  .
  "{" @append_hardline @append_indent_start
  (class_member_declaration)
  "}" @prepend_hardline @prepend_indent_end
  .
)

(enum_class_body
  .
  "{" @append_hardline @append_indent_start
  [(enum_entry) (class_member_declaration)]
  "}" @prepend_hardline @prepend_indent_end
  .
)

(when_expression
  "{" @append_hardline @append_indent_start
  "}" @prepend_hardline @prepend_indent_end
)

; Non-empty blocks (function bodies, if/while/for/try-catch bodies, ...)
; always render multi-line with indentation. Empty `{}` stays as-is.
(block
  .
  "{" @append_hardline @append_indent_start
  (statement)
  "}" @prepend_hardline @prepend_indent_end
  .
)

; Separate consecutive statements in a block.
(block
  (statement) @append_hardline
)

; Lambda literal: inner content is separated by a space, or a newline if the
; lambda spans multiple lines in input.
(lambda_literal
  .
  "{" @append_spaced_softline @append_indent_start
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

(lambda_literal
  (statement) @append_spaced_softline
)

; Function body with `=` expression: surround with spaces.
(function_body
  "=" @prepend_space @append_space
)

; --- when entries ---------------------------------------------------------

(when_expression
  (when_entry) @append_hardline
)

; --- Parentheses / brackets: tight spacing inside, regardless of context --

"(" @append_antispace
")" @prepend_antispace
"[" @append_antispace
"]" @prepend_antispace

; A block body (function, lambda, if/else, etc.) is always separated from
; what precedes it by a space. Same for class / enum / object bodies.
(block) @prepend_space
(class_body) @prepend_space
(enum_class_body) @prepend_space

; Space after the closing `)` of control-flow headers so bodies without
; braces still read correctly: `if (x) foo()`, `for (e in xs) println(e)`.
(if_expression
  ")" @append_space
)

(for_statement
  ")" @append_space
)

(while_statement
  ")" @append_space
)

(catch_block
  ")" @append_space
)

; --- Argument / parameter list glue ---------------------------------------

(call_expression
  (value_arguments) @prepend_antispace
)

(function_declaration
  (function_value_parameters) @prepend_antispace
)

(class_declaration
  (primary_constructor) @prepend_antispace
)

(primary_constructor
  (class_parameters) @prepend_antispace
)

(secondary_constructor
  (function_value_parameters) @prepend_antispace
)

(constructor_invocation
  (value_arguments) @prepend_antispace
)
