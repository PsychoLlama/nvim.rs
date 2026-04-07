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


// check_signcolumn is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)




// opt_values, check_str_opt, did_set_str_generic are now implemented in Rust
// (src/nvim-rs/optionstr/src/didset.rs)


// expand_set_opt_string, expand_set_opt_callback, expand_set_opt_generic, and their
// static state are implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

/// The 'ambiwidth' option is changed.

// did_set_background is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)




// did_set_buftype is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_global_chars_option and did_set_chars_option are now implemented in Rust
// (src/nvim-rs/optionstr/src/didset.rs)

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


// parse_border_opt is now implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)



// expand_set_winhighlight moved to Rust

// check_ff_value is implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

// e_conflicts_with_value_of_listchars/fillchars, fcs_tab, lcs_tab struct,
// CHARSTAB_ENTRY macro, fcs_chars/lcs_chars statics, field_value_err: all moved to Rust chars.rs

// set_chars_option and check_chars_options are now implemented in Rust
// (src/nvim-rs/optionstr/src/chars.rs)

