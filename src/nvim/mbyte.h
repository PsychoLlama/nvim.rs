#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <sys/types.h>  // IWYU pragma: keep
#include <utf8proc.h>
#include <uv.h>  // IWYU pragma: keep

#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/mbyte_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

#define GRAPHEME_STATE_INIT 0

#include "mbyte.h.generated.h"
#include "mbyte.h.inline.generated.h"

enum {
  kInvalidByteCells = 4,
};

// Return byte length of character that starts with byte "b".
// Returns 1 for a single-byte character.
// MB_BYTE2LEN_CHECK() can be used to count a special key as one byte.
// Don't call MB_BYTE2LEN(b) with b < 0 or b > 255!
#define MB_BYTE2LEN(b)         utf8len_tab[b]
#define MB_BYTE2LEN_CHECK(b)   (((b) < 0 || (b) > 255) ? 1 : utf8len_tab[b])

extern const uint8_t utf8len_tab_zero[256];

extern const uint8_t utf8len_tab[256];

// Use our own character-case definitions, because the current locale may
// differ from what the .spl file uses.
// These must not be called with negative number!
// Multi-byte implementation.  For Unicode we can call utf_*(), but don't do
// that for ASCII, because we don't want to use 'casemap' here.  Otherwise use
// the "w" library function for characters above 255.
#define SPELL_TOFOLD(c) ((c) >= 128 ? utf_fold(c) : (int)spelltab.st_fold[c])

#define SPELL_TOUPPER(c) ((c) >= 128 ? mb_toupper(c) : (int)spelltab.st_upper[c])

#define SPELL_ISUPPER(c) ((c) >= 128 ? mb_isupper(c) : spelltab.st_isu[c])

// MB_PTR_ADV(): advance a pointer to the next character, taking care of
// multi-byte characters if needed. Skip over composing chars.
#define MB_PTR_ADV(p)      (p += utfc_ptr2len((char *)p))

// MB_PTR_BACK(): backup a pointer to the previous character, taking care of
// multi-byte characters if needed. Only use with "p" > "s" !
#define MB_PTR_BACK(s, p) \
  (p -= utf_head_off((char *)(s), (char *)(p) - 1) + 1)

// Declarations for functions implemented in Rust (src/nvim-rs/mbyte/).
// These were previously thin C wrappers; the Rust implementations are now
// exported directly under the canonical C names via #[unsafe(export_name)].
extern int32_t utf_ptr2CharInfo_impl(const uint8_t *p, uintptr_t len);
extern int utf_char2len(int c);
extern int mb_char2len(int c);
extern int utf_char2bytes(int c, char *buf);
extern int utf_byte2len(int b);
extern int utf_ptr2char(const char *p);
extern int utf_ptr2len(const char *p);
extern int utf_ptr2len_len(const char *p, int size);
extern bool utf_valid_string(const char *s, const char *end);
extern bool utf_eat_space(int cc);
extern bool utf_allow_break_before(int cc);
extern bool utf_allow_break_after(int cc);
extern bool utf_allow_break(int cc, int ncc);
extern int mb_charlen(const char *str);
extern int mb_charlen_len(const char *str, int len);
extern size_t mb_string2cells(const char *str);
extern size_t mb_string2cells_len(const char *str, size_t size);
extern bool utf_printable(int c);
extern bool utf_iscomposing_legacy(int c);
extern bool utf_iscomposing_first(int c);
extern int utf_fold(int a);
extern int utf_strnicmp(const char *s1, const char *s2, size_t n1, size_t n2);
extern int mb_strnicmp(const char *s1, const char *s2, size_t nn);
extern int mb_stricmp(const char *s1, const char *s2);
extern int mb_strcmp_ic(bool ic, const char *s1, const char *s2);
extern bool utf_ambiguous_width(const char *p);
extern int cw_value(int c);
extern int utf_char2cells(int c);
extern int utf_ptr2cells(const char *p);
extern int utf_ptr2cells_len(const char *p, int size);
extern CharBoundsOff utf_cp_bounds_len(const char *base, const char *p_in, int p_len);
extern CharBoundsOff utf_cp_bounds(const char *base, const char *p_in);
extern void remove_bom(char *s);
extern int utf_class_tab(int c, const uint64_t *chartab);
extern int utf_class(int c);
extern int mb_get_class_tab(const char *p, const uint64_t *chartab);
extern int mb_get_class(const char *p);
extern void mb_utflen(const char *s, size_t len, size_t *codepoints, size_t *codeunits);
extern ssize_t mb_utf_index_to_bytes(const char *s, size_t len, size_t index,
                                     bool use_utf16_units);
extern int mb_cptr2char_adv(const char **pp);
extern char *enc_skip(char *p);
extern bool utf_composinglike(const char *p1, const char *p2, int *state);
extern bool utf_iscomposing(int c1, int c2, int *state);
extern int utfc_ptr2len(const char *p);
extern int utfc_ptr2len_len(const char *p, int size);
extern int utf_head_off(const char *base, const char *p);
extern int bomb_size(void);
extern int enc_canon_props(const char *name);
extern int mb_off_next(const char *base, const char *p);
extern int mb_ptr2char_adv(const char **pp);
extern void mb_copy_char(const char **fp, char **tp);
extern char *mb_prevptr(char *line, char *p);
extern const char *mb_unescape(const char **pp);
extern StrCharInfo utfc_next_impl(StrCharInfo cur);
extern schar_T utfc_ptr2schar(const char *p, int *firstc);
extern schar_T utfc_ptrlen2schar(const char *p, int len, int *firstc);

extern int rs_utf_is_trail_byte(int byte);
extern char *enc_canonize(char *enc);
extern void *my_iconv_open(char *to, char *from);
extern char *enc_locale(void);
extern int convert_setup(vimconv_T *vcp, char *from, char *to);
extern int convert_setup_ext(vimconv_T *vcp, char *from, bool from_unicode_is_utf8, char *to, bool to_unicode_is_utf8);
extern char *string_convert(const vimconv_T *vcp, char *ptr, size_t *lenp);
extern char *string_convert_ext(const vimconv_T *vcp, char *ptr, size_t *lenp, size_t *unconvlenp);
extern char *get_encoding_name(expand_T *xp, int idx);
extern int mb_toupper(int a);
extern int mb_tolower(int a);
extern bool mb_islower(int a);
extern bool mb_isupper(int a);
extern bool mb_isalpha(int a);

/// Check whether a given UTF-8 byte is a trailing byte (10xx.xxxx).

static inline bool utf_is_trail_byte(uint8_t const byte)
  FUNC_ATTR_CONST FUNC_ATTR_ALWAYS_INLINE
{
  return rs_utf_is_trail_byte(byte) != 0;
}

/// Convert a UTF-8 byte sequence to a Unicode code point.
/// Handles ascii, multibyte sequiences and illegal sequences.
///
/// @param[in]  p_in  String to convert.
///
/// @return information abouth the character. When the sequence is illegal,
/// "value" is negative, "len" is 1.
static inline CharInfo utf_ptr2CharInfo(char const *const p_in)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALWAYS_INLINE
{
  uint8_t const *const p = (uint8_t const *)p_in;
  uint8_t const first = *p;
  if (first < 0x80) {
    return (CharInfo){ .value = first, .len = 1 };
  } else {
    int len = utf8len_tab[first];
    int32_t const code_point = utf_ptr2CharInfo_impl(p, (uintptr_t)len);
    if (code_point < 0) {
      len = 1;
    }
    return (CharInfo){ .value = code_point, .len = len };
  }
}

/// Return information about the next character.
/// Composing and combining characters are considered a part of the current character.
///
/// @param[in] cur  Information about the current character in the string.
static inline StrCharInfo utfc_next(StrCharInfo cur)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_PURE
{
  // handle ASCII case inline
  uint8_t *next = (uint8_t *)(cur.ptr + cur.chr.len);
  if (EXPECT(*next < 0x80U, true)) {
    return (StrCharInfo){
      .ptr = (char *)next,
      .chr = (CharInfo){ .value = *next, .len = 1 },
    };
  }

  return utfc_next_impl(cur);
}

static inline StrCharInfo utf_ptr2StrCharInfo(char *ptr)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_PURE
{
  return (StrCharInfo){ .ptr = ptr, .chr = utf_ptr2CharInfo(ptr) };
}
