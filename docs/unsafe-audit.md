# The unsafe audit

This is the whole of the unsafe code in the editor, written so that someone
who does not know the code base — or Rust's `unsafe` in any depth — can check
it: what is unsafe, where it is, why it cannot be otherwise, and what would
have to change for it to go.

## The shape

Rust checks memory safety at compile time everywhere except inside `unsafe`
code, where the programmer promises what the compiler cannot verify. In this
tree that code is confined to **65 files**, holding **1,614 unchecked
statements** — every one of them on the _perimeter_ ([docs/perimeter.md](perimeter.md)),
the list of modules where the program meets something it does not own. The
other 1,287 source files of the editor (`crates/nvim/src`, 1,352 files) contain
no unsafe code, and 1,280 of them say so with `#![forbid(unsafe_code)]`, which
the compiler enforces; the remaining seven are module roots that cannot carry
the attribute because a file beneath them is on the perimeter. The unsafe code
falls into four classes:

- **A foreign ABI — 34 files, 1,039 statements.** Calls into C libraries the
  editor links: LuaJIT and its libraries, libuv (the event loop, processes,
  sockets, the terminal's I/O), tree-sitter and libxdiff. The unsafe is the
  call itself, and the reads of what the library answers.
- **The operating system — 14 files, 237 statements.** System calls the
  standard library does not offer, or offers with different error text:
  libuv's file-system calls, the environment, locales, the user database,
  clocks, the PTY, signals, `dlopen` and `iconv`.
- **Raw memory — 13 files, 244 statements.** The few primitives every safe
  type above them is built on: the allocator, the `xmalloc` family and the
  owned C strings, the checked cell that replaces C's global variables, the
  type-stable slots handles check their target's liveness against, and
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
- Every `unsafe fn` (229) carries a `# Safety` section saying what its caller
  must guarantee; the ratchet fails on one without. An `unsafe fn` whose
  signature shows no raw pointer needs a row in
  [docs/unsafe-fn-allowlist.md](unsafe-fn-allowlist.md) (five rows today).
- Checked builds add one check the release build compiles out: `GlobalCell`'s
  per-access main-thread check (its borrow checks, and the overlapping-edit
  check on Vimscript values, are in every build). It is the cfg
  `soundness_checks`, which `--features soundness-checks` turns on in a
  release build too. What the safe API over the unsafe code leaves to those
  checks, or to convention, is surveyed class by class in
  [docs/safe-surface.md](safe-surface.md). The functional, old-style
  and unit suites run against debug builds; whole-suite AddressSanitizer runs
  and Miri (over the crate's own tests) back them. `just miri-perimeter` runs
  Miri over the raw-memory rows' own tests alone: each of those files holds
  a `tests` module the recipe names by its whole path, and
  `just ratchet --check` fails on one that does not.

To check the numbers here: `python3 scripts/ratchet.py --dimension
unsafe_placement` lists any misplaced attribute (none);
`metrics/ratchet.json` holds each file's `unsafe_stmts`; `rg -l
'allow\(unsafe_code\)' crates/nvim/src` lists the 65 files.

## The rows

| row                                         | class    | files | statements | `unsafe fn`s | what it holds                                    |
| ------------------------------------------- | -------- | ----: | ---------: | -----------: | ------------------------------------------------ |
| `lua/ffi.rs`, `lua/executor/{luv,state}.rs` | foreign  |     3 |        105 |            8 | LuaJIT's declarations, luv's hooks, state making |
| `lua/stack/`                                | foreign  |     5 |        113 |            7 | the typed Lua stack                              |
| `lua/treesitter/`                           | foreign  |    10 |        204 |            9 | the tree-sitter library                          |
| `event/`                                    | foreign  |    11 |        551 |           90 | libuv: loop, streams, timers, signals, children  |
| `tui/{tui,input,output,events}.rs`          | foreign  |     4 |         69 |            6 | the terminal's libuv handles                     |
| `xdiff/`                                    | foreign  |     1 |          1 |            0 | `isspace(3)`'s table                             |
| `os/`                                       | OS       |    14 |        237 |           15 | system calls                                     |
| `allocator.rs`                              | memory   |     1 |         16 |            4 | the global allocator                             |
| `memory/`                                   | memory   |     4 |         99 |           36 | `xmalloc`, the block pool, owned C strings       |
| `global_cell.rs`                            | memory   |     1 |         13 |            0 | the cell under every global                      |
| `slot.rs`, `slot/`                          | memory   |     2 |         41 |            7 | type-stable slots and their checked handles      |
| `winlayer.rs`, `winlayer/`                  | memory   |     6 |         49 |            9 | window, buffer, grid and terminal handles        |
| `eval/typval/handle.rs`                     | memory   |     1 |         50 |            8 | Vimscript value handles                          |
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
  (`mod.rs` 62, `dispatch.rs` 24, `userdata.rs` 16, `debug.rs` 9,
  `table.rs` 2). Every region is one LuaJIT call, a read of what one
  answered, a userdata block, or the read of the state's `top`/`base`
  fields. The rest of `lua/` reaches Lua only through this module and
  forbids unsafe code.
- **Contract:** a `Stack` is a view of a C function's frame rooted at a
  base slot. Dropping or moving a slot at or below the base aborts in every
  build. An operation that consumes values off the top (`set_field`,
  `raw_set`, `next`, `concat`, …) consumes values this view pushed — an
  `assert!` in every build. The `lua_State`
  layout is read directly: LuaJIT is pinned by `nix/deps.json`, and
  `check_state_layout` aborts on every new state whose layout drifted. A Lua
  error (a C `longjmp`) is raised only after the Rust body has returned, so
  no Rust frame is unwound by it. A userdata block holds its value beside a
  collected record: `__gc` is reachable from plain Lua
  (`getmetatable(x).__gc(x)`), so the first one drops the value and marks
  the block, a second does nothing, and the accessors refuse a marked block
  with a Lua error. A `CollectedMark` is that record read without a lookup,
  sound while its holder keeps the userdatum reachable.
- **Retires:** with LuaJIT. Not planned.

### `lua/treesitter/` — the tree-sitter library

- **What:** parsers, trees, queries and cursors are C objects with C
  lifetimes, held by Lua userdata whose finaliser deletes them (`mod.rs` 76,
  `node.rs` 38, `lang.rs` 24, `parser.rs` 20, `query.rs` 14, `cursor.rs` 12,
  `tree.rs` 8, `ranges.rs` 7, `logger.rs` 4, `init.rs` 1). The unsafe is the
  calls and the reads of the arrays and strings they answer.
- **Contract:** a userdata's object lives until its first `__gc` (which
  plain Lua can call early); a node, cursor or match that points into
  another userdatum's object checks that userdatum's collected record before
  every use (a node through its tree's `CollectedMark`), and a parse lends
  its parser and old tree out of their userdata while the Lua log callback
  can run. A query's strings and capture arrays are read only while the
  query is alive; the Lua stack is reached through `lua/stack/` only.
- **Known, upstream too:** `_ts_inspect_language`'s subtype off-by-one is
  upstream's behaviour, kept. (A parse re-entered from its own logger is
  refused: the parser is lent out.)
- **Retires:** if the C library were replaced. Not planned.

### `event/` — libuv

- **What:** libuv's loop, streams, timers, signal watchers and child
  processes are C objects libuv holds _by address_ (`socket/mod.rs` 102,
  `libuv.rs` 100 — mostly the declarations — `proc.rs` 70, `loop.rs` 58,
  `channel.rs` 51, `libuv_proc.rs` 48, `rstream.rs` 41, `stream.rs` 34,
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
  `xmalloc` family uses (`malloc`, `posix_memalign`, `realloc`, `free`; 16).
- **Contract:** because both sides draw from one allocator, a block can cross
  the C boundary in either direction without a copy; adopting a `malloc`
  block as a smaller-capacity `Vec` is sound because `free` and `realloc`
  ignore the stated size. The `GlobalAlloc` trait's own contract covers the
  rest. A refused request calls out (cold, out of line) to the safe
  out-of-memory exit (`startup/out_of_memory.rs`); the allocator holds no
  state, so that exit may allocate through it again.
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
  access to the same cell. **The borrows are checked in every build**,
  RefCell-style in a word beside the value (a conflicting re-entry panics;
  a `with`/`with_field` loan is a node in the borrower's frame, so a write
  checks only the loans open on its cell). **The thread is a contract**,
  held by keeping the ways off the main thread few and audited (luv's
  thread states and their function list, the panic hook and the
  allocation-failure path, and thread starts, which clippy refuses outside
  an audited site; [safe-surface.md](safe-surface.md) S2), and checked on
  every access in checked builds only. The value comes first (`repr(C)`),
  so the exported cells keep their C address. `ptr()`, the raw
  escape hatch, is used only on the perimeter (22 sites: the globals libuv
  holds by address, and the cell's own tests).
- **Retires:** never; it is the replacement.

### `slot.rs`, `slot/` — type-stable slots

- **What:** `SlotPool<T>`, which allocates each slot once and never gives
  its memory back, `SlotRef<T>`, a `Copy` handle of the slot's address and
  the generation its value was handed out under, and `SlotBox<T>`, the one
  owner of a value in its type's pool, and the guards a value is lent
  through (`slot/guards.rs`): `SlotEdit` (the edit bit), `SlotRead` (a
  counted borrow) and `SlotPart` (one interior part lent mutably beside
  reads of the rest -- a buffer's memline) (41): the header reads (generation, index, reader count; 3), the value's
  writes, reads and drops as it enters and leaves a slot (7), the
  dereferences of a live handle, a detached slot, a vacant place being
  filled, the pool's own borrow and the two guards (9), and the `unsafe
fn`s that take a value address apart (`slot_of`, `generation_at`,
  `word_at`, `SlotRef::from_parts`; the graph handles' constructors and
  checks from a bare address) or lend through one (`edit_at`, `read_at`,
  `check_write`), with their tests (13).
  `IdTable` (`id_table.rs`, forbid) is built on it, and so are the window,
  buffer, tab page and frame pools (`winlayer/handles.rs`, forbid).
- **Contract:** a slot's value is initialised exactly while its generation
  is odd; every handle is handed an odd generation, and emptying a slot
  moves it on (to even, or to zero for a slot that reached the limit, which
  is never reused). Because the memory stays a `Slot<T>` for the rest of the
  run, reading the generation through any handle ever made is sound, so a
  dereference is one load and one compare, and a stale one panics. A pool
  has no destructor: every pool is a field of a static. A value address
  handed to any of the value-address `unsafe fn`s must be a slot's
  (`SlotRef::raw`). The generation's top bit is the edit bit: `edit_at`
  sets it while its guard lends the value, so every exact compare refuses
  meanwhile; `read_at` counts its guard, and an edit, `check_write` or
  emptying the slot refuses while the count is not zero. `lend_part_at`
  counts one reader and sets the reader word's top bit, so a second part
  loan and a part read (`check_part_unlent`) refuse; the part lives in an
  `UnsafeCell` and is reached only through these. `abandon_loans` zeroes
  every guard's mark, for the emergency exits alone (their guards' frames
  never resume). Exclusivity holds
  for everything lent through those guards (`Win`, `Buf`, `TabPage`,
  `FrameRef`, `PosRef`); a `SlotRef`'s `DerefMut` through two copies still
  lends overlapping `&mut` ([safe-surface.md](safe-surface.md) S5).
- **Retires:** never; it is the floor the window, buffer and tab page
  handles stand on.

### `winlayer.rs`, `winlayer/` — the window, buffer, grid and terminal handles

- **What:** `Win`, `Buf`, `TabPage`, `FrameRef`, `PosRef`, the
  screen grid handle `GridRef` and the terminal handle `Term` — each a
  `Copy` value over one pointer (`winlayer.rs` 10, `graph.rs` 17,
  `deref.rs` 18 — the checked handles, the place getters and setters
  (`deref/buf_fields.rs`, `deref/win_fields.rs`) and `edit()`/`read()` —, `grid.rs`
  2, `terminal.rs` 2). Building one from an address is the
  unsafe step; dereferencing it is not. `curwin`/`CURBUF`/`CURTAB` are cells
  holding a raw pointer (`curwin` is a C symbol a functional spec reads).
- **Contract:** the constructor's caller promises the object is live when
  the handle is made: the handle takes its slot's generation then. The
  handle registries build handles from ids and are safe. Every
  `Win`/`Buf`/`TabPage`/`FrameRef` dereference compares that generation
  with the slot's header (`winlayer/deref.rs`: one load, one compare), and
  a slot's memory is never given back, so a handle that outlived its object
  panics instead of reading freed memory or the slot's next tenant.
  **The re-entry rule:** a handle held across a call that may fire an
  autocommand or run Lua or Vimscript is re-derived from its id afterwards
  (now checked: the stale use panics), and no `&mut` reached through one is
  held across such a call (`Win`, `Buf`, `TabPage`, `FrameRef` and
  `PosRef` lend no reference but through checked guards, so a breach
  panics; S5 in [safe-surface.md](safe-surface.md)). A freed window's block is an owned
  `ParkedWindow`, freed by dropping it; freeing an unregistered window is
  debug-asserted impossible.
- **Retires:** only with a different object model (owning registries with
  ids end to end). Not planned; the settled verdict is that re-entry handles
  stay handles.

### `eval/typval/handle.rs` — Vimscript value handles

- **What:** `ListRef`, `DictRef`, `BlobRef` and `PartialRef`, and a
  dictionary's item store (50): the dereference core (13), the borrow word
  each object carries, reached through its own bytes (2), allocation and
  release (14), the item slots that name a boxed or an embedded `DictItem`
  (10), `TypVal::bit_copy` and its two users (3), the deep copy's memo (2),
  two raw bridges for callers that still hold a pointer (2) and four test
  cases.
- **Contract:** a value is reached from many holders at once and the
  evaluator re-enters at every call, so a List, Dict or Blob is borrowed
  only through `read()` and `edit()` from a shared handle. **Every build
  counts both:** any number of reads or one edit, as a `RefCell` (the
  object's `Lent` word, `typval/edit.rs`); a partial is never written once
  built, except by its last holder (`PartialRef::get_mut`). The reference
  count and the word are `Cell`s, each reached through its own bytes, so
  neither forms a borrow of the whole object. `bit_copy` duplicates a value without its reference:
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

Four places are sound by a rule the type system does not hold. Each was
decided in writing; each has a cheaper-to-break form that was measured and
not taken, or a fix listed for later.

- **The Lua stack's consuming operations** (`lua/stack/`): an operation that
  consumes values off the top consumes values its view pushed. Debug builds
  check it; the release check cost 0.8–1.7 % on the Lua benchmarks, so the
  row's text is the contract.
- **The handle borrow model** (`winlayer`): a `Copy` handle lends `&`/`&mut`
  into an object the type system cannot tie it to — `Buf::header`/
  `header_mut` (an undo header through the buffer), `Win::syn_block`
  (which may be the buffer's own block; its writes are in-place writers
  that lend no `&mut`), `GridRef`'s
  `Deref` (the compositor reads a grid while it is drawn; its writes are
  in-place writers that lend no `&mut`). Two copies of one handle can
  produce overlapping borrows; the rule is that no borrow is held across an
  access through another handle or across user code.
- **`TypVal::named_for_call`** (`eval/typval/handle.rs`): a function's `a:`
  and `a:000` items are `bit_copy`s naming the caller's arguments;
  `cleanup_function_call` disowns them, or they are upgraded to real copies
  when the scope outlives the call. Residual: a panic unwinding through a
  call would release the named values a second time while the process dies.
- **The emulator's offset keepers** (`vterm/types.rs`, `vterm/internal.rs`): `_lineinfo_slots`
  and `_buffer_slot` exist only so that C declarations the battery and the
  fixture compile keep reading the right offsets. Not unsafe in Rust, but a
  layout contract with C; re-cutting `vtsweep`'s declarations would drop them.

## Listed for later

Editor-shaped residue on the perimeter, each with what removes it: the
socket's address buffer (build the watcher whole instead of through
`Box::new_uninit`), the read stream's block positions (a read callback that
hands a slice), a child's `char **` argv/environment (an owned argv type
handed to libuv at the one spawn call), `handle.rs`'s `Box` allocation and
generic handle, and the emulator's
offset keepers.
