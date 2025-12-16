#pragma once

#include <stdbool.h>
#include <stdint.h>

extern int rs_is_power_of_two(uint64_t x);

/// Check if number is a power of two
static inline bool is_power_of_two(uint64_t x)
{
  return rs_is_power_of_two(x) != 0;
}

#include "math.h.generated.h"
