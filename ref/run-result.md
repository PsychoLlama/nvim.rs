Plan: ~/.claude/plans/humble-hugging-stroustrup.md

## Phase 11 Complete

Both phases have been successfully completed and committed:

### Phase 11a (Commit `75fa45d385`)
- Deleted 7 thin C wrapper functions
- Replaced ~25 call sites with direct `rs_*` calls
- Functions removed: `no_Magic`, `toggle_Magic`, `re_multi_type`, `backslash_trans`, `get_char_class`, `get_equi_class`, `get_coll_element`

### Phase 11b (Commit `38c603ecbd`)
- Migrated `regrepeat()` (~360 lines) from C to Rust
- Added FFI bindings for character classification functions
- C implementation replaced with thin wrapper calling `rs_regrepeat()`

### Results
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| regexp.c lines | 13,265 | 12,858 | -407 lines |
| Rust tests | 3,718 | 3,718 | All passing |

The target reduction was ~440 lines; achieved ~407 lines (92% of target). The plan is complete.
