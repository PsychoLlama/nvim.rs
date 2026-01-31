Plan: ~/.claude/plans/inherited-prancing-glade.md

Phase 8 is now complete. Here's a summary of what was accomplished:

## Phase 8 Summary: Quickfix Entry Navigation Migration

### Changes Made

**Rust Implementation (src/nvim-rs/quickfix/src/lib.rs):**
- Added `rs_qf_get_nth_below_entry()` - Find nth entry below current position
- Added `rs_qf_get_nth_above_entry()` - Find nth entry above current position
- Added `rs_qf_find_nth_adj_entry()` - Main API for `:cabove/:cbelow` commands

**C Code Changes (src/nvim/quickfix.c):**
- Updated `ex_cbelow()` to call `rs_qf_find_nth_adj_entry()` instead of the C version
- Added extern declarations for the new Rust functions
- Deleted 9 C functions that are no longer needed:
  - `qf_find_first_entry_in_buf()`
  - `qf_find_first_entry_on_line()`
  - `qf_find_last_entry_on_line()`
  - `qf_entry_after_pos()` (thin wrapper)
  - `qf_entry_before_pos()` (thin wrapper)
  - `qf_entry_on_or_after_pos()` (thin wrapper)
  - `qf_entry_on_or_before_pos()` (thin wrapper)
  - `qf_find_entry_after_pos()`
  - `qf_find_entry_before_pos()`
  - `qf_find_closest_entry()`
  - `qf_get_nth_below_entry()`
  - `qf_get_nth_above_entry()`
  - `qf_find_nth_adj_entry()`

### Metrics
- **C lines deleted:** 264
- **Rust lines added:** 151
- **Net reduction:** 113 lines
- **Commands affected:** `:cabove`, `:cbelow`, `:labove`, `:lbelow`, `:cafter`, `:cbefore`, `:lafter`, `:lbefore`

### Verification
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` ✓ (3717 tests passed)
