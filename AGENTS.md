# Project

- A fork of neovim ported to Rust. The port was bootstrapped with `c2rust`.
- Original C code exists under git tag `v0.12.4`.
- No backward/forward compat guarantees. Breaking changes allowed.

# Developing

- Tests and builds are verbose; redirect to a log file and grep if failed.
- Pre-commit hooks (`$REPO/.gitconfig`) validate formatting, `abi-ledger`, and ratcheted metrics.
- Note big changes in `CHANGELOG.md`. It should not be exhaustive: only a hint about which release carried a sweeping change over a large feature. Small-to-mid size changes to not belong in the changelog.
- Transpilation inherited original authors' terse naming. Don't match for parity's sake. When appropriate, use clear names.
- When rewriting vendored code, keep the upstream copyright/license notice in the ported file and keep `LICENSE.txt` accurate.
- A panic ends the process (swap files preserved, then `abort()`): `expect` an invariant with a message naming it, return an error for anything input reaches. See [docs/panic-policy.md](docs/panic-policy.md).

# Ratchet

- The ratchet (`just ratchet --check`) constrains per-file unchecked statements (`unsafe_stmts` — code inside `unsafe` regions), `#[no_mangle]`/C-ABI-signature counts, `GlobalCell` raw escape-hatch uses (`.ptr()`/`.as_raw()`), `unsafe fn`s with no `# Safety` section, file line counts, and internal exports to only shrink — plus the count of files allowing the cast lints — and fails outright on an unsafe-code attribute in the wrong place (`unsafe_placement`, below).
- `unsafe_code` is **denied** in `crates/nvim/Cargo.toml`, so a file needing `unsafe` carries an inner `#![allow(unsafe_code)]` — with its `docs/perimeter.md` row named in the comment above it — and only a file on the perimeter may carry it. Every other file carries `forbid(unsafe_code)`. Generated files take the attribute from `tools/apigen`, never by hand.
- An `unsafe fn` body scores as unchecked _in full_ until its file carries `deny(unsafe_op_in_unsafe_fn)`. Land the deny first to see a file's true count, and take a `mod.rs` last — a lint attribute there reaches the whole subtree.
- Clippy's cast family (`cast_lossless`, `cast_possible_truncation`, `cast_possible_wrap`, `cast_sign_loss`, `ptr_as_ptr`) is `deny` in every crate's `[lints.clippy]`, tests included: widen with `From`, narrow with `try_from` or a `narrow.rs` helper. A file that still casts carries an inner `#![allow(...)]` of the five with its reason in a comment above, and is a row of `CAST_ALLOW_FLOOR` in `scripts/ratchet.py` (perimeter files, and `narrow.rs` — the cast home — for good); `files_allowing_casts` may only fall, and `--dimension files_allowing_casts` prints it. It is a hard check (`check_cast_attributes`, `crates/*/tests` too): an allow off the floor fails, a row that excuses nothing fails, and so does a per-file `#![deny(...)]` of them — redundant — except in a file below an allowing module root, which must carry it.
- `internal_exports` is the abi-ledger's own number: exports nobody outside the
  crate resolves. It is 0, and [docs/abi-ledger.md](docs/abi-ledger.md) says why
  each of the 46 remaining `test`-class rows is still there.
- Unsafe code lives only on the `PERIMETER` list (the FFI/OS/raw-memory modules where unsafe legitimately bottoms out). See [docs/perimeter.md](docs/perimeter.md) for what qualifies and how the list prunes itself, and [docs/unsafe-audit.md](docs/unsafe-audit.md) for what each row holds, why it cannot be otherwise, and what would retire it.
- `unsafe_placement` is a hard check: `just ratchet --check` (so `minimal-ci` and the pre-commit hook) fails on an `allow(unsafe_code)` off the `PERIMETER` (generated files included), a file with neither the allow nor `forbid(unsafe_code)` that is not a `mod.rs`/`lib.rs`/`x.rs` over an allowing descendant, and an allow file missing `deny(unsafe_op_in_unsafe_fn)` or its row named in the comment above the attribute. The message names each file and its two exits (finish it, or a `PERIMETER` row with a reason); `--dimension unsafe_placement` prints the same list.
- A region costs its **statements**, at every depth: every `;`-terminated statement, every block's tail expression, every `match` arm, minimum one. Splitting `unsafe { a; b; c }` into three regions is not progress — deleting an operation is. Layout is free, so a rustfmt-wrapped call is one statement and a SAFETY comment costs nothing.
- `wrappers` counts single-call `unsafe { f(..) }` regions whatever their line count; `metrics/wrapper-callees.tsv` is the per-callee breakdown — its `no_raw` column is the wrappers a slice deletes when the callee drops the keyword.
- `unsafe_fns_without_raw_params` counts `unsafe fn`s whose signature (parameters and return) shows no raw pointer and no `CPtr` bound, that are not `extern "C"`, and that are not a row in [docs/unsafe-fn-allowlist.md](docs/unsafe-fn-allowlist.md). That list is the floor: every row opens with a reason class (`alloc:`/`cap:`/`ffi:`/`field:`), names the file that defines its item, and a row that excuses nothing (or names the wrong file) fails the run.
- `char_as_c_int` counts a character literal widened into C's `int`, as `b'x' as c_int` or `c_int::from(b'x')` alike: the second is the cast lints' spelling of the same byte, not a change of domain (a byte compares as `u8`, a key is `Key`).
- `cchar_scratch` counts fixed-size C string buffers: a `[c_char; N]` type anywhere (local, parameter, return, field) or a `[0 as c_char; N]` initialiser, `vec!` included; a typed `let` and its initialiser are one site, comments are not counted. `--dimension cchar_scratch` lists where they sit.
- `emit_sentinel` counts a message call (`emsg`, `semsg!`, `iemsg`, `smsg!`, `give_warning`, `msg_puts*`, `api_error!`, … — `EMIT_CALL` in `scripts/ratchet.py` — or a local wrapper of one: a free `fn err*`/`emsg_*` that answers nothing and calls an emitter) whose statement is followed directly by a `return`/tail of `false`, `None`, `-1`, `0` or `FAIL`. After a message the answer is `Err(Failed)` (or a family's own error): `Failed` means "a message is already out", Vim's `emsg` channel being the boundary that reports it, so `Err(..)` after a message is not counted. `--dimension emit_sentinel` lists where they sit.
- Writes into an accessor's by-value answer are hard checks: `place_writes` (`f().field = v`, `slot_fields!` getters included) and `accessor_temporary_mutations` (`&mut x.getter()`, `x.getter().take()` on a `slot_fields!` getter). A flag word changes in place only through `|=`/`-=`/`^=`, which refuse a temporary at compile time.
- `container_callouts` is a hard check: an `impl Dict`/`List`/`Blob` method taking `&mut self` may not call a message or anything that reaches user code or variables (`CONTAINER_CALLOUT` in `scripts/ratchet.py`); run the check at the handle before the edit.
- `clippy::significant_drop_tightening` is `deny` (`just lint`): a guard marked `#[clippy::has_significant_drop]` (`LineText`, `SlotRead`/`SlotEdit`/`SlotPart`, a value's `Read`/`Edit`) is `drop`ped after its last use, a tail use as `let out = …; drop(guard); out`. The lint sees no use through a borrow and no last use inside an `if`/`match`, so let the borrow checker place the `drop`, not the lint's suggestion.
- `labeled_blocks` counts `'label: {` blocks only — c2rust's forward `goto`. A labelled `loop`/`while`/`for` is Rust's multi-level `break`/`continue`, not a goto: `--dimension labeled_loops` lists them, unratcheted.
- `untested_dirs` counts top-level modules with no `#[test]` beneath them and no unit file `crates/nvim/tests/unit/<mod>.rs`, `<mod>/` or `<mod>_<what>.rs` (the longest module name wins, so `ui_compositor.rs` is not `ui`'s).
- `COPY_KEEPERS` and `REPR_C_FLOOR` in `scripts/ratchet.py` are the floors `derive_copy` and `repr(C)` (`repr_c_editor_state` + `repr_c_ffi_types`) fall to: rows `path::Type` (or a whole `path`) whose reason opens with a class (`ffi`/`handle`/`value`; `abi`/`libuv`/`codec`/`ffi`/`layout`). A row that excuses nothing fails the run; a site off the list is still counted, not refused — `--dimension copy_keepers`/`repr_c_floor` list it as OFF THE FLOOR. A field bag (six or more fields) keeps `Copy`, and a type keeps `repr(C)` for its field order (`layout`), only with a measured hot path in its row.
- Lines inside a `#[cfg(test)] mod … { … }` are exempt from the 1,000-line file cap, so tests can sit next to the code they cover. Every other metric still counts them — unchecked code in a test is still unchecked code.
- `metrics/clippy.json`'s per-file warning counts cover `crates/*/tests` as well as `crates/*/src` (clippy runs `--all-targets`), so a new warning in a _test_ file fails `just lint`. Only its two posture totals (`unreachable_pub`, `unused_qualifications`) are restricted to `crates/*/src`, and `metrics/ratchet.json` never looks at `tests/` at all.
- After reducing any of them, run `just refresh` and commit the refreshed `metrics/*.{json,jsonl}`.
- Justified growth requires `just refresh --allow-growth`.

# Generators

- `tools/apigen` and `tools/ffigen` live outside the workspace; `just lint-tools` clippies them at `-D warnings` (baseline: zero) and `just fmt-check` already reaches them.
- Never hand-edit generated output. `just apigen`/`just ffigen` rewrite it; the `--check` form of each fails on drift and runs in `just minimal-ci`.
- `tools/ffigen/unit-cdefs.h` is a committed golden, not an input: the unit suite regenerates its own copy under `target/ffi`.

# Sweeps

- A tree-wide mechanical rewrite is a throwaway script in a scratchpad, not a
  committed tool. `scripts/xform.py` is the library those scripts import: call
  spans matched against masked text, byte-offset edits, `c"…"`/`b"…"` literals.
  Read its docstring before writing a sweep — every piece of it is a mistake a
  previous sweep shipped.
- Nothing runs it, so it has no `--check`; `treefmt` formats it with `ruff`.

# Testing

Tests only run local, not in CI. Any failure, even out of scope, is your responsibility to fix.

- **Lanes.** Every change runs `just minimal-ci`, `just functionaltest` and `just oldtest all` (the routine chain; on an idle machine with `--jobs=12` on both suites, ~6 min against ~25 serial). The expensive lanes run once, at the close of a piece of work, and only for what it touched; work that touched none of these says so in its report and skips them:
  - `just asan functionaltest` + `just asan oldtest all` (`--jobs=N` as below; 12 workers: ~13 + ~6 min against ~50 + ~25 serial) — a `PERIMETER` file (the handle modules `winlayer/` and `eval/typval/handle.rs` among them) or the pinned C dependencies. A graph handle that outlives its object panics on its next dereference (slot generation), so handle lifetimes elsewhere are the routine chain's to find.
  - `just miri-perimeter` — a raw-memory perimeter file: one under a `memory` row of `docs/unsafe-audit.md` (`MIRI_PERIMETER_FILES` in `scripts/ratchet.py`).
  - `just battery [--jobs=N]` — work that retypes or restructures without meaning to change behaviour (not work whose point is a behaviour change). `--jobs=N` runs the rows N at a time, longest first, then its `SERIAL_TAIL` alone (12 cores, warm cache: ~6 min against ~13 serial).
  - `just miri-lib`/`just miri` are on no routine chain.
- `just cargo-unit [filter...]` — `crates/nvim/tests/unit` under cargo-nextest (part of `minimal-ci`): a process per case, all cores, so a case sets up the editor state it needs rather than inheriting an earlier case's, and a prompt that would wait for a key panics. Lib tests and doctests stay on `cargo test`.
- `just functionaltest [paths...] [--filter=pat]` — RPC-driven tests from `test/functional`. Pass spec files/dirs to run a subset.
  - `--jobs=N` (or `NVIM_TEST_JOBS=N`) shards it by spec file over N workers with GNU `parallel`, longest first by the timing table; opt-in, the caller picks N (12 cores: ~2 min against ~14 serial, on an idle machine). One merged report names each failing spec's log (`<run root>/logs/`) and kept root (`<run root>/failed/`). `SERIAL_TAIL` in `scripts/run-tests.sh` lists specs that run serially after the parallel part, each with its reason; a test that fails only under load is fixed first (wait on a condition, not a clock). A worker's root is `<run root>/w<N>/`, so a spec must not depend on the length of its cwd. A sharded spec's screen waits get three times the usual maximum (`NVIM_TEST_SHARDED=1`, `test/functional/ui/screen.lua`). A spec that runs past `NVIM_TEST_SPEC_TIMEOUT` seconds (default 900) is killed and fails: a failed screen wait that leaves nvim at a prompt otherwise hangs its `after_each` for good.
- `just oldtest <all|test_name...|clean>` — legacy Vim tests from `test/old/testdir`. The mode is required; `all` runs the whole suite from scratch, names run a subset, e.g. `just oldtest test_arglist`.
  - `--jobs=N` (or `NVIM_TEST_JOBS=N`) runs the test files N at a time the same way (12 cores: ~50 s against ~330 serial, idle); the report is upstream's `make report` over every file's `messages`, then each failing file's log (`<run root>/logs/`). `SERIAL_TAIL` in `scripts/run-oldtest.sh` is the same list as functional's. A run root is a checkout of its own (`test/old`, a `runtime/` copy, `build/`, `xdg/`) and each worker gets one (`<run root>/w<N>/`): oldtest writes nothing outside `target/`.
- `just unittest [paths...]` — unit tests from `test/unit` (part of `minimal-ci`, ~4 s). Their FFI declarations are generated from the Rust crate (`tools/ffigen`, wired in by `scripts/gen-unit-cdefs.sh`), rooted at the identifiers the specs, `testutil.lua` and the C fixtures spell, so only what they reach is declared; the specs call the exported symbols of the nvim binary, and the C fixtures (`unit-fixtures.so`) compile against the same generated declarations.
- Different suites may run beside each other (functional, oldtest, unittest, battery, and an ASan suite beside a plain one): each run has its own root, and `scripts/prep-test-tree.sh` (fixtures, `bin/nvim` link, `runtime/doc/tags`) runs once under a lock. Running the same suite twice at once needs distinct roots: `NVIM_TEST_RUN_DIR` (functional/unit/benchmark; default `<build>/testrun/<suite>`), `NVIM_OLDTEST_RUN_DIR` (default `target/oldtest`). `NVIM_TEST_BUILD_DIR` (default `target`) is the `bin/`/`runtime/` tree; the ASan recipes use `target/asan/tree`. `NVIM_TEST_SKIP_PREP=1` skips the prep for a caller that already ran it. A battery takes a work root slot (`/tmp/nvb<N>`, `test/battery/tmproot.sh`); rows mask the slot out of their artifacts (`slotscrub.py`), so every slot and a second battery beside a first share one cut cache.
- Each functional/oldtest run writes a per-file timing table (`seconds  tests  file`, slowest first): `timings.tsv` in its run root, folded into `<build>/timings/<suite>.tsv`.
- `just miri-perimeter` — Miri over the lib tests of the raw-memory perimeter files, each test module named by its whole path (libtest matches a filter anywhere in a name); minutes, not `just miri-lib`'s hours. Each raw-memory file keeps a `#[cfg(test)] mod tests` the recipe names, exercising its unsafe operations through the safe API (`just ratchet --check` fails otherwise, and on a filter that names no module or matches another's).
- `just asan <recipe>` — AddressSanitizer equivalents (`build`, `functionaltest`, `oldtest`, `leaktest`), defined in `just/asan.just`, with the same arguments as the plain suites (`--jobs=N` included) and a build tree of their own (`target/asan/tree`, `target/oldtest-asan`). Reports land in `target/asan/logs/<suite>/asan.log.<pid>`, emptied when the suite starts; any report fails the run, and `scripts/asan-reports.sh` names each one.
- `just benchmark [paths...]` — benchmarks from `test/benchmark`. It depends on
  `just build`, so the binary it times is the **dev profile** (only `nvim-mainthread`
  is optimized); read the numbers as a dev-profile comparison, and build
  `--release` with `codegen-units = 1` by hand for anything that has to hold
  against a shipped binary.
- `just drop-glue [binary]` — fails if `drop_in_place::<TypVal>` or
  `drop_in_place::<Object>` in a **release** binary has a cleanup path. Both
  types release themselves, so their payloads are `ManuallyDrop` and the
  compiler must append nothing; a payload it drops instead gives the shim a
  landing pad, which stops it being a tail call and stops it inlining at the
  hundreds of sites that drop a value. Worth ~0.5 % on four of the five
  benches, and neither the source nor the suites show it. Run it whenever a
  value type gains a field.
- `just state-exports [binary]` — fails if a state record's cell (`state_record!`, `OPTIONS`) or a hot cell is an exported `D`/`B` symbol in a **release** binary, i.e. read through the GOT, or if any other `GlobalCell` is exported without a row in the shrink-only [docs/exported-cells.md](docs/exported-cells.md); run it (default and `codegen-units = 1`) after changing a cell's or its accessors' visibility.
