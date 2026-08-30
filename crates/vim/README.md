This contains the code for Gram's Vim emulation mode.

Vim mode in Gram is supposed to primarily "do what you expect": it mostly tries
to copy vim exactly, but will use Gram-specific functionality when available to
make things smoother. This means Gram will never be 100% vim compatible, but
should be 100% vim familiar!

The backlog is maintained in the `#vim` channel notes.

## Testing gram-only behavior

Gram does more than vim in its default mode. The `VimTestContext` can be used
instead. This lets you test integration with the language server and other parts
of gram's UI that don't have a vim equivalent.
