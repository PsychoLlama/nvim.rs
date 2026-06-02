// register.c: C accessor wrappers for the Rust register crate (nvim-register).
//
// All register business logic lives in src/nvim-rs/register/src/lib.rs.
// This file contains only thin FFI bridges for C struct fields and compound
// C operations that cannot be expressed directly in Rust.

#include <stdbool.h>

#include "nvim/api/private/helpers.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/clipboard.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/extmark.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/register.h"
#include "nvim/state.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"

#include "register.c.generated.h"

// Global register state is owned by Rust; the C side declares them extern so
// that legacy C callers can still access them.
extern yankreg_T y_regs[NUM_REGISTERS];
extern yankreg_T *y_previous;
extern char *expr_line;
extern int execreg_lastc;

/// Set curwin->w_alt_fnum from a buf_T pointer (called from Rust write_reg_contents_ex).
void nvim_register_set_alt_fnum(buf_T *buf) { curwin->w_alt_fnum = buf->b_fnum; }

/// Free a register's contents (delegating to the Rust free_register export).
extern void free_register(yankreg_T *reg);
void nvim_free_register(yankreg_T *reg) { free_register(reg); }

/// Return curbuf->b_fname (for Rust get_spec_reg).
char *nvim_register_get_curbuf_fname(void) { return curbuf->b_fname; }

/// Return curwin (for Rust op_yank_reg update_topline call).
void *nvim_register_get_curwin(void) { return curwin; }

/// cbuf_as_string is a macro; provide a real function for Rust FFI.
String nvim_register_cbuf_as_string(char *buf, size_t len)
{
  return cbuf_as_string(buf, len);
}

/// Copy curwin->w_cursor into *pos (for Rust insert_reg).
void nvim_register_get_curwin_cursor(pos_T *pos) { *pos = curwin->w_cursor; }

/// Set curwin->w_cursor from *pos (for Rust insert_reg).
void nvim_register_set_curwin_cursor(const pos_T *pos) { curwin->w_cursor = *pos; }

/// Return the global State variable (for Rust insert_reg).
int nvim_register_get_State(void) { return State; }

/// Return ml_get_buf(curwin->w_buffer, curwin->w_cursor.lnum) (for Rust get_spec_reg).
char *nvim_register_ml_get_buf_curwin_lnum(void)
{
  return ml_get_buf(curwin->w_buffer, curwin->w_cursor.lnum);
}

// --- oparg_T accessors for Rust op_yank_reg ---
int nvim_oap_get_motion_type(oparg_T *oap) { return (int)oap->motion_type; }
void nvim_oap_get_start(oparg_T *oap, pos_T *pos) { *pos = oap->start; }
void nvim_oap_get_end(oparg_T *oap, pos_T *pos) { *pos = oap->end; }
bool nvim_oap_get_inclusive(oparg_T *oap) { return oap->inclusive; }
bool nvim_oap_get_is_VIsual(oparg_T *oap) { return oap->is_VIsual; }
int nvim_oap_get_line_count(oparg_T *oap) { return (int)oap->line_count; }
int nvim_oap_get_start_vcol(oparg_T *oap) { return (int)oap->start_vcol; }
int nvim_oap_get_end_vcol(oparg_T *oap) { return (int)oap->end_vcol; }
int nvim_oap_get_regname(oparg_T *oap) { return oap->regname; }
bool nvim_oap_get_excl_tr_ws(oparg_T *oap) { return oap->excl_tr_ws; }
int nvim_oap_get_op_type(oparg_T *oap) { return oap->op_type; }

// --- Yank message (NGETTEXT cannot be used from Rust) ---
void nvim_register_yank_msg(size_t yanklines, const char *namebuf, bool is_block)
{
  if (is_block) {
    smsg(0, NGETTEXT("block of %" PRId64 " line yanked%s",
                     "block of %" PRId64 " lines yanked%s", yanklines),
         (int64_t)yanklines, namebuf);
  } else {
    smsg(0, NGETTEXT("%" PRId64 " line yanked%s",
                     "%" PRId64 " lines yanked%s", yanklines),
         (int64_t)yanklines, namebuf);
  }
}

// --- Yank name buffer: fills buf with " into "regname or empty string ---
void nvim_register_yank_namebuf(int regname, char *buf, size_t bufsz)
{
  if (regname == NUL) {
    buf[0] = NUL;
  } else {
    vim_snprintf(buf, bufsz, _(" into \"%c"), regname);
  }
}

// --- Option accessors ---
int nvim_register_get_p_sel_char(void) { return *p_sel; }
int64_t nvim_register_get_p_report(void) { return p_report; }
bool nvim_register_p_cpo_has_regappend(void)
{
  return vim_strchr(p_cpo, CPO_REGAPPEND) != NULL;
}
bool nvim_register_cmod_lockmarks(void)
{
  return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0;
}

// --- curwin/curbuf accessors ---
int nvim_register_get_curwin_curswant(void) { return (int)curwin->w_curswant; }
void nvim_register_curbuf_set_op_start(const pos_T *pos) { curbuf->b_op_start = *pos; }
void nvim_register_curbuf_set_op_end(const pos_T *pos) { curbuf->b_op_end = *pos; }
void nvim_register_curbuf_set_op_start_col(int col) { curbuf->b_op_start.col = (colnr_T)col; }
void nvim_register_curbuf_set_op_end_col(int col) { curbuf->b_op_end.col = (colnr_T)col; }
void nvim_register_curbuf_decl_op_end(void) { decl(&curbuf->b_op_end); }

// --- tv_list_set_lock wrapper (inline in C, needs wrapper for Rust) ---
void nvim_register_tv_list_set_lock_fixed(list_T *list) { tv_list_set_lock(list, VAR_FIXED); }

// --- do_put accessor wrappers ---

/// Return true if curbuf->terminal is non-NULL (for Rust do_put).
bool nvim_dp_curbuf_is_terminal(void) { return curbuf->terminal != NULL; }

/// Return curbuf->b_op_start (for Rust do_put).
void nvim_dp_get_op_start(pos_T *pos) { *pos = curbuf->b_op_start; }

/// Return curbuf->b_op_end (for Rust do_put).
void nvim_dp_get_op_end(pos_T *pos) { *pos = curbuf->b_op_end; }

/// Set curbuf->b_op_start.lnum (for Rust do_put).
void nvim_dp_set_op_start_lnum(int lnum) { curbuf->b_op_start.lnum = (linenr_T)lnum; }

/// Set curbuf->b_op_end.lnum (for Rust do_put).
void nvim_dp_set_op_end_lnum(int lnum) { curbuf->b_op_end.lnum = (linenr_T)lnum; }

/// Set curbuf->b_op_end.col (for Rust do_put).
void nvim_dp_set_op_end_col(int col) { curbuf->b_op_end.col = (colnr_T)col; }

/// Set curbuf->b_op_end.coladd (for Rust do_put).
void nvim_dp_set_op_end_coladd(int coladd) { curbuf->b_op_end.coladd = (colnr_T)coladd; }

/// Set curwin->w_cursor = curbuf->b_visual.vi_end (for Rust do_put linewise).
void nvim_dp_set_cursor_to_b_visual_vi_end(void)
{
  curwin->w_cursor = curbuf->b_visual.vi_end;
}

/// Get curbuf->b_ml.ml_line_count (for Rust do_put).
int nvim_dp_get_ml_line_count(void) { return (int)curbuf->b_ml.ml_line_count; }

/// Call getvcol(curwin, &curwin->w_cursor, start, NULL, endcol2) (for Rust do_put).
void nvim_dp_getvcol_cursor(int *start, int *endcol2)
{
  colnr_T cs, ce;
  getvcol(curwin, &curwin->w_cursor, &cs, NULL, &ce);
  *start = (int)cs;
  *endcol2 = (int)ce;
}

/// Call getvcol(curwin, &pos, NULL, NULL, *col) (for Rust do_put forward block).
void nvim_dp_getvcol_cursor_end_only(int *col)
{
  colnr_T ce;
  getvcol(curwin, &curwin->w_cursor, NULL, NULL, &ce);
  *col = (int)ce;
}

/// Call getvcol(curwin, pos, NULL, vcol, NULL) (for Rust do_put charwise multi-line).
int nvim_dp_getvcol_pos(int lnum, int col, int coladd, int *vcol_mid)
{
  pos_T pos = { .lnum = (linenr_T)lnum, .col = (colnr_T)col, .coladd = (colnr_T)coladd };
  colnr_T cursor_col;
  getvcol(curwin, &pos, NULL, &cursor_col, NULL);
  if (vcol_mid) *vcol_mid = (int)cursor_col;
  return 0;
}

/// Call getvpos(curwin, pos, wcol) (for Rust do_put charwise multi-line).
int nvim_dp_getvpos(int *lnum, int *col, int *coladd, int wcol)
{
  pos_T pos = { .lnum = (linenr_T)*lnum, .col = (colnr_T)*col, .coladd = (colnr_T)*coladd };
  int r = getvpos(curwin, &pos, (colnr_T)wcol);
  *lnum = (int)pos.lnum;
  *col = (int)pos.col;
  *coladd = (int)pos.coladd;
  return r;
}

/// Call buf_updates_send_changes(curbuf, lnum, num_added, num_removed).
void nvim_dp_buf_updates_send_changes(int lnum, int64_t num_added, int64_t num_removed)
{
  buf_updates_send_changes(curbuf, (linenr_T)lnum, num_added, num_removed);
}

/// Call changed_lines(curbuf, lnum, col, lnume, xtra, true).
void nvim_dp_changed_lines(int lnum, int col, int lnume, int xtra)
{
  changed_lines(curbuf, (linenr_T)lnum, (colnr_T)col, (linenr_T)lnume, (linenr_T)xtra, true);
}

/// Call changed_bytes(lnum, col).
void nvim_dp_changed_bytes(int lnum, int col)
{
  changed_bytes((linenr_T)lnum, (colnr_T)col);
}

/// Call mark_adjust(line1, MAXLNUM, nr_lines, 0, kind).
void nvim_dp_mark_adjust(int line1, int nr_lines, int kind)
{
  mark_adjust((linenr_T)line1, (linenr_T)MAXLNUM, (linenr_T)nr_lines, 0, kind);
}

/// Call extmark_splice(curbuf, ...).
void nvim_dp_extmark_splice(int start_row, int start_col, int old_row, int old_col, int new_row,
                            int new_col, long totsize, int kind)
{
  extmark_splice(curbuf, start_row, (colnr_T)start_col, old_row, (colnr_T)old_col, 0,
                 new_row, (colnr_T)new_col, (bcount_t)totsize, (ExtmarkOp)kind);
}

/// Call extmark_splice_cols(curbuf, ...).
void nvim_dp_extmark_splice_cols(int start_row, int start_col, int old_col, int new_col,
                                 int lines_appended, int kind)
{
  extmark_splice_cols(curbuf, start_row, (colnr_T)start_col, (colnr_T)old_col,
                      new_col + lines_appended, (ExtmarkOp)kind);
}

/// Call terminal_paste(count, y_array, y_size).
void nvim_dp_terminal_paste(int count, String *y_array, size_t y_size)
{
  terminal_paste(count, y_array, y_size);
}

/// Return curbuf->b_p_ts.
int64_t nvim_dp_get_b_p_ts(void) { return curbuf->b_p_ts; }

/// Return curbuf->b_p_vts_array.
const int *nvim_dp_get_b_p_vts_array(void) { return curbuf->b_p_vts_array; }

/// Call tabstop_padding(col, ts, vts).
int nvim_dp_tabstop_padding(int col, int64_t ts, const int *vts)
{
  return tabstop_padding((colnr_T)col, ts, vts);
}

/// Call set_indent(size, SIN_NOMARK).
bool nvim_dp_set_indent(int size) { return set_indent(size, SIN_NOMARK); }

/// Call get_indent().
int nvim_dp_get_indent(void) { return get_indent(); }

/// Call preprocs_left().
bool nvim_dp_preprocs_left(void) { return preprocs_left(); }

/// Call beginline(BL_WHITE | BL_FIX).
void nvim_dp_beginline(void) { beginline(BL_WHITE | BL_FIX); }


/// Call hasFolding(curwin, lnum, firstp, lastp) for do_put backward fold correction.
void nvim_dp_hasFolding_backward(int *lnum)
{
  hasFolding(curwin, (linenr_T)*lnum, (linenr_T *)lnum, NULL);
}

/// Call hasFolding(curwin, lnum, NULL, lastp) for do_put forward fold correction.
void nvim_dp_hasFolding_forward(int *lnum)
{
  hasFolding(curwin, (linenr_T)*lnum, NULL, (linenr_T *)lnum);
}

/// Call buf_is_empty(curbuf).
bool nvim_dp_buf_is_empty(void) { return buf_is_empty(curbuf); }

/// Return ml_get_len(curwin->w_cursor.lnum) as int -- used by do_put blockwise.
int nvim_dp_get_cursor_line_len(void) { return (int)get_cursor_line_len(); }

/// Return get_cursor_line_ptr().
char *nvim_dp_get_cursor_line_ptr(void) { return get_cursor_line_ptr(); }

/// Call changed_cline_bef_curs(curwin).
void nvim_dp_changed_cline_bef_curs(void) { changed_cline_bef_curs(curwin); }

/// Call invalidate_botline(curwin).
void nvim_dp_invalidate_botline(void) { invalidate_botline(curwin); }

/// Call msgmore(n).
void nvim_dp_msgmore(int n) { msgmore(n); }

/// Return gchar_cursor() (UTF codepoint at cursor position).
int nvim_dp_gchar_cursor(void) { return gchar_cursor(); }


/// Return curwin->w_cursor.lnum.
int nvim_dp_get_cursor_lnum(void) { return (int)curwin->w_cursor.lnum; }

/// Set curwin->w_cursor.lnum.
void nvim_dp_set_cursor_lnum(int lnum) { curwin->w_cursor.lnum = (linenr_T)lnum; }

/// Return curwin->w_cursor.col.
int nvim_dp_get_cursor_col(void) { return (int)curwin->w_cursor.col; }

/// Set curwin->w_cursor.col.
void nvim_dp_set_cursor_col(int col) { curwin->w_cursor.col = (colnr_T)col; }

/// Set curwin->w_cursor.coladd.
void nvim_dp_set_cursor_coladd(int coladd) { curwin->w_cursor.coladd = (colnr_T)coladd; }

/// Get curwin->w_cursor.coladd.
int nvim_dp_get_cursor_coladd(void) { return (int)curwin->w_cursor.coladd; }

/// Set curwin->w_set_curswant = true.
void nvim_dp_set_curswant(void) { curwin->w_set_curswant = true; }

/// Return curwin->w_cursor (full pos_T).
void nvim_dp_get_cursor(pos_T *pos) { *pos = curwin->w_cursor; }

/// Set curwin->w_cursor (full pos_T).
void nvim_dp_set_cursor(const pos_T *pos) { curwin->w_cursor = *pos; }


/// Return b_visual.vi_start.lnum for do_put.
int nvim_dp_get_b_visual_vi_start_lnum(void) { return (int)curbuf->b_visual.vi_start.lnum; }

/// Return b_visual.vi_end.lnum for do_put.
int nvim_dp_get_b_visual_vi_end_lnum(void) { return (int)curbuf->b_visual.vi_end.lnum; }


/// Call init_charsize_arg for do_put blockwise.
bool nvim_dp_init_charsize_arg(void *csarg, int lnum, const char *line)
{
  return init_charsize_arg((CharsizeArg *)csarg, curwin, (linenr_T)lnum, line);
}

/// Return init_charsize_arg for y_array line (lnum=0 for block line charsize).
bool nvim_dp_init_charsize_arg_lnum0(void *csarg, const char *line)
{
  return init_charsize_arg((CharsizeArg *)csarg, curwin, 0, line);
}


/// Return curbuf->b_op_start.lnum.
int nvim_dp_get_op_start_lnum(void) { return (int)curbuf->b_op_start.lnum; }

/// Return e_resulting_text_too_long string.
const char *nvim_dp_get_e_resulting_text_too_long(void) { return e_resulting_text_too_long; }


/// Set curbuf->b_op_end.col += delta.
void nvim_dp_op_end_col_add(int delta) { curbuf->b_op_end.col += (colnr_T)delta; }

/// Call adjust_cursor_eol().
void nvim_dp_adjust_cursor_eol(void) { adjust_cursor_eol(); }

/// Call getviscol() for do_put virtual edit.
int nvim_dp_getviscol(void) { return getviscol(); }

/// Call coladvance_force(viscol) for do_put virtual edit.
int nvim_dp_coladvance_force(int viscol) { return coladvance_force((colnr_T)viscol); }

/// Call get_ve_flags(curwin).
unsigned nvim_dp_get_ve_flags(void) { return get_ve_flags(curwin); }

/// Call get_cursor_pos_ptr().
char *nvim_dp_get_cursor_pos_ptr(void) { return get_cursor_pos_ptr(); }

/// Call get_cursor_pos_len().
int nvim_dp_get_cursor_pos_len(void) { return (int)get_cursor_pos_len(); }

/// Call semsg for do_put E353.
void nvim_dp_semsg_E353(int regname) { semsg(_("E353: Nothing in register %s"), regname == 0 ? "\"" : transchar(regname)); }

// =============================================================================
// Phase 28: setreg / getregion / getregionpos run-helpers
//
// The three exported f_* builtins live in Rust (register crate) and call
// these thin C wrappers for the struct-heavy / VimL-plumbing operations that
// are awkward to express in Rust FFI (typval dict/list iteration, curbuf
// mutation, position arithmetic over oparg_T, …).
//
// Static helpers (get_yank_type_rs, block_def2str_rs, getregionpos) are moved
// here from funcs_shim.c; the original C bodies there are replaced by
// breadcrumb comments.
// =============================================================================

/// Helper: parse a regtype string into yank_type + block_len.
/// Returns 1 (OK) or 0 (FAIL).  *pp is advanced past the parsed characters.
static int get_yank_type_rs(char **pp, MotionType *yank_type, int *block_len)
{
  char *stropt = *pp;
  switch (*stropt) {
  case 'v':
  case 'c':
    *yank_type = kMTCharWise;
    break;
  case 'V':
  case 'l':
    *yank_type = kMTLineWise;
    break;
  case 'b':
  case Ctrl_V:
    *yank_type = kMTBlockWise;
    if (ascii_isdigit(stropt[1])) {
      stropt++;
      *block_len = getdigits_int(&stropt, false, 0) - 1;
      stropt--;
    }
    break;
  default:
    return FAIL;
  }
  *pp = stropt;
  return OK;
}

/// Convert from block_def to a heap-allocated NUL-terminated string.
static char *block_def2str_rs(struct block_def *bd)
{
  size_t size = (size_t)bd->startspaces + (size_t)bd->endspaces + (size_t)bd->textlen;
  char *ret = xmalloc(size + 1);
  char *p = ret;
  memset(p, ' ', (size_t)bd->startspaces);
  p += bd->startspaces;
  memmove(p, bd->textstart, (size_t)bd->textlen);
  p += bd->textlen;
  memset(p, ' ', (size_t)bd->endspaces);
  *(p + bd->endspaces) = NUL;
  return ret;
}

/// Core region resolver (ported from funcs_shim.c getregionpos).
/// Parses two list positions + opts dict, validates buf/line/col,
/// sets curbuf/curwin->w_buffer/virtual_op, builds oparg_T for blockwise.
/// Callers must save/restore curbuf and virtual_op.
static int getregionpos_rs(typval_T *argvars, typval_T *rettv, pos_T *p1, pos_T *p2,
                           bool *const inclusive, MotionType *region_type, oparg_T *oap)
  FUNC_ATTR_NONNULL_ALL
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (tv_check_for_list_arg(argvars, 0) == FAIL
      || tv_check_for_list_arg(argvars, 1) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 2) == FAIL) {
    return FAIL;
  }

  int fnum1 = -1;
  int fnum2 = -1;
  if (list2fpos(&argvars[0], p1, &fnum1, NULL, false) != OK
      || list2fpos(&argvars[1], p2, &fnum2, NULL, false) != OK
      || fnum1 != fnum2) {
    return FAIL;
  }

  bool is_select_exclusive;
  char *type;
  char default_type[] = "v";
  if (argvars[2].v_type == VAR_DICT) {
    is_select_exclusive = tv_dict_get_bool(argvars[2].vval.v_dict, "exclusive",
                                           *p_sel == 'e');
    type = tv_dict_get_string(argvars[2].vval.v_dict, "type", false);
    if (type == NULL) {
      type = default_type;
    }
  } else {
    is_select_exclusive = *p_sel == 'e';
    type = default_type;
  }

  int block_width = 0;
  if (type[0] == 'v' && type[1] == NUL) {
    *region_type = kMTCharWise;
  } else if (type[0] == 'V' && type[1] == NUL) {
    *region_type = kMTLineWise;
  } else if (type[0] == Ctrl_V) {
    char *p = type + 1;
    if (*p != NUL && ((block_width = getdigits_int(&p, false, 0)) <= 0 || *p != NUL)) {
      semsg(_(e_invargNval), "type", type);
      return FAIL;
    }
    *region_type = kMTBlockWise;
  } else {
    semsg(_(e_invargNval), "type", type);
    return FAIL;
  }

  buf_T *findbuf = fnum1 != 0 ? buflist_findnr(fnum1) : curbuf;
  if (findbuf == NULL || findbuf->b_ml.ml_mfp == NULL) {
    emsg(_(e_buffer_is_not_loaded));
    return FAIL;
  }

  if (p1->lnum < 1 || p1->lnum > findbuf->b_ml.ml_line_count) {
    semsg(_(e_invalid_line_number_nr), p1->lnum);
    return FAIL;
  }
  if (p1->col == MAXCOL) {
    p1->col = ml_get_buf_len(findbuf, p1->lnum) + 1;
  } else if (p1->col < 1 || p1->col > ml_get_buf_len(findbuf, p1->lnum) + 1) {
    semsg(_(e_invalid_column_number_nr), p1->col);
    return FAIL;
  }

  if (p2->lnum < 1 || p2->lnum > findbuf->b_ml.ml_line_count) {
    semsg(_(e_invalid_line_number_nr), p2->lnum);
    return FAIL;
  }
  if (p2->col == MAXCOL) {
    p2->col = ml_get_buf_len(findbuf, p2->lnum) + 1;
  } else if (p2->col < 1 || p2->col > ml_get_buf_len(findbuf, p2->lnum) + 1) {
    semsg(_(e_invalid_column_number_nr), p2->col);
    return FAIL;
  }

  curbuf = findbuf;
  curwin->w_buffer = curbuf;
  virtual_op = virtual_active(curwin);

  // NOTE: Adjustment is needed.
  p1->col--;
  p2->col--;

  if (!lt(*p1, *p2)) {
    // swap position
    pos_T p = *p1;
    *p1 = *p2;
    *p2 = p;
  }

  if (*region_type == kMTCharWise) {
    // Handle 'selection' == "exclusive".
    if (is_select_exclusive && !equalpos(*p1, *p2)) {
      // When backing up to previous line, inclusive becomes false.
      *inclusive = !unadjust_for_sel_inner(p2);
    }
    // If p2 is on NUL (end of line), inclusive becomes false.
    if (*inclusive && !virtual_op && *ml_get_pos(p2) == NUL) {
      *inclusive = false;
    }
  } else if (*region_type == kMTBlockWise) {
    colnr_T sc1, ec1, sc2, ec2;
    getvvcol(curwin, p1, &sc1, NULL, &ec1);
    getvvcol(curwin, p2, &sc2, NULL, &ec2);
    oap->motion_type = kMTBlockWise;
    oap->inclusive = true;
    oap->op_type = OP_NOP;
    oap->start = *p1;
    oap->end = *p2;
    oap->start_vcol = MIN(sc1, sc2);
    if (block_width > 0) {
      oap->end_vcol = oap->start_vcol + block_width - 1;
    } else if (is_select_exclusive && ec1 < sc2 && 0 < sc2 && ec2 > ec1) {
      oap->end_vcol = sc2 - 1;
    } else {
      oap->end_vcol = MAX(ec1, ec2);
    }
  }

  // Include the trailing byte of a multi-byte char.
  int l = utfc_ptr2len(ml_get_pos(p2));
  if (l > 1) {
    p2->col += l - 1;
  }

  return OK;
}

/// Run the full f_setreg() builtin from Rust.
/// This keeps all typval/dict/list pointer plumbing in C.
void nvim_register_setreg_run(typval_T *argvars, typval_T *rettv)
{
  bool append = false;
  int block_len = -1;
  MotionType yank_type = kMTUnknown;

  rettv->vval.v_number = 1;  // FAIL is default.

  const char *const strregname = tv_get_string_chk(argvars);
  if (strregname == NULL) {
    return;
  }
  char regname = *strregname;
  if (regname == 0 || regname == '@') {
    regname = '"';
  }

  const typval_T *regcontents = NULL;
  char pointreg = 0;
  if (argvars[1].v_type == VAR_DICT) {
    dict_T *const d = argvars[1].vval.v_dict;

    if (tv_dict_len(d) == 0) {
      char *lstval[2] = { NULL, NULL };
      write_reg_contents_lst(regname, lstval, false, kMTUnknown, -1);
      return;
    }

    dictitem_T *const di = tv_dict_find(d, "regcontents", -1);
    if (di != NULL) {
      regcontents = &di->di_tv;
    }

    const char *stropt = tv_dict_get_string(d, "regtype", false);
    if (stropt != NULL) {
      const int ret = get_yank_type_rs((char **)&stropt, &yank_type, &block_len);
      if (ret == FAIL || *(++stropt) != NUL) {
        semsg(_(e_invargval), "value");
        return;
      }
    }

    if (regname == '"') {
      stropt = tv_dict_get_string(d, "points_to", false);
      if (stropt != NULL) {
        pointreg = *stropt;
        regname = pointreg;
      }
    } else if (tv_dict_get_number(d, "isunnamed")) {
      pointreg = regname;
    }
  } else {
    regcontents = &argvars[1];
  }

  bool set_unnamed = false;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (yank_type != kMTUnknown) {
      semsg(_(e_toomanyarg), "setreg");
      return;
    }
    const char *stropt = tv_get_string_chk(&argvars[2]);
    if (stropt == NULL) {
      return;
    }
    for (; *stropt != NUL; stropt++) {
      switch (*stropt) {
      case 'a':
      case 'A':
        append = true;
        break;
      case 'u':
      case '"':
        set_unnamed = true;
        break;
      default:
        get_yank_type_rs((char **)&stropt, &yank_type, &block_len);
      }
    }
  }

  if (regcontents != NULL && regcontents->v_type == VAR_LIST) {
    list_T *const ll = regcontents->vval.v_list;
    const int len = tv_list_len(ll);
    char **lstval = xmalloc(sizeof(char *) * (((size_t)len + 1) * 2));
    const char **curval = (const char **)lstval;
    char **allocval = lstval + len + 2;
    char **curallocval = allocval;

    TV_LIST_ITER_CONST(ll, li, {
      char buf[NUMBUFLEN];
      *curval = tv_get_string_buf_chk(TV_LIST_ITEM_TV(li), buf);
      if (*curval == NULL) {
        goto free_lstval;
      }
      if (*curval == buf) {
        *curallocval = xstrdup(*curval);
        *curval = *curallocval;
        curallocval++;
      }
      curval++;
    });
    *curval++ = NULL;

    write_reg_contents_lst(regname, lstval, append, yank_type, (colnr_T)block_len);

free_lstval:
    while (curallocval > allocval) {
      xfree(*--curallocval);
    }
    xfree(lstval);
  } else if (regcontents != NULL) {
    const char *const strval = tv_get_string_chk(regcontents);
    if (strval == NULL) {
      return;
    }
    write_reg_contents_ex(regname, strval, (ssize_t)strlen(strval),
                          append, yank_type, (colnr_T)block_len);
  }

  if (pointreg != 0) {
    get_yank_register(pointreg, YREG_YANK);
  }
  rettv->vval.v_number = 0;

  if (set_unnamed) {
    op_reg_set_previous(regname);
  }
}

/// Run the full f_getregion() builtin from Rust.
void nvim_register_getregion_run(typval_T *argvars, typval_T *rettv)
{
  buf_T *const save_curbuf = curbuf;
  const TriState save_virtual = virtual_op;

  pos_T p1, p2;
  bool inclusive = true;
  MotionType region_type = kMTUnknown;
  oparg_T oa;

  if (getregionpos_rs(argvars, rettv, &p1, &p2, &inclusive, &region_type, &oa) == FAIL) {
    return;
  }

  for (linenr_T lnum = p1.lnum; lnum <= p2.lnum; lnum++) {
    char *akt = NULL;

    if (region_type == kMTLineWise) {
      akt = xstrdup(ml_get(lnum));
    } else if (region_type == kMTBlockWise) {
      struct block_def bd;
      block_prep(&oa, &bd, lnum, false);
      akt = block_def2str_rs(&bd);
    } else if (p1.lnum < lnum && lnum < p2.lnum) {
      akt = xstrdup(ml_get(lnum));
    } else {
      struct block_def bd;
      charwise_block_prep(p1, p2, &bd, lnum, inclusive);
      akt = block_def2str_rs(&bd);
    }

    assert(akt != NULL);
    tv_list_append_allocated_string(rettv->vval.v_list, akt);
  }

  curbuf = save_curbuf;
  curwin->w_buffer = curbuf;
  virtual_op = save_virtual;
}

/// Run the full f_getregionpos() builtin from Rust.
void nvim_register_getregionpos_run(typval_T *argvars, typval_T *rettv)
{
  buf_T *const save_curbuf = curbuf;
  const TriState save_virtual = virtual_op;

  pos_T p1, p2;
  bool inclusive = true;
  MotionType region_type = kMTUnknown;
  bool allow_eol = false;
  oparg_T oa;

  if (getregionpos_rs(argvars, rettv, &p1, &p2, &inclusive, &region_type, &oa) == FAIL) {
    return;
  }

  if (argvars[2].v_type == VAR_DICT) {
    allow_eol = tv_dict_get_bool(argvars[2].vval.v_dict, "eol", false);
  }

  for (linenr_T lnum = p1.lnum; lnum <= p2.lnum; lnum++) {
    pos_T ret_p1, ret_p2;
    char *line = ml_get(lnum);
    colnr_T line_len = ml_get_len(lnum);

    if (region_type == kMTLineWise) {
      ret_p1.col = 1;
      ret_p1.coladd = 0;
      ret_p2.col = MAXCOL;
      ret_p2.coladd = 0;
    } else {
      struct block_def bd;

      if (region_type == kMTBlockWise) {
        block_prep(&oa, &bd, lnum, false);
      } else {
        charwise_block_prep(p1, p2, &bd, lnum, inclusive);
      }

      if (bd.is_oneChar) {
        if (region_type == kMTBlockWise) {
          ret_p1.col = (colnr_T)(mb_prevptr(line, bd.textstart) - line) + 1;
          ret_p1.coladd = bd.start_char_vcols - (bd.start_vcol - oa.start_vcol);
        } else {
          ret_p1.col = p1.col + 1;
          ret_p1.coladd = p1.coladd;
        }
      } else if (region_type == kMTBlockWise && oa.start_vcol > bd.start_vcol) {
        ret_p1.col = MAXCOL;
        ret_p1.coladd = oa.start_vcol - bd.start_vcol;
        bd.is_oneChar = true;
      } else if (bd.startspaces > 0) {
        ret_p1.col = (colnr_T)(mb_prevptr(line, bd.textstart) - line) + 1;
        ret_p1.coladd = bd.start_char_vcols - bd.startspaces;
      } else {
        ret_p1.col = bd.textcol + 1;
        ret_p1.coladd = 0;
      }

      if (bd.is_oneChar) {
        ret_p2.col = ret_p1.col;
        ret_p2.coladd = ret_p1.coladd + bd.startspaces + bd.endspaces;
      } else if (bd.endspaces > 0) {
        ret_p2.col = bd.textcol + bd.textlen + 1;
        ret_p2.coladd = bd.endspaces;
      } else {
        ret_p2.col = bd.textcol + bd.textlen;
        ret_p2.coladd = 0;
      }
    }

    if (!allow_eol && ret_p1.col > line_len) {
      ret_p1.col = 0;
      ret_p1.coladd = 0;
    } else if (ret_p1.col > line_len + 1) {
      ret_p1.col = line_len + 1;
    }

    if (!allow_eol && ret_p2.col > line_len) {
      ret_p2.col = ret_p1.col == 0 ? 0 : line_len;
      ret_p2.coladd = 0;
    } else if (ret_p2.col > line_len + 1) {
      ret_p2.col = line_len + 1;
    }

    ret_p1.lnum = lnum;
    ret_p2.lnum = lnum;

    // Build [[buf,lnum,col,coladd],[buf,lnum,col,coladd]] entry.
    list_T *l1 = tv_list_alloc(2);
    tv_list_append_list(rettv->vval.v_list, l1);

    list_T *l2 = tv_list_alloc(4);
    tv_list_append_list(l1, l2);
    list_T *l3 = tv_list_alloc(4);
    tv_list_append_list(l1, l3);

    tv_list_append_number(l2, curbuf->b_fnum);
    tv_list_append_number(l2, ret_p1.lnum);
    tv_list_append_number(l2, ret_p1.col);
    tv_list_append_number(l2, ret_p1.coladd);

    tv_list_append_number(l3, curbuf->b_fnum);
    tv_list_append_number(l3, ret_p2.lnum);
    tv_list_append_number(l3, ret_p2.col);
    tv_list_append_number(l3, ret_p2.coladd);
  }

  curbuf = save_curbuf;
  curwin->w_buffer = curbuf;
  virtual_op = save_virtual;
}

