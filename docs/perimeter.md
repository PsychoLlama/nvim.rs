# The unsafe perimeter

Unchecked code is one of two things: transpiled editor logic that safe Rust
can replace, or the seam where this program meets a C library, the operating
system, or raw memory — code that will still be `unsafe` when the migration is
finished, because the thing on the other side cannot be rewritten from here.
For most of the migration the tree carried tens of thousands of statements of
the first kind; reading them as one number with the second was misleading.

The **perimeter** is that second part, named explicitly. What was left over
was the migration's debt, counted by the ratchet as
`unsafe_stmts_outside_perimeter` and driven to zero; the ratchet now refuses
unsafe code anywhere off this list (see "How the ratchet enforces it").
[docs/unsafe-audit.md](unsafe-audit.md) is the same list read row by row for a
reader who is not a specialist: what each row holds, the contract that keeps
it sound, and what would retire it.

Today: **1,615** unchecked statements inside the perimeter (64 files),
**0** outside it (0 files, of 1,400 measured). It was 138,877 when this
file was written, at the end of phase 23's slice 15 — in _lines_, which is what
this metric counted until phase 31 changed the unit to statements.

## What qualifies

A module belongs on the perimeter when removing its unchecked code would mean
rewriting something this tree does not own:

- **A foreign ABI.** It calls into, or is called by, a C library — LuaJIT,
  libuv, libc, libvterm, termkey, unibilium, tree-sitter, libxdiff — or it
  exports C symbols someone else resolves.
- **The operating system.** Syscalls, the PTY, process control, the
  environment.
- **A raw-memory primitive everything above it is built on.** The allocator,
  the checked wrapper over c2rust's mutable statics, the swap file's page
  store, the handles that make dereferencing the window/buffer graph and the
  value graph (Lists, Dictionaries, Blobs, partials) safe.
  These exist so that a raw operation happens in one place and nowhere else;
  moving their unsafe upward would be a regression.

Nothing else qualifies, however raw it looks today. A transpiled algorithm
over pointers, a codec, an in-memory data structure and an on-disk parse can
all become safe Rust, and listing them here would retire them by fiat. So
`memline/` (including the on-disk block structs), `marktree/`, `shada/`,
`spellfile/`, `undo/`, `msgpack_rpc/`, `mpack/`'s own codec, `grid/` and
`log.rs` are **outside** the perimeter and are expected to leave the debt
column the ordinary way. `memline/`, `undo/`, `shada/`, `spellfile/`,
`msgpack_rpc/` and the whole of `mpack/` (the codec and `vim.mpack`'s binding),
with `cjson/` (`vim.json`), have: each forbids unsafe code (the `:mkspell` word tree is an index slab that
charges the arena's old byte counts, so compression runs where it did). `memline/` reads its pages through bounds-checked block views
over the `Box<[u8]>` pages `memfile/` owns, and holds a block by a
generation-checked id; `memfile/` itself keeps its swap file as a
`std::fs::File` and forbids unsafe code too, so the page store, which had a
row here at 206 unchecked statements, has none.

**Membership is by module, not by line or by file.** An entry is a directory
prefix or one exact path, because a module is the unit that owns a boundary. A
file inside a perimeter module that turns out to be ordinary editor logic
should move out of the module rather than be carved out of the list.

## The list

| module                                      | files | unchecked | why, and what would retire it                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------------- | ----: | --------: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lua/ffi.rs`, `lua/executor/{luv,state}.rs` |     3 |       105 | LuaJIT's C API declared (`ffi.rs`), luv's C hooks that run a libuv callback on the right `lua_State` (`luv.rs`), and making and closing states and opening the C libraries (LuaJIT's own, lpeg, luv) onto them (`state.rs`). Retires with the embedded interpreter, not before. The rest of `lua/` — the executor, the standard library, the converter, the generated `api_wrappers/` and the root files — reads the stack through `lua/stack/` and forbids unsafe code.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `lua/stack/`                                |     5 |       110 | The typed `Stack`: the one module that pushes, pops and reads the `lua_State` stack (references, userdata and the debug interface included), and the one trampoline every safe C-function body enters through. It also reads and writes the head of LuaJIT's `lua_State` directly (`L->top`/`L->base`, for `top` and `pop`): LuaJIT is pinned by `nix/deps.json`, and `check_state_layout` aborts on every new state whose layout drifted. Every region is one LuaJIT call (a `lua_*`/`luaL_*` function), a read of what one answered (`lua_tolstring`'s bytes, `lua_getinfo`'s strings), the head read of `top`/`pop`, or a userdata block (`lua_newuserdata`'s write, `luaL_testudata`'s loan); `dispatch.rs`'s raise is `luaL_where`/`luaL_argerror`/`luaL_check*`/`lua_error` once the body has returned. Dropping or moving a slot by index aborts at or below a view's base in every build; an operation consuming values off the top (`set_field`, `raw_set`, `next`, `concat`, ...) is checked in debug builds only -- its contract is that it consumes values this view pushed -- because the release check cost `luapush`/`luapop`/`objpop` 0.8-1.7 % Ir over the floor.                                                                                                                                                                                        |
| `lua/treesitter/`                           |    10 |       203 | the tree-sitter C library — parsers, trees, queries and cursors are opaque C objects with C lifetimes, held by userdata whose `__gc` deletes them. What is unsafe here is the calls into the library and the reads of what it answers; the Lua stack is reached through `lua/stack/` only.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `event/`                                    |    11 |       558 | libuv: the loop, streams, timers, signals and processes are C objects registered by address. `event/channel.rs` is the one place a channel's transport is handed to libuv, by channel id; callbacks carry ids, not addresses. A transport lives in a pinned box (`Pin<Box<ChannelStream>>`, private to `Channel`, and a `Stream` is neither `Unpin` nor `Copy`), so no safe code can move or swap a registered handle: outside `event/` a transport is read through `&` and changed through the methods `event/channel.rs` defines on its pin, which is the one place it is projected. The event queue is not here: `event_queue.rs` keeps every queue in a table by id and forbids unsafe code; what is here is running an event, whose C-shaped form (`RawEvent`: a handler and one word, the watcher's own address) only `event/` builds. Each region is a libuv call, a handle rebuilt from the address libuv hands back, or a pin projection. Still editor-shaped, and listed for the audit: the listening socket's address held in a `[c_char; 256]` and split in place (`socket/mod.rs`), the read stream's block positions (`rstream.rs`), and the `char **` argv and environment a child is spawned with (`libuv_proc.rs`, `proc.rs`). Running a command to completion is the safe `libuv_proc::run_to_exit`; what becomes of its output is `shell/system.rs`'s. |
| `os/`                                       |    14 |       237 | The operating system, and in each file only the foreign call, its declaration and the buffer it fills: libuv's `uv_fs_*` and libc's `xattr`/`FILE`/`fcntl`/`write` calls (`fs/`), `uv_os_setenv`/`unsetenv`/`homedir`, `uv_random`, `uname` and the `environ` walk (`env/`), `setlocale`, `nl_langinfo` and gettext's binds (`lang.rs`), the `getpw*` family (`users.rs`), `tzset`/`localtime_r`/`strftime`/`strptime` (`time.rs`), stdin's `RStream` and the loop polls (`input.rs`), the PTY (`pty_proc_unix/`: `forkpty`, `fcntl`, `ioctl`, `ptsname`, `killpg`, `poll`, and in the forked child `setsid`/`signal`/`execvp`, each on a `&PtyProc` the process layer lends), signals, `dlopen`, `iconv`, and `cshim.rs`'s declarations. Reading one variable, the working directory, `realpath`, the executable path and descriptors (`File`/`OwnedFd`/`BorrowedFd`) are `std`. What the editor makes of them -- `$VAR`/`~` expansion, `$VIM`/`$VIMRUNTIME`, the XDG directories, building the shell's command line, running it and collecting its output, buffered file I/O -- is outside, in `env/`, `shell/` and `fileio/buffered.rs`.                                                                                                                                                                                                                               |
| `vterm/abi/`                                |     3 |        90 | libvterm's C interface, and nothing else: the 30 exports `docs/abi-ledger.md` lists (fronts over `VTerm` methods that recover the terminal from the state or screen pointer they are handed), the callback tables a C consumer installs (`foreign.rs`: each a table and a data pointer the installing export's caller keeps live, implementing the emulator's own report traits by calling through the table), the character-set tables the unit specs call (`charset.rs`), and the `VTermColor` union's reads. The emulator itself -- parser, state machine, screen, pen, encodings, keyboard and mouse -- is safe Rust over owned buffers, and the structs C reads keep libvterm's layout because the fixture and the battery read them from C. Retires with the unit fixture and `vtsweep`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `tui/{tui,input,output,events}.rs`          |     4 |        69 | The terminal's libuv handles, and only those four files: the output tty/pipe handle on a private write loop, the startup timer, the SIGWINCH watcher and the suspend `kill` (`tui.rs`); stdin's read stream, the escape and background-query timers and the line discipline's erase character via `tcgetattr` (`input.rs`); the flush's one `uv_write` (`output.rs`); and the UI events that reach libuv -- tty mode, the bell's sleep, `ui_send`'s direct write, `uv_chdir` (`events.rs`). The TUI's state is one cell (`tui.rs`) for the life of the process: every callback looks it up rather than carrying its address, the lifecycle steps that turn the event loop hold nothing of it while they wait, and the files take `&mut TUIData`. termkey's parser, the terminfo description, negotiation, quirks, painting and attributes are safe Rust beside them.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `xdiff/`                                    |     1 |         1 | libxdiff, vendored: which bytes are whitespace is `isspace(3)`'s answer in the user's locale, read through glibc's table.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `harness.rs`                                |     1 |         8 | The C symbols the in-repo test harnesses resolve by name — the editor rows of `docs/abi-ledger.md` (`starting`, `test_disable_char_avail`, `early_init`/`event_init`, the UTF-8 readers and `utf8len_tab`, `schar_get`, …). Each function is a front over its module's slice form that reads the C side's pointer here; each data symbol is defined here and re-exported by its module, so the module forbids unsafe code. Retires row by row with the specs and fixtures that resolve the names.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `allocator.rs`                              |     1 |        20 | The global allocator: malloc/realloc/free.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `memory/`                                   |     4 |        99 | `xmalloc` and its family over the global allocator, with the out-of-memory retry and the allocation log the unit suite reads (`mod.rs`); the C-shaped string helpers the perimeter's callers and the unit specs still call (`xstrdup`, `xstrlcpy`, `xmemdupz`, …) and the C `memchr`/`strchr`/`strlen` scans over a terminated slice the matchers use; the block pool the event loop reads into and the packers write into, chained through each spent block's first bytes (`blocks.rs`); and the two owned C strings, `XString` and `ThinCString`, which adopt a `malloc` block from C and hand one back (`xstring.rs`, `thin_cstring.rs`) — the floor under every owned type in the tree.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `global_cell.rs`                            |     1 |        13 | The checked wrapper over c2rust's mutable statics; the raw static is touched here so it is nowhere else.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `winlayer.rs`, `winlayer/`                  |     5 |        51 | The window/buffer/position handles, the screen grid handle (`GridRef` in `winlayer/grid.rs`, which the compositor reads while it is drawn on), the terminal handle (`Term` in `winlayer/terminal.rs`, which the emulator's callbacks, the channel and the buffer all reach a `:terminal` through), and the window's syntax block (`w_s`): constructing one is the unsafe step, dereferencing it is not.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `eval/typval/handle.rs`                     |     1 |        51 | The value handles (`ListRef`, `DictRef`, `BlobRef`, `PartialRef`) and a dictionary's item store: constructing a handle is the unsafe step, dereferencing it is not. A value is reached from many holders at once and the evaluator re-enters at every call, so `edit()` lends `&mut` from a shared handle on the `Live` promise (checked in debug builds by `typval/edit.rs`); the reference count, the `xcalloc`/`Box` that holds each object, the item slots that name a boxed or an embedded `DictItem`, and `TypVal::bit_copy` (an argument frame naming its caller's values) are the rest. Retires only with a different value model.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

Paths are relative to `crates/nvim/src/`. Counts are `unsafe_stmts` from
`metrics/ratchet.json` and drift as work lands; the list, not the table, is
the contract.

## How the ratchet enforces it

`PERIMETER` in `scripts/ratchet.py` is the list, one entry per row above, each
carrying its reason. Five things follow from it:

- **Unsafe code is allowed here and nowhere else.** `check_unsafe_placement`
  fails `just ratchet --check` — so `just minimal-ci` and the pre-commit hook —
  and `just refresh` alike on any `crates/nvim/src` file whose attributes are in
  the wrong place: an `allow(unsafe_code)` off the list, generated files
  included; a file with neither attribute that is not a module file over an
  allowing descendant (the "neither" list below, computed rather than listed);
  and an allow file without `deny(unsafe_op_in_unsafe_fn)` or without its row
  named above the attribute. The message names each file, the rule it breaks
  and its two exits — finish the file, or give its module a row here with a
  reason; `--dimension unsafe_placement` prints the same list. While the files
  outside were finishing, this was a shrink-only count (`unsafe_placement`)
  beside `unsafe_stmts_outside_perimeter` (the unchecked statements off the
  list) and `files_allowing_unsafe_code`; when the first read zero the check
  went hard, and the three counts retired with the four the check implies
  (`files_without_deny_unsafe_op`, `stored_addr_handles`, `lua_raw_stack`,
  `raw_variadic_calls`, beside the long-zero `ml_get_raw` and the per-file
  `static_mut`/`variadic` counts of `os/cshim.rs`'s shim).
- **The perimeter cannot silently grow.** It needs no check of its own for
  that: every file's `unsafe_stmts` is ratcheted individually, a brand-new
  file included, so unchecked code appearing inside the perimeter is a
  violation — and moving an unsafe file into a perimeter module shows up as a
  new path at full size. A region is charged by what it _does_: every
  `;`-terminated statement, every block's tail expression and every `match`
  arm, at every depth, minimum one. Splitting `unsafe { a; b; c }` into three
  regions is not progress; deleting an operation is.
- **The list is self-pruning.** `check_perimeter` fails the run when an entry
  has no file with unchecked code behind it, so a module that finishes,
  moves or disappears has to leave the list in the same commit. A module
  reaching zero is the outcome the list is for, and it has to say so.
- **A file on the list says so in its own source.** See below.
- **The "Today" line above is written, not typed.** `sync_perimeter_doc`
  rewrites its five numbers from the same measurement, and the `--check` form
  fails when they are stale. Hand-maintained, all five drifted — the line
  claimed 48,358 outside against a real 47,002 — and a number in prose that
  nothing checks is a number nobody can cite.

To add an entry: put the path and its reason in `PERIMETER`, add the row here,
run `just refresh`, and justify it in the commit message. The number this
lowers is the number the migration is judged by, so the bar is the criteria
above and nothing softer.

## The raw vocabularies, and where each bottoms out

Three vocabularies make up almost everything the perimeter does with raw
memory: C strings, manual allocation, and pointer arithmetic. This is where
each one's floor sits and why it is not lower. The totals are the ratchet's
needles (`metrics/ratchet.json`), split per row by the same needles. Off the
perimeter a raw pointer _type_ is safe to write — it is reading through one
that needs `unsafe` — so a few appear there; they are listed too.

**C strings.**

- `*const c_char`/`*mut c_char` (`raw_cstr`, 240). On the perimeter, 180:
  the declarations of C functions that take or answer a C string
  (`os/cshim.rs` 41, `event/libuv.rs` 35, `lua/ffi.rs` 19), the C-shaped string
  helpers and the owned strings' C views (`memory/` 29), tree-sitter's names
  and query strings (`lua/treesitter/` 18), the socket address, a child's
  argv/environment and a job's name (`event/`, 19 more), `libcall()`'s
  arguments and the PTY's name (`os/`, 7 more), and the C entry points the
  harnesses and libvterm's clients call (`harness.rs` 5, `vterm/abi/` 5,
  `tui/input.rs` 2). Off it, 60: fields of the `#[repr(C)]` structs a C library
  reads (`types/uv.rs` 13, `types/vterm.rs` 9, `types/event.rs` 7, …) and a
  dozen safe functions that hand an owned string's address (`as_ptr()`) to a
  perimeter callee without reading through it. The floor is the C side's
  signatures: a string crosses into C as a pointer.
- `CStr::from_ptr` (54, all on the perimeter): where a C callee's answer
  becomes a `&CStr` — `os/` 26 (`strerror`/`uv_strerror`, directory entries,
  the `getpw*` records, `environ`, locale names, `dlerror`, `ptsname`, the Miri
  stand-ins), `event/` 14 (the socket address 10; a child's argv words,
  program and executable path, and a libuv error 4), `memory/` 7 (the owned strings adopting a block), `lua/stack/` 6
  (`lua_tolstring`, `lua_getinfo`), `lua/treesitter/` 1. Each falls only
  with its callee.
- `[c_char; N]` (`cchar_scratch`, 6). On the perimeter, 3: the listening
  socket's address, split and appended in place (`event/socket/mod.rs`, the
  one that is editor-shaped — it needs the watcher built whole instead of
  through `Box::new_uninit`), the 400-byte buffer `iconv(3)` writes its
  probe into (`os/iconv.rs`), and the device-attribute answer the TUI specs
  overwrite through FFI (`vterm/abi/mod.rs`). Off it, 3 that are byte tables
  spelled in C's type: termkey's keypad table and report payload
  (`tui/termkey/`) and `windowsversion()`'s always-empty string
  (`os/state.rs`). None is filled through a pointer by editor code.
- `slice::from_raw_parts` (30, all on the perimeter): a slice over memory C
  describes by pointer and length — libuv's read buffers (`event/`,
  `os/input.rs`, `tui/input.rs`), `lua_tolstring`'s bytes (`lua/stack/` 5),
  tree-sitter's arrays (`lua/treesitter/` 5), the allocator family's blocks
  (`memory/` 8), the harness's and libvterm's C arguments (`harness.rs` 3,
  `vterm/abi/` 3).

**Allocation.**

- The `xmalloc` family (`manual_alloc`, 26, all on the perimeter): the family
  itself and the owned strings (`memory/` 17); Lists and Dictionaries, which
  stay `xcalloc`'d because the unit suite's allocation log pins that family
  (`eval/typval/handle.rs` 4); a spawned child's `char **` argv/environment
  and the socket watcher (`event/` 3); buffers tree-sitter frees
  (`lua/treesitter/` 2).
- `Box::from_raw`/`Box::into_raw` (13 each): a box parked behind an address —
  libuv handle data (`event/` 7 and 5), the window block (`winlayer/graph.rs`,
  `ParkedWindow`), the terminal (`winlayer/terminal.rs`), Blobs and partials
  (`eval/typval/handle.rs` 3 and 1), Lua userdata (`lua/stack/userdata.rs`),
  the `VTerm` a C client is handed (`vterm/abi/mod.rs`), the allocator. Three
  `into_raw`s sit off the perimeter (`eval/typval/dictitem.rs`,
  `memory/handoff.rs`, `types/api_string.rs`): `into_raw` is safe, and each
  hands its box to a perimeter file that takes it back.
- `GlobalCell`s holding a raw pointer (`cell_raw_ptr`, 4), all on the
  perimeter: `curwin`/`curbuf`/`curtab` (`winlayer/graph.rs`; `curwin` is a C
  symbol the unit suite reads) and the block pool's reuse-list head
  (`memory/blocks.rs`). The raw escape hatches `.ptr()`/`.as_raw()`
  (`cell_ptr`, 22) are all on the perimeter too: the globals libuv holds by
  address (`main_loop`, `read_stream`, the signal watchers, the refresh timer,
  `main_thread`, the saved termios) and `global_cell.rs`'s own tests.

**Pointer arithmetic** (`ptr_arith`, 100: `.add`/`.offset`/`.sub`/
`.offset_from` and pointer `wrapping_*`). On the perimeter, 46: the C-shaped
string helpers (`memory/` 13), the Miri stand-ins for libc's string
functions and the `isspace` table read (`os/cshim.rs` 10; `os/env/mod.rs`'s
`environ` walk 2; `os/signal.rs` 1), the socket address 6, the read stream's
block positions 4, a child's argv 3 and a channel 1 (`event/`), `lua/stack/`
2, `lua/treesitter/` 2, `harness.rs` 1 and `xdiff/ffi.rs` 1 (glibc's
character-class table). The 54 off the perimeter are not pointer arithmetic:
the needle matches any method named `add`/`offset`/`sub`, and those are the
evaluator's cursor `offset()` and the keyword and visited-file tables' `add`.
The needle keeps counting them, because one a rewrite could make grow would be
worse than one that over-counts.

The three editor-shaped items above — the socket's address buffer, the read
stream's block positions and a child's `char **` argv/environment, all in
`event/` — are the only places a vocabulary sits higher than its foreign
callee requires.

## The default, and what a file has to say

`unsafe_code` is allow-by-default in rustc, so for most of the migration
nothing in the tree said where unsafe was _permitted_ to live: the answer was
the absence of `#![forbid(unsafe_code)]`, which is not a claim a file makes but
one it fails to make, and a new file inherited permission by saying nothing.
Phase 28 flipped that. `crates/nvim/Cargo.toml` now has

    [lints.rust]
    unsafe_code = "deny"

which governs the library, the `nvim` binary and the test and bench roots
alike. A file that needs unsafe — a block, an `unsafe fn`, an `unsafe trait`
or `impl`, an `unsafe extern` block, an `#[unsafe(no_mangle)]` export — carries
an inner

    #![allow(unsafe_code)]

with its other inner attributes, and `scripts/ratchet.py`'s placement check
holds where that line may appear. Three rules about it:

- **A file inside a perimeter module names its row**, in a one-line comment
  directly above the attribute — "Unsafe perimeter: the `os/` row in
  docs/perimeter.md." That allow is permanent — the module is on the list
  precisely because its unsafe does not retire — and the comment is what makes
  the difference visible while reading the file rather than the list.
- **No other file carries it.** For most of the migration a file off the list
  carried it bare, because there the count was the reason: those files were
  the debt, and each that finished dropped the allow for a `forbid`. The last
  of them did, and the placement check now refuses the line off the list. A
  file exporting a C symbol somebody else resolves is no exception any more:
  an export that is only a front over a module's own function (or a data
  symbol the module can re-export) lives in `harness.rs`, a row of its own,
  so the module it reaches forbids.
- **A generated file takes it from the generator.** A hand-written attribute
  under `crates/nvim/src/api/private/dispatch*/` or `src/lua/api_wrappers/` is
  gone at the next `just apigen`, so `tools/apigen` emits it — and only for a
  chunk that actually holds unsafe. No generated file is on the list, so
  every one takes `forbid`, and a chunk that came out holding unsafe fails
  the placement check like any other file.

`forbid` still overrides the deny and still cannot be lifted by a module
underneath it, so it remains the stronger claim and the one a finished module
takes. What retired with the flip is the metric that used to stand in for
this: `files_without_forbid_unsafe` counted files that had not made a claim,
where `files_allowing_unsafe_code` counted the ones that had made the opposite
claim, deliberately, in a line a reviewer can see — until the placement
check made that count the perimeter's own file count, and it retired too.

**There are no free `forbid`s left.** Exactly 10 files carry neither
attribute, and every one of them is a module file with a descendant on the
perimeter that allows unsafe: `allow` under a `forbid` is `E0453`, a hard
error, so writing the attribute on any of them fails to compile. They stay
bare for as long as the row beneath them does: `lib.rs`, the root of
everything; `lua/mod.rs` and `lua/executor/mod.rs` (over `lua/ffi.rs`,
`lua/stack/` and `lua/executor/{luv,state}.rs`); `event/mod.rs`; `os/mod.rs`;
`tui/mod.rs` (over its four handle files); `vterm/mod.rs` (over
`vterm/abi/`); `xdiff/mod.rs` (over `xdiff/ffi.rs`); and `eval/mod.rs` and
`eval/typval/mod.rs` (over `eval/typval/handle.rs`). Every other module file
forbids — `charset/`, `getchar/`, `mbyte/`, `startup/`, `strings/`,
`tui/termkey/`, `terminal/`, `memfile/` and `marktree/` among the last, each in
the commit that finished the last of its descendants. This is the same rule as
"take a `mod.rs` last", stated for the attribute rather than for the lint.

## The types the perimeter cannot hold

The perimeter is measured in unchecked lines, and it prunes itself on them:
an entry with no `unsafe` behind it is stale by definition. That leaves one
population it structurally cannot describe. c2rust hoisted every C struct out
of the module that used it and into a per-library file under `types/`, so the
layout of a `uv_loop_t` is written in `types/uv.rs` while the `uv_run` that
needs it lives in `event/`, and a `TermKey`'s in `types/termkey.rs` while
termkey's parser runs in `tui/`. Those files hold **zero** unchecked lines.
They cannot join the list — `check_perimeter` would reject them — and yet
their `#[repr(C)]` is libuv's and libtermkey's layout, not this tree's.

`FOREIGN_ABI_TYPES` in `scripts/ratchet.py` names them, on exactly the "a
foreign ABI" bullet above and nothing softer: `types/uv.rs`, `types/vterm.rs`,
`types/vterm_internal.rs`, `types/termkey.rs`, `types/lua.rs`,
`types/libc.rs`, `types/libuv_proc.rs`, `types/pty_proc_unix.rs`. It splits
one metric in two:

- **`repr_c_ffi_types`** — `#[repr(C)]` in those files. Not debt, but
  ratcheted like everything else, so a new foreign type is visible.
- **`repr_c_editor_state`** — `#[repr(C)]` everywhere else off the perimeter:
  this tree's own aggregates, which are transpiler residue except where a
  codec, a flexible array member or a state-machine base pins the layout.
  This is the number to drive down.

The two partition what `repr_c_outside_perimeter` used to total. A type this
tree defines and only this tree reads is not on the list however C-shaped it
looks: `types/keysets.rs` is written through byte offsets by our own generated
keydict codec, `types/mpack*.rs` and `types/rpc.rs` describe a codec that was
vendored and ported rather than linked — the same reason `mpack/`'s codec is
outside the perimeter — and `types/terminal_defs.rs` is `#[repr(C)]` only
because the FFI-safety lint follows a pointer out of `Buffer`. All of those
are still expected to leave the residue column the ordinary way.
