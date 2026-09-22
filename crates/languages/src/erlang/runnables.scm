; Conventionally named EUnit test modules.
((module_attribute
  name: (atom) @run @erlang_module_name)
  (#match? @run "_tests$")
  (#set! tag erlang-eunit-module))

; EUnit tests.
(source_file
  (module_attribute
    name: (atom) @erlang_module_name)
  (fun_decl
    (function_clause
      name: (atom) @run @erlang_test_name
      args: (expr_args)))
  (#match? @run "_test$")
  (#set! tag erlang-eunit-test))

; EUnit test generators.
(source_file
  (module_attribute
    name: (atom) @erlang_module_name)
  (fun_decl
    (function_clause
      name: (atom) @run @erlang_test_name
      args: (expr_args)))
  (#match? @run "_test_$")
  (#set! tag erlang-eunit-generator))

; Common Test suites follow the required *_SUITE module naming convention.
((module_attribute
  name: (atom) @run @erlang_module_name)
  (#match? @run "_SUITE$")
  (#set! tag erlang-common-test-suite))

; Common Test cases are one-argument functions in a *_SUITE module.
; Exclude the standard suite, group, and test-case callbacks.
(source_file
  (module_attribute
    name: (atom) @erlang_module_name)
  (fun_decl
    (function_clause
      name: (atom) @run @erlang_test_name
      args: (expr_args
        .
        (_) .)))
  (#match? @erlang_module_name "_SUITE$")
  (#not-match? @run
    "^(all|groups|suite|group|init_per_suite|end_per_suite|init_per_group|end_per_group|init_per_testcase|end_per_testcase)$")
  (#set! tag erlang-common-test-case))
