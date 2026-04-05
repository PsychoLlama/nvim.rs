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

// C helpers for rs_server_connect / rs_os_exit (Phase 1)
uint64_t nvim_channel_connect(bool is_tcp, const char *server_addr, const char **error)
{
  CallbackReader on_data = CALLBACK_READER_INIT;
  return channel_connect(is_tcp, server_addr, true, on_data, 500, error);
}

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

void event_init(void)
{
  loop_init(&main_loop, NULL);
  resize_events = multiqueue_new_child(loop_get_events(&main_loop));

  signal_init();
  // mspgack-rpc initialization
  channel_init();
  terminal_init();
  ui_init();
  TIME_MSG("event init");
}

/// @returns false if main_loop could not be closed gracefully
static bool event_teardown(void)
{
  if (!loop_get_events(&main_loop)) {
    input_stop();
    return true;
  }

  multiqueue_process_events(loop_get_events(&main_loop));
  loop_poll_events(&main_loop, 0);  // Drain thread_events, fast_events.
  input_stop();
  channel_teardown();
  proc_teardown(&main_loop);
  timer_teardown();
  server_teardown();
  signal_teardown();
  terminal_teardown();

  return loop_close(&main_loop, true);
}

/// Performs early initialization.
///
/// Needed for unit tests.
void early_init(mparm_T *paramp)
{
  os_hint_priority();
  estack_init();
  cmdline_init();
  eval_init();          // init global variables
  rs_init_path(argv0 ? argv0 : "nvim");
  init_normal_cmds();   // Init the table of Normal mode commands.
  runtime_init();
  highlight_init();

#ifdef MSWIN
  OSVERSIONINFO ovi;
  ovi.dwOSVersionInfoSize = sizeof(ovi);
  // Disable warning about GetVersionExA being deprecated. There doesn't seem to be a convenient
  // replacement that doesn't add a ton of extra code as of writing this.
# ifdef _MSC_VER
#  pragma warning(suppress : 4996)
  GetVersionEx(&ovi);
# else
  GetVersionEx(&ovi);
# endif
  snprintf(windowsVersion, sizeof(windowsVersion), "%d.%d",
           (int)ovi.dwMajorVersion, (int)ovi.dwMinorVersion);
#endif

  TIME_MSG("early init");

  // Setup to use the current locale (for ctype() and many other things).
  // NOTE: Translated messages with encodings other than latin1 will not
  // work until set_init_1() has been called!
  init_locale();

  // tabpage local options (p_ch) must be set before allocating first tabpage.
  set_init_tablocal();

  // Allocate the first tabpage, window and buffer.
  win_alloc_first();
  TIME_MSG("init first window");

  alist_init(&global_alist);    // Init the argument list to empty.
  global_alist.id = 0;

  // Set the default values for the options.
  // First find out the home directory, needed to expand "~" in options.
  init_homedir();               // find real value of $HOME
  set_init_1(paramp != NULL ? paramp->clean : false);
  log_init();
  TIME_MSG("inits 1");

  set_lang_var();               // set v:lang and v:ctype

  // initialize quickfix list
  qf_init_stack();
}

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
    remote_request(&params, params.remote, params.server_addr, argc, argv,
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

  if (!server_init(params.listen_addr)) {
    rs_mainerr(IObuff, NULL, NULL);
  }

  TIME_MSG("expanding arguments");

  if (params.diff_mode && params.window_count == -1) {
    params.window_count = 0;            // open up to 3 windows
  }
  // Don't redraw until much later.
  RedrawingDisabled++;

  setbuf(stdout, NULL);  // NOLINT(bugprone-unsafe-functions)

  full_screen = !silent_mode;

  // Set the default values for the options that use Rows and Columns.
  win_init_size();
  // Set the 'diff' option now, so that it can be checked for in a vimrc
  // file.  There is no buffer yet though.
  if (params.diff_mode) {
    diff_win_options(firstwin, false);
  }

  assert(p_ch >= 0 && Rows >= p_ch && Rows - p_ch <= INT_MAX);
  cmdline_row = Rows - (int)p_ch;
  msg_row = cmdline_row;
  default_grid_alloc();  // allocate screen buffers
  set_init_2(headless_mode);
  TIME_MSG("inits 2");

  msg_scroll = true;
  no_wait_return = true;

  init_highlight(true, false);  // Default highlight groups.
  ui_comp_syn_init();
  TIME_MSG("init highlight");

  // Set the break level after the terminal is initialized.
  debug_break_level = params.use_debug_break_level;

  // Read ex-commands if invoked with "-es".
  if (!stdin_isatty && !params.input_istext && silent_mode && exmode_active) {
    input_start();
  }

  // Wait for UIs to set up Nvim or show early messages
  // and prompts (--cmd, swapfile dialog, …).
  bool use_remote_ui = (embedded_mode && !headless_mode);
  bool listen_and_embed = params.listen_addr != NULL;
  if (use_remote_ui) {
    TIME_MSG("waiting for UI");
    remote_ui_wait_for_attach(!listen_and_embed);
    TIME_MSG("done waiting for UI");
    firstwin->w_prev_height = firstwin->w_height;  // may have changed
  }

  // prepare screen now
  starting = NO_BUFFERS;
  screenclear();
  win_new_screensize();
  TIME_MSG("clear screen");

  // Handle "foo | nvim". EDIT_FILE may be overwritten now. #6299
  if (edit_stdin(&params)) {
    params.edit_type = EDIT_STDIN;
  }

  if (params.scriptin) {
    if (!open_scriptin(params.scriptin)) {
      os_exit(2);
    }
  }
  if (params.scriptout) {
    scriptout = os_fopen(params.scriptout, params.scriptout_append ? APPENDBIN : WRITEBIN);
    if (scriptout == NULL) {
      fprintf(stderr, _("Cannot open for script output: \""));
      fprintf(stderr, "%s\"\n", params.scriptout);
      os_exit(2);
    }
  }

  nlua_init_defaults();

  TIME_MSG("init default mappings & autocommands");

  bool vimrc_none = strequal(params.use_vimrc, "NONE");

  // Reset 'loadplugins' for "-u NONE" before "--cmd" arguments.
  // Allows for setting 'loadplugins' there.
  // For --clean we still want to load plugins.
  p_lpl = vimrc_none ? params.clean : p_lpl;

  // Execute --cmd arguments.
  rs_exe_pre_commands(&params);
  TIME_MSG("--cmd commands");

  if (!vimrc_none || params.clean) {
    // Sources ftplugin.vim and indent.vim. We do this *before* the user startup scripts to ensure
    // ftplugins run before FileType autocommands defined in the init file (which allows those
    // autocommands to overwrite settings from ftplugins).
    filetype_plugin_enable();
  }

  // Source startup scripts.
  rs_source_startup_scripts(&params);
  TIME_MSG("sourcing vimrc file(s)");

  // If using the runtime (-u is not NONE), enable syntax & filetype plugins.
  if (!vimrc_none || params.clean) {
    // Sources filetype.lua unless the user explicitly disabled it with :filetype off.
    filetype_maybe_enable();
    // Sources syntax/syntax.vim. We do this *after* the user startup scripts so that users can
    // disable syntax highlighting with `:syntax off` if they wish.
    syn_maybe_enable();
  }

  set_vim_var_nr(VV_VIM_DID_INIT, 1);

  // Read all the plugin files.
  load_plugins();

  // Decide about window layout for diff mode after reading vimrc.
  rs_set_window_layout(&params);

  // Recovery mode without a file name: List swap files.
  // Uses the 'dir' option, therefore it must be after the initializations.
  if (recoverymode && fname == NULL) {
    extern int rs_recover_names(const char *fname, int do_list, void *ret_list, int nr,
                                char **fname_out);
    rs_recover_names(NULL, true, NULL, 0, NULL);
    os_exit(0);
  }

  // Set some option defaults after reading vimrc files.
  set_init_3();
  TIME_MSG("inits 3");

  // "-n" argument: Disable swap file by setting 'updatecount' to 0.
  // Note that this overrides anything from a vimrc file.
  if (params.no_swap_file) {
    p_uc = 0;
  }

  // XXX: Minimize 'updatetime' for -es/-Es. #7679
  if (silent_mode) {
    p_ut = 1;
  }

  // Read in registers, history etc, from the ShaDa file.
  // This is where v:oldfiles gets filled.
  if (*p_shada != NUL) {
    rs_shada_read_everything(NULL, false, true);
    TIME_MSG("reading ShaDa");
  }
  // It's better to make v:oldfiles an empty list than NULL.
  if (get_vim_var_list(VV_OLDFILES) == NULL) {
    set_vim_var_list(VV_OLDFILES, tv_list_alloc(0));
  }

  // "-q errorfile": Load the error file now.
  // If the error file can't be read, exit before doing anything else.
  rs_handle_quickfix(&params);
  if (params.edit_type == EDIT_QF) {
    TIME_MSG("reading errorfile");
  }

  //
  // Start putting things on the screen.
  // Scroll screen down before drawing over it
  // Clear screen now, so file message will not be cleared.
  //
  starting = NO_BUFFERS;
  no_wait_return = false;
  if (!exmode_active) {
    msg_scroll = false;
  }

  // Read file (text, not commands) from stdin if:
  //    - stdin is not a tty
  //    - and -e/-es was not given
  //
  // Do this before starting Raw mode, because it may change things that the
  // writing end of the pipe doesn't like, e.g., in case stdin and stderr
  // are the same terminal: "cat | vim -".
  // Using autocommands here may cause trouble...
  if (params.edit_type == EDIT_STDIN && !recoverymode) {
    read_stdin();
  }

  setmouse();  // may start using the mouse

  redraw_later(curwin, UPD_VALID);

  no_wait_return = true;

  // Create the requested number of windows and edit buffers in them.
  // Also does recovery if "recoverymode" set.
  create_windows(&params);
  TIME_MSG("opening buffers");

  // Clear v:swapcommand
  set_vim_var_string(VV_SWAPCOMMAND, NULL, -1);

  // Ex starts at last line of the file.
  if (exmode_active) {
    curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
  }

  apply_autocmds(EVENT_BUFENTER, NULL, NULL, false, curbuf);
  TIME_MSG("BufEnter autocommands");
  setpcmark();

  // When started with "-q errorfile" jump to first error now.
  if (params.edit_type == EDIT_QF) {
    rs_qf_jump_newwin(NULL, 0, 0, false, false);
    TIME_MSG("jump to first error");
  }

  // If opened more than one window, start editing files in the other
  // windows.
  edit_buffers(&params, cwd);
  xfree(cwd);

  if (params.diff_mode) {
    // set options in each window for "nvim -d".
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (!wp->w_arg_idx_invalid) {
        diff_win_options(wp, true);
      }
    }
  }

  // Shorten any of the filenames, but only when absolute.
  shorten_fnames(false);

  // Need to jump to the tag before executing the '-c command'.
  // Makes "vim -c '/return' -t main" work.
  rs_handle_tag(params.tagname);
  if (params.tagname != NULL) {
    TIME_MSG("jumping to tag");
  }

  // Execute any "+", "-c" and "-S" arguments.
  if (params.n_commands > 0) {
    rs_exe_commands(&params);
    TIME_MSG("executing command arguments");
  }

  starting = 0;

  RedrawingDisabled = 0;
  redraw_all_later(UPD_NOT_VALID);
  no_wait_return = false;

  // 'autochdir' has been postponed.
  do_autochdir();

  set_vim_var_nr(VV_VIM_DID_ENTER, 1);
  apply_autocmds(EVENT_VIMENTER, NULL, NULL, false, curbuf);
  TIME_MSG("VimEnter autocommands");
  if (use_remote_ui) {
    do_autocmd_uienter_all();
    TIME_MSG("UIEnter autocommands");
  }

#ifdef MSWIN
  if (use_remote_ui) {
    os_icon_init();
  }
  os_title_save();
#endif

  // Adjust default register name for "unnamed" in 'clipboard'. Can only be
  // done after the clipboard is available and all initial commands that may
  // modify the 'clipboard' setting have run; i.e. just before entering the
  // main loop.
  set_reg_var(get_default_register_name());

  // When a startup script or session file setup for diff'ing and
  // scrollbind, sync the scrollbind now.
  if (curwin->w_p_diff && curwin->w_p_scb) {
    update_topline(curwin);
    check_scrollbind(0, 0);
    TIME_MSG("diff scrollbinding");
  }

  // If ":startinsert" command used, stuff a dummy command to be able to
  // call normal_cmd(), which will then start Insert mode.
  if (restart_edit != 0) {
    stuffcharReadbuff(K_NOP);
  }

  // WORKAROUND(mhi): #3023
  if (cb_flags & (kOptCbFlagUnnamed | kOptCbFlagUnnamedplus)) {
    eval_has_provider("clipboard", false);
  }

  if (params.luaf != NULL) {
    // Like "--cmd", "+", "-c" and "-S", don't truncate messages.
    msg_scroll = true;
    DLOG("executing Lua -l script");
    bool lua_ok = nlua_exec_file(params.luaf);
    TIME_MSG("executing Lua -l script");
    if (msg_didout) {
      msg_putchar('\n');
      msg_didout = false;
    }
    getout(lua_ok ? 0 : 1);
  }

  TIME_MSG("before starting main loop");
  ILOG("starting main loop");

  // Main loop: never returns.
  normal_enter(false, false);

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

/// Exit properly
void getout(int exitval)
  FUNC_ATTR_NORETURN
{
  assert(!ui_client_channel_id);
  exiting = true;

  // make sure startuptimes have been flushed
  time_finish();

  // On error during Ex mode, exit with a non-zero code.
  // POSIX requires this, although it's not 100% clear from the standard.
  if (exmode_active) {
    exitval += ex_exitval;
  }

  set_vim_var_type(VV_EXITING, VAR_NUMBER);
  set_vim_var_nr(VV_EXITING, exitval);

  // Invoked all deferred functions in the function stack.
  invoke_all_defer();

  // Optionally print hashtable efficiency.
  hash_debug_results();

  if (v_dying <= 1) {
    const tabpage_T *next_tp;

    // Trigger BufWinLeave for all windows, but only once per buffer.
    for (const tabpage_T *tp = first_tabpage; tp != NULL; tp = next_tp) {
      next_tp = tp->tp_next;
      FOR_ALL_WINDOWS_IN_TAB(wp, tp) {
        if (wp->w_buffer == NULL || !buf_valid(wp->w_buffer)) {
          // Autocmd must have close the buffer already, skip.
          continue;
        }

        buf_T *buf = wp->w_buffer;
        if (buf_get_changedtick(buf) != -1) {
          bufref_T bufref;

          set_bufref(&bufref, buf);
          apply_autocmds(EVENT_BUFWINLEAVE, buf->b_fname, buf->b_fname, false, buf);
          if (bufref_valid(&bufref)) {
            buf_set_changedtick(buf, -1);  // note that we did it already
          }
          // start all over, autocommands may mess up the lists
          next_tp = first_tabpage;
          break;
        }
      }
    }

    // Trigger BufUnload for buffers that are loaded
    FOR_ALL_BUFFERS(buf) {
      if (buf->b_ml.ml_mfp != NULL) {
        bufref_T bufref;
        set_bufref(&bufref, buf);
        apply_autocmds(EVENT_BUFUNLOAD, buf->b_fname, buf->b_fname, false, buf);
        if (!bufref_valid(&bufref)) {
          // Autocmd deleted the buffer.
          break;
        }
      }
    }

    int unblock = 0;
    // deathtrap() blocks autocommands, but we do want to trigger
    // VimLeavePre.
    if (is_autocmd_blocked()) {
      unblock_autocmds();
      unblock++;
    }
    apply_autocmds(EVENT_VIMLEAVEPRE, NULL, NULL, false, curbuf);
    if (unblock) {
      block_autocmds();
    }
  }

  if (
#ifdef EXITFREE
      !entered_free_all_mem &&
#endif
      p_shada && *p_shada != NUL) {
    // Write out the registers, history, marks etc, to the ShaDa file
    rs_shada_write_file(NULL, false);
  }

  if (v_dying <= 1) {
    int unblock = 0;

    // deathtrap() blocks autocommands, but we do want to trigger VimLeave.
    if (is_autocmd_blocked()) {
      unblock_autocmds();
      unblock++;
    }
    apply_autocmds(EVENT_VIMLEAVE, NULL, NULL, false, curbuf);
    if (unblock) {
      block_autocmds();
    }
  }

  profile_dump();

  if (did_emsg) {
    // give the user a chance to read the (error) message
    no_wait_return = false;
    // TODO(justinmk): this may call getout(0), clobbering exitval...
    wait_return(false);
  }

  // Apply 'titleold'.
  if (p_title && *p_titleold != NUL) {
    ui_call_set_title(cstr_as_string(p_titleold));
  }

  if (restarting) {
    Error err = ERROR_INIT;
    if (!remote_ui_restart(current_ui, &err)) {
      if (ERROR_SET(&err)) {
        ELOG("%s", err.msg);  // UI disappeared already?
        api_clear_error(&err);
      }
    }
    restarting = false;
  }

  if (garbage_collect_at_exit) {
    garbage_collect(false);
  }

#ifdef MSWIN
  // Restore Windows console icon before exiting.
  os_icon_reset();
  os_title_reset();
#endif

  os_exit(exitval);
}

/// Preserve files, print contents of `errmsg`, and exit 1.
/// @param errmsg  If NULL, this function will not print anything.
///
/// May be called from deadly_signal().
void preserve_exit(const char *errmsg)
  FUNC_ATTR_NORETURN
{
  // 'true' when we are sure to exit, e.g., after a deadly signal
  static bool really_exiting = false;

  // Prevent repeated calls into this method.
  if (really_exiting) {
    if (used_stdin) {
      // normalize stream (#2598)
      stream_set_blocking(STDIN_FILENO, true);
    }
    exit(2);
  }

  really_exiting = true;
  // Ignore SIGHUP while we are already exiting. #9274
  signal_reject_deadly();

  if (ui_client_channel_id) {
    // For TUI: exit alternate screen so that the error messages can be seen.
    ui_client_stop();
  }
  if (errmsg != NULL && errmsg[0] != NUL) {
    size_t has_eol = '\n' == errmsg[strlen(errmsg) - 1];
    fprintf(stderr, has_eol ? "%s" : "%s\n", errmsg);
  }
  if (ui_client_channel_id) {
    os_exit(1);
  }

  ml_close_notmod();                // close all not-modified buffers

  FOR_ALL_BUFFERS(buf) {
    if (buf->b_ml.ml_mfp != NULL && buf->b_ml.ml_mfp->mf_fname != NULL) {
      if (errmsg != NULL) {
        fprintf(stderr, "Nvim: preserving files...\n");
      }
      ml_sync_all(false, false, true);  // preserve all swap files
      break;
    }
  }

  ml_close_all(false);              // close all memfiles, without deleting

  if (errmsg != NULL) {
    fprintf(stderr, "Nvim: Finished.\n");
  }

  getout(1);
}

static uint64_t server_connect(char *server_addr, const char **errmsg)
{
  return rs_server_connect(server_addr, errmsg);
}

/// Handle remote subcommands
static void remote_request(mparm_T *params, int remote_args, char *server_addr, int argc,
                           char **argv, bool ui_only)
{
  bool is_ui = strequal(argv[remote_args], "--remote-ui");
  if (ui_only && !is_ui) {
    // TODO(bfredl): this implies always starting the TUI.
    // if we be smart we could delay this past should_exit
    return;
  }

  const char *connect_error = NULL;
  uint64_t chan = server_connect(server_addr, &connect_error);
  Object rvobj = OBJECT_INIT;

  if (is_ui) {
    if (!chan) {
      fprintf(stderr, "Remote ui failed to start: %s\n", connect_error);
      os_exit(1);
    } else if (strequal(server_addr, os_getenv_noalloc("NVIM"))) {
      fprintf(stderr, "%s", "Cannot attach UI of :terminal child to its parent. ");
      fprintf(stderr, "%s\n", "(Unset $NVIM to skip this check)");
      os_exit(1);
    }
    ui_client_channel_id = chan;
    return;
  }

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

  if (o.type == kObjectTypeDict) {
    rvobj.data.dict = o.data.dict;
  } else {
    fprintf(stderr, "vim._cs_remote returned unexpected value\n");
    os_exit(2);
  }

  TriState should_exit = kNone;
  TriState tabbed = kNone;

  for (size_t i = 0; i < rvobj.data.dict.size; i++) {
    if (strequal(rvobj.data.dict.items[i].key.data, "errmsg")) {
      if (rvobj.data.dict.items[i].value.type != kObjectTypeString) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'errmsg'\n");
        os_exit(2);
      }
      fprintf(stderr, "%s\n", rvobj.data.dict.items[i].value.data.string.data);
      os_exit(2);
    } else if (strequal(rvobj.data.dict.items[i].key.data, "result")) {
      if (rvobj.data.dict.items[i].value.type != kObjectTypeString) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'result'\n");
        os_exit(2);
      }
      printf("%s", rvobj.data.dict.items[i].value.data.string.data);
    } else if (strequal(rvobj.data.dict.items[i].key.data, "tabbed")) {
      if (rvobj.data.dict.items[i].value.type != kObjectTypeBoolean) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'tabbed'\n");
        os_exit(2);
      }
      tabbed = rvobj.data.dict.items[i].value.data.boolean ? kTrue : kFalse;
    } else if (strequal(rvobj.data.dict.items[i].key.data, "should_exit")) {
      if (rvobj.data.dict.items[i].value.type != kObjectTypeBoolean) {
        fprintf(stderr, "vim._cs_remote returned an unexpected type for 'should_exit'\n");
        os_exit(2);
      }
      should_exit = rvobj.data.dict.items[i].value.data.boolean ? kTrue : kFalse;
    }
  }
  if (should_exit == kNone || tabbed == kNone) {
    fprintf(stderr, "vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n");
    os_exit(2);
  }
  api_free_object(o);

  if (should_exit == kTrue) {
    os_exit(0);
  }
  if (tabbed == kTrue) {
    params->window_count = argc - remote_args - 1;
    params->window_layout = WIN_TABS;
  }
}


/// Read text from stdin.
static void read_stdin(void)
{
  rs_read_stdin();
}

// Create the requested number of windows and edit buffers in them.
// Also does recovery if "recoverymode" set.
static void create_windows(mparm_T *parmp)
{
  // Create the number of windows that was requested.
  if (parmp->window_count == -1) {      // was not set
    parmp->window_count = 1;
  }
  if (parmp->window_count == 0) {
    parmp->window_count = GARGCOUNT;
  }
  if (parmp->window_count > 1) {
    // Don't change the windows if there was a command in vimrc that
    // already split some windows
    if (parmp->window_layout == 0) {
      parmp->window_layout = WIN_HOR;
    }
    if (parmp->window_layout == WIN_TABS) {
      parmp->window_count = make_tabpages(parmp->window_count);
      TIME_MSG("making tab pages");
    } else if (firstwin->w_next == NULL || firstwin->w_next->w_floating) {
      parmp->window_count = make_windows(parmp->window_count, parmp->window_layout == WIN_VER);
      TIME_MSG("making windows");
    } else {
      parmp->window_count = rs_win_count();
    }
  } else {
    parmp->window_count = 1;
  }

  if (recoverymode) {                   // do recover
    msg_scroll = true;                  // scroll message up
    ml_recover(true);
    if (curbuf->b_ml.ml_mfp == NULL) {   // failed
      getout(1);
    }
    do_modelines(0);                    // do modelines
  } else {
    int done = 0;
    // Open a buffer for windows that don't have one yet.
    // Commands in the vimrc might have loaded a file or split the window.
    // Watch out for autocommands that delete a window.
    //
    // Don't execute Win/Buf Enter/Leave autocommands here
    autocmd_no_enter++;
    autocmd_no_leave++;
    bool dorewind = true;
    while (done++ < 1000) {
      if (dorewind) {
        if (parmp->window_layout == WIN_TABS) {
          goto_tabpage(1);
        } else {
          curwin = firstwin;
        }
      } else if (parmp->window_layout == WIN_TABS) {
        if (curtab->tp_next == NULL) {
          break;
        }
        goto_tabpage(0);
      } else {
        if (curwin->w_next == NULL) {
          break;
        }
        curwin = curwin->w_next;
      }
      dorewind = false;
      curbuf = curwin->w_buffer;
      if (curbuf->b_ml.ml_mfp == NULL) {
        // Set 'foldlevel' to 'foldlevelstart' if it's not negative..
        if (p_fdls >= 0) {
          curwin->w_p_fdl = p_fdls;
        }
        // When getting the ATTENTION prompt here, use a dialog.
        swap_exists_action = SEA_DIALOG;
        set_buflisted(true);

        // create memfile, read file
        open_buffer(false, NULL, 0);

        if (swap_exists_action == SEA_QUIT) {
          if (got_int || rs_only_one_window()) {
            // abort selected or quit and only one window
            did_emsg = false;               // avoid hit-enter prompt
            ui_call_error_exit(1);
            getout(1);
          }
          // We can't close the window, it would disturb what
          // happens next.  Clear the file name and set the arg
          // index to -1 to delete it later.
          setfname(curbuf, NULL, NULL, false);
          curwin->w_arg_idx = -1;
          swap_exists_action = SEA_NONE;
        } else {
          handle_swap_exists(NULL);
        }
        dorewind = true;                        // start again
      }
      os_breakcheck();
      if (got_int) {
        vgetc();          // only break the file loading, not the rest
        break;
      }
    }
    if (parmp->window_layout == WIN_TABS) {
      goto_tabpage(1);
    } else {
      curwin = firstwin;
    }
    curbuf = curwin->w_buffer;
    autocmd_no_enter--;
    autocmd_no_leave--;
  }
}

/// If opened more than one window, start editing files in the other
/// windows. make_windows() has already opened the windows.
static void edit_buffers(mparm_T *parmp, char *cwd)
{
  int arg_idx;                          // index in argument list
  bool advance = true;
  win_T *win;
  char *p_shm_save = NULL;

  // Don't execute Win/Buf Enter/Leave autocommands here
  autocmd_no_enter++;
  autocmd_no_leave++;

  // When w_arg_idx is -1 remove the window (see create_windows()).
  if (curwin->w_arg_idx == -1) {
    win_close(curwin, true, false);
    advance = false;
  }

  arg_idx = 1;
  for (int i = 1; i < parmp->window_count; i++) {
    if (cwd != NULL) {
      os_chdir(cwd);
    }
    // When w_arg_idx is -1 remove the window (see create_windows()).
    if (curwin->w_arg_idx == -1) {
      arg_idx++;
      win_close(curwin, true, false);
      advance = false;
      continue;
    }

    if (advance) {
      if (parmp->window_layout == WIN_TABS) {
        if (curtab->tp_next == NULL) {          // just checking
          break;
        }
        goto_tabpage(0);
        // Temporarily reset 'shm' option to not print fileinfo when
        // loading the other buffers. This would overwrite the already
        // existing fileinfo for the first tab.
        if (i == 1) {
          char buf[100];

          p_shm_save = xstrdup(p_shm);
          snprintf(buf, sizeof(buf), "F%s", p_shm);
          set_option_value_give_err(kOptShortmess, CSTR_AS_OPTVAL(buf), 0);
        }
      } else {
        if (curwin->w_next == NULL) {           // just checking
          break;
        }
        win_enter(curwin->w_next, false);
      }
    }
    advance = true;

    // Only open the file if there is no file in this window yet (that can
    // happen when vimrc contains ":sall").
    if (curbuf == firstwin->w_buffer || curbuf->b_ffname == NULL) {
      curwin->w_arg_idx = arg_idx;
      // Edit file from arg list, if there is one.  When "Quit" selected
      // at the ATTENTION prompt close the window.
      swap_exists_did_quit = false;
      do_ecmd(0, arg_idx < GARGCOUNT
              ? alist_name(&GARGLIST[arg_idx])
              : NULL, NULL, NULL, ECMD_LASTL, ECMD_HIDE, curwin);
      if (swap_exists_did_quit) {
        // abort or quit selected
        if (got_int || rs_only_one_window()) {
          // abort selected and only one window
          did_emsg = false;             // avoid hit-enter prompt
          ui_call_error_exit(1);
          getout(1);
        }
        win_close(curwin, true, false);
        advance = false;
      }
      if (arg_idx == GARGCOUNT - 1) {
        arg_had_last = true;
      }
      arg_idx++;
    }
    os_breakcheck();
    if (got_int) {
      vgetc();            // only break the file loading, not the rest
      break;
    }
  }

  if (p_shm_save != NULL) {
    set_option_value_give_err(kOptShortmess, CSTR_AS_OPTVAL(p_shm_save), 0);
    xfree(p_shm_save);
  }

  if (parmp->window_layout == WIN_TABS) {
    goto_tabpage(1);
  }
  autocmd_no_enter--;

  // make the first window the current window
  win = firstwin;
  // Avoid making a preview window the current window.
  while (win->w_p_pvw) {
    win = win->w_next;
    if (win == NULL) {
      win = firstwin;
      break;
    }
  }
  win_enter(win, false);

  autocmd_no_leave--;
  TIME_MSG("editing files in windows");
  if (parmp->window_count > 1 && parmp->window_layout != WIN_TABS) {
    rs_win_equal(curwin, 0, 'b');      // adjust heights
  }
}


#ifdef ENABLE_ASAN_UBSAN
const char *__ubsan_default_options(void) { return "print_stacktrace=1"; }

const char *__asan_default_options(void) { return "handle_abort=1,handle_sigill=1"; }
#endif
