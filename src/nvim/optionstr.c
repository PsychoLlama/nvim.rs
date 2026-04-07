#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/win_config.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/cursor_shape.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/digraph.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_getln.h"
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/insexpand.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/os.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/shada.h"
#include "nvim/spell.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "nvim/winfloat.h"

#include "optionstr.c.generated.h"

// Rust FFI declarations (window wrappers removed)
extern int rs_global_stl_height(void);
extern int rs_get_shada_parameter(int type);

// Rust fold FFI declarations
extern void rs_foldUpdateAll(win_T *win);
extern void rs_newFoldLevel(void);

extern void rs_did_set_title(void);
extern int rs_valid_name(const char *val, const char *allowed);
extern int rs_get_fileformat(buf_T *buf);

// String option flag utilities (from Rust optionstr crate)
// did_set_str_generic and didset_string_options are implemented in Rust
extern const char *did_set_str_generic(optset_T *args);
extern void didset_string_options(void);

// Option type utilities

// Option scope utilities

// Set operation utilities

// Flag character validation
extern int rs_diffopt_changed(void);

// Number validation utilities

// Character validation

// Fillchars/listchars utilities
extern schar_T rs_get_encoded_char_adv(const char **p);

// Statusline format validation (symbol exported from Rust option crate)
extern const char *check_stl_option(char *s);

// Signcolumn validation (from Rust option crate)
typedef struct {
  int min_width;
  int max_width;
  int valid;
} SigncolumnResult;
extern SigncolumnResult rs_parse_signcolumn(const char *val);

// Option string flags parsing
typedef struct {
  bool ok;
  uint32_t flags;
} OptStringsFlagsResult;
extern OptStringsFlagsResult rs_opt_strings_flags(const char *val, const char **values, bool is_list);

// Flag list validation
typedef struct {
  bool ok;
  char invalid_char;
} FlagListValidateResult;

static const char e_illegal_character_after_chr[]
  = N_("E535: Illegal character after <%c>");
static const char e_wrong_number_of_characters_for_field_str[]
  = N_("E1511: Wrong number of characters for field \"%s\"");
static const char e_wrong_character_width_for_field_str[]
  = N_("E1512: Wrong character width for field \"%s\"");


// didset_string_options() is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// illegal_char is implemented in Rust (src/nvim-rs/optionstr/src/errors.rs)
// (declaration in optionstr.h)

static char *illegal_char_after_chr(char *errbuf, size_t errbuflen, int c)
{
  if (errbuf == NULL) {
    return "";
  }
  vim_snprintf(errbuf, errbuflen, _(e_illegal_character_after_chr), c);
  return errbuf;
}

/// Check string options in a buffer for NULL value.
void check_buf_options(buf_T *buf)
{
  check_string_option(&buf->b_p_bh);
  check_string_option(&buf->b_p_bt);
  check_string_option(&buf->b_p_fenc);
  check_string_option(&buf->b_p_ff);
  check_string_option(&buf->b_p_def);
  check_string_option(&buf->b_p_inc);
  check_string_option(&buf->b_p_inex);
  check_string_option(&buf->b_p_inde);
  check_string_option(&buf->b_p_indk);
  check_string_option(&buf->b_p_fp);
  check_string_option(&buf->b_p_fex);
  check_string_option(&buf->b_p_kp);
  check_string_option(&buf->b_p_mps);
  check_string_option(&buf->b_p_fo);
  check_string_option(&buf->b_p_flp);
  check_string_option(&buf->b_p_isk);
  check_string_option(&buf->b_p_com);
  check_string_option(&buf->b_p_cms);
  check_string_option(&buf->b_p_nf);
  check_string_option(&buf->b_p_qe);
  check_string_option(&buf->b_p_syn);
  check_string_option(&buf->b_s.b_syn_isk);
  check_string_option(&buf->b_s.b_p_spc);
  check_string_option(&buf->b_s.b_p_spf);
  check_string_option(&buf->b_s.b_p_spl);
  check_string_option(&buf->b_s.b_p_spo);
  check_string_option(&buf->b_p_sua);
  check_string_option(&buf->b_p_cink);
  check_string_option(&buf->b_p_cino);
  parse_cino(buf);
  check_string_option(&buf->b_p_lop);
  check_string_option(&buf->b_p_ft);
  check_string_option(&buf->b_p_cinw);
  check_string_option(&buf->b_p_cinsd);
  check_string_option(&buf->b_p_cot);
  check_string_option(&buf->b_p_cpt);
  check_string_option(&buf->b_p_cfu);
  check_string_option(&buf->b_p_ofu);
  check_string_option(&buf->b_p_keymap);
  check_string_option(&buf->b_p_gefm);
  check_string_option(&buf->b_p_gp);
  check_string_option(&buf->b_p_mp);
  check_string_option(&buf->b_p_efm);
  check_string_option(&buf->b_p_ep);
  check_string_option(&buf->b_p_path);
  check_string_option(&buf->b_p_tags);
  check_string_option(&buf->b_p_ffu);
  check_string_option(&buf->b_p_tfu);
  check_string_option(&buf->b_p_tc);
  check_string_option(&buf->b_p_dict);
  check_string_option(&buf->b_p_dia);
  check_string_option(&buf->b_p_tsr);
  check_string_option(&buf->b_p_tsrfu);
  check_string_option(&buf->b_p_lw);
  check_string_option(&buf->b_p_bkc);
  check_string_option(&buf->b_p_menc);
  check_string_option(&buf->b_p_vsts);
  check_string_option(&buf->b_p_vts);
}

/// Free the string allocated for an option.
/// Checks for the string being empty_string_option. This may happen if we're out of memory,
/// xstrdup() returned NULL, which was replaced by empty_string_option by check_options().
// free_string_option, clear_string_option, check_string_option implemented in Rust
// (src/nvim-rs/optionstr/src/lib.rs)


/// Handle setting 'signcolumn' for value 'val'. Store minimum and maximum width.
///
/// @param wcl  when NULL: use "wp->w_p_scl"
/// @param wp   when NULL: only parse "scl"
///
/// @return OK when the value is valid, FAIL otherwise
int check_signcolumn(char *scl, win_T *wp)
{
  char *val = scl != NULL ? scl : (wp != NULL ? wp->w_p_scl : empty_string_option);
  if (*val == NUL) {
    return FAIL;
  }
  SigncolumnResult r = rs_parse_signcolumn(val);
  if (!r.valid) {
    return FAIL;
  }
  if (wp == NULL) {
    return OK;
  }
  // "number" mode only applies when 'number' or 'relativenumber' is set
  if (r.min_width == SCL_NUM && !(wp->w_p_nu || wp->w_p_rnu)) {
    wp->w_minscwidth = 0;
    wp->w_maxscwidth = 1;
  } else {
    wp->w_minscwidth = r.min_width;
    wp->w_maxscwidth = r.max_width;
  }
  int scwidth = wp->w_minscwidth <= 0 ? 0 : MIN(wp->w_maxscwidth, wp->w_scwidth);
  wp->w_scwidth = MAX(wp->w_minscwidth, scwidth);
  return OK;
}




// opt_values, check_str_opt, did_set_str_generic are now implemented in Rust
// (src/nvim-rs/optionstr/src/didset.rs)


// expand_set_opt_string, expand_set_opt_callback, expand_set_opt_generic, and their
// static state are implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

/// The 'ambiwidth' option is changed.

// did_set_background is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)




// did_set_buftype is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

/// The global 'listchars' or 'fillchars' option is changed.
static const char *did_set_global_chars_option(win_T *win, char *val, CharsOption what,
                                               int opt_flags, char *errbuf, size_t errbuflen)
{
  const char *errmsg = NULL;
  char **local_ptr = (what == kListchars) ? &win->w_p_lcs : &win->w_p_fcs;

  // only apply the global value to "win" when it does not have a
  // local value
  errmsg = set_chars_option(win, val, what,
                            **local_ptr == NUL || !(opt_flags & OPT_GLOBAL),
                            errbuf, errbuflen);
  if (errmsg != NULL) {
    return errmsg;
  }

  // If the current window is set to use the global
  // 'listchars'/'fillchars' value, clear the window-local value.
  if (!(opt_flags & OPT_GLOBAL)) {
    clear_string_option(local_ptr);
  }

  FOR_ALL_TAB_WINDOWS(tp, wp) {
    // If the current window has a local value need to apply it
    // again, it was changed when setting the global value.
    // If no error was returned above, we don't expect an error
    // here, so ignore the return value.
    char *opt = (what == kListchars) ? wp->w_p_lcs : wp->w_p_fcs;
    if (*opt == NUL) {
      set_chars_option(wp, opt, what, true, errbuf, errbuflen);
    }
  }

  redraw_all_later(UPD_NOT_VALID);

  return NULL;
}

/// The 'fillchars' option or the 'listchars' option is changed.
const char *did_set_chars_option(optset_T *args)
{
  win_T *win = (win_T *)args->os_win;
  char **varp = (char **)args->os_varp;
  const char *errmsg = NULL;

  if (varp == &p_lcs) {      // global 'listchars'
    errmsg = did_set_global_chars_option(win, *varp, kListchars, args->os_flags,
                                         args->os_errbuf, args->os_errbuflen);
  } else if (varp == &p_fcs) {  // global 'fillchars'
    errmsg = did_set_global_chars_option(win, *varp, kFillchars, args->os_flags,
                                         args->os_errbuf, args->os_errbuflen);
  } else if (varp == &win->w_p_lcs) {  // local 'listchars'
    errmsg = set_chars_option(win, *varp, kListchars, true,
                              args->os_errbuf, args->os_errbuflen);
  } else if (varp == &win->w_p_fcs) {  // local 'fillchars'
    errmsg = set_chars_option(win, *varp, kFillchars, true,
                              args->os_errbuf, args->os_errbuflen);
  }

  return errmsg;
}

// expand_set_chars_option is now implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

// did_set_colorcolumn is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_complete is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_completeitemalign is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_completeopt is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

#ifdef BACKSLASH_IN_FILENAME
/// The 'completeslash' option is changed.
const char *did_set_completeslash(optset_T *args)
{
  buf_T *buf = (buf_T *)args->os_buf;
  if (!rs_opt_strings_flags(p_csl, opt_csl_values, false).ok
      || !rs_opt_strings_flags(buf->b_p_csl, opt_csl_values, false).ok) {
    return e_invarg;
  }
  return NULL;
}
#endif

// expand_set_concealcursor, expand_set_cpoptions, and expand_set_diffopt moved to Rust

// did_set_encoding is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// expand_set_encoding, expand_set_eventignore, get_eventignore_name, and
// expand_eiw static are implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

// did_set_fileformat is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// get_fileformat_name is implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)



// expand_set_formatoptions moved to Rust





// did_set_iskeyword is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_isopt is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_keymap is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)


// expand_set_mouse moved to Rust





// did_set_shada is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// expand_set_shortmess moved to Rust

/// The 'showbreak' option is changed.

// did_set_signcolumn is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)



// did_set_statustabline_rulerformat is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)


// did_set_tagcase is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_titleiconstring is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)


// did_set_varsofttabstop is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_vartabstop is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_virtualedit is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// expand_set_whichwrap moved to Rust


bool parse_border_opt(char *border_opt)
{
  WinConfig fconfig = WIN_CONFIG_INIT;
  Error err = ERROR_INIT;
  bool result = true;
  if (!parse_winborder(&fconfig, border_opt, &err)) {
    result = false;
  }
  api_clear_error(&err);
  return result;
}



// expand_set_winhighlight moved to Rust

// check_ff_value is implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

static const char e_conflicts_with_value_of_listchars[]
  = N_("E834: Conflicts with value of 'listchars'");
static const char e_conflicts_with_value_of_fillchars[]
  = N_("E835: Conflicts with value of 'fillchars'");


struct chars_tab {
  schar_T *cp;           ///< char value
  String name;           ///< char id
  const char *def;       ///< default value
  const char *fallback;  ///< default value when "def" isn't single-width
};

#define CHARSTAB_ENTRY(cp, name, def, fallback) \
  { (cp), { name, STRLEN_LITERAL(name) }, def, fallback }

static fcs_chars_T fcs_chars;
static const struct chars_tab fcs_tab[] = {
  CHARSTAB_ENTRY(&fcs_chars.stl,        "stl",       " ",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.stlnc,      "stlnc",     " ",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.wbr,        "wbr",       " ",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.horiz,      "horiz",     "─",  "-"),
  CHARSTAB_ENTRY(&fcs_chars.horizup,    "horizup",   "┴",  "-"),
  CHARSTAB_ENTRY(&fcs_chars.horizdown,  "horizdown", "┬",  "-"),
  CHARSTAB_ENTRY(&fcs_chars.vert,       "vert",      "│",  "|"),
  CHARSTAB_ENTRY(&fcs_chars.vertleft,   "vertleft",  "┤",  "|"),
  CHARSTAB_ENTRY(&fcs_chars.vertright,  "vertright", "├",  "|"),
  CHARSTAB_ENTRY(&fcs_chars.verthoriz,  "verthoriz", "┼",  "+"),
  CHARSTAB_ENTRY(&fcs_chars.fold,       "fold",      "·",  "-"),
  CHARSTAB_ENTRY(&fcs_chars.foldopen,   "foldopen",  "-",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.foldclosed, "foldclose", "+",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.foldsep,    "foldsep",   "│",  "|"),
  CHARSTAB_ENTRY(&fcs_chars.foldinner,  "foldinner", NULL, NULL),
  CHARSTAB_ENTRY(&fcs_chars.diff,       "diff",      "-",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.msgsep,     "msgsep",    " ",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.eob,        "eob",       "~",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.lastline,   "lastline",  "@",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.trunc,      "trunc",     ">",  NULL),
  CHARSTAB_ENTRY(&fcs_chars.truncrl,    "truncrl",   "<",  NULL),
};

static lcs_chars_T lcs_chars;
static const struct chars_tab lcs_tab[] = {
  CHARSTAB_ENTRY(&lcs_chars.eol,     "eol",            NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.ext,     "extends",        NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.nbsp,    "nbsp",           NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.prec,    "precedes",       NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.space,   "space",          NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.tab2,    "tab",            NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.lead,    "lead",           NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.trail,   "trail",          NULL, NULL),
  CHARSTAB_ENTRY(&lcs_chars.conceal, "conceal",        NULL, NULL),
  CHARSTAB_ENTRY(NULL,               "multispace",     NULL, NULL),
  CHARSTAB_ENTRY(NULL,               "leadmultispace", NULL, NULL),
};

#undef CHARSTAB_ENTRY

static char *field_value_err(char *errbuf, size_t errbuflen, const char *fmt, const char *field)
{
  if (errbuf == NULL) {
    return "";
  }
  vim_snprintf(errbuf, errbuflen, _(fmt), field);
  return errbuf;
}

/// Handle setting 'listchars' or 'fillchars'.
/// Assume monocell characters
///
/// @param value      points to either the global or the window-local value.
/// @param what       kListchars or kFillchars
/// @param apply      if false, do not store the flags, only check for errors.
/// @param errbuf     buffer for error message, can be NULL if it won't be used.
/// @param errbuflen  size of error buffer.
///
/// @return error message, NULL if it's OK.
const char *set_chars_option(win_T *wp, const char *value, CharsOption what, bool apply,
                             char *errbuf, size_t errbuflen)
{
  const char *last_multispace = NULL;   // Last occurrence of "multispace:"
  const char *last_lmultispace = NULL;  // Last occurrence of "leadmultispace:"
  int multispace_len = 0;           // Length of lcs-multispace string
  int lead_multispace_len = 0;      // Length of lcs-leadmultispace string

  const struct chars_tab *tab;
  int entries;
  if (what == kListchars) {
    tab = lcs_tab;
    entries = ARRAY_SIZE(lcs_tab);
    if (wp->w_p_lcs[0] == NUL) {
      value = p_lcs;  // local value is empty, use the global value
    }
  } else {
    tab = fcs_tab;
    entries = ARRAY_SIZE(fcs_tab);
    if (wp->w_p_fcs[0] == NUL) {
      value = p_fcs;  // local value is empty, use the global value
    }
  }

  // first round: check for valid value, second round: assign values
  for (int round = 0; round <= (apply ? 1 : 0); round++) {
    if (round > 0) {
      // After checking that the value is valid: set defaults
      for (int i = 0; i < entries; i++) {
        if (tab[i].cp != NULL) {
          // XXX: Characters taking 2 columns is forbidden (TUI limitation?).
          // Set old defaults in this case.
          *(tab[i].cp) = schar_from_str((tab[i].def && ptr2cells(tab[i].def) == 1)
                                        ? tab[i].def : tab[i].fallback);
        }
      }

      if (what == kListchars) {
        lcs_chars.tab1 = NUL;
        lcs_chars.tab3 = NUL;

        if (multispace_len > 0) {
          lcs_chars.multispace = xmalloc(((size_t)multispace_len + 1) * sizeof(schar_T));
          lcs_chars.multispace[multispace_len] = NUL;
        } else {
          lcs_chars.multispace = NULL;
        }

        if (lead_multispace_len > 0) {
          lcs_chars.leadmultispace = xmalloc(((size_t)lead_multispace_len + 1) * sizeof(schar_T));
          lcs_chars.leadmultispace[lead_multispace_len] = NUL;
        } else {
          lcs_chars.leadmultispace = NULL;
        }
      }
    }

    const char *p = value;
    while (*p) {
      int i;
      for (i = 0; i < entries; i++) {
        if (!(strncmp(p, tab[i].name.data,
                      tab[i].name.size) == 0 && p[tab[i].name.size] == ':')) {
          continue;
        }

        const char *s = p + tab[i].name.size + 1;

        if (what == kListchars && strcmp(tab[i].name.data, "multispace") == 0) {
          if (round == 0) {
            // Get length of lcs-multispace string in the first round
            last_multispace = p;
            multispace_len = 0;
            while (*s != NUL && *s != ',') {
              schar_T c1 = rs_get_encoded_char_adv(&s);
              if (c1 == 0) {
                return field_value_err(errbuf, errbuflen,
                                       e_wrong_character_width_for_field_str,
                                       tab[i].name.data);
              }
              multispace_len++;
            }
            if (multispace_len == 0) {
              // lcs-multispace cannot be an empty string
              return field_value_err(errbuf, errbuflen,
                                     e_wrong_number_of_characters_for_field_str,
                                     tab[i].name.data);
            }
          } else {
            int multispace_pos = 0;
            while (*s != NUL && *s != ',') {
              schar_T c1 = rs_get_encoded_char_adv(&s);
              if (p == last_multispace) {
                lcs_chars.multispace[multispace_pos++] = c1;
              }
            }
          }
          p = s;
          break;
        }

        if (what == kListchars && strcmp(tab[i].name.data, "leadmultispace") == 0) {
          if (round == 0) {
            // Get length of lcs-leadmultispace string in first round
            last_lmultispace = p;
            lead_multispace_len = 0;
            while (*s != NUL && *s != ',') {
              schar_T c1 = rs_get_encoded_char_adv(&s);
              if (c1 == 0) {
                return field_value_err(errbuf, errbuflen,
                                       e_wrong_character_width_for_field_str,
                                       tab[i].name.data);
              }
              lead_multispace_len++;
            }
            if (lead_multispace_len == 0) {
              // lcs-leadmultispace cannot be an empty string
              return field_value_err(errbuf, errbuflen,
                                     e_wrong_number_of_characters_for_field_str,
                                     tab[i].name.data);
            }
          } else {
            int multispace_pos = 0;
            while (*s != NUL && *s != ',') {
              schar_T c1 = rs_get_encoded_char_adv(&s);
              if (p == last_lmultispace) {
                lcs_chars.leadmultispace[multispace_pos++] = c1;
              }
            }
          }
          p = s;
          break;
        }

        if (*s == NUL) {
          return field_value_err(errbuf, errbuflen,
                                 e_wrong_number_of_characters_for_field_str,
                                 tab[i].name.data);
        }
        schar_T c1 = rs_get_encoded_char_adv(&s);
        if (c1 == 0) {
          return field_value_err(errbuf, errbuflen,
                                 e_wrong_character_width_for_field_str,
                                 tab[i].name.data);
        }
        schar_T c2 = 0;
        schar_T c3 = 0;
        if (tab[i].cp == &lcs_chars.tab2) {
          if (*s == NUL) {
            return field_value_err(errbuf, errbuflen,
                                   e_wrong_number_of_characters_for_field_str,
                                   tab[i].name.data);
          }
          c2 = rs_get_encoded_char_adv(&s);
          if (c2 == 0) {
            return field_value_err(errbuf, errbuflen,
                                   e_wrong_character_width_for_field_str,
                                   tab[i].name.data);
          }
          if (!(*s == ',' || *s == NUL)) {
            c3 = rs_get_encoded_char_adv(&s);
            if (c3 == 0) {
              return field_value_err(errbuf, errbuflen,
                                     e_wrong_character_width_for_field_str,
                                     tab[i].name.data);
            }
          }
        }

        if (*s == ',' || *s == NUL) {
          if (round > 0) {
            if (tab[i].cp == &lcs_chars.tab2) {
              lcs_chars.tab1 = c1;
              lcs_chars.tab2 = c2;
              lcs_chars.tab3 = c3;
            } else if (tab[i].cp != NULL) {
              *(tab[i].cp) = c1;
            }
          }
          p = s;
          break;
        } else {
          return field_value_err(errbuf, errbuflen,
                                 e_wrong_number_of_characters_for_field_str,
                                 tab[i].name.data);
        }
      }

      if (i == entries) {
        return e_invarg;
      }

      if (*p == ',') {
        p++;
      }
    }
  }

  if (apply) {
    if (what == kListchars) {
      xfree(wp->w_p_lcs_chars.multispace);
      xfree(wp->w_p_lcs_chars.leadmultispace);
      wp->w_p_lcs_chars = lcs_chars;
    } else {
      wp->w_p_fcs_chars = fcs_chars;
    }
  }

  return NULL;          // no error
}


/// Check all global and local values of 'listchars' and 'fillchars'.
/// May set different defaults in case character widths change.
///
/// @return  an untranslated error message if any of them is invalid, NULL otherwise.
const char *check_chars_options(void)
{
  if (set_chars_option(curwin, p_lcs, kListchars, false, NULL, 0) != NULL) {
    return e_conflicts_with_value_of_listchars;
  }
  if (set_chars_option(curwin, p_fcs, kFillchars, false, NULL, 0) != NULL) {
    return e_conflicts_with_value_of_fillchars;
  }
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (set_chars_option(wp, wp->w_p_lcs, kListchars, true, NULL, 0) != NULL) {
      return e_conflicts_with_value_of_listchars;
    }
    if (set_chars_option(wp, wp->w_p_fcs, kFillchars, true, NULL, 0) != NULL) {
      return e_conflicts_with_value_of_fillchars;
    }
  }
  return NULL;
}
