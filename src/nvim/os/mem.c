/// Functions for accessing system memory information.

#include <stdint.h>
#include <uv.h>

#include "nvim/os/os.h"

extern uint64_t rs_os_get_total_mem_kib(void);

/// Get the total system physical memory in KiB.
uint64_t os_get_total_mem_kib(void)
{
  return rs_os_get_total_mem_kib();
}
