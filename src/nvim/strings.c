#include <assert.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"

#include "strings.c.generated.h"

// TYPE_* constants -- values must match rs_format_typeof return values
enum {
  TYPE_UNKNOWN = -1,
  TYPE_INT,
  TYPE_LONGINT,
  TYPE_LONGLONGINT,
  TYPE_SIGNEDSIZET,
  TYPE_UNSIGNEDINT,
  TYPE_UNSIGNEDLONGINT,
  TYPE_UNSIGNEDLONGLONGINT,
  TYPE_SIZET,
  TYPE_POINTER,
  TYPE_PERCENT,
  TYPE_CHAR,
  TYPE_STRING,
  TYPE_FLOAT,
};

/// Extract all va_list arguments into the FmtArg array.
///
/// @param[in]  ap_types  Type array from rs_parse_fmt_types (may be NULL for tvs path).
/// @param[in]  num_args  Number of arguments to extract.
/// @param[in]  ap        The va_list to read from (already va_copy'd).
/// @param[out] args      Output array, must have room for num_args entries.
static void extract_va_args(const char **ap_types, int num_args, va_list ap, FmtArg *args)
  FUNC_ATTR_NONNULL_ARG(4)
{
  for (int i = 0; i < num_args; i++) {
    if (ap_types == NULL || ap_types[i] == NULL) {
      args[i].tag = TYPE_UNKNOWN;
      continue;
    }
    int fmt_type = rs_format_typeof(ap_types[i]);
    args[i].tag = fmt_type;
    switch (fmt_type) {
    case TYPE_PERCENT:
    case TYPE_UNKNOWN:
      break;
    case TYPE_CHAR:
      args[i].val.i = va_arg(ap, int);
      break;
    case TYPE_STRING:
      args[i].val.s = va_arg(ap, const char *);
      break;
    case TYPE_POINTER:
      args[i].val.p = va_arg(ap, void *);
      break;
    case TYPE_INT:
      args[i].val.i = va_arg(ap, int);
      break;
    case TYPE_LONGINT:
      args[i].val.l = va_arg(ap, long);
      break;
    case TYPE_LONGLONGINT:
      args[i].val.ll = va_arg(ap, long long);  // NOLINT(runtime/int)
      break;
    case TYPE_SIGNEDSIZET:
      args[i].val.z = va_arg(ap, ptrdiff_t);
      break;
    case TYPE_UNSIGNEDINT:
      args[i].val.u = va_arg(ap, unsigned);
      break;
    case TYPE_UNSIGNEDLONGINT:
      args[i].val.ul = va_arg(ap, unsigned long);
      break;
    case TYPE_UNSIGNEDLONGLONGINT:
      args[i].val.ull = va_arg(ap, unsigned long long);  // NOLINT(runtime/int)
      break;
    case TYPE_SIZET:
      args[i].val.uz = va_arg(ap, size_t);
      break;
    case TYPE_FLOAT:
      args[i].val.f = va_arg(ap, double);
      break;
    }
  }
}

/// Count non-positional format arguments needed (when num_posarg == 0).
static int count_fmt_args(const char **ap_types)
{
  if (ap_types == NULL) {
    return 0;
  }
  int n = 0;
  while (ap_types[n] != NULL) {
    n++;
  }
  return n;
}

/// Append a formatted value to the string
///
/// @see vim_vsnprintf_typval().
int vim_snprintf_add(char *str, size_t str_m, const char *fmt, ...)
  FUNC_ATTR_PRINTF(3, 4)
{
  const size_t len = strlen(str);
  size_t space;

  if (str_m <= len) {
    space = 0;
  } else {
    space = str_m - len;
  }
  va_list ap;
  va_start(ap, fmt);
  const int str_l = vim_vsnprintf(str + len, space, fmt, ap);
  va_end(ap);
  return str_l;
}

/// Write formatted value to the string
///
/// @param[out]  str  String to write to.
/// @param[in]  str_m  String length.
/// @param[in]  fmt  String format.
///
/// @return Number of bytes excluding NUL byte that would be written to the
///         string if str_m was greater or equal to the return value.
int vim_snprintf(char *str, size_t str_m, const char *fmt, ...)
  FUNC_ATTR_PRINTF(3, 4)
{
  va_list ap;
  va_start(ap, fmt);
  const int str_l = vim_vsnprintf(str, str_m, fmt, ap);
  va_end(ap);
  return str_l;
}

/// Like vim_snprintf() except the return value can be safely used to increment a
/// buffer length.
size_t vim_snprintf_safelen(char *str, size_t str_m, const char *fmt, ...)
{
  va_list ap;
  int str_l;

  va_start(ap, fmt);
  str_l = vim_vsnprintf_typval(str, str_m, fmt, ap, NULL);
  va_end(ap);

  if (str_l < 0) {
    *str = NUL;
    return 0;
  }
  return ((size_t)str_l >= str_m) ? str_m - 1 : (size_t)str_l;
}

int vim_vsnprintf(char *str, size_t str_m, const char *fmt, va_list ap)
{
  return vim_vsnprintf_typval(str, str_m, fmt, ap, NULL);
}

/// Wrapper for vim_snprintf callable from Rust.
int rs_vim_snprintf(char *str, size_t str_m, const char *fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  const int str_l = vim_vsnprintf(str, str_m, fmt, ap);
  va_end(ap);
  return str_l;
}

/// Write formatted value to the string (thin wrapper around Rust engine).
///
/// @param[out]  str  String to write to.
/// @param[in]  str_m  String length.
/// @param[in]  fmt  String format.
/// @param[in]  ap_start  va_list; ignored when tvs != NULL.
/// @param[in]  tvs  Typval array for VimL printf(); NULL for C callers.
///
/// @return Number of bytes that would be written excluding NUL.
int vim_vsnprintf_typval(char *str, size_t str_m, const char *fmt, va_list ap_start,
                         typval_T *const tvs)
{
  const char **ap_types = NULL;
  int num_posarg = 0;

  if (rs_parse_fmt_types(&ap_types, &num_posarg, fmt, tvs) == FAIL) {
    return 0;
  }

  int num_args = (num_posarg != 0) ? num_posarg : count_fmt_args(ap_types);

  FmtArg args[64];
  memset(args, 0, sizeof(args));

  if (tvs == NULL && num_args > 0) {
    va_list ap;
    va_copy(ap, ap_start);
    extract_va_args(ap_types, num_args, ap, args);
    va_end(ap);
  }

  int result = rs_vim_vsnprintf_extracted(str, str_m, fmt,
                                          tvs == NULL ? (const FmtArg *)args : NULL,
                                          num_args, ap_types, num_posarg, tvs);
  xfree(ap_types);
  return result;
}

int kv_do_printf(StringBuilder *str, const char *fmt, ...)
  FUNC_ATTR_PRINTF(2, 3)
{
  size_t remaining = str->capacity - str->size;

  va_list ap;
  va_start(ap, fmt);
  int printed = vsnprintf(str->items ? str->items + str->size : NULL, remaining, fmt, ap);
  va_end(ap);

  if (printed < 0) {
    return -1;
  }

  // printed string didn't fit, resize and try again
  if ((size_t)printed >= remaining) {
    kv_ensure_space(*str, (size_t)printed + 1);  // include space for NUL terminator at the end
    assert(str->items != NULL);
    va_start(ap, fmt);
    printed = vsnprintf(str->items + str->size, str->capacity - str->size, fmt, ap);
    va_end(ap);
    if (printed < 0) {
      return -1;
    }
  }

  str->size += (size_t)printed;
  return printed;
}

String arena_printf(Arena *arena, const char *fmt, ...)
  FUNC_ATTR_PRINTF(2, 3)
{
  size_t remaining = 0;
  char *buf = NULL;
  if (arena) {
    if (!arena->cur_blk) {
      arena_alloc_block(arena);
    }

    // happy case, we can fit the printed string in the rest of the current
    // block (one pass):
    remaining = arena->size - arena->pos;
    buf = arena->cur_blk + arena->pos;
  }

  va_list ap;
  va_start(ap, fmt);
  int printed = vsnprintf(buf, remaining, fmt, ap);
  va_end(ap);

  if (printed < 0) {
    return (String)STRING_INIT;
  }

  // printed string didn't fit, allocate and try again
  if ((size_t)printed >= remaining) {
    buf = arena_alloc(arena, (size_t)printed + 1, false);
    va_start(ap, fmt);
    printed = vsnprintf(buf, (size_t)printed + 1, fmt, ap);
    va_end(ap);
    if (printed < 0) {
      return (String)STRING_INIT;
    }
  } else {
    arena->pos += (size_t)printed + 1;
  }

  return cbuf_as_string(buf, (size_t)printed);
}
