// Code to handle tags and the tag stack

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/help.h"
#include "nvim/highlight_defs.h"
#include "nvim/input.h"
#include "nvim/insexpand.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Structure to hold pointers to various items in a tag line.
typedef struct {
  // filled in by parse_tag_line():
  char *tagname;        // start of tag name (skip "file:")
  char *tagname_end;    // char after tag name
  char *fname;          // first char of file name
  char *fname_end;      // char after file name
  char *command;        // first char of command
  // filled in by parse_match():
  char *command_end;    // first char after command
  char *tag_fname;      // file name of the tags file. This is used
  // when 'tr' is set.
  char *tagkind;          // "kind:" value
  char *tagkind_end;      // end of tagkind
  char *user_data;        // user_data string
  char *user_data_end;    // end of user_data
  linenr_T tagline;       // "line:" value
} tagptrs_T;

// Structure to hold info about the tag pattern being used.
typedef struct {
  char *pat;            // the pattern
  int len;              // length of pat[]
  char *head;           // start of pattern head
  int headlen;          // length of head[]
  regmatch_T regmatch;  // regexp program, may be NULL
} pat_T;

// The matching tags are first stored in one of the hash tables.  In
// which one depends on the priority of the match.
// ht_match[] is used to find duplicates, ga_match[] to keep them in sequence.
// At the end, the matches from ga_match[] are concatenated, to make a list
// sorted on priority.
enum {
  MT_ST_CUR = 0,  // static match in current file
  MT_GL_CUR = 1,  // global match in current file
  MT_GL_OTH = 2,  // global match in other file
  MT_ST_OTH = 3,  // static match in other file
  MT_IC_OFF = 4,  // add for icase match
  MT_RE_OFF = 8,  // add for regexp match
  MT_MASK = 7,    // mask for printing priority
  MT_COUNT = 16,
};

static char *mt_names[MT_COUNT/2] =
{ "FSC", "F C", "F  ", "FS ", " SC", "  C", "   ", " S " };

#define NOTAGFILE       99              // return value for jumpto_tag
static char *nofile_fname = NULL;       // fname for NOTAGFILE error

/// Return values used when reading lines from a tags file.
typedef enum {
  TAGS_READ_SUCCESS = 1,
  TAGS_READ_EOF,
  TAGS_READ_IGNORE,
} tags_read_status_T;

/// States used during a tags search
typedef enum {
  TS_START,         ///< at start of file
  TS_LINEAR,        ///< linear searching forward, till EOF
  TS_BINARY,        ///< binary searching
  TS_SKIP_BACK,     ///< skipping backwards
  TS_STEP_FORWARD,  ///< stepping forwards
} tagsearch_state_T;

/// Binary search file offsets in a tags file
typedef struct {
  off_T low_offset;        ///< offset for first char of first line that
                           ///< could match
  off_T high_offset;       ///< offset of char after last line that could
                           ///< match
  off_T curr_offset;       ///< Current file offset in search range
  off_T curr_offset_used;  ///< curr_offset used when skipping back
  off_T match_offset;      ///< Where the binary search found a tag
  int low_char;            ///< first char at low_offset
  int high_char;           ///< first char at high_offset
} tagsearch_info_T;

/// Return values used when matching tags against a pattern.
typedef enum {
  TAG_MATCH_SUCCESS = 1,
  TAG_MATCH_FAIL,
  TAG_MATCH_STOP,
  TAG_MATCH_NEXT,
} tagmatch_status_T;

/// Arguments used for matching tags read from a tags file against a pattern.
typedef struct {
  int matchoff;      ///< tag match offset
  bool match_re;     ///< true if the tag matches a regexp
  bool match_no_ic;  ///< true if the tag matches with case
  bool has_re;       ///< regular expression used
  bool sortic;       ///< tags file sorted ignoring case (foldcase)
  bool sort_error;   ///< tags file not sorted
} findtags_match_args_T;

/// State information used during a tag search
typedef struct {
  tagsearch_state_T state;       ///< tag search state
  bool stop_searching;           ///< stop when match found or error
  pat_T *orgpat;                 ///< holds unconverted pattern info
  char *lbuf;                    ///< line buffer
  int lbuf_size;                 ///< length of lbuf
  char *tag_fname;               ///< name of the tag file
  FILE *fp;                      ///< current tags file pointer
  int flags;                     ///< flags used for tag search
  int tag_file_sorted;           ///< !_TAG_FILE_SORTED value
  bool get_searchpat;            ///< used for 'showfulltag'
  bool help_only;                ///< only search for help tags
  bool did_open;                 ///< did open a tag file
  int mincount;                  ///< MAXCOL: find all matches
                                 ///< other: minimal number of matches
  bool linear;                   ///< do a linear search
  vimconv_T vimconv;
  char help_lang[3];             ///< lang of current tags file
  int help_pri;                  ///< help language priority
  char *help_lang_find;          ///< lang to be found
  bool is_txt;                   ///< flag of file extension
  int match_count;               ///< number of matches found
  garray_T ga_match[MT_COUNT];   ///< stores matches in sequence
  hashtab_T ht_match[MT_COUNT];  ///< stores matches by key
} findtags_state_T;

// ============================================================================
// Rust FFI accessor functions
// ============================================================================

/// Get tag stack length from win_T for Rust
int nvim_win_get_tagstacklen(const void *wp_void)
{
  const win_T *wp = (const win_T *)wp_void;
  return wp->w_tagstacklen;
}

/// Get tag stack index from win_T for Rust
int nvim_win_get_tagstackidx(const void *wp_void)
{
  const win_T *wp = (const win_T *)wp_void;
  return wp->w_tagstackidx;
}

/// Get tag stack entry at index from win_T for Rust
void *nvim_win_get_tagstack_entry(const void *wp_void, int idx)
{
  const win_T *wp = (const win_T *)wp_void;
  return (void *)&wp->w_tagstack[idx];
}

/// Get tagname from taggy_T for Rust
const char *nvim_taggy_get_tagname(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->tagname;
}

/// Get cur_match from taggy_T for Rust
int nvim_taggy_get_cur_match(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->cur_match;
}

/// Get cur_fnum from taggy_T for Rust
int nvim_taggy_get_cur_fnum(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->cur_fnum;
}

/// Get fmark pointer from taggy_T for Rust
void *nvim_taggy_get_fmark(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return (void *)&tg->fmark;
}

/// Get user_data from taggy_T for Rust
const char *nvim_taggy_get_user_data(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->user_data;
}

/// Get lnum from fmark_T for Rust
linenr_T nvim_fmark_get_lnum(const void *fm_void)
{
  const fmark_T *fm = (const fmark_T *)fm_void;
  return fm->mark.lnum;
}

/// Get col from fmark_T for Rust
int nvim_fmark_get_col(const void *fm_void)
{
  const fmark_T *fm = (const fmark_T *)fm_void;
  return fm->mark.col;
}

/// Get fnum from fmark_T for Rust
int nvim_fmark_get_fnum(const void *fm_void)
{
  const fmark_T *fm = (const fmark_T *)fm_void;
  return fm->fnum;
}

// ============================================================================
// Setter functions for Rust tag stack operations
// ============================================================================

/// Set tag stack length for win_T from Rust
void nvim_win_set_tagstacklen(void *wp_void, int len)
{
  win_T *wp = (win_T *)wp_void;
  wp->w_tagstacklen = len;
}

/// Set tag stack index for win_T from Rust
void nvim_win_set_tagstackidx(void *wp_void, int idx)
{
  win_T *wp = (win_T *)wp_void;
  wp->w_tagstackidx = idx;
}

/// Set tagname in taggy_T from Rust
void nvim_taggy_set_tagname(void *tg_void, char *name)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->tagname = name;
}

/// Set cur_match in taggy_T from Rust
void nvim_taggy_set_cur_match(void *tg_void, int match_idx)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->cur_match = match_idx;
}

/// Set cur_fnum in taggy_T from Rust
void nvim_taggy_set_cur_fnum(void *tg_void, int fnum)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->cur_fnum = fnum;
}

/// Set user_data in taggy_T from Rust
void nvim_taggy_set_user_data(void *tg_void, char *data)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->user_data = data;
}

/// Get fmark lnum from taggy_T for Rust
linenr_T nvim_taggy_get_fmark_lnum(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->fmark.mark.lnum;
}

/// Get fmark col from taggy_T for Rust
int nvim_taggy_get_fmark_col(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->fmark.mark.col;
}

/// Get fmark fnum from taggy_T for Rust
int nvim_taggy_get_fmark_fnum(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->fmark.fnum;
}

/// Set fmark lnum in taggy_T from Rust
void nvim_taggy_set_fmark_lnum(void *tg_void, linenr_T lnum)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->fmark.mark.lnum = lnum;
}

/// Set fmark col in taggy_T from Rust
void nvim_taggy_set_fmark_col(void *tg_void, int col)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->fmark.mark.col = col;
}

/// Set fmark fnum in taggy_T from Rust
void nvim_taggy_set_fmark_fnum(void *tg_void, int fnum)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg->fmark.fnum = fnum;
}

/// Get state from findtags_state_T for Rust
int nvim_findtags_get_state(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return (int)st->state;
}

/// Get match_count from findtags_state_T for Rust
int nvim_findtags_get_match_count(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->match_count;
}

/// Get help_only flag from findtags_state_T for Rust
bool nvim_findtags_get_help_only(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->help_only;
}

/// Get linear flag from findtags_state_T for Rust
bool nvim_findtags_get_linear(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->linear;
}

/// Get tag_file_sorted from findtags_state_T for Rust
int nvim_findtags_get_tag_file_sorted(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->tag_file_sorted;
}

/// Get the 'taglength' option value for Rust
int64_t nvim_get_p_tl(void)
{
  return p_tl;
}

// ============================================================================
// Rust FFI accessor functions for findtags_state_T initialization (Phase 2)
// ============================================================================

/// Allocate and set tag_fname field
void nvim_findtags_init_tag_fname(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->tag_fname = xmalloc(MAXPATHL + 1);
}

/// Set fp to NULL
void nvim_findtags_set_fp_null(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->fp = NULL;
}

/// Allocate orgpat and initialize it with pattern
void nvim_findtags_init_orgpat(void *st_void, char *pat)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->orgpat = xmalloc(sizeof(pat_T));
  st->orgpat->pat = pat;
  st->orgpat->len = (int)strlen(pat);
  st->orgpat->regmatch.regprog = NULL;
}

/// Set scalar fields of findtags_state_T
void nvim_findtags_set_fields(void *st_void, int flags, int mincount)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->flags = flags;
  st->tag_file_sorted = NUL;
  st->help_lang_find = NULL;
  st->is_txt = false;
  st->did_open = false;
  st->help_only = (flags & TAG_HELP);
  st->get_searchpat = false;
  st->help_lang[0] = NUL;
  st->help_pri = 0;
  st->mincount = mincount;
  st->lbuf_size = LSIZE;
  st->lbuf = xmalloc((size_t)st->lbuf_size);
  st->match_count = 0;
  st->stop_searching = false;
}

/// Initialize ga_match and ht_match arrays
void nvim_findtags_init_match_arrays(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  for (int mtt = 0; mtt < MT_COUNT; mtt++) {
    ga_init(&st->ga_match[mtt], sizeof(char *), 100);
    hash_init(&st->ht_match[mtt]);
  }
}

/// Free findtags_state_T inner resources
void nvim_findtags_state_free_inner(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  xfree(st->tag_fname);
  xfree(st->lbuf);
  vim_regfree(st->orgpat->regmatch.regprog);
  xfree(st->orgpat);
}

// ============================================================================
// Rust FFI accessor functions for findtags_match_args_T (Phase 2)
// ============================================================================

/// Initialize findtags_match_args_T
void nvim_findtags_matchargs_init(void *margs_void, int flags)
{
  findtags_match_args_T *margs = (findtags_match_args_T *)margs_void;
  margs->matchoff = 0;
  margs->match_re = false;
  margs->match_no_ic = false;
  margs->has_re = (flags & TAG_REGEXP);
  margs->sortic = false;
  margs->sort_error = false;
}

// ============================================================================
// Rust FFI accessor functions for tag file iteration (functions not using tag_fnames)
// ============================================================================

/// Check if the current buffer is a help buffer
bool nvim_curbuf_is_help(void)
{
  return curbuf->b_help;
}

/// Get the 'helpfile' option value
const char *nvim_get_p_hf(void)
{
  return p_hf;
}

/// Get the buffer-local 'tags' option
const char *nvim_get_curbuf_tags(void)
{
  return curbuf->b_p_tags;
}

/// Get the global 'tags' option
const char *nvim_get_p_tags(void)
{
  return p_tags;
}

/// Get path_tail for Rust
char *nvim_path_tail(char *path)
{
  return path_tail(path);
}

/// Simplify filename for Rust
void nvim_simplify_filename(char *fname)
{
  simplify_filename(fname);
}

/// Initialize vim_findfile for Rust
void *nvim_vim_findfile_init(const char *path, const char *filename, size_t filename_len,
                              const char *stopdirs, int level, bool free_visited,
                              int find_what, void *search_ctx, bool tagfile,
                              const char *buf_ffname)
{
  return vim_findfile_init((char *)path, (char *)filename, filename_len,
                           (char *)stopdirs, level, free_visited,
                           find_what, search_ctx, tagfile, (char *)buf_ffname);
}

/// Find next file for Rust
char *nvim_vim_findfile(void *search_ctx)
{
  return vim_findfile(search_ctx);
}

/// Cleanup vim_findfile context for Rust
void nvim_vim_findfile_cleanup(void *search_ctx)
{
  vim_findfile_cleanup(search_ctx);
}

/// Get stop directory from path for Rust
char *nvim_vim_findfile_stopdir(char *buf)
{
  return vim_findfile_stopdir(buf);
}

/// Get current buffer's full file name for Rust
const char *nvim_get_curbuf_ffname(void)
{
  return curbuf->b_ffname;
}

/// Copy next part of option value for Rust
void nvim_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep)
{
  copy_option_part(option, buf, maxlen, (char *)sep);
}

// ============================================================================
// Rust FFI accessor functions for jump.rs
// ============================================================================

/// Check if a path exists for Rust (tag module)
bool nvim_tag_path_exists(const char *path)
{
  return os_path_exists(path);
}

/// Check if there's a BufReadCmd autocmd for this file
bool nvim_has_bufreadcmd(const char *fname)
{
  return has_autocmd(EVENT_BUFREADCMD, fname, NULL);
}

/// Get the postponed_split global
int nvim_get_postponed_split(void)
{
  return postponed_split;
}

/// Set the postponed_split global
void nvim_set_postponed_split(int val)
{
  postponed_split = val;
}

/// Get the g_do_tagpreview global
int nvim_get_g_do_tagpreview(void)
{
  return g_do_tagpreview;
}

/// Set the g_do_tagpreview global
void nvim_set_g_do_tagpreview(int val)
{
  g_do_tagpreview = val;
}

/// Check if buffer can be set (with forceit flag)
bool nvim_check_can_set_curbuf_forceit(int forceit)
{
  return check_can_set_curbuf_forceit(forceit);
}

/// Set the nofile_fname (for error reporting)
void nvim_set_nofile_fname(const char *fname)
{
  xfree(nofile_fname);
  nofile_fname = fname != NULL ? xstrdup(fname) : NULL;
}

/// Get the nofile_fname (for error reporting)
const char *nvim_get_nofile_fname(void)
{
  return nofile_fname;
}

// ============================================================================
// Rust FFI function declarations
// ============================================================================

extern void rs_tagstack_clear_entry(void *tg);
extern void rs_tagstack_clear(void *wp);
extern void rs_tagstack_shift(void *wp);
extern void rs_tagstack_push(void *wp, char *tagname, int cur_fnum, int cur_match,
                             linenr_T mark_lnum, int mark_col, int fnum, char *user_data);
extern void rs_tagstack_set_idx(void *wp, int idx);
extern void rs_tagstack_truncate(void *wp);

// Parse functions
extern int rs_parse_tag_line(char *lbuf, tagptrs_T *tagp);
extern int rs_parse_match(char *lbuf, tagptrs_T *tagp);
extern bool rs_test_for_static(const tagptrs_T *tagp);
extern size_t rs_matching_line_len(const char *lbuf);
extern int rs_find_extra(char **pp);

// Phase 1 leaf utilities
extern void rs_tag_freematch(void);
extern void rs_taglen_advance(int l);
extern int rs_tag_strnicmp(const char *s1, const char *s2, size_t len);
extern char *rs_tag_full_fname(tagptrs_T *tagp);
extern int rs_test_for_current(char *fname, char *fname_end, char *tag_fname, char *buf_ffname);
extern void rs_free_tag_stuff(void);
extern void rs_tagname_free(void *tnp);

// Phase 2 pattern/state initialization
extern void rs_prepare_pats(pat_T *pats, bool has_re);
extern void rs_findtags_state_init(findtags_state_T *st, char *pat, int flags, int mincount);
extern void rs_findtags_state_free(findtags_state_T *st);
extern void rs_findtags_matchargs_init(findtags_match_args_T *margs, int flags);

// Phase 3 tag file enumeration and filename expansion
extern int rs_get_tagfname(tagname_T *tnp, int first, char *buf);
extern bool rs_found_tagfile_cb(int num_fnames, char **fnames, bool all, void *cookie);
extern char *rs_expand_tag_fname(char *fname, char *tag_fname, bool expand);

// Phase 4 search state machine — file reading and header parsing
extern int rs_findtags_get_next_line(findtags_state_T *st, tagsearch_info_T *sinfo_p);
extern bool rs_findtags_hdr_parse(findtags_state_T *st);
extern bool rs_findtags_start_state_handler(findtags_state_T *st, bool *sortic,
                                            tagsearch_info_T *sinfo_p);
extern void rs_findtags_string_convert(findtags_state_T *st);

// Phase 5 search state machine — line parsing and matching
extern int rs_findtags_parse_line(findtags_state_T *st, tagptrs_T *tagpp,
                                  findtags_match_args_T *margs, tagsearch_info_T *sinfo_p);
extern bool rs_findtags_match_tag(findtags_state_T *st, tagptrs_T *tagpp,
                                  findtags_match_args_T *margs);
extern void rs_findtags_add_match(findtags_state_T *st, tagptrs_T *tagpp,
                                  findtags_match_args_T *margs, char *buf_ffname, hash_T *hash);
extern int rs_findtags_copy_matches(findtags_state_T *st, char ***matchesp);

// Phase 6 search orchestration
extern bool rs_findtags_in_help_init(findtags_state_T *st);
extern void rs_findtags_get_all_tags(findtags_state_T *st, findtags_match_args_T *margs,
                                     char *buf_ffname);
extern void rs_findtags_in_file(findtags_state_T *st, int flags, char *buf_ffname);

// Phase 7 Rust implementations
extern void rs_print_tag_list(bool new_tag, bool use_tagstack, int num_matches, char **matches);
extern int rs_add_llist_tags(const char *tag, int num_matches, char **matches);
extern void rs_do_tags(void);
extern int rs_add_tag_field(dict_T *dict, const char *field_name, const char *start, const char *end);

// Phase 8 Rust implementations
extern int rs_get_tags(void *list, char *pat, char *buf_fname);
extern void rs_get_tag_details(void *tag, void *retdict);
extern void rs_get_tagstack(void *wp, void *retdict);
extern int rs_set_tagstack(void *wp, const void *d, int action);
extern int rs_expand_tags(bool tagnames, char *pat, int *num_file, char ***file);

// Phase 9 Rust implementations
extern const char *rs_did_set_tagfunc(void *args);
extern void rs_free_tagfunc_option(void);
extern bool rs_set_ref_in_tagfunc(int copyID);
extern void rs_set_buflocal_tfu_callback(void *buf);
extern int rs_find_tagfunc_tags(char *pat, void *ga, int *match_count, int flags, char *buf_ffname);

#include "tag.c.generated.h"

static const char e_tag_stack_empty[]
  = N_("E73: Tag stack empty");
static const char e_tag_not_found_str[]
  = N_("E426: Tag not found: %s");
static const char e_at_bottom_of_tag_stack[]
  = N_("E555: At bottom of tag stack");
static const char e_at_top_of_tag_stack[]
  = N_("E556: At top of tag stack");
static const char e_cannot_modify_tag_stack_within_tagfunc[]
  = N_("E986: Cannot modify the tag stack within tagfunc");
static const char e_invalid_return_value_from_tagfunc[]
  = N_("E987: Invalid return value from tagfunc");
static const char e_window_unexpectedly_close_while_searching_for_tags[]
  = N_("E1299: Window unexpectedly closed while searching for tags");

static char *tagmatchname = NULL;   // name of last used tag

// Tag for preview window is remembered separately, to avoid messing up the
// normal tagstack.
static taggy_T ptag_entry = { NULL, INIT_FMARK, 0, 0, NULL };

static bool tfu_in_use = false;  // disallow recursive call of tagfunc
static Callback tfu_cb;          // 'tagfunc' callback function

// Used instead of NUL to separate tag fields in the growarrays.
#define TAG_SEP 0x02

// ============================================================================
// Rust FFI accessor functions for Phase 1 migration
// ============================================================================

/// Free and clear the tagmatchname global
void nvim_xfree_clear_tagmatchname(void)
{
  XFREE_CLEAR(tagmatchname);
}

/// Get the tagmatchname global (name of last used tag)
const char *nvim_get_tagmatchname(void)
{
  return tagmatchname;
}

/// Set the tagmatchname global (takes ownership of the string)
void nvim_set_tagmatchname(char *name)
{
  tagmatchname = name;
}

/// Get a pointer to the ptag_entry global
void *nvim_get_ptag_entry(void)
{
  return &ptag_entry;
}

/// Wrapper for msg_advance (msg_putchar already exists in message.c)
void nvim_tag_msg_advance(int col)
{
  msg_advance(col);
}

/// Wrapper for path_full_compare with kEqualFiles check
int nvim_path_full_compare_equal(const char *s1, const char *s2)
{
  return (path_full_compare((char *)s1, (char *)s2, true, true) & kEqualFiles);
}

/// Check if curwin is NULL
bool nvim_tag_curwin_is_null(void)
{
  return curwin == NULL;
}

/// Call do_tag with DT_FREE to free cached matches
void nvim_do_tag_free(void)
{
  do_tag(NULL, DT_FREE, 0, 0, 0);
}

// ============================================================================
// Rust FFI accessor functions for Phase 3 (expand_tag_fname)
// ============================================================================

/// Check if a path has wildcards
bool nvim_path_has_wildcard(const char *fname)
{
  return path_has_wildcard(fname);
}

/// Expand wildcards in a filename (ExpandInit + ExpandOne)
char *nvim_expand_one_file(char *fname)
{
  expand_T xpc;
  ExpandInit(&xpc);
  xpc.xp_context = EXPAND_FILES;
  return ExpandOne(&xpc, fname, NULL,
                   WILD_LIST_NOTFOUND|WILD_SILENT, WILD_EXPAND_FREE);
}

/// Check if a filename is absolute
bool nvim_vim_isAbsName(const char *fname)
{
  return vim_isAbsName(fname);
}

/// Get the 'tagrelative' option value
bool nvim_get_p_tr(void)
{
  return p_tr;
}

// ============================================================================
// Rust FFI accessor functions for Phase 4 (search state machine)
// ============================================================================

/// Get st->state
int nvim_findtags_get_state_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return (int)st->state;
}

/// Set st->state
void nvim_findtags_set_state_val(void *st_void, int state)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->state = (tagsearch_state_T)state;
}

/// Get st->lbuf
char *nvim_findtags_get_lbuf(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->lbuf;
}

/// Get st->lbuf_size
int nvim_findtags_get_lbuf_size(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->lbuf_size;
}

/// Set st->lbuf and st->lbuf_size (for string_convert swap)
void nvim_findtags_set_lbuf(void *st_void, char *lbuf, int lbuf_size)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->lbuf = lbuf;
  st->lbuf_size = lbuf_size;
}

/// vim_fgets on st->fp into st->lbuf
bool nvim_findtags_fgets(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  return vim_fgets(st->lbuf, st->lbuf_size, st->fp);
}

/// vim_fseek on st->fp
int nvim_findtags_fseek(void *st_void, int64_t offset, int whence)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  return vim_fseek(st->fp, (off_T)offset, whence);
}

/// vim_ftell on st->fp
int64_t nvim_findtags_ftell(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return (int64_t)vim_ftell(st->fp);
}

/// fseek(st->fp, 0, SEEK_SET)
void nvim_findtags_fseek_zero(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  fseek(st->fp, 0, SEEK_SET);
}

/// Check if st->lbuf is a blank line
bool nvim_findtags_lbuf_is_blank(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return vim_isblankline(st->lbuf);
}

/// Get st->flags
int nvim_findtags_get_flags(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->flags;
}

/// Get st->linear
bool nvim_findtags_get_linear_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->linear;
}

/// Set st->linear
void nvim_findtags_set_linear(void *st_void, bool linear)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->linear = linear;
}

/// Get st->tag_file_sorted
int nvim_findtags_get_sorted(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->tag_file_sorted;
}

/// Set st->tag_file_sorted
void nvim_findtags_set_sorted(void *st_void, int val)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->tag_file_sorted = val;
}

/// Get st->orgpat->regmatch.rm_ic
bool nvim_findtags_get_orgpat_rm_ic(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->regmatch.rm_ic;
}

/// Set st->orgpat->regmatch.rm_ic
void nvim_findtags_set_orgpat_rm_ic(void *st_void, bool ic)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->orgpat->regmatch.rm_ic = ic;
}

/// Call convert_setup on st->vimconv
void nvim_findtags_convert_setup(void *st_void, const char *from)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  convert_setup(&st->vimconv, (char *)from, p_enc);
}

/// Call string_convert on st->vimconv with st->lbuf
/// Returns converted string (caller must free), or NULL if no conversion.
char *nvim_findtags_string_convert(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  return string_convert(&st->vimconv, st->lbuf, NULL);
}

// ============================================================================
// Rust FFI accessor functions for Phase 5 (parse line and matching)
// ============================================================================

/// Get st->orgpat->headlen
int nvim_findtags_get_orgpat_headlen(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->headlen;
}

/// Get st->orgpat->head
const char *nvim_findtags_get_orgpat_head(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->head;
}

/// Get st->orgpat->pat
const char *nvim_findtags_get_orgpat_pat(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->pat;
}

/// Get st->orgpat->len
int nvim_findtags_get_orgpat_len(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->len;
}

/// Check if st->orgpat->regmatch.regprog is not NULL
bool nvim_findtags_has_regprog(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->regmatch.regprog != NULL;
}

/// Run vim_regexec on st->orgpat->regmatch with tagname
bool nvim_findtags_vim_regexec(void *st_void, const char *tagname)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  return vim_regexec(&st->orgpat->regmatch, (char *)tagname, 0);
}

/// Get st->orgpat->regmatch.startp[0] offset from tagname
int nvim_findtags_get_regmatch_startoff(const void *st_void, const char *tagname)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return (int)(st->orgpat->regmatch.startp[0] - tagname);
}

/// Call mb_strnicmp for tag comparison
int nvim_mb_strnicmp(const char *s1, const char *s2, size_t len)
{
  return mb_strnicmp(s1, s2, len);
}

/// Get st->tag_fname
const char *nvim_findtags_get_tag_fname(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->tag_fname;
}

/// Get st->help_lang (3-byte buffer)
const char *nvim_findtags_get_help_lang(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->help_lang;
}

/// Get st->help_pri
int nvim_findtags_get_help_pri(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->help_pri;
}

/// Get st->get_searchpat
bool nvim_findtags_get_searchpat(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->get_searchpat;
}

/// Set st->get_searchpat
void nvim_findtags_set_searchpat(void *st_void, bool val)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->get_searchpat = val;
}

/// Get st->match_count
int nvim_findtags_get_match_count_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->match_count;
}

/// Increment st->match_count
void nvim_findtags_inc_match_count(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->match_count++;
}

/// Set st->match_count
void nvim_findtags_set_match_count(void *st_void, int count)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->match_count = count;
}

/// Get the current State for insert mode check
int nvim_get_current_State(void)
{
  return State;
}

/// Get 'showfulltag' option
bool nvim_get_p_sft(void)
{
  return p_sft;
}

/// Call help_heuristic
int nvim_help_heuristic(const char *tagname, int match_offset, bool wrong_case)
{
  return help_heuristic((char *)tagname, match_offset, wrong_case);
}

// Verify hash_T size matches Rust usize assumption
_Static_assert(sizeof(hash_T) == sizeof(size_t), "hash_T must be size_t");

// Verify tag flag constants match Rust values
_Static_assert(TAG_VERBOSE == 32, "TAG_VERBOSE must be 32");
_Static_assert(TAG_INS_COMP == 64, "TAG_INS_COMP must be 64");
_Static_assert(TAG_KEEP_LANG == 128, "TAG_KEEP_LANG must be 128");
_Static_assert(TAG_NO_TAGFUNC == 256, "TAG_NO_TAGFUNC must be 256");
_Static_assert(TAG_MANY == 300, "TAG_MANY must be 300");

/// Add a match to ht_match/ga_match arrays.
/// Returns true if the match was added (not a duplicate).
bool nvim_findtags_add_match_entry(void *st_void, int mtt, char *mfp, hash_T *hash)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  *hash = hash_hash(mfp);
  hashitem_T *hi = hash_lookup(&st->ht_match[mtt], mfp, strlen(mfp), *hash);
  if (HASHITEM_EMPTY(hi)) {
    hash_add_item(&st->ht_match[mtt], hi, mfp, *hash);
    GA_APPEND(char *, &st->ga_match[mtt], mfp);
    st->match_count++;
    return true;
  }
  return false;
}

/// Get ga_match[mtt].ga_len
int nvim_findtags_ga_match_len(const void *st_void, int mtt)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->ga_match[mtt].ga_len;
}

/// Get ga_match[mtt].ga_data[idx]
char *nvim_findtags_ga_match_get(const void *st_void, int mtt, int idx)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return ((char **)(st->ga_match[mtt].ga_data))[idx];
}

/// Clear ga_match[mtt] and ht_match[mtt]
void nvim_findtags_clear_match(void *st_void, int mtt)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  ga_clear(&st->ga_match[mtt]);
  hash_clear(&st->ht_match[mtt]);
}

/// Get st->stop_searching
bool nvim_findtags_get_stop_searching(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->stop_searching;
}

/// Set st->stop_searching
void nvim_findtags_set_stop_searching(void *st_void, bool val)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->stop_searching = val;
}

// ============================================================================
// Rust FFI accessor functions for Phase 6 (search orchestration)
// ============================================================================

/// Get st->is_txt
bool nvim_findtags_get_is_txt(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->is_txt;
}

/// Set st->is_txt
void nvim_findtags_set_is_txt(void *st_void, bool val)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->is_txt = val;
}

/// Get st->help_lang_find (may be NULL)
const char *nvim_findtags_get_help_lang_find(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->help_lang_find;
}

/// Set st->help_lang_find
void nvim_findtags_set_help_lang_find(void *st_void, const char *val)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->help_lang_find = (char *)val;
}

/// Set st->help_pri
void nvim_findtags_set_help_pri(void *st_void, int pri)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->help_pri = pri;
}

/// Set st->help_lang (copies 2 bytes + NUL)
void nvim_findtags_set_help_lang(void *st_void, const char *lang)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->help_lang[0] = lang[0];
  st->help_lang[1] = lang[1];
  st->help_lang[2] = NUL;
}

/// Get st->vimconv.vc_type
int nvim_findtags_get_vimconv_type(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return (int)st->vimconv.vc_type;
}

/// Set st->vimconv.vc_type = CONV_NONE
void nvim_findtags_set_vimconv_none(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->vimconv.vc_type = CONV_NONE;
}

/// convert_setup on st->vimconv with NULL, NULL (cleanup)
void nvim_findtags_convert_cleanup(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  convert_setup(&st->vimconv, NULL, NULL);
}

/// Open st->tag_fname for reading, set st->fp
bool nvim_findtags_fopen(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->fp = os_fopen(st->tag_fname, "r");
  return st->fp != NULL;
}

/// Close st->fp if not NULL
void nvim_findtags_fclose(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  if (st->fp != NULL) {
    fclose(st->fp);
    st->fp = NULL;
  }
}

/// Set st->did_open = true
void nvim_findtags_set_did_open(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->did_open = true;
}

/// Get st->did_open
bool nvim_findtags_get_did_open(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->did_open;
}

/// Set st->state = TS_START
void nvim_findtags_set_state_start(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->state = TS_START;
}

/// Get st->mincount
int nvim_findtags_get_mincount(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->mincount;
}

/// Grow st->lbuf and optionally re-seek to re-read the line.
/// Returns true if a grow+re-seek was needed.
bool nvim_findtags_grow_lbuf(void *st_void, void *sinfo_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  tagsearch_info_T *sinfo = (tagsearch_info_T *)sinfo_void;
  if (st->lbuf[st->lbuf_size - 2] == NUL) {
    return false;  // line fits, no grow needed
  }
  st->lbuf_size *= 2;
  xfree(st->lbuf);
  st->lbuf = xmalloc((size_t)st->lbuf_size);
  if (st->state == TS_STEP_FORWARD || st->state == TS_LINEAR) {
    vim_ignored = vim_fseek(st->fp, sinfo->curr_offset, SEEK_SET);
  }
  sinfo->curr_offset = 0;
  return true;
}

/// Get help_only from findtags_state_T (same as nvim_findtags_get_help_only but named for phase 6)
bool nvim_findtags_get_help_only_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->help_only;
}

/// Get st->orgpat->len (phase 6 variant)
int nvim_findtags_get_orgpat_len_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->len;
}

/// Set st->orgpat->len
void nvim_findtags_set_orgpat_len(void *st_void, int len)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->orgpat->len = len;
}

/// Set st->orgpat->pat
void nvim_findtags_set_orgpat_pat(void *st_void, char *pat)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->orgpat->pat = pat;
}

/// Get st->orgpat->headlen (phase 6 variant)
int nvim_findtags_get_orgpat_headlen_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->headlen;
}

/// Get st->orgpat->regmatch.regprog != NULL (phase 6 variant)
bool nvim_findtags_has_regprog_val(const void *st_void)
{
  const findtags_state_T *st = (const findtags_state_T *)st_void;
  return st->orgpat->regmatch.regprog != NULL;
}

/// Set st->linear (phase 6 variant)
void nvim_findtags_set_linear_val(void *st_void, bool val)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  st->linear = val;
}

// Global variable accessors for Phase 6

/// Set p_ic (ignorecase)
void nvim_set_p_ic(int val)
{
  p_ic = val;
}

/// Get p_tbs (tagbsearch)
bool nvim_get_p_tbs(void)
{
  return p_tbs;
}

/// Get tc_flags (global tagcase flags)
int nvim_get_tc_flags(void)
{
  return (int)tc_flags;
}

/// Get curbuf->b_tc_flags
int nvim_get_curbuf_tc_flags(void)
{
  return (int)curbuf->b_tc_flags;
}

/// Get p_hlg (helplang option)
const char *nvim_get_p_hlg(void)
{
  return p_hlg;
}

/// Get curbuf->b_fname
const char *nvim_get_curbuf_b_fname(void)
{
  return curbuf->b_fname;
}

/// Get curbuf->b_p_tfu
const char *nvim_get_curbuf_p_tfu(void)
{
  return curbuf->b_p_tfu;
}

/// Set curbuf->b_help
void nvim_set_curbuf_b_help(int val)
{
  curbuf->b_help = val;
}

/// Get curbuf->b_help
int nvim_get_curbuf_b_help(void)
{
  return curbuf->b_help;
}

// Function wrappers for Phase 6

/// ins_compl_check_keys wrapper
void nvim_ins_compl_check_keys(int interval, bool pum_wanted)
{
  ins_compl_check_keys(interval, pum_wanted);
}

/// ins_compl_interrupted wrapper
bool nvim_ins_compl_interrupted(void)
{
  return ins_compl_interrupted();
}

/// verbose_enter/leave + smsg for "Searching tags file %s"
void nvim_verbose_searching_tags(const char *tag_fname)
{
  verbose_enter();
  smsg(0, _("Searching tags file %s"), tag_fname);
  verbose_leave();
}

/// ignorecase() wrapper
bool nvim_ignorecase(const char *pat)
{
  return ignorecase((char *)pat);
}

/// ignorecase_opt() wrapper
bool nvim_ignorecase_opt(const char *pat, bool ic_strstrp, bool ic_strstrp2)
{
  return ignorecase_opt((char *)pat, ic_strstrp, ic_strstrp2);
}

/// semsg E431 wrapper
void nvim_semsg_e431(const char *tag_fname)
{
  semsg(_("E431: Format error in tags file \"%s\""), tag_fname);
}

/// semsg "Before byte" wrapper
void nvim_semsg_before_byte(int64_t offset)
{
  semsg(_("Before byte %" PRId64), offset);
}

/// semsg E432 wrapper
void nvim_semsg_e432(const char *tag_fname)
{
  semsg(_("E432: Tags file not sorted: %s"), tag_fname);
}

/// emsg E433 wrapper
void nvim_emsg_e433(void)
{
  emsg(_("E433: No tags file"));
}

/// Call find_tagfunc_tags via st->ga_match (keeps tfu_in_use in C)
int nvim_findtags_apply_tfu(void *st_void, char *pat, char *buf_ffname)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  const bool use_tfu = ((st->flags & TAG_NO_TAGFUNC) == 0);

  if (!use_tfu || tfu_in_use || *curbuf->b_p_tfu == NUL) {
    return NOTDONE;
  }

  tfu_in_use = true;
  int retval = find_tagfunc_tags(pat, st->ga_match, &st->match_count,
                                 st->flags, buf_ffname);
  tfu_in_use = false;
  return retval;
}

/// prepare_pats on st->orgpat
void nvim_findtags_prepare_pats(void *st_void, bool has_re)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  prepare_pats(st->orgpat, has_re);
}

// ============================================================================
// Rust FFI accessor functions for Phase 7 (tag display and location list)
// ============================================================================

// Verify highlight constants used in Rust
_Static_assert(HLF_T == 23, "HLF_T value for Rust");
_Static_assert(HLF_D == 5, "HLF_D value for Rust");
_Static_assert(HLF_CM == 11, "HLF_CM value for Rust");
_Static_assert(MT_MASK == 7, "MT_MASK value for Rust");
_Static_assert(IOSIZE == 1025, "IOSIZE value for Rust");

/// Get mt_names entry by index
const char *nvim_tag_get_mt_name(int idx)
{
  if (idx < 0 || idx >= MT_COUNT / 2) {
    return "   ";
  }
  return mt_names[idx];
}

/// Wrapper for msg_puts (no highlight)
void nvim_tag_msg_puts(const char *s)
{
  msg_puts(s);
}

/// Wrapper for msg_puts_title
void nvim_tag_msg_puts_title(const char *s)
{
  msg_puts_title(s);
}

/// Wrapper for msg_outtrans
void nvim_tag_msg_outtrans(const char *str, int attr, bool right)
{
  msg_outtrans(str, attr, right);
}

/// Wrapper for msg_outtrans_len
void nvim_tag_msg_outtrans_len(const char *str, int len, int attr, bool right)
{
  msg_outtrans_len(str, len, attr, right);
}

/// Wrapper for msg_outtrans_one - returns pointer to next char
const char *nvim_tag_msg_outtrans_one(const char *p, int hl_id, bool right)
{
  return msg_outtrans_one((char *)p, hl_id, right);
}

/// Wrapper for msg_putchar
void nvim_tag_msg_putchar(int c)
{
  msg_putchar(c);
}

/// Wrapper for msg_start
void nvim_tag_msg_start(void)
{
  msg_start();
}

/// Wrapper for os_breakcheck
void nvim_tag_os_breakcheck(void)
{
  os_breakcheck();
}

/// Wrapper for verbose_enter
void nvim_tag_verbose_enter(void)
{
  verbose_enter();
}

/// Wrapper for verbose_leave
void nvim_tag_verbose_leave(void)
{
  verbose_leave();
}

/// Wrapper for smsg with one string argument (for duplicate field name)
void nvim_tag_smsg_dup_field(const char *field_name)
{
  smsg(0, _("Duplicate field name: %s"), field_name);
}

/// Get the curwin pointer for Rust
void *nvim_tag_get_curwin(void)
{
  return (void *)curwin;
}

/// Wrapper for tv_dict_alloc
void *nvim_tag_tv_dict_alloc(void)
{
  return (void *)tv_dict_alloc();
}

/// Wrapper for tv_dict_find - returns non-null if found
bool nvim_tag_tv_dict_find(void *dict, const char *key, int key_len)
{
  return tv_dict_find((dict_T *)dict, key, key_len) != NULL;
}

/// Wrapper for tv_dict_add_str
int nvim_tag_tv_dict_add_str(void *dict, const char *key, size_t key_len, char *val)
{
  return tv_dict_add_str((dict_T *)dict, key, key_len, val);
}

/// Wrapper for tv_dict_add_nr
int nvim_tag_tv_dict_add_nr(void *dict, const char *key, size_t key_len, int64_t nr)
{
  return tv_dict_add_nr((dict_T *)dict, key, key_len, (varnumber_T)nr);
}

/// Wrapper for tv_list_alloc
void *nvim_tag_tv_list_alloc(int count)
{
  return (void *)tv_list_alloc(count);
}

/// Wrapper for tv_list_append_dict
void nvim_tag_tv_list_append_dict(void *list, void *dict)
{
  tv_list_append_dict((list_T *)list, (dict_T *)dict);
}

/// Wrapper for tv_list_free
void nvim_tag_tv_list_free(void *list)
{
  tv_list_free((list_T *)list);
}

/// Wrapper for set_errorlist
void nvim_tag_set_errorlist(void *list, const char *title)
{
  set_errorlist(curwin, (list_T *)list, ' ', (char *)title, NULL);
}

/// Wrapper for vim_snprintf into IObuff
void nvim_tag_snprintf_iobuff(const char *fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  vim_vsnprintf(IObuff, IOSIZE, fmt, ap);
  va_end(ap);
}

/// Format do_tags line into IObuff and output it
void nvim_tag_do_tags_line(int is_current, int idx, int cur_match,
                           const char *tagname, int64_t lnum)
{
  vim_snprintf(IObuff, IOSIZE, "%c%2d %2d %-15s %5" PRIdLINENR "  ",
               is_current ? '>' : ' ',
               idx + 1,
               cur_match + 1,
               tagname,
               (linenr_T)lnum);
  msg_outtrans(IObuff, 0, false);
}

/// Wrapper for fm_getname from taggy_T
char *nvim_tag_fm_getname(const void *tg_void, int lead_len)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return fm_getname(&((taggy_T *)tg)->fmark, lead_len);
}

/// Get fmark.fnum from taggy_T (for do_tags comparison)
int nvim_tag_taggy_fmark_fnum(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->fmark.fnum;
}

/// Format print_tag_list header line into IObuff
void nvim_tag_list_format_entry(bool is_current, int i, const char *mt_name)
{
  *IObuff = is_current ? '>' : ' ';
  vim_snprintf(IObuff + 1, IOSIZE - 1, "%2d %s ", i + 1, mt_name);
  msg_puts(IObuff);
}

/// Get IOSIZE constant
int nvim_tag_get_iosize(void)
{
  return IOSIZE;
}

/// Get MAXPATHL constant
int nvim_tag_get_maxpathl(void)
{
  return MAXPATHL;
}

/// Wrapper for xstrlcpy
void nvim_tag_xstrlcpy(char *dst, const char *src, size_t dstsize)
{
  xstrlcpy(dst, src, dstsize);
}

/// Wrapper for xmemcpyz
void nvim_tag_xmemcpyz(char *dst, const char *src, size_t len)
{
  xmemcpyz(dst, src, len);
}

/// Get translate string via _() macro
const char *nvim_tag_gettext(const char *s)
{
  return _(s);
}

/// Get g_do_tagpreview value
int nvim_tag_get_g_do_tagpreview(void)
{
  return g_do_tagpreview;
}

/// Get ptag_entry.cur_match
int nvim_tag_get_ptag_cur_match(void)
{
  return ptag_entry.cur_match;
}

// ============================================================================
// Rust FFI accessor functions for Phase 8 (VimL API and tag stack setters)
// ============================================================================

// Verify constants used in Rust
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL value for Rust");

/// Wrapper for find_tags callable from Rust
int nvim_tag_find_tags(char *pat, int *num_matches, char ***matchesp,
                       int flags, int mincount, char *buf_ffname)
{
  return find_tags(pat, num_matches, matchesp, flags, mincount, buf_ffname);
}

/// Wrapper for FreeWild
void nvim_tag_free_wild(int count, char **files)
{
  FreeWild(count, files);
}

/// Get curbuf->b_ffname
char *nvim_tag_get_curbuf_ffname(void)
{
  return curbuf->b_ffname;
}

/// Wrapper for xrealloc
void *nvim_tag_xrealloc(void *ptr, size_t size)
{
  return xrealloc(ptr, size);
}

/// Wrapper for memmove
void nvim_tag_memmove(void *dest, const void *src, size_t n)
{
  memmove(dest, src, n);
}

/// MB_PTR_ADV wrapper - advance pointer past one multi-byte char
const char *nvim_tag_mb_ptr_adv(const char *p)
{
  const char *result = p;
  MB_PTR_ADV(result);
  return result;
}

/// ascii_iswhite wrapper
bool nvim_tag_ascii_iswhite(int c)
{
  return ascii_iswhite(c);
}

/// Get tfu_in_use flag
bool nvim_tag_get_tfu_in_use(void)
{
  return tfu_in_use;
}

/// Get e_cannot_modify_tag_stack_within_tagfunc message
void nvim_tag_emsg_tfu_in_use(void)
{
  emsg(_(e_cannot_modify_tag_stack_within_tagfunc));
}

/// Get e_listreq message
void nvim_tag_emsg_listreq(void)
{
  emsg(_(e_listreq));
}

/// tv_dict_find returning opaque dictitem handle (NULL if not found)
void *nvim_tag_tv_dict_find_item(const void *dict, const char *key, int key_len)
{
  return (void *)tv_dict_find((const dict_T *)dict, key, key_len);
}

/// Get the typval from a dictitem
void *nvim_tag_dictitem_tv(void *di)
{
  return (void *)&((dictitem_T *)di)->di_tv;
}

/// Check if typval is a list type
bool nvim_tag_tv_is_list(const void *tv)
{
  return ((const typval_T *)tv)->v_type == VAR_LIST;
}

/// Get list from typval
void *nvim_tag_tv_get_list(const void *tv)
{
  return (void *)((const typval_T *)tv)->vval.v_list;
}

/// Get number from typval
int64_t nvim_tag_tv_get_number(const void *tv)
{
  return (int64_t)tv_get_number((const typval_T *)tv);
}

/// Wrapper for tv_dict_get_string
char *nvim_tag_tv_dict_get_string(const void *dict, const char *key, bool save)
{
  return tv_dict_get_string((const dict_T *)dict, key, save);
}

/// Wrapper for tv_dict_get_number
int64_t nvim_tag_tv_dict_get_number(const void *dict, const char *key)
{
  return (int64_t)tv_dict_get_number((const dict_T *)dict, key);
}

/// Wrapper for tv_list_first
void *nvim_tag_tv_list_first(const void *list)
{
  return (void *)tv_list_first((const list_T *)list);
}

/// Wrapper for TV_LIST_ITEM_NEXT
void *nvim_tag_tv_list_item_next(const void *list, const void *li)
{
  return (void *)TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li);
}

/// Get dict from list item (NULL if not a dict)
void *nvim_tag_tv_list_item_dict(const void *li)
{
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  if (tv->v_type != VAR_DICT || tv->vval.v_dict == NULL) {
    return NULL;
  }
  return (void *)tv->vval.v_dict;
}

/// Wrapper for list2fpos - fills pos and fnum from a list typval
/// Returns OK or FAIL
int nvim_tag_list2fpos(void *tv, int32_t *lnum, int32_t *col, int32_t *coladd, int *fnum)
{
  pos_T pos;
  int result = list2fpos((typval_T *)tv, &pos, fnum, NULL, false);
  if (result == OK) {
    *lnum = pos.lnum;
    *col = pos.col;
    *coladd = pos.coladd;
  }
  return result;
}

/// Wrapper for tv_list_append_number
void nvim_tag_tv_list_append_number(void *list, int64_t nr)
{
  tv_list_append_number((list_T *)list, (varnumber_T)nr);
}

/// Wrapper for tv_dict_add_list
void nvim_tag_tv_dict_add_list(void *dict, const char *key, size_t key_len, void *list)
{
  tv_dict_add_list((dict_T *)dict, key, key_len, (list_T *)list);
}

/// Get taggy_T user_data field
const char *nvim_tag_taggy_get_user_data_val(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->user_data;
}

/// Get fmark mark.col from taggy_T
int nvim_tag_taggy_fmark_col(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->fmark.mark.col;
}

/// Get fmark mark.coladd from taggy_T
int nvim_tag_taggy_fmark_coladd(const void *tg_void)
{
  const taggy_T *tg = (const taggy_T *)tg_void;
  return tg->fmark.mark.coladd;
}

/// Set w_tagstackidx directly
void nvim_tag_win_set_tagstackidx(void *wp_void, int idx)
{
  win_T *wp = (win_T *)wp_void;
  wp->w_tagstackidx = idx;
}

/// Get w_tagstacklen directly
int nvim_tag_win_get_tagstacklen(const void *wp_void)
{
  const win_T *wp = (const win_T *)wp_void;
  return wp->w_tagstacklen;
}

// ============================================================================
// Phase 9 C accessor functions for tagfunc and option management
// ============================================================================

/// Free the global tfu_cb callback
void nvim_tag_callback_free_tfu(void)
{
  callback_free(&tfu_cb);
}

/// Free the buffer-local tfu callback
void nvim_tag_callback_free_buf_tfu(void *buf_void)
{
  buf_T *buf = (buf_T *)buf_void;
  callback_free(&buf->b_tfu_cb);
}

/// Check if buffer's b_p_tfu is empty (NUL)
bool nvim_tag_buf_tfu_is_empty(const void *buf_void)
{
  const buf_T *buf = (const buf_T *)buf_void;
  return *buf->b_p_tfu == NUL;
}

/// Call option_set_callback_func with buf's b_p_tfu into tfu_cb
/// Returns FAIL or OK
int nvim_tag_option_set_tfu_callback(void *buf_void)
{
  buf_T *buf = (buf_T *)buf_void;
  return option_set_callback_func(buf->b_p_tfu, &tfu_cb);
}

/// Copy global tfu_cb to buffer-local b_tfu_cb
void nvim_tag_callback_copy_tfu_to_buf(void *buf_void)
{
  buf_T *buf = (buf_T *)buf_void;
  callback_copy(&buf->b_tfu_cb, &tfu_cb);
}

/// Check if global tfu_cb is kCallbackNone
bool nvim_tag_tfu_cb_is_none(void)
{
  return tfu_cb.type == kCallbackNone;
}

/// set_ref_in_callback for global tfu_cb
bool nvim_tag_set_ref_in_tfu_callback(int copyID)
{
  return set_ref_in_callback(&tfu_cb, copyID, NULL, NULL);
}

/// Get the buf_T * from optset_T args (os_buf field)
void *nvim_tag_optset_get_buf(const void *args_void)
{
  const optset_T *args = (const optset_T *)args_void;
  return (void *)args->os_buf;
}

/// Get e_invarg pointer (for did_set_tagfunc return)
const char *nvim_tag_get_e_invarg(void)
{
  return e_invarg;
}

/// Call the tagfunc callback and validate the result.
/// Returns:
///   0 (OK) with *out_list set to the returned list (caller must call nvim_tag_tv_clear_rettv)
///   1 (FAIL) if callback call failed
///   2 (NOTDONE) if result was v:null
///   3 if result was not a list (emsg already shown)
///   4 if curbuf tfu is empty or callback is none
int nvim_tag_call_tagfunc(const char *pat, int flags, const char *buf_ffname,
                          void **out_list, void *rettv_storage)
{
  typval_T *rettv = (typval_T *)rettv_storage;
  typval_T args[4];
  char flagString[4];

  // Check prerequisites
  if (*curbuf->b_p_tfu == NUL || curbuf->b_tfu_cb.type == kCallbackNone) {
    return 4;
  }

  args[0].v_type = VAR_STRING;
  args[0].vval.v_string = (char *)pat;
  args[1].v_type = VAR_STRING;
  args[1].vval.v_string = flagString;

  // create 'info' dict argument
  dict_T *const d = tv_dict_alloc_lock(VAR_FIXED);

  // Get tag entry for user_data
  taggy_T *tag = NULL;
  if (curwin->w_tagstacklen > 0) {
    if (curwin->w_tagstackidx == curwin->w_tagstacklen) {
      tag = &curwin->w_tagstack[curwin->w_tagstackidx - 1];
    } else {
      tag = &curwin->w_tagstack[curwin->w_tagstackidx];
    }
  }
  if (tag != NULL && tag->user_data != NULL) {
    tv_dict_add_str(d, S_LEN("user_data"), tag->user_data);
  }
  if (buf_ffname != NULL) {
    tv_dict_add_str(d, S_LEN("buf_ffname"), (char *)buf_ffname);
  }

  d->dv_refcount++;
  args[2].v_type = VAR_DICT;
  args[2].vval.v_dict = d;

  args[3].v_type = VAR_UNKNOWN;

  vim_snprintf(flagString, sizeof(flagString),
               "%s%s%s",
               g_tag_at_cursor ? "c" : "",
               flags & TAG_INS_COMP ? "i" : "",
               flags & TAG_REGEXP ? "r" : "");

  pos_T save_pos = curwin->w_cursor;
  int result = callback_call(&curbuf->b_tfu_cb, 3, args, rettv);
  curwin->w_cursor = save_pos;
  check_cursor(curwin);
  d->dv_refcount--;

  if (result == FAIL) {
    return 1;
  }
  if (rettv->v_type == VAR_SPECIAL && rettv->vval.v_special == kSpecialVarNull) {
    tv_clear(rettv);
    return 2;
  }
  if (rettv->v_type != VAR_LIST || !rettv->vval.v_list) {
    tv_clear(rettv);
    emsg(_(e_invalid_return_value_from_tagfunc));
    return 3;
  }

  *out_list = (void *)rettv->vval.v_list;
  return 0;
}

/// Clear the rettv storage after tagfunc call is done
void nvim_tag_tv_clear_rettv(void *rettv_storage)
{
  tv_clear((typval_T *)rettv_storage);
}

/// Get the size needed for rettv storage (sizeof(typval_T))
size_t nvim_tag_rettv_size(void)
{
  return sizeof(typval_T);
}

/// Check if a list item is a dict type
bool nvim_tag_listitem_is_dict(const void *li)
{
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  return tv->v_type == VAR_DICT;
}

/// Get the dict from a list item (returns dict handle, or NULL)
void *nvim_tag_listitem_get_dict(const void *li)
{
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  if (tv->v_type != VAR_DICT || !tv->vval.v_dict) {
    return NULL;
  }
  return (void *)tv->vval.v_dict;
}

/// Count string-valued entries in a dict, and compute total length
/// needed for match string building. Returns count of string entries.
int nvim_tag_dict_string_entry_count(const void *dict_void)
{
  const dict_T *dict = (const dict_T *)dict_void;
  int count = 0;
  TV_DICT_ITER(dict, di, {
    if (di->di_tv.v_type == VAR_STRING && di->di_tv.vval.v_string != NULL) {
      count++;
    }
  });
  return count;
}

/// Compute total length needed for tagfunc match string from dict entries.
/// Counts: sum of strlen(key) + 1 + strlen(value) + 1 for each string entry.
size_t nvim_tag_dict_compute_match_len(const void *dict_void)
{
  const dict_T *dict = (const dict_T *)dict_void;
  size_t len = 2;  // base overhead
  TV_DICT_ITER(dict, di, {
    if (di->di_tv.v_type == VAR_STRING && di->di_tv.vval.v_string != NULL) {
      len += strlen(di->di_tv.vval.v_string) + 1;  // "\tVALUE"
      // Non-standard keys also need key + ":"
      if (strcmp(di->di_key, "name") != 0
          && strcmp(di->di_key, "filename") != 0
          && strcmp(di->di_key, "cmd") != 0
          && strcmp(di->di_key, "kind") != 0) {
        len += strlen(di->di_key) + 1;  // "KEY:"
      }
    }
  });
  return len;
}

/// Get specific fields from a tagfunc result dict.
/// Fills in name, filename, cmd, kind pointers (NULL if missing).
/// Also sets has_extra to true if there are extra fields beyond the standard 4.
void nvim_tag_dict_get_tag_fields(const void *dict_void,
                                  const char **res_name,
                                  const char **res_fname,
                                  const char **res_cmd,
                                  const char **res_kind,
                                  bool *has_extra)
{
  const dict_T *dict = (const dict_T *)dict_void;
  *res_name = NULL;
  *res_fname = NULL;
  *res_cmd = NULL;
  *res_kind = NULL;
  *has_extra = false;

  TV_DICT_ITER(dict, di, {
    if (di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL) {
      continue;
    }
    if (!strcmp(di->di_key, "name")) {
      *res_name = di->di_tv.vval.v_string;
    } else if (!strcmp(di->di_key, "filename")) {
      *res_fname = di->di_tv.vval.v_string;
    } else if (!strcmp(di->di_key, "cmd")) {
      *res_cmd = di->di_tv.vval.v_string;
    } else if (!strcmp(di->di_key, "kind")) {
      *res_kind = di->di_tv.vval.v_string;
      *has_extra = true;
    } else {
      *has_extra = true;
    }
  });
}

/// Build the extra fields portion of a tagfunc match string.
/// Writes extra tab-separated "KEY:VALUE" entries for non-standard fields.
/// Returns number of bytes written.
size_t nvim_tag_dict_write_extra_fields(const void *dict_void, char *p)
{
  const dict_T *dict = (const dict_T *)dict_void;
  char *start = p;

  TV_DICT_ITER(dict, di, {
    if (di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL) {
      continue;
    }
    const char *key = di->di_key;
    if (!strcmp(key, "name") || !strcmp(key, "filename")
        || !strcmp(key, "cmd") || !strcmp(key, "kind")) {
      continue;
    }
    *p++ = TAB;
    STRCPY(p, key);
    p += strlen(p);
    STRCPY(p, ":");
    p += 1;
    STRCPY(p, di->di_tv.vval.v_string);
    p += strlen(p);
  });

  return (size_t)(p - start);
}

/// emsg for e_invalid_return_value_from_tagfunc
void nvim_tag_emsg_invalid_tagfunc_return(void)
{
  emsg(_(e_invalid_return_value_from_tagfunc));
}

/// Grow a garray_T by 1 and append a string pointer
void nvim_tag_ga_grow_append(void *ga_void, char *mfp)
{
  garray_T *ga = (garray_T *)ga_void;
  ga_grow(ga, 1);
  ((char **)(ga->ga_data))[ga->ga_len++] = mfp;
}

_Static_assert(TAG_INS_COMP == 64, "TAG_INS_COMP value for Rust");
_Static_assert(TAG_REGEXP == 4, "TAG_REGEXP value for Rust");
_Static_assert(TAG_NAMES == 2, "TAG_NAMES value for Rust");

// ============================================================================
// End of Phase 9 C accessor functions
// ============================================================================

/// Reads the 'tagfunc' option value and convert that to a callback value.
/// Invoked when the 'tagfunc' option is set. The option value can be a name of
/// a function (string), or function(<name>) or funcref(<name>) or a lambda.
const char *did_set_tagfunc(optset_T *args)
{
  return rs_did_set_tagfunc(args);
}

#if defined(EXITFREE)
void free_tagfunc_option(void)
{
  rs_free_tagfunc_option();
}
#endif

/// Mark the global 'tagfunc' callback with "copyID" so that it is not garbage
/// collected.
bool set_ref_in_tagfunc(int copyID)
{
  return rs_set_ref_in_tagfunc(copyID);
}

/// Copy the global 'tagfunc' callback function to the buffer-local 'tagfunc'
/// callback for 'buf'.
void set_buflocal_tfu_callback(buf_T *buf)
{
  rs_set_buflocal_tfu_callback(buf);
}

/// Jump to tag; handling of tag commands and tag stack
///
/// *tag != NUL: ":tag {tag}", jump to new tag, add to tag stack
///
/// type == DT_TAG:      ":tag [tag]", jump to newer position or same tag again
/// type == DT_HELP:     like DT_TAG, but don't use regexp.
/// type == DT_POP:      ":pop" or CTRL-T, jump to old position
/// type == DT_NEXT:     jump to next match of same tag
/// type == DT_PREV:     jump to previous match of same tag
/// type == DT_FIRST:    jump to first match of same tag
/// type == DT_LAST:     jump to last match of same tag
/// type == DT_SELECT:   ":tselect [tag]", select tag from a list of all matches
/// type == DT_JUMP:     ":tjump [tag]", jump to tag or select tag from a list
/// type == DT_LTAG:     use location list for displaying tag matches
/// type == DT_FREE:     free cached matches
///
/// @param tag  tag (pattern) to jump to
/// @param forceit  :ta with !
/// @param verbose  print "tag not found" message
void do_tag(char *tag, int type, int count, int forceit, bool verbose)
{
  taggy_T *tagstack = curwin->w_tagstack;
  int tagstackidx = curwin->w_tagstackidx;
  int tagstacklen = curwin->w_tagstacklen;
  int cur_match = 0;
  int cur_fnum = curbuf->b_fnum;
  int oldtagstackidx = tagstackidx;
  int prevtagstackidx = tagstackidx;
  bool new_tag = false;
  bool no_regexp = false;
  int error_cur_match = 0;
  bool save_pos = false;
  fmark_T saved_fmark;
  int new_num_matches;
  char **new_matches;
  bool use_tagstack;
  bool skip_msg = false;
  char *buf_ffname = curbuf->b_ffname;  // name for priority computation
  bool use_tfu = true;
  char *tofree = NULL;

  // remember the matches for the last used tag
  static int num_matches = 0;
  static int max_num_matches = 0;             // limit used for match search
  static char **matches = NULL;
  static int flags;

#ifdef EXITFREE
  if (type == DT_FREE) {
    // remove the list of matches
    FreeWild(num_matches, matches);
    num_matches = 0;
    return;
  }
#endif

  if (tfu_in_use) {
    emsg(_(e_cannot_modify_tag_stack_within_tagfunc));
    return;
  }

  if (postponed_split == 0 && !check_can_set_curbuf_forceit(forceit)) {
    return;
  }

  if (type == DT_HELP) {
    type = DT_TAG;
    no_regexp = true;
    use_tfu = false;
  }

  int prev_num_matches = num_matches;
  free_string_option(nofile_fname);
  nofile_fname = NULL;

  clearpos(&saved_fmark.mark);          // shutup gcc 4.0
  saved_fmark.fnum = 0;

  // Don't add a tag to the tagstack if 'tagstack' has been reset.
  assert(tag != NULL);
  if (!p_tgst && *tag != NUL) {
    use_tagstack = false;
    new_tag = true;
    if (g_do_tagpreview != 0) {
      tagstack_clear_entry(&ptag_entry);
      ptag_entry.tagname = xstrdup(tag);
    }
  } else {
    if (g_do_tagpreview != 0) {
      use_tagstack = false;
    } else {
      use_tagstack = true;
    }

    // new pattern, add to the tag stack
    if (*tag != NUL
        && (type == DT_TAG || type == DT_SELECT || type == DT_JUMP
            || type == DT_LTAG)) {
      if (g_do_tagpreview != 0) {
        if (ptag_entry.tagname != NULL
            && strcmp(ptag_entry.tagname, tag) == 0) {
          // Jumping to same tag: keep the current match, so that
          // the CursorHold autocommand example works.
          cur_match = ptag_entry.cur_match;
          cur_fnum = ptag_entry.cur_fnum;
        } else {
          tagstack_clear_entry(&ptag_entry);
          ptag_entry.tagname = xstrdup(tag);
        }
      } else {
        // If the last used entry is not at the top, delete all tag
        // stack entries above it.
        while (tagstackidx < tagstacklen) {
          tagstack_clear_entry(&tagstack[--tagstacklen]);
        }

        // if the tagstack is full: remove oldest entry
        if (++tagstacklen > TAGSTACKSIZE) {
          tagstacklen = TAGSTACKSIZE;
          tagstack_clear_entry(&tagstack[0]);
          for (int i = 1; i < tagstacklen; i++) {
            tagstack[i - 1] = tagstack[i];
          }
          tagstack[--tagstackidx].user_data = NULL;
        }

        // put the tag name in the tag stack
        tagstack[tagstackidx].tagname = xstrdup(tag);

        curwin->w_tagstacklen = tagstacklen;

        save_pos = true;                // save the cursor position below
      }

      new_tag = true;
    } else {
      if (g_do_tagpreview != 0 ? ptag_entry.tagname == NULL
                               : tagstacklen == 0) {
        // empty stack
        emsg(_(e_tag_stack_empty));
        goto end_do_tag;
      }

      if (type == DT_POP) {             // go to older position
        const bool old_KeyTyped = KeyTyped;
        if ((tagstackidx -= count) < 0) {
          emsg(_(e_at_bottom_of_tag_stack));
          if (tagstackidx + count == 0) {
            // We did [num]^T from the bottom of the stack
            tagstackidx = 0;
            goto end_do_tag;
          }
          // We weren't at the bottom of the stack, so jump all the
          // way to the bottom now.
          tagstackidx = 0;
        } else if (tagstackidx >= tagstacklen) {        // count == 0?
          emsg(_(e_at_top_of_tag_stack));
          goto end_do_tag;
        }

        // Make a copy of the fmark, autocommands may invalidate the
        // tagstack before it's used.
        saved_fmark = tagstack[tagstackidx].fmark;
        if (saved_fmark.fnum != curbuf->b_fnum) {
          // Jump to other file. If this fails (e.g. because the
          // file was changed) keep original position in tag stack.
          if (buflist_getfile(saved_fmark.fnum, saved_fmark.mark.lnum,
                              GETF_SETMARK, forceit) == FAIL) {
            tagstackidx = oldtagstackidx;              // back to old posn
            goto end_do_tag;
          }
          // A BufReadPost autocommand may jump to the '" mark, but
          // we don't what that here.
          curwin->w_cursor.lnum = saved_fmark.mark.lnum;
        } else {
          setpcmark();
          curwin->w_cursor.lnum = saved_fmark.mark.lnum;
        }
        curwin->w_cursor.col = saved_fmark.mark.col;
        curwin->w_set_curswant = true;
        check_cursor(curwin);
        if ((fdo_flags & kOptFdoFlagTag) && old_KeyTyped) {
          foldOpenCursor();
        }

        // remove the old list of matches
        FreeWild(num_matches, matches);
        num_matches = 0;
        tag_freematch();
        goto end_do_tag;
      }

      if (type == DT_TAG
          || type == DT_LTAG) {
        if (g_do_tagpreview != 0) {
          cur_match = ptag_entry.cur_match;
          cur_fnum = ptag_entry.cur_fnum;
        } else {
          // ":tag" (no argument): go to newer pattern
          save_pos = true;              // save the cursor position below
          if ((tagstackidx += count - 1) >= tagstacklen) {
            // Beyond the last one, just give an error message and
            // go to the last one.  Don't store the cursor
            // position.
            tagstackidx = tagstacklen - 1;
            emsg(_(e_at_top_of_tag_stack));
            save_pos = false;
          } else if (tagstackidx < 0) {         // must have been count == 0
            emsg(_(e_at_bottom_of_tag_stack));
            tagstackidx = 0;
            goto end_do_tag;
          }
          cur_match = tagstack[tagstackidx].cur_match;
          cur_fnum = tagstack[tagstackidx].cur_fnum;
        }
        new_tag = true;
      } else {                                // go to other matching tag
        // Save index for when selection is cancelled.
        prevtagstackidx = tagstackidx;

        if (g_do_tagpreview != 0) {
          cur_match = ptag_entry.cur_match;
          cur_fnum = ptag_entry.cur_fnum;
        } else {
          if (--tagstackidx < 0) {
            tagstackidx = 0;
          }
          cur_match = tagstack[tagstackidx].cur_match;
          cur_fnum = tagstack[tagstackidx].cur_fnum;
        }
        switch (type) {
        case DT_FIRST:
          cur_match = count - 1; break;
        case DT_SELECT:
        case DT_JUMP:
        case DT_LAST:
          cur_match = MAXCOL - 1; break;
        case DT_NEXT:
          cur_match += count; break;
        case DT_PREV:
          cur_match -= count; break;
        }
        if (cur_match >= MAXCOL) {
          cur_match = MAXCOL - 1;
        } else if (cur_match < 0) {
          emsg(_("E425: Cannot go before first matching tag"));
          skip_msg = true;
          cur_match = 0;
          cur_fnum = curbuf->b_fnum;
        }
      }
    }

    if (g_do_tagpreview != 0) {
      if (type != DT_SELECT && type != DT_JUMP) {
        ptag_entry.cur_match = cur_match;
        ptag_entry.cur_fnum = cur_fnum;
      }
    } else {
      // For ":tag [arg]" or ":tselect" remember position before the jump.
      saved_fmark = tagstack[tagstackidx].fmark;
      if (save_pos) {
        tagstack[tagstackidx].fmark.mark = curwin->w_cursor;
        tagstack[tagstackidx].fmark.fnum = curbuf->b_fnum;
      }

      // Curwin will change in the call to jumpto_tag() if ":stag" was
      // used or an autocommand jumps to another window; store value of
      // tagstackidx now.
      curwin->w_tagstackidx = tagstackidx;
      if (type != DT_SELECT && type != DT_JUMP) {
        curwin->w_tagstack[tagstackidx].cur_match = cur_match;
        curwin->w_tagstack[tagstackidx].cur_fnum = cur_fnum;
      }
    }
  }

  // When not using the current buffer get the name of buffer "cur_fnum".
  // Makes sure that the tag order doesn't change when using a remembered
  // position for "cur_match".
  if (cur_fnum != curbuf->b_fnum) {
    buf_T *buf = buflist_findnr(cur_fnum);

    if (buf != NULL) {
      buf_ffname = buf->b_ffname;
    }
  }

  // Repeat searching for tags, when a file has not been found.
  while (true) {
    char *name;

    // When desired match not found yet, try to find it (and others).
    if (use_tagstack) {
      // make a copy, the tagstack may change in 'tagfunc'
      name = xstrdup(tagstack[tagstackidx].tagname);
      xfree(tofree);
      tofree = name;
    } else if (g_do_tagpreview != 0) {
      name = ptag_entry.tagname;
    } else {
      name = tag;
    }
    bool other_name = (tagmatchname == NULL || strcmp(tagmatchname, name) != 0);
    if (new_tag
        || (cur_match >= num_matches && max_num_matches != MAXCOL)
        || other_name) {
      if (other_name) {
        xfree(tagmatchname);
        tagmatchname = xstrdup(name);
      }

      if (type == DT_SELECT || type == DT_JUMP
          || type == DT_LTAG) {
        cur_match = MAXCOL - 1;
      }
      max_num_matches = type == DT_TAG ? MAXCOL : cur_match + 1;

      // when the argument starts with '/', use it as a regexp
      if (!no_regexp && *name == '/') {
        flags = TAG_REGEXP;
        name++;
      } else {
        flags = TAG_NOIC;
      }

      flags |= verbose ? TAG_VERBOSE : 0;
      flags |= !use_tfu ? TAG_NO_TAGFUNC : 0;

      if (find_tags(name, &new_num_matches, &new_matches, flags,
                    max_num_matches, buf_ffname) == OK
          && new_num_matches < max_num_matches) {
        max_num_matches = MAXCOL;  // If less than max_num_matches
                                   // found: all matches found.
      }

      // A tag function may do anything, which may cause various
      // information to become invalid.  At least check for the tagstack
      // to still be the same.
      if (tagstack != curwin->w_tagstack) {
        emsg(_(e_window_unexpectedly_close_while_searching_for_tags));
        FreeWild(new_num_matches, new_matches);
        break;
      }

      // If there already were some matches for the same name, move them
      // to the start.  Avoids that the order changes when using
      // ":tnext" and jumping to another file.
      if (!new_tag && !other_name) {
        int idx = 0;
        tagptrs_T tagp, tagp2;

        // Find the position of each old match in the new list.  Need
        // to use parse_match() to find the tag line.
        for (int j = 0; j < num_matches; j++) {
          parse_match(matches[j], &tagp);
          for (int i = idx; i < new_num_matches; i++) {
            parse_match(new_matches[i], &tagp2);
            if (strcmp(tagp.tagname, tagp2.tagname) == 0) {
              char *p = new_matches[i];
              for (int k = i; k > idx; k--) {
                new_matches[k] = new_matches[k - 1];
              }
              new_matches[idx++] = p;
              break;
            }
          }
        }
      }
      FreeWild(num_matches, matches);
      num_matches = new_num_matches;
      matches = new_matches;
    }

    if (num_matches <= 0) {
      if (verbose) {
        semsg(_(e_tag_not_found_str), name);
      }
      g_do_tagpreview = 0;
    } else {
      bool ask_for_selection = false;

      if (type == DT_TAG && *tag != NUL) {
        // If a count is supplied to the ":tag <name>" command, then
        // jump to count'th matching tag.
        cur_match = count > 0 ? count - 1 : 0;
      } else if (type == DT_SELECT || (type == DT_JUMP && num_matches > 1)) {
        print_tag_list(new_tag, use_tagstack, num_matches, matches);
        ask_for_selection = true;
      } else if (type == DT_LTAG) {
        if (add_llist_tags(tag, num_matches, matches) == FAIL) {
          goto end_do_tag;
        }

        cur_match = 0;                  // Jump to the first tag
      }

      if (ask_for_selection) {
        // Ask to select a tag from the list.
        int i = prompt_for_input(NULL, 0, false, NULL);
        if (i <= 0 || i > num_matches || got_int) {
          // no valid choice: don't change anything
          if (use_tagstack) {
            tagstack[tagstackidx].fmark = saved_fmark;
            tagstackidx = prevtagstackidx;
          }
          break;
        }
        cur_match = i - 1;
      }

      if (cur_match >= num_matches) {
        // Avoid giving this error when a file wasn't found and we're
        // looking for a match in another file, which wasn't found.
        // There will be an emsg("file doesn't exist") below then.
        if ((type == DT_NEXT || type == DT_FIRST)
            && nofile_fname == NULL) {
          if (num_matches == 1) {
            emsg(_("E427: There is only one matching tag"));
          } else {
            emsg(_("E428: Cannot go beyond last matching tag"));
          }
          skip_msg = true;
        }
        cur_match = num_matches - 1;
      }
      if (use_tagstack) {
        tagptrs_T tagp2;

        tagstack[tagstackidx].cur_match = cur_match;
        tagstack[tagstackidx].cur_fnum = cur_fnum;

        // store user-provided data originating from tagfunc
        if (use_tfu && parse_match(matches[cur_match], &tagp2) == OK
            && tagp2.user_data) {
          XFREE_CLEAR(tagstack[tagstackidx].user_data);
          tagstack[tagstackidx].user_data =
            xmemdupz(tagp2.user_data, (size_t)(tagp2.user_data_end - tagp2.user_data));
        }

        tagstackidx++;
      } else if (g_do_tagpreview != 0) {
        ptag_entry.cur_match = cur_match;
        ptag_entry.cur_fnum = cur_fnum;
      }

      // Only when going to try the next match, report that the previous
      // file didn't exist.  Otherwise an emsg() is given below.
      if (nofile_fname != NULL && error_cur_match != cur_match) {
        smsg(0, _("File \"%s\" does not exist"), nofile_fname);
      }

      bool ic = (matches[cur_match][0] & MT_IC_OFF);
      if (type != DT_TAG && type != DT_SELECT && type != DT_JUMP
          && (num_matches > 1 || ic)
          && !skip_msg) {
        // Give an indication of the number of matching tags
        snprintf(IObuff, sizeof(IObuff), _("tag %d of %d%s"),
                 cur_match + 1,
                 num_matches,
                 max_num_matches != MAXCOL ? _(" or more") : "");
        if (ic) {
          xstrlcat(IObuff, _("  Using tag with different case!"), IOSIZE);
        }
        if ((num_matches > prev_num_matches || new_tag)
            && num_matches > 1) {
          msg(IObuff, ic ? HLF_W : 0);
          msg_scroll = true;  // Don't overwrite this message.
        } else {
          give_warning(IObuff, ic);
        }
        if (ic && !msg_scrolled && msg_silent == 0 && !ui_has(kUIMessages)) {
          ui_flush();
          os_delay(1007, true);
        }
      }

      // Let the SwapExists event know what tag we are jumping to.
      vim_snprintf(IObuff, IOSIZE, ":ta %s\r", name);
      set_vim_var_string(VV_SWAPCOMMAND, IObuff, -1);

      // Jump to the desired match.
      int i = jumpto_tag(matches[cur_match], forceit, true);

      set_vim_var_string(VV_SWAPCOMMAND, NULL, -1);

      if (i == NOTAGFILE) {
        // File not found: try again with another matching tag
        if ((type == DT_PREV && cur_match > 0)
            || ((type == DT_TAG || type == DT_NEXT
                 || type == DT_FIRST)
                && (max_num_matches != MAXCOL
                    || cur_match < num_matches - 1))) {
          error_cur_match = cur_match;
          if (use_tagstack) {
            tagstackidx--;
          }
          if (type == DT_PREV) {
            cur_match--;
          } else {
            type = DT_NEXT;
            cur_match++;
          }
          continue;
        }
        semsg(_("E429: File \"%s\" does not exist"), nofile_fname);
      } else {
        // We may have jumped to another window, check that
        // tagstackidx is still valid.
        if (use_tagstack && tagstackidx > curwin->w_tagstacklen) {
          tagstackidx = curwin->w_tagstackidx;
        }
      }
    }
    break;
  }

end_do_tag:
  // Only store the new index when using the tagstack and it's valid.
  if (use_tagstack && tagstackidx <= curwin->w_tagstacklen) {
    curwin->w_tagstackidx = tagstackidx;
  }
  postponed_split = 0;          // don't split next time
  g_do_tagpreview = 0;          // don't do tag preview next time
  xfree(tofree);
}

// List all the matching tags.
static void print_tag_list(bool new_tag, bool use_tagstack, int num_matches, char **matches)
{
  rs_print_tag_list(new_tag, use_tagstack, num_matches, matches);
}

/// Add the matching tags to the location list for the current
/// window.
static int add_llist_tags(char *tag, int num_matches, char **matches)
{
  return rs_add_llist_tags(tag, num_matches, matches);
}

// Free cached tags.
void tag_freematch(void)
{
  rs_tag_freematch();
}

static void taglen_advance(int l)
{
  rs_taglen_advance(l);
}

// Print the tag stack
void do_tags(exarg_T *eap)
{
  rs_do_tags();
}

// Compare two strings, for length "len", ignoring case the ASCII way.
// return 0 for match, < 0 for smaller, > 0 for bigger
// Make sure case is folded to uppercase in comparison (like for 'sort -f')
static int tag_strnicmp(char *s1, char *s2, size_t len)
{
  return rs_tag_strnicmp(s1, s2, len);
}

// Extract info from the tag search pattern "pats->pat".
static void prepare_pats(pat_T *pats, bool has_re)
{
  rs_prepare_pats(pats, has_re);
}

/// Call the user-defined function to generate a list of tags used by
/// find_tags().
///
/// Return OK if at least 1 tag has been successfully found,
/// NOTDONE if the function returns v:null, and FAIL otherwise.
///
/// @param pat  pattern supplied to the user-defined function
/// @param ga  the tags will be placed here
/// @param match_count  here the number of tags found will be placed
/// @param flags  flags from find_tags (TAG_*)
/// @param buf_ffname  name of buffer for priority
static int find_tagfunc_tags(char *pat, garray_T *ga, int *match_count, int flags, char *buf_ffname)
{
  return rs_find_tagfunc_tags(pat, ga, match_count, flags, buf_ffname);
}

/// Initialize the state used by find_tags()
static void findtags_state_init(findtags_state_T *st, char *pat, int flags, int mincount)
{
  rs_findtags_state_init(st, pat, flags, mincount);
}

/// Free the state used by find_tags()
static void findtags_state_free(findtags_state_T *st)
{
  rs_findtags_state_free(st);
}

/// Initialize the language and priority used for searching tags in a Vim help
/// file.
/// Returns true to process the help file for tags and false to skip the file.
static bool findtags_in_help_init(findtags_state_T *st)
{
  return rs_findtags_in_help_init(st);
}

/// Use the function set in 'tagfunc' (if configured and enabled) to get the
/// tags.
/// Return OK if at least 1 tag has been successfully found, NOTDONE if the
/// 'tagfunc' is not used, still executing or the 'tagfunc' returned v:null and
/// FAIL otherwise.
static int findtags_apply_tfu(findtags_state_T *st, char *pat, char *buf_ffname)
{
  return nvim_findtags_apply_tfu(st, pat, buf_ffname);
}

/// Read the next line from a tags file.
/// Returns TAGS_READ_SUCCESS if a tags line is successfully read and should be
/// processed.
/// Returns TAGS_READ_EOF if the end of file is reached.
/// Returns TAGS_READ_IGNORE if the current line should be ignored (used when
/// reached end of a emacs included tags file)
static tags_read_status_T findtags_get_next_line(findtags_state_T *st, tagsearch_info_T *sinfo_p)
{
  return (tags_read_status_T)rs_findtags_get_next_line(st, sinfo_p);
}

/// Parse a tags file header line in "st->lbuf".
/// Returns true if the current line in st->lbuf is not a tags header line and
/// should be parsed as a regular tag line. Returns false if the line is a
/// header line and the next header line should be read.
static bool findtags_hdr_parse(findtags_state_T *st)
{
  return rs_findtags_hdr_parse(st);
}

/// Handler to initialize the state when starting to process a new tags file.
/// Called in the TS_START state when finding tags from a tags file.
/// Returns true if the line read from the tags file should be parsed and
/// false if the line should be ignored.
static bool findtags_start_state_handler(findtags_state_T *st, bool *sortic,
                                         tagsearch_info_T *sinfo_p)
{
  return rs_findtags_start_state_handler(st, sortic, sinfo_p);
}

/// Parse a tag line read from a tags file.
/// Also compares the tag name in "tagpp->tagname" with a search pattern in
/// "st->orgpat->head" as a quick check if the tag may match.
/// Returns:
/// - TAG_MATCH_SUCCESS if the tag may match
/// - TAG_MATCH_FAIL if the tag doesn't match
/// - TAG_MATCH_NEXT to look for the next matching tag (used in a binary search)
/// - TAG_MATCH_STOP if all the tags are processed without a match.
/// Uses the values in "margs" for doing the comparison.
static tagmatch_status_T findtags_parse_line(findtags_state_T *st, tagptrs_T *tagpp,
                                             findtags_match_args_T *margs,
                                             tagsearch_info_T *sinfo_p)
{
  return (tagmatch_status_T)rs_findtags_parse_line(st, tagpp, margs, sinfo_p);
}

/// Initialize the structure used for tag matching.
static void findtags_matchargs_init(findtags_match_args_T *margs, int flags)
{
  rs_findtags_matchargs_init(margs, flags);
}

/// Compares the tag name in "tagpp->tagname" with a search pattern in
/// "st->orgpat->pat".
/// Returns true if the tag matches, false if the tag doesn't match.
/// Uses the values in "margs" for doing the comparison.
static bool findtags_match_tag(findtags_state_T *st, tagptrs_T *tagpp, findtags_match_args_T *margs)
{
  return rs_findtags_match_tag(st, tagpp, margs);
}

/// Convert the encoding of a line read from a tags file in "st->lbuf".
/// Converting the pattern from 'enc' to the tags file encoding doesn't work,
/// because characters are not recognized. The converted line is saved in
/// st->lbuf.
static void findtags_string_convert(findtags_state_T *st)
{
  rs_findtags_string_convert(st);
}

/// Add a matching tag found in a tags file to st->ht_match and st->ga_match.
static void findtags_add_match(findtags_state_T *st, tagptrs_T *tagpp, findtags_match_args_T *margs,
                               char *buf_ffname, hash_T *hash)
{
  rs_findtags_add_match(st, tagpp, margs, buf_ffname, hash);
}

/// Read and get all the tags from file st->tag_fname.
/// Sets "st->stop_searching" to true to stop searching for additional tags.
static void findtags_get_all_tags(findtags_state_T *st, findtags_match_args_T *margs,
                                  char *buf_ffname)
{
  rs_findtags_get_all_tags(st, margs, buf_ffname);
}

/// Search for tags matching "st->orgpat.pat" in the "st->tag_fname" tags file.
/// Information needed to search for the tags is in the "st" state structure.
/// The matching tags are returned in "st". If an error is encountered, then
/// "st->stop_searching" is set to true.
static void findtags_in_file(findtags_state_T *st, int flags, char *buf_ffname)
{
  rs_findtags_in_file(st, flags, buf_ffname);
}

/// Copy the tags found by find_tags() to "matchesp".
/// Returns the number of matches copied.
static int findtags_copy_matches(findtags_state_T *st, char ***matchesp)
{
  return rs_findtags_copy_matches(st, matchesp);
}

/// find_tags() - search for tags in tags files
///
/// Return FAIL if search completely failed (*num_matches will be 0, *matchesp
/// will be NULL), OK otherwise.
///
/// There is a priority in which type of tag is recognized.
///
///  6.  A static or global tag with a full matching tag for the current file.
///  5.  A global tag with a full matching tag for another file.
///  4.  A static tag with a full matching tag for another file.
///  3.  A static or global tag with an ignore-case matching tag for the
///      current file.
///  2.  A global tag with an ignore-case matching tag for another file.
///  1.  A static tag with an ignore-case matching tag for another file.
///
/// Tags in an emacs-style tags file are always global.
///
/// flags:
/// TAG_HELP       only search for help tags
/// TAG_NAMES      only return name of tag
/// TAG_REGEXP     use "pat" as a regexp
/// TAG_NOIC       don't always ignore case
/// TAG_KEEP_LANG  keep language
/// TAG_NO_TAGFUNC do not call the 'tagfunc' function
///
/// @param pat  pattern to search for
/// @param num_matches  return: number of matches found
/// @param matchesp  return: array of matches found
/// @param mincount  MAXCOL: find all matches
///                  other: minimal number of matches
/// @param buf_ffname  name of buffer for priority
int find_tags(char *pat, int *num_matches, char ***matchesp, int flags, int mincount,
              char *buf_ffname)
{
  findtags_state_T st;
  tagname_T tn;                         // info for get_tagfname()
  int first_file;                       // trying first tag file
  int retval = FAIL;                    // return value

  int i;
  char *saved_pat = NULL;                // copy of pat[]

  int findall = (mincount == MAXCOL || mincount == TAG_MANY);  // find all matching tags
  bool has_re = (flags & TAG_REGEXP);            // regexp used
  int noic = (flags & TAG_NOIC);
  int verbose = (flags & TAG_VERBOSE);
  int save_p_ic = p_ic;

  // uncrustify:off

  // Change the value of 'ignorecase' according to 'tagcase' for the
  // duration of this function.
  switch (curbuf->b_tc_flags ? curbuf->b_tc_flags : tc_flags) {
  case kOptTcFlagFollowic: break;
  case kOptTcFlagIgnore:
    p_ic = true;
    break;
  case kOptTcFlagMatch:
    p_ic = false;
    break;
  case kOptTcFlagFollowscs:
    p_ic = ignorecase(pat);
    break;
  case kOptTcFlagSmart:
    p_ic = ignorecase_opt(pat, true, true);
    break;
  default:
    abort();
  }

  // uncrustify:on

  int help_save = curbuf->b_help;

  findtags_state_init(&st, pat, flags, mincount);

  // Initialize a few variables
  if (st.help_only) {                           // want tags from help file
    curbuf->b_help = true;                      // will be restored later
  }

  if (curbuf->b_help) {
    // When "@ab" is specified use only the "ab" language, otherwise
    // search all languages.
    if (st.orgpat->len > 3 && pat[st.orgpat->len - 3] == '@'
        && ASCII_ISALPHA(pat[st.orgpat->len - 2])
        && ASCII_ISALPHA(pat[st.orgpat->len - 1])) {
      saved_pat = xstrnsave(pat, (size_t)st.orgpat->len - 3);
      st.help_lang_find = &pat[st.orgpat->len - 2];
      st.orgpat->pat = saved_pat;
      st.orgpat->len -= 3;
    }
  }
  if (p_tl != 0 && st.orgpat->len > p_tl) {  // adjust for 'taglength'
    st.orgpat->len = (int)p_tl;
  }

  int save_emsg_off = emsg_off;
  emsg_off = true;    // don't want error for invalid RE here
  prepare_pats(st.orgpat, has_re);
  emsg_off = save_emsg_off;
  if (has_re && st.orgpat->regmatch.regprog == NULL) {
    goto findtag_end;
  }

  retval = findtags_apply_tfu(&st, pat, buf_ffname);
  if (retval != NOTDONE) {
    goto findtag_end;
  }

  // re-initialize the default return value
  retval = FAIL;

  // Set a flag if the file extension is .txt
  if ((flags & TAG_KEEP_LANG)
      && st.help_lang_find == NULL
      && curbuf->b_fname != NULL
      && (i = (int)strlen(curbuf->b_fname)) > 4
      && STRICMP(curbuf->b_fname + i - 4, ".txt") == 0) {
    st.is_txt = true;
  }

  // When finding a specified number of matches, first try with matching
  // case, so binary search can be used, and try ignore-case matches in a
  // second loop.
  // When finding all matches, 'tagbsearch' is off, or there is no fixed
  // string to look for, ignore case right away to avoid going though the
  // tags files twice.
  // When the tag file is case-fold sorted, it is either one or the other.
  // Only ignore case when TAG_NOIC not used or 'ignorecase' set.
  st.orgpat->regmatch.rm_ic = ((p_ic || !noic)
                               && (findall || st.orgpat->headlen == 0 || !p_tbs));
  for (int round = 1; round <= 2; round++) {
    st.linear = (st.orgpat->headlen == 0 || !p_tbs || round == 2);

    // Try tag file names from tags option one by one.
    for (first_file = true;
         get_tagfname(&tn, first_file, st.tag_fname) == OK;
         first_file = false) {
      findtags_in_file(&st, flags, buf_ffname);
      if (st.stop_searching) {
        retval = OK;
        break;
      }
    }   // end of for-each-file loop

    tagname_free(&tn);

    // stop searching when already did a linear search, or when TAG_NOIC
    // used, and 'ignorecase' not set or already did case-ignore search
    if (st.stop_searching || st.linear || (!p_ic && noic)
        || st.orgpat->regmatch.rm_ic) {
      break;
    }

    // try another time while ignoring case
    st.orgpat->regmatch.rm_ic = true;
  }

  if (!st.stop_searching) {
    if (!st.did_open && verbose) {  // never opened any tags file
      emsg(_("E433: No tags file"));
    }
    retval = OK;                // It's OK even when no tag found
  }

findtag_end:
  findtags_state_free(&st);

  // Move the matches from the ga_match[] arrays into one list of
  // matches.  When retval == FAIL, free the matches.
  if (retval == FAIL) {
    st.match_count = 0;
  }

  *num_matches = findtags_copy_matches(&st, matchesp);

  curbuf->b_help = help_save;
  xfree(saved_pat);

  p_ic = save_p_ic;

  return retval;
}

static garray_T tag_fnames = GA_EMPTY_INIT_VALUE;

// Callback function for finding all "tags" and "tags-??" files in
// 'runtimepath' doc directories.
static bool found_tagfile_cb(int num_fnames, char **fnames, bool all, void *cookie)
{
  return rs_found_tagfile_cb(num_fnames, fnames, all, cookie);
}

// ============================================================================
// Rust FFI accessor functions for tag_fnames
// ============================================================================

/// Get the number of tag file names in the help file list
int nvim_tag_fnames_len(void)
{
  return tag_fnames.ga_len;
}

/// Get a tag file name from the help file list by index
const char *nvim_tag_fnames_get(int idx)
{
  if (idx < 0 || idx >= tag_fnames.ga_len) {
    return NULL;
  }
  return ((char **)(tag_fnames.ga_data))[idx];
}

/// Clear the help file tag names list
void nvim_tag_fnames_clear(void)
{
  ga_clear_strings(&tag_fnames);
}

/// Initialize the help file tag names list
void nvim_tag_fnames_init(void)
{
  ga_init(&tag_fnames, (int)sizeof(char *), 10);
}

/// Add a tag file name to the help file list
void nvim_tag_fnames_add(char *fname)
{
  GA_APPEND(char *, &tag_fnames, fname);
}

/// Do in runtimepath for tags (finds doc/tags files)
void nvim_do_in_runtimepath_for_tags(void)
{
  do_in_runtimepath("doc/tags doc/tags-??", DIP_ALL, found_tagfile_cb, NULL);
}

#if defined(EXITFREE)
void free_tag_stuff(void)
{
  rs_free_tag_stuff();
}

#endif

/// Get the next name of a tag file from the tag file list.
/// For help files, use "tags" file only.
///
/// @param tnp  holds status info
/// @param first  true when first file name is wanted
/// @param buf  pointer to buffer of MAXPATHL chars
///
/// @return  FAIL if no more tag file names, OK otherwise.
int get_tagfname(tagname_T *tnp, int first, char *buf)
{
  return rs_get_tagfname(tnp, first, buf);
}

// Free the contents of a tagname_T that was filled by get_tagfname().
void tagname_free(tagname_T *tnp)
{
  rs_tagname_free(tnp);
}

/// Parse one line from the tags file. Find start/end of tag name, start/end of
/// file name and start of search pattern.
///
/// If is_etag is true, tagp->fname and tagp->fname_end are not set.
///
/// @param lbuf  line to be parsed
///
/// @return  FAIL if there is a format error in this line, OK otherwise.
static int parse_tag_line(char *lbuf, tagptrs_T *tagp)
{
  return rs_parse_tag_line(lbuf, tagp);
}

// Check if tagname is a static tag
//
// Static tags produced by the older ctags program have the format:
//      'file:tag  file  /pattern'.
// This is only recognized when both occurrence of 'file' are the same, to
// avoid recognizing "string::string" or ":exit".
//
// Static tags produced by the new ctags program have the format:
//      'tag  file  /pattern/;"<Tab>file:'          "
//
// Return true if it is a static tag and adjust *tagname to the real tag.
// Return false if it is not a static tag.
static bool test_for_static(tagptrs_T *tagp)
{
  return rs_test_for_static(tagp);
}

/// @return  the length of a matching tag line.
static size_t matching_line_len(const char *const lbuf)
{
  return rs_matching_line_len(lbuf);
}

/// Parse a line from a matching tag.  Does not change the line itself.
///
/// The line that we get looks like this:
/// Emacs tag: <mtt><tag_fname><NUL><ebuf><NUL><lbuf>
/// other tag: <mtt><tag_fname><NUL><NUL><lbuf>
/// without Emacs tags: <mtt><tag_fname><NUL><lbuf>
///
/// @param lbuf  input: matching line
/// @param tagp  output: pointers into the line
///
/// @return  OK or FAIL.
static int parse_match(char *lbuf, tagptrs_T *tagp)
{
  return rs_parse_match(lbuf, tagp);
}

// Find out the actual file name of a tag.  Concatenate the tags file name
// with the matching tag file name.
// Returns an allocated string.
static char *tag_full_fname(tagptrs_T *tagp)
{
  return rs_tag_full_fname(tagp);
}

/// Jump to a tag that has been found in one of the tag files
///
/// @param lbuf_arg  line from the tags file for this tag
/// @param forceit  :ta with !
/// @param keep_help  keep help flag
///
/// @return  OK for success, NOTAGFILE when file not found, FAIL otherwise.
static int jumpto_tag(const char *lbuf_arg, int forceit, bool keep_help)
{
  if (postponed_split == 0 && !check_can_set_curbuf_forceit(forceit)) {
    return FAIL;
  }

  char *pbuf_end;
  char *tofree_fname = NULL;
  tagptrs_T tagp;
  int retval = FAIL;
  int getfile_result = GETFILE_UNUSED;
  int search_options;
  win_T *curwin_save = NULL;
  char *full_fname = NULL;
  const bool old_KeyTyped = KeyTyped;       // getting the file may reset it
  const int l_g_do_tagpreview = g_do_tagpreview;
  const size_t len = matching_line_len(lbuf_arg) + 1;
  char *lbuf = xmalloc(len);
  memmove(lbuf, lbuf_arg, len);

  char *pbuf = xmalloc(LSIZE);  // search pattern buffer

  // parse the match line into the tagp structure
  if (parse_match(lbuf, &tagp) == FAIL) {
    tagp.fname_end = NULL;
    goto erret;
  }

  // truncate the file name, so it can be used as a string
  *tagp.fname_end = NUL;
  char *fname = tagp.fname;

  // copy the command to pbuf[], remove trailing CR/NL
  char *str = tagp.command;
  for (pbuf_end = pbuf; *str && *str != '\n' && *str != '\r';) {
    *pbuf_end++ = *str++;
    if (pbuf_end - pbuf + 1 >= LSIZE) {
      break;
    }
  }
  *pbuf_end = NUL;

  {
    // Remove the "<Tab>fieldname:value" stuff; we don't need it here.
    str = pbuf;
    if (find_extra(&str) == OK) {
      pbuf_end = str;
      *pbuf_end = NUL;
    }
  }

  // Expand file name, when needed (for environment variables).
  // If 'tagrelative' option set, may change file name.
  fname = expand_tag_fname(fname, tagp.tag_fname, true);
  tofree_fname = fname;         // free() it later

  // Check if the file with the tag exists before abandoning the current
  // file.  Also accept a file name for which there is a matching BufReadCmd
  // autocommand event (e.g., http://sys/file).
  if (!os_path_exists(fname)
      && !has_autocmd(EVENT_BUFREADCMD, fname, NULL)) {
    retval = NOTAGFILE;
    xfree(nofile_fname);
    nofile_fname = xstrdup(fname);
    goto erret;
  }

  RedrawingDisabled++;

  if (l_g_do_tagpreview != 0) {
    postponed_split = 0;        // don't split again below
    curwin_save = curwin;       // Save current window

    // If we are reusing a window, we may change dir when
    // entering it (autocommands) so turn the tag filename
    // into a fullpath
    if (!curwin->w_p_pvw) {
      full_fname = FullName_save(fname, false);
      fname = full_fname;

      // Make the preview window the current window.
      // Open a preview window when needed.
      prepare_tagpreview(true);
    }
  }

  // If it was a CTRL-W CTRL-] command split window now.  For ":tab tag"
  // open a new tab page.
  if (postponed_split && (swb_flags & (kOptSwbFlagUseopen | kOptSwbFlagUsetab))) {
    buf_T *const existing_buf = buflist_findname_exp(fname);

    if (existing_buf != NULL) {
      // If 'switchbuf' is set jump to the window containing "buf".
      if (swbuf_goto_win_with_buf(existing_buf) != NULL) {
        // We've switched to the buffer, the usual loading of the file
        // must be skipped.
        getfile_result = GETFILE_SAME_FILE;
      }
    }
  }
  if (getfile_result == GETFILE_UNUSED
      && (postponed_split || cmdmod.cmod_tab != 0)) {
    if (win_split(postponed_split > 0 ? postponed_split : 0,
                  postponed_split_flags) == FAIL) {
      RedrawingDisabled--;
      goto erret;
    }
    RESET_BINDING(curwin);
  }

  if (keep_help) {
    // A :ta from a help file will keep the b_help flag set.  For ":ptag"
    // we need to use the flag from the window where we came from.
    if (l_g_do_tagpreview != 0) {
      keep_help_flag = bt_help(curwin_save->w_buffer);
    } else {
      keep_help_flag = curbuf->b_help;
    }
  }

  if (getfile_result == GETFILE_UNUSED) {
    // Careful: getfile() may trigger autocommands and call jumpto_tag()
    // recursively.
    getfile_result = getfile(0, fname, NULL, true, 0, forceit);
  }
  keep_help_flag = false;

  if (GETFILE_SUCCESS(getfile_result)) {    // got to the right file
    curwin->w_set_curswant = true;
    postponed_split = 0;

    const optmagic_T save_magic_overruled = magic_overruled;
    magic_overruled = OPTION_MAGIC_OFF;  // always execute with 'nomagic'
    // Save value of no_hlsearch, jumping to a tag is not a real search
    const bool save_no_hlsearch = no_hlsearch;

    // If 'cpoptions' contains 't', store the search pattern for the "n"
    // command.  If 'cpoptions' does not contain 't', the search pattern
    // is not stored.
    if (vim_strchr(p_cpo, CPO_TAGPAT) != NULL) {
      search_options = 0;
    } else {
      search_options = SEARCH_KEEP;
    }

    // If the command is a search, try here.
    //
    // Reset 'smartcase' for the search, since the search pattern was not
    // typed by the user.
    // Only use do_search() when there is a full search command, without
    // anything following.
    str = pbuf;
    if (pbuf[0] == '/' || pbuf[0] == '?') {
      str = skip_regexp(pbuf + 1, pbuf[0], false) + 1;
    }
    if (str > pbuf_end - 1) {   // search command with nothing following
      size_t pbuflen = (size_t)(pbuf_end - pbuf);

      bool save_p_ws = p_ws;
      int save_p_ic = p_ic;
      int save_p_scs = p_scs;
      p_ws = true;              // need 'wrapscan' for backward searches
      p_ic = false;             // don't ignore case now
      p_scs = false;
      linenr_T save_lnum = curwin->w_cursor.lnum;

      curwin->w_cursor.lnum = tagp.tagline > 0
                              // start search before line from "line:" field
                              ? tagp.tagline - 1
                              // start search before first line
                              : 0;

      if (do_search(NULL, pbuf[0], pbuf[0], pbuf + 1, pbuflen - 1, 1,
                    search_options, NULL)) {
        retval = OK;
      } else {
        int found = 1;

        // try again, ignore case now
        p_ic = true;
        if (!do_search(NULL, pbuf[0], pbuf[0], pbuf + 1, pbuflen - 1, 1,
                       search_options, NULL)) {
          // Failed to find pattern, take a guess: "^func  ("
          found = 2;
          test_for_static(&tagp);
          char cc = *tagp.tagname_end;
          *tagp.tagname_end = NUL;
          pbuflen = (size_t)snprintf(pbuf, LSIZE, "^%s\\s\\*(", tagp.tagname);
          if (!do_search(NULL, '/', '/', pbuf, pbuflen, 1, search_options, NULL)) {
            // Guess again: "^char * \<func  ("
            pbuflen = (size_t)snprintf(pbuf, LSIZE, "^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(",
                                       tagp.tagname);
            if (!do_search(NULL, '/', '/', pbuf, pbuflen, 1, search_options, NULL)) {
              found = 0;
            }
          }
          *tagp.tagname_end = cc;
        }
        if (found == 0) {
          emsg(_("E434: Can't find tag pattern"));
          curwin->w_cursor.lnum = save_lnum;
        } else {
          // Only give a message when really guessed, not when 'ic'
          // is set and match found while ignoring case.
          if (found == 2 || !save_p_ic) {
            msg(_("E435: Couldn't find tag, just guessing!"), 0);
            if (!msg_scrolled && msg_silent == 0 && !ui_has(kUIMessages)) {
              ui_flush();
              os_delay(1010, true);
            }
          }
          retval = OK;
        }
      }
      p_ws = save_p_ws;
      p_ic = save_p_ic;
      p_scs = save_p_scs;

      // A search command may have positioned the cursor beyond the end
      // of the line.  May need to correct that here.
      check_cursor(curwin);
    } else {
      const int save_secure = secure;

      // Setup the sandbox for executing the command from the tags file.
      secure = 1;
      sandbox++;
      curwin->w_cursor.lnum = 1;  // start command in line 1
      curwin->w_cursor.col = 0;
      curwin->w_cursor.coladd = 0;
      do_cmdline_cmd(pbuf);
      retval = OK;

      // When the command has done something that is not allowed make sure
      // the error message can be seen.
      if (secure == 2) {
        wait_return(true);
      }
      secure = save_secure;
      sandbox--;
    }

    magic_overruled = save_magic_overruled;
    // restore no_hlsearch when keeping the old search pattern
    if (search_options) {
      set_no_hlsearch(save_no_hlsearch);
    }

    // Return OK if jumped to another file (at least we found the file!).
    if (getfile_result == GETFILE_OPEN_OTHER) {
      retval = OK;
    }

    if (retval == OK) {
      // For a help buffer: Put the cursor line at the top of the window,
      // the help subject will be below it.
      if (curbuf->b_help) {
        set_topline(curwin, curwin->w_cursor.lnum);
      }
      if ((fdo_flags & kOptFdoFlagTag) && old_KeyTyped) {
        foldOpenCursor();
      }
    }

    if (l_g_do_tagpreview != 0
        && curwin != curwin_save && win_valid(curwin_save)) {
      // Return cursor to where we were
      validate_cursor(curwin);
      redraw_later(curwin, UPD_VALID);
      win_enter(curwin_save, true);
    }

    RedrawingDisabled--;
  } else {
    RedrawingDisabled--;
    if (postponed_split) {              // close the window
      win_close(curwin, false, false);
      postponed_split = 0;
    }
  }

erret:
  g_do_tagpreview = 0;  // For next time
  xfree(lbuf);
  xfree(pbuf);
  xfree(tofree_fname);
  xfree(full_fname);

  return retval;
}

/// If "expand" is true, expand wildcards in fname.
/// If 'tagrelative' option set, change fname (name of file containing tag)
/// according to tag_fname (name of tag file containing fname).
///
/// @return  a pointer to allocated memory.
static char *expand_tag_fname(char *fname, char *const tag_fname, const bool expand)
{
  return rs_expand_tag_fname(fname, tag_fname, expand);
}

/// Expand tag filename relative to tag file (wrapper for Rust)
char *nvim_expand_tag_fname(const char *fname, const char *tag_fname, bool expand)
{
  return rs_expand_tag_fname((char *)fname, (char *)tag_fname, expand);
}

/// Check if we have a tag for the buffer with name "buf_ffname".
/// This is a bit slow, because of the full path compare in path_full_compare().
///
/// @return  true if tag for file "fname" if tag file "tag_fname" is for current
///          file.
static int test_for_current(char *fname, char *fname_end, char *tag_fname, char *buf_ffname)
{
  return rs_test_for_current(fname, fname_end, tag_fname, buf_ffname);
}

// Find the end of the tagaddress.
// Return OK if ";\"" is following, FAIL otherwise.
static int find_extra(char **pp)
{
  return rs_find_extra(pp);
}

/// Free a single entry in a tag stack
void tagstack_clear_entry(taggy_T *item)
{
  rs_tagstack_clear_entry(item);
}

/// @param tagnames  expand tag names
int expand_tags(bool tagnames, char *pat, int *num_file, char ***file)
{
  return rs_expand_tags(tagnames, pat, num_file, file);
}

/// Add a tag field to the dictionary "dict".
/// Return OK or FAIL.
///
/// @param start  start of the value
/// @param end  after the value; can be NULL
static int add_tag_field(dict_T *dict, const char *field_name, const char *start, const char *end)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  return rs_add_tag_field(dict, field_name, start, end);
}

/// Add the tags matching the specified pattern "pat" to the list "list"
/// as a dictionary. Use "buf_fname" for priority, unless NULL.
int get_tags(list_T *list, char *pat, char *buf_fname)
{
  return rs_get_tags(list, pat, buf_fname);
}

// Return information about 'tag' in dict 'retdict'.
static void get_tag_details(taggy_T *tag, dict_T *retdict)
{
  rs_get_tag_details(tag, retdict);
}

// Return the tag stack entries of the specified window 'wp' in dictionary
// 'retdict'.
void get_tagstack(win_T *wp, dict_T *retdict)
{
  rs_get_tagstack(wp, retdict);
}

// Free all the entries in the tag stack of the specified window
static void tagstack_clear(win_T *wp)
{
  rs_tagstack_clear(wp);
}

// Remove the oldest entry from the tag stack and shift the rest of
// the entries to free up the top of the stack.
static void tagstack_shift(win_T *wp)
{
  rs_tagstack_shift(wp);
}

// tagstack_push_item, tagstack_push_items, tagstack_set_curidx:
// now handled by rs_set_tagstack in Rust

// Set the tag stack entries of the specified window.
// 'action' is set to one of:
//    'a' for append
//    'r' for replace
//    't' for truncate
int set_tagstack(win_T *wp, const dict_T *d, int action)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_set_tagstack(wp, d, action);
}
