# Code bloat and binary size

My current focus is to reduce the size of the binary. So far 
I have mainly been trying to do this by removing dependencies and code wherever
I can. These are some notes on tools and techniques that have been helpful.

First helpful source is the Rust performance book: <https://nnethercote.github.io/perf-book/title-page.html>

This repository had some helpful tips, mainly on cargo options: <https://github.com/johnthagen/min-sized-rust>

## Finding unused dependencies

There are two tools for this:

- `cargo-machete`: <https://github.com/bnjbvr/cargo-machete>
- `cargo-shear`: <https://github.com/Boshen/cargo-shear>

I run both. `cargo-shear` seems to be both more accurate / to find more unused
dependencies, and less prone to false positives.

## The cost of async

So far this is not something I've really looked at, but async code in rust has
a significant cost in code amplification due to rewriting function bodies into
state machines. See
<https://tweedegolf.nl/en/blog/237/async-rust-never-left-the-mvp-state/>. Since
this code base is so heavily into async everywhere I expect that there should
be a lot of wins possible by rewriting code that doesn't need to be async into
straight-line code and running longer tasks on background threads more.

## Generic code amplification

Functions and structs that take generic arguments are compiled as a separate
copy for each variant in use. I found a couple of tools that help with finding
flarge functions as well as functions that end up generating a lot of copies:

- `cargo-bloat`: <https://github.com/RazrFalcon/cargo-bloat>
- `cargo-llvm-lines`: <https://github.com/dtolnay/cargo-llvm-lines>

`cargo llvm-lines` reveals how much code is generated for each function:

```sh
cargo llvm-lines -p settings_ui --release | head -20
```

This turned out to be huge: The settings code in particular generates crazy
amounts of code, and while I have managed to reduce the size a lot just by
finding ways to move code out of generic functions the whole settings
implementation should probably be rewritten from scratch.
