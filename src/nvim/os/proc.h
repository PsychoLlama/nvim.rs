#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep

// Functions implemented in Rust (nvim-os crate)
extern bool os_proc_running(int pid);

#include "os/proc.h.generated.h"
