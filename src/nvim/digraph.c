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

static const char e_digraph_must_be_just_two_characters_str[]
  = N_("E1214: Digraph must be just two characters: %s");
static const char e_digraph_argument_must_be_one_character_str[]
  = N_("E1215: Digraph must be one character: %s");
static const char e_digraph_setlist_argument_must_be_list_of_lists_with_two_items[]
  = N_("E1216: digraph_setlist() argument must be a list of lists with two items");

#include "digraph.c.generated.h"

// Rust implementations (functions exported directly from Rust)
// check_digraph_chars_valid, get_digraph_for_char, putdigraph are now in Rust (viml.rs)
extern void rs_registerdigraph(int char1, int char2, int result);

// Callback type for digraph iteration
typedef int (*DigraphIterCallback)(uint8_t char1, uint8_t char2, int result, void *ctx);
extern int rs_digraph_iterate_default(DigraphIterCallback callback, void *ctx);
extern int rs_digraph_iterate_user(DigraphIterCallback callback, void *ctx);

// Verify highlight constants match Rust values
_Static_assert(HLF_8 == 1, "HLF_8");
_Static_assert(HLF_CM == 11, "HLF_CM");

// digraphs added by the user
static garray_T user_digraphs = { 0, 0, (int)sizeof(digr_T), 10, NULL };

// Default digraph table is now in Rust (data.rs)
extern const digr_T *rs_get_digraphdefault(void);
extern int rs_get_digraphdefault_len(void);

// DELETED: digraphdefault[] array (~1400 entries) moved to Rust data.rs
// The following block was here: { 'N', 'U', 0x0a } through { NUL, NUL, NUL }
// plus all DG_START_* #defines and _Static_asserts

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

// header_table[] moved to Rust (list.rs HEADER_STRINGS)
// get_digraph_chars, digraph_set_common, f_digraph_get, f_digraph_set moved to Rust (funcs.rs)
// digraph_getlist_callback, digraph_getlist_common, f_digraph_getlist, f_digraph_setlist moved to Rust (funcs.rs)

extern bool digraph_set_common(const typval_T *argchars, const typval_T *argdigraph);

/// structure used for b_kmap_ga.ga_data
typedef struct {
  char *from;
  char *to;
} kmap_T;

#define KMAP_MAXLEN 20  // maximum length of "from" or "to"

/// Set up key mapping tables for the 'keymap' option.
///
/// @return NULL if OK, an error message for failure.  This only needs to be
///         used when setting the option, not later when the value has already
///         been checked.
char *keymap_init(void)
{
  curbuf->b_kmap_state &= ~KEYMAP_INIT;

  if (*curbuf->b_p_keymap == NUL) {
    // Stop any active keymap and clear the table.  Also remove
    // b:keymap_name, as no keymap is active now.
    keymap_unload();
    do_cmdline_cmd("unlet! b:keymap_name");
  } else {
    // Source the keymap file.  It will contain a ":loadkeymap" command
    // which will call ex_loadkeymap() below.
    size_t buflen = strlen(curbuf->b_p_keymap) + strlen(p_enc) + 14;
    char *buf = xmalloc(buflen);

    // try finding "keymap/'keymap'_'encoding'.vim"  in 'runtimepath'
    vim_snprintf(buf, buflen, "keymap/%s_%s.vim",
                 curbuf->b_p_keymap, p_enc);

    if (source_runtime(buf, 0) == FAIL) {
      // try finding "keymap/'keymap'.vim" in 'runtimepath'
      vim_snprintf(buf, buflen, "keymap/%s.vim",
                   curbuf->b_p_keymap);

      if (source_runtime(buf, 0) == FAIL) {
        xfree(buf);
        return N_("E544: Keymap file not found");
      }
    }
    xfree(buf);
  }

  return NULL;
}

/// ":loadkeymap" command: load the following lines as the keymap.
///
/// @param eap
void ex_loadkeymap(exarg_T *eap)
{
#define KMAP_LLEN 200  // max length of "to" and "from" together
  char buf[KMAP_LLEN + 11];
  char *save_cpo = p_cpo;

  if (!getline_equal(eap->ea_getline, eap->cookie, getsourceline)) {
    emsg(_("E105: Using :loadkeymap not in a sourced file"));
    return;
  }

  // Stop any active keymap and clear the table.
  keymap_unload();

  curbuf->b_kmap_state = 0;
  ga_init(&curbuf->b_kmap_ga, (int)sizeof(kmap_T), 20);

  // Set 'cpoptions' to "C" to avoid line continuation.
  p_cpo = "C";

  // Get each line of the sourced file, break at the end.
  while (true) {
    char *line = eap->ea_getline(0, eap->cookie, 0, true);

    if (line == NULL) {
      break;
    }

    char *p = skipwhite(line);

    if ((*p != '"') && (*p != NUL)) {
      kmap_T *kp = GA_APPEND_VIA_PTR(kmap_T, &curbuf->b_kmap_ga);
      char *s = skiptowhite(p);
      kp->from = xmemdupz(p, (size_t)(s - p));
      p = skipwhite(s);
      s = skiptowhite(p);
      kp->to = xmemdupz(p, (size_t)(s - p));

      if ((strlen(kp->from) + strlen(kp->to) >= KMAP_LLEN)
          || (*kp->from == NUL)
          || (*kp->to == NUL)) {
        if (*kp->to == NUL) {
          emsg(_("E791: Empty keymap entry"));
        }
        xfree(kp->from);
        xfree(kp->to);
        curbuf->b_kmap_ga.ga_len--;
      }
    }
    xfree(line);
  }

  // setup ":lmap" to map the keys
  for (int i = 0; i < curbuf->b_kmap_ga.ga_len; i++) {
    vim_snprintf(buf, sizeof(buf), "<buffer> %s %s",
                 ((kmap_T *)curbuf->b_kmap_ga.ga_data)[i].from,
                 ((kmap_T *)curbuf->b_kmap_ga.ga_data)[i].to);
    do_map(MAPTYPE_MAP, buf, MODE_LANGMAP, false);
  }

  p_cpo = save_cpo;

  curbuf->b_kmap_state |= KEYMAP_LOADED;
  status_redraw_curbuf();
}

/// Frees the buf_T.b_kmap_ga field of a buffer.
void keymap_ga_clear(garray_T *kmap_ga)
{
  kmap_T *kp = (kmap_T *)kmap_ga->ga_data;
  for (int i = 0; i < kmap_ga->ga_len; i++) {
    xfree(kp[i].from);
    xfree(kp[i].to);
  }
}

/// Stop using 'keymap'.
static void keymap_unload(void)
{
  char buf[KMAP_MAXLEN + 10];
  char *save_cpo = p_cpo;

  if (!(curbuf->b_kmap_state & KEYMAP_LOADED)) {
    return;
  }

  // Set 'cpoptions' to "C" to avoid line continuation.
  p_cpo = "C";

  // clear the ":lmap"s
  kmap_T *kp = (kmap_T *)curbuf->b_kmap_ga.ga_data;

  for (int i = 0; i < curbuf->b_kmap_ga.ga_len; i++) {
    vim_snprintf(buf, sizeof(buf), "<buffer> %s", kp[i].from);
    do_map(MAPTYPE_UNMAP, buf, MODE_LANGMAP, false);
  }
  keymap_ga_clear(&curbuf->b_kmap_ga);

  p_cpo = save_cpo;

  ga_clear(&curbuf->b_kmap_ga);
  curbuf->b_kmap_state &= ~KEYMAP_LOADED;
  status_redraw_curbuf();
}

/// Get the value to show for the language mappings, active 'keymap'.
///
/// @param fmt  format string containing one %s item
/// @param buf  buffer for the result
/// @param len  length of buffer
int get_keymap_str(win_T *wp, char *fmt, char *buf, int len)
{
  char *p;

  if (wp->w_buffer->b_p_iminsert != B_IMODE_LMAP) {
    return 0;
  }

  buf_T *old_curbuf = curbuf;
  win_T *old_curwin = curwin;
  char to_evaluate[] = "b:keymap_name";

  curbuf = wp->w_buffer;
  curwin = wp;
  emsg_skip++;
  char *s = p = eval_to_string(to_evaluate, false, false);
  emsg_skip--;
  curbuf = old_curbuf;
  curwin = old_curwin;
  if (p == NULL || *p == NUL) {
    if (wp->w_buffer->b_kmap_state & KEYMAP_LOADED) {
      p = wp->w_buffer->b_p_keymap;
    } else {
      p = "lang";
    }
  }
  int plen = vim_snprintf(buf, (size_t)len, fmt, p);
  xfree(s);
  if (plen < 0 || plen > len - 1) {
    buf[0] = NUL;
    plen = 0;
  }

  return plen;
}
