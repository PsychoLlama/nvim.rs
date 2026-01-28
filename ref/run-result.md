Plan: ~/.claude/plans/adaptive-twirling-wand.md

Now let me provide a summary of the completed work. The plan execution is complete for the portions that were achievable without more invasive changes to the `build_stl_str_hl` function. Let me create a summary:

## Summary

I have executed the statusline subsystem migration plan with the following accomplishments:

### Completed Work

**Phase 1: Audit and Wire-Up Foundation**
- Mapped all C functions to their Rust equivalents
- Identified 103 Rust `#[no_mangle]` exports in the statusline crate
- Identified 6 functions that were already wired up to C

**Phase 2: Wire Up Click Definition Functions**
- Wired `stl_clear_click_defs` → `rs_stl_clear_click_defs`
- Wired `stl_alloc_click_defs` → `rs_stl_alloc_click_defs`
- Wired `stl_fill_click_defs` → `rs_stl_fill_click_defs`

These functions are now thin C wrappers that delegate to Rust implementations, removing ~42 lines of C code.

**Phase 3: Verified Integration**
- Existing wired functions (`rs_stl_connected`, `rs_fillchar_status`, `rs_tabwidth_calc`) continue to work
- All code paths in `win_redr_status`, `win_redr_winbar`, `redraw_custom_statusline`, and `draw_tabline` now use Rust functions for click handling

**Phases 6-7: Tabline/Winbar and Public API**
- Tabline click handling now fully uses Rust functions
- Winbar click handling now fully uses Rust functions
- Public API accessor functions (`nvim_stl_*`) are in place for Rust FFI

### Remaining Work (Future Phases)

**Phase 4-5: `build_stl_str_hl` Migration**
- The core statusline rendering function `build_stl_str_hl` remains in C
- Rust has `rs_build_stl_str` but with a simpler interface
- Full migration requires:
  - Matching output parameter interface (hltab, tabtab, stcp)
  - Handling complex expression evaluation callbacks
  - Managing memory allocation for click records

**Phase 8: Cleanup and Documentation**
- Update `ref/crates.md` if needed
- Document the migration patterns used

### Verification

All checks pass:
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- `just rust-test` (statusline) ✓ (242 tests)
