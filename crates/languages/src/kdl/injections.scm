[
  (single_line_comment)
  (multi_line_comment)
] @comment

(node
    (identifier) @key (#eq? @key "custom-shader")
    (node_field (value (string) @content (#set! "language" "glsl"))))
