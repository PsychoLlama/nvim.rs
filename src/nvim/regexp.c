// Handling of regular expressions: vim_regcomp(), vim_regexec(), vim_regsub()

// By default: do not create debugging logs or files related to regular
// expressions, even when compiling with -DDEBUG.
// Uncomment the second line to get the regexp debugging.
// #undef REGEXP_DEBUG
// #define REGEXP_DEBUG

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

typedef enum {
  RGLF_LINE = 0x01,
  RGLF_LENGTH = 0x02,
  RGLF_SUBMATCH = 0x04,
} reg_getline_flags_T;

enum {
  /// In the NFA engine: how many braces are allowed.
  /// TODO(RE): Use dynamic memory allocation instead of static, like here
  NFA_MAX_BRACES = 20,
};

enum {
  /// In the NFA engine: how many states are allowed.
  NFA_MAX_STATES = 100000,
  NFA_TOO_EXPENSIVE = -1,
};

/// Which regexp engine to use? Needed for vim_regcomp().
/// Must match with 'regexpengine'.
enum {
  AUTOMATIC_ENGINE    = 0,
  BACKTRACKING_ENGINE = 1,
  NFA_ENGINE          = 2,
};

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
#ifdef REGEXP_DEBUG
  uint8_t *expr;
#endif
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

// Values for rs_state in regitem_T.
typedef enum regstate_E {
  RS_NOPEN = 0,         // NOPEN and NCLOSE
  RS_MOPEN,             // MOPEN + [0-9]
  RS_MCLOSE,            // MCLOSE + [0-9]
  RS_ZOPEN,             // ZOPEN + [0-9]
  RS_ZCLOSE,            // ZCLOSE + [0-9]
  RS_BRANCH,            // BRANCH
  RS_BRCPLX_MORE,       // BRACE_COMPLEX and trying one more match
  RS_BRCPLX_LONG,       // BRACE_COMPLEX and trying longest match
  RS_BRCPLX_SHORT,      // BRACE_COMPLEX and trying shortest match
  RS_NOMATCH,           // NOMATCH
  RS_BEHIND1,           // BEHIND / NOBEHIND matching rest
  RS_BEHIND2,           // BEHIND / NOBEHIND matching behind part
  RS_STAR_LONG,         // STAR/PLUS/BRACE_SIMPLE longest match
  RS_STAR_SHORT,  // STAR/PLUS/BRACE_SIMPLE shortest match
} regstate_T;

// When there are alternatives a regstate_T is put on the regstack to remember
// what we are doing.
// Before it may be another type of item, depending on rs_state, to remember
// more things.
typedef struct regitem_S {
  regstate_T rs_state;         // what we are doing, one of RS_ above
  int16_t rs_no;            // submatch nr or BEHIND/NOBEHIND
  uint8_t *rs_scan;         // current node in program
  union {
    save_se_T sesave;
    regsave_T regsave;
  } rs_un;                      // room for saving rex.input
} regitem_T;

// used for BEHIND and NOBEHIND matching
typedef struct regbehind_S {
  regsave_T save_after;
  regsave_T save_behind;
  int save_need_clear_subexpr;
  save_se_T save_start[NSUBEXP];
  save_se_T save_end[NSUBEXP];
} regbehind_T;

// Since the out pointers in the list are always
// uninitialized, we use the pointers themselves
// as storage for the Ptrlists.
typedef union Ptrlist Ptrlist;
union Ptrlist {
  Ptrlist *next;
  nfa_state_T *s;
};

struct Frag {
  nfa_state_T *start;
  Ptrlist *out;
};
typedef struct Frag Frag_T;

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
  nfa_pim_T pim;                // if pim.result != NFA_PIM_UNUSED: postponed
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

#ifdef REGEXP_DEBUG
// show/save debugging data when BT engine is used
# define BT_REGEXP_DUMP
// save the debugging data to a file instead of displaying it
# define BT_REGEXP_LOG
# define BT_REGEXP_DEBUG_LOG
# define BT_REGEXP_DEBUG_LOG_NAME       "bt_regexp_debug.log"
#endif

// Magic characters have a special meaning, they don't match literally.
// Magic characters are negative.  This separates them from literal characters
// (possibly multi-byte).  Only ASCII characters can be Magic.
#define Magic(x)        ((int)(x) - 256)
#define un_Magic(x)     ((x) + 256)
#define is_Magic(x)     ((x) < 0)

typedef void (*fptr_T)(int *, int);

// Rust implementations
extern int rs_no_magic(int x);
extern int rs_toggle_magic(int x);

// Rust implementations for BT optimization (Phase 1)
extern uint8_t *rs_bt_find_regmust(uint8_t *scan, int flags, int *out_len);
extern int rs_bt_get_regstart(uint8_t *scan);

// The first byte of the BT regexp internal "program" is actually this magic
// number; the start node begins in the second byte.  It's used to catch the
// most severe mutilation of the program by the caller.
#define REGMAGIC        0234

// Utility definitions.
#define UCHARAT(p)      ((int)(*(uint8_t *)(p)))

// Used for an error (down from) vim_regcomp(): give the error message, set
// rc_did_emsg and return NULL
#define EMSG_RET_NULL(m) return (emsg(m), rc_did_emsg = true, (void *)NULL)
#define IEMSG_RET_NULL(m) return (iemsg(m), rc_did_emsg = true, (void *)NULL)
#define EMSG_RET_FAIL(m) return (emsg(m), rc_did_emsg = true, FAIL)
#define EMSG2_RET_NULL(m, c) \
  return (semsg((m), (c) ? "" : "\\"), rc_did_emsg = true, (void *)NULL)
#define EMSG3_RET_NULL(m, c, a) \
  return (semsg((m), (c) ? "" : "\\", (a)), rc_did_emsg = true, (void *)NULL)
#define EMSG2_RET_FAIL(m, c) \
  return (semsg((m), (c) ? "" : "\\"), rc_did_emsg = true, FAIL)
#define EMSG_ONE_RET_NULL EMSG2_RET_NULL(_(e_invalid_item_in_str_brackets), reg_magic == MAGIC_ALL)

#define MAX_LIMIT       (32767 << 16)

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

#define NOT_MULTI       0
#define MULTI_ONE       1
#define MULTI_MULT      2

// return values for regmatch()
#define RA_FAIL         1       // something failed, abort
#define RA_CONT         2       // continue in inner loop
#define RA_BREAK        3       // break inner loop
#define RA_MATCH        4       // successful match
#define RA_NOMATCH      5       // didn't match

// Rust implementation
extern int rs_re_multi_type(int c);

static char *reg_prev_sub = NULL;
static size_t reg_prev_sublen = 0;

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

// Rust implementation
extern int rs_backslash_trans(int c);

enum {
  CLASS_ALNUM = 0,
  CLASS_ALPHA,
  CLASS_BLANK,
  CLASS_CNTRL,
  CLASS_DIGIT,
  CLASS_GRAPH,
  CLASS_LOWER,
  CLASS_PRINT,
  CLASS_PUNCT,
  CLASS_SPACE,
  CLASS_UPPER,
  CLASS_XDIGIT,
  CLASS_TAB,
  CLASS_RETURN,
  CLASS_BACKSPACE,
  CLASS_ESCAPE,
  CLASS_IDENT,
  CLASS_KEYWORD,
  CLASS_FNAME,
  CLASS_NONE = 99,
};

// Rust implementation
extern int rs_get_char_class(char **pp);

// Non-static wrapper for other C code
int nvim_get_char_class(char **pp)
{
  return rs_get_char_class(pp);
}

// Specific version of character class functions.
// Using a table to keep this fast.
static int16_t class_tab[256];

#define     RI_DIGIT    0x01
#define     RI_HEX      0x02
#define     RI_OCTAL    0x04
#define     RI_WORD     0x08
#define     RI_HEAD     0x10
#define     RI_ALPHA    0x20
#define     RI_LOWER    0x40
#define     RI_UPPER    0x80
#define     RI_WHITE    0x100

static void init_class_tab(void)
{
  int i;
  static int done = false;

  if (done) {
    return;
  }

  for (i = 0; i < 256; i++) {
    if (i >= '0' && i <= '7') {
      class_tab[i] = RI_DIGIT + RI_HEX + RI_OCTAL + RI_WORD;
    } else if (i >= '8' && i <= '9') {
      class_tab[i] = RI_DIGIT + RI_HEX + RI_WORD;
    } else if (i >= 'a' && i <= 'f') {
      class_tab[i] = RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER;
    } else if (i >= 'g' && i <= 'z') {
      class_tab[i] = RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER;
    } else if (i >= 'A' && i <= 'F') {
      class_tab[i] = RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER;
    } else if (i >= 'G' && i <= 'Z') {
      class_tab[i] = RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER;
    } else if (i == '_') {
      class_tab[i] = RI_WORD + RI_HEAD;
    } else {
      class_tab[i] = 0;
    }
  }
  class_tab[' '] |= RI_WHITE;
  class_tab['\t'] |= RI_WHITE;
  done = true;
}

#define ri_digit(c)    ((c) < 0x100 && (class_tab[c] & RI_DIGIT))
#define ri_hex(c)      ((c) < 0x100 && (class_tab[c] & RI_HEX))
#define ri_octal(c)    ((c) < 0x100 && (class_tab[c] & RI_OCTAL))
#define ri_word(c)     ((c) < 0x100 && (class_tab[c] & RI_WORD))
#define ri_head(c)     ((c) < 0x100 && (class_tab[c] & RI_HEAD))
#define ri_alpha(c)    ((c) < 0x100 && (class_tab[c] & RI_ALPHA))
#define ri_lower(c)    ((c) < 0x100 && (class_tab[c] & RI_LOWER))
#define ri_upper(c)    ((c) < 0x100 && (class_tab[c] & RI_UPPER))
#define ri_white(c)    ((c) < 0x100 && (class_tab[c] & RI_WHITE))

// flags for regflags
#define RF_ICASE    1   // ignore case
#define RF_NOICASE  2   // don't ignore case
#define RF_HASNL    4   // can match a NL
#define RF_ICOMBINE 8   // ignore combining characters
#define RF_LOOKBH   16  // uses "\@<=" or "\@<!"

// Global work variables for vim_regcomp().

static char *regparse;          ///< Input-scan pointer.
static int regnpar;             ///< () count.
static bool wants_nfa;          ///< regex should use NFA engine
static int regnzpar;            ///< \z() count.
static int re_has_z;            ///< \z item detected
static unsigned regflags;       ///< RF_ flags for prog
static int had_eol;             ///< true when EOL found by vim_regcomp()

static magic_T reg_magic;       ///< magicness of the pattern

static int reg_string;          // matching with a string instead of a buffer
                                // line
static int reg_strict;          // "[abc" is illegal

// META contains all characters that may be magic, except '^' and '$'.
// Note: META_flags table is now in Rust (scanner.rs)

static int curchr;              // currently parsed character
// Previous character.  Note: prevchr is sometimes -1 when we are not at the
// start, eg in /[ ^I]^ the pattern was never found even if it existed,
// because ^ was taken to be magic -- webb
static int prevchr;
static int prevprevchr;         // previous-previous character
static int nextchr;             // used for ungetchr()

// arguments for reg()
#define REG_NOPAREN     0       // toplevel reg()
#define REG_PAREN       1       // \(\)
#define REG_ZPAREN      2       // \z(\)
#define REG_NPAREN      3       // \%(\)

typedef struct {
  char *regparse;
  int prevchr_len;
  int curchr;
  int prevchr;
  int prevprevchr;
  int nextchr;
  int at_start;
  int prev_at_start;
  int regnpar;
} parse_state_T;

static regengine_T bt_regengine;
static regengine_T nfa_regengine;

#include "regexp.c.generated.h"

// Rust implementation
extern int rs_re_multiline(const regprog_T *prog);

// Return true if compiled regular expression "prog" can match a line break.
int re_multiline(const regprog_T *prog)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_re_multiline(prog);
}

// Rust implementations for character class parsing
extern int rs_get_equi_class(char **pp);
extern int rs_get_coll_element(char **pp);
extern char *rs_skip_anyof(char *p);
extern int rs_check_char_class(int cls, int c);

// Rust implementations for scanner (Phase 7)
extern void rs_initchr(char *str);
extern int rs_peekchr(void);
extern void rs_skipchr(void);
extern void rs_skipchr_keepstart(void);
extern int rs_getchr(void);
extern void rs_ungetchr(void);

// Rust implementations for string helpers
extern char *rs_cstrchr(const char *s, int c);
extern int rs_cstrncmp(const char *s1, const char *s2, int *n);

// Rust implementations for rex state helpers
extern void rs_reg_breakcheck(void);
extern int rs_reg_iswordc(int c);
extern int rs_reg_prev_class(void);
extern void rs_reg_nextline(void);
extern int rs_regrepeat(uint8_t *p, int64_t maxcount);

// Rust implementations for NFA execution helpers (Phase 12a)
extern int rs_skip_to_start(int c, colnr_T *colp);
extern int rs_find_match_text(colnr_T *startcol, int regstart, uint8_t *match_text);
extern int rs_nfa_did_time_out(void);

// Wrapper for get_equi_class (used by Rust)
int nvim_get_equi_class(char **pp) { return rs_get_equi_class(pp); }

// Wrapper for get_coll_element (used by Rust)
int nvim_get_coll_element(char **pp) { return rs_get_coll_element(pp); }

static int reg_cpo_lit;  // 'cpoptions' contains 'l' flag

static void get_cpo_flags(void)
{
  reg_cpo_lit = vim_strchr(p_cpo, CPO_LITERAL) != NULL;
}

/// Skip over a "[]" range.
/// "p" must point to the character after the '['.
/// The returned pointer is on the matching ']', or the terminating NUL.
static char *skip_anyof(char *p)
{
  return rs_skip_anyof(p);
}

/// Skip past regular expression.
/// Stop at end of "startp" or where "delim" is found ('/', '?', etc).
/// Take care of characters with a backslash in front of it.
/// Skip strings inside [ and ].
char *skip_regexp(char *startp, int delim, int magic)
{
  return skip_regexp_ex(startp, delim, magic, NULL, NULL, NULL);
}

/// Call skip_regexp() and when the delimiter does not match give an error and
/// return NULL.
char *skip_regexp_err(char *startp, int delim, int magic)
{
  char *p = skip_regexp(startp, delim, magic);

  if (*p != delim) {
    semsg(_(e_missing_delimiter_after_search_pattern_str), startp);
    return NULL;
  }
  return p;
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
  magic_T mymagic;
  char *p = startp;
  size_t startplen = 0;

  if (magic) {
    mymagic = MAGIC_ON;
  } else {
    mymagic = MAGIC_OFF;
  }
  get_cpo_flags();

  for (; p[0] != NUL; MB_PTR_ADV(p)) {
    if (p[0] == dirc) {         // found end of regexp
      break;
    }
    if ((p[0] == '[' && mymagic >= MAGIC_ON)
        || (p[0] == '\\' && p[1] == '[' && mymagic <= MAGIC_OFF)) {
      p = skip_anyof(p + 1);
      if (p[0] == NUL) {
        break;
      }
    } else if (p[0] == '\\' && p[1] != NUL) {
      if (dirc == '?' && newp != NULL && p[1] == '?') {
        // change "\?" to "?", make a copy first.
        if (startplen == 0) {
          startplen = strlen(startp);
        }
        if (*newp == NULL) {
          *newp = xstrnsave(startp, startplen);
          p = *newp + (p - startp);
          startp = *newp;
        }
        if (dropped != NULL) {
          (*dropped)++;
        }
        memmove(p, p + 1, startplen - (size_t)((p + 1) - startp) + 1);
      } else {
        p++;            // skip next character
      }
      if (*p == 'v') {
        mymagic = MAGIC_ALL;
      } else if (*p == 'V') {
        mymagic = MAGIC_NONE;
      }
    }
  }
  if (magic_val != NULL) {
    *magic_val = mymagic;
  }
  return p;
}

/// Accessor for Rust FFI: skip regexp and get magic value.
const char *nvim_skip_regexp_ex(const char *startp, int dirc, int magic,
                                char **newp, int *dropped, int *magic_val)
{
  magic_T mval = MAGIC_ON;
  char *result = skip_regexp_ex((char *)startp, dirc, magic, newp, dropped, &mval);
  if (magic_val != NULL) {
    *magic_val = (int)mval;
  }
  return result;
}

// variables used for parsing
static int prevchr_len;    // byte length of previous char
static int at_start;       // True when on the first character
static int prev_at_start;  // True when on the second character

// Start parsing at "str".
static void initchr(char *str)
{
  rs_initchr(str);
}

// Save the current parse state, so that it can be restored and parsing
// starts in the same state again.
static void save_parse_state(parse_state_T *ps)
{
  ps->regparse = regparse;
  ps->prevchr_len = prevchr_len;
  ps->curchr = curchr;
  ps->prevchr = prevchr;
  ps->prevprevchr = prevprevchr;
  ps->nextchr = nextchr;
  ps->at_start = at_start;
  ps->prev_at_start = prev_at_start;
  ps->regnpar = regnpar;
}

// Restore a previously saved parse state.
static void restore_parse_state(parse_state_T *ps)
{
  regparse = ps->regparse;
  prevchr_len = ps->prevchr_len;
  curchr = ps->curchr;
  prevchr = ps->prevchr;
  prevprevchr = ps->prevprevchr;
  nextchr = ps->nextchr;
  at_start = ps->at_start;
  prev_at_start = ps->prev_at_start;
  regnpar = ps->regnpar;
}

// Get the next character without advancing.
static int peekchr(void)
{
  return rs_peekchr();
}

// Eat one lexed character.  Do this in a way that we can undo it.
static void skipchr(void)
{
  rs_skipchr();
}

// Skip a character while keeping the value of prev_at_start for at_start.
// prevchr and prevprevchr are also kept.
static void skipchr_keepstart(void)
{
  rs_skipchr_keepstart();
}

// Get the next character from the pattern. We know about magic and such, so
// therefore we need a lexical analyzer.
static int getchr(void)
{
  return rs_getchr();
}

// put character back.  Works only once!
static void ungetchr(void)
{
  rs_ungetchr();
}

// Rust implementations for number parsing
extern int64_t rs_gethexchrs(int maxinputlen);
extern int64_t rs_getdecchrs(void);
extern int64_t rs_getoctchrs(void);

// Get and return the value of the hex string at the current position.
// Return -1 if there is no valid hex number.
// The position is updated:
//     blahblah\%x20asdf
//         before-^ ^-after
// The parameter controls the maximum number of input characters. This will be
// 2 when reading a \%x20 sequence and 4 when reading a \%u20AC sequence.
static int64_t gethexchrs(int maxinputlen)
{
  return rs_gethexchrs(maxinputlen);
}

// Get and return the value of the decimal string immediately after the
// current position. Return -1 for invalid.  Consumes all digits.
static int64_t getdecchrs(void)
{
  return rs_getdecchrs();
}

// get and return the value of the octal string immediately after the current
// position. Return -1 for invalid, or 0-255 for valid. Smart enough to handle
// numbers > 377 correctly (for example, 400 is treated as 40) and doesn't
// treat 8 or 9 as recognised characters. Position is updated:
//     blahblah\%o210asdf
//         before-^  ^-after
static int64_t getoctchrs(void)
{
  return rs_getoctchrs();
}

// Rust implementation for read_limits (Phase 8)
extern int rs_read_limits(int *minval, int *maxval);

// read_limits - Read two integers to be taken as a minimum and maximum.
// If the first character is '-', then the range is reversed.
// Should end with 'end'.  If minval is missing, zero is default, if maxval is
// missing, a very big number is the default.
static int read_limits(int *minval, int *maxval)
{
  return rs_read_limits(minval, maxval);
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

static void reg_breakcheck(void)
{
  rs_reg_breakcheck();
}

// Return true if character 'c' is included in 'iskeyword' option for
// "reg_buf" buffer.
static bool reg_iswordc(int c)
{
  return rs_reg_iswordc(c) != 0;
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
static void reg_getline_common(linenr_T lnum, reg_getline_flags_T flags, char **line,
                               colnr_T *length)
{
  bool get_line = flags & RGLF_LINE;
  bool get_length = flags & RGLF_LENGTH;
  linenr_T firstlnum;
  linenr_T maxline;

  if (flags & RGLF_SUBMATCH) {
    firstlnum = rsm.sm_firstlnum + lnum;
    maxline = rsm.sm_maxline;
  } else {
    firstlnum = rex.reg_firstlnum + lnum;
    maxline = rex.reg_maxline;
  }

  // when looking behind for a match/no-match lnum is negative. but we
  // can't go before line 1.
  if (firstlnum < 1) {
    if (get_line) {
      *line = NULL;
    }
    if (get_length) {
      *length = 0;
    }

    return;
  }

  if (lnum > maxline) {
    // must have matched the "\n" in the last line.
    if (get_line) {
      *line = "";
    }
    if (get_length) {
      *length = 0;
    }

    return;
  }

  if (get_line) {
    *line = ml_get_buf(rex.reg_buf, firstlnum);
  }
  if (get_length) {
    *length = ml_get_buf_len(rex.reg_buf, firstlnum);
  }
}

/// Get pointer to the line "lnum", which is relative to "reg_firstlnum".
static char *reg_getline(linenr_T lnum)
{
  char *line;
  reg_getline_common(lnum, RGLF_LINE, &line, NULL);
  return line;
}

// Non-static wrapper for Rust to call reg_getline
char *nvim_reg_getline(linenr_T lnum)
{
  return reg_getline(lnum);
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

// Create a new extmatch and mark it as referenced once.
static reg_extmatch_T *make_extmatch(void)
  FUNC_ATTR_NONNULL_RET
{
  reg_extmatch_T *em = xcalloc(1, sizeof(reg_extmatch_T));
  em->refcnt = 1;
  return em;
}

// Add a reference to an extmatch.
reg_extmatch_T *ref_extmatch(reg_extmatch_T *em)
{
  if (em != NULL) {
    em->refcnt++;
  }
  return em;
}

// Remove a reference to an extmatch.  If there are no references left, free
// the info.
void unref_extmatch(reg_extmatch_T *em)
{
  int i;

  if (em != NULL && --em->refcnt <= 0) {
    for (i = 0; i < NSUBEXP; i++) {
      xfree(em->matches[i]);
    }
    xfree(em);
  }
}

// Get class of previous character.
static int reg_prev_class(void)
{
  return rs_reg_prev_class();
}

// Return true if the current rex.input position matches the Visual area.
static bool reg_match_visual(void)
{
  pos_T top, bot;
  linenr_T lnum;
  colnr_T col;
  win_T *wp = rex.reg_win == NULL ? curwin : rex.reg_win;
  int mode;
  colnr_T start, end;
  colnr_T start2, end2;
  colnr_T curswant;

  // Check if the buffer is the current buffer and not using a string.
  if (rex.reg_buf != curbuf || VIsual.lnum == 0 || !REG_MULTI) {
    return false;
  }

  if (VIsual_active) {
    if (lt(VIsual, wp->w_cursor)) {
      top = VIsual;
      bot = wp->w_cursor;
    } else {
      top = wp->w_cursor;
      bot = VIsual;
    }
    mode = VIsual_mode;
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
    mode = curbuf->b_visual.vi_mode;
    curswant = curbuf->b_visual.vi_curswant;
  }
  lnum = rex.lnum + rex.reg_firstlnum;
  if (lnum < top.lnum || lnum > bot.lnum) {
    return false;
  }

  col = (colnr_T)(rex.input - rex.line);
  if (mode == 'v') {
    if ((lnum == top.lnum && col < top.col)
        || (lnum == bot.lnum && col >= bot.col + (*p_sel != 'e'))) {
      return false;
    }
  } else if (mode == Ctrl_V) {
    getvvcol(wp, &top, &start, NULL, &end);
    getvvcol(wp, &bot, &start2, NULL, &end2);
    if (start2 < start) {
      start = start2;
    }
    if (end2 > end) {
      end = end2;
    }
    if (top.col == MAXCOL || bot.col == MAXCOL || curswant == MAXCOL) {
      end = MAXCOL;
    }

    // getvvcol() flushes rex.line, need to get it again
    rex.line = (uint8_t *)reg_getline(rex.lnum);
    rex.input = rex.line + col;

    colnr_T cols = win_linetabsize(wp, rex.reg_firstlnum + rex.lnum, (char *)rex.line, col);
    if (cols < start || cols > end - (*p_sel == 'e')) {
      return false;
    }
  }
  return true;
}

// Check the regexp program for its magic number.
// Return true if it's wrong.
static int prog_magic_wrong(void)
{
  regprog_T *prog;

  prog = REG_MULTI ? rex.reg_mmatch->regprog : rex.reg_match->regprog;
  if (prog->engine == &nfa_regengine) {
    // For NFA matcher we don't check the magic
    return false;
  }

  if (UCHARAT(((bt_regprog_T *)prog)->program) != REGMAGIC) {
    emsg(_(e_re_corr));
    return true;
  }
  return false;
}

// Cleanup the subexpressions, if this wasn't done yet.
// This construction is used to clear the subexpressions only when they are
// used (to increase speed).
static void cleanup_subexpr(void)
{
  if (!rex.need_clear_subexpr) {
    return;
  }

  if (REG_MULTI) {
    // Use 0xff to set lnum to -1
    memset(rex.reg_startpos, 0xff, sizeof(lpos_T) * NSUBEXP);
    memset(rex.reg_endpos, 0xff, sizeof(lpos_T) * NSUBEXP);
  } else {
    memset(rex.reg_startp, 0, sizeof(char *) * NSUBEXP);
    memset(rex.reg_endp, 0, sizeof(char *) * NSUBEXP);
  }
  rex.need_clear_subexpr = false;
}

static void cleanup_zsubexpr(void)
{
  if (!rex.need_clear_zsubexpr) {
    return;
  }

  if (REG_MULTI) {
    // Use 0xff to set lnum to -1
    memset(reg_startzpos, 0xff, sizeof(lpos_T) * NSUBEXP);
    memset(reg_endzpos, 0xff, sizeof(lpos_T) * NSUBEXP);
  } else {
    memset(reg_startzp, 0, sizeof(char *) * NSUBEXP);
    memset(reg_endzp, 0, sizeof(char *) * NSUBEXP);
  }
  rex.need_clear_zsubexpr = false;
}

// Exposed for Rust - wraps static cleanup_subexpr()
void nvim_cleanup_subexpr(void) { cleanup_subexpr(); }

// Exposed for Rust - wraps static cleanup_zsubexpr()
void nvim_cleanup_zsubexpr(void) { cleanup_zsubexpr(); }

// Advance rex.lnum, rex.line and rex.input to the next line.
static void reg_nextline(void)
{
  rs_reg_nextline();
}

// Check whether a backreference matches.
// Returns RA_FAIL, RA_NOMATCH or RA_MATCH.
// If "bytelen" is not NULL, it is set to the byte length of the match in the
// last line.
// Optional: ignore case if rex.reg_ic is set.
static int match_with_backref(linenr_T start_lnum, colnr_T start_col, linenr_T end_lnum,
                              colnr_T end_col, int *bytelen)
{
  linenr_T clnum = start_lnum;
  colnr_T ccol = start_col;
  int len;
  char *p;

  if (bytelen != NULL) {
    *bytelen = 0;
  }
  while (true) {
    // Since getting one line may invalidate the other, need to make copy.
    // Slow!
    if (rex.line != reg_tofree) {
      len = (int)strlen((char *)rex.line);
      if (reg_tofree == NULL || len >= (int)reg_tofreelen) {
        len += 50;              // get some extra
        xfree(reg_tofree);
        reg_tofree = xmalloc((size_t)len);
        reg_tofreelen = (unsigned)len;
      }
      STRCPY(reg_tofree, rex.line);
      rex.input = reg_tofree + (rex.input - rex.line);
      rex.line = reg_tofree;
    }

    // Get the line to compare with.
    p = reg_getline(clnum);
    assert(p);

    if (clnum == end_lnum) {
      len = end_col - ccol;
    } else {
      len = reg_getline_len(clnum) - ccol;
    }

    if ((!rex.reg_ic && cstrncmp(p + ccol, (char *)rex.input, &len) != 0)
        || (rex.reg_ic && mb_strnicmp(p + ccol, (char *)rex.input, (size_t)len) != 0)) {
      return RA_NOMATCH;  // doesn't match
    }
    if (bytelen != NULL) {
      *bytelen += len;
    }
    if (clnum == end_lnum) {
      break;  // match and at end!
    }
    if (rex.lnum >= rex.reg_maxline) {
      return RA_NOMATCH;  // text too short
    }

    // Advance to next line.
    reg_nextline();
    if (bytelen != NULL) {
      *bytelen = 0;
    }
    clnum++;
    ccol = 0;
    if (got_int) {
      return RA_FAIL;
    }
  }

  // found a match!  Note that rex.line may now point to a copy of the line,
  // that should not matter.
  return RA_MATCH;
}

/// Used in a place where no * or \+ can follow.
static bool re_mult_next(char *what)
{
  if (rs_re_multi_type(peekchr()) == MULTI_MULT) {
    semsg(_("E888: (NFA regexp) cannot repeat %s"), what);
    rc_did_emsg = true;
    return false;
  }
  return true;
}

// Rust implementation for mb_decompose
extern void rs_mb_decompose(int c, int *c1, int *c2, int *c3);

static void mb_decompose(int c, int *c1, int *c2, int *c3)
{
  rs_mb_decompose(c, c1, c2, c3);
}

/// Compare two strings, ignore case if rex.reg_ic set.
/// Return 0 if strings match, non-zero otherwise.
/// Correct the length "*n" when composing characters are ignored
/// or when both utf codepoints are considered equal because of
/// case-folding but have different length (e.g. 's' and 'ſ')
static int cstrncmp(char *s1, char *s2, int *n)
{
  return rs_cstrncmp(s1, s2, n);
}

/// Wrapper around strchr which accounts for case-insensitive searches and
/// non-ASCII characters.
///
/// This function is used a lot for simple searches, keep it fast!
///
/// @param  s  string to search
/// @param  c  character to find in @a s
///
/// @return  NULL if no match, otherwise pointer to the position in @a s
static inline char *cstrchr(const char *const s, const int c)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
  FUNC_ATTR_ALWAYS_INLINE
{
  return rs_cstrchr(s, c);
}

////////////////////////////////////////////////////////////////
//                    regsub stuff                            //
////////////////////////////////////////////////////////////////

static void do_upper(int *d, int c)
{
  *d = mb_toupper(c);
}

static void do_lower(int *d, int c)
{
  *d = mb_tolower(c);
}

// Rust implementation
extern char *rs_regtilde(char *source, int magic, int preview);

/// regtilde(): Replace tildes in the pattern by the old pattern.
///
/// Short explanation of the tilde: It stands for the previous replacement
/// pattern.  If that previous pattern also contains a ~ we should go back a
/// step further...  But we insert the previous pattern into the current one
/// and remember that.
/// This still does not handle the case where "magic" changes.  So require the
/// user to keep his hands off of "magic".
///
/// The tildes are parsed once before the first call to vim_regsub().
char *regtilde(char *source, int magic, bool preview)
{
  return rs_regtilde(source, magic, preview ? 1 : 0);
}

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

/// vim_regsub() - perform substitutions after a vim_regexec() or
/// vim_regexec_multi() match.
///
/// If "flags" has REGSUB_COPY really copy into "dest[destlen]".
/// Otherwise nothing is copied, only compute the length of the result.
///
/// If "flags" has REGSUB_MAGIC then behave like 'magic' is set.
///
/// If "flags" has REGSUB_BACKSLASH a backslash will be removed later, need to
/// double them to keep them, and insert a backslash before a CR to avoid it
/// being replaced with a line break later.
///
/// Note: The matched text must not change between the call of
/// vim_regexec()/vim_regexec_multi() and vim_regsub()!  It would make the back
/// references invalid!
///
/// Returns the size of the replacement, including terminating NUL.
int vim_regsub(regmatch_T *rmp, char *source, typval_T *expr, char *dest, int destlen, int flags)
{
  regexec_T rex_save;
  bool rex_in_use_save = rex_in_use;

  if (rex_in_use) {
    // Being called recursively, save the state.
    rex_save = rex;
  }
  rex_in_use = true;

  rex.reg_match = rmp;
  rex.reg_mmatch = NULL;
  rex.reg_maxline = 0;
  rex.reg_buf = curbuf;
  rex.reg_line_lbr = true;
  int result = vim_regsub_both(source, expr, dest, destlen, flags);

  rex_in_use = rex_in_use_save;
  if (rex_in_use) {
    rex = rex_save;
  }

  return result;
}

int vim_regsub_multi(regmmatch_T *rmp, linenr_T lnum, char *source, char *dest, int destlen,
                     int flags)
{
  regexec_T rex_save;
  bool rex_in_use_save = rex_in_use;

  if (rex_in_use) {
    // Being called recursively, save the state.
    rex_save = rex;
  }
  rex_in_use = true;

  rex.reg_match = NULL;
  rex.reg_mmatch = rmp;
  rex.reg_buf = curbuf;  // always works on the current buffer!
  rex.reg_firstlnum = lnum;
  rex.reg_maxline = curbuf->b_ml.ml_line_count - lnum;
  rex.reg_line_lbr = false;
  int result = vim_regsub_both(source, NULL, dest, destlen, flags);

  rex_in_use = rex_in_use_save;
  if (rex_in_use) {
    rex = rex_save;
  }

  return result;
}

// When nesting more than a couple levels it's probably a mistake.
#define MAX_REGSUB_NESTING 4
static char *eval_result[MAX_REGSUB_NESTING] = { NULL, NULL, NULL, NULL };

#if defined(EXITFREE)
void free_resub_eval_result(void)
{
  for (int i = 0; i < MAX_REGSUB_NESTING; i++) {
    XFREE_CLEAR(eval_result[i]);
  }
}
#endif

static int vim_regsub_both(char *source, typval_T *expr, char *dest, int destlen, int flags)
{
  char *src;
  char *dst;
  char *s;
  int c;
  int cc;
  int no = -1;
  fptr_T func_all = (fptr_T)NULL;
  fptr_T func_one = (fptr_T)NULL;
  linenr_T clnum = 0;           // init for GCC
  int len = 0;                  // init for GCC
  static int nesting = 0;
  bool copy = flags & REGSUB_COPY;

  // Be paranoid...
  if ((source == NULL && expr == NULL) || dest == NULL) {
    emsg(_(e_null));
    return 0;
  }
  if (prog_magic_wrong()) {
    return 0;
  }
  if (nesting == MAX_REGSUB_NESTING) {
    emsg(_(e_substitute_nesting_too_deep));
    return 0;
  }
  int nested = nesting;
  src = source;
  dst = dest;

  // When the substitute part starts with "\=" evaluate it as an expression.
  if (expr != NULL || (source[0] == '\\' && source[1] == '=')) {
    // To make sure that the length doesn't change between checking the
    // length and copying the string, and to speed up things, the
    // resulting string is saved from the call with
    // "flags & REGSUB_COPY" == 0 to the call with
    // "flags & REGSUB_COPY" != 0.
    if (copy) {
      if (eval_result[nested] != NULL) {
        size_t eval_len = strlen(eval_result[nested]);
        if (eval_len < (size_t)destlen) {
          STRCPY(dest, eval_result[nested]);
          dst += eval_len;
          XFREE_CLEAR(eval_result[nested]);
        }
      }
    } else {
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
      nesting++;

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
        if (expr->v_type == VAR_FUNC) {
          s = expr->vval.v_string;
          call_func(s, -1, &rettv, 1, argv, &funcexe);
        } else if (expr->v_type == VAR_PARTIAL) {
          partial_T *partial = expr->vval.v_partial;

          s = partial_name(partial);
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
      nesting--;

      if (eval_result[nested] != NULL) {
        int had_backslash = false;

        for (s = eval_result[nested]; *s != NUL; MB_PTR_ADV(s)) {
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
          s = vim_strsave_escaped(eval_result[nested], "\\");
          xfree(eval_result[nested]);
          eval_result[nested] = s;
        }

        dst += strlen(eval_result[nested]);
      }

      can_f_submatch = prev_can_f_submatch;
      if (can_f_submatch) {
        rsm = rsm_save;
      }
    }
  } else {
    while ((c = (uint8_t)(*src++)) != NUL) {
      if (c == '&' && (flags & REGSUB_MAGIC)) {
        no = 0;
      } else if (c == '\\' && *src != NUL) {
        if (*src == '&' && !(flags & REGSUB_MAGIC)) {
          src++;
          no = 0;
        } else if ('0' <= *src && *src <= '9') {
          no = *src++ - '0';
        } else if (vim_strchr("uUlLeE", (uint8_t)(*src))) {
          switch (*src++) {
          case 'u':
            func_one = do_upper;
            continue;
          case 'U':
            func_all = do_upper;
            continue;
          case 'l':
            func_one = do_lower;
            continue;
          case 'L':
            func_all = do_lower;
            continue;
          case 'e':
          case 'E':
            func_one = func_all = (fptr_T)NULL;
            continue;
          }
        }
      }
      if (no < 0) {           // Ordinary character.
        if (c == K_SPECIAL && src[0] != NUL && src[1] != NUL) {
          // Copy a special key as-is.
          if (copy) {
            if (dst + 3 > dest + destlen) {
              iemsg("vim_regsub_both(): not enough space");
              return 0;
            }
            *dst++ = (char)c;
            *dst++ = *src++;
            *dst++ = *src++;
          } else {
            dst += 3;
            src += 2;
          }
          continue;
        }

        if (c == '\\' && *src != NUL) {
          // Check for abbreviations -- webb
          switch (*src) {
          case 'r':
            c = CAR;        ++src;  break;
          case 'n':
            c = NL;         ++src;  break;
          case 't':
            c = TAB;        ++src;  break;
          // Oh no!  \e already has meaning in subst pat :-(
          // case 'e':   c = ESC;        ++src;  break;
          case 'b':
            c = Ctrl_H;     ++src;  break;

          // If "backslash" is true the backslash will be removed
          // later.  Used to insert a literal CR.
          default:
            if (flags & REGSUB_BACKSLASH) {
              if (copy) {
                if (dst + 1 > dest + destlen) {
                  iemsg("vim_regsub_both(): not enough space");
                  return 0;
                }
                *dst = '\\';
              }
              dst++;
            }
            c = (uint8_t)(*src++);
          }
        } else {
          c = utf_ptr2char(src - 1);
        }

        // Write to buffer, if copy is set.
        if (func_one != NULL) {
          func_one(&cc, c);
          func_one = NULL;
        } else if (func_all != NULL) {
          func_all(&cc, c);
        } else {
          // just copy
          cc = c;
        }

        int totlen = utfc_ptr2len(src - 1);
        int charlen = utf_char2len(cc);

        if (copy) {
          if (dst + charlen > dest + destlen) {
            iemsg("vim_regsub_both(): not enough space");
            return 0;
          }
          utf_char2bytes(cc, dst);
        }
        dst += charlen - 1;
        int clen = utf_ptr2len(src - 1);

        // If the character length is shorter than "totlen", there
        // are composing characters; copy them as-is.
        if (clen < totlen) {
          if (copy) {
            if (dst + totlen - clen > dest + destlen) {
              iemsg("vim_regsub_both(): not enough space");
              return 0;
            }
            memmove(dst + 1, src - 1 + clen, (size_t)(totlen - clen));
          }
          dst += totlen - clen;
        }
        src += totlen - 1;
        dst++;
      } else {
        if (REG_MULTI) {
          clnum = rex.reg_mmatch->startpos[no].lnum;
          if (clnum < 0 || rex.reg_mmatch->endpos[no].lnum < 0) {
            s = NULL;
          } else {
            s = reg_getline(clnum) + rex.reg_mmatch->startpos[no].col;
            if (rex.reg_mmatch->endpos[no].lnum == clnum) {
              len = rex.reg_mmatch->endpos[no].col
                    - rex.reg_mmatch->startpos[no].col;
            } else {
              len = reg_getline_len(clnum) - rex.reg_mmatch->startpos[no].col;
            }
          }
        } else {
          s = rex.reg_match->startp[no];
          if (rex.reg_match->endp[no] == NULL) {
            s = NULL;
          } else {
            len = (int)(rex.reg_match->endp[no] - s);
          }
        }
        if (s != NULL) {
          while (true) {
            if (len == 0) {
              if (REG_MULTI) {
                if (rex.reg_mmatch->endpos[no].lnum == clnum) {
                  break;
                }
                if (copy) {
                  if (dst + 1 > dest + destlen) {
                    iemsg("vim_regsub_both(): not enough space");
                    return 0;
                  }
                  *dst = CAR;
                }
                dst++;
                s = reg_getline(++clnum);
                if (rex.reg_mmatch->endpos[no].lnum == clnum) {
                  len = rex.reg_mmatch->endpos[no].col;
                } else {
                  len = reg_getline_len(clnum);
                }
              } else {
                break;
              }
            } else if (*s == NUL) {  // we hit NUL.
              if (copy) {
                iemsg(_(e_re_damg));
              }
              goto exit;
            } else {
              if ((flags & REGSUB_BACKSLASH) && (*s == CAR || *s == '\\')) {
                // Insert a backslash in front of a CR, otherwise
                // it will be replaced by a line break.
                // Number of backslashes will be halved later,
                // double them here.
                if (copy) {
                  if (dst + 2 > dest + destlen) {
                    iemsg("vim_regsub_both(): not enough space");
                    return 0;
                  }
                  dst[0] = '\\';
                  dst[1] = *s;
                }
                dst += 2;
              } else {
                c = utf_ptr2char(s);

                if (func_one != (fptr_T)NULL) {
                  func_one(&cc, c);
                  func_one = NULL;
                } else if (func_all != (fptr_T)NULL) {
                  func_all(&cc, c);
                } else {  // just copy
                  cc = c;
                }

                {
                  int l;
                  int charlen;

                  // Copy composing characters separately, one
                  // at a time.
                  l = utf_ptr2len(s) - 1;

                  s += l;
                  len -= l;
                  charlen = utf_char2len(cc);
                  if (copy) {
                    if (dst + charlen > dest + destlen) {
                      iemsg("vim_regsub_both(): not enough space");
                      return 0;
                    }
                    utf_char2bytes(cc, dst);
                  }
                  dst += charlen - 1;
                }
                dst++;
              }

              s++;
              len--;
            }
          }
        }
        no = -1;
      }
    }
  }
  if (copy) {
    *dst = NUL;
  }

exit:
  return (int)((dst - dest) + 1);
}

static char *reg_getline_submatch(linenr_T lnum)
{
  char *line;
  reg_getline_common(lnum, RGLF_LINE | RGLF_SUBMATCH, &line, NULL);
  return line;
}

static colnr_T reg_getline_submatch_len(linenr_T lnum)
{
  colnr_T length;
  reg_getline_common(lnum, RGLF_LENGTH | RGLF_SUBMATCH, NULL, &length);
  return length;
}

/// Used for the submatch() function: get the string from the n'th submatch in
/// allocated memory.
///
/// @return  NULL when not in a ":s" command and for a non-existing submatch.
char *reg_submatch(int no)
{
  char *retval = NULL;
  char *s;
  int round;
  linenr_T lnum;

  if (!can_f_submatch || no < 0) {
    return NULL;
  }

  if (rsm.sm_match == NULL) {
    ssize_t len;

    // First round: compute the length and allocate memory.
    // Second round: copy the text.
    for (round = 1; round <= 2; round++) {
      lnum = rsm.sm_mmatch->startpos[no].lnum;
      if (lnum < 0 || rsm.sm_mmatch->endpos[no].lnum < 0) {
        return NULL;
      }

      s = reg_getline_submatch(lnum);
      if (s == NULL) {  // anti-crash check, cannot happen?
        break;
      }
      s += rsm.sm_mmatch->startpos[no].col;
      if (rsm.sm_mmatch->endpos[no].lnum == lnum) {
        // Within one line: take form start to end col.
        len = rsm.sm_mmatch->endpos[no].col - rsm.sm_mmatch->startpos[no].col;
        if (round == 2) {
          xmemcpyz(retval, s, (size_t)len);
        }
        len++;
      } else {
        // Multiple lines: take start line from start col, middle
        // lines completely and end line up to end col.
        len = reg_getline_submatch_len(lnum) - rsm.sm_mmatch->startpos[no].col;
        if (round == 2) {
          STRCPY(retval, s);
          retval[len] = '\n';
        }
        len++;
        lnum++;
        while (lnum < rsm.sm_mmatch->endpos[no].lnum) {
          s = reg_getline_submatch(lnum);
          if (round == 2) {
            STRCPY(retval + len, s);
          }
          len += reg_getline_submatch_len(lnum);
          if (round == 2) {
            retval[len] = '\n';
          }
          len++;
          lnum++;
        }
        if (round == 2) {
          strncpy(retval + len,  // NOLINT(runtime/printf)
                  reg_getline_submatch(lnum),
                  (size_t)rsm.sm_mmatch->endpos[no].col);
        }
        len += rsm.sm_mmatch->endpos[no].col;
        if (round == 2) {
          retval[len] = NUL;
        }
        len++;
      }

      if (retval == NULL) {
        retval = xmalloc((size_t)len);
      }
    }
  } else {
    s = rsm.sm_match->startp[no];
    if (s == NULL || rsm.sm_match->endp[no] == NULL) {
      retval = NULL;
    } else {
      retval = xstrnsave(s, (size_t)(rsm.sm_match->endp[no] - s));
    }
  }

  return retval;
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

// regexp_bt.c {{{1

// Backtracking regular expression implementation.
//
// NOTICE:
//
// This is NOT the original regular expression code as written by Henry
// Spencer.  This code has been modified specifically for use with the VIM
// editor, and should not be used separately from Vim.  If you want a good
// regular expression library, get the original code.  The copyright notice
// that follows is from the original.
//
// END NOTICE
//
//      Copyright (c) 1986 by University of Toronto.
//      Written by Henry Spencer.  Not derived from licensed software.
//
//      Permission is granted to anyone to use this software for any
//      purpose on any computer system, and to redistribute it freely,
//      subject to the following restrictions:
//
//      1. The author is not responsible for the consequences of use of
//              this software, no matter how awful, even if they arise
//              from defects in it.
//
//      2. The origin of this software must not be misrepresented, either
//              by explicit claim or by omission.
//
//      3. Altered versions must be plainly marked as such, and must not
//              be misrepresented as being the original software.
//
// Beware that some of this code is subtly aware of the way operator
// precedence is structured in regular expressions.  Serious changes in
// regular-expression syntax might require a total rethink.
//
// Changes have been made by Tony Andrews, Olaf 'Rhialto' Seibert, Robert
// Webb, Ciaran McCreesh and Bram Moolenaar.
// Named character class support added by Walter Briscoe (1998 Jul 01)

// The "internal use only" fields in regexp_defs.h are present to pass info from
// compile to execute that permits the execute phase to run lots faster on
// simple cases.  They are:
//
// regstart     char that must begin a match; NUL if none obvious; Can be a
//              multi-byte character.
// reganch      is the match anchored (at beginning-of-line only)?
// regmust      string (pointer into program) that match must include, or NULL
// regmlen      length of regmust string
// regflags     RF_ values or'ed together
//
// Regstart and reganch permit very fast decisions on suitable starting points
// for a match, cutting down the work a lot.  Regmust permits fast rejection
// of lines that cannot possibly match.  The regmust tests are costly enough
// that vim_regcomp() supplies a regmust only if the r.e. contains something
// potentially expensive (at present, the only such thing detected is * or +
// at the start of the r.e., which can involve a lot of backup).  Regmlen is
// supplied because the test in vim_regexec() needs it and vim_regcomp() is
// computing it anyway.

// Structure for regexp "program".  This is essentially a linear encoding
// of a nondeterministic finite-state machine (aka syntax charts or
// "railroad normal form" in parsing technology).  Each node is an opcode
// plus a "next" pointer, possibly plus an operand.  "Next" pointers of
// all nodes except BRANCH and BRACES_COMPLEX implement concatenation; a "next"
// pointer with a BRANCH on both ends of it is connecting two alternatives.
// (Here we have one of the subtle syntax dependencies: an individual BRANCH
// (as opposed to a collection of them) is never concatenated with anything
// because of operator precedence).  The "next" pointer of a BRACES_COMPLEX
// node points to the node after the stuff to be repeated.
// The operand of some types of node is a literal string; for others, it is a
// node leading into a sub-FSM.  In particular, the operand of a BRANCH node
// is the first node of the branch.
// (NB this is *not* a tree structure: the tail of the branch connects to the
// thing following the set of BRANCHes.)
//
// pattern      is coded like:
//
//                        +-----------------+
//                        |                 V
// <aa>\|<bb>   BRANCH <aa> BRANCH <bb> --> END
//                   |      ^    |          ^
//                   +------+    +----------+
//
//
//                     +------------------+
//                     V                  |
// <aa>*        BRANCH BRANCH <aa> --> BACK BRANCH --> NOTHING --> END
//                   |      |               ^                      ^
//                   |      +---------------+                      |
//                   +---------------------------------------------+
//
//
//                     +----------------------+
//                     V                      |
// <aa>\+       BRANCH <aa> --> BRANCH --> BACK  BRANCH --> NOTHING --> END
//                   |               |           ^                      ^
//                   |               +-----------+                      |
//                   +--------------------------------------------------+
//
//
//                                      +-------------------------+
//                                      V                         |
// <aa>\{}      BRANCH BRACE_LIMITS --> BRACE_COMPLEX <aa> --> BACK  END
//                   |                              |                ^
//                   |                              +----------------+
//                   +-----------------------------------------------+
//
//
// <aa>\@!<bb>  BRANCH NOMATCH <aa> --> END  <bb> --> END
//                   |       |                ^       ^
//                   |       +----------------+       |
//                   +--------------------------------+
//
//                                                    +---------+
//                                                    |         V
// \z[abc]      BRANCH BRANCH  a  BRANCH  b  BRANCH  c  BRANCH  NOTHING --> END
//                   |      |          |          |     ^                   ^
//                   |      |          |          +-----+                   |
//                   |      |          +----------------+                   |
//                   |      +---------------------------+                   |
//                   +------------------------------------------------------+
//
// They all start with a BRANCH for "\|" alternatives, even when there is only
// one alternative.

// The opcodes are:

// definition   number             opnd?    meaning
#define END             0       //      End of program or NOMATCH operand.
#define BOL             1       //      Match "" at beginning of line.
#define EOL             2       //      Match "" at end of line.
#define BRANCH          3       // node Match this alternative, or the
                                //      next...
#define BACK            4       //      Match "", "next" ptr points backward.
#define EXACTLY         5       // str  Match this string.
#define NOTHING         6       //      Match empty string.
#define STAR            7       // node Match this (simple) thing 0 or more
                                //      times.
#define PLUS            8       // node Match this (simple) thing 1 or more
                                //      times.
#define MATCH           9       // node match the operand zero-width
#define NOMATCH         10      // node check for no match with operand
#define BEHIND          11      // node look behind for a match with operand
#define NOBEHIND        12      // node look behind for no match with operand
#define SUBPAT          13      // node match the operand here
#define BRACE_SIMPLE    14      // node Match this (simple) thing between m and
                                //      n times (\{m,n\}).
#define BOW             15      //      Match "" after [^a-zA-Z0-9_]
#define EOW             16      //      Match "" at    [^a-zA-Z0-9_]
#define BRACE_LIMITS    17      // nr nr  define the min & max for BRACE_SIMPLE
                                //      and BRACE_COMPLEX.
#define NEWL            18      //      Match line-break
#define BHPOS           19      //      End position for BEHIND or NOBEHIND

// character classes: 20-48 normal, 50-78 include a line-break
#define ADD_NL          30
#define FIRST_NL        ANY + ADD_NL
#define ANY             20      //      Match any one character.
#define ANYOF           21      // str  Match any character in this string.
#define ANYBUT          22      // str  Match any character not in this
                                //      string.
#define IDENT           23      //      Match identifier char
#define SIDENT          24      //      Match identifier char but no digit
#define KWORD           25      //      Match keyword char
#define SKWORD          26      //      Match word char but no digit
#define FNAME           27      //      Match file name char
#define SFNAME          28      //      Match file name char but no digit
#define PRINT           29      //      Match printable char
#define SPRINT          30      //      Match printable char but no digit
#define WHITE           31      //      Match whitespace char
#define NWHITE          32      //      Match non-whitespace char
#define DIGIT           33      //      Match digit char
#define NDIGIT          34      //      Match non-digit char
#define HEX             35      //      Match hex char
#define NHEX            36      //      Match non-hex char
#define OCTAL           37      //      Match octal char
#define NOCTAL          38      //      Match non-octal char
#define WORD            39      //      Match word char
#define NWORD           40      //      Match non-word char
#define HEAD            41      //      Match head char
#define NHEAD           42      //      Match non-head char
#define ALPHA           43      //      Match alpha char
#define NALPHA          44      //      Match non-alpha char
#define LOWER           45      //      Match lowercase char
#define NLOWER          46      //      Match non-lowercase char
#define UPPER           47      //      Match uppercase char
#define NUPPER          48      //      Match non-uppercase char
#define LAST_NL         NUPPER + ADD_NL
#define WITH_NL(op)     ((op) >= FIRST_NL && (op) <= LAST_NL)

#define MOPEN           80   // -89 Mark this point in input as start of
                             //     \( … \) subexpr.  MOPEN + 0 marks start of
                             //     match.
#define MCLOSE          90   // -99 Analogous to MOPEN.  MCLOSE + 0 marks
                             //     end of match.
#define BACKREF         100  // -109 node Match same string again \1-\9.

#define ZOPEN          110  // -119 Mark this point in input as start of
                            //  \z( … \) subexpr.
#define ZCLOSE         120  // -129 Analogous to ZOPEN.
#define ZREF           130  // -139 node Match external submatch \z1-\z9

#define BRACE_COMPLEX   140  // -149 node Match nodes between m & n times

#define NOPEN           150     // Mark this point in input as start of
                                // \%( subexpr.
#define NCLOSE          151     // Analogous to NOPEN.

#define MULTIBYTECODE   200     // mbc  Match one multi-byte character
#define RE_BOF          201     //      Match "" at beginning of file.
#define RE_EOF          202     //      Match "" at end of file.
#define CURSOR          203     //      Match location of cursor.

#define RE_LNUM         204     // nr cmp  Match line number
#define RE_COL          205     // nr cmp  Match column number
#define RE_VCOL         206     // nr cmp  Match virtual column number

#define RE_MARK         207     // mark cmp  Match mark position
#define RE_VISUAL       208     //      Match Visual area
#define RE_COMPOSING    209     // any composing characters

// Flags to be passed up and down.
#define HASWIDTH        0x1     // Known never to match null string.
#define SIMPLE          0x2     // Simple enough to be STAR/PLUS operand.
#define SPSTART         0x4     // Starts with * or +.
#define HASNL           0x8     // Contains some \n.
#define HASLOOKBH       0x10    // Contains "\@<=" or "\@<!".
#define WORST           0       // Worst case.

static int prevchr_len;         ///< byte length of previous char
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
static int classcodes[] = {
  ANY, IDENT, SIDENT, KWORD, SKWORD,
  FNAME, SFNAME, PRINT, SPRINT,
  WHITE, NWHITE, DIGIT, NDIGIT,
  HEX, NHEX, OCTAL, NOCTAL,
  WORD, NWORD, HEAD, NHEAD,
  ALPHA, NALPHA, LOWER, NLOWER,
  UPPER, NUPPER
};

// When regcode is set to this value, code is not emitted and size is computed
// instead.
#define JUST_CALC_SIZE  ((uint8_t *)-1)

// used for STAR, PLUS and BRACE_SIMPLE matching
typedef struct regstar_S {
  int nextb;            // next byte
  int nextb_ic;         // next byte reverse case
  int64_t count;
  int64_t minval;
  int64_t maxval;
} regstar_T;

// used to store input position when a BACK was encountered, so that we now if
// we made any progress since the last time.
typedef struct backpos_S {
  uint8_t *bp_scan;         // "scan" where BACK was encountered
  regsave_T bp_pos;           // last input position
} backpos_T;

// "regstack" and "backpos" are used by regmatch().  They are kept over calls
// to avoid invoking malloc() and free() often.
// "regstack" is a stack with regitem_T items, sometimes preceded by regstar_T
// or regbehind_T.
// "backpos_T" is a table with backpos_T for BACK
static garray_T regstack = GA_EMPTY_INIT_VALUE;
static garray_T backpos = GA_EMPTY_INIT_VALUE;

static regsave_T behind_pos;

// Both for regstack and backpos tables we use the following strategy of
// allocation (to reduce malloc/free calls):
// - Initial size is fairly small.
// - When needed, the tables are grown bigger (8 times at first, double after
//   that).
// - After executing the match we free the memory only if the array has grown.
//   Thus the memory is kept allocated when it's at the initial size.
// This makes it fast while not keeping a lot of memory allocated.
// A three times speed increase was observed when using many simple patterns.
#define REGSTACK_INITIAL        2048
#define BACKPOS_INITIAL         64

// Opcode notes:
//
// BRANCH       The set of branches constituting a single choice are hooked
//              together with their "next" pointers, since precedence prevents
//              anything being concatenated to any individual branch.  The
//              "next" pointer of the last BRANCH in a choice points to the
//              thing following the whole choice.  This is also where the
//              final "next" pointer of each individual branch points; each
//              branch starts with the operand node of a BRANCH node.
//
// BACK         Normal "next" pointers all implicitly point forward; BACK
//              exists to make loop structures possible.
//
// STAR,PLUS    '=', and complex '*' and '+', are implemented as circular
//              BRANCH structures using BACK.  Simple cases (one character
//              per match) are implemented with STAR and PLUS for speed
//              and to minimize recursive plunges.
//
// BRACE_LIMITS This is always followed by a BRACE_SIMPLE or BRACE_COMPLEX
//              node, and defines the min and max limits to be used for that
//              node.
//
// MOPEN,MCLOSE ...are numbered at compile time.
// ZOPEN,ZCLOSE ...ditto
///
//
//
// A node is one char of opcode followed by two chars of "next" pointer.
// "Next" pointers are stored as two 8-bit bytes, high order first.  The
// value is a positive offset from the opcode of the node containing it.
// An operand, if any, simply follows the node.  (Note that much of the
// code generation knows about this implicit relationship.)
//
// Using two bytes for the "next" pointer is vast overkill for most things,
// but allows patterns to get big without disasters.
#define OP(p)           ((int)(*(p)))
#define NEXT(p)         (((*((p) + 1) & 0377) << 8) + (*((p) + 2) & 0377))
#define OPERAND(p)      ((p) + 3)
// Obtain an operand that was stored as four bytes, MSB first.
#define OPERAND_MIN(p)  (((int64_t)(p)[3] << 24) + ((int64_t)(p)[4] << 16) \
                         + ((int64_t)(p)[5] << 8) + (int64_t)(p)[6])
// Obtain a second operand stored as four bytes.
#define OPERAND_MAX(p)  OPERAND_MIN((p) + 4)
// Obtain a second single-byte operand stored after a four bytes operand.
#define OPERAND_CMP(p)  (p)[7]

static uint8_t *reg(int paren, int *flagp);

#ifdef BT_REGEXP_DUMP
static void     regdump(uint8_t *, bt_regprog_T *);
#endif

#ifdef REGEXP_DEBUG
static uint8_t *regprop(uint8_t *);

static int regnarrate = 0;
#endif

// Setup to parse the regexp.  Used once to get the length and once to do it.
static void regcomp_start(uint8_t *expr, int re_flags)                        // see vim_regcomp()
{
  initchr((char *)expr);
  if (re_flags & RE_MAGIC) {
    reg_magic = MAGIC_ON;
  } else {
    reg_magic = MAGIC_OFF;
  }
  reg_string = (re_flags & RE_STRING);
  reg_strict = (re_flags & RE_STRICT);
  get_cpo_flags();

  num_complex_braces = 0;
  regnpar = 1;
  CLEAR_FIELD(had_endbrace);
  regnzpar = 1;
  re_has_z = 0;
  regsize = 0L;
  reg_toolong = false;
  regflags = 0;
  had_eol = false;
}

// Rust implementation for use_multibytecode
extern int rs_use_multibytecode(int c);

// Return true if MULTIBYTECODE should be used instead of EXACTLY for
// character "c".
static bool use_multibytecode(int c)
{
  return rs_use_multibytecode(c) != 0;
}

// Emit (if appropriate) a byte of code
static void regc(int b)
{
  if (regcode == JUST_CALC_SIZE) {
    regsize++;
  } else {
    *regcode++ = (uint8_t)b;
  }
}

// Emit (if appropriate) a multi-byte character of code
static void regmbc(int c)
{
  if (regcode == JUST_CALC_SIZE) {
    regsize += utf_char2len(c);
  } else {
    regcode += utf_char2bytes(c, (char *)regcode);
  }
}

// Produce the bytes for equivalence class "c".
// Currently only handles latin1, latin9 and utf-8.
// NOTE: When changing this function, also change nfa_emit_equi_class()
static void reg_equi_class(int c)
{
  {
    switch (c) {
    // Do not use '\300' style, it results in a negative number.
    case 'A':
    case 0xc0:
    case 0xc1:
    case 0xc2:
    case 0xc3:
    case 0xc4:
    case 0xc5:
    case 0x100:
    case 0x102:
    case 0x104:
    case 0x1cd:
    case 0x1de:
    case 0x1e0:
    case 0x1fa:
    case 0x202:
    case 0x226:
    case 0x23a:
    case 0x1e00:
    case 0x1ea0:
    case 0x1ea2:
    case 0x1ea4:
    case 0x1ea6:
    case 0x1ea8:
    case 0x1eaa:
    case 0x1eac:
    case 0x1eae:
    case 0x1eb0:
    case 0x1eb2:
    case 0x1eb4:
    case 0x1eb6:
      regmbc('A'); regmbc(0xc0); regmbc(0xc1); regmbc(0xc2);
      regmbc(0xc3); regmbc(0xc4); regmbc(0xc5);
      regmbc(0x100); regmbc(0x102); regmbc(0x104);
      regmbc(0x1cd); regmbc(0x1de); regmbc(0x1e0);
      regmbc(0x1fa); regmbc(0x202); regmbc(0x226);
      regmbc(0x23a); regmbc(0x1e00); regmbc(0x1ea0);
      regmbc(0x1ea2); regmbc(0x1ea4); regmbc(0x1ea6);
      regmbc(0x1ea8); regmbc(0x1eaa); regmbc(0x1eac);
      regmbc(0x1eae); regmbc(0x1eb0); regmbc(0x1eb2);
      regmbc(0x1eb4); regmbc(0x1eb6);
      return;
    case 'B':
    case 0x181:
    case 0x243:
    case 0x1e02:
    case 0x1e04:
    case 0x1e06:
      regmbc('B');
      regmbc(0x181); regmbc(0x243); regmbc(0x1e02);
      regmbc(0x1e04); regmbc(0x1e06);
      return;
    case 'C':
    case 0xc7:
    case 0x106:
    case 0x108:
    case 0x10a:
    case 0x10c:
    case 0x187:
    case 0x23b:
    case 0x1e08:
    case 0xa792:
      regmbc('C'); regmbc(0xc7);
      regmbc(0x106); regmbc(0x108); regmbc(0x10a);
      regmbc(0x10c); regmbc(0x187); regmbc(0x23b);
      regmbc(0x1e08); regmbc(0xa792);
      return;
    case 'D':
    case 0x10e:
    case 0x110:
    case 0x18a:
    case 0x1e0a:
    case 0x1e0c:
    case 0x1e0e:
    case 0x1e10:
    case 0x1e12:
      regmbc('D'); regmbc(0x10e); regmbc(0x110);
      regmbc(0x18a); regmbc(0x1e0a); regmbc(0x1e0c);
      regmbc(0x1e0e); regmbc(0x1e10); regmbc(0x1e12);
      return;
    case 'E':
    case 0xc8:
    case 0xc9:
    case 0xca:
    case 0xcb:
    case 0x112:
    case 0x114:
    case 0x116:
    case 0x118:
    case 0x11a:
    case 0x204:
    case 0x206:
    case 0x228:
    case 0x246:
    case 0x1e14:
    case 0x1e16:
    case 0x1e18:
    case 0x1e1a:
    case 0x1e1c:
    case 0x1eb8:
    case 0x1eba:
    case 0x1ebc:
    case 0x1ebe:
    case 0x1ec0:
    case 0x1ec2:
    case 0x1ec4:
    case 0x1ec6:
      regmbc('E'); regmbc(0xc8); regmbc(0xc9);
      regmbc(0xca); regmbc(0xcb); regmbc(0x112);
      regmbc(0x114); regmbc(0x116); regmbc(0x118);
      regmbc(0x11a); regmbc(0x204); regmbc(0x206);
      regmbc(0x228); regmbc(0x246); regmbc(0x1e14);
      regmbc(0x1e16); regmbc(0x1e18); regmbc(0x1e1a);
      regmbc(0x1e1c); regmbc(0x1eb8); regmbc(0x1eba);
      regmbc(0x1ebc); regmbc(0x1ebe); regmbc(0x1ec0);
      regmbc(0x1ec2); regmbc(0x1ec4); regmbc(0x1ec6);
      return;
    case 'F':
    case 0x191:
    case 0x1e1e:
    case 0xa798:
      regmbc('F'); regmbc(0x191); regmbc(0x1e1e);
      regmbc(0xa798);
      return;
    case 'G':
    case 0x11c:
    case 0x11e:
    case 0x120:
    case 0x122:
    case 0x193:
    case 0x1e4:
    case 0x1e6:
    case 0x1f4:
    case 0x1e20:
    case 0xa7a0:
      regmbc('G'); regmbc(0x11c); regmbc(0x11e);
      regmbc(0x120); regmbc(0x122); regmbc(0x193);
      regmbc(0x1e4); regmbc(0x1e6); regmbc(0x1f4);
      regmbc(0x1e20); regmbc(0xa7a0);
      return;
    case 'H':
    case 0x124:
    case 0x126:
    case 0x21e:
    case 0x1e22:
    case 0x1e24:
    case 0x1e26:
    case 0x1e28:
    case 0x1e2a:
    case 0x2c67:
      regmbc('H'); regmbc(0x124); regmbc(0x126);
      regmbc(0x21e); regmbc(0x1e22); regmbc(0x1e24);
      regmbc(0x1e26); regmbc(0x1e28); regmbc(0x1e2a);
      regmbc(0x2c67);
      return;
    case 'I':
    case 0xcc:
    case 0xcd:
    case 0xce:
    case 0xcf:
    case 0x128:
    case 0x12a:
    case 0x12c:
    case 0x12e:
    case 0x130:
    case 0x197:
    case 0x1cf:
    case 0x208:
    case 0x20a:
    case 0x1e2c:
    case 0x1e2e:
    case 0x1ec8:
    case 0x1eca:
      regmbc('I'); regmbc(0xcc); regmbc(0xcd);
      regmbc(0xce); regmbc(0xcf); regmbc(0x128);
      regmbc(0x12a); regmbc(0x12c); regmbc(0x12e);
      regmbc(0x130); regmbc(0x197); regmbc(0x1cf);
      regmbc(0x208); regmbc(0x20a); regmbc(0x1e2c);
      regmbc(0x1e2e); regmbc(0x1ec8); regmbc(0x1eca);
      return;
    case 'J':
    case 0x134:
    case 0x248:
      regmbc('J'); regmbc(0x134); regmbc(0x248);
      return;
    case 'K':
    case 0x136:
    case 0x198:
    case 0x1e8:
    case 0x1e30:
    case 0x1e32:
    case 0x1e34:
    case 0x2c69:
    case 0xa740:
      regmbc('K'); regmbc(0x136); regmbc(0x198);
      regmbc(0x1e8); regmbc(0x1e30); regmbc(0x1e32);
      regmbc(0x1e34); regmbc(0x2c69); regmbc(0xa740);
      return;
    case 'L':
    case 0x139:
    case 0x13b:
    case 0x13d:
    case 0x13f:
    case 0x141:
    case 0x23d:
    case 0x1e36:
    case 0x1e38:
    case 0x1e3a:
    case 0x1e3c:
    case 0x2c60:
      regmbc('L'); regmbc(0x139); regmbc(0x13b);
      regmbc(0x13d); regmbc(0x13f); regmbc(0x141);
      regmbc(0x23d); regmbc(0x1e36); regmbc(0x1e38);
      regmbc(0x1e3a); regmbc(0x1e3c); regmbc(0x2c60);
      return;
    case 'M':
    case 0x1e3e:
    case 0x1e40:
    case 0x1e42:
      regmbc('M'); regmbc(0x1e3e); regmbc(0x1e40);
      regmbc(0x1e42);
      return;
    case 'N':
    case 0xd1:
    case 0x143:
    case 0x145:
    case 0x147:
    case 0x1f8:
    case 0x1e44:
    case 0x1e46:
    case 0x1e48:
    case 0x1e4a:
    case 0xa7a4:
      regmbc('N'); regmbc(0xd1);
      regmbc(0x143); regmbc(0x145); regmbc(0x147);
      regmbc(0x1f8); regmbc(0x1e44); regmbc(0x1e46);
      regmbc(0x1e48); regmbc(0x1e4a); regmbc(0xa7a4);
      return;
    case 'O':
    case 0xd2:
    case 0xd3:
    case 0xd4:
    case 0xd5:
    case 0xd6:
    case 0xd8:
    case 0x14c:
    case 0x14e:
    case 0x150:
    case 0x19f:
    case 0x1a0:
    case 0x1d1:
    case 0x1ea:
    case 0x1ec:
    case 0x1fe:
    case 0x20c:
    case 0x20e:
    case 0x22a:
    case 0x22c:
    case 0x22e:
    case 0x230:
    case 0x1e4c:
    case 0x1e4e:
    case 0x1e50:
    case 0x1e52:
    case 0x1ecc:
    case 0x1ece:
    case 0x1ed0:
    case 0x1ed2:
    case 0x1ed4:
    case 0x1ed6:
    case 0x1ed8:
    case 0x1eda:
    case 0x1edc:
    case 0x1ede:
    case 0x1ee0:
    case 0x1ee2:
      regmbc('O'); regmbc(0xd2); regmbc(0xd3); regmbc(0xd4);
      regmbc(0xd5); regmbc(0xd6); regmbc(0xd8);
      regmbc(0x14c); regmbc(0x14e); regmbc(0x150);
      regmbc(0x19f); regmbc(0x1a0); regmbc(0x1d1);
      regmbc(0x1ea); regmbc(0x1ec); regmbc(0x1fe);
      regmbc(0x20c); regmbc(0x20e); regmbc(0x22a);
      regmbc(0x22c); regmbc(0x22e); regmbc(0x230);
      regmbc(0x1e4c); regmbc(0x1e4e); regmbc(0x1e50);
      regmbc(0x1e52); regmbc(0x1ecc); regmbc(0x1ece);
      regmbc(0x1ed0); regmbc(0x1ed2); regmbc(0x1ed4);
      regmbc(0x1ed6); regmbc(0x1ed8); regmbc(0x1eda);
      regmbc(0x1edc); regmbc(0x1ede); regmbc(0x1ee0);
      regmbc(0x1ee2);
      return;
    case 'P':
    case 0x1a4:
    case 0x1e54:
    case 0x1e56:
    case 0x2c63:
      regmbc('P'); regmbc(0x1a4); regmbc(0x1e54);
      regmbc(0x1e56); regmbc(0x2c63);
      return;
    case 'Q':
    case 0x24a:
      regmbc('Q'); regmbc(0x24a);
      return;
    case 'R':
    case 0x154:
    case 0x156:
    case 0x158:
    case 0x210:
    case 0x212:
    case 0x24c:
    case 0x1e58:
    case 0x1e5a:
    case 0x1e5c:
    case 0x1e5e:
    case 0x2c64:
    case 0xa7a6:
      regmbc('R'); regmbc(0x154); regmbc(0x156);
      regmbc(0x210); regmbc(0x212); regmbc(0x158);
      regmbc(0x24c); regmbc(0x1e58); regmbc(0x1e5a);
      regmbc(0x1e5c); regmbc(0x1e5e); regmbc(0x2c64);
      regmbc(0xa7a6);
      return;
    case 'S':
    case 0x15a:
    case 0x15c:
    case 0x15e:
    case 0x160:
    case 0x218:
    case 0x1e60:
    case 0x1e62:
    case 0x1e64:
    case 0x1e66:
    case 0x1e68:
    case 0x2c7e:
    case 0xa7a8:
      regmbc('S'); regmbc(0x15a); regmbc(0x15c);
      regmbc(0x15e); regmbc(0x160); regmbc(0x218);
      regmbc(0x1e60); regmbc(0x1e62); regmbc(0x1e64);
      regmbc(0x1e66); regmbc(0x1e68); regmbc(0x2c7e);
      regmbc(0xa7a8);
      return;
    case 'T':
    case 0x162:
    case 0x164:
    case 0x166:
    case 0x1ac:
    case 0x1ae:
    case 0x21a:
    case 0x23e:
    case 0x1e6a:
    case 0x1e6c:
    case 0x1e6e:
    case 0x1e70:
      regmbc('T'); regmbc(0x162); regmbc(0x164);
      regmbc(0x166); regmbc(0x1ac); regmbc(0x23e);
      regmbc(0x1ae); regmbc(0x21a); regmbc(0x1e6a);
      regmbc(0x1e6c); regmbc(0x1e6e); regmbc(0x1e70);
      return;
    case 'U':
    case 0xd9:
    case 0xda:
    case 0xdb:
    case 0xdc:
    case 0x168:
    case 0x16a:
    case 0x16c:
    case 0x16e:
    case 0x170:
    case 0x172:
    case 0x1af:
    case 0x1d3:
    case 0x1d5:
    case 0x1d7:
    case 0x1d9:
    case 0x1db:
    case 0x214:
    case 0x216:
    case 0x244:
    case 0x1e72:
    case 0x1e74:
    case 0x1e76:
    case 0x1e78:
    case 0x1e7a:
    case 0x1ee4:
    case 0x1ee6:
    case 0x1ee8:
    case 0x1eea:
    case 0x1eec:
    case 0x1eee:
    case 0x1ef0:
      regmbc('U'); regmbc(0xd9); regmbc(0xda);
      regmbc(0xdb); regmbc(0xdc); regmbc(0x168);
      regmbc(0x16a); regmbc(0x16c); regmbc(0x16e);
      regmbc(0x170); regmbc(0x172); regmbc(0x1af);
      regmbc(0x1d3); regmbc(0x1d5); regmbc(0x1d7);
      regmbc(0x1d9); regmbc(0x1db); regmbc(0x214);
      regmbc(0x216); regmbc(0x244); regmbc(0x1e72);
      regmbc(0x1e74); regmbc(0x1e76); regmbc(0x1e78);
      regmbc(0x1e7a); regmbc(0x1ee4); regmbc(0x1ee6);
      regmbc(0x1ee8); regmbc(0x1eea); regmbc(0x1eec);
      regmbc(0x1eee); regmbc(0x1ef0);
      return;
    case 'V':
    case 0x1b2:
    case 0x1e7c:
    case 0x1e7e:
      regmbc('V'); regmbc(0x1b2); regmbc(0x1e7c);
      regmbc(0x1e7e);
      return;
    case 'W':
    case 0x174:
    case 0x1e80:
    case 0x1e82:
    case 0x1e84:
    case 0x1e86:
    case 0x1e88:
      regmbc('W'); regmbc(0x174); regmbc(0x1e80);
      regmbc(0x1e82); regmbc(0x1e84); regmbc(0x1e86);
      regmbc(0x1e88);
      return;
    case 'X':
    case 0x1e8a:
    case 0x1e8c:
      regmbc('X'); regmbc(0x1e8a); regmbc(0x1e8c);
      return;
    case 'Y':
    case 0xdd:
    case 0x176:
    case 0x178:
    case 0x1b3:
    case 0x232:
    case 0x24e:
    case 0x1e8e:
    case 0x1ef2:
    case 0x1ef6:
    case 0x1ef4:
    case 0x1ef8:
      regmbc('Y'); regmbc(0xdd); regmbc(0x176);
      regmbc(0x178); regmbc(0x1b3); regmbc(0x232);
      regmbc(0x24e); regmbc(0x1e8e); regmbc(0x1ef2);
      regmbc(0x1ef4); regmbc(0x1ef6); regmbc(0x1ef8);
      return;
    case 'Z':
    case 0x179:
    case 0x17b:
    case 0x17d:
    case 0x1b5:
    case 0x1e90:
    case 0x1e92:
    case 0x1e94:
    case 0x2c6b:
      regmbc('Z'); regmbc(0x179); regmbc(0x17b);
      regmbc(0x17d); regmbc(0x1b5); regmbc(0x1e90);
      regmbc(0x1e92); regmbc(0x1e94); regmbc(0x2c6b);
      return;
    case 'a':
    case 0xe0:
    case 0xe1:
    case 0xe2:
    case 0xe3:
    case 0xe4:
    case 0xe5:
    case 0x101:
    case 0x103:
    case 0x105:
    case 0x1ce:
    case 0x1df:
    case 0x1e1:
    case 0x1fb:
    case 0x201:
    case 0x203:
    case 0x227:
    case 0x1d8f:
    case 0x1e01:
    case 0x1e9a:
    case 0x1ea1:
    case 0x1ea3:
    case 0x1ea5:
    case 0x1ea7:
    case 0x1ea9:
    case 0x1eab:
    case 0x1ead:
    case 0x1eaf:
    case 0x1eb1:
    case 0x1eb3:
    case 0x1eb5:
    case 0x1eb7:
    case 0x2c65:
      regmbc('a'); regmbc(0xe0); regmbc(0xe1);
      regmbc(0xe2); regmbc(0xe3); regmbc(0xe4);
      regmbc(0xe5); regmbc(0x101); regmbc(0x103);
      regmbc(0x105); regmbc(0x1ce); regmbc(0x1df);
      regmbc(0x1e1); regmbc(0x1fb); regmbc(0x201);
      regmbc(0x203); regmbc(0x227); regmbc(0x1d8f);
      regmbc(0x1e01); regmbc(0x1e9a); regmbc(0x1ea1);
      regmbc(0x1ea3); regmbc(0x1ea5); regmbc(0x1ea7);
      regmbc(0x1ea9); regmbc(0x1eab); regmbc(0x1ead);
      regmbc(0x1eaf); regmbc(0x1eb1); regmbc(0x1eb3);
      regmbc(0x1eb5); regmbc(0x1eb7); regmbc(0x2c65);
      return;
    case 'b':
    case 0x180:
    case 0x253:
    case 0x1d6c:
    case 0x1d80:
    case 0x1e03:
    case 0x1e05:
    case 0x1e07:
      regmbc('b');
      regmbc(0x180); regmbc(0x253); regmbc(0x1d6c);
      regmbc(0x1d80); regmbc(0x1e03); regmbc(0x1e05);
      regmbc(0x1e07);
      return;
    case 'c':
    case 0xe7:
    case 0x107:
    case 0x109:
    case 0x10b:
    case 0x10d:
    case 0x188:
    case 0x23c:
    case 0x1e09:
    case 0xa793:
    case 0xa794:
      regmbc('c'); regmbc(0xe7); regmbc(0x107);
      regmbc(0x109); regmbc(0x10b); regmbc(0x10d);
      regmbc(0x188); regmbc(0x23c); regmbc(0x1e09);
      regmbc(0xa793); regmbc(0xa794);
      return;
    case 'd':
    case 0x10f:
    case 0x111:
    case 0x257:
    case 0x1d6d:
    case 0x1d81:
    case 0x1d91:
    case 0x1e0b:
    case 0x1e0d:
    case 0x1e0f:
    case 0x1e11:
    case 0x1e13:
      regmbc('d'); regmbc(0x10f); regmbc(0x111);
      regmbc(0x257); regmbc(0x1d6d); regmbc(0x1d81);
      regmbc(0x1d91); regmbc(0x1e0b); regmbc(0x1e0d);
      regmbc(0x1e0f); regmbc(0x1e11); regmbc(0x1e13);
      return;
    case 'e':
    case 0xe8:
    case 0xe9:
    case 0xea:
    case 0xeb:
    case 0x113:
    case 0x115:
    case 0x117:
    case 0x119:
    case 0x11b:
    case 0x205:
    case 0x207:
    case 0x229:
    case 0x247:
    case 0x1d92:
    case 0x1e15:
    case 0x1e17:
    case 0x1e19:
    case 0x1e1b:
    case 0x1eb9:
    case 0x1ebb:
    case 0x1e1d:
    case 0x1ebd:
    case 0x1ebf:
    case 0x1ec1:
    case 0x1ec3:
    case 0x1ec5:
    case 0x1ec7:
      regmbc('e'); regmbc(0xe8); regmbc(0xe9);
      regmbc(0xea); regmbc(0xeb); regmbc(0x113);
      regmbc(0x115); regmbc(0x117); regmbc(0x119);
      regmbc(0x11b); regmbc(0x205); regmbc(0x207);
      regmbc(0x229); regmbc(0x247); regmbc(0x1d92);
      regmbc(0x1e15); regmbc(0x1e17); regmbc(0x1e19);
      regmbc(0x1e1b); regmbc(0x1e1d); regmbc(0x1eb9);
      regmbc(0x1ebb); regmbc(0x1ebd); regmbc(0x1ebf);
      regmbc(0x1ec1); regmbc(0x1ec3); regmbc(0x1ec5);
      regmbc(0x1ec7);
      return;
    case 'f':
    case 0x192:
    case 0x1d6e:
    case 0x1d82:
    case 0x1e1f:
    case 0xa799:
      regmbc('f'); regmbc(0x192); regmbc(0x1d6e);
      regmbc(0x1d82); regmbc(0x1e1f); regmbc(0xa799);
      return;
    case 'g':
    case 0x11d:
    case 0x11f:
    case 0x121:
    case 0x123:
    case 0x1e5:
    case 0x1e7:
    case 0x260:
    case 0x1f5:
    case 0x1d83:
    case 0x1e21:
    case 0xa7a1:
      regmbc('g'); regmbc(0x11d); regmbc(0x11f);
      regmbc(0x121); regmbc(0x123); regmbc(0x1e5);
      regmbc(0x1e7); regmbc(0x1f5); regmbc(0x260);
      regmbc(0x1d83); regmbc(0x1e21); regmbc(0xa7a1);
      return;
    case 'h':
    case 0x125:
    case 0x127:
    case 0x21f:
    case 0x1e23:
    case 0x1e25:
    case 0x1e27:
    case 0x1e29:
    case 0x1e2b:
    case 0x1e96:
    case 0x2c68:
    case 0xa795:
      regmbc('h'); regmbc(0x125); regmbc(0x127);
      regmbc(0x21f); regmbc(0x1e23); regmbc(0x1e25);
      regmbc(0x1e27); regmbc(0x1e29); regmbc(0x1e2b);
      regmbc(0x1e96); regmbc(0x2c68); regmbc(0xa795);
      return;
    case 'i':
    case 0xec:
    case 0xed:
    case 0xee:
    case 0xef:
    case 0x129:
    case 0x12b:
    case 0x12d:
    case 0x12f:
    case 0x1d0:
    case 0x209:
    case 0x20b:
    case 0x268:
    case 0x1d96:
    case 0x1e2d:
    case 0x1e2f:
    case 0x1ec9:
    case 0x1ecb:
      regmbc('i'); regmbc(0xec); regmbc(0xed);
      regmbc(0xee); regmbc(0xef); regmbc(0x129);
      regmbc(0x12b); regmbc(0x12d); regmbc(0x12f);
      regmbc(0x1d0); regmbc(0x209); regmbc(0x20b);
      regmbc(0x268); regmbc(0x1d96); regmbc(0x1e2d);
      regmbc(0x1e2f); regmbc(0x1ec9); regmbc(0x1ecb);
      return;
    case 'j':
    case 0x135:
    case 0x1f0:
    case 0x249:
      regmbc('j'); regmbc(0x135); regmbc(0x1f0);
      regmbc(0x249);
      return;
    case 'k':
    case 0x137:
    case 0x199:
    case 0x1e9:
    case 0x1d84:
    case 0x1e31:
    case 0x1e33:
    case 0x1e35:
    case 0x2c6a:
    case 0xa741:
      regmbc('k'); regmbc(0x137); regmbc(0x199);
      regmbc(0x1e9); regmbc(0x1d84); regmbc(0x1e31);
      regmbc(0x1e33); regmbc(0x1e35); regmbc(0x2c6a);
      regmbc(0xa741);
      return;
    case 'l':
    case 0x13a:
    case 0x13c:
    case 0x13e:
    case 0x140:
    case 0x142:
    case 0x19a:
    case 0x1e37:
    case 0x1e39:
    case 0x1e3b:
    case 0x1e3d:
    case 0x2c61:
      regmbc('l'); regmbc(0x13a); regmbc(0x13c);
      regmbc(0x13e); regmbc(0x140); regmbc(0x142);
      regmbc(0x19a); regmbc(0x1e37); regmbc(0x1e39);
      regmbc(0x1e3b); regmbc(0x1e3d); regmbc(0x2c61);
      return;
    case 'm':
    case 0x1d6f:
    case 0x1e3f:
    case 0x1e41:
    case 0x1e43:
      regmbc('m'); regmbc(0x1d6f); regmbc(0x1e3f);
      regmbc(0x1e41); regmbc(0x1e43);
      return;
    case 'n':
    case 0xf1:
    case 0x144:
    case 0x146:
    case 0x148:
    case 0x149:
    case 0x1f9:
    case 0x1d70:
    case 0x1d87:
    case 0x1e45:
    case 0x1e47:
    case 0x1e49:
    case 0x1e4b:
    case 0xa7a5:
      regmbc('n'); regmbc(0xf1); regmbc(0x144);
      regmbc(0x146); regmbc(0x148); regmbc(0x149);
      regmbc(0x1f9); regmbc(0x1d70); regmbc(0x1d87);
      regmbc(0x1e45); regmbc(0x1e47); regmbc(0x1e49);
      regmbc(0x1e4b); regmbc(0xa7a5);
      return;
    case 'o':
    case 0xf2:
    case 0xf3:
    case 0xf4:
    case 0xf5:
    case 0xf6:
    case 0xf8:
    case 0x14d:
    case 0x14f:
    case 0x151:
    case 0x1a1:
    case 0x1d2:
    case 0x1eb:
    case 0x1ed:
    case 0x1ff:
    case 0x20d:
    case 0x20f:
    case 0x22b:
    case 0x22d:
    case 0x22f:
    case 0x231:
    case 0x275:
    case 0x1e4d:
    case 0x1e4f:
    case 0x1e51:
    case 0x1e53:
    case 0x1ecd:
    case 0x1ecf:
    case 0x1ed1:
    case 0x1ed3:
    case 0x1ed5:
    case 0x1ed7:
    case 0x1ed9:
    case 0x1edb:
    case 0x1edd:
    case 0x1edf:
    case 0x1ee1:
    case 0x1ee3:
      regmbc('o'); regmbc(0xf2); regmbc(0xf3);
      regmbc(0xf4); regmbc(0xf5); regmbc(0xf6);
      regmbc(0xf8); regmbc(0x14d); regmbc(0x14f);
      regmbc(0x151); regmbc(0x1a1); regmbc(0x1d2);
      regmbc(0x1eb); regmbc(0x1ed); regmbc(0x1ff);
      regmbc(0x20d); regmbc(0x20f); regmbc(0x22b);
      regmbc(0x22d); regmbc(0x22f); regmbc(0x231);
      regmbc(0x275); regmbc(0x1e4d); regmbc(0x1e4f);
      regmbc(0x1e51); regmbc(0x1e53); regmbc(0x1ecd);
      regmbc(0x1ecf); regmbc(0x1ed1); regmbc(0x1ed3);
      regmbc(0x1ed5); regmbc(0x1ed7); regmbc(0x1ed9);
      regmbc(0x1edb); regmbc(0x1edd); regmbc(0x1edf);
      regmbc(0x1ee1); regmbc(0x1ee3);
      return;
    case 'p':
    case 0x1a5:
    case 0x1d71:
    case 0x1d88:
    case 0x1d7d:
    case 0x1e55:
    case 0x1e57:
      regmbc('p'); regmbc(0x1a5); regmbc(0x1d71);
      regmbc(0x1d7d); regmbc(0x1d88); regmbc(0x1e55);
      regmbc(0x1e57);
      return;
    case 'q':
    case 0x24b:
    case 0x2a0:
      regmbc('q'); regmbc(0x24b); regmbc(0x2a0);
      return;
    case 'r':
    case 0x155:
    case 0x157:
    case 0x159:
    case 0x211:
    case 0x213:
    case 0x24d:
    case 0x27d:
    case 0x1d72:
    case 0x1d73:
    case 0x1d89:
    case 0x1e59:
    case 0x1e5b:
    case 0x1e5d:
    case 0x1e5f:
    case 0xa7a7:
      regmbc('r'); regmbc(0x155); regmbc(0x157);
      regmbc(0x159); regmbc(0x211); regmbc(0x213);
      regmbc(0x24d); regmbc(0x1d72); regmbc(0x1d73);
      regmbc(0x1d89); regmbc(0x1e59); regmbc(0x27d);
      regmbc(0x1e5b); regmbc(0x1e5d); regmbc(0x1e5f);
      regmbc(0xa7a7);
      return;
    case 's':
    case 0x15b:
    case 0x15d:
    case 0x15f:
    case 0x161:
    case 0x1e61:
    case 0x219:
    case 0x23f:
    case 0x1d74:
    case 0x1d8a:
    case 0x1e63:
    case 0x1e65:
    case 0x1e67:
    case 0x1e69:
    case 0xa7a9:
      regmbc('s'); regmbc(0x15b); regmbc(0x15d);
      regmbc(0x15f); regmbc(0x161); regmbc(0x23f);
      regmbc(0x219); regmbc(0x1d74); regmbc(0x1d8a);
      regmbc(0x1e61); regmbc(0x1e63); regmbc(0x1e65);
      regmbc(0x1e67); regmbc(0x1e69); regmbc(0xa7a9);
      return;
    case 't':
    case 0x163:
    case 0x165:
    case 0x167:
    case 0x1ab:
    case 0x1ad:
    case 0x21b:
    case 0x288:
    case 0x1d75:
    case 0x1e6b:
    case 0x1e6d:
    case 0x1e6f:
    case 0x1e71:
    case 0x1e97:
    case 0x2c66:
      regmbc('t'); regmbc(0x163); regmbc(0x165);
      regmbc(0x167); regmbc(0x1ab); regmbc(0x21b);
      regmbc(0x1ad); regmbc(0x288); regmbc(0x1d75);
      regmbc(0x1e6b); regmbc(0x1e6d); regmbc(0x1e6f);
      regmbc(0x1e71); regmbc(0x1e97); regmbc(0x2c66);
      return;
    case 'u':
    case 0xf9:
    case 0xfa:
    case 0xfb:
    case 0xfc:
    case 0x169:
    case 0x16b:
    case 0x16d:
    case 0x16f:
    case 0x171:
    case 0x173:
    case 0x1b0:
    case 0x1d4:
    case 0x1d6:
    case 0x1d8:
    case 0x1da:
    case 0x1dc:
    case 0x215:
    case 0x217:
    case 0x289:
    case 0x1e73:
    case 0x1d7e:
    case 0x1d99:
    case 0x1e75:
    case 0x1e77:
    case 0x1e79:
    case 0x1e7b:
    case 0x1ee5:
    case 0x1ee7:
    case 0x1ee9:
    case 0x1eeb:
    case 0x1eed:
    case 0x1eef:
    case 0x1ef1:
      regmbc('u'); regmbc(0xf9); regmbc(0xfa);
      regmbc(0xfb); regmbc(0xfc); regmbc(0x169);
      regmbc(0x16b); regmbc(0x16d); regmbc(0x16f);
      regmbc(0x171); regmbc(0x173); regmbc(0x1d6);
      regmbc(0x1d8); regmbc(0x1da); regmbc(0x1dc);
      regmbc(0x215); regmbc(0x217); regmbc(0x1b0);
      regmbc(0x1d4); regmbc(0x289); regmbc(0x1d7e);
      regmbc(0x1d99); regmbc(0x1e73); regmbc(0x1e75);
      regmbc(0x1e77); regmbc(0x1e79); regmbc(0x1e7b);
      regmbc(0x1ee5); regmbc(0x1ee7); regmbc(0x1ee9);
      regmbc(0x1eeb); regmbc(0x1eed); regmbc(0x1eef);
      regmbc(0x1ef1);
      return;
    case 'v':
    case 0x28b:
    case 0x1d8c:
    case 0x1e7d:
    case 0x1e7f:
      regmbc('v'); regmbc(0x28b); regmbc(0x1d8c);
      regmbc(0x1e7d); regmbc(0x1e7f);
      return;
    case 'w':
    case 0x175:
    case 0x1e81:
    case 0x1e83:
    case 0x1e85:
    case 0x1e87:
    case 0x1e89:
    case 0x1e98:
      regmbc('w'); regmbc(0x175); regmbc(0x1e81);
      regmbc(0x1e83); regmbc(0x1e85); regmbc(0x1e87);
      regmbc(0x1e89); regmbc(0x1e98);
      return;
    case 'x':
    case 0x1e8b:
    case 0x1e8d:
      regmbc('x'); regmbc(0x1e8b); regmbc(0x1e8d);
      return;
    case 'y':
    case 0xfd:
    case 0xff:
    case 0x177:
    case 0x1b4:
    case 0x233:
    case 0x24f:
    case 0x1e8f:
    case 0x1e99:
    case 0x1ef3:
    case 0x1ef5:
    case 0x1ef7:
    case 0x1ef9:
      regmbc('y'); regmbc(0xfd); regmbc(0xff);
      regmbc(0x177); regmbc(0x1b4); regmbc(0x233);
      regmbc(0x24f); regmbc(0x1e8f); regmbc(0x1e99);
      regmbc(0x1ef3); regmbc(0x1ef5); regmbc(0x1ef7);
      regmbc(0x1ef9);
      return;
    case 'z':
    case 0x17a:
    case 0x17c:
    case 0x17e:
    case 0x1b6:
    case 0x1d76:
    case 0x1d8e:
    case 0x1e91:
    case 0x1e93:
    case 0x1e95:
    case 0x2c6c:
      regmbc('z'); regmbc(0x17a); regmbc(0x17c);
      regmbc(0x17e); regmbc(0x1b6); regmbc(0x1d76);
      regmbc(0x1d8e); regmbc(0x1e91); regmbc(0x1e93);
      regmbc(0x1e95); regmbc(0x2c6c);
      return;
    }
  }
  regmbc(c);
}

// Emit a node.
// Return pointer to generated code.
static uint8_t *regnode(int op)
{
  uint8_t *ret;

  ret = regcode;
  if (ret == JUST_CALC_SIZE) {
    regsize += 3;
  } else {
    *regcode++ = (uint8_t)op;
    *regcode++ = NUL;                   // Null "next" pointer.
    *regcode++ = NUL;
  }
  return ret;
}

// Rust implementation for re_put_uint32
extern uint8_t *rs_re_put_uint32(uint8_t *p, uint32_t val);

// Write a four bytes number at "p" and return pointer to the next char.
static uint8_t *re_put_uint32(uint8_t *p, uint32_t val)
{
  return rs_re_put_uint32(p, val);
}

// regnext - dig the "next" pointer out of a node
// Returns NULL when calculating size, when there is no next item and when
// there is an error.
static uint8_t *regnext(uint8_t *p)
  FUNC_ATTR_NONNULL_ALL
{
  int offset;

  if (p == JUST_CALC_SIZE || reg_toolong) {
    return NULL;
  }

  offset = NEXT(p);
  if (offset == 0) {
    return NULL;
  }

  if (OP(p) == BACK) {
    return p - offset;
  } else {
    return p + offset;
  }
}

// Set the next-pointer at the end of a node chain.
static void regtail(uint8_t *p, const uint8_t *val)
{
  int offset;

  if (p == JUST_CALC_SIZE) {
    return;
  }

  // Find last node.
  uint8_t *scan = p;
  while (true) {
    uint8_t *temp = regnext(scan);
    if (temp == NULL) {
      break;
    }
    scan = temp;
  }

  if (OP(scan) == BACK) {
    offset = (int)(scan - val);
  } else {
    offset = (int)(val - scan);
  }
  // When the offset uses more than 16 bits it can no longer fit in the two
  // bytes available.  Use a global flag to avoid having to check return
  // values in too many places.
  if (offset > 0xffff) {
    reg_toolong = true;
  } else {
    *(scan + 1) = (uint8_t)(((unsigned)offset >> 8) & 0377);
    *(scan + 2) = (uint8_t)(offset & 0377);
  }
}

// Like regtail, on item after a BRANCH; nop if none.
static void regoptail(uint8_t *p, uint8_t *val)
{
  // When op is neither BRANCH nor BRACE_COMPLEX0-9, it is "operandless"
  if (p == NULL || p == JUST_CALC_SIZE
      || (OP(p) != BRANCH
          && (OP(p) < BRACE_COMPLEX || OP(p) > BRACE_COMPLEX + 9))) {
    return;
  }
  regtail(OPERAND(p), val);
}

// Insert an operator in front of already-emitted operand
//
// Means relocating the operand.
static void reginsert(int op, uint8_t *opnd)
{
  uint8_t *src;
  uint8_t *dst;
  uint8_t *place;

  if (regcode == JUST_CALC_SIZE) {
    regsize += 3;
    return;
  }
  src = regcode;
  regcode += 3;
  dst = regcode;
  while (src > opnd) {
    *--dst = *--src;
  }

  place = opnd;                 // Op node, where operand used to be.
  *place++ = (uint8_t)op;
  *place++ = NUL;
  *place = NUL;
}

// Insert an operator in front of already-emitted operand.
// Add a number to the operator.
static void reginsert_nr(int op, int64_t val, uint8_t *opnd)
{
  uint8_t *src;
  uint8_t *dst;
  uint8_t *place;

  if (regcode == JUST_CALC_SIZE) {
    regsize += 7;
    return;
  }
  src = regcode;
  regcode += 7;
  dst = regcode;
  while (src > opnd) {
    *--dst = *--src;
  }

  place = opnd;                 // Op node, where operand used to be.
  *place++ = (uint8_t)op;
  *place++ = NUL;
  *place++ = NUL;
  assert(val >= 0 && (uintmax_t)val <= UINT32_MAX);
  re_put_uint32(place, (uint32_t)val);
}

// Insert an operator in front of already-emitted operand.
// The operator has the given limit values as operands.  Also set next pointer.
//
// Means relocating the operand.
static void reginsert_limits(int op, int64_t minval, int64_t maxval, uint8_t *opnd)
{
  uint8_t *src;
  uint8_t *dst;
  uint8_t *place;

  if (regcode == JUST_CALC_SIZE) {
    regsize += 11;
    return;
  }
  src = regcode;
  regcode += 11;
  dst = regcode;
  while (src > opnd) {
    *--dst = *--src;
  }

  place = opnd;                 // Op node, where operand used to be.
  *place++ = (uint8_t)op;
  *place++ = NUL;
  *place++ = NUL;
  assert(minval >= 0 && (uintmax_t)minval <= UINT32_MAX);
  place = re_put_uint32(place, (uint32_t)minval);
  assert(maxval >= 0 && (uintmax_t)maxval <= UINT32_MAX);
  place = re_put_uint32(place, (uint32_t)maxval);
  regtail(opnd, place);
}

/// Return true if the back reference is legal. We must have seen the close
/// brace.
/// TODO(vim): Should also check that we don't refer to something repeated
/// (+*=): what instance of the repetition should we match?
static int seen_endbrace(int refnum)
{
  if (!had_endbrace[refnum]) {
    uint8_t *p;

    // Trick: check if "@<=" or "@<!" follows, in which case
    // the \1 can appear before the referenced match.
    for (p = (uint8_t *)regparse; *p != NUL; p++) {
      if (p[0] == '@' && p[1] == '<' && (p[2] == '!' || p[2] == '=')) {
        break;
      }
    }

    if (*p == NUL) {
      emsg(_("E65: Illegal back reference"));
      rc_did_emsg = true;
      return false;
    }
  }
  return true;
}

// Parse the lowest level.
//
// Optimization:  gobbles an entire sequence of ordinary characters so that
// it can turn them into a single node, which is smaller to store and
// faster to run.  Don't do this when one_exactly is set.
static uint8_t *regatom(int *flagp)
{
  uint8_t *ret;
  int flags;
  int c;
  uint8_t *p;
  int extra = 0;
  int save_prev_at_start = prev_at_start;

  *flagp = WORST;               // Tentatively.

  c = getchr();
  switch (c) {
  case Magic('^'):
    ret = regnode(BOL);
    break;

  case Magic('$'):
    ret = regnode(EOL);
    had_eol = true;
    break;

  case Magic('<'):
    ret = regnode(BOW);
    break;

  case Magic('>'):
    ret = regnode(EOW);
    break;

  case Magic('_'):
    c = rs_no_magic(getchr());
    if (c == '^') {             // "\_^" is start-of-line
      ret = regnode(BOL);
      break;
    }
    if (c == '$') {             // "\_$" is end-of-line
      ret = regnode(EOL);
      had_eol = true;
      break;
    }

    extra = ADD_NL;
    *flagp |= HASNL;

    // "\_[" is character range plus newline
    if (c == '[') {
      goto collection;
    }

    // "\_x" is character class plus newline
    FALLTHROUGH;

  // Character classes.
  case Magic('.'):
  case Magic('i'):
  case Magic('I'):
  case Magic('k'):
  case Magic('K'):
  case Magic('f'):
  case Magic('F'):
  case Magic('p'):
  case Magic('P'):
  case Magic('s'):
  case Magic('S'):
  case Magic('d'):
  case Magic('D'):
  case Magic('x'):
  case Magic('X'):
  case Magic('o'):
  case Magic('O'):
  case Magic('w'):
  case Magic('W'):
  case Magic('h'):
  case Magic('H'):
  case Magic('a'):
  case Magic('A'):
  case Magic('l'):
  case Magic('L'):
  case Magic('u'):
  case Magic('U'):
    p = (uint8_t *)vim_strchr((char *)classchars, rs_no_magic(c));
    if (p == NULL) {
      EMSG_RET_NULL(_(e_invalid_use_of_underscore));
    }
    // When '.' is followed by a composing char ignore the dot, so that
    // the composing char is matched here.
    if (c == Magic('.') && utf_iscomposing_legacy(peekchr())) {
      c = getchr();
      goto do_multibyte;
    }
    ret = regnode(classcodes[p - classchars] + extra);
    *flagp |= HASWIDTH | SIMPLE;
    break;

  case Magic('n'):
    if (reg_string) {
      // In a string "\n" matches a newline character.
      ret = regnode(EXACTLY);
      regc(NL);
      regc(NUL);
      *flagp |= HASWIDTH | SIMPLE;
    } else {
      // In buffer text "\n" matches the end of a line.
      ret = regnode(NEWL);
      *flagp |= HASWIDTH | HASNL;
    }
    break;

  case Magic('('):
    if (one_exactly) {
      EMSG_ONE_RET_NULL;
    }
    ret = reg(REG_PAREN, &flags);
    if (ret == NULL) {
      return NULL;
    }
    *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
    break;

  case NUL:
  case Magic('|'):
  case Magic('&'):
  case Magic(')'):
    if (one_exactly) {
      EMSG_ONE_RET_NULL;
    }
    // Supposed to be caught earlier.
    IEMSG_RET_NULL(_(e_internal_error_in_regexp));
  // NOTREACHED

  case Magic('='):
  case Magic('?'):
  case Magic('+'):
  case Magic('@'):
  case Magic('{'):
  case Magic('*'):
    c = rs_no_magic(c);
    EMSG3_RET_NULL(_("E64: %s%c follows nothing"),
                   (c == '*' ? reg_magic >= MAGIC_ON : reg_magic == MAGIC_ALL), c);
  // NOTREACHED

  case Magic('~'):              // previous substitute pattern
    if (reg_prev_sub != NULL) {
      uint8_t *lp;

      ret = regnode(EXACTLY);
      lp = (uint8_t *)reg_prev_sub;
      while (*lp != NUL) {
        regc(*lp++);
      }
      regc(NUL);
      if (*reg_prev_sub != NUL) {
        *flagp |= HASWIDTH;
        if ((lp - (uint8_t *)reg_prev_sub) == 1) {
          *flagp |= SIMPLE;
        }
      }
    } else {
      EMSG_RET_NULL(_(e_nopresub));
    }
    break;

  case Magic('1'):
  case Magic('2'):
  case Magic('3'):
  case Magic('4'):
  case Magic('5'):
  case Magic('6'):
  case Magic('7'):
  case Magic('8'):
  case Magic('9'): {
    int refnum;

    refnum = c - Magic('0');
    if (!seen_endbrace(refnum)) {
      return NULL;
    }
    ret = regnode(BACKREF + refnum);
  }
  break;

  case Magic('z'):
    c = rs_no_magic(getchr());
    switch (c) {
    case '(':
      if ((reg_do_extmatch & REX_SET) == 0) {
        EMSG_RET_NULL(_(e_z_not_allowed));
      }
      if (one_exactly) {
        EMSG_ONE_RET_NULL;
      }
      ret = reg(REG_ZPAREN, &flags);
      if (ret == NULL) {
        return NULL;
      }
      *flagp |= flags & (HASWIDTH|SPSTART|HASNL|HASLOOKBH);
      re_has_z = REX_SET;
      break;

    case '1':
    case '2':
    case '3':
    case '4':
    case '5':
    case '6':
    case '7':
    case '8':
    case '9':
      if ((reg_do_extmatch & REX_USE) == 0) {
        EMSG_RET_NULL(_(e_z1_not_allowed));
      }
      ret = regnode(ZREF + c - '0');
      re_has_z = REX_USE;
      break;

    case 's':
      ret = regnode(MOPEN + 0);
      if (!re_mult_next("\\zs")) {
        return NULL;
      }
      break;

    case 'e':
      ret = regnode(MCLOSE + 0);
      if (!re_mult_next("\\ze")) {
        return NULL;
      }
      break;

    default:
      EMSG_RET_NULL(_("E68: Invalid character after \\z"));
    }
    break;

  case Magic('%'):
    c = rs_no_magic(getchr());
    switch (c) {
    // () without a back reference
    case '(':
      if (one_exactly) {
        EMSG_ONE_RET_NULL;
      }
      ret = reg(REG_NPAREN, &flags);
      if (ret == NULL) {
        return NULL;
      }
      *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
      break;

    // Catch \%^ and \%$ regardless of where they appear in the
    // pattern -- regardless of whether or not it makes sense.
    case '^':
      ret = regnode(RE_BOF);
      break;

    case '$':
      ret = regnode(RE_EOF);
      break;

    case '#':
      if (regparse[0] == '=' && regparse[1] >= 48 && regparse[1] <= 50) {
        // misplaced \%#=1
        semsg(_(e_atom_engine_must_be_at_start_of_pattern), regparse[1]);
        return FAIL;
      }
      ret = regnode(CURSOR);
      break;

    case 'V':
      ret = regnode(RE_VISUAL);
      break;

    case 'C':
      ret = regnode(RE_COMPOSING);
      break;

    // \%[abc]: Emit as a list of branches, all ending at the last
    // branch which matches nothing.
    case '[':
      if (one_exactly) {                        // doesn't nest
        EMSG_ONE_RET_NULL;
      }
      {
        uint8_t *lastbranch;
        uint8_t *lastnode = NULL;
        uint8_t *br;

        ret = NULL;
        while ((c = getchr()) != ']') {
          if (c == NUL) {
            EMSG2_RET_NULL(_(e_missing_sb),
                           reg_magic == MAGIC_ALL);
          }
          br = regnode(BRANCH);
          if (ret == NULL) {
            ret = br;
          } else {
            regtail(lastnode, br);
            if (reg_toolong) {
              return NULL;
            }
          }

          ungetchr();
          one_exactly = true;
          lastnode = regatom(flagp);
          one_exactly = false;
          if (lastnode == NULL) {
            return NULL;
          }
        }
        if (ret == NULL) {
          EMSG2_RET_NULL(_(e_empty_sb),
                         reg_magic == MAGIC_ALL);
        }
        lastbranch = regnode(BRANCH);
        br = regnode(NOTHING);
        if (ret != JUST_CALC_SIZE) {
          regtail(lastnode, br);
          regtail(lastbranch, br);
          // connect all branches to the NOTHING
          // branch at the end
          for (br = ret; br != lastnode;) {
            if (OP(br) == BRANCH) {
              regtail(br, lastbranch);
              if (reg_toolong) {
                return NULL;
              }
              br = OPERAND(br);
            } else {
              br = regnext(br);
            }
          }
        }
        *flagp &= ~(HASWIDTH | SIMPLE);
        break;
      }

    case 'd':               // %d123 decimal
    case 'o':               // %o123 octal
    case 'x':               // %xab hex 2
    case 'u':               // %uabcd hex 4
    case 'U':               // %U1234abcd hex 8
    {
      int64_t i;

      switch (c) {
      case 'd':
        i = getdecchrs(); break;
      case 'o':
        i = getoctchrs(); break;
      case 'x':
        i = gethexchrs(2); break;
      case 'u':
        i = gethexchrs(4); break;
      case 'U':
        i = gethexchrs(8); break;
      default:
        i = -1; break;
      }

      if (i < 0 || i > INT_MAX) {
        EMSG2_RET_NULL(_("E678: Invalid character after %s%%[dxouU]"),
                       reg_magic == MAGIC_ALL);
      }
      if (use_multibytecode((int)i)) {
        ret = regnode(MULTIBYTECODE);
      } else {
        ret = regnode(EXACTLY);
      }
      if (i == 0) {
        regc(0x0a);
      } else {
        regmbc((int)i);
      }
      regc(NUL);
      *flagp |= HASWIDTH;
      break;
    }

    default:
      if (ascii_isdigit(c) || c == '<' || c == '>' || c == '\'' || c == '.') {
        uint32_t n = 0;
        int cmp;
        bool cur = false;
        bool got_digit = false;

        cmp = c;
        if (cmp == '<' || cmp == '>') {
          c = getchr();
        }
        if (rs_no_magic(c) == '.') {
          cur = true;
          c = getchr();
        }
        while (ascii_isdigit(c)) {
          got_digit = true;
          n = n * 10 + (uint32_t)(c - '0');
          c = getchr();
        }
        if (rs_no_magic(c) == '\'' && n == 0) {
          // "\%'m", "\%<'m" and "\%>'m": Mark
          c = getchr();
          ret = regnode(RE_MARK);
          if (ret == JUST_CALC_SIZE) {
            regsize += 2;
          } else {
            *regcode++ = (uint8_t)c;
            *regcode++ = (uint8_t)cmp;
          }
          break;
        } else if ((c == 'l' || c == 'c' || c == 'v') && (cur || got_digit)) {
          if (cur && n) {
            semsg(_(e_regexp_number_after_dot_pos_search_chr), rs_no_magic(c));
            rc_did_emsg = true;
            return NULL;
          }
          if (c == 'l') {
            if (cur) {
              n = (uint32_t)curwin->w_cursor.lnum;
            }
            ret = regnode(RE_LNUM);
            if (save_prev_at_start) {
              at_start = true;
            }
          } else if (c == 'c') {
            if (cur) {
              n = (uint32_t)curwin->w_cursor.col;
              n++;
            }
            ret = regnode(RE_COL);
          } else {
            if (cur) {
              colnr_T vcol = 0;
              getvvcol(curwin, &curwin->w_cursor, NULL, NULL, &vcol);
              n = (uint32_t)(++vcol);
            }
            ret = regnode(RE_VCOL);
          }
          if (ret == JUST_CALC_SIZE) {
            regsize += 5;
          } else {
            // put the number and the optional
            // comparator after the opcode
            regcode = re_put_uint32(regcode, n);
            *regcode++ = (uint8_t)cmp;
          }
          break;
        }
      }

      EMSG2_RET_NULL(_("E71: Invalid character after %s%%"),
                     reg_magic == MAGIC_ALL);
    }
    break;

  case Magic('['):
collection:
    {
      uint8_t *lp;

      // If there is no matching ']', we assume the '[' is a normal
      // character.  This makes 'incsearch' and ":help [" work.
      lp = (uint8_t *)skip_anyof(regparse);
      if (*lp == ']') {         // there is a matching ']'
        int startc = -1;                // > 0 when next '-' is a range
        int endc;

        // In a character class, different parsing rules apply.
        // Not even \ is special anymore, nothing is.
        if (*regparse == '^') {             // Complement of range.
          ret = regnode(ANYBUT + extra);
          regparse++;
        } else {
          ret = regnode(ANYOF + extra);
        }

        // At the start ']' and '-' mean the literal character.
        if (*regparse == ']' || *regparse == '-') {
          startc = (uint8_t)(*regparse);
          regc(*regparse++);
        }

        while (*regparse != NUL && *regparse != ']') {
          if (*regparse == '-') {
            regparse++;
            // The '-' is not used for a range at the end and
            // after or before a '\n'.
            if (*regparse == ']' || *regparse == NUL
                || startc == -1
                || (regparse[0] == '\\' && regparse[1] == 'n')) {
              regc('-');
              startc = '-';                     // [--x] is a range
            } else {
              // Also accept "a-[.z.]"
              endc = 0;
              if (*regparse == '[') {
                endc = rs_get_coll_element(&regparse);
              }
              if (endc == 0) {
                endc = mb_ptr2char_adv((const char **)&regparse);
              }

              // Handle \o40, \x20 and \u20AC style sequences
              if (endc == '\\' && !reg_cpo_lit) {
                endc = coll_get_char();
              }

              if (startc > endc) {
                EMSG_RET_NULL(_(e_reverse_range));
              }
              if (utf_char2len(startc) > 1
                  || utf_char2len(endc) > 1) {
                // Limit to a range of 256 chars
                if (endc > startc + 256) {
                  EMSG_RET_NULL(_(e_large_class));
                }
                while (++startc <= endc) {
                  regmbc(startc);
                }
              } else {
                while (++startc <= endc) {
                  regc(startc);
                }
              }
              startc = -1;
            }
          }
          // Only "\]", "\^", "\]" and "\\" are special in Vi.  Vim
          // accepts "\t", "\e", etc., but only when the 'l' flag in
          // 'cpoptions' is not included.
          else if (*regparse == '\\'
                   && (vim_strchr(REGEXP_INRANGE, (uint8_t)regparse[1]) != NULL
                       || (!reg_cpo_lit
                           && vim_strchr(REGEXP_ABBR,
                                         (uint8_t)regparse[1]) != NULL))) {
            regparse++;
            if (*regparse == 'n') {
              // '\n' in range: also match NL
              if (ret != JUST_CALC_SIZE) {
                // Using \n inside [^] does not change what
                // matches. "[^\n]" is the same as ".".
                if (*ret == ANYOF) {
                  *ret = ANYOF + ADD_NL;
                  *flagp |= HASNL;
                }
                // else: must have had a \n already
              }
              regparse++;
              startc = -1;
            } else if (*regparse == 'd'
                       || *regparse == 'o'
                       || *regparse == 'x'
                       || *regparse == 'u'
                       || *regparse == 'U') {
              startc = coll_get_char();
              // max UTF-8 Codepoint is U+10FFFF,
              // but allow values until INT_MAX
              if (startc == INT_MAX) {
                EMSG_RET_NULL(_(e_unicode_val_too_large));
              }
              if (startc == 0) {
                regc(0x0a);
              } else {
                regmbc(startc);
              }
            } else {
              startc = rs_backslash_trans(*regparse++);
              regc(startc);
            }
          } else if (*regparse == '[') {
            int c_class;
            int cu;

            c_class = rs_get_char_class(&regparse);
            startc = -1;
            // Characters assumed to be 8 bits!
            switch (c_class) {
            case CLASS_NONE:
              c_class = rs_get_equi_class(&regparse);
              if (c_class != 0) {
                // produce equivalence class
                reg_equi_class(c_class);
              } else if ((c_class = rs_get_coll_element(&regparse)) != 0) {
                // produce a collating element
                regmbc(c_class);
              } else {
                // literal '[', allow [[-x] as a range
                startc = (uint8_t)(*regparse++);
                regc(startc);
              }
              break;
            case CLASS_ALNUM:
              for (cu = 1; cu < 128; cu++) {
                if (isalnum(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_ALPHA:
              for (cu = 1; cu < 128; cu++) {
                if (isalpha(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_BLANK:
              regc(' ');
              regc('\t');
              break;
            case CLASS_CNTRL:
              for (cu = 1; cu <= 127; cu++) {
                if (iscntrl(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_DIGIT:
              for (cu = 1; cu <= 127; cu++) {
                if (ascii_isdigit(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_GRAPH:
              for (cu = 1; cu <= 127; cu++) {
                if (isgraph(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_LOWER:
              for (cu = 1; cu <= 255; cu++) {
                if (mb_islower(cu) && cu != 170 && cu != 186) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_PRINT:
              for (cu = 1; cu <= 255; cu++) {
                if (vim_isprintc(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_PUNCT:
              for (cu = 1; cu < 128; cu++) {
                if (ispunct(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_SPACE:
              for (cu = 9; cu <= 13; cu++) {
                regc(cu);
              }
              regc(' ');
              break;
            case CLASS_UPPER:
              for (cu = 1; cu <= 255; cu++) {
                if (mb_isupper(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_XDIGIT:
              for (cu = 1; cu <= 255; cu++) {
                if (ascii_isxdigit(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_TAB:
              regc('\t');
              break;
            case CLASS_RETURN:
              regc('\r');
              break;
            case CLASS_BACKSPACE:
              regc('\b');
              break;
            case CLASS_ESCAPE:
              regc(ESC);
              break;
            case CLASS_IDENT:
              for (cu = 1; cu <= 255; cu++) {
                if (vim_isIDc(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_KEYWORD:
              for (cu = 1; cu <= 255; cu++) {
                if (reg_iswordc(cu)) {
                  regmbc(cu);
                }
              }
              break;
            case CLASS_FNAME:
              for (cu = 1; cu <= 255; cu++) {
                if (vim_isfilec(cu)) {
                  regmbc(cu);
                }
              }
              break;
            }
          } else {
            // produce a multibyte character, including any
            // following composing characters.
            startc = utf_ptr2char(regparse);
            int len = utfc_ptr2len(regparse);
            if (utf_char2len(startc) != len) {
              // composing chars
              startc = -1;
            }
            while (--len >= 0) {
              regc(*regparse++);
            }
          }
        }
        regc(NUL);
        prevchr_len = 1;                // last char was the ']'
        if (*regparse != ']') {
          EMSG_RET_NULL(_(e_toomsbra));                 // Cannot happen?
        }
        skipchr();                  // let's be friends with the lexer again
        *flagp |= HASWIDTH | SIMPLE;
        break;
      } else if (reg_strict) {
        EMSG2_RET_NULL(_(e_missingbracket), reg_magic > MAGIC_OFF);
      }
    }
    FALLTHROUGH;

  default: {
    int len;

    // A multi-byte character is handled as a separate atom if it's
    // before a multi and when it's a composing char.
    if (use_multibytecode(c)) {
do_multibyte:
      ret = regnode(MULTIBYTECODE);
      regmbc(c);
      *flagp |= HASWIDTH | SIMPLE;
      break;
    }

    ret = regnode(EXACTLY);

    // Append characters as long as:
    // - there is no following multi, we then need the character in
    //   front of it as a single character operand
    // - not running into a Magic character
    // - "one_exactly" is not set
    // But always emit at least one character.  Might be a Multi,
    // e.g., a "[" without matching "]".
    for (len = 0; c != NUL && (len == 0
                               || (rs_re_multi_type(peekchr()) == NOT_MULTI
                                   && !one_exactly
                                   && !is_Magic(c))); len++) {
      c = rs_no_magic(c);
      {
        regmbc(c);
        {
          int l;

          // Need to get composing character too.
          GraphemeState state = GRAPHEME_STATE_INIT;
          while (true) {
            l = utf_ptr2len(regparse);
            if (!utf_composinglike(regparse, regparse + l, &state)) {
              break;
            }
            regmbc(utf_ptr2char(regparse));
            skipchr();
          }
        }
      }
      c = getchr();
    }
    ungetchr();

    regc(NUL);
    *flagp |= HASWIDTH;
    if (len == 1) {
      *flagp |= SIMPLE;
    }
  }
  break;
  }

  return ret;
}

// Parse something followed by possible [*+=].
//
// Note that the branching code sequences used for = and the general cases
// of * and + are somewhat optimized:  they use the same NOTHING node as
// both the endmarker for their branch list and the body of the last branch.
// It might seem that this node could be dispensed with entirely, but the
// endmarker role is not redundant.
static uint8_t *regpiece(int *flagp)
{
  uint8_t *ret;
  int op;
  uint8_t *next;
  int flags;
  int minval;
  int maxval;

  ret = regatom(&flags);
  if (ret == NULL) {
    return NULL;
  }

  op = peekchr();
  if (rs_re_multi_type(op) == NOT_MULTI) {
    *flagp = flags;
    return ret;
  }
  // default flags
  *flagp = (WORST | SPSTART | (flags & (HASNL | HASLOOKBH)));

  skipchr();
  switch (op) {
  case Magic('*'):
    if (flags & SIMPLE) {
      reginsert(STAR, ret);
    } else {
      // Emit x* as (x&|), where & means "self".
      reginsert(BRANCH, ret);           // Either x
      regoptail(ret, regnode(BACK));            // and loop
      regoptail(ret, ret);              // back
      regtail(ret, regnode(BRANCH));            // or
      regtail(ret, regnode(NOTHING));           // null.
    }
    break;

  case Magic('+'):
    if (flags & SIMPLE) {
      reginsert(PLUS, ret);
    } else {
      // Emit x+ as x(&|), where & means "self".
      next = regnode(BRANCH);           // Either
      regtail(ret, next);
      regtail(regnode(BACK), ret);              // loop back
      regtail(next, regnode(BRANCH));           // or
      regtail(ret, regnode(NOTHING));           // null.
    }
    *flagp = (WORST | HASWIDTH | (flags & (HASNL | HASLOOKBH)));
    break;

  case Magic('@'): {
    int lop = END;
    int64_t nr = getdecchrs();

    switch (rs_no_magic(getchr())) {
    case '=':
      lop = MATCH; break;                                 // \@=
    case '!':
      lop = NOMATCH; break;                               // \@!
    case '>':
      lop = SUBPAT; break;                                // \@>
    case '<':
      switch (rs_no_magic(getchr())) {
      case '=':
        lop = BEHIND; break;                               // \@<=
      case '!':
        lop = NOBEHIND; break;                             // \@<!
      }
    }
    if (lop == END) {
      EMSG2_RET_NULL(_(e_invalid_character_after_str_at),
                     reg_magic == MAGIC_ALL);
    }
    // Look behind must match with behind_pos.
    if (lop == BEHIND || lop == NOBEHIND) {
      regtail(ret, regnode(BHPOS));
      *flagp |= HASLOOKBH;
    }
    regtail(ret, regnode(END));             // operand ends
    if (lop == BEHIND || lop == NOBEHIND) {
      if (nr < 0) {
        nr = 0;                 // no limit is same as zero limit
      }
      reginsert_nr(lop, (uint32_t)nr, ret);
    } else {
      reginsert(lop, ret);
    }
    break;
  }

  case Magic('?'):
  case Magic('='):
    // Emit x= as (x|)
    reginsert(BRANCH, ret);                     // Either x
    regtail(ret, regnode(BRANCH));              // or
    next = regnode(NOTHING);                    // null.
    regtail(ret, next);
    regoptail(ret, next);
    break;

  case Magic('{'):
    if (!read_limits(&minval, &maxval)) {
      return NULL;
    }
    if (flags & SIMPLE) {
      reginsert(BRACE_SIMPLE, ret);
      reginsert_limits(BRACE_LIMITS, minval, maxval, ret);
    } else {
      if (num_complex_braces >= 10) {
        EMSG2_RET_NULL(_("E60: Too many complex %s{...}s"),
                       reg_magic == MAGIC_ALL);
      }
      reginsert(BRACE_COMPLEX + num_complex_braces, ret);
      regoptail(ret, regnode(BACK));
      regoptail(ret, ret);
      reginsert_limits(BRACE_LIMITS, minval, maxval, ret);
      num_complex_braces++;
    }
    if (minval > 0 && maxval > 0) {
      *flagp = (HASWIDTH | (flags & (HASNL | HASLOOKBH)));
    }
    break;
  }
  if (rs_re_multi_type(peekchr()) != NOT_MULTI) {
    // Can't have a multi follow a multi.
    if (peekchr() == Magic('*')) {
      EMSG2_RET_NULL(_("E61: Nested %s*"), reg_magic >= MAGIC_ON);
    }
    EMSG3_RET_NULL(_("E62: Nested %s%c"), reg_magic == MAGIC_ALL, rs_no_magic(peekchr()));
  }

  return ret;
}

// Parse one alternative of an | or & operator.
// Implements the concatenation operator.
static uint8_t *regconcat(int *flagp)
{
  uint8_t *first = NULL;
  uint8_t *chain = NULL;
  uint8_t *latest;
  int flags;
  int cont = true;

  *flagp = WORST;               // Tentatively.

  while (cont) {
    switch (peekchr()) {
    case NUL:
    case Magic('|'):
    case Magic('&'):
    case Magic(')'):
      cont = false;
      break;
    case Magic('Z'):
      regflags |= RF_ICOMBINE;
      skipchr_keepstart();
      break;
    case Magic('c'):
      regflags |= RF_ICASE;
      skipchr_keepstart();
      break;
    case Magic('C'):
      regflags |= RF_NOICASE;
      skipchr_keepstart();
      break;
    case Magic('v'):
      reg_magic = MAGIC_ALL;
      skipchr_keepstart();
      curchr = -1;
      break;
    case Magic('m'):
      reg_magic = MAGIC_ON;
      skipchr_keepstart();
      curchr = -1;
      break;
    case Magic('M'):
      reg_magic = MAGIC_OFF;
      skipchr_keepstart();
      curchr = -1;
      break;
    case Magic('V'):
      reg_magic = MAGIC_NONE;
      skipchr_keepstart();
      curchr = -1;
      break;
    default:
      latest = regpiece(&flags);
      if (latest == NULL || reg_toolong) {
        return NULL;
      }
      *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
      if (chain == NULL) {                      // First piece.
        *flagp |= flags & SPSTART;
      } else {
        regtail(chain, latest);
      }
      chain = latest;
      if (first == NULL) {
        first = latest;
      }
      break;
    }
  }
  if (first == NULL) {          // Loop ran zero times.
    first = regnode(NOTHING);
  }
  return first;
}

// Parse one alternative of an | operator.
// Implements the & operator.
static uint8_t *regbranch(int *flagp)
{
  uint8_t *ret;
  uint8_t *chain = NULL;
  uint8_t *latest;
  int flags;

  *flagp = WORST | HASNL;               // Tentatively.

  ret = regnode(BRANCH);
  while (true) {
    latest = regconcat(&flags);
    if (latest == NULL) {
      return NULL;
    }
    // If one of the branches has width, the whole thing has.  If one of
    // the branches anchors at start-of-line, the whole thing does.
    // If one of the branches uses look-behind, the whole thing does.
    *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
    // If one of the branches doesn't match a line-break, the whole thing
    // doesn't.
    *flagp &= ~HASNL | (flags & HASNL);
    if (chain != NULL) {
      regtail(chain, latest);
    }
    if (peekchr() != Magic('&')) {
      break;
    }
    skipchr();
    regtail(latest, regnode(END));     // operand ends
    if (reg_toolong) {
      break;
    }
    reginsert(MATCH, latest);
    chain = latest;
  }

  return ret;
}

/// Parse regular expression, i.e. main body or parenthesized thing.
///
/// Caller must absorb opening parenthesis.
///
/// Combining parenthesis handling with the base level of regular expression
/// is a trifle forced, but the need to tie the tails of the branches to what
/// follows makes it hard to avoid.
///
/// @param paren  REG_NOPAREN, REG_PAREN, REG_NPAREN or REG_ZPAREN
static uint8_t *reg(int paren, int *flagp)
{
  uint8_t *ret;
  uint8_t *br;
  uint8_t *ender;
  int parno = 0;
  int flags;

  *flagp = HASWIDTH;            // Tentatively.

  if (paren == REG_ZPAREN) {
    // Make a ZOPEN node.
    if (regnzpar >= NSUBEXP) {
      EMSG_RET_NULL(_("E50: Too many \\z("));
    }
    parno = regnzpar;
    regnzpar++;
    ret = regnode(ZOPEN + parno);
  } else if (paren == REG_PAREN) {
    // Make a MOPEN node.
    if (regnpar >= NSUBEXP) {
      EMSG2_RET_NULL(_("E51: Too many %s("), reg_magic == MAGIC_ALL);
    }
    parno = regnpar;
    regnpar++;
    ret = regnode(MOPEN + parno);
  } else if (paren == REG_NPAREN) {
    // Make a NOPEN node.
    ret = regnode(NOPEN);
  } else {
    ret = NULL;
  }

  // Pick up the branches, linking them together.
  br = regbranch(&flags);
  if (br == NULL) {
    return NULL;
  }
  if (ret != NULL) {
    regtail(ret, br);           // [MZ]OPEN -> first.
  } else {
    ret = br;
  }
  // If one of the branches can be zero-width, the whole thing can.
  // If one of the branches has * at start or matches a line-break, the
  // whole thing can.
  if (!(flags & HASWIDTH)) {
    *flagp &= ~HASWIDTH;
  }
  *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
  while (peekchr() == Magic('|')) {
    skipchr();
    br = regbranch(&flags);
    if (br == NULL || reg_toolong) {
      return NULL;
    }
    regtail(ret, br);           // BRANCH -> BRANCH.
    if (!(flags & HASWIDTH)) {
      *flagp &= ~HASWIDTH;
    }
    *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
  }

  // Make a closing node, and hook it on the end.
  ender = regnode(paren == REG_ZPAREN ? ZCLOSE + parno
                                      : paren == REG_PAREN ? MCLOSE + parno
                                                           : paren == REG_NPAREN ? NCLOSE : END);
  regtail(ret, ender);

  // Hook the tails of the branches to the closing node.
  for (br = ret; br != NULL; br = regnext(br)) {
    regoptail(br, ender);
  }

  // Check for proper termination.
  if (paren != REG_NOPAREN && getchr() != Magic(')')) {
    if (paren == REG_ZPAREN) {
      EMSG_RET_NULL(_("E52: Unmatched \\z("));
    } else if (paren == REG_NPAREN) {
      EMSG2_RET_NULL(_(e_unmatchedpp), reg_magic == MAGIC_ALL);
    } else {
      EMSG2_RET_NULL(_(e_unmatchedp), reg_magic == MAGIC_ALL);
    }
  } else if (paren == REG_NOPAREN && peekchr() != NUL) {
    if (curchr == Magic(')')) {
      EMSG2_RET_NULL(_(e_unmatchedpar), reg_magic == MAGIC_ALL);
    } else {
      EMSG_RET_NULL(_(e_trailing));             // "Can't happen".
    }
    // NOTREACHED
  }
  // Here we set the flag allowing back references to this set of
  // parentheses.
  if (paren == REG_PAREN) {
    had_endbrace[parno] = true;  // have seen the close paren
  }
  return ret;
}

// bt_regcomp() - compile a regular expression into internal code for the
// traditional back track matcher.
// Returns the program in allocated space.  Returns NULL for an error.
//
// We can't allocate space until we know how big the compiled form will be,
// but we can't compile it (and thus know how big it is) until we've got a
// place to put the code.  So we cheat:  we compile it twice, once with code
// generation turned off and size counting turned on, and once "for real".
// This also means that we don't allocate space until we are sure that the
// thing really will compile successfully, and we never have to move the
// code and thus invalidate pointers into it.  (Note that it has to be in
// one piece because free() must be able to free it all.)
//
// Whether upper/lower case is to be ignored is decided when executing the
// program, it does not matter here.
//
// Beware that the optimization-preparation code in here knows about some
// of the structure of the compiled regexp.
// "re_flags": RE_MAGIC and/or RE_STRING.
static regprog_T *bt_regcomp(uint8_t *expr, int re_flags)
{
  uint8_t *scan;
  int flags;

  if (expr == NULL) {
    IEMSG_RET_NULL(_(e_null));
  }

  init_class_tab();

  // First pass: determine size, legality.
  regcomp_start(expr, re_flags);
  regcode = JUST_CALC_SIZE;
  regc(REGMAGIC);
  if (reg(REG_NOPAREN, &flags) == NULL) {
    return NULL;
  }

  // Allocate space.
  bt_regprog_T *r = xmalloc(offsetof(bt_regprog_T, program) + (size_t)regsize);
  r->re_in_use = false;

  // Second pass: emit code.
  regcomp_start(expr, re_flags);
  regcode = r->program;
  regc(REGMAGIC);
  if (reg(REG_NOPAREN, &flags) == NULL || reg_toolong) {
    xfree(r);
    if (reg_toolong) {
      EMSG_RET_NULL(_("E339: Pattern too long"));
    }
    return NULL;
  }

  // Dig out information for optimizations.
  r->regstart = NUL;            // Worst-case defaults.
  r->reganch = 0;
  r->regmust = NULL;
  r->regmlen = 0;
  r->regflags = regflags;
  if (flags & HASNL) {
    r->regflags |= RF_HASNL;
  }
  if (flags & HASLOOKBH) {
    r->regflags |= RF_LOOKBH;
  }
  // Remember whether this pattern has any \z specials in it.
  r->reghasz = (uint8_t)re_has_z;
  scan = &r->program[1];  // First BRANCH.
  if (OP(regnext(scan)) == END) {   // Only one top-level choice.
    scan = OPERAND(scan);

    // Starting-point info.
    if (OP(scan) == BOL || OP(scan) == RE_BOF) {
      r->reganch++;
      scan = regnext(scan);
    }

    // Use Rust implementation for regstart extraction
    r->regstart = rs_bt_get_regstart(scan);

    // If there's something expensive in the r.e., find the longest
    // literal string that must appear and make it the regmust.  Resolve
    // ties in favor of later strings, since the regstart check works
    // with the beginning of the r.e. and avoiding duplication
    // strengthens checking.  Not a strong reason, but sufficient in the
    // absence of others.

    // When the r.e. starts with BOW, it is faster to look for a regmust
    // first. Used a lot for "#" and "*" commands. (Added by mool).
    if ((flags & SPSTART || OP(scan) == BOW || OP(scan) == EOW)
        && !(flags & HASNL)) {
      // Use Rust implementation for regmust extraction
      r->regmust = rs_bt_find_regmust(scan, flags, &r->regmlen);
    }
  }
#ifdef BT_REGEXP_DUMP
  regdump(expr, r);
#endif
  r->engine = &bt_regengine;
  return (regprog_T *)r;
}

// C accessor for had_eol static variable (used by Rust)
int nvim_get_regexp_had_eol(void) { return had_eol; }

// C accessor for regprog_T fields (used by Rust)
int nvim_regprog_get_regflags(const regprog_T *prog) { return (int)prog->regflags; }

// C accessor for reg_cpo_lit static variable (used by Rust)
int nvim_get_reg_cpo_lit(void) { return reg_cpo_lit; }

// C accessors for reg_prev_sub (used by Rust regtilde)
char *nvim_get_reg_prev_sub(void) { return reg_prev_sub; }
size_t nvim_get_reg_prev_sublen(void) { return reg_prev_sublen; }
void nvim_set_reg_prev_sub(char *s, size_t len) {
  xfree(reg_prev_sub);
  reg_prev_sub = s;
  reg_prev_sublen = len;
}

// =============================================================================
// Phase 4: rex structure accessors (used by Rust)
// =============================================================================

// Current position accessors
linenr_T nvim_rex_get_lnum(void) { return rex.lnum; }
void nvim_rex_set_lnum(linenr_T lnum) { rex.lnum = lnum; }
uint8_t *nvim_rex_get_line(void) { return rex.line; }
void nvim_rex_set_line(uint8_t *line) { rex.line = line; }
uint8_t *nvim_rex_get_input(void) { return rex.input; }
void nvim_rex_set_input(uint8_t *input) { rex.input = input; }

// Match state accessors
regmatch_T *nvim_rex_get_reg_match(void) { return rex.reg_match; }
void nvim_rex_set_reg_match(regmatch_T *m) { rex.reg_match = m; }
regmmatch_T *nvim_rex_get_reg_mmatch(void) { return rex.reg_mmatch; }
void nvim_rex_set_reg_mmatch(regmmatch_T *m) { rex.reg_mmatch = m; }
void nvim_regmmatch_set_rmm_matchcol(regmmatch_T *m, colnr_T col)
{
  if (m != NULL) {
    m->rmm_matchcol = col;
  }
}

// regmatch_T accessors for Rust (Phase 12c)
void *nvim_regmatch_get_regprog(regmatch_T *m) { return m ? m->regprog : NULL; }
void **nvim_regmatch_get_startp(regmatch_T *m) { return m ? (void **)m->startp : NULL; }
void **nvim_regmatch_get_endp(regmatch_T *m) { return m ? (void **)m->endp : NULL; }
void nvim_regmatch_set_rm_matchcol(regmatch_T *m, colnr_T col)
{
  if (m != NULL) {
    m->rm_matchcol = col;
  }
}

// regmmatch_T accessors for Rust (Phase 12c)
void *nvim_regmmatch_get_regprog(regmmatch_T *m) { return m ? m->regprog : NULL; }
lpos_T *nvim_regmmatch_get_startpos(regmmatch_T *m) { return m ? m->startpos : NULL; }
lpos_T *nvim_regmmatch_get_endpos(regmmatch_T *m) { return m ? m->endpos : NULL; }

// Initialize NFA program states for execution (Phase 12c)
static void nfa_init_prog_states(nfa_regprog_T *prog)
{
  for (int i = 0; i < prog->nstate; i++) {
    prog->state[i].id = i;
    prog->state[i].lastlist[0] = 0;
    prog->state[i].lastlist[1] = 0;
  }
}

// Exposed wrapper for Rust (uses void* to avoid header export issues)
void nvim_nfa_init_prog_states(void *prog)
{
  nfa_init_prog_states((nfa_regprog_T *)prog);
}

// Submatch position accessors
uint8_t **nvim_rex_get_reg_startp(void) { return rex.reg_startp; }
void nvim_rex_set_reg_startp(uint8_t **p) { rex.reg_startp = p; }
uint8_t **nvim_rex_get_reg_endp(void) { return rex.reg_endp; }
void nvim_rex_set_reg_endp(uint8_t **p) { rex.reg_endp = p; }
lpos_T *nvim_rex_get_reg_startpos(void) { return rex.reg_startpos; }
void nvim_rex_set_reg_startpos(lpos_T *p) { rex.reg_startpos = p; }
lpos_T *nvim_rex_get_reg_endpos(void) { return rex.reg_endpos; }
void nvim_rex_set_reg_endpos(lpos_T *p) { rex.reg_endpos = p; }

// Buffer/window context accessors
win_T *nvim_rex_get_reg_win(void) { return rex.reg_win; }
void nvim_rex_set_reg_win(win_T *win) { rex.reg_win = win; }
buf_T *nvim_rex_get_reg_buf(void) { return rex.reg_buf; }
void nvim_rex_set_reg_buf(buf_T *buf) { rex.reg_buf = buf; }
uint64_t *nvim_rex_get_reg_buf_chartab(void) { return rex.reg_buf ? rex.reg_buf->b_chartab : NULL; }
linenr_T nvim_rex_get_reg_firstlnum(void) { return rex.reg_firstlnum; }

// Cursor position accessors for position matching (NFA_CURSOR)
linenr_T nvim_rex_get_cursor_lnum(void) {
  return rex.reg_win != NULL ? rex.reg_win->w_cursor.lnum : 0;
}
colnr_T nvim_rex_get_cursor_col(void) {
  return rex.reg_win != NULL ? rex.reg_win->w_cursor.col : 0;
}
void nvim_rex_set_reg_firstlnum(linenr_T lnum) { rex.reg_firstlnum = lnum; }
linenr_T nvim_rex_get_reg_maxline(void) { return rex.reg_maxline; }
void nvim_rex_set_reg_maxline(linenr_T lnum) { rex.reg_maxline = lnum; }

// Flag accessors
bool nvim_rex_get_reg_ic(void) { return rex.reg_ic; }
void nvim_rex_set_reg_ic(bool ic) { rex.reg_ic = ic; }
bool nvim_rex_get_reg_icombine(void) { return rex.reg_icombine; }
void nvim_rex_set_reg_icombine(bool ic) { rex.reg_icombine = ic; }
bool nvim_rex_get_reg_line_lbr(void) { return rex.reg_line_lbr; }
void nvim_rex_set_reg_line_lbr(bool lbr) { rex.reg_line_lbr = lbr; }
bool nvim_rex_get_reg_nobreak(void) { return rex.reg_nobreak; }
void nvim_rex_set_reg_nobreak(bool nb) { rex.reg_nobreak = nb; }
colnr_T nvim_rex_get_reg_maxcol(void) { return rex.reg_maxcol; }
void nvim_rex_set_reg_maxcol(colnr_T col) { rex.reg_maxcol = col; }

// Subexpression clearing flags
int nvim_rex_get_need_clear_subexpr(void) { return rex.need_clear_subexpr; }
void nvim_rex_set_need_clear_subexpr(int v) { rex.need_clear_subexpr = v; }
int nvim_rex_get_need_clear_zsubexpr(void) { return rex.need_clear_zsubexpr; }
void nvim_rex_set_need_clear_zsubexpr(int v) { rex.need_clear_zsubexpr = v; }

// reg_prev_class wrapper for Rust
int nvim_rex_reg_prev_class(void) { return reg_prev_class(); }

// NFA engine state accessors
int nvim_rex_get_nfa_has_zend(void) { return rex.nfa_has_zend; }
void nvim_rex_set_nfa_has_zend(int v) { rex.nfa_has_zend = v; }
int nvim_rex_get_nfa_has_backref(void) { return rex.nfa_has_backref; }
void nvim_rex_set_nfa_has_backref(int v) { rex.nfa_has_backref = v; }
int nvim_rex_get_nfa_nsubexpr(void) { return rex.nfa_nsubexpr; }
void nvim_rex_set_nfa_nsubexpr(int v) { rex.nfa_nsubexpr = v; }
int nvim_rex_get_nfa_listid(void) { return rex.nfa_listid; }
void nvim_rex_set_nfa_listid(int v) { rex.nfa_listid = v; }
int nvim_rex_get_nfa_alt_listid(void) { return rex.nfa_alt_listid; }
void nvim_rex_set_nfa_alt_listid(int v) { rex.nfa_alt_listid = v; }
int nvim_rex_get_nfa_has_zsubexpr(void) { return rex.nfa_has_zsubexpr; }
void nvim_rex_set_nfa_has_zsubexpr(int v) { rex.nfa_has_zsubexpr = v; }

// rex_in_use flag accessor
bool nvim_rex_in_use(void) { return rex_in_use; }
void nvim_rex_set_in_use(bool in_use) { rex_in_use = in_use; }

// REG_MULTI check
int nvim_rex_is_multi(void) { return rex.reg_match == NULL; }

// Character classification wrappers for Rust (these are macros in C)
int nvim_ri_digit(int c) { return ri_digit(c); }
int nvim_ri_hex(int c) { return ri_hex(c); }
int nvim_ri_octal(int c) { return ri_octal(c); }
int nvim_ri_word(int c) { return ri_word(c); }
int nvim_ri_head(int c) { return ri_head(c); }
int nvim_ri_alpha(int c) { return ri_alpha(c); }
int nvim_ri_lower(int c) { return ri_lower(c); }
int nvim_ri_upper(int c) { return ri_upper(c); }

// Memory limit (p_mmp)
int64_t nvim_get_p_mmp(void) { return p_mmp; }

// Error for max memory pattern
void nvim_regexp_emsg_maxmempattern(void)
{
  emsg(_(e_pattern_uses_more_memory_than_maxmempattern));
}

// =============================================================================
// Position matching wrappers for Rust (Phase 5.8)
// =============================================================================

/// Check NFA_VCOL match. Returns true if the position matches.
/// op: 0 = exact, 1 = greater than, 2 = less than
bool nvim_nfa_check_vcol(int val, int op)
{
  colnr_T col = (colnr_T)(rex.input - rex.line);

  // Bail out quickly when there can't be a match, avoid the overhead of
  // win_linetabsize() on long lines.
  if (op != 1 && col > val * MB_MAXBYTES) {
    return false;
  }

  bool result = false;
  win_T *wp = rex.reg_win == NULL ? curwin : rex.reg_win;
  if (op == 1 && col - 1 > val && col > 100) {
    int64_t ts = (int64_t)wp->w_buffer->b_p_ts;

    // Guess that a character won't use more columns than 'tabstop',
    // with a minimum of 4.
    if (ts < 4) {
      ts = 4;
    }
    result = col > val * ts;
  }
  if (!result) {
    linenr_T lnum = REG_MULTI ? rex.reg_firstlnum + rex.lnum : 1;
    if (REG_MULTI && (lnum <= 0 || lnum > wp->w_buffer->b_ml.ml_line_count)) {
      lnum = 1;
    }
    int vcol = win_linetabsize(wp, lnum, (char *)rex.line, col);
    assert(val >= 0);
    result = nfa_re_num_cmp((uintmax_t)val, op, (uintmax_t)vcol + 1);
  }
  return result;
}

/// Check NFA_MARK match. Returns true if the position matches.
/// mark_id: the mark identifier (state->val)
/// op: 0 = exact, 1 = greater than, 2 = less than
bool nvim_nfa_check_mark(int mark_id, int op)
{
  size_t col = REG_MULTI ? (size_t)(rex.input - rex.line) : 0;
  fmark_T *fm = mark_get(rex.reg_buf, curwin, NULL, kMarkBufLocal, mark_id);

  // Line may have been freed, get it again.
  if (REG_MULTI) {
    rex.line = (uint8_t *)reg_getline(rex.lnum);
    rex.input = rex.line + col;
  }

  // Compare the mark position to the match position, if the mark
  // exists and mark is set in reg_buf.
  if (fm == NULL || fm->mark.lnum <= 0) {
    return false;
  }

  pos_T *pos = &fm->mark;
  const colnr_T pos_col = pos->lnum == rex.lnum + rex.reg_firstlnum
                          && pos->col == MAXCOL
                          ? reg_getline_len(pos->lnum - rex.reg_firstlnum)
                          : pos->col;

  // op: 0 = NFA_MARK (exact), 1 = NFA_MARK_GT, 2 = NFA_MARK_LT
  if (pos->lnum == rex.lnum + rex.reg_firstlnum) {
    if (pos_col == (colnr_T)(rex.input - rex.line)) {
      return op == 0;  // NFA_MARK
    }
    if (pos_col < (colnr_T)(rex.input - rex.line)) {
      return op == 1;  // NFA_MARK_GT
    }
    return op == 2;  // NFA_MARK_LT
  }
  if (pos->lnum < rex.lnum + rex.reg_firstlnum) {
    return op == 1;  // NFA_MARK_GT
  }
  return op == 2;  // NFA_MARK_LT
}

/// Check NFA_VISUAL match. Returns true if the position is in the visual area.
bool nvim_nfa_check_visual(void)
{
  return reg_match_visual();
}

// wants_nfa - pattern requires NFA engine (for BT-only patterns like [[:upper:]])
int nvim_parse_get_wants_nfa(void) { return wants_nfa; }
void nvim_parse_set_wants_nfa(int v) { wants_nfa = v; }

// =============================================================================
// Phase 5: Parse state accessors (used by Rust)
// =============================================================================

// regparse - input scan pointer
char *nvim_parse_get_regparse(void) { return regparse; }
void nvim_parse_set_regparse(char *p) { regparse = p; }

// prevchr_len - byte length of previous char
int nvim_parse_get_prevchr_len(void) { return prevchr_len; }
void nvim_parse_set_prevchr_len(int len) { prevchr_len = len; }

// curchr - currently parsed character
int nvim_parse_get_curchr(void) { return curchr; }
void nvim_parse_set_curchr(int c) { curchr = c; }

// prevchr - previous character
int nvim_parse_get_prevchr(void) { return prevchr; }
void nvim_parse_set_prevchr(int c) { prevchr = c; }

// prevprevchr - previous-previous character
int nvim_parse_get_prevprevchr(void) { return prevprevchr; }
void nvim_parse_set_prevprevchr(int c) { prevprevchr = c; }

// nextchr - used for ungetchr()
int nvim_parse_get_nextchr(void) { return nextchr; }
void nvim_parse_set_nextchr(int c) { nextchr = c; }

// at_start - true when on first character
int nvim_parse_get_at_start(void) { return at_start; }
void nvim_parse_set_at_start(int v) { at_start = v; }

// prev_at_start - true when on second character
int nvim_parse_get_prev_at_start(void) { return prev_at_start; }
void nvim_parse_set_prev_at_start(int v) { prev_at_start = v; }

// regnpar - parenthesis count
int nvim_parse_get_regnpar(void) { return regnpar; }
void nvim_parse_set_regnpar(int n) { regnpar = n; }

// regnzpar - external subexpression count
int nvim_parse_get_regnzpar(void) { return regnzpar; }
void nvim_parse_set_regnzpar(int n) { regnzpar = n; }

// had_endbrace - flags for closed parentheses
int nvim_parse_get_had_endbrace(int i) { return i >= 0 && i < NSUBEXP ? had_endbrace[i] : 0; }
void nvim_parse_set_had_endbrace(int i, int v) { if (i >= 0 && i < NSUBEXP) had_endbrace[i] = (uint8_t)v; }

// regflags - RF_ flags for compiled program
int nvim_parse_get_regflags(void) { return (int)regflags; }
void nvim_parse_set_regflags(int f) { regflags = (unsigned)f; }

// reg_string - matching with string vs buffer
int nvim_parse_get_reg_string(void) { return reg_string; }

// reg_strict - whether "[abc" is illegal
int nvim_parse_get_reg_strict(void) { return reg_strict; }

// reg_cpo_lit - cpoptions 'l' flag
int nvim_parse_get_reg_cpo_lit(void) { return reg_cpo_lit; }

// reg_magic - magicness of pattern
int nvim_parse_get_reg_magic(void) { return (int)reg_magic; }
void nvim_parse_set_reg_magic(int m) { reg_magic = (magic_T)m; }

// re_has_z - \z item detected
int nvim_parse_get_re_has_z(void) { return re_has_z; }
void nvim_parse_set_re_has_z(int v) { re_has_z = v; }

// reg_do_extmatch - external match context (REX_SET or REX_USE)
int nvim_parse_get_reg_do_extmatch(void) { return reg_do_extmatch; }

// had_eol - EOL found during compilation
int nvim_parse_get_had_eol(void) { return had_eol; }
void nvim_parse_set_had_eol(int v) { had_eol = v; }

// classchars - character class characters (for nfa_regatom)
const uint8_t *nvim_parse_get_classchars(void) { return classchars; }

// Wrapper for re_mult_next
int nvim_re_mult_next(const char *what) { return re_mult_next((char *)what); }

// Wrapper for seen_endbrace
int nvim_seen_endbrace(int refnum) { return seen_endbrace(refnum); }

// Wrapper for skip_anyof
const char *nvim_skip_anyof(const char *p) { return skip_anyof(p); }

// prev_at_start save/restore helper
int nvim_parse_get_save_prev_at_start(void) { return prev_at_start; }

// getvvcol wrapper for current cursor position
void nvim_getvvcol_curwin(colnr_T *vcol) {
  getvvcol(curwin, &curwin->w_cursor, NULL, NULL, vcol);
}

// =============================================================================
// Phase 1.1: bt_regprog_T accessors (used by Rust BT engine)
// =============================================================================

// bt_regprog_T field accessors
int nvim_bt_regprog_get_regstart(const regprog_T *prog)
{
  return ((const bt_regprog_T *)prog)->regstart;
}

int nvim_bt_regprog_get_reganch(const regprog_T *prog)
{
  return ((const bt_regprog_T *)prog)->reganch;
}

const uint8_t *nvim_bt_regprog_get_regmust(const regprog_T *prog)
{
  return ((const bt_regprog_T *)prog)->regmust;
}

int nvim_bt_regprog_get_regmlen(const regprog_T *prog)
{
  return ((const bt_regprog_T *)prog)->regmlen;
}

int nvim_bt_regprog_get_reghasz(const regprog_T *prog)
{
  return ((const bt_regprog_T *)prog)->reghasz;
}

const uint8_t *nvim_bt_regprog_get_program(const regprog_T *prog)
{
  return ((const bt_regprog_T *)prog)->program;
}

// =============================================================================
// Phase 1.3: nfa_regprog_T accessors (used by Rust NFA engine)
// Using void* for opaque handle pattern since nfa_state_T is internal
// =============================================================================

// nfa_regprog_T field accessors
void *nvim_nfa_regprog_get_start(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->start;
}

int nvim_nfa_regprog_get_reganch(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->reganch;
}

int nvim_nfa_regprog_get_regstart(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->regstart;
}

const uint8_t *nvim_nfa_regprog_get_match_text(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->match_text;
}

int nvim_nfa_regprog_get_has_zend(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->has_zend;
}

int nvim_nfa_regprog_get_has_backref(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->has_backref;
}

int nvim_nfa_regprog_get_reghasz(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->reghasz;
}

const char *nvim_nfa_regprog_get_pattern(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->pattern;
}

int nvim_nfa_regprog_get_nsubexp(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->nsubexp;
}

int nvim_nfa_regprog_get_nstate(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->nstate;
}

unsigned nvim_nfa_regprog_get_re_engine(const regprog_T *prog)
{
  return ((const nfa_regprog_T *)prog)->re_engine;
}

void *nvim_nfa_regprog_get_state(const regprog_T *prog, int idx)
{
  const nfa_regprog_T *nfa = (const nfa_regprog_T *)prog;
  if (idx >= 0 && idx < nfa->nstate) {
    return (void *)&nfa->state[idx];
  }
  return NULL;
}

// nfa_state_T field accessors (using void* opaque handles)
int nvim_nfa_state_get_c(const void *state)
{
  return ((const nfa_state_T *)state)->c;
}

void *nvim_nfa_state_get_out(const void *state)
{
  return ((const nfa_state_T *)state)->out;
}

void *nvim_nfa_state_get_out1(const void *state)
{
  return ((const nfa_state_T *)state)->out1;
}

int nvim_nfa_state_get_id(const void *state)
{
  return ((const nfa_state_T *)state)->id;
}

int nvim_nfa_state_get_lastlist(const void *state, int idx)
{
  const nfa_state_T *s = (const nfa_state_T *)state;
  return (idx == 0 || idx == 1) ? s->lastlist[idx] : 0;
}

void nvim_nfa_state_set_lastlist(void *state, int idx, int val)
{
  nfa_state_T *s = (nfa_state_T *)state;
  if (idx == 0 || idx == 1) {
    s->lastlist[idx] = val;
  }
}

int nvim_nfa_state_get_val(const void *state)
{
  return ((const nfa_state_T *)state)->val;
}

// Helper functions for number parsing
int nvim_hex2nr(int c) { return hex2nr(c); }
int nvim_ascii_isxdigit(int c) { return ascii_isxdigit(c); }

// Error reporting for Rust code
void nvim_regexp_report_error(int error_id, int is_magic_all)
{
  switch (error_id) {
  case 554:  // E554: Syntax error in %s{...}
    semsg(_("E554: Syntax error in %s{...}"), is_magic_all ? "" : "\\");
    break;
  default:
    semsg("Unknown regexp error: E%d", error_id);
    break;
  }
  rc_did_emsg = true;
}

// Rust implementation
extern int rs_vim_regcomp_had_eol(void);

// Check if during the previous call to vim_regcomp the EOL item "$" has been
// found.  This is messy, but it works fine.
int vim_regcomp_had_eol(void)
{
  return rs_vim_regcomp_had_eol();
}

// Get a number after a backslash that is inside [].
// When nothing is recognized return a backslash.
static int coll_get_char(void)
{
  int64_t nr = -1;

  switch (*regparse++) {
  case 'd':
    nr = getdecchrs(); break;
  case 'o':
    nr = getoctchrs(); break;
  case 'x':
    nr = gethexchrs(2); break;
  case 'u':
    nr = gethexchrs(4); break;
  case 'U':
    nr = gethexchrs(8); break;
  }
  if (nr < 0) {
    // If getting the number fails be backwards compatible: the character
    // is a backslash.
    regparse--;
    nr = '\\';
  }
  if (nr > INT_MAX) {
    nr = INT_MAX;
  }
  return (int)nr;
}

// Free a compiled regexp program, returned by bt_regcomp().
static void bt_regfree(regprog_T *prog)
{
  xfree(prog);
}

#define ADVANCE_REGINPUT() MB_PTR_ADV(rex.input)

// The arguments from BRACE_LIMITS are stored here.  They are actually local
// to regmatch(), but they are here to reduce the amount of stack space used
// (it can be called recursively many times).
static int64_t bl_minval;
static int64_t bl_maxval;

// Save the input line and position in a regsave_T.
static void reg_save(regsave_T *save, garray_T *gap)
  FUNC_ATTR_NONNULL_ALL
{
  if (REG_MULTI) {
    save->rs_u.pos.col = (colnr_T)(rex.input - rex.line);
    save->rs_u.pos.lnum = rex.lnum;
  } else {
    save->rs_u.ptr = rex.input;
  }
  save->rs_len = gap->ga_len;
}

// Restore the input line and position from a regsave_T.
static void reg_restore(regsave_T *save, garray_T *gap)
  FUNC_ATTR_NONNULL_ALL
{
  if (REG_MULTI) {
    if (rex.lnum != save->rs_u.pos.lnum) {
      // only call reg_getline() when the line number changed to save
      // a bit of time
      rex.lnum = save->rs_u.pos.lnum;
      rex.line = (uint8_t *)reg_getline(rex.lnum);
    }
    rex.input = rex.line + save->rs_u.pos.col;
  } else {
    rex.input = save->rs_u.ptr;
  }
  gap->ga_len = save->rs_len;
}

// Return true if current position is equal to saved position.
static bool reg_save_equal(const regsave_T *save)
  FUNC_ATTR_NONNULL_ALL
{
  if (REG_MULTI) {
    return rex.lnum == save->rs_u.pos.lnum
           && rex.input == rex.line + save->rs_u.pos.col;
  }
  return rex.input == save->rs_u.ptr;
}

// Save the sub-expressions before attempting a match.
#define save_se(savep, posp, pp) \
  REG_MULTI ? save_se_multi((savep), (posp)) : save_se_one((savep), (pp))

// After a failed match restore the sub-expressions.
#define restore_se(savep, posp, pp) { \
  if (REG_MULTI) \
  *(posp) = (savep)->se_u.pos; \
  else \
  *(pp) = (savep)->se_u.ptr; }

// Tentatively set the sub-expression start to the current position (after
// calling regmatch() they will have changed).  Need to save the existing
// values for when there is no match.
// Use se_save() to use pointer (save_se_multi()) or position (save_se_one()),
// depending on REG_MULTI.
static void save_se_multi(save_se_T *savep, lpos_T *posp)
{
  savep->se_u.pos = *posp;
  posp->lnum = rex.lnum;
  posp->col = (colnr_T)(rex.input - rex.line);
}

static void save_se_one(save_se_T *savep, uint8_t **pp)
{
  savep->se_u.ptr = *pp;
  *pp = rex.input;
}

/// regrepeat - repeatedly match something simple, return how many.
/// Advances rex.input (and rex.lnum) to just after the matched chars.
///
/// @param maxcount  maximum number of matches allowed
static int regrepeat(uint8_t *p, int64_t maxcount)
{
  return rs_regrepeat(p, maxcount);
}

// Push an item onto the regstack.
// Returns pointer to new item.  Returns NULL when out of memory.
static regitem_T *regstack_push(regstate_T state, uint8_t *scan)
{
  regitem_T *rp;

  if ((int64_t)((unsigned)regstack.ga_len >> 10) >= p_mmp) {
    emsg(_(e_pattern_uses_more_memory_than_maxmempattern));
    return NULL;
  }
  ga_grow(&regstack, sizeof(regitem_T));

  rp = (regitem_T *)((char *)regstack.ga_data + regstack.ga_len);
  rp->rs_state = state;
  rp->rs_scan = scan;

  regstack.ga_len += (int)sizeof(regitem_T);
  return rp;
}

// Pop an item from the regstack.
static void regstack_pop(uint8_t **scan)
{
  regitem_T *rp;

  rp = (regitem_T *)((char *)regstack.ga_data + regstack.ga_len) - 1;
  *scan = rp->rs_scan;

  regstack.ga_len -= (int)sizeof(regitem_T);
}

// Save the current subexpr to "bp", so that they can be restored
// later by restore_subexpr().
static void save_subexpr(regbehind_T *bp)
  FUNC_ATTR_NONNULL_ALL
{
  // When "rex.need_clear_subexpr" is set we don't need to save the values, only
  // remember that this flag needs to be set again when restoring.
  bp->save_need_clear_subexpr = rex.need_clear_subexpr;
  if (rex.need_clear_subexpr) {
    return;
  }

  for (int i = 0; i < NSUBEXP; i++) {
    if (REG_MULTI) {
      bp->save_start[i].se_u.pos = rex.reg_startpos[i];
      bp->save_end[i].se_u.pos = rex.reg_endpos[i];
    } else {
      bp->save_start[i].se_u.ptr = rex.reg_startp[i];
      bp->save_end[i].se_u.ptr = rex.reg_endp[i];
    }
  }
}

// Restore the subexpr from "bp".
static void restore_subexpr(regbehind_T *bp)
  FUNC_ATTR_NONNULL_ALL
{
  // Only need to restore saved values when they are not to be cleared.
  rex.need_clear_subexpr = bp->save_need_clear_subexpr;
  if (rex.need_clear_subexpr) {
    return;
  }

  for (int i = 0; i < NSUBEXP; i++) {
    if (REG_MULTI) {
      rex.reg_startpos[i] = bp->save_start[i].se_u.pos;
      rex.reg_endpos[i] = bp->save_end[i].se_u.pos;
    } else {
      rex.reg_startp[i] = bp->save_start[i].se_u.ptr;
      rex.reg_endp[i] = bp->save_end[i].se_u.ptr;
    }
  }
}
/// Main matching routine
///
/// Conceptually the strategy is simple: Check to see whether the current node
/// matches, push an item onto the regstack and loop to see whether the rest
/// matches, and then act accordingly.  In practice we make some effort to
/// avoid using the regstack, in particular by going through "ordinary" nodes
/// (that don't need to know whether the rest of the match failed) by a nested
/// loop.
///
/// @param scan       Current node.
/// @param tm         timeout limit or NULL
/// @param timed_out  flag set on timeout or NULL
///
/// @return - true when there is a match.  Leaves rex.input and rex.lnum
///         just after the last matched character.
///         - false when there is no match.  Leaves rex.input and rex.lnum in an
///         undefined state!
static bool regmatch(uint8_t *scan, const proftime_T *tm, int *timed_out)
{
  uint8_t *next;          // Next node.
  int op;
  int c;
  regitem_T *rp;
  int no;
  int status;                   // one of the RA_ values:
  int tm_count = 0;

  // Make "regstack" and "backpos" empty.  They are allocated and freed in
  // bt_regexec_both() to reduce malloc()/free() calls.
  regstack.ga_len = 0;
  backpos.ga_len = 0;

  // Repeat until "regstack" is empty.
  while (true) {
    // Some patterns may take a long time to match, e.g., "\([a-z]\+\)\+Q".
    // Allow interrupting them with CTRL-C.
    reg_breakcheck();

#ifdef REGEXP_DEBUG
    if (scan != NULL && regnarrate) {
      fprintf(stderr, "%s", (char *)regprop(scan));
      fprintf(stderr, "%s", "(\n");
    }
#endif

    // Repeat for items that can be matched sequentially, without using the
    // regstack.
    while (true) {
      if (got_int || scan == NULL) {
        status = RA_FAIL;
        break;
      }
      // Check for timeout once in a 100 times to avoid overhead.
      if (tm != NULL && ++tm_count == 100) {
        tm_count = 0;
        if (profile_passed_limit(*tm)) {
          if (timed_out != NULL) {
            *timed_out = true;
          }
          status = RA_FAIL;
          break;
        }
      }
      status = RA_CONT;

#ifdef REGEXP_DEBUG
      if (regnarrate) {
        fprintf(stderr, "%s", (char *)regprop(scan));
        fprintf(stderr, "%s", "...\n");
        if (re_extmatch_in != NULL) {
          int i;

          fprintf(stderr, _("External submatches:\n"));
          for (i = 0; i < NSUBEXP; i++) {
            fprintf(stderr, "%s", "    \"");
            if (re_extmatch_in->matches[i] != NULL) {
              fprintf(stderr, "%s", (char *)re_extmatch_in->matches[i]);
            }
            fprintf(stderr, "%s", "\"\n");
          }
        }
      }
#endif
      next = regnext(scan);

      op = OP(scan);
      // Check for character class with NL added.
      if (!rex.reg_line_lbr && WITH_NL(op) && REG_MULTI
          && *rex.input == NUL && rex.lnum <= rex.reg_maxline) {
        reg_nextline();
      } else if (rex.reg_line_lbr && WITH_NL(op) && *rex.input == '\n') {
        ADVANCE_REGINPUT();
      } else {
        if (WITH_NL(op)) {
          op -= ADD_NL;
        }
        c = utf_ptr2char((char *)rex.input);
        switch (op) {
        case BOL:
          if (rex.input != rex.line) {
            status = RA_NOMATCH;
          }
          break;

        case EOL:
          if (c != NUL) {
            status = RA_NOMATCH;
          }
          break;

        case RE_BOF:
          // We're not at the beginning of the file when below the first
          // line where we started, not at the start of the line or we
          // didn't start at the first line of the buffer.
          if (rex.lnum != 0 || rex.input != rex.line
              || (REG_MULTI && rex.reg_firstlnum > 1)) {
            status = RA_NOMATCH;
          }
          break;

        case RE_EOF:
          if (rex.lnum != rex.reg_maxline || c != NUL) {
            status = RA_NOMATCH;
          }
          break;

        case CURSOR:
          // Check if the buffer is in a window and compare the
          // rex.reg_win->w_cursor position to the match position.
          if (rex.reg_win == NULL
              || (rex.lnum + rex.reg_firstlnum != rex.reg_win->w_cursor.lnum)
              || ((colnr_T)(rex.input - rex.line) !=
                  rex.reg_win->w_cursor.col)) {
            status = RA_NOMATCH;
          }
          break;

        case RE_MARK:
          // Compare the mark position to the match position.
        {
          int mark = OPERAND(scan)[0];
          int cmp = OPERAND(scan)[1];
          pos_T *pos;
          size_t col = REG_MULTI ? (size_t)(rex.input - rex.line) : 0;
          fmark_T *fm = mark_get(rex.reg_buf, curwin, NULL, kMarkBufLocal, mark);

          // Line may have been freed, get it again.
          if (REG_MULTI) {
            rex.line = (uint8_t *)reg_getline(rex.lnum);
            rex.input = rex.line + col;
          }

          if (fm == NULL                    // mark doesn't exist
              || fm->mark.lnum <= 0) {           // mark isn't set in reg_buf
            status = RA_NOMATCH;
          } else {
            pos = &fm->mark;
            const colnr_T pos_col = pos->lnum == rex.lnum + rex.reg_firstlnum
                                    && pos->col == MAXCOL
                                    ? reg_getline_len(pos->lnum - rex.reg_firstlnum)
                                    : pos->col;

            if (pos->lnum == rex.lnum + rex.reg_firstlnum
                ? (pos_col == (colnr_T)(rex.input - rex.line)
                   ? (cmp == '<' || cmp == '>')
                   : (pos_col < (colnr_T)(rex.input - rex.line)
                      ? cmp != '>'
                      : cmp != '<'))
                : (pos->lnum < rex.lnum + rex.reg_firstlnum
                   ? cmp != '>'
                   : cmp != '<')) {
              status = RA_NOMATCH;
            }
          }
        }
        break;

        case RE_VISUAL:
          if (!reg_match_visual()) {
            status = RA_NOMATCH;
          }
          break;

        case RE_LNUM:
          assert(rex.lnum + rex.reg_firstlnum >= 0
                 && (uintmax_t)(rex.lnum + rex.reg_firstlnum) <= UINT32_MAX);
          if (!REG_MULTI
              || !re_num_cmp((uint32_t)(rex.lnum + rex.reg_firstlnum), scan)) {
            status = RA_NOMATCH;
          }
          break;

        case RE_COL:
          assert(rex.input - rex.line + 1 >= 0
                 && (uintmax_t)(rex.input - rex.line + 1) <= UINT32_MAX);
          if (!re_num_cmp((uint32_t)(rex.input - rex.line + 1), scan)) {
            status = RA_NOMATCH;
          }
          break;

        case RE_VCOL: {
          win_T *wp = rex.reg_win == NULL ? curwin : rex.reg_win;
          linenr_T lnum = REG_MULTI ? rex.reg_firstlnum + rex.lnum : 1;
          if (REG_MULTI && (lnum <= 0 || lnum > wp->w_buffer->b_ml.ml_line_count)) {
            lnum = 1;
          }
          int vcol = win_linetabsize(wp, lnum, (char *)rex.line,
                                     (colnr_T)(rex.input - rex.line));
          if (!re_num_cmp((uint32_t)vcol + 1, scan)) {
            status = RA_NOMATCH;
          }
          break;
        }
        break;

        case BOW:  // \<word; rex.input points to w
          if (c == NUL) {  // Can't match at end of line
            status = RA_NOMATCH;
          } else {
            // Get class of current and previous char (if it exists).
            const int this_class =
              mb_get_class_tab((char *)rex.input, rex.reg_buf->b_chartab);
            if (this_class <= 1) {
              status = RA_NOMATCH;  // Not on a word at all.
            } else if (reg_prev_class() == this_class) {
              status = RA_NOMATCH;  // Previous char is in same word.
            }
          }
          break;

        case EOW:  // word\>; rex.input points after d
          if (rex.input == rex.line) {  // Can't match at start of line
            status = RA_NOMATCH;
          } else {
            int this_class, prev_class;

            // Get class of current and previous char (if it exists).
            this_class = mb_get_class_tab((char *)rex.input, rex.reg_buf->b_chartab);
            prev_class = reg_prev_class();
            if (this_class == prev_class
                || prev_class == 0 || prev_class == 1) {
              status = RA_NOMATCH;
            }
          }
          break;  // Matched with EOW

        case ANY:
          // ANY does not match new lines.
          if (c == NUL) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case IDENT:
          if (!vim_isIDc(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case SIDENT:
          if (ascii_isdigit(*rex.input) || !vim_isIDc(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case KWORD:
          if (!vim_iswordp_buf((char *)rex.input, rex.reg_buf)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case SKWORD:
          if (ascii_isdigit(*rex.input)
              || !vim_iswordp_buf((char *)rex.input, rex.reg_buf)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case FNAME:
          if (!vim_isfilec(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case SFNAME:
          if (ascii_isdigit(*rex.input) || !vim_isfilec(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case PRINT:
          if (!vim_isprintc(utf_ptr2char((char *)rex.input))) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case SPRINT:
          if (ascii_isdigit(*rex.input) || !vim_isprintc(utf_ptr2char((char *)rex.input))) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case WHITE:
          if (!ascii_iswhite(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NWHITE:
          if (c == NUL || ascii_iswhite(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case DIGIT:
          if (!ri_digit(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NDIGIT:
          if (c == NUL || ri_digit(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case HEX:
          if (!ri_hex(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NHEX:
          if (c == NUL || ri_hex(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case OCTAL:
          if (!ri_octal(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NOCTAL:
          if (c == NUL || ri_octal(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case WORD:
          if (!ri_word(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NWORD:
          if (c == NUL || ri_word(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case HEAD:
          if (!ri_head(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NHEAD:
          if (c == NUL || ri_head(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case ALPHA:
          if (!ri_alpha(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NALPHA:
          if (c == NUL || ri_alpha(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case LOWER:
          if (!ri_lower(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NLOWER:
          if (c == NUL || ri_lower(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case UPPER:
          if (!ri_upper(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case NUPPER:
          if (c == NUL || ri_upper(c)) {
            status = RA_NOMATCH;
          } else {
            ADVANCE_REGINPUT();
          }
          break;

        case EXACTLY: {
          int len;
          uint8_t *opnd;

          opnd = OPERAND(scan);
          // Inline the first byte, for speed.
          if (*opnd != *rex.input
              && (!rex.reg_ic)) {
            status = RA_NOMATCH;
          } else if (*opnd == NUL) {
            // match empty string always works; happens when "~" is
            // empty.
          } else {
            if (opnd[1] == NUL && !rex.reg_ic) {
              len = 1;  // matched a single byte above
            } else {
              // Need to match first byte again for multi-byte.
              len = (int)strlen((char *)opnd);
              if (cstrncmp((char *)opnd, (char *)rex.input, &len) != 0) {
                status = RA_NOMATCH;
              }
            }
            // Check for following composing character, unless %C
            // follows (skips over all composing chars).
            if (status != RA_NOMATCH
                && utf_composinglike((char *)rex.input, (char *)rex.input + len, NULL)
                && !rex.reg_icombine
                && OP(next) != RE_COMPOSING) {
              // raaron: This code makes a composing character get
              // ignored, which is the correct behavior (sometimes)
              // for voweled Hebrew texts.
              status = RA_NOMATCH;
            }
            if (status != RA_NOMATCH) {
              rex.input += len;
            }
          }
        }
        break;

        case ANYOF:
        case ANYBUT: {
          uint8_t *q = OPERAND(scan);

          if (c == NUL) {
            status = RA_NOMATCH;
          } else if ((cstrchr((char *)q, c) == NULL) == (op == ANYOF)) {
            status = RA_NOMATCH;
          } else {  // Check following combining characters
            int len = utfc_ptr2len((char *)q) - utf_ptr2len((char *)q);

            rex.input += utf_ptr2len((char *)rex.input);
            q += utf_ptr2len((char *)q);

            if (len == 0) {
              break;
            }

            for (int i = 0; i < len; i++) {
              if (q[i] != rex.input[i]) {
                status = RA_NOMATCH;
                break;
              }
            }
            rex.input += len;
          }
          break;
        }

        case MULTIBYTECODE: {
          int i, len;

          const uint8_t *opnd = OPERAND(scan);
          // Safety check (just in case 'encoding' was changed since
          // compiling the program).
          if ((len = utfc_ptr2len((char *)opnd)) < 2) {
            status = RA_NOMATCH;
            break;
          }
          const int opndc = utf_ptr2char((char *)opnd);
          if (utf_iscomposing_legacy(opndc)) {
            // When only a composing char is given match at any
            // position where that composing char appears.
            status = RA_NOMATCH;
            for (i = 0; rex.input[i] != NUL;
                 i += utf_ptr2len((char *)rex.input + i)) {
              const int inpc = utf_ptr2char((char *)rex.input + i);
              if (!utf_iscomposing_legacy(inpc)) {
                if (i > 0) {
                  break;
                }
              } else if (opndc == inpc) {
                // Include all following composing chars.
                len = i + utfc_ptr2len((char *)rex.input + i);
                status = RA_MATCH;
                break;
              }
            }
          } else {
            if (cstrncmp((char *)opnd, (char *)rex.input, &len) != 0) {
              status = RA_NOMATCH;
              break;
            }
          }
          rex.input += len;
        }
        break;

        case RE_COMPOSING:
          // Skip composing characters.
          while (utf_iscomposing_legacy(utf_ptr2char((char *)rex.input))) {
            rex.input += utf_ptr2len((char *)rex.input);
          }
          break;

        case NOTHING:
          break;

        case BACK: {
          int i;

          // When we run into BACK we need to check if we don't keep
          // looping without matching any input.  The second and later
          // times a BACK is encountered it fails if the input is still
          // at the same position as the previous time.
          // The positions are stored in "backpos" and found by the
          // current value of "scan", the position in the RE program.
          backpos_T *bp = (backpos_T *)backpos.ga_data;
          for (i = 0; i < backpos.ga_len; i++) {
            if (bp[i].bp_scan == scan) {
              break;
            }
          }
          if (i == backpos.ga_len) {
            backpos_T *p = GA_APPEND_VIA_PTR(backpos_T, &backpos);
            p->bp_scan = scan;
          } else if (reg_save_equal(&bp[i].bp_pos)) {
            // Still at same position as last time, fail.
            status = RA_NOMATCH;
          }

          assert(status != RA_FAIL);
          if (status != RA_NOMATCH) {
            reg_save(&bp[i].bp_pos, &backpos);
          }
        }
        break;

        case MOPEN + 0:     // Match start: \zs
        case MOPEN + 1:     // \(
        case MOPEN + 2:
        case MOPEN + 3:
        case MOPEN + 4:
        case MOPEN + 5:
        case MOPEN + 6:
        case MOPEN + 7:
        case MOPEN + 8:
        case MOPEN + 9:
          no = op - MOPEN;
          cleanup_subexpr();
          rp = regstack_push(RS_MOPEN, scan);
          if (rp == NULL) {
            status = RA_FAIL;
          } else {
            rp->rs_no = (int16_t)no;
            save_se(&rp->rs_un.sesave, &rex.reg_startpos[no],
                    &rex.reg_startp[no]);
            // We simply continue and handle the result when done.
          }
          break;

        case NOPEN:         // \%(
        case NCLOSE:        // \) after \%(
          if (regstack_push(RS_NOPEN, scan) == NULL) {
            status = RA_FAIL;
          }
          // We simply continue and handle the result when done.
          break;

        case ZOPEN + 1:
        case ZOPEN + 2:
        case ZOPEN + 3:
        case ZOPEN + 4:
        case ZOPEN + 5:
        case ZOPEN + 6:
        case ZOPEN + 7:
        case ZOPEN + 8:
        case ZOPEN + 9:
          no = op - ZOPEN;
          cleanup_zsubexpr();
          rp = regstack_push(RS_ZOPEN, scan);
          if (rp == NULL) {
            status = RA_FAIL;
          } else {
            rp->rs_no = (int16_t)no;
            save_se(&rp->rs_un.sesave, &reg_startzpos[no],
                    &reg_startzp[no]);
            // We simply continue and handle the result when done.
          }
          break;

        case MCLOSE + 0:    // Match end: \ze
        case MCLOSE + 1:    // \)
        case MCLOSE + 2:
        case MCLOSE + 3:
        case MCLOSE + 4:
        case MCLOSE + 5:
        case MCLOSE + 6:
        case MCLOSE + 7:
        case MCLOSE + 8:
        case MCLOSE + 9:
          no = op - MCLOSE;
          cleanup_subexpr();
          rp = regstack_push(RS_MCLOSE, scan);
          if (rp == NULL) {
            status = RA_FAIL;
          } else {
            rp->rs_no = (int16_t)no;
            save_se(&rp->rs_un.sesave, &rex.reg_endpos[no], &rex.reg_endp[no]);
            // We simply continue and handle the result when done.
          }
          break;

        case ZCLOSE + 1:    // \) after \z(
        case ZCLOSE + 2:
        case ZCLOSE + 3:
        case ZCLOSE + 4:
        case ZCLOSE + 5:
        case ZCLOSE + 6:
        case ZCLOSE + 7:
        case ZCLOSE + 8:
        case ZCLOSE + 9:
          no = op - ZCLOSE;
          cleanup_zsubexpr();
          rp = regstack_push(RS_ZCLOSE, scan);
          if (rp == NULL) {
            status = RA_FAIL;
          } else {
            rp->rs_no = (int16_t)no;
            save_se(&rp->rs_un.sesave, &reg_endzpos[no],
                    &reg_endzp[no]);
            // We simply continue and handle the result when done.
          }
          break;

        case BACKREF + 1:
        case BACKREF + 2:
        case BACKREF + 3:
        case BACKREF + 4:
        case BACKREF + 5:
        case BACKREF + 6:
        case BACKREF + 7:
        case BACKREF + 8:
        case BACKREF + 9: {
          int len;

          no = op - BACKREF;
          cleanup_subexpr();
          if (!REG_MULTI) {  // Single-line regexp
            if (rex.reg_startp[no] == NULL || rex.reg_endp[no] == NULL) {
              // Backref was not set: Match an empty string.
              len = 0;
            } else {
              // Compare current input with back-ref in the same line.
              len = (int)(rex.reg_endp[no] - rex.reg_startp[no]);
              if (cstrncmp((char *)rex.reg_startp[no], (char *)rex.input, &len) != 0) {
                status = RA_NOMATCH;
              }
            }
          } else {  // Multi-line regexp
            if (rex.reg_startpos[no].lnum < 0 || rex.reg_endpos[no].lnum < 0) {
              // Backref was not set: Match an empty string.
              len = 0;
            } else {
              if (rex.reg_startpos[no].lnum == rex.lnum
                  && rex.reg_endpos[no].lnum == rex.lnum) {
                // Compare back-ref within the current line.
                len = rex.reg_endpos[no].col - rex.reg_startpos[no].col;
                if (cstrncmp((char *)rex.line + rex.reg_startpos[no].col,
                             (char *)rex.input, &len) != 0) {
                  status = RA_NOMATCH;
                }
              } else {
                // Messy situation: Need to compare between two lines.
                int r = match_with_backref(rex.reg_startpos[no].lnum,
                                           rex.reg_startpos[no].col,
                                           rex.reg_endpos[no].lnum,
                                           rex.reg_endpos[no].col,
                                           &len);
                if (r != RA_MATCH) {
                  status = r;
                }
              }
            }
          }

          // Matched the backref, skip over it.
          rex.input += len;
        }
        break;

        case ZREF + 1:
        case ZREF + 2:
        case ZREF + 3:
        case ZREF + 4:
        case ZREF + 5:
        case ZREF + 6:
        case ZREF + 7:
        case ZREF + 8:
        case ZREF + 9:
          cleanup_zsubexpr();
          no = op - ZREF;
          if (re_extmatch_in != NULL
              && re_extmatch_in->matches[no] != NULL) {
            int len = (int)strlen((char *)re_extmatch_in->matches[no]);
            if (cstrncmp((char *)re_extmatch_in->matches[no], (char *)rex.input, &len) != 0) {
              status = RA_NOMATCH;
            } else {
              rex.input += len;
            }
          } else {
            // Backref was not set: Match an empty string.
          }
          break;

        case BRANCH:
          if (OP(next) != BRANCH) {     // No choice.
            next = OPERAND(scan);               // Avoid recursion.
          } else {
            rp = regstack_push(RS_BRANCH, scan);
            if (rp == NULL) {
              status = RA_FAIL;
            } else {
              status = RA_BREAK;                // rest is below
            }
          }
          break;

        case BRACE_LIMITS:
          if (OP(next) == BRACE_SIMPLE) {
            bl_minval = OPERAND_MIN(scan);
            bl_maxval = OPERAND_MAX(scan);
          } else if (OP(next) >= BRACE_COMPLEX
                     && OP(next) < BRACE_COMPLEX + 10) {
            no = OP(next) - BRACE_COMPLEX;
            brace_min[no] = OPERAND_MIN(scan);
            brace_max[no] = OPERAND_MAX(scan);
            brace_count[no] = 0;
          } else {
            internal_error("BRACE_LIMITS");
            status = RA_FAIL;
          }
          break;

        case BRACE_COMPLEX + 0:
        case BRACE_COMPLEX + 1:
        case BRACE_COMPLEX + 2:
        case BRACE_COMPLEX + 3:
        case BRACE_COMPLEX + 4:
        case BRACE_COMPLEX + 5:
        case BRACE_COMPLEX + 6:
        case BRACE_COMPLEX + 7:
        case BRACE_COMPLEX + 8:
        case BRACE_COMPLEX + 9:
          no = op - BRACE_COMPLEX;
          brace_count[no]++;

          // If not matched enough times yet, try one more
          if (brace_count[no] <= (brace_min[no] <= brace_max[no]
                                  ? brace_min[no] : brace_max[no])) {
            rp = regstack_push(RS_BRCPLX_MORE, scan);
            if (rp == NULL) {
              status = RA_FAIL;
            } else {
              rp->rs_no = (int16_t)no;
              reg_save(&rp->rs_un.regsave, &backpos);
              next = OPERAND(scan);
              // We continue and handle the result when done.
            }
            break;
          }

          // If matched enough times, may try matching some more
          if (brace_min[no] <= brace_max[no]) {
            // Range is the normal way around, use longest match
            if (brace_count[no] <= brace_max[no]) {
              rp = regstack_push(RS_BRCPLX_LONG, scan);
              if (rp == NULL) {
                status = RA_FAIL;
              } else {
                rp->rs_no = (int16_t)no;
                reg_save(&rp->rs_un.regsave, &backpos);
                next = OPERAND(scan);
                // We continue and handle the result when done.
              }
            }
          } else {
            // Range is backwards, use shortest match first
            if (brace_count[no] <= brace_min[no]) {
              rp = regstack_push(RS_BRCPLX_SHORT, scan);
              if (rp == NULL) {
                status = RA_FAIL;
              } else {
                reg_save(&rp->rs_un.regsave, &backpos);
                // We continue and handle the result when done.
              }
            }
          }
          break;

        case BRACE_SIMPLE:
        case STAR:
        case PLUS: {
          regstar_T rst;

          // Lookahead to avoid useless match attempts when we know
          // what character comes next.
          if (OP(next) == EXACTLY) {
            rst.nextb = *OPERAND(next);
            if (rex.reg_ic) {
              if (mb_isupper(rst.nextb)) {
                rst.nextb_ic = mb_tolower(rst.nextb);
              } else {
                rst.nextb_ic = mb_toupper(rst.nextb);
              }
            } else {
              rst.nextb_ic = rst.nextb;
            }
          } else {
            rst.nextb = NUL;
            rst.nextb_ic = NUL;
          }
          if (op != BRACE_SIMPLE) {
            rst.minval = (op == STAR) ? 0 : 1;
            rst.maxval = MAX_LIMIT;
          } else {
            rst.minval = bl_minval;
            rst.maxval = bl_maxval;
          }

          // When maxval > minval, try matching as much as possible, up
          // to maxval.  When maxval < minval, try matching at least the
          // minimal number (since the range is backwards, that's also
          // maxval!).
          rst.count = regrepeat(OPERAND(scan), rst.maxval);
          if (got_int) {
            status = RA_FAIL;
            break;
          }
          if (rst.minval <= rst.maxval
              ? rst.count >= rst.minval : rst.count >= rst.maxval) {
            // It could match.  Prepare for trying to match what
            // follows.  The code is below.  Parameters are stored in
            // a regstar_T on the regstack.
            if ((int64_t)((unsigned)regstack.ga_len >> 10) >= p_mmp) {
              emsg(_(e_pattern_uses_more_memory_than_maxmempattern));
              status = RA_FAIL;
            } else {
              ga_grow(&regstack, sizeof(regstar_T));
              regstack.ga_len += (int)sizeof(regstar_T);
              rp = regstack_push(rst.minval <= rst.maxval ? RS_STAR_LONG : RS_STAR_SHORT, scan);
              if (rp == NULL) {
                status = RA_FAIL;
              } else {
                *(((regstar_T *)rp) - 1) = rst;
                status = RA_BREAK;                  // skip the restore bits
              }
            }
          } else {
            status = RA_NOMATCH;
          }
        }
        break;

        case NOMATCH:
        case MATCH:
        case SUBPAT:
          rp = regstack_push(RS_NOMATCH, scan);
          if (rp == NULL) {
            status = RA_FAIL;
          } else {
            rp->rs_no = (int16_t)op;
            reg_save(&rp->rs_un.regsave, &backpos);
            next = OPERAND(scan);
            // We continue and handle the result when done.
          }
          break;

        case BEHIND:
        case NOBEHIND:
          // Need a bit of room to store extra positions.
          if ((int64_t)((unsigned)regstack.ga_len >> 10) >= p_mmp) {
            emsg(_(e_pattern_uses_more_memory_than_maxmempattern));
            status = RA_FAIL;
          } else {
            ga_grow(&regstack, sizeof(regbehind_T));
            regstack.ga_len += (int)sizeof(regbehind_T);
            rp = regstack_push(RS_BEHIND1, scan);
            if (rp == NULL) {
              status = RA_FAIL;
            } else {
              // Need to save the subexpr to be able to restore them
              // when there is a match but we don't use it.
              save_subexpr(((regbehind_T *)rp) - 1);

              rp->rs_no = (int16_t)op;
              reg_save(&rp->rs_un.regsave, &backpos);
              // First try if what follows matches.  If it does then we
              // check the behind match by looping.
            }
          }
          break;

        case BHPOS:
          if (REG_MULTI) {
            if (behind_pos.rs_u.pos.col != (colnr_T)(rex.input - rex.line)
                || behind_pos.rs_u.pos.lnum != rex.lnum) {
              status = RA_NOMATCH;
            }
          } else if (behind_pos.rs_u.ptr != rex.input) {
            status = RA_NOMATCH;
          }
          break;

        case NEWL:
          if ((c != NUL || !REG_MULTI || rex.lnum > rex.reg_maxline
               || rex.reg_line_lbr) && (c != '\n' || !rex.reg_line_lbr)) {
            status = RA_NOMATCH;
          } else if (rex.reg_line_lbr) {
            ADVANCE_REGINPUT();
          } else {
            reg_nextline();
          }
          break;

        case END:
          status = RA_MATCH;    // Success!
          break;

        default:
          iemsg(_(e_re_corr));
#ifdef REGEXP_DEBUG
          printf("Illegal op code %d\n", op);
#endif
          status = RA_FAIL;
          break;
        }
      }

      // If we can't continue sequentially, break the inner loop.
      if (status != RA_CONT) {
        break;
      }

      // Continue in inner loop, advance to next item.
      scan = next;
    }  // end of inner loop

    // If there is something on the regstack execute the code for the state.
    // If the state is popped then loop and use the older state.
    while (!GA_EMPTY(&regstack) && status != RA_FAIL) {
      rp = (regitem_T *)((char *)regstack.ga_data + regstack.ga_len) - 1;
      switch (rp->rs_state) {
      case RS_NOPEN:
        // Result is passed on as-is, simply pop the state.
        regstack_pop(&scan);
        break;

      case RS_MOPEN:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          restore_se(&rp->rs_un.sesave, &rex.reg_startpos[rp->rs_no],
                     &rex.reg_startp[rp->rs_no]);
        }
        regstack_pop(&scan);
        break;

      case RS_ZOPEN:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          restore_se(&rp->rs_un.sesave, &reg_startzpos[rp->rs_no],
                     &reg_startzp[rp->rs_no]);
        }
        regstack_pop(&scan);
        break;

      case RS_MCLOSE:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          restore_se(&rp->rs_un.sesave, &rex.reg_endpos[rp->rs_no],
                     &rex.reg_endp[rp->rs_no]);
        }
        regstack_pop(&scan);
        break;

      case RS_ZCLOSE:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          restore_se(&rp->rs_un.sesave, &reg_endzpos[rp->rs_no],
                     &reg_endzp[rp->rs_no]);
        }
        regstack_pop(&scan);
        break;

      case RS_BRANCH:
        if (status == RA_MATCH) {
          // this branch matched, use it
          regstack_pop(&scan);
        } else {
          if (status != RA_BREAK) {
            // After a non-matching branch: try next one.
            reg_restore(&rp->rs_un.regsave, &backpos);
            scan = rp->rs_scan;
          }
          if (scan == NULL || OP(scan) != BRANCH) {
            // no more branches, didn't find a match
            status = RA_NOMATCH;
            regstack_pop(&scan);
          } else {
            // Prepare to try a branch.
            rp->rs_scan = regnext(scan);
            reg_save(&rp->rs_un.regsave, &backpos);
            scan = OPERAND(scan);
          }
        }
        break;

      case RS_BRCPLX_MORE:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          reg_restore(&rp->rs_un.regsave, &backpos);
          brace_count[rp->rs_no]--;             // decrement match count
        }
        regstack_pop(&scan);
        break;

      case RS_BRCPLX_LONG:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          // There was no match, but we did find enough matches.
          reg_restore(&rp->rs_un.regsave, &backpos);
          brace_count[rp->rs_no]--;
          // continue with the items after "\{}"
          status = RA_CONT;
        }
        regstack_pop(&scan);
        if (status == RA_CONT) {
          scan = regnext(scan);
        }
        break;

      case RS_BRCPLX_SHORT:
        // Pop the state.  Restore pointers when there is no match.
        if (status == RA_NOMATCH) {
          // There was no match, try to match one more item.
          reg_restore(&rp->rs_un.regsave, &backpos);
        }
        regstack_pop(&scan);
        if (status == RA_NOMATCH) {
          scan = OPERAND(scan);
          status = RA_CONT;
        }
        break;

      case RS_NOMATCH:
        // Pop the state.  If the operand matches for NOMATCH or
        // doesn't match for MATCH/SUBPAT, we fail.  Otherwise backup,
        // except for SUBPAT, and continue with the next item.
        if (status == (rp->rs_no == NOMATCH ? RA_MATCH : RA_NOMATCH)) {
          status = RA_NOMATCH;
        } else {
          status = RA_CONT;
          if (rp->rs_no != SUBPAT) {            // zero-width
            reg_restore(&rp->rs_un.regsave, &backpos);
          }
        }
        regstack_pop(&scan);
        if (status == RA_CONT) {
          scan = regnext(scan);
        }
        break;

      case RS_BEHIND1:
        if (status == RA_NOMATCH) {
          regstack_pop(&scan);
          regstack.ga_len -= (int)sizeof(regbehind_T);
        } else {
          // The stuff after BEHIND/NOBEHIND matches.  Now try if
          // the behind part does (not) match before the current
          // position in the input.  This must be done at every
          // position in the input and checking if the match ends at
          // the current position.

          // save the position after the found match for next
          reg_save(&(((regbehind_T *)rp) - 1)->save_after, &backpos);

          // Start looking for a match with operand at the current
          // position.  Go back one character until we find the
          // result, hitting the start of the line or the previous
          // line (for multi-line matching).
          // Set behind_pos to where the match should end, BHPOS
          // will match it.  Save the current value.
          (((regbehind_T *)rp) - 1)->save_behind = behind_pos;
          behind_pos = rp->rs_un.regsave;

          rp->rs_state = RS_BEHIND2;

          reg_restore(&rp->rs_un.regsave, &backpos);
          scan = OPERAND(rp->rs_scan) + 4;
        }
        break;

      case RS_BEHIND2:
        // Looping for BEHIND / NOBEHIND match.
        if (status == RA_MATCH && reg_save_equal(&behind_pos)) {
          // found a match that ends where "next" started
          behind_pos = (((regbehind_T *)rp) - 1)->save_behind;
          if (rp->rs_no == BEHIND) {
            reg_restore(&(((regbehind_T *)rp) - 1)->save_after,
                        &backpos);
          } else {
            // But we didn't want a match.  Need to restore the
            // subexpr, because what follows matched, so they have
            // been set.
            status = RA_NOMATCH;
            restore_subexpr(((regbehind_T *)rp) - 1);
          }
          regstack_pop(&scan);
          regstack.ga_len -= (int)sizeof(regbehind_T);
        } else {
          int64_t limit;

          // No match or a match that doesn't end where we want it: Go
          // back one character.  May go to previous line once.
          no = OK;
          limit = OPERAND_MIN(rp->rs_scan);
          if (REG_MULTI) {
            if (limit > 0
                && ((rp->rs_un.regsave.rs_u.pos.lnum
                     < behind_pos.rs_u.pos.lnum
                     ? (colnr_T)strlen((char *)rex.line)
                     : behind_pos.rs_u.pos.col)
                    - rp->rs_un.regsave.rs_u.pos.col >= limit)) {
              no = FAIL;
            } else if (rp->rs_un.regsave.rs_u.pos.col == 0) {
              if (rp->rs_un.regsave.rs_u.pos.lnum
                  < behind_pos.rs_u.pos.lnum
                  || reg_getline(--rp->rs_un.regsave.rs_u.pos.lnum)
                  == NULL) {
                no = FAIL;
              } else {
                reg_restore(&rp->rs_un.regsave, &backpos);
                rp->rs_un.regsave.rs_u.pos.col =
                  (colnr_T)strlen((char *)rex.line);
              }
            } else {
              const uint8_t *const line =
                (uint8_t *)reg_getline(rp->rs_un.regsave.rs_u.pos.lnum);

              rp->rs_un.regsave.rs_u.pos.col -=
                utf_head_off((char *)line,
                             (char *)line + rp->rs_un.regsave.rs_u.pos.col - 1)
                + 1;
            }
          } else {
            if (rp->rs_un.regsave.rs_u.ptr == rex.line) {
              no = FAIL;
            } else {
              MB_PTR_BACK(rex.line, rp->rs_un.regsave.rs_u.ptr);
              if (limit > 0
                  && (behind_pos.rs_u.ptr - rp->rs_un.regsave.rs_u.ptr) > (ptrdiff_t)limit) {
                no = FAIL;
              }
            }
          }
          if (no == OK) {
            // Advanced, prepare for finding match again.
            reg_restore(&rp->rs_un.regsave, &backpos);
            scan = OPERAND(rp->rs_scan) + 4;
            if (status == RA_MATCH) {
              // We did match, so subexpr may have been changed,
              // need to restore them for the next try.
              status = RA_NOMATCH;
              restore_subexpr(((regbehind_T *)rp) - 1);
            }
          } else {
            // Can't advance.  For NOBEHIND that's a match.
            behind_pos = (((regbehind_T *)rp) - 1)->save_behind;
            if (rp->rs_no == NOBEHIND) {
              reg_restore(&(((regbehind_T *)rp) - 1)->save_after,
                          &backpos);
              status = RA_MATCH;
            } else {
              // We do want a proper match.  Need to restore the
              // subexpr if we had a match, because they may have
              // been set.
              if (status == RA_MATCH) {
                status = RA_NOMATCH;
                restore_subexpr(((regbehind_T *)rp) - 1);
              }
            }
            regstack_pop(&scan);
            regstack.ga_len -= (int)sizeof(regbehind_T);
          }
        }
        break;

      case RS_STAR_LONG:
      case RS_STAR_SHORT: {
        regstar_T *rst = ((regstar_T *)rp) - 1;

        if (status == RA_MATCH) {
          regstack_pop(&scan);
          regstack.ga_len -= (int)sizeof(regstar_T);
          break;
        }

        // Tried once already, restore input pointers.
        if (status != RA_BREAK) {
          reg_restore(&rp->rs_un.regsave, &backpos);
        }

        // Repeat until we found a position where it could match.
        while (true) {
          if (status != RA_BREAK) {
            // Tried first position already, advance.
            if (rp->rs_state == RS_STAR_LONG) {
              // Trying for longest match, but couldn't or
              // didn't match -- back up one char.
              if (--rst->count < rst->minval) {
                break;
              }
              if (rex.input == rex.line) {
                // backup to last char of previous line
                if (rex.lnum == 0) {
                  status = RA_NOMATCH;
                  break;
                }
                rex.lnum--;
                rex.line = (uint8_t *)reg_getline(rex.lnum);
                // Just in case regrepeat() didn't count right.
                if (rex.line == NULL) {
                  break;
                }
                rex.input = rex.line + reg_getline_len(rex.lnum);
                reg_breakcheck();
              } else {
                MB_PTR_BACK(rex.line, rex.input);
              }
            } else {
              // Range is backwards, use shortest match first.
              // Careful: maxval and minval are exchanged!
              // Couldn't or didn't match: try advancing one
              // char.
              if (rst->count == rst->minval
                  || regrepeat(OPERAND(rp->rs_scan), 1L) == 0) {
                break;
              }
              rst->count++;
            }
            if (got_int) {
              break;
            }
          } else {
            status = RA_NOMATCH;
          }

          // If it could match, try it.
          if (rst->nextb == NUL || *rex.input == rst->nextb
              || *rex.input == rst->nextb_ic) {
            reg_save(&rp->rs_un.regsave, &backpos);
            scan = regnext(rp->rs_scan);
            status = RA_CONT;
            break;
          }
        }
        if (status != RA_CONT) {
          // Failed.
          regstack_pop(&scan);
          regstack.ga_len -= (int)sizeof(regstar_T);
          status = RA_NOMATCH;
        }
      }
      break;
      }

      // If we want to continue the inner loop or didn't pop a state
      // continue matching loop
      if (status == RA_CONT || rp == (regitem_T *)
          ((char *)regstack.ga_data + regstack.ga_len) - 1) {
        break;
      }
    }

    // May need to continue with the inner loop, starting at "scan".
    if (status == RA_CONT) {
      continue;
    }

    // If the regstack is empty or something failed we are done.
    if (GA_EMPTY(&regstack) || status == RA_FAIL) {
      if (scan == NULL) {
        // We get here only if there's trouble -- normally "case END" is
        // the terminating point.
        iemsg(_(e_re_corr));
#ifdef REGEXP_DEBUG
        printf("Premature EOL\n");
#endif
      }
      return status == RA_MATCH;
    }
  }  // End of loop until the regstack is empty.

  // NOTREACHED
}

/// Try match of "prog" with at rex.line["col"].
///
/// @param tm         timeout limit or NULL
/// @param timed_out  flag set on timeout or NULL
///
/// @return  0 for failure, or number of lines contained in the match.
static int regtry(bt_regprog_T *prog, colnr_T col, proftime_T *tm, int *timed_out)
{
  rex.input = rex.line + col;
  rex.need_clear_subexpr = true;
  // Clear the external match subpointers if necessaey.
  rex.need_clear_zsubexpr = (prog->reghasz == REX_SET);

  if (regmatch(&prog->program[1], tm, timed_out) == 0) {
    return 0;
  }

  cleanup_subexpr();
  if (REG_MULTI) {
    if (rex.reg_startpos[0].lnum < 0) {
      rex.reg_startpos[0].lnum = 0;
      rex.reg_startpos[0].col = col;
    }
    if (rex.reg_endpos[0].lnum < 0) {
      rex.reg_endpos[0].lnum = rex.lnum;
      rex.reg_endpos[0].col = (int)(rex.input - rex.line);
    } else {
      // Use line number of "\ze".
      rex.lnum = rex.reg_endpos[0].lnum;
    }
  } else {
    if (rex.reg_startp[0] == NULL) {
      rex.reg_startp[0] = rex.line + col;
    }
    if (rex.reg_endp[0] == NULL) {
      rex.reg_endp[0] = rex.input;
    }
  }
  // Package any found \z(...\) matches for export. Default is none.
  unref_extmatch(re_extmatch_out);
  re_extmatch_out = NULL;

  if (prog->reghasz == REX_SET) {
    int i;

    cleanup_zsubexpr();
    re_extmatch_out = make_extmatch();
    for (i = 0; i < NSUBEXP; i++) {
      if (REG_MULTI) {
        // Only accept single line matches.
        if (reg_startzpos[i].lnum >= 0
            && reg_endzpos[i].lnum == reg_startzpos[i].lnum
            && reg_endzpos[i].col >= reg_startzpos[i].col) {
          re_extmatch_out->matches[i] =
            (uint8_t *)xstrnsave(reg_getline(reg_startzpos[i].lnum) + reg_startzpos[i].col,
                                 (size_t)(reg_endzpos[i].col - reg_startzpos[i].col));
        }
      } else {
        if (reg_startzp[i] != NULL && reg_endzp[i] != NULL) {
          re_extmatch_out->matches[i] =
            (uint8_t *)xstrnsave((char *)reg_startzp[i], (size_t)(reg_endzp[i] - reg_startzp[i]));
        }
      }
    }
  }
  return 1 + rex.lnum;
}

/// Match a regexp against a string ("line" points to the string) or multiple
/// lines (if "line" is NULL, use reg_getline()).
///
/// @param startcol   column to start looking for match
/// @param tm         timeout limit or NULL
/// @param timed_out  flag set on timeout or NULL
///
/// @return  0 for failure, or number of lines contained in the match.
static int bt_regexec_both(uint8_t *line, colnr_T startcol, proftime_T *tm, int *timed_out)
{
  bt_regprog_T *prog;
  uint8_t *s;
  colnr_T col = startcol;
  int retval = 0;

  // Create "regstack" and "backpos" if they are not allocated yet.
  // We allocate *_INITIAL amount of bytes first and then set the grow size
  // to much bigger value to avoid many malloc calls in case of deep regular
  // expressions.
  if (regstack.ga_data == NULL) {
    // Use an item size of 1 byte, since we push different things
    // onto the regstack.
    ga_init(&regstack, 1, REGSTACK_INITIAL);
    ga_grow(&regstack, REGSTACK_INITIAL);
    ga_set_growsize(&regstack, REGSTACK_INITIAL * 8);
  }

  if (backpos.ga_data == NULL) {
    ga_init(&backpos, sizeof(backpos_T), BACKPOS_INITIAL);
    ga_grow(&backpos, BACKPOS_INITIAL);
    ga_set_growsize(&backpos, BACKPOS_INITIAL * 8);
  }

  if (REG_MULTI) {
    prog = (bt_regprog_T *)rex.reg_mmatch->regprog;
    line = (uint8_t *)reg_getline(0);
    rex.reg_startpos = rex.reg_mmatch->startpos;
    rex.reg_endpos = rex.reg_mmatch->endpos;
  } else {
    prog = (bt_regprog_T *)rex.reg_match->regprog;
    rex.reg_startp = (uint8_t **)rex.reg_match->startp;
    rex.reg_endp = (uint8_t **)rex.reg_match->endp;
  }

  // Be paranoid...
  if (prog == NULL || line == NULL) {
    iemsg(_(e_null));
    goto theend;
  }

  // Check validity of program.
  if (prog_magic_wrong()) {
    goto theend;
  }

  // If the start column is past the maximum column: no need to try.
  if (rex.reg_maxcol > 0 && col >= rex.reg_maxcol) {
    goto theend;
  }

  // If pattern contains "\c" or "\C": overrule value of rex.reg_ic
  if (prog->regflags & RF_ICASE) {
    rex.reg_ic = true;
  } else if (prog->regflags & RF_NOICASE) {
    rex.reg_ic = false;
  }

  // If pattern contains "\Z" overrule value of rex.reg_icombine
  if (prog->regflags & RF_ICOMBINE) {
    rex.reg_icombine = true;
  }

  // If there is a "must appear" string, look for it.
  if (prog->regmust != NULL) {
    int c = utf_ptr2char((char *)prog->regmust);
    s = line + col;

    // This is used very often, esp. for ":global".  Use two versions of
    // the loop to avoid overhead of conditions.
    if (!rex.reg_ic) {
      while ((s = (uint8_t *)vim_strchr((char *)s, c)) != NULL) {
        if (cstrncmp((char *)s, (char *)prog->regmust, &prog->regmlen) == 0) {
          break;  // Found it.
        }
        MB_PTR_ADV(s);
      }
    } else {
      while ((s = (uint8_t *)cstrchr((char *)s, c)) != NULL) {
        if (cstrncmp((char *)s, (char *)prog->regmust, &prog->regmlen) == 0) {
          break;  // Found it.
        }
        MB_PTR_ADV(s);
      }
    }
    if (s == NULL) {  // Not present.
      goto theend;
    }
  }

  rex.line = line;
  rex.lnum = 0;
  reg_toolong = false;

  // Simplest case: Anchored match need be tried only once.
  if (prog->reganch) {
    int c = utf_ptr2char((char *)rex.line + col);
    if (prog->regstart == NUL
        || prog->regstart == c
        || (rex.reg_ic
            && (utf_fold(prog->regstart) == utf_fold(c)
                || (c < 255 && prog->regstart < 255
                    && mb_tolower(prog->regstart) == mb_tolower(c))))) {
      retval = regtry(prog, col, tm, timed_out);
    } else {
      retval = 0;
    }
  } else {
    int tm_count = 0;
    // Messy cases:  unanchored match.
    while (!got_int) {
      if (prog->regstart != NUL) {
        // Skip until the char we know it must start with.
        s = (uint8_t *)cstrchr((char *)rex.line + col, prog->regstart);
        if (s == NULL) {
          retval = 0;
          break;
        }
        col = (int)(s - rex.line);
      }

      // Check for maximum column to try.
      if (rex.reg_maxcol > 0 && col >= rex.reg_maxcol) {
        retval = 0;
        break;
      }

      retval = regtry(prog, col, tm, timed_out);
      if (retval > 0) {
        break;
      }

      // if not currently on the first line, get it again
      if (rex.lnum != 0) {
        rex.lnum = 0;
        rex.line = (uint8_t *)reg_getline(0);
      }
      if (rex.line[col] == NUL) {
        break;
      }
      col += utfc_ptr2len((char *)rex.line + col);
      // Check for timeout once in a twenty times to avoid overhead.
      if (tm != NULL && ++tm_count == 20) {
        tm_count = 0;
        if (profile_passed_limit(*tm)) {
          if (timed_out != NULL) {
            *timed_out = true;
          }
          break;
        }
      }
    }
  }

theend:
  // Free "reg_tofree" when it's a bit big.
  // Free regstack and backpos if they are bigger than their initial size.
  if (reg_tofreelen > 400) {
    XFREE_CLEAR(reg_tofree);
  }
  if (regstack.ga_maxlen > REGSTACK_INITIAL) {
    ga_clear(&regstack);
  }
  if (backpos.ga_maxlen > BACKPOS_INITIAL) {
    ga_clear(&backpos);
  }

  if (retval > 0) {
    // Make sure the end is never before the start.  Can happen when \zs
    // and \ze are used.
    if (REG_MULTI) {
      const lpos_T *const start = &rex.reg_mmatch->startpos[0];
      const lpos_T *const end = &rex.reg_mmatch->endpos[0];

      if (end->lnum < start->lnum
          || (end->lnum == start->lnum && end->col < start->col)) {
        rex.reg_mmatch->endpos[0] = rex.reg_mmatch->startpos[0];
      }

      // startpos[0] may be set by "\zs", also return the column where
      // the whole pattern matched.
      rex.reg_mmatch->rmm_matchcol = col;
    } else {
      if (rex.reg_match->endp[0] < rex.reg_match->startp[0]) {
        rex.reg_match->endp[0] = rex.reg_match->startp[0];
      }

      // startpos[0] may be set by "\zs", also return the column where
      // the whole pattern matched.
      rex.reg_match->rm_matchcol = col;
    }
  }

  return retval;
}

/// Match a regexp against a string.
/// "rmp->regprog" is a compiled regexp as returned by vim_regcomp().
/// Uses curbuf for line count and 'iskeyword'.
/// If "line_lbr" is true, consider a "\n" in "line" to be a line break.
///
/// @param line  string to match against
/// @param col   column to start looking for match
///
/// @return  0 for failure, number of lines contained in the match otherwise.
static int bt_regexec_nl(regmatch_T *rmp, uint8_t *line, colnr_T col, bool line_lbr)
{
  rex.reg_match = rmp;
  rex.reg_mmatch = NULL;
  rex.reg_maxline = 0;
  rex.reg_line_lbr = line_lbr;
  rex.reg_buf = curbuf;
  rex.reg_win = NULL;
  rex.reg_ic = rmp->rm_ic;
  rex.reg_icombine = false;
  rex.reg_nobreak = rmp->regprog->re_flags & RE_NOBREAK;
  rex.reg_maxcol = 0;

  int64_t r = bt_regexec_both(line, col, NULL, NULL);
  assert(r <= INT_MAX);
  return (int)r;
}

/// Matches a regexp against multiple lines.
/// "rmp->regprog" is a compiled regexp as returned by vim_regcomp().
/// Uses curbuf for line count and 'iskeyword'.
///
/// @param win Window in which to search or NULL
/// @param buf Buffer in which to search
/// @param lnum Number of line to start looking for match
/// @param col Column to start looking for match
/// @param tm Timeout limit or NULL
///
/// @return zero if there is no match and number of lines contained in the match
///         otherwise.
static int bt_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum, colnr_T col,
                            proftime_T *tm, int *timed_out)
{
  init_regexec_multi(rmp, win, buf, lnum);
  return bt_regexec_both(NULL, col, tm, timed_out);
}

// Rust implementation for re_num_cmp
extern int rs_re_num_cmp(uint32_t val, const uint8_t *scan);

// Compare a number with the operand of RE_LNUM, RE_COL or RE_VCOL.
static int re_num_cmp(uint32_t val, const uint8_t *scan)
{
  return rs_re_num_cmp(val, scan);
}

#ifdef BT_REGEXP_DUMP

// regdump - dump a regexp onto stdout in vaguely comprehensible form
static void regdump(uint8_t *pattern, bt_regprog_T *r)
{
  uint8_t *s;
  int op = EXACTLY;             // Arbitrary non-END op.
  uint8_t *next;
  uint8_t *end = NULL;
  FILE *f;

# ifdef BT_REGEXP_LOG
  f = fopen("bt_regexp_log.log", "a");
# else
  f = stdout;
# endif
  if (f == NULL) {
    return;
  }
  fprintf(f, "-------------------------------------\n\r\nregcomp(%s):\r\n",
          pattern);

  s = &r->program[1];
  // Loop until we find the END that isn't before a referred next (an END
  // can also appear in a NOMATCH operand).
  while (op != END || s <= end) {
    op = OP(s);
    fprintf(f, "%2d%s", (int)(s - r->program), regprop(s));     // Where, what.
    next = regnext(s);
    if (next == NULL) {         // Next ptr.
      fprintf(f, "(0)");
    } else {
      fprintf(f, "(%d)", (int)((s - r->program) + (next - s)));
    }
    if (end < next) {
      end = next;
    }
    if (op == BRACE_LIMITS) {
      // Two ints
      fprintf(f, " minval %" PRId64 ", maxval %" PRId64,
              (int64_t)OPERAND_MIN(s), (int64_t)OPERAND_MAX(s));
      s += 8;
    } else if (op == BEHIND || op == NOBEHIND) {
      // one int
      fprintf(f, " count %" PRId64, (int64_t)OPERAND_MIN(s));
      s += 4;
    } else if (op == RE_LNUM || op == RE_COL || op == RE_VCOL) {
      // one int plus comparator
      fprintf(f, " count %" PRId64, (int64_t)OPERAND_MIN(s));
      s += 5;
    }
    s += 3;
    if (op == ANYOF || op == ANYOF + ADD_NL
        || op == ANYBUT || op == ANYBUT + ADD_NL
        || op == EXACTLY) {
      // Literal string, where present.
      fprintf(f, "\nxxxxxxxxx\n");
      while (*s != NUL) {
        fprintf(f, "%c", *s++);
      }
      fprintf(f, "\nxxxxxxxxx\n");
      s++;
    }
    fprintf(f, "\r\n");
  }

  // Header fields of interest.
  if (r->regstart != NUL) {
    fprintf(f, "start `%s' 0x%x; ", r->regstart < 256
            ? (char *)transchar(r->regstart)
            : "multibyte", r->regstart);
  }
  if (r->reganch) {
    fprintf(f, "anchored; ");
  }
  if (r->regmust != NULL) {
    fprintf(f, "must have \"%s\"", r->regmust);
  }
  fprintf(f, "\r\n");

# ifdef BT_REGEXP_LOG
  fclose(f);
# endif
}
#endif      // BT_REGEXP_DUMP

#ifdef REGEXP_DEBUG

// regprop - printable representation of opcode
static uint8_t *regprop(uint8_t *op)
{
  char *p;
  static char buf[50];
  static size_t buflen = 0;

  STRCPY(buf, ":");
  buflen = 1;

  switch ((int)OP(op)) {
  case BOL:
    p = "BOL";
    break;
  case EOL:
    p = "EOL";
    break;
  case RE_BOF:
    p = "BOF";
    break;
  case RE_EOF:
    p = "EOF";
    break;
  case CURSOR:
    p = "CURSOR";
    break;
  case RE_VISUAL:
    p = "RE_VISUAL";
    break;
  case RE_LNUM:
    p = "RE_LNUM";
    break;
  case RE_MARK:
    p = "RE_MARK";
    break;
  case RE_COL:
    p = "RE_COL";
    break;
  case RE_VCOL:
    p = "RE_VCOL";
    break;
  case BOW:
    p = "BOW";
    break;
  case EOW:
    p = "EOW";
    break;
  case ANY:
    p = "ANY";
    break;
  case ANY + ADD_NL:
    p = "ANY+NL";
    break;
  case ANYOF:
    p = "ANYOF";
    break;
  case ANYOF + ADD_NL:
    p = "ANYOF+NL";
    break;
  case ANYBUT:
    p = "ANYBUT";
    break;
  case ANYBUT + ADD_NL:
    p = "ANYBUT+NL";
    break;
  case IDENT:
    p = "IDENT";
    break;
  case IDENT + ADD_NL:
    p = "IDENT+NL";
    break;
  case SIDENT:
    p = "SIDENT";
    break;
  case SIDENT + ADD_NL:
    p = "SIDENT+NL";
    break;
  case KWORD:
    p = "KWORD";
    break;
  case KWORD + ADD_NL:
    p = "KWORD+NL";
    break;
  case SKWORD:
    p = "SKWORD";
    break;
  case SKWORD + ADD_NL:
    p = "SKWORD+NL";
    break;
  case FNAME:
    p = "FNAME";
    break;
  case FNAME + ADD_NL:
    p = "FNAME+NL";
    break;
  case SFNAME:
    p = "SFNAME";
    break;
  case SFNAME + ADD_NL:
    p = "SFNAME+NL";
    break;
  case PRINT:
    p = "PRINT";
    break;
  case PRINT + ADD_NL:
    p = "PRINT+NL";
    break;
  case SPRINT:
    p = "SPRINT";
    break;
  case SPRINT + ADD_NL:
    p = "SPRINT+NL";
    break;
  case WHITE:
    p = "WHITE";
    break;
  case WHITE + ADD_NL:
    p = "WHITE+NL";
    break;
  case NWHITE:
    p = "NWHITE";
    break;
  case NWHITE + ADD_NL:
    p = "NWHITE+NL";
    break;
  case DIGIT:
    p = "DIGIT";
    break;
  case DIGIT + ADD_NL:
    p = "DIGIT+NL";
    break;
  case NDIGIT:
    p = "NDIGIT";
    break;
  case NDIGIT + ADD_NL:
    p = "NDIGIT+NL";
    break;
  case HEX:
    p = "HEX";
    break;
  case HEX + ADD_NL:
    p = "HEX+NL";
    break;
  case NHEX:
    p = "NHEX";
    break;
  case NHEX + ADD_NL:
    p = "NHEX+NL";
    break;
  case OCTAL:
    p = "OCTAL";
    break;
  case OCTAL + ADD_NL:
    p = "OCTAL+NL";
    break;
  case NOCTAL:
    p = "NOCTAL";
    break;
  case NOCTAL + ADD_NL:
    p = "NOCTAL+NL";
    break;
  case WORD:
    p = "WORD";
    break;
  case WORD + ADD_NL:
    p = "WORD+NL";
    break;
  case NWORD:
    p = "NWORD";
    break;
  case NWORD + ADD_NL:
    p = "NWORD+NL";
    break;
  case HEAD:
    p = "HEAD";
    break;
  case HEAD + ADD_NL:
    p = "HEAD+NL";
    break;
  case NHEAD:
    p = "NHEAD";
    break;
  case NHEAD + ADD_NL:
    p = "NHEAD+NL";
    break;
  case ALPHA:
    p = "ALPHA";
    break;
  case ALPHA + ADD_NL:
    p = "ALPHA+NL";
    break;
  case NALPHA:
    p = "NALPHA";
    break;
  case NALPHA + ADD_NL:
    p = "NALPHA+NL";
    break;
  case LOWER:
    p = "LOWER";
    break;
  case LOWER + ADD_NL:
    p = "LOWER+NL";
    break;
  case NLOWER:
    p = "NLOWER";
    break;
  case NLOWER + ADD_NL:
    p = "NLOWER+NL";
    break;
  case UPPER:
    p = "UPPER";
    break;
  case UPPER + ADD_NL:
    p = "UPPER+NL";
    break;
  case NUPPER:
    p = "NUPPER";
    break;
  case NUPPER + ADD_NL:
    p = "NUPPER+NL";
    break;
  case BRANCH:
    p = "BRANCH";
    break;
  case EXACTLY:
    p = "EXACTLY";
    break;
  case NOTHING:
    p = "NOTHING";
    break;
  case BACK:
    p = "BACK";
    break;
  case END:
    p = "END";
    break;
  case MOPEN + 0:
    p = "MATCH START";
    break;
  case MOPEN + 1:
  case MOPEN + 2:
  case MOPEN + 3:
  case MOPEN + 4:
  case MOPEN + 5:
  case MOPEN + 6:
  case MOPEN + 7:
  case MOPEN + 8:
  case MOPEN + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "MOPEN%d", OP(op) - MOPEN);
    p = NULL;
    break;
  case MCLOSE + 0:
    p = "MATCH END";
    break;
  case MCLOSE + 1:
  case MCLOSE + 2:
  case MCLOSE + 3:
  case MCLOSE + 4:
  case MCLOSE + 5:
  case MCLOSE + 6:
  case MCLOSE + 7:
  case MCLOSE + 8:
  case MCLOSE + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "MCLOSE%d", OP(op) - MCLOSE);
    p = NULL;
    break;
  case BACKREF + 1:
  case BACKREF + 2:
  case BACKREF + 3:
  case BACKREF + 4:
  case BACKREF + 5:
  case BACKREF + 6:
  case BACKREF + 7:
  case BACKREF + 8:
  case BACKREF + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "BACKREF%d", OP(op) - BACKREF);
    p = NULL;
    break;
  case NOPEN:
    p = "NOPEN";
    break;
  case NCLOSE:
    p = "NCLOSE";
    break;
  case ZOPEN + 1:
  case ZOPEN + 2:
  case ZOPEN + 3:
  case ZOPEN + 4:
  case ZOPEN + 5:
  case ZOPEN + 6:
  case ZOPEN + 7:
  case ZOPEN + 8:
  case ZOPEN + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "ZOPEN%d", OP(op) - ZOPEN);
    p = NULL;
    break;
  case ZCLOSE + 1:
  case ZCLOSE + 2:
  case ZCLOSE + 3:
  case ZCLOSE + 4:
  case ZCLOSE + 5:
  case ZCLOSE + 6:
  case ZCLOSE + 7:
  case ZCLOSE + 8:
  case ZCLOSE + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "ZCLOSE%d", OP(op) - ZCLOSE);
    p = NULL;
    break;
  case ZREF + 1:
  case ZREF + 2:
  case ZREF + 3:
  case ZREF + 4:
  case ZREF + 5:
  case ZREF + 6:
  case ZREF + 7:
  case ZREF + 8:
  case ZREF + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "ZREF%d", OP(op) - ZREF);
    p = NULL;
    break;
  case STAR:
    p = "STAR";
    break;
  case PLUS:
    p = "PLUS";
    break;
  case NOMATCH:
    p = "NOMATCH";
    break;
  case MATCH:
    p = "MATCH";
    break;
  case BEHIND:
    p = "BEHIND";
    break;
  case NOBEHIND:
    p = "NOBEHIND";
    break;
  case SUBPAT:
    p = "SUBPAT";
    break;
  case BRACE_LIMITS:
    p = "BRACE_LIMITS";
    break;
  case BRACE_SIMPLE:
    p = "BRACE_SIMPLE";
    break;
  case BRACE_COMPLEX + 0:
  case BRACE_COMPLEX + 1:
  case BRACE_COMPLEX + 2:
  case BRACE_COMPLEX + 3:
  case BRACE_COMPLEX + 4:
  case BRACE_COMPLEX + 5:
  case BRACE_COMPLEX + 6:
  case BRACE_COMPLEX + 7:
  case BRACE_COMPLEX + 8:
  case BRACE_COMPLEX + 9:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "BRACE_COMPLEX%d", OP(op) - BRACE_COMPLEX);
    p = NULL;
    break;
  case MULTIBYTECODE:
    p = "MULTIBYTECODE";
    break;
  case NEWL:
    p = "NEWL";
    break;
  default:
    buflen += (size_t)snprintf(buf + buflen, sizeof(buf) - buflen,
                               "corrupt %d", OP(op));
    p = NULL;
    break;
  }
  if (p != NULL) {
    STRCPY(buf + buflen, p);
  }
  return (uint8_t *)buf;
}
#endif      // REGEXP_DEBUG

// }}}1

// regexp_nfa.c {{{1
// NFA regular expression implementation.

// Logging of NFA engine.
//
// The NFA engine can write four log files:
// - Error log: Contains NFA engine's fatal errors.
// - Dump log: Contains compiled NFA state machine's information.
// - Run log: Contains information of matching procedure.
// - Debug log: Contains detailed information of matching procedure. Can be
//   disabled by undefining NFA_REGEXP_DEBUG_LOG.
// The first one can also be used without debug mode.
// The last three are enabled when compiled as debug mode and individually
// disabled by commenting them out.
// The log files can get quite big!
// To disable all of this when compiling Vim for debugging, undefine REGEXP_DEBUG in
// regexp.c
#ifdef REGEXP_DEBUG
# define NFA_REGEXP_ERROR_LOG   "nfa_regexp_error.log"
# define NFA_REGEXP_DUMP_LOG    "nfa_regexp_dump.log"
# define NFA_REGEXP_RUN_LOG     "nfa_regexp_run.log"
# define NFA_REGEXP_DEBUG_LOG   "nfa_regexp_debug.log"
#endif

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

// Variables only used in nfa_regcomp() and descendants.
static int nfa_re_flags;  ///< re_flags passed to nfa_regcomp().
static int *post_start;   ///< holds the postfix form of r.e.
static int *post_end;
static int *post_ptr;

// Set when the pattern should use the NFA engine.
// E.g. [[:upper:]] only allows 8bit characters for BT engine,
// while NFA engine handles multibyte characters correctly.
static bool wants_nfa;

static int nstate;  ///< Number of states in the NFA. Also used when executing.

// Global nstate accessor for Rust (Phase 12c)
void nvim_set_nstate(int v) { nstate = v; }

// If not NULL match must end at this position
static save_se_T *nfa_endp = NULL;

// 0 for first call to nfa_regmatch(), 1 for recursive call.
static int nfa_ll_index = 0;

// NFA accessors for Rust
int nvim_rex_get_nfa_ll_index(void) { return nfa_ll_index; }
void nvim_rex_set_nfa_ll_index(int v) { nfa_ll_index = v; }
void *nvim_rex_get_nfa_endp(void) { return nfa_endp; }
void nvim_rex_set_nfa_endp(void *p) { nfa_endp = p; }
int nvim_parse_get_nfa_re_flags(void) { return nfa_re_flags; }
void nvim_parse_set_nfa_re_flags(int f) { nfa_re_flags = f; }

// NFA execution globals accessors for Rust (Phase 1: nfa_regmatch migration)
int nvim_nfa_get_match(void);  // Forward declaration - defined after nfa_match
void nvim_nfa_set_match(int v);
proftime_T *nvim_nfa_get_time_limit(void);
void nvim_nfa_set_time_limit(proftime_T *p);
int *nvim_nfa_get_timed_out_ptr(void);
void nvim_nfa_set_timed_out_ptr(int *p);
int nvim_nfa_get_time_count(void);
void nvim_nfa_set_time_count(int v);
int nvim_nfa_did_time_out(void);

// Forward declarations for wrapper functions
static void copy_sub(regsub_T *to, regsub_T *from);
static int recursive_regmatch(nfa_state_T *state, nfa_pim_T *pim,
                              nfa_regprog_T *prog, regsubs_T *submatch,
                              regsubs_T *m, int **listids, int *listids_len);
static void reg_nextline(void);  // Forward declaration

// Wrapper for reg_getline_len for Rust
static colnr_T reg_getline_len(linenr_T lnum);  // Forward declaration
colnr_T nvim_reg_getline_len(linenr_T lnum) {
  return reg_getline_len(lnum);
}

// Forward declarations for listid save/restore
static void nfa_save_listids(nfa_regprog_T *prog, int *list);
static void nfa_restore_listids(nfa_regprog_T *prog, const int *list);

// Wrapper for nfa_save_listids for Rust
void nvim_nfa_save_listids(void *prog, int *list) {
  nfa_save_listids((nfa_regprog_T *)prog, list);
}

// Wrapper for nfa_restore_listids for Rust
void nvim_nfa_restore_listids(void *prog, const int *list) {
  nfa_restore_listids((nfa_regprog_T *)prog, list);
}

// Forward declaration for nfa_regmatch
static int nfa_regmatch(nfa_regprog_T *prog, nfa_state_T *start,
                        regsubs_T *submatch, regsubs_T *m);

// Wrapper for nfa_regmatch for Rust (calls into C main loop)
int nvim_nfa_regmatch(void *prog, void *start, void *submatch, void *m) {
  return nfa_regmatch((nfa_regprog_T *)prog, (nfa_state_T *)start,
                      (regsubs_T *)submatch, (regsubs_T *)m);
}

// Accessor for nfa_regprog_T.nstate for Rust
int nvim_nfa_prog_get_nstate(const void *prog) {
  return ((const nfa_regprog_T *)prog)->nstate;
}

// Forward declarations for backref functions
static int match_backref(regsub_T *sub, int subidx, int *bytelen);
static int match_zref(int subidx, int *bytelen);

// Wrapper for match_backref (static function) for Rust
// Returns match result (0/1), sets bytelen output parameter
int nvim_nfa_match_backref(void *sub, int subidx, int *bytelen) {
  return match_backref((regsub_T *)sub, subidx, bytelen);
}

// =============================================================================
// Invisible/lookaround wrappers for Rust (Phase 5.9)
// =============================================================================

/// Check NFA_END_INVISIBLE match.
/// Returns:
/// - 0: No match (break)
/// - 1: Match found, set nfa_match = true and goto nextchar
///
/// Parameters:
/// - state_c: The state code (NFA_END_INVISIBLE, NFA_END_INVISIBLE_NEG, NFA_END_PATTERN)
/// - t_subs: Thread submatches (regsubs_T*)
/// - m: Output match info (regsubs_T*)
/// - nextlist_n: Number of items in nextlist (unused, for future use)
/// - nfa_endp_ptr: The nfa_endp pointer (may be NULL)
int nvim_nfa_check_end_invisible(int state_c, const void *t_subs, void *m,
                                 int nextlist_n, const void *nfa_endp_ptr)
{
  (void)nextlist_n;  // Currently unused
  const regsubs_T *subs = (const regsubs_T *)t_subs;
  regsubs_T *out_m = (regsubs_T *)m;
  const save_se_T *endp = (const save_se_T *)nfa_endp_ptr;

  // If "nfa_endp" is set it's only a match if it ends at "nfa_endp"
  if (endp != NULL) {
    if (REG_MULTI) {
      if (rex.lnum != endp->se_u.pos.lnum
          || (int)(rex.input - rex.line) != endp->se_u.pos.col) {
        return 0;  // No match
      }
    } else {
      if (rex.input != endp->se_u.ptr) {
        return 0;  // No match
      }
    }
  }

  // Do not set submatches for \@!
  if (state_c != NFA_END_INVISIBLE_NEG) {
    copy_sub(&out_m->norm, (regsub_T *)&subs->norm);
    if (rex.nfa_has_zsubexpr) {
      copy_sub(&out_m->synt, (regsub_T *)&subs->synt);
    }
  }

  // Match found!
  // The caller sets nfa_match = true and handles the nextlist check for clen
  return 1;
}

// Wrapper for match_zref (static function) for Rust
// Returns match result (0/1), sets bytelen output parameter
int nvim_nfa_match_zref(int subidx, int *bytelen) {
  return match_zref(subidx, bytelen);
}

// Forward declarations for helper functions used by Rust wrappers
static void copy_sub_off(regsub_T *to, regsub_T *from);
static void copy_ze_off(regsub_T *to, regsub_T *from);
static bool state_in_list(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs);

// Wrapper for copy_sub_off for Rust (Phase 8)
void nvim_nfa_copy_sub_off(void *to, const void *from) {
  copy_sub_off((regsub_T *)to, (regsub_T *)from);
}

// Wrapper for copy_ze_off for Rust (Phase 8)
void nvim_nfa_copy_ze_off(void *to, const void *from) {
  copy_ze_off((regsub_T *)to, (regsub_T *)from);
}

// Wrapper for state_in_list for Rust (Phase 8)
int nvim_nfa_state_in_list(const void *list, const void *state, const void *subs) {
  return state_in_list((nfa_list_T *)list, (nfa_state_T *)state, (regsubs_T *)subs) ? 1 : 0;
}

// Rust state processing function (Phase 4, updated in Phase 10e)
extern int rs_nfa_process_state(
    const void *t_ptr, int curc, int clen,
    void *prog_ptr, void *thislist_ptr, void *nextlist_ptr,
    void *start_ptr, void *submatch_ptr, void *m_ptr,
    int **listids, int *listids_len,
    int *listidx,
    void **add_state_ptr, int *add_here, int *add_count, int *add_off);

// Compatibility wrapper for rs_nfa_regmatch (old interface without listidx)
// TODO: Remove in Phase 10f when rs_nfa_regmatch is deleted
int nfa_regmatch_process_state(
    const void *t_ptr, int curc, int clen,
    void *prog_ptr, void *thislist_ptr, void *nextlist_ptr,
    void *start_ptr, void *submatch_ptr, void *m_ptr,
    int **listids, int *listids_len,
    void **add_state_ptr, int *add_here, int *add_count, int *add_off) {
  // Pass NULL for listidx - the old code path doesn't support return_code 3
  return rs_nfa_process_state(t_ptr, curc, clen, prog_ptr, thislist_ptr,
                              nextlist_ptr, start_ptr, submatch_ptr, m_ptr,
                              listids, listids_len, NULL, add_state_ptr, add_here,
                              add_count, add_off);
}

// NFA postfix output accessors for Rust
int *nvim_nfa_get_post_ptr(void) { return post_ptr; }
int *nvim_nfa_get_post_start(void) { return post_start; }

// Emit a token to the NFA postfix output
void nvim_nfa_emit(int c)
{
  if (post_ptr >= post_end) {
    realloc_post_list();
  }
  *post_ptr++ = c;
}

// nfa_classcodes - NFA codes for character classes (used by Rust)
const int *nvim_parse_get_nfa_classcodes(void) { return nfa_classcodes; }
int nvim_parse_get_nfa_classcodes_count(void) { return (int)(sizeof(nfa_classcodes) / sizeof(nfa_classcodes[0])); }

// post_ptr decrement helper for range emission
void nvim_nfa_post_ptr_decr(void) { post_ptr--; }

// Forward declarations for NFA helper functions (used in wrappers below)
static int nfa_recognize_char_class(uint8_t *start, const uint8_t *end, int extra_newl);
static void nfa_emit_equi_class(int c);
static int coll_get_char(void);

// Wrapper for nfa_recognize_char_class
int nvim_nfa_recognize_char_class(const uint8_t *start, const uint8_t *end, int extra)
{
  return nfa_recognize_char_class((uint8_t *)start, (uint8_t *)end, extra);
}

// Wrapper for nfa_emit_equi_class
void nvim_nfa_emit_equi_class(int c) { nfa_emit_equi_class(c); }

// Wrapper for coll_get_char
int nvim_coll_get_char(void) { return coll_get_char(); }

// Check if did_emsg was set (error occurred during parsing)
int nvim_regexp_check_did_emsg(void) { return did_emsg; }

// Rust implementation of nfa_regatom
extern int rs_nfa_regatom(void);

// Wrapper to expose nfa_regatom - now calls Rust
int nvim_nfa_regatom(void) { return rs_nfa_regatom(); }

// Helper functions used when doing re2post() ... regatom() parsing
#define EMIT(c) \
  do { \
    if (post_ptr >= post_end) { \
      realloc_post_list(); \
    } \
    *post_ptr++ = c; \
  } while (0)

/// Initialize internal variables before NFA compilation.
///
/// @param re_flags  @see vim_regcomp()
static void nfa_regcomp_start(uint8_t *expr, int re_flags)
{
  size_t postfix_size;
  size_t nstate_max;

  nstate = 0;
  // A reasonable estimation for maximum size
  nstate_max = (strlen((char *)expr) + 1) * 25;

  // Some items blow up in size, such as [A-z].  Add more space for that.
  // When it is still not enough realloc_post_list() will be used.
  nstate_max += 1000;

  // Size for postfix representation of expr.
  postfix_size = sizeof(int) * nstate_max;

  post_start = (int *)xmalloc(postfix_size);
  post_ptr = post_start;
  post_end = post_start + nstate_max;
  wants_nfa = false;
  rex.nfa_has_zend = false;
  rex.nfa_has_backref = false;

  // shared with BT engine
  regcomp_start(expr, re_flags);
}

// Figure out if the NFA state list starts with an anchor, must match at start
// of the line.
static int nfa_get_reganch(nfa_state_T *start, int depth)
{
  nfa_state_T *p = start;

  if (depth > 4) {
    return 0;
  }

  while (p != NULL) {
    switch (p->c) {
    case NFA_BOL:
    case NFA_BOF:
      return 1;           // yes!

    case NFA_ZSTART:
    case NFA_ZEND:
    case NFA_CURSOR:
    case NFA_VISUAL:

    case NFA_MOPEN:
    case NFA_MOPEN1:
    case NFA_MOPEN2:
    case NFA_MOPEN3:
    case NFA_MOPEN4:
    case NFA_MOPEN5:
    case NFA_MOPEN6:
    case NFA_MOPEN7:
    case NFA_MOPEN8:
    case NFA_MOPEN9:
    case NFA_NOPEN:
    case NFA_ZOPEN:
    case NFA_ZOPEN1:
    case NFA_ZOPEN2:
    case NFA_ZOPEN3:
    case NFA_ZOPEN4:
    case NFA_ZOPEN5:
    case NFA_ZOPEN6:
    case NFA_ZOPEN7:
    case NFA_ZOPEN8:
    case NFA_ZOPEN9:
      p = p->out;
      break;

    case NFA_SPLIT:
      return nfa_get_reganch(p->out, depth + 1)
             && nfa_get_reganch(p->out1, depth + 1);

    default:
      return 0;           // noooo
    }
  }
  return 0;
}

// Figure out if the NFA state list starts with a character which must match
// at start of the match.
static int nfa_get_regstart(nfa_state_T *start, int depth)
{
  nfa_state_T *p = start;

  if (depth > 4) {
    return 0;
  }

  while (p != NULL) {
    switch (p->c) {
    // all kinds of zero-width matches
    case NFA_BOL:
    case NFA_BOF:
    case NFA_BOW:
    case NFA_EOW:
    case NFA_ZSTART:
    case NFA_ZEND:
    case NFA_CURSOR:
    case NFA_VISUAL:
    case NFA_LNUM:
    case NFA_LNUM_GT:
    case NFA_LNUM_LT:
    case NFA_COL:
    case NFA_COL_GT:
    case NFA_COL_LT:
    case NFA_VCOL:
    case NFA_VCOL_GT:
    case NFA_VCOL_LT:
    case NFA_MARK:
    case NFA_MARK_GT:
    case NFA_MARK_LT:

    case NFA_MOPEN:
    case NFA_MOPEN1:
    case NFA_MOPEN2:
    case NFA_MOPEN3:
    case NFA_MOPEN4:
    case NFA_MOPEN5:
    case NFA_MOPEN6:
    case NFA_MOPEN7:
    case NFA_MOPEN8:
    case NFA_MOPEN9:
    case NFA_NOPEN:
    case NFA_ZOPEN:
    case NFA_ZOPEN1:
    case NFA_ZOPEN2:
    case NFA_ZOPEN3:
    case NFA_ZOPEN4:
    case NFA_ZOPEN5:
    case NFA_ZOPEN6:
    case NFA_ZOPEN7:
    case NFA_ZOPEN8:
    case NFA_ZOPEN9:
      p = p->out;
      break;

    case NFA_SPLIT: {
      int c1 = nfa_get_regstart(p->out, depth + 1);
      int c2 = nfa_get_regstart(p->out1, depth + 1);

      if (c1 == c2) {
        return c1;             // yes!
      }
      return 0;
    }

    default:
      if (p->c > 0) {
        return p->c;             // yes!
      }
      return 0;
    }
  }
  return 0;
}

// Figure out if the NFA state list contains just literal text and nothing
// else.  If so return a string in allocated memory with what must match after
// regstart.  Otherwise return NULL.
static uint8_t *nfa_get_match_text(nfa_state_T *start)
{
  nfa_state_T *p = start;
  int len = 0;
  uint8_t *ret;
  uint8_t *s;

  if (p->c != NFA_MOPEN) {
    return NULL;     // just in case
  }
  p = p->out;
  while (p->c > 0) {
    len += utf_char2len(p->c);
    p = p->out;
  }
  if (p->c != NFA_MCLOSE || p->out->c != NFA_MATCH) {
    return NULL;
  }

  ret = xmalloc((size_t)len);
  p = start->out->out;     // skip first char, it goes into regstart
  s = ret;
  while (p->c > 0) {
    s += utf_char2bytes(p->c, (char *)s);
    p = p->out;
  }
  *s = NUL;

  return ret;
}

// Allocate more space for post_start.  Called when
// running above the estimated number of states.
static void realloc_post_list(void)
{
  // For weird patterns the number of states can be very high. Increasing by
  // 50% seems a reasonable compromise between memory use and speed.
  const size_t new_max = (size_t)(post_end - post_start) * 3 / 2;
  int *new_start = xrealloc(post_start, new_max * sizeof(int));
  post_ptr = new_start + (post_ptr - post_start);
  post_end = new_start + new_max;
  post_start = new_start;
}

// Search between "start" and "end" and try to recognize a
// Rust implementation
extern int rs_nfa_recognize_char_class(const uint8_t *start, const uint8_t *end, int extra_newl);

// Rust NFA compilation functions (Phase 6)
extern nfa_state_T *rs_post2nfa(int *postfix, int *end, int nfa_calc_size,
                                nfa_state_T *state_ptr, int nstate, int *out_nstate);
extern void rs_nfa_postprocess(void *prog_ptr);
extern int rs_nfa_get_reganch_full(nfa_state_T *start, int depth);
extern int rs_nfa_get_regstart_full(nfa_state_T *start, int depth);
extern uint8_t *rs_nfa_get_match_text_full(nfa_state_T *start);

// Recognize a character class in expanded form. For example [0-9].
// On success, return the id the character class to be emitted.
// On failure, return 0 (=FAIL)
// Start points to the first char of the range, while end should point
// to the closing brace.
// Keep in mind that 'ignorecase' applies at execution time, thus [a-z] may
// need to be interpreted as [a-zA-Z].
static int nfa_recognize_char_class(uint8_t *start, const uint8_t *end, int extra_newl)
{
  return rs_nfa_recognize_char_class(start, end, extra_newl);
}

// Produce the bytes for equivalence class "c".
// Currently only handles latin1, latin9 and utf-8.
// Emits bytes in postfix notation: 'a,b,NFA_OR,c,NFA_OR' is
// equivalent to 'a OR b OR c'
//
// NOTE! When changing this function, also update reg_equi_class()
static void nfa_emit_equi_class(int c)
{
#define EMIT2(c)   EMIT(c); EMIT(NFA_CONCAT);

  {
#define A_grave 0xc0
#define A_acute 0xc1
#define A_circumflex 0xc2
#define A_virguilla 0xc3
#define A_diaeresis 0xc4
#define A_ring 0xc5
#define C_cedilla 0xc7
#define E_grave 0xc8
#define E_acute 0xc9
#define E_circumflex 0xca
#define E_diaeresis 0xcb
#define I_grave 0xcc
#define I_acute 0xcd
#define I_circumflex 0xce
#define I_diaeresis 0xcf
#define N_virguilla 0xd1
#define O_grave 0xd2
#define O_acute 0xd3
#define O_circumflex 0xd4
#define O_virguilla 0xd5
#define O_diaeresis 0xd6
#define O_slash 0xd8
#define U_grave 0xd9
#define U_acute 0xda
#define U_circumflex 0xdb
#define U_diaeresis 0xdc
#define Y_acute 0xdd
#define a_grave 0xe0
#define a_acute 0xe1
#define a_circumflex 0xe2
#define a_virguilla 0xe3
#define a_diaeresis 0xe4
#define a_ring 0xe5
#define c_cedilla 0xe7
#define e_grave 0xe8
#define e_acute 0xe9
#define e_circumflex 0xea
#define e_diaeresis 0xeb
#define i_grave 0xec
#define i_acute 0xed
#define i_circumflex 0xee
#define i_diaeresis 0xef
#define n_virguilla 0xf1
#define o_grave 0xf2
#define o_acute 0xf3
#define o_circumflex 0xf4
#define o_virguilla 0xf5
#define o_diaeresis 0xf6
#define o_slash 0xf8
#define u_grave 0xf9
#define u_acute 0xfa
#define u_circumflex 0xfb
#define u_diaeresis 0xfc
#define y_acute 0xfd
#define y_diaeresis 0xff
    switch (c) {
    case 'A':
    case A_grave:
    case A_acute:
    case A_circumflex:
    case A_virguilla:
    case A_diaeresis:
    case A_ring:
    case 0x100:
    case 0x102:
    case 0x104:
    case 0x1cd:
    case 0x1de:
    case 0x1e0:
    case 0x1fa:
    case 0x200:
    case 0x202:
    case 0x226:
    case 0x23a:
    case 0x1e00:
    case 0x1ea0:
    case 0x1ea2:
    case 0x1ea4:
    case 0x1ea6:
    case 0x1ea8:
    case 0x1eaa:
    case 0x1eac:
    case 0x1eae:
    case 0x1eb0:
    case 0x1eb2:
    case 0x1eb4:
    case 0x1eb6:
      EMIT2('A') EMIT2(A_grave) EMIT2(A_acute)
      EMIT2(A_circumflex) EMIT2(A_virguilla)
      EMIT2(A_diaeresis) EMIT2(A_ring)
      EMIT2(0x100) EMIT2(0x102) EMIT2(0x104)
      EMIT2(0x1cd) EMIT2(0x1de) EMIT2(0x1e0)
      EMIT2(0x1fa) EMIT2(0x200) EMIT2(0x202)
      EMIT2(0x226) EMIT2(0x23a) EMIT2(0x1e00)
      EMIT2(0x1ea0) EMIT2(0x1ea2) EMIT2(0x1ea4)
      EMIT2(0x1ea6) EMIT2(0x1ea8) EMIT2(0x1eaa)
      EMIT2(0x1eac) EMIT2(0x1eae) EMIT2(0x1eb0)
      EMIT2(0x1eb2) EMIT2(0x1eb6) EMIT2(0x1eb4)
      return;

    case 'B':
    case 0x181:
    case 0x243:
    case 0x1e02:
    case 0x1e04:
    case 0x1e06:
      EMIT2('B')
      EMIT2(0x181) EMIT2(0x243) EMIT2(0x1e02)
      EMIT2(0x1e04) EMIT2(0x1e06)
      return;

    case 'C':
    case C_cedilla:
    case 0x106:
    case 0x108:
    case 0x10a:
    case 0x10c:
    case 0x187:
    case 0x23b:
    case 0x1e08:
    case 0xa792:
      EMIT2('C') EMIT2(C_cedilla)
      EMIT2(0x106) EMIT2(0x108) EMIT2(0x10a)
      EMIT2(0x10c) EMIT2(0x187) EMIT2(0x23b)
      EMIT2(0x1e08) EMIT2(0xa792)
      return;

    case 'D':
    case 0x10e:
    case 0x110:
    case 0x18a:
    case 0x1e0a:
    case 0x1e0c:
    case 0x1e0e:
    case 0x1e10:
    case 0x1e12:
      EMIT2('D') EMIT2(0x10e) EMIT2(0x110) EMIT2(0x18a)
      EMIT2(0x1e0a) EMIT2(0x1e0c) EMIT2(0x1e0e)
      EMIT2(0x1e10) EMIT2(0x1e12)
      return;

    case 'E':
    case E_grave:
    case E_acute:
    case E_circumflex:
    case E_diaeresis:
    case 0x112:
    case 0x114:
    case 0x116:
    case 0x118:
    case 0x11a:
    case 0x204:
    case 0x206:
    case 0x228:
    case 0x246:
    case 0x1e14:
    case 0x1e16:
    case 0x1e18:
    case 0x1e1a:
    case 0x1e1c:
    case 0x1eb8:
    case 0x1eba:
    case 0x1ebc:
    case 0x1ebe:
    case 0x1ec0:
    case 0x1ec2:
    case 0x1ec4:
    case 0x1ec6:
      EMIT2('E') EMIT2(E_grave) EMIT2(E_acute)
      EMIT2(E_circumflex) EMIT2(E_diaeresis)
      EMIT2(0x112) EMIT2(0x114) EMIT2(0x116)
      EMIT2(0x118) EMIT2(0x11a) EMIT2(0x204)
      EMIT2(0x206) EMIT2(0x228) EMIT2(0x246)
      EMIT2(0x1e14) EMIT2(0x1e16) EMIT2(0x1e18)
      EMIT2(0x1e1a) EMIT2(0x1e1c) EMIT2(0x1eb8)
      EMIT2(0x1eba) EMIT2(0x1ebc) EMIT2(0x1ebe)
      EMIT2(0x1ec0) EMIT2(0x1ec2) EMIT2(0x1ec4)
      EMIT2(0x1ec6)
      return;

    case 'F':
    case 0x191:
    case 0x1e1e:
    case 0xa798:
      EMIT2('F') EMIT2(0x191) EMIT2(0x1e1e) EMIT2(0xa798)
      return;

    case 'G':
    case 0x11c:
    case 0x11e:
    case 0x120:
    case 0x122:
    case 0x193:
    case 0x1e4:
    case 0x1e6:
    case 0x1f4:
    case 0x1e20:
    case 0xa7a0:
      EMIT2('G') EMIT2(0x11c) EMIT2(0x11e) EMIT2(0x120)
      EMIT2(0x122) EMIT2(0x193) EMIT2(0x1e4)
      EMIT2(0x1e6) EMIT2(0x1f4) EMIT2(0x1e20)
      EMIT2(0xa7a0)
      return;

    case 'H':
    case 0x124:
    case 0x126:
    case 0x21e:
    case 0x1e22:
    case 0x1e24:
    case 0x1e26:
    case 0x1e28:
    case 0x1e2a:
    case 0x2c67:
      EMIT2('H') EMIT2(0x124) EMIT2(0x126) EMIT2(0x21e)
      EMIT2(0x1e22) EMIT2(0x1e24) EMIT2(0x1e26)
      EMIT2(0x1e28) EMIT2(0x1e2a) EMIT2(0x2c67)
      return;

    case 'I':
    case I_grave:
    case I_acute:
    case I_circumflex:
    case I_diaeresis:
    case 0x128:
    case 0x12a:
    case 0x12c:
    case 0x12e:
    case 0x130:
    case 0x197:
    case 0x1cf:
    case 0x208:
    case 0x20a:
    case 0x1e2c:
    case 0x1e2e:
    case 0x1ec8:
    case 0x1eca:
      EMIT2('I') EMIT2(I_grave) EMIT2(I_acute)
      EMIT2(I_circumflex) EMIT2(I_diaeresis)
      EMIT2(0x128) EMIT2(0x12a) EMIT2(0x12c)
      EMIT2(0x12e) EMIT2(0x130) EMIT2(0x197)
      EMIT2(0x1cf) EMIT2(0x208) EMIT2(0x20a)
      EMIT2(0x1e2c) EMIT2(0x1e2e) EMIT2(0x1ec8)
      EMIT2(0x1eca)
      return;

    case 'J':
    case 0x134:
    case 0x248:
      EMIT2('J') EMIT2(0x134) EMIT2(0x248)
      return;

    case 'K':
    case 0x136:
    case 0x198:
    case 0x1e8:
    case 0x1e30:
    case 0x1e32:
    case 0x1e34:
    case 0x2c69:
    case 0xa740:
      EMIT2('K') EMIT2(0x136) EMIT2(0x198) EMIT2(0x1e8)
      EMIT2(0x1e30) EMIT2(0x1e32) EMIT2(0x1e34)
      EMIT2(0x2c69) EMIT2(0xa740)
      return;

    case 'L':
    case 0x139:
    case 0x13b:
    case 0x13d:
    case 0x13f:
    case 0x141:
    case 0x23d:
    case 0x1e36:
    case 0x1e38:
    case 0x1e3a:
    case 0x1e3c:
    case 0x2c60:
      EMIT2('L') EMIT2(0x139) EMIT2(0x13b)
      EMIT2(0x13d) EMIT2(0x13f) EMIT2(0x141)
      EMIT2(0x23d) EMIT2(0x1e36) EMIT2(0x1e38)
      EMIT2(0x1e3a) EMIT2(0x1e3c) EMIT2(0x2c60)
      return;

    case 'M':
    case 0x1e3e:
    case 0x1e40:
    case 0x1e42:
      EMIT2('M') EMIT2(0x1e3e) EMIT2(0x1e40)
      EMIT2(0x1e42)
      return;

    case 'N':
    case N_virguilla:
    case 0x143:
    case 0x145:
    case 0x147:
    case 0x1f8:
    case 0x1e44:
    case 0x1e46:
    case 0x1e48:
    case 0x1e4a:
    case 0xa7a4:
      EMIT2('N') EMIT2(N_virguilla)
      EMIT2(0x143) EMIT2(0x145) EMIT2(0x147)
      EMIT2(0x1f8) EMIT2(0x1e44) EMIT2(0x1e46)
      EMIT2(0x1e48) EMIT2(0x1e4a) EMIT2(0xa7a4)
      return;

    case 'O':
    case O_grave:
    case O_acute:
    case O_circumflex:
    case O_virguilla:
    case O_diaeresis:
    case O_slash:
    case 0x14c:
    case 0x14e:
    case 0x150:
    case 0x19f:
    case 0x1a0:
    case 0x1d1:
    case 0x1ea:
    case 0x1ec:
    case 0x1fe:
    case 0x20c:
    case 0x20e:
    case 0x22a:
    case 0x22c:
    case 0x22e:
    case 0x230:
    case 0x1e4c:
    case 0x1e4e:
    case 0x1e50:
    case 0x1e52:
    case 0x1ecc:
    case 0x1ece:
    case 0x1ed0:
    case 0x1ed2:
    case 0x1ed4:
    case 0x1ed6:
    case 0x1ed8:
    case 0x1eda:
    case 0x1edc:
    case 0x1ede:
    case 0x1ee0:
    case 0x1ee2:
      EMIT2('O') EMIT2(O_grave) EMIT2(O_acute)
      EMIT2(O_circumflex) EMIT2(O_virguilla)
      EMIT2(O_diaeresis) EMIT2(O_slash)
      EMIT2(0x14c) EMIT2(0x14e) EMIT2(0x150)
      EMIT2(0x19f) EMIT2(0x1a0) EMIT2(0x1d1)
      EMIT2(0x1ea) EMIT2(0x1ec) EMIT2(0x1fe)
      EMIT2(0x20c) EMIT2(0x20e) EMIT2(0x22a)
      EMIT2(0x22c) EMIT2(0x22e) EMIT2(0x230)
      EMIT2(0x1e4c) EMIT2(0x1e4e) EMIT2(0x1e50)
      EMIT2(0x1e52) EMIT2(0x1ecc) EMIT2(0x1ece)
      EMIT2(0x1ed0) EMIT2(0x1ed2) EMIT2(0x1ed4)
      EMIT2(0x1ed6) EMIT2(0x1ed8) EMIT2(0x1eda)
      EMIT2(0x1edc) EMIT2(0x1ede) EMIT2(0x1ee0)
      EMIT2(0x1ee2)
      return;

    case 'P':
    case 0x1a4:
    case 0x1e54:
    case 0x1e56:
    case 0x2c63:
      EMIT2('P') EMIT2(0x1a4) EMIT2(0x1e54) EMIT2(0x1e56)
      EMIT2(0x2c63)
      return;

    case 'Q':
    case 0x24a:
      EMIT2('Q') EMIT2(0x24a)
      return;

    case 'R':
    case 0x154:
    case 0x156:
    case 0x158:
    case 0x210:
    case 0x212:
    case 0x24c:
    case 0x1e58:
    case 0x1e5a:
    case 0x1e5c:
    case 0x1e5e:
    case 0x2c64:
    case 0xa7a6:
      EMIT2('R') EMIT2(0x154) EMIT2(0x156) EMIT2(0x158)
      EMIT2(0x210) EMIT2(0x212) EMIT2(0x24c) EMIT2(0x1e58)
      EMIT2(0x1e5a) EMIT2(0x1e5c) EMIT2(0x1e5e) EMIT2(0x2c64)
      EMIT2(0xa7a6)
      return;

    case 'S':
    case 0x15a:
    case 0x15c:
    case 0x15e:
    case 0x160:
    case 0x218:
    case 0x1e60:
    case 0x1e62:
    case 0x1e64:
    case 0x1e66:
    case 0x1e68:
    case 0x2c7e:
    case 0xa7a8:
      EMIT2('S') EMIT2(0x15a) EMIT2(0x15c) EMIT2(0x15e)
      EMIT2(0x160) EMIT2(0x218) EMIT2(0x1e60) EMIT2(0x1e62)
      EMIT2(0x1e64) EMIT2(0x1e66) EMIT2(0x1e68) EMIT2(0x2c7e)
      EMIT2(0xa7a8)
      return;

    case 'T':
    case 0x162:
    case 0x164:
    case 0x166:
    case 0x1ac:
    case 0x1ae:
    case 0x21a:
    case 0x23e:
    case 0x1e6a:
    case 0x1e6c:
    case 0x1e6e:
    case 0x1e70:
      EMIT2('T') EMIT2(0x162) EMIT2(0x164) EMIT2(0x166)
      EMIT2(0x1ac) EMIT2(0x1ae) EMIT2(0x23e) EMIT2(0x21a)
      EMIT2(0x1e6a) EMIT2(0x1e6c) EMIT2(0x1e6e) EMIT2(0x1e70)
      return;

    case 'U':
    case U_grave:
    case U_acute:
    case U_diaeresis:
    case U_circumflex:
    case 0x168:
    case 0x16a:
    case 0x16c:
    case 0x16e:
    case 0x170:
    case 0x172:
    case 0x1af:
    case 0x1d3:
    case 0x1d5:
    case 0x1d7:
    case 0x1d9:
    case 0x1db:
    case 0x214:
    case 0x216:
    case 0x244:
    case 0x1e72:
    case 0x1e74:
    case 0x1e76:
    case 0x1e78:
    case 0x1e7a:
    case 0x1ee4:
    case 0x1ee6:
    case 0x1ee8:
    case 0x1eea:
    case 0x1eec:
    case 0x1eee:
    case 0x1ef0:
      EMIT2('U') EMIT2(U_grave) EMIT2(U_acute)
      EMIT2(U_diaeresis) EMIT2(U_circumflex)
      EMIT2(0x168) EMIT2(0x16a)
      EMIT2(0x16c) EMIT2(0x16e) EMIT2(0x170)
      EMIT2(0x172) EMIT2(0x1af) EMIT2(0x1d3)
      EMIT2(0x1d5) EMIT2(0x1d7) EMIT2(0x1d9)
      EMIT2(0x1db) EMIT2(0x214) EMIT2(0x216)
      EMIT2(0x244) EMIT2(0x1e72) EMIT2(0x1e74)
      EMIT2(0x1e76) EMIT2(0x1e78) EMIT2(0x1e7a)
      EMIT2(0x1ee4) EMIT2(0x1ee6) EMIT2(0x1ee8)
      EMIT2(0x1eea) EMIT2(0x1eec) EMIT2(0x1eee)
      EMIT2(0x1ef0)
      return;

    case 'V':
    case 0x1b2:
    case 0x1e7c:
    case 0x1e7e:
      EMIT2('V') EMIT2(0x1b2) EMIT2(0x1e7c) EMIT2(0x1e7e)
      return;

    case 'W':
    case 0x174:
    case 0x1e80:
    case 0x1e82:
    case 0x1e84:
    case 0x1e86:
    case 0x1e88:
      EMIT2('W') EMIT2(0x174) EMIT2(0x1e80) EMIT2(0x1e82)
      EMIT2(0x1e84) EMIT2(0x1e86) EMIT2(0x1e88)
      return;

    case 'X':
    case 0x1e8a:
    case 0x1e8c:
      EMIT2('X') EMIT2(0x1e8a) EMIT2(0x1e8c)
      return;

    case 'Y':
    case Y_acute:
    case 0x176:
    case 0x178:
    case 0x1b3:
    case 0x232:
    case 0x24e:
    case 0x1e8e:
    case 0x1ef2:
    case 0x1ef4:
    case 0x1ef6:
    case 0x1ef8:
      EMIT2('Y') EMIT2(Y_acute)
      EMIT2(0x176) EMIT2(0x178) EMIT2(0x1b3)
      EMIT2(0x232) EMIT2(0x24e) EMIT2(0x1e8e)
      EMIT2(0x1ef2) EMIT2(0x1ef4) EMIT2(0x1ef6)
      EMIT2(0x1ef8)
      return;

    case 'Z':
    case 0x179:
    case 0x17b:
    case 0x17d:
    case 0x1b5:
    case 0x1e90:
    case 0x1e92:
    case 0x1e94:
    case 0x2c6b:
      EMIT2('Z') EMIT2(0x179) EMIT2(0x17b) EMIT2(0x17d)
      EMIT2(0x1b5) EMIT2(0x1e90) EMIT2(0x1e92)
      EMIT2(0x1e94) EMIT2(0x2c6b)
      return;

    case 'a':
    case a_grave:
    case a_acute:
    case a_circumflex:
    case a_virguilla:
    case a_diaeresis:
    case a_ring:
    case 0x101:
    case 0x103:
    case 0x105:
    case 0x1ce:
    case 0x1df:
    case 0x1e1:
    case 0x1fb:
    case 0x201:
    case 0x203:
    case 0x227:
    case 0x1d8f:
    case 0x1e01:
    case 0x1e9a:
    case 0x1ea1:
    case 0x1ea3:
    case 0x1ea5:
    case 0x1ea7:
    case 0x1ea9:
    case 0x1eab:
    case 0x1ead:
    case 0x1eaf:
    case 0x1eb1:
    case 0x1eb3:
    case 0x1eb5:
    case 0x1eb7:
    case 0x2c65:
      EMIT2('a') EMIT2(a_grave) EMIT2(a_acute)
      EMIT2(a_circumflex) EMIT2(a_virguilla)
      EMIT2(a_diaeresis) EMIT2(a_ring)
      EMIT2(0x101) EMIT2(0x103) EMIT2(0x105)
      EMIT2(0x1ce) EMIT2(0x1df) EMIT2(0x1e1)
      EMIT2(0x1fb) EMIT2(0x201) EMIT2(0x203)
      EMIT2(0x227) EMIT2(0x1d8f) EMIT2(0x1e01)
      EMIT2(0x1e9a) EMIT2(0x1ea1) EMIT2(0x1ea3)
      EMIT2(0x1ea5) EMIT2(0x1ea7) EMIT2(0x1ea9)
      EMIT2(0x1eab) EMIT2(0x1ead) EMIT2(0x1eaf)
      EMIT2(0x1eb1) EMIT2(0x1eb3) EMIT2(0x1eb5)
      EMIT2(0x1eb7) EMIT2(0x2c65)
      return;

    case 'b':
    case 0x180:
    case 0x253:
    case 0x1d6c:
    case 0x1d80:
    case 0x1e03:
    case 0x1e05:
    case 0x1e07:
      EMIT2('b') EMIT2(0x180) EMIT2(0x253) EMIT2(0x1d6c)
      EMIT2(0x1d80) EMIT2(0x1e03) EMIT2(0x1e05) EMIT2(0x1e07)
      return;

    case 'c':
    case c_cedilla:
    case 0x107:
    case 0x109:
    case 0x10b:
    case 0x10d:
    case 0x188:
    case 0x23c:
    case 0x1e09:
    case 0xa793:
    case 0xa794:
      EMIT2('c') EMIT2(c_cedilla)
      EMIT2(0x107) EMIT2(0x109) EMIT2(0x10b)
      EMIT2(0x10d) EMIT2(0x188) EMIT2(0x23c)
      EMIT2(0x1e09) EMIT2(0xa793) EMIT2(0xa794)
      return;

    case 'd':
    case 0x10f:
    case 0x111:
    case 0x257:
    case 0x1d6d:
    case 0x1d81:
    case 0x1d91:
    case 0x1e0b:
    case 0x1e0d:
    case 0x1e0f:
    case 0x1e11:
    case 0x1e13:
      EMIT2('d') EMIT2(0x10f) EMIT2(0x111)
      EMIT2(0x257) EMIT2(0x1d6d) EMIT2(0x1d81)
      EMIT2(0x1d91) EMIT2(0x1e0b) EMIT2(0x1e0d)
      EMIT2(0x1e0f) EMIT2(0x1e11) EMIT2(0x1e13)
      return;

    case 'e':
    case e_grave:
    case e_acute:
    case e_circumflex:
    case e_diaeresis:
    case 0x113:
    case 0x115:
    case 0x117:
    case 0x119:
    case 0x11b:
    case 0x205:
    case 0x207:
    case 0x229:
    case 0x247:
    case 0x1d92:
    case 0x1e15:
    case 0x1e17:
    case 0x1e19:
    case 0x1e1b:
    case 0x1e1d:
    case 0x1eb9:
    case 0x1ebb:
    case 0x1ebd:
    case 0x1ebf:
    case 0x1ec1:
    case 0x1ec3:
    case 0x1ec5:
    case 0x1ec7:
      EMIT2('e') EMIT2(e_grave) EMIT2(e_acute)
      EMIT2(e_circumflex) EMIT2(e_diaeresis)
      EMIT2(0x113) EMIT2(0x115)
      EMIT2(0x117) EMIT2(0x119) EMIT2(0x11b)
      EMIT2(0x205) EMIT2(0x207) EMIT2(0x229)
      EMIT2(0x247) EMIT2(0x1d92) EMIT2(0x1e15)
      EMIT2(0x1e17) EMIT2(0x1e19) EMIT2(0x1e1b)
      EMIT2(0x1e1d) EMIT2(0x1eb9) EMIT2(0x1ebb)
      EMIT2(0x1ebd) EMIT2(0x1ebf) EMIT2(0x1ec1)
      EMIT2(0x1ec3) EMIT2(0x1ec5) EMIT2(0x1ec7)
      return;

    case 'f':
    case 0x192:
    case 0x1d6e:
    case 0x1d82:
    case 0x1e1f:
    case 0xa799:
      EMIT2('f') EMIT2(0x192) EMIT2(0x1d6e) EMIT2(0x1d82)
      EMIT2(0x1e1f) EMIT2(0xa799)
      return;

    case 'g':
    case 0x11d:
    case 0x11f:
    case 0x121:
    case 0x123:
    case 0x1e5:
    case 0x1e7:
    case 0x1f5:
    case 0x260:
    case 0x1d83:
    case 0x1e21:
    case 0xa7a1:
      EMIT2('g') EMIT2(0x11d) EMIT2(0x11f) EMIT2(0x121)
      EMIT2(0x123) EMIT2(0x1e5) EMIT2(0x1e7)
      EMIT2(0x1f5) EMIT2(0x260) EMIT2(0x1d83)
      EMIT2(0x1e21) EMIT2(0xa7a1)
      return;

    case 'h':
    case 0x125:
    case 0x127:
    case 0x21f:
    case 0x1e23:
    case 0x1e25:
    case 0x1e27:
    case 0x1e29:
    case 0x1e2b:
    case 0x1e96:
    case 0x2c68:
    case 0xa795:
      EMIT2('h') EMIT2(0x125) EMIT2(0x127) EMIT2(0x21f)
      EMIT2(0x1e23) EMIT2(0x1e25) EMIT2(0x1e27)
      EMIT2(0x1e29) EMIT2(0x1e2b) EMIT2(0x1e96)
      EMIT2(0x2c68) EMIT2(0xa795)
      return;

    case 'i':
    case i_grave:
    case i_acute:
    case i_circumflex:
    case i_diaeresis:
    case 0x129:
    case 0x12b:
    case 0x12d:
    case 0x12f:
    case 0x1d0:
    case 0x209:
    case 0x20b:
    case 0x268:
    case 0x1d96:
    case 0x1e2d:
    case 0x1e2f:
    case 0x1ec9:
    case 0x1ecb:
      EMIT2('i') EMIT2(i_grave) EMIT2(i_acute)
      EMIT2(i_circumflex) EMIT2(i_diaeresis)
      EMIT2(0x129) EMIT2(0x12b) EMIT2(0x12d)
      EMIT2(0x12f) EMIT2(0x1d0) EMIT2(0x209)
      EMIT2(0x20b) EMIT2(0x268) EMIT2(0x1d96)
      EMIT2(0x1e2d) EMIT2(0x1e2f) EMIT2(0x1ec9)
      EMIT2(0x1ecb) EMIT2(0x1ecb)
      return;

    case 'j':
    case 0x135:
    case 0x1f0:
    case 0x249:
      EMIT2('j') EMIT2(0x135) EMIT2(0x1f0) EMIT2(0x249)
      return;

    case 'k':
    case 0x137:
    case 0x199:
    case 0x1e9:
    case 0x1d84:
    case 0x1e31:
    case 0x1e33:
    case 0x1e35:
    case 0x2c6a:
    case 0xa741:
      EMIT2('k') EMIT2(0x137) EMIT2(0x199) EMIT2(0x1e9)
      EMIT2(0x1d84) EMIT2(0x1e31) EMIT2(0x1e33)
      EMIT2(0x1e35) EMIT2(0x2c6a) EMIT2(0xa741)
      return;

    case 'l':
    case 0x13a:
    case 0x13c:
    case 0x13e:
    case 0x140:
    case 0x142:
    case 0x19a:
    case 0x1e37:
    case 0x1e39:
    case 0x1e3b:
    case 0x1e3d:
    case 0x2c61:
      EMIT2('l') EMIT2(0x13a) EMIT2(0x13c)
      EMIT2(0x13e) EMIT2(0x140) EMIT2(0x142)
      EMIT2(0x19a) EMIT2(0x1e37) EMIT2(0x1e39)
      EMIT2(0x1e3b) EMIT2(0x1e3d) EMIT2(0x2c61)
      return;

    case 'm':
    case 0x1d6f:
    case 0x1e3f:
    case 0x1e41:
    case 0x1e43:
      EMIT2('m') EMIT2(0x1d6f) EMIT2(0x1e3f)
      EMIT2(0x1e41) EMIT2(0x1e43)
      return;

    case 'n':
    case n_virguilla:
    case 0x144:
    case 0x146:
    case 0x148:
    case 0x149:
    case 0x1f9:
    case 0x1d70:
    case 0x1d87:
    case 0x1e45:
    case 0x1e47:
    case 0x1e49:
    case 0x1e4b:
    case 0xa7a5:
      EMIT2('n') EMIT2(n_virguilla)
      EMIT2(0x144) EMIT2(0x146) EMIT2(0x148)
      EMIT2(0x149) EMIT2(0x1f9) EMIT2(0x1d70)
      EMIT2(0x1d87) EMIT2(0x1e45) EMIT2(0x1e47)
      EMIT2(0x1e49) EMIT2(0x1e4b) EMIT2(0xa7a5)
      return;

    case 'o':
    case o_grave:
    case o_acute:
    case o_circumflex:
    case o_virguilla:
    case o_diaeresis:
    case o_slash:
    case 0x14d:
    case 0x14f:
    case 0x151:
    case 0x1a1:
    case 0x1d2:
    case 0x1eb:
    case 0x1ed:
    case 0x1ff:
    case 0x20d:
    case 0x20f:
    case 0x22b:
    case 0x22d:
    case 0x22f:
    case 0x231:
    case 0x275:
    case 0x1e4d:
    case 0x1e4f:
    case 0x1e51:
    case 0x1e53:
    case 0x1ecd:
    case 0x1ecf:
    case 0x1ed1:
    case 0x1ed3:
    case 0x1ed5:
    case 0x1ed7:
    case 0x1ed9:
    case 0x1edb:
    case 0x1edd:
    case 0x1edf:
    case 0x1ee1:
    case 0x1ee3:
      EMIT2('o') EMIT2(o_grave) EMIT2(o_acute)
      EMIT2(o_circumflex) EMIT2(o_virguilla)
      EMIT2(o_diaeresis) EMIT2(o_slash)
      EMIT2(0x14d) EMIT2(0x14f) EMIT2(0x151)
      EMIT2(0x1a1) EMIT2(0x1d2) EMIT2(0x1eb)
      EMIT2(0x1ed) EMIT2(0x1ff) EMIT2(0x20d)
      EMIT2(0x20f) EMIT2(0x22b) EMIT2(0x22d)
      EMIT2(0x22f) EMIT2(0x231) EMIT2(0x275)
      EMIT2(0x1e4d) EMIT2(0x1e4f) EMIT2(0x1e51)
      EMIT2(0x1e53) EMIT2(0x1ecd) EMIT2(0x1ecf)
      EMIT2(0x1ed1) EMIT2(0x1ed3) EMIT2(0x1ed5)
      EMIT2(0x1ed7) EMIT2(0x1ed9) EMIT2(0x1edb)
      EMIT2(0x1edd) EMIT2(0x1edf) EMIT2(0x1ee1)
      EMIT2(0x1ee3)
      return;

    case 'p':
    case 0x1a5:
    case 0x1d71:
    case 0x1d7d:
    case 0x1d88:
    case 0x1e55:
    case 0x1e57:
      EMIT2('p') EMIT2(0x1a5) EMIT2(0x1d71) EMIT2(0x1d7d)
      EMIT2(0x1d88) EMIT2(0x1e55) EMIT2(0x1e57)
      return;

    case 'q':
    case 0x24b:
    case 0x2a0:
      EMIT2('q') EMIT2(0x24b) EMIT2(0x2a0)
      return;

    case 'r':
    case 0x155:
    case 0x157:
    case 0x159:
    case 0x211:
    case 0x213:
    case 0x24d:
    case 0x27d:
    case 0x1d72:
    case 0x1d73:
    case 0x1d89:
    case 0x1e59:
    case 0x1e5b:
    case 0x1e5d:
    case 0x1e5f:
    case 0xa7a7:
      EMIT2('r') EMIT2(0x155) EMIT2(0x157) EMIT2(0x159)
      EMIT2(0x211) EMIT2(0x213) EMIT2(0x24d) EMIT2(0x27d)
      EMIT2(0x1d72) EMIT2(0x1d73) EMIT2(0x1d89) EMIT2(0x1e59)
      EMIT2(0x1e5b) EMIT2(0x1e5d) EMIT2(0x1e5f) EMIT2(0xa7a7)
      return;

    case 's':
    case 0x15b:
    case 0x15d:
    case 0x15f:
    case 0x161:
    case 0x219:
    case 0x23f:
    case 0x1d74:
    case 0x1d8a:
    case 0x1e61:
    case 0x1e63:
    case 0x1e65:
    case 0x1e67:
    case 0x1e69:
    case 0xa7a9:
      EMIT2('s') EMIT2(0x15b) EMIT2(0x15d) EMIT2(0x15f)
      EMIT2(0x161) EMIT2(0x219) EMIT2(0x23f) EMIT2(0x1d74)
      EMIT2(0x1d8a) EMIT2(0x1e61) EMIT2(0x1e63) EMIT2(0x1e65)
      EMIT2(0x1e67) EMIT2(0x1e69) EMIT2(0xa7a9)
      return;

    case 't':
    case 0x163:
    case 0x165:
    case 0x167:
    case 0x1ab:
    case 0x1ad:
    case 0x21b:
    case 0x288:
    case 0x1d75:
    case 0x1e6b:
    case 0x1e6d:
    case 0x1e6f:
    case 0x1e71:
    case 0x1e97:
    case 0x2c66:
      EMIT2('t') EMIT2(0x163) EMIT2(0x165) EMIT2(0x167)
      EMIT2(0x1ab) EMIT2(0x1ad) EMIT2(0x21b) EMIT2(0x288)
      EMIT2(0x1d75) EMIT2(0x1e6b) EMIT2(0x1e6d) EMIT2(0x1e6f)
      EMIT2(0x1e71) EMIT2(0x1e97) EMIT2(0x2c66)
      return;

    case 'u':
    case u_grave:
    case u_acute:
    case u_circumflex:
    case u_diaeresis:
    case 0x169:
    case 0x16b:
    case 0x16d:
    case 0x16f:
    case 0x171:
    case 0x173:
    case 0x1b0:
    case 0x1d4:
    case 0x1d6:
    case 0x1d8:
    case 0x1da:
    case 0x1dc:
    case 0x215:
    case 0x217:
    case 0x289:
    case 0x1d7e:
    case 0x1d99:
    case 0x1e73:
    case 0x1e75:
    case 0x1e77:
    case 0x1e79:
    case 0x1e7b:
    case 0x1ee5:
    case 0x1ee7:
    case 0x1ee9:
    case 0x1eeb:
    case 0x1eed:
    case 0x1eef:
    case 0x1ef1:
      EMIT2('u') EMIT2(u_grave) EMIT2(u_acute)
      EMIT2(u_circumflex) EMIT2(u_diaeresis)
      EMIT2(0x169) EMIT2(0x16b)
      EMIT2(0x16d) EMIT2(0x16f) EMIT2(0x171)
      EMIT2(0x173) EMIT2(0x1d6) EMIT2(0x1d8)
      EMIT2(0x215) EMIT2(0x217) EMIT2(0x1b0)
      EMIT2(0x1d4) EMIT2(0x1da) EMIT2(0x1dc)
      EMIT2(0x289) EMIT2(0x1e73) EMIT2(0x1d7e)
      EMIT2(0x1d99) EMIT2(0x1e75) EMIT2(0x1e77)
      EMIT2(0x1e79) EMIT2(0x1e7b) EMIT2(0x1ee5)
      EMIT2(0x1ee7) EMIT2(0x1ee9) EMIT2(0x1eeb)
      EMIT2(0x1eed) EMIT2(0x1eef) EMIT2(0x1ef1)
      return;

    case 'v':
    case 0x28b:
    case 0x1d8c:
    case 0x1e7d:
    case 0x1e7f:
      EMIT2('v') EMIT2(0x28b) EMIT2(0x1d8c) EMIT2(0x1e7d)
      EMIT2(0x1e7f)
      return;

    case 'w':
    case 0x175:
    case 0x1e81:
    case 0x1e83:
    case 0x1e85:
    case 0x1e87:
    case 0x1e89:
    case 0x1e98:
      EMIT2('w') EMIT2(0x175) EMIT2(0x1e81) EMIT2(0x1e83)
      EMIT2(0x1e85) EMIT2(0x1e87) EMIT2(0x1e89) EMIT2(0x1e98)
      return;

    case 'x':
    case 0x1e8b:
    case 0x1e8d:
      EMIT2('x') EMIT2(0x1e8b) EMIT2(0x1e8d)
      return;

    case 'y':
    case y_acute:
    case y_diaeresis:
    case 0x177:
    case 0x1b4:
    case 0x233:
    case 0x24f:
    case 0x1e8f:
    case 0x1e99:
    case 0x1ef3:
    case 0x1ef5:
    case 0x1ef7:
    case 0x1ef9:
      EMIT2('y') EMIT2(y_acute) EMIT2(y_diaeresis)
      EMIT2(0x177) EMIT2(0x1b4) EMIT2(0x233) EMIT2(0x24f)
      EMIT2(0x1e8f) EMIT2(0x1e99) EMIT2(0x1ef3)
      EMIT2(0x1ef5) EMIT2(0x1ef7) EMIT2(0x1ef9)
      return;

    case 'z':
    case 0x17a:
    case 0x17c:
    case 0x17e:
    case 0x1b6:
    case 0x1d76:
    case 0x1d8e:
    case 0x1e91:
    case 0x1e93:
    case 0x1e95:
    case 0x2c6c:
      EMIT2('z') EMIT2(0x17a) EMIT2(0x17c) EMIT2(0x17e)
      EMIT2(0x1b6) EMIT2(0x1d76) EMIT2(0x1d8e) EMIT2(0x1e91)
      EMIT2(0x1e93) EMIT2(0x1e95) EMIT2(0x2c6c)
      return;

      // default: character itself
    }
  }

  EMIT2(c);
#undef EMIT2
}

// Code to parse regular expression.
//
// We try to reuse parsing functions in regexp.c to
// minimize surprise and keep the syntax consistent.
// NOTE: nfa_regatom() is now implemented in Rust (src/nvim-rs/regexp/src/nfa_parser.rs)

// Parse something followed by possible [*+=].
//
// A piece is an atom, possibly followed by a multi, an indication of how many
// times the atom can be matched.  Example: "a*" matches any sequence of "a"
// characters: "", "a", "aa", etc.
//
// piece   ::=      atom
//      or  atom  multi
static int nfa_regpiece(void)
{
  int i;
  int op;
  int ret;
  int minval, maxval;
  bool greedy = true;  // Braces are prefixed with '-' ?
  parse_state_T old_state;
  parse_state_T new_state;
  int64_t c2;
  int old_post_pos;
  int my_post_start;
  int quest;

  // Save the current parse state, so that we can use it if <atom>{m,n} is
  // next.
  save_parse_state(&old_state);

  // store current pos in the postfix form, for \{m,n} involving 0s
  my_post_start = (int)(post_ptr - post_start);

  ret = rs_nfa_regatom();
  if (ret == FAIL) {
    return FAIL;            // cascaded error
  }
  op = peekchr();
  if (rs_re_multi_type(op) == NOT_MULTI) {
    return OK;
  }

  skipchr();
  switch (op) {
  case Magic('*'):
    EMIT(NFA_STAR);
    break;

  case Magic('+'):
    // Trick: Normally, (a*)\+ would match the whole input "aaa".  The
    // first and only submatch would be "aaa". But the backtracking
    // engine interprets the plus as "try matching one more time", and
    // a* matches a second time at the end of the input, the empty
    // string.
    // The submatch will be the empty string.
    //
    // In order to be consistent with the old engine, we replace
    // <atom>+ with <atom><atom>*
    restore_parse_state(&old_state);
    curchr = -1;
    if (rs_nfa_regatom() == FAIL) {
      return FAIL;
    }
    EMIT(NFA_STAR);
    EMIT(NFA_CONCAT);
    skipchr();                  // skip the \+
    break;

  case Magic('@'):
    c2 = getdecchrs();
    op = rs_no_magic(getchr());
    i = 0;
    switch (op) {
    case '=':
      // \@=
      i = NFA_PREV_ATOM_NO_WIDTH;
      break;
    case '!':
      // \@!
      i = NFA_PREV_ATOM_NO_WIDTH_NEG;
      break;
    case '<':
      op = rs_no_magic(getchr());
      if (op == '=') {
        // \@<=
        i = NFA_PREV_ATOM_JUST_BEFORE;
      } else if (op == '!') {
        // \@<!
        i = NFA_PREV_ATOM_JUST_BEFORE_NEG;
      }
      break;
    case '>':
      // \@>
      i = NFA_PREV_ATOM_LIKE_PATTERN;
      break;
    }
    if (i == 0) {
      semsg(_("E869: (NFA) Unknown operator '\\@%c'"), op);
      return FAIL;
    }
    EMIT(i);
    if (i == NFA_PREV_ATOM_JUST_BEFORE
        || i == NFA_PREV_ATOM_JUST_BEFORE_NEG) {
      EMIT((int)c2);
    }
    break;

  case Magic('?'):
  case Magic('='):
    EMIT(NFA_QUEST);
    break;

  case Magic('{'):
    // a{2,5} will expand to 'aaa?a?a?'
    // a{-1,3} will expand to 'aa??a??', where ?? is the nongreedy
    // version of '?'
    // \v(ab){2,3} will expand to '(ab)(ab)(ab)?', where all the
    // parenthesis have the same id

    greedy = true;
    c2 = peekchr();
    if (c2 == '-' || c2 == Magic('-')) {
      skipchr();
      greedy = false;
    }
    if (!read_limits(&minval, &maxval)) {
      EMSG_RET_FAIL(_("E870: (NFA regexp) Error reading repetition limits"));
    }

    //  <atom>{0,inf}, <atom>{0,} and <atom>{}  are equivalent to
    //  <atom>*
    if (minval == 0 && maxval == MAX_LIMIT) {
      if (greedy) {
        // \{}, \{0,}
        EMIT(NFA_STAR);
      } else {
        // \{-}, \{-0,}
        EMIT(NFA_STAR_NONGREEDY);
      }
      break;
    }

    // Special case: x{0} or x{-0}
    if (maxval == 0) {
      // Ignore result of previous call to rs_nfa_regatom()
      post_ptr = post_start + my_post_start;
      // NFA_EMPTY is 0-length and works everywhere
      EMIT(NFA_EMPTY);
      return OK;
    }

    // The engine is very inefficient (uses too many states) when the
    // maximum is much larger than the minimum and when the maximum is
    // large.  However, when maxval is MAX_LIMIT, it is okay, as this
    // will emit NFA_STAR.
    // Bail out if we can use the other engine, but only, when the
    // pattern does not need the NFA engine like (e.g. [[:upper:]]\{2,\}
    // does not work with characters > 8 bit with the BT engine)
    if ((nfa_re_flags & RE_AUTO)
        && (maxval > 500 || maxval > minval + 200)
        && (maxval != MAX_LIMIT && minval < 200)
        && !wants_nfa) {
      return FAIL;
    }

    // Ignore previous call to rs_nfa_regatom()
    post_ptr = post_start + my_post_start;
    // Save parse state after the repeated atom and the \{}
    save_parse_state(&new_state);

    quest = (greedy == true ? NFA_QUEST : NFA_QUEST_NONGREEDY);
    for (i = 0; i < maxval; i++) {
      // Goto beginning of the repeated atom
      restore_parse_state(&old_state);
      old_post_pos = (int)(post_ptr - post_start);
      if (rs_nfa_regatom() == FAIL) {
        return FAIL;
      }
      // after "minval" times, atoms are optional
      if (i + 1 > minval) {
        if (maxval == MAX_LIMIT) {
          if (greedy) {
            EMIT(NFA_STAR);
          } else {
            EMIT(NFA_STAR_NONGREEDY);
          }
        } else {
          EMIT(quest);
        }
      }
      if (old_post_pos != my_post_start) {
        EMIT(NFA_CONCAT);
      }
      if (i + 1 > minval && maxval == MAX_LIMIT) {
        break;
      }
    }

    // Go to just after the repeated atom and the \{}
    restore_parse_state(&new_state);
    curchr = -1;

    break;

  default:
    break;
  }     // end switch

  if (rs_re_multi_type(peekchr()) != NOT_MULTI) {
    // Can't have a multi follow a multi.
    EMSG_RET_FAIL(_("E871: (NFA regexp) Can't have a multi follow a multi"));
  }

  return OK;
}

// Parse one or more pieces, concatenated.  It matches a match for the
// first piece, followed by a match for the second piece, etc.  Example:
// "f[0-9]b", first matches "f", then a digit and then "b".
//
// concat  ::=      piece
//      or  piece piece
//      or  piece piece piece
//      etc.
static int nfa_regconcat(void)
{
  bool cont = true;
  bool first = true;

  while (cont) {
    switch (peekchr()) {
    case NUL:
    case Magic('|'):
    case Magic('&'):
    case Magic(')'):
      cont = false;
      break;

    case Magic('Z'):
      regflags |= RF_ICOMBINE;
      skipchr_keepstart();
      break;
    case Magic('c'):
      regflags |= RF_ICASE;
      skipchr_keepstart();
      break;
    case Magic('C'):
      regflags |= RF_NOICASE;
      skipchr_keepstart();
      break;
    case Magic('v'):
      reg_magic = MAGIC_ALL;
      skipchr_keepstart();
      curchr = -1;
      break;
    case Magic('m'):
      reg_magic = MAGIC_ON;
      skipchr_keepstart();
      curchr = -1;
      break;
    case Magic('M'):
      reg_magic = MAGIC_OFF;
      skipchr_keepstart();
      curchr = -1;
      break;
    case Magic('V'):
      reg_magic = MAGIC_NONE;
      skipchr_keepstart();
      curchr = -1;
      break;

    default:
      if (nfa_regpiece() == FAIL) {
        return FAIL;
      }
      if (first == false) {
        EMIT(NFA_CONCAT);
      } else {
        first = false;
      }
      break;
    }
  }

  return OK;
}

// Parse a branch, one or more concats, separated by "\&".  It matches the
// last concat, but only if all the preceding concats also match at the same
// position.  Examples:
//      "foobeep\&..." matches "foo" in "foobeep".
//      ".*Peter\&.*Bob" matches in a line containing both "Peter" and "Bob"
//
// branch ::=       concat
//              or  concat \& concat
//              or  concat \& concat \& concat
//              etc.
static int nfa_regbranch(void)
{
  int old_post_pos;

  old_post_pos = (int)(post_ptr - post_start);

  // First branch, possibly the only one
  if (nfa_regconcat() == FAIL) {
    return FAIL;
  }

  // Try next concats
  while (peekchr() == Magic('&')) {
    skipchr();
    // if concat is empty do emit a node
    if (old_post_pos == (int)(post_ptr - post_start)) {
      EMIT(NFA_EMPTY);
    }
    EMIT(NFA_NOPEN);
    EMIT(NFA_PREV_ATOM_NO_WIDTH);
    old_post_pos = (int)(post_ptr - post_start);
    if (nfa_regconcat() == FAIL) {
      return FAIL;
    }
    // if concat is empty do emit a node
    if (old_post_pos == (int)(post_ptr - post_start)) {
      EMIT(NFA_EMPTY);
    }
    EMIT(NFA_CONCAT);
  }

  // if a branch is empty, emit one node for it
  if (old_post_pos == (int)(post_ptr - post_start)) {
    EMIT(NFA_EMPTY);
  }

  return OK;
}

///  Parse a pattern, one or more branches, separated by "\|".  It matches
///  anything that matches one of the branches.  Example: "foo\|beep" matches
///  "foo" and matches "beep".  If more than one branch matches, the first one
///  is used.
///
///  pattern ::=     branch
///      or  branch \| branch
///      or  branch \| branch \| branch
///      etc.
///
/// @param paren  REG_NOPAREN, REG_PAREN, REG_NPAREN or REG_ZPAREN
static int nfa_reg(int paren)
{
  int parno = 0;

  if (paren == REG_PAREN) {
    if (regnpar >= NSUBEXP) {   // Too many `('
      EMSG_RET_FAIL(_("E872: (NFA regexp) Too many '('"));
    }
    parno = regnpar++;
  } else if (paren == REG_ZPAREN) {
    // Make a ZOPEN node.
    if (regnzpar >= NSUBEXP) {
      EMSG_RET_FAIL(_("E879: (NFA regexp) Too many \\z("));
    }
    parno = regnzpar++;
  }

  if (nfa_regbranch() == FAIL) {
    return FAIL;            // cascaded error
  }
  while (peekchr() == Magic('|')) {
    skipchr();
    if (nfa_regbranch() == FAIL) {
      return FAIL;          // cascaded error
    }
    EMIT(NFA_OR);
  }

  // Check for proper termination.
  if (paren != REG_NOPAREN && getchr() != Magic(')')) {
    if (paren == REG_NPAREN) {
      EMSG2_RET_FAIL(_(e_unmatchedpp), reg_magic == MAGIC_ALL);
    } else {
      EMSG2_RET_FAIL(_(e_unmatchedp), reg_magic == MAGIC_ALL);
    }
  } else if (paren == REG_NOPAREN && peekchr() != NUL) {
    if (peekchr() == Magic(')')) {
      EMSG2_RET_FAIL(_(e_unmatchedpar), reg_magic == MAGIC_ALL);
    } else {
      EMSG_RET_FAIL(_("E873: (NFA regexp) proper termination error"));
    }
  }
  // Here we set the flag allowing back references to this set of
  // parentheses.
  if (paren == REG_PAREN) {
    had_endbrace[parno] = true;  // have seen the close paren
    EMIT(NFA_MOPEN + parno);
  } else if (paren == REG_ZPAREN) {
    EMIT(NFA_ZOPEN + parno);
  }

  return OK;
}

#ifdef REGEXP_DEBUG
static uint8_t code[50];

static void nfa_set_code(int c)
{
  int addnl = false;

  if (c >= NFA_FIRST_NL && c <= NFA_LAST_NL) {
    addnl = true;
    c -= NFA_ADD_NL;
  }

  STRCPY(code, "");
  switch (c) {
  case NFA_MATCH:
    STRCPY(code, "NFA_MATCH "); break;
  case NFA_SPLIT:
    STRCPY(code, "NFA_SPLIT "); break;
  case NFA_CONCAT:
    STRCPY(code, "NFA_CONCAT "); break;
  case NFA_NEWL:
    STRCPY(code, "NFA_NEWL "); break;
  case NFA_ZSTART:
    STRCPY(code, "NFA_ZSTART"); break;
  case NFA_ZEND:
    STRCPY(code, "NFA_ZEND"); break;

  case NFA_BACKREF1:
    STRCPY(code, "NFA_BACKREF1"); break;
  case NFA_BACKREF2:
    STRCPY(code, "NFA_BACKREF2"); break;
  case NFA_BACKREF3:
    STRCPY(code, "NFA_BACKREF3"); break;
  case NFA_BACKREF4:
    STRCPY(code, "NFA_BACKREF4"); break;
  case NFA_BACKREF5:
    STRCPY(code, "NFA_BACKREF5"); break;
  case NFA_BACKREF6:
    STRCPY(code, "NFA_BACKREF6"); break;
  case NFA_BACKREF7:
    STRCPY(code, "NFA_BACKREF7"); break;
  case NFA_BACKREF8:
    STRCPY(code, "NFA_BACKREF8"); break;
  case NFA_BACKREF9:
    STRCPY(code, "NFA_BACKREF9"); break;
  case NFA_ZREF1:
    STRCPY(code, "NFA_ZREF1"); break;
  case NFA_ZREF2:
    STRCPY(code, "NFA_ZREF2"); break;
  case NFA_ZREF3:
    STRCPY(code, "NFA_ZREF3"); break;
  case NFA_ZREF4:
    STRCPY(code, "NFA_ZREF4"); break;
  case NFA_ZREF5:
    STRCPY(code, "NFA_ZREF5"); break;
  case NFA_ZREF6:
    STRCPY(code, "NFA_ZREF6"); break;
  case NFA_ZREF7:
    STRCPY(code, "NFA_ZREF7"); break;
  case NFA_ZREF8:
    STRCPY(code, "NFA_ZREF8"); break;
  case NFA_ZREF9:
    STRCPY(code, "NFA_ZREF9"); break;
  case NFA_SKIP:
    STRCPY(code, "NFA_SKIP"); break;

  case NFA_PREV_ATOM_NO_WIDTH:
    STRCPY(code, "NFA_PREV_ATOM_NO_WIDTH"); break;
  case NFA_PREV_ATOM_NO_WIDTH_NEG:
    STRCPY(code, "NFA_PREV_ATOM_NO_WIDTH_NEG"); break;
  case NFA_PREV_ATOM_JUST_BEFORE:
    STRCPY(code, "NFA_PREV_ATOM_JUST_BEFORE"); break;
  case NFA_PREV_ATOM_JUST_BEFORE_NEG:
    STRCPY(code, "NFA_PREV_ATOM_JUST_BEFORE_NEG"); break;
  case NFA_PREV_ATOM_LIKE_PATTERN:
    STRCPY(code, "NFA_PREV_ATOM_LIKE_PATTERN"); break;

  case NFA_NOPEN:
    STRCPY(code, "NFA_NOPEN"); break;
  case NFA_NCLOSE:
    STRCPY(code, "NFA_NCLOSE"); break;
  case NFA_START_INVISIBLE:
    STRCPY(code, "NFA_START_INVISIBLE"); break;
  case NFA_START_INVISIBLE_FIRST:
    STRCPY(code, "NFA_START_INVISIBLE_FIRST"); break;
  case NFA_START_INVISIBLE_NEG:
    STRCPY(code, "NFA_START_INVISIBLE_NEG"); break;
  case NFA_START_INVISIBLE_NEG_FIRST:
    STRCPY(code, "NFA_START_INVISIBLE_NEG_FIRST"); break;
  case NFA_START_INVISIBLE_BEFORE:
    STRCPY(code, "NFA_START_INVISIBLE_BEFORE"); break;
  case NFA_START_INVISIBLE_BEFORE_FIRST:
    STRCPY(code, "NFA_START_INVISIBLE_BEFORE_FIRST"); break;
  case NFA_START_INVISIBLE_BEFORE_NEG:
    STRCPY(code, "NFA_START_INVISIBLE_BEFORE_NEG"); break;
  case NFA_START_INVISIBLE_BEFORE_NEG_FIRST:
    STRCPY(code, "NFA_START_INVISIBLE_BEFORE_NEG_FIRST"); break;
  case NFA_START_PATTERN:
    STRCPY(code, "NFA_START_PATTERN"); break;
  case NFA_END_INVISIBLE:
    STRCPY(code, "NFA_END_INVISIBLE"); break;
  case NFA_END_INVISIBLE_NEG:
    STRCPY(code, "NFA_END_INVISIBLE_NEG"); break;
  case NFA_END_PATTERN:
    STRCPY(code, "NFA_END_PATTERN"); break;

  case NFA_COMPOSING:
    STRCPY(code, "NFA_COMPOSING"); break;
  case NFA_END_COMPOSING:
    STRCPY(code, "NFA_END_COMPOSING"); break;
  case NFA_OPT_CHARS:
    STRCPY(code, "NFA_OPT_CHARS"); break;

  case NFA_MOPEN:
  case NFA_MOPEN1:
  case NFA_MOPEN2:
  case NFA_MOPEN3:
  case NFA_MOPEN4:
  case NFA_MOPEN5:
  case NFA_MOPEN6:
  case NFA_MOPEN7:
  case NFA_MOPEN8:
  case NFA_MOPEN9:
    STRCPY(code, "NFA_MOPEN(x)");
    code[10] = c - NFA_MOPEN + '0';
    break;
  case NFA_MCLOSE:
  case NFA_MCLOSE1:
  case NFA_MCLOSE2:
  case NFA_MCLOSE3:
  case NFA_MCLOSE4:
  case NFA_MCLOSE5:
  case NFA_MCLOSE6:
  case NFA_MCLOSE7:
  case NFA_MCLOSE8:
  case NFA_MCLOSE9:
    STRCPY(code, "NFA_MCLOSE(x)");
    code[11] = c - NFA_MCLOSE + '0';
    break;
  case NFA_ZOPEN:
  case NFA_ZOPEN1:
  case NFA_ZOPEN2:
  case NFA_ZOPEN3:
  case NFA_ZOPEN4:
  case NFA_ZOPEN5:
  case NFA_ZOPEN6:
  case NFA_ZOPEN7:
  case NFA_ZOPEN8:
  case NFA_ZOPEN9:
    STRCPY(code, "NFA_ZOPEN(x)");
    code[10] = c - NFA_ZOPEN + '0';
    break;
  case NFA_ZCLOSE:
  case NFA_ZCLOSE1:
  case NFA_ZCLOSE2:
  case NFA_ZCLOSE3:
  case NFA_ZCLOSE4:
  case NFA_ZCLOSE5:
  case NFA_ZCLOSE6:
  case NFA_ZCLOSE7:
  case NFA_ZCLOSE8:
  case NFA_ZCLOSE9:
    STRCPY(code, "NFA_ZCLOSE(x)");
    code[11] = c - NFA_ZCLOSE + '0';
    break;
  case NFA_EOL:
    STRCPY(code, "NFA_EOL "); break;
  case NFA_BOL:
    STRCPY(code, "NFA_BOL "); break;
  case NFA_EOW:
    STRCPY(code, "NFA_EOW "); break;
  case NFA_BOW:
    STRCPY(code, "NFA_BOW "); break;
  case NFA_EOF:
    STRCPY(code, "NFA_EOF "); break;
  case NFA_BOF:
    STRCPY(code, "NFA_BOF "); break;
  case NFA_LNUM:
    STRCPY(code, "NFA_LNUM "); break;
  case NFA_LNUM_GT:
    STRCPY(code, "NFA_LNUM_GT "); break;
  case NFA_LNUM_LT:
    STRCPY(code, "NFA_LNUM_LT "); break;
  case NFA_COL:
    STRCPY(code, "NFA_COL "); break;
  case NFA_COL_GT:
    STRCPY(code, "NFA_COL_GT "); break;
  case NFA_COL_LT:
    STRCPY(code, "NFA_COL_LT "); break;
  case NFA_VCOL:
    STRCPY(code, "NFA_VCOL "); break;
  case NFA_VCOL_GT:
    STRCPY(code, "NFA_VCOL_GT "); break;
  case NFA_VCOL_LT:
    STRCPY(code, "NFA_VCOL_LT "); break;
  case NFA_MARK:
    STRCPY(code, "NFA_MARK "); break;
  case NFA_MARK_GT:
    STRCPY(code, "NFA_MARK_GT "); break;
  case NFA_MARK_LT:
    STRCPY(code, "NFA_MARK_LT "); break;
  case NFA_CURSOR:
    STRCPY(code, "NFA_CURSOR "); break;
  case NFA_VISUAL:
    STRCPY(code, "NFA_VISUAL "); break;
  case NFA_ANY_COMPOSING:
    STRCPY(code, "NFA_ANY_COMPOSING "); break;

  case NFA_STAR:
    STRCPY(code, "NFA_STAR "); break;
  case NFA_STAR_NONGREEDY:
    STRCPY(code, "NFA_STAR_NONGREEDY "); break;
  case NFA_QUEST:
    STRCPY(code, "NFA_QUEST"); break;
  case NFA_QUEST_NONGREEDY:
    STRCPY(code, "NFA_QUEST_NON_GREEDY"); break;
  case NFA_EMPTY:
    STRCPY(code, "NFA_EMPTY"); break;
  case NFA_OR:
    STRCPY(code, "NFA_OR"); break;

  case NFA_START_COLL:
    STRCPY(code, "NFA_START_COLL"); break;
  case NFA_END_COLL:
    STRCPY(code, "NFA_END_COLL"); break;
  case NFA_START_NEG_COLL:
    STRCPY(code, "NFA_START_NEG_COLL"); break;
  case NFA_END_NEG_COLL:
    STRCPY(code, "NFA_END_NEG_COLL"); break;
  case NFA_RANGE:
    STRCPY(code, "NFA_RANGE"); break;
  case NFA_RANGE_MIN:
    STRCPY(code, "NFA_RANGE_MIN"); break;
  case NFA_RANGE_MAX:
    STRCPY(code, "NFA_RANGE_MAX"); break;

  case NFA_CLASS_ALNUM:
    STRCPY(code, "NFA_CLASS_ALNUM"); break;
  case NFA_CLASS_ALPHA:
    STRCPY(code, "NFA_CLASS_ALPHA"); break;
  case NFA_CLASS_BLANK:
    STRCPY(code, "NFA_CLASS_BLANK"); break;
  case NFA_CLASS_CNTRL:
    STRCPY(code, "NFA_CLASS_CNTRL"); break;
  case NFA_CLASS_DIGIT:
    STRCPY(code, "NFA_CLASS_DIGIT"); break;
  case NFA_CLASS_GRAPH:
    STRCPY(code, "NFA_CLASS_GRAPH"); break;
  case NFA_CLASS_LOWER:
    STRCPY(code, "NFA_CLASS_LOWER"); break;
  case NFA_CLASS_PRINT:
    STRCPY(code, "NFA_CLASS_PRINT"); break;
  case NFA_CLASS_PUNCT:
    STRCPY(code, "NFA_CLASS_PUNCT"); break;
  case NFA_CLASS_SPACE:
    STRCPY(code, "NFA_CLASS_SPACE"); break;
  case NFA_CLASS_UPPER:
    STRCPY(code, "NFA_CLASS_UPPER"); break;
  case NFA_CLASS_XDIGIT:
    STRCPY(code, "NFA_CLASS_XDIGIT"); break;
  case NFA_CLASS_TAB:
    STRCPY(code, "NFA_CLASS_TAB"); break;
  case NFA_CLASS_RETURN:
    STRCPY(code, "NFA_CLASS_RETURN"); break;
  case NFA_CLASS_BACKSPACE:
    STRCPY(code, "NFA_CLASS_BACKSPACE"); break;
  case NFA_CLASS_ESCAPE:
    STRCPY(code, "NFA_CLASS_ESCAPE"); break;
  case NFA_CLASS_IDENT:
    STRCPY(code, "NFA_CLASS_IDENT"); break;
  case NFA_CLASS_KEYWORD:
    STRCPY(code, "NFA_CLASS_KEYWORD"); break;
  case NFA_CLASS_FNAME:
    STRCPY(code, "NFA_CLASS_FNAME"); break;

  case NFA_ANY:
    STRCPY(code, "NFA_ANY"); break;
  case NFA_IDENT:
    STRCPY(code, "NFA_IDENT"); break;
  case NFA_SIDENT:
    STRCPY(code, "NFA_SIDENT"); break;
  case NFA_KWORD:
    STRCPY(code, "NFA_KWORD"); break;
  case NFA_SKWORD:
    STRCPY(code, "NFA_SKWORD"); break;
  case NFA_FNAME:
    STRCPY(code, "NFA_FNAME"); break;
  case NFA_SFNAME:
    STRCPY(code, "NFA_SFNAME"); break;
  case NFA_PRINT:
    STRCPY(code, "NFA_PRINT"); break;
  case NFA_SPRINT:
    STRCPY(code, "NFA_SPRINT"); break;
  case NFA_WHITE:
    STRCPY(code, "NFA_WHITE"); break;
  case NFA_NWHITE:
    STRCPY(code, "NFA_NWHITE"); break;
  case NFA_DIGIT:
    STRCPY(code, "NFA_DIGIT"); break;
  case NFA_NDIGIT:
    STRCPY(code, "NFA_NDIGIT"); break;
  case NFA_HEX:
    STRCPY(code, "NFA_HEX"); break;
  case NFA_NHEX:
    STRCPY(code, "NFA_NHEX"); break;
  case NFA_OCTAL:
    STRCPY(code, "NFA_OCTAL"); break;
  case NFA_NOCTAL:
    STRCPY(code, "NFA_NOCTAL"); break;
  case NFA_WORD:
    STRCPY(code, "NFA_WORD"); break;
  case NFA_NWORD:
    STRCPY(code, "NFA_NWORD"); break;
  case NFA_HEAD:
    STRCPY(code, "NFA_HEAD"); break;
  case NFA_NHEAD:
    STRCPY(code, "NFA_NHEAD"); break;
  case NFA_ALPHA:
    STRCPY(code, "NFA_ALPHA"); break;
  case NFA_NALPHA:
    STRCPY(code, "NFA_NALPHA"); break;
  case NFA_LOWER:
    STRCPY(code, "NFA_LOWER"); break;
  case NFA_NLOWER:
    STRCPY(code, "NFA_NLOWER"); break;
  case NFA_UPPER:
    STRCPY(code, "NFA_UPPER"); break;
  case NFA_NUPPER:
    STRCPY(code, "NFA_NUPPER"); break;
  case NFA_LOWER_IC:
    STRCPY(code, "NFA_LOWER_IC"); break;
  case NFA_NLOWER_IC:
    STRCPY(code, "NFA_NLOWER_IC"); break;
  case NFA_UPPER_IC:
    STRCPY(code, "NFA_UPPER_IC"); break;
  case NFA_NUPPER_IC:
    STRCPY(code, "NFA_NUPPER_IC"); break;

  default:
    STRCPY(code, "CHAR(x)");
    code[5] = c;
  }

  if (addnl == true) {
    strcat(code, " + NEWLINE ");
  }
}

static FILE *log_fd;
static const uint8_t e_log_open_failed[] =
  N_("Could not open temporary log file for writing, displaying on stderr... ");

// Print the postfix notation of the current regexp.
static void nfa_postfix_dump(uint8_t *expr, int retval)
{
  int *p;
  FILE *f;

  f = fopen(NFA_REGEXP_DUMP_LOG, "a");
  if (f == NULL) {
    return;
  }

  fprintf(f, "\n-------------------------\n");
  if (retval == FAIL) {
    fprintf(f, ">>> NFA engine failed... \n");
  } else if (retval == OK) {
    fprintf(f, ">>> NFA engine succeeded !\n");
  }
  fprintf(f, "Regexp: \"%s\"\nPostfix notation (char): \"", expr);
  for (p = post_start; *p && p < post_ptr; p++) {
    nfa_set_code(*p);
    fprintf(f, "%s, ", code);
  }
  fprintf(f, "\"\nPostfix notation (int): ");
  for (p = post_start; *p && p < post_ptr; p++) {
    fprintf(f, "%d ", *p);
  }
  fprintf(f, "\n\n");
  fclose(f);
}

// Print the NFA starting with a root node "state".
static void nfa_print_state(FILE *debugf, nfa_state_T *state)
{
  garray_T indent;

  ga_init(&indent, 1, 64);
  ga_append(&indent, NUL);
  nfa_print_state2(debugf, state, &indent);
  ga_clear(&indent);
}

static void nfa_print_state2(FILE *debugf, nfa_state_T *state, garray_T *indent)
{
  uint8_t *p;

  if (state == NULL) {
    return;
  }

  fprintf(debugf, "(%2d)", abs(state->id));

  // Output indent
  p = (uint8_t *)indent->ga_data;
  if (indent->ga_len >= 3) {
    int last = indent->ga_len - 3;
    uint8_t save[2];

    strncpy(save, &p[last], 2);  // NOLINT(runtime/printf)
    memcpy(&p[last], "+-", 2);
    fprintf(debugf, " %s", p);
    strncpy(&p[last], save, 2);  // NOLINT(runtime/printf)
  } else {
    fprintf(debugf, " %s", p);
  }

  nfa_set_code(state->c);
  fprintf(debugf, "%s (%d) (id=%d) val=%d\n",
          code,
          state->c,
          abs(state->id),
          state->val);
  if (state->id < 0) {
    return;
  }

  state->id = abs(state->id) * -1;

  // grow indent for state->out
  indent->ga_len -= 1;
  if (state->out1) {
    ga_concat(indent, (uint8_t *)"| ");
  } else {
    ga_concat(indent, (uint8_t *)"  ");
  }
  ga_append(indent, NUL);

  nfa_print_state2(debugf, state->out, indent);

  // replace last part of indent for state->out1
  indent->ga_len -= 3;
  ga_concat(indent, (uint8_t *)"  ");
  ga_append(indent, NUL);

  nfa_print_state2(debugf, state->out1, indent);

  // shrink indent
  indent->ga_len -= 3;
  ga_append(indent, NUL);
}

// Print the NFA state machine.
static void nfa_dump(nfa_regprog_T *prog)
{
  FILE *debugf = fopen(NFA_REGEXP_DUMP_LOG, "a");

  if (debugf == NULL) {
    return;
  }

  nfa_print_state(debugf, prog->start);

  if (prog->reganch) {
    fprintf(debugf, "reganch: %d\n", prog->reganch);
  }
  if (prog->regstart != NUL) {
    fprintf(debugf, "regstart: %c (decimal: %d)\n",
            prog->regstart, prog->regstart);
  }
  if (prog->match_text != NULL) {
    fprintf(debugf, "match_text: \"%s\"\n", prog->match_text);
  }

  fclose(debugf);
}
#endif  // REGEXP_DEBUG

// Parse r.e. @expr and convert it into postfix form.
// Return the postfix string on success, NULL otherwise.
static int *re2post(void)
{
  if (nfa_reg(REG_NOPAREN) == FAIL) {
    return NULL;
  }
  EMIT(NFA_MOPEN);
  return post_start;
}

// After building the NFA program, inspect it to add optimization hints.
static void nfa_postprocess(nfa_regprog_T *prog)
{
  int i;
  int c;

  for (i = 0; i < prog->nstate; i++) {
    c = prog->state[i].c;
    if (c == NFA_START_INVISIBLE
        || c == NFA_START_INVISIBLE_NEG
        || c == NFA_START_INVISIBLE_BEFORE
        || c == NFA_START_INVISIBLE_BEFORE_NEG) {
      int directly;

      // Do it directly when what follows is possibly the end of the
      // match.
      if (match_follows(prog->state[i].out1->out, 0)) {
        directly = true;
      } else {
        int ch_invisible = failure_chance(prog->state[i].out, 0);
        int ch_follows = failure_chance(prog->state[i].out1->out, 0);

        // Postpone when the invisible match is expensive or has a
        // lower chance of failing.
        if (c == NFA_START_INVISIBLE_BEFORE
            || c == NFA_START_INVISIBLE_BEFORE_NEG) {
          // "before" matches are very expensive when
          // unbounded, always prefer what follows then,
          // unless what follows will always match.
          // Otherwise strongly prefer what follows.
          if (prog->state[i].val <= 0 && ch_follows > 0) {
            directly = false;
          } else {
            directly = ch_follows * 10 < ch_invisible;
          }
        } else {
          // normal invisible, first do the one with the
          // highest failure chance
          directly = ch_follows < ch_invisible;
        }
      }
      if (directly) {
        // switch to the _FIRST state
        prog->state[i].c++;
      }
    }
  }
}

/////////////////////////////////////////////////////////////////
// NFA execution code.
/////////////////////////////////////////////////////////////////

// Values for done in nfa_pim_T.
#define NFA_PIM_UNUSED   0      // pim not used
#define NFA_PIM_TODO     1      // pim not done yet
#define NFA_PIM_MATCH    2      // pim executed, matches
#define NFA_PIM_NOMATCH  3      // pim executed, no match

#ifdef REGEXP_DEBUG
static void log_subsexpr(regsubs_T *subs)
{
  log_subexpr(&subs->norm);
  if (rex.nfa_has_zsubexpr) {
    log_subexpr(&subs->synt);
  }
}

static void log_subexpr(regsub_T *sub)
{
  int j;

  for (j = 0; j < sub->in_use; j++) {
    if (REG_MULTI) {
      fprintf(log_fd, "*** group %d, start: c=%d, l=%d, end: c=%d, l=%d\n",
              j,
              sub->list.multi[j].start_col,
              (int)sub->list.multi[j].start_lnum,
              sub->list.multi[j].end_col,
              (int)sub->list.multi[j].end_lnum);
    } else {
      char *s = (char *)sub->list.line[j].start;
      char *e = (char *)sub->list.line[j].end;

      fprintf(log_fd, "*** group %d, start: \"%s\", end: \"%s\"\n",
              j,
              s == NULL ? "NULL" : s,
              e == NULL ? "NULL" : e);
    }
  }
}

static char *pim_info(const nfa_pim_T *pim)
{
  static char buf[30];

  if (pim == NULL || pim->result == NFA_PIM_UNUSED) {
    buf[0] = NUL;
  } else {
    snprintf(buf, sizeof(buf), " PIM col %d",
             REG_MULTI
             ? (int)pim->end.pos.col
             : (int)(pim->end.ptr - rex.input));
  }
  return buf;
}

#endif

// Used during execution: whether a match has been found.
static int nfa_match;
static proftime_T *nfa_time_limit;
static int *nfa_timed_out;
static int nfa_time_count;

// NFA execution globals accessor definitions for Rust (Phase 1: nfa_regmatch migration)
int nvim_nfa_get_match(void) { return nfa_match; }
void nvim_nfa_set_match(int v) { nfa_match = v; }
proftime_T *nvim_nfa_get_time_limit(void) { return nfa_time_limit; }
void nvim_nfa_set_time_limit(proftime_T *p) { nfa_time_limit = p; }
int *nvim_nfa_get_timed_out_ptr(void) { return nfa_timed_out; }
void nvim_nfa_set_timed_out_ptr(int *p) { nfa_timed_out = p; }
int nvim_nfa_get_time_count(void) { return nfa_time_count; }
void nvim_nfa_set_time_count(int v) { nfa_time_count = v; }

// Rust FFI declarations for copy/clear functions
extern void rs_copy_pim(nfa_pim_T *to, const nfa_pim_T *from);
extern void rs_copy_sub(regsub_T *to, const regsub_T *from);
extern void rs_clear_sub(regsub_T *sub);
extern void rs_copy_sub_off(regsub_T *to, const regsub_T *from);
extern void rs_copy_ze_off(regsub_T *to, const regsub_T *from);

// Copy postponed invisible match info from "from" to "to".
static void copy_pim(nfa_pim_T *to, nfa_pim_T *from)
{
  rs_copy_pim(to, from);
}

static void clear_sub(regsub_T *sub)
{
  rs_clear_sub(sub);
}

// Copy the submatches from "from" to "to".
static void copy_sub(regsub_T *to, regsub_T *from)
{
  rs_copy_sub(to, from);
}

// Like copy_sub() but exclude the main match.
static void copy_sub_off(regsub_T *to, regsub_T *from)
{
  rs_copy_sub_off(to, from);
}

// Like copy_sub() but only do the end of the main match if \ze is present.
static void copy_ze_off(regsub_T *to, regsub_T *from)
{
  rs_copy_ze_off(to, from);
}

// Return true if "sub1" and "sub2" have the same start positions.
// When using back-references also check the end position.
extern int rs_sub_equal(regsub_T *sub1, regsub_T *sub2);

static bool sub_equal(regsub_T *sub1, regsub_T *sub2)
{
  return rs_sub_equal(sub1, sub2) != 0;
}

#ifdef REGEXP_DEBUG
static void open_debug_log(TriState result)
{
  log_fd = fopen(NFA_REGEXP_RUN_LOG, "a");
  if (log_fd == NULL) {
    emsg(_(e_log_open_failed));
    log_fd = stderr;
  }

  fprintf(log_fd, "****************************\n");
  fprintf(log_fd, "FINISHED RUNNING nfa_regmatch() recursively\n");
  fprintf(log_fd, "MATCH = %s\n", result == kTrue ? "OK" : result == kNone ? "MAYBE" : "FALSE");
  fprintf(log_fd, "****************************\n");
}

static void report_state(char *action, regsub_T *sub, nfa_state_T *state, int lid, nfa_pim_T *pim)
{
  int col;

  if (sub->in_use <= 0) {
    col = -1;
  } else if (REG_MULTI) {
    col = sub->list.multi[0].start_col;
  } else {
    col = (int)(sub->list.line[0].start - rex.line);
  }
  nfa_set_code(state->c);
  if (log_fd == NULL) {
    open_debug_log(kNone);
  }
  fprintf(log_fd, "> %s state %d to list %d. char %d: %s (start col %d)%s\n",
          action, abs(state->id), lid, state->c, code, col,
          pim_info(pim));
}

#endif

// Return true if "one" and "two" are equal.  That includes when both are not set.
extern int rs_pim_equal(const nfa_pim_T *one, const nfa_pim_T *two);

static bool pim_equal(const nfa_pim_T *one, const nfa_pim_T *two)
{
  return rs_pim_equal(one, two) != 0;
}

/// @param l      runtime state list
/// @param state  state to update
/// @param subs   pointers to subexpressions
/// @param pim    postponed match or NULL
///
/// @return  true if the same state is already in list "l" with the same
///          positions as "subs".
extern int rs_has_state_with_pos(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs, nfa_pim_T *pim);

static bool has_state_with_pos(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs, nfa_pim_T *pim)
  FUNC_ATTR_NONNULL_ARG(1, 2, 3)
{
  return rs_has_state_with_pos(l, state, subs, pim) != 0;
}

// Return true if "state" leads to a NFA_MATCH without advancing the input.
extern int rs_match_follows(const nfa_state_T *state, int depth);

static bool match_follows(const nfa_state_T *startstate, int depth)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_match_follows(startstate, depth) != 0;
}

#if 0  // Replaced by rs_match_follows
static bool match_follows_OLD(const nfa_state_T *startstate, int depth)
  FUNC_ATTR_NONNULL_ALL
{
  const nfa_state_T *state = startstate;

  // avoid too much recursion
  if (depth > 10) {
    return false;
  }
  while (state != NULL) {
    switch (state->c) {
    case NFA_MATCH:
    case NFA_MCLOSE:
    case NFA_END_INVISIBLE:
    case NFA_END_INVISIBLE_NEG:
    case NFA_END_PATTERN:
      return true;

    case NFA_SPLIT:
      return match_follows_OLD(state->out, depth + 1)
             || match_follows_OLD(state->out1, depth + 1);

    case NFA_START_INVISIBLE:
    case NFA_START_INVISIBLE_FIRST:
    case NFA_START_INVISIBLE_BEFORE:
    case NFA_START_INVISIBLE_BEFORE_FIRST:
    case NFA_START_INVISIBLE_NEG:
    case NFA_START_INVISIBLE_NEG_FIRST:
    case NFA_START_INVISIBLE_BEFORE_NEG:
    case NFA_START_INVISIBLE_BEFORE_NEG_FIRST:
    case NFA_COMPOSING:
      // skip ahead to next state
      state = state->out1->out;
      continue;

    case NFA_ANY:
    case NFA_ANY_COMPOSING:
    case NFA_IDENT:
    case NFA_SIDENT:
    case NFA_KWORD:
    case NFA_SKWORD:
    case NFA_FNAME:
    case NFA_SFNAME:
    case NFA_PRINT:
    case NFA_SPRINT:
    case NFA_WHITE:
    case NFA_NWHITE:
    case NFA_DIGIT:
    case NFA_NDIGIT:
    case NFA_HEX:
    case NFA_NHEX:
    case NFA_OCTAL:
    case NFA_NOCTAL:
    case NFA_WORD:
    case NFA_NWORD:
    case NFA_HEAD:
    case NFA_NHEAD:
    case NFA_ALPHA:
    case NFA_NALPHA:
    case NFA_LOWER:
    case NFA_NLOWER:
    case NFA_UPPER:
    case NFA_NUPPER:
    case NFA_LOWER_IC:
    case NFA_NLOWER_IC:
    case NFA_UPPER_IC:
    case NFA_NUPPER_IC:
    case NFA_START_COLL:
    case NFA_START_NEG_COLL:
    case NFA_NEWL:
      // state will advance input
      return false;

    default:
      if (state->c > 0) {
        // state will advance input
        return false;
      }
      // Others: zero-width or possibly zero-width, might still find
      // a match at the same position, keep looking.
      break;
    }
    state = state->out;
  }
  return false;
}
#endif  // End of match_follows_OLD

/// @param l      runtime state list
/// @param state  state to update
/// @param subs   pointers to subexpressions
///
/// @return  true if "state" is already in list "l".
static bool state_in_list(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs)
  FUNC_ATTR_NONNULL_ALL
{
  if (state->lastlist[nfa_ll_index] == l->id) {
    if (!rex.nfa_has_backref || has_state_with_pos(l, state, subs, NULL)) {
      return true;
    }
  }
  return false;
}

// Offset used for "off" by addstate_here().
#define ADDSTATE_HERE_OFFSET 10

/// Add "state" and possibly what follows to state list ".".
///
/// @param l         runtime state list
/// @param state     state to update
/// @param subs_arg  pointers to subexpressions
/// @param pim       postponed look-behind match
/// @param off_arg   byte offset, when -1 go to next line
///
/// @return  "subs_arg", possibly copied into temp_subs.
///          NULL when recursiveness is too deep.
static regsubs_T *addstate(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs_arg, nfa_pim_T *pim,
                           int off_arg)
  FUNC_ATTR_NONNULL_ARG(1, 2) FUNC_ATTR_WARN_UNUSED_RESULT
{
  int subidx;
  int off = off_arg;
  int add_here = false;
  int listindex = 0;
  int k;
  int found = false;
  nfa_thread_T *thread;
  struct multipos save_multipos;
  int save_in_use;
  uint8_t *save_ptr;
  int i;
  regsub_T *sub;
  regsubs_T *subs = subs_arg;
  static regsubs_T temp_subs;
#ifdef REGEXP_DEBUG
  int did_print = false;
#endif
  static int depth = 0;

  // This function is called recursively.  When the depth is too much we run
  // out of stack and crash, limit recursiveness here.
  if (++depth >= 5000 || subs == NULL) {
    depth--;
    return NULL;
  }

  if (off_arg <= -ADDSTATE_HERE_OFFSET) {
    add_here = true;
    off = 0;
    listindex = -(off_arg + ADDSTATE_HERE_OFFSET);
  }

  switch (state->c) {
  case NFA_NCLOSE:
  case NFA_MCLOSE:
  case NFA_MCLOSE1:
  case NFA_MCLOSE2:
  case NFA_MCLOSE3:
  case NFA_MCLOSE4:
  case NFA_MCLOSE5:
  case NFA_MCLOSE6:
  case NFA_MCLOSE7:
  case NFA_MCLOSE8:
  case NFA_MCLOSE9:
  case NFA_ZCLOSE:
  case NFA_ZCLOSE1:
  case NFA_ZCLOSE2:
  case NFA_ZCLOSE3:
  case NFA_ZCLOSE4:
  case NFA_ZCLOSE5:
  case NFA_ZCLOSE6:
  case NFA_ZCLOSE7:
  case NFA_ZCLOSE8:
  case NFA_ZCLOSE9:
  case NFA_MOPEN:
  case NFA_ZEND:
  case NFA_SPLIT:
  case NFA_EMPTY:
    // These nodes are not added themselves but their "out" and/or
    // "out1" may be added below.
    break;

  case NFA_BOL:
  case NFA_BOF:
    // "^" won't match past end-of-line, don't bother trying.
    // Except when at the end of the line, or when we are going to the
    // next line for a look-behind match.
    if (rex.input > rex.line
        && *rex.input != NUL
        && (nfa_endp == NULL
            || !REG_MULTI
            || rex.lnum == nfa_endp->se_u.pos.lnum)) {
      goto skip_add;
    }
    FALLTHROUGH;

  case NFA_MOPEN1:
  case NFA_MOPEN2:
  case NFA_MOPEN3:
  case NFA_MOPEN4:
  case NFA_MOPEN5:
  case NFA_MOPEN6:
  case NFA_MOPEN7:
  case NFA_MOPEN8:
  case NFA_MOPEN9:
  case NFA_ZOPEN:
  case NFA_ZOPEN1:
  case NFA_ZOPEN2:
  case NFA_ZOPEN3:
  case NFA_ZOPEN4:
  case NFA_ZOPEN5:
  case NFA_ZOPEN6:
  case NFA_ZOPEN7:
  case NFA_ZOPEN8:
  case NFA_ZOPEN9:
  case NFA_NOPEN:
  case NFA_ZSTART:
  // These nodes need to be added so that we can bail out when it
  // was added to this list before at the same position to avoid an
  // endless loop for "\(\)*"

  default:
    if (state->lastlist[nfa_ll_index] == l->id && state->c != NFA_SKIP) {
      // This state is already in the list, don't add it again,
      // unless it is an MOPEN that is used for a backreference or
      // when there is a PIM. For NFA_MATCH check the position,
      // lower position is preferred.
      if (!rex.nfa_has_backref && pim == NULL && !l->has_pim
          && state->c != NFA_MATCH) {
        // When called from addstate_here() do insert before
        // existing states.
        if (add_here) {
          for (k = 0; k < l->n && k < listindex; k++) {
            if (l->t[k].state->id == state->id) {
              found = true;
              break;
            }
          }
        }

        if (!add_here || found) {
skip_add:
#ifdef REGEXP_DEBUG
          nfa_set_code(state->c);
          fprintf(log_fd,
                  "> Not adding state %d to list %d. char %d: %s pim: %s has_pim: %d found: %d\n",
                  abs(state->id), l->id, state->c, code,
                  pim == NULL ? "NULL" : "yes", l->has_pim, found);
#endif
          depth--;
          return subs;
        }
      }

      // Do not add the state again when it exists with the same
      // positions.
      if (has_state_with_pos(l, state, subs, pim)) {
        goto skip_add;
      }
    }

    // When there are backreferences or PIMs the number of states may
    // be (a lot) bigger than anticipated.
    if (l->n == l->len) {
      const int newlen = l->len * 3 / 2 + 50;
      const size_t newsize = (size_t)newlen * sizeof(nfa_thread_T);

      if ((int64_t)(newsize >> 10) >= p_mmp) {
        emsg(_(e_pattern_uses_more_memory_than_maxmempattern));
        depth--;
        return NULL;
      }
      if (subs != &temp_subs) {
        // "subs" may point into the current array, need to make a
        // copy before it becomes invalid.
        copy_sub(&temp_subs.norm, &subs->norm);
        if (rex.nfa_has_zsubexpr) {
          copy_sub(&temp_subs.synt, &subs->synt);
        }
        subs = &temp_subs;
      }

      nfa_thread_T *const newt = xrealloc(l->t, newsize);
      l->t = newt;
      l->len = newlen;
    }

    // add the state to the list
    state->lastlist[nfa_ll_index] = l->id;
    thread = &l->t[l->n++];
    thread->state = state;
    if (pim == NULL) {
      thread->pim.result = NFA_PIM_UNUSED;
    } else {
      copy_pim(&thread->pim, pim);
      l->has_pim = true;
    }
    copy_sub(&thread->subs.norm, &subs->norm);
    if (rex.nfa_has_zsubexpr) {
      copy_sub(&thread->subs.synt, &subs->synt);
    }
#ifdef REGEXP_DEBUG
    report_state("Adding", &thread->subs.norm, state, l->id, pim);
    did_print = true;
#endif
  }

#ifdef REGEXP_DEBUG
  if (!did_print) {
    report_state("Processing", &subs->norm, state, l->id, pim);
  }
#endif
  switch (state->c) {
  case NFA_MATCH:
    break;

  case NFA_SPLIT:
    // order matters here
    subs = addstate(l, state->out, subs, pim, off_arg);
    subs = addstate(l, state->out1, subs, pim, off_arg);
    break;

  case NFA_EMPTY:
  case NFA_NOPEN:
  case NFA_NCLOSE:
    subs = addstate(l, state->out, subs, pim, off_arg);
    break;

  case NFA_MOPEN:
  case NFA_MOPEN1:
  case NFA_MOPEN2:
  case NFA_MOPEN3:
  case NFA_MOPEN4:
  case NFA_MOPEN5:
  case NFA_MOPEN6:
  case NFA_MOPEN7:
  case NFA_MOPEN8:
  case NFA_MOPEN9:
  case NFA_ZOPEN:
  case NFA_ZOPEN1:
  case NFA_ZOPEN2:
  case NFA_ZOPEN3:
  case NFA_ZOPEN4:
  case NFA_ZOPEN5:
  case NFA_ZOPEN6:
  case NFA_ZOPEN7:
  case NFA_ZOPEN8:
  case NFA_ZOPEN9:
  case NFA_ZSTART:
    if (state->c == NFA_ZSTART) {
      subidx = 0;
      sub = &subs->norm;
    } else if (state->c >= NFA_ZOPEN && state->c <= NFA_ZOPEN9) {
      subidx = state->c - NFA_ZOPEN;
      sub = &subs->synt;
    } else {
      subidx = state->c - NFA_MOPEN;
      sub = &subs->norm;
    }

    // avoid compiler warnings
    save_ptr = NULL;
    CLEAR_FIELD(save_multipos);

    // Set the position (with "off" added) in the subexpression.  Save
    // and restore it when it was in use.  Otherwise fill any gap.
    if (REG_MULTI) {
      if (subidx < sub->in_use) {
        save_multipos = sub->list.multi[subidx];
        save_in_use = -1;
      } else {
        save_in_use = sub->in_use;
        for (i = sub->in_use; i < subidx; i++) {
          sub->list.multi[i].start_lnum = -1;
          sub->list.multi[i].end_lnum = -1;
        }
        sub->in_use = subidx + 1;
      }
      if (off == -1) {
        sub->list.multi[subidx].start_lnum = rex.lnum + 1;
        sub->list.multi[subidx].start_col = 0;
      } else {
        sub->list.multi[subidx].start_lnum = rex.lnum;
        sub->list.multi[subidx].start_col =
          (colnr_T)(rex.input - rex.line + off);
      }
      sub->list.multi[subidx].end_lnum = -1;
    } else {
      if (subidx < sub->in_use) {
        save_ptr = sub->list.line[subidx].start;
        save_in_use = -1;
      } else {
        save_in_use = sub->in_use;
        for (i = sub->in_use; i < subidx; i++) {
          sub->list.line[i].start = NULL;
          sub->list.line[i].end = NULL;
        }
        sub->in_use = subidx + 1;
      }
      sub->list.line[subidx].start = rex.input + off;
    }

    subs = addstate(l, state->out, subs, pim, off_arg);
    if (subs == NULL) {
      break;
    }
    // "subs" may have changed, need to set "sub" again.
    if (state->c >= NFA_ZOPEN && state->c <= NFA_ZOPEN9) {
      sub = &subs->synt;
    } else {
      sub = &subs->norm;
    }

    if (save_in_use == -1) {
      if (REG_MULTI) {
        sub->list.multi[subidx] = save_multipos;
      } else {
        sub->list.line[subidx].start = save_ptr;
      }
    } else {
      sub->in_use = save_in_use;
    }
    break;

  case NFA_MCLOSE:
    if (rex.nfa_has_zend
        && (REG_MULTI
            ? subs->norm.list.multi[0].end_lnum >= 0
            : subs->norm.list.line[0].end != NULL)) {
      // Do not overwrite the position set by \ze.
      subs = addstate(l, state->out, subs, pim, off_arg);
      break;
    }
    FALLTHROUGH;
  case NFA_MCLOSE1:
  case NFA_MCLOSE2:
  case NFA_MCLOSE3:
  case NFA_MCLOSE4:
  case NFA_MCLOSE5:
  case NFA_MCLOSE6:
  case NFA_MCLOSE7:
  case NFA_MCLOSE8:
  case NFA_MCLOSE9:
  case NFA_ZCLOSE:
  case NFA_ZCLOSE1:
  case NFA_ZCLOSE2:
  case NFA_ZCLOSE3:
  case NFA_ZCLOSE4:
  case NFA_ZCLOSE5:
  case NFA_ZCLOSE6:
  case NFA_ZCLOSE7:
  case NFA_ZCLOSE8:
  case NFA_ZCLOSE9:
  case NFA_ZEND:
    if (state->c == NFA_ZEND) {
      subidx = 0;
      sub = &subs->norm;
    } else if (state->c >= NFA_ZCLOSE && state->c <= NFA_ZCLOSE9) {
      subidx = state->c - NFA_ZCLOSE;
      sub = &subs->synt;
    } else {
      subidx = state->c - NFA_MCLOSE;
      sub = &subs->norm;
    }

    // We don't fill in gaps here, there must have been an MOPEN that
    // has done that.
    save_in_use = sub->in_use;
    if (sub->in_use <= subidx) {
      sub->in_use = subidx + 1;
    }
    if (REG_MULTI) {
      save_multipos = sub->list.multi[subidx];
      if (off == -1) {
        sub->list.multi[subidx].end_lnum = rex.lnum + 1;
        sub->list.multi[subidx].end_col = 0;
      } else {
        sub->list.multi[subidx].end_lnum = rex.lnum;
        sub->list.multi[subidx].end_col =
          (colnr_T)(rex.input - rex.line + off);
      }
      // avoid compiler warnings
      save_ptr = NULL;
    } else {
      save_ptr = sub->list.line[subidx].end;
      sub->list.line[subidx].end = rex.input + off;
      // avoid compiler warnings
      CLEAR_FIELD(save_multipos);
    }

    subs = addstate(l, state->out, subs, pim, off_arg);
    if (subs == NULL) {
      break;
    }
    // "subs" may have changed, need to set "sub" again.
    if (state->c >= NFA_ZCLOSE && state->c <= NFA_ZCLOSE9) {
      sub = &subs->synt;
    } else {
      sub = &subs->norm;
    }

    if (REG_MULTI) {
      sub->list.multi[subidx] = save_multipos;
    } else {
      sub->list.line[subidx].end = save_ptr;
    }
    sub->in_use = save_in_use;
    break;
  }
  depth--;
  return subs;
}

/// Like addstate(), but the new state(s) are put at position "*ip".
/// Used for zero-width matches, next state to use is the added one.
/// This makes sure the order of states to be tried does not change, which
/// matters for alternatives.
///
/// @param l      runtime state list
/// @param state  state to update
/// @param subs   pointers to subexpressions
/// @param pim    postponed look-behind match
extern regsubs_T *rs_addstate_here(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs,
                                   nfa_pim_T *pim, int *ip);

static regsubs_T *addstate_here(nfa_list_T *l, nfa_state_T *state, regsubs_T *subs, nfa_pim_T *pim,
                                int *ip)
  FUNC_ATTR_NONNULL_ARG(1, 2, 5) FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_addstate_here(l, state, subs, pim, ip);
}

// Check character class "class" against current character c.
static int check_char_class(int cls, int c)
{
  return rs_check_char_class(cls, c);
}

/// Check for a match with subexpression "subidx".
///
/// @param sub      pointers to subexpressions
/// @param bytelen  out: length of match in bytes
///
/// @return  true if it matches.
static int match_backref(regsub_T *sub, int subidx, int *bytelen)
{
  int len;

  if (sub->in_use <= subidx) {
retempty:
    // backref was not set, match an empty string
    *bytelen = 0;
    return true;
  }

  if (REG_MULTI) {
    if (sub->list.multi[subidx].start_lnum < 0
        || sub->list.multi[subidx].end_lnum < 0) {
      goto retempty;
    }
    if (sub->list.multi[subidx].start_lnum == rex.lnum
        && sub->list.multi[subidx].end_lnum == rex.lnum) {
      len = sub->list.multi[subidx].end_col
            - sub->list.multi[subidx].start_col;
      if (cstrncmp((char *)rex.line + sub->list.multi[subidx].start_col,
                   (char *)rex.input, &len) == 0) {
        *bytelen = len;
        return true;
      }
    } else {
      if (match_with_backref(sub->list.multi[subidx].start_lnum,
                             sub->list.multi[subidx].start_col,
                             sub->list.multi[subidx].end_lnum,
                             sub->list.multi[subidx].end_col,
                             bytelen) == RA_MATCH) {
        return true;
      }
    }
  } else {
    if (sub->list.line[subidx].start == NULL
        || sub->list.line[subidx].end == NULL) {
      goto retempty;
    }
    len = (int)(sub->list.line[subidx].end - sub->list.line[subidx].start);
    if (cstrncmp((char *)sub->list.line[subidx].start, (char *)rex.input, &len) == 0) {
      *bytelen = len;
      return true;
    }
  }
  return false;
}

/// Check for a match with \z subexpression "subidx".
///
/// @param bytelen  out: length of match in bytes
///
/// @return  true if it matches.
static int match_zref(int subidx, int *bytelen)
{
  int len;

  cleanup_zsubexpr();
  if (re_extmatch_in == NULL || re_extmatch_in->matches[subidx] == NULL) {
    // backref was not set, match an empty string
    *bytelen = 0;
    return true;
  }

  len = (int)strlen((char *)re_extmatch_in->matches[subidx]);
  if (cstrncmp((char *)re_extmatch_in->matches[subidx], (char *)rex.input, &len) == 0) {
    *bytelen = len;
    return true;
  }
  return false;
}

// Save list IDs for all NFA states of "prog" into "list".
// Also reset the IDs to zero.
// Only used for the recursive value lastlist[1].
static void nfa_save_listids(nfa_regprog_T *prog, int *list)
{
  int i;
  nfa_state_T *p;

  // Order in the list is reverse, it's a bit faster that way.
  p = &prog->state[0];
  for (i = prog->nstate; --i >= 0;) {
    list[i] = p->lastlist[1];
    p->lastlist[1] = 0;
    p++;
  }
}

// Restore list IDs from "list" to all NFA states.
static void nfa_restore_listids(nfa_regprog_T *prog, const int *list)
{
  int i;
  nfa_state_T *p;

  p = &prog->state[0];
  for (i = prog->nstate; --i >= 0;) {
    p->lastlist[1] = list[i];
    p++;
  }
}

// Rust implementation for nfa_re_num_cmp
extern int rs_nfa_re_num_cmp(uint64_t val, int op, uint64_t pos);

static bool nfa_re_num_cmp(uintmax_t val, int op, uintmax_t pos)
{
  return rs_nfa_re_num_cmp((uint64_t)val, op, (uint64_t)pos) != 0;
}

// Recursively call nfa_regmatch() - now calls Rust implementation
extern int rs_recursive_regmatch(nfa_state_T *state, const nfa_pim_T *pim,
                                 void *prog, regsubs_T *submatch,
                                 regsubs_T *m, int **listids, int *listids_len);

static int recursive_regmatch(nfa_state_T *state, nfa_pim_T *pim, nfa_regprog_T *prog,
                              regsubs_T *submatch, regsubs_T *m, int **listids, int *listids_len)
  FUNC_ATTR_NONNULL_ARG(1, 3, 5, 6, 7)
{
  return rs_recursive_regmatch(state, pim, prog, submatch, m, listids, listids_len);
}

// Estimate the chance of a match with "state" failing.
// empty match: 0
// NFA_ANY: 1
// specific character: 99
static int failure_chance(nfa_state_T *state, int depth)
{
  int c = state->c;
  int l, r;

  // detect looping
  if (depth > 4) {
    return 1;
  }

  switch (c) {
  case NFA_SPLIT:
    if (state->out->c == NFA_SPLIT || state->out1->c == NFA_SPLIT) {
      // avoid recursive stuff
      return 1;
    }
    // two alternatives, use the lowest failure chance
    l = failure_chance(state->out, depth + 1);
    r = failure_chance(state->out1, depth + 1);
    return l < r ? l : r;

  case NFA_ANY:
    // matches anything, unlikely to fail
    return 1;

  case NFA_MATCH:
  case NFA_MCLOSE:
  case NFA_ANY_COMPOSING:
    // empty match works always
    return 0;

  case NFA_START_INVISIBLE:
  case NFA_START_INVISIBLE_FIRST:
  case NFA_START_INVISIBLE_NEG:
  case NFA_START_INVISIBLE_NEG_FIRST:
  case NFA_START_INVISIBLE_BEFORE:
  case NFA_START_INVISIBLE_BEFORE_FIRST:
  case NFA_START_INVISIBLE_BEFORE_NEG:
  case NFA_START_INVISIBLE_BEFORE_NEG_FIRST:
  case NFA_START_PATTERN:
    // recursive regmatch is expensive, use low failure chance
    return 5;

  case NFA_BOL:
  case NFA_EOL:
  case NFA_BOF:
  case NFA_EOF:
  case NFA_NEWL:
    return 99;

  case NFA_BOW:
  case NFA_EOW:
    return 90;

  case NFA_MOPEN:
  case NFA_MOPEN1:
  case NFA_MOPEN2:
  case NFA_MOPEN3:
  case NFA_MOPEN4:
  case NFA_MOPEN5:
  case NFA_MOPEN6:
  case NFA_MOPEN7:
  case NFA_MOPEN8:
  case NFA_MOPEN9:
  case NFA_ZOPEN:
  case NFA_ZOPEN1:
  case NFA_ZOPEN2:
  case NFA_ZOPEN3:
  case NFA_ZOPEN4:
  case NFA_ZOPEN5:
  case NFA_ZOPEN6:
  case NFA_ZOPEN7:
  case NFA_ZOPEN8:
  case NFA_ZOPEN9:
  case NFA_ZCLOSE:
  case NFA_ZCLOSE1:
  case NFA_ZCLOSE2:
  case NFA_ZCLOSE3:
  case NFA_ZCLOSE4:
  case NFA_ZCLOSE5:
  case NFA_ZCLOSE6:
  case NFA_ZCLOSE7:
  case NFA_ZCLOSE8:
  case NFA_ZCLOSE9:
  case NFA_NOPEN:
  case NFA_MCLOSE1:
  case NFA_MCLOSE2:
  case NFA_MCLOSE3:
  case NFA_MCLOSE4:
  case NFA_MCLOSE5:
  case NFA_MCLOSE6:
  case NFA_MCLOSE7:
  case NFA_MCLOSE8:
  case NFA_MCLOSE9:
  case NFA_NCLOSE:
    return failure_chance(state->out, depth + 1);

  case NFA_BACKREF1:
  case NFA_BACKREF2:
  case NFA_BACKREF3:
  case NFA_BACKREF4:
  case NFA_BACKREF5:
  case NFA_BACKREF6:
  case NFA_BACKREF7:
  case NFA_BACKREF8:
  case NFA_BACKREF9:
  case NFA_ZREF1:
  case NFA_ZREF2:
  case NFA_ZREF3:
  case NFA_ZREF4:
  case NFA_ZREF5:
  case NFA_ZREF6:
  case NFA_ZREF7:
  case NFA_ZREF8:
  case NFA_ZREF9:
    // backreferences don't match in many places
    return 94;

  case NFA_LNUM_GT:
  case NFA_LNUM_LT:
  case NFA_COL_GT:
  case NFA_COL_LT:
  case NFA_VCOL_GT:
  case NFA_VCOL_LT:
  case NFA_MARK_GT:
  case NFA_MARK_LT:
  case NFA_VISUAL:
    // before/after positions don't match very often
    return 85;

  case NFA_LNUM:
    return 90;

  case NFA_CURSOR:
  case NFA_COL:
  case NFA_VCOL:
  case NFA_MARK:
    // specific positions rarely match
    return 98;

  case NFA_COMPOSING:
    return 95;

  default:
    if (c > 0) {
      // character match fails often
      return 95;
    }
  }

  // something else, includes character classes
  return 50;
}

// Skip until the char "c" we know a match must start with.
static int skip_to_start(int c, colnr_T *colp)
{
  return rs_skip_to_start(c, colp);
}

// Check for a match with match_text.
// Called after skip_to_start() has found regstart.
// Returns zero for no match, 1 for a match.
static int find_match_text(colnr_T *startcol, int regstart, uint8_t *match_text)
{
  return rs_find_match_text(startcol, regstart, match_text);
}

static int nfa_did_time_out(void)
{
  return rs_nfa_did_time_out();
}

// Exposed for Rust - wraps static nfa_did_time_out()
int nvim_nfa_did_time_out(void) { return nfa_did_time_out(); }

/// Main matching routine.
///
/// Run NFA to determine whether it matches rex.input.
///
/// When "nfa_endp" is not NULL it is a required end-of-match position.
///
/// Return true if there is a match, false if there is no match,
/// NFA_TOO_EXPENSIVE if we end up with too many states.
/// When there is a match "submatch" contains the positions.
///
/// Note: Caller must ensure that: start != NULL.
static int nfa_regmatch(nfa_regprog_T *prog, nfa_state_T *start, regsubs_T *submatch, regsubs_T *m)
  FUNC_ATTR_NONNULL_ARG(1, 2, 4)
{
  int result = false;
  int flag = 0;
  bool go_to_nextline = false;
  nfa_thread_T *t;
  nfa_list_T list[2];
  int listidx;
  nfa_list_T *thislist;
  nfa_list_T *nextlist;
  int *listids = NULL;
  int listids_len = 0;
  nfa_state_T *add_state;
  bool add_here;
  int add_count;
  int add_off = 0;
  int toplevel = start->c == NFA_MOPEN;
  regsubs_T *r;
  // Some patterns may take a long time to match, especially when using
  // recursive_regmatch(). Allow interrupting them with CTRL-C.
  reg_breakcheck();
  if (got_int) {
    return false;
  }
  if (nfa_did_time_out()) {
    return false;
  }

#ifdef NFA_REGEXP_DEBUG_LOG
  FILE *debug = fopen(NFA_REGEXP_DEBUG_LOG, "a");

  if (debug == NULL) {
    semsg("(NFA) COULD NOT OPEN %s!", NFA_REGEXP_DEBUG_LOG);
    return false;
  }
#endif
  nfa_match = false;

  // Allocate memory for the lists of nodes.
  size_t size = (size_t)(prog->nstate + 1) * sizeof(nfa_thread_T);
  list[0].t = xmalloc(size);
  list[0].len = prog->nstate + 1;
  list[1].t = xmalloc(size);
  list[1].len = prog->nstate + 1;

#ifdef REGEXP_DEBUG
  log_fd = fopen(NFA_REGEXP_RUN_LOG, "a");
  if (log_fd == NULL) {
    emsg(_(e_log_open_failed));
    log_fd = stderr;
  }
  fprintf(log_fd, "**********************************\n");
  nfa_set_code(start->c);
  fprintf(log_fd, " RUNNING nfa_regmatch() starting with state %d, code %s\n",
          abs(start->id), code);
  fprintf(log_fd, "**********************************\n");
#endif

  thislist = &list[0];
  thislist->n = 0;
  thislist->has_pim = false;
  nextlist = &list[1];
  nextlist->n = 0;
  nextlist->has_pim = false;
#ifdef REGEXP_DEBUG
  fprintf(log_fd, "(---) STARTSTATE first\n");
#endif
  thislist->id = rex.nfa_listid + 1;

  // Inline optimized code for addstate(thislist, start, m, 0) if we know
  // it's the first MOPEN.
  if (toplevel) {
    if (REG_MULTI) {
      m->norm.list.multi[0].start_lnum = rex.lnum;
      m->norm.list.multi[0].start_col = (colnr_T)(rex.input - rex.line);
      m->norm.orig_start_col = m->norm.list.multi[0].start_col;
    } else {
      m->norm.list.line[0].start = rex.input;
    }
    m->norm.in_use = 1;
    r = addstate(thislist, start->out, m, NULL, 0);
  } else {
    r = addstate(thislist, start, m, NULL, 0);
  }
  if (r == NULL) {
    nfa_match = NFA_TOO_EXPENSIVE;
    goto theend;
  }

#define ADD_STATE_IF_MATCH(state) \
  if (result) { \
    add_state = (state)->out; \
    add_off = clen; \
  }

  // Run for each character.
  while (true) {
    int curc = utf_ptr2char((char *)rex.input);
    int clen = utfc_ptr2len((char *)rex.input);
    if (curc == NUL) {
      clen = 0;
      go_to_nextline = false;
    }

    // swap lists
    thislist = &list[flag];
    nextlist = &list[flag ^= 1];
    nextlist->n = 0;                // clear nextlist
    nextlist->has_pim = false;
    rex.nfa_listid++;
    if (prog->re_engine == AUTOMATIC_ENGINE
        && (rex.nfa_listid >= NFA_MAX_STATES)) {
      // Too many states, retry with old engine.
      nfa_match = NFA_TOO_EXPENSIVE;
      goto theend;
    }

    thislist->id = rex.nfa_listid;
    nextlist->id = rex.nfa_listid + 1;

#ifdef REGEXP_DEBUG
    fprintf(log_fd, "------------------------------------------\n");
    fprintf(log_fd, ">>> Reginput is \"%s\"\n", rex.input);
    fprintf(log_fd,
            ">>> Advanced one character... Current char is %c (code %d) \n",
            curc,
            (int)curc);
    fprintf(log_fd, ">>> Thislist has %d states available: ", thislist->n);
    {
      int i;

      for (i = 0; i < thislist->n; i++) {
        fprintf(log_fd, "%d  ", abs(thislist->t[i].state->id));
      }
    }
    fprintf(log_fd, "\n");
#endif

#ifdef NFA_REGEXP_DEBUG_LOG
    fprintf(debug, "\n-------------------\n");
#endif
    // If the state lists are empty we can stop.
    if (thislist->n == 0) {
      break;
    }

    // compute nextlist
    for (listidx = 0; listidx < thislist->n; listidx++) {
      // If the list gets very long there probably is something wrong.
      // At least allow interrupting with CTRL-C.
      reg_breakcheck();
      if (got_int) {
        break;
      }
      if (nfa_time_limit != NULL && ++nfa_time_count == 20) {
        nfa_time_count = 0;
        if (nfa_did_time_out()) {
          break;
        }
      }
      t = &thislist->t[listidx];

#ifdef NFA_REGEXP_DEBUG_LOG
      nfa_set_code(t->state->c);
      fprintf(debug, "%s, ", code);
#endif
#ifdef REGEXP_DEBUG
      {
        int col;

        if (t->subs.norm.in_use <= 0) {
          col = -1;
        } else if (REG_MULTI) {
          col = t->subs.norm.list.multi[0].start_col;
        } else {
          col = (int)(t->subs.norm.list.line[0].start - rex.line);
        }
        nfa_set_code(t->state->c);
        fprintf(log_fd, "(%d) char %d %s (start col %d)%s... \n",
                abs(t->state->id), (int)t->state->c, code, col,
                pim_info(&t->pim));
      }
#endif

      // Handle the possible codes of the current state.
      // The most important is NFA_MATCH.
      add_state = NULL;
      add_here = false;
      add_count = 0;

      // Try Rust state processing first
      {
        int add_off_rs = 0;
        int add_here_rs = 0;
        int add_count_rs = 0;
        nfa_state_T *add_state_rs = NULL;
        int rs_result = rs_nfa_process_state(
            t, curc, clen,
            prog, thislist, nextlist,
            start, submatch, m,
            &listids, &listids_len,
            &listidx,
            (void **)&add_state_rs, &add_here_rs, &add_count_rs, &add_off_rs);

        if (rs_result == 2) {
          // Rust handled NFA_MATCH, goto nextchar
          if (nextlist->n == 0) {
            clen = 0;
          }
          goto nextchar;
        } else if (rs_result == 3) {
          // Rust fully handled this state (e.g., addstate_here was called)
          // Skip to next iteration, don't go through state_handled
          continue;
        } else if (rs_result == -1) {
          // Error (NFA_TOO_EXPENSIVE)
          nfa_match = NFA_TOO_EXPENSIVE;
          goto theend;
        } else if (add_state_rs != NULL) {
          // Rust handled this state
          add_state = add_state_rs;
          add_here = add_here_rs;
          add_count = add_count_rs;
          add_off = add_off_rs;
          // NFA_NEWL uses add_off = -1 to signal go_to_nextline
          if (add_off_rs == -1) {
            go_to_nextline = true;
          }
          goto state_handled;
        }
        // Otherwise fall through to C switch
      }

      switch (t->state->c) {
      // NFA_MATCH is handled by Rust (rs_nfa_process_state)

      // NFA_END_INVISIBLE, NFA_END_INVISIBLE_NEG, NFA_END_PATTERN are handled
      // by Rust (rs_nfa_process_state)

      // NFA_START_INVISIBLE* variants are handled by Rust (rs_nfa_process_state)

      // NFA_START_PATTERN is handled by Rust (rs_nfa_process_state)

      // Anchors (NFA_BOL, NFA_EOL, NFA_BOW, NFA_EOW, NFA_BOF, NFA_EOF)
      // are handled by Rust (rs_nfa_process_state)

      // NFA_COMPOSING is handled by Rust (rs_nfa_process_state)

      // NFA_NEWL is handled by Rust (rs_nfa_process_state)

      // NFA_START_COLL and NFA_START_NEG_COLL are handled by Rust (rs_nfa_process_state)

      // NFA_ANY and NFA_ANY_COMPOSING are handled by Rust (rs_nfa_process_state)

      // Character classes (NFA_IDENT through NFA_NUPPER_IC) are handled
      // by Rust (rs_nfa_process_state)

      // Backreferences (NFA_BACKREF1-9, NFA_ZREF1-9), NFA_SKIP, NFA_LNUM,
      // NFA_COL, NFA_CURSOR, NFA_VCOL, NFA_MARK, NFA_VISUAL are handled by Rust
      // (rs_nfa_process_state)

      // NFA_MOPEN1-9, NFA_ZOPEN, NFA_ZOPEN1-9, NFA_NOPEN, NFA_ZSTART:
      // These states are only added to be able to bail out when
      // they are added again, nothing is to be done. Handled by default case.

      default:
        // Literal character matching is handled by Rust (rs_nfa_process_state)
        // If we get here with a positive c, Rust should have handled it.
        // Negative values that aren't handled above are internal errors.
#ifdef REGEXP_DEBUG
        if (t->state->c < 0) {
          siemsg("INTERNAL: Negative state char: %" PRId64, (int64_t)t->state->c);
        }
#endif
        break;
      }       // switch (t->state->c)

state_handled:
      if (add_state != NULL) {
        nfa_pim_T *pim;
        nfa_pim_T pim_copy;

        if (t->pim.result == NFA_PIM_UNUSED) {
          pim = NULL;
        } else {
          pim = &t->pim;
        }

        // Handle the postponed invisible match if the match might end
        // without advancing and before the end of the line.
        if (pim != NULL && (clen == 0 || match_follows(add_state, 0))) {
          if (pim->result == NFA_PIM_TODO) {
#ifdef REGEXP_DEBUG
            fprintf(log_fd, "\n");
            fprintf(log_fd, "==================================\n");
            fprintf(log_fd, "Postponed recursive nfa_regmatch()\n");
            fprintf(log_fd, "\n");
#endif
            result = recursive_regmatch(pim->state, pim, prog, submatch, m,
                                        &listids, &listids_len);
            pim->result = result ? NFA_PIM_MATCH : NFA_PIM_NOMATCH;
            // for \@! and \@<! it is a match when the result is
            // false
            if (result != (pim->state->c == NFA_START_INVISIBLE_NEG
                           || pim->state->c == NFA_START_INVISIBLE_NEG_FIRST
                           || pim->state->c
                           == NFA_START_INVISIBLE_BEFORE_NEG
                           || pim->state->c
                           == NFA_START_INVISIBLE_BEFORE_NEG_FIRST)) {
              // Copy submatch info from the recursive call
              copy_sub_off(&pim->subs.norm, &m->norm);
              if (rex.nfa_has_zsubexpr) {
                copy_sub_off(&pim->subs.synt, &m->synt);
              }
            }
          } else {
            result = (pim->result == NFA_PIM_MATCH);
#ifdef REGEXP_DEBUG
            fprintf(log_fd, "\n");
            fprintf(log_fd,
                    "Using previous recursive nfa_regmatch() result, result == %d\n",
                    pim->result);
            fprintf(log_fd, "MATCH = %s\n", result ? "OK" : "false");
            fprintf(log_fd, "\n");
#endif
          }

          // for \@! and \@<! it is a match when result is false
          if (result != (pim->state->c == NFA_START_INVISIBLE_NEG
                         || pim->state->c == NFA_START_INVISIBLE_NEG_FIRST
                         || pim->state->c
                         == NFA_START_INVISIBLE_BEFORE_NEG
                         || pim->state->c
                         == NFA_START_INVISIBLE_BEFORE_NEG_FIRST)) {
            // Copy submatch info from the recursive call
            copy_sub_off(&t->subs.norm, &pim->subs.norm);
            if (rex.nfa_has_zsubexpr) {
              copy_sub_off(&t->subs.synt, &pim->subs.synt);
            }
          } else {
            // look-behind match failed, don't add the state
            continue;
          }

          // Postponed invisible match was handled, don't add it to
          // following states.
          pim = NULL;
        }

        // If "pim" points into l->t it will become invalid when
        // adding the state causes the list to be reallocated.  Make a
        // local copy to avoid that.
        if (pim == &t->pim) {
          copy_pim(&pim_copy, pim);
          pim = &pim_copy;
        }

        if (add_here) {
          r = addstate_here(thislist, add_state, &t->subs, pim, &listidx);
        } else {
          r = addstate(nextlist, add_state, &t->subs, pim, add_off);
          if (add_count > 0) {
            nextlist->t[nextlist->n - 1].count = add_count;
          }
        }
        if (r == NULL) {
          nfa_match = NFA_TOO_EXPENSIVE;
          goto theend;
        }
      }
    }     // for (thislist = thislist; thislist->state; thislist++)

    // Look for the start of a match in the current position by adding the
    // start state to the list of states.
    // The first found match is the leftmost one, thus the order of states
    // matters!
    // Do not add the start state in recursive calls of nfa_regmatch(),
    // because recursive calls should only start in the first position.
    // Unless "nfa_endp" is not NULL, then we match the end position.
    // Also don't start a match past the first line.
    if (!nfa_match
        && ((toplevel
             && rex.lnum == 0
             && clen != 0
             && (rex.reg_maxcol == 0
                 || (colnr_T)(rex.input - rex.line) < rex.reg_maxcol))
            || (nfa_endp != NULL
                && (REG_MULTI
                    ? (rex.lnum < nfa_endp->se_u.pos.lnum
                       || (rex.lnum == nfa_endp->se_u.pos.lnum
                           && (int)(rex.input - rex.line)
                           < nfa_endp->se_u.pos.col))
                    : rex.input < nfa_endp->se_u.ptr)))) {
#ifdef REGEXP_DEBUG
      fprintf(log_fd, "(---) STARTSTATE\n");
#endif
      // Inline optimized code for addstate() if we know the state is
      // the first MOPEN.
      if (toplevel) {
        int add = true;

        if (prog->regstart != NUL && clen != 0) {
          if (nextlist->n == 0) {
            colnr_T col = (colnr_T)(rex.input - rex.line) + clen;

            // Nextlist is empty, we can skip ahead to the
            // character that must appear at the start.
            if (skip_to_start(prog->regstart, &col) == FAIL) {
              break;
            }
#ifdef REGEXP_DEBUG
            fprintf(log_fd, "  Skipping ahead %d bytes to regstart\n",
                    col - ((colnr_T)(rex.input - rex.line) + clen));
#endif
            rex.input = rex.line + col - clen;
          } else {
            // Checking if the required start character matches is
            // cheaper than adding a state that won't match.
            const int c = utf_ptr2char((char *)rex.input + clen);
            if (c != prog->regstart
                && (!rex.reg_ic
                    || utf_fold(c) != utf_fold(prog->regstart))) {
#ifdef REGEXP_DEBUG
              fprintf(log_fd,
                      "  Skipping start state, regstart does not match\n");
#endif
              add = false;
            }
          }
        }

        if (add) {
          if (REG_MULTI) {
            m->norm.list.multi[0].start_col =
              (colnr_T)(rex.input - rex.line) + clen;
            m->norm.orig_start_col =
              m->norm.list.multi[0].start_col;
          } else {
            m->norm.list.line[0].start = rex.input + clen;
          }
          if (addstate(nextlist, start->out, m, NULL, clen) == NULL) {
            nfa_match = NFA_TOO_EXPENSIVE;
            goto theend;
          }
        }
      } else {
        if (addstate(nextlist, start, m, NULL, clen) == NULL) {
          nfa_match = NFA_TOO_EXPENSIVE;
          goto theend;
        }
      }
    }

#ifdef REGEXP_DEBUG
    fprintf(log_fd, ">>> Thislist had %d states available: ", thislist->n);
    {
      int i;

      for (i = 0; i < thislist->n; i++) {
        fprintf(log_fd, "%d  ", abs(thislist->t[i].state->id));
      }
    }
    fprintf(log_fd, "\n");
#endif

nextchar:
    // Advance to the next character, or advance to the next line, or
    // finish.
    if (clen != 0) {
      rex.input += clen;
    } else if (go_to_nextline || (nfa_endp != NULL && REG_MULTI
                                  && rex.lnum < nfa_endp->se_u.pos.lnum)) {
      reg_nextline();
    } else {
      break;
    }

    // Allow interrupting with CTRL-C.
    reg_breakcheck();
    if (got_int) {
      break;
    }
    // Check for timeout once every twenty times to avoid overhead.
    if (nfa_time_limit != NULL && ++nfa_time_count == 20) {
      nfa_time_count = 0;
      if (nfa_did_time_out()) {
        break;
      }
    }
  }

#ifdef REGEXP_DEBUG
  if (log_fd != stderr) {
    fclose(log_fd);
  }
  log_fd = NULL;
#endif

theend:
  // Free memory
  xfree(list[0].t);
  xfree(list[1].t);
  xfree(listids);
#undef ADD_STATE_IF_MATCH
#ifdef NFA_REGEXP_DEBUG_LOG
  fclose(debug);
#endif

  return nfa_match;
}

// Phase 12b helper: Handle \z() external matches for Rust nfa_regtry
// Packages found \z(...\) matches for export into re_extmatch_out.
static void nfa_handle_extmatch(nfa_regprog_T *prog, const regsub_T *subs_synt)
{
  unref_extmatch(re_extmatch_out);
  re_extmatch_out = NULL;

  if (prog->reghasz == REX_SET && subs_synt != NULL) {
    cleanup_zsubexpr();
    re_extmatch_out = make_extmatch();
    // Loop over \z1, \z2, etc.  There is no \z0.
    for (int i = 1; i < subs_synt->in_use; i++) {
      if (REG_MULTI) {
        struct multipos *mpos = &subs_synt->list.multi[i];

        // Only accept single line matches that are valid.
        if (mpos->start_lnum >= 0
            && mpos->start_lnum == mpos->end_lnum
            && mpos->end_col >= mpos->start_col) {
          re_extmatch_out->matches[i] =
            (uint8_t *)xstrnsave(reg_getline(mpos->start_lnum) + mpos->start_col,
                                 (size_t)(mpos->end_col - mpos->start_col));
        }
      } else {
        struct linepos *lpos = &subs_synt->list.line[i];

        if (lpos->start != NULL && lpos->end != NULL) {
          re_extmatch_out->matches[i] =
            (uint8_t *)xstrnsave((char *)lpos->start, (size_t)(lpos->end - lpos->start));
        }
      }
    }
  }
}

// Exposed wrapper for Rust (uses void* to avoid header export issues)
void nvim_nfa_handle_extmatch(void *prog, const void *subs_synt)
{
  nfa_handle_extmatch((nfa_regprog_T *)prog, (const regsub_T *)subs_synt);
}

// Rust implementation of nfa_regtry
extern int rs_nfa_regtry(nfa_regprog_T *prog, colnr_T col, proftime_T *tm, int *timed_out);

/// Try match of "prog" with at rex.line["col"].
///
/// @param tm         timeout limit or NULL
/// @param timed_out  flag set on timeout or NULL
///
/// @return  <= 0 for failure, number of lines contained in the match otherwise.
static int nfa_regtry(nfa_regprog_T *prog, colnr_T col, proftime_T *tm, int *timed_out)
{
  return rs_nfa_regtry(prog, col, tm, timed_out);
}

// Rust implementation of nfa_regexec_both
extern int rs_nfa_regexec_both(uint8_t *line, colnr_T startcol, proftime_T *tm, int *timed_out);

// Compile a regular expression into internal code for the NFA matcher.
// Returns the program in allocated space.  Returns NULL for an error.
static regprog_T *nfa_regcomp(uint8_t *expr, int re_flags)
{
  nfa_regprog_T *prog = NULL;
  int *postfix;

  if (expr == NULL) {
    return NULL;
  }

#ifdef REGEXP_DEBUG
  nfa_regengine.expr = expr;
#endif
  nfa_re_flags = re_flags;

  init_class_tab();

  nfa_regcomp_start(expr, re_flags);

  // Build postfix form of the regexp. Needed to build the NFA
  // (and count its size).
  postfix = re2post();
  if (postfix == NULL) {
    goto fail;              // Cascaded (syntax?) error
  }

  // In order to build the NFA, we parse the input regexp twice:
  // 1. first pass to count size (so we can allocate space)
  // 2. second to emit code
#ifdef REGEXP_DEBUG
  {
    FILE *f = fopen(NFA_REGEXP_RUN_LOG, "a");

    if (f != NULL) {
      fprintf(f,
              "\n*****************************\n\n\n\n\t"
              "Compiling regexp \"%s\"... hold on !\n",
              expr);
      fclose(f);
    }
  }
#endif

  // PASS 1
  // Count number of NFA states in "nstate". Do not build the NFA.
  int out_nstate = 0;
  rs_post2nfa(postfix, post_ptr, 1, NULL, 0, &out_nstate);
  nstate = out_nstate;

  // allocate the regprog with space for the compiled regexp
  size_t prog_size = offsetof(nfa_regprog_T, state) + sizeof(nfa_state_T) * (size_t)nstate;
  prog = xmalloc(prog_size);
  prog->re_in_use = false;

  // PASS 2
  // Build the NFA
  prog->start = rs_post2nfa(postfix, post_ptr, 0, prog->state, nstate, NULL);
  if (prog->start == NULL) {
    goto fail;
  }
  prog->regflags = regflags;
  prog->engine = &nfa_regengine;
  prog->nstate = nstate;
  prog->has_zend = rex.nfa_has_zend;
  prog->has_backref = rex.nfa_has_backref;
  prog->nsubexp = regnpar;

  rs_nfa_postprocess(prog);

  prog->reganch = rs_nfa_get_reganch_full(prog->start, 0);
  prog->regstart = rs_nfa_get_regstart_full(prog->start, 0);
  prog->match_text = rs_nfa_get_match_text_full(prog->start);

#ifdef REGEXP_DEBUG
  nfa_postfix_dump(expr, OK);
  nfa_dump(prog);
#endif
  // Remember whether this pattern has any \z specials in it.
  prog->reghasz = re_has_z;
  prog->pattern = xstrdup((char *)expr);
#ifdef REGEXP_DEBUG
  nfa_regengine.expr = NULL;
#endif

out:
  xfree(post_start);
  post_start = post_ptr = post_end = NULL;
  return (regprog_T *)prog;

fail:
  XFREE_CLEAR(prog);
#ifdef REGEXP_DEBUG
  nfa_postfix_dump(expr, FAIL);
  nfa_regengine.expr = NULL;
#endif
  goto out;
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

/// Match a regexp against a string.
/// "rmp->regprog" is a compiled regexp as returned by nfa_regcomp().
/// Uses curbuf for line count and 'iskeyword'.
/// If "line_lbr" is true, consider a "\n" in "line" to be a line break.
///
/// @param line  string to match against
/// @param col   column to start looking for match
///
/// @return  <= 0 for failure, number of lines contained in the match otherwise.
static int nfa_regexec_nl(regmatch_T *rmp, uint8_t *line, colnr_T col, bool line_lbr)
{
  rex.reg_match = rmp;
  rex.reg_mmatch = NULL;
  rex.reg_maxline = 0;
  rex.reg_line_lbr = line_lbr;
  rex.reg_buf = curbuf;
  rex.reg_win = NULL;
  rex.reg_ic = rmp->rm_ic;
  rex.reg_icombine = false;
  rex.reg_nobreak = rmp->regprog->re_flags & RE_NOBREAK;
  rex.reg_maxcol = 0;
  return rs_nfa_regexec_both(line, col, NULL, NULL);
}

/// Matches a regexp against multiple lines.
/// "rmp->regprog" is a compiled regexp as returned by vim_regcomp().
/// Uses curbuf for line count and 'iskeyword'.
///
/// @param win Window in which to search or NULL
/// @param buf Buffer in which to search
/// @param lnum Number of line to start looking for match
/// @param col Column to start looking for match
/// @param tm Timeout limit or NULL
/// @param timed_out Flag set on timeout or NULL
///
/// @return <= 0 if there is no match and number of lines contained in the match
/// otherwise.
///
/// @note The body is the same as bt_regexec() except for nfa_regexec_both()
///
/// @warning
/// Match may actually be in another line. e.g.:
/// when r.e. is \nc, cursor is at 'a' and the text buffer looks like
///
/// @par
///
///     +-------------------------+
///     |a                        |
///     |b                        |
///     |c                        |
///     |                         |
///     +-------------------------+
///
/// @par
/// then nfa_regexec_multi() returns 3. while the original vim_regexec_multi()
/// returns 0 and a second call at line 2 will return 2.
///
/// @par
/// FIXME if this behavior is not compatible.
static int nfa_regexec_multi(regmmatch_T *rmp, win_T *win, buf_T *buf, linenr_T lnum, colnr_T col,
                             proftime_T *tm, int *timed_out)
{
  init_regexec_multi(rmp, win, buf, lnum);
  return rs_nfa_regexec_both(NULL, col, tm, timed_out);
}
// }}}1

static regengine_T bt_regengine = {
  bt_regcomp,
  bt_regfree,
  bt_regexec_nl,
  bt_regexec_multi,
#ifdef REGEXP_DEBUG
  "",
#endif
};

static regengine_T nfa_regengine = {
  nfa_regcomp,
  nfa_regfree,
  nfa_regexec_nl,
  nfa_regexec_multi,
#ifdef REGEXP_DEBUG
  "",
#endif
};

// Which regexp engine to use? Needed for vim_regcomp().
// Must match with 'regexpengine'.
static int regexp_engine = 0;

#ifdef REGEXP_DEBUG
static uint8_t regname[][30] = {
  "AUTOMATIC Regexp Engine",
  "BACKTRACKING Regexp Engine",
  "NFA Regexp Engine"
};
#endif

// Compile a regular expression into internal code.
// Returns the program in allocated memory.
// Use vim_regfree() to free the memory.
// Returns NULL for an error.
regprog_T *vim_regcomp(const char *expr_arg, int re_flags)
{
  regprog_T *prog = NULL;
  const char *expr = expr_arg;

  regexp_engine = (int)p_re;

  // Check for prefix "\%#=", that sets the regexp engine
  if (strncmp(expr, "\\%#=", 4) == 0) {
    int newengine = expr[4] - '0';

    if (newengine == AUTOMATIC_ENGINE
        || newengine == BACKTRACKING_ENGINE
        || newengine == NFA_ENGINE) {
      regexp_engine = expr[4] - '0';
      expr += 5;
#ifdef REGEXP_DEBUG
      smsg(0, "New regexp mode selected (%d): %s",
           regexp_engine,
           regname[newengine]);
#endif
    } else {
      emsg(_("E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used "));
      regexp_engine = AUTOMATIC_ENGINE;
    }
  }
#ifdef REGEXP_DEBUG
  bt_regengine.expr = expr;
  nfa_regengine.expr = expr;
#endif
  // reg_iswordc() uses rex.reg_buf
  rex.reg_buf = curbuf;

  //
  // First try the NFA engine, unless backtracking was requested.
  //
  const int called_emsg_before = called_emsg;
  if (regexp_engine != BACKTRACKING_ENGINE) {
    prog = nfa_regengine.regcomp((uint8_t *)expr,
                                 re_flags + (regexp_engine == AUTOMATIC_ENGINE ? RE_AUTO : 0));
  } else {
    prog = bt_regengine.regcomp((uint8_t *)expr, re_flags);
  }

  // Check for error compiling regexp with initial engine.
  if (prog == NULL) {
#ifdef BT_REGEXP_DEBUG_LOG
    // Debugging log for BT engine.
    if (regexp_engine != BACKTRACKING_ENGINE) {
      FILE *f = fopen(BT_REGEXP_DEBUG_LOG_NAME, "a");
      if (f) {
        fprintf(f, "Syntax error in \"%s\"\n", expr);
        fclose(f);
      } else {
        semsg("(NFA) Could not open \"%s\" to write !!!",
              BT_REGEXP_DEBUG_LOG_NAME);
      }
    }
#endif
    // If the NFA engine failed, try the backtracking engine. The NFA engine
    // also fails for patterns that it can't handle well but are still valid
    // patterns, thus a retry should work.
    // But don't try if an error message was given.
    if (regexp_engine == AUTOMATIC_ENGINE && called_emsg == called_emsg_before) {
      regexp_engine = BACKTRACKING_ENGINE;
      report_re_switch(expr);
      prog = bt_regengine.regcomp((uint8_t *)expr, re_flags);
    }
  }

  if (prog != NULL) {
    // Store the info needed to call regcomp() again when the engine turns out
    // to be very slow when executing it.
    prog->re_engine = (unsigned)regexp_engine;
    prog->re_flags = (unsigned)re_flags;
  }

  return prog;
}

// Free a compiled regexp program, returned by vim_regcomp().
void vim_regfree(regprog_T *prog)
{
  if (prog != NULL) {
    prog->engine->regfree(prog);
  }
}

#if defined(EXITFREE)
void free_regexp_stuff(void)
{
  ga_clear(&regstack);
  ga_clear(&backpos);
  xfree(reg_tofree);
  xfree(reg_prev_sub);
}

#endif

static void report_re_switch(const char *pat)
{
  if (p_verbose > 0) {
    verbose_enter();
    msg_puts(_("Switching to backtracking RE engine for pattern: "));
    msg_puts(pat);
    verbose_leave();
  }
}

/// Match a regexp against a string.
/// "rmp->regprog" must be a compiled regexp as returned by vim_regcomp().
/// Note: "rmp->regprog" may be freed and changed.
/// Uses curbuf for line count and 'iskeyword'.
/// When "nl" is true consider a "\n" in "line" to be a line break.
///
/// @param rmp
/// @param line the string to match against
/// @param col  the column to start looking for match
/// @param nl
///
/// @return true if there is a match, false if not.
static bool vim_regexec_string(regmatch_T *rmp, const char *line, colnr_T col, bool nl)
{
  regexec_T rex_save;
  bool rex_in_use_save = rex_in_use;

  // Cannot use the same prog recursively, it contains state.
  if (rmp->regprog->re_in_use) {
    emsg(_(e_recursive));
    return false;
  }
  rmp->regprog->re_in_use = true;

  if (rex_in_use) {
    // Being called recursively, save the state.
    rex_save = rex;
  }
  rex_in_use = true;

  rex.reg_startp = NULL;
  rex.reg_endp = NULL;
  rex.reg_startpos = NULL;
  rex.reg_endpos = NULL;

  int result = rmp->regprog->engine->regexec_nl(rmp, (uint8_t *)line, col, nl);
  rmp->regprog->re_in_use = false;

  // NFA engine aborted because it's very slow, use backtracking engine instead.
  if (rmp->regprog->re_engine == AUTOMATIC_ENGINE
      && result == NFA_TOO_EXPENSIVE) {
    int save_p_re = (int)p_re;
    int re_flags = (int)rmp->regprog->re_flags;
    char *pat = xstrdup(((nfa_regprog_T *)rmp->regprog)->pattern);

    p_re = BACKTRACKING_ENGINE;
    vim_regfree(rmp->regprog);
    report_re_switch(pat);
    rmp->regprog = vim_regcomp(pat, re_flags);
    if (rmp->regprog != NULL) {
      rmp->regprog->re_in_use = true;
      result = rmp->regprog->engine->regexec_nl(rmp, (uint8_t *)line, col, nl);
      rmp->regprog->re_in_use = false;
    }

    xfree(pat);
    p_re = save_p_re;
  }

  rex_in_use = rex_in_use_save;
  if (rex_in_use) {
    rex = rex_save;
  }

  return result > 0;
}

// Note: "*prog" may be freed and changed.
// Return true if there is a match, false if not.
bool vim_regexec_prog(regprog_T **prog, bool ignore_case, const char *line, colnr_T col)
{
  regmatch_T regmatch = { .regprog = *prog, .rm_ic = ignore_case };
  bool r = vim_regexec_string(&regmatch, line, col, false);
  *prog = regmatch.regprog;
  return r;
}

// Note: "rmp->regprog" may be freed and changed.
// Return true if there is a match, false if not.
bool vim_regexec(regmatch_T *rmp, const char *line, colnr_T col)
{
  return vim_regexec_string(rmp, line, col, false);
}

// Like vim_regexec(), but consider a "\n" in "line" to be a line break.
// Note: "rmp->regprog" may be freed and changed.
// Return true if there is a match, false if not.
bool vim_regexec_nl(regmatch_T *rmp, const char *line, colnr_T col)
{
  return vim_regexec_string(rmp, line, col, true);
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
  regexec_T rex_save;
  bool rex_in_use_save = rex_in_use;

  // Cannot use the same prog recursively, it contains state.
  if (rmp->regprog->re_in_use) {
    emsg(_(e_recursive));
    return false;
  }
  rmp->regprog->re_in_use = true;

  if (rex_in_use) {
    // Being called recursively, save the state.
    rex_save = rex;
  }
  rex_in_use = true;

  int result = rmp->regprog->engine->regexec_multi(rmp, win, buf, lnum, col, tm, timed_out);
  rmp->regprog->re_in_use = false;

  // NFA engine aborted because it's very slow, use backtracking engine instead.
  if (rmp->regprog->re_engine == AUTOMATIC_ENGINE
      && result == NFA_TOO_EXPENSIVE) {
    int save_p_re = (int)p_re;
    int re_flags = (int)rmp->regprog->re_flags;
    char *pat = xstrdup(((nfa_regprog_T *)rmp->regprog)->pattern);

    p_re = BACKTRACKING_ENGINE;
    regprog_T *prev_prog = rmp->regprog;

    report_re_switch(pat);
    // checking for \z misuse was already done when compiling for NFA,
    // allow all here
    reg_do_extmatch = REX_ALL;
    rmp->regprog = vim_regcomp(pat, re_flags);
    reg_do_extmatch = 0;

    if (rmp->regprog == NULL) {
      // Somehow compiling the pattern failed now, put back the
      // previous one to avoid "regprog" becoming NULL.
      rmp->regprog = prev_prog;
    } else {
      vim_regfree(prev_prog);

      rmp->regprog->re_in_use = true;
      result = rmp->regprog->engine->regexec_multi(rmp, win, buf, lnum, col, tm, timed_out);
      rmp->regprog->re_in_use = false;
    }

    xfree(pat);
    p_re = save_p_re;
  }

  rex_in_use = rex_in_use_save;
  if (rex_in_use) {
    rex = rex_save;
  }

  return result <= 0 ? 0 : result;
}
