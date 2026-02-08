Plan: ~/.claude/plans/zany-questing-pixel.md

All 7 phases of the NFA Regexp Compiler migration are complete. Here's a summary of what was accomplished:

**Phase 6** (completed at start of session):
- Fixed clippy `match_same_arms` error (merged `NFA_LNUM => 90` into `NFA_BOW | NFA_EOW` arm)
- Added 25 test patterns, committed as `0cb5c95ce7`

**Phase 7** (nfa_regcomp Entry Point):
- Created `rs_nfa_regcomp()` in Rust (~90 lines) — orchestrates the full NFA compilation pipeline:
  1. `rs_nfa_regcomp_start()` → `rs_re2post()` → `rs_post2nfa()` (sizing) → `alloc_nfa_prog()` → `rs_post2nfa()` (build) → `rs_nfa_postprocess()` → `rs_nfa_get_reganch/regstart/match_text()`
- Added 8 new C accessor functions (split across two locations due to `state_ptr` forward declaration)
- C `nfa_regcomp()` is now a 1-line thin wrapper
- Added 10 integration test patterns (754 total corpus entries)
- All checks pass: build, smoke-test, fmt, clippy, 3513 tests, regexp-baseline
- Committed as `6e12ddced3`

The entire NFA compiler — from regexp parsing through Thompson NFA construction and postprocessing — now runs in Rust.
