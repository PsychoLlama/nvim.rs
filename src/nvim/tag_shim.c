// tag_shim.c: Rust FFI accessors for tag crate.

#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cursor.h"
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
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/help.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/window.h"

typedef struct {
  char *pat;            // the pattern
  int len;              // length of pat[]
  char *head;           // start of pattern head
  int headlen;          // length of head[]
  regmatch_T regmatch;  // regexp program, may be NULL
} pat_T;

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

#define NOTAGFILE       99              // return value for jumpto_tag

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
  off_T low_offset;        ///< offset for first char of first line that could match
  off_T high_offset;       ///< offset of char after last line that could match
  off_T curr_offset;       ///< Current file offset in search range
  off_T curr_offset_used;  ///< curr_offset used when skipping back
  off_T match_offset;      ///< Where the binary search found a tag
  int low_char;            ///< first char at low_offset
  int high_char;           ///< first char at high_offset
} tagsearch_info_T;

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
  int mincount;                  ///< MAXCOL: find all matches, other: minimal number of matches
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
void nvim_findtags_alloc_orgpat(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat = xmalloc(sizeof(pat_T)); }
void nvim_findtags_clear_orgpat_regprog(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->regmatch.regprog = NULL; }
void nvim_findtags_set_flags(void *st_void, int flags) { findtags_state_T *st = (findtags_state_T *)st_void; st->flags = flags; }
void nvim_findtags_set_help_only_from_flags(void *st_void, int flags) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_only = (flags & TAG_HELP) != 0; }
void nvim_findtags_set_mincount(void *st_void, int mincount) { findtags_state_T *st = (findtags_state_T *)st_void; st->mincount = mincount; }
void nvim_findtags_alloc_lbuf(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->lbuf_size = LSIZE; st->lbuf = xmalloc((size_t)st->lbuf_size); }
void nvim_findtags_free_tag_fname(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; xfree(st->tag_fname); }
void nvim_findtags_free_lbuf(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; xfree(st->lbuf); }
void nvim_findtags_free_orgpat_regprog(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; vim_regfree(st->orgpat->regmatch.regprog); }
void nvim_findtags_free_orgpat(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; xfree(st->orgpat); }
void *nvim_findtags_state_xcalloc(void) { return xcalloc(1, sizeof(findtags_state_T)); }
void nvim_findtags_init_match_arrays(void *st_void)
{ findtags_state_T *st = (findtags_state_T *)st_void; for (int mtt = 0; mtt < MT_COUNT; mtt++) { ga_init(&st->ga_match[mtt], sizeof(char *), 100); hash_init(&st->ht_match[mtt]); } }
char *nvim_findtags_get_tag_fname_buf(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return st->tag_fname; }
bool nvim_curbuf_is_help(void) { return curbuf->b_help; }
const char *nvim_get_p_hf(void) { return p_hf; }
const char *nvim_get_curbuf_tags(void) { return curbuf->b_p_tags; }
const char *nvim_get_p_tags(void) { return p_tags; }
extern bool rs_has_autocmd(int event, const char *sfname, int buf_fnum);
bool nvim_has_bufreadcmd(const char *fname) { return rs_has_autocmd(EVENT_BUFREADCMD, fname, 0); }
bool nvim_check_can_set_curbuf_forceit(int forceit) { return check_can_set_curbuf_forceit(forceit); }
const char *nvim_get_curbuf_ffname(void) { return curbuf->b_ffname; }
bool nvim_ignorecase(const char *pat) { return ignorecase((char *)pat); }
char *nvim_path_tail(char *path) { return path_tail(path); }
bool nvim_path_has_wildcard(const char *fname) { return path_has_wildcard(fname); }
void nvim_vim_findfile_cleanup(void *search_ctx) { vim_findfile_cleanup(search_ctx); }
int nvim_get_postponed_split(void) { return postponed_split; }
void nvim_set_postponed_split(int val) { postponed_split = val; }
int nvim_get_g_do_tagpreview(void) { return g_do_tagpreview; }
void nvim_set_g_do_tagpreview(int val) { g_do_tagpreview = val; }
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern void rs_prepare_pats(pat_T *pats, bool has_re);
extern bool rs_found_tagfile_cb(int num_fnames, char **fnames, bool all, void *cookie);

#include "tag_shim.c.generated.h"

static taggy_T ptag_entry = { NULL, INIT_FMARK, 0, 0, NULL };
static Callback tfu_cb;          // 'tagfunc' callback function

void *nvim_get_ptag_entry(void) { return &ptag_entry; }
bool nvim_tag_curwin_is_null(void) { return curwin == NULL; }
char *nvim_expand_one_file(char *fname)
{ expand_T xpc; ExpandInit(&xpc); xpc.xp_context = EXPAND_FILES; return ExpandOne(&xpc, fname, NULL, WILD_LIST_NOTFOUND|WILD_SILENT, WILD_EXPAND_FREE); }
bool nvim_get_p_tr(void) { return p_tr; }
void nvim_findtags_set_state_val(void *st_void, int state) { findtags_state_T *st = (findtags_state_T *)st_void; st->state = (tagsearch_state_T)state; }
char *nvim_findtags_get_lbuf(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->lbuf; }
int nvim_findtags_get_lbuf_size(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->lbuf_size; }
void nvim_findtags_set_lbuf(void *st_void, char *lbuf, int lbuf_size) { findtags_state_T *st = (findtags_state_T *)st_void; st->lbuf = lbuf; st->lbuf_size = lbuf_size; }
bool nvim_findtags_fgets(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return vim_fgets(st->lbuf, st->lbuf_size, st->fp); }
int nvim_findtags_fseek(void *st_void, int64_t offset, int whence) { findtags_state_T *st = (findtags_state_T *)st_void; return vim_fseek(st->fp, (off_T)offset, whence); }
int64_t nvim_findtags_ftell(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return (int64_t)vim_ftell(st->fp); }
void nvim_findtags_fseek_zero(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; fseek(st->fp, 0, SEEK_SET); }
bool nvim_findtags_lbuf_is_blank(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return vim_isblankline(st->lbuf); }
int nvim_findtags_get_flags(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->flags; }
void nvim_findtags_set_linear(void *st_void, bool linear) { findtags_state_T *st = (findtags_state_T *)st_void; st->linear = linear; }
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
_Static_assert(sizeof(hash_T) == sizeof(size_t), "hash_T must be size_t");
bool nvim_findtags_add_match_entry(void *st_void, int mtt, char *mfp, hash_T *hash)
{ findtags_state_T *st = (findtags_state_T *)st_void; *hash = hash_hash(mfp); hashitem_T *hi = hash_lookup(&st->ht_match[mtt], mfp, strlen(mfp), *hash); if (HASHITEM_EMPTY(hi)) { hash_add_item(&st->ht_match[mtt], hi, mfp, *hash); GA_APPEND(char *, &st->ga_match[mtt], mfp); st->match_count++; return true; } return false; }
int nvim_findtags_ga_match_len(const void *st_void, int mtt) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->ga_match[mtt].ga_len; }
char *nvim_findtags_ga_match_get(const void *st_void, int mtt, int idx) { const findtags_state_T *st = (const findtags_state_T *)st_void; return ((char **)(st->ga_match[mtt].ga_data))[idx]; }
void nvim_findtags_clear_match(void *st_void, int mtt) { findtags_state_T *st = (findtags_state_T *)st_void; ga_clear(&st->ga_match[mtt]); hash_clear(&st->ht_match[mtt]); }
bool nvim_findtags_get_stop_searching(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->stop_searching; }
void nvim_findtags_set_stop_searching(void *st_void, bool val) { findtags_state_T *st = (findtags_state_T *)st_void; st->stop_searching = val; }
bool nvim_findtags_get_is_txt(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->is_txt; }
void nvim_findtags_set_is_txt(void *st_void, bool val) { findtags_state_T *st = (findtags_state_T *)st_void; st->is_txt = val; }
const char *nvim_findtags_get_help_lang_find(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->help_lang_find; }
void nvim_findtags_set_help_lang_find(void *st_void, const char *val) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_lang_find = (char *)val; }
void nvim_findtags_set_help_pri(void *st_void, int pri) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_pri = pri; }
void nvim_findtags_set_help_lang(void *st_void, const char *lang) { findtags_state_T *st = (findtags_state_T *)st_void; st->help_lang[0] = lang[0]; st->help_lang[1] = lang[1]; st->help_lang[2] = NUL; }
int nvim_findtags_get_vimconv_type(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return (int)st->vimconv.vc_type; }
void nvim_findtags_set_vimconv_none(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->vimconv.vc_type = CONV_NONE; }
void nvim_findtags_convert_cleanup(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; convert_setup(&st->vimconv, NULL, NULL); }
bool nvim_findtags_fopen(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->fp = os_fopen(st->tag_fname, "r"); return st->fp != NULL; }
void nvim_findtags_fclose(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; if (st->fp != NULL) { fclose(st->fp); st->fp = NULL; } }
void nvim_findtags_set_did_open(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->did_open = true; }
bool nvim_findtags_get_did_open(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->did_open; }
void nvim_findtags_set_state_start(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; st->state = TS_START; }
int nvim_findtags_get_mincount(const void *st_void) { const findtags_state_T *st = (const findtags_state_T *)st_void; return st->mincount; }
void nvim_findtags_set_orgpat_len(void *st_void, int len) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->len = len; }
void nvim_findtags_set_orgpat_pat(void *st_void, char *pat) { findtags_state_T *st = (findtags_state_T *)st_void; st->orgpat->pat = pat; }
void nvim_set_p_ic(int val) { p_ic = val; }
bool nvim_get_p_tbs(void) { return p_tbs; }
int nvim_get_tc_flags(void) { return (int)tc_flags; }
int nvim_get_curbuf_tc_flags(void) { return (int)curbuf->b_tc_flags; }
const char *nvim_get_p_hlg(void) { return p_hlg; }
const char *nvim_get_curbuf_b_fname(void) { return curbuf->b_fname; }
const char *nvim_get_curbuf_p_tfu(void) { return curbuf->b_p_tfu; }
void nvim_set_curbuf_b_help(int val) { curbuf->b_help = val; }
int nvim_get_curbuf_b_help(void) { return curbuf->b_help; }
void nvim_findtags_prepare_pats(void *st_void, bool has_re) { findtags_state_T *st = (findtags_state_T *)st_void; rs_prepare_pats(st->orgpat, has_re); }
void *nvim_tag_get_curwin(void) { return (void *)curwin; }
bool nvim_tag_tv_dict_find(void *dict, const char *key, int key_len) { return tv_dict_find((dict_T *)dict, key, key_len) != NULL; }
int nvim_tag_tv_dict_add_nr(void *dict, const char *key, size_t key_len, int64_t nr) { return tv_dict_add_nr((dict_T *)dict, key, key_len, (varnumber_T)nr); }
void nvim_tag_set_errorlist(void *list, const char *title) { set_errorlist(curwin, (list_T *)list, ' ', (char *)title, NULL); }
void nvim_tag_dec_RedrawingDisabled(void) { RedrawingDisabled--; }
void nvim_tag_set_topline_curwin(void) { set_topline(curwin, curwin->w_cursor.lnum); }
void nvim_tag_win_close_curwin(void) { win_close(curwin, false, false); }
char *nvim_tag_fm_getname(const void *tg_void, int lead_len) { const taggy_T *tg = (const taggy_T *)tg_void; return fm_getname(&((taggy_T *)tg)->fmark, lead_len); }
int nvim_tag_get_ptag_cur_match(void) { return ptag_entry.cur_match; }
char *nvim_tag_get_curbuf_ffname(void) { return curbuf->b_ffname; }
const char *nvim_tag_mb_ptr_adv(const char *p) { const char *result = p; MB_PTR_ADV(result); return result; }
void *nvim_findtags_get_ga_match_ptr(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return (void *)st->ga_match; }
int *nvim_findtags_get_match_count_ptr(void *st_void) { findtags_state_T *st = (findtags_state_T *)st_void; return &st->match_count; }
void *nvim_tag_tv_dict_find_item(const void *dict, const char *key, int key_len) { return (void *)tv_dict_find((const dict_T *)dict, key, key_len); }
void *nvim_tag_dictitem_tv(void *di) { return (void *)&((dictitem_T *)di)->di_tv; }
bool nvim_tag_tv_is_list(const void *tv) { return ((const typval_T *)tv)->v_type == VAR_LIST; }
void *nvim_tag_tv_get_list(const void *tv) { return (void *)((const typval_T *)tv)->vval.v_list; }
void *nvim_tag_tv_list_first(const void *list) { return (void *)tv_list_first((const list_T *)list); }
void *nvim_tag_tv_list_item_next(const void *list, const void *li) { return (void *)TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li); }
int nvim_tag_list2fpos(void *tv, int32_t *lnum, int32_t *col, int32_t *coladd, int *fnum)
{ pos_T pos; int r = list2fpos((typval_T *)tv, &pos, fnum, NULL, false); if (r == OK) { *lnum = pos.lnum; *col = pos.col; *coladd = pos.coladd; } return r; }
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
bool nvim_tag_get_g_tag_at_cursor(void) { return g_tag_at_cursor; }
void nvim_tag_dict_refcount_inc(void *dict_void) { ((dict_T *)dict_void)->dv_refcount++; }
void nvim_tag_dict_refcount_dec(void *dict_void) { ((dict_T *)dict_void)->dv_refcount--; }
int nvim_tag_do_callback_call_tfu(const char *pat, const char *flag_str, void *dict, void *rettv_storage)
{
  typval_T args[4] = {
    [0] = { .v_type = VAR_STRING, .vval.v_string = (char *)pat },
    [1] = { .v_type = VAR_STRING, .vval.v_string = (char *)flag_str },
    [2] = { .v_type = VAR_DICT, .vval.v_dict = (dict_T *)dict },
    [3] = { .v_type = VAR_UNKNOWN },
  };
  return callback_call(&curbuf->b_tfu_cb, 3, args, (typval_T *)rettv_storage);
}
void nvim_tag_save_cursor(void *pos_storage) { *(pos_T *)pos_storage = curwin->w_cursor; }
void nvim_tag_restore_cursor_check(void *pos_storage) { curwin->w_cursor = *(pos_T *)pos_storage; check_cursor(curwin); }
bool nvim_tag_rettv_is_null_special(const void *rettv_storage) { const typval_T *rettv = (const typval_T *)rettv_storage; return rettv->v_type == VAR_SPECIAL && rettv->vval.v_special == kSpecialVarNull; }
void *nvim_tag_rettv_get_list(const void *rettv_storage) { const typval_T *rettv = (const typval_T *)rettv_storage; return (rettv->v_type == VAR_LIST && rettv->vval.v_list) ? (void *)rettv->vval.v_list : NULL; }
size_t nvim_tag_pos_size(void) { return sizeof(pos_T); }
size_t nvim_tag_rettv_size(void) { return sizeof(typval_T); }
bool nvim_tag_listitem_is_dict(const void *li) { const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li); return tv->v_type == VAR_DICT; }
void *nvim_tag_listitem_get_dict(const void *li) { const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li); return (tv->v_type == VAR_DICT && tv->vval.v_dict) ? (void *)tv->vval.v_dict : NULL; }
void *nvim_tag_dict_iter_start(const void *dict_void)
{ const hashtab_T *ht = &((const dict_T *)dict_void)->dv_hashtab; for (hashitem_T *hi = ht->ht_array; ht->ht_used && hi < ht->ht_array + ht->ht_mask + 1; hi++) { if (!HASHITEM_EMPTY(hi)) { return (void *)hi; } } return NULL; }
void *nvim_tag_dict_iter_next(const void *dict_void, const void *hi_void)
{ const hashtab_T *ht = &((const dict_T *)dict_void)->dv_hashtab; const hashitem_T *end = ht->ht_array + ht->ht_mask + 1; for (const hashitem_T *hi = (const hashitem_T *)hi_void + 1; hi < end; hi++) { if (!HASHITEM_EMPTY(hi)) { return (void *)hi; } } return NULL; }
const char *nvim_tag_dict_iter_key(const void *hi_void) { return TV_DICT_HI2DI((const hashitem_T *)hi_void)->di_key; }
bool nvim_tag_dict_iter_value_is_string(const void *hi_void) { const dictitem_T *di = TV_DICT_HI2DI((const hashitem_T *)hi_void); return di->di_tv.v_type == VAR_STRING && di->di_tv.vval.v_string != NULL; }
const char *nvim_tag_dict_iter_value_string(const void *hi_void) { const dictitem_T *di = TV_DICT_HI2DI((const hashitem_T *)hi_void); return di->di_tv.v_type == VAR_STRING ? di->di_tv.vval.v_string : NULL; }
void nvim_tag_ga_grow_append(void *ga_void, char *mfp) { garray_T *ga = (garray_T *)ga_void; ga_grow(ga, 1); ((char **)(ga->ga_data))[ga->ga_len++] = mfp; }
_Static_assert(kOptSwbFlagUseopen == 0x01, "kOptSwbFlagUseopen value for Rust");
_Static_assert(kOptSwbFlagUsetab == 0x02, "kOptSwbFlagUsetab value for Rust");
void nvim_tag_inc_RedrawingDisabled(void) { RedrawingDisabled++; }
bool nvim_tag_curwin_pvw(void) { return curwin->w_p_pvw; }
char *nvim_tag_fullname_save(char *fname) { return FullName_save(fname, false); }
bool nvim_tag_swb_has_useopen_or_usetab(void) { return (swb_flags & (kOptSwbFlagUseopen | kOptSwbFlagUsetab)) != 0; }
void *nvim_tag_buflist_findname_exp(char *fname) { return (void *)buflist_findname_exp(fname); }
bool nvim_tag_swbuf_goto_win_with_buf(void *buf) { return swbuf_goto_win_with_buf((buf_T *)buf) != NULL; }
int nvim_tag_get_postponed_split_flags(void) { return postponed_split_flags; }
void nvim_tag_reset_binding_curwin(void) { RESET_BINDING(curwin); }
void nvim_tag_set_keep_help_flag(bool val) { keep_help_flag = val; }
bool nvim_tag_bt_help_saved_win(const void *win) { return bt_help(((const win_T *)win)->w_buffer); }
int nvim_tag_get_cmdmod_tab(void) { return cmdmod.cmod_tab; }
bool nvim_tag_curbuf_b_p_tfu_is_empty(void) { return *curbuf->b_p_tfu == NUL; }
bool nvim_tag_curbuf_tfu_cb_is_none(void) { return curbuf->b_tfu_cb.type == kCallbackNone; }
void nvim_tag_set_curswant(bool val) { curwin->w_set_curswant = val; }
int nvim_tag_get_magic_overruled(void) { return (int)magic_overruled; }
void nvim_tag_set_magic_overruled(int val) { magic_overruled = (optmagic_T)val; }
bool nvim_tag_get_no_hlsearch(void) { return no_hlsearch; }
void nvim_tag_set_no_hlsearch_val(bool val) { set_no_hlsearch(val); }
bool nvim_tag_cpo_has_tagpat(void) { return vim_strchr(p_cpo, CPO_TAGPAT) != NULL; }
bool nvim_tag_get_p_ws(void) { return p_ws; }
void nvim_tag_set_p_ws(bool val) { p_ws = val; }
int nvim_tag_get_p_ic(void) { return p_ic; }
int nvim_tag_get_p_scs(void) { return p_scs; }
void nvim_tag_set_p_scs(int val) { p_scs = val; }
linenr_T nvim_tag_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }
void nvim_tag_set_cursor_lnum(linenr_T val) { curwin->w_cursor.lnum = val; }
void nvim_tag_set_cursor_start(void) { curwin->w_cursor.lnum = 1; curwin->w_cursor.col = 0; curwin->w_cursor.coladd = 0; }
int nvim_tag_get_secure(void) { return secure; }
void nvim_tag_set_secure(int val) { secure = val; }
void nvim_tag_inc_sandbox(void) { sandbox++; }
void nvim_tag_dec_sandbox(void) { sandbox--; }
char *nvim_tag_skip_regexp(char *p, int delim) { return skip_regexp(p, delim, false); }
void nvim_tag_check_cursor(void) { check_cursor(curwin); }
bool nvim_tag_get_p_tgst(void) { return p_tgst; }
int nvim_tag_get_curbuf_fnum(void) { return curbuf->b_fnum; }
bool nvim_tag_get_got_int(void) { return got_int; }
void nvim_tag_set_msg_scroll(int val) { msg_scroll = val; }
int nvim_tag_get_msg_scrolled(void) { return msg_scrolled; }
int nvim_tag_get_msg_silent(void) { return msg_silent; }
char *nvim_tag_buflist_findnr_ffname(int fnum) { buf_T *buf = buflist_findnr(fnum); return buf != NULL ? buf->b_ffname : NULL; }
void nvim_tag_give_warning(const char *msg_str, bool ic) { give_warning(msg_str, ic); }
bool nvim_tag_get_KeyTyped(void) { return KeyTyped; }
bool nvim_tag_tagstack_changed(void *saved_tagstack) { return saved_tagstack != curwin->w_tagstack; }
void *nvim_tag_get_tagstack_ptr(void) { return curwin->w_tagstack; }
void nvim_tag_save_cursor_in_entry(void *tg_void, int idx) { taggy_T *tg = (taggy_T *)tg_void; tg[idx].fmark.mark = curwin->w_cursor; tg[idx].fmark.fnum = curbuf->b_fnum; }
void nvim_tag_copy_fmark_from_entry(void *tg_void, int idx, void *out_buf) { taggy_T *tg = (taggy_T *)tg_void; memcpy(out_buf, &tg[idx].fmark, sizeof(fmark_T)); }
void nvim_tag_restore_fmark_to_entry(void *tg_void, int idx, const void *buf) { taggy_T *tg = (taggy_T *)tg_void; memcpy(&tg[idx].fmark, buf, sizeof(fmark_T)); }
void nvim_tag_clear_swap_command(void) { set_vim_var_string(VV_SWAPCOMMAND, NULL, -1); }
void nvim_do_in_runtimepath_for_tags(void) { do_in_runtimepath("doc/tags doc/tags-??", DIP_ALL, rs_found_tagfile_cb, NULL); }
