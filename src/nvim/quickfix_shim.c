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

// Phase 8: qf_get_properties / qf_set_properties cluster (migrated to Rust)
extern int rs_get_errorlist(const void *qi_arg, const void *wp, int qf_idx, int eidx, void *list);
extern int rs_qf_get_list_from_lines(const void *what, void *retdict);
extern int rs_qf_get_properties(const void *wp, void *what, void *retdict);
extern void rs_get_qf_loc_list(bool is_qf, void *wp, const void *what_arg, void *rettv);
extern void rs_f_getqflist(const void *argvars, void *rettv, void *fptr);
extern void rs_f_getloclist(const void *argvars, void *rettv, void *fptr);
extern int rs_qf_set_properties(void *qi, const void *what, int action, char *title);
extern void rs_set_qf_ll_list(void *wp, const void *args, void *rettv);
extern void rs_f_setqflist(const void *argvars, void *rettv, void *fptr);
extern void rs_f_setloclist(const void *argvars, void *rettv, void *fptr);

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
extern int rs_qf_open_new_cwindow(void *qi_void, int height);
extern const char *rs_did_set_quickfixtextfunc(const void *args);
extern void rs_qf_update_buffer(void *qi_void, const void *old_last);
extern bool rs_set_ref_in_quickfix(int copyID);

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

// nvim_qf_get_fnum_for_entry deleted: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).

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

// Phase 8 accessors: get_properties / set_properties cluster

/// Allocate a new list and set it as the return value (qf-specific void* version).
void nvim_qf_tv_list_alloc_ret(void *rettv_void)
{
  typval_T *rettv = (typval_T *)rettv_void;
  tv_list_alloc_ret(rettv, kListLenMayKnow);
}

/// Allocate a new dict and set it as the return value (qf-specific void* version).
void nvim_qf_tv_dict_alloc_ret(void *rettv_void)
{
  typval_T *rettv = (typval_T *)rettv_void;
  tv_dict_alloc_ret(rettv);
}

/// Allocate a plain dict_T (qf-specific void* version).
void *nvim_qf_tv_dict_alloc(void) { return tv_dict_alloc(); }

/// Append a dict to a list (qf-specific void* version).
void nvim_qf_tv_list_append_dict(void *list, void *dict)
{
  if (list != NULL && dict != NULL) {
    tv_list_append_dict((list_T *)list, (dict_T *)dict);
  }
}

/// Add a tv (copy) to a dict under 'key' of 'key_len'; returns OK or FAIL.
int nvim_tv_dict_add_tv(void *dict, const char *key, int key_len, void *tv)
{
  return tv_dict_add_tv((dict_T *)dict, key, (size_t)key_len, (typval_T *)tv);
}

/// Allocate a new dictitem_T with the given key (length exclusive).
void *nvim_tv_dict_item_alloc_len(const char *key, int key_len)
{
  return tv_dict_item_alloc_len(key, (size_t)key_len);
}

/// Add a dictitem_T to a dict (qf-specific void* version); returns OK or FAIL.
int nvim_qf_tv_dict_add_item(void *dict, void *item)
{
  return tv_dict_add((dict_T *)dict, (dictitem_T *)item);
}

/// Free a dictitem_T including its di_tv (qf-specific void* version).
void nvim_qf_tv_dict_item_free(void *item)
{
  if (item != NULL) {
    tv_dict_item_free((dictitem_T *)item);
  }
}

/// Copy a typval_T value (shallow copy with reference counting).
void nvim_tv_copy(const void *from, void *to)
{
  if (from != NULL && to != NULL) {
    tv_copy((const typval_T *)from, (typval_T *)to);
  }
}

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
void nvim_tv_clear(void *tv)
{
  if (tv != NULL) {
    tv_clear((typval_T *)tv);
  }
}

/// Get the v_type field of a typval_T (qf-specific void* version).
int nvim_qf_tv_get_type(const void *tv) { return tv == NULL ? VAR_UNKNOWN : ((const typval_T *)tv)->v_type; }

/// Get the vval.v_number field of a typval_T.
int64_t nvim_tv_get_vval_nr(const void *tv) { return tv == NULL ? 0 : (int64_t)((const typval_T *)tv)->vval.v_number; }

/// Get the dictitem's v_type.
int nvim_di_get_type(const void *di) { return di == NULL ? VAR_UNKNOWN : ((const dictitem_T *)di)->di_tv.v_type; }

/// Get the dictitem's vval.v_number.
int64_t nvim_di_get_nr(const void *di) { return di == NULL ? 0 : (int64_t)((const dictitem_T *)di)->di_tv.vval.v_number; }

/// Get the dictitem's vval.v_string (may be NULL).
const char *nvim_di_get_string(const void *di) { return di == NULL ? NULL : ((const dictitem_T *)di)->di_tv.vval.v_string; }

/// Get a pointer to the dictitem's di_tv (qf-specific void* version).
void *nvim_qf_di_get_tv(void *di) { return di == NULL ? NULL : (void *)&((dictitem_T *)di)->di_tv; }

/// Find a dictitem_T by key in a dict (key_len = -1 for NUL-terminated).
void *nvim_tv_dict_find(const void *dict, const char *key, int key_len)
{
  if (dict == NULL) {
    return NULL;
  }
  return tv_dict_find((const dict_T *)dict, key, (ptrdiff_t)key_len);
}

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

/// Emit e_dictreq error.
void nvim_emsg_dictreq(void) { emsg(_(e_dictreq)); }

/// Get the qfl->qf_ctx as a raw pointer (NULL if not set).
void *nvim_qfl_get_ctx(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_ctx; }

/// tv_dict_add_list: add an existing list to a dict (qf-specific); returns OK or FAIL.
int nvim_qf_tv_dict_add_list(void *dict, const char *key, int key_len, void *list)
{
  return tv_dict_add_list((dict_T *)dict, key, (size_t)key_len, (list_T *)list);
}

/// Allocate a list (qf-specific void* version).
void *nvim_qf_tv_list_alloc(void) { return tv_list_alloc(kListLenMayKnow); }

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

/// Get the qfl->qf_index field.
int nvim_qfl_get_index(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_index; }

/// Get the qfl->qf_count field.
int nvim_qfl_get_count(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_count; }

/// Get the qfl->qf_id field.
unsigned nvim_qfl_get_id(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_id; }

/// Get the qfl->qf_changedtick field.
int nvim_qfl_get_changedtick(const void *qfl_void) { return qfl_void == NULL ? 0 : ((const qf_list_T *)qfl_void)->qf_changedtick; }

/// Get the qfl->qf_title field (may be NULL).
const char *nvim_qfl_get_title(const void *qfl_void) { return qfl_void == NULL ? NULL : ((const qf_list_T *)qfl_void)->qf_title; }

/// qf_alloc_stack wrapper for internal stacks (used by qf_get_list_from_lines).
void *nvim_qf_alloc_internal_stack(void) { return qf_alloc_stack(QFLT_INTERNAL, 1); }

/// qf_free_lists wrapper (free qi->qf_lists array after iterating).
void nvim_qf_free_lists_for_qi(void *qi_void)
{
  if (qi_void == NULL) { return; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  for (int i = 0; i < qi->qf_listcount; i++) {
    rs_qf_free_list(qf_get_list(qi, i));
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

/// Get the got_int flag (qf-specific version to avoid conflict with insexpand_shim).
bool nvim_qf_got_int(void) { return got_int; }

// Phase 8 set-side accessors

/// tv_dict_get_string: get string from dict by key (alloc=true means heap copy, qf void* version).
char *nvim_qf_tv_dict_get_string(const void *dict, const char *key, bool alloc)
{
  return tv_dict_get_string((const dict_T *)dict, key, alloc);
}

/// tv_dict_get_number: get number from dict by key (0 if not found, qf void* version).
int64_t nvim_qf_tv_dict_get_number(const void *dict, const char *key)
{
  return (int64_t)tv_dict_get_number((const dict_T *)dict, key);
}

/// tv_dict_get_tv: copy tv from dict key into *tv_out (VAR_UNKNOWN if not found).
void nvim_tv_dict_get_tv(const void *dict, const char *key, void *tv_out)
{
  tv_dict_get_tv((const dict_T *)dict, key, (typval_T *)tv_out);
}

/// tv_get_number_chk: get number from typval (qf void* version).
int64_t nvim_qf_tv_get_number_chk(const void *tv, bool *denote)
{
  return (int64_t)tv_get_number_chk((const typval_T *)tv, denote);
}

/// tv_get_string_chk: get string from typval (qf void* version, NULL on error).
const char *nvim_qf_tv_get_string_chk(const void *tv)
{
  return tv_get_string_chk((const typval_T *)tv);
}

/// tv_free: free a heap-allocated typval_T (qf void* version).
void nvim_qf_tv_free(void *tv) { tv_free((typval_T *)tv); }

/// Allocate a heap typval_T (zeroed).
void *nvim_tv_alloc(void) { return xcalloc(1, sizeof(typval_T)); }

/// tv_copy from src tv into a newly allocated heap typval_T.
void *nvim_tv_alloc_copy(const void *src_tv)
{
  typval_T *dst = xcalloc(1, sizeof(typval_T));
  tv_copy((const typval_T *)src_tv, dst);
  return dst;
}

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

/// Set qfl->qf_title from a title dict value (tv_dict_get_string for "title" key in what dict).
/// Frees existing title. Returns OK.
int nvim_qfl_set_title_from_what(void *qi_void, int qf_idx, const void *what_void, const void *di_void)
{
  if (qi_void == NULL || what_void == NULL || di_void == NULL) { return FAIL; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  const dictitem_T *di = (const dictitem_T *)di_void;
  if (di->di_tv.v_type != VAR_STRING) { return FAIL; }
  qf_list_T *qfl = qf_get_list(qi, qf_idx);
  xfree(qfl->qf_title);
  qfl->qf_title = tv_dict_get_string((const dict_T *)what_void, "title", true);
  if (qf_idx == qi->qf_curlist) {
    qf_update_win_titlevar(qi);
  }
  return OK;
}

/// Set quickfix list items via rs_qf_add_entries. Returns OK or FAIL.
int nvim_qfl_set_items(void *qi_void, int qf_idx, void *di_void, int action)
{
  if (qi_void == NULL || di_void == NULL) { return FAIL; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  dictitem_T *di = (dictitem_T *)di_void;
  if (di->di_tv.v_type != VAR_LIST) { return FAIL; }
  char *title_save = xstrdup(qi->qf_lists[qf_idx].qf_title);
  int retval = rs_qf_add_entries(qi, qf_idx, di->di_tv.vval.v_list, title_save,
                                  action == ' ' ? 'a' : action);
  xfree(title_save);
  return retval;
}

/// Set quickfix list items from lines via rs_qf_init_ext. Returns OK or FAIL.
int nvim_qfl_set_items_from_lines(void *qi_void, int qf_idx, const void *what_void,
                                   void *di_void, int action)
{
  if (qi_void == NULL || what_void == NULL || di_void == NULL) { return FAIL; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  const dict_T *what = (const dict_T *)what_void;
  dictitem_T *di = (dictitem_T *)di_void;

  const char *errorformat = p_efm;
  const dictitem_T *efm_di = tv_dict_find(what, S_LEN("efm"));
  if (efm_di != NULL) {
    if (efm_di->di_tv.v_type != VAR_STRING || efm_di->di_tv.vval.v_string == NULL) {
      return FAIL;
    }
    errorformat = efm_di->di_tv.vval.v_string;
  }

  if (di->di_tv.v_type != VAR_LIST || di->di_tv.vval.v_list == NULL) {
    return FAIL;
  }

  if (action == 'r' || action == 'u') {
    rs_qf_free_items(&qi->qf_lists[qf_idx]);
  }
  if (rs_qf_init_ext(qi, qf_idx, NULL, NULL, &di->di_tv, errorformat, false, 0, 0, NULL, NULL) >= 0) {
    return OK;
  }
  return FAIL;
}

/// Set the current index in the specified quickfix list via rs_qf_get_nth_entry.
/// Returns OK or FAIL.
int nvim_qfl_set_curidx(void *qi_void, void *qfl_void, void *di_void)
{
  if (qi_void == NULL || qfl_void == NULL || di_void == NULL) { return FAIL; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  qf_list_T *qfl = (qf_list_T *)qfl_void;
  const dictitem_T *di = (const dictitem_T *)di_void;

  int newidx;
  if (di->di_tv.v_type == VAR_STRING
      && di->di_tv.vval.v_string != NULL
      && strcmp(di->di_tv.vval.v_string, "$") == 0) {
    newidx = qfl->qf_count;
  } else {
    bool denote = false;
    newidx = (int)tv_get_number_chk(&di->di_tv, &denote);
    if (denote) { return FAIL; }
  }

  if (newidx < 1) { return FAIL; }
  newidx = MIN(newidx, qfl->qf_count);
  const int old_qfidx = qfl->qf_index;
  qfline_T *const qf_ptr = rs_qf_get_nth_entry(qfl, newidx, &newidx);
  if (qf_ptr == NULL) { return FAIL; }
  qfl->qf_ptr = qf_ptr;
  qfl->qf_index = newidx;

  if (qi->qf_lists[qi->qf_curlist].qf_id == qfl->qf_id) {
    qf_win_pos_update(qi, old_qfidx);
  }
  return OK;
}

/// Set qf_list_changed + update buffer for a qfl.
void nvim_qfl_list_changed_and_update_buf(void *qi_void, void *qfl_void)
{
  if (qi_void == NULL) { return; }
  qf_info_T *qi = (qf_info_T *)qi_void;
  if (qfl_void != NULL) {
    rs_qf_incr_changedtick((qf_list_T *)qfl_void);
  }
  rs_qf_update_buffer(qi, NULL);
}

/// qf_list_changed only (no buffer update).
void nvim_qfl_list_changed(void *qfl_void)
{
  if (qfl_void != NULL) {
    rs_qf_incr_changedtick((qf_list_T *)qfl_void);
  }
}

/// Find a dictitem_T by key in what dict (NUL-terminated key). Returns di_tv ptr or NULL.
void *nvim_tv_dict_find_di_tv(void *dict, const char *key)
{
  if (dict == NULL || key == NULL) { return NULL; }
  dictitem_T *di = tv_dict_find((dict_T *)dict, key, -1);
  return di == NULL ? NULL : (void *)di;
}

/// Check if the action_arg is VAR_STRING and get the string.
const char *nvim_tv_get_string_if_string(const void *tv)
{
  if (tv == NULL || ((const typval_T *)tv)->v_type != VAR_STRING) { return NULL; }
  return tv_get_string_chk((const typval_T *)tv);
}

/// Emit E927 invalid action error.
void nvim_emsg_invact(const char *act)
{
  static const char *e_invact = N_("E927: Invalid action: '%s'");
  semsg(_(e_invact), act);
}

/// Emit e_listreq error.
void nvim_emsg_listreq(void) { emsg(_(e_listreq)); }

/// Emit e_au_recursive error.
void nvim_emsg_au_recursive(void) { emsg(_(e_au_recursive)); }

/// Emit e_string_required error.
void nvim_emsg_string_required(void) { emsg(_(e_string_required)); }

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

static bool qf_win_pos_update(qf_info_T *qi, int old_qf_index);
// qf_update_buffer forward declaration deleted: migrated to Rust rs_qf_update_buffer (Phase 10 Pass 10 Phase 4).

// qf_find_win, qf_find_buf: deleted -- migrated to Rust rs_qf_find_win_for_stack /
// rs_qf_find_buf_for_stack in lib.rs (Phase 10, Pass 10).
// nvim_qf_find_win_for_stack, nvim_qf_find_buf_for_stack: deleted -- callers use
// rs_qf_find_win_for_stack / rs_qf_find_buf_for_stack directly.
bool nvim_qf_win_pos_update(void *qi_void, int old_qf_index) { return qi_void == NULL ? false : qf_win_pos_update((qf_info_T *)qi_void, old_qf_index); }
void nvim_qf_update_buffer(void *qi_void, void *old_last) { rs_qf_update_buffer(qi_void, old_last); }
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

// ---- Phase 4 accessors ----

/// Set wp->w_llist_ref = qi (raw assignment; caller manages refcount separately).
void nvim_win_set_llist_ref(void *wp_void, void *qi_void)
{
  if (wp_void != NULL) {
    ((win_T *)wp_void)->w_llist_ref = (qf_info_T *)qi_void;
  }
}

/// Emit "cannot have both a list and a 'what' argument" error.
void nvim_semsg_list_and_what(void)
{
  semsg(_(e_invarg2), _("cannot have both a list and a \"what\" argument"));
}

/// Call rs_qf_set_properties (Rust implementation).
int nvim_qf_set_properties(void *qi, const void *what, int action, void *title)
{
  return rs_qf_set_properties(qi, what, action, (char *)title);
}

/// Call qf_list_changed(qfl) -> rs_qf_incr_changedtick.
void nvim_qf_list_changed(void *qfl) { if (qfl != NULL) rs_qf_incr_changedtick(qfl); }

// ---- Rust Phase 4 forward declarations ----
extern void rs_qf_free_stack(void *wp, void *qi);
extern int rs_set_errorlist(void *wp, void *list, int action, char *title, void *what);

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

// qf_init_process_nextline deleted: inlined into Rust process_nextline in init.rs (Phase 9).

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
// efm_to_regpat, fmt_start, free_efm_list, parse_efm_option deleted:
// migrated to Rust parse_efm_option / EfmPattern in reader.rs (Phase 9).
static const size_t LINE_MAXLEN = 4096;

// callback function for 'quickfixtextfunc'
static Callback qftf_cb;

// qf_grow_linebuf deleted: migrated to Rust QfParserState::grow_linebuf (Phase 9).

// qf_get_next_str_line, qf_get_next_list_line, qf_get_next_buf_line,
// qf_get_next_file_line, qf_get_nextline deleted: migrated to Rust
// QfParserState methods in reader.rs (Phase 9).

/// Return a pointer to a list in the specified quickfix stack
static qf_list_T *qf_get_list(qf_info_T *qi, int idx)
  FUNC_ATTR_NONNULL_ALL
{
  return &qi->qf_lists[idx];
}

// qf_parse_line (thin wrapper), qf_alloc_fields, qf_free_fields,
// qf_setup_state, qf_cleanup_state deleted: migrated to Rust (Phase 9).
// nvim_qf_init_alloc_fields, nvim_qf_init_free_fields deleted:
// replaced by rs_qf_alloc_fields / rs_qf_free_fields in Rust reader.rs (Phase 9).
// nvim_qf_init_update_efm_cache, s_fmt_first, s_last_efm deleted:
// replaced by rs_qf_init_update_efm_cache + EFM_CACHE in Rust reader.rs (Phase 9).

// nvim_qf_init_setup_state, nvim_qf_init_cleanup_state deleted:
// replaced by rs_qf_parser_state_new / rs_qf_parser_state_free in Rust (Phase 9).

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

// nvim_qf_init_process_nextline, nvim_qf_init_state_no_fd_error deleted:
// inlined into Rust rs_qf_init_ext / process_nextline (Phase 9).

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

// nvim_qf_get_fnum_for_fields deleted: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).

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

// =============================================================================
// Phase 9: Reader state accessors for Rust QfParserState
// =============================================================================

/// Open a file for reading; handles "-" (stdin). Returns NULL on failure.
/// Emits E_OPENERRF on failure. Caller must fclose() when done.
FILE *nvim_qf_open_file_for_read(const char *efile)
{
  if (efile == NULL) {
    return NULL;
  }
  FILE *fd = (strequal(efile, "-")
              ? fdopen(os_open_stdin_fd(), "r")
              : os_fopen(efile, "r"));
  if (fd == NULL) {
    semsg(_(e_openerrf), efile);
  }
  return fd;
}

/// Close a FILE*. Safe to call with NULL.
void nvim_qf_fclose(FILE *fd)
{
  if (fd != NULL) {
    fclose(fd);
  }
}

/// fgets wrapper: reads up to size-1 bytes into buf from fd.
/// Returns true if data was read, false on EOF/error (sets errno on EINTR).
bool nvim_qf_fgets(char *buf, int size, FILE *fd)
{
  return fgets(buf, size, fd) != NULL;
}

/// Return errno value (used for EINTR check after failed fgets).
int nvim_qf_errno(void) { return errno; }

/// Returns true if the string has non-ASCII bytes.
bool nvim_qf_has_non_ascii(const char *buf) { return has_non_ascii(buf); }

/// Remove BOM from the start of buf (modifies in place).
void nvim_qf_remove_bom(char *buf) { remove_bom(buf); }

/// Allocate a zeroed vimconv_T on the heap.
void *nvim_qf_alloc_vimconv(void) { return xcalloc(1, sizeof(vimconv_T)); }

/// Free a heap-allocated vimconv_T (no cleanup; caller must convert_setup first).
void nvim_qf_free_vimconv(void *vc) { xfree(vc); }

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

/// Convert buf with encoding conversion (string_convert).
/// Returns allocated converted string (caller must xfree), or NULL.
/// Sets *lenp to the new length.
char *nvim_qf_string_convert_with_len(void *vc, char *buf, size_t *lenp)
{
  if (vc == NULL || buf == NULL) {
    return NULL;
  }
  return string_convert((vimconv_T *)vc, buf, lenp);
}

/// Return ml_get_buf(buf, lnum) - a pointer to the buffer line (not allocated).
char *nvim_qf_ml_get_buf(void *buf, int32_t lnum) { return ml_get_buf((buf_T *)buf, (linenr_T)lnum); }

/// Return ml_get_buf_len(buf, lnum).
int32_t nvim_qf_ml_get_buf_len(void *buf, int32_t lnum) { return (int32_t)ml_get_buf_len((buf_T *)buf, (linenr_T)lnum); }

/// Return IObuff pointer.
char *nvim_qf_get_iobuff_ptr(void) { return IObuff; }

/// Return IOSIZE constant.
int nvim_qf_get_iosize(void) { return IOSIZE; }

/// xmalloc wrapper for growbuf allocation.
char *nvim_qf_xmalloc_buf(size_t sz) { return xmalloc(sz); }

/// xrealloc wrapper for growbuf grow.
char *nvim_qf_xrealloc_buf(char *ptr, size_t sz) { return xrealloc(ptr, sz); }

/// xfree wrapper for growbuf free.
void nvim_qf_xfree_buf(void *ptr) { xfree(ptr); }

/// xstrlcpy: copy at most n-1 bytes of src to dst, always NUL-terminate.
void nvim_qf_xstrlcpy(char *dst, const char *src, size_t n) { xstrlcpy(dst, src, n); }

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
char *nvim_qf_strchr_nl(char *str)
{
  return vim_strchr(str, '\n');
}

// =============================================================================
// Phase 9 (Phase 2): vim_regcomp/vim_regfree wrappers and efm error messages
// =============================================================================

/// Compile a regex pattern; returns allocated regprog_T* or NULL.
void *nvim_qf_vim_regcomp(const char *pat, int flags)
{
  return vim_regcomp((char *)pat, flags);
}

/// Free a compiled regex (regprog_T*).
void nvim_qf_vim_regfree(void *prog) { vim_regfree(prog); }

/// Wrapper for xstrdup used by Rust's EFM cache.
char *nvim_qf_xstrdup(const char *s) { return s == NULL ? NULL : xstrdup(s); }

/// Error message E372: Too many %%%c in format string
void nvim_qf_semsg_efm_e372(char ch) { semsg(_("E372: Too many %%%c in format string"), ch); }

/// Error message E373: Unexpected %%%c in format string
void nvim_qf_semsg_efm_e373(char ch) { semsg(_("E373: Unexpected %%%c in format string"), ch); }

/// Error message E374: Missing ] in format string
void nvim_qf_emsg_efm_e374(void) { emsg(_("E374: Missing ] in format string")); }

/// Error message E375: Unsupported %%%c in format string
void nvim_qf_semsg_efm_e375(char ch) { semsg(_("E375: Unsupported %%%c in format string"), ch); }

/// Error message E376: Invalid %%%c in format string prefix
void nvim_qf_semsg_efm_e376(char ch) { semsg(_("E376: Invalid %%%c in format string prefix"), ch); }

/// Error message E377: Invalid %%%c in format string
void nvim_qf_semsg_efm_e377(char ch) { semsg(_("E377: Invalid %%%c in format string"), ch); }

/// Error message E378: 'errorformat' contains no pattern
void nvim_qf_emsg_efm_e378(void) { emsg(_("E378: 'errorformat' contains no pattern")); }

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

  rs_qf_update_buffer(qi, NULL);
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


extern void rs_copy_loclist_stack(void *from, void *to);

// Copy the location list stack 'from' window to 'to' window.
void copy_loclist_stack(win_T *from, win_T *to)
  FUNC_ATTR_NONNULL_ALL
{
  rs_copy_loclist_stack((void *)from, (void *)to);
}

// qf_get_fnum deleted: migrated to Rust rs_qf_get_fnum (Phase 10 Pass 10 Phase 5).

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
void nvim_qf_win_goto_lnum(void *win_void, linenr_T lnum) { nvim_qf_win_goto_impl(win_void, lnum); }
linenr_T nvim_qf_win_get_cursor_lnum(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_cursor.lnum; }
linenr_T nvim_qf_win_get_buf_line_count(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_buffer->b_ml.ml_line_count; }
int nvim_qf_win_get_width(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_width; }
int nvim_qf_win_get_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_height; }
int nvim_qf_win_get_hsep_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_hsep_height; }
int nvim_qf_win_get_status_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_status_height; }

int nvim_qf_cmdline_row(void) { return (int)cmdline_row; }


int nvim_qf_open_new_cwindow(void *qi_void, int height) { return rs_qf_open_new_cwindow(qi_void, height); }
void nvim_qf_set_title_var(void *qfl_void) { if (qfl_void != NULL) qf_set_title_var((qf_list_T *)qfl_void); }

void nvim_qf_curwin_set_cursor(linenr_T lnum, int col) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; }

void nvim_qf_check_cursor_curwin(void) { check_cursor(curwin); }

void nvim_qf_update_topline_curwin(void) { update_topline(curwin); }

void nvim_qf_update_win_titlevar(void *qi_void) { if (qi_void != NULL) qf_update_win_titlevar((qf_info_T *)qi_void); }

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
/// Delegates to rs_did_set_quickfixtextfunc (Phase 10 Pass 10 Phase 3).
const char *did_set_quickfixtextfunc(optset_T *args FUNC_ATTR_UNUSED)
{
  return rs_did_set_quickfixtextfunc(args);
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

// qf_update_buffer deleted: migrated to Rust rs_qf_update_buffer (Phase 10 Pass 10 Phase 4).

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
const char *nvim_curbuf_get_b_fname(void) { return curbuf->b_fname; }

// Shell/message helpers
void nvim_append_redir(char *buf, size_t buflen, const char *opt, const char *name) { append_redir(buf, buflen, opt, name); }
void nvim_msg_puts_colon_bang(void) { msg_puts(":!"); }
void nvim_msg_outtrans_cmd(const char *cmd) { msg_outtrans(cmd, 0, false); }

// autowrite, shell, remove
void nvim_autowrite_all(void) { autowrite_all(); }
void nvim_do_shell(const char *cmd) { do_shell(cmd, 0); }

// vim_tempname wrapper
char *nvim_vim_tempname(void) { return vim_tempname(); }

// OS helpers for get_mef_name
int nvim_os_get_pid(void) { return (int)os_get_pid(); }
bool nvim_os_fileinfo_link_exists(const char *name) { FileInfo fi; return os_fileinfo_link(name, &fi); }
void nvim_emsg_notmp(void) { emsg(_(e_notmp)); }

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
const char *nvim_skipdigits_str(const char *str) { return (const char *)skipdigits(str); }
void nvim_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_emsg_buf_not_loaded(void) { emsg(_(e_buffer_is_not_loaded)); }

// eval_expr / tv_free wrappers for ex_cexpr
void *nvim_eval_expr(const void *arg_ptr, void *eap) { return (void *)eval_expr((char *)arg_ptr, (exarg_T *)eap); }
// nvim_tv_get_type: already defined in eval/typval.h (takes const typval_T*)
// nvim_tv_free: already defined in eval_shim.c (takes typval_T*)
// Use void* wrappers with different names to avoid conflicts.
int nvim_tv_get_type_void(const void *tv) { return ((const typval_T *)tv)->v_type; }
const char *nvim_tv_get_vval_string(const void *tv) { return ((const typval_T *)tv)->vval.v_string; }
bool nvim_tv_is_list(const void *tv) { return ((const typval_T *)tv)->v_type == VAR_LIST; }
void nvim_tv_free_void(void *tv) { tv_free((typval_T *)tv); }
void nvim_emsg_e777(void) { emsg(_("E777: String or List expected")); }

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
int nvim_qi_get_listcount_qi(const void *qi) { return ((const qf_info_T *)qi)->qf_listcount; }
void nvim_qi_set_listcount_qi(void *qi, int n) { ((qf_info_T *)qi)->qf_listcount = n; }
int nvim_qi_get_curlist_qi(const void *qi) { return ((const qf_info_T *)qi)->qf_curlist; }
void nvim_qi_set_curlist_qi(void *qi, int n) { ((qf_info_T *)qi)->qf_curlist = n; }
void *nvim_qi_get_list_qi(void *qi, int idx) { return (void *)&((qf_info_T *)qi)->qf_lists[idx]; }
int nvim_qi_get_maxcount_qi(const void *qi) { return ((const qf_info_T *)qi)->qf_maxcount; }
void nvim_qf_free_all_win(void *to_win) { qf_free_all((win_T *)to_win); }

// Extern declarations for Phase 7 Rust entry points
extern void rs_ex_make(void *eap);
extern void rs_ex_cfile(void *eap);
extern void rs_ex_cbuffer(void *eap);
extern void rs_ex_cexpr(void *eap);
// rs_copy_loclist_stack declared earlier (before copy_loclist_stack)

// Form the complete command line to invoke 'make'/'grep'. Quote the command
// using 'shellquote' and append 'shellpipe'. Echo the fully formed command.
// Now a thin wrapper; real implementation in Rust (rs_make_get_fullcmd).
extern char *rs_make_get_fullcmd(const char *makecmd, const char *fname);
static char *make_get_fullcmd(const char *makecmd, const char *fname)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  return rs_make_get_fullcmd(makecmd, fname);
}

// Used for ":make", ":lmake", ":grep", ":lgrep", ":grepadd", and ":lgrepadd"
void ex_make(exarg_T *eap)
{
  rs_ex_make((void *)eap);
}

// Return the name for the errorfile, in allocated memory.
// Find a new unique name when 'makeef' contains "##".
// Returns NULL for error.
// Now a thin wrapper; real implementation in Rust (rs_get_mef_name).
extern char *rs_get_mef_name(void);
static char *get_mef_name(void)
{
  return rs_get_mef_name();
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
  rs_ex_cfile((void *)eap);
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
  rs_qf_update_buffer(qi, NULL);
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

/// Accessor for Rust: get winid for a quickfix info (0 if not found).
extern int rs_qf_winid(void *qi_void);
int nvim_qf_winid(const void *qi_void) { return rs_qf_winid((void *)(uintptr_t)qi_void); }

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

/// Create a quickfix entry from a VimL dict.
/// Extracted from dict fields, then calls rs_qf_add_entry.
static int qf_add_entry_from_dict(qf_list_T *qfl, dict_T *d, bool first_entry,
                                   bool *valid_entry)
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
    valid = (bool)tv_dict_get_number(d, "valid");
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


/// When "what" is not NULL then only set some properties.
int set_errorlist(win_T *wp, list_T *list, int action, char *title, dict_T *what)
{
  return rs_set_errorlist((void *)wp, (void *)list, action, title, (void *)what);
}

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

/// Return a const pointer to a qf_list_T item at index i.
const void *nvim_qf_get_list_at_const(const void *qi_void, int idx)
{
  if (qi_void == NULL) { return NULL; }
  const qf_info_T *qi = (const qf_info_T *)qi_void;
  if (idx < 0 || idx >= qi->qf_maxcount) { return NULL; }
  return (const void *)&qi->qf_lists[idx];
}

// mark_quickfix_user_data deleted: migrated to Rust rs_set_ref_in_quickfix (Phase 10 Pass 10 Phase 6).
// mark_quickfix_ctx deleted: migrated to Rust rs_set_ref_in_quickfix (Phase 10 Pass 10 Phase 6).

/// "in use". So that garbage collection doesn't free the context.
bool set_ref_in_quickfix(int copyID)
{
  return rs_set_ref_in_quickfix(copyID);
}

/// :cgetbuffer, :lbuffer, :laddbuffer, :lgetbuffer Ex commands.
// ":[range]cbuffer [bufnr]" command.
// ":[range]caddbuffer [bufnr]" command.
// ":[range]cgetbuffer [bufnr]" command.
// ":[range]lbuffer [bufnr]" command.
// ":[range]laddbuffer [bufnr]" command.
// ":[range]lgetbuffer [bufnr]" command.
void ex_cbuffer(exarg_T *eap)
{
  rs_ex_cbuffer((void *)eap);
}


/// ":lexpr {expr}", ":lgetexpr {expr}", ":laddexpr {expr}" command.
void ex_cexpr(exarg_T *eap)
{
  rs_ex_cexpr((void *)eap);
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

void f_getloclist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_getloclist(argvars, rettv, NULL); }

void f_getqflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_getqflist(argvars, rettv, NULL); }

void f_setloclist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_setloclist(argvars, rettv, NULL); }

void f_setqflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { rs_f_setqflist(argvars, rettv, NULL); }
