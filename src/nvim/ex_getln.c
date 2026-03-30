// ex_getln.c: Functions for entering and editing an Ex command line.

#include <assert.h>
#include <inttypes.h>
#include "nvim/assert_defs.h"
#include <limits.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/clipboard.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdpreview.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cmdhist.h"
#include "nvim/cursor.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/mapping.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/usercmd.h"
#include "nvim/vim_defs.h"
#include "nvim/viml/parser/expressions.h"
#include "nvim/viml/parser/parser.h"
#include "nvim/viml/parser/parser_defs.h"
#include "nvim/window.h"

/// Last value of prompt_id, incremented when doing new prompt
static unsigned last_prompt_id = 0;

// Struct to store the state of 'incsearch' highlighting.
typedef struct {
  pos_T search_start;   // where 'incsearch' starts searching
  pos_T save_cursor;
  handle_T winid;       // window where this state is valid
  viewstate_T init_viewstate;
  viewstate_T old_viewstate;
  pos_T match_start;
  pos_T match_end;
  bool did_incsearch;
  bool incsearch_postponed;
  optmagic_T magic_overruled_save;
} incsearch_state_T;

typedef struct {
  VimState state;
  int firstc;
  int count;
  int indent;
  int c;
  bool gotesc;                          // true when <ESC> just typed
  bool do_abbr;                         // when true check for abbr.
  char *lookfor;                        // string to match
  int lookforlen;
  int hiscnt;                           // current history line in use
  int save_hiscnt;                      // history line before attempting
                                        // to jump to next match
  int histype;                          // history type to be used
  incsearch_state_T is_state;
  bool did_wild_list;                   // did wild_list() recently
  int wim_index;                        // index in wim_flags[]
  int save_msg_scroll;
  int save_State;                       // remember State when called
  int prev_cmdpos;
  char *prev_cmdbuff;
  char *save_p_icm;
  bool skip_pum_redraw;
  bool some_key_typed;                  // one of the keys was typed
  // mouse drag and release events are ignored, unless they are
  // preceded with a mouse down event
  bool ignore_drag_release;
  bool break_ctrl_c;
  expand_T xpc;
  OptInt *b_im_ptr;
  buf_T *b_im_ptr_buf;  ///< buffer where b_im_ptr is valid
  int cmdline_type;
  bool event_cmdlineleavepre_triggered;
  bool did_hist_navigate;
} CommandLineState;

// Verify CommandLineState layout matches Rust repr(C) struct.
// If these fail, the Rust CommandLineState in command_line_state.rs must be updated.
_Static_assert(sizeof(CommandLineState) == 640,
               "CommandLineState size mismatch with Rust");
_Static_assert(offsetof(CommandLineState, firstc) == 16,
               "CommandLineState.firstc offset mismatch");
_Static_assert(offsetof(CommandLineState, lookfor) == 40,
               "CommandLineState.lookfor offset mismatch");
_Static_assert(offsetof(CommandLineState, is_state) == 64,
               "CommandLineState.is_state offset mismatch");
_Static_assert(offsetof(CommandLineState, did_wild_list) == 180,
               "CommandLineState.did_wild_list offset mismatch");
_Static_assert(offsetof(CommandLineState, xpc) == 224,
               "CommandLineState.xpc offset mismatch");
_Static_assert(offsetof(CommandLineState, b_im_ptr) == 616,
               "CommandLineState.b_im_ptr offset mismatch");
_Static_assert(offsetof(CommandLineState, cmdline_type) == 632,
               "CommandLineState.cmdline_type offset mismatch");

/// Return value when handling keys in command-line mode.
enum {
  CMDLINE_NOT_CHANGED = 1,
  CMDLINE_CHANGED     = 2,
  GOTO_NORMAL_MODE    = 3,
  PROCESS_NEXT_KEY    = 4,
};

/// The current cmdline_info.  It is initialized in getcmdline() and after that
/// used by other functions.  When invoking getcmdline() recursively, the old
/// value is copied to a local variable and ccline.prev_ccline is set.
static CmdlineInfo ccline;

static int new_cmdpos;          // position set by setcmdpos()/setcmdline() VimL functions

/// currently displayed block of context
static Array cmdline_block = ARRAY_DICT_INIT;

/// Flag for command_line_handle_key to ignore <C-c>
///
/// Used if it was received while processing highlight function in order for
/// user interrupting highlight function to not interrupt command-line.
static bool getln_interrupted_highlight = false;

static int cedit_key = -1;  ///< key value of 'cedit' option

#include "ex_getln.c.generated.h"
extern int rs_get_echo_hl_id(void);

// Rust FFI declarations
extern void correct_screencol(int idx, int cells, int *col);
extern int rs_magic_isset(void);
extern void rs_init_incsearch_state(incsearch_state_T *state);
extern void rs_finish_incsearch_highlighting(int gotesc, incsearch_state_T *state,
                                             int call_update_screen);
extern void rs_may_do_incsearch_highlighting(int firstc, int count, incsearch_state_T *s);
extern int may_add_char_to_search(int firstc, int *c, incsearch_state_T *s);
extern void draw_cmdline(int start, int len);
extern int rs_entry_should_use_cmdmsg_rl(int firstc, int win_p_rl, int win_p_rlc_has_s);
extern int rs_entry_should_add_to_history(int histype, int cmdlen, int firstc, int some_key_typed);
extern int rs_entry_should_save_last_cmdline(int firstc);
extern int rs_entry_hist_char2type(int firstc);
extern int rs_entry_cmdline_type(int firstc);
extern int rs_should_skip_coloring(unsigned int current_prompt_id, unsigned int prev_prompt_id,
                                   int prev_errors);
extern int rs_should_reset_callback_errors(unsigned int current_prompt_id,
                                           unsigned int prev_prompt_id);
extern int rs_cmd_startcol(void);
extern int rs_cmdline_charsize(int idx);
extern void rs_redrawcmdprompt(void);
extern int command_line_handle_key(void *s);
extern int command_line_wildchar_complete(void *s);
extern int command_line_execute(VimState *state, int key);
extern int command_line_check(VimState *state);
extern void rs_putcmdline(int c, bool shift);
extern void rs_unputcmdline(void);

// History browsing state (used by Rust cmdline/history.rs)
typedef struct {
  int c;
  int firstc;
  int hiscnt;
  int save_hiscnt;
  int histype;
  char *lookfor;
  int lookforlen;
} HistoryBrowseState;
extern int rs_command_line_browse_history(HistoryBrowseState *state);

/// Thin wrapper for Rust: trigger autocmd with int event (avoids event_T in public header).
void nvim_trigger_cmd_autocmd(int typechar, int evt)
{
  char typestr[2] = { (char)typechar, NUL };
  apply_autocmds((event_T)evt, typestr, typestr, false, curbuf);
}

/// Internal entry point for cmdline mode.
///
/// @param count  only used for incremental search
/// @param indent  indent for inside conditionals
/// @param clear_ccline  clear ccline first
static uint8_t *command_line_enter(int firstc, int count, int indent, bool clear_ccline)
{
  // can be invoked recursively, identify each level
  static int cmdline_level = 0;
  cmdline_level++;

  bool save_cmdpreview = cmdpreview;
  cmdpreview = false;
  CommandLineState state = {
    .firstc = firstc,
    .count = count,
    .indent = indent,
    .save_msg_scroll = msg_scroll,
    .save_State = State,
    .prev_cmdpos = -1,
    .ignore_drag_release = true,
  };
  CommandLineState *s = &state;
  s->save_p_icm = xstrdup(p_icm);
  rs_init_incsearch_state(&s->is_state);
  CmdlineInfo save_ccline;
  bool did_save_ccline = false;

  if (ccline.cmdbuff != NULL) {
    // Currently ccline can never be in use if clear_ccline is false.
    // Some changes will be needed if this is no longer the case.
    assert(clear_ccline);
    // Being called recursively.  Since ccline is global, we need to save
    // the current buffer and restore it when returning.
    save_ccline = ccline;
    CLEAR_FIELD(ccline);
    ccline.prev_ccline = &save_ccline;
    ccline.cmdbuff = NULL;  // signal that ccline is not in use
    did_save_ccline = true;
  } else if (clear_ccline) {
    CLEAR_FIELD(ccline);
  }

  if (s->firstc == -1) {
    s->firstc = NUL;
    s->break_ctrl_c = true;
  }

  ccline.overstrike = false;                // always start in insert mode
  assert(s->indent >= 0);
  // set some variables for redrawcmd()
  ccline.cmdfirstc = (s->firstc == '@' ? 0 : s->firstc);
  ccline.cmdindent = (s->firstc > 0 ? s->indent : 0);
  // alloc initial ccline.cmdbuff
  alloc_cmdbuff(s->indent + 50);
  ccline.cmdlen = ccline.cmdpos = 0;
  ccline.cmdbuff[0] = NUL;
  ccline.last_colors = (ColoredCmdline){ .cmdbuff = NULL,
                                         .colors = KV_INITIAL_VALUE };
  sb_text_start_cmdline();
  // autoindent for :insert and :append
  if (s->firstc <= 0) {
    memset(ccline.cmdbuff, ' ', (size_t)s->indent);
    ccline.cmdbuff[s->indent] = NUL;
    ccline.cmdpos = s->indent;
    ccline.cmdspos = s->indent;
    ccline.cmdlen = s->indent;
  }
  ccline.prompt_id = last_prompt_id++;
  ccline.level = cmdline_level;

  if (cmdline_level == 50) {
    // Somehow got into a loop recursively calling getcmdline(), bail out.
    emsg(_(e_command_too_recursive));
    goto theend;
  }

  ExpandInit(&s->xpc);
  ccline.xpc = &s->xpc;
  clear_cmdline_orig();

  // Use Rust helper to determine RTL command line mode
  cmdmsg_rl = rs_entry_should_use_cmdmsg_rl(s->firstc, curwin->w_p_rl,
                                            *curwin->w_p_rlc == 's');

  msg_grid_validate();

  redir_off = true;             // don't redirect the typed command
  if (!cmd_silent) {
    gotocmdline(true);
    rs_redrawcmdprompt();          // draw prompt or indent
    ccline.cmdspos = rs_cmd_startcol();
  }
  s->xpc.xp_context = EXPAND_NOTHING;
  s->xpc.xp_backslash = XP_BS_NONE;
#ifndef BACKSLASH_IN_FILENAME
  s->xpc.xp_shell = false;
#endif

  if (ccline.input_fn) {
    s->xpc.xp_context = ccline.xp_context;
    s->xpc.xp_pattern = ccline.cmdbuff;
    s->xpc.xp_arg = ccline.xp_arg;
  }

  // Avoid scrolling when called by a recursive do_cmdline(), e.g. when
  // doing ":@0" when register 0 doesn't contain a CR.
  msg_scroll = false;

  State = MODE_CMDLINE;

  if (s->firstc == '/' || s->firstc == '?' || s->firstc == '@') {
    // Use ":lmap" mappings for search pattern and input().
    if (curbuf->b_p_imsearch == B_IMODE_USE_INSERT) {
      s->b_im_ptr = &curbuf->b_p_iminsert;
    } else {
      s->b_im_ptr = &curbuf->b_p_imsearch;
    }
    s->b_im_ptr_buf = curbuf;
    if (*s->b_im_ptr == B_IMODE_LMAP) {
      State |= MODE_LANGMAP;
    }
  }

  setmouse();

  // Use Rust helper to get the cmdline type for events
  s->cmdline_type = rs_entry_cmdline_type(firstc);
  Error err = ERROR_INIT;
  char firstcbuf[2];
  firstcbuf[0] = (char)s->cmdline_type;
  firstcbuf[1] = 0;

  if (has_event(EVENT_CMDLINEENTER)) {
    save_v_event_T save_v_event;
    dict_T *dict = get_v_event(&save_v_event);

    // set v:event to a dictionary with information about the commandline
    tv_dict_add_str(dict, S_LEN("cmdtype"), firstcbuf);
    tv_dict_add_nr(dict, S_LEN("cmdlevel"), ccline.level);
    tv_dict_set_keys_readonly(dict);
    TRY_WRAP(&err, {
      apply_autocmds(EVENT_CMDLINEENTER, firstcbuf, firstcbuf, false, curbuf);
      restore_v_event(dict, &save_v_event);
    });

    if (ERROR_SET(&err)) {
      msg_putchar('\n');
      msg_scroll = true;
      msg_puts_hl(err.msg, HLF_E, true);
      api_clear_error(&err);
      redrawcmd();
    }
    err = ERROR_INIT;
  }
  may_trigger_modechanged();

  init_history();
  s->hiscnt = get_hislen();  // set hiscnt to impossible history value
  s->histype = rs_entry_hist_char2type(s->firstc);
  do_digraph(-1);                       // init digraph typeahead

  // If something above caused an error, reset the flags, we do want to type
  // and execute commands. Display may be messed up a bit.
  if (did_emsg) {
    redrawcmd();
  }

  // Redraw the statusline in case it uses the current mode using the mode()
  // function.
  if (!cmd_silent && !exmode_active) {
    bool found_one = false;

    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (*p_stl != NUL || *wp->w_p_stl != NUL || *p_wbr != NUL || *wp->w_p_wbr != NUL) {
        wp->w_redr_status = true;
        found_one = true;
      }
    }

    if (*p_tal != NUL) {
      redraw_tabline = true;
      found_one = true;
    }

    if (redraw_custom_title_later()) {
      found_one = true;
    }

    if (found_one) {
      redraw_statuslines();
    }
  }

  did_emsg = false;
  got_int = false;
  s->state.check = command_line_check;
  s->state.execute = command_line_execute;

  state_enter(&s->state);

  // Trigger CmdlineLeavePre autocommands if not already triggered.
  if (!s->event_cmdlineleavepre_triggered) {
    set_vim_var_char(s->c);  // Set v:char
    nvim_trigger_cmd_autocmd(s->cmdline_type, EVENT_CMDLINELEAVEPRE);
  }

  if (has_event(EVENT_CMDLINELEAVE)) {
    save_v_event_T save_v_event;
    dict_T *dict = get_v_event(&save_v_event);

    tv_dict_add_str(dict, S_LEN("cmdtype"), firstcbuf);
    tv_dict_add_nr(dict, S_LEN("cmdlevel"), ccline.level);
    tv_dict_set_keys_readonly(dict);
    // not readonly:
    tv_dict_add_bool(dict, S_LEN("abort"),
                     s->gotesc ? kBoolVarTrue : kBoolVarFalse);
    set_vim_var_char(s->c);  // Set v:char
    TRY_WRAP(&err, {
      apply_autocmds(EVENT_CMDLINELEAVE, firstcbuf, firstcbuf, false, curbuf);
      // error printed below, to avoid redraw issues
    });
    if (tv_dict_get_number(dict, "abort") != 0) {
      s->gotesc = true;
    }
    restore_v_event(dict, &save_v_event);
  }

  cmdmsg_rl = false;

  // We could have reached here without having a chance to clean up wild menu
  // if certain special keys like <Esc> or <C-\> were used as wildchar. Make
  // sure to still clean up to avoid memory corruption.
  if (cmdline_pum_active()) {
    cmdline_pum_remove(false);
  } else {
    // A previous cmdline_pum_remove() may have deferred redraw.
    pum_check_clear();
  }
  wildmenu_cleanup(&ccline);
  s->did_wild_list = false;
  s->wim_index = 0;

  ExpandCleanup(&s->xpc);
  ccline.xpc = NULL;
  clear_cmdline_orig();

  rs_finish_incsearch_highlighting(s->gotesc ? 1 : 0, &s->is_state, 0);

  if (ccline.cmdbuff != NULL) {
    // Put line in history buffer (":" and "=" only when it was typed).
    // Use Rust helper to determine if we should add to history.
    if (rs_entry_should_add_to_history(s->histype, ccline.cmdlen, s->firstc,
                                       s->some_key_typed)) {
      add_to_history(s->histype, ccline.cmdbuff, (size_t)ccline.cmdlen, true,
                     s->histype == HIST_SEARCH ? s->firstc : NUL);
      if (rs_entry_should_save_last_cmdline(s->firstc)) {
        xfree(new_last_cmdline);
        new_last_cmdline = xstrnsave(ccline.cmdbuff, (size_t)ccline.cmdlen);
      }
    }

    if (s->gotesc) {
      dealloc_cmdbuff();
      if (msg_scrolled == 0) {
        compute_cmdrow();
      }
      // Avoid overwriting key prompt
      if (!ccline.one_key) {
        msg("", 0);
        redraw_cmdline = true;
      }
    }
  }

  // If the screen was shifted up, redraw the whole screen (later).
  // If the line is too long, clear it, so ruler and shown command do
  // not get printed in the middle of it.
  msg_check();
  if (p_ch == 0 && !ui_has(kUIMessages)) {
    set_must_redraw(UPD_VALID);
  }
  msg_scroll = s->save_msg_scroll;
  redir_off = false;

  if (ERROR_SET(&err)) {
    msg_putchar('\n');
    emsg(err.msg);
    did_emsg = false;
    api_clear_error(&err);
  }

  // When the command line was typed, no need for a wait-return prompt.
  if (s->some_key_typed && !ERROR_SET(&err)) {
    need_wait_return = false;
  }

  set_option_direct(kOptInccommand, CSTR_AS_OPTVAL(s->save_p_icm), 0, SID_NONE);
  State = s->save_State;
  if (cmdpreview != save_cmdpreview) {
    cmdpreview = save_cmdpreview;  // restore preview state
    redraw_all_later(UPD_SOME_VALID);
  }
  may_trigger_modechanged();
  setmouse();
  sb_text_end_cmdline();

theend:
  xfree(s->save_p_icm);
  xfree(ccline.last_colors.cmdbuff);
  kv_destroy(ccline.last_colors.colors);

  char *p = ccline.cmdbuff;

  if (ui_has(kUICmdline)) {
    cmdline_was_last_drawn = false;
    ccline.redraw_state = kCmdRedrawNone;
    ui_call_cmdline_hide(ccline.level, s->gotesc);
  }
  if (!cmd_silent) {
    redraw_custom_title_later();
    status_redraw_all();  // redraw to show mode change
  }

  cmdline_level--;

  if (did_save_ccline) {
    ccline = save_ccline;
  } else {
    ccline.cmdbuff = NULL;
  }

  xfree(s->prev_cmdbuff);
  return (uint8_t *)p;
}

/// Trigger CmdlineChanged autocommands.
static void do_autocmd_cmdlinechanged(int firstc)
{
  if (has_event(EVENT_CMDLINECHANGED)) {
    Error err = ERROR_INIT;
    save_v_event_T save_v_event;
    dict_T *dict = get_v_event(&save_v_event);

    char firstcbuf[2];
    firstcbuf[0] = (char)firstc;
    firstcbuf[1] = 0;

    // set v:event to a dictionary with information about the commandline
    tv_dict_add_str(dict, S_LEN("cmdtype"), firstcbuf);
    tv_dict_add_nr(dict, S_LEN("cmdlevel"), ccline.level);
    tv_dict_set_keys_readonly(dict);
    TRY_WRAP(&err, {
      apply_autocmds(EVENT_CMDLINECHANGED, firstcbuf, firstcbuf, false, curbuf);
      restore_v_event(dict, &save_v_event);
    });
    if (ERROR_SET(&err)) {
      msg_putchar('\n');
      msg_scroll = true;
      msg_puts_hl(err.msg, HLF_E, true);
      api_clear_error(&err);
      redrawcmd();
    }
  }
}

/// Abandon the command line.
/// getcmdline() - accept a command line starting with firstc.
///
/// firstc == ':'            get ":" command line.
/// firstc == '/' or '?'     get search pattern
/// firstc == '='            get expression
/// firstc == '@'            get text for input() function
/// firstc == '>'            get text for debug mode
/// firstc == NUL            get text for :insert command
/// firstc == -1             like NUL, and break on CTRL-C
///
/// The line is collected in ccline.cmdbuff, which is reallocated to fit the
/// command line.
///
/// Careful: getcmdline() can be called recursively!
///
/// Return pointer to allocated string if there is a commandline, NULL
/// otherwise.
///
/// @param count  only used for incremental search
/// @param indent  indent for inside conditionals
char *getcmdline(int firstc, int count, int indent, bool do_concat FUNC_ATTR_UNUSED)
{
  return (char *)command_line_enter(firstc, count, indent, true);
}

/// Get a command line with a prompt
///
/// This is prepared to be called recursively from getcmdline() (e.g. by
/// f_input() when evaluating an expression from `<C-r>=`).
///
/// @param[in]  firstc  Prompt type: e.g. '@' for input(), '>' for debug.
/// @param[in]  prompt  Prompt string: what is displayed before the user text.
/// @param[in]  hl_id  Prompt highlight id.
/// @param[in]  xp_context  Type of expansion.
/// @param[in]  xp_arg  User-defined expansion argument.
/// @param[in]  highlight_callback  Callback used for highlighting user input.
/// @param[in]  one_key  Return after one key press for button prompt.
/// @param[in]  mouse_used  Set to true when returning after right mouse click.
///
/// @return [allocated] Command line or NULL.
char *getcmdline_prompt(const int firstc, const char *const prompt, const int hl_id,
                        const int xp_context, const char *const xp_arg,
                        const Callback highlight_callback, bool one_key, bool *mouse_used)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_MALLOC
{
  const int msg_col_save = msg_col;

  CmdlineInfo save_ccline;
  bool did_save_ccline = false;
  if (ccline.cmdbuff != NULL) {
    // Save the values of the current cmdline and restore them below.
    save_ccline = ccline;
    CLEAR_FIELD(ccline);
    ccline.prev_ccline = &save_ccline;
    ccline.cmdbuff = NULL;  // signal that ccline is not in use
    did_save_ccline = true;
  } else {
    CLEAR_FIELD(ccline);
  }
  ccline.prompt_id = last_prompt_id++;
  ccline.cmdprompt = (char *)prompt;
  ccline.hl_id = hl_id;
  ccline.xp_context = xp_context;
  ccline.xp_arg = (char *)xp_arg;
  ccline.input_fn = (firstc == '@');
  ccline.highlight_callback = highlight_callback;
  ccline.one_key = one_key;
  ccline.mouse_used = mouse_used;

  const bool cmd_silent_saved = cmd_silent;
  int msg_silent_saved = msg_silent;
  msg_silent = 0;
  cmd_silent = false;  // Want to see the prompt.

  char *const ret = (char *)command_line_enter(firstc, 1, 0, false);
  ccline.redraw_state = kCmdRedrawNone;

  if (did_save_ccline) {
    ccline = save_ccline;
  }
  msg_silent = msg_silent_saved;
  cmd_silent = cmd_silent_saved;
  // Restore msg_col, the prompt from input() may have changed it.
  // But only if called recursively and the commandline is therefore being
  // restored to an old one; if not, the input() prompt stays on the screen,
  // so we need its modified msg_col left intact.
  if (ccline.cmdbuff != NULL) {
    msg_col = msg_col_save;
  }

  return ret;
}

// check_opt_wim() is implemented in Rust (cmdline crate, wildmenu.rs).

/// Get an Ex command line for the ":" command.
///
/// @param c  normally ':', NUL for ":append"
/// @param indent  indent for inside conditionals
char *getexline(int c, void *cookie, int indent, bool do_concat)
{
  // When executing a register, remove ':' that's in front of each line.
  if (exec_from_reg && vpeekc() == ':') {
    vgetc();
  }

  return getcmdline(c, 1, indent, do_concat);
}

/// Deallocate a command line buffer, updating the buffer size and length.
void dealloc_cmdbuff(void)
{
  XFREE_CLEAR(ccline.cmdbuff);
  ccline.cmdlen = ccline.cmdbufflen = 0;
}

/// Allocate a new command line buffer.
/// Assigns the new buffer to ccline.cmdbuff and ccline.cmdbufflen.
void alloc_cmdbuff(int len)
{
  // give some extra space to avoid having to allocate all the time
  if (len < 80) {
    len = 100;
  } else {
    len += 20;
  }

  ccline.cmdbuff = xmalloc((size_t)len);
  ccline.cmdbufflen = len;
}

/// Re-allocate the command line to length len + something extra.
void realloc_cmdbuff(int len)
{
  if (len < ccline.cmdbufflen) {
    return;  // no need to resize
  }

  char *p = ccline.cmdbuff;

  alloc_cmdbuff(len);                   // will get some more
  // There isn't always a NUL after the command, but it may need to be
  // there, thus copy up to the NUL and add a NUL.
  memmove(ccline.cmdbuff, p, (size_t)ccline.cmdlen);
  ccline.cmdbuff[ccline.cmdlen] = NUL;

  if (ccline.xpc != NULL
      && ccline.xpc->xp_pattern != NULL
      && ccline.xpc->xp_context != EXPAND_NOTHING
      && ccline.xpc->xp_context != EXPAND_UNSUCCESSFUL) {
    int i = (int)(ccline.xpc->xp_pattern - p);

    // If xp_pattern points inside the old cmdbuff it needs to be adjusted
    // to point into the newly allocated memory.
    if (i >= 0 && i <= ccline.cmdlen) {
      ccline.xpc->xp_pattern = ccline.cmdbuff + i;
    }
  }

  xfree(p);
}

enum { MAX_CB_ERRORS = 1, };

/// Color the command-line (ccline).
///
/// Uses built-in expression parser for '=' prompt, or user-specified callback.
/// Caches results: if prompt_id and cmdbuff are unchanged, returns immediately.
///
/// @return true if draw_cmdline may proceed, false if nothing to do.
bool nvim_color_cmdline(void)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  bool printed_errmsg = false;

#define PRINT_ERRMSG(...) \
  do { \
    msg_scroll = true; \
    msg_putchar('\n'); \
    smsg(HLF_E, __VA_ARGS__); \
    printed_errmsg = true; \
  } while (0)
  bool ret = true;

  ColoredCmdline *ccline_colors = &ccline.last_colors;

  // Check whether result of the previous call is still valid.
  if (ccline_colors->prompt_id == ccline.prompt_id
      && ccline_colors->cmdbuff != NULL
      && strcmp(ccline_colors->cmdbuff, ccline.cmdbuff) == 0) {
    return ret;
  }

  kv_size(ccline_colors->colors) = 0;

  if (ccline.cmdbuff == NULL || *ccline.cmdbuff == NUL) {
    // Nothing to do, exiting.
    XFREE_CLEAR(ccline_colors->cmdbuff);
    return ret;
  }

  bool arg_allocated = false;
  typval_T arg = {
    .v_type = VAR_STRING,
    .vval.v_string = ccline.cmdbuff,
  };
  typval_T tv = { .v_type = VAR_UNKNOWN };

  static unsigned prev_prompt_id = UINT_MAX;
  static int prev_prompt_errors = 0;
  Callback color_cb = CALLBACK_NONE;
  bool can_free_cb = false;
  Error err = ERROR_INIT;
  const char *err_errmsg = e_intern2;
  bool dgc_ret = true;

  // Use Rust helper to check if callback errors should be reset.
  if (rs_should_reset_callback_errors(ccline.prompt_id, prev_prompt_id)) {
    prev_prompt_errors = 0;
    prev_prompt_id = ccline.prompt_id;
  } else if (rs_should_skip_coloring(ccline.prompt_id, prev_prompt_id, prev_prompt_errors)) {
    // Skip coloring due to too many previous errors.
    goto color_cmdline_end;
  }
  if (ccline.highlight_callback.type != kCallbackNone) {
    // Currently this should only happen while processing input() prompts.
    assert(ccline.input_fn);
    color_cb = ccline.highlight_callback;
  } else if (ccline.cmdfirstc == ':') {
    TRY_WRAP(&err, {
      err_errmsg = N_("E5408: Unable to get g:Nvim_color_cmdline callback: %s");
      dgc_ret = tv_dict_get_callback(get_globvar_dict(), S_LEN("Nvim_color_cmdline"),
                                     &color_cb);
    });
    can_free_cb = true;
  } else if (ccline.cmdfirstc == '=') {
    // Inline color_expr_cmdline: parse expression and build highlight chunks.
    ParserLine parser_lines[] = {
      {
        .data = ccline.cmdbuff,
        .size = strlen(ccline.cmdbuff),
        .allocated = false,
      },
      { NULL, 0, false },
    };
    ParserLine *plines_p = parser_lines;
    ParserHighlight expr_colors;
    kvi_init(expr_colors);
    ParserState pstate;
    viml_parser_init(&pstate, parser_simple_get_line, &plines_p, &expr_colors);
    ExprAST east = viml_pexpr_parse(&pstate, kExprFlagsDisallowEOC);
    viml_pexpr_free_ast(east);
    viml_parser_destroy(&pstate);
    kv_resize(ccline_colors->colors, kv_size(expr_colors));
    size_t expr_prev_end = 0;
    for (size_t ei = 0; ei < kv_size(expr_colors); ei++) {
      const ParserHighlightChunk chunk = kv_A(expr_colors, ei);
      assert(chunk.start.col < INT_MAX);
      assert(chunk.end_col < INT_MAX);
      if (chunk.start.col != expr_prev_end) {
        kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
          .start = (int)expr_prev_end,
          .end = (int)chunk.start.col,
          .hl_id = 0,
        }));
      }
      kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
        .start = (int)chunk.start.col,
        .end = (int)chunk.end_col,
        .hl_id = syn_name2id(chunk.group),
      }));
      expr_prev_end = chunk.end_col;
    }
    if (expr_prev_end < (size_t)ccline.cmdlen) {
      kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
        .start = (int)expr_prev_end,
        .end = ccline.cmdlen,
        .hl_id = 0,
      }));
    }
    kvi_destroy(expr_colors);
  }
  if (ERROR_SET(&err) || !dgc_ret) {
    goto color_cmdline_error;
  }

  if (color_cb.type == kCallbackNone) {
    goto color_cmdline_end;
  }
  if (ccline.cmdbuff[ccline.cmdlen] != NUL) {
    arg_allocated = true;
    arg.vval.v_string = xmemdupz(ccline.cmdbuff, (size_t)ccline.cmdlen);
  }
  // msg_start() called by e.g. :echo may shift command-line to the first column
  // even though msg_silent is here. Two ways to workaround this problem without
  // altering message.c: use full_screen or save and restore msg_col.
  //
  // Saving and restoring full_screen does not work well with :redraw!. Saving
  // and restoring msg_col is neither ideal, but while with full_screen it
  // appears shifted one character to the right and cursor position is no longer
  // correct, with msg_col it just misses leading `:`. Since `redraw!` in
  // callback lags this is least of the user problems.
  //
  // Also using TRY_WRAP because error messages may overwrite typed
  // command-line which is not expected.
  getln_interrupted_highlight = false;
  bool cbcall_ret = true;
  TRY_WRAP(&err, {
    err_errmsg = N_("E5407: Callback has thrown an exception: %s");
    const int saved_msg_col = msg_col;
    msg_silent++;
    cbcall_ret = callback_call(&color_cb, 1, &arg, &tv);
    msg_silent--;
    msg_col = saved_msg_col;
    if (got_int) {
      getln_interrupted_highlight = true;
    }
  });
  if (ERROR_SET(&err) || !cbcall_ret) {
    goto color_cmdline_error;
  }
  if (tv.v_type != VAR_LIST) {
    PRINT_ERRMSG("%s", _("E5400: Callback should return list"));
    goto color_cmdline_error;
  }
  if (tv.vval.v_list == NULL) {
    goto color_cmdline_end;
  }
  varnumber_T prev_end = 0;
  int i = 0;
  TV_LIST_ITER_CONST(tv.vval.v_list, li, {
    if (TV_LIST_ITEM_TV(li)->v_type != VAR_LIST) {
      PRINT_ERRMSG(_("E5401: List item %i is not a List"), i);
      goto color_cmdline_error;
    }
    const list_T *const l = TV_LIST_ITEM_TV(li)->vval.v_list;
    if (tv_list_len(l) != 3) {
      PRINT_ERRMSG(_("E5402: List item %i has incorrect length: %d /= 3"),
                   i, tv_list_len(l));
      goto color_cmdline_error;
    }
    bool error = false;
    const varnumber_T start = (
                               tv_get_number_chk(TV_LIST_ITEM_TV(tv_list_first(l)), &error));
    if (error) {
      goto color_cmdline_error;
    } else if (!(prev_end <= start && start < ccline.cmdlen)) {
      PRINT_ERRMSG(_("E5403: Chunk %i start %" PRIdVARNUMBER " not in range "
                     "[%" PRIdVARNUMBER ", %i)"),
                   i, start, prev_end, ccline.cmdlen);
      goto color_cmdline_error;
    } else if (utf8len_tab_zero[(uint8_t)ccline.cmdbuff[start]] == 0) {
      PRINT_ERRMSG(_("E5405: Chunk %i start %" PRIdVARNUMBER " splits "
                     "multibyte character"), i, start);
      goto color_cmdline_error;
    }
    if (start != prev_end) {
      kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
        .start = (int)prev_end,
        .end = (int)start,
        .hl_id = 0,
      }));
    }
    const varnumber_T end =
      tv_get_number_chk(TV_LIST_ITEM_TV(TV_LIST_ITEM_NEXT(l, tv_list_first(l))), &error);
    if (error) {
      goto color_cmdline_error;
    } else if (!(start < end && end <= ccline.cmdlen)) {
      PRINT_ERRMSG(_("E5404: Chunk %i end %" PRIdVARNUMBER " not in range "
                     "(%" PRIdVARNUMBER ", %i]"),
                   i, end, start, ccline.cmdlen);
      goto color_cmdline_error;
    } else if (end < ccline.cmdlen
               && (utf8len_tab_zero[(uint8_t)ccline.cmdbuff[end]]
                   == 0)) {
      PRINT_ERRMSG(_("E5406: Chunk %i end %" PRIdVARNUMBER " splits multibyte "
                     "character"), i, end);
      goto color_cmdline_error;
    }
    prev_end = end;
    const char *const group = tv_get_string_chk(TV_LIST_ITEM_TV(tv_list_last(l)));
    if (group == NULL) {
      goto color_cmdline_error;
    }
    kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
      .start = (int)start,
      .end = (int)end,
      .hl_id = syn_name2id(group),
    }));
    i++;
  });
  if (prev_end < ccline.cmdlen) {
    kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
      .start = (int)prev_end,
      .end = ccline.cmdlen,
      .hl_id = 0,
    }));
  }
  prev_prompt_errors = 0;
color_cmdline_end:
  assert(!ERROR_SET(&err));
  if (can_free_cb) {
    callback_free(&color_cb);
  }
  xfree(ccline_colors->cmdbuff);
  // Note: errors “output” is cached just as well as regular results.
  ccline_colors->prompt_id = ccline.prompt_id;
  if (arg_allocated) {
    ccline_colors->cmdbuff = arg.vval.v_string;
  } else {
    ccline_colors->cmdbuff = xmemdupz(ccline.cmdbuff, (size_t)ccline.cmdlen);
  }
  tv_clear(&tv);
  return ret;
color_cmdline_error:
  if (ERROR_SET(&err)) {
    PRINT_ERRMSG(_(err_errmsg), err.msg);
    api_clear_error(&err);
  }
  assert(printed_errmsg);
  (void)printed_errmsg;

  prev_prompt_errors++;
  kv_size(ccline_colors->colors) = 0;
  redrawcmdline();
  ret = false;
  goto color_cmdline_end;
#undef PRINT_ERRMSG
}

void ui_ext_cmdline_block_append(size_t indent, const char *line)
{
  char *buf = xmallocz(indent + strlen(line));
  memset(buf, ' ', indent);
  memcpy(buf + indent, line, strlen(line));

  Array item = ARRAY_DICT_INIT;
  ADD(item, INTEGER_OBJ(0));
  ADD(item, CSTR_AS_OBJ(buf));
  ADD(item, INTEGER_OBJ(0));
  Array content = ARRAY_DICT_INIT;
  ADD(content, ARRAY_OBJ(item));
  ADD(cmdline_block, ARRAY_OBJ(content));
  if (cmdline_block.size > 1) {
    ui_call_cmdline_block_append(content);
  } else {
    ui_call_cmdline_block_show(cmdline_block);
  }
}

void ui_ext_cmdline_block_leave(void)
{
  api_free_array(cmdline_block);
  cmdline_block = (Array)ARRAY_DICT_INIT;
  ui_call_cmdline_block_hide();
}

// cmdline_screen_cleared: implemented in Rust (cmdline crate, ui.rs)

/// called by ui_flush, do what redraws necessary to keep cmdline updated.
void cmdline_ui_flush(void)
{
  if (!ui_has(kUICmdline)) {
    return;
  }
  int level = ccline.level;
  CmdlineInfo *line = &ccline;
  while (level > 0 && line) {
    if (line->level == level) {
      CmdRedraw redraw_state = line->redraw_state;
      line->redraw_state = kCmdRedrawNone;
      if (redraw_state == kCmdRedrawAll) {
        cmdline_was_last_drawn = true;
        {
          Arena arena = ARENA_EMPTY;
          Array content;
          if (cmdline_star) {
            content = arena_array(&arena, 1);
            size_t len = 0;
            for (char *p = ccline.cmdbuff; *p; MB_PTR_ADV(p)) {
              len++;
            }
            char *buf = arena_alloc(&arena, len, false);
            memset(buf, '*', len);
            Array item = arena_array(&arena, 3);
            ADD_C(item, INTEGER_OBJ(0));
            ADD_C(item, STRING_OBJ(cbuf_as_string(buf, len)));
            ADD_C(item, INTEGER_OBJ(0));
            ADD_C(content, ARRAY_OBJ(item));
          } else if (kv_size(line->last_colors.colors)) {
            content = arena_array(&arena, kv_size(line->last_colors.colors));
            for (size_t i = 0; i < kv_size(line->last_colors.colors); i++) {
              CmdlineColorChunk chunk = kv_A(line->last_colors.colors, i);
              Array item = arena_array(&arena, 3);
              ADD_C(item, INTEGER_OBJ(chunk.hl_id == 0 ? 0 : syn_id2attr(chunk.hl_id)));
              assert(chunk.end >= chunk.start);
              ADD_C(item, STRING_OBJ(cbuf_as_string(line->cmdbuff + chunk.start,
                                                    (size_t)(chunk.end - chunk.start))));
              ADD_C(item, INTEGER_OBJ(chunk.hl_id));
              ADD_C(content, ARRAY_OBJ(item));
            }
          } else {
            Array item = arena_array(&arena, 3);
            ADD_C(item, INTEGER_OBJ(0));
            ADD_C(item, CSTR_AS_OBJ(line->cmdbuff));
            ADD_C(item, INTEGER_OBJ(0));
            content = arena_array(&arena, 1);
            ADD_C(content, ARRAY_OBJ(item));
          }
          char charbuf[2] = { (char)line->cmdfirstc, 0 };
          ui_call_cmdline_show(content, line->cmdpos,
                               cstr_as_string(charbuf),
                               cstr_as_string((line->cmdprompt)),
                               line->cmdindent, line->level, line->hl_id);
          if (line->special_char) {
            charbuf[0] = line->special_char;
            ui_call_cmdline_special_char(cstr_as_string(charbuf),
                                         line->special_shift,
                                         line->level);
          }
          arena_mem_free(arena_finish(&arena));
        }
      } else if (redraw_state == kCmdRedrawPos && cmdline_was_last_drawn) {
        ui_call_cmdline_pos(line->cmdpos, line->level);
      }
      level--;
    }
    line = line->prev_ccline;
  }
}

/// Thin C wrapper: emit kUICmdline special_char event (for Rust FFI).
void nvim_ui_cmdline_special_char(int c, bool shift, int level)
{
  char charbuf[2] = { (char)c, 0 };
  ui_call_cmdline_special_char(cstr_as_string(charbuf), shift, level);
}

// Put a character on the command line.  Thin C wrapper: delegates to Rust.
void putcmdline(char c, bool shift) { rs_putcmdline((int)(unsigned char)c, shift); }

/// Undo a putcmdline(c, false).  Thin C wrapper: delegates to Rust.
void unputcmdline(void) { rs_unputcmdline(); }

// put_on_cmdline, cmdline_paste_str, redrawcmdline, redrawcmd: implemented in Rust (cmdline crate).

/// Get a pointer to the current command line info.
CmdlineInfo *get_cmdline_info(void) { return &ccline; }

unsigned get_cmdline_last_prompt_id(void) { return last_prompt_id; }

/// Get pointer to the active command line info, or NULL if not in cmdline mode.
/// When ccline is saved recursively, the previous value is in ccline.prev_ccline.
CmdlineInfo *nvim_get_ccline_ptr(void)
{
  if ((State & MODE_CMDLINE) == 0) {
    return NULL;
  } else if (ccline.cmdbuff != NULL) {
    return &ccline;
  } else if (ccline.prev_ccline && ccline.prev_ccline->cmdbuff != NULL) {
    return ccline.prev_ccline;
  } else {
    return NULL;
  }
}

/// Public wrapper for do_autocmd_cmdlinechanged (for VimL functions in other files).
void nvim_do_autocmd_cmdlinechanged(int firstc) { do_autocmd_cmdlinechanged(firstc); }

/// Get the current command-line type.
/// Returns ':' or '/' or '?' or '@' or '>' or '-'
/// Only works when the command line is being edited.
/// Returns NUL when something is wrong.
int nvim_get_cmdline_type(void)
{
  CmdlineInfo *p = nvim_get_ccline_ptr();
  if (p == NULL) {
    return NUL;
  }
  if (p->cmdfirstc == NUL) {
    return (p->input_fn) ? '@' : '-';
  }
  return p->cmdfirstc;
}


// C accessor for ccline.cmdfirstc (used by Rust)
int nvim_get_ccline_cmdfirstc(void) { return ccline.cmdfirstc; }

void cmdline_init(void) { CLEAR_FIELD(ccline); }

/// Check value of 'cedit' and set cedit_key.
/// Returns NULL if value is OK, error message otherwise.
const char *did_set_cedit(optset_T *args)
{
  if (*p_cedit == NUL) {
    cedit_key = -1;
  } else {
    int n = string_to_key(p_cedit);
    if (n == 0 || vim_isprintc(n)) {
      return e_invarg;
    }
    cedit_key = n;
  }
  return NULL;
}
// C accessors for Rust to read/write ccline fields
int nvim_get_ccline_overstrike(void) { return ccline.overstrike; }
int nvim_get_ccline_cmdpos(void) { return ccline.cmdpos; }
int nvim_get_ccline_cmdlen(void) { return ccline.cmdlen; }
int nvim_get_cmdwin_type(void) { return cmdwin_type; }
int nvim_get_textlock(void) { return textlock; }
const char *nvim_get_e_cmdwin(void) { return e_cmdwin; }
const char *nvim_get_e_textlock(void) { return e_textlock; }
int nvim_get_ccline_cmdspos(void) { return ccline.cmdspos; }
int nvim_get_ccline_cmdindent(void) { return ccline.cmdindent; }
unsigned int nvim_get_ccline_prompt_id(void) { return ccline.prompt_id; }
int nvim_get_ccline_level(void) { return ccline.level; }
int nvim_get_ccline_input_fn(void) { return ccline.input_fn; }
char *nvim_get_ccline_cmdbuff(void) { return ccline.cmdbuff; }
int nvim_get_ccline_cmdbufflen(void) { return ccline.cmdbufflen; }
void nvim_set_ccline_cmdpos(int pos) { ccline.cmdpos = pos; }
void nvim_set_ccline_cmdlen(int len) { ccline.cmdlen = len; }
void nvim_set_ccline_cmdspos(int spos) { ccline.cmdspos = spos; }
void nvim_set_ccline_cmdbuff(char *buff) { ccline.cmdbuff = buff; }
void nvim_set_ccline_cmdbufflen(int len) { ccline.cmdbufflen = len; }
void nvim_set_ccline_overstrike(int overstrike) { ccline.overstrike = overstrike; }
int nvim_get_ccline_redraw_state(void) { return (int)ccline.redraw_state; }
void nvim_set_ccline_redraw_state(int state) { ccline.redraw_state = (CmdRedraw)state; }
int nvim_get_ccline_special_char(void) { return ccline.special_char; }
void nvim_set_ccline_special_char(int c) { ccline.special_char = (char)c; }
int nvim_get_ccline_special_shift(void) { return ccline.special_shift; }
void nvim_set_ccline_special_shift(int shift) { ccline.special_shift = shift; }
int nvim_get_ccline_one_key(void) { return ccline.one_key; }
void nvim_set_ccline_one_key(int one_key) { ccline.one_key = one_key; }

int nvim_get_ccline_hl_id(void) { return ccline.hl_id; }
void nvim_set_ccline_hl_id(int hl_id) { ccline.hl_id = hl_id; }
int nvim_get_ccline_xp_context(void) { return ccline.xp_context; }
void nvim_set_ccline_xp_context(int context) { ccline.xp_context = context; }
char *nvim_get_ccline_cmdprompt(void) { return ccline.cmdprompt; }
void nvim_set_ccline_cmdprompt(char *prompt) { ccline.cmdprompt = prompt; }
void nvim_set_ccline_cmdindent(int indent) { ccline.cmdindent = indent; }
void nvim_set_ccline_cmdfirstc(int firstc) { ccline.cmdfirstc = firstc; }
void nvim_set_ccline_cmdbuff_at(int idx, char val) { ccline.cmdbuff[idx] = val; }

void nvim_strcpy_cmdbuff(const char *src)
{
  if (src != NULL && ccline.cmdbuff != NULL) {
    STRCPY(ccline.cmdbuff, src);
  }
}

int nvim_get_key_typed(void) { return KeyTyped; }
int nvim_get_cmdline_star(void) { return cmdline_star; }
int nvim_cmdline_win_is_active(void) { return cmdline_win != NULL; }
int nvim_cmdline_win_width(void) { return cmdline_win ? cmdline_win->w_view_width : 0; }
int nvim_cmdline_win_height(void) { return cmdline_win ? cmdline_win->w_view_height : 0; }

/// Simplified wrapper around getcmdline_prompt for FFI use.
/// Avoids passing the complex Callback union across FFI boundary.
char *nvim_getcmdline_prompt_simple(int firstc, const char *prompt, int hl_id,
                                    int xp_context, bool one_key, bool *mouse_used)
{
  return getcmdline_prompt(firstc, prompt, hl_id, xp_context, NULL,
                           CALLBACK_NONE, one_key, mouse_used);
}

/// Increment textlock.
void nvim_inc_textlock(void) { textlock++; }
/// Decrement textlock.
void nvim_dec_textlock(void) { textlock--; }

/// Get p_wim (wildmode option string) for Rust.
char *nvim_get_p_wim(void) { return p_wim; }

uint8_t nvim_get_wim_flags(int idx) { return wim_flags[idx]; }
void nvim_set_wim_flags(int idx, uint8_t val) { wim_flags[idx] = val; }
size_t nvim_get_ccline_colors_size(void) { return kv_size(ccline.last_colors.colors); }

/// Get a color chunk's fields by index.
void nvim_get_ccline_color_chunk(size_t idx, int *start_out, int *end_out, int *hl_id_out)
{
  CmdlineColorChunk chunk = kv_A(ccline.last_colors.colors, idx);
  *start_out = chunk.start;
  *end_out = chunk.end;
  *hl_id_out = chunk.hl_id;
}

void nvim_set_redrawing_cmdline(int val) { redrawing_cmdline = (val != 0); }
void nvim_set_msg_no_more(int val) { msg_no_more = (val != 0); }
void nvim_set_skip_redraw2(int val) { skip_redraw = (val != 0); }
int nvim_get_key_typed_cmdline(void) { return KeyTyped ? 1 : 0; }
void nvim_msg_check(void) { msg_check(); }
int nvim_get_p_ru(void) { return p_ru; }

/// Move cursor to end of search match (called from Rust).
void nvim_set_search_match(pos_T *t)
{
  // First move cursor to end of match, then to the start.  This
  // moves the whole match onto the screen when 'nowrap' is set.
  t->lnum += search_match_lines;
  t->col = search_match_endcol;
  if (t->lnum > curbuf->b_ml.ml_line_count) {
    t->lnum = curbuf->b_ml.ml_line_count;
    coladvance(curwin, MAXCOL);
  }
}

int nvim_get_new_cmdpos(void) { return new_cmdpos; }
void nvim_set_new_cmdpos(int val) { new_cmdpos = val; }
void nvim_set_key_typed(int val) { KeyTyped = (val != 0); }


// CommandLineState field accessors
int nvim_cls_get_c(void *s) { return ((CommandLineState *)s)->c; }
void nvim_cls_set_c(void *s, int val) { ((CommandLineState *)s)->c = val; }
int nvim_cls_get_firstc(void *s) { return ((CommandLineState *)s)->firstc; }
int nvim_cls_get_count(void *s) { return ((CommandLineState *)s)->count; }
int nvim_cls_get_indent(void *s) { return ((CommandLineState *)s)->indent; }
int nvim_cls_get_gotesc(void *s) { return ((CommandLineState *)s)->gotesc ? 1 : 0; }
void nvim_cls_set_gotesc(void *s, int val) { ((CommandLineState *)s)->gotesc = (val != 0); }
int nvim_cls_get_do_abbr(void *s) { return ((CommandLineState *)s)->do_abbr ? 1 : 0; }
void nvim_cls_set_do_abbr(void *s, int val) { ((CommandLineState *)s)->do_abbr = (val != 0); }
int nvim_cls_get_ignore_drag_release(void *s) { return ((CommandLineState *)s)->ignore_drag_release ? 1 : 0; }
void nvim_cls_set_ignore_drag_release(void *s, int val) { ((CommandLineState *)s)->ignore_drag_release = (val != 0); }
int nvim_cls_get_did_wild_list(void *s) { return ((CommandLineState *)s)->did_wild_list ? 1 : 0; }
void nvim_cls_set_did_wild_list(void *s, int val) { ((CommandLineState *)s)->did_wild_list = (val != 0); }
int nvim_cls_get_wim_index(void *s) { return ((CommandLineState *)s)->wim_index; }
void nvim_cls_set_wim_index(void *s, int val) { ((CommandLineState *)s)->wim_index = val; }
void nvim_cls_set_skip_pum_redraw(void *s, int val) { ((CommandLineState *)s)->skip_pum_redraw = (val != 0); }
void nvim_cls_set_prev_cmdpos(void *s, int val) { ((CommandLineState *)s)->prev_cmdpos = val; }
void nvim_cls_xfree_prev_cmdbuff(void *s) { XFREE_CLEAR(((CommandLineState *)s)->prev_cmdbuff); }

/// Set s->prev_cmdbuff to a copy of ccline.cmdbuff (if non-NULL).
void nvim_cls_dup_cmdbuff_to_prev(void *s)
{
  if (ccline.cmdbuff != NULL) {
    XFREE_CLEAR(((CommandLineState *)s)->prev_cmdbuff);
    ((CommandLineState *)s)->prev_cmdbuff = xstrdup(ccline.cmdbuff);
  }
}

void nvim_cls_set_some_key_typed(void *s, int val) { ((CommandLineState *)s)->some_key_typed = (val != 0); }
int nvim_cls_get_did_hist_navigate(void *s) { return ((CommandLineState *)s)->did_hist_navigate ? 1 : 0; }
void nvim_cls_set_did_hist_navigate(void *s, int val) { ((CommandLineState *)s)->did_hist_navigate = (val != 0); }
void *nvim_cls_get_is_state(void *s) { return (void *)&((CommandLineState *)s)->is_state; }
void *nvim_cls_get_xpc(void *s) { return (void *)&((CommandLineState *)s)->xpc; }
int nvim_cls_get_xpc_numfiles(void *s) { return ((CommandLineState *)s)->xpc.xp_numfiles; }
void nvim_cls_set_xpc_context(void *s, int val) { ((CommandLineState *)s)->xpc.xp_context = val; }
void nvim_cls_set_event_cmdlineleavepre_triggered(void *s, int val) { ((CommandLineState *)s)->event_cmdlineleavepre_triggered = (val != 0); }
int nvim_cls_get_break_ctrl_c(void *s) { return ((CommandLineState *)s)->break_ctrl_c ? 1 : 0; }

/// Get ccline.mouse_used pointer (may be NULL).
int nvim_cls_get_ccline_mouse_used(void) { return ccline.mouse_used != NULL ? 1 : 0; }

/// Set the value at ccline.mouse_used if non-NULL.
void nvim_cls_set_ccline_mouse_used_val(int val)
{
  if (ccline.mouse_used != NULL) {
    *ccline.mouse_used = (val != 0);
  }
}

/// Wrapper for cmdline_pum_cleanup(&ccline) (called from Rust).
void nvim_cmdline_pum_cleanup(void) { cmdline_pum_cleanup(&ccline); }


/// Get getln_interrupted_highlight global.
int nvim_get_getln_interrupted_highlight(void) { return getln_interrupted_highlight ? 1 : 0; }

/// Set getln_interrupted_highlight global.
void nvim_set_getln_interrupted_highlight(int val) { getln_interrupted_highlight = (val != 0); }


/// Get cedit_key value (static variable, exposed for Rust).
int nvim_get_cedit_key(void) { return cedit_key; }

/// Get s->lookfor field (may be NULL).
char *nvim_cls_get_lookfor(void *s) { return ((CommandLineState *)s)->lookfor; }

/// Free s->lookfor and set to NULL (XFREE_CLEAR equivalent).
void nvim_cls_xfree_lookfor(void *s)
{
  CommandLineState *cs = (CommandLineState *)s;
  XFREE_CLEAR(cs->lookfor);
  cs->lookforlen = 0;
}

/// Get cmdline_was_last_drawn global.
int nvim_get_cmdline_was_last_drawn(void) { return cmdline_was_last_drawn ? 1 : 0; }

/// Wrapper for wildmenu_translate_key (called from Rust).
int nvim_wildmenu_translate_key(void *s)
{
  CommandLineState *cs = (CommandLineState *)s;
  return wildmenu_translate_key(&ccline, cs->c, &cs->xpc, cs->did_wild_list);
}

/// Wrapper for wildmenu_process_key (called from Rust).
int nvim_wildmenu_process_key(void *s)
{
  CommandLineState *cs = (CommandLineState *)s;
  return wildmenu_process_key(&ccline, cs->c, &cs->xpc);
}

/// Get is_state.did_incsearch field.
int nvim_cls_get_is_state_did_incsearch(void *s) { return ((CommandLineState *)s)->is_state.did_incsearch ? 1 : 0; }

/// Call do_cmdline(NULL, getcmdkeycmd, NULL, DOCMD_NOWAIT) for Rust.
void nvim_cmdline_do_cmdline_nowait(void) { do_cmdline(NULL, getcmdkeycmd, NULL, DOCMD_NOWAIT); }

// =============================================================================
// Helpers for Rust implementations of cmdline_screen_cleared / cmdline_ui_flush
// =============================================================================

/// Get cmdline_block.size (for Rust cmdline_screen_cleared).
size_t nvim_get_cmdline_block_size(void) { return cmdline_block.size; }

/// Call ui_call_cmdline_block_show(cmdline_block) (macro wrapper for Rust).
void nvim_ui_call_cmdline_block_show_all(void) { ui_call_cmdline_block_show(cmdline_block); }

/// Call ui_call_cmdline_block_hide() (macro wrapper for Rust).
void nvim_ui_call_cmdline_block_hide(void) { ui_call_cmdline_block_hide(); }

/// Get ccline.prev_ccline as opaque pointer (for Rust iteration).
void *nvim_get_ccline_prev_ptr(void) { return (void *)ccline.prev_ccline; }

/// Get a CmdlineInfo node's level field (for Rust iteration).
int nvim_ccline_ptr_get_level(void *p) { return ((CmdlineInfo *)p)->level; }

/// Set a CmdlineInfo node's redraw_state to kCmdRedrawAll (for Rust).
void nvim_ccline_ptr_set_redraw_all(void *p) { ((CmdlineInfo *)p)->redraw_state = kCmdRedrawAll; }

/// Get a CmdlineInfo node's prev_ccline field (for Rust iteration).
void *nvim_ccline_ptr_get_prev(void *p) { return (void *)((CmdlineInfo *)p)->prev_ccline; }

/// Get cmdwin_level value.
int nvim_get_cmdwin_level(void) { return cmdwin_level; }


