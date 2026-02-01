Plan: ~/.claude/plans/elegant-leaping-stearns.md

The Phase 18a commit is complete. Let me provide a summary of what was accomplished and what remains.

## Summary

**Completed:**
- **Phase 18a**: Migrated 4 simple `:syntax` settings commands to Rust:
  - `:syntax case [match|ignore]` - ~26 lines of C logic moved to Rust
  - `:syntax conceal [on|off]` - ~26 lines of C logic moved to Rust  
  - `:syntax spell [toplevel|notoplevel|default]` - ~34 lines of C logic moved to Rust
  - `:syntax foldlevel [start|minimum]` - ~38 lines of C logic moved to Rust

**Added:**
- `rs_syn_cmd_case()`, `rs_syn_cmd_conceal()`, `rs_syn_cmd_spell()`, `rs_syn_cmd_foldlevel()` in commands.rs
- Synblock setter functions: `nvim_synblock_set_syn_ic()`, `nvim_synblock_set_syn_spell()`, `nvim_synblock_set_syn_foldlevel()`, `nvim_synblock_set_conceal()`
- FFI declarations for the setters in lib.rs

**Lines deleted from C:** ~70 lines (function bodies replaced with thin wrappers)

**Deferred phases (18b-18m):** These phases require significant C infrastructure for:
- Pattern management (match, region, keyword)
- Hashtable operations (keyword, clear)
- Message output formatting (list)
- File sourcing (include)  
- Memory allocation and string management (iskeyword)
- Variable operations (clear - do_unlet)

The remaining phases would be better approached after establishing more comprehensive FFI infrastructure for these C subsystems.
