# Scheme

## Formatting

To support automatically formatting your code, you can use
[raviqqe/schemat](https://github.com/raviqqe/schemat), a Scheme code formatter.

Install with:

```bash
cargo install schemat
```

Then add the following to your Gram `settings.json`:

```json
"languages": {
  "Scheme": {
    "formatter": {
      "external": {
        "command": "schemat",
        "arguments": [""]
      }
    }
  }
}
```

## Tree-sitter

- Tree-sitter:
  [6cdh/tree-sitter-scheme](https://github.com/6cdh/tree-sitter-scheme)
