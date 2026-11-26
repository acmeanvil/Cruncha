# ADR-0029: A debug log of interface actions and backend operations, with hang detection and a log viewer

**Status:** Accepted (2026-11-26)
**Date:** 2026-11-26
**Deciders:** acmeanvil

## Context

On 2026-11-26 the app froze after a template was inserted into a new document, and nothing recorded why:
- Cruncha writes no log file.
- `tauri dev`'s output stays in its terminal.
- The web view's console is gone with the window.

The backend reproduced the insert without trouble (0.3 s), so the cause is somewhere a log would have shown: the page, a command waiting on the session lock, or the app shell.

The owner asked for "an interface and operation debug log and log viewer similar to what is in Puzzle". Puzzle's (`src/puzzle/gui/errors.py`, `timing.py`, `log_viewer.py`) is small and plain:
- **The file:** a line-buffered text file in the user data folder, rotated at 2 MB at startup with one old generation kept.
- **What it records:** uncaught errors with their tracebacks and native crash stacks; warnings; user actions slower than 100 ms, broken into their steps; and a stall watch that logs "the window was held N ms (action)".
- **The viewer:** opened from Help → Log. It has "Errors only", search with every match marked, previous and next problem, Refresh, Copy all, Save as… and Open folder.

Cruncha has two halves that can each hang, the Svelte page and the Rust backend. They meet in Tauri commands, which mostly take the one `Session` lock. A freeze is either the page's thread busy, or a command waiting on (or holding) that lock.

## Decision

### 1. One log file, written by the backend

- **Where:** `~/Library/Logs/io.github.acmeanvil.cruncha/cruncha.log` (Tauri's `app_log_dir`), appended and flushed line by line, so a crash loses nothing.
- **Rotation:** at startup, past 2 MB it becomes `cruncha.old.log`. One old generation is kept, as in Puzzle.
- **The line format:** plain text, one record per line, readable without the viewer.
  - Example: `2026-11-26T10:07:03.406 WARN  ui.stall  the window was held 4210 ms (Insert template Bolt shear and tension)`.
  - The fields are the local time to the millisecond, the level (ERROR, WARN, INFO, DEBUG), a category (`app`, `ui`, `ui.action`, `ui.stall`, `cmd`, `cmd.slow`, `lock`, `engine`, `python`, `rust`, `mcp`, `agent`, `store`, `report`) and the message.
  - A multi-line record (a panic, a stack) continues on lines indented by two spaces.
- **Session markers:** `=== Cruncha <version> started (pid N) ===` at startup, and `=== stopped ===` on a clean quit. A session without a stop marker ended in a crash or a kill; the next start logs that.
- **Implementation:**
  - `tracing` (already in the tree through rmcp and tokio's ecosystem) with `tracing-subscriber` (MIT). The file layer is our own: a few lines, not `tracing-appender`.
  - The same records also go to an in-memory ring buffer (the last 5000), which feeds the viewer.
  - `RUST_LOG`-style filtering works through the `CRUNCHA_LOG` environment variable. The default is INFO and above to the file, and DEBUG in the ring buffer.

### 2. What is logged

**Backend:**
- **Every Tauri command:** its name, its document's short id, and how long it took. Under 100 ms it is a DEBUG record (ring buffer only); otherwise it is an INFO `cmd.slow` record with the time spent waiting for the session lock and the time spent running.
  - The slow threshold is set by `CRUNCHA_SLOW_MS`; 0 logs every command at INFO.
  - Arguments are not logged, only counts (cells, bytes).
- **The lock watchdog:**
  - The `Session` lock is taken through a wrapper that records which command holds it and since when.
  - A watchdog thread checks once a second. When one command has held the lock for 5 s, it logs a WARN `lock` record naming the command and the commands waiting behind it, and again every 10 s while the command still holds it.
  - This is the record that would have named Wednesday's freeze if the backend held it.
- **Panics:** a panic hook logs each panic, with its thread, location and backtrace, as ERROR `app`.
- **Operations:**
  - opening, saving and closing a document (version, cells, time);
  - evaluation: cells evaluated, errors, time;
  - Python and Rust runs (start, end, timeout, worker restart, and a crash with the worker's stderr tail);
  - agents: a session starts or ends, MCP tool calls by name, proposals made and decided;
  - reports;
  - prunes and archives;
  - template inserts and renames.
  - Each is one line at INFO, with failures at WARN or ERROR and the message the user saw.

**Front end** (sent to the backend in batches every 500 ms through a `log` command, and at once for errors):
- **User actions:** menu items, toolbar buttons, shortcuts, dialog decisions (Insert, Paste, Accept, Reject), tab switches, opening a panel. Each is INFO `ui.action` with the action's name only: `Insert template structural.bolts.shearTension 1.1.0 (19 cells)`.
- **Errors:** uncaught errors and unhandled promise rejections in the page, with their stack (ERROR `ui`). So are a command's rejected call (WARN `cmd`, the command and the message) and any message the app showed the user as an error.
- **The stall watch:** a 100 ms timer in the page. A tick that arrives at least 500 ms late is a WARN `ui.stall` record: the window was held N ms, and the last action, if it ended in the last 2 s. Svelte's effect-loop errors are caught and logged with the component.

**Privacy:** the log has no document contents (no sources, values or prose), no API keys, tokens or Authorization headers, and no file contents.
- Paths are logged; a document is named by its file name and short id.
- The agents' MCP tool calls are logged by tool name and cell ids, without their text.

### 3. The log viewer

- **Where:** a **Log** tab in the bottom panel (⌘J, beside Variables and Proposals), and Help → Debug log (⌘⌥L). It works with no document open.
- **It shows:** the ring buffer's records, live (pushed with a `log` event, no polling), newest at the bottom; following the tail unless the reader scrolls up.
- **Filters:**
  - level: Errors, Warnings and errors, Info, All (Debug included);
  - category, multi-select;
  - text search, with matches marked, and next and previous match;
  - Previous and next problem;
  - a count of errors and warnings.
- **Actions:**
  - Copy (shown records);
  - Save as…;
  - Open folder (Finder at the log file);
  - Clear view (the file is kept);
  - "Earlier sessions", which reads the file (the last 400 kB, as Puzzle does) for records before this start, including a crashed session's last lines.
- **The status bar** gets a small red dot while new errors are unseen. Clicking it opens the Log tab.

### 4. With the cleanup in `scripts/run-app.sh`

- When the script restarts the app after a crash, it appends a `=== run-app.sh: the app exited with status N; cleaned up and restarting ===` line to the log file. The record sits in the same place as the app's own.
- A frozen app is stopped with Ctrl-C. The next start's "previous session ended without a stop marker" line, the last `ui.stall` or `lock` records, and the crash's last lines are all in the viewer's "Earlier sessions".

## Options Considered

### Option A: tracing + our file layer + ring buffer + an in-app viewer (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: a logging module, a lock wrapper and watchdog, a command timing wrapper, a page-side logger and stall watch, a viewer tab |
| Cost | One small MIT dependency (`tracing-subscriber`); `tracing` is already built |
| Scalability | A 2 MB file and 5000 records in memory |
| Team familiarity | Puzzle's design, and looking_glass's `applog` (its hang detection and stack files) |

**Pros:**
- one place for both halves;
- the watchdog names the command holding the lock;
- plain text that survives a crash;
- live in the app.

**Cons:** every new command and operation needs its log line, which reviews have to check.

### Option B: `tauri-plugin-log`

**Pros:** ready-made, writes the page's console and Rust logs to one file.
**Cons:**
- a plugin dependency for little gain;
- no lock watchdog or stall watch;
- no structured categories, or ring buffer for a live viewer;
- the viewer is still ours.

Rejected.

### Option C: OS logging (`os_log`) and Console.app

**Pros:** no file to manage.
**Cons:**
- hard for the owner to read or send;
- no in-app viewer;
- macOS only.

Rejected.

## Consequences

- **The next freeze or crash** leaves the actions before it, the slow command or the held lock, and the stall in the log. "Check the logs" has an answer.
- **Changes that come with it:**
  - every Tauri command goes through a timing wrapper;
  - the `Session` lock goes through a wrapper (the same `tokio::sync::Mutex` underneath);
  - the page calls a small `log` module for actions;
  - CLAUDE.md gains a rule: new commands and operations log a line, and the log never holds document contents or secrets.
- **To revisit:**
  - a stack dump of all threads while the lock watchdog fires, as looking_glass's hang files do;
  - a "Save a bug report" bundle (log, versions, settings without keys);
  - a structured JSON log, if a tool ever needs it.

## Action Items

1. [x] `app/src-tauri/src/applog.rs`: the file layer (location, rotation, session markers, flush per line), the ring buffer, `CRUNCHA_LOG`, the panic hook, the crashed-last-session line; tests for rotation, format and the privacy filter. *(2026-11-26: `applog::init`, `stopped`, `Record`, `redact`, `bridge_tracing` (dependencies' tracing WARN and ERROR events join the log under their crate's name); its own file layer, as decided; tests cover the line format, the crashed-session check, the tail read and redaction. Every `eprintln!` note in the backend now goes to the log.)*
2. [x] Command timing (`cmd`, `cmd.slow`, `CRUNCHA_SLOW_MS`) and the session-lock wrapper with its watchdog (`lock`), with a test that a held lock is reported. *(2026-11-26: `SessionLock`/`Guard`: the lock is taken as `s.lock("<command>")` at all 85 sites, so the command's time is the guard's (waiting and holding), logged at DEBUG, or at INFO as `cmd.slow` past `CRUNCHA_SLOW_MS`; commands that don't take the session aren't timed. The watchdog thread is `cruncha-lock-watchdog`.)*
3. [ ] Operation lines: documents, evaluation, Python and Rust runs, agents and MCP tools, reports, prune and archive, template inserts and renames. *(Partial, 2026-11-26: `applog::Op` lines for open, create, commit, restore, evaluate, trim, Python runs, proposals made, accepted and rejected, and templates prepared for agents; agents' MCP calls take the session as "agent tool call". Still to do: each MCP tool by name, reports, archive, Rust runs, renames.)*
4. [x] Front end: `log.ts` (actions, errors, batching), the stall watch, logging the toolbar, menus, dialogs and shortcuts; vitest for batching and the stall watch. *(2026-11-26: `log.ts` (installed in `main.ts`), `api.ts` logs a rejected command as `ui.cmd`; the stall watch skips ticks while the window is hidden, since macOS slows its timers then.)*
5. [x] The viewer: the bottom panel's Log tab and Help → Debug log (⌘⌥L), with filters, search, problems, Copy, Save as…, Open folder, Earlier sessions, and the status bar's error dot. *(2026-11-26: `LogViewer.svelte`, `logView.ts`, `logFeed.svelte.ts`; Save as… writes through a `log_save` command. There is no separate status bar, so the dot sits after the save status in the header. The bottom panel (⌘J) now opens with no document, so the log is readable on the start screen.)*
6. [x] `scripts/run-app.sh` writes its restart line to the log; CLAUDE.md's logging rule. *(2026-11-26.)*
