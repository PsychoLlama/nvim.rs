# The unsafe audit

This is the whole of the unsafe code in the editor, written so that someone
who does not know the code base — or Rust's `unsafe` in any depth — can check
it: what is unsafe, where it is, why it cannot be otherwise, and what would
have to change for it to go.

## The shape

Rust checks memory safety at compile time everywhere except inside `unsafe`
code, where the programmer promises what the compiler cannot verify. In this
tree that code is confined to **64 files**, holding **1,615 unchecked
statements** — every one of them on the _perimeter_ ([docs/perimeter.md](perimeter.md)),
the list of modules where the program meets something it does not own. The
other 1,324 source files of the editor (`crates/nvim/src`, 1,388 files) contain
no unsafe code, and 1,314 of them say so with `#![forbid(unsafe_code)]`, which
the compiler enforces; the remaining ten are module roots that cannot carry
the attribute because a file beneath them is on the perimeter. The unsafe code
falls into four classes:

- **A foreign ABI — 34 files, 1,046 statements.** Calls into C libraries the
  editor links: LuaJIT and its libraries, libuv (the event loop, processes,
  sockets, the terminal's I/O), tree-sitter and libxdiff. The unsafe is the
  call itself, and the reads of what the library answers.
- **The operating system — 14 files, 237 statements.** System calls the
  standard library does not offer, or offers with different error text:
  libuv's file-system calls, the environment, locales, the user database,
  clocks, the PTY, signals, `dlopen` and `iconv`.
- **Raw memory — 12 files, 234 statements.** The few primitives every safe
  type above them is built on: the allocator, the `xmalloc` family and the
  owned C strings, the checked cell that replaces C's global variables, and
  the handles through which the editor reaches windows, buffers and
  Vimscript values.
- **Exported test ABI — 4 files, 98 statements.** C functions and data the
  in-repo test harnesses resolve by name: the terminal emulator's C interface
  (its unit tests are upstream's, written in C and Lua against libvterm's
  API) and a handful of editor entry points the unit and old-style test
  harnesses call.

## How it is held

- `crates/nvim/Cargo.toml` denies `unsafe_code`, so a file can contain unsafe
  code only by writing `#![allow(unsafe_code)]` itself.
- `scripts/ratchet.py`'s placement check fails `just ratchet --check` — run by
  `just minimal-ci` and by the pre-commit hook — when that line appears in a
  file not on the perimeter list, when an allowing file lacks
  `#![deny(unsafe_op_in_unsafe_fn)]` (which makes every unsafe operation
  inside an `unsafe fn` need its own visible `unsafe { }` block), or when a
  perimeter file does not name its row in a comment above the attribute.
- Every file's count of unchecked statements is ratcheted: it may shrink and
  never grow without an explicit, justified override. A statement is what the
  code _does_ — splitting a block into smaller ones does not lower it.
- Every `unsafe fn` (228) carries a `# Safety` section saying what its caller
  must guarantee; the ratchet fails on one without. An `unsafe fn` whose
  signature shows no raw pointer needs a row in
  [docs/unsafe-fn-allowlist.md](unsafe-fn-allowlist.md) (four rows today).
- Debug builds add checks the release build compiles out: the main-thread
  and borrow checks of `GlobalCell`, the overlapping-edit check on Vimscript
  values, the Lua stack's consuming-operation check. The functional, old-style
  and unit suites run against debug builds; whole-suite AddressSanitizer runs
  and Miri (over the crate's own tests) back them.

To check the numbers here: `python3 scripts/ratchet.py --dimension
unsafe_placement` lists any misplaced attribute (none);
`metrics/ratchet.json` holds each file's `unsafe_stmts`; `rg -l
'allow\(unsafe_code\)' crates/nvim/src` lists the 64 files.

## The rows

| row                                         | class    | files | statements | `unsafe fn`s | what it holds                                    |
| ------------------------------------------- | -------- | ----: | ---------: | -----------: | ------------------------------------------------ |
| `lua/ffi.rs`, `lua/executor/{luv,state}.rs` | foreign  |     3 |        105 |            8 | LuaJIT's declarations, luv's hooks, state making |
| `lua/stack/`                                | foreign  |     5 |        110 |            6 | the typed Lua stack                              |
| `lua/treesitter/`                           | foreign  |    10 |        203 |            9 | the tree-sitter library                          |
| `event/`                                    | foreign  |    11 |        558 |           90 | libuv: loop, streams, timers, signals, children  |
| `tui/{tui,input,output,events}.rs`          | foreign  |     4 |         69 |            6 | the terminal's libuv handles                     |
| `xdiff/`                                    | foreign  |     1 |          1 |            0 | `isspace(3)`'s table                             |
| `os/`                                       | OS       |    14 |        237 |           15 | system calls                                     |
| `allocator.rs`                              | memory   |     1 |         20 |            4 | the global allocator                             |
| `memory/`                                   | memory   |     4 |         99 |           36 | `xmalloc`, the block pool, owned C strings       |
| `global_cell.rs`                            | memory   |     1 |         13 |            0 | the cell under every global                      |
| `winlayer.rs`, `winlayer/`                  | memory   |     5 |         51 |           10 | window, buffer, grid and terminal handles        |
| `eval/typval/handle.rs`                     | memory   |     1 |         51 |            8 | Vimscript value handles                          |
| `vterm/abi/`                                | test ABI |     3 |         90 |           31 | libvterm's C interface                           |
| `harness.rs`                                | test ABI |     1 |          8 |            5 | editor symbols the harnesses resolve             |

Paths are relative to `crates/nvim/src/`. Counts drift as work lands;
`metrics/ratchet.json` is the source.

### `lua/ffi.rs`, `lua/executor/{luv,state}.rs` — LuaJIT, luv, lpeg

- **What:** `ffi.rs` (68) declares LuaJIT's, luv's and lpeg's C functions —
  a declaration is an unchecked promise about a C signature, and costs one
  statement each — and the inline shims over LuaJIT's header macros.
  `luv.rs` (19) holds the four hooks luv calls to run a libuv callback on the
  right Lua state; `state.rs` (18) creates and closes Lua states and opens
  the C libraries onto them.
- **Contract:** a hook runs on the frame luv calls it on, with the state luv
  hands it; a state is closed once, by the code that made it.
- **Retires:** with the embedded Lua interpreter. Not planned.

### `lua/stack/` — the Lua stack

- **What:** the one module that pushes, pops and reads LuaJIT's value stack
  (`mod.rs` 62, `dispatch.rs` 24, `userdata.rs` 13, `debug.rs` 9,
  `table.rs` 2). Every region is one LuaJIT call, a read of what one
  answered, a userdata block, or the read of the state's `top`/`base`
  fields. The rest of `lua/` reaches Lua only through this module and
  forbids unsafe code.
- **Contract:** a `Stack` is a view of a C function's frame rooted at a
  base slot. Dropping or moving a slot at or below the base aborts in every
  build. An operation that consumes values off the top (`set_field`,
  `raw_set`, `next`, `concat`, …) consumes values this view pushed — **checked
  in debug builds only** (see "Soundness by contract"). The `lua_State`
  layout is read directly: LuaJIT is pinned by `nix/deps.json`, and
  `check_state_layout` aborts on every new state whose layout drifted. A Lua
  error (a C `longjmp`) is raised only after the Rust body has returned, so
  no Rust frame is unwound by it.
- **Retires:** with LuaJIT. Not planned.

### `lua/treesitter/` — the tree-sitter library

- **What:** parsers, trees, queries and cursors are C objects with C
  lifetimes, held by Lua userdata whose finaliser deletes them (`mod.rs` 76,
  `node.rs` 37, `lang.rs` 24, `parser.rs` 20, `query.rs` 14, `cursor.rs` 12,
  `tree.rs` 8, `ranges.rs` 7, `logger.rs` 4, `init.rs` 1). The unsafe is the
  calls and the reads of the arrays and strings they answer.
- **Contract:** a userdata's object lives until its finaliser runs, and a
  query's strings and capture arrays are read only while the query is alive;
  the Lua stack is reached through `lua/stack/` only.
- **Known, upstream too:** `_ts_inspect_language`'s subtype off-by-one and an
  unguarded re-entry of a parse from its own logger are upstream's
  behaviour, kept.
- **Retires:** if the C library were replaced. Not planned.

### `event/` — libuv

- **What:** libuv's loop, streams, timers, signal watchers and child
  processes are C objects libuv holds _by address_ (`socket/mod.rs` 108,
  `libuv.rs` 100 — mostly the declarations — `proc.rs` 70, `loop.rs` 58,
  `channel.rs` 52, `libuv_proc.rs` 48, `rstream.rs` 41, `stream.rs` 34,
  `signal.rs` 17, `time.rs` 16, `wstream.rs` 14). Each region is a libuv
  call, a handle rebuilt from the address libuv hands back to a callback, or
  a pin projection.
- **Contract:** a registered handle never moves. A channel's transport sits
  in a pinned box (`Pin<Box<ChannelStream>>`, private to `Channel`) and a
  stream is neither `Unpin` nor `Copy`, so safe code cannot move or swap one;
  `event/channel.rs` is the one place it is projected. Callbacks carry a
  channel id rather than an address wherever libuv allows it, and the
  C-shaped event (`RawEvent`: a handler and one word) is built only here; the
  event queue itself (`event_queue.rs`) is a table by id and forbids unsafe
  code. The `# Safety` sections say the rest: "`stream` has been through
  `rstream_init`", "`uv_loop` is live and `stream` points at storage that
  does not move afterwards".
- **Editor-shaped, still here:** the listening socket's address held in a
  `[c_char; 256]` and split in place; the read stream's block positions,
  raw pointers into a pool block; a spawned child's `char **` argv and
  environment. Each is listed for later work (see below).
- **Retires:** with libuv. Not planned; the three items above can leave
  without that.

### `tui/{tui,input,output,events}.rs` — the terminal's libuv handles

- **What:** the output tty/pipe handle on a private write loop, the startup
  timer, the window-size signal and the suspend `kill` (`tui.rs` 29); stdin's
  read stream, the escape and query timers and `tcgetattr` (`input.rs` 25);
  the UI events that reach libuv (`events.rs` 11); the flush's one
  `uv_write` (`output.rs` 4).
- **Contract:** the TUI's state is one cell for the life of the process;
  every callback looks it up rather than carrying its address, and the
  lifecycle steps that turn the event loop hold no borrow of it while they
  wait. termkey's parser, the terminfo description, negotiation and painting
  are safe Rust beside these files.
- **Retires:** with libuv. Not planned.

### `xdiff/` — libxdiff's whitespace

- **What:** one read of glibc's character-class table (`xdiff/ffi.rs`):
  which bytes are whitespace is `isspace(3)`'s answer in the user's locale.
- **Retires:** if the diff engine's whitespace became ASCII-only — a
  behaviour change, so kept.

### `os/` — the operating system

- **What:** in each file only the foreign call, its declaration and the
  buffer it fills: libuv's `uv_fs_*` and libc's `xattr`/`FILE`/`fcntl`/`write`
  (`fs/` 43), the environment, `uv_random`, `uname` and the `environ` walk
  (`env/mod.rs` 13), `setlocale`/`nl_langinfo`/gettext's binds (`lang.rs` 5),
  the `getpw*` family (`users.rs` 8), the clocks and `strftime`/`strptime`
  (`time.rs` 10), stdin's read stream and the loop polls (`input.rs` 10), the
  PTY — `forkpty`, `ioctl`, `ptsname`, and in the forked child
  `setsid`/`execvp` (`pty_proc_unix/mod.rs` 38) — signals (`signal.rs` 11),
  `dlopen` for `libcall()` (`dl.rs` 13), `iconv` (`iconv.rs` 7), process
  inspection (`proc.rs` 7), and the C library surface the `libc` crate
  lacks (`cshim.rs` 72: declarations — among them the four C globals
  `environ`, `stdout`, `stderr` and gettext's catalogue counter, `static mut`
  because C writes them — and the stand-ins the Miri lane runs instead of
  libc, one of them the C-variadic `snprintf`).
- **Contract:** each region's `// SAFETY:` comment names the buffer and
  length handed to the call; descriptors the editor owns are
  `std::fs::File`/`OwnedFd`/`BorrowedFd`, so closing one twice is not
  expressible. Reading one variable, the working directory, `realpath`,
  the executable path and opening files are `std`, not here.
- **Retires:** never; this is the operating system.

### `allocator.rs` — the global allocator

- **What:** Rust's global allocator over the same libc `malloc` the
  `xmalloc` family uses (`malloc`, `posix_memalign`, `realloc`, `free`; 20).
- **Contract:** because both sides draw from one allocator, a block can cross
  the C boundary in either direction without a copy; adopting a `malloc`
  block as a smaller-capacity `Vec` is sound because `free` and `realloc`
  ignore the stated size. The `GlobalAlloc` trait's own contract covers the
  rest.
- **Retires:** never.

### `memory/` — `xmalloc`, the block pool, owned C strings

- **What:** the `xmalloc` family with its out-of-memory retry and the
  allocation log the unit suite asserts sequences against, and the C-shaped
  string helpers (`xstrdup`, `xstrlcpy`, `xmemdupz`, …) the perimeter's
  callers and the unit specs still call (`mod.rs` 65); `XString` and
  `ThinCString`, the owned NUL-terminated strings that adopt a `malloc`
  block from C and hand one back (`thin_cstring.rs` 18, `xstring.rs` 9); the
  block pool the event loop reads into, whose reuse list is chained through
  each spent block's first bytes (`blocks.rs` 7).
- **Contract:** a block from the family is freed once, with `xfree`; the
  retry frees the scrollback and memfile caches, so no borrow of them may be
  live across an allocation (`try_to_free_memory`'s `# Safety`); a string
  helper's caller supplies NUL-terminated input and a destination of the
  stated size.
- **Retires:** the pointer-form string helpers go with their last perimeter
  and unit-spec callers; the family and the pool stay.

### `global_cell.rs` — the cell under every global

- **What:** `GlobalCell<T>`, the replacement for the C globals c2rust
  rendered as mutable statics: one `UnsafeCell` and the `Sync` promise
  (13).
- **Contract:** editor state is touched from the main thread only, and no
  reference handed out by `with`/`with_mut`/`with_field` overlaps another
  access to the same cell. **Debug builds check both** — the thread on
  every access, the borrows RefCell-style (a conflicting re-entry panics,
  and `with_field` loans lock the field and the record); release builds
  compile the checks out, which is why it is a contract. `ptr()`, the raw
  escape hatch, is used only on the perimeter (22 sites: the globals libuv
  holds by address, and the cell's own tests).
- **Retires:** never; it is the replacement.

### `winlayer.rs`, `winlayer/` — the window, buffer, grid and terminal handles

- **What:** `Win`, `Buf`, `TabPage`, `FrameRef`, `PosRef`, `Live<T>`, the
  screen grid handle `GridRef` and the terminal handle `Term` — each a
  `Copy` value over one pointer (`winlayer.rs` 22, `graph.rs` 17, `live.rs`
  5, `grid.rs` 4, `terminal.rs` 3). Building one from an address is the
  unsafe step; dereferencing it is not. `curwin`/`curbuf`/`curtab` are cells
  holding a raw pointer (`curwin` is a C symbol a functional spec reads).
- **Contract:** the constructor's caller promises the object is live for as
  long as the handle is used. The handle registries build handles from ids
  and are safe. **The re-entry rule:** a handle held across a call that may
  fire an autocommand or run Lua or Vimscript is re-derived from its id
  afterwards, and no `&mut` reached through one is held across such a call.
  A freed window's block is an owned `ParkedWindow`, freed by dropping it;
  freeing an unregistered window is debug-asserted impossible.
- **Retires:** only with a different object model (owning registries with
  ids end to end). Not planned; the settled verdict is that re-entry handles
  stay handles.

### `eval/typval/handle.rs` — Vimscript value handles

- **What:** `ListRef`, `DictRef`, `BlobRef` and `PartialRef`, and a
  dictionary's item store (51): the dereference core (16), allocation and
  release (14), the item slots that name a boxed or an embedded `DictItem`
  (10), `TypVal::bit_copy` and its two users (3), the deep copy's memo (2),
  two raw bridges for callers that still hold a pointer (2) and four test
  cases.
- **Contract:** a value is reached from many holders at once and the
  evaluator re-enters at every call, so `edit()` lends `&mut` from a shared
  handle for as long as no user code runs. **Debug builds check it:** two
  overlapping `edit()`s of one object, or a `DerefMut` under an `edit()`,
  panic (`typval/edit.rs`). The reference count is a `Cell`, so a count read
  never forms `&mut`. `bit_copy` duplicates a value without its reference:
  exactly one of the two may be released (`docs/unsafe-fn-allowlist.md`).
- **Retires:** only with a different value model. Two cuts are listed: Lists
  and Dictionaries in a `Box` (needs the unit suite's allocation log
  rewritten) and one generic handle type (needs ffigen to emit the C
  aliases from it).

### `vterm/abi/` — libvterm's C interface

- **What:** the 30 exports [docs/abi-ledger.md](abi-ledger.md) lists, each a
  front over a method of the Rust emulator (`mod.rs` 33); the callback tables
  a C client installs, wrapped as the emulator's report traits (`foreign.rs`
  48); the character-set tables the specs call (`charset.rs` 9). The emulator
  itself — parser, state machine, screen, pen, encodings, keyboard, mouse —
  is safe Rust.
- **Contract:** a pointer an export receives is one the emulator handed out;
  a table's data pointer is kept alive by whoever installed it. The structs
  C reads keep libvterm's byte layout, because the C fixture and the
  battery's `vtsweep` read their fields directly; owned data sits behind
  `Box`es in pointer slots no reader reads.
- **Retires:** together with its three C clients (upstream's unit fixture,
  the unit spec and `vtsweep`), which means re-expressing upstream's
  libvterm tests in Rust.

### `harness.rs` — editor symbols the harnesses resolve

- **What:** the editor rows of [docs/abi-ledger.md](abi-ledger.md):
  `starting` and `test_disable_char_avail` (the old-style harness's
  overrides), `early_init`/`event_init` (the unit harness boots the editor),
  the UTF-8 readers and `utf8len_tab`, `schar_get`, `display_tick`, the
  autocommand block pair and two UI events (8 statements).
- **Contract:** each function is a front over its module's safe slice form
  that reads the C caller's pointer here — the UTF-8 readers read at most one
  sequence, stopping at the NUL. Each data symbol is defined here and
  re-exported by its module, so the module forbids unsafe code.
- **Retires:** row by row, with the spec or fixture that resolves the name.

## Soundness by contract

Five places are sound by a rule the type system does not hold. Each was
decided in writing; each has a cheaper-to-break form that was measured and
not taken, or a fix listed for later.

- **`memline::held_line`** (`memline/lines.rs`): the regexp matcher borrows
  a buffer line for as long as its buffer handle lives, on upstream's
  `ml_get_buf` contract — the bytes are good until another line of the
  buffer is read — and fetches the line again after anything that may have
  moved it (`\%V`, `\%'m`, a back-reference to another line). The sound
  form, a copy of each line a match reads, measured +0.6 % on the regexp
  benchmark, most of it the per-line context the copy needs; a matcher-shaped
  line cursor is the listed fix.
- **The Lua stack's consuming operations** (`lua/stack/`): an operation that
  consumes values off the top consumes values its view pushed. Debug builds
  check it; the release check cost 0.8–1.7 % on the Lua benchmarks, so the
  row's text is the contract.
- **The handle borrow model** (`winlayer`): a `Copy` handle lends `&`/`&mut`
  into an object the type system cannot tie it to — `Buf::header`/
  `header_mut` (an undo header through the buffer), `Win::syn_block`/
  `syn_block_mut` (which may be the buffer's own block), `GridRef` (the
  compositor reads a grid while it is drawn). Two copies of one handle can
  produce overlapping borrows; the rule is that no borrow is held across an
  access through another handle or across user code.
- **`TypVal::named_for_call`** (`eval/typval/handle.rs`): a function's `a:`
  and `a:000` items are `bit_copy`s naming the caller's arguments;
  `cleanup_function_call` disowns them, or they are upgraded to real copies
  when the scope outlives the call. Residual: a panic unwinding through a
  call would release the named values a second time while the process dies.
- **The emulator's offset keepers** (`types/vterm*.rs`): `_lineinfo_slots`
  and `_buffer_slot` exist only so that C declarations the battery and the
  fixture compile keep reading the right offsets. Not unsafe in Rust, but a
  layout contract with C; re-cutting `vtsweep`'s declarations would drop them.

## Listed for later

Editor-shaped residue on the perimeter, each with what removes it: the
socket's address buffer (build the watcher whole instead of through
`Box::new_uninit`), the read stream's block positions (a read callback that
hands a slice), a child's `char **` argv/environment (an owned argv type
handed to libuv at the one spawn call), `handle.rs`'s `Box` allocation and
generic handle, the matcher's line cursor (`held_line`), and the emulator's
offset keepers.
