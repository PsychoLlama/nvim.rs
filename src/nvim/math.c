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

#ifdef USE_RUST_MATH
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
#endif

int xfpclassify(double d)
  FUNC_ATTR_CONST
{
#ifdef USE_RUST_MATH
  return rs_fp_to_libc(rs_xfpclassify(d));
#else
  uint64_t m;

  memcpy(&m, &d, sizeof(m));
  int e = 0x7ff & (m >> 52);
  m = 0xfffffffffffffULL & m;

  switch (e) {
  default:
    return FP_NORMAL;
  case 0x000:
    return m ? FP_SUBNORMAL : FP_ZERO;
  case 0x7ff:
    return m ? FP_NAN : FP_INFINITE;
  }
#endif
}

int xisinf(double d)
  FUNC_ATTR_CONST
{
#ifdef USE_RUST_MATH
  return rs_xisinf(d);
#else
  return FP_INFINITE == xfpclassify(d);
#endif
}

int xisnan(double d)
  FUNC_ATTR_CONST
{
#ifdef USE_RUST_MATH
  return rs_xisnan(d);
#else
  return FP_NAN == xfpclassify(d);
#endif
}

/// Count trailing zeroes at the end of bit field.
int xctz(uint64_t x)
{
#ifdef USE_RUST_MATH
  return rs_xctz(x);
#else
  // If x == 0, that means all bits are zeroes.
  if (x == 0) {
    return 8 * sizeof(x);
  }

  // Use compiler builtin if possible.
#if defined(__clang__) || (defined(__GNUC__) && (__GNUC__ >= 4))
  return __builtin_ctzll(x);
#elif defined(HAVE_BITSCANFORWARD64)
  unsigned long index;
  _BitScanForward64(&index, x);
  return (int)index;
#else
  int count = 0;
  // Set x's trailing zeroes to ones and zero the rest.
  x = (x ^ (x - 1)) >> 1;

  // Increment count until there are just zero bits remaining.
  while (x) {
    count++;
    x >>= 1;
  }

  return count;
#endif
#endif  // USE_RUST_MATH
}

/// Count number of set bits in bit field.
unsigned xpopcount(uint64_t x)
{
#ifdef USE_RUST_MATH
  return rs_xpopcount(x);
#else
  // Use compiler builtin if possible.
#if defined(__NetBSD__)
  return popcount64(x);
#elif defined(__clang__) || defined(__GNUC__)
  return (unsigned)__builtin_popcountll(x);
#else
  unsigned count = 0;
  for (; x != 0; x >>= 1) {
    if (x & 1) {
      count++;
    }
  }
  return count;
#endif
#endif  // USE_RUST_MATH
}

/// For overflow detection, add a digit safely to an int value.
int vim_append_digit_int(int *value, int digit)
{
#ifdef USE_RUST_MATH
  return rs_vim_append_digit_int(value, digit);
#else
  int x = *value;
  if (x > ((INT_MAX - digit) / 10)) {
    return FAIL;
  }
  *value = x * 10 + digit;
  return OK;
#endif
}

/// Return something that fits into an int.
int trim_to_int(int64_t x)
{
#ifdef USE_RUST_MATH
  return rs_trim_to_int(x);
#else
  return x > INT_MAX ? INT_MAX : x < INT_MIN ? INT_MIN : (int)x;
#endif
}
