// regexp_shim.c -- FFI bridge between Neovim's C core and the Rust regexp engine.
//
// All regexp logic (matching, compilation, parsing, substitution) lives in
// src/nvim-rs/regexp/src/lib.rs. This file contains only:
//   - Struct/type definitions (regexec_T, bt_regprog_T, nfa_regprog_T, etc.)
//   - Static global variables (rex, regstack, parser state, etc.)
//   - Accessor functions that let Rust read/write C struct fields and globals
//   - Error message helpers wrapping emsg()/semsg() with gettext
//   - Engine vtable definitions (bt_regengine, nfa_regengine)
//   - Thin wrappers for public API functions that need C type casts

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

// Rust FFI: skip_regexp implementation
extern char *rs_partial_name(partial_T *pt);
extern char *rs_skip_regexp_ex(char *startp, int dirc, int magic, char **newp,
                               int *dropped, int *magic_val);
// Rust FFI: regexp utility functions (directly exported from Rust)
extern reg_extmatch_T *rs_make_extmatch(void);
extern void rs_cleanup_zsubexpr(void);
// Rust FFI: NFA execution engine entry points
extern int rs_nfa_regexec_nl(void *rmp, uint8_t *line, int32_t col, int line_lbr);
extern int rs_nfa_regexec_multi(void *rmp, void *win, void *buf, int32_t lnum,
                                int32_t col, void *tm, int *timed_out);
// Rust FFI: BT execution engine entry points
extern int rs_bt_regexec_nl(void *rmp, uint8_t *line, int32_t col, int line_lbr);
extern int rs_bt_regexec_multi(void *rmp, void *win, void *buf, int32_t lnum,
                               int32_t col, void *tm, int *timed_out);
extern int rs_vim_regexec(void *rmp, const uint8_t *line, int32_t col);
extern int rs_vim_regexec_nl(void *rmp, const uint8_t *line, int32_t col);
extern int rs_vim_regexec_prog(void **prog_ptr, int ignore_case, const uint8_t *line, int32_t col);
extern int rs_vim_regexec_multi(void *rmp, void *win, void *buf, int32_t lnum,
                                int32_t col, void *tm, int *timed_out);
extern void *rs_vim_regcomp(const uint8_t *expr, int re_flags);
extern void *rs_bt_regcomp(uint8_t *expr, int re_flags);
typedef enum {
  RGLF_LINE = 0x01,
  RGLF_LENGTH = 0x02,
  RGLF_SUBMATCH = 0x04,
} reg_getline_flags_T;

/// Structure returned by vim_regcomp() to pass on to vim_regexec().
/// This is the general structure. For the actual matcher, two specific
/// structures are used. See code below.
struct regprog {
  regengine_T *engine;
  unsigned regflags;
  unsigned re_engine;  ///< Automatic, backtracking or NFA engine.
  unsigned re_flags;   ///< Second argument for vim_regcomp().
  bool re_in_use;      ///< prog is being executed
};

/// Structure used by the back track matcher.
/// These fields are only to be used in regexp.c!
/// See regexp.c for an explanation.
typedef struct {
  // These four members implement regprog_T.
  regengine_T *engine;
  unsigned regflags;
  unsigned re_engine;
  unsigned re_flags;
  bool re_in_use;

  int regstart;
  uint8_t reganch;
  uint8_t *regmust;
  int regmlen;
  uint8_t reghasz;
  uint8_t program[];
} bt_regprog_T;

/// Structure representing a NFA state.
/// An NFA state may have no outgoing edge, when it is a NFA_MATCH state.
typedef struct nfa_state nfa_state_T;
struct nfa_state {
  int c;
  nfa_state_T *out;
  nfa_state_T *out1;
  int id;
  int lastlist[2];  ///< 0: normal, 1: recursive
  int val;
};

/// Structure used by the NFA matcher.
typedef struct {
  // These four members implement regprog_T.
  regengine_T *engine;
  unsigned regflags;
  unsigned re_engine;
  unsigned re_flags;
  bool re_in_use;

  nfa_state_T *start;   ///< points into state[]

  int reganch;          ///< pattern starts with ^
  int regstart;         ///< char at start of pattern
  uint8_t *match_text;  ///< plain text to match with

  int has_zend;         ///< pattern contains \ze
  int has_backref;      ///< pattern contains \1 .. \9
  int reghasz;
  char *pattern;
  int nsubexp;          ///< number of ()
  int nstate;
  nfa_state_T state[];
} nfa_regprog_T;

struct regengine {
  /// bt_regcomp or nfa_regcomp
  regprog_T *(*regcomp)(uint8_t *, int);
  /// bt_regfree or nfa_regfree
  void (*regfree)(regprog_T *);
  /// bt_regexec_nl or nfa_regexec_nl
  int (*regexec_nl)(regmatch_T *, uint8_t *, colnr_T, bool);
  /// bt_regexec_mult or nfa_regexec_mult
  int (*regexec_multi)(regmmatch_T *, win_T *, buf_T *, linenr_T, colnr_T, proftime_T *, int *);
};

// Structure used to save the current input state, when it needs to be
// restored after trying a match.  Used by reg_save() and reg_restore().
// Also stores the length of "backpos".
typedef struct {
  union {
    uint8_t *ptr;       // rex.input pointer, for single-line regexp
    lpos_T pos;        // rex.input pos, for multi-line regexp
  } rs_u;
  int rs_len;
} regsave_T;

// struct to save start/end pointer/position in for \(\)
typedef struct {
  union {
    uint8_t *ptr;
    lpos_T pos;
  } se_u;
} save_se_T;

typedef struct {
  int in_use;       ///< number of subexpr with useful info

  // When REG_MULTI is true list.multi is used, otherwise list.line.
  union {
    struct multipos {
      linenr_T start_lnum;
      linenr_T end_lnum;
      colnr_T start_col;
      colnr_T end_col;
    } multi[NSUBEXP];
    struct linepos {
      uint8_t *start;
      uint8_t *end;
    } line[NSUBEXP];
  } list;
  colnr_T orig_start_col;  // list.multi[0].start_col without \zs
} regsub_T;

typedef struct {
  regsub_T norm;      // \( .. \) matches
  regsub_T synt;      // \z( .. \) matches
} regsubs_T;

// nfa_pim_T stores a Postponed Invisible Match.
typedef struct nfa_pim_S nfa_pim_T;
struct nfa_pim_S {
  int result;                   // NFA_PIM_*, see below
  nfa_state_T *state;           // the invisible match start state
  regsubs_T subs;               // submatch info, only party used
  union {
    lpos_T pos;
    uint8_t *ptr;
  } end;                        // where the match must end
};

// nfa_thread_T contains execution information of a NFA state
typedef struct {
  nfa_state_T *state;
  int count;
  nfa_pim_T pim;                // if pim.result != 0 (NFA_PIM_UNUSED): postponed
                                // invisible match
  regsubs_T subs;               // submatch info, only party used
} nfa_thread_T;

// nfa_list_T contains the alternative NFA execution states.
typedef struct {
  nfa_thread_T *t;           ///< allocated array of states
  int n;                        ///< nr of states currently in "t"
  int len;                      ///< max nr of states in "t"
  int id;                       ///< ID of the list
  int has_pim;                  ///< true when any state has a PIM
} nfa_list_T;

// The first byte of the BT regexp internal "program" is actually this magic
// number; the start node begins in the second byte.  It's used to catch the
// most severe mutilation of the program by the caller.
#define REGMAGIC        0234

// Utility definitions.
#define UCHARAT(p)      ((int)(*(uint8_t *)(p)))

static const char e_invalid_character_after_str_at[]
  = N_("E59: Invalid character after %s@");
static const char e_invalid_use_of_underscore[]
  = N_("E63: Invalid use of \\_");
static const char e_pattern_uses_more_memory_than_maxmempattern[]
  = N_("E363: Pattern uses more memory than 'maxmempattern'");
static const char e_invalid_item_in_str_brackets[]
  = N_("E369: Invalid item in %s%%[]");
static const char e_missing_delimiter_after_search_pattern_str[]
  = N_("E654: Missing delimiter after search pattern: %s");
static const char e_missingbracket[] = N_("E769: Missing ] after %s[");
static const char e_reverse_range[] = N_("E944: Reverse range in character class");
static const char e_large_class[] = N_("E945: Range too large in character class");
static const char e_unmatchedpp[] = N_("E53: Unmatched %s%%(");
static const char e_unmatchedp[] = N_("E54: Unmatched %s(");
static const char e_unmatchedpar[] = N_("E55: Unmatched %s)");
static const char e_z_not_allowed[] = N_("E66: \\z( not allowed here");
static const char e_z1_not_allowed[] = N_("E67: \\z1 - \\z9 not allowed here");
static const char e_missing_sb[] = N_("E69: Missing ] after %s%%[");
static const char e_empty_sb[] = N_("E70: Empty %s%%[]");
static const char e_recursive[] = N_("E956: Cannot use pattern recursively");
static const char e_regexp_number_after_dot_pos_search_chr[]
  = N_("E1204: No Number allowed after .: '\\%%%c'");
static const char e_nfa_regexp_missing_value_in_chr[]
  = N_("E1273: (NFA regexp) missing value in '\\%%%c'");
static const char e_atom_engine_must_be_at_start_of_pattern[]
  = N_("E1281: Atom '\\%%#=%c' must be at the start of the pattern");
static const char e_substitute_nesting_too_deep[] = N_("E1290: substitute nesting too deep");
static const char e_unicode_val_too_large[]
  = N_("E1541: Value too large, max Unicode codepoint is U+10FFFF");

static char *reg_prev_sub = NULL;
static size_t reg_prev_sublen = 0;

// regtilde accessors for Rust FFI
char *nvim_regexp_get_reg_prev_sub(void) { return reg_prev_sub; }
void nvim_regexp_set_reg_prev_sub(char *p) { reg_prev_sub = p; }
size_t nvim_regexp_get_reg_prev_sublen(void) { return reg_prev_sublen; }
void nvim_regexp_set_reg_prev_sublen(size_t v) { reg_prev_sublen = v; }
void nvim_regexp_emsg_resulting_text_too_long(void) { emsg(_(e_resulting_text_too_long)); }

// REGEXP_INRANGE contains all characters which are always special in a []
// range after '\'.
// REGEXP_ABBR contains all characters which act as abbreviations after '\'.
// These are:
//  \n  - New line (NL).
//  \r  - Carriage Return (CR).
//  \t  - Tab (TAB).
//  \e  - Escape (ESC).
//  \b  - Backspace (Ctrl_H).
//  \d  - Character code in decimal, eg \d123
//  \o  - Character code in octal, eg \o80
//  \x  - Character code in hex, eg \x4a
//  \u  - Multibyte character code, eg \u20ac
//  \U  - Long multibyte character code, eg \U12345678
static char REGEXP_INRANGE[] = "]^-n\\";
static char REGEXP_ABBR[] = "nrtebdoxuU";

/// Check for a character class name "[:name:]".  "pp" points to the '['.
/// Returns one of the CLASS_ items. CLASS_NONE means that no item was
/// recognized.  Otherwise "pp" is advanced to after the item.
extern int rs_get_char_class(char **pp);

// flags for regflags
#define RF_ICASE    1   // ignore case
#define RF_NOICASE  2   // don't ignore case
#define RF_ICOMBINE 8   // ignore combining characters

static regengine_T bt_regengine;
static regengine_T nfa_regengine;

#include "regexp_shim.c.generated.h"

// Accessors for Rust FFI (static helpers exposed for the regexp crate)
int nvim_regexp_get_char_class(char **pp) { return rs_get_char_class(pp); }

unsigned int nvim_regexp_get_regflags(const regprog_T *prog) { return prog->regflags; }

void nvim_regexp_semsg_e654(const char *startp) { semsg(_(e_missing_delimiter_after_search_pattern_str), startp); }

/// Skip strings inside [ and ].
char *skip_regexp(char *startp, int delim, int magic)
{
  return skip_regexp_ex(startp, delim, magic, NULL, NULL, NULL);
}

/// skip_regexp() with extra arguments:
/// When "newp" is not NULL and "dirc" is '?', make an allocated copy of the
/// expression and change "\?" to "?".  If "*newp" is not NULL the expression
/// is changed in-place.
/// If a "\?" is changed to "?" then "dropped" is incremented, unless NULL.
/// If "magic_val" is not NULL, returns the effective magicness of the pattern
char *skip_regexp_ex(char *startp, int dirc, int magic, char **newp, int *dropped,
                     magic_T *magic_val)
{
  return rs_skip_regexp_ex(startp, dirc, magic, newp, dropped, (int *)magic_val);
}

// vim_regexec and friends

// Global work variables for vim_regexec().

// Sometimes need to save a copy of a line.  Since alloc()/free() is very
// slow, we keep one allocated piece of memory and only re-allocate it when
// it's too small.  It's freed in bt_regexec_both() when finished.
static uint8_t *reg_tofree = NULL;
static unsigned reg_tofreelen;

// Structure used to store the execution state of the regex engine.
// Which ones are set depends on whether a single-line or multi-line match is
// done:
//                      single-line             multi-line
// reg_match            &regmatch_T             NULL
// reg_mmatch           NULL                    &regmmatch_T
// reg_startp           reg_match->startp       <invalid>
// reg_endp             reg_match->endp         <invalid>
// reg_startpos         <invalid>               reg_mmatch->startpos
// reg_endpos           <invalid>               reg_mmatch->endpos
// reg_win              NULL                    window in which to search
// reg_buf              curbuf                  buffer in which to search
// reg_firstlnum        <invalid>               first line in which to search
// reg_maxline          0                       last line nr
// reg_line_lbr         false or true           false
typedef struct {
  regmatch_T *reg_match;
  regmmatch_T *reg_mmatch;

  uint8_t **reg_startp;
  uint8_t **reg_endp;
  lpos_T *reg_startpos;
  lpos_T *reg_endpos;

  win_T *reg_win;
  buf_T *reg_buf;
  linenr_T reg_firstlnum;
  linenr_T reg_maxline;
  bool reg_line_lbr;  // "\n" in string is line break

  // The current match-position is remembered with these variables:
  linenr_T lnum;  ///< line number, relative to first line
  uint8_t *line;   ///< start of current line
  uint8_t *input;  ///< current input, points into "line"

  int need_clear_subexpr;   ///< subexpressions still need to be cleared
  int need_clear_zsubexpr;  ///< extmatch subexpressions still need to be
                            ///< cleared

  // Internal copy of 'ignorecase'.  It is set at each call to vim_regexec().
  // Normally it gets the value of "rm_ic" or "rmm_ic", but when the pattern
  // contains '\c' or '\C' the value is overruled.
  bool reg_ic;

  // Similar to "reg_ic", but only for 'combining' characters.  Set with \Z
  // flag in the regexp.  Defaults to false, always.
  bool reg_icombine;

  bool reg_nobreak;

  // Copy of "rmm_maxcol": maximum column to search for a match.  Zero when
  // there is no maximum.
  colnr_T reg_maxcol;

  // State for the NFA engine regexec.
  int nfa_has_zend;     ///< NFA regexp \ze operator encountered.
  int nfa_has_backref;  ///< NFA regexp \1 .. \9 encountered.
  int nfa_nsubexpr;     ///< Number of sub expressions actually being used
                        ///< during execution. 1 if only the whole match
                        ///< (subexpr 0) is used.
  // listid is global, so that it increases on recursive calls to
  // nfa_regmatch(), which means we don't have to clear the lastlist field of
  // all the states.
  int nfa_listid;
  int nfa_alt_listid;

  int nfa_has_zsubexpr;  ///< NFA regexp has \z( ), set zsubexpr.
} regexec_T;

static regexec_T rex;
static bool rex_in_use = false;

// --- Rex and error accessors for Rust FFI ---
int nvim_regexp_get_rex_reg_ic(void) { return rex.reg_ic; }
int nvim_regexp_get_rex_reg_icombine(void) { return rex.reg_icombine; }

void nvim_regexp_set_rc_did_emsg(int v) { rc_did_emsg = (bool)v; }
void nvim_regexp_semsg_e888(const char *what) { semsg(_("E888: (NFA regexp) cannot repeat %s"), what); }

int nvim_regexp_emsg2_fail(const char *msg, int is_magic_all)
{
  semsg(msg, is_magic_all ? "" : "\\");
  rc_did_emsg = true;
  return FAIL;
}

static bool can_f_submatch = false;  ///< true when submatch() can be used

/// These pointers are used for reg_submatch().  Needed for when the
/// substitution string is an expression that contains a call to substitute()
/// and submatch().
typedef struct {
  regmatch_T *sm_match;
  regmmatch_T *sm_mmatch;
  linenr_T sm_firstlnum;
  linenr_T sm_maxline;
  int sm_line_lbr;
} regsubmatch_T;

static regsubmatch_T rsm;  ///< can only be used when can_f_submatch is true

/// Common code for reg_getline(), reg_getline_len(), reg_getline_submatch() and
/// reg_getline_submatch_len().
///
/// @param flags  a bitmask that controls what info is to be returned
///               and whether or not submatch is in effect.
extern void rs_reg_getline_common(int32_t lnum, int flags, char **line, int32_t *length);

static void reg_getline_common(linenr_T lnum, reg_getline_flags_T flags, char **line,
                               colnr_T *length)
{
  rs_reg_getline_common((int32_t)lnum, (int)flags, line, (int32_t *)length);
}

/// Get pointer to the line "lnum", which is relative to "reg_firstlnum".
static char *reg_getline(linenr_T lnum)
{
  char *line;
  reg_getline_common(lnum, RGLF_LINE, &line, NULL);
  return line;
}

/// Get length of line "lnum", which is relative to "reg_firstlnum".
static colnr_T reg_getline_len(linenr_T lnum)
{
  colnr_T length;
  reg_getline_common(lnum, RGLF_LENGTH, NULL, &length);
  return length;
}

static uint8_t *reg_startzp[NSUBEXP];  // Workspace to mark beginning
static uint8_t *reg_endzp[NSUBEXP];    //   and end of \z(...\) matches
static lpos_T reg_startzpos[NSUBEXP];   // idem, beginning pos
static lpos_T reg_endzpos[NSUBEXP];     // idem, end pos

// true if using multi-line regexp.
#define REG_MULTI       (rex.reg_match == NULL)

// cleanup_subexpr / cleanup_zsubexpr accessors for Rust FFI
int nvim_regexp_get_rex_need_clear_subexpr(void) { return rex.need_clear_subexpr; }
void nvim_regexp_set_rex_need_clear_subexpr(int v) { rex.need_clear_subexpr = (bool)v; }
int nvim_regexp_get_rex_need_clear_zsubexpr(void) { return rex.need_clear_zsubexpr; }
void nvim_regexp_set_rex_need_clear_zsubexpr(int v) { rex.need_clear_zsubexpr = (bool)v; }
int nvim_regexp_is_reg_multi(void) { return REG_MULTI; }
// Subexpression position/pointer array accessors for Rust FFI
lpos_T *nvim_regexp_get_rex_startpos_array(void) { return rex.reg_startpos; }
lpos_T *nvim_regexp_get_rex_endpos_array(void) { return rex.reg_endpos; }
uint8_t **nvim_regexp_get_rex_startp_array(void) { return (uint8_t **)rex.reg_startp; }
uint8_t **nvim_regexp_get_rex_endp_array(void) { return (uint8_t **)rex.reg_endp; }

void nvim_regexp_clear_rex_startpos(void) { memset(rex.reg_startpos, 0xff, sizeof(lpos_T) * NSUBEXP); }
void nvim_regexp_clear_rex_endpos(void) { memset(rex.reg_endpos, 0xff, sizeof(lpos_T) * NSUBEXP); }
void nvim_regexp_clear_rex_startp(void) { memset(rex.reg_startp, 0, sizeof(char *) * NSUBEXP); }
void nvim_regexp_clear_rex_endp(void) { memset(rex.reg_endp, 0, sizeof(char *) * NSUBEXP); }
void nvim_regexp_clear_reg_startzpos(void) { memset(reg_startzpos, 0xff, sizeof(lpos_T) * NSUBEXP); }
void nvim_regexp_clear_reg_endzpos(void) { memset(reg_endzpos, 0xff, sizeof(lpos_T) * NSUBEXP); }
void nvim_regexp_clear_reg_startzp(void) { memset(reg_startzp, 0, sizeof(char *) * NSUBEXP); }
void nvim_regexp_clear_reg_endzp(void) { memset(reg_endzp, 0, sizeof(char *) * NSUBEXP); }

// reg_prev_class accessors for Rust FFI
uint8_t *nvim_regexp_get_rex_input(void) { return rex.input; }
uint8_t *nvim_regexp_get_rex_line(void) { return rex.line; }
int64_t *nvim_regexp_get_rex_reg_buf_chartab(void) { return rex.reg_buf->b_chartab; }

// reg_nextline accessors for Rust FFI
int32_t nvim_regexp_get_rex_lnum(void) { return (int32_t)rex.lnum; }
void nvim_regexp_set_rex_lnum(int32_t v) { rex.lnum = (linenr_T)v; }
void nvim_regexp_set_rex_line_and_input(uint8_t *line) { rex.line = line; rex.input = line; }
char *nvim_regexp_call_reg_getline(int32_t lnum) { return reg_getline((linenr_T)lnum); }

// match_with_backref accessors for Rust FFI
uint8_t *nvim_regexp_get_reg_tofree(void) { return reg_tofree; }
void nvim_regexp_set_reg_tofree(uint8_t *p) { reg_tofree = p; }
unsigned nvim_regexp_get_reg_tofreelen(void) { return reg_tofreelen; }
void nvim_regexp_set_reg_tofreelen(unsigned v) { reg_tofreelen = v; }
void nvim_regexp_set_rex_line(uint8_t *line) { rex.line = line; }
void nvim_regexp_set_rex_input(uint8_t *input) { rex.input = input; }
int nvim_regexp_get_got_int(void) { return got_int; }
int nvim_regexp_get_rex_line_strlen(void) { return (int)strlen((char *)rex.line); }
int32_t nvim_regexp_call_reg_getline_len(int32_t lnum) { return (int32_t)reg_getline_len((linenr_T)lnum); }

// regrepeat accessors for Rust FFI
int nvim_regexp_get_rex_reg_line_lbr(void) { return rex.reg_line_lbr; }
int nvim_regexp_call_vim_iswordp_buf(const char *p) { return vim_iswordp_buf(p, rex.reg_buf); }
void nvim_regexp_iemsg_re_corr(void) { iemsg(_(e_re_corr)); }

// regtry accessors for Rust FFI
uint8_t nvim_regexp_get_prog_reghasz(const void *prog) { return ((const bt_regprog_T *)prog)->reghasz; }
uint8_t *nvim_regexp_get_prog_program(void *prog) { return ((bt_regprog_T *)prog)->program; }
void nvim_regexp_unref_re_extmatch_out(void) { unref_extmatch(re_extmatch_out); }
void nvim_regexp_set_re_extmatch_out(void *em) { re_extmatch_out = (reg_extmatch_T *)em; }
int32_t nvim_regexp_get_reg_startzpos_lnum(int i) { return (int32_t)reg_startzpos[i].lnum; }
int32_t nvim_regexp_get_reg_startzpos_col(int i) { return (int32_t)reg_startzpos[i].col; }
int32_t nvim_regexp_get_reg_endzpos_lnum(int i) { return (int32_t)reg_endzpos[i].lnum; }
int32_t nvim_regexp_get_reg_endzpos_col(int i) { return (int32_t)reg_endzpos[i].col; }
uint8_t *nvim_regexp_get_reg_startzp(int i) { return reg_startzp[i]; }
uint8_t *nvim_regexp_get_reg_endzp(int i) { return reg_endzp[i]; }

// reg_breakcheck / reg_iswordc accessors for Rust FFI
int nvim_regexp_get_rex_reg_nobreak(void) { return rex.reg_nobreak; }
void *nvim_regexp_get_rex_reg_buf(void) { return (void *)rex.reg_buf; }

// reg_submatch accessors for Rust FFI
int nvim_regexp_get_can_f_submatch(void) { return can_f_submatch ? 1 : 0; }
int nvim_regexp_is_rsm_sm_match_null(void) { return rsm.sm_match == NULL ? 1 : 0; }
const char *nvim_regexp_get_rsm_sm_match_startp(int i) { return rsm.sm_match->startp[i]; }
const char *nvim_regexp_get_rsm_sm_match_endp(int i) { return rsm.sm_match->endp[i]; }
int32_t nvim_regexp_get_rsm_sm_mmatch_startpos_lnum(int i) { return (int32_t)rsm.sm_mmatch->startpos[i].lnum; }
int32_t nvim_regexp_get_rsm_sm_mmatch_startpos_col(int i) { return (int32_t)rsm.sm_mmatch->startpos[i].col; }
int32_t nvim_regexp_get_rsm_sm_mmatch_endpos_lnum(int i) { return (int32_t)rsm.sm_mmatch->endpos[i].lnum; }
int32_t nvim_regexp_get_rsm_sm_mmatch_endpos_col(int i) { return (int32_t)rsm.sm_mmatch->endpos[i].col; }

// reg_match_visual accessors for Rust FFI

// Returns 0 if quick-reject (rex.reg_buf != curbuf || VIsual.lnum == 0 || !REG_MULTI), 1 otherwise
int nvim_regexp_visual_quick_check(void) { return (rex.reg_buf == curbuf && VIsual.lnum != 0 && REG_MULTI) ? 1 : 0; }

// Populate visual area top/bot/mode/curswant for reg_match_visual.
// The caller passes output pointers.  Returns wp (window pointer) for getvvcol calls.
void *nvim_regexp_get_visual_area(int32_t *top_lnum, int32_t *top_col,
                                  int32_t *bot_lnum, int32_t *bot_col,
                                  int *mode, int32_t *curswant_out)
{
  pos_T top, bot;
  win_T *wp = rex.reg_win == NULL ? curwin : rex.reg_win;
  int vmode;
  colnr_T curswant;

  if (VIsual_active) {
    if (lt(VIsual, wp->w_cursor)) {
      top = VIsual;
      bot = wp->w_cursor;
    } else {
      top = wp->w_cursor;
      bot = VIsual;
    }
    vmode = VIsual_mode;
    curswant = wp->w_curswant;
  } else {
    if (lt(curbuf->b_visual.vi_start, curbuf->b_visual.vi_end)) {
      top = curbuf->b_visual.vi_start;
      bot = curbuf->b_visual.vi_end;
    } else {
      top = curbuf->b_visual.vi_end;
      bot = curbuf->b_visual.vi_start;
    }
    // a substitute command may have removed some lines
    if (bot.lnum > curbuf->b_ml.ml_line_count) {
      bot.lnum = curbuf->b_ml.ml_line_count;
    }
    vmode = curbuf->b_visual.vi_mode;
    curswant = curbuf->b_visual.vi_curswant;
  }

  *top_lnum = (int32_t)top.lnum;
  *top_col = (int32_t)top.col;
  *bot_lnum = (int32_t)bot.lnum;
  *bot_col = (int32_t)bot.col;
  *mode = vmode;
  *curswant_out = (int32_t)curswant;
  return (void *)wp;
}

int nvim_regexp_get_p_sel_char(void) { return *p_sel; }

// Wrapper: calls getvvcol with a constructed pos_T, returns start and end.
void nvim_regexp_call_getvvcol(void *wp, int32_t lnum, int32_t col,
                               int32_t *start_out, int32_t *end_out)
{
  pos_T pos;
  pos.lnum = (linenr_T)lnum;
  pos.col = (colnr_T)col;
  pos.coladd = 0;
  colnr_T s, e;
  getvvcol((win_T *)wp, &pos, &s, NULL, &e);
  *start_out = (int32_t)s;
  *end_out = (int32_t)e;
}

// Wrapper: calls win_linetabsize
int32_t nvim_regexp_call_win_linetabsize(void *wp, int32_t lnum,
                                         const char *line, int32_t col)
{
  return (int32_t)win_linetabsize((win_T *)wp, (linenr_T)lnum, (char *)line, (colnr_T)col);
}

// vim_regsub_both accessors for Rust FFI
const char *nvim_regexp_get_rex_reg_match_startp(int no) { return rex.reg_match->startp[no]; }
const char *nvim_regexp_get_rex_reg_match_endp(int no) { return rex.reg_match->endp[no]; }
int32_t nvim_regexp_get_rex_reg_mmatch_startpos_lnum(int no) { return (int32_t)rex.reg_mmatch->startpos[no].lnum; }
int32_t nvim_regexp_get_rex_reg_mmatch_startpos_col(int no) { return (int32_t)rex.reg_mmatch->startpos[no].col; }
int32_t nvim_regexp_get_rex_reg_mmatch_endpos_lnum(int no) { return (int32_t)rex.reg_mmatch->endpos[no].lnum; }
int32_t nvim_regexp_get_rex_reg_mmatch_endpos_col(int no) { return (int32_t)rex.reg_mmatch->endpos[no].col; }
int nvim_regexp_call_prog_magic_wrong(void) { return prog_magic_wrong(); }
void nvim_regexp_call_iemsg_not_enough_space(void) { iemsg("vim_regsub_both(): not enough space"); }
void nvim_regexp_call_iemsg_re_damg(void) { iemsg(_(e_re_damg)); }
void nvim_regexp_emsg_e_null(void) { emsg(_(e_null)); }
void nvim_regexp_emsg_e_substitute_nesting(void) { emsg(_(e_substitute_nesting_too_deep)); }

// Compound setup for vim_regsub: sets rex fields for single-line substitution
void nvim_regexp_setup_vim_regsub(void *rmp)
{
  rex.reg_match = (regmatch_T *)rmp;
  rex.reg_mmatch = NULL;
  rex.reg_maxline = 0;
  rex.reg_buf = curbuf;
  rex.reg_line_lbr = true;
}

// Compound setup for vim_regsub_multi: sets rex fields for multi-line substitution
void nvim_regexp_setup_vim_regsub_multi(void *rmp, int32_t lnum)
{
  rex.reg_match = NULL;
  rex.reg_mmatch = (regmmatch_T *)rmp;
  rex.reg_buf = curbuf;  // always works on the current buffer!
  rex.reg_firstlnum = (linenr_T)lnum;
  rex.reg_maxline = curbuf->b_ml.ml_line_count - (linenr_T)lnum;
  rex.reg_line_lbr = false;
}

// reg_getline_common accessors for Rust FFI
int32_t nvim_regexp_get_rex_reg_firstlnum(void) { return (int32_t)rex.reg_firstlnum; }
int32_t nvim_regexp_get_rex_reg_maxline(void) { return (int32_t)rex.reg_maxline; }
int32_t nvim_regexp_get_rsm_firstlnum(void) { return (int32_t)rsm.sm_firstlnum; }
int32_t nvim_regexp_get_rsm_maxline(void) { return (int32_t)rsm.sm_maxline; }
char *nvim_regexp_call_ml_get_buf(int32_t lnum) { return ml_get_buf(rex.reg_buf, (linenr_T)lnum); }
int32_t nvim_regexp_call_ml_get_buf_len(int32_t lnum) { return (int32_t)ml_get_buf_len(rex.reg_buf, (linenr_T)lnum); }

// prog_magic_wrong logic is now in Rust (rs_prog_magic_wrong).
// The C accessor wraps the Rust implementation.
extern int rs_prog_magic_wrong(void);
static int prog_magic_wrong(void)
{
  return rs_prog_magic_wrong();
}

// Accessor: check if a prog uses the NFA engine
int nvim_regexp_prog_is_nfa_engine(void *prog) {
  return ((regprog_T *)prog)->engine == &nfa_regengine ? 1 : 0;
}
////////////////////////////////////////////////////////////////
//                    regsub stuff                            //

/// Put the submatches in "argv[argskip]" which is a list passed into
/// call_func() by vim_regsub_both().
static int fill_submatch_list(int argc FUNC_ATTR_UNUSED, typval_T *argv, int argskip, ufunc_T *fp)
  FUNC_ATTR_NONNULL_ALL
{
  typval_T *listarg = argv + argskip;

  if (!fp->uf_varargs && fp->uf_args.ga_len <= argskip) {
    // called function doesn't take a submatches argument
    return argskip;
  }

  // Relies on sl_list to be the first item in staticList10_T.
  tv_list_init_static10((staticList10_T *)listarg->vval.v_list);

  // There are always 10 list items in staticList10_T.
  listitem_T *li = tv_list_first(listarg->vval.v_list);
  for (int i = 0; i < 10; i++) {
    char *s = rsm.sm_match->startp[i];
    if (s == NULL || rsm.sm_match->endp[i] == NULL) {
      s = NULL;
    } else {
      s = xstrnsave(s, (size_t)(rsm.sm_match->endp[i] - s));
    }
    TV_LIST_ITEM_TV(li)->v_type = VAR_STRING;
    TV_LIST_ITEM_TV(li)->vval.v_string = s;
    li = TV_LIST_ITEM_NEXT(argv->vval.v_list, li);
  }
  return argskip + 1;
}

static void clear_submatch_list(staticList10_T *sl)
{
  TV_LIST_ITER(&sl->sl_list, li, {
    xfree(TV_LIST_ITEM_TV(li)->vval.v_string);
  });
}

// Rust FFI: vim_regsub functions
extern int rs_vim_regsub(void *rmp, char *source, void *expr, char *dest, int destlen, int flags);
extern int rs_vim_regsub_multi(void *rmp, int32_t lnum, char *source, char *dest, int destlen,
                               int flags);

/// vim_regsub() - perform substitutions after a vim_regexec() or
/// vim_regexec_multi() match.
int vim_regsub(regmatch_T *rmp, char *source, typval_T *expr, char *dest, int destlen, int flags)
{
  return rs_vim_regsub(rmp, source, expr, dest, destlen, flags);
}

int vim_regsub_multi(regmmatch_T *rmp, linenr_T lnum, char *source, char *dest, int destlen,
                     int flags)
{
  return rs_vim_regsub_multi(rmp, (int32_t)lnum, source, dest, destlen, flags);
}

// When nesting more than a couple levels it's probably a mistake.
#define MAX_REGSUB_NESTING 4
static char *eval_result[MAX_REGSUB_NESTING] = { NULL, NULL, NULL, NULL };
static int regsub_nesting = 0;

#if defined(EXITFREE)
void free_resub_eval_result(void)
{
  for (int i = 0; i < MAX_REGSUB_NESTING; i++) {
    XFREE_CLEAR(eval_result[i]);
  }
}
#endif

// --- eval_result accessors for Rust FFI ---
int nvim_regexp_get_max_regsub_nesting(void) { return MAX_REGSUB_NESTING; }
int nvim_regexp_get_regsub_nesting(void) { return regsub_nesting; }
void nvim_regexp_set_regsub_nesting(int v) { regsub_nesting = v; }
char *nvim_regexp_get_eval_result(int i) { return eval_result[i]; }
void nvim_regexp_set_eval_result(int i, char *v) { eval_result[i] = v; }
void nvim_regexp_free_eval_result(int i) { XFREE_CLEAR(eval_result[i]); }

/// Compound accessor: evaluate a \= substitution expression.
/// Handles all VimL type interactions (call_func, eval_to_string, typval_T, etc.)
/// so that Rust does not need to know about VimL types.
///
/// @param source     the substitution string (for eval_to_string path, source+2 is used)
/// @param expr       opaque pointer to typval_T* (or NULL for string \= path)
/// @param flags      REGSUB_* flags
/// @param nested     nesting level (index into eval_result[])
///
/// Stores result in eval_result[nested] and returns its strlen, or -1 on error.
/// Side effects: saves/restores can_f_submatch and rsm; increments/decrements nesting.
int nvim_regexp_eval_regsub_expr(char *source, void *expr_ptr, int flags, int nested)
{
  typval_T *expr = (typval_T *)expr_ptr;
  const bool prev_can_f_submatch = can_f_submatch;
  regsubmatch_T rsm_save;

  XFREE_CLEAR(eval_result[nested]);

  // The expression may contain substitute(), which calls us
  // recursively.  Make sure submatch() gets the text from the first
  // level.
  if (can_f_submatch) {
    rsm_save = rsm;
  }
  can_f_submatch = true;
  rsm.sm_match = rex.reg_match;
  rsm.sm_mmatch = rex.reg_mmatch;
  rsm.sm_firstlnum = rex.reg_firstlnum;
  rsm.sm_maxline = rex.reg_maxline;
  rsm.sm_line_lbr = rex.reg_line_lbr;

  // Although unlikely, it is possible that the expression invokes a
  // substitute command (it might fail, but still).  Therefore keep
  // an array of eval results.
  regsub_nesting++;

  if (expr != NULL) {
    typval_T argv[2];
    typval_T rettv;
    staticList10_T matchList = TV_LIST_STATIC10_INIT;
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = NULL;
    argv[0].v_type = VAR_LIST;
    argv[0].vval.v_list = &matchList.sl_list;
    funcexe_T funcexe = FUNCEXE_INIT;
    funcexe.fe_argv_func = fill_submatch_list;
    funcexe.fe_evaluate = true;
    char *s;
    if (expr->v_type == VAR_FUNC) {
      s = expr->vval.v_string;
      call_func(s, -1, &rettv, 1, argv, &funcexe);
    } else if (expr->v_type == VAR_PARTIAL) {
      partial_T *partial = expr->vval.v_partial;

      s = rs_partial_name(partial);
      funcexe.fe_partial = partial;
      call_func(s, -1, &rettv, 1, argv, &funcexe);
    }
    if (tv_list_len(&matchList.sl_list) > 0) {
      // fill_submatch_list() was called.
      clear_submatch_list(&matchList);
    }
    if (rettv.v_type == VAR_UNKNOWN) {
      // something failed, no need to report another error
      eval_result[nested] = NULL;
    } else {
      char buf[NUMBUFLEN];
      eval_result[nested] = (char *)tv_get_string_buf_chk(&rettv, buf);
      if (eval_result[nested] != NULL) {
        eval_result[nested] = xstrdup(eval_result[nested]);
      }
    }
    tv_clear(&rettv);
  } else {
    eval_result[nested] = eval_to_string(source + 2, true, false);
  }
  regsub_nesting--;

  if (eval_result[nested] != NULL) {
    int had_backslash = false;

    for (char *s = eval_result[nested]; *s != NUL; MB_PTR_ADV(s)) {
      // Change NL to CR, so that it becomes a line break,
      // unless called from vim_regexec_nl().
      // Skip over a backslashed character.
      if (*s == NL && !rsm.sm_line_lbr) {
        *s = CAR;
      } else if (*s == '\\' && s[1] != NUL) {
        s++;
        // Change NL to CR here too, so that this works:
        // :s/abc\\\ndef/\="aaa\\\nbbb"/  on text:
        //   abc{backslash}
        //   def
        // Not when called from vim_regexec_nl().
        if (*s == NL && !rsm.sm_line_lbr) {
          *s = CAR;
        }
        had_backslash = true;
      }
    }
    if (had_backslash && (flags & REGSUB_BACKSLASH)) {
      // Backslashes will be consumed, need to double them.
      char *s = vim_strsave_escaped(eval_result[nested], "\\");
      xfree(eval_result[nested]);
      eval_result[nested] = s;
    }
  }

  can_f_submatch = prev_can_f_submatch;
  if (can_f_submatch) {
    rsm = rsm_save;
  }

  if (eval_result[nested] != NULL) {
    return (int)strlen(eval_result[nested]);
  }
  return 0;
}

// vim_regsub_both is now implemented in Rust as rs_vim_regsub_both

static char *reg_getline_submatch(linenr_T lnum)
{
  char *line;
  reg_getline_common(lnum, RGLF_LINE | RGLF_SUBMATCH, &line, NULL);
  return line;
}

// Used for the submatch() function with the optional non-zero argument: get
// the list of strings from the n'th submatch in allocated memory with NULs
// represented in NLs.
// Returns a list of allocated strings.  Returns NULL when not in a ":s"
// command, for a non-existing submatch and for any error.
list_T *reg_submatch_list(int no)
{
  if (!can_f_submatch || no < 0) {
    return NULL;
  }

  linenr_T slnum;
  linenr_T elnum;
  list_T *list;
  const char *s;

  if (rsm.sm_match == NULL) {
    slnum = rsm.sm_mmatch->startpos[no].lnum;
    elnum = rsm.sm_mmatch->endpos[no].lnum;
    if (slnum < 0 || elnum < 0) {
      return NULL;
    }

    colnr_T scol = rsm.sm_mmatch->startpos[no].col;
    colnr_T ecol = rsm.sm_mmatch->endpos[no].col;

    list = tv_list_alloc(elnum - slnum + 1);

    s = reg_getline_submatch(slnum) + scol;
    if (slnum == elnum) {
      tv_list_append_string(list, s, ecol - scol);
    } else {
      int max_lnum = elnum - slnum;
      tv_list_append_string(list, s, -1);
      for (int i = 1; i < max_lnum; i++) {
        s = reg_getline_submatch(slnum + i);
        tv_list_append_string(list, s, -1);
      }
      s = reg_getline_submatch(elnum);
      tv_list_append_string(list, s, ecol);
    }
  } else {
    s = rsm.sm_match->startp[no];
    if (s == NULL || rsm.sm_match->endp[no] == NULL) {
      return NULL;
    }
    list = tv_list_alloc(1);
    tv_list_append_string(list, s, rsm.sm_match->endp[no] - s);
  }

  tv_list_ref(list);
  return list;
}

/// Initialize the values used for matching against multiple lines
///
/// @param win   window in which to search or NULL
/// @param buf   buffer in which to search
/// @param lnum  nr of line to start looking for match
static void init_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum)
{
  rex.reg_match = NULL;
  rex.reg_mmatch = rmp;
  rex.reg_buf = buf;
  rex.reg_win = win;
  rex.reg_firstlnum = lnum;
  rex.reg_maxline = rex.reg_buf->b_ml.ml_line_count - lnum;
  rex.reg_line_lbr = false;
  rex.reg_ic = rmp->rmm_ic;
  rex.reg_icombine = false;
  rex.reg_nobreak = rmp->regprog->re_flags & RE_NOBREAK;
  rex.reg_maxcol = rmp->rmm_maxcol;
}

static int num_complex_braces;  ///< Complex \{...} count
static uint8_t *regcode;         ///< Code-emit pointer, or JUST_CALC_SIZE
static int64_t regsize;            ///< Code size.
static int reg_toolong;         ///< true when offset out of range
static uint8_t had_endbrace[NSUBEXP];  ///< flags, true if end of () found
static int64_t brace_min[10];        ///< Minimums for complex brace repeats
static int64_t brace_max[10];        ///< Maximums for complex brace repeats
static int brace_count[10];       ///< Current counts for complex brace repeats
static int one_exactly = false;   ///< only do one char for EXACTLY

// When making changes to classchars also change nfa_classcodes.
static uint8_t *classchars = (uint8_t *)".iIkKfFpPsSdDxXoOwWhHaAlLuU";

// When regcode is set to this value, code is not emitted and size is computed
// instead.
#define JUST_CALC_SIZE  ((uint8_t *)-1)

// --- Compilation global accessors for Rust FFI ---
uint8_t *nvim_regexp_get_regcode(void) { return regcode; }
void nvim_regexp_set_regcode(uint8_t *p) { regcode = p; }
int64_t nvim_regexp_get_regsize(void) { return regsize; }
void nvim_regexp_set_regsize(int64_t v) { regsize = v; }
int nvim_regexp_get_reg_toolong(void) { return reg_toolong; }
void nvim_regexp_set_reg_toolong(int v) { reg_toolong = v; }
uint8_t *nvim_regexp_get_just_calc_size(void) { return JUST_CALC_SIZE; }
int nvim_regexp_get_num_complex_braces(void) { return num_complex_braces; }
void nvim_regexp_set_num_complex_braces(int v) { num_complex_braces = v; }

// Error helpers for Rust FFI (keeps gettext _() calls in C)
void nvim_regexp_emsg2_e59(int m)
{
  semsg(_(e_invalid_character_after_str_at), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e60(int m)
{
  semsg(_("E60: Too many complex %s{...}s"), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e61(int m)
{
  semsg(_("E61: Nested %s*"), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg3_e62(int m, int c)
{
  semsg(_("E62: Nested %s%c"), m ? "" : "\\", c);
  rc_did_emsg = true;
}
void nvim_regexp_set_had_endbrace(int parno, int v) { had_endbrace[parno] = (uint8_t)v; }
void nvim_regexp_emsg_e50(void)
{
  emsg(_("E50: Too many \\z("));
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e51(int m)
{
  semsg(_("E51: Too many %s("), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e52(void)
{
  emsg(_("E52: Unmatched \\z("));
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e53(int m)
{
  semsg(_(e_unmatchedpp), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e54(int m)
{
  semsg(_(e_unmatchedp), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e55(int m)
{
  semsg(_(e_unmatchedpar), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e488(void)
{
  emsg(_(e_trailing));
  rc_did_emsg = true;
}

int nvim_regexp_get_had_endbrace(int refnum) { return had_endbrace[refnum]; }
void nvim_regexp_clear_had_endbrace(void) { CLEAR_FIELD(had_endbrace); }
// reg_do_extmatch is a global (globals.h), not a C static -- accessor kept for Rust FFI
int nvim_regexp_get_reg_do_extmatch(void) { return reg_do_extmatch; }
// reg_prev_sub is a C static (kept here until it is moved to Rust in a later phase)
char *nvim_regexp_get_reg_prev_sub_ptr(void) { return reg_prev_sub; }
int32_t nvim_regexp_get_curwin_lnum(void) { return (int32_t)curwin->w_cursor.lnum; }
int32_t nvim_regexp_get_curwin_col(void) { return (int32_t)curwin->w_cursor.col; }
int32_t nvim_regexp_get_curwin_vcol(void)
{
  colnr_T vcol = 0;
  getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol);
  return (int32_t)(++vcol);
}

void nvim_regexp_emsg_e63_underscore(void)
{
  emsg(_(e_invalid_use_of_underscore));
  rc_did_emsg = true;
}
void nvim_regexp_iemsg_internal(void)
{
  iemsg(_(e_internal_error_in_regexp));
  rc_did_emsg = true;
}
void nvim_regexp_emsg3_e64(int m, int c)
{
  semsg(_("E64: %s%c follows nothing"),
        m ? "" : "\\", c);
  rc_did_emsg = true;
}
void nvim_regexp_emsg_nopresub(void)
{
  emsg(_(e_nopresub));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e65(void)
{
  emsg(_("E65: Illegal back reference"));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e66(void)
{
  emsg(_(e_z_not_allowed));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e67(void)
{
  emsg(_(e_z1_not_allowed));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e68(void)
{
  emsg(_("E68: Invalid character after \\z"));
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e69(int m)
{
  semsg(_(e_missing_sb), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e70(int m)
{
  semsg(_(e_empty_sb), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e71(int m)
{
  semsg(_("E71: Invalid character after %s%%"),
        m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e678(int m)
{
  semsg(_("E678: Invalid character after %s%%[dxouU]"),
        m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e769(int m)
{
  semsg(_(e_missingbracket), m ? "" : "\\");
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e944(void)
{
  emsg(_(e_reverse_range));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e945(void)
{
  emsg(_(e_large_class));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e949(void)
{
  emsg(_(e_unicode_val_too_large));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_toomsbra(void)
{
  emsg(_(e_toomsbra));
  rc_did_emsg = true;
}
void nvim_regexp_semsg_e_atom_engine(int c)
{
  semsg(_(e_atom_engine_must_be_at_start_of_pattern), c);
  rc_did_emsg = true;
}
void nvim_regexp_semsg_e_dot_pos(int c)
{
  semsg(_(e_regexp_number_after_dot_pos_search_chr), c);
  rc_did_emsg = true;
}
void nvim_regexp_emsg2_e369(int m)
{
  semsg(_(e_invalid_item_in_str_brackets), m ? "" : "\\");
  rc_did_emsg = true;
}

// used to store input position when a BACK was encountered, so that we now if
// we made any progress since the last time.
typedef struct backpos_S {
  uint8_t *bp_scan;         // "scan" where BACK was encountered
  regsave_T bp_pos;           // last input position
} backpos_T;

static garray_T regstack = GA_EMPTY_INIT_VALUE;
static garray_T backpos = GA_EMPTY_INIT_VALUE;
static regsave_T behind_pos;
#define REGSTACK_INITIAL        2048
#define BACKPOS_INITIAL         64

// Thin wrapper: compile via Rust BT engine.
static regprog_T *bt_regcomp(uint8_t *expr, int re_flags)
{
  return (regprog_T *)rs_bt_regcomp(expr, re_flags);
}

extern int rs_vim_regcomp_had_eol(void);
int vim_regcomp_had_eol(void) { return rs_vim_regcomp_had_eol(); }

// Free a compiled regexp program, returned by bt_regcomp().
static void bt_regfree(regprog_T *prog)
{
  xfree(prog);
}

// The arguments from BRACE_LIMITS are stored here.  They are actually local
// to regmatch(), but they are here to reduce the amount of stack space used
// (it can be called recursively many times).
static int64_t bl_minval;
static int64_t bl_maxval;
// --- regmatch accessor functions for Rust FFI (rs_regmatch) ---

// Regstack/backpos management
uint8_t *nvim_regexp_get_regstack_data(void) { return (uint8_t *)regstack.ga_data; }
int nvim_regexp_get_regstack_len(void) { return regstack.ga_len; }
int nvim_regexp_get_regstack_maxlen(void) { return regstack.ga_maxlen; }
void nvim_regexp_set_regstack_len(int v) { regstack.ga_len = v; }
void nvim_regexp_call_ga_grow_regstack(int n) { ga_grow(&regstack, n); }

uint8_t *nvim_regexp_get_backpos_data(void) { return (uint8_t *)backpos.ga_data; }
int nvim_regexp_get_backpos_len(void) { return backpos.ga_len; }
void nvim_regexp_set_backpos_len(int v) { backpos.ga_len = v; }
void nvim_regexp_call_ga_grow_backpos(int n) { ga_grow(&backpos, n); }

// Brace static variable access
int64_t nvim_regexp_get_brace_min(int no) { return brace_min[no]; }
void nvim_regexp_set_brace_min(int no, int64_t v) { brace_min[no] = v; }
int64_t nvim_regexp_get_brace_max(int no) { return brace_max[no]; }
void nvim_regexp_set_brace_max(int no, int64_t v) { brace_max[no] = v; }
int nvim_regexp_get_brace_count(int no) { return brace_count[no]; }
void nvim_regexp_set_brace_count(int no, int v) { brace_count[no] = v; }

int64_t nvim_regexp_get_bl_minval(void) { return bl_minval; }
void nvim_regexp_set_bl_minval(int64_t v) { bl_minval = v; }
int64_t nvim_regexp_get_bl_maxval(void) { return bl_maxval; }
void nvim_regexp_set_bl_maxval(int64_t v) { bl_maxval = v; }

// Behind position (return void* to avoid exposing local regsave_T type in generated header)
void *nvim_regexp_get_behind_pos(void) { return (void *)&behind_pos; }

// maxmempattern option
int64_t nvim_regexp_get_p_mmp(void) { return p_mmp; }

// External match
uint8_t *nvim_regexp_get_re_extmatch_in_match(int no) {
  if (re_extmatch_in != NULL && re_extmatch_in->matches[no] != NULL) {
    return re_extmatch_in->matches[no];
  }
  return NULL;
}

// Mark support
void *nvim_regexp_call_mark_get(int mark) {
  return (void *)mark_get(rex.reg_buf, curwin, NULL, kMarkBufLocal, mark);
}
int32_t nvim_regexp_get_fmark_lnum(void *fm) { return (int32_t)((fmark_T *)fm)->mark.lnum; }
int32_t nvim_regexp_get_fmark_col(void *fm) { return (int32_t)((fmark_T *)fm)->mark.col; }

// Window/cursor support
void *nvim_regexp_get_rex_reg_win_or_curwin(void) {
  return (void *)(rex.reg_win == NULL ? curwin : rex.reg_win);
}
int nvim_regexp_has_rex_reg_win(void) { return rex.reg_win != NULL ? 1 : 0; }
int32_t nvim_regexp_get_win_line_count(void *wp) {
  return (int32_t)((win_T *)wp)->w_buffer->b_ml.ml_line_count;
}
int32_t nvim_regexp_get_rex_reg_win_cursor_lnum(void) {
  return rex.reg_win != NULL ? (int32_t)rex.reg_win->w_cursor.lnum : 0;
}
int32_t nvim_regexp_get_rex_reg_win_cursor_col(void) {
  return rex.reg_win != NULL ? (int32_t)rex.reg_win->w_cursor.col : 0;
}
// Error/utility
void nvim_regexp_emsg_maxmempattern(void) {
  emsg(_(e_pattern_uses_more_memory_than_maxmempattern));
}
int nvim_regexp_call_profile_passed_limit(const void *tm) {
  return profile_passed_limit(*(const proftime_T *)tm) ? 1 : 0;
}

// mb_get_class_tab accessor
int nvim_regexp_call_mb_get_class_tab(uint8_t *p) {
  return mb_get_class_tab((char *)p, rex.reg_buf->b_chartab);
}

void nvim_regexp_internal_error(const char *msg) { internal_error(msg); }

// z-subexpr element-pointer accessors for save_se/restore_se in rs_regmatch
lpos_T *nvim_regexp_get_reg_startzpos_ptr(int i) { return &reg_startzpos[i]; }
lpos_T *nvim_regexp_get_reg_endzpos_ptr(int i) { return &reg_endzpos[i]; }
uint8_t **nvim_regexp_get_reg_startzp_ptr(int i) { return &reg_startzp[i]; }
uint8_t **nvim_regexp_get_reg_endzp_ptr(int i) { return &reg_endzp[i]; }

// Thin wrappers: BT engine vtable entry points delegate to Rust.
static int bt_regexec_nl(regmatch_T *rmp, uint8_t *line, colnr_T col, bool line_lbr)
{
  return rs_bt_regexec_nl(rmp, line, col, line_lbr);
}

static int bt_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum, colnr_T col,
                            proftime_T *tm, int *timed_out)
{
  return rs_bt_regexec_multi(rmp, win, buf, lnum, col, tm, timed_out);
}

// NFA engine opcodes and definitions

// Added to NFA_ANY - NFA_NUPPER_IC to include a NL.
#define NFA_ADD_NL              31

enum {
  NFA_SPLIT = -1024,
  NFA_MATCH,
  NFA_EMPTY,                        // matches 0-length

  NFA_START_COLL,                   // [abc] start
  NFA_END_COLL,                     // [abc] end
  NFA_START_NEG_COLL,               // [^abc] start
  NFA_END_NEG_COLL,                 // [^abc] end (postfix only)
  NFA_RANGE,                        // range of the two previous items
                                    // (postfix only)
  NFA_RANGE_MIN,                    // low end of a range
  NFA_RANGE_MAX,                    // high end of a range

  NFA_CONCAT,                       // concatenate two previous items (postfix
                                    // only)
  NFA_OR,                           // \| (postfix only)
  NFA_STAR,                         // greedy * (postfix only)
  NFA_STAR_NONGREEDY,               // non-greedy * (postfix only)
  NFA_QUEST,                        // greedy \? (postfix only)
  NFA_QUEST_NONGREEDY,              // non-greedy \? (postfix only)

  NFA_BOL,                          // ^    Begin line
  NFA_EOL,                          // $    End line
  NFA_BOW,                          // \<   Begin word
  NFA_EOW,                          // \>   End word
  NFA_BOF,                          // \%^  Begin file
  NFA_EOF,                          // \%$  End file
  NFA_NEWL,
  NFA_ZSTART,                       // Used for \zs
  NFA_ZEND,                         // Used for \ze
  NFA_NOPEN,                        // Start of subexpression marked with \%(
  NFA_NCLOSE,                       // End of subexpr. marked with \%( ... \)
  NFA_START_INVISIBLE,
  NFA_START_INVISIBLE_FIRST,
  NFA_START_INVISIBLE_NEG,
  NFA_START_INVISIBLE_NEG_FIRST,
  NFA_START_INVISIBLE_BEFORE,
  NFA_START_INVISIBLE_BEFORE_FIRST,
  NFA_START_INVISIBLE_BEFORE_NEG,
  NFA_START_INVISIBLE_BEFORE_NEG_FIRST,
  NFA_START_PATTERN,
  NFA_END_INVISIBLE,
  NFA_END_INVISIBLE_NEG,
  NFA_END_PATTERN,
  NFA_COMPOSING,                    // Next nodes in NFA are part of the
                                    // composing multibyte char
  NFA_END_COMPOSING,                // End of a composing char in the NFA
  NFA_ANY_COMPOSING,                // \%C: Any composing characters.
  NFA_OPT_CHARS,                    // \%[abc]

  // The following are used only in the postfix form, not in the NFA
  NFA_PREV_ATOM_NO_WIDTH,           // Used for \@=
  NFA_PREV_ATOM_NO_WIDTH_NEG,       // Used for \@!
  NFA_PREV_ATOM_JUST_BEFORE,        // Used for \@<=
  NFA_PREV_ATOM_JUST_BEFORE_NEG,    // Used for \@<!
  NFA_PREV_ATOM_LIKE_PATTERN,       // Used for \@>

  NFA_BACKREF1,                     // \1
  NFA_BACKREF2,                     // \2
  NFA_BACKREF3,                     // \3
  NFA_BACKREF4,                     // \4
  NFA_BACKREF5,                     // \5
  NFA_BACKREF6,                     // \6
  NFA_BACKREF7,                     // \7
  NFA_BACKREF8,                     // \8
  NFA_BACKREF9,                     // \9
  NFA_ZREF1,                        // \z1
  NFA_ZREF2,                        // \z2
  NFA_ZREF3,                        // \z3
  NFA_ZREF4,                        // \z4
  NFA_ZREF5,                        // \z5
  NFA_ZREF6,                        // \z6
  NFA_ZREF7,                        // \z7
  NFA_ZREF8,                        // \z8
  NFA_ZREF9,                        // \z9
  NFA_SKIP,                         // Skip characters

  NFA_MOPEN,
  NFA_MOPEN1,
  NFA_MOPEN2,
  NFA_MOPEN3,
  NFA_MOPEN4,
  NFA_MOPEN5,
  NFA_MOPEN6,
  NFA_MOPEN7,
  NFA_MOPEN8,
  NFA_MOPEN9,

  NFA_MCLOSE,
  NFA_MCLOSE1,
  NFA_MCLOSE2,
  NFA_MCLOSE3,
  NFA_MCLOSE4,
  NFA_MCLOSE5,
  NFA_MCLOSE6,
  NFA_MCLOSE7,
  NFA_MCLOSE8,
  NFA_MCLOSE9,

  NFA_ZOPEN,
  NFA_ZOPEN1,
  NFA_ZOPEN2,
  NFA_ZOPEN3,
  NFA_ZOPEN4,
  NFA_ZOPEN5,
  NFA_ZOPEN6,
  NFA_ZOPEN7,
  NFA_ZOPEN8,
  NFA_ZOPEN9,

  NFA_ZCLOSE,
  NFA_ZCLOSE1,
  NFA_ZCLOSE2,
  NFA_ZCLOSE3,
  NFA_ZCLOSE4,
  NFA_ZCLOSE5,
  NFA_ZCLOSE6,
  NFA_ZCLOSE7,
  NFA_ZCLOSE8,
  NFA_ZCLOSE9,

  // NFA_FIRST_NL
  NFA_ANY,              //      Match any one character.
  NFA_IDENT,            //      Match identifier char
  NFA_SIDENT,           //      Match identifier char but no digit
  NFA_KWORD,            //      Match keyword char
  NFA_SKWORD,           //      Match word char but no digit
  NFA_FNAME,            //      Match file name char
  NFA_SFNAME,           //      Match file name char but no digit
  NFA_PRINT,            //      Match printable char
  NFA_SPRINT,           //      Match printable char but no digit
  NFA_WHITE,            //      Match whitespace char
  NFA_NWHITE,           //      Match non-whitespace char
  NFA_DIGIT,            //      Match digit char
  NFA_NDIGIT,           //      Match non-digit char
  NFA_HEX,              //      Match hex char
  NFA_NHEX,             //      Match non-hex char
  NFA_OCTAL,            //      Match octal char
  NFA_NOCTAL,           //      Match non-octal char
  NFA_WORD,             //      Match word char
  NFA_NWORD,            //      Match non-word char
  NFA_HEAD,             //      Match head char
  NFA_NHEAD,            //      Match non-head char
  NFA_ALPHA,            //      Match alpha char
  NFA_NALPHA,           //      Match non-alpha char
  NFA_LOWER,            //      Match lowercase char
  NFA_NLOWER,           //      Match non-lowercase char
  NFA_UPPER,            //      Match uppercase char
  NFA_NUPPER,           //      Match non-uppercase char
  NFA_LOWER_IC,         //      Match [a-z]
  NFA_NLOWER_IC,        //      Match [^a-z]
  NFA_UPPER_IC,         //      Match [A-Z]
  NFA_NUPPER_IC,        //      Match [^A-Z]

  NFA_FIRST_NL = NFA_ANY + NFA_ADD_NL,
  NFA_LAST_NL = NFA_NUPPER_IC + NFA_ADD_NL,

  NFA_CURSOR,           //      Match cursor pos
  NFA_LNUM,             //      Match line number
  NFA_LNUM_GT,          //      Match > line number
  NFA_LNUM_LT,          //      Match < line number
  NFA_COL,              //      Match cursor column
  NFA_COL_GT,           //      Match > cursor column
  NFA_COL_LT,           //      Match < cursor column
  NFA_VCOL,             //      Match cursor virtual column
  NFA_VCOL_GT,          //      Match > cursor virtual column
  NFA_VCOL_LT,          //      Match < cursor virtual column
  NFA_MARK,             //      Match mark
  NFA_MARK_GT,          //      Match > mark
  NFA_MARK_LT,          //      Match < mark
  NFA_VISUAL,           //      Match Visual area

  // Character classes [:alnum:] etc
  NFA_CLASS_ALNUM,
  NFA_CLASS_ALPHA,
  NFA_CLASS_BLANK,
  NFA_CLASS_CNTRL,
  NFA_CLASS_DIGIT,
  NFA_CLASS_GRAPH,
  NFA_CLASS_LOWER,
  NFA_CLASS_PRINT,
  NFA_CLASS_PUNCT,
  NFA_CLASS_SPACE,
  NFA_CLASS_UPPER,
  NFA_CLASS_XDIGIT,
  NFA_CLASS_TAB,
  NFA_CLASS_RETURN,
  NFA_CLASS_BACKSPACE,
  NFA_CLASS_ESCAPE,
  NFA_CLASS_IDENT,
  NFA_CLASS_KEYWORD,
  NFA_CLASS_FNAME,
};

// Keep in sync with classchars.
static int nfa_classcodes[] = {
  NFA_ANY, NFA_IDENT, NFA_SIDENT, NFA_KWORD, NFA_SKWORD,
  NFA_FNAME, NFA_SFNAME, NFA_PRINT, NFA_SPRINT,
  NFA_WHITE, NFA_NWHITE, NFA_DIGIT, NFA_NDIGIT,
  NFA_HEX, NFA_NHEX, NFA_OCTAL, NFA_NOCTAL,
  NFA_WORD, NFA_NWORD, NFA_HEAD, NFA_NHEAD,
  NFA_ALPHA, NFA_NALPHA, NFA_LOWER, NFA_NLOWER,
  NFA_UPPER, NFA_NUPPER
};

static const char e_nul_found[] = N_("E865: (NFA) Regexp end encountered prematurely");
static const char e_misplaced[] = N_("E866: (NFA regexp) Misplaced %c");
static const char e_ill_char_class[] = N_("E877: (NFA regexp) Invalid character class: %" PRId64);
static const char e_value_too_large[] = N_("E951: \\% value too large");

void nvim_regexp_emsg_nul_found(void)
{
  emsg(_(e_nul_found));
  rc_did_emsg = true;
}
void nvim_regexp_semsg_misplaced(int c) { semsg(_(e_misplaced), (char)c); }
void nvim_regexp_semsg_ill_char_class(int64_t c)
{
  semsg(_(e_ill_char_class), c);
  rc_did_emsg = true;
}
void nvim_regexp_siemsg_unknown_class(int64_t c) { siemsg("INTERNAL: Unknown character class char: %" PRId64, c); }
void nvim_regexp_semsg_e867_z(int c) { semsg(_("E867: (NFA) Unknown operator '\\z%c'"), c); }
void nvim_regexp_semsg_e867_pct(int c) { semsg(_("E867: (NFA) Unknown operator '\\%%%c'"), c); }
void nvim_regexp_emsg_value_too_large(void) { emsg(_(e_value_too_large)); }
void nvim_regexp_semsg_missing_value(int c) { semsg(_(e_nfa_regexp_missing_value_in_chr), c); }
uint8_t *nvim_regexp_get_classchars(void) { return classchars; }
int nvim_regexp_get_nfa_classcodes(int index) { return nfa_classcodes[index]; }
char *nvim_regexp_get_regexp_inrange(void) { return REGEXP_INRANGE; }
char *nvim_regexp_get_regexp_abbr(void) { return REGEXP_ABBR; }
void nvim_regexp_set_rc_did_emsg_true(void) { rc_did_emsg = true; }

void nvim_regexp_semsg_e869(int op) { semsg(_("E869: (NFA) Unknown operator '\\@%c'"), op); }
void nvim_regexp_emsg_e870(void)
{
  emsg(_("E870: (NFA regexp) Error reading repetition limits"));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e871(void)
{
  emsg(_("E871: (NFA regexp) Can't have a multi follow a multi"));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e872(void)
{
  emsg(_("E872: (NFA regexp) Too many '('"));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e879(void)
{
  emsg(_("E879: (NFA regexp) Too many \\z("));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e873(void)
{
  emsg(_("E873: (NFA regexp) proper termination error"));
  rc_did_emsg = true;
}

// nfa_state_T field accessors
int nvim_nfa_state_get_c(void *s) { return ((nfa_state_T *)s)->c; }
void nvim_nfa_state_set_c(void *s, int v) { ((nfa_state_T *)s)->c = v; }
void *nvim_nfa_state_get_out(void *s) { return (void *)((nfa_state_T *)s)->out; }
void nvim_nfa_state_set_out(void *s, void *v) { ((nfa_state_T *)s)->out = (nfa_state_T *)v; }
void *nvim_nfa_state_get_out1(void *s) { return (void *)((nfa_state_T *)s)->out1; }
void nvim_nfa_state_set_out1(void *s, void *v) { ((nfa_state_T *)s)->out1 = (nfa_state_T *)v; }
int nvim_nfa_state_get_val(void *s) { return ((nfa_state_T *)s)->val; }
void nvim_nfa_state_set_val(void *s, int v) { ((nfa_state_T *)s)->val = v; }
void nvim_nfa_state_set_id(void *s, int v) { ((nfa_state_T *)s)->id = v; }
void nvim_nfa_state_clear_lastlist(void *s)
{
  ((nfa_state_T *)s)->lastlist[0] = 0;
  ((nfa_state_T *)s)->lastlist[1] = 0;
}

// Address-of accessors for NFA state pointer manipulation
void **nvim_nfa_state_out_addr(void *s) { return (void **)&((nfa_state_T *)s)->out; }
void **nvim_nfa_state_out1_addr(void *s) { return (void **)&((nfa_state_T *)s)->out1; }

// Error messages for post2nfa
void nvim_regexp_emsg_e874(void) { emsg(_("E874: (NFA) Could not pop the stack!")); }
void nvim_regexp_emsg_e875(void)
{
  emsg(_("E875: (NFA regexp) (While converting from postfix to NFA),"
         "too many states left on stack"));
  rc_did_emsg = true;
}
void nvim_regexp_emsg_e876(void)
{
  emsg(_("E876: (NFA regexp) "
         "Not enough space to store the whole NFA "));
  rc_did_emsg = true;
}

extern void *rs_nfa_regcomp(uint8_t *expr, int re_flags);

// nfa_regprog_T field accessors
int nvim_nfa_prog_get_nstate(void *prog) { return ((nfa_regprog_T *)prog)->nstate; }
void *nvim_nfa_prog_get_state(void *prog, int i) { return (void *)&((nfa_regprog_T *)prog)->state[i]; }
void *nvim_nfa_prog_get_start(void *prog) { return (void *)((nfa_regprog_T *)prog)->start; }
void nvim_nfa_prog_set_has_zend(void *prog, int v) { ((nfa_regprog_T *)prog)->has_zend = v; }
void nvim_nfa_prog_set_has_backref(void *prog, int v) { ((nfa_regprog_T *)prog)->has_backref = v; }
void nvim_nfa_prog_set_nsubexp(void *prog, int v) { ((nfa_regprog_T *)prog)->nsubexp = v; }
void nvim_nfa_prog_set_regflags(void *prog, int v) { ((nfa_regprog_T *)prog)->regflags = (unsigned)v; }
void nvim_nfa_prog_set_reganch(void *prog, int v) { ((nfa_regprog_T *)prog)->reganch = v; }
void nvim_nfa_prog_set_regstart(void *prog, int v) { ((nfa_regprog_T *)prog)->regstart = v; }
void nvim_nfa_prog_set_match_text(void *prog, uint8_t *v) { ((nfa_regprog_T *)prog)->match_text = v; }
void nvim_nfa_prog_set_reghasz(void *prog, int v) { ((nfa_regprog_T *)prog)->reghasz = v; }
void nvim_nfa_prog_set_pattern(void *prog, char *v) { ((nfa_regprog_T *)prog)->pattern = v; }

int nvim_regexp_get_rex_nfa_has_zend(void) { return rex.nfa_has_zend; }
int nvim_regexp_get_rex_nfa_has_backref(void) { return rex.nfa_has_backref; }
void nvim_regexp_set_nfa_prog_engine(void *prog) { ((nfa_regprog_T *)prog)->engine = &nfa_regengine; }
void nvim_nfa_prog_set_re_in_use(void *prog, int v) { ((nfa_regprog_T *)prog)->re_in_use = (bool)v; }
void nvim_nfa_prog_set_start(void *prog, void *s) { ((nfa_regprog_T *)prog)->start = (nfa_state_T *)s; }
void nvim_nfa_prog_set_nstate(void *prog, int v) { ((nfa_regprog_T *)prog)->nstate = v; }
char *nvim_regexp_xstrdup(const char *s) { return xstrdup(s); }

// --- NFA Execution accessor functions ---

// nfa_state_T id/lastlist accessors
int nvim_nfa_state_get_id(void *s) { return ((nfa_state_T *)s)->id; }
int nvim_nfa_state_get_lastlist(void *s, int idx) { return ((nfa_state_T *)s)->lastlist[idx]; }
void nvim_nfa_state_set_lastlist(void *s, int idx, int val) { ((nfa_state_T *)s)->lastlist[idx] = val; }

// nfa_regprog_T execution field accessors
int nvim_nfa_prog_get_has_zend(void *prog) { return ((nfa_regprog_T *)prog)->has_zend; }
int nvim_nfa_prog_get_has_backref(void *prog) { return ((nfa_regprog_T *)prog)->has_backref; }
int nvim_nfa_prog_get_nsubexp(void *prog) { return ((nfa_regprog_T *)prog)->nsubexp; }
int nvim_nfa_prog_get_reghasz(void *prog) { return ((nfa_regprog_T *)prog)->reghasz; }
int nvim_nfa_prog_get_regflags(void *prog) { return (int)((nfa_regprog_T *)prog)->regflags; }
int nvim_nfa_prog_get_regstart(void *prog) { return ((nfa_regprog_T *)prog)->regstart; }
int nvim_nfa_prog_get_reganch(void *prog) { return ((nfa_regprog_T *)prog)->reganch; }
uint8_t *nvim_nfa_prog_get_match_text(void *prog) { return ((nfa_regprog_T *)prog)->match_text; }

// siemsg wrapper for check_char_class
void nvim_regexp_siemsg_ill_char_class(int64_t cls) { siemsg(_(e_ill_char_class), cls); }
// regsub_T field accessors
int nvim_regexp_regsub_get_in_use(void *sub) { return ((regsub_T *)sub)->in_use; }
int32_t nvim_regexp_regsub_get_multi_start_lnum(void *sub, int idx) { return (int32_t)((regsub_T *)sub)->list.multi[idx].start_lnum; }
int32_t nvim_regexp_regsub_get_multi_start_col(void *sub, int idx) { return (int32_t)((regsub_T *)sub)->list.multi[idx].start_col; }
int32_t nvim_regexp_regsub_get_multi_end_lnum(void *sub, int idx) { return (int32_t)((regsub_T *)sub)->list.multi[idx].end_lnum; }
int32_t nvim_regexp_regsub_get_multi_end_col(void *sub, int idx) { return (int32_t)((regsub_T *)sub)->list.multi[idx].end_col; }
uint8_t *nvim_regexp_regsub_get_line_start(void *sub, int idx) { return ((regsub_T *)sub)->list.line[idx].start; }
uint8_t *nvim_regexp_regsub_get_line_end(void *sub, int idx) { return ((regsub_T *)sub)->list.line[idx].end; }

// nfa_pim_T field accessors
int nvim_nfa_pim_get_result(void *pim) { return ((nfa_pim_T *)pim)->result; }
int nvim_nfa_pim_get_state_id(void *pim) { return ((nfa_pim_T *)pim)->state->id; }
int32_t nvim_nfa_pim_get_end_pos_lnum(void *pim) { return (int32_t)((nfa_pim_T *)pim)->end.pos.lnum; }
int32_t nvim_nfa_pim_get_end_pos_col(void *pim) { return (int32_t)((nfa_pim_T *)pim)->end.pos.col; }
uint8_t *nvim_nfa_pim_get_end_ptr(void *pim) { return ((nfa_pim_T *)pim)->end.ptr; }

// nfa_list_T / nfa_thread_T read accessors
int nvim_nfa_list_get_n(void *l) { return ((nfa_list_T *)l)->n; }
int nvim_nfa_list_get_id(void *l) { return ((nfa_list_T *)l)->id; }
int nvim_nfa_thread_get_state_id(void *l, int idx) { return ((nfa_list_T *)l)->t[idx].state->id; }
void *nvim_nfa_thread_get_subs_norm(void *l, int idx) { return (void *)&((nfa_list_T *)l)->t[idx].subs.norm; }
void *nvim_nfa_thread_get_subs_synt(void *l, int idx) { return (void *)&((nfa_list_T *)l)->t[idx].subs.synt; }
void *nvim_nfa_thread_get_pim_ptr(void *l, int idx) { return (void *)&((nfa_list_T *)l)->t[idx].pim; }

int nvim_regexp_get_nfa_has_zsubexpr(void) { return rex.nfa_has_zsubexpr; }

// Variables only used in nfa_regcomp() and descendants.
static int nfa_re_flags;  ///< re_flags passed to nfa_regcomp().
static int *post_start;   ///< holds the postfix form of r.e.
static int *post_end;
static int *post_ptr;

static int nstate;  ///< Number of states in the NFA. Also used when executing.
static int istate;  ///< Index in the state vector, used in alloc_state()

// If not NULL match must end at this position
static save_se_T *nfa_endp = NULL;

// 0 for first call to nfa_regmatch(), 1 for recursive call.
static int nfa_ll_index = 0;

// --- NFA accessor functions for Rust FFI ---
int *nvim_regexp_get_post_start(void) { return post_start; }
void nvim_regexp_set_post_start(int *p) { post_start = p; }
int *nvim_regexp_get_post_ptr(void) { return post_ptr; }
void nvim_regexp_set_post_ptr(int *p) { post_ptr = p; }
int *nvim_regexp_get_post_end(void) { return post_end; }
void nvim_regexp_set_post_end(int *p) { post_end = p; }

int nvim_regexp_get_nstate(void) { return nstate; }
void nvim_regexp_set_nstate(int v) { nstate = v; }
int nvim_regexp_get_istate(void) { return istate; }
void nvim_regexp_set_istate(int v) { istate = v; }

int nvim_regexp_get_nfa_re_flags(void) { return nfa_re_flags; }
void nvim_regexp_set_nfa_re_flags(int v) { nfa_re_flags = v; }

void nvim_regexp_set_rex_nfa_has_zend(int v) { rex.nfa_has_zend = v; }
void nvim_regexp_set_rex_nfa_has_backref(int v) { rex.nfa_has_backref = v; }

static nfa_state_T *state_ptr;  // points to nfa_prog->state

// state_ptr accessors
void *nvim_regexp_get_state_ptr(void) { return (void *)state_ptr; }
void nvim_regexp_set_state_ptr(void *v) { state_ptr = (nfa_state_T *)v; }
void *nvim_regexp_state_ptr_add(int index) { return (void *)&state_ptr[index]; }

// NFA prog allocation (needs state_ptr)
void *nvim_regexp_alloc_nfa_prog(int nstate_count)
{
  size_t prog_size = offsetof(nfa_regprog_T, state) + sizeof(nfa_state_T) * (size_t)nstate_count;
  nfa_regprog_T *prog = xmalloc(prog_size);
  state_ptr = prog->state;
  return prog;
}
/////////////////////////////////////////////////////////////////
// NFA execution code.
/////////////////////////////////////////////////////////////////

// Used during execution: whether a match has been found.
static int nfa_match;
static proftime_T *nfa_time_limit;
static int *nfa_timed_out;
static int nfa_time_count;
// NFA execution globals accessors
int nvim_regexp_get_nfa_match(void) { return nfa_match; }
void nvim_regexp_set_nfa_match(int v) { nfa_match = v; }
int nvim_regexp_get_nfa_ll_index(void) { return nfa_ll_index; }
void nvim_regexp_set_nfa_ll_index(int v) { nfa_ll_index = v; }
// nfa_endp accessor
void *nvim_regexp_get_nfa_endp(void) { return (void *)nfa_endp; }
void nvim_regexp_set_nfa_endp(void *v) { nfa_endp = (save_se_T *)v; }

// nfa_endp field accessors
int32_t nvim_regexp_get_nfa_endp_pos_lnum(void) { return nfa_endp ? (int32_t)nfa_endp->se_u.pos.lnum : -1; }
int32_t nvim_regexp_get_nfa_endp_pos_col(void) { return nfa_endp ? (int32_t)nfa_endp->se_u.pos.col : -1; }
uint8_t *nvim_regexp_get_nfa_endp_ptr(void) { return nfa_endp ? nfa_endp->se_u.ptr : NULL; }

// nfa_list_T memory management
void *nvim_nfa_list_alloc_threads(int nstate)
{
  nfa_list_T *l = xcalloc(1, sizeof(nfa_list_T));
  l->t = xmalloc(sizeof(nfa_thread_T) * (size_t)nstate);
  l->n = 0;
  l->has_pim = false;
  l->id = 0;
  return (void *)l;
}
void nvim_nfa_list_free_threads(void *l)
{
  if (l) {
    xfree(((nfa_list_T *)l)->t);
    xfree(l);
  }
}

// nfa_time_limit / nfa_timed_out / nfa_time_count accessors
void *nvim_regexp_get_nfa_time_limit(void) { return (void *)nfa_time_limit; }
void nvim_regexp_set_nfa_time_limit(void *v) { nfa_time_limit = (proftime_T *)v; }
int *nvim_regexp_get_nfa_timed_out(void) { return nfa_timed_out; }
void nvim_regexp_set_nfa_timed_out(int *v) { nfa_timed_out = v; }
int nvim_regexp_get_nfa_time_count(void) { return nfa_time_count; }
void nvim_regexp_set_nfa_time_count(int v) { nfa_time_count = v; }

// Thread field accessors for the main loop
int nvim_nfa_thread_get_state_c(void *l, int idx) { return ((nfa_list_T *)l)->t[idx].state->c; }
void *nvim_nfa_thread_get_state_ptr(void *l, int idx) { return (void *)((nfa_list_T *)l)->t[idx].state; }
void *nvim_nfa_thread_get_state_out(void *l, int idx) { return (void *)((nfa_list_T *)l)->t[idx].state->out; }
void *nvim_nfa_thread_get_state_out1(void *l, int idx) { return (void *)((nfa_list_T *)l)->t[idx].state->out1; }
int nvim_nfa_thread_get_state_val(void *l, int idx) { return ((nfa_list_T *)l)->t[idx].state->val; }
int nvim_nfa_thread_get_count(void *l, int idx) { return ((nfa_list_T *)l)->t[idx].count; }
void nvim_nfa_thread_set_count(void *l, int idx, int v) { ((nfa_list_T *)l)->t[idx].count = v; }
void *nvim_nfa_thread_get_subs_ptr(void *l, int idx) { return (void *)&((nfa_list_T *)l)->t[idx].subs; }

// Thread PIM field accessors
int nvim_nfa_thread_get_pim_result(void *l, int idx) { return ((nfa_list_T *)l)->t[idx].pim.result; }
void *nvim_nfa_thread_get_pim_state(void *l, int idx) { return (void *)((nfa_list_T *)l)->t[idx].pim.state; }
int nvim_nfa_thread_get_pim_state_c(void *l, int idx) { return ((nfa_list_T *)l)->t[idx].pim.state->c; }

// nfa_list_T management
void nvim_nfa_list_set_n(void *l, int n) { ((nfa_list_T *)l)->n = n; }
void nvim_nfa_list_set_has_pim(void *l, int v) { ((nfa_list_T *)l)->has_pim = v; }
void nvim_nfa_list_set_id(void *l, int id) { ((nfa_list_T *)l)->id = id; }

// regsubs_T operations (bulk operations on submatch/m params)
void *nvim_regexp_regsubs_get_norm(void *s) { return (void *)&((regsubs_T *)s)->norm; }
void *nvim_regexp_regsubs_get_synt(void *s) { return (void *)&((regsubs_T *)s)->synt; }
int nvim_regexp_regsubs_get_norm_in_use(void *s) { return ((regsubs_T *)s)->norm.in_use; }
void nvim_regexp_regsubs_set_norm_in_use(void *s, int v) { ((regsubs_T *)s)->norm.in_use = v; }

// regsubs_T multi-line startpos operations for inline addstate optimization
void nvim_regexp_regsubs_set_multi_start(void *s, int idx, int32_t lnum, int32_t col)
{
  ((regsubs_T *)s)->norm.list.multi[idx].start_lnum = lnum;
  ((regsubs_T *)s)->norm.list.multi[idx].start_col = col;
}
int32_t nvim_regexp_regsubs_get_multi_start_col(void *s, int idx) { return ((regsubs_T *)s)->norm.list.multi[idx].start_col; }
int32_t nvim_regexp_regsubs_get_multi_end_col(void *s, int idx) { return ((regsubs_T *)s)->norm.list.multi[idx].end_col; }
void nvim_regexp_regsubs_set_norm_orig_start_col(void *s, int32_t v) { ((regsubs_T *)s)->norm.orig_start_col = v; }
void nvim_regexp_regsubs_set_line_start(void *s, int idx, uint8_t *ptr) { ((regsubs_T *)s)->norm.list.line[idx].start = ptr; }
uint8_t *nvim_regexp_regsubs_get_line_end(void *s, int idx) { return ((regsubs_T *)s)->norm.list.line[idx].end; }

// rex execution field accessors for nfa_regmatch
int nvim_regexp_get_rex_nfa_listid(void) { return rex.nfa_listid; }
void nvim_regexp_set_rex_nfa_listid(int v) { rex.nfa_listid = v; }
int32_t nvim_regexp_get_rex_reg_maxcol(void) { return (int32_t)rex.reg_maxcol; }
int nvim_regexp_get_rex_nfa_nsubexpr(void) { return rex.nfa_nsubexpr; }

// NFA prog field accessors for nfa_regmatch
int nvim_nfa_prog_get_re_engine(void *prog) { return ((nfa_regprog_T *)prog)->re_engine; }

// PIM operations for nfa_regmatch post-switch logic
void nvim_nfa_pim_set_result(void *pim, int v) { ((nfa_pim_T *)pim)->result = v; }
void *nvim_nfa_pim_get_state(void *pim) { return (void *)((nfa_pim_T *)pim)->state; }
int nvim_nfa_pim_get_state_c(void *pim) { return ((nfa_pim_T *)pim)->state->c; }
void *nvim_nfa_pim_get_subs_norm(void *pim) { return (void *)&((nfa_pim_T *)pim)->subs.norm; }
void *nvim_nfa_pim_get_subs_synt(void *pim) { return (void *)&((nfa_pim_T *)pim)->subs.synt; }

// Allocate/init a temporary nfa_pim_T on the C side (for the PIM deferral path)
void *nvim_regexp_alloc_pim(void)
{
  nfa_pim_T *pim = xcalloc(1, sizeof(nfa_pim_T));
  return (void *)pim;
}
void nvim_regexp_free_pim(void *p) { xfree(p); }
void nvim_regexp_pim_init(void *p, void *state, int result,
                          int32_t lnum, int32_t col, uint8_t *ptr,
                          int is_multi)
{
  nfa_pim_T *pim = (nfa_pim_T *)p;
  pim->state = (nfa_state_T *)state;
  pim->result = result;
  pim->subs.norm.in_use = 0;
  pim->subs.synt.in_use = 0;
  if (is_multi) {
    pim->end.pos.lnum = lnum;
    pim->end.pos.col = col;
  } else {
    pim->end.ptr = ptr;
  }
}

// win_T and buffer accessors for VCOL/MARK cases
void *nvim_regexp_get_curwin(void) { return (void *)curwin; }
int64_t nvim_regexp_get_win_b_p_ts(void *wp) { return (int64_t)((win_T *)wp)->w_buffer->b_p_ts; }
int32_t nvim_regexp_get_win_buf_line_count(void *wp) { return (int32_t)((win_T *)wp)->w_buffer->b_ml.ml_line_count; }

// Mark access for NFA_MARK cases
void *nvim_regexp_call_mark_get_for_nfa(void *buf, void *win, int mark_val) { return (void *)mark_get((buf_T *)buf, (win_T *)win, NULL, kMarkBufLocal, mark_val); }
int nvim_regexp_fmark_is_set(void *fm) { return fm != NULL && ((fmark_T *)fm)->mark.lnum > 0; }
int32_t nvim_regexp_fmark_get_lnum(void *fm) { return (int32_t)((fmark_T *)fm)->mark.lnum; }
int32_t nvim_regexp_fmark_get_col(void *fm) { return (int32_t)((fmark_T *)fm)->mark.col; }
int32_t nvim_regexp_fmark_get_col_adj(void *fm, int32_t lnum_match)
{
  fmark_T *f = (fmark_T *)fm;
  if (f->mark.lnum == lnum_match && f->mark.col == MAXCOL) {
    return (int32_t)reg_getline_len(f->mark.lnum - rex.reg_firstlnum);
  }
  return (int32_t)f->mark.col;
}

// Nextlist thread count setter (for add_count after addstate)
void nvim_nfa_list_set_last_thread_count(void *l, int count)
{
  nfa_list_T *list = (nfa_list_T *)l;
  list->t[list->n - 1].count = count;
}

void nvim_regexp_xfree(void *p) { xfree(p); }
void *nvim_regexp_xmalloc(size_t size) { return xmalloc(size); }

// nfa_alt_listid accessors (for recursive_regmatch in Rust)
int nvim_regexp_get_rex_nfa_alt_listid(void) { return rex.nfa_alt_listid; }
void nvim_regexp_set_rex_nfa_alt_listid(int v) { rex.nfa_alt_listid = v; }

// Allocate/free regsubs_T on the heap (Rust cannot stack-allocate opaque C structs)
void *nvim_regexp_alloc_regsubs(void) { return xcalloc(1, sizeof(regsubs_T)); }
void nvim_regexp_free_regsubs(void *s) { xfree(s); }

// nfa_regtry setup: set rex.input and NFA time fields
void nvim_regexp_nfa_regtry_setup(void *prog, int32_t col, void *tm, int *timed_out) {
  rex.input = rex.line + col;
  nfa_time_limit = (proftime_T *)tm;
  nfa_timed_out = timed_out;
  nfa_time_count = 0;
}

// nfa_regtry: extract submatch data from subs into rex fields (multi-line mode)
void nvim_regexp_nfa_regtry_extract_multi(void *subs_ptr, int32_t col) {
  regsubs_T *subs = (regsubs_T *)subs_ptr;
  for (int i = 0; i < subs->norm.in_use; i++) {
    rex.reg_startpos[i].lnum = subs->norm.list.multi[i].start_lnum;
    rex.reg_startpos[i].col = subs->norm.list.multi[i].start_col;
    rex.reg_endpos[i].lnum = subs->norm.list.multi[i].end_lnum;
    rex.reg_endpos[i].col = subs->norm.list.multi[i].end_col;
  }
  if (rex.reg_mmatch != NULL) {
    rex.reg_mmatch->rmm_matchcol = subs->norm.orig_start_col;
  }
  if (rex.reg_startpos[0].lnum < 0) {
    rex.reg_startpos[0].lnum = 0;
    rex.reg_startpos[0].col = col;
  }
  if (rex.reg_endpos[0].lnum < 0) {
    rex.reg_endpos[0].lnum = rex.lnum;
    rex.reg_endpos[0].col = (int)(rex.input - rex.line);
  } else {
    rex.lnum = rex.reg_endpos[0].lnum;
  }
}

// nfa_regtry: extract submatch data (single-line mode)
void nvim_regexp_nfa_regtry_extract_single(void *subs_ptr, int32_t col) {
  regsubs_T *subs = (regsubs_T *)subs_ptr;
  for (int i = 0; i < subs->norm.in_use; i++) {
    rex.reg_startp[i] = subs->norm.list.line[i].start;
    rex.reg_endp[i] = subs->norm.list.line[i].end;
  }
  if (rex.reg_startp[0] == NULL) {
    rex.reg_startp[0] = rex.line + col;
  }
  if (rex.reg_endp[0] == NULL) {
    rex.reg_endp[0] = rex.input;
  }
}

// nfa_regtry: handle \z(...\) extmatch extraction
void nvim_regexp_nfa_regtry_extract_extmatch(void *subs_ptr) {
  regsubs_T *subs = (regsubs_T *)subs_ptr;
  rs_cleanup_zsubexpr();
  re_extmatch_out = rs_make_extmatch();
  for (int i = 1; i < subs->synt.in_use; i++) {
    if (REG_MULTI) {
      struct multipos *mpos = &subs->synt.list.multi[i];
      if (mpos->start_lnum >= 0
          && mpos->start_lnum == mpos->end_lnum
          && mpos->end_col >= mpos->start_col) {
        re_extmatch_out->matches[i] =
          (uint8_t *)xstrnsave(reg_getline(mpos->start_lnum) + mpos->start_col,
                               (size_t)(mpos->end_col - mpos->start_col));
      }
    } else {
      struct linepos *lpos = &subs->synt.list.line[i];
      if (lpos->start != NULL && lpos->end != NULL) {
        re_extmatch_out->matches[i] =
          (uint8_t *)xstrnsave((char *)lpos->start, (size_t)(lpos->end - lpos->start));
      }
    }
  }
}

// nfa_regexec_both: get the prog from rex.reg_match or rex.reg_mmatch
void *nvim_regexp_nfa_regexec_both_get_prog(void) {
  if (REG_MULTI) {
    return (void *)rex.reg_mmatch->regprog;
  } else {
    return (void *)rex.reg_match->regprog;
  }
}

// nfa_regexec_both: get line (calls reg_getline(0) for multi-line mode)
uint8_t *nvim_regexp_nfa_regexec_both_get_line(uint8_t *line) {
  if (REG_MULTI) {
    return (uint8_t *)reg_getline(0);
  }
  return line;
}

// nfa_regexec_both: set up rex pointer fields from match structs
void nvim_regexp_nfa_regexec_both_setup_pointers(void) {
  if (REG_MULTI) {
    rex.reg_startpos = rex.reg_mmatch->startpos;
    rex.reg_endpos = rex.reg_mmatch->endpos;
  } else {
    rex.reg_startp = (uint8_t **)rex.reg_match->startp;
    rex.reg_endp = (uint8_t **)rex.reg_match->endp;
  }
}

// nfa_regexec_both: apply regflags overrides
void nvim_regexp_nfa_regexec_both_apply_flags(void *prog_ptr) {
  nfa_regprog_T *prog = (nfa_regprog_T *)prog_ptr;
  if (prog->regflags & RF_ICASE) {
    rex.reg_ic = true;
  } else if (prog->regflags & RF_NOICASE) {
    rex.reg_ic = false;
  }
  if (prog->regflags & RF_ICOMBINE) {
    rex.reg_icombine = true;
  }
}

// nfa_regexec_both: set up NFA-specific rex fields from prog
void nvim_regexp_nfa_regexec_both_setup_nfa(void *prog_ptr) {
  nfa_regprog_T *prog = (nfa_regprog_T *)prog_ptr;
  rex.line = NULL;  // caller must set rex.line separately
  rex.lnum = 0;
  rex.nfa_has_zend = prog->has_zend;
  rex.nfa_has_backref = prog->has_backref;
  rex.nfa_nsubexpr = prog->nsubexp;
  rex.nfa_listid = 1;
  rex.nfa_alt_listid = 2;
  rex.need_clear_subexpr = true;
  if (prog->reghasz == REX_SET) {
    rex.nfa_has_zsubexpr = true;
    rex.need_clear_zsubexpr = true;
  } else {
    rex.nfa_has_zsubexpr = false;
    rex.need_clear_zsubexpr = false;
  }
}

// nfa_regexec_both: initialize state array (id and lastlist fields)
void nvim_regexp_nfa_regexec_both_init_states(void *prog_ptr) {
  nfa_regprog_T *prog = (nfa_regprog_T *)prog_ptr;
  nstate = 0;
  for (int i = 0; i < prog->nstate; i++) {
    prog->state[i].id = i;
    prog->state[i].lastlist[0] = 0;
    prog->state[i].lastlist[1] = 0;
  }
}

// nfa_regexec_both: post-match validation (ensure end >= start)
void nvim_regexp_nfa_regexec_both_validate_match(void) {
  if (REG_MULTI) {
    const lpos_T *start = &rex.reg_mmatch->startpos[0];
    const lpos_T *end = &rex.reg_mmatch->endpos[0];
    if (end->lnum < start->lnum
        || (end->lnum == start->lnum && end->col < start->col)) {
      rex.reg_mmatch->endpos[0] = rex.reg_mmatch->startpos[0];
    }
  } else {
    if (rex.reg_match->endp[0] < rex.reg_match->startp[0]) {
      rex.reg_match->endp[0] = rex.reg_match->startp[0];
    }
  }
}

// nfa_regexec_both: set rmm_matchcol or rm_matchcol
void nvim_regexp_nfa_regexec_both_set_matchcol(int32_t col) {
  if (REG_MULTI) {
    rex.reg_mmatch->rmm_matchcol = col;
  } else {
    rex.reg_match->rm_matchcol = col;
  }
}

// nfa_regexec_nl: set up rex fields for single-line NFA matching
void nvim_regexp_nfa_regexec_nl_setup(void *rmp, int line_lbr) {
  regmatch_T *rm = (regmatch_T *)rmp;
  rex.reg_match = rm;
  rex.reg_mmatch = NULL;
  rex.reg_maxline = 0;
  rex.reg_line_lbr = (bool)line_lbr;
  rex.reg_buf = curbuf;
  rex.reg_win = NULL;
  rex.reg_ic = rm->rm_ic;
  rex.reg_icombine = false;
  rex.reg_nobreak = rm->regprog->re_flags & RE_NOBREAK;
  rex.reg_maxcol = 0;
}

// nfa_regexec_multi: wraps init_regexec_multi
void nvim_regexp_call_init_regexec_multi(void *rmp, void *win, void *buf, int32_t lnum) {
  init_regexec_multi((regmmatch_T *)rmp, (win_T *)win, (buf_T *)buf, lnum);
}

// nfa_regexec_both: iemsg for null prog/line
void nvim_regexp_call_iemsg_null(void) { iemsg(_(e_null)); }

// bt_regexec_both accessors

// Init regstack and backpos if not allocated yet
void nvim_regexp_bt_init_stacks(void) {
  if (regstack.ga_data == NULL) {
    ga_init(&regstack, 1, REGSTACK_INITIAL);
    ga_grow(&regstack, REGSTACK_INITIAL);
    ga_set_growsize(&regstack, REGSTACK_INITIAL * 8);
  }
  if (backpos.ga_data == NULL) {
    ga_init(&backpos, sizeof(backpos_T), BACKPOS_INITIAL);
    ga_grow(&backpos, BACKPOS_INITIAL);
    ga_set_growsize(&backpos, BACKPOS_INITIAL * 8);
  }
}

// Cleanup stacks and reg_tofree after bt_regexec_both
void nvim_regexp_bt_cleanup_stacks(void) {
  if (reg_tofreelen > 400) {
    XFREE_CLEAR(reg_tofree);
  }
  if (regstack.ga_maxlen > REGSTACK_INITIAL) {
    ga_clear(&regstack);
  }
  if (backpos.ga_maxlen > BACKPOS_INITIAL) {
    ga_clear(&backpos);
  }
}

// bt_regprog_T field getters
uint8_t *nvim_bt_prog_get_regmust(const void *prog) { return ((const bt_regprog_T *)prog)->regmust; }
int nvim_bt_prog_get_regmlen(const void *prog) { return ((const bt_regprog_T *)prog)->regmlen; }
int nvim_bt_prog_get_regstart(const void *prog) { return ((const bt_regprog_T *)prog)->regstart; }
int nvim_bt_prog_get_reganch(const void *prog) { return ((const bt_regprog_T *)prog)->reganch; }
// vim_regfree + free_regexp_stuff accessors
void nvim_regexp_call_engine_regfree(void *prog) {
  ((regprog_T *)prog)->engine->regfree((regprog_T *)prog);
}

void nvim_regexp_call_free_regexp_stuff(void) {
  ga_clear(&regstack);
  ga_clear(&backpos);
  xfree(reg_tofree);
  xfree(reg_prev_sub);
}
// vim_regexec public API accessors

// rex save/restore: opaque buffer approach
// Rust stack-allocates a buffer of this size to save/restore rex + rex_in_use.
size_t nvim_regexp_get_rex_save_size(void) {
  return sizeof(regexec_T) + sizeof(bool);
}

void nvim_regexp_save_rex(void *out_buf) {
  char *p = (char *)out_buf;
  memcpy(p, &rex, sizeof(rex));
  memcpy(p + sizeof(rex), &rex_in_use, sizeof(rex_in_use));
}

void nvim_regexp_restore_rex(const void *saved_buf) {
  const char *p = (const char *)saved_buf;
  memcpy(&rex, p, sizeof(rex));
  memcpy(&rex_in_use, p + sizeof(rex), sizeof(rex_in_use));
}

int nvim_regexp_get_rex_in_use(void) { return rex_in_use ? 1 : 0; }
void nvim_regexp_set_rex_in_use(int v) { rex_in_use = (bool)v; }

// Clear rex pointer fields
void nvim_regexp_clear_rex_pointers(void) {
  rex.reg_startp = NULL;
  rex.reg_endp = NULL;
  rex.reg_startpos = NULL;
  rex.reg_endpos = NULL;
}

// Engine vtable dispatch
int nvim_regexp_call_engine_regexec_nl(void *prog, void *rmp, uint8_t *line, int32_t col, int nl) {
  return ((regprog_T *)prog)->engine->regexec_nl(
    (regmatch_T *)rmp, line, (colnr_T)col, (bool)nl);
}

int nvim_regexp_call_engine_regexec_multi(void *prog, void *rmp, void *win, void *buf,
                                          int32_t lnum, int32_t col, void *tm, int *timed_out) {
  return ((regprog_T *)prog)->engine->regexec_multi(
    (regmmatch_T *)rmp, (win_T *)win, (buf_T *)buf, (linenr_T)lnum, (colnr_T)col,
    (proftime_T *)tm, timed_out);
}

// regprog_T field accessors
int nvim_regprog_get_re_in_use(const void *prog) { return ((const regprog_T *)prog)->re_in_use ? 1 : 0; }
void nvim_regprog_set_re_in_use(void *prog, int v) { ((regprog_T *)prog)->re_in_use = (bool)v; }
unsigned nvim_regprog_get_re_engine(const void *prog) { return ((const regprog_T *)prog)->re_engine; }
unsigned nvim_regprog_get_re_flags(const void *prog) { return ((const regprog_T *)prog)->re_flags; }

// regmatch_T field accessors
void *nvim_regmatch_get_regprog(const void *rmp) { return ((const regmatch_T *)rmp)->regprog; }
void nvim_regmatch_set_regprog(void *rmp, void *prog) { ((regmatch_T *)rmp)->regprog = (regprog_T *)prog; }
int nvim_regmatch_get_rm_ic(const void *rmp) { return ((const regmatch_T *)rmp)->rm_ic ? 1 : 0; }

// regmmatch_T field accessors
void *nvim_regmmatch_get_regprog(const void *rmp) { return ((const regmmatch_T *)rmp)->regprog; }
void nvim_regmmatch_set_regprog(void *rmp, void *prog) { ((regmmatch_T *)rmp)->regprog = (regprog_T *)prog; }

// p_re option
int32_t nvim_regexp_get_p_re(void) { return (int32_t)p_re; }
void nvim_regexp_set_p_re(int32_t v) { p_re = (long)v; }

// nfa_regprog_T pattern accessor
const char *nvim_nfa_prog_get_pattern(const void *prog) {
  return ((const nfa_regprog_T *)prog)->pattern;
}

// reg_do_extmatch setter
void nvim_regexp_set_reg_do_extmatch(int v) { reg_do_extmatch = v; }

// vim_regcomp/vim_regfree calls (for NFA_TOO_EXPENSIVE recompile)
void *nvim_regexp_call_vim_regcomp(const char *pat, int re_flags) {
  return vim_regcomp(pat, re_flags);
}
void nvim_regexp_call_vim_regfree(void *prog) {
  vim_regfree((regprog_T *)prog);
}

// Emit e_recursive error
void nvim_regexp_call_emsg_recursive(void) { emsg(_(e_recursive)); }

// p_verbose option
int64_t nvim_regexp_get_p_verbose(void) { return p_verbose; }

// regmatch_T size for Rust stack allocation
size_t nvim_regexp_get_regmatch_size(void) { return sizeof(regmatch_T); }

// Initialize a regmatch_T buffer with prog and rm_ic
void nvim_regexp_init_regmatch(void *buf, void *prog, int rm_ic) {
  regmatch_T *rmp = (regmatch_T *)buf;
  memset(rmp, 0, sizeof(regmatch_T));
  rmp->regprog = (regprog_T *)prog;
  rmp->rm_ic = (bool)rm_ic;
}
// vim_regcomp accessors

// Forward declaration (defined later in file)
static int regexp_engine;

// regexp_engine variable
int nvim_regexp_get_regexp_engine(void) { return regexp_engine; }
void nvim_regexp_set_regexp_engine(int v) { regexp_engine = v; }

// rex.reg_buf = curbuf
void nvim_regexp_set_rex_reg_buf_curbuf(void) { rex.reg_buf = curbuf; }

// called_emsg counter
int nvim_regexp_get_called_emsg(void) { return called_emsg; }

// Engine vtable compile dispatch
void *nvim_regexp_call_nfa_regcomp(const uint8_t *expr, int re_flags) {
  return nfa_regengine.regcomp(expr, re_flags);
}

void *nvim_regexp_call_bt_regcomp(const uint8_t *expr, int re_flags) {
  return bt_regengine.regcomp(expr, re_flags);
}

// regprog_T field setters
void nvim_regprog_set_re_engine(void *prog, unsigned v) { ((regprog_T *)prog)->re_engine = v; }
void nvim_regprog_set_re_flags(void *prog, unsigned v) { ((regprog_T *)prog)->re_flags = v; }

// E864 error message
void nvim_regexp_call_emsg_e864(void) {
  emsg(_("E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used "));
}
// bt_regcomp accessors

// Allocate bt_regprog_T with flexible array member for program bytes
void *nvim_regexp_alloc_bt_regprog(int64_t regsize_val) {
  bt_regprog_T *r = xmalloc(offsetof(bt_regprog_T, program) + (size_t)regsize_val);
  r->re_in_use = false;
  return r;
}

// bt_regprog_T field setters
void nvim_bt_prog_set_regstart(void *prog, int v) { ((bt_regprog_T *)prog)->regstart = v; }
void nvim_bt_prog_set_reganch(void *prog, int v) { ((bt_regprog_T *)prog)->reganch = v; }
void nvim_bt_prog_set_regmust(void *prog, uint8_t *v) { ((bt_regprog_T *)prog)->regmust = v; }
void nvim_bt_prog_set_regmlen(void *prog, int v) { ((bt_regprog_T *)prog)->regmlen = v; }
void nvim_bt_prog_set_regflags(void *prog, unsigned v) { ((bt_regprog_T *)prog)->regflags = v; }
void nvim_bt_prog_set_reghasz(void *prog, uint8_t v) { ((bt_regprog_T *)prog)->reghasz = v; }
void nvim_bt_prog_set_engine_bt(void *prog) { ((bt_regprog_T *)prog)->engine = &bt_regengine; }

// E339 error + rc_did_emsg
void nvim_regexp_call_emsg_e339(void) {
  emsg(_("E339: Pattern too long"));
  rc_did_emsg = true;
}

// Compile a regular expression into internal code for the NFA matcher.
// Returns the program in allocated space.  Returns NULL for an error.
static regprog_T *nfa_regcomp(uint8_t *expr, int re_flags)
{
  return (regprog_T *)rs_nfa_regcomp(expr, re_flags);
}
// Free a compiled regexp program, returned by nfa_regcomp().
static void nfa_regfree(regprog_T *prog)
{
  if (prog == NULL) {
    return;
  }

  xfree(((nfa_regprog_T *)prog)->match_text);
  xfree(((nfa_regprog_T *)prog)->pattern);
  xfree(prog);
}

// Thin wrappers: NFA engine vtable entry points delegate to Rust.
static int nfa_regexec_nl(regmatch_T *rmp, uint8_t *line, colnr_T col, bool line_lbr)
{
  return rs_nfa_regexec_nl(rmp, line, col, line_lbr);
}

static int nfa_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum, colnr_T col,
                             proftime_T *tm, int *timed_out)
{
  return rs_nfa_regexec_multi(rmp, win, buf, lnum, col, tm, timed_out);
}

static regengine_T bt_regengine = {
  bt_regcomp,
  bt_regfree,
  bt_regexec_nl,
  bt_regexec_multi,
};

static regengine_T nfa_regengine = {
  nfa_regcomp,
  nfa_regfree,
  nfa_regexec_nl,
  nfa_regexec_multi,
};

// Which regexp engine to use? Needed for vim_regcomp().
// Must match with 'regexpengine'.
static int regexp_engine = 0;
// Compile a regular expression into internal code.
// Returns the program in allocated memory.
// Use vim_regfree() to free the memory.
// Returns NULL for an error.
regprog_T *vim_regcomp(const char *expr_arg, int re_flags)
{
  return (regprog_T *)rs_vim_regcomp((const uint8_t *)expr_arg, re_flags);
}

// Note: "*prog" may be freed and changed.
// Return true if there is a match, false if not.
bool vim_regexec_prog(regprog_T **prog, bool ignore_case, const char *line, colnr_T col)
{
  return rs_vim_regexec_prog((void **)prog, ignore_case, (const uint8_t *)line, col) > 0;
}

// Note: "rmp->regprog" may be freed and changed.
// Return true if there is a match, false if not.
bool vim_regexec(regmatch_T *rmp, const char *line, colnr_T col)
{
  return rs_vim_regexec(rmp, (const uint8_t *)line, col) > 0;
}

// Like vim_regexec(), but consider a "\n" in "line" to be a line break.
// Note: "rmp->regprog" may be freed and changed.
// Return true if there is a match, false if not.
bool vim_regexec_nl(regmatch_T *rmp, const char *line, colnr_T col)
{
  return rs_vim_regexec_nl(rmp, (const uint8_t *)line, col) > 0;
}

/// Match a regexp against multiple lines.
/// "rmp->regprog" must be a compiled regexp as returned by vim_regcomp().
/// Note: "rmp->regprog" may be freed and changed, even set to NULL.
/// Uses curbuf for line count and 'iskeyword'.
///
/// @param win        window in which to search or NULL
/// @param buf        buffer in which to search
/// @param lnum       nr of line to start looking for match
/// @param col        column to start looking for match
/// @param tm         timeout limit or NULL
/// @param timed_out  flag is set when timeout limit reached
///
/// @return  zero if there is no match.  Return number of lines contained in the
///          match otherwise.
int vim_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum, colnr_T col,
                      proftime_T *tm, int *timed_out)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_vim_regexec_multi(rmp, win, buf, lnum, col, (void *)tm, timed_out);
}
