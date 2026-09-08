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
  (addition)
  (new_file)
] @string
;; TODO: This should eventually be `@diff.plus` with a fallback of `@string`

[
  (deletion)
  (old_file)
] @keyword
;; TODO: This should eventually be `@diff.minus` with a fallback of `@keyword`

(commit) @constant

(location) @attribute

(command
  "diff" @function
  (argument) @variable.parameter)

(mode) @number

([
  ".."
  "+"
  "++"
  "+++"
  "++++"
  "-"
  "--"
  "---"
  "----"
] @punctuation.special)

[
  (binary_change)
  (similarity)
  (file_change)
] @label

(index
  "index" @keyword)

(similarity
  (score) @number
  "%" @number)
