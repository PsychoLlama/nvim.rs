#pragma once

#include <stddef.h>
#include <stdint.h>
#include <time.h>

#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep
#include "nvim/undo_defs.h"  // IWYU pragma: keep

// Functions implemented in Rust (nvim-undo crate), exported directly.
void f_undotree(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
int u_save_cursor(void);
int u_save(linenr_T top, linenr_T bot);
int u_save_buf(buf_T *buf, linenr_T top, linenr_T bot);
int u_savesub(linenr_T lnum);
int u_inssub(linenr_T lnum);
int u_savedel(linenr_T lnum, linenr_T nlines);
bool undo_allowed(buf_T *buf);
int u_savecommon(buf_T *buf, linenr_T top, linenr_T bot, linenr_T newbot, bool reload);
void u_compute_hash(buf_T *buf, uint8_t *hash);
void u_write_undo(const char *name, bool forceit, buf_T *buf, uint8_t *hash);
void u_read_undo(char *name, const uint8_t *hash, const char *orig_name);
void u_undo(int count);
void u_redo(int count);
bool u_undo_and_forget(int count, bool do_buf_event);
void undo_time(int step, bool sec, bool file, bool absolute);
void undo_fmt_time(char *buf, size_t buflen, time_t tt);
void u_sync(bool force);
void ex_undolist(exarg_T *eap);
void ex_undojoin(exarg_T *eap);
void u_unchanged(buf_T *buf);
void u_find_first_changed(void);
void u_update_save_nr(buf_T *buf);
void u_clearall(buf_T *buf);
void u_blockfree(buf_T *buf);
void u_clearallandblockfree(buf_T *buf);
void u_clearline(buf_T *buf);
void u_undoline(void);
bool bufIsChanged(buf_T *buf);
bool anyBufIsChanged(void);
bool curbufIsChanged(void);
u_header_T *u_force_get_undo_header(buf_T *buf);

#include "undo.h.generated.h"
