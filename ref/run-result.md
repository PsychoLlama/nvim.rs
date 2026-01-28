Plan: ~/.claude/plans/nested-stirring-blanket.md

The plan has been fully executed. Here's a summary of what was accomplished:

## Summary

All 6 phases of the statusline migration plan have been completed:

### Phase 1: UI Extension Tabline
- Added `ui_ext.rs` module with tab/buffer iteration helpers
- Created FFI functions for counting tabs/buffers and getting handles
- Added iterator types (`TabIterator`, `BufferIterator`) for C consumption
- Added C accessor functions for tabpage handle and curwin

### Phase 2: Ruler Redraw
- Added `RulerRedrawAction` enum for redraw decisions
- Created `RulerRedrawContext` struct for input state
- Implemented `decide_ruler_action()` for complex conditional logic
- Added `calc_ruler_col()` for ruler positioning

### Phase 3: Tabline Drawing
- Added `TablineDrawAction` enum for draw decisions
- Created `TablineDrawContext` for input state
- Implemented `decide_tabline_action()` for draw path selection
- Added `TabRenderInfo` for per-tab state tracking
- Added helpers for prefix width, room checking, close button, and showcmd positioning

### Phase 4: Custom Window Redraw
- Added `CustomRedrawMode` enum for redraw mode selection
- Created `CustomRedrawContext` for setup state
- Implemented helpers for ruler column/maxwidth calculations
- Added userhl processing helpers for highlight record handling

### Phase 5: Public API Entry Points
- Added `StatusRedrawAction`/`StatusRedrawContext` for `win_redr_status`
- Added `WinbarRedrawAction`/`WinbarRedrawContext` for `win_redr_winbar`
- Added `CustomStatusRedrawAction` for `redraw_custom_statusline`

### Phase 6: Cleanup and Validation
- All checks pass: `just build`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test`
- 278 unit tests pass in the statusline crate
- 133 FFI exported functions total in the statusline crate

The migration follows the opaque handle pattern - Rust handles the complex conditional logic and state machine decisions, while C retains the actual grid/UI operations. This allows incremental migration without breaking the existing functionality.
