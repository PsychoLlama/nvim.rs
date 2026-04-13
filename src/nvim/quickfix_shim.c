#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/ex_eval.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_docmd.h"
#include "nvim/extmark.h"
#include "nvim/fileio.h"
#include "nvim/globals.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/path.h"
#include "nvim/quickfix.h"
#include "nvim/regexp.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/undo.h"
#include "nvim/window.h"
struct dir_stack_T {
  struct dir_stack_T *next;
  char *dirname;
};
typedef struct qfline_S qfline_T;
struct qfline_S {
  qfline_T *qf_next;
  qfline_T *qf_prev;
  linenr_T qf_lnum;
  linenr_T qf_end_lnum;
  int qf_fnum;
  int qf_col;
  int qf_end_col;
  int qf_nr;
  char *qf_module;
  char *qf_fname;
  char *qf_pattern;
  char *qf_text;
  char qf_viscol;
  char qf_cleared;
  char qf_type;
  typval_T qf_user_data;
  char qf_valid;
};
typedef enum {
  QFLT_QUICKFIX,
  QFLT_LOCATION,
  QFLT_INTERNAL,
} qfltype_T;
typedef struct {
  unsigned qf_id;
  qfltype_T qfl_type;
  qfline_T *qf_start;
  qfline_T *qf_last;
  qfline_T *qf_ptr;
  int qf_count;
  int qf_index;
  bool qf_nonevalid;
  bool qf_has_user_data;
  char *qf_title;
  typval_T *qf_ctx;
  Callback qf_qftf_cb;
  struct dir_stack_T *qf_dir_stack;
  char *qf_directory;
  struct dir_stack_T *qf_file_stack;
  char *qf_currfile;
  bool qf_multiline;
  bool qf_multiignore;
  bool qf_multiscan;
  int qf_changedtick;
} qf_list_T;
struct qf_info_S {
  int qf_refcount;  // ref count (2 if loclist win refs it, else 1; freed at 0)
  int qf_listcount;                 // current number of lists
  int qf_curlist;                   // current error list
  int qf_maxcount;                  // maximum number of lists
  qf_list_T *qf_lists;
  qfltype_T qfl_type;  // type of list
  int qf_bufnr;                     // quickfix window buffer number
};
// Layout assertions for Rust ffi_types.rs repr(C) structs.
_Static_assert(sizeof(qf_info_T) == 32, "qf_info_T size changed - update QfInfoRaw in ffi_types.rs");
_Static_assert(offsetof(qf_info_T, qf_refcount) == 0, "qf_info_T.qf_refcount offset");
_Static_assert(offsetof(qf_info_T, qf_listcount) == 4, "qf_info_T.qf_listcount offset");
_Static_assert(offsetof(qf_info_T, qf_curlist) == 8, "qf_info_T.qf_curlist offset");
_Static_assert(offsetof(qf_info_T, qf_maxcount) == 12, "qf_info_T.qf_maxcount offset");
_Static_assert(offsetof(qf_info_T, qf_lists) == 16, "qf_info_T.qf_lists offset");
_Static_assert(offsetof(qf_info_T, qfl_type) == 24, "qf_info_T.qfl_type offset");
_Static_assert(offsetof(qf_info_T, qf_bufnr) == 28, "qf_info_T.qf_bufnr offset");
static qf_info_T ql_info_actual;  // global quickfix list
static qf_info_T *ql_info;        // points to ql_info_actual after allocation
static unsigned last_qf_id = 0;   // Last Used quickfix list id
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
int nvim_qf_get_listcount(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_listcount; }
void *nvim_qf_get_curlist(const void *qi_void) { return (void *)&((const qf_info_T *)qi_void)->qf_lists[((const qf_info_T *)qi_void)->qf_curlist]; }
void *nvim_qf_get_list_at(const void *qi_void, int idx) { return (void *)&((const qf_info_T *)qi_void)->qf_lists[idx]; }
int nvim_qf_get_curlist_idx(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_curlist; }
int nvim_qf_get_maxcount(const void *qi_void) { return ((const qf_info_T *)qi_void)->qf_maxcount; }
void nvim_qf_set_curlist_idx(void *qi_void, int idx) { ((qf_info_T *)qi_void)->qf_curlist = idx; }
void nvim_qf_set_listcount(void *qi_void, int count) { ((qf_info_T *)qi_void)->qf_listcount = count; }
int nvim_qf_get_qi_type(const void *qi_void) { return ((const qf_info_T *)qi_void)->qfl_type; }
void *nvim_get_ql_info(void) { return (void *)ql_info; }
void nvim_set_ql_info(void *qi) { ql_info = (qf_info_T *)qi; }
int nvim_get_p_chi(void) { return (int)p_chi; }
int nvim_qf_buf_get_fnum(const void *buf_void) { return ((const buf_T *)buf_void)->b_fnum; }
void *nvim_buf_win_get_llist(const void *win_void) { return ((const win_T *)win_void)->w_llist; }
#include "quickfix_shim.c.generated.h"
extern int rs_win_valid(win_T *win);
#define IS_QF_WINDOW(wp) (bt_quickfix((wp)->w_buffer) && (wp)->w_llist_ref == NULL)
#define IS_LL_WINDOW(wp) (bt_quickfix((wp)->w_buffer) && (wp)->w_llist_ref != NULL)
#define IS_LL_STACK(qi)       ((qi)->qfl_type == QFLT_LOCATION)
#define GET_LOC_LIST(wp) (IS_LL_WINDOW(wp) ? (wp)->w_llist_ref : (wp)->w_llist)
bool nvim_win_valid(const void *wp_void) { return wp_void == NULL ? false : rs_win_valid((win_T *)wp_void) != 0; }
void *nvim_win_get_loclist(const void *wp_void) { return wp_void == NULL ? NULL : (void *)GET_LOC_LIST((win_T *)wp_void); }
int nvim_qf_win_get_handle(const void *wp_void) { return wp_void == NULL ? 0 : ((const win_T *)wp_void)->handle; }
void nvim_qf_decrement_listcount(void *qi_void) { if (qi_void != NULL && ((qf_info_T *)qi_void)->qf_listcount > 0) ((qf_info_T *)qi_void)->qf_listcount--; }
size_t nvim_qf_sizeof_qfinfo(void) { return sizeof(qf_info_T); }
void nvim_qf_buf_or_has_entry(void *buf_void, bool is_location_list) { if (buf_void != NULL) ((buf_T *)buf_void)->b_has_qf_entry |= is_location_list ? BUF_HAS_LL_ENTRY : BUF_HAS_QF_ENTRY; }
void *nvim_qf_get_list_at_mut(void *qi_void, int idx) { if (qi_void == NULL) { return NULL; } qf_info_T *qi = (qf_info_T *)qi_void; return (idx < 0 || idx >= qi->qf_listcount) ? NULL : &qi->qf_lists[idx]; }
unsigned nvim_qf_alloc_next_id(void) { return ++last_qf_id; }
void nvim_qf_shift_lists_down(void *qi_void) { if (qi_void == NULL) { return; } qf_info_T *qi = (qf_info_T *)qi_void; for (int i = 1; i < qi->qf_listcount; i++) { qi->qf_lists[i - 1] = qi->qf_lists[i]; } }
bool nvim_tv_dict_find_str_is_dollar(const void *dict, const char *key, int key_len) { const dictitem_T *di = tv_dict_find((const dict_T *)dict, key, (ptrdiff_t)key_len); return di != NULL && di->di_tv.v_type == VAR_STRING && di->di_tv.vval.v_string != NULL && strequal(di->di_tv.vval.v_string, "$"); }
int nvim_qf_tv_get_type(const void *tv) { return tv == NULL ? VAR_UNKNOWN : ((const typval_T *)tv)->v_type; }
int nvim_di_get_type(const void *di) { return di == NULL ? VAR_UNKNOWN : ((const dictitem_T *)di)->di_tv.v_type; }
int64_t nvim_di_get_nr(const void *di) { return di == NULL ? 0 : (int64_t)((const dictitem_T *)di)->di_tv.vval.v_number; }
const char *nvim_di_get_string(const void *di) { return di == NULL ? NULL : ((const dictitem_T *)di)->di_tv.vval.v_string; }
void *nvim_qf_di_get_tv(void *di) { return di == NULL ? NULL : (void *)&((dictitem_T *)di)->di_tv; }
void *nvim_tv_advance(const void *tv) { return (void *)((const typval_T *)tv + 1); }
bool nvim_tv_is_unknown(const void *tv) { return tv == NULL || ((const typval_T *)tv)->v_type == VAR_UNKNOWN; }
bool nvim_tv_is_dict(const void *tv) { return tv != NULL && ((const typval_T *)tv)->v_type == VAR_DICT; }
void *nvim_qf_tv_get_dict(const void *tv) { return (tv == NULL || ((const typval_T *)tv)->v_type != VAR_DICT) ? NULL : ((const typval_T *)tv)->vval.v_dict; }
void *nvim_qf_tv_get_list(const void *tv) { return (tv == NULL || ((const typval_T *)tv)->v_type != VAR_LIST) ? NULL : ((const typval_T *)tv)->vval.v_list; }
bool nvim_tv_dict_has_lines_key(const void *dict) { if (dict == NULL) { return false; } const dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("lines")); return di != NULL && di->di_tv.v_type == VAR_LIST && di->di_tv.vval.v_list != NULL; }
void *nvim_tv_dict_get_lines_di_tv(const void *dict) { if (dict == NULL) { return NULL; } dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("lines")); return (di == NULL || di->di_tv.v_type != VAR_LIST) ? NULL : &di->di_tv; }
const char *nvim_tv_dict_get_efm_str(const void *dict) { if (dict == NULL) { return NULL; } const dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("efm")); return (di == NULL || di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL) ? NULL : di->di_tv.vval.v_string; }
bool nvim_tv_dict_efm_wrong_type(const void *dict) { if (dict == NULL) { return false; } const dictitem_T *di = tv_dict_find((const dict_T *)dict, S_LEN("efm")); return di != NULL && (di->di_tv.v_type != VAR_STRING || di->di_tv.vval.v_string == NULL); }
void *nvim_qf_get_list_handle(const void *qi_void, int qf_idx) { if (qi_void == NULL) { return NULL; } const qf_info_T *qi = (const qf_info_T *)qi_void; return (qf_idx < 0 || qf_idx >= qi->qf_listcount) ? NULL : (void *)&qi->qf_lists[qf_idx]; }
void nvim_qf_tv_set_number(void *tv_void, int64_t nr) { if (tv_void != NULL) ((typval_T *)tv_void)->vval.v_number = (varnumber_T)nr; }
int nvim_qf_get_valid_bufnr(const void *qi_void) { if (qi_void == NULL) { return 0; } const qf_info_T *qi = (const qf_info_T *)qi_void; return buflist_findnr(qi->qf_bufnr) != NULL ? qi->qf_bufnr : 0; }
int nvim_qf_get_bufnr(const void *qi_void) { return qi_void == NULL ? -1 : ((const qf_info_T *)qi_void)->qf_bufnr; }
void nvim_qf_set_bufnr(void *qi_void, int bufnr) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_bufnr = bufnr; }
bool nvim_win_is_qf_win(const void *win_void) { if (win_void == NULL) { return false; } const win_T *win = (const win_T *)win_void; return buf_valid(win->w_buffer) && bt_quickfix(win->w_buffer); }
void *nvim_win_get_llist_ref(const void *win_void) { return win_void == NULL ? NULL : ((const win_T *)win_void)->w_llist_ref; }
bool nvim_qf_is_qf_stack(const void *qi_void) { return qi_void == NULL ? false : qi_void == ql_info; }
bool nvim_qf_is_ll_stack(const void *qi_void) { return qi_void == NULL ? false : qi_void != ql_info; }
int nvim_qf_get_refcount(const void *qi_void) { return qi_void == NULL ? 0 : ((const qf_info_T *)qi_void)->qf_refcount; }
void nvim_qf_incr_refcount(void *qi_void) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_refcount++; }
void nvim_qf_set_refcount(void *qi_void, int v) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_refcount = v; }
void nvim_qf_free_lists_array(void *qi_void) { if (qi_void != NULL) { xfree(((qf_info_T *)qi_void)->qf_lists); ((qf_info_T *)qi_void)->qf_lists = NULL; } }
void *nvim_curwin_get_buffer(void) { return (void *)curwin->w_buffer; }
void nvim_curwin_set_buffer(void *buf) { curwin->w_buffer = (buf_T *)buf; }
void nvim_close_buffer_wipe(void *buf_void) { if (buf_void != NULL) close_buffer(NULL, (buf_T *)buf_void, DOBUF_WIPE, false, false); }
void *nvim_get_ql_info_actual(void) { return (void *)&ql_info_actual; }
void nvim_qf_set_qi_type(void *qi_void, int qfltype) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qfl_type = (qfltype_T)qfltype; }
void nvim_qf_set_maxcount(void *qi_void, int n) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_maxcount = n; }
void nvim_qf_set_new_lists(void *qi_void, int n) { if (qi_void != NULL) ((qf_info_T *)qi_void)->qf_lists = xcalloc((size_t)n, sizeof(qf_list_T)); }
void nvim_qf_resize_lists_array(void *qi_void, int n) { if (qi_void == NULL) { return; } qf_info_T *qi = (qf_info_T *)qi_void; size_t lsz = sizeof(*qi->qf_lists); int old_maxcount = qi->qf_maxcount; qf_list_T *new = xrealloc(qi->qf_lists, lsz * (size_t)n); if (n > old_maxcount) { memset(new + old_maxcount, 0, lsz * (size_t)(n - old_maxcount)); } qi->qf_lists = new; qi->qf_maxcount = n; }
int nvim_win_get_p_lhi(const void *wp_void) { return wp_void == NULL ? 0 : (int)((const win_T *)wp_void)->w_p_lhi; }
static char *qf_last_bufname = NULL;
static bufref_T qf_last_bufref = { NULL, 0, 0 };
static Callback qftf_cb;
void nvim_qf_init_emsg_readerrf(void) { emsg(_(e_readerrf)); }
const char *nvim_qf_regmatch_startp(const void *rm, int idx) { return (rm == NULL || idx < 0 || idx >= NSUBEXP) ? NULL : ((const regmatch_T *)rm)->startp[idx]; }
const char *nvim_qf_regmatch_endp(const void *rm, int idx) { return (rm == NULL || idx < 0 || idx >= NSUBEXP) ? NULL : ((const regmatch_T *)rm)->endp[idx]; }
void *nvim_qf_regmatch_create_ic(void *prog) { regmatch_T *rm = xcalloc(1, sizeof(regmatch_T)); rm->rm_ic = true; rm->regprog = (regprog_T *)prog; return rm; }
void *nvim_qf_regmatch_create(void *prog, bool ic) { regmatch_T *rm = xcalloc(1, sizeof(regmatch_T)); rm->rm_ic = ic; rm->regprog = (regprog_T *)prog; return rm; }
void *nvim_qf_regmatch_extract_prog(void *rm_void) { if (rm_void == NULL) { return NULL; } void *prog = ((regmatch_T *)rm_void)->regprog; xfree(rm_void); return prog; }
bool nvim_qf_vim_regexec(void *rm_void, const char *line) { return (rm_void != NULL && line != NULL) && vim_regexec((regmatch_T *)rm_void, line, 0); }
size_t nvim_qf_sizeof_vimconv(void) { return sizeof(vimconv_T); }
void nvim_qf_convert_setup(void *vc, const char *enc) { if (vc != NULL && enc != NULL && *enc != NUL) convert_setup((vimconv_T *)vc, (char *)enc, p_enc); }
void nvim_qf_convert_setup_cleanup(void *vc) { if (vc != NULL) { ((vimconv_T *)vc)->vc_type = CONV_NONE; convert_setup((vimconv_T *)vc, NULL, NULL); } }
int nvim_qf_vc_type(const void *vc) { return vc == NULL ? 0 : ((const vimconv_T *)vc)->vc_type; }
bool nvim_qf_tv_is_string(const void *tv_void) { return tv_void != NULL && ((const typval_T *)tv_void)->v_type == VAR_STRING; }
char *nvim_qf_tv_get_string(void *tv_void) { return tv_void == NULL ? NULL : ((typval_T *)tv_void)->vval.v_string; }
void *nvim_qf_tv_list_first(void *tv_void) { if (tv_void == NULL) { return NULL; } const typval_T *tv = (const typval_T *)tv_void; return (tv->v_type != VAR_LIST || tv->vval.v_list == NULL) ? NULL : tv_list_first(tv->vval.v_list); }
void *nvim_qf_list_item_next(const void *list, const void *li) { return (list == NULL || li == NULL) ? NULL : TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li); }
bool nvim_qf_list_item_is_string(const void *li) { if (li == NULL) { return false; } const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li); return tv->v_type == VAR_STRING && tv->vval.v_string != NULL; }
char *nvim_qf_list_item_string(void *li) { if (li == NULL) { return NULL; } const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li); return (tv->v_type != VAR_STRING || tv->vval.v_string == NULL) ? NULL : tv->vval.v_string; }
void nvim_no_write_message(void) { no_write_message(); }
int nvim_do_ecmd_help(int fnum, int prev_winid) { return do_ecmd(fnum, NULL, NULL, NULL, 1, ECMD_HIDE + ECMD_SET_HELP, prev_winid == curwin->handle ? curwin : NULL); }
bool nvim_curwin_get_wfb(void) { return curwin->w_p_wfb; }
void nvim_qf_set_swb_empty_option(void) { p_swb = empty_string_option; swb_flags = 0; }
bool nvim_qf_prevwin_valid_for_wfb(void) { return rs_win_valid(prevwin) && !prevwin->w_p_wfb && !bt_quickfix(prevwin->w_buffer); }
void *nvim_qf_find_help_win(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (bt_help(wp->w_buffer) && !wp->w_config.hide && wp->w_config.focusable) {
      return wp;
    }
  }
  return NULL;
}
void *nvim_qf_find_win_with_loclist(const void *ll)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_llist == (qf_info_T *)ll && !bt_quickfix(wp->w_buffer)) {
      return wp;
    }
  }
  return NULL;
}
void *nvim_qf_find_win_with_normal_buf(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (bt_normal(wp->w_buffer)) {
      return wp;
    }
  }
  return NULL;
}
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
void *nvim_qf_curwin_get_llist_ref(void) { return curwin->w_llist_ref; }
bool nvim_qf_curwin_buf_is_help(void) { return bt_help(curwin->w_buffer); }
int nvim_qf_get_cmdmod_tab(void) { return cmdmod.cmod_tab; }
bool nvim_qf_is_one_window(void) { return ONE_WINDOW; }
bool nvim_qf_swb_has_usetab(void) { return (swb_flags & kOptSwbFlagUsetab) != 0; }
int nvim_qf_curwin_handle(void) { return curwin->handle; }
int nvim_qf_win_buf_nwindows(const void *win) { return ((const win_T *)win)->w_buffer->b_nwindows; }
int nvim_qf_win_buf_fnum(const void *win) { return ((const win_T *)win)->w_buffer->b_fnum; }
void *nvim_qf_win_get_llist(const void *win) { return ((const win_T *)win)->w_llist; }
void nvim_qf_win_set_loclist(void *win, void *qi) { ((win_T *)win)->w_llist = (qf_info_T *)qi; ((qf_info_T *)qi)->qf_refcount++; }
int nvim_qf_get_cmdmod_split(void) { return cmdmod.cmod_split; }
int nvim_qf_curwin_width(void) { return curwin->w_width; }
int nvim_qf_curwin_height(void) { return curwin->w_height; }
bool nvim_qf_is_ll_stack_qi(const void *qi) { return IS_LL_STACK((const qf_info_T *)qi); }
bool nvim_qf_win_is_qf_window(const void *win) { return IS_QF_WINDOW((const win_T *)win); }
void *nvim_qf_win_prev(const void *win) { return ((const win_T *)win)->w_prev; }
void *nvim_qf_win_next(const void *win) { return ((const win_T *)win)->w_next; }
bool nvim_qf_win_bt_normal(const void *win) { return bt_normal(((const win_T *)win)->w_buffer); }
bool nvim_qf_swb_uselast_prevwin_ok(void) { return (swb_flags & kOptSwbFlagUselast) && rs_win_valid(prevwin) && !prevwin->w_p_wfb; }
bool nvim_qf_win_is_preview(const void *win) { return ((const win_T *)win)->w_p_pvw; }
bool nvim_qf_win_is_wfb(const void *win) { return ((const win_T *)win)->w_p_wfb; }
linenr_T nvim_qf_curbuf_line_count(void) { return curbuf->b_ml.ml_line_count; }
void nvim_qf_curwin_set_col(int col) { curwin->w_cursor.col = col; }
void nvim_qf_curwin_set_coladd_zero(void) { curwin->w_cursor.coladd = 0; }
void nvim_qf_curwin_set_curswant(void) { curwin->w_set_curswant = true; }
void nvim_qf_coladvance(int col) { coladvance(curwin, col); }
void nvim_qf_beginline_white_fix(void) { beginline(BL_WHITE | BL_FIX); }
bool nvim_qf_do_search_pattern(const char *pat) { pos_T save_cursor = curwin->w_cursor; curwin->w_cursor.lnum = 0; if (!do_search(NULL, '/', '/', (char *)pat, strlen(pat), 1, SEARCH_KEEP, NULL)) { curwin->w_cursor = save_cursor; return false; } return true; }
const char *nvim_qf_gettext_line_deleted(void) { return _(" (line deleted)"); }
bool nvim_qf_fdo_quickfix(void) { return (fdo_flags & kOptFdoFlagQuickfix) != 0; }
void *nvim_qf_get_p_swb(void) { return p_swb; }
unsigned nvim_qf_get_swb_flags(void) { return swb_flags; }
void nvim_qf_restore_swb(void *old_swb, unsigned old_swb_flags) { if (p_swb != (char *)old_swb && p_swb == empty_string_option) { p_swb = (char *)old_swb; swb_flags = old_swb_flags; } }
linenr_T nvim_qf_win_get_cursor_lnum(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_cursor.lnum; }
linenr_T nvim_qf_win_get_buf_line_count(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_buffer->b_ml.ml_line_count; }
int nvim_qf_win_get_width(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_width; }
int nvim_qf_win_get_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_height; }
int nvim_qf_win_get_hsep_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_hsep_height; }
int nvim_qf_win_get_status_height(const void *win_void) { return win_void == NULL ? 0 : ((const win_T *)win_void)->w_status_height; }
void nvim_qf_curwin_set_cursor(linenr_T lnum, int col) { curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = col; }
void nvim_qf_win_set_redraw_bounds(void *win_void, linenr_T top, linenr_T bot) { if (win_void != NULL) { ((win_T *)win_void)->w_redraw_top = top; ((win_T *)win_void)->w_redraw_bot = bot; } }
void nvim_qf_win_goto_impl(void *win_void, linenr_T lnum) { if (win_void == NULL) { return; } win_T *win = (win_T *)win_void; win_T *old_curwin = curwin; curwin = win; curbuf = win->w_buffer; curwin->w_cursor.lnum = lnum; curwin->w_cursor.col = 0; curwin->w_cursor.coladd = 0; curwin->w_curswant = 0; update_topline(curwin); redraw_later(curwin, UPD_VALID); curwin->w_redr_status = true; curwin = old_curwin; curbuf = curwin->w_buffer; }
void nvim_qf_set_title_var_for_list(void *qfl_void) { if (qfl_void != NULL && ((qf_list_T *)qfl_void)->qf_title != NULL) set_internal_string_var("w:quickfix_title", ((qf_list_T *)qfl_void)->qf_title); }
void nvim_qf_set_cwindow_options(void) { set_option_value_give_err(kOptSwapfile, BOOLEAN_OPTVAL(false), OPT_LOCAL); set_option_value_give_err(kOptBuftype, STATIC_CSTR_AS_OPTVAL("quickfix"), OPT_LOCAL); set_option_value_give_err(kOptBufhidden, STATIC_CSTR_AS_OPTVAL("hide"), OPT_LOCAL); RESET_BINDING(curwin); curwin->w_p_diff = false; set_option_value_give_err(kOptFoldmethod, STATIC_CSTR_AS_OPTVAL("manual"), OPT_LOCAL); }
void nvim_qf_curwin_set_llist_ref_incr(void *qi_void) { if (qi_void != NULL) { curwin->w_llist_ref = (qf_info_T *)qi_void; ((qf_info_T *)qi_void)->qf_refcount++; } }
void nvim_qf_curwin_set_wfh(void) { curwin->w_p_wfh = true; }
void nvim_qf_curwin_reset_binding(void) { RESET_BINDING(curwin); }
int nvim_qf_option_set_callback_func_for_qftf(void) { return option_set_callback_func(p_qftf, &qftf_cb); }
void nvim_qf_extmark_splice(void *buf, int r1, colnr_T c1, int r2, colnr_T c2, bcount_t bc, int nr, colnr_T nc, bcount_t nbc) { extmark_splice((buf_T *)buf, r1, c1, r2, c2, bc, nr, nc, nbc, kExtmarkNoUndo); }
void nvim_qf_buf_set_changed_false(void *buf) { ((buf_T *)buf)->b_changed = false; }
linenr_T nvim_qf_win_botline(const void *win) { return ((const win_T *)win)->w_botline; }
void *nvim_qf_aucmd_prepbuf_alloc(void *buf) { aco_save_T *aco = xmalloc(sizeof(aco_save_T)); aucmd_prepbuf(aco, (buf_T *)buf); return aco; }
void nvim_qf_aucmd_restbuf_free(void *aco_void) { if (aco_void != NULL) { aucmd_restbuf((aco_save_T *)aco_void); xfree(aco_void); } }
void *nvim_qf_fnum_cache_check(const char *bufname) { if (bufname == NULL) { return NULL; } return (qf_last_bufname != NULL && strcmp(bufname, qf_last_bufname) == 0 && bufref_valid(&qf_last_bufref)) ? qf_last_bufref.br_buf : NULL; }
void nvim_qf_fnum_cache_update(const char *bufname, void *buf) { xfree(qf_last_bufname); qf_last_bufname = xstrdup(bufname); set_bufref(&qf_last_bufref, (buf_T *)buf); }
int nvim_qf_buf_fnum_from_ptr(const void *buf_void) { return buf_void == NULL ? 0 : ((const buf_T *)buf_void)->b_fnum; }
void nvim_qf_buf_set_has_qf_entry(void *buf_void, bool is_qf_list) { if (buf_void != NULL) ((buf_T *)buf_void)->b_has_qf_entry = is_qf_list ? BUF_HAS_QF_ENTRY : BUF_HAS_LL_ENTRY; }
bool nvim_qf_vim_is_abs_name(const char *fname) { return fname != NULL && vim_isAbsName(fname); }
char *nvim_qf_concat_fnames(const char *dir, const char *fname) { return concat_fnames((char *)dir, (char *)fname, true); }
void nvim_qf_clear_fnum_cache(void) { XFREE_CLEAR(qf_last_bufname); }
void nvim_tv_dict_incr_refcount(void *dict) { if (dict != NULL) ((dict_T *)dict)->dv_refcount++; }
bool nvim_callback_is_none(const void *cb) { return cb == NULL || ((const Callback *)cb)->type == kCallbackNone; }
bool nvim_callback_call_one_dict(void *cb, void *dict, void *rettv) { if (cb == NULL || dict == NULL || rettv == NULL) { return false; } typval_T args[1] = { { .v_type = VAR_DICT, .vval = { .v_dict = (dict_T *)dict } } }; return callback_call((Callback *)cb, 1, args, (typval_T *)rettv); }
void *nvim_tv_rettv_list_if_var_list(const void *rettv_void) { if (rettv_void == NULL) { return NULL; } const typval_T *rettv = (const typval_T *)rettv_void; return rettv->v_type == VAR_LIST ? (void *)rettv->vval.v_list : NULL; }
bool nvim_qf_buf_is_curbuf(const void *buf) { return (const buf_T *)buf == curbuf; }
bool nvim_qf_delete_all_lines(void) { while ((curbuf->b_ml.ml_flags & ML_EMPTY) == 0) { if (ml_delete(1) == FAIL) { internal_error("rs_qf_fill_buffer()"); return false; } } return true; }
void nvim_qf_zero_skipcol_for_curbuf(void)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    if (wp->w_buffer == curbuf) {
      wp->w_skipcol = 0;
    }
  }
}
void nvim_qf_u_clearallandblockfree(void) { u_clearallandblockfree(curbuf); }
char *nvim_tv_list_item_string(const void *li) { return li == NULL ? NULL : (char *)tv_get_string_chk(TV_LIST_ITEM_TV((const listitem_T *)li)); }
const char *nvim_qf_buf_get_fname(const void *buf) { return ((const buf_T *)buf)->b_fname; }
void nvim_qf_curbuf_incr_ro_locked(void) { curbuf->b_ro_locked++; }
void nvim_qf_curbuf_decr_ro_locked(void) { curbuf->b_ro_locked--; }
void nvim_qf_curbuf_set_ma_false(void) { curbuf->b_p_ma = false; }
void nvim_qf_curbuf_set_keep_filetype(bool val) { curbuf->b_keep_filetype = val; }
void nvim_qf_set_option_filetype_qf(void) { set_option_value_give_err(kOptFiletype, STATIC_CSTR_AS_OPTVAL("qf"), OPT_LOCAL); }
void nvim_qf_redraw_curbuf_later(void) { redraw_curbuf_later(UPD_NOT_VALID); }
bool nvim_qf_get_key_typed(void) { return KeyTyped; }
void nvim_qf_set_key_typed(bool val) { KeyTyped = val; }
const char *nvim_curbuf_get_b_p_menc(void) { return curbuf->b_p_menc; }
const char *nvim_curbuf_get_b_p_gefm(void) { return curbuf->b_p_gefm; }
bool nvim_os_fileinfo_link_exists(const char *name) { FileInfo fi; return os_fileinfo_link(name, &fi); }
void nvim_win_set_p_lhi(void *win, int v) { ((win_T *)win)->w_p_lhi = (OptInt)v; }
char *nvim_eap_get_cmdlinep_deref_make(const void *eap) { return *((const exarg_T *)eap)->cmdlinep; }
void nvim_set_option_direct_ef(const char *val) { set_option_direct(kOptErrorfile, CSTR_AS_OPTVAL(val), 0, 0); }
bool nvim_buf_has_ml_mfp_void(const void *buf) { return ((const buf_T *)buf)->b_ml.ml_mfp != NULL; }
void *nvim_buflist_findnr_ptr(int nr) { return (void *)buflist_findnr(nr); }
void *nvim_eval_expr(const void *arg_ptr, void *eap) { return (void *)eval_expr((char *)arg_ptr, (exarg_T *)eap); }
int nvim_tv_get_type_void(const void *tv) { return ((const typval_T *)tv)->v_type; }
const char *nvim_tv_get_vval_string(const void *tv) { return ((const typval_T *)tv)->vval.v_string; }
bool nvim_tv_is_list(const void *tv) { return ((const typval_T *)tv)->v_type == VAR_LIST; }
void nvim_tv_free_void(void *tv) { tv_free((typval_T *)tv); }
void nvim_qf_snprintf_iobuff(const char *title, const char *sfname) { vim_snprintf(IObuff, IOSIZE, "%s (%s)", title, sfname); }
void *nvim_win_get_llist_or_ref(const void *from_win) { const win_T *from = (const win_T *)from_win; return (void *)(IS_LL_WINDOW(from) ? from->w_llist_ref : from->w_llist); }
void nvim_qf_free_all_win(void *to_win) { qf_free_all((win_T *)to_win); }
bool nvim_qf_curwin_is_ll(void) { return IS_LL_WINDOW(curwin); }
bool nvim_qf_is_ll_window(const void *wp_void) { return wp_void != NULL && IS_LL_WINDOW((const win_T *)wp_void); }
void *nvim_qf_curwin_get_loclist(void) { return GET_LOC_LIST(curwin); }
linenr_T nvim_qf_get_cursor_lnum(void) { return curwin->w_cursor.lnum; }
bool nvim_qf_curbuf_has_flag(int flag) { return (curbuf->b_has_qf_entry & flag) != 0; }
int nvim_qf_curbuf_fnum(void) { return curbuf->b_fnum; }
bool nvim_grep_uses_internal(void) { return strcmp("internal", *curbuf->b_p_gp == NUL ? p_gp : curbuf->b_p_gp) == 0; }
const void *nvim_qf_curwin_pos_adj(void) { static pos_T pos; pos = curwin->w_cursor; pos.col++; return &pos; }
void *nvim_qf_get_curlist_mut(void *qi_void) { return (void *)&((qf_info_T *)qi_void)->qf_lists[((qf_info_T *)qi_void)->qf_curlist]; }
const char *nvim_buf_get_mfp_fname(const void *buf) { const buf_T *b = (const buf_T *)buf; return b->b_ml.ml_mfp != NULL ? b->b_ml.ml_mfp->mf_fname : NULL; }
bool nvim_cmdmod_has_cmod_hide(void) { return (cmdmod.cmod_flags & CMOD_HIDE) != 0; }
void nvim_buf_clear_bf_dummy(void *buf) { ((buf_T *)buf)->b_flags &= ~BF_DUMMY; }
void *nvim_buflist_new(char *ffname, char *sfname, int lnum, int flags) { return buflist_new(ffname, sfname, (linenr_T)lnum, flags); }
void nvim_setfname_curbuf(char *fname) { setfname(curbuf, fname, NULL, false); }
void nvim_check_need_swap_newfile(void) { check_need_swap(true); }
int nvim_readfile_for_dummy(char *fname) { return readfile(fname, NULL, 0, 0, (linenr_T)MAXLNUM, NULL, READ_NEW | READ_DUMMY, false); }
void nvim_buf_inc_locked(void *buf) { ((buf_T *)buf)->b_locked++; }
void nvim_buf_dec_locked(void *buf) { ((buf_T *)buf)->b_locked--; }
int nvim_close_buffer_unload(void *buf) { return close_buffer(NULL, (buf_T *)buf, DOBUF_UNLOAD, false, true) ? 1 : 0; }
void *nvim_cleanup_enter_alloc(void) { cleanup_T *cs = xmalloc(sizeof(cleanup_T)); enter_cleanup(cs); return cs; }
void nvim_cleanup_leave_free(void *cs) { leave_cleanup((cleanup_T *)cs); xfree(cs); }
void *nvim_qf_bufref_alloc(void *buf) { bufref_T *br = xmalloc(sizeof(bufref_T)); set_bufref(br, (buf_T *)buf); return br; }
bool nvim_qf_bufref_is_valid(void *br) { return bufref_valid((bufref_T *)br); }
void *nvim_qf_bufref_get_buf(void *br) { return ((bufref_T *)br)->br_buf; }
void nvim_qf_bufref_set_buf_null(void *br) { ((bufref_T *)br)->br_buf = NULL; }
void nvim_qf_bufref_free(void *br) { xfree(br); }
void nvim_apply_filetype_autocmds_and_modelines(void *buf_void) { buf_T *buf = (buf_T *)buf_void; aco_save_T aco; aucmd_prepbuf(&aco, buf); apply_autocmds(EVENT_FILETYPE, buf->b_p_ft, buf->b_fname, true, buf); do_modelines(OPT_NOWIN); aucmd_restbuf(&aco); }
void nvim_ex_cd_arg(char *arg, bool is_lcd) { exarg_T ea = { .arg = arg, .cmdidx = is_lcd ? CMD_lcd : CMD_cd }; ex_cd(&ea); }
linenr_T nvim_regmatch_startpos_lnum(const regmmatch_T *rm, int idx) { return rm->startpos[idx].lnum; }
colnr_T nvim_regmatch_startpos_col(const regmmatch_T *rm, int idx) { return rm->startpos[idx].col; }
linenr_T nvim_regmatch_endpos_lnum(const regmmatch_T *rm, int idx) { return rm->endpos[idx].lnum; }
colnr_T nvim_regmatch_endpos_col(const regmmatch_T *rm, int idx) { return rm->endpos[idx].col; }
colnr_T nvim_ml_get_buf_len(void *buf, linenr_T lnum) { return ml_get_buf_len((buf_T *)buf, lnum); }
void *nvim_vgr_regcomp_init(const char *pat) { regmmatch_T *rm = xcalloc(1, sizeof(regmmatch_T)); if (pat == NULL || *pat == NUL) { if (last_search_pat() == NULL) { emsg(_(e_noprevre)); xfree(rm); return NULL; } rm->regprog = vim_regcomp(last_search_pat(), RE_MAGIC); } else { rm->regprog = vim_regcomp((char *)pat, RE_MAGIC); } if (rm->regprog == NULL) { xfree(rm); return NULL; } rm->rmm_ic = p_ic; rm->rmm_maxcol = 0; return rm; }
void nvim_vgr_regmatch_free(void *rm_void) { if (rm_void == NULL) { return; } regmmatch_T *rm = (regmmatch_T *)rm_void; vim_regfree(rm->regprog); xfree(rm); }
void *nvim_tv_list_first(const void *list) { return list == NULL ? NULL : tv_list_first((const list_T *)list); }
void *nvim_tv_list_item_next(const void *list, const void *li) { return (list == NULL || li == NULL) ? NULL : TV_LIST_ITEM_NEXT((const list_T *)list, (const listitem_T *)li); }
void *nvim_tv_list_item_dict(const void *li) { if (li == NULL) { return NULL; } const typval_T *tv = TV_LIST_ITEM_TV((const listitem_T *)li); return tv->v_type != VAR_DICT ? NULL : tv->vval.v_dict; }
void nvim_xfree_char(char *ptr) { xfree(ptr); }
bool nvim_tv_list_item_is_first(const void *list, const void *li) { return (list != NULL && li != NULL) && li == tv_list_first((const list_T *)list); }
void *nvim_qf_get_global_qftf_cb_ptr(void) { return (void *)&qftf_cb; }
void *nvim_qf_win_get_llist_mut(void *win_void) { return win_void == NULL ? NULL : ((win_T *)win_void)->w_llist; }
void *nvim_qf_win_get_llist_ref_mut(void *win_void) { return win_void == NULL ? NULL : ((win_T *)win_void)->w_llist_ref; }
bool nvim_qf_win_is_ll_and_refcount_one(const void *win_void) { if (win_void == NULL) { return false; } const win_T *win = (const win_T *)win_void; return IS_LL_WINDOW(win) && win->w_llist_ref->qf_refcount == 1; }
void *nvim_save_cpo_set_empty(void) { char *save_cpo = p_cpo; p_cpo = empty_string_option; return save_cpo; }
void nvim_restore_cpo(void *saved_cpo_void) { char *save_cpo = (char *)saved_cpo_void; if (p_cpo == empty_string_option) { p_cpo = save_cpo; } else { if (*p_cpo == NUL) { set_option_value_give_err(kOptCpoptions, CSTR_AS_OPTVAL(save_cpo), 0); } free_string_option(save_cpo); } }
