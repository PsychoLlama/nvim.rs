#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/edit.h"
#include "nvim/eval/typval.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/keycodes.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/math.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"

extern const char *rs_skip_to_option_part(const char *p);
extern bool rs_cindent_on(void);

// Phase 137: C indentation Rust helpers
// Comment detection
extern bool rs_cin_iscomment(const char *p);
extern bool rs_cin_islinecomment(const char *p);
extern bool rs_cin_ispreproc(const char *s);
extern const char *rs_cin_skipcomment(const char *s);
extern bool rs_cin_nocode(const char *s);
// String handling
extern const char *rs_skip_string(const char *p);
extern bool rs_is_pos_in_string(const char *line, int col);
// Keyword detection
extern bool rs_cin_starts_with(const char *s, const char *word);
extern bool rs_cin_isif(const char *p);
extern bool rs_cin_iselse(const char *p);
extern bool rs_cin_isdo(const char *p);
extern bool rs_cin_isbreak(const char *p);
extern bool rs_cin_isdefault(const char *s);
extern bool rs_cin_iscase(const char *s, bool strict);
extern bool rs_cin_has_js_key(const char *text);
extern bool rs_cin_is_cpp_namespace(const char *s);
extern bool rs_cin_ends_in(const char *s, const char *find);
extern const char *rs_cin_skip_comment_and_string(const char *s);
// Bracket matching
typedef struct {
  bool found;
  int col;
} BracketMatch;
extern BracketMatch rs_find_last_paren(const char *line, char start, char end);
extern int rs_count_unmatched_open(const char *line, char start, char end, int max_col);
extern bool rs_is_inside_brackets(const char *line, int col, char start, char end);
extern int rs_cin_skip2pos_col(const char *line, int col);
extern bool rs_cin_iscomment_pos(const char *line, int col);
// Statement detection
extern char rs_cin_isterminated(const char *s, bool incl_open, bool incl_comma);
extern bool rs_cin_is_terminated(const char *s);
extern bool rs_cin_iswhile(const char *p);
extern bool rs_cin_isfor(const char *p);
extern bool rs_cin_isreturn(const char *p);
extern bool rs_cin_iscontinue(const char *p);
extern bool rs_cin_isgoto(const char *p);
extern bool rs_cin_isswitch(const char *p);
extern int rs_cin_find_equal(const char *s);
extern bool rs_cin_is_compound_init(const char *s);
extern bool rs_cin_istypedef(const char *p);
extern bool rs_cin_isstatic(const char *p);
extern bool rs_cin_ispublic(const char *p);
extern bool rs_cin_isprotected(const char *p);
extern bool rs_cin_isprivate(const char *p);
extern bool rs_cin_isenum(const char *p);
extern bool rs_cin_isstruct(const char *p);
extern bool rs_cin_isclass(const char *p);
extern bool rs_cin_isunion(const char *p);
// Indentation helpers
extern bool rs_cin_ends_in_backslash(const char *s);
extern bool rs_linewhite(const char *s);
extern bool rs_cin_looks_like_funcdecl(const char *s);
extern bool rs_cin_is_kr_param(const char *s);
extern bool rs_cin_is_cpp_lambda(const char *s);
extern bool rs_cin_is_template_decl(const char *s);
extern bool rs_cin_is_extern_c(const char *s);
extern bool rs_cin_starts_multiline_comment(const char *s);
extern bool rs_cin_inside_multiline_comment(const char *s);
extern bool rs_cin_ends_multiline_comment(const char *s);
extern bool rs_cin_is_closing_brace_line(const char *s);
extern bool rs_cin_is_opening_brace_line(const char *s);
extern bool rs_cin_is_ternary_continuation(const char *s);
extern bool rs_cin_is_bool_continuation(const char *s);
extern int rs_cin_brace_position(const char *line, int brace_col);
extern bool rs_cin_is_operator_continuation(const char *s);

/// C accessor for p_paste global option.
int nvim_get_p_paste(void)
{
  return p_paste;
}

/// C accessor for curbuf->b_p_cin (cindent option).
int nvim_curbuf_get_p_cin(void)
{
  return curbuf->b_p_cin;
}

/// C accessor for whether curbuf->b_p_inde is non-empty.
int nvim_curbuf_get_inde_nonempty(void)
{
  return *curbuf->b_p_inde != NUL;
}

/// C accessor for curbuf->b_p_si (smartindent option).
int nvim_curbuf_get_p_si(void)
{
  return curbuf->b_p_si;
}

/// C accessor for curbuf->b_ind_hash_comment (# comment indentation).
int nvim_curbuf_get_ind_hash_comment(void)
{
  return curbuf->b_ind_hash_comment;
}

/// C accessor for curbuf->b_p_lisp (lisp option).
int nvim_curbuf_get_p_lisp(void)
{
  return curbuf->b_p_lisp;
}

/// C accessor for curbuf->b_p_inde (indentexpr) as pointer.
const char *nvim_curbuf_get_inde_ptr(void)
{
  return curbuf->b_p_inde;
}

/// C accessor for curbuf->b_p_lop (lispoptions).
const char *nvim_curbuf_get_p_lop(void)
{
  return curbuf->b_p_lop;
}

/// C accessor for curbuf->b_p_lw (lispwords local).
const char *nvim_curbuf_get_p_lw(void)
{
  return curbuf->b_p_lw;
}

/// C accessor for global p_lispwords.
const char *nvim_get_p_lispwords(void)
{
  return p_lispwords;
}

/// C accessor for in_cinkeys function (for Rust FFI).
bool nvim_in_cinkeys(int keytyped, int when, bool line_is_empty)
{
  return in_cinkeys(keytyped, when, line_is_empty);
}

// Phase 2 C accessors
/// C accessor for curwin->w_cursor.lnum.
int nvim_cindent_curwin_get_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

/// C accessor for curbuf->b_ind_maxparen.
int nvim_cindent_curbuf_get_ind_maxparen(void)
{
  return curbuf->b_ind_maxparen;
}

/// C accessor for curbuf->b_p_cinw (cinwords option).
const char *nvim_cindent_curbuf_get_cinw(void)
{
  return curbuf->b_p_cinw;
}

/// C accessor for ml_get(lnum).
const char *nvim_cindent_ml_get(int lnum)
{
  return ml_get(lnum);
}

/// C accessor for get_indent_lnum(lnum).
int nvim_cindent_get_indent_lnum(int lnum)
{
  return get_indent_lnum(lnum);
}

// Phase 2 Rust function declarations
extern bool rs_cin_islabel_skip(const char *s, int *new_offset);
extern const char *rs_after_label(const char *l);
extern bool rs_cin_is_if_for_while_before_offset(const char *line, int *poffset);
extern int rs_corr_ind_maxparen(int startpos_lnum);
extern bool rs_cin_isinit(const char *line);
extern bool rs_cin_is_cinword(const char *line);
extern int rs_cin_ispreproc_cont(int lnum, int amount, int *out_lnum, int *out_amount);

// Phase 5: parse_cino
extern void rs_parse_cino(const char *cino, int sw, CindentOptions *opts);

/// C accessor for curbuf->b_p_cinsd (cinscopedecls option).
const char *nvim_cindent_curbuf_get_cinsd(void)
{
  return curbuf->b_p_cinsd;
}

// Phase 3 C accessors

/// C accessor for findmatchlimit, returning a copyable result.
FindMatchResult nvim_cindent_findmatchlimit(int what, int flags, int64_t maxtravel)
{
  FindMatchResult result = { false, 0, 0 };
  pos_T *pos = findmatchlimit(NULL, what, flags, maxtravel);
  if (pos != NULL) {
    result.found = true;
    result.lnum = pos->lnum;
    result.col = pos->col;
  }
  return result;
}

/// C accessor for ml_get_pos: get string at (lnum, col) and return pointer offset by col.
const char *nvim_cindent_ml_get_pos_lnum_col(int lnum, int col)
{
  return ml_get(lnum) + col;
}

/// C accessor for getvcol.
int nvim_cindent_getvcol(int lnum, int col)
{
  pos_T fp;
  colnr_T vcol;
  fp.lnum = lnum;
  fp.col = col;
  getvcol(curwin, &fp, &vcol, NULL, NULL);
  return (int)vcol;
}

/// C accessor for curwin->w_cursor.col.
int nvim_cindent_curwin_get_cursor_col(void)
{
  return curwin->w_cursor.col;
}

/// C accessor to set curwin->w_cursor.
void nvim_cindent_curwin_set_cursor(int lnum, int col)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
}

/// C accessor for curbuf->b_ind_maxcomment.
int nvim_cindent_curbuf_get_ind_maxcomment(void)
{
  return curbuf->b_ind_maxcomment;
}

/// C accessor for curbuf->b_ml.ml_line_count.
int nvim_cindent_curbuf_get_ml_line_count(void)
{
  return curbuf->b_ml.ml_line_count;
}

/// C accessor for curbuf->b_ind_cpp_baseclass.
int nvim_cindent_curbuf_get_ind_cpp_baseclass(void)
{
  return curbuf->b_ind_cpp_baseclass;
}

/// C accessor for get_indent().
int nvim_cindent_get_indent(void)
{
  return get_indent();
}

/// C accessor for get_cursor_line_ptr().
const char *nvim_cindent_get_cursor_line_ptr(void)
{
  return get_cursor_line_ptr();
}

// Phase 3 Rust function declarations
extern FindMatchResult rs_find_start_comment(int ind_maxcomment);
extern FindMatchResult rs_find_start_rawstring(int ind_maxcomment);
extern void rs_ind_find_start_CORS(int *out_lnum, int *out_col, int *is_raw);
extern int rs_cin_skip2pos_lnum_col(int lnum, int col);
extern FindMatchResult rs_find_match_char(int c, int ind_maxparen);
extern FindMatchResult rs_find_match_paren(int ind_maxparen);
extern FindMatchResult rs_find_start_brace(void);
extern FindMatchResult rs_find_match_paren_after_brace(int ind_maxparen);
extern int rs_cin_first_id_amount(void);
extern int rs_cin_get_equal_amount(int lnum);
extern int rs_get_indent_nolabel(int lnum);
extern int rs_get_baseclass_amount(int col);
extern FindMatchResult rs_find_line_comment(void);
extern bool rs_cin_iswhileofdo(const char *p, int lnum);
extern bool rs_cin_iswhileofdo_end(int terminated);

// Phase 4: Complex state machine functions
extern bool rs_cin_isscopedecl(const char *p);
extern bool rs_cin_islabel(void);
extern int rs_cin_isfuncdecl(const char **sp, int first_lnum, int min_lnum);
extern int rs_cin_is_cpp_baseclass(int *found, int *pos_lnum, int *pos_col);
extern int rs_skip_label(int lnum, const char **pp);
extern int rs_find_match(int lookfor, int ourscope);

// Find result cache for cpp_baseclass
typedef struct {
  int found;
  lpos_T lpos;
} cpp_baseclass_cache_T;

#include "indent_c.c.generated.h"
// Find the start of a comment, not knowing if we are in a comment right now.
// Search starts at w_cursor.lnum and goes backwards.
// Return NULL when not inside a comment.
static pos_T *ind_find_start_comment(void)  // XXX
{
  return find_start_comment(curbuf->b_ind_maxcomment);
}

pos_T *find_start_comment(int ind_maxcomment)  // XXX
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_start_comment(ind_maxcomment);
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

static pos_T *find_start_rawstring(int ind_maxcomment)  // XXX
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_start_rawstring(ind_maxcomment);
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

/// Find the start of a comment or raw string, not knowing if we are in a
/// comment or raw string right now.
/// Search starts at w_cursor.lnum and goes backwards.
/// If is_raw is given and returns start of raw_string, sets it to true.
///
/// @returns NULL when not inside a comment or raw string.
///
/// @note "CORS" -> Comment Or Raw String
static pos_T *ind_find_start_CORS(linenr_T *is_raw)
{
  static pos_T pos_copy;
  int out_lnum = -1, out_col = 0;
  int raw_lnum = 0;
  rs_ind_find_start_CORS(&out_lnum, &out_col, is_raw != NULL ? &raw_lnum : NULL);
  if (out_lnum == -1) {
    return NULL;
  }
  if (is_raw != NULL && raw_lnum != 0) {
    *is_raw = raw_lnum;
  }
  pos_copy.lnum = out_lnum;
  pos_copy.col = out_col;
  return &pos_copy;
}

// Skip to the end of a "string" and a 'c' character.
// If there is no string or character, return argument unmodified.
static const char *skip_string(const char *p)
{
  return rs_skip_string(p);
}

/// @returns true if "line[col]" is inside a C string.
int is_pos_in_string(const char *line, colnr_T col)
{
  return rs_is_pos_in_string(line, (int)col);
}

// Functions for C-indenting.
// Most of this originally comes from Eric Fischer.

// Below "XXX" means that this function may unlock the current line.

/// @return  true if the string "line" starts with a word from 'cinwords'.
bool cin_is_cinword(const char *line)
{
  return rs_cin_is_cinword(line);
}

/// Check that C-indenting is on.
bool cindent_on(void)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_cindent_on();
}

// Skip over white space and C comments within the line.
// Also skip over Perl/shell comments if desired.
static const char *cin_skipcomment(const char *s)
{
  return rs_cin_skipcomment(s);
}

/// Return true if there is no code at *s.  White space and comments are
/// not considered code.
static int cin_nocode(const char *s)
{
  return rs_cin_nocode(s);
}

// Check previous lines for a "//" line comment, skipping over blank lines.
static pos_T *find_line_comment(void)  // XXX
{
  static pos_T pos;
  FindMatchResult result = rs_find_line_comment();
  if (result.found) {
    pos.lnum = result.lnum;
    pos.col = result.col;
    return &pos;
  }
  return NULL;
}

/// Checks if `text` starts with "key:".
static bool cin_has_js_key(const char *text)
{
  return rs_cin_has_js_key(text);
}

/// Checks if string matches "label:"; move to character after ':' if true.
/// "*s" must point to the start of the label, if there is one.
static bool cin_islabel_skip(const char **s)
  FUNC_ATTR_NONNULL_ALL
{
  int new_offset = 0;
  if (rs_cin_islabel_skip(*s, &new_offset)) {
    *s = *s + new_offset;
    return true;
  }
  return false;
}

// Recognize a label: "label:".
// Note: curwin->w_cursor must be where we are looking for the label.
static bool cin_islabel(void)  // XXX
{
  return rs_cin_islabel();
}

/// Strings can be concatenated with comments between:
/// "string0" |*comment*| "string1"
static const char *cin_skip_comment_and_string(const char *s)
{
  return rs_cin_skip_comment_and_string(s);
}

/// Recognize structure or compound literal initialization:
/// =|return [&][(typecast)] [{]
/// The number of opening braces is arbitrary.
static bool cin_is_compound_init(const char *s)
{
  return rs_cin_is_compound_init(s);
}

/// Recognize enumerations:
/// "[typedef] [static|public|protected|private] enum"
/// Calls another function to recognize structure initialization.
static bool cin_isinit(void)
{
  return rs_cin_isinit(get_cursor_line_ptr());
}

/// Recognize a switch label: "case .*:" or "default:".
///
/// @param strict  Allow relaxed check of case statement for JS
static bool cin_iscase(const char *s, bool strict)
{
  return rs_cin_iscase(s, strict);
}

// Recognize a "default" switch label.
static int cin_isdefault(const char *s)
{
  return rs_cin_isdefault(s);
}

/// Recognize a scope declaration label from the 'cinscopedecls' option.
static bool cin_isscopedecl(const char *p)
{
  return rs_cin_isscopedecl(p);
}

// Maximum number of lines to search back for a "namespace" line.
#define FIND_NAMESPACE_LIM 20

// Recognize a "namespace" scope declaration.
static bool cin_is_cpp_namespace(const char *s)
{
  return rs_cin_is_cpp_namespace(s);
}

// Return a pointer to the first non-empty non-comment character after a ':'.
// Return NULL if not found.
//        case 234:    a = b;
//                     ^
static const char *after_label(const char *l)
{
  return rs_after_label(l);
}

// Get indent of line "lnum", skipping a label.
// Return 0 if there is nothing after the label.
static int get_indent_nolabel(linenr_T lnum)  // XXX
{
  return rs_get_indent_nolabel(lnum);
}

// Find indent for line "lnum", ignoring any case or jump label.
// Also return a pointer to the text (after the label) in "pp".
//   label:     if (asdf && asdfasdf)
//              ^
static int skip_label(linenr_T lnum, const char **pp)
{
  return rs_skip_label(lnum, pp);
}

// Return the indent of the first variable name after a type in a declaration.
//  int     a,                  indent of "a"
//  static struct foo    b,     indent of "b"
//  enum bla    c,              indent of "c"
// Returns zero when it doesn't look like a declaration.
static int cin_first_id_amount(void)
{
  return rs_cin_first_id_amount();
}

// Return the indent of the first non-blank after an equal sign.
//       char *foo = "here";
// Return zero if no (useful) equal sign found.
// Return -1 if the line above "lnum" ends in a backslash.
//      foo = "asdf{backslash}
//             asdf{backslash}
//             here";
static int cin_get_equal_amount(linenr_T lnum)
{
  return rs_cin_get_equal_amount(lnum);
}

// Recognize a preprocessor statement: Any line that starts with '#'.
static int cin_ispreproc(const char *s)
{
  return rs_cin_ispreproc(s);
}

/// Return true if line "*pp" at "*lnump" is a preprocessor statement or a
/// continuation line of a preprocessor statement.  Decrease "*lnump" to the
/// start and return the line in "*pp".
/// Put the amount of indent in "*amount".
static int cin_ispreproc_cont(const char **pp, linenr_T *lnump, int *amount)
{
  int out_lnum = *lnump;
  int out_amount = *amount;
  int retval = rs_cin_ispreproc_cont(*lnump, *amount, &out_lnum, &out_amount);
  if (retval) {
    if (out_lnum != *lnump) {
      *pp = ml_get(out_lnum);
    }
    *lnump = out_lnum;
    *amount = out_amount;
  }
  return retval;
}

// Recognize the start of a C or C++ comment.
static int cin_iscomment(const char *p)
{
  return rs_cin_iscomment(p);
}

// Recognize the start of a "//" comment.
static int cin_islinecomment(const char *p)
{
  return rs_cin_islinecomment(p);
}

/// Recognize a line that starts with '{' or '}', or ends with ';', ',', '{' or
/// '}'.
/// Don't consider "} else" a terminated line.
/// If a line begins with an "else", only consider it terminated if no unmatched
/// opening braces follow (handle "else { foo();" correctly).
///
/// @param incl_open   include '{' at the end as terminator
/// @param incl_comma  recognize a trailing comma
///
/// @return  the character terminating the line (ending char's have precedence if
///          both apply in order to determine initializations).
static char cin_isterminated(const char *s, int incl_open, int incl_comma)
{
  return rs_cin_isterminated(s, (bool)incl_open, (bool)incl_comma);
}

/// Recognizes the basic picture of a function declaration -- it needs to
/// have an open paren somewhere and a close paren at the end of the line and
/// no semicolons anywhere.
/// When a line ends in a comma we continue looking in the next line.
///
/// @param[in]  sp  Points to a string with the line. When looking at other
///                 lines it must be restored to the line. When it's NULL fetch
///                 lines here.
/// @param[in]  first_lnum Where to start looking.
/// @param[in]  min_lnum The line before which we will not be looking.
static int cin_isfuncdecl(const char **sp, linenr_T first_lnum, linenr_T min_lnum)
{
  return rs_cin_isfuncdecl(sp, first_lnum, min_lnum);
}

static int cin_isif(const char *p)
{
  return rs_cin_isif(p);
}

static int cin_iselse(const char *p)
{
  return rs_cin_iselse(p);
}

static int cin_isdo(const char *p)
{
  return rs_cin_isdo(p);
}

// Check if this is a "while" that should have a matching "do".
// We only accept a "while (condition) ;", with only white space between the
// ')' and ';'. The condition may be spread over several lines.
static int cin_iswhileofdo(const char *p, linenr_T lnum)  // XXX
{
  return rs_cin_iswhileofdo(p, lnum);
}

// Check whether in "p" there is an "if", "for" or "while" before "*poffset".
// Return 0 if there is none.
// Otherwise return !0 and update "*poffset" to point to the place where the
// string was found.
static int cin_is_if_for_while_before_offset(const char *line, int *poffset)
{
  return rs_cin_is_if_for_while_before_offset(line, poffset);
}

/// Return true if we are at the end of a do-while.
///    do
///       nothing;
///    while (foo
///             && bar);  <-- here
/// Adjust the cursor to the line with "while".
static int cin_iswhileofdo_end(int terminated)
{
  return rs_cin_iswhileofdo_end(terminated);
}

static int cin_isbreak(const char *p)
{
  return rs_cin_isbreak(p);
}

// Find the position of a C++ base-class declaration or
// constructor-initialization. eg:
//
// class MyClass :
//      baseClass               <-- here
// class MyClass : public baseClass,
//      anotherBaseClass        <-- here (should probably lineup ??)
// MyClass::MyClass(...) :
//      baseClass(...)          <-- here (constructor-initialization)
//
// This is a lot of guessing.  Watch out for "cond ? func() : foo".
static int cin_is_cpp_baseclass(cpp_baseclass_cache_T *cached)
{
  int found = cached->found;
  int pos_lnum = cached->lpos.lnum;
  int pos_col = cached->lpos.col;
  int result = rs_cin_is_cpp_baseclass(&found, &pos_lnum, &pos_col);
  cached->found = found;
  cached->lpos.lnum = pos_lnum;
  cached->lpos.col = pos_col;
  return result;
}

static int get_baseclass_amount(int col)
{
  return rs_get_baseclass_amount(col);
}

/// Return true if string "s" ends with the string "find", possibly followed by
/// white space and comments.  Skip strings and comments.
static int cin_ends_in(const char *s, const char *find)
{
  return rs_cin_ends_in(s, find);
}

/// Return true when "s" starts with "word" and then a non-ID character.
static int cin_starts_with(const char *s, const char *word)
{
  return rs_cin_starts_with(s, word);
}

/// Recognize a `extern "C"` or `extern "C++"` linkage specifications.
static int cin_is_cpp_extern_c(const char *s)
{
  return rs_cin_is_extern_c(s);
}

// Skip strings, chars and comments until at or past "trypos".
// Return the column found.
static int cin_skip2pos(pos_T *trypos)
{
  return rs_cin_skip2pos_col(ml_get(trypos->lnum), trypos->col);
}

// Find the '{' at the start of the block we are in.
// Return NULL if no match found.
// Ignore a '{' that is in a comment, makes indenting the next three lines
// work.
// foo()
// {
// }

static pos_T *find_start_brace(void)  // XXX
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_start_brace();
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

/// Find the matching '(', ignoring it if it is in a comment.
/// @returns NULL or the found match.
static pos_T *find_match_paren(int ind_maxparen)
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_match_paren(ind_maxparen);
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

static pos_T *find_match_char(char c, int ind_maxparen)
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_match_char((int)(uint8_t)c, ind_maxparen);
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

/// Find the matching '(', ignoring it if it is in a comment or before an
/// unmatched {.
/// @returns NULL or the found match.
static pos_T *find_match_paren_after_brace(int ind_maxparen)
{
  static pos_T pos_copy;
  FindMatchResult result = rs_find_match_paren_after_brace(ind_maxparen);
  if (result.found) {
    pos_copy.lnum = result.lnum;
    pos_copy.col = result.col;
    return &pos_copy;
  }
  return NULL;
}

// Return ind_maxparen corrected for the difference in line number between the
// cursor position and "startpos".  This makes sure that searching for a
// matching paren above the cursor line doesn't find a match because of
// looking a few lines further.
static int corr_ind_maxparen(pos_T *startpos)
{
  return rs_corr_ind_maxparen(startpos->lnum);
}

// Set w_cursor.col to the column number of the last unmatched ')' or '{' in
// line "l".  "l" must point to the start of the line.
static int find_last_paren(const char *l, char start, char end)
{
  BracketMatch result = rs_find_last_paren(l, start, end);
  if (result.found) {
    curwin->w_cursor.col = result.col;
  } else {
    curwin->w_cursor.col = 0;
  }
  return result.found;
}

// Parse 'cinoptions' and set the values in "curbuf".
// Must be called when 'cinoptions', 'shiftwidth' and/or 'tabstop' changes.
void parse_cino(buf_T *buf)
{
  int sw = get_sw_value(buf);
  CindentOptions opts;
  rs_parse_cino(buf->b_p_cino, sw, &opts);

  buf->b_ind_level = opts.ind_level;
  buf->b_ind_open_imag = opts.ind_open_imag;
  buf->b_ind_no_brace = opts.ind_no_brace;
  buf->b_ind_first_open = opts.ind_first_open;
  buf->b_ind_open_extra = opts.ind_open_extra;
  buf->b_ind_close_extra = opts.ind_close_extra;
  buf->b_ind_open_left_imag = opts.ind_open_left_imag;
  buf->b_ind_jump_label = opts.ind_jump_label;
  buf->b_ind_case = opts.ind_case;
  buf->b_ind_case_code = opts.ind_case_code;
  buf->b_ind_case_break = opts.ind_case_break;
  buf->b_ind_scopedecl = opts.ind_scopedecl;
  buf->b_ind_scopedecl_code = opts.ind_scopedecl_code;
  buf->b_ind_param = opts.ind_param;
  buf->b_ind_func_type = opts.ind_func_type;
  buf->b_ind_cpp_baseclass = opts.ind_cpp_baseclass;
  buf->b_ind_continuation = opts.ind_continuation;
  buf->b_ind_unclosed = opts.ind_unclosed;
  buf->b_ind_unclosed2 = opts.ind_unclosed2;
  buf->b_ind_unclosed_noignore = opts.ind_unclosed_noignore;
  buf->b_ind_unclosed_wrapped = opts.ind_unclosed_wrapped;
  buf->b_ind_unclosed_whiteok = opts.ind_unclosed_whiteok;
  buf->b_ind_matching_paren = opts.ind_matching_paren;
  buf->b_ind_paren_prev = opts.ind_paren_prev;
  buf->b_ind_comment = opts.ind_comment;
  buf->b_ind_in_comment = opts.ind_in_comment;
  buf->b_ind_in_comment2 = opts.ind_in_comment2;
  buf->b_ind_maxparen = opts.ind_maxparen;
  buf->b_ind_maxcomment = opts.ind_maxcomment;
  buf->b_ind_java = opts.ind_java;
  buf->b_ind_js = opts.ind_js;
  buf->b_ind_keep_case_label = opts.ind_keep_case_label;
  buf->b_ind_cpp_namespace = opts.ind_cpp_namespace;
  buf->b_ind_if_for_while = opts.ind_if_for_while;
  buf->b_ind_hash_comment = opts.ind_hash_comment;
  buf->b_ind_cpp_extern_c = opts.ind_cpp_extern_c;
  buf->b_ind_pragma = opts.ind_pragma;
}

// Return the desired indent for C code.
// Return -1 if the indent should be left alone (inside a raw string).
int get_c_indent(void)
{
  pos_T cur_curpos;
  int amount;
  int scope_amount;
  int cur_amount = MAXCOL;
  colnr_T col;
  char *theline;
  char *linecopy;
  pos_T *trypos;
  pos_T *comment_pos;
  pos_T *tryposBrace = NULL;
  pos_T tryposCopy;
  pos_T our_paren_pos;
  char *start;
  int start_brace;
#define BRACE_IN_COL0           1           // '{' is in column 0
#define BRACE_AT_START          2           // '{' is at start of line
#define BRACE_AT_END            3           // '{' is at end of line
  linenr_T ourscope;
  const char *l;
  const char *look;
  char terminated;
  int lookfor;
#define LOOKFOR_INITIAL         0
#define LOOKFOR_IF              1
#define LOOKFOR_DO              2
#define LOOKFOR_CASE            3
#define LOOKFOR_ANY             4
#define LOOKFOR_TERM            5
#define LOOKFOR_UNTERM          6
#define LOOKFOR_SCOPEDECL       7
#define LOOKFOR_NOBREAK         8
#define LOOKFOR_CPP_BASECLASS   9
#define LOOKFOR_ENUM_OR_INIT    10
#define LOOKFOR_JS_KEY          11
#define LOOKFOR_COMMA           12

  int whilelevel;
  linenr_T lnum;
  int n;
  int lookfor_break;
  bool lookfor_cpp_namespace = false;
  int cont_amount = 0;              // amount for continuation line
  int original_line_islabel;
  int added_to_amount = 0;
  linenr_T raw_string_start = 0;
  cpp_baseclass_cache_T cache_cpp_baseclass = { false, { MAXLNUM, 0 } };

  // make a copy, value is changed below
  int ind_continuation = curbuf->b_ind_continuation;

  // remember where the cursor was when we started
  cur_curpos = curwin->w_cursor;

  // if we are at line 1 zero indent is fine, right?
  if (cur_curpos.lnum == 1) {
    return 0;
  }

  // Get a copy of the current contents of the line.
  // This is required, because only the most recent line obtained with
  // ml_get is valid!
  linecopy = xstrdup(ml_get(cur_curpos.lnum));

  // In insert mode and the cursor is on a ')' truncate the line at the
  // cursor position.  We don't want to line up with the matching '(' when
  // inserting new stuff.
  // For unknown reasons the cursor might be past the end of the line, thus
  // check for that.
  if ((State & MODE_INSERT)
      && curwin->w_cursor.col < (colnr_T)strlen(linecopy)
      && linecopy[curwin->w_cursor.col] == ')') {
    linecopy[curwin->w_cursor.col] = NUL;
  }

  theline = skipwhite(linecopy);

  // move the cursor to the start of the line

  curwin->w_cursor.col = 0;

  original_line_islabel = cin_islabel();    // XXX

  // If we are inside a raw string don't change the indent.
  // Ignore a raw string inside a comment.
  comment_pos = ind_find_start_comment();
  if (comment_pos != NULL) {
    // findmatchlimit() static pos is overwritten, make a copy
    tryposCopy = *comment_pos;
    comment_pos = &tryposCopy;
  }
  trypos = find_start_rawstring(curbuf->b_ind_maxcomment);
  if (trypos != NULL && (comment_pos == NULL || lt(*trypos, *comment_pos))) {
    amount = -1;
    goto laterend;
  }

  // #defines and so on go at the left when included in 'cinkeys',
  // excluding pragmas when customized in 'cinoptions'
  if (*theline == '#' && (*linecopy == '#' || in_cinkeys('#', ' ', true))) {
    const char *const directive = skipwhite(theline + 1);
    if (curbuf->b_ind_pragma == 0 || strncmp(directive, "pragma", 6) != 0) {
      amount = curbuf->b_ind_hash_comment;
      goto theend;
    }
  }

  // Is it a non-case label? Then that goes at the left margin too unless:
  //  - JS flag is set.
  //  - 'L' item has a positive value.
  if (original_line_islabel && !curbuf->b_ind_js && curbuf->b_ind_jump_label < 0) {
    amount = 0;
    goto theend;
  }
  // If we're inside a "//" comment and there is a "//" comment in a
  // previous line, lineup with that one.
  if (cin_islinecomment(theline)) {
    pos_T linecomment_pos;

    trypos = find_line_comment();  // XXX
    if (trypos == NULL && curwin->w_cursor.lnum > 1) {
      // There may be a statement before the comment, search from the end
      // of the line for a comment start.
      linecomment_pos.col = check_linecomment(ml_get(curwin->w_cursor.lnum - 1));
      if (linecomment_pos.col != MAXCOL) {
        trypos = &linecomment_pos;
        trypos->lnum = curwin->w_cursor.lnum - 1;
      }
    }
    if (trypos != NULL) {
      // find how indented the line beginning the comment is
      getvcol(curwin, trypos, &col, NULL, NULL);
      amount = col;
      goto theend;
    }
  }
  // If we're inside a comment and not looking at the start of the
  // comment, try using the 'comments' option.
  if (!cin_iscomment(theline) && comment_pos != NULL) {  // XXX
    int lead_start_len = 2;
    int lead_middle_len = 1;
    char lead_start[COM_MAX_LEN];             // start-comment string
    char lead_middle[COM_MAX_LEN];            // middle-comment string
    char lead_end[COM_MAX_LEN];               // end-comment string
    char *p;
    int start_align = 0;
    int start_off = 0;
    int done = false;

    // find how indented the line beginning the comment is
    getvcol(curwin, comment_pos, &col, NULL, NULL);
    amount = col;
    *lead_start = NUL;
    *lead_middle = NUL;

    p = curbuf->b_p_com;
    while (*p != NUL) {
      int align = 0;
      int off = 0;
      int what = 0;

      while (*p != NUL && *p != ':') {
        if (*p == COM_START || *p == COM_END || *p == COM_MIDDLE) {
          what = (unsigned char)(*p++);
        } else if (*p == COM_LEFT || *p == COM_RIGHT) {
          align = (unsigned char)(*p++);
        } else if (ascii_isdigit(*p) || *p == '-') {
          off = getdigits_int(&p, true, 0);
        } else {
          p++;
        }
      }

      if (*p == ':') {
        p++;
      }
      (void)copy_option_part(&p, lead_end, COM_MAX_LEN, ",");
      if (what == COM_START) {
        STRCPY(lead_start, lead_end);
        lead_start_len = (int)strlen(lead_start);
        start_off = off;
        start_align = align;
      } else if (what == COM_MIDDLE) {
        STRCPY(lead_middle, lead_end);
        lead_middle_len = (int)strlen(lead_middle);
      } else if (what == COM_END) {
        // If our line starts with the middle comment string, line it
        // up with the comment opener per the 'comments' option.
        if (strncmp(theline, lead_middle, (size_t)lead_middle_len) == 0
            && strncmp(theline, lead_end, strlen(lead_end)) != 0) {
          done = true;
          if (curwin->w_cursor.lnum > 1) {
            // If the start comment string matches in the previous
            // line, use the indent of that line plus offset.  If
            // the middle comment string matches in the previous
            // line, use the indent of that line.  XXX
            look = skipwhite(ml_get(curwin->w_cursor.lnum - 1));
            if (strncmp(look, lead_start, (size_t)lead_start_len) == 0) {
              amount = get_indent_lnum(curwin->w_cursor.lnum - 1);
            } else if (strncmp(look, lead_middle, (size_t)lead_middle_len) == 0) {
              amount = get_indent_lnum(curwin->w_cursor.lnum - 1);
              break;
            } else if (strncmp(ml_get(comment_pos->lnum) + comment_pos->col,
                               lead_start, (size_t)lead_start_len) != 0) {
              // If the start comment string doesn't match with the
              // start of the comment, skip this entry. XXX
              continue;
            }
          }
          if (start_off != 0) {
            amount += start_off;
          } else if (start_align == COM_RIGHT) {
            amount += vim_strsize(lead_start) - vim_strsize(lead_middle);
          }
          break;
        }

        // If our line starts with the end comment string, line it up
        // with the middle comment
        if (strncmp(theline, lead_middle, (size_t)lead_middle_len) != 0
            && strncmp(theline, lead_end, strlen(lead_end)) == 0) {
          amount = get_indent_lnum(curwin->w_cursor.lnum - 1);
          // XXX
          if (off != 0) {
            amount += off;
          } else if (align == COM_RIGHT) {
            amount += vim_strsize(lead_start) - vim_strsize(lead_middle);
          }
          done = true;
          break;
        }
      }
    }

    // If our line starts with an asterisk, line up with the
    // asterisk in the comment opener; otherwise, line up
    // with the first character of the comment text.
    if (done) {
      // skip
    } else if (theline[0] == '*') {
      amount += 1;
    } else {
      // If we are more than one line away from the comment opener, take
      // the indent of the previous non-empty line.  If 'cino' has "CO"
      // and we are just below the comment opener and there are any
      // white characters after it line up with the text after it;
      // otherwise, add the amount specified by "c" in 'cino'
      amount = -1;
      for (lnum = cur_curpos.lnum - 1; lnum > comment_pos->lnum; lnum--) {
        if (linewhite(lnum)) {                      // skip blank lines
          continue;
        }
        amount = get_indent_lnum(lnum);             // XXX
        break;
      }
      if (amount == -1) {                           // use the comment opener
        if (!curbuf->b_ind_in_comment2) {
          start = ml_get(comment_pos->lnum);
          look = start + comment_pos->col + 2;  // skip / and *
          if (*look != NUL) {                   // if something after it
            comment_pos->col = (colnr_T)(skipwhite(look) - start);
          }
        }
        getvcol(curwin, comment_pos, &col, NULL, NULL);
        amount = col;
        if (curbuf->b_ind_in_comment2 || *look == NUL) {
          amount += curbuf->b_ind_in_comment;
        }
      }
    }
    goto theend;
  }
  // Are we looking at a ']' that has a match?
  if (*skipwhite(theline) == ']'
      && (trypos = find_match_char('[', curbuf->b_ind_maxparen)) != NULL) {
    // align with the line containing the '['.
    amount = get_indent_lnum(trypos->lnum);
    goto theend;
  }
  // Are we inside parentheses or braces?
  // XXX
  if (((trypos = find_match_paren(curbuf->b_ind_maxparen)) != NULL
       && curbuf->b_ind_java == 0)
      || (tryposBrace = find_start_brace()) != NULL
      || trypos != NULL) {
    if (trypos != NULL && tryposBrace != NULL) {
      // Both an unmatched '(' and '{' is found.  Use the one which is
      // closer to the current cursor position, set the other to NULL.
      if (trypos->lnum != tryposBrace->lnum
          ? trypos->lnum < tryposBrace->lnum
          : trypos->col < tryposBrace->col) {
        trypos = NULL;
      } else {
        tryposBrace = NULL;
      }
    }

    if (trypos != NULL) {
      our_paren_pos = *trypos;
      // If the matching paren is more than one line away, use the indent of
      // a previous non-empty line that matches the same paren.
      if (theline[0] == ')' && curbuf->b_ind_paren_prev) {
        // Line up with the start of the matching paren line.
        amount = get_indent_lnum(curwin->w_cursor.lnum - 1);      // XXX
      } else {
        amount = -1;
        for (lnum = cur_curpos.lnum - 1; lnum > our_paren_pos.lnum; lnum--) {
          l = skipwhite(ml_get(lnum));
          if (cin_nocode(l)) {                   // skip comment lines
            continue;
          }
          if (cin_ispreproc_cont(&l, &lnum, &amount)) {
            continue;                           // ignore #define, #if, etc.
          }
          curwin->w_cursor.lnum = lnum;

          // Skip a comment or raw string. XXX
          if ((trypos = ind_find_start_CORS(NULL)) != NULL) {
            lnum = trypos->lnum + 1;
            continue;
          }

          // XXX
          if ((trypos = find_match_paren(corr_ind_maxparen(&cur_curpos))) != NULL
              && trypos->lnum == our_paren_pos.lnum
              && trypos->col == our_paren_pos.col) {
            amount = get_indent_lnum(lnum);             // XXX

            if (theline[0] == ')') {
              if (our_paren_pos.lnum != lnum
                  && cur_amount > amount) {
                cur_amount = amount;
              }
              amount = -1;
            }
            break;
          }
        }
      }

      // Line up with line where the matching paren is. XXX
      // If the line starts with a '(' or the indent for unclosed
      // parentheses is zero, line up with the unclosed parentheses.
      if (amount == -1) {
        int ignore_paren_col = 0;
        int is_if_for_while = 0;

        if (curbuf->b_ind_if_for_while) {
          // Look for the outermost opening parenthesis on this line
          // and check whether it belongs to an "if", "for" or "while".

          pos_T cursor_save = curwin->w_cursor;
          pos_T outermost;
          char *line;

          trypos = &our_paren_pos;
          do {
            outermost = *trypos;
            curwin->w_cursor.lnum = outermost.lnum;
            curwin->w_cursor.col = outermost.col;

            trypos = find_match_paren(curbuf->b_ind_maxparen);
          } while (trypos && trypos->lnum == outermost.lnum);

          curwin->w_cursor = cursor_save;

          line = ml_get(outermost.lnum);

          is_if_for_while =
            cin_is_if_for_while_before_offset(line, &outermost.col);
        }

        amount = skip_label(our_paren_pos.lnum, &look);
        look = skipwhite(look);
        if (*look == '(') {
          linenr_T save_lnum = curwin->w_cursor.lnum;
          char *line;
          int look_col;

          // Ignore a '(' in front of the line that has a match before
          // our matching '('.
          curwin->w_cursor.lnum = our_paren_pos.lnum;
          line = get_cursor_line_ptr();
          look_col = (int)(look - line);
          curwin->w_cursor.col = look_col + 1;
          if ((trypos = findmatchlimit(NULL, ')', 0,
                                       curbuf->b_ind_maxparen))
              != NULL
              && trypos->lnum == our_paren_pos.lnum
              && trypos->col < our_paren_pos.col) {
            ignore_paren_col = trypos->col + 1;
          }

          curwin->w_cursor.lnum = save_lnum;
          look = ml_get(our_paren_pos.lnum) + look_col;
        }
        if (theline[0] == ')' || (curbuf->b_ind_unclosed == 0
                                  && is_if_for_while == 0)
            || (!curbuf->b_ind_unclosed_noignore && *look == '('
                && ignore_paren_col == 0)) {
          // If we're looking at a close paren, line up right there;
          // otherwise, line up with the next (non-white) character.
          // When b_ind_unclosed_wrapped is set and the matching paren is
          // the last nonwhite character of the line, use either the
          // indent of the current line or the indentation of the next
          // outer paren and add b_ind_unclosed_wrapped (for very long
          // lines).
          if (theline[0] != ')') {
            cur_amount = MAXCOL;
            l = ml_get(our_paren_pos.lnum);
            if (curbuf->b_ind_unclosed_wrapped && cin_ends_in(l, "(")) {
              // look for opening unmatched paren, indent one level
              // for each additional level
              n = 1;
              for (col = 0; col < our_paren_pos.col; col++) {
                switch (l[col]) {
                case '(':
                case '{':
                  n++;
                  break;

                case ')':
                case '}':
                  if (n > 1) {
                    n--;
                  }
                  break;
                }
              }

              our_paren_pos.col = 0;
              amount += n * curbuf->b_ind_unclosed_wrapped;
            } else if (curbuf->b_ind_unclosed_whiteok) {
              our_paren_pos.col++;
            } else {
              col = our_paren_pos.col + 1;
              while (ascii_iswhite(l[col])) {
                col++;
              }
              if (l[col] != NUL) {              // In case of trailing space
                our_paren_pos.col = col;
              } else {
                our_paren_pos.col++;
              }
            }
          }

          // Find how indented the paren is, or the character after it
          // if we did the above "if".
          if (our_paren_pos.col > 0) {
            getvcol(curwin, &our_paren_pos, &col, NULL, NULL);
            if (cur_amount > (int)col) {
              cur_amount = col;
            }
          }
        }

        if (theline[0] == ')' && curbuf->b_ind_matching_paren) {
          // Line up with the start of the matching paren line.
        } else if ((curbuf->b_ind_unclosed == 0 && is_if_for_while == 0)
                   || (!curbuf->b_ind_unclosed_noignore
                       && *look == '(' && ignore_paren_col == 0)) {
          if (cur_amount != MAXCOL) {
            amount = cur_amount;
          }
        } else {
          // Add b_ind_unclosed2 for each '(' before our matching one,
          // but ignore (void) before the line (ignore_paren_col).
          col = our_paren_pos.col;
          while ((int)our_paren_pos.col > ignore_paren_col) {
            our_paren_pos.col--;
            switch (*ml_get_pos(&our_paren_pos)) {
            case '(':
              amount += curbuf->b_ind_unclosed2;
              col = our_paren_pos.col;
              break;
            case ')':
              amount -= curbuf->b_ind_unclosed2;
              col = MAXCOL;
              break;
            }
          }

          // Use b_ind_unclosed once, when the first '(' is not inside
          // braces
          if (col == MAXCOL) {
            amount += curbuf->b_ind_unclosed;
          } else {
            curwin->w_cursor.lnum = our_paren_pos.lnum;
            curwin->w_cursor.col = col;
            if (find_match_paren_after_brace(curbuf->b_ind_maxparen)) {
              amount += curbuf->b_ind_unclosed2;
            } else {
              if (is_if_for_while) {
                amount += curbuf->b_ind_if_for_while;
              } else {
                amount += curbuf->b_ind_unclosed;
              }
            }
          }
          // For a line starting with ')' use the minimum of the two
          // positions, to avoid giving it more indent than the previous
          // lines:
          //  func_long_name(               if (x
          //    arg                                 && yy
          //    )         ^ not here           )    ^ not here
          if (cur_amount < amount) {
            amount = cur_amount;
          }
        }
      }

      // add extra indent for a comment
      if (cin_iscomment(theline)) {
        amount += curbuf->b_ind_comment;
      }
    } else {
      // We are inside braces, there is a { before this line at the position
      // stored in tryposBrace.
      // Make a copy of tryposBrace, it may point to pos_copy inside
      // find_start_brace(), which may be changed somewhere.
      tryposCopy = *tryposBrace;
      tryposBrace = &tryposCopy;
      trypos = tryposBrace;
      ourscope = trypos->lnum;
      start = ml_get(ourscope);

      // Now figure out how indented the line is in general.
      // If the brace was at the start of the line, we use that;
      // otherwise, check out the indentation of the line as
      // a whole and then add the "imaginary indent" to that.
      look = skipwhite(start);
      if (*look == '{') {
        getvcol(curwin, trypos, &col, NULL, NULL);
        amount = col;
        if (*start == '{') {
          start_brace = BRACE_IN_COL0;
        } else {
          start_brace = BRACE_AT_START;
        }
      } else {
        // That opening brace might have been on a continuation
        // line.  If so, find the start of the line.
        curwin->w_cursor.lnum = ourscope;

        // Position the cursor over the rightmost paren, so that
        // matching it will take us back to the start of the line.
        lnum = ourscope;
        if (find_last_paren(start, '(', ')')
            && (trypos = find_match_paren(curbuf->b_ind_maxparen)) != NULL) {
          lnum = trypos->lnum;
        }

        // It could have been something like
        //         case 1: if (asdf &&
        //                      ldfd) {
        //                  }
        if ((curbuf->b_ind_js || curbuf->b_ind_keep_case_label)
            && cin_iscase(skipwhite(get_cursor_line_ptr()), false)) {
          amount = get_indent();
        } else if (curbuf->b_ind_js) {
          amount = get_indent_lnum(lnum);
        } else {
          amount = skip_label(lnum, &l);
        }

        start_brace = BRACE_AT_END;
      }

      // For Javascript check if the line starts with "key:".
      bool js_cur_has_key = curbuf->b_ind_js ? cin_has_js_key(theline) : false;

      // If we're looking at a closing brace, that's where
      // we want to be.  Otherwise, add the amount of room
      // that an indent is supposed to be.
      if (theline[0] == '}') {
        // they may want closing braces to line up with something
        // other than the open brace.  indulge them, if so.
        amount += curbuf->b_ind_close_extra;
      } else {
        // If we're looking at an "else", try to find an "if"
        // to match it with.
        // If we're looking at a "while", try to find a "do"
        // to match it with.
        lookfor = LOOKFOR_INITIAL;
        if (cin_iselse(theline)) {
          lookfor = LOOKFOR_IF;
        } else if (cin_iswhileofdo(theline, cur_curpos.lnum)) {   // XXX
          lookfor = LOOKFOR_DO;
        }
        if (lookfor != LOOKFOR_INITIAL) {
          curwin->w_cursor.lnum = cur_curpos.lnum;
          if (find_match(lookfor, ourscope) == OK) {
            amount = get_indent();              // XXX
            goto theend;
          }
        }

        // We get here if we are not on an "while-of-do" or "else" (or
        // failed to find a matching "if").
        // Search backwards for something to line up with.
        // First set amount for when we don't find anything.

        // if the '{' is  _really_ at the left margin, use the imaginary
        // location of a left-margin brace.  Otherwise, correct the
        // location for b_ind_open_extra.

        if (start_brace == BRACE_IN_COL0) {     // '{' is in column 0
          amount = curbuf->b_ind_open_left_imag;
          lookfor_cpp_namespace = true;
        } else if (start_brace == BRACE_AT_START
                   && lookfor_cpp_namespace) {  // '{' is at start
          lookfor_cpp_namespace = true;
        } else {
          if (start_brace == BRACE_AT_END) {    // '{' is at end of line
            amount += curbuf->b_ind_open_imag;

            l = skipwhite(get_cursor_line_ptr());
            if (cin_is_cpp_namespace(l)) {
              amount += curbuf->b_ind_cpp_namespace;
            } else if (cin_is_cpp_extern_c(l)) {
              amount += curbuf->b_ind_cpp_extern_c;
            }
          } else {
            // Compensate for adding b_ind_open_extra later.
            amount -= curbuf->b_ind_open_extra;
            if (amount < 0) {
              amount = 0;
            }
          }
        }

        lookfor_break = false;

        if (cin_iscase(theline, false)) {       // it's a switch() label
          lookfor = LOOKFOR_CASE;       // find a previous switch() label
          amount += curbuf->b_ind_case;
        } else if (cin_isscopedecl(theline)) {  // private:, ...
          lookfor = LOOKFOR_SCOPEDECL;          // class decl is this block
          amount += curbuf->b_ind_scopedecl;
        } else {
          if (curbuf->b_ind_case_break && cin_isbreak(theline)) {
            // break; ...
            lookfor_break = true;
          }

          lookfor = LOOKFOR_INITIAL;
          // b_ind_level from start of block
          amount += curbuf->b_ind_level;
        }
        scope_amount = amount;
        whilelevel = 0;

        // Search backwards.  If we find something we recognize, line up
        // with that.
        //
        // If we're looking at an open brace, indent
        // the usual amount relative to the conditional
        // that opens the block.
        curwin->w_cursor = cur_curpos;
        while (true) {
          curwin->w_cursor.lnum--;
          curwin->w_cursor.col = 0;

          // If we went all the way back to the start of our scope, line
          // up with it.
          if (curwin->w_cursor.lnum <= ourscope) {
            // We reached end of scope:
            // If looking for a enum or structure initialization
            // go further back:
            // If it is an initializer (enum xxx or xxx =), then
            // don't add ind_continuation, otherwise it is a variable
            // declaration:
            // int x,
            //     here; <-- add ind_continuation
            if (lookfor == LOOKFOR_ENUM_OR_INIT) {
              if (curwin->w_cursor.lnum == 0
                  || curwin->w_cursor.lnum
                  < ourscope - curbuf->b_ind_maxparen) {
                // nothing found (abuse curbuf->b_ind_maxparen as
                // limit) assume terminated line (i.e. a variable
                // initialization)
                if (cont_amount > 0) {
                  amount = cont_amount;
                } else if (!curbuf->b_ind_js) {
                  amount += ind_continuation;
                }
                break;
              }

              // If we're in a comment or raw string now, skip to
              // the start of it.
              trypos = ind_find_start_CORS(NULL);
              if (trypos != NULL) {
                curwin->w_cursor.lnum = trypos->lnum + 1;
                curwin->w_cursor.col = 0;
                continue;
              }

              l = get_cursor_line_ptr();

              // Skip preprocessor directives and blank lines.
              if (cin_ispreproc_cont(&l, &curwin->w_cursor.lnum, &amount)) {
                continue;
              }

              if (cin_nocode(l)) {
                continue;
              }

              terminated = cin_isterminated(l, false, true);

              // If we are at top level and the line looks like a
              // function declaration, we are done
              // (it's a variable declaration).
              if (start_brace != BRACE_IN_COL0
                  || !cin_isfuncdecl(&l, curwin->w_cursor.lnum, 0)) {
                // if the line is terminated with another ','
                // it is a continued variable initialization.
                // don't add extra indent.
                // TODO(vim): does not work, if  a function
                // declaration is split over multiple lines:
                // cin_isfuncdecl returns false then.
                if (terminated == ',') {
                  break;
                }

                // if it is an enum declaration or an assignment,
                // we are done.
                if (terminated != ';' && cin_isinit()) {
                  break;
                }

                // nothing useful found
                if (terminated == 0 || terminated == '{') {
                  continue;
                }
              }

              if (terminated != ';') {
                // Skip parens and braces. Position the cursor
                // over the rightmost paren, so that matching it
                // will take us back to the start of the line.
                // XXX
                trypos = NULL;
                if (find_last_paren(l, '(', ')')) {
                  trypos = find_match_paren(curbuf->b_ind_maxparen);
                }

                if (trypos == NULL && find_last_paren(l, '{', '}')) {
                  trypos = find_start_brace();
                }

                if (trypos != NULL) {
                  curwin->w_cursor.lnum = trypos->lnum + 1;
                  curwin->w_cursor.col = 0;
                  continue;
                }
              }

              // it's a variable declaration, add indentation
              // like in
              // int a,
              //    b;
              if (cont_amount > 0) {
                amount = cont_amount;
              } else {
                amount += ind_continuation;
              }
            } else if (lookfor == LOOKFOR_UNTERM) {
              if (cont_amount > 0) {
                amount = cont_amount;
              } else {
                amount += ind_continuation;
              }
            } else {
              if (lookfor != LOOKFOR_TERM
                  && lookfor != LOOKFOR_CPP_BASECLASS
                  && lookfor != LOOKFOR_COMMA) {
                amount = scope_amount;
                if (theline[0] == '{') {
                  amount += curbuf->b_ind_open_extra;
                  added_to_amount = curbuf->b_ind_open_extra;
                }
              }

              if (lookfor_cpp_namespace) {
                // Looking for C++ namespace, need to look further
                // back.
                if (curwin->w_cursor.lnum == ourscope) {
                  continue;
                }

                if (curwin->w_cursor.lnum == 0
                    || curwin->w_cursor.lnum
                    < ourscope - FIND_NAMESPACE_LIM) {
                  break;
                }

                // If we're in a comment or raw string now, skip
                // to the start of it.
                trypos = ind_find_start_CORS(NULL);
                if (trypos != NULL) {
                  curwin->w_cursor.lnum = trypos->lnum + 1;
                  curwin->w_cursor.col = 0;
                  continue;
                }

                l = get_cursor_line_ptr();

                // Skip preprocessor directives and blank lines.
                if (cin_ispreproc_cont(&l, &curwin->w_cursor.lnum, &amount)) {
                  continue;
                }

                // Finally the actual check for "namespace".
                if (cin_is_cpp_namespace(l)) {
                  amount += curbuf->b_ind_cpp_namespace
                            - added_to_amount;
                  break;
                } else if (cin_is_cpp_extern_c(l)) {
                  amount += curbuf->b_ind_cpp_extern_c - added_to_amount;
                  break;
                }

                if (cin_nocode(l)) {
                  continue;
                }
              }
            }
            break;
          }

          // If we're in a comment or raw string now, skip to the start
          // of it.
          // XXX
          if ((trypos = ind_find_start_CORS(&raw_string_start)) != NULL) {
            curwin->w_cursor.lnum = trypos->lnum + 1;
            curwin->w_cursor.col = 0;
            continue;
          }

          l = get_cursor_line_ptr();

          // If this is a switch() label, may line up relative to that.
          // If this is a C++ scope declaration, do the same.
          bool iscase = cin_iscase(l, false);
          if (iscase || cin_isscopedecl(l)) {
            // we are only looking for cpp base class
            // declaration/initialization any longer
            if (lookfor == LOOKFOR_CPP_BASECLASS) {
              break;
            }

            // When looking for a "do" we are not interested in
            // labels.
            if (whilelevel > 0) {
              continue;
            }

            //  case xx:
            //      c = 99 +        <- this indent plus continuation
            // ->          here;
            if (lookfor == LOOKFOR_UNTERM || lookfor == LOOKFOR_ENUM_OR_INIT) {
              if (cont_amount > 0) {
                amount = cont_amount;
              } else {
                amount += ind_continuation;
              }
              break;
            }

            // case xx: <- line up with this case
            //     x = 333;
            // case yy:
            if ((iscase && lookfor == LOOKFOR_CASE)
                || (iscase && lookfor_break)
                || (!iscase && lookfor == LOOKFOR_SCOPEDECL)) {
              // Check that this case label is not for another
              // switch()
              // XXX
              if ((trypos = find_start_brace()) == NULL
                  || trypos->lnum == ourscope) {
                amount = get_indent();                  // XXX
                break;
              }
              continue;
            }

            n = get_indent_nolabel(curwin->w_cursor.lnum);          // XXX

            //   case xx: if (cond)         <- line up with this if
            //                y = y + 1;
            // ->         s = 99;
            //
            //   case xx:
            //       if (cond)          <- line up with this line
            //           y = y + 1;
            // ->    s = 99;
            if (lookfor == LOOKFOR_TERM) {
              if (n) {
                amount = n;
              }

              if (!lookfor_break) {
                break;
              }
            }

            //   case xx: x = x + 1;        <- line up with this x
            // ->         y = y + 1;
            //
            //   case xx: if (cond)         <- line up with this if
            // ->              y = y + 1;
            if (n) {
              amount = n;
              l = after_label(get_cursor_line_ptr());
              if (l != NULL && cin_is_cinword(l)) {
                if (theline[0] == '{') {
                  amount += curbuf->b_ind_open_extra;
                } else {
                  amount += curbuf->b_ind_level
                            + curbuf->b_ind_no_brace;
                }
              }
              break;
            }

            // Try to get the indent of a statement before the switch
            // label.  If nothing is found, line up relative to the
            // switch label.
            //      break;              <- may line up with this line
            //   case xx:
            // ->   y = 1;
            scope_amount = get_indent() + (iscase            // XXX
                                           ? curbuf->b_ind_case_code
                                           : curbuf->b_ind_scopedecl_code);
            lookfor = curbuf->b_ind_case_break
                      ? LOOKFOR_NOBREAK : LOOKFOR_ANY;
            continue;
          }

          // Looking for a switch() label or C++ scope declaration,
          // ignore other lines, skip {}-blocks.
          if (lookfor == LOOKFOR_CASE || lookfor == LOOKFOR_SCOPEDECL) {
            if (find_last_paren(l, '{', '}')
                && (trypos = find_start_brace()) != NULL) {
              curwin->w_cursor.lnum = trypos->lnum + 1;
              curwin->w_cursor.col = 0;
            }
            continue;
          }

          // Ignore jump labels with nothing after them.
          if (!curbuf->b_ind_js && cin_islabel()) {
            l = after_label(get_cursor_line_ptr());
            if (l == NULL || cin_nocode(l)) {
              continue;
            }
          }

          // Ignore #defines, #if, etc.
          // Ignore comment and empty lines.
          // (need to get the line again, cin_islabel() may have
          // unlocked it)
          l = get_cursor_line_ptr();
          if (cin_ispreproc_cont(&l, &curwin->w_cursor.lnum, &amount)
              || cin_nocode(l)) {
            continue;
          }

          // Are we at the start of a cpp base class declaration or
          // constructor initialization?
          // XXX
          n = 0;
          if (lookfor != LOOKFOR_TERM && curbuf->b_ind_cpp_baseclass > 0) {
            n = cin_is_cpp_baseclass(&cache_cpp_baseclass);
            l = get_cursor_line_ptr();
          }
          if (n) {
            if (lookfor == LOOKFOR_UNTERM) {
              if (cont_amount > 0) {
                amount = cont_amount;
              } else {
                amount += ind_continuation;
              }
            } else if (theline[0] == '{') {
              // Need to find start of the declaration.
              lookfor = LOOKFOR_UNTERM;
              ind_continuation = 0;
              continue;
            } else {
              // XXX
              amount = get_baseclass_amount(cache_cpp_baseclass.lpos.col);
            }
            break;
          } else if (lookfor == LOOKFOR_CPP_BASECLASS) {
            // only look, whether there is a cpp base class
            // declaration or initialization before the opening brace.
            if (cin_isterminated(l, true, false)) {
              break;
            } else {
              continue;
            }
          }

          // What happens next depends on the line being terminated.
          // If terminated with a ',' only consider it terminating if
          // there is another unterminated statement behind, eg:
          //   123,
          //   sizeof
          //      here
          // Otherwise check whether it is an enumeration or structure
          // initialisation (not indented) or a variable declaration
          // (indented).
          terminated = cin_isterminated(l, false, true);

          if (js_cur_has_key) {
            js_cur_has_key = false;  // only check the first line
            if (curbuf->b_ind_js && terminated == ',') {
              // For Javascript we might be inside an object:
              //   key: something,  <- align with this
              //   key: something
              // or:
              //   key: something +  <- align with this
              //       something,
              //   key: something
              lookfor = LOOKFOR_JS_KEY;
            }
          }
          if (lookfor == LOOKFOR_JS_KEY && cin_has_js_key(l)) {
            amount = get_indent();
            break;
          }
          if (lookfor == LOOKFOR_COMMA) {
            if (tryposBrace != NULL && tryposBrace->lnum
                >= curwin->w_cursor.lnum) {
              break;
            }
            if (terminated == ',') {
              // Line below current line is the one that starts a
              // (possibly broken) line ending in a comma.
              break;
            }
            amount = get_indent();
            if (curwin->w_cursor.lnum - 1 == ourscope) {
              // line above is start of the scope, thus current
              // line is the one that stars a (possibly broken)
              // line ending in a comma.
              break;
            }
          }

          if (terminated == 0 || (lookfor != LOOKFOR_UNTERM
                                  && terminated == ',')) {
            if (lookfor != LOOKFOR_ENUM_OR_INIT
                && (*skipwhite(l) == '[' || l[strlen(l) - 1] == '[')) {
              amount += ind_continuation;
            }
            // If we're in the middle of a paren thing, Go back to the line
            // that starts it so we can get the right prevailing indent
            //     if ( foo &&
            //              bar )

            // Position the cursor over the rightmost paren, so that
            // matching it will take us back to the start of the line.
            // Ignore a match before the start of the block.
            (void)find_last_paren(l, '(', ')');
            trypos = find_match_paren(corr_ind_maxparen(&cur_curpos));
            if (trypos != NULL && (trypos->lnum < tryposBrace->lnum
                                   || (trypos->lnum == tryposBrace->lnum
                                       && trypos->col < tryposBrace->col))) {
              trypos = NULL;
            }

            l = get_cursor_line_ptr();

            // If we are looking for ',', we also look for matching
            // braces.
            if (trypos == NULL && terminated == ',') {
              if (find_last_paren(l, '{', '}')) {
                trypos = find_start_brace();
              }
              l = get_cursor_line_ptr();
            }

            if (trypos != NULL) {
              // Check if we are on a case label now.  This is
              // handled above.
              //     case xx:  if ( asdf &&
              //                        asdf)
              curwin->w_cursor = *trypos;
              l = get_cursor_line_ptr();
              if (cin_iscase(l, false) || cin_isscopedecl(l)) {
                curwin->w_cursor.lnum++;
                curwin->w_cursor.col = 0;
                continue;
              }
            }

            // Skip over continuation lines to find the one to get the
            // indent from
            // char *usethis = "bla{backslash}
            //           bla",
            //      here;
            if (terminated == ',') {
              while (curwin->w_cursor.lnum > 1) {
                l = ml_get(curwin->w_cursor.lnum - 1);
                if (*l == NUL || l[strlen(l) - 1] != '\\') {
                  break;
                }
                curwin->w_cursor.lnum--;
                curwin->w_cursor.col = 0;
              }
              l = get_cursor_line_ptr();
            }

            // Get indent and pointer to text for current line,
            // ignoring any jump label.     XXX
            if (curbuf->b_ind_js) {
              cur_amount = get_indent();
            } else {
              cur_amount = skip_label(curwin->w_cursor.lnum, &l);
            }
            // If this is just above the line we are indenting, and it
            // starts with a '{', line it up with this line.
            //          while (not)
            // ->       {
            //          }
            if (terminated != ',' && lookfor != LOOKFOR_TERM
                && theline[0] == '{') {
              amount = cur_amount;
              // Only add b_ind_open_extra when the current line
              // doesn't start with a '{', which must have a match
              // in the same line (scope is the same).  Probably:
              //        { 1, 2 },
              // ->     { 3, 4 }
              if (*skipwhite(l) != '{') {
                amount += curbuf->b_ind_open_extra;
              }

              if (curbuf->b_ind_cpp_baseclass && !curbuf->b_ind_js) {
                // have to look back, whether it is a cpp base
                // class declaration or initialization
                lookfor = LOOKFOR_CPP_BASECLASS;
                continue;
              }
              break;
            }

            // Check if we are after an "if", "while", etc.
            // Also allow "   } else".
            if (cin_is_cinword(l) || cin_iselse(skipwhite(l))) {
              // Found an unterminated line after an if (), line up
              // with the last one.
              //   if (cond)
              //             100 +
              // ->              here;
              if (lookfor == LOOKFOR_UNTERM
                  || lookfor == LOOKFOR_ENUM_OR_INIT) {
                if (cont_amount > 0) {
                  amount = cont_amount;
                } else {
                  amount += ind_continuation;
                }
                break;
              }

              // If this is just above the line we are indenting, we
              // are finished.
              //            while (not)
              // ->             here;
              // Otherwise this indent can be used when the line
              // before this is terminated.
              //        yyy;
              //        if (stat)
              //            while (not)
              //                xxx;
              // ->     here;
              amount = cur_amount;
              if (theline[0] == '{') {
                amount += curbuf->b_ind_open_extra;
              }
              if (lookfor != LOOKFOR_TERM) {
                amount += curbuf->b_ind_level
                          + curbuf->b_ind_no_brace;
                break;
              }

              // Special trick: when expecting the while () after a
              // do, line up with the while()
              //     do
              //            x = 1;
              // ->  here
              l = skipwhite(get_cursor_line_ptr());
              if (cin_isdo(l)) {
                if (whilelevel == 0) {
                  break;
                }
                whilelevel--;
              }

              // When searching for a terminated line, don't use the
              // one between the "if" and the matching "else".
              // Need to use the scope of this "else".  XXX
              // If whilelevel != 0 continue looking for a "do {".
              if (cin_iselse(l) && whilelevel == 0) {
                // If we're looking at "} else", let's make sure we
                // find the opening brace of the enclosing scope,
                // not the one from "if () {".
                if (*l == '}') {
                  curwin->w_cursor.col =
                    (colnr_T)(l - get_cursor_line_ptr()) + 1;
                }

                if ((trypos = find_start_brace()) == NULL
                    || find_match(LOOKFOR_IF, trypos->lnum)
                    == FAIL) {
                  break;
                }
              }
            } else {
              // If we're below an unterminated line that is not an
              // "if" or something, we may line up with this line or
              // add something for a continuation line, depending on
              // the line before this one.

              // Found two unterminated lines on a row, line up with
              // the last one.
              //   c = 99 +
              //            100 +
              // ->         here;
              if (lookfor == LOOKFOR_UNTERM) {
                // When line ends in a comma add extra indent
                if (terminated == ',') {
                  amount += ind_continuation;
                }
                break;
              }

              if (lookfor == LOOKFOR_ENUM_OR_INIT) {
                // Found two lines ending in ',', lineup with the
                // lowest one, but check for cpp base class
                // declaration/initialization, if it is an
                // opening brace or we are looking just for
                // enumerations/initializations.
                if (terminated == ',') {
                  if (curbuf->b_ind_cpp_baseclass == 0) {
                    break;
                  }

                  lookfor = LOOKFOR_CPP_BASECLASS;
                  continue;
                }

                // Ignore unterminated lines in between, but
                // reduce indent.
                if (amount > cur_amount) {
                  amount = cur_amount;
                }
              } else {
                // Found first unterminated line on a row, may
                // line up with this line, remember its indent
                //          100 +
                // ->       here;
                l = get_cursor_line_ptr();
                amount = cur_amount;

                n = (int)strlen(l);
                if (curbuf->b_ind_js && terminated == ','
                    && (*skipwhite(l) == ']' || (n >= 2 && l[n - 2] == ']'))) {
                  break;
                }

                // If previous line ends in ',', check whether we
                // are in an initialization or enum
                // struct xxx =
                // {
                //      sizeof a,
                //      124 };
                // or a normal possible continuation line.
                // but only, of no other statement has been found
                // yet.
                if (lookfor == LOOKFOR_INITIAL && terminated == ',') {
                  if (curbuf->b_ind_js) {
                    // Search for a line ending in a comma
                    // and line up with the line below it
                    // (could be the current line).
                    // some = [
                    //     1,     <- line up here
                    //     2,
                    // some = [
                    //     3 +    <- line up here
                    //       4 *
                    //        5,
                    //     6,
                    if (cin_iscomment(skipwhite(l))) {
                      break;
                    }
                    lookfor = LOOKFOR_COMMA;
                    trypos = find_match_char('[', curbuf->b_ind_maxparen);
                    if (trypos != NULL) {
                      if (trypos->lnum == curwin->w_cursor.lnum - 1) {
                        // Current line is first inside
                        // [], line up with it.
                        break;
                      }
                      ourscope = trypos->lnum;
                    }
                  } else {
                    lookfor = LOOKFOR_ENUM_OR_INIT;
                    cont_amount = cin_first_id_amount();
                  }
                } else {
                  if (lookfor == LOOKFOR_INITIAL
                      && *l != NUL
                      && l[strlen(l) - 1] == '\\') {
                    // XXX
                    cont_amount = cin_get_equal_amount(curwin->w_cursor.lnum);
                  }
                  if (lookfor != LOOKFOR_TERM
                      && lookfor != LOOKFOR_JS_KEY
                      && lookfor != LOOKFOR_COMMA
                      && raw_string_start != curwin->w_cursor.lnum) {
                    lookfor = LOOKFOR_UNTERM;
                  }
                }
              }
            }
            // Check if we are after a while (cond);
            // If so: Ignore until the matching "do".
          } else if (cin_iswhileofdo_end((uint8_t)terminated)) {  // XXX
            // Found an unterminated line after a while ();, line up
            // with the last one.
            //      while (cond);
            //      100 +               <- line up with this one
            // ->           here;
            if (lookfor == LOOKFOR_UNTERM
                || lookfor == LOOKFOR_ENUM_OR_INIT) {
              if (cont_amount > 0) {
                amount = cont_amount;
              } else {
                amount += ind_continuation;
              }
              break;
            }

            if (whilelevel == 0) {
              lookfor = LOOKFOR_TERM;
              amount = get_indent();                // XXX
              if (theline[0] == '{') {
                amount += curbuf->b_ind_open_extra;
              }
            }
            whilelevel++;
          } else {
            // We are after a "normal" statement.
            // If we had another statement we can stop now and use the
            // indent of that other statement.
            // Otherwise the indent of the current statement may be used,
            // search backwards for the next "normal" statement.

            // Skip single break line, if before a switch label. It
            // may be lined up with the case label.
            if (lookfor == LOOKFOR_NOBREAK
                && cin_isbreak(skipwhite(get_cursor_line_ptr()))) {
              lookfor = LOOKFOR_ANY;
              continue;
            }

            // Handle "do {" line.
            if (whilelevel > 0) {
              l = cin_skipcomment(get_cursor_line_ptr());
              if (cin_isdo(l)) {
                amount = get_indent();                  // XXX
                whilelevel--;
                continue;
              }
            }

            // Found a terminated line above an unterminated line. Add
            // the amount for a continuation line.
            //   x = 1;
            //   y = foo +
            // ->       here;
            // or
            //   int x = 1;
            //   int foo,
            // ->       here;
            if (lookfor == LOOKFOR_UNTERM
                || lookfor == LOOKFOR_ENUM_OR_INIT) {
              if (cont_amount > 0) {
                amount = cont_amount;
              } else {
                amount += ind_continuation;
              }
              break;
            }

            // Found a terminated line above a terminated line or "if"
            // etc. line. Use the amount of the line below us.
            //   x = 1;                         x = 1;
            //   if (asdf)                  y = 2;
            //       while (asdf)         ->here;
            //          here;
            // ->foo;
            if (lookfor == LOOKFOR_TERM) {
              if (!lookfor_break && whilelevel == 0) {
                break;
              }
            } else {
              // First line above the one we're indenting is terminated.
              // To know what needs to be done look further backward for
              // a terminated line.

              // position the cursor over the rightmost paren, so
              // that matching it will take us back to the start of
              // the line.  Helps for:
              //     func(asdr,
              //              asdfasdf);
              //     here;
term_again:
              l = get_cursor_line_ptr();
              if (find_last_paren(l, '(', ')')
                  && (trypos = find_match_paren(curbuf->b_ind_maxparen)) != NULL) {
                // Check if we are on a case label now.  This is
                // handled above.
                //         case xx:  if ( asdf &&
                //                          asdf)
                curwin->w_cursor = *trypos;
                l = get_cursor_line_ptr();
                if (cin_iscase(l, false) || cin_isscopedecl(l)) {
                  curwin->w_cursor.lnum++;
                  curwin->w_cursor.col = 0;
                  continue;
                }
              }

              // When aligning with the case statement, don't align
              // with a statement after it.
              //  case 1: {   <-- don't use this { position
              //        stat;
              //  }
              //  case 2:
              //        stat;
              // }
              iscase = curbuf->b_ind_keep_case_label && cin_iscase(l, false);

              // Get indent and pointer to text for current line,
              // ignoring any jump label.
              amount = skip_label(curwin->w_cursor.lnum, &l);

              if (theline[0] == '{') {
                amount += curbuf->b_ind_open_extra;
              }
              // See remark above: "Only add b_ind_open_extra.."
              l = skipwhite(l);
              if (*l == '{') {
                amount -= curbuf->b_ind_open_extra;
              }
              lookfor = iscase ? LOOKFOR_ANY : LOOKFOR_TERM;

              // When a terminated line starts with "else" skip to
              // the matching "if":
              //       else 3;
              //             indent this;
              // Need to use the scope of this "else".  XXX
              // If whilelevel != 0 continue looking for a "do {".
              if (lookfor == LOOKFOR_TERM
                  && *l != '}'
                  && cin_iselse(l)
                  && whilelevel == 0) {
                if ((trypos = find_start_brace()) == NULL
                    || find_match(LOOKFOR_IF, trypos->lnum)
                    == FAIL) {
                  break;
                }
                continue;
              }

              // If we're at the end of a block, skip to the start of
              // that block.
              l = get_cursor_line_ptr();
              if (find_last_paren(l, '{', '}')           // XXX
                  && (trypos = find_start_brace()) != NULL) {
                curwin->w_cursor = *trypos;
                // if not "else {" check for terminated again
                // but skip block for "} else {"
                l = cin_skipcomment(get_cursor_line_ptr());
                if (*l == '}' || !cin_iselse(l)) {
                  goto term_again;
                }
                curwin->w_cursor.lnum++;
                curwin->w_cursor.col = 0;
              }
            }
          }
        }
      }
    }

    // add extra indent for a comment
    if (cin_iscomment(theline)) {
      amount += curbuf->b_ind_comment;
    }
    // subtract extra left-shift for jump labels
    if (curbuf->b_ind_jump_label > 0 && original_line_islabel) {
      amount -= curbuf->b_ind_jump_label;
    }

    goto theend;
  }

  // Ok -- we're not inside any sort of structure at all!
  //
  // this means we're at the top level, and everything should
  // basically just match where the previous line is, except
  // for the lines immediately following a function declaration,
  // which are K&R-style parameters and need to be indented.

  // if our line starts with an open brace, forget about any
  // prevailing indent and make sure it looks like the start
  // of a function

  if (theline[0] == '{') {
    amount = curbuf->b_ind_first_open;
    goto theend;
  }
  // If the NEXT line is a function declaration, the current
  // line needs to be indented as a function type spec.
  // Don't do this if the current line looks like a comment or if the
  // current line is terminated, ie. ends in ';', or if the current line
  // contains { or }: "void f() {\n if (1)"
  if (cur_curpos.lnum < curbuf->b_ml.ml_line_count
      && !cin_nocode(theline)
      && vim_strchr(theline, '{') == NULL
      && vim_strchr(theline, '}') == NULL
      && !cin_ends_in(theline, ":")
      && !cin_ends_in(theline, ",")
      && cin_isfuncdecl(NULL, cur_curpos.lnum + 1, cur_curpos.lnum + 1)
      && !cin_isterminated(theline, false, true)) {
    amount = curbuf->b_ind_func_type;
    goto theend;
  }

  // search backwards until we find something we recognize
  amount = 0;
  curwin->w_cursor = cur_curpos;
  while (curwin->w_cursor.lnum > 1) {
    curwin->w_cursor.lnum--;
    curwin->w_cursor.col = 0;

    l = get_cursor_line_ptr();

    // If we're in a comment or raw string now, skip to the start
    // of it.
    // XXX
    if ((trypos = ind_find_start_CORS(NULL)) != NULL) {
      curwin->w_cursor.lnum = trypos->lnum + 1;
      curwin->w_cursor.col = 0;
      continue;
    }

    // Are we at the start of a cpp base class declaration or
    // constructor initialization?  XXX
    n = 0;
    if (curbuf->b_ind_cpp_baseclass != 0) {
      n = cin_is_cpp_baseclass(&cache_cpp_baseclass);
      l = get_cursor_line_ptr();
    }
    if (n) {
      // XXX
      amount = get_baseclass_amount(cache_cpp_baseclass.lpos.col);
      break;
    }

    // Skip preprocessor directives and blank lines.
    if (cin_ispreproc_cont(&l, &curwin->w_cursor.lnum, &amount)) {
      continue;
    }

    if (cin_nocode(l)) {
      continue;
    }

    // If the previous line ends in ',', use one level of
    // indentation:
    // int foo,
    //     bar;
    // do this before checking for '}' in case of eg.
    // enum foobar
    // {
    //   ...
    // } foo,
    //   bar;
    if (cin_ends_in(l, ",")
        || (*l != NUL && (n = (uint8_t)l[strlen(l) - 1]) == '\\')) {
      // take us back to opening paren
      if (find_last_paren(l, '(', ')')
          && (trypos = find_match_paren(curbuf->b_ind_maxparen)) != NULL) {
        curwin->w_cursor = *trypos;
      }

      // For a line ending in ',' that is a continuation line go
      // back to the first line with a backslash:
      // char *foo = "bla{backslash}
      //           bla",
      //      here;
      while (n == 0 && curwin->w_cursor.lnum > 1) {
        l = ml_get(curwin->w_cursor.lnum - 1);
        if (*l == NUL || l[strlen(l) - 1] != '\\') {
          break;
        }
        curwin->w_cursor.lnum--;
        curwin->w_cursor.col = 0;
      }

      amount = get_indent();                    // XXX

      if (amount == 0) {
        amount = cin_first_id_amount();
      }
      if (amount == 0) {
        amount = ind_continuation;
      }
      break;
    }

    // If the line looks like a function declaration, and we're
    // not in a comment, put it the left margin.
    if (cin_isfuncdecl(NULL, cur_curpos.lnum, 0)) {        // XXX
      break;
    }
    l = get_cursor_line_ptr();

    // Finding the closing '}' of a previous function.  Put
    // current line at the left margin.  For when 'cino' has "fs".
    if (*skipwhite(l) == '}') {
      break;
    }

    //                      (matching {)
    // If the previous line ends on '};' (maybe followed by
    // comments) align at column 0.  For example:
    // char *string_array[] = { "foo",
    //     / * x * / "b};ar" }; / * foobar * /
    if (cin_ends_in(l, "};")) {
      break;
    }

    // If the previous line ends on '[' we are probably in an
    // array constant:
    // something = [
    //     234,  <- extra indent
    if (cin_ends_in(l, "[")) {
      amount = get_indent() + ind_continuation;
      break;
    }

    // Find a line only has a semicolon that belongs to a previous
    // line ending in '}', e.g. before an #endif.  Don't increase
    // indent then.
    if (*(look = skipwhite(l)) == ';' && cin_nocode(look + 1)) {
      pos_T curpos_save = curwin->w_cursor;

      while (curwin->w_cursor.lnum > 1) {
        look = ml_get(--curwin->w_cursor.lnum);
        if (!(cin_nocode(look)
              || cin_ispreproc_cont(&look, &curwin->w_cursor.lnum, &amount))) {
          break;
        }
      }
      if (curwin->w_cursor.lnum > 0 && cin_ends_in(look, "}")) {
        break;
      }

      curwin->w_cursor = curpos_save;
    }

    // If the PREVIOUS line is a function declaration, the current
    // line (and the ones that follow) needs to be indented as
    // parameters.
    if (cin_isfuncdecl(&l, curwin->w_cursor.lnum, 0)) {
      amount = curbuf->b_ind_param;
      break;
    }

    // If the previous line ends in ';' and the line before the
    // previous line ends in ',' or '\', ident to column zero:
    // int foo,
    //     bar;
    // indent_to_0 here;
    if (cin_ends_in(l, ";")) {
      l = ml_get(curwin->w_cursor.lnum - 1);
      if (cin_ends_in(l, ",")
          || (*l != NUL && l[strlen(l) - 1] == '\\')) {
        break;
      }
      l = get_cursor_line_ptr();
    }

    // Doesn't look like anything interesting -- so just
    // use the indent of this line.
    //
    // Position the cursor over the rightmost paren, so that
    // matching it will take us back to the start of the line.
    (void)find_last_paren(l, '(', ')');

    if ((trypos = find_match_paren(curbuf->b_ind_maxparen)) != NULL) {
      curwin->w_cursor = *trypos;
    }
    amount = get_indent();              // XXX
    break;
  }

  // add extra indent for a comment
  if (cin_iscomment(theline)) {
    amount += curbuf->b_ind_comment;
  }

  // add extra indent if the previous line ended in a backslash:
  //          "asdfasdf{backslash}
  //              here";
  //        char *foo = "asdf{backslash}
  //                     here";
  if (cur_curpos.lnum > 1) {
    l = ml_get(cur_curpos.lnum - 1);
    if (*l != NUL && l[strlen(l) - 1] == '\\') {
      cur_amount = cin_get_equal_amount(cur_curpos.lnum - 1);
      if (cur_amount > 0) {
        amount = cur_amount;
      } else if (cur_amount == 0) {
        amount += ind_continuation;
      }
    }
  }

theend:
  if (amount < 0) {
    amount = 0;
  }

laterend:
  // put the cursor back where it belongs
  curwin->w_cursor = cur_curpos;

  xfree(linecopy);

  return amount;
}

static int find_match(int lookfor, linenr_T ourscope)
{
  return rs_find_match(lookfor, ourscope);
}

/// Check that "cinkeys" contains the key "keytyped",
/// when == '*': Only if key is preceded with '*' (indent before insert)
/// when == '!': Only if key is preceded with '!' (don't insert)
/// when == ' ': Only if key is not preceded with '*' (indent afterwards)
///
/// "keytyped" can have a few special values:
/// KEY_OPEN_FORW :
/// KEY_OPEN_BACK :
/// KEY_COMPLETE  : Just finished completion.
///
/// @param  keytyped       key that was typed
/// @param  when           condition on when to perform the check
/// @param  line_is_empty  when true, accept keys with '0' before them.
bool in_cinkeys(int keytyped, int when, bool line_is_empty)
{
  char *look;
  bool try_match;
  bool try_match_word;
  char *p;
  bool icase;

  if (keytyped == NUL) {
    // Can happen with CTRL-Y and CTRL-E on a short line.
    return false;
  }

  if (*curbuf->b_p_inde != NUL) {
    look = curbuf->b_p_indk;            // 'indentexpr' set: use 'indentkeys'
  } else {
    look = curbuf->b_p_cink;            // 'indentexpr' empty: use 'cinkeys'
  }
  while (*look) {
    // Find out if we want to try a match with this key, depending on
    // 'when' and a '*' or '!' before the key.
    switch (when) {
    case '*':
      try_match = (*look == '*'); break;
    case '!':
      try_match = (*look == '!'); break;
    default:
      try_match = (*look != '*'); break;
    }
    if (*look == '*' || *look == '!') {
      look++;
    }

    // If there is a '0', only accept a match if the line is empty.
    // But may still match when typing last char of a word.
    if (*look == '0') {
      try_match_word = try_match;
      if (!line_is_empty) {
        try_match = false;
      }
      look++;
    } else {
      try_match_word = false;
    }

    // Does it look like a control character?
    if (*look == '^' && look[1] >= '?' && look[1] <= '_') {
      if (try_match && keytyped == CTRL_CHR(look[1])) {
        return true;
      }
      look += 2;

      // 'o' means "o" command, open forward.
      // 'O' means "O" command, open backward.
    } else if (*look == 'o') {
      if (try_match && keytyped == KEY_OPEN_FORW) {
        return true;
      }
      look++;
    } else if (*look == 'O') {
      if (try_match && keytyped == KEY_OPEN_BACK) {
        return true;
      }
      look++;

      // 'e' means to check for "else" at start of line and just before the
      // cursor.
    } else if (*look == 'e') {
      if (try_match && keytyped == 'e' && curwin->w_cursor.col >= 4) {
        p = get_cursor_line_ptr();
        if (skipwhite(p) == p + curwin->w_cursor.col - 4
            && strncmp(p + curwin->w_cursor.col - 4, "else", 4) == 0) {
          return true;
        }
      }
      look++;

      // ':' only causes an indent if it is at the end of a label or case
      // statement, or when it was before typing the ':' (to fix
      // class::method for C++).
    } else if (*look == ':') {
      if (try_match && keytyped == ':') {
        p = get_cursor_line_ptr();
        if (cin_iscase(p, false) || cin_isscopedecl(p) || cin_islabel()) {
          return true;
        }
        // Need to get the line again after cin_islabel().
        p = get_cursor_line_ptr();
        if (curwin->w_cursor.col > 2
            && p[curwin->w_cursor.col - 1] == ':'
            && p[curwin->w_cursor.col - 2] == ':') {
          p[curwin->w_cursor.col - 1] = ' ';
          const bool i = cin_iscase(p, false)
                         || cin_isscopedecl(p)
                         || cin_islabel();
          p = get_cursor_line_ptr();
          p[curwin->w_cursor.col - 1] = ':';
          if (i) {
            return true;
          }
        }
      }
      look++;

      // Is it a key in <>, maybe?
    } else if (*look == '<') {
      if (try_match) {
        // make up some named keys <o>, <O>, <e>, <0>, <>>, <<>, <*>,
        // <:> and <!> so that people can re-indent on o, O, e, 0, <,
        // >, *, : and ! keys if they really really want to.
        if (vim_strchr("<>!*oOe0:", (uint8_t)look[1]) != NULL
            && keytyped == look[1]) {
          return true;
        }

        if (keytyped == get_special_key_code(look + 1)) {
          return true;
        }
      }
      while (*look && *look != '>') {
        look++;
      }
      while (*look == '>') {
        look++;
      }
      // Is it a word: "=word"?
    } else if (*look == '=' && look[1] != ',' && look[1] != NUL) {
      look++;
      if (*look == '~') {
        icase = true;
        look++;
      } else {
        icase = false;
      }
      p = vim_strchr(look, ',');
      if (p == NULL) {
        p = look + strlen(look);
      }
      if ((try_match || try_match_word)
          && curwin->w_cursor.col >= (colnr_T)(p - look)) {
        bool match = false;

        if (keytyped == KEY_COMPLETE) {
          char *n, *s;

          // Just completed a word, check if it starts with "look".
          // search back for the start of a word.
          char *line = get_cursor_line_ptr();
          for (s = line + curwin->w_cursor.col; s > line; s = n) {
            n = mb_prevptr(line, s);
            if (!vim_iswordp(n)) {
              break;
            }
          }
          assert(p >= look && (uintmax_t)(p - look) <= SIZE_MAX);
          if (s + (p - look) <= line + curwin->w_cursor.col
              && (icase
                  ? mb_strnicmp(s, look, (size_t)(p - look))
                  : strncmp(s, look, (size_t)(p - look))) == 0) {
            match = true;
          }
        } else {
          // TODO(@brammool): multi-byte
          if (keytyped == (int)(uint8_t)p[-1]
              || (icase && keytyped < 256 && keytyped >= 0
                  && TOLOWER_LOC(keytyped) == TOLOWER_LOC((uint8_t)p[-1]))) {
            char *line = get_cursor_pos_ptr();
            assert(p >= look && (uintmax_t)(p - look) <= SIZE_MAX);
            if ((curwin->w_cursor.col == (colnr_T)(p - look)
                 || !vim_iswordc((uint8_t)line[-(p - look) - 1]))
                && (icase
                    ? mb_strnicmp(line - (p - look), look, (size_t)(p - look))
                    : strncmp(line - (p - look), look, (size_t)(p - look))) == 0) {
              match = true;
            }
          }
        }
        if (match && try_match_word && !try_match) {
          // "0=word": Check if there are only blanks before the
          // word.
          if (getwhitecols_curline() !=
              (int)(curwin->w_cursor.col - (p - look))) {
            match = false;
          }
        }
        if (match) {
          return true;
        }
      }
      look = p;

      // Ok, it's a boring generic character.
    } else {
      if (try_match && (uint8_t)(*look) == keytyped) {
        return true;
      }
      if (*look != NUL) {
        look++;
      }
    }

    // Skip over ", ".
    look = (char *)rs_skip_to_option_part(look);
  }
  return false;
}

// Do C or expression indenting on the current line.
void do_c_expr_indent(void)
{
  if (*curbuf->b_p_inde != NUL) {
    fixthisline(get_expr_indent);
  } else {
    fixthisline(get_c_indent);
  }
}

/// "cindent(lnum)" function
void f_cindent(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  pos_T pos = curwin->w_cursor;
  linenr_T lnum = tv_get_lnum(argvars);
  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count) {
    curwin->w_cursor.lnum = lnum;
    rettv->vval.v_number = get_c_indent();
    curwin->w_cursor = pos;
  } else {
    rettv->vval.v_number = -1;
  }
}
