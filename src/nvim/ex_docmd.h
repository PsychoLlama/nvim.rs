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
