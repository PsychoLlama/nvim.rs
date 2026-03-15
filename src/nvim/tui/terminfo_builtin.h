// uncrustify:off

// X-list macros for terminfo capability names.
// The static TerminfoEntry data has been moved to src/nvim-rs/tui/src/terminfo.rs.
// These macros are still used by terminfo_from_database() in terminfo.c.

#pragma once

#define XLIST_TERMINFO_BUILTIN \
  X(carriage_return) \
  X(change_scroll_region) \
  X(clear_screen) \
  X(clr_eol) \
  X(clr_eos) \
  X(cursor_address) \
  X(cursor_down) \
  X(cursor_invisible) \
  X(cursor_left) \
  X(cursor_home) \
  X(cursor_normal) \
  X(cursor_up) \
  X(cursor_right) \
  X(delete_line) \
  X(enter_bold_mode) \
  X(enter_ca_mode) \
  X(enter_italics_mode) \
  X(enter_reverse_mode) \
  X(enter_standout_mode) \
  X(enter_underline_mode) \
  X(erase_chars) \
  X(exit_attribute_mode) \
  X(exit_ca_mode) \
  X(from_status_line) \
  X(insert_line) \
  X(keypad_local) \
  X(keypad_xmit) \
  X(parm_delete_line) \
  X(parm_down_cursor) \
  X(parm_insert_line) \
  X(parm_left_cursor) \
  X(parm_right_cursor) \
  X(parm_up_cursor) \
  X(set_a_background) \
  X(set_a_foreground) \
  X(set_attributes) \
  X(set_lr_margin) \
  X(to_status_line) \
// end of list

#define XLIST_TERMINFO_EXT \
  X(reset_cursor_style, Se) \
  X(set_cursor_style, Ss) \
  X(enter_strikethrough_mode, smxx) \
  X(set_rgb_foreground, setrgbf) \
  X(set_rgb_background, setrgbb) \
  X(set_cursor_color, Cs) \
  X(reset_cursor_color, Cr) \
  X(set_underline_style, Smulx) \
// end of list

#define XYLIST_TERMINFO_KEYS \
  X(backspace) \
  Y(beg) \
  X(btab) \
  X(clear) \
  Y(dc) \
  Y(end) \
  Y(find) \
  Y(home) \
  Y(ic) \
  X(npage) \
  X(ppage) \
  X(select) \
  Y(suspend) \
  Y(undo) \
// end of list

#define XLIST_TERMINFO_FKEYS \
  X(f1) \
  X(f2) \
  X(f3) \
  X(f4) \
  X(f5) \
  X(f6) \
  X(f7) \
  X(f8) \
  X(f9) \
  X(f10) \
  X(f11) \
  X(f12) \
  X(f13) \
  X(f14) \
  X(f15) \
  X(f16) \
  X(f17) \
  X(f18) \
  X(f19) \
  X(f20) \
  X(f21) \
  X(f22) \
  X(f23) \
  X(f24) \
  X(f25) \
  X(f26) \
  X(f27) \
  X(f28) \
  X(f29) \
  X(f30) \
  X(f31) \
  X(f32) \
  X(f33) \
  X(f34) \
  X(f35) \
  X(f36) \
  X(f37) \
  X(f38) \
  X(f39) \
  X(f40) \
  X(f41) \
  X(f42) \
  X(f43) \
  X(f44) \
  X(f45) \
  X(f46) \
  X(f47) \
  X(f48) \
  X(f49) \
  X(f50) \
  X(f51) \
  X(f52) \
  X(f53) \
  X(f54) \
  X(f55) \
  X(f56) \
  X(f57) \
  X(f58) \
  X(f59) \
  X(f60) \
  X(f61) \
  X(f62) \
  X(f63) \
// end of list
