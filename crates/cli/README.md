# Cli

## Testing

You can test your changes to the `cli` crate by first building the main gram
binary:

```
cargo build -p gram
```

And then building and running the `cli` crate with the following parameters:

```
cargo run -p cli -- --gram ./target/debug/gram.exe
```
