; Folding ranges for V in Zed
; Folds on { } blocks — covers fn bodies, struct bodies,
; if/else, for, match, interface, enum, and all other block constructs

(function_declaration
  body: (block) @fold)

(struct_declaration
  (struct_field_scope) @fold)

; Interface and enum bodies — fold the entire declaration block.
; Using a single @fold on the declaration node is the correct Zed API;
; @fold.start / @fold.end are not supported in the Zed tree-sitter query engine.
(interface_declaration) @fold

(enum_declaration) @fold

(if_expression
  (block) @fold)

(for_statement
  (block) @fold)

(match_expression
  (block) @fold)

(block) @fold
