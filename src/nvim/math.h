#pragma once

#include <stdbool.h>
#include <stdint.h>

extern int rs_is_power_of_two(uint64_t x);

/// Check if number is a power of two
static inline bool is_power_of_two(uint64_t x)
{
  return rs_is_power_of_two(x) != 0;
}

int xfpclassify(double d);
int xisinf(double d);
int xisnan(double d);
int xctz(uint64_t x);
unsigned xpopcount(uint64_t x);
int vim_append_digit_int(int *value, int digit);
int trim_to_int(int64_t x);
