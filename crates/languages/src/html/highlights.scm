(tag_name) @tag
(doctype) @tag.doctype
(attribute_name) @attribute
[
  "\""
  "'"
  (attribute_value)
] @string
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

"=" @punctuation.delimiter.html

[
  "<"
  ">"
  "<!"
  "</"
  "/>"
] @punctuation.bracket.html
