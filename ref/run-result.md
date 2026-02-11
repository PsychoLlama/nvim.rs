Plan: ~/.claude/plans/cheeky-gliding-bunny.md

All steps complete. Here's a summary:

## Summary

Successfully executed the cmdexpand.c Wave 4 migration plan:

### What was done

**Phase 1 — C Accessor Infrastructure:**
- Added 8 new C accessor functions: `nvim_cmdexpand_ptr2cells`, `nvim_cmdexpand_menu_is_separator`, `nvim_cmdexpand_ml_get`, `nvim_cmdexpand_ml_get_len`, `nvim_cmdexpand_msg_silent_inc/dec`, `nvim_cmdexpand_get_p_ic`, `nvim_cmdexpand_get_p_scs`
- Added 10 `_Static_assert` entries for `kOptWopFlagExacttext`, `RE_MAGIC`, `RE_STRING`, `EXPAND_HELP/MENUS/MENUNAMES/PATTERN_IN_BUF`, and `pos_T` layout
- Added `extern` declarations for 5 Rust functions

**Phase 2 — Wildmenu Display Helpers:**
- Replaced `skip_wildmenu_char` and `wildmenu_match_len` bodies with thin `rs_` wrapper calls
- These functions were already implemented in the `nvim_cmdline` crate, so no new Rust module was needed (avoiding duplicate symbols)

**Phase 3 — Pattern-in-Buffer Helpers:**
- Created `src/nvim-rs/cmdexpand/src/pattern.rs` with 3 migrated functions:
  - `rs_is_regex_match` — regex matching with ignorecase/smartcase
  - `rs_concat_pattern_with_buffer_match` — pattern+buffer text concatenation
  - `rs_copy_substring_from_pos` — multi-line buffer substring extraction (using `Vec<u8>` instead of `garray_T`)
- Replaced C function bodies with `rs_` wrapper calls

**Phase 4 — Verification:** All checks pass:
- `just build` — clean
- `just smoke-test` — passes (including 29 regexp smoke tests)
- `just rust-test` — 4058 tests pass
- `just rust-clippy` — clean
- `just rust-fmt-check` — clean

**Additional:** Added `nvim-cmdexpand` to the `justfile` `rust-test` list.
