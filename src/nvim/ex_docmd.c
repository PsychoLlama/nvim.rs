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
// Phase 4 Rust exports: do_one_cmd and ex_errmsg implemented in do_one_cmd.rs
extern char *do_one_cmd(char **cmdlinep, int flags, cstack_T *cstack, LineGetter fgetline,
                        void *cookie);
extern char *ex_errmsg(const char *const msg, const char *const arg);

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
extern void nvim_docmd_apply_cmdmod_impl(cmdmod_T *cmod);
extern void nvim_docmd_undo_cmdmod_impl(cmdmod_T *cmod);
extern bool nvim_docmd_save_current_state_impl(save_state_T *sst);
extern void nvim_docmd_restore_current_state_impl(save_state_T *sst);
extern void nvim_docmd_ex_splitview_impl(exarg_T *eap);
extern void nvim_docmd_exec_normal_cmd_impl(char *cmd, int remap, bool silent);
extern void nvim_docmd_exec_normal_impl(bool was_typed, bool use_vpeekc);

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

/// do_cmdline(): execute one Ex command line.
///
/// 1. Execute "cmdline" when it is not NULL.
///    If "cmdline" is NULL, or more lines are needed, fgetline() is used.
/// 2. Split up in parts separated with '|'.
///
/// This function can be called recursively!
///
/// @param cookie  argument for fgetline()
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


/// Helper function to apply an offset for buffer commands, i.e. ":bdelete",
/// ":bwipeout", etc.
///
/// @return  the buffer number.
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


/// The "+" string used in place of an empty command in Ex mode.
/// This string is used in pointer comparison.
static char exmode_plus[] = "+";

// excmd_get_argt is implemented in Rust (do_one_cmd.rs), declared in ex_docmd.h.
// nvim_docmd_get_bad_name is implemented in Rust (completion.rs).
extern char *nvim_docmd_get_bad_name(expand_T *xp, int idx);

/// callback function for 'findfunc'
static Callback ffu_cb;

/// Return a pointer to the global ffu_cb (for Rust FFI).
Callback *nvim_docmd_get_ffu_cb_ptr(void) { return &ffu_cb; }

static Callback *get_findfunc_callback(void)
{
  return *curbuf->b_p_ffu != NUL ? &curbuf->b_ffu_cb : &ffu_cb;
}

/// Call 'findfunc' to obtain a list of file names.
/// Public C wrapper callable from Rust (hides typval_T / Callback complexity).
list_T *nvim_docmd_call_findfunc(char *pat, bool cmdcomplete)
{
  const sctx_T saved_sctx = current_sctx;

  typval_T args[3];
  args[0].v_type = VAR_STRING;
  args[0].vval.v_string = pat;
  args[1].v_type = VAR_BOOL;
  args[1].vval.v_bool = (BoolVarValue)cmdcomplete;
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

/// List length accessor for Rust FFI (tv_list_len is a static inline).
int nvim_docmd_tv_list_len(const list_T *l) { return tv_list_len(l); }

/// List free accessor for Rust FFI.
void nvim_docmd_tv_list_free(list_T *l) { tv_list_free(l); }

/// Error message accessor for Rust FFI.
const char *nvim_docmd_e_cant_find_file_str_in_path(void)
{
  return _(e_cant_find_file_str_in_path);
}

/// Error message accessor for Rust FFI.
const char *nvim_docmd_e_no_more_file_str_found_in_path(void)
{
  return _(e_no_more_file_str_found_in_path);
}

// expand_findfunc is implemented in Rust (findfunc.rs).
extern int expand_findfunc(char *pat, char ***files, int *numMatches);

// nvim_docmd_findfunc_find_file is implemented in Rust (findfunc.rs).
extern char *nvim_docmd_findfunc_find_file(char *arg, size_t len, int count);

// nvim_docmd_did_set_findfunc_impl is implemented in Rust (impl_bodies.rs).

// nvim_docmd_free_findfunc_option_impl is implemented in Rust (findfunc.rs).
extern void nvim_docmd_free_findfunc_option_impl(void);

// nvim_docmd_set_ref_in_findfunc_impl is implemented in Rust (findfunc.rs).
extern bool nvim_docmd_set_ref_in_findfunc_impl(int copyID);

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
// Body migrated to Rust (Phase N+16). Forward decl at top of file.

// nvim_docmd_tabpage_new_impl is implemented in Rust (impl_bodies.rs).



// nvim_docmd_ex_detach_impl is implemented in Rust (impl_bodies.rs).

// nvim_docmd_ex_connect_impl is implemented in Rust (impl_bodies.rs).

/// ":edit", ":badd", ":balt", ":visual".
/// ":edit <file>" command and alike.
///
/// @param old_curwin  curwin before doing a split or NULL
// Body migrated to Rust (Phase N+17). Forward decl at top of file.


// nvim_docmd_ex_read_impl is implemented in Rust (impl_bodies.rs).

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


// nvim_docmd_eval_vars_impl is implemented in Rust (eval_vars.rs).
extern char *nvim_docmd_eval_vars_impl(char *src, const char *srcstart, size_t *usedlen,
                                       linenr_T *lnump, const char **errormsg, int *escaped,
                                       bool empty_is_error);



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

// nvim_docmd_ex_checkhealth_impl is implemented in Rust (impl_bodies.rs).

// nvim_docmd_ex_terminal_impl is implemented in Rust (impl_bodies.rs).


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

// --- Accessors for apply_cmdmod_impl / undo_cmdmod_impl (Rust Phase N+14) ---
int nvim_cmod_get_flags(cmdmod_T *cmod) { return cmod->cmod_flags; }
int nvim_cmod_get_did_sandbox(cmdmod_T *cmod) { return cmod->cmod_did_sandbox ? 1 : 0; }
void nvim_cmod_set_did_sandbox(cmdmod_T *cmod, int v) { cmod->cmod_did_sandbox = (bool)v; }
int nvim_cmod_get_verbose(cmdmod_T *cmod) { return cmod->cmod_verbose; }
int nvim_cmod_get_verbose_save(cmdmod_T *cmod) { return cmod->cmod_verbose_save; }
void nvim_cmod_set_verbose_save(cmdmod_T *cmod, int v) { cmod->cmod_verbose_save = v; }
int nvim_cmod_get_save_msg_silent(cmdmod_T *cmod) { return cmod->cmod_save_msg_silent; }
void nvim_cmod_set_save_msg_silent(cmdmod_T *cmod, int v) { cmod->cmod_save_msg_silent = v; }
void nvim_cmod_capture_msg_scroll(cmdmod_T *cmod) { cmod->cmod_save_msg_scroll = msg_scroll; }
void nvim_cmod_inc_did_esilent(cmdmod_T *cmod) { cmod->cmod_did_esilent++; }
int nvim_cmod_get_did_esilent(cmdmod_T *cmod) { return cmod->cmod_did_esilent; }
void nvim_cmod_set_did_esilent(cmdmod_T *cmod, int v) { cmod->cmod_did_esilent = v; }
char *nvim_cmod_get_save_ei(cmdmod_T *cmod) { return cmod->cmod_save_ei; }
void nvim_cmod_set_save_ei(cmdmod_T *cmod, char *s) { cmod->cmod_save_ei = s; }
void nvim_docmd_set_eventignore_all(void)
{
  set_option_direct(kOptEventignore, STATIC_CSTR_AS_OPTVAL("all"), 0, SID_NONE);
}
void nvim_docmd_set_eventignore_str(char *s)
{
  set_option_direct(kOptEventignore, CSTR_AS_OPTVAL(s), 0, SID_NONE);
}
void nvim_cmod_regfree_filter(cmdmod_T *cmod)
{
  vim_regfree(cmod->cmod_filter_regmatch.regprog);
  cmod->cmod_filter_regmatch.regprog = NULL;
}
char *nvim_cmod_get_filter_pat(cmdmod_T *cmod) { return cmod->cmod_filter_pat; }
void nvim_docmd_restore_msg_scroll(cmdmod_T *cmod) { msg_scroll = cmod->cmod_save_msg_scroll; }

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

// nvim_docmd_check_more is implemented in Rust (impl_bodies.rs).

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
// nvim_find_excmd_after_range: inline the logic to avoid circular call with Rust export.
char *nvim_find_excmd_after_range(exarg_T *eap)
{
  char *cmd = eap->cmd;
  eap->cmd = skip_range(eap->cmd, NULL);
  if (*eap->cmd == '*') {
    eap->cmd = skipwhite(eap->cmd + 1);
  }
  char *p = find_ex_command(eap, NULL);
  eap->cmd = cmd;
  return p;
}
const char *nvim_get_e_ambiguous_use_of_user_defined_command(void) {
  return _(e_ambiguous_use_of_user_defined_command);
}
void nvim_set_cmd_addr_type(exarg_T *eap, char *p) { set_cmd_addr_type(eap, p); }
char *nvim_skip_colon_white(const char *p, bool skipleadingwhite)
{
  return skip_colon_white(p, skipleadingwhite);
}
bool nvim_parse_bang(exarg_T *eap, char **p_ptr) { return parse_bang(eap, p_ptr); }
void nvim_set_eap_arg_from_p(exarg_T *eap, char *p)
{
  eap->arg = (eap->cmdidx == CMD_bang) ? p : skipwhite(p);
}
void nvim_eap_set_forceit(exarg_T *eap, bool forceit) { eap->forceit = forceit; }
bool nvim_eap_get_forceit_bool(const exarg_T *eap) { return eap->forceit; }
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

const char *nvim_get_e_not_an_editor_command(void) { return _(e_not_an_editor_command); }
void nvim_save_last_search_pattern(void) { save_last_search_pattern(); }
void nvim_restore_last_search_pattern(void) { restore_last_search_pattern(); }
// Note: parse_command_modifiers, parse_cmd_address, correct_range, parse_count
//       are all public functions accessible from Rust via extern "C"

// apply_cmdmod / undo_cmdmod on global cmdmod
void nvim_apply_global_cmdmod(void) { nvim_docmd_apply_cmdmod_impl(&cmdmod); }
void nvim_undo_global_cmdmod(void) { nvim_docmd_undo_cmdmod_impl(&cmdmod); }

/// For do_exbuffer_impl: call goto_buffer(eap, DOBUF_CURRENT, FORWARD, 0).
void nvim_docmd_goto_buffer_current(exarg_T *eap)
{
  goto_buffer(eap, DOBUF_CURRENT, FORWARD, 0);
}
/// For do_exbuffer_impl: call goto_buffer(eap, DOBUF_FIRST, FORWARD, n).
void nvim_docmd_goto_buffer_first(exarg_T *eap, int n)
{
  goto_buffer(eap, DOBUF_FIRST, FORWARD, n);
}
/// For do_exbuffer_impl: get eap->do_ecmd_cmd.
char *nvim_docmd_eap_get_do_ecmd_cmd(const exarg_T *eap) { return eap->do_ecmd_cmd; }
/// For do_exbuffer_impl: ex_errmsg(e_trailing_arg, arg).
char *nvim_docmd_errmsg_trailing_arg(const char *arg) { return ex_errmsg(e_trailing_arg, arg); }

/// For ex_find_impl: returns true if 'findfunc' option is non-empty.
bool nvim_docmd_get_findfunc_nonempty(void) { return *get_findfunc() != NUL; }
// nvim_docmd_findfunc_find_file is implemented in Rust (findfunc.rs).
/// For ex_find_impl: returns curbuf->b_ffname.
const char *nvim_docmd_curbuf_b_ffname(void) { return curbuf->b_ffname; }

/// For handle_did_throw_impl: free SOURCING_NAME then pop estack.
void nvim_docmd_free_sourcing_name_and_pop(void)
{
  xfree(SOURCING_NAME);
  estack_pop();
}

/// Helpers for nvim_docmd_ex_tabs_impl (Rust).
/// Write translated "Tab page %d" into IObuff and return a pointer to it.
char *nvim_docmd_tab_page_fmt(int n)
{
  vim_snprintf(IObuff, IOSIZE, _("Tab page %d"), n);
  return IObuff;
}
/// Call msg_outtrans with an attribute (e.g. HLF_T).
void nvim_docmd_msg_outtrans_attr(const char *s, int attr)
{
  msg_outtrans((char *)s, attr, false);
}
/// home_replace into IObuff.
void nvim_docmd_home_replace(buf_T *buf, const char *src)
{
  home_replace(buf, src, IObuff, IOSIZE, true);
}
// nvim_docmd_get_iobuff already defined in option_shim.c as nvim_get_iobuff
void nvim_undo_cmdmod_p(CmdParseInfo *cmdinfo) { nvim_docmd_undo_cmdmod_impl(&cmdinfo->cmdmod); }

// e_nobang and e_norange error strings
const char *nvim_get_e_nobang(void) { return _(e_nobang); }
// nvim_get_e_norange already exists above

// ascii_iswhite wrapper (the inline version can't be called from Rust)
int nvim_ascii_iswhite_fn(int c) { return ascii_iswhite(c) ? 1 : 0; }



// Wrappers for static Phase 2 helpers called from Rust
int nvim_do_cmdline_start(void) { return do_cmdline_start(); }
void nvim_do_cmdline_end(void) { do_cmdline_end(); }
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

bool nvim_docmd_check_can_set_curbuf_forceit(bool forceit)
{
  return check_can_set_curbuf_forceit(forceit);
}
bool nvim_docmd_bt_prompt_curbuf(void) { return bt_prompt(curbuf); }

// nvim_docmd_get_argopt_name is implemented in Rust (completion.rs).


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

// Phase N: ex_win_close_impl helpers for Rust FFI
// Show dialog_changed prompt and return true if buffer is still changed after.
bool nvim_docmd_dialog_changed_still_dirty(buf_T *buf)
{
  bufref_T bufref;
  set_bufref(&bufref, buf);
  dialog_changed(buf, false);
  return bufref_valid(&bufref) && bufIsChanged(buf);
}
void nvim_docmd_no_write_message(void) { no_write_message(); }

// Phase N: tabpage_close_impl / tabpage_close_other_impl helpers for Rust FFI
int nvim_docmd_curwin_is_floating(void) { return curwin->w_floating ? 1 : 0; }
int nvim_docmd_is_one_window(void) { return ONE_WINDOW ? 1 : 0; }
void nvim_docmd_ex_win_close_curwin(int forceit) { ex_win_close(forceit, curwin, NULL); }
win_T *nvim_docmd_tabpage_get_lastwin(void *tp) { return ((tabpage_T *)tp)->tp_lastwin; }
int nvim_docmd_tabpage_lastwin_eq(void *tp, void *wp) { return ((tabpage_T *)tp)->tp_lastwin == (win_T *)wp ? 1 : 0; }
void nvim_docmd_ex_win_close_in_tab(int forceit, void *wp, void *tp)
{
  ex_win_close(forceit, (win_T *)wp, (tabpage_T *)tp);
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
void dialog_msg(char *buff, char *format, char *fname)
{
  if (fname == NULL) {
    fname = _("Untitled");
  }
  vim_snprintf(buff, DIALOG_MSG_SIZE, format, fname);
}
void ex_may_print(exarg_T *eap) { nvim_docmd_ex_may_print_impl(eap); }
// set_ref_in_findfunc: only called from Rust directly as nvim_docmd_set_ref_in_findfunc_impl
// free_findfunc_option: only called from option_shim.c as nvim_docmd_free_findfunc_option_impl
// did_set_findfunc: only called from option_shim.c as nvim_docmd_did_set_findfunc_impl

// Phase 3 C forwarding wrappers (original names forward to renamed impl bodies).
// These maintain ABI compatibility while Rust takes ownership via #[export_name].

// apply_cmdmod / undo_cmdmod - thin C wrappers calling Rust _impl bodies.
void apply_cmdmod(cmdmod_T *cmod) { nvim_docmd_apply_cmdmod_impl(cmod); }
void undo_cmdmod(cmdmod_T *cmod) { nvim_docmd_undo_cmdmod_impl(cmod); }
char *replace_makeprg(exarg_T *eap, char *arg, char **cmdlinep)
{
  return nvim_docmd_replace_makeprg_impl(eap, arg, cmdlinep);
}
// Forward declaration for Rust-implemented expand_argopt (impl_bodies.rs)
extern int nvim_docmd_expand_argopt_impl(char *pat, expand_T *xp, regmatch_T *rmp,
                                         char ***matches, int *numMatches);
int expand_argopt(char *pat, expand_T *xp, regmatch_T *rmp, char ***matches, int *numMatches)
{
  return nvim_docmd_expand_argopt_impl(pat, xp, rmp, matches, numMatches);
}
FILE *open_exfile(char *fname, int forceit, char *mode)
{
  return nvim_docmd_open_exfile_impl(fname, forceit, mode);
}
// --- Accessors for save_current_state_impl / restore_current_state_impl (Phase N+15) ---
// Global accessors
int nvim_get_restart_edit(void) { return restart_edit; }
void nvim_set_restart_edit(int val) { restart_edit = val; }
int nvim_get_force_restart_edit(void) { return force_restart_edit ? 1 : 0; }
void nvim_set_force_restart_edit(int val) { force_restart_edit = (bool)val; }
void nvim_set_current_State(int val) { State = val; }
// sst field setters
void nvim_docmd_sst_set_msg_scroll(save_state_T *sst, int v) { sst->save_msg_scroll = v; }
void nvim_docmd_sst_set_restart_edit(save_state_T *sst, int v) { sst->save_restart_edit = v; }
void nvim_docmd_sst_set_msg_didout(save_state_T *sst, int v) { sst->save_msg_didout = (bool)v; }
void nvim_docmd_sst_set_State(save_state_T *sst, int v) { sst->save_State = v; }
void nvim_docmd_sst_set_finish_op(save_state_T *sst, int v) { sst->save_finish_op = (bool)v; }
void nvim_docmd_sst_set_opcount(save_state_T *sst, int v) { sst->save_opcount = v; }
void nvim_docmd_sst_set_reg_executing(save_state_T *sst, int v) { sst->save_reg_executing = v; }
void nvim_docmd_sst_set_pending_end_reg_executing(save_state_T *sst, int v)
{
  sst->save_pending_end_reg_executing = (bool)v;
}
// sst field getters
int nvim_docmd_sst_get_msg_scroll(save_state_T *sst) { return sst->save_msg_scroll; }
int nvim_docmd_sst_get_restart_edit(save_state_T *sst) { return sst->save_restart_edit; }
int nvim_docmd_sst_get_State(save_state_T *sst) { return sst->save_State; }
int nvim_docmd_sst_get_finish_op(save_state_T *sst) { return sst->save_finish_op ? 1 : 0; }
int nvim_docmd_sst_get_opcount(save_state_T *sst) { return sst->save_opcount; }
int nvim_docmd_sst_get_reg_executing(save_state_T *sst) { return sst->save_reg_executing; }
int nvim_docmd_sst_get_pending_end_reg_executing(save_state_T *sst)
{
  return sst->save_pending_end_reg_executing ? 1 : 0;
}
int nvim_docmd_sst_get_msg_didout(save_state_T *sst) { return sst->save_msg_didout ? 1 : 0; }
// typeahead save/restore
int nvim_docmd_sst_save_typeahead(save_state_T *sst)
{
  save_typeahead(&sst->tabuf);
  return sst->tabuf.typebuf_valid ? 1 : 0;
}
void nvim_docmd_sst_restore_typeahead(save_state_T *sst)
{
  restore_typeahead(&sst->tabuf);
}
// forward decls for Rust implementations
extern bool nvim_docmd_save_current_state_impl(save_state_T *sst);
extern void nvim_docmd_restore_current_state_impl(save_state_T *sst);

bool save_current_state(save_state_T *sst) { return nvim_docmd_save_current_state_impl(sst); }
void restore_current_state(save_state_T *sst) { nvim_docmd_restore_current_state_impl(sst); }

// --- Accessors for ex_splitview_impl (Phase N+16) ---
int nvim_docmd_get_CMD_tabedit(void) { return (int)CMD_tabedit; }
int nvim_docmd_get_CMD_tabfind(void) { return (int)CMD_tabfind; }
int nvim_docmd_get_CMD_tabnew(void) { return (int)CMD_tabnew; }
int nvim_docmd_get_CMD_split(void) { return (int)CMD_split; }
int nvim_docmd_get_CMD_vsplit(void) { return (int)CMD_vsplit; }
int nvim_docmd_get_CMD_new(void) { return (int)CMD_new; }
int nvim_docmd_get_CMD_vnew(void) { return (int)CMD_vnew; }
int nvim_docmd_get_CMD_sfind(void) { return (int)CMD_sfind; }
/// Set w_alt_fnum for an arbitrary window (used after tabpage switch).
void nvim_docmd_win_set_alt_fnum(win_T *wp, int fnum) { wp->w_alt_fnum = fnum; }
/// Get curbuf->b_fnum.
int nvim_docmd_curbuf_b_fnum(void) { return curbuf->b_fnum; }
/// Get cmdmod.cmod_flags (global).
int nvim_docmd_get_global_cmdmod_flags(void) { return cmdmod.cmod_flags; }

// --- Accessors for do_exedit_impl (Phase N+17) ---
int nvim_docmd_get_CMD_visual(void) { return (int)CMD_visual; }
int nvim_docmd_get_CMD_view(void) { return (int)CMD_view; }
int nvim_docmd_get_CMD_enew(void) { return (int)CMD_enew; }
int nvim_docmd_get_CMD_sview(void) { return (int)CMD_sview; }
int nvim_docmd_get_CMD_balt(void) { return (int)CMD_balt; }
int nvim_docmd_get_CMD_badd(void) { return (int)CMD_badd; }
int nvim_docmd_get_readonlymode(void) { return readonlymode ? 1 : 0; }
void nvim_docmd_set_readonlymode(int v) { readonlymode = (v != 0); }
int nvim_docmd_buf_hide_buf(buf_T *buf) { return buf_hide(buf) ? 1 : 0; }
void nvim_docmd_set_curbuf_b_p_ro(int v) { curbuf->b_p_ro = (v != 0); }
linenr_T nvim_docmd_eap_get_do_ecmd_lnum(const exarg_T *eap) { return eap->do_ecmd_lnum; }
// nvim_docmd_do_exedit_handle_exmode, nvim_docmd_do_exedit_split_fail_cleanup,
// and nvim_docmd_do_exedit_split_fallback are implemented in Rust (impl_bodies.rs).

char *eval_vars(char *src, const char *srcstart, size_t *usedlen, linenr_T *lnump,
                const char **errormsg, int *escaped, bool empty_is_error)
{
  return nvim_docmd_eval_vars_impl(src, srcstart, usedlen, lnump, errormsg, escaped,
                                   empty_is_error);
}

// =============================================================================
// Phase 3 eval_vars.rs C accessor functions
// =============================================================================

char *nvim_docmd_eval_curbuf_fname(void) { return curbuf->b_fname; }

char *nvim_docmd_file_name_at_cursor(void)
{
  return file_name_at_cursor(FNAME_MESS | FNAME_HYP, 1, NULL);
}

char *nvim_docmd_fullname_save(const char *fname, bool force)
{
  return FullName_save(fname, force);
}

char *nvim_docmd_get_autocmd_fname(void) { return autocmd_fname; }
int nvim_docmd_get_autocmd_fname_full(void) { return autocmd_fname_full ? 1 : 0; }
void nvim_docmd_set_autocmd_fname_full(int v) { autocmd_fname_full = (v != 0); }
void nvim_docmd_set_autocmd_fname(const char *new_fname)
{
  xstrlcpy(autocmd_fname, new_fname, MAXPATHL);
}
const char *nvim_docmd_get_autocmd_match(void) { return autocmd_match; }

char *nvim_docmd_estack_sfile(int which) { return estack_sfile((estack_arg_T)which); }

int nvim_docmd_modify_fname(char *src, bool tilde_file, size_t *usedlen,
                             char **fnamep, char **bufp, size_t *fnamelen)
{
  return modify_fname(src, tilde_file, usedlen, fnamep, bufp, fnamelen);
}

const char *nvim_docmd_path_tail(const char *p) { return path_tail(p); }

char *nvim_docmd_path_try_shorten_fname(char *full_path)
{
  return path_try_shorten_fname(full_path);
}

int nvim_docmd_get_current_sctx_lnum(void) { return (int)current_sctx.sc_lnum; }
int nvim_docmd_get_current_sctx_sid(void) { return (int)current_sctx.sc_sid; }

int nvim_docmd_getdigits_int(char **pp)
{
  return getdigits_int(pp, false, 0);
}

// Error string accessors for eval_vars.rs
const char *nvim_docmd_eval_get_e_no_alt_file(void)
{
  return _("E194: No alternate file name to substitute for '#'");
}
const char *nvim_docmd_eval_get_e_no_afile(void)
{
  return _(e_no_autocommand_file_name_to_substitute_for_afile);
}
const char *nvim_docmd_eval_get_e_no_abuf(void)
{
  return _(e_no_autocommand_buffer_number_to_substitute_for_abuf);
}
const char *nvim_docmd_eval_get_e_no_amatch(void)
{
  return _(e_no_autocommand_match_name_to_substitute_for_amatch);
}
const char *nvim_docmd_eval_get_e_no_sfile(void)
{
  return _(e_no_source_file_name_to_substitute_for_sfile);
}
const char *nvim_docmd_eval_get_e_no_stack(void)
{
  return _(e_no_call_stack_to_substitute_for_stack);
}
const char *nvim_docmd_eval_get_e_no_slnum(void)
{
  return _(e_no_line_number_to_use_for_slnum);
}
const char *nvim_docmd_eval_get_e_no_sflnum(void)
{
  return _(e_no_line_number_to_use_for_sflnum);
}
const char *nvim_docmd_eval_get_e_no_script(void)
{
  return _(e_no_script_file_name_to_substitute_for_script);
}
const char *nvim_docmd_eval_get_e_usingsid(void)
{
  return _(e_usingsid);
}
const char *nvim_docmd_eval_get_e_empty_fname(void)
{
  // xgettext:no-c-format
  return _("E499: Empty file name for '%' or '#', only works with \":p:h\"");
}
const char *nvim_docmd_eval_get_e_empty_string(void)
{
  return _("E500: Evaluates to an empty string");
}

// =============================================================================
// Phase 1 impl_bodies.rs C accessor functions
// =============================================================================

// --- check_more helpers ---
int nvim_docmd_get_quitmore(void) { return quitmore; }
void nvim_docmd_set_quitmore(int n) { quitmore = n; }
void nvim_docmd_check_more_semsg(int n)
{
  semsg(NGETTEXT("E173: %" PRId64 " more file to edit",
                 "E173: %" PRId64 " more files to edit", (unsigned)n), (int64_t)n);
}
int nvim_docmd_check_more_dialog(int n)
{
  char buff[DIALOG_MSG_SIZE];
  vim_snprintf(buff, DIALOG_MSG_SIZE,
               NGETTEXT("%d more file to edit.  Quit anyway?",
                        "%d more files to edit.  Quit anyway?", n), n);
  return vim_dialog_yesno(VIM_QUESTION, NULL, buff, 1);
}
// nvim_al_get_arg_had_last is defined in arglist.c
// nvim_get_p_confirm and nvim_get_cmdmod_confirm are defined in window_shim.c

// --- tabpage_new_body helper ---
// Calls nvim_docmd_ex_splitview_impl with a stack-allocated ea for CMD_tabnew.
void nvim_docmd_tabpage_new_body(void)
{
  exarg_T ea = { .cmdidx = CMD_tabnew, .cmd = "tabn", .arg = "" };
  nvim_docmd_ex_splitview_impl(&ea);
}

// --- do_exedit_handle_exmode helpers ---
void nvim_docmd_set_exmode_active(int v) { exmode_active = (bool)v; }
void nvim_docmd_stuffReadbuff_str(const char *s) { stuffReadbuff(s); }
int nvim_docmd_get_pending_exmode_active(void) { return pending_exmode_active ? 1 : 0; }
void nvim_docmd_set_pending_exmode_active(int v) { pending_exmode_active = (bool)v; }
void nvim_docmd_normal_enter_false_true(void) { normal_enter(false, true); }

// --- do_exedit_split_fail_cleanup helpers ---
int nvim_docmd_curbuf_b_nwindows(void) { return curbuf->b_nwindows; }

// --- ex_read helpers ---
int nvim_docmd_curbuf_ml_has_empty(void) { return (curbuf->b_ml.ml_flags & ML_EMPTY) ? 1 : 0; }
void nvim_docmd_do_bang_read(exarg_T *eap) { do_bang(1, eap, false, false, true); }
const char *nvim_docmd_curbuf_b_fname(void) { return curbuf->b_fname; }
// nvim_get_p_cpo is defined in Rust (window crate globals.rs)
const char *nvim_docmd_e_notopen_str(void) { return _(e_notopen); }
linenr_T nvim_docmd_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
void nvim_docmd_curwin_cursor_lnum_maybe_dec(linenr_T lnum)
{
  if (curwin->w_cursor.lnum > 1 && curwin->w_cursor.lnum >= lnum) {
    curwin->w_cursor.lnum--;
  }
}

// --- do_exedit_split_fallback helpers ---
int nvim_docmd_curwin_w_arg_idx_invalid(void) { return curwin->w_arg_idx_invalid ? 1 : 0; }
void nvim_docmd_check_arg_idx_curwin(void) { check_arg_idx(curwin); }

// --- ex_checkhealth/ex_terminal helpers ---
size_t nvim_docmd_add_win_cmd_modifiers_global(char *buf, size_t bufsize)
{
  bool multi_mods = false;
  buf[0] = NUL;
  size_t len = add_win_cmd_modifiers(buf, &cmdmod, &multi_mods);
  assert(len < bufsize);
  return len;
}

// --- ex_terminal helpers ---
int nvim_docmd_p_sh_is_empty(void) { return *p_sh == NUL ? 1 : 0; }
const char *nvim_docmd_e_shellempty_str(void) { return _(e_shellempty); }
void nvim_docmd_terminal_get_shell_argv_str(char *buf, size_t buflen)
{
  char **argv = shell_build_argv(NULL, NULL);
  char **p = argv;
  char tempstring[512];
  buf[0] = NUL;
  while (*p != NULL) {
    char *escaped = vim_strsave_escaped(*p, "\"\\");
    snprintf(tempstring, sizeof(tempstring), ",\"%s\"", escaped);
    xfree(escaped);
    xstrlcat(buf, tempstring, buflen);
    p++;
  }
  shell_free_argv(argv);
}
int nvim_docmd_snprintf_terminal_suffix(char *buf, size_t buflen, const char *name)
{
  return snprintf(buf, buflen, " | call jobstart(\"%s\",{'term':v:true})", name);
}
int nvim_docmd_snprintf_terminal_shell(char *buf, size_t buflen, const char *shell_argv)
{
  return snprintf(buf, buflen, " | call jobstart([%s], {'term':v:true})", shell_argv);
}

// --- ex_restart helpers ---
void nvim_docmd_restart_patch_argv(const char *arg)
{
  const list_T *l = get_vim_var_list(VV_ARGV);
  int argc = tv_list_len(l);
  list_T *argv_cpy = tv_list_alloc(argc + 2);
  bool added_startup_arg = false;
  TV_LIST_ITER_CONST(l, li, {
    const char *item = tv_get_string(TV_LIST_ITEM_TV(li));
    size_t item_size = strlen(item);
    assert(item_size <= (size_t)SSIZE_MAX);
    tv_list_append_string(argv_cpy, item, (ssize_t)item_size);
    if (!added_startup_arg) {
      tv_list_append_string(argv_cpy, "-c", 2);
      size_t cmd_size = strlen(arg);
      assert(cmd_size <= (size_t)SSIZE_MAX);
      tv_list_append_string(argv_cpy, arg, (ssize_t)cmd_size);
      added_startup_arg = true;
    }
  });
  set_vim_var_list(VV_ARGV, argv_cpy);
}
void nvim_docmd_set_restarting(void) { restarting = true; }
int nvim_docmd_run_quit_cmd(const char *cmd)
{
  Error err = ERROR_INIT;
  nvim_command(cstr_as_string((char *)cmd), &err);
  if (ERROR_SET(&err)) {
    emsg(err.msg);
    api_clear_error(&err);
    return 0;
  }
  return 1;
}
int nvim_docmd_get_exiting(void) { return exiting ? 1 : 0; }
// Return "confirm " if CMOD_CONFIRM is set, else NULL.
const char *nvim_docmd_get_cmod_confirm_prefix(void)
{
  return (cmdmod.cmod_flags & CMOD_CONFIRM) ? "confirm " : NULL;
}

// --- ex_detach helpers ---
uint64_t nvim_docmd_get_current_ui(void) { return (uint64_t)current_ui; }
int nvim_docmd_detach_set_chan_detach(uint64_t id)
{
  Channel *chan = find_channel(id);
  if (!chan) {
    emsg(e_invchan);
    return 0;
  }
  chan->detach = true;
  return (int)chan->id;
}
int nvim_docmd_remote_ui_disconnect_checked(uint64_t id)
{
  Error err = ERROR_INIT;
  remote_ui_disconnect(id, &err, true);
  if (ERROR_SET(&err)) {
    emsg(err.msg);
    api_clear_error(&err);
    return 0;
  }
  return 1;
}
int nvim_docmd_channel_close_all(uint64_t id)
{
  const char *err = NULL;
  ILOG("detach current_ui=%" PRId64, (int64_t)id);
  bool rv = channel_close(id, kChannelPartAll, &err);
  if (!rv && err) {
    emsg(err);
    return 0;
  }
  return 1;
}

// --- ex_connect helpers ---
int nvim_docmd_ui_active_count(void) { return (int)ui_active(); }
int nvim_docmd_remote_ui_connect(uint64_t id, const char *addr)
{
  Error err = ERROR_INIT;
  remote_ui_connect(id, (char *)addr, &err);
  if (ERROR_SET(&err)) {
    emsg(err.msg);
    api_clear_error(&err);
    return 0;
  }
  return 1;
}
void nvim_docmd_set_exiting_true(void) { exiting = true; }
void nvim_docmd_getout_zero(void) { getout(0); }

// --- ex_checkhealth helpers ---
int nvim_docmd_checkhealth_exec_lua(const char *mods, size_t mlen, const char *arg,
                                    char **err_msg_out)
{
  Error err = ERROR_INIT;
  MAXSIZE_TEMP_ARRAY(args, 2);
  ADD_C(args, STRING_OBJ(((String){ .data = (char *)mods, .size = mlen })));
  ADD_C(args, CSTR_AS_OBJ((char *)arg));
  NLUA_EXEC_STATIC("vim.health._check(...)", args, kRetNilBool, NULL, &err);
  if (!ERROR_SET(&err)) {
    return 1;
  }
  *err_msg_out = xstrdup(err.msg);
  api_clear_error(&err);
  return 0;
}
const char *nvim_docmd_get_vimruntime(void) { return os_getenv_noalloc("VIMRUNTIME"); }
const char *nvim_docmd_get_p_rtp(void) { return p_rtp; }
void nvim_docmd_semsg_multiline_emsg(const char *msg)
{
  semsg_multiline("emsg", (char *)msg);
}
void nvim_docmd_xfree_str(void *p) { xfree(p); }

// --- did_set_findfunc helpers ---
int nvim_docmd_findfunc_set_global(void)
{
  return option_set_callback_func(p_ffu, &ffu_cb);
}
int nvim_docmd_findfunc_set_local(buf_T *buf)
{
  return option_set_callback_func(buf->b_p_ffu, &buf->b_ffu_cb);
}
void nvim_docmd_findfunc_free_local_cb(buf_T *buf)
{
  callback_free(&buf->b_ffu_cb);
}
char *nvim_docmd_optset_varp_deref(optset_T *args)
{
  return *(char **)args->os_varp;
}
void nvim_docmd_optset_varp_set(optset_T *args, char *name)
{
  *(char **)args->os_varp = name;
}

// nvim_xp_get_pattern and nvim_xp_get_line are defined in option_shim.c
// nvim_xp_get_pattern_len: only defined in ex_docmd.c for impl_bodies.rs
size_t nvim_xp_get_pattern_len(expand_T *xp) { return xp->xp_pattern_len; }

// =============================================================================
// Phase 4 (ex_docmd plan) accessor functions for do_one_cmd Rust port
// =============================================================================

// nvim_docmd_get_quitmore already defined above.
/// Decrement quitmore.
void nvim_docmd_dec_quitmore(void) { quitmore--; }
// nvim_docmd_get_exiting already defined above (returns int 0/1).
/// Get ex_nesting_level global.
int nvim_docmd_get_ex_nesting_level(void) { return ex_nesting_level; }
/// Increment ex_nesting_level.
void nvim_docmd_inc_ex_nesting_level(void) { ex_nesting_level++; }
/// Decrement ex_nesting_level.
void nvim_docmd_dec_ex_nesting_level(void) { ex_nesting_level--; }
/// Get need_rethrow global.
bool nvim_docmd_get_need_rethrow(void) { return need_rethrow; }
/// Set need_rethrow global.
void nvim_docmd_set_need_rethrow(bool val) { need_rethrow = val; }
/// Get check_cstack global.
bool nvim_docmd_get_check_cstack(void) { return check_cstack; }
/// Set check_cstack global.
void nvim_docmd_set_check_cstack(bool val) { check_cstack = val; }
/// Set did_emsg_syntax to true.
void nvim_docmd_set_did_emsg_syntax(void) { did_emsg_syntax = true; }
/// Allocate a zeroed exarg_T on the heap (line1=1, line2=1).
exarg_T *nvim_eap_alloc(void)
{
  exarg_T *eap = xcalloc(1, sizeof(exarg_T));
  eap->line1 = 1;
  eap->line2 = 1;
  return eap;
}
/// Free a heap-allocated exarg_T (does NOT free cmdline_tofree -- caller must).
void nvim_eap_free(exarg_T *eap) { xfree(eap); }
/// Set eap->skip.
void nvim_eap_set_skip(exarg_T *eap, bool val) { eap->skip = val; }
/// Set eap->ea_getline.
void nvim_eap_set_ea_getline(exarg_T *eap, LineGetter fn_ptr) { eap->ea_getline = fn_ptr; }
/// Get eap->ea_getline.
LineGetter nvim_eap_get_ea_getline(const exarg_T *eap) { return eap->ea_getline; }
/// Set eap->cookie.
void nvim_eap_set_cookie(exarg_T *eap, void *cookie) { eap->cookie = cookie; }
/// Set eap->cmdlinep.
void nvim_eap_set_cmdlinep(exarg_T *eap, char **cmdlinep) { eap->cmdlinep = cmdlinep; }
/// do_errthrow wrapper for do_one_cmd (passes cmd_name from cmdnames or NULL).
void nvim_docmd_do_errthrow(cstack_T *cstack, const exarg_T *eap)
{
  const char *cmd_name = NULL;
  if (!IS_USER_CMDIDX(eap->cmdidx) && eap->cmdidx != CMD_SIZE) {
    cmd_name = cmdnames[(int)eap->cmdidx].cmd_name;
  }
  do_errthrow(cstack, (char *)cmd_name);
}
/// do_throw wrapper.
void nvim_docmd_do_throw(cstack_T *cstack) { do_throw(cstack); }
/// do_return wrapper.
void nvim_docmd_do_return(exarg_T *eap) { do_return(eap, true, false, NULL); }
/// do_finish wrapper.
void nvim_docmd_do_finish(exarg_T *eap) { do_finish(eap, true); }
/// source_finished wrapper (for do_one_cmd).
bool nvim_docmd_source_finished(LineGetter fgetline, void *cookie)
{
  return source_finished(fgetline, cookie);
}
/// getline_equal(fgetline, cookie, getnextac) wrapper.
bool nvim_getline_equal_getnextac(LineGetter fgetline, void *cookie)
{
  return getline_equal(fgetline, cookie, getnextac);
}
// nvim_docmd_cmdnames_name already defined at line 2217 (returns char*).
/// Set eap->cmd to *cmdlinep (used to init ea.cmd = *cmdlinep in do_one_cmd).
void nvim_eap_set_cmd_from_cmdlinep(exarg_T *eap) { eap->cmd = *eap->cmdlinep; }
/// Get *eap->cmdlinep[0] first char (for "#!" check in do_one_cmd).
char nvim_eap_cmdlinep_first_char(const exarg_T *eap) { return (*eap->cmdlinep)[0]; }
/// Get *eap->cmdlinep[1] second char (for "#!" check in do_one_cmd).
char nvim_eap_cmdlinep_second_char(const exarg_T *eap) { return (*eap->cmdlinep)[1]; }
/// Save the global cmdmod into a heap-allocated copy (returns pointer).
/// Caller must free with nvim_docmd_restore_cmdmod_save.
cmdmod_T *nvim_docmd_save_cmdmod(void)
{
  cmdmod_T *save = xmalloc(sizeof(cmdmod_T));
  *save = cmdmod;
  return save;
}
/// Restore global cmdmod from save and free the save buffer.
void nvim_docmd_restore_cmdmod(cmdmod_T *save)
{
  cmdmod = *save;
  xfree(save);
}
/// do_intthrow wrapper.
bool nvim_docmd_do_intthrow(cstack_T *cstack) { return do_intthrow(cstack); }
/// get_func_line function pointer value (for getline_equal comparison).
/// Returns the function pointer as a void* for comparison.
void *nvim_docmd_get_func_line_ptr(void) { return (void *)get_func_line; }
/// getargcmd wrapper (already public, but needs C accessor for Rust FFI).
char *nvim_docmd_getargcmd(char **argp) { return getargcmd(argp); }
/// Set eap->do_ecmd_cmd via getargcmd.
void nvim_eap_set_do_ecmd_cmd_from_arg(exarg_T *eap)
{
  eap->do_ecmd_cmd = getargcmd(&eap->arg);
}
/// eap->usefilter setter.
void nvim_eap_set_usefilter(exarg_T *eap, bool val) { eap->usefilter = val; }
/// eap->append setter.
void nvim_eap_set_append(exarg_T *eap, bool val) { eap->append = val; }
/// Get first char of eap->arg.
char nvim_eap_arg_first_char(const exarg_T *eap) { return eap->arg[0]; }
/// Newline-scan for shell cmd args in do_one_cmd (CMD_bang/terminal/global/vglobal/usefilter).
/// Sets eap->nextcmd and NUL-terminates at newline, handling backslash-newline.
void nvim_eap_scan_newline_nextcmd(exarg_T *eap)
{
  for (char *s = eap->arg; *s; s++) {
    if (*s == '\\' && s[1] == '\n') {
      STRMOVE(s, s + 1);
    } else if (*s == '\n') {
      eap->nextcmd = s + 1;
      *s = NUL;
      break;
    }
  }
}
/// CMD_bang cmdidx constant.
int nvim_docmd_CMD_bang(void) { return (int)CMD_bang; }
/// CMD_terminal cmdidx constant.
int nvim_docmd_CMD_terminal(void) { return (int)CMD_terminal; }
/// CMD_global cmdidx constant.
int nvim_docmd_CMD_global(void) { return (int)CMD_global; }
/// CMD_vglobal cmdidx constant.
int nvim_docmd_CMD_vglobal(void) { return (int)CMD_vglobal; }
/// CMD_write cmdidx constant.
int nvim_docmd_CMD_write(void) { return (int)CMD_write; }
/// CMD_update cmdidx constant.
int nvim_docmd_CMD_update(void) { return (int)CMD_update; }
/// CMD_read cmdidx constant.
int nvim_docmd_CMD_read(void) { return (int)CMD_read; }
/// CMD_lshift cmdidx constant.
int nvim_docmd_CMD_lshift(void) { return (int)CMD_lshift; }
/// CMD_rshift cmdidx constant.
int nvim_docmd_CMD_rshift(void) { return (int)CMD_rshift; }
/// CMD_file cmdidx constant.
int nvim_docmd_CMD_file(void) { return (int)CMD_file; }
/// eap->amount setter.
void nvim_eap_set_amount(exarg_T *eap, int val) { eap->amount = val; }
/// Increment eap->amount.
void nvim_eap_inc_amount(exarg_T *eap) { eap->amount++; }
/// Return eap->cmd[0] for amount counting in lshift/rshift.
char nvim_eap_cmd_first_char(const exarg_T *eap) { return *eap->cmd; }
/// Return eap->arg[0] for lshift/rshift loop.
char nvim_eap_arg_at(const exarg_T *eap, int i) { return eap->arg[i]; }
/// Advance eap->arg by 1 (for filter arg).
void nvim_eap_advance_arg(exarg_T *eap) { eap->arg++; }
/// Advance eap->arg by 2 (for >> arg).
void nvim_eap_advance_arg2(exarg_T *eap) { eap->arg += 2; }
/// skipwhite(eap->arg) -> eap->arg.
void nvim_eap_skipwhite_arg(exarg_T *eap) { eap->arg = skipwhite(eap->arg); }
/// getargopt wrapper (already public Rust export, add C wrapper for do_one_cmd).

/// get_flags wrapper (already public Rust export).

/// skip_cmd wrapper (already public Rust export).


/// Emit error and do_errthrow cleanup for do_one_cmd doend.
void nvim_docmd_do_one_cmd_doend(cstack_T *cstack, const char *errormsg,
                                  int flags, const exarg_T *eap)
{
  if (errormsg != NULL && *errormsg != NUL && !did_emsg) {
    if (flags & DOCMD_VERBOSE) {
      if (errormsg != IObuff) {
        xstrlcpy(IObuff, errormsg, IOSIZE);
        errormsg = IObuff;
      }
      append_command(*eap->cmdlinep);
    }
    emsg(errormsg);
  }
  const char *cmd_name = NULL;
  if (!IS_USER_CMDIDX(eap->cmdidx) && eap->cmdidx != CMD_SIZE) {
    cmd_name = cmdnames[(int)eap->cmdidx].cmd_name;
  }
  do_errthrow(cstack, (char *)cmd_name);
}
/// xfree wrapper for eap->cmdline_tofree in do_one_cmd cleanup.
void nvim_docmd_xfree(void *p) { xfree(p); }
/// get e_sandbox error string.
const char *nvim_docmd_get_e_sandbox(void) { return _(e_sandbox); }
/// has_event wrapper.
bool nvim_docmd_has_event(int event) { return has_event((event_T)event); }
/// apply_autocmds wrapper (for CmdUndefined event in do_one_cmd).
bool nvim_docmd_apply_autocmds_cmdundefined(const char *cmdname)
{
  return apply_autocmds(EVENT_CMDUNDEFINED, (char *)cmdname, (char *)cmdname, true, NULL);
}
/// xmemdupz wrapper for do_one_cmd cmdname copy.
char *nvim_docmd_xmemdupz(const char *s, size_t len) { return xmemdupz(s, len); }
/// ASCII_ISALNUM check for command name scanning.
bool nvim_docmd_ascii_isalnum(char c) { return ASCII_ISALNUM(c); }
/// IS_USER_CMDIDX check by integer index (as opposed to via exarg_T).
bool nvim_docmd_is_user_cmdidx_i(int cmdidx) { return IS_USER_CMDIDX(cmdidx); }
/// global_busy accessor.
bool nvim_docmd_global_busy(void) { return global_busy != 0; }
/// msg_silent accessor.
int nvim_docmd_msg_silent(void) { return msg_silent; }
/// exmode_active accessor.
bool nvim_docmd_exmode_active(void) { return exmode_active; }
/// ask_yesno for backwards range check.
char nvim_docmd_ask_yesno_backwards(void)
{
  return (char)ask_yesno(_("Backwards range given, OK to swap"));
}
/// invalid_range wrapper (already public Rust export).
const char *nvim_docmd_invalid_range(exarg_T *eap) { return (const char *)invalid_range(eap); }
/// eap->skip getter (return as bool).
bool nvim_eap_get_skip_bool(const exarg_T *eap) { return eap->skip; }
/// EX_ARGOPT bit check helper.
bool nvim_eap_argt_has_argopt(const exarg_T *eap) { return (eap->argt & EX_ARGOPT) != 0; }
/// EX_CMDARG bit check helper.
bool nvim_eap_argt_has_cmdarg(const exarg_T *eap) { return (eap->argt & EX_CMDARG) != 0; }
/// EX_EXTRA bit check helper.
bool nvim_eap_argt_has_extra(const exarg_T *eap) { return (eap->argt & EX_EXTRA) != 0; }
/// EX_NEEDARG bit check helper.
bool nvim_eap_argt_has_needarg(const exarg_T *eap) { return (eap->argt & EX_NEEDARG) != 0; }
/// EX_MODIFY bit check helper.
bool nvim_eap_argt_has_modify(const exarg_T *eap) { return (eap->argt & EX_MODIFY) != 0; }
/// EX_SBOXOK bit check helper.
bool nvim_eap_argt_has_sboxok(const exarg_T *eap) { return (eap->argt & EX_SBOXOK) != 0; }
/// EX_FLAGS bit check helper.
bool nvim_eap_argt_has_flags(const exarg_T *eap) { return (eap->argt & EX_FLAGS) != 0; }
/// EX_RANGE bit check helper (already have nvim_eap_argt_has_range, but using explicit helper).
bool nvim_eap_argt_has_range_bit(const exarg_T *eap) { return (eap->argt & EX_RANGE) != 0; }
/// EX_BANG bit check helper (already have nvim_eap_argt_has_bang).
bool nvim_eap_argt_has_bang_bit(const exarg_T *eap) { return (eap->argt & EX_BANG) != 0; }
/// ADDR_OTHER constant.
int nvim_docmd_ADDR_OTHER(void) { return (int)ADDR_OTHER; }
/// Get eap->forceit as int (0/1).
int nvim_eap_get_forceit_int(const exarg_T *eap) { return (int)eap->forceit; }
/// Wrap nvim_curbuf_modifiable (MODIFIABLE macro check).
bool nvim_docmd_curbuf_modifiable(void) { return MODIFIABLE(curbuf) != 0; }
/// curbuf->terminal check (already have nvim_curbuf_is_terminal).
/// e_trailing_arg getter (already used elsewhere).
const char *nvim_docmd_get_e_trailing_arg(void) { return e_trailing_arg; }
/// e_argreq getter.
const char *nvim_docmd_get_e_argreq(void) { return _(e_argreq); }
/// e_invarg getter.
const char *nvim_docmd_get_e_invarg(void) { return _(e_invarg); }
// nvim_docmd_get_e_norange already defined above (returns char* version).
/// e_nobang getter (already have nvim_get_e_nobang).
/// nvim_get_reg_executing (already defined in autocmd.c).
/// nvim_get_pending_end_reg_executing (already defined in getchar.c).
/// nvim_set_reg_executing (already defined in getchar.c).
/// nvim_set_pending_end_reg_executing (already defined in getchar.c).
/// ex_errmsg for e_trailing_arg.
char *nvim_docmd_ex_errmsg_trailing(const char *arg)
{
  return ex_errmsg(e_trailing_arg, arg);
}
/// Parse EX_ARGOPT loop in do_one_cmd (handles ++opt=val argument).
/// Returns FAIL if getargopt fails and !ni, OK otherwise.
int nvim_docmd_parse_argopt(exarg_T *eap, bool ni)
{
  while (eap->arg[0] == '+' && eap->arg[1] == '+') {
    if (getargopt(eap) == FAIL && !ni) {
      return FAIL;
    }
  }
  return OK;
}

// =============================================================================
// Phase 4 additional accessors for do_one_cmd.rs
// =============================================================================

/// CMD_put constant.
int nvim_docmd_CMD_put(void) { return (int)CMD_put; }
/// CMD_iput constant.
int nvim_docmd_CMD_iput(void) { return (int)CMD_iput; }
/// CMD_checktime constant.
int nvim_docmd_CMD_checktime(void) { return (int)CMD_checktime; }
/// CMD_edit constant.
int nvim_docmd_CMD_edit(void) { return (int)CMD_edit; }
/// "Backwards range given" error string.
const char *nvim_docmd_get_e_backwards_range(void)
{
  return _("E493: Backwards range given");
}
/// "E494: Use w or w>>" error string.
const char *nvim_docmd_get_e_w_usage(void) { return _("E494: Use w or w>>"); }
/// EX_WHOLEFOLD check on eap->argt.
bool nvim_eap_argt_has_wholefold(const exarg_T *eap)
{
  return (eap->argt & EX_WHOLEFOLD) != 0;
}
/// EVENT_CMDUNDEFINED constant.
int nvim_docmd_get_event_cmdundefined(void) { return (int)EVENT_CMDUNDEFINED; }
/// Fix cursor lnum if it is 0 (can happen with zero line number).
void nvim_docmd_fix_cursor_if_zero(void)
{
  if (curwin->w_cursor.lnum == 0) {
    curwin->w_cursor.lnum = 1;
    curwin->w_cursor.col = 0;
  }
}
/// Format an error message with arg into buf.
/// Wraps vim_snprintf(buf, buflen, _(msg), arg).
char *nvim_docmd_ex_errmsg_format(const char *msg, const char *arg,
                                   char *buf, size_t buflen)
{
  vim_snprintf(buf, buflen, _(msg), arg);
  return buf;
}
/// Get argt for cmdidx from cmdnames[] (used by rs_excmd_get_argt and do_one_cmd).
uint32_t nvim_docmd_get_argt_for_idx(int idx)
{
  return cmdnames[idx].cmd_argt;
}
/// parse_command_modifiers with global cmdmod (for do_one_cmd).
int nvim_docmd_parse_command_modifiers_global(exarg_T *eap, const char **errormsg)
{
  return parse_command_modifiers(eap, errormsg, &cmdmod, false);
}

// get_loop_line function pointer value for comparison (used by getline_equal/getline_cookie in Rust).
LineGetter nvim_docmd_get_loop_line_ptr(void) { return get_loop_line; }

/// Get lc_getline from loop_cookie (used by Rust getline_equal/getline_cookie).
LineGetter nvim_docmd_loop_cookie_get_lc_getline(void *lc)
{
  return ((struct loop_cookie *)lc)->lc_getline;
}

/// Get cookie from loop_cookie (used by Rust getline_equal/getline_cookie).
void *nvim_docmd_loop_cookie_get_cookie(void *lc)
{
  return ((struct loop_cookie *)lc)->cookie;
}
