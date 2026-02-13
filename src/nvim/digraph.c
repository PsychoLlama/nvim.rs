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

// Rust implementations
extern int rs_digraph_get(int char1, int char2, int meta_char);
extern int rs_getexactdigraph(int char1, int char2, int meta_char);
extern int rs_check_digraph_chars_valid(int char1, int char2);
extern void rs_registerdigraph(int char1, int char2, int result);
extern int rs_get_digraph_for_char(int val, uint8_t *out_char1, uint8_t *out_char2);
extern int rs_do_digraph(int c);
typedef struct {
  int error_code;  // 0 = success, 1 = char validation error, 2 = number expected
  int char1;       // First character (for error messages)
  int char2;       // Second character (for error messages)
} PutdigraphResult;

extern int rs_putdigraph(char **str, PutdigraphResult *result);
extern int rs_digraph_get_header_index(int previous, int current);
extern int rs_digraph_format_entry(uint8_t char1, uint8_t char2, int result, char *buf, int buf_len);

// Callback type for digraph iteration
typedef int (*DigraphIterCallback)(uint8_t char1, uint8_t char2, int result, void *ctx);
extern int rs_digraph_iterate_default(DigraphIterCallback callback, void *ctx);
extern int rs_digraph_iterate_user(DigraphIterCallback callback, void *ctx);

// digraphs added by the user
static garray_T user_digraphs = { 0, 0, (int)sizeof(digr_T), 10, NULL };

// Default digraph table is now in Rust (data.rs)
extern const digr_T *rs_get_digraphdefault(void);
extern int rs_get_digraphdefault_len(void);

// DELETED: digraphdefault[] array (~1400 entries) moved to Rust data.rs
// The following block was here: { 'N', 'U', 0x0a } through { NUL, NUL, NUL }
// plus all DG_START_* #defines and _Static_asserts

// =============================================================================
// Accessor functions for Rust FFI
// =============================================================================

/// Get pointer to user digraphs array data (opaque for Rust).
void *nvim_get_user_digraphs_data(void)
{
  return user_digraphs.ga_data;
}

/// Get length of user digraphs array.
int nvim_get_user_digraphs_len(void)
{
  return user_digraphs.ga_len;
}

/// Get pointer to user digraphs garray for mutation.
void *nvim_get_user_digraphs_ptr(void)
{
  return &user_digraphs;
}

/// Grow the user digraphs garray by n items.
void nvim_user_digraphs_grow(int n)
{
  ga_grow(&user_digraphs, n);
}

/// Increment the user digraphs garray length.
void nvim_user_digraphs_inc_len(void)
{
  user_digraphs.ga_len++;
}

/// Get the value of the 'digraph' option.
int nvim_get_p_dg(void)
{
  return p_dg;
}

/// Check if a character is a composing character (for Rust FFI).
int nvim_utf_iscomposing_first(int c)
{
  return utf_iscomposing_first(c);
}

/// Get display width of a character in cells (for Rust FFI).
int nvim_char2cells(int c)
{
  return char2cells(c);
}

/// Check if user pressed Ctrl-C (for Rust FFI).
int nvim_digraph_got_int(void)
{
  return got_int;
}

/// Fast check for user interrupt (for Rust FFI).
void nvim_digraph_fast_breakcheck(void)
{
  fast_breakcheck();
}

/// handle digraphs after typing a character
///
/// @param c
///
/// @return The digraph.
int do_digraph(int c)
{
  return rs_do_digraph(c);
}

/// Find a digraph for "val".  If found return the string to display it.
/// If not found return NULL.
char *get_digraph_for_char(int val_arg)
{
  static char r[3];
  uint8_t char1, char2;

  if (rs_get_digraph_for_char(val_arg, &char1, &char2)) {
    r[0] = (char)char1;
    r[1] = (char)char2;
    r[2] = NUL;
    return r;
  }
  return NULL;
}

/// Get a digraph.  Used after typing CTRL-K on the command line or in normal
/// mode.
///
/// @param cmdline true when called from the cmdline
///
/// @returns composed character, or NUL when ESC was used.
int get_digraph(bool cmdline)
{
  no_mapping++;
  allow_keys++;
  int c = plain_vgetc();
  no_mapping--;
  allow_keys--;

  if (c == ESC) {  // ESC cancels CTRL-K
    return NUL;
  }

  if (IS_SPECIAL(c)) {
    // insert special key code
    return c;
  }

  if (cmdline) {
    if ((char2cells(c) == 1) && c < 128 && (cmdline_star == 0)) {
      putcmdline((char)c, true);
    }
  } else {
    add_to_showcmd(c);
  }
  no_mapping++;
  allow_keys++;
  int cc = plain_vgetc();
  no_mapping--;
  allow_keys--;

  if (cc != ESC) {
    // ESC cancels CTRL-K
    return digraph_get(c, cc, true);
  }
  return NUL;
}

/// Lookup the pair "char1", "char2" in the digraph tables.
///
/// @param char1
/// @param char2
/// @param meta_char
///
/// @return If no match, return "char2". If "meta_char" is true and "char1"
//          is a space, return "char2" | 0x80.
static int getexactdigraph(int char1, int char2, bool meta_char)
  FUNC_ATTR_PURE
{
  return rs_getexactdigraph(char1, char2, meta_char ? 1 : 0);
}

/// Get digraph.
/// Allow for both char1-char2 and char2-char1
///
/// @param char1
/// @param char2
/// @param meta_char
///
/// @return The digraph.
int digraph_get(int char1, int char2, bool meta_char)
  FUNC_ATTR_PURE
{
  return rs_digraph_get(char1, char2, meta_char ? 1 : 0);
}

/// Add a digraph to the digraph table.
static void registerdigraph(int char1, int char2, int n)
{
  rs_registerdigraph(char1, char2, n);
}

/// Check the characters are valid for a digraph.
/// If they are valid, returns true; otherwise, give an error message and
/// returns false.
bool check_digraph_chars_valid(int char1, int char2)
{
  int result = rs_check_digraph_chars_valid(char1, char2);
  switch (result) {
  case 1: {
    // char2 is 0 - digraph must be two characters
    char msg[MB_MAXCHAR + 1];
    msg[utf_char2bytes(char1, msg)] = NUL;
    semsg(_(e_digraph_must_be_just_two_characters_str), msg);
    return false;
  }
  case 2:
    // ESC not allowed
    emsg(_("E104: Escape not allowed in digraph"));
    return false;
  default:
    // Valid (result == 3)
    return true;
  }
}

/// Add the digraphs in the argument to the digraph table.
/// format: {c1}{c2} char {c1}{c2} char ...
///
/// @param str
void putdigraph(char *str)
{
  PutdigraphResult result = { 0, 0, 0 };
  if (rs_putdigraph(&str, &result) == 0) {
    // Handle errors based on error_code
    switch (result.error_code) {
    case 1:
      // Character validation error - use check_digraph_chars_valid for message
      check_digraph_chars_valid(result.char1, result.char2);
      break;
    case 2:
      // Number expected
      emsg(_(e_number_exp));
      break;
    default:
      break;
    }
  }
}

static void digraph_header(const char *msg)
  FUNC_ATTR_NONNULL_ALL
{
  if (msg_col > 0) {
    msg_putchar('\n');
  }
  msg_outtrans(msg, HLF_CM, false);
  msg_putchar('\n');
}

void listdigraphs(bool use_headers)
{
  result_T previous = 0;

  msg_ext_set_kind("list_cmd");
  msg_putchar('\n');

  const digr_T *dp = rs_get_digraphdefault();
  int dp_len = rs_get_digraphdefault_len();

  for (int i = 0; i < dp_len && !got_int; i++) {
    digr_T tmp;

    // May need to convert the result to 'encoding'.
    tmp.char1 = dp[i].char1;
    tmp.char2 = dp[i].char2;
    tmp.result = getexactdigraph(tmp.char1, tmp.char2, false);

    if (tmp.result != 0 && tmp.result != tmp.char2) {
      printdigraph(&tmp, use_headers ? &previous : NULL);
    }
    fast_breakcheck();
  }

  dp = (const digr_T *)user_digraphs.ga_data;
  for (int i = 0; i < user_digraphs.ga_len && !got_int; i++) {
    if (previous >= 0 && use_headers) {
      digraph_header(_("Custom"));
    }
    previous = -1;
    printdigraph(dp, NULL);
    fast_breakcheck();
    dp++;
  }
}

/// Context for digraph_getlist iteration callback.
typedef struct {
  list_T *list;
} DigraphGetlistCtx;

/// Callback for digraph_getlist iteration.
static int digraph_getlist_callback(uint8_t char1, uint8_t char2, int result, void *ctx)
{
  DigraphGetlistCtx *gctx = (DigraphGetlistCtx *)ctx;

  // Create a 2-element sublist ["{c1}{c2}", "{result}"]
  list_T *l2 = tv_list_alloc(2);
  tv_list_append_list(gctx->list, l2);

  // Append digraph characters
  char buf[30];
  buf[0] = (char)char1;
  buf[1] = (char)char2;
  buf[2] = NUL;
  tv_list_append_string(l2, buf, -1);

  // Append result as UTF-8
  char *p = buf;
  p += utf_char2bytes(result, p);
  *p = NUL;
  tv_list_append_string(l2, buf, -1);

  // Continue iteration if not interrupted
  return !got_int;
}

void digraph_getlist_common(bool list_all, typval_T *rettv)
{
  tv_list_alloc_ret(rettv, (int)((size_t)rs_get_digraphdefault_len() * sizeof(digr_T)) + user_digraphs.ga_len);

  DigraphGetlistCtx ctx = { .list = rettv->vval.v_list };

  if (list_all) {
    rs_digraph_iterate_default(digraph_getlist_callback, &ctx);
  }
  rs_digraph_iterate_user(digraph_getlist_callback, &ctx);
}

static struct dg_header_entry {
  int dg_start;
  const char *dg_header;
} header_table[] = {
  { 0xa1, N_("Latin supplement") },
  { 0x0386, N_("Greek and Coptic") },
  { 0x0401, N_("Cyrillic") },
  { 0x05d0, N_("Hebrew") },
  { 0x060c, N_("Arabic") },
  { 0x1e02, N_("Latin extended") },
  { 0x1f00, N_("Greek extended") },
  { 0x2002, N_("Punctuation") },
  { 0x2070, N_("Super- and subscripts") },
  { 0x20a4, N_("Currency") },
  { 0x2103, N_("Other") },
  { 0x2160, N_("Roman numbers") },
  { 0x2190, N_("Arrows") },
  { 0x2200, N_("Mathematical operators") },
  { 0x2302, N_("Technical") },
  { 0x2423, N_("Other") },
  { 0x2500, N_("Box drawing") },
  { 0x2580, N_("Block elements") },
  { 0x25a0, N_("Geometric shapes") },
  { 0x2605, N_("Symbols") },
  { 0x2713, N_("Dingbats") },
  { 0x3000, N_("CJK symbols and punctuation") },
  { 0x3041, N_("Hiragana") },
  { 0x30a1, N_("Katakana") },
  { 0x3105, N_("Bopomofo") },
  { 0x3220, N_("Other") },
  { 0xfffffff, NULL },
};

static void printdigraph(const digr_T *dp, result_T *previous)
  FUNC_ATTR_NONNULL_ARG(1)
{
  char buf[30];
  int list_width = 13;

  if (dp->result == 0) {
    return;
  }

  if (previous != NULL) {
    int header_idx = rs_digraph_get_header_index(*previous, dp->result);
    if (header_idx >= 0 && header_idx < (int)ARRAY_SIZE(header_table) - 1) {
      digraph_header(_(header_table[header_idx].dg_header));
    }
    *previous = dp->result;
  }
  if (msg_col > Columns - list_width) {
    msg_putchar('\n');
  }

  // Make msg_col a multiple of list_width by using spaces.
  if (msg_col % list_width != 0) {
    int spaces = (msg_col / list_width + 1) * list_width - msg_col;
    while (spaces--) {
      msg_putchar(' ');
    }
  }

  char *p = &buf[0];
  *p++ = (char)dp->char1;
  *p++ = (char)dp->char2;
  *p++ = ' ';
  *p = NUL;
  msg_outtrans(buf, 0, false);
  p = buf;

  // add a space to draw a composing char on
  if (utf_iscomposing_first(dp->result)) {
    *p++ = ' ';
  }
  p += utf_char2bytes(dp->result, p);

  *p = NUL;
  msg_outtrans(buf, HLF_8, false);
  p = buf;
  if (char2cells(dp->result) == 1) {
    *p++ = ' ';
  }
  assert(p >= buf);
  vim_snprintf(p, sizeof(buf) - (size_t)(p - buf), " %3d", dp->result);
  msg_outtrans(buf, 0, false);
}

/// Get the two digraph characters from a typval.
/// @return OK or FAIL.
static int get_digraph_chars(const typval_T *arg, int *char1, int *char2)
{
  char buf_chars[NUMBUFLEN];
  const char *chars = tv_get_string_buf_chk(arg, buf_chars);
  const char *p = chars;

  if (p != NULL) {
    if (*p != NUL) {
      *char1 = mb_cptr2char_adv(&p);
      if (*p != NUL) {
        *char2 = mb_cptr2char_adv(&p);
        if (*p == NUL) {
          if (check_digraph_chars_valid(*char1, *char2)) {
            return OK;
          }
          return FAIL;
        }
      }
    }
  }
  semsg(_(e_digraph_must_be_just_two_characters_str), chars);
  return FAIL;
}

static bool digraph_set_common(const typval_T *argchars, const typval_T *argdigraph)
{
  int char1, char2;
  if (get_digraph_chars(argchars, &char1, &char2) == FAIL) {
    return false;
  }

  char buf_digraph[NUMBUFLEN];
  const char *digraph = tv_get_string_buf_chk(argdigraph, buf_digraph);
  if (digraph == NULL) {
    return false;
  }
  const char *p = digraph;
  int n = mb_cptr2char_adv(&p);
  if (*p != NUL) {
    semsg(_(e_digraph_argument_must_be_one_character_str), digraph);
    return false;
  }

  registerdigraph(char1, char2, n);
  return true;
}

/// "digraph_get()" function
void f_digraph_get(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;  // Return empty string for failure
  const char *digraphs = tv_get_string_chk(&argvars[0]);

  if (digraphs == NULL) {
    return;
  }
  if (strlen(digraphs) != 2) {
    semsg(_(e_digraph_must_be_just_two_characters_str), digraphs);
    return;
  }
  int code = digraph_get(digraphs[0], digraphs[1], false);

  char buf[NUMBUFLEN];
  buf[utf_char2bytes(code, buf)] = NUL;
  rettv->vval.v_string = xstrdup(buf);
}

/// "digraph_getlist()" function
void f_digraph_getlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (tv_check_for_opt_bool_arg(argvars, 0) == FAIL) {
    return;
  }

  bool flag_list_all;

  if (argvars[0].v_type == VAR_UNKNOWN) {
    flag_list_all = false;
  } else {
    varnumber_T flag = tv_get_bool(&argvars[0]);
    flag_list_all = flag != 0;
  }

  digraph_getlist_common(flag_list_all, rettv);
}

/// "digraph_set()" function
void f_digraph_set(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_BOOL;
  rettv->vval.v_bool = kBoolVarFalse;

  if (!digraph_set_common(&argvars[0], &argvars[1])) {
    return;
  }

  rettv->vval.v_bool = kBoolVarTrue;
}

/// "digraph_setlist()" function
void f_digraph_setlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_BOOL;
  rettv->vval.v_bool = kBoolVarFalse;

  if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_digraph_setlist_argument_must_be_list_of_lists_with_two_items));
    return;
  }

  list_T *pl = argvars[0].vval.v_list;
  if (pl == NULL) {
    // Empty list always results in success.
    rettv->vval.v_bool = kBoolVarTrue;
    return;
  }

  TV_LIST_ITER_CONST(pl, pli, {
    if (TV_LIST_ITEM_TV(pli)->v_type != VAR_LIST) {
      emsg(_(e_digraph_setlist_argument_must_be_list_of_lists_with_two_items));
      return;
    }

    list_T *l = TV_LIST_ITEM_TV(pli)->vval.v_list;
    if (l == NULL || tv_list_len(l) != 2) {
      emsg(_(e_digraph_setlist_argument_must_be_list_of_lists_with_two_items));
      return;
    }

    if (!digraph_set_common(TV_LIST_ITEM_TV(tv_list_first(l)),
                            TV_LIST_ITEM_TV(TV_LIST_ITEM_NEXT(l, tv_list_first(l))))) {
      return;
    }
  });

  rettv->vval.v_bool = kBoolVarTrue;
}

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
