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

### Off the main thread

Rust code runs on other threads in two ways, both started from Lua:
`vim.uv.new_thread()` runs a fresh Lua state on its own OS thread, and
`vim.uv.new_work()` runs one on a libuv pool thread. Each state gets the
Rust-backed helpers (`vim.mpack`, `vim.json`, `vim.base64`, `vim._get_runtime`,
the `require` loader), and its error path posts back to the main loop. The
editor starts no `std::thread` of its own and has no TUI thread: the TUI is
a separate process (`ui_client`), single-threaded.

Such a thread must not touch editor state, so its hook does not clean up.
It logs the panic (step 1), leaves its message for the main thread, posts
the cleanup as a job on the main loop's fast queue (the same inbox and
`uv_async_send` every worker uses to post back), and waits. The main thread
runs the job the next time it polls its loop: between commands, during
`:sleep`, and at every `os_breakcheck()` in a long Vimscript loop. The job
does what step 2 does — the TUI client gives the terminal back, the
message is printed, the swap files are written out — and aborts. The
panicking thread waits 5 s for the job to start and then 10 s for it to
finish, then prints the message and aborts on its own (see the gap below).
Two panicking threads both post; `begin_emergency_exit` lets one cleanup
through. A main-thread panic while a worker waits does the cleanup itself.

`nvim__panic(message, true)` panics on a new thread and returns, so the
test below covers this path too. With the job never posted, the test times
out waiting for `[Process exited 134]`: the server sits for the 5 s grace
instead.

### Known gaps

- **A panic off the main thread whose main thread never polls.** The
  hand-off (below) needs the main thread to reach its event loop within
  5 s. A tight Lua loop with no `os_breakcheck()`, a blocking system call,
  or the main thread joining the very thread that panicked keeps it away;
  the panicking thread then prints its message and aborts, and the swap
  files are as recent as their last sync (`'updatecount'`/`'updatetime'`),
  as after `SIGKILL`.
- **No test panics the TUI client process itself.** The client loads no
  config and talks only to its server, so there is no honest way to ask it to
  panic; a trigger would be a test-only hook in its input path. The client
  path (`ui_client_stop()`, then the message) is the same code the
  deadly-signal path runs there.
- **If the cleanup itself breaks** (the panic was inside the memfile code, or
  a non-`GlobalCell` tracker such as a value's edit loan objects), the
  in-hook panic aborts. The swap files on disk are then as recent as the last
  sync, like after `SIGKILL`.
- In a release build, the borrow that was live at the panic is aliased by the
  cleanup. Its owner never resumes, so nothing observes the aliasing.

### The test

`test/functional/ex_cmds/swapfile_preserve_recover_spec.lua`, "panic in the
server preserves the swap file and logs the message" and "panic off the main
thread ...", does the following:

1. Starts a TUI client in a `:terminal`, with its own log.
2. Edits a buffer below `'updatecount'`.
3. Calls `nvim__panic`, on the main thread or on a new one.
4. Asserts `[Process exited 134]`, the message in the log, and that a second
   Nvim recovers the edit from the swap file.

`nvim__panic(message, thread)` is the deliberate trigger: an internal
`nvim__*` method (no metadata, no compatibility, like
`nvim__id`/`nvim__screenshot`) that does nothing except `panic!`, on a new
thread when `thread` is true. Any API client could already end the server
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

Taken 2026-11-15. `just ratchet --dimension panic_sites` prints today's
counts per file and module; `--dimension bare_panics` lists each bare site.

### What was counted

Every explicit panic outside test code (inline `#[cfg(test)]` modules,
files pulled in by `#[cfg(test)] mod x;`, `build.rs` are skipped) and
outside `const` assertions, which fail the build instead:

| kind                                          |     sites |
| --------------------------------------------- | --------: |
| `.expect(..)`                                 |     1,213 |
| `unreachable!`                                |       165 |
| always-on `assert!`/`assert_eq!`/`assert_ne!` |       122 |
| `panic!`/`todo!`/`unimplemented!`             |        42 |
| `.unwrap()`                                   |         2 |
| **explicit**                                  | **1,544** |
| bare (says nothing about what broke)          |        14 |
| indexing/slicing expressions (heuristic)      |    ~9,700 |

Arithmetic overflow is not a panic in a release build (no
`overflow-checks`): it wraps, as C's signed overflow does in practice. A
debug build panics on it. It is classed by probe, not by site.

### Method

1. **Reading.** Every explicit site was read in context and classed
   _input_ (some user input can make it fire), _invariant_ (the code around
   it guarantees it) or _unknown_, with the error shape its caller could
   carry. Module families were read separately; the input sites were then
   reproduced where that was cheap.
2. **Probes** (headless debug binary, 4 GiB address-space cap):
   - every `nvim_*` function, one argument at a time at an extreme
     (0, -1, ±2^31, ±2^53, ±2^63, huge/NUL/invalid strings, odd
     containers, every keyset key);
   - every Vimscript builtin with 1–4 arguments, one at a time at an
     extreme (`v:numbermax`/`v:numbermin`, ±2^31, 2^32, odd strings, lists,
     dicts, blobs, `v:null`, funcrefs);
   - truncations and byte flips (every position up to 4 KiB, then every
     7th) of a ShaDa file, an undo file, a swap file, a `.spl` file, msgpack
     and JSON, read back through `:rshada!`, `:rundo`+`:undo`, `:recover!`,
     spell checking and suggestion, `msgpackparse()`/`vim.mpack.decode`,
     `json_decode()`/`vim.json.decode`;
   - per family: autocommand re-entrancy (an autocommand closing the
     window/buffer/tab or deleting the group being worked on), completion
     callbacks, Ex ranges and counts at the extremes, `'errorformat'` and
     hostile error files, escape sequences, decoration providers, a seeded
     marktree stress, ~100k random expressions through
     `nvim_parse_expression`.
3. **Indexing** cannot be classed site by site. It was ranked by where
   external bytes and user numbers arrive (the file readers, the RPC
   unpacker, the expression parser, the regexp engines, the vterm parser),
   and found by the probes. Every indexing panic the probes found is in
   that set.

### Result

| class                            | explicit sites |
| -------------------------------- | -------------: |
| input, reproduced                |             27 |
| input, judged by reading only    |              4 |
| input, only with ≥ 2 GiB of data |             10 |
| unknown                          |              3 |
| invariant                        |         ~1,500 |

Plus, found only by probing: 6 input-reachable index panics in release
(a `:terminal` program that saves the cursor or sets a top margin and is
then shrunk, a corrupt undo file, a corrupt `.spl` tree, `syn match …
lc=-1`, a dangling highlight link), about 35 debug-only overflows on user
numbers, and 7 `debug_assert!`s that user input reaches.

The input sites by module (every other module: all invariant):

| module                      | explicit | input | how it is reached                                                                                                                                             |
| --------------------------- | -------: | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `eval`                      |      189 |     7 | `filter()`/`map()` callback unlocking and shrinking its container; an API dict key with a NUL; an overridden internal Lua module; an `envp` entry with no `=` |
| `msgpack_rpc`               |       48 |     7 | an out-of-range handle in an RPC ext; `nvim_set_client_info` after `chanclose` in one `nvim_call_atomic`; ≥ 4 GiB replies                                     |
| `mpack`                     |       24 |     6 | a `vim.mpack` object used after calling its `__gc` by hand                                                                                                    |
| `memline`                   |       25 |     5 | ≥ 2 GiB lines only                                                                                                                                            |
| `highlight_group`           |        9 |     3 | `:hi ctermfg=` past 16 bits; a dangling `link` (both fixed)                                                                                                   |
| `shada`                     |       42 |     2 | an empty block register's width -1, written and read (fixed)                                                                                                  |
| `viml`                      |       34 |     2 | an unmatched `]`/`}` between two values (`1 2]3`), even typed at `<C-r>=`                                                                                     |
| `version`                   |        5 |     2 | `:version` after tampering with `jit`/`_VERSION`                                                                                                              |
| `api`                       |      130 |     2 | `ModeChanged` closing `nvim_set_current_win`'s target (fixed); `nvim__panic` (deliberate)                                                                     |
| `spellfile`, `spellsuggest` |       25 |     2 | a spell file with more than 32,767 `REP` items                                                                                                                |
| `terminal`                  |       22 |     1 | a deferred `TermRequest` reported against a new terminal on the same buffer                                                                                   |
| `help`, `highlight`         |        9 |     2 | an overridden help helper; a negative cterm colour under 'winblend' (unverified)                                                                              |

Converted since the census (each with a regression test): the two
`:terminal` resize panics (the resize clamps the saved cursor and drops a
margin left past the edge), the API dict key with a NUL (cut there, as
C's `strlen`), the out-of-range RPC handle (echoed as its 32 bits, as C
release), `nvim_set_client_info` on a channel the same request closed
(dropped), the overridden `vim._core` serverlist and help helpers (logged;
refused like a Lua error), `:version` after tampering with `jit`/`_VERSION`
(line left out), an `envp` entry with no `=` (skipped), a `TermRequest`
reported against a reopened terminal (no shift), and `syn match … lc=-1`
(no match past the line). Then the parser and walk family: an unmatched
`]`/`}` after an operand (`1 2]3`, `a is]!=b`) and its lambda-argument
siblings (`{a,{,1`, `{a,]->`) are parse errors; a `filter()`/`map()`/
`foreach()` callback that `:unlockvar`s and edits its container, and the
same edit from `indexof()`, `uniq()` and `matchfuzzy()` callbacks, are
walked through rather than read past; and a userdatum whose `__gc` plain
Lua called by hand (`vim.mpack` objects, `vim.regex`, every tree-sitter
object, and the nodes, cursors and matches that point into one) raises
"… has been collected" instead of panicking or using freed memory.
Then corrupt files: an undo file is refused (E825) when an entry's line
numbers fit no buffer, a sequence or save number could not be stepped, or
the header links do not make a tree (a cycle hung `:undo 0`); a spell
tree whose shared sub-tree index is not a node start, a cycle, or too deep
is E759; a `REP` list may hold its full 65,535 items; and a swap file's
page size above the maximum is refused like one below the minimum. A
length or count a file or peer claims no longer sizes an allocation before
the bytes are there (undo lines, ShaDa entries, spell strings and trees,
msgpack strings over RPC and in `vim.mpack`): the rule and its helpers are
`memory/claimed.rs`, and `test/functional/core/corrupt_files_spec.lua`
reads each such file in a child capped at 1 GiB.

What could not be classed: `viml/parser/expressions/ast.rs:182` (may fire
in release after the `1 2]3` family mis-wires a node; that family no
longer does, and the bounded enumeration that checked it never reached
the assert), and two low-
confidence underflows (`register/yank.rs:194`, `indent/mod.rs:430`). The
`debugger` `>frame +N` overflow could not be driven headless.

The pattern is clear. Explicit panics are almost all honest invariants:
1,500 of 1,544, with messages that name the invariant in all but about 150.
What input reaches is concentrated in a few families: values a user callback
or Lua override hands back unchecked, readers that size or index from a
file's own fields, and user numbers at the `int` extremes. The work list
for the conversion slices is kept outside the tree, with each site's
reproducer.
