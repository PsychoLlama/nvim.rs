// memory_accessors.c: C accessor functions for memory.c globals, used by Rust FFI.

#include <stddef.h>

#include "nvim/memory.h"

void nvim_inc_arena_alloc_count(void)
{
  arena_alloc_count++;
}
