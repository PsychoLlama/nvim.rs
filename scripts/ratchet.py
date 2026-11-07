#!/usr/bin/env python3
"""Ratchet the migration metrics: counts may hold or shrink, never grow.

The migration's promise is monotonic progress — every change leaves the tree
no less safe than it found it. This script is the mechanism. It measures, per
Rust source file (crates/*/src/**/*.rs plus the crate-root .rs files;
integration tests under crates/*/tests are not migration surface and stay
unmeasured, as they were when they lived at the repo root):

  unsafe_stmts  statements of *code* the compiler is not checking: the
              statements inside an `unsafe {}` block, inside an `unsafe
              extern` block, or in the body of an `unsafe fn` in a file that
              has not adopted #![deny(unsafe_op_in_unsafe_fn)] (there the body
              is implicitly unsafe throughout, which is exactly what the
              metric is about). A region is charged by what it *does*: every
              `;`-terminated statement, every block's tail expression and
              every `match` arm, at every depth, minimum one. A nested
              `unsafe {}` inside a charged region costs nothing; `unsafe
              impl`/`unsafe trait` and a bodyless declaration cost one — an
              unchecked promise with nowhere else to book it. An `unsafe fn`
              *type* (a function pointer) costs nothing: the obligation is
              paid where it is called. A comment is free, so a SAFETY note
              costs nothing, and so is layout: a call rustfmt wrapped over
              four lines is one statement, as it was on one.

              This metric counted *lines* until phase 31, and lines are
              gameable in the one direction that matters: `unsafe { a; b; c }`
              spread over five lines cost 5 and the same three calls in three
              one-line regions cost 3, so splitting a region scored as
              progress while the tree did exactly as much unchecked work as
              before. Statements price the work. Counting only a region's
              *top level* would be wrong the other way round — it prices
              `unsafe { loop { ..fifty lines.. } }` at 1, and the c2rust
              blanket a phase of work removed would come back free — so the
              charge recurses.

              Before lines it counted `unsafe {}` *blocks*, which diverged
              from the goal on exactly the change the migration wants most:
              splitting a 700-line transpiled body into fifteen functions
              with narrow blocks booked fifteen units where there was one
              (edit.rs went 89 -> 104 blocks across phase 15 while its clippy
              count went 33 -> 0). The statement charge keeps lines' answer to
              that — narrowing a region lowers it, deleting unchecked code
              lowers it, adopting the deny and wrapping a body is neutral —
              and adds the one lines got wrong.

  static_mut · variadic  retired: `static mut ` and `: ...` were counted per
              file until the last of each sat in `os/cshim.rs` (four
              `static mut`, one variadic definition: the `snprintf` shim) and
              were documented as the floor. Both need `unsafe` to touch, so
              the placement check keeps them on the perimeter and that file's
              own `unsafe_stmts` keeps them from growing.
  no_mangle   occurrences of "#[unsafe(no_mangle)]"
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
              obligation the signature announces but nobody wrote down.
              **At zero since phase 28, and retired**: it stood in for
              `clippy::missing_safety_doc` while that lint was *allowed* in
              Cargo.toml (2 380 findings when phase 19 switched the style tier
              on, which is a phase of work and not a slice), because clippy
              reports it at the crate lint level and leaving it warning would
              have buried `just lint`'s ~20 real findings. The allow is gone
              and the lint is a plain `warn` again, which `-D warnings` makes
              a deny, so a new one fails the build. The counter stays at its
              floor of zero as the second lock.

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

              Nothing else is exempt. `unsafe_stmts`, `missing_safety_doc`,
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
                    bound from an option slot or a grid). Without the list
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
                    transitively owns a raw pointer — a `ApiString`, a
                    `GArray`, a `regbehind_T`. `get` copies the struct out
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

  the perimeter  the files whose unchecked code is expected to stay
                    unchecked (PERIMETER, below). `unsafe_stmts_outside_perimeter`
                    -- `unsafe_stmts` restricted to the files *off* the list --
                    was the migration's debt number: the total said how much
                    unchecked code the binary contains, this said how much of
                    it the compiler should one day be checking. It reached 0
                    and retired when the placement check (below) became a hard
                    failure: a file off the list cannot allow unsafe code, so
                    it cannot hold an unchecked statement either.

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
                    file's `unsafe_stmts` is already ratcheted individually,
                    a new file included, so unchecked code appearing inside
                    the perimeter is a violation exactly as it is outside.

                    docs/perimeter.md's "Today" sentence is generated from
                    this same measurement rather than typed: this run
                    rewrites it, `--check` fails when it is stale. It was
                    hand-maintained for five phases and every one of its five
                    numbers drifted.

  the C vocabulary  what is left outside the perimeter is no longer blanket
                    `unsafe` — it is *C vocabulary*: integer status codes
                    where `Result` belongs, raw `c_char` strings, manual
                    allocation and pointer walks, integer constants and
                    unions where enums belong, and the transpiler's own type
                    names. Sixteen whole-tree counts measure the dialects, one
                    per idiom being retired. They are whole-tree and not
                    per-file on purpose: none of these is a per-file problem —
                    one signature changed in one file retires call sites in
                    forty, and a per-file row would book the helper that
                    absorbs them as a regression.

                    They are deliberately plain greps over the masked source.
                    Over-counting a little is fine (`.add(` on a `Vec` index
                    is not pointer arithmetic, and is counted anyway); what is
                    not fine is a needle a *rewrite* can make grow, so where
                    the idiom and its replacement look alike the needle is
                    narrowed, and the narrowing is written down here.

                      c_int_returns   `-> c_int` — the C status-code return,
                        in every spelling of the type (`c_int`,
                        `::core::ffi::c_int`, `core::ffi::c_int`,
                        `std::ffi::c_int`, `libc::c_int`), so that re-spelling
                        one is not progress and converting one is.
                        Function-pointer types count too: apigen's tables
                        declare them and they retire with their callees.
                      ok_fail         `return OK`/`return FAIL` and `== `/
                        `!= OK`/`FAIL` — the values those returns carry.
                      error_out_params  `*mut Error` *parameters*,
                        api/'s out-parameter. A field of that type is the
                        owning struct's storage rather than a lent channel,
                        so it is not counted; the needle was tree-wide and
                        that is what kept the number off zero.
                      semsg_c         `semsg_c!`, `semsg_multiline_c!`,
                        `tr_c!` and `tr_plural!` — message *templates* that
                        are data rather than literals, so `format_args!`
                        cannot check them. The `_c` macros are gone; the two
                        `tr_*` forms are what replaced them where the template
                        is genuinely chosen at runtime, and they are the
                        remaining debt. `emsg(gettext(` used to be counted
                        here and no longer is: with the message constants
                        `&CStr` and every conversion-carrying one inlined at
                        its call site, not one of the 344 remaining
                        `emsg(gettext(X))` carries a `printf` conversion —
                        the spelling is a translated *message*, not a format.
                      raw_cstr        `*mut c_char` + `*const c_char`, both,
                        because constness is not what is being retired.
                      libc_strings    the eleven `str*`/`mem*` calls. Word-
                        bounded, so the tree's own `xstrlcpy`/`vim_strchr`
                        wrappers are *not* counted: they are where the libc
                        call is meant to end up until the slice ops replace
                        them, and counting them would penalise the interim.
                      const_c_int     `pub const NAME: c_int`/`c_uint` — the
                        integer constant families that want to be enums.
                        The type may be path-qualified (`::core::ffi::`,
                        `std::ffi::`, `libc::`): c2rust wrote most of them
                        that way, and a bare-name needle missed 760.
                      const_int_alias the same debt worn under a different
                        name: `pub const NAME: Alias`, where `Alias` is one of
                        the tree's *own* integer type aliases. c2rust rendered
                        every C `enum` as a `typedef`ed integer plus a run of
                        `pub const`s, so `auto_event`, `CMD_index`, `StlFlag`
                        and their kin hide families `const_c_int` cannot see.
                        The alias set is derived from the tree rather than
                        listed here — every `type X = Y` chain that bottoms
                        out in a primitive integer — so a family leaves the
                        count only by becoming an enum, and a *new* alias is
                        counted the day it lands. The constant's name may be
                        any case, unlike `const_c_int`'s: c2rust kept the C
                        enumerator's spelling, and the largest families left
                        (`CMD_append`, `kOptIdx…`) are not SCREAMING_CASE.
                      unions          `union` declarations.
                      repr_c_ffi_types  `#[repr(C)]` in the FOREIGN_ABI_TYPES
                        files: the per-library type definitions c2rust
                        hoisted under `types/`, whose layout belongs to
                        libuv, libvterm, libtermkey, LuaJIT or libc. Not
                        debt -- but ratcheted anyway, the way a perimeter
                        module's `unsafe_stmts` are, so a new one is visible.
                      repr_c_editor_state  `#[repr(C)]` everywhere else
                        outside PERIMETER: this tree's own aggregates, which
                        is transpiler residue except where a codec, a
                        flexible array member or a state-machine base pins
                        the layout. This is the number the migration drives
                        down. The two together are every `#[repr(C)]` off
                        the perimeter -- they were one metric until phase
                        25's close, whose measurement was that 40% of it was
                        a foreign ABI wearing this tree's file name, so a
                        single total could fall while the residue grew. What
                        stays of both is written down in REPR_C_FLOOR (see
                        there); `--dimension repr_c_floor` lists the rest.
                      derive_copy     `#[derive(.. Copy ..)]` on a *braced*
                        `struct`/`union` — an aggregate with named fields.
                        Enums are excluded because an enum is Copy-worthy by
                        construction and phase 25 *creates* them by the
                        dozen; tuple and unit structs are excluded because
                        that is the shape of a handle (`WinId`, `BufId`, a
                        `flag_set!` newtype), whose `Copy` is the point. What
                        is left is exactly the class the migration is
                        retiring: a struct that owns something and is copied
                        anyway. The ones that stay are COPY_KEEPERS (see
                        there); `--dimension copy_keepers` lists the rest.
                      manual_alloc    `xmalloc`/`xmallocz`/`xcalloc`/
                        `xrealloc`/`xfree`.
                      garray_sites    `ga_grow`/`ga_init`/`ga_clear`/
                        `ga_concat`/`ga_append` — `GArray`'s five load-
                        bearing entry points, not its whole surface.
                      kvec_sites      `kv_*` calls plus `InitVec::new(`/
                        `Kvec::new(`. c2rust expanded klib's kvec macros, so
                        what is left is the `kv_puts`/`kv_do_printf`/`kv_size`
                        family of *calls* (a macro invocation writes `kv_x!`,
                        a function call `kv_x(` — the needle takes both) and
                        the port's own borrowed views over a `kvec_withinit_t`,
                        which exist only to bridge to that layout and vanish
                        the day the struct becomes a `Vec`. Field *names*
                        (`init_array`, `size`/`capacity`/`items`) are not
                        counted: they are the shape, not a call site, and
                        several belong to the api's RPC ABI, which stays.
                      khash_sites     `Map_*`/`Set_*` — the monomorph type
                        names c2rust produced for klib's khash. The needle is
                        over the *types*, not the API (`map_del(` also names a
                        `:map` function, `set_init_`/`set_has` are option and
                        visual-mode names), which makes it unambiguous, counts
                        the monomorphs nothing calls, and reaches zero exactly
                        when `map/` can be deleted. It is lumpy — most of it is
                        the two files that declare the monomorphs — and that is
                        the point: a table converts, its type goes with it.
                      ptr_arith       `.offset`/`.add`/`.sub`/`.offset_from`,
                        and `.wrapping_add`/`.wrapping_sub` on a receiver
                        `PointerArith` infers is a raw pointer -- most
                        `wrapping_*` calls are integer methods, which are not
                        this debt.
                      t_suffix_types  *distinct* names ending in `_T` that
                        some `struct`/`enum`/`union`/`type` item declares —
                        a set over the whole tree, so moving a type between
                        files is free and only deleting or renaming one
                        counts. `type` aliases are in deliberately: the
                        integer aliases are the bulk of phase 27's rename.
                      raw_win_buf_sigs  a raw `Window`/`Buffer`/`Tabpage`
                        pointer, `*mut` or `*const`, inside a `fn`
                        *signature* — the span from the `fn` keyword through
                        the return type, so a parameter rustfmt wrapped onto
                        its own line still counts and a local variable of that
                        type does not. Counted **outside `winlayer/`** only,
                        the way `curwin_raw` is: the handles' own
                        constructors and accessors are where an address
                        becomes an identity, and a number that counted them
                        could never reach zero.
                      raw_frame_sigs  a raw `Frame` pointer over the same
                        spans, with the same `winlayer/` carve-out. The layout
                        tree's nodes are freed under their holders --
                        `winframe_remove` frees one and `close_windows` frees
                        the window it named -- so an address in a signature is
                        a caller that cannot ask whether what it holds is
                        still there. `FrameId` is what can. A `*mut *mut
                        Frame` out-parameter counts once, as the one frame it
                        names.
                      mut_win_buf_refs  `&mut Window`/`&mut Buffer`/
                        `&mut Tabpage` over the same spans. The retype away
                        from the raw pointers above has exactly one wrong
                        landing place, and this is it: `&mut` is `noalias`,
                        while `curwin`/`curbuf` alias every window and buffer
                        the editor hands around, so such a parameter is
                        undefined behaviour the moment its argument is the
                        current one. The handle (`Win`/`Buf`/`TabPage`) is the
                        parameter type — `Copy`, and borrowing only for the
                        length of one field access. With the same `winlayer/`
                        carve-out: the handles' own `DerefMut` impls are
                        where that one-field borrow is made, so they are the
                        handle and not a parameter.
                      abbrev_params   the transpiler's parameter
                        abbreviations (`wp`, `eap`, `rettv`, `ptr` …)
                        bound in a `fn` signature, over the same span, with
                        `lua/` and `vterm/` carved out — those two are ports
                        whose parameter names are the upstream project's — and
                        with the API methods apigen dispatches
                        (tools/apigen/functions.txt) carved out under `api/`:
                        their parameter names are the msgpack-RPC surface,
                        published by `nvim_get_api_info()` and printed by
                        `Invalid '<name>'`, so upstream's spelling is frozen. A
                        leading `_` counts (an unused parameter is still that
                        parameter), a qualifier does not: `old_buf` already
                        says what `buf` does not. `buf`/`bp` count only when
                        the parameter's *type* names a buffer object: `buf`
                        for a byte buffer is idiomatic Rust and stays. `ptr`
                        splits the same way and counts only when the type
                        names something other than raw memory: `ptr` on a
                        `*mut c_void` or a `*mut T` is what Rust calls it too.
                      curwin_raw      `curwin`/`curbuf`/`curtab` `.get()`
                        reads outside `winlayer`, which is the module whose
                        job it is to turn those globals into handles. The
                        same outside-a-home shape as the perimeter's:
                        the reads inside
                        the home are the implementation, the ones outside are
                        the debt.

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

                    So the same inverted trick as `forbid(unsafe_code)`: a
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
                    family's marker. The files left without it are a written
                    floor, `CAST_DENY_FLOOR` (each with its reason: a
                    perimeter file's casts, a module root over one, and
                    `narrow.rs`, the cast home, for good), checked both ways
                    by every run. Naming the rest is the house convention,
                    not something the ratchet can check — as with
                    `forbid(unsafe_code)`, the attribute is a claim the
                    compiler then enforces, and the ratchet only counts who
                    has made it.

  files_without_deny_unsafe_op  (retired) the number of source files carrying neither
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
                    describes, and why lib.rs went last. Retired at 0 when the
                    placement check became a hard failure: its third rule asks
                    every file allowing unsafe code for the deny, a forbidding
                    file needs none, and a module file carrying neither holds
                    no unsafe code for the deny to govern.

  files_allowing_<lint>  one per entry in FILE_ALLOWS: the number of source
                    files carrying that lint's blanket `#![allow]`. The same
                    shape as the one above, one step further along. Each of
                    these lints was allowed by default -- three of them at the
                    crate root, because every name c2rust emitted was the C's,
                    and `unsafe_code` by rustc itself -- and each lift denies
                    it in Cargo.toml, so what is left carries a per-file allow
                    with a one-line reason.

                    files_allowing_non_camel_case_types  what still needs it
                    is a file whose *type* names are a foreign library's --
                    libuv's `uv_loop_t`, libc's `size_t`, LuaJIT's
                    `lua_CFunction` -- or one still holding c2rust's name for
                    an anonymous member.

                    files_allowing_non_snake_case  the same for value names:
                    LuaJIT's `L`, and the `KeyDict_*` keysets' field names,
                    which are the wire keys the API publishes.
                    The `nvim__*` API methods are *not* here -- a name the RPC
                    surface publishes gets an item-level allow, which this
                    does not count, so one frozen name never buys a file a
                    blanket one.

                    files_allowing_non_upper_case_globals  the widest of the
                    three, because it is every global the editor has: an
                    option's `p_<abbrev>` cell, a `kOpt<Name>` index, a
                    `k<Name>` tag, and the ~880 statics upstream
                    declares in `globals.h`. Nothing here is a rename this
                    phase could do -- the spelling is what the option
                    metadata, the FFI golden and the handler tables look up --
                    so the count falls a module at a time, as each is
                    rewritten to Rust's spelling for a global. Eight of the
                    files carrying it are generated and take it from apigen.

                    files_allowing_unsafe_code  (retired) the migration's own posture,
                    and the one entry here that is not about names. rustc
                    allows `unsafe_code` by default, so for the whole
                    transpile nothing said where unsafe was permitted to live:
                    the tree's answer was the *absence* of
                    #![forbid(unsafe_code)], which is not a claim a file makes
                    but one it fails to make. Phase 28 denied the lint in
                    crates/nvim/Cargo.toml, so a file that needs unsafe -- a
                    block, an `unsafe fn`, an `unsafe trait` or `impl`, an
                    `unsafe extern` block, an `#[unsafe(no_mangle)]` export --
                    now says so with an inner allow. A file inside the
                    perimeter (PERIMETER, docs/perimeter.md) names its row in
                    a comment above the attribute and keeps it for good; every
                    other one carries it bare, because there the count *is*
                    the reason and it is debt. `forbid` still overrides the
                    deny and cannot itself be lifted, so a module that
                    finishes drops the allow and takes the forbid instead --
                    which is what retired `files_without_forbid_unsafe`, this
                    metric's complement, when the default flipped. The
                    generated files take theirs from apigen, emitted only for
                    a chunk that holds unsafe. Retired when the placement
                    check became a hard failure: every file allowing unsafe
                    code is then on the perimeter, so the count is the
                    perimeter's file count (docs/unsafe-audit.md), and a new
                    one is a new PERIMETER row, which a reviewer reads.

                    The three left may only fall, and a new file has no
                    business taking any of them unless it is describing
                    someone else's names.

  the instruments  the C vocabulary above measures idioms; these measure the
                    C *calling convention* and the C *shape*, which is what is
                    left outside the perimeter once the transpiled control
                    flow is gone. Each is a whole-tree number for the same
                    reason the vocabulary's are — one signature retyped in one
                    file retires call sites in forty — and each decomposes per
                    file, so `--dimension NAME` lists where the debt sits
                    without a second script. Each says below what retires it.

                      unsafe_fns      `unsafe fn` *items* — a definition or a
                        bodyless declaration, never a function-pointer type
                        (`unsafe fn(..)`, whose obligation is paid where it is
                        called) and never a declaration inside an `unsafe
                        extern` block, whose obligation is the C library's.
                        The same walk `missing_safety_doc` uses. This is a
                        *consequence* number, not a target: a function is
                        `unsafe` because something it takes is a raw pointer,
                        so it falls when phases 29-31 retype the parameter,
                        and driving it down any other way means a wrapper.
                      unsafe_fns_without_raw_params  an `unsafe fn` whose
                        *signature* — parameters and return type both, joined
                        the way `raw_ptr_params` joins them, so a list rustfmt
                        wrapped still reads as one — carries no `*mut`/
                        `*const`, no `CPtr` bound and no `type X = *mut ...`
                        alias (collected tree-wide: a name is not a hiding
                        place), is not `extern "C"` (the
                        ABI is the contract), and whose name is not a row in
                        docs/unsafe-fn-allowlist.md. The keyword left behind
                        after a phase retyped the parameters: the body is
                        already checked under
                        deny(unsafe_op_in_unsafe_fn), so dropping it is a
                        semantic no-op and every caller's wrapper goes with
                        it. A function that hands raw memory *back* is not
                        counted — `-> *mut c_void` is a contract without
                        needing a row — and the four classes that are keep
                        one: see the allowlist. Phase 31.
                      safety_on_safe_fns  a `fn` *without* the `unsafe`
                        keyword whose doc comment still carries a `# Safety`
                        heading — the section outlived the keyword when a
                        retype made the function safe. A `# Safety` section is
                        a contract; on a safe function it promises nothing and
                        misleads the next reader. At zero, phase 34.
                      wrappers  single-call `unsafe {}` regions, tree-wide: a
                        region whose whole body, bar whitespace and one
                        trailing `;`, is one call. Such a region exists
                        because its callee is `unsafe fn` and for no other
                        reason, so it is the number that falls when the
                        callee's keyword does. Line-agnostic on purpose — a
                        call rustfmt wrapped over four lines is the same
                        wrapper — and blind to what surrounds the braces: a
                        `let`, a tail, a `return`, an argument and a method
                        chain are all wrappers. `unsafe { *p }` and
                        `unsafe { &mut *p }` are not; that obligation is the
                        caller's own. The per-callee breakdown is
                        metrics/wrapper-callees.tsv, written by this script
                        on every run that writes the baseline and compared
                        literally under `--check`: one JSON object per callee
                        with its `count`, how many `unsafe fn` definitions
                        answer to the name (`defs`, and `def` when exactly one
                        file holds them), and `no_raw` — whether every one of
                        them is an `unsafe fn` the needle above would count.
                        A `no_raw` row's `count` is the wrappers a slice
                        deletes when it drops the keyword. Phases 31-33. A
                        finish list (metrics/finish-list.tsv: per module, each
                        file off the perimeter still allowing unsafe code)
                        was the work order for phases 34-40; it was retired
                        empty once the placement check refused any such file.
                      stored_addr_handles  (retired) `Win::new`/`Buf::new`/
                        `TabPage::new`/`FrameRef::new`/`::at`/`::from_raw`
                        whose argument is not `<expr>.raw()`.
                        Building a handle *reads* the object's `handle` field,
                        so an address the C only ever compared becomes a
                        dereference — and phase 28 closed on two
                        heap-use-after-frees of exactly that shape, which
                        whole-suite ASan was the only gate to see. Counted
                        outside `winlayer/`, where the constructors live.
                        Retired at 0: every constructor that takes an
                        address is an `unsafe fn` or private to `winlayer/`,
                        so a call needs an `unsafe` region, which the
                        placement check keeps on the perimeter and the
                        file's `unsafe_stmts` keeps from growing. Phase 29.
                      raw_ptr_params  a parameter *directly* typed `*mut T`/
                        `*const T`, over the parameter list alone. The C
                        calling convention itself, and the thing every
                        `unsafe fn` above is downstream of. Phases 29-31.
                      raw_cstr_params · raw_cstr_returns  the same for
                        `*mut c_char`/`*const c_char`, split at the arrow
                        because a parameter becomes `&CStr`/`&[u8]` and a
                        return becomes `CString`/`Vec<u8>`/a borrow, which are
                        different rewrites with different exit clauses. These
                        two are nesting-blind — the type is the debt wherever
                        in the list it appears — so neither is a subset of
                        `raw_ptr_params`, and they match every spelling of the
                        type (`::core::ffi::c_char`, `libc::c_char`, …) so
                        that qualifying a forwarder cannot move the count.
                        Phase 29.
                      raw_cstr_fields  the same type again, counted in a
                        braced `struct`/`union`'s *fields* rather than in a
                        signature. A field is storage and not a channel, so
                        it is a different rewrite from a parameter -- the
                        struct grows an owner (`XString`, `Vec<u8>`) or a
                        borrow, and every caller's lifetime argument changes
                        with it -- and it was invisible to every other needle:
                        `raw_cstr` counts the *type* tree-wide and cannot say
                        whether a site is a field, a local or a cast. One per
                        field however many pointers the type holds, so an
                        array (`[*mut c_char; 10]`, `RegMatch.startp`) is one
                        field of debt and not ten. Enum payloads are out of
                        scope: `OptSlot::String(*mut *mut c_char)` is a
                        selector and not storage, and it retires with the
                        options table. Phase 32.
                      mbyte_raw · bytes_at  the families that hand a raw
                        pointer *back*: a multibyte cursor (`utf_ptr2*`,
                        pointer forms only — the `_len` variants are the safe
                        bodies and are deliberately not matched; the floor is
                        `harness.rs`'s C exports), and `bytes_at`. Phase 29.
                        Two more retired: `msg_raw` matched the message entry
                        points by name and reached 0 (a name list cannot see
                        a new pointer form), and `ml_get_raw` -- the forms that
                        handed back a buffer line's address -- reached 0 when
                        they were deleted, and a pointer one handed back could
                        only be read inside an `unsafe` region, which the
                        placement check keeps on the perimeter.
                      vval_raw · typval_raw_params  the C value model: reads
                        of the `TypVal` union's `vval` arm, and `*mut TypVal`
                        parameters. Both retire when the union becomes an
                        enum. Phase 30.
                      exarg_raw · cmdarg_raw  the two command-argument state
                        structs passed by address. Phase 31.
                      global_cells · cell_raw_ptr  C's global state: how many
                        cells there are, and how many of them hold a raw
                        pointer rather than owning what they name. The
                        declaration and not the reads, because narrowing forty
                        reads to one accessor is `cell_ptr`'s business and a
                        read-counting metric would book it as no progress.
                        Phase 31.
                      api_err_params  `&mut Error` parameters — the landing
                        place a sweep off `error_out_params` reaches for, and
                        itself debt: the answer is a `Result<T, Error>`
                        return. Counting both is what stops
                        `*mut Error` -> `&mut Error` reading as the end of the
                        job. Phase 32.
                      char_as_c_int  `'x' as c_int` — C's character
                        vocabulary. The one needle matched against the raw
                        source, because `mask()` blanks a char literal, quotes
                        and all; the `as` it captured is checked against the
                        masked copy, so a cast written inside a comment or a
                        string still costs nothing. Phase 32. It also counts
                        `c_int::from(b'x')`, the same byte widened into C's
                        `int` in the spelling the cast lints ask for (152 ->
                        584 when it learned it: an instrument change).
                      emit_sentinel  a message emitted, then a sentinel
                        returned: an EMIT_CALL (`emsg`, `semsg!`, `iemsg`,
                        `smsg!`, `give_warning`, `msg_puts*`, `api_error!`,
                        ...) whose statement is followed directly by a
                        `return`/tail of `false`, `None`, `-1`, `0` or
                        `FAIL`. The caller learns that something failed and
                        not what, and the integer or `bool` it reads is
                        indistinguishable from a real answer. `Err(Failed)`
                        and `Err(e)` after a message are not counted: `Failed`
                        is the crate's "a message is already out" error, and
                        Vim's `emsg` channel (`v:errmsg`, `:try`,
                        `emsg_silent`) is the boundary that reports it.
                        Replaced `failed_uses` (every `Failed`, which counted
                        the honest form of the same thing as debt). Phase 41.
                        A call to a local message wrapper (a free `fn err*` or
                        `emsg_*` that answers nothing and calls an EMIT_CALL,
                        found by `emit_wrappers`) is an emitter too: the
                        wrappers hid their callers' sentinels, 50 -> 74 when
                        the needle learned them (an instrument change). An
                        emitter nested in another's arguments (`semsg!(..,
                        msg_cstr(p))`) is the same message and counts once,
                        and one in a QUIET_MACRO's arguments (`logmsg!`,
                        `format!`, ...) is no message (30 -> 29, an
                        instrument change).
                      silent_failed  (listed, unratcheted) a `Failed`
                        made where nothing was said: `Err(Failed)` or
                        `.ok_or(Failed)` in a body with no EMIT_CALL and no
                        `?`/`map_err` from a callee. `Failed` means "a
                        message is already out"; a refusal that says nothing
                        answers `Option`, an enum or `IoFailed`. Phase 42,
                        179 at its landing, not a guard: an option checker
                        whose caller says E474 for it, a `let mut ret =
                        Err(Failed)` a callee's answer overwrites, and the
                        cursor's step functions are its false positives.
                      labeled_blocks  `'label: {` — the transpiler's
                        rendering of C's forward `goto`. A lifetime bound
                        (`'a: 'b`) cannot match: what follows the colon has to
                        open a block. A labelled `loop`/`while`/`for` is not a
                        goto but Rust's multi-level exit, so it left the count
                        (96 -> 71, an instrument change); `--dimension
                        labeled_loops` lists those, unratcheted. Phase 34.
                      lua_raw_stack · raw_variadic_calls  (retired) raw
                        `lua_State` stack calls outside `lua/stack/` and
                        `lua/ffi.rs`, and direct calls to the C printf-family
                        variadics outside the `CArg`-checked macros of
                        `variadic.rs`. Both reached 0. Every one of those
                        callees is a foreign `unsafe fn`, so a call needs an
                        `unsafe` region: off the perimeter the placement check
                        refuses the file, and on it the region is a new
                        statement in a file whose `unsafe_stmts` may only
                        shrink. Phase 33.
                      long_fns  functions whose item spans more than
                        LONG_FN_LINES lines, outside generated files — c2rust
                        translated each C function whole, so a 700-line body
                        is a family of operations sharing one stack frame. The
                        generated carve-out is by marker, not by path: see
                        GENERATED_MARKER. Phase 34, and a function split along
                        the seams its arguments draw happens when they are
                        retyped, not after.
                      dup_consts  redundant copies of a constant: for every
                        (name, value) pair declared in more than one file, the
                        number of files past the first. Counting *copies*
                        rather than *names* is what makes deleting one of five
                        `NULL`s move the number; a name-counting metric would
                        sit still until the last one went. The value is read
                        from the raw source, because masking blanks a string
                        literal and would make two different ones compare
                        equal, propping the count up with a duplicate nobody
                        could remove. Phase 34.
                      types_files  files under `types/` — c2rust's one
                        `types_defs.h`-sized namespace. The unit is the file
                        and not the type: what retires it is a type moving
                        next to the code that owns it, and the number reaches
                        zero when the directory does. Phase 34.
                      untested_dirs  top-level modules under crates/nvim/src
                        with no `#[test]` anywhere beneath them and no file or
                        directory of their name in crates/nvim/tests/unit --
                        `X.rs`, `X/`, or `X_<what>.rs` (`channel_reader.rs` is
                        `channel`'s; the longest module name wins, so
                        `ui_compositor.rs` would not be `ui`'s).
                        A module's `X.rs` beside its `X/` is beneath it, and a
                        unit file that drives two modules names the second in
                        UNIT_TEST_ALSO_COVERS. A directory that is not a
                        library module (`bin/`) is in UNTESTED_EXEMPT.
                        The ground rule is oracle-first, and this is the list
                        of families that have no oracle at all. It shrinks
                        every slice, by construction: a slice that touches a
                        family writes its test before the rewrite.
                      cchar_scratch  fixed-size C string buffers: a
                        `[c_char; N]` array type anywhere code can name one
                        (a local, a parameter, a return, a field, an alias)
                        or a `[0 as c_char; N]` initialiser, `vec![..]`
                        included. One array is one site -- a typed `let`
                        and its initialiser count once -- and comments do
                        not count. c2rust's rendering of C's `char buf[N]`:
                        the buffer is filled through a pointer, truncated at
                        `N` and read back up to its NUL, which is exactly
                        the text an owned `Vec<u8>`/`String` replaces. A
                        field an FFI struct needs for its layout stays; the
                        rest retire as their functions stop handing text
                        around by pointer.

A `warnings` metric used to sit alongside it; phase 5 drove the count to
zero and the dev shell (flake.nix) now sets `RUSTFLAGS="-D warnings"` for
every local and CI build instead, so the counter is retired.

Two things here are not counts but hard checks, and each fails the run
outright.

**Where the unsafe-code attributes sit** (`check_unsafe_placement`, over
crates/nvim/src): a file allowing unsafe code off PERIMETER; a file carrying
neither the allow nor `forbid(unsafe_code)` that is not a module file (`mod.rs`,
`lib.rs`, `x.rs` beside `x/`) over a descendant that allows -- the "neither"
list, computed, because `allow` under a `forbid` is E0453; and an allow file
without `deny(unsafe_op_in_unsafe_fn)` or -- on the perimeter -- without its
row named in the `//` comment directly above the attribute. The message names
each file, the rule it breaks and the two ways out (finish it, or a PERIMETER
row with a reason); `--dimension unsafe_placement` prints the same list. It
was a ratcheted count while the files outside finished, and became the check
when it read 0. A generated file has no exemption: apigen emits the allow only
for a chunk that holds unsafe, and such a chunk off the perimeter fails here
like any other file.

**A write through an accessor that answers by value**. `f().field = x`
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

Usage: ratchet.py [--check] [--allow-growth] [--dimension NAME]
  --check         compare the tree against the committed baseline instead of
                  writing: exit 1 if any metric grew, or if the baseline is
                  stale (a metric shrank but metrics/ratchet.json wasn't
                  regenerated).
  --allow-growth  write a baseline even though a metric grew. The override
                  for justified cases — the growth shows up in the
                  metrics/ratchet.json diff; explain it in the commit message.
  --dimension NAME  print where one instrument's sites are — per file, then
                  rolled up per directory — and write nothing. The triage
                  mode: the baseline says how much is left, this says where.
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
# The perimeter's prose. Its "Today" sentence is measured, not written:
# `sync_perimeter_doc` rewrites it and `--check` fails when it is stale.
PERIMETER_DOC = ROOT / "docs" / "perimeter.md"
# The `unsafe fn` allowlist: one row per function whose signature shows no
# raw pointer and whose `unsafe` is earned anyway, each with the class of
# obligation that earns it. It is the floor `unsafe_fns_without_raw_params`
# is measured against, and it prunes itself -- see `check_unsafe_fn_allowlist`.
UNSAFE_FN_ALLOW_DOC = ROOT / "docs" / "unsafe-fn-allowlist.md"
# The wrapper-callee table: the work order for the callee phases, written on
# every run that writes the baseline and compared literally under `--check`.
# A TSV rather than the ledgers' JSON lines because it has 5k rows and the
# repeated keys cost a quarter of a megabyte nobody reads; one row per line
# still diffs a moved count cleanly, which is the property that mattered.
WRAPPER_TABLE = ROOT / "metrics" / "wrapper-callees.tsv"
WRAPPER_TABLE_HEADER = "count\tcallee\tno_raw\tdefs\tdef\n"
FILE_ALLOWS_UNSAFE = "#![allow(unsafe_code)]"
# apigen's attribute spec: one line per method the API exposes. It is the
# list the generator dispatches, so it is also the list of signatures whose
# parameter names are the RPC surface's -- see API_EXPORTED below.
API_SPEC = ROOT / "tools" / "apigen" / "functions.txt"
# The crate's source root, and the out-of-crate unit suite. `untested_dirs`
# asks both: a top-level module is covered when something under it writes a
# `#[test]`, or when the unit suite drives it from `crates/nvim/tests/unit`.
CRATE_SRC = "crates/nvim/src/"
UNIT_TESTS = ROOT / "crates" / "nvim" / "tests" / "unit"
# A unit file named for one top-level module that also drives another, which
# its name cannot say: `options.rs` round-trips values through `options/`'s
# table *and* `option/`'s scopes and setters (both `pub(crate)`, so it reaches
# them through the API and `:set`). unit file stem -> the other modules.
UNIT_TEST_ALSO_COVERS = {"options": ("option",)}
# Directories under the crate's `src/` that are not modules of the library and
# so have no family to test: `bin/` is the binary's `main`, which hands its
# arguments to the library's entry point and nothing else.
UNTESTED_EXEMPT = {"bin": "the binary's entry point, not a module"}

LINE_CAP = 1000
# name -> needles counted in the masked source, summed.
COUNTED = {
    "no_mangle": ("#[unsafe(no_mangle)]",),
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
    # NB. `msgpack_rpc/server.rs` declares a second, unrelated `WATCHERS`;
    # the list is name-keyed, so a `.ptr()` there would be exempted too. It
    # has none today, and `cell_ptr` still counts it.
    "WATCHERS": "uv_signal_t array whose addresses are registered with uv",
    "REFRESH_TIMER": "TimeWatcher — a uv timer handle",
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
    "cell": "not one cell — a local bound from an option slot (S3) or a grid (S13)",
    "CELL": "`GlobalCell`'s own tests of `ptr`/`as_raw`; they must call them",
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
    # runtime is done: `script_items` and `ga_loaded` left the list in
    # phase 22's S16, and the two search-path buffers when the cache became
    # an `Rc<Vec<_>>` and the thread copy a `Mutex<Vec<_>>`: none is `Copy`.
    # getchar is done: the five `buffheader_T` cells and `typebuf` left the
    # list in phase 22's S7. `KeyBuffer` and `TypeAhead` are not `Copy` at
    # all now, so there is no `get` on either to count.
    # regexp is done: `rex` became a per-match `RegExec` on the API layer's
    # stack and `rsm` an owned snapshot (`RegSubMatch`), neither `Copy`.
    # the rest, one or a few sites each
    "curgrid",
    # `au_new_curbuf` left the list in phase 23's S11. It is a `BufferRef`,
    # which is upstream's own *weak* reference -- the `br_buf_free_count`
    # generation check exists precisely because the pointer is borrowed --
    # and since S11 the buffer's owner is a named one, the buffer registry's
    # `Owned<Buffer>`. A `get` on this cell copies a reference, not an owner.
    # `old_sub` left once the previous `:s` replacement came to own its text
    # and its ShaDa extra data, so the cell is not `Copy` and has no `get`.
    # ccline is done: `cmdline_block` and `restart_args` retired in phase 22's
    # S14 behind `CmdlineBlock`/`RestartArgs`, two owned newtypes over the
    # `Array` with a `Drop`. Neither is `Copy`, so there is no `get` on either
    # to count, and the one genuine move at each is `GlobalCell::take`.
    "CLIPBOARD",
    "EXPAND_WHAT",
    "provider_caller_scope",
    "counted",
    # The `Object` cells, from phase 22's F-P22-53 audit at the close: every
    # `GlobalCell<ApiString>` and `GlobalCell<Object>` in the tree, listed
    # whether or not the value it happens to hold owns anything today, so the
    # register can be re-derived by grepping for those two types. `names`
    # carries no site now -- `nth_lua_string` reads it through `with` and
    # `cache_lua_answer` frees the old answer with `replace` -- and is listed
    # so a `get` on it would be counted rather than discovered later.
    "METADATA",  # the API description; its arena is leaked, the tree shared
    # `msg_ext_id` left in phase 33: it is a field of the message state's
    # record now, whose `get` needs `Copy`, which `Object` is not.
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
# this tree does not own; `check_unsafe_placement` refuses an allow anywhere
# else, and docs/perimeter.md carries the prose. An entry
# is a directory prefix (trailing `/`) or one exact path, and every entry must
# have a file with unchecked lines behind it or `check_perimeter` fails --
# the list prunes itself as modules finish.
PERIMETER = {
    # -- Foreign ABIs. The unsafe is the call, and it retires when the
    # library does.
    "crates/nvim/src/lua/ffi.rs": "LuaJIT's C API, declared: the functions, "
    "`lua_Debug` and the `luaL_Reg` tables C reads",
    "crates/nvim/src/lua/stack/": "the typed `Stack`: the one module that "
    "pushes, pops and reads the `lua_State` stack, and reads LuaJIT's "
    "`lua_State` head directly (LuaJIT pinned by nix/deps.json; "
    "`check_state_layout` aborts on drift)",
    "crates/nvim/src/lua/executor/luv.rs": "luv: the C hooks that run a libuv "
    "callback against the right `lua_State`, and opening the library",
    "crates/nvim/src/lua/executor/state.rs": "making and closing `lua_State`s "
    "and opening the C libraries (LuaJIT's own, lpeg, luv) onto them",
    "crates/nvim/src/lua/treesitter/": "the tree-sitter C library: parsers, "
    "trees, queries and cursors are opaque C objects with C lifetimes",
    "crates/nvim/src/event/": "libuv: the loop, streams, timers, signals and "
    "processes are C objects registered by address",
    "crates/nvim/src/os/": "the operating system: syscalls through libc and "
    "libuv, the PTY, spawning a child, the environment",
    "crates/nvim/src/vterm/abi/": "libvterm's C interface: the 30 exports "
    "and the callback tables a C consumer installs (the emulator itself is "
    "Rust)",
    "crates/nvim/src/tui/tui.rs": "the terminal's lifecycle: the output "
    "tty/pipe handle on a private write loop, the startup timer, the SIGWINCH "
    "watcher, and stopping the process group on suspend",
    "crates/nvim/src/tui/input.rs": "reading the terminal: stdin's read "
    "stream, the escape and background-query timers, and the line "
    "discipline's erase character (`tcgetattr`)",
    "crates/nvim/src/tui/output.rs": "the flush: one `uv_write` to the "
    "output handle, run to completion on the write loop",
    "crates/nvim/src/tui/events.rs": "the UI events that reach libuv: tty "
    "mode, the visual bell's sleep, `ui_send`'s direct write, `uv_chdir`",
    "crates/nvim/src/xdiff/": "libxdiff, vendored: which bytes are whitespace "
    "is `isspace(3)`'s answer in the user's locale",
    "crates/nvim/src/harness.rs": "the C symbols the in-repo test harnesses "
    "resolve by name (docs/abi-ledger.md's editor rows): fronts over the "
    "modules' slice forms that read the C side's pointers, and the data "
    "symbols the modules re-export",
    # -- The primitives everything else stands on. Their whole job is to be
    # the one place a raw operation happens, so that callers need not.
    "crates/nvim/src/allocator.rs": "the global allocator: malloc/realloc/free",
    "crates/nvim/src/memory/": "xmalloc, the block pool and the owned C "
    "strings that adopt and hand back malloc blocks — the floor under every "
    "owned type in the tree",
    "crates/nvim/src/global_cell.rs": "the checked wrapper over c2rust's "
    "mutable statics; the raw static is touched here so it is nowhere else",
    "crates/nvim/src/winlayer.rs": "the window/buffer/position handles, the "
    "screen grid handle and the terminal handle: constructing one is the "
    "unsafe step, dereferencing it is not",
    "crates/nvim/src/winlayer/": "the same, split out by family",
    "crates/nvim/src/eval/typval/handle.rs": "the value handles (`ListRef`, "
    "`DictRef`, `BlobRef`, `PartialRef`) and a dictionary's item store: "
    "constructing a handle is the unsafe step, dereferencing it is not",
}

# The foreign ABIs whose *type definitions* live under `types/` instead of in
# the module that calls the library. c2rust hoisted every struct into a
# per-library file, so the layout of a `uv_loop_t` is described here while the
# `uv_run` that needs it is in `event/`, and the layout of a `TermKey` is
# described here while termkey's parser runs in `tui/`. Those callers are on
# PERIMETER; these files cannot join it, because they hold no unchecked code
# at all and `check_perimeter` rejects an entry with nothing behind it.
#
# The bar is PERIMETER's first bullet and nothing softer: the layout is
# defined by a library this tree links against or ports with its C ABI intact
# (libuv, libvterm, libtermkey, LuaJIT, libc), so `#[repr(C)]` here is that
# library's interface and not transpiler residue. A type this tree defines
# and only this tree reads does *not* qualify however C-shaped it looks --
# `types/keysets.rs` is written through byte offsets by our own generated
# keydict codec, `types/mpack*.rs` and `types/rpc.rs` describe a codec that
# was vendored and ported rather than linked (docs/perimeter.md keeps
# `mpack/`'s codec outside for the same reason), and `types/terminal_defs.rs`
# is `repr(C)` only because the FFI-safety lint follows a pointer out of
# `Buffer`. All of those stay in `repr_c_editor_state`, where a rewrite is
# still expected to retire them.
FOREIGN_ABI_TYPES = {
    "crates/nvim/src/types/uv.rs": "libuv's own structs: the loop, handles, "
    "requests and process options, all registered with the library by address",
    "crates/nvim/src/types/vterm.rs": "libvterm's public types -- the ported "
    "library keeps its C ABI, so these are its interface",
    "crates/nvim/src/types/vterm_internal.rs": "the same library's internal "
    "state, reached through the same ABI",
    "crates/nvim/src/types/termkey.rs": "libtermkey's instance and driver, "
    "ported with its ABI intact and driven from `tui/`",
    "crates/nvim/src/types/lua.rs": "LuaJIT's `lua_State`, `luaL_Buffer` and "
    "`luaL_Reg` -- the interpreter defines every one of them",
    "crates/nvim/src/types/libc.rs": "the C library's nominal aggregates, "
    "plus the pthread types libuv embeds by value",
    "crates/nvim/src/types/libuv_proc.rs": "a `uv_process_t` and its options "
    "embedded by value and handed to `uv_spawn` by address",
    "crates/nvim/src/types/pty_proc_unix.rs": "a libc `winsize` embedded by "
    "value and handed to the tty ioctls by address",
}

# `curwin`/`curbuf`/`curtab`'s home: the module whose job is to turn the raw
# current-object globals into handles. Same entry form as PERIMETER's.
WINLAYER = {
    "crates/nvim/src/winlayer.rs": "the handles themselves",
    "crates/nvim/src/winlayer/": "the same, split out by family",
}


class PointerArith:
    """The `ptr_arith` needle: pointer-arithmetic method calls.

    `.offset(`/`.add(`/`.sub(`/`.offset_from(` are counted wherever they are
    written, as they always were: an integer `.add(` is `ops::Add` spelled out,
    which nothing here does. `.wrapping_add(`/`.wrapping_sub(` are different --
    nine in ten of them are *integer* methods, `len.wrapping_sub(1)` -- so
    those two count only when the receiver is a pointer, which a text needle
    has to infer. The receiver is a pointer when it is

    - a cast to `*mut`/`*const`, or a call of a method that only answers a
      pointer (`as_ptr`, `cast`, `wrapping_offset`, ...), or pointer
      arithmetic whose own receiver is one;
    - a call of a function whose every definition with that many arguments
      returns a pointer (the file's own definition first, then the tree's);
    - a field whose declarations (the file's own structs first) include a
      pointer and no bare path -- an integer, or any named type, which has
      no `wrapping_add` either; `.0` reads the file's tuple structs;
    - a local whose nearest declaration above the site -- a parameter, a
      closure parameter, a typed `let` -- has a pointer type, or whose `let`
      initialiser is itself a pointer by these rules.

    It is an inference, not a type check: a shadowed name, a field name two
    structs type differently, or a pointer that only arrives through a
    generic is missed, and is booked as integer. The probe that introduced it
    found 61 of the tree's 471 `wrapping_*` calls on pointers.

    An object rather than a regex because the fields and return types are a
    property of the whole tree: `vocabulary` primes it before counting, and
    `findall` keeps the shape every other needle has.
    """

    ALWAYS = re.compile(r"\.(?:offset_from|offset|add|sub)\(")
    WRAPPING = re.compile(r"\.(?:wrapping_add|wrapping_sub)\(")
    PTR_TYPE = re.compile(r"(?:&\s*(?:mut\s+)?)?\*\s*(?:mut|const)\b")
    BARE_TYPE = re.compile(
        r"(?:::)?(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Za-z_][A-Za-z0-9_]*\s*$"
    )
    CAST = re.compile(r"\bas\s+\*\s*(?:mut|const)\s+[A-Za-z0-9_:<>\s]*$")
    NULL = re.compile(r"\bnull(?:_mut)?\s*(?:::\s*<.*>)?\s*\(\s*\)$", re.S)
    TAIL = re.compile(r"([A-Za-z0-9_]+)\s*$")
    POINTER_METHODS = frozenset(
        (
            "as_ptr",
            "as_mut_ptr",
            "cast",
            "cast_mut",
            "cast_const",
            "ptr",
            "wrapping_offset",
            "with_addr",
            "map_addr",
            "byte_offset",
            "byte_add",
            "byte_sub",
        )
    )
    ARITH = frozenset(("add", "sub", "offset", "wrapping_add", "wrapping_sub"))
    FIELD_DECL = re.compile(
        r"(?:^|[,{(])\s*(?:pub(?:\s*\([^)]*\))?\s+)?([a-z_][A-Za-z0-9_]*)\s*:(?!:)"
        r"\s*([^,\n]*)"
    )
    TUPLE_STRUCT = re.compile(
        r"\bstruct\s+[A-Za-z_][A-Za-z0-9_]*\s*(?:<[^>]*>)?\s*\(\s*"
        r"(?:pub(?:\s*\([^)]*\))?\s+)?([^,)]*)"
    )

    def __init__(self):
        self.returns = {}
        self.fields = {}
        self.primed = None

    def prime(self, tree):
        """Learn the tree's return and field types; once per tree."""
        if self.primed is tree:
            return
        returns, fields = {}, {}
        for masked in tree.values():
            for sig in fn_signatures(masked):
                key = (sig.name, call_arity(sig.params, True))
                returns.setdefault(key, []).append(sig.returns.strip())
            for body in struct_bodies(masked):
                for m in self.FIELD_DECL.finditer(body):
                    fields.setdefault(m.group(1), []).append(m.group(2).strip())
        self.returns = {k: self.verdict(v) for k, v in returns.items()}
        self.fields = {k: self.verdict(v) for k, v in fields.items()}
        self.primed = tree

    def verdict(self, types):
        """Some declaration is a pointer and none is a bare path."""
        return any(self.PTR_TYPE.match(t) for t in types) and not any(
            self.BARE_TYPE.match(t) for t in types
        )

    def findall(self, masked):
        found = [m.start() for m in self.ALWAYS.finditer(masked)]
        found += [
            m.start()
            for m in self.WRAPPING.finditer(masked)
            if self.pointer(masked, receiver_start(masked, m.start()), m.start(), 0)
        ]
        return found

    def pointer(self, text, start, end, depth):
        """Is `text[start:end]`, an expression, a raw pointer?"""
        span = text[start:end]
        expr = span.strip()
        if depth > 4 or not expr:
            return False
        lead = start + len(span) - len(span.lstrip())
        if expr.startswith("unsafe") and expr.endswith("}"):
            inner = lead + expr.index("{") + 1
            return self.pointer(text, inner, lead + len(expr) - 1, depth + 1)
        if self.CAST.search(expr):
            return True
        if expr.startswith("&"):
            return False
        if expr.startswith("*"):
            name = re.fullmatch(r"\*\s*([A-Za-z_][A-Za-z0-9_]*)", expr)
            return bool(name) and self.pointer_to_pointer(text, name.group(1))
        if expr.endswith(")"):
            open_at = backward_opener(expr, len(expr) - 1)
            if open_at == 0:
                return self.pointer(text, lead + 1, lead + len(expr) - 1, depth + 1)
            head = re.sub(r"::\s*<.*>$", "", expr[:open_at].rstrip(), flags=re.S)
            name = self.TAIL.search(head.rstrip())
            if not name:
                return False
            method = name.group(1)
            before = head[: name.start()].rstrip()
            if before.endswith(".") and not before.endswith(".."):
                if method in self.POINTER_METHODS:
                    return True
                if method in self.ARITH:
                    return self.pointer(text, lead, lead + len(before) - 1, depth + 1)
            if self.NULL.search(expr):
                return True
            key = (method, call_arity(expr[open_at:], False))
            here = [
                sig.returns.strip()
                for sig in fn_signatures(text)
                if (sig.name, call_arity(sig.params, True)) == key
            ]
            return self.verdict(here) if here else self.returns.get(key, False)
        name = self.TAIL.search(expr)
        if not name:
            return False
        ident = name.group(1)
        if re.search(r"\.\s*" + re.escape(ident) + r"$", expr):
            if ident.isdigit():
                firsts = [m.group(1).strip() for m in self.TUPLE_STRUCT.finditer(text)]
                return bool(firsts) and self.verdict(firsts)
            here = [
                m.group(2).strip()
                for body in struct_bodies(text)
                for m in self.FIELD_DECL.finditer(body)
                if m.group(1) == ident
            ]
            return self.verdict(here) if here else self.fields.get(ident, False)
        if ident[0].isdigit():
            return False
        return self.local(text, ident, start, depth)

    def pointer_to_pointer(self, text, name):
        return (
            re.search(
                r"\b" + re.escape(name) + r"\s*:\s*(?:&\s*(?:mut\s+)?)?"
                r"\*\s*(?:mut|const)\s+\*\s*(?:mut|const)\b",
                text,
            )
            is not None
        )

    def local(self, text, ident, at, depth):
        """The nearest declaration of `ident` above `at` decides."""
        name = re.escape(ident)
        decl = re.compile(
            r"\b"
            + name
            + r"\s*:(?!:)\s*|\blet\s+(?:mut\s+)?"
            + name
            + r"\b\s*(?::[^=;]*)?=\s*"
        )
        nearest = None
        for m in decl.finditer(text, 0, at):
            nearest = m
        if nearest is None:
            return False
        if nearest.group(0).startswith("let"):
            typed = re.search(r":\s*(.*?)\s*=\s*$", nearest.group(0), re.S)
            if typed:
                return bool(self.PTR_TYPE.match(typed.group(1)))
            stop = statement_end(text, nearest.end())
            return self.pointer(text, nearest.end(), stop, depth + 1)
        return bool(self.PTR_TYPE.match(text, nearest.end()))


def call_arity(params, declaration):
    """Top-level arguments in a parenthesised list; a declaration's `self`
    does not count, so a method and its call agree."""
    parts, depth, current = [], 0, []
    for c in params.strip()[1:-1]:
        if c in "([{<":
            depth += 1
        elif c in ")]}>":
            depth -= 1
        if c == "," and depth == 0:
            parts.append("".join(current))
            current = []
        else:
            current.append(c)
    parts = [p for p in [*parts, "".join(current)] if p.strip()]
    if (
        declaration
        and parts
        and re.match(r"\s*(?:&\s*(?:'\w+\s+)?(?:mut\s+)?)?(?:mut\s+)?self\b", parts[0])
    ):
        parts = parts[1:]
    return len(parts)


def backward_opener(text, close_at):
    """The index of the bracket that `text[close_at]` closes."""
    close = text[close_at]
    open_ = {")": "(", "]": "["}[close]
    depth = 0
    for i in range(close_at, -1, -1):
        if text[i] == close:
            depth += 1
        elif text[i] == open_:
            depth -= 1
            if depth == 0:
                return i
    return 0


def statement_end(text, at):
    """The `;` (or unmatched closer) ending the expression at `at`."""
    depth = 0
    for i in range(at, len(text)):
        c = text[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                return i
            depth -= 1
        elif c == ";" and depth == 0:
            return i
    return len(text)


def receiver_start(text, dot):
    """Where the postfix expression ending just before `text[dot]` begins:
    identifiers, `.`/`::` paths, calls, indexing and turbofish."""
    word = re.compile(r"[A-Za-z0-9_]")
    i = dot - 1
    while True:
        while i >= 0 and text[i].isspace():
            i -= 1
        if i < 0:
            return 0
        if text[i] in ")]":
            i = backward_opener(text, i) - 1
            j = i
            while j >= 0 and text[j].isspace():
                j -= 1
            if j >= 0 and text[j] == ">" and text[j - 1 : j + 1] != "->":
                depth = 0
                for k in range(j, -1, -1):
                    depth += {">": 1, "<": -1}.get(text[k], 0)
                    if depth == 0:
                        break
                j = k - 1
                while j >= 0 and text[j] in ": ":
                    j -= 1
            if j >= 0 and word.match(text[j]):
                i = j
            else:
                return i + 1
        while i >= 0 and word.match(text[i]):
            i -= 1
        j = i
        while j >= 0 and text[j].isspace():
            j -= 1
        if j >= 0 and text[j] == "." and text[j - 1 : j + 1] != "..":
            i = j - 1
            continue
        if j >= 1 and text[j - 1 : j + 1] == "::":
            i = j - 2
            continue
        return i + 1


PTR_ARITH = PointerArith()


# The C-vocabulary dimensions: whole-tree needle counts, one per dialect the
# migration is retiring. See "the C vocabulary" in the doc block for what each
# measures and, where the idiom and its replacement look alike, why the needle
# is drawn where it is. Everything here is matched against the *masked*
# source, so prose and string literals naming an idiom cost nothing.
# Every spelling of `c_char` c2rust and the tree between them produce. The
# needles below used the bare name alone, which made a *qualified* forwarder
# invisible: rewriting `*mut c_char` as `*mut ::core::ffi::c_char` -- the
# spelling every generated and every `unsafe extern` block uses -- lowered
# `raw_cstr_params` without retiring anything, and the reverse rewrite raised
# it. `c_int_returns` already spells the prefix this way; these now match it.
C_CHAR = r"(?:(?:::)?(?:core|std)::ffi::|(?:::)?libc::)?c_char"
VOCABULARY = {
    # `\s*` rather than a literal space: rustfmt wraps a long signature's
    # `-> c_int` onto its own line. `\b` so `c_int_ish` is not a match, and no
    # trailing anchor so `{`, `;`, `,` and end-of-line all count. The path
    # prefix is optional and covers every spelling the tree uses for the same
    # type — `c2rust` emitted `::core::ffi::c_int` in the files it left fully
    # qualified, and `libc::c_int` is what an FFI declaration reaches for — so
    # that converting one is progress and re-spelling one is not.
    "c_int_returns": re.compile(
        r"->\s*(?:(?:::)?(?:core|std)::ffi::|(?:::)?libc::)?c_int\b"
    ),
    # The comparisons are written `[=!]=` rather than `==|!=` so that `>=`/`<=`
    # do not match; `>= OK` is not a status-code test.
    "ok_fail": re.compile(r"\breturn\s+(?:OK|FAIL)\b|[=!]=\s*(?:OK|FAIL)\b"),
    # The `_c` macros no longer exist; keeping them in the needle is what
    # makes bringing one back visible. `tr_c!`/`tr_plural!` are the escape
    # hatch that replaced them: a message whose template arrives at runtime.
    "semsg_c": re.compile(r"\bsemsg_(?:multiline_)?c!|\btr_(?:c|plural)!"),
    # Every spelling, as the parameter and return needles below: the bare
    # name alone left 234 qualified ones (`*const ::core::ffi::c_char`,
    # every `unsafe extern` block's) uncounted, so respelling one moved it.
    "raw_cstr": re.compile(rf"\*(?:mut|const)\s+{C_CHAR}\b"),
    "libc_strings": re.compile(
        r"\b(?:str(?:len|cmp|ncmp|cpy|cat|chr|str)"
        r"|mem(?:cpy|move|set|cmp))\("
    ),
    "const_c_int": re.compile(
        r"\bpub const [A-Z_][A-Z0-9_]*: (?:(?:::)?(?:core|std)::ffi::|libc::)?c_u?int\b"
    ),
    "unions": re.compile(r"\bunion\s+[A-Za-z_]"),
    # A derive list holds no `)`, so `[^)]*` cannot run past the attribute.
    # The item must be a *braced* struct/union: see the doc block for why an
    # enum, a tuple struct and a unit struct are all excluded.
    "derive_copy": re.compile(
        r"#\[derive\([^)]*\bCopy\b[^)]*\)\]\s*(?:#\[[^\]]*\]\s*)*"
        r"(?:pub(?:\s*\([^)]*\))?\s+)?(?:struct|union)\s+[A-Za-z_][A-Za-z0-9_]*"
        r"\s*(?:<[^{;]*>)?\s*(?:where[^{;]*)?\{"
    ),
    "manual_alloc": re.compile(r"\bx(?:mallocz|malloc|calloc|realloc|free)\("),
    "garray_sites": re.compile(r"\bga_(?:grow|init|clear|concat|append)\("),
    # `[!(]` so that both spellings of a klib entry point count: the macros
    # c2rust could not expand are still written `kv_size!(x)`, the ones it
    # turned into functions `kv_size(x)`. Lower case only, so a *type* named
    # `kv_...` (there are none today) would not be mistaken for a call.
    "kvec_sites": re.compile(r"\bkv_[a-z_0-9]+[!(]|\b(?:InitVec|Kvec)::new\("),
    "khash_sites": re.compile(r"\b(?:Map|Set)_[A-Za-z0-9_]+\b"),
    # Not a regex: see `PointerArith` for how `wrapping_*` is told apart.
    "ptr_arith": PTR_ARITH,
}
# `#[repr(C)]`, shared by the two halves below so that they cannot drift
# apart: every match lands in exactly one of them.
REPR_C = re.compile(r"#\[repr\(\s*C\s*[,)]")

# The same, but counted only in files *outside* a home — the shape the
# perimeter established. name -> (needle, home).
VOCABULARY_OUTSIDE = {
    "repr_c_editor_state": (REPR_C, {**PERIMETER, **FOREIGN_ABI_TYPES}),
    "curwin_raw": (re.compile(r"\bcur(?:win|buf|tab)\s*\.\s*get\(\)"), WINLAYER),
}
# ... and the mirror image: counted only in files *inside* a home. `repr(C)`
# in a foreign ABI's type file is that library's layout, not this tree's, so
# it is not the same number as the residue and must not share a total with
# it — but it is still ratcheted, exactly as a perimeter module's
# `unsafe_stmts` still are, so a new one has to say why.
VOCABULARY_INSIDE = {
    "repr_c_ffi_types": (REPR_C, FOREIGN_ABI_TYPES),
}
# ... and counted inside a `fn`'s *parameter list* only, the shape
# `INSTRUMENTS_PARAMS` uses for the same reason. A `*mut Error` in a struct
# *field* is the owning struct's own storage, not a channel a caller lends a
# callee -- p31-2's ruling that a struct's raw fields are the struct's
# invariant and not each method's -- so it was never this number's debt. The
# tree-wide needle this replaces booked exactly one such field,
# `lua/xdiff.rs`'s `HunkContext.err`, and that row is what kept a metric whose
# subject is retired off zero.
VOCABULARY_PARAMS = {
    "error_out_params": re.compile(r"\*mut\s+Error\b"),
}
# The two halves of `const_int_alias`, which needs a pass over the whole tree
# before it can count anything: first every `type X = Y;` in the tree, then
# every `pub const NAME: T`. An alias counts when its chain bottoms out in one
# of the primitives below; a constant counts when its type is such an alias.
# Written as two needles rather than a list of alias names so that the set
# prunes and extends itself — see the doc block.
INT_ALIAS_DECL = re.compile(
    r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:::)?(?:[A-Za-z0-9_]+::)*"
    r"([A-Za-z_][A-Za-z0-9_]*)\s*;"
)
INT_PRIMITIVES = frozenset(
    (
        "c_char",
        "c_schar",
        "c_uchar",
        "c_short",
        "c_ushort",
        "c_int",
        "c_uint",
        "c_long",
        "c_ulong",
        "c_longlong",
        "c_ulonglong",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
    )
)
PUB_CONST_DECL = re.compile(
    r"\bpub const ([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:::)?(?:[A-Za-z0-9_]+::)*"
    r"([A-Za-z_][A-Za-z0-9_]*)\s*="
)

# Declarations of a `_T` type, counted as a *set* of names over the whole
# tree: `type` aliases included, because the integer names are aliases.
T_SUFFIX_DECL = re.compile(
    r"\b(?:struct|enum|union|type)\s+([A-Za-z_][A-Za-z0-9_]*_T)\b"
)
# The raw graph pointers, counted inside `fn` signature spans only, and only
# *outside* `winlayer/` -- the way `curwin_raw` is counted. `const` as well as
# `mut`: a `*const Window` parameter is the same C vocabulary and the same
# retype, and counting only the `mut` half would have let 37 of them sit
# outside the number the exit clause is written against.
#
# The residue inside the home is the floor and is deliberately not zero:
# `Win::new`/`from_raw`/`raw`/`current_raw` for each of the three handles are
# where an address becomes an identity and back; `window_at`/`buffer_at`/
# `tabpage_at` are the lookups that must still speak in addresses (a caller
# holding a pointer an autocommand may already have freed cannot ask for an
# identity -- reading one is the use-after-free); and `win_col_off` is a C ABI
# the functional suite calls through `ffi.cdef`. Every one of them is the
# *reason* the rest of the tree has none.
#
# One row sits *outside* the home and is counted: `optionstr::
# init_buf_string_options(at: *mut Buffer)`, which is why this number is 1 and
# not 0. It writes a freshly allocated buffer's string options before anything
# may read or drop them -- a zeroed `Option<XString>` is `Some` over a null
# pointer, because the niche is the vector's capacity -- so its argument is a
# block that is not yet a valid `Buffer` and a `&mut` would assert what is not
# true yet. Its doc comment carries the full argument, including why
# `&mut MaybeUninit<Buffer>` is not an improvement. The row retires when a
# buffer is built as a value and moved into place rather than allocated zeroed
# and patched up.
RAW_WIN_BUF = re.compile(r"\*(?:mut|const)\s+(?:Window|Buffer|Tabpage)\b")
# The graph objects behind an exclusive Rust borrow, same spans. `&mut` is
# `noalias`, and `curwin`/`curbuf` alias every window and buffer the editor
# passes around: 50 window-taking functions read `curwin` in the same body and
# 19 of those also write a `w_` field, so a `&mut Window` parameter is UB the
# moment the argument *is* the current window. The handle (`Win`/`Buf`/
# `TabPage`) is the parameter type -- it is `Copy` and borrows per access --
# and this number keeps the retype from landing on `&mut` by accident.
# The layout tree's node pointer, same spans and the same carve-out. A frame
# is reached from `w_frame`, `tp_topframe`, `tp_snapshot` and its own four
# links, all of which an autocommand can free under a caller; the identity
# (`FrameId`) is what survives such a call, and a `*mut Frame` in a signature
# is a caller that was handed an address instead. A `*mut *mut Frame`
# out-parameter counts once: the inner `*mut Frame` is the frame it names.
RAW_FRAME = re.compile(r"\*(?:mut|const)\s+Frame\b")
MUT_WIN_BUF_REF = re.compile(r"&\s*mut\s+(?:Window|Buffer|Tabpage)\b")
# The transpiler's parameter abbreviations, also inside `fn` signature spans
# only: the leading `\b` plus the optional `_` catches both `buf:` and the
# `_buf:` an unused parameter is spelled with, while keeping the needle off
# `bufp:` and `old_buf:` — a name with a qualifier in front of it already says
# more than the bare abbreviation does. `:` is what makes it a *binding* and
# not a mention, so `ptr.add` and `let buf = …` are not counted.
ABBREV_PARAM = re.compile(
    r"\b_?(?:wp|tp|eap|rettv|argvars|cap|oap|xp|fp|lp|sp|pp|cp)\s*:"
)
# `ptr` is the third spelling whose expansion depends on the type, and it
# splits the way `buf` does: a pointer into a *thing* has a better name (the
# line's `text`, the `name` being looked up, the `pattern` being matched),
# and a pointer to raw memory does not -- `ptr` is what Rust itself calls the
# argument of `dealloc`. So it counts only when the type names something
# other than untyped memory.
ABBREV_PTR_PARAM = re.compile(r"\b_?ptr\s*:([^,)]*)")
# The raw-memory types, read the same way `BUFFER_TYPE` reads `buf`'s: the
# void and byte pointers the allocator seams take, the address `alloc_log`
# records, and a bare type parameter, which names nothing by construction.
RAW_MEMORY_TYPE = re.compile(r"\b(?:c_void|u8|uint8_t|usize|T)\b")
# `buf`/`bp` are the two abbreviations whose expansion depends on the type:
# `buf` for a byte buffer is idiomatic Rust and stays, so only a parameter
# whose type names a *buffer object* counts. The type is read up to the next
# `,` or `)`, which is the whole of every shape this tree writes
# (`*mut Buffer`, `Option<Buf>`, `&mut Buf`, `BufferHandle`); a buffer named
# only after a comma inside a generic (`Result<E, Buffer>`) would be missed,
# and none exists.
ABBREV_BUF_PARAM = re.compile(r"\b_?(?:buf|bp)\s*:([^,)]*)")
# The buffer-object types: the raw struct, its safe handle and reference
# wrappers, and the API's integer handle. `\b` keeps `NumBuf`, `KeyBuffer`,
# `EnvBuf` and `uv_buf_t` — byte buffers, every one — out.
BUFFER_TYPE = re.compile(r"\b(?:Buffer|BufferRef|BufferHandle|Buf)\b")
# The two subtrees the exit clause carves out. `vterm/` is a port of libvterm
# and `lua/` of the Lua bindings; both keep the upstream project's own
# spellings, so renaming their parameters would move them away from the code
# they are read against.
ABBREV_PARAM_EXEMPT = {
    "crates/nvim/src/lua/": "the Lua bindings keep LuaJIT's own spellings",
    "crates/nvim/src/vterm/": "a port of libvterm, read against its C",
}
# The third carve-out, and the only one keyed by a function rather than a
# directory: apigen turns an API function's *parameter names* into the RPC
# surface. They are published by `nvim_get_api_info()` (the api-info blob in
# api/private/metadata.rs) and printed by every `Invalid '<name>'` message
# (`*err_param` in the generated Lua wrappers), so upstream's spelling is the
# contract and renaming one is a breaking API change, not a cleanup. The set
# is read from apigen's own spec rather than guessed from the directory: only
# the methods the generator dispatches are frozen, and the helpers around them
# under `api/` are ordinary code that renames like anything else.
API_DIR = "crates/nvim/src/api/"

# --------------------------------------------------------------------------
# The instruments. One whole-tree number per dialect of the C *calling
# convention* -- the parameters that are pointers, the value model, the state
# structs, the error channel and the tree's shape. See "the instruments" in
# the doc block for what each counts and what retires it. Every one of them
# decomposes per file, so `--dimension NAME` can list where the debt sits.

# The C printf-family variadics: `variadic.rs` gives each a CArg-checked macro
# of the same name.
# `vim_snprintf*`/`printf_string` left when they stopped being variadics (their
# macros build a typed argument array), and `nlua_push_errstr` when it began
# taking its message whole.
VARIADIC_CALLEES = (
    "snprintf",
    "fprintf",
    "printf",
    "sscanf",
    "lua_pushfstring",
    "luaL_error",
)
# The macros whose `unsafe { name!(..) }` is a wrapper of the callee `name`:
# the variadics', and the editor formatter's, whose macros build the typed
# argument array for the pointer fronts of the same name.
ARGUMENT_MACROS = VARIADIC_CALLEES + (
    "vim_snprintf",
    "vim_snprintf_add",
    "vim_snprintf_safelen",
    "printf_string",
)

# Counted over every masked file, tree-wide.
INSTRUMENTS = {
    # The `(` is what keeps the *pointer* forms apart from the `_len` ones
    # that take a slice's length and are the safe bodies underneath them:
    # `utfc_ptr2len(` matches, `utfc_ptr2len_len(` does not.
    "mbyte_raw": re.compile(r"\butfc?_ptr2(?:char|len|cells)\("),
    "bytes_at": re.compile(r"\bbytes_at\("),
    "vval_raw": re.compile(r"\.vval\."),
    # The declaration, not the reads: what phase 31 retires is a *cell*, and a
    # metric over reads would book narrowing forty of them to one accessor as
    # no progress at all. Any visibility may precede `static`.
    "global_cells": re.compile(
        r"\bstatic\s+[A-Za-z_][A-Za-z0-9_]*\s*:\s*GlobalCell\s*<"
    ),
    "cell_raw_ptr": re.compile(r"\bGlobalCell\s*<\s*\*(?:mut|const)\b"),
    # The transpiler's `goto` residue: c2rust renders a forward jump as a
    # labelled *block* it can `break` out of. A labelled `loop`/`while`/`for`
    # is Rust's own multi-level `break`/`continue` and is not counted here:
    # `LABELED_LOOPS` lists them (`--dimension labeled_loops`) without a
    # ratchet. `'a: 'b` (a lifetime bound) cannot match -- what follows the
    # colon has to open a block.
    "labeled_blocks": re.compile(r"'[A-Za-z_][A-Za-z0-9_]*\s*:\s*\{"),
    # The fixed-size C string buffer: a `[c_char; N]` type (a local, a
    # parameter, a return, a field) or a `[0 as c_char; N]` initialiser
    # (`vec![..]` included -- the same buffer on the heap). One array is one
    # site: the first alternative eats a typed `let` together with its
    # initialiser, so `let b: [c_char; N] = [0 as c_char; N]` is one and not
    # two, and a bare `[0; N]` is only ever counted through the type in front
    # of it. Masked text, so a comment quoting the shape is not a site.
    "cchar_scratch": re.compile(
        r"\[\s*c_char\s*;[^\]]*\]\s*=\s*(?:vec!\s*)?\[\s*0(?:\s+as\s+c_char)?\s*;"
        r"|\[\s*c_char\s*;"
        r"|\[\s*0\s+as\s+c_char\s*;"
    ),
}
# The labelled loops `labeled_blocks` leaves out: Rust's multi-level
# `break 'outer`/`continue 'outer`, not a goto. Listed, not ratcheted --
# rewriting one as a flag would be worse code to feed a needle.
LABELED_LOOPS = re.compile(r"'[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:loop|while|for)\b")
# `types/`: c2rust's one `types_defs.h`-sized namespace. Phase 34 moves each
# type next to the code that owns it, so the number is the count of *files*
# still living there, not of the types inside them.
TYPES_HOME = {"crates/nvim/src/types/": "the transpiler's one type namespace"}

# Counted inside a `fn`'s *parameter list* only -- not the return type, which
# `INSTRUMENTS_RETURNS` counts separately so the two never double-book a
# `-> *mut c_char`.
# `C_CHAR` (above) is every spelling of `c_char`.

INSTRUMENTS_PARAMS = {
    # The roadmap's needle exactly: a parameter *directly* typed as a raw
    # pointer. A pointer nested inside another type (`Option<*mut T>`, a
    # callback's own signature) is not counted -- retyping the outer type is
    # what retires those, and this number is about the parameter itself.
    # Qualification lands *after* the `*mut`, so this one needs no widening:
    # `: *const ::core::ffi::c_char` already matches.
    "raw_ptr_params": re.compile(r":\s*\*\s*(?:mut|const)\b"),
    # These four are nesting-blind on purpose: what is being retired is the
    # *type*, wherever in the parameter list it appears.
    "raw_cstr_params": re.compile(rf"\*(?:mut|const)\s+{C_CHAR}\b"),
    "typval_raw_params": re.compile(r"\*(?:mut|const)\s+TypVal\b"),
    # api/'s error channel as a borrow rather than a pointer -- the landing
    # place `error_out_params` retypes to, and itself debt: the answer is a
    # `Result<T, Error>` return. Counting both is what keeps a sweep from
    # booking `*mut Error` -> `&mut Error` as the end of the job.
    "api_err_params": re.compile(r"&\s*mut\s+Error\b"),
    "exarg_raw": re.compile(r"\*(?:mut|const)\s+ExArg\b"),
    "cmdarg_raw": re.compile(r"\*(?:mut|const)\s+CmdArg\b"),
}
# ... and inside the declared return type only.
INSTRUMENTS_RETURNS = {
    "raw_cstr_returns": re.compile(rf"\*(?:mut|const)\s+{C_CHAR}\b"),
}

# The head of a *braced* `struct`/`union` item -- the same exclusions
# `derive_copy` makes, and for the same reason: a tuple struct's members are
# positional and an enum's payload is a selector, neither of which is the
# named storage this counts. The generic list and a `where` clause may sit
# between the name and the brace, and `[^{;]*` is what stops either running
# past the body's opening brace or a bodyless declaration's `;`.
STRUCT_HEAD = re.compile(
    r"\b(?:struct|union)\s+[A-Za-z_][A-Za-z0-9_]*\s*(?:<[^{;]*>)?\s*(?:where[^{;]*)?\{"
)


# ... and inside a braced aggregate's field list only, over `struct_bodies`.
# One match per *field*: the leading run is `[ \t]*` and not `\s*` so that a
# match cannot start on the blank line above and be attributed to the wrong
# field, and the type is read to the field's comma, which is what makes
# `[*mut c_char; 10]` one field of debt rather than ten. A field whose type
# merely *mentions* the pointer -- `Option<*mut c_char>`, `*mut *mut c_char`,
# `[[*const c_char; 2]; 16]` -- is the same storage and counts once.
INSTRUMENTS_FIELDS = {
    "raw_cstr_fields": re.compile(
        r"^[ \t]*(?:pub(?:\s*\([^)]*\))?[ \t]+)?[A-Za-z_][A-Za-z0-9_]*[ \t]*:"
        rf"[^,;\n]*\*(?:mut|const)[ \t]+{C_CHAR}\b",
        re.M,
    ),
}

# A character literal cast to C's `int`, in every spelling of the type
# `c_int_returns` accepts. This is the one needle that cannot run against the
# masked source -- `mask()` blanks a char literal, quotes and all -- so it is
# matched against the *raw* text and confirmed as code by checking that the
# `as` it captured survived masking. A `b'x'` keeps its `b` through masking,
# which is why the prefix is optional here rather than required.
#
# `c_int::from(b'x')` is the same character respelled the way the cast lints
# ask for -- a byte widened into C's `int` -- so it is counted too: the cast
# deny turned many `b'x' as c_int` into it without changing the domain. The
# `from` is the token checked against the masked copy there.
CHAR_AS_C_INT = re.compile(
    r"b?'(?:\\.|[^'\\\n])'\s*(as)\s+"
    r"(?:(?:::)?(?:core|std)::ffi::|(?:::)?libc::)?c_u?int\b"
    r"|\b(?:(?:::)?(?:core|std)::ffi::|(?:::)?libc::)?c_u?int::(from)"
    r"\(\s*b?'(?:\\.|[^'\\\n])'\s*\)"
)


# The message entry points `emit_sentinel` follows, and the sentinel answer
# that may come straight after one. The emitter is a call (or a macro call):
# its argument list is skipped with `balanced`, so a nested call or a masked
# format string cannot end it early. The sentinel is the *next* statement --
# a `return x;` or a tail `x }` -- after the `;` that ends the emitting one,
# so a message that is followed by bookkeeping first is not counted: the
# needle is the shape, not every path that reaches a sentinel. A line number's
# sentinels are spelled `LineNr::ZERO`/`LineNr::new(-1)` since it became a
# newtype; they are the same `0`/`-1`.
EMIT_CALL = re.compile(
    r"\b(?:emsg|semsg!|iemsg|siemsg!|emsg_multiline|emsg_invreg|internal_error"
    r"|give_warning|smsg!|swmsg!|msg_warn\w*|api_error!|nlua_error"
    r"|report_emsg_multiline|msg_puts\w*|msg_outtrans\w*|msg_\w*hl|msg_bytes"
    r"|msg_cstr)\s*\("
)
EMIT_SENTINEL = re.compile(
    r"\s*(?:return\s+)?(?:false|None|-1|0|LineNr::(?:ZERO|new\(-1\))|FAIL)\s*[;}]"
)


# A local message wrapper: a free `fn` named `err*` or `emsg_*` that answers
# nothing and whose body calls an EMIT_CALL -- `err0(msg)` in eval/fs/dir.rs,
# `emsg_static` in eval/vars/store.rs. A call to one is a message as much as
# the `emsg` it wraps, so `emit_sentinel` follows them too; without this the
# wrappers hid their callers' sentinels (a wrapper inlined showed its count).
# Methods (`self` first) and a wrapper that answers something (`err_msg ->
# Option<CString>`, the API's `err_* -> Error` builders) are not wrappers.
EMIT_WRAPPER_NAME = re.compile(r"^(?:err\w*|emsg_\w+)$")


def emit_wrappers(tree):
    """The names of every local message wrapper in `tree` (see above)."""
    names = set()
    for masked in tree.values():
        for sig in fn_signatures(masked):
            if not EMIT_WRAPPER_NAME.match(sig.name) or sig.returns != "()":
                continue
            if re.match(r"\(\s*(?:&\s*(?:mut\s+)?)?(?:mut\s+)?self\b", sig.params):
                continue
            body = masked.find("{", sig.end)
            if body == -1 or masked[sig.end : body].strip():
                continue
            if EMIT_CALL.search(masked, body, matching_brace(masked, body)):
                names.add(sig.name)
    return names


def emit_call_re(wrappers):
    """EMIT_CALL widened to the wrapper names `emit_wrappers` found."""
    if not wrappers:
        return EMIT_CALL
    names = "|".join(sorted(map(re.escape, wrappers)))
    return re.compile(rf"{EMIT_CALL.pattern}|(?<![.\w])(?<!fn )(?:{names})\s*\(")


# Macros that build text without saying it. `msg_bytes`/`msg_cstr` are
# message *formatters* as well as names on EMIT_CALL, so one in a log line's
# arguments (`logmsg!(.., "{}", msg_bytes(addr))`) is no message: the
# `return false` after `server_stop`'s "Not listening" log entry was counted
# as one until the needle learned these (30 -> 29, an instrument change).
QUIET_MACRO = re.compile(
    r"\b(?:logmsg|format|format_args|write|writeln|panic|unreachable|todo"
    r"|assert\w*|debug_assert\w*)!\s*\("
)


def quiet_spans(masked):
    """The argument lists of every QUIET_MACRO call in `masked`."""
    return [
        (m.end() - 1, balanced(masked, m.end() - 1, "(", ")"))
        for m in QUIET_MACRO.finditer(masked)
    ]


def emit_sentinels(masked, emit_call=EMIT_CALL):
    """How many EMIT_CALLs in `masked` are followed by a sentinel answer."""
    found = 0
    inside = 0
    quiet = quiet_spans(masked)
    for match in emit_call.finditer(masked):
        if match.start() < inside:
            # An emitter in another's arguments (`semsg!(.., msg_cstr(p))`)
            # is the same message: the outer call is the statement.
            continue
        if any(a <= match.start() < b for a, b in quiet):
            continue
        close = balanced(masked, match.end() - 1, "(", ")")
        inside = close
        end = masked.find(";", close)
        if end != -1 and EMIT_SENTINEL.match(masked, end + 1):
            found += 1
    return found


# A `Failed` made where nothing was said: `Err(Failed)` or `.ok_or(Failed)`
# in a function body holding no EMIT_CALL (wrappers included) and no `?` or
# `map_err` -- the two ways a callee's error, said by the callee, reaches it.
# `Failed` means "a message is already out", so one made in a body that says
# nothing and passes nothing on is the lie p41's rulings x/xi named: a
# refusal that says nothing is a split answer (`Option`, an enum,
# `IoFailed`). A pattern (`Err(Failed) =>`, `let Err(Failed) =`, a
# comparison) is not a construction and is not counted; neither is test
# code. `--dimension silent_failed` lists them.
SILENT_FAILED = re.compile(r"\bErr\(\s*Failed\s*\)|\.ok_or\(\s*Failed\s*\)")
SILENT_FAILED_NOT = re.compile(r"\s*(?:=>|=[^=>]|\|)")
CALLEE_ERROR = re.compile(r"\?|\bmap_err\b")


def silent_failed(masked, emit_call=EMIT_CALL):
    """How many `Failed`s `masked` makes in a body that says nothing."""
    found = 0
    tests = test_module_lines(masked)
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]
    for sig in fn_signatures(masked):
        body = masked.find("{", sig.end)
        if body == -1 or masked[sig.end : body].strip().startswith(";"):
            continue
        if ";" in masked[sig.end : body]:
            continue  # a declaration without a body
        end = matching_brace(masked, body)
        text = masked[body:end]
        if emit_call.search(text) or CALLEE_ERROR.search(text):
            continue
        for match in SILENT_FAILED.finditer(text):
            before = text[: match.start()].rstrip()
            if before.endswith(("let", "if let", "|", "==", "!=", "while let")):
                continue
            if SILENT_FAILED_NOT.match(text, match.end()):
                continue
            if bisect_right(starts, body + match.start()) - 1 in tests:
                continue
            found += 1
    return found


# A function long enough that nobody reads it: c2rust translated a C function
# whole, and a 700-line body is a family of operations sharing one stack
# frame. Phase 34 splits them along the seams their arguments already draw.
LONG_FN_LINES = 200
# Generated output is exempt -- a table apigen emits is one `fn` by
# construction and splitting it is the generator's business, not a reviewer's.
# The marker is the module doc line every generator writes, so a new generated
# module is exempt the day it lands and a hand-written file cannot claim the
# exemption by accident: `help/tags.rs` says "generated by this code" in prose
# and is not matched, `ex_cmds/ecmd/mod.rs` says "Do not edit" in an item doc
# (`///`, not `//!`) and is not matched either.
GENERATED_MARKER = re.compile(
    r"^//![^\n]*\bgenerated\b[^\n]*(?:\bby tools/|\bfile\b)", re.I | re.M
)
# How far into a file the marker has to sit to be the module's own header.
GENERATED_HEAD = 2048

# `const NAME: Type = value;` -- the constant families c2rust duplicated into
# every translation unit that used them. The value is read from the *raw*
# source rather than the masked copy, because masking blanks a string literal
# to spaces and would make `c"foo"` and `c"bar"` compare equal, propping the
# count up with a duplicate nobody could ever remove.
# The value group starts immediately after the `=`, not after the whitespace
# behind it: a masked string literal *is* whitespace, so a greedy `\s*` ate
# every `const NAME: &str = "..."` value in the tree and made them all compare
# equal. The span is normalised for whitespace when it is read back.
CONST_DECL = re.compile(r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^=;{}]+=([^;]*);")

# Every instrument, in the order the phases retire them: the pointer
# parameters (29-31), the value model (30), the state (31), the error channel
# and the numbers (32), the Lua seam (33), and the shape (34).
INSTRUMENT_KEYS = (
    "unsafe_fns",
    "unsafe_fns_without_raw_params",
    "safety_on_safe_fns",
    "wrappers",
    "raw_ptr_params",
    "raw_cstr_params",
    "raw_cstr_returns",
    "raw_cstr_fields",
    "mbyte_raw",
    "bytes_at",
    "vval_raw",
    "typval_raw_params",
    "exarg_raw",
    "cmdarg_raw",
    "global_cells",
    "cell_raw_ptr",
    "api_err_params",
    "char_as_c_int",
    "emit_sentinel",
    "labeled_blocks",
    "long_fns",
    "dup_consts",
    "types_files",
    "untested_dirs",
    "cchar_scratch",
)

FORBID = "#![forbid(unsafe_code)]"
DENY_UNSAFE_OP = "#![deny(unsafe_op_in_unsafe_fn)]"
# A module's claim to have finished its casts. `.` spans newlines so the list
# may be wrapped; `clippy::cast_lossless` is the family's marker (see above).
DENY_CASTS = re.compile(r"#!\[deny\([^\]]*\bclippy::cast_lossless\b", re.DOTALL)

# The files that do not carry the cast deny, each with why. This is the floor
# `files_without_deny_casts` stands on, and it is checked both ways by
# `check_cast_deny_floor`: a file without the deny that is not here fails the
# run, and so does a row whose file now carries the deny (or is gone) -- drop
# the row and lower the count. Three classes:
#   perimeter  a perimeter file's own casts, out of the migration's scope;
#   root       a module root whose `#![deny]` would reach a perimeter file
#              below it that has casts, so it lands when they do;
#   home       `narrow.rs`, the one module whose job is to write the
#              deliberate narrowings once (a float-to-integer conversion has
#              no cast-free spelling), so that everything else can deny them.
#              It is permanent.
CAST_DENY_FLOOR = {
    "crates/nvim/src/narrow.rs": "home: the named, documented narrowings",
    "crates/nvim/src/lib.rs": "root: reaches xdiff/, os/, tui/input.rs",
    "crates/nvim/src/tui/mod.rs": "root: reaches tui/input.rs",
    "crates/nvim/src/os/mod.rs": "root: reaches os/input.rs, os/fs/",
    "crates/nvim/src/xdiff/mod.rs": "root: reaches the vendored xdiff files",
    "crates/nvim/src/tui/input.rs": "perimeter: timeout/mode widths (5)",
    "crates/nvim/src/os/input.rs": "perimeter: option widths, mouse codes, key bytes (35)",
    "crates/nvim/src/os/fs/mod.rs": "perimeter: modes and write counts (5); root over meta.rs",
    "crates/nvim/src/os/fs/meta.rs": "perimeter: stat mode, xattr sizes (6)",
    "crates/nvim/src/lua/treesitter/ranges.rs": "perimeter: a Lua number as a TSPoint row (2)",
    "crates/nvim/src/xdiff/xdiffi/mod.rs": "perimeter: vendored libxdiff (32)",
    "crates/nvim/src/xdiff/xhistogram.rs": "perimeter: vendored libxdiff (20)",
    "crates/nvim/src/xdiff/xpatience.rs": "perimeter: vendored libxdiff (18)",
    "crates/nvim/src/xdiff/xprepare.rs": "perimeter: vendored libxdiff (19)",
    "crates/nvim/src/xdiff/xtypes.rs": "perimeter: vendored libxdiff (21)",
}
# The types that keep `#[derive(Copy)]` (`derive_copy`) and `#[repr(C)]`
# (`repr_c_editor_state` and `repr_c_ffi_types`) for good: the floors those
# needles fall to. A row is `"path::Type"` (one braced struct/union) or a bare
# `"path"` (every one in that file), and its reason opens with a class. Both
# lists are checked by `check_keeper_floors` on every run, but only one way: a
# row that excuses nothing (the type lost the attribute, moved, or is gone)
# fails the run -- move or drop the row. A site that is *not* a row is still
# counted by its needle and is not an error; it is what is left to remove, and
# `--dimension copy_keepers` / `--dimension repr_c_floor` list it as OFF THE
# FLOOR. When a needle reaches its floor, its count equals the rows'.
#
# COPY_KEEPERS classes:
#   ffi     a C library's (or vendored C code's) layout, copied as C copies it;
#   handle  a `{pointer, id}` or a pointer-and-offset view, copied by design;
#   value   at most five plain fields, read whole (a position, a range, a
#           span's ends): cheaper to copy than to borrow.
# A struct of six or more fields is a field bag, and is not a keeper unless a
# measured hot path copies it -- then its row says so, with the number.
COPY_KEEPERS = {
    "crates/nvim/src/buffer/close.rs::Disposition": "value",
    "crates/nvim/src/buffer_updates.rs::Corner": "value",
    "crates/nvim/src/bufwrite/autocmds.rs::WriteMode": "value",
    "crates/nvim/src/bufwrite/autocmds.rs::OpMarks": "value",
    "crates/nvim/src/bufwrite/mod.rs::WriteRequest": "value",
    "crates/nvim/src/charset/display.rs::Chars": "value",
    "crates/nvim/src/clipboard.rs::ClipboardState": "value",
    "crates/nvim/src/decoration/mod.rs::ShRedraw": "value",
    "crates/nvim/src/decoration/query.rs::VirtLineRef": "value",
    "crates/nvim/src/decoration/state.rs::HlSource": "value",
    "crates/nvim/src/decoration/state.rs::DecorLists": "value",
    "crates/nvim/src/diff/mod.rs::DiffHunk": "value",
    "crates/nvim/src/digraph/mod.rs::Digraph": "value",
    "crates/nvim/src/eval/list/mod.rs::Item": "value",
    "crates/nvim/src/eval/lval.rs::Span": "value",
    "crates/nvim/src/eval/vars/assign.rs::VarList": "value",
    "crates/nvim/src/ex_cmds/mod.rs::SubResult": "value",
    "crates/nvim/src/ex_cmds/sort.rs::StringOrder": "value",
    "crates/nvim/src/ex_getln/mod.rs::CpWinInfo": "value",
    "crates/nvim/src/extmark/mod.rs::Extent": "value",
    "crates/nvim/src/getchar/mapmatch.rs::Candidate": "value",
    "crates/nvim/src/getchar/mod.rs::GotcharsState": "value",
    "crates/nvim/src/indent/tabstop.rs::TabStops": "value",
    "crates/nvim/src/insexpand/mod.rs::CptSource": "value",
    "crates/nvim/src/lua/treesitter/mod.rs": "ffi: tree-sitter's public structs",
    "crates/nvim/src/lua/xdiff.rs::Hunk": "value",
    "crates/nvim/src/mapping/table.rs::MapAt": "value",
    "crates/nvim/src/mark/adjust.rs::LineShift": "value",
    "crates/nvim/src/mark/adjust.rs::ColShift": "value",
    "crates/nvim/src/marktree/check.rs::MarkEnd": "value",
    "crates/nvim/src/mbyte/cells.rs::CellWidthRange": "value",
    "crates/nvim/src/mbyte/utf8/text.rs::Prefix": "value",
    "crates/nvim/src/memfile/table.rs::BlockId": "value",
    "crates/nvim/src/memline/blocks.rs::PointerEntry": "value",
    "crates/nvim/src/mouse/geom.rs::MousePos": "value",
    "crates/nvim/src/mouse/mod.rs::Landed": "value",
    "crates/nvim/src/move/page.rs::Saved": "value",
    "crates/nvim/src/mpack/mpack_core.rs::TokBuf": "value",
    "crates/nvim/src/mpack/rpc.rs::RpcMessage": "value",
    "crates/nvim/src/mpack/rpc.rs::Slot": "value",
    "crates/nvim/src/mpack/rpc.rs::Header": "value",
    "crates/nvim/src/mpack/token.rs::Tok": "value",
    "crates/nvim/src/normal/showcmd.rs::ShowCmd": "value",
    "crates/nvim/src/normal/visual.rs::VisualSelection": "value",
    "crates/nvim/src/normal/visual.rs::VisualState": "value",
    "crates/nvim/src/ops/mod.rs::RedoVisual": "value",
    "crates/nvim/src/option/check.rs::BinSave": "value",
    "crates/nvim/src/popupmenu/state.rs::PumWant": "value",
    "crates/nvim/src/quickfix/stack.rs::Qi": "value",
    "crates/nvim/src/quickfix/stack.rs::Qfl": "value",
    "crates/nvim/src/regexp/bt/state.rs::Brace": "value",
    "crates/nvim/src/regexp/mod.rs::TimeBudget": "value",
    "crates/nvim/src/regexp/mod.rs::RegSub": "value",
    "crates/nvim/src/regexp/mod.rs::RegSubs": "value",
    "crates/nvim/src/regexp/nfa/build.rs::Frag": "value",
    "crates/nvim/src/regexp/nfa/list.rs::NfaThread": "value",
    "crates/nvim/src/regexp/nfa/list.rs::NfaPim": "value",
    "crates/nvim/src/regexp/pos.rs::Capture": "value",
    "crates/nvim/src/regexp/pos.rs::SavedInput": "value",
    "crates/nvim/src/regexp/substitute.rs::Expanding": "value",
    "crates/nvim/src/search/find.rs::Found": "value",
    "crates/nvim/src/search/find.rs::StartPos": "value",
    "crates/nvim/src/sign/vimscript.rs::PlaceArgs": "value",
    "crates/nvim/src/spell/wordtree.rs::Tree": "value",
    "crates/nvim/src/spellsuggest/score.rs::Alternative": "value",
    "crates/nvim/src/spellsuggest/soundalike.rs::KeepCapLevel": "value",
    "crates/nvim/src/statusline/stl/arena.rs::StlItem": "value",
    "crates/nvim/src/strings/charindex.rs::CharLen": "value",
    "crates/nvim/src/syntax/mod.rs::sp_syn": "value",
    "crates/nvim/src/syntax/mod.rs::PatOffsets": "value",
    "crates/nvim/src/textformat/para.rs::Leader": "value",
    "crates/nvim/src/tui/terminfo/param.rs::Param": "value",
    "crates/nvim/src/tui/terminfo/param.rs::Slot": "value",
    "crates/nvim/src/types/buffer.rs::BufferRef": "value",
    "crates/nvim/src/types/buffer.rs::DiffLineChange": "value",
    "crates/nvim/src/types/buffer.rs::SynTime": "value",
    "crates/nvim/src/types/channel.rs::InternalState": "value",
    "crates/nvim/src/types/channel.rs::StderrState": "value",
    "crates/nvim/src/types/decoration.rs::DecorExt": "value",
    "crates/nvim/src/types/decoration.rs::DecorHighlightInline": "value",
    "crates/nvim/src/types/decoration.rs::RangeHighlight": "value",
    "crates/nvim/src/types/decoration.rs::DecorRange_data_ui": "value",
    "crates/nvim/src/types/event.rs::socket_watcher_uv_pipe": "value",
    "crates/nvim/src/types/event.rs::socket_watcher_uv_tcp": "ffi: a libuv TCP handle and its addrinfo",
    "crates/nvim/src/types/event.rs::stream_uv": "value",
    "crates/nvim/src/types/ex_cmds.rs::CmdParseMagic": "value",
    "crates/nvim/src/types/ex_getln.rs::CmdlineColorChunk": "value",
    "crates/nvim/src/types/expressions.rs::ExprASTError": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeAssignment": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeComparison": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeEnvironment": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeFigure": "value",
    "crates/nvim/src/types/expressions.rs::ExprFigureGuesses": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeFloat": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeInteger": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeOption": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeRegister": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeTernary": "value",
    "crates/nvim/src/types/expressions.rs::ExprNodeVariable": "value",
    "crates/nvim/src/types/extmark.rs::ExtmarkSavePos": "value",
    "crates/nvim/src/types/fold.rs::FoldInfo": "value",
    "crates/nvim/src/types/grid.rs::GridView": "handle: a grid and an offset into it",
    "crates/nvim/src/types/hashtab.rs::HashItem": "value",
    "crates/nvim/src/types/helpers.rs::TryState": "value",
    "crates/nvim/src/types/highlight.rs::ColorItem": "value",
    "crates/nvim/src/types/highlight.rs::HlEntry": "value",
    "crates/nvim/src/types/libc.rs": "ffi: the C library's and pthread's aggregates",
    "crates/nvim/src/types/mapping.rs::MapTwin": "value",
    "crates/nvim/src/types/mark.rs::FileMarkView": "value",
    "crates/nvim/src/types/mark.rs::SavedMark": "value",
    "crates/nvim/src/types/marktree.rs::MTKey": "value",
    "crates/nvim/src/types/marktree.rs::MTPair": "value",
    "crates/nvim/src/types/marktree.rs::MTPos": "value",
    "crates/nvim/src/types/marktree.rs::PathStep": "value",
    "crates/nvim/src/types/mbyte.rs::CharInfo": "value",
    "crates/nvim/src/types/mbyte.rs::StrCharInfo": "handle: a text position and its char",
    "crates/nvim/src/types/memline.rs::ChunkSize": "value",
    "crates/nvim/src/types/memline.rs::InfoPtr": "value",
    "crates/nvim/src/types/misc.rs::nvim_stats_s": "value",
    "crates/nvim/src/types/multiqueue_list.rs::Handle": "value",
    "crates/nvim/src/types/parser.rs::ParserHighlightChunk": "value",
    "crates/nvim/src/types/parser.rs::ParserPosition": "value",
    "crates/nvim/src/types/pos.rs::LPos": "value",
    "crates/nvim/src/types/pos.rs::Pos": "value",
    "crates/nvim/src/types/runtime.rs::SnPrl": "value",
    "crates/nvim/src/types/search.rs::SearchOffset": "value",
    "crates/nvim/src/types/sign.rs::SignItem": "value",
    "crates/nvim/src/types/sign.rs::SignTextAttrs": "value",
    "crates/nvim/src/types/spell.rs::LangP": "value",
    "crates/nvim/src/types/spell.rs::SpellTab": "value",
    "crates/nvim/src/types/statusline.rs::StlClickDefinition": "value",
    "crates/nvim/src/types/statusline.rs::StlHlRun": "value",
    "crates/nvim/src/types/termkey.rs": "ffi: libtermkey's key record",
    "crates/nvim/src/types/termkey_internal.rs::keyinfo": "value",
    "crates/nvim/src/types/types.rs::MTDamage": "value",
    "crates/nvim/src/types/types.rs::MTDamagePair": "value",
    "crates/nvim/src/types/types.rs::MsgpackRpcRequestHandler": "value",
    "crates/nvim/src/types/typval.rs::ScriptCtx": "value",
    "crates/nvim/src/types/ugrid.rs::UCell": "value",
    "crates/nvim/src/types/ui.rs::UIClientHandler": "handle: a static name and a fn pointer",
    "crates/nvim/src/types/undo.rs::UndoLink": "value",
    "crates/nvim/src/types/undo.rs::VisualInfo": "value",
    "crates/nvim/src/types/uv.rs": "ffi: libuv's structs, copied as C copies them",
    "crates/nvim/src/types/vterm.rs": "ffi: libvterm's public types",
    "crates/nvim/src/types/vterm_internal.rs": "ffi: libvterm's internal state",
    "crates/nvim/src/undo/apply/time.rs::Aim": "value",
    "crates/nvim/src/undo/apply/time.rs::UndoCounters": "value",
    "crates/nvim/src/undo/store.rs::Marks": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprToken": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenAssignment": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenNumber": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenError": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenVar": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenOption": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenString": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenRegister": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenBrace": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenMultiplication": "value",
    "crates/nvim/src/viml/parser/expressions/mod.rs::LexExprTokenComparison": "value",
    "crates/nvim/src/vterm/abi/foreign.rs": "ffi: the C callers' parser/state callbacks",
    "crates/nvim/src/vterm/host.rs::Fragment": "value",
    "crates/nvim/src/vterm/keyboard.rs::Keycode": "value",
    "crates/nvim/src/vterm/keyboard.rs::KeyModes": "value",
    "crates/nvim/src/window/arith.rs::MinSize": "value",
    "crates/nvim/src/winlayer.rs::Win": "handle: the window's {pointer, id}",
    "crates/nvim/src/winlayer.rs::Buf": "handle: the buffer's {pointer, id}",
    "crates/nvim/src/winlayer.rs::FrameRef": "handle: a frame's {pointer, id}",
    "crates/nvim/src/winlayer.rs::TabPage": "handle: the tab page's {pointer, id}",
    "crates/nvim/src/xdiff/xdiffi/mod.rs": "ffi: vendored libxdiff's values",
    "crates/nvim/src/xdiff/xtypes.rs": "ffi: vendored libxdiff's values",
}
# REPR_C_FLOOR classes:
#   abi     laid out by this tree's own C side: ffigen renders it into
#           `unit-cdefs.h` *and* a spec or C fixture reads or builds it;
#   libuv   embeds a libuv handle whose address libuv keeps;
#   codec   a swap-file block's on-disk layout;
#   ffi     a foreign ABI file (`FOREIGN_ABI_TYPES`): the library's layout.
REPR_C_FLOOR = {
    "crates/nvim/src/types/typval.rs::TypVal": "abi: the value model; ffigen renders it and the unit specs read it",
    "crates/nvim/src/types/typval.rs::Callback": "abi: the value model (a `TypVal` payload)",
    "crates/nvim/src/types/typval.rs::ScriptCtx": "abi: the value model (a `Callback`'s and a function's context)",
    "crates/nvim/src/types/pos.rs::Pos": "abi: ffigen renders it into every signature that takes a position",
    "crates/nvim/src/types/mbyte.rs::CharInfo": "abi: the C fixtures build it by value (`shim.h`)",
    "crates/nvim/src/types/mbyte.rs::StrCharInfo": "abi: the C fixtures build it by value (`shim.h`)",
    "crates/nvim/src/types/event.rs::stream": "libuv: holds a handle libuv keeps the address of",
    "crates/nvim/src/types/event.rs::stream_uv": "libuv: the stream's handle union",
    "crates/nvim/src/types/event.rs::rstream": "libuv: a stream libuv reads into",
    "crates/nvim/src/types/event.rs::proc": "libuv: a process libuv calls back by address",
    "crates/nvim/src/types/event.rs::signal_watcher": "libuv: holds a `uv_signal_t`",
    "crates/nvim/src/types/event.rs::socket_watcher_uv": "libuv: the listener's handle union",
    "crates/nvim/src/types/event.rs::socket_watcher_uv_pipe": "libuv: holds a `uv_pipe_t`",
    "crates/nvim/src/types/event.rs::socket_watcher_uv_tcp": "libuv: holds a `uv_tcp_t`",
    "crates/nvim/src/memline/blocks.rs::ZeroBlockLayout": "codec: the swap file's block 0 on disk",
    "crates/nvim/src/memline/blocks.rs::PointerBlockLayout": "codec: the swap file's pointer block on disk",
    "crates/nvim/src/memline/blocks.rs::PointerEntry": "codec: a pointer block's entry on disk",
    "crates/nvim/src/memline/blocks.rs::DataBlockLayout": "codec: the swap file's data block on disk",
    "crates/nvim/src/types/uv.rs": "ffi: libuv's own structs",
    "crates/nvim/src/types/vterm.rs": "ffi: libvterm's public types",
    "crates/nvim/src/types/vterm_internal.rs": "ffi: libvterm's internal state",
    "crates/nvim/src/types/lua.rs": "ffi: LuaJIT's `lua_State`",
    "crates/nvim/src/types/libc.rs": "ffi: the C library's and pthread's aggregates",
    "crates/nvim/src/types/libuv_proc.rs": "ffi: a `uv_process_t` handed to `uv_spawn`",
    "crates/nvim/src/types/pty_proc_unix.rs": "ffi: a `winsize` handed to the tty ioctls",
}
KEEPER_CLASSES = {
    "COPY_KEEPERS": ("ffi", "handle", "value"),
    "REPR_C_FLOOR": ("abi", "libuv", "codec", "ffi"),
}


# A file's claim that some of its names are someone else's -- a linked
# library's ABI, a generator's contract, or c2rust's spelling of an anonymous
# member. Counted per file and not per finding: the unit of work is emptying a
# file of them, and one file that needs the allow is one file however many
# names earned it. Each lift adds an entry here.
FILE_ALLOWS = {
    "non_camel_case_types": "#![allow(non_camel_case_types)]",
    "non_snake_case": "#![allow(non_snake_case)]",
    "non_upper_case_globals": "#![allow(non_upper_case_globals)]",
}
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
DERIVED = ("unsafe_stmts", "missing_safety_doc")

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
    r"\bimpl(?:<[^>]*>)?\s+(?:::)?(?:[A-Za-z0-9_]+::)*DerefMut\s+for\s+([A-Za-z_][A-Za-z0-9_]*)"
)
# `type Op = Live<OpArg>;` — a family's name for a shared generic wrapper.
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
# The block's value must be a *dereference* -- `*p` or `(*p)...` -- because
# that is what makes it a copy. `unsafe { &mut *p }.f` and
# `unsafe { Live::new(p) }.f` both start with something else and both write
# where the author meant, so neither may match.
DEREF_VALUE = r"unsafe\s*\{\s*\(?\s*\*[^{}]*?\}"
# A field chain may sit between the block and what is done to the copy.
FIELD_CHAIN = r"(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)*"
DEREF_METHOD = re.compile(
    DEREF_VALUE + FIELD_CHAIN + r"\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\("
)

# `unsafe { *p }.a = 42` -- the first face of the family (p23-7 §3), and the
# one that stayed uncovered longest: `check_deref_temporary_mutations` asks
# only about *method* calls, and `check_place_writes` only about an accessor
# `f().field = v`. A plain field assignment through the block's copy is
# neither. Batch 4 shipped seven, six of them in `match/mod.rs`'s
# `matchaddpos()` -- every position it stored stayed zeroed -- and one in
# `syntax/stack.rs`, which left the last state-cache entry's `sst_next`
# dangling.
DEREF_FIELD_WRITE = re.compile(
    DEREF_VALUE
    + r"\s*\.\s*[A-Za-z_][A-Za-z0-9_]*"
    + FIELD_CHAIN
    + r"\s*(?:[-+*/%|&^]|<<|>>)?=(?!=)"
)
# Names whose `&mut self` form is the only one that matters, listed because
# the exclusivity rule above would lose them: `Refcount::release` shares its
# name with two *consuming* `release(self)` methods, and `flags!`'s
# `clear(&mut self, flags)` shares its name with two `clear(&self)` methods on
# unrelated types. Losing either costs a mutation that silently does nothing
# -- `unsafe { (*cur).w_valid }.clear(VIRTCOL)` left the cached virtual
# column marked valid, so `getcurpos()` published a stale 'curswant' and
# never put the real one back.
# `set`, `insert`, `push` and `remove` are here for the same reason and were
# added after `wipe_ft_buf` shipped `unsafe { (*buf).b_flags }.clear(DUMMY)`
# and the fleet went looking for its siblings: `GlobalCell::set` takes `&self`,
# so the exclusivity rule drops the *name* `set` and every `flags!`
# `set(&mut self, ..)` on a temporary slipped through with it. The receiver's
# type is not visible here, so the answer has to be by name; all six are zero
# at `448061783a`, and a legitimate `&self`-with-interior-mutability receiver
# reached through a raw dereference does not occur in this tree.
MUTATING_BY_NAME = frozenset(
    {
        "retain",
        "release",
        "release_many",
        "clear",
        "set",
        "insert",
        "push",
        "remove",
        "toggle",
        "reset",
    }
)

# `*pp = *pp.add(3)` where the author meant `*pp = (*pp).add(3)`.
#
# A method call binds tighter than `*`, so on a `*mut *mut c_char` the first
# form advances the *outer* pointer by one whole pointer and stores whatever
# the neighbouring stack slot holds. It type-checks, because both sides are
# still `*mut c_char`, and nothing warns. Batch 3 shipped two: `get_lambda_tv`
# stored a stack neighbour into the expression cursor -- every lambda body
# `{_, v -> ...}` in the runtime files then parsed from a garbage address, so
# `nvim -l` and any `->method()` after a lambda segfaulted.
#
# Restricted to the self-assigning shape on purpose. `*files.offset(i)` on an
# array of pointers is ordinary and correct; taking the *i*th element of an
# array and storing it back into the array's own first slot is not something
# anyone means.
SELF_PROJECTION = re.compile(
    r"\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\*\s*\1\s*\.\s*"
    r"(?:add|offset|sub|wrapping_add|wrapping_offset|wrapping_sub)\s*\("
)

BORROWED_DEREF = re.compile(
    r"&\s*(?:mut|raw\s+mut|raw\s+const)\s+unsafe\s*\{\s*\(*\s*\*"
)

# `unsafe { (*p).arr }.as_ptr()` -- the sixth face of "an `unsafe` block is a
# value expression", and the one the compiler only sometimes catches.
#
# The block copies the array (or the struct holding it) out of the pointee,
# and a method that borrows `self` to hand back an address then answers the
# address of that *temporary*, dangling by the end of the statement. rustc's
# `dangling_pointers_from_temporaries` fires on the direct spelling and two of
# these still shipped during phase 23's batch 4 -- `spell/dump.rs` took the
# address of a copy of `sl_regions` and compared strings through it, and
# `cmdexpand/generate.rs` returned a pointer into a copy of a 1 KiB
# `xp_buf` that `:scriptnames` completion then wrote to.
#
# A field chain may sit between the block and the call, which is exactly the
# form that slipped past `DEREF_METHOD`: the method there must hang directly
# off the brace, and `unsafe { (*xp) }.xp_buf.as_mut_ptr()` does not.
TEMPORARY_ADDRESS = re.compile(
    r"unsafe\s*\{\s*\(?\s*\*[^{}]*?\}"
    r"(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)*"
    r"\s*\.\s*(as_ptr|as_mut_ptr|as_slice|as_mut_slice|as_bytes|as_bytes_mut)\s*\("
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


# One `fn` definition, as `fn_signatures` reports it. `start` is the offset of
# the `fn` keyword and `end` the offset just past the return type, which is
# where a body's `{` (or a `where` clause, or a `;`) begins.
Signature = collections.namedtuple("Signature", "name text params returns start end")


def fn_signatures(masked):
    """A `Signature` per `fn` *definition*.

    Only definitions: a `fn` type (a function pointer) and a trait bound have
    no parameter list after the name and are skipped. `text` runs from the
    `fn` keyword through the return type, so a parameter rustfmt wrapped onto
    its own line is inside it and a local of the same type is not; `params` is
    the parenthesised parameter list alone and `returns` the declared return
    type, `"()"` when none is written. The two are separate because a
    `-> *mut c_char` is a different debt from a `*mut c_char` parameter and
    must not be booked in both.
    """
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
        opened = i
        i = balanced(masked, i, "(", ")")
        params = masked[opened:i]
        stop = i
        while i < len(masked) and masked[i].isspace():
            i += 1
        if masked[i : i + 2] != "->":
            yield Signature(
                match.group(1),
                masked[match.start() : stop],
                params,
                "()",
                match.start(),
                stop,
            )
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
        yield Signature(
            match.group(1),
            masked[match.start() : end],
            params,
            masked[i:end].strip(),
            match.start(),
            end,
        )


def fn_returns(masked, out):
    """name -> the set of return types every `fn` of that name declares."""
    for sig in fn_signatures(masked):
        out.setdefault(sig.name, set()).add(sig.returns)


def is_place(ty, deref_mut):
    """Whether `.field` on a value of this type resolves to somewhere real."""
    return (
        ty.startswith("&")
        or ty.startswith("*")
        # `Self` in an inherent impl of a newtype; the impl block's own type is
        # not in reach of a name-keyed scan, and every one of these in the tree
        # is a handle.
        or ty == "Self"
        # A generic guard (`Edit<'_, List>`) is its type constructor's name.
        or ty.split("<", 1)[0].strip() in deref_mut
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
    """Method names that can only ever be `&mut self`, plus `MUTATING_BY_NAME`."""
    mutating, other = set(), set()
    for masked in tree.values():
        mutating.update(MUT_SELF_FN.findall(masked))
        other.update(NON_MUT_SELF_FN.findall(masked))
    return (mutating - other) | MUTATING_BY_NAME


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


def self_projections(tree):
    """`*pp = *pp.add(3)` -- the parentheses `(*pp).add(3)` needed."""
    found = []
    for file, masked in sorted(tree.items()):
        for match in SELF_PROJECTION.finditer(masked):
            line = masked.count("\n", 0, match.start()) + 1
            found.append(f"  {file}:{line}: {match.group(0).strip()}...)")
    return found


def check_self_projections(tree):
    if found := self_projections(tree):
        sys.exit(
            "ratchet: `*p = *p.add(n)` advances the OUTER pointer -- a method "
            "call binds tighter than `*`, so this stores the neighbouring "
            "slot's contents instead of stepping the pointee:\n"
            + "\n".join(found)
            + "\nWrite `*p = (*p).add(n)`."
        )


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


def temporary_addresses(tree):
    """`unsafe { (*p).arr }.as_ptr()` -- the address of the block's copy."""
    found = []
    for file, masked in sorted(tree.items()):
        for match in TEMPORARY_ADDRESS.finditer(masked):
            line = masked.count("\n", 0, match.start()) + 1
            found.append(f"  {file}:{line}: .{match.group(1)}()")
    return found


def check_temporary_addresses(tree):
    if found := temporary_addresses(tree):
        sys.exit(
            "ratchet: a method that borrows `self` to hand back an address, "
            "called on the value an `unsafe` block produced, answers the "
            "address of a *temporary* -- the copy the dereference made, which "
            "is gone by the end of the statement:\n"
            + "\n".join(found)
            + "\nThe region has to cover the call: "
            "`unsafe { (*p).arr.as_ptr() }`."
        )


def deref_field_writes(tree):
    """`unsafe { *p }.a = 42` -- an assignment into the block's temporary."""
    found = []
    for file, masked in sorted(tree.items()):
        for match in DEREF_FIELD_WRITE.finditer(masked):
            line = masked.count("\n", 0, match.start()) + 1
            found.append(f"  {file}:{line}: {match.group(0).strip()}")
    return found


def check_deref_field_writes(tree):
    if found := deref_field_writes(tree):
        sys.exit(
            "ratchet: a field written through the value an `unsafe` block "
            "produced lands in a *temporary* -- the block is a value "
            "expression, so the dereference made a copy and the write is "
            "discarded:\n"
            + "\n".join(found)
            + "\nThe region has to cover the assignment: "
            "`unsafe { (*p).a = 42 }`, or wrap the pointer in a "
            "`winlayer::Live<T>` once."
        )


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


def struct_bodies(masked):
    """The text between the braces of every braced `struct`/`union` item."""
    for head in STRUCT_HEAD.finditer(masked):
        open_at = head.end() - 1
        yield masked[open_at + 1 : matching_brace(masked, open_at)]


# A `{` that opens a *struct literal* rather than a block: an UpperCamel head
# (or `Self`) immediately before it. `Pos { lnum, col }` inside a region is one
# expression, not a block with a tail, and charging it as a block would price
# every initialiser twice. A SCREAMING_CASE constant before the brace
# (`if flag == LOCAL {`) is deliberately not matched -- it is an `if`, and its
# block's tail is real -- which is why the head must carry a lower-case letter.
STRUCT_LITERAL = re.compile(r"(?:\bSelf|\b[A-Z][A-Za-z0-9_]*[a-z][A-Za-z0-9_]*)\s*$")


def statements(body):
    """How many statements a region's body holds, counted at every depth.

    A region is charged by what it *does*, not by how it is laid out: each
    `;`-terminated statement, each block's tail expression and each `match`
    arm is one unit, wherever it sits. A multi-line call is one, a closure's
    statements are the caller's (they are inside the region), and an empty
    region is one -- a region that does nothing is still a promise.

    Counting only the top level would be wrong in the one direction that
    matters: `unsafe { loop { ..fifty lines.. } }` would price at 1, and the
    c2rust blanket a phase of work removed would come back free.

    `Ident {` with an UpperCamel head is a struct literal and not a block, so
    its fields are not a tail expression to charge for.
    """
    charge = 0
    # One frame per open brace: [is a block, holds an expression not yet
    # charged, is a `match`'s arm list]. The outermost frame is the region
    # body itself.
    stack = [[True, False, False]]
    i, n = 0, len(body)
    while i < n:
        char = body[i]
        if char == "{":
            block = not STRUCT_LITERAL.search(body, max(0, i - 64), i)
            stack.append([block, False, False])
        elif char == "}":
            frame = stack.pop() if len(stack) > 1 else stack[0]
            if frame[0] and frame[1] and not frame[2]:
                charge += 1  # the block's tail expression
            stack[-1][1] = True  # ... and the block is one itself
        elif char == ";":
            charge += 1
            stack[-1][1] = False
        elif body.startswith("=>", i):
            charge += 1  # a match arm
            stack[-1][2] = True  # ... and the arms are not a tail expression
            i += 1
        elif not char.isspace():
            stack[-1][1] = True
        i += 1
    if stack[0][1] and not stack[0][2]:
        charge += 1  # the region's own tail expression
    return max(charge, 1)


def unsafe_stmts(masked, deny):
    """Statements of code the compiler is not checking. See the module docs."""
    charge = 0
    # Everything up to here is inside a region already charged: a nested
    # `unsafe {}` costs nothing, and neither do the blocks of an `unsafe fn`
    # body charged whole in a file without the deny.
    charged_until = -1
    for match in UNSAFE_WORD.finditer(masked):
        if match.start() < charged_until:
            continue
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
                body_at = after  # `unsafe extern "C" { ... }`: its declarations
            elif follows and follows.group(0) == "fn":
                word = follows  # `unsafe extern "C" fn ...`
                keyword = "fn"
            else:
                continue
        elif keyword in ("impl", "trait"):
            charge += 1  # an unchecked promise with nowhere else to book it
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
                charge += 1  # a bodyless declaration
                continue
            body_at = brace

        close = matching_brace(masked, body_at)
        charge += statements(masked[body_at + 1 : close])
        charged_until = close
    return charge


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


def unsafe_fn_items(masked):
    """The offset of the `unsafe` keyword of every `unsafe fn` *item*.

    Walks the same `unsafe` keyword occurrences `unsafe_stmts` does and keeps
    the ones that introduce a *function* — a definition or a bodyless
    declaration, never a function-pointer type (`unsafe fn(..)`, whose
    obligation is paid where it is called) and never a declaration inside an
    `unsafe extern` block, whose obligation is the C library's.
    """
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
            continue  # an `unsafe fn(..)` *type*; it names nothing
        yield match.start()


# The head of a call expression, matched against a region's whole body: a path
# (`f`, `m::f`, `T::f`, `T::<X>::f`) or a receiver chain ending in a method
# (`x.f`, `x.y.f`), up to and including the `(` that opens its arguments.
WRAPPER_CALL = re.compile(
    r"^([A-Za-z_][A-Za-z0-9_]*"
    r"(?:(?:::|\.)[A-Za-z_][A-Za-z0-9_]*|::<[^()]*>)*)\s*\($"
)
# The same for a variadic's checked macro (`vim_snprintf!(..)`), which is the
# callee's call with its arguments bound: an `unsafe {}` around one alone is a
# wrapper of that callee, and keying it by the macro would let the sweep onto
# the macros book two hundred wrappers as retired.
VARIADIC_MACRO_CALL = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*!\s*\($")
# A path's generic arguments, stripped before the last segment is read.
TURBOFISH = re.compile(r"::<.*$")
PATH_SEPARATOR = re.compile(r"::(?!<)")
# An `impl` block's head, through the type it is an impl *of*: the name a
# method's row in the wrapper table is keyed by. A trait impl names the trait
# first and the type after `for`, so the capture is the last of the two.
IMPL_HEAD = re.compile(
    r"\bimpl\s*(?:<[^>]*>)?\s*"
    r"(?:[A-Za-z_][A-Za-z0-9_:]*(?:<[^>]*>)?\s+for\s+)?([A-Za-z_][A-Za-z0-9_]*)"
)
# A raw pointer in a signature, in the two spellings that are one: the pointer
# itself, and the `CPtr` bound behind which `message_fmt`'s `c_str` hides a
# `*const c_char`. `NonNull<T>` is deliberately *not* here -- it is a raw
# pointer whose contract is a row on the allowlist, not a parameter the needle
# can see for itself.
RAW_IN_SIGNATURE = re.compile(r"\*\s*(?:mut|const)\b|\bimpl\s+CPtr\b|:\s*CPtr\b")
# A `type` alias whose right-hand side *is* a raw pointer. The needle reads a
# signature as text, so a name like `MetaFilter` (`= *const uint32_t`) hides a
# pointer from it as completely as a `CPtr` bound does, and four marktree entry
# points spent a slice on the allowlist for it. The names are collected
# tree-wide and then count as raw wherever a signature spells one. One level is
# enough: no alias in the tree is written in terms of another, and a chain
# would want a fixpoint rather than a second pass.
RAW_ALIAS = re.compile(
    r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*=\s*\*\s*(?:mut|const)\b"
)
FN_WORD = re.compile(r"\bfn\b")


def raw_aliases(tree):
    """Every `type X = *mut/*const ...` name in the tree."""
    return frozenset(
        match.group(1)
        for masked in tree.values()
        for match in RAW_ALIAS.finditer(masked)
    )


def signature_has_raw(text, aliases):
    """Whether a signature shows a raw pointer, spelled out or behind a name."""
    if RAW_IN_SIGNATURE.search(text):
        return True
    return any(word in aliases for word in IDENT_AT.findall(text))


# One `unsafe fn` item: its name, the type whose `impl` block holds it (None
# for a free function), the file, and the two properties the needle asks about.
UnsafeFn = collections.namedtuple("UnsafeFn", "name owner file has_raw extern")


def callee_key(chain):
    """The wrapper table's key for a call's path.

    The last segment, qualified by the one before it when that is a type
    (`Live::new`, not `new`), and `.name` for a method call -- a receiver's
    type is out of reach of a name-keyed scan, and the method name is what a
    reader recognises. A bare generic name (`new`, `at`) is *not* resolved
    across the tree: the table says how many wrappers spell that call, and its
    `defs` column says whether one definition answers to it.
    """
    if "." in chain:
        return "." + chain.rsplit(".", 1)[1]
    segments = [TURBOFISH.sub("", s) for s in PATH_SEPARATOR.split(chain) if s]
    if len(segments) >= 2 and segments[-2][:1].isupper():
        return segments[-2] + "::" + segments[-1]
    return segments[-1]


def wrapper_callee(body):
    """The callee of a region that is exactly one call, else None.

    A *wrapper* is an `unsafe {}` region whose whole body, bar whitespace and
    one trailing `;`, is a single call expression -- the region exists because
    the callee is `unsafe fn`, not because the caller does anything unsafe.
    What sits outside the braces is irrelevant: `let x = unsafe { f(..) };`,
    the tail form, `return unsafe { f(..) };` and `g(unsafe { f(..) })` are all
    wrappers, and so is one rustfmt wrapped over four lines -- reflowing a
    retype must not move the number.

    `unsafe { *p }`, `unsafe { &mut *p }` and a compound body are not
    wrappers: their obligation is the caller's own.
    """
    text = body.strip()
    if text.endswith(";"):
        text = text[:-1].rstrip()
    if not text.endswith(")"):
        return None
    depth = 0
    for i in range(len(text) - 1, -1, -1):
        if text[i] == ")":
            depth += 1
        elif text[i] == "(":
            depth -= 1
            if not depth:
                head = WRAPPER_CALL.match(text[: i + 1])
                if head:
                    return callee_key(head.group(1))
                head = VARIADIC_MACRO_CALL.match(text[: i + 1])
                if head and head.group(1) in ARGUMENT_MACROS:
                    return head.group(1)
                return None
    return None


def wrapper_scan(tree):
    """(wrappers per file, wrappers per callee) over the whole tree."""
    by_file = collections.Counter()
    by_callee = collections.Counter()
    for file, masked in tree.items():
        for match in UNSAFE_WORD.finditer(masked):
            at = WHITESPACE.match(masked, match.end()).end()
            if at >= len(masked) or masked[at] != "{":
                continue
            close = matching_brace(masked, at)
            if callee := wrapper_callee(masked[at + 1 : close]):
                by_file[file] += 1
                by_callee[callee] += 1
    return by_file, by_callee


def impl_spans(masked):
    """(open brace, close brace, type name) for every `impl` block."""
    for match in IMPL_HEAD.finditer(masked):
        brace = masked.find("{", match.end())
        if brace < 0 or ";" in masked[match.end() : brace]:
            continue
        yield brace, matching_brace(masked, brace), match.group(1)


def unsafe_fn_defs(masked, file, aliases=frozenset()):
    """An `UnsafeFn` per `unsafe fn` item in one file.

    The signature is `fn_signatures`' -- `fn` through the end of the return
    type, so a parameter rustfmt wrapped onto its own line is inside it and a
    `-> *mut c_void` is too: a function that hands back raw memory is a
    contract whether or not it takes one. An item inside a `macro_rules!` body
    has no signature the join can see and is skipped.
    """
    spans = list(impl_spans(masked))
    signatures = {sig.start: sig for sig in fn_signatures(masked)}
    for at in unsafe_fn_items(masked):
        keyword = FN_WORD.search(masked, at)
        sig = signatures.get(keyword.start()) if keyword else None
        if sig is None:
            continue
        owner = None
        for open_at, close_at, name in spans:
            if open_at < at < close_at:
                owner = name  # the innermost `impl` block wins
        yield UnsafeFn(
            sig.name,
            owner,
            file,
            signature_has_raw(sig.text, aliases),
            "extern" in masked[at : sig.start],
        )


def unsafe_fn_key(item):
    """`Owner::name` for a method, `name` for a free function."""
    return f"{item.owner}::{item.name}" if item.owner else item.name


def unsafe_fn_index(tree):
    """key -> the `UnsafeFn`s it names, plus a bare-name entry per method.

    The bare name is what a call written `x.f(..)` or `f(..)` inside an `impl`
    can be keyed by, so the table can still say which definition a wrapper
    reaches when the spelling drops the type.
    """
    index = collections.defaultdict(list)
    aliases = raw_aliases(tree)
    for file, masked in tree.items():
        for item in unsafe_fn_defs(masked, file, aliases):
            index[unsafe_fn_key(item)].append(item)
            if item.owner:
                index[item.name].append(item)
    return index


# One allowlist row's reason class. Every row says which of the four things
# the type system cannot hold keeps the keyword on a function whose signature
# shows no raw pointer; `field:` is the class the string and state phases
# retire, and a row leaves the list when its field does.
ALLOW_CLASSES = ("alloc:", "cap:", "ffi:", "field:")
# A row of the allowlist's markdown table. Three cells: the item, the file it
# lives in, and the reason. prettier pads the columns, so everything is
# stripped before it is read.
ALLOW_ROW = re.compile(r"^\|([^|]*)\|([^|]*)\|(.*)\|\s*$")


# A parsed allowlist row: the files it says the item lives in (repo-relative,
# as the tree keys files; a private helper defined twice lists both, comma-
# separated) and the reason.
AllowRow = collections.namedtuple("AllowRow", "files reason")


def unsafe_fn_allowlist():
    """key -> `AllowRow`, read from docs/unsafe-fn-allowlist.md.

    The keys are `unsafe_fn_key`'s: `Owner::name` for a method, `name` for a
    free function. A bare name does *not* excuse a method of that name -- the
    row would then quietly cover every type that grows one -- so a method's
    row has to spell its owner, and `check_unsafe_fn_allowlist` says so when
    a row matches nothing. The file column is part of the claim: a row
    excuses its item only in the file it names, so a function that moves
    takes its row's file with it or the run fails. A name defined in more
    than one file lists each one, comma-separated.
    """
    allow = {}
    if not UNSAFE_FN_ALLOW_DOC.exists():
        sys.exit(
            f"ratchet: {UNSAFE_FN_ALLOW_DOC.relative_to(ROOT)} is missing. It "
            "is the floor `unsafe_fns_without_raw_params` is measured "
            "against; restore it, empty if need be."
        )
    for line in UNSAFE_FN_ALLOW_DOC.read_text().splitlines():
        row = ALLOW_ROW.match(line.strip())
        if not row:
            continue
        name = row.group(1).strip().strip("`").strip()
        files = frozenset(
            part.strip().strip("`").strip() for part in row.group(2).split(",")
        )
        reason = row.group(3).strip()
        if not name or set(name) <= set("-: ") or name == "item":
            continue  # the header and its separator
        if not reason.strip("`").startswith(ALLOW_CLASSES):
            sys.exit(
                f"ratchet: {UNSAFE_FN_ALLOW_DOC.relative_to(ROOT)}: the row "
                f"for `{name}` opens with no reason class. Every row starts "
                "with one of " + ", ".join(ALLOW_CLASSES) + "."
            )
        allow[name] = AllowRow(files, reason)
    return allow


def allow_excuses(allow, item):
    """Whether `item` has an allowlist row naming its own file."""
    row = allow.get(unsafe_fn_key(item))
    return row is not None and item.file in row.files


def unsafe_fns_without_raw(masked, file, allow, aliases=frozenset()):
    """`unsafe fn`s the needle counts in one file. See the doc block."""
    return sum(
        not (item.has_raw or item.extern or allow_excuses(allow, item))
        for item in unsafe_fn_defs(masked, file, aliases)
    )


def check_unsafe_fn_allowlist(index, allow):
    """Every allowlist row still excuses an `unsafe fn` the needle would count.

    A row whose function is gone, or that now carries a raw pointer (or an
    `extern "C"` ABI), excuses nothing: it would sit there propping the floor
    up, and would quietly excuse the name again if it came back. The list
    prunes itself the way PERIMETER and CELL_PTR_KEEPERS do. A row whose file
    column names a file that does not define the item is stale the same way:
    the column is how a reader finds the contract, so it has to be true.
    """
    stale = []
    for name in sorted(allow):
        named = [item for item in index.get(name, ()) if unsafe_fn_key(item) == name]
        matched = [item for item in named if item.file in allow[name].files]
        if not named:
            stale.append(f"{name}: no `unsafe fn` of that name")
        elif absent := allow[name].files - {item.file for item in named}:
            where = ", ".join(sorted({item.file for item in named}))
            stale.append(
                f"{name}: not in {', '.join(sorted(absent))} (defined in {where})"
            )
        elif all(item.has_raw or item.extern for item in matched):
            stale.append(f"{name}: carries a raw pointer or a C ABI now")
    if stale:
        sys.exit(
            "ratchet: these rows of "
            f"{UNSAFE_FN_ALLOW_DOC.relative_to(ROOT)} excuse nothing:\n  "
            + "\n  ".join(stale)
            + "\nDrop each row (or correct its file column) and run `just "
            "refresh` to lock the progress in."
        )


def wrapper_table(by_callee, index):
    """The wrapper-callee table: one tab-separated row per callee, biggest first.

    The work order for the callee phases. `count` is how many
    `unsafe { callee(..) }` regions the tree holds, `defs` how many
    `unsafe fn` definitions answer to the name, `def` the file when exactly
    one holds them, and `no_raw` the yield column: 1 when every one of those
    definitions is an `unsafe fn` the `unsafe_fns_without_raw_params` needle
    would count, so dropping its keyword deletes `count` wrappers (bar the
    ones docs/unsafe-fn-allowlist.md keeps, which is a list short enough to
    read).

    The join to a definition is by *name*: a row keyed `.name` or a bare
    generic one (`new`, `at`) is matched against every `unsafe fn` of that
    name, so `defs > 1` means the name is ambiguous and the row wants an eye
    before it is worked. `no_raw` demands that *every* candidate qualifies,
    which is the safe direction: an ambiguous row under-claims rather than
    promising a yield that is not there.

    Sorted by count then name, so a row moves only when its count does.
    """
    rows = [WRAPPER_TABLE_HEADER]
    for callee, count in sorted(by_callee.items(), key=lambda row: (-row[1], row[0])):
        matched = (
            index.get(callee) or index.get(callee.split("::")[-1].lstrip(".")) or []
        )
        no_raw = bool(matched) and all(not (i.has_raw or i.extern) for i in matched)
        files = sorted({item.file for item in matched})
        rows.append(
            f"{count}\t{callee}\t{int(no_raw)}\t{len(matched)}\t"
            f"{files[0] if len(files) == 1 else ''}\n"
        )
    return "".join(rows)


def sync_wrapper_table(content, check):
    """Write metrics/wrapper-callees.tsv, or fail when it is stale."""
    committed = WRAPPER_TABLE.read_text() if WRAPPER_TABLE.exists() else None
    if check:
        if committed != content:
            sys.exit(
                f"ratchet: {WRAPPER_TABLE.relative_to(ROOT)} is stale; run "
                "`just refresh` and commit the result."
            )
        return
    if committed != content:
        WRAPPER_TABLE.write_text(content)


def safety_on_safe_fns(text, masked, declarations):
    """Safe `fn` definitions whose doc comment has a `# Safety` section."""
    lines = text.splitlines()
    found = 0
    for sig in declarations:
        line_start = masked.rfind("\n", 0, sig.start) + 1
        if UNSAFE_WORD.search(masked, line_start, sig.start):
            continue
        found += has_safety_doc(lines, masked.count("\n", 0, sig.start))
    return found


def missing_safety_doc(text, masked):
    """`unsafe fn`s whose doc comment has no `# Safety` section."""
    lines = text.splitlines()
    starts = [0, *(m.end() for m in re.finditer("\n", masked))]
    return sum(
        not has_safety_doc(lines, bisect_right(starts, at) - 1)
        for at in unsafe_fn_items(masked)
    )


def measure():
    """(repo-relative file -> {metric: count} with zeros included,
    number of files not carrying the cast deny,
    lint name -> number of files carrying its blanket allow,
    repo-relative file -> its masked source, for the whole-tree checks,
    repo-relative file -> its *raw* source, for the three needles that cannot
    run against the masked copy -- a char literal is blanked, a generated
    file's marker is a comment, and a duplicated constant's value may be a
    string)."""
    stats = {}
    tree = {}
    sources = {}
    without_casts = 0
    allowing = dict.fromkeys(FILE_ALLOWS, 0)
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
        counts["unsafe_stmts"] = unsafe_stmts(masked, DENY_UNSAFE_OP in masked)
        counts["missing_safety_doc"] = missing_safety_doc(text, masked)
        counts["lines"] = len(text.splitlines()) - len(test_module_lines(masked))
        stats[str(path.relative_to(ROOT))] = counts
        tree[str(path.relative_to(ROOT))] = masked
        sources[str(path.relative_to(ROOT))] = text
        without_casts += DENY_CASTS.search(masked) is None
        for lint, needle in FILE_ALLOWS.items():
            allowing[lint] += needle in masked
    return stats, without_casts, allowing, tree, sources


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


def in_home(file, home):
    """Whether any entry of a module list claims this repo-relative path."""
    return any(in_perimeter_entry(file, entry) for entry in home)


def in_perimeter(file):
    """Whether a repo-relative path is on the unsafe perimeter."""
    return in_home(file, PERIMETER)


def perimeter_lines(stats):
    """(unchecked lines inside the perimeter, unchecked lines outside it)."""
    inside = sum(c["unsafe_stmts"] for f, c in stats.items() if in_perimeter(f))
    return inside, sum(c["unsafe_stmts"] for c in stats.values()) - inside


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
            counts["unsafe_stmts"] and in_perimeter_entry(file, entry)
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


# The measured sentence in docs/perimeter.md, from "Today:" to the end of its
# second sentence. Everything after it (the "It was N when this file was
# written" line) is history and is left alone. `re.DOTALL` because the
# paragraph is wrapped across source lines.
PERIMETER_TODAY = re.compile(r"Today: \*\*[\d,]+\*\*.*?measured\)\.", re.DOTALL)


def perimeter_today(stats):
    """The "Today" sentence docs/perimeter.md should be carrying.

    Five measured numbers: unchecked lines inside the perimeter and the files
    holding them, the same outside it, and how many files were measured
    altogether. Hand-maintained, all five went stale -- the line claimed
    48,358 outside against a real 47,002 -- which is the argument for
    generating it: a number in prose that nothing checks is a number nobody
    can cite.

    The line breaks are chosen here rather than by the formatter: prettier's
    default `proseWrap` is "preserve", so whatever this writes survives
    `just fmt` unchanged and `--check` can compare the two literally.
    """
    inside, outside = perimeter_lines(stats)
    counted = [c for c in stats.values() if c["unsafe_stmts"]]
    inside_files = sum(
        1
        for file, counts in stats.items()
        if counts["unsafe_stmts"] and in_perimeter(file)
    )
    return (
        f"Today: **{inside:,}** unchecked statements inside the perimeter "
        f"({inside_files} files),\n"
        f"**{outside:,}** outside it ({len(counted) - inside_files} files, "
        f"of {len(stats):,} measured)."
    )


def sync_perimeter_doc(stats, check):
    """Rewrite docs/perimeter.md's "Today" sentence, or fail when it is stale.

    The doc is the perimeter's contract and the ratchet is the only thing
    that can count; the two disagreeing is how the old line drifted by 1,356
    lines without anyone noticing.
    """
    text = PERIMETER_DOC.read_text()
    want = perimeter_today(stats)
    if not PERIMETER_TODAY.search(text):
        sys.exit(
            f'ratchet: {PERIMETER_DOC.relative_to(ROOT)} has no "Today:" '
            "sentence to write. Restore it, or update PERIMETER_TODAY."
        )
    fresh = PERIMETER_TODAY.sub(lambda _: want, text, count=1)
    if check:
        if fresh != text:
            sys.exit(
                f'ratchet: {PERIMETER_DOC.relative_to(ROOT)}\'s "Today" line '
                f"is stale. It should read:\n\n{want}\n\n"
                "Run `just refresh` and commit the result."
            )
        return
    if fresh != text:
        PERIMETER_DOC.write_text(fresh)


# The comment block naming a perimeter row: "Unsafe perimeter: the `os/` row
# in docs/perimeter.md." Every backticked path in it is a row's spelling,
# brace groups included (`lua/executor/{luv,state}.rs`).
PERIMETER_ROW_COMMENT = re.compile(r"\bUnsafe perimeter:")
BACKTICKED = re.compile(r"`([^`]+)`")
BRACE_GROUP = re.compile(r"\{([^{}]*)\}")
# How each placement finding reads, and the two ways out of it.
PLACEMENT_EXITS = (
    "finish it (drop the allow for `#![forbid(unsafe_code)]`), or give it a "
    "PERIMETER row with a reason (scripts/ratchet.py and docs/perimeter.md)"
)


def expand_braces(path):
    """`a/{b,c}.rs` -> [`a/b.rs`, `a/c.rs`]; nested groups expand in turn."""
    match = BRACE_GROUP.search(path)
    if match is None:
        return [path]
    return [
        expanded
        for part in match.group(1).split(",")
        for expanded in expand_braces(
            path[: match.start()] + part.strip() + path[match.end() :]
        )
    ]


def comment_above(source, needle):
    """The `//` comment lines directly above the line holding `needle`."""
    lines = source.splitlines()
    at = next(i for i, line in enumerate(lines) if line.strip() == needle)
    block = []
    for line in reversed(lines[:at]):
        stripped = line.strip()
        if not stripped.startswith("//") or stripped.startswith(("///", "//!")):
            break
        block.append(stripped[2:].strip())
    return " ".join(reversed(block))


def names_its_row(file, comment):
    """Whether a comment names the PERIMETER entry that claims `file`."""
    if not PERIMETER_ROW_COMMENT.search(comment):
        return False
    return any(
        in_perimeter_entry(file, CRATE_SRC + path)
        for spelled in BACKTICKED.findall(comment)
        for path in expand_braces(spelled.strip())
        if CRATE_SRC + path in PERIMETER
    )


def module_dir(file):
    """The directory whose files a module file's lint attributes reach, or
    None for a leaf: `x/mod.rs` and `lib.rs` reach their own directory, and
    `x.rs` reaches `x/` (whether that directory exists is the caller's
    question)."""
    head, _, name = file.rpartition("/")
    if name in ("mod.rs", "lib.rs", "main.rs"):
        return head + "/"
    return file.removesuffix(".rs") + "/"


def placement_findings(tree, sources):
    """repo-relative file -> the placement rules it breaks. See the doc block.

    Three rules, over `crates/nvim/src` only:

    1. a file carrying `#![allow(unsafe_code)]` is on PERIMETER;
    2. every other file carries `#![forbid(unsafe_code)]`, unless it is a
       module file with a descendant that carries the allow -- `allow` under
       a `forbid` is E0453, so that file cannot say either (the "neither"
       list, computed here rather than listed);
    3. an allow file carries `#![deny(unsafe_op_in_unsafe_fn)]`, and a
       perimeter one names its row in the `//` comment directly above the
       attribute ("Unsafe perimeter: the `os/` row in docs/perimeter.md.").

    A finding is one broken rule, so a file can hold more than one.
    """
    allow = FILE_ALLOWS_UNSAFE
    files = {f: m for f, m in tree.items() if f.startswith(CRATE_SRC)}
    allowing = {f for f, m in files.items() if allow in m}
    found = {}
    for file, masked in files.items():
        source = sources[file]
        broken = []
        generated = GENERATED_MARKER.search(source[:GENERATED_HEAD]) is not None
        if file in allowing:
            if not in_perimeter(file):
                broken.append("allows unsafe code off the perimeter")
            if DENY_UNSAFE_OP not in masked:
                broken.append(f"allows unsafe code without {DENY_UNSAFE_OP}")
            if (
                in_perimeter(file)
                and not generated
                and not names_its_row(file, comment_above(source, allow))
            ):
                broken.append(
                    "allows unsafe code without naming its perimeter row in "
                    "the comment above the attribute"
                )
        elif FORBID not in masked:
            under = module_dir(file)
            if not any(other.startswith(under) for other in allowing):
                broken.append(
                    f"carries neither {FORBID} nor the allow, and has no "
                    "descendant that allows"
                )
        if broken:
            found[file] = broken
    return found


def check_unsafe_placement(tree, sources):
    """Fail the run when an unsafe-code attribute is in the wrong place.

    The three rules are `placement_findings`'. A ratcheted count of them would
    only say "no worse than before"; at zero the only honest number is zero,
    so a finding stops the run, in `--check` (minimal-ci, the pre-commit hook)
    and when writing the baseline alike.
    """
    if found := placement_findings(tree, sources):
        sys.exit(
            "ratchet: unsafe code is allowed only on the perimeter "
            "(docs/perimeter.md). These files break a placement rule:\n  "
            + "\n  ".join(
                f"{file}: {rule}"
                for file, broken in sorted(found.items())
                for rule in broken
            )
            + f"\nEach: {PLACEMENT_EXITS}."
        )


def vocabulary(tree):
    """The C-vocabulary counts. See "the C vocabulary" in the doc block."""
    counts = dict.fromkeys(
        (*VOCABULARY, *VOCABULARY_OUTSIDE, *VOCABULARY_INSIDE, *VOCABULARY_PARAMS), 0
    )
    PTR_ARITH.prime(tree)
    for file, masked in tree.items():
        for name, needle in VOCABULARY.items():
            counts[name] += len(needle.findall(masked))
        for name, (needle, home) in VOCABULARY_OUTSIDE.items():
            if not in_home(file, home):
                counts[name] += len(needle.findall(masked))
        for name, (needle, home) in VOCABULARY_INSIDE.items():
            if in_home(file, home):
                counts[name] += len(needle.findall(masked))
    names = set()
    signatures = 0
    frames = 0
    mut_refs = 0
    aliases = {}
    constants = []
    abbrevs = 0
    exported = api_exported()
    for file, masked in tree.items():
        names.update(T_SUFFIX_DECL.findall(masked))
        for alias, target in INT_ALIAS_DECL.findall(masked):
            aliases.setdefault(alias, set()).add(target)
        constants.extend(type_ for _, type_ in PUB_CONST_DECL.findall(masked))
        declarations = list(fn_signatures(masked))
        for name, needle in VOCABULARY_PARAMS.items():
            counts[name] += sum(len(needle.findall(sig.params)) for sig in declarations)
        spans = [sig.text for sig in declarations]
        if not in_home(file, WINLAYER):
            signatures += sum(len(RAW_WIN_BUF.findall(sig)) for sig in spans)
            frames += sum(len(RAW_FRAME.findall(sig)) for sig in spans)
            mut_refs += sum(len(MUT_WIN_BUF_REF.findall(sig)) for sig in spans)
        if not in_home(file, ABBREV_PARAM_EXEMPT):
            frozen = exported if file.startswith(API_DIR) else ()
            spans = [sig.text for sig in declarations if sig.name not in frozen]
            abbrevs += sum(len(ABBREV_PARAM.findall(sig)) for sig in spans)
            abbrevs += sum(
                bool(BUFFER_TYPE.search(type_))
                for sig in spans
                for type_ in ABBREV_BUF_PARAM.findall(sig)
            )
            abbrevs += sum(
                not RAW_MEMORY_TYPE.search(type_)
                for sig in spans
                for type_ in ABBREV_PTR_PARAM.findall(sig)
            )
    integral = int_aliases(aliases)
    return {
        **counts,
        "const_int_alias": sum(type_ in integral for type_ in constants),
        "t_suffix_types": len(names),
        "raw_win_buf_sigs": signatures,
        "raw_frame_sigs": frames,
        "mut_win_buf_refs": mut_refs,
        "abbrev_params": abbrevs,
    }


def fn_body_lines(masked, sig):
    """The line count of a `fn` item, or 0 when the declaration has no body.

    From the end of the signature the next `{` at depth zero opens the body --
    a `where` clause may sit in between -- and its matching `}` closes it. A
    `->` inside a `where` bound is stepped over rather than read as a closing
    angle bracket, which is what keeps `where F: Fn() -> u32` from
    desynchronising the depth.
    """
    i, depth = sig.end, 0
    while i < len(masked):
        char = masked[i]
        if masked[i : i + 2] == "->":
            i += 2
            continue
        if char == ";" and not depth:
            return 0  # a bodyless declaration
        if char in "<([":
            depth += 1
        elif char in ">)]":
            depth -= 1
        elif char == "{" and depth <= 0:
            return masked.count("\n", sig.start, matching_brace(masked, i)) + 1
        i += 1
    return 0


def instrument_sites(tree, sources, wrappers=None):
    """{instrument: {unit -> count}} for every instrument. See the doc block.

    The unit is the repo-relative file for all but two: `dup_consts` is keyed
    by the constant's *name*, because a duplicate is a relationship between
    files and not a property of one, and `untested_dirs` by the directory. The
    ratcheted number is the sum of each counter's values, and `--dimension
    NAME` prints the counter itself, which is how a later slice finds where
    the debt sits.

    Counters are sparse: a file with none of an instrument's sites is absent
    rather than zero, so the breakdown reads as a list of what is left.
    """
    sites = {name: collections.Counter() for name in INSTRUMENT_KEYS}
    sites["wrappers"] = wrappers if wrappers is not None else wrapper_scan(tree)[0]
    allow = unsafe_fn_allowlist()
    aliases = raw_aliases(tree)
    consts = {}
    tops, tested = set(), set()
    emit_call = emit_call_re(emit_wrappers(tree))
    for file, masked in tree.items():
        source = sources[file]
        for name, needle in INSTRUMENTS.items():
            if found := len(needle.findall(masked)):
                sites[name][file] = found
        declarations = list(fn_signatures(masked))
        for name, needle in INSTRUMENTS_PARAMS.items():
            if found := sum(len(needle.findall(s.params)) for s in declarations):
                sites[name][file] = found
        for name, needle in INSTRUMENTS_RETURNS.items():
            if found := sum(len(needle.findall(s.returns)) for s in declarations):
                sites[name][file] = found
        fields = list(struct_bodies(masked))
        for name, needle in INSTRUMENTS_FIELDS.items():
            if found := sum(len(needle.findall(body)) for body in fields):
                sites[name][file] = found
        if found := sum(1 for _ in unsafe_fn_items(masked)):
            sites["unsafe_fns"][file] = found
        if found := unsafe_fns_without_raw(masked, file, allow, aliases):
            sites["unsafe_fns_without_raw_params"][file] = found
        if found := safety_on_safe_fns(source, masked, declarations):
            sites["safety_on_safe_fns"][file] = found
        if found := sum(
            1
            for m in CHAR_AS_C_INT.finditer(source)
            if masked[m.start(m.lastindex)] == source[m.start(m.lastindex)]
        ):
            sites["char_as_c_int"][file] = found
        if found := emit_sentinels(masked, emit_call):
            sites["emit_sentinel"][file] = found
        if in_home(file, TYPES_HOME):
            sites["types_files"][file] = 1
        if not GENERATED_MARKER.search(source[:GENERATED_HEAD]):
            if found := sum(
                fn_body_lines(masked, s) > LONG_FN_LINES for s in declarations
            ):
                sites["long_fns"][file] = found
        for match in CONST_DECL.finditer(masked):
            value = " ".join(source[match.start(2) : match.end(2)].split())
            consts.setdefault((match.group(1), value), set()).add(file)
        if file.startswith(CRATE_SRC):
            # `X.rs` at the top is `X/`'s module file when the directory
            # exists (`state.rs` beside `state/`), and its tests are that
            # module's; a lone top-level file is not a directory and is not
            # asked, which `tops` (directories only) already says.
            rest = file[len(CRATE_SRC) :]
            top = rest.split("/", 1)[0] if "/" in rest else rest.removesuffix(".rs")
            if "/" in rest:
                tops.add(top)
            if "#[test]" in masked:
                tested.add(top)
    for (name, _), files in consts.items():
        if len(files) > 1:
            sites["dup_consts"][name] += len(files) - 1
    for unit, others in UNIT_TEST_ALSO_COVERS.items():
        if (UNIT_TESTS / f"{unit}.rs").exists():
            tested.update(others)
    for path in UNIT_TESTS.iterdir():
        stem = path.name if path.is_dir() else path.name.removesuffix(".rs")
        if path.is_dir() or path.name.endswith(".rs"):
            if top := unit_covers(stem, tops):
                tested.add(top)
    for top in tops - tested - UNTESTED_EXEMPT.keys():
        sites["untested_dirs"][top] = 1
    return sites


def silent_failed_sites(tree):
    """`--dimension silent_failed`: the `Failed`s made where nothing was said,
    per file (unratcheted: see SILENT_FAILED)."""
    emit_call = emit_call_re(emit_wrappers(tree))
    return collections.Counter(
        {f: n for f, m in tree.items() if (n := silent_failed(m, emit_call))}
    )


def labeled_loops(tree):
    """`--dimension labeled_loops`: the labelled loops per file (unratcheted)."""
    return collections.Counter(
        {f: n for f, m in tree.items() if (n := len(LABELED_LOOPS.findall(m)))}
    )


def unit_covers(stem, tops):
    """The top-level module a unit file (or directory) named `stem` drives:
    the module of that exact name, else the longest module `M` the stem
    spells as `M_<what>` (`channel_reader` drives `channel`, `eval_decode`
    drives `eval`), else None. Longest first, so a module whose name holds an
    underscore (`ui_compositor`) is never mistaken for its prefix's (`ui`)."""
    if stem in tops:
        return stem
    return max((t for t in tops if stem.startswith(t + "_")), key=len, default=None)


def api_exported():
    """The API function names apigen dispatches, from its attribute spec.

    One method per non-comment line, name first. An entry carrying `alias=`
    is a deprecated *spelling* that shares the target's wrapper and one
    carrying `handler=` names a hand-written handler elsewhere in the crate;
    neither has an `nvim_*` function of its own under `api/`, so neither
    freezes a signature. Everything else does, `lua_only` included: the Lua
    binding names its parameters too.
    """
    names = set()
    for line in API_SPEC.read_text().splitlines():
        line = line.split("#", 1)[0].split()
        if not line or any(
            field.startswith(("alias=", "handler=")) for field in line[1:]
        ):
            continue
        names.add(line[0])
    return names


def int_aliases(aliases):
    """The names in `aliases` whose chain bottoms out in a primitive integer.

    `aliases` maps a name to the set of things the tree aliases it to — a set
    because the same name is declared in more than one module and because a
    `cfg` can give one two spellings. A name is integral when *any* of its
    targets is, which is the reading that keeps the count from depending on
    which declaration a scan happened to see first. Cycles cannot happen in
    Rust, but the walk guards against one anyway rather than recursing off the
    stack if a sweep ever writes `type A = A;`.
    """

    def integral(name, seen):
        if name in INT_PRIMITIVES:
            return True
        if name in seen:
            return False
        seen.add(name)
        return any(integral(target, seen) for target in aliases.get(name, ()))

    return {
        name for name in aliases if name not in INT_PRIMITIVES and integral(name, set())
    }


def whole_tree(stats, tree, sites):
    """The name-keyed whole-tree counts. See the doc block."""
    return {
        **cell_ptr_partition(stats, tree),
        "cell_copy_owner": sum(
            len(CELL_COPY_OWNER_RE.findall(m)) for m in tree.values()
        ),
        **vocabulary(tree),
        **{name: sum(sites[name].values()) for name in INSTRUMENT_KEYS},
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


def check_cast_deny_floor(tree, report=False):
    """Every file without the cast deny is a `CAST_DENY_FLOOR` row, and every
    row still names a file without it. With `report`, print the floor."""
    without = {f for f, m in tree.items() if DENY_CASTS.search(m) is None}
    if report:
        print(f"files_without_deny_casts: {len(without)}")
        for file in sorted(without):
            print(f"  {file}  {CAST_DENY_FLOOR.get(file, 'NOT ON THE FLOOR')}")
    off = sorted(without - CAST_DENY_FLOOR.keys())
    stale = sorted(CAST_DENY_FLOOR.keys() - without)
    if off or stale:
        sys.exit(
            "ratchet: the cast deny's floor list is out of date:\n  "
            + "\n  ".join(
                [f"{f}: no cast deny and no CAST_DENY_FLOOR row" for f in off]
                + [f"{f}: a CAST_DENY_FLOOR row that excuses nothing" for f in stale]
            )
            + "\nGive the file the deny (finish its casts), or -- only for a "
            "perimeter file, a module root over one, or narrow.rs -- a row "
            "with its reason; drop a row whose file now carries the deny."
        )


def copy_sites(tree):
    """Every `derive_copy` site as (file, type name)."""
    return [
        (file, KEEPER_NAME.search(m.group(0)).group(1))
        for file, masked in tree.items()
        for m in VOCABULARY["derive_copy"].finditer(masked)
    ]


def repr_c_sites(tree):
    """Every `#[repr(C)]` off the perimeter -- both halves, editor state and
    foreign ABI -- as (file, the struct/union/enum it lays out)."""
    return [
        (file, name.group(1) if name else "?")
        for file, masked in tree.items()
        if not in_perimeter(file)
        for m in REPR_C.finditer(masked)
        for name in [KEEPER_NAME.search(masked, m.end(), m.end() + 400)]
    ]


# The item a `derive`/`repr` attribute sits on: the first aggregate after it.
KEEPER_NAME = re.compile(r"\b(?:struct|union|enum)\s+([A-Za-z_][A-Za-z0-9_]*)")


def keeper_row(floor, file, name):
    """The row of `floor` that excuses `name` in `file`, or None."""
    return floor.get(f"{file}::{name}", floor.get(file))


def keeper_findings(floor, label, sites):
    """`floor`'s rows that excuse no site, and its rows without a class."""
    found = {(f"{f}::{n}" if f"{f}::{n}" in floor else f) for f, n in sites}
    stale = [
        f"{row}: a {label} row that excuses nothing"
        for row in floor
        if row not in found
    ]
    classes = KEEPER_CLASSES[label]
    unclassed = [
        f"{row}: a {label} row whose reason does not open with one of {', '.join(classes)}"
        for row, why in floor.items()
        if why.split(":", 1)[0] not in classes
    ]
    return stale + unclassed


def check_keeper_floors(tree):
    """Every `COPY_KEEPERS`/`REPR_C_FLOOR` row still names a site, with a class."""
    found = keeper_findings(COPY_KEEPERS, "COPY_KEEPERS", copy_sites(tree))
    found += keeper_findings(REPR_C_FLOOR, "REPR_C_FLOOR", repr_c_sites(tree))
    if found:
        sys.exit(
            "ratchet: a keeper list is out of date:\n  "
            + "\n  ".join(found)
            + "\nDrop the row (lower the needle's baseline with it) or move it "
            "to where the type now lives."
        )


def keeper_report(floor, label, sites):
    """`--dimension copy_keepers|repr_c_floor`: each site and the row that
    keeps it, the ones off the floor (what is left to remove) first."""
    rows = [(keeper_row(floor, f, n), f, n) for f, n in sites]
    off = sum(1 for why, _, _ in rows if why is None)
    print(f"{label}: {len(rows)} sites, {len(rows) - off} kept, {off} off the floor")
    for why, file, name in sorted(rows, key=lambda r: (r[0] is not None, r[1], r[2])):
        print(f"  {file}::{name}  {why or 'OFF THE FLOOR'}")


def render(
    stats,
    ledger_counts,
    without_casts,
    allowing,
):
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
    head = "".join(f'  "{name}": {ledger_counts[name]},\n' for name in WHOLE_TREE_LABEL)
    allows = "".join(
        f'  "files_allowing_{lint}": {allowing[lint]},\n' for lint in FILE_ALLOWS
    )
    return (
        "{\n"
        f"{head}"
        f'  "files_without_deny_casts": {without_casts},\n'
        f"{allows}"
        f'  "files": {{\n{body}\n  }}\n'
        "}\n"
    )


# The whole-tree counts, and how a violation of each reads. The order here is
# the order they are written to metrics/ratchet.json.
WHOLE_TREE_LABEL = {
    "internal_exports": "abi-ledger internal exports",
    "test_reached_pub": "test-reached pub items",
    "cell_ptr_keepers": "cell_ptr sites on a ruled multi-site keeper",
    "cell_ptr_accessors": "one-per-cell acquire-once cell_ptr sites",
    "cell_copy_owner": "get() copies of a Copy global owning a pointer",
    # The C vocabulary, in the order the phases retire it.
    "c_int_returns": "`-> c_int` status-code returns",
    "ok_fail": "OK/FAIL returns and comparisons",
    "error_out_params": "`*mut Error` out-parameters",
    "semsg_c": "message templates that are data, not literals",
    "raw_cstr": "raw `c_char` pointer types",
    "libc_strings": "libc str*/mem* calls",
    "const_c_int": "`pub const NAME: c_int` constants",
    "const_int_alias": "`pub const NAME: <integer alias>` constants",
    "unions": "union declarations",
    "repr_c_ffi_types": "`#[repr(C)]` on a foreign ABI's own types",
    "repr_c_editor_state": "`#[repr(C)]` on the editor's own state",
    "derive_copy": "Copy derives on braced aggregates",
    "manual_alloc": "xmalloc/xfree-family calls",
    "garray_sites": "GArray call sites",
    "kvec_sites": "kvec call sites and borrowed views",
    "khash_sites": "khash monomorph type names",
    "ptr_arith": "pointer-arithmetic method calls",
    "t_suffix_types": "distinct `_T` type declarations",
    "raw_win_buf_sigs": "raw win/buf/tabpage pointers in fn signatures",
    "raw_frame_sigs": "raw frame pointers in fn signatures",
    "mut_win_buf_refs": "&mut win/buf/tabpage borrows in fn signatures",
    "abbrev_params": "transpiler parameter abbreviations in fn signatures",
    "curwin_raw": "curwin/curbuf/curtab get()s outside winlayer",
    # The instruments, in INSTRUMENT_KEYS' order.
    "unsafe_fns": "`unsafe fn` items",
    "unsafe_fns_without_raw_params": "`unsafe fn`s with no raw pointer in the signature, off the allowlist",
    "wrappers": "single-call `unsafe { f(..) }` regions",
    "raw_ptr_params": "raw-pointer parameters",
    "raw_cstr_params": "raw `c_char` pointers in a parameter list",
    "raw_cstr_returns": "raw `c_char` pointer returns",
    "raw_cstr_fields": "raw `c_char` pointers in a struct's fields",
    "mbyte_raw": "pointer-form multibyte cursor calls",
    "bytes_at": "bytes_at() calls",
    "vval_raw": "`.vval.` union reads",
    "typval_raw_params": "raw `TypVal` pointers in a parameter list",
    "exarg_raw": "raw `ExArg` pointers in a parameter list",
    "cmdarg_raw": "raw `CmdArg` pointers in a parameter list",
    "global_cells": "GlobalCell static declarations",
    "cell_raw_ptr": "GlobalCells holding a raw pointer",
    "api_err_params": "`&mut Error` out-parameters",
    "char_as_c_int": "character literals cast to C's int",
    "emit_sentinel": "a message emitted, then a sentinel returned",
    "labeled_blocks": "labelled blocks and loops (the transpiled goto)",
    "long_fns": f"functions over {LONG_FN_LINES} lines outside generated files",
    "safety_on_safe_fns": "safe fns whose doc comment still has a `# Safety` section",
    "dup_consts": "redundant copies of a constant declared in another file",
    "types_files": "files under types/",
    "untested_dirs": "top-level modules with no test of any kind",
    "cchar_scratch": "fixed-size `[c_char; N]` buffers",
}
# The C-vocabulary subset of the above, for the run's summary line.
VOCABULARY_KEYS = (
    *VOCABULARY,
    *VOCABULARY_OUTSIDE,
    *VOCABULARY_INSIDE,
    *VOCABULARY_PARAMS,
    "const_int_alias",
    "t_suffix_types",
    "raw_win_buf_sigs",
    "raw_frame_sigs",
    "mut_win_buf_refs",
    "abbrev_params",
)


def violations(
    stats,
    counts,
    without_casts,
    allowing,
    baseline,
):
    """Every metric that grew past the committed baseline."""
    found = []
    for name, label in WHOLE_TREE_LABEL.items():
        # .get: absent from baselines committed before the metric existed.
        base = baseline.get(name, counts[name])
        if counts[name] > base:
            found.append(f"{label}: {base} -> {counts[name]}")
    base_casts = baseline.get("files_without_deny_casts", without_casts)
    if without_casts > base_casts:
        found.append(f"files without the cast deny: {base_casts} -> {without_casts}")
    for lint, count in allowing.items():
        # .get: absent from baselines committed before the metric existed.
        base_allow = baseline.get(f"files_allowing_{lint}", count)
        if count > base_allow:
            found.append(f"files allowing {lint}: {base_allow} -> {count}")
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


def summary(stats, counts, without_casts, allowing):
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
        f"{perimeter_lines(stats)[0]} unchecked statements on the perimeter",
        f"{without_casts} files without the cast deny",
    ]
    parts += [f"{n} files allowing {lint}" for lint, n in allowing.items()]
    parts += [f"{counts[name]} {name}" for name in VOCABULARY_KEYS]
    parts += [f"{counts[name]} {name}" for name in INSTRUMENT_KEYS]
    return ", ".join(parts)


# Scanner cases, checked on every run (a few hundred microseconds against a
# ~20 MB tree read). A silent regression in mask()/unsafe_stmts() would
# corrupt every number the ratchet enforces, so this is not opt-in.
SELF_TEST = [
    # (source, expected unsafe_stmts in a file without the deny)
    ("fn f() {\n    unsafe {\n        g();\n    }\n}\n", 1),
    ("fn f() {\n    let x = unsafe { *p };\n}\n", 1),
    # Comments and blank lines inside a region are free.
    ("fn f() {\n    unsafe {\n        // SAFETY: fine.\n\n        g();\n    }\n}\n", 1),
    # ... and so is layout: a call rustfmt wrapped over four lines is one
    # statement, which is the whole point of charging statements.
    (
        "fn f() {\n    unsafe {\n        g(\n            a,\n            b,\n        );\n    }\n}\n",
        1,
    ),
    # Three statements cost three however they are spread, and splitting the
    # region into three regions does not change that.
    ("fn f() {\n    unsafe {\n        a();\n        b();\n        c();\n    }\n}\n", 3),
    ("fn f() {\n    unsafe { a() };\n    unsafe { b() };\n    unsafe { c() };\n}\n", 3),
    ("fn f() {\n    unsafe { (*p).a = 1; (*p).b = 2 }\n}\n", 2),
    # A nested block's statements are charged too -- top-level-only counting
    # would price a wrapped-up transpiled body at 1.
    (
        "fn f() {\n    unsafe {\n        loop {\n            a();\n            b();\n        }\n    }\n}\n",
        3,
    ),
    # A closure's body is inside the region.
    (
        "fn f() {\n    unsafe {\n        g(|x| {\n            a(x);\n            b(x)\n        })\n    }\n}\n",
        3,
    ),
    # Each `match` arm is a statement; the arms' own blocks charge on top.
    (
        "fn f() {\n    unsafe {\n        match k {\n            A => a(),\n            B => b(),\n        }\n    }\n}\n",
        3,
    ),
    # A struct literal is one expression, not a block with a tail.
    (
        "fn f() {\n    unsafe {\n        Pos {\n            lnum: 1,\n            col: 2,\n        }\n    }\n}\n",
        1,
    ),
    (
        "fn f() {\n    unsafe {\n        Self {\n            n: 1,\n        }\n    }\n}\n",
        1,
    ),
    # ... but a SCREAMING_CASE constant before a brace is an `if`, and its
    # block's tail is real.
    (
        "fn f() {\n    unsafe {\n        if k == LOCAL {\n            a()\n        }\n    }\n}\n",
        2,
    ),
    # An empty region still promises something.
    ("fn f() {\n    unsafe {}\n}\n", 1),
    # Prose and strings never count.
    ("/// An unsafe fn would need one.\n/// unsafe { }\nfn f() {}\n", 0),
    ('fn f() {\n    let s = "unsafe { g(); }";\n}\n', 0),
    ('fn f() {\n    let s = r#"unsafe {"#;\n}\n', 0),
    ("/* unsafe { /* nested */ } */\nfn f() {}\n", 0),
    # A brace in a char literal must not desynchronise the scanner.
    ("fn f() {\n    unsafe {\n        g('{');\n    }\n    h();\n}\n", 1),
    # ... and neither must an escaped one. `'\\'` ends at its own quote; a
    # scanner that reads the second backslash as an escape runs on to the
    # next quote in the file and eats every brace in between.
    ("fn f() {\n    unsafe {\n        g('\\\\');\n    }\n    h('x');\n}\n", 1),
    ("fn f() {\n    unsafe {\n        g('\\'');\n    }\n    h('x');\n}\n", 1),
    ("fn f() {\n    unsafe {\n        g('\\u{1b}');\n    }\n    h('x');\n}\n", 1),
    ("fn f<'a>(x: &'a u8) {}\n", 0),
    # An unsafe fn body is implicitly unsafe throughout without the deny.
    ("unsafe fn f() {\n    g();\n}\n", 1),
    ('unsafe extern "C" fn f() {\n    g();\n}\n', 1),
    ("trait T {\n    unsafe fn f();\n}\n", 1),
    # A nested region inside a charged one is not charged again: the body is
    # one statement, and the block it holds is that statement.
    ("unsafe fn f() {\n    unsafe {\n        g();\n    }\n}\n", 2),
    # Declarations, promises, types.
    ("unsafe impl Sync for X {}\n", 1),
    ("unsafe trait T {}\n", 1),
    ('type F = unsafe extern "C" fn(u8);\n', 0),
    ("struct S(Option<unsafe fn(u8)>);\n", 0),
    ('#[unsafe(no_mangle)]\npub extern "C" fn f() {}\n', 0),
    # An `unsafe extern` block costs its declarations.
    ('unsafe extern "C" {\n    static x: u8;\n}\n', 1),
    ('unsafe extern "C" {\n    static x: u8;\n    fn g(n: u8);\n}\n', 2),
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
# (source, expected: does the file allow non_camel_case_types?)
# (source, the lints whose blanket allow it carries)
SELF_TEST_FILE_ALLOWS = [
    ("#![allow(non_camel_case_types)]\n", {"non_camel_case_types"}),
    ("#![allow(non_snake_case)]\n", {"non_snake_case"}),
    ("#![allow(non_upper_case_globals)]\n", {"non_upper_case_globals"}),
    (
        "#![allow(non_camel_case_types)]\n#![allow(non_snake_case)]\n",
        {"non_camel_case_types", "non_snake_case"},
    ),
    # An item-level allow is a different attribute and is not the claim: the
    # `nvim__*` API names carry one each and their files do not count.
    ("#[allow(non_camel_case_types)]\nstruct uv_loop_t;\n", set()),
    ("#[allow(non_snake_case)]\npub fn nvim__id() {}\n", set()),
    ("#[allow(non_upper_case_globals)]\nunsafe fn apply_opts() {}\n", set()),
    # Prose about the attribute does not switch it on.
    ("//! `#![allow(non_camel_case_types)]` is what libuv's names need.\n", set()),
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
    ("unsafe fn f() {\n    unsafe {\n        g();\n    }\n}\n", 1),
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
# (source, expected self_projections)
SELF_TEST_SELF_PROJECTION = [
    # The two batch 3 shipped.
    ("fn f() {\n    *arg = *arg.add(1);\n}\n", 1),
    ("fn f() {\n    *pp = *pp.add(3);\n}\n", 1),
    ("fn f() {\n    *p = *p.offset(-1);\n}\n", 1),
    # The parenthesised form is the point.
    ("fn f() {\n    *arg = (*arg).add(1);\n}\n", 0),
    # An array of pointers indexed into a *different* place is ordinary.
    ("fn f() {\n    *out = *files.offset(i);\n}\n", 0),
    ("fn f() {\n    let file = *files.offset(i);\n}\n", 0),
    # Prose about one costs nothing.
    ("// *arg = *arg.add(1);\n", 0),
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
    ("fn f() {\n    REFRESH_TIMER.as_raw();\n}\n", 1),
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
    ("fn f() {\n    highlight_attr.ptr();\n}\n", {"highlight_attr": 1}),
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
    ("// highlight_attr.ptr()\nfn f() {}\n", {}),
]
# (tree, expected partition). The boundary is subtracted, the keeper register
# is counted, and everything left is the accessor class -- including the site
# with no bare-identifier receiver.
SELF_TEST_CELL_PTR_PARTITION = [
    (
        {
            "a.rs": "fn f() {\n    main_loop.ptr();\n    curwin.ptr();\n}\n",
            "b.rs": "fn g() {\n    cell.ptr();\n    cell.ptr();\n"
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
    (
        {"a.rs": "fn f() {\n    cell.ptr();\n    cell.ptr();\n}\n"},
        False,
    ),
]
# (source, expected cell_copy_owner)
SELF_TEST_CELL_COPY_OWNER = [
    ("fn f() {\n    curgrid.get();\n}\n", 1),
    ("fn f() {\n    let n = CLIPBOARD.get().data;\n}\n", 1),
    # Only `get`: the other accessors do not hand out a second owner.
    ("fn f() {\n    curgrid.set(x);\n}\n", 0),
    ("fn f() {\n    curgrid.with(|s| s);\n}\n", 0),
    # rustfmt wraps a long chain, and the copy still happens.
    ("fn f() {\n    *CLIPBOARD\n        .get()\n}\n", 1),
    # A global that is not on the list.
    ("fn f() {\n    p_ai.get();\n}\n", 0),
    # A longer name ending in a listed one is not it.
    ("fn f() {\n    saved_curgrid.get();\n}\n", 0),
    ("// curgrid.get()\nfn f() {}\n", 0),
]


# (source, the wrapper callees the file holds, in source order). A wrapper is
# a single-call region whatever surrounds the braces and whatever its layout;
# a dereference, a borrow and a compound body are not wrappers at all.
SELF_TEST_WRAPPERS = [
    ("fn f() {\n    let x = unsafe { g(p) };\n}\n", ["g"]),
    ("fn f() -> u8 {\n    unsafe { g(p) }\n}\n", ["g"]),
    ("fn f() -> u8 {\n    return unsafe { g(p) };\n}\n", ["g"]),
    ("fn f() {\n    if unsafe { g(p) } {\n    }\n}\n", ["g"]),
    ("fn f() {\n    h(unsafe { g(p) });\n}\n", ["g"]),
    ("fn f() {\n    unsafe { g(p) }.m();\n}\n", ["g"]),
    # A call rustfmt wrapped over four lines is the same wrapper: reflowing a
    # retype must not move the number.
    (
        "fn f() {\n    unsafe {\n        g(\n            p,\n        )\n    };\n}\n",
        ["g"],
    ),
    # The key: the last segment, qualified when the one before it is a type.
    ("fn f() {\n    unsafe { module::g(p) };\n}\n", ["g"]),
    ("fn f() {\n    unsafe { Live::new(p) };\n}\n", ["Live::new"]),
    ("fn f() {\n    unsafe { Live::<u8>::new(p) };\n}\n", ["Live::new"]),
    # A method call is keyed by its name: the receiver's type is out of reach.
    ("fn f() {\n    unsafe { p.add(1) };\n}\n", [".add"]),
    ("fn f() {\n    unsafe { self.slots.at(i) };\n}\n", [".at"]),
    # A variadic's checked macro is its callee's wrapper; any other macro is not.
    ("fn f() {\n    unsafe { vim_snprintf!(b, n, f, a) };\n}\n", ["vim_snprintf"]),
    ("fn f() {\n    unsafe { other!(p) };\n}\n", []),
    # The deref class, which is the caller's own obligation and no wrapper.
    ("fn f() {\n    let x = unsafe { *p };\n}\n", []),
    ("fn f() {\n    unsafe { (*p).field = 1 };\n}\n", []),
    ("fn f() {\n    let r = unsafe { &mut *p };\n}\n", []),
    ("fn f() {\n    let r = &unsafe { *p };\n}\n", []),
    # ... and neither is a compound body or a parenthesised one.
    ("fn f() {\n    unsafe { a(); b() };\n}\n", []),
    ("fn f() {\n    unsafe { g(p) + 1 };\n}\n", []),
    ("fn f() {\n    unsafe { (g(p)) };\n}\n", []),
    ("fn f() {\n    unsafe { g(p)(q) };\n}\n", []),
    ("fn f() {\n    unsafe {};\n}\n", []),
    # Prose about one costs nothing.
    ("// unsafe { g(p) };\nfn f() {}\n", []),
]
# (source, allowlist, expected unsafe_fns_without_raw_params). The signature
# is the whole span -- parameters and return type -- so a function that hands
# raw memory back is a contract without needing a row.
SELF_TEST_NO_RAW = [
    ("unsafe fn f(tv: &TypVal) {\n}\n", (), 1),
    ("unsafe fn f(p: *mut u8) {\n}\n", (), 0),
    ("unsafe fn f(p: *const u8) {\n}\n", (), 0),
    ("unsafe fn f() -> *mut c_void {\n}\n", (), 0),
    # A rustfmt-wrapped list still reads as one signature.
    ("unsafe fn f(\n    a: u8,\n    b: *mut u8,\n) {\n}\n", (), 0),
    # The ABI is the contract.
    ('unsafe extern "C" fn f(x: u8) {\n}\n', (), 0),
    # `impl CPtr` hides a `*const c_char`, and so does a bound.
    ("unsafe fn c_str(p: impl CPtr) -> CDisplay {\n}\n", (), 0),
    ("unsafe fn c_str<P: CPtr>(p: P) {\n}\n", (), 0),
    # A safe fn is not this metric's business, and neither is a fn *type*.
    ("fn f(tv: &TypVal) {\n}\n", (), 0),
    ("type F = unsafe fn(tv: &TypVal);\n", (), 0),
    # An allowlist row excuses exactly its own item.
    ("unsafe fn f(tv: &TypVal) {\n}\n", (("f", "t.rs"),), 0),
    ("unsafe fn f(tv: &TypVal) {\n}\n", (("g", "t.rs"),), 1),
    # ... in its own file: a row naming another file excuses nothing.
    ("unsafe fn f(tv: &TypVal) {\n}\n", (("f", "u.rs"),), 1),
    # A method's row has to spell its owner; a bare name does not excuse it.
    (
        "impl Slots {\n    unsafe fn at(&self, i: usize) {\n    }\n}\n",
        (("Slots::at", "t.rs"),),
        0,
    ),
    (
        "impl Slots {\n    unsafe fn at(&self, i: usize) {\n    }\n}\n",
        (("at", "t.rs"),),
        1,
    ),
    # ... and a trait impl is keyed by the type, not the trait.
    (
        "impl Sink for Slots {\n    unsafe fn at(&self, i: usize) {\n    }\n}\n",
        (("Slots::at", "t.rs"),),
        0,
    ),
]
# (tree, allowlist, whether check_unsafe_fn_allowlist rejects it). A row that
# excuses nothing has to leave the list.
SELF_TEST_NO_RAW_CHECK = [
    ({"a.rs": "unsafe fn f(tv: &TypVal) {\n}\n"}, (("f", "a.rs"),), False),
    ({"a.rs": "unsafe fn f(tv: &TypVal) {\n}\n"}, (("gone", "a.rs"),), True),
    # The function is still there, but the needle would skip it now.
    ({"a.rs": "unsafe fn f(p: *mut u8) {\n}\n"}, (("f", "a.rs"),), True),
    ({"a.rs": 'unsafe extern "C" fn f(x: u8) {\n}\n'}, (("f", "a.rs"),), True),
    # A method row, spelled with its owner.
    (
        {"a.rs": "impl S {\n    unsafe fn f(&self) {\n    }\n}\n"},
        (("S::f", "a.rs"),),
        False,
    ),
    (
        {"a.rs": "impl S {\n    unsafe fn f(&self) {\n    }\n}\n"},
        (("f", "a.rs"),),
        True,
    ),
    # The file column is checked: a row naming the wrong file excuses nothing.
    ({"a.rs": "unsafe fn f(tv: &TypVal) {\n}\n"}, (("f", "b.rs"),), True),
]

# (tree, expected unsafe_fns_without_raw_params over the whole tree). An alias
# for a raw pointer counts as one wherever a signature spells it, and the
# alias may be declared in another file than the function that takes it.
SELF_TEST_RAW_ALIAS = [
    # No alias in sight: the parameter is a plain name.
    ({"a.rs": "unsafe fn f(m: MetaFilter) {\n}\n"}, 1),
    # ... declared next door, it is a pointer.
    (
        {
            "t.rs": "pub type MetaFilter = *const uint32_t;\n",
            "a.rs": "unsafe fn f(m: MetaFilter) {\n}\n",
        },
        0,
    ),
    # The return type is part of the signature here too.
    (
        {
            "t.rs": "type ArenaMem = *mut ConsumedBlk;\n",
            "a.rs": "unsafe fn f(n: usize) -> ArenaMem {\n}\n",
        },
        0,
    ),
    # An alias for something that is not a pointer stays invisible.
    (
        {
            "t.rs": "pub type LineNr = i64;\n",
            "a.rs": "unsafe fn f(l: LineNr) {\n}\n",
        },
        1,
    ),
    # A generic alias is still an alias.
    (
        {
            "t.rs": "pub type Slot<T> = *mut T;\n",
            "a.rs": "unsafe fn f(s: Slot<u8>) {\n}\n",
        },
        0,
    ),
    # A prefix of an alias is a different name.
    (
        {
            "t.rs": "pub type Meta = *const u8;\n",
            "a.rs": "unsafe fn f(m: MetaFilter) {\n}\n",
        },
        1,
    ),
]


# (path, whether the perimeter claims it). A directory entry claims its
# subtree and nothing else; an exact-path entry claims exactly itself.
SELF_TEST_PERIMETER = [
    ("crates/nvim/src/lua/executor/state.rs", True),
    ("crates/nvim/src/lua/stack/mod.rs", True),
    ("crates/nvim/src/os/fs/mod.rs", True),
    ("crates/nvim/src/memory/arena.rs", True),
    ("crates/nvim/src/winlayer/live.rs", True),
    # The editor proper, including modules that merely look raw.
    ("crates/nvim/src/memline/block0.rs", False),
    ("crates/nvim/src/memfile/swapfile.rs", False),
    ("crates/nvim/src/marktree/node.rs", False),
    ("crates/nvim/src/eval/typval.rs", False),
    # The Lua runtime's editor logic, beside the exact paths that stay.
    ("crates/nvim/src/lua/executor/exec.rs", False),
    ("crates/nvim/src/lua/stdlib/vars.rs", False),
    # A sibling whose name starts with a directory entry is not inside it.
    ("crates/nvim/src/luaref.rs", False),
    # ... and neither is a longer path built on an exact-path entry.
    ("crates/nvim/src/global_cell.rs.orig", False),
    ("crates/nvim/src/memory.rs", False),
]
# (stats, expected (inside, outside)). The split is over `unsafe_stmts`
# alone, and a file with none contributes to neither side.
SELF_TEST_PERIMETER_SPLIT = [
    (
        {
            "crates/nvim/src/lua/ffi.rs": {"unsafe_stmts": 150},
            "crates/nvim/src/memory/arena.rs": {"unsafe_stmts": 400},
            "crates/nvim/src/memline/mod.rs": {"unsafe_stmts": 183},
            "crates/nvim/src/types/memline.rs": {"unsafe_stmts": 0},
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


# The C vocabulary, case by case: ({repo-relative path: source}, {metric:
# expected}). Only the named metrics are asserted, so a case may say `-> c_int`
# without also stating what `c_int_returns` makes of it. Each case pins the
# variants the needle has to see (a wrapped signature, both constnesses, every
# spelling of a comparison) and the near-misses it must not (`xfree_clear`,
# `p.addr()`, `semsg(gettext(`, a tuple struct's `Copy`).
SELF_TEST_VOCABULARY = [
    (
        {
            "crates/nvim/src/a.rs": "fn a() -> c_int {\n}\n"
            "fn b() -> c_int;\n"
            "type F = fn() -> c_int;\n"
            "fn c(\n    x: u8,\n) -> c_int\n{\n}\n"
            "fn e() -> ::core::ffi::c_int {\n}\n"
            "fn f() -> core::ffi::c_int {\n}\n"
            "fn g() -> std::ffi::c_int {\n}\n"
            "fn h() -> libc::c_int {\n}\n"
            "fn d() -> c_int_ish {\n}\n"
            "fn i() -> c_uint {\n}\n"
        },
        {"c_int_returns": 8},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() -> c_int {\n"
            "    if x == OK { return FAIL; }\n"
            "    if y != FAIL { return OK; }\n"
            "    if z >= OK { }\n"
            "    OK\n}\n"
        },
        {"ok_fail": 4},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f(err: *mut Error) {\n}\n"
            "fn g(err: &mut Error) {\n}\n"
            "fn h(err: *mut ErrorType) {\n}\n"
            "struct S {\n    err: *mut Error,\n}\n"
        },
        {"error_out_params": 1},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    semsg_c!(x);\n"
            "    semsg_multiline_c!(y);\n"
            "    tr_c!(z, a);\n"
            "    tr_plural!(w, b);\n"
            "    emsg(gettext(v));\n"
            '    semsg!("E1: {a}");\n'
            "    emsg(other);\n}\n"
        },
        {"semsg_c": 4},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f(a: *mut c_char, b: *const c_char) "
            "-> *mut c_char {\n"
            "    let c: *mut c_uchar = q;\n}\n"
        },
        {"raw_cstr": 3},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    strlen(a);\n    xstrlcpy(b);\n    vim_strchr(c);\n"
            "    memcpy(d);\n    libc::strcmp(e);\n}\n"
        },
        {"libc_strings": 3},
    ),
    (
        {
            "crates/nvim/src/a.rs": "pub const A_B: c_int = 1;\n"
            "pub const C: c_uint = 2;\n"
            "const D: c_int = 3;\n"
            "pub const E: usize = 4;\n"
            # c2rust spelled most of them path-qualified.
            "pub const F: ::core::ffi::c_int = 5;\n"
            "pub const G: core::ffi::c_uint = 6;\n"
            "pub const H: libc::c_int = 7;\n"
            "pub const I: ::core::ffi::c_long = 8;\n"
        },
        {"const_c_int": 5},
    ),
    (
        # The alias set is whole-tree: `auto_event` is declared in one file
        # and spent in another. `Handle` reaches an integer through a second
        # alias; `Opaque` never does; `usize` is a primitive, not an alias,
        # so a size constant is not a family. A private `const` is out for
        # the same reason it is out of `const_c_int` — the debt is a family
        # other modules can name.
        {
            "crates/nvim/src/a.rs": "pub type auto_event = ::core::ffi::c_uint;\n"
            "type Handle = linenr_T;\n"
            "pub type linenr_T = c_long;\n"
            "pub type Opaque = SomeStruct;\n",
            "crates/nvim/src/b.rs": "pub const EVENT_BUF_NEW: auto_event = 0;\n"
            "pub const CMD_append: auto_event = 1;\n"
            "pub const FIRST: Handle = 2;\n"
            "pub const HIDDEN: Opaque = Opaque::X;\n"
            "pub const SIZE: usize = 4;\n"
            "const PRIVATE: auto_event = 5;\n",
        },
        {"const_int_alias": 3},
    ),
    (
        {"crates/nvim/src/a.rs": "pub union U {\n}\nunion V {\n}\nstruct W;\n"},
        {"unions": 2},
    ),
    (
        {
            "crates/nvim/src/a.rs": "#[derive(Clone, Copy)]\n#[repr(C)]\n"
            "pub struct A {\n    x: c_int,\n}\n"
            "#[derive(Copy, Clone)]\npub enum B {\n    X,\n}\n"
            "#[derive(Clone, Copy)]\nstruct C(u8);\n"
            "#[derive(Clone)]\nstruct D {\n    x: u8,\n}\n"
            "#[derive(Copy, Clone)]\npub union E {\n    x: u8,\n}\n"
            "#[repr(C, packed)]\nstruct G {\n    x: u8,\n}\n",
            # Inside the perimeter the layout is a foreign ABI's, so neither
            # `#[repr(C)]` here is debt.
            "crates/nvim/src/os/b.rs": "#[repr(C)]\nstruct H {\n    x: u8,\n}\n",
            # ... and the same goes for a foreign ABI's type file, which
            # counts on the other side of the split rather than not at all.
            "crates/nvim/src/types/uv.rs": "#[repr(C)]\n"
            "pub struct uv_loop_s {\n    x: u8,\n}\n"
            "#[repr(C)]\npub struct uv_idle_s {\n    y: u8,\n}\n",
        },
        {"derive_copy": 2, "repr_c_editor_state": 2, "repr_c_ffi_types": 2},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    xmalloc(1);\n    xmallocz(2);\n    xcalloc(3, 4);\n"
            "    xrealloc(p, 5);\n    xfree(p);\n"
            "    xstrdup(q);\n    xfree_clear(r);\n}\n"
        },
        {"manual_alloc": 5},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    ga_init(a);\n    ga_grow(b, 1);\n    ga_clear(c);\n"
            "    ga_concat(d, e);\n    ga_append(f, g);\n"
            "    ga_clear_strings(h);\n    ga_concat_len(i, j, k);\n}\n"
        },
        {"garray_sites": 5},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    kv_size!(a);\n    kv_push(b, c);\n    kv_destroy(d);\n"
            "    let v = InitVec::new(&mut buf);\n"
            "    let w = Kvec::new(&mut ga);\n"
            # Not calls: a field name, a bare mention, and a type.
            "    e.init_array[0] = 1;\n    let _ = kv_size;\n"
            "    let _: KvecOf<u8>;\n}\n"
        },
        {"kvec_sites": 5},
    ),
    (
        {
            "crates/nvim/src/a.rs": "static t: Map_String_int = f();\n"
            "fn f() -> Map_String_int {\n"
            "    let s: Set_cstr_t = g();\n"
            # `Map`/`Set` on their own, and as a suffix, are not monomorphs.
            "    let _: Map = h();\n    let _: HashSet_ish = i();\n}\n"
        },
        {"khash_sites": 3},
    ),
    (
        {
            "crates/nvim/src/a.rs": "struct S {\n    at: *mut u8,\n    n: usize,\n}\n"
            "fn f(p: *mut u8, n: usize, s: &S) {\n"
            "    p.offset(1);\n    p.add(2);\n    p.sub(3);\n"
            "    p.wrapping_add(4);\n    p.wrapping_sub(5);\n"
            "    p.offset_from(q);\n    p.addr();\n"
            # Integer `wrapping_*` are not pointer arithmetic: a parameter,
            # a field, a length.
            "    n.wrapping_add(1);\n    s.n.wrapping_sub(1);\n"
            "    v.len().wrapping_sub(1);\n"
            # ... and a pointer reached through a field, a `let`, a cast,
            # a method that answers one, and a function that returns one.
            "    s.at.wrapping_add(1);\n    let q = p;\n    q.wrapping_add(1);\n"
            "    (n as *const u8).wrapping_add(1);\n"
            "    v.as_ptr().wrapping_add(1);\n    g().wrapping_sub(1);\n}\n"
            "fn g() -> *mut u8 {\n    h()\n}\n"
        },
        {"ptr_arith": 11},
    ),
    (
        # A name is counted once however many files declare or mention it.
        # The `_T` spellings here are fixture *data* for the count being
        # asserted, not references to real types: a rename sweep that
        # reaches them changes the expected number.
        {
            "crates/nvim/src/a.rs": "pub struct buf_T {\n    x: u8,\n}\n"
            "pub type linenr_T = c_long;\n"
            "fn f(b: *mut buf_T) {\n}\n",
            "crates/nvim/src/b.rs": "pub type linenr_T = c_long;\n"
            "pub enum foo_T {\n    X,\n}\n"
            "fn g(l: linenr_T) {\n}\n",
        },
        {"t_suffix_types": 3},
    ),
    (
        # `*const` counts beside `*mut`; the local and the function-pointer
        # type do not. `&mut` on the same three types is its own number, and
        # `&Window` -- a shared borrow, which aliases legally -- is neither.
        {
            "crates/nvim/src/a.rs": "fn f(\n    wp: *mut Window,\n"
            "    buf: *const Buffer,\n) -> *mut Tabpage {\n"
            "    let x: *mut Window = q;\n}\n"
            "type Cb = fn(*mut Window);\n"
            "fn g(wp: &mut Window, other: &Window, tp: &mut Tabpage) {\n}\n"
            "fn h(fr: *mut Frame, out: *mut *mut Frame) -> *const Frame {\n}\n",
            # The handles' own `DerefMut` is where the borrow is made.
            "crates/nvim/src/winlayer/handles.rs": "fn deref_mut(&mut self) -> &mut Window {\n}\n",
        },
        {"raw_win_buf_sigs": 3, "raw_frame_sigs": 3, "mut_win_buf_refs": 2},
    ),
    (
        # `_eap` counts, `old_buf` and the local `ptr` do not, and the two
        # carved-out subtrees are silent however they spell a parameter.
        # `buf` counts on a buffer object and not on a byte buffer. Under
        # `api/` a method apigen dispatches is frozen and a helper beside it
        # is not, so `nvim_buf_line_count` is silent and `unpack` is not; the
        # same name outside `api/` is ordinary code and counts. A `ptr` is
        # counted on a `*mut c_char` and not on a `*mut c_void`, and the
        # local `ptr` in `f` is still not a binding either way.
        {
            "crates/nvim/src/a.rs": "fn f(\n    wp: *mut Window,\n"
            "    _eap: *mut ExArg,\n    old_buf: *mut Buffer,\n"
            "    buf: Option<Buf>,\n    _bp: &mut [u8],\n) {\n"
            "    let ptr: *mut c_char = q;\n}\n"
            "fn d(ptr: *mut c_char, q: u8) {\n}\n"
            "fn r(ptr: *mut c_void) {\n}\n",
            "crates/nvim/src/b.rs": "fn e(buf: &mut NumBuf, bp: *mut Buffer) {\n}\n",
            "crates/nvim/src/lua/b.rs": "fn g(buf: *mut Buffer) {\n}\n",
            "crates/nvim/src/vterm/c.rs": "fn h(cp: *mut c_char) {\n}\n",
            "crates/nvim/src/api/buffer.rs": "fn nvim_buf_line_count(buf: Buffer) {\n}\n"
            "fn unpack(buf: *mut Buffer) {\n}\n",
            "crates/nvim/src/eval/c.rs": "fn nvim_buf_line_count(buf: Buffer) {\n}\n",
        },
        {"abbrev_params": 7},
    ),
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n    curwin.get();\n"
            "    curbuf.get();\n    curtab.get();\n    curwin.with(|w| w);\n}\n",
            "crates/nvim/src/winlayer.rs": "fn g() {\n    curwin.get();\n}\n",
            "crates/nvim/src/winlayer/win.rs": "fn h() {\n    curbuf.get();\n}\n",
        },
        {"curwin_raw": 3},
    ),
]

# The instruments, each case naming the distinction it pins. Same shape as
# SELF_TEST_VOCABULARY: a fake tree, and the subset of the totals it fixes.
SELF_TEST_INSTRUMENTS = [
    # An `unsafe fn` item counts; a function-pointer *type*, a declaration
    # inside an `unsafe extern` block and a plain `unsafe {}` do not.
    (
        {
            "crates/nvim/src/a.rs": "pub unsafe fn f() {\n}\n"
            'pub unsafe extern "C" fn g() {\n}\n'
            "type H = unsafe fn(x: u8);\n"
            'unsafe extern "C" {\n    pub fn h(x: u8);\n}\n'
            "fn i() {\n    unsafe { f() };\n}\n",
        },
        {"unsafe_fns": 2},
    ),
    # A parameter and a return are different debts and never share a count.
    (
        {
            "crates/nvim/src/a.rs": "fn f(name: *mut c_char) -> *const c_char {\n}\n"
            "fn g(x: *mut u8, y: *const T, z: Option<*mut u8>) {\n}\n"
            "fn h() -> *mut c_char {\n}\n",
        },
        {
            "raw_cstr_params": 1,
            "raw_cstr_returns": 2,
            # `name`, `x` and `y`; the pointer nested in `z`'s `Option` is a
            # different retype and is deliberately not counted here.
            "raw_ptr_params": 3,
        },
    ),
    # ... and a qualified `c_char` is the same debt as a bare one. Spelling
    # the path is what an `unsafe extern` block and every generated file do,
    # so a forwarder rewritten into one must not move either number.
    (
        {
            "crates/nvim/src/a.rs": "fn f(\n"
            "    name: *mut ::core::ffi::c_char,\n"
            "    home: *const core::ffi::c_char,\n"
            "    tail: *mut libc::c_char,\n"
            ") -> *const ::std::ffi::c_char {\n}\n",
        },
        {
            "raw_cstr_params": 3,
            "raw_cstr_returns": 1,
            "raw_ptr_params": 3,
        },
    ),
    # A struct's *fields* are storage and are counted per field, whatever the
    # type wraps them in; a tuple struct, an enum payload, a `fn` line and a
    # local are not fields and are not counted.
    (
        {
            "crates/nvim/src/a.rs": "pub struct S {\n"
            "    pub name: *mut c_char,\n"
            "    home: Option<*const ::core::ffi::c_char>,\n"
            "    argv: *mut *mut libc::c_char,\n"
            "    startp: [*mut c_char; 10],\n"
            "    keys: [[*const c_char; 2]; 16],\n"
            "    lnum: LineNr,\n"
            "}\n"
            "pub union U {\n    text: *mut c_char,\n}\n"
            "struct T(*mut c_char);\n"
            "enum E {\n    Str(*mut *mut c_char),\n}\n"
            "fn f(name: *mut c_char) {\n    let p: *mut c_char = name;\n}\n",
        },
        {"raw_cstr_fields": 6},
    ),
    # The pointer forms count and the `_len` bodies underneath them do not.
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    utf_ptr2char(p);\n    utfc_ptr2len(p);\n"
            "    utfc_ptr2len_len(s, n);\n    utf_ptr2char_info(p);\n"
            "}\n",
        },
        {"mbyte_raw": 2},
    ),
    # A char-literal cast counts; the same text in a comment or a string does
    # not, and neither does an ordinary `as c_int` on a name.
    (
        {
            "crates/nvim/src/a.rs": "const NL: c_int = '\\n' as c_int;\n"
            "fn f() {\n"
            "    g(b'x' as libc::c_int);\n"
            "    h(k as c_int);\n"
            "    // 'z' as c_int\n"
            "    let s = \"'z' as c_int\";\n"
            "}\n",
        },
        {"char_as_c_int": 2},
    ),
    # ... and so does the same byte respelled `c_int::from(b'x')`; a
    # `u8::from`, a name, or the text in a comment does not.
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n"
            "    g(c_int::from(b'/'));\n"
            "    g(::core::ffi::c_int::from(b'\\n'));\n"
            "    g(c_int::from(k));\n"
            "    g(u32::from(b'x'));\n"
            "    // c_int::from(b'z')\n"
            "}\n",
        },
        {"char_as_c_int": 2},
    ),
    # A labelled block counts; a labelled loop (Rust's multi-level exit, see
    # LABELED_LOOPS) and a lifetime bound do not.
    (
        {
            "crates/nvim/src/a.rs": "fn f<'a: 'b>() {\n"
            "    'err: {\n        break 'err;\n    }\n"
            "    'outer: loop {\n        break 'outer;\n    }\n"
            "    'scan: while x {\n    }\n"
            "    'each: for y in z {\n    }\n"
            "}\n",
        },
        {"labeled_blocks": 1},
    ),
    # A constant duplicated across files counts once per copy past the first;
    # the same name with a *different* value is not a duplicate, and two
    # different string values must not be flattened into one by masking.
    (
        {
            "crates/nvim/src/a.rs": 'const NULL: c_int = 0;\nconst NAME: &str = "a";\n',
            "crates/nvim/src/b.rs": 'const NULL: c_int = 0;\nconst NAME: &str = "b";\n',
            "crates/nvim/src/c.rs": "const NULL: c_int = 0;\nconst OTHER: c_int = 1;\n",
        },
        {"dup_consts": 2},
    ),
    # A long function counts unless its file says a generator wrote it. The
    # marker has to be a module doc line naming a tool, so prose about
    # generated data does not buy the exemption.
    (
        {
            "crates/nvim/src/a.rs": "fn f() {\n" + "    g();\n" * 250 + "}\n",
            "crates/nvim/src/b.rs": "//! GENERATED by tools/apigen; do not edit.\n"
            "fn f() {\n" + "    g();\n" * 250 + "}\n",
            "crates/nvim/src/c.rs": "//! The tag file is generated by this code.\n"
            "fn f() {\n" + "    g();\n" * 250 + "}\n",
            "crates/nvim/src/d.rs": "fn f() {\n" + "    g();\n" * 10 + "}\n",
        },
        {"long_fns": 2},
    ),
    # One array is one site: a typed `let` with its initialiser, a bare
    # initialiser, a `vec!`, a parameter and a field each count once; a bare
    # `[0; N]` (no type in front), a byte array and a comment do not.
    (
        {
            "crates/nvim/src/a.rs": "fn f(p: &mut [c_char; 4]) {\n"
            "    let a: [c_char; N] = [0; N];\n"
            "    let b: [c_char; 8] = [0 as c_char; 8];\n"
            "    let c = [0 as c_char; MAXPATHL as usize];\n"
            "    let d = vec![0 as c_char; 9];\n"
            "    let e = [0; 3];\n"
            "    let g = [0u8; 3];\n"
            "    // let h: [c_char; 2] = [0 as c_char; 2];\n"
            "}\n"
            "struct S { x: [c_char; 5] }\n",
        },
        {"cchar_scratch": 6},
    ),
    # A `# Safety` section on a safe fn is stale; on an `unsafe fn`, however
    # its qualifiers are spelled, it is the contract.
    (
        {
            "crates/nvim/src/a.rs": "/// Does it.\n///\n/// # Safety\n/// None.\n"
            "pub(crate) fn f() {}\n"
            "/// # Safety\n/// `p` is live.\n#[inline]\npub const unsafe fn g(p: *mut u8) {}\n"
            '/// # Safety\n/// Real.\nunsafe extern "C" fn h() {}\n'
            "/// Plain.\nfn k() {}\n",
        },
        {"safety_on_safe_fns": 1},
    ),
    # A module with a `#[test]` anywhere beneath it is not untested; a
    # top-level *file* is not a directory and is not asked.
    (
        {
            "crates/nvim/src/lonely/mod.rs": "fn f() {\n}\n",
            "crates/nvim/src/lonely/more.rs": "fn g() {\n}\n",
            "crates/nvim/src/covered/mod.rs": "#[cfg(test)]\nmod tests {\n"
            "    #[test]\n    fn t() {}\n}\n",
            "crates/nvim/src/alone.rs": "fn h() {\n}\n",
            "crates/nvim/src/bin/nvim.rs": "fn main() {\n}\n",
        },
        {"untested_dirs": 1},
    ),
    # ... and a module file beside its directory is beneath it.
    (
        {
            "crates/nvim/src/split/inner.rs": "fn f() {\n}\n",
            "crates/nvim/src/split.rs": "#[cfg(test)]\nmod tests {\n"
            "    #[test]\n    fn t() {}\n}\n",
        },
        {"untested_dirs": 0},
    ),
    # A message then a sentinel, as a `return` or as a tail; a message that
    # bookkeeping follows first, an `Err` after one, a sentinel that follows
    # no message, and a format string holding a `;` are not.
    (
        {
            "crates/nvim/src/a.rs": "fn f() -> bool {\n    emsg(e_x);\n    return false;\n}\n"
            'fn g() -> Option<u8> {\n    if x {\n        semsg!("E1: {};", a);\n        None\n    } else {\n        Some(1)\n    }\n}\n'
            "fn h() -> c_int {\n    emsg(e_x);\n    did = true;\n    return -1;\n}\n"
            "fn k() -> Result<(), Failed> {\n    emsg(e_x);\n    return Err(Failed);\n}\n"
            "fn m() -> c_int {\n    let n = f(g(1));\n    0\n}\n"
            'fn n() -> c_int {\n    let v = match k {\n        _ => {\n            siemsg!("bad", k);\n            0\n        }\n    };\n    v\n}\n',
        },
        {"emit_sentinel": 3},
    ),
    # A local wrapper (`err*`/`emsg_*`, answering nothing, calling an emitter)
    # is an emitter too; a method, a builder that answers a value, and a call
    # through `.` are not.
    (
        {
            "crates/nvim/src/a.rs": "fn err0(msg: &CStr) {\n    emsg(msg);\n}\n"
            "fn f() -> bool {\n    err0(e_x);\n    return false;\n}\n"
            "fn err_msg(m: &CStr) -> Option<CString> {\n    emsg(m);\n    None\n}\n",
            "crates/nvim/src/b.rs": "fn g() -> c_int {\n    super::err0(e_x);\n    -1\n}\n"
            "fn error(&mut self) {\n    emsg(e_x);\n}\n"
            "fn h() -> bool {\n    self.error();\n    false\n}\n"
            "fn k() -> bool {\n    p.err0(e_x);\n    false\n}\n",
        },
        {"emit_sentinel": 3},
    ),
    # An emitter nested in another's arguments is one message, not two.
    (
        {
            "crates/nvim/src/a.rs": 'fn f() -> bool {\n    semsg!("E1: {}", msg_cstr(p));\n    false\n}\n',
        },
        {"emit_sentinel": 1},
    ),
    # A formatter in a log line's arguments is no message.
    (
        {
            "crates/nvim/src/a.rs": 'fn f() -> bool {\n    logmsg!(L, c"f", 1, "{}", msg_bytes(a));\n    return false;\n}\n',
        },
        {},
    ),
    # `types/` is counted by the file.
    (
        {
            "crates/nvim/src/types/eval.rs": "pub struct A;\n",
            "crates/nvim/src/types/uv.rs": "pub struct B;\n",
        },
        {"types_files": 2},
    ),
]


# Placement, case by case: ({repo-relative path: source}, {path: number of
# broken rules}). Files absent from the expectation break none.
_ROW = "#![deny(unsafe_op_in_unsafe_fn)]\n// Unsafe perimeter: the `os/` row in docs/perimeter.md.\n#![allow(unsafe_code)]\n"
# A unit file's stem -> the top-level module it drives, given these modules.
SELF_TEST_UNIT_COVERS_TOPS = {"channel", "ui", "ui_compositor", "eval", "option"}
SELF_TEST_UNIT_COVERS = [
    ("channel", "channel"),
    ("channel_reader", "channel"),
    ("ui_compositor", "ui_compositor"),
    ("ui_compositor_scratch", "ui_compositor"),
    ("ui_state", "ui"),
    ("eval_decode", "eval"),
    ("options_oracle", None),
    ("typval_dict", None),
]
# A keeper row names a site (by type, or a whole file) and opens with a class;
# a row naming nothing, or with no class, is a finding. An unlisted site is not.
SELF_TEST_KEEPERS = [
    (
        {"a.rs::S": "value", "b.rs": "ffi: c", "c.rs::Gone": "value", "a.rs::T": "big"},
        [("a.rs", "S"), ("a.rs", "T"), ("b.rs", "U"), ("b.rs", "V"), ("d.rs", "W")],
        [
            "c.rs::Gone: a COPY_KEEPERS row that excuses nothing",
            "a.rs::T: a COPY_KEEPERS row whose reason does not open with one of "
            "ffi, handle, value",
        ],
    ),
]
# A labelled loop is listed by `labeled_loops`; a labelled block is not.
SELF_TEST_LABELED_LOOPS = [
    (
        "fn f() {\n    'err: {\n    }\n    'outer: loop {\n    }\n"
        "    'scan: while x {\n    }\n    'each: for y in z {\n    }\n}\n",
        3,
    ),
]

# A `Failed` made in a body that says nothing and passes nothing on; one
# after a message, one passed on with `?`/`map_err`, a pattern, a
# comparison and test code are not.
SELF_TEST_SILENT_FAILED = [
    (
        "fn a() -> Result<(), Failed> {\n    if x {\n        return Err(Failed);\n    }\n"
        "    y.ok_or(Failed)\n}\n"
        "fn b() -> Result<(), Failed> {\n    emsg(e_x);\n    Err(Failed)\n}\n"
        "fn c() -> Result<(), Failed> {\n    d()?;\n    Err(Failed)\n}\n"
        "fn e() -> bool {\n    match f() {\n        Err(Failed) => true,\n"
        "        _ => g() == Err(Failed),\n    }\n}\n"
        "#[cfg(test)]\nmod tests {\n    fn h() -> Result<(), Failed> {\n"
        "        Err(Failed)\n    }\n}\n",
        2,
    ),
]

SELF_TEST_PLACEMENT = [
    # A perimeter file naming its row, and a finished file beside it.
    (
        {
            "crates/nvim/src/os/a.rs": _ROW,
            "crates/nvim/src/b.rs": "#![forbid(unsafe_code)]\n",
        },
        {},
    ),
    # A perimeter file may forbid; an allow off the perimeter may not stay.
    (
        {
            "crates/nvim/src/os/a.rs": "#![forbid(unsafe_code)]\n",
            "crates/nvim/src/b.rs": "#![deny(unsafe_op_in_unsafe_fn)]\n#![allow(unsafe_code)]\n",
        },
        {"crates/nvim/src/b.rs": 1},
    ),
    # A perimeter allow without the deny and without its row: two rules. A
    # row comment naming another entry does not count, and neither does a
    # doc comment.
    (
        {
            "crates/nvim/src/os/a.rs": "#![allow(unsafe_code)]\n",
            "crates/nvim/src/os/b.rs": _ROW.replace("`os/`", "`tui/`"),
            "crates/nvim/src/os/c.rs": "#![deny(unsafe_op_in_unsafe_fn)]\n//! Unsafe perimeter: the `os/` row.\n#![allow(unsafe_code)]\n",
        },
        {
            "crates/nvim/src/os/a.rs": 2,
            "crates/nvim/src/os/b.rs": 1,
            "crates/nvim/src/os/c.rs": 1,
        },
    ),
    # A brace group in the row's spelling expands, and the comment may wrap.
    (
        {
            "crates/nvim/src/lua/executor/luv.rs": "#![deny(unsafe_op_in_unsafe_fn)]\n// Unsafe perimeter: the `lua/ffi.rs`,\n// `lua/executor/{luv,state}.rs` row in docs/perimeter.md.\n#![allow(unsafe_code)]\n",
        },
        {},
    ),
    # A module file over an allowing descendant may say neither (`mod.rs`,
    # `lib.rs`, `x.rs` beside `x/`); without one it has to forbid.
    (
        {
            "crates/nvim/src/lib.rs": "",
            "crates/nvim/src/os/mod.rs": "",
            "crates/nvim/src/os/a.rs": _ROW,
            "crates/nvim/src/winlayer.rs": "",
            "crates/nvim/src/winlayer/a.rs": _ROW.replace("`os/`", "`winlayer/`"),
            "crates/nvim/src/x/mod.rs": "",
            "crates/nvim/src/x/y.rs": "#![forbid(unsafe_code)]\n",
            "crates/nvim/src/z.rs": "",
        },
        {"crates/nvim/src/x/mod.rs": 1, "crates/nvim/src/z.rs": 1},
    ),
    # Outside the crate's source nothing is asked.
    ({"crates/other/src/lib.rs": "#![allow(unsafe_code)]\n"}, {}),
]


def self_test():
    for source, expected in SELF_TEST:
        got = unsafe_stmts(mask(source), False)
        assert got == expected, f"unsafe_stmts={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_DENY:
        got = unsafe_stmts(mask(source), True)
        assert got == expected, f"unsafe_stmts={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_SAFETY_DOC:
        got = missing_safety_doc(source, mask(source))
        assert got == expected, (
            f"missing_safety_doc={got}, want {expected}, for {source!r}"
        )
    for source, expected in SELF_TEST_DENY_CASTS:
        got = DENY_CASTS.search(mask(source)) is not None
        assert got == expected, f"cast deny={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_FILE_ALLOWS:
        masked = mask(source)
        got = {lint for lint, needle in FILE_ALLOWS.items() if needle in masked}
        assert got == expected, (
            f"file allows={sorted(got)}, want {sorted(expected)}, for {source!r}"
        )
    for stem, expected in SELF_TEST_UNIT_COVERS:
        got = unit_covers(stem, SELF_TEST_UNIT_COVERS_TOPS)
        assert got == expected, f"unit_covers({stem!r})={got}, want {expected}"
    for floor, sites, expected in SELF_TEST_KEEPERS:
        got = keeper_findings(floor, "COPY_KEEPERS", sites)
        assert got == expected, f"keeper_findings={got}, want {expected}"
    for source, expected in SELF_TEST_SILENT_FAILED:
        got = silent_failed(mask(source))
        assert got == expected, f"silent_failed={got}, want {expected}, for {source!r}"
    for source, expected in SELF_TEST_LABELED_LOOPS:
        got = len(LABELED_LOOPS.findall(mask(source)))
        assert got == expected, f"labeled_loops={got}, want {expected}, for {source!r}"
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
    for source, expected in SELF_TEST_SELF_PROJECTION:
        got = len(self_projections({"t.rs": mask(source)}))
        assert got == expected, (
            f"self_projections={got}, want {expected}, for {source!r}"
        )
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
    for source, expected in SELF_TEST_WRAPPERS:
        masked = mask(source)
        got = [
            wrapper_callee(masked[at + 1 : matching_brace(masked, at)])
            for at in (
                WHITESPACE.match(masked, m.end()).end()
                for m in UNSAFE_WORD.finditer(masked)
            )
            if at < len(masked) and masked[at] == "{"
        ]
        got = [callee for callee in got if callee]
        assert got == expected, f"wrappers={got}, want {expected}, for {source!r}"
    for source, allow, expected in SELF_TEST_NO_RAW:
        rows = {name: AllowRow({file}, "cap: a fixture") for name, file in allow}
        got = unsafe_fns_without_raw(mask(source), "t.rs", rows)
        assert got == expected, (
            f"unsafe_fns_without_raw_params={got}, want {expected}, "
            f"for {source!r} with {allow!r}"
        )
    for sources, expected in SELF_TEST_RAW_ALIAS:
        tree = {f: mask(text) for f, text in sources.items()}
        aliases = raw_aliases(tree)
        got = sum(
            unsafe_fns_without_raw(masked, f, {}, aliases) for f, masked in tree.items()
        )
        assert got == expected, (
            f"unsafe_fns_without_raw_params={got}, want {expected}, for {sources!r}"
        )
    for sources, allow, expected in SELF_TEST_NO_RAW_CHECK:
        index = unsafe_fn_index({f: mask(text) for f, text in sources.items()})
        try:
            check_unsafe_fn_allowlist(
                index,
                {name: AllowRow({file}, "cap: a fixture") for name, file in allow},
            )
            got = False
        except SystemExit:
            got = True
        assert got == expected, (
            f"check_unsafe_fn_allowlist rejected={got}, want {expected}, "
            f"for {sources!r} with {allow!r}"
        )
    for sources, expected in SELF_TEST_INSTRUMENTS:
        sites = instrument_sites(
            {f: mask(text) for f, text in sources.items()}, sources
        )
        for name, want in expected.items():
            got = sum(sites[name].values())
            assert got == want, f"{name}={got}, want {want}, for {sources!r}"
    for sources, expected in SELF_TEST_VOCABULARY:
        got = vocabulary({f: mask(text) for f, text in sources.items()})
        for name, want in expected.items():
            assert got[name] == want, (
                f"{name}={got[name]}, want {want}, for {sources!r}"
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
    for files, expected in SELF_TEST_PLACEMENT:
        tree = {file: mask(source) for file, source in files.items()}
        got = {
            file: len(broken)
            for file, broken in placement_findings(tree, files).items()
        }
        assert got == expected, f"placement={got}, want {expected}, for {files}"
    assert expand_braces("a/{b,c}.rs") == ["a/b.rs", "a/c.rs"]
    for case, expected in SELF_TEST_PERIMETER_CHECK:
        stats = {
            (entry + "x.rs" if entry.endswith("/") else entry): {
                "unsafe_stmts": 0 if case == "zero" and entry == probe else 1
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


def breakdown(sites, name):
    """Print one instrument's per-file (or per-name) counts, largest first.

    The listing mode exists so a slice can triage without a second script:
    the total says how much debt there is, this says which files hold it, and
    the directory roll-up under it says which family to take next.
    """
    if name not in sites:
        sys.exit(
            f"ratchet: no such dimension: {name}\nKnown dimensions:\n  "
            + "\n  ".join(INSTRUMENT_KEYS)
        )
    counter = sites[name]
    print(f"{name}: {sum(counter.values())} across {len(counter)} units")
    for unit, count in counter.most_common():
        print(f"  {count:6}  {unit}")
    directories = collections.Counter()
    for unit, count in counter.items():
        directories[unit.rsplit("/", 1)[0] if "/" in unit else unit] += count
    if len(directories) < len(counter):
        print("by directory:")
        for directory, count in directories.most_common():
            print(f"  {count:6}  {directory}")


def placement_report(tree, sources):
    """`--dimension unsafe_placement`: each offender, what it breaks, and the
    two ways out. The count is the number of broken rules, so it matches the
    ratcheted total."""
    found = placement_findings(tree, sources)
    total = sum(len(broken) for broken in found.values())
    print(f"unsafe_placement: {total} across {len(found)} files")
    for file, broken in sorted(found.items()):
        for rule in broken:
            print(f"  {file}: {rule}")
    if found:
        print(f"Each: {PLACEMENT_EXITS}.")


def main():
    argv = sys.argv[1:]
    dimension = None
    if "--dimension" in argv:
        at = argv.index("--dimension")
        if at + 1 == len(argv):
            sys.exit("ratchet: --dimension wants a name")
        dimension = argv[at + 1]
        argv = argv[:at] + argv[at + 2 :]
    args = set(argv)
    if unknown := args - {"--check", "--allow-growth"}:
        sys.exit(f"ratchet: unknown argument(s): {' '.join(sorted(unknown))}")

    self_test()
    stats, without_casts, allowing, tree, sources = measure()
    wrappers, by_callee = wrapper_scan(tree)
    sites = instrument_sites(tree, sources, wrappers)
    if dimension == "unsafe_placement":
        placement_report(tree, sources)
        return
    if dimension == "files_without_deny_casts":
        check_cast_deny_floor(tree, report=True)
        return
    if dimension == "copy_keepers":
        keeper_report(COPY_KEEPERS, "derive_copy", copy_sites(tree))
        return
    if dimension == "repr_c_floor":
        keeper_report(REPR_C_FLOOR, "repr(C) off the perimeter", repr_c_sites(tree))
        return
    if dimension == "labeled_loops":
        breakdown({dimension: labeled_loops(tree)}, dimension)
        return
    if dimension == "silent_failed":
        breakdown({dimension: silent_failed_sites(tree)}, dimension)
        return
    if dimension is not None:
        breakdown(sites, dimension)
        return
    check_place_writes(tree)
    check_borrowed_derefs(tree)
    check_deref_temporary_mutations(tree)
    check_self_projections(tree)
    check_temporary_addresses(tree)
    check_deref_field_writes(tree)
    check_cell_ptr(tree)
    check_names(tree)
    check_perimeter(stats)
    check_unsafe_placement(tree, sources)
    check_cast_deny_floor(tree)
    check_keeper_floors(tree)
    index = unsafe_fn_index(tree)
    check_unsafe_fn_allowlist(index, unsafe_fn_allowlist())
    sync_perimeter_doc(stats, "--check" in args)
    sync_wrapper_table(wrapper_table(by_callee, index), "--check" in args)
    counts = {**ledgers(), **whole_tree(stats, tree, sites)}
    content = render(stats, counts, without_casts, allowing)
    committed = BASELINE.read_text() if BASELINE.exists() else None

    if "--check" in args:
        if committed is None:
            sys.exit(
                f"ratchet: {BASELINE.relative_to(ROOT)} is missing; run `just refresh`"
            )
        if grew := violations(
            stats,
            counts,
            without_casts,
            allowing,
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
            without_casts,
            allowing,
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
        f"{summary(stats, counts, without_casts, allowing)}"
    )


if __name__ == "__main__":
    main()
