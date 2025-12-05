/// Functions for accessing system memory information.

#include <stdint.h>
#include <uv.h>

#include "nvim/os/os.h"

#ifdef USE_RUST_OS_MEM
extern uint64_t rs_os_get_total_mem_kib(void);
#endif

/// Get the total system physical memory in KiB.
uint64_t os_get_total_mem_kib(void)
{
#ifdef USE_RUST_OS_MEM
  return rs_os_get_total_mem_kib();
#else
  // Convert bytes to KiB.
  return uv_get_total_memory() / 1024;
#endif
}
