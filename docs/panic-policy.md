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

## Allocation failure

A Rust allocation (`Vec`, `String`, `Box` growing) that `malloc` refuses
takes the same way out as `xmalloc`'s `E41`, not the standard library's
bare abort ("memory allocation of N bytes failed", no swap files, the
terminal left raw). Stable Rust has no allocation-error hook
(`alloc_error_hook` is nightly), so the global allocator (`allocator.rs`)
does it: after `malloc` it already answers, one null check, and on null an
out-of-line call into `startup/out_of_memory.rs`, which:

1. **Once only.** A second failure while the first is on its way out (the
   way out running dry, or another thread) aborts at once. So does one
   that finds a deadly signal or a panic already preserving.
2. **Frees a reserve.** `main` takes 256 KiB (`calloc`'d, untouched, so
   address space rather than memory) right after installing the panic
   hook. Released first, it gives the log line, the hand-off and the
   memfile sync room to allocate.
3. **Logs** `ERR <secs> nvim.<pid> out of memory: thread '<name>' failed
to allocate <N> bytes` (no backtrace: symbolising one allocates
   megabytes).
4. **Off the main thread**, hands the rest to the main thread exactly as
   a panic does ([above](#off-the-main-thread)), with the same 5 s / 10 s
   graces and the same gap.
5. **On the main thread**, runs the panic hook's cleanup: forget the debug
   borrow tracker, `ui_client_stop()` in a TUI client, print `E41: Out of
memory!`, write the swap files out (`Nvim: preserving files...`), put
   stdin back.
6. **Exits 1**, upstream's status for `E41` (`preserve_exit` → `getout(1)`),
   rather than aborting: the editor's invariants held, only memory ran
   out.

Two things differ from upstream's `xmalloc` failure path, on purpose:

- **No retry after freeing editor memory.** Upstream first calls
  `try_to_free_memory` (drop the scrollback, release memfile blocks) and
  tries again. Freeing those is sound only when nothing borrows them,
  which `xmalloc`'s callers promise (its `# Safety`) and a safe
  `Vec::push` cannot: the retry would run inside whatever borrow the
  pushing code holds. The `xmalloc` family itself still retries.
- **No `getout()`**: no `VimLeave`, no ShaDa write. The allocation failed
  in the middle of Rust code, possibly inside a `GlobalCell` borrow, so
  like the panic hook this runs no user code from there.

A request that asks to fail still fails: `memory::try_reserve_items` and
`try_grow_zeroed` make their `try_reserve` inside `startup::fallibly`, so
the allocator answers null and the caller reports `E342` and carries on
(`repeat('ab', v:numbermax)`, `repeat([1, 2], v:numbermax)`). A
`try_reserve` made outside it, including the standard library's own
(`Read::read_to_end`), takes the `E41` exit like any other request. A
process that never ran `main` (lib tests, the unit suite) keeps the
standard library's behaviour.

The success path is unchanged in cost: the null check sits where the
caller's own already was, and the failure branch is `#[cold]`,
`#[inline(never)]`.

`test/functional/ex_cmds/swapfile_preserve_recover_spec.lua`, "allocation
failure in the server preserves the swap file", runs a TUI client and its
server under `prlimit --as=1G`, asks for a 2 GB Blob, and asserts
`[Process exited 1]`, the log line, and that a second Nvim recovers the
edit from the swap file. It skips under ASan, whose shadow does not fit
the cap.

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

Taken 2026-11-15 and worked through by 2026-11-16: every input-reachable
site it found is converted or accepted below. `just ratchet --dimension
panic_sites` prints today's counts per file and module; `--dimension
bare_panics` lists each bare site.

### What was counted

Every explicit panic outside test code (inline `#[cfg(test)]` modules,
files pulled in by `#[cfg(test)] mod x;`, `build.rs` are skipped) and
outside `const` assertions, which fail the build instead:

| kind                                          | at the census |       now |
| --------------------------------------------- | ------------: | --------: |
| `.expect(..)`                                 |         1,213 |     1,208 |
| `unreachable!`                                |           165 |       165 |
| always-on `assert!`/`assert_eq!`/`assert_ne!` |           122 |       119 |
| `panic!`/`todo!`/`unimplemented!`             |            42 |        42 |
| `.unwrap()`                                   |             2 |         0 |
| **explicit**                                  |     **1,544** | **1,534** |
| bare (says nothing about what broke)          |            14 |         0 |
| indexing/slicing expressions (heuristic)      |        ~9,700 |    ~9,840 |

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

| class at the census              | explicit sites | outcome                                |
| -------------------------------- | -------------: | -------------------------------------- |
| input, reproduced                |             27 | converted, each with a regression test |
| input, judged by reading only    |              4 | converted (3) or found invariant (1)   |
| input, only with ≥ 2 GiB of data |             10 | accepted (below)                       |
| unknown                          |              3 | reclassified (below)                   |
| invariant                        |         ~1,500 | kept; messages name the invariant      |

Found only by probing, and also all converted: 6 input-reachable index
panics in release, about 35 debug-only overflows on user numbers (plus
the fold, 'diffexpr', undo-time and redraw-decoder siblings found while
fixing them), and 7 `debug_assert!`s user input reached.

The input sites by module, as found (every other module: all invariant):

| module                      | explicit | input | how it was reached                                                                                                                                            |
| --------------------------- | -------: | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `eval`                      |      189 |     7 | `filter()`/`map()` callback unlocking and shrinking its container; an API dict key with a NUL; an overridden internal Lua module; an `envp` entry with no `=` |
| `msgpack_rpc`               |       48 |     7 | an out-of-range handle in an RPC ext; `nvim_set_client_info` after `chanclose` in one `nvim_call_atomic`; ≥ 4 GiB replies                                     |
| `mpack`                     |       24 |     6 | a `vim.mpack` object used after calling its `__gc` by hand                                                                                                    |
| `memline`                   |       25 |     5 | ≥ 2 GiB lines only                                                                                                                                            |
| `highlight_group`           |        9 |     3 | `:hi ctermfg=` past 16 bits; a dangling `link`                                                                                                                |
| `shada`                     |       42 |     2 | an empty block register's width -1, written and read                                                                                                          |
| `viml`                      |       34 |     2 | an unmatched `]`/`}` between two values (`1 2]3`), even typed at `<C-r>=`                                                                                     |
| `version`                   |        5 |     2 | `:version` after tampering with `jit`/`_VERSION`                                                                                                              |
| `api`                       |      130 |     2 | `ModeChanged` closing `nvim_set_current_win`'s target; `nvim__panic` (deliberate)                                                                             |
| `spellfile`, `spellsuggest` |       25 |     2 | a spell file with more than 32,767 `REP` items                                                                                                                |
| `terminal`                  |       22 |     1 | a deferred `TermRequest` reported against a new terminal on the same buffer                                                                                   |
| `help`, `highlight`         |        9 |     2 | an overridden help helper; a negative cterm colour under 'winblend'                                                                                           |

### What was converted

Each family below became an error, a clamp or a refusal with a
regression test; the commits carry the detail.

| family                          | shape now                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `:terminal` resize              | the saved cursor is clamped and a margin left past the edge is dropped                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| values Lua or a peer hands back | a dict key with a NUL is cut there; an RPC handle is echoed as its 32 bits; client info for a closed channel is dropped; an overridden `vim._core` helper is refused like a Lua error; `:version` leaves out a tampered line                                                                                                                                                                                                                                                                                                                                              |
| parser and walks                | an unmatched `]`/`}` after an operand is a parse error; a `filter()`/`map()`/`foreach()`/`indexof()`/`uniq()`/`matchfuzzy()` callback that edits its container is walked through; a userdatum whose `__gc` plain Lua called raises "… has been collected"                                                                                                                                                                                                                                                                                                                 |
| corrupt files                   | undo files with impossible line numbers, counters or links are E825; spell trees with a bad shared index are E759; a swap page size above the maximum is refused; a claimed length never sizes an allocation before the bytes are there (`memory/claimed.rs`)                                                                                                                                                                                                                                                                                                             |
| numbers at the `int` extremes   | the number is clamped where it enters (a builtin's argument, a count, an option value, a fold level, a 'diffexpr' hunk), or the sum is taken wide (`strpart()`, undo time steps), so the arithmetic downstream stays in range; wrapping is kept only where the value is modular anyway (a cterm colour read back as 16 bits, a ±1 storage bias)                                                                                                                                                                                                                           |
| white-space widths and repeats  | 'shiftwidth' and 'softtabstop' are at most 9999 like 'tabstop', a 'cinoptions' amount past 9999 and a negative 'wrapmargin' are refused at `:set` (every entry point, modelines included); `:left`/`:center`/`:right` refuse a width past 9999 (E475, as `:retab` does) and count a larger 'textwidth' as 9999; an NFA `\{n,m}` that would write out more than 65,536 items is handed to the backtracker under 'regexpengine' 0 and is E339 otherwise. Each used to build an indent (or a program) of hundreds of megabytes; upstream hangs or runs out of memory on them |
| input-reached `debug_assert!`s  | each was either an input check in the wrong place (`nvim_del_augroup_by_id(2^32)`, a CSI parameter holding a skipped byte, a 'colorcolumn' past `int`) or was wrong (`$NVIM_APPNAME`'s length guarded a buffer the port does not have; a non-positive decoration-provider namespace is accepted by upstream's release build) and went                                                                                                                                                                                                                                     |

`test/functional/core/number_extremes_spec.lua` holds the `int`-extreme
reproducers and `test/functional/core/corrupt_files_spec.lua` the
claimed-length ones.

### Accepted, and why

- **≥ 2 GiB of data** (`memline` line lengths, `Blob::len_int`, the RPC
  packer's 4 GiB string/container headers): the `expect` names the limit
  ("a line shorter than INT_MAX", "a blob shorter than 2 GiB", "string
  too long for msgpack"). Reaching one takes a 2 GiB line or blob in
  memory; upstream's `int` lengths are undefined there too. Converting
  them means a `Result` through every memline and blob writer.
- **Work a count asks for, which CTRL-C stops**: `2147483647ix<Esc>`,
  `2147483647.`, `2147483647p` of a line, `>>` repeated by a script. Each
  is what was asked for, done one step at a time, and each checks for
  CTRL-C between steps. (The white-space widths and repeat counts that
  built megabytes in one step are bounded; see the table above.)
- **A count whose text fits upstream's own limit**: `2147483647p` of one
  character builds a line of 2 GiB, which E1240 allows (it refuses past
  `INT_MAX` bytes, as upstream does). It is the ≥ 2 GiB case above, and
  where memory runs out first it is the allocation-failure exit.
- **Off the main thread**, see [the known gaps](#known-gaps).
- **`debug` and `ffi` Lua** are out of the threat model: with them a
  script can break any invariant, in upstream as here.
- Reclassified: the yank's `line.len() - textcol` is an invariant
  (`charwise_block_def` clamps the start onto the line); the expression
  parser's `check_children` assert is an invariant once the `1 2]3`
  family stopped mis-wiring nodes; `get_indent`'s column count overflows
  only past a 2 GiB indent: a 2 GiB line, the first case above.

### Using the dimensions

- `panic_sites` is a listing, not a ratchet: a new invariant gets a new
  `expect`, and that is fine. Read it when a module changes hands, or to
  find where external bytes meet explicit panics.
- `bare_panics` is ratcheted at 0. A panic must say what broke: write the
  invariant (`expect("a row or column has a child")`), not the call
  ("checked", "fits", a pasted C condition).
- An input-reachable site is a bug whatever the dimension says. Find
  them the way the census did: probe the extremes, mutate the files,
  replay the reproducer on a debug and a release build.
