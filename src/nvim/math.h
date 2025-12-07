#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifdef USE_RUST_MATH
extern int rs_is_power_of_two(uint64_t x);
#endif

/// Check if number is a power of two
static inline bool is_power_of_two(uint64_t x)
{
#ifdef USE_RUST_MATH
  return rs_is_power_of_two(x) != 0;
#else
  return x != 0 && ((x & (x - 1)) == 0);
#endif
}

#include "math.h.generated.h"
