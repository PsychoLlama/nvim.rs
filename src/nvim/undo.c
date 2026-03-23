// undo.c: multi level undo facility

// The saved lines are stored in a list of lists (one for each buffer):
//
// b_u_oldhead------------------------------------------------+
//                                                            |
//                                                            V
//                +--------------+    +--------------+    +--------------+
// b_u_newhead--->| u_header     |    | u_header     |    | u_header     |
//                |     uh_next------>|     uh_next------>|     uh_next---->NULL
//         NULL<--------uh_prev  |<---------uh_prev  |<---------uh_prev  |
//                |     uh_entry |    |     uh_entry |    |     uh_entry |
//                +--------|-----+    +--------|-----+    +--------|-----+
//                         |                   |                   |
//                         V                   V                   V
//                +--------------+    +--------------+    +--------------+
//                | u_entry      |    | u_entry      |    | u_entry      |
//                |     ue_next  |    |     ue_next  |    |     ue_next  |
//                +--------|-----+    +--------|-----+    +--------|-----+
//                         |                   |                   |
//                         V                   V                   V
//                +--------------+            NULL                NULL
//                | u_entry      |
//                |     ue_next  |
//                +--------|-----+
//                         |
//                         V
//                        etc.
//
// Each u_entry list contains the information for one undo or redo.
// curbuf->b_u_curhead points to the header of the last undo (the next redo),
// or is NULL if nothing has been undone (end of the branch).
//
// For keeping alternate undo/redo branches the uh_alt field is used.  Thus at
// each point in the list a branch may appear for an alternate to redo.  The
// uh_seq field is numbered sequentially to be able to find a newer or older
// branch.
//
//                 +---------------+    +---------------+
// b_u_oldhead --->| u_header      |    | u_header      |
//                 |   uh_alt_next ---->|   uh_alt_next ----> NULL
//         NULL <----- uh_alt_prev |<------ uh_alt_prev |
//                 |   uh_prev     |    |   uh_prev     |
//                 +-----|---------+    +-----|---------+
//                       |                    |
//                       V                    V
//                 +---------------+    +---------------+
//                 | u_header      |    | u_header      |
//                 |   uh_alt_next |    |   uh_alt_next |
// b_u_newhead --->|   uh_alt_prev |    |   uh_alt_prev |
//                 |   uh_prev     |    |   uh_prev     |
//                 +-----|---------+    +-----|---------+
//                       |                    |
//                       V                    V
//                     NULL             +---------------+    +---------------+
//                                      | u_header      |    | u_header      |
//                                      |   uh_alt_next ---->|   uh_alt_next |
//                                      |   uh_alt_prev |<------ uh_alt_prev |
//                                      |   uh_prev     |    |   uh_prev     |
//                                      +-----|---------+    +-----|---------+
//                                            |                    |
//                                           etc.                 etc.
//
//
// All data is allocated and will all be freed when the buffer is unloaded.

// Uncomment the next line for including the u_check() function.  This warns
// for errors in the debug information.
// #define U_DEBUG 1
#define UH_MAGIC 0x18dade       // value for uh_magic when in use
#define UE_MAGIC 0xabc123       // value for ue_magic when in use

#include <assert.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/extmark_defs.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/sha256.h"
#include "nvim/spell.h"
#include "nvim/state.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"

// Rust FFI function declarations (accessors only - implementations are exported directly)
extern list_T *rs_u_eval_tree(buf_T *buf, const u_header_T *first_uhp);
extern char *rs_f_undofile(const char *fname);

#include "undo.c.generated.h"

// Rust fold FFI declaration
extern void rs_foldOpenCursor(void);

static const char e_undo_list_corrupt[]
  = N_("E439: Undo list corrupt");
static const char e_undo_line_missing[]
  = N_("E440: Undo line missing");
static const char e_write_error_in_undo_file_str[]
  = N_("E829: Write error in undo file: %s");

// used in undo_end() to report number of added and deleted lines
static int u_newcount, u_oldcount;

// When 'u' flag included in 'cpoptions', we behave like vi.  Need to remember
// the action that "u" should do.
static bool undo_undoes = false;

static int lastmark = 0;

#if defined(U_DEBUG)
// Check the undo structures for being valid.  Print a warning when something
// looks wrong.
static int seen_b_u_curhead;
static int seen_b_u_newhead;
static int header_count;

static void u_check_tree(u_header_T *uhp, u_header_T *exp_uh_next, u_header_T *exp_uh_alt_prev)
{
  if (uhp == NULL) {
    return;
  }
  header_count++;
  if (uhp == curbuf->b_u_curhead && ++seen_b_u_curhead > 1) {
    emsg("b_u_curhead found twice (looping?)");
    return;
  }
  if (uhp == curbuf->b_u_newhead && ++seen_b_u_newhead > 1) {
    emsg("b_u_newhead found twice (looping?)");
    return;
  }

  if (uhp->uh_magic != UH_MAGIC) {
    emsg("uh_magic wrong (may be using freed memory)");
  } else {
    // Check pointers back are correct.
    if (uhp->uh_next.ptr != exp_uh_next) {
      emsg("uh_next wrong");
      smsg(0, "expected: 0x%x, actual: 0x%x",
           exp_uh_next, uhp->uh_next.ptr);
    }
    if (uhp->uh_alt_prev.ptr != exp_uh_alt_prev) {
      emsg("uh_alt_prev wrong");
      smsg(0, "expected: 0x%x, actual: 0x%x",
           exp_uh_alt_prev, uhp->uh_alt_prev.ptr);
    }

    // Check the undo tree at this header.
    for (u_entry_T *uep = uhp->uh_entry; uep != NULL; uep = uep->ue_next) {
      if (uep->ue_magic != UE_MAGIC) {
        emsg("ue_magic wrong (may be using freed memory)");
        break;
      }
    }

    // Check the next alt tree.
    u_check_tree(uhp->uh_alt_next.ptr, uhp->uh_next.ptr, uhp);

    // Check the next header in this branch.
    u_check_tree(uhp->uh_prev.ptr, uhp, NULL);
  }
}

static void u_check(int newhead_may_be_NULL)
{
  seen_b_u_newhead = 0;
  seen_b_u_curhead = 0;
  header_count = 0;

  u_check_tree(curbuf->b_u_oldhead, NULL, NULL);

  if (seen_b_u_newhead == 0 && curbuf->b_u_oldhead != NULL
      && !(newhead_may_be_NULL && curbuf->b_u_newhead == NULL)) {
    semsg("b_u_newhead invalid: 0x%x", curbuf->b_u_newhead);
  }
  if (curbuf->b_u_curhead != NULL && seen_b_u_curhead == 0) {
    semsg("b_u_curhead invalid: 0x%x", curbuf->b_u_curhead);
  }
  if (header_count != curbuf->b_u_numhead) {
    emsg("b_u_numhead invalid");
    smsg(0, "expected: %" PRId64 ", actual: %" PRId64,
         (int64_t)header_count, (int64_t)curbuf->b_u_numhead);
  }
}

#endif


/// Get the 'undolevels' value for the current buffer.
static OptInt get_undolevel(buf_T *buf)
{
  if (buf->b_p_ul == NO_LOCAL_UNDOLEVEL) {
    return p_ul;
  }
  return buf->b_p_ul;
}

static inline void zero_fmark_additional_data(fmark_T *fmarks)
{
  for (size_t i = 0; i < NMARKS; i++) {
    XFREE_CLEAR(fmarks[i].additional_data);
  }
}


// Static assertions for Rust FFI constant verification
_Static_assert(kExtmarkSplice == 0, "kExtmarkSplice must be 0");
_Static_assert(kExtmarkMove == 1, "kExtmarkMove must be 1");
_Static_assert(NMARKS == 26, "NMARKS must be 26");
_Static_assert(UH_CHANGED == 0x01, "UH_CHANGED must be 0x01");
_Static_assert(UH_EMPTYBUF == 0x02, "UH_EMPTYBUF must be 0x02");
_Static_assert(UH_RELOAD == 0x04, "UH_RELOAD must be 0x04");
_Static_assert(MAXLNUM == 0x7fffffff, "MAXLNUM must be 0x7fffffff");
_Static_assert(kExtmarkNOOP == 0, "kExtmarkNOOP must be 0");
_Static_assert(kOptFdoFlagUndo == 0x200, "kOptFdoFlagUndo must be 0x200");


/// "undofile(name)" function
void f_undofile(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  const char *const fname = tv_get_string(&argvars[0]);
  rettv->vval.v_string = rs_f_undofile(fname);
}

/// "undotree(expr)" function
void f_undotree(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_dict_alloc_ret(rettv);

  typval_T *const tv = &argvars[0];
  buf_T *const buf = tv->v_type == VAR_UNKNOWN ? curbuf : get_buf_arg(tv);
  if (buf == NULL) {
    return;
  }

  dict_T *dict = rettv->vval.v_dict;

  tv_dict_add_nr(dict, S_LEN("synced"), (varnumber_T)buf->b_u_synced);
  tv_dict_add_nr(dict, S_LEN("seq_last"), (varnumber_T)buf->b_u_seq_last);
  tv_dict_add_nr(dict, S_LEN("save_last"), (varnumber_T)buf->b_u_save_nr_last);
  tv_dict_add_nr(dict, S_LEN("seq_cur"), (varnumber_T)buf->b_u_seq_cur);
  tv_dict_add_nr(dict, S_LEN("time_cur"), (varnumber_T)buf->b_u_time_cur);
  tv_dict_add_nr(dict, S_LEN("save_cur"), (varnumber_T)buf->b_u_save_nr_cur);

  tv_dict_add_list(dict, S_LEN("entries"), rs_u_eval_tree(buf, buf->b_u_oldhead));
}


// ============================================================================
// Rust FFI accessor functions
// ============================================================================

// Buffer undo field accessors
u_header_T *nvim_buf_get_b_u_oldhead(buf_T *buf)
{
  return buf->b_u_oldhead;
}

u_header_T *nvim_buf_get_b_u_newhead(buf_T *buf)
{
  return buf->b_u_newhead;
}

u_header_T *nvim_buf_get_b_u_curhead(buf_T *buf)
{
  return buf->b_u_curhead;
}

int nvim_buf_get_b_u_numhead(buf_T *buf)
{
  return buf->b_u_numhead;
}

bool nvim_buf_get_b_u_synced(buf_T *buf)
{
  return buf->b_u_synced;
}

char *nvim_buf_get_b_u_line_ptr(buf_T *buf)
{
  return buf->b_u_line_ptr;
}

linenr_T nvim_buf_get_b_u_line_lnum(buf_T *buf)
{
  return buf->b_u_line_lnum;
}

void nvim_buf_set_b_u_oldhead(buf_T *buf, u_header_T *val)
{
  buf->b_u_oldhead = val;
}

void nvim_buf_set_b_u_newhead(buf_T *buf, u_header_T *val)
{
  buf->b_u_newhead = val;
}

void nvim_buf_set_b_u_curhead(buf_T *buf, u_header_T *val)
{
  buf->b_u_curhead = val;
}

void nvim_buf_set_b_u_numhead(buf_T *buf, int val)
{
  buf->b_u_numhead = val;
}

void nvim_buf_set_b_u_synced(buf_T *buf, bool val)
{
  buf->b_u_synced = val;
}

void nvim_buf_set_b_u_line_ptr(buf_T *buf, char *val)
{
  buf->b_u_line_ptr = val;
}

void nvim_buf_set_b_u_line_lnum(buf_T *buf, linenr_T val)
{
  buf->b_u_line_lnum = val;
}

// Buffer state accessors
bool nvim_buf_get_b_changed(buf_T *buf)
{
  return buf->b_changed;
}

bool nvim_bt_dontwrite(buf_T *buf)
{
  return bt_dontwrite(buf);
}


// Global buffer iteration
buf_T *nvim_get_firstbuf(void)
{
  return firstbuf;
}

buf_T *nvim_buf_get_next(buf_T *buf)
{
  return buf->b_next;
}

// Static assertions for u_entry_T layout verification (Phase 1).
// These verify field offsets so the Rust repr(C) UEntry struct stays in sync.
_Static_assert(offsetof(u_entry_T, ue_next) == 0,
               "ue_next must be first field in u_entry_T");
_Static_assert(offsetof(u_entry_T, ue_top) == 8,
               "ue_top offset mismatch in u_entry_T (expected 8 on 64-bit)");
_Static_assert(offsetof(u_entry_T, ue_bot) == 12,
               "ue_bot offset mismatch in u_entry_T");
_Static_assert(offsetof(u_entry_T, ue_lcount) == 16,
               "ue_lcount offset mismatch in u_entry_T");
_Static_assert(offsetof(u_entry_T, ue_array) == 24,
               "ue_array offset mismatch in u_entry_T (pointer after padding)");
_Static_assert(offsetof(u_entry_T, ue_size) == 32,
               "ue_size offset mismatch in u_entry_T");
_Static_assert(sizeof(u_entry_T) == 40,
               "u_entry_T size mismatch: expected 40 bytes on 64-bit");

// Static assertions for Phase 2+3: fmark_T, visualinfo_T, u_header_T layout
// These verify that the Rust repr(C) structs match the C struct layouts.
_Static_assert(sizeof(pos_T) == 12, "pos_T size mismatch");
_Static_assert(offsetof(pos_T, lnum) == 0, "pos_T.lnum offset mismatch");
_Static_assert(offsetof(pos_T, col) == 4, "pos_T.col offset mismatch");
_Static_assert(offsetof(pos_T, coladd) == 8, "pos_T.coladd offset mismatch");
_Static_assert(sizeof(fmark_T) == 40, "fmark_T size mismatch");
_Static_assert(offsetof(fmark_T, mark) == 0, "fmark_T.mark offset mismatch");
_Static_assert(offsetof(fmark_T, fnum) == 12, "fmark_T.fnum offset mismatch");
_Static_assert(offsetof(fmark_T, timestamp) == 16, "fmark_T.timestamp offset mismatch");
_Static_assert(offsetof(fmark_T, view) == 24, "fmark_T.view offset mismatch");
_Static_assert(offsetof(fmark_T, additional_data) == 32, "fmark_T.additional_data offset mismatch");
_Static_assert(sizeof(visualinfo_T) == 32, "visualinfo_T size mismatch");
_Static_assert(offsetof(visualinfo_T, vi_start) == 0, "visualinfo_T.vi_start offset mismatch");
_Static_assert(offsetof(visualinfo_T, vi_end) == 12, "visualinfo_T.vi_end offset mismatch");
_Static_assert(offsetof(visualinfo_T, vi_mode) == 24, "visualinfo_T.vi_mode offset mismatch");
_Static_assert(offsetof(visualinfo_T, vi_curswant) == 28, "visualinfo_T.vi_curswant offset mismatch");
_Static_assert(offsetof(u_header_T, uh_next) == 0, "u_header_T.uh_next offset mismatch");
_Static_assert(offsetof(u_header_T, uh_prev) == 8, "u_header_T.uh_prev offset mismatch");
_Static_assert(offsetof(u_header_T, uh_alt_next) == 16, "u_header_T.uh_alt_next offset mismatch");
_Static_assert(offsetof(u_header_T, uh_alt_prev) == 24, "u_header_T.uh_alt_prev offset mismatch");
_Static_assert(offsetof(u_header_T, uh_seq) == 32, "u_header_T.uh_seq offset mismatch");
_Static_assert(offsetof(u_header_T, uh_walk) == 36, "u_header_T.uh_walk offset mismatch");
_Static_assert(offsetof(u_header_T, uh_entry) == 40, "u_header_T.uh_entry offset mismatch");
_Static_assert(offsetof(u_header_T, uh_getbot_entry) == 48, "u_header_T.uh_getbot_entry offset mismatch");
_Static_assert(offsetof(u_header_T, uh_cursor) == 56, "u_header_T.uh_cursor offset mismatch");
_Static_assert(offsetof(u_header_T, uh_cursor_vcol) == 68, "u_header_T.uh_cursor_vcol offset mismatch");
_Static_assert(offsetof(u_header_T, uh_flags) == 72, "u_header_T.uh_flags offset mismatch");
_Static_assert(offsetof(u_header_T, uh_namedm) == 80, "u_header_T.uh_namedm offset mismatch");
_Static_assert(offsetof(u_header_T, uh_extmark) == 1120, "u_header_T.uh_extmark offset mismatch");
_Static_assert(offsetof(u_header_T, uh_visual) == 1144, "u_header_T.uh_visual offset mismatch");
_Static_assert(offsetof(u_header_T, uh_time) == 1176, "u_header_T.uh_time offset mismatch");
_Static_assert(offsetof(u_header_T, uh_save_nr) == 1184, "u_header_T.uh_save_nr offset mismatch");

// Error message wrappers
void nvim_iemsg_undo_list_corrupt(void)
{
  iemsg(_(e_undo_list_corrupt));
}

void nvim_iemsg_undo_line_missing(void)
{
  iemsg(_(e_undo_line_missing));
}

void nvim_iemsg_undo_line_numbers_wrong(void)
{
  iemsg(_("E438: u_undo: line numbers wrong"));
}

// Global state accessors
int nvim_get_no_u_sync(void)
{
  return no_u_sync;
}

// Wrapper for get_undolevel
OptInt nvim_get_undolevel(buf_T *buf)
{
  return get_undolevel(buf);
}

// Buffer b_did_warn accessor
void nvim_buf_set_b_did_warn(buf_T *buf, bool val)
{
  buf->b_did_warn = val;
}

// Buffer save_nr accessors
int nvim_buf_get_b_u_save_nr_last(buf_T *buf)
{
  return buf->b_u_save_nr_last;
}

void nvim_buf_set_b_u_save_nr_last(buf_T *buf, int val)
{
  buf->b_u_save_nr_last = val;
}

void nvim_buf_set_b_u_save_nr_cur(buf_T *buf, int val)
{
  buf->b_u_save_nr_cur = val;
}

// undo_allowed accessors
bool nvim_buf_is_modifiable(buf_T *buf)
{
  return MODIFIABLE(buf);
}

int nvim_get_sandbox(void)
{
  return sandbox;
}

// undo_allowed error message wrappers
void nvim_emsg_modifiable(void)
{
  emsg(_(e_modifiable));
}

void nvim_emsg_sandbox(void)
{
  emsg(_(e_sandbox));
}

void nvim_emsg_textlock(void)
{
  emsg(_(e_textlock));
}

void nvim_emsg_undojoin_after_undo(void)
{
  emsg(_("E790: undojoin is not allowed after undo"));
}

// u_undo/u_redo accessors
bool nvim_has_cpo_undo(void)
{
  return vim_strchr(p_cpo, CPO_UNDO) != NULL;
}

bool nvim_get_undo_undoes(void)
{
  return undo_undoes;
}

void nvim_set_undo_undoes(bool val)
{
  undo_undoes = val;
}


// u_undo_and_forget accessors
int nvim_buf_get_b_u_seq_cur(buf_T *buf)
{
  return buf->b_u_seq_cur;
}

void nvim_buf_set_b_u_seq_cur(buf_T *buf, int val)
{
  buf->b_u_seq_cur = val;
}

int nvim_buf_get_b_u_seq_last(buf_T *buf)
{
  return buf->b_u_seq_last;
}

void nvim_buf_set_b_u_seq_last(buf_T *buf, int val)
{
  buf->b_u_seq_last = val;
}

// u_doit accessors
bool nvim_buf_ml_is_empty(buf_T *buf)
{
  return buf->b_ml.ml_flags & ML_EMPTY;
}

int nvim_get_u_newcount(void)
{
  return u_newcount;
}

void nvim_set_u_newcount(int val)
{
  u_newcount = val;
}

int nvim_get_u_oldcount(void)
{
  return u_oldcount;
}

void nvim_set_u_oldcount(int val)
{
  u_oldcount = val;
}

void nvim_msg_ext_set_kind_undo(void)
{
  msg_ext_set_kind("undo");
}

void nvim_change_warning_curbuf(void)
{
  change_warning(curbuf, 0);
}

void nvim_beep_flush(void)
{
  beep_flush();
}

void nvim_msg_oldest_change(void)
{
  msg(_("Already at oldest change"), 0);
}

void nvim_msg_newest_change(void)
{
  msg(_("Already at newest change"), 0);
}

// Buffer line access (infrastructure for future migration)
// Returns allocated copy of line - caller must free with nvim_xfree
char *nvim_ml_get_buf_copy(buf_T *buf, linenr_T lnum)
{
  return xstrdup(ml_get_buf(buf, lnum));
}

void nvim_fast_breakcheck(void)
{
  fast_breakcheck();
}

bool nvim_undo_got_int(void)
{
  return got_int;
}

time_t nvim_time_now(void)
{
  return time(NULL);
}

// Window cursor access for undo header
void nvim_get_curwin_cursor(linenr_T *lnum, colnr_T *col, colnr_T *coladd)
{
  *lnum = curwin->w_cursor.lnum;
  *col = curwin->w_cursor.col;
  *coladd = curwin->w_cursor.coladd;
}

bool nvim_curwin_virtual_active(void)
{
  return virtual_active(curwin);
}

colnr_T nvim_getviscol(void)
{
  return getviscol();
}

// u_savecommon infrastructure
void nvim_buf_set_b_new_change(buf_T *buf, bool val)
{
  buf->b_new_change = val;
}

void nvim_buf_set_b_u_time_cur(buf_T *buf, time_t val)
{
  buf->b_u_time_cur = val;
}

// Copy marks and visual from buffer to undo header
void nvim_uhp_copy_marks_visual(buf_T *buf, u_header_T *uhp)
{
  zero_fmark_additional_data(buf->b_namedm);
  memmove(uhp->uh_namedm, buf->b_namedm, sizeof(buf->b_namedm[0]) * NMARKS);
  uhp->uh_visual = buf->b_visual;
}


// Error message wrapper
void nvim_emsg_line_count_changed(void)
{
  emsg(_("E881: Line count changed unexpectedly"));
}

// Check if buf equals curbuf
bool nvim_buf_is_curbuf(buf_T *buf)
{
  return buf == curbuf;
}


// Get b_u_line_colnr
colnr_T nvim_buf_get_b_u_line_colnr(buf_T *buf)
{
  return buf->b_u_line_colnr;
}

// Set b_u_line_colnr
void nvim_buf_set_b_u_line_colnr(buf_T *buf, colnr_T val)
{
  buf->b_u_line_colnr = val;
}

// Get curwin->w_cursor.col (undo-specific)
colnr_T nvim_undo_curwin_get_cursor_col(void)
{
  return curwin->w_cursor.col;
}

// Set curwin->w_cursor.col (undo-specific)
void nvim_undo_curwin_set_cursor_col(colnr_T col)
{
  curwin->w_cursor.col = col;
}

// Set curwin->w_cursor.lnum (undo-specific)
void nvim_undo_curwin_set_cursor_lnum(linenr_T lnum)
{
  curwin->w_cursor.lnum = lnum;
}

// Call check_cursor_col for curwin
void nvim_check_cursor_col_curwin(void)
{
  check_cursor_col(curwin);
}

// Perform the u_undoline line replacement and swap operation
// Swaps b_u_line_ptr with the current line content
void nvim_u_undoline_replace_and_swap(void)
{
  linenr_T lnum = curbuf->b_u_line_lnum;
  char *oldp = xstrdup(ml_get_buf(curbuf, lnum));
  ml_replace(lnum, curbuf->b_u_line_ptr, true);
  extmark_splice_cols(curbuf, (int)lnum - 1, 0, (colnr_T)strlen(oldp),
                      (colnr_T)strlen(curbuf->b_u_line_ptr), kExtmarkUndo);
  changed_bytes(lnum, 0);
  xfree(curbuf->b_u_line_ptr);
  curbuf->b_u_line_ptr = oldp;
}

// Get curwin->w_cursor.lnum (undo-specific)
linenr_T nvim_undo_curwin_get_cursor_lnum(void)
{
  return curwin->w_cursor.lnum;
}

// undo_time accessors
time_t nvim_buf_get_b_u_time_cur(buf_T *buf)
{
  return buf->b_u_time_cur;
}

int nvim_buf_get_b_u_save_nr_cur(buf_T *buf)
{
  return buf->b_u_save_nr_cur;
}

bool nvim_text_locked(void)
{
  return text_locked();
}

void nvim_text_locked_msg(void)
{
  text_locked_msg();
}

time_t nvim_undo_os_time(void)
{
  return os_time();
}

// Strftime wrapper for Rust time formatting
size_t nvim_undo_strftime(char *buf, size_t buflen, const char *fmt, time_t tt)
{
  struct tm curtime;
  os_localtime_r(&tt, &curtime);
  return strftime(buf, buflen, fmt, &curtime);
}

// Return the localized "N second(s) ago" string
const char *nvim_undo_time(int64_t seconds)
{
  static char buf[256];
  vim_snprintf(buf, sizeof(buf),
               NGETTEXT("%" PRId64 " second ago",
                        "%" PRId64 " seconds ago", (uint32_t)seconds),
               seconds);
  return buf;
}

int nvim_inc_lastmark(void)
{
  return ++lastmark;
}

void nvim_internal_error_undo_time(void)
{
  internal_error("undo_time()");
}

void nvim_semsg_undo_number_not_found(int64_t step)
{
  semsg(_("E830: Undo number %" PRId64 " not found"), step);
}

// ============================================================================
// Undo File I/O FFI Functions
// ============================================================================

// File system operations still needed by other Rust crates (memline, quickfix)
bool nvim_os_path_exists(const char *path)
{
  return os_path_exists(path);
}

int nvim_os_remove(const char *path)
{
  return os_remove(path);
}

// Option accessors
int nvim_get_p_verbose(void)
{
  return p_verbose;
}

void nvim_set_p_verbose(int val) { p_verbose = val; }

bool nvim_get_p_fs(void)
{
  return p_fs;
}

// u_sync wrapper (still called from ex_cmds and window Rust crates)
void nvim_u_sync(bool force)
{
  u_sync(force);
}

// Buffer line count and line accessors for hash computation
linenr_T nvim_buf_get_b_ml_line_count(buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

// File info for Unix ownership checks
#ifdef UNIX
int nvim_undo_set_file_group(int fd, const char *orig_path, const char *undo_path, int perm)
{
  FileInfo file_info_old;
  FileInfo file_info_new;
  if (orig_path != NULL
      && os_fileinfo(orig_path, &file_info_old)
      && os_fileinfo(undo_path, &file_info_new)
      && file_info_old.stat.st_gid != file_info_new.stat.st_gid
      && os_fchown(fd, (uv_uid_t)-1, (uv_gid_t)file_info_old.stat.st_gid)) {
    // Group change failed, adjust permissions
    return (perm & 0707) | ((perm & 07) << 3);
  }
  return perm;
}
#else
int nvim_undo_set_file_group(int fd, const char *orig_path, const char *undo_path, int perm)
{
  (void)fd;
  (void)orig_path;
  (void)undo_path;
  return perm;
}
#endif

// ============================================================================
// Extmark Accessor Functions (for Rust FFI - extmark crate)
// ============================================================================

/// Force get undo header for current operation (wrapper for Rust FFI).
u_header_T *nvim_u_force_get_undo_header(buf_T *buf)
{
  return u_force_get_undo_header(buf);
}

/// Get extmark undo vector pointer from undo header.
extmark_undo_vec_t *nvim_uhp_get_extmark(u_header_T *uhp)
{
  return &uhp->uh_extmark;
}

// ============================================================================
// Undo File I/O Message Functions (for Rust FFI)
// ============================================================================

void nvim_undo_cannot_write_no_dir(void)
{
  verb_msg(_("Cannot write undo file in any directory in 'undodir'"));
}

void nvim_undo_will_not_overwrite_cannot_read(const char *file_name)
{
  smsg(0, _("Will not overwrite with undo file, cannot read: %s"), file_name);
}

void nvim_undo_will_not_overwrite_not_undo(const char *file_name)
{
  smsg(0, _("Will not overwrite, this is not an undo file: %s"), file_name);
}

void nvim_undo_skip_write_nothing(void)
{
  verb_msg(_("Skipping undo file write, nothing to undo"));
}

void nvim_undo_write_error(const char *file_name)
{
  semsg(_(e_write_error_in_undo_file_str), file_name);
}

void nvim_undo_writing(const char *file_name)
{
  smsg(0, _("Writing undo file: %s"), file_name);
}

void nvim_undo_reading(const char *file_name)
{
  smsg(0, _("Reading undo file: %s"), file_name);
}

void nvim_undo_not_reading_owner_differs(const char *file_name)
{
  smsg(0, _("Not reading undo file, owner differs: %s"), file_name);
}

void nvim_undo_cannot_open_for_reading(const char *file_name)
{
  semsg(_("E822: Cannot open undo file for reading: %s"), file_name);
}

void nvim_undo_not_undo_file(const char *file_name)
{
  semsg(_("E823: Not an undo file: %s"), file_name);
}

void nvim_undo_incompatible_version(const char *file_name)
{
  semsg(_("E824: Incompatible undo file: %s"), file_name);
}

void nvim_undo_corruption_error(const char *what, const char *file_name)
{
  semsg(_("E825: Corrupted undo file (%s): %s"), what, file_name);
}

void nvim_undo_file_changed_warning(void)
{
  give_warning(_("File contents changed, cannot use undo info"), true);
}

void nvim_undo_finished_reading(const char *file_name)
{
  smsg(0, _("Finished reading undo file %s"), file_name);
}

bool nvim_undo_check_owner(const char *orig_name, const char *file_name)
{
#ifdef UNIX
  FileInfo file_info_orig;
  FileInfo file_info_undo;
  if (os_fileinfo(orig_name, &file_info_orig)
      && os_fileinfo(file_name, &file_info_undo)
      && file_info_orig.stat.st_uid != file_info_undo.stat.st_uid
      && file_info_undo.stat.st_uid != getuid()) {
    return false;
  }
#endif
  return true;
}

// ============================================================================
// Phase 3: u_undoredo FFI Helpers
// ============================================================================

// Save named marks and visual info from buffer before undo/redo.
// Clears additional_data, saves namedm to uhp_saved_namedm[],
// and saves visual info. Returns opaque handle to saved state.
// The saved state is stored directly in the undo header's namedm array
// after swapping.
void nvim_undoredo_save_marks(buf_T *buf, u_header_T *curhead)
{
  zero_fmark_additional_data(buf->b_namedm);
}

// Restore named marks from undo header to buffer and vice versa
void nvim_undoredo_restore_marks(buf_T *buf, u_header_T *curhead,
                                 const fmark_T *saved_namedm)
{
  for (int i = 0; i < NMARKS; i++) {
    if (curhead->uh_namedm[i].mark.lnum != 0) {
      free_fmark(buf->b_namedm[i]);
      buf->b_namedm[i] = curhead->uh_namedm[i];
    }
    if (saved_namedm[i].mark.lnum != 0) {
      curhead->uh_namedm[i] = saved_namedm[i];
    } else {
      curhead->uh_namedm[i].mark.lnum = 0;
    }
  }
}

// Swap visual info between buffer and undo header
void nvim_undoredo_swap_visual(buf_T *buf, u_header_T *curhead,
                               const visualinfo_T *saved_visual)
{
  if (curhead->uh_visual.vi_start.lnum != 0) {
    buf->b_visual = curhead->uh_visual;
    curhead->uh_visual = *saved_visual;
  }
}

// Get saved namedm array and visual info from buffer (for save before undo)
// Copies buf->b_namedm to output array and returns buf->b_visual
void nvim_undoredo_get_buf_marks(buf_T *buf, fmark_T *out_namedm,
                                 visualinfo_T *out_visual)
{
  memmove(out_namedm, buf->b_namedm, sizeof(fmark_T) * NMARKS);
  *out_visual = buf->b_visual;
}

// Set b_op_start and b_op_end initial values
void nvim_undoredo_init_op_marks(buf_T *buf)
{
  buf->b_op_start.lnum = buf->b_ml.ml_line_count;
  buf->b_op_start.col = 0;
  buf->b_op_end.lnum = 0;
  buf->b_op_end.col = 0;
}

// Get b_op_start.lnum
linenr_T nvim_buf_get_op_start_lnum(buf_T *buf)
{
  return buf->b_op_start.lnum;
}

// Get b_op_end.lnum
linenr_T nvim_buf_get_op_end_lnum(buf_T *buf)
{
  return buf->b_op_end.lnum;
}

// Set b_op_start.lnum
void nvim_buf_set_op_start_lnum(buf_T *buf, linenr_T lnum)
{
  buf->b_op_start.lnum = lnum;
}

// Adjust b_op_start.lnum by delta
void nvim_buf_adjust_op_start_lnum(buf_T *buf, linenr_T delta)
{
  buf->b_op_start.lnum += delta;
}

// Set b_op_end.lnum
void nvim_buf_set_op_end_lnum(buf_T *buf, linenr_T lnum)
{
  buf->b_op_end.lnum = lnum;
}

// Adjust b_op_end.lnum by delta
void nvim_buf_adjust_op_end_lnum(buf_T *buf, linenr_T delta)
{
  buf->b_op_end.lnum += delta;
}

// Clamp op marks to ml_line_count
void nvim_undoredo_clamp_op_marks(buf_T *buf)
{
  buf->b_op_start.lnum = MIN(buf->b_op_start.lnum, buf->b_ml.ml_line_count);
  buf->b_op_end.lnum = MIN(buf->b_op_end.lnum, buf->b_ml.ml_line_count);
}

// Set ML_EMPTY flag if needed
void nvim_undoredo_set_ml_empty(buf_T *buf, int old_flags)
{
  if ((old_flags & UH_EMPTYBUF) && buf_is_empty(buf)) {
    buf->b_ml.ml_flags |= ML_EMPTY;
  }
}

// Cursor adjustment for u_undoredo:
// Handle the complex cursor positioning logic after undo/redo
void nvim_undoredo_adjust_cursor(u_header_T *curhead)
{
  // If the cursor is only off by one line, put it at the same position as
  // before starting the change (for the "o" command).
  if (curhead->uh_cursor.lnum + 1 == curwin->w_cursor.lnum
      && curwin->w_cursor.lnum > 1) {
    curwin->w_cursor.lnum--;
  }
  if (curwin->w_cursor.lnum <= curbuf->b_ml.ml_line_count) {
    if (curhead->uh_cursor.lnum == curwin->w_cursor.lnum) {
      curwin->w_cursor.col = curhead->uh_cursor.col;
      if (virtual_active(curwin) && curhead->uh_cursor_vcol >= 0) {
        coladvance(curwin, curhead->uh_cursor_vcol);
      } else {
        curwin->w_cursor.coladd = 0;
      }
    } else {
      beginline(BL_SOL | BL_FIX);
    }
  } else {
    curwin->w_cursor.col = 0;
    curwin->w_cursor.coladd = 0;
  }
  check_cursor(curwin);
}

// ============================================================================
// Phase 4: u_undo_end + helpers FFI
// ============================================================================

// Redraw conceal for all windows showing this buffer
void nvim_undo_end_redraw_conceal(buf_T *buf)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (wp->w_buffer == buf && wp->w_p_cole > 0) {
      redraw_later(wp, UPD_NOT_VALID);
    }
  }
}

// Check VIsual and call check_pos
void nvim_undo_end_check_visual(buf_T *buf)
{
  if (VIsual_active) {
    check_pos(buf, &VIsual);
  }
}

// Format and display the undo end message
void nvim_undo_end_smsg(int64_t count, const char *msgstr, bool did_undo,
                         int64_t seq, const char *timebuf)
{
  smsg_keep(0, _("%" PRId64 " %s; %s #%" PRId64 "  %s"),
            count,
            _(msgstr),
            did_undo ? _("before") : _("after"),
            seq,
            timebuf);
}

// ============================================================================
// Phase 5: u_get_undo_file_name FFI Helpers
// ============================================================================

// Resolve symlink if available, returning resolved path or original
// Returns allocated copy
char *nvim_undo_resolve_symlink(const char *ffname)
{
#ifdef HAVE_READLINK
  char fname_buf[MAXPATHL];
  if (resolve_symlink(ffname, fname_buf) == OK) {
    return xstrdup(fname_buf);
  }
#endif
  return xstrdup(ffname);
}

// Get p_udir option value
const char *nvim_undo_get_p_udir(void)
{
  return p_udir;
}

// Wrapper for copy_option_part
size_t nvim_undo_copy_option_part(const char **dirp, char *buf, size_t maxlen)
{
  return copy_option_part((char **)dirp, buf, maxlen, ",");
}

// Create directory recursively. Returns 0 on success, error code on failure.
// On failure, *failed_dir is set to the directory that failed.
int nvim_undo_mkdir_recurse(const char *dir, char **failed_dir)
{
  return os_mkdir_recurse(dir, 0755, failed_dir, NULL);
}

// Emit a semsg with one string argument (used from Rust undo code)
void nvim_undo_semsg(const char *msg, const char *arg)
{
  semsg(msg, arg);
}

// Format and emit E5003 error message
void nvim_undo_semsg_mkdir(const char *failed_dir, int err)
{
  semsg(_("E5003: Unable to create directory \"%s\" for undo file: %s"),
        failed_dir, os_strerror(err));
}

// Get path_tail for a string (returns offset into the string)
size_t nvim_undo_path_tail_offset(const char *path)
{
  return (size_t)(path_tail(path) - path);
}

// concat_fnames wrapper
char *nvim_undo_concat_fnames(const char *dir, const char *fname)
{
  return concat_fnames(dir, fname, true);
}

// Get MAXPATHL value
size_t nvim_undo_get_maxpathl(void)
{
  return MAXPATHL;
}

// ============================================================================
// Ex Command FFI Functions (for Rust FFI)
// ============================================================================

void nvim_undo_msg_simple(const char *s)
{
  msg(s, 0);
}

void nvim_msg_start(void)
{
  msg_start();
}

void nvim_msg_end(void)
{
  msg_end();
}

void nvim_undo_msg_puts_hl_title(const char *s)
{
  msg_puts_hl(s, HLF_T, false);
}

// ============================================================================
// Phase 5: VimL function FFI wrappers
// ============================================================================

list_T *nvim_tv_list_alloc(void)
{
  return tv_list_alloc(kListLenMayKnow);
}

// VimL typval wrappers (still needed by eval crate without #[link_name])
dict_T *nvim_tv_dict_alloc(void)
{
  return tv_dict_alloc();
}

void nvim_tv_list_append_dict(list_T *list, dict_T *dict)
{
  tv_list_append_dict(list, dict);
}

void nvim_tv_dict_add_nr(dict_T *dict, const char *key, size_t key_len, varnumber_T nr)
{
  tv_dict_add_nr(dict, key, key_len, nr);
}

void nvim_tv_dict_add_list(dict_T *dict, const char *key, size_t key_len, list_T *list)
{
  tv_dict_add_list(dict, key, key_len, list);
}

char *nvim_FullName_save(const char *fname, bool force)
{
  return FullName_save(fname, force);
}

// ============================================================================
// Memline Accessor Functions (for Rust FFI - u_undoredo migration)
// ============================================================================

/// Delete line 'lnum' in current buffer. Returns OK/FAIL.
int nvim_ml_delete_lnum(linenr_T lnum)
{
  return ml_delete(lnum);
}

/// Delete line 'lnum' in current buffer with flags. Returns OK/FAIL.
int nvim_ml_delete_flags(linenr_T lnum, int flags)
{
  return ml_delete_flags(lnum, flags);
}

/// Append line with flags. Returns OK/FAIL.
int nvim_ml_append_flags(linenr_T lnum, const char *line, colnr_T len, int flags)
{
  return ml_append_flags(lnum, (char *)line, len, flags);
}

/// Replace line in current buffer. Returns OK/FAIL.
int nvim_ml_replace_lnum(linenr_T lnum, const char *line, bool copy)
{
  return ml_replace(lnum, (char *)line, copy);
}

/// Block/unblock autocommands
void nvim_block_autocmds(void)
{
  block_autocmds();
}

void nvim_unblock_autocmds(void)
{
  unblock_autocmds();
}

/// Mark adjust for undo
void nvim_undo_mark_adjust(linenr_T top, linenr_T bot, linenr_T amount, linenr_T amount_after)
{
  mark_adjust(top, bot, amount, amount_after, kExtmarkNOOP);
}

/// Check spell for window
bool nvim_spell_check_window(win_T *win)
{
  return spell_check_window(win);
}

/// Redraw window line
void nvim_redrawWinline(win_T *win, linenr_T lnum)
{
  redrawWinline(win, lnum);
}

/// Apply extmark undo
void nvim_extmark_apply_undo(u_header_T *uhp, size_t idx, bool undo)
{
  if (idx < kv_size(uhp->uh_extmark)) {
    extmark_apply_undo(kv_A(uhp->uh_extmark, idx), undo);
  }
}

/// Check position validity
void nvim_check_pos(buf_T *buf, pos_T *pos)
{
  check_pos(buf, pos);
}

/// Current window handle accessor
win_T *nvim_undo_get_curwin(void)
{
  return curwin;
}

/// Window buffer accessor
buf_T *nvim_undo_win_get_buffer(win_T *win)
{
  return win->w_buffer;
}

/// Set window cursor
void nvim_undo_win_set_cursor_pos(win_T *win, linenr_T lnum, colnr_T col, colnr_T coladd)
{
  win->w_cursor.lnum = lnum;
  win->w_cursor.col = col;
  win->w_cursor.coladd = coladd;
}

/// Get window cursor line
linenr_T nvim_undo_win_get_cursor_lnum(win_T *win)
{
  return win->w_cursor.lnum;
}

/// Save line for undo (returns allocated string)
char *nvim_u_save_line_for_undo(buf_T *buf, linenr_T lnum)
{
  (void)buf;
  return xstrdup(ml_get_buf(curbuf, lnum));
}

/// Get global_busy flag
bool nvim_get_global_busy(void)
{
  return global_busy;
}

/// Increment global_busy (for breaking :global command on error)
void nvim_inc_global_busy(void)
{
  global_busy++;
}

/// Check if messaging is allowed
bool nvim_messaging(void)
{
  return messaging();
}

/// Get KeyTyped flag
bool nvim_undo_get_key_typed(void)
{
  return KeyTyped;
}

/// Get fdo_flags for fold options
int nvim_undo_get_fdo_flags(void)
{
  return fdo_flags;
}



/// Increment no_u_sync.
void nvim_inc_no_u_sync(void) { no_u_sync++; }
/// Decrement no_u_sync.
void nvim_dec_no_u_sync(void) { no_u_sync--; }

/// Increment sandbox.
void nvim_inc_sandbox(void) { sandbox++; }
/// Decrement sandbox.
void nvim_dec_sandbox(void) { sandbox--; }

/// Wrap u_inssub(lnum). Returns 1 on OK, 0 on FAIL.
int nvim_u_inssub(int lnum) { return u_inssub((linenr_T)lnum) == OK ? 1 : 0; }
/// Wrap u_savesub(lnum). Returns 1 on OK, 0 on FAIL.
int nvim_u_savesub(int lnum) { return u_savesub((linenr_T)lnum) == OK ? 1 : 0; }
/// Wrap u_savedel(lnum, count). Returns 1 on OK, 0 on FAIL.
int nvim_u_savedel2(int lnum, int count)
{
  return u_savedel((linenr_T)lnum, (linenr_T)count) == OK ? 1 : 0;
}
