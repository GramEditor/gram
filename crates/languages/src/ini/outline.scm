; Match sections in INI files
(section
    (section_name
        "[" @context.extra
        (text) @name
        "]" @context.extra
    ) @item
) @item

; Match key-value pairs
(setting
  (setting_name) @name
  "=" @context.extra
  (setting_value) @context.extra
) @item

(comment) @annotation
