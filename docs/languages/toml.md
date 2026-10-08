# TOML

TOML support is built into the editor.

- Tree-sitter:
  [tree-sitter/tree-sitter-toml](https://github.com/tree-sitter/tree-sitter-toml)

- Crate: [adot-tree-sitter-toml](https://crates.io/crates/adot-tree-sitter-toml)

LSP support defaults to [Tombi](https://github.com/tombi-toml/tombi).

There is also support via [Taplo](https://taplo.tamasfe.dev), but it is disabled
by default. To use Taplo instead, change this in your `settings.jsonc`:

```json
{
  "languages": {
    "TOML": {
      "language_servers": ["!tombi", "taplo", "..."],
    },
  },
}
```
