; Locals queries for V in Zed
; Defines scope boundaries and local variable definitions so that
; syntax-only highlighting (used on files >1000 lines) correctly
; scopes variables and prevents names leaking across function bodies.

; ── Scopes ────────────────────────────────────────────────────────────────────

(source_file) @local.scope

(function_declaration
  body: (block) @local.scope)

(function_literal
  body: (block) @local.scope)

(block) @local.scope

(if_expression
  (block) @local.scope)

(else_branch
  (block) @local.scope)

(for_statement
  (block) @local.scope)

(match_expression
  (block) @local.scope)

(lock_expression
  body: (block) @local.scope)

(unsafe_expression
  (block) @local.scope)

(defer_statement
  (block) @local.scope)

; ── Definitions ───────────────────────────────────────────────────────────────

; Short variable declarations: x := expr
(var_declaration
  var_list: (identifier_list
    (identifier) @local.definition))

; For-loop variables: for x in ...  /  for i, v in ...
; A single flat pattern captures all var_definition names under the
; range_clause's var_definition_list, covering both the index and value
; variables without double-defining any node.
(range_clause
  (var_definition_list
    (var_definition
      name: (identifier) @local.definition)))

; Function parameters
(parameter_declaration
  name: (identifier) @local.definition)

; Method receiver
(receiver
  name: (identifier) @local.definition)

; Function name (visible within its own body for recursion)
(function_declaration
  name: (identifier) @local.definition)

; ── References ────────────────────────────────────────────────────────────────

(reference_expression
  (identifier) @local.reference)
