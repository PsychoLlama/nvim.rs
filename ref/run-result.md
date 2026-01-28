Plan: ~/.claude/plans/scalable-pondering-ember.md

## Summary

I've successfully executed the migration plan. Here's what was accomplished:

### Phase 1: VTerm State Machine Migration

**Commits:**
1. `4fa475e16f` - feat(vterm): add C accessor functions and initial Rust state migration
2. `f65ab78a21` - feat(vterm): add scroll region handling and linefeed in Rust
3. `a236a28c73` - feat(vterm): add CSI cursor movement command handlers in Rust

**New C Accessor Functions Added to `state.c`:**
- Dimension accessors (rows, cols)
- Cursor position accessors (pos, at_phantom)
- Scroll region accessors (top, bottom, left, right) with computed bounds
- Line info accessors and scroll helpers
- Mode accessors (autowrap, insert, newline, origin, etc.)
- Protected cell accessor
- Callback accessors
- Tabstop accessors
- VTerm handle and grapheme buffer accessors

**New Rust FFI Functions in `state.rs`:**
- `rs_vterm_state_updatecursor` - cursor position update with callback
- `rs_vterm_state_setpos` - direct position setting
- `rs_vterm_state_cursor_moveto` - absolute cursor movement with bounds
- `rs_vterm_state_scroll` - full scroll implementation with lineinfo updates
- `rs_vterm_state_cursor_in_scrollregion` - check cursor in scroll region
- `rs_vterm_state_linefeed` - linefeed with scroll region handling
- Tabstop functions (set, clear, check)
- Mode query functions (autowrap, insert, newline, origin, protected)
- CSI cursor movement commands (CUU, CUD, CUF, CUB, CNL, CPL, CHA, CUP, VPA, HVP)

### Phase 2: Fold System

The exploration revealed that the fold system already has substantial Rust coverage with 46 FFI functions migrated across 7 modules (methods, tree, commands, level, markers, display, update). The existing implementation covers:
- Fold method checks
- Fold tree manipulation
- Fold level calculation
- Fold state management
- IEMS update algorithm
- Fold navigation and display

### Phase 3: Autocommand Pattern Matching

**Commit:** `e4d759c08a` - feat(autocmd): add glob pattern matching module in Rust

**New `pattern.rs` Module:**
- `glob_to_regex` - Convert shell-style glob patterns to regex patterns
  - `*` → `.*`
  - `?` → `.`
  - `{a,b}` → `\(a\|b\)` (Vim magic mode alternation)
  - Proper anchoring based on leading/trailing `*`
  - Path separator detection for `allow_dirs` flag
  - Error detection for unmatched braces

- `glob_match` - Simple glob matching without regex engine
  - Supports `*` and `?` wildcards
  - Case-insensitive matching option
  - Efficient backtracking algorithm

**FFI Exports:**
- `rs_glob_to_regex` - Convert glob to regex with buffer output
- `rs_glob_match_simple` - Direct pattern matching

**All Tests Pass:**
- 107 vterm tests
- 36 autocmd tests

**Verification:**
- `just build` ✓
- `just rust-fmt-check` ✓
- `just rust-clippy` ✓
- Unit tests ✓
