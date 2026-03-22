; Folding ranges for V in Zed
; Folds on { } blocks — covers fn bodies, struct bodies,
; if/else, for, match, interface, and enum constructs.
;
; NOTE: The bare (block) @fold is intentionally absent. It would produce
; duplicate and stacked folds for every anonymous block inside expressions.
; Each specific pattern below covers its own block exactly once.

(function_declaration
  body: (block) @fold)

(function_literal
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

(else_branch
  (block) @fold)

(for_statement
  (block) @fold)

(match_expression
  (block) @fold)

(lock_expression
  body: (block) @fold)

(unsafe_expression
  (block) @fold)

(defer_statement
  (block) @fold)
