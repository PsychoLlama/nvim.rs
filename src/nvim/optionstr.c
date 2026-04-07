#include <stddef.h>

#include "nvim/buffer_defs.h"
#include "nvim/indent_c.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/types_defs.h"

#ifdef BACKSLASH_IN_FILENAME
#  include "nvim/errors.h"
#endif

#include "optionstr.c.generated.h"

// Rust FFI declarations

// Option string flags parsing
typedef struct {
  bool ok;
  uint32_t flags;
} OptStringsFlagsResult;
extern OptStringsFlagsResult rs_opt_strings_flags(const char *val, const char **values, bool is_list);

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

// free_string_option, clear_string_option, check_string_option implemented in Rust
// (src/nvim-rs/optionstr/src/lib.rs)

// illegal_char is implemented in Rust (src/nvim-rs/optionstr/src/errors.rs)
// (declaration in optionstr.h)

// didset_string_options() is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)

// did_set_background is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_buftype is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_chars_option is implemented in Rust (src/nvim-rs/optionstr/src/chars.rs)
// did_set_colorcolumn is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_complete is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_completeitemalign is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_completeopt is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_encoding is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_fileformat is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_iskeyword is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_isopt is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_keymap is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_shada is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_signcolumn is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_statustabline_rulerformat is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_str_generic is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_tagcase is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_titleiconstring is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_varsofttabstop is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_vartabstop is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// did_set_virtualedit is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// set_chars_option is implemented in Rust (src/nvim-rs/optionstr/src/chars.rs)
// check_chars_options is implemented in Rust (src/nvim-rs/optionstr/src/chars.rs)
// check_signcolumn is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// parse_border_opt is implemented in Rust (src/nvim-rs/optionstr/src/didset.rs)
// expand_set_* functions are implemented in Rust (src/nvim-rs/optionstr/src/expand.rs)

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
