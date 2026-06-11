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
  return init_charsize_arg((CharsizeArg *)csarg, curwin, (linenr_T)lnum, (char *)line);  // stores in struct; callee does not mutate
}

/// Return init_charsize_arg for y_array line (lnum=0 for block line charsize).
bool nvim_dp_init_charsize_arg_lnum0(void *csarg, const char *line)
{
  return init_charsize_arg((CharsizeArg *)csarg, curwin, 0, (char *)line);  // stores in struct; callee does not mutate
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
// Phase 28: setreg / getregion / getregionpos — C accessor layer
//
// The three f_* builtins are implemented in Rust (register crate) and call
// these thin C accessors for plumbing that cannot be expressed directly in
// Rust FFI (typval dict/list iteration, curbuf/virtual_op mutation, position
// arithmetic via oparg_T, gettext message formatting).
// =============================================================================

// --- Global get/set: curbuf/virtual_op are C globals not reachable from Rust ---
buf_T *nvim_register_get_curbuf(void) { return curbuf; }
void nvim_register_set_curbuf(buf_T *buf) { curbuf = buf; }
void nvim_register_set_curwin_buffer(buf_T *buf) { curwin->w_buffer = buf; }
int nvim_register_get_virtual_op(void) { return (int)virtual_op; }
void nvim_register_set_virtual_op(int v) { virtual_op = (TriState)v; }
void nvim_register_set_virtual_op_from_curwin(void) { virtual_op = virtual_active(curwin); }

// --- Buffer / line: nested struct fields not reachable from Rust ---
bool nvim_register_buf_ml_mfp_is_null(buf_T *buf) { return buf->b_ml.ml_mfp == NULL; }
int nvim_register_buf_line_count(buf_T *buf) { return (int)buf->b_ml.ml_line_count; }
int nvim_register_curbuf_fnum(void) { return curbuf->b_fnum; }

// --- Position helper: getvvcol needs curwin ---
void nvim_register_getvvcol(pos_T *pos, int *sc, int *ec)
{
  colnr_T s, e;
  getvvcol(curwin, pos, &s, NULL, &e);
  *sc = (int)s; *ec = (int)e;
}

// --- oparg_T setters ---

/// Zero-init oparg_T, set blockwise motion fields for block_prep.
void nvim_oap_set_for_blockwise(oparg_T *oap, pos_T *start, pos_T *end,
                                int start_vcol, int end_vcol)
{
  memset(oap, 0, sizeof(*oap));
  oap->motion_type = kMTBlockWise; oap->inclusive = true; oap->op_type = OP_NOP;
  oap->start = *start; oap->end = *end;
  oap->start_vcol = (colnr_T)start_vcol; oap->end_vcol = (colnr_T)end_vcol;
}

/// Returns tv_dict_len(d) — needed because tv_dict_len is a static inline.
int nvim_register_tv_dict_len(dict_T *d) { return (int)tv_dict_len(d); }

/// Build and write a VimL list as register contents, then free the temp buffer.
/// Returns OK(0) on success, FAIL(-1) on type error.
int nvim_register_setreg_write_lst(int regname, list_T *ll, bool append,
                                   int yank_type, int block_len)
{
  const int len = tv_list_len(ll);
  char **lstval = xmalloc(sizeof(char *) * (((size_t)len + 1) * 2));
  const char **curval = (const char **)lstval;
  char **allocval = lstval + len + 2;
  char **curallocval = allocval;

  TV_LIST_ITER_CONST(ll, li, {
    char buf[NUMBUFLEN];
    *curval = tv_get_string_buf_chk(TV_LIST_ITEM_TV(li), buf);
    if (*curval == NULL) {
      while (curallocval > allocval) {
        xfree(*--curallocval);
      }
      xfree(lstval);
      return FAIL;
    }
    if (*curval == buf) {
      *curallocval = xstrdup(*curval);
      *curval = *curallocval;
      curallocval++;
    }
    curval++;
  });
  *curval = NULL;

  write_reg_contents_lst((char)regname, lstval, append,
                         (MotionType)yank_type, (colnr_T)block_len);

  while (curallocval > allocval) {
    xfree(*--curallocval);
  }
  xfree(lstval);
  return OK;
}

// --- kListLenMayKnow is a C enum; wrap the constant for Rust ---
void nvim_register_tv_list_alloc_ret(typval_T *rettv) { tv_list_alloc_ret(rettv, kListLenMayKnow); }

// --- Error message emitters (keep gettext/_()/NGETTEXT in C) ---
void nvim_register_emsg_buffer_not_loaded(void) { emsg(_(e_buffer_is_not_loaded)); }
void nvim_register_semsg_invalid_line(int lnum) { semsg(_(e_invalid_line_number_nr), (long)lnum); }
void nvim_register_semsg_invalid_col(int col) { semsg(_(e_invalid_column_number_nr), (long)col); }
void nvim_register_semsg_invargNval_type(const char *val) { semsg(_(e_invargNval), "type", val); }
void nvim_register_semsg_invargval_value(void) { semsg(_(e_invargval), "value"); }
void nvim_register_semsg_toomanyarg_setreg(void) { semsg(_(e_toomanyarg), "setreg"); }

