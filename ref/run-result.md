Plan: ~/.claude/plans/nested-frolicking-meadow.md

All 4 phases of the `context.c` → Rust migration are complete. Here's the summary:

**Commits (4 total):**
1. Phase 1: FFI foundation + `rs_ctx_free_all` (stack accessors, _Static_assert)
2. Phase 2: `rs_ctx_save` (shada encoding, `nvim_ctx_save_funcs` C accessor)
3. Phase 3: `rs_ctx_restore` (shada reading, option save/restore, `nvim_ctx_restore_funcs`)
4. Phase 4: `rs_ctx_to_dict` + `rs_ctx_from_dict` (thin wrappers over C accessors)

**Rust files created:**
- `src/nvim-rs/context/src/ffi.rs` — all FFI declarations (stack, shada, option, dict)
- `src/nvim-rs/context/src/stack.rs` — `rs_ctx_free_all`
- `src/nvim-rs/context/src/save.rs` — `rs_ctx_save`
- `src/nvim-rs/context/src/restore.rs` — `rs_ctx_restore`
- `src/nvim-rs/context/src/convert.rs` — `rs_ctx_to_dict`, `rs_ctx_from_dict`

**Result:** `context.c` is now reduced to statics (`ctx_stack`, `kCtxAll`, `saved_shada_opt`) + C accessor functions (stack ops, shada opt save/restore, function save/restore, dict conversion) + thin `rs_*` forwarding wrappers. All public functions delegate to Rust. Every phase passes `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, and `just rust-test` (4106 tests).
