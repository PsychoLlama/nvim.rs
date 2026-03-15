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

extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern bool rs_qf_list_empty(const void *qfl);
extern bool rs_qflist_valid(const void *wp, unsigned qf_id);
// rs_qf_msg deleted: Rust bypasses nvim_qf_msg via #[link_name]
extern int rs_copy_loclist(const void *from_qfl, void *to_qfl);

extern void rs_qf_free_list(void *qfl);

int nvim_qf_get_listcount(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_listcount; }

int nvim_qf_get_count(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_count; }

bool nvim_qf_get_nonevalid(const void *qfl_void) { return ((const qf_list_T *)qfl_void)->qf_nonevalid; }

linenr_T nvim_qfline_get_lnum(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_lnum; }

int nvim_qfline_get_col(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_col; }

void *nvim_qf_get_curlist(const void *qi_void) { return (void *)&((const qf_info_T *)qi_void)->qf_lists[((const qf_info_T *)qi_void)->qf_curlist]; }

void *nvim_qf_get_list_at(const void *qi_void, int idx) { return (void *)&((const qf_info_T *)qi_void)->qf_lists[idx]; }

int nvim_qf_get_curlist_idx(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_curlist; }

int nvim_qf_get_index(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_index; }

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

unsigned nvim_qf_get_id(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_id; }

int nvim_qf_get_changedtick(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_changedtick; }

const char *nvim_qf_get_title(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_title; }

int nvim_qf_get_maxcount(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_maxcount; }




// Phase 1: Auname lookups and qf_types (migrated to Rust)
extern const char *rs_qf_types(int c, int nr, char *buf, size_t bufsz);

// Phase 2: Format and title helpers (migrated to Rust)
extern size_t rs_qf_cmdtitle(const char *cmd, char *buf, size_t bufsz);

// Phase 3: Property flag operations and index resolution (migrated to Rust)

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

// Buffer size and part length helpers

// Prefix type helpers

// Phase 5: parse_match and parse_line (migrated to Rust)
extern int rs_qf_parse_match(const char *linebuf, size_t linelen, void *fmt_ptr, const void *rm,
                              void *fields, bool qf_multiline, bool qf_multiscan, char **tail);
extern int rs_qf_parse_line(void *qfl, char *linebuf, size_t linelen, void *fmt_first,
                             void *fields);

// Entry creation
extern int rs_qf_add_entry(void *qfl, char *dir, const char *fname, const char *module,
                           int bufnum, const char *mesg, linenr_T lnum, linenr_T end_lnum,
                           int col, int end_col, char vis_col, const char *pattern, int nr,
                           char type, const void *user_data, char valid);

// Directory stack operations (Phase 7)

// Vimgrep functions

// List management functions

// Display functions
extern void rs_qf_fill_buffer(void *qfl, void *buf, void *old_last, int qf_winid);

// Helpgrep functions (Phase 1)

// Init functions
extern int rs_qf_init_ext(void *qi, int qf_idx, const char *efile, void *buf,
                           void *tv, char *errorformat, bool newlist, linenr_T lnumfirst,
                           linenr_T lnumlast, const char *qf_title, char *enc);
extern int rs_qf_init(void *wp, const char *efile, char *errorformat, bool newlist,
                      const char *qf_title, char *enc);
extern void rs_ex_vimgrep(void *eap);

// Phase 11: window/title helpers and position update (migrated to Rust)
extern const void *rs_qf_find_win_for_stack(const void *qi);
extern bool rs_qf_win_pos_update(void *qi, int old_qf_index);
// rs_qf_set_title_var deleted: nvim_qf_set_title_var wrapper deleted (dead code)
// rs_qf_update_win_titlevar deleted: nvim_qf_update_win_titlevar wrapper deleted (dead code)

// Phase 11: stack resize and location list sync (migrated to Rust)
extern void rs_qf_resize_stack_base(void *qi, int n);
extern void rs_qf_sync_llw_to_win(void *llw);
extern void rs_qf_sync_win_to_llw(void *pwp);

// Phase 3: lifecycle functions (migrated to Rust)
extern void *rs_qf_alloc_stack(int qfltype, int n);
// rs_qf_cmd_get_stack deleted: Rust commands.rs uses #[link_name] directly.

// Pass 4: stack query entry points (Phase 1)
// rs_qf_get_size_eap, rs_qf_get_valid_size_eap, rs_qf_get_cur_idx_eap,
// rs_qf_get_cur_valid_idx_eap, rs_grep_internal removed: all now export under C names via #[export_name].
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

// Phase 14 Pass 4: thin replacement for nvim_qf_store_title
/// Free old title and store a duplicate of the new one (NULL clears it).
void nvim_qf_set_title_dup(void *qfl_void, const char *title)
{
  if (qfl_void == NULL) {
    return;
  }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  XFREE_CLEAR(qfl->qf_title);
  if (title != NULL) {
    qfl->qf_title = xstrdup(title);
  }
}

void *nvim_get_ql_info(void) { return (void *)ql_info; }

// Phase 2 accessors: buf_T/win_T field access for qf_mark_adjust_entry and qf_jump_first
int nvim_buf_get_has_qf_entry(const void *buf_void) { return ((const buf_T *)buf_void)->b_has_qf_entry; }
int nvim_qf_buf_get_fnum(const void *buf_void) { return ((const buf_T *)buf_void)->b_fnum; }
void *nvim_buf_win_get_llist(const void *win_void) { return ((const win_T *)win_void)->w_llist; }
// nvim_check_can_set_curbuf_forceit already defined in tag_shim.c

// Phase 2: rs_qf_jump_first
// rs_qf_mark_adjust_entry removed: exports as qf_mark_adjust via #[export_name].
extern void rs_qf_jump_first(void *qi, unsigned save_qfid, int forceit);

// Phase 3: qf_list_entry display
extern void rs_qf_list_entry(const void *qfp, int qf_idx, bool cursel,
                              int qfFile_hl_id, int qfSep_hl_id, int qfLine_hl_id);

// rs_ex_clist deleted: now exported as qf_list via #[export_name]

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
// rs_qf_open_new_cwindow deleted: Rust commands.rs uses #[link_name] directly.
// rs_did_set_quickfixtextfunc removed: exports as did_set_quickfixtextfunc via #[export_name].
// rs_qf_update_buffer deleted: Rust bypasses nvim_qf_update_buffer via #[link_name]
// rs_set_ref_in_quickfix removed: Rust eval gc.rs uses #[link_name] directly.
// rs_free_quickfix deleted: now exported as free_quickfix via #[export_name]

// Rust fold FFI declarations

// Phase 14: e_no_more_items, e_current_quickfix_list_was_changed,
// e_current_location_list_was_changed statics deleted; error strings inlined into wrappers.

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

void *nvim_qf_find_win_handle(const void *qi_void) { return (void *)rs_qf_find_win_for_stack(qi_void); }

int nvim_qf_win_get_handle(const void *wp_void) { return wp_void == NULL ? 0 : ((const win_T *)wp_void)->handle; }

// nvim_qflist_valid deleted: logic migrated to rs_qflist_valid Rust loop (Phase 16).
// nvim_qf_entry_present deleted: logic migrated to rs_qf_entry_present Rust loop (Phase 16).
// nvim_qf_types deleted: no callers; rs_qf_types handles logic directly (Phase 16).

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

size_t nvim_qf_sizeof_qfline(void) { return sizeof(qfline_T); }
size_t nvim_qf_sizeof_qflist(void) { return sizeof(qf_list_T); }
size_t nvim_qf_sizeof_qfinfo(void) { return sizeof(qf_info_T); }

/// Free qfline_T string fields and user_data, but NOT the struct itself.
void nvim_qfline_free_fields(void *qfp_void)
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

/// OR the appropriate flag into buf->b_has_qf_entry (used when adding entries).
void nvim_qf_buf_or_has_entry(void *buf_void, bool is_location_list)
{
  if (buf_void == NULL) { return; }
  buf_T *buf = (buf_T *)buf_void;
  buf->b_has_qf_entry |= is_location_list ? BUF_HAS_LL_ENTRY : BUF_HAS_QF_ENTRY;
}

// nvim_qf_get_fnum_for_entry deleted: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).
// nvim_qf_fix_fname deleted: migrated to Rust rs_qf_fix_fname (Phase 16).
// nvim_qf_is_printc deleted: no callers (Phase 16).

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


// qf_get_fnum forward declaration deleted: migrated to Rust rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).
// nvim_qf_get_fnum deleted: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).

void *nvim_qf_get_dir_stack(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_dir_stack; }
void nvim_qf_set_dir_stack(void *qfl_void, void *stack) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_dir_stack = (struct dir_stack_T *)stack; }
void *nvim_qf_get_file_stack(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_file_stack; }
void nvim_qf_set_file_stack(void *qfl_void, void *stack) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_file_stack = (struct dir_stack_T *)stack; }
const char *nvim_qf_get_directory(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_directory; }
void nvim_qf_set_directory(void *qfl_void, char *dir) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_directory = dir; }
const char *nvim_qf_get_currfile(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_currfile; }
void nvim_qf_set_currfile(void *qfl_void, char *file) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_currfile = file; }

// Phase 3 accessors: typval dict operations for property flag / index resolution functions

/// Return the string value of a dict key if it is VAR_STRING with value "$", else NULL.
bool nvim_tv_dict_find_str_is_dollar(const void *dict, const char *key, int key_len)
{
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, (ptrdiff_t)key_len);
  return di != NULL && di->di_tv.v_type == VAR_STRING
         && di->di_tv.vval.v_string != NULL
         && strequal(di->di_tv.vval.v_string, "$");
}

/// Add a number to a dict; returns OK or FAIL.
/// Add a string copy to a dict; returns OK or FAIL.
/// Allocate an empty list and add it to a dict; returns OK or FAIL.
// Phase 8 accessors: get_properties / set_properties cluster

/// Allocate a new list and set it as the return value (qf-specific void* version).
/// Allocate a new dict and set it as the return value (qf-specific void* version).
/// Allocate a plain dict_T (qf-specific void* version).
/// Append a dict to a list (qf-specific void* version).
/// Add a tv (copy) to a dict under 'key' of 'key_len'; returns OK or FAIL.
/// Allocate a new dictitem_T with the given key (length exclusive).
/// Add a dictitem_T to a dict (qf-specific void* version); returns OK or FAIL.
/// Free a dictitem_T including its di_tv (qf-specific void* version).
/// Copy a typval_T value (shallow copy with reference counting).
/// Serialize the qfl's quickfixtextfunc callback into a stack-allocated tv.
/// Returns true if callback was non-None.
bool nvim_qf_qftf_cb_put(void *qfl_void, void *tv_out)
{
  if (qfl_void == NULL || tv_out == NULL) {
    return false;
  }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  if (qfl->qf_qftf_cb.type == kCallbackNone) {
    return false;
  }
  callback_put(&qfl->qf_qftf_cb, (typval_T *)tv_out);
  return true;
}

/// Clear (free) an inline typval_T value without freeing the struct itself.
/// Get the v_type field of a typval_T (qf-specific void* version).
int nvim_qf_tv_get_type(const void *tv) { return tv == NULL ? VAR_UNKNOWN : ((const typval_T *)tv)->v_type; }

/// Get the dictitem's v_type.
int nvim_di_get_type(const void *di) { return di == NULL ? VAR_UNKNOWN : ((const dictitem_T *)di)->di_tv.v_type; }

/// Get the dictitem's vval.v_number.
int64_t nvim_di_get_nr(const void *di) { return di == NULL ? 0 : (int64_t)((const dictitem_T *)di)->di_tv.vval.v_number; }

/// Get the dictitem's vval.v_string (may be NULL).
const char *nvim_di_get_string(const void *di) { return di == NULL ? NULL : ((const dictitem_T *)di)->di_tv.vval.v_string; }

/// Get a pointer to the dictitem's di_tv (qf-specific void* version).
void *nvim_qf_di_get_tv(void *di) { return di == NULL ? NULL : (void *)&((dictitem_T *)di)->di_tv; }

/// Find a dictitem_T by key in a dict (key_len = -1 for NUL-terminated).
/// Look up the window for f_getloclist from argvars[0].
void *nvim_find_win_by_nr_or_id(const void *argvars_void)
{
  return (void *)find_win_by_nr_or_id((const typval_T *)argvars_void);
}

/// Advance the typval_T pointer by one element for argvars indexing.
void *nvim_tv_advance(const void *tv) { return (void *)((const typval_T *)tv + 1); }

/// Check if VAR_UNKNOWN type.
bool nvim_tv_is_unknown(const void *tv) { return tv == NULL || ((const typval_T *)tv)->v_type == VAR_UNKNOWN; }

/// Check if VAR_DICT type.
bool nvim_tv_is_dict(const void *tv) { return tv != NULL && ((const typval_T *)tv)->v_type == VAR_DICT; }

/// Get vval.v_dict pointer from typval (qf-specific void* version).
void *nvim_qf_tv_get_dict(const void *tv) { return (tv == NULL || ((const typval_T *)tv)->v_type != VAR_DICT) ? NULL : ((const typval_T *)tv)->vval.v_dict; }

/// Get a list from a typval_T (qf-specific void* version).
void *nvim_qf_tv_get_list(const void *tv) { return (tv == NULL || ((const typval_T *)tv)->v_type != VAR_LIST) ? NULL : ((const typval_T *)tv)->vval.v_list; }

/// Get the qfl->qf_ctx as a raw pointer (NULL if not set).
void *nvim_qfl_get_ctx(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_ctx; }

/// tv_dict_add_list: add an existing list to a dict (qf-specific); returns OK or FAIL.
/// Allocate a list (qf-specific void* version).
/// Check if a dict has 'lines' key with a VAR_LIST value and non-NULL list.
bool nvim_tv_dict_has_lines_key(const void *dict)
{
  if (dict == NULL) {
    return false;
  }
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("lines"));
  return di != NULL && di->di_tv.v_type == VAR_LIST && di->di_tv.vval.v_list != NULL;
}

/// Get the di_tv pointer for the "lines" dictitem in a dict (NULL if not found / wrong type).
void *nvim_tv_dict_get_lines_di_tv(const void *dict)
{
  if (dict == NULL) { return NULL; }
  dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("lines"));
  if (di == NULL || di->di_tv.v_type != VAR_LIST) { return NULL; }
  return &di->di_tv;
}

/// Get valid_bufnr for a qfline (check with buflist_findnr).
int nvim_qfline_get_valid_bufnr(const void *qfp_void)
{
  if (qfp_void == NULL) {
    return 0;
  }
  const qfline_T *qfp = (const qfline_T *)qfp_void;
  int bufnum = qfp->qf_fnum;
  if (bufnum != 0 && buflist_findnr(bufnum) == NULL) {
    bufnum = 0;
  }
  return bufnum;
}

// nvim_qfl_get_{index,count,id,changedtick,title} deleted: merged into nvim_qf_get_* (Phase 15).

/// qf_alloc_stack wrapper for internal stacks (used by qf_get_list_from_lines).
void *nvim_qf_alloc_internal_stack(void) { return rs_qf_alloc_stack(QFLT_INTERNAL, 1); }

/// qf_free_lists wrapper (free qi->qf_lists array after iterating).
void nvim_qf_free_lists_for_qi(void *qi_void)
{
  if (qi_void == NULL) { return; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  for (int i = 0; i < qi->qf_listcount; i++) {
    rs_qf_free_list(&qi->qf_lists[i]);
  }
  nvim_qf_free_lists_array(qi_void);
}

/// Get the "efm" string from a what dict (NULL if missing or wrong type).
const char *nvim_tv_dict_get_efm_str(const void *dict)
{
  if (dict == NULL) { return NULL; }
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("efm"));
  if (di == NULL) { return NULL; }
  if (di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL) {
    return NULL;  // wrong type/empty - will cause FAIL from caller
  }
  return di->di_tv.vval.v_string;
}

/// Check if "efm" key exists and has wrong type (not VAR_STRING or NULL string).
bool nvim_tv_dict_efm_wrong_type(const void *dict)
{
  if (dict == NULL) { return false; }
  const dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("efm"));
  if (di == NULL) { return false; }
  return di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL;
}

/// Get the qfl list handle at index qf_idx from qi.
void *nvim_qf_get_list_handle(const void *qi_void, int qf_idx)
{
  if (qi_void == NULL) { return NULL; }
  const qf_info_T *qi = (const qf_info_T *)qi_void;
  if (qf_idx < 0 || qf_idx >= qi->qf_listcount) { return NULL; }
  return (void *)&qi->qf_lists[qf_idx];
}

// Phase 8 set-side accessors

/// tv_dict_get_string: get string from dict by key (alloc=true means heap copy, qf void* version).
/// tv_dict_get_number: get number from dict by key (0 if not found, qf void* version).
/// tv_dict_get_tv: copy tv from dict key into *tv_out (VAR_UNKNOWN if not found).
/// tv_get_number_chk: get number from typval (qf void* version).
/// tv_get_string_chk: get string from typval (qf void* version, NULL on error).
/// tv_free: free a heap-allocated typval_T (qf void* version).
/// Allocate a heap typval_T (zeroed).
/// tv_copy from src tv into a newly allocated heap typval_T.
/// Free the qfl->qf_ctx field (tv_free + set to NULL).
void nvim_qfl_free_ctx(void *qfl_void)
{
  if (qfl_void == NULL) { return; }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  tv_free(qfl->qf_ctx);
  qfl->qf_ctx = NULL;
}

/// Set qfl->qf_ctx to the given heap typval_T pointer.
void nvim_qfl_set_ctx(void *qfl_void, void *ctx_tv)
{
  if (qfl_void != NULL) {
    ((qf_list_T *)qfl_void)->qf_ctx = (typval_T *)ctx_tv;
  }
}

/// callback_free on qfl->qf_qftf_cb.
void nvim_qfl_free_qftf_cb(void *qfl_void)
{
  if (qfl_void != NULL) {
    callback_free(&((qf_list_T *)qfl_void)->qf_qftf_cb);
  }
}

/// callback_from_typval into qfl->qf_qftf_cb.
/// Returns true if callback was valid.
bool nvim_qfl_set_qftf_cb_from_tv(void *qfl_void, void *tv_void)
{
  if (qfl_void == NULL || tv_void == NULL) { return false; }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  Callback cb;
  if (rs_callback_from_typval(&cb, (const typval_T *)tv_void)) {
    qfl->qf_qftf_cb = cb;
    return true;
  }
  return false;
}

// nvim_qfl_set_title_from_what deleted: inlined into rs_qf_set_properties (Phase 15)
// nvim_qfl_set_items deleted: inlined into rs_qf_set_properties (Phase 15)
// nvim_qfl_set_items_from_lines deleted: inlined into rs_qf_set_properties (Phase 15)
// nvim_qfl_set_curidx deleted: inlined into rs_qf_set_properties (Phase 15)
// nvim_qfl_list_changed_and_update_buf deleted: inlined into rs_qf_set_properties (Phase 15)
// nvim_qfl_list_changed deleted: callers use rs_qf_incr_changedtick directly (Phase 15)
// nvim_tv_dict_find_di_tv deleted: callers use nvim_tv_dict_find directly (Phase 15)
// nvim_tv_get_string_if_string deleted: no longer called from Rust (Phase 15)

/// Set typval_T vval.v_number (qf-specific void* version).
void nvim_qf_tv_set_number(void *tv_void, int64_t nr)
{
  if (tv_void != NULL) {
    ((typval_T *)tv_void)->vval.v_number = (varnumber_T)nr;
  }
}

/// Check if a typval_T has type VAR_LIST (qf-specific void* version).
bool nvim_qf_tv_is_list_type(const void *tv_void)
{
  return tv_void != NULL && ((const typval_T *)tv_void)->v_type == VAR_LIST;
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

// parse_efm_option, free_efm_list, nvim_qf_parse_efm_option, nvim_qf_free_efm_list,
// nvim_efm_get_* deleted: migrated to Rust EfmPattern in reader.rs (Phase 9).

// nvim_qf_state_* and qf_{setup,cleanup,get_nextline,grow_linebuf,get_next_*}_state
// deleted: migrated to Rust QfParserState in reader.rs (Phase 9).

// qf_win_pos_update forward declaration deleted: migrated to Rust rs_qf_win_pos_update (Phase 11).
// qf_update_buffer forward declaration deleted: migrated to Rust rs_qf_update_buffer (Phase 10 Pass 10 Phase 4).

// qf_find_win, qf_find_buf: deleted -- migrated to Rust rs_qf_find_win_for_stack /
// rs_qf_find_buf_for_stack in lib.rs (Phase 10, Pass 10), bodies deleted Phase 11.
// nvim_qf_find_win_for_stack, nvim_qf_find_buf_for_stack: deleted -- callers use
// rs_qf_find_win_for_stack / rs_qf_find_buf_for_stack directly.
// nvim_qf_update_buffer deleted: Rust bypasses via #[link_name = "rs_qf_update_buffer"]
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
extern void rs_locstack_queue_delreq(void *qi);
extern void rs_ll_free_all(void **pqi);

// ---- Phase 3 stack-allocation accessors ----

/// Returns address of the static ql_info_actual (global quickfix stack).
void *nvim_get_ql_info_actual(void) { return (void *)&ql_info_actual; }

/// Allocate a zeroed qf_info_T on the heap.
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

/// Phase 11: Realloc qi->qf_lists to n elements and zero-fill new entries.
/// Updates qi->qf_lists and qi->qf_maxcount.
void nvim_qf_resize_lists_array(void *qi_void, int n)
{
  if (qi_void == NULL) { return; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  size_t lsz = sizeof(*qi->qf_lists);
  int old_maxcount = qi->qf_maxcount;
  qf_list_T *new = xrealloc(qi->qf_lists, lsz * (size_t)n);
  if (n > old_maxcount) {
    memset(new + old_maxcount, 0, lsz * (size_t)(n - old_maxcount));
  }
  qi->qf_lists = new;
  qi->qf_maxcount = n;
}

/// Return wp->w_p_lhi (location history option value).
int nvim_win_get_p_lhi(const void *wp_void) { return wp_void == NULL ? 0 : (int)((const win_T *)wp_void)->w_p_lhi; }

/// Return true if cmdidx is a location-list command.
bool nvim_is_loclist_cmd(int cmdidx) { return is_loclist_cmd((cmdidx_T)cmdidx); }

// nvim_eap_get_cmdidx: already exists in ex_docmd.c

// ---- Phase 4 accessors ----

/// Set wp->w_llist_ref = qi (raw assignment; caller manages refcount separately).
void nvim_win_set_llist_ref(void *wp_void, void *qi_void)
{
  if (wp_void != NULL) {
    ((win_T *)wp_void)->w_llist_ref = (qf_info_T *)qi_void;
  }
}

// ---- Rust Phase 4 forward declarations ----
// rs_set_errorlist deleted: now exported as set_errorlist via #[export_name]

void *nvim_qf_get_ctx(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_ctx; }
bool nvim_qf_has_user_data(const void *qfl_void) { return qfl_void == NULL ? false : ((const qf_list_T *)qfl_void)->qf_has_user_data; }
void nvim_qf_incr_changedtick(void *qfl_void) { if (qfl_void != NULL) ((qf_list_T *)qfl_void)->qf_changedtick++; }

// Looking up a buffer can be slow if there are many.  Remember the last one
// to make this a lot faster if there are multiple matches in the same file.
static char *qf_last_bufname = NULL;
static bufref_T qf_last_bufref = { NULL, 0, 0 };

// quickfix_busy and qf_delq_head are now managed by Rust in lifecycle.rs.
// qfga static grow-array deleted (Phase 14): all C callers were inlined into Rust.

// qf_init_process_nextline deleted: inlined into Rust process_nextline in init.rs (Phase 9).

/// @returns -1 for error, number of errors for success.
// qf_init body migrated to rs_qf_init in Rust init.rs (Phase 16).
int qf_init(win_T *wp, const char *restrict efile, char *restrict errorformat, int newlist,
            const char *restrict qf_title, char *restrict enc)
{
  return rs_qf_init((void *)wp, efile, errorformat, (bool)newlist, qf_title, enc);
}

// LINE_MAXLEN deleted: migrated to Rust reader.rs (Phase 16).
// efm_to_regpat, fmt_start, free_efm_list, parse_efm_option deleted:
// migrated to Rust parse_efm_option / EfmPattern in reader.rs (Phase 9).

// callback function for 'quickfixtextfunc'
static Callback qftf_cb;

// qf_grow_linebuf deleted: migrated to Rust QfParserState::grow_linebuf (Phase 9).

// qf_get_next_str_line, qf_get_next_list_line, qf_get_next_buf_line,
// qf_get_next_file_line, qf_get_nextline deleted: migrated to Rust
// QfParserState methods in reader.rs (Phase 9).

// qf_get_list deleted: inlined as &qi->qf_lists[idx] (Phase 11).

// qf_parse_line (thin wrapper), qf_alloc_fields, qf_free_fields,
// qf_setup_state, qf_cleanup_state deleted: migrated to Rust (Phase 9).
// nvim_qf_init_alloc_fields, nvim_qf_init_free_fields deleted:
// replaced by rs_qf_alloc_fields / rs_qf_free_fields in Rust reader.rs (Phase 9).
// nvim_qf_init_update_efm_cache, s_fmt_first, s_last_efm deleted:
// replaced by rs_qf_init_update_efm_cache + EFM_CACHE in Rust reader.rs (Phase 9).

// nvim_qf_init_setup_state, nvim_qf_init_cleanup_state deleted:
// replaced by rs_qf_parser_state_new / rs_qf_parser_state_free in Rust (Phase 9).

// nvim_qf_init_clear_last_bufname deleted: Rust calls nvim_qf_clear_fnum_cache instead (Phase 16).
// nvim_qf_init_resolve_efm deleted: logic inlined into rs_qf_init_ext in Rust init.rs (Phase 16).
// nvim_qf_init_process_nextline, nvim_qf_init_state_no_fd_error deleted:
// inlined into Rust rs_qf_init_ext / process_nextline (Phase 9).

// nvim_qf_init_finalize_list deleted: inlined into Rust init.rs (Phase 14).

void nvim_qf_init_emsg_readerrf(void) { emsg(_(e_readerrf)); }

_Static_assert(QF_END_OF_INPUT == 2, "QF_END_OF_INPUT must be 2");
_Static_assert(QF_FAIL == 0, "QF_FAIL must be 0");

// qf_cmdtitle deleted: inlined as local char[IOSIZE] + rs_qf_cmdtitle (Phase 11).

// qf_get_curlist deleted: inlined as &qi->qf_lists[qi->qf_curlist] (Phase 11).

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

// nvim_qf_expand_env deleted: Rust calls expand_env directly.
// nvim_qf_os_path_exists deleted: Rust calls os_path_exists directly.
// nvim_qf_buflist_findnr_exists deleted: Rust calls buflist_findnr directly.

// =============================================================================
// Phase 5: qffields_T accessors deleted: migrated to Rust QfAllFields in reader.rs (Phase 9).
// nvim_efm_get_prog, nvim_efm_set_prog deleted:
// Rust parse.rs uses EfmPattern directly (inline struct field access).
// nvim_qf_regmatch_create_ic, nvim_qf_regmatch_extract_prog, nvim_qf_vim_regexec
// remain as C wrappers for vim regex lifecycle (parse.rs calls these for pattern matching).
// =============================================================================

/// Create a regmatch_T on the heap, set rm_ic=true, and assign the given prog.
/// Returns an opaque handle. The caller owns the memory; free after extracting prog.
void *nvim_qf_regmatch_create_ic(void *prog)
{
  regmatch_T *rm = xcalloc(1, sizeof(regmatch_T));
  rm->rm_ic = true;
  rm->regprog = (regprog_T *)prog;
  return rm;
}

/// Create a regmatch_T on the heap with given ic flag, assign the given prog.
/// Returns an opaque handle. The caller owns the memory; free after extracting prog.
void *nvim_qf_regmatch_create(void *prog, bool ic)
{
  regmatch_T *rm = xcalloc(1, sizeof(regmatch_T));
  rm->rm_ic = ic;
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

// nvim_qfline_append_text deleted: inlined into Rust parse.rs (Phase 14).

/// Replace qf_text with the given string (xfrees old, xstrdups new).
/// Used by Rust when it has already built the concatenated string.
void nvim_qfline_replace_text(void *qfp_void, const char *text)
{
  if (qfp_void == NULL) {
    return;
  }
  qfline_T *qfp = (qfline_T *)qfp_void;
  xfree(qfp->qf_text);
  qfp->qf_text = text != NULL ? xstrdup(text) : NULL;
}

/// Wrapper for line_breakcheck().
/// Call vim_isprintc() - returns nonzero if the char is printable.
// nvim_qf_get_fnum_for_fields deleted: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).

/// Move memory: STRMOVE(dst, src) - move overlapping memory.
/// Get IObuff pointer for reuse in file_pfx multiscan.
/// skipwhite wrapper.
// =============================================================================
// Phase 9: Reader state accessors for Rust QfParserState
// =============================================================================

// nvim_qf_fclose deleted: Rust uses fclose (libc) directly with null check.
// nvim_qf_fgets deleted: Rust uses fgets (libc) directly.
// nvim_qf_vim_fgets deleted: Rust uses vim_fgets directly.
// nvim_os_fopen_read deleted: Rust uses os_fopen(fname, "r") directly.

// nvim_qf_remove_bom deleted: Rust calls remove_bom directly.

/// Return sizeof(vimconv_T) for use in Rust xcalloc calls.
size_t nvim_qf_sizeof_vimconv(void) { return sizeof(vimconv_T); }

// nvim_qf_alloc_vimconv deleted: Rust uses xcalloc(1, nvim_qf_sizeof_vimconv()).
// nvim_qf_free_vimconv deleted: Rust uses xfree directly.

/// Setup encoding conversion: convert_setup(vc, from, p_enc).
/// enc may be NULL (no conversion set up in that case).
void nvim_qf_convert_setup(void *vc, const char *enc)
{
  if (vc == NULL) {
    return;
  }
  if (enc != NULL && *enc != NUL) {
    convert_setup((vimconv_T *)vc, (char *)enc, p_enc);
  }
}

/// Cleanup encoding conversion: convert_setup(vc, NULL, NULL).
void nvim_qf_convert_setup_cleanup(void *vc)
{
  if (vc != NULL) {
    ((vimconv_T *)vc)->vc_type = CONV_NONE;
    convert_setup((vimconv_T *)vc, NULL, NULL);
  }
}

/// Return vc->vc_type (CONV_NONE == 0).
int nvim_qf_vc_type(const void *vc) { return vc == NULL ? 0 : ((const vimconv_T *)vc)->vc_type; }

// nvim_qf_string_convert_with_len deleted: Rust calls string_convert directly.
// nvim_qf_ml_get_buf deleted: Rust calls ml_get_buf directly.
// nvim_qf_ml_get_buf_len deleted: Rust calls ml_get_buf_len directly.

/// Return IObuff pointer.
/// Return IOSIZE constant.
/// xmalloc wrapper for growbuf allocation.
/// xrealloc wrapper for growbuf grow.
/// xfree wrapper for growbuf free.
/// xstrlcpy: copy at most n-1 bytes of src to dst, always NUL-terminate.
/// Return true if tv is VAR_STRING.
bool nvim_qf_tv_is_string(const void *tv_void)
{
  return tv_void != NULL && ((const typval_T *)tv_void)->v_type == VAR_STRING;
}

/// Return the vval.v_string field of a VAR_STRING typval.
char *nvim_qf_tv_get_string(void *tv_void)
{
  return tv_void == NULL ? NULL : ((typval_T *)tv_void)->vval.v_string;
}

/// Get the first list item from a VAR_LIST typval (or NULL).
void *nvim_qf_tv_list_first(void *tv_void)
{
  if (tv_void == NULL) {
    return NULL;
  }
  const typval_T *tv = (const typval_T *)tv_void;
  if (tv->v_type != VAR_LIST || tv->vval.v_list == NULL) {
    return NULL;
  }
  return tv_list_first(tv->vval.v_list);
}

// nvim_qf_tv_get_list: use existing declaration at line ~1046 (nvim_qf_tv_get_list(const void*)).

/// Return the next list item (TV_LIST_ITEM_NEXT).
void *nvim_qf_list_item_next(const void *list, const void *li)
{
  if (list == NULL || li == NULL) {
    return NULL;
  }
  return TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li);
}

/// Return true if the list item has a non-null string value.
bool nvim_qf_list_item_is_string(const void *li)
{
  if (li == NULL) {
    return false;
  }
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  return tv->v_type == VAR_STRING && tv->vval.v_string != NULL;
}

/// Return the string value of a list item (or NULL if not a string/null string).
char *nvim_qf_list_item_string(void *li)
{
  if (li == NULL) {
    return NULL;
  }
  const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li);
  if (tv->v_type != VAR_STRING || tv->vval.v_string == NULL) {
    return NULL;
  }
  return tv->vval.v_string;
}

/// vim_strchr on a mutable char* with char NL character.
/// Returns pointer to first NL in str, or NULL.
// =============================================================================
// Phase 9 (Phase 2): vim_regcomp/vim_regfree wrappers and efm error messages
// =============================================================================

// nvim_qf_vim_regcomp deleted: Rust calls vim_regcomp directly.
// nvim_qf_vim_regfree deleted: Rust calls vim_regfree directly.

/// Wrapper for xstrdup used by Rust's EFM cache.
// qf_parse_fmt_f and all qf_parse_fmt_* functions deleted: migrated to Rust rs_qf_parse_match.

// All qf_parse_fmt_* functions, copy_nonerror_line, qf_parse_match, qf_parse_get_fields,
// qf_parse_dir_pfx, qf_parse_file_pfx, qf_parse_line_nomatch, and qf_parse_multiline_pfx
// have been deleted. They are now implemented in Rust in src/nvim-rs/quickfix/src/parse.rs
// as rs_qf_parse_match and helpers called from rs_qf_parse_line.

// locstack_queue_delreq deleted: migrated to Rust rs_locstack_queue_delreq in lifecycle.rs.

// qf_stack_get_bufnr, qf_free_all, check_quickfix_busy, qf_resize_stack, ll_resize_stack
// deleted: migrated to Rust with #[export_name] exporting under the C names directly.

// qf_resize_stack_base deleted: migrated to Rust rs_qf_resize_stack_base (Phase 11).

void qf_init_stack(void) { ql_info = (qf_info_T *)rs_qf_alloc_stack(QFLT_QUICKFIX, (int)p_chi); }

// qf_sync_llw_to_win deleted: migrated to Rust rs_qf_sync_llw_to_win (Phase 11).
// qf_sync_win_to_llw deleted: migrated to Rust rs_qf_sync_win_to_llw (Phase 11).

// qf_alloc_stack, qf_alloc_list_stack, ll_get_or_alloc_list,
// qf_cmd_get_stack, qf_cmd_get_or_alloc_stack deleted: migrated to Rust in lifecycle.rs.
// Dead static wrappers removed in Phase 16.

// rs_copy_loclist_stack deleted: now exported as copy_loclist_stack via #[export_name]

// qf_get_fnum deleted: migrated to Rust rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).

// qf_find_help_win deleted (Phase 5): logic inlined into Rust rs_ex_helpgrep; public
// nvim_qf_find_help_win wrapper at line 2333 retained for navigate.rs.

// win_set_loclist, qf_find_win_with_loclist, qf_find_win_with_normal_buf,
// qf_goto_tabwin_with_file deleted: dead statics (Phase 16).
// Their nvim_qf_* public counterparts below are the Rust-callable versions.

// Rust exports for jump machinery
extern void rs_qf_jump_newwin(void *qi, int dir, int errornr, int forceit, bool newwin);

// Phase 15 thin accessors for inlining jump-open helpers into Rust navigate.rs

/// can_abandon(curbuf, forceit): check if current buffer can be abandoned.
bool nvim_can_abandon_curbuf(int forceit) { return can_abandon(curbuf, forceit); }

/// no_write_message(): emit "no write" warning.
void nvim_no_write_message(void) { no_write_message(); }

/// do_ecmd for help file: open fnum with ECMD_HIDE+ECMD_SET_HELP flags.
int nvim_do_ecmd_help(int fnum, int prev_winid)
{
  return do_ecmd(fnum, NULL, NULL, NULL, 1,
                 ECMD_HIDE + ECMD_SET_HELP,
                 prev_winid == curwin->handle ? curwin : NULL);
}

/// buflist_getfile with GETF_SETMARK|GETF_SWITCH flags.
int nvim_qf_buflist_getfile(int fnum, int forceit)
{
  return buflist_getfile(fnum, 1, GETF_SETMARK | GETF_SWITCH, forceit);
}

/// curwin->w_p_wfb accessor.
bool nvim_curwin_get_wfb(void) { return curwin->w_p_wfb; }

/// win_id2wp: return win_T* for a window handle id.
void *nvim_win_id2wp(int id) { return win_id2wp(id); }

/// Set p_swb to empty_string_option and swb_flags to 0.
void nvim_qf_set_swb_empty_option(void) { p_swb = empty_string_option; swb_flags = 0; }

/// Check if prevwin is valid and usable for winfixbuf goto.
bool nvim_qf_prevwin_valid_for_wfb(void)
{
  return rs_win_valid(prevwin) && !prevwin->w_p_wfb && !bt_quickfix(prevwin->w_buffer);
}

// nvim_qf_jump_open_help, nvim_qf_jump_open_file, nvim_qf_jump_loc_win_closed
// deleted: logic inlined into rs_qf_jump_edit_buffer (Phase 15).

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

// nvim_qf_open_new_file_win deleted: logic inlined into rs_qf_jump_to_win (Phase 15).

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

void nvim_qf_win_set_loclist(void *win, void *qi)
{
  ((win_T *)win)->w_llist = (qf_info_T *)qi;
  ((qf_info_T *)qi)->qf_refcount++;
}

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

// Phase 14 Phase 2: Thin C accessors replacing nvim_qf_jump_goto_line and
// nvim_qf_jump_print_msg (both deleted; logic inlined into Rust navigate.rs).

linenr_T nvim_qf_curbuf_line_count(void) { return curbuf->b_ml.ml_line_count; }
void nvim_qf_curwin_set_col(int col) { curwin->w_cursor.col = col; }
void nvim_qf_curwin_set_coladd_zero(void) { curwin->w_cursor.coladd = 0; }
void nvim_qf_curwin_set_curswant(void) { curwin->w_set_curswant = true; }
void nvim_qf_coladvance(int col) { coladvance(curwin, col); }
void nvim_qf_beginline_white_fix(void) { beginline(BL_WHITE | BL_FIX); }
bool nvim_qf_do_search_pattern(const char *pat)
{
  pos_T save_cursor = curwin->w_cursor;
  curwin->w_cursor.lnum = 0;
  if (!do_search(NULL, '/', '/', (char *)pat, strlen(pat), 1, SEARCH_KEEP, NULL)) {
    curwin->w_cursor = save_cursor;
    return false;
  }
  return true;
}
int nvim_qf_get_curlist_count(const void *qi_void)
{
  const qf_info_T *qi = (const qf_info_T *)qi_void;
  return qi->qf_lists[qi->qf_curlist].qf_count;
}
bool nvim_qfline_get_cleared_bool(const void *qfp_void)
{
  return ((const qfline_T *)qfp_void)->qf_cleared != 0;
}
char nvim_qfline_get_type_char(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_type; }
int nvim_qfline_get_nr_int(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_nr; }
const char *nvim_qfline_get_text_ptr(const void *qfp_void) { return ((const qfline_T *)qfp_void)->qf_text; }
const char *nvim_qf_gettext_line_deleted(void) { return _(" (line deleted)"); }

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
void nvim_qf_win_goto_lnum(void *win_void, linenr_T lnum) { nvim_qf_win_goto_impl(win_void, lnum); }
linenr_T nvim_qf_win_get_cursor_lnum(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_cursor.lnum; }
linenr_T nvim_qf_win_get_buf_line_count(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_buffer->b_ml.ml_line_count; }
int nvim_qf_win_get_width(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_width; }
int nvim_qf_win_get_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_height; }
int nvim_qf_win_get_hsep_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_hsep_height; }
int nvim_qf_win_get_status_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_status_height; }

int nvim_qf_cmdline_row(void) { return (int)cmdline_row; }


// nvim_qf_open_new_cwindow deleted: Rust commands.rs now uses #[link_name = "rs_qf_open_new_cwindow"].
// nvim_qf_set_title_var deleted: dead code, only defined but never called externally

void nvim_qf_curwin_set_cursor(linenr_T lnum, int col) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; }

void nvim_qf_check_cursor_curwin(void) { check_cursor(curwin); }

void nvim_qf_update_topline_curwin(void) { update_topline(curwin); }

// nvim_qf_update_win_titlevar deleted: dead code, only defined but never called externally

// Phase 10 Pass 10 Phase 2: New C accessors for Rust implementations

/// Set w_redraw_top and w_redraw_bot on a window.
void nvim_qf_win_set_redraw_bounds(void *win_void, linenr_T top, linenr_T bot)
{
  if (win_void == NULL) { return; }
  win_T *win = (win_T *)win_void;
  win->w_redraw_top = top;
  win->w_redraw_bot = bot;
}

/// Perform the qf_win_goto operation: save curwin, switch to win, set cursor,
/// update topline, redraw, restore curwin. (Migrated body from qf_win_goto.)
void nvim_qf_win_goto_impl(void *win_void, linenr_T lnum)
{
  if (win_void == NULL) { return; }
  win_T *win = (win_T *)win_void;
  win_T *old_curwin = curwin;
  curwin = win;
  curbuf = win->w_buffer;
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = 0;
  curwin->w_cursor.coladd = 0;
  curwin->w_curswant = 0;
  update_topline(curwin);
  redraw_later(curwin, UPD_VALID);
  curwin->w_redr_status = true;
  curwin = old_curwin;
  curbuf = curwin->w_buffer;
}

/// Set the w:quickfix_title window variable for the current window.
/// Only sets if qfl->qf_title is not NULL.
void nvim_qf_set_title_var_for_list(void *qfl_void)
{
  if (qfl_void == NULL) { return; }
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  if (qfl->qf_title != NULL) {
    set_internal_string_var("w:quickfix_title", qfl->qf_title);
  }
}

/// Save and return the current curwin pointer.
void *nvim_qf_save_curwin(void) { return curwin; }

/// Restore curwin to a previously saved pointer.
void nvim_qf_restore_curwin(void *saved) { curwin = (win_T *)saved; }

/// Set curwin to the given window (for qf_update_win_titlevar pattern).
void nvim_qf_set_curwin(void *win_void) { curwin = (win_T *)win_void; }

// Phase 10 Pass 10 Phase 3: New C accessors for qf_open_new_cwindow / did_set_quickfixtextfunc

/// Set buffer options for the quickfix/location list window (swapfile, buftype, bufhidden,
/// foldmethod). Also resets key bindings and w_p_diff. (Migrated body from qf_set_cwindow_options.)
void nvim_qf_set_cwindow_options(void)
{
  set_option_value_give_err(kOptSwapfile, BOOLEAN_OPTVAL(false), OPT_LOCAL);
  set_option_value_give_err(kOptBuftype, STATIC_CSTR_AS_OPTVAL("quickfix"), OPT_LOCAL);
  set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("hide"), OPT_LOCAL);
  RESET_BINDING(curwin);
  curwin->w_p_diff = false;
  set_option_value_give_err(kOptFoldmethod, STATIC_CSTR_AS_OPTVAL("manual"), OPT_LOCAL);
}

/// do_ecmd for an existing quickfix buffer (ECMD_HIDE + ECMD_OLDBUF + ECMD_NOWINENTER).
/// oldwin may be NULL. Returns OK (1) or FAIL (0).
int nvim_qf_do_ecmd_existing_buf(int fnum, void *oldwin_void)
{
  return do_ecmd(fnum, NULL, NULL, NULL, ECMD_ONE,
                 ECMD_HIDE + ECMD_OLDBUF + ECMD_NOWINENTER,
                 (win_T *)oldwin_void);
}

/// do_ecmd creating a new quickfix buffer (ECMD_HIDE + ECMD_NOWINENTER).
/// oldwin may be NULL. Returns OK (1) or FAIL (0).
int nvim_qf_do_ecmd_new_buf(void *oldwin_void)
{
  return do_ecmd(0, NULL, NULL, NULL, ECMD_ONE,
                 ECMD_HIDE + ECMD_NOWINENTER,
                 (win_T *)oldwin_void);
}

/// Return the current tab page pointer (curtab).
const void *nvim_qf_get_curtab(void) { return curtab; }

// nvim_qf_curwin_width already defined at line 2529 (nvim_qf_get_columns section).

/// Set curwin->w_llist_ref = qi and increment qi->qf_refcount.
/// Only do this when qi is a location list stack (IS_LL_STACK).
void nvim_qf_curwin_set_llist_ref_incr(void *qi_void)
{
  if (qi_void == NULL) { return; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  curwin->w_llist_ref = qi;
  qi->qf_refcount++;
}

/// Set curwin->w_p_wfh to true (winfixheight).
void nvim_qf_curwin_set_wfh(void) { curwin->w_p_wfh = true; }

/// Reset key bindings on curwin (RESET_BINDING).
void nvim_qf_curwin_reset_binding(void) { RESET_BINDING(curwin); }

/// Set prevwin to the given window pointer.
void nvim_qf_set_prevwin(void *win_void) { prevwin = (win_T *)win_void; }

/// Check if the current tab page equals a previously saved tab page pointer.
bool nvim_qf_curtab_eq(const void *saved_tab) { return curtab == (const tabpage_T *)saved_tab; }

/// Call option_set_callback_func(p_qftf, &qftf_cb). Returns FAIL or OK.
int nvim_qf_option_set_callback_func_for_qftf(void)
{
  return option_set_callback_func(p_qftf, &qftf_cb);
}

/// Return the e_invarg error string pointer (for rs_did_set_quickfixtextfunc).
const char *nvim_qf_get_e_invarg(void) { return e_invarg; }

/// Return true if curwin equals the given window (for oldwin != curwin check).
bool nvim_qf_curwin_is(const void *win_void) { return curwin == (const win_T *)win_void; }

// Phase 10 Pass 10 Phase 4: New C accessors for rs_qf_update_buffer

/// get_region_bytecount for a quickfix buffer.
bcount_t nvim_qf_get_region_bytecount(void *buf, linenr_T l1, linenr_T l2, colnr_T c1, colnr_T c2)
{
  return get_region_bytecount((buf_T *)buf, l1, l2, c1, c2);
}

/// extmark_splice for quickfix buffer updates.
void nvim_qf_extmark_splice(void *buf, int r1, colnr_T c1, int r2, colnr_T c2,
                             bcount_t bc, int nr, colnr_T nc, bcount_t nbc)
{
  extmark_splice((buf_T *)buf, r1, c1, r2, c2, bc, nr, nc, nbc, kExtmarkNoUndo);
}

/// changed_lines for quickfix buffer updates.
void nvim_qf_changed_lines(void *buf, linenr_T lnum, colnr_T col, linenr_T lnume, linenr_T xtra, bool do_win)
{
  changed_lines((buf_T *)buf, lnum, col, lnume, xtra, do_win);
}

/// Set buf->b_changed = false.
void nvim_qf_buf_set_changed_false(void *buf) { ((buf_T *)buf)->b_changed = false; }

/// redraw_buf_later(buf, UPD_NOT_VALID).
void nvim_qf_redraw_buf_later(void *buf) { redraw_buf_later((buf_T *)buf, UPD_NOT_VALID); }

/// Return win->w_botline.
linenr_T nvim_qf_win_botline(const void *win) { return ((const win_T *)win)->w_botline; }

/// Allocate aco_save_T, call aucmd_prepbuf(aco, buf), return aco pointer.
void *nvim_qf_aucmd_prepbuf_alloc(void *buf)
{
  aco_save_T *aco = xmalloc(sizeof(aco_save_T));
  aucmd_prepbuf(aco, (buf_T *)buf);
  return aco;
}

/// Call aucmd_restbuf(aco) and free the aco_save_T pointer.
void nvim_qf_aucmd_restbuf_free(void *aco_void)
{
  if (aco_void == NULL) { return; }
  aucmd_restbuf((aco_save_T *)aco_void);
  xfree(aco_void);
}

// qf_list deleted: now exported directly from Rust via #[export_name]
// Phase 14: qfFile_hl_id, qfSep_hl_id, qfLine_hl_id statics and qf_list_entry wrapper
// deleted -- Rust rs_ex_clist computes hl_ids via nvim_syn_name2id_qf.

// qf_mark_adjust deleted: Rust navigate.rs exports directly via #[export_name = "qf_mark_adjust"].

// Phase 10 Pass 10 Phase 5: C accessors for rs_qf_get_fnum

/// Check the filename cache: if bufname matches and the bufref is still valid,
/// return the cached buf_T pointer. Otherwise return NULL.
void *nvim_qf_fnum_cache_check(const char *bufname)
{
  if (bufname == NULL) { return NULL; }
  if (qf_last_bufname != NULL
      && strcmp(bufname, qf_last_bufname) == 0
      && bufref_valid(&qf_last_bufref)) {
    return qf_last_bufref.br_buf;
  }
  return NULL;
}

/// Update the filename cache: free old name, store new buf.
/// Always copies bufname (caller retains ownership of the passed pointer).
void nvim_qf_fnum_cache_update(const char *bufname, void *buf)
{
  xfree(qf_last_bufname);
  qf_last_bufname = xstrdup(bufname);
  set_bufref(&qf_last_bufref, (buf_T *)buf);
}

/// Return buflist_new(bufname, NULL, 0, BLN_NOOPT).
void *nvim_qf_buflist_new(char *bufname)
{
  return buflist_new(bufname, NULL, 0, BLN_NOOPT);
}

/// Return the fnum of a buf_T pointer.
int nvim_qf_buf_fnum_from_ptr(const void *buf_void)
{
  return buf_void == NULL ? 0 : ((const buf_T *)buf_void)->b_fnum;
}

/// Set b_has_qf_entry on a buf_T (is_qf_list: true=QF, false=LL).
void nvim_qf_buf_set_has_qf_entry(void *buf_void, bool is_qf_list)
{
  if (buf_void == NULL) { return; }
  buf_T *buf = (buf_T *)buf_void;
  buf->b_has_qf_entry = is_qf_list ? BUF_HAS_QF_ENTRY : BUF_HAS_LL_ENTRY;
}

/// Return vim_isAbsName(fname).
bool nvim_qf_vim_is_abs_name(const char *fname)
{
  return fname != NULL && vim_isAbsName(fname);
}

/// Return concat_fnames(dir, fname, true) -- caller must free.
char *nvim_qf_concat_fnames(const char *dir, const char *fname)
{
  return concat_fnames((char *)dir, (char *)fname, true);
}

/// Return IS_QF_LIST(qfl).
bool nvim_qf_is_qf_list(const void *qfl_void)
{
  return qfl_void != NULL && IS_QF_LIST((const qf_list_T *)qfl_void);
}

/// Return nvim_qf_init_clear_last_bufname (XFREE_CLEAR(qf_last_bufname)).
void nvim_qf_clear_fnum_cache(void)
{
  XFREE_CLEAR(qf_last_bufname);
}

// qf_set_cwindow_options: deleted -- migrated to nvim_qf_set_cwindow_options accessor
// and Rust rs_qf_open_new_cwindow (Phase 10 Pass 10 Phase 3).

// qf_open_new_cwindow: deleted -- migrated to Rust rs_qf_open_new_cwindow
// (Phase 10 Pass 10 Phase 3). nvim_qf_open_new_cwindow now calls rs_qf_open_new_cwindow.

// qf_set_title_var deleted: migrated to Rust rs_qf_set_title_var (Phase 11).
// nvim_qf_set_title_var now calls rs_qf_set_title_var directly.

// qf_win_goto deleted: implementation moved to nvim_qf_win_goto_impl (Phase 10 Pass 10 Phase 2).

// Return the number of the current entry (line number in the quickfix window).

// qf_win_pos_update deleted: migrated to Rust rs_qf_win_pos_update (Phase 11).
// is_qf_win deleted: logic inlined into rs_qf_find_win_for_stack / rs_qf_win_pos_update (Phase 11).
// qf_find_win deleted: migrated to Rust rs_qf_find_win_for_stack (Phase 10/11).
// qf_find_buf deleted: migrated to Rust rs_qf_find_buf_for_stack (Phase 10/11).

// did_set_quickfixtextfunc deleted: Rust lib.rs exports directly via #[export_name = "did_set_quickfixtextfunc"].

// qf_update_win_titlevar deleted: migrated to Rust rs_qf_update_win_titlevar (Phase 11).

// qf_update_buffer deleted: migrated to Rust rs_qf_update_buffer (Phase 10 Pass 10 Phase 4).

// qf_buf_add_line migrated to Rust (Phase 3) -- see rs_qf_buf_add_line in display.rs

// call_qftf_func deleted: migrated to Rust rs_call_qftf_func in display.rs (Phase 11).

// C accessors for rs_call_qftf_func (Phase 11):

/// Allocate a new VAR_FIXED-locked dict.
/// Increment dict->dv_refcount by 1.
void nvim_tv_dict_incr_refcount(void *dict) { if (dict != NULL) ((dict_T *)dict)->dv_refcount++; }
/// Return true if callback cb has type kCallbackNone.
bool nvim_callback_is_none(const void *cb) { return cb == NULL || ((const Callback *)cb)->type == kCallbackNone; }

/// Call cb with a single VAR_DICT argument (dict) and write result to rettv.
/// Returns true on success.
bool nvim_callback_call_one_dict(void *cb, void *dict, void *rettv)
{
  if (cb == NULL || dict == NULL || rettv == NULL) { return false; }
  typval_T args[1];
  args[0].v_type = VAR_DICT;
  args[0].vval.v_dict = (dict_T *)dict;
  return callback_call((Callback *)cb, 1, args, (typval_T *)rettv);
}

/// If rettv->v_type == VAR_LIST, return rettv->vval.v_list; else NULL.
void *nvim_tv_rettv_list_if_var_list(const void *rettv_void)
{
  if (rettv_void == NULL) { return NULL; }
  const typval_T *rettv = (const typval_T *)rettv_void;
  return rettv->v_type == VAR_LIST ? (void *)rettv->vval.v_list : NULL;
}

/// tv_list_ref (qf-specific void* version): increment list reference count.
/// tv_dict_unref (qf-specific void* version): decrement dict reference count and free if zero.
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

// nvim_call_qftf_func deleted: Rust display.rs calls rs_call_qftf_func directly (Phase 11).

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
void nvim_ml_delete_one(linenr_T lnum) { ml_delete(lnum); }

// nvim_qf_set_filetype_and_autocmds deleted: inlined into Rust display.rs (Phase 14).

/// Increment curbuf->b_ro_locked.
void nvim_qf_curbuf_incr_ro_locked(void) { curbuf->b_ro_locked++; }
/// Decrement curbuf->b_ro_locked.
void nvim_qf_curbuf_decr_ro_locked(void) { curbuf->b_ro_locked--; }
/// Set curbuf->b_p_ma = false.
void nvim_qf_curbuf_set_ma_false(void) { curbuf->b_p_ma = false; }
/// Set curbuf->b_keep_filetype = val.
void nvim_qf_curbuf_set_keep_filetype(bool val) { curbuf->b_keep_filetype = val; }
/// Call set_option_value_give_err(kOptFiletype, "qf", OPT_LOCAL).
void nvim_qf_set_option_filetype_qf(void)
{
  set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("qf"), OPT_LOCAL);
}
/// Call apply_autocmds(EVENT_BUFREADPOST, "quickfix", NULL, false, curbuf).
void nvim_qf_apply_autocmds_bufreadpost_qf(void)
{
  apply_autocmds(EVENT_BUFREADPOST, "quickfix", NULL, false, curbuf);
}
/// Call apply_autocmds(EVENT_BUFWINENTER, "quickfix", NULL, false, curbuf).
void nvim_qf_apply_autocmds_bufwinenter_qf(void)
{
  apply_autocmds(EVENT_BUFWINENTER, "quickfix", NULL, false, curbuf);
}
/// Call redraw_curbuf_later(UPD_NOT_VALID).
void nvim_qf_redraw_curbuf_later(void) { redraw_curbuf_later(UPD_NOT_VALID); }

bool nvim_qf_get_key_typed(void) { return KeyTyped; }

void nvim_qf_set_key_typed(bool val) { KeyTyped = val; }

void *nvim_qf_get_start_nonnull(const void *qfl) { return qfl == NULL ? NULL : ((const qf_list_T *)qfl)->qf_start; }

// qf_list_changed deleted: callers use rs_qf_incr_changedtick directly (Phase 14).
// qf_jump_first deleted: callers use rs_qf_jump_first directly (Phase 14).

// grep_internal deleted: Rust commands.rs exports directly via #[export_name = "grep_internal"].


// Phase 7: C accessor wrappers needed by rs_ex_make / rs_make_get_fullcmd / rs_get_mef_name

// Global option accessors
const char *nvim_get_p_shq(void) { return p_shq; }
const char *nvim_get_p_sp(void) { return p_sp; }
const char *nvim_get_p_mef(void) { return p_mef; }
const char *nvim_get_p_efm(void) { return p_efm; }
const char *nvim_get_p_menc(void) { return p_menc; }
const char *nvim_get_p_gefm(void) { return p_gefm; }
const char *nvim_get_p_ef(void) { return p_ef; }

// curbuf option accessors
const char *nvim_curbuf_get_b_p_menc(void) { return curbuf->b_p_menc; }
const char *nvim_curbuf_get_b_p_gefm(void) { return curbuf->b_p_gefm; }
// Shell/message helpers
void nvim_append_redir(char *buf, size_t buflen, const char *opt, const char *name) { append_redir(buf, buflen, opt, name); }

// autowrite, shell, remove
void nvim_autowrite_all(void) { autowrite_all(); }
void nvim_do_shell(const char *cmd) { do_shell(cmd, 0); }

// vim_tempname wrapper
char *nvim_vim_tempname(void) { return vim_tempname(); }

// OS helpers for get_mef_name
int nvim_os_get_pid(void) { return (int)os_get_pid(); }
bool nvim_os_fileinfo_link_exists(const char *name) { FileInfo fi; return os_fileinfo_link(name, &fi); }

// curlist id accessor for quickfix list change tracking
unsigned nvim_qf_get_curlist_id(const void *qi_void)
{
  const qf_info_T *qi = (const qf_info_T *)qi_void;
  return qi->qf_lists[qi->qf_curlist].qf_id;
}

// eap line setters for cbuffer_process_args
// nvim_eap_set_line1, nvim_eap_set_line2 already declared in ex_docmd.c
// nvim_win_get_p_lhi returns int (OptInt) -- already in lifecycle/window module

// win_T lhi (p_lhi) setter for copy_loclist_stack
void nvim_win_set_p_lhi(void *win, int v) { ((win_T *)win)->w_p_lhi = (OptInt)v; }

// eap cmdlinep accessor (for qf_cmdtitle)
char *nvim_eap_get_cmdlinep_deref_make(const void *eap) { return *((const exarg_T *)eap)->cmdlinep; }

// set_option_direct wrapper for :cfile
void nvim_set_option_direct_ef(const char *val) { set_option_direct(kOptErrorfile, CSTR_AS_OPTVAL(val), 0, 0); }

// buf accessors for cbuffer_process_args
// Note: nvim_buf_has_ml_mfp in memline_shim.c takes buf_T* (not void*);
//       nvim_buf_get_ml_line_count in memline_shim.c takes buf_T*;
//       nvim_buf_get_sfname in buffer.c takes buf_T*.
// Use void* variants here so Rust can call them without needing buf_T layout.
bool nvim_buf_has_ml_mfp_void(const void *buf) { return ((const buf_T *)buf)->b_ml.ml_mfp != NULL; }
linenr_T nvim_buf_get_ml_line_count_void(const void *buf) { return ((const buf_T *)buf)->b_ml.ml_line_count; }
const char *nvim_buf_get_sfname_void(const void *buf) { return ((const buf_T *)buf)->b_sfname; }
void *nvim_buflist_findnr_ptr(int nr) { return (void *)buflist_findnr(nr); }
void *nvim_curbuf_ptr(void) { return (void *)curbuf; }
// eval_expr / tv_free wrappers for ex_cexpr
void *nvim_eval_expr(const void *arg_ptr, void *eap) { return (void *)eval_expr((char *)arg_ptr, (exarg_T *)eap); }
// nvim_tv_get_type: already defined in eval/typval.h (takes const typval_T*)
// nvim_tv_free: already defined in eval_shim.c (takes typval_T*)
// Use void* wrappers with different names to avoid conflicts.
int nvim_tv_get_type_void(const void *tv) { return ((const typval_T *)tv)->v_type; }
const char *nvim_tv_get_vval_string(const void *tv) { return ((const typval_T *)tv)->vval.v_string; }
bool nvim_tv_is_list(const void *tv) { return ((const typval_T *)tv)->v_type == VAR_LIST; }
void nvim_tv_free_void(void *tv) { tv_free((typval_T *)tv); }

// QuickFixCmdPre/Post autocmd wrappers for ex_make cluster
// Returns true if autocmd fired and aborting() is false (OK to continue),
// false if we should abort.
bool nvim_qf_apply_autocmd_pre(const char *au_name)
{
  if (au_name == NULL) {
    return true;
  }
  if (apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name, curbuf->b_fname, true, curbuf)) {
    if (aborting()) {
      return false;
    }
  }
  return true;
}

// Apply QuickFixCmdPre with explicit buffer fname (for :cfile which passes NULL).
bool nvim_qf_apply_autocmd_pre_null(const char *au_name)
{
  if (au_name == NULL) {
    return true;
  }
  if (apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name, NULL, false, curbuf)) {
    if (aborting()) {
      return false;
    }
  }
  return true;
}

void nvim_qf_apply_autocmd_post(const char *au_name)
{
  if (au_name != NULL) {
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
  }
}

void nvim_qf_apply_autocmd_post_null(const char *au_name)
{
  if (au_name != NULL) {
    apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, NULL, false, curbuf);
  }
}

// Returns true if curbuf changed during autocmd post (for ex_cbuffer curbuf tracking)
bool nvim_qf_apply_autocmd_post_track(const char *au_name)
{
  if (au_name == NULL) {
    return false;
  }
  const buf_T *const old = curbuf;
  apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
  return curbuf != old;
}

// IObuff/IOSIZE for cbuffer title formatting
// Note: nvim_qf_get_iobuff already defined above (returns char*).
void nvim_qf_snprintf_iobuff(const char *title, const char *sfname)
{
  vim_snprintf(IObuff, IOSIZE, "%s (%s)", title, sfname);
}

// GET_LOC_LIST wrapper
void *nvim_win_get_loclist_ptr(const void *wp) { return (void *)GET_LOC_LIST((const win_T *)wp); }

// copy_loclist_stack accessors
void *nvim_win_get_llist_or_ref(const void *from_win)
{
  const win_T *from = (const win_T *)from_win;
  return (void *)(IS_LL_WINDOW(from) ? from->w_llist_ref : from->w_llist);
}
void nvim_win_set_llist(void *to_win, void *qi) { ((win_T *)to_win)->w_llist = (qf_info_T *)qi; }
// nvim_win_get_p_lhi already defined at line 1121 (returns int).
// nvim_win_set_p_lhi defined earlier in this file.
// nvim_qi_{get,set}_{listcount,curlist}_qi and nvim_qi_get_maxcount_qi deleted:
// duplicates of nvim_qf_{get,set}_{listcount,curlist_idx,maxcount} (Phase 15).
void *nvim_qi_get_list_qi(void *qi, int idx) { return (void *)&((qf_info_T *)qi)->qf_lists[idx]; }
void nvim_qf_free_all_win(void *to_win) { qf_free_all((win_T *)to_win); }

// rs_ex_make, rs_ex_cfile, rs_ex_cbuffer, rs_ex_cexpr deleted: now exported via #[export_name]
// rs_copy_loclist_stack deleted: now exported as copy_loclist_stack via #[export_name]

// make_get_fullcmd deleted: dead static wrapper (Phase 16). Real impl in Rust rs_make_get_fullcmd.
// get_mef_name deleted: dead static wrapper (Phase 16). Real impl in Rust rs_get_mef_name.

// ex_make deleted: now exported directly from Rust via #[export_name]

// qf_get_size deleted: Rust commands.rs exports directly via #[export_name = "qf_get_size"].
// qf_get_valid_size deleted: Rust commands.rs exports directly via #[export_name = "qf_get_valid_size"].
// qf_get_cur_idx deleted: Rust commands.rs exports directly via #[export_name = "qf_get_cur_idx"].
// qf_get_cur_valid_idx deleted: Rust commands.rs exports directly via #[export_name = "qf_get_cur_valid_idx"].

// nvim_qf_cmd_get_stack deleted: Rust commands.rs now uses #[link_name = "rs_qf_cmd_get_stack"].

// nvim_qf_msg deleted: Rust bypasses via #[link_name = "rs_qf_msg"]

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

// Phase 14: Direct message output accessors (replacing nvim_qf_list_entry_output and
// nvim_qf_format_prefix which were deleted after inlining into Rust rs_qf_list_entry).
int nvim_hlf_qfl(void) { return HLF_QFL; }

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

// ex_cfile deleted: now exported directly from Rust via #[export_name]



// Phase 3 message API accessors for vgr_display_fname migration
// nvim_msg_start: already defined in undo.c
// nvim_msg_clr_eos: already defined in change_ffi.c
// nvim_ui_flush: already defined in change_ffi.c

// Phase 3 buffer management accessors
// nvim_buf_has_ml_mfp already exists in memline_shim.c (returns int, takes buf_T*)
// nvim_buflist_findname_exp already exists in window_shim.c (returns buf_T*, takes const char*)
const char *nvim_buf_get_mfp_fname(const void *buf)
{
  const buf_T *b = (const buf_T *)buf;
  if (b->b_ml.ml_mfp != NULL) {
    return b->b_ml.ml_mfp->mf_fname;
  }
  return NULL;
}
char nvim_buf_get_bh_first_char(const void *buf) { return ((const buf_T *)buf)->b_p_bh[0]; }
bool nvim_cmdmod_has_cmod_hide(void) { return (cmdmod.cmod_flags & CMOD_HIDE) != 0; }
void nvim_buf_clear_bf_dummy(void *buf) { ((buf_T *)buf)->b_flags &= ~BF_DUMMY; }
void nvim_wipe_dummy_buffer(void *buf, char *dirname_start) { wipe_dummy_buffer((buf_T *)buf, dirname_start); }
void nvim_unload_dummy_buffer(void *buf, char *dirname_start) { unload_dummy_buffer((buf_T *)buf, dirname_start); }
void *nvim_load_dummy_buf(const char *fname, char *dirname_start, char *dirname_now)
{
  return vgr_load_dummy_buf((char *)fname, dirname_start, dirname_now);
}

/// Apply Filetype autocmds and modelines to buf (for dummy buffer finalization).
void nvim_apply_filetype_autocmds_and_modelines(void *buf_void)
{
  buf_T *buf = (buf_T *)buf_void;
  aco_save_T aco;
  aucmd_prepbuf(&aco, buf);
  apply_autocmds(EVENT_FILETYPE, buf->b_p_ft, buf->b_fname, true, buf);
  do_modelines(OPT_NOWIN);
  aucmd_restbuf(&aco);
}

/// Execute ex_cd with either CMD_cd or CMD_lcd.
void nvim_ex_cd_arg(char *arg, bool is_lcd)
{
  exarg_T ea = {
    .arg = arg,
    .cmdidx = is_lcd ? CMD_lcd : CMD_cd,
  };
  ex_cd(&ea);
}

// Phase 3: path_try_shorten_fname wrapper (rename-compatible)
char *nvim_path_try_shorten_fname(const char *full_fname) { return path_try_shorten_fname((char *)full_fname); }

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

/// Heap-allocate and initialize a regmmatch_T for vimgrep.
/// Returns the heap pointer (caller must free with nvim_vgr_regmatch_free),
/// or NULL if compilation failed (error already emitted).
void *nvim_vgr_regcomp_init(const char *pat)
{
  regmmatch_T *rm = xcalloc(1, sizeof(regmmatch_T));
  rm->regprog = NULL;

  if (pat == NULL || *pat == NUL) {
    if (last_search_pat() == NULL) {
      emsg(_(e_noprevre));
      xfree(rm);
      return NULL;
    }
    rm->regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
  } else {
    rm->regprog = vim_regcomp((char *)pat, RE_MAGIC);
  }

  if (rm->regprog == NULL) {
    xfree(rm);
    return NULL;
  }

  rm->rmm_ic = p_ic;
  rm->rmm_maxcol = 0;
  return rm;
}

/// Free a heap-allocated regmmatch_T from nvim_vgr_regcomp_init.
void nvim_vgr_regmatch_free(void *rm_void)
{
  if (rm_void == NULL) {
    return;
  }
  regmmatch_T *rm = (regmmatch_T *)rm_void;
  vim_regfree(rm->regprog);
  xfree(rm);
}

/// Wrapper for get_arglist_exp.
int nvim_vgr_get_arglist_exp(const char *p, int *fcount_out, char ***fnames_out)
{
  return get_arglist_exp((char *)p, fcount_out, fnames_out, true);
}

/// FreeWild wrapper for vgr fnames.
void nvim_vgr_free_wild_raw(int fcount, char **fnames) { FreeWild(fcount, fnames); }

// apply_autocmds wrapper for QuickFixCmdPre/Post (for Phase 2 rs_ex_vimgrep).
bool nvim_apply_autocmds_quickfixcmdpre(const char *au_name)
{
  return apply_autocmds(EVENT_QUICKFIXCMDPRE, au_name, curbuf->b_fname, true, curbuf);
}
bool nvim_apply_autocmds_quickfixcmdpost(const char *au_name)
{
  apply_autocmds(EVENT_QUICKFIXCMDPOST, au_name, curbuf->b_fname, true, curbuf);
  return true;
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

// qf_winid deleted: migrated to Rust rs_qf_winid in lib.rs (Phase 10, Pass 10).

// nvim_qf_winid deleted: Rust api.rs now uses #[link_name = "rs_qf_winid"].

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

// qf_add_entry_from_dict + nvim_qf_add_entry_from_dict deleted:
// migrated to Rust rs_qf_add_entry_from_dict in list.rs (Phase 11).

/// Allocate a single null byte (empty C string). Caller must xfree/
/// Free a char * allocated by xmalloc/xstrdup/etc.
void nvim_xfree_char(char *ptr) { xfree(ptr); }

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


// set_errorlist deleted: Rust exports under the C name directly via #[export_name].

// Phase 10 Pass 10 Phase 6: C accessors for rs_set_ref_in_quickfix

/// Return a mutable pointer to qfl->qf_qftf_cb.
void *nvim_qfl_get_qftf_cb_ptr(void *qfl_void)
{
  return qfl_void == NULL ? NULL : (void *)&((qf_list_T *)qfl_void)->qf_qftf_cb;
}

/// Return a mutable pointer to the global qftf_cb static.
void *nvim_qf_get_global_qftf_cb_ptr(void)
{
  return (void *)&qftf_cb;
}

/// Return win->w_llist (mutable).
void *nvim_qf_win_get_llist_mut(void *win_void)
{
  return win_void == NULL ? NULL : ((win_T *)win_void)->w_llist;
}

/// Return win->w_llist_ref (mutable).
void *nvim_qf_win_get_llist_ref_mut(void *win_void)
{
  return win_void == NULL ? NULL : ((win_T *)win_void)->w_llist_ref;
}

/// Return true if IS_LL_WINDOW(win) and win->w_llist_ref->qf_refcount == 1.
bool nvim_qf_win_is_ll_and_refcount_one(const void *win_void)
{
  if (win_void == NULL) { return false; }
  const win_T *win = (const win_T *)win_void;
  return IS_LL_WINDOW(win) && win->w_llist_ref->qf_refcount == 1;
}

// nvim_qf_get_ql_info: use existing nvim_get_ql_info instead (Phase 10 Pass 10 Phase 6).


// mark_quickfix_user_data deleted: migrated to Rust rs_set_ref_in_quickfix (Phase 10 Pass 10 Phase 6).
// mark_quickfix_ctx deleted: migrated to Rust rs_set_ref_in_quickfix (Phase 10 Pass 10 Phase 6).

// set_ref_in_quickfix deleted: eval gc.rs bypasses via #[link_name = "rs_set_ref_in_quickfix"].

/// :cgetbuffer, :lbuffer, :laddbuffer, :lgetbuffer Ex commands.
// ":[range]cbuffer [bufnr]" command.
// ":[range]caddbuffer [bufnr]" command.
// ":[range]cgetbuffer [bufnr]" command.
// ":[range]lbuffer [bufnr]" command.
// ":[range]laddbuffer [bufnr]" command.
// ":[range]lgetbuffer [bufnr]" command.
// ex_cbuffer deleted: now exported directly from Rust via #[export_name]

// ex_cexpr deleted: now exported directly from Rust via #[export_name]

// hgr_get_ll deleted (Phase 5): logic inlined into Rust rs_ex_helpgrep.

// C accessor wrappers for rs_hgr_search_* functions (Phase 1 deleted, Phase 2 renamed)
// nvim_hgr_os_fopen deleted: use nvim_os_fopen_read
// nvim_hgr_vim_fgets deleted: use nvim_qf_vim_fgets
// nvim_hgr_vim_regexec deleted: use nvim_qf_vim_regexec
// nvim_hgr_regmatch_startp deleted: use nvim_qf_regmatch_startp(rmp, 0)
// nvim_hgr_regmatch_endp deleted: use nvim_qf_regmatch_endp(rmp, 0)
// nvim_hgr_fclose deleted: use nvim_qf_fclose
// nvim_hgr_get_IObuff deleted: use nvim_qf_get_iobuff
// nvim_hgr_get_IOSIZE deleted: use nvim_qf_get_iosize
// nvim_hgr_get_got_int deleted: use nvim_qf_got_int (returns bool)
// nvim_hgr_set_got_int deleted: use nvim_qf_set_got_int
// nvim_hgr_line_breakcheck deleted: use nvim_qf_line_breakcheck
// nvim_hgr_get_MAXPATHL deleted: use nvim_get_maxpathl (memline_shim.c, returns size_t)
// nvim_hgr_get_NameBuff deleted: use nvim_get_namebuff (buffer.c, returns char*)
int nvim_gen_expand_wildcards_file_silent(char *dirname, int *fcount_out, char ***fnames_out)
{
  return gen_expand_wildcards(1, &dirname, fcount_out, fnames_out, EW_FILE|EW_SILENT);
}
void nvim_free_wild(int fcount, char **fnames) { FreeWild(fcount, fnames); }
char *nvim_fname_at(char **fnames, int idx) { return fnames[idx]; }
void nvim_add_pathsep(char *dirname) { add_pathsep(dirname); }
void nvim_hgr_strcat_doc_glob(char *dirname) { strcat(dirname, "doc/*.\\(txt\\|??x\\)"); }  // NOLINT
int nvim_strnicmp(const char *a, const char *b, int n) { return STRNICMP(a, b, (size_t)n); }
char *nvim_get_p_rtp(void) { return p_rtp; }
void nvim_copy_option_part_comma(char **pp, char *buf, int maxlen)
{
  copy_option_part(pp, buf, (size_t)maxlen, ",");
}

// nvim_hgr_pre_check deleted (Phase 4): inlined into Rust using nvim_qf_apply_autocmd_pre.
// nvim_hgr_is_loclist_cmd deleted (Phase 4): use nvim_is_loclist_cmd + nvim_eap_get_cmdidx.
// nvim_hgr_get_ll deleted (Phase 4): inlined into Rust using nvim_qf_curwin_buf_is_help,
//   nvim_qf_find_help_win, nvim_qf_win_get_llist, rs_qf_alloc_stack.
// nvim_hgr_post_autocmd deleted (Phase 4): inlined into Rust using nvim_qf_apply_autocmd_post,
//   nvim_qf_is_ll_stack_qi, nvim_qf_find_win_with_loclist.
// nvim_hgr_jump_or_nomatch deleted (Phase 4): inlined into Rust using rs_qf_list_empty,
//   rs_qf_jump_newwin, nvim_semsg_nomatch2, nvim_eap_get_arg.
// nvim_hgr_is_lhelpgrep deleted (Phase 4): use nvim_eap_get_cmdidx comparison in Rust.
// nvim_hgr_cleanup deleted (Phase 4): inlined into Rust using nvim_qf_curwin_buf_is_help,
//   nvim_qf_get_curwin, nvim_qf_win_get_llist, rs_ll_free_all, nvim_win_set_llist.

// nvim_hgr_compile_and_search deleted (Phase 3): list creation and finalization inlined into Rust.
// nvim_check_help_lang deleted (Phase 16): call check_help_lang directly from Rust.
// nvim_hgr_regex_search deleted (Phase 16): inlined into rs_ex_helpgrep in Rust commands.rs.

/// Save p_cpo and set it to empty. Returns the old value as an opaque pointer.
void *nvim_save_cpo_set_empty(void)
{
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;
  return save_cpo;
}

/// Restore p_cpo from a saved value. Handles plugin interference.
void nvim_restore_cpo(void *saved_cpo_void)
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

// free_quickfix deleted: now exported directly from Rust via #[export_name] (EXITFREE guarded)

// f_getloclist, f_getqflist, f_setloclist, f_setqflist deleted:
// migrated to Rust with #[export_name] exporting under the C names directly.
