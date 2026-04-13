#include <inttypes.h>
#include <lauxlib.h>
#include <stdbool.h>
#include <stddef.h>

#include "klib/kvec.h"
#include "nvim/api/buffer.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/assert_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/lua/executor.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

#include "buffer_updates.c.generated.h"  // IWYU pragma: keep


/// Get the size of the update_channels kvec for a buffer.
size_t nvim_buf_get_update_channels_size(buf_T *buf) { return kv_size(buf->update_channels); }

/// Get the size of the update_callbacks kvec for a buffer.
size_t nvim_buf_get_update_callbacks_size(buf_T *buf) { return kv_size(buf->update_callbacks); }

// buf_updates_register: migrated to Rust (buffer_updates crate)

// buf_updates_send_end: migrated to Rust (buffer_updates crate)

// buf_updates_unregister: migrated to Rust (buffer_updates crate)

// buf_free_callbacks: migrated to Rust (buffer_updates crate)

// buf_updates_unload: migrated to Rust (buffer_updates crate)

// buf_updates_send_changes: migrated to Rust (buffer_updates crate)

// buf_updates_send_splice: migrated to Rust (buffer_updates crate)

// buf_updates_changedtick: migrated to Rust (buffer_updates crate)

// buf_updates_changedtick_single: migrated to Rust (buffer_updates crate)

// buffer_update_callbacks_free: migrated to Rust (buffer_updates crate)

// Accessor and helper functions for Rust FFI (buffer_updates crate)

/// Returns true if buf->b_ml.ml_mfp is NULL (buffer not loaded).
bool nvim_buf_get_ml_mfp_is_null(buf_T *buf) { return buf->b_ml.ml_mfp == NULL; }

/// Free the LuaRef fields of a BufUpdateCallbacks struct.
/// This is the C helper for buffer_update_callbacks_free (used in Phase 1).
void nvim_buf_callbacks_free_refs(BufUpdateCallbacks cb)
{
  api_free_luaref(cb.on_lines);
  api_free_luaref(cb.on_bytes);
  api_free_luaref(cb.on_changedtick);
  api_free_luaref(cb.on_reload);
  api_free_luaref(cb.on_detach);
}

/// Send the nvim_buf_changedtick_event RPC event to a single channel.
void nvim_buf_send_changedtick_event(buf_T *buf, uint64_t channel_id)
{
  MAXSIZE_TEMP_ARRAY(args, 2);
  ADD_C(args, BUFFER_OBJ(buf->handle));
  ADD_C(args, INTEGER_OBJ(buf_get_changedtick(buf)));
  rpc_send_event(channel_id, "nvim_buf_changedtick_event", args);
}

/// Send the nvim_buf_detach_event RPC event to a single channel.
void nvim_buf_send_detach_event(buf_T *buf, uint64_t channel_id)
{
  MAXSIZE_TEMP_ARRAY(args, 1);
  ADD_C(args, BUFFER_OBJ(buf->handle));
  rpc_send_event(channel_id, "nvim_buf_detach_event", args);
}

// Phase 2 accessor functions for Rust FFI (channel management)

/// Get the channel ID at index i in update_channels.
uint64_t nvim_buf_update_channels_get(buf_T *buf, size_t i)
{
  return kv_A(buf->update_channels, i);
}

/// Set the channel ID at index i in update_channels.
void nvim_buf_update_channels_set(buf_T *buf, size_t i, uint64_t channel_id)
{
  kv_A(buf->update_channels, i) = channel_id;
}

/// Shrink update_channels by removing `count` items from the end.
void nvim_buf_update_channels_shrink(buf_T *buf, size_t count)
{
  buf->update_channels.size -= count;
}

/// Push a channel ID onto update_channels.
void nvim_buf_update_channels_push(buf_T *buf, uint64_t channel_id)
{
  kv_push(buf->update_channels, channel_id);
}

/// Destroy and reinitialize update_channels (free allocated memory).
void nvim_buf_update_channels_destroy(buf_T *buf)
{
  kv_destroy(buf->update_channels);
  kv_init(buf->update_channels);
}

/// Push a BufUpdateCallbacks entry onto update_callbacks.
void nvim_buf_update_callbacks_push(buf_T *buf, BufUpdateCallbacks cb)
{
  kv_push(buf->update_callbacks, cb);
}

/// Set update_need_codepoints for a buffer.
void nvim_buf_set_update_need_codepoints(buf_T *buf, bool val)
{
  buf->update_need_codepoints = val;
}

/// Send the initial buffer contents as nvim_buf_lines_event to a channel.
/// This handles arena allocation, buf_collect_lines, and rpc_send_event.
void nvim_buf_send_initial_lines(buf_T *buf, uint64_t channel_id)
{
  MAXSIZE_TEMP_ARRAY(args, 6);
  ADD_C(args, BUFFER_OBJ(buf->handle));
  ADD_C(args, INTEGER_OBJ(buf_get_changedtick(buf)));
  ADD_C(args, INTEGER_OBJ(0));
  ADD_C(args, INTEGER_OBJ(-1));

  STATIC_ASSERT(SIZE_MAX >= MAXLNUM, "size_t smaller than MAXLNUM");
  size_t line_count = (size_t)buf->b_ml.ml_line_count;

  Array linedata = ARRAY_DICT_INIT;
  Arena arena = ARENA_EMPTY;
  if (line_count > 0) {
    linedata = arena_array(&arena, line_count);
    buf_collect_lines(buf, line_count, 1, 0, true, &linedata, NULL, &arena);
  }

  ADD_C(args, ARRAY_OBJ(linedata));
  ADD_C(args, BOOLEAN_OBJ(false));

  rpc_send_event(channel_id, "nvim_buf_lines_event", args);
  arena_mem_free(arena_finish(&arena));
}

// Phase 3 accessor functions for Rust FFI (callback notifications)

/// Get a BufUpdateCallbacks entry at index i.
BufUpdateCallbacks nvim_buf_update_callbacks_get(buf_T *buf, size_t i)
{
  return kv_A(buf->update_callbacks, i);
}

/// Set a BufUpdateCallbacks entry at index i.
void nvim_buf_update_callbacks_set(buf_T *buf, size_t i, BufUpdateCallbacks cb)
{
  kv_A(buf->update_callbacks, i) = cb;
}

/// Set the size of update_callbacks (for compaction).
void nvim_buf_update_callbacks_set_size(buf_T *buf, size_t new_size)
{
  kv_size(buf->update_callbacks) = new_size;
}

/// Destroy and reinitialize update_callbacks (free allocated memory).
void nvim_buf_update_callbacks_destroy(buf_T *buf)
{
  kv_destroy(buf->update_callbacks);
  kv_init(buf->update_callbacks);
}

/// Returns true if cmdpreview is active AND buf is the current buffer.
bool nvim_buf_is_cmdpreview_curbuf(buf_T *buf)
{
  return cmdpreview && buf == curbuf;
}

/// Send nvim_buf_lines_event to all registered channels for a change event.
/// Handles arena allocation, rpc_send_event, and unregistering dead channels.
/// Returns the bad channel ID (0 if all channels are alive).
uint64_t nvim_buf_send_lines_to_channels(buf_T *buf, int64_t firstline, int64_t num_added,
                                          int64_t num_removed, bool send_tick)
{
  Arena arena = ARENA_EMPTY;
  Array linedata = ARRAY_DICT_INIT;
  if (num_added > 0 && kv_size(buf->update_channels)) {
    STATIC_ASSERT(SIZE_MAX >= MAXLNUM, "size_t smaller than MAXLNUM");
    linedata = arena_array(&arena, (size_t)num_added);
    buf_collect_lines(buf, (size_t)num_added, (linenr_T)firstline, 0, true, &linedata,
                      NULL, &arena);
  }

  uint64_t badchannelid = 0;
  for (size_t i = 0; i < kv_size(buf->update_channels); i++) {
    uint64_t channelid = kv_A(buf->update_channels, i);
    MAXSIZE_TEMP_ARRAY(args, 6);
    ADD_C(args, BUFFER_OBJ(buf->handle));
    ADD_C(args, send_tick ? INTEGER_OBJ(buf_get_changedtick(buf)) : NIL);
    ADD_C(args, INTEGER_OBJ(firstline - 1));
    ADD_C(args, INTEGER_OBJ(firstline - 1 + num_removed));
    ADD_C(args, ARRAY_OBJ(linedata));
    ADD_C(args, BOOLEAN_OBJ(false));
    if (!rpc_send_event(channelid, "nvim_buf_lines_event", args)) {
      badchannelid = channelid;
    }
  }

  arena_mem_free(arena_finish(&arena));
  return badchannelid;
}

/// Log an error about a dead channel and unregister it.
void nvim_buf_log_dead_channel(buf_T *buf, uint64_t channel_id)
{
  ELOG("Disabling buffer updates for dead channel %" PRIu64, channel_id);
  buf_updates_unregister(buf, channel_id);
}

/// Call on_lines callback for buf_updates_send_changes.
/// Returns true if the callback returned truthy (callback wants to unregister).
bool nvim_buf_call_on_lines(buf_T *buf, LuaRef on_lines, bool send_tick, bool utf_sizes,
                             int64_t firstline, int64_t num_added, int64_t num_removed,
                             size_t deleted_bytes, size_t deleted_codepoints,
                             size_t deleted_codeunits)
{
  MAXSIZE_TEMP_ARRAY(args, 8);
  ADD_C(args, BUFFER_OBJ(buf->handle));
  ADD_C(args, send_tick ? INTEGER_OBJ(buf_get_changedtick(buf)) : NIL);
  ADD_C(args, INTEGER_OBJ(firstline - 1));
  ADD_C(args, INTEGER_OBJ(firstline - 1 + num_removed));
  ADD_C(args, INTEGER_OBJ(firstline - 1 + num_added));
  ADD_C(args, INTEGER_OBJ((Integer)deleted_bytes));
  if (utf_sizes) {
    ADD_C(args, INTEGER_OBJ((Integer)deleted_codepoints));
    ADD_C(args, INTEGER_OBJ((Integer)deleted_codeunits));
  }
  Object res;
  TEXTLOCK_WRAP({
    res = nlua_call_ref(on_lines, "lines", args, kRetNilBool, NULL, NULL);
  });
  return LUARET_TRUTHY(res);
}

/// Call on_bytes callback for buf_updates_send_splice.
/// Returns true if the callback returned truthy (callback wants to unregister).
bool nvim_buf_call_on_bytes(buf_T *buf, LuaRef on_bytes,
                             int start_row, int start_col, int64_t start_byte,
                             int old_row, int old_col, int64_t old_byte,
                             int new_row, int new_col, int64_t new_byte)
{
  MAXSIZE_TEMP_ARRAY(args, 11);
  ADD_C(args, BUFFER_OBJ(buf->handle));
  ADD_C(args, INTEGER_OBJ(buf_get_changedtick(buf)));
  ADD_C(args, INTEGER_OBJ(start_row));
  ADD_C(args, INTEGER_OBJ(start_col));
  ADD_C(args, INTEGER_OBJ(start_byte));
  ADD_C(args, INTEGER_OBJ(old_row));
  ADD_C(args, INTEGER_OBJ(old_col));
  ADD_C(args, INTEGER_OBJ(old_byte));
  ADD_C(args, INTEGER_OBJ(new_row));
  ADD_C(args, INTEGER_OBJ(new_col));
  ADD_C(args, INTEGER_OBJ(new_byte));
  Object res;
  TEXTLOCK_WRAP({
    res = nlua_call_ref(on_bytes, "bytes", args, kRetNilBool, NULL, NULL);
  });
  return LUARET_TRUTHY(res);
}

/// Call on_changedtick callback for buf_updates_changedtick.
/// Returns true if the callback returned truthy (callback wants to unregister).
bool nvim_buf_call_on_changedtick(buf_T *buf, LuaRef on_changedtick)
{
  MAXSIZE_TEMP_ARRAY(args, 2);
  ADD_C(args, BUFFER_OBJ(buf->handle));
  ADD_C(args, INTEGER_OBJ(buf_get_changedtick(buf)));
  Object res;
  TEXTLOCK_WRAP({
    res = nlua_call_ref(on_changedtick, "changedtick", args, kRetNilBool, NULL, NULL);
  });
  return LUARET_TRUTHY(res);
}

// Phase 4 helper functions for Rust FFI (unload and cleanup)

/// Call reload or detach callback for buf_updates_unload.
/// Returns true if the callback was the reload callback (i.e., we should keep this entry).
/// Returns false if the callback was the detach callback (or no callback at all).
bool nvim_buf_call_reload_or_detach(buf_T *buf, BufUpdateCallbacks cb, bool can_reload)
{
  LuaRef thecb = LUA_NOREF;
  bool keep = false;

  if (can_reload && cb.on_reload != LUA_NOREF) {
    keep = true;
    thecb = cb.on_reload;
  } else if (cb.on_detach != LUA_NOREF) {
    thecb = cb.on_detach;
  }

  if (thecb != LUA_NOREF) {
    MAXSIZE_TEMP_ARRAY(args, 1);
    ADD_C(args, BUFFER_OBJ(buf->handle));
    TEXTLOCK_WRAP({
      nlua_call_ref(thecb, keep ? "reload" : "detach", args, false, NULL, NULL);
    });
  }

  return keep;
}

// Extmark Accessor Functions (for Rust FFI - extmark crate)

/// Send splice event (wrapper for Rust FFI).
void nvim_buf_updates_send_splice(buf_T *buf, int start_row, colnr_T start_col,
                                  bcount_t start_byte, int old_row, colnr_T old_col,
                                  bcount_t old_byte, int new_row, colnr_T new_col,
                                  bcount_t new_byte)
{
  buf_updates_send_splice(buf, start_row, start_col, start_byte, old_row, old_col, old_byte,
                          new_row, new_col, new_byte);
}
