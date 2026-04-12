// Make sure extern symbols are exported on Windows
#ifdef WIN32
# define EXTERN __declspec(dllexport)
#else
# define EXTERN
#endif
#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef ENABLE_ASAN_UBSAN
# include <sanitizer/asan_interface.h>
# ifndef MSWIN
#  include <sanitizer/ubsan_interface.h>
# endif
#endif

#include "auto/config.h"  // IWYU pragma: keep
#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/ui.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/channel.h"
#include "nvim/channel_defs.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/diff.h"
#include "nvim/drawline.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/stream.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/hashtab.h"
#include "nvim/highlight.h"
#include "nvim/highlight_group.h"
#include "nvim/keycodes.h"
#include "nvim/log.h"
#include "nvim/lua/executor.h"
#include "nvim/lua/secure.h"
#include "nvim/lua/treesitter.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mark.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/msgpack_rpc/server.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/lang.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/signal.h"
#include "nvim/os/stdpaths_defs.h"
#include "nvim/path.h"
#include "nvim/popupmenu.h"
#include "nvim/profile.h"
#include "nvim/quickfix.h"
#include "nvim/register.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/shada.h"
#include "nvim/statusline.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_client.h"
#include "nvim/ui_compositor.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#ifdef MSWIN
# include "nvim/os/os_win_console.h"
# ifndef _UCRT
#  error UCRT is the only supported C runtime on windows
# endif
#endif

#if defined(MSWIN) && !defined(MAKE_LIB)
# include "nvim/mbyte.h"
#endif

// values for "window_layout"
enum {
  WIN_HOR = 1,   // "-o" horizontally split windows
  WIN_VER = 2,   // "-O" vertically split windows
  WIN_TABS = 3,  // "-p" windows on tab pages
};

// Values for edit_type.
enum {
  EDIT_NONE = 0,   // no edit type yet
  EDIT_FILE = 1,   // file name argument[s] given, use argument list
  EDIT_STDIN = 2,  // read file from stdin
  EDIT_TAG = 3,    // tag name argument given, use tagname
  EDIT_QF = 4,     // start in quickfix mode
};

// Result struct for cs_remote_call (must be before main.c.generated.h)
typedef struct {
  int should_exit;  // -1=kNone, 0=kFalse, 1=kTrue
  int tabbed;       // -1=kNone, 0=kFalse, 1=kTrue
} CsRemoteResult;

#include "main.c.generated.h"
extern int rs_only_one_window(void);

// Rust FFI declarations (window wrappers removed)
extern int rs_win_count(void);
extern void rs_win_equal(win_T *next_curwin, int current, int dir);

// Rust implementation in nvim-event crate
extern MultiQueue *rs_loop_get_events(Loop *loop);
extern int rs_shada_read_everything(const char *fname, bool forceit, bool missing_ok);
extern int rs_shada_write_file(const char *file, bool nomerge);
#define loop_get_events(l) rs_loop_get_events(l)

// Rust implementations (Phase 1: output and helpers)
extern void rs_usage(void);
extern void rs_version(void);
extern void rs_print_mainerr(const char *msg1, const char *msg2, const char *msg3);
extern void rs_mainerr(const char *msg1, const char *msg2, const char *msg3) FUNC_ATTR_NORETURN;
extern void rs_check_swap_exists_action(void);
extern void rs_command_line_scan(mparm_T *parmp);

// Rust implementations (Phase 2: init and config)
extern void rs_init_params(mparm_T *paramp, int argc, char **argv);
extern void rs_init_startuptime(const mparm_T *paramp);
extern void rs_check_and_set_isatty(mparm_T *paramp);
extern void rs_init_path(const char *exename);
extern void rs_set_window_layout(mparm_T *paramp);
extern void rs_source_startup_scripts(const mparm_T *parmp);
extern int get_number_arg(const char *p, int *idx, int def);
extern bool edit_stdin(const mparm_T *parmp);
extern char *get_fname(mparm_T *parmp, char *cwd);

// Expose GARGLIST[idx] name to Rust
char *nvim_garglist_name(int idx) { return alist_name(&GARGLIST[idx]); }

// Rust implementations (Phase 3: command execution and quickfix helpers)
extern void rs_exe_pre_commands(mparm_T *parmp);
extern void rs_exe_commands(mparm_T *parmp);
extern void rs_handle_quickfix(mparm_T *paramp);
extern void rs_handle_tag(char *tagname);

// Thin helper: set 'errorfile' option from Rust (avoids OptVal complexity)
void nvim_set_errorfile_opt(const char *val) { set_option_direct(kOptErrorfile, CSTR_AS_OPTVAL(val), 0, SID_CARG); }

// Rust implementations (Phase 1: lifecycle)
extern uint64_t rs_server_connect(char *server_addr, const char **errmsg);
extern void rs_os_exit(int r) FUNC_ATTR_NORETURN;

// Rust implementations (Phase 2: stdin)
extern void rs_read_stdin(void);

// Rust implementations (Phase 3+4: windows and buffers)
extern void rs_create_windows(mparm_T *parmp);
extern void rs_edit_buffers(mparm_T *parmp, char *cwd);

// Rust implementation (Phase 5: remote_request)
extern void rs_remote_request(mparm_T *params, int remote_args, char *server_addr, int argc,
                               char **argv, bool ui_only);

// C helper: builds API types, calls vim._cs_remote via nlua_exec.
// Calls os_exit on error. Returns result for Rust to process.
static CsRemoteResult cs_remote_call(uint64_t chan, const char *server_addr,
                                     const char *connect_error,
                                     int argc, char **argv, int remote_args)
{
  Array args = ARRAY_DICT_INIT;
  kv_resize(args, (size_t)(argc - remote_args));
  for (int t_argc = remote_args; t_argc < argc; t_argc++) {
    ADD_C(args, CSTR_AS_OBJ(argv[t_argc]));
  }

  Error err = ERROR_INIT;
  MAXSIZE_TEMP_ARRAY(a, 4);
  ADD_C(a, INTEGER_OBJ((int)chan));
  ADD_C(a, CSTR_AS_OBJ(server_addr));
  ADD_C(a, CSTR_AS_OBJ(connect_error));
  ADD_C(a, ARRAY_OBJ(args));
  String s = STATIC_CSTR_AS_STRING("return vim._cs_remote(...)");
  Object o = nlua_exec(s, NULL, a, kRetObject, NULL, &err);
  kv_destroy(args);
  if (ERROR_SET(&err)) {
    fprintf(stderr, "%s\n", err.msg);
    os_exit(2);
  }

  if (o.type != kObjectTypeDict) {
    fprintf(stderr, "vim._cs_remote returned unexpected value\n");
    os_exit(2);
  }

  CsRemoteResult result = { .should_exit = -1, .tabbed = -1 };
  Dict rvdict = o.data.dict;

  for (size_t i = 0; i < rvdict.size; i++) {
    if (strequal(rvdict.items[i].key.data, "errmsg")) {
      if (rvdict.items[i].value.type != kObjectTypeString) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'errmsg'\n");
        os_exit(2);
      }
      fprintf(stderr, "%s\n", rvdict.items[i].value.data.string.data);
      os_exit(2);
    } else if (strequal(rvdict.items[i].key.data, "result")) {
      if (rvdict.items[i].value.type != kObjectTypeString) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'result'\n");
        os_exit(2);
      }
      printf("%s", rvdict.items[i].value.data.string.data);
    } else if (strequal(rvdict.items[i].key.data, "tabbed")) {
      if (rvdict.items[i].value.type != kObjectTypeBoolean) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'tabbed'\n");
        os_exit(2);
      }
      result.tabbed = rvdict.items[i].value.data.boolean ? 1 : 0;
    } else if (strequal(rvdict.items[i].key.data, "should_exit")) {
      if (rvdict.items[i].value.type != kObjectTypeBoolean) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'should_exit'\n");
        os_exit(2);
      }
      result.should_exit = rvdict.items[i].value.data.boolean ? 1 : 0;
    }
  }

  if (result.should_exit == -1 || result.tabbed == -1) {
    fprintf(stderr, "vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n");
    os_exit(2);
  }
  api_free_object(o);
  return result;
}

// Non-static wrapper for Rust: packs {should_exit, tabbed} into int64_t.
// High 32 bits = should_exit (-1/0/1), low 32 bits = tabbed (-1/0/1).
int64_t nvim_call_cs_remote(uint64_t chan, const char *server_addr,
                             const char *connect_error, int argc, char **argv,
                             int remote_args)
{
  CsRemoteResult r = cs_remote_call(chan, server_addr, connect_error, argc, argv, remote_args);
  return ((int64_t)r.should_exit << 32) | (uint32_t)r.tabbed;
}

// C helper: set 'shortmess' option from Rust (avoids OptVal complexity)
void nvim_set_shortmess_opt(const char *val)
{
  set_option_value_give_err(kOptShortmess, CSTR_AS_OPTVAL(val), 0);
}

// C helpers for rs_server_connect / rs_os_exit (Phase 1)
uint64_t nvim_channel_connect(bool is_tcp, const char *server_addr, const char **error)
{
  CallbackReader on_data = CALLBACK_READER_INIT;
  return channel_connect(is_tcp, server_addr, true, on_data, 500, error);
}

// Forward declaration for Rust-exported event_teardown
extern bool event_teardown(void);
bool nvim_event_teardown(void) { return event_teardown(); }

void nvim_free_all_mem_if_exitfree(void)
{
#ifdef EXITFREE
  free_all_mem();
#endif
}

Loop main_loop;

char *nvim_argv0 = NULL;
#define argv0 nvim_argv0

// C accessors for Rust event_init/event_teardown (Phase 1)
void *nvim_get_main_loop(void) { return &main_loop; }
void nvim_set_resize_events(void *mq) { resize_events = (MultiQueue *)mq; }
void nvim_time_msg(const char *msg) { if (time_fd != NULL) time_msg(msg, NULL); }

// Forward declaration for Rust-exported event_init (Phase 1)
extern void event_init(void);

// event_init() and event_teardown() are implemented in Rust (src/nvim-rs/main/src/init.rs)

// C accessors for Rust early_init (Phase 2)
void *nvim_get_global_alist_ptr(void) { return &global_alist; }
void nvim_set_global_alist_id(int id) { global_alist.id = id; }
bool nvim_paramp_get_clean(const mparm_T *paramp) { return paramp->clean; }

// early_init() is implemented in Rust (src/nvim-rs/main/src/init.rs)
// Forward declaration for Rust-exported symbol
extern void early_init(mparm_T *paramp);

// =============================================================================
// C accessors for rs_main_server_setup / rs_main_post_startup / rs_main_enter_loop
// =============================================================================

// server setup helpers
void nvim_diff_win_options_firstwin(void) { diff_win_options(firstwin, false); }

void nvim_setup_cmdline_row(void)
{
  assert(p_ch >= 0 && Rows >= p_ch && Rows - p_ch <= INT_MAX);
  cmdline_row = Rows - (int)p_ch;
  msg_row = cmdline_row;
}

void nvim_sync_firstwin_height(void) { firstwin->w_prev_height = firstwin->w_height; }

// Opens scriptout file; returns false and prints error if it fails.
bool nvim_open_scriptout(char *path, bool append)
{
  scriptout = os_fopen(path, append ? APPENDBIN : WRITEBIN);
  if (scriptout == NULL) {
    fprintf(stderr, _("Cannot open for script output: \""));
    fprintf(stderr, "%s\"\n", path);
    return false;
  }
  return true;
}

// Returns true if use_vimrc is "NONE".
bool nvim_vimrc_is_none(const mparm_T *parmp) { return strequal(parmp->use_vimrc, "NONE"); }

void nvim_set_p_lpl(bool val) { p_lpl = val; }

// post-startup helpers
void nvim_set_vv_vim_did_init(void) { set_vim_var_nr(VV_VIM_DID_INIT, 1); }
void nvim_set_p_uc_zero(void) { p_uc = 0; }
void nvim_set_p_ut_one(void) { p_ut = 1; }
bool nvim_p_shada_is_empty(void) { return p_shada == NULL || *p_shada == NUL; }
bool nvim_vv_oldfiles_is_null(void) { return get_vim_var_list(VV_OLDFILES) == NULL; }
void nvim_init_vv_oldfiles(void) { set_vim_var_list(VV_OLDFILES, tv_list_alloc(0)); }

void nvim_redraw_later_curwin(void) { redraw_later(curwin, UPD_VALID); }

void nvim_clear_vv_swapcommand(void) { set_vim_var_string(VV_SWAPCOMMAND, NULL, -1); }

void nvim_set_curwin_lnum_to_last_line(void)
{
  curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
}

void nvim_apply_event_bufenter(void)
{
  apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
}

// FOR_ALL_WINDOWS_IN_TAB over curtab: apply diff_win_options to non-invalid windows.
void nvim_diff_win_options_curtab(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (!wp->w_arg_idx_invalid) {
      diff_win_options(wp, true);
    }
  }
}

// Sets starting=0, RedrawingDisabled=0.
void nvim_clear_starting_state(void) { starting = 0; RedrawingDisabled = 0; }

// enter-loop helpers
void nvim_set_vv_vim_did_enter(void) { set_vim_var_nr(VV_VIM_DID_ENTER, 1); }

void nvim_apply_event_vimenter(void)
{
  apply_autocmds(EVENT_VIMENTER, NULL, NULL, false, curbuf);
}

void nvim_set_reg_var_default(void) { set_reg_var(get_default_register_name()); }

bool nvim_curwin_needs_scrollbind_sync(void) { return curwin->w_p_diff && curwin->w_p_scb; }

// nvim_update_topline_curwin defined in eval_shim.c
// nvim_get_restart_edit defined in ex_docmd.c
// Returns a pointer to IObuff for error messages.
const char *nvim_get_ioerr_ptr(void) { return IObuff; }

void nvim_stuffchar_K_NOP(void) { stuffcharReadbuff(K_NOP); }

bool nvim_has_clipboard_flags(void) { return (cb_flags & (kOptCbFlagUnnamed | kOptCbFlagUnnamedplus)) != 0; }

// Rust entry points
extern void rs_main_server_setup(mparm_T *parmp);
extern void rs_main_post_startup(mparm_T *parmp, const char *fname, char *cwd, bool use_remote_ui);
extern void rs_main_enter_loop(mparm_T *parmp, bool use_remote_ui) FUNC_ATTR_NORETURN;

#ifdef MAKE_LIB
int nvim_main(int argc, char **argv);  // silence -Wmissing-prototypes
int nvim_main(int argc, char **argv)
#else
int main(int argc, char **argv)
#endif
{
  argv0 = argv[0];

  if (!appname_is_valid()) {
    fprintf(stderr, "$NVIM_APPNAME must be a name or relative path.\n");
    exit(1);
  }

  if (argc > 1 && STRICMP(argv[1], "-ll") == 0) {
    if (argc == 2) {
      rs_print_mainerr("Argument missing after", argv[1], NULL);
      exit(1);
    }
    nlua_run_script(argv, argc, 3);
  }

  char *fname = NULL;     // file name from command line
  mparm_T params;         // various parameters passed between
                          // main() and other functions.
  char *cwd = NULL;       // current working dir on startup

  // Many variables are in `params` so that we can pass them around easily.
  // `argc` and `argv` are also copied, so that they can be changed.
  rs_init_params(&params, argc, argv);

  rs_init_startuptime(&params);

  // Need to find "--clean" before actually parsing arguments.
  for (int i = 1; i < params.argc; i++) {
    if (STRICMP(params.argv[i], "--clean") == 0) {
      params.clean = true;
      break;
    }
  }

  event_init();

  early_init(&params);

  set_argv_var(argv, argc);  // set v:argv

  // Check if we have an interactive window.
  rs_check_and_set_isatty(&params);
  TIME_MSG("window checked");

  // Process the command line arguments.  File names are put in the global
  // argument list "global_alist".
  rs_command_line_scan(&params);
  TIME_MSG("parsing arguments");

  nlua_init(argv, argc, params.lua_arg0);
  TIME_MSG("init lua interpreter");

  if (embedded_mode) {
    const char *err;
    if (!channel_from_stdio(true, CALLBACK_READER_INIT, &err)) {
      abort();
    }
  }

  if (GARGCOUNT > 0) {
    fname = get_fname(&params, cwd);
  }

  // Recovery mode without a file name: List swap files.
  // In this case, no UI is needed.
  if (recoverymode && fname == NULL) {
    headless_mode = true;
  }

#ifdef MSWIN
  // on windows we use CONIN special file, thus we don't know this yet.
  bool has_term = true;
#else
  bool has_term = (stdin_isatty || stdout_isatty || stderr_isatty);
#endif
  bool use_builtin_ui = (has_term && !headless_mode && !embedded_mode && !silent_mode);

  if (params.remote) {
    rs_remote_request(&params, params.remote, params.server_addr, argc, argv,
                      use_builtin_ui);
  }

  bool remote_ui = (ui_client_channel_id != 0);

  if (use_builtin_ui && !remote_ui) {
    ui_client_forward_stdin = !stdin_isatty;
    uint64_t rv = ui_client_start_server(get_vim_var_str(VV_PROGPATH),
                                         (size_t)params.argc, params.argv);
    if (!rv) {
      fprintf(stderr, "Failed to start Nvim server!\n");
      os_exit(1);
    }
    ui_client_channel_id = rv;
  }

  // NORETURN: Start builtin UI client.
  if (ui_client_channel_id) {
    ui_client_run(remote_ui);  // NORETURN
  }
  assert(!ui_client_channel_id && !use_builtin_ui);
  // Nvim server...

  // Server setup, vimrc sourcing, plugin loading.
  setbuf(stdout, NULL);  // NOLINT(bugprone-unsafe-functions) -- must stay in C (macros)
  rs_main_server_setup(&params);

  // Post-startup: plugins, ShaDa, windows, buffers.
  bool use_remote_ui = (embedded_mode && !headless_mode);
  rs_main_post_startup(&params, fname, cwd, use_remote_ui);

#ifdef MSWIN
  if (use_remote_ui) {
    os_icon_init();
  }
  os_title_save();
#endif

  // Enter main loop. NORETURN.
  rs_main_enter_loop(&params, use_remote_ui);

#if defined(MSWIN) && !defined(MAKE_LIB)
  xfree(argv);
#endif
  return 0;
}

void os_exit(int r)
  FUNC_ATTR_NORETURN
{
  rs_os_exit(r);
}

// C helpers for Rust getout (Phase 3) ------------------------------------

// Returns adjusted exitval (adds ex_exitval if exmode_active)
int nvim_getout_exmode_adjust(int exitval)
{
  if (exmode_active) {
    exitval += ex_exitval;
  }
  return exitval;
}

// Set VV_EXITING vim variable
void nvim_getout_set_vv_exiting(int exitval)
{
  set_vim_var_type(VV_EXITING, VAR_NUMBER);
  set_vim_var_nr(VV_EXITING, exitval);
}

// Trigger BufWinLeave for all windows (once per buffer).
void nvim_getout_trigger_bufwinleave(void)
{
  const tabpage_T *next_tp;
  for (const tabpage_T *tp = first_tabpage; tp != NULL; tp = next_tp) {
    next_tp = tp->tp_next;
    FOR_ALL_WINDOWS_IN_TAB(wp, tp) {
      if (wp->w_buffer == NULL || !buf_valid(wp->w_buffer)) {
        continue;
      }
      buf_T *buf = wp->w_buffer;
      if (buf_get_changedtick(buf) != -1) {
        bufref_T bufref;
        set_bufref(&bufref, buf);
        apply_autocmds(EVENT_BUFWINLEAVE, buf->b_fname, buf->b_fname, false, buf);
        if (bufref_valid(&bufref)) {
          buf_set_changedtick(buf, -1);
        }
        next_tp = first_tabpage;
        break;
      }
    }
  }
}

// Trigger BufUnload for loaded buffers
void nvim_getout_trigger_bufunload(void)
{
  FOR_ALL_BUFFERS(buf) {
    if (buf->b_ml.ml_mfp != NULL) {
      bufref_T bufref;
      set_bufref(&bufref, buf);
      apply_autocmds(EVENT_BUFUNLOAD, buf->b_fname, buf->b_fname, false, buf);
      if (!bufref_valid(&bufref)) {
        break;
      }
    }
  }
}

// Trigger autocmd event (VimLeavePre or VimLeave), unblocking if needed
void nvim_getout_apply_autocmd_event(int event)
{
  int unblock = 0;
  if (is_autocmd_blocked()) {
    unblock_autocmds();
    unblock++;
  }
  apply_autocmds((event_T)event, NULL, NULL, false, curbuf);
  if (unblock) {
    block_autocmds();
  }
}

// Returns true if ShaDa should be written
bool nvim_getout_should_write_shada(void)
{
  return
#ifdef EXITFREE
    !entered_free_all_mem &&
#endif
    p_shada && *p_shada != NUL;
}

// Handle did_emsg: clear no_wait_return and call wait_return
void nvim_getout_handle_emsg(void)
{
  no_wait_return = false;
  wait_return(false);
}

// Restore titleold if p_title is set
void nvim_getout_restore_title(void)
{
  if (p_title && *p_titleold != NUL) {
    ui_call_set_title(cstr_as_string(p_titleold));
  }
}

// Handle restart via remote_ui_restart; sets restarting=false
void nvim_getout_do_restart(void)
{
  Error err = ERROR_INIT;
  if (!remote_ui_restart(current_ui, &err)) {
    if (ERROR_SET(&err)) {
      ELOG("%s", err.msg);
      api_clear_error(&err);
    }
  }
  restarting = false;
}

// getout() is implemented in Rust (src/nvim-rs/main/src/exit.rs)
extern void getout(int exitval) FUNC_ATTR_NORETURN;

// preserve_exit() is implemented in Rust (src/nvim-rs/main/src/exit.rs)
extern void preserve_exit(const char *errmsg) FUNC_ATTR_NORETURN;

// C helper for rs_preserve_exit: iterate buffers to check/sync swap files.
bool nvim_preserve_exit_buf_check(const char *errmsg)
{
  FOR_ALL_BUFFERS(buf) {
    if (buf->b_ml.ml_mfp != NULL && buf->b_ml.ml_mfp->mf_fname != NULL) {
      if (errmsg != NULL) {
        fprintf(stderr, "Nvim: preserving files...\n");
      }
      ml_sync_all(false, false, true);  // preserve all swap files
      return true;
    }
  }
  return false;
}


#ifdef ENABLE_ASAN_UBSAN
const char *__ubsan_default_options(void) { return "print_stacktrace=1"; }

const char *__asan_default_options(void) { return "handle_abort=1,handle_sigill=1"; }
#endif
