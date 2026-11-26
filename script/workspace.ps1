function ParseGramWorkspace {
    $metadata = cargo metadata --no-deps --offline | ConvertFrom-Json
    $env:GRAM_WORKSPACE = $metadata.workspace_root
    $env:RELEASE_VERSION = $metadata.packages | Where-Object { $_.name -eq "gram" } | Select-Object -ExpandProperty version
}
