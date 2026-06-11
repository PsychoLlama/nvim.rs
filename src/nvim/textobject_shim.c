// textobject_shim.c: C accessor functions for the textobject Rust crate.
//
// These functions provide the "opaque handle" pattern, giving Rust code
// access to C globals and structs without exposing their internals.

#include <stdbool.h>
#include <stdint.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/fold.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/move.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/textobject.h"
#include "nvim/vim_defs.h"

#include "nvim/textobject_shim.h"
#include "textobject_shim.c.generated.h"

// Rust function declarations
extern bool rs_unadjust_for_sel(void);

// =============================================================================
// Cursor / line accessors
// =============================================================================

int nvim_textobj_get_cursor_col(void) { return curwin->w_cursor.col; }
int nvim_textobj_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }
int nvim_textobj_get_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
bool nvim_textobj_is_lineempty(int lnum) { return LINEEMPTY(lnum); }
void nvim_textobj_set_cursor_coladd_zero(void) { curwin->w_cursor.coladd = 0; }
bool nvim_textobj_hasFolding(int lnum, int *first, int *last)
{
  linenr_T first_lnum = 0;
  linenr_T last_lnum = 0;
  bool result = hasFolding(curwin, lnum, first ? &first_lnum : NULL, last ? &last_lnum : NULL);
  if (first) { *first = first_lnum; }
  if (last) { *last = last_lnum; }
  return result;
}
void nvim_textobj_set_cursor_lnum(int lnum) { curwin->w_cursor.lnum = lnum; }
void nvim_textobj_set_cursor_col(int col) { curwin->w_cursor.col = col; }
void nvim_textobj_unadjust_for_sel_if_needed(void)
{
  if (*p_sel == 'e' && VIsual_active && VIsual_mode == 'v' && VIsual_select_exclu_adj) {
    rs_unadjust_for_sel();
  }
}

// =============================================================================
// Accessors for current_word function
// =============================================================================

bool nvim_textobj_get_VIsual_active(void) { return VIsual_active; }
int nvim_textobj_get_VIsual_lnum(void) { return VIsual.lnum; }
int nvim_textobj_get_VIsual_col(void) { return VIsual.col; }
int nvim_textobj_get_VIsual_mode(void) { return VIsual_mode; }
void nvim_textobj_set_VIsual_mode(int mode) { VIsual_mode = mode; }
void nvim_textobj_set_VIsual(int lnum, int col) { VIsual.lnum = lnum; VIsual.col = col; }
int nvim_textobj_get_p_sel_first(void) { return *p_sel; }
bool nvim_textobj_lt_cursor_VIsual(void) { return lt(curwin->w_cursor, VIsual); }
bool nvim_textobj_equalpos_cursor_VIsual(void) { return equalpos(curwin->w_cursor, VIsual); }
bool nvim_textobj_lt_VIsual_cursor(void) { return lt(VIsual, curwin->w_cursor); }
bool nvim_textobj_ltoreq_VIsual_cursor(void) { return ltoreq(VIsual, curwin->w_cursor); }
void nvim_textobj_set_oap_start_from_cursor(oparg_T *oap) { oap->start = curwin->w_cursor; }
void nvim_textobj_set_oap_motion_type(oparg_T *oap, int type) { oap->motion_type = type; }
void nvim_textobj_set_oap_inclusive(oparg_T *oap, bool val) { oap->inclusive = val; }
int nvim_textobj_incl_cursor(void) { return incl(&curwin->w_cursor); }
int nvim_textobj_decl_cursor(void) { return decl(&curwin->w_cursor); }
void nvim_textobj_set_redraw_cmdline(bool val) { redraw_cmdline = val; }
void nvim_textobj_get_cursor_pos(int *lnum, int *col)
{
  *lnum = curwin->w_cursor.lnum;
  *col = curwin->w_cursor.col;
}
void nvim_textobj_set_cursor_pos(int lnum, int col)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
}
void nvim_textobj_set_VIsual_from_cursor(void) { VIsual = curwin->w_cursor; }
void nvim_textobj_set_oap_start(oparg_T *oap, int lnum, int col)
{
  oap->start.lnum = lnum;
  oap->start.col = col;
}

// =============================================================================
// Accessors for paragraph functions
// =============================================================================

char *nvim_textobj_get_p_sections(void) { return p_sections; }
char *nvim_textobj_get_p_para(void) { return p_para; }

// =============================================================================
// Accessors for sentence/block functions
// =============================================================================

bool nvim_textobj_equalpos(pos_T *a, pos_T *b) { return equalpos(*a, *b); }
bool nvim_textobj_lt_pos(pos_T *a, pos_T *b) { return lt(*a, *b); }
char *nvim_textobj_get_p_cpo(void) { return p_cpo; }
pos_T *nvim_textobj_get_cursor_ptr(void) { return &curwin->w_cursor; }
pos_T *nvim_textobj_get_VIsual_ptr(void) { return &VIsual; }
void nvim_textobj_copy_pos(pos_T *dst, pos_T *src) { *dst = *src; }
int nvim_textobj_pos_get_lnum(pos_T *pos) { return (int)pos->lnum; }
int nvim_textobj_pos_get_col(pos_T *pos) { return pos->col; }
void nvim_textobj_pos_set_lnum(pos_T *pos, int lnum) { pos->lnum = lnum; }
void nvim_textobj_pos_set_col(pos_T *pos, int col) { pos->col = col; }
bool nvim_textobj_lineempty(int lnum) { return LINEEMPTY(lnum); }
bool nvim_textobj_ascii_iswhite(int c) { return ascii_iswhite(c); }
pos_T *nvim_textobj_alloc_pos(void)
{
  pos_T *p = xmalloc(sizeof(pos_T));
  clearpos(p);
  return p;
}

void nvim_textobj_set_cursor_from_pos(pos_T *pos) { curwin->w_cursor = *pos; }

static char *saved_p_cpo = NULL;
void nvim_textobj_set_p_cpo_temp(const char *val) { saved_p_cpo = p_cpo; p_cpo = (char *)val; }
void nvim_textobj_restore_p_cpo(void)
{
  if (saved_p_cpo != NULL) { p_cpo = saved_p_cpo; saved_p_cpo = NULL; }
}
bool nvim_textobj_cpo_has_matchbsl(void) { return vim_strchr(p_cpo, CPO_MATCHBSL) != NULL; }
bool nvim_textobj_ltoreq_pos(pos_T *a, pos_T *b) { return ltoreq(*a, *b); }

// =============================================================================
// Accessors for tag functions
// =============================================================================

void nvim_textobj_mb_ptr_back(const char *base, char **p) { MB_PTR_BACK(base, *p); }
void nvim_textobj_mb_ptr_adv(char **p) { MB_PTR_ADV(*p); }

// =============================================================================
// Accessor for current_quote (Phase 1)
// =============================================================================

/// Returns curbuf->b_p_qe (the 'quoteescape' option string).
const char *nvim_textobj_get_curbuf_qe(void) { return curbuf->b_p_qe; }

// =============================================================================
// Accessors for current_tagblock (Phase 2)
// =============================================================================

#include "nvim/eval/funcs.h"

/// Get p_ws (wrapscan option).
bool nvim_textobj_get_p_ws(void) { return p_ws; }

/// Set p_ws (wrapscan option).
void nvim_textobj_set_p_ws(bool val) { p_ws = val; }

/// Returns the byte at the current cursor position.
int nvim_textobj_get_cursor_char(void) { return (unsigned char)*get_cursor_pos_ptr(); }

/// Wrapper for do_searchpair with NULL skip/match_pos and zero flags.
/// Used by current_tagblock to avoid exposing typval_T in Rust FFI.
int nvim_textobj_do_searchpair(const char *spat, const char *mpat, const char *epat, int dir)
{
  return do_searchpair(spat, mpat, epat, dir, NULL, 0, NULL, 0, 0);
}

