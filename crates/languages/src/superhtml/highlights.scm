(doctype) @constant

(comment) @comment

(tag_name) @tag

((tag_name) @string.special
  (#any-of? @string.special "super" "extend"))

(attribute_name) @attribute

(attribute_value) @string

((element
  (start_tag
    (attribute
      (attribute_name) @attribute
      [
        (attribute_value) @link_uri
        (quoted_attribute_value
          (attribute_value) @link_uri)
      ]))
  (element
    (start_tag
      (tag_name) @tag)))
  (#eq? @tag "super")
  (#eq? @attribute "id"))

(element
  (start_tag
    (tag_name) @string.special)
  (#eq? @string.special "super"))

"\"" @string

[
  "<"
  ">"
  "</"
  "/>"
  "<!"
] @punctuation.bracket

"=" @punctuation.delimiter
