#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "nvim/eval/typval_defs.h"
#include "nvim/option_vars.h"
#include "nvim/strings.h"  // IWYU pragma: keep

/// Flags for vim_str2nr()
typedef enum {
  STR2NR_DEC = 0,
  STR2NR_BIN = (1 << 0),  ///< Allow binary numbers.
  STR2NR_OCT = (1 << 1),  ///< Allow octal numbers.
  STR2NR_HEX = (1 << 2),  ///< Allow hexadecimal numbers.
  STR2NR_OOCT = (1 << 3),  ///< Octal with prefix "0o": 0o777
  /// Force one of the above variants.
  ///
  /// STR2NR_FORCE|STR2NR_DEC is actually not different from supplying zero
  /// as flags, but still present for completeness.
  ///
  /// STR2NR_FORCE|STR2NR_OCT|STR2NR_OOCT is the same as STR2NR_FORCE|STR2NR_OCT
  /// or STR2NR_FORCE|STR2NR_OOCT.
  STR2NR_FORCE = (1 << 7),
  /// Recognize all formats vim_str2nr() can recognize.
  STR2NR_ALL = STR2NR_BIN | STR2NR_OCT | STR2NR_HEX | STR2NR_OOCT,
  /// Disallow octals numbers without the 0o prefix.
  STR2NR_NO_OCT = STR2NR_BIN | STR2NR_HEX | STR2NR_OOCT,
  STR2NR_QUOTE = (1 << 4),  ///< Ignore embedded single quotes.
} ChStr2NrFlags;

// Character classification table - exposed for Rust FFI
// Flags: CT_CELL_MASK (0x07), CT_PRINT_CHAR (0x10), CT_ID_CHAR (0x20), CT_FNAME_CHAR (0x40)
extern uint8_t g_chartab[256];

// Functions exported directly from Rust (no C wrapper needed)
extern char *skipwhite(const char *p);
extern char *skipwhite_len(const char *p, size_t len);
extern char *skipdigits(const char *q);
extern const char *skipbin(const char *q);
extern char *skiphex(char *q);
extern char *skiptodigit(char *q);
extern const char *skiptobin(const char *q);
extern char *skiptohex(char *q);
extern char *skiptowhite(const char *p);
extern char *skiptowhite_esc(const char *p);
extern intptr_t getwhitecols(const char *p);
extern intptr_t getwhitecols_curline(void);
extern char *skip_to_newline(const char *p);
extern void trans_characters(char *buf, int bufsize);
extern size_t transstr_len(const char *s, bool untab);
extern size_t transstr_buf(const char *s, ssize_t slen, char *buf, size_t buflen, bool untab);
extern size_t transchar_hex(char *buf, int c);
extern void rl_mirror_ascii(char *str, char *end);
extern int hex2nr(int c);
extern int hexhex2nr(const char *p);
extern int byte2cells(int b);
extern int char2cells(int c);
extern int ptr2cells(const char *p);
extern int vim_strsize(const char *s);
extern int vim_strnsize(const char *s, int len);
extern bool vim_isblankline(char *lbuf);
extern bool rem_backslash(const char *str);
extern void backslash_halve(char *p);
extern char *backslash_halve_save(const char *p);
extern void vim_str2nr(const char *start, int *prep, int *len, int what,
                       varnumber_T *nptr, uvarnumber_T *unptr, int maxlen,
                       bool strict, bool *overflow);
extern bool vim_isIDc(int c);
extern bool vim_iswordc(int c);
extern bool vim_iswordc_tab(int c, const uint64_t *chartab);
extern bool vim_iswordc_buf(int c, buf_T *buf);
extern bool vim_iswordp(const char *p);
extern bool vim_iswordp_buf(const char *p, buf_T *buf);
extern bool vim_isfilec(int c);
extern bool vim_is_fname_char(int c);
extern bool vim_isfilec_or_wc(int c);
extern bool vim_isprintc(int c);

#include "charset.h.generated.h"
#include "charset.h.inline.generated.h"

/// Check if `c` is one of the characters in 'breakat'.
/// Used very often if 'linebreak' is set
static inline bool vim_isbreak(int c)
  FUNC_ATTR_CONST FUNC_ATTR_ALWAYS_INLINE
{
  return breakat_flags[(uint8_t)c];
}
