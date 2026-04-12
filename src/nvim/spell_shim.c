// spell_shim.c: C accessor wrappers for the Rust spell crate.
//
// These functions provide access to C internals (DecorState, syntax, window
// fields) that cannot be accessed directly from Rust FFI.
// Also contains spell_load_lang, spell_load_cb, int_wordlist_spl which are
// kept in C due to complex C-only dependencies (do_in_runtimepath, autocmds).

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/runtime.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/syntax_bridge.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

#include "spell_shim.c.generated.h"

// =============================================================================
// Decoration state helpers for spell navigation (rs_spell_move_to)
// =============================================================================

/// Return sizeof(DecorState) so Rust can allocate enough space.
size_t nvim_spell_decor_state_size(void)
{
  return sizeof(DecorState);
}

/// Reset decor_state for spell navigation. Saves the current decor_state into
/// the provided buffer and initializes a fresh decor_state for spell nav.
/// The saved state must be restored with nvim_spell_restore_decor_state().
///
/// @param wp  The window being navigated.
/// @param saved_out  Caller-allocated storage (sizeof(DecorState)) for saving.
void nvim_spell_nav_start(win_T *wp, void *saved_out)
{
  *(DecorState *)saved_out = decor_state;
  decor_state = (DecorState){ 0 };
  decor_redraw_reset(wp, &decor_state);
}

/// Free the current (temporary) decor_state and restore the saved one.
///
/// @param saved  Pointer to the previously saved DecorState.
void nvim_spell_restore_decor_state(void *saved)
{
  decor_state_free(&decor_state);
  decor_state = *(DecorState *)saved;
}

/// Check decoration spell state at a column.
///
/// Returns: 1 (kTrue), 0 (kFalse), or -1 (kNone).
int nvim_spell_nav_decor_col(win_T *wp, int lnum, int *decor_lnum, int col)
{
  if (*decor_lnum != lnum) {
    decor_providers_invoke_spell(wp, lnum - 1, col, lnum - 1, -1);
    decor_redraw_line(wp, lnum - 1, &decor_state);
    *decor_lnum = lnum;
  }
  decor_redraw_col(wp, col, 0, false, &decor_state);
  switch (decor_state.spell) {
  case kTrue:  return 1;
  case kFalse: return 0;
  default:     return -1;
  }
}

// =============================================================================
// Syntax helpers for spell navigation
// =============================================================================

/// Check if syntax is present in the window.
bool nvim_spell_syntax_present(win_T *wp)
{
  return syntax_present(wp);
}

/// Check if syntax allows spell checking at a position.
bool nvim_spell_can_syn_spell(win_T *wp, int lnum, int col)
{
  bool can_spell;
  syn_get_id(wp, lnum, col, false, &can_spell, false);
  return can_spell;
}

// =============================================================================
// Misc accessors for spell navigation
// =============================================================================

/// Get the number of whitespace columns at the start of a line.
int nvim_spell_getwhitecols(const char *p)
{
  return (int)getwhitecols(p);
}

/// Get the line count of a window's buffer.
int nvim_spell_win_ml_line_count(win_T *wp)
{
  return wp->w_buffer->b_ml.ml_line_count;
}

/// Get line content from a window's buffer.
char *nvim_spell_ml_get_buf_win(win_T *wp, int lnum)
{
  return ml_get_buf(wp->w_buffer, (linenr_T)lnum);
}

/// Get line length from a window's buffer.
int nvim_spell_ml_get_buf_len_win(win_T *wp, int lnum)
{
  return ml_get_buf_len(wp->w_buffer, (linenr_T)lnum);
}

/// Check if the 'noplainbuffer' spelloptions flag is set.
bool nvim_spell_win_noplainbuffer(win_T *wp)
{
  return (wp->w_s->b_p_spo_flags & kOptSpoFlagNoplainbuffer) != 0;
}

/// Give the "search hit TOP, continuing at BOTTOM" or vice-versa warning.
/// @param forward  true => forward search (show "TOP...BOTTOM"), false => backward.
void nvim_spell_give_wrap_warning(bool forward)
{
  give_warning(_(forward ? bot_top_msg : top_bot_msg), true);
}

// =============================================================================
// Accessors for parse_spelllang (Rust implementation in lang.rs)
// =============================================================================

/// Emit "Warning: region %s not supported" message.
void nvim_spell_warn_region(const char *region)
{
  smsg(0, _("Warning: region %s not supported"), region);
}

/// Get curwin->w_s->b_p_spf (spellfile option string).
const char *nvim_spell_get_b_p_spf(void)
{
  return curwin->w_s->b_p_spf;
}

/// Set wp->w_s->b_cjk.
void nvim_win_set_b_cjk(win_T *wp, int val)
{
  wp->w_s->b_cjk = val;
}

/// Set wp->w_s->b_langp from the provided garray (transfer ownership).
void nvim_win_set_b_langp(win_T *wp, garray_T ga)
{
  ga_clear(&wp->w_s->b_langp);
  wp->w_s->b_langp = ga;
}

/// Append a new langp_T entry via pointer to a garray.
/// Returns pointer to the newly appended entry.
langp_T *nvim_spell_ga_append_langp(garray_T *ga)
{
  return GA_APPEND_VIA_PTR(langp_T, ga);
}

/// Initialize a bufref to the given buffer.
void nvim_spell_set_bufref(bufref_T *bufref, buf_T *buf)
{
  set_bufref(bufref, buf);
}

/// Check if a bufref is still valid.
bool nvim_spell_bufref_valid(bufref_T *bufref)
{
  return bufref_valid(bufref);
}

/// Wrapper for copy_option_part with a pointer-to-pointer interface.
/// Advances *pp past the copied part. Returns the length of the copied part.
size_t nvim_spell_copy_option_part(char **pp, char *buf, size_t maxlen, const char *sep_chars)
{
  return copy_option_part(pp, buf, maxlen, (char *)sep_chars);
}

/// Call path_full_compare(s1, s2, false, true). Returns 1 if equal files.
int nvim_spell_path_full_compare(const char *s1, const char *s2)
{
  return (int)path_full_compare((char *)s1, (char *)s2, false, true);
}

/// Call path_fnamecmp(s1, s2).
int nvim_spell_path_fnamecmp(const char *s1, const char *s2)
{
  return path_fnamecmp(s1, s2);
}

/// Return path_tail(fname).
const char *nvim_spell_path_tail(const char *fname)
{
  return path_tail(fname);
}

/// Case-insensitive string compare (STRICMP).
int nvim_spell_stricmp(const char *a, const char *b)
{
  return STRICMP(a, b);
}

/// Call redraw_later(wp, UPD_NOT_VALID).
void nvim_spell_redraw_later(win_T *wp)
{
  redraw_later(wp, UPD_NOT_VALID);
}

/// Returns true if c is an ASCII alphabetic character.
bool nvim_spell_ascii_isalpha(int c)
{
  return ASCII_ISALPHA(c);
}

/// Returns the starting global (non-zero while Nvim is starting).
int nvim_spell_get_starting(void)
{
  return starting;
}

// Structure used for the cookie argument of do_in_runtimepath().
// (Moved from spell.c)
typedef struct {
  char sl_lang[MAXWLEN + 1];            // language name
  slang_T *sl_slang;                    // resulting slang_T struct
  int sl_nobreak;                       // NOBREAK language found
} spelload_T;

/// Get the name of the .spl file for the internal wordlist into fname[MAXPATHL].
void nvim_spell_int_wordlist_spl(char *fname)
{
  vim_snprintf(fname, MAXPATHL, SPL_FNAME_TMPL, int_wordlist, spell_enc());
}

// Forward declaration (defined below)
static bool spell_load_cb_impl(int num_fnames, char **fnames, bool all, void *cookie);

/// Load word list(s) for "lang" from Vim spell file(s).
/// "lang" must be the language without the region: e.g., "en".
/// This is the C implementation kept here due to do_in_runtimepath + autocmds.
void nvim_spell_load_lang(char *lang)
{
  char fname_enc[85];
  int r;
  spelload_T sl;

  // Copy the language name to pass it to spell_load_cb_impl() as a cookie.
  STRCPY(sl.sl_lang, lang);
  sl.sl_slang = NULL;
  sl.sl_nobreak = false;

  // Disallow deleting the current buffer.
  curbuf->b_locked++;

  for (int round = 1; round <= 2; round++) {
    vim_snprintf(fname_enc, sizeof(fname_enc) - 5,
                 "spell/%s.%s.spl", lang, spell_enc());
    r = do_in_runtimepath(fname_enc, 0, spell_load_cb_impl, &sl);

    if (r == FAIL && *sl.sl_lang != NUL) {
      vim_snprintf(fname_enc, sizeof(fname_enc) - 5,
                   "spell/%s.ascii.spl", lang);
      r = do_in_runtimepath(fname_enc, 0, spell_load_cb_impl, &sl);

      if (r == FAIL && *sl.sl_lang != NUL && round == 1
          && apply_autocmds(EVENT_SPELLFILEMISSING, lang,
                            curbuf->b_fname, false, curbuf)) {
        continue;
      }
      break;
    }
    break;
  }

  if (r == FAIL) {
    if (starting) {
      char autocmd_buf[512] = { 0 };
      snprintf(autocmd_buf, sizeof(autocmd_buf),
               "autocmd VimEnter * call v:lua.require'nvim.spellfile'.get('%s')|set spell",
               lang);
      do_cmdline_cmd(autocmd_buf);
    } else {
      smsg(0, _("Warning: Cannot find word list \"%s.%s.spl\" or \"%s.ascii.spl\""),
           lang, spell_enc(), lang);
    }
  } else if (sl.sl_slang != NULL) {
    STRCPY(fname_enc + strlen(fname_enc) - 3, "add.spl");
    do_in_runtimepath(fname_enc, DIP_ALL, spell_load_cb_impl, &sl);
  }

  curbuf->b_locked--;
}

static bool spell_load_cb_impl(int num_fnames, char **fnames, bool all, void *cookie)
{
  spelload_T *slp = (spelload_T *)cookie;
  for (int i = 0; i < num_fnames; i++) {
    slang_T *slang = spell_load_file(fnames[i], slp->sl_lang, NULL, false);

    if (slang == NULL) {
      continue;
    }

    if (slp->sl_nobreak && slang->sl_add) {
      slang->sl_nobreak = true;
    } else if (slang->sl_nobreak) {
      slp->sl_nobreak = true;
    }

    slp->sl_slang = slang;

    if (!all) {
      break;
    }
  }

  return num_fnames > 0;
}

// =============================================================================
// Accessors for midword helpers (use_midword / clear_midword in lang.rs)
// =============================================================================

/// Get wp->w_s->b_spell_ismw[c].
bool nvim_spell_win_get_ismw(win_T *wp, int c)
{
  return wp->w_s->b_spell_ismw[c];
}

/// Set wp->w_s->b_spell_ismw[c] = val.
void nvim_spell_win_set_ismw(win_T *wp, int c, bool val)
{
  wp->w_s->b_spell_ismw[c] = val;
}

/// Get wp->w_s->b_spell_ismw_mb (may be NULL).
const char *nvim_spell_win_get_ismw_mb(win_T *wp)
{
  return wp->w_s->b_spell_ismw_mb;
}

/// Set wp->w_s->b_spell_ismw_mb to new_val (takes ownership, frees old).
void nvim_spell_win_set_ismw_mb(win_T *wp, char *new_val)
{
  xfree(wp->w_s->b_spell_ismw_mb);
  wp->w_s->b_spell_ismw_mb = new_val;
}

/// Clear wp->w_s->b_spell_ismw[] and free+NULL b_spell_ismw_mb.
void nvim_spell_win_clear_ismw(win_T *wp)
{
  CLEAR_FIELD(wp->w_s->b_spell_ismw);
  XFREE_CLEAR(wp->w_s->b_spell_ismw_mb);
}

// =============================================================================
// exarg_T accessors for ex_mkspell / ex_spell (now in Rust)
// =============================================================================

#include "nvim/ex_cmds_defs.h"

/// Get eap->arg for spell commands.
char *nvim_spell_eap_get_arg(const exarg_T *eap) { return eap->arg; }

/// Get eap->forceit for spell commands.
bool nvim_spell_eap_get_forceit(const exarg_T *eap) { return eap->forceit; }

/// Get eap->cmdidx as int for spell commands.
int nvim_spell_eap_get_cmdidx(const exarg_T *eap) { return (int)eap->cmdidx; }

/// Get eap->line2 as int for spell commands.
int nvim_spell_eap_get_line2(const exarg_T *eap) { return (int)eap->line2; }

#include "nvim/spell_defs.h"

/// Map eap->cmdidx to SpellAddType for ex_spell (SPELL_ADD_BAD, _RARE, or _GOOD).
int nvim_spell_eap_get_add_type(const exarg_T *eap)
{
  if (eap->cmdidx == CMD_spellwrong) {
    return SPELL_ADD_BAD;
  }
  if (eap->cmdidx == CMD_spellrare) {
    return SPELL_ADD_RARE;
  }
  return SPELL_ADD_GOOD;
}

/// Return true if eap->cmdidx is CMD_spellundo.
bool nvim_spell_eap_is_undo(const exarg_T *eap)
{
  return eap->cmdidx == CMD_spellundo;
}

// =============================================================================
// spell_add_word / init_spellfile accessors (Phase 3)
// =============================================================================

// These are already included above, but listed here for clarity:
// globals.h, garray.h, buffer_defs.h are already included at top of file.

/// Get int_wordlist global (temporary file for internal word list).
char *nvim_get_int_wordlist(void) { return int_wordlist; }

/// Set int_wordlist global.
void nvim_set_int_wordlist(char *val) { int_wordlist = val; }

/// Get curwin->w_s->b_p_spf (spell file option).
char *nvim_curwin_get_ws_b_p_spf(void) { return curwin->w_s->b_p_spf; }

/// Get curwin->w_s->b_p_spl (spell language option).
char *nvim_curwin_get_ws_b_p_spl(void) { return curwin->w_s->b_p_spl; }

/// Return true if curwin->w_s->b_langp is empty (GA_EMPTY).
bool nvim_curwin_ws_b_langp_is_empty(void) { return GA_EMPTY(&curwin->w_s->b_langp); }

/// Get curwin->w_s->b_langp as GArray pointer for langp_entry access.
const garray_T *nvim_curwin_get_ws_b_langp(void) { return &curwin->w_s->b_langp; }

/// Get curbuf->b_s.b_p_spl (buffer spell language option).
char *nvim_curbuf_get_b_s_b_p_spl(void) { return curbuf->b_s.b_p_spl; }

/// Get pointer to NameBuff global.
char *nvim_get_NameBuff(void) { return NameBuff; }

// nvim_buf_get_b_orig_mode() already defined in buffer_shim.c (Phase 3 dedup).

/// Check buf->b_ml.ml_mfp == NULL.
bool nvim_buf_ml_mfp_is_null(buf_T *buf) { return buf->b_ml.ml_mfp == NULL; }
