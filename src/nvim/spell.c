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
// Phase 2+3: dump functions now implemented in Rust
extern void spell_dump_compl(char *pat, int ic, Direction *dir, int dumpflags_arg);
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

// spell_dump_compl, dump_word, and dump_prefixes are now implemented in Rust.
