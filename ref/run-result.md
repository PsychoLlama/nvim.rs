Plan: ~/.claude/plans/concurrent-wandering-zephyr.md

All 4 phases complete. Summary:

## BT Parser Migration — Complete

**4 commits, 4 functions migrated:**

| Commit | Function | Lines |
|--------|----------|-------|
| `aad261b` | `regpiece` — quantifier parsing (*, \+, \=, \{n,m}, \@=) | ~134 |
| `531c997` | `regconcat` — atom concatenation with inline magic mode switches | ~73 |
| `66e60c8` | `regbranch` — branch parsing with \& operators | ~39 |
| `c69f2ac` | `reg` — top-level parser with \(\), \z(\), \%(\) groups and \| alternation | ~98 |

**All checks pass:** build, smoke-test, rust-fmt-check, rust-clippy, rust-test (3486 tests).

**State:** 58 `#[no_mangle]` exports in regexp crate. BT compiler recursive descent parser fully in Rust except `regatom` (~800 lines) and `bt_regcomp` entry point.
