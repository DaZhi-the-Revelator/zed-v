; V language highlights for Zed
; Built on top of v-analyzer's tree_sitter_v grammar
; Combines the best of the base highlights.scm and helix.highlights.scm
; with Zed-specific refinements

; ============================================================
; ERRORS
; ============================================================

(ERROR) @error

; ============================================================
; COMMENTS
; ============================================================

(line_comment) @comment
(block_comment) @comment
(shebang) @comment

; ============================================================
; MODULES & IMPORTS
; ============================================================

(module_clause
  (identifier) @namespace)

(import_path
  (import_name) @namespace)

(import_alias
  (import_name) @namespace)

; ============================================================
; FUNCTIONS & METHODS
; ============================================================

; Function declarations
(function_declaration
  name: (identifier) @function)

; Method declarations (have a receiver)
(function_declaration
  receiver: (receiver)
  name: (identifier) @function.method)

; Interface method declarations
(interface_method_definition
  name: (identifier) @function.method)

; Function calls
(call_expression
  name: (reference_expression) @function.call)

; Method calls
(call_expression
  name: (selector_expression
    field: (reference_expression) @function.method))

; ============================================================
; TYPES
; ============================================================

; Named type declarations
(struct_declaration
  name: (identifier) @type)

(enum_declaration
  name: (identifier) @type)

(interface_declaration
  name: (identifier) @type)

(type_declaration
  name: (identifier) @type)

; Type references in expressions
(type_reference_expression) @type

; Pointer/array type expressions
(pointer_type) @type
(array_type) @type

; ============================================================
; VARIABLES & PARAMETERS
; ============================================================

(identifier) @variable

(parameter_declaration
  name: (identifier) @variable.parameter)

(receiver
  name: (identifier) @variable.parameter)

; Short lambda parameter
(short_lambda
  (reference_expression) @variable.parameter)

; ============================================================
; STRUCT FIELDS & SELECTORS
; ============================================================

(struct_field_declaration
  name: (identifier) @property)

(field_name) @property

(selector_expression
  field: (reference_expression) @property)

; ============================================================
; CONSTANTS & COMPILE-TIME
; ============================================================

; Enum fetch (e.g. Direction.left)
(enum_fetch
  (reference_expression) @constant)

; Enum field definitions
(enum_field_definition
  (identifier) @constant)

; Global variable definitions
(global_var_definition
  (identifier) @constant)

; Compile-time conditions
(compile_time_if_expression
  condition: (reference_expression) @constant)

; ============================================================
; LITERALS
; ============================================================

(int_literal) @number
(float_literal) @number

(interpreted_string_literal) @string
(c_string_literal) @string
(raw_string_literal) @string
(string_interpolation) @string
(rune_literal) @string
(escape_sequence) @string.escape

; String interpolation delimiters
(string_interpolation
  (interpolation_opening) @punctuation.bracket
  (interpolation_closing) @punctuation.bracket)

(true) @boolean
(false) @boolean

(nil) @constant.builtin
(none) @constant.builtin

; ============================================================
; ATTRIBUTES
; ============================================================

(attribute) @attribute

; ============================================================
; LABELS
; ============================================================

(label_reference) @label

; ============================================================
; KEYWORDS — Control Flow
; ============================================================

[
  "if"
  "$if"
  "$else"
  "else"
  "select"
  "match"
] @keyword.control.conditional

[
  "for"
  "$for"
] @keyword.control.repeat

[
  "return"
  "goto"
] @keyword.control.return

[
  "break"
  "continue"
  "go"
  "spawn"
  "shared"
  "lock"
  "rlock"
] @keyword.control

[
  "import"
] @keyword.control.import

[
  "fn"
] @keyword.function

; ============================================================
; KEYWORDS — Storage / Type Definition
; ============================================================

[
  "struct"
  "interface"
  "enum"
  "type"
  "union"
  "module"
] @keyword.storage.type

[
  "const"
  "static"
  "__global"
] @keyword.storage.modifier

[
  "mut"
] @keyword.storage.modifier.mut

[
  "pub"
] @keyword.modifier

; ============================================================
; KEYWORDS — Operators / Other
; ============================================================

[
  "as"
  "in"
  "!in"
  "is"
  "!is"
  "or"
  "implements"
] @keyword.operator

[
  "assert"
  "asm"
  "defer"
  "unsafe"
  "sql"
] @keyword

; ============================================================
; OPERATORS
; ============================================================

[
  "++"
  "--"

  "+"
  "-"
  "*"
  "/"
  "%"

  "~"
  "&"
  "|"
  "^"

  "!"
  "&&"
  "||"
  "!="

  "<<"
  ">>"

  "<"
  ">"
  "<="
  ">="

  "+="
  "-="
  "*="
  "/="
  "&="
  "|="
  "^="
  "<<="
  ">>="

  "="
  ":="
  "=="

  "?"
  "<-"
  "$"
  ".."
  "..."
] @operator

; ============================================================
; PUNCTUATION
; ============================================================

[
  "."
  ","
  ":"
  ";"
] @punctuation.delimiter

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

(array_creation) @punctuation.bracket
