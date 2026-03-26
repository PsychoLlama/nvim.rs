// C accessor functions for the Rust session crate.
//
// This file contains thin C wrapper/accessor functions that Rust calls via FFI
// to access C struct fields, globals, and standard library functions.
// These exist because Rust cannot directly access C struct fields without
// bindgen-generated definitions.
//
// All nvim_ses_* functions are called from: src/nvim-rs/session/src/ffi.rs
// No other C files depend on these functions.

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "klib/kvec.h"
#include "nvim/arglist.h"
#include "nvim/arglist_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/mapping.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "session_shim.c.generated.h"

// Rust FFI declarations
extern var_flavour_T rs_var_flavour(const char *varname);

/// Whether ":lcd" or ":tcd" was produced for a session.
static int did_lcd;

// C accessor functions for Rust FFI

// --- Window accessors ---
bool nvim_ses_win_get_floating(const win_T *wp) { return wp->w_floating; }
buf_T *nvim_ses_win_get_buffer(const win_T *wp) { return wp->w_buffer; }
win_T *nvim_ses_win_get_next(const win_T *wp) { return wp->w_next; }

// --- Buffer query accessors ---
const char *nvim_ses_buf_get_fname(const buf_T *buf) { return buf->b_fname; }
bool nvim_ses_buf_is_terminal(const buf_T *buf) { return buf->terminal != NULL; }
bool nvim_ses_bt_nofilename(const buf_T *buf) { return bt_nofilename(buf); }
bool nvim_ses_bt_help(const buf_T *buf) { return bt_help(buf); }
bool nvim_ses_bt_terminal(const buf_T *buf) { return bt_terminal(buf); }

// --- Session flags accessors ---
unsigned nvim_ses_get_ssop_flags(void) { return ssop_flags; }
unsigned *nvim_ses_get_ssop_flags_ptr(void) { return &ssop_flags; }

// --- Frame accessors ---
int nvim_ses_frame_get_layout(const frame_T *fr) { return fr->fr_layout; }
frame_T *nvim_ses_frame_get_child(const frame_T *fr) { return fr->fr_child; }
frame_T *nvim_ses_frame_get_next(const frame_T *fr) { return fr->fr_next; }
win_T *nvim_ses_frame_get_win(const frame_T *fr) { return fr->fr_win; }

// _Static_assert for frame layout constants
_Static_assert(FR_LEAF == 0, "FR_LEAF must be 0");
_Static_assert(FR_ROW == 1, "FR_ROW must be 1");
_Static_assert(FR_COL == 2, "FR_COL must be 2");

// _Static_assert for session option flags used by Rust
_Static_assert(kOptSsopFlagBlank == 0x80, "kOptSsopFlagBlank");
_Static_assert(kOptSsopFlagHelp == 0x40, "kOptSsopFlagHelp");
_Static_assert(kOptSsopFlagTerminal == 0x10000, "kOptSsopFlagTerminal");

// --- Filename helper accessors ---
const char *nvim_ses_buf_get_sfname(const buf_T *buf) { return buf->b_sfname; }
const char *nvim_ses_buf_get_ffname(const buf_T *buf) { return buf->b_ffname; }
unsigned *nvim_ses_get_vop_flags_ptr(void) { return &vop_flags; }
int nvim_ses_get_p_acd(void) { return p_acd; }
int nvim_ses_get_did_lcd(void) { return did_lcd; }
void nvim_ses_set_did_lcd(int val) { did_lcd = val; }

// Wraps home_replace_save(NULL, name) - returns xmalloc'd string
char *nvim_ses_home_replace_save(const char *name) { return home_replace_save(NULL, name); }
// Wraps vim_strsave_fnameescape(name, VSE_NONE) - returns xmalloc'd string
char *nvim_ses_vim_strsave_fnameescape(const char *name) { return vim_strsave_fnameescape(name, VSE_NONE); }
// Wraps xfree
void nvim_ses_xfree(void *p) { xfree(p); }
// Wraps utfc_ptr2len for MB_PTR_ADV
int nvim_ses_utfc_ptr2len(const char *p) { return utfc_ptr2len(p); }

// Static assertions for session directory flags
_Static_assert(kOptSsopFlagCurdir == 0x1000, "kOptSsopFlagCurdir");
_Static_assert(kOptSsopFlagSesdir == 0x800, "kOptSsopFlagSesdir");

// --- Window struct accessors ---
colnr_T nvim_ses_win_get_curswant(const win_T *wp) { return wp->w_curswant; }
colnr_T nvim_ses_win_get_virtcol(const win_T *wp) { return wp->w_virtcol; }
int nvim_ses_win_get_height(const win_T *wp) { return wp->w_height; }
int nvim_ses_win_get_hsep_height(const win_T *wp) { return wp->w_hsep_height; }
int nvim_ses_win_get_status_height(const win_T *wp) { return wp->w_status_height; }
int nvim_ses_win_get_width(const win_T *wp) { return wp->w_width; }

// --- Global variables ---
frame_T *nvim_ses_get_topframe(void) { return topframe; }
int nvim_ses_topframe_get_height(void) { return topframe->fr_height; }
int nvim_ses_get_Rows(void) { return Rows; }
int nvim_ses_get_Columns(void) { return Columns; }

// --- garray / arglist accessors ---
int nvim_ses_ga_get_len(const garray_T *gap) { return gap->ga_len; }
char *nvim_ses_alist_name_at(garray_T *gap, int i) { return alist_name(&((aentry_T *)gap->ga_data)[i]); }
void *nvim_ses_xmalloc(size_t size) { return xmalloc(size); }
int nvim_ses_vim_FullName(const char *fname, char *buf, size_t len, bool force) { return vim_FullName(fname, buf, len, force); }

// Static assertions for cursor/window size
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL");
_Static_assert(kOptSsopFlagWinsize == 0x08, "kOptSsopFlagWinsize");

// --- store_session_globals complex helper ---
int nvim_ses_foreach_session_global(
    int (*cb)(const char *key, int var_type, const char *escaped_val,
              double float_val, int float_sign, void *ud),
    void *ud)
{
  TV_DICT_ITER(get_globvar_dict(), this_var, {
    if ((this_var->di_tv.v_type == VAR_NUMBER
         || this_var->di_tv.v_type == VAR_STRING)
        && rs_var_flavour(this_var->di_key) == VAR_FLAVOUR_SESSION) {
      // Escape special characters with a backslash. Turn LF/CR into \n and \r.
      char *const p = vim_strsave_escaped(tv_get_string(&this_var->di_tv), "\\\"\n\r");
      for (char *t = p; *t != NUL; t++) {
        if (*t == '\n') {
          *t = 'n';
        } else if (*t == '\r') {
          *t = 'r';
        }
      }
      int is_string = (this_var->di_tv.v_type == VAR_STRING) ? 1 : 0;
      int result = cb(this_var->di_key, is_string, p, 0.0, ' ', ud);
      xfree(p);
      if (result == FAIL) {
        return FAIL;
      }
    } else if (this_var->di_tv.v_type == VAR_FLOAT
               && rs_var_flavour(this_var->di_key) == VAR_FLAVOUR_SESSION) {
      float_T f = this_var->di_tv.vval.v_float;
      int sign = ' ';
      if (f < 0) {
        f = -f;
        sign = '-';
      }
      int result = cb(this_var->di_key, 2, NULL, f, sign, ud);
      if (result == FAIL) {
        return FAIL;
      }
    }
  });
  return OK;
}

// --- get_view_file accessors ---
const char *nvim_ses_get_curbuf_ffname(void) { return curbuf->b_ffname; }
void nvim_ses_emsg_noname(void) { emsg(_(e_noname)); }
const char *nvim_ses_get_p_vdir(void) { return p_vdir; }
bool nvim_ses_vim_ispathsep(int c) { return vim_ispathsep(c); }
bool nvim_ses_add_pathsep(char *p) { return add_pathsep(p); }

// --- put_view accessors ---

// Window accessors for argument list
bool nvim_ses_win_uses_global_alist(const win_T *wp) { return wp->w_alist == &global_alist; }
garray_T *nvim_ses_win_get_alist_ga(win_T *wp) { return &wp->w_alist->al_ga; }
int nvim_ses_win_get_arg_idx(const win_T *wp) { return wp->w_arg_idx; }
bool nvim_ses_win_get_arg_idx_invalid(const win_T *wp) { return wp->w_arg_idx_invalid; }
int nvim_ses_win_wargcount(const win_T *wp) { return WARGCOUNT(wp); }

// Window tag stack
int nvim_ses_win_get_tagstackidx(const win_T *wp) { return wp->w_tagstackidx; }
int nvim_ses_win_get_tagstacklen(const win_T *wp) { return wp->w_tagstacklen; }
const char *nvim_ses_win_get_tagname(const win_T *wp, int idx) { return wp->w_tagstack[idx].tagname; }

// Window alternate file
int nvim_ses_win_get_alt_fnum(const win_T *wp) { return wp->w_alt_fnum; }

// Window cursor/view
int32_t nvim_ses_win_get_cursor_lnum(const win_T *wp) { return wp->w_cursor.lnum; }
int nvim_ses_win_get_cursor_col(const win_T *wp) { return wp->w_cursor.col; }
int32_t nvim_ses_win_get_topline(const win_T *wp) { return wp->w_topline; }
int nvim_ses_win_get_view_height(const win_T *wp) { return wp->w_view_height; }
bool nvim_ses_win_get_p_wrap(const win_T *wp) { return wp->w_p_wrap; }
int nvim_ses_win_get_leftcol(const win_T *wp) { return wp->w_leftcol; }
char *nvim_ses_win_get_localdir(const win_T *wp) { return wp->w_localdir; }

// Buffer query
bool nvim_ses_buf_get_p_bl(const buf_T *buf) { return buf->b_p_bl; }
bool nvim_ses_bt_normal(const buf_T *buf) { return bt_normal(buf); }

// Tabpage accessors
char *nvim_ses_tp_get_localdir(const tabpage_T *tp) { return tp->tp_localdir; }

// Buffer lookup
buf_T *nvim_ses_buflist_findnr(int nr) { return buflist_findnr(nr); }

// Global state manipulation for local options
win_T *nvim_ses_get_curwin(void) { return curwin; }
void nvim_ses_set_curwin(win_T *wp)
{
  curwin = wp;
  curbuf = curwin->w_buffer;
}

// C functions called from put_view that we wrap rather than migrate
int nvim_ses_makemap(FILE *fd, buf_T *buf) { return makemap(fd, buf); }
int nvim_ses_makeset(FILE *fd, int opt, bool local_only) { return makeset(fd, opt, local_only); }
int nvim_ses_makefoldset(FILE *fd) { return makefoldset(fd); }

// Static assertions for session option flags
_Static_assert(kOptSsopFlagCursor == 0x4000, "kOptSsopFlagCursor");
_Static_assert(kOptSsopFlagOptions == 0x20, "kOptSsopFlagOptions");
_Static_assert(kOptSsopFlagLocaloptions == 0x10, "kOptSsopFlagLocaloptions");
_Static_assert(kOptSsopFlagFolds == 0x2000, "kOptSsopFlagFolds");
_Static_assert(OPT_LOCAL == 0x02, "OPT_LOCAL");

// --- makeopens accessors ---

// Buffer iteration callback
int nvim_ses_foreach_buffer(
    int (*cb)(buf_T *buf, bool only_save_windows, void *ud),
    bool only_save_windows,
    void *ud)
{
  FOR_ALL_BUFFERS(buf) {
    int result = cb(buf, only_save_windows, ud);
    if (result == FAIL) {
      return FAIL;
    }
  }
  return OK;
}

// Buffer fields for makeopens
int nvim_ses_buf_get_nwindows(const buf_T *buf) { return buf->b_nwindows; }
bool nvim_ses_buf_is_help(const buf_T *buf) { return buf->b_help; }
int64_t nvim_ses_buf_get_wininfo_lnum(const buf_T *buf) { return kv_size(buf->b_wininfo) == 0 ? 1 : (int64_t)kv_A(buf->b_wininfo, 0)->wi_mark.mark.lnum; }

// Global argument list
garray_T *nvim_ses_get_global_alist_ga(void) { return &global_alist.al_ga; }

// Tabpage iteration and fields
tabpage_T *nvim_ses_get_first_tabpage(void) { return first_tabpage; }
tabpage_T *nvim_ses_get_curtab(void) { return curtab; }
tabpage_T *nvim_ses_tp_get_next(const tabpage_T *tp) { return tp->tp_next; }
win_T *nvim_ses_tp_get_firstwin(const tabpage_T *tp) { return tp->tp_firstwin; }
frame_T *nvim_ses_tp_get_topframe(const tabpage_T *tp) { return tp->tp_topframe; }

// Window globals
win_T *nvim_ses_get_firstwin(void) { return firstwin; }

// Session option globals
const char *nvim_ses_get_globaldir(void) { return globaldir; }
int64_t nvim_ses_get_p_wh(void) { return p_wh; }
int64_t nvim_ses_get_p_wiw(void) { return p_wiw; }
const char *nvim_ses_get_p_shm(void) { return p_shm; }
int64_t nvim_ses_get_p_stal(void) { return p_stal; }

// Static assertions for buffer/tab/resize flags
_Static_assert(kOptSsopFlagBuffers == 0x01, "kOptSsopFlagBuffers");
_Static_assert(kOptSsopFlagGlobals == 0x100, "kOptSsopFlagGlobals");
_Static_assert(kOptSsopFlagTabpages == 0x8000, "kOptSsopFlagTabpages");
_Static_assert(kOptSsopFlagResize == 0x04, "kOptSsopFlagResize");

// --- ex_mkrc, ex_loadview accessors ---

// exarg_T field accessors
int nvim_ses_eap_get_cmdidx(const exarg_T *eap) { return (int)eap->cmdidx; }
char *nvim_ses_eap_get_arg(const exarg_T *eap) { return eap->arg; }
bool nvim_ses_eap_get_forceit(const exarg_T *eap) { return eap->forceit; }
void nvim_ses_eap_set_forceit(exarg_T *eap, bool val) { eap->forceit = val; }

// CMD enum accessors (generated values — no _Static_assert)
int nvim_ses_get_CMD_mksession(void) { return CMD_mksession; }
int nvim_ses_get_CMD_mkview(void) { return CMD_mkview; }
int nvim_ses_get_CMD_mkvimrc(void) { return CMD_mkvimrc; }
int nvim_ses_get_CMD_mkexrc(void) { return CMD_mkexrc; }

// File I/O wrappers
FILE *nvim_ses_open_exfile(char *fname, int forceit, char *mode) { return open_exfile(fname, forceit, mode); }
int nvim_ses_fclose(FILE *fd) { return fclose(fd); }
int nvim_ses_do_source(char *fname) { return do_source(fname, false, DOSO_NONE, NULL); }

// OS wrappers
bool nvim_ses_os_isdir(const char *dir) { return os_isdir(dir); }
int nvim_ses_vim_mkdir_emsg(const char *dir, int perm) { return vim_mkdir_emsg(dir, perm); }
int nvim_ses_os_dirname(char *buf, size_t len) { return os_dirname(buf, len); }
int nvim_ses_os_chdir(const char *dir) { return os_chdir(dir); }
int nvim_ses_vim_chdirfile(char *fname) { return vim_chdirfile(fname, kCdCauseOther); }
void nvim_ses_shorten_fnames(int force) { shorten_fnames(force); }

// Session-related global state
bool nvim_ses_get_p_hls(void) { return p_hls; }
bool nvim_ses_get_no_hlsearch(void) { return no_hlsearch; }
void nvim_ses_set_vim_var_string(const char *val) { set_vim_var_string(VV_THIS_SESSION, val, -1); }
void nvim_ses_apply_autocmds_session(void) { apply_autocmds(EVENT_SESSIONWRITEPOST, NULL, NULL, false, curbuf); }
void nvim_ses_emsg(const char *s) { emsg(s); }
void nvim_ses_semsg(const char *fmt, const char *arg) { semsg(fmt, arg); }
buf_T *nvim_ses_get_curbuf(void) { return curbuf; }

// Error message string accessors
const char *nvim_ses_get_e_prev_dir(void) { return _(e_prev_dir); }
const char *nvim_ses_get_e_write(void) { return _(e_write); }
const char *nvim_ses_get_e_notopen(void) { return _(e_notopen); }

// Filename constants
const char *nvim_ses_get_VIMRC_FILE(void) { return VIMRC_FILE; }
const char *nvim_ses_get_SESSION_FILE(void) { return SESSION_FILE; }
const char *nvim_ses_get_EXRC_FILE(void) { return EXRC_FILE; }

// Option flag values
int nvim_ses_get_OPT_GLOBAL(void) { return OPT_GLOBAL; }
int nvim_ses_get_OPT_SKIPRTP(void) { return OPT_SKIPRTP; }

// Static assertions for source/global flags
_Static_assert(kOptSsopFlagSkiprtp == 0x20000, "kOptSsopFlagSkiprtp");
_Static_assert(DOSO_NONE == 0, "DOSO_NONE");
_Static_assert(OPT_GLOBAL == 0x01, "OPT_GLOBAL");
_Static_assert(OPT_SKIPRTP == 0x80, "OPT_SKIPRTP");

// _Static_assert for OK/FAIL values used by Rust FFI
_Static_assert(OK == 1, "OK must be 1");
_Static_assert(FAIL == 0, "FAIL must be 0");
