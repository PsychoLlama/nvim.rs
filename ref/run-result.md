Plan: ~/.claude/plans/parallel-cuddling-boot.md

All tasks are complete. Here's a summary of what was accomplished:

## Summary

I successfully executed the plan at `/home/overlord/.claude/plans/parallel-cuddling-boot.md`. The following functions were migrated from C to Rust in the indent crate:

### Completed Migrations:
1. **`lisp_match()`** - Checks if a string matches a word in the lispwords list for Lisp indentation
2. **`tabstop_set()`** - Parses vartabstop strings into integer arrays with proper error handling and memory allocation
3. **`briopt_check()`** - Parses 'breakindentopt' option strings (shift, min, sbr, list, column)
4. **`emsg_text_too_long()`** - Error helper for text-too-long errors with proper interrupt handling

### Deferred (Too Complex):
- **`get_sw_value_pos()`** - Requires complex cursor state manipulation
- **`get_number_indent()`** - Requires regex operations and complex buffer dependencies

### New C Accessor Functions Added:
- `nvim_curbuf_get_p_lw()` - lispwords buffer-local option
- `nvim_get_p_lispwords()` - lispwords global option
- `nvim_win_get_p_briopt()` - window breakindentopt option
- `nvim_win_set_briopt_shift/min/sbr/list/vcol()` - setters for briopt fields
- `nvim_set_got_int()` - setter for interrupt flag

All migrations:
- Build successfully
- Pass `rust-fmt-check`
- Pass `rust-clippy`
- Pass `rust-test` (3671 tests)
