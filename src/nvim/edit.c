// edit.c: functions for Insert mode

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <uv.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/marktree_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/terminal.h"
#include "nvim/textformat.h"
#include "nvim/textobject.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Rust implementations (called directly instead of C wrappers)
extern int rs_ctrl_x_mode_none(void);
extern int rs_ctrl_x_mode_scroll(void);
extern int rs_ctrl_x_mode_cmdline(void);
extern int rs_ctrl_x_mode_line_or_eval(void);
extern int rs_ins_compl_active(void);
extern int rs_ins_compl_accept_char(int c);
extern int rs_ins_compl_used_match(void);
extern int rs_ins_compl_enter_selects(void);
extern int rs_ins_compl_col(void);
extern int rs_ins_compl_preinsert_effect(void);
extern int rs_ins_compl_has_autocomplete(void);
extern int rs_ins_compl_is_match_selected(void);
extern int rs_ins_compl_preinsert_longest(void);
extern int rs_ins_compl_has_shown_match(void);
extern int rs_ins_compl_long_shown_match(void);
extern int rs_pum_wanted(void);
extern void rs_ins_compl_init_get_longest(void);
extern void rs_ins_compl_enable_autocomplete(void);

typedef struct {
  VimState state;
  cmdarg_T *ca;
  int mincol;
  int cmdchar;
  int cmdchar_todo;                  // cmdchar to handle once in init_prompt
  bool ins_just_started;
  int startln;
  int count;
  int c;
  int lastc;
  int i;
  bool did_backspace;                // previous char was backspace
  bool line_is_white;                // line is empty before insert
  linenr_T old_topline;              // topline before insertion
  int old_topfill;
  int inserted_space;                // just inserted a space
  int replaceState;
  int did_restart_edit;              // remember if insert mode was restarted
                                     // after a ctrl+o
  bool nomove;
} InsertState;

// Forward declarations for functions now implemented in Rust (dispatch.rs / enter.rs).
extern int insert_handle_key(InsertState *s);
extern void insert_do_complete(InsertState *s);
extern void insert_do_cindent(InsertState *s);
extern void insert_handle_key_post(InsertState *s);
extern void insert_enter(InsertState *s);

#include "edit.c.generated.h"

extern int rs_get_scrolloff_value(win_T *wp);

// Rust fold FFI declarations
extern void rs_foldOpenCursor(void);
extern void rs_foldCheckClose(void);

enum {
  BACKSPACE_CHAR = 1,
  BACKSPACE_WORD = 2,
  BACKSPACE_WORD_NOT_SPACE = 3,
  BACKSPACE_LINE = 4,
};

/// Set when doing something for completion that may call edit() recursively,
/// which is not allowed.
static bool compl_busy = false;

static colnr_T Insstart_textlen;        // length of line when insert started
static colnr_T Insstart_blank_vcol;     // vcol for first inserted blank
static bool update_Insstart_orig = true;  // set Insstart_orig to Insstart

/// the text of the previous insert, K_SPECIAL is escaped
static String last_insert = STRING_INIT;
static int last_insert_skip;            // nr of chars in front of previous insert
static int new_insert_skip;             // nr of chars in front of current insert
static int did_restart_edit;            // "restart_edit" when calling edit()

static bool can_cindent;                // may do cindenting on this line

static bool revins_on;                  // reverse insert mode on
static int revins_chars;                // how much to skip after edit
static int revins_legal;                // was the last char 'legal'?
static int revins_scol;                 // start column of revins session

static bool ins_need_undo;              // call u_save() before inserting a
                                        // char.  Set when edit() is called.
                                        // after that arrow_used is used.

typedef struct {
  char *data;
  size_t size;
} RsNvimString;

// Rust FFI declarations (only functions called directly in this file)
extern RsNvimString rs_get_last_insert(void);
extern void rs_replace_stack_clear(void);
extern void ins_ctrl_v(void);
extern void rs_clear_showcmd(void);
extern void rs_start_selection(void);

// Phase 7: insert_check and insert_execute Rust implementations
extern int insert_check_rs(VimState *state);
extern int insert_execute_rs(VimState *state, int key);

// Phase 2: ins_esc and ins_reg are now implemented in Rust.
// Declaring them extern so C wrappers can call through to Rust-exported symbols.
// NOTE: ins_esc returns int (not bool) to match Rust c_int ABI.
extern int ins_esc(int *count, int cmdchar, int nomove);
extern void ins_reg(void);

// Rust functions exported with canonical C names (Phase 1+2 wrappers eliminated)
// Phase 1: static wrappers
extern void insert_special(int c, int allow_modmask, int ctrlv);
extern void start_arrow_with_change(pos_T *end_insert_pos, bool end_change);
extern void check_spell_redraw(void);
extern int echeck_abbr(int c);
extern int replace_pop_if_nul(void);
extern void replace_join(int off);
extern void replace_pop_ins(void);
extern void mb_replace_pop_ins(void);
extern void replace_do_bs(int limit_col);
extern void ins_ctrl_o(void);
extern int ins_start_select(int c);
extern char *do_insert_char_pre(int c);
extern int del_char_after_col(int limit_col);
// Phase 2: public wrappers
extern void undisplay_dollar(void);
extern void backspace_until_column(int col);
extern int get_literal(bool no_simplify);
extern void start_arrow(pos_T *end_insert_pos);
extern int stop_arrow(void);
extern void set_last_insert(int c);
extern void free_last_insert(void);
extern void beginline(int flags);
extern int oneright(void);
extern int oneleft(void);
extern int cursor_up(linenr_T n, bool upd_topline);
extern int cursor_down(int n, bool upd_topline);
extern int stuff_inserted(int c, int count, int no_esc);
extern char *get_last_insert_save(void);
extern void replace_push_nul(void);
extern int ins_copychar(linenr_T lnum);
extern colnr_T get_nolist_virtcol(void);
extern char *buf_prompt_text(const buf_T *buf);
extern char *prompt_text(void);

/// Get the no_abbr global variable (accessor for Rust).
int nvim_get_no_abbr(void)
{
  return no_abbr;
}

// nvim_get_p_paste is defined in indent_c.c

/// Get the ins_need_undo static variable (accessor for Rust).
int nvim_get_ins_need_undo(void)
{
  return ins_need_undo;
}

/// Set the ins_need_undo static variable (accessor for Rust).
void nvim_set_ins_need_undo(int val)
{
  ins_need_undo = val != 0;
}

/// Get the can_cindent static variable (accessor for Rust).
int nvim_get_can_cindent(void)
{
  return can_cindent;
}

/// Set the can_cindent static variable (accessor for Rust).
void nvim_set_can_cindent(int val)
{
  can_cindent = val != 0;
}

/// Get the revins_on static variable (accessor for Rust).
int nvim_get_revins_on(void)
{
  return revins_on;
}

/// Get the did_restart_edit static variable (accessor for Rust).
int nvim_get_did_restart_edit(void)
{
  return did_restart_edit;
}

/// Get buf->b_prompt_text (accessor for Rust).
char *nvim_buf_get_b_prompt_text(const buf_T *buf)
{
  return buf->b_prompt_text;
}

/// Get curwin->w_cursor.lnum (accessor for Rust).
linenr_T nvim_curwin_get_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// Get curwin->w_cursor.col (accessor for Rust).
colnr_T nvim_curwin_get_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// Get curbuf->b_prompt_start.mark.lnum (accessor for Rust).
linenr_T nvim_curbuf_get_b_prompt_start_lnum(void)
{
  return curbuf->b_prompt_start.mark.lnum;
}

/// Get Insstart.lnum (accessor for Rust).
linenr_T nvim_get_Insstart_lnum(void)
{
  return Insstart.lnum;
}

/// Get Insstart.col (accessor for Rust).
colnr_T nvim_get_Insstart_col(void)
{
  return Insstart.col;
}

/// Set Insstart (accessor for Rust).
void nvim_set_Insstart(linenr_T lnum, colnr_T col)
{
  Insstart.lnum = lnum;
  Insstart.col = col;
}

/// Get Insstart_orig.lnum (accessor for Rust).
linenr_T nvim_get_Insstart_orig_lnum(void)
{
  return Insstart_orig.lnum;
}

/// Get Insstart_orig.col (accessor for Rust).
colnr_T nvim_get_Insstart_orig_col(void)
{
  return Insstart_orig.col;
}

/// Set Insstart_orig (accessor for Rust).
void nvim_set_Insstart_orig(linenr_T lnum, colnr_T col)
{
  Insstart_orig.lnum = lnum;
  Insstart_orig.col = col;
}

/// Get Insstart_textlen (accessor for Rust).
colnr_T nvim_get_Insstart_textlen(void)
{
  return Insstart_textlen;
}

/// Set Insstart_textlen (accessor for Rust).
void nvim_set_Insstart_textlen(colnr_T val)
{
  Insstart_textlen = val;
}

/// Get Insstart_blank_vcol (accessor for Rust).
colnr_T nvim_get_Insstart_blank_vcol(void)
{
  return Insstart_blank_vcol;
}

/// Set Insstart_blank_vcol (accessor for Rust).
void nvim_set_Insstart_blank_vcol(colnr_T val)
{
  Insstart_blank_vcol = val;
}

/// Get update_Insstart_orig (accessor for Rust).
int nvim_get_update_Insstart_orig(void)
{
  return update_Insstart_orig;
}

/// Set update_Insstart_orig (accessor for Rust).
void nvim_set_update_Insstart_orig(int val)
{
  update_Insstart_orig = val != 0;
}

/// Get revins_chars (accessor for Rust).
int nvim_get_revins_chars(void)
{
  return revins_chars;
}

/// Set revins_chars (accessor for Rust).
void nvim_set_revins_chars(int val)
{
  revins_chars = val;
}

/// Get revins_legal (accessor for Rust).
int nvim_get_revins_legal(void)
{
  return revins_legal;
}

/// Set revins_legal (accessor for Rust).
void nvim_set_revins_legal(int val)
{
  revins_legal = val;
}

/// Get revins_scol (accessor for Rust).
int nvim_get_revins_scol(void)
{
  return revins_scol;
}

/// Set revins_scol (accessor for Rust).
void nvim_set_revins_scol(int val)
{
  revins_scol = val;
}

/// Set did_restart_edit (accessor for Rust).
void nvim_set_did_restart_edit(int val)
{
  did_restart_edit = val;
}

/// Get compl_busy (accessor for Rust).
int nvim_get_compl_busy(void)
{
  return compl_busy;
}

/// Get last_insert_skip (accessor for Rust).
int nvim_get_last_insert_skip(void)
{
  return last_insert_skip;
}

/// Get new_insert_skip (accessor for Rust).
int nvim_get_new_insert_skip(void)
{
  return new_insert_skip;
}

/// Set new_insert_skip (accessor for Rust).
void nvim_set_new_insert_skip(int val)
{
  new_insert_skip = val;
}

static TriState dont_sync_undo = kFalse;  // CTRL-G U prevents syncing undo
                                          // for the next left/right cursor key

static linenr_T o_lnum = 0;

/// Get dont_sync_undo (accessor for Rust).
int nvim_get_dont_sync_undo(void)
{
  return dont_sync_undo;
}

/// Set dont_sync_undo (accessor for Rust).
void nvim_set_dont_sync_undo(int val)
{
  dont_sync_undo = (TriState)val;
}

/// Get o_lnum (accessor for Rust).
linenr_T nvim_get_o_lnum(void)
{
  return o_lnum;
}

/// Set o_lnum (accessor for Rust).
void nvim_set_o_lnum(linenr_T val)
{
  o_lnum = val;
}

/// Set State (accessor for Rust).
void nvim_edit_set_State(int val)
{
  State = val;
}

/// Set restart_edit (accessor for Rust).
void nvim_edit_set_restart_edit(int val)
{
  restart_edit = val;
}

/// Get arrow_used (accessor for Rust).
int nvim_get_arrow_used(void)
{
  return arrow_used;
}

/// Set arrow_used (accessor for Rust).
void nvim_set_arrow_used(int val)
{
  arrow_used = val != 0;
}

/// Get p_ri (accessor for Rust).
int nvim_get_p_ri(void)
{
  return p_ri;
}

/// Get p_ari (allowrevins option) (accessor for Rust).
int nvim_get_p_ari(void)
{
  return p_ari;
}

// ============================================================================
// Accessors for helpers.rs (Phase 1)
// ============================================================================

/// Check if curwin buffer is valid for cursor operations (accessor for Rust).
int nvim_curwin_buf_valid(void)
{
  return curwin->w_buffer != NULL && curwin->w_buffer->b_ml.ml_mfp != NULL;
}

/// Get curwin buffer line count (accessor for Rust).
linenr_T nvim_curwin_buf_line_count(void)
{
  return curwin->w_buffer->b_ml.ml_line_count;
}

/// Get curwin->w_p_list (accessor for Rust).
int nvim_curwin_w_p_list(void)
{
  return curwin->w_p_list;
}

/// Check if p_cpo contains CPO_LISTWM ('L') (accessor for Rust).
int nvim_p_cpo_has_listwm(void)
{
  return vim_strchr(p_cpo, CPO_LISTWM) != NULL;
}

/// Get getvcol_nolist(&curwin->w_cursor) (accessor for Rust).
colnr_T nvim_getvcol_nolist(void)
{
  return getvcol_nolist(&curwin->w_cursor);
}

/// Run validate_virtcol(curwin) (accessor for Rust).
void nvim_validate_virtcol_curwin(void)
{
  validate_virtcol(curwin);
}

/// Get curwin->w_virtcol (accessor for Rust).
colnr_T nvim_curwin_get_w_virtcol(void)
{
  return curwin->w_virtcol;
}

/// Get get_cursor_pos_ptr() (accessor for Rust).
char *nvim_get_cursor_pos_ptr(void)
{
  return get_cursor_pos_ptr();
}

/// Get last_insert.data (accessor for Rust).
char *nvim_get_last_insert_data(void)
{
  return last_insert.data;
}

/// Get last_insert.size (accessor for Rust).
size_t nvim_get_last_insert_size(void)
{
  return last_insert.size;
}

/// Set last_insert data and size (accessor for Rust).
void nvim_set_last_insert(char *data, size_t size)
{
  last_insert.data = data;
  last_insert.size = size;
}

/// Clear last_insert (free and zero) (accessor for Rust).
void nvim_clear_last_insert(void)
{
  API_CLEAR_STRING(last_insert);
}

/// Set last_insert_skip (accessor for Rust).
void nvim_set_last_insert_skip(int val)
{
  last_insert_skip = val;
}

// Static asserts for constants used in Rust helpers/replace modules
_Static_assert(REPLACE_FLAG == 0x100, "REPLACE_FLAG mismatch");
_Static_assert(VREPLACE_FLAG == 0x200, "VREPLACE_FLAG mismatch");
_Static_assert(MODE_NORMAL == 0x01, "MODE_NORMAL mismatch");
_Static_assert(MB_MAXBYTES == 21, "MB_MAXBYTES mismatch");
_Static_assert(Ctrl_V == 22, "Ctrl_V mismatch");
_Static_assert(DEL == 0x7f, "DEL mismatch");
_Static_assert(ESC == '\033', "ESC mismatch");

/// Get replace_offset global (accessor for Rust).
int nvim_get_replace_offset(void)
{
  return replace_offset;
}

/// Get &curwin->w_cursor as opaque pointer (accessor for Rust).
const void *nvim_curwin_get_cursor_ptr(void)
{
  return &curwin->w_cursor;
}

// -- Phase 3: Movement module accessors --

/// Get curwin->w_cursor.coladd (accessor for Rust).
colnr_T nvim_edit_get_cursor_coladd(void)
{
  return curwin->w_cursor.coladd;
}

/// Set curwin->w_cursor.coladd (accessor for Rust).
void nvim_edit_set_cursor_coladd(colnr_T val)
{
  curwin->w_cursor.coladd = val;
}

/// Get curwin->w_curswant (accessor for Rust).
colnr_T nvim_edit_get_w_curswant(void)
{
  return curwin->w_curswant;
}

/// Set curwin->w_set_curswant (accessor for Rust).
void nvim_edit_set_w_set_curswant(int val)
{
  curwin->w_set_curswant = (val != 0);
}

/// Call coladvance(curwin, col) (accessor for Rust).
void nvim_edit_coladvance(colnr_T col)
{
  coladvance(curwin, col);
}

/// Call adjust_skipcol() (accessor for Rust).
void nvim_edit_adjust_skipcol(void)
{
  adjust_skipcol();
}

/// Call getviscol() (accessor for Rust).
colnr_T nvim_edit_getviscol(void)
{
  return getviscol();
}

/// Call virtual_active(curwin) (accessor for Rust).
int nvim_edit_virtual_active(void)
{
  return virtual_active(curwin) ? 1 : 0;
}

/// Get get_ve_flags(curwin) (accessor for Rust).
int nvim_edit_get_ve_flags(void)
{
  return (int)get_ve_flags(curwin);
}

/// Call vim_isprintc(c) (accessor for Rust).
int nvim_edit_vim_isprintc(int c)
{
  return vim_isprintc(c) ? 1 : 0;
}

/// Call ptr2cells(ptr) (accessor for Rust).
int nvim_edit_ptr2cells(const char *ptr)
{
  return ptr2cells(ptr);
}

/// Call utf_ptr2char(ptr) (accessor for Rust).
int nvim_edit_utf_ptr2char(const char *ptr)
{
  return utf_ptr2char(ptr);
}

/// Get wp->w_cursor.lnum (accessor for Rust).
linenr_T nvim_edit_win_get_cursor_lnum(win_T *wp)
{
  return wp->w_cursor.lnum;
}

/// Set wp->w_cursor.lnum (accessor for Rust).
void nvim_edit_win_set_cursor_lnum(win_T *wp, linenr_T lnum)
{
  wp->w_cursor.lnum = lnum;
}

/// Get wp->w_buffer->b_ml.ml_line_count (accessor for Rust).
linenr_T nvim_edit_win_get_buf_line_count(win_T *wp)
{
  return wp->w_buffer->b_ml.ml_line_count;
}

/// Get fdo_flags global (accessor for Rust).
int nvim_edit_get_fdo_flags(void)
{
  return (int)fdo_flags;
}

/// Call hasFoldingWin(wp, lnum, firstp, lastp, true, NULL) (accessor for Rust).
int nvim_edit_hasFoldingWin(win_T *wp, linenr_T lnum, linenr_T *firstp, linenr_T *lastp)
{
  return hasFoldingWin(wp, lnum, firstp, lastp, true, NULL) ? 1 : 0;
}

/// Call update_topline(curwin) (accessor for Rust).
void nvim_edit_update_topline(void)
{
  update_topline(curwin);
}

// Static asserts for Phase 3 constants
_Static_assert(BL_WHITE == 1, "BL_WHITE mismatch");
_Static_assert(BL_SOL == 2, "BL_SOL mismatch");
_Static_assert(BL_FIX == 4, "BL_FIX mismatch");
_Static_assert(OK == 1, "OK mismatch");
_Static_assert(FAIL == 0, "FAIL mismatch");
_Static_assert(MODE_INSERT == 0x10, "MODE_INSERT mismatch");
_Static_assert(kOptFdoFlagAll == 0x01, "kOptFdoFlagAll mismatch");
_Static_assert(kOptVeFlagOnemore == 0x08, "kOptVeFlagOnemore mismatch");
_Static_assert(TAB == '\011', "TAB mismatch");

// Static asserts for Wave 3 constants
_Static_assert(VREPLACE_FLAG == 0x200, "VREPLACE_FLAG mismatch");
_Static_assert(MODE_CMDLINE == 0x08, "MODE_CMDLINE mismatch");
_Static_assert(MOD_MASK_SHIFT == 0x02, "MOD_MASK_SHIFT mismatch");
_Static_assert(MOD_MASK_CMD == 0x80, "MOD_MASK_CMD mismatch");
_Static_assert(INSCHAR_CTRLV == 4, "INSCHAR_CTRLV mismatch");
_Static_assert(Ctrl_G == 7, "Ctrl_G mismatch");
_Static_assert(Ctrl_C == 3, "Ctrl_C mismatch");
_Static_assert(Ctrl_RSB == 29, "Ctrl_RSB mismatch");
_Static_assert(MB_MAXBYTES == 21, "MB_MAXBYTES mismatch");

// -- Wave 3: Global accessors for insert mode helpers --

/// Get spell_redraw_lnum (accessor for Rust).
linenr_T nvim_edit_get_spell_redraw_lnum(void)
{
  return spell_redraw_lnum;
}

/// Set spell_redraw_lnum (accessor for Rust).
void nvim_edit_set_spell_redraw_lnum(linenr_T val)
{
  spell_redraw_lnum = val;
}

/// Get ai_col (accessor for Rust).
colnr_T nvim_edit_get_ai_col(void)
{
  return ai_col;
}

/// Set ai_col (accessor for Rust).
void nvim_edit_set_ai_col(colnr_T val)
{
  ai_col = val;
}

/// Get orig_line_count (accessor for Rust).
linenr_T nvim_edit_get_orig_line_count(void)
{
  return orig_line_count;
}

/// Set orig_line_count (accessor for Rust).
void nvim_edit_set_orig_line_count(linenr_T val)
{
  orig_line_count = val;
}

/// Set vr_lines_changed (accessor for Rust).
void nvim_edit_set_vr_lines_changed(int val)
{
  vr_lines_changed = val;
}

/// Increment no_mapping (accessor for Rust).
void nvim_edit_inc_no_mapping(void)
{
  no_mapping++;
}

/// Decrement no_mapping (accessor for Rust).
void nvim_edit_dec_no_mapping(void)
{
  no_mapping--;
}

/// Get got_int (accessor for Rust).
int nvim_edit_get_got_int(void)
{
  return got_int ? 1 : 0;
}

/// Set got_int (accessor for Rust).
void nvim_edit_set_got_int(int val)
{
  got_int = val != 0;
}

/// Get mod_mask (accessor for Rust).
int nvim_edit_get_mod_mask(void)
{
  return mod_mask;
}

/// Set mod_mask (accessor for Rust).
void nvim_edit_set_mod_mask(int val)
{
  mod_mask = val;
}

/// Increment textlock (accessor for Rust).
void nvim_edit_textlock_inc(void)
{
  textlock++;
}

/// Decrement textlock (accessor for Rust).
void nvim_edit_textlock_dec(void)
{
  textlock--;
}

/// Call AppendToRedobuff(s) (accessor for Rust).
void nvim_edit_AppendToRedobuff(const char *s)
{
  AppendToRedobuff(s);
}

/// Call AppendToRedobuffLit(s, len) (accessor for Rust).
void nvim_edit_AppendToRedobuffLit(const char *s, int len)
{
  AppendToRedobuffLit(s, len);
}

/// Call ResetRedobuff() (accessor for Rust).
void nvim_edit_ResetRedobuff(void)
{
  ResetRedobuff();
}

/// Call u_save_cursor() (accessor for Rust).
int nvim_edit_u_save_cursor(void)
{
  return u_save_cursor();
}

/// Set Insstart from curwin->w_cursor (accessor for Rust).
void nvim_edit_set_insstart_from_cursor(void)
{
  Insstart = curwin->w_cursor;
}

/// Check if Insstart.col > Insstart_orig.col (accessor for Rust).
int nvim_edit_insstart_col_gt_orig(void)
{
  return Insstart.col > Insstart_orig.col ? 1 : 0;
}

/// Get linetabsize_str(get_cursor_line_ptr()) (accessor for Rust).
colnr_T nvim_edit_linetabsize_cursor_line(void)
{
  return linetabsize_str(get_cursor_line_ptr());
}

/// Get curbuf->b_ml.ml_line_count (accessor for Rust).
linenr_T nvim_edit_curbuf_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}


/// Call plain_vgetc() (accessor for Rust).
int nvim_edit_plain_vgetc(void)
{
  return plain_vgetc();
}

/// Call merge_modifiers(c, &mod_mask) — mutates global mod_mask (accessor for Rust).
int nvim_edit_merge_modifiers(int c)
{
  return merge_modifiers(c, &mod_mask);
}

/// Call add_to_showcmd(c) (accessor for Rust).
void nvim_edit_add_to_showcmd(int c)
{
  add_to_showcmd(c);
}

/// Call MB_BYTE2LEN_CHECK(c) (accessor for Rust).
int nvim_edit_MB_BYTE2LEN_CHECK(int c)
{
  return MB_BYTE2LEN_CHECK(c);
}

/// Call vungetc(c) (accessor for Rust).
void nvim_edit_vungetc(int c)
{
  vungetc(c);
}

/// Get K_ZERO computed macro value (accessor for Rust).
int nvim_edit_get_K_ZERO(void)
{
  return K_ZERO;
}

/// Get special key name string (accessor for Rust).
char *nvim_edit_get_special_key_name(int c, int modifiers)
{
  return get_special_key_name(c, modifiers);
}

/// Call ins_str(p, len) (accessor for Rust).
void nvim_edit_ins_str(const char *p, size_t len)
{
  ins_str((char *)p, len);
}

// -- Phase 3 accessors (for insertchar Rust migration) --

/// Call comp_textwidth(ff) (accessor for Rust).
int nvim_edit_comp_textwidth(int ff)
{
  return comp_textwidth((bool)ff);
}

/// Call internal_format(textwidth, second_indent, flags, format_only, c) (accessor for Rust).
void nvim_edit_internal_format(int textwidth, int second_indent, int flags, int format_only, int c)
{
  internal_format(textwidth, second_indent, flags, (bool)format_only, c);
}

/// Call fex_format(lnum, count, c) (accessor for Rust).
int nvim_edit_fex_format(linenr_T lnum, long count, int c)
{
  return fex_format(lnum, count, c);
}

/// Call char2cells(c) (accessor for Rust).
int nvim_edit_char2cells(int c)
{
  return char2cells(c);
}

/// Call gchar_cursor() (accessor for Rust).
int nvim_edit_gchar_cursor(void)
{
  return gchar_cursor();
}

/// Call byte2cells(b) (accessor for Rust).
int nvim_edit_byte2cells(int b)
{
  return byte2cells((uint8_t)b);
}

/// Call vpeekc() (accessor for Rust).
int nvim_edit_vpeekc(void)
{
  return vpeekc();
}

/// Call vgetc() (accessor for Rust).
int nvim_edit_vgetc(void)
{
  return vgetc();
}

/// Call do_digraph(c) (accessor for Rust).
int nvim_edit_do_digraph(int c)
{
  return do_digraph(c);
}

/// Call ins_char(c) (accessor for Rust).
void nvim_edit_ins_char(int c)
{
  ins_char(c);
}

/// Call ins_char_bytes(buf, charlen) (accessor for Rust).
void nvim_edit_ins_char_bytes(const char *buf, size_t charlen)
{
  ins_char_bytes((char *)buf, charlen);
}

/// Call utf_char2len(c) (accessor for Rust).
int nvim_edit_utf_char2len(int c)
{
  return utf_char2len(c);
}

/// Check if curbuf->b_p_fex (formatexpr) is non-empty (accessor for Rust).
int nvim_edit_has_b_p_fex(void)
{
  return *curbuf->b_p_fex != NUL ? 1 : 0;
}

/// Handle the end_comment_pending char replacement for insertchar (accessor for Rust).
/// This is the complex comment-leader removal section from insertchar().
void nvim_edit_handle_end_comment_pending(int c)
{
  char *p;
  char lead_end[COM_MAX_LEN];  // end-comment string

  // Need to remove existing (middle) comment leader and insert end
  // comment leader.  First, check what comment leader we can find.
  char *line = get_cursor_line_ptr();
  int i = get_leader_len(line, &p, false, true);
  if (i > 0 && vim_strchr(p, COM_MIDDLE) != NULL) {  // Just checking
    // Skip middle-comment string
    while (*p && p[-1] != ':') {  // find end of middle flags
      p++;
    }
    int middle_len = (int)copy_option_part(&p, lead_end, COM_MAX_LEN, ",");
    // Don't count trailing white space for middle_len
    while (middle_len > 0 && ascii_iswhite(lead_end[middle_len - 1])) {
      middle_len--;
    }

    // Find the end-comment string
    while (*p && p[-1] != ':') {  // find end of end flags
      p++;
    }
    int end_len = (int)copy_option_part(&p, lead_end, COM_MAX_LEN, ",");

    // Skip white space before the cursor
    i = curwin->w_cursor.col;
    while (--i >= 0 && ascii_iswhite(line[i])) {}
    i++;

    // Skip to before the middle leader
    i -= middle_len;

    // Check some expected things before we go on
    if (i >= 0 && end_len > 0
        && (uint8_t)lead_end[end_len - 1] == end_comment_pending) {
      // Backspace over all the stuff we want to replace
      backspace_until_column(i);

      // Insert the end-comment string, except for the last
      // character, which will get inserted as normal later.
      ins_bytes_len(lead_end, (size_t)(end_len - 1));
    }
  }
}

/// Call stop_insert logic for end_insert_pos (accessor for Rust).
void nvim_edit_stop_insert(void *end_insert_pos, int esc, int nomove)
{
  pos_T *pos = (pos_T *)end_insert_pos;
  stop_redo_ins();
  rs_replace_stack_clear();

  // Save inserted text for redo (^@ / CTRL-A).
  String inserted = get_inserted();
  int added = inserted.data == NULL ? 0 : (int)inserted.size - new_insert_skip;
  if (did_restart_edit == 0 || added > 0) {
    xfree(last_insert.data);
    last_insert = inserted;
    last_insert_skip = added < 0 ? 0 : new_insert_skip;
  } else {
    xfree(inserted.data);
  }

  if (!arrow_used && pos != NULL) {
    // Auto-format + strip trailing auto-indent whitespace.
    int cc;
    if (!ins_need_undo && has_format_option(FO_AUTO)) {
      pos_T tpos = curwin->w_cursor;
      cc = 'x';
      if (curwin->w_cursor.col > 0 && gchar_cursor() == NUL) {
        dec_cursor();
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          curwin->w_cursor = tpos;
        }
      }
      auto_format(true, false);
      if (ascii_iswhite(cc)) {
        if (gchar_cursor() != NUL) {
          inc_cursor();
        }
        if (gchar_cursor() == NUL
            && curwin->w_cursor.lnum == tpos.lnum
            && curwin->w_cursor.col == tpos.col) {
          curwin->w_cursor.coladd = tpos.coladd;
        }
      }
    }
    check_auto_format(true);
    if (!nomove && did_ai && (esc || (vim_strchr(p_cpo, CPO_INDENT) == NULL
                                      && curwin->w_cursor.lnum != pos->lnum))
        && pos->lnum <= curbuf->b_ml.ml_line_count) {
      pos_T tpos = curwin->w_cursor;
      colnr_T prev_col = pos->col;
      curwin->w_cursor = *pos;
      check_cursor_col(curwin);
      while (true) {
        if (gchar_cursor() == NUL && curwin->w_cursor.col > 0) {
          curwin->w_cursor.col--;
        }
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          break;
        }
        if (del_char(true) == FAIL) {
          break;
        }
      }
      if (curwin->w_cursor.lnum != tpos.lnum) {
        curwin->w_cursor = tpos;
      } else if (curwin->w_cursor.col < prev_col) {
        tpos = curwin->w_cursor;
        tpos.col++;
        if (cc != NUL && gchar_pos(&tpos) == NUL) {
          curwin->w_cursor.col++;
        }
      }
      if (VIsual_active) {
        check_visual_pos();
      }
    }
  }

  did_ai = false;
  did_si = false;
  can_si = false;
  can_si_back = false;
  if (pos != NULL) {
    curbuf->b_op_start = Insstart;
    curbuf->b_op_start_orig = Insstart_orig;
    curbuf->b_op_end = *pos;
  }
}

/// Check has_event(EVENT_INSERTCHARPRE) (accessor for Rust).
int nvim_edit_has_event_insertcharpre(void)
{
  return has_event(EVENT_INSERTCHARPRE) ? 1 : 0;
}

/// Call set_vim_var_string(VV_CHAR, buf, len) (accessor for Rust).
void nvim_edit_set_vim_var_char(const char *buf, ptrdiff_t len)
{
  set_vim_var_string(VV_CHAR, buf, len);
}

/// Get get_vim_var_str(VV_CHAR) (accessor for Rust).
const char *nvim_edit_get_vim_var_char(void)
{
  return get_vim_var_str(VV_CHAR);
}

/// Call ins_apply_autocmds(EVENT_INSERTCHARPRE) (accessor for Rust).
int nvim_edit_ins_apply_autocmds_insertcharpre(void)
{
  return ins_apply_autocmds(EVENT_INSERTCHARPRE);
}

// -- Phase 4: Key handler module accessors --

// Saved cursor positions for start_arrow calls (2 slots)
static pos_T edit_saved_cursor[2];
static linenr_T saved_topline;
static int saved_topfill;

/// Check fdo_flags & kOptFdoFlagHor && KeyTyped (accessor for Rust).
int nvim_edit_fdo_hor_and_key_typed(void)
{
  return ((fdo_flags & kOptFdoFlagHor) && KeyTyped) ? 1 : 0;
}

/// Save cursor position to a slot (accessor for Rust).
void nvim_edit_save_cursor(int slot)
{
  edit_saved_cursor[slot] = curwin->w_cursor;
}

/// Call start_arrow() with saved cursor slot (accessor for Rust).
void nvim_edit_start_arrow_from_slot(int slot)
{
  start_arrow(&edit_saved_cursor[slot]);
}

/// Call start_arrow_with_change() with saved cursor slot (accessor for Rust).
void nvim_edit_start_arrow_with_change_from_slot(int slot, int end_change)
{
  start_arrow_with_change(&edit_saved_cursor[slot], end_change != 0);
}

/// Call start_arrow(&curwin->w_cursor) (accessor for Rust).
void nvim_edit_start_arrow_curpos(void)
{
  start_arrow(&curwin->w_cursor);
}

/// Call start_arrow_with_change(&curwin->w_cursor, end_change) (accessor for Rust).
void nvim_edit_start_arrow_with_change_curpos(bool end_change)
{
  start_arrow_with_change(&curwin->w_cursor, end_change);
}

/// Call AppendCharToRedobuff(c) (accessor for Rust).
void nvim_edit_append_char_to_redobuff(int c)
{
  AppendCharToRedobuff(c);
}

/// Call vim_beep(val) (accessor for Rust).
void nvim_edit_vim_beep(int val)
{
  vim_beep((unsigned)val);
}

/// Check if p_ww (whichwrap) allows the given character (accessor for Rust).
int nvim_edit_ww_allows(int ch)
{
  return vim_strchr(p_ww, (char)ch) != NULL ? 1 : 0;
}

/// Adjust cursor lnum relative to current position (accessor for Rust).
void nvim_edit_set_cursor_lnum_rel(linenr_T delta)
{
  curwin->w_cursor.lnum += delta;
}

/// Set cursor lnum to an absolute value (accessor for Rust).
void nvim_edit_set_cursor_lnum_abs(linenr_T lnum)
{
  curwin->w_cursor.lnum = lnum;
}

/// Set curwin->w_curswant (accessor for Rust).
void nvim_edit_set_w_curswant(colnr_T val)
{
  curwin->w_curswant = val;
}

/// Get curbuf->b_ml.ml_line_count via curwin (accessor for Rust).
linenr_T nvim_edit_curwin_buf_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Call coladvance(curwin, getvcol_nolist(&Insstart)) (accessor for Rust).
void nvim_edit_coladvance_insstart(void)
{
  coladvance(curwin, getvcol_nolist(&Insstart));
}

/// Save topline/topfill for later comparison (accessor for Rust).
void nvim_edit_save_topline(void)
{
  saved_topline = curwin->w_topline;
  saved_topfill = curwin->w_topfill;
}

/// Check if topline/topfill changed since save (accessor for Rust).
int nvim_edit_topline_changed(void)
{
  return (saved_topline != curwin->w_topline
          || saved_topfill != curwin->w_topfill) ? 1 : 0;
}

/// Call redraw_later(curwin, UPD_VALID) (accessor for Rust).
void nvim_edit_redraw_later_valid(void)
{
  redraw_later(curwin, UPD_VALID);
}

/// Check if mod_mask has MOD_MASK_CTRL (accessor for Rust).
int nvim_edit_mod_mask_ctrl(void)
{
  return (mod_mask & MOD_MASK_CTRL) ? 1 : 0;
}

/// Check if there is a next tab page (accessor for Rust).
int nvim_edit_has_next_tabpage(void)
{
  return first_tabpage->tp_next != NULL ? 1 : 0;
}

/// Call pagescroll(BACKWARD, 1, false) and return result (accessor for Rust).
int nvim_edit_pagescroll_backward(void)
{
  return pagescroll(BACKWARD, 1, false);
}

/// Call pagescroll(FORWARD, 1, false) and return result (accessor for Rust).
int nvim_edit_pagescroll_forward(void)
{
  return pagescroll(FORWARD, 1, false);
}

// -- Phase 4b: Complex key handler delegated wrappers --

/// ins_insert() wrapper — handles set_vim_var_string, autocmds, mode change.
void nvim_edit_ins_insert(int replaceState)
{
  set_vim_var_string(VV_INSERTMODE, ((State & REPLACE_FLAG)
                                     ? "i"
                                     : replaceState == MODE_VREPLACE ? "v" : "r"), 1);
  ins_apply_autocmds(EVENT_INSERTCHANGE);
  if (State & REPLACE_FLAG) {
    State = MODE_INSERT | (State & MODE_LANGMAP);
  } else {
    State = replaceState | (State & MODE_LANGMAP);
  }
  may_trigger_modechanged();
  AppendCharToRedobuff(K_INS);
  showmode();
  ui_cursor_shape();
}

/// ins_ctrl_o() wrapper — sets restart_edit and ins_at_eol.
void nvim_edit_ins_ctrl_o(void)
{
  restart_VIsual_select = 0;
  if (State & VREPLACE_FLAG) {
    restart_edit = 'V';
  } else if (State & REPLACE_FLAG) {
    restart_edit = 'R';
  } else {
    restart_edit = 'I';
  }
  if (virtual_active(curwin)) {
    ins_at_eol = false;
  } else {
    ins_at_eol = (gchar_cursor() == NUL);
  }
}

/// ins_ctrl_hat() wrapper — toggles langmap mode.
void nvim_edit_ins_ctrl_hat(void)
{
  if (map_to_exists_mode("", MODE_LANGMAP, false)) {
    if (State & MODE_LANGMAP) {
      curbuf->b_p_iminsert = B_IMODE_NONE;
      State &= ~MODE_LANGMAP;
    } else {
      curbuf->b_p_iminsert = B_IMODE_LMAP;
      State |= MODE_LANGMAP;
    }
  }
  set_iminsert_global(curbuf);
  showmode();
  status_redraw_curbuf();
}

/// ins_ctrl_() helper — handles the state changes that need C access.
/// The Rust side computes revins_on, this function handles the rest.
void nvim_edit_ins_ctrl_(int new_revins_on)
{
  if (revins_on && revins_chars && revins_scol >= 0) {
    while (gchar_cursor() != NUL && revins_chars--) {
      curwin->w_cursor.col++;
    }
  }
  p_ri = !p_ri;
  revins_on = (new_revins_on != 0);
  if (revins_on) {
    revins_scol = curwin->w_cursor.col;
    revins_legal++;
    revins_chars = 0;
    undisplay_dollar();
  } else {
    revins_scol = -1;
  }
  showmode();
}

/// ins_start_select() wrapper — handles key constant switch and stuffing.
int nvim_edit_ins_start_select(int c)
{
  if (!km_startsel) {
    return 0;
  }
  switch (c) {
  case K_KHOME:
  case K_KEND:
  case K_PAGEUP:
  case K_KPAGEUP:
  case K_PAGEDOWN:
  case K_KPAGEDOWN:
    if (!(mod_mask & MOD_MASK_SHIFT)) {
      break;
    }
    FALLTHROUGH;
  case K_S_LEFT:
  case K_S_RIGHT:
  case K_S_UP:
  case K_S_DOWN:
  case K_S_END:
  case K_S_HOME:
    rs_start_selection();
    stuffcharReadbuff(Ctrl_O);
    if (mod_mask) {
      const char buf[] = { (char)K_SPECIAL, (char)KS_MODIFIER,
                           (char)(uint8_t)mod_mask, NUL };
      stuffReadbuffLen(buf, 3);
    }
    stuffcharReadbuff(c);
    return 1;
  }
  return 0;
}

/// ins_ctrl_g() get key helper — reads key and classifies it.
/// Returns: 1=up, 2=down, 3=u_sync, 4=no_sync, 5=ESC, 0=unknown
int nvim_edit_ins_ctrl_g_get_key(void)
{
  setcursor();
  no_mapping++;
  allow_keys++;
  int c = plain_vgetc();
  no_mapping--;
  allow_keys--;
  switch (c) {
  case K_UP:
  case Ctrl_K:
  case 'k':
    return 1;  // up
  case K_DOWN:
  case Ctrl_J:
  case 'j':
    return 2;  // down
  case 'u':
    return 3;  // u_sync
  case 'U':
    return 4;  // no_sync
  case ESC:
    return 5;  // ESC
  default:
    return 0;  // unknown
  }
}

/// ins_ctrl_g 'u' sync handler (accessor for Rust).
void nvim_edit_ctrl_g_u_sync(void)
{
  u_sync(true);
  ins_need_undo = true;
  update_Insstart_orig = false;
  Insstart = curwin->w_cursor;
}

/// ins_shift() wrapper — handles indent changes.
void nvim_edit_ins_shift(int c, int lastc)
{
  if (stop_arrow() == FAIL) {
    return;
  }
  AppendCharToRedobuff(c);

  if (c == Ctrl_D && (lastc == '0' || lastc == '^')
      && curwin->w_cursor.col > 0) {
    curwin->w_cursor.col--;
    del_char(false);
    if (State & REPLACE_FLAG) {
      replace_pop_ins();
    }
    if (lastc == '^') {
      old_indent = get_indent();
    }
    change_indent(INDENT_SET, 0, true, true);
  } else {
    change_indent(c == Ctrl_D ? INDENT_DEC : INDENT_INC, 0, true, true);
  }

  if (did_ai && *skipwhite(get_cursor_line_ptr()) != NUL) {
    did_ai = false;
  }
  did_si = false;
  can_si = false;
  can_si_back = false;
  can_cindent = false;
}

/// ins_del() wrapper — handles delete key in insert mode.
void nvim_edit_ins_del(void)
{
  if (stop_arrow() == FAIL) {
    return;
  }
  if (gchar_cursor() == NUL) {
    const int temp = curwin->w_cursor.col;
    if (!can_bs(BS_EOL)
        || do_join(2, false, true, false, false) == FAIL) {
      vim_beep(kOptBoFlagBackspace);
    } else {
      curwin->w_cursor.col = temp;
      if (State & VREPLACE_FLAG
          && orig_line_count > curbuf->b_ml.ml_line_count) {
        orig_line_count = curbuf->b_ml.ml_line_count;
      }
    }
  } else if (del_char(false) == FAIL) {
    vim_beep(kOptBoFlagBackspace);
  }
  did_ai = false;
  did_si = false;
  can_si = false;
  can_si_back = false;
  AppendCharToRedobuff(K_DEL);
}

// -- Phase 4 dispatch module accessors --

/// Check if pum (popup menu) is visible (accessor for Rust).
int nvim_edit_pum_visible(void)
{
  return pum_visible() ? 1 : 0;
}

/// Get pum_want.active (accessor for Rust).
int nvim_edit_get_pum_want_active(void)
{
  return pum_want.active ? 1 : 0;
}

/// Set pum_want.active (accessor for Rust).
void nvim_edit_set_pum_want_active(int val)
{
  pum_want.active = val != 0;
}

/// Get pum_want.finish (accessor for Rust).
int nvim_edit_get_pum_want_finish(void)
{
  return pum_want.finish ? 1 : 0;
}

/// Set edit_submode_extra to NULL (accessor for Rust).
void nvim_edit_clear_edit_submode_extra(void)
{
  edit_submode_extra = NULL;
}

/// Get cmdwin_type (accessor for Rust).
int nvim_edit_get_cmdwin_type(void)
{
  return cmdwin_type;
}

/// Set cmdwin_result (accessor for Rust).
void nvim_edit_set_cmdwin_result(int val)
{
  cmdwin_result = val;
}

/// Set ins_at_eol (accessor for Rust).
void nvim_edit_set_ins_at_eol(int val)
{
  ins_at_eol = val != 0;
}

/// Set did_cursorhold (accessor for Rust).
void nvim_edit_set_did_cursorhold(int val)
{
  did_cursorhold = val != 0;
}

/// Increment disable_fold_update (accessor for Rust).
void nvim_edit_inc_disable_fold_update(void)
{
  disable_fold_update++;
}

/// Decrement disable_fold_update (accessor for Rust).
void nvim_edit_dec_disable_fold_update(void)
{
  disable_fold_update--;
}

/// Set compl_busy (accessor for Rust).
void nvim_edit_set_compl_busy(int val)
{
  compl_busy = val != 0;
}

/// Call may_do_si() and assign to can_si (accessor for Rust).
void nvim_edit_update_can_si_from_may_do_si(void)
{
  can_si = may_do_si();
}

/// Call ins_complete(c, true) (accessor for Rust).
int nvim_edit_ins_complete(int c)
{
  return ins_complete(c, true);
}

/// Call check_compl_option(allow_always) (accessor for Rust).
int nvim_edit_check_compl_option(int allow_always)
{
  return check_compl_option(allow_always != 0) ? 1 : 0;
}

/// Call ins_ctrl_x() (accessor for Rust).
void nvim_edit_ins_ctrl_x(void)
{
  ins_ctrl_x();
}

/// Call do_cmdline(NULL, getcmdkeycmd, NULL, 0) (accessor for Rust).
void nvim_edit_do_cmdline_getcmdkeycmd(void)
{
  do_cmdline(NULL, getcmdkeycmd, NULL, 0);
}

/// Call map_execute_lua(false, false) (accessor for Rust).
void nvim_edit_map_execute_lua(void)
{
  map_execute_lua(false, false);
}

/// Call paste_repeat(1) (accessor for Rust).
void nvim_edit_paste_repeat(void)
{
  paste_repeat(1);
}

/// Call state_handle_k_event() (accessor for Rust).
void nvim_edit_state_handle_k_event(void)
{
  state_handle_k_event();
}

/// Check if curwin->w_llist_ref is NULL (for quickfix window check) (accessor for Rust).
int nvim_edit_curwin_is_qf_not_ll(void)
{
  return curwin->w_llist_ref == NULL ? 1 : 0;
}

/// Call do_cmdline_cmd(".cc") for quickfix (accessor for Rust).
void nvim_edit_quickfix_cc(void)
{
  do_cmdline_cmd(".cc");
}

/// Call do_cmdline_cmd(".ll") for location list (accessor for Rust).
void nvim_edit_quickfix_ll(void)
{
  do_cmdline_cmd(".ll");
}

/// Call invoke_prompt_interrupt() (accessor for Rust).
int nvim_edit_invoke_prompt_interrupt(void)
{
  return invoke_prompt_interrupt() ? 1 : 0;
}

/// Call prompt_invoke_callback() (accessor for Rust).
void nvim_edit_prompt_invoke_callback(void)
{
  prompt_invoke_callback();
}

/// Get curbuf->b_u_synced (accessor for Rust).
int nvim_edit_get_curbuf_b_u_synced(void)
{
  return curbuf->b_u_synced ? 1 : 0;
}

/// Get p_paste option (accessor for Rust).
int nvim_edit_get_p_paste(void)
{
  return p_paste ? 1 : 0;
}

/// Call char_before_cursor() (accessor for Rust).
int nvim_edit_char_before_cursor(void)
{
  return char_before_cursor();
}

/// Call char_avail() (accessor for Rust).
int nvim_edit_char_avail(void)
{
  return char_avail() ? 1 : 0;
}

/// Call inindent(0) (accessor for Rust).
int nvim_edit_inindent(void)
{
  return inindent(0) ? 1 : 0;
}

/// Call auto_format(false, force_format) (accessor for Rust).
void nvim_edit_auto_format(int force_format)
{
  auto_format(false, force_format != 0);
}

/// Call in_cinkeys(c, type, line_is_white) (accessor for Rust).
int nvim_edit_in_cinkeys(int c, int type, int line_is_white)
{
  return in_cinkeys(c, (char)type, line_is_white != 0) ? 1 : 0;
}

/// Call do_c_expr_indent() (accessor for Rust).
void nvim_edit_do_c_expr_indent(void)
{
  do_c_expr_indent();
}

/// Call ins_reg() (accessor for Rust).
void nvim_edit_ins_reg(void)
{
  ins_reg();
}

/// Call ins_try_si(c) (accessor for Rust).
void nvim_edit_ins_try_si(int c)
{
  ins_try_si(c);
}

/// Call update_screen() (accessor for Rust).
void nvim_edit_update_screen(void)
{
  update_screen();
}

/// Call ui_flush() (accessor for Rust).
void nvim_edit_ui_flush(void)
{
  ui_flush();
}

/// Check if bt_quickfix(curbuf) (accessor for Rust).
int nvim_edit_bt_quickfix_curbuf(void)
{
  return bt_quickfix(curbuf) ? 1 : 0;
}

/// Check if bt_prompt(curbuf) (accessor for Rust).
int nvim_edit_bt_prompt_curbuf(void)
{
  return bt_prompt(curbuf) ? 1 : 0;
}

/// Get curwin->w_p_rl (right-to-left option) (accessor for Rust).
int nvim_edit_get_curwin_p_rl(void)
{
  return curwin->w_p_rl ? 1 : 0;
}

/// Check if curwin->w_cursor.col >= rs_ins_compl_col() (accessor for Rust).
int nvim_edit_cursor_col_ge_compl_col(void)
{
  return curwin->w_cursor.col >= rs_ins_compl_col() ? 1 : 0;
}

/// Get curbuf->b_p_cpt (complete option), first char (accessor for Rust).
int nvim_edit_get_cpt_first_char(void)
{
  return (unsigned char)*curbuf->b_p_cpt;
}

/// Get vim_iswordc(c) (accessor for Rust -- thin wrapper to avoid name clashes).
int nvim_edit_vim_iswordc_dispatch(int c)
{
  return vim_iswordc(c) ? 1 : 0;
}

/// Get get_ve_flags(curwin) & kOptVeFlagOnemore (accessor for Rust).
int nvim_edit_ve_onemore(void)
{
  return (get_ve_flags(curwin) & kOptVeFlagOnemore) ? 1 : 0;
}

// Static asserts for Phase 4 constants
_Static_assert(kOptBoFlagCursor == 0x04, "kOptBoFlagCursor mismatch");
_Static_assert(kOptBoFlagCtrlg == 0x20, "kOptBoFlagCtrlg mismatch");
_Static_assert(kOptBoFlagBackspace == 0x02, "kOptBoFlagBackspace mismatch");
_Static_assert(kOptFdoFlagHor == 0x04, "kOptFdoFlagHor mismatch");
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL mismatch");
_Static_assert(K_LEFT == -((int)('k') + ((int)('l') << 8)), "K_LEFT mismatch");
_Static_assert(K_RIGHT == -((int)('k') + ((int)('r') << 8)), "K_RIGHT mismatch");
_Static_assert(K_S_LEFT == -((int)('#') + ((int)('4') << 8)), "K_S_LEFT mismatch");
_Static_assert(K_S_RIGHT == -((int)('%') + ((int)('i') << 8)), "K_S_RIGHT mismatch");
_Static_assert(K_C_HOME == -((int)(KS_EXTRA) + ((int)(87) << 8)), "K_C_HOME mismatch");
_Static_assert(K_C_END == -((int)(KS_EXTRA) + ((int)(88) << 8)), "K_C_END mismatch");

// -- Phase 5: Editing module accessors and delegated wrappers --

// Forward declarations for Phase 5 wrappers (definitions are later in the file)
static int pc_status;
#ifndef PC_STATUS_UNSET
#define PC_STATUS_UNSET 0
#endif

/// Delegated wrapper for ins_eol (Rust FFI export).
int nvim_edit_ins_eol(int c)
{
  if (echeck_abbr(c + ABBR_OFF)) {
    return true;
  }
  if (stop_arrow() == FAIL) {
    return false;
  }
  undisplay_dollar();

  if ((State & REPLACE_FLAG) && !(State & VREPLACE_FLAG)) {
    replace_push_nul();
  }

  if (virtual_active(curwin) && curwin->w_cursor.coladd > 0) {
    coladvance(curwin, getviscol());
  }

  if (revins_on) {
    curwin->w_cursor.col += get_cursor_pos_len();
  }

  AppendToRedobuff(NL_STR);
  bool i = open_line(FORWARD,
                     has_format_option(FO_RET_COMS) ? OPENLINE_DO_COM : 0,
                     old_indent, NULL);
  old_indent = 0;
  can_cindent = true;
  rs_foldOpenCursor();

  return i;
}

/// Delegated wrapper for ins_ctrl_v (Rust FFI export).
void nvim_edit_ins_ctrl_v(void)
{
  bool did_putchar = false;

  ins_redraw(false);

  if (redrawing() && !char_avail()) {
    edit_putchar('^', true);
    did_putchar = true;
  }
  AppendToRedobuff(CTRL_V_STR);

  add_to_showcmd_c(Ctrl_V);

  int c = get_literal(mod_mask & MOD_MASK_SHIFT);
  if (did_putchar) {
    edit_unputchar();
  }
  rs_clear_showcmd();
  insert_special(c, true, true);
  revins_chars++;
  revins_legal++;
}

/// Delegated wrapper for ins_copychar (Rust FFI export).
int nvim_edit_ins_copychar(linenr_T lnum)
{
  if (lnum < 1 || lnum > curbuf->b_ml.ml_line_count) {
    vim_beep(kOptBoFlagCopy);
    return NUL;
  }

  validate_virtcol(curwin);
  int const end_vcol = curwin->w_virtcol;
  char *line = ml_get(lnum);

  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, lnum, line);
  StrCharInfo ci = utf_ptr2StrCharInfo(line);
  int vcol = 0;
  while (vcol < end_vcol && *ci.ptr != NUL) {
    vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
    if (vcol > end_vcol) {
      break;
    }
    ci = utfc_next(ci);
  }

  int c = ci.chr.value < 0 ? (uint8_t)(*ci.ptr) : ci.chr.value;
  if (c == NUL) {
    vim_beep(kOptBoFlagCopy);
  }
  return c;
}

/// Delegated wrapper for ins_ctrl_ey (Rust FFI export).
int nvim_edit_ins_ctrl_ey(int tc)
{
  int c = tc;

  if (rs_ctrl_x_mode_scroll()) {
    if (c == Ctrl_Y) {
      scrolldown_clamp();
    } else {
      scrollup_clamp();
    }
    redraw_later(curwin, UPD_VALID);
  } else {
    c = ins_copychar(curwin->w_cursor.lnum + (c == Ctrl_Y ? -1 : 1));
    if (c != NUL) {
      if (c < 256 && !isalnum(c)) {
        AppendToRedobuff(CTRL_V_STR);
      }
      OptInt tw_save = curbuf->b_p_tw;
      curbuf->b_p_tw = -1;
      insert_special(c, true, false);
      curbuf->b_p_tw = tw_save;
      revins_chars++;
      revins_legal++;
      c = Ctrl_V;
      auto_format(false, true);
    }
  }
  return c;
}

/// Delegated wrapper for ins_digraph (Rust FFI export).
int nvim_edit_ins_digraph(void)
{
  bool did_putchar = false;

  pc_status = PC_STATUS_UNSET;
  if (redrawing() && !char_avail()) {
    ins_redraw(false);
    edit_putchar('?', true);
    did_putchar = true;
    add_to_showcmd_c(Ctrl_K);
  }

  no_mapping++;
  allow_keys++;
  int c = plain_vgetc();
  no_mapping--;
  allow_keys--;
  if (did_putchar) {
    edit_unputchar();
  }

  if (IS_SPECIAL(c) || mod_mask) {
    rs_clear_showcmd();
    insert_special(c, true, false);
    return NUL;
  }
  if (c != ESC) {
    did_putchar = false;
    if (redrawing() && !char_avail()) {
      ins_redraw(false);
      if (char2cells(c) == 1) {
        ins_redraw(false);
        edit_putchar(c, true);
        did_putchar = true;
      }
      add_to_showcmd_c(c);
    }
    no_mapping++;
    allow_keys++;
    int cc = plain_vgetc();
    no_mapping--;
    allow_keys--;
    if (did_putchar) {
      edit_unputchar();
    }
    if (cc != ESC) {
      AppendToRedobuff(CTRL_V_STR);
      c = digraph_get(c, cc, true);
      rs_clear_showcmd();
      return c;
    }
  }
  rs_clear_showcmd();
  return NUL;
}

/// Accessor for stuff_inserted: stuffcharReadbuff (for Rust).
void nvim_stuffcharReadbuff(int c)
{
  stuffcharReadbuff(c);
}

/// Accessor for stuff_inserted: stuffReadbuffLen (for Rust).
void nvim_stuffReadbuffLen(const char *data, ptrdiff_t len)
{
  stuffReadbuffLen(data, len);
}

// nvim_emsg_noinstext is already defined in register.c

// -- Phase 6: insert_enter accessors --

/// Get restart_edit (accessor for Rust, Phase 6).
/// Note: nvim_get_restart_edit is defined in cursor_shape.c; use this wrapper.
int nvim_edit_get_restart_edit(void)
{
  return restart_edit;
}

/// Get stop_insert_mode (accessor for Rust).
int nvim_edit_get_stop_insert_mode(void)
{
  return stop_insert_mode ? 1 : 0;
}

/// Set stop_insert_mode (accessor for Rust).
void nvim_edit_set_stop_insert_mode(int val)
{
  stop_insert_mode = (val != 0);
}

/// Set where_paste_started.lnum to 0 (accessor for Rust).
void nvim_edit_clear_where_paste_started(void)
{
  where_paste_started.lnum = 0;
}

/// Set Insstart from where_paste_started or curwin->w_cursor (accessor for Rust).
/// If where_paste_started.lnum != 0, use it; otherwise use curwin->w_cursor.
/// If startln is nonzero, set Insstart.col = 0.
void nvim_edit_init_Insstart(int startln)
{
  if (where_paste_started.lnum != 0) {
    Insstart = where_paste_started;
  } else {
    Insstart = curwin->w_cursor;
    if (startln) {
      Insstart.col = 0;
    }
  }
}

/// Get did_ai (accessor for Rust).
int nvim_edit_get_did_ai(void)
{
  return did_ai ? 1 : 0;
}

/// Get need_highlight_changed (accessor for Rust).
int nvim_edit_get_need_highlight_changed(void)
{
  return need_highlight_changed ? 1 : 0;
}

/// Check if cursor is on a TAB or inline virtual text (accessor for Rust).
int nvim_edit_cursor_on_tab_or_inline(void)
{
  return (gchar_cursor() == TAB || buf_meta_total(curbuf, kMTMetaInline) > 0) ? 1 : 0;
}

/// Invalidate WROW/WCOL/VIRTCOL in curwin->w_valid (accessor for Rust).
void nvim_edit_invalidate_wrow_wcol_virtcol(void)
{
  curwin->w_valid &= ~(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
}

/// Get curbuf->b_p_iminsert (accessor for Rust).
int nvim_edit_get_curbuf_b_p_iminsert(void)
{
  return curbuf->b_p_iminsert;
}

/// Set revins_on (accessor for Rust).
void nvim_edit_set_revins_on(int val)
{
  revins_on = (val != 0);
}

/// Set need_start_insertmode (accessor for Rust).
void nvim_edit_set_need_start_insertmode(int val)
{
  need_start_insertmode = (val != 0);
}

/// Get p_smd (showmode option) (accessor for Rust).
int nvim_edit_get_p_smd(void)
{
  return p_smd;
}

/// Get msg_silent (accessor for Rust).
int nvim_edit_get_msg_silent(void)
{
  return msg_silent;
}

/// Set old_indent (accessor for Rust).
void nvim_edit_set_old_indent(int val)
{
  old_indent = val;
}

/// Save curwin->w_cursor into out-params (accessor for Rust, Phase 6).
void nvim_edit_save_cursor_pos(linenr_T *lnum_out, colnr_T *col_out, colnr_T *coladd_out)
{
  *lnum_out = curwin->w_cursor.lnum;
  *col_out = curwin->w_cursor.col;
  *coladd_out = curwin->w_cursor.coladd;
}

/// Restore curwin->w_cursor from saved values (accessor for Rust, Phase 6).
void nvim_edit_restore_cursor_pos(linenr_T lnum, colnr_T col, colnr_T coladd)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
  curwin->w_cursor.coladd = coladd;
}

/// Check if current cursor equals saved pos (accessor for Rust, Phase 6).
int nvim_edit_cursor_equals_saved(linenr_T lnum, colnr_T col, colnr_T coladd)
{
  pos_T saved = { .lnum = lnum, .col = col, .coladd = coladd };
  return equalpos(curwin->w_cursor, saved) ? 1 : 0;
}

/// Get *get_vim_var_str(VV_CHAR) == NUL (accessor for Rust).
int nvim_edit_vv_char_is_empty(void)
{
  return (*get_vim_var_str(VV_CHAR) == NUL) ? 1 : 0;
}

/// Get curbuf->b_ml.ml_line_count (accessor for Rust).
int nvim_edit_get_curbuf_ml_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// Set State to MODE_INSERT temporarily, call check_cursor_col, restore State.
void nvim_edit_check_cursor_col_in_insert_mode(void)
{
  int save_state = State;
  State = MODE_INSERT;
  check_cursor_col(curwin);
  State = save_state;
}

/// Call set_vim_var_string(VV_INSERTMODE, ptr, 1) where ptr depends on cmdchar.
void nvim_edit_set_vv_insertmode(int cmdchar)
{
  const char *ptr = cmdchar == 'R' ? "r" : cmdchar == 'V' ? "v" : "i";
  set_vim_var_string(VV_INSERTMODE, ptr, 1);
}

/// Call set_vim_var_string(VV_CHAR, NULL, -1) (accessor for Rust).
void nvim_edit_clear_vv_char(void)
{
  set_vim_var_string(VV_CHAR, NULL, -1);
}

/// Call ins_apply_autocmds(EVENT_INSERTENTER) (accessor for Rust).
void nvim_edit_ins_apply_insertenter(void)
{
  ins_apply_autocmds(EVENT_INSERTENTER);
}

/// Call ins_apply_autocmds(EVENT_INSERTLEAVE) (accessor for Rust).
void nvim_edit_ins_apply_insertleave(void)
{
  ins_apply_autocmds(EVENT_INSERTLEAVE);
}

/// Get Insstart_textlen from linetabsize_str(get_cursor_line_ptr()) (accessor for Rust).
void nvim_edit_init_Insstart_textlen(void)
{
  Insstart_textlen = linetabsize_str(get_cursor_line_ptr());
  Insstart_blank_vcol = MAXCOL;
}

/// Get size of get_inserted() and free the result (for new_insert_skip).
int nvim_edit_get_inserted_size(void)
{
  String inserted = get_inserted();
  int sz = (int)inserted.size;
  if (inserted.data != NULL) {
    xfree(inserted.data);
  }
  return sz;
}

/// Update curbuf->b_last_changedtick if TextChangedI was triggered (accessor for Rust).
void nvim_curbuf_sync_changedtick_after_insert(void)
{
  if (!char_avail() && curbuf->b_last_changedtick_i == buf_get_changedtick(curbuf)) {
    curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  }
}

/// Update o_lnum if ins_at_eol (accessor for Rust).
void nvim_edit_update_o_lnum_if_at_eol(void)
{
  if (ins_at_eol) {
    o_lnum = curwin->w_cursor.lnum;
  }
}

/// Check restart_edit conditions and maybe advance cursor (accessor for Rust).
/// Returns: 0=arrow_used=false path, 1=restart_edit path (arrow_used set from paste).
int nvim_edit_handle_restart_edit_cursor(void)
{
  if (restart_edit != 0 && stuff_empty()) {
    arrow_used = where_paste_started.lnum == 0;
    restart_edit = 0;

    validate_virtcol(curwin);
    update_curswant();
    const char *ptr;
    if (((ins_at_eol && curwin->w_cursor.lnum == o_lnum)
         || curwin->w_curswant > curwin->w_virtcol)
        && *(ptr = get_cursor_line_ptr() + curwin->w_cursor.col) != NUL) {
      if (ptr[1] == NUL) {
        curwin->w_cursor.col++;
      } else {
        int i = utfc_ptr2len(ptr);
        if (ptr[i] == NUL) {
          curwin->w_cursor.col += i;
        }
      }
    }
    ins_at_eol = false;
    return 1;
  }
  arrow_used = false;
  return 0;
}

/// Call AppendNumberToRedobuff(n) (accessor for Rust).
void nvim_edit_AppendNumberToRedobuff(int n)
{
  AppendNumberToRedobuff(n);
}

/// Call showmode() and return result (accessor for Rust).
int nvim_edit_showmode(void)
{
  return showmode();
}

/// Call change_warning(curbuf, col) (accessor for Rust).
void nvim_edit_change_warning(int col)
{
  change_warning(curbuf, col);
}

/// Call pum_check_clear() (accessor for Rust).
void nvim_edit_pum_check_clear(void)
{
  pum_check_clear();
}

/// Call state_enter(&s->state) (accessor for Rust).
void nvim_edit_state_enter(void *state)
{
  state_enter((VimState *)state);
}

/// Call ins_esc(&count, cmdchar, nomove) (accessor for Rust).
/// ins_esc is now implemented in Rust (src/nvim-rs/edit/src/esc.rs).
int nvim_edit_ins_esc(int *count, int cmdchar, int nomove)
{
  return ins_esc(count, cmdchar, nomove);
}

/// Call msg_check_for_delay(true) (accessor for Rust).
void nvim_edit_msg_check_for_delay(void)
{
  msg_check_for_delay(true);
}

/// Call highlight_changed() (accessor for Rust).
void nvim_edit_highlight_changed(void)
{
  highlight_changed();
}

/// Call ui_cursor_shape() and do_digraph(-1) (accessor for Rust).
void nvim_edit_ui_cursor_shape_and_clear_digraph(void)
{
  ui_cursor_shape();
  do_digraph(-1);
}

// Static asserts for Phase 5 constants
_Static_assert(ABBR_OFF == 0x100, "ABBR_OFF mismatch");
_Static_assert(OPENLINE_DO_COM == 0x02, "OPENLINE_DO_COM mismatch");
_Static_assert(Ctrl_D == 4, "Ctrl_D mismatch");
_Static_assert(kOptBoFlagCopy == 0x10, "kOptBoFlagCopy mismatch");

// =============================================================================
// Phase 7: insert_check / insert_execute accessors (for state_machine.rs)
// =============================================================================

_Static_assert(kOptFdoFlagInsert == 0x100, "kOptFdoFlagInsert mismatch");

/// Set Insstart_orig to Insstart (accessor for Rust state_machine).
void nvim_set_Insstart_orig_from_Insstart(void)
{
  Insstart_orig = Insstart;
}

/// Check if curbuf->terminal is set (accessor for Rust state_machine).
int nvim_edit_curbuf_is_terminal(void)
{
  return curbuf->terminal != NULL ? 1 : 0;
}

/// Call stuffcharReadbuff(K_NOP) (accessor for Rust state_machine).
void nvim_stuffcharReadbuff_K_NOP(void)
{
  stuffcharReadbuff(K_NOP);
}

/// Get curwin->w_p_scb (accessor for Rust state_machine).
int nvim_edit_curwin_p_scb(void)
{
  return curwin->w_p_scb ? 1 : 0;
}

/// Get curwin->w_p_crb (accessor for Rust state_machine).
int nvim_edit_curwin_p_crb(void)
{
  return curwin->w_p_crb ? 1 : 0;
}

/// Get curwin->w_topline (accessor for Rust state_machine).
linenr_T nvim_edit_get_curwin_topline(void)
{
  return curwin->w_topline;
}

/// Get curwin->w_topfill (accessor for Rust state_machine).
int nvim_edit_get_curwin_topfill(void)
{
  return curwin->w_topfill;
}

/// Handle the scroll detection block from insert_check (composite accessor for Rust).
/// Checks if window should be scrolled up one line. Returns new mincol if scroll
/// was adjusted (side effect: may call set_topline()), or -1 if no change.
int nvim_edit_insert_check_scroll(int mincol, linenr_T old_topline, int old_topfill,
                                  int did_backspace, int count)
{
  if (!curbuf->b_mod_set
      || !curwin->w_p_wrap
      || curwin->w_p_sms
      || did_backspace
      || curwin->w_topline != old_topline
      || curwin->w_topfill != old_topfill
      || count > 1) {
    return -1;
  }

  int new_mincol = curwin->w_wcol;
  validate_cursor_col(curwin);

  if (curwin->w_wcol < new_mincol - tabstop_at(get_nolist_virtcol(),
                                                curbuf->b_p_ts,
                                                curbuf->b_p_vts_array,
                                                false)
      && curwin->w_wrow == curwin->w_view_height - 1 - rs_get_scrolloff_value(curwin)
      && (curwin->w_cursor.lnum != curwin->w_topline
          || curwin->w_topfill > 0)) {
    if (curwin->w_topfill > 0) {
      curwin->w_topfill--;
    } else if (hasFolding(curwin, curwin->w_topline, NULL, &old_topline)) {
      set_topline(curwin, old_topline + 1);
    } else {
      set_topline(curwin, curwin->w_topline + 1);
    }
  }
  return new_mincol;
}

/// Set did_cursorhold (accessor for Rust state_machine, to avoid duplicate).
void nvim_edit_set_did_cursorhold_rs(int val)
{
  did_cursorhold = val != 0;
}

/// Call ins_redraw(false) (accessor for Rust state_machine).
void ins_redraw_false(void)
{
  nvim_edit_ins_redraw_impl(false);
}

/// Call plain_vgetc() with no_mapping++/-- around it (for CTRL-\ handling in Rust).
int nvim_edit_plain_vgetc_no_mapping(void)
{
  no_mapping++;
  allow_keys++;
  int c = plain_vgetc();
  no_mapping--;
  allow_keys--;
  return c;
}

/// Call ins_ctrl_v() (wrapper for Rust state_machine to avoid name clash).
void ins_ctrl_v_fn(void)
{
  ins_ctrl_v();
}

/// Composite: prepare prompt buffer for insert mode (for Rust redraw.rs).
/// Ensures the last line has prompt text and positions the cursor.
void nvim_edit_init_prompt_impl(int cmdchar_todo)
{
  char *prompt = prompt_text();

  if (curwin->w_cursor.lnum < curbuf->b_prompt_start.mark.lnum) {
    curwin->w_cursor.lnum = curbuf->b_prompt_start.mark.lnum;
  }
  char *text = get_cursor_line_ptr();
  if ((curbuf->b_prompt_start.mark.lnum == curwin->w_cursor.lnum
       && strncmp(text, prompt, strlen(prompt)) != 0)
      || curbuf->b_prompt_start.mark.lnum > curwin->w_cursor.lnum) {
    if (*text == NUL) {
      ml_replace(curbuf->b_ml.ml_line_count, prompt, true);
    } else {
      ml_append(curbuf->b_ml.ml_line_count, prompt, 0, false);
      curbuf->b_prompt_start.mark.lnum += 1;
    }
    curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
    coladvance(curwin, MAXCOL);
    inserted_bytes(curbuf->b_ml.ml_line_count, 0, 0, (colnr_T)strlen(prompt));
  }
  if (Insstart_orig.lnum != curbuf->b_prompt_start.mark.lnum
      || Insstart_orig.col != (colnr_T)strlen(prompt)) {
    Insstart.lnum = curbuf->b_prompt_start.mark.lnum;
    Insstart.col = (colnr_T)strlen(prompt);
    Insstart_orig = Insstart;
    Insstart_textlen = Insstart.col;
    Insstart_blank_vcol = MAXCOL;
    arrow_used = false;
  }
  if (cmdchar_todo == 'A') {
    coladvance(curwin, MAXCOL);
  }
  if (curbuf->b_prompt_start.mark.lnum == curwin->w_cursor.lnum) {
    curwin->w_cursor.col = MAX(curwin->w_cursor.col, (colnr_T)strlen(prompt));
  }
  check_cursor(curwin);
}


/// nvim_edit_iswhite_nl_or_nul: returns 1 if c is whitespace, newline, or NUL (accessor for Rust).
int nvim_edit_iswhite_nl_or_nul(int c)
{
  return (ascii_iswhite(c) || c == NL || c == CAR || c == NUL) ? 1 : 0;
}



/// edit(): Start inserting text.
///
/// "cmdchar" can be:
/// 'i' normal insert command
/// 'a' normal append command
/// 'R' replace command
/// 'r' "r<CR>" command: insert one <CR>.
///     Note: count can be > 1, for redo, but still only one <CR> is inserted.
///           <Esc> is not used for redo.
/// 'g' "gI" command.
/// 'V' "gR" command for Virtual Replace mode.
/// 'v' "gr" command for single character Virtual Replace mode.
///
/// This function is not called recursively.  For CTRL-O commands, it returns
/// and lets the caller handle the Normal-mode command.
///
/// @param  cmdchar  command that started the insert
/// @param  startln  if true, insert at start of line
/// @param  count    repeat count for the command
///
/// @return true if a CTRL-O command caused the return (insert mode pending).
// edit: now implemented in Rust (src/nvim-rs/edit/src/enter.rs, export_name = "edit").
// The full body is provided by nvim_edit_edit_entry below.

/// Composite entry point for `edit()` called by the Rust implementation.
/// Returns the same bool as `edit()`: true iff a CTRL-O caused the return.
bool nvim_edit_edit_entry(int cmdchar, bool startln, int count)
{
  if (curbuf->terminal) {
    if (ex_normal_busy) {
      // Do not enter terminal mode from ex_normal(), which would cause havoc
      // (such as terminal-mode recursiveness). Instead set a flag to force-set
      // the value of `restart_edit` before `ex_normal` returns.
      restart_edit = 'i';
      force_restart_edit = true;
      return false;
    }
    return terminal_enter();
  }

  // Don't allow inserting in the sandbox.
  if (sandbox != 0) {
    emsg(_(e_sandbox));
    return false;
  }

  // Don't allow changes in the buffer while editing the cmdline.  The
  // caller of getcmdline() may get confused.
  // Don't allow recursive insert mode when busy with completion.
  // Allow in dummy buffers since they are only used internally
  if (textlock != 0 || rs_ins_compl_active() || compl_busy || pum_visible()
      || expr_map_locked()) {
    emsg(_(e_textlock));
    return false;
  }

  InsertState s[1];
  memset(s, 0, sizeof(InsertState));
  s->state.execute = insert_execute_rs;
  s->state.check = insert_check_rs;
  s->cmdchar = cmdchar;
  s->startln = startln;
  s->count = count;
  insert_enter(s);
  return s->c == Ctrl_O;
}


/// Composite implementation of ins_redraw() for the Rust port.
void nvim_edit_ins_redraw_impl(int ready)
{
  if (char_avail()) {
    return;
  }

  // Trigger CursorMoved if the cursor moved.  Not when the popup menu is
  // visible, the command might delete it.
  if (ready && has_event(EVENT_CURSORMOVEDI)
      && (last_cursormoved_win != curwin
          || !equalpos(last_cursormoved, curwin->w_cursor))
      && !pum_visible()) {
    // Need to update the screen first, to make sure syntax
    // highlighting is correct after making a change (e.g., inserting
    // a "(".  The autocommand may also require a redraw, so it's done
    // again below, unfortunately.
    if (syntax_present(curwin) && must_redraw) {
      update_screen();
    }
    // Make sure curswant is correct, an autocommand may call
    // getcurpos()
    update_curswant();
    ins_apply_autocmds(EVENT_CURSORMOVEDI);
    last_cursormoved_win = curwin;
    last_cursormoved = curwin->w_cursor;
  }

  // Trigger TextChangedI if changedtick_i differs.
  if (ready && has_event(EVENT_TEXTCHANGEDI)
      && curbuf->b_last_changedtick_i != buf_get_changedtick(curbuf)
      && !pum_visible()) {
    aco_save_T aco;
    varnumber_T tick = buf_get_changedtick(curbuf);

    // save and restore curwin and curbuf, in case the autocmd changes them
    aucmd_prepbuf(&aco, curbuf);
    apply_autocmds(EVENT_TEXTCHANGEDI, NULL, NULL, false, curbuf);
    aucmd_restbuf(&aco);
    curbuf->b_last_changedtick_i = buf_get_changedtick(curbuf);
    if (tick != buf_get_changedtick(curbuf)) {  // see ins_apply_autocmds()
      u_save(curwin->w_cursor.lnum,
             (linenr_T)(curwin->w_cursor.lnum + 1));
    }
  }

  // Trigger TextChangedP if changedtick_pum differs. When the popupmenu
  // closes TextChangedI will need to trigger for backwards compatibility,
  // thus use different b_last_changedtick* variables.
  if (ready && has_event(EVENT_TEXTCHANGEDP)
      && curbuf->b_last_changedtick_pum != buf_get_changedtick(curbuf)
      && pum_visible()) {
    aco_save_T aco;
    varnumber_T tick = buf_get_changedtick(curbuf);

    // save and restore curwin and curbuf, in case the autocmd changes them
    aucmd_prepbuf(&aco, curbuf);
    apply_autocmds(EVENT_TEXTCHANGEDP, NULL, NULL, false, curbuf);
    aucmd_restbuf(&aco);
    curbuf->b_last_changedtick_pum = buf_get_changedtick(curbuf);
    if (tick != buf_get_changedtick(curbuf)) {  // see ins_apply_autocmds()
      u_save(curwin->w_cursor.lnum,
             (linenr_T)(curwin->w_cursor.lnum + 1));
    }
  }

  if (ready) {
    may_trigger_win_scrolled_resized();
  }

  // Trigger BufModified if b_changed_invalid is set.
  if (ready && has_event(EVENT_BUFMODIFIEDSET)
      && curbuf->b_changed_invalid == true
      && !pum_visible()) {
    apply_autocmds(EVENT_BUFMODIFIEDSET, NULL, NULL, false, curbuf);
    curbuf->b_changed_invalid = false;
  }

  // Trigger SafeState if nothing is pending.
  may_trigger_safestate(ready
                        && !rs_ins_compl_active()
                        && !pum_visible());

  pum_check_clear();
  show_cursor_info_later(false);
  if (must_redraw) {
    update_screen();
  } else {
    redraw_statuslines();
    if (clear_cmdline || redraw_cmdline || redraw_mode) {
      showmode();  // clear cmdline and show mode
    }
  }
  setcursor();
  emsg_on_display = false;      // may remove error message now
}


// Put a character directly onto the screen.  It's not stored in a buffer.
// Used while handling CTRL-K, CTRL-V, etc. in Insert mode.
static int pc_status;
#define PC_STATUS_UNSET 0  // nothing was put on screen
#define PC_STATUS_RIGHT 1  // right half of double-wide char
#define PC_STATUS_LEFT  2  // left half of double-wide char
#define PC_STATUS_SET   3  // pc_schar was filled
static schar_T pc_schar;   // saved char
static int pc_attr;
static int pc_row;
static int pc_col;

void edit_putchar(int c, bool highlight)
{
  if (curwin->w_grid_alloc.chars == NULL && default_grid.chars == NULL) {
    return;
  }

  int attr;
  update_topline(curwin);  // just in case w_topline isn't valid
  validate_cursor(curwin);
  if (highlight) {
    attr = HL_ATTR(HLF_8);
  } else {
    attr = 0;
  }
  pc_row = curwin->w_wrow;
  pc_status = PC_STATUS_UNSET;
  grid_line_start(&curwin->w_grid, pc_row);
  if (curwin->w_p_rl) {
    pc_col = curwin->w_view_width - 1 - curwin->w_wcol;

    if (grid_line_getchar(pc_col, NULL) == NUL) {
      grid_line_put_schar(pc_col - 1, schar_from_ascii(' '), attr);
      curwin->w_wcol--;
      pc_status = PC_STATUS_RIGHT;
    }
  } else {
    pc_col = curwin->w_wcol;

    if (grid_line_getchar(pc_col + 1, NULL) == NUL) {
      // pc_col is the left half of a double-width char
      pc_status = PC_STATUS_LEFT;
    }
  }

  // save the character to be able to put it back
  if (pc_status == PC_STATUS_UNSET) {
    pc_schar = grid_line_getchar(pc_col, &pc_attr);
    pc_status = PC_STATUS_SET;
  }

  char buf[MB_MAXCHAR + 1];
  grid_line_puts(pc_col, buf, utf_char2bytes(c, buf), attr);
  grid_line_flush();
}



// Undo the previous edit_putchar().
void edit_unputchar(void)
{
  if (pc_status != PC_STATUS_UNSET) {
    if (pc_status == PC_STATUS_RIGHT) {
      curwin->w_wcol++;
    }
    if (pc_status == PC_STATUS_RIGHT || pc_status == PC_STATUS_LEFT) {
      redrawWinline(curwin, curwin->w_cursor.lnum);
    } else {
      // TODO(bfredl): this could be smarter and also handle the dubyawidth case
      grid_line_start(&curwin->w_grid, pc_row);
      grid_line_put_schar(pc_col, pc_schar, pc_attr);
      grid_line_flush();
    }
  }
}

/// Called when "$" is in 'cpoptions': display a '$' at the end of the changed
/// text.  Only works when cursor is in the line that changes.
void display_dollar(colnr_T col_arg)
{
  colnr_T col = MAX(col_arg, 0);

  if (!redrawing()) {
    return;
  }

  colnr_T save_col = curwin->w_cursor.col;
  curwin->w_cursor.col = col;

  // If on the last byte of a multi-byte move to the first byte.
  char *p = get_cursor_line_ptr();
  curwin->w_cursor.col -= utf_head_off(p, p + col);
  curs_columns(curwin, false);              // Recompute w_wrow and w_wcol
  if (curwin->w_wcol < curwin->w_view_width) {
    edit_putchar('$', false);
    dollar_vcol = curwin->w_virtcol;
  }
  curwin->w_cursor.col = save_col;
}


String get_last_insert(void)
  FUNC_ATTR_PURE
{
  RsNvimString rs = rs_get_last_insert();
  return rs.data == NULL ? NULL_STRING : (String){ .data = rs.data, .size = rs.size };
}


/// Check the word in front of the cursor for an abbreviation.
/// Called when the non-id character "c" has been entered.
// replace-stack functions
//
// When replacing characters, the replaced characters are remembered for each
// new character.  This is used to re-insert the old text when backspacing.
//
// There is a NUL headed list of characters for each character that is
// currently in the file after the insertion point.  When BS is used, one NUL
// headed list is put back for the deleted character.
//
// For a newline, there are two NUL headed lists.  One contains the characters
// that the NL replaced.  The extra one stores the characters after the cursor
// that were deleted (always white space).

/// Push character that is replaced onto the replace stack.
///
/// replace_offset is normally 0, in which case replace_push will add a new
/// character at the end of the stack.  If replace_offset is not 0, that many
/// characters will be left on the stack above the newly inserted character.
// get_can_cindent: now exported directly from Rust (export_name = "get_can_cindent").

void set_can_cindent(bool val)
{
  can_cindent = val;
}

/// Trigger "event" and take care of fixing undo.
int ins_apply_autocmds(event_T event)
{
  varnumber_T tick = buf_get_changedtick(curbuf);

  int r = apply_autocmds(event, NULL, NULL, false, curbuf);

  // If u_savesub() was called then we are not prepared to start
  // a new line.  Call u_save() with no contents to fix that.
  // Except when leaving Insert mode.
  if (event != EVENT_INSERTLEAVE && tick != buf_get_changedtick(curbuf)) {
    u_save(curwin->w_cursor.lnum, (linenr_T)(curwin->w_cursor.lnum + 1));
  }

  return r;
}

// =============================================================================
// C Wrappers for Rust FFI
// =============================================================================

// nvim_scroll_cursor_up, nvim_scroll_cursor_down: deleted (move crate now calls cursor_up/cursor_down directly).

/// Get the dollar_vcol global variable (accessor for Rust).
colnr_T nvim_get_dollar_vcol(void)
{
  return dollar_vcol;
}

/// Set the dollar_vcol global variable (accessor for Rust).
void nvim_set_dollar_vcol(colnr_T val)
{
  dollar_vcol = val;
}

// =============================================================================
// Phase 1: ins_tab accessors
// (did_ai, did_si, can_si, can_si_back are in change_ffi.c)
// =============================================================================

// nvim_get_p_sta is in option_shim.c
// nvim_curbuf_get_b_p_et is in option_shim.c

/// Get curbuf->b_p_sts (accessor for Rust).
long nvim_curbuf_get_b_p_sts(void)
{
  return (long)curbuf->b_p_sts;
}

/// Get tabstop_count(curbuf->b_p_vts_array) (accessor for Rust).
int nvim_curbuf_tabstop_count_vts(void)
{
  return (int)tabstop_count(curbuf->b_p_vts_array);
}

/// Get tabstop_count(curbuf->b_p_vsts_array) (accessor for Rust).
int nvim_curbuf_tabstop_count_vsts(void)
{
  return (int)tabstop_count(curbuf->b_p_vsts_array);
}

/// Get tabstop_first(curbuf->b_p_vts_array) (accessor for Rust).
long nvim_curbuf_tabstop_first_vts(void)
{
  return (long)tabstop_first(curbuf->b_p_vts_array);
}

/// Get get_sw_value(curbuf) (accessor for Rust).
long nvim_curbuf_get_sw_value(void)
{
  return get_sw_value(curbuf);
}

/// Get get_sts_value() (accessor for Rust).
long nvim_get_sts_value(void)
{
  return get_sts_value();
}

/// Get tabstop_padding for softtabstop (accessor for Rust).
int nvim_curbuf_tabstop_padding_sts(void)
{
  return tabstop_padding(get_nolist_virtcol(), get_sts_value(), curbuf->b_p_vsts_array);
}

/// Get tabstop_padding for tabstop (accessor for Rust).
int nvim_curbuf_tabstop_padding_ts(void)
{
  return tabstop_padding(get_nolist_virtcol(), curbuf->b_p_ts, curbuf->b_p_vts_array);
}

/// Replace spaces with TABs in current line (helper for Rust ins_tab).
///
/// This handles the complex memory manipulation part of ins_tab:
/// the space-to-TAB replacement optimization when 'expandtab' is off.
/// Returns false when done.
bool nvim_edit_ins_tab_replace_spaces(bool p_sta_val, bool ind)
{
  char *ptr;
  char *saved_line = NULL;
  pos_T pos;
  pos_T *cursor;
  colnr_T want_vcol, vcol;
  int change_col = -1;
  int temp = 0;
  int save_list = curwin->w_p_list;

  if (State & VREPLACE_FLAG) {
    pos = curwin->w_cursor;
    cursor = &pos;
    saved_line = xstrnsave(get_cursor_line_ptr(), (size_t)get_cursor_line_len());
    ptr = saved_line + pos.col;
  } else {
    ptr = get_cursor_pos_ptr();
    cursor = &curwin->w_cursor;
  }

  if (vim_strchr(p_cpo, CPO_LISTWM) == NULL) {
    curwin->w_p_list = false;
  }

  pos_T fpos = curwin->w_cursor;
  while (fpos.col > 0 && ascii_iswhite(ptr[-1])) {
    fpos.col--;
    ptr--;
  }

  if ((State & REPLACE_FLAG)
      && fpos.lnum == Insstart.lnum
      && fpos.col < Insstart.col) {
    ptr += Insstart.col - fpos.col;
    fpos.col = Insstart.col;
  }

  getvcol(curwin, &fpos, &vcol, NULL, NULL);
  getvcol(curwin, cursor, &want_vcol, NULL, NULL);

  char *tab = "\t";
  int32_t tab_v = (uint8_t)(*tab);

  CharsizeArg csarg;
  CSType cstype = init_charsize_arg(&csarg, curwin, 0, tab);

  while (ascii_iswhite(*ptr)) {
    int i = win_charsize(cstype, vcol, tab, tab_v, &csarg).width;
    if (vcol + i > want_vcol) {
      break;
    }
    if (*ptr != TAB) {
      *ptr = TAB;
      if (change_col < 0) {
        change_col = fpos.col;
        if (fpos.lnum == Insstart.lnum && fpos.col < Insstart.col) {
          Insstart.col = fpos.col;
        }
      }
    }
    fpos.col++;
    ptr++;
    vcol += i;
  }

  if (change_col >= 0) {
    int repl_off = 0;
    cstype = init_charsize_arg(&csarg, curwin, 0, ptr);
    while (vcol < want_vcol && *ptr == ' ') {
      vcol += win_charsize(cstype, vcol, ptr, (uint8_t)(' '), &csarg).width;
      ptr++;
      repl_off++;
    }

    if (vcol > want_vcol) {
      ptr--;
      repl_off--;
    }
    fpos.col += repl_off;

    int i = cursor->col - fpos.col;
    if (i > 0) {
      if (!(State & VREPLACE_FLAG)) {
        char *newp = xmalloc((size_t)(curbuf->b_ml.ml_line_len - i));
        ptrdiff_t col = ptr - curbuf->b_ml.ml_line_ptr;
        if (col > 0) {
          memmove(newp, ptr - col, (size_t)col);
        }
        memmove(newp + col, ptr + i, (size_t)(curbuf->b_ml.ml_line_len - col - i));
        if (curbuf->b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED)) {
          xfree(curbuf->b_ml.ml_line_ptr);
        }
        curbuf->b_ml.ml_line_ptr = newp;
        curbuf->b_ml.ml_line_len -= i;
        curbuf->b_ml.ml_flags = (curbuf->b_ml.ml_flags | ML_LINE_DIRTY) & ~ML_EMPTY;
        inserted_bytes(fpos.lnum, change_col,
                       cursor->col - change_col, fpos.col - change_col);
      } else {
        STRMOVE(ptr, ptr + i);
      }
      if ((State & REPLACE_FLAG) && !(State & VREPLACE_FLAG)) {
        for (temp = i; --temp >= 0;) {
          replace_join(repl_off);
        }
      }
    }
    cursor->col -= i;

    if (State & VREPLACE_FLAG) {
      backspace_until_column(change_col);
      ins_bytes_len(saved_line + change_col, (size_t)(cursor->col - change_col));
    }
  }

  if (State & VREPLACE_FLAG) {
    xfree(saved_line);
  }
  curwin->w_p_list = save_list;
  return false;
}

// Static asserts for Phase 1 (ins_tab)
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL mismatch");
_Static_assert(ABBR_OFF == 0x100, "ABBR_OFF mismatch (Phase1)");

// =============================================================================
// Phase 2: ins_bs accessors and helpers
// =============================================================================

// Static asserts for Phase 2 constants
_Static_assert(BACKSPACE_CHAR == 1, "BACKSPACE_CHAR mismatch");
_Static_assert(BACKSPACE_WORD == 2, "BACKSPACE_WORD mismatch");
_Static_assert(BACKSPACE_WORD_NOT_SPACE == 3, "BACKSPACE_WORD_NOT_SPACE mismatch");
_Static_assert(BACKSPACE_LINE == 4, "BACKSPACE_LINE mismatch");
_Static_assert(kOptBoFlagBackspace == 0x02, "kOptBoFlagBackspace mismatch (Phase2)");
_Static_assert(FO_AUTO == 'a', "FO_AUTO mismatch");
_Static_assert(FO_WHITE_PAR == 'w', "FO_WHITE_PAR mismatch");

/// Check if backspace mode is allowed (accessor for Rust).
int nvim_edit_can_bs(int what)
{
  return can_bs((char)what) ? 1 : 0;
}

/// Increment cursor position (accessor for Rust).
void nvim_edit_inc_cursor(void)
{
  inc_cursor();
}

/// Decrement cursor position (accessor for Rust).
void nvim_edit_dec_cursor(void)
{
  dec_cursor();
}

// nvim_edit_get_cursor_coladd and nvim_edit_set_cursor_coladd are defined
// in the Phase 3 accessors section (lines ~701-710).

/// Call u_save(lnum1, lnum2) (accessor for Rust).
int nvim_edit_u_save(int lnum1, int lnum2)
{
  return u_save((linenr_T)lnum1, (linenr_T)lnum2);
}

/// Get ml_get_len(lnum) (accessor for Rust).
int nvim_edit_ml_get_len(int lnum)
{
  return ml_get_len((linenr_T)lnum);
}

/// Get has_format_option(c) (accessor for Rust).
int nvim_edit_has_format_option(int c)
{
  return has_format_option((char)c) ? 1 : 0;
}

/// Trim last char of previous line if space (FO_WHITE_PAR helper).
void nvim_edit_trim_eol_space(void)
{
  char *ptr = ml_get_buf_mut(curbuf, curwin->w_cursor.lnum);
  int len = get_cursor_line_len();
  if (len > 0 && ptr[len - 1] == ' ') {
    ptr[len - 1] = NUL;
    curbuf->b_ml.ml_line_len--;
  }
}

/// Call do_join(2, false, false, false, false) (accessor for Rust).
void nvim_edit_do_join_simple(void)
{
  do_join(2, false, false, false, false);
}

/// Get mb_get_class(get_cursor_pos_ptr()) (accessor for Rust).
int nvim_edit_mb_get_class_cursor(void)
{
  return mb_get_class(get_cursor_pos_ptr());
}

/// Get vim_iswordc(c) (accessor for Rust).
int nvim_edit_vim_iswordc(int c)
{
  return vim_iswordc((unsigned)c) ? 1 : 0;
}

/// Check utf_composinglike at cursor (accessor for Rust).
int nvim_edit_cursor_has_composing(void)
{
  if (!p_deco) {
    return 0;
  }
  char *p0 = get_cursor_pos_ptr();
  return utf_composinglike(p0, p0 + utf_ptr2len(p0), NULL) ? 1 : 0;
}

/// Call fix_indent() (accessor for Rust).
void nvim_edit_fix_indent(void)
{
  fix_indent();
}

/// Check if p_cpo contains CPO_BACKSPACE (accessor for Rust).
int nvim_edit_p_cpo_has_backspace(void)
{
  return vim_strchr(p_cpo, CPO_BACKSPACE) != NULL ? 1 : 0;
}

/// Get cindent_on() (accessor for Rust).
int nvim_edit_cindent_on(void)
{
  return cindent_on() ? 1 : 0;
}

/// Handle softtabstop-aware backspace alignment (helper for Rust ins_bs).
///
/// Handles the BACKSPACE_CHAR case when softtabstop or smarttab is active:
/// finds the target column and deletes/inserts spaces to align properly.
/// Returns true if the softtabstop case was handled.
bool nvim_edit_ins_bs_softtabstop(int *inserted_space_p, bool in_indent)
{
  bool const use_ts = !curwin->w_p_list || curwin->w_p_lcs_chars.tab1;
  char *const line = get_cursor_line_ptr();
  char *const cursor_ptr = line + curwin->w_cursor.col;

  colnr_T vcol = 0;
  colnr_T space_vcol = 0;
  StrCharInfo sci = utf_ptr2StrCharInfo(line);
  StrCharInfo space_sci = sci;
  bool prev_space = false;

  // Compute virtual column of cursor position.
  while (sci.ptr < cursor_ptr) {
    bool cur_space = ascii_iswhite(sci.chr.value);
    if (!prev_space && cur_space) {
      space_sci = sci;
      space_vcol = vcol;
    }
    vcol += charsize_nowrap(curbuf, sci.ptr, use_ts, vcol, sci.chr.value);
    sci = utfc_next(sci);
    prev_space = cur_space;
  }

  // Compute the virtual column where we want to be.
  colnr_T want_vcol = vcol > 0 ? vcol - 1 : 0;
  if (p_sta && in_indent) {
    want_vcol -= want_vcol % get_sw_value(curbuf);
  } else {
    want_vcol = tabstop_start(want_vcol, get_sts_value(), curbuf->b_p_vsts_array);
  }

  // Find stop position.
  while (true) {
    int size = charsize_nowrap(curbuf, space_sci.ptr, use_ts, space_vcol, space_sci.chr.value);
    if (space_vcol + size > want_vcol) {
      break;
    }
    space_vcol += size;
    space_sci = utfc_next(space_sci);
  }
  colnr_T const want_col = (int)(space_sci.ptr - line);

  // Delete characters until we are at or before want_col.
  while (curwin->w_cursor.col > want_col) {
    dec_cursor();
    if (State & REPLACE_FLAG) {
      if (curwin->w_cursor.lnum != Insstart.lnum
          || curwin->w_cursor.col >= Insstart.col) {
        replace_do_bs(-1);
      }
    } else {
      del_char(false);
    }
  }

  // Insert extra spaces until we are at want_vcol.
  for (; space_vcol < want_vcol; space_vcol++) {
    if (curwin->w_cursor.lnum == Insstart_orig.lnum
        && curwin->w_cursor.col < Insstart_orig.col) {
      Insstart_orig.col = curwin->w_cursor.col;
    }
    if (State & VREPLACE_FLAG) {
      ins_char(' ');
    } else {
      ins_str(S_LEN(" "));
      if (State & REPLACE_FLAG) {
        replace_push_nul();
      }
    }
  }
  return true;
}

/// Check if softtabstop-aware backspace should be used (accessor for Rust).
/// Returns true when the BACKSPACE_CHAR softtabstop path is taken.
bool nvim_edit_ins_bs_check_sts(int *inserted_space_p, bool in_indent)
{
  if (curwin->w_cursor.col == 0) {
    return false;
  }
  return (p_sta && in_indent)
    || ((get_sts_value() != 0 || tabstop_count(curbuf->b_p_vsts_array))
        && curwin->w_cursor.col > 0
        && (*(get_cursor_pos_ptr() - 1) == TAB
            || (*(get_cursor_pos_ptr() - 1) == ' '
                && (!*inserted_space_p || arrow_used))));
}

// =============================================================================
// Phase 2 accessors: stop_insert, ins_esc, ins_reg
// =============================================================================

/// Clear VALID_WCOL and VALID_VIRTCOL bits from curwin->w_valid (accessor for Rust).
void nvim_edit_curwin_clear_wcol_virtcol(void)
{
  curwin->w_valid &= ~(VALID_WCOL | VALID_VIRTCOL);
}

/// Clear VALID_WROW, VALID_WCOL, VALID_VIRTCOL bits from curwin->w_valid
/// (for gchar_cursor() == TAB or inline virtual text -- accessor for Rust).
void nvim_edit_curwin_clear_wrow_wcol_virtcol(void)
{
  curwin->w_valid &= ~(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
}

/// Call ins_apply_autocmds(EVENT_INSERTLEAVEPRE) (accessor for Rust).
void nvim_edit_ins_apply_autocmds_insertleavepre(void)
{
  ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
}

/// Call unshowmode(false) (accessor for Rust).
void nvim_edit_unshowmode_false(void)
{
  unshowmode(false);
}

/// Call skip_showmode() (accessor for Rust).
int nvim_edit_skip_showmode(void)
{
  return skip_showmode() ? 1 : 0;
}

/// Set curbuf->b_last_insert mark to current cursor + topline (composite for Rust).
/// Calls mark_view_make and RESET_FMARK.
void nvim_edit_set_b_last_insert_mark(void)
{
  fmarkv_T view = mark_view_make(curwin->w_topline, curwin->w_cursor);
  RESET_FMARK(&curbuf->b_last_insert, curwin->w_cursor, curbuf->b_fnum, view);
}

/// Get buf_meta_total(curbuf, kMTMetaInline) (accessor for Rust).
int nvim_edit_curbuf_meta_total_inline(void)
{
  return buf_meta_total(curbuf, kMTMetaInline);
}

/// Get u_sync_once global (accessor for Rust).
int nvim_get_u_sync_once(void)
{
  return u_sync_once;
}

/// Set u_sync_once global (accessor for Rust).
void nvim_set_u_sync_once(int val)
{
  u_sync_once = val;
}

/// Get yankreg_T* for register regname (YREG_PASTE), as void* (accessor for Rust).
void *nvim_edit_get_yank_register(int regname)
{
  return get_yank_register(regname, YREG_PASTE);
}

/// Call insert_reg(regname, NULL, literally != 0) (accessor for Rust).
int nvim_edit_insert_reg(int regname, int literally)
{
  return insert_reg(regname, NULL, literally != 0);
}

/// Call is_literal_register(regname) (accessor for Rust).
int nvim_edit_is_literal_register(int regname)
{
  return is_literal_register(regname) ? 1 : 0;
}

/// Get reg->y_size for a yankreg_T* (accessor for Rust).
size_t nvim_edit_reg_y_size(void *reg)
{
  return ((yankreg_T *)reg)->y_size;
}

/// Call end_visual_mode() (accessor for Rust).
void nvim_edit_end_visual_mode(void)
{
  end_visual_mode();
}

/// Set pc_status = PC_STATUS_UNSET (accessor for Rust).
void nvim_edit_set_pc_status_unset(void)
{
  pc_status = PC_STATUS_UNSET;
}

/// Call edit_putchar(c, highlight != 0) (accessor for Rust).
void nvim_edit_putchar(int c, int highlight)
{
  edit_putchar(c, highlight != 0);
}

/// Call edit_unputchar() (accessor for Rust).
void nvim_edit_edit_unputchar(void)
{
  edit_unputchar();
}

// ---- ins_esc accessors ----

/// Decrement RedrawingDisabled (for ins_esc undo of repeat-path increment).
void nvim_edit_dec_redrawing_disabled(void)
{
  RedrawingDisabled--;
}

/// Increment RedrawingDisabled (for ins_esc repeat path).
void nvim_edit_inc_RedrawingDisabled(void)
{
  RedrawingDisabled++;
}

/// Check if p_cpo contains CPO_REPLCNT (accessor for Rust).
int nvim_edit_p_cpo_has_replcnt(void)
{
  return vim_strchr(p_cpo, CPO_REPLCNT) != NULL ? 1 : 0;
}

/// Get get_ve_flags(curwin) (accessor for Rust).
int nvim_edit_get_ve_flags_curwin(void)
{
  return (int)get_ve_flags(curwin);
}

/// Check if (cmdmod.cmod_flags & CMOD_KEEPJUMPS) != 0 (accessor for Rust).
int nvim_edit_cmod_keepjumps(void)
{
  return (cmdmod.cmod_flags & CMOD_KEEPJUMPS) != 0 ? 1 : 0;
}

/// Call stop_insert logic at curwin->w_cursor (composite for Rust).
void nvim_edit_stop_insert_curpos(int nomove)
{
  pos_T *pos = &curwin->w_cursor;
  stop_redo_ins();
  rs_replace_stack_clear();

  // Save inserted text for redo (^@ / CTRL-A).
  String inserted = get_inserted();
  int added = inserted.data == NULL ? 0 : (int)inserted.size - new_insert_skip;
  if (did_restart_edit == 0 || added > 0) {
    xfree(last_insert.data);
    last_insert = inserted;
    last_insert_skip = added < 0 ? 0 : new_insert_skip;
  } else {
    xfree(inserted.data);
  }

  if (!arrow_used) {
    // Auto-format + strip trailing auto-indent whitespace.
    int cc;
    if (!ins_need_undo && has_format_option(FO_AUTO)) {
      pos_T tpos = curwin->w_cursor;
      cc = 'x';
      if (curwin->w_cursor.col > 0 && gchar_cursor() == NUL) {
        dec_cursor();
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          curwin->w_cursor = tpos;
        }
      }
      auto_format(true, false);
      if (ascii_iswhite(cc)) {
        if (gchar_cursor() != NUL) {
          inc_cursor();
        }
        if (gchar_cursor() == NUL
            && curwin->w_cursor.lnum == tpos.lnum
            && curwin->w_cursor.col == tpos.col) {
          curwin->w_cursor.coladd = tpos.coladd;
        }
      }
    }
    check_auto_format(true);
    // esc=true (called from ins_esc path), so the esc||... condition is always true.
    if (!nomove && did_ai && pos->lnum <= curbuf->b_ml.ml_line_count) {
      pos_T tpos = curwin->w_cursor;
      colnr_T prev_col = pos->col;
      curwin->w_cursor = *pos;
      check_cursor_col(curwin);
      while (true) {
        if (gchar_cursor() == NUL && curwin->w_cursor.col > 0) {
          curwin->w_cursor.col--;
        }
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          break;
        }
        if (del_char(true) == FAIL) {
          break;
        }
      }
      if (curwin->w_cursor.lnum != tpos.lnum) {
        curwin->w_cursor = tpos;
      } else if (curwin->w_cursor.col < prev_col) {
        tpos = curwin->w_cursor;
        tpos.col++;
        if (cc != NUL && gchar_pos(&tpos) == NUL) {
          curwin->w_cursor.col++;
        }
      }
      if (VIsual_active) {
        check_visual_pos();
      }
    }
  }

  did_ai = false;
  did_si = false;
  can_si = false;
  can_si_back = false;
  curbuf->b_op_start = Insstart;
  curbuf->b_op_start_orig = Insstart_orig;
  curbuf->b_op_end = *pos;
}

/// Get p_ch == 0 && !ui_has(kUIMessages) (accessor for Rust ins_esc showmode check).
int nvim_edit_get_p_ch_zero_no_ui_messages(void)
{
  return (p_ch == 0 && !ui_has(kUIMessages)) ? 1 : 0;
}

/// Get curwin->w_cursor.coladd (accessor for Rust).
colnr_T nvim_curwin_get_cursor_coladd(void)
{
  return curwin->w_cursor.coladd;
}

// ---- ins_reg accessors ----

/// Save cursor position for expression register evaluation (composite for Rust).
static pos_T ins_reg_saved_cursor;
void nvim_edit_ins_reg_restore_cursor_save(void)
{
  ins_reg_saved_cursor = curwin->w_cursor;
}

/// Restore cursor position after expression register evaluation (composite for Rust).
void nvim_edit_ins_reg_restore_cursor(void)
{
  curwin->w_cursor = ins_reg_saved_cursor;
  check_cursor(curwin);
}

