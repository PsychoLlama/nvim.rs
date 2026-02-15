#pragma once

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/vim_defs.h"  // IWYU pragma: keep

// Rust-exported functions (from nvim-rs/move, nvim-rs/plines, nvim-rs/window)
void update_topline(win_T *wp);
void update_curswant_force(void);
void update_curswant(void);
void check_cursor_moved(win_T *wp);
void changed_window_setting(win_T *wp);
void changed_window_setting_all(void);
void set_topline(win_T *wp, linenr_T lnum);
void changed_cline_bef_curs(win_T *wp);
void changed_line_abv_curs(void);
void changed_line_abv_curs_win(win_T *wp);
void validate_botline(win_T *wp);
void invalidate_botline(win_T *wp);
void approximate_botline_win(win_T *wp);
int cursor_valid(win_T *wp);
void validate_cursor(win_T *wp);
void validate_virtcol(win_T *wp);
void validate_cheight(win_T *wp);
int win_col_off(win_T *wp);
int win_col_off2(win_T *wp);
void curs_columns(win_T *wp, int may_scroll);
void scroll_redraw(int up, linenr_T count);
void adjust_skipcol(void);
void scrolldown_clamp(void);
void scrollup_clamp(void);
void set_empty_rows(win_T *wp, int used);
void cursor_correct(win_T *wp);
void do_check_cursorbind(void);
int sms_marker_overlap(win_T *wp, int extra2);
void set_valid_virtcol(win_T *wp, colnr_T vcol);
void scroll_cursor_top(win_T *wp, int min_scroll, int always);
void scroll_cursor_bot(win_T *wp, int min_scroll, int set_topbot);
void scroll_cursor_halfway(win_T *wp, int atend, int prefer_above);
int scrolldown(win_T *wp, linenr_T line_count, int byfold);
int scrollup(win_T *wp, linenr_T line_count, int byfold);
void check_topfill(win_T *wp, int down);
int pagescroll(int dir, int count, int half);
int adjust_plines_for_skipcol(win_T *wp);
int skipcol_from_plines(win_T *wp, int plines_off);
int scrolljump_value(win_T *wp);
int check_top_offset(win_T *wp);
void cursor_correct_sms(win_T *wp);
void redraw_for_cursorcolumn(win_T *wp);
void comp_botline(win_T *wp);
int virtcol2col(win_T *wp, linenr_T lnum, int vcol);

#include "move.h.generated.h"
