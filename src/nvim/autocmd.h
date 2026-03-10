#pragma once

#include <stdbool.h>
#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>  // IWYU pragma: keep

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/autocmd_defs.h"  // IWYU pragma: keep
#include "nvim/buffer_defs.h"
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/macros_defs.h"
#include "nvim/pos_defs.h"
#include "nvim/types_defs.h"

/// For CursorMoved event
EXTERN win_T *last_cursormoved_win INIT( = NULL);
/// For CursorMoved event, only used when last_cursormoved_win == curwin
EXTERN pos_T last_cursormoved INIT( = { 0, 0, 0 });

EXTERN bool autocmd_busy INIT( = false);     ///< Is apply_autocmds() busy?
EXTERN int autocmd_no_enter INIT( = false);  ///< Buf/WinEnter autocmds disabled
EXTERN int autocmd_no_leave INIT( = false);  ///< Buf/WinLeave autocmds disabled

/// When deleting the current buffer, another one must be loaded.
/// If we know which one is preferred, au_new_curbuf is set to it.
EXTERN bufref_T au_new_curbuf INIT( = { NULL, 0, 0 });

// When deleting a buffer/window and autocmd_busy is true, do not free the
// buffer/window. but link it in the list starting with
// au_pending_free_buf/ap_pending_free_win, using b_next/w_next.
// Free the buffer/window when autocmd_busy is being set to false.
EXTERN buf_T *au_pending_free_buf INIT( = NULL);
EXTERN win_T *au_pending_free_win INIT( = NULL);

EXTERN char *autocmd_fname INIT( = NULL);       ///< fname for <afile> on cmdline
EXTERN bool autocmd_fname_full INIT( = false);  ///< autocmd_fname is full path
EXTERN int autocmd_bufnr INIT( = 0);            ///< fnum for <abuf> on cmdline
EXTERN char *autocmd_match INIT( = NULL);       ///< name for <amatch> on cmdline
EXTERN bool did_cursorhold INIT( = true);       ///< set when CursorHold t'gerd

typedef struct {
  win_T *auc_win;     ///< Window used in aucmd_prepbuf().  When not NULL the
                      ///< window has been allocated.
  bool auc_win_used;  ///< This auc_win is being used.
} aucmdwin_T;

/// When executing autocommands for a buffer that is not in any window, a
/// special window is created to handle the side effects.  When autocommands
/// nest we may need more than one.
EXTERN kvec_t(aucmdwin_T) aucmd_win_vec INIT( = KV_INITIAL_VALUE);
#define aucmd_win (aucmd_win_vec.items)
#define AUCMD_WIN_COUNT ((int)aucmd_win_vec.size)

enum {
  AUGROUP_DEFAULT = -1,  ///< default autocmd group
  AUGROUP_ERROR   = -2,  ///< erroneous autocmd group
  AUGROUP_ALL     = -3,  ///< all autocmd groups
  AUGROUP_DELETED = -4,  ///< all autocmd groups
  // AUGROUP_NS      = -5,  // TODO(tjdevries): Support namespaced based augroups
};

enum { BUFLOCAL_PAT_LEN = 25, };

/// Iterates over all the events for auto commands
#define FOR_ALL_AUEVENTS(event) \
  for (event_T event = (event_T)0; (int)event < (int)NUM_EVENTS; event = (event_T)((int)event + 1))

#include "autocmd.h.generated.h"

// Declarations for functions now implemented in Rust (nvim-autocmd crate).
// These replace the C thin wrappers that were deleted in Phase 1.
void block_autocmds(void);
void unblock_autocmds(void);
bool apply_autocmds(event_T event, char *fname, char *fname_io, bool force, buf_T *buf);
bool apply_autocmds_exarg(event_T event, char *fname, char *fname_io, bool force, buf_T *buf,
                          exarg_T *eap);
bool apply_autocmds_retval(event_T event, char *fname, char *fname_io, bool force, buf_T *buf,
                            int *retval);
int do_doautocmd(char *arg, bool do_msg, bool *did_something);
int check_ei(const char *ei);
char *au_event_disable(const char *what);
void au_event_restore(char *old_ei);
bool check_nomodeline(char **argp);
bool au_exists(const char *const arg);
void do_autocmd(exarg_T *eap, char *arg, int forceit);
void do_augroup(char *arg, int del_group);
int augroup_find(const char *name);
int augroup_add(const char *name);
const char *augroup_name(int id);
void au_cleanup(void);
size_t aucmd_pattern_length(const char *pat);
const char *aucmd_next_pattern(const char *pat, size_t patlen);
int aupat_get_buflocal_nr(const char *pat, int patlen);
void aupat_normalize_buflocal_pat(char *dest, const char *pat, int patlen, int buflocal_nr);
void aucmd_del_for_event_and_group(int event, int group);
event_T event_name2nr_str(String str);
int has_event(event_T event);
int is_autocmd_blocked(void);
int trigger_cursorhold(void);
int has_cursorhold(void);
void do_autocmd_focusgained(bool gained);
