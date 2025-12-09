#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <uv.h>

#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/socket.h"
#include "nvim/event/stream.h"
#include "nvim/gettext_defs.h"
#include "nvim/log.h"
#include "nvim/main.h"
#include "nvim/memory.h"
#include "nvim/os/fs.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/types_defs.h"

#include "event/socket.c.generated.h"

#ifdef USE_RUST_EVENT
// Rust function declarations
extern void *rs_socket_watcher_get_data(SocketWatcher *watcher);
extern void rs_socket_watcher_set_data(SocketWatcher *watcher, void *data);
#define socket_watcher_get_data(w) rs_socket_watcher_get_data(w)
#define socket_watcher_set_data(w, d) rs_socket_watcher_set_data(w, d)
extern MultiQueue *rs_socket_watcher_get_events(SocketWatcher *watcher);
extern void rs_socket_watcher_set_events(SocketWatcher *watcher, MultiQueue *events);
#define socket_watcher_get_events(w) rs_socket_watcher_get_events(w)
#define socket_watcher_set_events(w, e) rs_socket_watcher_set_events(w, e)
extern void *rs_socket_watcher_get_cb(SocketWatcher *watcher);
extern void rs_socket_watcher_set_cb(SocketWatcher *watcher, void *cb);
#define socket_watcher_get_cb(w) ((socket_cb)rs_socket_watcher_get_cb(w))
#define socket_watcher_set_cb(w, c) rs_socket_watcher_set_cb(w, (void *)(c))
extern void *rs_socket_watcher_get_close_cb(SocketWatcher *watcher);
extern void rs_socket_watcher_set_close_cb(SocketWatcher *watcher, void *cb);
#define socket_watcher_get_close_cb(w) ((socket_close_cb)rs_socket_watcher_get_close_cb(w))
#define socket_watcher_set_close_cb(w, c) rs_socket_watcher_set_close_cb(w, (void *)(c))
extern void rs_socket_watcher_call_cb(SocketWatcher *watcher, int status);
extern void rs_socket_watcher_call_close_cb(SocketWatcher *watcher);
#define socket_watcher_call_cb(w, s) rs_socket_watcher_call_cb(w, s)
#define socket_watcher_call_close_cb(w) rs_socket_watcher_call_close_cb(w)
#else
#define socket_watcher_get_data(w) ((w)->data)
#define socket_watcher_set_data(w, d) ((w)->data = (d))
#define socket_watcher_get_events(w) ((w)->events)
#define socket_watcher_set_events(w, e) ((w)->events = (e))
#define socket_watcher_get_cb(w) ((w)->cb)
#define socket_watcher_set_cb(w, c) ((w)->cb = (c))
#define socket_watcher_get_close_cb(w) ((w)->close_cb)
#define socket_watcher_set_close_cb(w, c) ((w)->close_cb = (c))
#define socket_watcher_call_cb(w, s) do { if ((w)->cb) (w)->cb((w), (s), (w)->data); } while (0)
#define socket_watcher_call_close_cb(w) do { if ((w)->close_cb) (w)->close_cb((w), (w)->data); } while (0)
#endif

int socket_watcher_init(Loop *loop, SocketWatcher *watcher, const char *endpoint)
  FUNC_ATTR_NONNULL_ALL
{
  xstrlcpy(watcher->addr, endpoint, sizeof(watcher->addr));
  char *addr = watcher->addr;
  char *host_end = strrchr(addr, ':');

  if (host_end && addr != host_end) {
    // Split user specified address into two strings, addr (hostname) and port.
    // The port part in watcher->addr will be updated later.
    *host_end = NUL;
    char *port = host_end + 1;
    intmax_t iport;

    int ok = try_getdigits(&(char *){ port }, &iport);
    if (!ok || iport < 0 || iport > UINT16_MAX) {
      ELOG("Invalid port: %s", port);
      return UV_EINVAL;
    }

    if (*port == NUL) {
      // When no port is given, (uv_)getaddrinfo expects NULL otherwise the
      // implementation may attempt to lookup the service by name (and fail)
      port = NULL;
    }

    uv_getaddrinfo_t request;

    int retval = uv_getaddrinfo(&loop->uv, &request, NULL, addr, port,
                                &(struct addrinfo){ .ai_family = AF_UNSPEC,
                                                    .ai_socktype = SOCK_STREAM, });
    if (retval != 0) {
      ELOG("Host lookup failed: %s", endpoint);
      return retval;
    }
    watcher->uv.tcp.addrinfo = request.addrinfo;

    uv_tcp_init(&loop->uv, &watcher->uv.tcp.handle);
    uv_tcp_nodelay(&watcher->uv.tcp.handle, true);
    watcher->stream = (uv_stream_t *)(&watcher->uv.tcp.handle);
  } else {
    uv_pipe_init(&loop->uv, &watcher->uv.pipe.handle, 0);
    watcher->stream = (uv_stream_t *)(&watcher->uv.pipe.handle);
  }

  watcher->stream->data = watcher;
  socket_watcher_set_cb(watcher, NULL);
  socket_watcher_set_close_cb(watcher, NULL);
  socket_watcher_set_events(watcher, NULL);
  socket_watcher_set_data(watcher, NULL);

  return 0;
}

int socket_watcher_start(SocketWatcher *watcher, int backlog, socket_cb cb)
  FUNC_ATTR_NONNULL_ALL
{
  socket_watcher_set_cb(watcher, cb);
  int result = UV_EINVAL;

  if (watcher->stream->type == UV_TCP) {
    struct addrinfo *ai = watcher->uv.tcp.addrinfo;

    for (; ai; ai = ai->ai_next) {
      result = uv_tcp_bind(&watcher->uv.tcp.handle, ai->ai_addr, 0);
      if (result != 0) {
        continue;
      }
      result = uv_listen(watcher->stream, backlog, connection_cb);
      if (result == 0) {
        struct sockaddr_storage sas;

        // When the endpoint in socket_watcher_init() didn't specify a port
        // number, a free random port number will be assigned. sin_port will
        // contain 0 in this case, unless uv_tcp_getsockname() is used first.
        uv_tcp_getsockname(&watcher->uv.tcp.handle, (struct sockaddr *)&sas,
                           &(int){ sizeof(sas) });
        uint16_t port = (sas.ss_family == AF_INET) ? ((struct sockaddr_in *)(&sas))->sin_port
                                                   : ((struct sockaddr_in6 *)(&sas))->sin6_port;
        // v:servername uses the string from watcher->addr
        size_t len = strlen(watcher->addr);
        snprintf(watcher->addr + len, sizeof(watcher->addr) - len, ":%" PRIu16,
                 ntohs(port));
        break;
      }
    }
    uv_freeaddrinfo(watcher->uv.tcp.addrinfo);
  } else {
    result = uv_pipe_bind(&watcher->uv.pipe.handle, watcher->addr);
    if (result == 0) {
      result = uv_listen(watcher->stream, backlog, connection_cb);
    }
  }

  assert(result <= 0);  // libuv should return negative error code or zero.
  if (result < 0) {
    if (result == UV_EACCES) {
      // Libuv converts ENOENT to EACCES for Windows compatibility, but if
      // the parent directory does not exist, ENOENT would be more accurate.
      *path_tail(watcher->addr) = NUL;
      if (!os_path_exists(watcher->addr)) {
        result = UV_ENOENT;
      }
    }
    return result;
  }

  return 0;
}

int socket_watcher_accept(SocketWatcher *watcher, RStream *stream)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_NONNULL_ARG(2)
{
  uv_stream_t *client;

  if (watcher->stream->type == UV_TCP) {
    client = (uv_stream_t *)(&stream->s.uv.tcp);
    uv_tcp_init(watcher->uv.tcp.handle.loop, (uv_tcp_t *)client);
    uv_tcp_nodelay((uv_tcp_t *)client, true);
  } else {
    client = (uv_stream_t *)&stream->s.uv.pipe;
    uv_pipe_init(watcher->uv.pipe.handle.loop, (uv_pipe_t *)client, 0);
  }

  int result = uv_accept(watcher->stream, client);

  if (result) {
    uv_close((uv_handle_t *)client, NULL);
    return result;
  }

  stream_init(NULL, &stream->s, -1, client);
  return 0;
}

void socket_watcher_close(SocketWatcher *watcher, socket_close_cb cb)
  FUNC_ATTR_NONNULL_ARG(1)
{
  socket_watcher_set_close_cb(watcher, cb);
  uv_close((uv_handle_t *)watcher->stream, close_cb);
}

static void connection_event(void **argv)
{
  SocketWatcher *watcher = argv[0];
  int status = (int)(uintptr_t)(argv[1]);
  socket_watcher_call_cb(watcher, status);
}

static void connection_cb(uv_stream_t *handle, int status)
{
  SocketWatcher *watcher = handle->data;
  CREATE_EVENT(socket_watcher_get_events(watcher), connection_event, watcher, (void *)(uintptr_t)status);
}

static void close_cb(uv_handle_t *handle)
{
  SocketWatcher *watcher = handle->data;
  socket_watcher_call_close_cb(watcher);
}

static void connect_cb(uv_connect_t *req, int status)
{
  int *ret_status = req->data;
  *ret_status = status;
  uv_handle_t *handle = (uv_handle_t *)req->handle;
  if (status != 0 && !uv_is_closing(handle)) {
    uv_close(handle, NULL);
  }
}

bool socket_connect(Loop *loop, RStream *stream, bool is_tcp, const char *address, int timeout,
                    const char **error)
{
  bool success = false;
  int status;
  uv_connect_t req;
  req.data = &status;
  uv_stream_t *uv_stream;

  uv_tcp_t *tcp = &stream->s.uv.tcp;
  uv_getaddrinfo_t addr_req;
  addr_req.addrinfo = NULL;
  const struct addrinfo *addrinfo = NULL;
  char *addr = NULL;
  if (is_tcp) {
    addr = xstrdup(address);
    char *host_end = strrchr(addr, ':');
    if (!host_end) {
      *error = _("tcp address must be host:port");
      goto cleanup;
    }
    *host_end = NUL;

    const struct addrinfo hints = { .ai_family = AF_UNSPEC,
                                    .ai_socktype = SOCK_STREAM,
                                    .ai_flags = AI_NUMERICSERV };
    int retval = uv_getaddrinfo(&loop->uv, &addr_req, NULL,
                                addr, host_end + 1, &hints);
    if (retval != 0) {
      *error = _("failed to lookup host or port");
      goto cleanup;
    }
    addrinfo = addr_req.addrinfo;

tcp_retry:
    uv_tcp_init(&loop->uv, tcp);
    uv_tcp_nodelay(tcp, true);
    uv_tcp_connect(&req,  tcp, addrinfo->ai_addr, connect_cb);
    uv_stream = (uv_stream_t *)tcp;
  } else {
    uv_pipe_t *pipe = &stream->s.uv.pipe;
    uv_pipe_init(&loop->uv, pipe, 0);
    uv_pipe_connect(&req,  pipe, address, connect_cb);
    uv_stream = (uv_stream_t *)pipe;
  }
  status = 1;
  LOOP_PROCESS_EVENTS_UNTIL(&main_loop, NULL, timeout, status != 1);
  if (status == 0) {
    stream_init(NULL, &stream->s, -1, uv_stream);
    success = true;
  } else {
    if (!uv_is_closing((uv_handle_t *)uv_stream)) {
      uv_close((uv_handle_t *)uv_stream, NULL);
      if (status == 1) {
        // The uv_close() above will make libuv call connect_cb() with UV_ECANCELED.
        // Make sure connect_cb() has been called here, as if it's called after this
        // function ends it will cause a stack-use-after-scope.
        LOOP_PROCESS_EVENTS_UNTIL(&main_loop, NULL, -1, status != 1);
      }
    }

    if (is_tcp && addrinfo->ai_next) {
      addrinfo = addrinfo->ai_next;
      goto tcp_retry;
    } else {
      *error = _("connection refused");
    }
  }

cleanup:
  xfree(addr);
  uv_freeaddrinfo(addr_req.addrinfo);
  return success;
}

// =============================================================================
// Rust accessor functions for opaque handle pattern
// =============================================================================

/// Get the address from a SocketWatcher (accessor for Rust).
const char *nvim_socket_watcher_get_addr(SocketWatcher *watcher)
{
  return watcher->addr;
}

/// Get the events queue from a SocketWatcher (accessor for Rust).
MultiQueue *nvim_socket_watcher_get_events(SocketWatcher *watcher)
{
  return watcher->events;
}

/// Get the user data from a SocketWatcher (accessor for Rust).
void *nvim_socket_watcher_get_data(SocketWatcher *watcher)
{
  return watcher->data;
}

/// Check if a SocketWatcher is TCP type (accessor for Rust).
int nvim_socket_watcher_is_tcp(SocketWatcher *watcher)
{
  return (watcher->stream && watcher->stream->type == UV_TCP) ? 1 : 0;
}

/// Set the user data for a SocketWatcher (accessor for Rust).
void nvim_socket_watcher_set_data(SocketWatcher *watcher, void *data)
{
  watcher->data = data;
}

/// Set the events queue for a SocketWatcher (accessor for Rust).
void nvim_socket_watcher_set_events(SocketWatcher *watcher, MultiQueue *events)
{
  watcher->events = events;
}

/// Get the cb from a SocketWatcher (accessor for Rust).
void *nvim_socket_watcher_get_cb(SocketWatcher *watcher)
{
  return (void *)watcher->cb;
}

/// Set the cb for a SocketWatcher (accessor for Rust).
void nvim_socket_watcher_set_cb(SocketWatcher *watcher, void *cb)
{
  watcher->cb = (socket_cb)cb;
}

/// Get the close_cb from a SocketWatcher (accessor for Rust).
void *nvim_socket_watcher_get_close_cb(SocketWatcher *watcher)
{
  return (void *)watcher->close_cb;
}

/// Set the close_cb for a SocketWatcher (accessor for Rust).
void nvim_socket_watcher_set_close_cb(SocketWatcher *watcher, void *cb)
{
  watcher->close_cb = (socket_close_cb)cb;
}

/// Call the socket callback if set (accessor for Rust).
void nvim_socket_watcher_call_cb(SocketWatcher *watcher, int status)
{
  if (watcher->cb) {
    watcher->cb(watcher, status, watcher->data);
  }
}

/// Call the close callback if set (accessor for Rust).
void nvim_socket_watcher_call_close_cb(SocketWatcher *watcher)
{
  if (watcher->close_cb) {
    watcher->close_cb(watcher, watcher->data);
  }
}
