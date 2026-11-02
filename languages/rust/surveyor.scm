;; Surveyor query pack for Rust (tree-sitter-rust 0.24).
;; Capture names: definition.<kind>, reference.<role>, literal.<role>, name, doc, attr.<key>.

;; ---- definitions -------------------------------------------------------------

(function_item name: (identifier) @name) @definition.function

(impl_item body: (declaration_list
  (function_item name: (identifier) @name) @definition.method))

(trait_item body: (declaration_list
  (function_item name: (identifier) @name) @definition.method))

(function_signature_item name: (identifier) @name) @definition.method

(struct_item name: (type_identifier) @name) @definition.struct
(union_item name: (type_identifier) @name) @definition.struct
(enum_item name: (type_identifier) @name) @definition.enum
(enum_variant name: (identifier) @name) @definition.variant
(trait_item name: (type_identifier) @name) @definition.trait
(type_item name: (type_identifier) @name) @definition.type
(const_item name: (identifier) @name) @definition.const
(static_item name: (identifier) @name) @definition.const
(macro_definition name: (identifier) @name) @definition.macro
(mod_item name: (identifier) @name) @definition.module

(impl_item type: (_) @name) @definition.impl
(impl_item trait: (_) @attr.trait type: (_) @name) @definition.impl

;; tests: a function directly preceded by a #[test]-like attribute
(_
  (attribute_item (attribute) @_attr)
  .
  (function_item name: (identifier) @name) @definition.test
  (#match? @_attr "^(tokio::|async_std::|actix_rt::|sqlx::|rstest::)?(test|rstest)(\\(|$)"))

(_
  (attribute_item (attribute) @_attr)
  .
  (attribute_item)
  .
  (function_item name: (identifier) @name) @definition.test
  (#match? @_attr "^(tokio::|async_std::|actix_rt::|sqlx::|rstest::)?(test|rstest)(\\(|$)"))

;; ---- references --------------------------------------------------------------

(call_expression function: (identifier) @name) @reference.call
(call_expression function: (scoped_identifier) @name) @reference.call
(call_expression function: (field_expression value: (_) @receiver field: (field_identifier) @name)) @reference.call.method
(call_expression function: (generic_function function: (identifier) @name)) @reference.call
(call_expression function: (generic_function function: (scoped_identifier) @name)) @reference.call
(macro_invocation macro: (identifier) @name) @reference.call.macro
(macro_invocation macro: (scoped_identifier) @name) @reference.call.macro

(impl_item trait: (_) @name) @reference.implements

(use_declaration argument: (_) @name) @reference.import

(type_identifier) @name @reference.type
(scoped_type_identifier name: (type_identifier) @name) @reference.type

;; a path used as a value or in a pattern: `NodeKind::is_callable` passed as an argument,
;; `Fact::Declaration(d)` matched; the host drops the ones that are already calls or imports
(scoped_identifier) @reference.mention

;; ---- literals ----------------------------------------------------------------

;; sqlx-style query functions and macros: the first string argument is SQL
(call_expression
  function: [(identifier) (scoped_identifier) (field_expression)] @_f
  arguments: (arguments . [(string_literal) (raw_string_literal)] @literal.sql)
  (#match? @_f "(^|::|\\.)(query|query_as|query_scalar|query_unchecked|query_as_unchecked|execute|batch_execute|prepare|fetch_one|fetch_all|fetch_optional|simple_query)$"))

;; `sqlx::query_as::<_, Row>("...")`: the turbofish hides the name in a generic_function
(call_expression
  function: (generic_function function: (_) @_f)
  arguments: (arguments . [(string_literal) (raw_string_literal)] @literal.sql)
  (#match? @_f "(^|::|\\.)(query|query_as|query_scalar|query_unchecked|query_as_unchecked|execute|batch_execute|prepare|fetch_one|fetch_all|fetch_optional|simple_query)$"))

(macro_invocation
  macro: (_) @_m
  (token_tree . [(string_literal) (raw_string_literal)] @literal.sql)
  (#match? @_m "(^|::)(query|query_as|query_scalar|query_unchecked|query_as_unchecked)$"))

;; SQL held in a binding or built with format!: `let sql = "SELECT ..."`, `format!("UPDATE {t} ...")`.
;; Non-SQL strings are dropped later because they do not start with a statement keyword.
(let_declaration value: [(string_literal) (raw_string_literal)] @literal.sql)
(const_item value: [(string_literal) (raw_string_literal)] @literal.sql)
(static_item value: [(string_literal) (raw_string_literal)] @literal.sql)
(macro_invocation
  macro: (_) @_m
  (token_tree . [(string_literal) (raw_string_literal)] @literal.sql)
  (#match? @_m "(^|::)(format|concat|write|writeln)$"))
