Plan: ~/.claude/plans/shiny-yawning-thompson.md

All 6 phases of the menu migration plan are complete. Here's a summary of what was accomplished:

**Phase 6 (this session)** ported the final 5 functions:
- `set_context_in_menu_cmd` → `rs_set_context_in_menu_cmd` in `complete_expand.rs` — command-line completion context setup
- `get_menu_name` → `rs_get_menu_name` — completion generator for submenus only
- `get_menu_names` → `rs_get_menu_names` — completion generator for menus + entries
- `ex_menutranslate` → `rs_ex_menutranslate` in `translate.rs` — `:menutranslate` command handler
- `menutrans_lookup` → `rs_menutrans_lookup` — translation table lookup

**Overall migration stats across all 6 phases:**
- ~28 functions migrated from C to Rust
- `menu.c` reduced from ~1,985 lines to thin wrappers + accessors + deferred functions
- 5 functions deferred (execute_menu, menu_get, menu_get_recursive, menuitem_getinfo, f_menu_info) as they're deeply tied to VimL eval types
- All checks pass: build, smoke-test, rust-fmt-check, rust-clippy, rust-test (4155 tests)
