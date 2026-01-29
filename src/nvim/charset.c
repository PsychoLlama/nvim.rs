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

extern const char *rs_skipwhite(const char *p);
extern const char *rs_skipwhite_len(const char *p, size_t len);
extern const char *rs_skipdigits(const char *q);
extern const char *rs_skipbin(const char *q);
extern const char *rs_skiphex(const char *q);
extern const char *rs_skiptodigit(const char *q);
extern const char *rs_skiptobin(const char *q);
extern const char *rs_skiptohex(const char *q);
extern const char *rs_skiptowhite(const char *p);
extern const char *rs_skiptowhite_esc(const char *p);
extern intptr_t rs_getwhitecols(const char *p);
extern int rs_hex2nr(int c);
extern int rs_hexhex2nr(const char *p);
extern unsigned rs_nr2hex(unsigned n);
extern char *rs_skip_to_newline(const char *p);
extern bool rs_vim_isblankline(const char *lbuf);
extern size_t rs_transchar_hex(char *buf, int c);
extern int rs_vim_isfilec(int c);
extern int rs_vim_is_fname_char(int c);
extern int rs_byte2cells(int b);
extern int rs_vim_isIDc(int c);
extern int rs_vim_isprintc(int c);
extern int rs_ptr2cells(const char *p);
extern int rs_vim_strsize(const char *s);
extern int rs_vim_strnsize(const char *s, int len);
extern void rs_rl_mirror_ascii(char *str, const char *end);
extern bool rs_rem_backslash(const char *str);
extern void rs_backslash_halve(char *p);
extern int rs_vim_iswordc_tab(int c, const uint64_t *chartab);
extern int rs_vim_iswordc(int c);
extern int rs_vim_iswordc_buf(int c, buf_T *buf);
extern int rs_vim_iswordp(const char *p);
extern int rs_vim_iswordp_buf(const char *p, buf_T *buf);
extern size_t rs_transstr_len(const char *s, bool untab);
extern int rs_try_getdigits(char **pp, intptr_t *nr);
extern intptr_t rs_getdigits(char **pp, int strict, intptr_t def);
extern int rs_getdigits_int(char **pp, int strict, int def);
extern long rs_getdigits_long(char **pp, int strict, long def);
extern int32_t rs_getdigits_int32(char **pp, int strict, int32_t def);
extern char *rs_backslash_halve_save(const char *p);
extern int rs_vim_isfilec_or_wc(int c);
extern void rs_vim_str2nr(const char *start, int *prep, int *len, int what,
                          varnumber_T *nptr, uvarnumber_T *unptr, int maxlen,
                          bool strict, bool *overflow);
extern void rs_transchar_nonprint(char *charbuf, int c, bool use_uhex, int fileformat);
extern char *rs_str_foldcase(const char *str, int orglen, char *buf, int buflen,
                             int (*tolower_fn)(int));
extern int rs_parse_isopt(const char *var, buf_T *buf, bool only_check);
extern int rs_check_isopt(const char *var);
extern int rs_buf_init_chartab(buf_T *buf, bool global);
extern void rs_trans_characters(char *buf, int bufsize);
extern size_t rs_transstr_buf(const char *s, ssize_t slen, char *buf, size_t buflen, bool untab);
extern char *rs_transstr(const char *s, bool untab);

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
// Chartab macros and definitions
// ============================================================================

// b_chartab[] is an array with 256 bits, each bit representing one of the
// characters 0-255.
#define SET_CHARTAB(buf, c) \
  (buf)->b_chartab[(unsigned)(c) >> 6] |= (1ull << ((c) & 0x3f))
#define RESET_CHARTAB(buf, c) \
  (buf)->b_chartab[(unsigned)(c) >> 6] &= ~(1ull << ((c) & 0x3f))
#define GET_CHARTAB_TAB(chartab, c) \
  ((chartab)[(unsigned)(c) >> 6] & (1ull << ((c) & 0x3f)))

// Table used below, see init_chartab() for an explanation
// Not static - exposed for Rust FFI access
uint8_t g_chartab[256];

// Flags for g_chartab[].
#define CT_CELL_MASK  0x07  ///< mask: nr of display cells (1, 2 or 4)
#define CT_PRINT_CHAR 0x10  ///< flag: set for printable chars
#define CT_ID_CHAR    0x20  ///< flag: set for ID chars
#define CT_FNAME_CHAR 0x40  ///< flag: set for file name chars

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

/// Translate any special characters in buf[bufsize] in-place.
///
/// The result is a string with only printable characters, but if there is not
/// enough room, not all characters will be translated.
///
/// @param buf
/// @param bufsize
void trans_characters(char *buf, int bufsize)
{
  rs_trans_characters(buf, bufsize);
}

/// Find length of a string capable of holding s with all specials replaced
///
/// Assumes replacing special characters with printable ones just like
/// strtrans() does.
///
/// @param[in]  s  String to check.
///
/// @return number of bytes needed to hold a translation of `s`, NUL byte not
///         included.
size_t transstr_len(const char *const s, bool untab)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return rs_transstr_len(s, untab);
}

/// Replace special characters with printable ones
///
/// @param[in]  s  String to replace characters from.
/// @param[out]  buf  Buffer to which result should be saved.
/// @param[in]  len  Buffer length. Resulting string may not occupy more then
///                  len - 1 bytes (one for trailing NUL byte).
/// @param[in]  untab  remove tab characters
///
/// @return length of the resulting string, without the NUL byte.
size_t transstr_buf(const char *const s, const ssize_t slen, char *const buf, const size_t buflen,
                    bool untab)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_transstr_buf(s, slen, buf, buflen, untab);
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
  int i = 0;
  if (IS_SPECIAL(c)) {
    // special key code, display as ~@ char
    transchar_charbuf[0] = '~';
    transchar_charbuf[1] = '@';
    i = 2;
    c = K_SECOND(c);
  }

  if ((!chartab_initialized && (c >= ' ' && c <= '~'))
      || ((c <= 0xFF) && vim_isprintc(c))) {
    // printable character
    transchar_charbuf[i] = (uint8_t)c;
    transchar_charbuf[i + 1] = NUL;
  } else if (c <= 0xFF) {
    transchar_nonprint(buf, (char *)transchar_charbuf + i, c);
  } else {
    transchar_hex((char *)transchar_charbuf + i, c);
  }
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
  if (c >= 0x80) {
    transchar_nonprint(buf, (char *)transchar_charbuf, c);
    return (char *)transchar_charbuf;
  }
  return transchar_buf(buf, c);
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
  int fileformat = (buf != NULL) ? get_fileformat(buf) : -1;
  rs_transchar_nonprint(charbuf, c, use_uhex, fileformat);
}

/// Convert a non-printable character to hex C string like "<FFFF>"
///
/// @param[out]  buf  Buffer to store result in.
/// @param[in]  c  Character to convert.
///
/// @return Number of bytes stored in buffer, excluding trailing NUL byte.
size_t transchar_hex(char *const buf, const int c)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_transchar_hex(buf, c);
}

/// Mirror text "str" for right-left displaying.
/// Only works for single-byte characters (e.g., numbers).
void rl_mirror_ascii(char *str, char *end)
{
  rs_rl_mirror_ascii(str, end);
}

/// Convert the lower 4 bits of byte "c" to its hex character
///
/// Lower case letters are used to avoid the confusion of <F1> being 0xf1 or
/// function key 1.
///
/// @param[in]  n  Number to convert.
///
/// @return the hex character.
static inline unsigned nr2hex(unsigned n)
  FUNC_ATTR_CONST FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_nr2hex(n);
}

/// Return number of display cells occupied by byte "b".
///
/// Caller must make sure 0 <= b <= 255.
/// For multi-byte mode "b" must be the first byte of a character.
/// A TAB is counted as two cells: "^I".
/// This will return 0 for bytes >= 0x80, because the number of
/// cells depends on further bytes in UTF-8.
///
/// @param b
///
/// @return Number of display cells.
int byte2cells(int b)
  FUNC_ATTR_PURE
{
  return rs_byte2cells(b);
}

extern int rs_char2cells(int c);

/// Return number of display cells occupied by character "c".
///
/// "c" can be a special key (negative number) in which case 3 or 4 is returned.
/// A TAB is counted as two cells: "^I" or four: "<09>".
///
/// @param c
///
/// @return Number of display cells.
int char2cells(int c)
{
  return rs_char2cells(c);
}

/// Return number of display cells occupied by character at "*p".
/// A TAB is counted as two cells: "^I" or four: "<09>".
///
/// @param p
///
/// @return number of display cells.
int ptr2cells(const char *p_in)
{
  return rs_ptr2cells(p_in);
}

/// Return the number of character cells string "s" will take on the screen,
/// counting TABs as two characters: "^I".
///
/// 's' must be non-null.
///
/// @param s
///
/// @return number of character cells.
int vim_strsize(const char *s)
{
  return rs_vim_strsize(s);
}

/// Return the number of character cells string "s[len]" will take on the
/// screen, counting TABs as two characters: "^I".
///
/// 's' must be non-null.
///
/// @param s
/// @param len
///
/// @return Number of character cells.
int vim_strnsize(const char *s, int len)
{
  return rs_vim_strnsize(s, len);
}

/// Check that "c" is a normal identifier character:
/// Letters and characters from the 'isident' option.
///
/// @param  c  character to check
bool vim_isIDc(int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_vim_isIDc(c) != 0;
}

/// Check that "c" is a keyword character:
/// Letters and characters from 'iskeyword' option for the current buffer.
/// For multi-byte characters mb_get_class() is used (builtin rules).
///
/// @param  c  character to check
bool vim_iswordc(const int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_vim_iswordc(c) != 0;
}

/// Check that "c" is a keyword character
/// Letters and characters from 'iskeyword' option for given buffer.
/// For multi-byte characters mb_get_class() is used (builtin rules).
///
/// @param[in]  c  Character to check.
/// @param[in]  chartab  Buffer chartab.
bool vim_iswordc_tab(const int c, const uint64_t *const chartab)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_vim_iswordc_tab(c, chartab) != 0;
}

/// Check that "c" is a keyword character:
/// Letters and characters from 'iskeyword' option for given buffer.
/// For multi-byte characters mb_get_class() is used (builtin rules).
///
/// @param  c    character to check
/// @param  buf  buffer whose keywords to use
bool vim_iswordc_buf(const int c, buf_T *const buf)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(2)
{
  return rs_vim_iswordc_buf(c, buf) != 0;
}

/// Just like vim_iswordc() but uses a pointer to the (multi-byte) character.
///
/// @param  p  pointer to the multi-byte character
///
/// @return true if "p" points to a keyword character.
bool vim_iswordp(const char *const p)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_vim_iswordp(p) != 0;
}

/// Just like vim_iswordc_buf() but uses a pointer to the (multi-byte)
/// character.
///
/// @param  p    pointer to the multi-byte character
/// @param  buf  buffer whose keywords to use
///
/// @return true if "p" points to a keyword character.
bool vim_iswordp_buf(const char *const p, buf_T *const buf)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_vim_iswordp_buf(p, buf) != 0;
}

/// Check that "c" is a valid file-name character as specified with the
/// 'isfname' option.
/// Assume characters above 0x100 are valid (multi-byte).
/// To be used for commands like "gf".
///
/// @param  c  character to check
bool vim_isfilec(int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_vim_isfilec(c) != 0;
}

/// Check if "c" is a valid file-name character, including characters left
/// out of 'isfname' to make "gf" work, such as ',', ' ', '@', ':', etc.
bool vim_is_fname_char(int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_vim_is_fname_char(c) != 0;
}

/// Check that "c" is a valid file-name character or a wildcard character
/// Assume characters above 0x100 are valid (multi-byte).
/// Explicitly interpret ']' as a wildcard character as path_has_wildcard("]")
/// returns false.
///
/// @param  c  character to check
bool vim_isfilec_or_wc(int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_vim_isfilec_or_wc(c) != 0;
}

/// Check that "c" is a printable character.
///
/// @param  c  character to check
bool vim_isprintc(int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_vim_isprintc(c) != 0;
}

/// skipwhite: skip over ' ' and '\t'.
///
/// @param[in]  p  String to skip in.
///
/// @return Pointer to character after the skipped whitespace.
char *skipwhite(const char *p)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_NONNULL_RET
{
  return (char *)rs_skipwhite(p);
}

/// Like `skipwhite`, but skip up to `len` characters.
/// @see skipwhite
///
/// @param[in]  p    String to skip in.
/// @param[in]  len  Max length to skip.
///
/// @return Pointer to character after the skipped whitespace, or the `len`-th
///         character in the string.
char *skipwhite_len(const char *p, size_t len)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_NONNULL_RET
{
  return (char *)rs_skipwhite_len(p, len);
}

// getwhitecols: return the number of whitespace
// columns (bytes) at the start of a given line
intptr_t getwhitecols_curline(void)
{
  return getwhitecols(get_cursor_line_ptr());
}

intptr_t getwhitecols(const char *p)
  FUNC_ATTR_PURE
{
  return rs_getwhitecols(p);
}

/// Skip over digits
///
/// @param[in]  q  String to skip digits in.
///
/// @return Pointer to the character after the skipped digits.
char *skipdigits(const char *q)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_NONNULL_RET
{
  return (char *)rs_skipdigits(q);
}

/// skip over binary digits
///
/// @param q pointer to string
///
/// @return Pointer to the character after the skipped digits.
const char *skipbin(const char *q)
  FUNC_ATTR_PURE
  FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_NONNULL_RET
{
  return rs_skipbin(q);
}

/// skip over digits and hex characters
///
/// @param q
///
/// @return Pointer to the character after the skipped digits and hex
///         characters.
char *skiphex(char *q)
  FUNC_ATTR_PURE
{
  return (char *)rs_skiphex(q);
}

/// skip to digit (or NUL after the string)
///
/// @param q
///
/// @return Pointer to the digit or (NUL after the string).
char *skiptodigit(char *q)
  FUNC_ATTR_PURE
{
  return (char *)rs_skiptodigit(q);
}

/// skip to binary character (or NUL after the string)
///
/// @param q pointer to string
///
/// @return Pointer to the binary character or (NUL after the string).
const char *skiptobin(const char *q)
  FUNC_ATTR_PURE
  FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_NONNULL_RET
{
  return rs_skiptobin(q);
}

/// skip to hex character (or NUL after the string)
///
/// @param q
///
/// @return Pointer to the hex character or (NUL after the string).
char *skiptohex(char *q)
  FUNC_ATTR_PURE
{
  return (char *)rs_skiptohex(q);
}

/// Skip over text until ' ' or '\t' or NUL
///
/// @param[in]  p  Text to skip over.
///
/// @return Pointer to the next whitespace or NUL character.
char *skiptowhite(const char *p)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE
{
  return (char *)rs_skiptowhite(p);
}

/// skiptowhite_esc: Like skiptowhite(), but also skip escaped chars
///
/// @param p
///
/// @return Pointer to the next whitespace character.
char *skiptowhite_esc(const char *p)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE
{
  return (char *)rs_skiptowhite_esc(p);
}

/// Skip over text until '\n' or NUL.
///
/// @param[in]  p  Text to skip over.
///
/// @return Pointer to the next '\n' or NUL character.
char *skip_to_newline(const char *const p)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_NONNULL_RET
{
  return rs_skip_to_newline(p);
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

/// Check that "lbuf" is empty or only contains blanks.
///
/// @param  lbuf  line buffer to check
bool vim_isblankline(char *lbuf)
  FUNC_ATTR_PURE
{
  return rs_vim_isblankline(lbuf);
}

/// Convert a string into a long and/or unsigned long, taking care of
/// hexadecimal, octal and binary numbers.  Accepts a '-' sign.
/// If "prep" is not NULL, returns a flag to indicate the type of the number:
///   0      decimal
///   '0'    octal
///   'O'    octal
///   'o'    octal
///   'B'    bin
///   'b'    bin
///   'X'    hex
///   'x'    hex
/// If "len" is not NULL, the length of the number in characters is returned.
/// If "nptr" is not NULL, the signed result is returned in it.
/// If "unptr" is not NULL, the unsigned result is returned in it.
/// If "what" contains STR2NR_BIN recognize binary numbers.
/// If "what" contains STR2NR_OCT recognize octal numbers.
/// If "what" contains STR2NR_HEX recognize hex numbers.
/// If "what" contains STR2NR_FORCE always assume bin/oct/hex.
/// If "what" contains STR2NR_QUOTE ignore embedded single quotes
/// If maxlen > 0, check at a maximum maxlen chars.
/// If strict is true, check the number strictly. return *len = 0 if fail.
///
/// @param start
/// @param prep Returns guessed type of number 0 = decimal, 'x' or 'X' is
///             hexadecimal, '0', 'o' or 'O' is octal, 'b' or 'B' is binary.
///             When using STR2NR_FORCE is always zero.
/// @param len Returns the detected length of number.
/// @param what Recognizes what number passed, @see ChStr2NrFlags.
/// @param nptr Returns the signed result.
/// @param unptr Returns the unsigned result.
/// @param maxlen Max length of string to check.
/// @param strict If true, fail if the number has unexpected trailing
///               alphanumeric chars: *len is set to 0 and nothing else is
///               returned.
/// @param overflow When not NULL, set to true for overflow.
void vim_str2nr(const char *const start, int *const prep, int *const len, const int what,
                varnumber_T *const nptr, uvarnumber_T *const unptr, const int maxlen,
                const bool strict, bool *const overflow)
  FUNC_ATTR_NONNULL_ARG(1)
{
  rs_vim_str2nr(start, prep, len, what, nptr, unptr, maxlen, strict, overflow);
}

/// Return the value of a single hex character.
/// Only valid when the argument is '0' - '9', 'A' - 'F' or 'a' - 'f'.
///
/// @param c
///
/// @return The value of the hex character.
int hex2nr(int c)
  FUNC_ATTR_CONST
{
  return rs_hex2nr(c);
}

/// Convert two hex characters to a byte.
///
/// @return  -1 if one of the characters is not hex.
int hexhex2nr(const char *p)
  FUNC_ATTR_PURE
{
  return rs_hexhex2nr(p);
}

/// Check that "str" starts with a backslash that should be removed.
/// For Windows this is only done when the character after the
/// backslash is not a normal file name character.
/// '$' is a valid file name character, we don't remove the backslash before
/// it.  This means it is not possible to use an environment variable after a
/// backslash.  "C:\$VIM\doc" is taken literally, only "$VIM\doc" works.
/// Although "\ name" is valid, the backslash in "Program\ files" must be
/// removed.  Assume a file name doesn't start with a space.
/// For multi-byte names, never remove a backslash before a non-ascii
/// character, assume that all multi-byte characters are valid file name
/// characters.
///
/// @param  str  file path string to check
bool rem_backslash(const char *str)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_rem_backslash(str);
}

/// Halve the number of backslashes in a file name argument.
///
/// @param p
void backslash_halve(char *p)
{
  rs_backslash_halve(p);
}

/// backslash_halve() plus save the result in allocated memory.
///
/// @param p
///
/// @return String with the number of backslashes halved.
char *backslash_halve_save(const char *p)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  return rs_backslash_halve_save(p);
}
