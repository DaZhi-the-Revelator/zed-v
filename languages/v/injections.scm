; Injections for V in Zed
; Gives embedded sub-languages their own syntax highlighting

; ── String interpolation ─────────────────────────────────────────────────────
; The V sub-expression inside ${...} is itself V — highlight it as such
(string_interpolation
  (interpolation_expression) @injection.content
  (#set! injection.language "v"))

; ── SQL (ORM) ─────────────────────────────────────────────────────────────────
; sql db { select from Table where ... }
(sql_expression
  (_) @injection.content
  (#set! injection.language "sql"))

; ── Inline assembly ──────────────────────────────────────────────────────────
; asm x64 { ... }  — highlight body as asm if the language is available
(asm_statement
  (_) @injection.content
  (#set! injection.language "asm"))
