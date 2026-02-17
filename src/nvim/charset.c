/// @file charset.c
///
/// Code related to character sets.

#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/globals.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "charset.c.generated.h"

extern int rs_get_fileformat(buf_T *buf);

// Rust functions that keep their rs_ prefix (have C wrapper logic)
extern void rs_transchar_nonprint(char *charbuf, int c, bool use_uhex, int fileformat);
extern void rs_transchar_buf(char *buf, int c, bool chartab_initialized, bool use_uhex, int fileformat);
extern void rs_transchar_byte_buf(char *buf, int c, bool chartab_initialized, bool use_uhex, int fileformat);
extern char *rs_str_foldcase(const char *str, int orglen, char *buf, int buflen,
                             int (*tolower_fn)(int));
extern int rs_parse_isopt(const char *var, buf_T *buf, bool only_check);
extern int rs_check_isopt(const char *var);
extern int rs_buf_init_chartab(buf_T *buf, bool global);
extern char *rs_transstr(const char *s, bool untab);
extern int rs_try_getdigits(char **pp, intptr_t *nr);
extern intptr_t rs_getdigits(char **pp, int strict, intptr_t def);
extern int rs_getdigits_int(char **pp, int strict, int def);
extern long rs_getdigits_long(char **pp, int strict, long def);
extern int32_t rs_getdigits_int32(char **pp, int strict, int32_t def);

static bool chartab_initialized = false;

// ============================================================================
// Accessor functions for Rust FFI
// ============================================================================

/// Get 'isident' option value.
const char *nvim_charset_get_p_isi(void)
{
  return p_isi;
}

/// Get 'isprint' option value.
const char *nvim_charset_get_p_isp(void)
{
  return p_isp;
}

/// Get 'isfname' option value.
const char *nvim_charset_get_p_isf(void)
{
  return p_isf;
}

/// Get 'iskeyword' for a buffer.
const char *nvim_charset_get_buf_p_isk(buf_T *buf)
{
  return buf ? buf->b_p_isk : NULL;
}

/// Get dy_flags (display flags).
unsigned nvim_charset_get_dy_flags(void)
{
  return dy_flags;
}

/// Get b_p_lisp flag from a buffer.
int nvim_charset_get_buf_lisp(buf_T *buf)
{
  return buf ? buf->b_p_lisp : 0;
}

/// Advance pointer and get UTF character (skips composing characters).
int nvim_charset_mb_ptr2char_adv(const char **pp)
{
  return mb_ptr2char_adv(pp);
}

/// Check if character is lowercase.
int nvim_charset_mb_islower(int c)
{
  return mb_islower(c) ? 1 : 0;
}

/// Check if character is uppercase.
int nvim_charset_mb_isupper(int c)
{
  return mb_isupper(c) ? 1 : 0;
}

/// Get pointer to g_chartab array.
uint8_t *nvim_charset_get_g_chartab(void)
{
  return g_chartab;
}

/// Get pointer to the cursor line.
const char *nvim_charset_get_cursor_line_ptr(void)
{
  return get_cursor_line_ptr();
}

/// Check if chartab is initialized.
int nvim_charset_is_initialized(void)
{
  return chartab_initialized ? 1 : 0;
}

/// Get pointer comparison values for option string detection.
/// These are used in parse_isopt to determine which option is being set.
const char *nvim_charset_get_p_isi_ptr(void)
{
  return p_isi;
}

const char *nvim_charset_get_p_isp_ptr(void)
{
  return p_isp;
}

const char *nvim_charset_get_p_isf_ptr(void)
{
  return p_isf;
}

// ============================================================================
// ============================================================================
// g_chartab - Character properties table
// ============================================================================
//
// g_chartab is owned by Rust and contains properties for bytes 0-255:
// - Lower 3 bits (0x07): display cell count (1, 2, or 4)
// - Bit 4 (0x10): CT_PRINT_CHAR - printable character
// - Bit 5 (0x20): CT_ID_CHAR - identifier character
// - Bit 6 (0x40): CT_FNAME_CHAR - filename character
//
// The chartab manipulation macros and flags are now in Rust.
extern uint8_t g_chartab[256];

/// Fill g_chartab[].  Also fills curbuf->b_chartab[] with flags for keyword
/// characters for current buffer.
///
/// Depends on the option settings 'iskeyword', 'isident', 'isfname',
/// 'isprint' and 'encoding'.
///
/// The index in g_chartab[] is the character when first byte is up to 0x80,
/// if the first byte is 0x80 and above it depends on further bytes.
///
/// The contents of g_chartab[]:
/// - The lower two bits, masked by CT_CELL_MASK, give the number of display
///   cells the character occupies (1 or 2).  Not valid for UTF-8 above 0x80.
/// - CT_PRINT_CHAR bit is set when the character is printable (no need to
///   translate the character before displaying it).  Note that only DBCS
///   characters can have 2 display cells and still be printable.
/// - CT_FNAME_CHAR bit is set when the character can be in a file name.
/// - CT_ID_CHAR bit is set when the character can be in an identifier.
///
/// @return FAIL if 'iskeyword', 'isident', 'isfname' or 'isprint' option has
/// an error, OK otherwise.
int init_chartab(void)
{
  return buf_init_chartab(curbuf, true);
}

/// Helper for init_chartab
///
/// @param global false: only set buf->b_chartab[]
///
/// @return FAIL if 'iskeyword', 'isident', 'isfname' or 'isprint' option has
/// an error, OK otherwise.
int buf_init_chartab(buf_T *buf, bool global)
{
  int result = rs_buf_init_chartab(buf, global);
  if (result == 0) {
    chartab_initialized = true;
    return OK;
  }
  return FAIL;
}

/// Checks the format for the option settings 'iskeyword', 'isident', 'isfname'
/// or 'isprint'.
/// Returns FAIL if has an error, OK otherwise.
int check_isopt(char *var)
{
  return rs_check_isopt(var) == 0 ? OK : FAIL;
}

/// Copy string and replace special characters with printable characters
///
/// Works like `strtrans()` does, used for that and in some other places.
///
/// @param[in]  s  String to replace characters from.
///
/// @return [allocated] translated string
char *transstr(const char *const s, bool untab)
  FUNC_ATTR_NONNULL_RET
{
  // Compute the length of the result, taking account of unprintable
  // multi-byte characters.
  const size_t len = transstr_len(s, untab) + 1;
  char *const buf = xmalloc(len);
  transstr_buf(s, -1, buf, len, untab);
  return buf;
}

size_t kv_transstr(StringBuilder *str, const char *const s, bool untab)
  FUNC_ATTR_NONNULL_ARG(1)
{
  if (!s) {
    return 0;
  }

  // Compute the length of the result, taking account of unprintable
  // multi-byte characters.
  const size_t len = transstr_len(s, untab);
  kv_ensure_space(*str, len + 1);
  transstr_buf(s, -1, str->items + str->size, len + 1, untab);
  str->size += len;  // do not include NUL byte
  return len;
}

/// Convert the string "str[orglen]" to do ignore-case comparing.
/// Use the current locale.
///
/// When "buf" is NULL, return an allocated string.
/// Otherwise, put the result in buf, limited by buflen, and return buf.
char *str_foldcase(char *str, int orglen, char *buf, int buflen)
  FUNC_ATTR_NONNULL_RET
{
  return rs_str_foldcase(str, orglen, buf, buflen, mb_tolower);
}

// Catch 22: g_chartab[] can't be initialized before the options are
// initialized, and initializing options may cause transchar() to be called!
// When chartab_initialized == false don't use g_chartab[].
// Does NOT work for multi-byte characters, c must be <= 255.
// Also doesn't work for the first byte of a multi-byte, "c" must be a
// character!
static uint8_t transchar_charbuf[11];

/// Translate a character into a printable one, leaving printable ASCII intact
///
/// All unicode characters are considered non-printable in this function.
///
/// @param[in]  c  Character to translate.
///
/// @return translated character into a static buffer.
char *transchar(int c)
{
  return transchar_buf(curbuf, c);
}

char *transchar_buf(const buf_T *buf, int c)
{
  bool use_uhex = (dy_flags & kOptDyFlagUhex) != 0;
  int fileformat = (buf != NULL) ? rs_get_fileformat((buf_T *)buf) : -1;
  rs_transchar_buf((char *)transchar_charbuf, c, chartab_initialized, use_uhex, fileformat);
  return (char *)transchar_charbuf;
}

/// Like transchar(), but called with a byte instead of a character.
///
/// Checks for an illegal UTF-8 byte.  Uses 'fileformat' of the current buffer.
///
/// @param[in]  c  Byte to translate.
///
/// @return pointer to translated character in transchar_charbuf.
char *transchar_byte(const int c)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return transchar_byte_buf(curbuf, c);
}

/// Like transchar_buf(), but called with a byte instead of a character.
///
/// Checks for an illegal UTF-8 byte.  Uses 'fileformat' of "buf", unless it is NULL.
///
/// @param[in]  c  Byte to translate.
///
/// @return pointer to translated character in transchar_charbuf.
char *transchar_byte_buf(const buf_T *buf, const int c)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  bool use_uhex = (dy_flags & kOptDyFlagUhex) != 0;
  int fileformat = (buf != NULL) ? rs_get_fileformat((buf_T *)buf) : -1;
  rs_transchar_byte_buf((char *)transchar_charbuf, c, chartab_initialized, use_uhex, fileformat);
  return (char *)transchar_charbuf;
}

/// Convert non-printable characters to 2..4 printable ones
///
/// @warning Does not work for multi-byte characters, c must be <= 255.
///
/// @param[in]  buf  Required to check the file format
/// @param[out]  charbuf  Buffer to store result in, must be able to hold
///                       at least 5 bytes (conversion result + NUL).
/// @param[in]  c  Character to convert. NUL is assumed to be NL according to
///                `:h NL-used-for-NUL`.
void transchar_nonprint(const buf_T *buf, char *charbuf, int c)
{
  bool use_uhex = (dy_flags & kOptDyFlagUhex) != 0;
  int fileformat = (buf != NULL) ? rs_get_fileformat((buf_T *)buf) : -1;
  rs_transchar_nonprint(charbuf, c, use_uhex, fileformat);
}

/// Gets a number from a string and skips over it, signalling overflow.
///
/// @param[out]  pp  A pointer to a pointer to char.
///                  It will be advanced past the read number.
/// @param[out]  nr  Number read from the string.
///
/// @return true on success, false on error/overflow
bool try_getdigits(char **pp, intmax_t *nr)
{
  return rs_try_getdigits(pp, (intptr_t *)nr) != 0;
}

/// Gets a number from a string and skips over it.
///
/// @param[out]  pp  Pointer to a pointer to char.
///                  It will be advanced past the read number.
/// @param strict    Abort on overflow.
/// @param def       Default value, if parsing fails or overflow occurs.
///
/// @return Number read from the string, or `def` on parse failure or overflow.
intmax_t getdigits(char **pp, bool strict, intmax_t def)
{
  return rs_getdigits(pp, strict ? 1 : 0, def);
}

/// Gets an int number from a string.
///
/// @see getdigits
int getdigits_int(char **pp, bool strict, int def)
{
  return rs_getdigits_int(pp, strict ? 1 : 0, def);
}

/// Gets a long number from a string.
///
/// @see getdigits
long getdigits_long(char **pp, bool strict, long def)
{
  return rs_getdigits_long(pp, strict ? 1 : 0, def);
}

/// Gets a int32_t number from a string.
///
/// @see getdigits
int32_t getdigits_int32(char **pp, bool strict, int32_t def)
{
  return rs_getdigits_int32(pp, strict ? 1 : 0, def);
}

