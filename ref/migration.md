# Neovim C-to-Rust Migration

## Crate Structure

All Rust code in `src/nvim-rs/`. Each crate handles a specific domain:

| Crate        | Purpose                                            |
| ------------ | -------------------------------------------------- |
| api          | API types (Object, Dict, Array), type conversions  |
| arabic       | Arabic text shaping, combining char support        |
| ascii        | ASCII character predicates                         |
| autocmd      | Autocommand state, pattern parsing, event checking |
| buffer       | Buffer type/state queries, bufref validation       |
| buffer_updates | Buffer update tracking                           |
| charset      | Character classification, skip*, str2nr, case folding |
| cmdhist      | History type conversion                            |
| cmdline      | Command line state queries                         |
| collections  | Data structures (garray, hashtab)                  |
| compositor   | Grid compositor for multi-grid UI: layer management, grid selection |
| context      | Context stack management                           |
| cursor_shape | Cursor mode/shape queries                          |
| decoration   | Decoration/virtual text system, DecorState access  |
| diff         | Diffopt flag queries                               |
| drawline     | Line drawing and rendering helpers                 |
| drawscreen   | Window separator drawing, connector functions      |
| edit         | Edit mode state queries, prompt text functions     |
| encoding     | Base64, SHA-256                                    |
| eval         | VimL name validation, tristate                     |
| event        | Event loop, libuv wrappers, multiqueue             |
| ex_docmd     | Ex command parsing helpers                         |
| ex_eval      | Exception state checking                           |
| fileio       | File time comparison                               |
| fold         | Fold method checks, fold state queries             |
| fuzzy        | Fuzzy matching algorithm (fzy-based)               |
| getchar      | Typeahead buffer state                             |
| grid         | Screen character (schar_T) operations              |
| help         | Help tag heuristics                                |
| highlight    | Color/attr system, HlEntry storage, namespace mgmt |
| indent       | Tab/indent size calculation                        |
| insexpand    | CTRL-X completion mode checking                    |
| keycodes     | Key modifier parsing                               |
| linematch    | Line matching algorithm for diff alignment         |
| lua          | Lua executor state, callback FFI                   |
| mark         | Mark indices, position comparison                  |
| math         | Floating-point classification, bit ops             |
| mbyte        | UTF-8/multibyte encoding, char width               |
| memory       | xmalloc/xfree wrappers                             |
| memutil      | Memory/string utilities                            |
| menu         | Menu type classification                           |
| msgpack      | MessagePack serialization primitives               |
| ops          | Operator type queries                              |
| os           | OS abstractions (env, time, fs)                    |
| path         | Path manipulation, separators                      |
| plines       | Physical line display calculations                 |
| popupmenu    | Popup menu visibility state queries                |
| profile      | Profiling time arithmetic                          |
| quickfix     | Quickfix/location list state queries               |
| register     | Register validation, type formatting, width calc   |
| search       | Character search state, search/substitute tracking |
| spell        | Spell check utilities                              |
| statusline   | Status line rendering helpers                      |
| strings      | String comparison, case conversion, escape utils   |
| textformat   | Text formatting options, format queries            |
| tui          | Terminal UI, terminfo formatting, TUI output       |
| typval       | VimL typval_T type checking and value extraction   |
| ugrid        | Unicode grid (UGrid) operations for TUI            |
| unpacker     | MessagePack unpacking                              |
| utf8proc     | utf8proc FFI bindings                              |
| version      | Version compatibility checks                       |
| window       | Window state, validation, frame layout             |

## Remaining C Code (By Design)

### Highlight

- `dict2hlattrs` - Uses generated API keyset `Dict(highlight)`, complex to port
- `ns_get_hl` callback logic - Blocked by dict2hlattrs
- `highlight_changed` - Calls into syntax/screen subsystems

**highlight_group.c** (highlight group functions - MIGRATED):
- `highlight_num_groups` - Migrated to Rust (rs_highlight_num_groups)
- `highlight_has_attr` - Migrated to Rust (rs_highlight_has_attr) - Check if highlight group has specific attribute

### Grid

- `grid_alloc/grid_free` - Tightly coupled to C memory management
- `win_grid_alloc` - Window system integration
- `grid_draw_border` - VirtText iteration, decoration system FFI complexity

## Functions Requiring Infrastructure

### Complex Struct FFI (win_T, buf_T)

**window.c** (frame tree functions - MIGRATED):
- `frame_has_win` - Migrated to Rust (rs_frame_has_win)
- `is_bottom_win` - Migrated to Rust (rs_is_bottom_win)
- `frame_fixed_height/width` - Migrated to Rust
- `frame2win` - Migrated to Rust (rs_frame2win)
- `frame_check_height/width` - Migrated to Rust
- `frame_minheight/minwidth` - Migrated to Rust (rs_frame_minheight, rs_frame_minwidth)
- `win_comp_pos/frame_comp_pos` - Migrated to Rust (rs_win_comp_pos, rs_frame_comp_pos)
- `frame_setheight/setwidth` - Migrated to Rust (rs_frame_setheight, rs_frame_setwidth)
- `win_setheight_win/setwidth_win` - Migrated to Rust (rs_win_setheight_win, rs_win_setwidth_win)
- `frame_add_height` - Migrated to Rust (rs_frame_add_height)
- `frame_add_statusline` - Migrated to Rust (rs_frame_add_statusline)
- `frame_set_vsep` - Migrated to Rust (rs_frame_set_vsep)
- `frame_add_hsep` - Migrated to Rust (rs_frame_add_hsep)
- `frame_fix_width/height` - Migrated to Rust (rs_frame_fix_width, rs_frame_fix_height)

**drawscreen.c** (separator and status functions - MIGRATED):
- `hsep_connected/vsep_connected` - Migrated to Rust
- `draw_vsep_win/draw_hsep_win` - Migrated to Rust
- `get_corner_sep_connector` - Migrated to Rust
- `draw_sep_connectors_win` - Migrated to Rust
- `win_redraw_last_status` - Migrated to Rust (rs_win_redraw_last_status) - Frame tree traversal for status line redraw
- `status_redraw_all` - Migrated to Rust (rs_status_redraw_all) - Mark all windows for status redraw
- `status_redraw_curbuf` - Migrated to Rust (rs_status_redraw_curbuf) - Mark current buffer windows for redraw
- `status_redraw_buf` - Migrated to Rust (rs_status_redraw_buf) - Mark windows of given buffer for redraw

**ui_compositor.c** (compositor functions - MIGRATED):
- `ui_comp_should_draw` - Migrated to Rust (rs_ui_comp_should_draw)
- `curgrid_covered_above` - Migrated to Rust (rs_curgrid_covered_above) - Layer check for cursor grid
- `ui_comp_set_grid` - Migrated to Rust (rs_ui_comp_set_grid) - Set current grid by handle
- `ui_comp_compose_grid` - Migrated to Rust (rs_ui_comp_compose_grid) - Compose a grid's area onto the screen
- `ui_comp_raise_grid` - Migrated to Rust (rs_ui_comp_raise_grid) - Raise grid in layer stack
- `ui_comp_remove_grid` - Migrated to Rust (rs_ui_comp_remove_grid) - Remove grid from layer stack
- `ui_comp_put_grid` - Migrated to Rust (rs_ui_comp_put_grid) - Place grid at position in compositor
- `ui_comp_grid_cursor_goto` - Migrated to Rust (rs_ui_comp_grid_cursor_goto) - Cursor positioning with grid focus
- `ui_comp_layers_adjust` - Migrated to Rust (rs_ui_comp_layers_adjust) - Adjust layer position by z-index
- `ui_comp_set_screen_valid` - Migrated to Rust (rs_ui_comp_set_screen_valid) - Set screen validity flag
- `ui_comp_get_grid_at_coord` - Migrated to Rust (rs_ui_comp_get_grid_at_coord) - Get topmost grid at coordinates
- `ui_comp_mouse_focus` - Migrated to Rust (rs_ui_comp_mouse_focus) - Get grid for mouse focus at coordinates

**window.c** (validation functions - MIGRATED):
- `win_valid` - Migrated to Rust (rs_win_valid) - Window pointer validation
- `tabpage_win_valid` - Migrated to Rust (rs_tabpage_win_valid) - Window in tabpage validation
- `win_valid_any_tab` - Migrated to Rust (rs_win_valid_any_tab) - Window validation across tabs
- `win_find_by_handle` - Migrated to Rust (rs_win_find_by_handle) - Find window by handle
- `win_count` - Migrated to Rust (rs_win_count) - Count windows

**buffer.c**:
- `buf_valid` - Buffer pointer validation (needs buffer list)

**plines.c** (display calculations - partially MIGRATED):
- `win_chartabsize` - Migrated to Rust (rs_win_chartabsize)
- `charsize_fast` - Migrated to Rust (rs_charsize_fast)
- `linesize_fast` - Migrated to Rust (rs_linesize_fast)
- `in_win_border` - Migrated to Rust (rs_in_win_border)
- `charsize_regular` - Migrated to Rust (rs_charsize_regular) - includes marktree accessor pattern
- `linesize_regular` - Migrated to Rust (rs_linesize_regular) - includes character iteration accessors
- `getvcol` - Migrated to Rust (rs_getvcol) - includes visual mode accessors
- `plines_win_nofold` - Migrated to Rust (rs_plines_win_nofold)
- `plines_win_col` - Migrated to Rust (rs_plines_win_col)

**move.c** (scroll/position calculations - partially MIGRATED):
- `sms_marker_overlap` - Migrated to Rust (rs_sms_marker_overlap)
- `adjust_plines_for_skipcol` - Migrated to Rust (rs_adjust_plines_for_skipcol)
- `skipcol_from_plines` - Migrated to Rust (rs_skipcol_from_plines)
- `scrolljump_value` - Migrated to Rust (rs_scrolljump_value)

### Global State Dependencies

**cursor_shape.c** (shape_table global - MIGRATED):
- `cursor_is_block_during_visual` - Migrated to Rust (rs_cursor_is_block_during_visual)
- `cursor_mode_uses_syn_id` - Migrated to Rust (rs_cursor_mode_uses_syn_id)
- `cursor_get_mode_idx` - Migrated to Rust (rs_cursor_get_mode_idx)
- Uses accessor functions for shape_table global

**version.c** (static version arrays - MIGRATED):
- `min_vim_version`, `highest_patch`, `has_vim_patch` - Migrated to Rust via accessor functions for vim_versions/included_patchsets arrays

**textformat.c** (MIGRATED):
- `has_format_option` - Migrated to Rust (rs_has_format_option)

### Static Data Tables

**digraph.c**:
- `digraph_get`, `getexactdigraph` - Use user_digraphs global and static table

**keycodes.c**:
- `get_special_key_code` - Uses key_names_table static array

**autocmd.c** (MIGRATED):
- `event_nr2name` - Migrated to Rust (rs_event_nr2name) - Uses event_names static array via accessor

### Blocked OS Functions

| Function | Blocker |
|----------|---------|
| `os_chdir` | Uses verbose_enter/leave, ui_call_chdir |
| `os_fsync` | Updates g_stats.fsync counter |
| `os_can_exe` | PATH searching with helper functions |
| `os_mkdir_recurse` | Uses xmalloc, path helpers |
| `os_readv` | Uses struct iovec (vectored I/O) |

## Recent Progress: Rendering System Migration

### drawline.c Migrated Functions

- `advance_color_col` - Color column pointer advancement (rs_advance_color_col)
- `draw_col_fill` - Fill column with character (rs_draw_col_fill)
- `use_cursor_line_highlight` - Cursorline highlight check (rs_use_cursor_line_highlight)
- `draw_foldcolumn` - Foldcolumn setup (rs_draw_foldcolumn)
- `fill_foldcolumn` - Foldcolumn filling (rs_fill_foldcolumn)
- `line_putchar` - UTF-8 char to screen char (rs_line_putchar)
- `draw_col_buf` - Draw character buffer to line (rs_draw_col_buf)
- `apply_cursorline_highlight` - Apply cursorline highlight to line (rs_apply_cursorline_highlight)
- `set_line_attr_for_diff` - Set line attr for diff mode (rs_set_line_attr_for_diff)
- `handle_breakindent` - Breakindent for wrapped text (rs_handle_breakindent)
- `handle_showbreak_and_filler` - Showbreak and filler lines (rs_handle_showbreak_and_filler)
- `has_more_inline_virt` - Check for more inline virtual text (rs_has_more_inline_virt)
- `handle_inline_virtual_text` - Inline virtual text processing (rs_handle_inline_virtual_text)
- `wlv_put_linebuf` - Put line buffer to screen grid (rs_wlv_put_linebuf)

### drawline.c Accessor Functions (for Rust access to winlinevars_T)

Added ~30 accessor functions for winlinevars_T fields:
- `nvim_wlv_get/set_vcol`, `nvim_wlv_get/set_col`, `nvim_wlv_get/set_off`
- `nvim_wlv_get/set_n_extra`, `nvim_wlv_get/set_p_extra`, `nvim_wlv_get/set_sc_extra`
- `nvim_wlv_get/set_char_attr`, `nvim_wlv_get/set_n_attr`, `nvim_wlv_get/set_extra_attr`
- `nvim_wlv_get/set_color_cols`, `nvim_wlv_get/set_line_attr`
- And more for virtual text, skip cells, boguscols, etc.

### decoration.c Migrated Functions

- `decor_virt_pos` - Check if decor has virtual position (rs_decor_virt_pos)
- `decor_virt_pos_kind` - Get virtual text position kind (rs_decor_virt_pos_kind)

### decoration.c Accessor Functions (for Rust access to DecorState)

Added ~25 accessor functions for decoration system:

**DecorState accessors:**
- `nvim_get_decor_state` - Get global decor_state
- `nvim_decor_state_get/set_eol_col`, `nvim_decor_state_get_row`
- `nvim_decor_state_get_current_end`, `nvim_decor_state_get_current`
- `nvim_decor_state_get_conceal`, `nvim_decor_state_get_conceal_char`
- `nvim_decor_state_get_win`, `nvim_decor_state_get_range`

**DecorRange accessors:**
- `nvim_decor_range_get_start_row/col`, `nvim_decor_range_get_end_row/col`
- `nvim_decor_range_get/set_draw_col`, `nvim_decor_range_get_kind`
- `nvim_decor_range_has_virt_pos`, `nvim_decor_range_get_virt_pos_kind`
- `nvim_decor_range_get_virt_text`

**DecorVirtText accessors:**
- `nvim_decor_virt_text_get_hl_mode`, `nvim_decor_virt_text_get_pos`
- `nvim_decor_virt_text_get_width`, `nvim_decor_virt_text_get_flags`
- `nvim_decor_virt_text_get_chunk_count/text/hl_id`

### drawline.c Virtual Text Functions

- `draw_virt_text_item` - Individual virtual text chunk rendering (rs_draw_virt_text_item)
- `draw_virt_text` - Virtual text positioning loop (rs_draw_virt_text)

### drawline.c Line Initialization Functions

- `win_line_start` - Initialize line buffer for rendering (rs_win_line_start)
- `fix_for_boguscols` - Adjust columns for bogus chars (rs_fix_for_boguscols)

### Rendering Functions Pending

**drawscreen.c:**
- `showmode` - Mode indicator display (message system integration)
- `update_screen` - Screen orchestration (large, many side effects)
- `redraw_statuslines` - Orchestration function, needs win_check_ns_hl, win_redr_winbar, win_redr_status, draw_tabline, maketitle callable

**statusline.c:**
- `win_redr_status` - Status line redraw
- `draw_tabline` - Tab line rendering

## Migration Status Summary

Most simple state-checking and utility functions across all crates have been migrated. The remaining functions fall into these categories:

### Orchestration Functions (Need Callable Dependencies)
Functions that call multiple other C functions and would require making those functions extern:
- `redraw_statuslines` - calls win_check_ns_hl, win_redr_winbar, win_redr_status, draw_tabline, maketitle
- `update_screen` - complex screen orchestration with many subsystem calls

### Functions Requiring New Accessors
Functions blocked on missing accessor infrastructure:
- `cursor_valid`, `validate_cursor` - need check_cursor_moved callable from Rust

**move.c** (window validity functions - MIGRATED):
- `invalidate_botline` - Migrated to Rust (rs_invalidate_botline) - clears VALID_BOTLINE and VALID_BOTLINE_AP
- `approximate_botline_win` - Migrated to Rust (rs_approximate_botline_win) - clears only VALID_BOTLINE
- `changed_cline_bef_curs` - Migrated to Rust (rs_changed_cline_bef_curs) - clears cursor line validity flags
- `changed_line_abv_curs` - Migrated to Rust (rs_changed_line_abv_curs) - clears validity flags on curwin
- `changed_line_abv_curs_win` - Migrated to Rust (rs_changed_line_abv_curs_win) - clears validity flags on window
- `changed_window_setting` - Migrated to Rust (rs_changed_window_setting) - handles window setting changes
- `changed_window_setting_all` - Migrated to Rust (rs_changed_window_setting_all) - iterates all tabpages/windows

**tui.c** (TUI output functions - MIGRATED):
- `attrs_differ` - Migrated to Rust (rs_attrs_differ) - compares highlight attributes
- `tui_grid_cursor_goto` - Migrated to Rust (rs_tui_grid_cursor_goto) - sets cursor position
- `tui_hl_attr_define` - Migrated to Rust (rs_tui_hl_attr_define) - stores highlight attrs
- `tui_default_colors_set` - Migrated to Rust (rs_tui_default_colors_set) - sets default colors
- `tui_grid_resize` - Migrated to Rust (rs_tui_grid_resize) - resizes grid and clips invalid regions
- `tui_grid_clear` - Migrated to Rust (rs_tui_grid_clear) - clears grid and screen region
- `tui_busy_start` - Migrated to Rust (rs_tui_busy_start) - sets busy flag
- `tui_busy_stop` - Migrated to Rust (rs_tui_busy_stop) - clears busy flag
- `tui_bell` - Migrated to Rust (rs_tui_bell) - outputs bell character
- `tui_set_icon` - Migrated to Rust (rs_tui_set_icon) - stub (not implemented)
- `tui_mouse_on` - Migrated to Rust (rs_tui_mouse_on) - enables mouse tracking
- `tui_mouse_off` - Migrated to Rust (rs_tui_mouse_off) - disables mouse tracking
- `tui_update_menu` - Migrated to Rust (rs_tui_update_menu) - stub (menus are GUI only)
- `tui_visual_bell` - Migrated to Rust (rs_tui_visual_bell) - visual bell effect
- `set_scroll_region` - Migrated to Rust (rs_set_scroll_region) - sets terminal scroll region
- `reset_scroll_region` - Migrated to Rust (rs_reset_scroll_region) - resets scroll region
- `print_cell` - Migrated to Rust (rs_print_cell) - outputs a cell to terminal
- `tui_grid_scroll` - Migrated to Rust (rs_tui_grid_scroll) - scrolls grid region
- `tui_is_stopped` - Migrated to Rust (rs_tui_is_stopped) - checks if TUI is stopped
- `tui_set_title` - Migrated to Rust (rs_tui_set_title) - sets terminal title
- `tui_enable_extended_underline` - Migrated to Rust (rs_tui_enable_extended_underline) - enables extended underline support
- `tui_query_bg_color` - Migrated to Rust (rs_tui_query_bg_color) - queries terminal background color

**quickfix.c** (quickfix list functions):
- `qf_stack_empty` - Migrated to Rust (rs_qf_stack_empty) - checks if quickfix stack is empty
- `qf_list_empty` - Migrated to Rust (rs_qf_list_empty) - checks if quickfix list is empty
- `qf_list_has_valid_entries` - Migrated to Rust (rs_qf_list_has_valid_entries) - checks for valid entries
- `qf_entry_after_pos` - Migrated to Rust (rs_qf_entry_after_pos) - checks if entry is after position
- `qf_entry_before_pos` - Migrated to Rust (rs_qf_entry_before_pos) - checks if entry is before position
- `qf_entry_on_or_after_pos` - Migrated to Rust (rs_qf_entry_on_or_after_pos) - checks if entry is on or after position
- `qf_entry_on_or_before_pos` - Migrated to Rust (rs_qf_entry_on_or_before_pos) - checks if entry is on or before position

**ex_eval.c** (exception handling functions):
- `aborted_in_try` - Migrated to Rust (rs_aborted_in_try) - checks force_abort flag
- `aborting` - Migrated to Rust (rs_aborting) - checks if execution should abort
- `should_abort` - Migrated to Rust (rs_should_abort) - checks if command should abort
- `update_force_abort` - Migrated to Rust (rs_update_force_abort) - updates force_abort from cause_abort

**ex_getln.c** (command line state functions):
- `text_locked` - Migrated to Rust (rs_text_locked) - checks if text editing is locked
- `get_text_locked_msg` - Migrated to Rust (rs_get_text_locked_msg) - returns error message for text lock

**edit.c** (edit mode functions):
- `prompt_curpos_editable` - Migrated to Rust (rs_prompt_curpos_editable) - checks if cursor is in editable position of prompt line

**eval.c** (expression evaluation):
- `eval_expr_valid_arg` - Migrated to Rust (rs_eval_expr_valid_arg) - checks if typval is valid expression
- `partial_name` - Migrated to Rust (rs_partial_name) - returns function name of a partial
- `is_luafunc` - Migrated to Rust (rs_is_luafunc) - checks if partial is the v:lua value

**message.c** (message display functions):
- `msg_use_printf` - Migrated to Rust (rs_msg_use_printf) - checks if messages should use printf (batch mode)

**indent.c** (indentation functions):
- `may_do_si` - Migrated to Rust (rs_may_do_si) - checks if conditions are OK for smart indenting

**insexpand.c** (completion functions):
- `pum_wanted` - Migrated to Rust (rs_pum_wanted) - checks if popup menu should be displayed
- `ins_compl_accept_char` - Migrated to Rust (rs_ins_compl_accept_char) - checks if char is part of completion item

**mbyte.c** (multibyte encoding functions):
- `bomb_size` - Migrated to Rust (rs_bomb_size) - returns BOM size for current buffer's encoding

TUIData accessor functions added for opaque handle pattern:
- nvim_tui_get/set_rgb, nvim_tui_get/set_row, nvim_tui_get/set_col
- nvim_tui_get/set_attrs, nvim_tui_get/set_clear_attrs
- nvim_tui_set_print_attr_id, nvim_tui_set_default_colors_flag
- nvim_tui_get_grid_height/width, nvim_tui_invalidate
- nvim_tui_get_is_starting, nvim_tui_get/set_pending_resize_events
- nvim_tui_get_invalid_regions_size, nvim_tui_clear_invalid_regions
- nvim_tui_clip_invalid_region, nvim_tui_get_grid
- nvim_tui_invalidate_grid_cursor, nvim_tui_get_width/height
- nvim_tui_out_resize, nvim_tui_clear_region

Terminfo output infrastructure for Rust:
- nvim_tui_out, nvim_tui_terminfo_out, nvim_tui_terminfo_print_num1/2
- nvim_tui_get_grid_row/col, nvim_tui_get/set_url
- nvim_tui_get_print_attr_id, nvim_tui_get_immediate_wrap
- nvim_tui_cursor_goto, nvim_tui_update_attrs
- nvim_tui_get_can_clear_attr, nvim_tui_get_can_erase_chars
- nvim_tui_get_set_default_colors, nvim_tui_cheap_to_print
- nvim_tui_get_default_attr, nvim_tui_get/set_busy
- nvim_tui_get/set_mouse_enabled, nvim_tui_get_mouse_move_enabled
- nvim_tui_get_screen_or_tmux, nvim_tui_flush_buf, nvim_tui_set_term_mode
- nvim_tui_get_can_scroll, nvim_tui_get_can_change_scroll_region
- nvim_tui_get_has_lr_margin_mode, nvim_tui_get_can_set_lr_margin
- nvim_tui_cursor_goto_internal, nvim_tui_update_attrs_internal
- nvim_tui_invalidate_region, nvim_tui_ugrid_scroll
- nvim_tui_get_stopped, nvim_tui_get_can_set_title
- nvim_tui_get/set_title_enabled, nvim_tui_get_buf_space

### Register Module (register.c - PARTIALLY MIGRATED)

**Migrated Functions (23 rs_* functions):**
- `rs_valid_yank_reg` - Register name validation
- `rs_get_unname_register` - Get unnamed register index
- `rs_is_literal_register` - Check if literal register (clipboard)
- `rs_op_reg_index` - Convert register name to index
- `rs_is_append_register` - Check if uppercase (append mode)
- `rs_get_register_name` - Get register name from index
- `rs_format_reg_type` - Format register type string
- `rs_update_yankreg_width` - Update blockwise register width
- `rs_op_reg_amount` - Get register count
- `rs_shift_delete_registers` - Shift numbered registers
- `rs_set_expr_line` - Set expression register
- `rs_get_expr_line_src` - Get expression source
- `rs_get_expr_line` - Evaluate and get expression result
- `rs_init_write_reg` - Initialize register for writing
- `rs_finish_write_reg` - Finalize register write
- `rs_get_reg_type` - Get register motion type
- `rs_yank_register_mline` - Check if register is linewise
- `rs_free_register` - Free register contents
- `rs_stuff_yank` - Store string in register
- `rs_copy_register` - Deep copy a register
- `rs_str_to_reg` - Convert string/list to register (UTF-8 aware)

**Accessor Functions (~45):**
- yankreg_T field getters/setters (size, type, width, timestamp, array)
- Memory allocation wrappers (xmalloc, xcalloc, xmallocz, xfree)
- String/line manipulation helpers (memcnt, memchrsub, cstr_to_string)
- UTF-8 functions (mb_string2cells, utf_ptr2cells_len, utf_ptr2len_len)

**Remaining Functions (complex, need extensive infrastructure):**
- `do_put` (~789 lines) - Buffer operations, undo, extmarks
- `do_record`, `do_execreg` - Macro recording/execution, typeahead
- `ex_display` - Register display, message system
- `insert_reg`, `get_spec_reg` - Special register handling
- `op_yank_reg` - Yank operations with block_def struct

### Complex Memory/Buffer Operations
Functions involving memory management or buffer content access:
- `buf_is_empty` - needs ml_get_buf for buffer line access
- `op_delete`, `op_change` - complex editing operations modifying buffer content
- `compose_line`, `compose_area`, `ui_comp_raw_line` - grid memory array access
