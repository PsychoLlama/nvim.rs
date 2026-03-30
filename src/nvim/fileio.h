#pragma once

#include <stdint.h>  // IWYU pragma: keep
#include <stdio.h>  // IWYU pragma: keep
#include <time.h>  // IWYU pragma: keep

#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/os/fs_defs.h"  // IWYU pragma: keep
#include "nvim/os/os_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Values for readfile() flags
enum {
  READ_NEW        = 0x01,   ///< read a file into a new buffer
  READ_FILTER     = 0x02,   ///< read filter output
  READ_STDIN      = 0x04,   ///< read from stdin
  READ_BUFFER     = 0x08,   ///< read from curbuf (converting stdin)
  READ_DUMMY      = 0x10,   ///< reading into a dummy buffer
  READ_KEEP_UNDO  = 0x20,   ///< keep undo info
  READ_FIFO       = 0x40,   ///< read from fifo or socket
  READ_NOWINENTER = 0x80,   ///< do not trigger BufWinEnter
  READ_NOFILE     = 0x100,  ///< do not read a file, do trigger BufReadCmd
};

typedef varnumber_T (*CheckItem)(void *expr, const char *name);

enum {
  FIO_LATIN1 = 0x01,       ///< convert Latin1
  FIO_UTF8 = 0x02,         ///< convert UTF-8
  FIO_UCS2 = 0x04,         ///< convert UCS-2
  FIO_UCS4 = 0x08,         ///< convert UCS-4
  FIO_UTF16 = 0x10,        ///< convert UTF-16
  FIO_ENDIAN_L = 0x80,     ///< little endian
  FIO_NOCONVERT = 0x2000,  ///< skip encoding conversion
  FIO_UCSBOM = 0x4000,     ///< check for BOM at start of file
  FIO_ALL = -1,            ///< allow all formats
};

enum {
  /// When converting, a read() or write() may leave some bytes to be converted
  /// for the next call.  The value is guessed...
  CONV_RESTLEN = 30,
};

enum { WRITEBUFSIZE = 8192, };  ///< size of normal write buffer

enum {
  /// We have to guess how much a sequence of bytes may expand when converting
  /// with iconv() to be able to allocate a buffer.
  ICONV_MULT = 8,
};

// Forward declarations for Rust-implemented functions (exported under C names via #[export_name])
char *modname(const char *fname, const char *ext, bool prepend_dot);
bool match_file_pat(char *pattern, regprog_T **prog, char *fname, char *sfname, char *tail,
                    int allow_dirs);
bool match_file_list(char *list, char *sfname, char *ffname);
char *file_pat_to_reg_pat(const char *pat, const char *pat_end, char *allow_dirs, int no_bslash);
bool vim_fgets(char *buf, int size, FILE *fp);
int get2c(FILE *fd);
int get3c(FILE *fd);
int get4c(FILE *fd);
time_t get8ctime(FILE *fd);
char *read_string(FILE *fd, size_t cnt);
bool put_bytes(FILE *fd, uintmax_t number, size_t len);
int put_time(FILE *fd, time_t time_);
int vim_rename(const char *from, const char *to);
int vim_copyfile(const char *from, const char *to);

// Phase 1 migrations: simple utility functions replaced by Rust
int read_eintr(int fd, void *buf, size_t bufsize);
int write_eintr(int fd, void *buf, size_t bufsize);
void add_quoted_fname(char *ret_buf, size_t buf_len, const buf_T *buf, const char *fname);
bool msg_add_fileformat(int eol_type);
void msg_add_lines(int insert_space, linenr_T lnum, off_T nchars);
void write_lnum_adjust(linenr_T offset);
linenr_T readfile_linenr(linenr_T linecnt, char *p, const char *endp);

// Phase 2 migrations: tempdir subsystem replaced by Rust
void vim_deltempdir(void);
char *vim_gettempdir(void);
char *vim_tempname(void);

// Phase 5 migrations: utility functions replaced by Rust
void filemess(buf_T *buf, char *name, char *s);
int readdir_core(garray_T *gap, const char *path, void *context, CheckItem checkitem);
int delete_recursive(const char *name);

// Phase 6 migrations: wrapper functions replaced by Rust (exported via #[export_name])
void prep_exarg(exarg_T *eap, const buf_T *buf);
void set_file_options(int set_options, exarg_T *eap);
void set_forced_fenc(exarg_T *eap);
int set_rw_fname(char *fname, char *sfname);
void shorten_buf_fname(buf_T *buf, char *dirname, int force);
void shorten_fnames(int force);
int check_timestamps(int focus);
int buf_check_timestamp(buf_T *buf);
void buf_reload(buf_T *buf, int orig_mode, int reload_options);
void buf_store_file_info(buf_T *buf, FileInfo *file_info);
void forward_slash(char *fname);

#include "fileio.h.generated.h"
