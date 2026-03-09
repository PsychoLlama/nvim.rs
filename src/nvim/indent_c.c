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

// Phase 6: get_c_indent in Rust
extern int rs_get_c_indent(const CindentOptions *opts);

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
// Delegates to rs_get_c_indent() in Rust.
int get_c_indent(void)
{
  // Build CindentOptions from curbuf->b_ind_* fields
  CindentOptions opts;
  opts.ind_level = curbuf->b_ind_level;
  opts.ind_open_imag = curbuf->b_ind_open_imag;
  opts.ind_no_brace = curbuf->b_ind_no_brace;
  opts.ind_first_open = curbuf->b_ind_first_open;
  opts.ind_open_extra = curbuf->b_ind_open_extra;
  opts.ind_close_extra = curbuf->b_ind_close_extra;
  opts.ind_open_left_imag = curbuf->b_ind_open_left_imag;
  opts.ind_jump_label = curbuf->b_ind_jump_label;
  opts.ind_case = curbuf->b_ind_case;
  opts.ind_case_code = curbuf->b_ind_case_code;
  opts.ind_case_break = curbuf->b_ind_case_break;
  opts.ind_scopedecl = curbuf->b_ind_scopedecl;
  opts.ind_scopedecl_code = curbuf->b_ind_scopedecl_code;
  opts.ind_param = curbuf->b_ind_param;
  opts.ind_func_type = curbuf->b_ind_func_type;
  opts.ind_cpp_baseclass = curbuf->b_ind_cpp_baseclass;
  opts.ind_continuation = curbuf->b_ind_continuation;
  opts.ind_unclosed = curbuf->b_ind_unclosed;
  opts.ind_unclosed2 = curbuf->b_ind_unclosed2;
  opts.ind_unclosed_noignore = curbuf->b_ind_unclosed_noignore;
  opts.ind_unclosed_wrapped = curbuf->b_ind_unclosed_wrapped;
  opts.ind_unclosed_whiteok = curbuf->b_ind_unclosed_whiteok;
  opts.ind_matching_paren = curbuf->b_ind_matching_paren;
  opts.ind_paren_prev = curbuf->b_ind_paren_prev;
  opts.ind_comment = curbuf->b_ind_comment;
  opts.ind_in_comment = curbuf->b_ind_in_comment;
  opts.ind_in_comment2 = curbuf->b_ind_in_comment2;
  opts.ind_maxparen = curbuf->b_ind_maxparen;
  opts.ind_maxcomment = curbuf->b_ind_maxcomment;
  opts.ind_java = curbuf->b_ind_java;
  opts.ind_js = curbuf->b_ind_js;
  opts.ind_keep_case_label = curbuf->b_ind_keep_case_label;
  opts.ind_cpp_namespace = curbuf->b_ind_cpp_namespace;
  opts.ind_if_for_while = curbuf->b_ind_if_for_while;
  opts.ind_hash_comment = curbuf->b_ind_hash_comment;
  opts.ind_cpp_extern_c = curbuf->b_ind_cpp_extern_c;
  opts.ind_pragma = curbuf->b_ind_pragma;

  return rs_get_c_indent(&opts);
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
