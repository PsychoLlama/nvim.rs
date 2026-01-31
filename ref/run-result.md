Plan: ~/.claude/plans/warm-popping-haven.md

All steps complete. Here's a summary:

## Summary

Successfully executed Phase 9 of the migration plan:

### Changes Made

1. **Updated call sites** in `nfa_regcomp()` (regexp.c):
   - Changed two `post2nfa()` calls to `rs_post2nfa()` with explicit parameters
   - Removed now-unused `state_ptr = prog->state` and `state_ptr = NULL` lines

2. **Deleted C code** (~876 lines from regexp.c):
   - `alloc_state()` - NFA state allocator
   - `frag()` - NFA fragment constructor
   - `list1()` - Pointer list creator
   - `patch()` - Output list patcher
   - `append()` - List concatenation
   - `st_error()` - Stack error reporter
   - `st_push()` - Stack push
   - `st_pop()` - Stack pop
   - `nfa_max_width()` - Maximum width estimator
   - `post2nfa()` - Main postfix-to-NFA converter

3. **Removed unused globals**:
   - `istate` - State allocation index
   - `state_ptr` - State array pointer
   - `empty` - Empty fragment sentinel

4. **Fixed Rust code**:
   - Added NULL checks for `out_nstate` parameter in `rs_post2nfa` to prevent crashes when called with NULL

### Verification

- ✅ `just build` - passes
- ✅ `just rust-fmt-check` - passes
- ✅ `just rust-clippy` - passes
- ✅ `just rust-test` - 3718 tests pass
- ✅ Functional tests (substitution, matching, groups, quantifiers) - all work correctly
- ✅ Line count reduced from 14,634 to 13,758 (-876 lines)
