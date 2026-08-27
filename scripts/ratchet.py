#!/usr/bin/env python3
"""Ratchet the migration metrics: counts may hold or shrink, never grow.

The migration's promise is monotonic progress — every change leaves the tree
no less safe than it found it. This script is the mechanism. It measures, per
Rust source file (crates/*/src/**/*.rs plus the crate-root .rs files;
integration tests under crates/*/tests are not migration surface and stay
unmeasured, as they were when they lived at the repo root):

  unsafe_lines  lines of *code* the compiler is not checking: the lines
              spanned by an `unsafe {}` block, by an `unsafe extern` block,
              or by the body of an `unsafe fn` in a file that has not
              adopted #![deny(unsafe_op_in_unsafe_fn)] (there the body is
              implicitly unsafe throughout, which is exactly what the metric
              is about). Spans are unioned, so a nested block never counts
              twice, and blank/comment-only lines are excluded, so a SAFETY
              comment is free and a rewrite is never penalised for
              explaining itself. `unsafe impl`/`unsafe trait` cost their
              header line — an unchecked promise with nowhere else to book
              it. An `unsafe fn` *type* (a function pointer) costs nothing:
              the obligation is paid where it is called.

              This metric used to count `unsafe {}` *blocks*, and blocks and
              the goal diverge on exactly the change the migration wants
              most. Splitting a 700-line transpiled body into fifteen
              functions with narrow blocks books fifteen units where there
              was one: edit.rs went 89 -> 104 blocks across phase 15 while
              its clippy count went 33 -> 0. Lines-of-unchecked-code moves
              the right way for every shape: narrowing a block lowers it,
              deleting unsafe code lowers it, adopting the deny and wrapping
              a body is neutral, and adding unchecked code raises it. It is
              also the number that states the goal — phase by phase, how
              much of the editor the compiler still cannot vouch for.

  static_mut  occurrences of "static mut "
  no_mangle   occurrences of "#[unsafe(no_mangle)]"
  variadic    occurrences of ": ..." — C-variadic parameters, whose calls
              are format-string-unchecked. They retire as their callers
              migrate to the format_args!-based macros (semsg! and friends)
              or their modules are rewritten; vim_snprintf/vim_vsnprintf
              (vim's own user-visible format language) are expected to be
              the long-lived remainder.
  extern_abi  definitions of functions carrying a C ABI — `extern "C" fn
              name` / `extern "C-unwind" fn name`, i.e. the named form, so an
              `extern` *type* (a function pointer) and a declaration inside an
              `unsafe extern` block are not counted. Each one is a signature
              the compiler cannot check across and that clippy skips wholesale,
              and phase 18's census classed every survivor: exported to C
              (`#[unsafe(no_mangle)]`, an abi-ledger entry), address-taken by a
              declared C caller (lua_CFunction, libvterm, libuv, qsort), a
              variadic (where `...` requires the ABI), or an api entry point
              whose ABI is only apigen's recognition convention. The last class
              retires as `api/` converts to `Result<T, Error>`; the rest retire
              as their C callers do.
  missing_safety_doc
              `unsafe fn`s whose doc comment has no `# Safety` section — the
              obligation the signature announces but nobody wrote down. It
              stands in for `clippy::missing_safety_doc`, which is *allowed*
              in Cargo.toml (2 380 findings when phase 19 switched the style
              tier on; that is a phase of work, not a slice, and it retires as
              modules are rewritten rather than by a sweep of stub sections).
              The lint could not be ratcheted where it was: clippy reports it
              at the crate lint level, so a site-level or module-level
              `#[allow]` cannot scope it, and leaving it warning would bury
              `just lint`'s ~20 real findings under 2 380.

              Counted here instead, from the source, which also makes it
              *broader* than the lint in the one direction that helps:
              clippy only asks exported functions, this asks every `unsafe fn`
              the tree defines or declares, because a private one's caller has
              the same obligation to discharge. Declarations inside an
              `unsafe extern` block are not counted (their obligation is the C
              library's) and neither is an `unsafe fn` *type*. The section is
              recognised as a `# Safety` heading in the run of `///` lines
              immediately above the item, attributes skipped — which is the
              shape rustfmt keeps and the shape every rewritten module already
              uses.

  cell_ptr    raw escape-hatch accesses to global editor state:
              GlobalCell/SharedCell `.ptr()` and `.as_raw()`. The cells
              themselves are the safe replacement for c2rust's mutable
              statics, but `ptr()` hands back a bare `*mut T` and reinstates
              every obligation the cell exists to check — it was the
              landing zone for the mechanical conversion, not a destination.
              Call sites narrow to get/set/with/with_mut as their modules
              are rewritten, and cells whose state can simply be owned are
              deleted outright; both show up here. Counting accesses rather
              than cells is deliberate: converting a surviving `extern`
              global into a real cell is progress, and a metric over cell
              *declarations* would book it as regression. The needles are
              receiver-blind, as everything else here is; today no other
              type in the tree has a nullary `ptr()`/`as_raw()`, and one
              that did would only over-count, which is the safe direction.

              Since phase 22 it no longer falls to zero: see "the cell_ptr
              partition" below for what the residue is and which of the three
              whole-tree numbers each site lands in.
  lines       line count, **excluding `#[cfg(test)]` modules**. No file may
              exceed 1,000 lines; files already over the cap are
              grandfathered at their committed size and may shrink or hold,
              never grow. New files start at the cap.

              The exemption is the cap's alone, and it exists because the two
              were pulling against each other. The migration wants tests next
              to the code they cover -- a rewritten module's safe core with
              its Miri-runnable `#[cfg(test)] mod tests` in the same file is
              the shape every phase-20 slice produced -- while the cap wants
              files small enough to read. Counting test lines toward the cap
              taxes exactly the posture being asked for: cjson's decoder sat
              at 955 lines, 200 of them its 21 tests, so the next test to be
              written would have forced a split of the *production* code that
              nothing about the production code justified.

              Nothing else is exempt. `unsafe_lines`, `missing_safety_doc`,
              `cell_ptr` and the rest are still counted inside a test module,
              because unchecked code in a test is still unchecked code and a
              test is a fine place to be pushed away from writing it. Only
              the cap looks away, and only at a `#[cfg(test)] mod name { .. }`
              item -- a `#[cfg(test)] mod name;` declaration is a separate
              file, and that file is measured in full like any other.

  pub_items   items at the crate's outer boundary: a `pub` declaration or
              `pub use` re-export at column 0, which is where rustfmt puts
              every module-level item (an indented one is an associated item
              or lives in an inline module, and neither is nameable through a
              `use` path). `unreachable_pub` is denied in both packages, so
              every one of these really is reachable from outside the crate —
              which is what makes the number mean something. It is the size of
              a surface that ought to be a boundary and is not: c2rust made
              every translation unit's symbols visible, so the tree is ~15k
              items wide at the root, of which a few hundred are earned (see
              metrics/visibility-ledger.jsonl). A `pub use` tree counts once
              however many leaves it names; the point is pressure, not a
              census. It falls as modules narrow, which is phase 22's business.

plus these whole-tree metrics, which are not per-file:

  internal_exports  the number of internal-only exports in the committed ABI
                    ledger (metrics/abi-ledger.jsonl — `just abi-ledger
                    --check` separately guarantees that file matches the
                    tree).

  test_reached_pub  the number of records in metrics/visibility-ledger.jsonl:
                    `pub` items whose only reacher from outside the crate is
                    an integration test under crates/nvim/tests. The ABI
                    ledger's `test` class, continued in Rust — an entry point
                    that stays public because a ported spec drives it. Falls
                    when a test stops needing the entry point; a new one is
                    growth that has to be justified.

  the cell_ptr partition   phase 22 finished the sweep, and what `cell_ptr`
                    is left counting is no longer one undifferentiated pile
                    to be driven to zero. Every site now falls in exactly one
                    of four classes, and three of them are ratcheted numbers
                    here (the fourth, the boundary, is the floor and is not
                    counted at all):

                      boundary   — CELL_PTR_ALLOW. Addresses libuv or libc
                        owns: a `uv_loop_t` with self-referential
                        multiqueues, uv stream/timer/signal handles, two
                        `uv_mutex_t`s, a `uv_thread_t`, a `struct termios`
                        handed to `tcsetattr`. They cannot become owned Rust
                        state without rewriting the C library underneath
                        them, so they are exempt outright.
                      keepers    — CELL_PTR_KEEPERS, counted as
                        `cell_ptr_keepers`.
                      accessors  — everything else, counted as
                        `cell_ptr_accessors`, and capped at **one site per
                        receiver** (see `check_cell_ptr`).

                    Both lists are keyed by *name*, not by file,
                    deliberately: `main_loop` alone is read from 29 files
                    that also hold unrelated `.ptr()` sites, so a file-keyed
                    list would exempt those too. A rename cannot silently
                    widen CELL_PTR_ALLOW — `check_names` fails the run if a
                    listed name no longer declares a GlobalCell/SharedCell
                    static — and it cannot widen CELL_PTR_KEEPERS either,
                    because a renamed keeper's sites land in the accessor
                    class and trip the one-site cap.

                    (A third register, CELL_PTR_DEFERRED, held the handle
                    registries while phase 23 took ownership of them. It
                    reached 0 in that phase's S4 — `buffer_handles`,
                    `window_handles` and `tabpage_handles` are owned
                    `SlotTable`s now, and there is no address left to hand
                    out — and it was deleted with its metric, as its own
                    rule requires: an empty register is a floor with nothing
                    under it.)

                    (This replaces `cell_ptr_unlisted`, which was
                    `cell_ptr` minus the boundary and was retired at the
                    close of phase 22 with the three numbers below summing to
                    exactly what it held. It was written to reach 0; the
                    phase's own finding is that it cannot, because
                    `GlobalCell` offers no way *but* `ptr()` to answer a
                    family that works from an address, and the accessor class
                    is where those live.)

  cell_ptr_keepers  `cell_ptr` sites on a global a slice *ruled* may keep
                    more than one, with the ruling named in CELL_PTR_KEEPERS.
                    Two shapes are in there: a family whose whole point is an
                    address (the NFA postfix program, the backtracking
                    engine's state, `GlobalCell`'s own tests of `ptr`), and a
                    receiver that is not one cell at all (`cell` is a local
                    bound from an option slot or a grid; `SCRATCH` names two
                    unrelated statics with one site each). Without the list
                    these would trip the accessor cap, and the cap is worth
                    more than the exemption costs: the regression it forbids
                    is a family reaching for the escape hatch at nine sites
                    again.

  cell_ptr_accessors  every `cell_ptr` site that is none of the above. Each
                    such receiver may have **at most one** — the acquire-once
                    shape phase 22 converged on, a function or handle
                    constructor whose whole body is `X.ptr()` with a doc
                    comment saying why the address is what the family works
                    from. `check_cell_ptr` fails the run on a second one,
                    naming the receiver; the count itself is ratcheted, so
                    the class can only shrink. A site whose receiver is not a
                    bare identifier (`self.0.ptr()` in an accessor method) is
                    counted here and is not subject to the by-name cap, which
                    the ratcheted total covers instead.

  cell_copy_owner   `.get()` reads of a global whose `T` derives `Copy` *and*
                    transitively owns a raw pointer — a `String_0`, a
                    `garray_T`, a `regbehind_T`. `get` copies the struct out
                    of the cell, so the copy and the global now hold the same
                    pointer: whoever frees or reallocates through one leaves
                    the other dangling (`getchar/redo.rs` used to write
                    `old_redobuff.set(redobuff.get())`, and both then owned
                    the same block chain -- phase 22's S7 retired that whole
                    family by making the type not `Copy`). Most sites are
                    borrows *spelled* as
                    copies —
                    `script_items.get().ga_len` — and their fix is the owning
                    family's rewrite, not a blanket transformation; a few are
                    genuine moves and want `replace`/`take`.

                    Counting them is what stops a 32nd static from joining the
                    set unnoticed while the families are being rewritten. The
                    list of statics (CELL_COPY_OWNER) is hand-maintained,
                    because "derives Copy and transitively owns a pointer" is
                    a type-graph question this script has no business
                    answering; `check_names` does guarantee every name on it
                    still declares a cell, so a deleted global cannot leave a
                    stale entry propping the floor up.

  unsafe_lines_outside_perimeter
                    `unsafe_lines` restricted to files that are *not* on the
                    unsafe perimeter (PERIMETER, below) — the tree's unchecked
                    code minus the part that is expected to stay unchecked.
                    This is the migration's debt number: the total says how
                    much unchecked code the binary contains, this says how
                    much of it the compiler should one day be checking, and
                    the two stop being confused for each other.

                    The perimeter is where unsafe *bottoms out* in something
                    the migration cannot rewrite: a C library's ABI (LuaJIT,
                    libuv, libc, libvterm, termkey, tree-sitter, xdiff), the
                    operating system, the C symbols this tree exports for
                    others to call, and the raw-memory primitives every safe
                    abstraction above them is built on (the allocator,
                    `GlobalCell`, the swap file's page store, winlayer's
                    handles). Everything else is debt, however raw it looks
                    today: a transpiled algorithm over pointers, a codec, a
                    data structure and an on-disk parse can all be safe Rust,
                    and listing them here would retire them by fiat.

                    Membership is by *module*, not by line or by file: an
                    entry is a directory prefix (trailing `/`) or one exact
                    path, because a module is the unit that owns a boundary.
                    A file inside a perimeter module that turns out to be
                    ordinary editor logic should move out of the module
                    rather than be carved out of the list. docs/perimeter.md
                    is the prose version — what qualifies, the current list,
                    and what would have to change for an entry to leave.

                    The list is self-pruning: `check_perimeter` fails the run
                    when an entry has no file with unchecked lines behind it,
                    so a module that finishes, moves or disappears has to
                    leave the list in the same commit — the discipline
                    CELL_PTR_KEEPERS already uses. It cannot silently *grow*
                    either, and needs no check of its own for that: every
                    file's `unsafe_lines` is already ratcheted individually,
                    a new file included, so unchecked code appearing inside
                    the perimeter is a violation exactly as it is outside.

  files_without_forbid_unsafe  the number of source files not carrying
                    #![forbid(unsafe_code)]. The shrink-only trick inverted:
                    fully safe modules take the attribute — which makes
                    "safe module" a compiler-enforced status instead of a
                    grep result — and the count of files still lacking it
                    may only fall. New files are expected to be born safe.

  files_without_deny_casts  the number of source files that have not adopted
                    the cast lints. `as` is the transpile's universal
                    conversion — ~21k of them, ~5k infallible widenings that
                    `From` answers and a long tail of narrowings that need
                    `TryFrom` and an error — and clippy's cast family
                    (`cast_lossless`, `cast_possible_truncation`,
                    `cast_possible_wrap`, `cast_sign_loss`, `ptr_as_ptr`) is
                    pedantic, i.e. allowed by default. Turning it on tree-wide
                    is a big-bang sweep nobody can review; the migration adopts
                    it per module, as the roadmap's phase 19 item 6 asks.

                    So the same inverted trick as the two attributes above: a
                    module that has finished its casts writes

                        #![deny(
                            clippy::cast_lossless,
                            clippy::cast_possible_truncation,
                            clippy::cast_possible_wrap,
                            clippy::cast_sign_loss,
                            clippy::ptr_as_ptr
                        )]

                    and the count of files *not* carrying it may only fall.
                    The needle is `clippy::cast_lossless` inside a `deny(...)`,
                    spanning newlines so rustfmt may wrap the list: that lint
                    is the one the `From` vocabulary answers, so it is the
                    family's marker. Naming the rest is the house convention,
                    not something the ratchet can check — as with
                    `forbid(unsafe_code)`, the attribute is a claim the
                    compiler then enforces, and the ratchet only counts who
                    has made it.

  files_without_deny_unsafe_op  the number of source files carrying neither
                    #![forbid(unsafe_code)] nor
                    #![deny(unsafe_op_in_unsafe_fn)]. Same trick for edition
                    2024's honest-unsafe lint: Cargo.toml allows it (blanket
                    body-wrapping would double the textual unsafe count), each
                    module denies it once its unsafe fns use explicit unsafe
                    blocks, and the count of files doing neither may only fall.
                    Phase 20 drove it to **0**: the crate root's inner
                    attribute is crate-level rather than per-module, so lib.rs
                    could only take it once every file beneath it already had
                    a marker — which is exactly the state the criterion
                    describes, and why lib.rs went last.

A `warnings` metric used to sit alongside it; phase 5 drove the count to
zero and the dev shell (flake.nix) now sets `RUSTFLAGS="-D warnings"` for
every local and CI build instead, so the counter is retired.

One thing here is not a count but a hard check, and it fails the run outright:
**a write through an accessor that answers by value**. `f().field = x`
compiles when `f` returns a value — the assignment lands in a temporary that
is dropped on the next line, and the write is a silent no-op. Three of those
shipped past `cargo test`, the unit suite, Miri and 2 743 functional tests;
only `oldtest test_profile` caught them, because it is the only lane that
reads a derived number back. The check is name-based and deliberately
over-approximating in the *safe* direction: a call is accepted if **any** `fn`
of that name in the tree answers with a place (a `&`/`*` type, or a newtype
that `impl DerefMut`s, which is how `cur_buf()`/`cur_win()` write through a
pointer — a `type` alias of such a newtype, as `ops::Op` is of
`winlayer::Live`, counts as one too). So a same-named sibling can hide a real one; what it cannot do is
cry wolf, which is what would get it switched off.

Everything is measured over a *masked* copy of the source, in which comments,
string literals and character literals are blanked out (offsets and newlines
preserved) so that only code is scanned. That is what makes the counts mean
what they say: prose about `unsafe` costs nothing, a doc comment quoting
`#![deny(unsafe_op_in_unsafe_fn)]` does not switch on the deny, and a string
containing a brace cannot desynchronise the block scanner. Everything else is
plain substring matching — bar `extern_abi`, whose needle is a regex because
masking erases the very ABI string a substring would key on — which still
over-counts a little (a macro naming
`static mut ` in its expansion counts), but is deterministic, cheap enough for
a pre-commit hook, and kept canonical by rustfmt (enforced by fmt-check). The
point is monotonic pressure, not precision.

The baseline is committed at metrics/ratchet.json (one file per line, so diffs
review like the ledger's). A metric above its baseline is a violation; a
metric below it means progress that must be locked in by regenerating the
baseline and committing it alongside the change.

Regenerate through `just refresh`, not this script directly: the measurement
is only valid on a formatted tree with a current ledger, and refresh sequences
those. Calling ratchet.py first and formatting after bakes in line counts the
formatter is about to change.

Usage: ratchet.py [--check] [--allow-growth]
  --check         compare the tree against the committed baseline instead of
                  writing: exit 1 if any metric grew, or if the baseline is
                  stale (a metric shrank but metrics/ratchet.json wasn't
                  regenerated).
  --allow-growth  write a baseline even though a metric grew. The override
                  for justified cases — the growth shows up in the
                  metrics/ratchet.json diff; explain it in the commit message.
"""

import collections
import json
import re
import sys
from bisect import bisect_right
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "metrics" / "ratchet.json"
LEDGER = ROOT / "metrics" / "abi-ledger.jsonl"
VISIBILITY = ROOT / "metrics" / "visibility-ledger.jsonl"

LINE_CAP = 1000
# name -> needles counted in the masked source, summed.
COUNTED = {
    "static_mut": ("static mut ",),
    "no_mangle": ("#[unsafe(no_mangle)]",),
    "variadic": (": ...",),
    "cell_ptr": (".ptr()", ".as_raw()"),
}
# name -> regex counted in the masked source, for what a substring cannot
# separate. Masking blanks the ABI string itself (`extern "C" fn` reads
# `extern     fn`), which is why this is a regex and why it is ABI-blind —
# every `extern` the tree writes names a C ABI. `fn <name>` is the definition
# form: a function-pointer type writes `fn(` with no name, and a declaration
# inside an `unsafe extern` block has the block's `{` between the two words.
COUNTED_RE = {
    "extern_abi": re.compile(r"\bextern\s+fn [A-Za-z_]"),
    # ABI-blind for the same reason `extern_abi` is: masking blanks the ABI
    # string. `pub(crate)`/`pub(super)` do not match — the space is required.
    "pub_items": re.compile(
        r"^pub (?:unsafe )?(?:extern\s+)?"
        r"(?:fn|static|const|struct|enum|union|trait|type|mod|use)\b",
        re.M,
    ),
}
# The globals whose raw address a C library owns, and the reason for each.
# These are the `cell_ptr` floor: they are subtracted from the tree's total
# and are not counted by any of the three whole-tree numbers, because they
# cannot become owned Rust state without rewriting the C library underneath
# them. Keyed by name because a file-keyed list would exempt every unrelated
# site in the 29 files `main_loop` reaches.
CELL_PTR_ALLOW = {
    "main_loop": "uv_loop_t plus self-referential multiqueues; libuv owns the address",
    "read_stream": "RStream — a uv stream handle registered with the loop",
    "dummy_ap": "VaList<'static>; retires with the variadics, not with the cells",
    # NB. `msgpack_rpc/server.rs` declares a second, unrelated `WATCHERS`;
    # the list is name-keyed, so a `.ptr()` there would be exempted too. It
    # has none today, and `cell_ptr` still counts it.
    "WATCHERS": "uv_signal_t array whose addresses are registered with uv",
    "REFRESH_TIMER": "TimeWatcher — a uv timer handle",
    "MUTEX": "uv_mutex_t, const-initialised in place",
    "runtime_search_path_mutex": "uv_mutex_t",
    "main_thread": "uv_thread_t, compared with uv_thread_equal",
    "TERMIOS_DEFAULT": "struct termios handed to tcsetattr",
}
# No whitespace is tolerated around the `.`, so that what this subtracts is
# exactly a subset of what `cell_ptr`'s substring needles counted; rustfmt
# writes the canonical form and `just fmt-check` enforces it.
CELL_PTR_ALLOW_RE = re.compile(
    r"\b(?:" + "|".join(map(re.escape, CELL_PTR_ALLOW)) + r")\.(?:ptr|as_raw)\(\)"
)
# Globals a slice ruled may hold more than one site, and the ruling. Counted
# as `cell_ptr_keepers`. Without this list each would trip the accessor cap;
# see "the cell_ptr partition" in the doc block for why the cap is kept.
CELL_PTR_KEEPERS = {
    "POSTFIX": "the NFA postfix program; phase 22's S10 ruled the address is the program",
    "cell": "not one cell — a local bound from an option slot (S3) or a grid (S13)",
    "buf": "F-P22-49: `TSInput.read`'s contract needs a buffer surviving the return",
    "CELL": "`GlobalCell`'s own tests of `ptr`/`as_raw`; they must call them",
    "simple_diffline_change": "F-P22-37: the address goes into `diffline.changes` and is compared back",
    "highlight_attr": "the attribute table `hl_attr_active` holds; one site is its const initialiser",
    "compl_xp": "the completion `expand_T`, taken by pointer by `expand_cmdline`/`nlua_expand_pat` (S5/S6)",
    "SCRATCH": "not one cell — two unrelated statics (mark F-P22-52, quickfix) with one site each",
    "BT_STATE": "the backtracking engine's state; phase 22's S10 ruled it taken raw per match",
}
# Every `.ptr()`/`.as_raw()` site with its receiver, for the partition. The
# spacing is pinned exactly as CELL_PTR_ALLOW_RE's is, so what this classifies
# is a subset of what `cell_ptr`'s substring needles counted; a receiver that
# is not a bare identifier (`self.0.ptr()`) simply does not match, and lands
# in the accessor count without being subject to the by-name cap.
CELL_PTR_SITE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\.(?:ptr|as_raw)\(\)")
# Globals whose `T` derives Copy *and* transitively owns a raw pointer, so a
# `.get()` hands out a second owner of the same allocation. Hand-maintained:
# see the doc block. Grouped by the family whose slice retires them.
CELL_COPY_OWNER = (
    # insexpand is done: `compl_pattern`, `compl_leader`, `compl_orig_text`
    # and `adjusted_leader` retired in phase 22's S5 behind `ComplStr`, and
    # `compl_orig_extmarks` in S6 behind `ComplOrigExtmarks`. Each is now
    # reached only through the single owner of its buffer, so no site copies
    # the words out of the cell to get at the allocation.
    # runtime — the search-path buffers. `script_items` and `ga_loaded` left
    # the list in phase 22's S16: both are `Vec`s now, so neither is `Copy`
    # and there is no `get` on either to count.
    "runtime_search_path",
    "runtime_search_path_thread",
    # getchar is done: the five `buffheader_T` cells and `typebuf` left the
    # list in phase 22's S7. `KeyBuffer` and `TypeAhead` are not `Copy` at
    # all now, so there is no `get` on either to count.
    # regexp — saved matcher state holding pointers into the subject
    "rex",
    "rsm",
    "behind_pos",
    # the rest, one or a few sites each
    "curgrid",
    # `au_new_curbuf` left the list in phase 23's S11. It is a `bufref_T`,
    # which is upstream's own *weak* reference -- the `br_buf_free_count`
    # generation check exists precisely because the pointer is borrowed --
    # and since S11 the buffer's owner is a named one, the buffer registry's
    # `Owned<buf_T>`. A `get` on this cell copies a reference, not an owner.
    "dont_sync_undo",
    "old_sub",
    # ccline is done: `cmdline_block` and `restart_args` retired in phase 22's
    # S14 behind `CmdlineBlock`/`RestartArgs`, two owned newtypes over the
    # `Array` with a `Drop`. Neither is `Copy`, so there is no `get` on either
    # to count, and the one genuine move at each is `GlobalCell::take`.
    "pending_vimresume",
    "CLIPBOARD",
    "pc_status",
    "EXPAND_WHAT",
    "value_init_String",
    "provider_caller_scope",
    "saved_last_search_spat",
    "counted",
    # The `Object` cells, from phase 22's F-P22-53 audit at the close: every
    # `GlobalCell<String_0>` and `GlobalCell<Object>` in the tree, listed
    # whether or not the value it happens to hold owns anything today, so the
    # register can be re-derived by grepping for those two types. `names`
    # carries no site now -- `nth_lua_string` reads it through `with` and
    # `cache_lua_answer` frees the old answer with `replace` -- and is listed
    # so a `get` on it would be counted rather than discovered later.
    "METADATA",  # the API description; its arena is leaked, the tree shared
    "msg_ext_id",  # an Object that is always an Integer; both writers agree
    "names",  # the two cached Lua completion answers in cmdexpand/generate
)
# Whitespace *is* tolerated here, unlike CELL_PTR_ALLOW_RE: this count is
# not a subtraction from a substring total, and rustfmt wraps a long chain
# onto its own line (`*script_items\n    .get()`), which is still a copy.
CELL_COPY_OWNER_RE = re.compile(
    r"\b(?:" + "|".join(map(re.escape, CELL_COPY_OWNER)) + r")\s*\.\s*get\(\)"
)
# What `check_names` demands of every name on either list: that somewhere in
# the tree it still names a `static X: GlobalCell<..>`/`SharedCell<..>`. Any
# visibility may precede `static`, and the static may sit inside a function
# (`TERMIOS_DEFAULT` does), so only the two words either side of the name are
# pinned.
CELL_DECL = r"\bstatic\s+{}\s*:\s*(?:GlobalCell|SharedCell)\b"

# The unsafe perimeter: the modules whose unchecked code is expected to
# outlive the migration, because removing it would mean rewriting something
# this tree does not own. `unsafe_lines_outside_perimeter` is the tree's
# unchecked lines minus these; docs/perimeter.md carries the prose. An entry
# is a directory prefix (trailing `/`) or one exact path, and every entry must
# have a file with unchecked lines behind it or `check_perimeter` fails --
# the list prunes itself as modules finish.
PERIMETER = {
    # -- Foreign ABIs. The unsafe is the call, and it retires when the
    # library does.
    "crates/nvim/src/lua/": "LuaJIT's C API: a `lua_State`, its stack, and the "
    "registry — plus luv and lpeg, which are C libraries of their own",
    "crates/nvim/src/lua/treesitter/": "the tree-sitter C library: parsers, "
    "trees, queries and cursors are opaque C objects with C lifetimes",
    "crates/nvim/src/mpack/lmpack/": "libmpack-lua: a Lua C module, so every "
    "value it moves crosses the `lua_State` stack",
    "crates/nvim/src/cjson/lua_cjson/": "lua-cjson: a Lua C module, same stack",
    "crates/nvim/src/event/": "libuv: the loop, streams, timers, signals and "
    "processes are C objects registered by address",
    "crates/nvim/src/os/": "the operating system: syscalls through libc and "
    "libuv, the PTY, the shell, the environment",
    "crates/nvim/src/vterm/": "libvterm, ported with its C ABI intact — the "
    "callbacks it takes and the symbols it exports are that library's",
    "crates/nvim/src/terminal/": "the terminal emulator's glue: a PTY on one "
    'side and libvterm\'s `extern "C"` callbacks on the other',
    "crates/nvim/src/tui/": "the terminal: libuv tty handles, termkey's "
    "parser, and the terminfo entry unibilium hands back",
    "crates/nvim/src/xdiff/": "libxdiff, vendored: `mmfile_t` and the emit "
    "callbacks keep their C layout because the engine's interface is C",
    # -- The primitives everything else stands on. Their whole job is to be
    # the one place a raw operation happens, so that callers need not.
    "crates/nvim/src/allocator.rs": "the global allocator: malloc/realloc/free",
    "crates/nvim/src/memory/": "xmalloc and the arena — the floor under every "
    "owned type in the tree",
    "crates/nvim/src/global_cell.rs": "the checked wrapper over c2rust's "
    "mutable statics; the raw static is touched here so it is nowhere else",
    "crates/nvim/src/winlayer.rs": "the window/buffer/position handles: "
    "constructing one is the unsafe step, dereferencing it is not",
    "crates/nvim/src/winlayer/": "the same, split out by family",
    "crates/nvim/src/memfile.rs": "the swap file's page store — the only thing "
    "that hands out the address of a `.swp` page",
}

FORBID = "#![forbid(unsafe_code)]"
DENY_UNSAFE_OP = "#![deny(unsafe_op_in_unsafe_fn)]"
# A module's claim to have finished its casts. `.` spans newlines so the list
# may be wrapped; `clippy::cast_lossless` is the family's marker (see above).
DENY_CASTS = re.compile(r"#!\[deny\([^\]]*\bclippy::cast_lossless\b", re.DOTALL)
# A `# Safety` heading in a doc comment. Any heading level, any case, because
# what is being counted is whether the obligation is written down.
SAFETY_HEADING = re.compile(r"^\s*///\s*#+\s*safety\b", re.IGNORECASE)
DOC_LINE = re.compile(r"^\s*///")
# A test module's header, `#[cfg(test)]` through the `{` that opens it. Further
# attributes may sit between the two, and the module may carry any visibility.
# The trailing `{` is required: `#[cfg(test)] mod tests;` names another file,
# which is measured on its own.
CFG_TEST_MOD = re.compile(
    r"#\[cfg\(test\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub\s*(?:\([^)]*\)\s*)?)?"
    r"mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{"
)

# Metrics computed from the source rather than counted with a needle.
DERIVED = ("unsafe_lines", "missing_safety_doc")

# `accessor().field = value` and its compound-assignment forms, which is a
# silent no-op when `accessor` answers by value. Nullary on purpose: that is
# the shape an accessor has, and requiring it keeps the needle away from
# builder chains. `=(?!=)` so a comparison is not a write.
PLACE_WRITE = re.compile(
    r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(\)\s*\.\s*[A-Za-z_][A-Za-z0-9_]*"
    r"\s*(?:[-+*/%|&^]|<<|>>)?=(?!=)"
)
# A newtype whose `.field` reaches through to something it points at.
DEREF_MUT = re.compile(
    r"\bimpl(?:<[^>]*>)?\s+(?:[A-Za-z0-9_]+::)*DerefMut\s+for\s+([A-Za-z_][A-Za-z0-9_]*)"
)
# `type Op = Live<oparg_T>;` — a family's name for a shared generic wrapper.
# The scan is keyed on names, so an alias of a `DerefMut` type is one too.
TYPE_ALIAS = re.compile(
    r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*=\s*"
    r"(?:[A-Za-z0-9_]+::)*([A-Za-z_][A-Za-z0-9_]*)"
)
FN_NAME = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")
# `&mut unsafe { *p }` and `&raw mut unsafe { (*p).f }`: a borrow of an
# `unsafe` block whose value is
# a *dereference*. A block is a value expression, so the borrow binds to a copy
# on the stack that temporary-lifetime-extension keeps alive for the statement.
# Every write through it is discarded, it compiles, and nothing warns -- the
# same trap as `unsafe { *p }.f = v`, one rung up. Phase 23's S16 shipped five
# in one hour: `save_dbg_stuff` saved nothing, and `get_loop_line`'s
# `cp.current_line += 1` never advanced, which is a `:while` that never ends.
#
# A borrow of a *call*'s result is a different thing and is fine
# (`&unsafe { render_char(buf, c) }` borrows a value that was returned), so the
# needle demands a `*` -- optionally behind parentheses -- as the block's first
# token.
# `unsafe { *p }.retain()` -- a mutating method called on the *value* an
# `unsafe` block produced. `&mut self` autorefs the temporary, so the mutation
# lands in the copy and is discarded, exactly as an assignment would be. rustc
# allows `&mut` on an rvalue and says nothing.
#
# The receiver's type is out of reach of a name-keyed scan, so the needle asks
# instead which *names* can only be mutating ones: declared `&mut self`
# somewhere and never `&self`, `self` or `mut self`. That drops `.is_null()`
# (a `&self` on a raw pointer) and `.has()` (a shared flag test) without
# needing to know what the receiver is. The three reference-count methods are
# named outright: `Refcount::release` shares its name with two *consuming*
# `release(self)` methods, so the exclusivity rule would lose it -- and losing
# it costs a reference that is never given back. Batch 3 wrote twelve of these
# in one afternoon, including `:function!` over a referenced function, which
# leaked the old definition.
MUT_SELF_FN = re.compile(
    r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*\(\s*&\s*mut\s+self\b"
)
NON_MUT_SELF_FN = re.compile(
    r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*\(\s*(?:&\s*self|mut\s+self|self)\b"
)
DEREF_METHOD = re.compile(
    r"unsafe\s*\{\s*\(?\s*\*[^{}]*?\}\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\("
)
# `Refcount`/`RefcountSize`, whose whole point is that the count moves.
REFCOUNT_METHODS = frozenset({"retain", "release", "release_many"})

BORROWED_DEREF = re.compile(
    r"&\s*(?:mut|raw\s+mut|raw\s+const)\s+unsafe\s*\{\s*\(*\s*\*"
)


def balanced(text, start, opens, closes):
    """The index just past the bracket group beginning at `start`."""
    depth = 0
    i = start
    while i < len(text):
        if text[i] in opens:
            depth += 1
        elif text[i] in closes:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return i


def fn_returns(masked, out):
    """name -> the set of return types every `fn` of that name declares."""
    for match in FN_NAME.finditer(masked):
        i = match.end()
        while i < len(masked) and masked[i].isspace():
            i += 1
        if i < len(masked) and masked[i] == "<":  # generic parameters
            i = balanced(masked, i, "<", ">")
        while i < len(masked) and masked[i].isspace():
            i += 1
        if i >= len(masked) or masked[i] != "(":
            continue  # not a definition: a `fn` type, or a trait bound
        i = balanced(masked, i, "(", ")")
        while i < len(masked) and masked[i].isspace():
            i += 1
        if masked[i : i + 2] != "->":
            out.setdefault(match.group(1), set()).add("()")
            continue
        i += 2
        end, depth = i, 0
        while end < len(masked):
            char = masked[end]
            if char in "<([":
                depth += 1
            elif char in ">)]":
                depth -= 1
            elif depth == 0 and (char in "{;" or masked[end : end + 6] == "where "):
                break
            end += 1
        out.setdefault(match.group(1), set()).add(masked[i:end].strip())


def is_place(ty, deref_mut):
    """Whether `.field` on a value of this type resolves to somewhere real."""
    return (
        ty.startswith("&")
        or ty.startswith("*")
        # `Self` in an inherent impl of a newtype; the impl block's own type is
        # not in reach of a name-keyed scan, and every one of these in the tree
        # is a handle.
        or ty == "Self"
        or ty in deref_mut
    )


def place_writes(tree):
    """`accessor().field = …` where the accessor answers by value."""
    returns = {}
    deref_mut = set()
    aliases = {}
    for masked in tree.values():
        deref_mut.update(DEREF_MUT.findall(masked))
        aliases.update(TYPE_ALIAS.findall(masked))
        fn_returns(masked, returns)
    # Chase aliases to a fixed point: `Op` is a place because `Live<T>` is.
    for _ in range(len(aliases)):
        grown = {name for name, base in aliases.items() if base in deref_mut}
        if grown <= deref_mut:
            break
        deref_mut |= grown
    found = []
    for file, masked in sorted(tree.items()):
        for match in PLACE_WRITE.finditer(masked):
            declared = returns.get(match.group(1))
            if declared is None or any(is_place(t, deref_mut) for t in declared):
                continue
            line = masked.count("\n", 0, match.start()) + 1
            found.append(
                f"  {file}:{line}: {match.group(1)}() answers with "
                f"{' or '.join(sorted(declared))}"
            )
    return found


def mutating_methods(tree):
    """Method names that can only ever be `&mut self`, plus the refcount three."""
    mutating, other = set(), set()
    for masked in tree.values():
        mutating.update(MUT_SELF_FN.findall(masked))
        other.update(NON_MUT_SELF_FN.findall(masked))
    return (mutating - other) | REFCOUNT_METHODS


def deref_temporary_mutations(tree):
    """`unsafe { *p }.retain()` -- a mutation of the block's temporary."""
    names = mutating_methods(tree)
    found = []
    for file, masked in sorted(tree.items()):
        for match in DEREF_METHOD.finditer(masked):
            if match.group(1) not in names:
                continue
            line = masked.count("\n", 0, match.start()) + 1
            found.append(f"  {file}:{line}: .{match.group(1)}()")
    return found


def check_deref_temporary_mutations(tree):
    if found := deref_temporary_mutations(tree):
        sys.exit(
            "ratchet: a mutating method called on the value an `unsafe` block "
            "produced mutates a *temporary* -- `&mut self` autorefs the copy "
            "the dereference made, and the change is discarded:\n"
            + "\n".join(found)
            + "\nThe region has to cover the call: `unsafe { (*p).count."
            "retain() }`, or wrap the pointer in a `winlayer::Live<T>` once."
        )


def borrowed_derefs(tree):
    """`&mut unsafe { *p }` -- a borrow of a copy of the pointee."""
    found = []
    for file, masked in sorted(tree.items()):
        for match in BORROWED_DEREF.finditer(masked):
            line = masked.count("\n", 0, match.start()) + 1
            found.append(f"  {file}:{line}: {match.group(0).strip()}...")
    return found


def check_borrowed_derefs(tree):
    if found := borrowed_derefs(tree):
        sys.exit(
            "ratchet: a borrow of an `unsafe` block that dereferences a "
            "pointer binds to a *copy* -- an `unsafe` block is a value "
            "expression, so the borrow names a temporary and every write "
            "through it is discarded:\n"
            + "\n".join(found)
            + "\nWrap the pointer once instead -- `winlayer::Live<T>` and the "
            "`Win`/`Buf` handles exist for this -- or, where the borrow really "
            "must be one, put it inside the block: `unsafe { &mut *p }`."
        )


def check_place_writes(tree):
    if found := place_writes(tree):
        sys.exit(
            "ratchet: a write through an accessor that answers by value is a "
            "silent no-op — the assignment lands in a temporary:\n"
            + "\n".join(found)
            + "\nTake `&mut`/`*mut` from the accessor, or write through the "
            "cell the accessor reads."
        )


IDENT = re.compile(r"[A-Za-z0-9_]")
IDENT_AT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
WHITESPACE = re.compile(r"\s*")
UNSAFE_WORD = re.compile(r"\bunsafe\b")
# Literal prefixes that make a following `#`/`"` open a raw string.
RAW_PREFIXES = {"r", "br", "cr", "rb", "rc"}


def mask(text):
    """The source with comments, strings and char literals blanked to spaces.

    Offsets and newlines are preserved, so the result can be scanned
    structurally (brace matching, keyword search) and mapped back to line
    numbers, while nothing inside a literal or a comment can be mistaken for
    code.
    """
    out = list(text)
    n = len(text)
    i = 0

    def blank(start, stop):
        for k in range(start, stop):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if IDENT.match(c):
            # Consume whole identifiers so `r` inside one can't open a raw
            # string, and so `b'x'` reaches the char-literal branch below.
            j = i
            while j < n and IDENT.match(text[j]):
                j += 1
            if text[i:j] in RAW_PREFIXES and j < n and text[j] in '#"':
                k = j
                while k < n and text[k] == "#":
                    k += 1
                if k < n and text[k] == '"':
                    close = text.find('"' + "#" * (k - j), k + 1)
                    j = n if close < 0 else close + 1 + (k - j)
                    blank(i, j)
            i = j
        elif c == "/" and text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            j = min(j + 1, n)
            blank(i, j)
            i = j
        elif c == "'":
            # A char literal, or a lifetime — `'a` looks like the start of
            # one until the closing quote fails to show up.
            if i + 1 < n and text[i + 1] == "\\":
                # The escape consumes exactly one character, so the earliest
                # the closing quote can sit is i+3 -- and starting the scan
                # there is what makes `'\\'` terminate. Treating the second
                # backslash as an escape instead (`j += 2`) walked straight
                # past the closing quote and blanked source as far as the
                # next quote in the file, taking its braces with it.
                j = i + 3
                while j < n and text[j] != "'":
                    j += 1
                j = min(j + 1, n)
            elif i + 2 < n and text[i + 2] == "'":
                j = i + 3
            else:
                i += 1
                continue
            blank(i, j)
            i = j
        else:
            i += 1
    return "".join(out)


def matching_brace(masked, open_at):
    """Offset of the `}` closing the `{` at open_at (end of text if unpaired)."""
    depth = 0
    for i in range(open_at, len(masked)):
        if masked[i] == "{":
            depth += 1
        elif masked[i] == "}":
            depth -= 1
            if not depth:
                return i
    return len(masked) - 1


def unsafe_lines(masked, deny):
    """Lines of code the compiler is not checking. See the module docs."""
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]

    def lineno(offset):
        """0-based line holding `offset`."""
        return bisect_right(starts, offset) - 1

    # Lines whose masked content is blank hold no code: comments (SAFETY
    # notes included) and empty lines inside a block are free.
    code_lines = {i for i, line in enumerate(masked.splitlines()) if line.strip()}

    covered = set()
    for match in UNSAFE_WORD.finditer(masked):
        at = WHITESPACE.match(masked, match.end()).end()
        word = IDENT_AT.match(masked, at)
        keyword = word.group(0) if word else ""
        body_at = None
        if at < len(masked) and masked[at] == "{":
            body_at = at  # `unsafe { ... }`
        elif keyword == "extern":
            after = WHITESPACE.match(masked, word.end()).end()
            follows = IDENT_AT.match(masked, after)
            if after < len(masked) and masked[after] == "{":
                body_at = after  # `unsafe extern "C" { ... }`
            elif follows and follows.group(0) == "fn":
                word = follows  # `unsafe extern "C" fn ...`
                keyword = "fn"
            else:
                continue
        elif keyword in ("impl", "trait"):
            covered.add(lineno(match.start()))
            continue
        elif keyword != "fn":
            continue  # `unsafe(no_mangle)` and friends: not a region

        if body_at is None:
            named = WHITESPACE.match(masked, word.end()).end()
            if named < len(masked) and masked[named] == "(":
                continue  # an `unsafe fn(..)` *type*; paid at the call site
            if deny:
                continue  # the body's own blocks state its unsafe surface
            brace = masked.find("{", named)
            semi = masked.find(";", named)
            if brace < 0 or 0 <= semi < brace:
                covered.add(lineno(match.start()))  # a bodyless declaration
                continue
            body_at = brace

        span = range(lineno(match.start()), lineno(matching_brace(masked, body_at)) + 1)
        covered.update(span)
    return len(covered & code_lines)


def test_module_lines(masked):
    """The 0-based line numbers a `#[cfg(test)] mod name { .. }` item spans.

    Attribute line through closing brace, both included -- the whole item is
    the thing the cap looks away from. Spans are unioned, so a test module
    nested inside another counts once.
    """
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]
    covered = set()
    for match in CFG_TEST_MOD.finditer(masked):
        first = bisect_right(starts, match.start()) - 1
        last = bisect_right(starts, matching_brace(masked, match.end() - 1)) - 1
        covered.update(range(first, last + 1))
    return covered


def has_safety_doc(lines, at):
    """Whether a `# Safety` heading sits in the doc comment above line `at`.

    Attribute lines are skipped: they sit between the doc comment and the item.
    So are plain `//` comments, which sit there too — a note to the next
    reader that is deliberately not part of the rendered docs (`// keep the
    export: <spec> still resolves it`, say). Counting one of those as the end
    of the doc comment scored the item as undocumented while its `# Safety`
    section sat two lines above.

    A doc comment hidden behind an attribute rustfmt wrapped over several lines
    would read as absent, which over-counts — the direction that keeps the
    ratchet honest.
    """
    i = at - 1
    while i >= 0 and (
        lines[i].lstrip().startswith("#")
        or (lines[i].lstrip().startswith("//") and not DOC_LINE.match(lines[i]))
    ):
        i -= 1
    while i >= 0 and DOC_LINE.match(lines[i]):
        if SAFETY_HEADING.match(lines[i]):
            return True
        i -= 1
    return False


def missing_safety_doc(text, masked):
    """`unsafe fn`s whose doc comment has no `# Safety` section.

    Walks the same `unsafe` keyword occurrences `unsafe_lines` does and keeps
    the ones that introduce a *function* — a definition or a bodyless
    declaration, never a function-pointer type and never a declaration inside
    an `unsafe extern` block, whose obligation is the C library's.
    """
    lines = text.splitlines()
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]
    missing = 0
    skip_until = 0
    for match in UNSAFE_WORD.finditer(masked):
        if match.start() < skip_until:
            continue
        at = WHITESPACE.match(masked, match.end()).end()
        word = IDENT_AT.match(masked, at)
        keyword = word.group(0) if word else ""
        if keyword == "extern":
            after = WHITESPACE.match(masked, word.end()).end()
            follows = IDENT_AT.match(masked, after)
            if after < len(masked) and masked[after] == "{":
                skip_until = matching_brace(masked, after)
                continue
            if not (follows and follows.group(0) == "fn"):
                continue
            word = follows  # `unsafe extern "C" fn ...`
        elif keyword != "fn":
            continue  # `unsafe {`, `unsafe impl`, `unsafe(no_mangle)`, ...
        named = WHITESPACE.match(masked, word.end()).end()
        if named < len(masked) and masked[named] == "(":
            continue  # an `unsafe fn(..)` *type*; it has no docs to carry
        missing += not has_safety_doc(lines, bisect_right(starts, match.start()) - 1)
    return missing


def measure():
    """(repo-relative file -> {metric: count} with zeros included,
    number of files not carrying the forbid attribute,
    number of files carrying neither forbid nor the unsafe-op deny,
    number of files not carrying the cast deny,
    repo-relative file -> its masked source, for the whole-tree checks)."""
    stats = {}
    tree = {}
    without_forbid = 0
    without_deny = 0
    without_casts = 0
    for path in sorted(
        [*ROOT.glob("crates/*/src/**/*.rs"), *ROOT.glob("crates/*/*.rs")]
    ):
        text = path.read_text()
        masked = mask(text)
        counts = {
            name: sum(masked.count(needle) for needle in needles)
            for name, needles in COUNTED.items()
        }
        counts.update(
            (name, len(rx.findall(masked))) for name, rx in COUNTED_RE.items()
        )
        counts["unsafe_lines"] = unsafe_lines(masked, DENY_UNSAFE_OP in masked)
        counts["missing_safety_doc"] = missing_safety_doc(text, masked)
        counts["lines"] = len(text.splitlines()) - len(test_module_lines(masked))
        stats[str(path.relative_to(ROOT))] = counts
        tree[str(path.relative_to(ROOT))] = masked
        without_forbid += FORBID not in masked
        without_deny += FORBID not in masked and DENY_UNSAFE_OP not in masked
        without_casts += DENY_CASTS.search(masked) is None
    return stats, without_forbid, without_deny, without_casts, tree


def ledgers():
    """The two whole-tree counts the committed ledgers carry."""
    for path in (LEDGER, VISIBILITY):
        if not path.exists():
            sys.exit(
                f"ratchet: {path.relative_to(ROOT)} is missing; run `just refresh`"
            )
    return {
        "internal_exports": sum(
            json.loads(line)["class"] == "internal"
            for line in LEDGER.read_text().splitlines()
        ),
        "test_reached_pub": len(VISIBILITY.read_text().splitlines()),
    }


def cell_ptr_receivers(tree):
    """receiver name -> how many `.ptr()`/`.as_raw()` sites it has, tree-wide.

    A site whose receiver is not a bare identifier is absent from this, which
    is what `cell_ptr_partition` reconciles against the `cell_ptr` total.
    """
    seen = collections.Counter()
    for masked in tree.values():
        seen.update(match.group(1) for match in CELL_PTR_SITE.finditer(masked))
    return seen


def cell_ptr_partition(stats, tree):
    """The two ratcheted `cell_ptr` classes. See the doc block.

    The boundary is subtracted rather than counted, so the two returned
    numbers plus the boundary's own sites are exactly the `cell_ptr` total.
    """
    seen = cell_ptr_receivers(tree)
    total = sum(counts["cell_ptr"] for counts in stats.values())
    boundary = sum(seen[name] for name in CELL_PTR_ALLOW)
    keepers = sum(seen[name] for name in CELL_PTR_KEEPERS)
    return {
        "cell_ptr_keepers": keepers,
        "cell_ptr_accessors": total - boundary - keepers,
    }


def in_perimeter_entry(file, entry):
    """Whether one perimeter entry claims this repo-relative path.

    A directory entry (trailing `/`) claims its whole subtree; anything else
    is one exact path, so a sibling whose name merely starts with an entry is
    not caught.
    """
    return file.startswith(entry) if entry.endswith("/") else file == entry


def in_perimeter(file):
    """Whether a repo-relative path is on the unsafe perimeter."""
    return any(in_perimeter_entry(file, entry) for entry in PERIMETER)


def perimeter_lines(stats):
    """(unchecked lines inside the perimeter, unchecked lines outside it)."""
    inside = sum(c["unsafe_lines"] for f, c in stats.items() if in_perimeter(f))
    return inside, sum(c["unsafe_lines"] for c in stats.values()) - inside


def check_perimeter(stats):
    """Every perimeter entry still has unchecked code behind it.

    An entry matching nothing is stale: it would sit there excusing a module
    that has finished, moved or gone away, and would quietly excuse it again
    if the path came back. A module reaching zero unchecked lines is the
    outcome the list is for, and it is an outcome that has to say so by
    leaving the list.
    """
    if stale := sorted(
        entry
        for entry in PERIMETER
        if not any(
            counts["unsafe_lines"] and in_perimeter_entry(file, entry)
            for file, counts in stats.items()
        )
    ):
        sys.exit(
            "ratchet: these perimeter entries have no unchecked code behind "
            "them:\n  "
            + "\n  ".join(stale)
            + "\nDrop each from PERIMETER (and from docs/perimeter.md) and "
            "run `just refresh` to lock the progress in."
        )


def whole_tree(stats, tree):
    """The name-keyed whole-tree counts. See the doc block."""
    return {
        **cell_ptr_partition(stats, tree),
        "unsafe_lines_outside_perimeter": perimeter_lines(stats)[1],
        "cell_copy_owner": sum(
            len(CELL_COPY_OWNER_RE.findall(m)) for m in tree.values()
        ),
    }


def check_cell_ptr(tree):
    """The accessor cap, and the keeper register's freshness.

    One `.ptr()` per receiver is the acquire-once shape phase 22 converged
    on; a second one means a family has started working from the address
    again, which is the regression the partition exists to forbid. Ratcheting
    the count alone would not catch it -- nine new sites on one cell and nine
    retired elsewhere is a flat number and a real regression.

    A register entry matching nothing is stale: it would sit there exempting
    a name the tree no longer uses, and quietly exempt it again if the name
    came back. Reducing a listed global to one site is progress, and it is
    progress that has to say so by moving the entry out of the register.
    """
    seen = cell_ptr_receivers(tree)
    listed = {*CELL_PTR_ALLOW, *CELL_PTR_KEEPERS}
    if over := sorted(
        (name, n) for name, n in seen.items() if n > 1 and name not in listed
    ):
        sys.exit(
            "ratchet: a cell may hand out at most one raw pointer, from one "
            "named accessor. These receivers have more:\n  "
            + "\n  ".join(f"{name}: {n} sites" for name, n in over)
            + "\nNarrow them to a single accessor, or -- if a slice has "
            "ruled the address is what the family works from -- add the name "
            "to CELL_PTR_KEEPERS with the ruling."
        )
    if stale := sorted(name for name in CELL_PTR_KEEPERS if not seen[name]):
        sys.exit(
            "ratchet: these names are in a cell_ptr register but have no "
            "sites left:\n  "
            + "\n  ".join(stale)
            + "\nDrop the entry and run `just refresh` to lock the progress in."
        )


def check_names(tree):
    """Every allowlisted name still declares a cell.

    The lists are keyed by name, so a rename would quietly widen the
    `cell_ptr` allowlist or quietly drop a `cell_copy_owner` floor. Neither
    list is large enough for that to be caught by reading the diff, so the
    run asserts it.

    CELL_PTR_KEEPERS is deliberately not checked here: two of its entries
    (`cell`, `SCRATCH`) do not name one static apiece, and a renamed keeper
    is caught anyway -- its sites land in the accessor class and trip the
    one-site cap in `check_cell_ptr`.
    """
    missing = [
        name
        for name in (*CELL_PTR_ALLOW, *CELL_COPY_OWNER)
        if not any(re.search(CELL_DECL.format(name), m) for m in tree.values())
    ]
    if missing:
        sys.exit(
            "ratchet: these names are on a cell allowlist but no longer "
            "declare a GlobalCell/SharedCell static:\n  "
            + "\n  ".join(missing)
            + "\nRename them in CELL_PTR_ALLOW/CELL_COPY_OWNER, or drop "
            "the entry (and lower the baseline) if "
            "the global is gone."
        )


def render(stats, ledger_counts, without_forbid, without_deny, without_casts):
    """The baseline document: only metrics with ratchet room are recorded
    (nonzero counts, over-cap line counts), so files that are already clean
    and under the cap don't churn the file as they're edited."""
    entries = []
    for file, counts in sorted(stats.items()):
        kept = {
            name: n
            for name, n in counts.items()
            if n > (LINE_CAP if name == "lines" else 0)
        }
        if kept:
            entries.append(
                f"    {json.dumps(file)}: {json.dumps(kept, sort_keys=True)}"
            )
    body = ",\n".join(entries)
    return (
        "{\n"
        f'  "internal_exports": {ledger_counts["internal_exports"]},\n'
        f'  "test_reached_pub": {ledger_counts["test_reached_pub"]},\n'
        f'  "cell_ptr_keepers": {ledger_counts["cell_ptr_keepers"]},\n'
        f'  "cell_ptr_accessors": {ledger_counts["cell_ptr_accessors"]},\n'
        f'  "cell_copy_owner": {ledger_counts["cell_copy_owner"]},\n'
        f'  "unsafe_lines_outside_perimeter": '
        f"{ledger_counts['unsafe_lines_outside_perimeter']},\n"
        f'  "files_without_forbid_unsafe": {without_forbid},\n'
        f'  "files_without_deny_unsafe_op": {without_deny},\n'
        f'  "files_without_deny_casts": {without_casts},\n'
        f'  "files": {{\n{body}\n  }}\n'
        "}\n"
    )


# The whole-tree counts, and how a violation of each reads.
WHOLE_TREE_LABEL = {
    "internal_exports": "abi-ledger internal exports",
    "test_reached_pub": "test-reached pub items",
    "cell_ptr_keepers": "cell_ptr sites on a ruled multi-site keeper",
    "cell_ptr_accessors": "one-per-cell acquire-once cell_ptr sites",
    "cell_copy_owner": "get() copies of a Copy global owning a pointer",
    "unsafe_lines_outside_perimeter": "unchecked lines outside the unsafe perimeter",
}


def violations(stats, counts, without_forbid, without_deny, without_casts, baseline):
    """Every metric that grew past the committed baseline."""
    found = []
    for name, label in WHOLE_TREE_LABEL.items():
        # .get: absent from baselines committed before the metric existed.
        base = baseline.get(name, counts[name])
        if counts[name] > base:
            found.append(f"{label}: {base} -> {counts[name]}")
    # .get: absent from baselines committed before the metric existed.
    base_forbid = baseline.get("files_without_forbid_unsafe", without_forbid)
    if without_forbid > base_forbid:
        found.append(f"files without {FORBID}: {base_forbid} -> {without_forbid}")
    base_deny = baseline.get("files_without_deny_unsafe_op", without_deny)
    if without_deny > base_deny:
        found.append(
            f"files without {FORBID} or {DENY_UNSAFE_OP}: {base_deny} -> {without_deny}"
        )
    base_casts = baseline.get("files_without_deny_casts", without_casts)
    if without_casts > base_casts:
        found.append(f"files without the cast deny: {base_casts} -> {without_casts}")
    base_files = baseline["files"]
    counted = (*COUNTED, *COUNTED_RE, *DERIVED)
    for file in sorted(stats.keys() | base_files.keys()):
        cur = stats.get(file, {**dict.fromkeys(counted, 0), "lines": 0})
        base = base_files.get(file, {})
        for name in counted:
            if cur[name] > base.get(name, 0):
                found.append(f"{file}: {name} {base.get(name, 0)} -> {cur[name]}")
        limit = max(LINE_CAP, base.get("lines", 0))
        if cur["lines"] > limit:
            grandfathered = " (grandfathered)" if limit > LINE_CAP else ""
            found.append(f"{file}: {cur['lines']} lines > {limit}{grandfathered}")
    return found


def summary(stats, counts, without_forbid, without_deny, without_casts):
    counted = (*COUNTED, *COUNTED_RE, *DERIVED)
    totals = {name: sum(c[name] for c in stats.values()) for name in counted}
    over = sum(c["lines"] > LINE_CAP for c in stats.values())
    parts = [f"{n} {name}" for name, n in totals.items()]
    parts += [
        f"{over} files over {LINE_CAP} lines",
        f"{counts['internal_exports']} internal exports",
        f"{counts['test_reached_pub']} test-reached pub items",
        f"{counts['cell_ptr_keepers']} keeper cell_ptr sites",
        f"{counts['cell_ptr_accessors']} acquire-once cell_ptr sites",
        f"{counts['cell_copy_owner']} Copy-owner get()s",
        f"{counts['unsafe_lines_outside_perimeter']} unchecked lines outside the "
        f"perimeter ({perimeter_lines(stats)[0]} inside)",
        f"{without_forbid} files without forbid(unsafe_code)",
        f"{without_deny} files also without deny(unsafe_op_in_unsafe_fn)",
        f"{without_casts} files without the cast deny",
    ]
    return ", ".join(parts)


# Scanner cases, checked on every run (a few hundred microseconds against a
# ~20 MB tree read). A silent regression in mask()/unsafe_lines() would
# corrupt every number the ratchet enforces, so this is not opt-in.
SELF_TEST = [
    # (source, expected unsafe_lines in a file without the deny)
    ("fn f() {\n    unsafe {\n        g();\n    }\n}\n", 3),
    ("fn f() {\n    let x = unsafe { *p };\n}\n", 1),
    # Comments and blank lines inside a block are free.
    ("fn f() {\n    unsafe {\n        // SAFETY: fine.\n\n        g();\n    }\n}\n", 3),
    # Prose and strings never count.
    ("/// An unsafe fn would need one.\n/// unsafe { }\nfn f() {}\n", 0),
    ('fn f() {\n    let s = "unsafe { g(); }";\n}\n', 0),
    ('fn f() {\n    let s = r#"unsafe {"#;\n}\n', 0),
    ("/* unsafe { /* nested */ } */\nfn f() {}\n", 0),
    # A brace in a char literal must not desynchronise the scanner.
    ("fn f() {\n    unsafe {\n        g('{');\n    }\n    h();\n}\n", 3),
    # ... and neither must an escaped one. `'\\'` ends at its own quote; a
    # scanner that reads the second backslash as an escape runs on to the
    # next quote in the file and eats every brace in between.
    ("fn f() {\n    unsafe {\n        g('\\\\');\n    }\n    h('x');\n}\n", 3),
    ("fn f() {\n    unsafe {\n        g('\\'');\n    }\n    h('x');\n}\n", 3),
    ("fn f() {\n    unsafe {\n        g('\\u{1b}');\n    }\n    h('x');\n}\n", 3),
    ("fn f<'a>(x: &'a u8) {}\n", 0),
    # An unsafe fn body is implicitly unsafe throughout without the deny.
    ("unsafe fn f() {\n    g();\n}\n", 3),
    ('unsafe extern "C" fn f() {\n    g();\n}\n', 3),
    ("trait T {\n    unsafe fn f();\n}\n", 1),
    # Nested blocks are unioned, not summed.
    ("unsafe fn f() {\n    unsafe {\n        g();\n    }\n}\n", 5),
    # Declarations, promises, types.
    ("unsafe impl Sync for X {}\n", 1),
    ("unsafe trait T {}\n", 1),
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ("struct S(Option<unsafe fn(u8)>);\n", 0),
    ('#[unsafe(no_mangle)]\npub extern "C" fn f() {}\n', 0),
    ('unsafe extern "C" {\n    static x: u8;\n}\n', 3),
]
# (source, expected extern_abi) — the needle is a regex over masked source,
# so it needs its own cases: masking has already erased the ABI string by the
# time it runs.
SELF_TEST_EXTERN_ABI = [
    ('pub unsafe extern "C" fn f() {}\n', 1),
    ('extern "C-unwind" fn f() {}\n', 1),
    ("extern fn f() {}\n", 1),
    # A function-pointer type is not a definition.
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ('struct S(Option<extern "C-unwind" fn(u8)>);\n', 0),
    # Neither is a declaration inside an extern block.
    ('unsafe extern "C" {\n    fn f(x: u8);\n}\n', 0),
    # Prose about one costs nothing.
    ('/// An extern "C" fn f() would.\nfn f() {}\n', 0),
]
# (source, expected missing_safety_doc). Reads the raw text as well as the
# masked copy, since the heading it looks for lives in a comment.
SELF_TEST_SAFETY_DOC = [
    ("unsafe fn f() {}\n", 1),
    ("/// # Safety\n/// Anything.\nunsafe fn f() {}\n", 0),
    ("/// # safety\nunsafe fn f() {}\n", 0),
    ("/// ## Safety\nunsafe fn f() {}\n", 0),
    # The section has to be this item's, not the one above it.
    ("/// # Safety\nunsafe fn f() {}\nunsafe fn g() {}\n", 1),
    # A blank line between ends the doc comment, so the heading is not f's.
    ("/// # Safety\n\nunsafe fn f() {}\n", 1),
    # Attributes sit between the comment and the item.
    ('/// # Safety\n#[unsafe(no_mangle)]\npub unsafe extern "C" fn f() {}\n', 0),
    # ... and so may a plain `//` note, which is not part of the docs.
    ("/// # Safety\n// Keep the export.\n#[unsafe(no_mangle)]\nunsafe fn f() {}\n", 0),
    ("/// # Safety\n// A note.\nunsafe fn f() {}\n", 0),
    # A `//` note that is the *only* thing above still leaves it undocumented.
    ("// A note.\nunsafe fn f() {}\n", 1),
    # A note after the previous item does not reach back past it.
    ("/// # Safety\nunsafe fn f() {}\n// A note.\nunsafe fn g() {}\n", 1),
    # A trait's declaration carries the obligation too.
    ("trait T {\n    unsafe fn f();\n}\n", 1),
    # ... but a C library's does not.
    ('unsafe extern "C" {\n    unsafe fn f(x: u8);\n}\n', 0),
    ('unsafe extern "C" {\n    fn f(x: u8);\n}\n', 0),
    # Neither a function-pointer type nor a promise is a function.
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ("unsafe impl Sync for X {}\n", 0),
    ("fn f() {\n    unsafe {\n        g();\n    }\n}\n", 0),
    # Prose about one costs nothing.
    ("/// An unsafe fn f would.\nfn f() {}\n", 0),
]
# (source, whether the file counts as having adopted the cast lints)
SELF_TEST_DENY_CASTS = [
    ("#![deny(clippy::cast_lossless)]\n", True),
    (
        "#![deny(\n    clippy::cast_lossless,\n    clippy::ptr_as_ptr\n)]\n",
        True,
    ),
    ("#![deny(clippy::ptr_as_ptr)]\n", False),
    # Prose about the attribute does not switch it on.
    ("//! Adopt `#![deny(clippy::cast_lossless)]` here one day.\n", False),
]
# (source, expected number of lines exempted from the line cap)
SELF_TEST_TEST_MODULE = [
    ("#[cfg(test)]\nmod tests {\n    fn t() {}\n}\n", 4),
    ("fn f() {}\n#[cfg(test)]\nmod tests {\n    fn t() {}\n}\nfn g() {}\n", 4),
    # Visibility and extra attributes sit between the two.
    ("#[cfg(test)]\npub(crate) mod tests {\n}\n", 3),
    ("#[cfg(test)]\n#[allow(clippy::all)]\nmod tests {\n}\n", 4),
    ("#[cfg(test)]\nmod tests {\n}\n#[cfg(test)]\nmod more {\n}\n", 6),
    # A nested test module is inside the outer span, not counted twice.
    ("#[cfg(test)]\nmod tests {\n    #[cfg(test)]\n    mod inner {\n    }\n}\n", 6),
    # A declaration names another file, which is measured on its own.
    ("#[cfg(test)]\nmod tests;\n", 0),
    # Everything else keeps its lines: another cfg, and production code.
    ("#[cfg(unix)]\nmod unix {\n}\n", 0),
    ("mod tests {\n}\n", 0),
    # Prose about one costs nothing, and neither does a string holding it.
    ("// #[cfg(test)]\n// mod tests {\nfn f() {}\n", 0),
    ('fn f() {\n    let s = "#[cfg(test)] mod tests {";\n}\n', 0),
]
SELF_TEST_DENY = [
    # With the deny, a body's own blocks state its unsafe surface.
    ("unsafe fn f() {\n    g();\n}\n", 0),
    ("unsafe fn f() {\n    unsafe {\n        g();\n    }\n}\n", 3),
]
# (source, expected number of by-value accessor writes)
SELF_TEST_PLACE_WRITE = [
    # The shape that shipped three silent no-ops: a value accessor, written to.
    ("fn a() -> E {\n    E\n}\nfn f() {\n    a().x = 1;\n}\n", 1),
    ("fn a() -> E {\n    E\n}\nfn f() {\n    a().x += 1;\n}\n", 1),
    # Accessors that answer with a place.
    ("fn a() -> &mut E {\n}\nfn f() {\n    a().x = 1;\n}\n", 0),
    ("fn a() -> *mut E {\n}\nfn f() {\n    a().x = 1;\n}\n", 0),
    # A handle newtype: `.x` reaches through to what it points at.
    ("impl DerefMut for H {}\nfn a() -> H {\n}\nfn f() {\n    a().x = 1;\n}\n", 0),
    (
        "impl core::ops::DerefMut for H {}\nfn a() -> H {}\nfn f() {\n    a().x = 1;\n}\n",
        0,
    ),
    # A comparison is not a write, and neither is a call with arguments.
    ("fn a() -> E {}\nfn f() {\n    if a().x == 1 {}\n}\n", 0),
    ("fn a(n: c_int) -> E {}\nfn f() {\n    a(1).x = 1;\n}\n", 0),
    # An unknown name is left alone: it is someone else's method.
    ("fn f() {\n    unknown().x = 1;\n}\n", 0),
    # Prose about one costs nothing.
    ("fn a() -> E {}\n// a().x = 1;\n", 0),
]
# (source, expected borrowed_derefs)
SELF_TEST_BORROWED_DEREF = [
    # The shapes S16 shipped, all silent.
    ("fn f() {\n    let d = &mut unsafe { *dsp };\n}\n", 1),
    ("fn f() {\n    let d = &mut unsafe { (*p).field };\n}\n", 1),
    ("fn f() {\n    let d = &raw mut unsafe { (*p).field };\n}\n", 1),
    ("fn f() {\n    let d = &raw const unsafe { (*p).field };\n}\n", 1),
    ("fn f() {\n    g(&mut unsafe { *(cookie as *mut C) });\n}\n", 1),
    # A *shared* `&unsafe { *p }` is left alone on purpose: `&` is also the
    # binary operator and `&&` its neighbour, so the needle cannot tell a
    # borrow from a mask without parsing, and a shared borrow of a copy is a
    # waste rather than a lost write. `mut`/`raw` can only follow a borrow.
    ("fn f() {\n    if a & unsafe { *p } != 0 {}\n}\n", 0),
    ("fn f() {\n    if a && unsafe { *p } {}\n}\n", 0),
    # Borrowing what a call *returned* is a real value, not a copy of a
    # pointee, and is how three sites in the tree are written.
    ("fn f() {\n    owned(&mut unsafe { render_char(buf, c) })\n}\n", 0),
    # The sound spellings: the borrow is inside the block, or there is none.
    ("fn f() {\n    let d = unsafe { &mut *dsp };\n}\n", 0),
    ("fn f() {\n    let d = unsafe { Live::new(dsp) };\n}\n", 0),
    # Prose about one costs nothing.
    ("// let d = &mut unsafe { *dsp };\n", 0),
]
# (source, expected deref_temporary_mutations)
SELF_TEST_DEREF_MUTATION = [
    # The shape batch 3 wrote twelve times: a reference given away for free.
    (
        "impl R {\n    fn retain(&mut self) {}\n}\nfn f() {\n"
        "    unsafe { (*fp).uf_refcount }.retain();\n}\n",
        1,
    ),
    # `release` shares its name with a consuming `release(self)`, so it is
    # named outright rather than left to the exclusivity rule.
    (
        "impl S {\n    fn release(self) -> *mut u8 {}\n}\nfn f() {\n"
        "    unsafe { (*fp).uf_refcount }.release();\n}\n",
        1,
    ),
    # A shared method on the copy reads the same bytes: not this bug.
    (
        "impl P {\n    fn is_null(&self) -> bool {}\n}\nfn f() {\n"
        "    if unsafe { (*ac).pat }.is_null() {}\n}\n",
        0,
    ),
    # A name that is `&mut self` on one type and `&self` on another cannot be
    # judged by name, so it is left alone.
    (
        "impl A {\n    fn has(&mut self, f: c_int) -> bool {}\n}\n"
        "impl B {\n    fn has(&self, f: c_int) -> bool {}\n}\nfn f() {\n"
        "    if unsafe { (*args).os_flags }.has(LOCAL) {}\n}\n",
        0,
    ),
    # The sound spelling: the region covers the call.
    (
        "impl R {\n    fn retain(&mut self) {}\n}\nfn f() {\n"
        "    unsafe { (*fp).uf_refcount.retain() };\n}\n",
        0,
    ),
    # Prose about one costs nothing.
    (
        "impl R {\n    fn retain(&mut self) {}\n}\n"
        "// unsafe { (*fp).uf_refcount }.retain();\n",
        0,
    ),
]
# (source, expected pub_items)
SELF_TEST_PUB_ITEMS = [
    ("pub fn f() {}\n", 1),
    ("pub unsafe fn f() {}\n", 1),
    ('pub unsafe extern "C" fn f() {}\n', 1),
    ("pub static mut X: c_int = 0;\n", 1),
    ("pub const X: c_int = 0;\n", 1),
    ("pub struct S;\npub enum E {}\npub union U {}\n", 3),
    ("pub trait T {}\npub type A = c_int;\npub mod m;\n", 3),
    # A re-export is public surface however many leaves it names.
    ("pub use self::a::{b, c};\n", 1),
    # Anything narrower is not the crate's boundary.
    ("pub(crate) fn f() {}\npub(super) fn g() {}\npub(in crate::a) fn h() {}\n", 0),
    # Indented: an associated item or an inline module, not nameable by path.
    ("impl S {\n    pub fn f() {}\n}\n", 0),
    ("mod m {\n    pub const X: c_int = 0;\n}\n", 0),
    # Prose about one costs nothing, and neither does a string holding it.
    ("// pub fn f() {}\n", 0),
    ('fn f() {\n    let s = "pub fn g() {}";\n}\n', 0),
    # `pubx` is not `pub`.
    ("pubfn f() {}\n", 0),
]

# (source, expected allowlisted sites). The needle runs over masked source
# and subtracts from `cell_ptr`, so it must match exactly what `cell_ptr`
# matched — receiver-blind `.ptr()`/`.as_raw()`, with the receiver pinned.
SELF_TEST_CELL_PTR_ALLOW = [
    ("fn f() {\n    main_loop.ptr();\n}\n", 1),
    ("fn f() {\n    MUTEX.as_raw();\n}\n", 1),
    # Spacing rustfmt never writes is not matched either — the subtraction
    # has to stay a subset of what `cell_ptr`'s substring needle counted.
    ("fn f() {\n    main_loop . ptr();\n}\n", 0),
    # A field or a method of the same name is not the global.
    ("fn f() {\n    loop_.main_loop.ptr();\n}\n", 1),
    ("fn f() {\n    x.not_main_loop.ptr();\n}\n", 0),
    # Another global's site is exactly what the metric is left holding.
    ("fn f() {\n    curbuf.ptr();\n}\n", 0),
    # Prose about one costs nothing.
    ("// main_loop.ptr()\nfn f() {}\n", 0),
]
# (source, expected sites classified by receiver). `CELL_PTR_SITE` drives the
# partition and the accessor cap, so it has to see exactly the receivers
# `cell_ptr`'s substring needles counted -- no more, and no fewer than the
# ones a by-name cap can police.
SELF_TEST_CELL_PTR_SITE = [
    ("fn f() {\n    POSTFIX.ptr();\n}\n", {"POSTFIX": 1}),
    ("fn f() {\n    MUTEX.as_raw();\n}\n", {"MUTEX": 1}),
    # Two receivers, counted apart.
    ("fn f() {\n    a.ptr();\n    b.ptr();\n    a.ptr();\n}\n", {"a": 2, "b": 1}),
    # A field access answers the field's name, which is what the cap wants:
    # `x.compl_xp.ptr()` is a second site on the same cell however it is
    # spelled.
    ("fn f() {\n    x.compl_xp.ptr();\n}\n", {"compl_xp": 1}),
    # Not a bare identifier: no receiver to cap, and the site still counts in
    # the accessor total because that is `cell_ptr` minus the classified.
    ("fn f() {\n    self.0.ptr();\n}\n", {}),
    # Spacing rustfmt never writes stays out, as it does for the allowlist.
    ("fn f() {\n    a . ptr();\n}\n", {}),
    # Prose about one costs nothing.
    ("// POSTFIX.ptr()\nfn f() {}\n", {}),
]
# (tree, expected partition). The boundary is subtracted, the keeper register
# is counted, and everything left is the accessor class -- including the site
# with no bare-identifier receiver.
SELF_TEST_CELL_PTR_PARTITION = [
    (
        {
            "a.rs": "fn f() {\n    main_loop.ptr();\n    curwin.ptr();\n}\n",
            "b.rs": "fn g() {\n    POSTFIX.ptr();\n    POSTFIX.ptr();\n"
            "    curbuf.ptr();\n    self.0.ptr();\n}\n",
        },
        {"cell_ptr_keepers": 2, "cell_ptr_accessors": 3},
    ),
]
# (tree, whether check_cell_ptr should reject it)
SELF_TEST_CELL_PTR_CHECK = [
    # One site per unlisted receiver is the shape; the register is intact.
    ({"a.rs": "fn f() {\n    curbuf.ptr();\n    curwin.ptr();\n}\n"}, False),
    # A second site on an unlisted receiver is the regression.
    ({"a.rs": "fn f() {\n    curbuf.ptr();\n    curbuf.ptr();\n}\n"}, True),
    # Split across files, which is how a family spreads.
    (
        {
            "a.rs": "fn f() {\n    curbuf.ptr();\n}\n",
            "b.rs": "fn g() {\n    curbuf.as_raw();\n}\n",
        },
        True,
    ),
    # A listed name may hold as many as its ruling allows.
    ({"a.rs": "fn f() {\n    main_loop.ptr();\n    main_loop.ptr();\n}\n"}, False),
    ({"a.rs": "fn f() {\n    POSTFIX.ptr();\n    POSTFIX.ptr();\n}\n"}, False),
]
# (source, expected cell_copy_owner)
SELF_TEST_CELL_COPY_OWNER = [
    ("fn f() {\n    rex.get();\n}\n", 1),
    ("fn f() {\n    let n = runtime_search_path.get().data;\n}\n", 1),
    # Only `get`: the other accessors do not hand out a second owner.
    ("fn f() {\n    rex.set(x);\n}\n", 0),
    ("fn f() {\n    rex.with(|s| s);\n}\n", 0),
    # rustfmt wraps a long chain, and the copy still happens.
    ("fn f() {\n    *runtime_search_path\n        .get()\n}\n", 1),
    # A global that is not on the list.
    ("fn f() {\n    p_ai.get();\n}\n", 0),
    # A longer name ending in a listed one is not it.
    ("fn f() {\n    saved_rex.get();\n}\n", 0),
    ("// rex.get()\nfn f() {}\n", 0),
]


# (path, whether the perimeter claims it). A directory entry claims its
# subtree and nothing else; an exact-path entry claims exactly itself.
SELF_TEST_PERIMETER = [
    ("crates/nvim/src/lua/executor/exec.rs", True),
    ("crates/nvim/src/os/fs/mod.rs", True),
    ("crates/nvim/src/memfile.rs", True),
    ("crates/nvim/src/winlayer/live.rs", True),
    # The editor proper, including modules that merely look raw.
    ("crates/nvim/src/memline/block0.rs", False),
    ("crates/nvim/src/marktree/node.rs", False),
    ("crates/nvim/src/eval/typval.rs", False),
    # A sibling whose name starts with a directory entry is not inside it.
    ("crates/nvim/src/luaref.rs", False),
    # ... and neither is a longer path built on an exact-path entry.
    ("crates/nvim/src/memfile.rs.orig", False),
    ("crates/nvim/src/memory.rs", False),
]
# (stats, expected (inside, outside)). The split is over `unsafe_lines`
# alone, and a file with none contributes to neither side.
SELF_TEST_PERIMETER_SPLIT = [
    (
        {
            "crates/nvim/src/lua/ffi.rs": {"unsafe_lines": 150},
            "crates/nvim/src/memfile.rs": {"unsafe_lines": 400},
            "crates/nvim/src/memline/mod.rs": {"unsafe_lines": 183},
            "crates/nvim/src/types/memline.rs": {"unsafe_lines": 0},
        },
        (550, 183),
    ),
]
# (what happens to an otherwise complete tree, whether check_perimeter
# rejects it). Every entry needs a file with unchecked lines behind it, so
# the cases are built by taking one entry's file away or making it safe.
SELF_TEST_PERIMETER_CHECK = [
    ("keep", False),
    ("drop", True),
    ("zero", True),
]


def self_test():
    for source, expected in SELF_TEST:
        got = unsafe_lines(mask(source), False)
        assert got == expected, f"unsafe_lines={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_DENY:
        got = unsafe_lines(mask(source), True)
        assert got == expected, f"unsafe_lines={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_SAFETY_DOC:
        got = missing_safety_doc(source, mask(source))
        assert got == expected, (
            f"missing_safety_doc={got}, want {expected}, for {source!r}"
        )
    for source, expected in SELF_TEST_DENY_CASTS:
        got = DENY_CASTS.search(mask(source)) is not None
        assert got == expected, f"cast deny={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_TEST_MODULE:
        got = len(test_module_lines(mask(source)))
        assert got == expected, (
            f"test_module_lines={got}, want {expected}, for {source!r}"
        )
    needle = COUNTED_RE["extern_abi"]
    for source, expected in SELF_TEST_EXTERN_ABI:
        got = len(needle.findall(mask(source)))
        assert got == expected, f"extern_abi={got}, want {expected}, for {source!r}"
    needle = COUNTED_RE["pub_items"]
    for source, expected in SELF_TEST_PUB_ITEMS:
        got = len(needle.findall(mask(source)))
        assert got == expected, f"pub_items={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_CELL_PTR_ALLOW:
        got = len(CELL_PTR_ALLOW_RE.findall(mask(source)))
        assert got == expected, (
            f"cell_ptr allowlist={got}, want {expected}, for {source!r}"
        )
    for source, expected in SELF_TEST_CELL_PTR_SITE:
        got = dict(cell_ptr_receivers({"t.rs": mask(source)}))
        assert got == expected, (
            f"cell_ptr receivers={got}, want {expected}, for {source!r}"
        )
    for sources, expected in SELF_TEST_CELL_PTR_PARTITION:
        tree = {f: mask(text) for f, text in sources.items()}
        stats = {
            f: {"cell_ptr": sum(m.count(n) for n in COUNTED["cell_ptr"])}
            for f, m in tree.items()
        }
        got = cell_ptr_partition(stats, tree)
        assert got == expected, (
            f"cell_ptr partition={got}, want {expected}, for {sources!r}"
        )
    for sources, expected in SELF_TEST_CELL_PTR_CHECK:
        tree = {f: mask(text) for f, text in sources.items()}
        # Every register name is present, so only the cap can fire.
        tree["registers.rs"] = mask(
            "fn keep() {\n"
            + "".join(f"    {name}.ptr();\n" for name in CELL_PTR_KEEPERS)
            + "}\n"
        )
        try:
            check_cell_ptr(tree)
            got = False
        except SystemExit:
            got = True
        assert got == expected, (
            f"check_cell_ptr rejected={got}, want {expected}, for {sources!r}"
        )
    for source, expected in SELF_TEST_CELL_COPY_OWNER:
        got = len(CELL_COPY_OWNER_RE.findall(mask(source)))
        assert got == expected, (
            f"cell_copy_owner={got}, want {expected}, for {source!r}"
        )
    for source, expected in SELF_TEST_PLACE_WRITE:
        got = len(place_writes({"t.rs": mask(source)}))
        assert got == expected, f"place_writes={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_DEREF_MUTATION:
        got = len(deref_temporary_mutations({"t.rs": mask(source)}))
        assert got == expected, (
            f"deref_temporary_mutations={got}, want {expected}, for {source!r}"
        )
    for source, expected in SELF_TEST_BORROWED_DEREF:
        got = len(borrowed_derefs({"t.rs": mask(source)}))
        assert got == expected, (
            f"borrowed_derefs={got}, want {expected}, for {source!r}"
        )
    for file, expected in SELF_TEST_PERIMETER:
        got = in_perimeter(file)
        assert got == expected, f"in_perimeter={got}, want {expected}, for {file!r}"
    for stats, expected in SELF_TEST_PERIMETER_SPLIT:
        got = perimeter_lines(stats)
        assert got == expected, f"perimeter_lines={got}, want {expected}"
    # A nested entry (`lua/treesitter/` under `lua/`) keeps its parent alive,
    # which is right -- the parent really does still have unchecked code under
    # it -- so the probe has to be an entry that stands alone.
    probe = next(
        entry
        for entry in PERIMETER
        if not any(
            other != entry and (other.startswith(entry) or entry.startswith(other))
            for other in PERIMETER
        )
    )
    for case, expected in SELF_TEST_PERIMETER_CHECK:
        stats = {
            (entry + "x.rs" if entry.endswith("/") else entry): {
                "unsafe_lines": 0 if case == "zero" and entry == probe else 1
            }
            for entry in PERIMETER
            if not (case == "drop" and entry == probe)
        }
        try:
            check_perimeter(stats)
            got = False
        except SystemExit:
            got = True
        assert got == expected, (
            f"check_perimeter rejected={got}, want {expected}, for {case}"
        )


def main():
    args = set(sys.argv[1:])
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

    self_test()
    stats, without_forbid, without_deny, without_casts, tree = measure()
    check_place_writes(tree)
    check_borrowed_derefs(tree)
    check_deref_temporary_mutations(tree)
    check_cell_ptr(tree)
    check_names(tree)
    check_perimeter(stats)
    counts = {**ledgers(), **whole_tree(stats, tree)}
    content = render(stats, counts, without_forbid, without_deny, without_casts)
    committed = BASELINE.read_text() if BASELINE.exists() else None

    if "--check" in args:
        if committed is None:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is missing; run `just refresh`"
            )
        if grew := violations(
            stats,
            counts,
            without_forbid,
            without_deny,
            without_casts,
            json.loads(committed),
        ):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: counts may only shrink. Reduce them, or if the "
                "growth is justified run `just refresh --allow-growth` and "
                "explain it in the commit message."
            )
        if committed != content:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is stale (progress "
                "to lock in); run `just refresh` and commit the result"
            )
        return

    if committed is not None and "--allow-growth" not in args:
        if grew := violations(
            stats,
            counts,
            without_forbid,
            without_deny,
            without_casts,
            json.loads(committed),
        ):
            print("\n".join(grew), file=sys.stderr)
            sys.exit(
                "ratchet: refusing to raise the baseline. If the growth is "
                "justified, rerun with --allow-growth."
            )
    BASELINE.write_text(content)
    print(
        f"wrote {BASELINE.relative_to(ROOT)}: "
        f"{summary(stats, counts, without_forbid, without_deny, without_casts)}"
    )


if __name__ == "__main__":
    main()
