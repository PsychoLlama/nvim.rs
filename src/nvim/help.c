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
extern void rs_ex_helptags(void *eap);
extern void rs_get_local_additions(void);
extern void rs_helptags_one(char *dir, const char *ext, const char *tagfname,
                            bool add_help_tags, bool ignore_writeerr);
extern void rs_do_helptags(char *dirname, bool add_help_tags, bool ignore_writeerr);

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

// C accessors for Phase 6 (helptags + get_local_additions)
char *nvim_help_get_namebuff_mut(void) { return NameBuff; }
size_t nvim_help_get_namebuff_size(void) { return sizeof(NameBuff); }
const char *nvim_help_get_p_rtp(void) { return p_rtp; }
char *nvim_help_get_curbuf_fname(void) { return curbuf->b_fname; }
int nvim_help_get_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
void *nvim_help_get_curbuf_ptr(void) { return curbuf; }
bool nvim_help_get_got_int(void) { return got_int; }
const char *nvim_help_get_p_enc(void) { return p_enc; }

/// Wrap ExpandInit + ExpandOne for ex_helptags to avoid exposing expand_T.
char *nvim_help_expand_dir(const char *arg)
{
  expand_T xpc;
  ExpandInit(&xpc);
  xpc.xp_context = EXPAND_DIRECTORIES;
  return ExpandOne(&xpc, (char *)arg, NULL, WILD_LIST_NOTFOUND | WILD_SILENT, WILD_EXPAND_FREE);
}

/// Encapsulate encoding detection + conversion for get_local_additions.
/// Takes IObuff (already populated), detects utf-8 vs latin1, converts to
/// current 'encoding'. Returns IObuff if no conversion needed, or a newly
/// allocated string (caller must xfree) if conversion was done.
char *nvim_help_convert_help_line(char *buf)
{
  TriState this_utf = kNone;
  char *s = buf;
  while (*s != NUL) {
    if ((uint8_t)(*s) >= 0x80 && this_utf != kFalse) {
      this_utf = kTrue;
      int l = utf_ptr2len(s);
      if (l == 1) {
        this_utf = kFalse;
      }
      s += l - 1;
    }
    s++;
  }
  vimconv_T vc;
  vc.vc_type = CONV_NONE;
  convert_setup(&vc,
                (char *)(this_utf == kTrue ? "utf-8" : "latin1"),
                p_enc);
  char *cp;
  if (vc.vc_type == CONV_NONE) {
    cp = buf;
  } else {
    cp = string_convert(&vc, buf, NULL);
    if (cp == NULL) {
      cp = buf;
    }
  }
  convert_setup(&vc, NULL, NULL);
  return cp;
}

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
  rs_get_local_additions();
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
static void helptags_one(char *dir, const char *ext, const char *tagfname, bool add_help_tags,
                         bool ignore_writeerr)
  FUNC_ATTR_NONNULL_ALL
{
  rs_helptags_one(dir, ext, tagfname, add_help_tags, ignore_writeerr);
}

/// Generate tags in one help directory, taking care of translations.
static void do_helptags(char *dirname, bool add_help_tags, bool ignore_writeerr)
  FUNC_ATTR_NONNULL_ALL
{
  rs_do_helptags(dirname, add_help_tags, ignore_writeerr);
}

/// ":helptags"
void ex_helptags(exarg_T *eap)
{
  rs_ex_helptags(eap);
}
