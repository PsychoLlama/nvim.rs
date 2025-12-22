/// @file garray.c
///
/// Functions for handling growing arrays.

#include <stdint.h>

#include "nvim/garray.h"
#include "nvim/memory.h"
#include "nvim/path.h"
#include "nvim/strings.h"

#include "garray.c.generated.h"  // IWYU pragma: keep

// Rust implementations
extern void rs_ga_init(garray_T *gap, int itemsize, int growsize);
extern void rs_ga_set_growsize(garray_T *gap, int growsize);
extern void rs_ga_clear(garray_T *gap);
extern void rs_ga_grow(garray_T *gap, int n);
extern void rs_ga_concat(garray_T *gap, const char *s);
extern void rs_ga_concat_len(garray_T *gap, const char *s, size_t len);
extern void rs_ga_append(garray_T *gap, uint8_t c);
extern void *rs_ga_append_via_ptr(garray_T *gap, size_t item_size);
extern void rs_ga_clear_strings(garray_T *gap);
extern char *rs_ga_concat_strings(const garray_T *gap, const char *sep);
extern void rs_ga_remove_duplicate_strings(garray_T *gap);

/// Clear an allocated growing array.
void ga_clear(garray_T *gap)
{
  rs_ga_clear(gap);
}

/// Clear a growing array that contains a list of strings.
///
/// @param gap
void ga_clear_strings(garray_T *gap)
{
  rs_ga_clear_strings(gap);
}

/// Initialize a growing array.
///
/// @param gap
/// @param itemsize
/// @param growsize
void ga_init(garray_T *gap, int itemsize, int growsize)
{
  rs_ga_init(gap, itemsize, growsize);
}

/// A setter for the growsize that guarantees it will be at least 1.
///
/// @param gap
/// @param growsize
void ga_set_growsize(garray_T *gap, int growsize)
{
  rs_ga_set_growsize(gap, growsize);
}

/// Make room in growing array "gap" for at least "n" items.
///
/// @param gap
/// @param n
void ga_grow(garray_T *gap, int n)
{
  rs_ga_grow(gap, n);
}

/// Sort "gap" and remove duplicate entries. "gap" is expected to contain a
/// list of file names in allocated memory.
///
/// @param gap
void ga_remove_duplicate_strings(garray_T *gap)
{
  rs_ga_remove_duplicate_strings(gap);
}

/// For a growing array that contains a list of strings: concatenate all the
/// strings with "sep" as separator.
///
/// @param gap
/// @param sep
///
/// @returns the concatenated strings
char *ga_concat_strings(const garray_T *gap, const char *sep)
  FUNC_ATTR_NONNULL_RET
{
  return rs_ga_concat_strings(gap, sep);
}

/// Concatenate a string to a growarray which contains characters.
/// When "s" is NULL does not do anything.
///
/// WARNING:
/// - Does NOT copy the NUL at the end!
/// - The parameter may not overlap with the growing array
///
/// @param gap
/// @param s
void ga_concat(garray_T *gap, const char *restrict s)
{
  rs_ga_concat(gap, s);
}

/// Concatenate a string to a growarray which contains characters
///
/// @param[out]  gap  Growarray to modify.
/// @param[in]  s  String to concatenate.
/// @param[in]  len  String length.
void ga_concat_len(garray_T *const gap, const char *restrict s, const size_t len)
  FUNC_ATTR_NONNULL_ALL
{
  rs_ga_concat_len(gap, s, len);
}

/// Append one byte to a growarray which contains bytes.
///
/// @param gap
/// @param c
void ga_append(garray_T *gap, uint8_t c)
{
  rs_ga_append(gap, c);
}

void *ga_append_via_ptr(garray_T *gap, size_t item_size)
{
  return rs_ga_append_via_ptr(gap, item_size);
}
