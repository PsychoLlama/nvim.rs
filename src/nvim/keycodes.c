#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <uv.h>

#include "nvim/api/private/defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/errors.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/option_vars.h"
#include "nvim/strings.h"

#include "keycode_names.generated.h"
#include "keycodes.c.generated.h"

extern int rs_name_to_mod_mask(int c);
extern int rs_handle_x_keys(int key);
extern int rs_simplify_key(int key, int *modifiers);
extern int rs_get_mouse_button(int code, bool *is_click, bool *is_drag);
extern void rs_vim_unescape_ks(char *p);
extern char *rs_add_char2buf(int c, char *s);
extern char *rs_vim_strsave_escape_ks(char *p);
extern unsigned rs_special_to_buf(int key, int modifiers, bool escape_ks, char *dst);

typedef struct {
  int key;
  int did_simplify;
} ExtractModifiersResult;

extern ExtractModifiersResult rs_extract_modifiers(int key, int *modp, bool simplify);
extern int rs_find_special_key_in_table(int c);
extern int rs_get_special_key_code(const char *name);
extern char *rs_get_special_key_name(int c, int modifiers);
extern int rs_find_special_key(const char **srcp, size_t src_len, int *modp, int flags,
                                bool *did_simplify);
extern unsigned rs_trans_special(const char **srcp, size_t src_len, char *dst, int flags,
                                  bool escape_ks, bool *did_simplify);
extern char *rs_replace_termcodes(const char *from, size_t from_len, char *buf,
                                   scid_T sid_arg, int flags, bool *did_simplify,
                                   bool do_backslash, bool do_special);

// =============================================================================
// Accessor functions for key_names_table (for Rust FFI)
// =============================================================================

/// Get the length of the key_names_table array.
int nvim_get_key_names_table_len(void)
{
  return (int)ARRAY_SIZE(key_names_table);
}

/// Get the key code at the specified index in key_names_table.
int nvim_get_key_names_table_key(int idx)
{
  if (idx < 0 || idx >= (int)ARRAY_SIZE(key_names_table)) {
    return 0;
  }
  return key_names_table[idx].key;
}

/// Get whether the entry at idx is an alternative name.
bool nvim_get_key_names_table_is_alt(int idx)
{
  if (idx < 0 || idx >= (int)ARRAY_SIZE(key_names_table)) {
    return false;
  }
  return key_names_table[idx].is_alt;
}

/// Get the name data pointer at the specified index.
const char *nvim_get_key_names_table_name_data(int idx)
{
  if (idx < 0 || idx >= (int)ARRAY_SIZE(key_names_table)) {
    return NULL;
  }
  return key_names_table[idx].name.data;
}

/// Get the name length at the specified index.
size_t nvim_get_key_names_table_name_size(int idx)
{
  if (idx < 0 || idx >= (int)ARRAY_SIZE(key_names_table)) {
    return 0;
  }
  return key_names_table[idx].name.size;
}

// =============================================================================
// Accessor functions for replace_termcodes (for Rust FFI)
// =============================================================================

/// Get current script ID for <SID> translation.
scid_T nvim_keycodes_get_current_sid(void)
{
  return current_sctx.sc_sid;
}

/// Get value of g:mapleader variable.
/// Returns NULL if not set.
char *nvim_keycodes_get_leader(void)
{
  return get_var_value("g:mapleader");
}

/// Get value of g:maplocalleader variable.
/// Returns NULL if not set.
char *nvim_keycodes_get_local_leader(void)
{
  return get_var_value("g:maplocalleader");
}

/// Emit the "using <SID> not in script context" error message.
void nvim_keycodes_emit_sid_error(void)
{
  emsg(_(e_usingsid));
}

/// Lookup a special key code by name using the generated hash function.
/// Returns the index in key_names_table, or -1 if not found.
int nvim_get_special_key_code_hash(const char *name, size_t len)
{
  return get_special_key_code_hash(name, len);
}

/// Return the modifier mask bit (#MOD_MASK_*) corresponding to mod name
///
/// E.g. 'S' for shift, 'C' for ctrl.
int name_to_mod_mask(int c)
  FUNC_ATTR_CONST FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_name_to_mod_mask(c);
}

/// Check if there is a special key code for "key" with specified modifiers
///
/// @param[in]  key  Initial key code.
/// @param[in,out]  modifiers  Initial modifiers, is adjusted to have simplified
///                            modifiers.
///
/// @return Simplified key code.
int simplify_key(const int key, int *modifiers)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_simplify_key(key, modifiers);
}

/// Change <xKey> to <Key>
int handle_x_keys(const int key)
  FUNC_ATTR_CONST FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_handle_x_keys(key);
}

/// @return  a string which contains the name of the given key when the given modifiers are down.
char *get_special_key_name(int c, int modifiers)
{
  return rs_get_special_key_name(c, modifiers);
}

/// Try translating a <> name ("keycode").
///
/// @param[in,out]  srcp  Source from which <> are translated. Is advanced to
///                       after the <> name if there is a match.
/// @param[in]  src_len  Length of the srcp.
/// @param[out]  dst  Location where translation result will be kept. It must
//                    be at least 19 bytes per "<x>" form.
/// @param[in]  flags  FSK_ values
/// @param[in]  escape_ks  escape K_SPECIAL bytes in the character
/// @param[out]  did_simplify  found <C-H>, etc.
///
/// @return Number of characters added to dst, zero for no match.
unsigned trans_special(const char **const srcp, const size_t src_len, char *const dst,
                       const int flags, const bool escape_ks, bool *const did_simplify)
  FUNC_ATTR_NONNULL_ARG(1, 3) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_trans_special(srcp, src_len, dst, flags, escape_ks, did_simplify);
}

/// Put the character sequence for "key" with "modifiers" into "dst" and return
/// the resulting length.
/// When "escape_ks" is true escape K_SPECIAL bytes in the character.
/// The sequence is not NUL terminated.
/// This is how characters in a string are encoded.
unsigned special_to_buf(int key, int modifiers, bool escape_ks, char *dst)
{
  return rs_special_to_buf(key, modifiers, escape_ks, dst);
}

/// Try translating a <> name
///
/// @param[in,out]  srcp  Translated <> name. Is advanced to after the <> name.
/// @param[in]  src_len  srcp length.
/// @param[out]  modp  Location where information about modifiers is saved.
/// @param[in]  flags  FSK_ values
/// @param[out]  did_simplify  FSK_SIMPLIFY and found <C-H>, etc.
///
/// @return Key and modifiers or 0 if there is no match.
int find_special_key(const char **const srcp, const size_t src_len, int *const modp,
                     const int flags, bool *const did_simplify)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(1, 3)
{
  return rs_find_special_key(srcp, src_len, modp, flags, did_simplify);
}

/// Try to include modifiers (except alt/meta) in the key.
/// Changes "Shift-a" to 'A', "Ctrl-@" to <Nul>, etc.
/// @param[in]  simplify  if false, don't do Ctrl
/// @param[out]  did_simplify  set when it is not NULL and "simplify" is true and
///                            Ctrl is removed from modifiers
static int extract_modifiers(int key, int *modp, const bool simplify, bool *const did_simplify)
{
  ExtractModifiersResult result = rs_extract_modifiers(key, modp, simplify);
  if (did_simplify != NULL) {
    *did_simplify = (bool)result.did_simplify;
  }
  return result.key;
}

/// Try to find key "c" in the special key table.
/// @return  the index when found, -1 when not found.
int find_special_key_in_table(int c)
{
  return rs_find_special_key_in_table(c);
}

/// Find the special key with the given name
///
/// @param[in]  name  Name of the special. Does not have to end with NUL, it is
///                   assumed to end before the first non-idchar. If name starts
///                   with "t_" the next two characters are interpreted as
///                   a termcap name.
///
/// @return Key code or 0 if not found.
int get_special_key_code(const char *name)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_get_special_key_code(name);
}

/// Look up the given mouse code to return the relevant information in the other arguments.
/// @return  which button is down or was released.
int get_mouse_button(int code, bool *is_click, bool *is_drag)
{
  return rs_get_mouse_button(code, is_click, is_drag);
}

/// Replace any terminal code strings with the equivalent internal representation.
///
/// Used for the "from" and "to" part of a mapping, and the "to" part of a menu command.
/// Any strings like "<C-UP>" are also replaced, unless `special` is false.
/// K_SPECIAL by itself is replaced by K_SPECIAL KS_SPECIAL KE_FILLER.
///
/// When "flags" has REPTERM_FROM_PART, trailing <C-v> is included, otherwise it is removed (to make
/// ":map xx ^V" map xx to nothing). When cpo_val contains CPO_BSLASH, a backslash can be used in
/// place of <C-v>. All other <C-v> characters are removed.
///
/// @param[in]  from  What characters to replace.
/// @param[in]  from_len  Length of the "from" argument.
/// @param[out]  bufp  Location where results were saved in case of success (allocated).
///                    If `*bufp` is non-NULL, it will be used directly,
///                    and is assumed to be 128 bytes long (enough for transcoding LHS of mapping),
///                    and will be set to NULL in case of failure.
/// @param[in]  sid_arg  Script ID to use for <SID>, or 0 to use current_sctx
/// @param[in]  flags  REPTERM_FROM_PART    see above
///                    REPTERM_DO_LT        also translate <lt>
///                    REPTERM_NO_SPECIAL   do not accept <key> notation
///                    REPTERM_NO_SIMPLIFY  do not simplify <C-H> into 0x08, etc.
/// @param[out]  did_simplify  set when some <C-H> code was simplified, unless it is NULL.
/// @param[in]  cpo_val  The value of 'cpoptions' to use. Only CPO_BSLASH matters.
///
/// @return  The same as what `*bufp` is set to.
char *replace_termcodes(const char *const from, const size_t from_len, char **const bufp,
                        const scid_T sid_arg, const int flags, bool *const did_simplify,
                        const char *const cpo_val)
  FUNC_ATTR_NONNULL_ARG(1, 3, 7)
{
  // backslash is a special character
  const bool do_backslash = (vim_strchr(cpo_val, CPO_BSLASH) == NULL);
  const bool do_special = !(flags & REPTERM_NO_SPECIAL);

  bool allocated = (*bufp == NULL);

  // Allocate space for the translation.  Worst case a single character is
  // replaced by 6 bytes (shifted special key), plus a NUL at the end.
  const size_t buf_len = allocated ? from_len * 6 + 1 : 128;
  char *result = allocated ? xmalloc(buf_len) : *bufp;

  char *ret = rs_replace_termcodes(from, from_len, result, sid_arg, flags,
                                   did_simplify, do_backslash, do_special);

  if (ret == NULL) {
    // Overflow with fixed buffer
    if (allocated) {
      xfree(result);
    }
    *bufp = NULL;
    return NULL;
  }

  if (allocated) {
    *bufp = xrealloc(result, strlen(result) + 1);
  }

  return *bufp;
}

/// Add character "c" to buffer "s"
///
/// Escapes the special meaning of K_SPECIAL, handles multi-byte
/// characters.
///
/// @param[in]  c  Character to add.
/// @param[out]  s  Buffer to add to. Must have at least MB_MAXBYTES + 1 bytes.
///
/// @return Pointer to after the added bytes.
char *add_char2buf(int c, char *s)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_add_char2buf(c, s);
}

/// Copy "p" to allocated memory, escaping K_SPECIAL so that the result
/// can be put in the typeahead buffer.
char *vim_strsave_escape_ks(char *p)
{
  return rs_vim_strsave_escape_ks(p);
}

/// Remove escaping from K_SPECIAL characters.  Reverse of
/// vim_strsave_escape_ks().  Works in-place.
void vim_unescape_ks(char *p)
{
  rs_vim_unescape_ks(p);
}
