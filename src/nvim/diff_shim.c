/// @file diff.c
///
/// Code for diff'ing two, three or four buffers.
///
/// There are three ways to diff:
/// - Shell out to an external diff program, using files.
/// - Use the compiled-in xdiff library.
/// - Let 'diffexpr' do the work, using files.

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/bufwrite.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/linematch.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"
#include "xdiff/xdiff.h"

// Rust implementations (only declarations still referenced by remaining C code)
extern void rs_clear_diffblock(diff_T *dp);
extern diff_T *rs_diff_alloc_new(tabpage_T *tp, diff_T *dprev, diff_T *dp);
extern diff_T *rs_diff_free(tabpage_T *tp, diff_T *dprev, diff_T *dp);
extern void rs_diff_clear(tabpage_T *tp);
extern void rs_diff_buf_clear(void);
extern void rs_diff_buf_add(buf_T *buf);
extern void rs_diff_buf_adjust(win_T *win);
extern bool rs_diff_equal_entry_full(diff_T *dp, int idx1, int idx2);
extern int rs_lnum_compare(const void *s1, const void *s2);
extern bool rs_valid_diff(diff_T *diff);
extern void rs_set_diff_option(win_T *wp, bool value);
extern void rs_diff_fold_update(diff_T *dp, int skip_idx);
extern int rs_diff_buf_idx_tp(buf_T *buf, tabpage_T *tp);
extern void rs_diff_read(int idx_orig, int idx_new, void *dio);
extern int rs_diff_check_with_linestatus(win_T *wp, linenr_T lnum, int *linestatus);
extern int rs_diff_check_fill(win_T *wp, linenr_T lnum);
extern void rs_diff_set_topline(win_T *fromwin, win_T *towin);
extern linenr_T rs_diff_get_corresponding_line(buf_T *buf1, linenr_T lnum1);
extern bool rs_diff_change_parse(diffline_T *diffline, diffline_change_T *change,
                                 int *change_start, int *change_end);
extern bool rs_diff_find_change(win_T *wp, linenr_T lnum, diffline_T *diffline);
extern void rs_diff_ex_diffupdate(exarg_T *eap);
extern void rs_f_diff_filler(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_nv_diffgetput(bool put, size_t count);
extern void rs_ex_diffthis(exarg_T *eap);
extern void rs_f_diff_hlID(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_diff_redraw(bool dofold);
extern void rs_diff_win_options(win_T *wp, bool addbuf);
extern void rs_ex_diffoff(exarg_T *eap);
extern void rs_ex_diffsplit(exarg_T *eap);
extern void rs_ex_diffgetput(exarg_T *eap);
extern void rs_diffgetput(int addr_count, int idx_cur, int idx_from, int idx_to, linenr_T line1, linenr_T line2);
extern void rs_run_linematch(diff_T *dp);
extern void rs_compute_inline_diff(diff_T *dp);
extern int rs_parse_diffanchors(bool check_only, buf_T *buf, linenr_T *anchors, int *num_anchors);
extern void rs_ex_diffpatch(exarg_T *eap);
extern int rs_diff_write_buffer(buf_T *buf, char **m_ptr, int *m_size,
                                linenr_T start, linenr_T end, int diff_flags);
extern int rs_diff_file_internal(void *dio);
extern int rs_diff_file(void *dio);
extern int rs_check_external_diff(void *dio);

static bool diff_busy = false;         // using diff structs, don't change them
static bool diff_need_update = false;  // ex_diffupdate needs to be called

// Flags obtained from the 'diffopt' option
#define DIFF_FILLER     0x001   // display filler lines
#define DIFF_IBLANK     0x002   // ignore empty lines
#define DIFF_ICASE      0x004   // ignore case
#define DIFF_IWHITE     0x008   // ignore change in white space
#define DIFF_IWHITEALL  0x010   // ignore all white space changes
#define DIFF_IWHITEEOL  0x020   // ignore change in white space at EOL
#define DIFF_HORIZONTAL 0x040   // horizontal splits
#define DIFF_VERTICAL   0x080   // vertical splits
#define DIFF_HIDDEN_OFF 0x100   // diffoff when hidden
#define DIFF_INTERNAL   0x200   // use internal xdiff algorithm
#define DIFF_CLOSE_OFF  0x400   // diffoff when closing window
#define DIFF_FOLLOWWRAP 0x800   // follow the wrap option
#define DIFF_LINEMATCH  0x1000  // match most similar lines within diff
#define DIFF_INLINE_NONE    0x2000  // no inline highlight
#define DIFF_INLINE_SIMPLE  0x4000  // inline highlight with simple algorithm
#define DIFF_INLINE_CHAR    0x8000  // inline highlight with character diff
#define DIFF_INLINE_WORD    0x10000  // inline highlight with word diff
#define DIFF_ANCHOR     0x20000  // use 'diffanchors' to anchor the diff
#define ALL_WHITE_DIFF (DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL)
#define ALL_INLINE (DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD)
#define ALL_INLINE_DIFF (DIFF_INLINE_CHAR | DIFF_INLINE_WORD)
static int diff_flags = DIFF_INTERNAL | DIFF_FILLER | DIFF_CLOSE_OFF
                        | DIFF_LINEMATCH | DIFF_INLINE_CHAR;

static int diff_algorithm = XDF_INDENT_HEURISTIC;
static int linematch_lines = 40;

#define LBUFLEN 50               // length of line in diff file

// kTrue when "diff -a" works, kFalse when it doesn't work,
// kNone when not checked yet
static TriState diff_a_works = kNone;

enum { MAX_DIFF_ANCHORS = 20, };

// used for diff input
typedef struct {
  char *din_fname;   // used for external diff
  mmfile_t din_mmfile;  // used for internal diff
} diffin_T;

// used for diff result
typedef struct {
  char *dout_fname;  // used for external diff
  garray_T dout_ga;     // used for internal diff
} diffout_T;

// used for recording hunks from xdiff
typedef struct {
  linenr_T lnum_orig;
  int count_orig;
  linenr_T lnum_new;
  int count_new;
} diffhunk_T;

extern int rs_parse_diff_ed(const char *line, diffhunk_T *hunk);
extern int rs_parse_diff_unified(const char *line, diffhunk_T *hunk);

// two diff inputs and one result
typedef struct {
  diffin_T dio_orig;      // original file input
  diffin_T dio_new;       // new file input
  diffout_T dio_diff;      // diff result
  int dio_internal;  // using internal diff
} diffio_T;

typedef enum {
  DIFF_ED,
  DIFF_UNIFIED,
  DIFF_NONE,
} diffstyle_T;

#include "diff_shim.c.generated.h"
extern int rs_win_valid(win_T *win);

// Rust FFI declarations (window wrappers removed)
extern void rs_set_fraction(win_T *wp);

// Rust fold FFI declarations
extern void rs_newFoldLevel(void);
extern void rs_foldUpdateAll(win_T *win);

#define FOR_ALL_DIFFBLOCKS_IN_TAB(tp, dp) \
  for ((dp) = (tp)->tp_first_diff; (dp) != NULL; (dp) = (dp)->df_next)

/// Mark all diff buffers in the current tab page for redraw.
/// Thin wrapper -- implementation moved to Rust (rs_diff_redraw in update.rs).
///
/// @param dofold Also recompute the folds
void diff_redraw(bool dofold)
{
  rs_diff_redraw(dofold);
}





/// Completely update the diffs for the buffers involved.
///
/// @param eap can be NULL
void ex_diffupdate(exarg_T *eap)
{
  rs_diff_ex_diffupdate(eap);
}




/// Create a new version of a file from the current buffer and a diff file.
///
/// The buffer is written to a file, also for unmodified buffers (the file
/// could have been produced by autocommands, e.g. the netrw plugin).
///
/// Thin wrapper -- implementation moved to Rust (rs_ex_diffpatch in patch.rs).
///
/// @param eap
void ex_diffpatch(exarg_T *eap)
{
  rs_ex_diffpatch(eap);
}

/// Split the window and edit another file, setting options to show the diffs.
/// Thin wrapper -- implementation moved to Rust (rs_ex_diffsplit in winopts.rs).
///
/// @param eap
void ex_diffsplit(exarg_T *eap)
{
  rs_ex_diffsplit(eap);
}

// Set options to show diffs for the current window -- thin wrapper calling Rust rs_ex_diffthis.
void ex_diffthis(exarg_T *eap)
{
  rs_ex_diffthis(eap);
}

/// Set options in window "wp" for diff mode -- thin wrapper calling Rust rs_diff_win_options.
///
/// @param addbuf Add buffer to diff.
void diff_win_options(win_T *wp, bool addbuf)
{
  rs_diff_win_options(wp, addbuf);
}

/// Set options not to show diffs.  For the current window or all windows.
/// Only in the current tab page -- thin wrapper calling Rust rs_ex_diffoff.
///
/// @param eap
void ex_diffoff(exarg_T *eap)
{
  rs_ex_diffoff(eap);
}


/// Check diff status for line "lnum" in buffer "buf":
///
/// Returns > 0 for inserting that many filler lines above it (never happens
/// when 'diffopt' doesn't contain "filler"). Otherwise returns 0.
///
/// "linestatus" (can be NULL) will be set to:
/// 0 for nothing special.
/// -1 for a line that should be highlighted as changed.
/// -2 for a line that should be highlighted as added/deleted.
///
/// This should only be used for windows where 'diff' is set.
///
/// Note that it's possible for a changed/added/deleted line to also have filler
/// lines above it. This happens when using linematch or using diff anchors (at
/// the anchored lines).
///
/// @param wp
/// @param lnum
/// @param[out] linestatus


/// used for simple inline diff algorithm
static diffline_change_T simple_diffline_change;


/// "dp" and "do" commands -- thin wrapper calling Rust rs_nv_diffgetput.
void nv_diffgetput(bool put, size_t count)
{
  rs_nv_diffgetput(put, count);
}

/// ":diffget" and ":diffput" -- thin wrapper calling Rust rs_ex_diffgetput.
///
/// @param eap
void ex_diffgetput(exarg_T *eap)
{
  rs_ex_diffgetput(eap);
}


/// Checks that the buffer is in diff-mode.


/// "diff_filler()" function -- thin wrapper calling Rust rs_f_diff_filler.
void f_diff_filler(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_diff_filler(argvars, rettv, fptr);
}

/// "diff_hlID()" function -- thin wrapper calling Rust rs_f_diff_hlID.
void f_diff_hlID(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_diff_hlID(argvars, rettv, fptr);
}

// Rust FFI accessor functions
int nvim_get_diff_flags(void) { return diff_flags; }
bool nvim_is_diffexpr_empty(void) { return *p_dex == NUL; }
buf_T *nvim_get_curtab_diffbuf(int idx) { if (idx < 0 || idx >= DB_COUNT) { return NULL; } return curtab->tp_diffbuf[idx]; }
int nvim_get_curtab_diff_invalid(void) { return curtab->tp_diff_invalid; }
diff_T *nvim_get_diff_first_block(void) { return curtab->tp_first_diff; }
diff_T *nvim_diffblock_get_next(diff_T *dp) { if (dp == NULL) { return NULL; } return dp->df_next; }
linenr_T nvim_diffblock_get_lnum(diff_T *dp, int idx) { if (dp == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return dp->df_lnum[idx]; }
linenr_T nvim_diffblock_get_count(diff_T *dp, int idx) { if (dp == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return dp->df_count[idx]; }
void nvim_diffblock_set_lnum(diff_T *dp, int idx, linenr_T lnum) { if (dp == NULL || idx < 0 || idx >= DB_COUNT) { return; } dp->df_lnum[idx] = lnum; }
void nvim_diffblock_set_count(diff_T *dp, int idx, linenr_T count) { if (dp == NULL || idx < 0 || idx >= DB_COUNT) { return; } dp->df_count[idx] = count; }
bool nvim_diffblock_is_linematched(diff_T *dp) { if (dp == NULL) { return false; } return dp->is_linematched; }
void nvim_diffblock_set_linematched(diff_T *dp, bool val) { if (dp == NULL) { return; } dp->is_linematched = val; }
bool nvim_diffblock_has_changes(diff_T *dp) { if (dp == NULL) { return false; } return dp->df_changes.ga_len > 0; }
diff_T *nvim_tabpage_get_first_diff(tabpage_T *tp) { if (tp == NULL) { return NULL; } return tp->tp_first_diff; }
buf_T *nvim_tabpage_get_diffbuf(tabpage_T *tp, int idx) { if (tp == NULL || idx < 0 || idx >= DB_COUNT) { return NULL; } return tp->tp_diffbuf[idx]; }
bool nvim_tabpage_is_diff_invalid(tabpage_T *tp) { if (tp == NULL) { return true; } return tp->tp_diff_invalid; }
void nvim_tabpage_set_diff_invalid(tabpage_T *tp, int val) { if (tp != NULL) { tp->tp_diff_invalid = val != 0; } }
void nvim_tabpage_set_diff_update(tabpage_T *tp, int val) { if (tp != NULL) { tp->tp_diff_update = val != 0; } }
diff_T *nvim_diff_alloc_new(tabpage_T *tp, diff_T *prev, diff_T *next) { diff_T *dnew = xmalloc(sizeof(diff_T)); CLEAR_POINTER(dnew); dnew->df_next = next; if (prev == NULL) { tp->tp_first_diff = dnew; } else { prev->df_next = dnew; } return dnew; }
int nvim_diffblock_get_changes_len(diff_T *dp) { if (dp == NULL) { return 0; } return dp->df_changes.ga_len; }
diffline_change_T *nvim_diffblock_get_change(diff_T *dp, int change_idx) { if (dp == NULL || change_idx < 0 || change_idx >= dp->df_changes.ga_len) { return NULL; } return &((diffline_change_T *)dp->df_changes.ga_data)[change_idx]; }
void nvim_diff_write_buffer(buf_T *buf, void *m, linenr_T start, linenr_T end) { mmfile_t *mm = (mmfile_t *)m; rs_diff_write_buffer(buf, &mm->ptr, &mm->size, start, end, diff_flags); }
int nvim_diff_buf_get_ml_flags(buf_T *buf) { return buf ? buf->b_ml.ml_flags : 0; }
int nvim_diff_ml_get_buf_len(buf_T *buf, linenr_T lnum) { return buf ? ml_get_buf_len(buf, lnum) : 0; }
void nvim_curtab_set_diffbuf(int idx, buf_T *buf) { if (idx >= 0 && idx < DB_COUNT) { curtab->tp_diffbuf[idx] = buf; } }
void nvim_tabpage_set_diffbuf(tabpage_T *tp, int idx, buf_T *buf) { if (tp != NULL && idx >= 0 && idx < DB_COUNT) { tp->tp_diffbuf[idx] = buf; } }
void nvim_tabpage_set_first_diff(tabpage_T *tp, diff_T *dp) { if (tp != NULL) { tp->tp_first_diff = dp; } }
void nvim_diff_set_next(diff_T *dp, diff_T *next) { if (dp != NULL) { dp->df_next = next; } }
void nvim_diffblock_clear_and_free(diff_T *dp) { if (dp != NULL) { ga_clear(&dp->df_changes); xfree(dp); } }
void nvim_diffblock_init_new(diff_T *dp) { if (dp != NULL) { dp->is_linematched = false; dp->has_changes = false; ga_init(&dp->df_changes, sizeof(diffline_change_T), 20); } }
void nvim_set_need_diff_redraw(bool val) { need_diff_redraw = val; }
int nvim_diff_get_linematch_lines(void) { return linematch_lines; }
int nvim_diff_get_diff_flags(void) { return diff_flags; }
void nvim_diff_semsg_e96(void) { semsg(_("E96: Cannot diff more than %" PRId64 " buffers"), (int64_t)DB_COUNT); }
void nvim_redraw_later_win(win_T *wp, int type) { if (wp != NULL) { redraw_later(wp, type); } }
win_T *nvim_tabpage_first_win(tabpage_T *tp) { if (tp == NULL) { return NULL; } if (tp == curtab) { return firstwin; } return tp->tp_firstwin; }
win_T *nvim_win_next(win_T *wp) { if (wp == NULL) { return NULL; } return wp->w_next; }
void nvim_diff_foldUpdate(win_T *wp, linenr_T top, linenr_T bot) { if (wp != NULL) { rs_foldUpdate(wp, top, bot); } }
void nvim_diff_set_diff_option(win_T *wp, bool value) { if (wp == NULL) { return; } win_T *old_curwin = curwin; curwin = wp; curbuf = curwin->w_buffer; curbuf->b_ro_locked++; set_option_value_give_err(kOptDiff, BOOLEAN_OPTVAL(value), OPT_LOCAL); curbuf->b_ro_locked--; curwin = old_curwin; curbuf = curwin->w_buffer; }
const char *nvim_diff_ml_get_buf(buf_T *buf, linenr_T lnum) { if (buf == NULL) { return ""; } return ml_get_buf(buf, lnum); }
char *nvim_diff_xstrdup(const char *s) { if (s == NULL) { return NULL; } return xstrdup(s); }
void nvim_diff_xfree(void *p) { xfree(p); }
int nvim_upd_valid(void) { return UPD_VALID; }
int nvim_upd_some_valid(void) { return UPD_SOME_VALID; }
bool nvim_diff_get_busy(void) { return diff_busy; }
void nvim_diff_set_need_scrollbind(bool val) { diff_need_scrollbind = val; }
linenr_T nvim_diff_maxlnum(void) { return MAXLNUM; }
int nvim_diff_get_algorithm(void) { return diff_algorithm; }
void nvim_diff_set_options(int flags, int context, int linematch, int foldcol, int algorithm) { diff_flags = flags; diff_context = context; linematch_lines = linematch; diff_foldcolumn = foldcol; diff_algorithm = algorithm; }
void nvim_diff_check_scrollbind(void) { check_scrollbind(0, 0); }
int nvim_diff_parse_diffanchors(void) { return rs_parse_diffanchors(true, curbuf, NULL, NULL); }
const char *nvim_diff_get_p_dip(void) { return p_dip; }
void *nvim_diffio_new(bool use_internal) { diffio_T *dio = xcalloc(1, sizeof(diffio_T)); dio->dio_internal = use_internal ? 1 : 0; return dio; }
void nvim_diffio_free(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; xfree(dio); }
bool nvim_diffio_is_internal(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio != NULL && dio->dio_internal; }
void nvim_diffio_init_ga(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio != NULL) { ga_init(&dio->dio_diff.dout_ga, sizeof(diffhunk_T), 100); } }
bool nvim_diffio_alloc_tempfiles(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return false; } dio->dio_orig.din_fname = vim_tempname(); dio->dio_new.din_fname = vim_tempname(); dio->dio_diff.dout_fname = vim_tempname(); return (dio->dio_orig.din_fname != NULL && dio->dio_new.din_fname != NULL && dio->dio_diff.dout_fname != NULL); }
void nvim_diffio_free_tempfiles(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return; } xfree(dio->dio_orig.din_fname); xfree(dio->dio_new.din_fname); xfree(dio->dio_diff.dout_fname); dio->dio_orig.din_fname = NULL; dio->dio_new.din_fname = NULL; dio->dio_diff.dout_fname = NULL; }
int nvim_diffio_write_orig(void *dio_ptr, buf_T *buf, linenr_T start, linenr_T end) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL || buf == NULL) { return FAIL; } if (dio->dio_orig.din_fname == NULL) { return rs_diff_write_buffer(buf, &dio->dio_orig.din_mmfile.ptr, &dio->dio_orig.din_mmfile.size, start, end, diff_flags); } return nvim_diff_write_to_file(buf, dio->dio_orig.din_fname, start, end); }
int nvim_diffio_write_new(void *dio_ptr, buf_T *buf, linenr_T start, linenr_T end) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL || buf == NULL) { return FAIL; } if (dio->dio_new.din_fname == NULL) { return rs_diff_write_buffer(buf, &dio->dio_new.din_mmfile.ptr, &dio->dio_new.din_mmfile.size, start, end, diff_flags); } return nvim_diff_write_to_file(buf, dio->dio_new.din_fname, start, end); }
int nvim_diffio_run_diff(void *dio_ptr) { if (dio_ptr == NULL) { return FAIL; } return rs_diff_file(dio_ptr); }
int nvim_diffio_check_external(void *dio_ptr) { if (dio_ptr == NULL) { return FAIL; } return rs_check_external_diff(dio_ptr); }
void nvim_diffio_clear_new(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return; } if (dio->dio_new.din_fname == NULL) { XFREE_CLEAR(dio->dio_new.din_mmfile.ptr); } else { os_remove(dio->dio_new.din_fname); } }
void nvim_diffio_clear_output(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return; } if (dio->dio_diff.dout_fname == NULL) { ga_clear(&dio->dio_diff.dout_ga); } else { os_remove(dio->dio_diff.dout_fname); } }
void nvim_diffio_clear_orig(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return; } if (dio->dio_orig.din_fname == NULL) { XFREE_CLEAR(dio->dio_orig.din_mmfile.ptr); } else { os_remove(dio->dio_orig.din_fname); } }
int nvim_diffio_get_hunk_count(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return 0; } return dio->dio_diff.dout_ga.ga_len; }
bool nvim_diffio_get_hunk(void *dio_ptr, int idx,
                          linenr_T *lnum_orig, int *count_orig,
                          linenr_T *lnum_new, int *count_new) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL || idx < 0 || idx >= dio->dio_diff.dout_ga.ga_len) { return false; } diffhunk_T *hunks = (diffhunk_T *)dio->dio_diff.dout_ga.ga_data; *lnum_orig = hunks[idx].lnum_orig; *count_orig = hunks[idx].count_orig; *lnum_new = hunks[idx].lnum_new; *count_new = hunks[idx].count_new; return true; }
void *nvim_diffio_open_output(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL || dio->dio_diff.dout_fname == NULL) { return NULL; } return os_fopen(dio->dio_diff.dout_fname, "r"); }
bool nvim_diff_fgets(void *fd, char *buf, int buflen) { if (fd == NULL) { return true; } return vim_fgets(buf, buflen, (FILE *)fd); }
void nvim_diff_fclose(void *fd) { if (fd != NULL) { fclose((FILE *)fd); } }
bool nvim_diff_buf_valid(buf_T *buf) { return buf_valid(buf); }
void nvim_diff_buf_check_timestamp(buf_T *buf) { if (buf != NULL) { buf_check_timestamp(buf); } }
bool nvim_diff_buf_is_loaded(buf_T *buf) { return buf != NULL && buf->b_ml.ml_mfp != NULL; }
void nvim_diff_curtab_set_first_diff(diff_T *dp) { curtab->tp_first_diff = dp; }
diff_T *nvim_diff_curtab_get_first_diff(void) { return curtab->tp_first_diff; }
bool nvim_eap_forceit(const exarg_T *eap) { return eap != NULL && eap->forceit; }
buf_T *nvim_diff_curtab_diffbuf(int idx) { if (idx < 0 || idx >= DB_COUNT) { return NULL; } return curtab->tp_diffbuf[idx]; }
void nvim_diff_invalidate_cursor(void) { curwin->w_valid_cursor.lnum = 0; }
void nvim_diff_fire_diffupdated(void) { apply_autocmds(EVENT_DIFFUPDATED, NULL, NULL, false, curbuf); }
bool nvim_diff_get_need_update(void) { return diff_need_update; }
void nvim_diff_set_need_update(bool val) { diff_need_update = val; }
void nvim_diff_set_busy(bool val) { diff_busy = val; }
int nvim_diff_max_anchors(void) { return MAX_DIFF_ANCHORS; }
void nvim_diff_emsg_e98(void) { emsg(_("E98: Cannot read diff output")); }
void nvim_diff_emsg_anchors(void) { emsg(_(e_failed_to_find_all_diff_anchors)); }
int nvim_diff_parse_buf_anchors(buf_T *buf, linenr_T *anchors, int max_anchors) { if (buf == NULL) { return -1; } int num = 0; if (rs_parse_diffanchors(false, buf, anchors, &num) != OK) { return -1; } return num; }
void nvim_diff_sort_lnums(linenr_T *arr, int count) { if (arr != NULL && count > 0) { qsort(arr, (size_t)count, sizeof(linenr_T), rs_lnum_compare); } }
int nvim_diff_parse_ed(const char *line, linenr_T *lnum_orig, int *count_orig,
                       linenr_T *lnum_new, int *count_new) { diffhunk_T hunk = { 0 }; int r = rs_parse_diff_ed(line, &hunk); if (r == OK) { *lnum_orig = hunk.lnum_orig; *count_orig = hunk.count_orig; *lnum_new = hunk.lnum_new; *count_new = hunk.count_new; } return r; }
int nvim_diff_parse_unified(const char *line, linenr_T *lnum_orig, int *count_orig,
                            linenr_T *lnum_new, int *count_new) { diffhunk_T hunk = { 0 }; int r = rs_parse_diff_unified(line, &hunk); if (r == OK) { *lnum_orig = hunk.lnum_orig; *count_orig = hunk.count_orig; *lnum_new = hunk.lnum_new; *count_new = hunk.count_new; } return r; }
int nvim_diff_get_context(void) { return diff_context; }
bool nvim_diff_hasFolding(win_T *wp, linenr_T lnum) { return hasFolding(wp, lnum, NULL, NULL); }
bool nvim_diff_hasFolding_topline(win_T *wp, linenr_T lnum, linenr_T *topline) { return hasFolding(wp, lnum, topline, NULL); }
bool nvim_diff_decor_conceal_line(win_T *wp, linenr_T lnum) { return decor_conceal_line(wp, lnum - 1, false); }
void nvim_diff_invalidate_botline_win(win_T *wp) { invalidate_botline(wp); }
void nvim_diff_changed_line_abv_curs_win(win_T *wp) { changed_line_abv_curs_win(wp); }
void nvim_diff_check_topfill(win_T *wp, bool down) { check_topfill(wp, down); }
void nvim_diff_setpcmark(void) { setpcmark(); }
void nvim_diff_run_linematch(diff_T *dp) { rs_run_linematch(dp); }
bool nvim_diffblock_get_has_changes(diff_T *dp) { if (dp == NULL) { return false; } return dp->has_changes; }
void nvim_diffblock_set_has_changes(diff_T *dp, bool val) { if (dp != NULL) { dp->has_changes = val; } }
void nvim_diffblock_reset_changes_len(diff_T *dp) { if (dp != NULL) { dp->df_changes.ga_len = 0; } }
diffline_change_T *nvim_diff_get_simple_change(void) { return &simple_diffline_change; }
int nvim_diffchange_get_start_lnum_off(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_start_lnum_off[idx]; }
int nvim_diffchange_get_end_lnum_off(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_end_lnum_off[idx]; }
colnr_T nvim_diffchange_get_start(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_start[idx]; }
colnr_T nvim_diffchange_get_end(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_end[idx]; }
bool nvim_diff_is_simple_change(diffline_change_T *change) { return change == &simple_diffline_change; }
const char *nvim_diff_skipwhite(const char *p) { return skipwhite(p); }
// Phase 2 (diff_file_internal) accessors
char *nvim_diffio_get_orig_ptr(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_orig.din_mmfile.ptr : NULL; }
int nvim_diffio_get_orig_size(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_orig.din_mmfile.size : 0; }
char *nvim_diffio_get_new_ptr(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_new.din_mmfile.ptr : NULL; }
int nvim_diffio_get_new_size(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_new.din_mmfile.size : 0; }
void *nvim_diffio_get_dout(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? &dio->dio_diff : NULL; }
void nvim_diff_emsg_e960(void) { emsg(_("E960: Problem creating the internal diff")); }
// Phase 3 (diff_file, diff_write) accessors
const char *nvim_diffio_get_orig_fname(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_orig.din_fname : NULL; }
const char *nvim_diffio_get_new_fname(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_new.din_fname : NULL; }
const char *nvim_diffio_get_diff_fname(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio ? dio->dio_diff.dout_fname : NULL; }
int nvim_diff_write_to_file(buf_T *buf, const char *fname, linenr_T start, linenr_T end) { if (end < 0) { end = buf->b_ml.ml_line_count; } int save_ml_flags = buf->b_ml.ml_flags; char *save_ff = buf->b_p_ff; buf->b_p_ff = xstrdup("unix"); const bool save_cmod_flags = cmdmod.cmod_flags; cmdmod.cmod_flags |= CMOD_LOCKMARKS; if (end < start) { end = start; buf->b_ml.ml_flags |= ML_EMPTY; } int r = buf_write(buf, (char *)fname, NULL, start, end, NULL, false, false, false, true); cmdmod.cmod_flags = save_cmod_flags; free_string_option(buf->b_p_ff); buf->b_p_ff = save_ff; buf->b_ml.ml_flags = (buf->b_ml.ml_flags & ~ML_EMPTY) | (save_ml_flags & ML_EMPTY); return r; }
int nvim_diff_get_a_works(void) { return (int)diff_a_works; }
void nvim_diff_set_a_works(int val) { diff_a_works = (TriState)val; }
void nvim_diff_eval_diff(const char *orig, const char *new_f, const char *diff) { eval_diff((char *)orig, (char *)new_f, (char *)diff); }
int nvim_diff_run_external_shell(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return FAIL; } char *tmp_orig = dio->dio_orig.din_fname; char *tmp_new = dio->dio_new.din_fname; char *tmp_diff = dio->dio_diff.dout_fname; const size_t len = (strlen(tmp_orig) + strlen(tmp_new) + strlen(tmp_diff) + strlen(p_srr) + 27); char *const cmd = xmalloc(len); if (os_env_exists("DIFF_OPTIONS", true)) { os_unsetenv("DIFF_OPTIONS"); } vim_snprintf(cmd, len, "diff %s%s%s%s%s%s%s%s %s", diff_a_works == kFalse ? "" : "-a ", "", (diff_flags & DIFF_IWHITE) ? "-b " : "", (diff_flags & DIFF_IWHITEALL) ? "-w " : "", (diff_flags & DIFF_IWHITEEOL) ? "-Z " : "", (diff_flags & DIFF_IBLANK) ? "-B " : "", (diff_flags & DIFF_ICASE) ? "-i " : "", tmp_orig, tmp_new); append_redir(cmd, len, p_srr, tmp_diff); block_autocmds(); call_shell(cmd, kShellOptFilter | kShellOptSilent | kShellOptDoOut, NULL); unblock_autocmds(); xfree(cmd); return OK; }
// Phase 4 (check_external_diff) accessors
void *nvim_diff_fopen_write(const char *fname) { return os_fopen(fname, "w"); }
bool nvim_diff_fwrite_line(void *fd, const char *data, size_t len) { return fwrite(data, len, 1, (FILE *)fd) == 1; }
void *nvim_diff_fopen_read(const char *fname) { return os_fopen(fname, "r"); }
void nvim_diff_emsg_e810(void) { emsg(_("E810: Cannot read or write temp files")); }
void nvim_diff_emsg_e97(void) { emsg(_("E97: Cannot create diffs")); }
// Phase 5 (inline_compute) accessors
int nvim_xdiff_internal_run(const char *orig_data, int orig_size, const char *new_data, int new_size, void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return FAIL; } dio->dio_orig.din_mmfile.ptr = (char *)orig_data; dio->dio_orig.din_mmfile.size = orig_size; dio->dio_new.din_mmfile.ptr = (char *)new_data; dio->dio_new.din_mmfile.size = new_size; return rs_diff_file_internal(dio); }
uint64_t *nvim_diffbuf_get_chartab(int idx) { if (idx < 0 || idx >= DB_COUNT || curtab->tp_diffbuf[idx] == NULL) { return NULL; } return curtab->tp_diffbuf[idx]->b_chartab; }
void nvim_diffblock_append_change(diff_T *dp, const int *dc_start, const int *dc_end, const int *dc_start_lnum_off, const int *dc_end_lnum_off) { if (dp == NULL) { return; } diffline_change_T change = { 0 }; for (int i = 0; i < DB_COUNT; i++) { change.dc_start[i] = dc_start[i]; change.dc_end[i] = dc_end[i]; change.dc_start_lnum_off[i] = dc_start_lnum_off[i]; change.dc_end_lnum_off[i] = dc_end_lnum_off[i]; } GA_APPEND(diffline_change_T, &dp->df_changes, change); }
// Phase 1 accessors: f_diff_filler
void nvim_diffout_append_hunk(void *dout, linenr_T lnum_orig, int count_orig, linenr_T lnum_new, int count_new) { diffout_T *d = (diffout_T *)dout; if (d == NULL) { return; } GA_APPEND(diffhunk_T, &(d->dout_ga), ((diffhunk_T){ .lnum_orig = lnum_orig, .count_orig = count_orig, .lnum_new = lnum_new, .count_new = count_new, })); }
linenr_T nvim_diff_tv_get_lnum(typval_T *argvars) { return tv_get_lnum(argvars); }
// Phase 2 accessors: nv_diffgetput and ex_diffthis
void nvim_vim_beep_operator(void) { vim_beep(kOptBoFlagOperator); }
void nvim_diff_call_nv_ex_diffgetput(int cmdidx, const char *arg, int addr_count, linenr_T line1, linenr_T line2) { exarg_T ea; CLEAR_FIELD(ea); ea.cmdidx = (cmdidx_T)cmdidx; ea.arg = (char *)arg; ea.addr_count = addr_count; ea.line1 = line1; ea.line2 = line2; rs_ex_diffgetput(&ea); }
// Phase 3 (diff_win_options / ex_diffoff) accessors
bool nvim_win_get_w_p_diff_saved(win_T *wp) { return wp->w_p_diff_saved != 0; }
void nvim_win_set_w_p_diff_saved(win_T *wp, bool val) { wp->w_p_diff_saved = val ? 1 : 0; }
bool nvim_win_get_w_p_scb(win_T *wp) { return wp->w_p_scb != 0; }
void nvim_win_set_w_p_scb(win_T *wp, bool val) { wp->w_p_scb = val ? 1 : 0; }
bool nvim_win_get_w_p_scb_save(win_T *wp) { return wp->w_p_scb_save != 0; }
void nvim_win_set_w_p_scb_save(win_T *wp, bool val) { wp->w_p_scb_save = val ? 1 : 0; }
bool nvim_win_get_w_p_crb(win_T *wp) { return wp->w_p_crb != 0; }
void nvim_win_set_w_p_crb(win_T *wp, bool val) { wp->w_p_crb = val ? 1 : 0; }
bool nvim_win_get_w_p_crb_save(win_T *wp) { return wp->w_p_crb_save != 0; }
void nvim_win_set_w_p_crb_save(win_T *wp, bool val) { wp->w_p_crb_save = val ? 1 : 0; }
bool nvim_win_get_w_p_wrap(win_T *wp) { return wp->w_p_wrap != 0; }
void nvim_win_set_w_p_wrap(win_T *wp, bool val) { wp->w_p_wrap = val ? 1 : 0; }
bool nvim_win_get_w_p_wrap_save(win_T *wp) { return wp->w_p_wrap_save != 0; }
void nvim_win_set_w_p_wrap_save(win_T *wp, bool val) { wp->w_p_wrap_save = val ? 1 : 0; }
bool nvim_win_get_w_p_fen(win_T *wp) { return wp->w_p_fen != 0; }
void nvim_win_set_w_p_fen(win_T *wp, bool val) { wp->w_p_fen = val ? 1 : 0; }
bool nvim_win_get_w_p_fen_save(win_T *wp) { return wp->w_p_fen_save != 0; }
void nvim_win_set_w_p_fen_save(win_T *wp, bool val) { wp->w_p_fen_save = val ? 1 : 0; }
bool nvim_win_get_w_p_diff(win_T *wp) { return wp->w_p_diff; }
linenr_T nvim_win_get_w_p_fdl(win_T *wp) { return (linenr_T)wp->w_p_fdl; }
void nvim_win_set_w_p_fdl(win_T *wp, linenr_T val) { wp->w_p_fdl = (OptInt)val; }
linenr_T nvim_win_get_w_p_fdl_save(win_T *wp) { return (linenr_T)wp->w_p_fdl_save; }
void nvim_win_set_w_p_fdl_save(win_T *wp, linenr_T val) { wp->w_p_fdl_save = (OptInt)val; }
void nvim_win_free_and_set_fdm(win_T *wp, const char *val) { free_string_option(wp->w_p_fdm); wp->w_p_fdm = xstrdup(val ? val : ""); }
void nvim_win_free_and_set_fdc(win_T *wp, const char *val) { free_string_option(wp->w_p_fdc); wp->w_p_fdc = xstrdup(val ? val : "0"); }
void nvim_win_free_and_set_fdm_save(win_T *wp, const char *val) { free_string_option(wp->w_p_fdm_save); wp->w_p_fdm_save = xstrdup(val ? val : ""); }
void nvim_win_free_and_set_fdc_save(win_T *wp, const char *val) { free_string_option(wp->w_p_fdc_save); wp->w_p_fdc_save = xstrdup(val ? val : ""); }
bool nvim_win_get_fdm_save_empty(win_T *wp) { return *wp->w_p_fdm_save == NUL; }
bool nvim_win_get_fdc_save_empty(win_T *wp) { return *wp->w_p_fdc_save == NUL; }
const char *nvim_win_get_w_p_fdm(win_T *wp) { return wp->w_p_fdm; }
const char *nvim_win_get_w_p_fdm_save(win_T *wp) { return wp->w_p_fdm_save; }
const char *nvim_win_get_w_p_fdc_save(win_T *wp) { return wp->w_p_fdc_save; }
int nvim_diff_get_foldcolumn(void) { return diff_foldcolumn; }
void nvim_diff_set_fdm_to_diff(win_T *wp) { set_option_direct_for(kOptFoldmethod, STATIC_CSTR_AS_OPTVAL("diff"), OPT_LOCAL, 0, kOptScopeWin, wp); }
void nvim_diff_changed_window_setting(win_T *wp) { changed_window_setting(wp); }
bool nvim_diff_sbo_has_hor(void) { return vim_strchr(p_sbo, 'h') != NULL; }
void nvim_diff_do_cmdline_cmd(const char *cmd) { do_cmdline_cmd(cmd); }
bool nvim_diff_is_curwin(win_T *wp) { return wp == curwin; }
void nvim_diff_changed_window_foldlevel_reset(win_T *wp) { win_T *old_curwin = curwin; curwin = wp; rs_newFoldLevel(); curwin = old_curwin; }
int nvim_upd_not_valid(void) { return UPD_NOT_VALID; }
// Phase 4 (ex_diffsplit) accessors
void nvim_diff_validate_cursor_curwin(void) { validate_cursor(curwin); }
void nvim_diff_set_cmdmod_tab_zero(void) { cmdmod.cmod_tab = 0; }
void nvim_diff_do_exedit_with_old_curwin(exarg_T *eap, win_T *old_curwin) { do_exedit(eap, old_curwin); }
void nvim_diff_set_curwin_w_p_diff(bool val) { curwin->w_p_diff = val; }
win_T *nvim_diff_get_curwin(void) { return curwin; }
int nvim_diff_get_CMD_split(void) { return (int)CMD_split; }
static bufref_T diff_split_bufref;
void nvim_diff_bufref_set_to_curbuf(void *r) { set_bufref((bufref_T *)r, curbuf); }
bool nvim_diff_bufref_valid(const void *r) { return bufref_valid((bufref_T *)r); }
buf_T *nvim_diff_bufref_get_buf(const void *r) { return ((bufref_T *)r)->br_buf; }
void *nvim_diff_bufref_alloc(void) { return &diff_split_bufref; }
void nvim_diff_bufref_free(void *r) { (void)r; /* static storage, no-op */ }
// Phase 5 (ex_diffgetput) accessors
void nvim_diff_emsg_e99(void) { emsg(_("E99: Current buffer is not in diff mode")); }
void nvim_diff_emsg_e793(void) { emsg(_("E793: No other buffer in diff mode is modifiable")); }
void nvim_diff_emsg_e100(void) { emsg(_("E100: No other buffer in diff mode")); }
void nvim_diff_emsg_e101(void) { emsg(_("E101: More than two buffers in diff mode, don't know which one to use")); }
void nvim_diff_semsg_e102(const char *arg) { semsg(_("E102: Can't find buffer \"%s\""), arg); }
void nvim_diff_semsg_e103(const char *arg) { semsg(_("E103: Buffer \"%s\" is not in diff mode"), arg); }
void nvim_diff_emsg_e787(void) { emsg(_("E787: Buffer changed unexpectedly")); }
bool nvim_diff_buf_is_modifiable(buf_T *buf) { return MODIFIABLE(buf); }
buf_T *nvim_diff_get_curbuf(void) { return curbuf; }
int nvim_diff_buflist_findpat(const char *arg, const char *end) { return buflist_findpat(arg, end, false, true, false); }
buf_T *nvim_diff_buflist_findnr(int nr) { return buflist_findnr(nr); }
static aco_save_T diff_aucmd_aco;
void nvim_diff_aucmd_prepbuf_idx(int idx) { aucmd_prepbuf(&diff_aucmd_aco, curtab->tp_diffbuf[idx]); }
void nvim_diff_aucmd_restbuf(void) { aucmd_restbuf(&diff_aucmd_aco); }
void nvim_diff_change_warning_curbuf(void) { change_warning(curbuf, 0); }
bool nvim_diff_curbuf_changed(void) { return curbuf->b_changed; }
bool nvim_diff_key_typed(void) { return KeyTyped; }
void nvim_diff_u_sync(void) { u_sync(false); }
void nvim_diff_check_cursor_curwin(void) { check_cursor(curwin); }
void nvim_diff_changed_line_abv_curs(void) { changed_line_abv_curs(); }
// Phase 4 (diffgetput) accessors
int nvim_diff_u_save(linenr_T top, linenr_T bot) { return u_save(top, bot); }
int nvim_diff_ml_delete(linenr_T lnum) { return ml_delete(lnum); }
int nvim_diff_ml_append(linenr_T lnum, const char *line, int len, bool newfile) { return ml_append(lnum, (char *)line, len, newfile); }
bool nvim_diff_buf_is_empty_curbuf(void) { return buf_is_empty(curbuf); }
linenr_T nvim_diff_curbuf_ml_line_count_direct(void) { return curbuf->b_ml.ml_line_count; }
void nvim_diff_mark_adjust(linenr_T line1, linenr_T line2, linenr_T amount, linenr_T amount_after) { mark_adjust(line1, line2, amount, amount_after, kExtmarkNOOP); }
void nvim_diff_extmark_adjust(linenr_T line1, linenr_T line2, linenr_T amount, linenr_T amount_after) { extmark_adjust(curbuf, line1, line2, amount, amount_after, kExtmarkUndo); }
void nvim_diff_changed_lines(linenr_T lnum, int col, linenr_T lnum_end, linenr_T xtra) { changed_lines(curbuf, lnum, col, lnum_end, xtra, true); }
const char *nvim_diff_ml_get_buf_diffbuf(int idx, linenr_T nr) { if (idx < 0 || idx >= DB_COUNT || curtab->tp_diffbuf[idx] == NULL) { return NULL; } return ml_get_buf(curtab->tp_diffbuf[idx], nr); }
linenr_T nvim_diff_diffbuf_ml_line_count(int idx) { if (idx < 0 || idx >= DB_COUNT || curtab->tp_diffbuf[idx] == NULL) { return 0; } return curtab->tp_diffbuf[idx]->b_ml.ml_line_count; }
int nvim_diff_get_CMD_diffget(void) { return (int)CMD_diffget; }
int nvim_diff_get_CMD_diffput(void) { return (int)CMD_diffput; }
linenr_T nvim_diff_curbuf_ml_line_count(void) { return curbuf->b_ml.ml_line_count; }
bool nvim_diff_curtab_first_diff_is_null(void) { return curtab->tp_first_diff == NULL; }
bool nvim_diff_win_get_w_p_fdm_starts_d(win_T *wp) { return wp->w_p_fdm[0] == 'd'; }
buf_T *nvim_diff_get_curtab_diffbuf_idx(int idx) { if (idx < 0 || idx >= DB_COUNT) { return NULL; } return curtab->tp_diffbuf[idx]; }
bool nvim_diff_curbuf_is_curtab_diffbuf(int idx_to) { if (idx_to < 0 || idx_to >= DB_COUNT) { return false; } return rs_diff_buf_idx_tp(curbuf, curtab) == idx_to; }
void nvim_diff_fire_diffupdated_curbuf(void) { apply_autocmds(EVENT_DIFFUPDATED, NULL, NULL, false, curbuf); }
// Phase 6 (parse_diffanchors) accessors
const char *nvim_diff_get_buf_dia(buf_T *buf) { return buf->b_p_dia; }
const char *nvim_diff_get_p_dia(void) { return p_dia; }
void nvim_diff_set_curwin_curbuf(win_T *wp) { curwin = wp; curbuf = wp->w_buffer; }
void nvim_diff_restore_curwin_curbuf(win_T *old_curwin) { curwin = old_curwin; curbuf = old_curwin->w_buffer; }
linenr_T nvim_diff_buf_get_ml_line_count(buf_T *buf) { return buf->b_ml.ml_line_count; }
void nvim_diff_emsg_hidden_diff_anchors(void) { emsg(_(e_diff_anchors_with_hidden_windows)); }
void nvim_diff_emsg_invrange(void) { emsg(_(e_invrange)); }
void nvim_diff_semsg_too_many_anchors(int max) { semsg(_(e_cannot_have_more_than_nr_diff_anchors), max); }
win_T *nvim_diff_get_firstwin(void) { return firstwin; }
bool nvim_win_get_w_p_diff_bool(win_T *wp) { return wp->w_p_diff; }
void nvim_diff_emsg(const char *msg) { emsg(msg); }
// Phase 7 (ex_diffpatch) accessors
char *nvim_diff_vim_tempname(void) { return vim_tempname(); }
int nvim_diff_buf_write_curbuf(const char *fname) { return buf_write(curbuf, (char *)fname, NULL, 1, curbuf->b_ml.ml_line_count, NULL, false, false, false, true); }
char *nvim_diff_FullName_save(const char *fname) { return FullName_save((char *)fname, false); }
char *nvim_diff_vim_strsave_shellescape(const char *s) { return vim_strsave_shellescape(s, true, true); }
int nvim_diff_os_dirname(char *buf, int size) { return os_dirname(buf, (size_t)size); }
int nvim_diff_os_chdir(const char *dir) { return os_chdir(dir); }
const char *nvim_diff_vim_gettempdir(void) { return vim_gettempdir(); }
void nvim_diff_shorten_fnames(void) { shorten_fnames(true); }
bool nvim_diff_is_patchexpr_set(void) { return *p_pex != NUL; }
void nvim_diff_eval_patch(const char *orig, const char *diff, const char *out) { eval_patch((char *)orig, (char *)diff, (char *)out); }
void nvim_diff_call_shell_filter(const char *cmd) { block_autocmds(); call_shell((char *)cmd, kShellOptFilter, NULL); unblock_autocmds(); }
bool nvim_diff_os_fileinfo_size(const char *fname, uint64_t *size_out) { FileInfo fi; bool ok = os_fileinfo(fname, &fi); if (ok && size_out != NULL) { *size_out = os_fileinfo_size(&fi); } return ok; }
void nvim_diff_os_remove(const char *fname) { os_remove(fname); }
char *nvim_diff_xstrnsave(const char *s, size_t len) { return xstrnsave(s, len); }
int nvim_diff_get_MAXPATHL(void) { return MAXPATHL; }
char *nvim_diff_xmalloc(size_t size) { return xmalloc(size); }
void nvim_diff_vim_snprintf_patch(char *buf, size_t buflen, const char *tmp_new, const char *tmp_orig, const char *esc_name) { vim_snprintf(buf, buflen, "patch -o %s %s < %s", tmp_new, tmp_orig, esc_name); }
size_t nvim_diff_strlen(const char *s) { return s ? strlen(s) : 0; }
const char *nvim_diff_get_curbuf_fname(void) { return curbuf->b_fname; }
void nvim_diff_emsg_e816(void) { emsg(_("E816: Cannot read patch output")); }
void nvim_diff_emsg_prev_dir(void) { emsg(_(e_prev_dir)); }
void nvim_diff_ex_file(exarg_T *eap) { ex_file(eap); }
bool nvim_diff_augroup_exists_filetypedetect(void) { return augroup_exists("filetypedetect"); }
// Phase 3 accessors: f_diff_hlID
int64_t nvim_curbuf_changedtick_i64(void) { return (int64_t)buf_get_changedtick(curbuf); }
int nvim_diff_tv_get_number_idx(typval_T *argvars, int idx) { return (int)tv_get_number(&argvars[idx]); }
int nvim_diff_hlf_add(void) { return (int)HLF_ADD; }
int nvim_diff_hlf_chd(void) { return (int)HLF_CHD; }
int nvim_diff_hlf_txd(void) { return (int)HLF_TXD; }
int nvim_diff_hlf_txa(void) { return (int)HLF_TXA; }
diffline_change_T *nvim_diff_diffline_get_change(diffline_T *dl, int i) { if (!dl || i < 0 || i >= dl->num_changes) { return NULL; } return &dl->changes[i]; }
colnr_T nvim_diff_change_dc_start(diffline_change_T *dc, int idx) { if (!dc || idx < 0 || idx >= DB_COUNT) { return 0; } return dc->dc_start[idx]; }
colnr_T nvim_diff_change_dc_end(diffline_change_T *dc, int idx) { if (!dc || idx < 0 || idx >= DB_COUNT) { return 0; } return dc->dc_end[idx]; }

