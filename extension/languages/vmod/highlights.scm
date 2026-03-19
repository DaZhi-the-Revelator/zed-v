; Syntax highlighting for v.mod manifest files
; v.mod uses V's type-initializer syntax: Module { key: 'value' }
; The V grammar parses this correctly, so we reuse the v grammar
; and write focused queries for the manifest structure.

; "Module" keyword — the manifest type name
(type_initializer
  type: (plain_type
    (type_reference_expression) @keyword.storage.type))

; Field keys (name:, description:, version:, etc.)
(keyed_element
  key: (field_name
    (reference_expression) @property))

; String values
(interpreted_string_literal) @string
(raw_string_literal) @string
(c_string_literal) @string
(escape_sequence) @string.escape

; Array brackets (for dependencies list)
(array_creation) @punctuation.bracket

[
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

[
  ":"
  ","
] @punctuation.delimiter

; Comments
(line_comment) @comment
(block_comment) @comment
