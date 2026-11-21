# YAML

YAML support is available natively in Gram.

- Tree-sitter:
  [zed-industries/tree-sitter-yaml](https://github.com/zed-industries/tree-sitter-yaml)
- Language Servers:
  [redhat-developer/yaml-language-server](https://github.com/redhat-developer/yaml-language-server),
  [home-operations/yayamlls](https://github.com/home-operations/yayamlls)

## Choosing a language server

`yaml-language-server` is Node.js LSP used by default. `yayamlls` is a static Go
binary with Kubernetes and Flux support, but no formatter.

With the
[yayamlls extension](https://github.com/home-operations/yayamlls/tree/main/editors/zed)
installed, which downloads the binary for you:

```json
"languages": {
  "YAML": {
    "language_servers": ["yayamlls", "!yaml-language-server"]
  }
}
```

Without the extension, install `yayamlls` yourself and override the bundled
binary, since the `lsp` key only accepts known LSP identifiers:

```json
"lsp": {
  "yaml-language-server": {
    "binary": {
      "path": "yayamlls",
      "arguments": [],
      "ignore_system_version": true
    }
  }
}
```

> [!NOTE]
> `arguments` must be empty: Gram launches this server with `--stdio`, which
> `yayamlls` does not accept.

## Formatting

By default, Gram uses Prettier for formatting YAML files. For example, to use
single quotes in yaml files add the following to your `.prettierrc`:

```json
{
  "overrides": [
    {
      "files": ["*.yaml", "*.yml"],
      "options": {
        "singleQuote": true
      }
    }
  ]
}
```

### LSP Formatting

To use `yaml-language-server` instead of Prettier for YAML formatting, add the
following to your Gram `settings.jsonc`:

```json
"languages": {
  "YAML": {
    "formatter": "language_server"
  }
}
```

`yayamlls` has no formatting.

## Configuration

Language server options can be set in the `settings.jsonc` file. See the
[Configuring Languages](../configuring-languages.md) documentation for more
information.

### yaml-language-server

For more details, see the
[yaml-language-server settings](https://github.com/redhat-developer/yaml-language-server?tab=readme-ov-file#language-server-settings).
Settings keys must be nested, so `yaml.keyOrdering` becomes `{"yaml":
{"keyOrdering": true}}`:

```json
"lsp": {
  "yaml-language-server": {
    "settings": {
      "yaml": {
        "keyOrdering": true,
        "format": {
          "singleQuote": true
        },
        "schemas": {
          "https://getcomposer.org/schema.json": ["/*"],
          "../relative/path/schema.json": ["/config*.yaml"]
        },
        "schemaStore": {
          "enable": true
        },
        "customTags": [
          "!Ref scalar",
          "!Sub mapping",
          "!GetAtt sequence"
        ]
      }
    }
  }
}
```

Schemas are auto-detected from [Json Schema Store](https://schemastore.org). The
`schemas` key overrides that, `schemaStore.enable: false` turns it off.
[Custom tags](https://github.com/redhat-developer/yaml-language-server#adding-custom-tags)
need a `scalar`, `mapping` or `sequence` suffix, e.g. for CloudFormation.

A modeline pins the schema for a single file and works with both servers:

```yaml
# yaml-language-server: $schema=https://www.schemastore.org/github-action.json
name: Issue Assignment
on:
  issues:
    types: [opened]
```

### yayamlls

For more details, see the
[yayamlls documentation](https://github.com/home-operations/yayamlls#configuration).
Configuration lives in `.yayamlls.yaml` at the workspace root, which keeps it
editor-agnostic:

```yaml
schemas:
  "https://json.schemastore.org/github-workflow.json":
    - ".github/workflows/*.yml"
catalog: true
kubernetes:
  enabled: false
customTags:
  - "!Ref"
```

The same keys can be set from Gram, flat and without the `yaml` wrapper. Use the
`yayamlls` key with the extension, or `yaml-language-server` with the binary
override above:

```json
"lsp": {
  "yayamlls": {
    "initialization_options": {
      "catalog": true,
      "kubernetes": { "enabled": false }
    }
  }
}
```

Schemas resolve per document: modeline, `schemas` globs, the Schema Store
catalog (`catalog: false` disables it), then Kubernetes `apiVersion`+`kind`.
Custom tags are listed bare, and nodes carrying one skip validation.

Diagnostics can be muted with `# yayamlls-disable-line`, `# yayamlls-disable`
and `# yayamlls-enable` around a block, or `# yayamlls-disable-file`.

Flux `HelmRelease` and `Kustomization` documents are also validated against
their rendered manifests, with **View rendered** and **Diff rendered** code
lenses. The lenses use `window/showDocument`, which Zed ignores for local files
([zed#53123](https://github.com/zed-industries/zed/discussions/53123)).

For CI, `yayamlls validate [--render] <paths>` resolves schemas the same way and
exits `1` on errors, `2` on usage or I/O failures, `0` otherwise.
