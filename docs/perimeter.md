# The unsafe perimeter

The tree carries some 42,000 statements of unchecked code, and reading that as
one number is misleading: some of it is transpiled editor logic that safe Rust
will eventually replace, and some of it is the seam where this program meets a
C library, the operating system, or raw memory — code that will still be
`unsafe` when the migration is finished, because the thing on the other side
cannot be rewritten from here.

The **perimeter** is that second part, named explicitly. What is left over is
the migration's debt, counted by the ratchet as
`unsafe_stmts_outside_perimeter` and shrink-only like every other metric. That
number is the one to drive to zero.

Today: **7,478** unchecked statements inside the perimeter (137 files),
**28,341** outside it (838 files, of 1,351 measured). It was 138,877 when this
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
  store, the handles that make dereferencing the window/buffer graph safe.
  These exist so that a raw operation happens in one place and nowhere else;
  moving their unsafe upward would be a regression.

Nothing else qualifies, however raw it looks today. A transpiled algorithm
over pointers, a codec, an in-memory data structure and an on-disk parse can
all become safe Rust, and listing them here would retire them by fiat. So
`memline/` (including the on-disk block structs), `marktree/`, `shada/`,
`spellfile/`, `undo/`, `msgpack_rpc/`, `mpack/`'s own codec, `grid/` and
`log.rs` are **outside** the perimeter and are expected to leave the debt
column the ordinary way. When `memline`'s raw page access is isolated into a
page-view module of its own, _that_ module joins the perimeter; the mixed
files it lives in today do not.

**Membership is by module, not by line or by file.** An entry is a directory
prefix or one exact path, because a module is the unit that owns a boundary. A
file inside a perimeter module that turns out to be ordinary editor logic
should move out of the module rather than be carved out of the list.

## The list

| module                     | files | unchecked | why, and what would retire it                                                                                                                     |
| -------------------------- | ----: | --------: | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lua/`                     |    56 |     3,555 | LuaJIT's C API: a `lua_State`, its stack and the registry, plus luv and lpeg. Retires with the embedded interpreter, not before.                  |
| `lua/treesitter/`          |    10 |       874 | (inside `lua/`, listed for its own reason) the tree-sitter C library — parsers, trees, queries and cursors are opaque C objects with C lifetimes. |
| `mpack/lmpack/`            |     3 |       528 | libmpack-lua: a Lua C module, so every value it moves crosses the `lua_State` stack.                                                              |
| `cjson/lua_cjson/`         |     3 |       178 | lua-cjson: a Lua C module, same stack.                                                                                                            |
| `event/`                   |    11 |       562 | libuv: the loop, streams, timers, signals and processes are C objects registered by address.                                                      |
| `os/`                      |    21 |     1,306 | The operating system: libc and libuv syscalls, the PTY, the shell, the environment.                                                               |
| `vterm/`                   |    11 |       283 | libvterm, ported with its C ABI intact — the callbacks it takes and the symbols it exports are that library's interface.                          |
| `terminal/`                |     7 |       206 | The emulator's glue: a PTY on one side, libvterm's `extern "C"` callbacks on the other.                                                           |
| `tui/`                     |    14 |       585 | The terminal: libuv tty handles, termkey's parser, the terminfo entry unibilium hands back.                                                       |
| `xdiff/`                   |     1 |         8 | libxdiff, vendored: `mmfile_t` and the emit callbacks keep their C layout because the engine's interface is C.                                    |
| `allocator.rs`             |     1 |        21 | The global allocator: malloc/realloc/free.                                                                                                        |
| `memory/`                  |     2 |       113 | `xmalloc` and the arena — the floor under every owned type in the tree.                                                                           |
| `global_cell.rs`           |     1 |         9 | The checked wrapper over c2rust's mutable statics; the raw static is touched here so it is nowhere else.                                          |
| `winlayer.rs`, `winlayer/` |     3 |        28 | The window/buffer/position handles: constructing one is the unsafe step, dereferencing it is not.                                                 |
| `memfile/`                 |     2 |       203 | The swap file's page store — the only thing that hands out the address of a `.swp` page.                                                          |

Paths are relative to `crates/nvim/src/`. Counts are `unsafe_stmts` from
`metrics/ratchet.json` and drift as work lands; the list, not the table, is
the contract.

## How the ratchet enforces it

`PERIMETER` in `scripts/ratchet.py` is the list, one entry per row above, each
carrying its reason. Four things follow from it:

- **`unsafe_stmts_outside_perimeter`** — the tree's unchecked statements minus
  the perimeter's — is recorded in `metrics/ratchet.json` and may only shrink.
  A region is charged by what it _does_: every `;`-terminated statement, every
  block's tail expression and every `match` arm, at every depth, minimum one.
  Splitting `unsafe { a; b; c }` into three regions is not progress; deleting
  an operation is.
- **The list is self-pruning.** `check_perimeter` fails the run when an entry
  has no file with unchecked code behind it, so a module that finishes,
  moves or disappears has to leave the list in the same commit. A module
  reaching zero is the outcome the list is for, and it has to say so.
- **The perimeter cannot silently grow.** It needs no check of its own for
  that: every file's `unsafe_stmts` is already ratcheted individually, a
  brand-new file included, so unchecked code appearing inside the perimeter is
  a violation exactly as it is outside — and moving an unsafe file into a
  perimeter module shows up as a new path at full size.
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

with its other inner attributes, and `scripts/ratchet.py` holds the count of
files doing so as `files_allowing_unsafe_code`, shrink-only like everything
else. Three rules about that line:

- **A file inside a perimeter module names its row**, in a one-line comment
  directly above the attribute — "Unsafe perimeter: the `os/` row in
  docs/perimeter.md." That allow is permanent — the module is on the list
  precisely because its unsafe does not retire — and the comment is what makes
  the difference visible while reading the file rather than the list.
- **Every other file carries it bare.** There is no reason to write, because
  the count is the reason: those files are the migration's debt, the same
  population `unsafe_stmts_outside_perimeter` measures, and each one that
  finishes drops the allow for a `forbid` and the total falls by one. The
  exception is the dozen non-perimeter files exporting a C symbol somebody
  else resolves: they name their `metrics/abi-ledger.jsonl` rows, because the
  export is the whole reason the file cannot be finished.
- **A generated file takes it from the generator.** A hand-written attribute
  under `crates/nvim/src/api/private/dispatch*/` or `src/lua/api_wrappers/` is
  gone at the next `just apigen`, so `tools/apigen` emits it — and only for a
  chunk that actually holds unsafe, which is what keeps the count honest.

`forbid` still overrides the deny and still cannot be lifted by a module
underneath it, so it remains the stronger claim and the one a finished module
takes. What retired with the flip is the metric that used to stand in for
this: `files_without_forbid_unsafe` counted files that had not made a claim,
where `files_allowing_unsafe_code` counts the ones that have made the opposite
claim, deliberately, in a line a reviewer can see.

**There are no free `forbid`s left, and there will not be until a subtree
finishes.** Exactly 49 files carry neither attribute, and every one of them is
a `mod.rs` (or `lib.rs`) with at least one descendant that still allows
unsafe: `allow` under a `forbid` is `E0453`, a hard error, so writing the
attribute on any of them fails to compile. Counting files without the
attribute therefore says nothing about progress — the number cannot move until
`api/`, `eval/`, `lua/`, `os/` and the other 45 roots have emptied their
subtrees, at which point it falls in one step. This is the same rule as
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
