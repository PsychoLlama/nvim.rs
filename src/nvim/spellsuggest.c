// spellsuggest.c: functions for spelling suggestions

#include <stdint.h>

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/input.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

/// Information used when looking for suggestions.
typedef struct {
  garray_T su_ga;                  ///< suggestions, contains "suggest_T"
  int su_maxcount;                 ///< max. number of suggestions displayed
  int su_maxscore;                 ///< maximum score for adding to su_ga
  int su_sfmaxscore;               ///< idem, for when doing soundfold words
  garray_T su_sga;                 ///< like su_ga, sound-folded scoring
  char *su_badptr;                 ///< start of bad word in line
  int su_badlen;                   ///< length of detected bad word in line
  int su_badflags;                 ///< caps flags for bad word
  char su_badword[MAXWLEN];        ///< bad word truncated at su_badlen
  char su_fbadword[MAXWLEN];       ///< su_badword case-folded
  char su_sal_badword[MAXWLEN];    ///< su_badword soundfolded
  hashtab_T su_banned;             ///< table with banned words
  slang_T *su_sallang;             ///< default language for sound folding
} suginfo_T;

/// One word suggestion.  Used in "si_ga".
typedef struct {
  char *st_word;      ///< suggested word, allocated string
  int st_wordlen;     ///< strlen(st_word)
  int st_orglen;      ///< length of replaced text
  int st_score;       ///< lower is better
  int st_altscore;    ///< used when st_score compares equal
  bool st_salscore;   ///< st_score is for soundalike
  bool st_had_bonus;  ///< bonus already included in score
  slang_T *st_slang;  ///< language used for sound folding
} suggest_T;

#define SUG(ga, i) (((suggest_T *)(ga).ga_data)[i])

// score for various changes
enum {
  SCORE_SPLIT = 149,     // split bad word
  SCORE_SPLIT_NO = 249,  // split bad word with NOSPLITSUGS
  SCORE_ICASE = 52,      // slightly different case
  SCORE_REGION = 200,    // word is for different region
  SCORE_RARE = 180,      // rare word
  SCORE_SWAP = 75,       // swap two characters
  SCORE_SWAP3 = 110,     // swap two characters in three
  SCORE_REP = 65,        // REP replacement
  SCORE_SUBST = 93,      // substitute a character
  SCORE_SIMILAR = 33,    // substitute a similar character
  SCORE_SUBCOMP = 33,    // substitute a composing character
  SCORE_DEL = 94,        // delete a character
  SCORE_DELDUP = 66,     // delete a duplicated character
  SCORE_DELCOMP = 28,    // delete a composing character
  SCORE_INS = 96,        // insert a character
  SCORE_INSDUP = 67,     // insert a duplicate character
  SCORE_INSCOMP = 30,    // insert a composing character
  SCORE_NONWORD = 103,   // change non-word to word char
};

enum {
  SCORE_FILE = 30,      // suggestion from a file
  SCORE_MAXINIT = 350,  // Initial maximum score: higher == slower.
                        // 350 allows for about three changes.
};

static int spell_suggest_timeout = 5000;

#include "spellsuggest.c.generated.h"

// values for sps_flags
enum {
  SPS_BEST = 1,
  SPS_FAST = 2,
  SPS_DOUBLE = 4,
};

static int sps_flags = SPS_BEST;  ///< flags from 'spellsuggest'
static int sps_limit = 9999;      ///< max nr of suggestions given

// C accessors for sps_flags and sps_limit (used by Rust via FFI).
void nvim_spellsug_set_sps_flags(int f) { sps_flags = f; }
void nvim_spellsug_set_sps_limit(int l) { sps_limit = l; }
int nvim_spellsug_get_sps_flags(void) { return sps_flags; }
int nvim_spellsug_get_sps_limit(void) { return sps_limit; }


// Accessor for the HLF_COUNT sentinel value (used by rs_check_suggestions).
int nvim_hlf_count(void) { return (int)HLF_COUNT; }

// Accessors for suginfo_T fields (used by rs_score_combine).
// Use void* for suginfo_T* to avoid exposing the type in the generated header.
garray_T *nvim_suginfo_get_ga(void *su) { return &((suginfo_T *)su)->su_ga; }
garray_T *nvim_suginfo_get_sga(void *su) { return &((suginfo_T *)su)->su_sga; }
const char *nvim_suginfo_get_fbadword(void *su) { return ((suginfo_T *)su)->su_fbadword; }
const char *nvim_suginfo_get_badword(void *su) { return ((suginfo_T *)su)->su_badword; }
const char *nvim_suginfo_get_badptr(void *su) { return ((suginfo_T *)su)->su_badptr; }
int nvim_suginfo_get_maxscore(void *su) { return ((suginfo_T *)su)->su_maxscore; }
int nvim_suginfo_get_maxcount(void *su) { return ((suginfo_T *)su)->su_maxcount; }
void nvim_suginfo_set_ga(void *su, garray_T ga) { ((suginfo_T *)su)->su_ga = ga; }
int nvim_suginfo_get_badlen(void *su) { return ((suginfo_T *)su)->su_badlen; }
// Phase 1 accessors
int nvim_suginfo_get_sfmaxscore(void *su) { return ((suginfo_T *)su)->su_sfmaxscore; }
void nvim_suginfo_set_sfmaxscore(void *su, int v) { ((suginfo_T *)su)->su_sfmaxscore = v; }
void nvim_suginfo_set_maxscore(void *su, int v) { ((suginfo_T *)su)->su_maxscore = v; }
hashtab_T *nvim_suginfo_get_banned(void *su) { return &((suginfo_T *)su)->su_banned; }
const char *nvim_suginfo_get_sal_badword(void *su) { return ((suginfo_T *)su)->su_sal_badword; }
slang_T *nvim_suginfo_get_sallang(void *su) { return ((suginfo_T *)su)->su_sallang; }
int nvim_suginfo_get_badflags(void *su) { return ((suginfo_T *)su)->su_badflags; }
void nvim_suginfo_set_badflags(void *su, int v) { ((suginfo_T *)su)->su_badflags = v; }
// Phase 2 accessors
int nvim_spellsug_get_timeout(void) { return spell_suggest_timeout; }

extern int badword_captype(char *word, char *end);
extern int rs_spell_check_sps_full(const char *p_sps_val);
extern int rs_cleanup_suggestions(suggest_T *data, int *gap_len, int maxscore, int keep);
extern void rs_check_suggestions(suggest_T *data, int *gap_len, const char *su_badptr);
extern void rs_score_combine_lists(void *su);
extern void rs_add_suggestion(void *su, garray_T *gap, const char *goodword, int badlenarg,
                              int score, int altscore, bool had_bonus, slang_T *slang,
                              bool maxsf);
extern void rs_add_banned(void *su, char *word);
extern void rs_spell_suggest_intern(void *su, bool interactive);
extern void rs_spell_suggest_file(void *su, char *fname);

/// Check the 'spellsuggest' option.  Return FAIL if it's wrong.
/// Sets "sps_flags" and "sps_limit".
int spell_check_sps(void)
{
  return rs_spell_check_sps_full(p_sps);
}

/// "z=": Find badly spelled word under or after the cursor.
/// Give suggestions for the properly spelled word.
/// In Visual mode use the highlighted word as the bad word.
/// When "count" is non-zero use that suggestion.
void spell_suggest(int count)
{
  pos_T prev_cursor = curwin->w_cursor;
  char wcopy[MAXWLEN + 2];
  suginfo_T sug;
  suggest_T *stp;
  bool mouse_used = false;
  int selected = count;
  int badlen = 0;
  int msg_scroll_save = msg_scroll;
  const int wo_spell_save = curwin->w_p_spell;

  if (!curwin->w_p_spell) {
    parse_spelllang(curwin);
    curwin->w_p_spell = true;
  }

  if (*curwin->w_s->b_p_spl == NUL) {
    emsg(_(e_no_spell));
    return;
  }

  if (VIsual_active) {
    // Use the Visually selected text as the bad word.  But reject
    // a multi-line selection.
    if (curwin->w_cursor.lnum != VIsual.lnum) {
      vim_beep(kOptBoFlagSpell);
      return;
    }
    badlen = (int)curwin->w_cursor.col - (int)VIsual.col;
    if (badlen < 0) {
      badlen = -badlen;
    } else {
      curwin->w_cursor.col = VIsual.col;
    }
    badlen++;
    end_visual_mode();
    // make sure we don't include the NUL at the end of the line
    badlen = MIN(badlen, get_cursor_line_len() - curwin->w_cursor.col);
    // Find the start of the badly spelled word.
  } else if (spell_move_to(curwin, FORWARD, SMT_ALL, true, NULL) == 0
             || curwin->w_cursor.col > prev_cursor.col) {
    // No bad word or it starts after the cursor: use the word under the
    // cursor.
    curwin->w_cursor = prev_cursor;
    char *line = get_cursor_line_ptr();
    char *p = line + curwin->w_cursor.col;
    // Backup to before start of word.
    while (p > line && spell_iswordp_nmw(p, curwin)) {
      MB_PTR_BACK(line, p);
    }
    // Forward to start of word.
    while (*p != NUL && !spell_iswordp_nmw(p, curwin)) {
      MB_PTR_ADV(p);
    }

    if (!spell_iswordp_nmw(p, curwin)) {                // No word found.
      beep_flush();
      return;
    }
    curwin->w_cursor.col = (colnr_T)(p - line);
  }

  // Get the word and its length.

  // Figure out if the word should be capitalised.
  int need_cap = check_need_cap(curwin, curwin->w_cursor.lnum, curwin->w_cursor.col);

  // Make a copy of current line since autocommands may free the line.
  char *line = xstrnsave(get_cursor_line_ptr(), (size_t)get_cursor_line_len());
  spell_suggest_timeout = 5000;

  // Get the list of suggestions.  Limit to 'lines' - 2 or the number in
  // 'spellsuggest', whatever is smaller.
  int limit = MIN(sps_limit, Rows - 2);
  spell_find_suggest(line + curwin->w_cursor.col, badlen, &sug, limit,
                     true, need_cap, true);

  msg_ext_set_kind("confirm");
  if (GA_EMPTY(&sug.su_ga)) {
    msg(_("Sorry, no suggestions"), 0);
  } else if (count > 0) {
    if (count > sug.su_ga.ga_len) {
      smsg(0, _("Sorry, only %" PRId64 " suggestions"),
           (int64_t)sug.su_ga.ga_len);
    }
  } else {
    // When 'rightleft' is set the list is drawn right-left.
    cmdmsg_rl = curwin->w_p_rl;

    // List the suggestions.
    msg_start();
    msg_row = Rows - 1;         // for when 'cmdheight' > 1
    lines_left = Rows;          // avoid more prompt
    char *fmt = _("Change \"%.*s\" to:");
    if (cmdmsg_rl && strncmp(fmt, "Change", 6) == 0) {
      // And now the rabbit from the high hat: Avoid showing the
      // untranslated message rightleft.
      fmt = ":ot \"%.*s\" egnahC";
    }
    vim_snprintf(IObuff, IOSIZE, fmt, sug.su_badlen, sug.su_badptr);
    msg_puts(IObuff);
    msg_clr_eos();
    msg_putchar('\n');

    msg_scroll = true;
    for (int i = 0; i < sug.su_ga.ga_len; i++) {
      stp = &SUG(sug.su_ga, i);

      // The suggested word may replace only part of the bad word, add
      // the not replaced part.  But only when it's not getting too long.
      xstrlcpy(wcopy, stp->st_word, MAXWLEN + 1);
      int el = sug.su_badlen - stp->st_orglen;
      if (el > 0 && stp->st_wordlen + el <= MAXWLEN) {
        assert(sug.su_badptr != NULL);
        xmemcpyz(wcopy + stp->st_wordlen, sug.su_badptr + stp->st_orglen, (size_t)el);
      }
      vim_snprintf(IObuff, IOSIZE, "%2d", i + 1);
      if (cmdmsg_rl) {
        rl_mirror_ascii(IObuff, NULL);
      }
      msg_puts(IObuff);

      vim_snprintf(IObuff, IOSIZE, " \"%s\"", wcopy);
      msg_puts(IObuff);

      // The word may replace more than "su_badlen".
      if (sug.su_badlen < stp->st_orglen) {
        vim_snprintf(IObuff, IOSIZE, _(" < \"%.*s\""),
                     stp->st_orglen, sug.su_badptr);
        msg_puts(IObuff);
      }

      if (p_verbose > 0) {
        // Add the score.
        if (sps_flags & (SPS_DOUBLE | SPS_BEST)) {
          vim_snprintf(IObuff, IOSIZE, " (%s%d - %d)",
                       stp->st_salscore ? "s " : "",
                       stp->st_score, stp->st_altscore);
        } else {
          vim_snprintf(IObuff, IOSIZE, " (%d)",
                       stp->st_score);
        }
        if (cmdmsg_rl) {
          // Mirror the numbers, but keep the leading space.
          rl_mirror_ascii(IObuff + 1, NULL);
        }
        msg_advance(30);
        msg_puts(IObuff);
      }
      if (!ui_has(kUIMessages) || i < sug.su_ga.ga_len - 1) {
        msg_putchar('\n');
      }
    }

    cmdmsg_rl = false;
    msg_col = 0;
    // Ask for choice.
    selected = prompt_for_input(NULL, 0, false, &mouse_used);
    if (mouse_used) {
      selected = sug.su_ga.ga_len + 1 - (cmdline_row - mouse_row);
    }

    lines_left = Rows;                  // avoid more prompt
    // don't delay for 'smd' in normal_cmd()
    msg_scroll = msg_scroll_save;
  }

  if (selected > 0 && selected <= sug.su_ga.ga_len && u_save_cursor() == OK) {
    // Save the from and to text for :spellrepall.
    XFREE_CLEAR(repl_from);
    XFREE_CLEAR(repl_to);

    stp = &SUG(sug.su_ga, selected - 1);
    if (sug.su_badlen > stp->st_orglen) {
      // Replacing less than "su_badlen", append the remainder to
      // repl_to.
      repl_from = xstrnsave(sug.su_badptr, (size_t)sug.su_badlen);
      vim_snprintf(IObuff, IOSIZE, "%s%.*s", stp->st_word,
                   sug.su_badlen - stp->st_orglen,
                   sug.su_badptr + stp->st_orglen);
      repl_to = xstrdup(IObuff);
    } else {
      // Replacing su_badlen or more, use the whole word.
      repl_from = xstrnsave(sug.su_badptr, (size_t)stp->st_orglen);
      repl_to = xstrdup(stp->st_word);
    }

    // Replace the word.
    char *p = xmalloc(strlen(line) - (size_t)stp->st_orglen + (size_t)stp->st_wordlen + 1);
    int c = (int)(sug.su_badptr - line);
    memmove(p, line, (size_t)c);
    STRCPY(p + c, stp->st_word);
    strcat(p, sug.su_badptr + stp->st_orglen);

    // For redo we use a change-word command.
    ResetRedobuff();
    AppendToRedobuff("ciw");
    AppendToRedobuffLit(p + c,
                        stp->st_wordlen + sug.su_badlen - stp->st_orglen);
    AppendCharToRedobuff(ESC);

    // "p" may be freed here
    ml_replace(curwin->w_cursor.lnum, p, false);
    curwin->w_cursor.col = c;

    inserted_bytes(curwin->w_cursor.lnum, c, stp->st_orglen, stp->st_wordlen);
  } else {
    curwin->w_cursor = prev_cursor;
  }

  spell_find_cleanup(&sug);
  xfree(line);
  curwin->w_p_spell = wo_spell_save;
}

/// Find spell suggestions for "word".  Return them in the growarray "*gap" as
/// a list of allocated strings.
///
/// @param maxcount  maximum nr of suggestions
/// @param need_cap  'spellcapcheck' matched
void spell_suggest_list(garray_T *gap, char *word, int maxcount, bool need_cap, bool interactive)
{
  suginfo_T sug;

  spell_find_suggest(word, 0, &sug, maxcount, false, need_cap, interactive);

  // Make room in "gap".
  ga_init(gap, sizeof(char *), sug.su_ga.ga_len + 1);
  ga_grow(gap, sug.su_ga.ga_len);
  for (int i = 0; i < sug.su_ga.ga_len; i++) {
    suggest_T *stp = &SUG(sug.su_ga, i);

    // The suggested word may replace only part of "word", add the not
    // replaced part.
    char *wcopy = xmalloc((size_t)stp->st_wordlen + strlen(sug.su_badptr + stp->st_orglen) + 1);
    STRCPY(wcopy, stp->st_word);
    STRCPY(wcopy + stp->st_wordlen, sug.su_badptr + stp->st_orglen);
    ((char **)gap->ga_data)[gap->ga_len++] = wcopy;
  }

  spell_find_cleanup(&sug);
}

/// Find spell suggestions for the word at the start of "badptr".
/// Return the suggestions in "su->su_ga".
/// The maximum number of suggestions is "maxcount".
/// Note: does use info for the current window.
/// This is based on the mechanisms of Aspell, but completely reimplemented.
///
/// @param badlen  length of bad word or 0 if unknown
/// @param banbadword  don't include badword in suggestions
/// @param need_cap  word should start with capital
static void spell_find_suggest(char *badptr, int badlen, suginfo_T *su, int maxcount,
                               bool banbadword, bool need_cap, bool interactive)
{
  hlf_T attr = HLF_COUNT;
  char buf[MAXPATHL];
  bool do_combine = false;
  static bool expr_busy = false;
  bool did_intern = false;

  // Set the info in "*su".
  CLEAR_POINTER(su);
  ga_init(&su->su_ga, (int)sizeof(suggest_T), 10);
  ga_init(&su->su_sga, (int)sizeof(suggest_T), 10);
  if (*badptr == NUL) {
    return;
  }
  hash_init(&su->su_banned);

  su->su_badptr = badptr;
  if (badlen != 0) {
    su->su_badlen = badlen;
  } else {
    size_t tmplen = spell_check(curwin, su->su_badptr, &attr, NULL, false);
    assert(tmplen <= INT_MAX);
    su->su_badlen = (int)tmplen;
  }
  su->su_maxcount = maxcount;
  su->su_maxscore = SCORE_MAXINIT;
  su->su_badlen = MIN(su->su_badlen, MAXWLEN - 1);  // just in case
  xmemcpyz(su->su_badword, su->su_badptr, (size_t)su->su_badlen);
  spell_casefold(curwin, su->su_badptr, su->su_badlen, su->su_fbadword,
                 MAXWLEN);

  // TODO(vim): make this work if the case-folded text is longer than the
  // original text. Currently an illegal byte causes wrong pointer
  // computations.
  su->su_fbadword[su->su_badlen] = NUL;

  // get caps flags for bad word
  su->su_badflags = badword_captype(su->su_badptr,
                                    su->su_badptr + su->su_badlen);
  if (need_cap) {
    su->su_badflags |= WF_ONECAP;
  }

  // Find the default language for sound folding.  We simply use the first
  // one in 'spelllang' that supports sound folding.  That's good for when
  // using multiple files for one language, it's not that bad when mixing
  // languages (e.g., "pl,en").
  for (int i = 0; i < curbuf->b_s.b_langp.ga_len; i++) {
    langp_T *lp = LANGP_ENTRY(curbuf->b_s.b_langp, i);
    if (lp->lp_sallang != NULL) {
      su->su_sallang = lp->lp_sallang;
      break;
    }
  }

  // Soundfold the bad word with the default sound folding, so that we don't
  // have to do this many times.
  if (su->su_sallang != NULL) {
    spell_soundfold(su->su_sallang, su->su_fbadword, true,
                    su->su_sal_badword);
  }

  // If the word is not capitalised and spell_check() doesn't consider the
  // word to be bad then it might need to be capitalised.  Add a suggestion
  // for that.
  int c = utf_ptr2char(su->su_badptr);
  if (!SPELL_ISUPPER(c) && attr == HLF_COUNT) {
    make_case_word(su->su_badword, buf, WF_ONECAP);
    rs_add_suggestion(su, &su->su_ga, buf, su->su_badlen, SCORE_ICASE,
                      0, true, su->su_sallang, false);
  }

  // Ban the bad word itself.  It may appear in another region.
  if (banbadword) {
    rs_add_banned(su, su->su_badword);
  }

  // Make a copy of 'spellsuggest', because the expression may change it.
  char *sps_copy = xstrdup(p_sps);

  // Loop over the items in 'spellsuggest'.
  for (char *p = sps_copy; *p != NUL;) {
    copy_option_part(&p, buf, MAXPATHL, ",");

    if (strncmp(buf, "expr:", 5) == 0) {
      // Evaluate an expression.  Skip this when called recursively,
      // when using spellsuggest() in the expression.
      if (!expr_busy) {
        expr_busy = true;
        spell_suggest_expr(su, buf + 5);
        expr_busy = false;
      }
    } else if (strncmp(buf, "file:", 5) == 0) {
      // Use list of suggestions in a file.
      rs_spell_suggest_file(su, buf + 5);
    } else if (strncmp(buf, "timeout:", 8) == 0) {
      // Limit the time searching for suggestions.
      spell_suggest_timeout = atoi(buf + 8);
    } else if (!did_intern) {
      // Use internal method once.
      rs_spell_suggest_intern(su, interactive);
      if (sps_flags & SPS_DOUBLE) {
        do_combine = true;
      }
      did_intern = true;
    }
  }

  xfree(sps_copy);

  if (do_combine) {
    // Combine the two list of suggestions.  This must be done last,
    // because sorting changes the order again.
    rs_score_combine_lists(su);
  }
}

/// Find suggestions by evaluating expression "expr".
static void spell_suggest_expr(suginfo_T *su, char *expr)
{
  const char *p;

  // The work is split up in a few parts to avoid having to export
  // suginfo_T.
  // First evaluate the expression and get the resulting list.
  list_T *const list = eval_spell_expr(su->su_badword, expr);
  if (list != NULL) {
    // Loop over the items in the list.
    TV_LIST_ITER(list, li, {
      if (TV_LIST_ITEM_TV(li)->v_type == VAR_LIST) {
        // Get the word and the score from the items.
        int score = get_spellword(TV_LIST_ITEM_TV(li)->vval.v_list, &p);
        if (score >= 0 && score <= su->su_maxscore) {
          rs_add_suggestion(su, &su->su_ga, p, su->su_badlen,
                            score, 0, true, su->su_sallang, false);
        }
      }
    });
    tv_list_unref(list);
  }

  // Remove bogus suggestions, sort and truncate at "maxcount".
  rs_check_suggestions((suggest_T *)su->su_ga.ga_data, &su->su_ga.ga_len, su->su_badptr);
  rs_cleanup_suggestions((suggest_T *)su->su_ga.ga_data, &su->su_ga.ga_len,
                         su->su_maxscore, su->su_maxcount);
}

/// Free the info put in "*su" by spell_find_suggest().
static void spell_find_cleanup(suginfo_T *su)
{
#define FREE_SUG_WORD(sug) xfree((sug)->st_word)
  // Free the suggestions.
  GA_DEEP_CLEAR(&su->su_ga, suggest_T, FREE_SUG_WORD);
  GA_DEEP_CLEAR(&su->su_sga, suggest_T, FREE_SUG_WORD);

  // Free the banned words.
  hash_clear_all(&su->su_banned, 0);
}


