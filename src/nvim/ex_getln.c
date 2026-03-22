// ex_getln.c: Functions for entering and editing an Ex command line.

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

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
#include "nvim/os/os.h"
#include "nvim/path.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
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
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/usercmd.h"
#include "nvim/vim_defs.h"
#include "nvim/viml/parser/expressions.h"
#include "nvim/viml/parser/parser.h"
#include "nvim/viml/parser/parser_defs.h"
#include "nvim/window.h"

/// Last value of prompt_id, incremented when doing new prompt
static unsigned last_prompt_id = 0;

// Struct to store the viewstate during 'incsearch' highlighting and 'inccommand' preview.
typedef struct {
  colnr_T vs_curswant;
  colnr_T vs_leftcol;
  colnr_T vs_skipcol;
  linenr_T vs_topline;
  int vs_topfill;
  linenr_T vs_botline;
  int vs_empty_rows;
} viewstate_T;

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

typedef struct {
  u_header_T *save_b_u_oldhead;
  u_header_T *save_b_u_newhead;
  u_header_T *save_b_u_curhead;
  int save_b_u_numhead;
  bool save_b_u_synced;
  int save_b_u_seq_last;
  int save_b_u_save_nr_last;
  int save_b_u_seq_cur;
  time_t save_b_u_time_cur;
  int save_b_u_save_nr_cur;
  char *save_b_u_line_ptr;
  linenr_T save_b_u_line_lnum;
  colnr_T save_b_u_line_colnr;
} CpUndoInfo;

typedef struct {
  buf_T *buf;
  OptInt save_b_p_ul;
  int save_b_p_ma;
  int save_b_changed;
  pos_T save_b_op_start;
  pos_T save_b_op_end;
  varnumber_T save_changedtick;
  CpUndoInfo undo_info;
} CpBufInfo;

typedef struct {
  win_T *win;
  pos_T save_w_cursor;
  viewstate_T save_viewstate;
  int save_w_p_cul;
  int save_w_p_cuc;
} CpWinInfo;

typedef struct {
  kvec_t(CpWinInfo) win_info;
  kvec_t(CpBufInfo) buf_info;
  bool save_hls;
  cmdmod_T save_cmdmod;
  garray_T save_view;
} CpInfo;

/// Return value when handling keys in command-line mode.
enum {
  CMDLINE_NOT_CHANGED = 1,
  CMDLINE_CHANGED     = 2,
  GOTO_NORMAL_MODE    = 3,
  PROCESS_NEXT_KEY    = 4,
};

/// The current cmdline_info.  It is initialized in getcmdline() and after that
/// used by other functions.  When invoking getcmdline() recursively it needs
/// to be saved with save_cmdline() and restored with restore_cmdline().
static CmdlineInfo ccline;

static int new_cmdpos;          // position set by set_cmdline_pos()

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
extern int rs_win_valid(win_T *win);
extern int rs_last_window(win_T *win);

// Rust FFI declarations
extern int rs_global_stl_height(void);
// Phase 3: Rust-implemented functions replacing C static/non-static functions
extern void correct_screencol(int idx, int cells, int *col);
extern void rs_win_size_restore(garray_T *gap);
extern void rs_win_size_save(garray_T *gap);
extern void rs_clear_showcmd(void);

extern int rs_magic_isset(void);

extern int rs_cmdline_delete_char_before(void);
extern int rs_cmdline_delete_word_before(void);

// Phase 1: History browsing from Rust
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

// Rust incsearch state functions
extern void rs_init_incsearch_state(incsearch_state_T *state);
extern void rs_finish_incsearch_highlighting(int gotesc, incsearch_state_T *state,
                                             int call_update_screen);

// do_incsearch_highlighting: implemented in Rust (cmdline/search.rs)
extern bool do_incsearch_highlighting(int firstc, int *search_delim, incsearch_state_T *is_state,
                                      int *skiplen, int *patlen);
// may_do_incsearch_highlighting: implemented in Rust (cmdline/search.rs)
extern void rs_may_do_incsearch_highlighting(int firstc, int count, incsearch_state_T *s);
// may_do_command_line_next_incsearch: implemented in Rust (cmdline/search.rs)
extern int rs_may_do_command_line_next_incsearch(int firstc, int count, incsearch_state_T *s,
                                                 bool next_match);
// may_add_char_to_search: implemented in Rust (cmdline/search.rs)
extern int may_add_char_to_search(int firstc, int *c, incsearch_state_T *s);
// draw_cmdline: implemented in Rust (cmdline/screen.rs)
extern void draw_cmdline(int start, int len);

// Rust key dispatch helpers
extern int rs_invert_rtl_key(int key);
extern int rs_should_end_wildmenu(int key, int p_wc, int p_wcm);
extern int rs_should_end_wildmenu_pum(int key);
extern int rs_is_stab_to_ctrl_p(int key, int p_wc);

// Entry/exit orchestration helpers from Rust
extern int rs_entry_should_use_cmdmsg_rl(int firstc, int win_p_rl, int win_p_rlc_has_s);
extern int rs_entry_should_add_to_history(int histype, int cmdlen, int firstc, int some_key_typed);
extern int rs_entry_should_save_last_cmdline(int firstc);
extern int rs_entry_hist_char2type(int firstc);
extern int rs_entry_cmdline_type(int firstc);

// Phase 7: Command window helpers from Rust
extern int rs_cmdwin_can_open(int cmdwin_type_active, int text_locked, int cmdline_star);
extern int rs_cmdwin_split_invalid(int old_curwin_valid, int curwin_is_old,
                                   int old_curbuf_valid, int buf_changed);
extern int rs_cmdwin_buffer_invalid(int newbuf_status_ok, int cmdwin_valid,
                                    int curwin_is_cmdwin, int old_curwin_valid,
                                    int old_curbuf_valid, int buf_changed);
extern int rs_cmdwin_needs_tab_mapping(int histtype, int p_wc);
extern int rs_cmdwin_needs_vim_filetype(int histtype);
extern int rs_cmdwin_cleanup_had_error(int old_curwin_valid, int old_curbuf_valid, int buf_changed);
extern int rs_cmdwin_to_hist_type(int win_type);

// Drawing and coloring helpers from Rust
extern int rs_should_skip_coloring(unsigned int current_prompt_id, unsigned int prev_prompt_id,
                                   int prev_errors);
extern int rs_should_reset_callback_errors(unsigned int current_prompt_id,
                                           unsigned int prev_prompt_id);

// Phase 10: VimL API helpers from Rust
extern int rs_clamp_cmdpos(int pos, int cmdlen);

// Phase 67: Screen position helpers from Rust
extern int rs_cmd_startcol(void);
extern int rs_cmdline_charsize(int idx);

// Phase 67: Empty pattern detection from Rust
extern int rs_empty_pattern_magic(const char *p, size_t len, int magic_val);
extern int rs_empty_pattern(char *p, size_t len, int delim);

// Phase 67: Redraw helpers from Rust
extern void rs_redrawcmdprompt(void);

// Phase 67: Abbreviation check from Rust
extern int rs_ccheck_abbr(int c);

// Phase 67: Viewstate helpers from Rust
extern void rs_save_viewstate_win(win_T *wp, viewstate_T *vs);
extern void rs_restore_viewstate_win(win_T *wp, viewstate_T *vs);

// Phase 3: command_line_handle_key implemented in Rust (cmdline/keys.rs)
extern int command_line_handle_key(void *s);


static handle_T cmdpreview_bufnr = 0;
static int cmdpreview_ns = 0;

static const char e_active_window_or_buffer_changed_or_deleted[]
  = N_("E199: Active window or buffer changed or deleted");

static void trigger_cmd_autocmd(int typechar, event_T evt)
{
  char typestr[2] = { (char)typechar, NUL };
  apply_autocmds(evt, typestr, typestr, false, curbuf);
}


static void set_search_match(pos_T *t)
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

/// Parses the :[range]s/foo like commands and returns details needed for
/// incsearch and wildmenu completion.
/// Returns true if pattern is valid.
/// Sets skiplen, patlen, search_first_line, and search_last_line.
bool parse_pattern_and_range(pos_T *incsearch_start, int *search_delim, int *skiplen, int *patlen)
  FUNC_ATTR_NONNULL_ALL
{
  char *p;
  bool delim_optional = false;
  const char *dummy;
  magic_T magic = 0;

  *skiplen = 0;
  *patlen = ccline.cmdlen;

  // Default range
  search_first_line = 0;
  search_last_line = MAXLNUM;

  exarg_T ea = {
    .line1 = 1,
    .line2 = 1,
    .cmd = ccline.cmdbuff,
    .addr_type = ADDR_LINES,
  };

  cmdmod_T dummy_cmdmod;
  // Skip over command modifiers
  parse_command_modifiers(&ea, &dummy, &dummy_cmdmod, true);

  // Skip over the range to find the command.
  char *cmd = skip_range(ea.cmd, NULL);
  if (vim_strchr("sgvl", (uint8_t)(*cmd)) == NULL) {
    return false;
  }

  // Skip over command name to find pattern separator
  for (p = cmd; ASCII_ISALPHA(*p); p++) {}
  if (*skipwhite(p) == NUL) {
    return false;
  }

  if (strncmp(cmd, "substitute", (size_t)(p - cmd)) == 0
      || strncmp(cmd, "smagic", (size_t)(p - cmd)) == 0
      || strncmp(cmd, "snomagic", (size_t)MAX(p - cmd, 3)) == 0
      || strncmp(cmd, "vglobal", (size_t)(p - cmd)) == 0) {
    if (*cmd == 's' && cmd[1] == 'm') {
      magic_overruled = OPTION_MAGIC_ON;
    } else if (*cmd == 's' && cmd[1] == 'n') {
      magic_overruled = OPTION_MAGIC_OFF;
    }
  } else if (strncmp(cmd, "sort", (size_t)MAX(p - cmd, 3)) == 0
             || strncmp(cmd, "uniq", (size_t)MAX(p - cmd, 3)) == 0) {
    // skip over ! and flags
    if (*p == '!') {
      p = skipwhite(p + 1);
    }
    while (ASCII_ISALPHA(*(p = skipwhite(p)))) {
      p++;
    }
    if (*p == NUL) {
      return false;
    }
  } else if (strncmp(cmd, "vimgrep", (size_t)MAX(p - cmd, 3)) == 0
             || strncmp(cmd, "vimgrepadd", (size_t)MAX(p - cmd, 8)) == 0
             || strncmp(cmd, "lvimgrep", (size_t)MAX(p - cmd, 2)) == 0
             || strncmp(cmd, "lvimgrepadd", (size_t)MAX(p - cmd, 9)) == 0
             || strncmp(cmd, "global", (size_t)(p - cmd)) == 0) {
    // skip optional "!"
    if (*p == '!') {
      p++;
      if (*skipwhite(p) == NUL) {
        return false;
      }
    }
    if (*cmd != 'g') {
      delim_optional = true;
    }
  } else {
    return false;
  }

  p = skipwhite(p);
  int delim = (delim_optional && vim_isIDc((uint8_t)(*p))) ? ' ' : *p++;
  *search_delim = delim;

  char *end = skip_regexp_ex(p, delim, rs_magic_isset(), NULL, NULL, &magic);
  bool use_last_pat = end == p && *end == delim;

  if (end == p && !use_last_pat) {
    return false;
  }

  // Skip if the pattern matches everything (e.g., for 'hlsearch')
  if (!use_last_pat) {
    char c = *end;
    *end = NUL;
    bool empty = (bool)rs_empty_pattern_magic(p, (size_t)(end - p), (int)magic);
    *end = c;
    if (empty) {
      return false;
    }
  }

  // Found a non-empty pattern or //
  *skiplen = (int)(p - ccline.cmdbuff);
  *patlen = (int)(end - p);

  // Parse the address range
  pos_T save_cursor = curwin->w_cursor;
  curwin->w_cursor = *incsearch_start;

  parse_cmd_address(&ea, &dummy, true);

  if (ea.addr_count > 0) {
    // Allow for reverse match.
    search_first_line = MIN(ea.line2, ea.line1);
    search_last_line = MAX(ea.line2, ea.line1);
  } else if (cmd[0] == 's' && cmd[1] != 'o') {
    // :s defaults to the current line
    search_first_line = search_last_line = curwin->w_cursor.lnum;
  }

  curwin->w_cursor = save_cursor;
  return true;
}

// do_incsearch_highlighting() is implemented in Rust (cmdline crate, search.rs);
// declared below in the extern section.

// may_do_incsearch_highlighting() and may_add_char_to_search() are implemented in Rust (cmdline crate, search.rs).

/// Initialize the current command-line info.
static void init_ccline(int firstc, int indent)
{
  ccline.overstrike = false;                // always start in insert mode

  assert(indent >= 0);

  // set some variables for redrawcmd()
  ccline.cmdfirstc = (firstc == '@' ? 0 : firstc);
  ccline.cmdindent = (firstc > 0 ? indent : 0);

  // alloc initial ccline.cmdbuff
  alloc_cmdbuff(indent + 50);
  ccline.cmdlen = ccline.cmdpos = 0;
  ccline.cmdbuff[0] = NUL;

  ccline.last_colors = (ColoredCmdline){ .cmdbuff = NULL,
                                         .colors = KV_INITIAL_VALUE };
  sb_text_start_cmdline();

  // autoindent for :insert and :append
  if (firstc <= 0) {
    memset(ccline.cmdbuff, ' ', (size_t)indent);
    ccline.cmdbuff[indent] = NUL;
    ccline.cmdpos = indent;
    ccline.cmdspos = indent;
    ccline.cmdlen = indent;
  }
}

static void ui_ext_cmdline_hide(bool abort)
{
  if (ui_has(kUICmdline)) {
    cmdline_was_last_drawn = false;
    ccline.redraw_state = kCmdRedrawNone;
    ui_call_cmdline_hide(ccline.level, abort);
  }
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
    save_cmdline(&save_ccline);
    did_save_ccline = true;
  } else if (clear_ccline) {
    CLEAR_FIELD(ccline);
  }

  if (s->firstc == -1) {
    s->firstc = NUL;
    s->break_ctrl_c = true;
  }

  init_ccline(s->firstc, s->indent);
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
    trigger_cmd_autocmd(s->cmdline_type, EVENT_CMDLINELEAVEPRE);
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
      abandon_cmdline();
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
    ui_ext_cmdline_hide(s->gotesc);
  }
  if (!cmd_silent) {
    redraw_custom_title_later();
    status_redraw_all();  // redraw to show mode change
  }

  cmdline_level--;

  if (did_save_ccline) {
    restore_cmdline(&save_ccline);
  } else {
    ccline.cmdbuff = NULL;
  }

  xfree(s->prev_cmdbuff);
  return (uint8_t *)p;
}

static int command_line_check(VimState *state)
{
  CommandLineState *s = (CommandLineState *)state;

  s->prev_cmdpos = ccline.cmdpos;
  XFREE_CLEAR(s->prev_cmdbuff);

  redir_off = true;        // Don't redirect the typed command.
  // Repeated, because a ":redir" inside
  // completion may switch it on.
  quit_more = false;       // reset after CTRL-D which had a more-prompt

  did_emsg = false;        // There can't really be a reason why an error
                           // that occurs while typing a command should
                           // cause the command not to be executed.

  if (stuff_empty() && typebuf.tb_len == 0) {
    // There is no pending input from sources other than user input, so
    // Vim is going to wait for the user to type a key.  Consider the
    // command line typed even if next key will trigger a mapping.
    s->some_key_typed = true;
  }

  // Trigger SafeState if nothing is pending.
  may_trigger_safestate(s->xpc.xp_numfiles <= 0);

  if (ccline.cmdbuff != NULL) {
    s->prev_cmdbuff = xstrdup(ccline.cmdbuff);
  }

  // Defer screen update to avoid pum flicker during wildtrigger()
  if (s->c == K_WILD && s->firstc != '@') {
    s->skip_pum_redraw = true;
  }

  cursorcmd();             // set the cursor on the right spot
  ui_cursor_shape();
  return 1;
}

// command_line_handle_ctrl_bsl() is implemented in Rust (cmdline crate, keys.rs).

/// Completion for 'wildchar' or 'wildcharm' key.
/// - hitting <ESC> twice means: abandon command line.
/// - wildcard expansion is only done when the 'wildchar' key is really
///   typed, not when it comes from a macro
/// @return  CMDLINE_CHANGED if command line is changed or CMDLINE_NOT_CHANGED.
static int command_line_wildchar_complete(CommandLineState *s)
{
  int res;
  int options = WILD_NO_BEEP;
  bool escape = s->firstc != '@';
  bool redraw_if_menu_empty = s->c == K_WILD;
  bool wim_noselect = p_wmnu && (wim_flags[0] & kOptWimFlagNoselect) != 0;

  if (wim_flags[s->wim_index] & kOptWimFlagLastused) {
    options |= WILD_BUFLASTUSED;
  }
  if (s->xpc.xp_numfiles > 0) {       // typed p_wc at least twice
    // If "list" is present, list matches unless already listed
    if (s->xpc.xp_numfiles > 1
        && !s->did_wild_list
        && (wim_flags[s->wim_index] & kOptWimFlagList)) {
      showmatches(&s->xpc, false, true, wim_noselect);
      redrawcmd();
      s->did_wild_list = true;
    }
    if (wim_flags[s->wim_index] & kOptWimFlagLongest) {
      res = nextwild(&s->xpc, WILD_LONGEST, options, escape);
    } else if (wim_flags[s->wim_index] & kOptWimFlagFull) {
      res = nextwild(&s->xpc, WILD_NEXT, options, escape);
    } else {
      res = OK;                 // don't insert 'wildchar' now
    }
  } else {                    // typed p_wc first time
    bool wim_longest = (wim_flags[0] & kOptWimFlagLongest);
    bool wim_list = (wim_flags[0] & kOptWimFlagList);
    bool wim_full = (wim_flags[0] & kOptWimFlagFull);

    s->wim_index = 0;
    if (s->c == p_wc || s->c == p_wcm || s->c == K_WILD || s->c == Ctrl_Z) {
      options |= WILD_MAY_EXPAND_PATTERN;
      if (s->c == K_WILD) {
        options |= WILD_FUNC_TRIGGER;
      }
      s->xpc.xp_pre_incsearch_pos = s->is_state.search_start;
    }
    int cmdpos_before = ccline.cmdpos;

    // if 'wildmode' first contains "longest", get longest
    // common part
    if (wim_longest) {
      res = nextwild(&s->xpc, WILD_LONGEST, options, escape);
    } else {
      if (wim_noselect || wim_list) {
        options |= WILD_NOSELECT;
      }
      res = nextwild(&s->xpc, WILD_EXPAND_KEEP, options, escape);
    }

    // Remove popup menu if no completion items are available
    if (redraw_if_menu_empty && s->xpc.xp_numfiles <= 0) {
      pum_check_clear();
    }

    // if interrupted while completing, behave like it failed
    if (got_int) {
      vpeekc();               // remove <C-C> from input stream
      got_int = false;              // don't abandon the command line
      ExpandOne(&s->xpc, NULL, NULL, 0, WILD_FREE);
      s->xpc.xp_context = EXPAND_NOTHING;
      return CMDLINE_CHANGED;
    }

    // Display matches
    if (res == OK && s->xpc.xp_numfiles > (wim_noselect ? 0 : 1)) {
      if (wim_longest) {
        bool found_longest_prefix = (ccline.cmdpos != cmdpos_before);
        if (wim_list || (p_wmnu && wim_full)) {
          showmatches(&s->xpc, p_wmnu, wim_list, true);
        } else if (!found_longest_prefix) {
          bool wim_list_next = (wim_flags[1] & kOptWimFlagList);
          bool wim_full_next = (wim_flags[1] & kOptWimFlagFull);
          bool wim_noselect_next = (wim_flags[1] & kOptWimFlagNoselect);
          if (wim_list_next || (p_wmnu && (wim_full_next || wim_noselect_next))) {
            if (wim_full_next && !wim_noselect_next) {
              nextwild(&s->xpc, WILD_NEXT, options, escape);
            } else {
              showmatches(&s->xpc, p_wmnu, wim_list_next, wim_noselect_next);
            }
            if (wim_list_next) {
              s->did_wild_list = true;
            }
          }
        }
      } else {
        if (wim_list || (p_wmnu && (wim_full || wim_noselect))) {
          showmatches(&s->xpc, p_wmnu, wim_list, wim_noselect);
        } else {
          vim_beep(kOptBoFlagWildmode);
        }
      }

      redrawcmd();
      if (wim_list) {
        s->did_wild_list = true;
      }
    } else if (s->xpc.xp_numfiles == -1) {
      s->xpc.xp_context = EXPAND_NOTHING;
    }
  }

  if (s->wim_index < 3) {
    s->wim_index++;
  }

  if (s->c == ESC) {
    s->gotesc = true;
  }

  return (res == OK) ? CMDLINE_CHANGED : CMDLINE_NOT_CHANGED;
}

static void command_line_end_wildmenu(CommandLineState *s, bool key_is_wc)
{
  if (cmdline_pum_active()) {
    s->skip_pum_redraw = (s->skip_pum_redraw && !key_is_wc
                          && !ascii_iswhite(s->c)
                          && (vim_isprintc(s->c)
                              || s->c == K_BS || s->c == Ctrl_H || s->c == K_DEL
                              || s->c == K_KDEL || s->c == Ctrl_W || s->c == Ctrl_U));
    cmdline_pum_remove(s->skip_pum_redraw);
  }
  if (s->xpc.xp_numfiles != -1) {
    ExpandOne(&s->xpc, NULL, NULL, 0, WILD_FREE);
  }
  s->did_wild_list = false;
  if (!p_wmnu || (s->c != K_UP && s->c != K_DOWN)) {
    s->xpc.xp_context = EXPAND_NOTHING;
  }
  s->wim_index = 0;
  wildmenu_cleanup(&ccline);
}

static int command_line_execute(VimState *state, int key)
{
  if (key == K_IGNORE || key == K_NOP) {
    return -1;  // get another key
  }

  disptick_T display_tick_saved = display_tick;
  CommandLineState *s = (CommandLineState *)state;
  s->c = key;

  // Skip wildmenu during history navigation via Up/Down keys
  if (s->c == K_WILD && s->did_hist_navigate) {
    s->did_hist_navigate = false;
    return 1;
  }

  if (s->c == K_EVENT || s->c == K_COMMAND || s->c == K_LUA) {
    if (s->c == K_EVENT) {
      state_handle_k_event();
    } else if (s->c == K_COMMAND) {
      do_cmdline(NULL, getcmdkeycmd, NULL, DOCMD_NOWAIT);
    } else {
      map_execute_lua(false, false);
    }
    // If the window changed incremental search state is not valid.
    if (s->is_state.winid != curwin->handle) {
      rs_init_incsearch_state(&s->is_state);
    }
    // Re-apply 'incsearch' highlighting in case it was cleared.
    if (display_tick > display_tick_saved && s->is_state.did_incsearch) {
      rs_may_do_incsearch_highlighting(s->firstc, s->count, &s->is_state);
    }

    // nvim_select_popupmenu_item() can be called from the handling of
    // K_EVENT, K_COMMAND, or K_LUA.
    if (pum_want.active) {
      if (cmdline_pum_active()) {
        nextwild(&s->xpc, WILD_PUM_WANT, 0, s->firstc != '@');
        if (pum_want.finish) {
          nextwild(&s->xpc, WILD_APPLY, WILD_NO_BEEP, s->firstc != '@');
          command_line_end_wildmenu(s, false);
        }
      }
      pum_want.active = false;
    }

    if (!cmdline_was_last_drawn) {
      redrawcmdline();
    }
    return 1;
  }

  if (KeyTyped) {
    s->some_key_typed = true;

    if (cmdmsg_rl && !KeyStuffed) {
      // Invert horizontal movements and operations.  Only when
      // typed by the user directly, not when the result of a
      // mapping.
      s->c = rs_invert_rtl_key(s->c);
    }
  }

  // Ignore got_int when CTRL-C was typed here.
  // Don't ignore it in :global, we really need to break then, e.g., for
  // ":g/pat/normal /pat" (without the <CR>).
  // Don't ignore it for the input() function.
  if ((s->c == Ctrl_C)
      && s->firstc != '@'
      // do clear got_int in Ex mode to avoid infinite Ctrl-C loop
      && (!s->break_ctrl_c || exmode_active)
      && !global_busy) {
    got_int = false;
  }

  // free old command line when finished moving around in the history
  // list
  if (s->lookfor != NULL
      && s->c != K_S_DOWN && s->c != K_S_UP
      && s->c != K_DOWN && s->c != K_UP
      && s->c != K_PAGEDOWN && s->c != K_PAGEUP
      && s->c != K_KPAGEDOWN && s->c != K_KPAGEUP
      && s->c != K_LEFT && s->c != K_RIGHT
      && (s->xpc.xp_numfiles > 0 || (s->c != Ctrl_P && s->c != Ctrl_N))) {
    XFREE_CLEAR(s->lookfor);
    s->lookforlen = 0;
  }

  // When there are matching completions to select <S-Tab> works like
  // CTRL-P (unless 'wc' is <S-Tab>).
  if (rs_is_stab_to_ctrl_p(s->c, (int)p_wc) && s->xpc.xp_numfiles > 0) {
    s->c = Ctrl_P;
  }

  if (p_wmnu) {
    s->c = wildmenu_translate_key(&ccline, s->c, &s->xpc, s->did_wild_list);
  }

  int wild_type = 0;
  const bool key_is_wc = (s->c == p_wc && KeyTyped) || s->c == p_wcm;
  if ((cmdline_pum_active() || wild_menu_showing || s->did_wild_list) && !key_is_wc) {
    // Ctrl-Y: Accept the current selection and close the popup menu.
    // Ctrl-E: cancel the cmdline popup menu and return the original text.
    if (s->c == Ctrl_E || s->c == Ctrl_Y) {
      wild_type = (s->c == Ctrl_E) ? WILD_CANCEL : WILD_APPLY;
      nextwild(&s->xpc, wild_type, WILD_NO_BEEP, s->firstc != '@');
    }
  }

  // Trigger CmdlineLeavePre autocommand
  if ((KeyTyped && (s->c == '\n' || s->c == '\r' || s->c == K_KENTER || s->c == ESC))
      || s->c == Ctrl_C) {
    set_vim_var_char(s->c);  // Set v:char
    trigger_cmd_autocmd(s->cmdline_type, EVENT_CMDLINELEAVEPRE);
    s->event_cmdlineleavepre_triggered = true;
    if ((s->c == ESC || s->c == Ctrl_C) && (wim_flags[0] & kOptWimFlagList)) {
      set_no_hlsearch(true);
    }
  }

  // The wildmenu is cleared if the pressed key is not used for
  // navigating the wild menu (i.e. the key is not 'wildchar' or
  // 'wildcharm' or Ctrl-N or Ctrl-P or Ctrl-A or Ctrl-L).
  // If the popup menu is displayed, then PageDown and PageUp keys are
  // also used to navigate the menu.
  bool end_wildmenu = !key_is_wc && rs_should_end_wildmenu(s->c, (int)p_wc, (int)p_wcm);
  end_wildmenu = end_wildmenu && (!cmdline_pum_active() || rs_should_end_wildmenu_pum(s->c));

  // free expanded names when finished walking through matches
  if (end_wildmenu) {
    command_line_end_wildmenu(s, key_is_wc);
  }

  if (p_wmnu) {
    s->c = wildmenu_process_key(&ccline, s->c, &s->xpc);
  }

  // CTRL-\ CTRL-N or CTRL-\ CTRL-G goes to Normal mode,
  // CTRL-\ e prompts for an expression.
  if (s->c == Ctrl_BSL) {
    switch (rs_command_line_handle_ctrl_bsl(&s->c, &s->gotesc)) {
    case CMDLINE_CHANGED:
      return command_line_changed(s);
    case CMDLINE_NOT_CHANGED:
      return command_line_not_changed(s);
    case GOTO_NORMAL_MODE:
      return 0;                   // back to cmd mode
    default:
      s->c = Ctrl_BSL;            // backslash key not processed by
                                  // rs_command_line_handle_ctrl_bsl()
    }
  }

  if (s->c == cedit_key || s->c == K_CMDWIN) {
    // TODO(vim): why is ex_normal_busy checked here?
    if ((s->c == K_CMDWIN || ex_normal_busy == 0)
        && got_int == false) {
      // Open a window to edit the command line (and history).
      s->c = open_cmdwin();
      s->some_key_typed = true;
    }
  } else {
    s->c = do_digraph(s->c);
  }

  if (s->c == '\n'
      || s->c == '\r'
      || s->c == K_KENTER
      || (s->c == ESC
          && (!KeyTyped || vim_strchr(p_cpo, CPO_ESC) != NULL))) {
    // In Ex mode a backslash escapes a newline.
    if (exmode_active
        && s->c != ESC
        && ccline.cmdpos == ccline.cmdlen
        && ccline.cmdpos > 0
        && ccline.cmdbuff[ccline.cmdpos - 1] == '\\') {
      if (s->c == K_KENTER) {
        s->c = '\n';
      }
    } else {
      s->gotesc = false;         // Might have typed ESC previously, don't
                                 // truncate the cmdline now.
      if (rs_ccheck_abbr(s->c + ABBR_OFF)) {
        return command_line_changed(s);
      }

      if (!cmd_silent) {
        if (!ui_has(kUICmdline)) {
          msg_cursor_goto(msg_row, 0);
        }
        ui_flush();
      }
      return 0;
    }
  }

  // Completion for 'wildchar', 'wildcharm', and wildtrigger()
  if ((s->c == p_wc && !s->gotesc && KeyTyped) || s->c == p_wcm || s->c == K_WILD
      || s->c == Ctrl_Z) {
    if (s->c == K_WILD) {
      emsg_silent++;  // Silence the bell
    }
    int res = command_line_wildchar_complete(s);
    if (s->c == K_WILD) {
      emsg_silent--;
    }
    if (res == CMDLINE_CHANGED) {
      return command_line_changed(s);
    }
    if (s->c == K_WILD) {
      return command_line_not_changed(s);
    }
  }

  s->gotesc = false;

  // <S-Tab> goes to last match, in a clumsy way
  if (s->c == K_S_TAB && KeyTyped) {
    if (nextwild(&s->xpc, WILD_EXPAND_KEEP, 0, s->firstc != '@') == OK) {
      if (s->xpc.xp_numfiles > 1
          && ((!s->did_wild_list && (wim_flags[s->wim_index] & kOptWimFlagList)) || p_wmnu)) {
        // Trigger the popup menu when wildoptions=pum
        showmatches(&s->xpc, p_wmnu, wim_flags[s->wim_index] & kOptWimFlagList,
                    wim_flags[0] & kOptWimFlagNoselect);
      }
      nextwild(&s->xpc, WILD_PREV, 0, s->firstc != '@');
      nextwild(&s->xpc, WILD_PREV, 0, s->firstc != '@');
      return command_line_changed(s);
    }
  }

  if (s->c == NUL || s->c == K_ZERO) {
    // NUL is stored as NL
    s->c = NL;
  }

  s->do_abbr = true;             // default: check for abbreviation

  // If already used to cancel/accept wildmenu, don't process the key further.
  if (wild_type == WILD_CANCEL || wild_type == WILD_APPLY) {
    // Apply search highlighting
    if (s->is_state.winid != curwin->handle) {
      rs_init_incsearch_state(&s->is_state);
    }
    if (KeyTyped || vpeekc() == NUL) {
      rs_may_do_incsearch_highlighting(s->firstc, s->count, &s->is_state);
    }
    return command_line_not_changed(s);
  }

  return command_line_handle_key(s);
}

// may_do_command_line_next_incsearch() is implemented in Rust (cmdline crate, search.rs).
// command_line_erase_chars() is implemented in Rust (cmdline crate, keys.rs).

/// Handle the CTRL-^ key in the command-line mode and toggle the use of the
/// language :lmap mappings and/or Input Method.
static void command_line_toggle_langmap(CommandLineState *s)
{
  OptInt *b_im_ptr = buf_valid(s->b_im_ptr_buf) ? s->b_im_ptr : NULL;
  if (map_to_exists_mode("", MODE_LANGMAP, false)) {
    // ":lmap" mappings exists, toggle use of mappings.
    State ^= MODE_LANGMAP;
    if (b_im_ptr != NULL) {
      if (State & MODE_LANGMAP) {
        *b_im_ptr = B_IMODE_LMAP;
      } else {
        *b_im_ptr = B_IMODE_NONE;
      }
    }
  }

  if (b_im_ptr != NULL) {
    if (b_im_ptr == &curbuf->b_p_iminsert) {
      set_iminsert_global(curbuf);
    } else {
      set_imsearch_global(curbuf);
    }
  }
  ui_cursor_shape();                // may show different cursor shape
  // Show/unshow value of 'keymap' in status lines later.
  status_redraw_curbuf();
}

// command_line_insert_reg() is implemented in Rust (cmdline crate, keys.rs).

/// Handle the Left and Right mouse clicks in the command-line mode.
static void command_line_left_right_mouse(CommandLineState *s)
{
  if (s->c == K_LEFTRELEASE || s->c == K_RIGHTRELEASE) {
    s->ignore_drag_release = true;
  } else {
    s->ignore_drag_release = false;
  }

  ccline.cmdspos = rs_cmd_startcol();
  for (ccline.cmdpos = 0; ccline.cmdpos < ccline.cmdlen;
       ccline.cmdpos++) {
    int cells = rs_cmdline_charsize(ccline.cmdpos);
    if (mouse_row <= cmdline_row + ccline.cmdspos / Columns
        && mouse_col < ccline.cmdspos % Columns + cells) {
      break;
    }

    // Count ">" for double-wide char that doesn't fit.
    correct_screencol(ccline.cmdpos, cells, &ccline.cmdspos);
    ccline.cmdpos += utfc_ptr2len(ccline.cmdbuff + ccline.cmdpos) - 1;
    ccline.cmdspos += cells;
  }
}

// command_line_browse_history is now implemented via nvim_command_line_browse_history (below).

// command_line_handle_key is implemented in Rust (cmdline/keys.rs).


/// Trigger CursorMovedC autocommands.
static void may_trigger_cursormovedc(CommandLineState *s)
{
  if (ccline.cmdpos != s->prev_cmdpos) {
    trigger_cmd_autocmd(s->cmdline_type, EVENT_CURSORMOVEDC);
    ccline.redraw_state = MAX(ccline.redraw_state, kCmdRedrawPos);
  }
}

static int command_line_not_changed(CommandLineState *s)
{
  may_trigger_cursormovedc(s);
  s->prev_cmdpos = ccline.cmdpos;
  // Incremental searches for "/" and "?":
  // Enter command_line_not_changed() when a character has been read but the
  // command line did not change. Then we only search and redraw if something
  // changed in the past.
  // Enter command_line_changed() when the command line did change.
  if (!s->is_state.incsearch_postponed) {
    return 1;
  }
  return command_line_changed(s);
}


handle_T cmdpreview_get_bufnr(void)
{
  return cmdpreview_bufnr;
}

// C accessor for cmdpreview_ns (used by Rust)
int nvim_get_cmdpreview_ns(void) { return cmdpreview_ns; }

// Command preview helpers from Rust
extern int rs_cmdpreview_should_skip_buffer(int64_t buf_handle, int64_t preview_bufnr);
extern int rs_cmdpreview_needs_undo_restore(int64_t current_seq, int64_t saved_seq);

/// Sets up command preview buffer.
///
/// @return Pointer to command preview buffer if succeeded, NULL if failed.
static buf_T *cmdpreview_open_buf(void)
{
  buf_T *cmdpreview_buf = cmdpreview_bufnr ? buflist_findnr(cmdpreview_bufnr) : NULL;

  // If preview buffer doesn't exist, open one.
  if (cmdpreview_buf == NULL) {
    Error err = ERROR_INIT;
    handle_T bufnr = nvim_create_buf(false, true, &err);

    if (ERROR_SET(&err)) {
      return NULL;
    }

    cmdpreview_buf = buflist_findnr(bufnr);
  }

  // Preview buffer cannot preview itself!
  if (cmdpreview_buf == curbuf) {
    return NULL;
  }

  // Rename preview buffer.
  aco_save_T aco;
  aucmd_prepbuf(&aco, cmdpreview_buf);
  int retv = rename_buffer("[Preview]");
  aucmd_restbuf(&aco);

  if (retv == FAIL) {
    return NULL;
  }

  // Temporarily switch to preview buffer to set it up for previewing.
  aucmd_prepbuf(&aco, cmdpreview_buf);
  buf_clear();
  curbuf->b_p_ma = true;
  curbuf->b_p_ul = -1;
  curbuf->b_p_tw = 0;  // Reset 'textwidth' (was set by ftplugin)
  aucmd_restbuf(&aco);
  cmdpreview_bufnr = cmdpreview_buf->handle;

  return cmdpreview_buf;
}

/// Open command preview window if it's not already open.
/// Returns to original window after opening command preview window.
///
/// @param cmdpreview_buf Pointer to command preview buffer
///
/// @return Pointer to command preview window if succeeded, NULL if failed.
static win_T *cmdpreview_open_win(buf_T *cmdpreview_buf)
  FUNC_ATTR_NONNULL_ALL
{
  win_T *save_curwin = curwin;

  // Open preview window.
  if (win_split((int)p_cwh, WSP_BOT) == FAIL) {
    return NULL;
  }

  win_T *preview_win = curwin;
  Error err = ERROR_INIT;
  int result = OK;

  // Switch to preview buffer
  TRY_WRAP(&err, {
    result = do_buffer(DOBUF_GOTO, DOBUF_FIRST, FORWARD, cmdpreview_buf->handle, 0);
  });
  if (ERROR_SET(&err) || result == FAIL) {
    api_clear_error(&err);
    return NULL;
  }

  curwin->w_p_cul = false;
  curwin->w_p_cuc = false;
  curwin->w_p_spell = false;
  curwin->w_p_fen = false;

  win_enter(save_curwin, false);
  return preview_win;
}

/// Closes any open command preview windows.
static void cmdpreview_close_win(void)
{
  buf_T *buf = cmdpreview_bufnr ? buflist_findnr(cmdpreview_bufnr) : NULL;
  if (buf != NULL) {
    close_windows(buf, false);
  }
}

/// Save the undo state of a buffer for command preview.
static void cmdpreview_save_undo(CpUndoInfo *cp_undoinfo, buf_T *buf)
  FUNC_ATTR_NONNULL_ALL
{
  cp_undoinfo->save_b_u_synced = buf->b_u_synced;
  cp_undoinfo->save_b_u_oldhead = buf->b_u_oldhead;
  cp_undoinfo->save_b_u_newhead = buf->b_u_newhead;
  cp_undoinfo->save_b_u_curhead = buf->b_u_curhead;
  cp_undoinfo->save_b_u_numhead = buf->b_u_numhead;
  cp_undoinfo->save_b_u_seq_last = buf->b_u_seq_last;
  cp_undoinfo->save_b_u_save_nr_last = buf->b_u_save_nr_last;
  cp_undoinfo->save_b_u_seq_cur = buf->b_u_seq_cur;
  cp_undoinfo->save_b_u_time_cur = buf->b_u_time_cur;
  cp_undoinfo->save_b_u_save_nr_cur = buf->b_u_save_nr_cur;
  cp_undoinfo->save_b_u_line_ptr = buf->b_u_line_ptr;
  cp_undoinfo->save_b_u_line_lnum = buf->b_u_line_lnum;
  cp_undoinfo->save_b_u_line_colnr = buf->b_u_line_colnr;
}

/// Restore the undo state of a buffer for command preview.
static void cmdpreview_restore_undo(const CpUndoInfo *cp_undoinfo, buf_T *buf)
{
  buf->b_u_oldhead = cp_undoinfo->save_b_u_oldhead;
  buf->b_u_newhead = cp_undoinfo->save_b_u_newhead;
  buf->b_u_curhead = cp_undoinfo->save_b_u_curhead;
  buf->b_u_numhead = cp_undoinfo->save_b_u_numhead;
  buf->b_u_seq_last = cp_undoinfo->save_b_u_seq_last;
  buf->b_u_save_nr_last = cp_undoinfo->save_b_u_save_nr_last;
  buf->b_u_seq_cur = cp_undoinfo->save_b_u_seq_cur;
  buf->b_u_time_cur = cp_undoinfo->save_b_u_time_cur;
  buf->b_u_save_nr_cur = cp_undoinfo->save_b_u_save_nr_cur;
  buf->b_u_line_ptr = cp_undoinfo->save_b_u_line_ptr;
  buf->b_u_line_lnum = cp_undoinfo->save_b_u_line_lnum;
  buf->b_u_line_colnr = cp_undoinfo->save_b_u_line_colnr;
  if (buf->b_u_curhead == NULL) {
    buf->b_u_synced = cp_undoinfo->save_b_u_synced;
  }
}

/// Save current state and prepare windows and buffers for command preview.
static void cmdpreview_prepare(CpInfo *cpinfo)
  FUNC_ATTR_NONNULL_ALL
{
  Set(ptr_t) saved_bufs = SET_INIT;

  kv_init(cpinfo->buf_info);
  kv_init(cpinfo->win_info);

  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    buf_T *buf = win->w_buffer;

    // Don't save state of command preview buffer or preview window.
    // Use Rust helper to check if buffer should be skipped for preview.
    if (rs_cmdpreview_should_skip_buffer(buf->handle, cmdpreview_bufnr)) {
      continue;
    }

    if (!set_has(ptr_t, &saved_bufs, buf)) {
      CpBufInfo cp_bufinfo;
      cp_bufinfo.buf = buf;
      cp_bufinfo.save_b_p_ma = buf->b_p_ma;
      cp_bufinfo.save_b_p_ul = buf->b_p_ul;
      cp_bufinfo.save_b_changed = buf->b_changed;
      cp_bufinfo.save_b_op_start = buf->b_op_start;
      cp_bufinfo.save_b_op_end = buf->b_op_end;
      cp_bufinfo.save_changedtick = buf_get_changedtick(buf);
      cmdpreview_save_undo(&cp_bufinfo.undo_info, buf);
      kv_push(cpinfo->buf_info, cp_bufinfo);
      set_put(ptr_t, &saved_bufs, buf);

      u_clearall(buf);
      buf->b_p_ul = INT_MAX;  // Make sure we can undo all changes
    }

    CpWinInfo cp_wininfo;
    cp_wininfo.win = win;

    // Save window cursor position and viewstate
    cp_wininfo.save_w_cursor = win->w_cursor;
    rs_save_viewstate_win(win, &cp_wininfo.save_viewstate);

    // Save 'cursorline' and 'cursorcolumn'
    cp_wininfo.save_w_p_cul = win->w_p_cul;
    cp_wininfo.save_w_p_cuc = win->w_p_cuc;

    kv_push(cpinfo->win_info, cp_wininfo);

    win->w_p_cul = false;       // Disable 'cursorline' so it doesn't mess up the highlights
    win->w_p_cuc = false;       // Disable 'cursorcolumn' so it doesn't mess up the highlights
  }

  set_destroy(ptr_t, &saved_bufs);

  cpinfo->save_hls = p_hls;
  cpinfo->save_cmdmod = cmdmod;
  rs_win_size_save(&cpinfo->save_view);
  save_search_patterns();

  p_hls = false;                 // Don't show search highlighting during live substitution
  cmdmod.cmod_split = 0;         // Disable :leftabove/botright modifiers
  cmdmod.cmod_tab = 0;           // Disable :tab modifier
  cmdmod.cmod_flags |= CMOD_NOSWAPFILE;  // Disable swap for preview buffer

  u_sync(true);
}

/// Restore the state of buffers and windows for command preview.
static void cmdpreview_restore_state(CpInfo *cpinfo)
  FUNC_ATTR_NONNULL_ALL
{
  for (size_t i = 0; i < cpinfo->buf_info.size; i++) {
    CpBufInfo cp_bufinfo = cpinfo->buf_info.items[i];
    buf_T *buf = cp_bufinfo.buf;

    buf->b_changed = cp_bufinfo.save_b_changed;

    // Clear preview highlights.
    extmark_clear(buf, (uint32_t)cmdpreview_ns, 0, 0, MAXLNUM, MAXCOL);

    // Use Rust helper to check if undo restoration is needed.
    if (rs_cmdpreview_needs_undo_restore(buf->b_u_seq_cur,
                                         cp_bufinfo.undo_info.save_b_u_seq_cur)) {
      int count = 0;

      // Calculate how many undo steps are necessary to restore earlier state.
      for (u_header_T *uhp = buf->b_u_curhead ? buf->b_u_curhead : buf->b_u_newhead;
           uhp != NULL;
           uhp = uhp->uh_next.ptr, ++count) {}

      aco_save_T aco;
      aucmd_prepbuf(&aco, buf);
      // Ensure all the entries will be undone
      if (curbuf->b_u_synced == false) {
        u_sync(true);
      }
      // Undo invisibly. This also moves the cursor!
      if (!u_undo_and_forget(count, false)) {
        abort();
      }
      aucmd_restbuf(&aco);
    }

    u_blockfree(buf);
    cmdpreview_restore_undo(&cp_bufinfo.undo_info, buf);

    buf->b_op_start = cp_bufinfo.save_b_op_start;
    buf->b_op_end = cp_bufinfo.save_b_op_end;

    if (cp_bufinfo.save_changedtick != buf_get_changedtick(buf)) {
      buf_set_changedtick(buf, cp_bufinfo.save_changedtick);
    }

    buf->b_p_ul = cp_bufinfo.save_b_p_ul;        // Restore 'undolevels'
    buf->b_p_ma = cp_bufinfo.save_b_p_ma;        // Restore 'modifiable'
  }

  for (size_t i = 0; i < cpinfo->win_info.size; i++) {
    CpWinInfo cp_wininfo = cpinfo->win_info.items[i];
    win_T *win = cp_wininfo.win;

    // Restore window cursor position and viewstate
    win->w_cursor = cp_wininfo.save_w_cursor;
    rs_restore_viewstate_win(win, &cp_wininfo.save_viewstate);

    // Restore 'cursorline' and 'cursorcolumn'
    win->w_p_cul = cp_wininfo.save_w_p_cul;
    win->w_p_cuc = cp_wininfo.save_w_p_cuc;

    update_topline(win);
  }

  cmdmod = cpinfo->save_cmdmod;                // Restore cmdmod
  p_hls = cpinfo->save_hls;                    // Restore 'hlsearch'
  restore_search_patterns();           // Restore search patterns
  rs_win_size_restore(&cpinfo->save_view);        // Restore window sizes

  ga_clear(&cpinfo->save_view);
  kv_destroy(cpinfo->win_info);
  kv_destroy(cpinfo->buf_info);
}

/// Show 'inccommand' preview if command is previewable. It works like this:
///    1. Store current undo information so we can revert to current state later.
///    2. Execute the preview callback with the parsed command, preview buffer number and preview
///       namespace number as arguments. The preview callback sets the highlight and does the
///       changes required for the preview if needed.
///    3. Preview callback returns 0, 1 or 2. 0 means no preview is shown. 1 means preview is shown
///       but preview window doesn't need to be opened. 2 means preview is shown and preview window
///       needs to be opened if inccommand=split.
///    4. Use the return value of the preview callback to determine whether to
///       open the preview window or not and open preview window if needed.
///    5. If the return value of the preview callback is not 0, update the screen while the effects
///       of the preview are still in place.
///    6. Revert all changes made by the preview callback.
///
/// @return whether preview is shown or not.
static bool cmdpreview_may_show(CommandLineState *s)
{
  // Parse the command line and return if it fails.
  exarg_T ea;
  CmdParseInfo cmdinfo;
  // Copy the command line so we can modify it.
  int cmdpreview_type = 0;
  char *cmdline = xstrdup(ccline.cmdbuff);
  const char *errormsg = NULL;
  emsg_off++;  // Block errors when parsing the command line, and don't update v:errmsg
  if (!parse_cmdline(&cmdline, &ea, &cmdinfo, &errormsg)) {
    emsg_off--;
    goto end;
  }
  emsg_off--;

  // Check if command is previewable, if not, don't attempt to show preview
  if (!(ea.argt & EX_PREVIEW)) {
    undo_cmdmod(&cmdinfo.cmdmod);
    goto end;
  }

  // Cursor may be at the end of the message grid rather than at cmdspos.
  // Place it there in case preview callback flushes it. #30696
  cursorcmd();
  // Flush now: external cmdline may itself wish to update the screen which is
  // currently disallowed during cmdpreview (no longer needed in case that changes).
  cmdline_ui_flush();

  // Swap invalid command range if needed
  if ((ea.argt & EX_RANGE) && ea.line1 > ea.line2) {
    linenr_T lnum = ea.line1;
    ea.line1 = ea.line2;
    ea.line2 = lnum;
  }

  CpInfo cpinfo;
  bool icm_split = *p_icm == 's';  // inccommand=split
  buf_T *cmdpreview_buf = NULL;
  win_T *cmdpreview_win = NULL;

  emsg_silent++;                 // Block error reporting as the command may be incomplete,
                                 // but still update v:errmsg
  msg_silent++;                  // Block messages, namely ones that prompt
  block_autocmds();              // Block events

  // Save current state and prepare for command preview.
  cmdpreview_prepare(&cpinfo);

  // Open preview buffer if inccommand=split.
  if (icm_split && (cmdpreview_buf = cmdpreview_open_buf()) == NULL) {
    // Failed to create preview buffer, so disable preview.
    set_option_direct(kOptInccommand, STATIC_CSTR_AS_OPTVAL("nosplit"), 0, SID_NONE);
    icm_split = false;
  }
  // Setup preview namespace if it's not already set.
  if (!cmdpreview_ns) {
    cmdpreview_ns = (int)nvim_create_namespace((String)STRING_INIT);
  }

  // Set cmdpreview state.
  cmdpreview = true;

  // Execute the preview callback and use its return value to determine whether to show preview or
  // open the preview window. The preview callback also handles doing the changes and highlights for
  // the preview.
  Error err = ERROR_INIT;
  TRY_WRAP(&err, {
    cmdpreview_type = execute_cmd(&ea, &cmdinfo, true);
  });
  if (ERROR_SET(&err)) {
    api_clear_error(&err);
    cmdpreview_type = 0;
  }

  // If inccommand=split and preview callback returns 2, open preview window.
  if (icm_split && cmdpreview_type == 2
      && (cmdpreview_win = cmdpreview_open_win(cmdpreview_buf)) == NULL) {
    // If there's not enough room to open the preview window, just preview without the window.
    cmdpreview_type = 1;
  }

  // If preview callback return value is nonzero, update screen now.
  if (cmdpreview_type != 0) {
    int save_rd = RedrawingDisabled;
    RedrawingDisabled = 0;
    update_screen();
    RedrawingDisabled = save_rd;
  }

  // Close preview window if it's open.
  if (icm_split && cmdpreview_type == 2 && cmdpreview_win != NULL) {
    cmdpreview_close_win();
  }

  // Restore state.
  cmdpreview_restore_state(&cpinfo);

  unblock_autocmds();                  // Unblock events
  msg_silent--;                        // Unblock messages
  emsg_silent--;                       // Unblock error reporting
  redrawcmdline();
end:
  xfree(cmdline);
  return cmdpreview_type != 0;
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

static int command_line_changed(CommandLineState *s)
{
  const bool prev_cmdpreview = cmdpreview;
  if (s->firstc == ':'
      && current_sctx.sc_sid == 0    // only if interactive
      && *p_icm != NUL       // 'inccommand' is set
      && !exmode_active      // not in ex mode
      && cmdline_star == 0   // not typing a password
      && !vpeekc_any()
      && cmdpreview_may_show(s)) {
    // 'inccommand' preview has been shown.
  } else {
    cmdpreview = false;
    if (prev_cmdpreview) {
      // TODO(bfredl): add an immediate redraw flag for cmdline mode which will trigger
      // at next wait-for-input
      update_screen();  // Clear 'inccommand' preview.
    }
    if (s->xpc.xp_context == EXPAND_NOTHING && (KeyTyped || vpeekc() == NUL)) {
      rs_may_do_incsearch_highlighting(s->firstc, s->count, &s->is_state);
    }
  }

  if (ccline.cmdpos != s->prev_cmdpos
      || (s->prev_cmdbuff != NULL && strcmp(s->prev_cmdbuff, ccline.cmdbuff) != 0)) {
    // Trigger CmdlineChanged autocommands.
    do_autocmd_cmdlinechanged(s->firstc > 0 ? s->firstc : '-');
  }

  may_trigger_cursormovedc(s);

  if (p_arshape && !p_tbidi) {
    // Always redraw the whole command line to fix shaping and
    // right-left typing.  Not efficient, but it works.
    // Do it only when there are no characters left to read
    // to avoid useless intermediate redraws.
    // if cmdline is external the ui handles shaping, no redraw needed.
    if (!ui_has(kUICmdline) && vpeekc() == NUL) {
      redrawcmd();
    }
  }

  return 1;
}

/// Abandon the command line.
static void abandon_cmdline(void)
{
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
    save_cmdline(&save_ccline);
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
    restore_cmdline(&save_ccline);
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

// Give an error message for a command that isn't allowed while the cmdline
// window is open or editing the cmdline in another way.
void text_locked_msg(void)
{
  emsg(_(get_text_locked_msg()));
}

/// Check for text, window or buffer locked.
/// Give an error message and return true if something is locked.
bool text_or_buf_locked(void)
{
  if (text_locked()) {
    text_locked_msg();
    return true;
  }
  return curbuf_locked();
}

/// Check if "curbuf->b_ro_locked" or "allbuf_lock" is set and
/// return true when it is and give an error message.
bool curbuf_locked(void)
{
  if (curbuf->b_ro_locked > 0) {
    emsg(_(e_cannot_edit_other_buf));
    return true;
  }
  return allbuf_locked();
}

// Check if "allbuf_lock" is set and return true when it is and give an error
// message.
bool allbuf_locked(void)
{
  if (allbuf_lock > 0) {
    emsg(_("E811: Not allowed to change buffer information now"));
    return true;
  }
  return false;
}


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

/// Color expression cmdline using built-in expressions parser
///
/// @param[in]  colored_ccline  Command-line to color.
/// @param[out]  ret_ccline_colors  What should be colored.
///
/// Always colors the whole cmdline.
static void color_expr_cmdline(const CmdlineInfo *const colored_ccline,
                               ColoredCmdline *const ret_ccline_colors)
  FUNC_ATTR_NONNULL_ALL
{
  ParserLine parser_lines[] = {
    {
      .data = colored_ccline->cmdbuff,
      .size = strlen(colored_ccline->cmdbuff),
      .allocated = false,
    },
    { NULL, 0, false },
  };
  ParserLine *plines_p = parser_lines;
  ParserHighlight colors;
  kvi_init(colors);
  ParserState pstate;
  viml_parser_init(&pstate, parser_simple_get_line, &plines_p, &colors);
  ExprAST east = viml_pexpr_parse(&pstate, kExprFlagsDisallowEOC);
  viml_pexpr_free_ast(east);
  viml_parser_destroy(&pstate);
  kv_resize(ret_ccline_colors->colors, kv_size(colors));
  size_t prev_end = 0;
  for (size_t i = 0; i < kv_size(colors); i++) {
    const ParserHighlightChunk chunk = kv_A(colors, i);
    assert(chunk.start.col < INT_MAX);
    assert(chunk.end_col < INT_MAX);
    if (chunk.start.col != prev_end) {
      kv_push(ret_ccline_colors->colors, ((CmdlineColorChunk) {
        .start = (int)prev_end,
        .end = (int)chunk.start.col,
        .hl_id = 0,
      }));
    }
    kv_push(ret_ccline_colors->colors, ((CmdlineColorChunk) {
      .start = (int)chunk.start.col,
      .end = (int)chunk.end_col,
      .hl_id = syn_name2id(chunk.group),
    }));
    prev_end = chunk.end_col;
  }
  if (prev_end < (size_t)colored_ccline->cmdlen) {
    kv_push(ret_ccline_colors->colors, ((CmdlineColorChunk) {
      .start = (int)prev_end,
      .end = colored_ccline->cmdlen,
      .hl_id = 0,
    }));
  }
  kvi_destroy(colors);
}

/// Color command-line
///
/// Should use built-in command parser or user-specified one. Currently only the
/// latter is supported.
///
/// @param[in,out]  colored_ccline  Command-line to color. Also holds a cache:
///                                 if ->prompt_id and ->cmdbuff values happen
///                                 to be equal to those from colored_cmdline it
///                                 will just do nothing, assuming that ->colors
///                                 already contains needed data.
///
/// Always colors the whole cmdline.
///
/// @return true if draw_cmdline may proceed, false if it does not need anything
///         to do.
static bool color_cmdline(CmdlineInfo *colored_ccline)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
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

  ColoredCmdline *ccline_colors = &colored_ccline->last_colors;

  // Check whether result of the previous call is still valid.
  if (ccline_colors->prompt_id == colored_ccline->prompt_id
      && ccline_colors->cmdbuff != NULL
      && strcmp(ccline_colors->cmdbuff, colored_ccline->cmdbuff) == 0) {
    return ret;
  }

  kv_size(ccline_colors->colors) = 0;

  if (colored_ccline->cmdbuff == NULL || *colored_ccline->cmdbuff == NUL) {
    // Nothing to do, exiting.
    XFREE_CLEAR(ccline_colors->cmdbuff);
    return ret;
  }

  bool arg_allocated = false;
  typval_T arg = {
    .v_type = VAR_STRING,
    .vval.v_string = colored_ccline->cmdbuff,
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
  if (rs_should_reset_callback_errors(colored_ccline->prompt_id, prev_prompt_id)) {
    prev_prompt_errors = 0;
    prev_prompt_id = colored_ccline->prompt_id;
  } else if (rs_should_skip_coloring(colored_ccline->prompt_id, prev_prompt_id, prev_prompt_errors)) {
    // Skip coloring due to too many previous errors.
    goto color_cmdline_end;
  }
  if (colored_ccline->highlight_callback.type != kCallbackNone) {
    // Currently this should only happen while processing input() prompts.
    assert(colored_ccline->input_fn);
    color_cb = colored_ccline->highlight_callback;
  } else if (colored_ccline->cmdfirstc == ':') {
    TRY_WRAP(&err, {
      err_errmsg = N_("E5408: Unable to get g:Nvim_color_cmdline callback: %s");
      dgc_ret = tv_dict_get_callback(get_globvar_dict(), S_LEN("Nvim_color_cmdline"),
                                     &color_cb);
    });
    can_free_cb = true;
  } else if (colored_ccline->cmdfirstc == '=') {
    color_expr_cmdline(colored_ccline, ccline_colors);
  }
  if (ERROR_SET(&err) || !dgc_ret) {
    goto color_cmdline_error;
  }

  if (color_cb.type == kCallbackNone) {
    goto color_cmdline_end;
  }
  if (colored_ccline->cmdbuff[colored_ccline->cmdlen] != NUL) {
    arg_allocated = true;
    arg.vval.v_string = xmemdupz(colored_ccline->cmdbuff, (size_t)colored_ccline->cmdlen);
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
    } else if (!(prev_end <= start && start < colored_ccline->cmdlen)) {
      PRINT_ERRMSG(_("E5403: Chunk %i start %" PRIdVARNUMBER " not in range "
                     "[%" PRIdVARNUMBER ", %i)"),
                   i, start, prev_end, colored_ccline->cmdlen);
      goto color_cmdline_error;
    } else if (utf8len_tab_zero[(uint8_t)colored_ccline->cmdbuff[start]] == 0) {
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
    } else if (!(start < end && end <= colored_ccline->cmdlen)) {
      PRINT_ERRMSG(_("E5404: Chunk %i end %" PRIdVARNUMBER " not in range "
                     "(%" PRIdVARNUMBER ", %i]"),
                   i, end, start, colored_ccline->cmdlen);
      goto color_cmdline_error;
    } else if (end < colored_ccline->cmdlen
               && (utf8len_tab_zero[(uint8_t)colored_ccline->cmdbuff[end]]
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
  if (prev_end < colored_ccline->cmdlen) {
    kv_push(ccline_colors->colors, ((CmdlineColorChunk) {
      .start = (int)prev_end,
      .end = colored_ccline->cmdlen,
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
  ccline_colors->prompt_id = colored_ccline->prompt_id;
  if (arg_allocated) {
    ccline_colors->cmdbuff = arg.vval.v_string;
  } else {
    ccline_colors->cmdbuff = xmemdupz(colored_ccline->cmdbuff, (size_t)colored_ccline->cmdlen);
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

// Draw part of the cmdline at the current cursor position.  But draw stars
// when cmdline_star is true.
// draw_cmdline() is implemented in Rust (cmdline crate, screen.rs).

static void ui_ext_cmdline_show(CmdlineInfo *line)
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

/// Extra redrawing needed for redraw! and on ui_attach.
void cmdline_screen_cleared(void)
{
  if (!ui_has(kUICmdline)) {
    return;
  }

  if (cmdline_block.size) {
    ui_call_cmdline_block_show(cmdline_block);
  }

  int prev_level = ccline.level - 1;
  CmdlineInfo *line = ccline.prev_ccline;
  while (prev_level > 0 && line) {
    if (line->level == prev_level) {
      // don't redraw a cmdline already shown in the cmdline window
      if (prev_level != cmdwin_level) {
        line->redraw_state = kCmdRedrawAll;
      }
      prev_level--;
    }
    line = line->prev_ccline;
  }
  redrawcmd();
}

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
        ui_ext_cmdline_show(line);
      } else if (redraw_state == kCmdRedrawPos && cmdline_was_last_drawn) {
        ui_call_cmdline_pos(line->cmdpos, line->level);
      }
      level--;
    }
    line = line->prev_ccline;
  }
}

// Put a character on the command line.  Shifts the following text to the
// right when "shift" is true.  Used for CTRL-V, CTRL-K, etc.
// "c" must be printable (fit in one display cell)!
void putcmdline(char c, bool shift)
{
  if (cmd_silent) {
    return;
  }
  if (!ui_has(kUICmdline)) {
    msg_no_more = true;
    msg_putchar(c);
    if (shift) {
      draw_cmdline(ccline.cmdpos, ccline.cmdlen - ccline.cmdpos);
    }
    msg_no_more = false;
  } else if (ccline.redraw_state != kCmdRedrawAll) {
    char charbuf[2] = { c, 0 };
    ui_call_cmdline_special_char(cstr_as_string(charbuf), shift,
                                 ccline.level);
  }
  cursorcmd();
  ccline.special_char = c;
  ccline.special_shift = shift;
  ui_cursor_shape();
}

/// Undo a putcmdline(c, false).
void unputcmdline(void)
{
  if (cmd_silent) {
    return;
  }
  msg_no_more = true;
  if (ccline.cmdlen == ccline.cmdpos && !ui_has(kUICmdline)) {
    msg_putchar(' ');
  } else {
    draw_cmdline(ccline.cmdpos, utfc_ptr2len(ccline.cmdbuff + ccline.cmdpos));
  }
  msg_no_more = false;
  cursorcmd();
  ccline.special_char = NUL;
  ui_cursor_shape();
}

// put_on_cmdline() is implemented in Rust (cmdline crate, edit.rs).

/// Save ccline, because obtaining the "=" register may execute "normal :cmd"
/// and overwrite it.
static void save_cmdline(CmdlineInfo *ccp)
{
  *ccp = ccline;
  CLEAR_FIELD(ccline);
  ccline.prev_ccline = ccp;
  ccline.cmdbuff = NULL;  // signal that ccline is not in use
}

/// Restore ccline after it has been saved with save_cmdline().
static void restore_cmdline(CmdlineInfo *ccp)
  FUNC_ATTR_NONNULL_ALL
{
  ccline = *ccp;
}

/// Paste a yank register into the command line.
/// Used by CTRL-R command in command-line mode.
/// insert_reg() can't be used here, because special characters from the
/// register contents will be interpreted as commands.
///
/// @param regname   Register name.
/// @param literally Insert text literally instead of "as typed".
/// @param remcr     When true, remove trailing CR.
///
/// @returns FAIL for failure, OK otherwise
static bool cmdline_paste(int regname, bool literally, bool remcr)
{
  char *arg;
  bool allocated;

  // check for valid regname; also accept special characters for CTRL-R in
  // the command line
  if (regname != Ctrl_F && regname != Ctrl_P && regname != Ctrl_W
      && regname != Ctrl_A && regname != Ctrl_L
      && !valid_yank_reg(regname, false)) {
    return FAIL;
  }

  // A register containing CTRL-R can cause an endless loop.  Allow using
  // CTRL-C to break the loop.
  line_breakcheck();
  if (got_int) {
    return FAIL;
  }

  // Need to  set "textlock" to avoid nasty things like going to another
  // buffer when evaluating an expression.
  textlock++;
  const bool i = get_spec_reg(regname, &arg, &allocated, true);
  textlock--;

  if (i) {
    // Got the value of a special register in "arg".
    if (arg == NULL) {
      return FAIL;
    }

    // When 'incsearch' is set and CTRL-R CTRL-W used: skip the duplicate
    // part of the word.
    char *p = arg;
    if (p_is && regname == Ctrl_W) {
      char *w;
      int len;

      // Locate start of last word in the cmd buffer.
      for (w = ccline.cmdbuff + ccline.cmdpos; w > ccline.cmdbuff;) {
        len = utf_head_off(ccline.cmdbuff, w - 1) + 1;
        if (!vim_iswordc(utf_ptr2char(w - len))) {
          break;
        }
        w -= len;
      }
      len = (int)((ccline.cmdbuff + ccline.cmdpos) - w);
      if (p_ic ? STRNICMP(w, arg, len) == 0 : strncmp(w, arg, (size_t)len) == 0) {
        p += len;
      }
    }

    cmdline_paste_str(p, literally);
    if (allocated) {
      xfree(arg);
    }
    return OK;
  }

  return cmdline_paste_reg(regname, literally, remcr);
}

// cmdline_paste_str() and redrawcmdline() are implemented in Rust (cmdline crate).
// redrawcmd() is also implemented in Rust (cmdline crate, screen.rs).



/// Get a pointer to the current command line info.
CmdlineInfo *get_cmdline_info(void)
{
  return &ccline;
}

unsigned get_cmdline_last_prompt_id(void)
{
  return last_prompt_id;
}

/// Get pointer to the command line info to use. save_cmdline() may clear
/// ccline and put the previous value in ccline.prev_ccline.
static CmdlineInfo *get_ccline_ptr(void)
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

/// Get the current command-line type.
/// Returns ':' or '/' or '?' or '@' or '>' or '-'
/// Only works when the command line is being edited.
/// Returns NUL when something is wrong.
static int get_cmdline_type(void)
{
  CmdlineInfo *p = get_ccline_ptr();

  if (p == NULL) {
    return NUL;
  }
  if (p->cmdfirstc == NUL) {
    return (p->input_fn) ? '@' : '-';
  }
  return p->cmdfirstc;
}

/// Get the current command-line completion pattern.
static char *get_cmdline_completion_pattern(void)
{
  if (cmdline_star > 0) {
    return NULL;
  }

  CmdlineInfo *p = get_ccline_ptr();
  if (p == NULL || p->xpc == NULL) {
    return NULL;
  }

  int xp_context = p->xpc->xp_context;
  if (xp_context == EXPAND_NOTHING) {
    set_expand_context(p->xpc);
    xp_context = p->xpc->xp_context;
    p->xpc->xp_context = EXPAND_NOTHING;
  }
  if (xp_context == EXPAND_UNSUCCESSFUL) {
    return NULL;
  }

  char *compl_pat = p->xpc->xp_pattern;
  if (compl_pat == NULL) {
    return NULL;
  }

  return xstrdup(compl_pat);
}

/// Get the command-line completion type.
static char *get_cmdline_completion(void)
{
  if (cmdline_star > 0) {
    return NULL;
  }

  CmdlineInfo *p = get_ccline_ptr();
  if (p == NULL || p->xpc == NULL) {
    return NULL;
  }

  int xp_context = p->xpc->xp_context;
  if (xp_context == EXPAND_NOTHING) {
    set_expand_context(p->xpc);
    xp_context = p->xpc->xp_context;
    p->xpc->xp_context = EXPAND_NOTHING;
  }
  if (xp_context == EXPAND_UNSUCCESSFUL) {
    return NULL;
  }

  return cmdcomplete_type_to_str(xp_context, p->xpc->xp_arg);
}

/// "getcmdcomplpat()" function
void f_getcmdcomplpat(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = get_cmdline_completion_pattern();
}

/// "getcmdcompltype()" function
void f_getcmdcompltype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = get_cmdline_completion();
}

/// Set the command line str to "str".
/// @return  1 when failed, 0 when OK.
static int set_cmdline_str(const char *str, int pos)
{
  CmdlineInfo *p = get_ccline_ptr();

  if (p == NULL) {
    return 1;
  }

  int len = (int)strlen(str);
  realloc_cmdbuff(len + 1);
  p->cmdlen = len;
  STRCPY(p->cmdbuff, str);

  // Use Rust helper to clamp position to valid range.
  p->cmdpos = rs_clamp_cmdpos(pos, p->cmdlen);
  new_cmdpos = p->cmdpos;

  redrawcmd();

  // Trigger CmdlineChanged autocommands.
  do_autocmd_cmdlinechanged(get_cmdline_type());

  return 0;
}

/// Set the command line byte position to "pos".  Zero is the first position.
/// Only works when the command line is being edited.
/// @return  1 when failed, 0 when OK.
static int set_cmdline_pos(int pos)
{
  CmdlineInfo *p = get_ccline_ptr();

  if (p == NULL) {
    return 1;
  }

  // The position is not set directly but after CTRL-\ e or CTRL-R = has
  // changed the command line.
  new_cmdpos = MAX(0, pos);

  return 0;
}

/// "setcmdline()" function
void f_setcmdline(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (tv_check_for_string_arg(argvars, 0) == FAIL
      || tv_check_for_opt_number_arg(argvars, 1) == FAIL) {
    return;
  }

  int pos = -1;
  if (argvars[1].v_type != VAR_UNKNOWN) {
    bool error = false;

    pos = (int)tv_get_number_chk(&argvars[1], &error) - 1;
    if (error) {
      return;
    }
    if (pos < 0) {
      emsg(_(e_positive));
      return;
    }
  }

  // Use tv_get_string() to handle a NULL string like an empty string.
  rettv->vval.v_number = set_cmdline_str(tv_get_string(&argvars[0]), pos);
}

/// "setcmdpos()" function
void f_setcmdpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const int pos = (int)tv_get_number(&argvars[0]) - 1;

  if (pos >= 0) {
    rettv->vval.v_number = set_cmdline_pos(pos);
  }
}

// C accessor for ccline.cmdfirstc (used by Rust)
int nvim_get_ccline_cmdfirstc(void) { return ccline.cmdfirstc; }


void cmdline_init(void)
{
  CLEAR_FIELD(ccline);
}

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

/// Open a window on the current command line and history.  Allow editing in
/// the window.  Returns when the window is closed.
/// Returns:
///     CR       if the command is to be executed
///     Ctrl_C   if it is to be abandoned
///     K_IGNORE if editing continues
static int open_cmdwin(void)
{
  bufref_T old_curbuf;
  bufref_T bufref;
  win_T *old_curwin = curwin;
  int i;
  garray_T winsizes;
  int save_restart_edit = restart_edit;
  int save_State = State;
  bool save_exmode = exmode_active;
  bool save_cmdmsg_rl = cmdmsg_rl;

  // Use Rust helper to check if cmdwin can be opened.
  // Can't do this when text or buffer is locked, recursively, or typing a password.
  if (rs_cmdwin_can_open(cmdwin_type != 0, text_or_buf_locked(), cmdline_star) != 0) {
    beep_flush();
    return K_IGNORE;
  }

  set_bufref(&old_curbuf, curbuf);

  // Save current window sizes.
  rs_win_size_save(&winsizes);

  // When using completion in Insert mode with <C-R>=<C-F> one can open the
  // command line window, but we don't want the popup menu then.
  pum_undisplay(true);

  // don't use a new tab page
  cmdmod.cmod_tab = 0;
  cmdmod.cmod_flags |= CMOD_NOSWAPFILE;

  // Create a window for the command-line buffer.
  if (win_split((int)p_cwh, WSP_BOT) == FAIL) {
    beep_flush();
    ga_clear(&winsizes);
    return K_IGNORE;
  }
  // win_split() autocommands may have messed with the old window or buffer.
  // Treat it as abandoning this command-line.
  // Use Rust helper for validation check.
  if (rs_cmdwin_split_invalid(rs_win_valid(old_curwin), curwin == old_curwin,
                              bufref_valid(&old_curbuf),
                              old_curwin->w_buffer != old_curbuf.br_buf)) {
    beep_flush();
    ga_clear(&winsizes);
    return Ctrl_C;
  }
  // Don't let quitting the More prompt make this fail.
  got_int = false;

  // Set "cmdwin_..." variables before any autocommands may mess things up.
  cmdwin_type = get_cmdline_type();
  cmdwin_level = ccline.level;
  cmdwin_win = curwin;
  cmdwin_old_curwin = old_curwin;

  // Create empty command-line buffer.  Be especially cautious of BufLeave
  // autocommands from do_ecmd(), as cmdwin restrictions do not apply to them!
  const int newbuf_status = buf_open_scratch(0, NULL);
  const bool cmdwin_valid = rs_win_valid(cmdwin_win);
  // Use Rust helper for buffer creation validation.
  if (rs_cmdwin_buffer_invalid(newbuf_status == OK, cmdwin_valid, curwin == cmdwin_win,
                               rs_win_valid(old_curwin), bufref_valid(&old_curbuf),
                               old_curwin->w_buffer != old_curbuf.br_buf)) {
    if (newbuf_status == OK) {
      set_bufref(&bufref, curbuf);
    }
    if (cmdwin_valid && !rs_last_window(cmdwin_win)) {
      win_close(cmdwin_win, true, false);
    }
    // win_close() autocommands may have already deleted the buffer.
    if (newbuf_status == OK && bufref_valid(&bufref) && bufref.br_buf != curbuf) {
      close_buffer(NULL, bufref.br_buf, DOBUF_WIPE, false, false);
    }

    cmdwin_type = 0;
    cmdwin_level = 0;
    cmdwin_win = NULL;
    cmdwin_old_curwin = NULL;
    beep_flush();
    ga_clear(&winsizes);
    return Ctrl_C;
  }
  cmdwin_buf = curbuf;

  // Command-line buffer has bufhidden=wipe, unlike a true "scratch" buffer.
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("wipe"), OPT_LOCAL);
  curbuf->b_p_ma = true;
  curwin->w_p_fen = false;
  curwin->w_p_rl = cmdmsg_rl;
  cmdmsg_rl = false;

  // Don't allow switching to another buffer.
  curbuf->b_ro_locked++;

  // Showing the prompt may have set need_wait_return, reset it.
  need_wait_return = false;

  // Use Rust helper to get history type and check if Tab mapping/filetype needed.
  const int histtype = rs_cmdwin_to_hist_type(cmdwin_type);
  if (rs_cmdwin_needs_tab_mapping(histtype, p_wc)) {
    add_map("<Tab>", "<C-X><C-V>", MODE_INSERT, true);
    add_map("<Tab>", "a<C-X><C-V>", MODE_NORMAL, true);
  }
  if (rs_cmdwin_needs_vim_filetype(histtype)) {
    set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("vim"), OPT_LOCAL);
  }
  curbuf->b_ro_locked--;

  // Reset 'textwidth' after setting 'filetype' (the Vim filetype plugin
  // sets 'textwidth' to 78).
  curbuf->b_p_tw = 0;

  // Fill the buffer with the history.
  init_history();
  if (get_hislen() > 0 && histtype != HIST_INVALID) {
    i = *get_hisidx(histtype);
    if (i >= 0) {
      linenr_T lnum = 0;
      do {
        if (++i == get_hislen()) {
          i = 0;
        }
        if (get_histentry(histtype)[i].hisstr != NULL) {
          ml_append(lnum++, get_histentry(histtype)[i].hisstr, 0, false);
        }
      } while (i != *get_hisidx(histtype));
    }
  }

  // Replace the empty last line with the current command-line and put the
  // cursor there.
  ml_replace(curbuf->b_ml.ml_line_count, ccline.cmdbuff, true);
  curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
  curwin->w_cursor.col = ccline.cmdpos;
  changed_line_abv_curs();
  invalidate_botline(curwin);
  ui_ext_cmdline_hide(false);
  redraw_later(curwin, UPD_SOME_VALID);

  // No Ex mode here!
  exmode_active = false;

  State = MODE_NORMAL;
  setmouse();
  rs_clear_showcmd();

  // Reset here so it can be set by a CmdwinEnter autocommand.
  cmdwin_result = 0;

  // Trigger CmdwinEnter autocommands.
  trigger_cmd_autocmd(cmdwin_type, EVENT_CMDWINENTER);
  if (restart_edit != 0) {  // autocmd with ":startinsert"
    stuffcharReadbuff(K_NOP);
  }

  i = RedrawingDisabled;
  RedrawingDisabled = 0;
  int save_count = save_batch_count();

  // Call the main loop until <CR> or CTRL-C is typed.
  normal_enter(true, false);

  RedrawingDisabled = i;
  restore_batch_count(save_count);

  const bool save_KeyTyped = KeyTyped;

  // Trigger CmdwinLeave autocommands.
  trigger_cmd_autocmd(cmdwin_type, EVENT_CMDWINLEAVE);

  // Restore KeyTyped in case it is modified by autocommands
  KeyTyped = save_KeyTyped;

  cmdwin_type = 0;
  cmdwin_level = 0;
  cmdwin_buf = NULL;
  cmdwin_win = NULL;
  cmdwin_old_curwin = NULL;

  exmode_active = save_exmode;

  // Safety check: The old window or buffer was changed or deleted: It's a bug
  // when this happens! Use Rust helper for validation.
  if (rs_cmdwin_cleanup_had_error(rs_win_valid(old_curwin), bufref_valid(&old_curbuf),
                                  old_curwin->w_buffer != old_curbuf.br_buf)) {
    cmdwin_result = Ctrl_C;
    emsg(_(e_active_window_or_buffer_changed_or_deleted));
  } else {
    win_T *wp;
    // autocmds may abort script processing
    if (aborting() && cmdwin_result != K_IGNORE) {
      cmdwin_result = Ctrl_C;
    }
    // Set the new command line from the cmdline buffer.
    dealloc_cmdbuff();

    if (cmdwin_result == K_XF1 || cmdwin_result == K_XF2) {  // :qa[!] typed
      const char *p = (cmdwin_result == K_XF2) ? "qa" : "qa!";
      size_t plen = (cmdwin_result == K_XF2) ? 2 : 3;

      if (histtype == HIST_CMD) {
        // Execute the command directly.
        ccline.cmdbuff = xmemdupz(p, plen);
        ccline.cmdlen = (int)plen;
        ccline.cmdbufflen = (int)plen + 1;
        cmdwin_result = CAR;
      } else {
        // First need to cancel what we were doing.
        stuffcharReadbuff(':');
        stuffReadbuff(p);
        stuffcharReadbuff(CAR);
      }
    } else if (cmdwin_result == Ctrl_C) {
      // :q or :close, don't execute any command
      // and don't modify the cmd window.
      ccline.cmdbuff = NULL;
    } else {
      ccline.cmdlen = get_cursor_line_len();
      ccline.cmdbufflen = ccline.cmdlen + 1;
      ccline.cmdbuff = xstrnsave(get_cursor_line_ptr(), (size_t)ccline.cmdlen);
    }

    if (ccline.cmdbuff == NULL) {
      ccline.cmdbuff = xmemdupz("", 0);
      ccline.cmdlen = 0;
      ccline.cmdbufflen = 1;
      ccline.cmdpos = 0;
      cmdwin_result = Ctrl_C;
    } else {
      ccline.cmdpos = curwin->w_cursor.col;
      // If the cursor is on the last character, it probably should be after it.
      if (ccline.cmdpos == ccline.cmdlen - 1 || ccline.cmdpos > ccline.cmdlen) {
        ccline.cmdpos = ccline.cmdlen;
      }
      if (cmdwin_result == K_IGNORE) {
        ccline.cmdspos = cmd_screencol(ccline.cmdpos);
        redrawcmd();
      }
    }

    // Avoid command-line window first character being concealed.
    curwin->w_p_cole = 0;
    // First go back to the original window.
    wp = curwin;
    set_bufref(&bufref, curbuf);
    skip_win_fix_cursor = true;
    win_goto(old_curwin);

    // win_goto() may trigger an autocommand that already closes the
    // cmdline window.
    if (rs_win_valid(wp) && wp != curwin) {
      win_close(wp, true, false);
    }

    // win_close() may have already wiped the buffer when 'bh' is
    // set to 'wipe', autocommands may have closed other windows
    if (bufref_valid(&bufref) && bufref.br_buf != curbuf) {
      close_buffer(NULL, bufref.br_buf, DOBUF_WIPE, false, false);
    }

    // Restore window sizes.
    rs_win_size_restore(&winsizes);
    skip_win_fix_cursor = false;
  }

  ga_clear(&winsizes);
  restart_edit = save_restart_edit;
  cmdmsg_rl = save_cmdmsg_rl;

  State = save_State;
  may_trigger_modechanged();
  setmouse();
  setcursor();

  return cmdwin_result;
}


/// Get script string
///
/// Used for commands which accept either `:command script` or
///
///     :command << endmarker
///       script
///     endmarker
///
/// @param  eap  Command being run.
/// @param[out]  lenp  Location where length of resulting string is saved. Will
///                    be set to zero when skipping.
///
/// @return [allocated] NULL or script. Does not show any error messages.
///                     NULL is returned when skipping and on error.
char *script_get(exarg_T *const eap, size_t *const lenp)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_MALLOC
{
  char *cmd = eap->arg;

  if (cmd[0] != '<' || cmd[1] != '<' || eap->ea_getline == NULL) {
    *lenp = strlen(eap->arg);
    return eap->skip ? NULL : xmemdupz(eap->arg, *lenp);
  }
  cmd += 2;

  garray_T ga = { .ga_data = NULL, .ga_len = 0 };

  list_T *const l = heredoc_get(eap, cmd, true);
  if (l == NULL) {
    return NULL;
  }

  if (!eap->skip) {
    ga_init(&ga, 1, 0x400);
  }

  TV_LIST_ITER_CONST(l, li, {
    if (!eap->skip) {
      ga_concat(&ga, tv_get_string(TV_LIST_ITEM_TV(li)));
      ga_append(&ga, '\n');
    }
  });
  *lenp = (size_t)ga.ga_len;  // Set length without trailing NUL.
  if (!eap->skip) {
    ga_append(&ga, NUL);
  }

  tv_list_free(l);
  return (char *)ga.ga_data;
}

/// This function is used by f_input() and f_inputdialog() functions. The third
/// argument to f_input() specifies the type of completion to use at the
/// prompt. The third argument to f_inputdialog() specifies the value to return
/// when the user cancels the prompt.
void get_user_input(const typval_T *const argvars, typval_T *const rettv, const bool inputdialog,
                    const bool secret)
  FUNC_ATTR_NONNULL_ALL
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  if (cmdpreview) {
    return;
  }

  const char *prompt;
  const char *defstr = "";
  typval_T *cancelreturn = NULL;
  typval_T cancelreturn_strarg2 = TV_INITIAL_VALUE;
  const char *xp_name = NULL;
  Callback input_callback = { .type = kCallbackNone };
  char prompt_buf[NUMBUFLEN];
  char defstr_buf[NUMBUFLEN];
  char cancelreturn_buf[NUMBUFLEN];
  char xp_name_buf[NUMBUFLEN];
  char def[1] = { 0 };
  if (argvars[0].v_type == VAR_DICT) {
    if (argvars[1].v_type != VAR_UNKNOWN) {
      emsg(_("E5050: {opts} must be the only argument"));
      return;
    }
    dict_T *const dict = argvars[0].vval.v_dict;
    prompt = tv_dict_get_string_buf_chk(dict, S_LEN("prompt"), prompt_buf, "");
    if (prompt == NULL) {
      return;
    }
    defstr = tv_dict_get_string_buf_chk(dict, S_LEN("default"), defstr_buf, "");
    if (defstr == NULL) {
      return;
    }
    dictitem_T *cancelreturn_di = tv_dict_find(dict, S_LEN("cancelreturn"));
    if (cancelreturn_di != NULL) {
      cancelreturn = &cancelreturn_di->di_tv;
    }
    xp_name = tv_dict_get_string_buf_chk(dict, S_LEN("completion"),
                                         xp_name_buf, def);
    if (xp_name == NULL) {  // error
      return;
    }
    if (xp_name == def) {  // default to NULL
      xp_name = NULL;
    }
    if (!tv_dict_get_callback(dict, S_LEN("highlight"), &input_callback)) {
      return;
    }
  } else {
    prompt = tv_get_string_buf_chk(&argvars[0], prompt_buf);
    if (prompt == NULL) {
      return;
    }
    if (argvars[1].v_type != VAR_UNKNOWN) {
      defstr = tv_get_string_buf_chk(&argvars[1], defstr_buf);
      if (defstr == NULL) {
        return;
      }
      if (argvars[2].v_type != VAR_UNKNOWN) {
        const char *const strarg2 = tv_get_string_buf_chk(&argvars[2], cancelreturn_buf);
        if (strarg2 == NULL) {
          return;
        }
        if (inputdialog) {
          cancelreturn_strarg2.v_type = VAR_STRING;
          cancelreturn_strarg2.vval.v_string = (char *)strarg2;
          cancelreturn = &cancelreturn_strarg2;
        } else {
          xp_name = strarg2;
        }
      }
    }
  }

  int xp_type = EXPAND_NOTHING;
  char *xp_arg = NULL;
  if (xp_name != NULL) {
    // input() with a third argument: completion
    const int xp_namelen = (int)strlen(xp_name);

    uint32_t argt = 0;
    if (parse_compl_arg(xp_name, xp_namelen, &xp_type,
                        &argt, &xp_arg) == FAIL) {
      return;
    }
  }

  // Only the part of the message after the last NL is considered as
  // prompt for the command line, unlsess cmdline is externalized
  const char *p = prompt;
  if (!ui_has(kUICmdline)) {
    const char *lastnl = strrchr(prompt, '\n');
    if (lastnl != NULL) {
      p = lastnl + 1;
      msg_start();
      msg_clr_eos();
      msg_puts_len(prompt, p - prompt, rs_get_echo_hl_id(), false);
      msg_didout = false;
      msg_starthere();
    }
  }
  cmdline_row = msg_row;

  stuffReadbuffSpec(defstr);

  const int save_ex_normal_busy = ex_normal_busy;
  ex_normal_busy = 0;
  rettv->vval.v_string = getcmdline_prompt(secret ? NUL : '@', p, rs_get_echo_hl_id(),
                                           xp_type, xp_arg, input_callback, false, NULL);
  ex_normal_busy = save_ex_normal_busy;
  callback_free(&input_callback);

  if (rettv->vval.v_string == NULL && cancelreturn != NULL) {
    tv_copy(cancelreturn, rettv);
  }

  xfree(xp_arg);

  // Since the user typed this, no need to wait for return.
  need_wait_return = false;
  msg_didout = false;
}

/// "wildtrigger()" function
void f_wildtrigger(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (!(State & MODE_CMDLINE) || char_avail()
      || wild_menu_showing
      || cmdline_pum_active()) {
    return;
  }

  int cmd_type = get_cmdline_type();

  if (cmd_type == ':' || cmd_type == '/' || cmd_type == '?') {
    // Add K_WILD as a single special key
    uint8_t key_string[4];
    key_string[0] = K_SPECIAL;
    key_string[1] = KS_EXTRA;
    key_string[2] = KE_WILD;
    key_string[3] = NUL;

    // Insert it into the typeahead buffer
    ins_typebuf((char *)key_string, REMAP_NONE, 0, true, false);
  }
}

// C accessors for Rust to read ccline fields
int nvim_get_ccline_overstrike(void)
{
  return ccline.overstrike;
}

int nvim_get_ccline_cmdpos(void)
{
  return ccline.cmdpos;
}

int nvim_get_ccline_cmdlen(void)
{
  return ccline.cmdlen;
}

// C accessor for cmdwin_type global
int nvim_get_cmdwin_type(void)
{
  return cmdwin_type;
}

// C accessor wrapper for static get_cmdline_type()
int nvim_get_cmdline_type(void)
{
  return get_cmdline_type();
}

// C accessor for textlock global
int nvim_get_textlock(void)
{
  return textlock;
}

// C accessor for e_cmdwin error message
const char *nvim_get_e_cmdwin(void)
{
  return e_cmdwin;
}

// C accessor for e_textlock error message
const char *nvim_get_e_textlock(void)
{
  return e_textlock;
}

// Additional C accessors for Rust cmdline state management
int nvim_get_ccline_cmdspos(void)
{
  return ccline.cmdspos;
}

int nvim_get_ccline_cmdindent(void)
{
  return ccline.cmdindent;
}

unsigned int nvim_get_ccline_prompt_id(void)
{
  return ccline.prompt_id;
}

int nvim_get_ccline_level(void)
{
  return ccline.level;
}

int nvim_get_ccline_input_fn(void)
{
  return ccline.input_fn;
}

char *nvim_get_ccline_cmdbuff(void)
{
  return ccline.cmdbuff;
}

int nvim_get_ccline_cmdbufflen(void)
{
  return ccline.cmdbufflen;
}

// Setters for Rust to update ccline state
void nvim_set_ccline_cmdpos(int pos)
{
  ccline.cmdpos = pos;
}

void nvim_set_ccline_cmdlen(int len)
{
  ccline.cmdlen = len;
}

void nvim_set_ccline_cmdspos(int spos)
{
  ccline.cmdspos = spos;
}

void nvim_set_ccline_overstrike(int overstrike)
{
  ccline.overstrike = overstrike;
}

// Additional accessors for Phase 13.1

int nvim_get_ccline_redraw_state(void)
{
  return (int)ccline.redraw_state;
}

void nvim_set_ccline_redraw_state(int state)
{
  ccline.redraw_state = (CmdRedraw)state;
}

int nvim_get_ccline_special_char(void)
{
  return ccline.special_char;
}

void nvim_set_ccline_special_char(int c)
{
  ccline.special_char = (char)c;
}

int nvim_get_ccline_special_shift(void)
{
  return ccline.special_shift;
}

void nvim_set_ccline_special_shift(int shift)
{
  ccline.special_shift = shift;
}

int nvim_get_ccline_one_key(void)
{
  return ccline.one_key;
}

void nvim_set_ccline_one_key(int one_key)
{
  ccline.one_key = one_key;
}

int nvim_get_ccline_hl_id(void)
{
  return ccline.hl_id;
}

void nvim_set_ccline_hl_id(int hl_id)
{
  ccline.hl_id = hl_id;
}

int nvim_get_ccline_xp_context(void)
{
  return ccline.xp_context;
}

void nvim_set_ccline_xp_context(int context)
{
  ccline.xp_context = context;
}

char *nvim_get_ccline_cmdprompt(void)
{
  return ccline.cmdprompt;
}

void nvim_set_ccline_cmdprompt(char *prompt)
{
  ccline.cmdprompt = prompt;
}

void nvim_set_ccline_cmdindent(int indent)
{
  ccline.cmdindent = indent;
}

void nvim_set_ccline_cmdfirstc(int firstc)
{
  ccline.cmdfirstc = firstc;
}

// Accessors for history browsing (Phase 1 cmdline migration)

void nvim_set_ccline_cmdbuff_at(int idx, char val)
{
  ccline.cmdbuff[idx] = val;
}

void nvim_strcpy_cmdbuff(const char *src)
{
  if (src != NULL && ccline.cmdbuff != NULL) {
    STRCPY(ccline.cmdbuff, src);
  }
}

// Accessors for screen position calculations (Phase 13.2)

int nvim_get_columns(void)
{
  return Columns;
}

int nvim_get_rows(void)
{
  return Rows;
}

int nvim_get_key_typed(void)
{
  return KeyTyped;
}

int nvim_get_cmdline_star(void)
{
  return cmdline_star;
}

int nvim_get_cmdline_row(void)
{
  return cmdline_row;
}

int nvim_cmdline_win_is_active(void)
{
  return cmdline_win != NULL;
}

int nvim_cmdline_win_width(void)
{
  return cmdline_win ? cmdline_win->w_view_width : 0;
}

int nvim_cmdline_win_height(void)
{
  return cmdline_win ? cmdline_win->w_view_height : 0;
}

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

/// Get a wim_flags entry for Rust.
uint8_t nvim_get_wim_flags(int idx) { return wim_flags[idx]; }

/// Set a wim_flags entry for Rust.
void nvim_set_wim_flags(int idx, uint8_t val) { wim_flags[idx] = val; }

// Phase 67 Phase 2: Accessors for draw_cmdline and redrawcmd

/// Call color_cmdline(&ccline) and return its result.
bool nvim_color_cmdline(void) { return color_cmdline(&ccline); }

/// Get number of color chunks in ccline.last_colors.colors.
size_t nvim_get_ccline_colors_size(void) { return kv_size(ccline.last_colors.colors); }

/// Get a color chunk's fields by index.
void nvim_get_ccline_color_chunk(size_t idx, int *start_out, int *end_out, int *hl_id_out)
{
  CmdlineColorChunk chunk = kv_A(ccline.last_colors.colors, idx);
  *start_out = chunk.start;
  *end_out = chunk.end;
  *hl_id_out = chunk.hl_id;
}

/// Get cmdline_star global (for draw_cmdline star mode).
int nvim_get_cmdline_star_count(void) { return cmdline_star; }

/// Set redrawing_cmdline global.
void nvim_set_redrawing_cmdline(int val) { redrawing_cmdline = (val != 0); }

/// Set msg_no_more global.
void nvim_set_msg_no_more(int val) { msg_no_more = (val != 0); }

/// Set skip_redraw global.
void nvim_set_skip_redraw2(int val) { skip_redraw = (val != 0); }

// Phase 67 Phase 3: Accessors for put_on_cmdline

/// Get cmdbuff byte at given index.
char nvim_get_ccline_cmdbuff_byte(int idx) { return ccline.cmdbuff[idx]; }

/// Set cmdbuff byte at given index.
void nvim_set_ccline_cmdbuff_byte(int idx, char val) { ccline.cmdbuff[idx] = val; }

/// Get KeyTyped global.
int nvim_get_key_typed_cmdline(void) { return KeyTyped ? 1 : 0; }

/// Call msg_check() for Rust.
void nvim_msg_check(void) { msg_check(); }

// Phase 67 (Phase 5): Accessors for command_line_handle_ctrl_bsl, command_line_insert_reg,
// and command_line_erase_chars

/// Get p_ru option (ruler) for Rust.
int nvim_get_p_ru(void) { return p_ru; }

/// Call set_search_match() for Rust.
void nvim_set_search_match(pos_T *t) { set_search_match(t); }

/// Get new_cmdpos (used by set_cmdline_pos).
int nvim_get_new_cmdpos(void) { return new_cmdpos; }

/// Set new_cmdpos.
void nvim_set_new_cmdpos(int val) { new_cmdpos = val; }

/// Set KeyTyped global.
void nvim_set_key_typed(int val) { KeyTyped = (val != 0); }

/// Thin non-static wrapper for cmdline_paste (called from Rust).
bool nvim_cmdline_paste(int regname, bool literally, bool remcr)
{
  return cmdline_paste(regname, literally, remcr);
}

/// Wrapper for command_line_not_changed (called from Rust via opaque handle).
int nvim_command_line_not_changed(void *s)
{
  return command_line_not_changed((CommandLineState *)s);
}

/// Wrapper for command_line_changed (called from Rust via opaque handle).
int nvim_command_line_changed(void *s)
{
  return command_line_changed((CommandLineState *)s);
}

/// Wrapper for command_line_toggle_langmap (called from Rust via opaque handle).
void nvim_command_line_toggle_langmap(void *s)
{
  command_line_toggle_langmap((CommandLineState *)s);
}

/// Wrapper for command_line_left_right_mouse (called from Rust via opaque handle).
void nvim_command_line_left_right_mouse(void *s)
{
  command_line_left_right_mouse((CommandLineState *)s);
}

/// Browse history (called from Rust via opaque handle).
/// Inlines the former C static command_line_browse_history.
int nvim_command_line_browse_history(void *vs)
{
  CommandLineState *s = (CommandLineState *)vs;
  // Save current command string so it can be restored later.
  if (s->lookfor == NULL) {
    s->lookfor = xstrnsave(ccline.cmdbuff, (size_t)ccline.cmdlen);
    s->lookfor[ccline.cmdpos] = NUL;
    s->lookforlen = ccline.cmdpos;
  }
  // Pack state for Rust
  HistoryBrowseState rs_state = {
    .c = s->c,
    .firstc = s->firstc,
    .hiscnt = s->hiscnt,
    .save_hiscnt = s->save_hiscnt,
    .histype = s->histype,
    .lookfor = s->lookfor,
    .lookforlen = s->lookforlen,
  };
  // Call Rust implementation
  int result = rs_command_line_browse_history(&rs_state);
  // Update state from Rust
  s->hiscnt = rs_state.hiscnt;
  s->save_hiscnt = rs_state.save_hiscnt;
  // Clear xp_context on history change
  if (result == CMDLINE_CHANGED) {
    s->xpc.xp_context = EXPAND_NOTHING;
  }
  return result;
}

/// Wrapper for command_line_wildchar_complete (called from Rust via opaque handle).
int nvim_command_line_wildchar_complete(void *s)
{
  return command_line_wildchar_complete((CommandLineState *)s);
}

/// Wrapper for command_line_end_wildmenu (called from Rust via opaque handle).
void nvim_command_line_end_wildmenu(void *s, bool key_is_wc)
{
  command_line_end_wildmenu((CommandLineState *)s, key_is_wc);
}

/// Wrapper for may_trigger_cursormovedc (called from Rust via opaque handle).
void nvim_may_trigger_cursormovedc(void *s)
{
  may_trigger_cursormovedc((CommandLineState *)s);
}

/// Wrapper for do_autocmd_cmdlinechanged (called from Rust via opaque handle).
void nvim_do_autocmd_cmdlinechanged(int firstc)
{
  do_autocmd_cmdlinechanged(firstc);
}

/// Wrapper for trigger_cmd_autocmd (called from Rust).
void nvim_trigger_cmd_autocmd(int typechar, int evt)
{
  trigger_cmd_autocmd(typechar, (event_T)evt);
}

/// Wrapper for cmdpreview_may_show (called from Rust via opaque handle).
int nvim_cmdpreview_may_show(void *s)
{
  return (int)cmdpreview_may_show((CommandLineState *)s);
}

/// Wrapper for abandon_cmdline (called from Rust).
void nvim_abandon_cmdline(void)
{
  abandon_cmdline();
}

// CommandLineState field accessors (for Rust opaque handle pattern)

/// Get s->c field.
int nvim_cls_get_c(void *s)
{
  return ((CommandLineState *)s)->c;
}

/// Set s->c field.
void nvim_cls_set_c(void *s, int val)
{
  ((CommandLineState *)s)->c = val;
}

/// Get s->firstc field.
int nvim_cls_get_firstc(void *s)
{
  return ((CommandLineState *)s)->firstc;
}

/// Get s->count field.
int nvim_cls_get_count(void *s)
{
  return ((CommandLineState *)s)->count;
}

/// Get s->indent field.
int nvim_cls_get_indent(void *s)
{
  return ((CommandLineState *)s)->indent;
}

/// Get s->gotesc field.
int nvim_cls_get_gotesc(void *s)
{
  return ((CommandLineState *)s)->gotesc ? 1 : 0;
}

/// Set s->gotesc field.
void nvim_cls_set_gotesc(void *s, int val)
{
  ((CommandLineState *)s)->gotesc = (val != 0);
}

/// Get s->do_abbr field.
int nvim_cls_get_do_abbr(void *s)
{
  return ((CommandLineState *)s)->do_abbr ? 1 : 0;
}

/// Set s->do_abbr field.
void nvim_cls_set_do_abbr(void *s, int val)
{
  ((CommandLineState *)s)->do_abbr = (val != 0);
}

/// Get s->ignore_drag_release field.
int nvim_cls_get_ignore_drag_release(void *s)
{
  return ((CommandLineState *)s)->ignore_drag_release ? 1 : 0;
}

/// Set s->ignore_drag_release field.
void nvim_cls_set_ignore_drag_release(void *s, int val)
{
  ((CommandLineState *)s)->ignore_drag_release = (val != 0);
}

/// Get s->did_wild_list field.
int nvim_cls_get_did_wild_list(void *s)
{
  return ((CommandLineState *)s)->did_wild_list ? 1 : 0;
}

/// Set s->did_wild_list field.
void nvim_cls_set_did_wild_list(void *s, int val)
{
  ((CommandLineState *)s)->did_wild_list = (val != 0);
}

/// Get s->wim_index field.
int nvim_cls_get_wim_index(void *s)
{
  return ((CommandLineState *)s)->wim_index;
}

/// Set s->wim_index field.
void nvim_cls_set_wim_index(void *s, int val)
{
  ((CommandLineState *)s)->wim_index = val;
}

/// Get s->skip_pum_redraw field.
int nvim_cls_get_skip_pum_redraw(void *s)
{
  return ((CommandLineState *)s)->skip_pum_redraw ? 1 : 0;
}

/// Set s->skip_pum_redraw field.
void nvim_cls_set_skip_pum_redraw(void *s, int val)
{
  ((CommandLineState *)s)->skip_pum_redraw = (val != 0);
}

/// Get s->cmdline_type field.
int nvim_cls_get_cmdline_type(void *s)
{
  return ((CommandLineState *)s)->cmdline_type;
}

/// Get s->prev_cmdpos field.
int nvim_cls_get_prev_cmdpos(void *s)
{
  return ((CommandLineState *)s)->prev_cmdpos;
}

/// Set s->prev_cmdpos field.
void nvim_cls_set_prev_cmdpos(void *s, int val)
{
  ((CommandLineState *)s)->prev_cmdpos = val;
}

/// Get s->prev_cmdbuff field.
char *nvim_cls_get_prev_cmdbuff(void *s)
{
  return ((CommandLineState *)s)->prev_cmdbuff;
}

/// Get s->some_key_typed field.
int nvim_cls_get_some_key_typed(void *s)
{
  return ((CommandLineState *)s)->some_key_typed ? 1 : 0;
}

/// Set s->some_key_typed field.
void nvim_cls_set_some_key_typed(void *s, int val)
{
  ((CommandLineState *)s)->some_key_typed = (val != 0);
}

/// Get s->did_hist_navigate field.
int nvim_cls_get_did_hist_navigate(void *s)
{
  return ((CommandLineState *)s)->did_hist_navigate ? 1 : 0;
}

/// Set s->did_hist_navigate field.
void nvim_cls_set_did_hist_navigate(void *s, int val)
{
  ((CommandLineState *)s)->did_hist_navigate = (val != 0);
}

/// Get s->hiscnt field.
int nvim_cls_get_hiscnt(void *s)
{
  return ((CommandLineState *)s)->hiscnt;
}

/// Get s->save_hiscnt field.
int nvim_cls_get_save_hiscnt(void *s)
{
  return ((CommandLineState *)s)->save_hiscnt;
}

/// Get s->histype field.
int nvim_cls_get_histype(void *s)
{
  return ((CommandLineState *)s)->histype;
}

/// Get pointer to s->is_state (incsearch state).
void *nvim_cls_get_is_state(void *s)
{
  return (void *)&((CommandLineState *)s)->is_state;
}

/// Get pointer to s->xpc (expand context).
void *nvim_cls_get_xpc(void *s)
{
  return (void *)&((CommandLineState *)s)->xpc;
}

/// Get s->xpc.xp_numfiles field.
int nvim_cls_get_xpc_numfiles(void *s)
{
  return ((CommandLineState *)s)->xpc.xp_numfiles;
}

/// Set s->xpc.xp_context field.
void nvim_cls_set_xpc_context(void *s, int val)
{
  ((CommandLineState *)s)->xpc.xp_context = val;
}

/// Get s->xpc.xp_context field.
int nvim_cls_get_xpc_context(void *s)
{
  return ((CommandLineState *)s)->xpc.xp_context;
}

// nvim_get_mod_mask, nvim_set_mod_mask defined in getchar.c
// nvim_get_iobuff defined in option_shim.c

/// Get s->event_cmdlineleavepre_triggered field.
int nvim_cls_get_event_cmdlineleavepre_triggered(void *s)
{
  return ((CommandLineState *)s)->event_cmdlineleavepre_triggered ? 1 : 0;
}

/// Set s->event_cmdlineleavepre_triggered field.
void nvim_cls_set_event_cmdlineleavepre_triggered(void *s, int val)
{
  ((CommandLineState *)s)->event_cmdlineleavepre_triggered = (val != 0);
}

/// Get s->break_ctrl_c field.
int nvim_cls_get_break_ctrl_c(void *s)
{
  return ((CommandLineState *)s)->break_ctrl_c ? 1 : 0;
}

/// Get ccline.mouse_used pointer (may be NULL).
int nvim_cls_get_ccline_mouse_used(void)
{
  return ccline.mouse_used != NULL ? 1 : 0;
}

/// Set the value at ccline.mouse_used if non-NULL.
void nvim_cls_set_ccline_mouse_used_val(int val)
{
  if (ccline.mouse_used != NULL) {
    *ccline.mouse_used = (val != 0);
  }
}

/// Wrapper for cmdline_pum_cleanup(&ccline) (called from Rust).
void nvim_cmdline_pum_cleanup(void)
{
  cmdline_pum_cleanup(&ccline);
}

/// Call showmatches for &s->xpc (called from Rust with opaque xp pointer).
int nvim_showmatches(void *xp, bool display_wildmenu, bool display_list, bool noselect)
{
  return showmatches((expand_T *)xp, display_wildmenu, display_list, noselect);
}

/// Call nextwild for &s->xpc (called from Rust with opaque xp pointer).
int nvim_nextwild(void *xp, int type, int options, bool escape)
{
  return nextwild((expand_T *)xp, type, options, escape);
}

// nvim_get_mouse_row: defined in getchar.c
// nvim_get_cmdline_row: use nvim_get_cmdline_row from existing accessors
// nvim_get_ex_normal_busy: defined in getchar.c
// nvim_get_typebuf_len: defined in getchar.c
// nvim_get_p_ari: defined in edit.c

/// Get cmdline_row global (different from existing nvim_get_cmdline_row).
int nvim_get_cmdline_row_exgetln(void)
{
  return cmdline_row;
}

/// Get getln_interrupted_highlight global.
int nvim_get_getln_interrupted_highlight(void)
{
  return getln_interrupted_highlight ? 1 : 0;
}

/// Set getln_interrupted_highlight global.
void nvim_set_getln_interrupted_highlight(int val)
{
  getln_interrupted_highlight = (val != 0);
}

/// Set ccline.cmdbuff[0] to NUL (for 'q' with mouse prompt).
void nvim_ccline_cmdbuff_set_nul(void)
{
  if (ccline.cmdbuff != NULL) {
    *ccline.cmdbuff = NUL;
  }
}

/// Get may_add_char_to_search result (called from Rust).
int nvim_may_add_char_to_search(int firstc, int *c, void *is_state)
{
  return may_add_char_to_search(firstc, c, (incsearch_state_T *)is_state);
}

/// Get cedit_key value (static variable, exposed for Rust).
int nvim_get_cedit_key(void)
{
  return cedit_key;
}

/// Non-static wrapper for open_cmdwin (static function, exposed for Rust).
int nvim_open_cmdwin(void)
{
  return open_cmdwin();
}

