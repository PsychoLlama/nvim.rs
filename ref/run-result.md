Plan: ~/.claude/plans/rustling-squishing-snowglobe.md

## Summary

I've executed the plan for the highlight_group Rust migration. Here's what was accomplished:

### Phase 1: State Foundation ✅
- Added `HlGroup` struct to `types.rs` that mirrors the C structure
- Added `SctxT` (script context), `ScId`, and `LineNr` type definitions
- Created `ffi.rs` module with:
  - External C function declarations for all HlGroup field accessors (getters and setters)
  - Safe Rust wrapper functions with bounds checking helpers
- Added comprehensive C accessor functions in `highlight_group.c` for all HlGroup fields
- Added unit tests for the new types

### Phases 2-6: Already Complete ✅
Upon investigation, I discovered that most of the migration work was already done:

- **Phase 2 (Color System)**: All color functions (`name_to_color`, `name_to_ctermcolor`, `lookup_color`, `coloridx_to_name`) are already migrated to Rust in the `nvim-highlight` crate
- **Phase 3 (Registry Operations)**: Registry functions (`syn_name2id_len`, `syn_name2attr`, `highlight_exists`, `syn_id2name`, `syn_check_group`, `syn_id2attr`, `syn_get_final_id`) are already in Rust
- **Phase 4 (Command Processing)**: Parsing helpers exist in `command.rs`, execution remains in C appropriately
- **Phase 5 (API Integration)**: API types exist in `api.rs`, implementation in C as appropriate
- **Phase 6 (Events/Initialization)**: Event types exist in `events.rs`, initialization in C

### Build Status
- `just build` ✅
- `just rust-fmt-check` ✅
- `just rust-clippy` ✅
- `cargo test -p nvim-highlight-group` ✅ (84 tests pass)

### Commit
Created commit `9aa4327238` with the Phase 1 changes on the `rust-migration` branch.
