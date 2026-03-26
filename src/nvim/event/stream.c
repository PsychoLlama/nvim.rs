#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <uv.h>
#include <uv/version.h>

#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/stream.h"
#include "nvim/log.h"
#include "nvim/memory.h"
#include "nvim/types_defs.h"
#ifdef MSWIN
# include "nvim/os/os_win_console.h"
#endif

#include "event/stream.c.generated.h"

// Rust implementation in nvim-event crate
extern int rs_stream_is_closed(Stream *stream);
#define stream_is_closed(s) rs_stream_is_closed(s)
extern int rs_stream_get_fd(Stream *stream);
#define stream_get_fd(s) rs_stream_get_fd(s)
extern void rs_stream_set_closed(Stream *stream, int closed);
#define stream_set_closed(s, c) rs_stream_set_closed(s, c)
extern size_t rs_stream_get_pending_reqs(Stream *stream);
#define stream_get_pending_reqs(s) rs_stream_get_pending_reqs(s)
extern void *rs_stream_get_internal_data(Stream *stream);
extern void rs_stream_set_internal_data(Stream *stream, void *data);
#define stream_get_internal_data(s) rs_stream_get_internal_data(s)
#define stream_set_internal_data(s, d) rs_stream_set_internal_data(s, d)
extern void *rs_stream_get_internal_close_cb(Stream *stream);
extern void rs_stream_set_internal_close_cb(Stream *stream, void *cb);
#define stream_get_internal_close_cb(s) ((stream_close_cb)rs_stream_get_internal_close_cb(s))
#define stream_set_internal_close_cb(s, c) rs_stream_set_internal_close_cb(s, (void *)(c))
extern void rs_stream_call_internal_close_cb(Stream *stream);
#define stream_call_internal_close_cb(s) rs_stream_call_internal_close_cb(s)
extern void *rs_stream_get_close_cb(Stream *stream);
extern void rs_stream_set_close_cb(Stream *stream, void *cb);
extern void *rs_stream_get_close_cb_data(Stream *stream);
extern void rs_stream_set_close_cb_data(Stream *stream, void *data);
#define stream_get_close_cb(s) ((stream_close_cb)rs_stream_get_close_cb(s))
#define stream_set_close_cb(s, c) rs_stream_set_close_cb(s, (void *)(c))
#define stream_get_close_cb_data(s) rs_stream_get_close_cb_data(s)
#define stream_set_close_cb_data(s, d) rs_stream_set_close_cb_data(s, d)
extern void rs_stream_call_close_cb(Stream *stream);
#define stream_call_close_cb(s) rs_stream_call_close_cb(s)
// Additional stream field accessors
extern void rs_stream_set_curmem(Stream *stream, size_t curmem);
#define stream_set_curmem(s, c) rs_stream_set_curmem(s, c)
extern void rs_stream_set_maxmem(Stream *stream, size_t maxmem);
#define stream_set_maxmem(s, m) rs_stream_set_maxmem(s, m)
extern void rs_stream_set_pending_reqs(Stream *stream, size_t pending_reqs);
#define stream_set_pending_reqs(s, p) rs_stream_set_pending_reqs(s, p)
extern void rs_stream_set_write_cb(Stream *stream, void *cb);
#define stream_set_write_cb(s, c) rs_stream_set_write_cb(s, (void *)(c))
extern void rs_stream_set_events(Stream *stream, MultiQueue *events);
#define stream_set_events(s, e) rs_stream_set_events(s, e)
extern void rs_stream_set_fpos(Stream *stream, int64_t fpos);
#define stream_set_fpos(s, f) rs_stream_set_fpos(s, f)

// For compatibility with libuv < 1.19.0 (tested on 1.18.0)
#if UV_VERSION_MINOR < 19
# define uv_stream_get_write_queue_size(stream) stream->write_queue_size
#endif

/// Sets the stream associated with `fd` to "blocking" mode.
///
/// @return `0` on success, or libuv error code on failure.
int stream_set_blocking(int fd, bool blocking)
{
  // Private loop to avoid conflict with existing watcher(s):
  //    uv__io_stop: Assertion `loop->watchers[w->fd] == w' failed.
  uv_loop_t loop;
  uv_pipe_t stream;
  uv_loop_init(&loop);
  uv_pipe_init(&loop, &stream, 0);
  uv_pipe_open(&stream, fd);
  int retval = uv_stream_set_blocking((uv_stream_t *)&stream, blocking);
  uv_close((uv_handle_t *)&stream, NULL);
  uv_run(&loop, UV_RUN_NOWAIT);  // not necessary, but couldn't hurt.
  uv_loop_close(&loop);
  return retval;
}

void stream_init(Loop *loop, Stream *stream, int fd, uv_stream_t *uvstream)
  FUNC_ATTR_NONNULL_ARG(2)
{
  // The underlying stream is either a file or an existing uv stream.
  assert(uvstream == NULL ? fd >= 0 : fd < 0);
  stream->uvstream = uvstream;

  if (fd >= 0) {
    uv_handle_type type = uv_guess_handle(fd);
    stream->fd = fd;

    if (type == UV_FILE) {
      // Non-blocking file reads are simulated with an idle handle that reads in
      // chunks of the ring buffer size, giving time for other events to be
      // processed between reads.
      uv_idle_init(&loop->uv, &stream->uv.idle);
      stream->uv.idle.data = stream;
    } else {
      assert(type == UV_NAMED_PIPE || type == UV_TTY);
#ifdef MSWIN
      if (type == UV_TTY) {
        uv_tty_init(&loop->uv, &stream->uv.tty, fd, 0);
        uv_tty_set_mode(&stream->uv.tty, UV_TTY_MODE_RAW);
        DWORD dwMode;
        if (GetConsoleMode(stream->uv.tty.handle, &dwMode)) {
          dwMode |= ENABLE_VIRTUAL_TERMINAL_INPUT;
          SetConsoleMode(stream->uv.tty.handle, dwMode);
        }
        stream->uvstream = (uv_stream_t *)&stream->uv.tty;
      } else {
#endif
      uv_pipe_init(&loop->uv, &stream->uv.pipe, 0);
      uv_pipe_open(&stream->uv.pipe, fd);
      stream->uvstream = (uv_stream_t *)&stream->uv.pipe;
#ifdef MSWIN
    }
#endif
    }
  }

  if (stream->uvstream) {
    stream->uvstream->data = stream;
  }

  stream_set_fpos(stream, 0);
  stream_set_internal_data(stream, NULL);
  stream_set_curmem(stream, 0);
  stream_set_maxmem(stream, 0);
  stream_set_pending_reqs(stream, 0);
  stream_set_write_cb(stream, NULL);
  stream_set_close_cb(stream, NULL);
  stream_set_internal_close_cb(stream, NULL);
  stream_set_closed(stream, false);
  stream_set_events(stream, NULL);
}

void stream_may_close(Stream *stream)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (stream_is_closed(stream)) {
    return;
  }
  DLOG("closing Stream: %p", (void *)stream);
  stream_set_closed(stream, true);

#ifdef MSWIN
  if (UV_TTY == uv_guess_handle(stream_get_fd(stream))) {
    // Undo UV_TTY_MODE_RAW from stream_init(). #10801
    uv_tty_set_mode(&stream->uv.tty, UV_TTY_MODE_NORMAL);
  }
#endif

  if (!stream_get_pending_reqs(stream)) {
    stream_close_handle(stream);
  }  // Else: rstream.c:read_event() or wstream.c:write_cb() will call stream_close_handle().
}

void stream_close_handle(Stream *stream)
  FUNC_ATTR_NONNULL_ALL
{
  uv_handle_t *handle = NULL;
  if (stream->uvstream) {
    if (uv_stream_get_write_queue_size(stream->uvstream) > 0) {
      WLOG("closed Stream (%p) with %zu unwritten bytes",
           (void *)stream,
           uv_stream_get_write_queue_size(stream->uvstream));
    }
    handle = (uv_handle_t *)stream->uvstream;
  } else {
    handle = (uv_handle_t *)&stream->uv.idle;
  }

  assert(handle != NULL);

  if (!uv_is_closing(handle)) {
    uv_close(handle, close_cb);
  }
}

static void close_cb(uv_handle_t *handle)
{
  Stream *stream = handle->data;
  // Need to check if handle->data is NULL here as this callback may be called between
  // the handle's initialization and stream_init() (e.g. in socket_connect()).
  if (stream && stream_get_close_cb(stream)) {
    stream_call_close_cb(stream);
  }
  if (stream && stream_get_internal_close_cb(stream)) {
    stream_call_internal_close_cb(stream);
  }
}

// Rust accessor functions for opaque handle pattern

/// Check if a Stream is closed (accessor for Rust).
int nvim_stream_is_closed(Stream *stream) { return stream->closed ? 1 : 0; }

/// Get the pending requests count from a Stream (accessor for Rust).
size_t nvim_stream_pending_reqs(Stream *stream) { return stream->pending_reqs; }

/// Get the file descriptor from a Stream (accessor for Rust).
int nvim_stream_get_fd(Stream *stream) { return stream->fd; }

/// Get the current memory usage from a Stream (accessor for Rust).
size_t nvim_stream_get_curmem(Stream *stream) { return stream->curmem; }

/// Get the maximum memory limit from a Stream (accessor for Rust).
size_t nvim_stream_get_maxmem(Stream *stream) { return stream->maxmem; }

/// Get the events queue from a Stream (accessor for Rust).
MultiQueue *nvim_stream_get_events(Stream *stream) { return stream->events; }

/// Increment pending requests count for a Stream (accessor for Rust).
void nvim_stream_pending_reqs_inc(Stream *stream) { stream->pending_reqs++; }

/// Decrement pending requests count for a Stream (accessor for Rust).
void nvim_stream_pending_reqs_dec(Stream *stream) { stream->pending_reqs--; }

/// Set the closed flag for a Stream (accessor for Rust).
void nvim_stream_set_closed(Stream *stream, int closed) { stream->closed = closed != 0; }

/// Get the pending requests count for a Stream (accessor for Rust).
size_t nvim_stream_get_pending_reqs(Stream *stream) { return stream->pending_reqs; }

/// Set the maxmem field for a Stream (accessor for Rust).
void nvim_stream_set_maxmem(Stream *stream, size_t maxmem) { stream->maxmem = maxmem; }

/// Set the curmem field for a Stream (accessor for Rust).
void nvim_stream_set_curmem(Stream *stream, size_t curmem) { stream->curmem = curmem; }

/// Add to the curmem field for a Stream (accessor for Rust).
void nvim_stream_curmem_add(Stream *stream, size_t amount) { stream->curmem += amount; }

/// Subtract from the curmem field for a Stream (accessor for Rust).
void nvim_stream_curmem_sub(Stream *stream, size_t amount) { stream->curmem -= amount; }

/// Get the write callback from a Stream (accessor for Rust).
void *nvim_stream_get_write_cb(Stream *stream) { return (void *)stream->write_cb; }

/// Set the write callback for a Stream (accessor for Rust).
void nvim_stream_set_write_cb(Stream *stream, void *cb) { stream->write_cb = (stream_write_cb)cb; }

/// Call the write callback if set (accessor for Rust).
void nvim_stream_call_write_cb(Stream *stream, void *data, int status)
{
  if (stream->write_cb) {
    stream->write_cb(stream, data, status);
  }
}

/// Get the cb_data from a Stream (accessor for Rust).
void *nvim_stream_get_cb_data(Stream *stream) { return stream->cb_data; }

/// Set the cb_data for a Stream (accessor for Rust).
void nvim_stream_set_cb_data(Stream *stream, void *data) { stream->cb_data = data; }

/// Get the fpos from a Stream (accessor for Rust).
int64_t nvim_stream_get_fpos(Stream *stream) { return stream->fpos; }

/// Set the fpos for a Stream (accessor for Rust).
void nvim_stream_set_fpos(Stream *stream, int64_t fpos) { stream->fpos = fpos; }

/// Add to the fpos for a Stream (accessor for Rust).
void nvim_stream_fpos_add(Stream *stream, int64_t amount) { stream->fpos += amount; }

/// Get the close_cb from a Stream (accessor for Rust).
void *nvim_stream_get_close_cb(Stream *stream) { return (void *)stream->close_cb; }

/// Set the close_cb for a Stream (accessor for Rust).
void nvim_stream_set_close_cb(Stream *stream, void *cb) { stream->close_cb = (stream_close_cb)cb; }

/// Get the close_cb_data from a Stream (accessor for Rust).
void *nvim_stream_get_close_cb_data(Stream *stream) { return stream->close_cb_data; }

/// Set the close_cb_data for a Stream (accessor for Rust).
void nvim_stream_set_close_cb_data(Stream *stream, void *data) { stream->close_cb_data = data; }

/// Get the internal_data from a Stream (accessor for Rust).
void *nvim_stream_get_internal_data(Stream *stream) { return stream->internal_data; }

/// Set the internal_data for a Stream (accessor for Rust).
void nvim_stream_set_internal_data(Stream *stream, void *data) { stream->internal_data = data; }

/// Get the internal_close_cb from a Stream (accessor for Rust).
void *nvim_stream_get_internal_close_cb(Stream *stream) { return (void *)stream->internal_close_cb; }

/// Set the internal_close_cb for a Stream (accessor for Rust).
void nvim_stream_set_internal_close_cb(Stream *stream, void *cb) { stream->internal_close_cb = (stream_close_cb)cb; }

/// Call the close_cb if set (accessor for Rust).
void nvim_stream_call_close_cb(Stream *stream)
{
  if (stream->close_cb) {
    stream->close_cb(stream, stream->close_cb_data);
  }
}

/// Call the internal_close_cb if set (accessor for Rust).
void nvim_stream_call_internal_close_cb(Stream *stream)
{
  if (stream->internal_close_cb) {
    stream->internal_close_cb(stream, stream->internal_data);
  }
}

/// Set the pending_reqs for a Stream (accessor for Rust).
void nvim_stream_set_pending_reqs(Stream *stream, size_t pending_reqs) { stream->pending_reqs = pending_reqs; }

/// Set the events queue for a Stream (accessor for Rust).
void nvim_stream_set_events(Stream *stream, MultiQueue *events) { stream->events = events; }
