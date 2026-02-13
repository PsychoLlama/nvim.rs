Plan: ~/.claude/plans/whimsical-leaping-kay.md

All 6 phases of the `cmdhist.c` → Rust migration are complete. Here's the summary:

**Commits (6 total):**
1. Phase 1: FFI foundation + core helpers (`rs_get_histtype`, `HISTORY_NAMES`, `calc_hist_idx`, etc.)
2. Phase 2: History modification (`rs_init_history`, `rs_add_to_history`, `rs_clr_history`, `LAST_MAPTICK`)
3. Phase 3: Deletion (`rs_del_history_entry`, `rs_del_history_idx`)
4. Phase 4: VimL functions (`rs_f_histadd`, `rs_f_histdel`, `rs_f_histget`, `rs_f_histnr`)
5. Phase 5: Ex command + completion (`rs_ex_history`, `rs_get_history_arg`)
6. Phase 6: Iterators (`rs_hist_iter`, `rs_hist_get_array`)

**Rust files created:**
- `src/nvim-rs/cmdhist/src/ffi.rs` — opaque handle types + extern declarations
- `src/nvim-rs/cmdhist/src/helpers.rs` — core helpers, name table, index calculations
- `src/nvim-rs/cmdhist/src/modify.rs` — history init/add/clear
- `src/nvim-rs/cmdhist/src/delete.rs` — regex-based and index-based deletion
- `src/nvim-rs/cmdhist/src/viml.rs` — VimL function implementations
- `src/nvim-rs/cmdhist/src/command.rs` — `:history` command + completion
- `src/nvim-rs/cmdhist/src/iter.rs` — ShaDa iteration/array access

**Result:** `cmdhist.c` is now reduced to statics (`history[]`, `hisidx[]`, `hisnum[]`, `hislen`) + C accessor functions + thin `rs_*` forwarding wrappers. All real logic lives in Rust. Every phase passes `just build`, `just smoke-test`, `just rust-fmt-check`, `just rust-clippy`, and `just rust-test` (4106 tests).
