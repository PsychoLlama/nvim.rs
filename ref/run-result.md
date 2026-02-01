Plan: ~/.claude/plans/synchronous-yawning-llama.md

All the work is committed. Let me provide a summary of what was accomplished.

## Summary

I have executed the plan at `/home/overlord/.claude/plans/synchronous-yawning-llama.md`. Here's what was completed:

### Phase 21: Fix Infinite Loop Bug ✓
- Added `MATCH_FOLLOWS_MAX_ITER` constant (10,000) to `nfa_exec.rs`
- Added iteration counter to the `match_follows()` while loop
- Returns `false` when limit exceeded to prevent infinite loops on cyclic NFA graphs
- **Commit**: `1bb5ba7dae`

### Phase 22: Remove Dead NFA Post-Processing Functions ✓
- Removed C implementations that were replaced by Rust:
  - `nfa_get_reganch()` (~55 lines)
  - `nfa_get_regstart()` (~80 lines)
  - `nfa_get_match_text()` (~35 lines)
  - `nfa_postprocess()` (~47 lines)
- The Rust implementations (`rs_nfa_*`) are already being called from `nfa_regcomp()`
- **Lines removed**: ~217
- **Commit**: `9c7c21b998`

### Phase 23: Remove Dead NFA Parsing Wrapper Functions ✓
- Removed thin C wrapper functions that were calling Rust directly:
  - `nfa_regpiece()`, `nfa_regconcat()`, `nfa_regbranch()`, `nfa_reg()`
- Updated `re2post()` to call `rs_nfa_reg()` directly
- Removed unused error message strings (moved to Rust)
- **Lines removed**: ~71
- **Commit**: `9b02c61745`

### Phase 24 & 25: Deferred
- **Phase 24** (state save/restore): Functions are still needed by the C backtracking (BT) engine. Cannot be removed until BT engine is migrated.
- **Phase 25** (debug functions): Optional, inside `#ifdef REGEXP_DEBUG` blocks. Low priority.

### Results
- **Total C lines removed**: ~288 lines (from 9,027 to 8,739)
- **All verification steps pass**:
  - `just build` ✓
  - `just smoke-test` ✓
  - `just rust-test` (3,735 tests) ✓
  - `just rust-fmt-check` ✓
  - `just rust-clippy` ✓
