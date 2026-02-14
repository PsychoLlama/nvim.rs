Plan: ~/.claude/plans/moonlit-sleeping-frost.md

6 phases of diff.c migration completed (agent timed out during Phase 7). All committed phases build and pass smoke tests.

| Phase | Commit | Description |
|-------|--------|-------------|
| 1 | `97d6f1554e` | Migrate 18 utility functions to Rust |
| 2 | `bb63a441f8` | Migrate diff block management to Rust |
| 3 | `9c9e7ab5bc` | Migrate diff computation pipeline to Rust |
| 4 | `11c1312f13` | Migrate diff status checking & navigation to Rust |
| 5 | `f19cf462ca` | Migrate inline change detection to Rust |
| 6 | `55aa3ddd3c` | Migrate ex_diffupdate to Rust |

diff.c reduced from 4,620 → 3,757 lines (~863 lines of real C logic deleted). All checks pass: build, smoke-test.
