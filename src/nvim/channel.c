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

/// Teardown the module
void channel_teardown(void)
{
  Channel *chan;
  map_foreach_value(&channels, chan, {
    channel_close(chan->id, kChannelPartAll, NULL);
  });
}

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

/// Closes a channel
///
/// @param id The channel id
/// @return true if successful, false otherwise
bool channel_close(uint64_t id, ChannelPart part, const char **error)
{
  Channel *chan;
  Proc *proc;

  const char *dummy;
  if (!error) {
    error = &dummy;
  }

  if (!(chan = find_channel(id))) {
    if (id < next_chan_id) {
      // allow double close, even though we can't say what parts was valid.
      return true;
    }
    *error = e_invchan;
    return false;
  }

  bool close_main = false;
  if (part == kChannelPartRpc || part == kChannelPartAll) {
    close_main = true;
    if (chan->is_rpc) {
      rpc_close(chan);
    } else if (part == kChannelPartRpc) {
      *error = e_invstream;
      return false;
    }
  } else if ((part == kChannelPartStdin || part == kChannelPartStdout)
             && chan->is_rpc) {
    *error = e_invstreamrpc;
    return false;
  }

  switch (chan->streamtype) {
  case kChannelStreamSocket:
    if (!close_main) {
      *error = e_invstream;
      return false;
    }
    rstream_may_close(&chan->stream.socket);
    break;

  case kChannelStreamProc:
    proc = &chan->stream.proc;
    if (part == kChannelPartStdin || close_main) {
      stream_may_close(&proc->in);
    }
    if (part == kChannelPartStdout || close_main) {
      rstream_may_close(&proc->out);
    }
    if (part == kChannelPartStderr || part == kChannelPartAll) {
      rstream_may_close(&proc->err);
    }
    if (proc_get_type(proc) == kProcTypePty && part == kChannelPartAll) {
      pty_proc_close_master(&chan->stream.pty);
    }

    break;

  case kChannelStreamStdio:
    if (part == kChannelPartStdin || close_main) {
      rstream_may_close(&chan->stream.stdio.in);
    }
    if (part == kChannelPartStdout || close_main) {
      stream_may_close(&chan->stream.stdio.out);
    }
    if (part == kChannelPartStderr) {
      *error = e_invstream;
      return false;
    }
    break;

  case kChannelStreamStderr:
    if (part != kChannelPartAll && part != kChannelPartStderr) {
      *error = e_invstream;
      return false;
    }
    if (!chan->stream.err.closed) {
      chan->stream.err.closed = true;
      // Don't close on exit, in case late error messages
      if (!exiting) {
        fclose(stderr);
      }
      channel_decref(chan);
    }
    break;

  case kChannelStreamInternal:
    if (!close_main) {
      *error = e_invstream;
      return false;
    }
    if (chan->term) {
      api_free_luaref(chan->stream.internal.cb);
      chan->stream.internal.cb = LUA_NOREF;
      chan->stream.internal.closed = true;
      terminal_close(&chan->term, 0);
    } else {
      channel_decref(chan);
    }
    break;
  }

  return true;
}

// channel_init implemented in Rust (src/nvim-rs/channel/src/lib.rs)

/// Allocates a channel.
///
/// Channel is allocated with refcount 1, which should be decreased
/// when the underlying stream closes.
Channel *channel_alloc(ChannelStreamType type)
  FUNC_ATTR_NONNULL_RET
{
  Channel *chan = xcalloc(1, sizeof(*chan));
  if (type == kChannelStreamStdio) {
    chan->id = CHAN_STDIO;
  } else if (type == kChannelStreamStderr) {
    chan->id = CHAN_STDERR;
  } else {
    chan->id = next_chan_id++;
  }
  chan->events = multiqueue_new_child(loop_get_events(&main_loop));
  chan->refcount = 1;
  chan->exit_status = -1;
  chan->streamtype = type;
  chan->detach = false;
  assert(chan->id <= VARNUMBER_MAX);
  pmap_put(uint64_t)(&channels, chan->id, chan);
  return chan;
}

void channel_create_event(Channel *chan, const char *ext_source)
{
#ifdef NVIM_LOG_DEBUG
  const char *source;

  if (ext_source) {
    // TODO(bfredl): in a future improved traceback solution,
    // external events should be included.
    source = ext_source;
  } else {
    eval_fmt_source_name_line(IObuff, sizeof(IObuff));
    source = IObuff;
  }

  assert(chan->id <= VARNUMBER_MAX);
  Arena arena = ARENA_EMPTY;
  Dict info = channel_info(chan->id, &arena);
  typval_T tv = TV_INITIAL_VALUE;
  // TODO(bfredl): do the conversion in one step. Also would be nice
  // to pretty print top level dict in defined order
  object_to_vim(DICT_OBJ(info), &tv, NULL);
  assert(tv.v_type == VAR_DICT);
  char *str = encode_tv2json(&tv, NULL);
  ILOG("new channel %" PRIu64 " (%s) : %s", chan->id, source, str);
  xfree(str);
  arena_mem_free(arena_finish(&arena));

#else
  (void)ext_source;
#endif

  channel_info_changed(chan, true);
}

// channel_incref, channel_decref, callback_reader_free, callback_reader_start
// implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_destroy (static helper), free_channel_event, close_cb
// implemented in Rust (src/nvim-rs/channel/src/lib.rs)

static void channel_destroy_early(Channel *chan)
{
  if ((chan->id != --next_chan_id)) {
    abort();
  }
  pmap_del(uint64_t)(&channels, chan->id, NULL);
  chan->id = 0;

  if ((--chan->refcount != 0)) {
    abort();
  }

  // uv will keep a reference to handles until next loop tick, so delay free
  multiqueue_put(loop_get_events(&main_loop), free_channel_event, chan);
}

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

/// Creates an RPC channel from a tcp/pipe socket connection
///
/// @param watcher The SocketWatcher ready to accept the connection
void channel_from_connection(SocketWatcher *watcher)
{
  Channel *channel = channel_alloc(kChannelStreamSocket);
  socket_watcher_accept(watcher, &channel->stream.socket);
  channel->stream.socket.s.internal_close_cb = close_cb;
  channel->stream.socket.s.internal_data = channel;
  wstream_init(&channel->stream.socket.s, 0);
  rstream_init(&channel->stream.socket);
  rpc_start(channel);
  channel_create_event(channel, watcher->addr);
}

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

/// @param data will be consumed
size_t channel_send(uint64_t id, char *data, size_t len, bool data_owned, const char **error)
  FUNC_ATTR_NONNULL_ALL
{
  Channel *chan = find_channel(id);
  size_t written = 0;
  if (!chan) {
    *error = _(e_invchan);
    goto retfree;
  }

  if (chan->streamtype == kChannelStreamStderr) {
    if (chan->stream.err.closed) {
      *error = _("Can't send data to closed stream");
      goto retfree;
    }
    // unbuffered write
    ptrdiff_t wres = os_write(STDERR_FILENO, data, len, false);
    if (wres >= 0) {
      written = (size_t)wres;
    }
    goto retfree;
  }

  if (chan->streamtype == kChannelStreamInternal) {
    if (chan->is_rpc) {
      *error = _("Can't send raw data to rpc channel");
      goto retfree;
    }
    if (!chan->term || chan->stream.internal.closed) {
      *error = _("Can't send data to closed stream");
      goto retfree;
    }
    terminal_receive(chan->term, data, len);
    written = len;
    goto retfree;
  }

  Stream *in = channel_instream(chan);
  if (stream_is_closed(in)) {
    *error = _("Can't send data to closed stream");
    goto retfree;
  }

  if (chan->is_rpc) {
    *error = _("Can't send raw data to rpc channel");
    goto retfree;
  }

  // write can be delayed indefinitely, so always use an allocated buffer
  WBuffer *buf = wstream_new_buffer(data_owned ? data : xmemdup(data, len),
                                    len, 1, xfree);
  return wstream_write(in, buf) ? len : 0;

retfree:
  if (data_owned) {
    xfree(data);
  }
  return written;
}

// buffer_to_tv_list, on_channel_data, on_job_stderr, on_channel_output (static),
// schedule_channel_event (static), on_channel_event (static), channel_reader_callbacks,
// channel_proc_exit_cb (static), channel_callback_call (static)
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)

// channel_terminal_open, term_write, term_resize, term_delayed_free, term_close
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)
extern void term_delayed_free(void **argv);

// channel_info_changed implemented in Rust (src/nvim-rs/channel/src/lib.rs)

void set_info_event(void **argv)
{
  Channel *chan = argv[0];
  event_T event = (event_T)(ptrdiff_t)argv[1];

  save_v_event_T save_v_event;
  dict_T *dict = get_v_event(&save_v_event);
  Arena arena = ARENA_EMPTY;
  Dict info = channel_info(chan->id, &arena);
  typval_T retval;
  object_to_vim(DICT_OBJ(info), &retval, NULL);
  assert(retval.v_type == VAR_DICT);
  tv_dict_add_dict(dict, S_LEN("info"), retval.vval.v_dict);
  tv_dict_set_keys_readonly(dict);

  apply_autocmds(event, NULL, NULL, true, curbuf);

  restore_v_event(dict, &save_v_event);
  arena_mem_free(arena_finish(&arena));
  channel_decref(chan);
}


Dict channel_info(uint64_t id, Arena *arena)
{
  Channel *chan = find_channel(id);
  if (!chan) {
    return (Dict)ARRAY_DICT_INIT;
  }

  Dict info = arena_dict(arena, 8);
  PUT_C(info, "id", INTEGER_OBJ((Integer)chan->id));

  const char *stream_desc, *mode_desc;
  switch (chan->streamtype) {
  case kChannelStreamProc: {
    stream_desc = "job";
    if (chan->stream.proc.type == kProcTypePty) {
      const char *name = pty_proc_tty_name(&chan->stream.pty);
      PUT_C(info, "pty", CSTR_TO_ARENA_OBJ(arena, name));
    }

    char **args = chan->stream.proc.argv;
    Array argv = ARRAY_DICT_INIT;
    if (args != NULL) {
      size_t n;
      for (n = 0; args[n] != NULL; n++) {}
      argv = arena_array(arena, n);
      for (size_t i = 0; i < n; i++) {
        ADD_C(argv, CSTR_AS_OBJ(args[i]));
      }
    }
    PUT_C(info, "argv", ARRAY_OBJ(argv));
    break;
  }

  case kChannelStreamStdio:
    stream_desc = "stdio";
    break;

  case kChannelStreamStderr:
    stream_desc = "stderr";
    break;

  case kChannelStreamInternal:
    PUT_C(info, "internal", BOOLEAN_OBJ(true));
    FALLTHROUGH;

  case kChannelStreamSocket:
    stream_desc = "socket";
    break;
  }
  PUT_C(info, "stream", CSTR_AS_OBJ(stream_desc));

  if (chan->is_rpc) {
    mode_desc = "rpc";
    PUT_C(info, "client", DICT_OBJ(chan->rpc.info));
  } else if (chan->term) {
    mode_desc = "terminal";
    PUT_C(info, "buffer", BUFFER_OBJ(terminal_buf(chan->term)));
  } else {
    mode_desc = "bytes";
  }
  PUT_C(info, "mode", CSTR_AS_OBJ(mode_desc));

  return info;
}

// int64_t_cmp implemented in Rust (src/nvim-rs/channel/src/lib.rs)

Array channel_all_info(Arena *arena)
{
  // order the items in the array by channel number, for Determinism™
  kvec_t(int64_t) ids = KV_INITIAL_VALUE;
  kv_fixsize_arena(arena, ids, map_size(&channels));
  uint64_t id;
  map_foreach_key(&channels, id, {
    kv_push(ids, (int64_t)id);
  });
  qsort(ids.items, ids.size, sizeof ids.items[0], int64_t_cmp);

  Array ret = arena_array(arena, ids.size);
  for (size_t i = 0; i < ids.size; i++) {
    ADD_C(ret, DICT_OBJ(channel_info((uint64_t)ids.items[i], arena)));
  }
  return ret;
}

// f_prompt_setcallback, f_prompt_setinterrupt, f_prompt_setprompt
// -- all implemented in Rust (src/nvim-rs/channel/src/lib.rs)

