// ex_docmd.c: functions for executing an Ex command line.

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "auto/config.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/ui.h"
#include "nvim/api/vimscript.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/clipboard.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/debugger.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/fs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/input.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/search.h"
#include "nvim/shada.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_client.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/usercmd.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

// Rust implementations - declarations
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern int rs_get_copyID(void);
extern void rs_do_tag(char *tag, int type, int count, int forceit, bool verbose);
extern void rs_listdigraphs(int use_headers);
// Forward declarations for Phase 3 Rust exports (wrappers replaced by Rust)
int ends_excmd(int c);
char *find_nextcmd(const char *p);
char *check_nextcmd(char *p);
bool is_loclist_cmd(int cmdidx);
int get_pressedreturn(void);
bool expr_map_locked(void);
int one_letter_cmd(const char *p, int *idx);
int modifier_len(char *cmd);
bool is_cmd_ni(cmdidx_T cmdidx);
bool cmd_has_expr_args(cmdidx_T cmdidx);
cmdidx_T excmd_get_cmdidx(const char *cmd, size_t len);
char *get_command_name(expand_T *xp, int idx);
int get_bad_opt(const char *p, exarg_T *eap);
int getargopt(exarg_T *eap);
char *skip_cmd_arg(char *p, bool rembs);
int get_tabpage_arg(exarg_T *eap);
bool changedir_func(char *new_dir, CdScope scope);
// Forward declarations for Phase 1 (ex_docmd plan): static functions replaced by Rust exports
int check_more(bool message, bool forceit);
char *ex_range_without_command(exarg_T *eap);
char *get_argopt_name(expand_T *xp, int idx);
// Forward declarations for Phase 2 Rust exports (static functions replaced by Rust)
bool is_other_file(int fnum, char *ffname);
void msg_verbose_cmd(linenr_T lnum, char *cmd);
char *skip_colon_white(const char *p, bool skipleadingwhite);
void parse_register(exarg_T *eap);
int parse_count(exarg_T *eap, const char **errormsg, bool validate);
bool parse_bang(const exarg_T *eap, char **p);
void shift_cmd_args(exarg_T *eap);
int execute_cmd0(int *retv, exarg_T *eap, const char **errormsg, bool preview);
void profile_cmd(const exarg_T *eap, cstack_T *cstack, LineGetter fgetline, void *cookie);
bool skip_cmd(const exarg_T *eap);
void append_command(const char *cmd);
void get_flags(exarg_T *eap);
void correct_range(exarg_T *eap);
char *skip_grep_pat(exarg_T *eap);
// Phase 3 normal-mode Rust exports used by ex_docmd
extern void rs_set_cursor_for_append_to_line(void);
extern size_t rs_find_ident_under_cursor(char **text, int find_type);

// Rust implementation in nvim-event crate
extern MultiQueue *rs_loop_get_events(Loop *loop);
extern int rs_shada_read_everything(const char *fname, bool forceit, bool missing_ok);
extern int rs_shada_write_file(const char *file, bool nomerge);
extern void rs_ex_copy(linenr_T line1, linenr_T line2, linenr_T dest);
extern int rs_do_move(linenr_T line1, linenr_T line2, linenr_T dest);
#define loop_get_events(l) rs_loop_get_events(l)

static const char e_ambiguous_use_of_user_defined_command[]
  = N_("E464: Ambiguous use of user-defined command");
static const char e_no_call_stack_to_substitute_for_stack[]
  = N_("E489: No call stack to substitute for \"<stack>\"");
static const char e_not_an_editor_command[]
  = N_("E492: Not an editor command");
static const char e_no_autocommand_file_name_to_substitute_for_afile[]
  = N_("E495: No autocommand file name to substitute for \"<afile>\"");
static const char e_no_autocommand_buffer_number_to_substitute_for_abuf[]
  = N_("E496: No autocommand buffer number to substitute for \"<abuf>\"");
static const char e_no_autocommand_match_name_to_substitute_for_amatch[]
  = N_("E497: No autocommand match name to substitute for \"<amatch>\"");
static const char e_no_source_file_name_to_substitute_for_sfile[]
  = N_("E498: No :source file name to substitute for \"<sfile>\"");
static const char e_no_line_number_to_use_for_slnum[]
  = N_("E842: No line number to use for \"<slnum>\"");
static const char e_no_line_number_to_use_for_sflnum[]
  = N_("E961: No line number to use for \"<sflnum>\"");
static const char e_no_script_file_name_to_substitute_for_script[]
  = N_("E1274: No script file name to substitute for \"<script>\"");

static int quitmore = 0;
static bool ex_pressedreturn = false;

// Struct for storing a line inside a while/for loop
typedef struct {
  char *line;            // command line
  linenr_T lnum;                // sourcing_lnum of the line
} wcmd_T;

#define FREE_WCMD(wcmd) xfree((wcmd)->line)

/// Structure used to store info for line position in a while or for loop.
/// This is required, because do_one_cmd() may invoke ex_function(), which
/// reads more lines that may come from the while/for loop.
struct loop_cookie {
  garray_T *lines_gap;               // growarray with line info
  int current_line;                     // last read line from growarray
  int repeating;                        // true when looping a second time
  // When "repeating" is false use "getline" and "cookie" to get lines
  LineGetter lc_getline;
  void *cookie;
};

// Struct to save a few things while debugging.  Used in do_cmdline() only.
struct dbg_stuff {
  int trylevel;
  int force_abort;
  except_T *caught_stack;
  char *vv_exception;
  char *vv_throwpoint;
  int did_emsg;
  int got_int;
  bool did_throw;
  int need_rethrow;
  int check_cstack;
  except_T *current_exception;
};

#include "ex_docmd.c.generated.h"
extern int rs_win_valid(win_T *win);
extern int rs_only_one_window(void);
extern int rs_valid_tabpage(tabpage_T *tpc);

// Rust fold FFI declarations
extern int rs_foldManualAllowed(bool create);
extern void rs_foldCreate(win_T *wp, linenr_T start_lnum, linenr_T end_lnum);
extern void rs_opFoldRange(linenr_T first_lnum, linenr_T last_lnum, int opening, int recurse, bool had_visual);

extern int rs_get_scrolloff_value(win_T *wp);

// Declarations for ex_* functions now exported directly from Rust (Phase 1 migration).
// These functions used to be static C stubs calling rs_ex_*; now Rust exports them directly.
extern void ex_autocmd(exarg_T *eap);
extern void ex_doautocmd(exarg_T *eap);
extern void ex_bunload(exarg_T *eap);
extern void ex_buffer(exarg_T *eap);
extern void ex_bmodified(exarg_T *eap);
extern void ex_bnext(exarg_T *eap);
extern void ex_bprevious(exarg_T *eap);
extern void ex_brewind(exarg_T *eap);
extern void ex_blast(exarg_T *eap);
extern void ex_colorscheme(exarg_T *eap);
extern void ex_highlight(exarg_T *eap);
extern void ex_quit(exarg_T *eap);
extern void ex_quitall(exarg_T *eap);
extern void ex_close(exarg_T *eap);
extern void ex_tabclose(exarg_T *eap);
extern void ex_only(exarg_T *eap);
extern void ex_hide(exarg_T *eap);
extern void ex_exit(exarg_T *eap);
extern void ex_print(exarg_T *eap);
extern void ex_preserve(exarg_T *eap);
extern void ex_recover(exarg_T *eap);
extern void ex_wrongmodifier(exarg_T *eap);
extern void ex_tabmove(exarg_T *eap);
extern void ex_resize(exarg_T *eap);
extern void ex_edit(exarg_T *eap);
extern void ex_cd(exarg_T *eap);
extern void ex_pwd(exarg_T *eap);
extern void ex_equal(exarg_T *eap);
extern void ex_winsize(exarg_T *eap);
extern void ex_wincmd(exarg_T *eap);
extern void ex_put(exarg_T *eap);
extern void ex_iput(exarg_T *eap);
extern void ex_copymove(exarg_T *eap);
extern void ex_join(exarg_T *eap);
extern void ex_at(exarg_T *eap);
extern void ex_bang(exarg_T *eap);
extern void ex_wundo(exarg_T *eap);
extern void ex_rundo(exarg_T *eap);
extern void ex_redo(exarg_T *eap);
extern void ex_later(exarg_T *eap);
extern void ex_redir(exarg_T *eap);
extern void ex_redraw(exarg_T *eap);
extern void ex_redrawstatus(exarg_T *eap);
extern void ex_redrawtabline(exarg_T *eap);
extern void ex_mark(exarg_T *eap);
extern void ex_normal(exarg_T *eap);
extern void ex_startinsert(exarg_T *eap);
extern void ex_stopinsert(exarg_T *eap);
extern void ex_checkpath(exarg_T *eap);
extern void ex_psearch(exarg_T *eap);
extern void ex_shada(exarg_T *eap);
extern void ex_filetype(exarg_T *eap);
extern void ex_setfiletype(exarg_T *eap);
extern void ex_nohlsearch(exarg_T *eap);
extern void ex_folddo(exarg_T *eap);
extern void ex_nogui(exarg_T *eap);
extern void ex_popup(exarg_T *eap);
// Phase 3 (ex_docmd plan): C implementations replaced by Rust exports.
extern void ex_ni(exarg_T *eap);
extern void ex_script_ni(exarg_T *eap);
extern void not_exiting(void);
extern void ex_cquit(exarg_T *eap);
extern void ex_fclose(exarg_T *eap);
extern void ex_stop(exarg_T *eap);
extern void ex_submagic(exarg_T *eap);
extern int ex_submagic_preview(exarg_T *eap, int cmdpreview_ns, handle_T cmdpreview_bufnr);
// Phase 4 (ex_docmd plan): C implementations replaced by Rust exports.
extern ssize_t find_cmdline_var(const char *src, size_t *usedlen);
// Phase 5 (ex_docmd plan): C implementations replaced by Rust exports.
extern void ex_fold(exarg_T *eap);
extern void ex_foldopen(exarg_T *eap);
extern void ex_digraphs(exarg_T *eap);
extern void ex_mode(exarg_T *eap);
extern void ex_swapname(exarg_T *eap);
// Phase 6 (ex_docmd plan): C implementations replaced by Rust exports.
extern void ex_tabnext(exarg_T *eap);
// Phase 7 (ex_docmd plan): C implementations replaced by Rust exports.
extern void ex_undo(exarg_T *eap);
// Phase 8 (ex_docmd plan): C implementations replaced by Rust exports.
extern void ex_sleep(exarg_T *eap);
extern void do_sleep(int64_t msec, bool hide_cursor);
// Phase 10 (ex_docmd plan): C implementations replaced by Rust exports.
extern void ex_operators(exarg_T *eap);
// Wave 2, Phase 2: Public utility functions migrated to Rust.
extern void do_exedit(exarg_T *eap, win_T *old_curwin);
extern void ex_splitview(exarg_T *eap);
extern void ex_find(exarg_T *eap);
extern bool before_quit_autocmds(win_T *wp, bool quit_all, bool forceit);
extern void ex_win_close(int forceit, win_T *win, tabpage_T *tp);
extern void tabpage_close(int forceit);
extern void tabpage_close_other(tabpage_T *tp, int forceit);
extern void tabpage_new(void);
extern void handle_did_throw(void);
// Wave 2, Phase 1: Static command functions migrated to Rust.
extern void ex_goto(exarg_T *eap);
extern void ex_tag(exarg_T *eap);
extern void ex_ptag(exarg_T *eap);
extern void ex_stag(exarg_T *eap);
extern void ex_pclose(exarg_T *eap);
extern void ex_pedit(exarg_T *eap);
extern void ex_pbuffer(exarg_T *eap);
extern void ex_findpat(exarg_T *eap);
extern void ex_tabs(exarg_T *eap);
extern void ex_syncbind(exarg_T *eap);
extern void ex_read(exarg_T *eap);
extern void ex_detach(exarg_T *eap);
extern void ex_connect(exarg_T *eap);
extern void ex_checkhealth(exarg_T *eap);
extern void ex_terminal(exarg_T *eap);
extern void ex_restart(exarg_T *eap);
extern void ex_tabonly(exarg_T *eap);
// Rust-implemented nvim_docmd helpers (previously C _impl bodies).
extern void nvim_docmd_ex_may_print_impl(exarg_T *eap);
extern int nvim_docmd_vim_mkdir_emsg_impl(const char *name, int prot);
extern FILE *nvim_docmd_open_exfile_impl(char *fname, int forceit, char *mode);
extern void nvim_docmd_update_topline_cursor_impl(void);
extern char *nvim_docmd_replace_makeprg_impl(exarg_T *eap, char *arg, char **cmdlinep);
extern void nvim_docmd_close_redir_impl(void);

// Declare cmdnames[].
#include "ex_cmds_defs.generated.h"

// Rust FFI declarations (typval functions migrated to Rust)
extern const char *tv_list_find_str(list_T *l, int n);

// Rust FFI declarations (memline crate)
extern void rs_goto_byte(int cnt);

extern int rs_magic_isset(void);

// Rust FFI declarations (window wrappers removed)
extern void rs_do_window(int nchar, int Prenum, int xchar);
extern tabpage_T *rs_find_tabpage(int n);
extern int rs_tabpage_index(tabpage_T *ftp);
extern void rs_win_setheight_win(int height, win_T *win);
extern void rs_win_setwidth_win(int width, win_T *wp);
extern int rs_get_vtopline(win_T *wp);

// Rust FFI declarations still needed from C
extern void verify_command(const char *cmd);

// Helper function to get first character of command name for Rust FFI
// Returns 0 if cmdidx is out of bounds
int cmdname_first_char(int cmdidx)
{
  if (cmdidx < 0 || cmdidx >= CMD_SIZE) {
    return 0;
  }
  return (unsigned char)cmdnames[cmdidx].cmd_name[0];
}

static char dollar_command[2] = { '$', 0 };

static void save_dbg_stuff(struct dbg_stuff *dsp)
{
  dsp->trylevel = trylevel;
  trylevel = 0;
  dsp->force_abort = force_abort;
  force_abort = false;
  dsp->caught_stack = caught_stack;
  caught_stack = NULL;
  dsp->vv_exception = v_exception(NULL);
  dsp->vv_throwpoint = v_throwpoint(NULL);

  // Necessary for debugging an inactive ":catch", ":finally", ":endtry".
  dsp->did_emsg = did_emsg;
  did_emsg = false;
  dsp->got_int = got_int;
  got_int = false;
  dsp->did_throw = did_throw;
  did_throw = false;
  dsp->need_rethrow = need_rethrow;
  need_rethrow = false;
  dsp->check_cstack = check_cstack;
  check_cstack = false;
  dsp->current_exception = current_exception;
  current_exception = NULL;
}

static void restore_dbg_stuff(struct dbg_stuff *dsp)
{
  suppress_errthrow = false;
  trylevel = dsp->trylevel;
  force_abort = dsp->force_abort;
  caught_stack = dsp->caught_stack;
  v_exception(dsp->vv_exception);
  v_throwpoint(dsp->vv_throwpoint);
  did_emsg = dsp->did_emsg;
  got_int = dsp->got_int;
  did_throw = dsp->did_throw;
  need_rethrow = dsp->need_rethrow;
  check_cstack = dsp->check_cstack;
  current_exception = dsp->current_exception;
}

// do_exmode: implemented in Rust (Phase 6). Symbol exported via #[no_mangle].

static int cmdline_call_depth = 0;  ///< recursiveness

/// Start executing an Ex command line.
///
/// @return  FAIL if too recursive, OK otherwise.
static int do_cmdline_start(void)
{
  assert(cmdline_call_depth >= 0);
  // It's possible to create an endless loop with ":execute", catch that
  // here.  The value of 200 allows nested function calls, ":source", etc.
  // Allow 200 or 'maxfuncdepth', whatever is larger.
  if (cmdline_call_depth >= 200 && cmdline_call_depth >= p_mfd) {
    return FAIL;
  }
  cmdline_call_depth++;
  start_batch_changes();
  return OK;
}

/// End executing an Ex command line.
static void do_cmdline_end(void)
{
  cmdline_call_depth--;
  assert(cmdline_call_depth >= 0);
  end_batch_changes();
}

// do_cmdline_cmd: implemented in Rust (Phase 6). Symbol exported via #[no_mangle].

/// do_cmdline(): execute one Ex command line
///
/// 1. Execute "cmdline" when it is not NULL.
///    If "cmdline" is NULL, or more lines are needed, fgetline() is used.
/// 2. Split up in parts separated with '|'.
///
/// This function can be called recursively!
///
/// flags:
///   DOCMD_VERBOSE  - The command will be included in the error message.
///   DOCMD_NOWAIT   - Don't call wait_return() and friends.
///   DOCMD_REPEAT   - Repeat execution until fgetline() returns NULL.
///   DOCMD_KEYTYPED - Don't reset KeyTyped.
///   DOCMD_EXCRESET - Reset the exception environment (used for debugging).
///   DOCMD_KEEPLINE - Store first typed line (for repeating with ".").
///
/// @param cookie  argument for fgetline()
///
/// @return FAIL if cmdline could not be executed, OK otherwise
int do_cmdline(char *cmdline, LineGetter fgetline, void *cookie, int flags)
{
  char *next_cmdline;                   // next cmd to execute
  char *cmdline_copy = NULL;            // copy of cmd line
  bool used_getline = false;            // used "fgetline" to obtain command
  static int recursive = 0;             // recursive depth
  bool msg_didout_before_start = false;
  int count = 0;                        // line number count
  bool did_inc = false;                 // incremented RedrawingDisabled
  bool did_block = false;               // emitted cmdline_block event
  int retval = OK;
  cstack_T cstack = {                   // conditional stack
    .cs_idx = -1,
  };
  garray_T lines_ga;                    // keep lines for ":while"/":for"
  int current_line = 0;                 // active line in lines_ga
  char *fname = NULL;                   // function or script name
  linenr_T *breakpoint = NULL;          // ptr to breakpoint field in cookie
  int *dbg_tick = NULL;                 // ptr to dbg_tick field in cookie
  struct dbg_stuff debug_saved;         // saved things for debug mode
  msglist_T *private_msg_list;

  // "fgetline" and "cookie" passed to do_one_cmd()
  char *(*cmd_getline)(int, void *, int, bool);
  void *cmd_cookie;
  struct loop_cookie cmd_loop_cookie;

  // For every pair of do_cmdline()/do_one_cmd() calls, use an extra memory
  // location for storing error messages to be converted to an exception.
  // This ensures that the do_errthrow() call in do_one_cmd() does not
  // combine the messages stored by an earlier invocation of do_one_cmd()
  // with the command name of the later one.  This would happen when
  // BufWritePost autocommands are executed after a write error.
  msglist_T **saved_msg_list = msg_list;
  msg_list = &private_msg_list;
  private_msg_list = NULL;

  if (do_cmdline_start() == FAIL) {
    emsg(_(e_command_too_recursive));
    // When converting to an exception, we do not include the command name
    // since this is not an error of the specific command.
    do_errthrow((cstack_T *)NULL, NULL);
    msg_list = saved_msg_list;
    return FAIL;
  }

  ga_init(&lines_ga, (int)sizeof(wcmd_T), 10);

  void *real_cookie = getline_cookie(fgetline, cookie);

  // Inside a function use a higher nesting level.
  bool getline_is_func = getline_equal(fgetline, cookie, get_func_line);
  if (getline_is_func && ex_nesting_level == func_level(real_cookie)) {
    ex_nesting_level++;
  }

  // Get the function or script name and the address where the next breakpoint
  // line and the debug tick for a function or script are stored.
  if (getline_is_func) {
    fname = func_name(real_cookie);
    breakpoint = func_breakpoint(real_cookie);
    dbg_tick = func_dbg_tick(real_cookie);
  } else if (getline_equal(fgetline, cookie, getsourceline)) {
    fname = SOURCING_NAME;
    breakpoint = source_breakpoint(real_cookie);
    dbg_tick = source_dbg_tick(real_cookie);
  }

  // Initialize "force_abort"  and "suppress_errthrow" at the top level.
  if (!recursive) {
    force_abort = false;
    suppress_errthrow = false;
  }

  // If requested, store and reset the global values controlling the
  // exception handling (used when debugging).  Otherwise clear it to avoid
  // a bogus compiler warning when the optimizer uses inline functions...
  if (flags & DOCMD_EXCRESET) {
    save_dbg_stuff(&debug_saved);
  } else {
    CLEAR_FIELD(debug_saved);
  }

  int initial_trylevel = trylevel;

  // "did_throw" will be set to true when an exception is being thrown.
  did_throw = false;
  // "did_emsg" will be set to true when emsg() is used, in which case we
  // cancel the whole command line, and any if/endif or loop.
  // If force_abort is set, we cancel everything.
  did_emsg = false;

  // KeyTyped is only set when calling vgetc().  Reset it here when not
  // calling vgetc() (sourced command lines).
  if (!(flags & DOCMD_KEYTYPED)
      && !getline_equal(fgetline, cookie, getexline)) {
    KeyTyped = false;
  }

  // Continue executing command lines:
  // - when inside an ":if", ":while" or ":for"
  // - for multiple commands on one line, separated with '|'
  // - when repeating until there are no more lines (for ":source")
  next_cmdline = cmdline;
  do {
    getline_is_func = getline_equal(fgetline, cookie, get_func_line);

    // stop skipping cmds for an error msg after all endif/while/for
    if (next_cmdline == NULL
        && !force_abort
        && cstack.cs_idx < 0
        && !(getline_is_func
             && func_has_abort(real_cookie))) {
      did_emsg = false;
    }

    // 1. If repeating a line in a loop, get a line from lines_ga.
    // 2. If no line given: Get an allocated line with fgetline().
    // 3. If a line is given: Make a copy, so we can mess with it.

    // 1. If repeating, get a previous line from lines_ga.
    if (cstack.cs_looplevel > 0 && current_line < lines_ga.ga_len) {
      // Each '|' separated command is stored separately in lines_ga, to
      // be able to jump to it.  Don't use next_cmdline now.
      XFREE_CLEAR(cmdline_copy);

      // Check if a function has returned or, unless it has an unclosed
      // try conditional, aborted.
      if (getline_is_func) {
        if (do_profiling == PROF_YES) {
          func_line_end(real_cookie);
        }
        if (func_has_ended(real_cookie)) {
          retval = FAIL;
          break;
        }
      } else if (do_profiling == PROF_YES
                 && getline_equal(fgetline, cookie, getsourceline)) {
        script_line_end();
      }

      // Check if a sourced file hit a ":finish" command.
      if (source_finished(fgetline, cookie)) {
        retval = FAIL;
        break;
      }

      // If breakpoints have been added/deleted need to check for it.
      if (breakpoint != NULL && dbg_tick != NULL
          && *dbg_tick != debug_tick) {
        *breakpoint = dbg_find_breakpoint(getline_equal(fgetline, cookie, getsourceline),
                                          fname, SOURCING_LNUM);
        *dbg_tick = debug_tick;
      }

      next_cmdline = ((wcmd_T *)(lines_ga.ga_data))[current_line].line;
      SOURCING_LNUM = ((wcmd_T *)(lines_ga.ga_data))[current_line].lnum;

      // Did we encounter a breakpoint?
      if (breakpoint != NULL && *breakpoint != 0 && *breakpoint <= SOURCING_LNUM) {
        dbg_breakpoint(fname, SOURCING_LNUM);
        // Find next breakpoint.
        *breakpoint = dbg_find_breakpoint(getline_equal(fgetline, cookie, getsourceline),
                                          fname, SOURCING_LNUM);
        *dbg_tick = debug_tick;
      }
      if (do_profiling == PROF_YES) {
        if (getline_is_func) {
          func_line_start(real_cookie);
        } else if (getline_equal(fgetline, cookie, getsourceline)) {
          script_line_start();
        }
      }
    }

    // 2. If no line given, get an allocated line with fgetline().
    if (next_cmdline == NULL) {
      int indent = cstack.cs_idx < 0 ? 0 : (cstack.cs_idx + 1) * 2;

      if (count == 1 && getline_equal(fgetline, cookie, getexline)) {
        if (ui_has(kUICmdline)) {
          // Emit cmdline_block event for loop/conditional block.
          ui_ext_cmdline_block_append(0, last_cmdline);
          did_block = true;
        }
        // Need to set msg_didout for the first line after an ":if",
        // otherwise the ":if" will be overwritten.
        msg_didout = true;
      }

      if (fgetline == NULL || (next_cmdline = fgetline(':', cookie, indent, true)) == NULL) {
        // Don't call wait_return() for aborted command line.  The NULL
        // returned for the end of a sourced file or executed function
        // doesn't do this.
        if (KeyTyped && !(flags & DOCMD_REPEAT)) {
          need_wait_return = false;
        }
        retval = FAIL;
        break;
      }
      used_getline = true;

      // Emit all but the first cmdline_block event immediately; waiting until after
      // command execution would mess up event ordering with nested command lines.
      if (ui_has(kUICmdline) && count > 0 && getline_equal(fgetline, cookie, getexline)) {
        ui_ext_cmdline_block_append((size_t)indent, next_cmdline);
      }

      // Keep the first typed line.  Clear it when more lines are typed.
      if (flags & DOCMD_KEEPLINE) {
        xfree(repeat_cmdline);
        if (count == 0) {
          repeat_cmdline = xstrdup(next_cmdline);
        } else {
          repeat_cmdline = NULL;
        }
      }
    } else if (cmdline_copy == NULL) {
      // 3. Make a copy of the command so we can mess with it.
      next_cmdline = xstrdup(next_cmdline);
    }
    cmdline_copy = next_cmdline;

    int current_line_before = 0;
    // Inside a while/for loop, and when the command looks like a ":while"
    // or ":for", the line is stored, because we may need it later when
    // looping.
    //
    // When there is a '|' and another command, it is stored separately,
    // because we need to be able to jump back to it from an
    // :endwhile/:endfor.
    //
    // Pass a different "fgetline" function to do_one_cmd() below,
    // that it stores lines in or reads them from "lines_ga".  Makes it
    // possible to define a function inside a while/for loop.
    if ((cstack.cs_looplevel > 0 || has_loop_cmd(next_cmdline))) {
      cmd_getline = get_loop_line;
      cmd_cookie = (void *)&cmd_loop_cookie;
      cmd_loop_cookie.lines_gap = &lines_ga;
      cmd_loop_cookie.current_line = current_line;
      cmd_loop_cookie.lc_getline = fgetline;
      cmd_loop_cookie.cookie = cookie;
      cmd_loop_cookie.repeating = (current_line < lines_ga.ga_len);

      // Save the current line when encountering it the first time.
      if (current_line == lines_ga.ga_len) {
        store_loop_line(&lines_ga, next_cmdline);
      }
      current_line_before = current_line;
    } else {
      cmd_getline = fgetline;
      cmd_cookie = cookie;
    }

    did_endif = false;

    if (count++ == 0) {
      // All output from the commands is put below each other, without
      // waiting for a return. Don't do this when executing commands
      // from a script or when being called recursive (e.g. for ":e
      // +command file").
      if (!(flags & DOCMD_NOWAIT) && !recursive) {
        msg_didout_before_start = msg_didout;
        msg_didany = false;         // no output yet
        msg_start();
        msg_scroll = true;          // put messages below each other
        no_wait_return++;           // don't wait for return until finished
        RedrawingDisabled++;
        did_inc = true;
      }
    }

    if ((p_verbose >= 15 && SOURCING_NAME != NULL) || p_verbose >= 16) {
      msg_verbose_cmd(SOURCING_LNUM, cmdline_copy);
    }

    // 2. Execute one '|' separated command.
    //    do_one_cmd() will return NULL if there is no trailing '|'.
    //    "cmdline_copy" can change, e.g. for '%' and '#' expansion.
    recursive++;
    next_cmdline = do_one_cmd(&cmdline_copy, flags, &cstack, cmd_getline, cmd_cookie);
    recursive--;

    if (cmd_cookie == (void *)&cmd_loop_cookie) {
      // Use "current_line" from "cmd_loop_cookie", it may have been
      // incremented when defining a function.
      current_line = cmd_loop_cookie.current_line;
    }

    if (next_cmdline == NULL) {
      XFREE_CLEAR(cmdline_copy);

      // If the command was typed, remember it for the ':' register.
      // Do this AFTER executing the command to make :@: work.
      if (getline_equal(fgetline, cookie, getexline) && new_last_cmdline != NULL) {
        xfree(last_cmdline);
        last_cmdline = new_last_cmdline;
        new_last_cmdline = NULL;
      }
    } else {
      // need to copy the command after the '|' to cmdline_copy, for the
      // next do_one_cmd()
      STRMOVE(cmdline_copy, next_cmdline);
      next_cmdline = cmdline_copy;
    }

    // reset did_emsg for a function that is not aborted by an error
    if (did_emsg && !force_abort
        && getline_equal(fgetline, cookie, get_func_line)
        && !func_has_abort(real_cookie)) {
      did_emsg = false;
    }

    if (cstack.cs_looplevel > 0) {
      current_line++;

      // An ":endwhile", ":endfor" and ":continue" is handled here.
      // If we were executing commands, jump back to the ":while" or
      // ":for".
      // If we were not executing commands, decrement cs_looplevel.
      if (cstack.cs_lflags & (CSL_HAD_CONT | CSL_HAD_ENDLOOP)) {
        cstack.cs_lflags &= ~(CSL_HAD_CONT | CSL_HAD_ENDLOOP);

        // Jump back to the matching ":while" or ":for".  Be careful
        // not to use a cs_line[] from an entry that isn't a ":while"
        // or ":for": It would make "current_line" invalid and can
        // cause a crash.
        if (!did_emsg && !got_int && !did_throw
            && cstack.cs_idx >= 0
            && (cstack.cs_flags[cstack.cs_idx]
                & (CSF_WHILE | CSF_FOR))
            && cstack.cs_line[cstack.cs_idx] >= 0
            && (cstack.cs_flags[cstack.cs_idx] & CSF_ACTIVE)) {
          current_line = cstack.cs_line[cstack.cs_idx];
          // remember we jumped there
          cstack.cs_lflags |= CSL_HAD_LOOP;
          line_breakcheck();                    // check if CTRL-C typed

          // Check for the next breakpoint at or after the ":while"
          // or ":for".
          if (breakpoint != NULL && lines_ga.ga_len > current_line) {
            *breakpoint = dbg_find_breakpoint(getline_equal(fgetline, cookie, getsourceline), fname,
                                              ((wcmd_T *)lines_ga.ga_data)[current_line].lnum - 1);
            *dbg_tick = debug_tick;
          }
        } else {
          // can only get here with ":endwhile" or ":endfor"
          if (cstack.cs_idx >= 0) {
            rewind_conditionals(&cstack, cstack.cs_idx - 1,
                                CSF_WHILE | CSF_FOR, &cstack.cs_looplevel);
          }
        }
      } else if (cstack.cs_lflags & CSL_HAD_LOOP) {
        // For a ":while" or ":for" we need to remember the line number.
        cstack.cs_lflags &= ~CSL_HAD_LOOP;
        cstack.cs_line[cstack.cs_idx] = current_line_before;
      }
    }

    // When not inside any ":while" loop, clear remembered lines.
    if (cstack.cs_looplevel == 0) {
      if (!GA_EMPTY(&lines_ga)) {
        SOURCING_LNUM = ((wcmd_T *)lines_ga.ga_data)[lines_ga.ga_len - 1].lnum;
        GA_DEEP_CLEAR(&lines_ga, wcmd_T, FREE_WCMD);
      }
      current_line = 0;
    }

    // A ":finally" makes did_emsg, got_int and did_throw pending for
    // being restored at the ":endtry".  Reset them here and set the
    // ACTIVE and FINALLY flags, so that the finally clause gets executed.
    // This includes the case where a missing ":endif", ":endwhile" or
    // ":endfor" was detected by the ":finally" itself.
    if (cstack.cs_lflags & CSL_HAD_FINA) {
      cstack.cs_lflags &= ~CSL_HAD_FINA;
      report_make_pending((cstack.cs_pending[cstack.cs_idx]
                           & (CSTP_ERROR | CSTP_INTERRUPT | CSTP_THROW)),
                          did_throw ? current_exception : NULL);
      did_emsg = got_int = did_throw = false;
      cstack.cs_flags[cstack.cs_idx] |= CSF_ACTIVE | CSF_FINALLY;
    }

    // Update global "trylevel" for recursive calls to do_cmdline() from
    // within this loop.
    trylevel = initial_trylevel + cstack.cs_trylevel;

    // If the outermost try conditional (across function calls and sourced
    // files) is aborted because of an error, an interrupt, or an uncaught
    // exception, cancel everything.  If it is left normally, reset
    // force_abort to get the non-EH compatible abortion behavior for
    // the rest of the script.
    if (trylevel == 0 && !did_emsg && !got_int && !did_throw) {
      force_abort = false;
    }

    // Convert an interrupt to an exception if appropriate.
    do_intthrow(&cstack);

    // Continue executing command lines when:
    // - no CTRL-C typed, no aborting error, no exception thrown or try
    //   conditionals need to be checked for executing finally clauses or
    //   catching an interrupt exception
    // - didn't get an error message or lines are not typed
    // - there is a command after '|', inside a :if, :while, :for or :try, or
    //   looping for ":source" command or function call.
  } while (!((got_int || (did_emsg && force_abort) || did_throw)
             && cstack.cs_trylevel == 0)
           && !(did_emsg
                // Keep going when inside try/catch, so that the error can be
                // deal with, except when it is a syntax error, it may cause
                // the :endtry to be missed.
                && (cstack.cs_trylevel == 0 || did_emsg_syntax)
                && used_getline
                && getline_equal(fgetline, cookie, getexline))
           && (next_cmdline != NULL
               || cstack.cs_idx >= 0
               || (flags & DOCMD_REPEAT)));

  xfree(cmdline_copy);
  did_emsg_syntax = false;
  GA_DEEP_CLEAR(&lines_ga, wcmd_T, FREE_WCMD);

  if (cstack.cs_idx >= 0) {
    // If a sourced file or executed function ran to its end, report the
    // unclosed conditional.
    if (!got_int && !did_throw && !aborting()
        && ((getline_equal(fgetline, cookie, getsourceline)
             && !source_finished(fgetline, cookie))
            || (getline_equal(fgetline, cookie, get_func_line)
                && !func_has_ended(real_cookie)))) {
      if (cstack.cs_flags[cstack.cs_idx] & CSF_TRY) {
        emsg(_(e_endtry));
      } else if (cstack.cs_flags[cstack.cs_idx] & CSF_WHILE) {
        emsg(_(e_endwhile));
      } else if (cstack.cs_flags[cstack.cs_idx] & CSF_FOR) {
        emsg(_(e_endfor));
      } else {
        emsg(_(e_endif));
      }
    }

    // Reset "trylevel" in case of a ":finish" or ":return" or a missing
    // ":endtry" in a sourced file or executed function.  If the try
    // conditional is in its finally clause, ignore anything pending.
    // If it is in a catch clause, finish the caught exception.
    // Also cleanup any "cs_forinfo" structures.
    do {
      int idx = cleanup_conditionals(&cstack, 0, true);

      if (idx >= 0) {
        idx--;              // remove try block not in its finally clause
      }
      rewind_conditionals(&cstack, idx, CSF_WHILE | CSF_FOR,
                          &cstack.cs_looplevel);
    } while (cstack.cs_idx >= 0);
    trylevel = initial_trylevel;
  }

  // If a missing ":endtry", ":endwhile", ":endfor", or ":endif" or a memory
  // lack was reported above and the error message is to be converted to an
  // exception, do this now after rewinding the cstack.
  do_errthrow(&cstack, getline_equal(fgetline, cookie, get_func_line) ? "endfunction" : NULL);

  if (trylevel == 0) {
    // When an exception is being thrown out of the outermost try
    // conditional, discard the uncaught exception, disable the conversion
    // of interrupts or errors to exceptions, and ensure that no more
    // commands are executed.
    if (did_throw) {
      handle_did_throw();
    } else if (got_int || (did_emsg && force_abort)) {
      // On an interrupt or an aborting error not converted to an exception,
      // disable the conversion of errors to exceptions.  (Interrupts are not
      // converted any more, here.) This enables also the interrupt message
      // when force_abort is set and did_emsg unset in case of an interrupt
      // from a finally clause after an error.
      suppress_errthrow = true;
    }
  }

  // The current cstack will be freed when do_cmdline() returns.  An uncaught
  // exception will have to be rethrown in the previous cstack.  If a function
  // has just returned or a script file was just finished and the previous
  // cstack belongs to the same function or, respectively, script file, it
  // will have to be checked for finally clauses to be executed due to the
  // ":return" or ":finish".  This is done in do_one_cmd().
  if (did_throw) {
    need_rethrow = true;
  }
  if ((getline_equal(fgetline, cookie, getsourceline)
       && ex_nesting_level > source_level(real_cookie))
      || (getline_equal(fgetline, cookie, get_func_line)
          && ex_nesting_level > func_level(real_cookie) + 1)) {
    if (!did_throw) {
      check_cstack = true;
    }
  } else {
    // When leaving a function, reduce nesting level.
    if (getline_equal(fgetline, cookie, get_func_line)) {
      ex_nesting_level--;
    }
    // Go to debug mode when returning from a function in which we are
    // single-stepping.
    if ((getline_equal(fgetline, cookie, getsourceline)
         || getline_equal(fgetline, cookie, get_func_line))
        && ex_nesting_level + 1 <= debug_break_level) {
      do_debug(getline_equal(fgetline, cookie, getsourceline)
               ? _("End of sourced file")
               : _("End of function"));
    }
  }

  // Restore the exception environment (done after returning from the
  // debugger).
  if (flags & DOCMD_EXCRESET) {
    restore_dbg_stuff(&debug_saved);
  }

  msg_list = saved_msg_list;

  // Cleanup if "cs_emsg_silent_list" remains.
  if (cstack.cs_emsg_silent_list != NULL) {
    eslist_T *temp;
    for (eslist_T *elem = cstack.cs_emsg_silent_list; elem != NULL; elem = temp) {
      temp = elem->next;
      xfree(elem);
    }
  }

  // If there was too much output to fit on the command line, ask the user to
  // hit return before redrawing the screen. With the ":global" command we do
  // this only once after the command is finished.
  if (did_inc) {
    RedrawingDisabled--;
    no_wait_return--;
    msg_scroll = false;

    // When just finished an ":if"-":else" which was typed, no need to
    // wait for hit-return.  Also for an error situation.
    if (retval == FAIL
        || (did_endif && KeyTyped && !did_emsg)) {
      need_wait_return = false;
      msg_didany = false;               // don't wait when restarting edit
    } else if (need_wait_return) {
      // The msg_start() above clears msg_didout. The wait_return() we do
      // here should not overwrite the command that may be shown before
      // doing that.
      msg_didout |= msg_didout_before_start;
      wait_return(false);
    }
  }

  if (did_block) {
    ui_ext_cmdline_block_leave();
  }

  did_endif = false;    // in case do_cmdline used recursively

  do_cmdline_end();
  return retval;
}

/// Handle when "did_throw" is set after executing commands.
/// handle_did_throw implementation. Called by Rust handle_did_throw.
void nvim_docmd_handle_did_throw_impl(void)
{
  assert(current_exception != NULL);
  char *p = NULL;
  msglist_T *messages = NULL;

  // If the uncaught exception is a user exception, report it as an
  // error.  If it is an error exception, display the saved error
  // message now.  For an interrupt exception, do nothing; the
  // interrupt message is given elsewhere.
  switch (current_exception->type) {
  case ET_USER:
    vim_snprintf(IObuff, IOSIZE,
                 _("E605: Exception not caught: %s"),
                 current_exception->value);
    p = xstrdup(IObuff);
    break;
  case ET_ERROR:
    messages = current_exception->messages;
    current_exception->messages = NULL;
    break;
  case ET_INTERRUPT:
    break;
  }

  estack_push(ETYPE_EXCEPT, current_exception->throw_name, current_exception->throw_lnum);
  current_exception->throw_name = NULL;

  discard_current_exception();              // uses IObuff if 'verbose'

  // If "silent!" is active the uncaught exception is not fatal.
  if (emsg_silent == 0) {
    suppress_errthrow = true;
    force_abort = true;
  }

  if (messages != NULL) {
    do {
      msglist_T *next = messages->next;
      emsg_multiline(messages->msg, "emsg", HLF_E, messages->multiline);
      xfree(messages->msg);
      xfree(messages->sfile);
      xfree(messages);
      messages = next;
    } while (messages != NULL);
  } else if (p != NULL) {
    emsg(p);
    xfree(p);
  }
  xfree(SOURCING_NAME);
  estack_pop();
}

/// Obtain a line when inside a ":while" or ":for" loop.
static char *get_loop_line(int c, void *cookie, int indent, bool do_concat)
{
  struct loop_cookie *cp = (struct loop_cookie *)cookie;

  if (cp->current_line + 1 >= cp->lines_gap->ga_len) {
    if (cp->repeating) {
      return NULL;              // trying to read past ":endwhile"/":endfor"
    }
    char *line;
    // First time inside the ":while"/":for": get line normally.
    if (cp->lc_getline == NULL) {
      line = getcmdline(c, 0, indent, do_concat);
    } else {
      line = cp->lc_getline(c, cp->cookie, indent, do_concat);
    }
    if (line != NULL) {
      store_loop_line(cp->lines_gap, line);
      cp->current_line++;
    }

    return line;
  }

  KeyTyped = false;
  cp->current_line++;
  wcmd_T *wp = (wcmd_T *)(cp->lines_gap->ga_data) + cp->current_line;
  SOURCING_LNUM = wp->lnum;
  return xstrdup(wp->line);
}

/// Store a line in "gap" so that a ":while" loop can execute it again.
static void store_loop_line(garray_T *gap, char *line)
{
  wcmd_T *p = GA_APPEND_VIA_PTR(wcmd_T, gap);
  p->line = xstrdup(line);
  p->lnum = SOURCING_LNUM;
}

/// If "fgetline" is get_loop_line(), return true if the getline it uses equals
/// "func".  * Otherwise return true when "fgetline" equals "func".
///
/// @param cookie  argument for fgetline()
bool getline_equal(LineGetter fgetline, void *cookie, LineGetter func)
{
  // When "fgetline" is "get_loop_line()" use the "cookie" to find the
  // function that's originally used to obtain the lines.  This may be
  // nested several levels.
  LineGetter gp = fgetline;
  struct loop_cookie *cp = (struct loop_cookie *)cookie;
  while (gp == get_loop_line) {
    gp = cp->lc_getline;
    cp = cp->cookie;
  }
  return gp == func;
}

/// If "fgetline" is get_loop_line(), return the cookie used by the original
/// getline function.  Otherwise return "cookie".
///
/// @param cookie  argument for fgetline()
void *getline_cookie(LineGetter fgetline, void *cookie)
{
  // When "fgetline" is "get_loop_line()" use the "cookie" to find the
  // cookie that's originally used to obtain the lines.  This may be nested
  // several levels.
  LineGetter gp = fgetline;
  struct loop_cookie *cp = (struct loop_cookie *)cookie;
  while (gp == get_loop_line) {
    gp = cp->lc_getline;
    cp = cp->cookie;
  }
  return cp;
}

/// Helper function to apply an offset for buffer commands, i.e. ":bdelete",
/// ":bwipeout", etc.
///
/// @return  the buffer number.
// nvim_docmd_compute_buffer_local_count_impl is implemented in Rust (address.rs).
extern int nvim_docmd_compute_buffer_local_count_impl(cmd_addr_T addr_type, linenr_T lnum, int offset);

/// @return  the window number of "win" or,
///          the number of windows if "win" is NULL
static int current_win_nr(const win_T *win)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  int nr = 0;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    nr++;
    if (wp == win) {
      break;
    }
  }
  return nr;
}

static int current_tab_nr(tabpage_T *tab)
{
  int nr = 0;

  FOR_ALL_TABS(tp) {
    nr++;
    if (tp == tab) {
      break;
    }
  }
  return nr;
}

#define CURRENT_WIN_NR current_win_nr(curwin)
#define LAST_WIN_NR current_win_nr(NULL)
#define CURRENT_TAB_NR current_tab_nr(curtab)
#define LAST_TAB_NR current_tab_nr(NULL)

// get_wincmd_addr_type: now in Rust (rs_set_cmd_addr_type calls it internally)


// Find the command name after skipping range specifiers.
static char *find_excmd_after_range(exarg_T *eap)
{
  // Save location after command modifiers.
  char *cmd = eap->cmd;
  eap->cmd = skip_range(eap->cmd, NULL);
  if (*eap->cmd == '*') {
    eap->cmd = skipwhite(eap->cmd + 1);
  }
  char *p = find_ex_command(eap, NULL);
  eap->cmd = cmd;  // Restore original position for address parsing
  return p;
}


/// Execute one Ex command.
///
/// If "flags" has DOCMD_VERBOSE, the command will be included in the error
/// message.
///
/// 1. skip comment lines and leading space
/// 2. handle command modifiers
/// 3. skip over the range to find the command
/// 4. parse the range
/// 5. parse the command
/// 6. parse arguments
/// 7. switch on command name
///
/// Note: "fgetline" can be NULL.
///
/// This function may be called recursively!
///
/// @param cookie  argument for fgetline()
static char *do_one_cmd(char **cmdlinep, int flags, cstack_T *cstack, LineGetter fgetline,
                        void *cookie)
{
  const char *errormsg = NULL;  // error message
  const int save_reg_executing = reg_executing;
  const bool save_pending_end_reg_executing = pending_end_reg_executing;

  exarg_T ea = {
    .line1 = 1,
    .line2 = 1,
  };
  ex_nesting_level++;

  // When the last file has not been edited :q has to be typed twice.
  if (quitmore
      // avoid that a function call in 'statusline' does this
      && !getline_equal(fgetline, cookie, get_func_line)
      // avoid that an autocommand, e.g. QuitPre, does this
      && !getline_equal(fgetline, cookie, getnextac)) {
    quitmore--;
  }

  // Reset browse, confirm, etc..  They are restored when returning, for
  // recursive calls.
  cmdmod_T save_cmdmod = cmdmod;

  // "#!anything" is handled like a comment.
  if ((*cmdlinep)[0] == '#' && (*cmdlinep)[1] == '!') {
    goto doend;
  }

  // 1. Skip comment lines and leading white space and colons.
  // 2. Handle command modifiers.

  // The "ea" structure holds the arguments that can be used.
  ea.cmd = *cmdlinep;
  ea.cmdlinep = cmdlinep;
  ea.ea_getline = fgetline;
  ea.cookie = cookie;
  ea.cstack = cstack;

  if (parse_command_modifiers(&ea, &errormsg, &cmdmod, false) == FAIL) {
    goto doend;
  }
  nvim_docmd_apply_cmdmod_impl(&cmdmod);

  char *after_modifier = ea.cmd;

  ea.skip = (did_emsg
             || got_int
             || did_throw
             || (cstack->cs_idx >= 0
                 && !(cstack->cs_flags[cstack->cs_idx] & CSF_ACTIVE)));

  // 3. Skip over the range to find the command. Let "p" point to after it.
  //
  // We need the command to know what kind of range it uses.
  char *p = find_excmd_after_range(&ea);
  profile_cmd(&ea, cstack, fgetline, cookie);

  if (!exiting) {
    // May go to debug mode.  If this happens and the ">quit" debug command is
    // used, throw an interrupt exception and skip the next command.
    dbg_check_breakpoint(&ea);
  }
  if (!ea.skip && got_int) {
    ea.skip = true;
    do_intthrow(cstack);
  }

  // 4. Parse a range specifier of the form: addr [,addr] [;addr] ..
  //
  // where 'addr' is:
  //
  // %          (entire file)
  // $  [+-NUM]
  // 'x [+-NUM] (where x denotes a currently defined mark)
  // .  [+-NUM]
  // [+-NUM]..
  // NUM
  //
  // The ea.cmd pointer is updated to point to the first character following the
  // range spec. If an initial address is found, but no second, the upper bound
  // is equal to the lower.
  set_cmd_addr_type(&ea, p);

  if (parse_cmd_address(&ea, &errormsg, false) == FAIL) {
    goto doend;
  }

  // 5. Parse the command.

  // Skip ':' and any white space
  ea.cmd = skip_colon_white(ea.cmd, true);

  // If we got a line, but no command, then go to the line.
  // If we find a '|' or '\n' we set ea.nextcmd.
  if (*ea.cmd == NUL || *ea.cmd == '"'
      || (ea.nextcmd = check_nextcmd(ea.cmd)) != NULL) {
    // strange vi behaviour:
    // ":3"     jumps to line 3
    // ":3|..." prints line 3
    // ":|"     prints current line
    if (ea.skip) {  // skip this if inside :if
      goto doend;
    }
    assert(errormsg == NULL);
    errormsg = ex_range_without_command(&ea);
    goto doend;
  }

  // If this looks like an undefined user command and there are CmdUndefined
  // autocommands defined, trigger the matching autocommands.
  if (p != NULL && ea.cmdidx == CMD_SIZE && !ea.skip
      && ASCII_ISUPPER(*ea.cmd)
      && has_event(EVENT_CMDUNDEFINED)) {
    char *cmdname = ea.cmd;
    while (ASCII_ISALNUM(*cmdname)) {
      cmdname++;
    }
    cmdname = xmemdupz(ea.cmd, (size_t)(cmdname - ea.cmd));
    int ret = apply_autocmds(EVENT_CMDUNDEFINED, cmdname, cmdname, true, NULL);
    xfree(cmdname);
    // If the autocommands did something and didn't cause an error, try
    // finding the command again.
    p = (ret && !aborting()) ? find_ex_command(&ea, NULL) : ea.cmd;
  }

  if (p == NULL) {
    if (!ea.skip) {
      errormsg = _(e_ambiguous_use_of_user_defined_command);
    }
    goto doend;
  }

  // Check for wrong commands.
  if (ea.cmdidx == CMD_SIZE) {
    if (!ea.skip) {
      xstrlcpy(IObuff, _(e_not_an_editor_command), IOSIZE);
      // If the modifier was parsed OK the error must be in the following
      // command
      char *cmdname = after_modifier ? after_modifier : *cmdlinep;
      if (!(flags & DOCMD_VERBOSE)) {
        append_command(cmdname);
      }
      errormsg = IObuff;
      did_emsg_syntax = true;
      verify_command(cmdname);
    }
    goto doend;
  }

  // set when Not Implemented
  const int ni = is_cmd_ni(ea.cmdidx);

  // Determine if command has forceit flag ('!')
  ea.forceit = parse_bang(&ea, &p);

  // 6. Parse arguments.  Then check for errors.
  if (!IS_USER_CMDIDX(ea.cmdidx)) {
    ea.argt = cmdnames[(int)ea.cmdidx].cmd_argt;
  }

  if (!ea.skip) {
    if (sandbox != 0 && !(ea.argt & EX_SBOXOK)) {
      // Command not allowed in sandbox.
      errormsg = _(e_sandbox);
      goto doend;
    }
    if (!MODIFIABLE(curbuf) && (ea.argt & EX_MODIFY)
        // allow :put in terminals
        && !(curbuf->terminal && (ea.cmdidx == CMD_put || ea.cmdidx == CMD_iput))) {
      // Command not allowed in non-'modifiable' buffer
      errormsg = _(e_modifiable);
      goto doend;
    }

    if (!IS_USER_CMDIDX(ea.cmdidx)) {
      if (cmdwin_type != 0 && !(ea.argt & EX_CMDWIN)) {
        // Command not allowed in the command line window
        errormsg = _(e_cmdwin);
        goto doend;
      }
      if (text_locked() && !(ea.argt & EX_LOCK_OK)) {
        // Command not allowed when text is locked
        errormsg = _(get_text_locked_msg());
        goto doend;
      }
    }

    // Disallow editing another buffer when "curbuf->b_ro_locked" is set.
    // Do allow ":checktime" (it is postponed).
    // Do allow ":edit" (check for an argument later).
    // Do allow ":file" with no arguments (check for an argument later).
    if (!(ea.argt & EX_CMDWIN)
        && ea.cmdidx != CMD_checktime
        && ea.cmdidx != CMD_edit
        && ea.cmdidx != CMD_file
        && !IS_USER_CMDIDX(ea.cmdidx)
        && curbuf_locked()) {
      goto doend;
    }

    if (!ni && !(ea.argt & EX_RANGE) && ea.addr_count > 0) {
      // no range allowed
      errormsg = _(e_norange);
      goto doend;
    }
  }

  if (!ni && !(ea.argt & EX_BANG) && ea.forceit) {  // no <!> allowed
    errormsg = _(e_nobang);
    goto doend;
  }

  // Don't complain about the range if it is not used
  // (could happen if line_count is accidentally set to 0).
  if (!ea.skip && !ni && (ea.argt & EX_RANGE)) {
    // If the range is backwards, ask for confirmation and, if given, swap
    // ea.line1 & ea.line2 so it's forwards again.
    // When global command is busy, don't ask, will fail below.
    if (!global_busy && ea.line1 > ea.line2) {
      if (msg_silent == 0) {
        if ((flags & DOCMD_VERBOSE) || exmode_active) {
          errormsg = _("E493: Backwards range given");
          goto doend;
        }
        if (ask_yesno(_("Backwards range given, OK to swap")) != 'y') {
          goto doend;
        }
      }
      linenr_T lnum = ea.line1;
      ea.line1 = ea.line2;
      ea.line2 = lnum;
    }
    if ((errormsg = invalid_range(&ea)) != NULL) {
      goto doend;
    }
  }

  if ((ea.addr_type == ADDR_OTHER) && ea.addr_count == 0) {
    // default is 1, not cursor
    ea.line2 = 1;
  }

  correct_range(&ea);

  if (((ea.argt & EX_WHOLEFOLD) || ea.addr_count >= 2) && !global_busy
      && ea.addr_type == ADDR_LINES) {
    // Put the first line at the start of a closed fold, put the last line
    // at the end of a closed fold.
    hasFolding(curwin, ea.line1, &ea.line1, NULL);
    hasFolding(curwin, ea.line2, NULL, &ea.line2);
  }

  // For the ":make" and ":grep" commands we insert the 'makeprg'/'grepprg'
  // option here, so things like % get expanded.
  p = nvim_docmd_replace_makeprg_impl(&ea, p, cmdlinep);
  if (p == NULL) {
    goto doend;
  }

  // Skip to start of argument.
  // Don't do this for the ":!" command, because ":!! -l" needs the space.
  ea.arg = ea.cmdidx == CMD_bang ? p : skipwhite(p);

  // ":file" cannot be run with an argument when "curbuf->b_ro_locked" is set
  if (ea.cmdidx == CMD_file && *ea.arg != NUL && curbuf_locked()) {
    goto doend;
  }

  // Check for "++opt=val" argument.
  // Must be first, allow ":w ++enc=utf8 !cmd"
  if (ea.argt & EX_ARGOPT) {
    while (ea.arg[0] == '+' && ea.arg[1] == '+') {
      if (getargopt(&ea) == FAIL && !ni) {
        errormsg = _(e_invarg);
        goto doend;
      }
    }
  }

  if (ea.cmdidx == CMD_write || ea.cmdidx == CMD_update) {
    if (*ea.arg == '>') {                       // append
      if (*++ea.arg != '>') {                   // typed wrong
        errormsg = _("E494: Use w or w>>");
        goto doend;
      }
      ea.arg = skipwhite(ea.arg + 1);
      ea.append = true;
    } else if (*ea.arg == '!' && ea.cmdidx == CMD_write) {  // :w !filter
      ea.arg++;
      ea.usefilter = true;
    }
  } else if (ea.cmdidx == CMD_read) {
    if (ea.forceit) {
      ea.usefilter = true;                      // :r! filter if ea.forceit
      ea.forceit = false;
    } else if (*ea.arg == '!') {              // :r !filter
      ea.arg++;
      ea.usefilter = true;
    }
  } else if (ea.cmdidx == CMD_lshift || ea.cmdidx == CMD_rshift) {
    ea.amount = 1;
    while (*ea.arg == *ea.cmd) {                // count number of '>' or '<'
      ea.arg++;
      ea.amount++;
    }
    ea.arg = skipwhite(ea.arg);
  }

  // Check for "+command" argument, before checking for next command.
  // Don't do this for ":read !cmd" and ":write !cmd".
  if ((ea.argt & EX_CMDARG) && !ea.usefilter) {
    ea.do_ecmd_cmd = getargcmd(&ea.arg);
  }

  // Check for '|' to separate commands and '"' to start comments.
  // Don't do this for ":read !cmd" and ":write !cmd".
  if ((ea.argt & EX_TRLBAR) && !ea.usefilter) {
    separate_nextcmd(&ea);
  } else if (ea.cmdidx == CMD_bang
             || ea.cmdidx == CMD_terminal
             || ea.cmdidx == CMD_global
             || ea.cmdidx == CMD_vglobal
             || ea.usefilter) {
    // Check for <newline> to end a shell command.
    // Also do this for ":read !cmd", ":write !cmd" and ":global".
    // Any others?
    for (char *s = ea.arg; *s; s++) {
      // Remove one backslash before a newline, so that it's possible to
      // pass a newline to the shell and also a newline that is preceded
      // with a backslash.  This makes it impossible to end a shell
      // command in a backslash, but that doesn't appear useful.
      // Halving the number of backslashes is incompatible with previous
      // versions.
      if (*s == '\\' && s[1] == '\n') {
        STRMOVE(s, s + 1);
      } else if (*s == '\n') {
        ea.nextcmd = s + 1;
        *s = NUL;
        break;
      }
    }
  }

  if ((ea.argt & EX_DFLALL) && ea.addr_count == 0) {
    set_cmd_dflall_range(&ea);
  }

  // Parse register and count
  parse_register(&ea);
  if (parse_count(&ea, &errormsg, true) == FAIL) {
    goto doend;
  }

  // Check for flags: 'l', 'p' and '#'.
  if (ea.argt & EX_FLAGS) {
    get_flags(&ea);
  }
  if (!ni && !(ea.argt & EX_EXTRA) && *ea.arg != NUL
      && *ea.arg != '"' && (*ea.arg != '|' || (ea.argt & EX_TRLBAR) == 0)) {
    // no arguments allowed but there is something
    errormsg = ex_errmsg(e_trailing_arg, ea.arg);
    goto doend;
  }

  if (!ni && (ea.argt & EX_NEEDARG) && *ea.arg == NUL) {
    errormsg = _(e_argreq);
    goto doend;
  }

  if (skip_cmd(&ea)) {
    goto doend;
  }

  // 7. Execute the command.
  int retv = 0;
  if (execute_cmd0(&retv, &ea, &errormsg, false) == FAIL) {
    goto doend;
  }

  // If the command just executed called do_cmdline(), any throw or ":return"
  // or ":finish" encountered there must also check the cstack of the still
  // active do_cmdline() that called this do_one_cmd().  Rethrow an uncaught
  // exception, or reanimate a returned function or finished script file and
  // return or finish it again.
  if (need_rethrow) {
    do_throw(cstack);
  } else if (check_cstack) {
    if (source_finished(fgetline, cookie)) {
      do_finish(&ea, true);
    } else if (getline_equal(fgetline, cookie, get_func_line)
               && current_func_returned()) {
      do_return(&ea, true, false, NULL);
    }
  }
  need_rethrow = check_cstack = false;

doend:
  // can happen with zero line number
  if (curwin->w_cursor.lnum == 0) {
    curwin->w_cursor.lnum = 1;
    curwin->w_cursor.col = 0;
  }

  if (errormsg != NULL && *errormsg != NUL && !did_emsg) {
    if (flags & DOCMD_VERBOSE) {
      if (errormsg != IObuff) {
        xstrlcpy(IObuff, errormsg, IOSIZE);
        errormsg = IObuff;
      }
      append_command(*ea.cmdlinep);
    }
    emsg(errormsg);
  }
  do_errthrow(cstack,
              (ea.cmdidx != CMD_SIZE
               && !IS_USER_CMDIDX(ea.cmdidx)) ? cmdnames[(int)ea.cmdidx].cmd_name : NULL);

  nvim_docmd_undo_cmdmod_impl(&cmdmod);
  cmdmod = save_cmdmod;
  reg_executing = save_reg_executing;
  pending_end_reg_executing = save_pending_end_reg_executing;

  if (ea.nextcmd && *ea.nextcmd == NUL) {       // not really a next command
    ea.nextcmd = NULL;
  }

  ex_nesting_level--;
  xfree(ea.cmdline_tofree);

  return ea.nextcmd;
}

static char ex_error_buf[MSG_BUF_LEN];

/// @return an error message with argument included.
/// Uses a static buffer, only the last error will be kept.
/// "msg" will be translated, caller should use N_().
char *ex_errmsg(const char *const msg, const char *const arg)
  FUNC_ATTR_NONNULL_ALL
{
  vim_snprintf(ex_error_buf, MSG_BUF_LEN, _(msg), arg);
  return ex_error_buf;
}

/// The "+" string used in place of an empty command in Ex mode.
/// This string is used in pointer comparison.
static char exmode_plus[] = "+";

/// Parse and skip over command modifiers:
/// - update eap->cmd
/// - store flags in "cmod".
/// - Set ex_pressedreturn for an empty command line.
///
/// @param skip_only      if false, undo_cmdmod() must be called later to free
///                       any cmod_filter_pat and cmod_filter_regmatch.regprog,
///                       and ex_pressedreturn may be set.
/// @param[out] errormsg  potential error message.
///
/// Call apply_cmdmod() to get the side effects of the modifiers:
/// - Increment "sandbox" for ":sandbox"
/// - set p_verbose for ":verbose"
/// - set msg_silent for ":silent"
/// - set 'eventignore' to "all" for ":noautocmd"
///
/// Apply the command modifiers.  Saves current state in "cmdmod", call
/// undo_cmdmod() later.
void nvim_docmd_apply_cmdmod_impl(cmdmod_T *cmod)
{
  if ((cmod->cmod_flags & CMOD_SANDBOX) && !cmod->cmod_did_sandbox) {
    sandbox++;
    cmod->cmod_did_sandbox = true;
  }
  if (cmod->cmod_verbose > 0) {
    if (cmod->cmod_verbose_save == 0) {
      cmod->cmod_verbose_save = p_verbose + 1;
    }
    p_verbose = cmod->cmod_verbose - 1;
  }

  if ((cmod->cmod_flags & (CMOD_SILENT | CMOD_UNSILENT))
      && cmod->cmod_save_msg_silent == 0) {
    cmod->cmod_save_msg_silent = msg_silent + 1;
    cmod->cmod_save_msg_scroll = msg_scroll;
  }
  if (cmod->cmod_flags & CMOD_SILENT) {
    msg_silent++;
  }
  if (cmod->cmod_flags & CMOD_UNSILENT) {
    msg_silent = 0;
  }

  if (cmod->cmod_flags & CMOD_ERRSILENT) {
    emsg_silent++;
    cmod->cmod_did_esilent++;
  }

  if ((cmod->cmod_flags & CMOD_NOAUTOCMD) && cmod->cmod_save_ei == NULL) {
    // Set 'eventignore' to "all".
    // First save the existing option value for restoring it later.
    cmod->cmod_save_ei = xstrdup(p_ei);
    set_option_direct(kOptEventignore, STATIC_CSTR_AS_OPTVAL("all"), 0, SID_NONE);
  }
}

/// Undo and free contents of "cmod".
void nvim_docmd_undo_cmdmod_impl(cmdmod_T *cmod)
  FUNC_ATTR_NONNULL_ALL
{
  if (cmod->cmod_verbose_save > 0) {
    p_verbose = cmod->cmod_verbose_save - 1;
    cmod->cmod_verbose_save = 0;
  }

  if (cmod->cmod_did_sandbox) {
    sandbox--;
    cmod->cmod_did_sandbox = false;
  }

  if (cmod->cmod_save_ei != NULL) {
    // Restore 'eventignore' to the value before ":noautocmd".
    set_option_direct(kOptEventignore, CSTR_AS_OPTVAL(cmod->cmod_save_ei), 0, SID_NONE);
    free_string_option(cmod->cmod_save_ei);
    cmod->cmod_save_ei = NULL;
  }

  xfree(cmod->cmod_filter_pat);
  vim_regfree(cmod->cmod_filter_regmatch.regprog);

  if (cmod->cmod_save_msg_silent > 0) {
    // messages could be enabled for a serious error, need to check if the
    // counters don't become negative
    if (!did_emsg || msg_silent > cmod->cmod_save_msg_silent - 1) {
      msg_silent = cmod->cmod_save_msg_silent - 1;
    }
    emsg_silent -= cmod->cmod_did_esilent;
    emsg_silent = MAX(emsg_silent, 0);
    // Restore msg_scroll, it's set by file I/O commands, even when no
    // message is actually displayed.
    msg_scroll = cmod->cmod_save_msg_scroll;

    // "silent reg" or "silent echo x" inside "redir" leaves msg_col
    // somewhere in the line.  Put it back in the first column.
    if (redirecting()) {
      msg_col = 0;
    }

    cmod->cmod_save_msg_silent = 0;
    cmod->cmod_did_esilent = 0;
  }
}

/// Parse the address range, if any, in "eap".
/// May set the last search pattern, unless "silent" is true.
///
uint32_t excmd_get_argt(cmdidx_T idx)
{
  return cmdnames[(int)idx].cmd_argt;
}


/// Correct the range for zero line number, if required.
// nvim_docmd_replace_makeprg_impl is implemented in Rust (args.rs).


/// Function given to ExpandGeneric() to obtain the list of bad= names.
static char *get_bad_name(expand_T *xp FUNC_ATTR_UNUSED, int idx)
{
  // Note: Keep this in sync with get_bad_opt().
  static char *(p_bad_values[]) = {
    "?",
    "keep",
    "drop",
  };

  if (idx < (int)ARRAY_SIZE(p_bad_values)) {
    return p_bad_values[idx];
  }
  return NULL;
}


/// Command-line expansion for ++opt=name.
int nvim_docmd_expand_argopt_impl(char *pat, expand_T *xp, regmatch_T *rmp, char ***matches, int *numMatches)
{
  if (xp->xp_pattern > xp->xp_line && *(xp->xp_pattern - 1) == '=') {
    CompleteListItemGetter cb = NULL;

    char *name_end = xp->xp_pattern - 1;
    if (name_end - xp->xp_line >= 2
        && strncmp(name_end - 2, "ff", 2) == 0) {
      cb = get_fileformat_name;
    } else if (name_end - xp->xp_line >= 10
               && strncmp(name_end - 10, "fileformat", 10) == 0) {
      cb = get_fileformat_name;
    } else if (name_end - xp->xp_line >= 3
               && strncmp(name_end - 3, "enc", 3) == 0) {
      cb = get_encoding_name;
    } else if (name_end - xp->xp_line >= 8
               && strncmp(name_end - 8, "encoding", 8) == 0) {
      cb = get_encoding_name;
    } else if (name_end - xp->xp_line >= 3
               && strncmp(name_end - 3, "bad", 3) == 0) {
      cb = get_bad_name;
    }

    if (cb != NULL) {
      ExpandGeneric(pat, xp, rmp, matches, numMatches, cb, false);
      return OK;
    }
    return FAIL;
  }

  // Special handling of "ff" which acts as a short form of
  // "fileformat", as "ff" is not a substring of it.
  if (xp->xp_pattern_len == 2
      && strncmp(xp->xp_pattern, "ff", xp->xp_pattern_len) == 0) {
    *matches = xmalloc(sizeof(char *));
    *numMatches = 1;
    (*matches)[0] = xstrdup("fileformat=");
    return OK;
  }

  ExpandGeneric(pat, xp, rmp, matches, numMatches, get_argopt_name, false);
  return OK;
}

/// ":buffer" command and alike.
/// do_exbuffer implementation. Called by Rust do_exbuffer.
void nvim_docmd_do_exbuffer_impl(exarg_T *eap)
{
  if (*eap->arg) {
    eap->errmsg = ex_errmsg(e_trailing_arg, eap->arg);
  } else {
    if (eap->addr_count == 0) {  // default is current buffer
      goto_buffer(eap, DOBUF_CURRENT, FORWARD, 0);
    } else {
      goto_buffer(eap, DOBUF_FIRST, FORWARD, (int)eap->line2);
    }
    if (eap->do_ecmd_cmd != NULL) {
      do_cmdline_cmd(eap->do_ecmd_cmd);
    }
  }
}



/// Call this function if we thought we were going to exit, but we won't
/// (because of an error).  May need to restore the terminal mode.

/// before_quit_autocmds implementation. Called by Rust before_quit_autocmds.
bool nvim_docmd_before_quit_autocmds_impl(win_T *wp, bool quit_all, bool forceit)
{
  apply_autocmds(EVENT_QUITPRE, NULL, NULL, false, wp->w_buffer);

  // Bail out when autocommands closed the window.
  // Refuse to quit when the buffer in the last window is being closed (can
  // only happen in autocommands).
  if (!rs_win_valid(wp)
      || curbuf_locked()
      || (wp->w_buffer->b_nwindows == 1 && wp->w_buffer->b_locked > 0)) {
    return true;
  }

  if (quit_all
      || (check_more(false, forceit) == OK && rs_only_one_window())) {
    apply_autocmds(EVENT_EXITPRE, NULL, NULL, false, curbuf);
    // Refuse to quit when locked or when the window was closed or the
    // buffer in the last window is being closed (can only happen in
    // autocommands).
    if (!rs_win_valid(wp)
        || curbuf_locked()
        || (curbuf->b_nwindows == 1 && curbuf->b_locked > 0)) {
      return true;
    }
  }

  return false;
}

/// ":restart": restart the Nvim server (using ":qall!").
/// ":restart +cmd": restart the Nvim server using ":cmd".
/// ":restart +cmd <command>": restart the Nvim server using ":cmd" and add -c <command> to the new server.
/// Implementation called by Rust ex_restart.
void nvim_docmd_ex_restart_impl(exarg_T *eap)
{
  // Patch v:argv to include "-c <arg>" when it restarts.
  if (eap->arg != NULL) {
    const list_T *l = get_vim_var_list(VV_ARGV);
    int argc = tv_list_len(l);
    list_T *argv_cpy = tv_list_alloc(argc + 2);
    bool added_startup_arg = false;
    TV_LIST_ITER_CONST(l, li, {
      const char *arg = tv_get_string(TV_LIST_ITEM_TV(li));
      size_t arg_size = strlen(arg);
      assert(arg_size <= (size_t)SSIZE_MAX);
      tv_list_append_string(argv_cpy, arg, (ssize_t)arg_size);
      if (!added_startup_arg) {
        tv_list_append_string(argv_cpy, "-c", 2);
        size_t cmd_size = strlen(eap->arg);
        assert(cmd_size <= (size_t)SSIZE_MAX);
        tv_list_append_string(argv_cpy, eap->arg, (ssize_t)cmd_size);
        added_startup_arg = true;
      }
    });
    set_vim_var_list(VV_ARGV, argv_cpy);
  }

  char *quit_cmd = (eap->do_ecmd_cmd) ? eap->do_ecmd_cmd : "qall";
  char *quit_cmd_copy = NULL;

  // Prepend "confirm " to cmd if :confirm is used
  if (cmdmod.cmod_flags & CMOD_CONFIRM) {
    quit_cmd_copy = concat_str("confirm ", quit_cmd);
    quit_cmd = quit_cmd_copy;
  }

  Error err = ERROR_INIT;
  restarting = true;
  nvim_command(cstr_as_string(quit_cmd), &err);
  xfree(quit_cmd_copy);
  if (ERROR_SET(&err)) {
    emsg(err.msg);  // Could not exit
    api_clear_error(&err);
    not_restarting();
    return;
  }
  if (!exiting) {
    emsg("restart failed: +cmd did not quit the server");
    not_restarting();
  }
}


/// Close window "win" and take care of handling closing the last window for a
/// modified buffer.
///
/// @param tp  NULL or the tab page "win" is in
/// ex_win_close implementation. Called by Rust ex_win_close.
void nvim_docmd_ex_win_close_impl(int forceit, win_T *win, tabpage_T *tp)
{
  // Never close the autocommand window.
  if (is_aucmd_win(win)) {
    emsg(_(e_autocmd_close));
    return;
  }

  buf_T *buf = win->w_buffer;

  bool need_hide = (bufIsChanged(buf) && buf->b_nwindows <= 1);
  if (need_hide && !buf_hide(buf) && !forceit) {
    if ((p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) && p_write) {
      bufref_T bufref;
      set_bufref(&bufref, buf);
      dialog_changed(buf, false);
      if (bufref_valid(&bufref) && bufIsChanged(buf)) {
        return;
      }
      need_hide = false;
    } else {
      no_write_message();
      return;
    }
  }

  // free buffer when not hiding it or when it's a scratch buffer
  if (tp == NULL) {
    win_close(win, !need_hide && !buf_hide(buf), forceit);
  } else {
    win_close_othertab(win, !need_hide && !buf_hide(buf), tp, forceit);
  }
}

/// ":tabonly" implementation called by Rust ex_tabonly.

/// Close the current tab page.
void nvim_docmd_tabpage_close_impl(int forceit)
{
  // First close all the windows but the current one.  If that worked then
  // close the last window in this tab, that will close it.
  while (curwin->w_floating) {
    ex_win_close(forceit, curwin, NULL);
  }
  if (!ONE_WINDOW) {
    close_others(true, forceit);
  }
  if (ONE_WINDOW) {
    ex_win_close(forceit, curwin, NULL);
  }
}

/// Close tab page "tp", which is not the current tab page.
/// Note that autocommands may make "tp" invalid.
/// Also takes care of the tab pages line disappearing when closing the
/// last-but-one tab page.
void nvim_docmd_tabpage_close_other_impl(tabpage_T *tp, int forceit)
{
  int done = 0;
  char prev_idx[NUMBUFLEN];

  // Limit to 1000 windows, autocommands may add a window while we close
  // one.  OK, so I'm paranoid...
  while (++done < 1000) {
    snprintf(prev_idx, sizeof(prev_idx), "%i", rs_tabpage_index(tp));
    win_T *wp = tp->tp_lastwin;
    ex_win_close(forceit, wp, tp);

    // Autocommands may delete the tab page under our fingers and we may
    // fail to close a window with a modified buffer.
    if (!rs_valid_tabpage(tp) || tp->tp_lastwin == wp) {
      break;
    }
  }
}

/// callback function for 'findfunc'
static Callback ffu_cb;

static Callback *get_findfunc_callback(void)
{
  return *curbuf->b_p_ffu != NUL ? &curbuf->b_ffu_cb : &ffu_cb;
}

/// Call 'findfunc' to obtain a list of file names.
static list_T *call_findfunc(char *pat, BoolVarValue cmdcomplete)
{
  const sctx_T saved_sctx = current_sctx;

  typval_T args[3];
  args[0].v_type = VAR_STRING;
  args[0].vval.v_string = pat;
  args[1].v_type = VAR_BOOL;
  args[1].vval.v_bool = cmdcomplete;
  args[2].v_type = VAR_UNKNOWN;

  // Lock the text to prevent weird things from happening.  Also disallow
  // switching to another window, it should not be needed and may end up in
  // Insert mode in another buffer.
  textlock++;

  sctx_T *ctx = get_option_sctx(kOptFindfunc);
  if (ctx != NULL) {
    current_sctx = *ctx;
  }

  Callback *cb = get_findfunc_callback();
  typval_T rettv;
  int retval = callback_call(cb, 2, args, &rettv);

  current_sctx = saved_sctx;

  textlock--;

  list_T *retlist = NULL;

  if (retval == OK) {
    if (rettv.v_type == VAR_LIST) {
      retlist = tv_list_copy(NULL, rettv.vval.v_list, false, rs_get_copyID());
    } else {
      emsg(_(e_invalid_return_type_from_findfunc));
    }

    tv_clear(&rettv);
  }

  return retlist;
}

/// Find file names matching "pat" using 'findfunc' and return it in "files".
/// Used for expanding the :find, :sfind and :tabfind command argument.
/// Returns OK on success and FAIL otherwise.
int expand_findfunc(char *pat, char ***files, int *numMatches)
{
  *numMatches = 0;
  *files = NULL;

  list_T *l = call_findfunc(pat, kBoolVarTrue);
  if (l == NULL) {
    return FAIL;
  }

  int len = tv_list_len(l);
  if (len == 0) {  // empty List
    return FAIL;
  }

  *files = xmalloc(sizeof(char *) * (size_t)len);

  // Copy all the List items
  int idx = 0;
  TV_LIST_ITER_CONST(l, li, {
    if (TV_LIST_ITEM_TV(li)->v_type == VAR_STRING) {
      (*files)[idx] = xstrdup(TV_LIST_ITEM_TV(li)->vval.v_string);
      idx++;
    }
  });

  *numMatches = idx;
  tv_list_free(l);

  return OK;
}

/// Use 'findfunc' to find file 'findarg'.  The 'count' argument is used to find
/// the n'th matching file.
static char *findfunc_find_file(char *findarg, size_t findarg_len, int count)
{
  char *ret_fname = NULL;

  const char cc = findarg[findarg_len];
  findarg[findarg_len] = NUL;

  list_T *fname_list = call_findfunc(findarg, kBoolVarFalse);
  int fname_count = tv_list_len(fname_list);

  if (fname_count == 0) {
    semsg(_(e_cant_find_file_str_in_path), findarg);
  } else {
    if (count > fname_count) {
      semsg(_(e_no_more_file_str_found_in_path), findarg);
    } else {
      listitem_T *li = tv_list_find(fname_list, count - 1);
      if (li != NULL && TV_LIST_ITEM_TV(li)->v_type == VAR_STRING) {
        ret_fname = xstrdup(TV_LIST_ITEM_TV(li)->vval.v_string);
      }
    }
  }

  if (fname_list != NULL) {
    tv_list_free(fname_list);
  }

  findarg[findarg_len] = cc;

  return ret_fname;
}

/// Process the 'findfunc' option value.
/// Returns NULL on success and an error message on failure.
const char *nvim_docmd_did_set_findfunc_impl(optset_T *args)
{
  buf_T *buf = (buf_T *)args->os_buf;
  int retval;

  if (args->os_flags & OPT_LOCAL) {
    // buffer-local option set
    retval = option_set_callback_func(buf->b_p_ffu, &buf->b_ffu_cb);
  } else {
    // global option set
    retval = option_set_callback_func(p_ffu, &ffu_cb);
    // when using :set, free the local callback
    if (!(args->os_flags & OPT_GLOBAL)) {
      callback_free(&buf->b_ffu_cb);
    }
  }

  if (retval == FAIL) {
    return e_invarg;
  }

  // If the option value starts with <SID> or s:, then replace that with
  // the script identifier.
  char **varp = (char **)args->os_varp;
  char *name = get_scriptlocal_funcname(*varp);
  if (name != NULL) {
    free_string_option(*varp);
    *varp = name;
  }

  return NULL;
}

void nvim_docmd_free_findfunc_option_impl(void)
{
  callback_free(&ffu_cb);
}

/// Mark the global 'findfunc' callback with "copyID" so that it is not
/// garbage collected.
bool nvim_docmd_set_ref_in_findfunc_impl(int copyID)
{
  bool abort = false;
  abort = rs_set_ref_in_callback(&ffu_cb, copyID, NULL, NULL);
  return abort;
}

/// :sview [+command] file       split window with new file, read-only
/// :split [[+command] file]     split window with current or new file
/// :vsplit [[+command] file]    split window vertically with current or new file
/// :new [[+command] file]       split window with no or new file
/// :vnew [[+command] file]      split vertically window with no or new file
/// :sfind [+command] file       split window with file in 'path'
///
/// :tabedit                     open new Tab page with empty window
/// :tabedit [+command] file     open new Tab page and edit "file"
/// :tabnew [[+command] file]    just like :tabedit
/// :tabfind [+command] file     open new Tab page and find "file"
void nvim_docmd_ex_splitview_impl(exarg_T *eap)
{
  win_T *old_curwin = curwin;
  char *fname = NULL;
  const bool use_tab = eap->cmdidx == CMD_tabedit
                       || eap->cmdidx == CMD_tabfind
                       || eap->cmdidx == CMD_tabnew;

  // A ":split" in the quickfix window works like ":new".  Don't want two
  // quickfix windows.  But it's OK when doing ":tab split".
  if (bt_quickfix(curbuf) && cmdmod.cmod_tab == 0) {
    if (eap->cmdidx == CMD_split) {
      eap->cmdidx = CMD_new;
    }
    if (eap->cmdidx == CMD_vsplit) {
      eap->cmdidx = CMD_vnew;
    }
  }

  if (eap->cmdidx == CMD_sfind || eap->cmdidx == CMD_tabfind) {
    if (*get_findfunc() != NUL) {
      fname = findfunc_find_file(eap->arg, strlen(eap->arg),
                                 eap->addr_count > 0 ? eap->line2 : 1);
    } else {
      char *file_to_find = NULL;
      char *search_ctx = NULL;
      fname = find_file_in_path(eap->arg, strlen(eap->arg), FNAME_MESS, true,
                                curbuf->b_ffname, &file_to_find, &search_ctx);
      xfree(file_to_find);
      vim_findfile_cleanup(search_ctx);
    }
    if (fname == NULL) {
      goto theend;
    }
    eap->arg = fname;
  }

  // Either open new tab page or split the window.
  if (use_tab) {
    if (win_new_tabpage(cmdmod.cmod_tab != 0 ? cmdmod.cmod_tab : eap->addr_count == 0
                        ? 0 : (int)eap->line2 + 1, eap->arg) != FAIL) {
      nvim_docmd_do_exedit_impl(eap, old_curwin);
      apply_autocmds(EVENT_TABNEWENTERED, NULL, NULL, false, curbuf);

      // set the alternate buffer for the window we came from
      if (curwin != old_curwin
          && rs_win_valid(old_curwin)
          && old_curwin->w_buffer != curbuf
          && (cmdmod.cmod_flags & CMOD_KEEPALT) == 0) {
        old_curwin->w_alt_fnum = curbuf->b_fnum;
      }
    }
  } else if (win_split(eap->addr_count > 0 ? (int)eap->line2 : 0,
                       *eap->cmd == 'v' ? WSP_VERT : 0) != FAIL) {
    // Reset 'scrollbind' when editing another file, but keep it when
    // doing ":split" without arguments.
    if (*eap->arg != NUL) {
      RESET_BINDING(curwin);
    } else {
      do_check_scrollbind(false);
    }
    nvim_docmd_do_exedit_impl(eap, old_curwin);
  }

theend:
  xfree(fname);
}

/// Open a new tab page.
void nvim_docmd_tabpage_new_impl(void)
{
  exarg_T ea = {
    .cmdidx = CMD_tabnew,
    .cmd = "tabn",
    .arg = "",
  };
  nvim_docmd_ex_splitview_impl(&ea);
}


/// :tabs command: List tabs and their contents. Called by Rust ex_tabs.
void nvim_docmd_ex_tabs_impl(exarg_T *eap)
{
  int tabcount = 1;

  msg_start();
  msg_scroll = true;

  win_T *lastused_win = rs_valid_tabpage(lastused_tabpage)
                        ? lastused_tabpage->tp_curwin
                        : NULL;

  FOR_ALL_TABS(tp) {
    if (got_int) {
      break;
    }

    msg_putchar('\n');
    vim_snprintf(IObuff, IOSIZE, _("Tab page %d"), tabcount++);
    msg_outtrans(IObuff, HLF_T, false);
    os_breakcheck();

    FOR_ALL_WINDOWS_IN_TAB(wp, tp) {
      if (got_int) {
        break;
      } else if (!wp->w_config.focusable || wp->w_config.hide) {
        continue;
      }

      msg_putchar('\n');
      msg_putchar(wp == curwin ? '>' : wp == lastused_win ? '#' : ' ');
      msg_putchar(' ');
      msg_putchar(bufIsChanged(wp->w_buffer) ? '+' : ' ');
      msg_putchar(' ');
      if (buf_spname(wp->w_buffer) != NULL) {
        xstrlcpy(IObuff, buf_spname(wp->w_buffer), IOSIZE);
      } else {
        home_replace(wp->w_buffer, wp->w_buffer->b_fname, IObuff, IOSIZE, true);
      }
      msg_outtrans(IObuff, 0, false);
      os_breakcheck();
    }
  }
}

/// ":detach" - called by Rust ex_detach.
void nvim_docmd_ex_detach_impl(exarg_T *eap)
{
  // come on pooky let's burn this mf down
  if (eap && eap->forceit) {
    emsg("bang (!) not supported yet");
  } else {
    // 1. Send "error_exit" UI-event (notification only).
    // 2. Perform server-side UI detach.
    // 3. Close server-side channel without self-exit.

    if (!current_ui) {
      emsg("UI not attached");
      return;
    }

    Channel *chan = find_channel(current_ui);
    if (!chan) {
      emsg(e_invchan);
      return;
    }
    chan->detach = true;  // Prevent self-exit on channel-close.

    // Server-side UI detach. Doesn't close the channel.
    Error err2 = ERROR_INIT;
    remote_ui_disconnect(chan->id, &err2, true);
    if (ERROR_SET(&err2)) {
      emsg(err2.msg);  // UI disappeared already?
      api_clear_error(&err2);
      return;
    }

    // Server-side channel close.
    const char *err = NULL;
    bool rv = channel_close(chan->id, kChannelPartAll, &err);
    if (!rv && err) {
      emsg(err);  // UI disappeared already?
      return;
    }
    // XXX: Can't do this, channel_decref() is async...
    // assert(!find_channel(chan->id));

    ILOG("detach current_ui=%" PRId64, chan->id);
  }
}

/// ":connect" - called by Rust ex_connect.
void nvim_docmd_ex_connect_impl(exarg_T *eap)
{
  bool stop_server = eap->forceit ? (ui_active() == 1) : false;

  Error err = ERROR_INIT;
  remote_ui_connect(current_ui, eap->arg, &err);

  if (ERROR_SET(&err)) {
    emsg(err.msg);
    api_clear_error(&err);
    return;
  }

  nvim_docmd_ex_detach_impl(NULL);
  if (stop_server) {
    exiting = true;
    getout(0);
  }
}

/// ":resize".
/// set, increment or decrement current window height
/// ":find [+command] <file>" command.
void nvim_docmd_ex_find_impl(exarg_T *eap)
{
  if (!check_can_set_curbuf_forceit(eap->forceit)) {
    return;
  }

  char *fname = NULL;
  if (*get_findfunc() != NUL) {
    fname = findfunc_find_file(eap->arg, strlen(eap->arg),
                               eap->addr_count > 0 ? eap->line2 : 1);
  } else {
    char *file_to_find = NULL;
    char *search_ctx = NULL;
    fname = find_file_in_path(eap->arg, strlen(eap->arg), FNAME_MESS, true,
                              curbuf->b_ffname, &file_to_find, &search_ctx);
    if (eap->addr_count > 0) {
      // Repeat finding the file "count" times.  This matters when it appears
      // several times in the path.
      linenr_T count = eap->line2;
      while (fname != NULL && --count > 0) {
        xfree(fname);
        fname = find_file_in_path(NULL, 0, FNAME_MESS, false,
                                  curbuf->b_ffname, &file_to_find, &search_ctx);
      }
    }
    xfree(file_to_find);
    vim_findfile_cleanup(search_ctx);
  }

  if (fname == NULL) {
    return;
  }

  eap->arg = fname;
  nvim_docmd_do_exedit_impl(eap, NULL);
  xfree(fname);
}

/// ":edit", ":badd", ":balt", ":visual".
/// ":edit <file>" command and alike.
///
/// @param old_curwin  curwin before doing a split or NULL
void nvim_docmd_do_exedit_impl(exarg_T *eap, win_T *old_curwin)
{
  // ":vi" command ends Ex mode.
  if (exmode_active && (eap->cmdidx == CMD_visual
                        || eap->cmdidx == CMD_view)) {
    exmode_active = false;
    ex_pressedreturn = false;
    if (*eap->arg == NUL) {
      // Special case:  ":global/pat/visual\NLvi-commands"
      if (global_busy) {
        if (eap->nextcmd != NULL) {
          stuffReadbuff(eap->nextcmd);
          eap->nextcmd = NULL;
        }

        const int save_rd = RedrawingDisabled;
        RedrawingDisabled = 0;
        const int save_nwr = no_wait_return;
        no_wait_return = 0;
        need_wait_return = false;
        const int save_ms = msg_scroll;
        msg_scroll = 0;
        redraw_all_later(UPD_NOT_VALID);
        pending_exmode_active = true;

        normal_enter(false, true);

        pending_exmode_active = false;
        RedrawingDisabled = save_rd;
        no_wait_return = save_nwr;
        msg_scroll = save_ms;
      }
      return;
    }
  }

  if ((eap->cmdidx == CMD_new
       || eap->cmdidx == CMD_tabnew
       || eap->cmdidx == CMD_tabedit
       || eap->cmdidx == CMD_vnew) && *eap->arg == NUL) {
    // ":new" or ":tabnew" without argument: edit a new empty buffer
    setpcmark();
    do_ecmd(0, NULL, NULL, eap, ECMD_ONE,
            ECMD_HIDE + (eap->forceit ? ECMD_FORCEIT : 0),
            old_curwin == NULL ? curwin : NULL);
  } else if ((eap->cmdidx != CMD_split && eap->cmdidx != CMD_vsplit)
             || *eap->arg != NUL) {
    // Can't edit another file when "textlock" or "curbuf->b_ro_locked" is set.
    // Only ":edit" or ":script" can bring us here, others are stopped earlier.
    if (*eap->arg != NUL && text_or_buf_locked()) {
      return;
    }
    int n = readonlymode;
    if (eap->cmdidx == CMD_view || eap->cmdidx == CMD_sview) {
      readonlymode = true;
    } else if (eap->cmdidx == CMD_enew) {
      readonlymode = false;  // 'readonly' doesn't make sense
                             // in an empty buffer
    }
    if (eap->cmdidx != CMD_balt && eap->cmdidx != CMD_badd) {
      setpcmark();
    }
    if (do_ecmd(0, eap->cmdidx == CMD_enew ? NULL : eap->arg,
                NULL, eap, eap->do_ecmd_lnum,
                (buf_hide(curbuf) ? ECMD_HIDE : 0)
                + (eap->forceit ? ECMD_FORCEIT : 0)
                // After a split we can use an existing buffer.
                + (old_curwin != NULL ? ECMD_OLDBUF : 0)
                + (eap->cmdidx == CMD_badd ? ECMD_ADDBUF : 0)
                + (eap->cmdidx == CMD_balt ? ECMD_ALTBUF : 0),
                old_curwin == NULL ? curwin : NULL) == FAIL) {
      // Editing the file failed.  If the window was split, close it.
      if (old_curwin != NULL) {
        bool need_hide = (curbufIsChanged() && curbuf->b_nwindows <= 1);
        if (!need_hide || buf_hide(curbuf)) {
          cleanup_T cs;

          // Reset the error/interrupt/exception state here so that
          // aborting() returns false when closing a window.
          enter_cleanup(&cs);
          win_close(curwin, !need_hide && !buf_hide(curbuf), false);

          // Restore the error/interrupt/exception state if not
          // discarded by a new aborting error, interrupt, or
          // uncaught exception.
          leave_cleanup(&cs);
        }
      }
    } else if (readonlymode && curbuf->b_nwindows == 1) {
      // When editing an already visited buffer, 'readonly' won't be set
      // but the previous value is kept.  With ":view" and ":sview" we
      // want the  file to be readonly, except when another window is
      // editing the same buffer.
      curbuf->b_p_ro = true;
    }
    readonlymode = n;
  } else {
    if (eap->do_ecmd_cmd != NULL) {
      do_cmdline_cmd(eap->do_ecmd_cmd);
    }
    int n = curwin->w_arg_idx_invalid;
    check_arg_idx(curwin);
    if (n != curwin->w_arg_idx_invalid) {
      maketitle();
    }
  }

  // if ":split file" worked, set alternate file name in old window to new
  // file
  if (old_curwin != NULL
      && *eap->arg != NUL
      && curwin != old_curwin
      && rs_win_valid(old_curwin)
      && old_curwin->w_buffer != curbuf
      && (cmdmod.cmod_flags & CMOD_KEEPALT) == 0) {
    old_curwin->w_alt_fnum = curbuf->b_fnum;
  }

  ex_no_reprint = true;
}

/// ":syncbind" forces all 'scrollbind' windows to have the same relative
/// offset.
/// (1998-11-02 16:21:01  R. Edward Ralston <eralston@computer.org>)
/// ":syncbind" implementation called by Rust ex_syncbind.
void nvim_docmd_ex_syncbind_impl(exarg_T *eap)
{
  linenr_T vtopline;  // Target topline (including fill)

  linenr_T old_linenr = curwin->w_cursor.lnum;

  setpcmark();

  // determine max (virtual) topline
  if (curwin->w_p_scb) {
    vtopline = rs_get_vtopline(curwin);
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_p_scb && wp->w_buffer) {
        linenr_T y = plines_m_win_fill(wp, 1, wp->w_buffer->b_ml.ml_line_count)
                     - rs_get_scrolloff_value(curwin);
        vtopline = MIN(vtopline, y);
      }
    }
    vtopline = MAX(vtopline, 1);
  } else {
    vtopline = 1;
  }

  // Set all scrollbind windows to the same topline.
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_p_scb) {
      int y = vtopline - rs_get_vtopline(wp);
      if (y > 0) {
        scrollup(wp, y, true);
      } else {
        scrolldown(wp, -y, true);
      }
      wp->w_scbind_pos = vtopline;
      redraw_later(wp, UPD_VALID);
      cursor_correct(wp);
      wp->w_redr_status = true;
    }
  }

  if (curwin->w_p_scb) {
    did_syncbind = true;
    checkpcmark();
    if (old_linenr != curwin->w_cursor.lnum) {
      char ctrl_o[2];

      ctrl_o[0] = Ctrl_O;
      ctrl_o[1] = 0;
      ins_typebuf(ctrl_o, REMAP_NONE, 0, true, false);
    }
  }
}

/// ":read" implementation called by Rust ex_read.
void nvim_docmd_ex_read_impl(exarg_T *eap)
{
  int empty = (curbuf->b_ml.ml_flags & ML_EMPTY);

  if (eap->usefilter) {  // :r!cmd
    do_bang(1, eap, false, false, true);
    return;
  }

  if (u_save(eap->line2, (linenr_T)(eap->line2 + 1)) == FAIL) {
    return;
  }

  int i;
  if (*eap->arg == NUL) {
    if (check_fname() == FAIL) {       // check for no file name
      return;
    }
    i = readfile(curbuf->b_ffname, curbuf->b_fname,
                 eap->line2, 0, (linenr_T)MAXLNUM, eap, 0, false);
  } else {
    if (vim_strchr(p_cpo, CPO_ALTREAD) != NULL) {
      setaltfname(eap->arg, eap->arg, 1);
    }
    i = readfile(eap->arg, NULL,
                 eap->line2, 0, (linenr_T)MAXLNUM, eap, 0, false);
  }
  if (i != OK) {
    if (!aborting()) {
      semsg(_(e_notopen), eap->arg);
    }
  } else {
    if (empty && exmode_active) {
      // Delete the empty line that remains.  Historically ex does
      // this but vi doesn't.
      linenr_T lnum;
      if (eap->line2 == 0) {
        lnum = curbuf->b_ml.ml_line_count;
      } else {
        lnum = 1;
      }
      if (*ml_get(lnum) == NUL && u_savedel(lnum, 1) == OK) {
        ml_delete(lnum);
        if (curwin->w_cursor.lnum > 1
            && curwin->w_cursor.lnum >= lnum) {
          curwin->w_cursor.lnum--;
        }
        deleted_lines_mark(lnum, 1);
      }
    }
    redraw_curbuf_later(UPD_VALID);
  }
}

static char *prev_dir = NULL;

#if defined(EXITFREE)
void free_cd_dir(void)
{
  XFREE_CLEAR(prev_dir);
  XFREE_CLEAR(globaldir);
}

#endif

// nvim_docmd_post_chdir_impl is implemented in Rust (cmd_impl.rs).
extern void nvim_docmd_post_chdir_impl(CdScope scope, bool trigger_dirchanged);



// nvim_docmd_close_redir_impl is implemented in Rust (commands.rs).



/// Save the current State and go to Normal mode.
///
/// @return  true if the typeahead could be saved.
bool nvim_docmd_save_current_state_impl(save_state_T *sst)
  FUNC_ATTR_NONNULL_ALL
{
  sst->save_msg_scroll = msg_scroll;
  sst->save_restart_edit = restart_edit;
  sst->save_msg_didout = msg_didout;
  sst->save_State = State;
  sst->save_finish_op = finish_op;
  sst->save_opcount = opcount;
  sst->save_reg_executing = reg_executing;
  sst->save_pending_end_reg_executing = pending_end_reg_executing;

  msg_scroll = false;   // no msg scrolling in Normal mode
  restart_edit = 0;     // don't go to Insert mode

  // Save the current typeahead.  This is required to allow using ":normal"
  // from an event handler and makes sure we don't hang when the argument
  // ends with half a command.
  save_typeahead(&sst->tabuf);
  return sst->tabuf.typebuf_valid;
}

void nvim_docmd_restore_current_state_impl(save_state_T *sst)
  FUNC_ATTR_NONNULL_ALL
{
  // Restore the previous typeahead.
  restore_typeahead(&sst->tabuf);

  msg_scroll = sst->save_msg_scroll;
  if (force_restart_edit) {
    force_restart_edit = false;
  } else {
    // Some function (terminal_enter()) was aware of ex_normal and decided to
    // override the value of restart_edit anyway.
    restart_edit = sst->save_restart_edit;
  }
  finish_op = sst->save_finish_op;
  opcount = sst->save_opcount;
  reg_executing = sst->save_reg_executing;
  pending_end_reg_executing = sst->save_pending_end_reg_executing;

  // don't reset msg_didout now
  msg_didout |= sst->save_msg_didout;

  // Restore the state (needed when called from a function executed for
  // 'indentexpr'). Update the mouse and cursor, they may have changed.
  State = sst->save_State;
  ui_cursor_shape();  // may show different cursor shape
}


/// Execute normal mode command "cmd".
/// "remap" can be REMAP_NONE or REMAP_YES.
void nvim_docmd_exec_normal_cmd_impl(char *cmd, int remap, bool silent)
{
  // Stuff the argument into the typeahead buffer.
  ins_typebuf(cmd, remap, 0, true, silent);
  nvim_docmd_exec_normal_impl(false, false);
}

/// Execute normal_cmd() until there is no typeahead left.
///
/// @param was_typed whether or not something was typed
/// @param use_vpeekc  true to use vpeekc() to check for available chars
void nvim_docmd_exec_normal_impl(bool was_typed, bool use_vpeekc)
{
  oparg_T oa;
  int c;

  // When calling vpeekc() from feedkeys() it will return Ctrl_C when there
  // is nothing to get, so also check for Ctrl_C.
  clear_oparg(&oa);
  finish_op = false;
  while ((!stuff_empty()
          || ((was_typed || !typebuf_typed())
              && typebuf.tb_len > 0)
          || (use_vpeekc && (c = vpeekc()) != NUL && c != Ctrl_C))
         && !got_int) {
    nvim_docmd_update_topline_cursor_impl();
    normal_cmd(&oa, true);      // execute a Normal mode cmd
  }
}

/// Open the preview window or popup and make it the current window.
/// Called by Rust ex_pedit / ex_pbuffer.
void nvim_docmd_prepare_preview_window(void)
{
  g_do_tagpreview = (int)p_pvh;
  prepare_tagpreview(true);
}

/// Return cursor to previous window after preview operation.
/// Called by Rust ex_pedit / ex_pbuffer.
void nvim_docmd_back_to_current_window(win_T *curwin_save)
{
  if (curwin != curwin_save && rs_win_valid(curwin_save)) {
    validate_cursor(curwin);
    redraw_later(curwin, UPD_VALID);
    win_enter(curwin_save, true);
  }
  g_do_tagpreview = 0;
}


enum {
  SPEC_PERC = 0,
  SPEC_HASH,
  SPEC_CWORD,
  SPEC_CCWORD,
  SPEC_CEXPR,
  SPEC_CFILE,
  SPEC_SFILE,
  SPEC_SLNUM,
  SPEC_STACK,
  SPEC_SCRIPT,
  SPEC_AFILE,
  SPEC_ABUF,
  SPEC_AMATCH,
  SPEC_SFLNUM,
  SPEC_SID,
  // SPEC_CLIENT,
};


/// Evaluate cmdline variables.
///
/// change "%"       to curbuf->b_ffname
///        "#"       to curwin->w_alt_fnum
///        "<cword>" to word under the cursor
///        "<cWORD>" to WORD under the cursor
///        "<cexpr>" to C-expression under the cursor
///        "<cfile>" to path name under the cursor
///        "<sfile>" to sourced file name
///        "<stack>" to call stack
///        "<script>" to current script name
///        "<slnum>" to sourced file line number
///        "<afile>" to file name for autocommand
///        "<abuf>"  to buffer number for autocommand
///        "<amatch>" to matching name for autocommand
///
/// When an error is detected, "errormsg" is set to a non-NULL pointer (may be
/// "" for error without a message) and NULL is returned.
///
/// @param src             pointer into commandline
/// @param srcstart        beginning of valid memory for src
/// @param usedlen         characters after src that are used
/// @param lnump           line number for :e command, or NULL
/// @param errormsg        pointer to error message
/// @param escaped         return value has escaped white space (can be NULL)
/// @param empty_is_error  empty result is considered an error
///
/// @return          an allocated string if a valid match was found.
///                  Returns NULL if no match was found.  "usedlen" then still contains the
///                  number of characters to skip.
char *nvim_docmd_eval_vars_impl(char *src, const char *srcstart, size_t *usedlen, linenr_T *lnump,
                                const char **errormsg, int *escaped, bool empty_is_error)
{
  char *result = "";
  char *resultbuf = NULL;
  size_t resultlen;
  int valid = VALID_HEAD | VALID_PATH;  // Assume valid result.
  bool tilde_file = false;
  bool skip_mod = false;
  char strbuf[30];

  *errormsg = NULL;
  if (escaped != NULL) {
    *escaped = false;
  }

  // Check if there is something to do.
  ssize_t spec_idx = find_cmdline_var(src, usedlen);
  if (spec_idx < 0) {   // no match
    *usedlen = 1;
    return NULL;
  }

  // Skip when preceded with a backslash "\%" and "\#".
  // Note: In "\\%" the % is also not recognized!
  if (src > srcstart && src[-1] == '\\') {
    *usedlen = 0;
    STRMOVE(src - 1, src);      // remove backslash
    return NULL;
  }

  // word or WORD under cursor
  if (spec_idx == SPEC_CWORD
      || spec_idx == SPEC_CCWORD
      || spec_idx == SPEC_CEXPR) {
    resultlen = rs_find_ident_under_cursor(&result,
                                           spec_idx == SPEC_CWORD
                                           ? (FIND_IDENT | FIND_STRING)
                                           : (spec_idx == SPEC_CEXPR
                                              ? (FIND_IDENT | FIND_STRING | FIND_EVAL)
                                              : FIND_STRING));
    if (resultlen == 0) {
      *errormsg = "";
      return NULL;
    }
    //
    // '#': Alternate file name
    // '%': Current file name
    //        File name under the cursor
    //        File name for autocommand
    //    and following modifiers
    //
  } else {
    switch (spec_idx) {
    case SPEC_PERC:             // '%': current file
      if (curbuf->b_fname == NULL) {
        result = "";
        valid = 0;                  // Must have ":p:h" to be valid
      } else {
        result = curbuf->b_fname;
        tilde_file = strcmp(result, "~") == 0;
      }
      break;

    case SPEC_HASH:             // '#' or "#99": alternate file
      if (src[1] == '#') {          // "##": the argument list
        result = arg_all();
        resultbuf = result;
        *usedlen = 2;
        if (escaped != NULL) {
          *escaped = true;
        }
        skip_mod = true;
        break;
      }
      char *s = src + 1;
      if (*s == '<') {                  // "#<99" uses v:oldfiles.
        s++;
      }
      int i = getdigits_int(&s, false, 0);
      if (s == src + 2 && src[1] == '-') {
        // just a minus sign, don't skip over it
        s--;
      }
      *usedlen = (size_t)(s - src);           // length of what we expand

      if (src[1] == '<' && i != 0) {
        if (*usedlen < 2) {
          // Should we give an error message for #<text?
          *usedlen = 1;
          return NULL;
        }
        result = (char *)tv_list_find_str(get_vim_var_list(VV_OLDFILES), i - 1);
        if (result == NULL) {
          *errormsg = "";
          return NULL;
        }
      } else {
        if (i == 0 && src[1] == '<' && *usedlen > 1) {
          *usedlen = 1;
        }
        buf_T *buf = buflist_findnr(i);
        if (buf == NULL) {
          *errormsg = _("E194: No alternate file name to substitute for '#'");
          return NULL;
        }
        if (lnump != NULL) {
          *lnump = ECMD_LAST;
        }
        if (buf->b_fname == NULL) {
          result = "";
          valid = 0;                        // Must have ":p:h" to be valid
        } else {
          result = buf->b_fname;
          tilde_file = strcmp(result, "~") == 0;
        }
      }
      break;

    case SPEC_CFILE:            // file name under cursor
      result = file_name_at_cursor(FNAME_MESS|FNAME_HYP, 1, NULL);
      if (result == NULL) {
        *errormsg = "";
        return NULL;
      }
      resultbuf = result;                   // remember allocated string
      break;

    case SPEC_AFILE:  // file name for autocommand
      if (autocmd_fname != NULL && !autocmd_fname_full) {
        // Still need to turn the fname into a full path.  It was
        // postponed to avoid a delay when <afile> is not used.
        autocmd_fname_full = true;
        result = FullName_save(autocmd_fname, false);
        // Copy into `autocmd_fname`, don't reassign it. #8165
        xstrlcpy(autocmd_fname, result, MAXPATHL);
        xfree(result);
      }
      result = autocmd_fname;
      if (result == NULL) {
        *errormsg = _(e_no_autocommand_file_name_to_substitute_for_afile);
        return NULL;
      }
      result = path_try_shorten_fname(result);
      break;

    case SPEC_ABUF:             // buffer number for autocommand
      if (autocmd_bufnr <= 0) {
        *errormsg = _(e_no_autocommand_buffer_number_to_substitute_for_abuf);
        return NULL;
      }
      snprintf(strbuf, sizeof(strbuf), "%d", autocmd_bufnr);
      result = strbuf;
      break;

    case SPEC_AMATCH:           // match name for autocommand
      result = autocmd_match;
      if (result == NULL) {
        *errormsg = _(e_no_autocommand_match_name_to_substitute_for_amatch);
        return NULL;
      }
      break;

    case SPEC_SFILE:            // file name for ":so" command
      result = estack_sfile(ESTACK_SFILE);
      if (result == NULL) {
        *errormsg = _(e_no_source_file_name_to_substitute_for_sfile);
        return NULL;
      }
      resultbuf = result;  // remember allocated string
      break;
    case SPEC_STACK:            // call stack
      result = estack_sfile(ESTACK_STACK);
      if (result == NULL) {
        *errormsg = _(e_no_call_stack_to_substitute_for_stack);
        return NULL;
      }
      resultbuf = result;  // remember allocated string
      break;
    case SPEC_SCRIPT:           // script file name
      result = estack_sfile(ESTACK_SCRIPT);
      if (result == NULL) {
        *errormsg = _(e_no_script_file_name_to_substitute_for_script);
        return NULL;
      }
      resultbuf = result;  // remember allocated string
      break;

    case SPEC_SLNUM:            // line in file for ":so" command
      if (SOURCING_NAME == NULL || SOURCING_LNUM == 0) {
        *errormsg = _(e_no_line_number_to_use_for_slnum);
        return NULL;
      }
      snprintf(strbuf, sizeof(strbuf), "%" PRIdLINENR, SOURCING_LNUM);
      result = strbuf;
      break;

    case SPEC_SFLNUM:  // line in script file
      if (current_sctx.sc_lnum + SOURCING_LNUM == 0) {
        *errormsg = _(e_no_line_number_to_use_for_sflnum);
        return NULL;
      }
      snprintf(strbuf, sizeof(strbuf), "%" PRIdLINENR,
               current_sctx.sc_lnum + SOURCING_LNUM);
      result = strbuf;
      break;

    case SPEC_SID:
      if (current_sctx.sc_sid <= 0) {
        *errormsg = _(e_usingsid);
        return NULL;
      }
      snprintf(strbuf, sizeof(strbuf), "<SNR>%" PRIdSCID "_", current_sctx.sc_sid);
      result = strbuf;
      break;

    default:
      // should not happen
      *errormsg = "";
      break;
    }

    // Length of new string.
    resultlen = strlen(result);
    // Remove the file name extension.
    if (src[*usedlen] == '<') {
      (*usedlen)++;
      char *s;
      if ((s = strrchr(result, '.')) != NULL
          && s >= path_tail(result)) {
        resultlen = (size_t)(s - result);
      }
    } else if (!skip_mod) {
      valid |= modify_fname(src, tilde_file, usedlen, &result,
                            &resultbuf, &resultlen);
      if (result == NULL) {
        *errormsg = "";
        return NULL;
      }
    }
  }

  if (resultlen == 0 || valid != VALID_HEAD + VALID_PATH) {
    if (empty_is_error) {
      if (valid != VALID_HEAD + VALID_PATH) {
        // xgettext:no-c-format
        *errormsg = _("E499: Empty file name for '%' or '#', only works with \":p:h\"");
      } else {
        *errormsg = _("E500: Evaluates to an empty string");
      }
    }
    result = NULL;
  } else {
    result = xmemdupz(result, resultlen);
  }
  xfree(resultbuf);
  return result;
}

/// Expand the <sfile> string in "arg".
///
/// @return  an allocated string, or NULL for any error.
char *nvim_docmd_expand_sfile_impl(char *arg)
{
  char *result = xstrdup(arg);

  for (char *p = result; *p;) {
    if (strncmp(p, "<sfile>", 7) != 0) {
      p++;
    } else {
      // replace "<sfile>" with the sourced file name, and do ":" stuff
      size_t srclen;
      const char *errormsg;
      char *repl = nvim_docmd_eval_vars_impl(p, result, &srclen, NULL, &errormsg, NULL, true);
      if (errormsg != NULL) {
        if (*errormsg) {
          emsg(errormsg);
        }
        xfree(result);
        return NULL;
      }
      if (repl == NULL) {               // no match (cannot happen)
        p += srclen;
        continue;
      }
      size_t len = strlen(result) - srclen + strlen(repl) + 1;
      char *newres = xmalloc(len);
      memmove(newres, result, (size_t)(p - result));
      STRCPY(newres + (p - result), repl);
      len = strlen(newres);
      strcat(newres, p + srclen);
      xfree(repl);
      xfree(result);
      result = newres;
      p = newres + len;                 // continue after the match
    }
  }

  return result;
}

/// Make a dialog message in "buff[DIALOG_MSG_SIZE]".
/// "format" must contain "%s".
void nvim_docmd_dialog_msg_impl(char *buff, char *format, char *fname)
{
  if (fname == NULL) {
    fname = _("Untitled");
  }
  vim_snprintf(buff, DIALOG_MSG_SIZE, format, fname);
}

static TriState filetype_detect = kNone;
static TriState filetype_plugin = kNone;
static TriState filetype_indent = kNone;

/// ":filetype [plugin] [indent] {on,off,detect}"
/// on: Load the filetype.vim file to install autocommands for file types.
/// off: Load the ftoff.vim file to remove all autocommands for file types.
/// plugin on: load filetype.vim and ftplugin.vim
/// plugin off: load ftplugof.vim
/// indent on: load filetype.vim and indent.vim
/// indent off: load indoff.vim
/// Source ftplugin.vim and indent.vim to create the necessary FileType
/// autocommands. We do this separately from filetype.vim so that these
/// autocommands will always fire first (and thus can be overridden) while still
/// allowing general filetype detection to be disabled in the user's init file.
// filetype_plugin_enable / filetype_maybe_enable: implemented in Rust (Phase 6).
// Accessor functions already exist below (nvim_docmd_get_filetype_* / set_filetype_*).



// C accessor for Rust to read ex_pressedreturn
int nvim_get_ex_pressedreturn(void)
{
  return ex_pressedreturn ? 1 : 0;
}

// C accessor for Rust to read expr_map_lock
int nvim_get_expr_map_lock(void)
{
  return expr_map_lock;
}

// C accessor for Rust to check if curbuf has BF_DUMMY flag
int nvim_curbuf_is_dummy(void)
{
  return (curbuf->b_flags & BF_DUMMY) != 0;
}

// C accessors for Rust error string access
const char *nvim_get_e_invarg(void)
{
  return e_invarg;
}

const char *nvim_get_e_invarg2(void)
{
  return e_invarg2;
}

const char *nvim_get_e_invargval(void)
{
  return e_invargval;
}

const char *nvim_get_e_invrange(void)
{
  return e_invrange;
}

const char *nvim_get_e_norange(void)
{
  return e_norange;
}

const char *nvim_get_e_trailing_arg(void)
{
  return e_trailing_arg;
}

const char *nvim_get_e_curdir(void)
{
  return e_curdir;
}

const char *nvim_get_e_sandbox(void)
{
  return e_sandbox;
}

const char *nvim_get_e_using_number_as_bool_nr(void)
{
  return _(e_using_number_as_bool_nr);
}

// C accessors for Rust secure mode access
int nvim_get_secure(void)
{
  return secure;
}

void nvim_set_secure(int val)
{
  secure = val;
}

// C accessors for Rust sourcing info access
const char *nvim_get_sourcing_name(void)
{
  if (exestack.ga_data == NULL || exestack.ga_len == 0) {
    return NULL;
  }
  return SOURCING_NAME;
}

int nvim_get_sourcing_lnum(void)
{
  if (exestack.ga_data == NULL || exestack.ga_len == 0) {
    return 0;
  }
  return (int)SOURCING_LNUM;
}

int nvim_get_exestack_len(void)
{
  return exestack.ga_len;
}

// Phase 5 accessors for ex_mode, ex_swapname, ex_digraphs, ex_fold migrations
void nvim_docmd_set_must_redraw(int val) { must_redraw = val; }
const char *nvim_docmd_get_e_screenmode(void) { return _(e_screenmode); }
const char *nvim_docmd_get_curbuf_swapname(void)
{
  if (curbuf->b_ml.ml_mfp == NULL || curbuf->b_ml.ml_mfp->mf_fname == NULL) {
    return NULL;
  }
  return curbuf->b_ml.ml_mfp->mf_fname;
}
const char *nvim_docmd_no_swap_file_msg(void) { return _("No swap file"); }

// Phase 6 accessor for ex_tabnext: parse tabprevious/tabNext count argument.
// Returns the parsed count, or 0 on error (sets *errmsg_set = 1).
int nvim_docmd_parse_tabnext_count(exarg_T *eap, int *errmsg_set)
{
  char *p = eap->arg;
  char *p_save = p;
  int tab_number = (int)getdigits(&p, false, 0);
  if (p == p_save || *p_save == '-' || *p_save == '+' || *p != NUL
      || tab_number == 0) {
    eap->errmsg = ex_errmsg(e_invarg2, eap->arg);
    *errmsg_set = 1;
    return 0;
  }
  *errmsg_set = 0;
  return tab_number;
}

// Phase 7 accessors for ex_undo migration
const char *nvim_docmd_get_e_undobang(void)
{
  return _(e_undobang_cannot_redo_or_move_branch);
}
// Count the number of undo steps to reach sequence 'step' in the current branch.
// Sets *found to 1 if the target was found in the branch, 0 otherwise.
int nvim_docmd_undo_count_steps(linenr_T step, int *found)
{
  u_header_T *uhp;
  int count = 0;
  for (uhp = curbuf->b_u_curhead ? curbuf->b_u_curhead : curbuf->b_u_newhead;
       uhp != NULL && uhp->uh_seq > step;
       uhp = uhp->uh_next.ptr, ++count) {}
  *found = (step == 0 || (uhp != NULL && uhp->uh_seq >= step)) ? 1 : 0;
  return count;
}

// Phase 8 accessors for ex_sleep / do_sleep migration
int nvim_docmd_cursor_valid_curwin(void) { return cursor_valid(curwin) ? 1 : 0; }
void nvim_docmd_setcursor_mayforce_curwin(void) { setcursor_mayforce(curwin, true); }
void nvim_docmd_loop_sleep(int64_t msec)
{
  LOOP_PROCESS_EVENTS_UNTIL(&main_loop, loop_get_events(&main_loop), msec, got_int);
}

/// ":checkhealth [plugins]"
/// ":checkhealth" implementation called by Rust ex_checkhealth.
void nvim_docmd_ex_checkhealth_impl(exarg_T *eap)
{
  Error err = ERROR_INIT;
  MAXSIZE_TEMP_ARRAY(args, 2);

  char mods[1024];
  size_t mods_len = 0;
  mods[0] = NUL;

  if (cmdmod.cmod_tab > 0 || cmdmod.cmod_split != 0) {
    bool multi_mods = false;
    mods_len = add_win_cmd_modifiers(mods, &cmdmod, &multi_mods);
    assert(mods_len < sizeof(mods));
  }
  ADD_C(args, STRING_OBJ(((String){ .data = mods, .size = mods_len })));
  ADD_C(args, CSTR_AS_OBJ(eap->arg));

  NLUA_EXEC_STATIC("vim.health._check(...)", args, kRetNilBool, NULL, &err);
  if (!ERROR_SET(&err)) {
    return;
  }

  char *vimruntime_env = os_getenv_noalloc("VIMRUNTIME");
  if (!vimruntime_env) {
    emsg(_("E5009: $VIMRUNTIME is empty or unset"));
  } else {
    bool rtp_ok = NULL != strstr(p_rtp, vimruntime_env);
    if (rtp_ok) {
      semsg(_("E5009: Invalid $VIMRUNTIME: %s"), vimruntime_env);
    } else {
      emsg(_("E5009: Invalid 'runtimepath'"));
    }
  }
  semsg_multiline("emsg", err.msg);
  api_clear_error(&err);
}

/// ":terminal" implementation called by Rust ex_terminal.
void nvim_docmd_ex_terminal_impl(exarg_T *eap)
{
  char ex_cmd[1024];
  size_t len = 0;

  if (cmdmod.cmod_tab > 0 || cmdmod.cmod_split != 0) {
    bool multi_mods = false;
    // ex_cmd must be a null-terminated string before passing to add_win_cmd_modifiers
    ex_cmd[0] = NUL;
    len = add_win_cmd_modifiers(ex_cmd, &cmdmod, &multi_mods);
    assert(len < sizeof(ex_cmd));
    int result = snprintf(ex_cmd + len, sizeof(ex_cmd) - len, " new");
    assert(result > 0);
    len += (size_t)result;
  } else {
    int result = snprintf(ex_cmd, sizeof(ex_cmd), "enew%s", eap->forceit ? "!" : "");
    assert(result > 0);
    len += (size_t)result;
  }

  assert(len < sizeof(ex_cmd));

  if (*eap->arg != NUL) {  // Run {cmd} in 'shell'.
    char *name = vim_strsave_escaped(eap->arg, "\"\\");
    snprintf(ex_cmd + len, sizeof(ex_cmd) - len,
             " | call jobstart(\"%s\",{'term':v:true})", name);
    xfree(name);
  } else {  // No {cmd}: run the job with tokenized 'shell'.
    if (*p_sh == NUL) {
      emsg(_(e_shellempty));
      return;
    }

    char **argv = shell_build_argv(NULL, NULL);
    char **p = argv;
    char tempstring[512];
    char shell_argv[512] = { 0 };

    while (*p != NULL) {
      char *escaped = vim_strsave_escaped(*p, "\"\\");
      snprintf(tempstring, sizeof(tempstring), ",\"%s\"", escaped);
      xfree(escaped);
      xstrlcat(shell_argv, tempstring, sizeof(shell_argv));
      p++;
    }
    shell_free_argv(argv);

    snprintf(ex_cmd + len, sizeof(ex_cmd) - len,
             " | call jobstart([%s], {'term':v:true})", shell_argv + 1);
  }

  do_cmdline_cmd(ex_cmd);
}

/// Get argt of command with id
uint32_t get_cmd_argt(cmdidx_T cmdidx)
{
  return cmdnames[(int)cmdidx].cmd_argt;
}

/// Check if a command is a :map/:abbrev command.
bool is_map_cmd(cmdidx_T cmdidx)
{
  if (IS_USER_CMDIDX(cmdidx)) {
    return false;
  }

  ex_func_T func = cmdnames[cmdidx].cmd_func;
  return func == ex_map           // :map, :nmap, :noremap, etc.
         || func == ex_unmap         // :unmap, :nunmap, etc.
         || func == ex_mapclear      // :mapclear, :nmapclear, etc.
         || func == ex_abbreviate    // :abbreviate, :iabbrev, etc.
         || func == ex_abclear;      // :abclear, :iabclear, etc.
}

// =========================================================================
// C accessor functions for Rust FFI
// =========================================================================

/// Get the E319 "not available" error message string.
const char *nvim_docmd_e319_msg(void) { return _("E319: The command is not available in this version"); }


/// Get a pointer to IObuff.
char *nvim_docmd_get_iobuff(void)
{
  return IObuff;
}

/// Get the IOSIZE constant.
int nvim_docmd_get_iosize(void)
{
  return IOSIZE;
}


/// Concatenate to IObuff with size limit.
void nvim_docmd_xstrlcat_iobuff(const char *src)
{
  xstrlcat(IObuff, src, IOSIZE);
}

/// Inner helper for cmd_exists / f_fullcommand: look up built-in and user commands.
///
/// Creates a temporary exarg_T, calls find_ex_command, and returns:
/// - cmdidx via *out_cmdidx
/// - full match flag via *out_full
/// - useridx via *out_useridx (if non-NULL)
/// - pointer to end of command name (or NULL)
char *nvim_docmd_cmd_exists_inner(const char *name, int *out_cmdidx, int *out_full,
                                   int *out_useridx)
{
  exarg_T ea;
  ea.cmd = (char *)((*name == '2' || *name == '3') ? name + 1 : name);
  ea.cmdidx = 0;
  ea.flags = 0;
  *out_full = false;
  char *p = find_ex_command(&ea, out_full);
  *out_cmdidx = (int)ea.cmdidx;
  if (out_useridx != NULL) {
    *out_useridx = ea.useridx;
  }
  return p;
}

// =========================================================================
// exarg_T accessor functions for Rust FFI (Phase 2)
// =========================================================================

char *nvim_eap_get_arg(const exarg_T *eap) { return eap->arg; }
void nvim_eap_set_arg(exarg_T *eap, char *arg) { eap->arg = arg; }
int nvim_eap_get_cmdidx(const exarg_T *eap) { return (int)eap->cmdidx; }
uint32_t nvim_eap_get_argt(const exarg_T *eap) { return eap->argt; }
void nvim_eap_set_argt(exarg_T *eap, uint32_t argt) { eap->argt = argt; }
int nvim_eap_get_flags(const exarg_T *eap) { return eap->flags; }
void nvim_eap_set_flags(exarg_T *eap, int flags) { eap->flags = flags; }
linenr_T nvim_eap_get_line1(const exarg_T *eap) { return eap->line1; }
void nvim_eap_set_line1(exarg_T *eap, linenr_T line) { eap->line1 = line; }
linenr_T nvim_eap_get_line2(const exarg_T *eap) { return eap->line2; }
void nvim_eap_set_line2(exarg_T *eap, linenr_T line) { eap->line2 = line; }
int nvim_eap_get_addr_type(const exarg_T *eap) { return (int)eap->addr_type; }
int nvim_eap_get_addr_count(const exarg_T *eap) { return eap->addr_count; }
void nvim_eap_set_addr_count(exarg_T *eap, int count) { eap->addr_count = count; }


/// Check if a command function is "not implemented" (ex_ni or ex_script_ni).
int nvim_docmd_cmdnames_func_is_ni(int cmdidx)
{
  if (IS_USER_CMDIDX((cmdidx_T)cmdidx)) {
    return 0;
  }
  return cmdnames[cmdidx].cmd_func == ex_ni
         || cmdnames[cmdidx].cmd_func == ex_script_ni;
}

/// Wrap grep_internal for Rust access.
int nvim_docmd_grep_internal(int cmdidx)
{
  return grep_internal((cmdidx_T)cmdidx);
}

/// Get curbuf line count for set_cmd_count validation.
linenr_T nvim_docmd_get_curbuf_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

// =========================================================================
// Command table accessor functions for Rust FFI (Phase 3)
// =========================================================================

/// Get eap->cmd pointer.
char *nvim_eap_get_cmd(const exarg_T *eap) { return eap->cmd; }
/// Set eap->cmdidx.
void nvim_eap_set_cmdidx(exarg_T *eap, int idx) { eap->cmdidx = (cmdidx_T)idx; }


/// Get command_count (total commands in table).
int nvim_docmd_get_command_count(void) { return command_count; }

/// Look up initial cmdidx using cmdidxs1 table.
int nvim_docmd_get_cmdidxs1(int c)
{
  return (int)cmdidxs1[CHAR_ORD_LOW(c)];
}

/// Look up secondary offset using cmdidxs2 table.
int nvim_docmd_get_cmdidxs2(int c1, int c2)
{
  return (int)cmdidxs2[CHAR_ORD_LOW(c1)][CHAR_ORD_LOW(c2)];
}

/// Check if cmdnames[idx] name prefix-matches cmd for len chars.
int nvim_docmd_cmdnames_prefix_match(int idx, const char *cmd, int len)
{
  return strncmp(cmdnames[idx].cmd_name, cmd, (size_t)len) == 0;
}

/// Check if cmdnames[idx] name is exactly len chars (name[len] == NUL).
int nvim_docmd_cmdnames_name_complete(int idx, int len)
{
  return cmdnames[idx].cmd_name[len] == NUL;
}

/// Get cmdnames[idx].cmd_name pointer.
char *nvim_docmd_cmdnames_name(int idx)
{
  return cmdnames[idx].cmd_name;
}

/// Wrap find_ucmd for Rust access.
char *nvim_docmd_find_ucmd(exarg_T *eap, char *p, int *full)
{
  return find_ucmd(eap, p, full, NULL, NULL);
}

/// Wrap expand_user_command_name for Rust access.
char *nvim_docmd_expand_user_cmd_name(int idx)
{
  return expand_user_command_name(idx);
}

/// Report E943 and exit (command table mismatch).
void nvim_docmd_e943_abort(void)
{
  iemsg(_("E943: Command table needs to be updated, run 'make'"));
  getout(1);
}

/// Get first argvar string from typval array (for f_fullcommand).
char *nvim_docmd_tv_get_string(const void *argvars)
{
  return (char *)tv_get_string((const typval_T *)argvars);
}

/// Set rettv to VAR_STRING type with NULL initial value (for f_fullcommand).
void nvim_docmd_rettv_init_string(void *rettv)
{
  typval_T *tv = (typval_T *)rettv;
  tv->v_type = VAR_STRING;
  tv->vval.v_string = NULL;
}

/// Set rettv string to xstrdup of given string (for f_fullcommand).
void nvim_docmd_rettv_set_string(void *rettv, const char *s)
{
  typval_T *tv = (typval_T *)rettv;
  tv->vval.v_string = xstrdup(s);
}

/// Get user command name by useridx/cmdidx.
char *nvim_docmd_get_user_command_name(int useridx, int cmdidx)
{
  return get_user_command_name(useridx, (cmdidx_T)cmdidx);
}

// =========================================================================
// Phase 4 accessor functions for Rust FFI
// =========================================================================

// eap field accessors
int nvim_eap_get_regname(const exarg_T *eap) { return (int)eap->regname; }
int nvim_eap_get_amount(const exarg_T *eap) { return (int)eap->amount; }
void nvim_eap_set_regname(exarg_T *eap, int r) { eap->regname = (uint8_t)r; }
void nvim_eap_set_bad_char(exarg_T *eap, int c) { eap->bad_char = c; }
int nvim_eap_get_force_bin(const exarg_T *eap) { return eap->force_bin; }
void nvim_eap_set_force_bin(exarg_T *eap, int v) { eap->force_bin = v; }
int nvim_eap_get_force_ff(const exarg_T *eap) { return eap->force_ff; }
void nvim_eap_set_force_ff(exarg_T *eap, int v) { eap->force_ff = v; }
void nvim_eap_set_force_enc(exarg_T *eap, int v) { eap->force_enc = v; }
void nvim_eap_set_read_edit(exarg_T *eap, int v) { eap->read_edit = v; }
void nvim_eap_set_mkdir_p(exarg_T *eap, int v) { eap->mkdir_p = v; }
char *nvim_eap_get_nextcmd(const exarg_T *eap) { return eap->nextcmd; }
void nvim_eap_set_nextcmd(exarg_T *eap, char *p) { eap->nextcmd = p; }
int nvim_eap_get_skip(const exarg_T *eap) { return eap->skip; }


// Helper function wrappers
int nvim_docmd_valid_yank_reg(int regname, int writing)
{
  return valid_yank_reg(regname, writing);
}

void nvim_docmd_set_expr_line(const char *arg)
{
  set_expr_line(xstrdup(arg));
}

int nvim_docmd_check_ff_value(const char *p)
{
  return check_ff_value((char *)p);
}

void nvim_docmd_strmove(char *dst, const char *src)
{
  STRMOVE(dst, src);
}

int nvim_docmd_mb_byte2len(int b)
{
  return MB_BYTE2LEN((uint8_t)b);
}

void nvim_docmd_skip_expr(char **pp)
{
  skip_expr(pp, NULL);
}

int nvim_docmd_cpo_has_bar(void)
{
  return vim_strchr(p_cpo, CPO_BAR) != NULL;
}

char *nvim_docmd_get_dollar_command(void)
{
  return dollar_command;
}

/// Parse digits from eap->arg, advance eap->arg, return the number.
/// Also handles eap->args/arglens/argc adjustment.
int nvim_docmd_parse_count_digits(exarg_T *eap)
{
  linenr_T n = getdigits_int32(&eap->arg, false, INT32_MAX);
  eap->arg = skipwhite(eap->arg);

  if (eap->args != NULL) {
    assert(eap->argc > 0 && eap->arg >= eap->args[0]);
    if (eap->arg < eap->args[0] + eap->arglens[0]) {
      eap->arglens[0] -= (size_t)(eap->arg - eap->args[0]);
      eap->args[0] = eap->arg;
    } else {
      shift_cmd_args(eap);
    }
  }
  return (int)n;
}

/// Get e_zerocount error message.
const char *nvim_docmd_get_e_zerocount(void)
{
  return _(e_zerocount);
}


/// Advance eap->arg to end of string (skip to NUL).
void nvim_docmd_arg_skip_to_end(exarg_T *eap)
{
  eap->arg += strlen(eap->arg);
}

/// Check condition for skipdigits+whitespace in parse_count.
int nvim_docmd_count_buf_check(exarg_T *eap)
{
  char *p = skipdigits(eap->arg + 1);
  return *p == NUL || ascii_iswhite(*p);
}

// =========================================================================
// Phase 5 accessor functions for Rust FFI
// =========================================================================

// eap field accessors for Phase 5
void nvim_eap_set_addr_type(exarg_T *eap, int t) { eap->addr_type = (cmd_addr_T)t; }
char *nvim_eap_get_errmsg(const exarg_T *eap) { return eap->errmsg; }
void nvim_eap_set_errmsg(exarg_T *eap, char *msg) { eap->errmsg = msg; }
char **nvim_eap_get_cmdlinep(const exarg_T *eap) { return eap->cmdlinep; }


// cmdnames table accessor
int nvim_docmd_cmdnames_addr_type(int idx)
{
  return (int)cmdnames[idx].cmd_addr_type;
}

// bt_quickfix check for curbuf
int nvim_docmd_bt_quickfix_curbuf(void)
{
  return bt_quickfix(curbuf);
}

// Window/tab navigation accessors
int nvim_docmd_current_win_nr(void) { return CURRENT_WIN_NR; }
int nvim_docmd_last_win_nr(void) { return LAST_WIN_NR; }
int nvim_docmd_current_tab_nr(void) { return CURRENT_TAB_NR; }
int nvim_docmd_last_tab_nr(void) { return LAST_TAB_NR; }

// Cursor and arg accessors
linenr_T nvim_docmd_get_curwin_cursor_lnum(void) { return curwin->w_cursor.lnum; }
int nvim_docmd_get_curwin_arg_idx(void) { return curwin->w_arg_idx; }
int nvim_docmd_get_argcount(void) { return ARGCOUNT; }

// Buffer accessors
int nvim_docmd_get_curbuf_fnum(void) { return curbuf->b_fnum; }

// Quickfix accessors
int nvim_docmd_qf_get_cur_idx(const exarg_T *eap)
{
  return (int)qf_get_cur_idx(eap);
}

int nvim_docmd_qf_get_cur_valid_idx(const exarg_T *eap)
{
  return (int)qf_get_cur_valid_idx(eap);
}

size_t nvim_docmd_qf_get_valid_size(const exarg_T *eap)
{
  return qf_get_valid_size(eap);
}

/// Walk forward from firstbuf to find first loaded buffer.
/// Returns fnum of first loaded buffer, or -1 if none found.
int nvim_docmd_first_loaded_buf_fnum(void)
{
  buf_T *buf = firstbuf;
  while (buf->b_next != NULL && buf->b_ml.ml_mfp == NULL) {
    buf = buf->b_next;
  }
  return buf->b_fnum;
}

/// Walk backward from lastbuf to find last loaded buffer.
/// Returns fnum of last loaded buffer, or -1 if none found.
int nvim_docmd_last_loaded_buf_fnum(void)
{
  buf_T *buf = lastbuf;
  while (buf->b_prev != NULL && buf->b_ml.ml_mfp == NULL) {
    buf = buf->b_prev;
  }
  return buf->b_fnum;
}

/// Get firstbuf->b_fnum.
int nvim_docmd_firstbuf_fnum(void) { return firstbuf->b_fnum; }

/// Get lastbuf->b_fnum.
int nvim_docmd_lastbuf_fnum(void) { return lastbuf->b_fnum; }

/// Emit INTERNAL error for invalid EX_DFLALL addr_type.
void nvim_docmd_iemsg_dflall(void)
{
  iemsg(_("INTERNAL: Cannot use EX_DFLALL "
          "with ADDR_NONE, ADDR_UNSIGNED or ADDR_QUICKFIX"));
}

/// Get get_highest_fnum().
int nvim_docmd_get_highest_fnum(void) { return get_highest_fnum(); }

/// Walk forward from firstbuf: find first loaded buffer fnum.
/// Returns -1 if all buffers walked to end and none loaded.
int nvim_docmd_first_loaded_fnum_or_fail(void)
{
  buf_T *buf = firstbuf;
  while (buf->b_ml.ml_mfp == NULL) {
    if (buf->b_next == NULL) {
      return -1;
    }
    buf = buf->b_next;
  }
  return buf->b_fnum;
}

/// Walk backward from lastbuf: find last loaded buffer fnum.
/// Returns -1 if all buffers walked to start and none loaded.
int nvim_docmd_last_loaded_fnum_or_fail(void)
{
  buf_T *buf = lastbuf;
  while (buf->b_ml.ml_mfp == NULL) {
    if (buf->b_prev == NULL) {
      return -1;
    }
    buf = buf->b_prev;
  }
  return buf->b_fnum;
}

/// Get e_invrange error message.
char *nvim_docmd_get_e_invrange(void) { return _(e_invrange); }

/// Get e_no_errors error message.
char *nvim_docmd_get_e_no_errors(void) { return _(e_no_errors); }

// Tabpage accessors for get_tabpage_arg
/// Call getdigits on a string, return the number and advance the pointer.
int nvim_docmd_getdigits(char **pp, int def)
{
  return (int)getdigits(pp, false, def);
}

/// Get ex_errmsg(e_invargval, arg).
char *nvim_docmd_ex_errmsg_invargval(const char *arg)
{
  return (char *)ex_errmsg(e_invargval, arg);
}

/// Get ex_errmsg(e_invarg2, arg).
char *nvim_docmd_ex_errmsg_invarg2(const char *arg)
{
  return (char *)ex_errmsg(e_invarg2, arg);
}


// =========================================================================
// Phase 6 accessor functions for Rust FFI
// parse_command_modifiers, get_address, parse_cmd_address
// =========================================================================

// --- eap field accessors (additional) ---

/// Set eap->cmd pointer.
void nvim_eap_set_cmd(exarg_T *eap, char *p) { eap->cmd = p; }

// --- cmdmod_T field accessors ---

/// Clear a cmdmod_T struct.
void nvim_cmod_clear(cmdmod_T *cmod) { CLEAR_POINTER(cmod); }

/// OR into cmod_flags.
void nvim_cmod_or_flags(cmdmod_T *cmod, int f) { cmod->cmod_flags |= f; }

/// OR into cmod_split.
void nvim_cmod_or_split(cmdmod_T *cmod, int f) { cmod->cmod_split |= f; }
/// Set cmod_tab.
void nvim_cmod_set_tab(cmdmod_T *cmod, int v) { cmod->cmod_tab = v; }

/// Set cmod_verbose.
void nvim_cmod_set_verbose(cmdmod_T *cmod, int v) { cmod->cmod_verbose = v; }

/// Set cmod_filter_force.
void nvim_cmod_set_filter_force(cmdmod_T *cmod, int v) { cmod->cmod_filter_force = (bool)v; }

/// Set cmod_filter_pat (caller must have allocated with xstrdup).
void nvim_cmod_set_filter_pat(cmdmod_T *cmod, char *s) { cmod->cmod_filter_pat = s; }

/// Set cmod_filter_regmatch.regprog.
void nvim_cmod_set_filter_regprog(cmdmod_T *cmod, void *prog)
{
  cmod->cmod_filter_regmatch.regprog = (regprog_T *)prog;
}

// --- Global state accessors for parse_command_modifiers ---

/// Get exmode_active.
int nvim_docmd_get_exmode_active(void) { return (int)exmode_active; }

/// Check getline_equal(eap->ea_getline, eap->cookie, getexline).
int nvim_docmd_getline_is_getexline(const exarg_T *eap)
{
  return getline_equal(eap->ea_getline, eap->cookie, getexline);
}

/// Get pointer to exmode_plus string.
char *nvim_docmd_get_exmode_plus(void)
{
  return exmode_plus;
}

/// Set ex_pressedreturn.
void nvim_docmd_set_ex_pressedreturn(int val) { ex_pressedreturn = (bool)val; }

/// Wrap vim_regcomp for Rust.
void *nvim_docmd_vim_regcomp(const char *pat, int flags)
{
  return vim_regcomp((char *)pat, flags);
}

/// Wrap LAST_TAB_NR for Rust.
int nvim_docmd_LAST_TAB_NR(void) { return LAST_TAB_NR; }

/// Wrap skip_range for Rust.
char *nvim_docmd_skip_range(const char *cmd)
{
  return skip_range(cmd, NULL);
}

/// Get _(e_invrange).
char *nvim_docmd_get_e_invrange_msg(void) { return _(e_invrange); }

// --- Accessors for get_address ---

/// Set curwin->w_cursor.lnum.
void nvim_docmd_set_curwin_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }

/// Set curwin->w_cursor.col.
void nvim_docmd_set_curwin_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }


/// Get curwin->w_cursor.col.
colnr_T nvim_docmd_get_curwin_cursor_col(void) { return curwin->w_cursor.col; }

/// Get searchcmdlen.
int nvim_docmd_get_searchcmdlen(void) { return (int)searchcmdlen; }
/// Set searchcmdlen.
void nvim_docmd_set_searchcmdlen(int v) { searchcmdlen = (ptrdiff_t)v; }

/// Wrap do_search for Rust.
int nvim_docmd_do_search(exarg_T *eap, int type, int dirc, const char *pat,
                         size_t patlen, int count, int options)
{
  (void)eap;
  return do_search(NULL, type, dirc, (char *)pat, patlen, (long)count, options, NULL);
}

/// Wrap searchit for Rust.
/// Returns lnum of found position, or 0 on failure.
linenr_T nvim_docmd_searchit(int dir, int re_pat, linenr_T start_lnum,
                             colnr_T start_col, int flags)
{
  pos_T pos;
  pos.lnum = start_lnum;
  pos.col = start_col;
  pos.coladd = 0;
  if (searchit(curwin, curbuf, &pos, NULL, dir, "", 0, 1, flags, re_pat, NULL) != FAIL) {
    return pos.lnum;
  }
  return 0;
}

/// Wrap mark_get for Rust.
/// Returns opaque fmark_T pointer (NULL on failure).
void *nvim_docmd_mark_get(int flag, int ch)
{
  return mark_get(curbuf, curwin, NULL, (MarkGet)flag, (uint8_t)ch);
}

/// Check a mark and set errormsg if invalid.
int nvim_docmd_mark_check(void *fm, const char **errormsg)
{
  return mark_check((fmark_T *)fm, errormsg);
}

/// Get fmark_T->fnum.
int nvim_docmd_mark_fnum(const void *fm) { return ((const fmark_T *)fm)->fnum; }
/// Get fmark_T->mark.lnum.
linenr_T nvim_docmd_mark_lnum(const void *fm) { return ((const fmark_T *)fm)->mark.lnum; }

/// Wrap mark_move_to for Rust.
void nvim_docmd_mark_move_to(void *fm) { mark_move_to((fmark_T *)fm, 0); }

/// Get curbuf->handle.
int nvim_docmd_get_curbuf_handle(void) { return curbuf->handle; }

/// Wrap hasFolding for Rust.
/// Returns last line of fold containing lnum, or lnum if not folded.
linenr_T nvim_docmd_hasFolding(linenr_T lnum)
{
  linenr_T last;
  if (hasFolding(curwin, lnum, NULL, &last)) {
    return last;
  }
  return lnum;
}


/// Wrap getdigits_int32 for Rust.
int nvim_docmd_getdigits_int32(char **pp)
{
  return (int)getdigits_int32(pp, false, MAXLNUM);
}

/// Wrap qf_get_size for Rust.
int nvim_docmd_qf_get_size(exarg_T *eap)
{
  return (int)qf_get_size(eap);
}

/// Get _(e_norange).
char *nvim_docmd_get_e_norange(void) { return _(e_norange); }

/// Get _(e_backslash).
char *nvim_docmd_get_e_backslash(void) { return _(e_backslash); }

/// Get _(e_line_number_out_of_range).
char *nvim_docmd_get_e_line_number_out_of_range(void) { return _(e_line_number_out_of_range); }


// --- Accessors for parse_cmd_address ---

/// Wrap mark_get_visual for Rust.
void *nvim_docmd_mark_get_visual(int ch)
{
  return mark_get_visual(curbuf, (uint8_t)ch);
}

/// Wrap check_cursor(curwin).
void nvim_docmd_check_cursor(void) { check_cursor(curwin); }

/// Wrap check_cursor_col(curwin).
void nvim_docmd_check_cursor_col(void) { check_cursor_col(curwin); }

/// Check IS_USER_CMDIDX(eap->cmdidx).
int nvim_docmd_is_user_cmdidx(const exarg_T *eap) { return IS_USER_CMDIDX(eap->cmdidx); }

// =========================================================================
// Phase 1 accessor functions for Rust FFI
// (commands.rs: verify_command, skip_cmd, ex_redir, ex_normal, ex_filetype,
//  ex_quit, msg_verbose_cmd, is_other_file)
// =========================================================================

// eap accessors
// Note: nvim_eap_get_forceit already exists in indent_ffi.c

// Wrapper for static close_redir
void nvim_docmd_close_redir(void) { nvim_docmd_close_redir_impl(); }

// redir_fd: lives in globals.h as FILE*, exposed as *mut c_void
void *nvim_docmd_get_redir_fd(void) { return redir_fd; }
void nvim_docmd_set_redir_fd(void *fd) { redir_fd = (FILE *)fd; }

// redir_reg/vname setters (getters already in message.c)
void nvim_docmd_set_redir_reg(int reg) { redir_reg = (uint8_t)reg; }
void nvim_docmd_set_redir_vname(int val) { redir_vname = (bool)val; }
int nvim_docmd_get_redir_vname(void) { return redir_vname ? 1 : 0; }
void nvim_docmd_fclose_redir_fd(void) { fclose(redir_fd); redir_fd = NULL; }
void nvim_docmd_var_redir_stop(void) { var_redir_stop(); }

// ex_normal globals
int nvim_docmd_get_ex_normal_busy(void) { return ex_normal_busy; }
void nvim_docmd_set_ex_normal_busy(int val) { ex_normal_busy = val; }
int nvim_docmd_get_p_mmd(void) { return (int)p_mmd; }
int nvim_docmd_get_got_int(void) { return got_int ? 1 : 0; }

// Terminal-mode check for ex_normal
int nvim_docmd_curbuf_has_terminal(void) { return curbuf->terminal != NULL ? 1 : 0; }
int nvim_docmd_curwin_in_terminal_mode(void) { return (State & MODE_TERMINAL) ? 1 : 0; }

// exiting setter
void nvim_docmd_set_exiting(int val) { exiting = (bool)val; }

// autowriteall option
int nvim_docmd_get_p_awa(void) { return p_awa ? 1 : 0; }

// check_more logic (direct implementation for Rust FFI).
int nvim_docmd_check_more(int message, int forceit)
{
  int n = ARGCOUNT - curwin->w_arg_idx - 1;

  if (!forceit && rs_only_one_window()
      && ARGCOUNT > 1 && !arg_had_last && n > 0 && quitmore == 0) {
    if (message) {
      if ((p_confirm || (cmdmod.cmod_flags & CMOD_CONFIRM)) && curbuf->b_fname != NULL) {
        char buff[DIALOG_MSG_SIZE];

        vim_snprintf(buff, DIALOG_MSG_SIZE,
                     NGETTEXT("%d more file to edit.  Quit anyway?",
                              "%d more files to edit.  Quit anyway?", n), n);
        if (vim_dialog_yesno(VIM_QUESTION, NULL, buff, 1) == VIM_YES) {
          return OK;
        }
        return FAIL;
      }
      semsg(NGETTEXT("E173: %" PRId64 " more file to edit",
                     "E173: %" PRId64 " more files to edit", n), (int64_t)n);
      quitmore = 2;                 // next try to quit is allowed
    }
    return FAIL;
  }
  return OK;
}

// ONE_WINDOW || eap->addr_count == 0 check for ex_quit
int nvim_docmd_one_window_p(int addr_count)
{
  return (ONE_WINDOW || addr_count == 0) ? 1 : 0;
}

// filetype statics
int nvim_docmd_get_filetype_detect(void) { return (int)filetype_detect; }
void nvim_docmd_set_filetype_detect(int val) { filetype_detect = (TriState)val; }
int nvim_docmd_get_filetype_plugin(void) { return (int)filetype_plugin; }
void nvim_docmd_set_filetype_plugin(int val) { filetype_plugin = (TriState)val; }
int nvim_docmd_get_filetype_indent(void) { return (int)filetype_indent; }
void nvim_docmd_set_filetype_indent(int val) { filetype_indent = (TriState)val; }

// Filetype runtime file name constants
const char *nvim_docmd_get_filetype_file(void) { return FILETYPE_FILE; }
const char *nvim_docmd_get_ftplugin_file(void) { return FTPLUGIN_FILE; }
const char *nvim_docmd_get_indent_file(void) { return INDENT_FILE; }
const char *nvim_docmd_get_ftplugof_file(void) { return FTPLUGOF_FILE; }
const char *nvim_docmd_get_indoff_file(void) { return INDOFF_FILE; }
const char *nvim_docmd_get_ftoff_file(void) { return FTOFF_FILE; }
int nvim_docmd_get_dip_all(void) { return DIP_ALL; }

// is_other_file helpers
int nvim_docmd_curbuf_file_id_valid(void) { return curbuf->file_id_valid ? 1 : 0; }
const char *nvim_docmd_get_curbuf_sfname(void) { return curbuf->b_sfname; }

// e_invarg2 accessor (already in ex_docmd.c at line ~6369, named nvim_get_e_invarg2)
// e_secure accessor (in option_shim.c as nvim_get_e_secure)

// Phase 6 accessor: run do_cmdline with getexline callback, no flags (for do_exmode).
void nvim_docmd_do_cmdline_getexline_noflags(void) { do_cmdline(NULL, getexline, NULL, 0); }

// Phase 6 accessor: get curbuf changedtick as int64 for do_exmode.
int64_t nvim_docmd_curbuf_changedtick(void) { return (int64_t)buf_get_changedtick(curbuf); }

// Phase 6 accessor: get msg_scroll_flush.
void nvim_docmd_msg_scroll_flush(void) { msg_scroll_flush(); }

// =============================================================================
// Phase 2 accessor functions for Rust FFI (commands.rs / execute.rs)
// =============================================================================

// eap args/arglens/argc accessors
size_t nvim_eap_get_argc(const exarg_T *eap) { return (size_t)eap->argc; }
void nvim_eap_set_argc(exarg_T *eap, size_t n) { eap->argc = (int)n; }
char **nvim_eap_get_args(const exarg_T *eap) { return eap->args; }
void nvim_eap_set_args(exarg_T *eap, char **args) { eap->args = args; }
size_t *nvim_eap_get_arglens(const exarg_T *eap) { return eap->arglens; }
void nvim_eap_set_arglens(exarg_T *eap, size_t *arglens) { eap->arglens = arglens; }
bool nvim_eap_is_user_cmdidx(const exarg_T *eap) { return IS_USER_CMDIDX(eap->cmdidx); }
// Dispatch wrappers (cmdnames[] access)
void nvim_cmd_dispatch(exarg_T *eap) { (cmdnames[eap->cmdidx].cmd_func)(eap); }
int nvim_cmd_preview_dispatch(exarg_T *eap, int ns, int bufnr)
{
  return (cmdnames[eap->cmdidx].cmd_preview_func)(eap, ns, bufnr);
}

// cmdmod accessors for execute_cmd
int nvim_cmdmod_get_did_esilent(void) { return cmdmod.cmod_did_esilent; }
void nvim_cmdmod_set_did_esilent(int val) { cmdmod.cmod_did_esilent = val; }
void nvim_cmdmod_load_from_cmdinfo(const CmdParseInfo *cmdinfo) { cmdmod = cmdinfo->cmdmod; }
void nvim_cmdmod_store_to_save(cmdmod_T *save) { *save = cmdmod; }
void nvim_cmdmod_restore_from_save(const cmdmod_T *save) { cmdmod = *save; }
size_t nvim_sizeof_cmdmod_T(void) { return sizeof(cmdmod_T); }

// execute_cmd helpers
cstack_T *nvim_cstack_alloc(void)
{
  cstack_T *cs = xcalloc(1, sizeof(cstack_T));
  cs->cs_idx = -1;
  return cs;
}
void nvim_cstack_free(cstack_T *cs) { xfree(cs); }
void nvim_eap_set_cstack(exarg_T *eap, cstack_T *cstack) { eap->cstack = cstack; }
// nvim_curbuf_modifiable already exists in normal_shim.c (returns bool)
int nvim_curbuf_is_terminal(void) { return curbuf->terminal != NULL ? 1 : 0; }
const char *nvim_get_e_command_too_recursive(void) { return _(e_command_too_recursive); }
const char *nvim_get_e_modifiable(void) { return _(e_modifiable); }
// nvim_get_e_cmdwin already exists in ex_getln.c
// nvim_get_global_busy already exists in undo.c (returns bool)
int nvim_get_eap_addr_type_lines(const exarg_T *eap) { return eap->addr_type == ADDR_LINES ? 1 : 0; }
void nvim_hasFolding_line1(linenr_T lnum, linenr_T *line1_out)
{
  hasFolding(curwin, lnum, line1_out, NULL);
}
void nvim_hasFolding_line2(linenr_T lnum, linenr_T *line2_out)
{
  hasFolding(curwin, lnum, NULL, line2_out);
}

// cstack indexed field accessors
int nvim_cstack_get_idx(const cstack_T *cs) { return cs->cs_idx; }
int nvim_cstack_get_flags(const cstack_T *cs, int idx) { return cs->cs_flags[idx]; }

// profile_cmd helpers
bool nvim_getline_equal_func_line(LineGetter fgetline, void *cookie)
{
  return getline_equal(fgetline, cookie, get_func_line);
}
bool nvim_getline_equal_getsourceline(LineGetter fgetline, void *cookie)
{
  return getline_equal(fgetline, cookie, getsourceline);
}
void *nvim_getline_cookie(LineGetter fgetline, void *cookie)
{
  return getline_cookie(fgetline, cookie);
}
void nvim_func_line_exec(void *cookie) { func_line_exec(cookie); }
void nvim_script_line_exec(void) { script_line_exec(); }

// parse_cmdline helpers
void nvim_eap_init(exarg_T *eap, char *cmdline_val, char **cmdlinep)
{
  *eap = (exarg_T){
    .line1 = 1,
    .line2 = 1,
    .cmd = cmdline_val,
    .cmdlinep = cmdlinep,
    .ea_getline = NULL,
    .cookie = NULL,
  };
}
// nvim_get_ex_pressedreturn already exists above (returns int)
void nvim_set_ex_pressedreturn(bool val) { ex_pressedreturn = val; }
void nvim_save_cursor(pos_T *save) { *save = curwin->w_cursor; }
void nvim_restore_cursor(const pos_T *save) { curwin->w_cursor = *save; }
size_t nvim_sizeof_pos_T(void) { return sizeof(pos_T); }
char *nvim_find_excmd_after_range(exarg_T *eap) { return find_excmd_after_range(eap); }
const char *nvim_get_e_ambiguous_use_of_user_defined_command(void) {
  return _(e_ambiguous_use_of_user_defined_command);
}
void nvim_set_cmd_addr_type(exarg_T *eap, char *p) { set_cmd_addr_type(eap, p); }
char *nvim_skip_colon_white(const char *p, bool skipleadingwhite)
{
  return skip_colon_white(p, skipleadingwhite);
}
char *nvim_eap_get_cmd_field(const exarg_T *eap) { return eap->cmd; }
bool nvim_parse_bang(exarg_T *eap, char **p_ptr) { return parse_bang(eap, p_ptr); }
void nvim_set_eap_arg_from_p(exarg_T *eap, char *p)
{
  eap->arg = (eap->cmdidx == CMD_bang) ? p : skipwhite(p);
}
void nvim_eap_set_forceit(exarg_T *eap, bool forceit) { eap->forceit = forceit; }
bool nvim_eap_get_forceit_bool(const exarg_T *eap) { return eap->forceit; }
void nvim_separate_nextcmd(exarg_T *eap) { separate_nextcmd(eap); }
bool nvim_cmd_has_expr_args(int cmdidx) { return cmd_has_expr_args((cmdidx_T)cmdidx); }
void nvim_skip_expr_arg(char **arg) { skip_expr(arg, NULL); }
char *nvim_check_nextcmd(const char *p) { return check_nextcmd(p); }
void nvim_eap_set_nextcmd_from_colon_white(exarg_T *eap)
{
  if (eap->nextcmd) {
    eap->nextcmd = skip_colon_white(eap->nextcmd, true);
  }
}
bool nvim_eap_argt_has_trlbar(const exarg_T *eap) { return (eap->argt & EX_TRLBAR) != 0; }
bool nvim_eap_argt_has_bang(const exarg_T *eap) { return (eap->argt & EX_BANG) != 0; }
bool nvim_eap_argt_has_range(const exarg_T *eap) { return (eap->argt & EX_RANGE) != 0; }
bool nvim_eap_argt_has_dflall(const exarg_T *eap) { return (eap->argt & EX_DFLALL) != 0; }
void nvim_set_cmd_dflall_range(exarg_T *eap) { set_cmd_dflall_range(eap); }
void nvim_parse_register(exarg_T *eap) { parse_register(eap); }
void nvim_clear_cmdinfo(CmdParseInfo *cmdinfo) { CLEAR_POINTER(cmdinfo); }
bool nvim_eap_cmd_is_nul_or_comment(const exarg_T *eap)
{
  return *eap->cmd == NUL || *eap->cmd == '"';
}
size_t nvim_iosize(void) { return IOSIZE; }
void nvim_xstrlcpy(char *dst, const char *src, size_t n) { xstrlcpy(dst, src, n); }
// nvim_get_iobuff already exists in option_shim.c
void nvim_append_command(const char *cmdname) { append_command(cmdname); }
const char *nvim_get_e_not_an_editor_command(void) { return _(e_not_an_editor_command); }
void nvim_save_last_search_pattern(void) { save_last_search_pattern(); }
void nvim_restore_last_search_pattern(void) { restore_last_search_pattern(); }
// Note: parse_command_modifiers, parse_cmd_address, correct_range, parse_count
//       are all public functions accessible from Rust via extern "C"

// apply_cmdmod / undo_cmdmod on global cmdmod
void nvim_apply_global_cmdmod(void) { nvim_docmd_apply_cmdmod_impl(&cmdmod); }
void nvim_undo_global_cmdmod(void) { nvim_docmd_undo_cmdmod_impl(&cmdmod); }
void nvim_undo_cmdmod_p(CmdParseInfo *cmdinfo) { nvim_docmd_undo_cmdmod_impl(&cmdinfo->cmdmod); }

// e_nobang and e_norange error strings
const char *nvim_get_e_nobang(void) { return _(e_nobang); }
// nvim_get_e_norange already exists above

// ascii_iswhite wrapper (the inline version can't be called from Rust)
int nvim_ascii_iswhite_fn(int c) { return ascii_iswhite(c) ? 1 : 0; }



// Wrappers for static Phase 2 helpers called from Rust
int nvim_do_cmdline_start(void) { return do_cmdline_start(); }
void nvim_do_cmdline_end(void) { do_cmdline_end(); }
void nvim_correct_range(exarg_T *eap) { correct_range(eap); }
// nvim_parse_count_ex: alias for nvim_parse_count (same signature)
int nvim_parse_count_ex(exarg_T *eap, const char **errormsg, bool validate)
{
  return parse_count(eap, errormsg, validate);
}

// Phase 3 C accessor wrappers

// changedir_func helpers
bool nvim_allbuf_locked(void) { return allbuf_locked(); }

// Get previous directory string for scope (0=global, 1=tabpage, 2=window).
char *nvim_get_prevdir(int scope)
{
  switch ((CdScope)scope) {
  case kCdScopeTabpage: return curtab->tp_prevdir;
  case kCdScopeWindow:  return curwin->w_prevdir;
  default:              return prev_dir;
  }
}

// Set previous directory for scope. Returns old pdir (caller must free if replaced).
void nvim_set_prevdir(int scope, char *pdir)
{
  char **pp;
  switch ((CdScope)scope) {
  case kCdScopeTabpage:
    pp = &curtab->tp_prevdir;
    break;
  case kCdScopeWindow:
    pp = &curwin->w_prevdir;
    break;
  default:
    pp = &prev_dir;
    break;
  }
  xfree(*pp);
  *pp = pdir;
}

// os_dirname into NameBuff and return OK/FAIL.
int nvim_os_dirname_namebuff(void) { return (int)os_dirname(NameBuff, MAXPATHL); }

// expand_env("$HOME", NameBuff, MAXPATHL).
void nvim_expand_env_home_namebuff(void) { expand_env("$HOME", NameBuff, MAXPATHL); }

// Get p_cdh option value.
int nvim_get_p_cdh(void) { return p_cdh ? 1 : 0; }

// vim_chdir wrapper.
int nvim_vim_chdir(const char *dir) { return vim_chdir(dir); }

// do_autocmd_dirchanged with kCdCauseManual, pre==true.
void nvim_do_autocmd_dirchanged_manual_pre(const char *new_dir, int scope)
{
  do_autocmd_dirchanged(new_dir, (CdScope)scope, kCdCauseManual, true);
}

// post_chdir wrapper.
void nvim_post_chdir(int scope, bool dir_differs)
{
  nvim_docmd_post_chdir_impl((CdScope)scope, dir_differs);
}

const char *nvim_get_e_failed(void) { return _(e_failed); }

// Phase 3: expand_filename / repl_cmdline helpers

// eap->do_ecmd_cmd accessor
char *nvim_eap_get_do_ecmd_cmd(const exarg_T *eap) { return eap->do_ecmd_cmd; }
void nvim_eap_set_do_ecmd_cmd(exarg_T *eap, char *p) { eap->do_ecmd_cmd = p; }
char *nvim_docmd_get_do_ecmd_cmd_dollar(void) { return dollar_command; }

// eval_vars wrapper that returns the result and updates src/escaped via out-params.
// Returns NULL if no match, or the replacement string (caller must free).
char *nvim_eval_vars_wrap(exarg_T *eap, char *p, size_t *srclenp, const char **errormsgp,
                          int *escapedp)
{
  int escaped = 0;
  char *repl = nvim_docmd_eval_vars_impl(p, eap->arg, srclenp, &eap->do_ecmd_lnum, errormsgp, &escaped, true);
  *escapedp = escaped;
  return repl;
}

// eap->usefilter accessor
bool nvim_eap_get_usefilter(const exarg_T *eap) { return eap->usefilter; }

// p_wic option
int nvim_get_p_wic(void) { return p_wic ? 1 : 0; }

// backslash_halve wrapper
void nvim_backslash_halve(char *p) { backslash_halve(p); }

// expand_env_esc into NameBuff with $ and ~ expansion.
// expand_env_esc(str, NameBuff, MAXPATHL, false, true, NULL)
void nvim_expand_env_esc_namebuff_notilde(const char *str)
{
  expand_env_esc(str, NameBuff, MAXPATHL, false, true, NULL);
}

size_t nvim_ExpandT_size(void) { return sizeof(expand_T); }
void nvim_ExpandInit(expand_T *xpc) { ExpandInit(xpc); }
char *nvim_ExpandOne_files(expand_T *xpc, const char *str, int wildflags, bool icase)
{
  int opts = wildflags;
  if (icase) {
    opts += WILD_ICASE;
  }
  return ExpandOne(xpc, str, NULL, opts, WILD_EXPAND_FREE);
}

// strpbrk("!", ...) check for shell commands
bool nvim_repl_has_exclaim(const char *repl) { return strpbrk(repl, "!") != NULL; }

// vim_strsave_escaped with escape_chars (ESCAPE_CHARS on Linux = escape_chars)
char *nvim_vim_strsave_escaped_shell(const char *s)
{
#ifdef BACKSLASH_IN_FILENAME
  static char *nobslash = " \t\"|";
  return vim_strsave_escaped(s, nobslash);
#else
  return vim_strsave_escaped(s, escape_chars);
#endif
}
char *nvim_vim_strsave_escaped_bang(const char *s) { return vim_strsave_escaped(s, "!"); }

// Check if string has $ or ~ for environment variable expansion
bool nvim_has_dollar_or_tilde(const char *s)
{
  return vim_strchr(s, '$') != NULL || vim_strchr(s, '~') != NULL;
}

// vim_strchr check for %, #, < characters
bool nvim_is_expand_char(int c)
{
  return vim_strchr("%#<", (uint8_t)c) != NULL;
}

// nvim_path_has_wildcard is defined in tag_shim.c

// =============================================================================
// Phase 1 (batch plan) accessor functions for Rust FFI
// =============================================================================

/// Set eap->errmsg from a const string (for Rust FFI const safety).
void nvim_eap_set_errmsg_const(exarg_T *eap, const char *msg) { eap->errmsg = (char *)msg; }

/// Wrapper for not_restarting() -- sets restarting = false.
void nvim_docmd_not_restarting(void) { restarting = false; }

/// Set no_hlsearch and update v:hlsearch (direct implementation for Rust FFI).
void nvim_docmd_set_no_hlsearch(bool flag)
{
  no_hlsearch = flag;
  set_vim_var_nr(VV_HLSEARCH, !no_hlsearch && p_hls);
}

/// Set restart_edit to 0.
void nvim_docmd_clear_restart_edit(void) { restart_edit = 0; }

/// Set stop_insert_mode = true.
void nvim_docmd_set_stop_insert_mode(void) { stop_insert_mode = true; }

/// Call clearmode().
void nvim_docmd_clearmode(void) { clearmode(); }

/// Get the _(e_nogvim) string for :nogui.
const char *nvim_docmd_get_e_nogvim(void) { return _("E25: Nvim does not have a built-in GUI"); }

/// Get _(e_invcmd).
const char *nvim_docmd_get_e_invcmd(void) { return _(e_invcmd); }


/// Wrapper for goto_buffer(eap, DOBUF_MOD, FORWARD, eap->line2) + do_cmdline_cmd.
void nvim_docmd_goto_buffer_mod(exarg_T *eap)
{
  goto_buffer(eap, DOBUF_MOD, FORWARD, (int)eap->line2);
  if (eap->do_ecmd_cmd != NULL) {
    do_cmdline_cmd(eap->do_ecmd_cmd);
  }
}

/// Wrapper for goto_buffer(eap, DOBUF_CURRENT, FORWARD, eap->line2).
void nvim_docmd_goto_buffer_next(exarg_T *eap)
{
  goto_buffer(eap, DOBUF_CURRENT, FORWARD, (int)eap->line2);
  if (eap->do_ecmd_cmd != NULL) {
    do_cmdline_cmd(eap->do_ecmd_cmd);
  }
}

/// Wrapper for goto_buffer(eap, DOBUF_CURRENT, BACKWARD, eap->line2).
void nvim_docmd_goto_buffer_prev(exarg_T *eap)
{
  goto_buffer(eap, DOBUF_CURRENT, BACKWARD, (int)eap->line2);
  if (eap->do_ecmd_cmd != NULL) {
    do_cmdline_cmd(eap->do_ecmd_cmd);
  }
}

/// Wrapper for goto_buffer(eap, DOBUF_FIRST, FORWARD, 0).
void nvim_docmd_goto_buffer_rewind(exarg_T *eap)
{
  goto_buffer(eap, DOBUF_FIRST, FORWARD, 0);
  if (eap->do_ecmd_cmd != NULL) {
    do_cmdline_cmd(eap->do_ecmd_cmd);
  }
}

/// Wrapper for goto_buffer(eap, DOBUF_LAST, BACKWARD, 0).
void nvim_docmd_goto_buffer_last(exarg_T *eap)
{
  goto_buffer(eap, DOBUF_LAST, BACKWARD, 0);
  if (eap->do_ecmd_cmd != NULL) {
    do_cmdline_cmd(eap->do_ecmd_cmd);
  }
}


/// Wrapper for do_bang().
void nvim_docmd_do_bang(int addr_count, exarg_T *eap, bool forceit)
{
  do_bang(addr_count, eap, forceit, true, true);
}

/// Wrapper for ml_preserve(curbuf, true, true).
void nvim_docmd_ml_preserve(void) { ml_preserve(curbuf, true, true); }

/// Wrapper for u_redo(1).
void nvim_docmd_u_redo(void) { u_redo(1); }

/// Wrapper for pum_make_popup(arg, forceit).
void nvim_docmd_pum_make_popup(const char *arg, bool forceit)
{
  pum_make_popup(arg, (int)forceit);
}

/// Wrapper for u_compute_hash(curbuf, hash) + u_write_undo.
void nvim_docmd_wundo(const char *arg, bool forceit)
{
  uint8_t hash[UNDO_HASH_SIZE];
  u_compute_hash(curbuf, hash);
  u_write_undo(arg, forceit, curbuf, hash);
}

/// Wrapper for u_compute_hash(curbuf, hash) + u_read_undo.
void nvim_docmd_rundo(const char *arg)
{
  uint8_t hash[UNDO_HASH_SIZE];
  u_compute_hash(curbuf, hash);
  u_read_undo((char *)arg, hash, NULL);
}

/// Wrapper for get_tabpage_arg(eap).
int nvim_docmd_get_tabpage_arg(exarg_T *eap) { return get_tabpage_arg(eap); }

/// Wrapper for find_pattern_in_path for :checkpath.
void nvim_docmd_checkpath(bool forceit)
{
  find_pattern_in_path(NULL, 0, 0, false, false, CHECK_PATH, 1,
                       forceit ? ACTION_SHOW_ALL : ACTION_SHOW,
                       1, (linenr_T)MAXLNUM, forceit, false);
}

/// Wrapper for redraw_all_later(UPD_SOME_VALID).
void nvim_docmd_redraw_all_later_some_valid(void) { redraw_all_later(UPD_SOME_VALID); }


/// Set ex_pressedreturn (direct implementation for Rust FFI).
void nvim_docmd_set_pressedreturn(bool val) { ex_pressedreturn = val; }

// =============================================================================
// Phase 2 (batch plan) accessor functions for Rust FFI
// =============================================================================

/// Wrapper for do_bufdel (buffer unload/delete/wipe).
char *nvim_docmd_do_bufdel(int command, const char *arg, int addr_count, int start_bnr,
                            int end_bnr, int forceit)
{
  return do_bufdel(command, (char *)arg, addr_count, start_bnr, end_bnr, forceit);
}

/// Wrapper for do_autocmd.
void nvim_docmd_do_autocmd(exarg_T *eap, const char *arg, int forceit)
{
  do_autocmd(eap, (char *)arg, forceit);
}

/// Wrapper for do_augroup.
void nvim_docmd_do_augroup(const char *arg, int forceit) { do_augroup((char *)arg, forceit); }

/// Get e_curdir error message string.
const char *nvim_docmd_get_e_curdir(void) { return _(e_curdir); }

/// Wrapper for check_nomodeline.
int nvim_docmd_check_nomodeline(char **argp) { return check_nomodeline(argp) ? 1 : 0; }


// Phase 20 accessors for Rust FFI

/// Get curbuf->b_did_filetype.
bool nvim_docmd_curbuf_get_did_filetype(void) { return curbuf->b_did_filetype; }

/// Set curbuf->b_did_filetype.
void nvim_docmd_curbuf_set_did_filetype(bool val) { curbuf->b_did_filetype = val; }

/// Set filetype option to arg via set_option_value_give_err.
void nvim_docmd_set_filetype_option(const char *arg)
{
  set_option_value_give_err(kOptFiletype, CSTR_AS_OPTVAL((char *)arg), OPT_LOCAL);
}

/// setfname(curbuf, arg, NULL, true) for Rust FFI.
int nvim_docmd_setfname_curbuf(const char *arg)
{
  return setfname(curbuf, (char *)arg, NULL, true);
}

// Phase 21 accessors for Rust FFI

/// Get eval_to_string for colorscheme query.
char *nvim_docmd_eval_to_string_g_colors_name(void)
{
  char *expr = xstrdup("g:colors_name");
  emsg_off++;
  char *p = eval_to_string(expr, false, false);
  emsg_off--;
  xfree(expr);
  return p;
}

/// load_colors wrapper.
int nvim_docmd_load_colors(const char *name) { return load_colors((char *)name); }

/// Get curbuf ml_flags & ML_EMPTY.
bool nvim_docmd_curbuf_ml_empty(void) { return curbuf->b_ml.ml_flags & ML_EMPTY; }

/// Call os_breakcheck().
void nvim_docmd_os_breakcheck(void) { os_breakcheck(); }

/// Save curwin cursor position (lnum, col, coladd).
void nvim_docmd_get_curwin_cursor_pos(int *lnum, int *col, int *coladd)
{
  *lnum = (int)curwin->w_cursor.lnum;
  *col = (int)curwin->w_cursor.col;
  *coladd = (int)curwin->w_cursor.coladd;
}

/// Restore curwin cursor position (lnum, col, coladd).
void nvim_docmd_set_curwin_cursor_pos(int lnum, int col, int coladd)
{
  curwin->w_cursor.lnum = (linenr_T)lnum;
  curwin->w_cursor.col = (colnr_T)col;
  curwin->w_cursor.coladd = (colnr_T)coladd;
}

/// Get last_chdir_reason (NULL if not set).
const char *nvim_docmd_get_last_chdir_reason(void) { return last_chdir_reason; }

/// True if curwin has a local directory.
bool nvim_docmd_curwin_has_localdir(void) { return curwin->w_localdir != NULL; }

/// True if curtab has a local directory.
bool nvim_docmd_curtab_has_localdir(void) { return curtab->tp_localdir != NULL; }

/// Find Nth window in curtab (1-based). Returns lastwin if not found.
win_T *nvim_docmd_nth_curtab_window(int nr)
{
  win_T *win = NULL;
  int winnr = 0;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    winnr++;
    if (winnr == nr) {
      win = wp;
      break;
    }
  }
  return win != NULL ? win : lastwin;
}

/// win_goto wrapper.
void nvim_docmd_win_goto(win_T *wp) { win_goto(wp); }

/// close_others wrapper.
void nvim_docmd_close_others(bool message, bool forceit) { close_others(message, forceit); }


// Phase 3 C wrappers (direct implementations for Rust FFI)

/// ex_edit logic (direct implementation for Rust FFI).
// Phase 23 accessors for ex_edit
bool nvim_docmd_is_other_file(const char *ffname) { return is_other_file(0, (char *)ffname); }
bool nvim_docmd_check_can_set_curbuf_forceit(bool forceit)
{
  return check_can_set_curbuf_forceit(forceit);
}
bool nvim_docmd_bt_prompt_curbuf(void) { return bt_prompt(curbuf); }

/// get_argopt_name logic (direct implementation for Rust FFI).
char *nvim_docmd_get_argopt_name(int idx)
{
  // Note: Keep this in sync with getargopt().
  static char *(p_opt_values[]) = {
    "fileformat=",
    "encoding=",
    "binary",
    "nobinary",
    "bad=",
    "edit",
    "p",
  };

  if (idx < (int)ARRAY_SIZE(p_opt_values)) {
    return p_opt_values[idx];
  }
  return NULL;
}


// Phase 23 accessors for ex_at
int nvim_docmd_typebuf_tb_len(void) { return typebuf.tb_len; }
bool nvim_docmd_p_cpo_has_execbuf(void) { return vim_strchr(p_cpo, CPO_EXECBUF) != NULL; }
void nvim_docmd_do_cmdline_getexline(void)
{
  do_cmdline(NULL, getexline, NULL, DOCMD_NOWAIT | DOCMD_VERBOSE);
}





// =============================================================================
// Phase 10 accessor functions for Rust FFI
// =============================================================================

/// Get VIsual_active as int.
int nvim_docmd_get_VIsual_active(void) { return VIsual_active ? 1 : 0; }

/// Set virtual_op to kFalse (0).
void nvim_set_virtual_op_false(void) { virtual_op = kFalse; }

/// Set curwin->w_curswant (for ex_startinsert Rust FFI).
void nvim_docmd_set_curwin_curswant(int val) { curwin->w_curswant = (colnr_T)val; }

// =============================================================================
// Phase 17 accessor functions for Rust FFI (ex_tabclose, ex_hide, ex_wincmd,
//   ex_copymove)
// =============================================================================

/// Check if there is only one tab page.
int nvim_docmd_is_only_tabpage(void) { return first_tabpage->tp_next == NULL ? 1 : 0; }


/// Check if a tabpage handle equals curtab.
int nvim_docmd_tabpage_is_current(void *tp) { return tp == curtab ? 1 : 0; }

/// Check if a tabpage's topframe equals the global topframe.
int nvim_docmd_tabpage_is_curtopframe(void *tp) { return ((tabpage_T *)tp)->tp_topframe == topframe ? 1 : 0; }

/// Return the nth window in curtab (1-based), or lastwin if not found.
win_T *nvim_docmd_nth_window(int nr)
{
  int winnr = 0;
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    winnr++;
    if (winnr == nr) {
      return wp;
    }
  }
  return lastwin;
}

/// Get cmdmod.cmod_split.
int nvim_docmd_get_cmdmod_cmod_split(void) { return cmdmod.cmod_split; }

/// Get cmdmod.cmod_tab.
int nvim_docmd_get_cmdmod_cmod_tab(void) { return cmdmod.cmod_tab; }

/// Wrap get_address for the ex_copymove use case.
linenr_T nvim_docmd_get_address_for_copymove(exarg_T *eap, const char **errormsg)
{
  return get_address(eap, &eap->arg, eap->addr_type, false, false, false, 1, errormsg);
}

// Phase 18 accessors

/// Check if curwin->w_buffer should be hidden (for ex_exit).
int nvim_docmd_buf_hide_curwin(void) { return buf_hide(curwin->w_buffer) ? 1 : 0; }

// Accessor for update_topline_cursor_impl (migrated to Rust).
int nvim_docmd_curwin_p_wrap(void) { return curwin->w_p_wrap ? 1 : 0; }
void nvim_docmd_update_topline(void) { update_topline(curwin); }
void nvim_docmd_validate_cursor(void) { validate_cursor(curwin); }
void nvim_docmd_update_curswant(void) { update_curswant(); }

// Helpers for vim_mkdir_emsg_impl and open_exfile_impl (migrated to Rust).
void nvim_docmd_semsg_mkdir_err(const char *name, int errcode) {
  semsg(_(e_mkdir), name, os_strerror(errcode));
}
void nvim_docmd_semsg_file_exists(const char *fname) {
  semsg(_("E189: \"%s\" exists (add ! to override)"), fname);
}
void nvim_docmd_semsg_cant_open_write(const char *fname) {
  semsg(_("E190: Cannot open \"%s\" for writing"), fname);
}
void nvim_docmd_semsg_isadir2(const char *fname) { semsg(_(e_isadir2), fname); }
int nvim_docmd_os_mkdir(const char *name, int prot) { return os_mkdir(name, prot); }
int nvim_docmd_os_isdir(const char *fname) { return os_isdir(fname) ? 1 : 0; }
int nvim_docmd_os_path_exists(const char *fname) { return os_path_exists(fname) ? 1 : 0; }
FILE *nvim_docmd_os_fopen(const char *fname, const char *mode) { return os_fopen(fname, mode); }

// Phase N: replace_makeprg_impl helpers for Rust FFI
// Returns the effective grep or make program string (buffer-local if set, else global).
const char *nvim_docmd_get_grep_or_make_program(int isgrep)
{
  return isgrep ? (*curbuf->b_p_gp == NUL ? p_gp : curbuf->b_p_gp)
                : (*curbuf->b_p_mp == NUL ? p_mp : curbuf->b_p_mp);
}

// Phase N: post_chdir_impl helpers for Rust FFI
void nvim_docmd_curwin_clear_localdir(void) { XFREE_CLEAR(curwin->w_localdir); }
void nvim_docmd_curtab_clear_localdir(void) { XFREE_CLEAR(curtab->tp_localdir); }
const char *nvim_docmd_get_globaldir(void) { return globaldir; }
void nvim_docmd_set_globaldir_strdup(const char *pdir) { globaldir = xstrdup(pdir); }
void nvim_docmd_clear_globaldir(void) { XFREE_CLEAR(globaldir); }
int nvim_docmd_os_dirname_cwd(char *buf, size_t len) { return (int)os_dirname(buf, len); }
void nvim_docmd_curtab_set_localdir(const char *cwd) { curtab->tp_localdir = xstrdup(cwd); }
void nvim_docmd_curwin_set_localdir(const char *cwd) { curwin->w_localdir = xstrdup(cwd); }
void nvim_docmd_set_last_chdir_reason_null(void) { last_chdir_reason = NULL; }
void nvim_docmd_shorten_fnames_nosymlinks(void)
{
  shorten_fnames(vim_strchr(p_cpo, CPO_NOSYMLINKS) == NULL);
}
void nvim_docmd_do_autocmd_dirchanged_manual_post(const char *cwd, int scope)
{
  do_autocmd_dirchanged(cwd, (CdScope)scope, kCdCauseManual, false);
}

// Phase 4 C forwarding wrappers.
void exec_normal_cmd(char *cmd, int remap, bool silent) { nvim_docmd_exec_normal_cmd_impl(cmd, remap, silent); }
void exec_normal(bool was_typed, bool use_vpeekc) { nvim_docmd_exec_normal_impl(was_typed, use_vpeekc); }
void update_topline_cursor(void) { nvim_docmd_update_topline_cursor_impl(); }
int vim_mkdir_emsg(const char *const name, const int prot) { return nvim_docmd_vim_mkdir_emsg_impl(name, prot); }
void dialog_msg(char *buff, char *format, char *fname) { nvim_docmd_dialog_msg_impl(buff, format, fname); }
void ex_may_print(exarg_T *eap) { nvim_docmd_ex_may_print_impl(eap); }
// set_ref_in_findfunc: only called from Rust directly as nvim_docmd_set_ref_in_findfunc_impl
// free_findfunc_option: only called from option_shim.c as nvim_docmd_free_findfunc_option_impl
// did_set_findfunc: only called from option_shim.c as nvim_docmd_did_set_findfunc_impl

// Phase 3 C forwarding wrappers (original names forward to renamed impl bodies).
// These maintain ABI compatibility while Rust takes ownership via #[export_name].

void apply_cmdmod(cmdmod_T *cmod) { nvim_docmd_apply_cmdmod_impl(cmod); }
void undo_cmdmod(cmdmod_T *cmod) { nvim_docmd_undo_cmdmod_impl(cmod); }
char *replace_makeprg(exarg_T *eap, char *arg, char **cmdlinep)
{
  return nvim_docmd_replace_makeprg_impl(eap, arg, cmdlinep);
}
int expand_argopt(char *pat, expand_T *xp, regmatch_T *rmp, char ***matches, int *numMatches)
{
  return nvim_docmd_expand_argopt_impl(pat, xp, rmp, matches, numMatches);
}
FILE *open_exfile(char *fname, int forceit, char *mode)
{
  return nvim_docmd_open_exfile_impl(fname, forceit, mode);
}
bool save_current_state(save_state_T *sst) { return nvim_docmd_save_current_state_impl(sst); }
void restore_current_state(save_state_T *sst) { nvim_docmd_restore_current_state_impl(sst); }
char *eval_vars(char *src, const char *srcstart, size_t *usedlen, linenr_T *lnump,
                const char **errormsg, int *escaped, bool empty_is_error)
{
  return nvim_docmd_eval_vars_impl(src, srcstart, usedlen, lnump, errormsg, escaped,
                                   empty_is_error);
}
