# Troubleshooting

This guide covers common troubleshooting techniques for Gram. Sometimes you'll
be able to identify and resolve issues on your own using this information. Other
times, troubleshooting means gathering the right information—logs, profiles, or
reproduction steps—to help developers diagnose and fix the problem.

> **Note**: To open the command palette, use `cmd-shift-p` on macOS or
> `ctrl-shift-p` on Windows / Linux.

## Retrieve Gram and System Information

When reporting issues or seeking help, it's useful to know your Gram version and
system specifications. You can retrieve this information using the following
actions from the command palette:

- {#action gram::About}: Find your Gram version number
- {#action gram::CopySystemSpecsIntoClipboard}: Populate your clipboard with
  Gram version number, operating system version, and hardware specs

## Gram Log

Often, a good first place to look when troubleshooting any issue in Gram is the
Gram log, which might contain clues about what's going wrong. You can review the
most recent 1000 lines of the log by running the {#action gram::OpenLog} action
from the command palette. If you want to view the full file, you can reveal it
in your operating system's native file manager via {#action
gram::RevealLogInFileManager} from the command palette.

You'll find the Gram log in the respective location on each operating system:

- macOS: `~/Library/Logs/Gram/Gram.log`
- Windows: `C:\Users\YOU\AppData\Local\Gram\logs\Gram.log`
- Linux: `~/.local/share/gram/logs/Gram.log` or `$XDG_DATA_HOME`

> Note: In some cases, it might be useful to monitor the log live, such as when
> [developing a Gram extension](./extensions/developing-extensions.md). Example:
> `tail -f ~/Library/Logs/Gram/Gram.log`

The log may contain enough context to help you debug the issue yourself, or you
may find specific errors that are useful when filing an issue.

## Performance Issues (Profiling)

If you're running into performance issues — such as hitches, hangs, or general
unresponsiveness — having a performance profile will help zero in on what is
getting stuck.

## Minidumps

If Gram is crashing you can set `GRAM_GENERATE_MINIDUMPS=1` when running Gram to
get the editor to generate a crash dump file that can be analyzed to debug the
issue. See [debugging-crashes.md](./development/debugging-crashes.md) for more
details.

### macOS

Xcode Instruments (which comes bundled with
[Xcode](https://apps.apple.com/us/app/xcode/id497799835)) is the standard tool
for profiling on macOS.

1. With Gram running, open Instruments
1. Select `Time Profiler` as the profiling template
1. In the `Time Profiler` configuration, set the target to the running Gram
   process
1. Start recording
1. If the performance issue occurs when performing a specific action in Gram,
   perform that action now
1. Stop recording
1. Save the trace file

<!--### Windows-->

<!--### Linux-->

## Startup and Workspace Issues

Gram creates local SQLite databases to persist data relating to its workspace
and your projects. These databases store, for instance, the tabs and panes you
have open in a project, the scroll position of each open file, the list of all
projects you've opened (for the recent projects modal picker), etc. You can find
and explore these databases in the following locations:

- macOS: `~/Library/Application Support/Gram/db`
- Linux and FreeBSD: `~/.local/share/gram/db` (or within `XDG_DATA_HOME` or
  `FLATPAK_XDG_DATA_HOME`)
- Windows: `%LOCALAPPDATA%\Gram\db`

The naming convention of these databases takes on the form of
`0-<release_channel>`:

- Stable: `0-stable`
- Dev: `0-dev`

While rare, we've seen a few cases where workspace databases became corrupted,
which prevented Gram from starting. If you're experiencing startup issues, you
can test whether it's workspace-related by temporarily moving the database from
its location, then trying to start Gram again.

> **Note**: Moving the workspace database will cause Gram to create a fresh one.
> Your recent projects, open tabs, etc. will be reset to "factory".

If your issue persists after regenerating the database, please file an issue.

## Language Server Issues

If you're experiencing language-server related issues, such as stale diagnostics
or issues jumping to definitions, restarting the language server via {#action
editor::RestartLanguageServer} from the command palette will often resolve the
issue.

## Unable to watch for file system changes

By default, Gram uses a method native to the current platform to watch for file
system changes (inotify on Linux, FSEvents on Mac). For some file systems, this
method does not work, for example for NFS or SSHfs mounts, or when connecting to
a Windows remote.

It is possible to use a poll-based file watcher on these file systems instead.
The polling watcher is more resource-intensive than the native file watchers,
but works in more scenarios.

To use the poll watcher, set the `file_watcher` option in the Worktree settings.

For example:

```json
{
  "file_watcher": {
    "mode": "poll", // "native" | "poll"
    "poll_interval_ms": 2000 // 500-30000ms
  }
}
```
