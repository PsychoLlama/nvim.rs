#include <stdbool.h>
#include <stdint.h>
#include <uv.h>

#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/time.h"
#include "nvim/types_defs.h"

#include "event/time.c.generated.h"

#ifdef USE_RUST_EVENT
// Rust function declarations
extern int rs_timewatcher_should_skip(TimeWatcher *tw);
#define timewatcher_should_skip(tw) rs_timewatcher_should_skip(tw)
extern void *rs_timewatcher_get_data(TimeWatcher *tw);
extern void rs_timewatcher_set_data(TimeWatcher *tw, void *data);
#define timewatcher_get_data(tw) rs_timewatcher_get_data(tw)
#define timewatcher_set_data(tw, d) rs_timewatcher_set_data(tw, d)
extern MultiQueue *rs_timewatcher_get_events(TimeWatcher *tw);
extern void rs_timewatcher_set_events(TimeWatcher *tw, MultiQueue *events);
#define timewatcher_get_events(tw) rs_timewatcher_get_events(tw)
#define timewatcher_set_events(tw, e) rs_timewatcher_set_events(tw, e)
extern int rs_timewatcher_is_blockable(TimeWatcher *tw);
extern void rs_timewatcher_set_blockable(TimeWatcher *tw, int blockable);
#define timewatcher_is_blockable(tw) rs_timewatcher_is_blockable(tw)
#define timewatcher_set_blockable(tw, b) rs_timewatcher_set_blockable(tw, b)
extern void *rs_timewatcher_get_cb(TimeWatcher *tw);
extern void rs_timewatcher_set_cb(TimeWatcher *tw, void *cb);
#define timewatcher_get_cb(tw) ((time_cb)rs_timewatcher_get_cb(tw))
#define timewatcher_set_cb(tw, c) rs_timewatcher_set_cb(tw, (void *)(c))
extern void *rs_timewatcher_get_close_cb(TimeWatcher *tw);
extern void rs_timewatcher_set_close_cb(TimeWatcher *tw, void *cb);
#define timewatcher_get_close_cb(tw) ((time_cb)rs_timewatcher_get_close_cb(tw))
#define timewatcher_set_close_cb(tw, c) rs_timewatcher_set_close_cb(tw, (void *)(c))
extern void rs_timewatcher_call_cb(TimeWatcher *tw);
extern void rs_timewatcher_call_close_cb(TimeWatcher *tw);
#define timewatcher_call_cb(tw) rs_timewatcher_call_cb(tw)
#define timewatcher_call_close_cb(tw) rs_timewatcher_call_close_cb(tw)
#else
#define timewatcher_should_skip(tw) ((tw)->blockable && !multiqueue_empty((tw)->events))
#define timewatcher_get_data(tw) ((tw)->data)
#define timewatcher_set_data(tw, d) ((tw)->data = (d))
#define timewatcher_get_events(tw) ((tw)->events)
#define timewatcher_set_events(tw, e) ((tw)->events = (e))
#define timewatcher_is_blockable(tw) ((tw)->blockable)
#define timewatcher_set_blockable(tw, b) ((tw)->blockable = (b))
#define timewatcher_get_cb(tw) ((tw)->cb)
#define timewatcher_set_cb(tw, c) ((tw)->cb = (c))
#define timewatcher_get_close_cb(tw) ((tw)->close_cb)
#define timewatcher_set_close_cb(tw, c) ((tw)->close_cb = (c))
#define timewatcher_call_cb(tw) do { if ((tw)->cb) (tw)->cb((tw), (tw)->data); } while (0)
#define timewatcher_call_close_cb(tw) do { if ((tw)->close_cb) (tw)->close_cb((tw), (tw)->data); } while (0)
#endif

void time_watcher_init(Loop *loop, TimeWatcher *watcher, void *data)
  FUNC_ATTR_NONNULL_ARG(1) FUNC_ATTR_NONNULL_ARG(2)
{
  uv_timer_init(&loop->uv, &watcher->uv);
  watcher->uv.data = watcher;
  timewatcher_set_data(watcher, data);
  timewatcher_set_events(watcher, loop->fast_events);
  timewatcher_set_blockable(watcher, false);
}

void time_watcher_start(TimeWatcher *watcher, time_cb cb, uint64_t timeout, uint64_t repeat)
  FUNC_ATTR_NONNULL_ALL
{
  timewatcher_set_cb(watcher, cb);
  uv_timer_start(&watcher->uv, time_watcher_cb, timeout, repeat);
}

void time_watcher_stop(TimeWatcher *watcher)
  FUNC_ATTR_NONNULL_ALL
{
  uv_timer_stop(&watcher->uv);
}

void time_watcher_close(TimeWatcher *watcher, time_cb cb)
  FUNC_ATTR_NONNULL_ARG(1)
{
  timewatcher_set_close_cb(watcher, cb);
  uv_close((uv_handle_t *)&watcher->uv, close_cb);
}

static void time_event(void **argv)
{
  TimeWatcher *watcher = argv[0];
  timewatcher_call_cb(watcher);
}

static void time_watcher_cb(uv_timer_t *handle)
  FUNC_ATTR_NONNULL_ALL
{
  TimeWatcher *watcher = handle->data;
  // Check if the timer blocked and there already is an unprocessed event waiting
  if (timewatcher_should_skip(watcher)) {
    return;
  }
  CREATE_EVENT(timewatcher_get_events(watcher), time_event, watcher);
}

static void close_event(void **argv)
{
  TimeWatcher *watcher = argv[0];
  timewatcher_call_close_cb(watcher);
}

static void close_cb(uv_handle_t *handle)
  FUNC_ATTR_NONNULL_ALL
{
  TimeWatcher *watcher = handle->data;
  if (timewatcher_get_close_cb(watcher)) {
    CREATE_EVENT(timewatcher_get_events(watcher), close_event, watcher);
  }
}

// =============================================================================
// Rust accessor functions for opaque handle pattern
// =============================================================================

/// Get the data pointer from a TimeWatcher (accessor for Rust).
void *nvim_timewatcher_get_data(TimeWatcher *tw)
{
  return tw->data;
}

/// Get the events queue from a TimeWatcher (accessor for Rust).
MultiQueue *nvim_timewatcher_get_events(TimeWatcher *tw)
{
  return tw->events;
}

/// Check if a TimeWatcher is blockable (accessor for Rust).
int nvim_timewatcher_is_blockable(TimeWatcher *tw)
{
  return tw->blockable ? 1 : 0;
}

/// Set the data pointer for a TimeWatcher (accessor for Rust).
void nvim_timewatcher_set_data(TimeWatcher *tw, void *data)
{
  tw->data = data;
}

/// Get the events queue from a TimeWatcher (accessor for Rust).
void nvim_timewatcher_set_events(TimeWatcher *tw, MultiQueue *events)
{
  tw->events = events;
}

/// Set the blockable flag for a TimeWatcher (accessor for Rust).
void nvim_timewatcher_set_blockable(TimeWatcher *tw, int blockable)
{
  tw->blockable = blockable != 0;
}

/// Get the cb from a TimeWatcher (accessor for Rust).
void *nvim_timewatcher_get_cb(TimeWatcher *tw)
{
  return (void *)tw->cb;
}

/// Set the cb for a TimeWatcher (accessor for Rust).
void nvim_timewatcher_set_cb(TimeWatcher *tw, void *cb)
{
  tw->cb = (time_cb)cb;
}

/// Get the close_cb from a TimeWatcher (accessor for Rust).
void *nvim_timewatcher_get_close_cb(TimeWatcher *tw)
{
  return (void *)tw->close_cb;
}

/// Set the close_cb for a TimeWatcher (accessor for Rust).
void nvim_timewatcher_set_close_cb(TimeWatcher *tw, void *cb)
{
  tw->close_cb = (time_cb)cb;
}

/// Call the cb if set (accessor for Rust).
void nvim_timewatcher_call_cb(TimeWatcher *tw)
{
  if (tw->cb) {
    tw->cb(tw, tw->data);
  }
}

/// Call the close_cb if set (accessor for Rust).
void nvim_timewatcher_call_close_cb(TimeWatcher *tw)
{
  if (tw->close_cb) {
    tw->close_cb(tw, tw->data);
  }
}
