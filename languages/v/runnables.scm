; V main function — triggers v run via the v-main tag
(
  (function_declaration
    name: (identifier) @run
  ) @_v_main_decl
  (#eq? @run "main")
  (#set! tag v-main)
)

; V test functions (fn test_xxx) — matched by the v-test tag
(
  (function_declaration
    name: (identifier) @run
  ) @_v_test_decl
  (#match? @run "^test_")
  (#set! tag v-test)
)
