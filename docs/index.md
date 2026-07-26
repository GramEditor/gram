# Documentation

Gram is a **hard fork** of the Zed editor, with the following list (incomplete)
of changes:

- All AI integration has been removed
- All Telemetry has been removed
- All collaboration integration has been removed
- No proprietary server component
- No auto updates
- No license agreement on installation
- Only install language servers when [explicitly allowed](./language-servers.md)
- Integrated documentation viewer
- Support for more languages built in
- More syntax highlighting themes built in
- Extensions are installed from source only
- Partial support for Wasm extensions (due to AI removal)
- Added [SuperTab](./supertab.md)

For more details on the motivation behind this fork, read the
[Mission Statement](./mission.md).

### Migrating

- From [Zed](./migrate/zed.md)
- From [VS Code](./migrate/vs-code.md)

## Features

- [Debugger](./debugger.md): Integrated support for DAP, the debugger adapter
  protocol.
- [Remote Development](./remote-development.md): Connect to remote servers via
  SSH and edit as if working on a local project.
- [Extensions](./extensions.md): Add support for additional languages, themes
  and icons using the extension system.
- [Supported Languages](./languages.md)
- [Language Servers](./language-servers.md): Gram relies on language servers for
  providing advanced semantic functionality for various programming languages.

## Development

- [Development](./development.md)
  - [macOS](./development/macos.md)
  - [Linux](./development/linux.md)
  - [Windows](./development/windows.md)
  - [FreeBSD](./development/freebsd.md)
  - [Using Debuggers](./development/debuggers.md)
  - [Glossary](./development/glossary.md)
- [Debugging Crashes](./development/debugging-crashes.md)

## Configuration

- [Configuring Gram](./configuring-gram.md)
- [Configuring Languages](./configuring-languages.md)
  - [Toolchains](./toolchains.md)
- [Key bindings](./key-bindings.md)
  - [All Actions](./all-actions.md)
- [Snippets](./snippets.md)
- [Themes](./themes.md)
- [Icon Themes](./icon-themes.md)
- [Visual Customization](./visual-customization.md)
- [Vim Mode](./vim.md)
- [Helix Mode](./helix.md)
- [SuperTab](./supertab.md)

## Using Gram

- [Multibuffers](./multibuffers.md)
- [Command Palette](./command-palette.md)
- [Command-line Interface](./command-line-interface.md)
- [Outline Panel](./outline-panel.md)
- [Code Completions](./completions.md)
- [Git](./git.md)
- [Debugger](./debugger.md)
- [Diagnostics](./diagnostics.md)
- [Tasks](./tasks.md)
- [Tab Switcher](./tab-switcher.md)
- [Remote Development](./remote-development.md)
- [Environment Variables](./environment.md)
- [REPL](./repl.md)

## Platform Support

- [Windows](./windows.md)
- [Linux](./linux.md)

## Handling Problems

- [Troubleshooting](./troubleshooting.md)
- [Uninstall](./uninstall.md)

## Extensions

> **NOTE:** The Zed extension system relies on a closed-source server component,
> which is stripped from Gram. Instead, all extensions have to be built from
> source. Currently, there is no extension registry so the extensions have to be
> installed either via the suggestion popups or an URL and Wasm extensions need
> rustup installed in order to compile.

- [Overview](./extensions.md)
- [Installing Extensions](./extensions/installing-extensions.md)
- [Developing Extensions](./extensions/developing-extensions.md)
- [Extension Capabilities](./extensions/capabilities.md)
- [Language Extensions](./extensions/languages.md)
- [Debugger Extensions](./extensions/debugger-extensions.md)
- [Theme Extensions](./extensions/themes.md)
- [Icon Theme Extensions](./extensions/icon-themes.md)

## Integrations and related tools

There are some related projects mostly around making Gram available in various
package managers and Linux distributions. This list is not complete, if you know
of any packaging effort, Gram-specific extensions or anything like that, feel
free to submit a PR at <https://codeberg.org/GramEditor/gram>.

- **Homebrew (Mac):** <https://formulae.brew.sh/cask/gram>
- **Arch Linux:** <https://archlinux.org/packages/extra/x86_64/gram>
- **Arch Linux (AUR):** <https://aur.archlinux.org/packages/gram-git>
- **Alpine Linux:**
  <https://pkgs.alpinelinux.org/package/edge/testing/x86_64/gram>
- **Gentoo Linux:** <https://codeberg.org/GramEditor/gram-gentoo>
- **Raycast (Mac):** <https://www.raycast.com/justyt65/gram>
- **Chimera Linux (WIP):** <https://github.com/chimera-linux/cports/pull/5506>

## Legal note on accepting contributions

If you have previously installed Zed and agreed to their license agreement, you
may be legally prevented from contributing to Gram despite the open source
license of the code. I am not a lawyer and I suspect that the license that they
use would not hold up at least in European court, but I don't know. For that
exact reason, I never agreed to their license. This is the main reason this fork
even exists.

If you do want to contribute patches, you will have to accept full
responsibility for ensuring and warranting that you are legally allowed to do
so.

## You are the community

Gram is proudly open source, in spirit, not just in words. That said, we have
strong opinions about what we want to include in the editor. For example, the
main reason for this fork from Zed is to remove certain "features" that we
disagree with, morally. However, you are of course free to make it your own in
any way you see fit.

There is no official discord or reddit community, but there is an XMPP chat for
Gram at [gram@rooms.slidge.im][xmpp-link]. Any XMPP client should be able to
connect, and there is a [basic web UI][xmpp-webui] available. There are chat
logs available in an [online archive][xmpp-archive] as well.

[xmpp-link]: xmpp:gram@rooms.slidge.im?join
[xmpp-webui]: https://slidge.im/gram/#/guest?join=gram@rooms.slidge.im
[xmpp-archive]: https://rooms.slidge.im:5281/muc_log/gram/

## Strict No AI/LLM Policy

No more AI. I used to have a milder version of this statement here before, which
I wrote early on when I wasn't really aware of "vibe-coding" as such and was
mostly annoyed purely at the chatbox / autocomplete version of AI. That was bad
enough, but I really am not a fan of what that has become (in March 2026 when I
am writing this). I have copied this policy from the [Zig language project Code
of Conduct][zig-coc]:

> No LLMs for issues.
>
> No LLMs for pull requests.
>
> No LLMs for comments on the bug tracker, including translation. English is
> encouraged, but not required. You are welcome to post in your native language
> and rely on others to have their own translation tools of choice to interpret
> your words.

The Zed code base contains a lot of AI-generated code. It doesn't need a single
line more.

[zig-coc]: https://ziglang.org/code-of-conduct/
