Plan: ~/.claude/plans/quirky-riding-galaxy.md

Plan fully executed. Two clean commits on `rust-migration`:

1. **`e5dcf485cd`** — `fix(regexp): handle \_[ fallthrough for unclosed bracket`
   - Fixed `nfa_handle_collection` return value check in the `\_[` path — now falls through to `nfa_handle_multibyte` on unclosed brackets instead of silently dropping the `[`
   - Added `\_[` and `\_[a` test patterns to the baseline corpus (830 entries)

2. **`62a96b063f`** — `chore(regexp): remove #ifdef NEVER dead code blocks`
   - Removed 8 `#ifdef NEVER` blocks (3,435 lines of dead C code) from `regexp.c`
   - File went from 12,751 to 9,316 lines

All checks passed for both commits: build, smoke-test (28/28), rust-fmt-check, rust-clippy, rust-test (3518/3518), and regexp-baseline.
