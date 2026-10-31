;; Surveyor query pack for TypeScript and TSX (tree-sitter-typescript 0.23).

;; ---- definitions -------------------------------------------------------------

(function_declaration name: (identifier) @name) @definition.function
(generator_function_declaration name: (identifier) @name) @definition.function
(function_signature name: (identifier) @name) @definition.function

(lexical_declaration (variable_declarator
  name: (identifier) @name
  value: [(arrow_function) (function_expression)]) @definition.function)
(variable_declaration (variable_declarator
  name: (identifier) @name
  value: [(arrow_function) (function_expression)]) @definition.function)

(method_definition name: (property_identifier) @name) @definition.method
(method_signature name: (property_identifier) @name) @definition.method
(abstract_method_signature name: (property_identifier) @name) @definition.method

(class_declaration name: (type_identifier) @name) @definition.class
(abstract_class_declaration name: (type_identifier) @name) @definition.class
(interface_declaration name: (type_identifier) @name) @definition.interface
(type_alias_declaration name: (type_identifier) @name) @definition.type
(enum_declaration name: (identifier) @name) @definition.enum
(internal_module name: (identifier) @name) @definition.module

;; tests: describe/it/test blocks name the suite or case
(call_expression
  function: (identifier) @_f
  arguments: (arguments . (string) @name)
  (#match? @_f "^(describe|it|test)$")) @definition.test

;; ---- references --------------------------------------------------------------

(call_expression function: (identifier) @name) @reference.call
(call_expression function: (member_expression property: (property_identifier) @name)) @reference.call.method
(new_expression constructor: (identifier) @name) @reference.type

(import_statement source: (string) @name) @reference.import
(call_expression
  function: (identifier) @_r
  arguments: (arguments . (string) @name)
  (#eq? @_r "require")) @reference.import

(class_heritage (extends_clause value: (identifier) @name)) @reference.extends
(class_heritage (implements_clause (type_identifier) @name)) @reference.implements
(extends_type_clause type: (type_identifier) @name) @reference.extends

(type_annotation (type_identifier) @name) @reference.type
(generic_type name: (type_identifier) @name) @reference.type

;; ---- literals ----------------------------------------------------------------

(call_expression
  function: (_) @_f
  arguments: (arguments . [(string) (template_string)] @literal.sql)
  (#match? @_f "(^|\\.)(query|execute|unsafe|sql|raw)$"))

;; SQL held in a binding: `const sql = \`SELECT ...\``.
(variable_declarator value: [(string) (template_string)] @literal.sql)
