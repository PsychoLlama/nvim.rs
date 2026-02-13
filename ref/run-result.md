Plan: ~/.claude/plans/nested-frolicking-meadow.md

All 5 phases of the `context.c` → Rust migration are complete. Here's the summary:

**5 commits on `rust-migration`:**

| Commit | Description |
|--------|-------------|
| `ccf0ff3` | Phase 1: FFI foundation + `rs_ctx_free_all` |
| `9011c4c` | Phase 2: `rs_ctx_save` (shada encoding, function saving) |
| `2d42011` | Phase 3: `rs_ctx_restore` (shada reading, option save/restore) |
| `189a9ad` | Phase 4: `rs_ctx_to_dict` + `rs_ctx_from_dict` (thin wrappers) |
| `b2ff51f` | Phase 5: Cleanup (remove dead `shada.h` include, update run-result) |

**New Rust files (5):**
- `ffi.rs` — all FFI declarations
- `stack.rs` — `rs_ctx_free_all`
- `save.rs` — `rs_ctx_save`
- `restore.rs` — `rs_ctx_restore`
- `convert.rs` — `rs_ctx_to_dict`, `rs_ctx_from_dict`

**Result:** Every public function in `context.c` is now a thin `rs_*` wrapper. All business logic lives in Rust. The C file retains only statics, C accessor functions (for deeply C-coupled operations like HASHTAB_ITER, Arena allocation, OptVal management), and `_Static_assert` guards. All 4106 tests pass.
