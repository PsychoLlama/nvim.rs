#pragma once

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/normal_defs.h"
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

typedef int (*IndentGetter)(void);

/// flags for set_indent()
enum {
  SIN_CHANGED = 1,  ///< call changed_bytes() when line changed
  SIN_INSERT  = 2,  ///< insert indent before existing text
  SIN_UNDO    = 4,  ///< save line for undo before changing it
  SIN_NOMARK  = 8,  ///< don't adjust extmarks
};

typedef int (*Indenter)(void);

// Rust-exported functions (from nvim-rs/indent)
bool tabstop_set(const char *var, colnr_T **array);
int tabstop_padding(colnr_T col, int64_t ts_arg, const int *vts);
int tabstop_at(colnr_T col, int64_t ts, const int *vts, bool left);
colnr_T tabstop_start(colnr_T col, int ts, colnr_T *vts);
int *tabstop_copy(const int *oldts);
int tabstop_count(colnr_T *ts);
int tabstop_first(colnr_T *ts);
int get_sw_value(buf_T *buf);
int get_sw_value_indent(buf_T *buf, bool left);
int get_sw_value_col(buf_T *buf, colnr_T col, bool left);
int get_sts_value(void);
int get_indent(void);
int get_indent_lnum(linenr_T lnum);
int get_indent_buf(buf_T *buf, linenr_T lnum);
int indent_size_no_ts(const char *ptr);
int indent_size_ts(const char *ptr, int64_t ts, colnr_T *vts);
bool set_indent(int size, int flags);
bool briopt_check(const char *briopt, win_T *wp);
int get_breakindent_win(win_T *wp, const char *line);
bool inindent(int extra);
void op_reindent(oparg_T *oap, Indenter how);
bool preprocs_left(void);
bool may_do_si(void);
void ins_try_si(int c);
void change_indent(int type, int amount, int round, bool call_changed_bytes);
bool copy_indent(int size, const char *src);
void ex_retab(exarg_T *eap);
int get_lisp_indent(void);
bool use_indentexpr_for_lisp(void);

#include "indent.h.generated.h"
