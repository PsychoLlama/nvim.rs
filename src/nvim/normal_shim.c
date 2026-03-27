//
// normal.c:    Contains the main routine for processing characters in command
//              mode.  Communicates closely with the code in ops.c to handle
//              the operators.
//

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/help.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/plines.h"
#include "nvim/profile.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/textformat.h"
#include "nvim/textobject.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

typedef struct {
  VimState state;
  bool command_finished;
  bool ctrl_w;
  bool need_flushbuf;
  bool set_prevcount;
  bool previous_got_int;             // `got_int` was true
  bool cmdwin;                       // command-line window normal mode
  bool noexmode;                     // true if the normal mode was pushed from
                                     // ex mode (:global or :visual for example)
  bool toplevel;                     // top-level normal mode
  oparg_T oa;                        // operator arguments
  cmdarg_T ca;                       // command arguments
  int mapped_len;
  int old_mapped_len;
  int idx;
  int c;
  int old_col;
  pos_T old_pos;
} NormalState;

#include "normal_shim.c.generated.h"

static inline void normal_state_init(NormalState *s) { memset(s, 0, sizeof(NormalState)); s->state.check = normal_check; s->state.execute = normal_execute; }

// nv_*(): functions called to handle Normal and Visual mode commands.
// n_*(): functions called to handle Normal mode commands.
// v_*(): functions called to handle Visual mode commands.

// Normal mode state machine
extern int rs_normal_check(void *s);
extern int rs_normal_execute(void *s, int key);
extern void rs_normal_prepare(void *s);

// Diff and scrollbind helpers (called from nvim_scrollbind_sync_windows)
extern void rs_diff_set_topline(win_T *fromwin, win_T *towin);
extern int rs_get_vtopline(win_T *wp);

// Ident helper (called from rs_find_ident_under_cursor)
extern size_t rs_find_ident_at_pos(win_T *wp, linenr_T lnum, colnr_T startcol,
                                   char **text, int *textcol, int find_type);

extern void invoke_edit(cmdarg_T *cap, int repl, int cmd, int startln);
extern void del_from_showcmd(int len);

oparg_T *nvim_current_oap = NULL;

int nvim_get_opcount(void) { return opcount; }

void nvim_set_opcount(int val) { opcount = val; }

void nvim_set_motion_force(int val) { motion_force = val; }

bool nvim_get_VIsual_select(void) { return VIsual_select; }

void nvim_set_VIsual_select(bool val) { VIsual_select = val; }

void nvim_curwin_set_curswant(bool val) { curwin->w_set_curswant = val; }

linenr_T nvim_get_line_count(void) { return curbuf->b_ml.ml_line_count; }

linenr_T nvim_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }

void nvim_set_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }

bool nvim_get_KeyTyped(void) { return KeyTyped; }

/// Get fdo_flags global.
unsigned int nvim_get_fdo_flags(void)
{
  // Guards: ensure Rust constants match C kOptFdoFlag* values
  _Static_assert(kOptFdoFlagHor == 0x04,
                 "kOptFdoFlagHor changed - update K_OPT_FDO_FLAG_HOR in normal/src/lib.rs");
  _Static_assert(kOptFdoFlagBlock == 0x02,
                 "kOptFdoFlagBlock changed - update K_OPT_FDO_FLAG_BLOCK in normal/src/lib.rs");
  _Static_assert(kOptFdoFlagJump == 0x400,
                 "kOptFdoFlagJump changed - update K_OPT_FDO_FLAG_JUMP in normal/src/lib.rs");
  _Static_assert(kOptFdoFlagMark == 0x08,
                 "kOptFdoFlagMark changed - update K_OPT_FDO_FLAG_MARK in normal/src/lib.rs");
  return fdo_flags;
}

void nvim_set_ins_at_eol(bool val) { ins_at_eol = val; }

void nvim_set_curswant(colnr_T val) { curwin->w_curswant = val; }

bool nvim_virtual_active(void) { return virtual_active(curwin); }

int nvim_gchar_cursor(void) { return utf_ptr2char(get_cursor_pos_ptr()); }

void nvim_coladvance(colnr_T col) { coladvance(curwin, col); }

// cmdarg_T accessors for Rust FFI

// Word motion accessors for Rust FFI

int nvim_get_cursor_col(void) { return curwin->w_cursor.col; }

void nvim_set_cursor_col(int col) { curwin->w_cursor.col = col; }

int nvim_inc_cursor(void) { return inc(&curwin->w_cursor); }

int nvim_dec_cursor(void) { return dec(&curwin->w_cursor); }

unsigned int nvim_get_ve_flags(void) { return get_ve_flags(curwin); }

// Character search accessors for Rust FFI

int nvim_get_VIsual_mode(void) { return VIsual_mode; }

void nvim_getvcol_cursor(int *scol, int *ecol) { getvcol(curwin, &curwin->w_cursor, scol, NULL, ecol); }

void nvim_set_cursor_coladd(int val) { curwin->w_cursor.coladd = val; }

_Static_assert(TAB == 0x09, "TAB changed");

// Mark command accessors for Rust FFI

fmark_T *nvim_mark_get(int name) { return mark_get(curbuf, curwin, NULL, kMarkAll, name); }

fmark_T *nvim_get_changelist(int count1) { return get_changelist(curbuf, curwin, count1); }

fmark_T *nvim_get_jumplist(int count1) { return get_jumplist(curwin, count1); }

int nvim_get_changelistlen(void) { return curbuf->b_changelistlen; }

void nvim_emsg(const char *msg) { emsg(msg); }

// Register command accessors for Rust FFI

// nv_put C accessors
bool nvim_put_get_save_fen(void) { return curwin->w_p_fen; }
void nvim_put_do_put(int regname, void *savereg, int dir, int count, int flags) { do_put(regname, (yankreg_T *)savereg, dir, count, flags); }
void nvim_put_free_register(void *savereg) { if (savereg != NULL) { free_register((yankreg_T *)savereg); xfree(savereg); } }

// Put/replace helper accessors for Rust FFI

int nvim_get_b_prompt_start_lnum_put(void) { return curbuf->b_prompt_start.mark.lnum; }
void nvim_set_cursor_col_to_prompt_text_len(void) { curwin->w_cursor.col = (int)strlen(prompt_text()); }
void nvim_set_w_p_fen(bool val) { curwin->w_p_fen = val; }
void nvim_inc_msg_silent(void) { msg_silent++; }
bool nvim_curbuf_ml_empty(void) { return (curbuf->b_ml.ml_flags & ML_EMPTY) != 0; }
int nvim_get_cursor_col_vs_b_op_start_col(void) { return curwin->w_cursor.col - curbuf->b_op_start.col; }
int nvim_get_cursor_lnum_vs_b_op_start_lnum(void) { return (int)(curwin->w_cursor.lnum - curbuf->b_op_start.lnum); }
void nvim_set_b_visual_from_op(void) { curbuf->b_visual.vi_start = curbuf->b_op_start; curbuf->b_visual.vi_end = curbuf->b_op_end; }
void nvim_inc_b_visual_vi_end(void) { inc(&curbuf->b_visual.vi_end); }
void nvim_ml_delete_last_line(void) { ml_delete_flags(curbuf->b_ml.ml_line_count, ML_DEL_MESSAGE); deleted_lines(curbuf->b_ml.ml_line_count + 1, 1); }

bool nvim_curbuf_b_p_et(void) { return curbuf->b_p_et; }
void nvim_set_b_op_start_cursor(void) { curbuf->b_op_start = curwin->w_cursor; }

// Visual mode accessors for Rust FFI

void nvim_set_finish_op(bool val) { finish_op = val; }

void nvim_set_VIsual_mode(int val) { VIsual_mode = val; }

void nvim_redraw_curbuf_inverted(void) { redraw_curbuf_later(UPD_INVERTED); }

int nvim_get_resel_VIsual_mode(void) { return resel_VIsual_mode; }

int nvim_get_resel_VIsual_line_count(void) { return resel_VIsual_line_count; }

int nvim_get_resel_VIsual_vcol(void) { return resel_VIsual_vcol; }

void nvim_set_VIsual_lnum(int lnum) { VIsual.lnum = lnum; }

void nvim_set_VIsual_col(int col) { VIsual.col = col; }

void nvim_set_VIsual_coladd(int coladd) { VIsual.coladd = coladd; }

void nvim_set_VIsual_active(bool val) { VIsual_active = val; }

void nvim_set_VIsual_reselect(bool val) { VIsual_reselect = val; }

void nvim_check_cursor(void) { check_cursor(curwin); }

int nvim_get_curswant(void) { return curwin->w_curswant; }

_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL changed");

/// Clear b_syn_slow for all windows in current tab (for nv_clear).
void nvim_clear_b_syn_slow_all_windows(void) {
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_s->b_syn_slow = false;
  }
}

/// syn_stack_free_all(curwin->w_s) wrapper.
void nvim_syn_stack_free_all_curwin(void) { syn_stack_free_all(curwin->w_s); }

_Static_assert(GETF_SETMARK == 0x01, "GETF_SETMARK changed");
_Static_assert(GETF_ALT == 0x02, "GETF_ALT changed");

int nvim_get_curbuf_visual_vi_mode(void) { return curbuf->b_visual.vi_mode; }

void nvim_set_curbuf_visual_vi_mode(int val) { curbuf->b_visual.vi_mode = val; }

bool nvim_get_mode_displayed(void) { return mode_displayed; }
void nvim_set_mode_displayed(bool val) { mode_displayed = val; }

void nvim_set_clear_cmdline(bool val) { clear_cmdline = val; }

// Visual operator accessors for Rust FFI
_Static_assert(Ctrl_V == 22, "Ctrl_V mismatch");
_Static_assert(OP_DELETE == 1, "OP_DELETE mismatch");
_Static_assert(OP_YANK == 2, "OP_YANK mismatch");
_Static_assert(OP_LSHIFT == 4, "OP_LSHIFT mismatch");
_Static_assert(OP_RSHIFT == 5, "OP_RSHIFT mismatch");
_Static_assert(BL_WHITE == 1, "BL_WHITE mismatch");
_Static_assert(K_DEL == TERMCAP2KEY('k', 'D'), "K_DEL mismatch");
_Static_assert(K_KDEL == TERMCAP2KEY(KS_EXTRA, KE_KDEL), "K_KDEL mismatch");
_Static_assert(kMTLineWise == 1, "kMTLineWise mismatch");

// Selection/g-cmd accessors for Rust FFI
_Static_assert(Ctrl_N == 14, "Ctrl_N mismatch");
_Static_assert(Ctrl_G == 7, "Ctrl_G mismatch");
_Static_assert(Ctrl_C == 3, "Ctrl_C mismatch");
_Static_assert(kMTCharWise == 0, "kMTCharWise mismatch");

bool nvim_set_cursor_from_last_insert(void) { if (curbuf->b_last_insert.mark.lnum != 0) { curwin->w_cursor = curbuf->b_last_insert.mark; return true; } return false; }

void nvim_check_cursor_lnum_call(void) { check_cursor_lnum(curwin); }

int nvim_get_cursor_line_len(void) { return (int)get_cursor_line_len(); }

int nvim_get_cursor_coladd(void) { return curwin->w_cursor.coladd; }

void nvim_set_cmdwin_result(int val) { cmdwin_result = val; }

// Guards: ensure Rust constants match C values
_Static_assert(kOptFdoFlagPercent == 0x10,
               "kOptFdoFlagPercent changed - update K_OPT_FDO_FLAG_PERCENT in normal/src/lib.rs");
_Static_assert(BL_SOL == 2,
               "BL_SOL changed - update BL_SOL in normal/src/lib.rs");
_Static_assert(BL_FIX == 4,
               "BL_FIX changed - update BL_FIX in normal/src/lib.rs");
_Static_assert(UPD_INVERTED == 20,
               "UPD_INVERTED changed - update UPD_INVERTED in normal/src/lib.rs");

void nvim_set_VIsual_pos(int lnum, int col, int coladd) { VIsual.lnum = lnum; VIsual.col = col; VIsual.coladd = coladd; }

void nvim_set_cursor_pos(int lnum, int col, int coladd) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; curwin->w_cursor.coladd = coladd; }

int nvim_get_b_visual_vi_start_lnum(void) { return curbuf->b_visual.vi_start.lnum; }

int nvim_get_b_visual_vi_start_col(void) { return curbuf->b_visual.vi_start.col; }

int nvim_get_b_visual_vi_start_coladd(void) { return curbuf->b_visual.vi_start.coladd; }

void nvim_set_b_visual_vi_start(int lnum, int col, int coladd) { curbuf->b_visual.vi_start.lnum = lnum; curbuf->b_visual.vi_start.col = col; curbuf->b_visual.vi_start.coladd = coladd; }

int nvim_get_b_visual_vi_end_lnum(void) { return curbuf->b_visual.vi_end.lnum; }

int nvim_get_b_visual_vi_end_col(void) { return curbuf->b_visual.vi_end.col; }

int nvim_get_b_visual_vi_end_coladd(void) { return curbuf->b_visual.vi_end.coladd; }

void nvim_set_b_visual_vi_end(int lnum, int col, int coladd) { curbuf->b_visual.vi_end.lnum = lnum; curbuf->b_visual.vi_end.col = col; curbuf->b_visual.vi_end.coladd = coladd; }

int nvim_get_b_visual_vi_curswant(void) { return curbuf->b_visual.vi_curswant; }

void nvim_set_b_visual_vi_curswant(int val) { curbuf->b_visual.vi_curswant = val; }

void nvim_set_curbuf_visual_mode_eval(int val) { curbuf->b_visual_mode_eval = val; }

int nvim_get_VIsual_select_reg(void) { return VIsual_select_reg; }
int nvim_get_virtual_op(void) { return (int)virtual_op; }

bool nvim_p_sel_is_exclusive(void) { return *p_sel == 'e'; }

/// Wrapper for getvcols: takes two positions, returns left/right via out-params.

/// mark_mb_adjustpos for arbitrary pos (by lnum/col/coladd).
/// Updates *col_out after adjustment and returns new col.
int nvim_mark_mb_adjustpos_pos(int lnum, int col, int *col_out) { pos_T pp = { lnum, (colnr_T)col, 0 }; mark_mb_adjustpos(curbuf, &pp); *col_out = pp.col; return pp.col; }

/// getvcol coladd (ce - cs) for arbitrary pos.
int nvim_getvcol_pos_coladd(int lnum, int col, int coladd) { pos_T pp = { lnum, (colnr_T)col, (colnr_T)coladd }; colnr_T cs, ce; getvcol(curwin, &pp, &cs, NULL, &ce); return (int)(ce - cs); }

/// Call getcmdline for search and set cap->searchbuf. Returns the searchbuf (or NULL).
char *nvim_getcmdline_for_search(cmdarg_T *cap) { cap->searchbuf = getcmdline(cap->cmdchar, cap->count1, 0, true); return cap->searchbuf; }

/// Wrapper for searchit using curwin/curbuf cursor (for find_decl pattern).
/// Returns 1 on success, 0 on failure.
int nvim_searchit_decl(const char *pat, size_t patlen, int searchflags) { return searchit(curwin, curbuf, &curwin->w_cursor, NULL, FORWARD, (char *)pat, patlen, 1, searchflags, RE_LAST, NULL); }

// Accessors for operator Rust implementations
bool nvim_bt_prompt_curbuf(void) { return bt_prompt(curbuf); }

/// pos_to_mark(curbuf, NULL, curwin->w_cursor) -- returns fmark_T*.
fmark_T *nvim_pos_to_mark_cursor(void) { return pos_to_mark(curbuf, NULL, curwin->w_cursor); }

// g-command and n_opencmd accessors for Rust FFI

int nvim_get_curwin_w_skipcol(void) { return (int)curwin->w_skipcol; }
int nvim_get_curwin_w_topline(void) { return (int)curwin->w_topline; }
bool nvim_get_curwin_w_cline_folded(void) { return curwin->w_cline_folded; }
void nvim_clear_curwin_w_valid_wcol(void) { curwin->w_valid &= ~VALID_WCOL; }
int nvim_getvvcol_cursor_end(void) { colnr_T vcol; getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol); return (int)vcol; }
void nvim_hasFolding_cursor_set_lnum_up(void) { hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL); }
void nvim_hasFolding_cursor_set_lnum_down(void) { hasFolding(curwin, curwin->w_cursor.lnum, NULL, &curwin->w_cursor.lnum); }
void nvim_set_curbuf_b_last_changedtick_i(void) { curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf); }
void nvim_clear_curwin_w_valid_crow(void) { curwin->w_valid &= ~VALID_CROW; }
/// mark_mb_adjustpos for cursor: adjusts curwin->w_cursor via curbuf, returns new col.
int nvim_mark_mb_adjustpos_cursor_new(void) { mark_mb_adjustpos(curbuf, &curwin->w_cursor); return curwin->w_cursor.col; }
/// getvcol for cursor pos after mark_mb_adjustpos_cursor: returns coladd = ce - cs.
int nvim_getvcol_cursor_coladd_after_adj(void) { colnr_T cs, ce; getvcol(curwin, &curwin->w_cursor, &cs, NULL, &ce); return (int)(ce - cs); }
/// mark_mb_adjustpos for VIsual: adjusts VIsual via curbuf, returns new col.
int nvim_mark_mb_adjustpos_visual_new(void) { mark_mb_adjustpos(curbuf, &VIsual); return VIsual.col; }
/// getvcol for VIsual pos: returns coladd = ce - cs.
int nvim_getvcol_visual_coladd_after_adj(void) { colnr_T cs, ce; getvcol(curwin, &VIsual, &cs, NULL, &ce); return (int)(ce - cs); }
bool nvim_curbuf_modifiable(void) { return MODIFIABLE(curbuf); }
// z-command accessors for Rust FFI
int nvim_get_curwin_w_p_fdl(void) { return (int)curwin->w_p_fdl; }
void nvim_set_curwin_w_p_fdl(int val) { curwin->w_p_fdl = val; }
void nvim_set_curwin_w_foldinvalid(bool val) { curwin->w_foldinvalid = val; }
int nvim_get_curwin_w_view_width(void) { return curwin->w_view_width; }
int nvim_get_curwin_w_leftcol(void) { return curwin->w_leftcol; }
void nvim_set_curwin_w_leftcol(int val) { curwin->w_leftcol = val; }
void nvim_validate_botline_curwin(void) { validate_botline(curwin); }
int nvim_get_curwin_w_botline(void) { return curwin->w_botline; }
bool nvim_hasFolding_curwin(int lnum) { return hasFolding(curwin, lnum, NULL, NULL); }
void nvim_getvcol_curwin_cursor(int *vcol) { getvcol(curwin, &curwin->w_cursor, vcol, NULL, NULL); }
void nvim_getvcol_curwin_cursor_end(int *vcol) { getvcol(curwin, &curwin->w_cursor, NULL, NULL, vcol); }
bool nvim_get_curwin_w_p_wrap(void) { return curwin->w_p_wrap; }

/// Wrapper for ml_get_pos(&curwin->w_cursor) for Rust FFI.
char *nvim_ml_get_pos_cursor(void) { return ml_get_pos(&curwin->w_cursor); }

/// Sync w_p_fen in diff-synced windows for 'z' commands.
void nvim_sync_fen_in_diff_windows(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp != curwin && rs_foldmethodIsDiff(wp) && wp->w_p_scb) {
      wp->w_p_fen = curwin->w_p_fen;
      changed_window_setting(wp);
    }
  }
}

// g-command C accessors for Rust FFI
void nvim_set_oap_cursor_start(oparg_T *oap) { oap->cursor_start = curwin->w_cursor; }
// nv_screengo C accessors for Rust FFI
int nvim_get_curwin_w_virtcol(void) { return curwin->w_virtcol; }
int nvim_get_curwin_ml_line_count(void) { return curwin->w_buffer->b_ml.ml_line_count; }

static char *nvim_mps_save = NULL;
void nvim_save_and_set_mps(void) { nvim_mps_save = curbuf->b_p_mps; curbuf->b_p_mps = "(:),{:},[:],<:>"; }
void nvim_restore_mps(void) { curbuf->b_p_mps = nvim_mps_save; }

void nvim_u_clearline_curbuf(void) { u_clearline(curbuf); }
void nvim_changed_lines_call(int lnum, int col, int lnum_end, bool do_concealed) { changed_lines(curbuf, (linenr_T)lnum, (colnr_T)col, (linenr_T)lnum_end, 0, do_concealed); }
void nvim_set_b_op_start(int lnum, int col, int coladd) { curbuf->b_op_start.lnum = (linenr_T)lnum; curbuf->b_op_start.col = (colnr_T)col; curbuf->b_op_start.coladd = (colnr_T)coladd; }
void nvim_set_b_op_end_cursor(void) { curbuf->b_op_end = curwin->w_cursor; }
void nvim_dec_b_op_end_col(void) { if (curbuf->b_op_end.col > 0) curbuf->b_op_end.col--; }

// find_ident_at_pos accessors for Rust FFI

/// Constants for find_ident_at_pos (verified with _Static_assert).
_Static_assert(FIND_IDENT == 1, "FIND_IDENT changed");
_Static_assert(FIND_STRING == 2, "FIND_STRING changed");
_Static_assert(FIND_EVAL == 4, "FIND_EVAL changed");
_Static_assert(BACKWARD == -1, "BACKWARD changed");
_Static_assert(FORWARD == 1, "FORWARD changed");

/// Normal state entry point. This is called on:
///
/// - Startup, In this case the function never returns.
/// - The command-line window is opened (`q:`). Returns when `cmdwin_result` != 0.
/// - The :visual command is called from :global in ex mode, `:global/PAT/visual`
///   for example. Returns when re-entering ex mode (because ex mode recursion is
///   not allowed)
///
/// This used to be called main_loop() on main.c
void normal_enter(bool cmdwin, bool noexmode)
{
  NormalState state;
  normal_state_init(&state);
  oparg_T *prev_oap = nvim_current_oap;
  nvim_current_oap = &state.oa;
  state.cmdwin = cmdwin;
  state.noexmode = noexmode;
  state.toplevel = (!cmdwin || cmdwin_result == 0) && !noexmode;
  state_enter(&state.state);
  nvim_current_oap = prev_oap;
}

// normal_get_additional_char accessors for Rust FFI

_Static_assert(MODE_REPLACE == 0x110, "MODE_REPLACE changed");
_Static_assert(MODE_LREPLACE == 0x120, "MODE_LREPLACE changed");
_Static_assert(MODE_LANGMAP == 0x20, "MODE_LANGMAP changed");
_Static_assert(MODE_NORMAL_BUSY == 0x1001, "MODE_NORMAL_BUSY changed");
_Static_assert(B_IMODE_LMAP == 1, "B_IMODE_LMAP changed");
_Static_assert(CPO_DIGRAPH == 'D', "CPO_DIGRAPH changed");

int nvim_langmap_adjust(int c, bool condition) { LANGMAP_ADJUST(c, condition); return c; }

/// Increment no_mapping.
void nvim_inc_no_mapping(void) { no_mapping++; }
/// Decrement no_mapping.
void nvim_dec_no_mapping(void) { no_mapping--; }

/// Increment allow_keys.
void nvim_inc_allow_keys(void) { allow_keys++; }
/// Decrement allow_keys.
void nvim_dec_allow_keys(void) { allow_keys--; }

/// Set did_cursorhold.
void nvim_set_did_cursorhold(bool val) { did_cursorhold = val; }

/// Get curbuf->b_p_iminsert.
int nvim_get_curbuf_b_p_iminsert(void) { return curbuf->b_p_iminsert; }

// nvim_set_State and nvim_get_State are in window.c

/// Check vim_strchr(p_cpo, c) != NULL.
bool nvim_vim_strchr_p_cpo(int c) { return vim_strchr(p_cpo, c) != NULL; }

/// Wrapper for get_op_type.
/// Get MB_BYTE2LEN for a character.
int nvim_get_MB_BYTE2LEN(int c) { return MB_BYTE2LEN(c); }

// normal_finish_command accessors for Rust FFI

_Static_assert(K_IGNORE == -13821, "K_IGNORE changed");
_Static_assert(K_MOUSEMOVE == -25853, "K_MOUSEMOVE changed");
_Static_assert(K_EVENT == -26365, "K_EVENT changed");
_Static_assert(OP_NOP == 0, "OP_NOP changed");
_Static_assert(OP_COLON == 10, "OP_COLON changed");
_Static_assert(CA_COMMAND_BUSY == 1, "CA_COMMAND_BUSY changed");

// Layout guards for repr(C) struct mirrors in src/nvim-rs/normal/src/types.rs

_Static_assert(sizeof(pos_T) == 12, "pos_T size changed");
_Static_assert(offsetof(pos_T, lnum) == 0, "pos_T.lnum offset changed");
_Static_assert(offsetof(pos_T, col) == 4, "pos_T.col offset changed");
_Static_assert(offsetof(pos_T, coladd) == 8, "pos_T.coladd offset changed");

_Static_assert(sizeof(oparg_T) == 84, "oparg_T size changed - update OpargT in types.rs");
_Static_assert(offsetof(oparg_T, op_type) == 0, "oparg_T.op_type offset changed");
_Static_assert(offsetof(oparg_T, regname) == 4, "oparg_T.regname offset changed");
_Static_assert(offsetof(oparg_T, motion_type) == 8, "oparg_T.motion_type offset changed");
_Static_assert(offsetof(oparg_T, motion_force) == 12, "oparg_T.motion_force offset changed");
_Static_assert(offsetof(oparg_T, use_reg_one) == 16, "oparg_T.use_reg_one offset changed");
_Static_assert(offsetof(oparg_T, inclusive) == 17, "oparg_T.inclusive offset changed");
_Static_assert(offsetof(oparg_T, end_adjusted) == 18, "oparg_T.end_adjusted offset changed");
_Static_assert(offsetof(oparg_T, start) == 20, "oparg_T.start offset changed");
_Static_assert(offsetof(oparg_T, end) == 32, "oparg_T.end offset changed");
_Static_assert(offsetof(oparg_T, cursor_start) == 44, "oparg_T.cursor_start offset changed");
_Static_assert(offsetof(oparg_T, line_count) == 56, "oparg_T.line_count offset changed");
_Static_assert(offsetof(oparg_T, empty) == 60, "oparg_T.empty offset changed");
_Static_assert(offsetof(oparg_T, is_VIsual) == 61, "oparg_T.is_VIsual offset changed");
_Static_assert(offsetof(oparg_T, start_vcol) == 64, "oparg_T.start_vcol offset changed");
_Static_assert(offsetof(oparg_T, end_vcol) == 68, "oparg_T.end_vcol offset changed");
_Static_assert(offsetof(oparg_T, prev_opcount) == 72, "oparg_T.prev_opcount offset changed");
_Static_assert(offsetof(oparg_T, prev_count0) == 76, "oparg_T.prev_count0 offset changed");
_Static_assert(offsetof(oparg_T, excl_tr_ws) == 80, "oparg_T.excl_tr_ws offset changed");

_Static_assert(sizeof(cmdarg_T) == 88, "cmdarg_T size changed - update CmdargT in types.rs");
_Static_assert(offsetof(cmdarg_T, oap) == 0, "cmdarg_T.oap offset changed");
_Static_assert(offsetof(cmdarg_T, prechar) == 8, "cmdarg_T.prechar offset changed");
_Static_assert(offsetof(cmdarg_T, cmdchar) == 12, "cmdarg_T.cmdchar offset changed");
_Static_assert(offsetof(cmdarg_T, nchar) == 16, "cmdarg_T.nchar offset changed");
_Static_assert(offsetof(cmdarg_T, nchar_composing) == 20, "cmdarg_T.nchar_composing offset changed");
_Static_assert(offsetof(cmdarg_T, nchar_len) == 52, "cmdarg_T.nchar_len offset changed");
_Static_assert(offsetof(cmdarg_T, extra_char) == 56, "cmdarg_T.extra_char offset changed");
_Static_assert(offsetof(cmdarg_T, opcount) == 60, "cmdarg_T.opcount offset changed");
_Static_assert(offsetof(cmdarg_T, count0) == 64, "cmdarg_T.count0 offset changed");
_Static_assert(offsetof(cmdarg_T, count1) == 68, "cmdarg_T.count1 offset changed");
_Static_assert(offsetof(cmdarg_T, arg) == 72, "cmdarg_T.arg offset changed");
_Static_assert(offsetof(cmdarg_T, retval) == 76, "cmdarg_T.retval offset changed");
_Static_assert(offsetof(cmdarg_T, searchbuf) == 80, "cmdarg_T.searchbuf offset changed");

_Static_assert(sizeof(VimState) == 16, "VimState size changed - update VimState in types.rs");

_Static_assert(sizeof(NormalState) == 232, "NormalState size changed - update NormalState in types.rs");
_Static_assert(offsetof(NormalState, state) == 0, "NormalState.state offset changed");
_Static_assert(offsetof(NormalState, command_finished) == 16, "NormalState.command_finished offset changed");
_Static_assert(offsetof(NormalState, ctrl_w) == 17, "NormalState.ctrl_w offset changed");
_Static_assert(offsetof(NormalState, need_flushbuf) == 18, "NormalState.need_flushbuf offset changed");
_Static_assert(offsetof(NormalState, set_prevcount) == 19, "NormalState.set_prevcount offset changed");
_Static_assert(offsetof(NormalState, previous_got_int) == 20, "NormalState.previous_got_int offset changed");
_Static_assert(offsetof(NormalState, cmdwin) == 21, "NormalState.cmdwin offset changed");
_Static_assert(offsetof(NormalState, noexmode) == 22, "NormalState.noexmode offset changed");
_Static_assert(offsetof(NormalState, toplevel) == 23, "NormalState.toplevel offset changed");
_Static_assert(offsetof(NormalState, oa) == 24, "NormalState.oa offset changed");
_Static_assert(offsetof(NormalState, ca) == 112, "NormalState.ca offset changed");
_Static_assert(offsetof(NormalState, mapped_len) == 200, "NormalState.mapped_len offset changed");
_Static_assert(offsetof(NormalState, old_mapped_len) == 204, "NormalState.old_mapped_len offset changed");
_Static_assert(offsetof(NormalState, idx) == 208, "NormalState.idx offset changed");
_Static_assert(offsetof(NormalState, c) == 212, "NormalState.c offset changed");
_Static_assert(offsetof(NormalState, old_col) == 216, "NormalState.old_col offset changed");
_Static_assert(offsetof(NormalState, old_pos) == 220, "NormalState.old_pos offset changed");

/// Get clear_cmdline global.
bool nvim_get_clear_cmdline(void) { return clear_cmdline; }

/// Get in_assert_fails global.
bool nvim_get_in_assert_fails(void) { return in_assert_fails; }

/// Set curbuf->b_last_used to time(NULL).
void nvim_curbuf_set_b_last_used(void) { curbuf->b_last_used = time(NULL); }

/// curwin->w_p_scb.
bool nvim_curwin_get_p_scb(void) { return curwin->w_p_scb; }

/// curwin->w_p_crb.
bool nvim_curwin_get_p_crb(void) { return curwin->w_p_crb; }

/// validate_cursor(curwin) wrapper.
void nvim_validate_cursor_curwin_wrapper(void) { validate_cursor(curwin); }

// normal_execute accessors for Rust FFI

_Static_assert(K_KENTER == -16715, "K_KENTER changed");
_Static_assert(K_ZERO == -22783, "K_ZERO changed");
_Static_assert(ESC == 27, "ESC changed");
_Static_assert(NL == 10, "NL changed");
_Static_assert(CAR == 13, "CAR changed");
_Static_assert(Ctrl_W == 23, "Ctrl_W changed");
_Static_assert(MOD_MASK_SHIFT == 0x02, "MOD_MASK_SHIFT changed");
_Static_assert(MODE_NORMAL == 0x01, "MODE_NORMAL changed");
_Static_assert(MODE_SELECT == 0x40, "MODE_SELECT changed");

/// Get vgetc_char global.
int nvim_get_vgetc_char(void) { return vgetc_char; }

/// Get curwin->w_p_rl.
bool nvim_get_curwin_w_p_rl(void) { return curwin->w_p_rl; }

static int normal_execute(VimState *state, int key) { return rs_normal_execute((NormalState *)state, key); }

// normal_check accessors for Rust FFI

/// Set quit_more global.
void nvim_set_quit_more(bool val) { quit_more = val; }

/// Get skip_redraw global.
bool nvim_get_skip_redraw(void) { return skip_redraw; }

/// Set skip_redraw global.
void nvim_set_skip_redraw(bool val) { skip_redraw = val; }

bool nvim_curtab_needs_diff_update(void) { return curtab->tp_diff_update || curtab->tp_diff_invalid; }

/// Clear curtab diff update flag.
void nvim_curtab_clear_diff_update(void) { curtab->tp_diff_update = false; }

void nvim_set_diff_need_scrollbind(bool val) { diff_need_scrollbind = val; }

void nvim_time_msg_first_screen_and_finish(void) { TIME_MSG("first screen update"); time_finish(); }

// normal_check and normal_redraw accessors for Rust FFI

void nvim_set_did_check_timestamps(bool val) { did_check_timestamps = val; }
bool nvim_get_need_check_timestamps(void) { return need_check_timestamps; }
/// Check if last_cursormoved_win != curwin or cursor position differs.
bool nvim_last_cursormoved_check(void) { return last_cursormoved_win != curwin || !equalpos(last_cursormoved, curwin->w_cursor); }

/// Update last_cursormoved_win and last_cursormoved to curwin/cursor.
void nvim_update_last_cursormoved(void) { last_cursormoved_win = curwin; last_cursormoved = curwin->w_cursor; }

/// Check if curbuf changedtick has changed since b_last_changedtick.
bool nvim_curbuf_changedtick_changed(void) { return curbuf->b_last_changedtick != buf_get_changedtick(curbuf); }

/// Update curbuf->b_last_changedtick to the current changedtick.
void nvim_curbuf_update_last_changedtick(void) { curbuf->b_last_changedtick = buf_get_changedtick(curbuf); }

/// Get curbuf->b_changed_invalid.
bool nvim_curbuf_b_changed_invalid_get(void) { return curbuf->b_changed_invalid; }

/// Clear curbuf->b_changed_invalid.
void nvim_curbuf_b_changed_invalid_clear(void) { curbuf->b_changed_invalid = false; }

static int normal_check(VimState *state) { return rs_normal_check((NormalState *)state); }

// showcmd accessors for Rust FFI

/// Constants for clear_showcmd (verified with _Static_assert).
_Static_assert(SHOWCMD_COLS == 10, "SHOWCMD_COLS changed");
_Static_assert(SHOWCMD_BUFLEN == SHOWCMD_COLS + 1 + 30, "SHOWCMD_BUFLEN changed");

char *nvim_normal_showcmd_buf_ptr(void) { return showcmd_buf; }

/// Set w_redr_status for curwin (showcmdloc=statusline, clear path).
void nvim_showcmd_set_w_redr_status(void) { curwin->w_redr_status = true; }

/// Send showcmd via UI messages protocol.
/// If is_clear, sends an empty array; otherwise sends [{0, buf, 0}].
void nvim_showcmd_ui_msg_showcmd(const char *buf, bool is_clear)
{
  MAXSIZE_TEMP_ARRAY(content, 1);
  MAXSIZE_TEMP_ARRAY(chunk, 3);
  if (!is_clear) {
    ADD_C(chunk, INTEGER_OBJ(0));
    ADD_C(chunk, CSTR_AS_OBJ(buf));
    ADD_C(chunk, INTEGER_OBJ(0));
    ADD_C(content, ARRAY_OBJ(chunk));
  }
  ui_call_msg_showcmd(content);
}

/// Render the showcmd area on the grid last line.
/// buf is the current showcmd text (NULL or empty means clear).
/// len is the number of display cells already written (from grid_line_puts).
/// Performs msg_grid_validate, grid_line_start, grid_line_puts (content),
/// grid_line_puts (padding), grid_line_flush.
void nvim_showcmd_grid_render(const char *buf, bool is_clear)
{
  msg_grid_validate();
  int showcmd_row = Rows - 1;
  grid_line_start(&msg_grid_adj, showcmd_row);

  int len = 0;
  if (!is_clear) {
    len = grid_line_puts(sc_col, buf, -1, HL_ATTR(HLF_MSG));
  }

  // clear the rest of an old message by outputting up to SHOWCMD_COLS spaces
  grid_line_puts(sc_col + len, (char *)"          " + len, -1, HL_ATTR(HLF_MSG));

  grid_line_flush();
}

/// getvcols with p_sbr/w_p_sbr save-restore for block-Visual showcmd.
/// Saves p_sbr and curwin->w_p_sbr, sets them to empty, calls
/// getvcols(curwin, &w_cursor, &VIsual, out_left, out_right), then restores.
void nvim_getvcols_visual_sbr_save(int *out_left, int *out_right)
{
  char *const saved_sbr = p_sbr;
  char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option;
  curwin->w_p_sbr = empty_string_option;
  colnr_T leftcol, rightcol;
  getvcols(curwin, &curwin->w_cursor, &VIsual, &leftcol, &rightcol);
  p_sbr = saved_sbr;
  curwin->w_p_sbr = saved_w_sbr;
  *out_left = (int)leftcol;
  *out_right = (int)rightcol;
}

void add_to_showcmd_c(int c) { add_to_showcmd(c); setcursor(); }

/// Set did_syncbind global.
void nvim_set_did_syncbind(bool val) { did_syncbind = val; }

/// Check curwin->w_buffer pointer equality with a saved buffer handle.
bool nvim_curwin_buf_eq(buf_T *buf) { return curwin->w_buffer == buf; }

/// Get curwin->w_p_diff.
bool nvim_curwin_get_w_p_diff(void) { return curwin->w_p_diff; }

/// Get curwin->w_scbind_pos.
int nvim_curwin_get_w_scbind_pos(void) { return curwin->w_scbind_pos; }

/// Set curwin->w_scbind_pos.
void nvim_curwin_set_w_scbind_pos(int val) { curwin->w_scbind_pos = (linenr_T)val; }

/// Compound: iterate all windows in current tab and sync scrollbind.
/// Handles curwin/curbuf swapping internally (unsafe to do in Rust).
/// old_curwin_arg: the original window before the loop.
/// vtopline_diff: vertical scroll diff.
/// tgt_leftcol: target left column for horizontal scroll.
/// want_ver: do vertical sync.
/// want_hor: do horizontal sync.
void nvim_scrollbind_sync_windows(win_T *old_curwin_arg, int vtopline_diff,
                                   int tgt_leftcol, bool want_ver, bool want_hor)
{
  win_T *old_curbuf_win = curwin;
  buf_T *old_curbuf_buf = curbuf;
  int old_VIsual_select = VIsual_select;
  int old_VIsual_active = VIsual_active;
  VIsual_select = VIsual_active = 0;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    curwin = wp;
    curbuf = curwin->w_buffer;
    if (curwin == old_curwin_arg || !curwin->w_p_scb) {
      continue;
    }

    if (want_ver) {
      if (old_curwin_arg->w_p_diff && curwin->w_p_diff) {
        rs_diff_set_topline(old_curwin_arg, curwin);
      } else {
        curwin->w_scbind_pos += vtopline_diff;
        int curr_vtopline = rs_get_vtopline(curwin);
        int max_vtopline = curr_vtopline + curwin->w_topfill
                           + plines_m_win_fill(curwin, curwin->w_topline + 1,
                                               curbuf->b_ml.ml_line_count);
        int new_vtopline = MAX(MIN((linenr_T)curwin->w_scbind_pos, max_vtopline), 1);
        int y = new_vtopline - curr_vtopline;
        if (y > 0) {
          scrollup(curwin, y, false);
        } else {
          scrolldown(curwin, -y, false);
        }
      }
      redraw_later(curwin, UPD_VALID);
      cursor_correct(curwin);
      curwin->w_redr_status = true;
    }

    if (want_hor) {
      set_leftcol((colnr_T)tgt_leftcol);
    }
  }

  VIsual_select = old_VIsual_select;
  VIsual_active = old_VIsual_active;
  curwin = old_curbuf_win;
  curbuf = old_curbuf_buf;
}

/// Initializes static oparg_T/cmdarg_T and returns cap pointer.
/// nvim is single-threaded so function-static storage is safe.
cmdarg_T *nvim_create_temp_cap_for_ident(int c1, int c2)
{
  static oparg_T oa;
  static cmdarg_T ca;
  clear_oparg(&oa);
  CLEAR_FIELD(ca);
  ca.oap = &oa;
  ca.cmdchar = c1;
  ca.nchar = c2;
  return &ca;
}

void normal_cmd(oparg_T *oap, bool toplevel)
{
  NormalState s;
  normal_state_init(&s);
  s.toplevel = toplevel;
  s.oa = *oap;
  rs_normal_prepare(&s);
  normal_execute(&s.state, safe_vgetc());
  *oap = s.oa;
}

// nv_ident build accessors

/// Get the resolved keywordprg string (curbuf->b_p_kp or p_kp fallback).
char *nvim_ident_get_kp(void) { return *curbuf->b_p_kp == NUL ? p_kp : curbuf->b_p_kp; }

/// Return true if curbuf is a help buffer.
bool nvim_ident_curbuf_is_help(void) { return curbuf->b_help; }

/// Return curbuf's filetype string.
char *nvim_ident_get_curbuf_ft(void) { return curbuf->b_p_ft; }

/// Call do_cmdline with appropriate function pointer for colon/cmdkey.
/// Returns false on failure (mirrors do_cmdline return).
bool nvim_do_cmdline_for_colon(cmdarg_T *cap, bool is_cmdkey) { return do_cmdline(NULL, is_cmdkey ? getcmdkeycmd : getexline, NULL, cap->oap->op_type != OP_NOP ? DOCMD_KEEPLINE : 0); }

/// Return did_emsg.
int nvim_did_emsg_check(void) { return did_emsg; }

/// Returns true if cursor moved and highlights need refresh.
bool nvim_search_hls_needs_redraw(int prev_lnum, int prev_col, int prev_coladd) { pos_T prev = { .lnum = prev_lnum, .col = (colnr_T)prev_col, .coladd = (colnr_T)prev_coladd }; return !equalpos(curwin->w_cursor, prev) && p_hls && !no_hlsearch && win_hl_attr(curwin, HLF_LC) != win_hl_attr(curwin, HLF_L); }

/// Wrapper for grab_file_name(count1, &lnum). Sets *lnum_out to lnum.
char *nvim_grab_file_name(int count1, int *lnum_out)
{
  linenr_T lnum = -1;
  char *result = grab_file_name(count1, &lnum);
  if (lnum_out != NULL) {
    *lnum_out = (int)lnum;
  }
  return result;
}

/// Call do_ecmd for gotofile. Returns OK/FAIL (1/0).
int nvim_do_ecmd_for_gotofile(char *ptr) { return do_ecmd(0, ptr, NULL, NULL, ECMD_LAST, buf_hide(curbuf) ? ECMD_HIDE : 0, curwin); }

/// Call ml_get_pos(&VIsual).
char *nvim_ml_get_pos_visual(void) { return ml_get_pos(&VIsual); }

/// Set mouse_dragging to val.
void nvim_set_mouse_dragging(int val) { mouse_dragging = val; }

/// Call rs_find_ident_at_pos(curwin, cursor.lnum, cursor.col, text, NULL, find_type).
size_t rs_find_ident_under_cursor(char **text, int find_type) { return rs_find_ident_at_pos(curwin, curwin->w_cursor.lnum, curwin->w_cursor.col, text, NULL, find_type); }

/// Get curwin->w_redr_type.
int nvim_get_curwin_w_redr_type(void) { return curwin->w_redr_type; }

/// Set curwin->w_old_cursor_lnum and w_old_visual_lnum to cursor lnum.
void nvim_curwin_set_old_visual_lnums(void) { curwin->w_old_cursor_lnum = curwin->w_cursor.lnum; curwin->w_old_visual_lnum = curwin->w_cursor.lnum; }

// nv_z and operator implementation accessors for Rust FFI

/// Return true if curbuf is a terminal buffer.
bool nvim_get_curbuf_terminal(void) { return curbuf->terminal != NULL; }

// DPO (do_pending_operator) accessors for Rust FFI

bool nvim_dpo_get_VIsual_active(void) { return VIsual_active; }
bool nvim_dpo_get_finish_op(void) { return finish_op; }
bool nvim_get_redo_VIsual_busy(void) { return redo_VIsual_busy; }
void nvim_set_redo_VIsual_busy(bool val) { redo_VIsual_busy = val; }
void nvim_set_bangredo(bool val) { bangredo = val; }
int nvim_get_repeat_luaref(void) { return (int)repeat_luaref; }
bool nvim_dpo_get_p_sol(void) { return p_sol; }
int nvim_curwin_get_p_lbr(void) { return curwin->w_p_lbr; }
void nvim_curwin_reset_lbr(void) { if (curwin->w_p_lbr) { curwin->w_p_lbr = false; curwin->w_valid &= ~(VALID_WROW|VALID_WCOL|VALID_VIRTCOL); } }
void nvim_curwin_restore_lbr(int saved) { if (!curwin->w_p_lbr && saved) { curwin->w_p_lbr = true; curwin->w_valid &= ~(VALID_WROW|VALID_WCOL|VALID_VIRTCOL); } }
void nvim_dpo_validate_virtcol(void) { validate_virtcol(curwin); }
void nvim_oap_set_start_from_cursor(oparg_T *oap) { if (oap) { oap->start = curwin->w_cursor; } }
void nvim_oap_set_end_from_cursor(oparg_T *oap) { if (oap) { oap->end = curwin->w_cursor; } }

void nvim_set_resel_VIsual_mode(int val) { resel_VIsual_mode = val; }
void nvim_set_resel_VIsual_vcol(int val) { resel_VIsual_vcol = val; }
void nvim_set_resel_VIsual_line_count(int val) { resel_VIsual_line_count = val; }

/// Set oap->start = VIsual.
void nvim_oap_set_start_from_VIsual(oparg_T *oap) { if (oap) { oap->start = VIsual; } }
/// If VIsual_mode == 'V', zero out oap->start col fields.
void nvim_oap_start_zero_col_if_linewise(oparg_T *oap) { if (oap && VIsual_mode == 'V') { oap->start.col = 0; oap->start.coladd = 0; } }
/// Set VIsual = oap->start.
void nvim_VIsual_set_from_oap_start(oparg_T *oap) { if (oap) { VIsual = oap->start; } }
/// Set curbuf->b_visual from VIsual, cursor, VIsual_mode, curswant.
void nvim_dpo_save_visual_state(void) { curbuf->b_visual.vi_start = VIsual; curbuf->b_visual.vi_end = curwin->w_cursor; curbuf->b_visual.vi_mode = VIsual_mode; curbuf->b_visual.vi_curswant = curwin->w_curswant; curbuf->b_visual_mode_eval = VIsual_mode; }
/// Append repeat_cmdline to redo, then free it (handles ':' vs. K_COMMAND).
void nvim_dpo_append_repeat_cmdline_to_redo(int is_colon)
{
  if (repeat_cmdline != NULL) {
    if (is_colon) {
      AppendToRedobuffLit(repeat_cmdline, -1);
    } else {
      AppendToRedobuffSpec(repeat_cmdline);
    }
    AppendToRedobuff(NL_STR);
    XFREE_CLEAR(repeat_cmdline);
  } else {
    ResetRedobuff();
  }
}
/// Get ml_get_pos(&oap->end)[0] == NUL.
bool nvim_oap_end_is_NUL(oparg_T *oap) { return oap ? (*ml_get_pos(&oap->end) == NUL) : false; }
/// Get *p_sel == 'o'.
bool nvim_p_sel_is_old(void) { return *p_sel == 'o'; }
/// Set curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf).
void nvim_sync_curbuf_last_changedtick_i(void) { curbuf->b_last_changedtick_i = (varnumber_T)buf_get_changedtick(curbuf); }
/// hasFolding on oap->start.lnum: update oap->start.lnum if folded, return true if folded.
bool nvim_hasFolding_oap_start_up(oparg_T *oap) { return oap ? hasFolding(curwin, oap->start.lnum, &oap->start.lnum, NULL) : false; }
/// hasFolding on cursor: update cursor.lnum end-of-fold, return true if folded.
bool nvim_hasFolding_cursor_end_of_fold(void) { return hasFolding(curwin, curwin->w_cursor.lnum, NULL, &curwin->w_cursor.lnum); }
/// hasFolding on cursor: update cursor.lnum start-of-fold, return true if folded.
bool nvim_hasFolding_cursor_start_of_fold(void) { return hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL); }
/// hasFolding on oap->start.lnum: update to end-of-fold, return true.
bool nvim_hasFolding_oap_start_down(oparg_T *oap) { return oap ? hasFolding(curwin, oap->start.lnum, NULL, &oap->start.lnum) : false; }
/// check_pos on curbuf and &oap->end.
void nvim_check_pos_oap_end(oparg_T *oap) { if (oap) check_pos(curwin->w_buffer, &oap->end); }
/// virtual_active(curwin) and set virtual_op.
void nvim_set_virtual_op_from_active(void) { virtual_op = virtual_active(curwin); }
/// getvvcol on oap->end: returns end_vcol (sets oap->end_vcol).
void nvim_getvvcol_oap_end(oparg_T *oap) { if (oap) getvvcol(curwin, &oap->end, NULL, NULL, &oap->end_vcol); }
/// getvvcol on oap->start: returns start_vcol.
void nvim_getvvcol_oap_start(oparg_T *oap) { if (oap) getvvcol(curwin, &oap->start, &oap->start_vcol, NULL, NULL); }
/// mark_mb_adjustpos on oap->end.
void nvim_mark_mb_adjustpos_oap_end(oparg_T *oap) { if (oap) mark_mb_adjustpos(curwin->w_buffer, &oap->end); }
/// getvvcol on oap->start: get start_vcol and end_vcol (for block vcol init).
void nvim_getvvcol_oap_start_both(oparg_T *oap) { if (oap) getvvcol(curwin, &oap->start, &oap->start_vcol, NULL, &oap->end_vcol); }
/// getvvcol on oap->end for end: get start and end vcol.
void nvim_getvvcol_oap_end_both(oparg_T *oap, int *start_out, int *end_out) { if (!oap) return; colnr_T s, e; getvvcol(curwin, &oap->end, &s, NULL, &e); if (start_out) *start_out = s; if (end_out) *end_out = e; }
/// coladvance to oap->end_vcol, then set oap->end = cursor.
void nvim_coladvance_set_oap_end(oparg_T *oap) { if (!oap) return; curwin->w_cursor.lnum = oap->end.lnum; coladvance(curwin, oap->end_vcol); oap->end = curwin->w_cursor; }
/// coladvance to oap->start_vcol, then set oap->start = cursor.
void nvim_coladvance_set_oap_start(oparg_T *oap) { if (!oap) return; curwin->w_cursor = oap->start; coladvance(curwin, oap->start_vcol); oap->start = curwin->w_cursor; }
/// Call gchar_pos(&oap->end).
int nvim_gchar_pos_oap_end(oparg_T *oap) { return oap ? gchar_pos(&oap->end) : 0; }
/// equalpos(oap->start, oap->end).
bool nvim_equalpos_oap(oparg_T *oap) { return oap ? equalpos(oap->start, oap->end) : false; }
/// utfc_ptr2len(ml_get_pos(&oap->end)).
int nvim_utfc_ptr2len_oap_end(oparg_T *oap) { return oap ? (int)utfc_ptr2len(ml_get_pos(&oap->end)) : 1; }
/// lt(oap->start, curwin->w_cursor).
bool nvim_lt_oap_start_cursor(oparg_T *oap) { return oap ? lt(oap->start, curwin->w_cursor) : false; }
/// Cursor = oap->start, invalidate VALID_VIRTCOL.
void nvim_cursor_set_oap_start(oparg_T *oap) { if (oap) { curwin->w_cursor = oap->start; curwin->w_valid &= ~VALID_VIRTCOL; } }
/// Get curbuf->b_p_lisp.
bool nvim_get_curbuf_b_p_lisp(void) { return curbuf->b_p_lisp; }
/// Get *curbuf->b_p_fex != NUL.
bool nvim_get_curbuf_b_p_fex_nonempty(void) { return *curbuf->b_p_fex != NUL; }
/// Get *p_fp != NUL.
bool nvim_get_p_fp_nonempty(void) { return *p_fp != NUL; }
/// Get *curbuf->b_p_fp != NUL.
bool nvim_get_curbuf_b_p_fp_nonempty(void) { return *curbuf->b_p_fp != NUL; }
/// Get *curbuf->b_p_inde != NUL.
bool nvim_get_curbuf_b_p_inde_nonempty(void) { return *curbuf->b_p_inde != NUL; }
/// Check curwin->w_cursor.lnum + line_count - 1 > ml_line_count.
bool nvim_dpo_join_would_overflow(int line_count) { return curwin->w_cursor.lnum + line_count - 1 > curbuf->b_ml.ml_line_count; }
/// coladvance(curwin, curwin->w_curswant = old_col).
void nvim_coladvance_set_curswant(int old_col) { coladvance(curwin, curwin->w_curswant = (colnr_T)old_col); }
/// Set virtual_op = kNone.
void nvim_set_virtual_op_none(void) { virtual_op = kNone; }
/// Set motion_force = NUL.
void nvim_set_motion_force_nul(void) { motion_force = NUL; }
/// Return curwin as opaque pointer (for DPO use).
win_T *nvim_dpo_get_curwin(void) { return curwin; }
/// Call redraw_curbuf_later(UPD_INVERTED).
void nvim_redraw_curbuf_later_inverted(void) { redraw_curbuf_later(UPD_INVERTED); }
/// Set curwin->w_set_curswant = val (bool).
void nvim_curwin_set_curswant_flag(bool val) { curwin->w_set_curswant = val; }

int nvim_has_mod_mask_ctrl(void) { return (mod_mask & MOD_MASK_CTRL) ? 1 : 0; }
bool nvim_has_ve_flag_onemore(void) { return (get_ve_flags(curwin) & kOptVeFlagOnemore) != 0; }
bool nvim_fdo_hor_and_key_typed(void) { return (fdo_flags & kOptFdoFlagHor) && KeyTyped; }

