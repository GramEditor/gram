(comment) @comment
(
  (comment) @comment.todo
  (#match? @comment.todo "TODO:")
)
(
  (comment) @comment.note
  (#match? @comment.note "NOTE:")
)
(
  (comment) @comment.warning
  (#match? @comment.warning "WARNING:|WARN:|ATTENTION:")
)

(string) @string
(escape_sequence) @string.escape

(pair
  key: (string) @property.json_key)

(number) @number

[
  (true)
  (false)
] @boolean

(null) @constant.builtin

[
  ","
  ":"
] @punctuation.delimiter

[
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket
