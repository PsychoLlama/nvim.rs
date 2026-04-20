#include <stdbool.h>
#include <string.h>
#include <time.h>

#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
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
#include "nvim/state.h"
#include "nvim/syntax.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
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

extern int rs_normal_check(void *s);
extern int rs_normal_execute(void *s, int key);
extern void rs_normal_prepare(void *s);
extern void rs_diff_set_topline(win_T *fromwin, win_T *towin);
extern int rs_get_vtopline(win_T *wp);
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
unsigned int nvim_get_fdo_flags(void) { return fdo_flags; }
void nvim_set_ins_at_eol(bool val) { ins_at_eol = val; }
void nvim_set_curswant(colnr_T val) { curwin->w_curswant = val; }
bool nvim_virtual_active(void) { return virtual_active(curwin); }
int nvim_gchar_cursor(void) { return utf_ptr2char(get_cursor_pos_ptr()); }
void nvim_coladvance(colnr_T col) { coladvance(curwin, col); }
int nvim_get_cursor_col(void) { return curwin->w_cursor.col; }
void nvim_set_cursor_col(int col) { curwin->w_cursor.col = col; }
int nvim_inc_cursor(void) { return inc(&curwin->w_cursor); }
int nvim_dec_cursor(void) { return dec(&curwin->w_cursor); }
unsigned int nvim_get_ve_flags(void) { return get_ve_flags(curwin); }
int nvim_get_VIsual_mode(void) { return VIsual_mode; }
void nvim_getvcol_cursor(int *scol, int *ecol) { getvcol(curwin, &curwin->w_cursor, scol, NULL, ecol); }
void nvim_set_cursor_coladd(int val) { curwin->w_cursor.coladd = val; }
fmark_T *nvim_mark_get(int name) { return mark_get(curbuf, curwin, NULL, kMarkAll, name); }
fmark_T *nvim_get_changelist(int count1) { return get_changelist(curbuf, curwin, count1); }
fmark_T *nvim_get_jumplist(int count1) { return get_jumplist(curwin, count1); }
bool nvim_put_get_save_fen(void) { return curwin->w_p_fen; }
void nvim_put_free_register(void *savereg) { if (savereg != NULL) { free_register((yankreg_T *)savereg); xfree(savereg); } }
int nvim_get_b_prompt_start_lnum_put(void) { return curbuf->b_prompt_start.mark.lnum; }
void nvim_set_cursor_col_to_prompt_text_len(void) { curwin->w_cursor.col = (int)strlen(prompt_text()); }
void nvim_set_w_p_fen(bool val) { curwin->w_p_fen = val; }
void nvim_inc_msg_silent(void) { msg_silent++; }
bool nvim_curbuf_ml_empty(void) { return (curbuf->b_ml.ml_flags & ML_EMPTY) != 0; }
int nvim_get_cursor_col_vs_b_op_start_col(void) { return curwin->w_cursor.col - curbuf->b_op_start.col; }
int nvim_get_cursor_lnum_vs_b_op_start_lnum(void) { return (int)(curwin->w_cursor.lnum - curbuf->b_op_start.lnum); }
void nvim_inc_b_visual_vi_end(void) { inc(&curbuf->b_visual.vi_end); }
void nvim_ml_delete_last_line(void) { ml_delete_flags(curbuf->b_ml.ml_line_count, ML_DEL_MESSAGE); deleted_lines(curbuf->b_ml.ml_line_count + 1, 1); }
void nvim_set_b_op_start_cursor(void) { curbuf->b_op_start = curwin->w_cursor; }
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
void nvim_clear_b_syn_slow_all_windows(void) {
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    wp->w_s->b_syn_slow = false;
  }
}
void nvim_syn_stack_free_all_curwin(void) { syn_stack_free_all(curwin->w_s); }
bool nvim_get_mode_displayed(void) { return mode_displayed; }
void nvim_set_mode_displayed(bool val) { mode_displayed = val; }
void nvim_set_clear_cmdline(bool val) { clear_cmdline = val; }
bool nvim_set_cursor_from_last_insert(void) { if (curbuf->b_last_insert.mark.lnum != 0) { curwin->w_cursor = curbuf->b_last_insert.mark; return true; } return false; }
void nvim_check_cursor_lnum_call(void) { check_cursor_lnum(curwin); }
int nvim_get_cursor_line_len(void) { return (int)get_cursor_line_len(); }
int nvim_get_cursor_coladd(void) { return curwin->w_cursor.coladd; }
void nvim_set_cmdwin_result(int val) { cmdwin_result = val; }
void nvim_set_VIsual_pos(int lnum, int col, int coladd) { VIsual.lnum = lnum; VIsual.col = col; VIsual.coladd = coladd; }
void nvim_set_cursor_pos(int lnum, int col, int coladd) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; curwin->w_cursor.coladd = coladd; }
int nvim_get_b_visual_vi_curswant(void) { return curbuf->b_visual.vi_curswant; }
int nvim_get_VIsual_select_reg(void) { return VIsual_select_reg; }
int nvim_get_virtual_op(void) { return (int)virtual_op; }
bool nvim_p_sel_is_exclusive(void) { return *p_sel == 'e'; }
int nvim_mark_mb_adjustpos_pos(int lnum, int col, int *col_out) { pos_T pp = { lnum, (colnr_T)col, 0 }; mark_mb_adjustpos(curbuf, &pp); *col_out = pp.col; return pp.col; }
int nvim_getvcol_pos_coladd(int lnum, int col, int coladd) { pos_T pp = { lnum, (colnr_T)col, (colnr_T)coladd }; colnr_T cs, ce; getvcol(curwin, &pp, &cs, NULL, &ce); return (int)(ce - cs); }
char *nvim_getcmdline_for_search(cmdarg_T *cap) { cap->searchbuf = getcmdline(cap->cmdchar, cap->count1, 0, true); return cap->searchbuf; }
int nvim_searchit_decl(const char *pat, size_t patlen, int searchflags) { return searchit(curwin, curbuf, &curwin->w_cursor, NULL, FORWARD, (char *)pat, patlen, 1, searchflags, RE_LAST, NULL); }
bool nvim_bt_prompt_curbuf(void) { return bt_prompt(curbuf); }
fmark_T *nvim_pos_to_mark_cursor(void) { return pos_to_mark(curbuf, NULL, curwin->w_cursor); }
int nvim_get_curwin_w_skipcol(void) { return (int)curwin->w_skipcol; }
int nvim_get_curwin_w_topline(void) { return (int)curwin->w_topline; }
bool nvim_get_curwin_w_cline_folded(void) { return curwin->w_cline_folded; }
void nvim_clear_curwin_w_valid_wcol(void) { curwin->w_valid &= ~VALID_WCOL; }
int nvim_getvvcol_cursor_end(void) { colnr_T vcol; getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol); return (int)vcol; }
void nvim_hasFolding_cursor_set_lnum_up(void) { hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL); }
void nvim_hasFolding_cursor_set_lnum_down(void) { hasFolding(curwin, curwin->w_cursor.lnum, NULL, &curwin->w_cursor.lnum); }
void nvim_set_curbuf_b_last_changedtick_i(void) { curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf); }
void nvim_clear_curwin_w_valid_crow(void) { curwin->w_valid &= ~VALID_CROW; }
int nvim_mark_mb_adjustpos_cursor_new(void) { mark_mb_adjustpos(curbuf, &curwin->w_cursor); return curwin->w_cursor.col; }
int nvim_getvcol_cursor_coladd_after_adj(void) { colnr_T cs, ce; getvcol(curwin, &curwin->w_cursor, &cs, NULL, &ce); return (int)(ce - cs); }
int nvim_mark_mb_adjustpos_visual_new(void) { mark_mb_adjustpos(curbuf, &VIsual); return VIsual.col; }
int nvim_getvcol_visual_coladd_after_adj(void) { colnr_T cs, ce; getvcol(curwin, &VIsual, &cs, NULL, &ce); return (int)(ce - cs); }
bool nvim_curbuf_modifiable(void) { return MODIFIABLE(curbuf); }
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
char *nvim_ml_get_pos_cursor(void) { return ml_get_pos(&curwin->w_cursor); }
void nvim_sync_fen_in_diff_windows(void) {
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp != curwin && rs_foldmethodIsDiff(wp) && wp->w_p_scb) { wp->w_p_fen = curwin->w_p_fen; changed_window_setting(wp); }
  }
}
void nvim_set_oap_cursor_start(oparg_T *oap) { oap->cursor_start = curwin->w_cursor; }
int nvim_get_curwin_w_virtcol(void) { return curwin->w_virtcol; }
int nvim_get_curwin_ml_line_count(void) { return curwin->w_buffer->b_ml.ml_line_count; }

static char *nvim_mps_save = NULL;
void nvim_save_and_set_mps(void) { nvim_mps_save = curbuf->b_p_mps; curbuf->b_p_mps = "(:),{:},[:],<:>"; }
void nvim_restore_mps(void) { curbuf->b_p_mps = nvim_mps_save; }

void nvim_u_clearline_curbuf(void) { u_clearline(curbuf); }
void nvim_changed_lines_call(int lnum, int col, int lnum_end, bool do_concealed) { changed_lines(curbuf, (linenr_T)lnum, (colnr_T)col, (linenr_T)lnum_end, 0, do_concealed); }
void nvim_set_b_op_start(int lnum, int col, int coladd) { curbuf->b_op_start.lnum = (linenr_T)lnum; curbuf->b_op_start.col = (colnr_T)col; curbuf->b_op_start.coladd = (colnr_T)coladd; }
void nvim_set_b_op_end_cursor(void) { curbuf->b_op_end = curwin->w_cursor; }

void normal_enter(bool cmdwin, bool noexmode) {
  NormalState state; normal_state_init(&state);
  oparg_T *prev_oap = nvim_current_oap; nvim_current_oap = &state.oa;
  state.cmdwin = cmdwin; state.noexmode = noexmode;
  state.toplevel = (!cmdwin || cmdwin_result == 0) && !noexmode;
  state_enter(&state.state); nvim_current_oap = prev_oap;
}

int nvim_langmap_adjust(int c, bool condition) { LANGMAP_ADJUST(c, condition); return c; }
void nvim_inc_no_mapping(void) { no_mapping++; }
void nvim_dec_no_mapping(void) { no_mapping--; }
void nvim_inc_allow_keys(void) { allow_keys++; }
void nvim_dec_allow_keys(void) { allow_keys--; }
void nvim_set_did_cursorhold(bool val) { did_cursorhold = val; }
int nvim_get_curbuf_b_p_iminsert(void) { return curbuf->b_p_iminsert; }
bool nvim_vim_strchr_p_cpo(int c) { return vim_strchr(p_cpo, c) != NULL; }
int nvim_get_MB_BYTE2LEN(int c) { return MB_BYTE2LEN(c); }

// Layout guards for repr(C) struct mirrors in src/nvim-rs/normal/src/types.rs
_Static_assert(kOptFdoFlagHor == 0x04, "kOptFdoFlagHor changed - update K_OPT_FDO_FLAG_HOR in normal/src/lib.rs");
_Static_assert(kOptFdoFlagBlock == 0x02, "kOptFdoFlagBlock changed - update K_OPT_FDO_FLAG_BLOCK in normal/src/lib.rs");
_Static_assert(kOptFdoFlagJump == 0x400, "kOptFdoFlagJump changed - update K_OPT_FDO_FLAG_JUMP in normal/src/lib.rs");
_Static_assert(kOptFdoFlagMark == 0x08, "kOptFdoFlagMark changed - update K_OPT_FDO_FLAG_MARK in normal/src/lib.rs");
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

bool nvim_get_clear_cmdline(void) { return clear_cmdline; }
bool nvim_get_in_assert_fails(void) { return in_assert_fails; }
void nvim_curbuf_set_b_last_used(void) { curbuf->b_last_used = time(NULL); }
bool nvim_curwin_get_p_scb(void) { return curwin->w_p_scb; }
bool nvim_curwin_get_p_crb(void) { return curwin->w_p_crb; }
void nvim_validate_cursor_curwin_wrapper(void) { validate_cursor(curwin); }
int nvim_get_vgetc_char(void) { return vgetc_char; }
bool nvim_get_curwin_w_p_rl(void) { return curwin->w_p_rl; }

static int normal_execute(VimState *state, int key) { return rs_normal_execute((NormalState *)state, key); }

void nvim_set_quit_more(bool val) { quit_more = val; }
bool nvim_get_skip_redraw(void) { return skip_redraw; }
void nvim_set_skip_redraw(bool val) { skip_redraw = val; }
bool nvim_curtab_needs_diff_update(void) { return curtab->tp_diff_update || curtab->tp_diff_invalid; }
void nvim_curtab_clear_diff_update(void) { curtab->tp_diff_update = false; }
void nvim_set_diff_need_scrollbind(bool val) { diff_need_scrollbind = val; }
void nvim_time_msg_first_screen_and_finish(void) { TIME_MSG("first screen update"); time_finish(); }
void nvim_set_did_check_timestamps(bool val) { did_check_timestamps = val; }
bool nvim_get_need_check_timestamps(void) { return need_check_timestamps; }
bool nvim_last_cursormoved_check(void) { return last_cursormoved_win != curwin || !equalpos(last_cursormoved, curwin->w_cursor); }
void nvim_update_last_cursormoved(void) { last_cursormoved_win = curwin; last_cursormoved = curwin->w_cursor; }
bool nvim_curbuf_changedtick_changed(void) { return curbuf->b_last_changedtick != buf_get_changedtick(curbuf); }
void nvim_curbuf_update_last_changedtick(void) { curbuf->b_last_changedtick = buf_get_changedtick(curbuf); }
static int normal_check(VimState *state) { return rs_normal_check((NormalState *)state); }

char *nvim_normal_showcmd_buf_ptr(void) { return showcmd_buf; }
void nvim_showcmd_set_w_redr_status(void) { curwin->w_redr_status = true; }

// Phase 4 (check_timestamps) accessors
int nvim_get_no_check_timestamps(void) { return no_check_timestamps; }
bool nvim_get_did_check_timestamps(void) { return did_check_timestamps; }
int nvim_stuff_empty(void) { return stuff_empty() ? 1 : 0; }
int nvim_get_allbuf_lock(void) { return allbuf_lock; }
int nvim_get_curbuf_b_ro_locked(void) { return curbuf->b_ro_locked; }
int nvim_get_no_wait_return(void) { return no_wait_return; }
void nvim_set_no_wait_return(int val) { no_wait_return = val; }
bool nvim_get_need_wait_return(void) { return need_wait_return; }
int nvim_bufref_valid(void *br) { return bufref_valid((bufref_T *)br) ? 1 : 0; }
int nvim_bufref_size(void) { return (int)sizeof(bufref_T); }

void nvim_showcmd_ui_msg_showcmd(const char *buf, bool is_clear) {
  MAXSIZE_TEMP_ARRAY(content, 1); MAXSIZE_TEMP_ARRAY(chunk, 3);
  if (!is_clear) {
    ADD_C(chunk, INTEGER_OBJ(0)); ADD_C(chunk, CSTR_AS_OBJ(buf)); ADD_C(chunk, INTEGER_OBJ(0));
    ADD_C(content, ARRAY_OBJ(chunk));
  }
  ui_call_msg_showcmd(content);
}

void nvim_showcmd_grid_render(const char *buf, bool is_clear) {
  msg_grid_validate(); int showcmd_row = Rows - 1; grid_line_start(&msg_grid_adj, showcmd_row);
  int len = 0;
  if (!is_clear) { len = grid_line_puts(sc_col, buf, -1, HL_ATTR(HLF_MSG)); }
  // clear the rest of an old message by outputting up to SHOWCMD_COLS spaces
  grid_line_puts(sc_col + len, (char *)"          " + len, -1, HL_ATTR(HLF_MSG)); grid_line_flush();
}

void nvim_getvcols_visual_sbr_save(int *out_left, int *out_right) {
  char *const saved_sbr = p_sbr; char *const saved_w_sbr = curwin->w_p_sbr;
  p_sbr = empty_string_option; curwin->w_p_sbr = empty_string_option;
  colnr_T leftcol, rightcol;
  getvcols(curwin, &curwin->w_cursor, &VIsual, &leftcol, &rightcol);
  p_sbr = saved_sbr; curwin->w_p_sbr = saved_w_sbr;
  *out_left = (int)leftcol; *out_right = (int)rightcol;
}

void add_to_showcmd_c(int c) { add_to_showcmd(c); setcursor(); }
void nvim_set_did_syncbind(bool val) { did_syncbind = val; }
bool nvim_curwin_buf_eq(buf_T *buf) { return curwin->w_buffer == buf; }
bool nvim_curwin_get_w_p_diff(void) { return curwin->w_p_diff; }
int nvim_curwin_get_w_scbind_pos(void) { return curwin->w_scbind_pos; }
void nvim_curwin_set_w_scbind_pos(int val) { curwin->w_scbind_pos = (linenr_T)val; }

void nvim_scrollbind_sync_windows(win_T *old_curwin_arg, int vtopline_diff,
                                   int tgt_leftcol, bool want_ver, bool want_hor)
{
  win_T *old_curbuf_win = curwin; buf_T *old_curbuf_buf = curbuf;
  int old_VIsual_select = VIsual_select; int old_VIsual_active = VIsual_active;
  VIsual_select = VIsual_active = 0;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    curwin = wp; curbuf = curwin->w_buffer;
    if (curwin == old_curwin_arg || !curwin->w_p_scb) { continue; }
    if (want_ver) {
      if (old_curwin_arg->w_p_diff && curwin->w_p_diff) {
        rs_diff_set_topline(old_curwin_arg, curwin);
      } else {
        curwin->w_scbind_pos += vtopline_diff;
        int curr_vtopline = rs_get_vtopline(curwin);
        int max_vtopline = curr_vtopline + curwin->w_topfill
                           + plines_m_win_fill(curwin, curwin->w_topline + 1, curbuf->b_ml.ml_line_count);
        int new_vtopline = MAX(MIN((linenr_T)curwin->w_scbind_pos, max_vtopline), 1);
        int y = new_vtopline - curr_vtopline;
        if (y > 0) { scrollup(curwin, y, false); } else { scrolldown(curwin, -y, false); }
      }
      redraw_later(curwin, UPD_VALID); cursor_correct(curwin); curwin->w_redr_status = true;
    }
    if (want_hor) { set_leftcol((colnr_T)tgt_leftcol); }
  }
  VIsual_select = old_VIsual_select; VIsual_active = old_VIsual_active;
  curwin = old_curbuf_win; curbuf = old_curbuf_buf;
}

void normal_cmd(oparg_T *oap, bool toplevel) {
  NormalState s; normal_state_init(&s);
  s.toplevel = toplevel; s.oa = *oap;
  rs_normal_prepare(&s); normal_execute(&s.state, safe_vgetc()); *oap = s.oa;
}

char *nvim_ident_get_kp(void) { return *curbuf->b_p_kp == NUL ? p_kp : curbuf->b_p_kp; }
bool nvim_ident_curbuf_is_help(void) { return curbuf->b_help; }
char *nvim_ident_get_curbuf_ft(void) { return curbuf->b_p_ft; }
bool nvim_do_cmdline_for_colon(cmdarg_T *cap, bool is_cmdkey) { return do_cmdline(NULL, is_cmdkey ? getcmdkeycmd : getexline, NULL, cap->oap->op_type != OP_NOP ? DOCMD_KEEPLINE : 0); }
int nvim_did_emsg_check(void) { return did_emsg; }
bool nvim_search_hls_needs_redraw(int prev_lnum, int prev_col, int prev_coladd) { pos_T prev = { .lnum = prev_lnum, .col = (colnr_T)prev_col, .coladd = (colnr_T)prev_coladd }; return !equalpos(curwin->w_cursor, prev) && p_hls && !no_hlsearch && win_hl_attr(curwin, HLF_LC) != win_hl_attr(curwin, HLF_L); }

int nvim_do_ecmd_for_gotofile(char *ptr) { return do_ecmd(0, ptr, NULL, NULL, ECMD_LAST, buf_hide(curbuf) ? ECMD_HIDE : 0, curwin); }
char *nvim_ml_get_pos_visual(void) { return ml_get_pos(&VIsual); }
void nvim_set_mouse_dragging(int val) { mouse_dragging = val; }
size_t rs_find_ident_under_cursor(char **text, int find_type) { return rs_find_ident_at_pos(curwin, curwin->w_cursor.lnum, curwin->w_cursor.col, text, NULL, find_type); }
int nvim_get_curwin_w_redr_type(void) { return curwin->w_redr_type; }
void nvim_curwin_set_old_visual_lnums(void) { curwin->w_old_cursor_lnum = curwin->w_cursor.lnum; curwin->w_old_visual_lnum = curwin->w_cursor.lnum; }
bool nvim_get_curbuf_terminal(void) { return curbuf->terminal != NULL; }
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
void nvim_oap_set_start_from_VIsual(oparg_T *oap) { if (oap) { oap->start = VIsual; } }
void nvim_oap_start_zero_col_if_linewise(oparg_T *oap) { if (oap && VIsual_mode == 'V') { oap->start.col = 0; oap->start.coladd = 0; } }
void nvim_VIsual_set_from_oap_start(oparg_T *oap) { if (oap) { VIsual = oap->start; } }
void nvim_dpo_save_visual_state(void) { curbuf->b_visual.vi_start = VIsual; curbuf->b_visual.vi_end = curwin->w_cursor; curbuf->b_visual.vi_mode = VIsual_mode; curbuf->b_visual.vi_curswant = curwin->w_curswant; curbuf->b_visual_mode_eval = VIsual_mode; }

bool nvim_oap_end_is_NUL(oparg_T *oap) { return oap ? (*ml_get_pos(&oap->end) == NUL) : false; }
bool nvim_p_sel_is_old(void) { return *p_sel == 'o'; }
void nvim_sync_curbuf_last_changedtick_i(void) { curbuf->b_last_changedtick_i = (varnumber_T)buf_get_changedtick(curbuf); }
void nvim_getchar_semsg_lua_err(char *msg) { semsg_multiline("emsg", "E5108: %s", msg); }
bool nvim_hasFolding_oap_start_up(oparg_T *oap) { return oap ? hasFolding(curwin, oap->start.lnum, &oap->start.lnum, NULL) : false; }
bool nvim_hasFolding_cursor_end_of_fold(void) { return hasFolding(curwin, curwin->w_cursor.lnum, NULL, &curwin->w_cursor.lnum); }
bool nvim_hasFolding_cursor_start_of_fold(void) { return hasFolding(curwin, curwin->w_cursor.lnum, &curwin->w_cursor.lnum, NULL); }
bool nvim_hasFolding_oap_start_down(oparg_T *oap) { return oap ? hasFolding(curwin, oap->start.lnum, NULL, &oap->start.lnum) : false; }
void nvim_check_pos_oap_end(oparg_T *oap) { if (oap) check_pos(curwin->w_buffer, &oap->end); }
void nvim_set_virtual_op_from_active(void) { virtual_op = virtual_active(curwin); }
void nvim_getvvcol_oap_end(oparg_T *oap) { if (oap) getvvcol(curwin, &oap->end, NULL, NULL, &oap->end_vcol); }
void nvim_getvvcol_oap_start(oparg_T *oap) { if (oap) getvvcol(curwin, &oap->start, &oap->start_vcol, NULL, NULL); }
void nvim_mark_mb_adjustpos_oap_end(oparg_T *oap) { if (oap) mark_mb_adjustpos(curwin->w_buffer, &oap->end); }
void nvim_getvvcol_oap_start_both(oparg_T *oap) { if (oap) getvvcol(curwin, &oap->start, &oap->start_vcol, NULL, &oap->end_vcol); }
void nvim_getvvcol_oap_end_both(oparg_T *oap, int *start_out, int *end_out) { if (!oap) return; colnr_T s, e; getvvcol(curwin, &oap->end, &s, NULL, &e); if (start_out) *start_out = s; if (end_out) *end_out = e; }
void nvim_coladvance_set_oap_end(oparg_T *oap) { if (!oap) return; curwin->w_cursor.lnum = oap->end.lnum; coladvance(curwin, oap->end_vcol); oap->end = curwin->w_cursor; }
void nvim_coladvance_set_oap_start(oparg_T *oap) { if (!oap) return; curwin->w_cursor = oap->start; coladvance(curwin, oap->start_vcol); oap->start = curwin->w_cursor; }
int nvim_gchar_pos_oap_end(oparg_T *oap) { return oap ? gchar_pos(&oap->end) : 0; }
bool nvim_equalpos_oap(oparg_T *oap) { return oap ? equalpos(oap->start, oap->end) : false; }
int nvim_utfc_ptr2len_oap_end(oparg_T *oap) { return oap ? (int)utfc_ptr2len(ml_get_pos(&oap->end)) : 1; }
bool nvim_lt_oap_start_cursor(oparg_T *oap) { return oap ? lt(oap->start, curwin->w_cursor) : false; }
void nvim_cursor_set_oap_start(oparg_T *oap) { if (oap) { curwin->w_cursor = oap->start; curwin->w_valid &= ~VALID_VIRTCOL; } }
bool nvim_get_curbuf_b_p_lisp(void) { return curbuf->b_p_lisp; }
bool nvim_get_curbuf_b_p_fex_nonempty(void) { return *curbuf->b_p_fex != NUL; }
bool nvim_get_p_fp_nonempty(void) { return *p_fp != NUL; }
bool nvim_get_curbuf_b_p_fp_nonempty(void) { return *curbuf->b_p_fp != NUL; }
bool nvim_get_curbuf_b_p_inde_nonempty(void) { return *curbuf->b_p_inde != NUL; }
bool nvim_dpo_join_would_overflow(int line_count) { return curwin->w_cursor.lnum + line_count - 1 > curbuf->b_ml.ml_line_count; }
void nvim_coladvance_set_curswant(int old_col) { coladvance(curwin, curwin->w_curswant = (colnr_T)old_col); }
void nvim_set_virtual_op_none(void) { virtual_op = kNone; }
win_T *nvim_dpo_get_curwin(void) { return curwin; }
void nvim_curwin_set_curswant_flag(bool val) { curwin->w_set_curswant = val; }
int nvim_has_mod_mask_ctrl(void) { return (mod_mask & MOD_MASK_CTRL) ? 1 : 0; }
bool nvim_has_ve_flag_onemore(void) { return (get_ve_flags(curwin) & kOptVeFlagOnemore) != 0; }
bool nvim_fdo_hor_and_key_typed(void) { return (fdo_flags & kOptFdoFlagHor) && KeyTyped; }

// Accessors for vgetorpeek (Phase 2 migration)
int nvim_curwin_get_wcol(void) { return curwin->w_wcol; }
void nvim_curwin_set_wcol(int val) { curwin->w_wcol = val; }
int nvim_curwin_get_wrow(void) { return curwin->w_wrow; }
void nvim_curwin_set_wrow(int val) { curwin->w_wrow = val; }
int nvim_curbuf_get_mapped_ctrl_c(void) { return curbuf->b_mapped_ctrl_c; }

/// Check if get_cmdline_info()->cmdbuff is non-NULL (for showcmd display).
bool nvim_cmdline_info_has_cmdbuff(void) {
  return get_cmdline_info()->cmdbuff != NULL;
}

/// Helper for vgetorpeek ESC-in-insert-mode cursor optimization.
/// Moves the cursor left visually (for display purposes only), updating
/// w_wcol/w_wrow based on cursor position.
/// Returns new_wcol and new_wrow via out-parameters.
void nvim_vgetorpeek_esc_cursor_left(int *new_wcol, int *new_wrow)
{
  validate_cursor(curwin);
  int old_wcol = curwin->w_wcol;
  int old_wrow = curwin->w_wrow;

  if (curwin->w_cursor.col != 0) {
    colnr_T col = 0;
    char *ptr;
    if (curwin->w_wcol > 0) {
      if (did_ai && *skipwhite(get_cursor_line_ptr() + curwin->w_cursor.col) == NUL) {
        curwin->w_wcol = 0;
        ptr = get_cursor_line_ptr();
        char *endptr = ptr + curwin->w_cursor.col;

        CharsizeArg csarg;
        CSType cstype = init_charsize_arg(&csarg, curwin, curwin->w_cursor.lnum, ptr);
        StrCharInfo ci = utf_ptr2StrCharInfo(ptr);
        int vcol = 0;
        while (ci.ptr < endptr) {
          if (!ascii_iswhite(ci.chr.value)) {
            curwin->w_wcol = vcol;
          }
          vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
          ci = utfc_next(ci);
        }

        curwin->w_wrow = curwin->w_cline_row + curwin->w_wcol / curwin->w_view_width;
        curwin->w_wcol %= curwin->w_view_width;
        curwin->w_wcol += win_col_off(curwin);
        col = 0;
      } else {
        curwin->w_wcol--;
        col = curwin->w_cursor.col - 1;
      }
    } else if (curwin->w_p_wrap && curwin->w_wrow) {
      curwin->w_wrow--;
      curwin->w_wcol = curwin->w_view_width - 1;
      col = curwin->w_cursor.col - 1;
    }
    if (col > 0 && curwin->w_wcol > 0) {
      ptr = get_cursor_line_ptr();
      col -= utf_head_off(ptr, ptr + col);
      if (utf_ptr2cells(ptr + col) > 1) {
        curwin->w_wcol--;
      }
    }
  }
  setcursor();
  ui_flush();
  *new_wcol = curwin->w_wcol;
  *new_wrow = curwin->w_wrow;
  curwin->w_wcol = old_wcol;
  curwin->w_wrow = old_wrow;
}

// =============================================================================
// op_colon / op_function accessors (Phase 1 migration)
// =============================================================================
#include "nvim/eval/typval.h"

/// set virtual_op to a specific TriState value (as c_int).
void nvim_set_virtual_op(int val) { virtual_op = (TriState)val; }

/// Call hasFolding(curwin, lnum, NULL, end_out).
/// Returns true if lnum is in a closed fold.
bool nvim_hasFolding_lnum_end_out(int lnum, int *end_out)
{
  linenr_T end = (linenr_T)lnum;
  bool r = hasFolding(curwin, (linenr_T)lnum, NULL, &end);
  if (end_out != NULL) { *end_out = (int)end; }
  return r;
}

/// Set curbuf->b_op_start and b_op_end from oap->start and oap->end.
/// If not lockmarks and oap->motion_type != kMTLineWise and !inclusive,
/// also call decl(&curbuf->b_op_end).
void nvim_opfunc_set_op_marks(oparg_T *oap)
{
  curbuf->b_op_start = oap->start;
  curbuf->b_op_end = oap->end;
  if (oap->motion_type != kMTLineWise && !oap->inclusive) {
    decl(&curbuf->b_op_end);
  }
}

/// Emit "E774: 'operatorfunc' is empty" error.
void nvim_emsg_e774_operatorfunc(void) { emsg(_("E774: 'operatorfunc' is empty")); }

/// Return curbuf->b_p_fp string pointer (format program).
const char *nvim_get_curbuf_b_p_fp(void) { return curbuf->b_p_fp; }

/// Return p_fp string pointer (global format program).
const char *nvim_get_p_fp(void) { return p_fp; }

// =============================================================================
// State crate accessors (Phase 1)
// =============================================================================

/// Return whether currently using a script (for SafeState check).
int nvim_using_script(void) { return using_script() ? 1 : 0; }

/// Return whether debug_mode is set.
int nvim_is_debug_mode(void) { return debug_mode ? 1 : 0; }


/// Apply SafeState autocommand.
void nvim_apply_autocmds_safestate(void)
{
  apply_autocmds(EVENT_SAFESTATE, NULL, NULL, false, curbuf);
}

// =============================================================================
// State crate accessors (Phase 2)
// =============================================================================

/// Return motion_force global.
int nvim_get_motion_force(void) { return motion_force; }

/// Return restart_VIsual_select global.
int nvim_get_restart_VIsual_select(void) { return restart_VIsual_select; }

/// Return whether cmdline overstrike mode is active.
int nvim_cmdline_overstrike(void) { return cmdline_overstrike() ? 1 : 0; }

/// Return whether EVENT_MODECHANGED has any autocommands.
int nvim_has_event_modechanged(void) { return has_event(EVENT_MODECHANGED) ? 1 : 0; }

/// Apply ModeChanged autocommand with pattern "old:new".
void nvim_apply_autocmds_modechanged(const char *pattern_buf)
{
  apply_autocmds(EVENT_MODECHANGED, (char *)pattern_buf, NULL, false, curbuf);
}

/// Get last_mode global (returns pointer to static array of MODE_MAX_LENGTH bytes).
const char *nvim_get_last_mode(void) { return last_mode; }

/// Set last_mode from src string (copies up to MODE_MAX_LENGTH - 1 bytes).
void nvim_set_last_mode(const char *src) { STRCPY(last_mode, src); }

/// Return got_int (for ModeChanged check).
int nvim_state_got_int(void) { return got_int ? 1 : 0; }

/// Add new_mode/old_mode to v_event dict and set keys readonly.
void nvim_state_fill_v_event_modechanged(void *v_event, const char *new_mode, const char *old_mode)
{
  tv_dict_add_str(v_event, S_LEN("new_mode"), new_mode);
  tv_dict_add_str(v_event, S_LEN("old_mode"), old_mode);
  tv_dict_set_keys_readonly(v_event);
}
