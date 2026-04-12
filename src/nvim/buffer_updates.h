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
