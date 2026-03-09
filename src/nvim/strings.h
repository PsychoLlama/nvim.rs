#pragma once

#include <stdarg.h>  // IWYU pragma: keep
#include <string.h>

#include "auto/config.h"
#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/os/os_defs.h"
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Return the length of a string literal
#define STRLEN_LITERAL(s) (sizeof(s) - 1)

/// Store a key/value pair
typedef struct {
  int key;        ///< the key
  char *value;    ///< the value string
  size_t length;  ///< length of the value string
} keyvalue_T;

#define KEYVALUE_ENTRY(k, v) { (k), (v), STRLEN_LITERAL(v) }

#include "strings.h.generated.h"
#include "strings.h.inline.generated.h"

// Rust-implemented string functions (exported via #[export_name])
char *xstrnsave(const char *string, size_t len)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
char *vim_strsave_escaped(const char *string, const char *esc_chars)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
char *vim_strsave_escaped_ext(const char *string, const char *esc_chars, char cc, bool bsl)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
char *vim_strnsave_unquoted(const char *string, size_t length)
  FUNC_ATTR_MALLOC FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET;
char *vim_strsave_up(const char *string)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
char *vim_strnsave_up(const char *string, size_t len)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
void vim_strup(char *p) FUNC_ATTR_NONNULL_ALL;
void vim_strcpy_up(char *restrict dst, const char *restrict src) FUNC_ATTR_NONNULL_ALL;
void vim_strncpy_up(char *restrict dst, const char *restrict src, size_t n) FUNC_ATTR_NONNULL_ALL;
void vim_memcpy_up(char *restrict dst, const char *restrict src, size_t n) FUNC_ATTR_NONNULL_ALL;
char *strcase_save(const char *orig, bool upper)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
void del_trailing_spaces(char *ptr) FUNC_ATTR_NONNULL_ALL;
bool striequal(const char *a, const char *b) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT;
int vim_strnicmp_asc(const char *s1, const char *s2, size_t len)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT;
char *vim_strchr(const char *string, int c)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT;
void sort_strings(char **files, int count);
bool has_non_ascii(const char *s) FUNC_ATTR_PURE;
bool has_non_ascii_len(const char *s, size_t len) FUNC_ATTR_PURE;
char *concat_str(const char *restrict str1, const char *restrict str2)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;
char *reverse_text(char *s) FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET;
char *strrep(const char *src, const char *what, const char *rep);
char *vim_strsave_shellescape(const char *string, bool do_special, bool do_newline)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL;

// Rust-implemented VimL string functions (f_* exported via #[export_name])
void f_byteidx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_byteidxcomp(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_charidx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_str2list(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_str2nr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strgetchar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_stridx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_string(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strlen(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strcharlen(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strchars(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strutf16len(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strdisplaywidth(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strwidth(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strcharpart(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strpart(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strridx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_strtrans(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_utf16idx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_tolower(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_toupper(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_tr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_trim(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

/// Append string to string and return pointer to the next byte
///
/// Unlike strcat, this one does *not* add NUL byte and returns pointer to the
/// past of the added string.
///
/// @param[out]  dst  String to append to.
/// @param[in]  src  String to append.
///
/// @return pointer to the byte just past the appended byte.
static inline char *strappend(char *const dst, const char *const src)
  FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_NONNULL_ALL
    FUNC_ATTR_NONNULL_RET FUNC_ATTR_WARN_UNUSED_RESULT
{
  const size_t src_len = strlen(src);
  return (char *)memmove(dst, src, src_len) + src_len;
}

#ifdef HAVE_STRCASECMP
# define STRICMP(d, s)      strcasecmp((char *)(d), (char *)(s))
#else
# ifdef HAVE_STRICMP
#  define STRICMP(d, s)     stricmp((char *)(d), (char *)(s))
# else
#  define STRICMP(d, s)     vim_stricmp((char *)(d), (char *)(s))
# endif
#endif

#ifdef HAVE_STRNCASECMP
# define STRNICMP(d, s, n)  strncasecmp((char *)(d), (char *)(s), (size_t)(n))
#else
# ifdef HAVE_STRNICMP
#  define STRNICMP(d, s, n) strnicmp((char *)(d), (char *)(s), (size_t)(n))
# else
#  define STRNICMP(d, s, n) vim_strnicmp((char *)(d), (char *)(s), (size_t)(n))
# endif
#endif

#define kv_printf(v, ...) kv_do_printf(&(v), __VA_ARGS__)
