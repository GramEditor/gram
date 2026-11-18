# XML

XML syntax highlighting is supported natively.

- Tree-sitter:
  [tree-sitter-grammars/tree-sitter-xml](https://github.com/tree-sitter-grammars/tree-sitter-xml)

## Configuration

If you have additional file extensions that are not being automatically
recognized as XML just add them to
[file_types](../configuring-gram.md#file-types) in your Gram settings:

```json
  "file_types": {
    "XML": ["rdf", "gpx", "kml"]
  }
```

## Formatters

You can also set a different formatter for XML files through the [language specific configuration](../configuring-languages.md). [xmllint](https://gnome.pages.gitlab.gnome.org/libxml2/xmllint.html) is a popular XML parsing tool that can also format XML files, and usually is pre-installed, as part of [libxml2](https://gitlab.gnome.org/GNOME/libxml2/-/wikis/home):

```json
"languages": {
  "XML": {
    "formatter": {
      "external": {
        "command": "xmllint",
        "arguments": ["--format", "-"]
      },
    }
  }
}
```
