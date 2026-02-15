#include <stdbool.h>
#include <string.h>

#include "nvim/errors.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/strings.h"

#include "keycode_names.generated.h"
#include "keycodes.c.generated.h"

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

