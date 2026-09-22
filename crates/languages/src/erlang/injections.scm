((comment) @injection.content
  (#set! injection.language "comment"))

; EEP-59 doc attributes use Markdown by default.
(wild_attribute
  name: (attr_name
    name: (atom) @_attribute)
  value: (string) @injection.content
  (#any-of? @_attribute "doc" "moduledoc")
  (#set! injection.language "markdown"))

; Also support the accepted parenthesized attribute form.
(wild_attribute
  name: (attr_name
    name: (atom) @_attribute)
  value: (paren_expr
    expr: (string) @injection.content)
  (#any-of? @_attribute "doc" "moduledoc")
  (#set! injection.language "markdown"))
