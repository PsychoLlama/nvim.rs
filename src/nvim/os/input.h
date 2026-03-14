#pragma once

#include <stdbool.h>
#include <stdint.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/event/defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"

EXTERN bool used_stdin INIT( = false);

// Functions implemented in Rust (nvim-os crate)
extern bool os_isatty(int fd);
extern bool input_blocking(void);

#include "os/input.h.generated.h"
