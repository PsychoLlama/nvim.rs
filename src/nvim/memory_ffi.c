/// memory_ffi.c: Generic memory allocator wrappers for Rust FFI.
///
/// These thin wrappers provide a stable C ABI for Rust crates to call into
/// Neovim's memory allocator (xmalloc / xfree family).  They are shared
/// across all Rust crates in src/nvim-rs/ that need dynamic allocation.

#include <stddef.h>

#include "nvim/memory.h"

#include "memory_ffi.c.generated.h"

