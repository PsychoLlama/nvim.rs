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
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/indent.h"
#include "nvim/insexpand.h"
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

// Rust fold FFI declarations
extern void rs_foldOpenCursor(void);

enum {
  BACKSPACE_CHAR = 1,
  BACKSPACE_WORD = 2,
  BACKSPACE_WORD_NOT_SPACE = 3,
  BACKSPACE_LINE = 4,
};

/// the text of the previous insert, K_SPECIAL is escaped
static String last_insert = STRING_INIT;

typedef struct {
  char *data;
  size_t size;
} RsNvimString;

// Rust FFI declarations (only functions called directly in this file)
extern RsNvimString rs_get_last_insert(void);
extern void rs_replace_stack_clear(void);
extern void ins_ctrl_v(void);
extern void rs_clear_showcmd(void);
extern int insert_check_rs(VimState *state);
extern int insert_execute_rs(VimState *state, int key);

// NOTE: ins_esc returns int (not bool) to match Rust c_int ABI.

extern void insert_special(int c, int allow_modmask, int ctrlv);
extern void start_arrow_with_change(pos_T *end_insert_pos, bool end_change);
extern int echeck_abbr(int c);
extern void replace_join(int off);
extern void replace_pop_ins(void);
extern void replace_do_bs(int limit_col);
extern char *do_insert_char_pre(int c);

// Accessors for Rust-owned statics (defined in globals.rs)
extern int nvim_get_ins_need_undo(void);
extern void nvim_set_ins_need_undo(int val);
extern int nvim_get_can_cindent(void);
extern void nvim_set_can_cindent(int val);
extern int nvim_get_revins_on(void);
extern void nvim_edit_set_revins_on(int val);
extern int nvim_get_did_restart_edit(void);
extern void nvim_set_did_restart_edit(int val);
extern int nvim_get_revins_chars(void);
extern void nvim_set_revins_chars(int val);
extern int nvim_get_revins_legal(void);
extern void nvim_set_revins_legal(int val);
extern int nvim_get_revins_scol(void);
extern void nvim_set_revins_scol(int val);
extern int nvim_get_compl_busy(void);
extern void nvim_set_compl_busy(bool val);
extern int nvim_get_last_insert_skip(void);
extern void nvim_set_last_insert_skip(int val);
extern int nvim_get_new_insert_skip(void);
extern void nvim_set_new_insert_skip(int val);
extern int nvim_get_dont_sync_undo(void);
extern void nvim_set_dont_sync_undo(int val);
extern int nvim_get_o_lnum(void);
extern void nvim_set_o_lnum(linenr_T val);
extern int nvim_get_update_Insstart_orig(void);
extern void nvim_set_update_Insstart_orig(int val);
extern colnr_T nvim_get_Insstart_textlen(void);
extern void nvim_set_Insstart_textlen(colnr_T val);
extern colnr_T nvim_get_Insstart_blank_vcol(void);
extern void nvim_set_Insstart_blank_vcol(colnr_T val);

int nvim_get_no_abbr(void) { return no_abbr; }

char *nvim_buf_get_b_prompt_text(const buf_T *buf) { return buf->b_prompt_text; }

linenr_T nvim_curwin_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }

colnr_T nvim_curwin_get_cursor_col(void) { return curwin->w_cursor.col; }

linenr_T nvim_curbuf_get_b_prompt_start_lnum(void) { return curbuf->b_prompt_start.mark.lnum; }

linenr_T nvim_get_Insstart_lnum(void) { return Insstart.lnum; }

colnr_T nvim_get_Insstart_col(void) { return Insstart.col; }

void nvim_set_Insstart(linenr_T lnum, colnr_T col)
{
  Insstart.lnum = lnum;
  Insstart.col = col;
}

linenr_T nvim_get_Insstart_orig_lnum(void) { return Insstart_orig.lnum; }

colnr_T nvim_get_Insstart_orig_col(void) { return Insstart_orig.col; }

void nvim_set_Insstart_orig(linenr_T lnum, colnr_T col)
{
  Insstart_orig.lnum = lnum;
  Insstart_orig.col = col;
}

int nvim_get_arrow_used(void) { return arrow_used; }

void nvim_set_arrow_used(int val) { arrow_used = val != 0; }

int nvim_get_p_ri(void) { return p_ri; }

int nvim_get_p_ari(void) { return p_ari; }

int nvim_curwin_buf_valid(void) { return curwin->w_buffer != NULL && curwin->w_buffer->b_ml.ml_mfp != NULL; }

linenr_T nvim_curwin_buf_line_count(void) { return curwin->w_buffer->b_ml.ml_line_count; }

int nvim_curwin_w_p_list(void) { return curwin->w_p_list; }

int nvim_p_cpo_has_listwm(void) { return vim_strchr(p_cpo, CPO_LISTWM) != NULL; }

colnr_T nvim_getvcol_nolist(void) { return getvcol_nolist(&curwin->w_cursor); }

/// Run validate_virtcol(curwin) (accessor for Rust).
void nvim_validate_virtcol_curwin(void) { validate_virtcol(curwin); }

colnr_T nvim_curwin_get_w_virtcol(void) { return curwin->w_virtcol; }

char *nvim_get_cursor_pos_ptr(void) { return get_cursor_pos_ptr(); }

char *nvim_get_last_insert_data(void) { return last_insert.data; }

size_t nvim_get_last_insert_size(void) { return last_insert.size; }

void nvim_set_last_insert(char *data, size_t size)
{
  last_insert.data = data;
  last_insert.size = size;
}

void nvim_clear_last_insert(void) { API_CLEAR_STRING(last_insert); }

int nvim_get_replace_offset(void) { return replace_offset; }

const void *nvim_curwin_get_cursor_ptr(void) { return &curwin->w_cursor; }

/// Update curbuf->b_last_changedtick if TextChangedI was triggered (accessor for Rust).
void nvim_curbuf_sync_changedtick_after_insert(void)
{
  if (!char_avail() && curbuf->b_last_changedtick_i == buf_get_changedtick(curbuf)) {
    curbuf->b_last_changedtick = buf_get_changedtick(curbuf);
  }
}

// nvim_edit_handle_restart_edit_cursor is implemented in Rust (enter.rs).
extern int nvim_edit_handle_restart_edit_cursor(void);

/// Set Insstart_orig to Insstart (accessor for Rust state_machine).
void nvim_set_Insstart_orig_from_Insstart(void) { Insstart_orig = Insstart; }

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
    nvim_set_Insstart_textlen(Insstart.col);
    nvim_set_Insstart_blank_vcol(MAXCOL);
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

// edit() is implemented in Rust (src/nvim-rs/edit/src/enter.rs).
// nvim_edit_edit_entry provides the InsertState setup entry point.

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
  if (textlock != 0 || rs_ins_compl_active() || nvim_get_compl_busy() || pum_visible()
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

// edit_putchar and edit_unputchar are implemented in Rust (putchar.rs).
// nvim_set_pc_status_unset is also implemented in Rust (putchar.rs).
extern void edit_putchar(int c, bool highlight);
extern void edit_unputchar(void);
// Rust symbol: nvim_set_pc_status_unset (resets pc_status static in Rust).

// display_dollar is implemented in Rust (putchar.rs).
extern void display_dollar(colnr_T col_arg);


void set_can_cindent(bool val) { nvim_set_can_cindent(val ? 1 : 0); }

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

colnr_T nvim_get_dollar_vcol(void) { return dollar_vcol; }

void nvim_set_dollar_vcol(colnr_T val) { dollar_vcol = val; }

long nvim_curbuf_get_b_p_sts(void) { return (long)curbuf->b_p_sts; }

int nvim_curbuf_tabstop_count_vts(void) { return (int)tabstop_count(curbuf->b_p_vts_array); }

int nvim_curbuf_tabstop_count_vsts(void) { return (int)tabstop_count(curbuf->b_p_vsts_array); }

long nvim_curbuf_tabstop_first_vts(void) { return (long)tabstop_first(curbuf->b_p_vts_array); }

long nvim_curbuf_get_sw_value(void) { return get_sw_value(curbuf); }

long nvim_get_sts_value(void) { return get_sts_value(); }

int nvim_curbuf_tabstop_padding_sts(void)
{
  return tabstop_padding(get_nolist_virtcol(), get_sts_value(), curbuf->b_p_vsts_array);
}

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

// nvim_trim_eol_space is implemented in Rust (helpers.rs).
extern void nvim_trim_eol_space(void);

/// Call ins_apply_autocmds(EVENT_INSERTLEAVEPRE) (accessor for Rust).
void nvim_ins_apply_autocmds_insertleavepre(void) { ins_apply_autocmds(EVENT_INSERTLEAVEPRE); }

/// Call unshowmode(false) (accessor for Rust).
void nvim_unshowmode_false(void) { unshowmode(false); }

/// Calls mark_view_make and RESET_FMARK.
void nvim_set_b_last_insert_mark(void)
{
  fmarkv_T view = mark_view_make(curwin->w_topline, curwin->w_cursor);
  RESET_FMARK(&curbuf->b_last_insert, curwin->w_cursor, curbuf->b_fnum, view);
}

int nvim_get_u_sync_once(void) { return u_sync_once; }

void nvim_set_u_sync_once(int val) { u_sync_once = val; }

/// Call edit_putchar(c, highlight != 0) (accessor for Rust).
void nvim_putchar(int c, int highlight) { edit_putchar(c, highlight != 0); }

colnr_T nvim_curwin_get_cursor_coladd(void) { return curwin->w_cursor.coladd; }

// ---- ins_reg accessors ----

/// Save cursor position for expression register evaluation (composite for Rust).
static pos_T ins_reg_saved_cursor;
void nvim_ins_reg_restore_cursor_save(void) { ins_reg_saved_cursor = curwin->w_cursor; }

/// Restore cursor position after expression register evaluation (composite for Rust).
void nvim_ins_reg_restore_cursor(void)
{
  curwin->w_cursor = ins_reg_saved_cursor;
  check_cursor(curwin);
}

