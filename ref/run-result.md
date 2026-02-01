Plan: ~/.claude/plans/snappy-soaring-otter.md

All checks pass:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` (3718 tests) ✓

## Summary

I've completed the execution of Phase 15 from the plan. Here's what was accomplished:

### What was implemented:

1. **Phase 15a - Infrastructure**: Added FFI declarations for character class predicates (`nvim_ri_digit`, `nvim_ri_hex`, etc.) and the `advance_reginput()` helper function.

2. **Phase 15b - Tier 1 opcodes**: Implemented Rust handlers for zero-width assertions: `BOL`, `EOL`, `RE_BOF`, `RE_EOF`

3. **Phase 15c - Tier 2 opcodes**: Implemented Rust handlers for basic character classes: `ANY`, `DIGIT`/`NDIGIT`, `HEX`/`NHEX`, `OCTAL`/`NOCTAL`, `WORD`/`NWORD`, `HEAD`/`NHEAD`, `ALPHA`/`NALPHA`, `LOWER`/`NLOWER`, `UPPER`/`NUPPER`, `WHITE`/`NWHITE`

4. **Phase 15d - Tier 3 opcodes**: Implemented Rust handler for `NOTHING`

5. **Phase 15e - Wiring**: Updated `nvim_bt_match_op` stub (for future use when the Rust `rs_regmatch_full` becomes the primary entry point)

### Important Note:

The Rust opcode handlers in `rs_match_one_op_full()` are infrastructure for future phases. Currently, the C `regmatch()` function is still the primary entry point via `nvim_bt_regmatch()`. The C switch cases were preserved because deleting them would break the current execution path.

To activate the Rust handlers, a future phase would need to:
1. Switch `rs_bt_regtry()` to call `rs_regmatch_full()` instead of `nvim_bt_regmatch()`
2. Implement proper C-to-Rust delegation in `nvim_bt_match_op()` for unmigrated opcodes
3. Only then delete the migrated C switch cases

### Files Modified:
- `src/nvim-rs/regexp/src/bt_exec.rs`: +311 lines (opcode handlers and FFI)
- `src/nvim/regexp.c`: +10 lines (updated stub function comment)
