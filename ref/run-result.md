Plan: ~/.claude/plans/cheerful-riding-willow.md

All tests pass. Let me provide a summary of the completed work.

## Summary

I've executed the plan for Phases 28-32 of the regex engine migration. Here are the results:

### Completed Successfully:

**Phase 28: Inline Pure Static Wrappers** (-62 lines)
- Removed `reg_nextline()` wrapper - callers now call `rs_reg_nextline()` directly
- Removed `cstrncmp()` wrapper - callers now call `rs_cstrncmp()` directly  
- Removed `cstrchr()` wrapper (no C callers, only used from Rust)
- Removed `mb_decompose()` wrapper (no C callers, only used from Rust)
- Removed `nfa_regtry()` wrapper (no C callers, only used from Rust)
- Commit: `54885a131a`

**Phase 29: Consolidate skip_anyof, reg_equi_class, re_put_uint32** (-31 lines)
- Updated `skip_anyof()` callers to call `rs_skip_anyof()` directly
- Updated `reg_equi_class()` caller to call `rs_reg_equi_class()` directly
- Updated `re_put_uint32()` callers to call `rs_re_put_uint32()` directly
- Deleted static wrapper functions
- Commit: `5aac630f34`

**Phase 31: Remove Forward Declarations and Dead Code** (-1 line)
- Removed redundant forward declaration for `reg_getline_len()` (function was already defined earlier in file)
- Other forward declarations remain as they're still needed
- Commit: `41489875d1`

### Skipped:

**Phase 30: Migrate cleanup_subexpr/cleanup_zsubexpr** (SKIPPED)
- Attempted migration caused stack corruption ("stack smashing detected")
- Root cause appears to be related to struct size/alignment issues when accessing rex state from Rust
- Reverted to keep existing C implementation

**Phase 32: Migrate match_zref** (SKIPPED)
- Depends on Phase 30's `cleanup_zsubexpr()` migration
- Skipped because Phase 30 was unsuccessful

### Results:
- **Lines removed**: 94 total (62 + 31 + 1)
- **Target**: 120-150 lines (partially achieved due to Phase 30/32 skip)
- **regexp.c**: Reduced from 8,480 to 8,403 lines (77 lines net reduction accounting for some extern declarations added)
- **Commits**: 3 commits made
- **All tests pass**: build, smoke-test, rust-test, rust-fmt-check, rust-clippy
