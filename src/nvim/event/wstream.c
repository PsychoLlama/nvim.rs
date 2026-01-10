#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <uv.h>

#include "nvim/event/defs.h"
#include "nvim/event/stream.h"
#include "nvim/event/wstream.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/types_defs.h"

#define DEFAULT_MAXMEM 1024 * 1024 * 2000

typedef struct {
  Stream *stream;
  WBuffer *buffer;
  uv_write_t uv_req;
} WRequest;

#include "event/wstream.c.generated.h"

// Rust implementation in nvim-event crate
extern int rs_stream_is_closed(Stream *stream);
extern size_t rs_stream_pending_reqs(Stream *stream);
extern void rs_stream_pending_reqs_inc(Stream *stream);
extern void rs_stream_pending_reqs_dec(Stream *stream);
extern size_t rs_stream_get_curmem(Stream *stream);
extern size_t rs_stream_get_maxmem(Stream *stream);
#define stream_is_closed(s) rs_stream_is_closed(s)
#define stream_pending_reqs(s) rs_stream_pending_reqs(s)
#define stream_pending_reqs_inc(s) rs_stream_pending_reqs_inc(s)
#define stream_pending_reqs_dec(s) rs_stream_pending_reqs_dec(s)
#define stream_get_curmem(s) rs_stream_get_curmem(s)
#define stream_get_maxmem(s) rs_stream_get_maxmem(s)
extern void rs_stream_set_maxmem(Stream *stream, size_t maxmem);
#define stream_set_maxmem(s, m) rs_stream_set_maxmem(s, m)
extern void rs_stream_curmem_add(Stream *stream, size_t amount);
extern void rs_stream_curmem_sub(Stream *stream, size_t amount);
#define stream_curmem_add(s, a) rs_stream_curmem_add(s, a)
#define stream_curmem_sub(s, a) rs_stream_curmem_sub(s, a)
extern void *rs_stream_get_write_cb(Stream *stream);
extern void rs_stream_set_write_cb(Stream *stream, void *cb);
extern void rs_stream_call_write_cb(Stream *stream, void *data, int status);
#define stream_get_write_cb(s) ((stream_write_cb)rs_stream_get_write_cb(s))
#define stream_set_write_cb(s, c) rs_stream_set_write_cb(s, (void *)(c))
#define stream_call_write_cb(s, d, st) rs_stream_call_write_cb(s, d, st)
extern void *rs_stream_get_cb_data(Stream *stream);
extern void rs_stream_set_cb_data(Stream *stream, void *data);
#define stream_get_cb_data(s) rs_stream_get_cb_data(s)
#define stream_set_cb_data(s, d) rs_stream_set_cb_data(s, d)
extern int64_t rs_stream_get_fpos(Stream *stream);
extern void rs_stream_set_fpos(Stream *stream, int64_t fpos);
extern void rs_stream_fpos_add(Stream *stream, int64_t amount);
#define stream_get_fpos(s) rs_stream_get_fpos(s)
#define stream_set_fpos(s, f) rs_stream_set_fpos(s, f)
#define stream_fpos_add(s, a) rs_stream_fpos_add(s, a)

void wstream_init_fd(Loop *loop, Stream *stream, int fd, size_t maxmem)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_NONNULL_ARG(2)
{
  stream_init(loop, stream, fd, NULL);
  wstream_init(stream, maxmem);
}

void wstream_init_stream(Stream *stream, uv_stream_t *uvstream, size_t maxmem)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_NONNULL_ARG(2)
{
  stream_init(NULL, stream, -1, uvstream);
  wstream_init(stream, maxmem);
}

void wstream_init(Stream *stream, size_t maxmem)
{
  stream_set_maxmem(stream, maxmem ? maxmem : DEFAULT_MAXMEM);
}

/// Sets a callback that will be called on completion of a write request,
/// indicating failure/success.
///
/// This affects all requests currently in-flight as well. Overwrites any
/// possible earlier callback.
///
/// @note This callback will not fire if the write request couldn't even be
///       queued properly (i.e.: when `wstream_write() returns an error`).
///
/// @param stream The `Stream` instance
/// @param cb The callback
void wstream_set_write_cb(Stream *stream, stream_write_cb cb, void *data)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  stream_set_write_cb(stream, cb);
  stream_set_cb_data(stream, data);
}

/// Queues data for writing to the backing file descriptor of a `Stream`
/// instance. This will fail if the write would cause the Stream use more
/// memory than specified by `maxmem`.
///
/// @param stream The `Stream` instance
/// @param buffer The buffer which contains data to be written
/// @return false if the write failed
bool wstream_write(Stream *stream, WBuffer *buffer)
  FUNC_ATTR_NONNULL_ALL
{
  assert(stream_get_maxmem(stream));
  // This should not be called after a stream was freed
  assert(!stream_is_closed(stream));

  uv_buf_t uvbuf;
  uvbuf.base = buffer->data;
  uvbuf.len = UV_BUF_LEN(buffer->size);

  if (!stream->uvstream) {
    uv_fs_t req;

    // Synchronous write
    uv_fs_write(stream->uv.idle.loop, &req, stream->fd, &uvbuf, 1, stream_get_fpos(stream), NULL);

    uv_fs_req_cleanup(&req);

    wstream_release_wbuffer(buffer);

    assert(stream_get_write_cb(stream) == NULL);

    stream_fpos_add(stream, MAX(req.result, 0));
    return req.result > 0;
  }

  if (stream_get_curmem(stream) > stream_get_maxmem(stream)) {
    goto err;
  }

  stream_curmem_add(stream, buffer->size);

  WRequest *data = xmalloc(sizeof(WRequest));
  data->stream = stream;
  data->buffer = buffer;
  data->uv_req.data = data;

  if (uv_write(&data->uv_req, stream->uvstream, &uvbuf, 1, write_cb)) {
    xfree(data);
    goto err;
  }

  stream_pending_reqs_inc(stream);
  return true;

err:
  wstream_release_wbuffer(buffer);
  return false;
}

/// Creates a WBuffer object for holding output data. Instances of this
/// object can be reused across Stream instances, and the memory is freed
/// automatically when no longer needed (it tracks the number of references
/// internally)
///
/// @param data Data stored by the WBuffer
/// @param size The size of the data array
/// @param refcount The number of references for the WBuffer. This will be used
///        by Stream instances to decide when a WBuffer should be freed.
/// @param cb Pointer to function that will be responsible for freeing
///        the buffer data (passing `xfree` will work as expected).
/// @return The allocated WBuffer instance
WBuffer *wstream_new_buffer(char *data, size_t size, size_t refcount, wbuffer_data_finalizer cb)
  FUNC_ATTR_NONNULL_ARG(1)
{
  WBuffer *rv = xmalloc(sizeof(WBuffer));
  rv->size = size;
  rv->refcount = refcount;
  rv->cb = cb;
  rv->data = data;

  return rv;
}

static void write_cb(uv_write_t *req, int status)
{
  WRequest *data = req->data;

  stream_curmem_sub(data->stream, data->buffer->size);

  wstream_release_wbuffer(data->buffer);

  stream_call_write_cb(data->stream, stream_get_cb_data(data->stream), status);

  stream_pending_reqs_dec(data->stream);

  if (stream_is_closed(data->stream) && stream_pending_reqs(data->stream) == 0) {
    // Last pending write; free the stream.
    stream_close_handle(data->stream);
  }

  xfree(data);
}

void wstream_release_wbuffer(WBuffer *buffer)
  FUNC_ATTR_NONNULL_ALL
{
  if (!--buffer->refcount) {
    if (buffer->cb) {
      buffer->cb(buffer->data);
    }

    xfree(buffer);
  }
}

// =============================================================================
// Rust accessor functions for WBuffer opaque handle pattern
// =============================================================================

/// Get the size from a WBuffer (accessor for Rust).
size_t nvim_wbuffer_get_size(WBuffer *buffer)
{
  return buffer->size;
}

/// Get the refcount from a WBuffer (accessor for Rust).
size_t nvim_wbuffer_get_refcount(WBuffer *buffer)
{
  return buffer->refcount;
}

/// Get the data pointer from a WBuffer (accessor for Rust).
char *nvim_wbuffer_get_data(WBuffer *buffer)
{
  return buffer->data;
}

/// Get the callback from a WBuffer (accessor for Rust).
void *nvim_wbuffer_get_cb(WBuffer *buffer)
{
  return (void *)buffer->cb;
}

/// Set the size for a WBuffer (accessor for Rust).
void nvim_wbuffer_set_size(WBuffer *buffer, size_t size)
{
  buffer->size = size;
}

/// Set the refcount for a WBuffer (accessor for Rust).
void nvim_wbuffer_set_refcount(WBuffer *buffer, size_t refcount)
{
  buffer->refcount = refcount;
}

/// Decrement refcount for a WBuffer (accessor for Rust).
/// Returns 1 if refcount is now 0 and buffer should be freed.
int nvim_wbuffer_decref(WBuffer *buffer)
{
  return (--buffer->refcount == 0) ? 1 : 0;
}
