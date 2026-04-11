/// memory_ffi.c: Generic memory allocator wrappers for Rust FFI.
///
/// These thin wrappers provide a stable C ABI for Rust crates to call into
/// Neovim's memory allocator (xmalloc / xfree family).  They are shared
/// across all Rust crates in src/nvim-rs/ that need dynamic allocation.

#include <stddef.h>

#include "nvim/memory.h"

#include "memory_ffi.c.generated.h"

/// Free memory allocated by Neovim's allocator.
void nvim_xfree(void *ptr) { xfree(ptr); }

/// Duplicate a NUL-terminated string.
char *nvim_xstrdup(const char *str) { return xstrdup(str); }

/// Allocate `size` bytes (aborts on OOM).
void *nvim_xmalloc(size_t size) { return xmalloc(size); }

/// Allocate `count * size` bytes, zero-initialised (aborts on OOM).
void *nvim_xcalloc(size_t count, size_t size) { return xcalloc(count, size); }

/// Resize an allocation (aborts on OOM).
void *nvim_xrealloc(void *ptr, size_t size) { return xrealloc(ptr, size); }

/// Allocate `size` bytes with a NUL terminator at position `size` (aborts on OOM).
char *nvim_xmallocz(size_t size) { return xmallocz(size); }
