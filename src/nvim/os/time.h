#pragma once

#include <stdbool.h>
#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>  // IWYU pragma: keep
#include <time.h>  // IWYU pragma: keep

#include "nvim/os/time_defs.h"  // IWYU pragma: keep

// Functions implemented in Rust (src/nvim-rs/os/src/time.rs)
uint64_t os_hrtime(void);
Timestamp os_time(void);
void os_sleep(uint64_t ms);

#include "os/time.h.generated.h"
