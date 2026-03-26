// Code to handle tags and the tag stack

#include <assert.h>
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

int nvim_win_get_tagstacklen(const void *wp_void) { const win_T *wp = (const win_T *)wp_void; return wp->w_tagstacklen; }
int nvim_win_get_tagstackidx(const void *wp_void) { const win_T *wp = (const win_T *)wp_void; return wp->w_tagstackidx; }
void *nvim_win_get_tagstack_entry(const void *wp_void, int idx) { const win_T *wp = (const win_T *)wp_void; return (void *)&wp->w_tagstack[idx]; }
const char *nvim_taggy_get_tagname(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->tagname; }
int nvim_taggy_get_cur_match(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->cur_match; }
int nvim_taggy_get_cur_fnum(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->cur_fnum; }
void *nvim_taggy_get_fmark(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return (void *)&tg->fmark; }
const char *nvim_taggy_get_user_data(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->user_data; }
linenr_T nvim_fmark_get_lnum(const void *fm_void) { const fmark_T *fm = (const fmark_T *)fm_void; return fm->mark.lnum; }
int nvim_fmark_get_col(const void *fm_void) { const fmark_T *fm = (const fmark_T *)fm_void; return fm->mark.col; }
int nvim_fmark_get_fnum(const void *fm_void) { const fmark_T *fm = (const fmark_T *)fm_void; return fm->fnum; }
void nvim_win_set_tagstacklen(void *wp_void, int len) { win_T *wp = (win_T *)wp_void; wp->w_tagstacklen = len; }
void nvim_win_set_tagstackidx(void *wp_void, int idx) { win_T *wp = (win_T *)wp_void; wp->w_tagstackidx = idx; }
void nvim_taggy_set_tagname(void *tg_void, char *name) { taggy_T *tg = (taggy_T *)tg_void; tg->tagname = name; }
void nvim_taggy_set_cur_match(void *tg_void, int match_idx) { taggy_T *tg = (taggy_T *)tg_void; tg->cur_match = match_idx; }
void nvim_taggy_set_cur_fnum(void *tg_void, int fnum) { taggy_T *tg = (taggy_T *)tg_void; tg->cur_fnum = fnum; }
void nvim_taggy_set_user_data(void *tg_void, char *data) { taggy_T *tg = (taggy_T *)tg_void; tg->user_data = data; }
int nvim_taggy_get_fmark_col(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->fmark.mark.col; }
int nvim_taggy_get_fmark_fnum(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->fmark.fnum; }
void nvim_taggy_set_fmark_lnum(void *tg_void, linenr_T lnum) { taggy_T *tg = (taggy_T *)tg_void; tg->fmark.mark.lnum = lnum; }
void nvim_taggy_set_fmark_col(void *tg_void, int col) { taggy_T *tg = (taggy_T *)tg_void; tg->fmark.mark.col = col; }
void nvim_taggy_set_fmark_fnum(void *tg_void, int fnum) { taggy_T *tg = (taggy_T *)tg_void; tg->fmark.fnum = fnum; }
int nvim_findtags_get_state(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return (int)st->state; }
int nvim_findtags_get_match_count(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->match_count; }
bool nvim_findtags_get_help_only(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->help_only; }
bool nvim_findtags_get_linear(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->linear; }
int nvim_findtags_get_tag_file_sorted(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->tag_file_sorted; }
int64_t nvim_get_p_tl(void) { return p_tl; }
void nvim_findtags_init_tag_fname(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->tag_fname = xmalloc(MAXPATHL + 1); }
void nvim_findtags_set_fp_null(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->fp = NULL; }
/// Allocate orgpat struct (does not set fields -- Rust sets them individually).
void nvim_findtags_alloc_orgpat(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat = xmalloc(sizeof(pat_T)); }
/// Set orgpat->regmatch.regprog to NULL.
void nvim_findtags_clear_orgpat_regprog(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->regmatch.regprog = NULL; }
/// Set st->flags.
void nvim_findtags_set_flags(void *st_void, int flags) { findtags_state_T *st = (findtags_state_T *)st_void; st->flags = flags; }
/// Set st->help_only from flags.
void nvim_findtags_set_help_only_from_flags(void *st_void, int flags) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_only = (flags & TAG_HELP) != 0; }
/// Set st->mincount.
void nvim_findtags_set_mincount(void *st_void, int mincount) { findtags_state_T *st = (findtags_state_T *)st_void; st->mincount = mincount; }
/// Allocate st->lbuf at LSIZE bytes and set st->lbuf_size.
void nvim_findtags_alloc_lbuf(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->lbuf_size = LSIZE; st->lbuf = xmalloc((size_t)st->lbuf_size); }
/// Free st->tag_fname.
void nvim_findtags_free_tag_fname(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; xfree(st->tag_fname); }
/// Free st->lbuf.
void nvim_findtags_free_lbuf(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; xfree(st->lbuf); }
/// Free st->orgpat->regmatch.regprog via vim_regfree.
void nvim_findtags_free_orgpat_regprog(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; vim_regfree(st->orgpat->regmatch.regprog); }
/// Free st->orgpat.
void nvim_findtags_free_orgpat(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; xfree(st->orgpat); }
/// Heap-allocate a zero-initialized findtags_state_T (caller must call rs_findtags_state_init).
void *nvim_findtags_state_xcalloc(void) { return xcalloc(1, sizeof(findtags_state_T)); }

/// Initialize ga_match and ht_match arrays
void nvim_findtags_init_match_arrays(void *st_void)
{
  findtags_state_T *st = (findtags_state_T *)st_void;
  for (int mtt = 0; mtt < MT_COUNT; mtt++) {
    ga_init(&st->ga_match[mtt], sizeof(char *), 100);
    hash_init(&st->ht_match[mtt]);
  }
}

/// Free the findtags_state_T struct itself (inner resources already freed by rs_findtags_state_free).
void nvim_findtags_state_delete(void *st_void) { xfree(st_void); }
/// Get the mutable tag_fname buffer from the state.
char *nvim_findtags_get_tag_fname_buf(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return st->tag_fname; }

bool nvim_curbuf_is_help(void) { return curbuf->b_help; }
const char *nvim_get_p_hf(void) { return p_hf; }
const char *nvim_get_curbuf_tags(void) { return curbuf->b_p_tags; }
const char *nvim_get_p_tags(void) { return p_tags; }
char *nvim_path_tail(char *path) { return path_tail(path); }
void nvim_simplify_filename(char *fname) { simplify_filename(fname); }
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

char *nvim_vim_findfile(void *search_ctx) { return vim_findfile(search_ctx); }
void nvim_vim_findfile_cleanup(void *search_ctx) { vim_findfile_cleanup(search_ctx); }
char *nvim_vim_findfile_stopdir(char *buf) { return vim_findfile_stopdir(buf); }
const char *nvim_get_curbuf_ffname(void) { return curbuf->b_ffname; }
void nvim_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep) { copy_option_part(option, buf, maxlen, (char *)sep); }
bool nvim_tag_path_exists(const char *path) { return os_path_exists(path); }
bool nvim_has_bufreadcmd(const char *fname) { return has_autocmd(EVENT_BUFREADCMD, fname, NULL); }
int nvim_get_postponed_split(void) { return postponed_split; }
void nvim_set_postponed_split(int val) { postponed_split = val; }
int nvim_get_g_do_tagpreview(void) { return g_do_tagpreview; }
void nvim_set_g_do_tagpreview(int val) { g_do_tagpreview = val; }
bool nvim_check_can_set_curbuf_forceit(int forceit) { return check_can_set_curbuf_forceit(forceit); }
void nvim_set_nofile_fname(const char *fname) { xfree(nofile_fname); nofile_fname = fname != NULL ? xstrdup(fname) : NULL; }
const char *nvim_get_nofile_fname(void) { return nofile_fname; }
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern void rs_prepare_pats(pat_T *pats, bool has_re);
extern bool rs_found_tagfile_cb(int num_fnames, char **fnames, bool all, void *cookie);

#include "tag_shim.c.generated.h"

static char *tagmatchname = NULL;   // name of last used tag

// Tag for preview window is remembered separately, to avoid messing up the
// normal tagstack.
static taggy_T ptag_entry = { NULL, INIT_FMARK, 0, 0, NULL };

static bool tfu_in_use = false;  // disallow recursive call of tagfunc
static Callback tfu_cb;          // 'tagfunc' callback function

void nvim_xfree_clear_tagmatchname(void) { XFREE_CLEAR(tagmatchname); }
const char *nvim_get_tagmatchname(void) { return tagmatchname; }
void nvim_set_tagmatchname(char *name) { tagmatchname = name; }
void *nvim_get_ptag_entry(void) { return &ptag_entry; }
int nvim_path_full_compare_equal(const char *s1, const char *s2) { return (path_full_compare((char *)s1, (char *)s2, true, true) & kEqualFiles); }
bool nvim_tag_curwin_is_null(void) { return curwin == NULL; }
bool nvim_path_has_wildcard(const char *fname) { return path_has_wildcard(fname); }
/// Expand wildcards in a filename (ExpandInit + ExpandOne)
char *nvim_expand_one_file(char *fname)
{
  expand_T xpc;
  ExpandInit(&xpc);
  xpc.xp_context = EXPAND_FILES;
  return ExpandOne(&xpc, fname, NULL,
                   WILD_LIST_NOTFOUND|WILD_SILENT, WILD_EXPAND_FREE);
}

bool nvim_vim_isAbsName(const char *fname) { return vim_isAbsName(fname); }
bool nvim_get_p_tr(void) { return p_tr; }
void nvim_findtags_set_state_val(void *st_void, int state) { findtags_state_T *st = (findtags_state_T *)st_void; st->state = (tagsearch_state_T)state; }
char *nvim_findtags_get_lbuf(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->lbuf; }
int nvim_findtags_get_lbuf_size(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->lbuf_size; }
/// Set st->lbuf and st->lbuf_size (for string_convert swap)
void nvim_findtags_set_lbuf(void *st_void, char *lbuf, int lbuf_size) { findtags_state_T *st = (findtags_state_T *)st_void; st->lbuf = lbuf; st->lbuf_size = lbuf_size; }
bool nvim_findtags_fgets(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return vim_fgets(st->lbuf, st->lbuf_size, st->fp); }
int nvim_findtags_fseek(void *st_void, int64_t offset, int whence) { findtags_state_T *st = (findtags_state_T *)st_void; return vim_fseek(st->fp, (off_T)offset, whence); }
int64_t nvim_findtags_ftell(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return (int64_t)vim_ftell(st->fp); }
void nvim_findtags_fseek_zero(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; fseek(st->fp, 0, SEEK_SET); }
bool nvim_findtags_lbuf_is_blank(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return vim_isblankline(st->lbuf); }
int nvim_findtags_get_flags(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->flags; }
void nvim_findtags_set_linear(void *st_void, bool linear) { findtags_state_T *st = (findtags_state_T *)st_void; st->linear = linear; }
int nvim_findtags_get_sorted(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->tag_file_sorted; }
void nvim_findtags_set_sorted(void *st_void, int val) { findtags_state_T *st = (findtags_state_T *)st_void; st->tag_file_sorted = val; }
bool nvim_findtags_get_orgpat_rm_ic(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->orgpat->regmatch.rm_ic; }
void nvim_findtags_set_orgpat_rm_ic(void *st_void, bool ic) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->regmatch.rm_ic = ic; }
void nvim_findtags_convert_setup(void *st_void, const char *from) { findtags_state_T *st = (findtags_state_T *)st_void; convert_setup(&st->vimconv, (char *)from, p_enc); }
char *nvim_findtags_string_convert(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return string_convert(&st->vimconv, st->lbuf, NULL); }
int nvim_findtags_get_orgpat_headlen(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->orgpat->headlen; }
const char *nvim_findtags_get_orgpat_head(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->orgpat->head; }
const char *nvim_findtags_get_orgpat_pat(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->orgpat->pat; }
int nvim_findtags_get_orgpat_len(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->orgpat->len; }
bool nvim_findtags_has_regprog(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->orgpat->regmatch.regprog != NULL; }
bool nvim_findtags_vim_regexec(void *st_void, const char *tagname) { findtags_state_T *st = (findtags_state_T *)st_void; return vim_regexec(&st->orgpat->regmatch, (char *)tagname, 0); }
int nvim_findtags_get_regmatch_startoff(const void *st_void, const char *tagname) { const findtags_state_T *st = (const findtags_state_T *)st_void; return (int)(st->orgpat->regmatch.startp[0] - tagname); }
int nvim_mb_strnicmp(const char *s1, const char *s2, size_t len) { return mb_strnicmp(s1, s2, len); }
const char *nvim_findtags_get_tag_fname(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->tag_fname; }
const char *nvim_findtags_get_help_lang(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->help_lang; }
int nvim_findtags_get_help_pri(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->help_pri; }
bool nvim_findtags_get_searchpat(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->get_searchpat; }
void nvim_findtags_set_searchpat(void *st_void, bool val) { findtags_state_T *st = (findtags_state_T *)st_void; st->get_searchpat = val; }
void nvim_findtags_inc_match_count(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->match_count++; }
void nvim_findtags_set_match_count(void *st_void, int count) { findtags_state_T *st = (findtags_state_T *)st_void; st->match_count = count; }
int nvim_get_current_State(void) { return State; }
bool nvim_get_p_sft(void) { return p_sft; }
int nvim_help_heuristic(const char *tagname, int match_offset, bool wrong_case) { return help_heuristic((char *)tagname, match_offset, wrong_case); }
// Verify hash_T size matches Rust usize assumption
_Static_assert(sizeof(hash_T) == sizeof(size_t), "hash_T must be size_t");

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

int nvim_findtags_ga_match_len(const void *st_void, int mtt) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->ga_match[mtt].ga_len; }
char *nvim_findtags_ga_match_get(const void *st_void, int mtt, int idx) { const findtags_state_T *st = (const findtags_state_T *)st_void; return ((char **)(st->ga_match[mtt].ga_data))[idx]; }
/// Clear ga_match[mtt] and ht_match[mtt]
void nvim_findtags_clear_match(void *st_void, int mtt) { findtags_state_T *st = (findtags_state_T *)st_void; ga_clear(&st->ga_match[mtt]); hash_clear(&st->ht_match[mtt]); }
bool nvim_findtags_get_stop_searching(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->stop_searching; }
void nvim_findtags_set_stop_searching(void *st_void, bool val) { findtags_state_T *st = (findtags_state_T *)st_void; st->stop_searching = val; }
bool nvim_findtags_get_is_txt(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->is_txt; }
void nvim_findtags_set_is_txt(void *st_void, bool val) { findtags_state_T *st = (findtags_state_T *)st_void; st->is_txt = val; }
const char *nvim_findtags_get_help_lang_find(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->help_lang_find; }
void nvim_findtags_set_help_lang_find(void *st_void, const char *val) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_lang_find = (char *)val; }
void nvim_findtags_set_help_pri(void *st_void, int pri) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_pri = pri; }
/// Set st->help_lang (copies 2 bytes + NUL)
void nvim_findtags_set_help_lang(void *st_void, const char *lang) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_lang[0] = lang[0]; st->help_lang[1] = lang[1]; st->help_lang[2] = NUL; }
int nvim_findtags_get_vimconv_type(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return (int)st->vimconv.vc_type; }
void nvim_findtags_set_vimconv_none(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->vimconv.vc_type = CONV_NONE; }
void nvim_findtags_convert_cleanup(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; convert_setup(&st->vimconv, NULL, NULL); }
/// Open st->tag_fname for reading, set st->fp
bool nvim_findtags_fopen(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->fp = os_fopen(st->tag_fname, "r"); return st->fp != NULL; }
/// Close st->fp if not NULL
void nvim_findtags_fclose(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; if (st->fp != NULL) { fclose(st->fp); st->fp = NULL; } }
void nvim_findtags_set_did_open(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->did_open = true; }
bool nvim_findtags_get_did_open(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->did_open; }
void nvim_findtags_set_state_start(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->state = TS_START; }
int nvim_findtags_get_mincount(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->mincount; }
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

void nvim_findtags_set_orgpat_len(void *st_void, int len) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->len = len; }
void nvim_findtags_set_orgpat_pat(void *st_void, char *pat) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->pat = pat; }
// Global variable accessors for search orchestration

void nvim_set_p_ic(int val) { p_ic = val; }
bool nvim_get_p_tbs(void) { return p_tbs; }
int nvim_get_tc_flags(void) { return (int)tc_flags; }
int nvim_get_curbuf_tc_flags(void) { return (int)curbuf->b_tc_flags; }
const char *nvim_get_p_hlg(void) { return p_hlg; }
const char *nvim_get_curbuf_b_fname(void) { return curbuf->b_fname; }
const char *nvim_get_curbuf_b_ffname(void) { return curbuf->b_ffname; }
const char *nvim_get_curbuf_p_tfu(void) { return curbuf->b_p_tfu; }
void nvim_set_curbuf_b_help(int val) { curbuf->b_help = val; }
int nvim_get_curbuf_b_help(void) { return curbuf->b_help; }
// Function wrappers for search orchestration

void nvim_ins_compl_check_keys(int interval, bool pum_wanted) { rs_ins_compl_check_keys(interval, pum_wanted ? 1 : 0); }
bool nvim_ignorecase(const char *pat) { return ignorecase((char *)pat); }
bool nvim_ignorecase_opt(const char *pat, bool ic_strstrp, bool ic_strstrp2) { return ignorecase_opt((char *)pat, ic_strstrp, ic_strstrp2); }
void nvim_findtags_prepare_pats(void *st_void, bool has_re) { findtags_state_T *st = (findtags_state_T *)st_void; rs_prepare_pats(st->orgpat, has_re); }

void *nvim_tag_get_curwin(void) { return (void *)curwin; }
void *nvim_tag_tv_dict_alloc(void) { return (void *)tv_dict_alloc(); }
bool nvim_tag_tv_dict_find(void *dict, const char *key, int key_len) { return tv_dict_find((dict_T *)dict, key, key_len) != NULL; }
int nvim_tag_tv_dict_add_str(void *dict, const char *key, size_t key_len, char *val) { return tv_dict_add_str((dict_T *)dict, key, key_len, val); }
int nvim_tag_tv_dict_add_nr(void *dict, const char *key, size_t key_len, int64_t nr) { return tv_dict_add_nr((dict_T *)dict, key, key_len, (varnumber_T)nr); }
void *nvim_tag_tv_list_alloc(int count) { return (void *)tv_list_alloc(count); }
void nvim_tag_tv_list_append_dict(void *list, void *dict) { tv_list_append_dict((list_T *)list, (dict_T *)dict); }
void nvim_tag_tv_list_free(void *list) { tv_list_free((list_T *)list); }
void nvim_tag_set_errorlist(void *list, const char *title) { set_errorlist(curwin, (list_T *)list, ' ', (char *)title, NULL); }
/// Set VV_SWAPCOMMAND to ":ta <name>\r" (Rust-formatted).
void nvim_tag_set_vim_var_swapcommand(const char *cmd) { set_vim_var_string(VV_SWAPCOMMAND, (char *)cmd, -1); }

/// Decrement RedrawingDisabled.
void nvim_tag_dec_RedrawingDisabled(void) { RedrawingDisabled--; }

/// set_topline(curwin, curwin->w_cursor.lnum) wrapper.
void nvim_tag_set_topline_curwin(void) { set_topline(curwin, curwin->w_cursor.lnum); }
/// win_close(curwin, false, false) wrapper for post_fail.
void nvim_tag_win_close_curwin(void) { win_close(curwin, false, false); }
char *nvim_tag_fm_getname(const void *tg_void, int lead_len) { const taggy_T *tg = (const taggy_T *)tg_void; return fm_getname(&((taggy_T *)tg)->fmark, lead_len); }

int nvim_tag_get_ptag_cur_match(void) { return ptag_entry.cur_match; }

/// Wrapper for find_tags callable from Rust
int nvim_tag_find_tags(char *pat, int *num_matches, char ***matchesp,
                       int flags, int mincount, char *buf_ffname)
{
  return find_tags(pat, num_matches, matchesp, flags, mincount, buf_ffname);
}

void nvim_tag_free_wild(int count, char **files) { FreeWild(count, files); }
char *nvim_tag_get_curbuf_ffname(void) { return curbuf->b_ffname; }
/// MB_PTR_ADV wrapper - advance pointer past one multi-byte char
const char *nvim_tag_mb_ptr_adv(const char *p)
{
  const char *result = p;
  MB_PTR_ADV(result);
  return result;
}

bool nvim_tag_get_tfu_in_use(void) { return tfu_in_use; }
void nvim_tag_set_tfu_in_use(bool val) { tfu_in_use = val; }
void *nvim_findtags_get_ga_match_ptr(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return (void *)st->ga_match; }
int *nvim_findtags_get_match_count_ptr(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return &st->match_count; }
void *nvim_tag_tv_dict_find_item(const void *dict, const char *key, int key_len) { return (void *)tv_dict_find((const dict_T *)dict, key, key_len); }
void *nvim_tag_dictitem_tv(void *di) { return (void *)&((dictitem_T *)di)->di_tv; }
bool nvim_tag_tv_is_list(const void *tv) { return ((const typval_T *)tv)->v_type == VAR_LIST; }
void *nvim_tag_tv_get_list(const void *tv) { return (void *)((const typval_T *)tv)->vval.v_list; }
int64_t nvim_tag_tv_get_number(const void *tv) { return (int64_t)tv_get_number((const typval_T *)tv); }
char *nvim_tag_tv_dict_get_string(const void *dict, const char *key, bool save) { return tv_dict_get_string((const dict_T *)dict, key, save); }
int64_t nvim_tag_tv_dict_get_number(const void *dict, const char *key) { return (int64_t)tv_dict_get_number((const dict_T *)dict, key); }
void *nvim_tag_tv_list_first(const void *list) { return (void *)tv_list_first((const list_T *)list); }
void *nvim_tag_tv_list_item_next(const void *list, const void *li) { return (void *)TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li); }
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

void nvim_tag_tv_list_append_number(void *list, int64_t nr) { tv_list_append_number((list_T *)list, (varnumber_T)nr); }
void nvim_tag_tv_dict_add_list(void *dict, const char *key, size_t key_len, void *list) { tv_dict_add_list((dict_T *)dict, key, key_len, (list_T *)list); }
int nvim_tag_taggy_fmark_coladd(const void *tg_void) { const taggy_T *tg = (const taggy_T *)tg_void; return tg->fmark.mark.coladd; }
void nvim_tag_callback_free_tfu(void) { callback_free(&tfu_cb); }
void nvim_tag_callback_free_buf_tfu(void *buf_void) { buf_T *buf = (buf_T *)buf_void; callback_free(&buf->b_tfu_cb); }
bool nvim_tag_buf_tfu_is_empty(const void *buf_void) { const buf_T *buf = (const buf_T *)buf_void; return *buf->b_p_tfu == NUL; }
int nvim_tag_option_set_tfu_callback(void *buf_void) { buf_T *buf = (buf_T *)buf_void; return option_set_callback_func(buf->b_p_tfu, &tfu_cb); }
void nvim_tag_callback_copy_tfu_to_buf(void *buf_void) { buf_T *buf = (buf_T *)buf_void; callback_copy(&buf->b_tfu_cb, &tfu_cb); }
bool nvim_tag_tfu_cb_is_none(void) { return tfu_cb.type == kCallbackNone; }
bool nvim_tag_set_ref_in_tfu_callback(int copyID) { return rs_set_ref_in_callback(&tfu_cb, copyID, NULL, NULL); }
void *nvim_tag_optset_get_buf(const void *args_void) { const optset_T *args = (const optset_T *)args_void; return (void *)args->os_buf; }
const char *nvim_tag_get_e_invarg(void) { return e_invarg; }

/// Returns g_tag_at_cursor.
bool nvim_tag_get_g_tag_at_cursor(void) { return g_tag_at_cursor; }

/// tv_dict_alloc_lock(VAR_FIXED) wrapper.
void *nvim_tag_dict_alloc_lock_fixed(void) { return (void *)tv_dict_alloc_lock(VAR_FIXED); }

/// Increment dict dv_refcount.
void nvim_tag_dict_refcount_inc(void *dict_void) { ((dict_T *)dict_void)->dv_refcount++; }
/// Decrement dict dv_refcount.
void nvim_tag_dict_refcount_dec(void *dict_void) { ((dict_T *)dict_void)->dv_refcount--; }

/// Set up the args and invoke the curbuf tagfunc callback.
/// - pat: the tag pattern (VAR_STRING arg 0)
/// - flag_str: the flag string (VAR_STRING arg 1)
/// - dict: the info dict (VAR_DICT arg 2), refcount already managed by caller
/// - rettv_storage: storage for typval_T result (must be zeroed, will be filled)
/// Returns FAIL (0) or OK (1) matching C callback_call return value.
int nvim_tag_do_callback_call_tfu(const char *pat, const char *flag_str,
                                   void *dict, void *rettv_storage)
{
  typval_T args[4];
  args[0].v_type = VAR_STRING;
  args[0].vval.v_string = (char *)pat;
  args[1].v_type = VAR_STRING;
  args[1].vval.v_string = (char *)flag_str;
  args[2].v_type = VAR_DICT;
  args[2].vval.v_dict = (dict_T *)dict;
  args[3].v_type = VAR_UNKNOWN;
  return callback_call(&curbuf->b_tfu_cb, 3, args, (typval_T *)rettv_storage);
}

/// Save curwin->w_cursor into the provided storage (a pos_T*).
void nvim_tag_save_cursor(void *pos_storage) { *(pos_T *)pos_storage = curwin->w_cursor; }

/// Restore curwin->w_cursor from storage and call check_cursor.
void nvim_tag_restore_cursor_check(void *pos_storage) { curwin->w_cursor = *(pos_T *)pos_storage; check_cursor(curwin); }
/// Returns true if rettv is VAR_SPECIAL with kSpecialVarNull.
bool nvim_tag_rettv_is_null_special(const void *rettv_storage)
{
  const typval_T *rettv = (const typval_T *)rettv_storage;
  return rettv->v_type == VAR_SPECIAL && rettv->vval.v_special == kSpecialVarNull;
}

/// Returns the list pointer from rettv (NULL if not a non-empty VAR_LIST).
void *nvim_tag_rettv_get_list(const void *rettv_storage)
{
  const typval_T *rettv = (const typval_T *)rettv_storage;
  if (rettv->v_type != VAR_LIST || !rettv->vval.v_list) {
    return NULL;
  }
  return (void *)rettv->vval.v_list;
}

/// Size in bytes of pos_T (for stack allocation in Rust).
size_t nvim_tag_pos_size(void) { return sizeof(pos_T); }

void nvim_tag_tv_clear_rettv(void *rettv_storage) { tv_clear((typval_T *)rettv_storage); }
size_t nvim_tag_rettv_size(void) { return sizeof(typval_T); }
bool nvim_tag_listitem_is_dict(const void *li) { const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li); return tv->v_type == VAR_DICT; }
/// Get the dict from a list item (returns dict handle, or NULL)
void *nvim_tag_listitem_get_dict(const void *li)
{
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  if (tv->v_type != VAR_DICT || !tv->vval.v_dict) {
    return NULL;
  }
  return (void *)tv->vval.v_dict;
}

/// Dict iteration API for Rust (hashitem-pointer approach).
/// Each item in the array is a hashitem_T; we return these as opaque void* so Rust
/// can hold them between calls. We also expose the dict's ht_array start and
/// ht_mask so Rust can do arithmetic to advance the pointer without C involvement.
/// However, to keep this simple and correct, we use a helper that advances past
/// tombstones (HASHITEM_EMPTY slots) to return the next live dictitem.

/// Return pointer to the first live hashitem_T in the dict, or NULL if empty.
void *nvim_tag_dict_iter_start(const void *dict_void)
{
  const dict_T *dict = (const dict_T *)dict_void;
  const hashtab_T *ht = &dict->dv_hashtab;
  size_t todo = ht->ht_used;
  for (hashitem_T *hi = ht->ht_array; todo; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      return (void *)hi;
    }
  }
  return NULL;
}

/// Given the current hashitem_T pointer, advance to the next live hashitem_T.
/// Returns NULL when there are no more items.
void *nvim_tag_dict_iter_next(const void *dict_void, const void *hi_void)
{
  const dict_T *dict = (const dict_T *)dict_void;
  const hashtab_T *ht = &dict->dv_hashtab;
  // Count remaining live items after current position.
  const hashitem_T *cur = (const hashitem_T *)hi_void;
  // Scan from the slot AFTER cur to find the next non-empty slot.
  // The array size is ht_mask + 1.
  size_t array_size = ht->ht_mask + 1;
  const hashitem_T *start = ht->ht_array;
  for (const hashitem_T *hi = cur + 1; hi < start + array_size; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      return (void *)hi;
    }
  }
  return NULL;
}

/// Return the key of the current hashitem (as dictitem_T's di_key).
const char *nvim_tag_dict_iter_key(const void *hi_void)
{
  const hashitem_T *hi = (const hashitem_T *)hi_void;
  return TV_DICT_HI2DI(hi)->di_key;
}

/// Return true if the current hashitem has a non-null string value.
bool nvim_tag_dict_iter_value_is_string(const void *hi_void)
{
  const hashitem_T *hi = (const hashitem_T *)hi_void;
  const dictitem_T *di = TV_DICT_HI2DI(hi);
  return di->di_tv.v_type == VAR_STRING && di->di_tv.vval.v_string != NULL;
}

/// Return the string value of the current hashitem, or NULL if not a string.
const char *nvim_tag_dict_iter_value_string(const void *hi_void)
{
  const hashitem_T *hi = (const hashitem_T *)hi_void;
  const dictitem_T *di = TV_DICT_HI2DI(hi);
  if (di->di_tv.v_type != VAR_STRING) {
    return NULL;
  }
  return di->di_tv.vval.v_string;
}

/// Grow a garray_T by 1 and append a string pointer
void nvim_tag_ga_grow_append(void *ga_void, char *mfp)
{
  garray_T *ga = (garray_T *)ga_void;
  ga_grow(ga, 1);
  ((char **)(ga->ga_data))[ga->ga_len++] = mfp;
}

_Static_assert(kOptSwbFlagUseopen == 0x01, "kOptSwbFlagUseopen value for Rust");
_Static_assert(kOptSwbFlagUsetab == 0x02, "kOptSwbFlagUsetab value for Rust");

/// Increment RedrawingDisabled.
void nvim_tag_inc_RedrawingDisabled(void) { RedrawingDisabled++; }

/// Returns curwin->w_p_pvw.
bool nvim_tag_curwin_pvw(void) { return curwin->w_p_pvw; }

/// FullName_save(fname, false) wrapper.
char *nvim_tag_fullname_save(char *fname) { return FullName_save(fname, false); }

/// prepare_tagpreview(true) wrapper.
void nvim_tag_prepare_tagpreview(void) { prepare_tagpreview(true); }

/// Returns true if swb_flags has useopen or usetab set.
bool nvim_tag_swb_has_useopen_or_usetab(void) { return (swb_flags & (kOptSwbFlagUseopen | kOptSwbFlagUsetab)) != 0; }

/// buflist_findname_exp wrapper.
void *nvim_tag_buflist_findname_exp(char *fname) { return (void *)buflist_findname_exp(fname); }

/// swbuf_goto_win_with_buf wrapper. Returns true if a window was found.
bool nvim_tag_swbuf_goto_win_with_buf(void *buf) { return swbuf_goto_win_with_buf((buf_T *)buf) != NULL; }

/// win_split(size, flags) wrapper.
int nvim_tag_win_split(int size, int flags) { return win_split(size, flags); }

/// Returns postponed_split_flags.
int nvim_tag_get_postponed_split_flags(void) { return postponed_split_flags; }

/// RESET_BINDING(curwin) wrapper.
void nvim_tag_reset_binding_curwin(void) { RESET_BINDING(curwin); }

/// Sets keep_help_flag.
void nvim_tag_set_keep_help_flag(bool val) { keep_help_flag = val; }

/// bt_help(((win_T*)win)->w_buffer) wrapper.
bool nvim_tag_bt_help_saved_win(const void *win) { return bt_help(((const win_T *)win)->w_buffer); }

/// getfile(0, fname, NULL, true, 0, forceit) wrapper.
int nvim_tag_getfile_call(char *fname, int forceit) { return getfile(0, fname, NULL, true, 0, forceit); }

/// Returns cmdmod.cmod_tab.
int nvim_tag_get_cmdmod_tab(void) { return cmdmod.cmod_tab; }

/// Returns true if *curbuf->b_p_tfu == NUL (tagfunc option is empty).
bool nvim_tag_curbuf_b_p_tfu_is_empty(void) { return *curbuf->b_p_tfu == NUL; }

/// Returns true if curbuf->b_tfu_cb.type == kCallbackNone.
bool nvim_tag_curbuf_tfu_cb_is_none(void) { return curbuf->b_tfu_cb.type == kCallbackNone; }

/// Set curwin->w_set_curswant = val.
void nvim_tag_set_curswant(bool val) { curwin->w_set_curswant = val; }

/// Get magic_overruled as int.
int nvim_tag_get_magic_overruled(void) { return (int)magic_overruled; }
/// Set magic_overruled from int.
void nvim_tag_set_magic_overruled(int val) { magic_overruled = (optmagic_T)val; }

/// Get no_hlsearch flag.
bool nvim_tag_get_no_hlsearch(void) { return no_hlsearch; }
/// Call set_no_hlsearch(val).
void nvim_tag_set_no_hlsearch_val(bool val) { set_no_hlsearch(val); }

/// Returns true if CPO_TAGPAT is in p_cpo.
bool nvim_tag_cpo_has_tagpat(void) { return vim_strchr(p_cpo, CPO_TAGPAT) != NULL; }

/// Get p_ws.
bool nvim_tag_get_p_ws(void) { return p_ws; }
/// Set p_ws.
void nvim_tag_set_p_ws(bool val) { p_ws = val; }

/// Get p_ic (already have nvim_set_p_ic).
int nvim_tag_get_p_ic(void) { return p_ic; }

/// Get p_scs.
int nvim_tag_get_p_scs(void) { return p_scs; }
/// Set p_scs.
void nvim_tag_set_p_scs(int val) { p_scs = val; }

/// Get curwin->w_cursor.lnum.
linenr_T nvim_tag_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }
/// Set curwin->w_cursor.lnum.
void nvim_tag_set_cursor_lnum(linenr_T val) { curwin->w_cursor.lnum = val; }
/// Set cursor to lnum=1, col=0, coladd=0.
void nvim_tag_set_cursor_start(void) { curwin->w_cursor.lnum = 1; curwin->w_cursor.col = 0; curwin->w_cursor.coladd = 0; }

/// Get secure.
int nvim_tag_get_secure(void) { return secure; }
/// Set secure.
void nvim_tag_set_secure(int val) { secure = val; }
/// Increment sandbox.
void nvim_tag_inc_sandbox(void) { sandbox++; }
/// Decrement sandbox.
void nvim_tag_dec_sandbox(void) { sandbox--; }

/// skip_regexp(p, delim, false) wrapper.
char *nvim_tag_skip_regexp(char *p, int delim) { return skip_regexp(p, delim, false); }

/// do_search wrapper for tag search: do_search(NULL, dir, dir, pat, patlen, 1, options, NULL).
bool nvim_tag_do_search(int dir, char *pat, size_t patlen, int options)
{
  return do_search(NULL, dir, dir, pat, patlen, 1, options, NULL);
}

/// do_cmdline_cmd wrapper.
void nvim_tag_do_cmdline_cmd(char *cmd) { do_cmdline_cmd(cmd); }

/// wait_return(true) wrapper.
void nvim_tag_wait_return(void) { wait_return(true); }

/// check_cursor(curwin) wrapper.
void nvim_tag_check_cursor(void) { check_cursor(curwin); }

bool nvim_tag_get_p_tgst(void) { return p_tgst; }
int nvim_tag_get_curbuf_fnum(void) { return curbuf->b_fnum; }
bool nvim_tag_get_got_int(void) { return got_int; }
int nvim_tag_get_msg_scroll(void) { return msg_scroll; }
void nvim_tag_set_msg_scroll(int val) { msg_scroll = val; }
int nvim_tag_get_msg_scrolled(void) { return msg_scrolled; }
int nvim_tag_get_msg_silent(void) { return msg_silent; }
bool nvim_tag_ui_has_messages(void) { return ui_has(kUIMessages); }
void nvim_tag_ui_flush(void) { ui_flush(); }
void nvim_tag_os_delay(int msec) { os_delay(msec, true); }
char *nvim_tag_buflist_findnr_ffname(int fnum) { buf_T *buf = buflist_findnr(fnum); return buf != NULL ? buf->b_ffname : NULL; }
/// buflist_getfile wrapper that returns the result (OK/FAIL).
/// Used from Rust for DT_POP jump to different buffer.
int nvim_tag_buflist_getfile_with_result(int fnum, linenr_T lnum, int flags, int forceit)
{
  return buflist_getfile(fnum, lnum, flags, forceit);
}

/// give_warning wrapper for Rust.
void nvim_tag_give_warning(const char *msg_str, bool ic) { give_warning(msg_str, ic); }

/// Get the KeyTyped global (bool).
bool nvim_tag_get_KeyTyped(void) { return KeyTyped; }

bool nvim_tag_tagstack_changed(void *saved_tagstack) { return saved_tagstack != curwin->w_tagstack; }
void *nvim_tag_get_tagstack_ptr(void) { return curwin->w_tagstack; }
/// Save cursor position in tagstack entry
void nvim_tag_save_cursor_in_entry(void *tg_void, int idx)
{
  taggy_T *tg = (taggy_T *)tg_void;
  tg[idx].fmark.mark = curwin->w_cursor;
  tg[idx].fmark.fnum = curbuf->b_fnum;
}

void nvim_tag_copy_fmark_from_entry(void *tg_void, int idx, void *out_buf) { taggy_T *tg = (taggy_T *)tg_void; memcpy(out_buf, &tg[idx].fmark, sizeof(fmark_T)); }
void nvim_tag_restore_fmark_to_entry(void *tg_void, int idx, const void *buf) { taggy_T *tg = (taggy_T *)tg_void; memcpy(&tg[idx].fmark, buf, sizeof(fmark_T)); }
int nvim_tag_prompt_for_selection(void) { return prompt_for_input(NULL, 0, false, NULL); }
void nvim_tag_clear_swap_command(void) { set_vim_var_string(VV_SWAPCOMMAND, NULL, -1); }
/// Format "tag N of M[or more]" into caller-provided buffer.
void nvim_tag_snprintf_match_msg(char *buf, int buf_size, int cur_match, int num_matches, int max_num_matches)
{
  snprintf(buf, (size_t)buf_size, _("tag %d of %d%s"),
           cur_match + 1, num_matches,
           max_num_matches != MAXCOL ? _(" or more") : "");
}
/// Append IC warning to caller-provided buffer.
void nvim_tag_append_ic_warning_to_buf(char *buf, int buf_size)
{
  xstrlcat(buf, _("  Using tag with different case!"), (size_t)buf_size);
}

void nvim_tag_free_nofile_fname(void) { free_string_option(nofile_fname); nofile_fname = NULL; }
bool nvim_tag_nofile_fname_is_null(void) { return nofile_fname == NULL; }

static garray_T tag_fnames = GA_EMPTY_INIT_VALUE;

int nvim_tag_fnames_len(void) { return tag_fnames.ga_len; }
/// Get a tag file name from the help file list by index
const char *nvim_tag_fnames_get(int idx) { return (idx >= 0 && idx < tag_fnames.ga_len) ? ((char **)(tag_fnames.ga_data))[idx] : NULL; }
void nvim_tag_fnames_clear(void) { ga_clear_strings(&tag_fnames); }
void nvim_tag_fnames_init(void) { ga_init(&tag_fnames, (int)sizeof(char *), 10); }
void nvim_tag_fnames_add(char *fname) { GA_APPEND(char *, &tag_fnames, fname); }
void nvim_do_in_runtimepath_for_tags(void) { do_in_runtimepath("doc/tags doc/tags-??", DIP_ALL, rs_found_tagfile_cb, NULL); }
