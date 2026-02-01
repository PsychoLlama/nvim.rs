Plan: ~/.claude/plans/gentle-churning-conway.md

Phase 18 is complete. Here's a summary:

**Phase 18: Unicode Equivalence Classes Migration - Complete**

- Created `src/nvim-rs/regexp/src/equi_class.rs` with:
  - 52 Unicode equivalence class constants (A-Z, a-z variants)
  - `get_equi_class()` lookup function
  - `rs_reg_equi_class()` FFI export for BT engine
  - `rs_nfa_emit_equi_class()` FFI export for NFA engine
  - `emit_nfa_equi_class()` for internal Rust calls
  - Unit tests

- Modified files:
  - `src/nvim-rs/regexp/src/lib.rs` - added module and exports
  - `src/nvim-rs/regexp/src/nfa_parser.rs` - calls Rust directly
  - `src/nvim/regexp.c` - thin wrappers calling Rust

**Line count reduction:**
- regexp.c: 11,674 → 9,655 lines (**-2,019 lines**)
- Rust added: ~700 lines
- Net C reduction: **2,019 lines**

All builds pass, all tests pass, commit created: `736d25c5b2`.
