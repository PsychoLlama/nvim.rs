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

/// Pending highlight callback for getcmdline_prompt (passed through static to
/// avoid crossing the FFI boundary with the complex Callback union type).
static Callback pending_prompt_hl_callback = CALLBACK_INIT;

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
static int cmdline_enter_level = 0;  ///< nesting level of command_line_enter() calls

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

/// Thin C wrapper: delegates to Rust rs_trigger_cmd_autocmd.
void nvim_trigger_cmd_autocmd(int typechar, int evt)
{
  rs_trigger_cmd_autocmd(typechar, evt);
}

// command_line_enter migrated to Rust (entry_impl.rs as rs_command_line_enter).


/// Fire CmdlineChanged autocmd (called from Rust; skip has_event check since Rust does it).
void nvim_fire_cmdlinechanged_autocmd(int firstc)
{
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

/// Check if CmdlineChanged event has any listeners (for Rust).
int nvim_has_event_cmdlinechanged(void) { return has_event(EVENT_CMDLINECHANGED) ? 1 : 0; }

// do_autocmd_cmdlinechanged: implemented in Rust (cmdline crate, state.rs as rs_do_autocmd_cmdlinechanged).

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
  return (char *)rs_command_line_enter(firstc, count, indent, 1);
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
  // Store highlight_callback in static so Rust rs_getcmdline_prompt can apply
  // it after clearing ccline (Callback union cannot cross the FFI boundary).
  pending_prompt_hl_callback = highlight_callback;
  return rs_getcmdline_prompt(firstc, prompt, hl_id, xp_context, xp_arg, (int)one_key, mouse_used);
}

// check_opt_wim() is implemented in Rust (cmdline crate, wildmenu.rs).

// getexline: implemented in Rust (cmdline crate, entry_impl.rs).
// alloc_cmdbuff, dealloc_cmdbuff, realloc_cmdbuff: implemented in Rust (cmdline crate, cmdbuff.rs).

/// Adjust ccline.xpc->xp_pattern after buffer reallocation.
/// Called from Rust realloc_cmdbuff after the new buffer is installed.
void nvim_realloc_cmdbuff_xp_fixup(const char *old_buff)
{
  if (ccline.xpc != NULL
      && ccline.xpc->xp_pattern != NULL
      && ccline.xpc->xp_context != EXPAND_NOTHING
      && ccline.xpc->xp_context != EXPAND_UNSUCCESSFUL) {
    int i = (int)(ccline.xpc->xp_pattern - old_buff);
    if (i >= 0 && i <= ccline.cmdlen) {
      ccline.xpc->xp_pattern = ccline.cmdbuff + i;
    }
  }
}

enum { MAX_CB_ERRORS = 1, };

/// Color the command-line (ccline).
///
/// Uses built-in expression parser for '=' prompt, or user-specified callback.
/// Caches results: if prompt_id and cmdbuff are unchanged, returns immediately.
///
/// @return true if draw_cmdline may proceed, false if nothing to do.
// nvim_color_cmdline: implemented in Rust (cmdline crate, color.rs).
// The following C helpers expose C-specific functionality to the Rust implementation.

/// State shared between color_cmdline C helper functions.
typedef struct {
  Callback color_cb;
  bool can_free_cb;
  Error err;
  const char *err_errmsg;
  bool dgc_ret;
  typval_T arg;
  bool arg_allocated;
  typval_T tv;
} ColorCmdlineHelperState;
static ColorCmdlineHelperState ccs;

// nvim_color_cache_valid: inlined into Rust nvim_color_cmdline (color.rs).

/// Reset ccline.last_colors.colors kvec to size 0 and reset helper state.
void nvim_ccline_reset_colors(void)
{
  kv_size(ccline.last_colors.colors) = 0;
  // Reset helper state for a fresh call.
  ccs = (ColorCmdlineHelperState){
    .color_cb = CALLBACK_NONE,
    .can_free_cb = false,
    .err = ERROR_INIT,
    .err_errmsg = "",
    .dgc_ret = true,
    .arg = { .v_type = VAR_UNKNOWN },
    .arg_allocated = false,
    .tv = { .v_type = VAR_UNKNOWN },
  };
}

// nvim_color_is_empty: inlined into Rust nvim_color_cmdline (color.rs).

/// Acquire the coloring callback.
/// Returns: 0=none, 1=highlight_callback, 2=ex callback(':'), 3=expr path('=').
/// -1 on error (check nvim_color_has_acquire_err).
int nvim_color_acquire_callback(void)
{
  ccs.color_cb = CALLBACK_NONE;
  ccs.can_free_cb = false;
  ccs.err = ERROR_INIT;
  ccs.err_errmsg = e_intern2;
  ccs.dgc_ret = true;
  ccs.arg = (typval_T){ .v_type = VAR_STRING, .vval.v_string = ccline.cmdbuff };
  ccs.arg_allocated = false;
  ccs.tv = (typval_T){ .v_type = VAR_UNKNOWN };

  if (ccline.highlight_callback.type != kCallbackNone) {
    assert(ccline.input_fn);
    ccs.color_cb = ccline.highlight_callback;
    return 1;
  } else if (ccline.cmdfirstc == ':') {
    TRY_WRAP(&ccs.err, {
      ccs.err_errmsg = N_("E5408: Unable to get g:Nvim_color_cmdline callback: %s");
      ccs.dgc_ret = tv_dict_get_callback(get_globvar_dict(), S_LEN("Nvim_color_cmdline"),
                                         &ccs.color_cb);
    });
    ccs.can_free_cb = true;
    if (ERROR_SET(&ccs.err) || !ccs.dgc_ret) {
      return -1;
    }
    return 2;
  } else if (ccline.cmdfirstc == '=') {
    return 3;
  }
  return 0;
}

/// Run the full VimL expression coloring path ('=' firstc).
void nvim_color_run_expr_coloring(void)
{
  ColoredCmdline *ccline_colors = &ccline.last_colors;
  ParserLine parser_lines[] = {
    { .data = ccline.cmdbuff, .size = strlen(ccline.cmdbuff), .allocated = false },
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

/// Invoke the acquired callback. Returns 1=ok, 0=failed.
int nvim_color_run_callback_coloring(void)
{
  if (ccline.cmdbuff[ccline.cmdlen] != NUL) {
    ccs.arg_allocated = true;
    ccs.arg.vval.v_string = xmemdupz(ccline.cmdbuff, (size_t)ccline.cmdlen);
  }
  getln_interrupted_highlight = false;
  bool cbcall_ret = true;
  ccs.err = ERROR_INIT;
  TRY_WRAP(&ccs.err, {
    ccs.err_errmsg = N_("E5407: Callback has thrown an exception: %s");
    const int saved_msg_col = msg_col;
    msg_silent++;
    cbcall_ret = callback_call(&ccs.color_cb, 1, &ccs.arg, &ccs.tv);
    msg_silent--;
    msg_col = saved_msg_col;
    if (got_int) {
      getln_interrupted_highlight = true;
    }
  });
  return (ERROR_SET(&ccs.err) || !cbcall_ret) ? 0 : 1;
}

/// Get v_type of the result typval.
int nvim_color_result_tv_type(void) { return (int)ccs.tv.v_type; }

/// Get v_list of the result typval (NULL if not a list).
void *nvim_color_result_tv_list(void) { return (void *)ccs.tv.vval.v_list; }

/// Print an error message (PRINT_ERRMSG style: scroll + newline + smsg).
void nvim_color_errmsg(const char *msg)
{
  msg_scroll = true;
  msg_putchar('\n');
  smsg(HLF_E, "%s", msg);
}

/// Get ccline.last_colors.prompt_id (for Rust cache_valid check).
unsigned int nvim_get_ccline_last_colors_prompt_id(void) { return ccline.last_colors.prompt_id; }

/// Get ccline.last_colors.cmdbuff (for Rust cache_valid/is_empty check).
const char *nvim_get_ccline_last_colors_cmdbuff(void) { return ccline.last_colors.cmdbuff; }

/// Free and NULL ccline.last_colors.cmdbuff (for Rust is_empty check).
void nvim_ccline_clear_last_colors_cmdbuff(void) { XFREE_CLEAR(ccline.last_colors.cmdbuff); }

/// Push a color chunk to ccline.last_colors.colors.
void nvim_ccline_colors_push(int start, int end, int hl_id)
{
  kv_push(ccline.last_colors.colors, ((CmdlineColorChunk) {
    .start = start, .end = end, .hl_id = hl_id,
  }));
}

/// Finalize coloring: update cmdbuff cache, prompt_id, tv_clear, free callback.
/// If success==0 (error path): print pending error, clear colors, call redrawcmdline.
void nvim_color_finalize(int success)
{
  if (!success) {
    if (ERROR_SET(&ccs.err)) {
      msg_putchar('\n');
      msg_scroll = true;
      smsg(HLF_E, _(ccs.err_errmsg), ccs.err.msg);
      api_clear_error(&ccs.err);
    }
    kv_size(ccline.last_colors.colors) = 0;
  }
  if (ccs.can_free_cb) {
    callback_free(&ccs.color_cb);
  }
  xfree(ccline.last_colors.cmdbuff);
  ccline.last_colors.prompt_id = ccline.prompt_id;
  if (ccs.arg_allocated) {
    ccline.last_colors.cmdbuff = ccs.arg.vval.v_string;
  } else {
    ccline.last_colors.cmdbuff = xmemdupz(ccline.cmdbuff, (size_t)ccline.cmdlen);
  }
  tv_clear(&ccs.tv);
  if (!success) {
    redrawcmdline();
  }
}

/// Get ccline.cmdlen.
int nvim_color_cmdlen(void) { return ccline.cmdlen; }

/// Get one byte from ccline.cmdbuff at the given index.
unsigned char nvim_color_cmdbuff_at(int idx) { return (unsigned char)ccline.cmdbuff[idx]; }

/// Wrapper for tv_list_len (inline) called from Rust color.rs.
int nvim_color_tv_list_len(const list_T *l)
{
  return tv_list_len(l);
}

/// Wrapper for tv_get_number_chk (inline) called from Rust color.rs.
int64_t nvim_color_tv_get_number_chk(const typval_T *tv, bool *error)
{
  return tv_get_number_chk(tv, error);
}

/// Wrapper for tv_get_string_chk (inline) called from Rust color.rs.
const char *nvim_color_tv_get_string_chk(const typval_T *tv)
{
  return tv_get_string_chk(tv);
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

// cmdline_ui_flush: implemented in Rust (cmdline crate, ui.rs).
// See rs_cmdline_ui_flush exported by Rust and the thin C wrapper below.

/// Thin C wrapper for Rust rs_cmdline_ui_flush.
/// Called by ui_flush to keep cmdline updated for external UIs.
void cmdline_ui_flush(void) { rs_cmdline_ui_flush(); }

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

/// Public wrapper for rs_do_autocmd_cmdlinechanged (for VimL functions in other files).
void nvim_do_autocmd_cmdlinechanged(int firstc) { rs_do_autocmd_cmdlinechanged(firstc); }

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
/// Delegates to Rust rs_did_set_cedit (implemented in option crate).
const char *did_set_cedit(optset_T *args)
{
  return rs_did_set_cedit(args);
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

/// Set ccline.xp_arg (for getcmdline_prompt setup from Rust).
void nvim_set_ccline_xp_arg(const char *arg) { ccline.xp_arg = (char *)arg; }

/// Set ccline.input_fn (for getcmdline_prompt setup from Rust).
void nvim_set_ccline_input_fn(int val) { ccline.input_fn = (val != 0); }

/// Set ccline.mouse_used pointer (for getcmdline_prompt setup from Rust).
void nvim_set_ccline_mouse_used_ptr(bool *ptr) { ccline.mouse_used = ptr; }

/// Apply pending_prompt_hl_callback to ccline.highlight_callback.
/// Called from Rust rs_getcmdline_prompt after ccline fields are set.
/// Resets pending_prompt_hl_callback to CALLBACK_NONE after copying.
void nvim_apply_pending_hl_callback(void)
{
  ccline.highlight_callback = pending_prompt_hl_callback;
  pending_prompt_hl_callback = (Callback)CALLBACK_INIT;
}

/// Increment last_prompt_id and return new value.
unsigned int nvim_get_ccline_prompt_id_inc(void) { return ++last_prompt_id; }

int nvim_get_key_typed(void) { return KeyTyped; }
int nvim_get_cmdline_star(void) { return cmdline_star; }
int nvim_cmdline_win_is_active(void) { return cmdline_win != NULL; }
int nvim_cmdline_win_width(void) { return cmdline_win ? cmdline_win->w_view_width : 0; }
int nvim_cmdline_win_height(void) { return cmdline_win ? cmdline_win->w_view_height : 0; }

/// Simplified wrapper around rs_getcmdline_prompt for FFI use.
/// Avoids passing the complex Callback union across FFI boundary.
char *nvim_getcmdline_prompt_simple(int firstc, const char *prompt, int hl_id,
                                    int xp_context, bool one_key, bool *mouse_used)
{
  return rs_getcmdline_prompt(firstc, prompt, hl_id, xp_context, NULL, (int)one_key, mouse_used);
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

/// Get exec_from_reg global (for Rust getexline).
int nvim_get_exec_from_reg(void) { return exec_from_reg ? 1 : 0; }

/// Set cedit_key value (for Rust did_set_cedit implementation).
void nvim_set_cedit_key(int val) { cedit_key = val; }

/// Get p_cedit option string (for Rust did_set_cedit implementation).
const char *nvim_get_p_cedit(void) { return p_cedit; }



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

// Additional helpers for Rust cmdline_ui_flush implementation.

/// Get &ccline as opaque pointer (starting point for cmdline_ui_flush iteration).
void *nvim_get_ccline_self_ptr(void) { return (void *)&ccline; }

/// Set cmdline_was_last_drawn global.
void nvim_set_cmdline_was_last_drawn(int val) { cmdline_was_last_drawn = (val != 0); }

/// Get a CmdlineInfo node's redraw_state as int (kCmdRedrawNone=0, kCmdRedrawPos=1, kCmdRedrawAll=2).
int nvim_ccline_ptr_get_redraw_state(void *p) { return (int)((CmdlineInfo *)p)->redraw_state; }

/// Set a CmdlineInfo node's redraw_state to kCmdRedrawNone (for Rust).
void nvim_ccline_ptr_set_redraw_none(void *p) { ((CmdlineInfo *)p)->redraw_state = kCmdRedrawNone; }

/// Get a CmdlineInfo node's cmdpos field (for Rust cmdline_ui_flush).
int nvim_ccline_ptr_get_cmdpos(void *p) { return ((CmdlineInfo *)p)->cmdpos; }

/// Update cmdline UI for the given CmdlineInfo node.
/// show=1: call ui_call_cmdline_show (full redraw).
/// show=0: call ui_call_cmdline_pos (position update only).
void nvim_cmdline_ui_update_for_level(void *p, int show)
{
  CmdlineInfo *line = (CmdlineInfo *)p;
  if (!show) {
    ui_call_cmdline_pos(line->cmdpos, line->level);
    return;
  }
  Arena arena = ARENA_EMPTY;
  Array content;
  if (cmdline_star) {
    content = arena_array(&arena, 1);
    size_t len = 0;
    for (char *q = ccline.cmdbuff; *q; MB_PTR_ADV(q)) {
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
    ui_call_cmdline_special_char(cstr_as_string(charbuf), line->special_shift, line->level);
  }
  arena_mem_free(arena_finish(&arena));
}

// =============================================================================
// Helpers for Rust command_line_enter (Phase 1)
// =============================================================================

/// Increment and return cmdline_enter_level.
int nvim_cmdline_enter_level_inc(void) { return ++cmdline_enter_level; }

/// Decrement cmdline_enter_level.
void nvim_cmdline_enter_level_dec(void) { cmdline_enter_level--; }

/// Get cmdline_enter_level.
int nvim_cmdline_enter_level_get(void) { return cmdline_enter_level; }

/// Get and save cmdpreview, then set it to false.
bool nvim_cmdpreview_save_and_clear(void)
{
  bool saved = cmdpreview;
  cmdpreview = false;
  return saved;
}

/// Get current cmdpreview value (for Rust teardown).
bool nvim_get_cmdpreview(void) { return cmdpreview; }

/// Restore cmdpreview and optionally trigger a full redraw.
void nvim_cmdpreview_restore(bool saved, bool current)
{
  if (current != saved) {
    cmdpreview = saved;
    redraw_all_later(UPD_SOME_VALID);
  }
}

/// Get msg_scroll (global bool).
int nvim_get_msg_scroll(void) { return msg_scroll ? 1 : 0; }

/// Set msg_scroll.
void nvim_set_msg_scroll(int val) { msg_scroll = (val != 0); }

/// Set State global.
void nvim_set_State(int val) { State = val; }

/// Set got_int to false.
void nvim_clear_got_int(void) { got_int = false; }

/// Set did_emsg to false.
void nvim_clear_did_emsg(void) { did_emsg = false; }

/// Check did_emsg (for redrawcmd after error).
int nvim_get_did_emsg_for_redraw(void) { return did_emsg ? 1 : 0; }

/// Get p_icm as newly allocated string (caller must xfree).
char *nvim_get_p_icm_dup(void) { return xstrdup(p_icm); }

/// Set p_icm option via set_option_direct with SID_NONE.
void nvim_set_p_icm_option(const char *val)
{
  set_option_direct(kOptInccommand, CSTR_AS_OPTVAL((char *)val), 0, SID_NONE);
}

/// Clear ccline completely (CLEAR_FIELD(ccline)).
void nvim_clear_ccline(void) { CLEAR_FIELD(ccline); }

/// Setup ccline.prev_ccline link and save current ccline.
/// If ccline.cmdbuff != NULL (recursive call): saves ccline to *save_out and clears.
/// If ccline.cmdbuff == NULL and clear_ccline: clears ccline.
/// If ccline.cmdbuff == NULL and !clear_ccline: leaves ccline untouched.
/// Returns true if ccline was saved (recursive call).
/// save_out must point to CmdlineInfo-sized, 8-byte-aligned storage.
bool nvim_ccline_save_and_clear(void *save_out, bool clear_ccline_flag)
{
  CmdlineInfo *out = (CmdlineInfo *)save_out;
  if (ccline.cmdbuff != NULL) {
    // Recursive call: save current ccline and start fresh
    *out = ccline;
    CLEAR_FIELD(ccline);
    ccline.prev_ccline = out;
    ccline.cmdbuff = NULL;
    return true;
  }
  if (clear_ccline_flag) {
    CLEAR_FIELD(ccline);
  }
  return false;
}

/// Restore ccline from save_out (undo nvim_ccline_save_and_clear).
/// save_out must point to CmdlineInfo-sized, 8-byte-aligned storage.
void nvim_ccline_restore(const void *save_out)
{
  ccline = *(const CmdlineInfo *)save_out;
}

/// Initialize ccline fields for command_line_enter (called after allocation).
void nvim_ccline_enter_init(int firstc, int indent)
{
  ccline.overstrike = false;
  ccline.cmdfirstc = (firstc == '@' ? 0 : firstc);
  ccline.cmdindent = (firstc > 0 ? indent : 0);
  ccline.cmdlen = ccline.cmdpos = 0;
  ccline.cmdbuff[0] = NUL;
  ccline.last_colors = (ColoredCmdline){ .cmdbuff = NULL, .colors = KV_INITIAL_VALUE };
  ccline.prompt_id = last_prompt_id++;
}

/// Set ccline.level from cmdline_enter_level_out (already incremented).
void nvim_ccline_set_level(int level) { ccline.level = level; }

/// Apply autoindent spaces to cmdbuff for :insert/:append (firstc <= 0).
void nvim_ccline_apply_indent(int indent)
{
  memset(ccline.cmdbuff, ' ', (size_t)indent);
  ccline.cmdbuff[indent] = NUL;
  ccline.cmdpos = indent;
  ccline.cmdspos = indent;
  ccline.cmdlen = indent;
}

/// Init xpc and bind it to ccline.
void nvim_ccline_init_xpc(void *s)
{
  CommandLineState *cs = (CommandLineState *)s;
  ExpandInit(&cs->xpc);
  ccline.xpc = &cs->xpc;
  clear_cmdline_orig();
  cs->xpc.xp_context = EXPAND_NOTHING;
  cs->xpc.xp_backslash = XP_BS_NONE;
#ifndef BACKSLASH_IN_FILENAME
  cs->xpc.xp_shell = false;
#endif
  if (ccline.input_fn) {
    cs->xpc.xp_context = ccline.xp_context;
    cs->xpc.xp_pattern = ccline.cmdbuff;
    cs->xpc.xp_arg = ccline.xp_arg;
  }
}

/// Set langmap mode based on firstc and b_p_imsearch/b_p_iminsert.
/// Returns 1 if langmap was set.
int nvim_cmdline_setup_langmap(void *s, int firstc)
{
  CommandLineState *cs = (CommandLineState *)s;
  if (firstc == '/' || firstc == '?' || firstc == '@') {
    if (curbuf->b_p_imsearch == B_IMODE_USE_INSERT) {
      cs->b_im_ptr = &curbuf->b_p_iminsert;
    } else {
      cs->b_im_ptr = &curbuf->b_p_imsearch;
    }
    cs->b_im_ptr_buf = curbuf;
    if (*cs->b_im_ptr == B_IMODE_LMAP) {
      State |= MODE_LANGMAP;
      return 1;
    }
  }
  return 0;
}

/// Redraw statuslines for all windows where applicable.
void nvim_cmdline_redraw_statuslines(void)
{
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

// nvim_cmdline_leave_cleanup: implemented in Rust (entry_impl.rs, leave_cleanup).
// nvim_cmdline_final_teardown: implemented in Rust (entry_impl.rs, final_teardown).

/// Get b_p_imsearch or b_p_iminsert based on cs->b_im_ptr.
/// Returns the value to restore when leaving.
int nvim_cls_get_b_im_ptr_val(void *s)
{
  CommandLineState *cs = (CommandLineState *)s;
  if (cs->b_im_ptr == NULL) {
    return -1;
  }
  return *cs->b_im_ptr;
}

/// Set *cs->b_im_ptr = val (for restore on leave).
void nvim_cls_set_b_im_ptr_val(void *s, int val)
{
  CommandLineState *cs = (CommandLineState *)s;
  if (cs->b_im_ptr != NULL && cs->b_im_ptr_buf == curbuf) {
    *cs->b_im_ptr = val;
  }
}

// =============================================================================
// Additional C helpers for rs_command_line_enter (Phase 1 new wrappers)
// =============================================================================

/// Set cmdmsg_rl global.
void nvim_set_cmdmsg_rl(int val) { cmdmsg_rl = (val != 0); }

/// Set redir_off global.
void nvim_set_redir_off(int val) { redir_off = (val != 0); }

/// Get redir_off global.
int nvim_get_redir_off(void) { return redir_off ? 1 : 0; }

/// Wrapper for gotocmdline(true).
void nvim_gotocmdline(void) { gotocmdline(true); }

/// Initialize history and return hislen.
int nvim_init_history_and_get_hislen(void)
{
  init_history();
  return get_hislen();
}

/// Get exmode_active global.
int nvim_get_exmode_active(void) { return exmode_active ? 1 : 0; }

/// Fire CmdlineLeavePre autocmd if not already triggered.
/// Sets v:char to c_val first. Returns 1 if triggered, 0 if already done.
int nvim_cmdline_fire_leavepre_autocmd(void *s, int c_val)
{
  CommandLineState *cs = (CommandLineState *)s;
  if (cs->event_cmdlineleavepre_triggered) {
    return 0;
  }
  set_vim_var_char(c_val);
  nvim_trigger_cmd_autocmd(cs->cmdline_type, EVENT_CMDLINELEAVEPRE);
  cs->event_cmdlineleavepre_triggered = true;
  return 1;
}

/// Fire CmdlineLeave autocmd (with v:char and abort handling).
/// Sets v:char to c_val, fires autocmd, checks abort flag.
/// Returns 1 if user set abort, 0 otherwise. Handles error printing.
int nvim_cmdline_fire_leave_full(void *s, int c_val)
{
  CommandLineState *cs = (CommandLineState *)s;
  if (!has_event(EVENT_CMDLINELEAVE)) {
    return 0;
  }
  Error err = ERROR_INIT;
  save_v_event_T save_v_event;
  dict_T *dict = get_v_event(&save_v_event);
  char firstcbuf[2] = { (char)cs->cmdline_type, 0 };
  tv_dict_add_str(dict, S_LEN("cmdtype"), firstcbuf);
  tv_dict_add_nr(dict, S_LEN("cmdlevel"), ccline.level);
  tv_dict_set_keys_readonly(dict);
  tv_dict_add_bool(dict, S_LEN("abort"),
                   cs->gotesc ? kBoolVarTrue : kBoolVarFalse);
  set_vim_var_char(c_val);
  TRY_WRAP(&err, {
    apply_autocmds(EVENT_CMDLINELEAVE, firstcbuf, firstcbuf, false, curbuf);
  });
  int abort = 0;
  if (tv_dict_get_number(dict, "abort") != 0) {
    cs->gotesc = true;
    abort = 1;
  }
  restore_v_event(dict, &save_v_event);
  if (ERROR_SET(&err)) {
    msg_putchar('\n');
    emsg(err.msg);
    did_emsg = false;
    api_clear_error(&err);
  }
  return abort;
}

/// Combined CmdlineEnter autocmd with error handling.
/// Returns error message string if error occurred (static buffer), NULL if ok.
/// Prints error to cmdline and calls redrawcmd() if needed.
int nvim_cmdline_fire_enter_full(const char *firstcbuf, int level)
{
  if (!has_event(EVENT_CMDLINEENTER)) {
    return 0;
  }
  Error err = ERROR_INIT;
  save_v_event_T save_v_event;
  dict_T *dict = get_v_event(&save_v_event);
  tv_dict_add_str(dict, S_LEN("cmdtype"), firstcbuf);
  tv_dict_add_nr(dict, S_LEN("cmdlevel"), level);
  tv_dict_set_keys_readonly(dict);
  TRY_WRAP(&err, {
    apply_autocmds(EVENT_CMDLINEENTER, (char *)firstcbuf, (char *)firstcbuf, false, curbuf);
    restore_v_event(dict, &save_v_event);
  });
  if (ERROR_SET(&err)) {
    msg_putchar('\n');
    msg_scroll = true;
    msg_puts_hl(err.msg, HLF_E, true);
    api_clear_error(&err);
    redrawcmd();
    return 1;
  }
  return 0;
}

/// Wrapper to emit e_command_too_recursive error.
void nvim_emsg_command_too_recursive(void) { emsg(_(e_command_too_recursive)); }

// =============================================================================
// Phase 3 thin C wrappers (for Rust leave_cleanup / final_teardown)
// =============================================================================

/// Cleanup wildmenu for ccline.
void nvim_wildmenu_cleanup_ccline(void) { wildmenu_cleanup(&ccline); }

/// Clear xpc pointer in ccline and clear cmdline_orig.
void nvim_ccline_clear_xpc_and_orig(void) { ccline.xpc = NULL; clear_cmdline_orig(); }

/// Set ccline.xpc to NULL.
void nvim_ccline_xpc_clear(void) { ccline.xpc = NULL; }

/// Add current ccline.cmdbuff to history. histype, sep_char from Rust.
void nvim_add_to_history_ccline(int histype, int sep_char)
{
  add_to_history(histype, ccline.cmdbuff, (size_t)ccline.cmdlen, true, sep_char);
}

/// Save new_last_cmdline from current ccline.cmdbuff.
void nvim_save_last_cmdline(void)
{
  xfree(new_last_cmdline);
  new_last_cmdline = xstrnsave(ccline.cmdbuff, (size_t)ccline.cmdlen);
}

/// Compute and update cmdrow if msg_scrolled is 0.
void nvim_compute_cmdrow_if_not_scrolled(void) { if (msg_scrolled == 0) { compute_cmdrow(); } }

/// Set msg("", 0) and redraw_cmdline=true (for gotesc case).
void nvim_cmdline_gotesc_msg(void) { msg("", 0); redraw_cmdline = true; }

/// Check if p_ch==0 and kUIMessages not active, and set must redraw VALID.
void nvim_cmdline_check_must_redraw(void)
{
  if (p_ch == 0 && !ui_has(kUIMessages)) {
    set_must_redraw(UPD_VALID);
  }
}

/// Get ccline.cmdbuff (non-NULL means we have a line).
int nvim_ccline_has_cmdbuff(void) { return ccline.cmdbuff != NULL ? 1 : 0; }

/// Get ccline.one_key (for gotesc handling).
int nvim_ccline_get_one_key(void) { return ccline.one_key ? 1 : 0; }

/// Get msg_scrolled global.
int nvim_get_msg_scrolled(void) { return msg_scrolled; }

/// Set need_wait_return=false.
void nvim_clear_need_wait_return_wrap(void) { need_wait_return = false; }

/// Free ccline.last_colors.cmdbuff and kv_destroy ccline.last_colors.colors.
void nvim_ccline_free_last_colors(void)
{
  xfree(ccline.last_colors.cmdbuff);
  kv_destroy(ccline.last_colors.colors);
}

/// Call ui_call_cmdline_hide if kUICmdline is active.
void nvim_cmdline_ui_hide(int gotesc)
{
  if (ui_has(kUICmdline)) {
    cmdline_was_last_drawn = false;
    ccline.redraw_state = kCmdRedrawNone;
    ui_call_cmdline_hide(ccline.level, gotesc);
  }
}

/// Emit status_redraw_all() and redraw_custom_title_later() if not cmd_silent.
void nvim_cmdline_status_redraw(void)
{
  if (!cmd_silent) {
    redraw_custom_title_later();
    status_redraw_all();
  }
}

/// Restore ccline from save_ccline_in if did_save, else set cmdbuff=NULL.
void nvim_ccline_restore_or_clear(bool did_save, const void *save_ccline_in)
{
  if (did_save) {
    ccline = *(const CmdlineInfo *)save_ccline_in;
  } else {
    ccline.cmdbuff = NULL;
  }
}

