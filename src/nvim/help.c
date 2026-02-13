// help.c: functions for Vim help

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/help.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "help.c.generated.h"

extern int rs_help_heuristic(const char *matched_string, int offset, bool wrong_case);
extern char *rs_check_help_lang(char *arg);
extern int rs_help_compare(const void *s1, const void *s2);
extern int rs_find_help_tags(const char *arg, int *num_matches, char ***matches, bool keep_lang);
extern void rs_cleanup_help_tags(int num_file, char **file);
extern void rs_ex_exusage(void *eap);
extern void rs_ex_viusage(void *eap);
extern void rs_prepare_help_buffer(void);
extern void rs_ex_help(void *eap);
extern void rs_ex_helpclose(void *eap);

// C accessors for ex_help / ex_helpclose
char *nvim_help_eap_get_arg(exarg_T *eap) { return eap->arg; }
void nvim_help_eap_set_arg(exarg_T *eap, char *arg) { eap->arg = arg; }
void nvim_help_eap_set_nextcmd(exarg_T *eap, char *cmd) { eap->nextcmd = cmd; }
int nvim_help_eap_get_forceit(exarg_T *eap) { return eap->forceit; }
int nvim_help_eap_get_skip(exarg_T *eap) { return eap->skip; }

bool nvim_help_curbuf_is_help(void) { return curbuf->b_help; }
bool nvim_help_curwin_bt_help(void) { return bt_help(curwin->w_buffer); }
int nvim_help_get_cmdmod_tab(void) { return cmdmod.cmod_tab; }
int nvim_help_get_cmdmod_split(void) { return cmdmod.cmod_split; }
int nvim_help_get_cmdmod_flags(void) { return cmdmod.cmod_flags; }
int nvim_help_get_columns(void) { return Columns; }
int nvim_help_get_curwin_width(void) { return curwin->w_width; }
int nvim_help_get_curwin_height(void) { return curwin->w_height; }
int nvim_help_get_p_sb(void) { return p_sb; }
int64_t nvim_help_get_p_hh(void) { return p_hh; }
const char *nvim_help_get_p_hf(void) { return p_hf; }

bool nvim_help_get_KeyTyped(void) { return KeyTyped; }
void nvim_help_set_KeyTyped(bool val) { KeyTyped = val; }
int nvim_help_get_restart_edit(void) { return restart_edit; }
void nvim_help_set_restart_edit(int val) { restart_edit = val; }

int nvim_help_get_curbuf_fnum(void) { return curbuf->b_fnum; }
int nvim_help_get_curwin_alt_fnum(void) { return curwin->w_alt_fnum; }
void nvim_help_set_curwin_alt_fnum(int fnum) { curwin->w_alt_fnum = fnum; }

// Find a help window in current tab
void *nvim_help_find_help_win_in_tab(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp2, curtab) {
    if (bt_help(wp2->w_buffer) && !wp2->w_config.hide && wp2->w_config.focusable) {
      return wp2;
    }
  }
  return NULL;
}

int nvim_help_win_nwindows(void *wp)
{
  return ((win_T *)wp)->w_buffer->b_nwindows;
}

// Wrappers for functions needing complex types
int nvim_help_do_ecmd_help(void)
{
  return do_ecmd(0, NULL, NULL, NULL, ECMD_LASTL,
                 ECMD_HIDE + ECMD_SET_HELP, NULL);
}

int nvim_help_buf_nwindows(buf_T *buf) { return buf->b_nwindows; }

// C accessors for prepare_help_buffer
void nvim_help_set_curbuf_b_help(bool val) { curbuf->b_help = val; }
const char *nvim_help_get_curbuf_b_p_isk(void) { return curbuf->b_p_isk; }
void nvim_help_set_buftype_help(void)
{
  set_option_direct(kOptBuftype, STATIC_CSTR_AS_OPTVAL("help"), OPT_LOCAL, 0);
}
void nvim_help_set_isk_help(const char *p)
{
  set_option_direct(kOptIskeyword, CSTR_AS_OPTVAL(p), OPT_LOCAL, 0);
  check_buf_options(curbuf);
  buf_init_chartab(curbuf, false);
}
void nvim_help_set_foldmethod_manual(void)
{
  set_option_direct(kOptFoldmethod, STATIC_CSTR_AS_OPTVAL("manual"), OPT_LOCAL, 0);
}
void nvim_help_set_buf_fields(void)
{
  curbuf->b_p_ts = 8;
  curbuf->b_p_ma = false;
  curbuf->b_p_bin = false;
}
void nvim_help_set_win_help_options(void)
{
  curwin->w_p_list = false;
  curwin->w_p_nu = 0;
  curwin->w_p_rnu = 0;
  RESET_BINDING(curwin);
  curwin->w_p_arab = false;
  curwin->w_p_rl = false;
  curwin->w_p_fen = false;
  curwin->w_p_diff = false;
  curwin->w_p_spell = false;
}

// C accessor for 'helplang' option
const char *nvim_help_get_p_hlg(void) { return p_hlg; }

/// ":help": open a read-only window on a help file
void ex_help(exarg_T *eap)
{
  rs_ex_help(eap);
}

/// ":helpclose": Close one help window
void ex_helpclose(exarg_T *eap)
{
  rs_ex_helpclose(eap);
}

/// In an argument search for a language specifiers in the form "@xx".
/// Changes the "@" to NUL if found, and returns a pointer to "xx".
///
/// @return  NULL if not found.
char *check_help_lang(char *arg)
{
  return rs_check_help_lang(arg);
}

/// Return a heuristic indicating how well the given string matches.  The
/// smaller the number, the better the match.  This is the order of priorities,
/// from best match to worst match:
///      - Match with least alphanumeric characters is better.
///      - Match with least total characters is better.
///      - Match towards the start is better.
///      - Match starting with "+" is worse (feature instead of command)
/// Assumption is made that the matched_string passed has already been found to
/// match some string for which help is requested.  webb.
///
/// @param offset      offset for match
/// @param wrong_case  no matching case
///
/// @return  a heuristic indicating how well the given string matches.
int help_heuristic(char *matched_string, int offset, bool wrong_case)
  FUNC_ATTR_PURE
{
  return rs_help_heuristic(matched_string, offset, wrong_case);
}

/// Compare functions for qsort() below, that checks the help heuristics number
/// that has been put after the tagname by find_tags().
static int help_compare(const void *s1, const void *s2)
{
  return rs_help_compare(s1, s2);
}

/// Find all help tags matching "arg", sort them and return in matches[], with
/// the number of matches in num_matches.
/// The matches will be sorted with a "best" match algorithm.
/// When "keep_lang" is true try keeping the language of the current buffer.
int find_help_tags(const char *arg, int *num_matches, char ***matches, bool keep_lang)
{
  return rs_find_help_tags(arg, num_matches, matches, keep_lang);
}

// The original find_help_tags implementation has been migrated to Rust (rs_find_help_tags).
// See src/nvim-rs/help/src/lib.rs.
#if 0
  static char *(except_tbl[][2]) = {
    { "*",           "star" },
    { "g*",          "gstar" },
    { "[*",          "[star" },
    { "]*",          "]star" },
    { ":*",          ":star" },
    { "/*",          "/star" },  // NOLINT
    { "/\\*",        "/\\\\star" },
    { "\"*",         "quotestar" },
    { "**",          "starstar" },
    { "cpo-*",       "cpo-star" },
    { "/\\(\\)",     "/\\\\(\\\\)" },
    { "/\\%(\\)",    "/\\\\%(\\\\)" },
    { "?",           "?" },
    { "??",          "??" },
    { ":?",          ":?" },
    { "?<CR>",       "?<CR>" },
    { "g?",          "g?" },
    { "g?g?",        "g?g?" },
    { "g??",         "g??" },
    { "-?",          "-?" },
    { "q?",          "q?" },
    { "v_g?",        "v_g?" },
    { "/\\?",        "/\\\\?" },
    { "/\\z(\\)",    "/\\\\z(\\\\)" },
    { "\\=",         "\\\\=" },
    { ":s\\=",       ":s\\\\=" },
    { "[count]",     "\\[count]" },
    { "[quotex]",    "\\[quotex]" },
    { "[range]",     "\\[range]" },
    { ":[range]",    ":\\[range]" },
    { "[pattern]",   "\\[pattern]" },
    { "\\|",         "\\\\bar" },
    { "\\%$",        "/\\\\%\\$" },
    { "s/\\~",       "s/\\\\\\~" },
    { "s/\\U",       "s/\\\\U" },
    { "s/\\L",       "s/\\\\L" },
    { "s/\\1",       "s/\\\\1" },
    { "s/\\2",       "s/\\\\2" },
    { "s/\\3",       "s/\\\\3" },
    { "s/\\9",       "s/\\\\9" },
    { NULL, NULL }
  };

  static const char *(expr_table[]) = {
    "!=?", "!~?", "<=?", "<?", "==?", "=~?",
    ">=?", ">?", "is?", "isnot?"
  };
  char *d = IObuff;       // assume IObuff is long enough!
  d[0] = NUL;

  if (STRNICMP(arg, "expr-", 5) == 0) {
    // When the string starting with "expr-" and containing '?' and matches
    // the table, it is taken literally (but ~ is escaped).  Otherwise '?'
    // is recognized as a wildcard.
    for (int i = (int)ARRAY_SIZE(expr_table); --i >= 0;) {
      if (strcmp(arg + 5, expr_table[i]) == 0) {
        for (int si = 0, di = 0;; si++) {
          if (arg[si] == '~') {
            d[di++] = '\\';
          }
          d[di++] = arg[si];
          if (arg[si] == NUL) {
            break;
          }
        }
        break;
      }
    }
  } else {
    // Recognize a few exceptions to the rule.  Some strings that contain
    // '*'are changed to "star", otherwise '*' is recognized as a wildcard.
    for (int i = 0; except_tbl[i][0] != NULL; i++) {
      if (strcmp(arg, except_tbl[i][0]) == 0) {
        STRCPY(d, except_tbl[i][1]);
        break;
      }
    }
  }

  if (d[0] == NUL) {  // no match in table
    // Replace "\S" with "/\\S", etc.  Otherwise every tag is matched.
    // Also replace "\%^" and "\%(", they match every tag too.
    // Also "\zs", "\z1", etc.
    // Also "\@<", "\@=", "\@<=", etc.
    // And also "\_$" and "\_^".
    if (arg[0] == '\\'
        && ((arg[1] != NUL && arg[2] == NUL)
            || (vim_strchr("%_z@", (uint8_t)arg[1]) != NULL
                && arg[2] != NUL))) {
      vim_snprintf(d, IOSIZE, "/\\\\%s", arg + 1);
      // Check for "/\\_$", should be "/\\_\$"
      if (d[3] == '_' && d[4] == '$') {
        STRCPY(d + 4, "\\$");
      }
    } else {
      // Replace:
      // "[:...:]" with "\[:...:]"
      // "[++...]" with "\[++...]"
      // "\{" with "\\{"               -- matching "} \}"
      if ((arg[0] == '[' && (arg[1] == ':'
                             || (arg[1] == '+' && arg[2] == '+')))
          || (arg[0] == '\\' && arg[1] == '{')) {
        *d++ = '\\';
      }

      // If tag starts with "('", skip the "(". Fixes CTRL-] on ('option'.
      if (*arg == '(' && arg[1] == '\'') {
        arg++;
      }
      for (const char *s = arg; *s; s++) {
        // Replace "|" with "bar" and '"' with "quote" to match the name of
        // the tags for these commands.
        // Replace "*" with ".*" and "?" with "." to match command line
        // completion.
        // Insert a backslash before '~', '$' and '.' to avoid their
        // special meaning.
        if (d - IObuff > IOSIZE - 10) {           // getting too long!?
          break;
        }
        switch (*s) {
        case '|':
          STRCPY(d, "bar");
          d += 3;
          continue;
        case '"':
          STRCPY(d, "quote");
          d += 5;
          continue;
        case '*':
          *d++ = '.';
          break;
        case '?':
          *d++ = '.';
          continue;
        case '$':
        case '.':
        case '~':
          *d++ = '\\';
          break;
        }

        // Replace "^x" by "CTRL-X". Don't do this for "^_" to make
        // ":help i_^_CTRL-D" work.
        // Insert '-' before and after "CTRL-X" when applicable.
        if ((uint8_t)(*s) < ' '
            || (*s == '^' && s[1]
                && (ASCII_ISALPHA(s[1]) || vim_strchr("?@[\\]^", (uint8_t)s[1]) != NULL))) {
          if (d > IObuff && d[-1] != '_' && d[-1] != '\\') {
            *d++ = '_';                 // prepend a '_' to make x_CTRL-x
          }
          STRCPY(d, "CTRL-");
          d += 5;
          if (*s < ' ') {
            *d++ = (char)(*s + '@');
            if (d[-1] == '\\') {
              *d++ = '\\';              // double a backslash
            }
          } else {
            *d++ = *++s;
          }
          if (s[1] != NUL && s[1] != '_') {
            *d++ = '_';                 // append a '_'
          }
          continue;
        } else if (*s == '^') {         // "^" or "CTRL-^" or "^_"
          *d++ = '\\';
        } else if (s[0] == '\\' && s[1] != '\\' && *arg == '/' && s == arg + 1) {
          // Insert a backslash before a backslash after a slash, for search
          // pattern tags: "/\|" --> "/\\|".
          *d++ = '\\';
        }

        // "CTRL-\_" -> "CTRL-\\_" to avoid the special meaning of "\_" in
        // "CTRL-\_CTRL-N"
        if (STRNICMP(s, "CTRL-\\_", 7) == 0) {
          STRCPY(d, "CTRL-\\\\");
          d += 7;
          s += 6;
        }

        *d++ = *s;

        // If tag contains "({" or "([", tag terminates at the "(".
        // This is for help on functions, e.g.: abs({expr}).
        if (*s == '(' && (s[1] == '{' || s[1] == '[')) {
          break;
        }

        // If tag starts with ', toss everything after a second '. Fixes
        // CTRL-] on 'option'. (would include the trailing '.').
        if (*s == '\'' && s > arg && *arg == '\'') {
          break;
        }
        // Also '{' and '}'. Fixes CTRL-] on '{address}'.
        if (*s == '}' && s > arg && *arg == '{') {
          break;
        }
      }
      *d = NUL;

      if (*IObuff == '`') {
        if (d > IObuff + 2 && d[-1] == '`') {
          // remove the backticks from `command`
          memmove(IObuff, IObuff + 1, strlen(IObuff));
          d[-2] = NUL;
        } else if (d > IObuff + 3 && d[-2] == '`' && d[-1] == ',') {
          // remove the backticks and comma from `command`,
          memmove(IObuff, IObuff + 1, strlen(IObuff));
          d[-3] = NUL;
        } else if (d > IObuff + 4 && d[-3] == '`'
                   && d[-2] == '\\' && d[-1] == '.') {
          // remove the backticks and dot from `command`\.
          memmove(IObuff, IObuff + 1, strlen(IObuff));
          d[-4] = NUL;
        }
      }
    }
  }

  *matches = NULL;
  *num_matches = 0;
  int flags = TAG_HELP | TAG_REGEXP | TAG_NAMES | TAG_VERBOSE | TAG_NO_TAGFUNC;
  if (keep_lang) {
    flags |= TAG_KEEP_LANG;
  }
  if (find_tags(IObuff, num_matches, matches, flags, MAXCOL, NULL) == OK
      && *num_matches > 0) {
    // Sort the matches found on the heuristic number that is after the
    // tag name.
    qsort((void *)(*matches), (size_t)(*num_matches),
          sizeof(char *), help_compare);
    // Delete more than TAG_MANY to reduce the size of the listing.
    while (*num_matches > TAG_MANY) {
      xfree((*matches)[--*num_matches]);
    }
  }
  return OK;
}
#endif  // Migrated to Rust

/// Cleanup matches for help tags:
/// Remove "@ab" if the top of 'helplang' is "ab" and the language of the first
/// tag matches it.  Otherwise remove "@en" if "en" is the only language.
void cleanup_help_tags(int num_file, char **file)
{
  rs_cleanup_help_tags(num_file, file);
}

/// Called when starting to edit a buffer for a help file.
void prepare_help_buffer(void)
{
  rs_prepare_help_buffer();
}

/// After reading a help file: if help.txt, populate *local-additions*
void get_local_additions(void)
{
  // In the "help.txt" and "help.abx" file, add the locally added help
  // files.  This uses the very first line in the help file.
  char *const fname = path_tail(curbuf->b_fname);
  if (path_fnamecmp(fname, "help.txt") == 0
      || (path_fnamencmp(fname, "help.", 5) == 0
          && ASCII_ISALPHA(fname[5])
          && ASCII_ISALPHA(fname[6])
          && TOLOWER_ASC(fname[7]) == 'x'
          && fname[8] == NUL)) {
    for (linenr_T lnum = 1; lnum < curbuf->b_ml.ml_line_count; lnum++) {
      char *line = ml_get_buf(curbuf, lnum);
      if (strstr(line, "*local-additions*") == NULL) {
        continue;
      }

      int lnum_start = lnum;

      // Go through all directories in 'runtimepath', skipping
      // $VIMRUNTIME.
      char *p = p_rtp;
      while (*p != NUL) {
        copy_option_part(&p, NameBuff, MAXPATHL, ",");
        char *const rt = vim_getenv("VIMRUNTIME");
        if (rt != NULL
            && path_full_compare(rt, NameBuff, false, true) != kEqualFiles) {
          int fcount;
          char **fnames;
          vimconv_T vc;

          // Find all "doc/ *.txt" files in this directory.
          if (!add_pathsep(NameBuff)
              || xstrlcat(NameBuff, "doc/*.??[tx]",  // NOLINT
                          sizeof(NameBuff)) >= MAXPATHL) {
            emsg(_(e_fnametoolong));
            continue;
          }

          // Note: We cannot just do `&NameBuff` because it is a statically sized array
          //       so `NameBuff == &NameBuff` according to C semantics.
          char *buff_list[1] = { NameBuff };
          if (gen_expand_wildcards(1, buff_list, &fcount,
                                   &fnames, EW_FILE|EW_SILENT) == OK
              && fcount > 0) {
            char *s;
            char *cp;
            // If foo.abx is found use it instead of foo.txt in
            // the same directory.
            for (int i1 = 0; i1 < fcount; i1++) {
              const char *const f1 = fnames[i1];
              const char *const t1 = path_tail(f1);
              const char *const e1 = strrchr(t1, '.');
              if (e1 == NULL) {
                continue;
              }
              if (path_fnamecmp(e1, ".txt") != 0
                  && path_fnamecmp(e1, fname + 4) != 0) {
                // Not .txt and not .abx, remove it.
                XFREE_CLEAR(fnames[i1]);
                continue;
              }

              for (int i2 = i1 + 1; i2 < fcount; i2++) {
                const char *const f2 = fnames[i2];
                if (f2 == NULL) {
                  continue;
                }
                const char *const t2 = path_tail(f2);
                const char *const e2 = strrchr(t2, '.');
                if (e2 == NULL) {
                  continue;
                }
                if (e1 - f1 != e2 - f2
                    || path_fnamencmp(f1, f2, (size_t)(e1 - f1)) != 0) {
                  continue;
                }
                if (path_fnamecmp(e1, ".txt") == 0
                    && path_fnamecmp(e2, fname + 4) == 0) {
                  // use .abx instead of .txt
                  XFREE_CLEAR(fnames[i1]);
                }
              }
            }
            for (int fi = 0; fi < fcount; fi++) {
              if (fnames[fi] == NULL) {
                continue;
              }

              FILE *const fd = os_fopen(fnames[fi], "r");
              if (fd == NULL) {
                continue;
              }
              vim_fgets(IObuff, IOSIZE, fd);
              if (IObuff[0] == '*'
                  && (s = vim_strchr(IObuff + 1, '*'))
                  != NULL) {
                TriState this_utf = kNone;
                // Change tag definition to a
                // reference and remove <CR>/<NL>.
                IObuff[0] = '|';
                *s = '|';
                while (*s != NUL) {
                  if (*s == '\r' || *s == '\n') {
                    *s = NUL;
                  }
                  // The text is utf-8 when a byte
                  // above 127 is found and no
                  // illegal byte sequence is found.
                  if ((uint8_t)(*s) >= 0x80 && this_utf != kFalse) {
                    this_utf = kTrue;
                    const int l = utf_ptr2len(s);
                    if (l == 1) {
                      this_utf = kFalse;
                    }
                    s += l - 1;
                  }
                  s++;
                }
                // The help file is latin1 or utf-8;
                // conversion to the current
                // 'encoding' may be required.
                vc.vc_type = CONV_NONE;
                convert_setup(&vc,
                              (this_utf == kTrue ? "utf-8" : "latin1"),
                              p_enc);
                if (vc.vc_type == CONV_NONE) {
                  // No conversion needed.
                  cp = IObuff;
                } else {
                  // Do the conversion.  If it fails
                  // use the unconverted text.
                  cp = string_convert(&vc, IObuff, NULL);
                  if (cp == NULL) {
                    cp = IObuff;
                  }
                }
                convert_setup(&vc, NULL, NULL);

                ml_append(lnum, cp, 0, false);
                if (cp != IObuff) {
                  xfree(cp);
                }
                lnum++;
              }
              fclose(fd);
            }
            FreeWild(fcount, fnames);
          }
        }
        xfree(rt);
      }
      linenr_T appended = lnum - lnum_start;
      if (appended) {
        mark_adjust(lnum_start + 1, (linenr_T)MAXLNUM, appended, 0, kExtmarkUndo);
        changed_lines_redraw_buf(curbuf, lnum_start + 1, lnum_start + 1, appended);
      }
      break;
    }
  }
}

/// ":exusage"
void ex_exusage(exarg_T *eap)
{
  rs_ex_exusage(eap);
}

/// ":viusage"
void ex_viusage(exarg_T *eap)
{
  rs_ex_viusage(eap);
}

/// Generate tags in one help directory
///
/// @param dir  Path to the doc directory
/// @param ext  Suffix of the help files (".txt", ".itx", ".frx", etc.)
/// @param tagname  Name of the tags file ("tags" for English, "tags-fr" for
///                 French)
/// @param add_help_tags  Whether to add the "help-tags" tag
/// @param ignore_writeerr  ignore write error
static void helptags_one(char *dir, const char *ext, const char *tagfname, bool add_help_tags,
                         bool ignore_writeerr)
  FUNC_ATTR_NONNULL_ALL
{
  garray_T ga;
  int filecount;
  char **files;
  char *s;

  // Find all *.txt files.
  size_t dirlen = xstrlcpy(NameBuff, dir, sizeof(NameBuff));
  if (dirlen >= MAXPATHL
      || xstrlcat(NameBuff, "/**/*", sizeof(NameBuff)) >= MAXPATHL  // NOLINT
      || xstrlcat(NameBuff, ext, sizeof(NameBuff)) >= MAXPATHL) {
    emsg(_(e_fnametoolong));
    return;
  }

  // Note: We cannot just do `&NameBuff` because it is a statically sized array
  //       so `NameBuff == &NameBuff` according to C semantics.
  char *buff_list[1] = { NameBuff };
  const int res = gen_expand_wildcards(1, buff_list, &filecount, &files,
                                       EW_FILE|EW_SILENT);
  if (res == FAIL || filecount == 0) {
    if (!got_int) {
      semsg(_("E151: No match: %s"), NameBuff);
    }
    if (res != FAIL) {
      FreeWild(filecount, files);
    }
    return;
  }

  // Open the tags file for writing.
  // Do this before scanning through all the files.
  memcpy(NameBuff, dir, dirlen + 1);
  if (!add_pathsep(NameBuff)
      || xstrlcat(NameBuff, tagfname, sizeof(NameBuff)) >= MAXPATHL) {
    emsg(_(e_fnametoolong));
    return;
  }

  FILE *const fd_tags = os_fopen(NameBuff, "w");
  if (fd_tags == NULL) {
    if (!ignore_writeerr) {
      semsg(_("E152: Cannot open %s for writing"), NameBuff);
    }
    FreeWild(filecount, files);
    return;
  }

  // If using the "++t" argument or generating tags for "$VIMRUNTIME/doc"
  // add the "help-tags" tag.
  ga_init(&ga, (int)sizeof(char *), 100);
  if (add_help_tags
      || path_full_compare("$VIMRUNTIME/doc", dir, false, true) == kEqualFiles) {
    size_t s_len = 18 + strlen(tagfname);
    s = xmalloc(s_len);
    snprintf(s, s_len, "help-tags\t%s\t1\n", tagfname);
    GA_APPEND(char *, &ga, s);
  }

  // Go over all the files and extract the tags.
  for (int fi = 0; fi < filecount && !got_int; fi++) {
    FILE *const fd = os_fopen(files[fi], "r");
    if (fd == NULL) {
      semsg(_("E153: Unable to open %s for reading"), files[fi]);
      continue;
    }
    const char *const fname = files[fi] + dirlen + 1;

    bool in_example = false;
    while (!vim_fgets(IObuff, IOSIZE, fd) && !got_int) {
      if (in_example) {
        // skip over example; a non-white in the first column ends it
        if (vim_strchr(" \t\n\r", (uint8_t)IObuff[0])) {
          continue;
        }
        in_example = false;
      }
      char *p1 = vim_strchr(IObuff, '*');       // find first '*'
      while (p1 != NULL) {
        char *p2 = strchr(p1 + 1, '*');  // Find second '*'.
        if (p2 != NULL && p2 > p1 + 1) {         // Skip "*" and "**".
          for (s = p1 + 1; s < p2; s++) {
            if (*s == ' ' || *s == '\t' || *s == '|') {
              break;
            }
          }

          // Only accept a *tag* when it consists of valid
          // characters, there is white space before it and is
          // followed by a white character or end-of-line.
          if (s == p2
              && (p1 == IObuff || p1[-1] == ' ' || p1[-1] == '\t')
              && (vim_strchr(" \t\n\r", (uint8_t)s[1]) != NULL
                  || s[1] == NUL)) {
            *p2 = NUL;
            p1++;
            size_t s_len = (size_t)(p2 - p1) + strlen(fname) + 2;
            s = xmalloc(s_len);
            GA_APPEND(char *, &ga, s);
            snprintf(s, s_len, "%s\t%s", p1, fname);

            // find next '*'
            p2 = vim_strchr(p2 + 1, '*');
          }
        }
        p1 = p2;
      }
      size_t off = strlen(IObuff);
      if (off >= 2 && IObuff[off - 1] == '\n') {
        off -= 2;
        while (off > 0 && (ASCII_ISLOWER(IObuff[off]) || ascii_isdigit(IObuff[off]))) {
          off--;
        }
        if (IObuff[off] == '>' && (off == 0 || IObuff[off - 1] == ' ')) {
          in_example = true;
        }
      }
      line_breakcheck();
    }

    fclose(fd);
  }

  FreeWild(filecount, files);

  if (!got_int && ga.ga_data != NULL) {
    // Sort the tags.
    sort_strings(ga.ga_data, ga.ga_len);

    // Check for duplicates.
    for (int i = 1; i < ga.ga_len; i++) {
      char *p1 = ((char **)ga.ga_data)[i - 1];
      char *p2 = ((char **)ga.ga_data)[i];
      while (*p1 == *p2) {
        if (*p2 == '\t') {
          *p2 = NUL;
          vim_snprintf(NameBuff, MAXPATHL,
                       _("E154: Duplicate tag \"%s\" in file %s/%s"),
                       ((char **)ga.ga_data)[i], dir, p2 + 1);
          emsg(NameBuff);
          *p2 = '\t';
          break;
        }
        p1++;
        p2++;
      }
    }

    // Write the tags into the file.
    for (int i = 0; i < ga.ga_len; i++) {
      s = ((char **)ga.ga_data)[i];
      if (strncmp(s, "help-tags\t", 10) == 0) {
        // help-tags entry was added in formatted form
        fputs(s, fd_tags);
      } else {
        fprintf(fd_tags, "%s\t/" "*", s);
        for (char *p1 = s; *p1 != '\t'; p1++) {
          // insert backslash before '\\' and '/'
          if (*p1 == '\\' || *p1 == '/') {
            putc('\\', fd_tags);
          }
          putc(*p1, fd_tags);
        }
        fprintf(fd_tags, "*\n");
      }
    }
  }

  GA_DEEP_CLEAR_PTR(&ga);
  fclose(fd_tags);          // there is no check for an error...
}

/// Generate tags in one help directory, taking care of translations.
static void do_helptags(char *dirname, bool add_help_tags, bool ignore_writeerr)
  FUNC_ATTR_NONNULL_ALL
{
  garray_T ga;
  char lang[2];
  char ext[5];
  char fname[8];
  int filecount;
  char **files;

  // Get a list of all files in the help directory and in subdirectories.
  xstrlcpy(NameBuff, dirname, sizeof(NameBuff));
  if (!add_pathsep(NameBuff)
      || xstrlcat(NameBuff, "**", sizeof(NameBuff)) >= MAXPATHL) {
    emsg(_(e_fnametoolong));
    return;
  }

  // Note: We cannot just do `&NameBuff` because it is a statically sized array
  //       so `NameBuff == &NameBuff` according to C semantics.
  char *buff_list[1] = { NameBuff };
  if (gen_expand_wildcards(1, buff_list, &filecount, &files,
                           EW_FILE|EW_SILENT) == FAIL
      || filecount == 0) {
    semsg(_("E151: No match: %s"), NameBuff);
    return;
  }

  // Go over all files in the directory to find out what languages are
  // present.
  int j;
  ga_init(&ga, 1, 10);
  for (int i = 0; i < filecount; i++) {
    int len = (int)strlen(files[i]);
    if (len <= 4) {
      continue;
    }

    if (STRICMP(files[i] + len - 4, ".txt") == 0) {
      // ".txt" -> language "en"
      lang[0] = 'e';
      lang[1] = 'n';
    } else if (files[i][len - 4] == '.'
               && ASCII_ISALPHA(files[i][len - 3])
               && ASCII_ISALPHA(files[i][len - 2])
               && TOLOWER_ASC(files[i][len - 1]) == 'x') {
      // ".abx" -> language "ab"
      lang[0] = (char)TOLOWER_ASC(files[i][len - 3]);
      lang[1] = (char)TOLOWER_ASC(files[i][len - 2]);
    } else {
      continue;
    }

    // Did we find this language already?
    for (j = 0; j < ga.ga_len; j += 2) {
      if (strncmp(lang, ((char *)ga.ga_data) + j, 2) == 0) {
        break;
      }
    }
    if (j == ga.ga_len) {
      // New language, add it.
      ga_grow(&ga, 2);
      ((char *)ga.ga_data)[ga.ga_len++] = lang[0];
      ((char *)ga.ga_data)[ga.ga_len++] = lang[1];
    }
  }

  // Loop over the found languages to generate a tags file for each one.
  for (j = 0; j < ga.ga_len; j += 2) {
    STRCPY(fname, "tags-xx");
    fname[5] = ((char *)ga.ga_data)[j];
    fname[6] = ((char *)ga.ga_data)[j + 1];
    if (fname[5] == 'e' && fname[6] == 'n') {
      // English is an exception: use ".txt" and "tags".
      fname[4] = NUL;
      STRCPY(ext, ".txt");
    } else {
      // Language "ab" uses ".abx" and "tags-ab".
      STRCPY(ext, ".xxx");
      ext[1] = fname[5];
      ext[2] = fname[6];
    }
    helptags_one(dirname, ext, fname, add_help_tags, ignore_writeerr);
  }

  ga_clear(&ga);
  FreeWild(filecount, files);
}

static bool helptags_cb(int num_fnames, char **fnames, bool all, void *cookie)
  FUNC_ATTR_NONNULL_ALL
{
  for (int i = 0; i < num_fnames; i++) {
    do_helptags(fnames[i], *(bool *)cookie, true);
    if (!all) {
      return true;
    }
  }

  return num_fnames > 0;
}

/// ":helptags"
void ex_helptags(exarg_T *eap)
{
  expand_T xpc;
  bool add_help_tags = false;

  // Check for ":helptags ++t {dir}".
  if (strncmp(eap->arg, "++t", 3) == 0 && ascii_iswhite(eap->arg[3])) {
    add_help_tags = true;
    eap->arg = skipwhite(eap->arg + 3);
  }

  if (strcmp(eap->arg, "ALL") == 0) {
    do_in_path(p_rtp, "", "doc", DIP_ALL + DIP_DIR, helptags_cb, &add_help_tags);
  } else {
    ExpandInit(&xpc);
    xpc.xp_context = EXPAND_DIRECTORIES;
    char *dirname =
      ExpandOne(&xpc, eap->arg, NULL, WILD_LIST_NOTFOUND|WILD_SILENT, WILD_EXPAND_FREE);
    if (dirname == NULL || !os_isdir(dirname)) {
      semsg(_("E150: Not a directory: %s"), eap->arg);
    } else {
      do_helptags(dirname, add_help_tags, false);
    }
    xfree(dirname);
  }
}
