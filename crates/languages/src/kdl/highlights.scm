; Types
(node (identifier) @type)
(type) @type
(annotation_type) @type

; Variables
(identifier) @variable

; Properties
(prop (identifier) @keyword)


; Operators
[
 "="
 "+"
 "-"
] @operator

; Literals

(string) @string
(escape) @string.escape
(number) @number
(number (decimal) @number)
(number (exponent) @number)
(boolean) @boolean
"null" @constant

; Punctuation
["{" "}"] @punctuation.bracket
["(" ")"] @punctuation.bracket

[
  ";"
] @punctuation.delimiter

; Comments
[
  (single_line_comment)
  (multi_line_comment)
] @comment.line

(node (node_comment) (#set! "priority" 105)) @comment
(node (node_field (node_field_comment) (#set! "priority" 105)) @comment)
(node_children (node_children_comment) (#set! "priority" 105)) @comment
