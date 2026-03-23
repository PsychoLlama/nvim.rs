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
extern int rs_ctrl_x_mode_scroll(void);
extern int rs_ins_compl_active(void);
extern int rs_ins_compl_col(void);

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
extern void insert_enter(InsertState *s);

#include "edit.c.generated.h"

extern int rs_get_scrolloff_value(win_T *wp);

// Rust fold FFI declarations
extern void rs_foldOpenCursor(void);

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

extern int insert_check_rs(VimState *state);
extern int insert_execute_rs(VimState *state, int key);

// NOTE: ins_esc returns int (not bool) to match Rust c_int ABI.

extern void insert_special(int c, int allow_modmask, int ctrlv);
extern void start_arrow_with_change(pos_T *end_insert_pos, bool end_change);
extern int echeck_abbr(int c);
extern void replace_join(int off);
extern void replace_pop_ins(void);
extern void replace_do_bs(int limit_col);
extern void ins_ctrl_o(void);
extern int ins_start_select(int c);
extern char *do_insert_char_pre(int c);
extern void undisplay_dollar(void);
extern void backspace_until_column(int col);
extern int get_literal(bool no_simplify);
extern void start_arrow(pos_T *end_insert_pos);
extern int stop_arrow(void);
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

/// Initialize Insstart_textlen and Insstart_blank_vcol (accessor for Rust).
void nvim_edit_init_Insstart_textlen(void)
{
  Insstart_textlen = linetabsize_str(get_cursor_line_ptr());
  Insstart_blank_vcol = MAXCOL;
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

/// Set compl_busy (accessor for Rust).
void nvim_set_compl_busy(bool val)
{
  compl_busy = val;
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

/// Get replace_offset global (accessor for Rust).
int nvim_get_replace_offset(void)
{
  return replace_offset;
}

const void *nvim_curwin_get_cursor_ptr(void)
{
  return &curwin->w_cursor;
}

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

// Saved cursor positions for start_arrow calls (2 slots)
static pos_T edit_saved_cursor[2];
static linenr_T saved_topline;
static int saved_topfill;

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

/// Note: nvim_get_restart_edit is defined in cursor_shape.c; use this wrapper.

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

/// Set revins_on (accessor for Rust).
void nvim_edit_set_revins_on(int val)
{
  revins_on = (val != 0);
}

/// Update curbuf->b_last_changedtick if TextChangedI was triggered (accessor for Rust).
void nvim_curbuf_sync_changedtick_after_insert(void)
{
  if (!char_avail() && curbuf->b_last_changedtick_i == buf_get_changedtick(curbuf)) {
    curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  }
}

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

/// Set Insstart_orig to Insstart (accessor for Rust state_machine).
void nvim_set_Insstart_orig_from_Insstart(void)
{
  Insstart_orig = Insstart;
}

/// Call stuffcharReadbuff(K_NOP) (accessor for Rust state_machine).
void nvim_stuffcharReadbuff_K_NOP(void)
{
  stuffcharReadbuff(K_NOP);
}


/// Set did_cursorhold (accessor for Rust state_machine, to avoid duplicate).

/// Call ins_redraw(false) (accessor for Rust state_machine).
void ins_redraw_false(void)
{
  nvim_edit_ins_redraw_impl(false);
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

long nvim_curbuf_tabstop_first_vts(void)
{
  return (long)tabstop_first(curbuf->b_p_vts_array);
}

long nvim_curbuf_get_sw_value(void)
{
  return get_sw_value(curbuf);
}

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

/// Trim last char of previous line if space (FO_WHITE_PAR helper).
void nvim_trim_eol_space(void)
{
  char *ptr = ml_get_buf_mut(curbuf, curwin->w_cursor.lnum);
  int len = get_cursor_line_len();
  if (len > 0 && ptr[len - 1] == ' ') {
    ptr[len - 1] = NUL;
    curbuf->b_ml.ml_line_len--;
  }
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

/// Call ins_apply_autocmds(EVENT_INSERTLEAVEPRE) (accessor for Rust).
void nvim_ins_apply_autocmds_insertleavepre(void)
{
  ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
}

/// Call unshowmode(false) (accessor for Rust).
void nvim_unshowmode_false(void)
{
  unshowmode(false);
}

/// Calls mark_view_make and RESET_FMARK.
void nvim_set_b_last_insert_mark(void)
{
  fmarkv_T view = mark_view_make(curwin->w_topline, curwin->w_cursor);
  RESET_FMARK(&curbuf->b_last_insert, curwin->w_cursor, curbuf->b_fnum, view);
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

/// Set pc_status = PC_STATUS_UNSET (accessor for Rust).
void nvim_set_pc_status_unset(void)
{
  pc_status = PC_STATUS_UNSET;
}

/// Call edit_putchar(c, highlight != 0) (accessor for Rust).
void nvim_putchar(int c, int highlight)
{
  edit_putchar(c, highlight != 0);
}

// ---- ins_esc accessors ----

/// Call stop_insert logic at curwin->w_cursor (composite for Rust).
void nvim_stop_insert_curpos(int nomove)
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

/// Get curwin->w_cursor.coladd (accessor for Rust).
colnr_T nvim_curwin_get_cursor_coladd(void)
{
  return curwin->w_cursor.coladd;
}

// ---- ins_reg accessors ----

/// Save cursor position for expression register evaluation (composite for Rust).
static pos_T ins_reg_saved_cursor;
void nvim_ins_reg_restore_cursor_save(void)
{
  ins_reg_saved_cursor = curwin->w_cursor;
}

/// Restore cursor position after expression register evaluation (composite for Rust).
void nvim_ins_reg_restore_cursor(void)
{
  curwin->w_cursor = ins_reg_saved_cursor;
  check_cursor(curwin);
}

