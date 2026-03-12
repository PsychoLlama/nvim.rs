#pragma once

#include "nvim/ascii_defs.h"
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/memline_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
int ml_open(buf_T *buf);
linenr_T ml_firstmarked(void);
#if defined(HAVE_READLINK)
int resolve_symlink(const char *fname, char *buf);
#endif
int inc(pos_T *lp);
int incl(pos_T *lp);
int dec(pos_T *lp);
int decl(pos_T *lp);
void ml_setname(buf_T *buf);
void ml_open_files(void);
void ml_open_file(buf_T *buf);
void check_need_swap(bool newfile);
void ml_close(buf_T *buf, int del_file);
void ml_close_all(bool del_file);
void ml_close_notmod(void);
void ml_timestamp(buf_T *buf);
void ml_recover(bool checkext);
void ml_add_deleted_len(char *ptr, ssize_t len);
void ml_setmarked(linenr_T lnum);
void ml_clearmarked(void);
void ml_setflags(buf_T *buf);

#include "memline_shim.h.generated.h"

/// LINEEMPTY() - return true if the line is empty
#define LINEEMPTY(p) (*ml_get(p) == NUL)

// Values for the flags argument of ml_delete_flags().
enum {
  ML_DEL_MESSAGE = 1,  // may give a "No lines in buffer" message
  // ML_DEL_UNDO = 2,  // called from undo
};

// Values for the flags argument of ml_append_int().
enum {
  ML_APPEND_NEW = 1,   // starting to edit a new file
  ML_APPEND_MARK = 2,  // mark the new line
  // ML_APPEND_UNDO = 4,  // called from undo
};
