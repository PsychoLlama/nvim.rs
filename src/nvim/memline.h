#pragma once

#include "nvim/ascii_defs.h"
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/memline_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
int ml_open(buf_T *buf);
char *ml_get_pos(const pos_T *pos);
colnr_T ml_get_len(linenr_T lnum);
colnr_T ml_get_pos_len(pos_T *pos);
colnr_T ml_get_buf_len(buf_T *buf, linenr_T lnum);
int gchar_pos(pos_T *pos);
int ml_append_flags(linenr_T lnum, char *line, colnr_T len, int flags);
int ml_append_buf(buf_T *buf, linenr_T lnum, char *line, colnr_T len, bool newfile);
void ml_add_deleted_len_buf(buf_T *buf, char *ptr, ssize_t len);
int ml_replace_buf(buf_T *buf, linenr_T lnum, char *line, bool copy, bool noalloc);
int ml_replace_buf_len(buf_T *buf, linenr_T lnum, char *line_arg, size_t len_arg, bool copy,
                       bool noalloc);
int ml_delete_buf(buf_T *buf, linenr_T lnum, bool message);
int ml_delete_flags(linenr_T lnum, int flags);
size_t ml_flush_deleted_bytes(buf_T *buf, size_t *codepoints, size_t *codeunits);
void ml_flush_line(buf_T *buf, bool noalloc);
char *makeswapname(char *fname, char *ffname, buf_T *buf, char *dir_name);
char *get_file_in_dir(char *fname, char *dname);
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
char *make_percent_swname(char *dir, char *dir_end, const char *name);
void swapfile_dict(const char *fname, dict_T *d);
void ml_sync_all(int check_file, int check_char, bool do_fsync);
void ml_preserve(buf_T *buf, bool message, bool do_fsync);

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
