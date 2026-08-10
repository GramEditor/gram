# Debugging Crashes

To generate a minidump when Gram panics or crashes, set the environment variable
`GRAM_GENERATE_MINIDUMPS=1`:

```sh
$ GRAM_GENERATE_MINIDUMPS=1 gram
```

If set, Gram sends a message to a sidecar process which inspects the memory of
the crashing editor to create a
[minidump](https://chromium.googlesource.com/breakpad/breakpad/+/master/docs/getting_started_with_breakpad.md#the-minidump-file-format)
file in `~/Library/Logs/Gram` or `$XDG_DATA_HOME/gram/logs`. This minidump can
be used to generate backtraces for the stacks of all threads.

These crash reports contain rich information; but they are hard to read because
they don't contain spans or symbol information. You can still work with them
locally by downloading sources and an unstripped binary (or separate symbols
file) for your Gram release and running:

```sh
zstd -d ~/.local/share/gram/<uuid>.dmp -o minidump.dmp
minidump-stackwalk minidump.dmp
```

Alongside the minidump file in your logs dir, there should be a `<uuid>.json`
which contains additional metadata like the panic message, span, and system
specs.

## Using a Debugger

If you can reproduce the crash consistently, a debugger can be used to inspect
the state of the program at the time of the crash, often providing useful
insights into the cause of the crash.

You can read more about setting up and using a debugger with Gram, and
specifically for debugging crashes
[here](./debuggers.md#debugging-panics-and-crashes).
