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
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/ui.h"
#include "nvim/api/vimscript.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/channel.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cursor.h"
#include "nvim/debugger.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/fs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_group.h"
#include "nvim/input.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
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
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/state.h"
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


// Rust implementations - declarations
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern int rs_get_copyID(void);
extern void rs_do_tag(char *tag, int type, int count, int forceit, bool verbose);
extern void rs_listdigraphs(int use_headers);
// Forward declarations for Rust-exported do_one_cmd helpers called within this file.
// (Not in ex_docmd.h.generated.h since these are Rust exports, not C definitions.)
void shift_cmd_args(exarg_T *eap);
char *skip_colon_white(const char *p, bool skipleadingwhite);
bool parse_bang(const exarg_T *eap, char **p);
void parse_register(exarg_T *eap);
void append_command(const char *cmd);
void msg_verbose_cmd(linenr_T lnum, char *cmd);
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
static const char e_not_an_editor_command[]
  = N_("E492: Not an editor command");

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

// restore_dbg_stuff, get_loop_line, store_loop_line are all now in Rust
// (do_cmdline.rs). struct loop_cookie and wcmd_T remain here to support
// nvim_docmd_ga_deep_clear_lines used by Rust.
extern char *get_loop_line(int c, void *cookie, int indent, bool do_concat);

#include "ex_docmd.c.generated.h"
extern int rs_win_valid(win_T *win);
extern int rs_only_one_window(void);
extern int rs_valid_tabpage(tabpage_T *tpc);

// Rust fold FFI declarations
extern int rs_foldManualAllowed(bool create);
extern void rs_foldCreate(win_T *wp, linenr_T start_lnum, linenr_T end_lnum);
extern void rs_opFoldRange(linenr_T first_lnum, linenr_T last_lnum, int opening, int recurse, bool had_visual);

extern int rs_get_scrolloff_value(win_T *wp);

// ex_* Rust exports: declared in ex_docmd.h (included above).

// Declare cmdnames[].
#include "ex_cmds_defs.generated.h"

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

// Returns 0 if cmdidx is out of bounds
int cmdname_first_char(int cmdidx) { return (cmdidx < 0 || cmdidx >= CMD_SIZE) ? 0 : (unsigned char)cmdnames[cmdidx].cmd_name[0]; }

static char dollar_command[2] = { '$', 0 };

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

extern char *nvim_docmd_get_bad_name(expand_T *xp, int idx);

/// callback function for 'findfunc'
static Callback ffu_cb;

/// Return a pointer to the global ffu_cb (for Rust FFI).
Callback *nvim_docmd_get_ffu_cb_ptr(void) { return &ffu_cb; }
static Callback *get_findfunc_callback(void) { return *curbuf->b_p_ffu != NUL ? &curbuf->b_ffu_cb : &ffu_cb; }
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

int nvim_docmd_tv_list_len(const list_T *l) { return tv_list_len(l); }
const char *nvim_docmd_e_cant_find_file_str_in_path(void) { return _(e_cant_find_file_str_in_path); }
const char *nvim_docmd_e_no_more_file_str_found_in_path(void) { return _(e_no_more_file_str_found_in_path); }
extern int expand_findfunc(char *pat, char ***files, int *numMatches);

extern char *nvim_docmd_findfunc_find_file(char *arg, size_t len, int count);

extern void nvim_docmd_free_findfunc_option_impl(void);

extern bool nvim_docmd_set_ref_in_findfunc_impl(int copyID);

static char *prev_dir = NULL;

#if defined(EXITFREE)
void free_cd_dir(void)
{
  XFREE_CLEAR(prev_dir);
  XFREE_CLEAR(globaldir);
}

#endif

extern void nvim_docmd_post_chdir_impl(CdScope scope, bool trigger_dirchanged);

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

extern char *eval_vars(char *src, const char *srcstart, size_t *usedlen,
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

int nvim_get_ex_pressedreturn(void) { return ex_pressedreturn ? 1 : 0; }
int nvim_get_expr_map_lock(void) { return expr_map_lock; }
int nvim_curbuf_is_dummy(void) { return (curbuf->b_flags & BF_DUMMY) != 0; }
// C accessors for Rust sourcing info access
const char *nvim_get_sourcing_name(void) { return (exestack.ga_data == NULL || exestack.ga_len == 0) ? NULL : SOURCING_NAME; }
int nvim_get_sourcing_lnum(void) { return (exestack.ga_data == NULL || exestack.ga_len == 0) ? 0 : (int)SOURCING_LNUM; }
int nvim_get_exestack_len(void) { return exestack.ga_len; }
const char *nvim_docmd_get_curbuf_swapname(void) { return (curbuf->b_ml.ml_mfp == NULL || curbuf->b_ml.ml_mfp->mf_fname == NULL) ? NULL : curbuf->b_ml.ml_mfp->mf_fname; }
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

void nvim_docmd_loop_sleep(int64_t msec) { LOOP_PROCESS_EVENTS_UNTIL(&main_loop, loop_get_events(&main_loop), msec, got_int); }
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

int nvim_docmd_cmdnames_func_is_ni(int cmdidx) { return IS_USER_CMDIDX((cmdidx_T)cmdidx) ? 0 : (cmdnames[cmdidx].cmd_func == ex_ni || cmdnames[cmdidx].cmd_func == ex_script_ni); }
linenr_T nvim_docmd_get_curbuf_line_count(void) { return curbuf->b_ml.ml_line_count; }
int nvim_docmd_get_command_count(void) { return command_count; }
int nvim_docmd_get_cmdidxs1(int c) { return (int)cmdidxs1[CHAR_ORD_LOW(c)]; }
int nvim_docmd_get_cmdidxs2(int c1, int c2) { return (int)cmdidxs2[CHAR_ORD_LOW(c1)][CHAR_ORD_LOW(c2)]; }
/// Check if cmdnames[idx] name prefix-matches cmd for len chars.
int nvim_docmd_cmdnames_prefix_match(int idx, const char *cmd, int len)
{
  return strncmp(cmdnames[idx].cmd_name, cmd, (size_t)len) == 0;
}
int nvim_docmd_cmdnames_name_complete(int idx, int len) { return cmdnames[idx].cmd_name[len] == NUL; }
char *nvim_docmd_cmdnames_name(int idx) { return cmdnames[idx].cmd_name; }
/// Report E943 and exit (command table mismatch).
void nvim_docmd_e943_abort(void)
{
  iemsg(_("E943: Command table needs to be updated, run 'make'"));
  getout(1);
}

char *nvim_docmd_tv_get_string(const void *argvars) { return (char *)tv_get_string((const typval_T *)argvars); }
void nvim_docmd_rettv_init_string(void *rettv) { typval_T *tv = (typval_T *)rettv; tv->v_type = VAR_STRING; tv->vval.v_string = NULL; }
void nvim_docmd_rettv_set_string(void *rettv, const char *s) { ((typval_T *)rettv)->vval.v_string = xstrdup(s); }
char *nvim_docmd_get_user_command_name(int useridx, int cmdidx) { return get_user_command_name(useridx, (cmdidx_T)cmdidx); }
char *nvim_docmd_get_dollar_command(void) { return dollar_command; }
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

void nvim_docmd_arg_skip_to_end(exarg_T *eap) { eap->arg += strlen(eap->arg); }
int nvim_docmd_count_buf_check(exarg_T *eap) { char *p = skipdigits(eap->arg + 1); return *p == NUL || ascii_iswhite(*p); }
int nvim_docmd_cmdnames_addr_type(int idx) { return (int)cmdnames[idx].cmd_addr_type; }
int nvim_docmd_current_win_nr(void) { return CURRENT_WIN_NR; }
int nvim_docmd_last_win_nr(void) { return LAST_WIN_NR; }
int nvim_docmd_current_tab_nr(void) { return CURRENT_TAB_NR; }
int nvim_docmd_last_tab_nr(void) { return LAST_TAB_NR; }
linenr_T nvim_docmd_get_curwin_cursor_lnum(void) { return curwin->w_cursor.lnum; }
int nvim_docmd_get_curwin_arg_idx(void) { return curwin->w_arg_idx; }
int nvim_docmd_get_argcount(void) { return ARGCOUNT; }
int nvim_docmd_get_curbuf_fnum(void) { return curbuf->b_fnum; }
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

int nvim_docmd_firstbuf_fnum(void) { return firstbuf->b_fnum; }
int nvim_docmd_lastbuf_fnum(void) { return lastbuf->b_fnum; }
void nvim_docmd_iemsg_dflall(void) { iemsg(_("INTERNAL: Cannot use EX_DFLALL with ADDR_NONE, ADDR_UNSIGNED or ADDR_QUICKFIX")); }

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

int nvim_docmd_getdigits(char **pp, int def) { return (int)getdigits(pp, false, def); }
char *nvim_docmd_ex_errmsg_invargval(const char *arg) { return (char *)ex_errmsg(e_invargval, arg); }
char *nvim_docmd_ex_errmsg_invarg2(const char *arg) { return (char *)ex_errmsg(e_invarg2, arg); }
void nvim_cmod_capture_msg_scroll(cmdmod_T *cmod) { cmod->cmod_save_msg_scroll = msg_scroll; }
void nvim_cmod_regfree_filter(cmdmod_T *cmod) { vim_regfree(cmod->cmod_filter_regmatch.regprog); cmod->cmod_filter_regmatch.regprog = NULL; }
void nvim_docmd_restore_msg_scroll(cmdmod_T *cmod) { msg_scroll = cmod->cmod_save_msg_scroll; }
void nvim_docmd_set_eventignore_all(void) { set_option_direct(kOptEventignore, STATIC_CSTR_AS_OPTVAL("all"), 0, SID_NONE); }
void nvim_docmd_set_eventignore_str(char *s) { set_option_direct(kOptEventignore, CSTR_AS_OPTVAL(s), 0, SID_NONE); }
int nvim_docmd_getline_is_getexline(const exarg_T *eap) { return getline_equal(eap->ea_getline, eap->cookie, getexline); }
char *nvim_docmd_get_exmode_plus(void) { return exmode_plus; }
void nvim_docmd_set_curwin_cursor_lnum(linenr_T lnum) { curwin->w_cursor.lnum = lnum; }
void nvim_docmd_set_curwin_cursor_col(colnr_T col) { curwin->w_cursor.col = col; }
colnr_T nvim_docmd_get_curwin_cursor_col(void) { return curwin->w_cursor.col; }
int nvim_docmd_get_searchcmdlen(void) { return (int)searchcmdlen; }
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

/// Returns opaque fmark_T pointer (NULL on failure).
void *nvim_docmd_mark_get(int flag, int ch) { return mark_get(curbuf, curwin, NULL, (MarkGet)flag, (uint8_t)ch); }
/// Get fmark_T->fnum.
int nvim_docmd_mark_fnum(const void *fm) { return ((const fmark_T *)fm)->fnum; }
/// Get fmark_T->mark.lnum.
linenr_T nvim_docmd_mark_lnum(const void *fm) { return ((const fmark_T *)fm)->mark.lnum; }

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
/// Wrap qf_get_size for Rust.
/// Wrap mark_get_visual for Rust.
void *nvim_docmd_mark_get_visual(int ch) { return mark_get_visual(curbuf, (uint8_t)ch); }
// (commands.rs: verify_command, skip_cmd, ex_redir, ex_normal, ex_filetype,
void *nvim_docmd_get_redir_fd(void) { return redir_fd; }
void nvim_docmd_set_redir_fd(void *fd) { redir_fd = (FILE *)fd; }
void nvim_docmd_set_redir_reg(int reg) { redir_reg = (uint8_t)reg; }
void nvim_docmd_set_redir_vname(int val) { redir_vname = (bool)val; }
int nvim_docmd_get_redir_vname(void) { return redir_vname ? 1 : 0; }
void nvim_docmd_fclose_redir_fd(void) { fclose(redir_fd); redir_fd = NULL; }
int nvim_docmd_get_p_mmd(void) { return (int)p_mmd; }
int nvim_docmd_curbuf_has_terminal(void) { return curbuf->terminal != NULL ? 1 : 0; }
int nvim_docmd_curwin_in_terminal_mode(void) { return (State & MODE_TERMINAL) ? 1 : 0; }
int nvim_docmd_one_window_p(int addr_count) { return (ONE_WINDOW || addr_count == 0) ? 1 : 0; }
int nvim_docmd_get_filetype_detect(void) { return (int)filetype_detect; }
void nvim_docmd_set_filetype_detect(int val) { filetype_detect = (TriState)val; }
int nvim_docmd_get_filetype_plugin(void) { return (int)filetype_plugin; }
void nvim_docmd_set_filetype_plugin(int val) { filetype_plugin = (TriState)val; }
int nvim_docmd_get_filetype_indent(void) { return (int)filetype_indent; }
void nvim_docmd_set_filetype_indent(int val) { filetype_indent = (TriState)val; }
int nvim_docmd_curbuf_file_id_valid(void) { return curbuf->file_id_valid ? 1 : 0; }
const char *nvim_docmd_get_curbuf_sfname(void) { return curbuf->b_sfname; }
void nvim_docmd_do_cmdline_getexline_noflags(void) { do_cmdline(NULL, getexline, NULL, 0); }
int64_t nvim_docmd_curbuf_changedtick(void) { return (int64_t)buf_get_changedtick(curbuf); }
void nvim_docmd_msg_scroll_flush(void) { msg_scroll_flush(); }
void nvim_cmd_dispatch(exarg_T *eap) { (cmdnames[eap->cmdidx].cmd_func)(eap); }
int nvim_cmd_preview_dispatch(exarg_T *eap, int ns, int bufnr)
{
  return (cmdnames[eap->cmdidx].cmd_preview_func)(eap, ns, bufnr);
}
int nvim_cmdmod_get_did_esilent(void) { return cmdmod.cmod_did_esilent; }
void nvim_cmdmod_set_did_esilent(int val) { cmdmod.cmod_did_esilent = val; }
void nvim_cmdmod_load_from_cmdinfo(const CmdParseInfo *cmdinfo) { cmdmod = cmdinfo->cmdmod; }
void nvim_cmdmod_store_to_save(cmdmod_T *save) { *save = cmdmod; }
void nvim_cmdmod_restore_from_save(const cmdmod_T *save) { cmdmod = *save; }
size_t nvim_sizeof_cmdmod_T(void) { return sizeof(cmdmod_T); }

// execute_cmd helpers
cstack_T *nvim_cstack_alloc(void) { cstack_T *cs = xcalloc(1, sizeof(cstack_T)); cs->cs_idx = -1; return cs; }
int nvim_curbuf_is_terminal(void) { return curbuf->terminal != NULL ? 1 : 0; }
int nvim_get_eap_addr_type_lines(const exarg_T *eap) { return eap->addr_type == ADDR_LINES ? 1 : 0; }
void nvim_hasFolding_line1(linenr_T lnum, linenr_T *line1_out) { hasFolding(curwin, lnum, line1_out, NULL); }
void nvim_hasFolding_line2(linenr_T lnum, linenr_T *line2_out) { hasFolding(curwin, lnum, NULL, line2_out); }
int nvim_cstack_get_idx(const cstack_T *cs) { return cs->cs_idx; }
int nvim_cstack_get_flags(const cstack_T *cs, int idx) { return cs->cs_flags[idx]; }

// profile_cmd helpers
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
char *nvim_skip_colon_white(const char *p, bool skipleadingwhite) { return skip_colon_white(p, skipleadingwhite); }
void nvim_set_eap_arg_from_p(exarg_T *eap, char *p) { eap->arg = (eap->cmdidx == CMD_bang) ? p : skipwhite(p); }
void nvim_skip_expr_arg(char **arg) { skip_expr(arg, NULL); }
void nvim_clear_cmdinfo(CmdParseInfo *cmdinfo) { CLEAR_POINTER(cmdinfo); }
size_t nvim_iosize(void) { return IOSIZE; }
void nvim_xstrlcpy(char *dst, const char *src, size_t n) { xstrlcpy(dst, src, n); }
void nvim_docmd_goto_buffer_current(exarg_T *eap) { goto_buffer(eap, DOBUF_CURRENT, FORWARD, 0); }
void nvim_docmd_goto_buffer_first(exarg_T *eap, int n) { goto_buffer(eap, DOBUF_FIRST, FORWARD, n); }
char *nvim_docmd_eap_get_do_ecmd_cmd(const exarg_T *eap) { return eap->do_ecmd_cmd; }
char *nvim_docmd_errmsg_trailing_arg(const char *arg) { return ex_errmsg(e_trailing_arg, arg); }
bool nvim_docmd_get_findfunc_nonempty(void) { return *get_findfunc() != NUL; }
const char *nvim_docmd_curbuf_b_ffname(void) { return curbuf->b_ffname; }
void nvim_docmd_free_sourcing_name_and_pop(void) { xfree(SOURCING_NAME); estack_pop(); }
char *nvim_docmd_tab_page_fmt(int n) { vim_snprintf(IObuff, IOSIZE, _("Tab page %d"), n); return IObuff; }
/// Call msg_outtrans with an attribute (e.g. HLF_T).
void nvim_docmd_msg_outtrans_attr(const char *s, int attr) { msg_outtrans((char *)s, attr, false); }
/// home_replace into IObuff.
void nvim_docmd_home_replace(buf_T *buf, const char *src) { home_replace(buf, src, IObuff, IOSIZE, true); }
int nvim_ascii_iswhite_fn(int c) { return ascii_iswhite(c) ? 1 : 0; }
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

int nvim_os_dirname_namebuff(void) { return (int)os_dirname(NameBuff, MAXPATHL); }
void nvim_expand_env_home_namebuff(void) { expand_env("$HOME", NameBuff, MAXPATHL); }
int nvim_get_p_cdh(void) { return p_cdh ? 1 : 0; }
int nvim_vim_chdir(const char *dir) { return vim_chdir(dir); }
void nvim_do_autocmd_dirchanged_manual_pre(const char *new_dir, int scope) { do_autocmd_dirchanged(new_dir, (CdScope)scope, kCdCauseManual, true); }
void nvim_post_chdir(int scope, bool dir_differs) { nvim_docmd_post_chdir_impl((CdScope)scope, dir_differs); }
char *nvim_docmd_get_do_ecmd_cmd_dollar(void) { return dollar_command; }

// eval_vars wrapper that returns the result and updates src/escaped via out-params.
// Returns NULL if no match, or the replacement string (caller must free).
char *nvim_eval_vars_wrap(exarg_T *eap, char *p, size_t *srclenp, const char **errormsgp,
                          int *escapedp)
{
  int escaped = 0;
  char *repl = eval_vars(p, eap->arg, srclenp, &eap->do_ecmd_lnum, errormsgp, &escaped, true);
  *escapedp = escaped;
  return repl;
}

int nvim_get_p_wic(void) { return p_wic ? 1 : 0; }
void nvim_backslash_halve(char *p) { backslash_halve(p); }
void nvim_expand_env_esc_namebuff_notilde(const char *str) { expand_env_esc(str, NameBuff, MAXPATHL, false, true, NULL); }
size_t nvim_ExpandT_size(void) { return sizeof(expand_T); }
void nvim_ExpandInit(expand_T *xpc) { ExpandInit(xpc); }
char *nvim_ExpandOne_files(expand_T *xpc, const char *str, int wildflags, bool icase) { return ExpandOne(xpc, str, NULL, icase ? wildflags + WILD_ICASE : wildflags, WILD_EXPAND_FREE); }
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
bool nvim_has_dollar_or_tilde(const char *s) { return vim_strchr(s, '$') != NULL || vim_strchr(s, '~') != NULL; }
bool nvim_is_expand_char(int c) { return vim_strchr("%#<", (uint8_t)c) != NULL; }
/// Set eap->errmsg from a const string (for Rust FFI const safety).

void nvim_docmd_set_no_hlsearch(bool flag) { no_hlsearch = flag; set_vim_var_nr(VV_HLSEARCH, !no_hlsearch && p_hls); }

void nvim_docmd_goto_buffer_mod(exarg_T *eap) { goto_buffer(eap, DOBUF_MOD, FORWARD, (int)eap->line2); if (eap->do_ecmd_cmd != NULL) { do_cmdline_cmd(eap->do_ecmd_cmd); } }
void nvim_docmd_goto_buffer_next(exarg_T *eap) { goto_buffer(eap, DOBUF_CURRENT, FORWARD, (int)eap->line2); if (eap->do_ecmd_cmd != NULL) { do_cmdline_cmd(eap->do_ecmd_cmd); } }
void nvim_docmd_goto_buffer_prev(exarg_T *eap) { goto_buffer(eap, DOBUF_CURRENT, BACKWARD, (int)eap->line2); if (eap->do_ecmd_cmd != NULL) { do_cmdline_cmd(eap->do_ecmd_cmd); } }
void nvim_docmd_goto_buffer_rewind(exarg_T *eap) { goto_buffer(eap, DOBUF_FIRST, FORWARD, 0); if (eap->do_ecmd_cmd != NULL) { do_cmdline_cmd(eap->do_ecmd_cmd); } }
void nvim_docmd_goto_buffer_last(exarg_T *eap) { goto_buffer(eap, DOBUF_LAST, BACKWARD, 0); if (eap->do_ecmd_cmd != NULL) { do_cmdline_cmd(eap->do_ecmd_cmd); } }
void nvim_docmd_do_bang(int addr_count, exarg_T *eap, bool forceit) { do_bang(addr_count, eap, forceit, true, true); }
void nvim_docmd_pum_make_popup(const char *arg, bool forceit) { pum_make_popup(arg, (int)forceit); }
void nvim_docmd_wundo(const char *arg, bool forceit) { uint8_t hash[UNDO_HASH_SIZE]; u_compute_hash(curbuf, hash); u_write_undo(arg, forceit, curbuf, hash); }
void nvim_docmd_rundo(const char *arg) { uint8_t hash[UNDO_HASH_SIZE]; u_compute_hash(curbuf, hash); u_read_undo((char *)arg, hash, NULL); }
void nvim_docmd_checkpath(bool forceit) { find_pattern_in_path(NULL, 0, 0, false, false, CHECK_PATH, 1, forceit ? ACTION_SHOW_ALL : ACTION_SHOW, 1, (linenr_T)MAXLNUM, forceit, false); }


/// Wrapper for do_bufdel (buffer unload/delete/wipe).
char *nvim_docmd_do_bufdel(int command, const char *arg, int addr_count, int start_bnr,
                            int end_bnr, int forceit)
{
  return do_bufdel(command, (char *)arg, addr_count, start_bnr, end_bnr, forceit);
}
bool nvim_docmd_curbuf_get_did_filetype(void) { return curbuf->b_did_filetype; }
void nvim_docmd_curbuf_set_did_filetype(bool val) { curbuf->b_did_filetype = val; }

/// Set filetype option to arg via set_option_value_give_err.
void nvim_docmd_set_filetype_option(const char *arg)
{
  set_option_value_give_err(kOptFiletype, CSTR_AS_OPTVAL((char *)arg), OPT_LOCAL);
}
int nvim_docmd_setfname_curbuf(const char *arg) { return setfname(curbuf, (char *)arg, NULL, true); }
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

bool nvim_docmd_curbuf_ml_empty(void) { return curbuf->b_ml.ml_flags & ML_EMPTY; }
void nvim_docmd_get_curwin_cursor_pos(int *lnum, int *col, int *coladd) { *lnum = (int)curwin->w_cursor.lnum; *col = (int)curwin->w_cursor.col; *coladd = (int)curwin->w_cursor.coladd; }
void nvim_docmd_set_curwin_cursor_pos(int lnum, int col, int coladd) { curwin->w_cursor.lnum = (linenr_T)lnum; curwin->w_cursor.col = (colnr_T)col; curwin->w_cursor.coladd = (colnr_T)coladd; }
const char *nvim_docmd_get_last_chdir_reason(void) { return last_chdir_reason; }
bool nvim_docmd_curwin_has_localdir(void) { return curwin->w_localdir != NULL; }
bool nvim_docmd_curtab_has_localdir(void) { return curtab->tp_localdir != NULL; }
int nvim_docmd_typebuf_tb_len(void) { return typebuf.tb_len; }
void nvim_docmd_do_cmdline_getexline(void) { do_cmdline(NULL, getexline, NULL, DOCMD_NOWAIT | DOCMD_VERBOSE); }
void nvim_set_virtual_op_false(void) { virtual_op = kFalse; }
void nvim_docmd_set_curwin_curswant(int val) { curwin->w_curswant = (colnr_T)val; }
int nvim_docmd_is_only_tabpage(void) { return first_tabpage->tp_next == NULL ? 1 : 0; }
int nvim_docmd_tabpage_is_current(void *tp) { return tp == curtab ? 1 : 0; }
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

linenr_T nvim_docmd_get_address_for_copymove(exarg_T *eap, const char **errormsg) { return get_address(eap, &eap->arg, eap->addr_type, false, false, false, 1, errormsg); }
int nvim_docmd_buf_hide_curwin(void) { return buf_hide(curwin->w_buffer) ? 1 : 0; }
int nvim_docmd_curwin_p_wrap(void) { return curwin->w_p_wrap ? 1 : 0; }

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
FILE *nvim_docmd_os_fopen(const char *fname, const char *mode) { return os_fopen(fname, mode); }

// Returns the effective grep or make program string (buffer-local if set, else global).
const char *nvim_docmd_get_grep_or_make_program(int isgrep)
{
  return isgrep ? (*curbuf->b_p_gp == NUL ? p_gp : curbuf->b_p_gp)
                : (*curbuf->b_p_mp == NUL ? p_mp : curbuf->b_p_mp);
}

// Show dialog_changed prompt and return true if buffer is still changed after.
bool nvim_docmd_dialog_changed_still_dirty(buf_T *buf)
{
  bufref_T bufref;
  set_bufref(&bufref, buf);
  dialog_changed(buf, false);
  return bufref_valid(&bufref) && bufIsChanged(buf);
}
int nvim_docmd_curwin_is_floating(void) { return curwin->w_floating ? 1 : 0; }
int nvim_docmd_is_one_window(void) { return ONE_WINDOW ? 1 : 0; }
void nvim_docmd_ex_win_close_curwin(int forceit) { ex_win_close(forceit, curwin, NULL); }
win_T *nvim_docmd_tabpage_get_lastwin(void *tp) { return ((tabpage_T *)tp)->tp_lastwin; }
int nvim_docmd_tabpage_lastwin_eq(void *tp, void *wp) { return ((tabpage_T *)tp)->tp_lastwin == (win_T *)wp ? 1 : 0; }
void nvim_docmd_ex_win_close_in_tab(int forceit, void *wp, void *tp) { ex_win_close(forceit, (win_T *)wp, (tabpage_T *)tp); }
void nvim_docmd_curwin_clear_localdir(void) { XFREE_CLEAR(curwin->w_localdir); }
void nvim_docmd_curtab_clear_localdir(void) { XFREE_CLEAR(curtab->tp_localdir); }
const char *nvim_docmd_get_globaldir(void) { return globaldir; }
void nvim_docmd_set_globaldir_strdup(const char *pdir) { globaldir = xstrdup(pdir); }
void nvim_docmd_clear_globaldir(void) { XFREE_CLEAR(globaldir); }
int nvim_docmd_os_dirname_cwd(char *buf, size_t len) { return (int)os_dirname(buf, len); }
void nvim_docmd_curtab_set_localdir(const char *cwd) { curtab->tp_localdir = xstrdup(cwd); }
void nvim_docmd_curwin_set_localdir(const char *cwd) { curwin->w_localdir = xstrdup(cwd); }
void nvim_docmd_set_last_chdir_reason_null(void) { last_chdir_reason = NULL; }
void nvim_docmd_shorten_fnames_nosymlinks(void) { shorten_fnames(vim_strchr(p_cpo, CPO_NOSYMLINKS) == NULL); }
void nvim_docmd_do_autocmd_dirchanged_manual_post(const char *cwd, int scope) { do_autocmd_dirchanged(cwd, (CdScope)scope, kCdCauseManual, false); }
void ex_may_print(exarg_T *eap) { nvim_docmd_ex_may_print_impl(eap); }
int nvim_get_restart_edit(void) { return restart_edit; }
void nvim_set_restart_edit(int val) { restart_edit = val; }
int nvim_get_force_restart_edit(void) { return force_restart_edit ? 1 : 0; }
void nvim_set_force_restart_edit(int val) { force_restart_edit = (bool)val; }
void nvim_set_current_State(int val) { State = val; }
int nvim_docmd_sst_save_typeahead(save_state_T *sst) { save_typeahead(&sst->tabuf); return sst->tabuf.typebuf_valid ? 1 : 0; }
void nvim_docmd_sst_restore_typeahead(save_state_T *sst) { restore_typeahead(&sst->tabuf); }
void nvim_docmd_win_set_alt_fnum(win_T *wp, int fnum) { wp->w_alt_fnum = fnum; }
int nvim_docmd_get_global_cmdmod_flags(void) { return cmdmod.cmod_flags; }
void nvim_docmd_set_curbuf_b_p_ro(int v) { curbuf->b_p_ro = (v != 0); }
linenr_T nvim_docmd_eap_get_do_ecmd_lnum(const exarg_T *eap) { return eap->do_ecmd_lnum; }
char *nvim_docmd_eval_curbuf_fname(void) { return curbuf->b_fname; }
int nvim_docmd_get_quitmore(void) { return quitmore; }
void nvim_docmd_set_quitmore(int n) { quitmore = n; }
void nvim_docmd_check_more_semsg(int n) { semsg(NGETTEXT("E173: %" PRId64 " more file to edit", "E173: %" PRId64 " more files to edit", (unsigned)n), (int64_t)n); }
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

void nvim_docmd_tabpage_new_body(void) { exarg_T ea = { .cmdidx = CMD_tabnew, .cmd = "tabn", .arg = "" }; nvim_docmd_ex_splitview_impl(&ea); }
int nvim_docmd_curbuf_b_nwindows(void) { return curbuf->b_nwindows; }
int nvim_docmd_curbuf_ml_has_empty(void) { return (curbuf->b_ml.ml_flags & ML_EMPTY) ? 1 : 0; }
void nvim_docmd_do_bang_read(exarg_T *eap) { do_bang(1, eap, false, false, true); }
const char *nvim_docmd_curbuf_b_fname(void) { return curbuf->b_fname; }
const char *nvim_docmd_e_notopen_str(void) { return _(e_notopen); }
linenr_T nvim_docmd_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
void nvim_docmd_curwin_cursor_lnum_maybe_dec(linenr_T lnum) { if (curwin->w_cursor.lnum > 1 && curwin->w_cursor.lnum >= lnum) { curwin->w_cursor.lnum--; } }
int nvim_docmd_curwin_w_arg_idx_invalid(void) { return curwin->w_arg_idx_invalid ? 1 : 0; }
size_t nvim_docmd_add_win_cmd_modifiers_global(char *buf, size_t bufsize)
{
  bool multi_mods = false;
  buf[0] = NUL;
  size_t len = add_win_cmd_modifiers(buf, &cmdmod, &multi_mods);
  assert(len < bufsize);
  return len;
}

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
const char *nvim_docmd_get_cmod_confirm_prefix(void) { return (cmdmod.cmod_flags & CMOD_CONFIRM) ? "confirm " : NULL; }
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
void nvim_docmd_semsg_multiline_emsg(const char *msg) { semsg_multiline("emsg", (char *)msg); }
int nvim_docmd_findfunc_set_global(void) { return option_set_callback_func(p_ffu, &ffu_cb); }
int nvim_docmd_findfunc_set_local(buf_T *buf) { return option_set_callback_func(buf->b_p_ffu, &buf->b_ffu_cb); }
void nvim_docmd_findfunc_free_local_cb(buf_T *buf) { callback_free(&buf->b_ffu_cb); }
char *nvim_docmd_optset_varp_deref(optset_T *args) { return *(char **)args->os_varp; }
void nvim_docmd_optset_varp_set(optset_T *args, char *name) { *(char **)args->os_varp = name; }
size_t nvim_xp_get_pattern_len(expand_T *xp) { return xp->xp_pattern_len; }
void nvim_docmd_dec_quitmore(void) { quitmore--; }
/// Allocate a zeroed exarg_T on the heap (line1=1, line2=1).
exarg_T *nvim_eap_alloc(void)
{
  exarg_T *eap = xcalloc(1, sizeof(exarg_T));
  eap->line1 = 1;
  eap->line2 = 1;
  return eap;
}
void nvim_docmd_do_finish(exarg_T *eap) { do_finish(eap, true); }
cmdmod_T *nvim_docmd_save_cmdmod(void) { cmdmod_T *save = xmalloc(sizeof(cmdmod_T)); *save = cmdmod; return save; }
void nvim_docmd_restore_cmdmod(cmdmod_T *save) { cmdmod = *save; xfree(save); }
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
char nvim_docmd_ask_yesno_backwards(void) { return (char)ask_yesno(_("Backwards range given, OK to swap")); }
bool nvim_docmd_curbuf_modifiable(void) { return MODIFIABLE(curbuf) != 0; }
char *nvim_docmd_ex_errmsg_trailing(const char *arg) { return ex_errmsg(e_trailing_arg, arg); }
void nvim_docmd_fix_cursor_if_zero(void) { if (curwin->w_cursor.lnum == 0) { curwin->w_cursor.lnum = 1; curwin->w_cursor.col = 0; } }
/// Format an error message with arg into buf.
/// Wraps vim_snprintf(buf, buflen, _(msg), arg).
char *nvim_docmd_ex_errmsg_format(const char *msg, const char *arg,
                                   char *buf, size_t buflen)
{
  vim_snprintf(buf, buflen, _(msg), arg);
  return buf;
}
uint32_t nvim_docmd_get_argt_for_idx(int idx) { return cmdnames[idx].cmd_argt; }
int nvim_docmd_parse_command_modifiers_global(exarg_T *eap, const char **errormsg) { return parse_command_modifiers(eap, errormsg, &cmdmod, false); }
void nvim_docmd_set_sourcing_lnum(linenr_T lnum) { if (exestack.ga_data != NULL && exestack.ga_len > 0) { SOURCING_LNUM = lnum; } }
void nvim_docmd_ga_deep_clear_lines(garray_T *gap) { GA_DEEP_CLEAR(gap, wcmd_T, FREE_WCMD); }

/// Free the cs_emsg_silent_list from a cstack.
/// Called from Rust do_cmdline after the cstack goes out of scope.
void nvim_docmd_free_emsg_silent_list(cstack_T *cstack)
{
  if (cstack->cs_emsg_silent_list != NULL) {
    eslist_T *temp;
    for (eslist_T *elem = cstack->cs_emsg_silent_list; elem != NULL; elem = temp) {
      temp = elem->next;
      xfree(elem);
    }
  }
}
