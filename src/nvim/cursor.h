#pragma once

#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Rust-exported functions (from nvim-rs/cursor)
int inc_cursor(void);
int dec_cursor(void);
int gchar_cursor(void);
int getviscol(void);
int getviscol2(colnr_T col, colnr_T coladd);
int coladvance_force(colnr_T wcol);
int coladvance(win_T *wp, colnr_T wcol);
int getvpos(win_T *wp, pos_T *pos, colnr_T wcol);
linenr_T get_cursor_rel_lnum(win_T *wp, linenr_T lnum);
void check_pos(buf_T *buf, pos_T *pos);
void check_cursor_lnum(win_T *win);
void check_cursor_col(win_T *win);
void check_cursor(win_T *wp);
void check_visual_pos(void);
void adjust_cursor_col(void);
bool set_leftcol(colnr_T leftcol);
int char_before_cursor(void);
void pchar_cursor(char c);
char *get_cursor_line_ptr(void);
char *get_cursor_pos_ptr(void);
colnr_T get_cursor_line_len(void);
colnr_T get_cursor_pos_len(void);

#include "cursor.h.generated.h"
