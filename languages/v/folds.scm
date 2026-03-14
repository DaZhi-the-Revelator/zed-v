; Folding ranges for V in Zed
; Folds on { } blocks — covers fn bodies, struct bodies,
; if/else, for, match, interface, enum, and all other block constructs

(function_declaration
  body: (block) @fold)

(struct_declaration
  (struct_field_scope) @fold)

(interface_declaration
  "{" @fold.start
  "}" @fold.end)

(enum_declaration
  "{" @fold.start
  "}" @fold.end)

(if_expression
  (block) @fold)

(for_statement
  (block) @fold)

(match_expression
  (block) @fold)

(block) @fold
