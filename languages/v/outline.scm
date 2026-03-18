; Outline / breadcrumb queries for V in Zed
; Uses @open/@close to create nesting containers and @item on children
; for proper tree structure in the outline panel.

; ── Constants ────────────────────────────────────────────────────────────────

(const_declaration
  (const_definition
    name: (identifier) @name) @item)

; ── Type aliases & sum types ──────────────────────────────────────────────────

(type_declaration
  name: (identifier) @name) @item

; ── Enums ─────────────────────────────────────────────────────────────────────

(enum_declaration
  name: (identifier) @name
  "{" @open
  "}" @close) @item

(enum_declaration
  (enum_field_definition
    name: (identifier) @name) @item)

; ── Interfaces ────────────────────────────────────────────────────────────────

(interface_declaration
  name: (identifier) @name
  "{" @open
  "}" @close) @item

(interface_declaration
  (interface_method_definition
    name: (identifier) @name) @item)

(interface_declaration
  (struct_field_declaration
    name: (identifier) @name) @item)

; ── Structs ───────────────────────────────────────────────────────────────────

(struct_declaration
  name: (identifier) @name
  "{" @open
  "}" @close) @item

(struct_declaration
  (struct_field_declaration
    name: (identifier) @name) @item)

; ── Functions & methods ───────────────────────────────────────────────────────

; Top-level functions only — guard prevents methods matching this pattern too
(function_declaration
  name: (identifier) @name
  body: (block
    "{" @open
    "}" @close)
  (#not-exists? receiver)) @item

; Methods — function declarations that have a receiver.
; @context captures the receiver type (e.g. "Rect") so the breadcrumb
; bar and outline panel display "Rect.area" instead of just "area".
(function_declaration
  receiver: (receiver
    type: (plain_type) @context)
  name: (identifier) @name
  body: (block
    "{" @open
    "}" @close)) @item

(static_method_declaration
  name: (identifier) @name
  body: (block
    "{" @open
    "}" @close)) @item
