Plan: ~/.claude/plans/concurrent-orbiting-kahn.md

The migration is complete. Here's a summary of what was accomplished:

## Summary

Successfully migrated `replace_termcodes()` from C to Rust, completing the keycodes module migration.

### Changes Made

**Phase 1 - C Accessor Functions (`src/nvim/keycodes.c`)**:
- Added `nvim_keycodes_get_current_sid()` - returns current script ID for `<SID>` translation
- Added `nvim_keycodes_get_leader()` - returns value of `g:mapleader`
- Added `nvim_keycodes_get_local_leader()` - returns value of `g:maplocalleader`
- Added `nvim_keycodes_emit_sid_error()` - emits "using <SID> not in script context" error

**Phase 2 - Rust Constants and FFI (`src/nvim-rs/keycodes/src/lib.rs`)**:
- Added `REPTERM_*` flags constants
- Added `KE_SNR` constant for script-local prefix
- Added `CTRL_V` constant
- Added `ScidT` type alias
- Added FFI declarations for the new C accessor functions

**Phase 3 - Rust Implementation**:
- Implemented `rs_replace_termcodes()` handling:
  - Standard keycode translation via `rs_trans_special`
  - `<SID>` translation for script-local function prefixes
  - `<Leader>` and `<LocalLeader>` variable substitution
  - Ctrl-V/backslash escape handling
  - K_SPECIAL byte escaping for multibyte characters
- Added `starts_with_ignore_case()` helper function

**Phase 4 - Integration**:
- Updated C wrapper to call Rust implementation
- C wrapper handles buffer allocation/reallocation
- Rust function operates on pre-allocated buffer

**Phase 5 - Cleanup**:
- Removed unused `mod_mask_table` and `modifier_keys_table` from C (now in Rust)
- Committed with descriptive message

### Verification
- ✅ `just build` - passes
- ✅ `just rust-fmt-check` - passes
- ✅ `just rust-clippy` - passes
- ✅ `just rust-test` - 3671 tests pass
