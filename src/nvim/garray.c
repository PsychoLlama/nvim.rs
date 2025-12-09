/// @file garray.c
///
/// Functions for handling growing arrays.

#include <stdint.h>
#include <string.h>

#include "nvim/garray.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/path.h"
#include "nvim/strings.h"

#include "garray.c.generated.h"  // IWYU pragma: keep

#ifdef USE_RUST_GARRAY
// Rust implementations
extern void rs_ga_init(garray_T *gap, int itemsize, int growsize);
extern void rs_ga_set_growsize(garray_T *gap, int growsize);
extern void rs_ga_clear(garray_T *gap);
extern void rs_ga_grow(garray_T *gap, int n);
extern void rs_ga_concat(garray_T *gap, const char *s);
extern void rs_ga_concat_len(garray_T *gap, const char *s, size_t len);
extern void rs_ga_append(garray_T *gap, uint8_t c);
extern void *rs_ga_append_via_ptr(garray_T *gap, size_t item_size);
#endif

/// Clear an allocated growing array.
void ga_clear(garray_T *gap)
{
#ifdef USE_RUST_GARRAY
  rs_ga_clear(gap);
#else
  xfree(gap->ga_data);

  // Initialize growing array without resetting itemsize or growsize
  gap->ga_data = NULL;
  gap->ga_maxlen = 0;
  gap->ga_len = 0;
#endif
}

/// Clear a growing array that contains a list of strings.
///
/// @param gap
void ga_clear_strings(garray_T *gap)
{
  GA_DEEP_CLEAR_PTR(gap);
}

/// Initialize a growing array.
///
/// @param gap
/// @param itemsize
/// @param growsize
void ga_init(garray_T *gap, int itemsize, int growsize)
{
#ifdef USE_RUST_GARRAY
  rs_ga_init(gap, itemsize, growsize);
#else
  gap->ga_data = NULL;
  gap->ga_maxlen = 0;
  gap->ga_len = 0;
  gap->ga_itemsize = itemsize;
  ga_set_growsize(gap, growsize);
#endif
}

/// A setter for the growsize that guarantees it will be at least 1.
///
/// @param gap
/// @param growsize
void ga_set_growsize(garray_T *gap, int growsize)
{
#ifdef USE_RUST_GARRAY
  rs_ga_set_growsize(gap, growsize);
#else
  if (growsize < 1) {
    WLOG("trying to set an invalid ga_growsize: %d", growsize);
    gap->ga_growsize = 1;
  } else {
    gap->ga_growsize = growsize;
  }
#endif
}

/// Make room in growing array "gap" for at least "n" items.
///
/// @param gap
/// @param n
void ga_grow(garray_T *gap, int n)
{
#ifdef USE_RUST_GARRAY
  rs_ga_grow(gap, n);
#else
  if (gap->ga_maxlen - gap->ga_len >= n) {
    // the garray still has enough space, do nothing
    return;
  }

  if (gap->ga_growsize < 1) {
    WLOG("ga_growsize(%d) is less than 1", gap->ga_growsize);
  }

  // the garray grows by at least growsize
  n = MAX(n, gap->ga_growsize);

  // A linear growth is very inefficient when the array grows big.  This
  // is a compromise between allocating memory that won't be used and too
  // many copy operations. A factor of 1.5 seems reasonable.
  n = MAX(n, gap->ga_len / 2);

  int new_maxlen = gap->ga_len + n;

  size_t new_size = (size_t)gap->ga_itemsize * (size_t)new_maxlen;
  size_t old_size = (size_t)gap->ga_itemsize * (size_t)gap->ga_maxlen;

  // reallocate and clear the new memory
  char *pp = xrealloc(gap->ga_data, new_size);
  memset(pp + old_size, 0, new_size - old_size);

  gap->ga_maxlen = new_maxlen;
  gap->ga_data = pp;
#endif
}

/// Sort "gap" and remove duplicate entries. "gap" is expected to contain a
/// list of file names in allocated memory.
///
/// @param gap
void ga_remove_duplicate_strings(garray_T *gap)
{
  char **fnames = gap->ga_data;

  // sort the growing array, which puts duplicates next to each other
  sort_strings(fnames, gap->ga_len);

  // loop over the growing array in reverse
  for (int i = gap->ga_len - 1; i > 0; i--) {
    if (path_fnamecmp(fnames[i - 1], fnames[i]) == 0) {
      xfree(fnames[i]);

      // close the gap (move all strings one slot lower)
      for (int j = i + 1; j < gap->ga_len; j++) {
        fnames[j - 1] = fnames[j];
      }

      gap->ga_len--;
    }
  }
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
  const size_t nelem = (size_t)gap->ga_len;
  const char **strings = gap->ga_data;

  if (nelem == 0) {
    return xstrdup("");
  }

  size_t len = 0;
  for (size_t i = 0; i < nelem; i++) {
    len += strlen(strings[i]);
  }

  // add some space for the (num - 1) separators
  len += (nelem - 1) * strlen(sep);
  char *const ret = xmallocz(len);

  char *s = ret;
  for (size_t i = 0; i < nelem - 1; i++) {
    s = xstpcpy(s, strings[i]);
    s = xstpcpy(s, sep);
  }
  strcpy(s, strings[nelem - 1]);  // NOLINT(runtime/printf)

  return ret;
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
#ifdef USE_RUST_GARRAY
  rs_ga_concat(gap, s);
#else
  if (s == NULL) {
    return;
  }

  ga_concat_len(gap, s, strlen(s));
#endif
}

/// Concatenate a string to a growarray which contains characters
///
/// @param[out]  gap  Growarray to modify.
/// @param[in]  s  String to concatenate.
/// @param[in]  len  String length.
void ga_concat_len(garray_T *const gap, const char *restrict s, const size_t len)
  FUNC_ATTR_NONNULL_ALL
{
#ifdef USE_RUST_GARRAY
  rs_ga_concat_len(gap, s, len);
#else
  if (len == 0) {
    return;
  }
  ga_grow(gap, (int)len);
  char *data = gap->ga_data;
  memcpy(data + gap->ga_len, s, len);
  gap->ga_len += (int)len;
#endif
}

/// Append one byte to a growarray which contains bytes.
///
/// @param gap
/// @param c
void ga_append(garray_T *gap, uint8_t c)
{
#ifdef USE_RUST_GARRAY
  rs_ga_append(gap, c);
#else
  GA_APPEND(uint8_t, gap, c);
#endif
}

void *ga_append_via_ptr(garray_T *gap, size_t item_size)
{
#ifdef USE_RUST_GARRAY
  return rs_ga_append_via_ptr(gap, item_size);
#else
  if ((int)item_size != gap->ga_itemsize) {
    WLOG("wrong item size (%zu), should be %d", item_size, gap->ga_itemsize);
  }
  ga_grow(gap, 1);
  return ((char *)gap->ga_data) + (item_size * (size_t)gap->ga_len++);
#endif
}
