# Panic policy

## What a panic means

A panic ends the process. A panic means an invariant broke: after that the
editor's state can't be trusted, so nothing recovers into an `E` error at a
boundary. The job is to lose nothing and get out. A panic loses no more than a
deadly signal (`SIGTERM`, `SIGHUP`, ...) does.

C's undefined behaviour became Rust panics when the code was transpiled:
`.expect(..)`, `.unwrap()`, `unreachable!`, slice indexing. Some of those sites
are invariants. Others can be reached from user input, and those have to
become errors (see [the census](#census)).

## Mechanism: the default unwind, and a hook that aborts

The profiles keep the default `panic = "unwind"`.
`startup::panic_hook::install_panic_hook` (called first thing in
`startup::main`) installs a hook that never returns. It cleans up and calls
`abort()` itself. Nothing unwinds, so no destructor runs over the broken state,
and no unwind reaches an `extern "C"` frame.

### Why not `panic = "abort"`

Lua errors travel through Rust frames. LuaJIT on x86-64 raises an error with
the system unwinder (`_Unwind_RaiseException`). An error raised inside a C
function written in Rust (`lua/stack/dispatch.rs`'s `extern "C-unwind"`
trampoline, `Stack::entry`) unwinds through those Rust frames on its way to the
`lua_pcall` that catches it. With `panic = "abort"` those frames refuse a
foreign unwind, as the `C-unwind` rules specify for an abort build. Phase 1 of
the unwind then fails, and LuaJIT calls its panic function and exits 1.

The experiment (2026-11-15) put `panic = "abort"` in `[profile.dev]` and
`[profile.release]` and ran a debug build:

- `pcall(error, "boom")` still worked. That error is raised and caught entirely
  inside LuaJIT.
- Every error raised from a Rust C function killed the process with
  `PANIC: unprotected error in call to Lua API (...)`, exit 1. That covered
  `pcall(vim.api.nvim_buf_get_lines, 9999, 0, -1, true)`, `pcall(vim.base64.decode, {})`,
  `pcall(vim.fn.nosuch)`, `pcall(vim.cmd, "nosuchcmd")` and
  `pcall(vim.str_utfindex, {})`.
- `just functionaltest test/functional/lua`: 58 failures in the first 84 spec
  files, all of them this LuaJIT panic. Then a spec hung and the run was
  stopped.

So `panic = "abort"` is not an option while Lua errors unwind through Rust.
The hook gets the same "a panic ends the process" without touching the unwind
that Lua relies on. A panic inside an `extern "C"` callback would abort at the
boundary anyway, but the hook runs first: it is called before any unwinding
starts.

### Tests

`cargo test` installs no hook (only the binary's `main` does), and the test
harness unwinds, so a failing `assert!` fails its own test case.
`std::panic::catch_unwind` in lib tests (`guard.rs`'s
`a_panic_releases_the_guard`) keeps working with no extra setup. The FFI unit
suite (`test/unit`) calls exported symbols without running `main`, so it has no
hook either.

## What the hook guarantees

In order:

1. **The log.** The panic message, its location, the thread name and a
   forced backtrace are appended to `$NVIM_LOG_FILE`, the file `log_init`
   chose. When the variable is unset, they go to stderr instead. This bypasses
   `logmsg!`, because `logmsg!` reads editor state and refuses re-entry. The
   line starts `ERR <unix seconds> nvim.<pid> panic: thread '<name>' panicked
at <file>:<line>:<col>:`. In a release binary the backtrace has function
   names but no line numbers (`strip = "debuginfo"`).
2. **On the main thread:**
   - The `GlobalCell` borrow tracker is cleared (debug builds). The panic may
     have fired inside a `with_mut`, and the cleanup below must not trip over
     a borrow whose owner never runs again.
   - `begin_emergency_exit()`: if a deadly signal, an out-of-memory exit or an
     earlier panic is already preserving, the hook goes straight to `abort()`.
     Otherwise deadly signals are ignored from here on.
   - In a TUI client process, `ui_client_stop()` gives the terminal back
     (leaves the alternate screen and raw mode).
   - The standard panic message goes to stderr (`RUST_BACKTRACE` still works).
   - In the editor (server, `--embed`, `--headless`, plain), the swap files
     are written out and the memfiles closed without being deleted. This is
     `preserve_swap_files`, the same code `preserve_exit` runs for a deadly
     signal.
   - stdin is put back to blocking.
3. **`abort()`.** The parent sees `SIGABRT`. A shell reports 134, a job's exit
   status is 134 (`128 + 6`), and a TUI client whose server panicked exits 134,
   because it passes its server's status on. Unlike the deadly-signal path,
   there is no `getout()`: no autocommands, no ShaDa write, no `:defer`. Those
   run user code or write state from a process whose state is broken.

A panic raised while the hook runs does not come back into it: the standard
library aborts on a panic inside a panic hook. Either way the process ends,
and the log already has the first message.

### Known gaps

- **A panic off the main thread** (libuv pool work, `vim.uv.new_thread`
  states) logs, writes stderr and aborts. It touches no editor state, because
  that state belongs to the main thread, which is still running. Swap files
  are only as recent as their last sync (`'updatecount'`/`'updatetime'`), and
  a TUI client's terminal is not restored.
- **A second thread panicking while the first runs the hook** waits up to
  10 s for the first to abort, then aborts itself.
- **If the cleanup itself breaks** (the panic was inside the memfile code, or
  a non-`GlobalCell` tracker such as a value's edit loan objects), the
  in-hook panic aborts. The swap files on disk are then as recent as the last
  sync, like after `SIGKILL`.
- In a release build, the borrow that was live at the panic is aliased by the
  cleanup. Its owner never resumes, so nothing observes the aliasing.

### The test

`test/functional/ex_cmds/swapfile_preserve_recover_spec.lua`, "panic in the
server preserves the swap file and logs the message", does the following:

1. Starts a TUI client in a `:terminal`, with its own log.
2. Edits a buffer below `'updatecount'`.
3. Calls `nvim__panic`.
4. Asserts `[Process exited 134]`, the message in the log, and that a second
   Nvim recovers the edit from the swap file.

`nvim__panic(message)` is the deliberate trigger: an internal `nvim__*` method
(no metadata, no compatibility, like `nvim__id`/`nvim__screenshot`) that does
nothing except `panic!`. Any API client could already end the server
(`:qa!`, `os.exit()`), so it adds no new power.

## The rule for authors

- **A broken invariant** (something no input can reach if the rest of the
  editor is correct): `expect("…")` with a message that names the violated
  invariant, as in `.expect("the editor always has a window")`. The message
  is what the log shows. Write it for the person reading the panic, not as
  "should not fail". `unreachable!("…")` and `assert!(…, "…")` follow the same
  rule.
- **Anything input can reach** (a user's text, a file, an RPC argument, an
  option value, a Lua value, a race with an autocommand): an error. Use
  `Result`, the family's error type, or `Err(Failed)` after a message. Never
  a panic.
- Never catch a panic to turn it into an error. The hook ends the process
  before any `catch_unwind` could see one, and that's the point.

## Census

_To be filled by the census: `expect`/`unwrap`/`unreachable!`/indexing by
module, sorted by whether user input reaches each site. Rough counts at
`cbf4850257` (`crates/nvim/src`, tests included): ~1,395 `.expect(`,
139 `.unwrap()`, 167 `unreachable!`, 64 `panic!`._
