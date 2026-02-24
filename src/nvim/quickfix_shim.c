// quickfix.c: functions for quickfix mode, using a file with error messages

#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/fuzzy.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/help.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

struct dir_stack_T {
  struct dir_stack_T *next;
  char *dirname;
};

// For each error the next struct is allocated and linked in a list.
typedef struct qfline_S qfline_T;
struct qfline_S {
  qfline_T *qf_next;      ///< pointer to next error in the list
  qfline_T *qf_prev;      ///< pointer to previous error in the list
  linenr_T qf_lnum;       ///< line number where the error occurred
  linenr_T qf_end_lnum;   ///< line number when the error has range or zero

  int qf_fnum;            ///< file number for the line
  int qf_col;             ///< column where the error occurred
  int qf_end_col;         ///< column when the error has range or zero
  int qf_nr;              ///< error number
  char *qf_module;        ///< module name for this error
  char *qf_fname;         ///< different filename if there're hard links
  char *qf_pattern;       ///< search pattern for the error
  char *qf_text;          ///< description of the error
  char qf_viscol;         ///< set to true if qf_col and qf_end_col is screen column
  char qf_cleared;        ///< set to true if line has been deleted
  char qf_type;           ///< type of the error (mostly 'E'); 1 for :helpgrep
  typval_T qf_user_data;  ///< custom user data associated with this item
  char qf_valid;          ///< valid error message detected
};

// There is a stack of error lists.
#define INVALID_QFIDX (-1)
#define INVALID_QFBUFNR (0)

/// Quickfix list type.
typedef enum {
  QFLT_QUICKFIX,  ///< Quickfix list - global list
  QFLT_LOCATION,  ///< Location list - per window list
  QFLT_INTERNAL,  ///< Internal - Temporary list used by
  //   getqflist()/getloclist()
} qfltype_T;

/// information and entries can be added later using setqflist()/setloclist().
typedef struct {
  unsigned qf_id;         ///< Unique identifier for this list
  qfltype_T qfl_type;
  qfline_T *qf_start;     ///< pointer to the first error
  qfline_T *qf_last;      ///< pointer to the last error
  qfline_T *qf_ptr;       ///< pointer to the current error
  int qf_count;           ///< number of errors (0 means empty list)
  int qf_index;           ///< current index in the error list
  bool qf_nonevalid;      ///< true if not a single valid entry found
  bool qf_has_user_data;  ///< true if at least one item has user_data attached
  char *qf_title;         ///< title derived from the command that created
                          ///< the error list or set by setqflist
  typval_T *qf_ctx;       ///< context set by setqflist/setloclist
  Callback qf_qftf_cb;    ///< 'quickfixtextfunc' callback function

  struct dir_stack_T *qf_dir_stack;
  char *qf_directory;
  struct dir_stack_T *qf_file_stack;
  char *qf_currfile;
  bool qf_multiline;
  bool qf_multiignore;
  bool qf_multiscan;
  int qf_changedtick;
} qf_list_T;

/// Contains a list of quickfix/location lists (qf_list_T)
struct qf_info_S {
  // Count of references to this list. Used only for location lists.
  // When a location list window reference this list, qf_refcount
  // will be 2. Otherwise, qf_refcount will be 1. When qf_refcount
  // reaches 0, the list is freed.
  int qf_refcount;
  int qf_listcount;                 // current number of lists
  int qf_curlist;                   // current error list
  int qf_maxcount;                  // maximum number of lists
  qf_list_T *qf_lists;
  qfltype_T qfl_type;  // type of list
  int qf_bufnr;                     // quickfix window buffer number
};

static qf_info_T ql_info_actual;  // global quickfix list
static qf_info_T *ql_info;        // points to ql_info_actual after allocation
static unsigned last_qf_id = 0;   // Last Used quickfix list id

extern const char *rs_skip_to_option_part(const char *p);
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern bool rs_set_ref_in_item(typval_T *tv, int copyID, ht_stack_T **ht_stack,
                               list_stack_T **list_stack);
extern bool rs_set_ref_in_callback(Callback *callback, int copyID, ht_stack_T **ht_stack,
                                   list_stack_T **list_stack);
extern bool rs_qf_stack_empty(const void *qi);
extern bool rs_qf_list_empty(const void *qfl);
extern bool rs_qflist_valid(const void *wp, unsigned qf_id);
extern void rs_qf_msg(const void *qi, int which, const char *lead);
extern int rs_qf_getprop_filewinid(const void *wp, const void *qi);
extern int rs_qf_getprop_qfbufnr(const void *qi);
extern int rs_copy_loclist(const void *from_qfl, void *to_qfl);
extern char *rs_skip_vimgrep_pat(char *p, char **s, int *flags);
extern void rs_reset_VIsual_and_resel(void);

extern void rs_qf_new_list(void *qi, const char *title);
extern void rs_qf_free_items(void *qfl);
extern void rs_qf_free_list(void *qfl);
extern void rs_qf_pop_stack(void *qi, bool adjust);

int nvim_qf_get_listcount(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_listcount; }

int nvim_qf_get_count(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_count; }

bool nvim_qf_get_nonevalid(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_nonevalid; }

linenr_T nvim_qfline_get_lnum(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_lnum; }

int nvim_qfline_get_col(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_col; }

linenr_T nvim_qf_pos_get_lnum(const void *pos_void) { return ((const pos_T *)pos_void)->lnum; }

int nvim_qf_pos_get_col(const void *pos_void) { return ((const pos_T *)pos_void)->col; }

void *nvim_qf_get_curlist(const void *qi_void) { return (void *)&((const qf_info_T *)qi_void)->qf_lists[((const qf_info_T *)qi_void)->qf_curlist]; }

void *nvim_qf_get_list_at(const void *qi_void, int idx) { return (void *)&((const qf_info_T *)qi_void)->qf_lists[idx]; }

int nvim_qf_get_curlist_idx(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_curlist; }

int nvim_qf_get_index(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_index; }

void *nvim_qf_get_ptr(const void *qfl_void) { return (void *)((const qf_list_T *)qfl_void)->qf_ptr; }

void *nvim_qf_get_start(const void *qfl_void) { return (void *)((const qf_list_T *)qfl_void)->qf_start; }

void *nvim_qfline_get_next(const void *qfp_void) { return (void *)((const qfline_T *)qfp_void)->qf_next; }

void *nvim_qfline_get_prev(const void *qfp_void) { return (void *)((const qfline_T *)qfp_void)->qf_prev; }

bool nvim_qfline_get_valid(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_valid != 0; }

char nvim_qfline_get_type(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_type; }

int nvim_qfline_get_fnum(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_fnum; }

linenr_T nvim_qfline_get_end_lnum(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_end_lnum; }

int nvim_qfline_get_end_col(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_end_col; }

int nvim_qfline_get_nr(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_nr; }

const char *nvim_qfline_get_text(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_text; }

const char *nvim_qfline_get_module(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_module; }

const char *nvim_qfline_get_pattern(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_pattern; }

bool nvim_qfline_get_cleared(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_cleared != 0; }

bool nvim_qfline_get_viscol(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_viscol != 0; }

unsigned nvim_qf_get_id(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_id; }

int nvim_qf_get_changedtick(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_changedtick; }

const char *nvim_qf_get_title(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_title; }

int nvim_qf_get_maxcount(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_maxcount; }

extern bool rs_qf_list_has_valid_entries(const void *qfl);

extern int rs_qf_id2nr(const void *qi, unsigned qf_id);
extern int rs_qf_restore_list(void *qi, unsigned save_qfid);
extern void *rs_qf_get_nth_entry(const void *qfl, int errornr, int *new_qfidx);

extern bool rs_qf_should_update_cursor(const void *qfl, int old_idx);
extern unsigned char rs_qf_type_display_char(unsigned char type_code);
extern int rs_qf_is_error_type(unsigned char type_code);
extern int rs_qf_is_warning_type(unsigned char type_code);

// Phase 1: Auname lookups and qf_types (migrated to Rust)
extern const char *rs_make_get_auname(int cmdidx);
extern const char *rs_cfile_get_auname(int cmdidx);
extern const char *rs_cbuffer_get_auname(int cmdidx);
extern const char *rs_cexpr_get_auname(int cmdidx);
extern const char *rs_vgr_get_auname(int cmdidx);
extern const char *rs_qf_types(int c, int nr, char *buf, size_t bufsz);

// Phase 2: Format and title helpers (migrated to Rust)
extern size_t rs_qf_fmt_text(const char *text, char *out, size_t out_size);
extern size_t rs_qf_range_text(int32_t lnum, int32_t end_lnum, int col, int end_col, char *out, size_t out_size);
extern size_t rs_qf_cmdtitle(const char *cmd, char *buf, size_t bufsz);

// Phase 3: Property flag operations and index resolution (migrated to Rust)
extern int rs_qf_getprop_keys2flags(const void *what, bool loclist);
extern int rs_qf_getprop_qfidx(const void *qi, const void *what);
extern int rs_qf_getprop_defaults(const void *qi, int flags, bool locstack, void *retdict);
extern int rs_qf_setprop_get_qfidx(const void *qi, const void *what, int action, bool *newlist);

// Phase 4: mark_adjust and valid counting (migrated to Rust)
extern bool rs_qf_mark_adjust(void *qi, int buf_fnum, int buf_has_flag, int32_t line1,
                               int32_t line2, int32_t amount, int32_t amount_after);
extern int rs_qf_get_valid_size(const void *qfl, bool count_files);
extern int rs_qf_get_cur_valid_idx(const void *qfl, int qf_index, bool count_files);

/// Result of the full errorformat-to-regex conversion
typedef struct {
  size_t bytes_written;
  char prefix;
  char flags;
  bool conthere;
  int status;
  int error_code;
  char error_char;
} EfmToRegpatResult;

// Full errorformat to regex conversion
extern EfmToRegpatResult rs_efm_to_regpat(const char *efm, size_t efm_len,
                                           char *addr, char *out, size_t out_size);

// Buffer size and part length helpers
extern size_t rs_efm_regpat_bufsz(const char *efm, size_t efm_len);
extern int rs_efm_option_part_len(const char *efm, size_t efm_max_len);

// Prefix type helpers
extern char rs_qf_parse_prefix_type(char prefix);
extern bool rs_qf_should_skip_line(char flags);
extern bool rs_qf_is_continuation(char prefix);
extern bool rs_qf_starts_multiline(char prefix);
extern bool rs_qf_is_file_handler(char prefix);

// Phase 5: parse_match and parse_line (migrated to Rust)
extern int rs_qf_parse_match(const char *linebuf, size_t linelen, void *fmt_ptr, const void *rm,
                              void *fields, bool qf_multiline, bool qf_multiscan, char **tail);
extern int rs_qf_parse_line(void *qfl, char *linebuf, size_t linelen, void *fmt_first,
                             void *fields);
extern void rs_qf_reset_fmt_start(void);

// Entry creation
extern int rs_qf_add_entry(void *qfl, char *dir, const char *fname, const char *module,
                           int bufnum, const char *mesg, linenr_T lnum, linenr_T end_lnum,
                           int col, int end_col, char vis_col, const char *pattern, int nr,
                           char type, const void *user_data, char valid);

// Directory stack operations (Phase 7)
extern const char *rs_qf_push_dir(void *qfl, char *dirbuf, bool is_file_stack);
extern const char *rs_qf_pop_dir(void *qfl, bool is_file_stack);
extern const char *rs_qf_guess_filepath(void *qfl, char *filename);

// Vimgrep functions
extern bool rs_vgr_match_buflines(void *qfl, const char *fname, void *buf, const char *spat,
                                  void *regmatch, int *tomatch, int duplicate_name, int flags);

// List management functions
extern int rs_qf_add_entries(void *qi, int qf_idx, void *list, char *title, int action);

// Display functions
extern void rs_qf_fill_buffer(void *qfl, void *buf, void *old_last, int qf_winid);

// Helpgrep functions (Phase 1)
extern void rs_hgr_search_in_rtp(void *qfl, void *p_regmatch, const char *lang);

// Init functions
extern int rs_qf_init_ext(void *qi, int qf_idx, const char *efile, void *buf,
                           void *tv, char *errorformat, bool newlist, linenr_T lnumfirst,
                           linenr_T lnumlast, const char *qf_title, char *enc);

extern void rs_ex_vimgrep(void *eap);

// Pass 4: stack query entry points (Phase 1)
extern size_t rs_qf_get_size_eap(void *eap);
extern size_t rs_qf_get_valid_size_eap(void *eap);
extern size_t rs_qf_get_cur_idx_eap(void *eap);
extern int rs_qf_get_cur_valid_idx_eap(void *eap);
extern linenr_T rs_qf_current_entry(void *wp);
extern int rs_grep_internal(int cmdidx);
extern void rs_qf_incr_changedtick(void *qfl);

void nvim_qf_set_curlist_idx(void *qi_void, int idx) { ((qf_info_T *)qi_void)->qf_curlist = idx; }

void nvim_qf_set_listcount(void *qi_void, int count) { ((qf_info_T *)qi_void)->qf_listcount = count; }

void nvim_qf_set_index(void *qfl_void, int idx) { ((qf_list_T *)qfl_void)->qf_index = idx; }

void nvim_qf_set_ptr(void *qfl_void, void *ptr) { ((qf_list_T *)qfl_void)->qf_ptr = (qfline_T *)ptr; }

int nvim_qf_get_qfl_type(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qfl_type; }

int nvim_qf_get_qi_type(const void *qi_void) { return ((const qf_info_T *)qi_void)->qfl_type; }

int nvim_qi_get_qfl_type(const void *qi_void) { return nvim_qf_get_qi_type(qi_void); }

void *nvim_qf_get_last(const void *qfl_void) { return (void *)((const qf_list_T *)qfl_void)->qf_last; }

const char *nvim_qfline_get_fname(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_fname; }

bool nvim_qf_get_has_user_data(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_has_user_data; }

// Forward declarations for static functions
void nvim_qf_store_title(void *qfl_void, const char *title)
{
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  if (qfl == NULL) {
    return;
  }
  XFREE_CLEAR(qfl->qf_title);
  if (title != NULL) {
    size_t len = strlen(title) + 1;
    char *p = xmallocz(len);
    qfl->qf_title = p;
    xstrlcpy(p, title, len + 1);
  }
}

void *nvim_get_ql_info(void) { return (void *)ql_info; }

// Phase 2 accessors: buf_T/win_T field access for qf_mark_adjust_entry and qf_jump_first
int nvim_buf_get_has_qf_entry(const void *buf_void) { return ((const buf_T *)buf_void)->b_has_qf_entry; }
int nvim_qf_buf_get_fnum(const void *buf_void) { return ((const buf_T *)buf_void)->b_fnum; }
void *nvim_buf_win_get_llist(const void *win_void) { return ((const win_T *)win_void)->w_llist; }
// nvim_check_can_set_curbuf_forceit already defined in tag_shim.c

// Phase 2: rs_qf_mark_adjust and rs_qf_jump_first
extern bool rs_qf_mark_adjust_entry(const void *buf, const void *wp, int32_t line1, int32_t line2, int32_t amount, int32_t amount_after);
extern void rs_qf_jump_first(void *qi, unsigned save_qfid, int forceit);

// Phase 3: qf_list_entry display
extern void rs_qf_list_entry(const void *qfp, int qf_idx, bool cursel,
                              int qfFile_hl_id, int qfSep_hl_id, int qfLine_hl_id);

// Phase 4: qf_list (:clist/:llist)
extern void rs_ex_clist(void *eap);

bool nvim_qf_get_multiline(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_multiline; }

void nvim_qf_set_multiline(void *qfl_void, bool multiline) { ((qf_list_T *)qfl_void)->qf_multiline = multiline; }

bool nvim_qf_get_multiignore(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_multiignore; }

void nvim_qf_set_multiignore(void *qfl_void, bool multiignore) { ((qf_list_T *)qfl_void)->qf_multiignore = multiignore; }

bool nvim_qf_get_multiscan(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_multiscan; }

void nvim_qf_set_multiscan(void *qfl_void, bool multiscan) { ((qf_list_T *)qfl_void)->qf_multiscan = multiscan; }

#define FMT_PATTERNS 14           // maximum number of % recognized

// Structure used to hold the info of one part of 'errorformat'
typedef struct efm_S efm_T;
struct efm_S {
  regprog_T *prog;        // pre-formatted part of 'errorformat'
  efm_T *next;        // pointer to next (NULL if last)
  char addr[FMT_PATTERNS];    // indices of used % patterns
  char prefix;                // prefix of this format line:
                              // 'D' enter directory
                              // 'X' leave directory
                              // 'A' start of multi-line message
                              // 'E' error message
                              // 'W' warning message
                              // 'I' informational message
                              // 'N' note message
                              // 'C' continuation line
                              // 'Z' end of multi-line message
                              // 'G' general, unspecific message
                              // 'P' push file (partial) message
                              // 'Q' pop/quit file (partial) message
                              // 'O' overread (partial) message
  char flags;                 // additional flags given in prefix
                              // '-' do not include this line
                              // '+' include whole line in message
  int conthere;                 // %> used
};

/// Used to delay the deletion of locations lists by autocmds.
typedef struct qf_delq_S {
  struct qf_delq_S *next;
  qf_info_T *qi;
} qf_delq_T;

enum {
  QF_FAIL = 0,
  QF_OK = 1,
  QF_END_OF_INPUT = 2,
  QF_NOMEM = 3,
  QF_IGNORE_LINE = 4,
  QF_MULTISCAN = 5,
  QF_ABORT = 6,
};

/// list.
typedef struct {
  char *linebuf;
  size_t linelen;
  char *growbuf;
  size_t growbufsiz;
  FILE *fd;
  typval_T *tv;
  char *p_str;
  list_T *p_list;
  listitem_T *p_li;
  buf_T *buf;
  linenr_T buflnum;
  linenr_T lnumlast;
  vimconv_T vc;
} qfstate_T;

typedef struct {
  char *namebuf;
  int bnr;
  char *module;
  char *errmsg;
  size_t errmsglen;
  linenr_T lnum;
  linenr_T end_lnum;
  int col;
  int end_col;
  bool use_viscol;
  char *pattern;
  int enr;
  char type;
  typval_T *user_data;
  bool valid;
} qffields_T;

/// :vimgrep command arguments
typedef struct {
  int tomatch;          ///< maximum number of matches to find
  char *spat;          ///< search pattern
  int flags;             ///< search modifier
  char **fnames;       ///< list of files to search
  int fcount;            ///< number of files
  regmmatch_T regmatch;  ///< compiled search pattern
  char *qf_title;      ///< quickfix list title
} vgr_args_T;

#include "quickfix_shim.c.generated.h"
extern int rs_win_valid(win_T *win);

// Rust FFI declarations (window wrappers removed)
extern void rs_check_lnums(int do_curwin);
extern int rs_tabline_height(void);
extern void rs_win_setheight(int height);
extern void rs_win_setwidth(int width);

// Rust fold FFI declarations
extern void rs_foldOpenCursor(void);
extern void rs_foldUpdateAll(win_T *win);

static const char *e_no_more_items = N_("E553: No more items");
static const char *e_current_quickfix_list_was_changed =
  N_("E925: Current quickfix list was changed");
static const char *e_current_location_list_was_changed =
  N_("E926: Current location list was changed");

enum { QF_WINHEIGHT = 10, };  ///< default height for quickfix window

// Quickfix window check helper macro
#define IS_QF_WINDOW(wp) (bt_quickfix((wp)->w_buffer) && (wp)->w_llist_ref == NULL)
// Location list window check helper macro
#define IS_LL_WINDOW(wp) (bt_quickfix((wp)->w_buffer) && (wp)->w_llist_ref != NULL)

// Quickfix and location list stack check helper macros
#define IS_QF_STACK(qi)       ((qi)->qfl_type == QFLT_QUICKFIX)
#define IS_LL_STACK(qi)       ((qi)->qfl_type == QFLT_LOCATION)
#define IS_QF_LIST(qfl)       ((qfl)->qfl_type == QFLT_QUICKFIX)
#define IS_LL_LIST(qfl)       ((qfl)->qfl_type == QFLT_LOCATION)

// Return location list for window 'wp'
// For location list window, return the referenced location list
#define GET_LOC_LIST(wp) (IS_LL_WINDOW(wp) ? (wp)->w_llist_ref : (wp)->w_llist)

// Macro to loop through all the items in a quickfix list
// Quickfix item index starts from 1, so i below starts at 1
#define FOR_ALL_QFL_ITEMS(qfl, qfp, i) \
  for ((i) = 1, (qfp) = (qfl)->qf_start; \
       !got_int && (i) <= (qfl)->qf_count && (qfp) != NULL; \
       (i)++, (qfp) = (qfp)->qf_next)

bool nvim_win_valid(const void *wp_void) { return wp_void == NULL ? false : rs_win_valid((win_T *)wp_void) != 0; }

void *nvim_win_get_loclist(const void *wp_void) { return wp_void == NULL ? NULL : (void *)GET_LOC_LIST((win_T *)wp_void); }

void *nvim_qf_find_win_handle(const void *qi_void) { return qi_void == NULL ? NULL : (void *)qf_find_win((const qf_info_T *)qi_void); }

int nvim_qf_win_get_handle(const void *wp_void) { return wp_void == NULL ? 0 : ((const win_T *)wp_void)->handle; }

/// This is equivalent to qflist_valid() but works with a quickfix stack pointer
bool nvim_qflist_valid(const void *qi_void, unsigned qf_id)
{
  if (qi_void == NULL) {
    return false;
  }
  const qf_info_T *qi = (const qf_info_T *)qi_void;
  for (int i = 0; i < qi->qf_listcount; i++) {
    if (qi->qf_lists[i].qf_id == qf_id) {
      return true;
    }
  }
  return false;
}

/// This is equivalent to is_qf_entry_present()
bool nvim_qf_entry_present(const void *qfl_void, const void *qf_ptr_void)
{
  if (qfl_void == NULL || qf_ptr_void == NULL) {
    return false;
  }
  const qf_list_T *qfl = (const qf_list_T *)qfl_void;
  const qfline_T *target = (const qfline_T *)qf_ptr_void;

  qfline_T *qfp;
  int i;
  FOR_ALL_QFL_ITEMS(qfl, qfp, i) {
    if (qfp == target) {
      return true;
    }
  }
  return false;
}

const char *nvim_qf_types(int c, int nr) { static char buf[20]; return rs_qf_types(c, nr, buf, sizeof(buf)); }

void nvim_emsg_e_no_more_items(void) { emsg(_(e_no_more_items)); }

void nvim_qf_increment_listcount(void *qi_void) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_listcount++; }

/// Decrement the list count after removing a list
void nvim_qf_decrement_listcount(void *qi_void)
{
  if (qi_void == NULL) {
    return;
  }
  qf_info_T *qi = (qf_info_T *)qi_void;
  if (qi->qf_listcount > 0) {
    qi->qf_listcount--;
  }
}

void nvim_qf_set_start(void *qfl_void, void *start) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_start = (qfline_T *)start; }

void nvim_qf_set_last(void *qfl_void, void *last) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_last = (qfline_T *)last; }

void nvim_qf_set_count(void *qfl_void, int count) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_count = count; }

void nvim_qf_increment_count(void *qfl_void) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_count++; }

void nvim_qf_set_nonevalid(void *qfl_void, bool nonevalid) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_nonevalid = nonevalid; }

void nvim_qfline_set_next(void *qfp_void, void *next) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_next = (qfline_T *)next; }

void nvim_qfline_set_prev(void *qfp_void, void *prev) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_prev = (qfline_T *)prev; }

void *nvim_qfline_alloc(void) { return xcalloc(1, sizeof(qfline_T)); }

/// Free a qfline_T structure and its string fields
void nvim_qfline_free(void *qfp_void)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  xfree(qfp->qf_module);
  xfree(qfp->qf_fname);
  xfree(qfp->qf_pattern);
  xfree(qfp->qf_text);
  tv_clear(&qfp->qf_user_data);
  xfree(qfp);
}

void nvim_qfline_set_fnum(void *qfp_void, int fnum) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_fnum = fnum; }

void nvim_qfline_set_lnum(void *qfp_void, linenr_T lnum) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_lnum = lnum; }

void nvim_qfline_set_end_lnum(void *qfp_void, linenr_T end_lnum) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_end_lnum = end_lnum; }

void nvim_qfline_set_col(void *qfp_void, int col) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_col = col; }

void nvim_qfline_set_end_col(void *qfp_void, int end_col) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_end_col = end_col; }

void nvim_qfline_set_nr(void *qfp_void, int nr) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_nr = nr; }

void nvim_qfline_set_type(void *qfp_void, char type) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_type = type; }

void nvim_qfline_set_viscol(void *qfp_void, char viscol) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_viscol = viscol; }

void nvim_qfline_set_valid(void *qfp_void, char valid) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_valid = valid; }

void nvim_qfline_set_cleared(void *qfp_void, char cleared) { if (qfp_void != NULL) ((qfline_T *)qfp_void)->qf_cleared = cleared; }

/// Set qf_text field (duplicates the string)
void nvim_qfline_set_text(void *qfp_void, const char *text)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  xfree(qfp->qf_text);
  qfp->qf_text = (text != NULL) ? xstrdup(text) : NULL;
}

/// Set qf_module field (duplicates the string)
void nvim_qfline_set_module(void *qfp_void, const char *module)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  xfree(qfp->qf_module);
  qfp->qf_module = (module != NULL && *module != NUL) ? xstrdup(module) : NULL;
}

/// Set qf_fname field (duplicates the string)
void nvim_qfline_set_fname(void *qfp_void, const char *fname)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  xfree(qfp->qf_fname);
  qfp->qf_fname = (fname != NULL && *fname != NUL) ? xstrdup(fname) : NULL;
}

/// Set qf_pattern field (duplicates the string)
void nvim_qfline_set_pattern(void *qfp_void, const char *pattern)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  xfree(qfp->qf_pattern);
  qfp->qf_pattern = (pattern != NULL && *pattern != NUL) ? xstrdup(pattern) : NULL;
}

/// Set qf_user_data field (copies the typval)
void nvim_qfline_set_user_data(void *qfp_void, void *qfl_void, const void *user_data_void)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  const typval_T *user_data = (const typval_T *)user_data_void;

  if (user_data == NULL || user_data->v_type == VAR_UNKNOWN) {
    qfp->qf_user_data.v_type = VAR_UNKNOWN;
  } else {
    tv_copy(user_data, &qfp->qf_user_data);
    if (qfl_void != NULL) {
      qf_list_T *qfl = (qf_list_T *)qfl_void;
      qfl->qf_has_user_data = true;
    }
  }
}

/// Mark buffer as having quickfix/location list entry
void nvim_qf_mark_buf_has_entry(int bufnum, bool is_location_list)
{
  buf_T *buf = buflist_findnr(bufnum);
  if (buf != NULL) {
    buf->b_has_qf_entry |= is_location_list ? BUF_HAS_LL_ENTRY : BUF_HAS_QF_ENTRY;
  }
}

int nvim_qf_get_fnum_for_entry(void *qfl_void, char *directory, char *fname) { return qfl_void == NULL ? 0 : qf_get_fnum((qf_list_T *)qfl_void, directory, fname); }

/// Returns allocated string or NULL (caller must free)
char *nvim_qf_fix_fname(const char *fname, int bufnum)
{
  if (fname == NULL) {
    return NULL;
  }

  char *fullname = fix_fname((char *)fname);
  if (fullname == NULL) {
    return NULL;
  }

  buf_T *buf = buflist_findnr(bufnum);
  if (buf != NULL && buf->b_ffname != NULL) {
    if (path_fnamecmp(fullname, buf->b_ffname) != 0) {
      char *p = path_try_shorten_fname(fullname);
      if (p != NULL) {
        char *result = xstrdup(p);
        xfree(fullname);
        return result;
      }
    }
  }

  xfree(fullname);
  return NULL;
}

bool nvim_qf_is_printc(int c) { return vim_isprintc(c); }

void nvim_qf_set_id(void *qfl_void, unsigned id) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_id = id; }

void nvim_qf_set_qfl_type(void *qfl_void, int qfl_type) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qfl_type = (qfltype_T)qfl_type; }

void nvim_qf_set_has_user_data(void *qfl_void, bool has_user_data) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_has_user_data = has_user_data; }

/// Get a mutable list handle at the specified index
void *nvim_qf_get_list_at_mut(void *qi_void, int idx)
{
  if (qi_void == NULL) {
    return NULL;
  }
  qf_info_T *qi = (qf_info_T *)qi_void;
  if (idx < 0 || idx >= qi->qf_listcount) {
    return NULL;
  }
  return &qi->qf_lists[idx];
}

unsigned nvim_qf_alloc_next_id(void) { return ++last_qf_id; }

void nvim_qf_clear_list_struct(void *qfl_void) { if (qfl_void != NULL) memset(qfl_void, 0, sizeof(qf_list_T)); }

void nvim_qf_free_title(void *qfl_void) { if (qfl_void != NULL) XFREE_CLEAR(((qf_list_T *)qfl_void)->qf_title); }

/// Free the context typval of a quickfix list
void nvim_qf_free_ctx(void *qfl_void)
{
  if (qfl_void == NULL) {
    return;
  }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  tv_free(qfl->qf_ctx);
  qfl->qf_ctx = NULL;
}

void nvim_qf_free_callback(void *qfl_void) { if (qfl_void != NULL) callback_free(&((qf_list_T *)qfl_void)->qf_qftf_cb); }

void nvim_qf_copy_ctx(const void *from_qfl_void, void *to_qfl_void)
{
  if (from_qfl_void == NULL || to_qfl_void == NULL) { return; }
  const qf_list_T *from_qfl = (const qf_list_T *)from_qfl_void;
  qf_list_T *to_qfl = (qf_list_T *)to_qfl_void;
  if (from_qfl->qf_ctx != NULL) {
    to_qfl->qf_ctx = xcalloc(1, sizeof(*to_qfl->qf_ctx));
    tv_copy(from_qfl->qf_ctx, to_qfl->qf_ctx);
  } else {
    to_qfl->qf_ctx = NULL;
  }
}

void nvim_qf_copy_callback(const void *from_qfl_void, void *to_qfl_void)
{
  if (from_qfl_void == NULL || to_qfl_void == NULL) { return; }
  callback_copy(&((qf_list_T *)to_qfl_void)->qf_qftf_cb, &((const qf_list_T *)from_qfl_void)->qf_qftf_cb);
}

const void *nvim_qfline_get_user_data_ptr(const void *qfp_void) { return qfp_void == NULL ? NULL : (const void *)&((const qfline_T *)qfp_void)->qf_user_data; }

void nvim_qf_set_changedtick(void *qfl_void, int changedtick) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_changedtick = changedtick; }

/// Copies lists[1..count] to lists[0..count-1]
void nvim_qf_shift_lists_down(void *qi_void)
{
  if (qi_void == NULL) {
    return;
  }
  qf_info_T *qi = (qf_info_T *)qi_void;
  for (int i = 1; i < qi->qf_listcount; i++) {
    qi->qf_lists[i - 1] = qi->qf_lists[i];
  }
}


static int qf_get_fnum(qf_list_T *qfl, char *directory, char *fname);

void *nvim_qf_get_dir_stack(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_dir_stack; }
void nvim_qf_set_dir_stack(void *qfl_void, void *stack) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_dir_stack = (struct dir_stack_T *)stack; }
void *nvim_qf_get_file_stack(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_file_stack; }
void nvim_qf_set_file_stack(void *qfl_void, void *stack) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_file_stack = (struct dir_stack_T *)stack; }
const char *nvim_qf_get_directory(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_directory; }
void nvim_qf_set_directory(void *qfl_void, char *dir) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_directory = dir; }
const char *nvim_qf_get_currfile(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_currfile; }
void nvim_qf_set_currfile(void *qfl_void, char *file) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_currfile = file; }
int nvim_qf_get_fnum(void *qfl_void, char *directory, char *fname) { return qfl_void == NULL ? 0 : qf_get_fnum((qf_list_T *)qfl_void, directory, fname); }

// Phase 3 accessors: typval dict operations for property flag / index resolution functions

/// Check if a dict has the given key (returns true if tv_dict_find != NULL).
bool nvim_tv_dict_find_has_key(const void *dict, const char *key, int key_len)
{
  return tv_dict_find((const dict_T *)dict, key, (ptrdiff_t)key_len) != NULL;
}

/// Find a VAR_NUMBER item in dict.  Returns true and sets *out if found and typed correctly.
bool nvim_tv_dict_find_nr(const void *dict, const char *key, int key_len, int64_t *out)
{
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, (ptrdiff_t)key_len);
  if (di == NULL || di->di_tv.v_type != VAR_NUMBER) {
    return false;
  }
  *out = (int64_t)di->di_tv.vval.v_number;
  return true;
}

/// Return the string value of a dict key if it is VAR_STRING with value "$", else NULL.
bool nvim_tv_dict_find_str_is_dollar(const void *dict, const char *key, int key_len)
{
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, (ptrdiff_t)key_len);
  return di != NULL && di->di_tv.v_type == VAR_STRING
         && di->di_tv.vval.v_string != NULL
         && strequal(di->di_tv.vval.v_string, "$");
}

/// Add a number to a dict; returns OK or FAIL.
int nvim_tv_dict_add_nr_ret(void *dict, const char *key, int key_len, int64_t nr)
{
  return tv_dict_add_nr((dict_T *)dict, key, (size_t)key_len, (varnumber_T)nr);
}

/// Add a string copy to a dict; returns OK or FAIL.
int nvim_tv_dict_add_str_copy(void *dict, const char *key, int key_len, const char *val)
{
  return tv_dict_add_str((dict_T *)dict, key, (size_t)key_len, val == NULL ? "" : val);
}

/// Allocate an empty list and add it to a dict; returns OK or FAIL.
int nvim_tv_dict_add_list_empty(void *dict, const char *key, int key_len)
{
  list_T *l = tv_list_alloc(kListLenMayKnow);
  return tv_dict_add_list((dict_T *)dict, key, (size_t)key_len, l);
}

/// Get the valid quickfix buffer number for a qi (0 if not valid).
int nvim_qf_get_valid_bufnr(const void *qi_void)
{
  if (qi_void == NULL) {
    return 0;
  }
  const qf_info_T *qi = (const qf_info_T *)qi_void;
  if (buflist_findnr(qi->qf_bufnr) != NULL) {
    return qi->qf_bufnr;
  }
  return 0;
}

// Phase 4 accessors: qfl empty check for mark_adjust / valid counting

/// Check if a qf_list_T is empty (no entries).
bool nvim_qf_list_is_empty(const void *qfl_void)
{
  if (qfl_void == NULL) {
    return true;
  }
  return rs_qf_list_empty(qfl_void);
}

static efm_T *parse_efm_option(char *efm);
static void free_efm_list(efm_T **efm_first);
static int qf_parse_line(qf_list_T *qfl, char *linebuf, size_t linelen, efm_T *fmt_first,
                         qffields_T *fields);

// EfmHandle is now defined in quickfix.h

EfmHandle nvim_qf_parse_efm_option(char *efm) { return efm == NULL ? NULL : parse_efm_option(efm); }

void nvim_qf_free_efm_list(EfmHandle efm_first) { efm_T *efm = (efm_T *)efm_first; free_efm_list(&efm); }

/// Get the next pattern in the errorformat list
EfmHandle nvim_efm_get_next(EfmHandle efm) { return efm == NULL ? NULL : ((efm_T *)efm)->next; }
char nvim_efm_get_prefix(EfmHandle efm) { return efm == NULL ? '\0' : ((efm_T *)efm)->prefix; }
char nvim_efm_get_flags(EfmHandle efm) { return efm == NULL ? '\0' : ((efm_T *)efm)->flags; }
int nvim_efm_get_conthere(EfmHandle efm) { return efm == NULL ? 0 : ((efm_T *)efm)->conthere; }
char nvim_efm_get_addr(EfmHandle efm, int idx) { return (efm == NULL || idx < 0 || idx >= FMT_PATTERNS) ? 0 : ((efm_T *)efm)->addr[idx]; }

static int qf_get_nextline(qfstate_T *state);
static int qf_setup_state(qfstate_T *pstate, char *restrict enc, const char *restrict efile,
                          typval_T *tv, buf_T *buf, linenr_T lnumfirst, linenr_T lnumlast);
static void qf_cleanup_state(qfstate_T *pstate);

// QfStateHandle is now defined in quickfix.h

QfStateHandle nvim_qf_state_alloc(void) { return xcalloc(1, sizeof(qfstate_T)); }

/// Free a parser state object
void nvim_qf_state_free(QfStateHandle state)
{
  if (state != NULL) {
    qf_cleanup_state((qfstate_T *)state);
    xfree(state);
  }
}

/// Returns OK on success, FAIL on error
int nvim_qf_state_setup_file(QfStateHandle state, char *enc, const char *efile)
{
  if (state == NULL) {
    return FAIL;
  }
  return qf_setup_state((qfstate_T *)state, enc, efile, NULL, NULL, 0, 0);
}

/// Returns OK on success, FAIL on error
int nvim_qf_state_setup_buffer(QfStateHandle state, void *buf, int lnumfirst, int lnumlast)
{
  if (state == NULL || buf == NULL) {
    return FAIL;
  }
  return qf_setup_state((qfstate_T *)state, NULL, NULL, NULL,
                        (buf_T *)buf, lnumfirst, lnumlast);
}

int nvim_qf_state_get_nextline(QfStateHandle state) { return state == NULL ? QF_FAIL : qf_get_nextline((qfstate_T *)state); }
const char *nvim_qf_state_get_linebuf(QfStateHandle state) { return state == NULL ? NULL : ((qfstate_T *)state)->linebuf; }
size_t nvim_qf_state_get_linelen(QfStateHandle state) { return state == NULL ? 0 : ((qfstate_T *)state)->linelen; }
bool nvim_qf_state_has_fd(QfStateHandle state) { return state == NULL ? false : ((qfstate_T *)state)->fd != NULL; }
bool nvim_qf_state_has_tv(QfStateHandle state) { return state == NULL ? false : ((qfstate_T *)state)->tv != NULL; }

bool nvim_qf_state_has_buf(QfStateHandle state) { return state == NULL ? false : ((qfstate_T *)state)->buf != NULL; }

static win_T *qf_find_win(const qf_info_T *qi);
static buf_T *qf_find_buf(qf_info_T *qi);
static bool qf_win_pos_update(qf_info_T *qi, int old_qf_index);
static void qf_update_buffer(qf_info_T *qi, qfline_T *old_last);

void *nvim_qf_find_win_for_stack(const void *qi_void) { return qi_void == NULL ? NULL : qf_find_win((const qf_info_T *)qi_void); }
void *nvim_qf_find_buf_for_stack(void *qi_void) { return qi_void == NULL ? NULL : qf_find_buf((qf_info_T *)qi_void); }
bool nvim_qf_win_pos_update(void *qi_void, int old_qf_index) { return qi_void == NULL ? false : qf_win_pos_update((qf_info_T *)qi_void, old_qf_index); }
void nvim_qf_update_buffer(void *qi_void, void *old_last) { if (qi_void != NULL) qf_update_buffer((qf_info_T *)qi_void, (qfline_T *)old_last); }
int nvim_qf_get_bufnr(const void *qi_void) { return qi_void == NULL ? -1 : ((const qf_info_T *)qi_void)->qf_bufnr; }
void nvim_qf_set_bufnr(void *qi_void, int bufnr) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_bufnr = bufnr; }

/// Check if a window is a quickfix window
bool nvim_win_is_qf_win(const void *win_void)
{
  if (win_void == NULL) {
    return false;
  }
  const win_T *win = (const win_T *)win_void;
  if (!buf_valid(win->w_buffer)) {
    return false;
  }
  return bt_quickfix(win->w_buffer);
}

void *nvim_win_get_llist_ref(const void *win_void) { return win_void == NULL ? NULL : ((const win_T *)win_void)->w_llist_ref; }

static int qf_set_properties(qf_info_T *qi, const dict_T *what, int action, char *title);
static int qf_get_properties(win_T *wp, dict_T *what, dict_T *retdict);

bool nvim_qf_is_qf_stack(const void *qi_void) { return qi_void == NULL ? false : qi_void == ql_info; }
bool nvim_qf_is_ll_stack(const void *qi_void) { return qi_void == NULL ? false : qi_void != ql_info; }
int nvim_qf_get_refcount(const void *qi_void) { return qi_void == NULL ? 0 : ((const qf_info_T *)qi_void)->qf_refcount; }
void nvim_qf_incr_refcount(void *qi_void) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_refcount++; }
void nvim_qf_set_refcount(void *qi_void, int v) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_refcount = v; }

// ---- Phase 6 lifecycle accessors ----

/// Free the qf_lists array inside a qf_info_T (does NOT free the struct itself).
void nvim_qf_free_lists_array(void *qi_void)
{
  if (qi_void == NULL) { return; }
  xfree(((qf_info_T *)qi_void)->qf_lists);
  ((qf_info_T *)qi_void)->qf_lists = NULL;
}

/// Free the qf_info_T struct itself (only for heap-allocated stacks).
void nvim_qf_free_info(void *qi_void) { xfree(qi_void); }

// nvim_get_curbuf, nvim_get_curwin: already exist in window_shim.c
// nvim_buflist_findnr: already exists in buffer.c
// nvim_buf_get_nwindows: already exists in buffer.c

/// Return curwin->w_buffer (may be NULL).
void *nvim_curwin_get_buffer(void) { return (void *)curwin->w_buffer; }

/// Set curwin->w_buffer (may be set to NULL).
void nvim_curwin_set_buffer(void *buf) { curwin->w_buffer = (buf_T *)buf; }

/// Wipe a buffer (calls close_buffer with DOBUF_WIPE).
void nvim_close_buffer_wipe(void *buf_void)
{
  if (buf_void == NULL) { return; }
  close_buffer(NULL, (buf_T *)buf_void, DOBUF_WIPE, false, false);
}

/// Set wp->w_llist = NULL.
void nvim_win_set_llist_null(void *wp_void)
{
  if (wp_void != NULL) { ((win_T *)wp_void)->w_llist = NULL; }
}

/// Set wp->w_llist_ref = NULL.
void nvim_win_set_llist_ref_null(void *wp_void)
{
  if (wp_void != NULL) { ((win_T *)wp_void)->w_llist_ref = NULL; }
}

/// Atomically exchange wp->w_llist: set to NULL and return old value.
void *nvim_win_take_llist(void *wp_void)
{
  if (wp_void == NULL) { return NULL; }
  win_T *wp = (win_T *)wp_void;
  void *old = (void *)wp->w_llist;
  wp->w_llist = NULL;
  return old;
}

/// Atomically exchange wp->w_llist_ref: set to NULL and return old value.
void *nvim_win_take_llist_ref(void *wp_void)
{
  if (wp_void == NULL) { return NULL; }
  win_T *wp = (win_T *)wp_void;
  void *old = (void *)wp->w_llist_ref;
  wp->w_llist_ref = NULL;
  return old;
}

// ---- Rust lifecycle forward declarations ----
extern void rs_incr_quickfix_busy(void);
extern void rs_decr_quickfix_busy(void);
extern void rs_locstack_queue_delreq(void *qi);
extern void rs_check_quickfix_busy(void);
extern void rs_wipe_qf_buffer(void *qi);
extern void rs_ll_free_all(void **pqi);
extern void rs_qf_free_all(void *wp);

// ---- Phase 3 stack-allocation accessors ----

/// Returns address of the static ql_info_actual (global quickfix stack).
void *nvim_get_ql_info_actual(void) { return (void *)&ql_info_actual; }

/// Allocate a zeroed qf_info_T on the heap.
void *nvim_qf_alloc_info(void) { return xcalloc(1, sizeof(qf_info_T)); }

/// Set qi->qfl_type.
void nvim_qf_set_qi_type(void *qi_void, int qfltype) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qfl_type = (qfltype_T)qfltype; }

/// Set qi->qf_maxcount.
void nvim_qf_set_maxcount(void *qi_void, int n) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_maxcount = n; }

/// Set qi->qf_lists to a freshly xcalloc'd array of n qf_list_T elements.
void nvim_qf_set_new_lists(void *qi_void, int n)
{
  if (qi_void == NULL) { return; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  qi->qf_lists = xcalloc((size_t)n, sizeof(qf_list_T));
}

/// Return wp->w_p_lhi (location history option value).
int nvim_win_get_p_lhi(const void *wp_void) { return wp_void == NULL ? 0 : (int)((const win_T *)wp_void)->w_p_lhi; }

/// Set *pwinp = curwin.
void nvim_set_pwin_to_curwin(void **pwinp) { if (pwinp != NULL) *pwinp = (void *)curwin; }

/// Return true if cmdidx is a location-list command.
bool nvim_is_loclist_cmd(int cmdidx) { return is_loclist_cmd((cmdidx_T)cmdidx); }

// nvim_eap_get_cmdidx: already exists in ex_docmd.c

// ---- Rust Phase 3 forward declarations ----
extern void *rs_qf_alloc_stack(int qfltype, int n);
extern void *rs_ll_get_or_alloc_list(void *wp);
extern void *rs_qf_cmd_get_stack(void *eap, bool print_emsg);
extern void *rs_qf_cmd_get_or_alloc_stack(const void *eap, void **pwinp);

void *nvim_qf_get_ctx(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_ctx; }
bool nvim_qf_has_user_data(const void *qfl_void) { return qfl_void == NULL ? false : ((const qf_list_T *)qfl_void)->qf_has_user_data; }
void nvim_qf_incr_changedtick(void *qfl_void) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_changedtick++; }

// Looking up a buffer can be slow if there are many.  Remember the last one
// to make this a lot faster if there are multiple matches in the same file.
static char *qf_last_bufname = NULL;
static bufref_T qf_last_bufref = { NULL, 0, 0 };

static garray_T qfga;

/// many alloc/free calls.
static garray_T *qfga_get(void)
{
  static bool initialized = false;

  if (!initialized) {
    initialized = true;
    ga_init(&qfga, 1, 256);
  }

  // Reset the length to zero.  Retain ga_data from previous use to avoid
  // many alloc/free calls.
  qfga.ga_len = 0;

  return &qfga;
}

/// grow array.  Otherwise just reset the grow array length.
static void qfga_clear(void)
{
  if (qfga.ga_maxlen > 1000) {
    ga_clear(&qfga);
  } else {
    qfga.ga_len = 0;
  }
}

// quickfix_busy and qf_delq_head are now managed by Rust in lifecycle.rs.

/// to the quickfix list 'qfl'.
static int qf_init_process_nextline(qf_list_T *qfl, efm_T *fmt_first, qfstate_T *state,
                                    qffields_T *fields)
{
  // Get the next line from a file/buffer/list/string
  int status = qf_get_nextline(state);
  if (status != QF_OK) {
    return status;
  }

  status = qf_parse_line(qfl, state->linebuf, state->linelen,
                         fmt_first, fields);
  if (status != QF_OK) {
    return status;
  }

  return rs_qf_add_entry(qfl,
                      qfl->qf_directory,
                      (*fields->namebuf || qfl->qf_directory != NULL)
                      ? fields->namebuf
                      : ((qfl->qf_currfile != NULL && fields->valid)
                         ? qfl->qf_currfile : NULL),
                      fields->module,
                      fields->bnr,
                      fields->errmsg,
                      fields->lnum,
                      fields->end_lnum,
                      fields->col,
                      fields->end_col,
                      fields->use_viscol,
                      fields->pattern,
                      fields->enr,
                      fields->type,
                      fields->user_data,
                      fields->valid);
}

/// @returns -1 for error, number of errors for success.
int qf_init(win_T *wp, const char *restrict efile, char *restrict errorformat, int newlist,
            const char *restrict qf_title, char *restrict enc)
{
  qf_info_T *qi = wp == NULL ? ql_info : ll_get_or_alloc_list(wp);
  assert(qi != NULL);

  return rs_qf_init_ext(qi, qi->qf_curlist, efile, curbuf, NULL, errorformat,
                     newlist, 0, 0, qf_title, enc);
}

// Maximum number of bytes allowed per line while reading an errorfile.
static const size_t LINE_MAXLEN = 4096;

// Format pattern definitions and FMT_PATTERN_M/R indices are in Rust (parse.rs).

// Converts a 'errorformat' string to regular expression pattern
// Now uses Rust implementation and adapts the result.
static int efm_to_regpat(const char *efm, int len, efm_T *fmt_ptr, char *regpat)
  FUNC_ATTR_NONNULL_ALL
{
  // Use Rust implementation for the core conversion
  EfmToRegpatResult result = rs_efm_to_regpat(efm, (size_t)len,
                                               fmt_ptr->addr, regpat,
                                               rs_efm_regpat_bufsz(efm, (size_t)len));

  if (result.status != OK) {
    // Handle error messages based on error code
    switch (result.error_code) {
      case 372:
        semsg(_("E372: Too many %%%c in format string"), result.error_char);
        break;
      case 373:
        semsg(_("E373: Unexpected %%%c in format string"), result.error_char);
        break;
      case 374:
        emsg(_("E374: Missing ] in format string"));
        break;
      case 375:
        semsg(_("E375: Unsupported %%%c in format string"), result.error_char);
        break;
      case 376:
        semsg(_("E376: Invalid %%%c in format string prefix"), result.error_char);
        break;
      case 377:
        semsg(_("E377: Invalid %%%c in format string"), result.error_char);
        break;
      default:
        emsg(_("E378: 'errorformat' contains no pattern"));
        break;
    }
    return FAIL;
  }

  // Copy prefix, flags, and conthere from result to efm_T
  fmt_ptr->prefix = result.prefix;
  fmt_ptr->flags = result.flags;
  fmt_ptr->conthere = result.conthere;

  return OK;
}

static efm_T *fmt_start = NULL;  // cached across qf_parse_line() calls

// callback function for 'quickfixtextfunc'
static Callback qftf_cb;

static void free_efm_list(efm_T **efm_first)
{
  for (efm_T *efm_ptr = *efm_first; efm_ptr != NULL; efm_ptr = *efm_first) {
    *efm_first = efm_ptr->next;
    vim_regfree(efm_ptr->prog);
    xfree(efm_ptr);
  }

  fmt_start = NULL;
  rs_qf_reset_fmt_start();
}

/// the parsed 'errorformat' option.
static efm_T *parse_efm_option(char *efm)
{
  efm_T *fmt_first = NULL;
  efm_T *fmt_last = NULL;

  // Get some space to modify the format string into.
  size_t sz = rs_efm_regpat_bufsz(efm, strlen(efm));
  char *fmtstr = xmalloc(sz);

  while (efm[0] != NUL) {
    // Allocate a new eformat structure and put it at the end of the list
    efm_T *fmt_ptr = (efm_T *)xcalloc(1, sizeof(efm_T));
    if (fmt_first == NULL) {        // first one
      fmt_first = fmt_ptr;
    } else {
      fmt_last->next = fmt_ptr;
    }
    fmt_last = fmt_ptr;

    // Isolate one part in the 'errorformat' option
    int len = rs_efm_option_part_len(efm, strlen(efm));

    if (efm_to_regpat(efm, len, fmt_ptr, fmtstr) == FAIL) {
      goto parse_efm_error;
    }
    if ((fmt_ptr->prog = vim_regcomp(fmtstr, RE_MAGIC + RE_STRING)) == NULL) {
      goto parse_efm_error;
    }
    // Advance to next part
    efm = (char *)rs_skip_to_option_part(efm + len);       // skip comma and spaces
  }

  if (fmt_first == NULL) {      // nothing found
    emsg(_("E378: 'errorformat' contains no pattern"));
  }

  goto parse_efm_end;

parse_efm_error:
  free_efm_list(&fmt_first);

parse_efm_end:
  xfree(fmtstr);

  return fmt_first;
}

/// Allocate more memory for the line buffer used for parsing lines.
static char *qf_grow_linebuf(qfstate_T *state, size_t newsz)
{
  // If the line exceeds LINE_MAXLEN exclude the last
  // byte since it's not a NL character.
  state->linelen = newsz > LINE_MAXLEN ? LINE_MAXLEN - 1 : newsz;
  if (state->growbuf == NULL) {
    state->growbuf = xmalloc(state->linelen + 1);
    state->growbufsiz = state->linelen;
  } else if (state->linelen > state->growbufsiz) {
    state->growbuf = xrealloc(state->growbuf, state->linelen + 1);
    state->growbufsiz = state->linelen;
  }
  return state->growbuf;
}

/// Get the next string (separated by newline) from state->p_str.
static int qf_get_next_str_line(qfstate_T *state)
{
  // Get the next line from the supplied string
  char *p_str = state->p_str;

  if (*p_str == NUL) {  // Reached the end of the string
    return QF_END_OF_INPUT;
  }

  char *p = vim_strchr(p_str, '\n');
  size_t len = (p != NULL) ? (size_t)(p - p_str) + 1 : strlen(p_str);

  if (len > IOSIZE - 2) {
    state->linebuf = qf_grow_linebuf(state, len);
  } else {
    state->linebuf = IObuff;
    state->linelen = len;
  }
  memcpy(state->linebuf, p_str, state->linelen);
  state->linebuf[state->linelen] = NUL;

  // Increment using len in order to discard the rest of the line if it
  // exceeds LINE_MAXLEN.
  p_str += len;
  state->p_str = p_str;

  return QF_OK;
}

/// Get the next string from state->p_Li.
static int qf_get_next_list_line(qfstate_T *state)
{
  listitem_T *p_li = state->p_li;

  // Get the next line from the supplied list
  while (p_li != NULL
         && (TV_LIST_ITEM_TV(p_li)->v_type != VAR_STRING
             || TV_LIST_ITEM_TV(p_li)->vval.v_string == NULL)) {
    p_li = TV_LIST_ITEM_NEXT(state->p_list, p_li);  // Skip non-string items.
  }

  if (p_li == NULL) {  // End of the list.
    state->p_li = NULL;
    return QF_END_OF_INPUT;
  }

  size_t len = strlen(TV_LIST_ITEM_TV(p_li)->vval.v_string);
  if (len > IOSIZE - 2) {
    state->linebuf = qf_grow_linebuf(state, len);
  } else {
    state->linebuf = IObuff;
    state->linelen = len;
  }

  xstrlcpy(state->linebuf, TV_LIST_ITEM_TV(p_li)->vval.v_string,
           state->linelen + 1);

  state->p_li = TV_LIST_ITEM_NEXT(state->p_list, p_li);
  return QF_OK;
}

/// Get the next string from state->buf.
static int qf_get_next_buf_line(qfstate_T *state)
{
  // Get the next line from the supplied buffer
  if (state->buflnum > state->lnumlast) {
    return QF_END_OF_INPUT;
  }
  char *p_buf = ml_get_buf(state->buf, state->buflnum);
  size_t len = (size_t)ml_get_buf_len(state->buf, state->buflnum);
  state->buflnum += 1;

  if (len > IOSIZE - 2) {
    state->linebuf = qf_grow_linebuf(state, len);
  } else {
    state->linebuf = IObuff;
    state->linelen = len;
  }
  xstrlcpy(state->linebuf, p_buf, state->linelen + 1);

  return QF_OK;
}

/// Get the next string from file state->fd.
static int qf_get_next_file_line(qfstate_T *state)
{
retry:
  errno = 0;
  if (fgets(IObuff, IOSIZE, state->fd) == NULL) {
    if (errno == EINTR) {
      goto retry;
    }
    return QF_END_OF_INPUT;
  }

  bool discard = false;
  state->linelen = strlen(IObuff);
  if (state->linelen == IOSIZE - 1
      && !(IObuff[state->linelen - 1] == '\n')) {
    // The current line exceeds IObuff, continue reading using growbuf
    // until EOL or LINE_MAXLEN bytes is read.
    if (state->growbuf == NULL) {
      state->growbufsiz = 2 * (IOSIZE - 1);
      state->growbuf = xmalloc(state->growbufsiz);
    }

    // Copy the read part of the line, excluding null-terminator
    memcpy(state->growbuf, IObuff, IOSIZE - 1);
    size_t growbuflen = state->linelen;

    while (true) {
      errno = 0;
      if (fgets(state->growbuf + growbuflen,
                (int)(state->growbufsiz - growbuflen), state->fd) == NULL) {
        if (errno == EINTR) {
          continue;
        }
        break;
      }
      state->linelen = strlen(state->growbuf + growbuflen);
      growbuflen += state->linelen;
      if (state->growbuf[growbuflen - 1] == '\n') {
        break;
      }
      if (state->growbufsiz == LINE_MAXLEN) {
        discard = true;
        break;
      }

      state->growbufsiz = MIN(2 * state->growbufsiz, LINE_MAXLEN);
      state->growbuf = xrealloc(state->growbuf, state->growbufsiz);
    }

    while (discard) {
      // The current line is longer than LINE_MAXLEN, continue reading but
      // discard everything until EOL or EOF is reached.
      errno = 0;
      if (fgets(IObuff, IOSIZE, state->fd) == NULL) {
        if (errno == EINTR) {
          continue;
        }
        break;
      }
      if (strlen(IObuff) < IOSIZE - 1 || IObuff[IOSIZE - 2] == '\n') {
        break;
      }
    }

    state->linebuf = state->growbuf;
    state->linelen = growbuflen;
  } else {
    state->linebuf = IObuff;
  }

  // Convert a line if it contains a non-ASCII character
  if (state->vc.vc_type != CONV_NONE && has_non_ascii(state->linebuf)) {
    char *line = string_convert(&state->vc, state->linebuf, &state->linelen);
    if (line != NULL) {
      if (state->linelen < IOSIZE) {
        xstrlcpy(state->linebuf, line, state->linelen + 1);
        xfree(line);
      } else {
        xfree(state->growbuf);
        state->linebuf = line;
        state->growbuf = line;
        state->growbufsiz = MIN(state->linelen, LINE_MAXLEN);
      }
    }
  }
  return QF_OK;
}

/// Get the next string from a file/buffer/list/string.
static int qf_get_nextline(qfstate_T *state)
{
  int status = QF_FAIL;

  if (state->fd == NULL) {
    if (state->tv != NULL) {
      if (state->tv->v_type == VAR_STRING) {
        // Get the next line from the supplied string
        status = qf_get_next_str_line(state);
      } else if (state->tv->v_type == VAR_LIST) {
        // Get the next line from the supplied list
        status = qf_get_next_list_line(state);
      }
    } else {
      // Get the next line from the supplied buffer
      status = qf_get_next_buf_line(state);
    }
  } else {
    // Get the next line from the supplied file
    status = qf_get_next_file_line(state);
  }

  if (status != QF_OK) {
    return status;
  }

  if (state->linelen > 0 && state->linebuf[state->linelen - 1] == '\n') {
    state->linebuf[state->linelen - 1] = NUL;
#ifdef USE_CRNL
    if (state->linelen > 1 && state->linebuf[state->linelen - 2] == '\r') {
      state->linebuf[state->linelen - 2] = NUL;
    }
#endif
  }

  remove_bom(state->linebuf);

  return QF_OK;
}

/// Return a pointer to a list in the specified quickfix stack
static qf_list_T *qf_get_list(qf_info_T *qi, int idx)
  FUNC_ATTR_NONNULL_ALL
{
  return &qi->qf_lists[idx];
}

/// Thin wrapper: delegates to rs_qf_parse_line (Rust implementation).
static int qf_parse_line(qf_list_T *qfl, char *linebuf, size_t linelen, efm_T *fmt_first,
                         qffields_T *fields)
{
  return rs_qf_parse_line(qfl, linebuf, linelen, fmt_first, fields);
}

// Allocate the fields used for parsing lines and populating a quickfix list.
static void qf_alloc_fields(qffields_T *pfields)
  FUNC_ATTR_NONNULL_ALL
{
  pfields->namebuf = xmalloc(CMDBUFFSIZE + 1);
  pfields->module = xmalloc(CMDBUFFSIZE + 1);
  pfields->errmsglen = CMDBUFFSIZE + 1;
  pfields->errmsg = xmalloc(pfields->errmsglen);
  pfields->pattern = xmalloc(CMDBUFFSIZE + 1);
}

// Free the fields used for parsing lines and populating a quickfix list.
static void qf_free_fields(qffields_T *pfields)
  FUNC_ATTR_NONNULL_ALL
{
  xfree(pfields->namebuf);
  xfree(pfields->module);
  xfree(pfields->errmsg);
  xfree(pfields->pattern);
}

// Setup the state information used for parsing lines and populating a
// quickfix list.
static int qf_setup_state(qfstate_T *pstate, char *restrict enc, const char *restrict efile,
                          typval_T *tv, buf_T *buf, linenr_T lnumfirst, linenr_T lnumlast)
  FUNC_ATTR_NONNULL_ARG(1)
{
  pstate->vc.vc_type = CONV_NONE;
  if (enc != NULL && *enc != NUL) {
    convert_setup(&pstate->vc, enc, p_enc);
  }

  if (efile != NULL
      && (pstate->fd = (strequal(efile, "-")
                        ? fdopen(os_open_stdin_fd(), "r")
                        : os_fopen(efile, "r"))) == NULL) {
    semsg(_(e_openerrf), efile);
    return FAIL;
  }

  if (tv != NULL) {
    if (tv->v_type == VAR_STRING) {
      pstate->p_str = tv->vval.v_string;
    } else if (tv->v_type == VAR_LIST) {
      pstate->p_li = tv_list_first(tv->vval.v_list);
    }
    pstate->tv = tv;
  }
  pstate->buf = buf;
  pstate->buflnum = lnumfirst;
  pstate->lnumlast = lnumlast;

  return OK;
}

// Cleanup the state information used for parsing lines and populating a
// quickfix list.
static void qf_cleanup_state(qfstate_T *pstate)
  FUNC_ATTR_NONNULL_ALL
{
  if (pstate->fd != NULL) {
    fclose(pstate->fd);
  }
  xfree(pstate->growbuf);
  if (pstate->vc.vc_type != CONV_NONE) {
    convert_setup(&pstate->vc, NULL, NULL);
  }
}

/// Allocate a qffields_T and return it as an opaque handle.
void *nvim_qf_init_alloc_fields(void)
{
  qffields_T *pfields = xcalloc(1, sizeof(qffields_T));
  qf_alloc_fields(pfields);
  return pfields;
}

/// Free a qffields_T allocated by nvim_qf_init_alloc_fields.
void nvim_qf_init_free_fields(void *fields_void)
{
  if (fields_void == NULL) {
    return;
  }
  qffields_T *pfields = (qffields_T *)fields_void;
  qf_free_fields(pfields);
  xfree(pfields);
}

/// Allocate and setup a qfstate_T. Returns NULL on failure.
void *nvim_qf_init_setup_state(char *enc, const char *efile, void *tv_void,
                                void *buf_void, linenr_T lnumfirst, linenr_T lnumlast)
{
  qfstate_T *pstate = xcalloc(1, sizeof(qfstate_T));
  if (qf_setup_state(pstate, enc, efile, (typval_T *)tv_void,
                     (buf_T *)buf_void, lnumfirst, lnumlast) == FAIL) {
    xfree(pstate);
    return NULL;
  }
  return pstate;
}

/// Cleanup and free a qfstate_T.
void nvim_qf_init_cleanup_state(void *state_void)
{
  if (state_void == NULL) {
    return;
  }
  qf_cleanup_state((qfstate_T *)state_void);
  xfree(state_void);
}

void nvim_qf_init_clear_last_bufname(void) { XFREE_CLEAR(qf_last_bufname); }

/// Returns the efm string to use (NOT a copy - do not free).
char *nvim_qf_init_resolve_efm(char *errorformat, void *tv_void, void *buf_void)
{
  typval_T *tv = (typval_T *)tv_void;
  buf_T *buf = (buf_T *)buf_void;
  if (errorformat == p_efm && tv == NULL && buf && *buf->b_p_efm != NUL) {
    return buf->b_p_efm;
  }
  return errorformat;
}

/// Returns NULL if parsing failed.
static efm_T *s_fmt_first = NULL;
static char *s_last_efm = NULL;

void *nvim_qf_init_update_efm_cache(char *efm)
{
  if (s_last_efm == NULL || (strcmp(s_last_efm, efm) != 0)) {
    XFREE_CLEAR(s_last_efm);
    free_efm_list(&s_fmt_first);
    s_fmt_first = parse_efm_option(efm);
    if (s_fmt_first != NULL) {
      s_last_efm = xstrdup(efm);
    }
  }
  return s_fmt_first;
}

/// Process one line during qf_init. Returns QF_OK, QF_END_OF_INPUT, or QF_FAIL.
int nvim_qf_init_process_nextline(void *qfl_void, void *fmt_first_void,
                                   void *state_void, void *fields_void)
{
  return qf_init_process_nextline((qf_list_T *)qfl_void, (efm_T *)fmt_first_void,
                                  (qfstate_T *)state_void, (qffields_T *)fields_void);
}

bool nvim_qf_init_state_no_fd_error(void *state_void) { return ((qfstate_T *)state_void)->fd == NULL || !ferror(((qfstate_T *)state_void)->fd); }

/// Sets qf_ptr, qf_index, and qf_nonevalid based on whether valid entries exist.
void nvim_qf_init_finalize_list(void *qfl_void)
{
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  if (qfl->qf_index == 0) {
    // no valid entry found
    qfl->qf_ptr = qfl->qf_start;
    qfl->qf_index = 1;
    qfl->qf_nonevalid = true;
  } else {
    qfl->qf_nonevalid = false;
    if (qfl->qf_ptr == NULL) {
      qfl->qf_ptr = qfl->qf_start;
    }
  }
}

void nvim_qf_init_emsg_readerrf(void) { emsg(_(e_readerrf)); }

_Static_assert(QF_END_OF_INPUT == 2, "QF_END_OF_INPUT must be 2");
_Static_assert(QF_FAIL == 0, "QF_FAIL must be 0");

/// Returns a pointer to a static buffer with the title.
static char *qf_cmdtitle(char *cmd)
{
  static char qftitle_str[IOSIZE];
  rs_qf_cmdtitle(cmd, qftitle_str, sizeof(qftitle_str));
  return qftitle_str;
}

/// Return a pointer to the current list in the specified quickfix stack
static qf_list_T *qf_get_curlist(qf_info_T *qi)
  FUNC_ATTR_NONNULL_ALL
{
  return qf_get_list(qi, qi->qf_curlist);
}

// =============================================================================
// Phase 5: regmatch_T indexed accessors for Rust parse_match migration
// =============================================================================

/// Return the start pointer for submatch at index idx (0-based, 0-13).
const char *nvim_qf_regmatch_startp(const void *rm, int idx)
{
  if (rm == NULL || idx < 0 || idx >= NSUBEXP) {
    return NULL;
  }
  return ((const regmatch_T *)rm)->startp[idx];
}

/// Return the end pointer for submatch at index idx (0-based, 0-13).
const char *nvim_qf_regmatch_endp(const void *rm, int idx)
{
  if (rm == NULL || idx < 0 || idx >= NSUBEXP) {
    return NULL;
  }
  return ((const regmatch_T *)rm)->endp[idx];
}

/// Expand environment variables in src into dst (dstlen bytes).
void nvim_qf_expand_env(const char *src, char *dst, int dstlen)
{
  if (src != NULL && dst != NULL && dstlen > 0) {
    expand_env((char *)src, dst, dstlen);
  }
}

/// Return true if the path exists on the filesystem.
bool nvim_qf_os_path_exists(const char *path)
{
  return path != NULL && os_path_exists(path);
}

/// Return true if the buffer number bnr refers to a known buffer.
bool nvim_qf_buflist_findnr_exists(int bnr)
{
  return buflist_findnr(bnr) != NULL;
}

// =============================================================================
// Phase 5: qffields_T accessors for Rust
// =============================================================================

char *nvim_qf_fields_get_namebuf(void *fields) { return fields == NULL ? NULL : ((qffields_T *)fields)->namebuf; }
int nvim_qf_fields_get_bnr(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->bnr; }
void nvim_qf_fields_set_bnr(void *fields, int bnr) { if (fields != NULL) ((qffields_T *)fields)->bnr = bnr; }
char *nvim_qf_fields_get_module(void *fields) { return fields == NULL ? NULL : ((qffields_T *)fields)->module; }
char *nvim_qf_fields_get_errmsg(void *fields) { return fields == NULL ? NULL : ((qffields_T *)fields)->errmsg; }
size_t nvim_qf_fields_get_errmsglen(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->errmsglen; }
void nvim_qf_fields_set_errmsg(void *fields, const char *msg, size_t len)
{
  if (fields == NULL || msg == NULL) {
    return;
  }
  qffields_T *f = (qffields_T *)fields;
  if (len >= f->errmsglen) {
    f->errmsg = xrealloc(f->errmsg, len + 1);
    f->errmsglen = len + 1;
  }
  xstrlcpy(f->errmsg, msg, len + 1);
}
linenr_T nvim_qf_fields_get_lnum(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->lnum; }
void nvim_qf_fields_set_lnum(void *fields, linenr_T lnum) { if (fields != NULL) ((qffields_T *)fields)->lnum = lnum; }
linenr_T nvim_qf_fields_get_end_lnum(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->end_lnum; }
void nvim_qf_fields_set_end_lnum(void *fields, linenr_T end_lnum) { if (fields != NULL) ((qffields_T *)fields)->end_lnum = end_lnum; }
int nvim_qf_fields_get_col(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->col; }
void nvim_qf_fields_set_col(void *fields, int col) { if (fields != NULL) ((qffields_T *)fields)->col = col; }
int nvim_qf_fields_get_end_col(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->end_col; }
void nvim_qf_fields_set_end_col(void *fields, int end_col) { if (fields != NULL) ((qffields_T *)fields)->end_col = end_col; }
bool nvim_qf_fields_get_use_viscol(const void *fields) { return fields == NULL ? false : ((const qffields_T *)fields)->use_viscol; }
void nvim_qf_fields_set_use_viscol(void *fields, bool use_viscol) { if (fields != NULL) ((qffields_T *)fields)->use_viscol = use_viscol; }
char *nvim_qf_fields_get_pattern(void *fields) { return fields == NULL ? NULL : ((qffields_T *)fields)->pattern; }
int nvim_qf_fields_get_enr(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->enr; }
void nvim_qf_fields_set_enr(void *fields, int enr) { if (fields != NULL) ((qffields_T *)fields)->enr = enr; }
char nvim_qf_fields_get_type(const void *fields) { return fields == NULL ? 0 : ((const qffields_T *)fields)->type; }
void nvim_qf_fields_set_type(void *fields, char type) { if (fields != NULL) ((qffields_T *)fields)->type = type; }
bool nvim_qf_fields_get_valid(const void *fields) { return fields == NULL ? false : ((const qffields_T *)fields)->valid; }
void nvim_qf_fields_set_valid(void *fields, bool valid) { if (fields != NULL) ((qffields_T *)fields)->valid = valid; }

// =============================================================================
// Phase 5: Phase 2 accessors - efm prog, regmatch lifecycle, qfline text append
// =============================================================================

/// Get the regprog from an efm_T (returns opaque pointer).
void *nvim_efm_get_prog(void *efm)
{
  return efm == NULL ? NULL : ((efm_T *)efm)->prog;
}

/// Set the regprog on an efm_T (after vim_regexec may update it).
void nvim_efm_set_prog(void *efm, void *prog)
{
  if (efm != NULL) {
    ((efm_T *)efm)->prog = (regprog_T *)prog;
  }
}

/// Create a regmatch_T on the heap, set rm_ic=true, and assign the given prog.
/// Returns an opaque handle. The caller owns the memory; free with nvim_qf_regmatch_free.
void *nvim_qf_regmatch_create_ic(void *prog)
{
  regmatch_T *rm = xcalloc(1, sizeof(regmatch_T));
  rm->rm_ic = true;
  rm->regprog = (regprog_T *)prog;
  return rm;
}

/// Extract the regprog from a regmatch_T and free the regmatch_T struct.
/// Returns the prog (which may have been updated by vim_regexec).
void *nvim_qf_regmatch_extract_prog(void *rm_void)
{
  if (rm_void == NULL) {
    return NULL;
  }
  regmatch_T *rm = (regmatch_T *)rm_void;
  void *prog = rm->regprog;
  xfree(rm);
  return prog;
}

/// Execute vim_regexec using the regmatch_T handle (already has prog set).
/// Returns true if the regex matches the line.
bool nvim_qf_vim_regexec(void *rm_void, const char *line)
{
  if (rm_void == NULL || line == NULL) {
    return false;
  }
  return vim_regexec((regmatch_T *)rm_void, line, 0);
}

/// Append text to qfline_T.qf_text with a newline separator.
/// Handles the xrealloc + newline + STRCPY pattern safely.
void nvim_qfline_append_text(void *qfp_void, const char *text)
{
  if (qfp_void == NULL || text == NULL || *text == NUL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  size_t textlen = qfp->qf_text != NULL ? strlen(qfp->qf_text) : 0;
  size_t errlen = strlen(text);
  qfp->qf_text = xrealloc(qfp->qf_text, textlen + errlen + 2);
  qfp->qf_text[textlen] = '\n';
  STRCPY(qfp->qf_text + textlen + 1, text);
}

/// Wrapper for line_breakcheck().
void nvim_qf_line_breakcheck(void)
{
  line_breakcheck();
}

/// Call vim_isprintc() - returns nonzero if the char is printable.
int nvim_qf_vim_isprintc(int c)
{
  return vim_isprintc(c);
}

/// Get the qf_get_fnum result for a qfl + namebuf/currfile/valid.
/// Handles the complex conditional logic for directory + currfile selection.
int nvim_qf_get_fnum_for_fields(void *qfl_void, void *fields_void)
{
  if (qfl_void == NULL || fields_void == NULL) {
    return 0;
  }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  qffields_T *fields = (qffields_T *)fields_void;
  return qf_get_fnum(qfl, qfl->qf_directory,
                     *fields->namebuf || qfl->qf_directory
                     ? fields->namebuf
                     : qfl->qf_currfile && fields->valid
                     ? qfl->qf_currfile : 0);
}

/// Move memory: STRMOVE(dst, src) - move overlapping memory.
void nvim_qf_strmove(char *dst, const char *src)
{
  if (dst != NULL && src != NULL) {
    STRMOVE(dst, src);
  }
}

/// Get IObuff pointer for reuse in file_pfx multiscan.
char *nvim_qf_get_iobuff(void) { return IObuff; }

/// skipwhite wrapper.
const char *nvim_qf_skipwhite(const char *p)
{
  return p == NULL ? NULL : skipwhite(p);
}

/// Emit error E379 (missing directory name).
void nvim_qf_emsg_missing_dir(void)
{
  emsg(_("E379: Missing or empty directory name"));
}

// qf_parse_fmt_f and all qf_parse_fmt_* functions deleted: migrated to Rust rs_qf_parse_match.

// All qf_parse_fmt_* functions, copy_nonerror_line, qf_parse_match, qf_parse_get_fields,
// qf_parse_dir_pfx, qf_parse_file_pfx, qf_parse_line_nomatch, and qf_parse_multiline_pfx
// have been deleted. They are now implemented in Rust in src/nvim-rs/quickfix/src/parse.rs
// as rs_qf_parse_match and helpers called from rs_qf_parse_line.

// locstack_queue_delreq deleted: migrated to Rust rs_locstack_queue_delreq in lifecycle.rs.

int qf_stack_get_bufnr(void) { assert(ql_info != NULL); return ql_info->qf_bufnr; }

// wipe_qf_buffer, ll_free_all deleted: migrated to Rust in lifecycle.rs.
// incr_quickfix_busy, decr_quickfix_busy, check_quickfix_busy deleted: migrated to Rust in lifecycle.rs.

/// Free all lists in a qf_info_T and the struct itself.
/// (Was the static C qf_free_lists; now delegates to C accessor helpers that
/// mirror what the Rust qf_free_lists_and_info function does.)
static void qf_free_lists(qf_info_T *qi)
{
  for (int i = 0; i < qi->qf_listcount; i++) {
    rs_qf_free_list(qf_get_list(qi, i));
  }
  nvim_qf_free_lists_array((void *)qi);
  nvim_qf_free_info((void *)qi);
}

/// Thin C wrapper: forward to Rust rs_ll_free_all.
/// Kept for callers inside this file that pass a qf_info_T** directly.
static void ll_free_all(qf_info_T **pqi) { rs_ll_free_all((void **)pqi); }

/// Free all the quickfix/location lists in the stack.
/// Thin wrapper calling Rust rs_qf_free_all.
void qf_free_all(win_T *wp) { rs_qf_free_all((void *)wp); }

static void incr_quickfix_busy(void) { rs_incr_quickfix_busy(); }
static void decr_quickfix_busy(void) { rs_decr_quickfix_busy(); }

#if defined(EXITFREE)
void check_quickfix_busy(void) { rs_check_quickfix_busy(); }
#endif

void qf_resize_stack(int n) { assert(ql_info != NULL); qf_resize_stack_base(ql_info, n); }

/// Resize location list stack for window 'wp' to be able to hold n amount of lists.
void ll_resize_stack(win_T *wp, int n)
{
  // check if given window is a location list window;
  // if so then sync its 'lhistory' to the parent window or vice versa
  if (IS_LL_WINDOW(wp)) {
    qf_sync_llw_to_win(wp);
  } else {
    qf_sync_win_to_llw(wp);
  }

  qf_info_T *qi = ll_get_or_alloc_list(wp);
  qf_resize_stack_base(qi, n);
}

/// Resize quickfix/location lists stack to be able to hold n amount of lists.
static void qf_resize_stack_base(qf_info_T *qi, int n)
  FUNC_ATTR_NONNULL_ALL
{
  int amount_to_rm = 0;
  size_t lsz = sizeof(*qi->qf_lists);

  if (n == qi->qf_maxcount) {
    return;
  } else if (n < qi->qf_maxcount && n < qi->qf_listcount) {
    // We have too many lists to store them all in the new stack,
    // pop lists until we can fit them all in the newly resized stack
    amount_to_rm = qi->qf_listcount - n;

    for (int i = 0; i < amount_to_rm; i++) {
      rs_qf_pop_stack(qi, true);
    }
  }

  qf_list_T *new = xrealloc(qi->qf_lists, lsz * (size_t)n);

  // fill with zeroes any newly allocated memory
  if (n > qi->qf_maxcount) {
    memset(new + qi->qf_maxcount, 0, lsz * (size_t)(n - qi->qf_maxcount));
  }

  qi->qf_lists = new;
  qi->qf_maxcount = n;

  qf_update_buffer(qi, NULL);
}

void qf_init_stack(void) { ql_info = qf_alloc_stack(QFLT_QUICKFIX, (int)p_chi); }

/// Sync a location list window's 'lhistory' value to the parent window
static void qf_sync_llw_to_win(win_T *llw)
{
  win_T *wp = qf_find_win_with_loclist(llw->w_llist_ref);

  if (wp != NULL) {
    wp->w_p_lhi = llw->w_p_lhi;
  }
}

/// Sync a window's 'lhistory' value to its location list window, if any
static void qf_sync_win_to_llw(win_T *pwp)
{
  qf_info_T *llw = pwp->w_llist;

  if (llw != NULL) {
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_llist_ref == llw && bt_quickfix(wp->w_buffer)) {
        wp->w_p_lhi = pwp->w_p_lhi;
        return;
      }
    }
  }
}

// qf_alloc_stack, qf_alloc_list_stack, ll_get_or_alloc_list,
// qf_cmd_get_stack, qf_cmd_get_or_alloc_stack deleted: migrated to Rust in lifecycle.rs.

/// Thin C wrapper for callers inside this file.
static qf_info_T *qf_alloc_stack(qfltype_T qfltype, int n)
{
  return (qf_info_T *)rs_qf_alloc_stack((int)qfltype, n);
}

/// Thin C wrapper for callers inside this file.
static qf_info_T *ll_get_or_alloc_list(win_T *wp)
{
  return (qf_info_T *)rs_ll_get_or_alloc_list((void *)wp);
}

/// Thin C wrapper for callers inside this file.
static qf_info_T *qf_cmd_get_stack(exarg_T *eap, bool print_emsg)
{
  return (qf_info_T *)rs_qf_cmd_get_stack((void *)eap, print_emsg);
}

/// Thin C wrapper for callers inside this file.
static qf_info_T *qf_cmd_get_or_alloc_stack(const exarg_T *eap, win_T **pwinp)
{
  return (qf_info_T *)rs_qf_cmd_get_or_alloc_stack((const void *)eap, (void **)pwinp);
}


// Copy the location list stack 'from' window to 'to' window.
void copy_loclist_stack(win_T *from, win_T *to)
  FUNC_ATTR_NONNULL_ALL
{
  // When copying from a location list window, copy the referenced
  // location list. For other windows, copy the location list for
  // that window.
  qf_info_T *qi = IS_LL_WINDOW(from) ? from->w_llist_ref : from->w_llist;

  if (qi == NULL) {                 // no location list to copy
    return;
  }

  // allocate a new location list, set size of stack to 'from' window value
  to->w_llist = qf_alloc_stack(QFLT_LOCATION, (int)from->w_p_lhi);
  // set 'to' lhi to reflect new value
  to->w_p_lhi = to->w_llist->qf_maxcount;

  to->w_llist->qf_listcount = qi->qf_listcount;

  // Copy the location lists one at a time
  for (int idx = 0; idx < qi->qf_listcount; idx++) {
    to->w_llist->qf_curlist = idx;

    if (rs_copy_loclist(qf_get_list(qi, idx),
                        qf_get_list(to->w_llist, idx)) == FAIL) {
      qf_free_all(to);
      return;
    }
  }

  to->w_llist->qf_curlist = qi->qf_curlist;  // current list
}

/// Also sets the b_has_qf_entry flag.
static int qf_get_fnum(qf_list_T *qfl, char *directory, char *fname)
{
  char *ptr = NULL;
  char *bufname;
  buf_T *buf;
  if (fname == NULL || *fname == NUL) {         // no file name
    return 0;
  }

#ifdef BACKSLASH_IN_FILENAME
  if (directory != NULL) {
    slash_adjust(directory);
  }
  slash_adjust(fname);
#endif
  if (directory != NULL && !vim_isAbsName(fname)) {
    ptr = concat_fnames(directory, fname, true);
    // Here we check if the file really exists.
    // This should normally be true, but if make works without
    // "leaving directory"-messages we might have missed a
    // directory change.
    if (!os_path_exists(ptr)) {
      xfree(ptr);
      directory = (char *)rs_qf_guess_filepath(qfl, fname);
      if (directory) {
        ptr = concat_fnames(directory, fname, true);
      } else {
        ptr = xstrdup(fname);
      }
    }
    // Use concatenated directory name and file name.
    bufname = ptr;
  } else {
    bufname = fname;
  }

  if (qf_last_bufname != NULL
      && strcmp(bufname, qf_last_bufname) == 0
      && bufref_valid(&qf_last_bufref)) {
    buf = qf_last_bufref.br_buf;
    xfree(ptr);
  } else {
    xfree(qf_last_bufname);
    buf = buflist_new(bufname, NULL, 0, BLN_NOOPT);
    qf_last_bufname = (bufname == ptr) ? bufname : xstrdup(bufname);
    set_bufref(&qf_last_bufref, buf);
  }
  if (buf == NULL) {
    return 0;
  }
  buf->b_has_qf_entry =
    IS_QF_LIST(qfl) ? BUF_HAS_QF_ENTRY : BUF_HAS_LL_ENTRY;
  return buf->b_fnum;
}



// Find a window displaying a Vim help file in the current tab page.
static win_T *qf_find_help_win(void)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (bt_help(wp->w_buffer) && !wp->w_config.hide && wp->w_config.focusable) {
      return wp;
    }
  }
  return NULL;
}

static void win_set_loclist(win_T *wp, qf_info_T *qi) { wp->w_llist = qi; qi->qf_refcount++; }

// Rust exports for jump machinery
extern void rs_qf_jump_newwin(void *qi, int dir, int errornr, int forceit, bool newwin);

/// Returns NULL if a matching window is not found.
static win_T *qf_find_win_with_loclist(const qf_info_T *ll)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_llist == ll && !bt_quickfix(wp->w_buffer)) {
      return wp;
    }
  }
  return NULL;
}

/// Find a window containing a normal buffer in the current tab page.
static win_T *qf_find_win_with_normal_buf(void)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (bt_normal(wp->w_buffer)) {
      return wp;
    }
  }
  return NULL;
}

// Go to a window in any tabpage containing the specified file.  Returns true
// if successfully jumped to the window. Otherwise returns false.
static bool qf_goto_tabwin_with_file(int fnum)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer->b_fnum == fnum) {
      goto_tabpage_win(tp, wp);
      return true;
    }
  }
  return false;
}

/// post-validation), or -2 if can_abandon failed (skip post-validation).
int nvim_qf_jump_open_help(int qf_fnum, int forceit, int prev_winid)
{
  if (!can_abandon(curbuf, forceit)) {
    no_write_message();
    return -2;  // sentinel: skip post-validation checks
  }
  return do_ecmd(qf_fnum, NULL, NULL, NULL, 1,
                 ECMD_HIDE + ECMD_SET_HELP,
                 prev_winid == curwin->handle ? curwin : NULL);
}

/// Sets *opened_window if a new window was split.
int nvim_qf_jump_open_file(void *qi_void, int fnum, int forceit, bool *opened_window)
{
  qf_info_T *qi = (qf_info_T *)qi_void;
  int retval = OK;

  if (!forceit && curwin->w_p_wfb && curbuf->b_fnum != fnum) {
    if (qi->qfl_type == QFLT_LOCATION) {
      emsg(_(e_winfixbuf_cannot_go_to_buffer));
      return -2;  // sentinel: location list winfixbuf early return
    }

    if (rs_win_valid(prevwin) && !prevwin->w_p_wfb
        && !bt_quickfix(prevwin->w_buffer)) {
      win_goto(prevwin);
    }
    if (curwin->w_p_wfb) {
      if (win_split(0, 0) == OK) {
        *opened_window = true;
      }
      if (curwin->w_p_wfb) {
        emsg(_(e_winfixbuf_cannot_go_to_buffer));
        retval = FAIL;
      }
    }
  }

  if (retval == OK) {
    retval = buflist_getfile(fnum, 1, GETF_SETMARK | GETF_SWITCH, forceit);
  }
  return retval;
}

/// Check if the location list window was closed. Returns true if invalid.
bool nvim_qf_jump_loc_win_closed(int prev_winid, void *qi_void)
{
  win_T *wp = win_id2wp(prev_winid);
  qf_info_T *qi = (qf_info_T *)qi_void;
  return wp == NULL && curwin->w_llist != qi;
}

void nvim_qf_jump_emsg_win_closed(void) { emsg(_("E924: Current window was closed")); }

void nvim_qf_jump_emsg_qf_changed(void) { emsg(_(e_current_quickfix_list_was_changed)); }

void nvim_qf_jump_emsg_ll_changed(void) { emsg(_(e_current_location_list_was_changed)); }

_Static_assert(QFLT_QUICKFIX == 0, "QFLT_QUICKFIX must be 0");
_Static_assert(QFLT_LOCATION == 1, "QFLT_LOCATION must be 1");
_Static_assert(QF_ABORT == 6, "QF_ABORT must be 6");

/// Find a help window in the current tab. Returns win handle or NULL.
void *nvim_qf_find_help_win(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (bt_help(wp->w_buffer) && !wp->w_config.hide && wp->w_config.focusable) {
      return wp;
    }
  }
  return NULL;
}

/// Find a non-quickfix window using the given location list. Returns win handle or NULL.
void *nvim_qf_find_win_with_loclist(const void *ll)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_llist == (qf_info_T *)ll && !bt_quickfix(wp->w_buffer)) {
      return wp;
    }
  }
  return NULL;
}

/// Find a window containing a normal buffer in the current tab. Returns win handle or NULL.
void *nvim_qf_find_win_with_normal_buf(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (bt_normal(wp->w_buffer)) {
      return wp;
    }
  }
  return NULL;
}

/// Returns true if successfully jumped.
bool nvim_qf_goto_tabwin_with_file(int fnum)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer->b_fnum == fnum) {
      goto_tabpage_win(tp, wp);
      return true;
    }
  }
  return false;
}

/// ll_ref may be NULL. Returns OK or FAIL.
int nvim_qf_open_new_file_win(void *ll_ref)
{
  int flags = WSP_ABOVE;
  if (ll_ref != NULL) {
    flags |= WSP_NEWLOC;
  }
  if (win_split(0, flags) == FAIL) {
    return FAIL;
  }
  p_swb = empty_string_option;
  swb_flags = 0;
  RESET_BINDING(curwin);
  if (ll_ref != NULL) {
    win_set_loclist(curwin, (qf_info_T *)ll_ref);
  }
  return OK;
}

void *nvim_qf_curwin_get_llist_ref(void) { return curwin->w_llist_ref; }

bool nvim_qf_curbuf_is_quickfix(void) { return bt_quickfix(curbuf); }

bool nvim_qf_curwin_buf_is_help(void) { return bt_help(curwin->w_buffer); }

int nvim_qf_get_cmdmod_tab(void) { return cmdmod.cmod_tab; }

bool nvim_qf_is_one_window(void) { return ONE_WINDOW; }

bool nvim_qf_swb_has_usetab(void) { return (swb_flags & kOptSwbFlagUsetab) != 0; }

int nvim_qf_curwin_handle(void) { return curwin->handle; }

void nvim_qf_win_close_curwin(void) { win_close(curwin, true, false); }

void nvim_qf_win_goto(void *win) { win_goto((win_T *)win); }

void nvim_qf_win_enter(void *win) { win_enter((win_T *)win, true); }

int nvim_qf_win_buf_nwindows(const void *win) { return ((const win_T *)win)->w_buffer->b_nwindows; }

int nvim_qf_win_buf_fnum(const void *win) { return ((const win_T *)win)->w_buffer->b_fnum; }

void *nvim_qf_win_get_llist(const void *win) { return ((const win_T *)win)->w_llist; }

void nvim_qf_win_set_loclist(void *win, void *qi) { win_set_loclist((win_T *)win, (qf_info_T *)qi); }

int nvim_qf_get_cmdmod_split(void) { return cmdmod.cmod_split; }

int nvim_qf_curwin_width(void) { return curwin->w_width; }

int nvim_qf_get_columns(void) { return Columns; }

int nvim_qf_curwin_height(void) { return curwin->w_height; }

int nvim_qf_get_p_hh(void) { return (int)p_hh; }

int nvim_qf_win_split(int size, int flags) { return win_split(size, flags); }

void nvim_qf_clear_restart_edit(void) { restart_edit = 0; }

bool nvim_qf_is_ll_stack_qi(const void *qi) { return IS_LL_STACK((const qf_info_T *)qi); }

bool nvim_qf_win_is_qf_window(const void *win) { return IS_QF_WINDOW((const win_T *)win); }

void *nvim_qf_win_prev(const void *win) { return ((const win_T *)win)->w_prev; }

void *nvim_qf_win_next(const void *win) { return ((const win_T *)win)->w_next; }

void *nvim_qf_get_lastwin(void) { return lastwin; }

void *nvim_qf_get_curwin(void) { return curwin; }

bool nvim_qf_win_bt_normal(const void *win) { return bt_normal(((const win_T *)win)->w_buffer); }

bool nvim_qf_swb_uselast_prevwin_ok(void) { return (swb_flags & kOptSwbFlagUselast) && rs_win_valid(prevwin) && !prevwin->w_p_wfb; }

void *nvim_qf_get_prevwin(void) { return prevwin; }

bool nvim_qf_win_is_preview(const void *win) { return ((const win_T *)win)->w_p_pvw; }

bool nvim_qf_win_is_wfb(const void *win) { return ((const win_T *)win)->w_p_wfb; }

/// qf_pattern may be NULL (use line/col instead).
void nvim_qf_jump_goto_line(linenr_T qf_lnum, int qf_col, char qf_viscol, const char *qf_pattern)
{
  if (qf_pattern == NULL) {
    // Go to line with error, unless qf_lnum is 0.
    linenr_T i = qf_lnum;
    if (i > 0) {
      i = MIN(i, curbuf->b_ml.ml_line_count);
      curwin->w_cursor.lnum = i;
    }
    if (qf_col > 0) {
      curwin->w_cursor.coladd = 0;
      if (qf_viscol == true) {
        coladvance(curwin, qf_col - 1);
      } else {
        curwin->w_cursor.col = qf_col - 1;
      }
      curwin->w_set_curswant = true;
      check_cursor(curwin);
    } else {
      beginline(BL_WHITE | BL_FIX);
    }
  } else {
    // Move the cursor to the first line in the buffer
    pos_T save_cursor = curwin->w_cursor;
    curwin->w_cursor.lnum = 0;
    if (!do_search(NULL, '/', '/', (char *)qf_pattern, strlen(qf_pattern), 1, SEARCH_KEEP,
                   NULL)) {
      curwin->w_cursor = save_cursor;
    }
  }
}

/// Print the "(N of M)" quickfix jump status message.
void nvim_qf_jump_print_msg(void *qi_void, int qf_index, void *qf_ptr_void,
                             void *old_curbuf_void, linenr_T old_lnum)
{
  qf_info_T *qi = (qf_info_T *)qi_void;
  qfline_T *qf_ptr = (qfline_T *)qf_ptr_void;
  buf_T *old_curbuf = (buf_T *)old_curbuf_void;

  garray_T *const gap = qfga_get();

  // Update the screen before showing the message, unless messages scrolled.
  if (!msg_scrolled) {
    update_topline(curwin);
    if (must_redraw) {
      update_screen();
    }
  }
  char qf_types_buf[20];
  vim_snprintf(IObuff, IOSIZE, _("(%d of %d)%s%s: "), qf_index,
               qf_get_curlist(qi)->qf_count,
               qf_ptr->qf_cleared ? _(" (line deleted)") : "",
               rs_qf_types(qf_ptr->qf_type, qf_ptr->qf_nr, qf_types_buf, sizeof(qf_types_buf)));
  // Add the message, skipping leading whitespace and newlines.
  ga_concat(gap, IObuff);
  char fmt_buf[IOSIZE];
  size_t fmt_len = rs_qf_fmt_text(skipwhite(qf_ptr->qf_text), fmt_buf, sizeof(fmt_buf));
  ga_concat_len(gap, fmt_buf, fmt_len);
  ga_append(gap, NUL);

  // Output the message.  Overwrite to avoid scrolling when the 'O'
  // flag is present in 'shortmess'; But when not jumping, print the
  // whole message.
  linenr_T i = msg_scroll;
  if (curbuf == old_curbuf && curwin->w_cursor.lnum == old_lnum) {
    msg_scroll = true;
  } else if ((msg_scrolled == 0 || (p_ch == 0 && msg_scrolled == 1))
             && shortmess(SHM_OVERALL)) {
    msg_scroll = false;
  }
  msg_ext_set_kind("quickfix");
  msg_keep(gap->ga_data, 0, true, false);
  msg_scroll = (int)i;

  qfga_clear();
}

void *nvim_qf_get_curbuf(void) { return curbuf; }

bool nvim_qf_fdo_quickfix(void) { return (fdo_flags & kOptFdoFlagQuickfix) != 0; }

void nvim_qf_setpcmark(void) { setpcmark(); }

bool nvim_qf_curbuf_is(const void *buf) { return curbuf == (const buf_T *)buf; }

void *nvim_qf_get_p_swb(void) { return p_swb; }

unsigned nvim_qf_get_swb_flags(void) { return swb_flags; }

/// Restore p_swb if it was changed to empty_string_option
void nvim_qf_restore_swb(void *old_swb, unsigned old_swb_flags)
{
  if (p_swb != (char *)old_swb && p_swb == empty_string_option) {
    p_swb = (char *)old_swb;
    swb_flags = old_swb_flags;
  }
}

void nvim_qf_win_close(void *win_void) { if (win_void != NULL) win_close((win_T *)win_void, false, false); }
void nvim_qf_win_goto_lnum(void *win_void, linenr_T lnum) { if (win_void != NULL) qf_win_goto((win_T *)win_void, lnum); }
linenr_T nvim_qf_win_get_cursor_lnum(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_cursor.lnum; }
linenr_T nvim_qf_win_get_buf_line_count(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_buffer->b_ml.ml_line_count; }
int nvim_qf_win_get_width(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_width; }
int nvim_qf_win_get_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_height; }
int nvim_qf_win_get_hsep_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_hsep_height; }
int nvim_qf_win_get_status_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_status_height; }

int nvim_qf_cmdline_row(void) { return (int)cmdline_row; }


int nvim_qf_open_new_cwindow(void *qi_void, int height) { return qi_void == NULL ? FAIL : qf_open_new_cwindow((qf_info_T *)qi_void, height); }
void nvim_qf_set_title_var(void *qfl_void) { if (qfl_void != NULL) qf_set_title_var((qf_list_T *)qfl_void); }

void nvim_qf_curwin_set_cursor(linenr_T lnum, int col) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; }

void nvim_qf_check_cursor_curwin(void) { check_cursor(curwin); }

void nvim_qf_update_topline_curwin(void) { update_topline(curwin); }

void nvim_qf_update_win_titlevar(void *qi_void) { if (qi_void != NULL) qf_update_win_titlevar((qf_info_T *)qi_void); }

// Highlight ids used for displaying entries from the quickfix list.
static int qfFile_hl_id;
static int qfSep_hl_id;
static int qfLine_hl_id;

/// quickfix list.
static void qf_list_entry(qfline_T *qfp, int qf_idx, bool cursel)
{
  rs_qf_list_entry(qfp, qf_idx, cursel, qfFile_hl_id, qfSep_hl_id, qfLine_hl_id);
}

// ":clist": list all errors
// ":llist": list all locations
void qf_list(exarg_T *eap)
{
  rs_ex_clist(eap);
}

/// quickfix/location list.

/// into that buffer, or NULL to check the quickfix list.
bool qf_mark_adjust(buf_T *buf, win_T *wp, linenr_T line1, linenr_T line2, linenr_T amount,
                    linenr_T amount_after)
{
  return rs_qf_mark_adjust_entry(buf, wp, line1, line2, amount, amount_after);
}

// Set options for the buffer in the quickfix or location list window.
static void qf_set_cwindow_options(void)
{
  // switch off 'swapfile'
  set_option_value_give_err(kOptSwapfile, BOOLEAN_OPTVAL(false), OPT_LOCAL);
  set_option_value_give_err(kOptBuftype, STATIC_CSTR_AS_OPTVAL("quickfix"), OPT_LOCAL);
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("hide"), OPT_LOCAL);
  RESET_BINDING(curwin);
  curwin->w_p_diff = false;
  set_option_value_give_err(kOptFoldmethod, STATIC_CSTR_AS_OPTVAL("manual"), OPT_LOCAL);
}

// Open a new quickfix or location list window, load the quickfix buffer and
// set the appropriate options for the window.
// Returns FAIL if the window could not be opened.
static int qf_open_new_cwindow(qf_info_T *qi, int height)
  FUNC_ATTR_NONNULL_ALL
{
  win_T *oldwin = curwin;
  const tabpage_T *const prevtab = curtab;
  int flags = 0;

  const buf_T *const qf_buf = qf_find_buf(qi);

  // The current window becomes the previous window afterwards.
  win_T *const win = curwin;

  // Default is to open the window below the current window or at the bottom,
  // except when :belowright or :aboveleft is used.
  if (cmdmod.cmod_split == 0) {
    flags = IS_QF_STACK(qi) ? WSP_BOT : WSP_BELOW;
  }
  flags |= WSP_NEWLOC;
  if (win_split(height, flags) == FAIL) {
    return FAIL;  // not enough room for window
  }
  RESET_BINDING(curwin);

  if (IS_LL_STACK(qi)) {
    // For the location list window, create a reference to the
    // location list stack from the window 'win'.
    curwin->w_llist_ref = qi;
    qi->qf_refcount++;
  }

  if (oldwin != curwin) {
    oldwin = NULL;  // don't store info when in another window
  }
  if (qf_buf != NULL) {
    // Use the existing quickfix buffer
    if (do_ecmd(qf_buf->b_fnum, NULL, NULL, NULL, ECMD_ONE,
                ECMD_HIDE + ECMD_OLDBUF + ECMD_NOWINENTER, oldwin) == FAIL) {
      return FAIL;
    }
  } else {
    // Create a new quickfix buffer
    if (do_ecmd(0, NULL, NULL, NULL, ECMD_ONE, ECMD_HIDE + ECMD_NOWINENTER, oldwin) == FAIL) {
      return FAIL;
    }
    // save the number of the new buffer
    qi->qf_bufnr = curbuf->b_fnum;
  }

  // Set the options for the quickfix buffer/window (if not already done)
  // Do this even if the quickfix buffer was already present, as an autocmd
  // might have previously deleted (:bdelete) the quickfix buffer.
  if (!bt_quickfix(curbuf)) {
    qf_set_cwindow_options();
  }

  // Only set the height when still in the same tab page and there is no
  // window to the side.
  if (curtab == prevtab && curwin->w_width == Columns) {
    rs_win_setheight(height);
  }
  curwin->w_p_wfh = true;  // set 'winfixheight'
  if (rs_win_valid(win)) {
    prevwin = win;
  }
  return OK;
}

/// Set "w:quickfix_title" if "qi" has a title.
static void qf_set_title_var(qf_list_T *qfl)
{
  if (qfl->qf_title != NULL) {
    set_internal_string_var("w:quickfix_title", qfl->qf_title);
  }
}

// Move the cursor in the quickfix window to "lnum".
static void qf_win_goto(win_T *win, linenr_T lnum)
{
  win_T *old_curwin = curwin;

  curwin = win;
  curbuf = win->w_buffer;
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = 0;
  curwin->w_cursor.coladd = 0;
  curwin->w_curswant = 0;
  update_topline(curwin);              // scroll to show the line
  redraw_later(curwin, UPD_VALID);
  curwin->w_redr_status = true;  // update ruler
  curwin = old_curwin;
  curbuf = curwin->w_buffer;
}

// Return the number of the current entry (line number in the quickfix
// window).
linenr_T qf_current_entry(win_T *wp)
{
  return rs_qf_current_entry(wp);
}

linenr_T nvim_qf_current_entry(win_T *wp) { return qf_current_entry(wp); }

/// @param old_qf_index  previous qf_index or zero
static bool qf_win_pos_update(qf_info_T *qi, int old_qf_index)
{
  qf_list_T *qfl = qf_get_curlist(qi);
  int qf_index = qfl->qf_index;

  // Put the cursor on the current error in the quickfix window, so that
  // it's viewable.
  win_T *win = qf_find_win(qi);
  if (win != NULL
      && qf_index <= win->w_buffer->b_ml.ml_line_count
      && rs_qf_should_update_cursor(qfl, old_qf_index)) {
    win->w_redraw_top = MIN(old_qf_index, qf_index);
    win->w_redraw_bot = MAX(old_qf_index, qf_index);
    qf_win_goto(win, qf_index);
  }
  return win != NULL;
}

/// quickfix/location stack.
static int is_qf_win(const win_T *win, const qf_info_T *qi)
  FUNC_ATTR_NONNULL_ARG(2) FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  // A window displaying the quickfix buffer will have the w_llist_ref field
  // set to NULL.
  // A window displaying a location list buffer will have the w_llist_ref
  // pointing to the location list.
  if (buf_valid(win->w_buffer) && bt_quickfix(win->w_buffer)) {
    if ((IS_QF_STACK(qi) && win->w_llist_ref == NULL)
        || (IS_LL_STACK(qi) && win->w_llist_ref == qi)) {
      return true;
    }
  }

  return false;
}

/// page.
static win_T *qf_find_win(const qf_info_T *qi)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  FOR_ALL_WINDOWS_IN_TAB(win, curtab) {
    if (is_qf_win(win, qi)) {
      return win;
    }
  }

  return NULL;
}

/// Searches in windows opened in all the tab pages.
static buf_T *qf_find_buf(qf_info_T *qi)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (qi->qf_bufnr != INVALID_QFBUFNR) {
    buf_T *const qfbuf = buflist_findnr(qi->qf_bufnr);
    if (qfbuf != NULL) {
      return qfbuf;
    }
    // buffer is no longer present
    qi->qf_bufnr = INVALID_QFBUFNR;
  }

  FOR_ALL_TAB_WINDOWS(tp, win) {
    if (is_qf_win(win, qi)) {
      return win->w_buffer;
    }
  }

  return NULL;
}

/// Process the 'quickfixtextfunc' option value.
const char *did_set_quickfixtextfunc(optset_T *args FUNC_ATTR_UNUSED)
{
  if (option_set_callback_func(p_qftf, &qftf_cb) == FAIL) {
    return e_invarg;
  }
  return NULL;
}

/// all the tab pages.
static void qf_update_win_titlevar(qf_info_T *qi)
  FUNC_ATTR_NONNULL_ALL
{
  qf_list_T *const qfl = qf_get_curlist(qi);
  win_T *const save_curwin = curwin;

  FOR_ALL_TAB_WINDOWS(tp, win) {
    if (is_qf_win(win, qi)) {
      curwin = win;
      qf_set_title_var(qfl);
    }
  }
  curwin = save_curwin;
}

// Find the quickfix buffer.  If it exists, update the contents.
static void qf_update_buffer(qf_info_T *qi, qfline_T *old_last)
{
  // Check if a buffer for the quickfix list exists.  Update it.
  buf_T *buf = qf_find_buf(qi);
  if (buf == NULL) {
    return;
  }

  linenr_T old_line_count = buf->b_ml.ml_line_count;
  colnr_T old_endcol = ml_get_buf_len(buf, old_line_count);
  bcount_t old_bytecount = get_region_bytecount(buf, 1, old_line_count, 0, old_endcol);
  int qf_winid = 0;

  win_T *win;
  if (IS_LL_STACK(qi)) {
    if (curwin->w_llist == qi) {
      win = curwin;
    } else {
      // Find the file window (non-quickfix) with this location list
      win = qf_find_win_with_loclist(qi);
      if (win == NULL) {
        // File window is not found. Find the location list window.
        win = qf_find_win(qi);
      }
      if (win == NULL) {
        return;
      }
    }
    qf_winid = (int)win->handle;
  }

  // autocommands may cause trouble
  incr_quickfix_busy();

  aco_save_T aco;

  if (old_last == NULL) {
    // set curwin/curbuf to buf and save a few things
    aucmd_prepbuf(&aco, buf);
  }

  qf_update_win_titlevar(qi);

  rs_qf_fill_buffer(qf_get_curlist(qi), buf, old_last, qf_winid);

  linenr_T new_line_count = buf->b_ml.ml_line_count;
  colnr_T new_endcol = ml_get_buf_len(buf, new_line_count);
  bcount_t new_byte_count = 0;
  linenr_T delta = new_line_count - old_line_count;

  if (old_last == NULL) {
    new_byte_count = get_region_bytecount(buf, 1, new_line_count, 0, new_endcol);
    extmark_splice(buf, 0, 0, old_line_count - 1, 0, old_bytecount, new_line_count - 1, new_endcol,
                   new_byte_count, kExtmarkNoUndo);
    changed_lines(buf, 1, 0, old_line_count > 0 ? old_line_count + 1 : 1, delta, true);
  } else if (delta > 0) {
    linenr_T start_lnum = old_line_count + 1;
    new_byte_count = get_region_bytecount(buf, start_lnum, new_line_count, 0, new_endcol);
    extmark_splice(buf, old_line_count - 1, old_endcol, 0, 0, 0, delta, new_endcol, new_byte_count,
                   kExtmarkNoUndo);
    changed_lines(buf, start_lnum, 0, start_lnum, delta, true);
  }
  buf->b_changed = false;

  if (old_last == NULL) {
    qf_win_pos_update(qi, 0);

    // restore curwin/curbuf and a few other things
    aucmd_restbuf(&aco);
  }

  // Only redraw when added lines are visible.  This avoids flickering when
  // the added lines are not visible.
  if ((win = qf_find_win(qi)) != NULL && old_line_count < win->w_botline) {
    redraw_buf_later(buf, UPD_NOT_VALID);
  }

  // always called after incr_quickfix_busy()
  decr_quickfix_busy();
}

// qf_buf_add_line migrated to Rust (Phase 3) -- see rs_qf_buf_add_line in display.rs

// Call the 'quickfixtextfunc' function to get the list of lines to display in
// the quickfix window for the entries 'start_idx' to 'end_idx'.
static list_T *call_qftf_func(qf_list_T *qfl, int qf_winid, int start_idx, int end_idx)
{
  Callback *cb = &qftf_cb;
  list_T *qftf_list = NULL;
  static bool recursive = false;

  if (recursive) {
    return NULL;  // this doesn't work properly recursively
  }
  recursive = true;

  // If 'quickfixtextfunc' is set, then use the user-supplied function to get
  // the text to display. Use the local value of 'quickfixtextfunc' if it is
  // set.
  if (qfl->qf_qftf_cb.type != kCallbackNone) {
    cb = &qfl->qf_qftf_cb;
  }
  if (cb->type != kCallbackNone) {
    typval_T args[1];
    typval_T rettv;

    // create the dict argument
    dict_T *const dict = tv_dict_alloc_lock(VAR_FIXED);

    tv_dict_add_nr(dict, S_LEN("quickfix"), IS_QF_LIST(qfl));
    tv_dict_add_nr(dict, S_LEN("winid"), qf_winid);
    tv_dict_add_nr(dict, S_LEN("id"), qfl->qf_id);
    tv_dict_add_nr(dict, S_LEN("start_idx"), start_idx);
    tv_dict_add_nr(dict, S_LEN("end_idx"), end_idx);
    dict->dv_refcount++;
    args[0].v_type = VAR_DICT;
    args[0].vval.v_dict = dict;

    if (callback_call(cb, 1, args, &rettv)) {
      if (rettv.v_type == VAR_LIST) {
        qftf_list = rettv.vval.v_list;
        tv_list_ref(qftf_list);
      }
      tv_clear(&rettv);
    }
    tv_dict_unref(dict);
  }

  recursive = false;
  return qftf_list;
}

bool nvim_qf_buf_is_curbuf(const void *buf) { return (const buf_T *)buf == curbuf; }

/// Returns true on success.
bool nvim_qf_delete_all_lines(void)
{
  while ((curbuf->b_ml.ml_flags & ML_EMPTY) == 0) {
    if (ml_delete(1) == FAIL) {
      internal_error("rs_qf_fill_buffer()");
      return false;
    }
  }
  return true;
}

/// Zero skipcol for all windows showing curbuf
void nvim_qf_zero_skipcol_for_curbuf(void)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == curbuf) {
      wp->w_skipcol = 0;
    }
  }
}

void nvim_qf_u_clearallandblockfree(void) { u_clearallandblockfree(curbuf); }

void *nvim_call_qftf_func(void *qfl, int qf_winid, linenr_T start, int count) { return call_qftf_func((qf_list_T *)qfl, qf_winid, start, count); }

char *nvim_tv_list_item_string(const void *li) { return li == NULL ? NULL : (char *)tv_get_string_chk(TV_LIST_ITEM_TV((const listitem_T *)li)); }

// C accessor wrappers for rs_qf_buf_add_line (Phase 3)
// Note: nvim_buflist_findnr is in buffer.c (returns buf_T*)
// Note: nvim_buf_get_sfname is in buffer.c (takes buf_T*)
const char *nvim_qf_buf_get_fname(const void *buf) { return ((const buf_T *)buf)->b_fname; }
const char *nvim_path_tail_buf(const char *fname) { return path_tail((char *)fname); }
bool nvim_path_is_absolute(const char *fname) { return path_is_absolute(fname); }
void nvim_os_dirname(char *buf, int size) { os_dirname(buf, (size_t)size); }
void nvim_shorten_buf_fname(void *buf, const char *dirname, bool force)
{
  shorten_buf_fname((buf_T *)buf, (char *)dirname, force);
}
int nvim_ml_append_buf(void *buf, linenr_T lnum, char *line, int len, bool newfile)
{
  return ml_append_buf((buf_T *)buf, lnum, line, (colnr_T)len, newfile);
}
const char *nvim_skipwhite_const(const char *str) { return skipwhite(str); }

void nvim_ml_delete_one(linenr_T lnum) { ml_delete(lnum); }

void nvim_qfga_clear(void) { qfga_clear(); }

/// Set filetype, apply autocmds, and redraw for new qf buffer fill
void nvim_qf_set_filetype_and_autocmds(void)
{
  curbuf->b_ro_locked++;
  set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("qf"), OPT_LOCAL);
  curbuf->b_p_ma = false;

  curbuf->b_keep_filetype = true;
  apply_autocmds(EVENT_BUFREADPOST, "quickfix", NULL, false, curbuf);
  apply_autocmds(EVENT_BUFWINENTER, "quickfix", NULL, false, curbuf);
  curbuf->b_keep_filetype = false;
  curbuf->b_ro_locked--;

  redraw_curbuf_later(UPD_NOT_VALID);
}

bool nvim_qf_get_key_typed(void) { return KeyTyped; }

void nvim_qf_set_key_typed(bool val) { KeyTyped = val; }

void nvim_qf_fill_buffer_internal_error(void) { internal_error("rs_qf_fill_buffer()"); }

void *nvim_qf_get_start_nonnull(const void *qfl) { return qfl == NULL ? NULL : ((const qf_list_T *)qfl)->qf_start; }

static void qf_list_changed(qf_list_T *qfl) { rs_qf_incr_changedtick(qfl); }

// Jump to the first entry if there is one.
static void qf_jump_first(qf_info_T *qi, unsigned save_qfid, int forceit)
  FUNC_ATTR_NONNULL_ALL
{
  rs_qf_jump_first(qi, save_qfid, forceit);
}

// Return true when using ":vimgrep" for ":grep".
int grep_internal(cmdidx_T cmdidx)
{
  return rs_grep_internal(cmdidx);
}


// Form the complete command line to invoke 'make'/'grep'. Quote the command
// using 'shellquote' and append 'shellpipe'. Echo the fully formed command.
static char *make_get_fullcmd(const char *makecmd, const char *fname)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  size_t len = strlen(p_shq) * 2 + strlen(makecmd) + 1;
  if (*p_sp != NUL) {
    len += strlen(p_sp) + strlen(fname) + 3;
  }
  char *const cmd = xmalloc(len);
  snprintf(cmd, len, "%s%s%s", p_shq, makecmd, p_shq);

  // If 'shellpipe' empty: don't redirect to 'errorfile'.
  if (*p_sp != NUL) {
    append_redir(cmd, len, p_sp, fname);
  }

  // Display the fully formed command.  Output a newline if there's something
  // else than the :make command that was typed (in which case the cursor is
  // in column 0).
  if (msg_col == 0) {
    msg_didout = false;
  }
  msg_start();
  msg_puts(":!");
  msg_outtrans(cmd, 0, false);  // show what we are doing

  return cmd;
}

// Used for ":make", ":lmake", ":grep", ":lgrep", ":grepadd", and ":lgrepadd"
void ex_make(exarg_T *eap)
{
  char *enc = (*curbuf->b_p_menc != NUL) ? curbuf->b_p_menc : p_menc;

  // Redirect ":grep" to ":vimgrep" if 'grepprg' is "internal".
  if (grep_internal(eap->cmdidx)) {
    rs_ex_vimgrep(eap);
    return;
  }

  char *const au_name = (char *)rs_make_get_auname(eap->cmdidx);
  if (au_name != NULL && apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name,
                                        curbuf->b_fname, true, curbuf)) {
    if (aborting()) {
      return;
    }
  }

  win_T *wp = NULL;
  if (is_loclist_cmd(eap->cmdidx)) {
    wp = curwin;
  }

  autowrite_all();
  char *fname = get_mef_name();
  if (fname == NULL) {
    return;
  }
  os_remove(fname);  // in case it's not unique

  char *const cmd = make_get_fullcmd(eap->arg, fname);

  do_shell(cmd, 0);

  incr_quickfix_busy();

  char *errorformat = (eap->cmdidx != CMD_make && eap->cmdidx != CMD_lmake)
                      ? *curbuf->b_p_gefm != NUL ? curbuf->b_p_gefm : p_gefm
                      : p_efm;

  bool newlist = eap->cmdidx != CMD_grepadd && eap->cmdidx != CMD_lgrepadd;

  int res = qf_init(wp, fname, errorformat, newlist, qf_cmdtitle(*eap->cmdlinep), enc);

  qf_info_T *qi = ql_info;
  assert(qi != NULL);
  if (wp != NULL) {
    qi = GET_LOC_LIST(wp);
    if (qi == NULL) {
      goto cleanup;
    }
  }
  if (res >= 0) {
    qf_list_changed(qf_get_curlist(qi));
  }
  // Remember the current quickfix list identifier, so that we can
  // check for autocommands changing the current quickfix list.
  unsigned save_qfid = qf_get_curlist(qi)->qf_id;
  if (au_name != NULL) {
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true,
                   curbuf);
  }
  if (res > 0 && !eap->forceit && rs_qflist_valid((void *)wp, save_qfid)) {
    // display the first error
    qf_jump_first(qi, save_qfid, false);
  }

cleanup:
  decr_quickfix_busy();
  os_remove(fname);
  xfree(fname);
  xfree(cmd);
}

// Return the name for the errorfile, in allocated memory.
// Find a new unique name when 'makeef' contains "##".
// Returns NULL for error.
static char *get_mef_name(void)
{
  char *name;
  static int start = -1;
  static int off = 0;

  if (*p_mef == NUL) {
    name = vim_tempname();
    if (name == NULL) {
      emsg(_(e_notmp));
    }
    return name;
  }

  char *p;

  for (p = p_mef; *p; p++) {
    if (p[0] == '#' && p[1] == '#') {
      break;
    }
  }

  if (*p == NUL) {
    return xstrdup(p_mef);
  }

  // Keep trying until the name doesn't exist yet.
  while (true) {
    if (start == -1) {
      start = (int)os_get_pid();
    } else {
      off += 19;
    }
    name = xmalloc(strlen(p_mef) + 30);
    STRCPY(name, p_mef);
    snprintf(name + (p - p_mef), strlen(name), "%d%d", start, off);
    strcat(name, p + 2);
    // Don't accept a symbolic link, it's a security risk.
    FileInfo file_info;
    bool file_or_link_found = os_fileinfo_link(name, &file_info);
    if (!file_or_link_found) {
      break;
    }
    xfree(name);
  }
  return name;
}

/// Returns the number of entries in the current quickfix/location list.
size_t qf_get_size(exarg_T *eap)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_qf_get_size_eap(eap);
}

/// Returns the number of valid entries in the current quickfix/location list.
size_t qf_get_valid_size(exarg_T *eap)
{
  return rs_qf_get_valid_size_eap(eap);
}

/// Returns 0 if there is an error.
size_t qf_get_cur_idx(exarg_T *eap)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_qf_get_cur_idx_eap(eap);
}

/// Returns 1 if there are no valid entries.
int qf_get_cur_valid_idx(exarg_T *eap)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_qf_get_cur_valid_idx_eap(eap);
}

/// For :cfdo and :lfdo, returns the 'n'th valid file entry.
void *nvim_qf_cmd_get_stack(void *eap_void, bool print_emsg) { return qf_cmd_get_stack((exarg_T *)eap_void, print_emsg); }

void nvim_qf_msg(void *qi_void, int which, const char *lead) { rs_qf_msg(qi_void, which, lead); }

void nvim_emsg_loclist(void) { emsg(_(e_loclist)); }

void nvim_emsg_no_errors(void) { emsg(_(e_no_errors)); }

void nvim_emsg_at_bottom(void) { emsg(_("E380: At bottom of quickfix stack")); }

void nvim_emsg_at_top(void) { emsg(_("E381: At top of quickfix stack")); }

void nvim_msg_no_entries(void) { msg(_("No entries"), 0); }

void nvim_emsg_invrange(void) { emsg(_(e_invrange)); }

void nvim_qf_trunc_and_msg(const char *buf_in) { char buf[IOSIZE]; xstrlcpy(buf, buf_in, IOSIZE); trunc_string(buf, buf, Columns - 1, IOSIZE); msg(buf, 0); }

bool nvim_qf_curwin_is_ll(void) { return IS_LL_WINDOW(curwin); }

bool nvim_qf_is_ll_window(const void *wp_void) { return wp_void != NULL && IS_LL_WINDOW((const win_T *)wp_void); }

void *nvim_qf_curwin_get_loclist(void) { return GET_LOC_LIST(curwin); }

linenr_T nvim_qf_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }

void nvim_do_cmdline_cmd(const char *cmd) { do_cmdline_cmd(cmd); }

bool nvim_qf_curbuf_has_flag(int flag) { return (curbuf->b_has_qf_entry & flag) != 0; }

int nvim_qf_curbuf_fnum(void) { return curbuf->b_fnum; }

bool nvim_grep_uses_internal(void) { return strcmp("internal", *curbuf->b_p_gp == NUL ? p_gp : curbuf->b_p_gp) == 0; }

/// Returns pointer to static storage; only valid until next call.
const void *nvim_qf_curwin_pos_adj(void)
{
  static pos_T pos;
  pos = curwin->w_cursor;
  pos.col++;
  return &pos;
}

void *nvim_qf_get_curlist_mut(void *qi_void) { return (void *)&((qf_info_T *)qi_void)->qf_lists[((qf_info_T *)qi_void)->qf_curlist]; }

// Phase 3: qf_list_entry accessors
// nvim_message_filtered already exists in ex_cmds_shim.c (returns int)

/// Format quickfix entry prefix into IObuff and output the range+type+pattern+body
/// fields via message API using garray for intermediate buffers.
/// Takes Rust-computed text for range, type, pattern, and body.
void nvim_qf_list_entry_output(const char *prefix, bool cursel, int qfFile_hl_id_in,
                                int qfSep_hl_id_in, int qfLine_hl_id_in,
                                int lnum_nonzero, const char *range_text, size_t range_len,
                                const char *type_text,
                                int has_pattern, const char *pattern_text, size_t pattern_len,
                                const char *body_text, size_t body_len)
{
  msg_putchar('\n');
  msg_outtrans(prefix, cursel ? HLF_QFL : qfFile_hl_id_in, false);

  if (lnum_nonzero) {
    msg_puts_hl(":", qfSep_hl_id_in, false);
  }

  garray_T *gap = qfga_get();
  if (lnum_nonzero && range_text != NULL) {
    ga_concat_len(gap, range_text, range_len);
  }
  ga_concat(gap, type_text);
  ga_append(gap, NUL);
  msg_puts_hl(gap->ga_data, qfLine_hl_id_in, false);
  msg_puts_hl(":", qfSep_hl_id_in, false);

  if (has_pattern && pattern_text != NULL) {
    gap = qfga_get();
    ga_concat_len(gap, pattern_text, pattern_len);
    ga_append(gap, NUL);
    msg_puts(gap->ga_data);
    msg_puts_hl(":", qfSep_hl_id_in, false);
  }

  msg_puts(" ");

  gap = qfga_get();
  if (body_text != NULL) {
    ga_concat_len(gap, body_text, body_len);
  }
  ga_append(gap, NUL);
  msg_prt_line(gap->ga_data, false);
}

/// Format quickfix entry prefix: "%2d <name>" or "%2d" into buf.
void nvim_qf_format_prefix(char *buf, size_t bufsz, int idx, const char *name)
{
  if (name != NULL) {
    vim_snprintf(buf, bufsz, "%2d %s", idx, name);
  } else {
    snprintf(buf, bufsz, "%2d", idx);
  }
}

// Phase 4: qf_list (:clist/:llist) accessors
// nvim_eap_get_arg already exists in ex_docmd.c
// nvim_semsg_trailing_arg already exists in eval_shim.c
// nvim_eap_get_forceit already exists in indent_ffi.c (returns bool)
bool nvim_get_list_range(char **arg, int *idx1, int *idx2) { return get_list_range(arg, idx1, idx2); }
void nvim_shorten_fnames_qf(void) { shorten_fnames(false); }
int nvim_syn_name2id_qf(const char *name) { return syn_name2id(name); }
int nvim_hlf_d(void) { return HLF_D; }
int nvim_hlf_n(void) { return HLF_N; }
bool nvim_got_int_qf(void) { return got_int; }
void nvim_os_breakcheck_qf(void) { os_breakcheck(); }

// ":cfile"/":cgetfile"/":caddfile" commands.
// ":lfile"/":lgetfile"/":laddfile" commands.
void ex_cfile(exarg_T *eap)
{
  win_T *wp = NULL;
  qf_info_T *qi = ql_info;
  assert(qi != NULL);

  char *au_name = (char *)rs_cfile_get_auname(eap->cmdidx);
  if (au_name != NULL
      && apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name, NULL, false, curbuf)) {
    if (aborting()) {
      return;
    }
  }
  if (*eap->arg != NUL) {
    set_option_direct(kOptErrorfile, CSTR_AS_OPTVAL(eap->arg), 0, 0);
  }

  char *enc = (*curbuf->b_p_menc != NUL) ? curbuf->b_p_menc : p_menc;

  if (is_loclist_cmd(eap->cmdidx)) {
    wp = curwin;
  }

  incr_quickfix_busy();

  // This function is used by the :cfile, :cgetfile and :caddfile
  // commands.
  // :cfile always creates a new quickfix list and may jump to the
  // first error.
  // :cgetfile creates a new quickfix list but doesn't jump to the
  // first error.
  // :caddfile adds to an existing quickfix list. If there is no
  // quickfix list then a new list is created.
  int res = qf_init(wp, p_ef, p_efm, (eap->cmdidx != CMD_caddfile
                                      && eap->cmdidx != CMD_laddfile),
                    qf_cmdtitle(*eap->cmdlinep), enc);
  if (wp != NULL) {
    qi = GET_LOC_LIST(wp);
    if (qi == NULL) {
      decr_quickfix_busy();
      return;
    }
  }
  if (res >= 0) {
    qf_list_changed(qf_get_curlist(qi));
  }
  unsigned save_qfid = qf_get_curlist(qi)->qf_id;
  if (au_name != NULL) {
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, NULL, false, curbuf);
  }
  // Jump to the first error for a new list and if autocmds didn't free the
  // list.
  if (res > 0 && (eap->cmdidx == CMD_cfile || eap->cmdidx == CMD_lfile)
      && rs_qflist_valid((void *)wp, save_qfid)) {
    // display the first error
    qf_jump_first(qi, save_qfid, eap->forceit);
  }

  decr_quickfix_busy();
}


/// Initialize the regmatch used by vimgrep for pattern "s".
static void vgr_init_regmatch(regmmatch_T *regmatch, char *s)
{
  // Get the search pattern: either white-separated or enclosed in //.
  regmatch->regprog = NULL;

  if (s == NULL || *s == NUL) {
    // Pattern is empty, use last search pattern.
    if (last_search_pat() == NULL) {
      emsg(_(e_noprevre));
      return;
    }
    regmatch->regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
  } else {
    regmatch->regprog = vim_regcomp(s, RE_MAGIC);
  }

  regmatch->rmm_ic = p_ic;
  regmatch->rmm_maxcol = 0;
}

/// Display a file name when vimgrep is running.
static void vgr_display_fname(char *fname)
{
  msg_start();
  char *p = msg_strtrunc(fname, true);
  if (p == NULL) {
    msg_outtrans(fname, 0, false);
  } else {
    msg_outtrans(p, 0, false);
    xfree(p);
  }
  msg_clr_eos();
  msg_didout = false;  // overwrite this message
  msg_nowait = true;   // don't wait for this message
  msg_col = 0;
  ui_flush();
}

/// Load a dummy buffer to search for a pattern using vimgrep.
static buf_T *vgr_load_dummy_buf(char *fname, char *dirname_start, char *dirname_now)
{
  // Don't do Filetype autocommands to avoid loading syntax and
  // indent scripts, a great speed improvement.
  char *save_ei = au_event_disable(",Filetype");

  OptInt save_mls = p_mls;
  p_mls = 0;

  // Load file into a buffer, so that 'fileencoding' is detected,
  // autocommands applied, etc.
  buf_T *buf = load_dummy_buffer(fname, dirname_start, dirname_now);

  p_mls = save_mls;
  au_event_restore(save_ei);

  return buf;
}

/// create a new list.
static bool vgr_qflist_valid(win_T *wp, qf_info_T *qi, unsigned qfid, char *title)
{
  // Verify that the quickfix/location list was not freed by an autocmd
  if (!rs_qflist_valid((void *)wp, qfid)) {
    if (wp != NULL) {
      // An autocmd has freed the location list
      emsg(_(e_current_location_list_was_changed));
      return false;
    }
    // Quickfix list is not found, create a new one.
    rs_qf_new_list(qi, title);
    return true;
  }
  if (rs_qf_restore_list(qi, qfid) == FAIL) {
    return false;
  }

  return true;
}

int nvim_vim_regexec_multi(regmmatch_T *rm, void *win, void *buf, linenr_T lnum, colnr_T col) { return vim_regexec_multi(rm, (win_T *)win, (buf_T *)buf, lnum, col, NULL, NULL); }

linenr_T nvim_regmatch_startpos_lnum(const regmmatch_T *rm, int idx) { return rm->startpos[idx].lnum; }

colnr_T nvim_regmatch_startpos_col(const regmmatch_T *rm, int idx) { return rm->startpos[idx].col; }

linenr_T nvim_regmatch_endpos_lnum(const regmmatch_T *rm, int idx) { return rm->endpos[idx].lnum; }

colnr_T nvim_regmatch_endpos_col(const regmmatch_T *rm, int idx) { return rm->endpos[idx].col; }

colnr_T nvim_ml_get_buf_len(void *buf, linenr_T lnum) { return ml_get_buf_len((buf_T *)buf, lnum); }

/// Wrapper for fuzzy_match
int nvim_fuzzy_match(const char *str, const char *pat, bool matchseq,
                     int *score, uint32_t *matches, int max_matches)
{
  return fuzzy_match((char *)str, pat, matchseq, score, matches, max_matches);
}

// _Static_assert for constants used by rs_vgr_match_buflines
_Static_assert(VGR_GLOBAL == 1, "VGR_GLOBAL mismatch");
_Static_assert(VGR_NOJUMP == 2, "VGR_NOJUMP mismatch");
_Static_assert(VGR_FUZZY == 4, "VGR_FUZZY mismatch");
_Static_assert(FUZZY_MATCH_MAX_LEN == 1024, "FUZZY_MATCH_MAX_LEN mismatch");

// _Static_assert for Phase 1 CMD_* constants used in Rust auname lookups
_Static_assert(CMD_make == 273, "CMD_make mismatch");
_Static_assert(CMD_lmake == 248, "CMD_lmake mismatch");
_Static_assert(CMD_grep == 172, "CMD_grep mismatch");
_Static_assert(CMD_lgrep == 239, "CMD_lgrep mismatch");
_Static_assert(CMD_grepadd == 173, "CMD_grepadd mismatch");
_Static_assert(CMD_lgrepadd == 240, "CMD_lgrepadd mismatch");
_Static_assert(CMD_cfile == 65, "CMD_cfile mismatch");
_Static_assert(CMD_cgetfile == 68, "CMD_cgetfile mismatch");
_Static_assert(CMD_caddfile == 51, "CMD_caddfile mismatch");
_Static_assert(CMD_lfile == 233, "CMD_lfile mismatch");
_Static_assert(CMD_lgetfile == 236, "CMD_lgetfile mismatch");
_Static_assert(CMD_laddfile == 218, "CMD_laddfile mismatch");
_Static_assert(CMD_cbuffer == 55, "CMD_cbuffer mismatch");
_Static_assert(CMD_cgetbuffer == 69, "CMD_cgetbuffer mismatch");
_Static_assert(CMD_caddbuffer == 49, "CMD_caddbuffer mismatch");
_Static_assert(CMD_lbuffer == 221, "CMD_lbuffer mismatch");
_Static_assert(CMD_lgetbuffer == 237, "CMD_lgetbuffer mismatch");
_Static_assert(CMD_laddbuffer == 217, "CMD_laddbuffer mismatch");
_Static_assert(CMD_cexpr == 64, "CMD_cexpr mismatch");
_Static_assert(CMD_cgetexpr == 70, "CMD_cgetexpr mismatch");
_Static_assert(CMD_caddexpr == 50, "CMD_caddexpr mismatch");
_Static_assert(CMD_lexpr == 232, "CMD_lexpr mismatch");
_Static_assert(CMD_lgetexpr == 238, "CMD_lgetexpr mismatch");
_Static_assert(CMD_laddexpr == 216, "CMD_laddexpr mismatch");
_Static_assert(CMD_vimgrep == 509, "CMD_vimgrep mismatch");
_Static_assert(CMD_lvimgrep == 267, "CMD_lvimgrep mismatch");
_Static_assert(CMD_vimgrepadd == 510, "CMD_vimgrepadd mismatch");
_Static_assert(CMD_lvimgrepadd == 268, "CMD_lvimgrepadd mismatch");

// _Static_assert for Phase 4 CMD_* constants used in Rust valid counting functions
_Static_assert(CMD_cdo == 62, "CMD_cdo mismatch");
_Static_assert(CMD_ldo == 228, "CMD_ldo mismatch");
_Static_assert(CMD_cfdo == 66, "CMD_cfdo mismatch");
_Static_assert(CMD_lfdo == 234, "CMD_lfdo mismatch");

/// Jump to the first match and update the directory.
static void vgr_jump_to_match(qf_info_T *qi, int forceit, bool *redraw_for_dummy,
                              buf_T *first_match_buf, char *target_dir)  // NOLINT(readability-non-const-parameter)
{
  buf_T *buf = curbuf;
  rs_qf_jump_newwin(qi, 0, 0, forceit, false);
  if (buf != curbuf) {
    // If we jumped to another buffer redrawing will already be
    // taken care of.
    *redraw_for_dummy = false;
  }

  // Jump to the directory used after loading the buffer.
  if (curbuf == first_match_buf && target_dir != NULL) {
    exarg_T ea = {
      .arg = target_dir,
      .cmdidx = CMD_lcd,
    };
    ex_cd(&ea);
  }
}

// Return true if "buf" had an existing swap file, the current swap file does
// not end in ".swp".
static bool existing_swapfile(const buf_T *buf)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (buf->b_ml.ml_mfp != NULL && buf->b_ml.ml_mfp->mf_fname != NULL) {
    const char *const fname = buf->b_ml.ml_mfp->mf_fname;
    const size_t len = strlen(fname);

    return fname[len - 1] != 'p' || fname[len - 2] != 'w';
  }
  return false;
}

/// :{count}vimgrep /{pattern}/[g][j] {file} ...
static int vgr_process_args(exarg_T *eap, vgr_args_T *args)
{
  CLEAR_POINTER(args);

  args->regmatch.regprog = NULL;
  args->qf_title = xstrdup(qf_cmdtitle(*eap->cmdlinep));
  args->tomatch = eap->addr_count > 0 ? eap->line2 : MAXLNUM;

  // Get the search pattern: either white-separated or enclosed in //
  char *p = rs_skip_vimgrep_pat(eap->arg, &args->spat, &args->flags);
  if (p == NULL) {
    emsg(_(e_invalpat));
    return FAIL;
  }

  vgr_init_regmatch(&args->regmatch, args->spat);
  if (args->regmatch.regprog == NULL) {
    return FAIL;
  }

  p = skipwhite(p);
  if (*p == NUL) {
    emsg(_("E683: File name missing or invalid pattern"));
    return FAIL;
  }

  // Parse the list of arguments, wildcards have already been expanded.
  if (get_arglist_exp(p, &args->fcount, &args->fnames, true) == FAIL
      || args->fcount == 0) {
    emsg(_(e_nomatch));
    return FAIL;
  }

  return OK;
}

/// Returns dirname_start via *start_out, dirname_now via *now_out.
void nvim_vgr_alloc_dirnames(char **start_out, char **now_out)
{
  *start_out = xmalloc(MAXPATHL);
  *now_out = xmalloc(MAXPATHL);
  os_dirname(*start_out, MAXPATHL);
}

void nvim_vgr_free_dirnames(char *start, char *now) { xfree(now); xfree(start); }

char *nvim_vgr_shorten_fname(const char *full_fname) { return path_try_shorten_fname((char *)full_fname); }

void nvim_vgr_display_fname_wrapper(const char *fname) { vgr_display_fname((char *)fname); }

/// Returns: the buffer handle (or NULL), and sets *has_mfp if buffer has ml_mfp.
void *nvim_vgr_find_buf(const char *fname, bool *has_mfp)
{
  buf_T *buf = buflist_findname_exp((char *)fname);
  if (buf != NULL) {
    *has_mfp = (buf->b_ml.ml_mfp != NULL);
  } else {
    *has_mfp = false;
  }
  return buf;
}

void *nvim_vgr_load_dummy_buf_wrapper(const char *fname, char *dirname_start, char *dirname_now) { return vgr_load_dummy_buf((char *)fname, dirname_start, dirname_now); }

bool nvim_vgr_qflist_valid_wrapper(void *wp, void *qi, unsigned qfid, const char *title) { return vgr_qflist_valid((win_T *)wp, (qf_info_T *)qi, qfid, title); }

void nvim_vgr_smsg_cannot_open(const char *fname) { smsg(0, _("Cannot open file \"%s\""), fname); }

long nvim_vgr_time_now(void) { return (long)time(NULL); }

/// first_match_buf and target_dir may be updated.
void nvim_vgr_handle_dummy_buf(void *buf_void, bool found_match, bool duplicate_name,
                                int flags, char *dirname_start, char *dirname_now,
                                void **first_match_buf, char **target_dir)
{
  buf_T *buf = (buf_T *)buf_void;
  buf_T **fmb = (buf_T **)first_match_buf;

  if (found_match && *fmb == NULL) {
    *fmb = buf;
  }

  if (duplicate_name) {
    wipe_dummy_buffer(buf, dirname_start);
    return;
  }

  if ((cmdmod.cmod_flags & CMOD_HIDE) == 0
      || buf->b_p_bh[0] == 'u'
      || buf->b_p_bh[0] == 'w'
      || buf->b_p_bh[0] == 'd') {
    if (!found_match) {
      wipe_dummy_buffer(buf, dirname_start);
      return;
    }
    if (buf != *fmb
        || (flags & VGR_NOJUMP)
        || existing_swapfile(buf)) {
      unload_dummy_buffer(buf, dirname_start);
      buf->b_flags &= ~BF_DUMMY;
      return;
    }
  }

  // Buffer is kept loaded
  buf->b_flags &= ~BF_DUMMY;

  if (buf == *fmb
      && *target_dir == NULL
      && strcmp(dirname_start, dirname_now) != 0) {
    *target_dir = xstrdup(dirname_now);
  }

  // Apply Filetype autocommands and modelines
  aco_save_T aco;
  aucmd_prepbuf(&aco, buf);
  apply_autocmds(EVENT_FILETYPE, buf->b_p_ft, buf->b_fname, true, buf);
  do_modelines(OPT_NOWIN);
  aucmd_restbuf(&aco);
}

/// Returns false if we should abort.
bool nvim_vgr_pre_check(void *eap_void)
{
  exarg_T *eap = (exarg_T *)eap_void;
  if (!check_can_set_curbuf_forceit(eap->forceit)) {
    return false;
  }
  char *au_name = (char *)rs_vgr_get_auname(eap->cmdidx);
  if (au_name != NULL && apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name,
                                        curbuf->b_fname, true, curbuf)) {
    if (aborting()) {
      return false;
    }
  }
  return true;
}

/// Returns false if vgr_process_args failed.
bool nvim_vgr_setup(void *eap_void, void **qi_out, void **wp_out, void **args_out)
{
  exarg_T *eap = (exarg_T *)eap_void;
  win_T *wp = NULL;
  qf_info_T *qi = qf_cmd_get_or_alloc_stack(eap, &wp);
  *qi_out = qi;
  *wp_out = wp;

  vgr_args_T *args = xcalloc(1, sizeof(vgr_args_T));
  if (vgr_process_args(eap, args) == FAIL) {
    xfree(args);
    *args_out = NULL;
    return false;
  }
  *args_out = args;

  if ((eap->cmdidx != CMD_grepadd && eap->cmdidx != CMD_lgrepadd
       && eap->cmdidx != CMD_vimgrepadd && eap->cmdidx != CMD_lvimgrepadd)
      || rs_qf_stack_empty(qi)) {
    rs_qf_new_list(qi, args->qf_title);
  }

  return true;
}

/// Get vgr_args fields for Rust.
void nvim_vgr_args_get_fields(const void *args_void,
                               int *fcount, const char *const **fnames,
                               const char **spat, void **regmatch,
                               int **tomatch, int *flags,
                               const char **qf_title)
{
  const vgr_args_T *args = (const vgr_args_T *)args_void;
  *fcount = args->fcount;
  *fnames = (const char *const *)args->fnames;
  *spat = args->spat;
  *regmatch = (void *)&((vgr_args_T *)args)->regmatch;
  *tomatch = &((vgr_args_T *)args)->tomatch;
  *flags = args->flags;
  *qf_title = args->qf_title;
}

/// Free vgr_args wild files.
void nvim_vgr_free_wild(void *args_void)
{
  vgr_args_T *args = (vgr_args_T *)args_void;
  FreeWild(args->fcount, args->fnames);
  args->fnames = NULL;
  args->fcount = 0;
}

/// Finalize the vimgrep list: set nonevalid, ptr, index, list_changed.
void nvim_vgr_finalize_list(void *qi_void)
{
  qf_info_T *qi = (qf_info_T *)qi_void;
  qf_list_T *qfl = qf_get_curlist(qi);
  qfl->qf_nonevalid = false;
  qfl->qf_ptr = qfl->qf_start;
  qfl->qf_index = 1;
  qf_list_changed(qfl);
  qf_update_buffer(qi, NULL);
}

/// Apply QuickFixCmdPost autocmd for vimgrep.
void nvim_vgr_post_autocmd(void *eap_void)
{
  exarg_T *eap = (exarg_T *)eap_void;
  char *au_name = (char *)rs_vgr_get_auname(eap->cmdidx);
  if (au_name != NULL) {
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
  }
}

bool nvim_vgr_list_still_valid(void *wp_void, void *qi_void, unsigned save_qfid) { return rs_qflist_valid((void *)wp_void, save_qfid) && rs_qf_restore_list((qf_info_T *)qi_void, save_qfid) != FAIL; }

/// Jump to first match or emit nomatch error.
void nvim_vgr_jump_or_nomatch(void *qi_void, void *eap_void, bool *redraw_for_dummy,
                               void *first_match_buf, char *target_dir,
                               int flags, const char *spat)
{
  qf_info_T *qi = (qf_info_T *)qi_void;
  exarg_T *eap = (exarg_T *)eap_void;
  if (!rs_qf_list_empty(qf_get_curlist(qi))) {
    if ((flags & VGR_NOJUMP) == 0) {
      vgr_jump_to_match(qi, eap->forceit, redraw_for_dummy,
                        (buf_T *)first_match_buf, target_dir);
    }
  } else {
    semsg(_(e_nomatch2), spat);
  }
}

void nvim_incr_quickfix_busy(void) { incr_quickfix_busy(); }

void nvim_decr_quickfix_busy(void) { decr_quickfix_busy(); }

void nvim_vgr_foldUpdateAll_curwin(void) { rs_foldUpdateAll(curwin); }

/// Cleanup vgr_args: free title and regprog.
void nvim_vgr_cleanup_args(void *args_void)
{
  if (args_void == NULL) {
    return;
  }
  vgr_args_T *args = (vgr_args_T *)args_void;
  xfree(args->qf_title);
  vim_regfree(args->regmatch.regprog);
  xfree(args);
}

// Restore current working directory to "dirname_start" if they differ, taking
// into account whether it is set locally or globally.
static void restore_start_dir(char *dirname_start)
  FUNC_ATTR_NONNULL_ALL
{
  char *dirname_now = xmalloc(MAXPATHL);

  os_dirname(dirname_now, MAXPATHL);
  if (strcmp(dirname_start, dirname_now) != 0) {
    // If the directory has changed, change it back by building up an
    // appropriate ex command and executing it.
    exarg_T ea = {
      .arg = dirname_start,
      .cmdidx = (curwin->w_localdir == NULL) ? CMD_cd : CMD_lcd,
    };
    ex_cd(&ea);
  }
  xfree(dirname_now);
}

/// @return  NULL if it fails.
static buf_T *load_dummy_buffer(char *fname, char *dirname_start, char *resulting_dir)
{
  // Allocate a buffer without putting it in the buffer list.
  buf_T *newbuf = buflist_new(NULL, NULL, 1, BLN_DUMMY);
  if (newbuf == NULL) {
    return NULL;
  }

  bool failed = true;
  bufref_T newbufref;
  set_bufref(&newbufref, newbuf);

  // Init the options.
  buf_copy_options(newbuf, BCO_ENTER | BCO_NOHELP);

  // need to open the memfile before putting the buffer in a window
  if (ml_open(newbuf) == OK) {
    // Make sure this buffer isn't wiped out by autocommands.
    newbuf->b_locked++;
    // set curwin/curbuf to buf and save a few things
    aco_save_T aco;
    aucmd_prepbuf(&aco, newbuf);

    // Need to set the filename for autocommands.
    setfname(curbuf, fname, NULL, false);

    // Create swap file now to avoid the ATTENTION message.
    check_need_swap(true);

    // Remove the "dummy" flag, otherwise autocommands may not
    // work.
    curbuf->b_flags &= ~BF_DUMMY;

    bufref_T newbuf_to_wipe;
    newbuf_to_wipe.br_buf = NULL;
    int readfile_result = readfile(fname, NULL, 0, 0,
                                   (linenr_T)MAXLNUM, NULL,
                                   READ_NEW | READ_DUMMY, false);
    newbuf->b_locked--;
    if (readfile_result == OK
        && !got_int
        && !(curbuf->b_flags & BF_NEW)) {
      failed = false;
      if (curbuf != newbuf) {
        // Bloody autocommands changed the buffer!  Can happen when
        // using netrw and editing a remote file.  Use the current
        // buffer instead, delete the dummy one after restoring the
        // window stuff.
        set_bufref(&newbuf_to_wipe, newbuf);
        newbuf = curbuf;
      }
    }

    // Restore curwin/curbuf and a few other things.
    aucmd_restbuf(&aco);

    if (newbuf_to_wipe.br_buf != NULL && bufref_valid(&newbuf_to_wipe)) {
      block_autocmds();
      wipe_dummy_buffer(newbuf_to_wipe.br_buf, NULL);
      unblock_autocmds();
    }

    // Add back the "dummy" flag, otherwise buflist_findname_file_id()
    // won't skip it.
    newbuf->b_flags |= BF_DUMMY;
  }

  // When autocommands/'autochdir' option changed directory: go back.
  // Let the caller know what the resulting dir was first, in case it is
  // important.
  os_dirname(resulting_dir, MAXPATHL);
  restore_start_dir(dirname_start);

  if (!bufref_valid(&newbufref)) {
    return NULL;
  }
  if (failed) {
    wipe_dummy_buffer(newbuf, dirname_start);
    return NULL;
  }
  return newbuf;
}

/// the 'autochdir' option have changed it.
static void wipe_dummy_buffer(buf_T *buf, char *dirname_start)
  FUNC_ATTR_NONNULL_ARG(1)
{
  // If any autocommand opened a window on the dummy buffer, close that
  // window.  If we can't close them all then give up.
  while (buf->b_nwindows > 0) {
    bool did_one = false;

    if (firstwin->w_next != NULL) {
      for (win_T *wp = firstwin; wp != NULL; wp = wp->w_next) {
        if (wp->w_buffer == buf) {
          if (win_close(wp, false, false) == OK) {
            did_one = true;
          }
          break;
        }
      }
    }
    if (!did_one) {
      goto fail;
    }
  }

  if (curbuf != buf && buf->b_nwindows == 0) {  // safety check
    cleanup_T cs;

    // Reset the error/interrupt/exception state here so that aborting()
    // returns false when wiping out the buffer.  Otherwise it doesn't
    // work when got_int is set.
    enter_cleanup(&cs);

    wipe_buffer(buf, true);

    // Restore the error/interrupt/exception state if not discarded by a
    // new aborting error, interrupt, or uncaught exception.
    leave_cleanup(&cs);

    if (dirname_start != NULL) {
      // When autocommands/'autochdir' option changed directory: go back.
      restore_start_dir(dirname_start);
    }

    return;
  }

fail:
  // Keeping the buffer, remove the dummy flag.
  buf->b_flags &= ~BF_DUMMY;
}

/// 'autochdir' option have changed it.
static void unload_dummy_buffer(buf_T *buf, char *dirname_start)
{
  if (curbuf == buf) {          // safety check
    return;
  }

  close_buffer(NULL, buf, DOBUF_UNLOAD, false, true);

  // When autocommands/'autochdir' option changed directory: go back.
  restore_start_dir(dirname_start);
}

/// to 'list'.  Returns OK on success.
static int get_qfline_items(qfline_T *qfp, list_T *list)
{
  // Handle entries with a non-existing buffer number.
  int bufnum = qfp->qf_fnum;
  if (bufnum != 0 && (buflist_findnr(bufnum) == NULL)) {
    bufnum = 0;
  }

  dict_T *const dict = tv_dict_alloc();
  tv_list_append_dict(list, dict);

  char buf[2];
  buf[0] = qfp->qf_type;
  buf[1] = NUL;
  if (tv_dict_add_nr(dict, S_LEN("bufnr"), (varnumber_T)bufnum) == FAIL
      || (tv_dict_add_nr(dict, S_LEN("lnum"), (varnumber_T)qfp->qf_lnum) == FAIL)
      || (tv_dict_add_nr(dict, S_LEN("end_lnum"), (varnumber_T)qfp->qf_end_lnum) == FAIL)
      || (tv_dict_add_nr(dict, S_LEN("col"), (varnumber_T)qfp->qf_col) == FAIL)
      || (tv_dict_add_nr(dict, S_LEN("end_col"), (varnumber_T)qfp->qf_end_col) == FAIL)
      || (tv_dict_add_nr(dict, S_LEN("vcol"), (varnumber_T)qfp->qf_viscol) == FAIL)
      || (tv_dict_add_nr(dict, S_LEN("nr"), (varnumber_T)qfp->qf_nr) == FAIL)
      || (tv_dict_add_str(dict, S_LEN("module"), (qfp->qf_module == NULL ? "" : qfp->qf_module))
          == FAIL)
      || (tv_dict_add_str(dict, S_LEN("pattern"), (qfp->qf_pattern == NULL ? "" : qfp->qf_pattern))
          == FAIL)
      || (tv_dict_add_str(dict, S_LEN("text"), (qfp->qf_text == NULL ? "" : qfp->qf_text)) == FAIL)
      || (tv_dict_add_str(dict, S_LEN("type"), buf) == FAIL)
      || (qfp->qf_user_data.v_type != VAR_UNKNOWN
          && tv_dict_add_tv(dict, S_LEN("user_data"), &qfp->qf_user_data) == FAIL)
      || (tv_dict_add_nr(dict, S_LEN("valid"), (varnumber_T)qfp->qf_valid) == FAIL)) {
    // tv_dict_add* fail only if key already exist, but this is a newly
    // allocated dictionary which is thus guaranteed to have no existing keys.
    abort();
  }

  return OK;
}

/// all the entries.
static int get_errorlist(qf_info_T *qi_arg, win_T *wp, int qf_idx, int eidx, list_T *list)
{
  qf_info_T *qi = qi_arg;

  if (qi == NULL) {
    qi = ql_info;
    if (wp != NULL) {
      qi = GET_LOC_LIST(wp);
    }
    if (qi == NULL) {
      return FAIL;
    }
  }

  if (eidx < 0) {
    return OK;
  }

  if (qf_idx == INVALID_QFIDX) {
    qf_idx = qi->qf_curlist;
  }

  if (qf_idx >= qi->qf_listcount) {
    return FAIL;
  }

  qf_list_T *qfl = qf_get_list(qi, qf_idx);
  if (rs_qf_list_empty(qfl)) {
    return FAIL;
  }

  qfline_T *qfp;
  int i;
  FOR_ALL_QFL_ITEMS(qfl, qfp, i) {
    if (eidx > 0) {
      if (eidx == i) {
        return get_qfline_items(qfp, list);
      }
    } else if (get_qfline_items(qfp, list) == FAIL) {
      return FAIL;
    }
  }

  return OK;
}

/// Flags used by getqflist()/getloclist() to determine which fields to return.
enum {
  QF_GETLIST_NONE = 0x0,
  QF_GETLIST_TITLE = 0x1,
  QF_GETLIST_ITEMS = 0x2,
  QF_GETLIST_NR = 0x4,
  QF_GETLIST_WINID = 0x8,
  QF_GETLIST_CONTEXT = 0x10,
  QF_GETLIST_ID = 0x20,
  QF_GETLIST_IDX = 0x40,
  QF_GETLIST_SIZE = 0x80,
  QF_GETLIST_TICK = 0x100,
  QF_GETLIST_FILEWINID = 0x200,
  QF_GETLIST_QFBUFNR = 0x400,
  QF_GETLIST_QFTF = 0x800,
  QF_GETLIST_ALL = 0xFFF,
};

/// Existing quickfix lists are not modified.
static int qf_get_list_from_lines(dict_T *what, dictitem_T *di, dict_T *retdict)
{
  int status = FAIL;

  // Only a List value is supported
  if (di->di_tv.v_type != VAR_LIST || di->di_tv.vval.v_list == NULL) {
    return FAIL;
  }

  char *errorformat = p_efm;
  dictitem_T *efm_di;
  // If errorformat is supplied then use it, otherwise use the 'efm'
  // option setting
  if ((efm_di = tv_dict_find(what, S_LEN("efm"))) != NULL) {
    if (efm_di->di_tv.v_type != VAR_STRING
        || efm_di->di_tv.vval.v_string == NULL) {
      return FAIL;
    }
    errorformat = efm_di->di_tv.vval.v_string;
  }

  list_T *l = tv_list_alloc(kListLenMayKnow);
  qf_info_T *const qi = qf_alloc_stack(QFLT_INTERNAL, 1);

  if (rs_qf_init_ext(qi, 0, NULL, NULL, &di->di_tv, errorformat,
                  true, 0, 0, NULL, NULL) > 0) {
    get_errorlist(qi, NULL, 0, 0, l);
    rs_qf_free_list(&qi->qf_lists[0]);
  }

  qf_free_lists(qi);

  tv_dict_add_list(retdict, S_LEN("items"), l);
  status = OK;

  return status;
}

/// Return the quickfix/location list window identifier in the current tabpage.
static int qf_winid(qf_info_T *qi)
{
  // The quickfix window can be opened even if the quickfix list is not set
  // using ":copen". This is not true for location lists.
  if (qi == NULL) {
    return 0;
  }
  win_T *win = qf_find_win(qi);
  if (win != NULL) {
    return win->handle;
  }
  return 0;
}

/// Accessor for Rust: get winid for a quickfix info (0 if not found).
int nvim_qf_winid(const void *qi_void) { return qf_winid((qf_info_T *)(uintptr_t)qi_void); }

/// wiped out, then returns 0.
static int qf_getprop_qfbufnr(const qf_info_T *qi, dict_T *retdict)
  FUNC_ATTR_NONNULL_ARG(2)
{
  return tv_dict_add_nr(retdict, S_LEN("qfbufnr"), rs_qf_getprop_qfbufnr(qi));
}

// _Static_assert for Phase 3 QF_GETLIST_* constants used in Rust property functions
_Static_assert(QF_GETLIST_NONE == 0x0, "QF_GETLIST_NONE mismatch");
_Static_assert(QF_GETLIST_TITLE == 0x1, "QF_GETLIST_TITLE mismatch");
_Static_assert(QF_GETLIST_ITEMS == 0x2, "QF_GETLIST_ITEMS mismatch");
_Static_assert(QF_GETLIST_NR == 0x4, "QF_GETLIST_NR mismatch");
_Static_assert(QF_GETLIST_WINID == 0x8, "QF_GETLIST_WINID mismatch");
_Static_assert(QF_GETLIST_CONTEXT == 0x10, "QF_GETLIST_CONTEXT mismatch");
_Static_assert(QF_GETLIST_ID == 0x20, "QF_GETLIST_ID mismatch");
_Static_assert(QF_GETLIST_IDX == 0x40, "QF_GETLIST_IDX mismatch");
_Static_assert(QF_GETLIST_SIZE == 0x80, "QF_GETLIST_SIZE mismatch");
_Static_assert(QF_GETLIST_TICK == 0x100, "QF_GETLIST_TICK mismatch");
_Static_assert(QF_GETLIST_FILEWINID == 0x200, "QF_GETLIST_FILEWINID mismatch");
_Static_assert(QF_GETLIST_QFBUFNR == 0x400, "QF_GETLIST_QFBUFNR mismatch");
_Static_assert(QF_GETLIST_QFTF == 0x800, "QF_GETLIST_QFTF mismatch");
_Static_assert(QF_GETLIST_ALL == 0xFFF, "QF_GETLIST_ALL mismatch");

static int qf_getprop_title(qf_list_T *qfl, dict_T *retdict) { return tv_dict_add_str(retdict, S_LEN("title"), qfl->qf_title); }

// Returns the identifier of the window used to display files from a location
// list.  If there is no associated window, then returns 0. Useful only when
// called from a location list window.
static int qf_getprop_filewinid(const win_T *wp, const qf_info_T *qi, dict_T *retdict)
  FUNC_ATTR_NONNULL_ARG(3)
{
  return tv_dict_add_nr(retdict, S_LEN("filewinid"), rs_qf_getprop_filewinid(wp, qi));
}

/// If eidx is not 0, then return the item at the specified index.
static int qf_getprop_items(qf_info_T *qi, int qf_idx, int eidx, dict_T *retdict)
{
  list_T *l = tv_list_alloc(kListLenMayKnow);
  get_errorlist(qi, NULL, qf_idx, eidx, l);
  tv_dict_add_list(retdict, S_LEN("items"), l);

  return OK;
}

/// Return the quickfix list context (if any) as 'context' in retdict.
static int qf_getprop_ctx(qf_list_T *qfl, dict_T *retdict)
{
  int status;

  if (qfl->qf_ctx != NULL) {
    dictitem_T *di = tv_dict_item_alloc_len(S_LEN("context"));
    tv_copy(qfl->qf_ctx, &di->di_tv);
    status = tv_dict_add(retdict, di);
    if (status == FAIL) {
      tv_dict_item_free(di);
    }
  } else {
    status = tv_dict_add_str(retdict, S_LEN("context"), "");
  }

  return status;
}

/// If a specific entry index (eidx) is supplied, then use that.
static int qf_getprop_idx(qf_list_T *qfl, int eidx, dict_T *retdict)
{
  if (eidx == 0) {
    eidx = qfl->qf_index;
    if (rs_qf_list_empty(qfl)) {
      // For empty lists, current index is set to 0
      eidx = 0;
    }
  }
  return tv_dict_add_nr(retdict, S_LEN("idx"), eidx);
}

/// @return OK or FAIL
static int qf_getprop_qftf(qf_list_T *qfl, dict_T *retdict)
  FUNC_ATTR_NONNULL_ALL
{
  int status;

  if (qfl->qf_qftf_cb.type != kCallbackNone) {
    typval_T tv;

    callback_put(&qfl->qf_qftf_cb, &tv);
    status = tv_dict_add_tv(retdict, S_LEN("quickfixtextfunc"), &tv);
    tv_clear(&tv);
  } else {
    status = tv_dict_add_str(retdict, S_LEN("quickfixtextfunc"), "");
  }

  return status;
}

/// then current list is used. Otherwise the specified list is used.
static int qf_get_properties(win_T *wp, dict_T *what, dict_T *retdict)
{
  qf_info_T *qi = ql_info;
  assert(qi != NULL);
  dictitem_T *di = NULL;
  int status = OK;
  int qf_idx = INVALID_QFIDX;

  if ((di = tv_dict_find(what, S_LEN("lines"))) != NULL) {
    return qf_get_list_from_lines(what, di, retdict);
  }

  if (wp != NULL) {
    qi = GET_LOC_LIST(wp);
  }

  const int flags = rs_qf_getprop_keys2flags(what, wp != NULL);

  if (!rs_qf_stack_empty(qi)) {
    qf_idx = rs_qf_getprop_qfidx(qi, what);
  }

  // List is not present or is empty
  if (rs_qf_stack_empty(qi) || qf_idx == INVALID_QFIDX) {
    return rs_qf_getprop_defaults(qi, flags, wp != NULL, retdict);
  }

  qf_list_T *qfl = qf_get_list(qi, qf_idx);
  int eidx = 0;

  // If an entry index is specified, use that
  if ((di = tv_dict_find(what, S_LEN("idx"))) != NULL) {
    if (di->di_tv.v_type != VAR_NUMBER) {
      return FAIL;
    }
    eidx = (int)di->di_tv.vval.v_number;
  }

  if (flags & QF_GETLIST_TITLE) {
    status = qf_getprop_title(qfl, retdict);
  }
  if ((status == OK) && (flags & QF_GETLIST_NR)) {
    status = tv_dict_add_nr(retdict, S_LEN("nr"), qf_idx + 1);
  }
  if ((status == OK) && (flags & QF_GETLIST_WINID)) {
    status = tv_dict_add_nr(retdict, S_LEN("winid"), qf_winid(qi));
  }
  if ((status == OK) && (flags & QF_GETLIST_ITEMS)) {
    status = qf_getprop_items(qi, qf_idx, eidx, retdict);
  }
  if ((status == OK) && (flags & QF_GETLIST_CONTEXT)) {
    status = qf_getprop_ctx(qfl, retdict);
  }
  if ((status == OK) && (flags & QF_GETLIST_ID)) {
    status = tv_dict_add_nr(retdict, S_LEN("id"), qfl->qf_id);
  }
  if ((status == OK) && (flags & QF_GETLIST_IDX)) {
    status = qf_getprop_idx(qfl, eidx, retdict);
  }
  if ((status == OK) && (flags & QF_GETLIST_SIZE)) {
    status = tv_dict_add_nr(retdict, S_LEN("size"),
                            qfl->qf_count);
  }
  if ((status == OK) && (flags & QF_GETLIST_TICK)) {
    status = tv_dict_add_nr(retdict, S_LEN("changedtick"),
                            qfl->qf_changedtick);
  }
  if ((status == OK) && (wp != NULL) && (flags & QF_GETLIST_FILEWINID)) {
    status = qf_getprop_filewinid(wp, qi, retdict);
  }
  if ((status == OK) && (flags & QF_GETLIST_QFBUFNR)) {
    status = qf_getprop_qfbufnr(qi, retdict);
  }
  if ((status == OK) && (flags & QF_GETLIST_QFTF)) {
    status = qf_getprop_qftf(qfl, retdict);
  }

  return status;
}

/// @return OK
static int qf_setprop_qftf(qf_list_T *qfl, dictitem_T *di)
  FUNC_ATTR_NONNULL_ALL
{
  Callback cb;

  callback_free(&qfl->qf_qftf_cb);
  if (rs_callback_from_typval(&cb, &di->di_tv)) {
    qfl->qf_qftf_cb = cb;
  }
  return OK;
}

/// to true.
static int qf_add_entry_from_dict(qf_list_T *qfl, dict_T *d, bool first_entry, bool *valid_entry)
  FUNC_ATTR_NONNULL_ALL
{
  static bool did_bufnr_emsg;

  if (first_entry) {
    did_bufnr_emsg = false;
  }

  char *const filename = tv_dict_get_string(d, "filename", true);
  char *const module = tv_dict_get_string(d, "module", true);
  int bufnum = (int)tv_dict_get_number(d, "bufnr");
  const linenr_T lnum = (linenr_T)tv_dict_get_number(d, "lnum");
  const linenr_T end_lnum = (linenr_T)tv_dict_get_number(d, "end_lnum");
  const int col = (int)tv_dict_get_number(d, "col");
  const int end_col = (int)tv_dict_get_number(d, "end_col");
  const char vcol = (char)tv_dict_get_number(d, "vcol");
  const int nr = (int)tv_dict_get_number(d, "nr");
  const char *const type = tv_dict_get_string(d, "type", false);
  char *const pattern = tv_dict_get_string(d, "pattern", true);
  char *text = tv_dict_get_string(d, "text", true);
  if (text == NULL) {
    text = xcalloc(1, 1);
  }
  typval_T user_data = { .v_type = VAR_UNKNOWN };
  tv_dict_get_tv(d, "user_data", &user_data);

  bool valid = true;
  if ((filename == NULL && bufnum == 0)
      || (lnum == 0 && pattern == NULL)) {
    valid = false;
  }

  // Mark entries with non-existing buffer number as not valid. Give the
  // error message only once.
  if (bufnum != 0 && (buflist_findnr(bufnum) == NULL)) {
    if (!did_bufnr_emsg) {
      did_bufnr_emsg = true;
      semsg(_("E92: Buffer %" PRId64 " not found"), (int64_t)bufnum);
    }
    valid = false;
    bufnum = 0;
  }

  // If the 'valid' field is present it overrules the detected value.
  if (tv_dict_find(d, "valid", -1) != NULL) {
    valid = tv_dict_get_number(d, "valid");
  }

  const int status = rs_qf_add_entry(qfl,
                                  NULL,      // dir
                                  filename,
                                  module,
                                  bufnum,
                                  text,
                                  lnum,
                                  end_lnum,
                                  col,
                                  end_col,
                                  vcol,      // vis_col
                                  pattern,   // search pattern
                                  nr,
                                  type == NULL ? NUL : *type,
                                  &user_data,
                                  valid);

  xfree(filename);
  xfree(module);
  xfree(pattern);
  xfree(text);
  tv_clear(&user_data);

  if (valid) {
    *valid_entry = true;
  }

  return status;
}

/// Get the first item in a VimL list
void *nvim_tv_list_first(const void *list)
{
  if (list == NULL) {
    return NULL;
  }
  return tv_list_first((const list_T *)list);
}

/// Get the next item in a VimL list
void *nvim_tv_list_item_next(const void *list, const void *li)
{
  if (list == NULL || li == NULL) {
    return NULL;
  }
  return TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li);
}

/// Get dict from a list item, or NULL if not a dict type
void *nvim_tv_list_item_dict(const void *li)
{
  if (li == NULL) {
    return NULL;
  }
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  if (tv->v_type != VAR_DICT) {
    return NULL;
  }
  return tv->vval.v_dict;
}

int nvim_qf_add_entry_from_dict(void *qfl, void *d, bool first_entry, bool *valid_entry) { return qf_add_entry_from_dict((qf_list_T *)qfl, (dict_T *)d, first_entry, valid_entry); }

/// Get the current entry's fnum, lnum, col as a group
void nvim_qf_get_ptr_position(const void *qfl_void, int *fnum, int *lnum, int *col)
{
  const qf_list_T *qfl = (const qf_list_T *)qfl_void;
  if (qfl == NULL || qfl->qf_ptr == NULL) {
    *fnum = 0;
    *lnum = 0;
    *col = 0;
    return;
  }
  *fnum = qfl->qf_ptr->qf_fnum;
  *lnum = qfl->qf_ptr->qf_lnum;
  *col = qfl->qf_ptr->qf_col;
}

/// Check if a list item is the first item in the list
bool nvim_tv_list_item_is_first(const void *list, const void *li)
{
  if (list == NULL || li == NULL) {
    return false;
  }
  return li == tv_list_first((const list_T *)list);
}

// Set the quickfix list title.
static int qf_setprop_title(qf_info_T *qi, int qf_idx, const dict_T *what, const dictitem_T *di)
  FUNC_ATTR_NONNULL_ALL
{
  qf_list_T *qfl = qf_get_list(qi, qf_idx);
  if (di->di_tv.v_type != VAR_STRING) {
    return FAIL;
  }

  xfree(qfl->qf_title);
  qfl->qf_title = tv_dict_get_string(what, "title", true);
  if (qf_idx == qi->qf_curlist) {
    qf_update_win_titlevar(qi);
  }

  return OK;
}

// Set quickfix list items/entries.
static int qf_setprop_items(qf_info_T *qi, int qf_idx, dictitem_T *di, int action)
  FUNC_ATTR_NONNULL_ALL
{
  if (di->di_tv.v_type != VAR_LIST) {
    return FAIL;
  }

  char *title_save = xstrdup(qi->qf_lists[qf_idx].qf_title);
  const int retval = rs_qf_add_entries(qi, qf_idx, di->di_tv.vval.v_list, title_save,
                                    action == ' ' ? 'a' : action);
  xfree(title_save);

  return retval;
}

// Set quickfix list items/entries from a list of lines.
static int qf_setprop_items_from_lines(qf_info_T *qi, int qf_idx, const dict_T *what,
                                       dictitem_T *di, int action)
  FUNC_ATTR_NONNULL_ALL
{
  char *errorformat = p_efm;
  dictitem_T *efm_di;
  int retval = FAIL;

  // Use the user supplied errorformat settings (if present)
  if ((efm_di = tv_dict_find(what, S_LEN("efm"))) != NULL) {
    if (efm_di->di_tv.v_type != VAR_STRING
        || efm_di->di_tv.vval.v_string == NULL) {
      return FAIL;
    }
    errorformat = efm_di->di_tv.vval.v_string;
  }

  // Only a List value is supported
  if (di->di_tv.v_type != VAR_LIST || di->di_tv.vval.v_list == NULL) {
    return FAIL;
  }

  if (action == 'r' || action == 'u') {
    rs_qf_free_items(&qi->qf_lists[qf_idx]);
  }
  if (rs_qf_init_ext(qi, qf_idx, NULL, NULL, &di->di_tv, errorformat,
                  false, 0, 0, NULL, NULL) >= 0) {
    retval = OK;
  }

  return retval;
}

// Set quickfix list context.
static int qf_setprop_context(qf_list_T *qfl, dictitem_T *di)
  FUNC_ATTR_NONNULL_ALL
{
  tv_free(qfl->qf_ctx);
  typval_T *ctx = xcalloc(1, sizeof(typval_T));
  tv_copy(&di->di_tv, ctx);
  qfl->qf_ctx = ctx;

  return OK;
}

// Set the current index in the specified quickfix list
static int qf_setprop_curidx(qf_info_T *qi, qf_list_T *qfl, const dictitem_T *di)
  FUNC_ATTR_NONNULL_ALL
{
  int newidx;

  // If the specified index is '$', then use the last entry
  if (di->di_tv.v_type == VAR_STRING
      && di->di_tv.vval.v_string != NULL
      && strcmp(di->di_tv.vval.v_string, "$") == 0) {
    newidx = qfl->qf_count;
  } else {
    // Otherwise use the specified index
    bool denote = false;
    newidx = (int)tv_get_number_chk(&di->di_tv, &denote);
    if (denote) {
      return FAIL;
    }
  }

  if (newidx < 1) {  // sanity check
    return FAIL;
  }
  newidx = MIN(newidx, qfl->qf_count);
  const int old_qfidx = qfl->qf_index;
  qfline_T *const qf_ptr = rs_qf_get_nth_entry(qfl, newidx, &newidx);
  if (qf_ptr == NULL) {
    return FAIL;
  }
  qfl->qf_ptr = qf_ptr;
  qfl->qf_index = newidx;

  // If the current list is modified and it is displayed in the quickfix
  // window, then Update it.
  if (qi->qf_lists[qi->qf_curlist].qf_id == qfl->qf_id) {
    qf_win_pos_update(qi, old_qfidx);
  }
  return OK;
}

/// Used by the setqflist() and setloclist() Vim script functions.
static int qf_set_properties(qf_info_T *qi, const dict_T *what, int action, char *title)
  FUNC_ATTR_NONNULL_ALL
{
  bool newlist = action == ' ' || rs_qf_stack_empty(qi);
  int qf_idx = rs_qf_setprop_get_qfidx(qi, what, action, &newlist);
  if (qf_idx == INVALID_QFIDX) {  // List not found
    return FAIL;
  }

  if (newlist) {
    qi->qf_curlist = qf_idx;
    rs_qf_new_list(qi, title);
    qf_idx = qi->qf_curlist;
  }

  qf_list_T *qfl = qf_get_list(qi, qf_idx);
  dictitem_T *di;
  int retval = FAIL;
  if ((di = tv_dict_find(what, S_LEN("title"))) != NULL) {
    retval = qf_setprop_title(qi, qf_idx, what, di);
  }
  if ((di = tv_dict_find(what, S_LEN("items"))) != NULL) {
    retval = qf_setprop_items(qi, qf_idx, di, action);
  }
  if ((di = tv_dict_find(what, S_LEN("lines"))) != NULL) {
    retval = qf_setprop_items_from_lines(qi, qf_idx, what, di, action);
  }
  if ((di = tv_dict_find(what, S_LEN("context"))) != NULL) {
    retval = qf_setprop_context(qfl, di);
  }
  if ((di = tv_dict_find(what, S_LEN("idx"))) != NULL) {
    retval = qf_setprop_curidx(qi, qfl, di);
  }
  if ((di = tv_dict_find(what, S_LEN("quickfixtextfunc"))) != NULL) {
    retval = qf_setprop_qftf(qfl, di);
  }

  if (newlist || retval == OK) {
    qf_list_changed(qfl);
  }
  if (newlist) {
    qf_update_buffer(qi, NULL);
  }

  return retval;
}

// Free the entire quickfix/location list stack.
// If the quickfix/location list window is open, then clear it.
static void qf_free_stack(win_T *wp, qf_info_T *qi)
{
  win_T *qfwin = qf_find_win(qi);

  if (qfwin != NULL) {
    // If the quickfix/location list window is open, then clear it
    if (qi->qf_curlist < qi->qf_listcount) {
      rs_qf_free_list(qf_get_curlist(qi));
    }
    qf_update_buffer(qi, NULL);
  }

  if (wp != NULL && IS_LL_WINDOW(wp)) {
    // If in the location list window, then use the non-location list
    // window with this location list (if present)
    win_T *const llwin = qf_find_win_with_loclist(qi);
    if (llwin != NULL) {
      wp = llwin;
    }
  }

  qf_free_all(wp);
  if (wp == NULL) {
    // quickfix list
    qi->qf_curlist = 0;
    qi->qf_listcount = 0;
  } else if (qfwin != NULL) {
    // If the location list window is open, then create a new empty location
    // list
    qf_info_T *new_ll = qf_alloc_stack(QFLT_LOCATION, (int)wp->w_p_lhi);
    new_ll->qf_bufnr = qfwin->w_buffer->b_fnum;

    // first free the list reference in the location list window
    ll_free_all(&qfwin->w_llist_ref);

    qfwin->w_llist_ref = new_ll;
    if (wp != qfwin) {
      win_set_loclist(wp, new_ll);
    }
  }
}

/// When "what" is not NULL then only set some properties.
int set_errorlist(win_T *wp, list_T *list, int action, char *title, dict_T *what)
{
  qf_info_T *qi;
  if (wp != NULL) {
    qi = ll_get_or_alloc_list(wp);
  } else {
    qi = ql_info;
  }
  assert(qi != NULL);

  if (action == 'f') {
    // Free the entire quickfix or location list stack
    qf_free_stack(wp, qi);
    return OK;
  }

  // A dict argument cannot be specified with a non-empty list argument
  if (list != NULL && tv_list_len(list) != 0 && what != NULL) {
    semsg(_(e_invarg2), _("cannot have both a list and a \"what\" argument"));
    return FAIL;
  }

  incr_quickfix_busy();

  int retval = OK;
  if (what != NULL) {
    retval = qf_set_properties(qi, what, action, title);
  } else {
    retval = rs_qf_add_entries(qi, qi->qf_curlist, list, title, action);
    if (retval == OK) {
      qf_list_changed(qf_get_curlist(qi));
    }
  }

  decr_quickfix_busy();

  return retval;
}

static bool mark_quickfix_user_data(qf_info_T *qi, int copyID)
{
  bool abort = false;
  for (int i = 0; i < qi->qf_maxcount && !abort; i++) {
    qf_list_T *qfl = &qi->qf_lists[i];
    if (!qfl->qf_has_user_data) {
      continue;
    }
    qfline_T *qfp;
    int j;
    FOR_ALL_QFL_ITEMS(qfl, qfp, j) {
      typval_T *user_data = &qfp->qf_user_data;
      if (user_data != NULL && user_data->v_type != VAR_NUMBER
          && user_data->v_type != VAR_STRING && user_data->v_type != VAR_FLOAT) {
        abort = abort || rs_set_ref_in_item(user_data, copyID, NULL, NULL);
      }
    }
  }
  return abort;
}

/// in a quickfix stack.
static bool mark_quickfix_ctx(qf_info_T *qi, int copyID)
{
  bool abort = false;

  for (int i = 0; i < qi->qf_maxcount && !abort; i++) {
    typval_T *ctx = qi->qf_lists[i].qf_ctx;
    if (ctx != NULL && ctx->v_type != VAR_NUMBER
        && ctx->v_type != VAR_STRING && ctx->v_type != VAR_FLOAT) {
      abort = rs_set_ref_in_item(ctx, copyID, NULL, NULL);
    }

    Callback *cb = &qi->qf_lists[i].qf_qftf_cb;
    abort = abort || rs_set_ref_in_callback(cb, copyID, NULL, NULL);
  }

  return abort;
}

/// "in use". So that garbage collection doesn't free the context.
bool set_ref_in_quickfix(int copyID)
{
  assert(ql_info != NULL);
  if (mark_quickfix_ctx(ql_info, copyID)
      || mark_quickfix_user_data(ql_info, copyID)
      || rs_set_ref_in_callback(&qftf_cb, copyID, NULL, NULL)) {
    return true;
  }

  FOR_ALL_TAB_WINDOWS(tp, win) {
    if (win->w_llist != NULL) {
      if (mark_quickfix_ctx(win->w_llist, copyID)
          || mark_quickfix_user_data(win->w_llist, copyID)) {
        return true;
      }
    }

    if (IS_LL_WINDOW(win) && (win->w_llist_ref->qf_refcount == 1)) {
      // In a location list window and none of the other windows is
      // referring to this location list. Mark the location list
      // context as still in use.
      if (mark_quickfix_ctx(win->w_llist_ref, copyID)
          || mark_quickfix_user_data(win->w_llist_ref, copyID)) {
        return true;
      }
    }
  }

  return false;
}


/// :cgetbuffer, :lbuffer, :laddbuffer, :lgetbuffer Ex commands.
static int cbuffer_process_args(exarg_T *eap, buf_T **bufp, linenr_T *line1, linenr_T *line2)
{
  buf_T *buf = NULL;

  if (*eap->arg == NUL) {
    buf = curbuf;
  } else if (*skipwhite(skipdigits(eap->arg)) == NUL) {
    buf = buflist_findnr(atoi(eap->arg));
  }

  if (buf == NULL) {
    emsg(_(e_invarg));
    return FAIL;
  }

  if (buf->b_ml.ml_mfp == NULL) {
    emsg(_(e_buffer_is_not_loaded));
    return FAIL;
  }

  if (eap->addr_count == 0) {
    eap->line1 = 1;
    eap->line2 = buf->b_ml.ml_line_count;
  }

  if (eap->line1 < 1 || eap->line1 > buf->b_ml.ml_line_count
      || eap->line2 < 1 || eap->line2 > buf->b_ml.ml_line_count) {
    emsg(_(e_invrange));
    return FAIL;
  }

  *line1 = eap->line1;
  *line2 = eap->line2;
  *bufp = buf;

  return OK;
}

// ":[range]cbuffer [bufnr]" command.
// ":[range]caddbuffer [bufnr]" command.
// ":[range]cgetbuffer [bufnr]" command.
// ":[range]lbuffer [bufnr]" command.
// ":[range]laddbuffer [bufnr]" command.
// ":[range]lgetbuffer [bufnr]" command.
void ex_cbuffer(exarg_T *eap)
{
  char *au_name = (char *)rs_cbuffer_get_auname(eap->cmdidx);
  if (au_name != NULL && apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name,
                                        curbuf->b_fname, true, curbuf)) {
    if (aborting()) {
      return;
    }
  }

  // Must come after autocommands.
  win_T *wp = NULL;
  qf_info_T *qi = qf_cmd_get_or_alloc_stack(eap, &wp);

  buf_T *buf = NULL;
  linenr_T line1;
  linenr_T line2;
  if (cbuffer_process_args(eap, &buf, &line1, &line2) == FAIL) {
    return;
  }

  char *qf_title = qf_cmdtitle(*eap->cmdlinep);

  if (buf->b_sfname) {
    vim_snprintf(IObuff, IOSIZE, "%s (%s)", qf_title, buf->b_sfname);
    qf_title = IObuff;
  }

  incr_quickfix_busy();

  int res = rs_qf_init_ext(qi, qi->qf_curlist, NULL, buf, NULL, p_efm,
                        (eap->cmdidx != CMD_caddbuffer
                         && eap->cmdidx != CMD_laddbuffer),
                        eap->line1, eap->line2, qf_title, NULL);
  if (rs_qf_stack_empty(qi)) {
    decr_quickfix_busy();
    return;
  }
  if (res >= 0) {
    qf_list_changed(qf_get_curlist(qi));
  }
  // Remember the current quickfix list identifier, so that we can
  // check for autocommands changing the current quickfix list.
  unsigned save_qfid = qf_get_curlist(qi)->qf_id;
  if (au_name != NULL) {
    const buf_T *const curbuf_old = curbuf;
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
    if (curbuf != curbuf_old) {
      // Autocommands changed buffer, don't jump now, "qi" may
      // be invalid.
      res = 0;
    }
  }
  // Jump to the first error for new list and if autocmds didn't
  // free the list.
  if (res > 0 && (eap->cmdidx == CMD_cbuffer || eap->cmdidx == CMD_lbuffer)
      && rs_qflist_valid((void *)wp, save_qfid)) {
    // display the first error
    qf_jump_first(qi, save_qfid, eap->forceit);
  }

  decr_quickfix_busy();
}


/// ":lexpr {expr}", ":lgetexpr {expr}", ":laddexpr {expr}" command.
void ex_cexpr(exarg_T *eap)
{
  char *au_name = (char *)rs_cexpr_get_auname(eap->cmdidx);
  if (au_name != NULL && apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name,
                                        curbuf->b_fname, true, curbuf)) {
    if (aborting()) {
      return;
    }
  }

  win_T *wp = NULL;
  qf_info_T *qi = qf_cmd_get_or_alloc_stack(eap, &wp);

  // Evaluate the expression.  When the result is a string or a list we can
  // use it to fill the errorlist.
  typval_T *tv = eval_expr(eap->arg, eap);
  if (tv == NULL) {
    return;
  }

  if ((tv->v_type == VAR_STRING && tv->vval.v_string != NULL)
      || tv->v_type == VAR_LIST) {
    incr_quickfix_busy();
    int res = rs_qf_init_ext(qi, qi->qf_curlist, NULL, NULL, tv, p_efm,
                          (eap->cmdidx != CMD_caddexpr
                           && eap->cmdidx != CMD_laddexpr),
                          0, 0,
                          qf_cmdtitle(*eap->cmdlinep), NULL);
    if (rs_qf_stack_empty(qi)) {
      decr_quickfix_busy();
      goto cleanup;
    }
    if (res >= 0) {
      qf_list_changed(qf_get_curlist(qi));
    }
    // Remember the current quickfix list identifier, so that we can
    // check for autocommands changing the current quickfix list.
    unsigned save_qfid = qf_get_curlist(qi)->qf_id;
    if (au_name != NULL) {
      apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
    }
    // Jump to the first error for a new list and if autocmds didn't
    // free the list.
    if (res > 0
        && (eap->cmdidx == CMD_cexpr || eap->cmdidx == CMD_lexpr)
        && rs_qflist_valid((void *)wp, save_qfid)) {
      // display the first error
      qf_jump_first(qi, save_qfid, eap->forceit);
    }
    decr_quickfix_busy();
  } else {
    emsg(_("E777: String or List expected"));
  }
cleanup:
  tv_free(tv);
}

// Get the location list for ":lhelpgrep"
static qf_info_T *hgr_get_ll(bool *new_ll)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  // If the current window is a help window, then use it, else find an existing help window
  win_T *wp = bt_help(curwin->w_buffer) ? curwin : qf_find_help_win();

  qf_info_T *qi = wp == NULL ? NULL : wp->w_llist;
  if (qi == NULL) {
    // Allocate a new location list for help text matches
    qi = qf_alloc_stack(QFLT_LOCATION, 1);
    *new_ll = true;
  }

  return qi;
}

// C accessor wrappers for rs_hgr_search_* functions (Phase 1)
FILE *nvim_hgr_os_fopen(const char *fname) { return os_fopen(fname, "r"); }
bool nvim_hgr_vim_fgets(char *buf, int size, FILE *fd) { return vim_fgets(buf, size, fd); }
bool nvim_hgr_vim_regexec(void *rmp, char *line) { return vim_regexec((regmatch_T *)rmp, line, 0); }
char *nvim_hgr_regmatch_startp(void *rmp) { return ((regmatch_T *)rmp)->startp[0]; }
char *nvim_hgr_regmatch_endp(void *rmp) { return ((regmatch_T *)rmp)->endp[0]; }
void nvim_hgr_fclose(FILE *fd) { fclose(fd); }
char *nvim_hgr_get_IObuff(void) { return IObuff; }
int nvim_hgr_get_IOSIZE(void) { return IOSIZE; }
int nvim_hgr_get_got_int(void) { return got_int; }
void nvim_hgr_set_got_int(int val) { got_int = val; }
void nvim_hgr_line_breakcheck(void) { line_breakcheck(); }
int nvim_hgr_gen_expand_wildcards(char *dirname, int *fcount_out, char ***fnames_out)
{
  return gen_expand_wildcards(1, &dirname, fcount_out, fnames_out, EW_FILE|EW_SILENT);
}
void nvim_hgr_free_wild(int fcount, char **fnames) { FreeWild(fcount, fnames); }
char *nvim_hgr_fname_at(char **fnames, int idx) { return fnames[idx]; }
void nvim_hgr_add_pathsep(char *dirname) { add_pathsep(dirname); }
void nvim_hgr_strcat_doc_glob(char *dirname) { strcat(dirname, "doc/*.\\(txt\\|??x\\)"); }  // NOLINT
int nvim_hgr_STRNICMP(const char *a, const char *b, int n) { return STRNICMP(a, b, (size_t)n); }
char *nvim_hgr_get_p_rtp(void) { return p_rtp; }
void nvim_hgr_copy_option_part(char **pp, char *buf, int maxlen)
{
  copy_option_part(pp, buf, (size_t)maxlen, ",");
}
int nvim_hgr_get_MAXPATHL(void) { return MAXPATHL; }
char *nvim_hgr_get_NameBuff(void) { return NameBuff; }

/// Returns true if we should proceed, false if aborting.
bool nvim_hgr_pre_check(void *eap_void)
{
  exarg_T *eap = (exarg_T *)eap_void;
  char *au_name = NULL;
  switch (eap->cmdidx) {
  case CMD_helpgrep:
    au_name = "helpgrep"; break;
  case CMD_lhelpgrep:
    au_name = "lhelpgrep"; break;
  default:
    break;
  }
  if (au_name != NULL && apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name,
                                        curbuf->b_fname, true, curbuf)) {
    if (aborting()) {
      return false;
    }
  }
  return true;
}

/// Returns the saved value as an opaque pointer.
void *nvim_hgr_save_cpo(void)
{
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;
  return save_cpo;
}

bool nvim_hgr_is_loclist_cmd(const void *eap_void) { return is_loclist_cmd(((const exarg_T *)eap_void)->cmdidx); }

void *nvim_hgr_get_ll(bool *new_qi_out) { return hgr_get_ll(new_qi_out); }

/// Returns true if the list was updated (regex compiled + search done).
bool nvim_hgr_compile_and_search(void *eap_void, void *qi_void)
{
  exarg_T *eap = (exarg_T *)eap_void;
  qf_info_T *qi = (qf_info_T *)qi_void;

  char *const lang = check_help_lang(eap->arg);
  regmatch_T regmatch = {
    .regprog = vim_regcomp(eap->arg, RE_MAGIC + RE_STRING),
    .rm_ic = false,
  };
  if (regmatch.regprog == NULL) {
    return false;
  }

  rs_qf_new_list(qi, qf_cmdtitle(*eap->cmdlinep));
  qf_list_T *const qfl = qf_get_curlist(qi);

  rs_hgr_search_in_rtp(qfl, &regmatch, lang);

  vim_regfree(regmatch.regprog);

  qfl->qf_nonevalid = false;
  qfl->qf_ptr = qfl->qf_start;
  qfl->qf_index = 1;
  qf_list_changed(qfl);
  return true;
}

/// changed p_cpo while we had it set to empty_string_option.
void nvim_hgr_restore_cpo(void *saved_cpo_void)
{
  char *save_cpo = (char *)saved_cpo_void;
  if (p_cpo == empty_string_option) {
    p_cpo = save_cpo;
  } else {
    // Darn, some plugin changed the value.  If it's still empty it was
    // changed and restored, need to restore in the complicated way.
    if (*p_cpo == NUL) {
      set_option_value_give_err(kOptCpoptions, CSTR_AS_OPTVAL(save_cpo), 0);
    }
    free_string_option(save_cpo);
  }
}

/// Returns true if we should continue (not early-return).
bool nvim_hgr_post_autocmd(void *eap_void, void *qi_void, bool new_qi)
{
  exarg_T *eap = (exarg_T *)eap_void;
  qf_info_T *qi = (qf_info_T *)qi_void;

  char *au_name = NULL;
  switch (eap->cmdidx) {
  case CMD_helpgrep:
    au_name = "helpgrep"; break;
  case CMD_lhelpgrep:
    au_name = "lhelpgrep"; break;
  default:
    break;
  }

  if (au_name != NULL) {
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
    if (!new_qi && IS_LL_STACK(qi) && qf_find_win_with_loclist(qi) == NULL) {
      return false;  // stack invalid, caller should early-return
    }
  }
  return true;
}

/// Jump to first match or show "no match" error.
void nvim_hgr_jump_or_nomatch(void *eap_void, void *qi_void)
{
  exarg_T *eap = (exarg_T *)eap_void;
  qf_info_T *qi = (qf_info_T *)qi_void;

  if (!rs_qf_list_empty(qf_get_curlist(qi))) {
    rs_qf_jump_newwin(qi, 0, 0, false, false);
  } else {
    semsg(_(e_nomatch2), eap->arg);
  }
}

bool nvim_hgr_is_lhelpgrep(const void *eap_void) { return ((const exarg_T *)eap_void)->cmdidx == CMD_lhelpgrep; }

/// Cleanup for :lhelpgrep — free location list if not needed.
void nvim_hgr_cleanup(void *qi_void, bool new_qi)
{
  qf_info_T *qi = (qf_info_T *)qi_void;

  // If the help window is not opened or if it already points to the
  // correct location list, then free the new location list.
  if (!bt_help(curwin->w_buffer) || curwin->w_llist == qi) {
    if (new_qi) {
      ll_free_all(&qi);
    }
  } else if (curwin->w_llist == NULL && new_qi) {
    // current window didn't have a location list associated with it
    // before. Associate the new location list now.
    curwin->w_llist = qi;
  }
}

#if defined(EXITFREE)
void free_quickfix(void)
{
  qf_free_all(NULL);
  // Free all location lists
  FOR_ALL_TAB_WINDOWS(tab, win) {
    qf_free_all(win);
  }

  ga_clear(&qfga);
}
#endif

static void get_qf_loc_list(bool is_qf, win_T *wp, typval_T *what_arg, typval_T *rettv)
{
  if (what_arg->v_type == VAR_UNKNOWN) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    if (is_qf || wp != NULL) {
      get_errorlist(NULL, wp, -1, 0, rettv->vval.v_list);
    }
  } else {
    tv_dict_alloc_ret(rettv);
    if (is_qf || wp != NULL) {
      if (what_arg->v_type == VAR_DICT) {
        dict_T *d = what_arg->vval.v_dict;

        if (d != NULL) {
          qf_get_properties(wp, d, rettv->vval.v_dict);
        }
      } else {
        emsg(_(e_dictreq));
      }
    }
  }
}

void f_getloclist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { win_T *wp = find_win_by_nr_or_id(&argvars[0]); get_qf_loc_list(false, wp, &argvars[1], rettv); }

void f_getqflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { get_qf_loc_list(true, NULL, &argvars[0], rettv); }

/// @param[out]  rettv  Return value: 0 in case of success, -1 otherwise.
static void set_qf_ll_list(win_T *wp, typval_T *args, typval_T *rettv)
  FUNC_ATTR_NONNULL_ARG(2, 3)
{
  static const char *e_invact = N_("E927: Invalid action: '%s'");
  const char *title = NULL;
  char action = ' ';
  static int recursive = 0;
  rettv->vval.v_number = -1;
  dict_T *what = NULL;

  typval_T *list_arg = &args[0];
  if (list_arg->v_type != VAR_LIST) {
    emsg(_(e_listreq));
    return;
  } else if (recursive != 0) {
    emsg(_(e_au_recursive));
    return;
  }

  typval_T *action_arg = &args[1];
  if (action_arg->v_type == VAR_UNKNOWN) {
    // Option argument was not given.
    goto skip_args;
  } else if (action_arg->v_type != VAR_STRING) {
    emsg(_(e_string_required));
    return;
  }
  const char *const act = tv_get_string_chk(action_arg);
  if ((*act == 'a' || *act == 'r' || *act == 'u' || *act == ' ' || *act == 'f')
      && act[1] == NUL) {
    action = *act;
  } else {
    semsg(_(e_invact), act);
    return;
  }

  typval_T *const what_arg = &args[2];
  if (what_arg->v_type == VAR_UNKNOWN) {
    // Option argument was not given.
    goto skip_args;
  } else if (what_arg->v_type == VAR_STRING) {
    title = tv_get_string_chk(what_arg);
    if (!title) {
      // Type error. Error already printed by tv_get_string_chk().
      return;
    }
  } else if (what_arg->v_type == VAR_DICT && what_arg->vval.v_dict != NULL) {
    what = what_arg->vval.v_dict;
  } else {
    emsg(_(e_dictreq));
    return;
  }

skip_args:
  if (!title) {
    title = (wp ? ":setloclist()" : ":setqflist()");
  }

  recursive++;
  list_T *const l = list_arg->vval.v_list;
  if (set_errorlist(wp, l, action, (char *)title, what) == OK) {
    rettv->vval.v_number = 0;
  }
  recursive--;
}

/// "setloclist()" function
void f_setloclist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = -1;

  win_T *win = find_win_by_nr_or_id(&argvars[0]);
  if (win != NULL) {
    set_qf_ll_list(win, &argvars[1], rettv);
  }
}

void f_setqflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { set_qf_ll_list(NULL, argvars, rettv); }
