#include <assert.h>
#include <fcntl.h>
#include <inttypes.h>
#include <lauxlib.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/channel.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/rstream.h"
#include "nvim/event/socket.h"
#include "nvim/event/stream.h"
#include "nvim/event/wstream.h"
#include "nvim/ex_cmds.h"
#include "nvim/garray.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/lua/executor.h"
#include "nvim/main.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/msgpack_rpc/server.h"
#include "nvim/os/fs.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"

#ifdef MSWIN
# include "nvim/os/fs.h"
# include "nvim/os/os_win_console.h"
# include "nvim/os/pty_conpty_win.h"
#endif

static bool did_stdio = false;

/// next free id for a job or rpc channel
/// 1 is reserved for stdio channel
/// 2 is reserved for stderr channel
static uint64_t next_chan_id = CHAN_STDERR + 1;

#include "channel.c.generated.h"

// Rust implementations in channel crate (src/nvim-rs/channel/src/lib.rs)
extern void free_channel_event(void **argv);
extern void close_cb(Stream *stream, void *data);
extern size_t on_channel_data(RStream *stream, const char *buf, size_t count, void *data, bool eof);
extern size_t on_job_stderr(RStream *stream, const char *buf, size_t count, void *data, bool eof);
extern void channel_proc_exit_cb(Proc *proc, int status, void *data);

// Rust implementation in nvim-event crate
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern int rs_stream_is_closed(Stream *stream);
extern int rs_proc_get_status(Proc *proc);
extern int rs_proc_get_type(Proc *proc);
extern void rs_proc_set_detach(Proc *proc, int detach);
extern void rs_proc_set_events(Proc *proc, MultiQueue *events);
extern void rs_proc_set_argv(Proc *proc, char **argv);
extern void rs_proc_set_exepath(Proc *proc, const char *exepath);
extern void rs_proc_set_cwd(Proc *proc, const char *cwd);
extern dict_T *rs_proc_get_env(Proc *proc);
extern void rs_proc_set_env(Proc *proc, dict_T *env);
extern int rs_proc_get_fwd_err(Proc *proc);
extern void rs_proc_set_fwd_err(Proc *proc, int fwd_err);
extern int rs_proc_get_overlapped(Proc *proc);
extern void rs_proc_set_overlapped(Proc *proc, int overlapped);
#define stream_is_closed(s) rs_stream_is_closed(s)
#define proc_get_status(p) rs_proc_get_status(p)
#define proc_get_type(p) rs_proc_get_type(p)
#define proc_set_detach(p, d) rs_proc_set_detach(p, d)
#define proc_set_events(p, e) rs_proc_set_events(p, e)
#define proc_set_argv(p, a) rs_proc_set_argv(p, a)
#define proc_set_exepath(p, e) rs_proc_set_exepath(p, e)
#define proc_set_cwd(p, c) rs_proc_set_cwd(p, c)
#define proc_get_env(p) rs_proc_get_env(p)
#define proc_set_env(p, e) rs_proc_set_env(p, e)
#define proc_get_fwd_err(p) rs_proc_get_fwd_err(p)
#define proc_set_fwd_err(p, f) rs_proc_set_fwd_err(p, f)
#define proc_get_overlapped(p) rs_proc_get_overlapped(p)
#define proc_set_overlapped(p, o) rs_proc_set_overlapped(p, o)
extern void rs_proc_set_cb(Proc *proc, void *cb);
#define proc_set_cb(p, c) rs_proc_set_cb(p, (void *)(c))
// Stream accessors for proc->in/out
extern size_t rs_stream_get_pending_reqs(Stream *stream);
#define stream_get_pending_reqs(s) rs_stream_get_pending_reqs(s)
// Loop accessors
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define loop_get_events(l) rs_loop_get_events(l)

// channel_teardown implemented in Rust (src/nvim-rs/channel/src/lib.rs)

#ifdef EXITFREE
void channel_free_all_mem(void)
{
  Channel *chan;
  map_foreach_value(&channels, chan, {
    channel_destroy(chan);
  });
  map_destroy(uint64_t, &channels);

  callback_free(&on_print);
}
#endif

// channel_close implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_init implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_alloc implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// Accessors for next_chan_id (static in this file) used by Rust channel_alloc
uint64_t nvim_chan_get_next_chan_id(void) { return next_chan_id; }
void nvim_chan_set_next_chan_id(uint64_t v) { next_chan_id = v; }
void nvim_chan_map_put(uint64_t id, Channel *chan) { pmap_put(uint64_t)(&channels, id, chan); }

// channel_create_event implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_incref, channel_decref, callback_reader_free, callback_reader_start
// implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_destroy (static helper), free_channel_event, close_cb
// implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_destroy_early implemented in Rust (src/nvim-rs/channel/src/lib.rs)

/// Starts a job and returns the associated channel
///
/// @param[in]  argv  Arguments vector specifying the command to run,
///                   NULL-terminated
/// @param[in]  exepath  The path to the executable. If NULL, use `argv[0]`.
/// @param[in]  on_stdout  Callback to read the job's stdout
/// @param[in]  on_stderr  Callback to read the job's stderr
/// @param[in]  on_exit  Callback to receive the job's exit status
/// @param[in]  pty  True if the job should run attached to a pty
/// @param[in]  rpc  True to communicate with the job using msgpack-rpc,
///                  `on_stdout` is ignored
/// @param[in]  detach  True if the job should not be killed when nvim exits,
///                     ignored if `pty` is true
/// @param[in]  stdin_mode  Stdin mode. Either kChannelStdinPipe to open a
///                         channel for stdin or kChannelStdinNull to leave
///                         stdin disconnected.
/// @param[in]  cwd  Initial working directory for the job.  Nvim's working
///                  directory if `cwd` is NULL
/// @param[in]  pty_width  Width of the pty, ignored if `pty` is false
/// @param[in]  pty_height  Height of the pty, ignored if `pty` is false
/// @param[in]  env  Nvim's configured environment is used if this is NULL,
///                  otherwise defines all environment variables
/// @param[out]  status_out  0 for invalid arguments, > 0 for the channel id,
///                          < 0 if the job can't start
///
/// @returns [allocated] channel
Channel *channel_job_start(char **argv, const char *exepath, CallbackReader on_stdout,
                           CallbackReader on_stderr, Callback on_exit, bool pty, bool rpc,
                           bool overlapped, bool detach, ChannelStdinMode stdin_mode,
                           const char *cwd, uint16_t pty_width, uint16_t pty_height, dict_T *env,
                           varnumber_T *status_out)
{
  Channel *chan = channel_alloc(kChannelStreamProc);
  chan->on_data = on_stdout;
  chan->on_stderr = on_stderr;
  chan->on_exit = on_exit;

  if (pty) {
    if (detach) {
      semsg(_(e_invarg2), "terminal/pty job cannot be detached");
      shell_free_argv(argv);
      if (env) {
        tv_dict_free(env);
      }
      channel_destroy_early(chan);
      *status_out = 0;
      return NULL;
    }
    chan->stream.pty = pty_proc_init(&main_loop, chan);
    if (pty_width > 0) {
      chan->stream.pty.width = pty_width;
    }
    if (pty_height > 0) {
      chan->stream.pty.height = pty_height;
    }
  } else {
    chan->stream.uv = libuv_proc_init(&main_loop, chan);
  }

  Proc *proc = &chan->stream.proc;
  proc_set_argv(proc, argv);
  proc_set_exepath(proc, exepath);
  proc_set_cb(proc, channel_proc_exit_cb);
  proc_set_events(proc, chan->events);
  proc_set_detach(proc, detach);
  proc_set_cwd(proc, cwd);
  proc_set_env(proc, env);
  proc_set_overlapped(proc, overlapped);

  char *cmd = xstrdup(proc_get_exepath(proc));
  bool has_out, has_err;
  if (proc_get_type(proc) == kProcTypePty) {
    has_out = true;
    has_err = false;
  } else {
    has_out = rpc || callback_reader_set(chan->on_data);
    has_err = callback_reader_set(chan->on_stderr);
    proc_set_fwd_err(proc, chan->on_stderr.fwd_err);
  }

  bool has_in = stdin_mode == kChannelStdinPipe;

  int status = proc_spawn(proc, has_in, has_out, has_err);
  if (status) {
    semsg(_(e_jobspawn), os_strerror(status), cmd);
    xfree(cmd);
    if (proc_get_env(proc)) {
      tv_dict_free(proc_get_env(proc));
    }
    channel_destroy_early(chan);
    *status_out = proc_get_status(proc);
    return NULL;
  }
  xfree(cmd);
  if (proc_get_env(proc)) {
    tv_dict_free(proc_get_env(proc));
  }

  if (has_in) {
    wstream_init(&proc->in, 0);
  }
  if (has_out) {
    rstream_init(&proc->out);
  }

  if (rpc) {
    // the rpc takes over the in and out streams
    rpc_start(chan);
  } else {
    if (has_out) {
      callback_reader_start(&chan->on_data, "stdout");
      rstream_start(&proc->out, on_channel_data, chan);
    }
  }

  if (has_err) {
    callback_reader_start(&chan->on_stderr, "stderr");
    rstream_init(&proc->err);
    rstream_start(&proc->err, on_job_stderr, chan);
  }

  *status_out = (varnumber_T)chan->id;
  return chan;
}

uint64_t channel_connect(bool tcp, const char *address, bool rpc, CallbackReader on_output,
                         int timeout, const char **error)
{
  Channel *channel;

  if (!tcp && rpc) {
    if (server_owns_pipe_address(address)) {
      // Create a loopback channel. This avoids deadlock if nvim connects to
      // its own named pipe.
      channel = channel_alloc(kChannelStreamInternal);
      channel->stream.internal.cb = LUA_NOREF;
      rpc_start(channel);
      goto end;
    }
  }

  channel = channel_alloc(kChannelStreamSocket);
  if (!socket_connect(&main_loop, &channel->stream.socket,
                      tcp, address, timeout, error)) {
    channel_destroy_early(channel);
    return 0;
  }

  channel->stream.socket.s.internal_close_cb = close_cb;
  channel->stream.socket.s.internal_data = channel;
  wstream_init(&channel->stream.socket.s, 0);
  rstream_init(&channel->stream.socket);

  if (rpc) {
    rpc_start(channel);
  } else {
    channel->on_data = on_output;
    callback_reader_start(&channel->on_data, "data");
    rstream_start(&channel->stream.socket, on_channel_data, channel);
  }

end:
  channel_create_event(channel, address);
  return channel->id;
}

// channel_from_connection implemented in Rust (src/nvim-rs/channel/src/lib.rs)

/// Creates an API channel from stdin/stdout. Used when embedding Nvim.
uint64_t channel_from_stdio(bool rpc, CallbackReader on_output, const char **error)
  FUNC_ATTR_NONNULL_ALL
{
  if (!headless_mode && !embedded_mode) {
    *error = _("can only be opened in headless mode");
    return 0;
  }

  if (did_stdio) {
    *error = _("channel was already open");
    return 0;
  }
  did_stdio = true;

  Channel *channel = channel_alloc(kChannelStreamStdio);

  int stdin_dup_fd = STDIN_FILENO;
  int stdout_dup_fd = STDOUT_FILENO;
#ifdef MSWIN
  // Strangely, ConPTY doesn't work if stdin and stdout are pipes. So replace
  // stdin and stdout with CONIN$ and CONOUT$, respectively.
  if (embedded_mode && os_has_conpty_working()) {
    stdin_dup_fd = os_dup(STDIN_FILENO);
    os_replace_stdin_to_conin();
    stdout_dup_fd = os_dup(STDOUT_FILENO);
    os_replace_stdout_and_stderr_to_conout();
  }
#else
  if (embedded_mode) {
    // Redirect stdout/stdin (the UI channel) to stderr. Use fnctl(F_DUPFD_CLOEXEC) instead of dup()
    // to prevent child processes from inheriting the file descriptors, which are used by UIs to
    // detect when Nvim exits.
    stdin_dup_fd = fcntl(STDIN_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1);
    stdout_dup_fd = fcntl(STDOUT_FILENO, F_DUPFD_CLOEXEC, STDERR_FILENO + 1);
    dup2(STDERR_FILENO, STDOUT_FILENO);
    dup2(STDERR_FILENO, STDIN_FILENO);
  }
#endif
  rstream_init_fd(&main_loop, &channel->stream.stdio.in, stdin_dup_fd);
  wstream_init_fd(&main_loop, &channel->stream.stdio.out, stdout_dup_fd, 0);

  if (rpc) {
    rpc_start(channel);
  } else {
    channel->on_data = on_output;
    callback_reader_start(&channel->on_data, "stdin");
    rstream_start(&channel->stream.stdio.in, on_channel_data, channel);
  }

  return channel->id;
}

// channel_send implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// buffer_to_tv_list, on_channel_data, on_job_stderr, on_channel_output (static),
// schedule_channel_event (static), on_channel_event (static), channel_reader_callbacks,
// channel_proc_exit_cb (static), channel_callback_call (static)
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_terminal_open, term_write, term_resize, term_delayed_free, term_close
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)
extern void term_delayed_free(void **argv);

// channel_info_changed implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// set_info_event implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_info implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// int64_t_cmp implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_all_info implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// f_prompt_setcallback, f_prompt_setinterrupt, f_prompt_setprompt
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)

