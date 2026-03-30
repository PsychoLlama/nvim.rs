// fileio.c: read from and write to a file

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <iconv.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <uv.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/cursor.h"
#include "nvim/diff.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_eval.h"
#include "nvim/fileio.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/iconv_defs.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memfile.h"
#include "nvim/memfile_defs.h"
#include "nvim/memline.h"
#include "nvim/memline_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/sha256.h"
#include "nvim/shada.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/undo_defs.h"
#include "nvim/vim_defs.h"

#ifdef BACKSLASH_IN_FILENAME
# include "nvim/charset.h"
#endif

#ifdef HAVE_DIRFD_AND_FLOCK
# include <dirent.h>
# include <sys/file.h>
#endif

#ifdef OPEN_CHR_FILES
# include "nvim/charset.h"
#endif

// For compatibility with libuv < 1.20.0 (tested on 1.18.0)
#ifndef UV_FS_COPYFILE_FICLONE
# define UV_FS_COPYFILE_FICLONE 0
#endif

// Rust implementations - declarations
extern int rs_time_differs(int64_t file_sec, int64_t file_nsec, int64_t mtime, int64_t mtime_ns,
                           int fat_tolerance);
extern bool rs_is_dev_fd_file(const char *fname);
extern const char *rs_check_for_bom(const uint8_t *data, int size, int *lenp, int flags);
extern bool rs_need_conversion(const char *fenc);
extern int rs_get_fio_flags(const char *name);
extern void rs_check_marks_read(void);
extern void rs_diff_invalidate(buf_T *buf);
// next_fenc and readfile_charconvert are implemented in Rust (fileio crate).
extern char *next_fenc(char **pp, bool *alloced);
extern char *readfile_charconvert(char *fname, char *fenc, int *fdp);
// Phase 1 Rust replacements
extern void rs_buf_store_file_info(buf_T *buf, FileInfo *file_info);
extern void rs_forward_slash(char *fname);
extern void rs_prep_exarg(exarg_T *eap, const buf_T *buf);
extern void rs_set_forced_fenc(exarg_T *eap);
// Phase 2 Rust replacements
extern void rs_set_file_options(int set_options, exarg_T *eap);
extern int rs_set_rw_fname(char *fname, char *sfname);
// Phase 3 Rust replacements
extern void rs_shorten_buf_fname(buf_T *buf, char *dirname, int force);
extern void rs_shorten_fnames(int force);
// Phase 4 Rust replacements
extern int rs_check_timestamps(int focus);
// Phase 5 Rust replacements
extern int rs_buf_check_timestamp(buf_T *buf);
// Phase 6 Rust replacements
extern void rs_buf_reload(buf_T *buf, int orig_mode, int reload_options);
// Phase 1-5 readfile migration
extern int rs_readfile(char *fname, char *sfname, linenr_T from, linenr_T lines_to_skip,
                       linenr_T lines_to_read, exarg_T *eap, int flags, int silent);

#include "fileio.c.generated.h"

// Rust fold FFI declaration
extern void rs_foldUpdateAll(win_T *win);

extern int rs_default_fileformat(void);
extern int rs_get_fileformat(buf_T *buf);

static const char *e_auchangedbuf = N_("E812: Autocommands changed buffer or buffer name");

/// Read lines from file "fname" into the buffer after line "from".
///
/// 1. We allocate blocks with try_malloc, as big as possible.
/// 2. Each block is filled with characters from the file with a single read().
/// 3. The lines are inserted in the buffer with ml_append().
///
/// (caller must check that fname != NULL, unless READ_STDIN is used)
///
/// "lines_to_skip" is the number of lines that must be skipped
/// "lines_to_read" is the number of lines that are appended
/// When not recovering lines_to_skip is 0 and lines_to_read MAXLNUM.
///
/// flags:
/// READ_NEW     starting to edit a new buffer
/// READ_FILTER  reading filter output
/// READ_STDIN   read from stdin instead of a file
/// READ_BUFFER  read from curbuf instead of a file (converting after reading
///              stdin)
/// READ_NOFILE  do not read a file, only trigger BufReadCmd
/// READ_DUMMY   read into a dummy buffer (to check if file contents changed)
/// READ_KEEP_UNDO  don't clear undo info or read it from a file
/// READ_FIFO    read from fifo/socket instead of a file
///
/// @param eap  can be NULL!
///
/// @return     FAIL for failure, NOTDONE for directory (failure), or OK
int readfile(char *fname, char *sfname, linenr_T from, linenr_T lines_to_skip,
             linenr_T lines_to_read, exarg_T *eap, int flags, bool silent)
{
  return rs_readfile(fname, sfname, from, lines_to_skip, lines_to_read, eap, flags, (int)silent);
}



/// Fill "*eap" to force the 'fileencoding', 'fileformat' and 'binary' to be
/// equal to the buffer "buf".  Used for calling readfile().
void prep_exarg(exarg_T *eap, const buf_T *buf)
  FUNC_ATTR_NONNULL_ALL
{
  rs_prep_exarg(eap, buf);
}

/// Set default or forced 'fileformat' and 'binary'.
void set_file_options(bool set_options, exarg_T *eap)
{
  rs_set_file_options((int)set_options, eap);
}

/// Set forced 'fileencoding'.
void set_forced_fenc(exarg_T *eap)
{
  rs_set_forced_fenc(eap);
}


/// Set the name of the current buffer.  Use when the buffer doesn't have a
/// name and a ":r" or ":w" command with a file name is used.
int set_rw_fname(char *fname, char *sfname)
{
  return rs_set_rw_fname(fname, sfname);
}








/// Shorten filename of a buffer.
///
/// @param force  when true: Use full path from now on for files currently being
///               edited, both for file name and swap file name.  Try to shorten the file
///               names a bit, if safe to do so.
///               when false: Only try to shorten absolute file names.
///
/// For buffers that have buftype "nofile" or "scratch": never change the file
/// name.
void shorten_buf_fname(buf_T *buf, char *dirname, int force)
{
  rs_shorten_buf_fname(buf, dirname, force);
}

/// Shorten filenames for all buffers.
void shorten_fnames(int force)
{
  rs_shorten_fnames(force);
}


/// Check if any not hidden buffer has been changed.
/// Postpone the check if there are characters in the stuff buffer, a global
/// command is being executed, a mapping is being executed or an autocommand is
/// busy.
///
/// @param focus  called for GUI focus event
///
/// @return       true if some message was written (screen should be redrawn and cursor positioned).
int check_timestamps(int focus)
{
  return rs_check_timestamps(focus);
}


/// Check if buffer "buf" has been changed.
/// Also check if the file for a new buffer unexpectedly appeared.
///
/// @return  1 if a changed buffer was found or,
///          2 if a message has been displayed or,
///          0 otherwise.
int buf_check_timestamp(buf_T *buf)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_buf_check_timestamp(buf);
}


/// Reload a buffer that is already loaded.
/// Used when the file was changed outside of Vim.
/// "orig_mode" is buf->b_orig_mode before the need for reloading was detected.
/// buf->b_orig_mode may have been reset already.
void buf_reload(buf_T *buf, int orig_mode, bool reload_options)
{
  rs_buf_reload(buf, orig_mode, (int)reload_options);
}

void buf_store_file_info(buf_T *buf, FileInfo *file_info)
  FUNC_ATTR_NONNULL_ALL
{
  rs_buf_store_file_info(buf, file_info);
}


#if defined(BACKSLASH_IN_FILENAME)
/// Convert all backslashes in fname to forward slashes in-place,
/// unless when it looks like a URL.
void forward_slash(char *fname)
{
  rs_forward_slash(fname);
}
#endif

