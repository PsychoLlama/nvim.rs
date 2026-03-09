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
extern int rs_ctrl_x_mode_normal(void);
extern int rs_ctrl_x_mode_scroll(void);
extern int rs_ctrl_x_mode_whole_line(void);
extern int rs_ctrl_x_mode_files(void);
extern int rs_ctrl_x_mode_tags(void);
extern int rs_ctrl_x_mode_path_patterns(void);
extern int rs_ctrl_x_mode_path_defines(void);
extern int rs_ctrl_x_mode_dictionary(void);
extern int rs_ctrl_x_mode_thesaurus(void);
extern int rs_ctrl_x_mode_cmdline(void);
extern int rs_ctrl_x_mode_function(void);
extern int rs_ctrl_x_mode_omni(void);
extern int rs_ctrl_x_mode_spell(void);
extern int rs_ctrl_x_mode_line_or_eval(void);
extern int rs_ctrl_x_mode_register(void);
extern int rs_ins_compl_active(void);
extern int rs_ins_compl_accept_char(int c);
extern void rs_ins_compl_clear(void);
extern int rs_ins_compl_win_active(win_T *wp);
extern int rs_ins_compl_used_match(void);
extern int rs_ins_compl_enter_selects(void);
extern int rs_ins_compl_col(void);
extern int rs_ins_compl_has_preinsert(void);
extern int rs_ins_compl_preinsert_effect(void);
extern int rs_ins_compl_has_autocomplete(void);
extern int rs_compl_status_local(void);
extern int rs_ins_compl_is_match_selected(void);
extern int rs_ins_compl_preinsert_longest(void);
extern int rs_ins_compl_has_shown_match(void);
extern int rs_ins_compl_long_shown_match(void);
extern int rs_pum_wanted(void);
extern void rs_compl_status_clear(void);
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

#include "edit.c.generated.h"

extern int rs_get_scrolloff_value(win_T *wp);

// Rust fold FFI declarations
extern void rs_foldOpenCursor(void);
extern void rs_foldCheckClose(void);
extern void rs_foldUpdateAfterInsert(void);

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

// Rust FFI declarations
extern int rs_ins_need_undo_get(void);
extern int rs_get_can_cindent(void);
extern char *rs_buf_prompt_text(const buf_T *buf);
extern char *rs_prompt_text(void);
extern bool rs_prompt_curpos_editable(void);
// State module exports
extern int rs_state_ins_need_undo(void);
extern int rs_state_can_cindent(void);
extern void rs_state_set_can_cindent(int val);
extern int rs_state_revins_on(void);
extern int rs_state_did_restart_edit(void);
extern int rs_state_compl_busy(void);
extern linenr_T rs_state_insstart_lnum(void);
extern colnr_T rs_state_insstart_col(void);
extern linenr_T rs_state_insstart_orig_lnum(void);
extern colnr_T rs_state_insstart_orig_col(void);
extern colnr_T rs_state_insstart_textlen(void);
extern colnr_T rs_state_insstart_blank_vcol(void);
extern int rs_state_dont_sync_undo(void);
extern void rs_state_set_dont_sync_undo(int val);
extern linenr_T rs_state_o_lnum(void);
extern void rs_state_set_o_lnum(linenr_T val);
// Mode module exports
extern void rs_init_insert_state(int startln, int cmdchar);
extern int rs_handle_restart_edit(void);
extern int rs_in_insert_mode(void);
extern int rs_in_replace_mode(void);
extern int rs_in_vreplace_mode(void);
extern void rs_set_insert_mode(int cmdchar);
extern void rs_update_o_lnum_on_exit(void);
extern int rs_has_langmap(void);
extern void rs_enable_langmap(void);
extern void rs_disable_langmap(void);
extern int rs_revins_on(void);
extern int rs_did_restart_edit(void);
extern int rs_get_arrow_used(void);
extern void rs_set_arrow_used(int val);
extern linenr_T rs_get_o_lnum(void);
// Insert module exports
extern int rs_char_info_byte_len(int c);
extern int rs_char_needs_nul_conversion(int c);
extern int rs_insert_in_replace_mode(void);
extern int rs_insert_in_vreplace_mode(void);
extern int rs_is_printable(int c);
extern int rs_is_whitespace(int c);
extern int rs_is_newline(int c);
extern colnr_T rs_insert_cursor_col(void);
extern linenr_T rs_insert_cursor_lnum(void);
// Keys module exports (edit_ prefix to avoid cmdline conflicts)
extern int rs_edit_is_arrow_key(int key);
extern int rs_edit_is_navigation_key(int key);
extern int rs_edit_is_delete_key(int key);
extern int rs_edit_is_backspace_key(int key);
extern int rs_edit_is_enter_key(int key);
extern int rs_edit_is_tab_key(int key);
extern int rs_edit_is_ctrl_key(int key);
extern int rs_edit_is_escape_key(int key);
extern int rs_edit_is_printable_ascii(int key);
extern int rs_edit_is_special_key(int key);
extern int rs_edit_get_nav_direction(int key);
extern int rs_edit_classify_key(int key);
// Abbreviation module exports
extern int rs_abbr_disabled(void);
extern int rs_abbr_loaded(void);
extern int rs_add_abbr_off(int c);
extern int rs_remove_abbr_off(int c);
extern int rs_has_abbr_off(int c);
extern int rs_abbr_trigger_type(int c);
extern int rs_abbr_trigger_needs_offset(int trigger_type);
// Helpers module exports
extern void rs_undisplay_dollar(void);
extern colnr_T rs_get_nolist_virtcol(void);
extern int rs_echeck_abbr(int c);
extern void rs_truncate_spaces(char *line, size_t len);
extern int rs_del_char_after_col(int limit_col);
extern void rs_backspace_until_column(int col);
extern void rs_set_last_insert(int c);
extern void rs_free_last_insert(void);

typedef struct {
  char *data;
  size_t size;
} RsNvimString;
extern RsNvimString rs_get_last_insert(void);
extern char *rs_get_last_insert_save(void);
// Replace stack module exports (Phase 2)
extern void rs_replace_push(const char *str, size_t len);
extern void rs_replace_push_nul(void);
extern int rs_replace_pop_if_nul(void);
extern void rs_replace_join(int off);
extern void rs_replace_pop_ins(void);
extern void rs_mb_replace_pop_ins(void);
extern void rs_replace_do_bs(int limit_col);
extern void rs_replace_stack_clear(void);
// Movement module exports (Phase 3)
extern void rs_beginline(int flags);
extern int rs_oneright(void);
extern int rs_oneleft(void);
extern void rs_cursor_up_inner(win_T *wp, linenr_T n, bool skip_conceal);
extern int rs_cursor_up(linenr_T n, int upd_topline);
extern void rs_cursor_down_inner(win_T *wp, int n, bool skip_conceal);
extern int rs_cursor_down(int n, int upd_topline);
// Key handler module exports (Phase 4)
extern void rs_ins_left(void);
extern void rs_ins_right(void);
extern void rs_ins_s_left(void);
extern void rs_ins_s_right(void);
extern void rs_ins_home(int c);
extern void rs_ins_end(int c);
extern void rs_ins_up(int startcol);
extern void rs_ins_down(int startcol);
extern void rs_ins_pageup(void);
extern void rs_ins_pagedown(void);
extern void rs_ins_insert(int replaceState);
extern void rs_ins_ctrl_o(void);
extern void rs_ins_ctrl_hat(void);
extern void rs_ins_ctrl_(void);
extern int rs_ins_start_select(int c);
extern void rs_ins_ctrl_g(void);
extern void rs_ins_shift(int c, int lastc);
extern void rs_ins_del(void);
// Tab and EOL module exports (Phase 1)
// ins_tab and ins_eol are now exported directly by Rust and declared in edit.h
extern void rs_ins_ctrl_v(void);
extern int rs_ins_copychar(linenr_T lnum);
extern int rs_ins_ctrl_ey(int tc);
extern int rs_ins_digraph(void);
extern int rs_stuff_inserted(int c, int count, int no_esc);
extern void rs_redo_literal(int c);
extern void rs_check_spell_redraw(void);
extern char *rs_do_insert_char_pre(int c);
extern void rs_start_arrow(void *end_insert_pos);
extern void rs_start_arrow_with_change(void *end_insert_pos, int end_change);
extern void rs_start_arrow_common(void *end_insert_pos, int end_change);
extern int rs_stop_arrow(void);
extern void rs_insert_special(int c, int allow_modmask, int ctrlv);
extern int rs_get_literal(int no_simplify);
extern void rs_clear_showcmd(void);
extern void rs_start_selection(void);

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

/// Get vr_lines_changed (accessor for Rust).
int nvim_edit_get_vr_lines_changed(void)
{
  return vr_lines_changed;
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

/// Call rs_foldOpenCursor() (accessor for Rust).
void nvim_edit_foldOpenCursor(void)
{
  rs_foldOpenCursor();
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

/// Call insertchar(c, flags, second_indent) (accessor for Rust).
void nvim_edit_insertchar(int c, int flags, int second_indent)
{
  insertchar(c, flags, second_indent);
}

/// Call stop_insert(end_insert_pos, esc, nomove) (accessor for Rust).
void nvim_edit_stop_insert(void *end_insert_pos, int esc, int nomove)
{
  stop_insert((pos_T *)end_insert_pos, esc, nomove);
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

// Static asserts for Phase 5 constants
_Static_assert(ABBR_OFF == 0x100, "ABBR_OFF mismatch");
_Static_assert(OPENLINE_DO_COM == 0x02, "OPENLINE_DO_COM mismatch");
_Static_assert(Ctrl_D == 4, "Ctrl_D mismatch");
_Static_assert(kOptBoFlagCopy == 0x10, "kOptBoFlagCopy mismatch");

#define TRIGGER_AUTOCOMPLETE() \
  do { \
    redraw_later(curwin, UPD_VALID); \
    update_screen();  /* Show char deletion immediately */ \
    ui_flush(); \
    rs_ins_compl_enable_autocomplete(); \
    insert_do_complete(s); \
    break; \
  } while (0)

#define MAY_TRIGGER_AUTOCOMPLETE(c) \
  do { \
    if (rs_ins_compl_has_autocomplete() && !char_avail() && curwin->w_cursor.col > 0) { \
      (c) = char_before_cursor(); \
      if (vim_isprintc(c)) { \
        TRIGGER_AUTOCOMPLETE(); \
      } \
    } \
  } while (0)

static void insert_enter(InsertState *s)
{
  s->did_backspace = true;
  s->old_topfill = -1;
  s->replaceState = MODE_REPLACE;
  s->cmdchar_todo = s->cmdchar;
  s->ins_just_started = true;
  // Remember whether editing was restarted after CTRL-O
  did_restart_edit = restart_edit;
  // sleep before redrawing, needed for "CTRL-O :" that results in an
  // error message
  msg_check_for_delay(true);
  // set Insstart_orig to Insstart
  update_Insstart_orig = true;

  rs_ins_compl_clear();        // clear stuff for CTRL-X mode

  // Trigger InsertEnter autocommands.  Do not do this for "r<CR>" or "grx".
  if (s->cmdchar != 'r' && s->cmdchar != 'v') {
    pos_T save_cursor = curwin->w_cursor;

    const char *const ptr = s->cmdchar == 'R' ? "r" : s->cmdchar == 'V' ? "v" : "i";
    set_vim_var_string(VV_INSERTMODE, ptr, 1);
    set_vim_var_string(VV_CHAR, NULL, -1);
    ins_apply_autocmds(EVENT_INSERTENTER);

    // Check for changed highlighting, e.g. for ModeMsg.
    if (need_highlight_changed) {
      highlight_changed();
    }

    // Make sure the cursor didn't move.  Do call check_cursor_col() in
    // case the text was modified.  Since Insert mode was not started yet
    // a call to check_cursor_col() may move the cursor, especially with
    // the "A" command, thus set State to avoid that. Also check that the
    // line number is still valid (lines may have been deleted).
    // Do not restore if v:char was set to a non-empty string.
    if (!equalpos(curwin->w_cursor, save_cursor)
        && *get_vim_var_str(VV_CHAR) == NUL
        && save_cursor.lnum <= curbuf->b_ml.ml_line_count) {
      int save_state = State;

      curwin->w_cursor = save_cursor;
      State = MODE_INSERT;
      check_cursor_col(curwin);
      State = save_state;
    }
  }

  // When doing a paste with the middle mouse button, Insstart is set to
  // where the paste started.
  if (where_paste_started.lnum != 0) {
    Insstart = where_paste_started;
  } else {
    Insstart = curwin->w_cursor;
    if (s->startln) {
      Insstart.col = 0;
    }
  }

  Insstart_textlen = linetabsize_str(get_cursor_line_ptr());
  Insstart_blank_vcol = MAXCOL;

  if (!did_ai) {
    ai_col = 0;
  }

  if (s->cmdchar != NUL && restart_edit == 0) {
    ResetRedobuff();
    AppendNumberToRedobuff(s->count);
    if (s->cmdchar == 'V' || s->cmdchar == 'v') {
      // "gR" or "gr" command
      AppendCharToRedobuff('g');
      AppendCharToRedobuff((s->cmdchar == 'v') ? 'r' : 'R');
    } else {
      AppendCharToRedobuff(s->cmdchar);
      if (s->cmdchar == 'g') {          // "gI" command
        AppendCharToRedobuff('I');
      } else if (s->cmdchar == 'r') {  // "r<CR>" command
        s->count = 1;                  // insert only one <CR>
      }
    }
  }

  if (s->cmdchar == 'R') {
    State = MODE_REPLACE;
  } else if (s->cmdchar == 'V' || s->cmdchar == 'v') {
    State = MODE_VREPLACE;
    s->replaceState = MODE_VREPLACE;
    orig_line_count = curbuf->b_ml.ml_line_count;
    vr_lines_changed = 1;
  } else {
    State = MODE_INSERT;
  }

  may_trigger_modechanged();
  stop_insert_mode = false;

  // need to position cursor again when on a TAB and
  // when on a char with inline virtual text
  if (gchar_cursor() == TAB || buf_meta_total(curbuf, kMTMetaInline) > 0) {
    curwin->w_valid &= ~(VALID_WROW|VALID_WCOL|VALID_VIRTCOL);
  }

  // Enable langmap or IME, indicated by 'iminsert'.
  // Note that IME may enabled/disabled without us noticing here, thus the
  // 'iminsert' value may not reflect what is actually used.  It is updated
  // when hitting <Esc>.
  if (curbuf->b_p_iminsert == B_IMODE_LMAP) {
    State |= MODE_LANGMAP;
  }

  setmouse();
  rs_clear_showcmd();
  // there is no reverse replace mode
  revins_on = (State == MODE_INSERT && p_ri);
  if (revins_on) {
    undisplay_dollar();
  }
  revins_chars = 0;
  revins_legal = 0;
  revins_scol = -1;

  // Handle restarting Insert mode.
  // Don't do this for "CTRL-O ." (repeat an insert): we get here with
  // restart_edit non-zero, and something in the stuff buffer.
  if (restart_edit != 0 && stuff_empty()) {
    // After a paste we consider text typed to be part of the insert for
    // the pasted text. You can backspace over the pasted text too.
    arrow_used = where_paste_started.lnum == 0;
    restart_edit = 0;

    // If the cursor was after the end-of-line before the CTRL-O and it is
    // now at the end-of-line, put it after the end-of-line (this is not
    // correct in very rare cases).
    // Also do this if curswant is greater than the current virtual
    // column.  Eg after "^O$" or "^O80|".
    validate_virtcol(curwin);
    update_curswant();
    const char *ptr;
    if (((ins_at_eol && curwin->w_cursor.lnum == o_lnum)
         || curwin->w_curswant > curwin->w_virtcol)
        && *(ptr = get_cursor_line_ptr() + curwin->w_cursor.col) != NUL) {
      if (ptr[1] == NUL) {
        curwin->w_cursor.col++;
      } else {
        s->i = utfc_ptr2len(ptr);
        if (ptr[s->i] == NUL) {
          curwin->w_cursor.col += s->i;
        }
      }
    }
    ins_at_eol = false;
  } else {
    arrow_used = false;
  }

  // we are in insert mode now, don't need to start it anymore
  need_start_insertmode = false;

  // Need to save the line for undo before inserting the first char.
  ins_need_undo = true;

  where_paste_started.lnum = 0;
  can_cindent = true;
  // The cursor line is not in a closed fold, unless restarting.
  if (did_restart_edit == 0) {
    rs_foldOpenCursor();
  }

  // If 'showmode' is set, show the current (insert/replace/..) mode.
  // A warning message for changing a readonly file is given here, before
  // actually changing anything.  It's put after the mode, if any.
  s->i = 0;
  if (p_smd && msg_silent == 0) {
    s->i = showmode();
  }

  if (did_restart_edit == 0) {
    change_warning(curbuf, s->i == 0 ? 0 : s->i + 1);
  }

  ui_cursor_shape();            // may show different cursor shape
  do_digraph(-1);               // clear digraphs

  // Get the current length of the redo buffer, those characters have to be
  // skipped if we want to get to the inserted characters.
  String inserted = get_inserted();
  new_insert_skip = (int)inserted.size;
  if (inserted.data != NULL) {
    xfree(inserted.data);
  }

  old_indent = 0;

  do {
    state_enter(&s->state);
    // If s->count != 0, `ins_esc` will prepare the redo buffer for reprocessing
    // and return false, causing `state_enter` to be called again.
  } while (!ins_esc(&s->count, s->cmdchar, s->nomove));

  // Always update o_lnum, so that a "CTRL-O ." that adds a line
  // still puts the cursor back after the inserted text.
  if (ins_at_eol) {
    o_lnum = curwin->w_cursor.lnum;
  }

  pum_check_clear();

  rs_foldUpdateAfterInsert();
  // When CTRL-C was typed got_int will be set, with the result
  // that the autocommands won't be executed. When mapped got_int
  // is not set, but let's keep the behavior the same.
  if (s->cmdchar != 'r' && s->cmdchar != 'v' && s->c != Ctrl_C) {
    ins_apply_autocmds(EVENT_INSERTLEAVE);
  }
  did_cursorhold = false;

  // ins_redraw() triggers TextChangedI only when no characters
  // are in the typeahead buffer, so reset curbuf->b_last_changedtick
  // if the TextChangedI was not blocked by char_avail() (e.g. using :norm!)
  // and the TextChangedI autocommand has been triggered.
  if (!char_avail() && curbuf->b_last_changedtick_i == buf_get_changedtick(curbuf)) {
    curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  }
}

static int insert_check(VimState *state)
{
  InsertState *s = (InsertState *)state;

  if (!revins_legal) {
    revins_scol = -1;     // reset on illegal motions
  } else {
    revins_legal = 0;
  }

  if (arrow_used) {       // don't repeat insert when arrow key used
    s->count = 0;
  }

  if (update_Insstart_orig) {
    Insstart_orig = Insstart;
  }

  if (curbuf->terminal && !stop_insert_mode) {
    // Exit Insert mode and go to Terminal mode.
    stop_insert_mode = true;
    restart_edit = 'I';
    stuffcharReadbuff(K_NOP);
  }

  if (stop_insert_mode && !rs_ins_compl_active()) {
    // ":stopinsert" used
    s->count = 0;
    return 0;  // exit insert mode
  }

  // set curwin->w_curswant for next K_DOWN or K_UP
  if (!arrow_used) {
    curwin->w_set_curswant = true;
  }

  // If there is no typeahead may check for timestamps (e.g., for when a
  // menu invoked a shell command).
  if (stuff_empty()) {
    did_check_timestamps = false;
    if (need_check_timestamps) {
      check_timestamps(false);
    }
  }

  // When emsg() was called msg_scroll will have been set.
  msg_scroll = false;

  // Open fold at the cursor line, according to 'foldopen'.
  if (fdo_flags & kOptFdoFlagInsert) {
    rs_foldOpenCursor();
  }

  // Close folds where the cursor isn't, according to 'foldclose'
  if (!char_avail()) {
    rs_foldCheckClose();
  }

  if (bt_prompt(curbuf)) {
    init_prompt(s->cmdchar_todo);
    s->cmdchar_todo = NUL;
  }

  // If we inserted a character at the last position of the last line in the
  // window, scroll the window one line up. This avoids an extra redraw.  This
  // is detected when the cursor column is smaller after inserting something.
  // Don't do this when the topline changed already, it has already been
  // adjusted (by insertchar() calling open_line())).
  // Also don't do this when 'smoothscroll' is set, as the window should then
  // be scrolled by screen lines.
  if (curbuf->b_mod_set
      && curwin->w_p_wrap
      && !curwin->w_p_sms
      && !s->did_backspace
      && curwin->w_topline == s->old_topline
      && curwin->w_topfill == s->old_topfill
      && s->count <= 1) {
    s->mincol = curwin->w_wcol;
    validate_cursor_col(curwin);

    if (curwin->w_wcol < s->mincol - tabstop_at(get_nolist_virtcol(),
                                                curbuf->b_p_ts,
                                                curbuf->b_p_vts_array,
                                                false)
        && curwin->w_wrow == curwin->w_view_height - 1 - rs_get_scrolloff_value(curwin)
        && (curwin->w_cursor.lnum != curwin->w_topline
            || curwin->w_topfill > 0)) {
      if (curwin->w_topfill > 0) {
        curwin->w_topfill--;
      } else if (hasFolding(curwin, curwin->w_topline, NULL, &s->old_topline)) {
        set_topline(curwin, s->old_topline + 1);
      } else {
        set_topline(curwin, curwin->w_topline + 1);
      }
    }
  }

  // May need to adjust w_topline to show the cursor.
  if (s->count <= 1) {
    update_topline(curwin);
  }

  s->did_backspace = false;

  if (s->count <= 1) {
    validate_cursor(curwin);  // may set must_redraw
  }

  // Redraw the display when no characters are waiting.
  // Also shows mode, ruler and positions cursor.
  ins_redraw(true);

  if (curwin->w_p_scb) {
    do_check_scrollbind(true);
  }

  if (curwin->w_p_crb) {
    do_check_cursorbind();
  }

  if (s->count <= 1) {
    update_curswant();
  }
  s->old_topline = curwin->w_topline;
  s->old_topfill = curwin->w_topfill;

  if (s->c != K_EVENT) {
    s->lastc = s->c;  // remember previous char for CTRL-D
  }

  // After using CTRL-G U the next cursor key will not break undo.
  if (dont_sync_undo == kNone) {
    dont_sync_undo = kTrue;
  } else {
    dont_sync_undo = kFalse;
  }

  // Trigger autocomplete when entering Insert mode, either directly
  // or via change commands like 'ciw', 'cw', etc., before the first
  // character is typed.
  if (s->ins_just_started) {
    s->ins_just_started = false;
    if (rs_ins_compl_has_autocomplete() && !char_avail() && curwin->w_cursor.col > 0) {
      s->c = char_before_cursor();
      if (vim_isprintc(s->c)) {
        rs_ins_compl_enable_autocomplete();
        rs_ins_compl_init_get_longest();
        insert_do_complete(s);
        insert_handle_key_post(s);
        return 1;
      }
    }
  }

  return 1;
}

static int insert_execute(VimState *state, int key)
{
  InsertState *const s = (InsertState *)state;
  if (stop_insert_mode) {
    // Insert mode ended, possibly from a callback.
    if (key != K_IGNORE && key != K_NOP) {
      vungetc(key);
    }
    s->count = 0;
    s->nomove = true;
    rs_ins_compl_prep(ESC);
    return 0;
  }

  if (key == K_IGNORE || key == K_NOP) {
    return -1;  // get another key
  }
  s->c = key;

  // Don't want K_EVENT with cursorhold for the second key, e.g., after CTRL-V.
  if (key != K_EVENT) {
    did_cursorhold = true;
  }

  // Special handling of keys while the popup menu is visible or wanted
  // and the cursor is still in the completed word.  Only when there is
  // a match, skip this when no matches were found.
  if (rs_ins_compl_active() && curwin->w_cursor.col >= rs_ins_compl_col()
      && rs_ins_compl_has_shown_match() && rs_pum_wanted()) {
    // BS: Delete one character from "compl_leader".
    if ((s->c == K_BS || s->c == Ctrl_H)
        && curwin->w_cursor.col > rs_ins_compl_col()
        && (s->c = rs_ins_compl_bs()) == NUL) {
      return 1;  // continue
    }

    // When no match was selected or it was edited.
    if (!rs_ins_compl_used_match()) {
      // CTRL-L: Add one character from the current match to
      // "compl_leader".  Except when at the original match and
      // there is nothing to add, CTRL-L works like CTRL-P then.
      if (s->c == Ctrl_L
          && (!rs_ctrl_x_mode_line_or_eval()
              || rs_ins_compl_long_shown_match())) {
        rs_ins_compl_addfrommatch();
        return 1;  // continue
      }

      // A non-white character that fits in with the current
      // completion: Add to "compl_leader".
      if (rs_ins_compl_accept_char(s->c)) {
        // Trigger InsertCharPre.
        char *str = do_insert_char_pre(s->c);

        if (str != NULL) {
          for (char *p = str; *p != NUL; MB_PTR_ADV(p)) {
            rs_ins_compl_addleader(utf_ptr2char(p));
          }
          xfree(str);
        } else {
          rs_ins_compl_addleader(s->c);
        }
        return 1;  // continue
      }

      // Pressing CTRL-Y selects the current match.  When
      // compl_enter_selects is set the Enter key does the same.
      if ((s->c == Ctrl_Y
           || (rs_ins_compl_enter_selects()
               && (s->c == CAR || s->c == K_KENTER || s->c == NL)))
          && stop_arrow() == OK) {
        rs_ins_compl_delete(0);
        if (rs_ins_compl_preinsert_longest() && !rs_ins_compl_is_match_selected()) {
          rs_ins_compl_insert(0, 1);
          rs_ins_compl_init_get_longest();
          return 1;  // continue
        } else {
          rs_ins_compl_insert(0, 0);
        }
      } else if (ascii_iswhite_nl_or_nul(s->c) && rs_ins_compl_preinsert_effect()) {
        // Delete preinserted text when typing special chars
        rs_ins_compl_delete(0);
      }
    }
  }

  // Prepare for or stop CTRL-X mode. This doesn't do completion, but it does
  // fix up the text when finishing completion.
  rs_ins_compl_init_get_longest();
  if (rs_ins_compl_prep(s->c)) {
    return 1;  // continue
  }

  // CTRL-\ CTRL-N goes to Normal mode,
  // CTRL-\ CTRL-O is like CTRL-O but without moving the cursor
  if (s->c == Ctrl_BSL) {
    // may need to redraw when no more chars available now
    ins_redraw(false);
    no_mapping++;
    allow_keys++;
    s->c = plain_vgetc();
    no_mapping--;
    allow_keys--;
    if (s->c != Ctrl_N && s->c != Ctrl_G && s->c != Ctrl_O) {
      // it's something else
      vungetc(s->c);
      s->c = Ctrl_BSL;
    } else {
      if (s->c == Ctrl_O) {
        ins_ctrl_o();
        ins_at_eol = false;  // cursor keeps its column
        s->nomove = true;
      }
      s->count = 0;
      return 0;
    }
  }

  if (s->c != K_EVENT) {
    s->c = do_digraph(s->c);
  }

  if ((s->c == Ctrl_V || s->c == Ctrl_Q) && rs_ctrl_x_mode_cmdline()) {
    insert_do_complete(s);
    insert_handle_key_post(s);
    return 1;
  }

  if (s->c == Ctrl_V || s->c == Ctrl_Q) {
    ins_ctrl_v();
    s->c = Ctrl_V;       // pretend CTRL-V is last typed character
    return 1;  // continue
  }

  if (cindent_on() && rs_ctrl_x_mode_none()) {
    s->line_is_white = inindent(0);
    // A key name preceded by a bang means this key is not to be
    // inserted.  Skip ahead to the re-indenting below.
    if (in_cinkeys(s->c, '!', s->line_is_white)
        && stop_arrow() == OK) {
      do_c_expr_indent();
      return 1;  // continue
    }
    // A key name preceded by a star means that indenting has to be
    // done before inserting the key.
    if (can_cindent && in_cinkeys(s->c, '*', s->line_is_white)
        && stop_arrow() == OK) {
      do_c_expr_indent();
    }
  }

  if (curwin->w_p_rl) {
    switch (s->c) {
    case K_LEFT:
      s->c = K_RIGHT; break;
    case K_S_LEFT:
      s->c = K_S_RIGHT; break;
    case K_C_LEFT:
      s->c = K_C_RIGHT; break;
    case K_RIGHT:
      s->c = K_LEFT; break;
    case K_S_RIGHT:
      s->c = K_S_LEFT; break;
    case K_C_RIGHT:
      s->c = K_C_LEFT; break;
    }
  }

  // If 'keymodel' contains "startsel", may start selection.  If it
  // does, a CTRL-O and c will be stuffed, we need to get these
  // characters.
  if (ins_start_select(s->c)) {
    return 1;  // continue
  }

  return insert_handle_key(s);
}

static int insert_handle_key(InsertState *s)
{
  // The big switch to handle a character in insert mode.
  // TODO(tarruda): This could look better if a lookup table is used.
  // (similar to normal mode `nv_cmds[]`)
  switch (s->c) {
  case ESC:           // End input mode
    if (echeck_abbr(ESC + ABBR_OFF)) {
      break;
    }
    FALLTHROUGH;

  case Ctrl_C:        // End input mode
    if (s->c == Ctrl_C && cmdwin_type != 0) {
      // Close the cmdline window.
      cmdwin_result = K_IGNORE;
      got_int = false;         // don't stop executing autocommands et al
      s->nomove = true;
      return 0;  // exit insert mode
    }
    if (s->c == Ctrl_C && bt_prompt(curbuf)) {
      if (invoke_prompt_interrupt()) {
        if (!bt_prompt(curbuf)) {
          // buffer changed to a non-prompt buffer, get out of
          // Insert mode
          return 0;
        }
        break;
      }
    }

    return 0;  // exit insert mode

  case Ctrl_Z:
    goto normalchar;                // insert CTRL-Z as normal char

  case Ctrl_O:        // execute one command
    if (rs_ctrl_x_mode_omni()) {
      insert_do_complete(s);
      break;
    }

    if (echeck_abbr(Ctrl_O + ABBR_OFF)) {
      break;
    }

    ins_ctrl_o();

    // don't move the cursor left when 'virtualedit' has "onemore".
    if (get_ve_flags(curwin) & kOptVeFlagOnemore) {
      ins_at_eol = false;
      s->nomove = true;
    }

    s->count = 0;
    return 0;  // exit insert mode

  case K_INS:         // toggle insert/replace mode
  case K_KINS:
    ins_insert(s->replaceState);
    break;

  case K_SELECT:      // end of Select mode mapping - ignore
    break;

  case K_HELP:        // Help key works like <ESC> <Help>
  case K_F1:
  case K_XF1:
    stuffcharReadbuff(K_HELP);
    return 0;  // exit insert mode

  case ' ':
    if (mod_mask != MOD_MASK_CTRL) {
      goto normalchar;
    }
    FALLTHROUGH;
  case K_ZERO:        // Insert the previously inserted text.
  case NUL:
  case Ctrl_A:
    // For ^@ the trailing ESC will end the insert, unless there is an
    // error.
    if (stuff_inserted(NUL, 1, (s->c == Ctrl_A)) == FAIL
        && s->c != Ctrl_A) {
      return 0;  // exit insert mode
    }
    s->inserted_space = false;
    break;

  case Ctrl_R:        // insert the contents of a register
    if (rs_ctrl_x_mode_register() && !rs_ins_compl_active()) {
      insert_do_complete(s);
      break;
    }
    ins_reg();
    auto_format(false, true);
    s->inserted_space = false;
    break;

  case Ctrl_G:        // commands starting with CTRL-G
    ins_ctrl_g();
    break;

  case Ctrl_HAT:      // switch input mode and/or langmap
    ins_ctrl_hat();
    break;

  case Ctrl__:        // switch between languages
    if (!p_ari) {
      goto normalchar;
    }
    ins_ctrl_();
    break;

  case Ctrl_D:        // Make indent one shiftwidth smaller.
    if (rs_ctrl_x_mode_path_defines()) {
      insert_do_complete(s);
      break;
    }
    FALLTHROUGH;

  case Ctrl_T:        // Make indent one shiftwidth greater.
    if (s->c == Ctrl_T && rs_ctrl_x_mode_thesaurus()) {
      if (check_compl_option(false)) {
        insert_do_complete(s);
      }
      break;
    }
    ins_shift(s->c, s->lastc);
    auto_format(false, true);
    s->inserted_space = false;
    break;

  case K_DEL:         // delete character under the cursor
  case K_KDEL:
    ins_del();
    auto_format(false, true);
    break;

  case K_BS:          // delete character before the cursor
  case Ctrl_H:
    s->did_backspace = ins_bs(s->c, BACKSPACE_CHAR, &s->inserted_space);
    auto_format(false, true);
    if (s->did_backspace) {
      MAY_TRIGGER_AUTOCOMPLETE(s->c);
    }
    break;

  case Ctrl_W:        // delete word before the cursor
    if (bt_prompt(curbuf) && (mod_mask & MOD_MASK_SHIFT) == 0) {
      // In a prompt window CTRL-W is used for window commands.
      // Use Shift-CTRL-W to delete a word.
      stuffcharReadbuff(Ctrl_W);
      restart_edit = 'A';
      s->nomove = true;
      s->count = 0;
      return 0;
    }
    s->did_backspace = ins_bs(s->c, BACKSPACE_WORD, &s->inserted_space);
    auto_format(false, true);
    if (s->did_backspace) {
      MAY_TRIGGER_AUTOCOMPLETE(s->c);
    }
    break;

  case Ctrl_U:        // delete all inserted text in current line
    // CTRL-X CTRL-U completes with 'completefunc'.
    if (rs_ctrl_x_mode_function()) {
      insert_do_complete(s);
    } else {
      s->did_backspace = ins_bs(s->c, BACKSPACE_LINE, &s->inserted_space);
      auto_format(false, true);
      s->inserted_space = false;
      if (s->did_backspace) {
        MAY_TRIGGER_AUTOCOMPLETE(s->c);
      }
    }
    break;

  case K_LEFTMOUSE:     // mouse keys
  case K_LEFTMOUSE_NM:
  case K_LEFTDRAG:
  case K_LEFTRELEASE:
  case K_LEFTRELEASE_NM:
  case K_MOUSEMOVE:
  case K_MIDDLEMOUSE:
  case K_MIDDLEDRAG:
  case K_MIDDLERELEASE:
  case K_RIGHTMOUSE:
  case K_RIGHTDRAG:
  case K_RIGHTRELEASE:
  case K_X1MOUSE:
  case K_X1DRAG:
  case K_X1RELEASE:
  case K_X2MOUSE:
  case K_X2DRAG:
  case K_X2RELEASE:
    ins_mouse(s->c);
    break;

  case K_MOUSEDOWN:   // Default action for scroll wheel up: scroll up
    ins_mousescroll(MSCR_DOWN);
    break;

  case K_MOUSEUP:     // Default action for scroll wheel down: scroll down
    ins_mousescroll(MSCR_UP);
    break;

  case K_MOUSELEFT:   // Scroll wheel left
    ins_mousescroll(MSCR_LEFT);
    break;

  case K_MOUSERIGHT:  // Scroll wheel right
    ins_mousescroll(MSCR_RIGHT);
    break;

  case K_IGNORE:      // Something mapped to nothing
    break;

  case K_PASTE_START:
    paste_repeat(1);
    goto check_pum;

  case K_EVENT:       // some event
    state_handle_k_event();
    // If CTRL-G U was used apply it to the next typed key.
    if (dont_sync_undo == kTrue) {
      dont_sync_undo = kNone;
    }
    goto check_pum;

  case K_COMMAND:     // <Cmd>command<CR>
    do_cmdline(NULL, getcmdkeycmd, NULL, 0);
    goto check_pum;

  case K_LUA:
    map_execute_lua(false, false);

check_pum:
    // nvim_select_popupmenu_item() can be called from the handling of
    // K_EVENT, K_COMMAND, or K_LUA.
    // TODO(bfredl): Not entirely sure this indirection is necessary
    // but doing like this ensures using nvim_select_popupmenu_item is
    // equivalent to selecting the item with a typed key.
    if (pum_want.active) {
      if (pum_visible()) {
        // Set this to NULL so that ins_complete() will update the message.
        edit_submode_extra = NULL;
        insert_do_complete(s);
        if (pum_want.finish) {
          // accept the item and stop completion
          rs_ins_compl_prep(Ctrl_Y);
        }
      }
      pum_want.active = false;
    }

    if (curbuf->b_u_synced) {
      // The K_EVENT, K_COMMAND, or K_LUA caused undo to be synced.
      // Need to save the line for undo before inserting the next char.
      ins_need_undo = true;
    }
    break;

  case K_HOME:        // <Home>
  case K_KHOME:
  case K_S_HOME:
  case K_C_HOME:
    ins_home(s->c);
    break;

  case K_END:         // <End>
  case K_KEND:
  case K_S_END:
  case K_C_END:
    ins_end(s->c);
    break;

  case K_LEFT:        // <Left>
    if (mod_mask & (MOD_MASK_SHIFT|MOD_MASK_CTRL)) {
      ins_s_left();
    } else {
      ins_left();
    }
    break;

  case K_S_LEFT:      // <S-Left>
  case K_C_LEFT:
    ins_s_left();
    break;

  case K_RIGHT:       // <Right>
    if (mod_mask & (MOD_MASK_SHIFT|MOD_MASK_CTRL)) {
      ins_s_right();
    } else {
      ins_right();
    }
    break;

  case K_S_RIGHT:     // <S-Right>
  case K_C_RIGHT:
    ins_s_right();
    break;

  case K_UP:          // <Up>
    if (pum_visible()) {
      insert_do_complete(s);
    } else if (mod_mask & MOD_MASK_SHIFT) {
      ins_pageup();
    } else {
      ins_up(false);
    }
    break;

  case K_S_UP:        // <S-Up>
  case K_PAGEUP:
  case K_KPAGEUP:
    if (pum_visible()) {
      insert_do_complete(s);
    } else {
      ins_pageup();
    }
    break;

  case K_DOWN:        // <Down>
    if (pum_visible()) {
      insert_do_complete(s);
    } else if (mod_mask & MOD_MASK_SHIFT) {
      ins_pagedown();
    } else {
      ins_down(false);
    }
    break;

  case K_S_DOWN:      // <S-Down>
  case K_PAGEDOWN:
  case K_KPAGEDOWN:
    if (pum_visible()) {
      insert_do_complete(s);
    } else {
      ins_pagedown();
    }
    break;

  case K_S_TAB:       // When not mapped, use like a normal TAB
    s->c = TAB;
    FALLTHROUGH;

  case TAB:           // TAB or Complete patterns along path
    if (rs_ctrl_x_mode_path_patterns()) {
      insert_do_complete(s);
      break;
    }
    s->inserted_space = false;
    if (ins_tab()) {
      goto normalchar;                // insert TAB as a normal char
    }
    auto_format(false, true);
    break;

  case K_KENTER:      // <Enter>
    s->c = CAR;
    FALLTHROUGH;
  case CAR:
  case NL:
    // In a quickfix window a <CR> jumps to the error under the
    // cursor.
    if (bt_quickfix(curbuf) && s->c == CAR) {
      if (curwin->w_llist_ref == NULL) {          // quickfix window
        do_cmdline_cmd(".cc");
      } else {                                    // location list window
        do_cmdline_cmd(".ll");
      }
      break;
    }
    if (cmdwin_type != 0) {
      // Execute the command in the cmdline window.
      cmdwin_result = CAR;
      return 0;
    }
    if ((mod_mask & MOD_MASK_SHIFT) == 0 && bt_prompt(curbuf)) {
      prompt_invoke_callback();
      if (!bt_prompt(curbuf)) {
        // buffer changed to a non-prompt buffer, get out of
        // Insert mode
        return 0;
      }
      break;
    }
    if (!ins_eol(s->c)) {
      return 0;  // out of memory
    }
    auto_format(false, false);
    s->inserted_space = false;
    break;

  case Ctrl_K:        // digraph or keyword completion
    if (rs_ctrl_x_mode_dictionary()) {
      if (check_compl_option(true)) {
        insert_do_complete(s);
      }
      break;
    }

    s->c = ins_digraph();
    if (s->c == NUL) {
      break;
    }
    goto normalchar;

  case Ctrl_X:        // Enter CTRL-X mode
    ins_ctrl_x();
    break;

  case Ctrl_RSB:      // Tag name completion after ^X
    if (!rs_ctrl_x_mode_tags()) {
      goto normalchar;
    } else {
      insert_do_complete(s);
    }
    break;

  case Ctrl_F:        // File name completion after ^X
    if (!rs_ctrl_x_mode_files()) {
      goto normalchar;
    } else {
      insert_do_complete(s);
    }
    break;

  case 's':           // Spelling completion after ^X
  case Ctrl_S:
    if (!rs_ctrl_x_mode_spell()) {
      goto normalchar;
    } else {
      insert_do_complete(s);
    }
    break;

  case Ctrl_L:        // Whole line completion after ^X
    if (!rs_ctrl_x_mode_whole_line()) {
      goto normalchar;
    }
    FALLTHROUGH;

  case Ctrl_P:        // Do previous/next pattern completion
  case Ctrl_N:
    // if 'complete' is empty then plain ^P is no longer special,
    // but it is under other ^X modes
    if (*curbuf->b_p_cpt == NUL
        && (rs_ctrl_x_mode_normal() || rs_ctrl_x_mode_whole_line())
        && !rs_compl_status_local()) {
      goto normalchar;
    }

    insert_do_complete(s);
    break;

  case Ctrl_Y:        // copy from previous line or scroll down
  case Ctrl_E:        // copy from next line or scroll up
    s->c = ins_ctrl_ey(s->c);
    break;

  default:

normalchar:
    // Insert a normal character.

    if (!p_paste) {
      // Trigger InsertCharPre.
      char *str = do_insert_char_pre(s->c);

      if (str != NULL) {
        if (*str != NUL && stop_arrow() != FAIL) {
          // Insert the new value of v:char literally.
          for (char *p = str; *p != NUL; MB_PTR_ADV(p)) {
            s->c = utf_ptr2char(p);
            if (s->c == CAR || s->c == K_KENTER || s->c == NL) {
              ins_eol(s->c);
            } else {
              ins_char(s->c);
            }
          }
          AppendToRedobuffLit(str, -1);
        }
        xfree(str);
        s->c = NUL;
      }

      // If the new value is already inserted or an empty string
      // then don't insert any character.
      if (s->c == NUL) {
        break;
      }
    }
    // Try to perform smart-indenting.
    ins_try_si(s->c);

    if (s->c == ' ') {
      s->inserted_space = true;
      if (inindent(0)) {
        can_cindent = false;
      }
      if (Insstart_blank_vcol == MAXCOL
          && curwin->w_cursor.lnum == Insstart.lnum) {
        Insstart_blank_vcol = get_nolist_virtcol();
      }
    }

    // Insert a normal character and check for abbreviations on a
    // special character.  Let CTRL-] expand abbreviations without
    // inserting it.
    if (vim_iswordc(s->c)
        // Add ABBR_OFF for characters above 0x100, this is
        // what check_abbr() expects.
        || (!echeck_abbr((s->c >= 0x100) ? (s->c + ABBR_OFF) : s->c)
            && s->c != Ctrl_RSB)) {
      insert_special(s->c, false, false);
      revins_legal++;
      revins_chars++;
    }

    auto_format(false, true);

    // When inserting a character the cursor line must never be in a
    // closed fold.
    rs_foldOpenCursor();
    // Trigger autocompletion
    if (rs_ins_compl_has_autocomplete() && !char_avail() && vim_isprintc(s->c)) {
      TRIGGER_AUTOCOMPLETE();
    }

    break;
  }       // end of switch (s->c)

  insert_handle_key_post(s);
  return 1;  // continue
}

static void insert_do_complete(InsertState *s)
{
  compl_busy = true;
  disable_fold_update++;  // don't redraw folds here
  if (ins_complete(s->c, true) == FAIL) {
    rs_compl_status_clear();
  }
  disable_fold_update--;
  compl_busy = false;
  can_si = may_do_si();  // allow smartindenting
}

static void insert_do_cindent(InsertState *s)
{
  // Indent now if a key was typed that is in 'cinkeys'.
  if (in_cinkeys(s->c, ' ', s->line_is_white)) {
    if (stop_arrow() == OK) {
      // re-indent the current line
      do_c_expr_indent();
    }
  }
}

static void insert_handle_key_post(InsertState *s)
{
  // If typed something may trigger CursorHoldI again.
  if (s->c != K_EVENT
      // but not in CTRL-X mode, a script can't restore the state
      && rs_ctrl_x_mode_normal()) {
    did_cursorhold = false;
  }

  // Check if we need to cancel completion mode because the window
  // or tab page was changed
  if (rs_ins_compl_active() && !rs_ins_compl_win_active(curwin)) {
    rs_ins_compl_cancel();
  }

  // If the cursor was moved we didn't just insert a space
  if (arrow_used) {
    s->inserted_space = false;
  }

  if (can_cindent && cindent_on() && rs_ctrl_x_mode_normal()) {
    insert_do_cindent(s);
  }
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
bool edit(int cmdchar, bool startln, int count)
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
  s->state.execute = insert_execute;
  s->state.check = insert_check;
  s->cmdchar = cmdchar;
  s->startln = startln;
  s->count = count;
  insert_enter(s);
  return s->c == Ctrl_O;
}

bool ins_need_undo_get(void)
{
  return rs_ins_need_undo_get() != 0;
}

/// Redraw for Insert mode.
/// This is postponed until getting the next character to make '$' in the 'cpo'
/// option work correctly.
/// Only redraw when there are no characters available.  This speeds up
/// inserting sequences of characters (e.g., for CTRL-R).
///
/// @param ready  not busy with something
void ins_redraw(bool ready)
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

// Handle a CTRL-V or CTRL-Q typed in Insert mode.
static void ins_ctrl_v(void)
{
  rs_ins_ctrl_v();
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

/// @return    the effective prompt for the specified buffer.
char *buf_prompt_text(const buf_T *const buf)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return rs_buf_prompt_text(buf);
}

/// @return  the effective prompt for the current buffer.
char *prompt_text(void)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return rs_prompt_text();
}

// Prepare for prompt mode: Make sure the last line has the prompt text.
// Move the cursor to this line.
static void init_prompt(int cmdchar_todo)
{
  char *prompt = prompt_text();

  if (curwin->w_cursor.lnum < curbuf->b_prompt_start.mark.lnum) {
    curwin->w_cursor.lnum = curbuf->b_prompt_start.mark.lnum;
  }
  char *text = get_cursor_line_ptr();
  if ((curbuf->b_prompt_start.mark.lnum == curwin->w_cursor.lnum
       && strncmp(text, prompt, strlen(prompt)) != 0)
      || curbuf->b_prompt_start.mark.lnum > curwin->w_cursor.lnum) {
    // prompt is missing, insert it or append a line with it
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

  // Insert always starts after the prompt, allow editing text after it.
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
  // Make sure the cursor is in a valid position.
  check_cursor(curwin);
}

/// @return  true if the cursor is in the editable position of the prompt line.
bool prompt_curpos_editable(void)
  FUNC_ATTR_PURE
{
  return rs_prompt_curpos_editable();
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

// Call this function before moving the cursor from the normal insert position
// in insert mode.
void undisplay_dollar(void)
{
  rs_undisplay_dollar();
}

/// Truncate the space at the end of a line.  This is to be used only in an
/// insert mode.  It handles fixing the replace stack for MODE_REPLACE and
/// MODE_VREPLACE modes.
void truncate_spaces(char *line, size_t len)
{
  rs_truncate_spaces(line, len);
}

/// Backspace the cursor until the given column.  Handles MODE_REPLACE and
/// MODE_VREPLACE modes correctly.  May also be used when not in insert mode at
/// all.  Will attempt not to go before "col" even when there is a composing
/// character.
void backspace_until_column(int col)
{
  rs_backspace_until_column(col);
}

/// Like del_char(), but make sure not to go before column "limit_col".
/// Only matters when there are composing characters.
///
/// @param  limit_col  only delete the character if it is after this column
//
/// @return true when something was deleted.
static bool del_char_after_col(int limit_col)
{
  return rs_del_char_after_col(limit_col) != 0;
}

/// Next character is interpreted literally.
int get_literal(bool no_simplify)
{
  return rs_get_literal(no_simplify ? 1 : 0);
}

/// Insert character, taking care of special keys and mod_mask.
static void insert_special(int c, int allow_modmask, int ctrlv)
{
  rs_insert_special(c, allow_modmask, ctrlv);
}

// Special characters in this context are those that need processing other
// than the simple insertion that can be performed here. This includes ESC
// which terminates the insert, and CR/NL which need special processing to
// open up a new line. This routine tries to optimize insertions performed by
// the "redo", "undo" or "put" commands, so it needs to know when it should
// stop and defer processing to the "normal" mechanism.
// '0' and '^' are special, because they can be followed by CTRL-D.
#define ISSPECIAL(c)   ((c) < ' ' || (c) >= DEL || (c) == '0' || (c) == '^')

/// "flags": INSCHAR_FORMAT - force formatting
///          INSCHAR_CTRLV  - char typed just after CTRL-V
///          INSCHAR_NO_FEX - don't use 'formatexpr'
///
///   NOTE: passes the flags value straight through to internal_format() which,
///         beside INSCHAR_FORMAT (above), is also looking for these:
///          INSCHAR_DO_COM   - format comments
///          INSCHAR_COM_LIST - format comments with num list or 2nd line indent
///
/// @param c              character to insert or NUL
/// @param flags          INSCHAR_FORMAT, etc.
/// @param second_indent  indent for second line if >= 0
void insertchar(int c, int flags, int second_indent)
{
  char *p;
  int force_format = flags & INSCHAR_FORMAT;

  const int textwidth = comp_textwidth(force_format);
  const bool fo_ins_blank = has_format_option(FO_INS_BLANK);

  // Try to break the line in two or more pieces when:
  // - Always do this if we have been called to do formatting only.
  // - Always do this when 'formatoptions' has the 'a' flag and the line
  //   ends in white space.
  // - Otherwise:
  //     - Don't do this if inserting a blank
  //     - Don't do this if an existing character is being replaced, unless
  //       we're in MODE_VREPLACE state.
  //     - Do this if the cursor is not on the line where insert started
  //     or - 'formatoptions' doesn't have 'l' or the line was not too long
  //           before the insert.
  //        - 'formatoptions' doesn't have 'b' or a blank was inserted at or
  //          before 'textwidth'
  if (textwidth > 0
      && (force_format
          || (!ascii_iswhite(c)
              && !((State & REPLACE_FLAG)
                   && !(State & VREPLACE_FLAG)
                   && *get_cursor_pos_ptr() != NUL)
              && (curwin->w_cursor.lnum != Insstart.lnum
                  || ((!has_format_option(FO_INS_LONG)
                       || Insstart_textlen <= (colnr_T)textwidth)
                      && (!fo_ins_blank
                          || Insstart_blank_vcol <= (colnr_T)textwidth)))))) {
    // Format with 'formatexpr' when it's set.  Use internal formatting
    // when 'formatexpr' isn't set or it returns non-zero.
    bool do_internal = true;
    colnr_T virtcol = get_nolist_virtcol()
                      + char2cells(c != NUL ? c : gchar_cursor());

    if (*curbuf->b_p_fex != NUL && (flags & INSCHAR_NO_FEX) == 0
        && (force_format || virtcol > (colnr_T)textwidth)) {
      do_internal = (fex_format(curwin->w_cursor.lnum, 1, c) != 0);
      // It may be required to save for undo again, e.g. when setline()
      // was called.
      ins_need_undo = true;
    }
    if (do_internal) {
      internal_format(textwidth, second_indent, flags, c == NUL, c);
    }
  }

  if (c == NUL) {           // only formatting was wanted
    return;
  }

  // Check whether this character should end a comment.
  if (did_ai && c == end_comment_pending) {
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
  end_comment_pending = NUL;

  did_ai = false;
  did_si = false;
  can_si = false;
  can_si_back = false;

  // If there's any pending input, grab up to INPUT_BUFLEN at once.
  // This speeds up normal text input considerably.
  // Don't do this when 'cindent' or 'indentexpr' is set, because we might
  // need to re-indent at a ':', or any other character (but not what
  // 'paste' is set)..
  // Don't do this when there an InsertCharPre autocommand is defined,
  // because we need to fire the event for every character.
  // Do the check for InsertCharPre before the call to vpeekc() because the
  // InsertCharPre autocommand could change the input buffer.
  if (!ISSPECIAL(c)
      && (utf_char2len(c) == 1)
      && !has_event(EVENT_INSERTCHARPRE)
      && !test_disable_char_avail
      && vpeekc() != NUL
      && !(State & REPLACE_FLAG)
      && !cindent_on()
      && !p_ri) {
#define INPUT_BUFLEN 100
    char buf[INPUT_BUFLEN + 1];
    colnr_T virtcol = 0;

    buf[0] = (char)c;
    int i = 1;
    if (textwidth > 0) {
      virtcol = get_nolist_virtcol();
    }
    // Stop the string when:
    // - no more chars available
    // - finding a special character (command key)
    // - buffer is full
    // - running into the 'textwidth' boundary
    // - need to check for abbreviation: A non-word char after a word-char
    while ((c = vpeekc()) != NUL
           && !ISSPECIAL(c)
           && MB_BYTE2LEN(c) == 1
           && i < INPUT_BUFLEN
           && (textwidth == 0
               || (virtcol += byte2cells((uint8_t)buf[i - 1])) < (colnr_T)textwidth)
           && !(!no_abbr && !vim_iswordc(c) && vim_iswordc((uint8_t)buf[i - 1]))) {
      c = vgetc();
      buf[i++] = (char)c;
    }

    do_digraph(-1);                     // clear digraphs
    do_digraph((uint8_t)buf[i - 1]);               // may be the start of a digraph
    buf[i] = NUL;
    ins_str(buf, (size_t)i);
    if (flags & INSCHAR_CTRLV) {
      redo_literal((uint8_t)(*buf));
      i = 1;
    } else {
      i = 0;
    }
    if (buf[i] != NUL) {
      AppendToRedobuffLit(buf + i, -1);
    }
  } else {
    int cc;

    if ((cc = utf_char2len(c)) > 1) {
      char buf[MB_MAXCHAR + 1];

      utf_char2bytes(c, buf);
      buf[cc] = NUL;
      ins_char_bytes(buf, (size_t)cc);
      AppendCharToRedobuff(c);
    } else {
      ins_char(c);
      if (flags & INSCHAR_CTRLV) {
        redo_literal(c);
      } else {
        AppendCharToRedobuff(c);
      }
    }
  }
}

// Put a character in the redo buffer, for when just after a CTRL-V.
static void redo_literal(int c)
{
  rs_redo_literal(c);
}

/// start_arrow() is called when an arrow key is used in insert mode.
/// For undo/redo it resembles hitting the <ESC> key.
///
/// @param end_insert_pos  can be NULL
void start_arrow(pos_T *end_insert_pos)
{
  rs_start_arrow(end_insert_pos);
}

/// Like start_arrow() but with end_change argument.
/// Will prepare for redo of CTRL-G U if "end_change" is false.
///
/// @param end_insert_pos  can be NULL
/// @param end_change      end undoable change
static void start_arrow_with_change(pos_T *end_insert_pos, bool end_change)
{
  rs_start_arrow_with_change(end_insert_pos, end_change ? 1 : 0);
}

// If we skipped highlighting word at cursor, do it now.
// It may be skipped again, thus reset spell_redraw_lnum first.
static void check_spell_redraw(void)
{
  rs_check_spell_redraw();
}

// stop_arrow() is called before a change is made in insert mode.
// If an arrow key has been used, start a new insertion.
// Returns FAIL if undo is impossible, shouldn't insert then.
int stop_arrow(void)
{
  return rs_stop_arrow();
}

/// Do a few things to stop inserting.
/// "end_insert_pos" is where insert ended.  It is NULL when we already jumped
/// to another window/buffer.
///
/// @param esc     called by ins_esc()
/// @param nomove  <c-\><c-o>, don't move cursor
static void stop_insert(pos_T *end_insert_pos, int esc, int nomove)
{
  stop_redo_ins();
  rs_replace_stack_clear();  // abandon replace stack

  // Save the inserted text for later redo with ^@ and CTRL-A.
  // Don't do it when "restart_edit" was set and nothing was inserted,
  // otherwise CTRL-O w and then <Left> will clear "last_insert".
  String inserted = get_inserted();
  int added = inserted.data == NULL ? 0 : (int)inserted.size - new_insert_skip;
  if (did_restart_edit == 0 || added > 0) {
    xfree(last_insert.data);
    last_insert = inserted;  // structure copy
    last_insert_skip = added < 0 ? 0 : new_insert_skip;
  } else {
    xfree(inserted.data);
  }

  if (!arrow_used && end_insert_pos != NULL) {
    int cc;
    // Auto-format now.  It may seem strange to do this when stopping an
    // insertion (or moving the cursor), but it's required when appending
    // a line and having it end in a space.  But only do it when something
    // was actually inserted, otherwise undo won't work.
    if (!ins_need_undo && has_format_option(FO_AUTO)) {
      pos_T tpos = curwin->w_cursor;

      // When the cursor is at the end of the line after a space the
      // formatting will move it to the following word.  Avoid that by
      // moving the cursor onto the space.
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
        // If the cursor is still at the same character, also keep
        // the "coladd".
        if (gchar_cursor() == NUL
            && curwin->w_cursor.lnum == tpos.lnum
            && curwin->w_cursor.col == tpos.col) {
          curwin->w_cursor.coladd = tpos.coladd;
        }
      }
    }

    // If a space was inserted for auto-formatting, remove it now.
    check_auto_format(true);

    // If we just did an auto-indent, remove the white space from the end
    // of the line, and put the cursor back.
    // Do this when ESC was used or moving the cursor up/down.
    // Check for the old position still being valid, just in case the text
    // got changed unexpectedly.
    if (!nomove && did_ai && (esc || (vim_strchr(p_cpo, CPO_INDENT) == NULL
                                      && curwin->w_cursor.lnum !=
                                      end_insert_pos->lnum))
        && end_insert_pos->lnum <= curbuf->b_ml.ml_line_count) {
      pos_T tpos = curwin->w_cursor;
      colnr_T prev_col = end_insert_pos->col;

      curwin->w_cursor = *end_insert_pos;
      check_cursor_col(curwin);        // make sure it is not past the line
      while (true) {
        if (gchar_cursor() == NUL && curwin->w_cursor.col > 0) {
          curwin->w_cursor.col--;
        }
        cc = gchar_cursor();
        if (!ascii_iswhite(cc)) {
          break;
        }
        if (del_char(true) == FAIL) {
          break;            // should not happen
        }
      }
      if (curwin->w_cursor.lnum != tpos.lnum) {
        curwin->w_cursor = tpos;
      } else if (curwin->w_cursor.col < prev_col) {
        // reset tpos, could have been invalidated in the loop above
        tpos = curwin->w_cursor;
        tpos.col++;
        if (cc != NUL && gchar_pos(&tpos) == NUL) {
          curwin->w_cursor.col++;         // put cursor back on the NUL
        }
      }

      // <C-S-Right> may have started Visual mode, adjust the position for
      // deleted characters.
      if (VIsual_active) {
        check_visual_pos();
      }
    }
  }
  did_ai = false;
  did_si = false;
  can_si = false;
  can_si_back = false;

  // Set '[ and '] to the inserted text.  When end_insert_pos is NULL we are
  // now in a different buffer.
  if (end_insert_pos != NULL) {
    curbuf->b_op_start = Insstart;
    curbuf->b_op_start_orig = Insstart_orig;
    curbuf->b_op_end = *end_insert_pos;
  }
}

// Set the last inserted text to a single character.
// Used for the replace command.
void set_last_insert(int c)
{
  rs_set_last_insert(c);
}

#if defined(EXITFREE)
void free_last_insert(void)
{
  rs_free_last_insert();
}
#endif

// move cursor to start of line
// if flags & BL_WHITE  move to first non-white
// if flags & BL_SOL    move to first non-white if startofline is set,
//                          otherwise keep "curswant" column
// if flags & BL_FIX    don't leave the cursor on a NUL.
void beginline(int flags)
{
  rs_beginline(flags);
}

// oneright oneleft cursor_down cursor_up
//
// Move one char {right,left,down,up}.
// Doesn't move onto the NUL past the end of the line, unless it is allowed.
// Return OK when successful, FAIL when we hit a line of file boundary.

int oneright(void)
{
  return rs_oneright();
}

int oneleft(void)
{
  return rs_oneleft();
}

/// Move the cursor up "n" lines in window "wp". Takes care of closed folds.
/// Skips over concealed lines when "skip_conceal" is true.
void cursor_up_inner(win_T *wp, linenr_T n, bool skip_conceal)
{
  rs_cursor_up_inner(wp, n, skip_conceal);
}

/// @param upd_topline  When true: update topline
int cursor_up(linenr_T n, bool upd_topline)
{
  return rs_cursor_up(n, upd_topline ? 1 : 0);
}

/// Move the cursor down "n" lines in window "wp". Takes care of closed folds.
/// Skips over concealed lines when "skip_conceal" is true.
void cursor_down_inner(win_T *wp, int n, bool skip_conceal)
{
  rs_cursor_down_inner(wp, n, skip_conceal);
}

/// @param upd_topline  When true: update topline
int cursor_down(int n, bool upd_topline)
{
  return rs_cursor_down(n, upd_topline ? 1 : 0);
}

/// Stuff the last inserted text in the read buffer.
/// Last_insert actually is a copy of the redo buffer, so we
/// first have to remove the command.
///
/// @param c       Command character to be inserted
/// @param count   Repeat this many times
/// @param no_esc  Don't add an ESC at the end
int stuff_inserted(int c, int count, int no_esc)
{
  return rs_stuff_inserted(c, count, no_esc);
}

String get_last_insert(void)
  FUNC_ATTR_PURE
{
  RsNvimString rs = rs_get_last_insert();
  return rs.data == NULL ? NULL_STRING : (String){ .data = rs.data, .size = rs.size };
}

// Get last inserted string, and remove trailing <Esc>.
// Returns pointer to allocated memory (must be freed) or NULL.
char *get_last_insert_save(void)
{
  return rs_get_last_insert_save();
}

/// Check the word in front of the cursor for an abbreviation.
/// Called when the non-id character "c" has been entered.
/// When an abbreviation is recognized it is removed from the text and
/// the replacement string is inserted in typebuf.tb_buf[], followed by "c".
///
/// @param  c  character
///
/// @return true if the word is a known abbreviation.
static bool echeck_abbr(int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_echeck_abbr(c) != 0;
}

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
///
/// @param str character that is replaced (NUL is none)
/// @param len length of character in bytes
void replace_push(char *str, size_t len)
{
  rs_replace_push(str, len);
}

void replace_push_nul(void)
{
  rs_replace_push_nul();
}

static int replace_pop_if_nul(void)
{
  return rs_replace_pop_if_nul();
}

void replace_join(int off)
{
  rs_replace_join(off);
}

static void replace_pop_ins(void)
{
  rs_replace_pop_ins();
}

static void mb_replace_pop_ins(void)
{
  rs_mb_replace_pop_ins();
}

static void replace_do_bs(int limit_col)
{
  rs_replace_do_bs(limit_col);
}

static void ins_reg(void)
{
  bool need_redraw = false;
  int literally = 0;
  int vis_active = VIsual_active;

  // If we are going to wait for a character, show a '"'.
  pc_status = PC_STATUS_UNSET;
  if (redrawing() && !char_avail()) {
    // may need to redraw when no more chars available now
    ins_redraw(false);

    edit_putchar('"', true);
    add_to_showcmd_c(Ctrl_R);
  }

  // Don't map the register name. This also prevents the mode message to be
  // deleted when ESC is hit.
  no_mapping++;
  allow_keys++;
  int regname = plain_vgetc();
  LANGMAP_ADJUST(regname, true);
  if (regname == Ctrl_R || regname == Ctrl_O || regname == Ctrl_P) {
    // Get a third key for literal register insertion
    literally = regname;
    add_to_showcmd_c(literally);
    regname = plain_vgetc();
    LANGMAP_ADJUST(regname, true);
  }
  no_mapping--;
  allow_keys--;

  // Don't call u_sync() while typing the expression or giving an error
  // message for it. Only call it explicitly.
  no_u_sync++;
  if (regname == '=') {
    pos_T curpos = curwin->w_cursor;

    // Sync undo when evaluating the expression calls setline() or
    // append(), so that it can be undone separately.
    u_sync_once = 2;

    regname = get_expr_register();

    // Cursor may be moved back a column.
    curwin->w_cursor = curpos;
    check_cursor(curwin);
  }
  if (regname == NUL || !valid_yank_reg(regname, false)) {
    vim_beep(kOptBoFlagRegister);
    need_redraw = true;  // remove the '"'
  } else {
    yankreg_T *reg = get_yank_register(regname, YREG_PASTE);

    if (literally == Ctrl_O || literally == Ctrl_P) {
      // Append the command to the redo buffer.
      AppendCharToRedobuff(Ctrl_R);
      AppendCharToRedobuff(literally);
      AppendCharToRedobuff(regname);

      do_put(regname, NULL, BACKWARD, 1,
             (literally == Ctrl_P ? PUT_FIXINDENT : 0) | PUT_CURSEND);
    } else if (reg->y_size > 1 && is_literal_register(regname)) {
      AppendCharToRedobuff(Ctrl_R);
      AppendCharToRedobuff(regname);
      do_put(regname, NULL, BACKWARD, 1, PUT_CURSEND);
    } else if (insert_reg(regname, NULL, !!literally) == FAIL) {
      vim_beep(kOptBoFlagRegister);
      need_redraw = true;  // remove the '"'
    } else if (stop_insert_mode) {
      // When the '=' register was used and a function was invoked that
      // did ":stopinsert" then stuff_empty() returns false but we won't
      // insert anything, need to remove the '"'
      need_redraw = true;
    }
  }
  no_u_sync--;
  if (u_sync_once == 1) {
    ins_need_undo = true;
  }
  u_sync_once = 0;

  // If the inserted register is empty, we need to remove the '"'. Do this before
  // clearing showcmd, which emits an event that can also update the screen.
  if (need_redraw || stuff_empty()) {
    edit_unputchar();
  }
  rs_clear_showcmd();

  // Disallow starting Visual mode here, would get a weird mode.
  if (!vis_active && VIsual_active) {
    end_visual_mode();
  }
}

// CTRL-G commands in Insert mode.
static void ins_ctrl_g(void)
{
  rs_ins_ctrl_g();
}

// CTRL-^ in Insert mode.
static void ins_ctrl_hat(void)
{
  rs_ins_ctrl_hat();
}

/// Handle ESC in insert mode.
///
/// @param[in,out]  count    repeat count of the insert command
/// @param          cmdchar  command that started the insert
/// @param          nomove   when true, don't move the cursor
///
/// @return true when leaving insert mode, false when repeating the insert.
static bool ins_esc(int *count, int cmdchar, bool nomove)
  FUNC_ATTR_NONNULL_ARG(1)
{
  static bool disabled_redraw = false;

  check_spell_redraw();

  int temp = curwin->w_cursor.col;
  if (disabled_redraw) {
    RedrawingDisabled--;
    disabled_redraw = false;
  }
  if (!arrow_used) {
    // Don't append the ESC for "r<CR>" and "grx".
    if (cmdchar != 'r' && cmdchar != 'v') {
      AppendToRedobuff(ESC_STR);
    }

    // Repeating insert may take a long time.  Check for
    // interrupt now and then.
    if (*count > 0) {
      line_breakcheck();
      if (got_int) {
        *count = 0;
      }
    }

    if (--*count > 0) {         // repeat what was typed
      // Vi repeats the insert without replacing characters.
      if (vim_strchr(p_cpo, CPO_REPLCNT) != NULL) {
        State &= ~REPLACE_FLAG;
      }

      start_redo_ins();
      if (cmdchar == 'r' || cmdchar == 'v') {
        stuffRedoReadbuff(ESC_STR);  // No ESC in redo buffer
      }
      RedrawingDisabled++;
      disabled_redraw = true;
      // Repeat the insert
      return false;
    }
    stop_insert(&curwin->w_cursor, true, nomove);
    undisplay_dollar();
  }

  if (cmdchar != 'r' && cmdchar != 'v') {
    ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
  }

  // When an autoindent was removed, curswant stays after the
  // indent
  if (restart_edit == NUL && (colnr_T)temp == curwin->w_cursor.col) {
    curwin->w_set_curswant = true;
  }

  // Remember the last Insert position in the '^ mark.
  if ((cmdmod.cmod_flags & CMOD_KEEPJUMPS) == 0) {
    fmarkv_T view = mark_view_make(curwin->w_topline, curwin->w_cursor);
    RESET_FMARK(&curbuf->b_last_insert, curwin->w_cursor, curbuf->b_fnum, view);
  }

  // The cursor should end up on the last inserted character.
  // Don't do it for CTRL-O, unless past the end of the line.
  if (!nomove
      && (curwin->w_cursor.col != 0 || curwin->w_cursor.coladd > 0)
      && (restart_edit == NUL || (gchar_cursor() == NUL && !VIsual_active))
      && !revins_on) {
    if (curwin->w_cursor.coladd > 0 || get_ve_flags(curwin) == kOptVeFlagAll) {
      oneleft();
      if (restart_edit != NUL) {
        curwin->w_cursor.coladd++;
      }
    } else {
      curwin->w_cursor.col--;
      curwin->w_valid &= ~(VALID_WCOL|VALID_VIRTCOL);
      // Correct cursor for multi-byte character.
      mb_adjust_cursor();
    }
  }

  State = MODE_NORMAL;
  may_trigger_modechanged();
  // need to position cursor again when on a TAB and
  // when on a char with inline virtual text
  if (gchar_cursor() == TAB || buf_meta_total(curbuf, kMTMetaInline) > 0) {
    curwin->w_valid &= ~(VALID_WROW|VALID_WCOL|VALID_VIRTCOL);
  }

  setmouse();
  ui_cursor_shape();            // may show different cursor shape

  // When recording or for CTRL-O, need to display the new mode.
  // Otherwise remove the mode message.
  if (reg_recording != 0 || restart_edit != NUL) {
    showmode();
  } else if (p_smd && (got_int || !skip_showmode())
             && !(p_ch == 0 && !ui_has(kUIMessages))) {
    unshowmode(false);
  }
  // Exit Insert mode
  return true;
}

// Toggle language: revins_on.
// Move to end of reverse inserted text.
static void ins_ctrl_(void)
{
  rs_ins_ctrl_();
}

/// If 'keymodel' contains "startsel", may start selection.
///
/// @param  c  character to check
//
/// @return true when a CTRL-O and other keys stuffed.
static bool ins_start_select(int c)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_ins_start_select(c) != 0;
}

// <Insert> key in Insert mode: toggle insert/replace mode.
static void ins_insert(int replaceState)
{
  rs_ins_insert(replaceState);
}

// Pressed CTRL-O in Insert mode.
static void ins_ctrl_o(void)
{
  rs_ins_ctrl_o();
}

// If the cursor is on an indent, ^T/^D insert/delete one
// shiftwidth.  Otherwise ^T/^D behave like a "<<" or ">>".
// Always round the indent to 'shiftwidth', this is compatible
// with vi.  But vi only supports ^T and ^D after an
// autoindent, we support it everywhere.
static void ins_shift(int c, int lastc)
{
  rs_ins_shift(c, lastc);
}

static void ins_del(void)
{
  rs_ins_del();
}

/// Handle Backspace, delete-word and delete-line in Insert mode.
///
/// @param          c                 character that was typed
/// @param          mode              backspace mode to use
/// @param[in,out]  inserted_space_p  whether a space was the last
//                                    character inserted
///
/// @return true when backspace was actually used.
static bool ins_bs(int c, int mode, int *inserted_space_p)
  FUNC_ATTR_NONNULL_ARG(3)
{
  int cc;
  int temp = 0;                     // init for GCC
  bool did_backspace = false;
  bool call_fix_indent = false;

  // can't delete anything in an empty file
  // can't backup past first character in buffer
  // can't backup past starting point unless 'backspace' > 1
  // can backup to a previous line if 'backspace' == 0
  if (buf_is_empty(curbuf)
      || (!revins_on
          && ((curwin->w_cursor.lnum == 1 && curwin->w_cursor.col == 0)
              || (!can_bs(BS_START)
                  && ((arrow_used && !bt_prompt(curbuf))
                      || (curwin->w_cursor.lnum == Insstart_orig.lnum
                          && curwin->w_cursor.col <= Insstart_orig.col)))
              || (!can_bs(BS_INDENT) && !arrow_used && ai_col > 0
                  && curwin->w_cursor.col <= ai_col)
              || (!can_bs(BS_EOL) && curwin->w_cursor.col == 0)))) {
    vim_beep(kOptBoFlagBackspace);
    return false;
  }

  if (stop_arrow() == FAIL) {
    return false;
  }
  bool in_indent = inindent(0);
  if (in_indent) {
    can_cindent = false;
  }
  end_comment_pending = NUL;  // After BS, don't auto-end comment
  if (revins_on) {            // put cursor after last inserted char
    inc_cursor();
  }
  // Virtualedit:
  //    BACKSPACE_CHAR eats a virtual space
  //    BACKSPACE_WORD eats all coladd
  //    BACKSPACE_LINE eats all coladd and keeps going
  if (curwin->w_cursor.coladd > 0) {
    if (mode == BACKSPACE_CHAR) {
      curwin->w_cursor.coladd--;
      return true;
    }
    if (mode == BACKSPACE_WORD) {
      curwin->w_cursor.coladd = 0;
      return true;
    }
    curwin->w_cursor.coladd = 0;
  }

  // Delete newline!
  if (curwin->w_cursor.col == 0) {
    linenr_T lnum = Insstart.lnum;
    if (curwin->w_cursor.lnum == lnum || revins_on) {
      if (u_save((linenr_T)(curwin->w_cursor.lnum - 2),
                 (linenr_T)(curwin->w_cursor.lnum + 1)) == FAIL) {
        return false;
      }
      Insstart.lnum--;
      Insstart.col = ml_get_len(Insstart.lnum);
    }
    // In replace mode:
    // cc < 0: NL was inserted, delete it
    // cc >= 0: NL was replaced, put original characters back
    cc = -1;
    if (State & REPLACE_FLAG) {
      cc = replace_pop_if_nul();  // returns -1 if NL was inserted
    }
    // In replace mode, in the line we started replacing, we only move the
    // cursor.
    if ((State & REPLACE_FLAG) && curwin->w_cursor.lnum <= lnum) {
      dec_cursor();
    } else {
      if (!(State & VREPLACE_FLAG)
          || curwin->w_cursor.lnum > orig_line_count) {
        temp = gchar_cursor();          // remember current char
        curwin->w_cursor.lnum--;

        // When "aw" is in 'formatoptions' we must delete the space at
        // the end of the line, otherwise the line will be broken
        // again when auto-formatting.
        if (has_format_option(FO_AUTO)
            && has_format_option(FO_WHITE_PAR)) {
          char *ptr = ml_get_buf_mut(curbuf, curwin->w_cursor.lnum);
          int len = get_cursor_line_len();
          if (len > 0 && ptr[len - 1] == ' ') {
            ptr[len - 1] = NUL;
            curbuf->b_ml.ml_line_len--;
          }
        }

        do_join(2, false, false, false, false);
        if (temp == NUL && gchar_cursor() != NUL) {
          inc_cursor();
        }
      } else {
        dec_cursor();
      }

      // In MODE_REPLACE mode we have to put back the text that was
      // replaced by the NL. On the replace stack is first a
      // NUL-terminated sequence of characters that were deleted and then
      // the characters that NL replaced.
      if (State & REPLACE_FLAG) {
        // Do the next ins_char() in MODE_NORMAL state, to
        // prevent ins_char() from replacing characters and
        // avoiding showmatch().
        int oldState = State;
        State = MODE_NORMAL;
        // restore characters (blanks) deleted after cursor
        while (cc > 0) {
          colnr_T save_col = curwin->w_cursor.col;
          mb_replace_pop_ins();
          curwin->w_cursor.col = save_col;
          cc = replace_pop_if_nul();
        }
        // restore the characters that NL replaced
        replace_pop_ins();
        State = oldState;
      }
    }
    did_ai = false;
  } else {
    // Delete character(s) before the cursor.
    if (revins_on) {            // put cursor on last inserted char
      dec_cursor();
    }
    colnr_T mincol = 0;
    // keep indent
    if (mode == BACKSPACE_LINE
        && (curbuf->b_p_ai || cindent_on())
        && !revins_on) {
      colnr_T save_col = curwin->w_cursor.col;
      beginline(BL_WHITE);
      if (curwin->w_cursor.col < save_col) {
        mincol = curwin->w_cursor.col;
        // should now fix the indent to match with the previous line
        call_fix_indent = true;
      }
      curwin->w_cursor.col = save_col;
    }

    // Handle deleting one 'shiftwidth' or 'softtabstop'.
    if (mode == BACKSPACE_CHAR
        && ((p_sta && in_indent)
            || ((get_sts_value() != 0 || tabstop_count(curbuf->b_p_vsts_array))
                && curwin->w_cursor.col > 0
                && (*(get_cursor_pos_ptr() - 1) == TAB
                    || (*(get_cursor_pos_ptr() - 1) == ' '
                        && (!*inserted_space_p || arrow_used)))))) {
      *inserted_space_p = false;

      bool const use_ts = !curwin->w_p_list || curwin->w_p_lcs_chars.tab1;
      char *const line = get_cursor_line_ptr();
      char *const cursor_ptr = line + curwin->w_cursor.col;

      colnr_T vcol = 0;
      colnr_T space_vcol = 0;
      StrCharInfo sci = utf_ptr2StrCharInfo(line);
      StrCharInfo space_sci = sci;
      bool prev_space = false;

      // Compute virtual column of cursor position, and find the last
      // whitespace before cursor that is preceded by non-whitespace.
      // Use charsize_nowrap() so that virtual text and wrapping are ignored.
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

      // Find the position to stop backspacing.
      // Use charsize_nowrap() so that virtual text and wrapping are ignored.
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
          // Don't delete characters before the insert point when in Replace mode.
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
        // Remember the first char we inserted.
        if (curwin->w_cursor.lnum == Insstart_orig.lnum
            && curwin->w_cursor.col < Insstart_orig.col) {
          Insstart_orig.col = curwin->w_cursor.col;
        }

        if (State & VREPLACE_FLAG) {
          ins_char(' ');
        } else {
          ins_str(S_LEN(" "));
          if ((State & REPLACE_FLAG)) {
            replace_push_nul();
          }
        }
      }
    } else {
      // Delete up to starting point, start of line or previous word.

      int cclass = mb_get_class(get_cursor_pos_ptr());
      do {
        if (!revins_on) {   // put cursor on char to be deleted
          dec_cursor();
        }
        cc = gchar_cursor();
        // look multi-byte character class
        int prev_cclass = cclass;
        cclass = mb_get_class(get_cursor_pos_ptr());
        if (mode == BACKSPACE_WORD && !ascii_isspace(cc)) {   // start of word?
          mode = BACKSPACE_WORD_NOT_SPACE;
          temp = vim_iswordc(cc);
        } else if (mode == BACKSPACE_WORD_NOT_SPACE
                   && ((ascii_isspace(cc) || vim_iswordc(cc) != temp)
                       || prev_cclass != cclass)) {   // end of word?
          if (!revins_on) {
            inc_cursor();
          } else if (State & REPLACE_FLAG) {
            dec_cursor();
          }
          break;
        }
        if (State & REPLACE_FLAG) {
          replace_do_bs(-1);
        } else {
          bool has_composing = false;
          if (p_deco) {
            char *p0 = get_cursor_pos_ptr();
            has_composing = utf_composinglike(p0, p0 + utf_ptr2len(p0), NULL);
          }
          del_char(false);
          // If there are combining characters and 'delcombine' is set
          // move the cursor back.  Don't back up before the base character.
          if (has_composing) {
            inc_cursor();
          }
          if (revins_chars) {
            revins_chars--;
            revins_legal++;
          }
          if (revins_on && gchar_cursor() == NUL) {
            break;
          }
        }
        // Just a single backspace?:
        if (mode == BACKSPACE_CHAR) {
          break;
        }
      } while (revins_on
               || (curwin->w_cursor.col > mincol
                   && (can_bs(BS_NOSTOP)
                       || (curwin->w_cursor.lnum != Insstart_orig.lnum
                           || curwin->w_cursor.col != Insstart_orig.col))));
    }
    did_backspace = true;
  }
  did_si = false;
  can_si = false;
  can_si_back = false;
  if (curwin->w_cursor.col <= 1) {
    did_ai = false;
  }

  if (call_fix_indent) {
    fix_indent();
  }

  // It's a little strange to put backspaces into the redo
  // buffer, but it makes auto-indent a lot easier to deal
  // with.
  AppendCharToRedobuff(c);

  // If deleted before the insertion point, adjust it
  if (curwin->w_cursor.lnum == Insstart_orig.lnum
      && curwin->w_cursor.col < Insstart_orig.col) {
    Insstart_orig.col = curwin->w_cursor.col;
  }

  // vi behaviour: the cursor moves backward but the character that
  //               was there remains visible
  // Vim behaviour: the cursor moves backward and the character that
  //                was there is erased from the screen.
  // We can emulate the vi behaviour by pretending there is a dollar
  // displayed even when there isn't.
  //  --pkv Sun Jan 19 01:56:40 EST 2003
  if (vim_strchr(p_cpo, CPO_BACKSPACE) != NULL && dollar_vcol == -1) {
    dollar_vcol = curwin->w_virtcol;
  }

  // When deleting a char the cursor line must never be in a closed fold.
  // E.g., when 'foldmethod' is indent and deleting the first non-white
  // char before a Tab.
  if (did_backspace) {
    rs_foldOpenCursor();
  }
  return did_backspace;
}

static void ins_left(void)
{
  rs_ins_left();
}

static void ins_home(int c)
{
  rs_ins_home(c);
}

static void ins_end(int c)
{
  rs_ins_end(c);
}

static void ins_s_left(void)
{
  rs_ins_s_left();
}

static void ins_right(void)
{
  rs_ins_right();
}

static void ins_s_right(void)
{
  rs_ins_s_right();
}

/// @param startcol  when true move to Insstart.col
static void ins_up(bool startcol)
{
  rs_ins_up(startcol);
}

static void ins_pageup(void)
{
  rs_ins_pageup();
}

/// @param startcol  when true move to Insstart.col
static void ins_down(bool startcol)
{
  rs_ins_down(startcol);
}

static void ins_pagedown(void)
{
  rs_ins_pagedown();
}

// ins_tab is now implemented in Rust (src/nvim-rs/edit/src/tab.rs).
// ins_eol delegates to Rust rs_ins_eol (symbol exported directly).

// Handle digraph in insert mode.
// Returns character still to be inserted, or NUL when nothing remaining to be
// done.
static int ins_digraph(void)
{
  return rs_ins_digraph();
}

// Handle CTRL-E and CTRL-Y in Insert mode: copy char from other line.
// Returns the char to be inserted, or NUL if none found.
int ins_copychar(linenr_T lnum)
{
  return rs_ins_copychar(lnum);
}

// CTRL-Y or CTRL-E typed in Insert mode.
static int ins_ctrl_ey(int tc)
{
  return rs_ins_ctrl_ey(tc);
}

// Get the value that w_virtcol would have when 'list' is off.
// Unless 'cpo' contains the 'L' flag.
colnr_T get_nolist_virtcol(void)
{
  return rs_get_nolist_virtcol();
}

/// Get virtual column without list mode (accessor for Rust).
int nvim_get_nolist_virtcol(void)
{
  return (int)rs_get_nolist_virtcol();
}

// Handle the InsertCharPre autocommand.
// "c" is the character that was typed.
// Return a pointer to allocated memory with the replacement string.
// Return NULL to continue inserting "c".
static char *do_insert_char_pre(int c)
{
  return rs_do_insert_char_pre(c);
}

bool get_can_cindent(void)
{
  return rs_get_can_cindent() != 0;
}

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

/// Wrapper for cursor_up() (accessor for Rust).
/// Operates on curwin.
int nvim_scroll_cursor_up(long n, int upd_topline)
{
  return cursor_up((linenr_T)n, upd_topline != 0);
}

/// Wrapper for cursor_down() (accessor for Rust).
/// Operates on curwin.
int nvim_scroll_cursor_down(int n, int upd_topline)
{
  return cursor_down(n, upd_topline != 0);
}

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
