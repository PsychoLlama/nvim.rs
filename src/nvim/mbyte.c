/// mbyte.c: Code specifically for handling multi-byte characters.
/// Multibyte extensions partly by Sung-Hoon Baek
///
/// Strings internal to Nvim are always encoded as UTF-8 (thus the legacy
/// 'encoding' option is always "utf-8").
///
/// The cell width on the display needs to be determined from the character
/// value. Recognizing UTF-8 bytes is easy: 0xxx.xxxx is a single-byte char,
/// 10xx.xxxx is a trailing byte, 11xx.xxxx is a leading byte of a multi-byte
/// character. To make things complicated, up to six composing characters
/// are allowed. These are drawn on top of the first char. For most editing
/// the sequence of bytes with composing characters included is considered to
/// be one character.
///
/// UTF-8 is used everywhere in the core. This is in registers, text
/// manipulation, buffers, etc. Nvim core communicates with external plugins
/// and GUIs in this encoding.
///
/// The encoding of a file is specified with 'fileencoding'.  Conversion
/// is to be done when it's different from "utf-8".
///
/// Vim scripts may contain an ":scriptencoding" command. This has an effect
/// for some commands, like ":menutrans".

#include <assert.h>
#include <ctype.h>
#include <errno.h>
#include <iconv.h>
#include <limits.h>
#include <locale.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <utf8proc.h>
#include <uv.h>
#include <wctype.h>

#include "auto/config.h"
#include "nvim/arabic.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/iconv_defs.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/os.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

typedef struct {
  int rangeStart;
  int rangeEnd;
  int step;
  int offset;
} convertStruct;

struct interval {
  int first;
  int last;
};

// uncrustify:off
#include "mbyte.c.generated.h"
// uncrustify:on


static const char e_list_item_nr_is_not_list[]
  = N_("E1109: List item %d is not a List");
static const char e_list_item_nr_does_not_contain_3_numbers[]
  = N_("E1110: List item %d does not contain 3 numbers");
static const char e_list_item_nr_range_invalid[]
  = N_("E1111: List item %d range invalid");
static const char e_list_item_nr_cell_width_invalid[]
  = N_("E1112: List item %d cell width invalid");
static const char e_overlapping_ranges_for_nr[]
  = N_("E1113: Overlapping ranges for 0x%lx");
static const char e_only_values_of_0x80_and_higher_supported[]
  = N_("E1114: Only values of 0x80 and higher supported");

// =============================================================================
// Rust accessor functions for buffer properties
// =============================================================================

/// Check if current buffer has 'bomb' option set (accessor for Rust).
int nvim_curbuf_get_b_p_bomb(void)
{
  return curbuf->b_p_bomb ? 1 : 0;
}

/// Check if current buffer has 'binary' option set (accessor for Rust).
int nvim_curbuf_get_b_p_bin(void)
{
  return curbuf->b_p_bin ? 1 : 0;
}

/// Get current buffer's 'fileencoding' option (accessor for Rust).
const char *nvim_curbuf_get_b_p_fenc(void)
{
  return curbuf->b_p_fenc;
}


// Get character at **pp and advance *pp to the next character.
// Note: composing characters are skipped!
int mb_ptr2char_adv(const char **const pp)
{
  int c = utf_ptr2char(*pp);
  *pp += utfc_ptr2len(*pp);
  return c;
}

/// Get the screen char at the beginning of a string
///
/// Caller is expected to check for things like unprintable chars etc
/// If first char in string is a composing char, prepend a space to display it correctly.
///
/// If "p" starts with an invalid sequence, zero is returned.
///
/// @param[out] firstc (required) The first codepoint of the screen char,
///                    or the first byte of an invalid sequence
///
/// @return the char
schar_T utfc_ptr2schar(const char *p, int *firstc)
  FUNC_ATTR_NONNULL_ALL
{
  int c = utf_ptr2char(p);
  *firstc = c;  // NOT optional, you are gonna need it
  bool first_compose = utf_iscomposing_first(c);
  size_t maxlen = MAX_SCHAR_SIZE - 1 - first_compose;
  size_t len = (size_t)utfc_ptr2len_len(p, (int)maxlen);

  if (len == 1 && (uint8_t)(*p) >= 0x80) {
    return 0;  // invalid sequence
  }

  return schar_from_buf_first(p, len, first_compose);
}

/// Get the screen char from a char with a known length
///
/// Like utfc_ptr2schar but use no more than p[maxlen].
schar_T utfc_ptrlen2schar(const char *p, int len, int *firstc)
  FUNC_ATTR_NONNULL_ALL
{
  if ((len == 1 && (uint8_t)(*p) >= 0x80) || len == 0) {
    // invalid or truncated sequence
    *firstc = (uint8_t)(*p);
    return 0;
  }

  int c = utf_ptr2char(p);
  *firstc = c;
  bool first_compose = utf_iscomposing_first(c);
  int maxlen = MAX_SCHAR_SIZE - 1 - first_compose;
  if (len > maxlen) {
    len = utfc_ptr2len_len(p, maxlen);
  }

  return schar_from_buf_first(p, (size_t)len, first_compose);
}

/// Caller must ensure there is space for `first_compose`
static schar_T schar_from_buf_first(const char *buf, size_t len, bool first_compose)
{
  if (first_compose) {
    char cbuf[MAX_SCHAR_SIZE];
    cbuf[0] = ' ';
    memcpy(cbuf + 1, buf, len);
    return schar_from_buf(cbuf, len + 1);
  } else {
    return schar_from_buf(buf, len);
  }
}



/// Accessor for Rust FFI: get UTF character from byte pointer.
int nvim_utf_ptr2char(const char *p)
{
  return utf_ptr2char(p);
}

/// Accessor for Rust FFI: get UTF character length including composing chars.
int nvim_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}


#ifdef MSWIN
# ifndef CP_UTF8
#  define CP_UTF8 65001  // magic number from winnls.h
# endif

/// Converts string from UTF-8 to UTF-16.
///
/// @param utf8  UTF-8 string.
/// @param utf8len  Length of `utf8`. May be -1 if `utf8` is NUL-terminated.
/// @param utf16[out,allocated]  NUL-terminated UTF-16 string, or NULL on error
/// @return 0 on success, or libuv error code
int utf8_to_utf16(const char *utf8, int utf8len, wchar_t **utf16)
  FUNC_ATTR_NONNULL_ALL
{
  // Compute the length needed for the converted UTF-16 string.
  int bufsize = MultiByteToWideChar(CP_UTF8,
                                    0,     // dwFlags: must be 0 for UTF-8
                                    utf8,  // -1: process up to NUL
                                    utf8len,
                                    NULL,
                                    0);    // 0: get length, don't convert
  if (bufsize == 0) {
    *utf16 = NULL;
    return uv_translate_sys_error(GetLastError());
  }

  // Allocate the destination buffer adding an extra byte for the terminating
  // NULL. If `utf8len` is not -1 MultiByteToWideChar will not add it, so
  // we do it ourselves always, just in case.
  *utf16 = xmalloc(sizeof(wchar_t) * (bufsize + 1));

  // Convert to UTF-16.
  bufsize = MultiByteToWideChar(CP_UTF8, 0, utf8, utf8len, *utf16, bufsize);
  if (bufsize == 0) {
    XFREE_CLEAR(*utf16);
    return uv_translate_sys_error(GetLastError());
  }

  (*utf16)[bufsize] = L'\0';
  return 0;
}

/// Converts string from UTF-16 to UTF-8.
///
/// @param utf16  UTF-16 string.
/// @param utf16len  Length of `utf16`. May be -1 if `utf16` is NUL-terminated.
/// @param utf8[out,allocated]  NUL-terminated UTF-8 string, or NULL on error
/// @return 0 on success, or libuv error code
int utf16_to_utf8(const wchar_t *utf16, int utf16len, char **utf8)
  FUNC_ATTR_NONNULL_ALL
{
  // Compute the space needed for the converted UTF-8 string.
  DWORD bufsize = WideCharToMultiByte(CP_UTF8,
                                      0,
                                      utf16,
                                      utf16len,
                                      NULL,
                                      0,
                                      NULL,
                                      NULL);
  if (bufsize == 0) {
    *utf8 = NULL;
    return uv_translate_sys_error(GetLastError());
  }

  // Allocate the destination buffer adding an extra byte for the terminating
  // NULL. If `utf16len` is not -1 WideCharToMultiByte will not add it, so
  // we do it ourselves always, just in case.
  *utf8 = xmalloc(bufsize + 1);

  // Convert to UTF-8.
  bufsize = WideCharToMultiByte(CP_UTF8,
                                0,
                                utf16,
                                utf16len,
                                *utf8,
                                bufsize,
                                NULL,
                                NULL);
  if (bufsize == 0) {
    XFREE_CLEAR(*utf8);
    return uv_translate_sys_error(GetLastError());
  }

  (*utf8)[bufsize] = NUL;
  return 0;
}

#endif


// "g8": show bytes of the UTF-8 char under the cursor.  Doesn't matter what
// 'encoding' has been set to.
void show_utf8(void)
{
  // Get the byte length of the char under the cursor, including composing
  // characters.
  char *line = get_cursor_pos_ptr();
  int len = utfc_ptr2len(line);
  if (len == 0) {
    msg("NUL", 0);
    return;
  }

  size_t rlen = 0;
  int clen = 0;
  for (int i = 0; i < len; i++) {
    if (clen == 0) {
      // start of (composing) character, get its length
      if (i > 0) {
        STRCPY(IObuff + rlen, "+ ");
        rlen += 2;
      }
      clen = utf_ptr2len(line + i);
    }
    assert(IOSIZE > rlen);
    snprintf(IObuff + rlen, IOSIZE - rlen, "%02x ",
             (line[i] == NL) ? NUL : (uint8_t)line[i]);  // NUL is stored as NL
    clen--;
    rlen += strlen(IObuff + rlen);
    if (rlen > IOSIZE - 20) {
      break;
    }
  }

  msg(IObuff, 0);
}

/// @return true if boundclass bc always starts a new cluster regardless of what's before
/// false negatives are allowed (perf cost, not correctness)
static bool always_break(int bc)
{
  return (bc == UTF8PROC_BOUNDCLASS_CONTROL);
}

/// @return true if bc2 always starts a cluster after bc1
/// false negatives are allowed (perf cost, not correctness)
static bool always_break_two(int bc1, int bc2)
{
  // don't check for UTF8PROC_BOUNDCLASS_CONTROL for bc2 as it either has been checked by
  // "always_break" on first iteration or when it was bc1 in the previous iteration
  return ((bc1 != UTF8PROC_BOUNDCLASS_PREPEND && bc2 == UTF8PROC_BOUNDCLASS_OTHER)
          || (bc1 >= UTF8PROC_BOUNDCLASS_CR && bc1 <= UTF8PROC_BOUNDCLASS_CONTROL)
          || (bc2 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC
              && (bc1 == UTF8PROC_BOUNDCLASS_OTHER
                  || bc1 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC)));
}

/// Assumes caller already handles ascii. see `utfc_next`
StrCharInfo utfc_next_impl(StrCharInfo cur)
{
  int32_t prev_code = cur.chr.value;
  uint8_t *next = (uint8_t *)(cur.ptr + cur.chr.len);
  GraphemeState state = GRAPHEME_STATE_INIT;
  assert(*next >= 0x80);

  while (true) {
    uint8_t const next_len = utf8len_tab[*next];
    int32_t const next_code = utf_ptr2CharInfo_impl(next, (uintptr_t)next_len);
    if (!utf_iscomposing(prev_code, next_code, &state)) {
      return (StrCharInfo){
        .ptr = (char *)next,
        .chr = (CharInfo){ .value = next_code, .len = (next_code < 0 ? 1 : next_len) },
      };
    }

    prev_code = next_code;
    next += next_len;
    if (EXPECT(*next < 0x80U, true)) {
      return (StrCharInfo){
        .ptr = (char *)next,
        .chr = (CharInfo){ .value = *next, .len = 1 },
      };
    }
  }
}

/// Copy a character, advancing the pointers
///
/// @param[in,out]  fp  Source of the character to copy.
/// @param[in,out]  tp  Destination to copy to.
void mb_copy_char(const char **const fp, char **const tp)
{
  const size_t l = (size_t)utfc_ptr2len(*fp);

  memmove(*tp, *fp, l);
  *tp += l;
  *fp += l;
}

// Find the next illegal byte sequence.
void utf_find_illegal(void)
{
  pos_T pos = curwin->w_cursor;
  vimconv_T vimconv;
  char *tofree = NULL;

  vimconv.vc_type = CONV_NONE;
  if (enc_canon_props(curbuf->b_p_fenc) & ENC_8BIT) {
    // 'encoding' is "utf-8" but we are editing a 8-bit encoded file,
    // possibly a utf-8 file with illegal bytes.  Setup for conversion
    // from utf-8 to 'fileencoding'.
    convert_setup(&vimconv, p_enc, curbuf->b_p_fenc);
  }

  curwin->w_cursor.coladd = 0;
  while (true) {
    char *p = get_cursor_pos_ptr();
    if (vimconv.vc_type != CONV_NONE) {
      xfree(tofree);
      tofree = string_convert(&vimconv, p, NULL);
      if (tofree == NULL) {
        break;
      }
      p = tofree;
    }

    while (*p != NUL) {
      // Illegal means that there are not enough trail bytes (checked by
      // utf_ptr2len()) or too many of them (overlong sequence).
      int len = utf_ptr2len(p);
      if ((uint8_t)(*p) >= 0x80 && (len == 1 || utf_char2len(utf_ptr2char(p)) != len)) {
        if (vimconv.vc_type == CONV_NONE) {
          curwin->w_cursor.col += (colnr_T)(p - get_cursor_pos_ptr());
        } else {
          int l;

          len = (int)(p - tofree);
          for (p = get_cursor_pos_ptr(); *p != NUL && len-- > 0; p += l) {
            l = utf_ptr2len(p);
            curwin->w_cursor.col += l;
          }
        }
        goto theend;
      }
      p += len;
    }
    if (curwin->w_cursor.lnum == curbuf->b_ml.ml_line_count) {
      break;
    }
    curwin->w_cursor.lnum++;
    curwin->w_cursor.col = 0;
  }

  // didn't find it: don't move and beep
  curwin->w_cursor = pos;
  beep_flush();

theend:
  xfree(tofree);
  convert_setup(&vimconv, NULL, NULL);
}

// If the cursor moves on an trail byte, set the cursor on the lead byte.
// Thus it moves left if necessary.
void mb_adjust_cursor(void)
{
  mark_mb_adjustpos(curbuf, &curwin->w_cursor);
}

/// C accessor for mb_adjust_cursor (for Rust FFI).
void nvim_mb_adjust_cursor(void)
{
  mb_adjust_cursor();
}

/// Checks and adjusts cursor column. Not mode-dependent.
/// @see check_cursor_col
///
/// @param  win_  Places cursor on a valid column for this window.
void mb_check_adjust_col(void *win_)
{
  win_T *win = (win_T *)win_;
  colnr_T oldcol = win->w_cursor.col;

  // Column 0 is always valid.
  if (oldcol != 0) {
    char *p = ml_get_buf(win->w_buffer, win->w_cursor.lnum);
    colnr_T len = (colnr_T)strlen(p);

    // Empty line or invalid column?
    if (len == 0 || oldcol < 0) {
      win->w_cursor.col = 0;
    } else {
      // Cursor column too big for line?
      if (oldcol > len) {
        win->w_cursor.col = len - 1;
      }
      // Move the cursor to the head byte.
      win->w_cursor.col -= utf_head_off(p, p + win->w_cursor.col);
    }

    // Reset `coladd` when the cursor would be on the right half of a
    // double-wide character.
    if (win->w_cursor.coladd == 1 && p[win->w_cursor.col] != TAB
        && vim_isprintc(utf_ptr2char(p + win->w_cursor.col))
        && ptr2cells(p + win->w_cursor.col) > 1) {
      win->w_cursor.coladd = 0;
    }
  }
}

/// @param line  start of the string
///
/// @return      a pointer to the character before "*p", if there is one.
char *mb_prevptr(char *line, char *p)
{
  if (p > line) {
    MB_PTR_BACK(line, p);
  }
  return p;
}

/// Try to unescape a multibyte character
///
/// Used for the rhs and lhs of the mappings.
///
/// @param[in,out]  pp  String to unescape. Is advanced to just after the bytes
///                     that form a multibyte character.
///
/// @return Unescaped string if it is a multibyte character, NULL if no
///         multibyte character was found. Returns a static buffer, always one
///         and the same.
const char *mb_unescape(const char **const pp)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  static char buf[6];
  size_t buf_idx = 0;
  uint8_t *str = (uint8_t *)(*pp);

  // Must translate K_SPECIAL KS_SPECIAL KE_FILLER to K_SPECIAL.
  // Maximum length of a utf-8 character is 4 bytes.
  for (size_t str_idx = 0; str[str_idx] != NUL && buf_idx < 4; str_idx++) {
    if (str[str_idx] == K_SPECIAL
        && str[str_idx + 1] == KS_SPECIAL
        && str[str_idx + 2] == KE_FILLER) {
      buf[buf_idx++] = (char)K_SPECIAL;
      str_idx += 2;
    } else if (str[str_idx] == K_SPECIAL) {
      break;  // A special key can't be a multibyte char.
    } else {
      buf[buf_idx++] = (char)str[str_idx];
    }
    buf[buf_idx] = NUL;

    // Return a multi-byte character if it's found.  An illegal sequence
    // will result in a 1 here.
    if (utf_ptr2len(buf) > 1) {
      *pp = (const char *)str + str_idx + 1;
      return buf;
    }

    // Bail out quickly for ASCII.
    if ((uint8_t)buf[0] < 128) {
      break;
    }
  }
  return NULL;
}

// enc_canonize and enc_alias_search are implemented in Rust.

#ifdef HAVE_LANGINFO_H
# include <langinfo.h>
#endif

// Get the canonicalized encoding of the current locale.
// Returns an allocated string when successful, NULL when not.
char *enc_locale(void)
{
  int i;
  char buf[50];

  const char *s;

#ifdef HAVE_NL_LANGINFO_CODESET
  if (!(s = nl_langinfo(CODESET)) || *s == NUL)
#endif
  {
    if (!(s = setlocale(LC_CTYPE, NULL)) || *s == NUL) {
      if ((s = os_getenv_noalloc("LC_ALL"))) {
        if ((s = os_getenv_noalloc("LC_CTYPE"))) {
          s = os_getenv_noalloc("LANG");
        }
      }
    }
  }

  if (!s) {
    return NULL;
  }

  // The most generic locale format is:
  // language[_territory][.codeset][@modifier][+special][,[sponsor][_revision]]
  // If there is a '.' remove the part before it.
  // if there is something after the codeset, remove it.
  // Make the name lowercase and replace '_' with '-'.
  // Exception: "ja_JP.EUC" == "euc-jp", "zh_CN.EUC" = "euc-cn",
  // "ko_KR.EUC" == "euc-kr"
  const char *p = vim_strchr(s, '.');
  if (p != NULL) {
    if (p > s + 2 && !STRNICMP(p + 1, "EUC", 3)
        && !isalnum((uint8_t)p[4]) && p[4] != '-' && p[-3] == '_') {
      // Copy "XY.EUC" to "euc-XY" to buf[10].
      memmove(buf, "euc-", 4);
      buf[4] = (char)(ASCII_ISALNUM(p[-2]) ? TOLOWER_ASC(p[-2]) : 0);
      buf[5] = (char)(ASCII_ISALNUM(p[-1]) ? TOLOWER_ASC(p[-1]) : 0);
      buf[6] = NUL;
    } else {
      s = p + 1;
      goto enc_locale_copy_enc;
    }
  } else {
enc_locale_copy_enc:
    for (i = 0; i < (int)sizeof(buf) - 1 && s[i] != NUL; i++) {
      if (s[i] == '_' || s[i] == '-') {
        buf[i] = '-';
      } else if (ASCII_ISALNUM((uint8_t)s[i])) {
        buf[i] = (char)TOLOWER_ASC(s[i]);
      } else {
        break;
      }
    }
    buf[i] = NUL;
  }

  return enc_canonize(buf);
}

// Call iconv_open() with a check if iconv() works properly (there are broken
// versions).
// Returns (void *)-1 if failed.
// (should return iconv_t, but that causes problems with prototypes).
void *my_iconv_open(char *to, char *from)
{
#define ICONV_TESTLEN 400
  char tobuf[ICONV_TESTLEN];
  static WorkingStatus iconv_working = kUnknown;

  if (iconv_working == kBroken) {
    return (void *)-1;          // detected a broken iconv() previously
  }
  iconv_t fd = iconv_open(enc_skip(to), enc_skip(from));

  if (fd != (iconv_t)-1 && iconv_working == kUnknown) {
    // Do a dummy iconv() call to check if it actually works.  There is a
    // version of iconv() on Linux that is broken.  We can't ignore it,
    // because it's wide-spread.  The symptoms are that after outputting
    // the initial shift state the "to" pointer is NULL and conversion
    // stops for no apparent reason after about 8160 characters.
    char *p = tobuf;
    size_t tolen = ICONV_TESTLEN;
    iconv(fd, NULL, NULL, &p, &tolen);
    if (p == NULL) {
      iconv_working = kBroken;
      iconv_close(fd);
      fd = (iconv_t)-1;
    } else {
      iconv_working = kWorking;
    }
  }

  return (void *)fd;
}

// Convert the string "str[slen]" with iconv().
// If "unconvlenp" is not NULL handle the string ending in an incomplete
// sequence and set "*unconvlenp" to the length of it.
// Returns the converted string in allocated memory.  NULL for an error.
// If resultlenp is not NULL, sets it to the result length in bytes.
static char *iconv_string(const vimconv_T *const vcp, const char *str, size_t slen,
                          size_t *unconvlenp, size_t *resultlenp)
{
  char *to;
  size_t len = 0;
  size_t done = 0;
  char *result = NULL;

  const char *from = str;
  size_t fromlen = slen;
  while (true) {
    if (len == 0 || ICONV_ERRNO == ICONV_E2BIG) {
      // Allocate enough room for most conversions.  When re-allocating
      // increase the buffer size.
      len = len + fromlen * 2 + 40;
      char *p = xmalloc(len);
      if (done > 0) {
        memmove(p, result, done);
      }
      xfree(result);
      result = p;
    }

    to = result + done;
    size_t tolen = len - done - 2;
    // Avoid a warning for systems with a wrong iconv() prototype by
    // casting the second argument to void *.
    if (iconv(vcp->vc_fd, (void *)&from, &fromlen, &to, &tolen) != SIZE_MAX) {
      // Finished, append a NUL.
      *to = NUL;
      break;
    }

    // Check both ICONV_EINVAL and EINVAL, because the dynamically loaded
    // iconv library may use one of them.
    if (!vcp->vc_fail && unconvlenp != NULL
        && (ICONV_ERRNO == ICONV_EINVAL || ICONV_ERRNO == EINVAL)) {
      // Handle an incomplete sequence at the end.
      *to = NUL;
      *unconvlenp = fromlen;
      break;
    } else if (!vcp->vc_fail
               && (ICONV_ERRNO == ICONV_EILSEQ || ICONV_ERRNO == EILSEQ
                   || ICONV_ERRNO == ICONV_EINVAL || ICONV_ERRNO == EINVAL)) {
      // Check both ICONV_EILSEQ and EILSEQ, because the dynamically loaded
      // iconv library may use one of them.

      // Can't convert: insert a '?' and skip a character.  This assumes
      // conversion from 'encoding' to something else.  In other
      // situations we don't know what to skip anyway.
      *to++ = '?';
      if (utf_ptr2cells(from) > 1) {
        *to++ = '?';
      }
      int l = utfc_ptr2len_len(from, (int)fromlen);
      from += l;
      fromlen -= (size_t)l;
    } else if (ICONV_ERRNO != ICONV_E2BIG) {
      // conversion failed
      XFREE_CLEAR(result);
      break;
    }
    // Not enough room or skipping illegal sequence.
    done = (size_t)(to - result);
  }

  if (resultlenp != NULL && result != NULL) {
    *resultlenp = (size_t)(to - result);
  }
  return result;
}

/// iconv() function
void f_iconv(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  vimconv_T vimconv;

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  const char *const str = tv_get_string(&argvars[0]);
  char buf1[NUMBUFLEN];
  char *const from = enc_canonize(enc_skip((char *)tv_get_string_buf(&argvars[1], buf1)));
  char buf2[NUMBUFLEN];
  char *const to = enc_canonize(enc_skip((char *)tv_get_string_buf(&argvars[2], buf2)));
  vimconv.vc_type = CONV_NONE;
  convert_setup(&vimconv, from, to);

  // If the encodings are equal, no conversion needed.
  if (vimconv.vc_type == CONV_NONE) {
    rettv->vval.v_string = xstrdup(str);
  } else {
    rettv->vval.v_string = string_convert(&vimconv, (char *)str, NULL);
  }

  convert_setup(&vimconv, NULL, NULL);
  xfree(from);
  xfree(to);
}

/// Setup "vcp" for conversion from "from" to "to".
/// The names must have been made canonical with enc_canonize().
/// vcp->vc_type must have been initialized to CONV_NONE.
/// Note: cannot be used for conversion from/to ucs-2 and ucs-4 (will use utf-8
/// instead).
/// Afterwards invoke with "from" and "to" equal to NULL to cleanup.
///
/// @return  FAIL when conversion is not supported, OK otherwise.
int convert_setup(vimconv_T *vcp, char *from, char *to)
{
  return convert_setup_ext(vcp, from, true, to, true);
}

/// As convert_setup(), but only when from_unicode_is_utf8 is true will all
/// "from" unicode charsets be considered utf-8.  Same for "to".
int convert_setup_ext(vimconv_T *vcp, char *from, bool from_unicode_is_utf8, char *to,
                      bool to_unicode_is_utf8)
{
  int from_is_utf8;
  int to_is_utf8;

  // Reset to no conversion.
  if (vcp->vc_type == CONV_ICONV && vcp->vc_fd != (iconv_t)-1) {
    iconv_close(vcp->vc_fd);
  }
  *vcp = (vimconv_T)MBYTE_NONE_CONV;

  // No conversion when one of the names is empty or they are equal.
  if (from == NULL || *from == NUL || to == NULL || *to == NUL
      || strcmp(from, to) == 0) {
    return OK;
  }

  int from_prop = enc_canon_props(from);
  int to_prop = enc_canon_props(to);
  if (from_unicode_is_utf8) {
    from_is_utf8 = from_prop & ENC_UNICODE;
  } else {
    from_is_utf8 = from_prop == ENC_UNICODE;
  }
  if (to_unicode_is_utf8) {
    to_is_utf8 = to_prop & ENC_UNICODE;
  } else {
    to_is_utf8 = to_prop == ENC_UNICODE;
  }

  if ((from_prop & ENC_LATIN1) && to_is_utf8) {
    // Internal latin1 -> utf-8 conversion.
    vcp->vc_type = CONV_TO_UTF8;
    vcp->vc_factor = 2;         // up to twice as long
  } else if ((from_prop & ENC_LATIN9) && to_is_utf8) {
    // Internal latin9 -> utf-8 conversion.
    vcp->vc_type = CONV_9_TO_UTF8;
    vcp->vc_factor = 3;         // up to three as long (euro sign)
  } else if (from_is_utf8 && (to_prop & ENC_LATIN1)) {
    // Internal utf-8 -> latin1 conversion.
    vcp->vc_type = CONV_TO_LATIN1;
  } else if (from_is_utf8 && (to_prop & ENC_LATIN9)) {
    // Internal utf-8 -> latin9 conversion.
    vcp->vc_type = CONV_TO_LATIN9;
  } else {
    // Use iconv() for conversion.
    vcp->vc_fd = (iconv_t)my_iconv_open(to_is_utf8 ? "utf-8" : to,
                                        from_is_utf8 ? "utf-8" : from);
    if (vcp->vc_fd != (iconv_t)-1) {
      vcp->vc_type = CONV_ICONV;
      vcp->vc_factor = 4;       // could be longer too...
    }
  }
  if (vcp->vc_type == CONV_NONE) {
    return FAIL;
  }

  return OK;
}

/// Convert text "ptr[*lenp]" according to "vcp".
/// Returns the result in allocated memory and sets "*lenp".
/// When "lenp" is NULL, use NUL terminated strings.
/// Illegal chars are often changed to "?", unless vcp->vc_fail is set.
/// When something goes wrong, NULL is returned and "*lenp" is unchanged.
char *string_convert(const vimconv_T *const vcp, char *ptr, size_t *lenp)
{
  return string_convert_ext(vcp, ptr, lenp, NULL);
}

// Like string_convert(), but when "unconvlenp" is not NULL and there are is
// an incomplete sequence at the end it is not converted and "*unconvlenp" is
// set to the number of remaining bytes.
char *string_convert_ext(const vimconv_T *const vcp, char *ptr, size_t *lenp, size_t *unconvlenp)
{
  uint8_t *retval = NULL;
  uint8_t *d;
  int c;

  size_t len;
  if (lenp == NULL) {
    len = strlen(ptr);
  } else {
    len = *lenp;
  }
  if (len == 0) {
    return xstrdup("");
  }

  switch (vcp->vc_type) {
  case CONV_TO_UTF8:            // latin1 to utf-8 conversion
    retval = xmalloc(len * 2 + 1);
    d = retval;
    for (size_t i = 0; i < len; i++) {
      c = (uint8_t)ptr[i];
      if (c < 0x80) {
        *d++ = (uint8_t)c;
      } else {
        *d++ = (uint8_t)(0xc0 + (uint8_t)((unsigned)c >> 6));
        *d++ = (uint8_t)(0x80 + (c & 0x3f));
      }
    }
    *d = NUL;
    if (lenp != NULL) {
      *lenp = (size_t)(d - retval);
    }
    break;

  case CONV_9_TO_UTF8:          // latin9 to utf-8 conversion
    retval = xmalloc(len * 3 + 1);
    d = retval;
    for (size_t i = 0; i < len; i++) {
      c = (uint8_t)ptr[i];
      switch (c) {
      case 0xa4:
        c = 0x20ac; break;                 // euro
      case 0xa6:
        c = 0x0160; break;                 // S hat
      case 0xa8:
        c = 0x0161; break;                 // S -hat
      case 0xb4:
        c = 0x017d; break;                 // Z hat
      case 0xb8:
        c = 0x017e; break;                 // Z -hat
      case 0xbc:
        c = 0x0152; break;                 // OE
      case 0xbd:
        c = 0x0153; break;                 // oe
      case 0xbe:
        c = 0x0178; break;                 // Y
      }
      d += utf_char2bytes(c, (char *)d);
    }
    *d = NUL;
    if (lenp != NULL) {
      *lenp = (size_t)(d - retval);
    }
    break;

  case CONV_TO_LATIN1:          // utf-8 to latin1 conversion
  case CONV_TO_LATIN9:          // utf-8 to latin9 conversion
    retval = xmalloc(len + 1);
    d = retval;
    for (size_t i = 0; i < len; i++) {
      int l = utf_ptr2len_len(ptr + i, (int)(len - i));
      if (l == 0) {
        *d++ = NUL;
      } else if (l == 1) {
        uint8_t l_w = utf8len_tab_zero[(uint8_t)ptr[i]];

        if (l_w == 0) {
          // Illegal utf-8 byte cannot be converted
          xfree(retval);
          return NULL;
        }
        if (unconvlenp != NULL && l_w > len - i) {
          // Incomplete sequence at the end.
          *unconvlenp = len - i;
          break;
        }
        *d++ = (uint8_t)ptr[i];
      } else {
        c = utf_ptr2char(ptr + i);
        if (vcp->vc_type == CONV_TO_LATIN9) {
          switch (c) {
          case 0x20ac:
            c = 0xa4; break;                     // euro
          case 0x0160:
            c = 0xa6; break;                     // S hat
          case 0x0161:
            c = 0xa8; break;                     // S -hat
          case 0x017d:
            c = 0xb4; break;                     // Z hat
          case 0x017e:
            c = 0xb8; break;                     // Z -hat
          case 0x0152:
            c = 0xbc; break;                     // OE
          case 0x0153:
            c = 0xbd; break;                     // oe
          case 0x0178:
            c = 0xbe; break;                     // Y
          case 0xa4:
          case 0xa6:
          case 0xa8:
          case 0xb4:
          case 0xb8:
          case 0xbc:
          case 0xbd:
          case 0xbe:
            c = 0x100; break;                   // not in latin9
          }
        }
        if (!utf_iscomposing_legacy(c)) {  // skip composing chars
          if (c < 0x100) {
            *d++ = (uint8_t)c;
          } else if (vcp->vc_fail) {
            xfree(retval);
            return NULL;
          } else {
            *d++ = 0xbf;
            if (utf_char2cells(c) > 1) {
              *d++ = '?';
            }
          }
        }
        i += (size_t)l - 1;
      }
    }
    *d = NUL;
    if (lenp != NULL) {
      *lenp = (size_t)(d - retval);
    }
    break;

  case CONV_ICONV:  // conversion with vcp->vc_fd
    retval = (uint8_t *)iconv_string(vcp, ptr, len, unconvlenp, lenp);
    break;
  }

  return (char *)retval;
}

/// Table set by setcellwidths().
/// Exposed for Rust FFI access.
typedef struct {
  int64_t first;
  int64_t last;
  char width;
} cw_interval_T;

// Exposed for Rust FFI access (remove static).
cw_interval_T *cw_table = NULL;
size_t cw_table_size = 0;


static int tv_nr_compare(const void *a1, const void *a2)
{
  const listitem_T *const li1 = tv_list_first(*(const list_T **)a1);
  const listitem_T *const li2 = tv_list_first(*(const list_T **)a2);
  const varnumber_T n1 = TV_LIST_ITEM_TV(li1)->vval.v_number;
  const varnumber_T n2 = TV_LIST_ITEM_TV(li2)->vval.v_number;

  return n1 == n2 ? 0 : n1 > n2 ? 1 : -1;
}

/// "setcellwidths()" function
void f_setcellwidths(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (argvars[0].v_type != VAR_LIST || argvars[0].vval.v_list == NULL) {
    emsg(_(e_listreq));
    return;
  }

  const list_T *const l = argvars[0].vval.v_list;
  cw_interval_T *table = NULL;
  const size_t table_size = (size_t)tv_list_len(l);
  if (table_size == 0) {
    // Clearing the table.
    goto update;
  }

  // Note: use list_T instead of listitem_T so that TV_LIST_ITEM_NEXT can be used properly below.
  const list_T **ptrs = xmalloc(sizeof(const list_T *) * table_size);

  // Check that all entries are a list with three numbers, the range is
  // valid and the cell width is valid.
  int item = 0;
  TV_LIST_ITER_CONST(l, li, {
    const typval_T *const li_tv = TV_LIST_ITEM_TV(li);

    if (li_tv->v_type != VAR_LIST || li_tv->vval.v_list == NULL) {
      semsg(_(e_list_item_nr_is_not_list), item);
      xfree((void *)ptrs);
      return;
    }

    const list_T *const li_l = li_tv->vval.v_list;
    ptrs[item] = li_l;
    const listitem_T *lili = tv_list_first(li_l);
    int i;
    varnumber_T n1;
    for (i = 0; lili != NULL; lili = TV_LIST_ITEM_NEXT(li_l, lili), i++) {
      const typval_T *const lili_tv = TV_LIST_ITEM_TV(lili);
      if (lili_tv->v_type != VAR_NUMBER) {
        break;
      }
      if (i == 0) {
        n1 = lili_tv->vval.v_number;
        if (n1 < 0x80) {
          emsg(_(e_only_values_of_0x80_and_higher_supported));
          xfree((void *)ptrs);
          return;
        }
      } else if (i == 1 && lili_tv->vval.v_number < n1) {
        semsg(_(e_list_item_nr_range_invalid), item);
        xfree((void *)ptrs);
        return;
      } else if (i == 2 && (lili_tv->vval.v_number < 1 || lili_tv->vval.v_number > 2)) {
        semsg(_(e_list_item_nr_cell_width_invalid), item);
        xfree((void *)ptrs);
        return;
      }
    }

    if (i != 3) {
      semsg(_(e_list_item_nr_does_not_contain_3_numbers), item);
      xfree((void *)ptrs);
      return;
    }

    item++;
  });

  // Sort the list on the first number.
  qsort((void *)ptrs, table_size, sizeof(const list_T *), tv_nr_compare);

  table = xmalloc(sizeof(cw_interval_T) * table_size);

  // Store the items in the new table.
  for (item = 0; (size_t)item < table_size; item++) {
    const list_T *const li_l = ptrs[item];
    const listitem_T *lili = tv_list_first(li_l);
    const varnumber_T n1 = TV_LIST_ITEM_TV(lili)->vval.v_number;
    if (item > 0 && n1 <= table[item - 1].last) {
      semsg(_(e_overlapping_ranges_for_nr), (size_t)n1);
      xfree((void *)ptrs);
      xfree(table);
      return;
    }
    table[item].first = n1;
    lili = TV_LIST_ITEM_NEXT(li_l, lili);
    table[item].last = TV_LIST_ITEM_TV(lili)->vval.v_number;
    lili = TV_LIST_ITEM_NEXT(li_l, lili);
    table[item].width = (char)TV_LIST_ITEM_TV(lili)->vval.v_number;
  }

  xfree((void *)ptrs);

update:
  ;
  cw_interval_T *const cw_table_save = cw_table;
  const size_t cw_table_size_save = cw_table_size;
  cw_table = table;
  cw_table_size = table_size;

  // Check that the new value does not conflict with 'listchars' or
  // 'fillchars'.
  const char *const error = check_chars_options();
  if (error != NULL) {
    emsg(_(error));
    cw_table = cw_table_save;
    cw_table_size = cw_table_size_save;
    xfree(table);
    return;
  }

  xfree(cw_table_save);
  changed_window_setting_all();
  redraw_all_later(UPD_NOT_VALID);
}

/// "getcellwidths()" function
void f_getcellwidths(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, (ptrdiff_t)cw_table_size);

  for (size_t i = 0; i < cw_table_size; i++) {
    list_T *entry = tv_list_alloc(3);
    tv_list_append_number(entry, (varnumber_T)cw_table[i].first);
    tv_list_append_number(entry, (varnumber_T)cw_table[i].last);
    tv_list_append_number(entry, (varnumber_T)cw_table[i].width);

    tv_list_append_list(rettv->vval.v_list, entry);
  }
}

void f_charclass(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (tv_check_for_string_arg(argvars, 0) == FAIL
      || argvars[0].vval.v_string == NULL) {
    return;
  }
  rettv->vval.v_number = mb_get_class(argvars[0].vval.v_string);
}

// get_encoding_name is implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

