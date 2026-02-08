Plan: ~/.claude/plans/snoopy-roaming-neumann.md

All 4 phases of the plan are complete. Here's the summary:

## Completed Phases

| Phase | Functions | Commit |
|-------|-----------|--------|
| 1 | `reg_breakcheck` + `reg_iswordc` | `c1435556bd` |
| 2 | `reg_submatch` | `970c156b2d` |
| 3 | `reg_match_visual` | `26ad83248e` |
| 4 | `vim_regsub_both` literal path | `ec7e18213a` |

## Validation

All commits pass the full check suite:
- `just build` — links successfully
- `just smoke-test` — nvim starts without crashing
- `just rust-fmt-check` — formatting clean
- `just rust-clippy` — no warnings
- `just rust-test` — 3482 tests passing

## What Was Migrated

- **5 new `#[no_mangle]` exports**: `rs_reg_breakcheck`, `rs_reg_iswordc`, `rs_reg_submatch`, `rs_reg_match_visual`, `rs_vim_regsub_literal`
- **~30 new C accessors** in `regexp.c` (composite wrappers like `nvim_regexp_get_visual_area` kept the count manageable)
- **~500 lines of new Rust** including helpers for case conversion, backreference expansion, and space checking
- **Expression evaluation path** of `vim_regsub_both` remains in C as planned (deep eval subsystem dependency)

## Optional Manual Verification

The plan suggests interactive `:s` testing to confirm substitution works end-to-end — basic substitution, backreferences (`\1`, `\2`), case conversion (`\U&`), and expression substitution (verifies C path still works). The smoke test passes, but manual `:s` testing would add extra confidence.
