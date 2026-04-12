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
#include <uv.h>

#include "auto/config.h"
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

// Rust accessor functions for buffer properties

/// Check if current buffer has 'bomb' option set (accessor for Rust).
int nvim_curbuf_get_b_p_bomb(void) { return curbuf->b_p_bomb ? 1 : 0; }

/// Check if current buffer has 'binary' option set (accessor for Rust).
int nvim_curbuf_get_b_p_bin(void) { return curbuf->b_p_bin ? 1 : 0; }

/// Get current buffer's 'fileencoding' option (accessor for Rust).
const char *nvim_curbuf_get_b_p_fenc(void) { return curbuf->b_p_fenc; }


// mb_ptr2char_adv, utfc_ptr2schar, utfc_ptrlen2schar, schar_from_buf_first
// are implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

/// Accessor for Rust FFI: get UTF character from byte pointer.
int nvim_utf_ptr2char(const char *p) { return utf_ptr2char(p); }

/// Accessor for Rust FFI: get UTF character length including composing chars.
int nvim_utfc_ptr2len(const char *p) { return utfc_ptr2len(p); }


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

// utfc_next_impl, mb_copy_char are implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

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
void mb_adjust_cursor(void) { mark_mb_adjustpos(curbuf, &curwin->w_cursor); }

/// C accessor for mb_adjust_cursor (for Rust FFI).
void nvim_mb_adjust_cursor(void) { mb_adjust_cursor(); }

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

// mb_prevptr, mb_unescape are implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

// enc_canonize and enc_alias_search are implemented in Rust.

// enc_locale is implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

// my_iconv_open and iconv_string are implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

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

// convert_setup, convert_setup_ext, string_convert, string_convert_ext
// are implemented in Rust (src/nvim-rs/mbyte/src/lib.rs).

// cw_interval_T is defined in mbyte.h (exposed for Rust FFI).
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

/// Rust accessor: validate argvars[0] as a list and extract sorted cellwidths table.
///
/// Returns 0 on success, -1 on error (error already shown).
/// On success: *out_table is set to an xmalloc'd array (or NULL if empty),
/// and *out_size is set to the number of entries.
int nvim_mbyte_extract_cellwidths(const typval_T *argvars, void **out_table,
                                  size_t *out_size)
{
  if (argvars[0].v_type != VAR_LIST || argvars[0].vval.v_list == NULL) {
    emsg(_(e_listreq));
    return -1;
  }

  const list_T *const l = argvars[0].vval.v_list;
  const size_t table_size = (size_t)tv_list_len(l);
  *out_size = table_size;

  if (table_size == 0) {
    *out_table = NULL;
    return 0;
  }

  const list_T **ptrs = xmalloc(sizeof(const list_T *) * table_size);

  int item = 0;
  TV_LIST_ITER_CONST(l, li, {
    const typval_T *const li_tv = TV_LIST_ITEM_TV(li);

    if (li_tv->v_type != VAR_LIST || li_tv->vval.v_list == NULL) {
      semsg(_(e_list_item_nr_is_not_list), item);
      xfree((void *)ptrs);
      return -1;
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
          return -1;
        }
      } else if (i == 1 && lili_tv->vval.v_number < n1) {
        semsg(_(e_list_item_nr_range_invalid), item);
        xfree((void *)ptrs);
        return -1;
      } else if (i == 2 && (lili_tv->vval.v_number < 1 || lili_tv->vval.v_number > 2)) {
        semsg(_(e_list_item_nr_cell_width_invalid), item);
        xfree((void *)ptrs);
        return -1;
      }
    }

    if (i != 3) {
      semsg(_(e_list_item_nr_does_not_contain_3_numbers), item);
      xfree((void *)ptrs);
      return -1;
    }

    item++;
  });

  // Sort the list on the first number.
  qsort((void *)ptrs, table_size, sizeof(const list_T *), tv_nr_compare);

  cw_interval_T *table = xmalloc(sizeof(cw_interval_T) * table_size);

  // Store the items in the new table.
  for (item = 0; (size_t)item < table_size; item++) {
    const list_T *const li_l = ptrs[item];
    const listitem_T *lili = tv_list_first(li_l);
    const varnumber_T n1 = TV_LIST_ITEM_TV(lili)->vval.v_number;
    if (item > 0 && n1 <= table[item - 1].last) {
      semsg(_(e_overlapping_ranges_for_nr), (size_t)n1);
      xfree((void *)ptrs);
      xfree(table);
      return -1;
    }
    table[item].first = n1;
    lili = TV_LIST_ITEM_NEXT(li_l, lili);
    table[item].last = TV_LIST_ITEM_TV(lili)->vval.v_number;
    lili = TV_LIST_ITEM_NEXT(li_l, lili);
    table[item].width = (char)TV_LIST_ITEM_TV(lili)->vval.v_number;
  }

  xfree((void *)ptrs);
  *out_table = table;
  return 0;
}

/// Rust accessor: get the old cw_table and cw_table_size for save/restore.
void nvim_mbyte_get_cw_table_save(void **old_table, size_t *old_size)
{
  *old_table = cw_table;
  *old_size = cw_table_size;
}

/// Rust accessor: set new cw_table and cw_table_size.
void nvim_mbyte_set_cw_table(void *table, size_t size)
{
  cw_table = table;
  cw_table_size = size;
}

/// Rust accessor: check_chars_options() -- returns translated error string or NULL.
const char *nvim_mbyte_check_chars_options(void)
{
  const char *err = check_chars_options();
  return err != NULL ? _(err) : NULL;
}

/// Rust accessor: emsg() wrapper.
void nvim_mbyte_emsg(const char *msg) { emsg(msg); }

/// Rust accessor: changed_window_setting_all().
void nvim_mbyte_changed_window_setting_all(void) { changed_window_setting_all(); }

/// Rust accessor: redraw_all_later(UPD_NOT_VALID).
void nvim_mbyte_redraw_all_later(void) { redraw_all_later(UPD_NOT_VALID); }

// f_setcellwidths and tv_nr_compare are now implemented in Rust.

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

