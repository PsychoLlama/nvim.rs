Plan: ~/.claude/plans/wobbly-fluttering-orbit.md

All 4 phases of the insexpand completion helpers migration are complete. Here's the summary:

## Completed

**Phase 1** — Wired 3 existing Rust functions:
- `ins_compl_make_cyclic` → `rs_ins_compl_make_cyclic`
- `ins_compl_make_linear` → `rs_ins_compl_make_linear`
- `pum_enough_matches` → `rs_pum_enough_matches`

**Phase 2** — Migrated 3 functions + 6 C accessors:
- `compl_match_curr_select` → `rs_compl_match_curr_select` (pum.rs)
- `get_cpt_sources_count` → `rs_get_cpt_sources_count` (source.rs, +8 unit tests)
- `check_elapsed_time` → `rs_check_elapsed_time` (source.rs)

**Phase 3** — Migrated 1 function + 16 C setter accessors:
- `ins_compl_clear` → `rs_ins_compl_clear` (state.rs, resets 18 state variables)

**Phase 4** — Migrated 2 functions + 7 C accessors:
- `ins_compl_fixRedoBufForLeader` → `rs_ins_compl_fixRedoBufForLeader` (leader.rs)
- `set_fuzzy_score` → `rs_set_fuzzy_score` (sort.rs)

**Totals**: 9 functions migrated/wired, ~29 new C accessor functions, 8 new unit tests. All checks pass (build, smoke-test, rust-fmt-check, rust-clippy, rust-test with 4036 tests).
