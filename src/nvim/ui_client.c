/// Nvim's own UI client, which attaches to a child or remote Nvim server.

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/channel.h"
#include "nvim/channel_defs.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/event/multiqueue.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/highlight_defs.h"
#include "nvim/log.h"
#include "nvim/main.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/msgpack_rpc/channel_defs.h"
#include "nvim/os/os.h"
#include "nvim/profile.h"
#include "nvim/tui/tui.h"
#include "nvim/tui/tui_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_client.h"
#include "nvim/ui_defs.h"

#ifdef MSWIN
# include "nvim/os/os_win_console.h"
#endif

// Rust implementation in nvim-event crate
extern MultiQueue *rs_loop_get_fast_events(Loop *loop);
#define loop_get_fast_events(l) rs_loop_get_fast_events(l)

static TUIData *tui = NULL;
static int tui_width = 0;
static int tui_height = 0;
static char *tui_term = "";
static bool tui_rgb = false;
static bool ui_client_is_remote = false;

// uncrustify:off
#include "ui_client.c.generated.h"
#include "ui_events_client.generated.h"
// uncrustify:on

uint64_t ui_client_start_server(const char *exepath, size_t argc, char **argv)
{
  char **args = xmalloc((2 + argc) * sizeof(char *));
  int args_idx = 0;
  args[args_idx++] = xstrdup(argv[0]);
  args[args_idx++] = xstrdup("--embed");
  for (size_t i = 1; i < argc; i++) {
    args[args_idx++] = xstrdup(argv[i]);
  }
  args[args_idx++] = NULL;

  CallbackReader on_err = CALLBACK_READER_INIT;
  on_err.fwd_err = true;

#ifdef MSWIN
  // TODO(justinmk): detach breaks `tt.setup_child_nvim` tests on Windows?
  bool detach = os_env_exists("__NVIM_DETACH", true);
#else
  bool detach = true;
#endif
  varnumber_T exit_status;
  Channel *channel = channel_job_start(args, exepath,
                                       CALLBACK_READER_INIT, on_err, CALLBACK_NONE,
                                       false, true, true, detach, kChannelStdinPipe,
                                       NULL, 0, 0, NULL, &exit_status);
  if (!channel) {
    return 0;
  }

  // If stdin is not a pty, it is forwarded to the client.
  // Replace stdin in the TUI process with the tty fd.
  if (ui_client_forward_stdin) {
    close(0);
#ifdef MSWIN
    os_open_conin_fd();
#else
    int dupfd = dup(stderr_isatty ? STDERR_FILENO : STDOUT_FILENO);
    (void)dupfd;   // fd 0 reopened as lowest free fd; nothing further to do
#endif
  }

  return channel->id;
}

/// Attaches this client to the UI channel, and sets its client info.
void ui_client_attach(int width, int height, char *term, bool rgb)
{
  //
  // nvim_ui_attach
  //
  MAXSIZE_TEMP_ARRAY(args, 3);
  ADD_C(args, INTEGER_OBJ(width));
  ADD_C(args, INTEGER_OBJ(height));
  MAXSIZE_TEMP_DICT(opts, 9);
  PUT_C(opts, "rgb", BOOLEAN_OBJ(rgb));
  PUT_C(opts, "ext_linegrid", BOOLEAN_OBJ(true));
  PUT_C(opts, "ext_termcolors", BOOLEAN_OBJ(true));
  if (term) {
    PUT_C(opts, "term_name", CSTR_AS_OBJ(term));
  }
  PUT_C(opts, "term_colors", INTEGER_OBJ(t_colors));
  PUT_C(opts, "stdin_tty", BOOLEAN_OBJ(stdin_isatty));
  PUT_C(opts, "stdout_tty", BOOLEAN_OBJ(stdout_isatty));
  if (ui_client_forward_stdin) {
    PUT_C(opts, "stdin_fd", INTEGER_OBJ(UI_CLIENT_STDIN_FD));
    ui_client_forward_stdin = false;  // stdin shouldn't be forwarded again #22292
  }
  ADD_C(args, DICT_OBJ(opts));

  rpc_send_event(ui_client_channel_id, "nvim_ui_attach", args);
  ui_client_attached = true;

  TIME_MSG("nvim_ui_attach");

  //
  // nvim_set_client_info
  //
  MAXSIZE_TEMP_ARRAY(args2, 5);
  ADD_C(args2, CSTR_AS_OBJ("nvim-tui"));            // name
  Object m = api_metadata();
  Dict version = { 0 };
  assert(m.data.dict.size > 0);
  for (size_t i = 0; i < m.data.dict.size; i++) {
    if (strequal(m.data.dict.items[i].key.data, "version")) {
      version = m.data.dict.items[i].value.data.dict;
      break;
    } else if (i + 1 == m.data.dict.size) {
      abort();
    }
  }
  ADD_C(args2, DICT_OBJ(version));                  // version
  ADD_C(args2, CSTR_AS_OBJ("ui"));                  // type
  // We don't send api_metadata.functions as the "methods" because:
  // 1. it consumes memory.
  // 2. it is unlikely to be useful, since the peer can just call `nvim_get_api`.
  // 3. nvim_set_client_info expects a dict instead of an array.
  ADD_C(args2, ARRAY_OBJ((Array)ARRAY_DICT_INIT));  // methods
  MAXSIZE_TEMP_DICT(info, 9);                       // attributes
  PUT_C(info, "website", CSTR_AS_OBJ("https://neovim.io"));
  PUT_C(info, "license", CSTR_AS_OBJ("Apache 2"));
  PUT_C(info, "pid", INTEGER_OBJ(os_get_pid()));
  ADD_C(args2, DICT_OBJ(info));               // attributes
  rpc_send_event(ui_client_channel_id, "nvim_set_client_info", args2);

  TIME_MSG("nvim_set_client_info");
}

// C accessor: send nvim_ui_detach RPC event (used by Rust)
void nvim_uic_rpc_send_detach(void)
{
  rpc_send_event(ui_client_channel_id, "nvim_ui_detach", (Array)ARRAY_DICT_INIT);
}

// C accessor: check if tui is stopped (used by Rust)
bool nvim_uic_tui_is_stopped(void)
{
  return tui_is_stopped(tui);
}

// C accessor: stop the tui (used by Rust)
void nvim_uic_tui_stop(void)
{
  tui_stop(tui);
}

// C accessor: set api validation error (used by Rust)
void nvim_uic_api_set_error_validation(Error *err, const char *msg)
{
  api_set_error(err, kErrorTypeValidation, "%s", msg);
}

// C accessor: set ui_client_attached global (used by Rust)
void nvim_uic_set_attached(bool value)
{
  ui_client_attached = value;
}

// C accessor: set ui_client_error_exit global (used by Rust)
void nvim_uic_set_error_exit(int value)
{
  ui_client_error_exit = value;
}

// C accessor: set ui_client_channel_id global (used by Rust)
void nvim_uic_set_channel_id(uint64_t value)
{
  ui_client_channel_id = value;
}

// C accessor: queue a channel-connect event (used by Rust)
// Wraps multiqueue_put(fast_events, channel_connect_event, server_addr)
// since channel_connect_event is static and cannot be used from Rust.
void nvim_uic_queue_channel_connect(char *server_addr)
{
  multiqueue_put(loop_get_fast_events(&main_loop), channel_connect_event, server_addr);
}

// C accessor: call tui_grid_resize (used by Rust)
void nvim_uic_tui_grid_resize(Integer grid, Integer width, Integer height)
{
  tui_grid_resize(tui, grid, width, height);
}

// C accessor: reallocate grid_line buffers (used by Rust)
void nvim_uic_grid_line_buf_realloc(size_t new_size)
{
  xfree(grid_line_buf_char);
  xfree(grid_line_buf_attr);
  grid_line_buf_size = new_size;
  grid_line_buf_char = xmalloc(grid_line_buf_size * sizeof(schar_T));
  grid_line_buf_attr = xmalloc(grid_line_buf_size * sizeof(sattr_T));
}

// C accessor: get grid_line_buf_size (used by Rust)
size_t nvim_uic_get_grid_line_buf_size(void)
{
  return grid_line_buf_size;
}

// ui_client_detach: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

void ui_client_run(bool remote_ui)
  FUNC_ATTR_NORETURN
{
  ui_client_is_remote = remote_ui;
  tui_start(&tui, &tui_width, &tui_height, &tui_term, &tui_rgb);
  ui_client_attach(tui_width, tui_height, tui_term, tui_rgb);

  // TODO(justinmk): this is for log_spec. Can remove this after nvim_log #7062 is merged.
  if (os_env_exists("__NVIM_TEST_LOG", true)) {
    ELOG("test log message");
  }

  time_finish();

  // os_exit() will be invoked when the client channel detaches
  while (true) {
    LOOP_PROCESS_EVENTS(&main_loop, resize_events, -1);
  }
}

// ui_client_stop: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

// ui_client_set_size: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)
// ui_client_get_redraw_handler: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)
// handle_ui_client_redraw: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

static HlAttrs ui_client_dict2hlattrs(Dict d, bool rgb)
{
  Error err = ERROR_INIT;
  Dict(highlight) dict = KEYDICT_INIT;
  if (!api_dict_to_keydict(&dict, DictHash(highlight), d, &err)) {
    // TODO(bfredl): log "err"
    return HLATTRS_INIT;
  }

  HlAttrs attrs = dict2hlattrs(&dict, rgb, NULL, &err);

  if (HAS_KEY(&dict, highlight, url)) {
    attrs.url = tui_add_url(tui, dict.url.data);
  }

  return attrs;
}

// ui_client_event_grid_resize: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)
// ui_client_event_grid_line: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

void ui_client_event_raw_line(GridLineEvent *g)
{
  int grid = g->args[0];
  int row = g->args[1];
  int startcol = g->args[2];
  Integer endcol = startcol + g->coloff;
  Integer clearcol = endcol + g->clear_width;
  LineFlags lineflags = g->wrap ? kLineFlagWrap : 0;

  tui_raw_line(tui, grid, row, startcol, endcol, clearcol, g->cur_attr, lineflags,
               (const schar_T *)grid_line_buf_char, grid_line_buf_attr);
}

// ui_client_event_connect: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

static void channel_connect_event(void **argv)
{
  char *server_addr = argv[0];

  const char *err = "";
  bool is_tcp = !!strrchr(server_addr, ':');
  CallbackReader on_data = CALLBACK_READER_INIT;
  uint64_t chan = channel_connect(is_tcp, server_addr, true, on_data, 50, &err);

  if (!strequal(err, "")) {
    ELOG("Cannot connect to server %s: %s", server_addr, err);
    ui_client_exit_status = 1;
    os_exit(1);
  }

  ui_client_channel_id = chan;
  ui_client_is_remote = true;
  ui_client_attach(tui_width, tui_height, tui_term, tui_rgb);

  ILOG("Connected to server %s on channel %" PRId64, server_addr, chan);
}

/// When a "restart" UI event is received, its arguments are saved here when
/// waiting for the server to exit.
static Array restart_args = ARRAY_DICT_INIT;
static bool restart_pending = false;

// C accessor: save restart args and set pending flag (used by Rust)
void nvim_uic_save_restart_args(Array args)
{
  api_free_array(restart_args);
  restart_args = copy_array(args, NULL);
  restart_pending = true;
}

// C accessor: get ui_client_attached global (used by Rust)
bool nvim_uic_get_attached(void)
{
  return ui_client_attached;
}

// C accessor: set tui_width and tui_height (used by Rust)
void nvim_uic_set_tui_size(int width, int height)
{
  tui_width = width;
  tui_height = height;
}

// C accessor: send nvim_ui_try_resize RPC event (used by Rust)
// Handles MAXSIZE_TEMP_ARRAY/ADD_C/INTEGER_OBJ macros which can't be used from Rust.
void nvim_uic_send_resize(int width, int height)
{
  MAXSIZE_TEMP_ARRAY(args, 2);
  ADD_C(args, INTEGER_OBJ((int)width));
  ADD_C(args, INTEGER_OBJ((int)height));
  rpc_send_event(ui_client_channel_id, "nvim_ui_try_resize", args);
}

// C accessor: look up redraw event handler by name hash (used by Rust)
UIClientHandler nvim_uic_handler_hash_lookup(const char *name, size_t name_len)
{
  int hash = ui_client_handler_hash(name, name_len);
  if (hash < 0) {
    return (UIClientHandler){ NULL, NULL };
  }
  return event_handlers[hash];
}

// ui_client_event_restart: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

/// Called when the current server has exited.
void ui_client_may_restart_server(void)
{
  if (!restart_pending) {
    return;
  }
  restart_pending = false;

  size_t argc;
  char **argv = NULL;
  if (restart_args.size < 2
      || restart_args.items[0].type != kObjectTypeString
      || restart_args.items[1].type != kObjectTypeArray
      || (argc = restart_args.items[1].data.array.size) < 1) {
    ELOG("Error handling ui event 'restart'");
    goto cleanup;
  }

  // 1. Get executable path and command-line arguments.
  const char *exepath = restart_args.items[0].data.string.data;
  argv = xcalloc(argc + 1, sizeof(char *));
  for (size_t i = 0; i < argc; i++) {
    if (restart_args.items[1].data.array.items[i].type == kObjectTypeString) {
      argv[i] = restart_args.items[1].data.array.items[i].data.string.data;
    }
    if (argv[i] == NULL) {
      argv[i] = "";
    }
  }

  // 2. Start a new `nvim --embed` server.
  uint64_t rv = ui_client_start_server(exepath, argc, argv);
  if (!rv) {
    ELOG("failed to start nvim server");
    goto cleanup;
  }

  // 3. Client-side server re-attach.
  ui_client_channel_id = rv;
  ui_client_is_remote = false;
  ui_client_attach(tui_width, tui_height, tui_term, tui_rgb);

  ILOG("restarted server id=%" PRId64, rv);
cleanup:
  xfree(argv);
  api_free_array(restart_args);
  restart_args = (Array)ARRAY_DICT_INIT;
}

// ui_client_event_error_exit: implemented in Rust (src/nvim-rs/ui_client/src/events.rs)

#ifdef EXITFREE
void ui_client_free_all_mem(void)
{
  tui_free_all_mem(tui);
  xfree(grid_line_buf_char);
  xfree(grid_line_buf_attr);
}
#endif
