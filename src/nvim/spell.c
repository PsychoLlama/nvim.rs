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
// Rust implementations of spell functions
extern int rs_find_region(const char *rp, const char *region);

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

char *e_format = N_("E759: Format error in spell file");

// Remember what "z?" replaced.
char *repl_from = NULL;
char *repl_to = NULL;









// spell_move_to, decor_spell_nav_start, decor_spell_nav_col, can_syn_spell
// migrated to Rust: src/nvim-rs/spell/src/navigate.rs


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
