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
  int retval = FAIL;  // jump to "theend" instead of returning
  int fd = stdin_fd >= 0 ? stdin_fd : 0;
  bool newfile = (flags & READ_NEW);
  bool filtering = (flags & READ_FILTER);
  bool read_stdin = (flags & READ_STDIN);
  bool read_buffer = (flags & READ_BUFFER);
  bool read_fifo = (flags & READ_FIFO);
  bool set_options = newfile || read_buffer || (eap != NULL && eap->read_edit);
  linenr_T read_buf_lnum = 1;           // next line to read from curbuf
  colnr_T read_buf_col = 0;             // next char to read from this line
  char c;
  linenr_T lnum = from;
  char *ptr = NULL;              // pointer into read buffer
  char *buffer = NULL;           // read buffer
  char *new_buffer = NULL;       // init to shut up gcc
  char *line_start = NULL;       // init to shut up gcc
  int wasempty;                         // buffer was empty before reading
  colnr_T len;
  ptrdiff_t size = 0;
  uint8_t *p = NULL;
  off_T filesize = 0;
  bool skip_read = false;
  context_sha256_T sha_ctx;
  bool read_undo_file = false;
  int split = 0;  // number of split lines
  linenr_T linecnt;
  bool error = false;                   // errors encountered
  int ff_error = EOL_UNKNOWN;           // file format with errors
  ptrdiff_t linerest = 0;               // remaining chars in line
  int perm = 0;
#ifdef UNIX
  int swap_mode = -1;                   // protection bits for swap file
#endif
  int fileformat = 0;                   // end-of-line format
  bool keep_fileformat = false;
  FileInfo file_info;
  linenr_T skip_count = 0;
  linenr_T read_count = 0;
  int msg_save = msg_scroll;
  linenr_T read_no_eol_lnum = 0;        // non-zero lnum when last line of
                                        // last read was missing the eol
  bool file_rewind = false;
  linenr_T conv_error = 0;              // line nr with conversion error
  linenr_T illegal_byte = 0;            // line nr with illegal byte
  bool keep_dest_enc = false;           // don't retry when char doesn't fit
                                        // in destination encoding
  int bad_char_behavior = BAD_REPLACE;
  // BAD_KEEP, BAD_DROP or character to
  // replace with
  char *tmpname = NULL;          // name of 'charconvert' output file
  int fio_flags = 0;
  char *fenc;                    // fileencoding to use
  bool fenc_alloced;                    // fenc_next is in allocated memory
  char *fenc_next = NULL;        // next item in 'fencs' or NULL
  bool advance_fenc = false;
  int real_size = 0;
  iconv_t iconv_fd = (iconv_t)-1;       // descriptor for iconv() or -1
  bool did_iconv = false;               // true when iconv() failed and trying
                                        // 'charconvert' next
  bool converted = false;                // true if conversion done
  bool notconverted = false;             // true if conversion wanted but it wasn't possible
  char conv_rest[CONV_RESTLEN];
  int conv_restlen = 0;                 // nr of bytes in conv_rest[]
  pos_T orig_start;
  buf_T *old_curbuf;
  char *old_b_ffname;
  char *old_b_fname;
  int using_b_ffname;
  int using_b_fname;
  static char *msg_is_a_directory = N_("is a directory");

  curbuf->b_au_did_filetype = false;  // reset before triggering any autocommands

  curbuf->b_no_eol_lnum = 0;    // in case it was set by the previous read

  // If there is no file name yet, use the one for the read file.
  // BF_NOTEDITED is set to reflect this.
  // Don't do this for a read from a filter.
  // Only do this when 'cpoptions' contains the 'f' flag.
  if (curbuf->b_ffname == NULL
      && !filtering
      && fname != NULL
      && vim_strchr(p_cpo, CPO_FNAMER) != NULL
      && !(flags & READ_DUMMY)) {
    if (set_rw_fname(fname, sfname) == FAIL) {
      goto theend;
    }
  }

  // Remember the initial values of curbuf, curbuf->b_ffname and
  // curbuf->b_fname to detect whether they are altered as a result of
  // executing nasty autocommands.  Also check if "fname" and "sfname"
  // point to one of these values.
  old_curbuf = curbuf;
  old_b_ffname = curbuf->b_ffname;
  old_b_fname = curbuf->b_fname;
  using_b_ffname = (fname == curbuf->b_ffname) || (sfname == curbuf->b_ffname);
  using_b_fname = (fname == curbuf->b_fname) || (sfname == curbuf->b_fname);

  // After reading a file the cursor line changes but we don't want to
  // display the line.
  ex_no_reprint = true;

  // don't display the file info for another buffer now
  need_fileinfo = false;

  // For Unix: Use the short file name whenever possible.
  // Avoids problems with networks and when directory names are changed.
  // Don't do this for Windows, a "cd" in a sub-shell may have moved us to
  // another directory, which we don't detect.
  if (sfname == NULL) {
    sfname = fname;
  }
#if defined(UNIX)
  fname = sfname;
#endif

  // The BufReadCmd and FileReadCmd events intercept the reading process by
  // executing the associated commands instead.
  if (!filtering && !read_stdin && !read_buffer) {
    orig_start = curbuf->b_op_start;

    // Set '[ mark to the line above where the lines go (line 1 if zero).
    curbuf->b_op_start.lnum = ((from == 0) ? 1 : from);
    curbuf->b_op_start.col = 0;

    if (newfile) {
      if (apply_autocmds_exarg(EVENT_BUFREADCMD, NULL, sfname,
                               false, curbuf, eap)) {
        retval = OK;
        if (aborting()) {
          retval = FAIL;
        }

        // The BufReadCmd code usually uses ":read" to get the text and
        // perhaps ":file" to change the buffer name. But we should
        // consider this to work like ":edit", thus reset the
        // BF_NOTEDITED flag.  Then ":write" will work to overwrite the
        // same file.
        if (retval == OK) {
          curbuf->b_flags &= ~BF_NOTEDITED;
        }
        goto theend;
      }
    } else if (apply_autocmds_exarg(EVENT_FILEREADCMD, sfname, sfname,
                                    false, NULL, eap)) {
      retval = aborting() ? FAIL : OK;
      goto theend;
    }

    curbuf->b_op_start = orig_start;

    if (flags & READ_NOFILE) {
      // Return NOTDONE instead of FAIL so that BufEnter can be triggered
      // and other operations don't fail.
      retval = NOTDONE;
      goto theend;
    }
  }

  if (((shortmess(SHM_OVER) && !msg_listdo_overwrite) || curbuf->b_help) && p_verbose == 0) {
    msg_scroll = false;         // overwrite previous file message
  } else {
    msg_scroll = true;          // don't overwrite previous file message
  }
  // If the name is too long we might crash further on, quit here.
  if (fname != NULL && *fname != NUL) {
    size_t namelen = strlen(fname);

    // If the name is too long we might crash further on, quit here.
    if (namelen >= MAXPATHL) {
      filemess(curbuf, fname, _("Illegal file name"));
      msg_end();
      msg_scroll = msg_save;
      goto theend;
    }

    // If the name ends in a path separator, we can't open it.  Check here,
    // because reading the file may actually work, but then creating the
    // swap file may destroy it!  Reported on MS-DOS and Win 95.
    if (after_pathsep(fname, fname + namelen)) {
      if (!silent) {
        filemess(curbuf, fname, _(msg_is_a_directory));
      }
      msg_end();
      msg_scroll = msg_save;
      retval = NOTDONE;
      goto theend;
    }
  }

  if (!read_stdin && fname != NULL) {
    perm = os_getperm(fname);
  }

#ifdef OPEN_CHR_FILES
# define IS_CHR_DEV(perm, fname) S_ISCHR(perm) && rs_is_dev_fd_file(fname)
#else
# define IS_CHR_DEV(perm, fname) false
#endif

  if (!read_stdin && !read_buffer && !read_fifo) {
    if (perm >= 0 && !S_ISREG(perm)                 // not a regular file ...
        && !S_ISFIFO(perm)                          // ... or fifo
        && !S_ISSOCK(perm)                          // ... or socket
        && !(IS_CHR_DEV(perm, fname))
        // ... or a character special file named /dev/fd/<n>
        ) {
      // On Unix it is possible to read a directory, so we have to
      // check for it before os_open().
      if (S_ISDIR(perm)) {
        if (!silent) {
          filemess(curbuf, fname, _(msg_is_a_directory));
        }
        retval = NOTDONE;
      } else {
        filemess(curbuf, fname, _("is not a file"));
      }
      msg_end();
      msg_scroll = msg_save;
      goto theend;
    }
  }

  // Set default or forced 'fileformat' and 'binary'.
  set_file_options(set_options, eap);

  // When opening a new file we take the readonly flag from the file.
  // Default is r/w, can be set to r/o below.
  // Don't reset it when in readonly mode
  // Only set/reset b_p_ro when BF_CHECK_RO is set.
  bool check_readonly = (newfile && (curbuf->b_flags & BF_CHECK_RO));
  if (check_readonly && !readonlymode) {
    curbuf->b_p_ro = false;
  }

  if (newfile && !read_stdin && !read_buffer && !read_fifo) {
    // Remember time of file.
    if (os_fileinfo(fname, &file_info)) {
      buf_store_file_info(curbuf, &file_info);
      curbuf->b_mtime_read = curbuf->b_mtime;
      curbuf->b_mtime_read_ns = curbuf->b_mtime_ns;
#ifdef UNIX
      // Use the protection bits of the original file for the swap file.
      // This makes it possible for others to read the name of the
      // edited file from the swapfile, but only if they can read the
      // edited file.
      // Remove the "write" and "execute" bits for group and others
      // (they must not write the swapfile).
      // Add the "read" and "write" bits for the user, otherwise we may
      // not be able to write to the file ourselves.
      // Setting the bits is done below, after creating the swap file.
      swap_mode = ((int)file_info.stat.st_mode & 0644) | 0600;
#endif
    } else {
      curbuf->b_mtime = 0;
      curbuf->b_mtime_ns = 0;
      curbuf->b_mtime_read = 0;
      curbuf->b_mtime_read_ns = 0;
      curbuf->b_orig_size = 0;
      curbuf->b_orig_mode = 0;
    }

    // Reset the "new file" flag.  It will be set again below when the
    // file doesn't exist.
    curbuf->b_flags &= ~(BF_NEW | BF_NEW_W);
  }

  // Check readonly.
  bool file_readonly = false;
  if (!read_buffer && !read_stdin) {
    if (!newfile || readonlymode || !(perm & 0222)
        || !os_file_is_writable(fname)) {
      file_readonly = true;
    }
    fd = os_open(fname, O_RDONLY, 0);
  }

  if (fd < 0) {                     // cannot open at all
    msg_scroll = msg_save;
    if (!newfile) {
      goto theend;
    }
    if (perm == UV_ENOENT) {  // check if the file exists
      // Set the 'new-file' flag, so that when the file has
      // been created by someone else, a ":w" will complain.
      curbuf->b_flags |= BF_NEW;

      // Create a swap file now, so that other Vims are warned
      // that we are editing this file.  Don't do this for a
      // "nofile" or "nowrite" buffer type.
      if (!bt_dontwrite(curbuf)) {
        check_need_swap(newfile);
        // SwapExists autocommand may mess things up
        if (curbuf != old_curbuf
            || (using_b_ffname
                && (old_b_ffname != curbuf->b_ffname))
            || (using_b_fname
                && (old_b_fname != curbuf->b_fname))) {
          emsg(_(e_auchangedbuf));
          goto theend;
        }
      }
      if (!silent) {
        if (dir_of_file_exists(fname)) {
          filemess(curbuf, sfname, _("[New]"));
        } else {
          filemess(curbuf, sfname, _("[New DIRECTORY]"));
        }
      }
      // Even though this is a new file, it might have been
      // edited before and deleted.  Get the old marks.
      rs_check_marks_read();
      // Set forced 'fileencoding'.
      if (eap != NULL) {
        set_forced_fenc(eap);
      }
      apply_autocmds_exarg(EVENT_BUFNEWFILE, sfname, sfname,
                           false, curbuf, eap);
      // remember the current fileformat
      save_file_ff(curbuf);

      if (!aborting()) {  // autocmds may abort script processing
        retval = OK;      // a new file is not an error
      }
      goto theend;
    }
#if defined(UNIX) && defined(EOVERFLOW)
    filemess(curbuf, sfname, ((fd == UV_EFBIG) ? _("[File too big]")
                                               :
                              // libuv only returns -errno
                              // in Unix and in Windows
                              // open() does not set
                              // EOVERFLOW
                              (fd == -EOVERFLOW) ? _("[File too big]")
                                                 : _("[Permission Denied]")));
#else
    filemess(curbuf, sfname, ((fd == UV_EFBIG) ? _("[File too big]")
                                               : _("[Permission Denied]")));
#endif
    curbuf->b_p_ro = true;                  // must use "w!" now

    goto theend;
  }

  // Only set the 'ro' flag for readonly files the first time they are
  // loaded.    Help files always get readonly mode
  if ((check_readonly && file_readonly) || curbuf->b_help) {
    curbuf->b_p_ro = true;
  }

  if (set_options) {
    // Don't change 'eol' if reading from buffer as it will already be
    // correctly set when reading stdin.
    if (!read_buffer) {
      curbuf->b_p_eof = false;
      curbuf->b_start_eof = false;
      curbuf->b_p_eol = true;
      curbuf->b_start_eol = true;
    }
    curbuf->b_p_bomb = false;
    curbuf->b_start_bomb = false;
  }

  // Create a swap file now, so that other Vims are warned that we are
  // editing this file.
  // Don't do this for a "nofile" or "nowrite" buffer type.
  if (!bt_dontwrite(curbuf)) {
    check_need_swap(newfile);
    if (!read_stdin
        && (curbuf != old_curbuf
            || (using_b_ffname && (old_b_ffname != curbuf->b_ffname))
            || (using_b_fname && (old_b_fname != curbuf->b_fname)))) {
      emsg(_(e_auchangedbuf));
      if (!read_buffer) {
        close(fd);
      }
      goto theend;
    }
#ifdef UNIX
    // Set swap file protection bits after creating it.
    if (swap_mode > 0 && curbuf->b_ml.ml_mfp != NULL
        && curbuf->b_ml.ml_mfp->mf_fname != NULL) {
      const char *swap_fname = curbuf->b_ml.ml_mfp->mf_fname;

      // If the group-read bit is set but not the world-read bit, then
      // the group must be equal to the group of the original file.  If
      // we can't make that happen then reset the group-read bit.  This
      // avoids making the swap file readable to more users when the
      // primary group of the user is too permissive.
      if ((swap_mode & 044) == 040) {
        FileInfo swap_info;

        if (os_fileinfo(swap_fname, &swap_info)
            && file_info.stat.st_gid != swap_info.stat.st_gid
            && os_fchown(curbuf->b_ml.ml_mfp->mf_fd, (uv_uid_t)(-1),
                         (uv_gid_t)file_info.stat.st_gid)
            == -1) {
          swap_mode &= 0600;
        }
      }

      os_setperm(swap_fname, swap_mode);
    }
#endif
  }

  // If "Quit" selected at ATTENTION dialog, don't load the file.
  if (swap_exists_action == SEA_QUIT) {
    if (!read_buffer && !read_stdin) {
      close(fd);
    }
    goto theend;
  }

  no_wait_return++;         // don't wait for return yet

  // Set '[ mark to the line above where the lines go (line 1 if zero).
  orig_start = curbuf->b_op_start;
  curbuf->b_op_start.lnum = ((from == 0) ? 1 : from);
  curbuf->b_op_start.col = 0;

  int try_mac = (vim_strchr(p_ffs, 'm') != NULL);
  int try_dos = (vim_strchr(p_ffs, 'd') != NULL);
  int try_unix = (vim_strchr(p_ffs, 'x') != NULL);

  if (!read_buffer) {
    int m = msg_scroll;
    int n = msg_scrolled;

    // The file must be closed again, the autocommands may want to change
    // the file before reading it.
    if (!read_stdin) {
      close(fd);                // ignore errors
    }

    // The output from the autocommands should not overwrite anything and
    // should not be overwritten: Set msg_scroll, restore its value if no
    // output was done.
    msg_scroll = true;
    if (filtering) {
      apply_autocmds_exarg(EVENT_FILTERREADPRE, NULL, sfname,
                           false, curbuf, eap);
    } else if (read_stdin) {
      apply_autocmds_exarg(EVENT_STDINREADPRE, NULL, sfname,
                           false, curbuf, eap);
    } else if (newfile) {
      apply_autocmds_exarg(EVENT_BUFREADPRE, NULL, sfname,
                           false, curbuf, eap);
    } else {
      apply_autocmds_exarg(EVENT_FILEREADPRE, sfname, sfname,
                           false, NULL, eap);
    }

    // autocommands may have changed it
    try_mac = (vim_strchr(p_ffs, 'm') != NULL);
    try_dos = (vim_strchr(p_ffs, 'd') != NULL);
    try_unix = (vim_strchr(p_ffs, 'x') != NULL);
    curbuf->b_op_start = orig_start;

    if (msg_scrolled == n) {
      msg_scroll = m;
    }

    if (aborting()) {       // autocmds may abort script processing
      no_wait_return--;
      msg_scroll = msg_save;
      curbuf->b_p_ro = true;            // must use "w!" now
      goto theend;
    }
    // Don't allow the autocommands to change the current buffer.
    // Try to re-open the file.
    //
    // Don't allow the autocommands to change the buffer name either
    // (cd for example) if it invalidates fname or sfname.
    if (!read_stdin && (curbuf != old_curbuf
                        || (using_b_ffname && (old_b_ffname != curbuf->b_ffname))
                        || (using_b_fname && (old_b_fname != curbuf->b_fname))
                        || (fd = os_open(fname, O_RDONLY, 0)) < 0)) {
      no_wait_return--;
      msg_scroll = msg_save;
      if (fd < 0) {
        emsg(_("E200: *ReadPre autocommands made the file unreadable"));
      } else {
        emsg(_("E201: *ReadPre autocommands must not change current buffer"));
      }
      curbuf->b_p_ro = true;            // must use "w!" now
      goto theend;
    }
  }

  // Autocommands may add lines to the file, need to check if it is empty
  wasempty = (curbuf->b_ml.ml_flags & ML_EMPTY);

  if (!recoverymode && !filtering && !(flags & READ_DUMMY) && !silent) {
    if (!read_stdin && !read_buffer) {
      filemess(curbuf, sfname, "");
    }
  }

  msg_scroll = false;                   // overwrite the file message

  // Set linecnt now, before the "retry" caused by a wrong guess for
  // fileformat, and after the autocommands, which may change them.
  linecnt = curbuf->b_ml.ml_line_count;

  // "++bad=" argument.
  if (eap != NULL && eap->bad_char != 0) {
    bad_char_behavior = eap->bad_char;
    if (set_options) {
      curbuf->b_bad_char = eap->bad_char;
    }
  } else {
    curbuf->b_bad_char = 0;
  }

  // Decide which 'encoding' to use or use first.
  if (eap != NULL && eap->force_enc != 0) {
    fenc = enc_canonize(eap->cmd + eap->force_enc);
    fenc_alloced = true;
    keep_dest_enc = true;
  } else if (curbuf->b_p_bin) {
    fenc = "";                // binary: don't convert
    fenc_alloced = false;
  } else if (curbuf->b_help) {
    // Help files are either utf-8 or latin1.  Try utf-8 first, if this
    // fails it must be latin1.
    // It is needed when the first line contains non-ASCII characters.
    // That is only in *.??x files.
    fenc_next = "latin1";
    fenc = "utf-8";

    fenc_alloced = false;
  } else if (*p_fencs == NUL) {
    fenc = curbuf->b_p_fenc;            // use format from buffer
    fenc_alloced = false;
  } else {
    fenc_next = p_fencs;                // try items in 'fileencodings'
    fenc = next_fenc(&fenc_next, &fenc_alloced);
  }

  // Jump back here to retry reading the file in different ways.
  // Reasons to retry:
  // - encoding conversion failed: try another one from "fenc_next"
  // - BOM detected and fenc was set, need to setup conversion
  // - "fileformat" check failed: try another
  //
  // Variables set for special retry actions:
  // "file_rewind"      Rewind the file to start reading it again.
  // "advance_fenc"     Advance "fenc" using "fenc_next".
  // "skip_read"        Re-use already read bytes (BOM detected).
  // "did_iconv"        iconv() conversion failed, try 'charconvert'.
  // "keep_fileformat" Don't reset "fileformat".
  //
  // Other status indicators:
  // "tmpname"  When != NULL did conversion with 'charconvert'.
  //                    Output file has to be deleted afterwards.
  // "iconv_fd" When != -1 did conversion with iconv().
retry:

  if (file_rewind) {
    if (read_buffer) {
      read_buf_lnum = 1;
      read_buf_col = 0;
    } else if (read_stdin || vim_lseek(fd, 0, SEEK_SET) != 0) {
      // Can't rewind the file, give up.
      error = true;
      goto failed;
    }
    // Delete the previously read lines.
    while (lnum > from) {
      ml_delete(lnum--);
    }
    file_rewind = false;
    if (set_options) {
      curbuf->b_p_bomb = false;
      curbuf->b_start_bomb = false;
    }
    conv_error = 0;
  }

  // When retrying with another "fenc" and the first time "fileformat"
  // will be reset.
  if (keep_fileformat) {
    keep_fileformat = false;
  } else {
    if (eap != NULL && eap->force_ff != 0) {
      fileformat = get_fileformat_force(curbuf, eap);
      try_unix = try_dos = try_mac = false;
    } else if (curbuf->b_p_bin) {
      fileformat = EOL_UNIX;                    // binary: use Unix format
    } else if (*p_ffs ==
               NUL) {
      fileformat = rs_get_fileformat((buf_T *)curbuf);      // use format from buffer
    } else {
      fileformat = EOL_UNKNOWN;                 // detect from file
    }
  }

  if (iconv_fd != (iconv_t)-1) {
    // aborted conversion with iconv(), close the descriptor
    iconv_close(iconv_fd);
    iconv_fd = (iconv_t)-1;
  }

  if (advance_fenc) {
    // Try the next entry in 'fileencodings'.
    advance_fenc = false;

    if (eap != NULL && eap->force_enc != 0) {
      // Conversion given with "++cc=" wasn't possible, read
      // without conversion.
      notconverted = true;
      conv_error = 0;
      if (fenc_alloced) {
        xfree(fenc);
      }
      fenc = "";
      fenc_alloced = false;
    } else {
      if (fenc_alloced) {
        xfree(fenc);
      }
      if (fenc_next != NULL) {
        fenc = next_fenc(&fenc_next, &fenc_alloced);
      } else {
        fenc = "";
        fenc_alloced = false;
      }
    }
    if (tmpname != NULL) {
      os_remove(tmpname);  // delete converted file
      XFREE_CLEAR(tmpname);
    }
  }

  // Conversion may be required when the encoding of the file is different
  // from 'encoding' or 'encoding' is UTF-16, UCS-2 or UCS-4.
  fio_flags = 0;
  converted = rs_need_conversion(fenc);
  if (converted) {
    // "ucs-bom" means we need to check the first bytes of the file
    // for a BOM.
    if (strcmp(fenc, ENC_UCSBOM) == 0) {
      fio_flags = FIO_UCSBOM;
    } else {
      // Check if UCS-2/4 or Latin1 to UTF-8 conversion needs to be
      // done.  This is handled below after read().  Prepare the
      // fio_flags to avoid having to parse the string each time.
      // Also check for Unicode to Latin1 conversion, because iconv()
      // appears not to handle this correctly.  This works just like
      // conversion to UTF-8 except how the resulting character is put in
      // the buffer.
      fio_flags = rs_get_fio_flags(fenc);
    }

    // Try using iconv() if we can't convert internally.
    if (fio_flags == 0
        && !did_iconv) {
      iconv_fd = (iconv_t)my_iconv_open("utf-8", fenc);
    }

    // Use the 'charconvert' expression when conversion is required
    // and we can't do it internally or with iconv().
    if (fio_flags == 0 && !read_stdin && !read_buffer && *p_ccv != NUL
        && !read_fifo && iconv_fd == (iconv_t)-1) {
      did_iconv = false;
      // Skip conversion when it's already done (retry for wrong
      // "fileformat").
      if (tmpname == NULL) {
        tmpname = readfile_charconvert(fname, fenc, &fd);
        if (tmpname == NULL) {
          // Conversion failed.  Try another one.
          advance_fenc = true;
          if (fd < 0) {
            // Re-opening the original file failed!
            emsg(_("E202: Conversion made file unreadable!"));
            error = true;
            goto failed;
          }
          goto retry;
        }
      }
    } else {
      if (fio_flags == 0 && iconv_fd == (iconv_t)-1) {
        // Conversion wanted but we can't.
        // Try the next conversion in 'fileencodings'
        advance_fenc = true;
        goto retry;
      }
    }
  }

  // Set "can_retry" when it's possible to rewind the file and try with
  // another "fenc" value.  It's false when no other "fenc" to try, reading
  // stdin or fixed at a specific encoding.
  bool can_retry = (*fenc != NUL && !read_stdin && !keep_dest_enc && !read_fifo);

  if (!skip_read) {
    linerest = 0;
    filesize = 0;
    skip_count = lines_to_skip;
    read_count = lines_to_read;
    conv_restlen = 0;
    read_undo_file = (newfile && (flags & READ_KEEP_UNDO) == 0
                      && curbuf->b_ffname != NULL
                      && curbuf->b_p_udf
                      && !filtering
                      && !read_fifo
                      && !read_stdin
                      && !read_buffer);
    if (read_undo_file) {
      sha256_start(&sha_ctx);
    }
  }

  while (!error && !got_int) {
    // We allocate as much space for the file as we can get, plus
    // space for the old line plus room for one terminating NUL.
    // The amount is limited by the fact that read() only can read
    // up to max_unsigned characters (and other things).
    {
      if (!skip_read) {
        // Use buffer >= 64K.  Add linerest to double the size if the
        // line gets very long, to avoid a lot of copying. But don't
        // read more than 1 Mbyte at a time, so we can be interrupted.
        size = MIN(0x10000 + linerest, 0x100000);
      }

      // Protect against the argument of lalloc() going negative.
      if (size < 0 || size + linerest + 1 < 0 || linerest >= MAXCOL) {
        split++;
        *ptr = NL;  // split line by inserting a NL
        size = 1;
      } else if (!skip_read) {
        for (; size >= 10; size /= 2) {
          new_buffer = verbose_try_malloc((size_t)size + (size_t)linerest + 1);
          if (new_buffer) {
            break;
          }
        }
        if (new_buffer == NULL) {
          error = true;
          break;
        }
        if (linerest) {         // copy characters from the previous buffer
          memmove(new_buffer, ptr - linerest, (size_t)linerest);
        }
        xfree(buffer);
        buffer = new_buffer;
        ptr = buffer + linerest;
        line_start = buffer;

        // May need room to translate into.
        // For iconv() we don't really know the required space, use a
        // factor ICONV_MULT.
        // latin1 to utf-8: 1 byte becomes up to 2 bytes
        // utf-16 to utf-8: 2 bytes become up to 3 bytes, 4 bytes
        // become up to 4 bytes, size must be multiple of 2
        // ucs-2 to utf-8: 2 bytes become up to 3 bytes, size must be
        // multiple of 2
        // ucs-4 to utf-8: 4 bytes become up to 6 bytes, size must be
        // multiple of 4
        real_size = (int)size;
        if (iconv_fd != (iconv_t)-1) {
          size = size / ICONV_MULT;
        } else if (fio_flags & FIO_LATIN1) {
          size = size / 2;
        } else if (fio_flags & (FIO_UCS2 | FIO_UTF16)) {
          size = (size * 2 / 3) & ~1;
        } else if (fio_flags & FIO_UCS4) {
          size = (size * 2 / 3) & ~3;
        } else if (fio_flags == FIO_UCSBOM) {
          size = size / ICONV_MULT;  // worst case
        }

        if (conv_restlen > 0) {
          // Insert unconverted bytes from previous line.
          memmove(ptr, conv_rest, (size_t)conv_restlen);
          ptr += conv_restlen;
          size -= conv_restlen;
        }

        if (read_buffer) {
          // Read bytes from curbuf.  Used for converting text read
          // from stdin.
          if (read_buf_lnum > from) {
            size = 0;
          } else {
            int ni;
            int tlen = 0;
            while (true) {
              p = (uint8_t *)ml_get(read_buf_lnum) + read_buf_col;
              int n = ml_get_len(read_buf_lnum) - read_buf_col;
              if (tlen + n + 1 > size) {
                // Filled up to "size", append partial line.
                // Change NL to NUL to reverse the effect done
                // below.
                n = (int)(size - tlen);
                for (ni = 0; ni < n; ni++) {
                  if (p[ni] == NL) {
                    ptr[tlen++] = NUL;
                  } else {
                    ptr[tlen++] = (char)p[ni];
                  }
                }
                read_buf_col += n;
                break;
              }

              // Append whole line and new-line.  Change NL
              // to NUL to reverse the effect done below.
              for (ni = 0; ni < n; ni++) {
                if (p[ni] == NL) {
                  ptr[tlen++] = NUL;
                } else {
                  ptr[tlen++] = (char)p[ni];
                }
              }
              ptr[tlen++] = NL;
              read_buf_col = 0;
              if (++read_buf_lnum > from) {
                // When the last line didn't have an
                // end-of-line don't add it now either.
                if (!curbuf->b_p_eol) {
                  tlen--;
                }
                size = tlen;
                break;
              }
            }
          }
        } else {
          // Read bytes from the file.
          size_t read_size = (size_t)size;
          size = read_eintr(fd, ptr, read_size);
        }

        if (size <= 0) {
          if (size < 0) {                           // read error
            error = true;
          } else if (conv_restlen > 0) {
            // Reached end-of-file but some trailing bytes could
            // not be converted.  Truncated file?

            // When we did a conversion report an error.
            if (fio_flags != 0 || iconv_fd != (iconv_t)-1) {
              if (can_retry) {
                goto rewind_retry;
              }
              if (conv_error == 0) {
                conv_error = curbuf->b_ml.ml_line_count
                             - linecnt + 1;
              }
            } else if (illegal_byte == 0) {
              // Remember the first linenr with an illegal byte
              illegal_byte = curbuf->b_ml.ml_line_count
                             - linecnt + 1;
            }
            if (bad_char_behavior == BAD_DROP) {
              *(ptr - conv_restlen) = NUL;
              conv_restlen = 0;
            } else {
              // Replace the trailing bytes with the replacement
              // character if we were converting; if we weren't,
              // leave the UTF8 checking code to do it, as it
              // works slightly differently.
              if (bad_char_behavior != BAD_KEEP && (fio_flags != 0 || iconv_fd != (iconv_t)-1)) {
                while (conv_restlen > 0) {
                  *(--ptr) = (char)bad_char_behavior;
                  conv_restlen--;
                }
              }
              fio_flags = 0;  // don't convert this
              if (iconv_fd != (iconv_t)-1) {
                iconv_close(iconv_fd);
                iconv_fd = (iconv_t)-1;
              }
            }
          }
        }
      }

      skip_read = false;

      // At start of file: Check for BOM.
      // Also check for a BOM for other Unicode encodings, but not after
      // converting with 'charconvert' or when a BOM has already been
      // found.
      if ((filesize == 0)
          && (fio_flags == FIO_UCSBOM
              || (!curbuf->b_p_bomb
                  && tmpname == NULL
                  && (*fenc == 'u' || *fenc == NUL)))) {
        char *ccname;
        int blen = 0;

        // no BOM detection in a short file or in binary mode
        if (size < 2 || curbuf->b_p_bin) {
          ccname = NULL;
        } else {
          ccname = rs_check_for_bom((const uint8_t *)ptr, (int)size, &blen,
                                    fio_flags == FIO_UCSBOM ? FIO_ALL : rs_get_fio_flags(fenc));
        }
        if (ccname != NULL) {
          // Remove BOM from the text
          filesize += blen;
          size -= blen;
          memmove(ptr, ptr + blen, (size_t)size);
          if (set_options) {
            curbuf->b_p_bomb = true;
            curbuf->b_start_bomb = true;
          }
        }

        if (fio_flags == FIO_UCSBOM) {
          if (ccname == NULL) {
            // No BOM detected: retry with next encoding.
            advance_fenc = true;
          } else {
            // BOM detected: set "fenc" and jump back
            if (fenc_alloced) {
              xfree(fenc);
            }
            fenc = ccname;
            fenc_alloced = false;
          }
          // retry reading without getting new bytes or rewinding
          skip_read = true;
          goto retry;
        }
      }

      // Include not converted bytes.
      ptr -= conv_restlen;
      size += conv_restlen;
      conv_restlen = 0;
      // Break here for a read error or end-of-file.
      if (size <= 0) {
        break;
      }

      if (iconv_fd != (iconv_t)-1) {
        // Attempt conversion of the read bytes to 'encoding' using iconv().
        const char *fromp = ptr;
        size_t from_size = (size_t)size;
        ptr += size;
        char *top = ptr;
        size_t to_size = (size_t)(real_size - size);

        // If there is conversion error or not enough room try using
        // another conversion.  Except for when there is no
        // alternative (help files).
        while ((iconv(iconv_fd, (void *)&fromp, &from_size,
                      &top, &to_size)
                == (size_t)-1 && ICONV_ERRNO != ICONV_EINVAL)
               || from_size > CONV_RESTLEN) {
          if (can_retry) {
            goto rewind_retry;
          }
          if (conv_error == 0) {
            conv_error = readfile_linenr(linecnt, ptr, top);
          }

          // Deal with a bad byte and continue with the next.
          fromp++;
          from_size--;
          if (bad_char_behavior == BAD_KEEP) {
            *top++ = *(fromp - 1);
            to_size--;
          } else if (bad_char_behavior != BAD_DROP) {
            *top++ = (char)bad_char_behavior;
            to_size--;
          }
        }

        if (from_size > 0) {
          // Some remaining characters, keep them for the next
          // round.
          memmove(conv_rest, fromp, from_size);
          conv_restlen = (int)from_size;
        }

        // move the linerest to before the converted characters
        line_start = ptr - linerest;
        memmove(line_start, buffer, (size_t)linerest);
        size = (top - ptr);
      }

      if (fio_flags != 0) {
        unsigned u8c;
        char *tail = NULL;

        // Convert Unicode or Latin1 to UTF-8.
        // Go from end to start through the buffer, because the number
        // of bytes may increase.
        // "dest" points to after where the UTF-8 bytes go, "p" points
        // to after the next character to convert.
        char *dest = ptr + real_size;
        if (fio_flags == FIO_LATIN1 || fio_flags == FIO_UTF8) {
          p = (uint8_t *)ptr + size;
          if (fio_flags == FIO_UTF8) {
            // Check for a trailing incomplete UTF-8 sequence
            tail = ptr + size - 1;
            while (tail > ptr && (*tail & 0xc0) == 0x80) {
              tail--;
            }
            if (tail + utf_byte2len(*tail) <= ptr + size) {
              tail = NULL;
            } else {
              p = (uint8_t *)tail;
            }
          }
        } else if (fio_flags & (FIO_UCS2 | FIO_UTF16)) {
          // Check for a trailing byte
          p = (uint8_t *)ptr + (size & ~1);
          if (size & 1) {
            tail = (char *)p;
          }
          if ((fio_flags & FIO_UTF16) && p > (uint8_t *)ptr) {
            // Check for a trailing leading word
            if (fio_flags & FIO_ENDIAN_L) {
              u8c = (unsigned)(*--p) << 8;
              u8c += *--p;
            } else {
              u8c = *--p;
              u8c += (unsigned)(*--p) << 8;
            }
            if (u8c >= 0xd800 && u8c <= 0xdbff) {
              tail = (char *)p;
            } else {
              p += 2;
            }
          }
        } else {   //  FIO_UCS4
                   // Check for trailing 1, 2 or 3 bytes
          p = (uint8_t *)ptr + (size & ~3);
          if (size & 3) {
            tail = (char *)p;
          }
        }

        // If there is a trailing incomplete sequence move it to
        // conv_rest[].
        if (tail != NULL) {
          conv_restlen = (int)((ptr + size) - tail);
          memmove(conv_rest, tail, (size_t)conv_restlen);
          size -= conv_restlen;
        }

        while (p > (uint8_t *)ptr) {
          if (fio_flags & FIO_LATIN1) {
            u8c = *--p;
          } else if (fio_flags & (FIO_UCS2 | FIO_UTF16)) {
            if (fio_flags & FIO_ENDIAN_L) {
              u8c = (unsigned)(*--p) << 8;
              u8c += *--p;
            } else {
              u8c = *--p;
              u8c += (unsigned)(*--p) << 8;
            }
            if ((fio_flags & FIO_UTF16)
                && u8c >= 0xdc00 && u8c <= 0xdfff) {
              int u16c;

              if (p == (uint8_t *)ptr) {
                // Missing leading word.
                if (can_retry) {
                  goto rewind_retry;
                }
                if (conv_error == 0) {
                  conv_error = readfile_linenr(linecnt, ptr, (char *)p);
                }
                if (bad_char_behavior == BAD_DROP) {
                  continue;
                }
                if (bad_char_behavior != BAD_KEEP) {
                  u8c = (unsigned)bad_char_behavior;
                }
              }

              // found second word of double-word, get the first
              // word and compute the resulting character
              if (fio_flags & FIO_ENDIAN_L) {
                u16c = (*--p << 8);
                u16c += *--p;
              } else {
                u16c = *--p;
                u16c += (*--p << 8);
              }
              u8c = 0x10000 + (((unsigned)u16c & 0x3ff) << 10)
                    + (u8c & 0x3ff);

              // Check if the word is indeed a leading word.
              if (u16c < 0xd800 || u16c > 0xdbff) {
                if (can_retry) {
                  goto rewind_retry;
                }
                if (conv_error == 0) {
                  conv_error = readfile_linenr(linecnt, ptr, (char *)p);
                }
                if (bad_char_behavior == BAD_DROP) {
                  continue;
                }
                if (bad_char_behavior != BAD_KEEP) {
                  u8c = (unsigned)bad_char_behavior;
                }
              }
            }
          } else if (fio_flags & FIO_UCS4) {
            if (fio_flags & FIO_ENDIAN_L) {
              u8c = (unsigned)(*--p) << 24;
              u8c += (unsigned)(*--p) << 16;
              u8c += (unsigned)(*--p) << 8;
              u8c += *--p;
            } else {          // big endian
              u8c = *--p;
              u8c += (unsigned)(*--p) << 8;
              u8c += (unsigned)(*--p) << 16;
              u8c += (unsigned)(*--p) << 24;
            }
            // Replace characters over INT_MAX with Unicode replacement character
            if (u8c > INT_MAX) {
              u8c = 0xfffd;
            }
          } else {        // UTF-8
            if (*--p < 0x80) {
              u8c = *p;
            } else {
              len = utf_head_off(ptr, (char *)p);
              p -= len;
              u8c = (unsigned)utf_ptr2char((char *)p);
              if (len == 0) {
                // Not a valid UTF-8 character, retry with
                // another fenc when possible, otherwise just
                // report the error.
                if (can_retry) {
                  goto rewind_retry;
                }
                if (conv_error == 0) {
                  conv_error = readfile_linenr(linecnt, ptr, (char *)p);
                }
                if (bad_char_behavior == BAD_DROP) {
                  continue;
                }
                if (bad_char_behavior != BAD_KEEP) {
                  u8c = (unsigned)bad_char_behavior;
                }
              }
            }
          }
          assert(u8c <= INT_MAX);
          // produce UTF-8
          dest -= utf_char2len((int)u8c);
          utf_char2bytes((int)u8c, dest);
        }

        // move the linerest to before the converted characters
        line_start = dest - linerest;
        memmove(line_start, buffer, (size_t)linerest);
        size = ((ptr + real_size) - dest);
        ptr = dest;
      } else if (!curbuf->b_p_bin) {
        bool incomplete_tail = false;

        // Reading UTF-8: Check if the bytes are valid UTF-8.
        for (p = (uint8_t *)ptr;; p++) {
          int todo = (int)(((uint8_t *)ptr + size) - p);

          if (todo <= 0) {
            break;
          }
          if (*p >= 0x80) {
            // A length of 1 means it's an illegal byte.  Accept
            // an incomplete character at the end though, the next
            // read() will get the next bytes, we'll check it
            // then.
            int l = utf_ptr2len_len((char *)p, todo);
            if (l > todo && !incomplete_tail) {
              // Avoid retrying with a different encoding when
              // a truncated file is more likely, or attempting
              // to read the rest of an incomplete sequence when
              // we have already done so.
              if (p > (uint8_t *)ptr || filesize > 0) {
                incomplete_tail = true;
              }
              // Incomplete byte sequence, move it to conv_rest[]
              // and try to read the rest of it, unless we've
              // already done so.
              if (p > (uint8_t *)ptr) {
                conv_restlen = todo;
                memmove(conv_rest, p, (size_t)conv_restlen);
                size -= conv_restlen;
                break;
              }
            }
            if (l == 1 || l > todo) {
              // Illegal byte.  If we can try another encoding
              // do that, unless at EOF where a truncated
              // file is more likely than a conversion error.
              if (can_retry && !incomplete_tail) {
                break;
              }

              // When we did a conversion report an error.
              if (iconv_fd != (iconv_t)-1 && conv_error == 0) {
                conv_error = readfile_linenr(linecnt, ptr, (char *)p);
              }

              // Remember the first linenr with an illegal byte
              if (conv_error == 0 && illegal_byte == 0) {
                illegal_byte = readfile_linenr(linecnt, ptr, (char *)p);
              }

              // Drop, keep or replace the bad byte.
              if (bad_char_behavior == BAD_DROP) {
                memmove(p, p + 1, (size_t)(todo - 1));
                p--;
                size--;
              } else if (bad_char_behavior != BAD_KEEP) {
                *p = (uint8_t)bad_char_behavior;
              }
            } else {
              p += l - 1;
            }
          }
        }
        if (p < (uint8_t *)ptr + size && !incomplete_tail) {
          // Detected a UTF-8 error.
rewind_retry:
          // Retry reading with another conversion.
          if (*p_ccv != NUL && iconv_fd != (iconv_t)-1) {
            // iconv() failed, try 'charconvert'
            did_iconv = true;
          } else {
            // use next item from 'fileencodings'
            advance_fenc = true;
          }
          file_rewind = true;
          goto retry;
        }
      }

      // count the number of characters (after conversion!)
      filesize += size;

      // when reading the first part of a file: guess EOL type
      if (fileformat == EOL_UNKNOWN) {
        // First try finding a NL, for Dos and Unix
        if (try_dos || try_unix) {
          // Reset the carriage return counter.
          if (try_mac) {
            try_mac = 1;
          }

          for (p = (uint8_t *)ptr; p < (uint8_t *)ptr + size; p++) {
            if (*p == NL) {
              if (!try_unix
                  || (try_dos && p > (uint8_t *)ptr && p[-1] == CAR)) {
                fileformat = EOL_DOS;
              } else {
                fileformat = EOL_UNIX;
              }
              break;
            } else if (*p == CAR && try_mac) {
              try_mac++;
            }
          }

          // Don't give in to EOL_UNIX if EOL_MAC is more likely
          if (fileformat == EOL_UNIX && try_mac) {
            // Need to reset the counters when retrying fenc.
            try_mac = 1;
            try_unix = 1;
            for (; p >= (uint8_t *)ptr && *p != CAR; p--) {}
            if (p >= (uint8_t *)ptr) {
              for (p = (uint8_t *)ptr; p < (uint8_t *)ptr + size; p++) {
                if (*p == NL) {
                  try_unix++;
                } else if (*p == CAR) {
                  try_mac++;
                }
              }
              if (try_mac > try_unix) {
                fileformat = EOL_MAC;
              }
            }
          } else if (fileformat == EOL_UNKNOWN && try_mac == 1) {
            // Looking for CR but found no end-of-line markers at all:
            // use the default format.
            fileformat = rs_default_fileformat();
          }
        }

        // No NL found: may use Mac format
        if (fileformat == EOL_UNKNOWN && try_mac) {
          fileformat = EOL_MAC;
        }

        // Still nothing found?  Use first format in 'ffs'
        if (fileformat == EOL_UNKNOWN) {
          fileformat = rs_default_fileformat();
        }

        // May set 'p_ff' if editing a new file.
        if (set_options) {
          set_fileformat(fileformat, OPT_LOCAL);
        }
      }
    }

    // This loop is executed once for every character read.
    // Keep it fast!
    if (fileformat == EOL_MAC) {
      ptr--;
      while (++ptr, --size >= 0) {
        // catch most common case first
        if ((c = *ptr) != NUL && c != CAR && c != NL) {
          continue;
        }
        if (c == NUL) {
          *ptr = NL;            // NULs are replaced by newlines!
        } else if (c == NL) {
          *ptr = CAR;           // NLs are replaced by CRs!
        } else {
          if (skip_count == 0) {
            *ptr = NUL;                     // end of line
            len = (colnr_T)(ptr - line_start + 1);
            if (ml_append(lnum, line_start, len, newfile) == FAIL) {
              error = true;
              break;
            }
            if (read_undo_file) {
              sha256_update(&sha_ctx, (uint8_t *)line_start, (size_t)len);
            }
            lnum++;
            if (--read_count == 0) {
              error = true;                     // break loop
              line_start = ptr;                 // nothing left to write
              break;
            }
          } else {
            skip_count--;
          }
          line_start = ptr + 1;
        }
      }
    } else {
      ptr--;
      while (++ptr, --size >= 0) {
        if ((c = *ptr) != NUL && c != NL) {        // catch most common case
          continue;
        }
        if (c == NUL) {
          *ptr = NL;            // NULs are replaced by newlines!
        } else {
          if (skip_count == 0) {
            *ptr = NUL;                         // end of line
            len = (colnr_T)(ptr - line_start + 1);
            if (fileformat == EOL_DOS) {
              if (ptr > line_start && ptr[-1] == CAR) {
                // remove CR before NL
                ptr[-1] = NUL;
                len--;
              } else if (ff_error != EOL_DOS) {
                // Reading in Dos format, but no CR-LF found!
                // When 'fileformats' includes "unix", delete all
                // the lines read so far and start all over again.
                // Otherwise give an error message later.
                if (try_unix
                    && !read_stdin
                    && (read_buffer || vim_lseek(fd, 0, SEEK_SET) == 0)) {
                  fileformat = EOL_UNIX;
                  if (set_options) {
                    set_fileformat(EOL_UNIX, OPT_LOCAL);
                  }
                  file_rewind = true;
                  keep_fileformat = true;
                  goto retry;
                }
                ff_error = EOL_DOS;
              }
            }
            if (ml_append(lnum, line_start, len, newfile) == FAIL) {
              error = true;
              break;
            }
            if (read_undo_file) {
              sha256_update(&sha_ctx, (uint8_t *)line_start, (size_t)len);
            }
            lnum++;
            if (--read_count == 0) {
              error = true;                         // break loop
              line_start = ptr;                 // nothing left to write
              break;
            }
          } else {
            skip_count--;
          }
          line_start = ptr + 1;
        }
      }
    }
    linerest = (ptr - line_start);
    os_breakcheck();
  }

failed:
  // not an error, max. number of lines reached
  if (error && read_count == 0) {
    error = false;
  }

  // In Dos format ignore a trailing CTRL-Z, unless 'binary' is set.
  // In old days the file length was in sector count and the CTRL-Z the
  // marker where the file really ended.  Assuming we write it to a file
  // system that keeps file length properly the CTRL-Z should be dropped.
  // Set the 'endoffile' option so the user can decide what to write later.
  // In Unix format the CTRL-Z is just another character.
  if (linerest != 0
      && !curbuf->b_p_bin
      && fileformat == EOL_DOS
      && ptr[-1] == Ctrl_Z) {
    ptr--;
    linerest--;
    if (set_options) {
      curbuf->b_p_eof = true;
    }
  }

  // If we get EOF in the middle of a line, note the fact and
  // complete the line ourselves.
  if (!error
      && !got_int
      && linerest != 0) {
    // remember for when writing
    if (set_options) {
      curbuf->b_p_eol = false;
    }
    *ptr = NUL;
    len = (colnr_T)(ptr - line_start + 1);
    if (ml_append(lnum, line_start, len, newfile) == FAIL) {
      error = true;
    } else {
      if (read_undo_file) {
        sha256_update(&sha_ctx, (uint8_t *)line_start, (size_t)len);
      }
      read_no_eol_lnum = ++lnum;
    }
  }

  if (set_options) {
    // Remember the current file format.
    save_file_ff(curbuf);
    // If editing a new file: set 'fenc' for the current buffer.
    // Also for ":read ++edit file".
    set_option_direct(kOptFileencoding, CSTR_AS_OPTVAL(fenc), OPT_LOCAL, 0);
  }
  if (fenc_alloced) {
    xfree(fenc);
  }
  if (iconv_fd != (iconv_t)-1) {
    iconv_close(iconv_fd);
  }

  if (!read_buffer && !read_stdin) {
    close(fd);  // errors are ignored
  } else {
    os_set_cloexec(fd);
  }
  xfree(buffer);

  if (read_stdin) {
    close(fd);
    if (stdin_fd < 0) {
#ifndef MSWIN
      // On Unix, use stderr for stdin, makes shell commands work.
      vim_ignored = dup(2);
#else
      // On Windows, use the console input handle for stdin.
      HANDLE conin = CreateFile("CONIN$", GENERIC_READ | GENERIC_WRITE,
                                FILE_SHARE_READ, (LPSECURITY_ATTRIBUTES)NULL,
                                OPEN_EXISTING, 0, (HANDLE)NULL);
      vim_ignored = _open_osfhandle((intptr_t)conin, _O_RDONLY);
#endif
    }
  }

  if (tmpname != NULL) {
    os_remove(tmpname);  // delete converted file
    xfree(tmpname);
  }
  no_wait_return--;                     // may wait for return now

  // In recovery mode everything but autocommands is skipped.
  if (!recoverymode) {
    // need to delete the last line, which comes from the empty buffer
    if (newfile && wasempty && !(curbuf->b_ml.ml_flags & ML_EMPTY)) {
      ml_delete(curbuf->b_ml.ml_line_count);
      linecnt--;
    }
    curbuf->deleted_bytes = 0;
    curbuf->deleted_bytes2 = 0;
    curbuf->deleted_codepoints = 0;
    curbuf->deleted_codeunits = 0;
    linecnt = curbuf->b_ml.ml_line_count - linecnt;
    if (filesize == 0) {
      linecnt = 0;
    }
    if (newfile || read_buffer) {
      redraw_curbuf_later(UPD_NOT_VALID);
      // After reading the text into the buffer the diff info needs to
      // be updated.
      rs_diff_invalidate(curbuf);
      // All folds in the window are invalid now.  Mark them for update
      // before triggering autocommands.
      rs_foldUpdateAll(curwin);
    } else if (linecnt) {               // appended at least one line
      appended_lines_mark(from, linecnt);
    }

    if (got_int) {
      if (!(flags & READ_DUMMY)) {
        filemess(curbuf, sfname, _(e_interr));
        if (newfile) {
          curbuf->b_p_ro = true;                // must use "w!" now
        }
      }
      msg_scroll = msg_save;
      rs_check_marks_read();
      retval = OK;        // an interrupt isn't really an error
      goto theend;
    }

    if (!filtering && !(flags & READ_DUMMY) && !silent) {
      add_quoted_fname(IObuff, IOSIZE, curbuf, sfname);
      c = false;

#ifdef UNIX
      if (S_ISFIFO(perm)) {             // fifo
        xstrlcat(IObuff, _("[fifo]"), IOSIZE);
        c = true;
      }
      if (S_ISSOCK(perm)) {            // or socket
        xstrlcat(IObuff, _("[socket]"), IOSIZE);
        c = true;
      }
# ifdef OPEN_CHR_FILES
      if (S_ISCHR(perm)) {                          // or character special
        xstrlcat(IObuff, _("[character special]"), IOSIZE);
        c = true;
      }
# endif
#endif
      if (curbuf->b_p_ro) {
        xstrlcat(IObuff, shortmess(SHM_RO) ? _("[RO]") : _("[readonly]"), IOSIZE);
        c = true;
      }
      if (read_no_eol_lnum) {
        xstrlcat(IObuff, _("[noeol]"), IOSIZE);
        c = true;
      }
      if (ff_error == EOL_DOS) {
        xstrlcat(IObuff, _("[CR missing]"), IOSIZE);
        c = true;
      }
      if (split) {
        xstrlcat(IObuff, _("[long lines split]"), IOSIZE);
        c = true;
      }
      if (notconverted) {
        xstrlcat(IObuff, _("[NOT converted]"), IOSIZE);
        c = true;
      } else if (converted) {
        xstrlcat(IObuff, _("[converted]"), IOSIZE);
        c = true;
      }
      if (conv_error != 0) {
        snprintf(IObuff + strlen(IObuff), IOSIZE - strlen(IObuff),
                 _("[CONVERSION ERROR in line %" PRId64 "]"), (int64_t)conv_error);
        c = true;
      } else if (illegal_byte > 0) {
        snprintf(IObuff + strlen(IObuff), IOSIZE - strlen(IObuff),
                 _("[ILLEGAL BYTE in line %" PRId64 "]"), (int64_t)illegal_byte);
        c = true;
      } else if (error) {
        xstrlcat(IObuff, _("[READ ERRORS]"), IOSIZE);
        c = true;
      }
      if (msg_add_fileformat(fileformat)) {
        c = true;
      }

      msg_add_lines(c, linecnt, filesize);

      XFREE_CLEAR(keep_msg);
      p = NULL;
      msg_scrolled_ign = true;

      if (!read_stdin && !read_buffer) {
        if (msg_col > 0) {
          msg_putchar('\r');  // overwrite previous message
        }
        p = (uint8_t *)msg_trunc(IObuff, false, 0);
      }

      if (read_stdin || read_buffer || restart_edit != 0
          || (msg_scrolled != 0 && !need_wait_return)) {
        // Need to repeat the message after redrawing when:
        // - When reading from stdin (the screen will be cleared next).
        // - When restart_edit is set (otherwise there will be a delay before
        //   redrawing).
        // - When the screen was scrolled but there is no wait-return prompt.
        set_keep_msg((char *)p, 0);
      }
      msg_scrolled_ign = false;
    }

    // with errors writing the file requires ":w!"
    if (newfile && (error
                    || conv_error != 0
                    || (illegal_byte > 0 && bad_char_behavior != BAD_KEEP))) {
      curbuf->b_p_ro = true;
    }

    u_clearline(curbuf);   // cannot use "U" command after adding lines

    // In Ex mode: cursor at last new line.
    // Otherwise: cursor at first new line.
    if (exmode_active) {
      curwin->w_cursor.lnum = from + linecnt;
    } else {
      curwin->w_cursor.lnum = from + 1;
    }
    check_cursor_lnum(curwin);
    beginline(BL_WHITE | BL_FIX);           // on first non-blank

    if ((cmdmod.cmod_flags & CMOD_LOCKMARKS) == 0) {
      // Set '[ and '] marks to the newly read lines.
      curbuf->b_op_start.lnum = from + 1;
      curbuf->b_op_start.col = 0;
      curbuf->b_op_end.lnum = from + linecnt;
      curbuf->b_op_end.col = 0;
    }
  }
  msg_scroll = msg_save;

  // Get the marks before executing autocommands, so they can be used there.
  rs_check_marks_read();

  // We remember if the last line of the read didn't have
  // an eol even when 'binary' is off, to support turning 'fixeol' off,
  // or writing the read again with 'binary' on.  The latter is required
  // for ":autocmd FileReadPost *.gz set bin|'[,']!gunzip" to work.
  curbuf->b_no_eol_lnum = read_no_eol_lnum;

  // When reloading a buffer put the cursor at the first line that is
  // different.
  if (flags & READ_KEEP_UNDO) {
    u_find_first_changed();
  }

  // When opening a new file locate undo info and read it.
  if (read_undo_file) {
    uint8_t hash[UNDO_HASH_SIZE];

    sha256_finish(&sha_ctx, hash);
    u_read_undo(NULL, hash, fname);
  }

  if (!read_stdin && !read_fifo && (!read_buffer || sfname != NULL)) {
    int m = msg_scroll;
    int n = msg_scrolled;

    // Save the fileformat now, otherwise the buffer will be considered
    // modified if the format/encoding was automatically detected.
    if (set_options) {
      save_file_ff(curbuf);
    }

    // The output from the autocommands should not overwrite anything and
    // should not be overwritten: Set msg_scroll, restore its value if no
    // output was done.
    msg_scroll = true;
    if (filtering) {
      apply_autocmds_exarg(EVENT_FILTERREADPOST, NULL, sfname,
                           false, curbuf, eap);
    } else if (newfile || (read_buffer && sfname != NULL)) {
      apply_autocmds_exarg(EVENT_BUFREADPOST, NULL, sfname,
                           false, curbuf, eap);
      if (!curbuf->b_au_did_filetype && *curbuf->b_p_ft != NUL) {
        // EVENT_FILETYPE was not triggered but the buffer already has a
        // filetype.  Trigger EVENT_FILETYPE using the existing filetype.
        apply_autocmds(EVENT_FILETYPE, curbuf->b_p_ft, curbuf->b_fname, true, curbuf);
      }
    } else {
      apply_autocmds_exarg(EVENT_FILEREADPOST, sfname, sfname,
                           false, NULL, eap);
    }
    if (msg_scrolled == n) {
      msg_scroll = m;
    }
    if (aborting()) {       // autocmds may abort script processing
      return FAIL;
    }
  }

  if (!(recoverymode && error)) {
    retval = OK;
  }

theend:
  if (curbuf->b_ml.ml_mfp != NULL
      && curbuf->b_ml.ml_mfp->mf_dirty == MF_DIRTY_YES_NOSYNC) {
    // OK to sync the swap file now
    curbuf->b_ml.ml_mfp->mf_dirty = MF_DIRTY_YES;
  }

  return retval;
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

