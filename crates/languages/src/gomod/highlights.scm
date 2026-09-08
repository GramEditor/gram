[
  "require"
  "replace"
  "go"
  "toolchain"
  "tool"
  "exclude"
  "retract"
  "module"
  "ignore"
] @keyword

"=>" @operator

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

[
(version)
(go_version)
] @string
