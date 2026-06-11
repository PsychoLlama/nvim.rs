#include <assert.h>
#include <inttypes.h>
#include <signal.h>
#include <string.h>
#include <uv.h>

#include "klib/kvec.h"
#include "nvim/channel.h"
#include "nvim/event/libuv_proc.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/rstream.h"
#include "nvim/event/stream.h"
#include "nvim/event/wstream.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/memory_defs.h"
#include "nvim/os/proc.h"
#include "nvim/os/pty_proc.h"
#include "nvim/os/shell.h"
#include "nvim/os/time.h"
#include "nvim/ui_client.h"

#include "event/proc.c.generated.h"

// Rust implementation in nvim-event crate
extern int rs_multiqueue_empty(MultiQueue *mq);
#define multiqueue_empty(mq) rs_multiqueue_empty(mq)
extern int rs_rstream_is_closed(RStream *stream);
#define rstream_is_closed(s) rs_rstream_is_closed(s)
extern size_t rs_rstream_num_bytes(RStream *stream);
#define rstream_num_bytes(s) rs_rstream_num_bytes(s)
extern int rs_rstream_did_eof(RStream *stream);
#define rstream_did_eof(s) rs_rstream_did_eof(s)
extern int rs_proc_is_closed(Proc *proc);
#define proc_is_closed(p) rs_proc_is_closed(p)
extern void rs_proc_set_closed(Proc *proc, int closed);
#define proc_set_closed(p, c) rs_proc_set_closed(p, c)
extern int rs_proc_get_status(Proc *proc);
#define proc_get_status(p) rs_proc_get_status(p)
extern void rs_proc_set_status(Proc *proc, int status);
#define proc_set_status(p, s) rs_proc_set_status(p, s)
extern uint64_t rs_proc_get_stopped_time(Proc *proc);
#define proc_get_stopped_time(p) rs_proc_get_stopped_time(p)
extern int rs_proc_get_pid(Proc *proc);
#define proc_get_pid(p) rs_proc_get_pid(p)
extern void rs_proc_set_pid(Proc *proc, int pid);
#define proc_set_pid(p, pid) rs_proc_set_pid(p, pid)
extern int rs_proc_get_refcount(Proc *proc);
#define proc_get_refcount(p) rs_proc_get_refcount(p)
extern void rs_proc_incref(Proc *proc);
#define proc_incref(p) rs_proc_incref(p)
extern int rs_proc_decref(Proc *proc);
#define proc_decref(p) rs_proc_decref(p)
extern int rs_proc_get_type(Proc *proc);
#define proc_get_type(p) rs_proc_get_type(p)
extern int rs_proc_get_detach(Proc *proc);
#define proc_get_detach(p) rs_proc_get_detach(p)
extern void rs_proc_set_detach(Proc *proc, int detach);
#define proc_set_detach(p, d) rs_proc_set_detach(p, d)
extern MultiQueue *rs_proc_get_events(Proc *proc);
#define proc_get_events(p) rs_proc_get_events(p)
extern void rs_proc_set_events(Proc *proc, MultiQueue *events);
#define proc_set_events(p, e) rs_proc_set_events(p, e)
extern Loop *rs_proc_get_loop(Proc *proc);
#define proc_get_loop(p) rs_proc_get_loop(p)
extern char **rs_proc_get_argv(Proc *proc);
extern void rs_proc_set_argv(Proc *proc, char **argv);
#define proc_get_argv(p) rs_proc_get_argv(p)
#define proc_set_argv(p, a) rs_proc_set_argv(p, a)
extern const char *rs_proc_get_exepath_raw(Proc *proc);
extern void rs_proc_set_exepath(Proc *proc, const char *exepath);
#define proc_get_exepath_raw(p) rs_proc_get_exepath_raw(p)
#define proc_set_exepath(p, e) rs_proc_set_exepath(p, e)
extern const char *rs_proc_get_cwd(Proc *proc);
extern void rs_proc_set_cwd(Proc *proc, const char *cwd);
#define proc_get_cwd(p) rs_proc_get_cwd(p)
#define proc_set_cwd(p, c) rs_proc_set_cwd(p, c)
extern dict_T *rs_proc_get_env(Proc *proc);
extern void rs_proc_set_env(Proc *proc, dict_T *env);
#define proc_get_env(p) rs_proc_get_env(p)
#define proc_set_env(p, e) rs_proc_set_env(p, e)
extern void rs_proc_set_stopped_time(Proc *proc, uint64_t stopped_time);
#define proc_set_stopped_time(p, t) rs_proc_set_stopped_time(p, t)
extern uint8_t rs_proc_get_exit_signal(Proc *proc);
extern void rs_proc_set_exit_signal(Proc *proc, uint8_t sig);
#define proc_get_exit_signal(p) rs_proc_get_exit_signal(p)
#define proc_set_exit_signal(p, s) rs_proc_set_exit_signal(p, s)
extern int rs_proc_get_fwd_err(Proc *proc);
extern void rs_proc_set_fwd_err(Proc *proc, int fwd_err);
#define proc_get_fwd_err(p) rs_proc_get_fwd_err(p)
#define proc_set_fwd_err(p, f) rs_proc_set_fwd_err(p, f)
extern int rs_proc_get_overlapped(Proc *proc);
extern void rs_proc_set_overlapped(Proc *proc, int overlapped);
#define proc_get_overlapped(p) rs_proc_get_overlapped(p)
#define proc_set_overlapped(p, o) rs_proc_set_overlapped(p, o)
// Callback accessors - use void* for FFI compatibility
extern void *rs_proc_get_cb(Proc *proc);
extern void rs_proc_set_cb(Proc *proc, void *cb);
// ISO C forbids fn-ptr<->void* conversion; static inline wraps the pragma.
static inline proc_exit_cb proc_get_cb(Proc *p)
{
  PRAGMA_DIAG_PUSH_IGNORE_PEDANTIC
  return (proc_exit_cb)rs_proc_get_cb(p);
  PRAGMA_DIAG_POP
}
#define proc_set_cb(p, c) rs_proc_set_cb(p, (void *)(c))
extern void *rs_proc_get_internal_exit_cb(Proc *proc);
extern void rs_proc_set_internal_exit_cb(Proc *proc, void *cb);
static inline internal_proc_cb proc_get_internal_exit_cb(Proc *p)
{
  PRAGMA_DIAG_PUSH_IGNORE_PEDANTIC
  return (internal_proc_cb)rs_proc_get_internal_exit_cb(p);
  PRAGMA_DIAG_POP
}
static inline void proc_set_internal_exit_cb(Proc *p, internal_proc_cb c)
{
  PRAGMA_DIAG_PUSH_IGNORE_PEDANTIC
  rs_proc_set_internal_exit_cb(p, (void *)(c));
  PRAGMA_DIAG_POP
}
extern void *rs_proc_get_internal_close_cb(Proc *proc);
extern void rs_proc_set_internal_close_cb(Proc *proc, void *cb);
static inline internal_proc_cb proc_get_internal_close_cb(Proc *p)
{
  PRAGMA_DIAG_PUSH_IGNORE_PEDANTIC
  return (internal_proc_cb)rs_proc_get_internal_close_cb(p);
  PRAGMA_DIAG_POP
}
static inline void proc_set_internal_close_cb(Proc *p, internal_proc_cb c)
{
  PRAGMA_DIAG_PUSH_IGNORE_PEDANTIC
  rs_proc_set_internal_close_cb(p, (void *)(c));
  PRAGMA_DIAG_POP
}
extern void rs_proc_call_cb(Proc *proc, int status, void *data);
#define proc_call_cb(p, s, d) rs_proc_call_cb(p, s, d)
extern void rs_proc_call_internal_exit_cb(Proc *proc);
#define proc_call_internal_exit_cb(p) rs_proc_call_internal_exit_cb(p)
extern void rs_proc_call_internal_close_cb(Proc *proc);
#define proc_call_internal_close_cb(p) rs_proc_call_internal_close_cb(p)
// Loop accessors
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define loop_get_events(l) rs_loop_get_events(l)
extern MultiQueue *rs_loop_get_fast_events(Loop *loop);
#define loop_get_fast_events(l) rs_loop_get_fast_events(l)
// Stream accessors for proc->in/out/err
extern void rs_stream_set_internal_data(Stream *stream, void *data);
#define stream_set_internal_data(s, d) rs_stream_set_internal_data(s, d)
extern void rs_stream_set_internal_close_cb(Stream *stream, void *cb);
static inline void stream_set_internal_close_cb(Stream *s, stream_close_cb c)
{
  PRAGMA_DIAG_PUSH_IGNORE_PEDANTIC
  rs_stream_set_internal_close_cb(s, (void *)(c));
  PRAGMA_DIAG_POP
}
extern void rs_stream_set_closed(Stream *stream, int closed);
#define stream_set_closed(s, c) rs_stream_set_closed(s, c)
extern MultiQueue *rs_stream_get_events(Stream *stream);
#define stream_get_events(s) rs_stream_get_events(s)
extern void *rs_stream_get_cb_data(Stream *stream);
#define stream_get_cb_data(s) rs_stream_get_cb_data(s)

// Time for a process to exit cleanly before we send KILL.
// For PTY processes SIGTERM is sent first (in case SIGHUP was not enough).
#define KILL_TIMEOUT_MS 2000

/// Externally defined with gcov.
#ifdef USE_GCOV
void __gcov_flush(void);
#endif

static bool proc_is_tearing_down = false;

// Delay exit until handles are closed, to avoid deadlocks
static int exit_need_delay = 0;

/// @returns zero on success, or negative error code
int proc_spawn(Proc *proc, bool in, bool out, bool err)
  FUNC_ATTR_NONNULL_ALL
{
  // forwarding stderr contradicts with processing it internally
  assert(!(err && proc_get_fwd_err(proc)));

  if (in) {
    uv_pipe_init(&proc_get_loop(proc)->uv, &proc->in.uv.pipe, 0);
  } else {
    stream_set_closed(&proc->in, true);
  }

  if (out) {
    uv_pipe_init(&proc_get_loop(proc)->uv, &proc->out.s.uv.pipe, 0);
  } else {
    stream_set_closed(&proc->out.s, true);
  }

  if (err) {
    uv_pipe_init(&proc_get_loop(proc)->uv, &proc->err.s.uv.pipe, 0);
  } else {
    stream_set_closed(&proc->err.s, true);
  }

#ifdef USE_GCOV
  // Flush coverage data before forking, to avoid "Merge mismatch" errors.
  __gcov_flush();
#endif

  int status;
  switch (proc_get_type(proc)) {
  case kProcTypeUv:
    status = libuv_proc_spawn((LibuvProc *)proc);
    break;
  case kProcTypePty:
    status = pty_proc_spawn((PtyProc *)proc);
    break;
  }

  if (status) {
    if (in) {
      uv_close((uv_handle_t *)&proc->in.uv.pipe, NULL);
    }
    if (out) {
      uv_close((uv_handle_t *)&proc->out.s.uv.pipe, NULL);
    }
    if (err) {
      uv_close((uv_handle_t *)&proc->err.s.uv.pipe, NULL);
    }

    if (proc_get_type(proc) == kProcTypeUv) {
      uv_close((uv_handle_t *)&(((LibuvProc *)proc)->uv), NULL);
    } else {
      proc_close(proc);
    }
    proc_free(proc);
    proc_set_status(proc, -1);
    return status;
  }

  if (in) {
    stream_init(NULL, &proc->in, -1, (uv_stream_t *)&proc->in.uv.pipe);
    stream_set_internal_data(&proc->in, proc);
    stream_set_internal_close_cb(&proc->in, on_proc_stream_close);
    proc_incref(proc);
  }

  if (out) {
    stream_init(NULL, &proc->out.s, -1, (uv_stream_t *)&proc->out.s.uv.pipe);
    stream_set_internal_data(&proc->out.s, proc);
    stream_set_internal_close_cb(&proc->out.s, on_proc_stream_close);
    proc_incref(proc);
  }

  if (err) {
    stream_init(NULL, &proc->err.s, -1, (uv_stream_t *)&proc->err.s.uv.pipe);
    stream_set_internal_data(&proc->err.s, proc);
    stream_set_internal_close_cb(&proc->err.s, on_proc_stream_close);
    proc_incref(proc);
  }

  proc_set_internal_exit_cb(proc, on_proc_exit);
  proc_set_internal_close_cb(proc, decref);
  proc_incref(proc);
  kv_push(proc_get_loop(proc)->children, proc);
  DLOG("new: pid=%d exepath=[%s]", proc_get_pid(proc), proc_get_exepath(proc));
  return 0;
}

void proc_teardown(Loop *loop) FUNC_ATTR_NONNULL_ALL
{
  proc_is_tearing_down = true;
  for (size_t i = 0; i < kv_size(loop->children); i++) {
    Proc *proc = kv_A(loop->children, i);
    if (proc_get_detach(proc) || proc_get_type(proc) == kProcTypePty) {
      // Close handles to process without killing it.
      CREATE_EVENT(loop_get_events(loop), proc_close_handles, proc);
    } else {
      proc_stop(proc);
    }
  }

  // Wait until all children exit and all close events are processed.
  LOOP_PROCESS_EVENTS_UNTIL(loop, loop_get_events(loop), -1,
                            kv_size(loop->children) == 0 && multiqueue_empty(loop_get_events(loop)));
  pty_proc_teardown(loop);
}

void proc_close_streams(Proc *proc) FUNC_ATTR_NONNULL_ALL
{
  stream_may_close(&proc->in);
  rstream_may_close(&proc->out);
  rstream_may_close(&proc->err);
}

/// Synchronously wait for a process to finish
///
/// @param process  Process instance
/// @param ms       Time in milliseconds to wait for the process.
///                 0 for no wait. -1 to wait until the process quits.
/// @return Exit code of the process. proc->status will have the same value.
///         -1 if the timeout expired while the process is still running.
///         -2 if the user interrupted the wait.
int proc_wait(Proc *proc, int ms, MultiQueue *events)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (!proc_get_refcount(proc)) {
    int status = proc_get_status(proc);
    LOOP_PROCESS_EVENTS(proc_get_loop(proc), proc_get_events(proc), 0);
    return status;
  }

  if (!events) {
    events = proc_get_events(proc);
  }

  // Increase refcount to stop the exit callback from being called (and possibly
  // freed) before we have a chance to get the status.
  proc_incref(proc);
  LOOP_PROCESS_EVENTS_UNTIL(proc_get_loop(proc), events, ms,
                            // Until...
                            got_int                       // interrupted by the user
                            || proc_get_refcount(proc) == 1);  // job exited

  // Assume that a user hitting CTRL-C does not like the current job.  Kill it.
  if (got_int) {
    got_int = false;
    proc_stop(proc);
    if (ms == -1) {
      // We can only return if all streams/handles are closed and the job
      // exited.
      LOOP_PROCESS_EVENTS_UNTIL(proc_get_loop(proc), events, -1,
                                proc_get_refcount(proc) == 1);
    } else {
      LOOP_PROCESS_EVENTS(proc_get_loop(proc), events, 0);
    }

    proc_set_status(proc, -2);
  }

  if (proc_get_refcount(proc) == 1) {
    // Job exited, free its resources.
    decref(proc);
    if (proc_get_events(proc)) {
      // decref() created an exit event, process it now.
      multiqueue_process_events(proc_get_events(proc));
    }
  } else {
    proc_decref(proc);
  }

  return proc_get_status(proc);
}

/// Ask a process to terminate and eventually kill if it doesn't respond
void proc_stop(Proc *proc) FUNC_ATTR_NONNULL_ALL
{
  bool exited = (proc_get_status(proc) >= 0);
  if (exited || proc_get_stopped_time(proc)) {
    return;
  }
  proc_set_stopped_time(proc, os_hrtime());
  proc_set_exit_signal(proc, SIGTERM);

  switch (proc_get_type(proc)) {
  case kProcTypeUv:
    os_proc_tree_kill(proc_get_pid(proc), SIGTERM);
    break;
  case kProcTypePty:
    // close all streams for pty processes to send SIGHUP to the process
    proc_close_streams(proc);
    pty_proc_close_master((PtyProc *)proc);
    break;
  }

  // (Re)start timer to verify that stopped process(es) died.
  uv_timer_start(&proc_get_loop(proc)->children_kill_timer, children_kill_cb,
                 KILL_TIMEOUT_MS, 0);
}

/// Frees process-owned resources.
void proc_free(Proc *proc) FUNC_ATTR_NONNULL_ALL
{
  if (proc_get_argv(proc) != NULL) {
    shell_free_argv(proc_get_argv(proc));
    proc_set_argv(proc, NULL);
  }
}

/// Sends SIGKILL (or SIGTERM..SIGKILL for PTY jobs) to processes that did
/// not terminate after proc_stop().
static void children_kill_cb(uv_timer_t *handle)
{
  Loop *loop = handle->loop->data;

  for (size_t i = 0; i < kv_size(loop->children); i++) {
    Proc *proc = kv_A(loop->children, i);
    bool exited = (proc_get_status(proc) >= 0);
    if (exited || !proc_get_stopped_time(proc)) {
      continue;
    }
    uint64_t term_sent = UINT64_MAX == proc_get_stopped_time(proc);
    if (kProcTypePty != proc_get_type(proc) || term_sent) {
      proc_set_exit_signal(proc, SIGKILL);
      os_proc_tree_kill(proc_get_pid(proc), SIGKILL);
    } else {
      proc_set_exit_signal(proc, SIGTERM);
      os_proc_tree_kill(proc_get_pid(proc), SIGTERM);
      proc_set_stopped_time(proc, UINT64_MAX);  // Flag: SIGTERM was sent.
      // Restart timer.
      uv_timer_start(&proc_get_loop(proc)->children_kill_timer, children_kill_cb,
                     KILL_TIMEOUT_MS, 0);
    }
  }
}

static void proc_close_event(void **argv)
{
  Proc *proc = argv[0];
  if (proc_get_cb(proc)) {
    // User (hint: channel_job_start) is responsible for calling
    // proc_free().
    proc_call_cb(proc, proc_get_status(proc), proc->data);
  } else {
    proc_free(proc);
  }
}

static void decref(Proc *proc)
{
  if (proc_decref(proc) != 0) {
    return;
  }

  Loop *loop = proc_get_loop(proc);
  size_t i;
  for (i = 0; i < kv_size(loop->children); i++) {
    Proc *current = kv_A(loop->children, i);
    if (current == proc) {
      break;
    }
  }
  assert(i < kv_size(loop->children));  // element found
  if (i < kv_size(loop->children) - 1) {
    memmove(&kv_A(loop->children, i), &kv_A(loop->children, i + 1),
            sizeof(&kv_A(loop->children, i)) * (kv_size(loop->children) - (i + 1)));
  }
  kv_size(loop->children)--;
  CREATE_EVENT(proc_get_events(proc), proc_close_event, proc);
}

static void proc_close(Proc *proc)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (proc_is_tearing_down && proc_is_closed(proc) && (proc_get_detach(proc) || proc_get_type(proc) == kProcTypePty)) {
    // If a detached/pty process dies while tearing down it might get closed twice.
    return;
  }
  assert(!proc_is_closed(proc));
  proc_set_closed(proc, 1);

  if (proc_get_detach(proc)) {
    if (proc_get_type(proc) == kProcTypeUv) {
      uv_unref((uv_handle_t *)&(((LibuvProc *)proc)->uv));
    }
  }

  switch (proc_get_type(proc)) {
  case kProcTypeUv:
    libuv_proc_close((LibuvProc *)proc);
    break;
  case kProcTypePty:
    pty_proc_close((PtyProc *)proc);
    break;
  }
}

/// Flush output stream.
///
/// @param proc     Process, for which an output stream should be flushed.
/// @param stream   Stream to flush.
static void flush_stream(Proc *proc, RStream *stream)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (!stream || rstream_is_closed(stream)) {
    return;
  }

  // Maximal remaining data size of terminated process is system
  // buffer size.
  // Also helps with a child process that keeps the output streams open. If it
  // keeps sending data, we only accept as much data as the system buffer size.
  // Otherwise this would block cleanup/teardown.
  int system_buffer_size = 0;
  int err = uv_recv_buffer_size((uv_handle_t *)&stream->s.uv.pipe,
                                &system_buffer_size);
  if (err) {
    system_buffer_size = ARENA_BLOCK_SIZE;
  }

  size_t max_bytes = rstream_num_bytes(stream) + (size_t)system_buffer_size;

  // Read remaining data.
  while (!rstream_is_closed(stream) && rstream_num_bytes(stream) < max_bytes) {
    // Remember number of bytes before polling
    size_t num_bytes = rstream_num_bytes(stream);

    // Poll for data and process the generated events.
    loop_poll_events(proc_get_loop(proc), 0);
    if (stream_get_events(&stream->s)) {
      multiqueue_process_events(stream_get_events(&stream->s));
    }

    // Stream can be closed if it is empty.
    if (num_bytes == rstream_num_bytes(stream)) {
      if (stream->read_cb && !rstream_did_eof(stream)) {
        // Stream callback could miss EOF handling if a child keeps the stream
        // open. But only send EOF if we haven't already.
        stream->read_cb(stream, stream->buffer, 0, stream_get_cb_data(&stream->s), true);
      }
      break;
    }
  }
}

static void proc_close_handles(void **argv)
{
  Proc *proc = argv[0];

  exit_need_delay++;
  flush_stream(proc, &proc->out);
  flush_stream(proc, &proc->err);

  proc_close_streams(proc);
  proc_close(proc);
  exit_need_delay--;
}

static void exit_delay_cb(uv_timer_t *handle)
{
  uv_timer_stop(&main_loop.exit_delay_timer);
  multiqueue_put(loop_get_fast_events(&main_loop), exit_event, main_loop.exit_delay_timer.data);
}

static void exit_event(void **argv)
{
  int status = (int)(intptr_t)argv[0];
  if (exit_need_delay) {
    main_loop.exit_delay_timer.data = argv[0];
    uv_timer_start(&main_loop.exit_delay_timer, exit_delay_cb, 0, 0);
    return;
  }

  if (!exiting) {
    if (ui_client_channel_id) {
      ui_client_exit_status = status;
      os_exit(status);
    } else {
      assert(status == 0);  // Called from rpc_close(), which passes 0 as status.
      preserve_exit(NULL);
    }
  }
}

/// Performs self-exit because the primary RPC channel was closed.
void exit_on_closed_chan(int status)
{
  DLOG("self-exit triggered by closed RPC channel...");
  multiqueue_put(loop_get_fast_events(&main_loop), exit_event, (void *)(intptr_t)status);
}

static void on_proc_exit(Proc *proc)
{
  Loop *loop = proc_get_loop(proc);
  ILOG("child exited: pid=%d status=%d" PRIu64, proc_get_pid(proc), proc_get_status(proc));

  // TODO(justinmk): figure out why rpc_close sometimes(??) isn't called.
  // Theories:
  // - EOF not received in receive_msgpack, then doesn't call chan_close_on_err().
  // - proc_close_handles not tickled by ui_client.c's LOOP_PROCESS_EVENTS?
  if (ui_client_channel_id) {
    uint64_t server_chan_id = ui_client_channel_id;
    Channel *server_chan = find_channel(server_chan_id);
    if (server_chan != NULL && server_chan->streamtype == kChannelStreamProc
        && proc == &server_chan->stream.proc) {
      // Need to call ui_client_may_restart_server() here as well, as sometimes
      // rpc_close_event() hasn't been called yet (also see comments above).
      ui_client_may_restart_server();
      if (ui_client_channel_id == server_chan_id) {
        // If the current embedded server has exited and no new server is started,
        // the client should exit with the same status.
        exit_on_closed_chan(proc_get_status(proc));
      }
    }
  }

  // Process has terminated, but there could still be data to be read from the
  // OS. We are still in the libuv loop, so we cannot call code that polls for
  // more data directly. Instead delay the reading after the libuv loop by
  // queueing proc_close_handles() as an event.
  MultiQueue *queue = proc_get_events(proc) ? proc_get_events(proc) : loop_get_events(loop);
  CREATE_EVENT(queue, proc_close_handles, proc);
}

static void on_proc_stream_close(Stream *stream, void *data)
{
  Proc *proc = data;
  decref(proc);
}

// Rust accessor functions for opaque handle pattern

/// Get the status field from a Proc (accessor for Rust).
int nvim_proc_get_status(Proc *proc) { return proc->status; }

/// Get the stopped_time field from a Proc (accessor for Rust).
uint64_t nvim_proc_get_stopped_time(Proc *proc) { return proc->stopped_time; }

/// Get the pid field from a Proc (accessor for Rust).
int nvim_proc_get_pid(Proc *proc) { return proc->pid; }

/// Set the pid field of a Proc (accessor for Rust).
void nvim_proc_set_pid(Proc *proc, int pid) { proc->pid = pid; }

/// Get the refcount field from a Proc (accessor for Rust).
int nvim_proc_get_refcount(Proc *proc) { return proc->refcount; }

/// Check if a Proc is closed (accessor for Rust).
int nvim_proc_is_closed(Proc *proc) { return proc->closed ? 1 : 0; }

/// Set the status field of a Proc (accessor for Rust).
void nvim_proc_set_status(Proc *proc, int status) { proc->status = status; }

/// Get the loop field from a Proc (accessor for Rust).
Loop *nvim_proc_get_loop(Proc *proc) { return proc->loop; }

/// Get the type field from a Proc (accessor for Rust).
int nvim_proc_get_type(Proc *proc) { return (int)proc->type; }

/// Get the detach field from a Proc (accessor for Rust).
int nvim_proc_get_detach(Proc *proc) { return proc->detach ? 1 : 0; }

/// Set the detach field of a Proc (accessor for Rust).
void nvim_proc_set_detach(Proc *proc, int detach) { proc->detach = detach != 0; }

/// Get the events queue from a Proc (accessor for Rust).
MultiQueue *nvim_proc_get_events(Proc *proc) { return proc->events; }

/// Set the events queue of a Proc (accessor for Rust).
void nvim_proc_set_events(Proc *proc, MultiQueue *events) { proc->events = events; }

/// Set the closed field of a Proc (accessor for Rust).
void nvim_proc_set_closed(Proc *proc, int closed) { proc->closed = closed != 0; }

/// Increment the refcount of a Proc (accessor for Rust).
void nvim_proc_incref(Proc *proc) { proc->refcount++; }

/// Decrement the refcount of a Proc and return the new value (accessor for Rust).
int nvim_proc_decref(Proc *proc) { return --proc->refcount; }

/// Get the argv field from a Proc (accessor for Rust).
char **nvim_proc_get_argv(Proc *proc) { return proc->argv; }

/// Set the argv field of a Proc (accessor for Rust).
void nvim_proc_set_argv(Proc *proc, char **argv) { proc->argv = argv; }

/// Get the executable path for a Proc (accessor for Rust).
/// Falls back to argv[0] when exepath is NULL, matching proc_get_exepath() in proc.h.
const char *nvim_proc_get_exepath(Proc *proc)
{
  return proc->exepath != NULL ? proc->exepath : proc->argv[0];
}

/// Set the exepath field of a Proc (accessor for Rust).
void nvim_proc_set_exepath(Proc *proc, const char *exepath) { proc->exepath = exepath; }

/// Get the cwd field from a Proc (accessor for Rust).
const char *nvim_proc_get_cwd(Proc *proc) { return proc->cwd; }

/// Set the cwd field of a Proc (accessor for Rust).
void nvim_proc_set_cwd(Proc *proc, const char *cwd) { proc->cwd = cwd; }

/// Get the env field from a Proc (accessor for Rust).
dict_T *nvim_proc_get_env(Proc *proc) { return proc->env; }

/// Set the env field of a Proc (accessor for Rust).
void nvim_proc_set_env(Proc *proc, dict_T *env) { proc->env = env; }

/// Set the stopped_time field of a Proc (accessor for Rust).
void nvim_proc_set_stopped_time(Proc *proc, uint64_t stopped_time) { proc->stopped_time = stopped_time; }

/// Get the exit_signal field from a Proc (accessor for Rust).
uint8_t nvim_proc_get_exit_signal(Proc *proc) { return proc->exit_signal; }

/// Set the exit_signal field of a Proc (accessor for Rust).
void nvim_proc_set_exit_signal(Proc *proc, uint8_t exit_signal) { proc->exit_signal = exit_signal; }

/// Get the fwd_err field from a Proc (accessor for Rust).
int nvim_proc_get_fwd_err(Proc *proc) { return proc->fwd_err ? 1 : 0; }

/// Set the fwd_err field of a Proc (accessor for Rust).
void nvim_proc_set_fwd_err(Proc *proc, int fwd_err) { proc->fwd_err = fwd_err != 0; }

/// Get the overlapped field from a Proc (accessor for Rust).
int nvim_proc_get_overlapped(Proc *proc) { return proc->overlapped ? 1 : 0; }

/// Set the overlapped field of a Proc (accessor for Rust).
void nvim_proc_set_overlapped(Proc *proc, int overlapped) { proc->overlapped = overlapped != 0; }

/// Get the cb (exit callback) field of a Proc (accessor for Rust).
proc_exit_cb nvim_proc_get_cb(Proc *proc) { return proc->cb; }

/// Set the cb (exit callback) field of a Proc (accessor for Rust).
void nvim_proc_set_cb(Proc *proc, proc_exit_cb cb) { proc->cb = cb; }

/// Get the internal_exit_cb field of a Proc (accessor for Rust).
internal_proc_cb nvim_proc_get_internal_exit_cb(Proc *proc) { return proc->internal_exit_cb; }

/// Set the internal_exit_cb field of a Proc (accessor for Rust).
void nvim_proc_set_internal_exit_cb(Proc *proc, internal_proc_cb cb) { proc->internal_exit_cb = cb; }

/// Get the internal_close_cb field of a Proc (accessor for Rust).
internal_proc_cb nvim_proc_get_internal_close_cb(Proc *proc) { return proc->internal_close_cb; }

/// Set the internal_close_cb field of a Proc (accessor for Rust).
void nvim_proc_set_internal_close_cb(Proc *proc, internal_proc_cb cb) { proc->internal_close_cb = cb; }

/// Call proc->cb if set (accessor for Rust).
void nvim_proc_call_cb(Proc *proc, int status, void *data)
{
  if (proc->cb) {
    proc->cb(proc, status, data);
  }
}

/// Call proc->internal_exit_cb (accessor for Rust).
void nvim_proc_call_internal_exit_cb(Proc *proc)
{
  if (proc->internal_exit_cb) {
    proc->internal_exit_cb(proc);
  }
}

/// Call proc->internal_close_cb if set (accessor for Rust).
void nvim_proc_call_internal_close_cb(Proc *proc)
{
  if (proc->internal_close_cb) {
    proc->internal_close_cb(proc);
  }
}
