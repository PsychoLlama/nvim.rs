// spell.c: code for spell checking
//
// See spellfile.c for the Vim spell file format.
//
// The spell checking mechanism uses a tree (aka trie).  Each node in the tree
// has a list of bytes that can appear (siblings).  For each byte there is a
// pointer to the node with the byte that follows in the word (child).
//
// A NUL byte is used where the word may end.  The bytes are sorted, so that
// binary searching can be used and the NUL bytes are at the start.  The
// number of possible bytes is stored before the list of bytes.
//
// The tree uses two arrays: "byts" stores the characters, "idxs" stores
// either the next index or flags.  The tree starts at index 0.  For example,
// to lookup "vi" this sequence is followed:
//      i = 0
//      len = byts[i]
//      n = where "v" appears in byts[i + 1] to byts[i + len]
//      i = idxs[n]
//      len = byts[i]
//      n = where "i" appears in byts[i + 1] to byts[i + len]
//      i = idxs[n]
//      len = byts[i]
//      find that byts[i + 1] is 0, idxs[i + 1] has flags for "vi".
//
// There are two word trees: one with case-folded words and one with words in
// original case.  The second one is only used for keep-case words and is
// usually small.
//
// There is one additional tree for when not all prefixes are applied when
// generating the .spl file.  This tree stores all the possible prefixes, as
// if they were words.  At each word (prefix) end the prefix nr is stored, the
// following word must support this prefix nr.  And the condition nr is
// stored, used to lookup the condition that the word must match with.
//
// Thanks to Olaf Seibert for providing an example implementation of this tree
// and the compression mechanism.
// LZ trie ideas, original link (now dead)
//      irb.hr/hr/home/ristov/papers/RistovLZtrieRevision1.pdf
// More papers: http://www-igm.univ-mlv.fr/~laporte/publi_en.html
//
// Matching involves checking the caps type: Onecap ALLCAP KeepCap.
//
// Why doesn't Vim use aspell/ispell/myspell/etc.?
// See ":help develop-spell".

// Use SPELL_PRINTTREE for debugging: dump the word tree after adding a word.
// Only use it for small word lists!

// Use SPELL_COMPRESS_ALWAYS for debugging: compress the word tree after
// adding a word.  Only use it for small word lists!

// Use DEBUG_TRIEWALK to print the changes made in suggest_trie_walk() for a
// specific word.

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/insexpand.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"



// First language that is loaded, start of the linked list of loaded
// languages.
slang_T *first_lang = NULL;

// file used for "zG" and "zW"
char *int_wordlist = NULL;

// Structure to store info for word matching.


// Structure used for the cookie argument of do_in_runtimepath().
typedef struct {
  char sl_lang[MAXWLEN + 1];            // language name
  slang_T *sl_slang;                    // resulting slang_T struct
  int sl_nobreak;                       // NOBREAK language found
} spelload_T;

spelltab_T spelltab;
bool did_set_spelltab;

#include "spell.c.generated.h"
extern int rs_win_valid_any_tab(win_T *win);
extern int rs_ins_compl_interrupted(void);

extern void rs_optval_free(OptVal o);

// Rust implementations of spell functions
extern int rs_find_region(const char *rp, const char *region);
// spell_iswordp_w was static in C but is now exported from Rust
extern bool spell_iswordp_w(const int *p, const win_T *wp);
// Phase 5: functions now implemented in Rust (were static in C)
extern char *advance_camelcase_word(char *str, win_T *wp, bool *is_camel_case);
extern int count_syllables(slang_T *slang, const char *word);

// Static assertions to validate Rust repr(C) struct layout matches C struct layout.
// These catch layout mismatches at compile time before they cause silent bugs.
_Static_assert(sizeof(spelltab_T) == 1024, "spelltab_T size mismatch");
_Static_assert(sizeof(langp_T) == 32, "langp_T size mismatch");
_Static_assert(sizeof(slang_T) == 4344, "slang_T size mismatch");
_Static_assert(offsetof(slang_T, sl_fbyts) == 32, "sl_fbyts offset mismatch");
_Static_assert(offsetof(slang_T, sl_fidxs) == 48, "sl_fidxs offset mismatch");
_Static_assert(offsetof(slang_T, sl_sal) == 1088, "sl_sal offset mismatch");
_Static_assert(offsetof(slang_T, sl_sal_first) == 1112, "sl_sal_first offset mismatch");
_Static_assert(offsetof(slang_T, sl_rep) == 552, "sl_rep offset mismatch");
_Static_assert(offsetof(slang_T, sl_rep_first) == 576, "sl_rep_first offset mismatch");
_Static_assert(offsetof(slang_T, sl_sofo) == 2139, "sl_sofo offset mismatch");
_Static_assert(offsetof(slang_T, sl_compmax) == 424, "sl_compmax offset mismatch");
_Static_assert(offsetof(slang_T, sl_has_map) == 2721, "sl_has_map offset mismatch");
_Static_assert(offsetof(slang_T, sl_map_hash) == 2728, "sl_map_hash offset mismatch");
_Static_assert(offsetof(slang_T, sl_map_array) == 3024, "sl_map_array offset mismatch");
_Static_assert(offsetof(slang_T, sl_sounddone) == 4048, "sl_sounddone offset mismatch");
_Static_assert(offsetof(slang_T, sl_sbyts) == 2696, "sl_sbyts offset mismatch");
_Static_assert(offsetof(slang_T, sl_sidxs) == 2704, "sl_sidxs offset mismatch");



/// type values for get_char_type
enum {
  CHAR_OTHER = 0,
  CHAR_UPPER = 1,
  CHAR_DIGIT = 2,
};

char *e_format = N_("E759: Format error in spell file");

// Remember what "z?" replaced.
char *repl_from = NULL;
char *repl_to = NULL;












static void decor_spell_nav_start(win_T *wp)
{
  decor_state = (DecorState){ 0 };
  decor_redraw_reset(wp, &decor_state);
}

static TriState decor_spell_nav_col(win_T *wp, linenr_T lnum, linenr_T *decor_lnum, int col)
{
  if (*decor_lnum != lnum) {
    decor_providers_invoke_spell(wp, lnum - 1, col, lnum - 1, -1);
    decor_redraw_line(wp, lnum - 1, &decor_state);
    *decor_lnum = lnum;
  }
  decor_redraw_col(wp, col, 0, false, &decor_state);
  return decor_state.spell;
}

static inline bool can_syn_spell(win_T *wp, linenr_T lnum, int col)
{
  bool can_spell;
  syn_get_id(wp, lnum, col, false, &can_spell, false);
  return can_spell;
}

/// Moves to the next spell error.
/// "curline" is false for "[s", "]s", "[S" and "]S".
/// "curline" is true to find word under/after cursor in the same line.
/// For Insert mode completion "dir" is BACKWARD and "curline" is true: move
/// to after badly spelled word before the cursor.
///
/// @param dir  FORWARD or BACKWARD
/// @param behaviour  Behaviour of the function
/// @param attrp  return: attributes of bad word or NULL (only when "dir" is FORWARD)
///
/// @return  0 if not found, length of the badly spelled word otherwise.
size_t spell_move_to(win_T *wp, int dir, smt_T behaviour, bool curline, hlf_T *attrp)
{
  if (no_spell_checking(wp)) {
    return 0;
  }

  pos_T found_pos;
  size_t found_len = 0;
  hlf_T attr = HLF_COUNT;
  bool has_syntax = syntax_present(wp);
  char *buf = NULL;
  size_t buflen = 0;
  int skip = 0;
  colnr_T capcol = -1;
  bool found_one = false;
  bool wrapped = false;

  size_t ret = 0;

  // Start looking for bad word at the start of the line, because we can't
  // start halfway through a word, we don't know where it starts or ends.
  //
  // When searching backwards, we continue in the line to find the last
  // bad word (in the cursor line: before the cursor).
  //
  // We concatenate the start of the next line, so that wrapped words work
  // (e.g. "et<line-break>cetera").  Doesn't work when searching backwards
  // though...
  linenr_T lnum = wp->w_cursor.lnum;
  clearpos(&found_pos);

  // Ephemeral extmarks are currently stored in the global decor_state.
  // When looking for spell errors, we need to:
  //  - temporarily reset decor_state
  //  - run the _on_spell_nav decor callback for each line we look at
  //  - detect if any spell marks are present
  //  - restore decor_state to the value saved here.
  // TODO(lewis6991): un-globalize decor_state and allow ephemeral marks to be stored into a
  // temporary DecorState.
  DecorState saved_decor_start = decor_state;
  linenr_T decor_lnum = -1;
  decor_spell_nav_start(wp);

  while (!got_int) {
    char *line = ml_get_buf(wp->w_buffer, lnum);

    size_t len = (size_t)ml_get_buf_len(wp->w_buffer, lnum);
    if (buflen < len + MAXWLEN + 2) {
      xfree(buf);
      buflen = len + MAXWLEN + 2;
      buf = xmalloc(buflen);
    }
    assert(buf && buflen >= len + MAXWLEN + 2);

    // In first line check first word for Capital.
    if (lnum == 1) {
      capcol = 0;
    }

    // For checking first word with a capital skip white space.
    if (capcol == 0) {
      capcol = (colnr_T)getwhitecols(line);
    } else if (curline && wp == curwin) {
      // For spellbadword(): check if first word needs a capital.
      colnr_T col = (colnr_T)getwhitecols(line);
      if (check_need_cap(curwin, lnum, col)) {
        capcol = col;
      }

      // Need to get the line again, may have looked at the previous
      // one.
      line = ml_get_buf(wp->w_buffer, lnum);
    }

    // Copy the line into "buf" and append the start of the next line if
    // possible.  Note: this ml_get_buf() may make "line" invalid, check
    // for empty line first.
    bool empty_line = *skipwhite(line) == NUL;
    STRCPY(buf, line);
    if (lnum < wp->w_buffer->b_ml.ml_line_count) {
      spell_cat_line(buf + strlen(buf),
                     ml_get_buf(wp->w_buffer, lnum + 1),
                     MAXWLEN);
    }
    char *p = buf + skip;
    char *endp = buf + len;
    while (p < endp) {
      // When searching backward don't search after the cursor.  Unless
      // we wrapped around the end of the buffer.
      if (dir == BACKWARD
          && lnum == wp->w_cursor.lnum
          && !wrapped
          && (colnr_T)(p - buf) >= wp->w_cursor.col) {
        break;
      }

      // start of word
      attr = HLF_COUNT;
      len = spell_check(wp, p, &attr, &capcol, false);

      if (attr != HLF_COUNT) {
        // We found a bad word.  Check the attribute.
        if (behaviour == SMT_ALL
            || (behaviour == SMT_BAD && attr == HLF_SPB)
            || (behaviour == SMT_RARE && attr == HLF_SPR)) {
          // When searching forward only accept a bad word after
          // the cursor.
          if (dir == BACKWARD
              || lnum != wp->w_cursor.lnum
              || wrapped
              || ((colnr_T)(curline
                            ? p - buf + (ptrdiff_t)len
                            : p - buf) > wp->w_cursor.col)) {
            colnr_T col = (colnr_T)(p - buf);

            bool no_plain_buffer = (wp->w_s->b_p_spo_flags & kOptSpoFlagNoplainbuffer) != 0;
            bool can_spell = !no_plain_buffer;
            switch (decor_spell_nav_col(wp, lnum, &decor_lnum, col)) {
            case kTrue:
              can_spell = true; break;
            case kFalse:
              can_spell = false; break;
            case kNone:
              if (has_syntax) {
                can_spell = can_syn_spell(wp, lnum, col);
              }
            }

            if (!can_spell) {
              attr = HLF_COUNT;
            }

            if (can_spell) {
              found_one = true;
              found_pos = (pos_T) {
                .lnum = lnum,
                .col = col,
                .coladd = 0
              };
              if (dir == FORWARD) {
                // No need to search further.
                wp->w_cursor = found_pos;
                if (attrp != NULL) {
                  *attrp = attr;
                }
                ret = len;
                goto theend;
              } else if (curline) {
                // Insert mode completion: put cursor after
                // the bad word.
                assert(len <= INT_MAX);
                found_pos.col += (int)len;
              }
              found_len = len;
            }
          } else {
            found_one = true;
          }
        }
      }

      // advance to character after the word
      p += len;
      assert(len <= INT_MAX);
      capcol -= (int)len;
    }

    if (dir == BACKWARD && found_pos.lnum != 0) {
      // Use the last match in the line (before the cursor).
      wp->w_cursor = found_pos;
      ret = found_len;
      goto theend;
    }

    if (curline) {
      break;            // only check cursor line
    }

    // If we are back at the starting line and searched it again there
    // is no match, give up.
    if (lnum == wp->w_cursor.lnum && wrapped) {
      break;
    }

    // Advance to next line.
    if (dir == BACKWARD) {
      if (lnum > 1) {
        lnum--;
      } else if (!p_ws) {
        break;              // at first line and 'nowrapscan'
      } else {
        // Wrap around to the end of the buffer.  May search the
        // starting line again and accept the last match.
        lnum = wp->w_buffer->b_ml.ml_line_count;
        wrapped = true;
        if (!shortmess(SHM_SEARCH)) {
          give_warning(_(top_bot_msg), true);
        }
      }
      capcol = -1;
    } else {
      if (lnum < wp->w_buffer->b_ml.ml_line_count) {
        lnum++;
      } else if (!p_ws) {
        break;              // at first line and 'nowrapscan'
      } else {
        // Wrap around to the start of the buffer.  May search the
        // starting line again and accept the first match.
        lnum = 1;
        wrapped = true;
        if (!shortmess(SHM_SEARCH)) {
          give_warning(_(bot_top_msg), true);
        }
      }

      // If we are back at the starting line and there is no match then
      // give up.
      if (lnum == wp->w_cursor.lnum && !found_one) {
        break;
      }

      // Skip the characters at the start of the next line that were
      // included in a match crossing line boundaries.
      if (attr == HLF_COUNT) {
        skip = (int)(p - endp);
      } else {
        skip = 0;
      }

      // Capcol skips over the inserted space.
      capcol--;

      // But after empty line check first word in next line
      if (empty_line) {
        capcol = 0;
      }
    }

    line_breakcheck();
  }

theend:
  decor_state_free(&decor_state);
  decor_state = saved_decor_start;
  xfree(buf);
  return ret;
}


// Load word list(s) for "lang" from Vim spell file(s).
// "lang" must be the language without the region: e.g., "en".
static void spell_load_lang(char *lang)
{
  char fname_enc[85];
  int r;
  spelload_T sl;

  // Copy the language name to pass it to spell_load_cb() as a cookie.
  // It's truncated when an error is detected.
  STRCPY(sl.sl_lang, lang);
  sl.sl_slang = NULL;
  sl.sl_nobreak = false;

  // Disallow deleting the current buffer.  Autocommands can do weird things
  // and cause "lang" to be freed.
  curbuf->b_locked++;

  // We may retry when no spell file is found for the language, an
  // autocommand may load it then.
  for (int round = 1; round <= 2; round++) {
    // Find the first spell file for "lang" in 'runtimepath' and load it.
    vim_snprintf(fname_enc, sizeof(fname_enc) - 5,
                 "spell/%s.%s.spl", lang, spell_enc());
    r = do_in_runtimepath(fname_enc, 0, spell_load_cb, &sl);

    if (r == FAIL && *sl.sl_lang != NUL) {
      // Try loading the ASCII version.
      vim_snprintf(fname_enc, sizeof(fname_enc) - 5,
                   "spell/%s.ascii.spl", lang);
      r = do_in_runtimepath(fname_enc, 0, spell_load_cb, &sl);

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
      // Prompt the user at VimEnter if spell files are missing. #3027
      // Plugins aren't loaded yet, so nvim/spellfile.lua cannot handle this case.
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
    // At least one file was loaded, now load ALL the additions.
    STRCPY(fname_enc + strlen(fname_enc) - 3, "add.spl");
    do_in_runtimepath(fname_enc, DIP_ALL, spell_load_cb, &sl);
  }

  curbuf->b_locked--;
}


// Get the name of the .spl file for the internal wordlist into
// "fname[MAXPATHL]".
static void int_wordlist_spl(char *fname)
{
  vim_snprintf(fname, MAXPATHL, SPL_FNAME_TMPL,
               int_wordlist, spell_enc());
}

/// Allocate a new slang_T for language "lang".  "lang" can be NULL.
/// Caller must fill "sl_next".
slang_T *slang_alloc(char *lang)
  FUNC_ATTR_NONNULL_RET
{
  slang_T *lp = xcalloc(1, sizeof(slang_T));

  if (lang != NULL) {
    lp->sl_name = xstrdup(lang);
  }
  ga_init(&lp->sl_rep, sizeof(fromto_T), 10);
  ga_init(&lp->sl_repsal, sizeof(fromto_T), 10);
  lp->sl_compmax = MAXWLEN;
  lp->sl_compsylmax = MAXWLEN;
  hash_init(&lp->sl_wordcount);

  return lp;
}

// Free the contents of an slang_T and the structure itself.
void slang_free(slang_T *lp)
{
  xfree(lp->sl_name);
  xfree(lp->sl_fname);
  slang_clear(lp);
  xfree(lp);
}

/// Frees a salitem_T
static void free_salitem(salitem_T *smp)
{
  xfree(smp->sm_lead);
  // Don't free sm_oneof and sm_rules, they point into sm_lead.
  xfree(smp->sm_to);
  xfree(smp->sm_lead_w);
  xfree(smp->sm_oneof_w);
  xfree(smp->sm_to_w);
}

/// Frees a fromto_T
static void free_fromto(fromto_T *ftp)
{
  xfree(ftp->ft_from);
  xfree(ftp->ft_to);
}

// Clear an slang_T so that the file can be reloaded.
void slang_clear(slang_T *lp)
{
  garray_T *gap;

  XFREE_CLEAR(lp->sl_fbyts);
  XFREE_CLEAR(lp->sl_kbyts);
  XFREE_CLEAR(lp->sl_pbyts);

  XFREE_CLEAR(lp->sl_fidxs);
  XFREE_CLEAR(lp->sl_kidxs);
  XFREE_CLEAR(lp->sl_pidxs);

  GA_DEEP_CLEAR(&lp->sl_rep, fromto_T, free_fromto);
  GA_DEEP_CLEAR(&lp->sl_repsal, fromto_T, free_fromto);

  gap = &lp->sl_sal;
  if (lp->sl_sofo) {
    // "ga_len" is set to 1 without adding an item for latin1
    GA_DEEP_CLEAR_PTR(gap);
  } else {
    // SAL items: free salitem_T items
    GA_DEEP_CLEAR(gap, salitem_T, free_salitem);
  }

  for (int i = 0; i < lp->sl_prefixcnt; i++) {
    vim_regfree(lp->sl_prefprog[i]);
  }
  lp->sl_prefixcnt = 0;
  XFREE_CLEAR(lp->sl_prefprog);
  XFREE_CLEAR(lp->sl_info);
  XFREE_CLEAR(lp->sl_midword);

  vim_regfree(lp->sl_compprog);
  lp->sl_compprog = NULL;
  XFREE_CLEAR(lp->sl_comprules);
  XFREE_CLEAR(lp->sl_compstartflags);
  XFREE_CLEAR(lp->sl_compallflags);

  XFREE_CLEAR(lp->sl_syllable);
  ga_clear(&lp->sl_syl_items);

  ga_clear_strings(&lp->sl_comppat);

  hash_clear_all(&lp->sl_wordcount, WC_KEY_OFF);
  hash_init(&lp->sl_wordcount);

  hash_clear_all(&lp->sl_map_hash, 0);

  // Clear info from .sug file.
  slang_clear_sug(lp);

  lp->sl_compmax = MAXWLEN;
  lp->sl_compminlen = 0;
  lp->sl_compsylmax = MAXWLEN;
  lp->sl_regions[0] = NUL;
}

// Clear the info from the .sug file in "lp".
void slang_clear_sug(slang_T *lp)
{
  XFREE_CLEAR(lp->sl_sbyts);
  XFREE_CLEAR(lp->sl_sidxs);
  close_spellbuf(lp->sl_sugbuf);
  lp->sl_sugbuf = NULL;
  lp->sl_sugloaded = false;
  lp->sl_sugtime = 0;
}

// Load one spell file and store the info into a slang_T.
// Invoked through do_in_runtimepath().
static bool spell_load_cb(int num_fnames, char **fnames, bool all, void *cookie)
{
  spelload_T *slp = (spelload_T *)cookie;
  for (int i = 0; i < num_fnames; i++) {
    slang_T *slang = spell_load_file(fnames[i], slp->sl_lang, NULL, false);

    if (slang == NULL) {
      continue;
    }

    // When a previously loaded file has NOBREAK also use it for the
    // ".add" files.
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

/// Parse 'spelllang' and set w_s->b_langp accordingly.
/// @return  NULL if it's OK, an untranslated error message otherwise.
char *parse_spelllang(win_T *wp)
{
  char region_cp[3];
  char lang[MAXWLEN + 1];
  char spf_name[MAXPATHL];
  char *use_region = NULL;
  bool dont_use_region = false;
  bool nobreak = false;
  static bool recursive = false;
  char *ret_msg = NULL;

  bufref_T bufref;
  set_bufref(&bufref, wp->w_buffer);

  // We don't want to do this recursively.  May happen when a language is
  // not available and the SpellFileMissing autocommand opens a new buffer
  // in which 'spell' is set.
  if (recursive) {
    return NULL;
  }
  recursive = true;

  garray_T ga;
  ga_init(&ga, sizeof(langp_T), 2);
  clear_midword(wp);

  // Make a copy of 'spelllang', the SpellFileMissing autocommands may change
  // it under our fingers.
  char *spl_copy = xstrdup(wp->w_s->b_p_spl);

  wp->w_s->b_cjk = 0;

  // Loop over comma separated language names.
  for (char *splp = spl_copy; *splp != NUL;) {
    // Get one language name.
    copy_option_part(&splp, lang, MAXWLEN, ",");
    char *region = NULL;
    int len = (int)strlen(lang);

    if (!valid_spelllang(lang)) {
      continue;
    }

    if (strcmp(lang, "cjk") == 0) {
      wp->w_s->b_cjk = 1;
      continue;
    }

    slang_T *slang;
    bool filename;
    // If the name ends in ".spl" use it as the name of the spell file.
    // If there is a region name let "region" point to it and remove it
    // from the name.
    if (len > 4 && path_fnamecmp(lang + len - 4, ".spl") == 0) {
      filename = true;

      // Locate a region and remove it from the file name.
      char *p = vim_strchr(path_tail(lang), '_');
      if (p != NULL && ASCII_ISALPHA(p[1]) && ASCII_ISALPHA(p[2])
          && !ASCII_ISALPHA(p[3])) {
        xstrlcpy(region_cp, p + 1, 3);
        memmove(p, p + 3, (size_t)(len - (p - lang) - 2));
        region = region_cp;
      } else {
        dont_use_region = true;
      }

      // Check if we loaded this language before.
      for (slang = first_lang; slang != NULL; slang = slang->sl_next) {
        if (path_full_compare(lang, slang->sl_fname, false, true)
            == kEqualFiles) {
          break;
        }
      }
    } else {
      filename = false;
      if (len > 3 && lang[len - 3] == '_') {
        region = lang + len - 2;
        lang[len - 3] = NUL;
      } else {
        dont_use_region = true;
      }

      // Check if we loaded this language before.
      for (slang = first_lang; slang != NULL; slang = slang->sl_next) {
        if (STRICMP(lang, slang->sl_name) == 0) {
          break;
        }
      }
    }

    if (region != NULL) {
      // If the region differs from what was used before then don't
      // use it for 'spellfile'.
      if (use_region != NULL && strcmp(region, use_region) != 0) {
        dont_use_region = true;
      }
      use_region = region;
    }

    // If not found try loading the language now.
    if (slang == NULL) {
      if (filename) {
        spell_load_file(lang, lang, NULL, false);
      } else {
        spell_load_lang(lang);
        // SpellFileMissing autocommands may do anything, including
        // destroying the buffer we are using or closing the window.
        if (!bufref_valid(&bufref) || !rs_win_valid_any_tab(wp)) {
          ret_msg = N_("E797: SpellFileMissing autocommand deleted buffer");
          goto theend;
        }
      }
    }

    // Loop over the languages, there can be several files for "lang".
    for (slang = first_lang; slang != NULL; slang = slang->sl_next) {
      if (filename
          ? path_full_compare(lang, slang->sl_fname, false, true) == kEqualFiles
          : STRICMP(lang, slang->sl_name) == 0) {
        int region_mask = REGION_ALL;
        if (!filename && region != NULL) {
          // find region in sl_regions
          int c = rs_find_region(slang->sl_regions, region);
          if (c == REGION_ALL) {
            if (slang->sl_add) {
              if (*slang->sl_regions != NUL) {
                // This addition file is for other regions.
                region_mask = 0;
              }
            } else {
              // This is probably an error.  Give a warning and
              // accept the words anyway.
              smsg(0, _("Warning: region %s not supported"),
                   region);
            }
          } else {
            region_mask = 1 << c;
          }
        }

        if (region_mask != 0) {
          langp_T *p_ = GA_APPEND_VIA_PTR(langp_T, &ga);
          p_->lp_slang = slang;
          p_->lp_region = region_mask;

          use_midword(slang, wp);
          if (slang->sl_nobreak) {
            nobreak = true;
          }
        }
      }
    }
  }

  // round 0: load int_wordlist, if possible.
  // round 1: load first name in 'spellfile'.
  // round 2: load second name in 'spellfile.
  // etc.
  char *spf = curwin->w_s->b_p_spf;
  for (int round = 0; round == 0 || *spf != NUL; round++) {
    if (round == 0) {
      // Internal wordlist, if there is one.
      if (int_wordlist == NULL) {
        continue;
      }
      int_wordlist_spl(spf_name);
    } else {
      // One entry in 'spellfile'.
      copy_option_part(&spf, spf_name, MAXPATHL - 4, ",");
      strcat(spf_name, ".spl");
      int c;

      // If it was already found above then skip it.
      for (c = 0; c < ga.ga_len; c++) {
        char *p = LANGP_ENTRY(ga, c)->lp_slang->sl_fname;
        if (p != NULL
            && path_full_compare(spf_name, p, false, true) == kEqualFiles) {
          break;
        }
      }
      if (c < ga.ga_len) {
        continue;
      }
    }

    slang_T *slang;

    // Check if it was loaded already.
    for (slang = first_lang; slang != NULL; slang = slang->sl_next) {
      if (path_full_compare(spf_name, slang->sl_fname, false, true)
          == kEqualFiles) {
        break;
      }
    }
    if (slang == NULL) {
      // Not loaded, try loading it now.  The language name includes the
      // region name, the region is ignored otherwise.  for int_wordlist
      // use an arbitrary name.
      if (round == 0) {
        STRCPY(lang, "internal wordlist");
      } else {
        xstrlcpy(lang, path_tail(spf_name), MAXWLEN + 1);
        char *p = vim_strchr(lang, '.');
        if (p != NULL) {
          *p = NUL;             // truncate at ".encoding.add"
        }
      }
      slang = spell_load_file(spf_name, lang, NULL, true);

      // If one of the languages has NOBREAK we assume the addition
      // files also have this.
      if (slang != NULL && nobreak) {
        slang->sl_nobreak = true;
      }
    }
    if (slang != NULL) {
      int region_mask = REGION_ALL;
      if (use_region != NULL && !dont_use_region) {
        // find region in sl_regions
        int c = rs_find_region(slang->sl_regions, use_region);
        if (c != REGION_ALL) {
          region_mask = 1 << c;
        } else if (*slang->sl_regions != NUL) {
          // This spell file is for other regions.
          region_mask = 0;
        }
      }

      if (region_mask != 0) {
        langp_T *p_ = GA_APPEND_VIA_PTR(langp_T, &ga);
        p_->lp_slang = slang;
        p_->lp_sallang = NULL;
        p_->lp_replang = NULL;
        p_->lp_region = region_mask;

        use_midword(slang, wp);
      }
    }
  }

  // Everything is fine, store the new b_langp value.
  ga_clear(&wp->w_s->b_langp);
  wp->w_s->b_langp = ga;

  // For each language figure out what language to use for sound folding and
  // REP items.  If the language doesn't support it itself use another one
  // with the same name.  E.g. for "en-math" use "en".
  for (int i = 0; i < ga.ga_len; i++) {
    langp_T *lp = LANGP_ENTRY(ga, i);

    // sound folding
    if (!GA_EMPTY(&lp->lp_slang->sl_sal)) {
      // language does sound folding itself
      lp->lp_sallang = lp->lp_slang;
    } else {
      // find first similar language that does sound folding
      for (int j = 0; j < ga.ga_len; j++) {
        langp_T *lp2 = LANGP_ENTRY(ga, j);
        if (!GA_EMPTY(&lp2->lp_slang->sl_sal)
            && strncmp(lp->lp_slang->sl_name,
                       lp2->lp_slang->sl_name, 2) == 0) {
          lp->lp_sallang = lp2->lp_slang;
          break;
        }
      }
    }

    // REP items
    if (!GA_EMPTY(&lp->lp_slang->sl_rep)) {
      // language has REP items itself
      lp->lp_replang = lp->lp_slang;
    } else {
      // find first similar language that has REP items
      for (int j = 0; j < ga.ga_len; j++) {
        langp_T *lp2 = LANGP_ENTRY(ga, j);
        if (!GA_EMPTY(&lp2->lp_slang->sl_rep)
            && strncmp(lp->lp_slang->sl_name,
                       lp2->lp_slang->sl_name, 2) == 0) {
          lp->lp_replang = lp2->lp_slang;
          break;
        }
      }
    }
  }
  redraw_later(wp, UPD_NOT_VALID);

theend:
  xfree(spl_copy);
  recursive = false;
  return ret_msg;
}

// Clear the midword characters for buffer "buf".
static void clear_midword(win_T *wp)
{
  CLEAR_FIELD(wp->w_s->b_spell_ismw);
  XFREE_CLEAR(wp->w_s->b_spell_ismw_mb);
}

/// Use the "sl_midword" field of language "lp" for buffer "buf".
/// They add up to any currently used midword characters.
static void use_midword(slang_T *lp, win_T *wp)
  FUNC_ATTR_NONNULL_ALL
{
  if (lp->sl_midword == NULL) {  // there aren't any
    return;
  }

  for (char *p = lp->sl_midword; *p != NUL;) {
    const int c = utf_ptr2char(p);
    const int l = utfc_ptr2len(p);
    if (c < 256 && l <= 2) {
      wp->w_s->b_spell_ismw[c] = true;
    } else if (wp->w_s->b_spell_ismw_mb == NULL) {
      // First multi-byte char in "b_spell_ismw_mb".
      wp->w_s->b_spell_ismw_mb = xmemdupz(p, (size_t)l);
    } else {
      // Append multi-byte chars to "b_spell_ismw_mb".
      const int n = (int)strlen(wp->w_s->b_spell_ismw_mb);
      char *bp = xstrnsave(wp->w_s->b_spell_ismw_mb, (size_t)n + (size_t)l);
      xfree(wp->w_s->b_spell_ismw_mb);
      wp->w_s->b_spell_ismw_mb = bp;
      xmemcpyz(bp + n, p, (size_t)l);
    }
    p += l;
  }
}



// Delete the internal wordlist and its .spl file.
void spell_delete_wordlist(void)
{
  if (int_wordlist == NULL) {
    return;
  }

  char fname[MAXPATHL] = { 0 };
  os_remove(int_wordlist);
  int_wordlist_spl(fname);
  os_remove(fname);
  XFREE_CLEAR(int_wordlist);
}

// Free all languages.
void spell_free_all(void)
{
  // Go through all buffers and handle 'spelllang'. <VN>
  FOR_ALL_BUFFERS(buf) {
    ga_clear(&buf->b_s.b_langp);
  }

  while (first_lang != NULL) {
    slang_T *slang = first_lang;
    first_lang = slang->sl_next;
    slang_free(slang);
  }

  spell_delete_wordlist();

  XFREE_CLEAR(repl_to);
  XFREE_CLEAR(repl_from);
}

// Clear all spelling tables and reload them.
// Used after 'encoding' is set and when ":mkspell" was used.
void spell_reload(void)
{
  // Initialize the table for spell_iswordp().
  init_spell_chartab();

  // Unload all allocated memory.
  spell_free_all();

  // Go through all buffers and handle 'spelllang'.
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    // Only load the wordlists when 'spelllang' is set and there is a
    // window for this buffer in which 'spell' is set.
    if (*wp->w_s->b_p_spl != NUL) {
      if (wp->w_p_spell) {
        parse_spelllang(wp);
        break;
      }
    }
  }
}

// Open a spell buffer.  This is a nameless buffer that is not in the buffer
// list and only contains text lines.  Can use a swapfile to reduce memory
// use.
// Most other fields are invalid!  Esp. watch out for string options being
// NULL and there is no undo info.
buf_T *open_spellbuf(void)
{
  buf_T *buf = xcalloc(1, sizeof(buf_T));

  buf->b_spell = true;
  buf->b_p_swf = true;        // may create a swap file
  if (ml_open(buf) == FAIL) {
    ELOG("Error opening a new memline");
  }
  ml_open_file(buf);          // create swap file now

  return buf;
}

// Close the buffer used for spell info.
void close_spellbuf(buf_T *buf)
{
  if (buf == NULL) {
    return;
  }

  ml_close(buf, true);
  xfree(buf);
}


// Check if the word at line "lnum" column "col" is required to start with a
// capital.  This uses 'spellcapcheck' of the buffer in window "wp".
bool check_need_cap(win_T *wp, linenr_T lnum, colnr_T col)
{
  if (wp->w_s->b_cap_prog == NULL) {
    return false;
  }

  bool need_cap = false;
  char *line = col ? ml_get_buf(wp->w_buffer, lnum) : NULL;
  char *line_copy = NULL;
  colnr_T endcol = 0;
  if (col == 0 || getwhitecols(line) >= col) {
    // At start of line, check if previous line is empty or sentence
    // ends there.
    if (lnum == 1) {
      need_cap = true;
    } else {
      line = ml_get_buf(wp->w_buffer, lnum - 1);
      if (*skipwhite(line) == NUL) {
        need_cap = true;
      } else {
        // Append a space in place of the line break.
        line_copy = concat_str(line, " ");
        line = line_copy;
        endcol = (colnr_T)strlen(line);
      }
    }
  } else {
    endcol = col;
  }

  if (endcol > 0) {
    // Check if sentence ends before the bad word.
    regmatch_T regmatch = {
      .regprog = wp->w_s->b_cap_prog,
      .rm_ic = false
    };
    char *p = line + endcol;
    while (true) {
      MB_PTR_BACK(line, p);
      if (p == line || spell_iswordp_nmw(p, wp)) {
        break;
      }
      if (vim_regexec(&regmatch, p, 0)
          && regmatch.endp[0] == line + endcol) {
        need_cap = true;
        break;
      }
    }
    wp->w_s->b_cap_prog = regmatch.regprog;
  }

  xfree(line_copy);

  return need_cap;
}

// ":spellrepall"
void ex_spellrepall(exarg_T *eap)
{
  pos_T pos = curwin->w_cursor;
  bool save_ws = p_ws;
  linenr_T prev_lnum = 0;

  if (repl_from == NULL || repl_to == NULL) {
    emsg(_("E752: No previous spell replacement"));
    return;
  }
  const size_t repl_from_len = strlen(repl_from);
  const size_t repl_to_len = strlen(repl_to);
  const int addlen = (int)(repl_to_len - repl_from_len);

  const size_t frompatsize = repl_from_len + 7;
  char *frompat = xmalloc(frompatsize);
  size_t frompatlen = (size_t)snprintf(frompat, frompatsize, "\\V\\<%s\\>", repl_from);
  p_ws = false;

  sub_nsubs = 0;
  sub_nlines = 0;
  curwin->w_cursor.lnum = 0;
  while (!got_int) {
    if (do_search(NULL, '/', '/', frompat, frompatlen, 1, SEARCH_KEEP, NULL) == 0
        || u_save_cursor() == FAIL) {
      break;
    }

    // Only replace when the right word isn't there yet.  This happens
    // when changing "etc" to "etc.".
    char *line = get_cursor_line_ptr();
    if (addlen <= 0
        || strncmp(line + curwin->w_cursor.col, repl_to, repl_to_len) != 0) {
      char *p = xmalloc((size_t)get_cursor_line_len() + (size_t)addlen + 1);
      memmove(p, line, (size_t)curwin->w_cursor.col);
      STRCPY(p + curwin->w_cursor.col, repl_to);
      strcat(p, line + curwin->w_cursor.col + repl_from_len);
      ml_replace(curwin->w_cursor.lnum, p, false);
      inserted_bytes(curwin->w_cursor.lnum, curwin->w_cursor.col,
                     (int)repl_from_len, (int)repl_to_len);

      if (curwin->w_cursor.lnum != prev_lnum) {
        sub_nlines++;
        prev_lnum = curwin->w_cursor.lnum;
      }
      sub_nsubs++;
    }
    curwin->w_cursor.col += (colnr_T)repl_to_len;
  }

  p_ws = save_ws;
  curwin->w_cursor = pos;
  xfree(frompat);

  if (sub_nsubs == 0) {
    semsg(_("E753: Not found: %s"), repl_from);
  } else {
    do_sub_msg(false);
  }
}


// ":spellinfo"
void ex_spellinfo(exarg_T *eap)
{
  if (no_spell_checking(curwin)) {
    return;
  }

  msg_start();
  for (int lpi = 0; lpi < curwin->w_s->b_langp.ga_len && !got_int; lpi++) {
    langp_T *const lp = LANGP_ENTRY(curwin->w_s->b_langp, lpi);
    msg_puts("file: ");
    msg_puts(lp->lp_slang->sl_fname);
    msg_putchar('\n');
    const char *const p = lp->lp_slang->sl_info;
    if (p != NULL) {
      msg_puts(p);
      msg_putchar('\n');
    }
  }
  msg_end();
}

#define DUMPFLAG_KEEPCASE   1   // round 2: keep-case tree
#define DUMPFLAG_COUNT      2   // include word count
#define DUMPFLAG_ICASE      4   // ignore case when finding matches
#define DUMPFLAG_ONECAP     8   // pattern starts with capital
#define DUMPFLAG_ALLCAP     16  // pattern is all capitals

// ":spelldump"
void ex_spelldump(exarg_T *eap)
{
  if (no_spell_checking(curwin)) {
    return;
  }
  OptVal spl = get_option_value(kOptSpelllang, OPT_LOCAL);

  // Create a new empty buffer in a new window.
  do_cmdline_cmd("new");

  // enable spelling locally in the new window
  set_option_value_give_err(kOptSpell, BOOLEAN_OPTVAL(true), OPT_LOCAL);
  set_option_value_give_err(kOptSpelllang, spl, OPT_LOCAL);
  rs_optval_free(spl);

  if (!buf_is_empty(curbuf)) {
    return;
  }

  spell_dump_compl(NULL, 0, NULL, eap->forceit ? DUMPFLAG_COUNT : 0);

  // Delete the empty line that we started with.
  if (curbuf->b_ml.ml_line_count > 1) {
    ml_delete(curbuf->b_ml.ml_line_count);
  }
  redraw_later(curwin, UPD_NOT_VALID);
}

/// Go through all possible words and:
/// 1. When "pat" is NULL: dump a list of all words in the current buffer.
///      "ic" and "dir" are not used.
/// 2. When "pat" is not NULL: add matching words to insert mode completion.
///
/// @param pat  leading part of the word
/// @param ic  ignore case
/// @param dir  direction for adding matches
/// @param dumpflags_arg  DUMPFLAG_*
void spell_dump_compl(char *pat, int ic, Direction *dir, int dumpflags_arg)
{
  idx_T arridx[MAXWLEN];
  int curi[MAXWLEN];
  char word[MAXWLEN];
  linenr_T lnum = 0;
  char *region_names = NULL;         // region names being used
  bool do_region = true;                    // dump region names and numbers
  int dumpflags = dumpflags_arg;

  // When ignoring case or when the pattern starts with capital pass this on
  // to dump_word().
  if (pat != NULL) {
    if (ic) {
      dumpflags |= DUMPFLAG_ICASE;
    } else {
      int n = captype(pat, NULL);
      if (n == WF_ONECAP) {
        dumpflags |= DUMPFLAG_ONECAP;
      } else if (n == WF_ALLCAP
                 && (int)strlen(pat) > utfc_ptr2len(pat)) {
        dumpflags |= DUMPFLAG_ALLCAP;
      }
    }
  }

  // Find out if we can support regions: All languages must support the same
  // regions or none at all.
  for (int lpi = 0; lpi < curwin->w_s->b_langp.ga_len; lpi++) {
    langp_T *lp = LANGP_ENTRY(curwin->w_s->b_langp, lpi);
    char *p = lp->lp_slang->sl_regions;
    if (p[0] != 0) {
      if (region_names == NULL) {           // first language with regions
        region_names = p;
      } else if (strcmp(region_names, p) != 0) {
        do_region = false;                  // region names are different
        break;
      }
    }
  }

  if (do_region && region_names != NULL && pat == NULL) {
    vim_snprintf(IObuff, IOSIZE, "/regions=%s", region_names);
    ml_append(lnum++, IObuff, 0, false);
  } else {
    do_region = false;
  }

  // Loop over all files loaded for the entries in 'spelllang'.
  for (int lpi = 0; lpi < curwin->w_s->b_langp.ga_len; lpi++) {
    langp_T *lp = LANGP_ENTRY(curwin->w_s->b_langp, lpi);
    slang_T *slang = lp->lp_slang;
    if (slang->sl_fbyts == NULL) {          // reloading failed
      continue;
    }

    if (pat == NULL) {
      vim_snprintf(IObuff, IOSIZE, "# file: %s", slang->sl_fname);
      ml_append(lnum++, IObuff, 0, false);
    }

    int patlen;
    // When matching with a pattern and there are no prefixes only use
    // parts of the tree that match "pat".
    if (pat != NULL && slang->sl_pbyts == NULL) {
      patlen = (int)strlen(pat);
    } else {
      patlen = -1;
    }

    // round 1: case-folded tree
    // round 2: keep-case tree
    for (int round = 1; round <= 2; round++) {
      uint8_t *byts;
      idx_T *idxs;
      if (round == 1) {
        dumpflags &= ~DUMPFLAG_KEEPCASE;
        byts = slang->sl_fbyts;
        idxs = slang->sl_fidxs;
      } else {
        dumpflags |= DUMPFLAG_KEEPCASE;
        byts = slang->sl_kbyts;
        idxs = slang->sl_kidxs;
      }
      if (byts == NULL) {
        continue;                       // array is empty
      }
      int depth = 0;
      arridx[0] = 0;
      curi[0] = 1;
      while (depth >= 0 && !got_int
             && (pat == NULL || !rs_ins_compl_interrupted())) {
        if (curi[depth] > byts[arridx[depth]]) {
          // Done all bytes at this node, go up one level.
          depth--;
          line_breakcheck();
          rs_ins_compl_check_keys(50, 0);
        } else {
          // Do one more byte at this node.
          int n = arridx[depth] + curi[depth];
          curi[depth]++;
          int c = byts[n];
          if (c == 0 || depth >= MAXWLEN - 1) {
            // End of word or reached maximum length, deal with the
            // word.
            // Don't use keep-case words in the fold-case tree,
            // they will appear in the keep-case tree.
            // Only use the word when the region matches.
            int flags = (int)idxs[n];
            if ((round == 2 || (flags & WF_KEEPCAP) == 0)
                && (flags & WF_NEEDCOMP) == 0
                && (do_region
                    || (flags & WF_REGION) == 0
                    || (((unsigned)flags >> 16)
                        & (unsigned)lp->lp_region) != 0)) {
              word[depth] = NUL;
              if (!do_region) {
                flags &= ~WF_REGION;
              }

              // Dump the basic word if there is no prefix or
              // when it's the first one.
              c = (int)((unsigned)flags >> 24);
              if (c == 0 || curi[depth] == 2) {
                dump_word(slang, word, pat, dir, dumpflags, flags, lnum);
                if (pat == NULL) {
                  lnum++;
                }
              }

              // Apply the prefix, if there is one.
              if (c != 0) {
                lnum = dump_prefixes(slang, word, pat, dir,
                                     dumpflags, flags, lnum);
              }
            }
          } else {
            // Normal char, go one level deeper.
            word[depth++] = (char)c;
            arridx[depth] = idxs[n];
            curi[depth] = 1;

            // Check if this character matches with the pattern.
            // If not skip the whole tree below it.
            // Always ignore case here, dump_word() will check
            // proper case later.  This isn't exactly right when
            // length changes for multi-byte characters with
            // ignore case...
            assert(depth >= 0);
            if (depth <= patlen
                && mb_strnicmp(word, pat, (size_t)depth) != 0) {
              depth--;
            }
          }
        }
      }
    }
  }
}

/// Dumps one word: apply case modifications and append a line to the buffer.
/// When "lnum" is zero add insert mode completion.
static void dump_word(slang_T *slang, char *word, char *pat, Direction *dir, int dumpflags,
                      int wordflags, linenr_T lnum)
{
  bool keepcap = false;
  char *p;
  char cword[MAXWLEN];
  char badword[MAXWLEN + 10];
  int flags = wordflags;

  if (dumpflags & DUMPFLAG_ONECAP) {
    flags |= WF_ONECAP;
  }
  if (dumpflags & DUMPFLAG_ALLCAP) {
    flags |= WF_ALLCAP;
  }

  if ((dumpflags & DUMPFLAG_KEEPCASE) == 0 && (flags & WF_CAPMASK) != 0) {
    // Need to fix case according to "flags".
    make_case_word(word, cword, flags);
    p = cword;
  } else {
    p = word;
    if ((dumpflags & DUMPFLAG_KEEPCASE)
        && ((captype(word, NULL) & WF_KEEPCAP) == 0
            || (flags & WF_FIXCAP) != 0)) {
      keepcap = true;
    }
  }
  char *tw = p;

  if (pat == NULL) {
    // Add flags and regions after a slash.
    if ((flags & (WF_BANNED | WF_RARE | WF_REGION)) || keepcap) {
      STRCPY(badword, p);
      strcat(badword, "/");
      if (keepcap) {
        strcat(badword, "=");
      }
      if (flags & WF_BANNED) {
        strcat(badword, "!");
      } else if (flags & WF_RARE) {
        strcat(badword, "?");
      }
      if (flags & WF_REGION) {
        for (int i = 0; i < 7; i++) {
          if (flags & (0x10000 << i)) {
            const size_t badword_len = strlen(badword);
            snprintf(badword + badword_len,
                     sizeof(badword) - badword_len,
                     "%d", i + 1);
          }
        }
      }
      p = badword;
    }

    if (dumpflags & DUMPFLAG_COUNT) {
      hashitem_T *hi;

      // Include the word count for ":spelldump!".
      hi = hash_find(&slang->sl_wordcount, tw);
      if (!HASHITEM_EMPTY(hi)) {
        vim_snprintf(IObuff, IOSIZE, "%s\t%d",
                     tw, HI2WC(hi)->wc_count);
        p = IObuff;
      }
    }

    ml_append(lnum, p, 0, false);
  } else if (((dumpflags & DUMPFLAG_ICASE)
              ? mb_strnicmp(p, pat, strlen(pat)) == 0
              : strncmp(p, pat, strlen(pat)) == 0)
             && ins_compl_add_infercase(p, (int)strlen(p),
                                        p_ic, NULL, *dir, false, 0) == OK) {
    // if dir was BACKWARD then honor it just once
    *dir = FORWARD;
  }
}

/// For ":spelldump": Find matching prefixes for "word".  Prepend each to
/// "word" and append a line to the buffer.
/// When "lnum" is zero add insert mode completion.
///
/// @param word  case-folded word
/// @param flags  flags with prefix ID
///
/// @return  the updated line number.
static linenr_T dump_prefixes(slang_T *slang, char *word, char *pat, Direction *dir, int dumpflags,
                              int flags, linenr_T startlnum)
{
  idx_T arridx[MAXWLEN];
  int curi[MAXWLEN];
  char prefix[MAXWLEN];
  char word_up[MAXWLEN];
  bool has_word_up = false;
  linenr_T lnum = startlnum;

  // If the word starts with a lower-case letter make the word with an
  // upper-case letter in word_up[].
  int c = utf_ptr2char(word);
  if (SPELL_TOUPPER(c) != c) {
    onecap_copy(word, word_up, true);
    has_word_up = true;
  }

  uint8_t *byts = slang->sl_pbyts;
  idx_T *idxs = slang->sl_pidxs;
  if (byts != NULL) {           // array not is empty
    // Loop over all prefixes, building them byte-by-byte in prefix[].
    // When at the end of a prefix check that it supports "flags".
    int depth = 0;
    arridx[0] = 0;
    curi[0] = 1;
    while (depth >= 0 && !got_int) {
      int n = arridx[depth];
      int len = byts[n];
      if (curi[depth] > len) {
        // Done all bytes at this node, go up one level.
        depth--;
        line_breakcheck();
      } else {
        // Do one more byte at this node.
        n += curi[depth];
        curi[depth]++;
        c = byts[n];
        if (c == 0) {
          // End of prefix, find out how many IDs there are.
          int i;
          for (i = 1; i < len; i++) {
            if (byts[n + i] != 0) {
              break;
            }
          }
          curi[depth] += i - 1;

          c = valid_word_prefix(i, n, flags, word, slang, false);
          if (c != 0) {
            xstrlcpy(prefix + depth, word, (size_t)(MAXWLEN - depth));
            dump_word(slang, prefix, pat, dir, dumpflags,
                      (c & WF_RAREPFX) ? (flags | WF_RARE) : flags, lnum);
            if (lnum != 0) {
              lnum++;
            }
          }

          // Check for prefix that matches the word when the
          // first letter is upper-case, but only if the prefix has
          // a condition.
          if (has_word_up) {
            c = valid_word_prefix(i, n, flags, word_up, slang, true);
            if (c != 0) {
              xstrlcpy(prefix + depth, word_up, (size_t)(MAXWLEN - depth));
              dump_word(slang, prefix, pat, dir, dumpflags,
                        (c & WF_RAREPFX) ? (flags | WF_RARE) : flags, lnum);
              if (lnum != 0) {
                lnum++;
              }
            }
          }
        } else {
          // Normal char, go one level deeper.
          prefix[depth++] = (char)c;
          arridx[depth] = idxs[n];
          curi[depth] = 1;
        }
      }
    }
  }

  return lnum;
}
