# The unsafe perimeter

The tree carries ~71k lines of unchecked code, and reading that as one number
is misleading: some of it is transpiled editor logic that safe Rust will
eventually replace, and some of it is the seam where this program meets a C
library, the operating system, or raw memory — code that will still be
`unsafe` when the migration is finished, because the thing on the other side
cannot be rewritten from here.

The **perimeter** is that second part, named explicitly. What is left over is
the migration's debt, counted by the ratchet as
`unsafe_lines_outside_perimeter` and shrink-only like every other metric. That
number is the one to drive to zero.

Today: **13,523** unchecked lines inside the perimeter (136 files),
**57,918** outside it (873 files, of 1,230 measured). It was 138,877 when
this file was written, at the end of phase 23's slice 15.

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
| `lua/`                     |    59 |     6,508 | LuaJIT's C API: a `lua_State`, its stack and the registry, plus luv and lpeg. Retires with the embedded interpreter, not before.                  |
| `lua/treesitter/`          |    10 |     1,908 | (inside `lua/`, listed for its own reason) the tree-sitter C library — parsers, trees, queries and cursors are opaque C objects with C lifetimes. |
| `mpack/lmpack/`            |     3 |       899 | libmpack-lua: a Lua C module, so every value it moves crosses the `lua_State` stack.                                                              |
| `cjson/lua_cjson/`         |     3 |       292 | lua-cjson: a Lua C module, same stack.                                                                                                            |
| `event/`                   |    11 |       969 | libuv: the loop, streams, timers, signals and processes are C objects registered by address.                                                      |
| `os/`                      |    21 |     2,419 | The operating system: libc and libuv syscalls, the PTY, the shell, the environment.                                                               |
| `vterm/`                   |    10 |       292 | libvterm, ported with its C ABI intact — the callbacks it takes and the symbols it exports are that library's interface.                          |
| `terminal/`                |     7 |       254 | The emulator's glue: a PTY on one side, libvterm's `extern "C"` callbacks on the other.                                                           |
| `tui/`                     |    14 |     1,236 | The terminal: libuv tty handles, termkey's parser, the terminfo entry unibilium hands back.                                                       |
| `xdiff/`                   |     1 |        40 | libxdiff, vendored: `mmfile_t` and the emit callbacks keep their C layout because the engine's interface is C.                                    |
| `allocator.rs`             |     1 |        38 | The global allocator: malloc/realloc/free.                                                                                                        |
| `memory/`                  |     2 |       141 | `xmalloc` and the arena — the floor under every owned type in the tree.                                                                           |
| `global_cell.rs`           |     1 |         9 | The checked wrapper over c2rust's mutable statics; the raw static is touched here so it is nowhere else.                                          |
| `winlayer.rs`, `winlayer/` |     2 |        26 | The window/buffer/position handles: constructing one is the unsafe step, dereferencing it is not.                                                 |
| `memfile.rs`               |     1 |       400 | The swap file's page store — the only thing that hands out the address of a `.swp` page.                                                          |

Paths are relative to `crates/nvim/src/`. Counts are `unsafe_lines` from
`metrics/ratchet.json` and drift as work lands; the list, not the table, is
the contract.

## How the ratchet enforces it

`PERIMETER` in `scripts/ratchet.py` is the list, one entry per row above, each
carrying its reason. Three things follow from it:

- **`unsafe_lines_outside_perimeter`** — the tree's unchecked lines minus the
  perimeter's — is recorded in `metrics/ratchet.json` and may only shrink.
- **The list is self-pruning.** `check_perimeter` fails the run when an entry
  has no file with unchecked lines behind it, so a module that finishes,
  moves or disappears has to leave the list in the same commit. A module
  reaching zero is the outcome the list is for, and it has to say so.
- **The perimeter cannot silently grow.** It needs no check of its own for
  that: every file's `unsafe_lines` is already ratcheted individually, a
  brand-new file included, so unchecked code appearing inside the perimeter is
  a violation exactly as it is outside — and moving an unsafe file into a
  perimeter module shows up as a new path at full size.

To add an entry: put the path and its reason in `PERIMETER`, add the row here,
run `just refresh`, and justify it in the commit message. The number this
lowers is the number the migration is judged by, so the bar is the criteria
above and nothing softer.

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
because the FFI-safety lint follows a pointer out of `buf_T`. All of those
are still expected to leave the residue column the ordinary way.
