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
  `crates/nvim/build.rs` derives it. A shipped release build checks none of
  these (what it does check is an always-on check, a class closed).
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
  compare and do not show (they are always on now: S7).

**Verdict.** The tracker _as built_ cannot be always-on. Its cost is the
table, not the question: a borrow flag inline beside each cell's value
(`RefCell`'s shape) answers `with`/`with_mut` and every `get`/`set` with a
load of a line the access touches anyway. So S1/S2 are **check-always after a
restructure of the tracker**, re-measured; S5/S6/S8 have no tracker to measure
and are **restructure**.

### The inline tracker, re-measured

The table is gone. Each `GlobalCell` is `repr(C)`, value first, and a
checked build adds one word after it: `FREE`, `EXCLUSIVE`, or a pointer to
the innermost shared `Loan` — a `{start, end, outer}` node in the borrowing
call's own frame, so `with` and `with_field` are the same mechanism and a
field write walks only the loans open on its own cell. A conflict path is
`#[cold]`; the emergency exits call `abandon_borrows()` (conflicts stop
panicking for good) where they used to empty the table. The thread question
is the `nvim-mainthread` thread-local byte, inlined in an optimised build.
An unchecked build has no word and no checks: its instructions are the
old release build's (−0.0 % to −1.0 %, from the hot-site changes below).

Cost of the cell checks alone, on in a release build (codegen-units = 1,
cachegrind, Ir against the unchecked build; the edit-loan and Lua-stack
checks off):

| bench      | shared table | inline: borrows + thread | inline: borrows only |
| ---------- | -----------: | -----------------------: | -------------------: |
| mlbench    |        +11 % |                   +3.0 % |               +0.7 % |
| markbench  |        +23 % |                   +3.4 % |               +1.7 % |
| inbench    |        +25 % |                   +5.8 % |               +3.3 % |
| evalbench  |        +28 % |                   +4.8 % |               +2.7 % |
| (parser)   |        +48 % |                  +11.1 % |               +5.3 % |
| rebench    |        +29 % |                   +6.5 % |               +4.6 % |
| edbench    |        +30 % |                   +8.5 % |               +3.6 % |
| scrbench   |        +69 % |                  +10.5 % |               +6.1 % |
| spellbench |        +78 % |                  +10.1 % |               +3.0 % |

The whole checked build (`--features soundness-checks`, the edit loans and
the Lua stack checks included) is +3.7 % (mlbench) to +15.3 % (the
evalbench parser phase).

Where the rest goes, by line attribution (a `line-tables-only` build of
both sides, `cg_annotate` per file and function):

- **The thread check is half of it.** The `IS_MAIN_THREAD` load and branch
  is the hottest line in every profile; the borrow compare is the other.
  Comparing the thread pointer (`fs:0`) with a stored main-thread value
  instead was _worse_ (scrbench +21.6 %): the stored value is a static every
  generic accessor names, so it is exported and read through the GOT.
- **Restructured hot sites** (kept, they cost the unchecked build nothing):
  the `'isprint'` table borrowed once per line walk (`with_byte_cells` in
  `linesize_fast`, `getvcol`, `plines_win_col`: scrbench −0.9 G of checks);
  `hl_combine_attr`'s build path out of line, so the per-cell cache hit
  stays a leaf (it had grown a full prologue: +354 M); `arabic_maycombine`
  tests the character before reading two options (every cluster step of
  every walk read `'arabicshape'`/`'termbidi'`).
- **What is left is diffuse.** No single site above ~5 % of a bench's
  delta remains: functions that inlined before and now carry a prologue
  (`script_is_lua`, `handle_subscript`, `do_one_cmd`, the `Depth`/`Bump`
  guards), per-step `GOT_INT`/option reads in the spell trie walk and the
  NFA matcher, per-byte `SPELLTAB` reads in `captype`/`spell_iswordp`, the
  per-cell `LINEBUF.with_mut` (now ~6 instructions, no call). Each is a
  restructure of its own for a fraction of a percent.

**Verdict then: not flipped**, against a threshold of ≤ 2–3 % on every
bench.

### Decided: the borrows always on, the thread question closed by audit

The borrow check is unconditional code now (not a default feature): the
borrow word is in every build, and `soundness_checks` switched only the
thread check and the value handles' edit loans (now only the thread check:
the loans went always-on with S6a). Before the flip, the
profile of the always-on build was taken apart per function and line
against the build without it (codegen-units = 1, a `line-tables-only`
build of each side, which changes no code) and the reads made once per
step of an inner loop were moved out (`breakcheck_every`'s counter is an
atomic, `'maxmempattern'` is read once per match, the spell walk's
`GOT_INT` once per breakcheck). Cost of the always-on borrow check in the
shipped release build, cachegrind Ir, codegen-units = 1, one sitting:

| bench      | without | always-on |      Δ | before the hot sites |
| ---------- | ------: | --------: | -----: | -------------------: |
| mlbench    |  7.80 G |    7.92 G | +1.6 % |               +1.7 % |
| markbench  |  5.06 G |    5.16 G | +2.0 % |               +2.0 % |
| inbench    | 30.87 G |   31.91 G | +3.4 % |               +3.8 % |
| evalbench  | 65.54 G |   67.41 G | +2.9 % |               +2.8 % |
| (parser)   |  9.32 G |    9.83 G | +5.4 % |               +5.4 % |
| rebench    | 17.70 G |   17.98 G | +1.6 % |               +5.8 % |
| edbench    | 14.59 G |   15.08 G | +3.4 % |               +3.8 % |
| scrbench   | 13.16 G |   14.00 G | +6.4 % |               +6.4 % |
| spellbench | 10.55 G |   10.85 G | +2.9 % |               +2.4 % |

("Without" is the tree before this change; "always-on" also carries the
thread-safety fixes of S2 below. The hot-site column is the flip alone.
Whole-bench swings of ±0.3 % between builds that differ by unrelated
inlining are normal here.)

What is left is diffuse: about a third is the compares themselves (the
per-cell `LINEBUF.with_mut` and search-highlight state in `win_line`, the
`ExState` guards per command, `vgetorpeek`'s typeahead loans, the spell
table's per-byte reads), the rest inlining moving — a function that grew
a compare is kept out of line and its caller pays a call and moves
(`prepare_line`, `hl_combine_attr`, `findmatchlimit`, `parse_cmd_address`).
Forcing those back in (`#[inline]`, a cold split) was tried and measured
nothing: the inliner shuffles what it keeps.

## Classes

Sizes: XS < 1 h, S ~1–2 h, M ~half a day, L ~days, XL multi-slice.

### ~~S1 — cell borrows~~ (closed: check-always)

`GlobalCell::{with, with_mut, take, with_field, with_at}` lend a reference
for a closure; `get`/`set`/`get_field`/`set_field`/`replace_field` and the
`state_record!` accessors (`get`/`set`/`replace`/`take`/`with`/`with_mut`/
`update`, `Level`) copy in and out.

- **Broke:** a `with_mut` re-entered from inside a `with`/`with_mut` on the
  same cell → two `&mut` in release. Real shapes: an autocommand, Lua or
  Vimscript run inside the closure (`tui::with_tui`, the options record); a
  `get`/`set` of the cell inside its own `with_mut`; a `with_field` loan's
  field written from inside it; `LOADED_LANGS.with` around `os_breakcheck()`.
- **Closed:** every build tracks every loan in the word beside the value
  (`FREE`, `EXCLUSIVE`, or the innermost shared `Loan` in the borrower's
  frame), and a conflicting access panics. Layout: `repr(C)`, value first,
  so an exported cell's address is still its value's; `just state-exports`
  and `just drop-glue` (default and codegen-units = 1) are clean with the
  word in. Cost: the table above.

### ~~S2 — main-thread confinement~~ (closed: `unsafe` entry points with an audited contract)

`unsafe impl<T> Sync for GlobalCell<T>` for any `T`; `ptr()`, `get`, `set`
and the rest are callable from any thread.

- **Breaks:** any thread touching a cell — a data race in release, and a
  `!Send` value (`Rc`) moved across threads.
- **Found:** a real one. A luv thread's `require` of a module that is not
  preloaded calls `nvim__get_runtime`, whose pattern list went through the
  api converter, which kept its walk stack in a `GlobalCell` (`pop_object`'s
  `OPEN_VALUES`): two threads taking and putting back one `Vec` at once. The
  same converter reached `MAIN_REFS` (a `with_mut` on every reference
  made), and `vim.diff`'s `on_hunk` reference was given back through the
  _main_ state from the thread (upstream's bug too). Fixed: the walk stack
  is per thread (`thread_local!`), the reference bookkeeping is atomics, and
  `vim.diff` gives the reference back to the state that made it.
- **Closed — every way off the main thread is an audited entry point:**
  - _luv thread states_ (`vim.uv.new_thread`, `new_work`). luv calls
    `nlua_thread_acquire_vm` on the thread; the `unsafe` call that hands it
    to luv (`nlua_init`) carries the contract. A thread state can call only
    what `nlua_init_state` gives it, and that list is written there:
    `print` (posts its text through the loop's mutex-guarded inbox),
    `add_common` (`vim.is_thread`, the sentinels' metamethods, the
    preloader), `add_vendored` (mpack, lpeg, `vim.diff`, cjson), luv with
    the thread hooks (`install_luv`; errors are posted), `thread_api`
    (`nvim__get_runtime`, against its own mutex-guarded snapshot of the
    search path), the standard library. None reaches a cell now; the
    userdata claims were already a lock (`lua/stack/userdata.rs`).
  - _Rust threads and the panic hook_: `std::thread::{spawn, scope}`,
    `Builder::{spawn, spawn_scoped}` and `std::panic::set_hook` are
    `disallowed_methods` (`crates/nvim/clippy.toml`, denied in
    `Cargo.toml`). Each of the four sites carries an `#[expect]` saying what
    its thread touches: `nvim__panic`'s thread only panics; the panic hook
    asks which thread it is on (`main_thread_is_current`) before any editor
    state and hands another thread's cleanup to the main loop; two tests
    (the allocation log, the regexp watchdog) touch thread-local state and
    an atomic.
  - _The allocator's failure path_ (`allocation_failed`, any thread): asks
    the thread first, as the panic hook does.
  - libuv's pool runs only libuv's own work (`uv_fs_*`, `getaddrinfo`);
    completion callbacks run on the loop thread. No Rust code queues pool
    work, and signals arrive through the loop.
- **Kept as a net:** builds with `soundness_checks` (every debug build, so
  every suite) still check the thread on every access, and two
  `thread_spec` cases drive the converter paths above on a thread (the
  `require` case panicked before the fix). A release build does not ask:
  that read per access cost +2 % to +5 % on its own.
- **What would reopen it:** a function added to a thread state's list, or a
  new thread or hook, that reaches a cell. The first is reviewed at
  `nlua_init_state`'s list (and caught by a debug `thread_spec` run when it
  has a case); the second fails clippy.

### ~~S3 — `SharedCell<T>`~~ (closed: type holds it)

`Sync` for any `T`, safe `get`/`set` from any thread, by design: `IN_SCRIPT`,
`MAIN_THREAD`, `NLUA_DISABLE_PRELOAD`, `TRYING_TO_FREE`, `MAIN_LOOP`. A Lua
thread's state set-up reading `IN_SCRIPT` while the main thread wrote it
was a data race in safe code.

`SharedCell` is deleted. The three flags are `AtomicBool`s
(`TRYING_TO_FREE` a `swap`, so two threads out of memory at once cannot
both enter); `MAIN_THREAD` is a `OnceLock<uv_thread_t>` (unset answers "not
the main thread", as the zero id did); `MAIN_LOOP` is a `RawStatic<Loop>`,
whose only method is `ptr()` — sharing it shares an address, and every
dereference is the caller's `unsafe`.

### ~~S4 — `ConstTable<T>`~~ (closed: type holds it)

`Sync` for any `T` on the promise that the contents are never written.
Every row type it held (`VimOption`, `CommandDefinition`, `EvalFuncDef`,
`MsgpackRpcRequestHandler`) had since become `Sync` on its own — names are
`&'static CStr`, callbacks are `fn` pointers, the option's variable a
`&'static GlobalCell` — so the four tables are plain `static` arrays and
the compiler asks the question; `ConstTable` is deleted. A row type that
gains a `Cell` or a raw pointer fails to build.

### ~~S5 — `Copy` object handles lending `&`/`&mut`~~ (closed: type holds it, checked)

~~`Win`~~, ~~`Buf`~~, ~~`TabPage`, `FrameRef`, `PosRef`~~, ~~`Live<T>`~~, ~~`GridRef`~~, ~~`Term`~~,
~~`SlotRef` (the `IdTable` views)~~:
`Deref`/`DerefMut` from a `Copy` value. Constructors are `unsafe` (or the
registries, which check); every dereference after is safe and unchecked.
Rows: the eight `Deref`/`DerefMut` pairs; ~~`Win::syn_block`~~/~~`syn_block_mut`~~
(may be the buffer's `b_s`); ~~`GridRef::of_cell` (a cell's grid outside its
tracker)~~; ~~`GridRef`'s null "unresolved" form~~; ~~`memline::held_line` (a
line's bytes for the life of a `&Buf`, i.e. a local)~~.

- **Closed — `GridRef`'s write half (type holds it):** ~~`DerefMut`~~ is
  gone. A write is one of the handle's in-place writers (`grid_writes!` in
  `winlayer/grid.rs`): each runs one `ScreenGrid` method, or stores one
  field, through a `&mut` made by a private `write` and dropped before it
  returns, and a forwarded method touches only its grid (no handle, global
  or user code). The one write that was a borrow of the cells across code
  (`grid_put_linebuf`'s copy) is `ScreenGrid::put_line_cells`, forwarded
  the same way. ~~`of_cell`~~: the three screen grids live in a `GridCell`,
  which has no `with`/`with_mut` — the only way to the grid is a handle, so
  there is no tracker loan to go around. ~~The null form~~: the handle is a
  `NonNull`, and the one place a null could enter (a window's `w_grid`
  before `win_grid_alloc`) is `try_grid_of_view`'s `None`; `grid_of_view`
  panics on it (always on: one compare per view resolved). Two callers had
  built a null handle they never dereferenced (a float's anchor offsets, a
  winbar scrolled off the top); they read the offsets without resolving.
  The constructor is private: the three that make a handle are in its
  module. Cost, scrbench (release, codegen-units = 1, cachegrind Ir):
  13.1482 G → 13.1497 G, +0.012 %.
- **Closed — `Term` (type holds it, checked):** ~~`DerefMut`~~ is gone, and
  `Deref` lends only `&Terminal`: every field that changes is a `Cell` or a
  `RefCell` (`terminal/state.rs`), `opts` is never written. Two copies in
  use at once share the terminal as two `&RefCell`s do; the emulator is
  `vt: RefCell<Box<VTerm>>`, so a vterm callback reaching `vt` through its
  own copy while `input_write` holds it panics instead of aliasing. That
  found a real one: vterm's output sink called the driver's write callback
  from inside the emulator call, and `nvim_open_term`'s `on_input` (Lua)
  can feed the terminal from there — two live `&mut VTerm` before. The
  sink now collects (`Terminal::output`, or the in-flight `TermRequest`'s
  hold, keeping its order) and `terminal::with_vt` sends once `vt` is let
  go. Not measured: no bench drives a terminal,
  and the borrow flags sit on per-chunk and per-row paths.
- **Closed — `Win::syn_block_mut` (type holds it):** ~~`syn_block_mut`~~
  is gone. Its twelve writes are in-place writers on `Win` —
  `set_syn_error` (`syntax/mod.rs`), and `set_spell_cjk`,
  `set_spell_langs`, `take_cap_prog`/`set_cap_prog` and the two midword
  writers (`spell/lang.rs`) — each storing through a borrow that ends
  before it returns and touching only the block (what an assignment drops
  is a language list or a compiled program: plain data). The
  `'spellcapcheck'` program a match runs is taken out and put back, so no
  borrow of the block is held across `vim_regexec`. What is left is the
  read half (`syn_block()`'s `&SynBlock` held across a write through
  another copy) and `Win::syntax()`, a `SynBlockRef` whose `DerefMut` is
  its owner handle's (the `Live<T>` row below). No instruction cost: the
  same stores.
- **Closed — `memline::held_line` (check-always):** ~~`held_line`~~ is
  gone. The matcher reads buffer lines through `HeldLines`
  (`memline/lines.rs`), which its match context owns: the slices it hands
  out borrow it, and while it lives the bytes stay put. Its first read
  writes a pending replacement back and pins the buffer's memfile; each
  line read out of a data block lends that block (a mark of the current
  pin). A lent block refuses `page_mut`/`pages_mut`/`resize_page`/
  `free_block` (a panic), `release_unlocked` keeps it, no replacement
  line can be cached (`cache_replacement`/`swap_cached_text` panic) and a
  pinned memfile panics on drop; the last unpin moves the epoch, which
  un-lends every block at once. So the buffer's _text_ cannot change under
  a running match — reading other lines (redraw, `Lines`, the matcher's
  own next line) still can. A search's line walk holds one pin for the
  walk (it changes nothing between lines); `:substitute` runs a match, and
  so a pin, per line. Never fired across functional (9,358) and oldtest
  (4,682). Cost (release, codegen-units = 1, cachegrind Ir): one compare
  per page write while nothing is pinned, and a pin flag test plus a mark
  store per line read — rebench 17.982 G → 18.045 G (+0.35 %), scrbench
  +0.19 %, edbench +0.30 %, mlbench +1.64 % (its `search` phase is
  120k-line sequential reads of a pattern that never matches: the
  per-line path and nothing else). The per-match context the copy
  alternative needed (+0.6 % rebench, measured earlier) is not built.
- **Closed — `IdTable` views (check-always, liveness):** the views a
  table hands out (the quickfix stacks' `Qi`, the normal-mode frames'
  `Op`) are no longer `Live<T>`. `IdTable` keeps its values in a
  `SlotPool` (`slot.rs`): each slot is allocated once and never given back
  to the allocator, and carries a generation word that moves on whenever
  its value goes. A view is a `SlotRef` — the slot's address and the
  generation its value was handed — and every dereference compares the
  two, so a view kept past its value panics instead of reading freed
  memory (the slot's memory is still a slot, whatever became of the
  value). Aliasing closed later (below, "`SlotRef`"). This is the floor the window, buffer and tab page handles move
  onto. The same check faked on the four graph handles (one load and a
  compare per dereference, release, codegen-units = 1, cachegrind Ir):
  mlbench +2.5 %, readbench +2.2 %, spellbench +2.1 %, scrbench +1.9 %,
  edbench +1.7 %, the rest ≤ 1 %.
- **Closed — the graph handles' liveness (check-always):** every
  `Window`, `Buffer` (the two unregistered ones too), `Tabpage` and
  `Frame` lives in a slot of its type's `SlotPool`
  (`winlayer/handles.rs`), and a free ends the value in its slot at the
  moment the `xfree` ran -- the destructor in place, the generation moved
  on -- with the deferred-free order unchanged. `Win`/`Buf`/`TabPage`/
  `FrameRef` carry the slot's generation beside their address and id
  (`Stamp`: one word, generation low, so a handle still passes in two
  registers and the compare needs no shift), and **every `Deref`/
  `DerefMut` compares it with the slot header** (`winlayer/deref.rs`: one
  load, one compare, a cold panic naming the re-entry rule and, through
  `#[track_caller]`, the line). A slot's memory is never given back, so a
  handle that outlived its object can no longer reach freed memory or the
  slot's next tenant. Equality is slot _and_ generation, and so is
  `is_current`. The list walks yield an element only while it is live and
  end at one freed under them (a body's user code can free the neighbour
  read ahead). Never fired across functional (9,361) and oldtest (4,684).
  The autocmd-wipe sweep (every event × `:bwipe!`/`:bunload!`/`:close`/
  `:hide`/`:tabclose`/`:only`/`:tabonly` of the current and of another
  object, the API closes, and the same from `nvim_buf_attach`, decoration
  provider, timer, `on_key`, `vim.ui_attach`, `'foldexpr'`,
  `'statusline'`/`'tabline'`, completion, `'operatorfunc'`,
  `'indentexpr'`/`'formatexpr'`/`'includeexpr'`, `'tagfunc'` and
  `'quickfixtextfunc'` callbacks) stopped at one class: `DiffUpdated`,
  fired from inside a recompute that `diff_check_with_linestatus`,
  `did_set_diff` and `diff_redraw`'s walk run while holding a window; it
  now fires with the window layout locked (E1312), and those callers
  re-check. (It also found `:tabclose` retrying a float that refuses to
  close for ever, an upstream loop.) Cost against the build before the
  check (release, codegen-units = 1, cachegrind Ir), after the memline's
  per-line paths took one borrow per walk: edbench +2.73 %, mlbench
  +2.43 %, readbench +2.37 %, spellbench +2.07 %, scrbench +1.93 %,
  markbench +1.41 %, undobench +1.25 %, rpcbench +0.91 %, evalbench
  +0.41 %, rebench +0.39 %, inbench +0.23 %, parser +0.09 %, mkspellbench
  −0.33 %. A buffer or window freed while an autocommand runs is refused
  from the moment its free path parks it, not when the outermost
  autocommand gives its memory back: the slot's generation moves on at the
  park and the slot stays off the vacant list until then (`slot::Parked`),
  so a handler further out that kept a handle to the torn-down object
  panics at its next dereference instead of reading what is left of it.
  Functional, oldtest and the sweep (debug and release) ran unchanged with
  it: nothing read a parked object. **Liveness is what closed; aliasing is
  not** for `Win`/`Buf`: two copies still lend overlapping `&mut` (the
  `edit()` slices below), and `GridRef` and `Term` carry no generation yet.
- **Closed — `Live<T>` (check-always):** ~~`Live<T>`~~ is deleted, with
  the promise it recorded. Its last two users: the command line, `Cc`, is
  now a place rather than an address (`ex_getln/line.rs`: the `CCLINE` cell,
  or an entry of the saved stack by index) and every access is one leaf
  borrow of that cell -- a getter and setter per scalar field, `with`/
  `with_mut` for the rest, a copy of the text where it was held across
  calls -- so two copies can no longer lend overlapping `&mut`s (the
  tracker panics), the `CCLINE.ptr()` escape hatch is gone, and a `Cc`
  kept past the saved line it named panics instead of reading a freed
  `Box`. The syntax block, `SynBlockRef`, holds the handle of the object
  that owns it (the buffer for its `b_s`, the window for an `:ownsyntax`
  block) and dereferences through it, so it is checked as that handle is;
  its aliasing is the owner handle's, closed with theirs. Cost, with the
  park above, against the build before both (release, codegen-units = 1,
  cachegrind Ir, two runs): scrbench +0.32..0.37 % (the syntax block's owner check),
  mkspellbench +0.5..0.6 % (its base moves 0.6 % between runs), every
  other bench within ±0.3 %.
- **Closed — `TabPage`, `FrameRef`, `PosRef` (type holds it, checked,
  both halves):** ~~`DerefMut`~~ and ~~`Deref`~~ are gone from all three.
  A `Copy` field is read by value and written in place by a generated
  getter/setter (`slot_fields!`, `winlayer/deref.rs`): a place access
  through the slot's raw address, which forms no reference to the object.
  A compound change goes through `TabPage::edit()`, a guard that sets the
  **edit bit**, the top bit of the slot's generation word (`slot::edit_at`),
  and puts the word back on drop or unwind. Every handle check compares the
  whole word with its generation _exactly_, so the one compare that asks
  "alive?" also asks "not lent out?": while an edit is held, a getter, a
  setter, a read, another edit or a free through any copy panics. A borrow
  of a field that is not `Copy` (`tp_diff`, `tp_winvar`, `tp_localdir`,
  `tp_prevdir`) goes through `TabPage::read()`, a guard the slot counts
  (`slot::read_at`, a reader count beside the generation): an edit, a
  setter or emptying the slot refuses while one is out, so the read half
  is closed too (a borrow held across code that writes the tab page through
  another copy panics at that write). `PosRef` is no longer an address into
  a window but the window's handle: `get`/`set`/`set_lnum`/… checked
  against the window's slot, and `modify(f)` for the `inc`/`decl` helpers
  (copy, step, store back), so no `&mut Pos` into a window outlives a
  statement. `with_diff` lends `tp_diff` under an edit; the diff code
  already copied the slot table out first, and nothing reached the tab
  page from inside. ~520 sites, rewritten from the compiler's errors; none
  fired across functional (9,365) and oldtest (4,682). The same `edit()`/
  `read()` (`guarded_handles!`) serve `Win` and `Buf` when they give up
  `DerefMut`. `SynBlockRef`'s aliasing rides on its owner handle (`Buf`/
  `Win`), so it closes with theirs. Cost (release, codegen-units = 1,
  cachegrind Ir) against the build before: scrbench +0.70 % (a
  register-allocation shift inside `win_line`, where the checks of
  `Win`/`Buf` dereferences moved; no check was added there), rpcbench
  +0.44 %, undobench +0.40 %, mlbench +0.39 %, evalbench +0.34 %,
  readbench +0.26 %, the rest ≤ 0.2 %. The reader count is not asked
  when a pool lends its own `&mut` (`get_mut`: the condition stacks take
  it per command line, +0.15 % evalbench when it was).
- **Decided — the read half for `Win`/`Buf` (Q3):** a transient field read
  through `Deref` is already refused while an edit is out (the exact
  compare); what stays open is a `&Window`/`&Buffer` _held_ past its
  expression across another copy's edit. Closing it means what closed it
  above: `Deref` deleted, a getter per `Copy` field (no runtime cost: the
  same compare and load) and a counted `read()` for every borrow. Sites
  (grep, `crates/nvim/src`): ~6,200 `.w_*` and ~3,200 `.b_*` accesses, of
  which ~3,900 are nested projections (`w_cursor.lnum`,
  `w_onebuf_opt.wo_*`, `b_ml.*`: a getter per leaf, or per parent
  returning a copy), ~810 method calls on a field and ~130 `&x.w_*`/
  `&x.b_*` borrows — ~950 sites that need a counted `read()` (two loads
  and two stores each, several on hot paths: option strings in the
  drawer, `b_ml` in memline walks). Recommendation: do it in the `Win`/
  `Buf` slices together with `DerefMut`'s removal (the same sweep visits
  every site), counted reads only where a borrow outlives its statement,
  and budget it like S6b's read count (evalbench +1.8 % there):
  expect ≤ ~1 % on ml/scr/ed once hot walks take one guard per walk.
- **Closed — `Buf` (both halves, checked):** ~~`Deref`~~ and ~~`DerefMut`~~
  are gone. Each `Copy` field has a generated getter and setter
  (`winlayer/deref/buf_fields.rs`, `slot_fields!`: one compare, a place
  access through the raw address); every other field goes through
  `Buf::read()` / `Buf::edit()` (`guarded_handles!`), and accessors that
  lent a part of the buffer (`marktree()`, `extmark_ns()`, `header()`,
  `header_mut()`, the update subscribers) return `slot::ReadPart` /
  `EditPart`, a guard narrowed to the part. The guards take the handle by
  value: the slot's own count keeps the buffer in place, so a guard may
  outlive the handle it came from. The memline is a cell of its own,
  `b_ml: UnsafeCell<MemLine>`, lent for a walk by `Buf::ml()`
  (`slot::lend_part_at`: one reader of the buffer plus the slot's part
  bit, in one word): a line read fills the line cache, and must not be
  refused by a read of another field held across it.
  `line_count()`/`ml_flags()` refuse while a walk holds it; the
  line text `Lines`/`HeldLines` lend is the line cache's loan (S5b), formed
  from the raw place by `Buf::lend_cached_line*`. Hot walks (`fetch_line`,
  `ml_find_line`, `ml_append_int`, `update_line_chunks`,
  `find_line_or_offset`) take one `ml()` per stretch and end it before a
  call handed the buffer or a message. The suites found the guards held
  across user code: an autocommand fired with a read in its arguments
  (`FileType`, `Syntax`, `SwapExists`, …), `if let` scrutinees, update
  callbacks visited under the channel list's edit; each now copies out
  first.
- **Closed — `Win` (both halves, checked):** ~~`Deref`~~ and ~~`DerefMut`~~
  are gone. Each `Copy` field of `Window` has a generated getter and
  setter (`winlayer/deref/win_fields.rs`), and so does each `Copy`
  window-local option of the window's own set (`wo_*`, the leaves of
  `w_onebuf_opt`) and each `Copy` leaf of a field that is not `Copy`
  itself (the float's `w_config`, the saved view `w_save_cursor`, the
  'listchars'/'fillchars' glyphs); the cursor is `PosRef`; every other
  field goes through `Win::read()`/`Win::edit()`, with `Win::config()` and
  `Win::wline()` copying out the two non-`Copy` values read most. ~6,200
  sites, rewritten from the compiler's errors (scripted, then by hand
  where a guard would outlive its statement). What the hand pass and the
  suites found were guards held across another access to the same window,
  each a borrow the old `Deref` lent unchecked: `w_lines[a] =
w_lines[b].clone()` (the right side's read temporary lives to the end of
  the statement, where the left side's edit refuses it), a
  `let wl = &mut win.edit().w_lines[i]` held over a later read of the
  window, `&mut win.w_cursor` lent to a callee that reads the window
  (now `PosRef::modify`, a copy in and out), a jumplist entry borrowed
  across `fname2fnum` (whose autocommands edit every window's jumplist),
  `w:` dict handles taken in an `if let` head whose body runs watchers or
  prints, and `:tags`/`gettagstack()` printing under the tag stack's
  borrow (now a copy). The ratchet's `place_writes` and
  `accessor_temporary_mutations` read the new getters from `slot_fields!`
  like `Buf`'s (a planted `&mut win.w_topline()`, `win.w_cursor().lnum =`
  and `win.w_alt_fnum().take()` each failed it). Cost (release,
  codegen-units = 1, cachegrind Ir) against the build before: every bench
  within ±0.5 % (edbench +0.43 %, mkspellbench +0.48 %, scrbench +0.12 %).
- **Closed — `SynBlockRef` (both halves, checked):** its `read()` is the
  owner's counted `read()` and its `edit()` the owner's `edit()` (the
  buffer for its `b_s`, the window for an `:ownsyntax` block), each
  narrowed to the block (`winlayer/deref.rs`, `SynRead`/`SynEdit`), so a
  borrow of a block held across another way to it panics there. What made
  the engine hold an uncounted borrow -- a pattern's program taken out for
  a match while the pattern list was borrowed -- is gone: `sp_prog` and
  `b_syn_linecont_prog` are take/put cells (`syntax::ProgCell`), their
  `:syntime` totals `Cell`s, so a match needs only a read of the block.
  `Win::syn_block`, which lent `&SynBlock` through `w_s` with no check at
  all (the spell checker's way in), is gone too: those reads go through
  `Win::syntax().read()`. The suites found two borrows the counted guards
  refuse, both reads of the block held across the matcher installing the
  syntax's `iskeyword` table in the buffer (a write of the buffer):
  `push_next_match` and the zero-width-nextgroup test now copy what they
  need out of the pattern first, and the state cache walk in
  `record_line` scopes its borrow before storing a state. The spell
  checker's per-character `MIDWORD`/`cjk` questions read their two `Copy`
  fields in place (`Win::spell_ismw`, `Win::spell_cjk`: the owner checked,
  no count), and `current_attr` asks the block's three flags once per call
  rather than per loop. Cost (release, codegen-units = 1, cachegrind Ir)
  against the build before both rows: spellbench +1.25 % (the owner check
  in `spell_iswordp`), scrbench +0.65 % (most of it `win_line` register
  allocation again, the rest the counted reads in the syntax engine),
  edbench +0.44 %, every other bench within ±0.35 %.
- **Closed — `GridRef` (both halves, checked):** a window's own grid
  carries the window's slot generation and every access compares it with
  the slot's word, as a `Win` does, so a grid handle kept past its window,
  or used while an `edit()` of the window is out, panics; the three static
  grids (`GridCell`) have no owner. ~~`Deref`~~ is gone: a read is a
  by-value accessor (`grid_reads!`, a getter per field), a borrow that
  never leaves `winlayer/grid.rs`. The one borrow held across a call --
  the cells the UI layer and the compositor read -- is
  `GridRef::with_cells`, counted in the grid (`ScreenGrid::lent`): every
  write through any copy refuses while it is out, and so does an `edit()`
  of the owning window (the loan holds a counted read of it). The window's
  grid is an `OwnedGrid` (an `UnsafeCell` with no accessors), reachable
  only as `Win::grid_alloc()`, so a `read()` of the window and a grid write
  coexist. A view stores the checked handle itself (`GridView::target:
Option<GridRef>`). Never fired across functional and oldtest. Cost
  (release, codegen-units = 1, cachegrind Ir): scrbench +0.17 %, edbench
  and mlbench flat.
- **Closed — `Term` (liveness, checked):** terminals live in a slot pool
  (`TERMINAL_SLOTS`); `Term` is a slot handle carrying the generation, so
  a copy used after `Term::free` -- a read or a second free -- panics.
  ~~`Deref`~~ is gone: `Term::read()` is a counted guard and the free
  refuses while one is out, so no `&Terminal` outlives the terminal.
  `Buffer::terminal` stores the handle. No bench drives a terminal.
- **Closed — `SlotRef` (the `IdTable` views `Qi`, `Qfl`, `Op`; aliasing,
  checked):** ~~`Deref`/`DerefMut`~~ are gone; a view's value is borrowed
  only through `SlotRef::read()`/`edit()`, the counted guards above, with
  by-value getters for the `Copy` fields (each read let go before the
  getter returns). A table either hands out views (`ViewTable`, no
  `get_mut`) or lends through `&mut` of itself (`IdTable::get_mut`, no
  views), so the pool's uncounted borrow never meets a guard;
  `SlotPool::get_mut` is private to `slot/`. The suites found one borrow
  the old `DerefMut` lent unchecked across user code: a quickfix entry
  borrowed across the jump's `BufEnter`, which `Test_lhelpgrep_autocmd`
  answers by freeing the lists.

- **Breaks — aliasing:** two copies give two live `&mut`
  ~~(`let c = &mut w1.w_cursor; w2.w_cursor.lnum = 1; c.col = 0`)~~;
  ~~`syn_block_mut()` held while the buffer's block is reached through a
  `Buf`~~;
  ~~`GRID.with(|g| { grid_ref.cols = 0; g.cols })`~~; ~~`held_line`'s slice
  outliving the cache when a copy of the `Buf` reads another line~~.
- ~~**Breaks — liveness:**~~ (graph handles closed above) a handle held
  across a call that runs user code (an autocommand, a Lua callback — the
  p45 hazard — Vimscript) and dereferenced after `:bwipeout`/`:close` freed
  the object: now a panic at the dereference, not a use-after-free, for
  every handle (`GridRef` through its window, `Term` through its slot).
- **Closure:** restructure. Liveness: a handle is an id plus a generation
  checked against a side table on dereference (the object's own memory
  cannot carry it — it may be freed); a check-always that wants its own
  measurement. Aliasing: no `DerefMut` on a `Copy` type — reads by `&`
  through a checked handle, writes through a scoped, tracked `with_mut`
  (S1's tracker keyed by object). Order: ~~`GridRef` + `of_cell` (S)~~, ~~`Term`
  (S)~~, ~~`syn_block_mut` (13 sites, S)~~, ~~`held_line` (the matcher's line
  cursor, M)~~, ~~`Live<T>` users (M)~~, ~~`TabPage`/`FrameRef`/`PosRef`
  (M)~~, then ~~`Win`/`Buf`~~ (XL; 553 files name them). **XL** overall.

### ~~S6 — value-handle edits~~ (closed: check-always, and a type for partials)

`ListRef`/`DictRef`/`BlobRef`/`PartialRef`: `edit(&self) -> Edit<'_, T>`,
`DerefMut`, `Deref`, and the `&Dict` walks (`items`, `items_mut`) through
them.

- ~~**S6a — write/write:**~~ (closed: check-always) two live `edit()`s of
  one object through two handles, or a `DerefMut` under an `edit()`. Every
  build now panics where it happens: each object carries a `Lent` flag (a
  `Cell<bool>` field beside its lock, in padding for `List`/`Dict`/`Partial`;
  `Blob` grows 32 → 40 bytes, the same allocator size class), `edit()`
  raises it and the `Edit`'s loan lowers it on drop; `DerefMut` tests it.
  The loan reaches the flag through the handle's pointer projected to the
  field (`offset_of!`), never a borrow of the object, which the edit holds
  `&mut` (Miri: `edit::tests` under `just miri-perimeter`). The table it
  replaces (`EditTable`, a thread-local `Vec` of addresses, debug builds
  only) is gone. Cost, release, codegen-units = 1, cachegrind Ir against
  the build without it (three runs a side for evalbench and spellbench):
  evalbench +0.74 % (parser phase +0.35 %), undobench +0.74 %, edbench
  +0.24 %, rpcbench +0.15 %, markbench +0.09 %, scrbench +0.06 %,
  mlbench/rebench/inbench/spellbench within ±0.05 %. No hot site needed
  hoisting. `soundness_checks` now switches only
  `GlobalCell`'s per-access thread check.
- ~~**S6b — read/write:**~~ (closed: check-always) a `&List` from one
  handle's `Deref` live while a clone's `edit()` reallocates
  (`for item in l.items() { other.edit()… }`), or an `edit()` held across a
  call that runs user code which reads the object. `ListRef`, `DictRef` and
  `BlobRef` lost `Deref`/`DerefMut`: an object is borrowed only through
  `read()` (a `Read`, one count in the `Lent` word, now a `Cell<u32>`:
  0 free, `u32::MAX` an edit, else the count of reads) and `edit()`, as a
  `RefCell` counts its borrows, every build. A partial is fixed once built
  (a type: `Deref` only, `PartialRef::get_mut` for its last holder, its copy
  mark a `Cell`). The compiler holds the holder's own half: `Read` has a
  destructor and an `Edit`'s loan carries the handle's lifetime, so a
  `&mut` of the holder or a move of the handle under a live guard does not
  compile (it found 17 edits still open when their handle was moved into a
  result or a provider call, and a `d.key` lookup clearing its result under
  the read).
  - **Read sites that run user code**, surveyed by converting every one:
    `filter()`/`map()`/`mapnew()`/`foreach()`, `sort()`/`uniq()` with a
    function, `indexof()`, `reduce()`, `:for`, `call()`, `complete()`
    lists, `matchfuzzy()` with a `key`/`text_cb`, `getreg()`/`setreg()`
    lists, dict watchers, the collector's marking (it writes the mark of
    the container it walks) and the deep copy. Each reads per step and
    clones the item out before the call; the deep copy holds its reads
    across the copy because its only write, the copy memo, lands before a
    container is first read. A walker that hands out plain references for
    its whole run (the encoders, job/`call()` options) keeps its shares in a
    `ReadScope`.
  - **Messages under a read**: a message writes `v:errmsg` into `v:`, and a
    caller may pass `v:` where a dictionary is read (`json_encode([v:, 1])`,
    `mapset(v:)`, `has_key(v:, v:)`, `foreach(v:, ..)`,
    `extend({'count': 1}, v:, 'error')`). The message rows (`v:errmsg`,
    `v:warningmsg`, `v:statusmsg`, `v:scrollstart`) are written only while
    `v:` is free (`DictRef::is_free`, the `Lent` word read without a
    borrow); under any read of `v:` -- a plain `Read` or a `ReadScope` --
    they wait in a queue and land as soon as `v:` is free again: every way
    to `v:` (its handle, `vimvar_dict()`; a message row's read or write)
    and every scope's end writes them first. Only the read's own holder can
    see the old row meanwhile, which is what a read promises it anyway.
    Reading a `v:` row is a shared read. Found by passing `v:` to every
    builtin (a local sweep: `v:`/`b:` at each argument position, nested,
    and beside a dictionary sharing a `v:` key);
    `test/functional/vimscript/vvars_spec.lua` keeps the cases.
  - **Methods that call out under `&mut self`**: an `impl Dict`/`List`/
    `Blob` method holding the object exclusively must not put out a
    message nor run anything that can reach the object (the evaluator, a
    variable lookup, an autocommand, a watcher, a callback). Audited
    tree-wide (every `&mut self` method, `&mut Dict`/`&mut List` parameter
    and `edit()` binding live across such a call): `Dict::add_item` still
    ran the Funcref name check (a lookup in the very scope) -- now an
    invariant, with `dict_add_item` the only way a Funcref reaches `g:`/`l:`;
    `extend_from_self` reported under `&mut self` -- now `&self`;
    `remove()` on a list or blob checked its lock and converted `args`
    under the edit -- now it checks through the handle, then edits once.
    The `&mut Dict`/`&mut List` parameters left that put out messages fill
    a fresh result nobody else holds (`msgpackparse()`, `wordcount()`,
    `taglist()`, swap-file recovery). `just ratchet --check` refuses a
    message or call-out inside a value object's `&mut self` method
    (`CONTAINER_CALLOUT`); a free function holding an `Edit` is not
    checked mechanically. A `vim.ui_attach` `msg_show` handler is the one
    callback a message could run; for internal messages it runs in
    `api-fast` context, where it cannot reach a variable (a probe under
    `extend({..}, g:, 'error')` saw it not run during the call at all).
  - **Cost**, release, codegen-units = 1, cachegrind Ir against the build
    before the partial/blob/list/dict changes (three runs a side for
    evalbench): evalbench +1.8 % (+1.72..+2.06 %; parser phase +1.18 %),
    rpcbench +1.65 %, edbench +0.39 %, scrbench +0.28 %, spellbench
    +0.21 %, markbench +0.18 %, inbench +0.15 %, rebench +0.05 %, mlbench
    −0.01 %, undobench −0.30 %. The evalbench delta sits in variable lookup
    and assignment (`locate`, `eval_variable`, `set_var_const`: a read of
    the scope dictionary per access) and the call path; rpcbench's is
    mostly a codegen shift between `Object`'s drop and `release`, with
    ~0.2 % from the encoders' `ReadScope`. The list/dict commit alone
    measured evalbench +1.3 % (parser +1.02 %); the rest is the message-row
    hold (since replaced by the queue) and the `Edit` pointer.
  - **Miri** found the `Edit` of the write half unsound once a test moved
    one into a call (`drop(edit)`): a `&mut T` field is retagged protected
    for the call and the loan's release writes the word inside the object
    under it. The edit now holds its object by address (`NonNull`), its
    `Deref`/`DerefMut` in the handle file.
  - **The battery** found two more reads under an edit that no suite
    reaches: `:lockvar!` over a self-referencing container (the item's
    shared check read the container whose edit held the item's lock) and a
    Funcref added to `g:`/`l:` by key (the name check looked the name up in
    that scope under its edit). Both now run their read first.

### ~~S7 — the Lua stack's consuming operations~~ (closed: check-always)

`Stack::{concat, set_field, get_table, set_table, raw_get, raw_set,
raw_set_index, set_metatable, next}`, `debug::function_info`,
`table::reference`, `userdata::set_environment`: each pops values it assumes
its view pushed.

- **Breaks:** an operation on a view that pushed too few values pops a slot
  of the view below it, or below the C frame's base — LuaJIT then reads and
  writes outside the frame.
- **Closed:** the compare of `top - base` is an `assert!` in every build
  (`soundness_check!` in `lua/stack/mod.rs` no longer looks at the cfg).
  Cost, release, codegen-units = 1, cachegrind Ir against the build without
  it: evalbench +0.25 % (three runs a side, inside its 0.36 % run-to-run
  spread; parser phase +0.00 %), markbench +0.68 %, rpcbench +0.31 %. A
  pushed-count token would have threaded a value through every caller for
  the same compare.

### ~~S8 — event handles with `DerefMut` over pinned libuv objects~~ (closed: type holds it)

`EventLoop`, `Reader`, `Conn`, and the file-private `Uv`, `Child`,
`Signal`, socket `Watcher`, timer `Handle`: `Copy` wrappers of a raw pointer
to an object libuv holds by address (`Stream` carries `PhantomPinned`).

- **Broke:** `mem::swap(&mut *a, &mut *b)` over two `Reader`s moved two
  registered `RStream`s — the transports swap hole the pinned
  `Channel::stream` box closed, reopened one level down. A listening
  socket's connection callback was handed `&mut SocketWatcher` (an `Unpin`
  type with `pub` fields), so `mem::replace` from the callback overwrote the
  handles libuv was listening on.
- **Closed:** no handle is `DerefMut`. Reads go through `Deref`; a write is a
  generated field writer (`field_writers!` in `event/stream.rs`) that stores
  one field through the wrapped pointer, so no `&mut` to the object is ever
  made and nothing can move it. `EventLoop`, `Reader`, `Conn` and their
  methods are `pub(super)`: no code outside `event/` can hold one. The
  connection callback takes an `Incoming` that can accept the client and
  read the address, nothing else. A handle used after its close callback
  freed the object stays the construction contract (`unsafe fn new`),
  perimeter-internal.

### ~~S9 — a safe function adopting a raw pointer~~ (closed: `unsafe fn`)

`treesitter::push_tree(stack, *mut TSTree)` takes ownership: the userdatum's
`Drop` deletes the tree. It is an `unsafe fn` whose contract is "null, or a
tree the caller owns and hands over"; its three callers pass a tree fresh
from `ts_tree_copy` or a parse.

### ~~S10 — value bit copies~~ (closed: type holds it)

Found by the closing audit. Safe exports of `eval/typval/` let a caller
hold a bit copy of a value it does not own, or a handle with no reference:

- `TypVal::named_for_call(&self) -> TypVal`: the `a:` and `a:000` items
  named the caller's arguments; `cleanup_function_call` disowned them (or
  upgraded them to copies when the scope outlived the call). Dropping the
  result released the caller's payload a second time; keeping it past the
  original read freed memory. The `a:000` entry in `a:` named the funccall's
  own list the same way, uncounted.
- `CallFrame::push_borrowed(&mut self, &TypVal)` took no lifetime, so a
  frame could outlive the value it named: `filter()`/`map()`/`indexof()`
  pushed `v:key`/`v:val` from inside `with_vim_var`'s closure, and the
  `v:testing` argument stack (`FUNCARGS`) kept bit copies of every call's
  arguments in a global.
- `CallFrame::own(idx)` on a named slot made the frame release a value it
  did not own (`reduce()` used it to take the accumulator over).
- `registered_dict`/`registered_list` answered a `ManuallyDrop` handle
  owning no reference: `ManuallyDrop::into_inner` turned it into one that
  releases, and nothing bounded it by the registry slot's life.

**Closed:**

- `named_for_call` is gone: an `a:`/`a:000` item is a copy of its own
  (`Clone`: a container takes a reference, a string is duplicated), the
  `a:000` entry counts its reference, and the disown and upgrade paths
  (`List::disown_items`/`own_items`, the forgetting `clear_scope`) are
  deleted.
- `CallFrame<'a, N>`: every named slot borrows its value for `'a`
  (`push_borrowed(&mut self, &'a TypVal)`), so the frame cannot outlive
  it; `own` is deleted and the only push that takes ownership is
  `push_owned`. The `v:` callers copy the value in; `FUNCARGS` holds
  `CallFrame<'static, _>` of copies. A method call's base is a shared
  borrow (`CallWith::basetv: Option<&TypVal>`), which the frame's lifetime
  now required. `reduce()` moves (or, for a Blob, copies) the accumulator
  into the frame.
- The collector's views are ordinary handles (`registered_dict` retains),
  and `free_registered_dict(idx)` frees only when the view's reference is
  the last one — an always-on check: a holder the marking never saw keeps
  an emptied container instead of a dangling one.
- `TypVal::bit_copy` is private to `handle.rs`; its one caller is
  `push_borrowed`.

Cost (cgu1 release, cachegrind Ir against the open state): evalbench at
+0.11 %, its parser phase at +0.19 %, rpcbench at −0.06 %.

### ~~S11 — container header words through `&mut`~~ (closed: type holds it)

Found by S10's sweep. A List, Dict or Blob carried its reference count and
its `Lent` word inline, and `Edit` lent `&mut List` — the whole struct,
header included:

- **Broke:** `mem::swap(&mut *a.edit(), &mut *b.edit())` swapped two
  objects' counts (A's last handle then freed it under B's holders);
  `mem::replace(&mut *l.edit(), List::empty())` wrote a count of 0 and a
  free `Lent` word under a live edit. The count's mutators were `pub` over
  `&self` (`l.read().lv_refcount.release()`), the scopes seeded
  `DO_NOT_FREE_CNT` by assigning the field, and the registry slot
  (`lv_root`) and the deep copy's memo (`lv_copylist`) sat in the same lent
  region — a swapped root unregisters the wrong container and leaves the
  registry naming a freed one.
- **Closed:** a handle points at a `Counted<T, H>` (`eval/typval/handle.rs`,
  `Rc`'s shape): the count, the `Lent` word and a per-kind head (`Node`:
  the registry slot and the memo, for lists and dictionaries) beside the
  value. `read()`/`edit()` lend the value only and every header field is
  `handle.rs`'s, so a swap through two edits moves two values and each
  object keeps its count and word (`dict.rs`'s
  `swapping_two_edited_bodies_moves_values_not_counts`). `PartialRef::
get_mut` lends the value without its count the same way. Counts are read
  through `refcount()`; `Refcount::release_many` is gone and no
  container's `Refcount` is reachable outside `handle.rs`. The scope seeds
  are counted tokens (`DictSeed`/`ListSeed`: a handle plus the rest, given
  back on drop) — a scope entry holds its own, a funccall holds its three
  while a call runs. The memo pairs a `CopyWalk`'s id with the copy, and
  the walk holds a reference to every copy until it ends (64-bit ids, never
  reused); the registry is entered and left only with `handle.rs`'s
  `RootKey`. A `forbid(unsafe_code)` probe in `tests/unit` reaching the
  count through a `Read`/`Edit` (`release()`, a swap of two counts, a take
  of the `Lent` word) no longer compiles (E0609); S10's sweep had it
  compiling. (The remaining `Refcount`s — channels, argument lists,
  quickfix stacks, child processes, user functions — free through id tables
  or `Rc`, so a wrong count there is a lifetime error the table catches.)

### ~~S12 — the dictionary item store's raw slots~~ (closed: type holds it)

Found by S10's sweep. `DictEntry` (`handle.rs`) was a `Copy` raw pointer,
`HashTab`'s slot API (`slot`, `insert_at`, `slots_mut`) was safe and `pub`
over a `pub` `dv_hashtab`, and which slots owned their item was
`DI_FLAGS_ALLOC` in the item's `pub` `di_flags`.

- **Broke:** copying a slot's entry into a second dictionary's table made
  two tables free one `Box`; setting `DI_FLAGS_ALLOC` on `b:changedtick`
  (the one embedded item), or moving `b:`'s table into `g:` through a swap
  of two edited bodies, made `take_item` `Box::from_raw` the buffer's own
  bytes or outlive the buffer.
- **Closed:** `b:changedtick` is an ordinary item `b:` owns (the counter is
  the buffer's `b_changedtick`, written into the item by
  `buf_set_changedtick`), so no table names an item it does not own. The
  store is `eval/typval/store.rs`: `DictTab` wraps the hash table, its
  entry type is private, and every kept slot owns a box — items go in as
  boxes (`insert`), come out as boxes (`remove`) or with the table (`Drop`).
  No slot is reachable outside the file, so a table moved, swapped or
  replaced whole keeps one owner per item. `DI_FLAGS_ALLOC` and
  `RemovedItem` are gone. A probe copying an entry between two tables or
  setting the flag no longer compiles.

## The swap audit

S11 was a hole the survey's classing could not see: it classed handle
methods, not what a guard lends. So every guard or handle that lends a
`&mut T` was asked what swapping or replacing the whole `T`, or a lent
field, can break — header or bookkeeping words inside the lent region,
self-handles, owner generations, intrusive links, and anything unsafe code
trusts. Only perimeter code can turn a wrong value into undefined
behaviour, so each answer is about what the perimeter reads back out of the
lent region.

- **`SlotEdit`/`EditPart` of a `Window`, `Buffer`, `Tabpage`, `Frame`:**
  safe — the slot's generation word and reader count are outside the value
  (`slot.rs`); every handle stored inside (`b_next`, `w_buffer`, frame
  links, the API number `handle`) is checked against its slot's generation
  at each dereference, so a swapped one is a wrong answer, never a dangling
  one; `w_s` is compared, never dereferenced (`Win::syntax` asserts it is
  the buffer's block or the window's own).
- **`OwnedGrid` (inside a window):** safe — its `lent` count is raised only
  under `GridRef::with_cells`, which holds a counted read of the owning
  window, so no edit (and no swap) is out while it is up; a `GridRef`
  carries its owner's generation outside the grid.
- **`Term`:** safe — `Deref` lends `&Terminal` only; the emulator in its
  `RefCell` reports through its callbacks' `Term` handle, checked.
- **`GridRef`:** lends no `&mut` (in-place writers only).
- **The `SlotRef` tables (`Qi`, `Qfl` views, `Op`):** safe — generation
  outside the value; a quickfix stack's `refcount` frees through its table,
  so a swapped count frees a slot whose next access panics.
- **`GlobalCell::with_mut`:** safe — `set`/`replace` already write any
  value, so a cell's contents carry no invariant a swap could break, and
  the cells whose contents unsafe code trusts (the raw-pointer cells, the
  libuv-held statics) are private to their perimeter files
  (`cell_raw_ptr`/`cell_ptr`).
- **The value `Edit`s and `PartialRef::get_mut`:** safe since S11/S12 — the
  lent value holds no count, borrow word, registry slot or memo, and the
  dictionary's table owns its items; what is left inside (`lv_watch`
  cursors, the collector's mark, `lua_table_ref`, `dv_scope`) is logic,
  whose wrong value is a wrong answer.
- **The memline part loan (`Buf::ml`) and `HeldLines`:** found a hole,
  closed (check-always): two loans (or two buffer edits) could swap two
  buffers' memlines under a `HeldLines`, which then read out of a memfile
  nobody pinned and unpinned another's. The hold now remembers the pinned
  memfile's address — a pinned memfile cannot be dropped, so the address is
  its alone — and every read and the unpin compare it (`memline/lines.rs`).
  The loan itself is otherwise safe to swap: the part bit is in the slot,
  and the memfile's pins and lend epoch move with it.
- **The line cache's slices:** found a hole, closed (check-always) —
  S13: a lent line is counted in its memfile, and every walk refuses.
- **The TUI's state (`&mut TUIData`):** found a hole, closed (type) —
  S14: the libuv handles are out of the lent region.

## The lifetime sweep

After S13, every perimeter function was asked whether it answers a
reference whose lifetime the caller picks: `fn f<'a>(…) -> &'a …` with
`'a` tied to no input, a `&'static` from a source that is not, or a
`from_raw_parts`/`CStr::from_ptr` in a safe function.

- **The line cache's lenders** (`lend_cached_line*`): S13, gone.
- **`HeldLines`' pinned lender**: closed with S13 — the slice is borrowed
  from a `MemPin`, which only a pin makes.
- **Private helpers that lent for any lifetime** — `Buf::ml_unlent`/
  `ml_unlent_mut` (`winlayer/deref.rs`), `Child::pty` (`event/proc.rs`,
  a `&mut PtyProc` from a `Copy` handle), `Watcher::addr`
  (`event/socket/mod.rs`) and `loans` (`global_cell.rs`, the borrow
  chain's walk): each is an `unsafe fn` with its contract now, every call
  site a `SAFETY` comment (the first three are rows of
  [unsafe-fn-allowlist.md](unsafe-fn-allowlist.md)).
- **Sound as they stand:** the slot guards' phantom lifetimes and
  `header`/`readers` (a slot's memory is never given back); a tree-sitter
  language's names, supertypes and fields (`&'static`: a language's
  library is never closed once registered); `os_strerror`/`os_err_name`,
  `lua_typename`, `gettext` (static or leaked strings); every
  `from_raw_parts`/`CStr::from_ptr` in a safe function either copies out
  before returning or answers a borrow of `&self`/`&mut self` over memory
  the receiver owns (`ThinCString`, `Directory`'s entry, the Lua stack's
  strings — S7's ground); the rest are `unsafe fn`s taking the raw
  pointer.

### ~~S13 — the line cache's slices~~ (closed: check-always)

Found by the swap audit. `Buf::lend_cached_line*` (`winlayer/deref.rs`)
answered `&'a [u8]` of the buffer's one-line cache for any `'a`, and
`Lines::line` (`memline/lines.rs`) bounded that by its own `&mut self` —
but nothing stopped a second reader filling the same buffer's cache
meanwhile (the `Lines` docs left "the borrow ends before the call" to the
caller).

- **Broke:** `let x = a.line(1); let _ = b.line(2); x.len()` with two
  `Lines` of one buffer (or any `ml_*` call or redraw between) refilled the
  cache under `x`, freeing a pending replacement's text or moving to
  another block. A `forbid(unsafe_code)` probe compiled. The suites found
  three real holders of a line across a refill: `measure_join` across the
  seam's extmark splice (which reads other lines' offsets), the statusline
  across its `%{}` expansion, and `coladvance2` across the `'virtualedit'`
  padding's `ml_replace`.
- **Closed:** `Lines::line`/`line_cstr` answer a guard, `LineText`
  (`winlayer/deref.rs`; `Deref` to `[u8]`/`CStr`, `map` to narrow it).
  While one lives the memfile counts it out (`MemFile::lend_text`), and
  `Buf::ml` — every walk: a fill of another line, every `ml_*` change, a
  close — panics; the memfile's `Drop` panics too. Reads of the line the
  cache already holds take no walk (`Buf::lend_line_if_cached`), so two
  readers of one line, the line count and the rest of the buffer stay
  welcome. The count is in the memfile's box rather than the buffer's
  slot because a buffer's whole value can be swapped under two edits:
  the count goes wherever the lent text goes. For the same reason
  `Buffer::b_ml` is private — an edit of a buffer reaches no memline, so
  edits stay welcome while a line is out. `line_mut` answers
  `LineTextMut`, which holds the walk itself. A guard lives to the end of
  its scope rather than its last use, so the readers that call out after
  reading drop it first (`drop(line)`, about eighty sites, placed from
  clippy's `significant_drop_tightening` and a scan of holders across
  calls). The `lend_cached_line*` family is gone. Its pinned sibling
  (`HeldLines`) lends under a `MemPin` token now: `MemFile::pin` answers
  it, `unpin` consumes it, and `lend_page`/`Buf::lend_pinned_line` borrow
  the slice from it and check it is the memfile's own (an id, which a
  swap of two memfiles' values carries along — the address `HeldLines`
  compared before did not). `tests/unit/memline.rs`
  `a_second_reader_refilling_under_a_lent_line_panics` is the probe, now
  a panic. Debug builds name where each line still out was lent.
- **Cost** (cgu1 Ir vs `6191ecb101`): ed +0.58 %, ml +0.54 %, read
  +0.63 %, re +0.05 %, scr +0.13 %. Measured against two other designs:
  a per-reader memfile pin (`HeldLines`' mechanism for every `Lines`,
  prototyped without the copy-on-write a walk that edits between reads
  would need): ed +3.58 %, scr +0.78 %, the rest ≤ 0.12 %; and a copy of
  every line into a pooled per-reader buffer: ed +7.07 %, scr +0.36 %, the
  rest ≤ 0.12 %.

### ~~S14 — the TUI's libuv handles under `&mut TUIData`~~ (closed: type holds it)

Found by the swap audit. `TUIData` (`tui/state.rs`) held handles libuv
keeps by address — `write_loop`, `output_handle`, `winch_handle`,
`startup_delay_timer`, and the input's `timer_handle`, `bg_query_timer` and
`read_stream` — as `pub` fields, and every painting module takes
`&mut TUIData` (S8 closed the same shape in `event/`, not here).

- **Broke:** `t.startup_delay_timer.loop_0 = ptr::null_mut()` or
  `mem::swap(&mut t.input.timer_handle, &mut t.input.bg_query_timer)` in
  any function handed `&mut TUIData` rewrote memory libuv's loop links
  through. A `forbid(unsafe_code)` probe compiled.
- **Closed:** the handles left the lent region for `tui::tui::Handles`,
  which `TUIData` holds as `Pin<Box<Handles>>`: private fields (each an
  `UnsafeCell`, as libuv writes them while the state is borrowed),
  `!Unpin`, built only by `TUIData::new`, no `Default`/`Clone` — so safe
  code can neither reach a handle, lend `&mut Handles`, nor swap or replace
  the box (there is no second value). It lends raw pointers, for the four
  libuv files' `unsafe` calls. Which output handle is open (tty, pipe,
  none) is its own private record, so a painter flipping `out_isatty` can
  no longer send a write or a mode change through a handle libuv never
  initialised; a flush with no output open drops what was staged. The
  probe's field write and swap, and `Pin::get_mut`, a field read and
  `Handles::new()` against the new shape, all fail to compile (E0609,
  E0277, E0616, E0624).

### ~~S15 — the emergency abandons~~ (closed: a type, the `Emergency` token)

Found by the exit re-audit. The emergency exits forget every loan as if
it had been given back — `global_cell::abandon_borrows`,
`SlotPool::abandon_loans` (windows, buffers, tab pages, frames,
terminals), `MemFile::abandon_pins` — so the swap-file cleanup can reach
every buffer. All were safe `pub(crate)` functions: called anywhere else,
each turns a live guard's borrow into one the next edit writes under. And
two callers did return into the loop after abandoning (the off-thread
hand-offs, when another exit was already preserving).

- **Closed:** `global_cell::emergency(exit)` is the one way in: it marks
  borrows abandoned and hands `exit` a `&Emergency`, and `exit` must
  return `Infallible` — it diverges, so no frame that held a loan resumes.
  Every abandon (and the memline lend that skips the lent-line count,
  `Buf::ml_abandoning`) asks for the token, which nothing else makes. The
  hand-offs that may go back to the loop check `begin_emergency_exit`
  first and abandon nothing when they return.

## Closed before the survey

- A flag a libuv callback writes through an address it was handed, read
  by a wait's closure, is a `Cell` (`event/socket/`): the release-only hang
  above. The audit for the same shape elsewhere — a local whose address
  goes to C as callback data (`&raw mut`, `as_ptr().cast()`, `.data =`,
  `rstream_start`'s data, a treesitter payload) and that Rust reads while C
  may write it — found none: every other such local is either a `Cell`, a
  `Copy` handle that dereferences afresh on each read, or touched by Rust
  only before the address is handed over and after the call that uses it
  returns. `process_events_until`'s docs carry the rule.

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
