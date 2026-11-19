# The safe surface

`forbid(unsafe_code)` says where unsafe code is written; it does not say that
the **safe** API the perimeter exports over it holds. This document is the
survey of that API: every safe `pub`/`pub(crate)` function, method and trait
impl (`Deref`/`DerefMut`, `Sync`/`Send`, `Drop`) a
[perimeter](perimeter.md) file exports, classed by what keeps it sound.

- **sound** — no caller written in safe Rust can cause undefined behaviour
  through it.
- **sound-by-debug-check** — a caller can, and builds with the
  `soundness_checks` cfg panic where it does. That cfg is on in every build
  with debug assertions and in a release build with the `soundness-checks`
  feature (`cargo build --release --features soundness-checks`);
  `crates/nvim/build.rs` derives it. A shipped release build checks nothing.
- **sound-by-convention** — a caller can, and nothing checks it.

The work list is by **class**: rows that fail by the same mechanism and close
by the same fix. A class closes one of three ways — a type that holds the
invariant, a check that is always on, or an `unsafe fn` with a real contract —
and a row is struck here when its class closes. No fourth state.

## What the tracker costs

Every `soundness_checks` check, in a release build, run through the suites
and the A/B benches (codegen-units = 1 both sides, one sitting, cachegrind):

- **It never fired.** Functional (9,511 tests, `--jobs=8`), oldtest (4,682)
  and `tests/unit` (1,156) under the checked release binary: no borrow,
  loan, thread or stack panic. The run found one real bug, not the tracker's
  kind: three socket waits read a flag through a shared borrow the optimiser
  hoisted, so `nvim --listen <address in use>` hung in every release build
  (fixed: `Cell`). One release-only failure is the build, not a check:
  `Test_pattern_compile_speed` finishes under its 10 ms floor.
- **It costs +11 % to +78 % instructions:**

  | bench      | plain Ir | checked Ir |    Δ |
  | ---------- | -------: | ---------: | ---: |
  | mlbench    |   7.86 G |     8.76 G | +11% |
  | markbench  |   5.03 G |     6.19 G | +23% |
  | inbench    |  30.94 G |    38.68 G | +25% |
  | evalbench  |  65.35 G |    83.37 G | +28% |
  | rebench    |  17.68 G |    22.86 G | +29% |
  | edbench    |  14.60 G |    18.93 G | +30% |
  | scrbench   |  13.18 G |    22.24 G | +69% |
  | spellbench |  11.37 G |    20.27 G | +78% |

- **Where (cg_annotate, self Ir).** The tracker was built for unoptimised
  debug builds: one shared borrow table in `nvim-mainthread`, reached by a
  call per question. spellbench's +8.9 G: per-access table lookups while any
  borrow is live 5.4 G (`BorrowTable::state` + `check_read_slow` — the whole
  trie walk runs inside `LOADED_LANGS.with`, so every global read takes the
  slow path), the idle shortcut 1.4 G, the main-thread call 0.8 G.
  scrbench's +9.1 G: `acquire`/`release` 3.7 G (a `with_mut` of `LINEBUF`
  per drawn cell), idle 1.2 G, `state` 0.4 G, main-thread 0.25 G.
  evalbench's parser phase (+4.5 G): `acquire`/`release` 2.0 G, idle
  0.7 G, the value handles' edit loans (`EditTable::lend`/`give_back`)
  0.29 G (+3 % of the plain phase). The Lua stack's checks are one inline
  compare and do not show.

**Verdict.** The tracker _as built_ cannot be always-on. Its cost is the
table, not the question: a borrow flag inline beside each cell's value
(`RefCell`'s shape) answers `with`/`with_mut` and every `get`/`set` with a
load of a line the access touches anyway. So S1/S2 are **check-always after a
restructure of the tracker**, re-measured; S5/S6/S8 have no tracker to measure
and are **restructure**.

## Classes

Sizes: XS < 1 h, S ~1–2 h, M ~half a day, L ~days, XL multi-slice.

### S1 — cell borrows (sound-by-debug-check)

`GlobalCell::{with, with_mut, take, with_field, with_at}` lend a reference
for a closure; `get`/`set`/`get_field`/`set_field`/`replace_field` and the
`state_record!` accessors (`get`/`set`/`replace`/`take`/`with`/`with_mut`/
`update`, `Level`) copy in and out. Debug builds track every loan.

- **Breaks:** a `with_mut` re-entered from inside a `with`/`with_mut` on the
  same cell → two `&mut` in release. Real shapes: an autocommand, Lua or
  Vimscript run inside the closure (`tui::with_tui`, the options record); a
  `get`/`set` of the cell inside its own `with_mut`; a `with_field` loan's
  field written from inside it; `LOADED_LANGS.with` around `os_breakcheck()`
  (libuv callbacks run there — sound today only because fast callbacks may
  not set `'spelllang'`).
- **Closure:** check-always. Replace the shared table with a per-cell inline
  borrow count (`GlobalCell` loses `repr(transparent)`; the exported cells in
  [exported-cells.md](exported-cells.md) and the `curwin` C symbol need a
  layout story), measure, then drop the cfg. Hot sites the measurement names
  (`put_cell`'s per-cell `LINEBUF.with_mut`) take the buffer once. **M–L**,
  two slices (tracker, then hot sites + flip).

### S2 — main-thread confinement (sound-by-debug-check)

`unsafe impl<T> Sync for GlobalCell<T>` for any `T`; `ptr()`, `get`, `set`
and the rest are callable from any thread.

- **Breaks:** any thread — a `vim.uv.new_thread` state, a libuv pool
  callback, `std::thread::spawn` — touching a cell: a data race in release,
  and a `!Send` value (`Rc`) moved across threads.
- **Closure:** check-always, inlined: the thread question as a TLS load in
  the caller (today a cross-crate call, 4 % of spellbench), measured with
  S1's slice. **S.**

### S3 — `SharedCell<T>` (sound-by-convention)

`Sync` for any `T`, safe `get`/`set` from any thread, by design: `IN_SCRIPT`,
`MAIN_THREAD`, `NLUA_DISABLE_PRELOAD`, `TRYING_TO_FREE`, `MAIN_LOOP`.

- **Breaks:** two threads `set`ting one (a Lua thread's state set-up
  writing `IN_SCRIPT` while the main thread reads it).
- **Closure:** type holds it — atomics for the four scalars, a pointer-only
  cell (no safe `get`/`set`) for `MAIN_LOOP`, `SharedCell` deleted. **S.**

### S4 — `ConstTable<T>` (sound-by-convention)

`Sync` for any `T` on the promise that the contents are never written.

- **Breaks:** a table over a type with interior mutability (a `Cell`) read
  from two threads.
- **Closure:** `unsafe const fn new` whose contract is "no interior
  mutability; pointers name immutable statics" (4 tables). **XS.**

### S5 — `Copy` object handles lending `&`/`&mut` (sound-by-convention)

`Win`, `Buf`, `TabPage`, `FrameRef`, `PosRef`, `Live<T>`, `GridRef`, `Term`:
`Deref`/`DerefMut` from a `Copy` value. Constructors are `unsafe` (or the
registries, which check); every dereference after is safe and unchecked.
Rows: the eight `Deref`/`DerefMut` pairs; `Win::syn_block`/`syn_block_mut`
(may be the buffer's `b_s`); `GridRef::of_cell` (a cell's grid outside its
tracker); `GridRef`'s null "unresolved" form; `memline::held_line` (a line's
bytes for the life of a `&Buf`, i.e. a local).

- **Breaks — aliasing:** two copies give two live `&mut`
  (`let c = &mut w1.w_cursor; w2.w_cursor.lnum = 1; c.col = 0`);
  `syn_block_mut()` held while the buffer's block is reached through a `Buf`;
  `GRID.with(|g| { grid_ref.cols = 0; g.cols })`; `held_line`'s slice
  outliving the cache when a copy of the `Buf` reads another line.
- **Breaks — liveness:** a handle held across a call that runs user code
  (an autocommand, a Lua callback — the p45 hazard — Vimscript) and
  dereferenced after `:bwipeout`/`:close` freed the object: use-after-free in
  safe code. The re-entry rule in `winlayer`'s docs is the whole defence.
- **Closure:** restructure. Liveness: a handle is an id plus a generation
  checked against a side table on dereference (the object's own memory
  cannot carry it — it may be freed); a check-always that wants its own
  measurement. Aliasing: no `DerefMut` on a `Copy` type — reads by `&`
  through a checked handle, writes through a scoped, tracked `with_mut`
  (S1's tracker keyed by object). Order: `GridRef` + `of_cell` (S), `Term`
  (S), `syn_block_mut` (13 sites, S), `held_line` (the matcher's line
  cursor, M), `Live<T>` users (M), then `Win`/`Buf`/`TabPage` (XL; 553 files
  name them). **XL** overall.

### S6 — value-handle edits (write half by debug check, read half by convention)

`ListRef`/`DictRef`/`BlobRef`/`PartialRef`: `edit(&self) -> Edit<'_, T>`,
`DerefMut`, `Deref`, and the `&Dict` walks (`items`, `items_mut`) through
them.

- **Breaks — write/write:** two live `edit()`s of one object through two
  handles, or a `DerefMut` under an `edit()` (checked: `Loan`).
- **Breaks — read/write:** a `&List` from one handle's `Deref` live while a
  clone's `edit()` reallocates (`for item in l.items() { other.edit()… }`),
  or an `edit()` held across a call that runs user code which reads the
  object. Unchecked.
- **Closure:** write half check-always (the loan table is S1's shape;
  restructure it the same way, measure on evalbench); read half restructure
  — `Deref` answers a tracked shared loan, or the walks take the handle and
  re-borrow per item. **M** + **L**.

### S7 — the Lua stack's consuming operations (sound-by-debug-check)

`Stack::{concat, set_field, get_table, set_table, raw_get, raw_set,
raw_set_index, set_metatable, next}`, `debug::function_info`,
`table::reference`, `userdata::set_environment`: each pops values it assumes
its view pushed.

- **Breaks:** an operation on a view that pushed too few values pops a slot
  of the view below it, or below the C frame's base — LuaJIT then reads and
  writes outside the frame.
- **Closure:** check-always (one compare of `top - base`, measured
  0.8–1.7 % on the Lua benches when this was last tried; re-measure inside
  the evalbench run), or a pushed-count token the consuming operation takes.
  **S.**

### S8 — event handles with `DerefMut` over pinned libuv objects (sound-by-convention, perimeter-internal)

`EventLoop`, `Reader`, `Conn` (`pub`), and the file-private `Uv`, `Child`,
`Signal`, `Watcher`, timer `Handle`: `Copy` wrappers with `DerefMut` to
objects libuv holds by address (`Stream` carries `PhantomPinned`). Every user
is inside `event/`.

- **Breaks:** `mem::swap(&mut *a, &mut *b)` over two `Reader`s moves two
  registered `RStream`s — the transports swap hole the pinned
  `Channel::stream` box closed, reopened one level down; or a handle used
  after its close callback freed the watcher.
- **Closure:** type holds it — `pub(super)`, and `Pin<&mut T>` projections
  (or field accessors) in place of `DerefMut`. **S–M.**

### S9 — a safe function adopting a raw pointer (sound-by-convention)

`treesitter::push_tree(stack, *mut TSTree)` takes ownership: the userdatum's
`Drop` deletes the tree.

- **Breaks:** a safe caller passing a tree it does not own → double free.
- **Closure:** `unsafe fn` with the contract ("an owned tree, or null"). **XS.**

## Closed before the survey

- `apply_autocmds` and siblings take owned names (`XString`); no borrowed
  `&CStr` crosses an autocommand.
- The `Channel` transports sit in `Pin<Box<ChannelStream>>` (S8 is the
  level below).
- A freed window's block is an owned `ParkedWindow` (`forget_window`), freed
  by dropping it; "freed twice" answers `None`, which frees nothing.

## Sound

Every other export, by row: the `os/` and `tui/` system-call fronts (owned
or borrowed slices in, owned values out; `os_strerror`/`os_err_name` answer
libuv's static or leaked strings); `memory/`'s owned strings (`XString`,
`ThinCString`: `as_cstr` scans for the NUL, `filled` hands a raw pointer
whose writes need the caller's own `unsafe`); `allocator.rs`; `xdiff/`;
the `lua/ffi.rs`, `event/libuv.rs` and `os/cshim.rs` declarations (in
`unsafe extern` blocks: unsafe to call); the `lua/treesitter/` `check_*`
readers (answer raw pointers); the `vterm/abi/` exports and callback
tables (constructed by `unsafe` code from C's tables); `harness.rs`;
`proc_init`/`libuv_proc_init`/`pty_proc_init` (store pointers only — the
`unsafe` spawn reads them); `GlobalCell::ptr`/`as_raw`, `Field`.
