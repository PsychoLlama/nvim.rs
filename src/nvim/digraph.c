/// @file digraph.c
///
/// code for digraphs

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/garray.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/keycodes.h"
#include "nvim/mapping.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/normal.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/runtime.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

typedef int result_T;

typedef struct {
  uint8_t char1;
  uint8_t char2;
  result_T result;
} digr_T;

#include "digraph.c.generated.h"

// Rust implementations (functions exported directly from Rust)
// check_digraph_chars_valid, get_digraph_for_char, putdigraph are now in Rust (viml.rs)
// rs_registerdigraph, rs_digraph_iterate_* are now pure Rust exports
// keymap_ga_clear, get_keymap_str are now in Rust (digraph/src/keymap.rs)

// Rust-exported keymap functions are declared in digraph.h


// Verify highlight constants match Rust values
_Static_assert(HLF_8 == 1, "HLF_8");
_Static_assert(HLF_CM == 11, "HLF_CM");

// digraphs added by the user
static garray_T user_digraphs = { 0, 0, (int)sizeof(digr_T), 10, NULL };

// Default digraph table is now in Rust (data.rs)
// rs_get_digraphdefault and rs_get_digraphdefault_len are Rust exports

// Accessor functions for Rust FFI

/// Get pointer to user digraphs array data (opaque for Rust).
void *nvim_get_user_digraphs_data(void) { return user_digraphs.ga_data; }

/// Get length of user digraphs array.
int nvim_get_user_digraphs_len(void) { return user_digraphs.ga_len; }

/// Get pointer to user digraphs garray for mutation.
void *nvim_get_user_digraphs_ptr(void) { return &user_digraphs; }

/// Grow the user digraphs garray by n items.
void nvim_user_digraphs_grow(int n) { ga_grow(&user_digraphs, n); }

/// Increment the user digraphs garray length.
void nvim_user_digraphs_inc_len(void) { user_digraphs.ga_len++; }

/// Get the value of the 'digraph' option.
int nvim_get_p_dg(void) { return p_dg; }

/// Check if a character is a composing character (for Rust FFI).
int nvim_utf_iscomposing_first(int c) { return utf_iscomposing_first(c); }

/// Get display width of a character in cells (for Rust FFI).
int nvim_char2cells(int c) { return char2cells(c); }

/// Check if user pressed Ctrl-C (for Rust FFI).
int nvim_digraph_got_int(void) { return got_int; }

/// Fast check for user interrupt (for Rust FFI).
void nvim_digraph_fast_breakcheck(void) { fast_breakcheck(); }

/// Get a character without mapping (for Rust FFI).
int nvim_digraph_plain_vgetc(void) { return plain_vgetc(); }

/// Increment no_mapping and allow_keys (for Rust FFI).
void nvim_digraph_inc_no_mapping(void)
{
  no_mapping++;
  allow_keys++;
}

/// Decrement no_mapping and allow_keys (for Rust FFI).
void nvim_digraph_dec_no_mapping(void)
{
  no_mapping--;
  allow_keys--;
}

/// Get cmdline_star value (for Rust FFI).
int nvim_digraph_get_cmdline_star(void) { return cmdline_star; }

/// Put a character on the command line (for Rust FFI).
void nvim_digraph_putcmdline(int c, int shift) { putcmdline((char)c, shift != 0); }

/// Add a character to the showcmd display (for Rust FFI).
void nvim_digraph_add_to_showcmd(int c) { add_to_showcmd(c); }

/// Append a list to a list (for Rust FFI).
void nvim_tv_list_append_list(list_T *l, list_T *itemlist) { tv_list_append_list(l, itemlist); }

/// Check for optional bool arg in typval array (for Rust FFI). Returns OK or FAIL.
int nvim_tv_check_for_opt_bool_arg(const typval_T *args, int idx) { return tv_check_for_opt_bool_arg(args, idx); }

// get_digraph_for_char(), check_digraph_chars_valid(), putdigraph() moved to Rust (viml.rs)

// digraph_header(), listdigraphs(), printdigraph() moved to Rust (list.rs)
// f_digraph_*, digraph_getlist_*, digraph_set_common moved to Rust (funcs.rs)

/// structure used for b_kmap_ga.ga_data
typedef struct {
  char *from;
  char *to;
} kmap_T;

#define KMAP_MAXLEN 20  // maximum length of "from" or "to"

// kmap_T field accessors for Rust FFI

/// Get the 'from' field of a kmap_T entry (for Rust FFI).
char *nvim_kmap_entry_get_from(void *entry) { return ((kmap_T *)entry)->from; }

/// Get the 'to' field of a kmap_T entry (for Rust FFI).
char *nvim_kmap_entry_get_to(void *entry) { return ((kmap_T *)entry)->to; }

/// Set the 'from' field of a kmap_T entry (for Rust FFI).
void nvim_kmap_entry_set_from(void *entry, char *val) { ((kmap_T *)entry)->from = val; }

/// Set the 'to' field of a kmap_T entry (for Rust FFI).
void nvim_kmap_entry_set_to(void *entry, char *val) { ((kmap_T *)entry)->to = val; }

/// Get sizeof(kmap_T) (for Rust FFI).
size_t nvim_kmap_entry_size(void) { return sizeof(kmap_T); }

// curbuf kmap state accessors for keymap_unload / keymap_init (Rust)

/// Get curbuf->b_kmap_state (for Rust FFI).
int nvim_curbuf_get_b_kmap_state(void) { return (int)curbuf->b_kmap_state; }

/// Clear specific bits in curbuf->b_kmap_state (for Rust FFI).
void nvim_curbuf_clear_b_kmap_state_bits(int mask) { curbuf->b_kmap_state &= (int16_t)~mask; }

/// Get pointer to curbuf->b_kmap_ga (as opaque garray) (for Rust FFI).
garray_T *nvim_curbuf_get_b_kmap_ga(void) { return &curbuf->b_kmap_ga; }

/// Get curbuf->b_p_keymap (for Rust FFI).
const char *nvim_curbuf_get_b_p_keymap(void) { return curbuf->b_p_keymap; }

/// Set p_cpo global (for Rust FFI, to restore after operations).
void nvim_set_p_cpo(char *val) { p_cpo = val; }

/// Get p_enc global (for Rust FFI, keymap file path construction).
const char *nvim_get_p_enc(void) { return p_enc; }

/// Call do_map with MAPTYPE_MAP/UNMAP for keymap operations (for Rust FFI).
/// mode is MODE_LANGMAP, abbr is false.
int nvim_do_map_keymap(int maptype, char *arg) { return do_map(maptype, arg, MODE_LANGMAP, false); }
// Note: nvim_ga_clear already exists in fold_shim.c
// Note: nvim_get_p_cpo already exported from Rust (window/src/globals.rs)

// Additional accessors for ex_loadkeymap (Phase 3)

/// Set all bits in curbuf->b_kmap_state (for Rust FFI).
void nvim_curbuf_set_b_kmap_state_bits(int mask) { curbuf->b_kmap_state |= (int16_t)mask; }

/// Set curbuf->b_kmap_state to zero (for Rust FFI).
void nvim_curbuf_zero_b_kmap_state(void) { curbuf->b_kmap_state = 0; }

/// Decrement curbuf->b_kmap_ga.ga_len by 1 (for Rust FFI).
void nvim_curbuf_kmap_ga_dec_len(void) { curbuf->b_kmap_ga.ga_len--; }

/// Initialize curbuf->b_kmap_ga with kmap_T item size (for Rust FFI).
void nvim_curbuf_kmap_ga_init(void) { ga_init(&curbuf->b_kmap_ga, (int)sizeof(kmap_T), 20); }

/// Append a new kmap_T entry to curbuf->b_kmap_ga; returns pointer to new entry (for Rust FFI).
void *nvim_curbuf_kmap_ga_append(void) { return ga_append_via_ptr(&curbuf->b_kmap_ga, sizeof(kmap_T)); }

// keymap_init() moved to Rust (digraph/src/keymap.rs)
// ex_loadkeymap() moved to Rust (digraph/src/keymap.rs)

// keymap_ga_clear() moved to Rust (digraph/src/keymap.rs)
// keymap_unload() moved to Rust (digraph/src/keymap.rs)

// get_keymap_str() moved to Rust (digraph/src/keymap.rs)
