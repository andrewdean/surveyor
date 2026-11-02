;; Surveyor query pack for Python (tree-sitter-python 0.25).

(function_definition name: (identifier) @name) @definition.function
(class_definition body: (block
  (function_definition name: (identifier) @name) @definition.method))
(class_definition name: (identifier) @name) @definition.class

(function_definition
  name: (identifier) @name
  (#match? @name "^test_")) @definition.test

(module (expression_statement (assignment left: (identifier) @name)) @definition.const)

(call function: (identifier) @name) @reference.call
(call function: (attribute attribute: (identifier) @name)) @reference.call.method

(import_statement name: (dotted_name) @name) @reference.import
(import_statement name: (aliased_import name: (dotted_name) @name)) @reference.import
;; The whole statement is the name: the extractor expands `from m import a, b` into
;; one import per bound name.
(import_from_statement) @name @reference.import

(class_definition superclasses: (argument_list (identifier) @name)) @reference.extends
(class_definition superclasses: (argument_list (attribute attribute: (identifier) @name))) @reference.extends

(type (identifier) @name) @reference.type

(call
  function: (attribute attribute: (identifier) @_f)
  arguments: (argument_list . [(string) (concatenated_string)] @literal.sql)
  (#match? @_f "^(execute|executemany|fetch|fetchrow|fetchval|fetch_all|fetch_one|fetch_val|mogrify)$"))

;; SQL held in a binding: `sql = """SELECT ..."""`.
(assignment right: [(string) (concatenated_string)] @literal.sql)
