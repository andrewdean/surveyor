;; Surveyor query pack for shell scripts (tree-sitter-bash 0.25).

(function_definition name: (word) @name) @definition.function

(command name: (command_name (word) @name)) @reference.call

(command
  name: (command_name (word) @_c)
  argument: (word) @name
  (#match? @_c "^(source|\\.)$")) @reference.import
