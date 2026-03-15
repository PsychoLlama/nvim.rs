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

// replace_termcodes is now exported directly from Rust
// (src/nvim-rs/keycodes/src/lib.rs via #[unsafe(export_name = "replace_termcodes")]).

