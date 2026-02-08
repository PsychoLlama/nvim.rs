Plan: ~/.claude/plans/refactored-stirring-emerson.md

Here's a summary of what was accomplished:

## NFA Execution Engine Migration Complete

All 6 phases of the plan at `/home/overlord/.claude/plans/refactored-stirring-emerson.md` have been executed:

**Commits:**
1. `cd45f0d` - Phase 8.4: Port `nfa_regmatch` core engine (~1,400 lines) + 150 C accessors + 35 test patterns
2. `7216eb9` - Phase 8.5: Port entry points (`nfa_regtry`, `nfa_regexec_both`, `nfa_regexec_nl`, `nfa_regexec_multi`) + 12 test patterns
3. `cc482b0` - Phase 8.6a: Wire `recursive_regmatch` to Rust, fix list alloc bug, fix extmatch null
4. `9843cad` - Phase 8.6b: Full cut-over of `nfa_regexec_nl` and `nfa_regexec_multi` to Rust

**Bugs found and fixed:**
- **List allocation bug**: `nvim_nfa_list_alloc_threads` was allocating only the thread array, not the wrapping `nfa_list_T` struct — caused heap corruption
- **Double-advance bug**: `NFA_MATCH` and `NFA_END_INVISIBLE` handlers called `advance_input()` then fell through to the outer loop's nextchar code, advancing input twice — caused off-by-one match lengths
- **Extmatch dangling pointer**: `re_extmatch_out` wasn't set to NULL after unref

**Verification:**
- Shadow mode (C and Rust in parallel) showed zero mismatches across 821 corpus patterns
- All checks pass: build, smoke-test, fmt, clippy, 3518 unit tests, regexp-baseline
