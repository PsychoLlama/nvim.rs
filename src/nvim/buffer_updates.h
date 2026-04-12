#pragma once

#include <stdint.h>  // IWYU pragma: keep

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/extmark_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep

bool buf_updates_active(buf_T *buf);

#include "buffer_updates.h.generated.h"

// Declarations for functions migrated to Rust (buffer_updates crate).
void buffer_update_callbacks_free(BufUpdateCallbacks cb);
void buf_updates_changedtick_single(buf_T *buf, uint64_t channel_id);
void buf_updates_send_end(buf_T *buf, uint64_t channelid);
bool buf_updates_register(buf_T *buf, uint64_t channel_id, BufUpdateCallbacks cb, bool send_buffer);
void buf_updates_unregister(buf_T *buf, uint64_t channelid);
void buf_updates_send_changes(buf_T *buf, linenr_T firstline, int64_t num_added, int64_t num_removed);
void buf_updates_send_splice(buf_T *buf, int start_row, colnr_T start_col, bcount_t start_byte, int old_row, colnr_T old_col, bcount_t old_byte, int new_row, colnr_T new_col, bcount_t new_byte);
void buf_updates_changedtick(buf_T *buf);
