// Functions for creating a session file, i.e. implementing:
//   :mkexrc
//   :mkvimrc
//   :mkview
//   :mksession

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/arglist.h"
#include "nvim/arglist_defs.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
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
#include "nvim/ex_session.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
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
#include "nvim/pos_defs.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "ex_session.c.generated.h"

// Rust FFI declarations (window wrappers removed)
extern int rs_tabpage_index(tabpage_T *ftp);
extern var_flavour_T rs_var_flavour(const char *varname);

/// Whether ":lcd" or ":tcd" was produced for a session.
static int did_lcd;

#define PUTLINE_FAIL(s) \
  do { if (FAIL == put_line(fd, (s))) { return FAIL; } } while (0)

// =========================================================================
// C accessor functions for Rust FFI
// =========================================================================

// --- Window accessors (Phase 2) ---
bool nvim_ses_win_get_floating(const win_T *wp) { return wp->w_floating; }
buf_T *nvim_ses_win_get_buffer(const win_T *wp) { return wp->w_buffer; }
win_T *nvim_ses_win_get_next(const win_T *wp) { return wp->w_next; }

// --- Buffer query accessors (Phase 2) ---
const char *nvim_ses_buf_get_fname(const buf_T *buf) { return buf->b_fname; }
bool nvim_ses_buf_is_terminal(const buf_T *buf) { return buf->terminal != NULL; }
bool nvim_ses_bt_nofilename(const buf_T *buf) { return bt_nofilename(buf); }
bool nvim_ses_bt_help(const buf_T *buf) { return bt_help(buf); }
bool nvim_ses_bt_terminal(const buf_T *buf) { return bt_terminal(buf); }

// --- Session flags accessors (Phase 2) ---
unsigned nvim_ses_get_ssop_flags(void) { return ssop_flags; }
unsigned *nvim_ses_get_ssop_flags_ptr(void) { return &ssop_flags; }

// --- Frame accessors (Phase 2) ---
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

// --- Filename helper accessors (Phase 3) ---
const char *nvim_ses_buf_get_sfname(const buf_T *buf) { return buf->b_sfname; }
const char *nvim_ses_buf_get_ffname(const buf_T *buf) { return buf->b_ffname; }
unsigned *nvim_ses_get_vop_flags_ptr(void) { return &vop_flags; }
int nvim_ses_get_p_acd(void) { return p_acd; }
int nvim_ses_get_did_lcd(void) { return did_lcd; }
void nvim_ses_set_did_lcd(int val) { did_lcd = val; }

// Wraps home_replace_save(NULL, name) - returns xmalloc'd string
char *nvim_ses_home_replace_save(const char *name)
{
  return home_replace_save(NULL, name);
}

// Wraps vim_strsave_fnameescape(name, VSE_NONE) - returns xmalloc'd string
char *nvim_ses_vim_strsave_fnameescape(const char *name)
{
  return vim_strsave_fnameescape(name, VSE_NONE);
}

// Wraps xfree
void nvim_ses_xfree(void *p)
{
  xfree(p);
}

// Wraps utfc_ptr2len for MB_PTR_ADV
int nvim_ses_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}

// _Static_assert for Phase 3 constants
_Static_assert(kOptSsopFlagCurdir == 0x1000, "kOptSsopFlagCurdir");
_Static_assert(kOptSsopFlagSesdir == 0x800, "kOptSsopFlagSesdir");

// --- Window struct accessors (Phase 4) ---
colnr_T nvim_ses_win_get_curswant(const win_T *wp) { return wp->w_curswant; }
colnr_T nvim_ses_win_get_virtcol(const win_T *wp) { return wp->w_virtcol; }
int nvim_ses_win_get_height(const win_T *wp) { return wp->w_height; }
int nvim_ses_win_get_hsep_height(const win_T *wp) { return wp->w_hsep_height; }
int nvim_ses_win_get_status_height(const win_T *wp) { return wp->w_status_height; }
int nvim_ses_win_get_width(const win_T *wp) { return wp->w_width; }

// --- Global variables (Phase 4) ---
frame_T *nvim_ses_get_topframe(void) { return topframe; }
int nvim_ses_topframe_get_height(void) { return topframe->fr_height; }
int nvim_ses_get_Rows(void) { return Rows; }
int nvim_ses_get_Columns(void) { return Columns; }

// --- garray / arglist accessors (Phase 4) ---
int nvim_ses_ga_get_len(const garray_T *gap) { return gap->ga_len; }
char *nvim_ses_alist_name_at(garray_T *gap, int i)
{
  return alist_name(&((aentry_T *)gap->ga_data)[i]);
}
void *nvim_ses_xmalloc(size_t size) { return xmalloc(size); }
int nvim_ses_vim_FullName(const char *fname, char *buf, size_t len, bool force)
{
  return vim_FullName(fname, buf, len, force);
}

// _Static_assert for Phase 4 constants
_Static_assert(MAXCOL == 0x7fffffff, "MAXCOL");
_Static_assert(kOptSsopFlagWinsize == 0x08, "kOptSsopFlagWinsize");

// --- Phase 5 accessors: store_session_globals ---
// Type constants for session global variable callback
// 0 = VAR_NUMBER or VAR_STRING, 1 = VAR_FLOAT
//
// Callback signature: int (*cb)(const char *key, int var_type, const char *escaped_val,
//                               double float_val, int float_sign, void *fd)
// Returns FAIL if callback requests stop (write error).
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

// --- Phase 5 accessors: get_view_file ---
const char *nvim_ses_get_curbuf_ffname(void) { return curbuf->b_ffname; }
void nvim_ses_emsg_noname(void) { emsg(_(e_noname)); }
const char *nvim_ses_get_p_vdir(void) { return p_vdir; }
bool nvim_ses_vim_ispathsep(int c) { return vim_ispathsep(c); }
bool nvim_ses_add_pathsep(char *p) { return add_pathsep(p); }

// --- Phase 6 accessors: put_view ---

// Window accessors for argument list
bool nvim_ses_win_uses_global_alist(const win_T *wp) { return wp->w_alist == &global_alist; }
garray_T *nvim_ses_win_get_alist_ga(win_T *wp) { return &wp->w_alist->al_ga; }
int nvim_ses_win_get_arg_idx(const win_T *wp) { return wp->w_arg_idx; }
bool nvim_ses_win_get_arg_idx_invalid(const win_T *wp) { return wp->w_arg_idx_invalid; }
int nvim_ses_win_wargcount(const win_T *wp) { return WARGCOUNT(wp); }

// Window tag stack
int nvim_ses_win_get_tagstackidx(const win_T *wp) { return wp->w_tagstackidx; }
int nvim_ses_win_get_tagstacklen(const win_T *wp) { return wp->w_tagstacklen; }
const char *nvim_ses_win_get_tagname(const win_T *wp, int idx)
{
  return wp->w_tagstack[idx].tagname;
}

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
int nvim_ses_makeset(FILE *fd, int opt, bool local_only)
{
  return makeset(fd, opt, local_only);
}
int nvim_ses_makefoldset(FILE *fd) { return makefoldset(fd); }
int nvim_ses_put_folds(FILE *fd, win_T *wp) { return rs_put_folds(fd, wp); }

// _Static_assert for Phase 6 constants
_Static_assert(kOptSsopFlagCursor == 0x4000, "kOptSsopFlagCursor");
_Static_assert(kOptSsopFlagOptions == 0x20, "kOptSsopFlagOptions");
_Static_assert(kOptSsopFlagLocaloptions == 0x10, "kOptSsopFlagLocaloptions");
_Static_assert(kOptSsopFlagFolds == 0x2000, "kOptSsopFlagFolds");
_Static_assert(OPT_LOCAL == 0x02, "OPT_LOCAL");

// --- Phase 7 accessors: makeopens ---

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
int64_t nvim_ses_buf_get_wininfo_lnum(const buf_T *buf)
{
  return kv_size(buf->b_wininfo) == 0 ? 1 : (int64_t)kv_A(buf->b_wininfo, 0)->wi_mark.mark.lnum;
}

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
int nvim_ses_tabpage_index(tabpage_T *tp) { return rs_tabpage_index(tp); }

// Session option globals
const char *nvim_ses_get_globaldir(void) { return globaldir; }
int64_t nvim_ses_get_p_wh(void) { return p_wh; }
int64_t nvim_ses_get_p_wiw(void) { return p_wiw; }
const char *nvim_ses_get_p_shm(void) { return p_shm; }
int64_t nvim_ses_get_p_stal(void) { return p_stal; }

// _Static_assert for Phase 7 constants
_Static_assert(kOptSsopFlagBuffers == 0x01, "kOptSsopFlagBuffers");
_Static_assert(kOptSsopFlagGlobals == 0x100, "kOptSsopFlagGlobals");
_Static_assert(kOptSsopFlagTabpages == 0x8000, "kOptSsopFlagTabpages");
_Static_assert(kOptSsopFlagResize == 0x04, "kOptSsopFlagResize");

// --- Phase 8 accessors: ex_mkrc, ex_loadview ---

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
FILE *nvim_ses_open_exfile(char *fname, int forceit, char *mode)
{
  return open_exfile(fname, forceit, mode);
}
int nvim_ses_fclose(FILE *fd) { return fclose(fd); }
int nvim_ses_do_source(char *fname) { return do_source(fname, false, DOSO_NONE, NULL); }

// OS wrappers
bool nvim_ses_os_isdir(const char *dir) { return os_isdir(dir); }
int nvim_ses_vim_mkdir_emsg(const char *dir, int perm) { return vim_mkdir_emsg(dir, perm); }
int nvim_ses_os_dirname(char *buf, size_t len) { return os_dirname(buf, len); }
int nvim_ses_os_chdir(const char *dir) { return os_chdir(dir); }
int nvim_ses_vim_chdirfile(char *fname)
{
  return vim_chdirfile(fname, kCdCauseOther);
}
void nvim_ses_shorten_fnames(int force) { shorten_fnames(force); }

// Session-related global state
bool nvim_ses_get_p_hls(void) { return p_hls; }
bool nvim_ses_get_no_hlsearch(void) { return no_hlsearch; }
void nvim_ses_set_vim_var_string(const char *val)
{
  set_vim_var_string(VV_THIS_SESSION, val, -1);
}
void nvim_ses_apply_autocmds_session(void)
{
  apply_autocmds(EVENT_SESSIONWRITEPOST, NULL, NULL, false, curbuf);
}
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

// _Static_assert for Phase 8 constants
_Static_assert(kOptSsopFlagSkiprtp == 0x20000, "kOptSsopFlagSkiprtp");
_Static_assert(DOSO_NONE == 0, "DOSO_NONE");
_Static_assert(OPT_GLOBAL == 0x01, "OPT_GLOBAL");
_Static_assert(OPT_SKIPRTP == 0x80, "OPT_SKIPRTP");

static int put_view_curpos(FILE *fd, const win_T *wp, char *spaces)
{
  return rs_put_view_curpos(fd, (win_T *)wp, spaces);
}

static int ses_winsizes(FILE *fd, bool restore_size, win_T *tab_firstwin)
{
  return rs_ses_winsizes(fd, restore_size, tab_firstwin);
}

/// Write commands to "fd" to recursively create windows for frame "fr",
/// horizontally and vertically split.
/// After the commands the last window in the frame is the current window.
///
/// @return  FAIL when writing the commands to "fd" fails.
static int ses_win_rec(FILE *fd, frame_T *fr)
{
  return rs_ses_win_rec(fd, fr);
}

static frame_T *ses_skipframe(frame_T *fr)
{
  return rs_ses_skipframe(fr);
}

static bool ses_do_frame(const frame_T *fr)
  FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_ses_do_frame((frame_T *)fr);
}

static int ses_do_win(win_T *wp)
{
  return rs_ses_do_win(wp);
}

/// Writes an :argument list to the session file.
static int ses_arglist(FILE *fd, char *cmd, garray_T *gap, bool fullname, unsigned *flagp)
{
  return rs_ses_arglist(fd, cmd, gap, fullname, flagp);
}

static char *ses_get_fname(buf_T *buf, const unsigned *flagp)
{
  return rs_ses_get_fname(buf, flagp);
}

static int ses_fname(FILE *fd, buf_T *buf, unsigned *flagp, bool add_eol)
{
  return rs_ses_fname(fd, buf, flagp, add_eol);
}

static char *ses_escape_fname(char *name, unsigned *flagp)
{
  return rs_ses_escape_fname(name, flagp);
}

static int ses_put_fname(FILE *fd, char *name, unsigned *flagp)
{
  return rs_ses_put_fname(fd, name, flagp);
}

/// Write commands to "fd" to restore the view of a window.
static int put_view(FILE *fd, win_T *wp, tabpage_T *tp, bool add_edit, unsigned *flagp,
                    int current_arg_idx)
{
  return rs_put_view(fd, wp, tp, add_edit, flagp, current_arg_idx);
}

static int store_session_globals(FILE *fd)
{
  return rs_store_session_globals(fd);
}

/// Writes commands for restoring the current buffers, for :mksession.
static int makeopens(FILE *fd, char *dirnow)
{
  return rs_makeopens(fd, dirnow);
}

/// ":loadview [nr]"
void ex_loadview(exarg_T *eap)
{
  rs_ex_loadview(eap);
}

/// ":mkexrc", ":mkvimrc", ":mkview", ":mksession".
///
/// Legacy 'sessionoptions'/'viewoptions' flags are always enabled:
///   - kOptSsopFlagUnix: line-endings are LF
///   - kOptSsopFlagSlash: filenames are written with "/" slash
void ex_mkrc(exarg_T *eap)
{
  rs_ex_mkrc(eap);
}

/// @return  the name of the view file for the current buffer.
static char *get_view_file(char c)
{
  return rs_get_view_file(c);
}

int put_eol(FILE *fd)
{
  return rs_put_eol(fd);
}

int put_line(FILE *fd, char *s)
{
  return rs_put_line(fd, s);
}

// _Static_assert for OK/FAIL values used by Rust FFI
_Static_assert(OK == 1, "OK must be 1");
_Static_assert(FAIL == 0, "FAIL must be 0");
