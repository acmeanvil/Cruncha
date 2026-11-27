# ADR-0030: Windows: the Python worker's sandbox, cell interrupts and packaging

**Status:** Accepted
**Date:** 2026-11-26 (accepted 2026-11-27)
**Deciders:** acmeanvil

## Context

The owner asked what packaging Cruncha for Windows takes, then to "move forward with the fixes and use the CI runner to check". Most of the stack is portable:
- Tauri 2 uses WebView2 on Windows.
- deltalake's `native-tls` uses SChannel.
- `keyring` 4.2 already pulls in the Windows Credential Manager backend.
- wasmtime and Typst both run on Windows.
- rustup has `*-pc-windows-msvc` hosts, and uv installs python-build-standalone for Windows.
- The figure frames' Windows origin, `http://cruncha-figure.localhost`, is already in ADR-0020 and in the CSP.

Three things don't port:

1. **The worker's sandbox.** ADR-0005 item 2 confines the worker with `sandbox-exec`, which only exists on macOS. It reads only its environment, the worker package and system files; it writes only its scratch directory; and it has no network or subprocesses. Windows has no such command. Without one, a trusted document's Python would run with the user's full rights, which CLAUDE.md rules out.
2. **Interrupting a cell.** Stop and timeouts send SIGINT, and a kill sends SIGKILL. Windows can't signal another process. `GenerateConsoleCtrlEvent` only reaches processes on the caller's console, and a GUI app has none. On Windows, Stop and timeouts did nothing.
3. **Platform assumptions in the code.** These were all fixed with this ADR, because none of them is a decision:
   - `~/Library/...` paths in tools and tests;
   - `bin/cargo` without `.exe`;
   - `/bin/kill`, `open -R` and `hostname`;
   - a console window for every child process;
   - no menu outside macOS;
   - ⌘ in the labels;
   - CRLF checkouts that change the bytes the tests read.

## Decision

### 1. Interrupts by message (done with this ADR)

- **The interrupt:** the worker takes an interrupt mode as its first argument.
  - `signal` is today's SIGINT behaviour, and stays the default on Unix.
  - `message` is the Windows default. A reader thread in the worker owns stdin: an `{"op": "interrupt"}` message signals the main thread with SIGINT (`signal.pthread_kill` on Unix, `signal.raise_signal` on Windows; `_thread.interrupt_main()` only as a fallback, because it doesn't wake a blocking call such as `time.sleep`), and every other request waits in a queue for the main loop.
  - The app writes the interrupt under the same lock that requests write their frames under, so an interrupt never splits a frame.
  - An interrupt that arrives while no cell runs is raised during the loop's timed wait and dropped. It never reaches the next cell.
- **The kill:** `TerminateProcess` (through `windows-sys`, MIT or Apache-2.0).
- **Testing:** `message_interrupts_stop_a_cell_and_the_worker_keeps_going` exercises message mode on every platform, including macOS.

### 2. No Python cell runs on Windows until the sandbox exists (done with this ADR)

- **Runs:** the run loop answers every Python job on Windows with "Python cells don't run on Windows yet: the worker's sandbox isn't built (ADR-0030)". It never starts a runner there.
- **Scanning and renaming still work.** They parse cells with `ast` and `tokenize` and never execute them. So names, the graph and pasted renames behave as on macOS.
- **Stored results still show**, as they do for any computer that can't run a cell.

### 3. The Windows sandbox (proposed; not built)

The worker runs in an **AppContainer** with no capabilities, inside a **Job Object**:
- **The AppContainer** has no network (no `internetClient` capability) and no access to the user's files.
  - The environment directory, the worker package and the scratch directory are granted to its SID by ACL when the environment is set up.
  - System libraries and fonts are readable to AppContainers by default.
- **The Job Object** sets `JOB_OBJECT_LIMIT_ACTIVE_PROCESS = 1`, so the worker can't start other processes. It also sets `KILL_ON_JOB_CLOSE`, so the worker dies with the app.
- **Process creation:** a small launcher, `CreateProcessW` with `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES`. It goes in a `sandbox_windows.rs` beside `sandbox.rs`, through `windows-sys`.
- **Testing:** `the_sandboxed_worker_computes_but_reaches_nothing` runs on Windows CI against the same probes (home directory, `/etc/hosts`-style system file, write, network, subprocess).

### 4. Packaging (proposed)

- **Build:** on a Windows runner, never cross-built. `bundle.targets: "all"` already gives an MSI (WiX) and an NSIS `.exe`, and `icons/icon.ico` exists.
- **WebView2:** `bundle.windows.webviewInstallMode: downloadBootstrapper`.
- **Python bootstrap:** uv currently comes from a system Python. Many Windows machines have none, so download uv's standalone `uv-x86_64-pc-windows-msvc.zip`, verified against its published SHA-256, as the Rust setup does with `rustup-init`.
- **Signing:** an Authenticode certificate or Azure Trusted Signing; without one, SmartScreen warns on every download. This is the owner's cost decision.
- **CI:** a `windows` job builds the workspace, runs clippy and the tests, and runs the worker's tests against the locked environment. The lock is compiled on macOS, but its hashes cover every wheel of each pinned version, and a Windows resolve on 2026-11-26 found no Windows-only packages.

## Options considered

### Option A: AppContainer + Job Object (chosen for §3)

| Dimension | Assessment |
|---|---|
| Complexity | Medium: one launcher, plus ACL grants at setup |
| Strength | Comparable to the `sandbox-exec` profile: no network, files by grant only, no child processes |
| Dependencies | `windows-sys` only |

**Pros:**
- It is Windows' own app sandbox (Edge and Chrome renderers use it).
- It needs no admin rights or installer service.

**Cons:**
- ACL grants have to be kept in step with the environment directory.
- Some packages that probe the registry or the user profile may need small fixes, as matplotlib's `MPLCONFIGDIR` did on macOS.

### Option B: a restricted token or low integrity level only

**Pros:** simpler.

**Cons:**
- Low integrity still reads most of the user's files and keeps the network. That doesn't meet ADR-0005's bar.

### Option C: Windows Sandbox or WSL

**Cons:**
- It needs optional Windows features and admin rights.
- It starts slowly.
- It isn't on Home editions.

Rejected.

### Option D: ship Python cells off on Windows (the interim, §2)

**Pros:** safe now.

**Cons:** Python cells, the most-used kind after calcs, don't run.

## Consequences

**Easier:**
- Calcs, prose, reports, templates, history, Rust cells and agents work on Windows. Rust cells run in wasmtime with an empty WASI context, so they need no OS sandbox.
- Interrupts no longer depend on signals.

**Harder:**
- Two sandboxes to keep equivalent.
- A Windows CI job: about 67 minutes from a cold cache, less once the build cache is kept.

**To revisit:**
- Linux, which still runs the worker unsandboxed (ADR-0005 item 2, seccomp).

## Action items

1. [x] Interrupt by message and kill by `TerminateProcess`; tested on every platform.
2. [x] No Python cell runs on Windows until item 3; scanning and renaming still work.
3. [ ] The AppContainer + Job Object sandbox, with the sandbox test on Windows CI.
4. [x] Portable paths, console-less child processes, a File menu outside macOS, shortcut labels per platform, LF checkouts.
5. [ ] Packaging: WebView2 bootstrapper, standalone uv download, signing, a release workflow.
6. [ ] The agent connection file (`server.json`) gets an explicit owner-only ACL on Windows (Unix: 0600).
