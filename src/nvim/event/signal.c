#include <stddef.h>
#include <uv.h>

#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/signal.h"
#include "nvim/types_defs.h"

#include "event/signal.c.generated.h"

// Rust function declarations
extern void *rs_signal_watcher_get_data(SignalWatcher *watcher);
extern void rs_signal_watcher_set_data(SignalWatcher *watcher, void *data);
#define signal_watcher_get_data(w) rs_signal_watcher_get_data(w)
#define signal_watcher_set_data(w, d) rs_signal_watcher_set_data(w, d)
extern MultiQueue *rs_signal_watcher_get_events(SignalWatcher *watcher);
extern void rs_signal_watcher_set_events(SignalWatcher *watcher, MultiQueue *events);
#define signal_watcher_get_events(w) rs_signal_watcher_get_events(w)
#define signal_watcher_set_events(w, e) rs_signal_watcher_set_events(w, e)
extern void *rs_signal_watcher_get_cb(SignalWatcher *watcher);
extern void rs_signal_watcher_set_cb(SignalWatcher *watcher, void *cb);
#define signal_watcher_get_cb(w) ((signal_cb)rs_signal_watcher_get_cb(w))
#define signal_watcher_set_cb(w, c) rs_signal_watcher_set_cb(w, (void *)(c))
extern void *rs_signal_watcher_get_close_cb(SignalWatcher *watcher);
extern void rs_signal_watcher_set_close_cb(SignalWatcher *watcher, void *cb);
#define signal_watcher_get_close_cb(w) ((signal_close_cb)rs_signal_watcher_get_close_cb(w))
#define signal_watcher_set_close_cb(w, c) rs_signal_watcher_set_close_cb(w, (void *)(c))
extern void rs_signal_watcher_call_cb(SignalWatcher *watcher);
extern void rs_signal_watcher_call_close_cb(SignalWatcher *watcher);
#define signal_watcher_call_cb(w) rs_signal_watcher_call_cb(w)
#define signal_watcher_call_close_cb(w) rs_signal_watcher_call_close_cb(w)
// Loop accessors
extern MultiQueue *rs_loop_get_fast_events(Loop *loop);
#define loop_get_fast_events(l) rs_loop_get_fast_events(l)

void signal_watcher_init(Loop *loop, SignalWatcher *watcher, void *data)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_NONNULL_ARG(2)
{
  uv_signal_init(&loop->uv, &watcher->uv);
  watcher->uv.data = watcher;
  signal_watcher_set_data(watcher, data);
  signal_watcher_set_cb(watcher, NULL);
  signal_watcher_set_events(watcher, loop_get_fast_events(loop));
}

void signal_watcher_start(SignalWatcher *watcher, signal_cb cb, int signum)
  FUNC_ATTR_NONNULL_ALL
{
  signal_watcher_set_cb(watcher, cb);
  uv_signal_start(&watcher->uv, signal_watcher_cb, signum);
}

void signal_watcher_stop(SignalWatcher *watcher)
  FUNC_ATTR_NONNULL_ALL
{
  uv_signal_stop(&watcher->uv);
}

void signal_watcher_close(SignalWatcher *watcher, signal_close_cb cb)
  FUNC_ATTR_NONNULL_ARG(1)
{
  signal_watcher_set_close_cb(watcher, cb);
  uv_close((uv_handle_t *)&watcher->uv, close_cb);
}

static void signal_event(void **argv)
{
  SignalWatcher *watcher = argv[0];
  signal_watcher_call_cb(watcher);
}

static void signal_watcher_cb(uv_signal_t *handle, int signum)
{
  SignalWatcher *watcher = handle->data;
  CREATE_EVENT(signal_watcher_get_events(watcher), signal_event, watcher);
}

static void close_cb(uv_handle_t *handle)
{
  SignalWatcher *watcher = handle->data;
  signal_watcher_call_close_cb(watcher);
}

// Rust accessor functions for opaque handle pattern

/// Get the signal number from a SignalWatcher (accessor for Rust).
int nvim_signal_watcher_get_signum(SignalWatcher *watcher) { return watcher->uv.signum; }

/// Get the events queue from a SignalWatcher (accessor for Rust).
MultiQueue *nvim_signal_watcher_get_events(SignalWatcher *watcher) { return watcher->events; }

/// Get the user data from a SignalWatcher (accessor for Rust).
void *nvim_signal_watcher_get_data(SignalWatcher *watcher) { return watcher->data; }

/// Set the user data for a SignalWatcher (accessor for Rust).
void nvim_signal_watcher_set_data(SignalWatcher *watcher, void *data) { watcher->data = data; }

/// Set the events queue for a SignalWatcher (accessor for Rust).
void nvim_signal_watcher_set_events(SignalWatcher *watcher, MultiQueue *events) { watcher->events = events; }

/// Get the cb from a SignalWatcher (accessor for Rust).
void *nvim_signal_watcher_get_cb(SignalWatcher *watcher) { return (void *)watcher->cb; }

/// Set the cb for a SignalWatcher (accessor for Rust).
void nvim_signal_watcher_set_cb(SignalWatcher *watcher, void *cb) { watcher->cb = (signal_cb)cb; }

/// Get the close_cb from a SignalWatcher (accessor for Rust).
void *nvim_signal_watcher_get_close_cb(SignalWatcher *watcher) { return (void *)watcher->close_cb; }

/// Set the close_cb for a SignalWatcher (accessor for Rust).
void nvim_signal_watcher_set_close_cb(SignalWatcher *watcher, void *cb) { watcher->close_cb = (signal_close_cb)cb; }

/// Call the signal callback if set (accessor for Rust).
void nvim_signal_watcher_call_cb(SignalWatcher *watcher)
{
  if (watcher->cb) {
    watcher->cb(watcher, watcher->uv.signum, watcher->data);
  }
}

/// Call the close callback if set (accessor for Rust).
void nvim_signal_watcher_call_close_cb(SignalWatcher *watcher)
{
  if (watcher->close_cb) {
    watcher->close_cb(watcher, watcher->data);
  }
}
