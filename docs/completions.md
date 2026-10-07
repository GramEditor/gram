# Completions

"Code Completions" provided by Language Servers (LSPs) automatically installed
by Gram or via [Gram Language Extensions](./languages.md).

## Language Server Code Completions {#code-completions}

When there is an appropriate language server available, Gram will provide
completions of variable names, functions, and other symbols in the current file.
You can disable these by adding the following to your Gram `settings.jsonc`
file:

```json
"show_completions_on_input": false
```

You can manually trigger completions with `ctrl-space` or by triggering the
`editor::ShowCompletions` action from the command palette.

The code completions can optionally show a text icon to show what kind of item
it is. e.g. Function / Property / Constant etc. Defaults to `"Off"` Enable with:

```json
"completion_menu_item_kind": "symbol"
```

For more information, see:

- [Configuring Supported Languages](./configuring-languages.md)
- [List of Gram Supported Languages](./languages.md)
