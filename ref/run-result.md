Plan: ~/.claude/plans/synchronous-dazzling-karp.md

## Summary — Batch 3 Complete

**6 commits**, each independently passing all checks (`build`, `smoke-test`, `rust-fmt-check`, `rust-clippy`, `rust-test`):

| Commit | Functions | New Accessors |
|--------|-----------|---------------|
| `360a93d` | `do_upper`, `do_lower` | 0 |
| `5045094` | `get_char_class` | 0 |
| `7295ab4` | `skip_anyof` | 0 |
| `10682dc` | `reg_getline_common` | 6 |
| `b69be75` | `regtilde` | 5 |
| `7584fa3` | `match_with_backref` | 10 |

**Totals:**
- **8 functions migrated** (do_upper, do_lower, get_char_class, skip_anyof, reg_getline_common, regtilde, match_with_backref + get_char_class circular dep resolved)
- **39 `#[no_mangle]` exports** in regexp crate (up from 32)
- **21 new C accessors**
- **3480 unit tests** passing (8 new tests for get_char_class)
- **Deferred:** `reg_match_visual` — too many globals (15+) and `getvvcol` cache invalidation risk
