Plan: ~/.claude/plans/serialized-puzzling-hedgehog.md

Phase 13 is complete. Here's a summary of what was accomplished:

## Phase 13: BT Execution Path Migration - Complete

### Summary

**Phases completed:**
1. **Phase 13a** - Added C accessor functions for BT-specific state (regstack, backpos, Z-subexpr positions, extmatch handling)
2. **Phase 13b** - Implemented `rs_bt_regtry()` in Rust, the match attempt function
3. **Phase 13c** - Implemented `rs_bt_regexec_both()` in Rust, the main BT execution entry point
4. **Phase 13d** - Updated `bt_regexec_nl()` and `bt_regexec_multi()` to call Rust, deleted C `regtry()` and `bt_regexec_both()` 
5. **Phase 13e** - Verification complete

**Lines deleted from regexp.c:** 279 lines (was 12,646, now 12,510)

**What was migrated:**
- `regtry()` → `rs_bt_regtry()` 
- `bt_regexec_both()` → `rs_bt_regexec_both()`
- BT regex execution now flows through Rust for the outer execution loop

**What remains in C:**
- `regmatch()` (~1,362 lines) - The core matching function is still in C because `rs_bt_regtry()` calls it through the `nvim_bt_regmatch` wrapper. To fully delete `regmatch()`, we would need to wire `rs_bt_regtry()` to use the Rust `rs_bt_regmatch()` function instead.

**Commits made:**
1. `a80e9ab89d` - feat(regexp): add BT execution accessors for Rust (Phase 13a)
2. `6f785e7700` - feat(regexp): migrate regtry to Rust (Phase 13b)
3. `2df0c32ea7` - feat(regexp): migrate bt_regexec_both to Rust (Phase 13c)
4. `841fd34b7c` - refactor(regexp): use Rust BT execution, delete C functions (Phase 13d)
