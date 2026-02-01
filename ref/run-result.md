Plan: ~/.claude/plans/delightful-painting-map.md

Phase 16 is complete. Here's a summary of what was accomplished:

## Summary

**Phase 16: Activate Rust Opcode Handlers and Delete C Switch Cases**

### Completed Tasks:

1. **Phase 16a - C Fallback Dispatcher**: Expanded `nvim_bt_match_op()` from a stub into a full dispatcher handling all unmigrated opcodes (CURSOR, RE_MARK, RE_VISUAL, BOW, EOW, IDENT, EXACTLY, BRANCH, BRACE_*, etc.)

2. **Phase 16b - Switch Entry Point**: 
   - Added WITH_NL preprocessing in `rs_regmatch_full` to handle newline character class variants (opcodes 50-78)
   - Changed `rs_bt_regtry()` to call `rs_regmatch_full` instead of `nvim_bt_regmatch`
   - Updated `rs_match_one_op_full` signature to include `next_out` parameter for opcodes that modify navigation

3. **Phase 16c - Delete Migrated Cases**: Removed ~164 lines of C switch cases from `regmatch()` for opcodes now handled in Rust:
   - BOL, EOL, RE_BOF, RE_EOF
   - ANY
   - WHITE, NWHITE, DIGIT, NDIGIT, HEX, NHEX, OCTAL, NOCTAL
   - WORD, NWORD, HEAD, NHEAD, ALPHA, NALPHA, LOWER, NLOWER, UPPER, NUPPER
   - NOTHING

4. **Phase 16d - Verification**: All checks pass:
   - `just build` ✓
   - `just rust-fmt-check` ✓
   - `just rust-clippy` ✓
   - `just rust-test` ✓ (3718 tests)

The Rust code path is now **active** for the BT regex engine. Migrated opcodes are handled in Rust (`rs_match_one_op_full`), while unmigrated opcodes fall back to C (`nvim_bt_match_op`).
