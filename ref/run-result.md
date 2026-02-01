Plan: ~/.claude/plans/graceful-cuddling-wren.md

Phase 17 is now complete. Here's a summary of what was accomplished:

## Phase 17 Complete - BT Regex Opcode Migration

All tasks from the plan at `/home/overlord/.claude/plans/graceful-cuddling-wren.md` have been completed.

### Summary

**1,845 lines of C code deleted** from `regexp.c`:
- `nvim_bt_match_op()` (632 lines) - the C fallback dispatcher
- `regmatch()` (1,188 lines) - the original C matching function
- `nvim_bt_regmatch()` (4 lines) - the Rust wrapper

### Commits Made:
1. `413ad90c2c` - Phase 17a: Word Boundaries and Character Classes
2. `f59cc1ad93` - Phase 17b: Exact Match Opcodes
3. `85b1740f49` - Phase 17c: Submatch Markers and Backreferences
4. `28ae16074f` - Phase 17d: Control Flow Opcodes  
5. `0d956dde63` - Phase 17e: Look-around and Position Checks
6. `17054af068` - Phase 17g: Delete C Fallback Functions
7. `69d5a0e101` - Update run-result documentation

### Verification:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` ✓ (3718 tests)

### Result:
The BT regex engine's core matching loop is now **100% in Rust**. All opcodes are handled natively by `rs_match_one_op_full()` in Rust, with no remaining fallback to the C `regmatch()` function.
