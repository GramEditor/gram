# TOML

TOML support is built into the editor.

- Tree-sitter:
  [tree-sitter/tree-sitter-toml](https://github.com/tree-sitter/tree-sitter-toml)

- Crate: [adot-tree-sitter-toml](https://crates.io/crates/adot-tree-sitter-toml)

There's LSP support for [Tombi](https://github.com/tombi-toml/tombi) and [Taplo](https://taplo.tamasfe.dev). Tombi is the default.

To only enable `tombi`:

```json
  "languages": {
    "TOML": {
      "language_servers": ["!taplo", "tombi", "..."]
    },
  }
```

To only enable `taplo`:

```json
  "languages": {
    "TOML": {
      "language_servers": ["tombi", "!taplo", "..."]
    },
  }
```

Tombi additionally supports being used as a formatter (enabled by default). If you wish to use prettier with taplo instead, change the formatter setting from `language_server` to `prettier`:

```json
  "languages": {
    "TOML": {
      "language_servers": ["!tombi", "taplo", "..."]
      "formatter": "prettier",
      "format_on_save": "off",
    },
  }
```
