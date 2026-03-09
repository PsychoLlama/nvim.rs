#pragma once

#include <stdint.h>

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/gettext_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/marktree_defs.h"
#include "nvim/types_defs.h"

/// Values for buflist_getfile()
enum getf_values {
  GETF_SETMARK = 0x01,  ///< set pcmark before jumping
  GETF_ALT     = 0x02,  ///< jumping to alternate file (not buf num)
  GETF_SWITCH  = 0x04,  ///< respect 'switchbuf' settings when jumping
};

// Return values of getfile()
enum getf_retvalues {
  GETFILE_ERROR       = 1,   ///< normal error
  GETFILE_NOT_WRITTEN = 2,   ///< "not written" error
  GETFILE_SAME_FILE   = 0,   ///< success, same file
  GETFILE_OPEN_OTHER  = -1,  ///< success, opened another file
  GETFILE_UNUSED      = 8,
};

/// Values for buflist_new() flags
enum bln_values {
  BLN_CURBUF = 1,   ///< May re-use curbuf for new buffer
  BLN_LISTED = 2,   ///< Put new buffer in buffer list
  BLN_DUMMY  = 4,   ///< Allocating dummy buffer
  BLN_NEW    = 8,   ///< create a new buffer
  BLN_NOOPT  = 16,  ///< Don't copy options to existing buffer
  // BLN_DUMMY_OK = 32,  // also find an existing dummy buffer
  // BLN_REUSE = 64,   // may re-use number from buf_reuse
  BLN_NOCURWIN = 128,  ///< buffer is not associated with curwin
};

/// Values for action argument for do_buffer_ext() and close_buffer()
enum dobuf_action_values {
  DOBUF_GOTO   = 0,  ///< go to specified buffer
  DOBUF_SPLIT  = 1,  ///< split window and go to specified buffer
  DOBUF_UNLOAD = 2,  ///< unload specified buffer(s)
  DOBUF_DEL    = 3,  ///< delete specified buffer(s) from buflist
  DOBUF_WIPE   = 4,  ///< delete specified buffer(s) really
};

/// Values for start argument for do_buffer_ext()
enum dobuf_start_values {
  DOBUF_CURRENT = 0,  ///< "count" buffer from current buffer
  DOBUF_FIRST   = 1,  ///< "count" buffer from first buffer
  DOBUF_LAST    = 2,  ///< "count" buffer from last buffer
  DOBUF_MOD     = 3,  ///< "count" mod. buffer from current buffer
};

/// Values for flags argument of do_buffer_ext()
enum dobuf_flags_value {
  DOBUF_FORCEIT  = 1,  ///< :cmd!
  DOBUF_SKIPHELP = 4,  ///< skip or keep help buffers depending on b_help of the
                       ///< starting buffer
};

/// flags for buf_freeall()
enum bfa_values {
  BFA_DEL          = 1,  ///< buffer is going to be deleted
  BFA_WIPE         = 2,  ///< buffer is going to be wiped out
  BFA_KEEP_UNDO    = 4,  ///< do not free undo information
  BFA_IGNORE_ABORT = 8,  ///< do not abort for aborting()
};

EXTERN char *msg_loclist INIT( = N_("[Location List]"));
EXTERN char *msg_qflist INIT( = N_("[Quickfix List]"));

#include "buffer.h.generated.h"
#include "buffer.h.inline.generated.h"

// Rust FFI declarations for buffer functions (now implemented in Rust).
// The rs_* symbols are exported from libnvim_rs.a; the C-named inline
// wrappers below provide backward-compatible access from C call sites.
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// Forward-declare the Rust rs_* symbols used by the inline wrappers.
extern int rs_buf_valid(buf_T *buf);
extern int rs_bufref_valid(bufref_T *bufref);
extern int rs_buf_is_empty(buf_T *buf);
extern int rs_buf_hide(buf_T *buf);
extern bool rs_bt_prompt(buf_T *buf);
extern bool rs_bt_help(buf_T *buf);
extern bool rs_bt_normal(buf_T *buf);
extern bool rs_bt_quickfix(buf_T *buf);
extern bool rs_bt_terminal(buf_T *buf);
extern bool rs_bt_nofilename(buf_T *buf);
extern bool rs_bt_nofile(buf_T *buf);
extern bool rs_bt_dontwrite(buf_T *buf);
extern bool rs_bt_dontwrite_msg(buf_T *buf);
extern bool rs_curbuf_reusable(void);
extern bool rs_otherfile(char *ffname);
extern int rs_get_fileformat(buf_T *buf);
extern int rs_get_highest_fnum(void);
extern int rs_calc_percentage(int64_t part, int64_t whole);
extern int rs_col_print(uint8_t *buf, size_t buflen, int col, int vcol);
extern int rs_get_rel_pos(win_T *wp, char *buf, int buflen);
extern int rs_append_arg_number(win_T *wp, char *buf, size_t buflen);
extern int rs_buflist_name_nr(int fnum, char **fname, linenr_T *lnum);
extern int rs_buflist_add(char *fname, int flags);
extern char *rs_buf_spname(buf_T *buf);
extern char *rs_buf_get_fname(buf_T *buf);
extern char *rs_getaltfname(bool errmsg);
extern char *rs_buflist_nr2name(int n, int fullname, int helptail);
extern buf_T *rs_buflist_findnr(int nr);
extern buf_T *rs_buflist_findname(char *ffname);
extern buf_T *rs_buflist_findname_exp(char *fname);
extern int rs_buflist_findlnum(buf_T *buf);
extern void rs_set_buflisted(int on);
extern void rs_buf_clear_file(buf_T *buf);
extern void rs_fileinfo(int fullname, bool shorthelp, bool dont_truncate);
extern void rs_buf_inc_changedtick(buf_T *buf);
extern void rs_wipe_buffer(buf_T *buf, bool aucmd);
extern void rs_buf_set_file_id(buf_T *buf);
extern void rs_fname_expand(buf_T *buf, char **ffname, char **sfname);
extern void rs_buflist_altfpos(win_T *win);

// Inline wrappers: these replace the C wrapper functions deleted from buffer.c
static inline bool buf_valid(buf_T *buf) { return rs_buf_valid(buf) != 0; }
static inline bool bufref_valid(bufref_T *bufref) { return rs_bufref_valid(bufref) != 0; }
static inline bool buf_is_empty(buf_T *buf) { return rs_buf_is_empty(buf) != 0; }
static inline bool buf_hide(const buf_T *buf) { return rs_buf_hide((buf_T *)buf) != 0; }
static inline bool bt_prompt(buf_T *buf) { return rs_bt_prompt(buf); }
static inline bool bt_help(const buf_T *buf) { return rs_bt_help((buf_T *)buf); }
static inline bool bt_normal(const buf_T *buf) { return rs_bt_normal((buf_T *)buf); }
static inline bool bt_quickfix(const buf_T *buf) { return rs_bt_quickfix((buf_T *)buf); }
static inline bool bt_terminal(const buf_T *buf) { return rs_bt_terminal((buf_T *)buf); }
static inline bool bt_nofilename(const buf_T *buf) { return rs_bt_nofilename((buf_T *)buf); }
static inline bool bt_nofile(const buf_T *buf) { return rs_bt_nofile((buf_T *)buf); }
static inline bool bt_dontwrite(const buf_T *buf) { return rs_bt_dontwrite((buf_T *)buf); }
static inline bool bt_dontwrite_msg(const buf_T *buf) { return rs_bt_dontwrite_msg((buf_T *)buf); }
static inline bool curbuf_reusable(void) { return rs_curbuf_reusable(); }
static inline bool otherfile(char *ffname) { return rs_otherfile(ffname); }
static inline int get_fileformat(buf_T *buf) { return rs_get_fileformat(buf); }
static inline int get_highest_fnum(void) { return rs_get_highest_fnum(); }
static inline int calc_percentage(int64_t part, int64_t whole) { return rs_calc_percentage(part, whole); }
static inline int col_print(char *buf, size_t buflen, int col, int vcol) { return rs_col_print((uint8_t *)buf, buflen, col, vcol); }
static inline int get_rel_pos(win_T *wp, char *buf, int buflen) { return rs_get_rel_pos(wp, buf, buflen); }
static inline int append_arg_number(win_T *wp, char *buf, size_t buflen) { return rs_append_arg_number(wp, buf, buflen); }
static inline int buflist_name_nr(int fnum, char **fname, linenr_T *lnum) { return rs_buflist_name_nr(fnum, fname, lnum); }
static inline int buflist_add(char *fname, int flags) { return rs_buflist_add(fname, flags); }
static inline char *buf_spname(buf_T *buf) { return rs_buf_spname(buf); }
static inline char *buf_get_fname(const buf_T *buf) { return rs_buf_get_fname((buf_T *)buf); }
static inline char *getaltfname(bool errmsg) { return rs_getaltfname(errmsg); }
static inline char *buflist_nr2name(int n, int fullname, int helptail) { return rs_buflist_nr2name(n, fullname, helptail); }
static inline buf_T *buflist_findnr(int nr) { return rs_buflist_findnr(nr); }
static inline buf_T *buflist_findname(char *ffname) { return rs_buflist_findname(ffname); }
static inline buf_T *buflist_findname_exp(char *fname) { return rs_buflist_findname_exp(fname); }
static inline linenr_T buflist_findlnum(buf_T *buf) { return (linenr_T)rs_buflist_findlnum(buf); }
static inline void set_buflisted(int on) { rs_set_buflisted(on); }
static inline void buf_clear_file(buf_T *buf) { rs_buf_clear_file(buf); }
static inline void fileinfo(int fullname, bool shorthelp, bool dont_truncate) { rs_fileinfo(fullname, shorthelp, dont_truncate); }
static inline void buf_inc_changedtick(buf_T *buf) { rs_buf_inc_changedtick(buf); }
static inline void wipe_buffer(buf_T *buf, bool aucmd) { rs_wipe_buffer(buf, aucmd); }
static inline void buf_set_file_id(buf_T *buf) { rs_buf_set_file_id(buf); }
static inline void fname_expand(buf_T *buf, char **ffname, char **sfname) { rs_fname_expand(buf, ffname, sfname); }
static inline void buflist_altfpos(win_T *win) { rs_buflist_altfpos(win); }

/// Get b:changedtick value
///
/// Faster then querying b:.
///
/// @param[in]  buf  Buffer to get b:changedtick from.
static inline varnumber_T buf_get_changedtick(const buf_T *const buf)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_ALWAYS_INLINE FUNC_ATTR_PURE
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return buf->changedtick_di.di_tv.vval.v_number;
}

static inline uint32_t buf_meta_total(const buf_T *b, MetaIndex m)
{
  return b->b_marktree->meta_root[m];
}
