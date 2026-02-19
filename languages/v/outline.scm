; Outline / breadcrumb queries for V in Zed
; Used by Zed for the outline panel and breadcrumbs
; (v-analyzer's LSP documentSymbol also feeds the outline — this is the tree-sitter fallback)

(function_declaration
  name: (identifier) @name) @item

(struct_declaration
  name: (identifier) @name) @item

(interface_declaration
  name: (identifier) @name) @item

(enum_declaration
  name: (identifier) @name) @item

(type_declaration
  name: (identifier) @name) @item
