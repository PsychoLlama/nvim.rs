Plan: ~/.claude/plans/groovy-snuggling-clover.md

All phases complete. Here's a summary:

## Completed

**Phase 0** — Fixed 7 wrong `DG_START_*` constants in Rust, added `_Static_assert` for all 26 in C.

**Phase 1** — Moved `digraphdefault[]` (~1366 entries) from C to Rust `data.rs`. Script-generated from C source. Rust callers now access `DIGRAPH_DEFAULT` directly as a slice. C callers use `rs_get_digraphdefault()`/`rs_get_digraphdefault_len()` FFI. Removed ~1430 lines from `digraph.c`.

**Phase 2** — Migrated `digraph_header()`, `printdigraph()`, and `listdigraphs()` to Rust. Moved `header_table[]` (26 translatable strings) to Rust. `ex_digraphs()` now calls `rs_listdigraphs()` directly.

**Phase 3** — Migrated `get_digraph()` (Ctrl-K input) to Rust. Added 6 C accessor functions for globals (`no_mapping`, `allow_keys`, `cmdline_star`, `plain_vgetc`, `putcmdline`, `add_to_showcmd`).

**Phases 4 & 5** — Deferred as planned (VimL eval functions and keymap functions stay in C).

### Validation
- `just build` — passes
- `just smoke-test` — passes
- `just rust-fmt-check` — passes
- `just rust-clippy` — passes
- `just rust-test` — 4155 tests pass
- `digraph.c` reduced from ~2228 to 703 lines
- 26 `#[no_mangle]` FFI exports in the digraph crate
