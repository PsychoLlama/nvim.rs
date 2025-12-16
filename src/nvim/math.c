// uncrustify:off
#include <math.h>
// uncrustify:on
#include <limits.h>
#include <stdint.h>
#include <string.h>

#ifdef HAVE_BITSCANFORWARD64
# include <intrin.h>  // Required for _BitScanForward64
#endif

#include "nvim/math.h"
#include "nvim/vim_defs.h"

#include "math.c.generated.h"

// Rust implementations - declarations
extern int rs_xfpclassify(double d);
extern int rs_xisinf(double d);
extern int rs_xisnan(double d);
extern int rs_xctz(uint64_t x);
extern unsigned int rs_xpopcount(uint64_t x);
extern int rs_vim_append_digit_int(int *value, int digit);
extern int rs_trim_to_int(int64_t x);

// Map Rust FP constants to libc constants
// Rust: FP_NAN=0, FP_INFINITE=1, FP_ZERO=2, FP_SUBNORMAL=3, FP_NORMAL=4
static int rs_fp_to_libc(int rs_val)
{
  switch (rs_val) {
  case 0:
    return FP_NAN;
  case 1:
    return FP_INFINITE;
  case 2:
    return FP_ZERO;
  case 3:
    return FP_SUBNORMAL;
  case 4:
    return FP_NORMAL;
  default:
    return FP_NORMAL;
  }
}

int xfpclassify(double d)
  FUNC_ATTR_CONST
{
  return rs_fp_to_libc(rs_xfpclassify(d));
}

int xisinf(double d)
  FUNC_ATTR_CONST
{
  return rs_xisinf(d);
}

int xisnan(double d)
  FUNC_ATTR_CONST
{
  return rs_xisnan(d);
}

/// Count trailing zeroes at the end of bit field.
int xctz(uint64_t x)
{
  return rs_xctz(x);
}

/// Count number of set bits in bit field.
unsigned xpopcount(uint64_t x)
{
  return rs_xpopcount(x);
}

/// For overflow detection, add a digit safely to an int value.
int vim_append_digit_int(int *value, int digit)
{
  return rs_vim_append_digit_int(value, digit);
}

/// Return something that fits into an int.
int trim_to_int(int64_t x)
{
  return rs_trim_to_int(x);
}
