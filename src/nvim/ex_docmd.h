#pragma once

#include <stdbool.h>

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/getchar_defs.h"
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/vim_defs.h"  // IWYU pragma: keep

/// flags for do_cmdline()
enum {
  DOCMD_VERBOSE  = 0x01,  ///< included command in error message
  DOCMD_NOWAIT   = 0x02,  ///< don't call wait_return() and friends
  DOCMD_REPEAT   = 0x04,  ///< repeat exec. until getline() returns NULL
  DOCMD_KEYTYPED = 0x08,  ///< don't reset KeyTyped
  DOCMD_EXCRESET = 0x10,  ///< reset exception environment (for debugging
  DOCMD_KEEPLINE = 0x20,  ///< keep typed line for repeating with "."
};

/// defines for eval_vars()
enum {
  VALID_PATH = 1,
  VALID_HEAD = 2,
};

// Whether a command index indicates a user command.
#define IS_USER_CMDIDX(idx) ((int)(idx) < 0)

enum { DIALOG_MSG_SIZE = 1000, };  ///< buffer size for dialog_msg()

/// Fill `buff` with `format` applied to `fname` (uses "Untitled" if fname is NULL).
void dialog_msg(char *buff, char *format, char *fname);

/// Structure used to save the current state.  Used when executing Normal mode
/// commands while in any other mode.
typedef struct {
  int save_msg_scroll;
  int save_restart_edit;
  bool save_msg_didout;
  int save_State;
  bool save_finish_op;
  int save_opcount;
  int save_reg_executing;
  bool save_pending_end_reg_executing;
  tasave_T tabuf;
} save_state_T;

#include "ex_docmd.h.generated.h"

// Rust-exported functions (not in generated header)
bool is_map_cmd(cmdidx_T cmdidx);
char *nvim_docmd_cmd_exists_inner(const char *name, int *out_cmdidx, int *out_full, int *out_useridx);
void nvim_docmd_do_one_cmd_doend(cstack_T *cstack, const char *errormsg, int flags, const exarg_T *eap);
void nvim_set_prevdir(int scope, char *pdir);
int nvim_docmd_first_loaded_fnum_or_fail(void);
int nvim_docmd_last_loaded_fnum_or_fail(void);
int nvim_docmd_parse_count_digits(exarg_T *eap);

// Forward declarations for Rust-implemented functions exported under C names via #[export_name].
// These replace the C wrapper bodies that were deleted during the migration.
bool checkforcmd(char **pp, const char *cmd, int len);
char *find_ex_command(exarg_T *eap, int *full);
int cmd_exists(const char *name);
void f_fullcommand(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
char *skip_range(const char *cmd, int *ctx);
linenr_T get_address(exarg_T *eap, char **ptr, cmd_addr_T addr_type, bool skip, bool silent,
                     int to_other_file, int address_count, const char **errormsg);
char *invalid_range(exarg_T *eap);
int expand_filename(exarg_T *eap, char **cmdlinep, const char **errormsgp);
void separate_nextcmd(exarg_T *eap);
char *getargcmd(char **argp);
void set_cmd_addr_type(exarg_T *eap, char *p);
linenr_T get_cmd_default_range(exarg_T *eap);
void set_cmd_dflall_range(exarg_T *eap);
void set_cmd_count(exarg_T *eap, linenr_T count, bool validate);
bool parse_cmdline(char **cmdline, exarg_T *eap, CmdParseInfo *cmdinfo, const char **errormsg);
int execute_cmd(exarg_T *eap, CmdParseInfo *cmdinfo, bool preview);
int parse_command_modifiers(exarg_T *eap, const char **errormsg, cmdmod_T *cmod, bool skip_only);
bool parse_cmd_address(exarg_T *eap, const char **errormsg, bool silent);
// Phase 3: type-converting and widely-called wrappers replaced by Rust exports
int ends_excmd(int c);
char *find_nextcmd(const char *p);
char *check_nextcmd(char *p);
bool is_loclist_cmd(int cmdidx);
int get_pressedreturn(void);
bool expr_map_locked(void);
int modifier_len(char *cmd);
bool is_cmd_ni(cmdidx_T cmdidx);
bool cmd_has_expr_args(cmdidx_T cmdidx);
cmdidx_T excmd_get_cmdidx(const char *cmd, size_t len);
char *get_command_name(expand_T *xp, int idx);
int get_bad_opt(const char *p, exarg_T *eap);
int getargopt(exarg_T *eap);
char *skip_cmd_arg(char *p, bool rembs);
bool changedir_func(char *new_dir, CdScope scope);
void verify_command(const char *cmd);
// Rust-exported ex_* command implementations (Phase 1 migration).
void ex_autocmd(exarg_T *eap);
void ex_doautocmd(exarg_T *eap);
void ex_bunload(exarg_T *eap);
void ex_buffer(exarg_T *eap);
void ex_bmodified(exarg_T *eap);
void ex_bnext(exarg_T *eap);
void ex_bprevious(exarg_T *eap);
void ex_brewind(exarg_T *eap);
void ex_blast(exarg_T *eap);
void ex_colorscheme(exarg_T *eap);
void ex_highlight(exarg_T *eap);
void ex_quit(exarg_T *eap);
void ex_quitall(exarg_T *eap);
void ex_close(exarg_T *eap);
void ex_tabclose(exarg_T *eap);
void ex_only(exarg_T *eap);
void ex_hide(exarg_T *eap);
void ex_exit(exarg_T *eap);
void ex_print(exarg_T *eap);
void ex_preserve(exarg_T *eap);
void ex_recover(exarg_T *eap);
void ex_wrongmodifier(exarg_T *eap);
void ex_tabmove(exarg_T *eap);
void ex_resize(exarg_T *eap);
void ex_edit(exarg_T *eap);
void ex_pwd(exarg_T *eap);
void ex_equal(exarg_T *eap);
void ex_winsize(exarg_T *eap);
void ex_wincmd(exarg_T *eap);
void ex_put(exarg_T *eap);
void ex_iput(exarg_T *eap);
void ex_copymove(exarg_T *eap);
void ex_join(exarg_T *eap);
void ex_at(exarg_T *eap);
void ex_bang(exarg_T *eap);
void ex_wundo(exarg_T *eap);
void ex_rundo(exarg_T *eap);
void ex_redo(exarg_T *eap);
void ex_later(exarg_T *eap);
void ex_redir(exarg_T *eap);
void ex_redraw(exarg_T *eap);
void ex_redrawstatus(exarg_T *eap);
void ex_redrawtabline(exarg_T *eap);
void ex_mark(exarg_T *eap);
void ex_normal(exarg_T *eap);
void ex_startinsert(exarg_T *eap);
void ex_stopinsert(exarg_T *eap);
void ex_checkpath(exarg_T *eap);
void ex_psearch(exarg_T *eap);
void ex_shada(exarg_T *eap);
void ex_filetype(exarg_T *eap);
void ex_setfiletype(exarg_T *eap);
void ex_nohlsearch(exarg_T *eap);
void ex_folddo(exarg_T *eap);
void ex_nogui(exarg_T *eap);
void ex_popup(exarg_T *eap);
void ex_ni(exarg_T *eap);
void ex_script_ni(exarg_T *eap);
void not_exiting(void);
void ex_cquit(exarg_T *eap);
void ex_fclose(exarg_T *eap);
void ex_stop(exarg_T *eap);
void ex_submagic(exarg_T *eap);
int ex_submagic_preview(exarg_T *eap, int cmdpreview_ns, handle_T cmdpreview_bufnr);
ssize_t find_cmdline_var(const char *src, size_t *usedlen);
void ex_fold(exarg_T *eap);
void ex_foldopen(exarg_T *eap);
void ex_digraphs(exarg_T *eap);
void ex_mode(exarg_T *eap);
void ex_swapname(exarg_T *eap);
void ex_tabnext(exarg_T *eap);
void ex_undo(exarg_T *eap);
void ex_sleep(exarg_T *eap);
void ex_operators(exarg_T *eap);
void ex_find(exarg_T *eap);
void ex_goto(exarg_T *eap);
void ex_tag(exarg_T *eap);
void ex_ptag(exarg_T *eap);
void ex_stag(exarg_T *eap);
void ex_pclose(exarg_T *eap);
void ex_pedit(exarg_T *eap);
void ex_pbuffer(exarg_T *eap);
void ex_findpat(exarg_T *eap);
void ex_tabs(exarg_T *eap);
void ex_syncbind(exarg_T *eap);
void ex_read(exarg_T *eap);
void ex_detach(exarg_T *eap);
void ex_connect(exarg_T *eap);
void ex_checkhealth(exarg_T *eap);
void ex_terminal(exarg_T *eap);
void ex_restart(exarg_T *eap);
void ex_tabonly(exarg_T *eap);
// Rust-implemented nvim_docmd helpers (previously C _impl bodies).
void nvim_docmd_ex_may_print_impl(exarg_T *eap);
void nvim_docmd_ex_splitview_impl(exarg_T *eap);
// Phase 1 (ex_docmd plan): thin C stubs eliminated; Rust functions now exported directly.
void not_restarting(void);
int before_quit_all(exarg_T *eap);
void set_no_hlsearch(bool flag);
void set_pressedreturn(bool val);
void ex_cd(exarg_T *eap);
void do_sleep(int64_t msec, bool hide_cursor);
// Phase 6 (ex_docmd plan): C bodies deleted; Rust exports old names via #[no_mangle].
void do_exmode(void);
int do_cmdline_cmd(const char *cmd);
// Phase 4 (do_cmdline plan): do_cmdline implemented in Rust (do_cmdline.rs).
int do_cmdline(char *cmdline, LineGetter fgetline, void *cookie, int flags);
void filetype_plugin_enable(void);
void filetype_maybe_enable(void);
// Phase 2 (ex_docmd plan): C bodies renamed to nvim_docmd_*_impl; Rust exports old names.
void do_exedit(exarg_T *eap, win_T *old_curwin);
void ex_splitview(exarg_T *eap);
bool before_quit_autocmds(win_T *wp, bool quit_all, bool forceit);
void ex_win_close(int forceit, win_T *win, tabpage_T *tp);
void tabpage_close(int forceit);
void tabpage_close_other(tabpage_T *tp, int forceit);
void tabpage_new(void);
void handle_did_throw(void);
// Phase 4 (ex_docmd plan): do_one_cmd, ex_errmsg, excmd_get_argt implemented in Rust.
char *ex_errmsg(const char *msg, const char *arg);
uint32_t excmd_get_argt(cmdidx_T idx);
// Phase 5 (ex_docmd plan): getline_equal, getline_cookie implemented in Rust.
bool getline_equal(LineGetter fgetline, void *cookie, LineGetter func);
void *getline_cookie(LineGetter fgetline, void *cookie);
// Phase 3 (cleanup plan): C wrappers deleted; Rust exports these names directly via #[export_name].
void exec_normal(bool was_typed, bool use_vpeekc);
void exec_normal_cmd(char *cmd, int remap, bool silent);
void update_topline_cursor(void);
int vim_mkdir_emsg(const char *name, int prot);
void apply_cmdmod(cmdmod_T *cmod);
void undo_cmdmod(cmdmod_T *cmod);
char *replace_makeprg(exarg_T *eap, char *arg, char **cmdlinep);
int expand_argopt(char *pat, expand_T *xp, regmatch_T *rmp, char ***matches, int *numMatches);
FILE *open_exfile(char *fname, int forceit, char *mode);
bool save_current_state(save_state_T *sst);
void restore_current_state(save_state_T *sst);
char *eval_vars(char *src, const char *srcstart, size_t *usedlen, linenr_T *lnump,
                const char **errormsg, int *escaped, bool empty_is_error);
