#pragma once

#include <stdint.h>  // IWYU pragma: keep

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/extmark_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep

bool buf_updates_active(buf_T *buf);

#include "buffer_updates.h.generated.h"
