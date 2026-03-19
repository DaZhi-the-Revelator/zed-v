; Tags queries for V in Zed
; Used by Zed for symbol search (Cmd/Ctrl+T) and go-to-symbol
; Complements outline.scm

; Top-level functions
(function_declaration
  name: (identifier) @name
  (#set! "kind" "Function")
  (#not-exists? receiver)) @item

; Methods
(function_declaration
  receiver: (receiver)
  name: (identifier) @name
  (#set! "kind" "Method")) @item

; Static methods
(static_method_declaration
  name: (identifier) @name
  (#set! "kind" "Method")) @item

(struct_declaration
  name: (identifier) @name
  (#set! "kind" "Class")) @item

(interface_declaration
  name: (identifier) @name
  (#set! "kind" "Interface")) @item

(enum_declaration
  name: (identifier) @name
  (#set! "kind" "Enum")) @item

(type_declaration
  name: (identifier) @name
  (#set! "kind" "Type")) @item

(constant_declaration
  (constant_definition
    name: (identifier) @name
    (#set! "kind" "Constant"))) @item
