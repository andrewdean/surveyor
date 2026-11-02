;; Surveyor query pack for JavaScript and JSX (tree-sitter-javascript 0.25).

(function_declaration name: (identifier) @name) @definition.function
(generator_function_declaration name: (identifier) @name) @definition.function
(lexical_declaration (variable_declarator
  name: (identifier) @name
  value: [(arrow_function) (function_expression)]) @definition.function)
(variable_declaration (variable_declarator
  name: (identifier) @name
  value: [(arrow_function) (function_expression)]) @definition.function)
(method_definition name: (property_identifier) @name) @definition.method
(class_declaration name: (identifier) @name) @definition.class

(call_expression
  function: (identifier) @_f
  arguments: (arguments . (string) @name)
  (#match? @_f "^(describe|it|test)$")) @definition.test

(call_expression function: (identifier) @name) @reference.call
(call_expression function: (member_expression property: (property_identifier) @name)) @reference.call.method
(new_expression constructor: (identifier) @name) @reference.type

;; The whole statement is the name: the extractor expands `import { a, b } from "m"`
;; into one import per bound name.
(import_statement) @name @reference.import
(call_expression
  function: (identifier) @_r
  arguments: (arguments . (string) @name)
  (#eq? @_r "require")) @reference.import

(class_heritage (identifier) @name) @reference.extends

(call_expression
  function: (_) @_f
  arguments: (arguments . [(string) (template_string)] @literal.sql)
  (#match? @_f "(^|\\.)(query|execute|unsafe|sql|raw)$"))
