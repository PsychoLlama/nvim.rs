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
extern int rs_xdiff_out(int start_a, int count_a, int start_b, int count_b, void *priv);
extern void rs_f_diff_filler(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_nv_diffgetput(bool put, size_t count);
extern void rs_ex_diffthis(exarg_T *eap);
extern void rs_f_diff_hlID(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void rs_diff_redraw(bool dofold);

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

static void clear_diffin(diffin_T *din)
{
  if (din->din_fname == NULL) {
    XFREE_CLEAR(din->din_mmfile.ptr);
  } else {
    os_remove(din->din_fname);
  }
}

static void clear_diffout(diffout_T *dout)
{
  if (dout->dout_fname == NULL) {
    ga_clear(&dout->dout_ga);
  } else {
    os_remove(dout->dout_fname);
  }
}

/// Write buffer "buf" to a memory buffer.
///
/// @param buf
/// @param din
///
/// @return FAIL for failure.
static int diff_write_buffer(buf_T *buf, mmfile_t *m, linenr_T start, linenr_T end)
{
  if (end < 0) {
    end = buf->b_ml.ml_line_count;
  }

  if (buf->b_ml.ml_flags & ML_EMPTY || end < start) {
    m->ptr = NULL;
    m->size = 0;
    return OK;
  }

  size_t len = 0;

  // xdiff requires one big block of memory with all the text.
  for (linenr_T lnum = start; lnum <= end; lnum++) {
    len += (size_t)ml_get_buf_len(buf, lnum) + 1;
  }
  char *ptr = xmalloc(len);
  m->ptr = ptr;
  m->size = (int)len;

  len = 0;
  for (linenr_T lnum = start; lnum <= end; lnum++) {
    char *s = ml_get_buf(buf, lnum);
    if (diff_flags & DIFF_ICASE) {
      while (*s != NUL) {
        int c;
        int c_len = 1;
        char cbuf[MB_MAXBYTES + 1];

        if (*s == NL) {
          c = NUL;
        } else {
          // xdiff doesn't support ignoring case, fold-case the text.
          c = utf_ptr2char(s);
          c_len = utf_char2len(c);
          c = utf_fold(c);
        }
        const int orig_len = utfc_ptr2len(s);

        if (utf_char2bytes(c, cbuf) != c_len) {
          // TODO(Bram): handle byte length difference
          // One example is Å (3 bytes) and å (2 bytes).
          memmove(ptr + len, s, (size_t)orig_len);
        } else {
          memmove(ptr + len, cbuf, (size_t)c_len);
          if (orig_len > c_len) {
            // Copy remaining composing characters
            memmove(ptr + len + c_len, s + c_len, (size_t)(orig_len - c_len));
          }
        }

        s += orig_len;
        len += (size_t)orig_len;
      }
    } else {
      size_t slen = strlen(s);
      memmove(ptr + len, s, slen);
      // NUL is represented as NL; convert
      memchrsub(ptr + len, NL, NUL, slen);
      len += slen;
    }
    ptr[len++] = NL;
  }
  return OK;
}

/// Write buffer "buf" to file or memory buffer.
///
/// Always use 'fileformat' set to "unix".
///
/// @param buf
/// @param din
///
/// @return FAIL for failure
static int diff_write(buf_T *buf, diffin_T *din, linenr_T start, linenr_T end)
{
  if (din->din_fname == NULL) {
    return diff_write_buffer(buf, &din->din_mmfile, start, end);
  }

  if (end < 0) {
    end = buf->b_ml.ml_line_count;
  }

  // Always use 'fileformat' set to "unix".
  int save_ml_flags = buf->b_ml.ml_flags;
  char *save_ff = buf->b_p_ff;
  buf->b_p_ff = xstrdup("unix");
  const bool save_cmod_flags = cmdmod.cmod_flags;
  // Writing the buffer is an implementation detail of performing the diff,
  // so it shouldn't update the '[ and '] marks.
  cmdmod.cmod_flags |= CMOD_LOCKMARKS;
  if (end < start) {
    // The line range specifies a completely empty file.
    end = start;
    buf->b_ml.ml_flags |= ML_EMPTY;
  }
  int r = buf_write(buf, din->din_fname, NULL,
                    start, end,
                    NULL, false, false, false, true);
  cmdmod.cmod_flags = save_cmod_flags;
  free_string_option(buf->b_p_ff);
  buf->b_p_ff = save_ff;
  buf->b_ml.ml_flags = (buf->b_ml.ml_flags & ~ML_EMPTY) | (save_ml_flags & ML_EMPTY);
  return r;
}

/// Completely update the diffs for the buffers involved.
///
/// @param eap can be NULL
void ex_diffupdate(exarg_T *eap)
{
  rs_diff_ex_diffupdate(eap);
}

/// Do a quick test if "diff" really works.  Otherwise it looks like there
/// are no differences.  Can't use the return value, it's non-zero when
/// there are differences.
static int check_external_diff(diffio_T *diffio)
{
  // May try twice, first with "-a" and then without.
  bool io_error = false;
  TriState ok = kFalse;
  while (true) {
    ok = kFalse;
    FILE *fd = os_fopen(diffio->dio_orig.din_fname, "w");

    if (fd == NULL) {
      io_error = true;
    } else {
      if (fwrite("line1\n", 6, 1, fd) != 1) {
        io_error = true;
      }
      fclose(fd);
      fd = os_fopen(diffio->dio_new.din_fname, "w");

      if (fd == NULL) {
        io_error = true;
      } else {
        if (fwrite("line2\n", 6, 1, fd) != 1) {
          io_error = true;
        }
        fclose(fd);
        fd = diff_file(diffio) == OK
             ? os_fopen(diffio->dio_diff.dout_fname, "r")
             : NULL;

        if (fd == NULL) {
          io_error = true;
        } else {
          char linebuf[LBUFLEN];

          while (true) {
            // For normal diff there must be a line that contains
            // "1c1".  For unified diff "@@ -1 +1 @@".
            if (vim_fgets(linebuf, LBUFLEN, fd)) {
              break;
            }

            if (strncmp(linebuf, "1c1", 3) == 0
                || strncmp(linebuf, "@@ -1 +1 @@", 11) == 0) {
              ok = kTrue;
            }
          }
          fclose(fd);
        }
        os_remove(diffio->dio_diff.dout_fname);
        os_remove(diffio->dio_new.din_fname);
      }
      os_remove(diffio->dio_orig.din_fname);
    }

    // When using 'diffexpr' break here.
    if (*p_dex != NUL) {
      break;
    }

    // If we checked if "-a" works already, break here.
    if (diff_a_works != kNone) {
      break;
    }
    diff_a_works = ok;

    // If "-a" works break here, otherwise retry without "-a".
    if (ok) {
      break;
    }
  }

  if (!ok) {
    if (io_error) {
      emsg(_("E810: Cannot read or write temp files"));
    }
    emsg(_("E97: Cannot create diffs"));
    diff_a_works = kNone;
    return FAIL;
  }
  return OK;
}

/// Invoke the xdiff function.
static int diff_file_internal(diffio_T *diffio)
{
  xpparam_t param;
  xdemitconf_t emit_cfg;
  xdemitcb_t emit_cb;

  CLEAR_FIELD(param);
  CLEAR_FIELD(emit_cfg);
  CLEAR_FIELD(emit_cb);

  param.flags = (unsigned long)diff_algorithm;

  if (diff_flags & DIFF_IWHITE) {
    param.flags |= XDF_IGNORE_WHITESPACE_CHANGE;
  }
  if (diff_flags & DIFF_IWHITEALL) {
    param.flags |= XDF_IGNORE_WHITESPACE;
  }
  if (diff_flags & DIFF_IWHITEEOL) {
    param.flags |= XDF_IGNORE_WHITESPACE_AT_EOL;
  }
  if (diff_flags & DIFF_IBLANK) {
    param.flags |= XDF_IGNORE_BLANK_LINES;
  }

  emit_cfg.ctxlen = 0;  // don't need any diff_context here
  emit_cb.priv = &diffio->dio_diff;
  emit_cfg.hunk_func = xdiff_out;
  if (xdl_diff(&diffio->dio_orig.din_mmfile,
               &diffio->dio_new.din_mmfile,
               &param, &emit_cfg, &emit_cb) < 0) {
    emsg(_("E960: Problem creating the internal diff"));
    return FAIL;
  }
  return OK;
}

/// Make a diff between files "tmp_orig" and "tmp_new", results in "tmp_diff".
///
/// @param dio
///
/// @return OK or FAIL
static int diff_file(diffio_T *dio)
{
  char *tmp_orig = dio->dio_orig.din_fname;
  char *tmp_new = dio->dio_new.din_fname;
  char *tmp_diff = dio->dio_diff.dout_fname;
  if (*p_dex != NUL) {
    // Use 'diffexpr' to generate the diff file.
    eval_diff(tmp_orig, tmp_new, tmp_diff);
    return OK;
  }
  // Use xdiff for generating the diff.
  if (dio->dio_internal) {
    return diff_file_internal(dio);
  }

  const size_t len = (strlen(tmp_orig) + strlen(tmp_new) + strlen(tmp_diff)
                      + strlen(p_srr) + 27);
  char *const cmd = xmalloc(len);

  // We don't want $DIFF_OPTIONS to get in the way.
  if (os_env_exists("DIFF_OPTIONS", true)) {
    os_unsetenv("DIFF_OPTIONS");
  }

  // Build the diff command and execute it.  Always use -a, binary
  // differences are of no use.  Ignore errors, diff returns
  // non-zero when differences have been found.
  vim_snprintf(cmd, len, "diff %s%s%s%s%s%s%s%s %s",
               diff_a_works == kFalse ? "" : "-a ",
               "",
               (diff_flags & DIFF_IWHITE) ? "-b " : "",
               (diff_flags & DIFF_IWHITEALL) ? "-w " : "",
               (diff_flags & DIFF_IWHITEEOL) ? "-Z " : "",
               (diff_flags & DIFF_IBLANK) ? "-B " : "",
               (diff_flags & DIFF_ICASE) ? "-i " : "",
               tmp_orig, tmp_new);
  append_redir(cmd, len, p_srr, tmp_diff);
  block_autocmds();  // Avoid ShellCmdPost stuff
  call_shell(cmd,
             kShellOptFilter | kShellOptSilent | kShellOptDoOut,
             NULL);
  unblock_autocmds();
  xfree(cmd);
  return OK;
}

/// Create a new version of a file from the current buffer and a diff file.
///
/// The buffer is written to a file, also for unmodified buffers (the file
/// could have been produced by autocommands, e.g. the netrw plugin).
///
/// @param eap
void ex_diffpatch(exarg_T *eap)
{
  char *buf = NULL;
  win_T *old_curwin = curwin;
  char *newname = NULL;  // name of patched file buffer
  char *esc_name = NULL;

#ifdef UNIX
  char *fullname = NULL;
#endif

  // We need two temp file names.
  // Name of original temp file.
  char *tmp_orig = vim_tempname();
  // Name of patched temp file.
  char *tmp_new = vim_tempname();

  if ((tmp_orig == NULL) || (tmp_new == NULL)) {
    goto theend;
  }

  // Write the current buffer to "tmp_orig".
  if (buf_write(curbuf, tmp_orig, NULL,
                1, curbuf->b_ml.ml_line_count,
                NULL, false, false, false, true) == FAIL) {
    goto theend;
  }

#ifdef UNIX
  // Get the absolute path of the patchfile, changing directory below.
  fullname = FullName_save(eap->arg, false);
  esc_name = vim_strsave_shellescape(fullname != NULL ? fullname : eap->arg, true, true);
#else
  esc_name = vim_strsave_shellescape(eap->arg, true, true);
#endif
  size_t buflen = strlen(tmp_orig) + strlen(esc_name) + strlen(tmp_new) + 16;
  buf = xmalloc(buflen);

#ifdef UNIX
  char dirbuf[MAXPATHL];
  // Temporarily chdir to /tmp, to avoid patching files in the current
  // directory when the patch file contains more than one patch.  When we
  // have our own temp dir use that instead, it will be cleaned up when we
  // exit (any .rej files created).  Don't change directory if we can't
  // return to the current.
  if ((os_dirname(dirbuf, MAXPATHL) != OK)
      || (os_chdir(dirbuf) != 0)) {
    dirbuf[0] = NUL;
  } else {
    char *tempdir = vim_gettempdir();
    if (tempdir == NULL) {
      tempdir = "/tmp";
    }
    os_chdir(tempdir);
    shorten_fnames(true);
  }
#endif

  if (*p_pex != NUL) {
    // Use 'patchexpr' to generate the new file.
#ifdef UNIX
    eval_patch(tmp_orig, (fullname != NULL ? fullname : eap->arg), tmp_new);
#else
    eval_patch(tmp_orig, eap->arg, tmp_new);
#endif
  } else {
    // Build the patch command and execute it. Ignore errors.
    vim_snprintf(buf, buflen, "patch -o %s %s < %s",
                 tmp_new, tmp_orig, esc_name);
    block_autocmds();  // Avoid ShellCmdPost stuff
    call_shell(buf, kShellOptFilter, NULL);
    unblock_autocmds();
  }

#ifdef UNIX
  if (dirbuf[0] != NUL) {
    if (os_chdir(dirbuf) != 0) {
      emsg(_(e_prev_dir));
    }
    shorten_fnames(true);
  }
#endif

  // Delete any .orig or .rej file created.
  STRCPY(buf, tmp_new);
  strcat(buf, ".orig");
  os_remove(buf);
  STRCPY(buf, tmp_new);
  strcat(buf, ".rej");
  os_remove(buf);

  // Only continue if the output file was created.
  FileInfo file_info;
  bool info_ok = os_fileinfo(tmp_new, &file_info);
  uint64_t filesize = os_fileinfo_size(&file_info);
  if (!info_ok || filesize == 0) {
    emsg(_("E816: Cannot read patch output"));
  } else {
    if (curbuf->b_fname != NULL) {
      newname = xstrnsave(curbuf->b_fname, strlen(curbuf->b_fname) + 4);
      strcat(newname, ".new");
    }

    // don't use a new tab page, each tab page has its own diffs
    cmdmod.cmod_tab = 0;

    if (win_split(0, (diff_flags & DIFF_VERTICAL) ? WSP_VERT : 0) != FAIL) {
      // Pretend it was a ":split fname" command
      eap->cmdidx = CMD_split;
      eap->arg = tmp_new;
      do_exedit(eap, old_curwin);

      // check that split worked and editing tmp_new
      if ((curwin != old_curwin) && rs_win_valid(old_curwin)) {
        // Set 'diff', 'scrollbind' on and 'wrap' off.
        diff_win_options(curwin, true);
        diff_win_options(old_curwin, true);

        if (newname != NULL) {
          // do a ":file filename.new" on the patched buffer
          eap->arg = newname;
          ex_file(eap);

          // Do filetype detection with the new name.
          if (augroup_exists("filetypedetect")) {
            do_cmdline_cmd(":doau filetypedetect BufRead");
          }
        }
      }
    }
  }

theend:
  if (tmp_orig != NULL) {
    os_remove(tmp_orig);
  }
  xfree(tmp_orig);

  if (tmp_new != NULL) {
    os_remove(tmp_new);
  }
  xfree(tmp_new);
  xfree(newname);
  xfree(buf);
#ifdef UNIX
  xfree(fullname);
#endif
  xfree(esc_name);
}

/// Split the window and edit another file, setting options to show the diffs.
///
/// @param eap
void ex_diffsplit(exarg_T *eap)
{
  win_T *old_curwin = curwin;
  bufref_T old_curbuf;
  set_bufref(&old_curbuf, curbuf);

  // Need to compute w_fraction when no redraw happened yet.
  validate_cursor(curwin);
  rs_set_fraction(curwin);

  // don't use a new tab page, each tab page has its own diffs
  cmdmod.cmod_tab = 0;

  if (win_split(0, (diff_flags & DIFF_VERTICAL) ? WSP_VERT : 0) == FAIL) {
    return;
  }

  // Pretend it was a ":split fname" command
  eap->cmdidx = CMD_split;
  curwin->w_p_diff = true;
  do_exedit(eap, old_curwin);

  if (curwin == old_curwin) {  // split didn't work
    return;
  }

  // Set 'diff', 'scrollbind' on and 'wrap' off.
  diff_win_options(curwin, true);
  if (rs_win_valid(old_curwin)) {
    diff_win_options(old_curwin, true);

    if (bufref_valid(&old_curbuf)) {
      // Move the cursor position to that of the old window.
      curwin->w_cursor.lnum = rs_diff_get_corresponding_line(old_curbuf.br_buf,
                                                             old_curwin->w_cursor.lnum);
    }
  }
  // Now that lines are folded scroll to show the cursor at the same
  // relative position.
  scroll_to_fraction(curwin, curwin->w_height);
}

// Set options to show diffs for the current window -- thin wrapper calling Rust rs_ex_diffthis.
void ex_diffthis(exarg_T *eap)
{
  rs_ex_diffthis(eap);
}

/// Set options in window "wp" for diff mode.
///
/// @param addbuf Add buffer to diff.
void diff_win_options(win_T *wp, bool addbuf)
{
  win_T *old_curwin = curwin;

  // close the manually opened folds
  curwin = wp;
  rs_newFoldLevel();
  curwin = old_curwin;

  // Use 'scrollbind' and 'cursorbind' when available
  if (!wp->w_p_diff) {
    wp->w_p_scb_save = wp->w_p_scb;
  }
  wp->w_p_scb = true;

  if (!wp->w_p_diff) {
    wp->w_p_crb_save = wp->w_p_crb;
  }
  wp->w_p_crb = true;
  if (!(diff_flags & DIFF_FOLLOWWRAP)) {
    if (!wp->w_p_diff) {
      wp->w_p_wrap_save = wp->w_p_wrap;
    }
    wp->w_p_wrap = false;
    wp->w_skipcol = 0;
  }

  if (!wp->w_p_diff) {
    if (wp->w_p_diff_saved) {
      free_string_option(wp->w_p_fdm_save);
    }
    wp->w_p_fdm_save = xstrdup(wp->w_p_fdm);
  }
  set_option_direct_for(kOptFoldmethod, STATIC_CSTR_AS_OPTVAL("diff"), OPT_LOCAL, 0,
                        kOptScopeWin, wp);

  if (!wp->w_p_diff) {
    wp->w_p_fen_save = wp->w_p_fen;
    wp->w_p_fdl_save = wp->w_p_fdl;

    if (wp->w_p_diff_saved) {
      free_string_option(wp->w_p_fdc_save);
    }
    wp->w_p_fdc_save = xstrdup(wp->w_p_fdc);
  }
  free_string_option(wp->w_p_fdc);
  wp->w_p_fdc = xstrdup("2");
  assert(diff_foldcolumn >= 0 && diff_foldcolumn <= 9);
  snprintf(wp->w_p_fdc, strlen(wp->w_p_fdc) + 1, "%d", diff_foldcolumn);
  wp->w_p_fen = true;
  wp->w_p_fdl = 0;
  rs_foldUpdateAll(wp);

  // make sure topline is not halfway through a fold
  changed_window_setting(wp);
  if (vim_strchr(p_sbo, 'h') == NULL) {
    do_cmdline_cmd("set sbo+=hor");
  }

  // Save the current values, to be restored in ex_diffoff().
  wp->w_p_diff_saved = true;

  rs_set_diff_option(wp, true);

  if (addbuf) {
    rs_diff_buf_add(wp->w_buffer);
  }
  redraw_later(wp, UPD_NOT_VALID);
}

/// Set options not to show diffs.  For the current window or all windows.
/// Only in the current tab page.
///
/// @param eap
void ex_diffoff(exarg_T *eap)
{
  bool diffwin = false;

  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (eap->forceit ? wp->w_p_diff : (wp == curwin)) {
      // Set 'diff' off. If option values were saved in
      // diff_win_options(), restore the ones whose settings seem to have
      // been left over from diff mode.
      rs_set_diff_option(wp, false);

      if (wp->w_p_diff_saved) {
        if (wp->w_p_scb) {
          wp->w_p_scb = wp->w_p_scb_save;
        }

        if (wp->w_p_crb) {
          wp->w_p_crb = wp->w_p_crb_save;
        }
        if (!(diff_flags & DIFF_FOLLOWWRAP)) {
          if (!wp->w_p_wrap && wp->w_p_wrap_save) {
            wp->w_p_wrap = true;
            wp->w_leftcol = 0;
          }
        }
        free_string_option(wp->w_p_fdm);
        wp->w_p_fdm = xstrdup(*wp->w_p_fdm_save ? wp->w_p_fdm_save : "manual");
        free_string_option(wp->w_p_fdc);
        wp->w_p_fdc = xstrdup(*wp->w_p_fdc_save ? wp->w_p_fdc_save : "0");

        if (wp->w_p_fdl == 0) {
          wp->w_p_fdl = wp->w_p_fdl_save;
        }
        // Only restore 'foldenable' when 'foldmethod' is not
        // "manual", otherwise we continue to show the diff folds.
        if (wp->w_p_fen) {
          wp->w_p_fen = rs_foldmethodIsManual(wp) ? false : wp->w_p_fen_save;
        }

        rs_foldUpdateAll(wp);
      }
      // remove filler lines
      wp->w_topfill = 0;

      // make sure topline is not halfway a fold and cursor is
      // invalidated
      changed_window_setting(wp);

      // Note: 'sbo' is not restored, it's a global option.
      rs_diff_buf_adjust(wp);
    }
    diffwin |= wp->w_p_diff;
  }

  // Also remove hidden buffers from the list.
  if (eap->forceit) {
    rs_diff_buf_clear();
  }

  if (!diffwin) {
    diff_need_update = false;
    curtab->tp_diff_invalid = false;
    curtab->tp_diff_update = false;
    rs_diff_clear(curtab);
  }

  // Remove "hor" from 'scrollopt' if there are no diff windows left.
  if (!diffwin && (vim_strchr(p_sbo, 'h') != NULL)) {
    do_cmdline_cmd("set sbo-=hor");
  }
}

// Apply results from the linematch algorithm and apply to 'dp' by splitting it into multiple
// adjacent diff blocks.
static void apply_linematch_results(diff_T *dp, size_t decisions_length, const int *decisions)
{
  // get the start line number here in each diff buffer, and then increment
  int line_numbers[DB_COUNT];
  int outputmap[DB_COUNT];
  size_t ndiffs = 0;
  for (int i = 0; i < DB_COUNT; i++) {
    if (curtab->tp_diffbuf[i] != NULL) {
      line_numbers[i] = dp->df_lnum[i];
      dp->df_count[i] = 0;

      // Keep track of the index of the diff buffer we are using here.
      // We will use this to write the output of the algorithm to
      // diff_T structs at the correct indexes
      outputmap[ndiffs] = i;
      ndiffs++;
    }
  }

  // write the diffs starting with the current diff block
  diff_T *dp_s = dp;
  for (size_t i = 0; i < decisions_length; i++) {
    // Don't allocate on first iter since we can reuse the initial diffblock
    if (i != 0 && (decisions[i - 1] != decisions[i])) {
      // create new sub diff blocks to segment the original diff block which we
      // further divided by running the linematch algorithm
      dp_s = rs_diff_alloc_new(curtab, dp_s, dp_s->df_next);
      dp_s->is_linematched = true;
      for (int j = 0; j < DB_COUNT; j++) {
        if (curtab->tp_diffbuf[j] != NULL) {
          dp_s->df_lnum[j] = line_numbers[j];
          dp_s->df_count[j] = 0;
        }
      }
    }
    for (size_t j = 0; j < ndiffs; j++) {
      if (decisions[i] & (1 << j)) {
        // will need to use the map here
        dp_s->df_count[outputmap[j]]++;
        line_numbers[outputmap[j]]++;
      }
    }
  }
  dp->is_linematched = true;
}

static void run_linematch_algorithm(diff_T *dp)
{
  // define buffers for diff algorithm
  mmfile_t diffbufs_mm[DB_COUNT];
  const mmfile_t *diffbufs[DB_COUNT];
  int diff_length[DB_COUNT];
  size_t ndiffs = 0;
  for (int i = 0; i < DB_COUNT; i++) {
    if (curtab->tp_diffbuf[i] != NULL) {
      if (dp->df_count[i] > 0) {
        // write the contents of the entire buffer to
        // diffbufs_mm[diffbuffers_count]
        diff_write_buffer(curtab->tp_diffbuf[i], &diffbufs_mm[ndiffs],
                          dp->df_lnum[i], dp->df_lnum[i] + dp->df_count[i] - 1);
      } else {
        diffbufs_mm[ndiffs].size = 0;
        diffbufs_mm[ndiffs].ptr = NULL;
      }

      diffbufs[ndiffs] = &diffbufs_mm[ndiffs];

      // keep track of the length of this diff block to pass it to the linematch
      // algorithm
      diff_length[ndiffs] = dp->df_count[i];

      // increment the amount of diff buffers we are passing to the algorithm
      ndiffs++;
    }
  }

  // we will get the output of the linematch algorithm in the format of an array
  // of integers (*decisions) and the length of that array (decisions_length)
  int *decisions = NULL;
  const bool iwhite = (diff_flags & (DIFF_IWHITEALL | DIFF_IWHITE)) > 0;
  size_t decisions_length = linematch_nbuffers(diffbufs, diff_length, ndiffs, &decisions, iwhite);

  for (size_t i = 0; i < ndiffs; i++) {
    XFREE_CLEAR(diffbufs_mm[i].ptr);
  }

  apply_linematch_results(dp, decisions_length, decisions);

  xfree(decisions);
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

/// Parse the diff anchors. If "check_only" is set, will only make sure the
/// syntax is correct.
static int parse_diffanchors(bool check_only, buf_T *buf, linenr_T *anchors, int *num_anchors)
{
  int i;
  char *dia = (*buf->b_p_dia == NUL) ? p_dia : buf->b_p_dia;

  buf_T *orig_curbuf = curbuf;
  win_T *orig_curwin = curwin;

  win_T *bufwin = NULL;
  if (check_only) {
    bufwin = curwin;
  } else {
    // Find the first window tied to this buffer and ignore the rest. Will
    // only matter for window-specific addresses like `.` or `''`.
    for (bufwin = firstwin; bufwin != NULL; bufwin = bufwin->w_next) {
      if (bufwin->w_buffer == buf && bufwin->w_p_diff) {
        break;
      }
    }
    if (bufwin == NULL && *dia != NUL) {
      // The buffer is hidden. Currently this is not supported due to the
      // edge cases of needing to decide if an address is window-specific
      // or not. We could add more checks in the future so we can detect
      // whether an address relies on curwin to make this more fleixble.
      emsg(_(e_diff_anchors_with_hidden_windows));
      return FAIL;
    }
  }

  for (i = 0; i < MAX_DIFF_ANCHORS && *dia != NUL; i++) {
    if (*dia == ',') {  // don't allow empty values
      return FAIL;
    }

    curbuf = buf;
    curwin = bufwin;
    const char *errormsg = NULL;
    linenr_T lnum = get_address(NULL, &dia, ADDR_LINES, check_only, true, false, 1, &errormsg);
    curbuf = orig_curbuf;
    curwin = orig_curwin;

    if (errormsg != NULL) {
      emsg(errormsg);
    }
    if (dia == NULL) {  // error detected
      return FAIL;
    }
    if (*dia != ',' && *dia != NUL) {
      return FAIL;
    }

    if (!check_only
        && (lnum == MAXLNUM || lnum <= 0 || lnum > buf->b_ml.ml_line_count + 1)) {
      emsg(_(e_invrange));
      return FAIL;
    }

    if (anchors != NULL) {
      anchors[i] = lnum;
    }

    if (*dia == ',') {
      dia++;
    }
  }
  if (i == MAX_DIFF_ANCHORS && *dia != NUL) {
    semsg(_(e_cannot_have_more_than_nr_diff_anchors), MAX_DIFF_ANCHORS);
    return FAIL;
  }
  if (num_anchors != NULL) {
    *num_anchors = i;
  }
  return OK;
}

/// used for simple inline diff algorithm
static diffline_change_T simple_diffline_change;

/// Mapping used for mapping from temporary mmfile created for inline diff back
/// to original buffer's line/col.
typedef struct {
  colnr_T byte_start;
  colnr_T num_bytes;
  int lineoff;
} linemap_entry_T;

/// Refine inline character-wise diff blocks to create a more human readable
/// highlight. Otherwise a naive diff under existing algorithms tends to create
/// a messy output with lots of small gaps.
/// It does this by merging adjacent long diff blocks if they are only separated
/// by a couple characters.
/// These are done by heuristics and can be further tuned.
static void diff_refine_inline_char_highlight(diff_T *dp_orig, garray_T *linemap, int idx1)
{
  // Perform multiple passes so that newly merged blocks will now be long
  // enough which may cause other previously unmerged gaps to be merged as
  // well.
  int pass = 1;
  do {
    bool has_unmerged_gaps = false;
    bool has_merged_gaps = false;
    diff_T *dp = dp_orig;
    while (dp != NULL && dp->df_next != NULL) {
      // Only use first buffer to calculate the gap because the gap is
      // unchanged text, which would be the same in all buffers.
      if (dp->df_lnum[idx1] + dp->df_count[idx1] - 1 >= linemap[idx1].ga_len
          || dp->df_next->df_lnum[idx1] - 1 >= linemap[idx1].ga_len) {
        dp = dp->df_next;
        continue;
      }

      // If the gap occurs over different lines, don't consider it
      linemap_entry_T *entry1 =
        &((linemap_entry_T *)linemap[idx1].ga_data)[dp->df_lnum[idx1]
                                                    + dp->df_count[idx1] - 1];
      linemap_entry_T *entry2 =
        &((linemap_entry_T *)linemap[idx1].ga_data)[dp->df_next->df_lnum[idx1] - 1];
      if (entry1->lineoff != entry2->lineoff) {
        dp = dp->df_next;
        continue;
      }

      linenr_T gap = dp->df_next->df_lnum[idx1] - (dp->df_lnum[idx1] + dp->df_count[idx1]);
      if (gap <= 3) {
        linenr_T max_df_count = 0;
        for (int i = 0; i < DB_COUNT; i++) {
          max_df_count = MAX(max_df_count, dp->df_count[i] + dp->df_next->df_count[i]);
        }

        if (max_df_count >= gap * 4) {
          // Merge current block with the next one. Don't advance the
          // pointer so we try the same merged block against the next
          // one.
          for (int i = 0; i < DB_COUNT; i++) {
            dp->df_count[i] = dp->df_next->df_lnum[i]
                              + dp->df_next->df_count[i] - dp->df_lnum[i];
          }
          diff_T *dp_next = dp->df_next;
          dp->df_next = dp_next->df_next;
          rs_clear_diffblock(dp_next);
          has_merged_gaps = true;
          continue;
        } else {
          has_unmerged_gaps = true;
        }
      }
      dp = dp->df_next;
    }
    if (!has_unmerged_gaps || !has_merged_gaps) {
      break;
    }
  } while (pass++ < 4);  // use limited number of passes to avoid excessive looping
}

/// Find the inline difference within a diff block among different buffers.  Do
/// this by splitting each block's content into characters or words, and then
/// use internal xdiff to calculate the per-character/word diff.  The result is
/// stored in dp instead of returned by the function.
static void diff_find_change_inline_diff(diff_T *dp)
{
  const int save_diff_algorithm = diff_algorithm;

  diffio_T dio = { 0 };
  ga_init(&dio.dio_diff.dout_ga, sizeof(diffhunk_T), 1000);

  // inline diff only supports internal algo
  dio.dio_internal = true;

  // always use indent-heuristics to slide diff splits along
  // whitespace
  diff_algorithm |= XDF_INDENT_HEURISTIC;

  // diff_read() has an implicit dependency on curtab->tp_first_diff
  diff_T *orig_diff = curtab->tp_first_diff;
  curtab->tp_first_diff = NULL;

  // diff_read() also uses curtab->tp_diffbuf to determine what's an active
  // buffer
  buf_T *(orig_diffbuf[DB_COUNT]);
  memcpy(orig_diffbuf, curtab->tp_diffbuf, sizeof(orig_diffbuf));

  garray_T linemap[DB_COUNT];
  garray_T file1_str;
  garray_T file2_str;

  // Buffers to populate mmfile 1/2 that would be passed to xdiff as memory
  // files. Use a grow array as it is not obvious how much exact space we
  // need.
  ga_init(&file1_str, 1, 1024);
  ga_init(&file2_str, 1, 1024);

  // Line map to map from generated mmfiles' line numbers back to original
  // diff blocks' locations. Need this even for char diff because not all
  // characters are 1-byte long / ASCII.
  for (int i = 0; i < DB_COUNT; i++) {
    ga_init(&linemap[i], sizeof(linemap_entry_T), 128);
  }

  int file1_idx = -1;
  for (int i = 0; i < DB_COUNT; i++) {
    dio.dio_diff.dout_ga.ga_len = 0;

    buf_T *buf = curtab->tp_diffbuf[i];
    if (buf == NULL || buf->b_ml.ml_mfp == NULL) {
      continue;  // skip buffer that isn't loaded
    }
    if (dp->df_count[i] == 0) {
      // skip buffers that don't have any texts in this block so we don't
      // end up marking the entire block as modified in multi-buffer diff
      curtab->tp_diffbuf[i] = NULL;
      continue;
    }

    if (file1_idx == -1) {
      file1_idx = i;
    }

    garray_T *curstr = (file1_idx != i) ? &file2_str : &file1_str;

    linenr_T numlines = 0;
    curstr->ga_len = 0;

    // Split each line into chars/words and populate fake file buffer as
    // newline-delimited tokens as that's what xdiff requires.
    for (int off = 0; off < dp->df_count[i]; off++) {
      char *curline = ml_get_buf(curtab->tp_diffbuf[i], dp->df_lnum[i] + off);

      bool in_keyword = false;

      // iwhiteeol support vars
      bool last_white = false;
      int eol_ga_len = -1;
      int eol_linemap_len = -1;
      int eol_numlines = -1;

      char *s = curline;
      while (*s != NUL) {
        bool new_in_keyword = false;
        if (diff_flags & DIFF_INLINE_WORD) {
          // Always use the first buffer's 'iskeyword' to have a
          // consistent diff.
          // For multibyte chars, only treat alphanumeric chars
          // (class 2) as "word", as other classes such as emojis and
          // CJK ideographs do not usually benefit from word diff as
          // Vim doesn't have a good way to segment them.
          new_in_keyword = (mb_get_class_tab(s, curtab->tp_diffbuf[file1_idx]->b_chartab) == 2);
        }
        if (in_keyword && !new_in_keyword) {
          ga_append(curstr, NL);
          numlines++;
        }

        if (ascii_iswhite(*s)) {
          if (diff_flags & DIFF_IWHITEALL) {
            in_keyword = false;
            s = skipwhite(s);
            continue;
          } else if ((diff_flags & DIFF_IWHITEEOL) || (diff_flags & DIFF_IWHITE)) {
            if (!last_white) {
              eol_ga_len = curstr->ga_len;
              eol_linemap_len = linemap[i].ga_len;
              eol_numlines = numlines;
              last_white = true;
            }
          }
        } else {
          if ((diff_flags & DIFF_IWHITEEOL) || (diff_flags & DIFF_IWHITE)) {
            last_white = false;
            eol_ga_len = -1;
            eol_linemap_len = -1;
            eol_numlines = -1;
          }
        }

        int char_len = 1;
        if (*s == NL) {
          // NL is internal substitute for NUL
          ga_append(curstr, NUL);
        } else {
          char_len = utfc_ptr2len(s);

          if (ascii_iswhite(*s) && (diff_flags & DIFF_IWHITE)) {
            // Treat the entire white space span as a single char.
            char_len = (int)(skipwhite(s) - s);
          }

          if (diff_flags & DIFF_ICASE) {
            // xdiff doesn't support ignoring case, fold-case the text manually.
            int c = utf_ptr2char(s);
            int c_len = utf_char2len(c);
            c = utf_fold(c);
            char cbuf[MB_MAXBYTES + 1];
            int c_fold_len = utf_char2bytes(c, cbuf);
            ga_concat_len(curstr, cbuf, (size_t)c_fold_len);
            if (char_len > c_len) {
              // There may be remaining composing characters. Write those back in.
              // Composing characters don't need case folding.
              ga_concat_len(curstr, s + c_len, (size_t)(char_len - c_len));
            }
          } else {
            ga_concat_len(curstr, s, (size_t)char_len);
          }
        }

        if (!new_in_keyword) {
          ga_append(curstr, NL);
          numlines++;
        }

        if (!new_in_keyword || (new_in_keyword && !in_keyword)) {
          // create a new mapping entry from the xdiff mmfile back to
          // original line/col.
          linemap_entry_T linemap_entry = {
            .lineoff = off,
            .byte_start = (colnr_T)(s - curline),
            .num_bytes = char_len,
          };
          GA_APPEND(linemap_entry_T, &linemap[i], linemap_entry);
        } else {
          // Still inside a keyword. Just increment byte count but
          // don't make a new entry.
          // linemap always has at least one entry here
          ((linemap_entry_T *)linemap[i].ga_data)[linemap[i].ga_len - 1].num_bytes += char_len;
        }

        in_keyword = new_in_keyword;
        s += char_len;
      }
      if (in_keyword) {
        ga_append(curstr, NL);
        numlines++;
      }

      if ((diff_flags & DIFF_IWHITEEOL) || (diff_flags & DIFF_IWHITE)) {
        // Need to trim trailing whitespace. Do this simply by
        // resetting arrays back to before we encountered them.
        if (eol_ga_len != -1) {
          curstr->ga_len = eol_ga_len;
          linemap[i].ga_len = eol_linemap_len;
          numlines = eol_numlines;
        }
      }

      if (!(diff_flags & DIFF_IWHITEALL)) {
        // Add an empty line token mapped to the end-of-line in the
        // original file. This helps diff newline differences among
        // files, which will be visualized when using 'list' as the eol
        // listchar will be highlighted.
        ga_append(curstr, NL);
        numlines++;

        linemap_entry_T linemap_entry = {
          .lineoff = off,
          .byte_start = (colnr_T)(s - curline),
          .num_bytes = sizeof(NL),
        };
        GA_APPEND(linemap_entry_T, &linemap[i], linemap_entry);
      }
    }

    if (file1_idx != i) {
      dio.dio_new.din_mmfile.ptr = (char *)curstr->ga_data;
      dio.dio_new.din_mmfile.size = curstr->ga_len;
    } else {
      dio.dio_orig.din_mmfile.ptr = (char *)curstr->ga_data;
      dio.dio_orig.din_mmfile.size = curstr->ga_len;
    }
    if (file1_idx != i) {
      // Perform diff with first file and read the results
      int diff_status = diff_file_internal(&dio);
      if (diff_status == FAIL) {
        goto done;
      }

      rs_diff_read(0, i, &dio);
      clear_diffout(&dio.dio_diff);
    }
  }
  diff_T *new_diff = curtab->tp_first_diff;

  if (diff_flags & DIFF_INLINE_CHAR && file1_idx != -1) {
    diff_refine_inline_char_highlight(new_diff, linemap, file1_idx);
  }

  // After the diff, use the linemap to obtain the original line/col of the
  // changes and cache them in dp.
  dp->df_changes.ga_len = 0;  // this should already be zero
  for (; new_diff != NULL; new_diff = new_diff->df_next) {
    diffline_change_T change = { 0 };
    for (int i = 0; i < DB_COUNT; i++) {
      if (new_diff->df_lnum[i] <= 0) {  // should never be < 0. Checking just for safety
        continue;
      }
      linenr_T diff_lnum = new_diff->df_lnum[i] - 1;  // use zero-index
      linenr_T diff_lnum_end = diff_lnum + new_diff->df_count[i];

      if (diff_lnum >= linemap[i].ga_len) {
        change.dc_start[i] = MAXCOL;
        change.dc_start_lnum_off[i] = INT_MAX;
      } else {
        change.dc_start[i] = ((linemap_entry_T *)linemap[i].ga_data)[diff_lnum].byte_start;
        change.dc_start_lnum_off[i] = ((linemap_entry_T *)linemap[i].ga_data)[diff_lnum].lineoff;
      }

      if (diff_lnum == diff_lnum_end) {
        change.dc_end[i] = change.dc_start[i];
        change.dc_end_lnum_off[i] = change.dc_start_lnum_off[i];
      } else if (diff_lnum_end - 1 >= linemap[i].ga_len) {
        change.dc_end[i] = MAXCOL;
        change.dc_end_lnum_off[i] = INT_MAX;
      } else {
        change.dc_end[i] = ((linemap_entry_T *)linemap[i].ga_data)[diff_lnum_end - 1].byte_start +
                           ((linemap_entry_T *)linemap[i].ga_data)[diff_lnum_end - 1].num_bytes;
        change.dc_end_lnum_off[i] = ((linemap_entry_T *)linemap[i].ga_data)[diff_lnum_end -
                                                                            1].lineoff;
      }
    }
    GA_APPEND(diffline_change_T, &dp->df_changes, change);
  }

done:
  diff_algorithm = save_diff_algorithm;

  dp->has_changes = true;

  rs_diff_clear(curtab);
  curtab->tp_first_diff = orig_diff;
  memcpy(curtab->tp_diffbuf, orig_diffbuf, sizeof(orig_diffbuf));

  ga_clear(&file1_str);
  ga_clear(&file2_str);
  // No need to clear dio.dio_orig/dio_new because they were referencing
  // strings that are now cleared.
  clear_diffout(&dio.dio_diff);
  for (int i = 0; i < DB_COUNT; i++) {
    ga_clear(&linemap[i]);
  }
}

/// "dp" and "do" commands -- thin wrapper calling Rust rs_nv_diffgetput.
void nv_diffgetput(bool put, size_t count)
{
  rs_nv_diffgetput(put, count);
}

/// ":diffget" and ":diffput"
///
/// @param eap
void ex_diffgetput(exarg_T *eap)
{
  int idx_other;

  // Find the current buffer in the list of diff buffers.
  int idx_cur = rs_diff_buf_idx_tp(curbuf, curtab);
  if (idx_cur == DB_COUNT) {
    emsg(_("E99: Current buffer is not in diff mode"));
    return;
  }

  if (*eap->arg == NUL) {
    bool found_not_ma = false;
    // No argument: Find the other buffer in the list of diff buffers.
    for (idx_other = 0; idx_other < DB_COUNT; idx_other++) {
      if ((curtab->tp_diffbuf[idx_other] != curbuf)
          && (curtab->tp_diffbuf[idx_other] != NULL)) {
        if ((eap->cmdidx != CMD_diffput)
            || MODIFIABLE(curtab->tp_diffbuf[idx_other])) {
          break;
        }
        found_not_ma = true;
      }
    }

    if (idx_other == DB_COUNT) {
      if (found_not_ma) {
        emsg(_("E793: No other buffer in diff mode is modifiable"));
      } else {
        emsg(_("E100: No other buffer in diff mode"));
      }
      return;
    }

    // Check that there isn't a third buffer in the list
    for (int i = idx_other + 1; i < DB_COUNT; i++) {
      if ((curtab->tp_diffbuf[i] != curbuf)
          && (curtab->tp_diffbuf[i] != NULL)
          && ((eap->cmdidx != CMD_diffput)
              || MODIFIABLE(curtab->tp_diffbuf[i]))) {
        emsg(_("E101: More than two buffers in diff mode, don't know "
               "which one to use"));
        return;
      }
    }
  } else {
    // Buffer number or pattern given. Ignore trailing white space.
    char *p = eap->arg + strlen(eap->arg);
    while (p > eap->arg && ascii_iswhite(p[-1])) {
      p--;
    }

    int i;
    for (i = 0; ascii_isdigit(eap->arg[i]) && eap->arg + i < p; i++) {}

    if (eap->arg + i == p) {
      // digits only
      i = (int)atol(eap->arg);
    } else {
      i = buflist_findpat(eap->arg, p, false, true, false);

      if (i < 0) {
        // error message already given
        return;
      }
    }
    buf_T *buf = buflist_findnr(i);

    if (buf == NULL) {
      semsg(_("E102: Can't find buffer \"%s\""), eap->arg);
      return;
    }

    if (buf == curbuf) {
      // nothing to do
      return;
    }
    idx_other = rs_diff_buf_idx_tp(buf, curtab);

    if (idx_other == DB_COUNT) {
      semsg(_("E103: Buffer \"%s\" is not in diff mode"), eap->arg);
      return;
    }
  }

  diff_busy = true;

  // When no range given include the line above or below the cursor.
  if (eap->addr_count == 0) {
    // Make it possible that ":diffget" on the last line gets line below
    // the cursor line when there is no difference above the cursor.
    int linestatus = 0;
    if (eap->line1 == curbuf->b_ml.ml_line_count
        && (rs_diff_check_with_linestatus(curwin, eap->line1, &linestatus) == 0
            && linestatus == 0)
        && (eap->line1 == 1
            || (rs_diff_check_with_linestatus(curwin, eap->line1 - 1, &linestatus) >= 0
                && linestatus == 0))) {
      eap->line2++;
    } else if (eap->line1 > 0) {
      eap->line1--;
    }
  }

  aco_save_T aco;

  if (eap->cmdidx != CMD_diffget) {
    // Need to make the other buffer the current buffer to be able to make
    // changes in it.

    // Set curwin/curbuf to buf and save a few things.
    aucmd_prepbuf(&aco, curtab->tp_diffbuf[idx_other]);
  }

  const int idx_from = eap->cmdidx == CMD_diffget ? idx_other : idx_cur;
  const int idx_to = eap->cmdidx == CMD_diffget ? idx_cur : idx_other;

  // May give the warning for a changed buffer here, which can trigger the
  // FileChangedRO autocommand, which may do nasty things and mess
  // everything up.
  if (!curbuf->b_changed) {
    change_warning(curbuf, 0);
    if (rs_diff_buf_idx_tp(curbuf, curtab) != idx_to) {
      emsg(_("E787: Buffer changed unexpectedly"));
      goto theend;
    }
  }

  diffgetput(eap->addr_count, idx_cur, idx_from, idx_to, eap->line1, eap->line2);

  // restore curwin/curbuf and a few other things
  if (eap->cmdidx != CMD_diffget) {
    // Syncing undo only works for the current buffer, but we change
    // another buffer.  Sync undo if the command was typed.  This isn't
    // 100% right when ":diffput" is used in a function or mapping.
    if (KeyTyped) {
      u_sync(false);
    }
    aucmd_restbuf(&aco);
  }

theend:
  diff_busy = false;

  if (diff_need_update) {
    rs_diff_ex_diffupdate(NULL);
  }

  // Check that the cursor is on a valid character and update its
  // position.  When there were filler lines the topline has become
  // invalid.
  check_cursor(curwin);
  changed_line_abv_curs();

  // If all diffs are gone, update folds in all diff windows.
  if (curtab->tp_first_diff == NULL) {
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_p_diff && wp->w_p_fdm[0] == 'd' && wp->w_p_fen) {
        rs_foldUpdateAll(wp);
      }
    }
  }

  if (diff_need_update) {
    // redraw already done by ex_diffupdate()
    diff_need_update = false;
  } else {
    // Also need to redraw the other buffers.
    diff_redraw(false);
    apply_autocmds(EVENT_DIFFUPDATED, NULL, NULL, false, curbuf);
  }
}

/// Apply diffget/diffput to buffers and diffblocks
///
/// @param idx_cur   index of "curbuf" before aucmd_prepbuf() in the list of diff buffers
/// @param idx_from  index of the buffer to read from in the list of diff buffers
/// @param idx_to    index of the buffer to modify in the list of diff buffers
static void diffgetput(const int addr_count, const int idx_cur, const int idx_from,
                       const int idx_to, const linenr_T line1, const linenr_T line2)
{
  linenr_T off = 0;
  diff_T *dprev = NULL;

  for (diff_T *dp = curtab->tp_first_diff; dp != NULL;) {
    if (!addr_count) {
      // Handle the case with adjacent diff blocks (e.g. using linematch
      // or anchors) at/above the cursor. Since a range wasn't specified,
      // we just want to grab one diff block rather than all of them in
      // the vicinity.
      while (dp->df_next
             && dp->df_next->df_lnum[idx_cur] == dp->df_lnum[idx_cur] + dp->df_count[idx_cur]
             && dp->df_next->df_lnum[idx_cur] == line1 + off + 1) {
        dprev = dp;
        dp = dp->df_next;
      }
    }

    if (dp->df_lnum[idx_cur] > line2 + off) {
      // past the range that was specified
      break;
    }
    diff_T dfree = { 0 };
    bool did_free = false;
    linenr_T lnum = dp->df_lnum[idx_to];
    linenr_T count = dp->df_count[idx_to];

    if ((dp->df_lnum[idx_cur] + dp->df_count[idx_cur] > line1 + off)
        && (u_save(lnum - 1, lnum + count) != FAIL)) {
      // Inside the specified range and saving for undo worked.
      linenr_T start_skip = 0;
      linenr_T end_skip = 0;

      if (addr_count > 0) {
        // A range was specified: check if lines need to be skipped.
        start_skip = line1 + off - dp->df_lnum[idx_cur];
        if (start_skip > 0) {
          // range starts below start of current diff block
          if (start_skip > count) {
            lnum += count;
            count = 0;
          } else {
            count -= start_skip;
            lnum += start_skip;
          }
        } else {
          start_skip = 0;
        }

        end_skip = dp->df_lnum[idx_cur] + dp->df_count[idx_cur] - 1
                   - (line2 + off);

        if (end_skip > 0) {
          // range ends above end of current/from diff block
          if (idx_cur == idx_from) {
            // :diffput
            count = MIN(count, dp->df_count[idx_cur] - start_skip - end_skip);
          } else {
            // :diffget
            count -= end_skip;
            end_skip = MAX(dp->df_count[idx_from] - start_skip - count, 0);
          }
        } else {
          end_skip = 0;
        }
      }

      bool buf_empty = buf_is_empty(curbuf);
      int added = 0;

      for (int i = 0; i < count; i++) {
        // remember deleting the last line of the buffer
        buf_empty = curbuf->b_ml.ml_line_count == 1;
        if (ml_delete(lnum) == OK) {
          added--;
        }
      }

      for (int i = 0; i < dp->df_count[idx_from] - start_skip - end_skip; i++) {
        linenr_T nr = dp->df_lnum[idx_from] + start_skip + i;
        if (nr > curtab->tp_diffbuf[idx_from]->b_ml.ml_line_count) {
          break;
        }
        char *p = xstrdup(ml_get_buf(curtab->tp_diffbuf[idx_from], nr));
        ml_append(lnum + i - 1, p, 0, false);
        xfree(p);
        added++;
        if (buf_empty && (curbuf->b_ml.ml_line_count == 2)) {
          // Added the first line into an empty buffer, need to
          // delete the dummy empty line.
          // This has a side effect of incrementing curbuf->deleted_bytes,
          // which results in inaccurate reporting of the byte count of
          // previous contents in buffer-update events.
          buf_empty = false;
          ml_delete(2);
        }
      }
      linenr_T new_count = dp->df_count[idx_to] + added;
      dp->df_count[idx_to] = new_count;

      if ((start_skip == 0) && (end_skip == 0)) {
        // Check if there are any other buffers and if the diff is
        // equal in them.
        int i;
        for (i = 0; i < DB_COUNT; i++) {
          if ((curtab->tp_diffbuf[i] != NULL)
              && (i != idx_from)
              && (i != idx_to)
              && !rs_diff_equal_entry_full(dp, idx_from, i)) {
            break;
          }
        }

        if (i == DB_COUNT) {
          // delete the diff entry, the buffers are now equal here
          dfree = *dp;
          did_free = true;
          dp = rs_diff_free(curtab, dprev, dp);
        }
      }

      if (added != 0) {
        // Adjust marks.  This will change the following entries!
        mark_adjust(lnum, lnum + count - 1, MAXLNUM, added, kExtmarkNOOP);
        if (curwin->w_cursor.lnum >= lnum) {
          // Adjust the cursor position if it's in/after the changed
          // lines.
          if (curwin->w_cursor.lnum >= lnum + count) {
            curwin->w_cursor.lnum += added;
          } else if (added < 0) {
            curwin->w_cursor.lnum = lnum;
          }
        }
      }
      extmark_adjust(curbuf, lnum, lnum + count - 1, MAXLNUM, added, kExtmarkUndo);
      changed_lines(curbuf, lnum, 0, lnum + count, added, true);

      if (did_free) {
        // Diff is deleted, update folds in other windows.
        rs_diff_fold_update(&dfree, idx_to);
      }

      // mark_adjust() may have made "dp" invalid.  We don't know where
      // to continue then, bail out.
      if (added != 0 && !rs_valid_diff(dp)) {
        break;
      }

      if (!did_free) {
        // mark_adjust() may have changed the count in a wrong way
        dp->df_count[idx_to] = new_count;
      }

      // When changing the current buffer, keep track of line numbers
      if (idx_cur == idx_to) {
        off += added;
      }
    }

    // If before the range or not deleted, go to next diff.
    if (!did_free) {
      dprev = dp;
      dp = dp->df_next;
    }
  }
}

/// Checks that the buffer is in diff-mode.

/// Callback function for the xdl_diff() function.
/// Thin wrapper -- implementation moved to Rust (rs_xdiff_out in viml.rs).
static int xdiff_out(int start_a, int count_a, int start_b, int count_b, void *priv)
{
  return rs_xdiff_out(start_a, count_a, start_b, count_b, priv);
}

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
int nvim_get_db_count(void) { return DB_COUNT; }
bool nvim_diffblock_is_linematched(diff_T *dp) { if (dp == NULL) { return false; } return dp->is_linematched; }
void nvim_diffblock_set_linematched(diff_T *dp, bool val) { if (dp == NULL) { return; } dp->is_linematched = val; }
bool nvim_diffblock_has_changes(diff_T *dp) { if (dp == NULL) { return false; } return dp->df_changes.ga_len > 0; }
diff_T *nvim_tabpage_get_first_diff(tabpage_T *tp) { if (tp == NULL) { return NULL; } return tp->tp_first_diff; }
buf_T *nvim_tabpage_get_diffbuf(tabpage_T *tp, int idx) { if (tp == NULL || idx < 0 || idx >= DB_COUNT) { return NULL; } return tp->tp_diffbuf[idx]; }
bool nvim_tabpage_is_diff_invalid(tabpage_T *tp) { if (tp == NULL) { return true; } return tp->tp_diff_invalid; }
void nvim_tabpage_set_diff_invalid(tabpage_T *tp, int val) { if (tp != NULL) { tp->tp_diff_invalid = val != 0; } }
void nvim_tabpage_set_diff_update(tabpage_T *tp, int val) { if (tp != NULL) { tp->tp_diff_update = val != 0; } }
diff_T *nvim_diff_alloc_new(tabpage_T *tp, diff_T *prev, diff_T *next) { diff_T *dnew = xmalloc(sizeof(diff_T)); CLEAR_POINTER(dnew); dnew->df_next = next; if (prev == NULL) { tp->tp_first_diff = dnew; } else { prev->df_next = dnew; } return dnew; }
int nvim_diff_is_busy(void) { return diff_busy ? 1 : 0; }
int nvim_diffblock_get_changes_len(diff_T *dp) { if (dp == NULL) { return 0; } return dp->df_changes.ga_len; }
diffline_change_T *nvim_diffblock_get_change(diff_T *dp, int change_idx) { if (dp == NULL || change_idx < 0 || change_idx >= dp->df_changes.ga_len) { return NULL; } return &((diffline_change_T *)dp->df_changes.ga_data)[change_idx]; }
void nvim_diff_write_buffer(buf_T *buf, void *m, linenr_T start, linenr_T end) { diff_write_buffer(buf, (mmfile_t *)m, start, end); }
void nvim_curtab_set_diffbuf(int idx, buf_T *buf) { if (idx >= 0 && idx < DB_COUNT) { curtab->tp_diffbuf[idx] = buf; } }
void nvim_tabpage_set_diffbuf(tabpage_T *tp, int idx, buf_T *buf) { if (tp != NULL && idx >= 0 && idx < DB_COUNT) { tp->tp_diffbuf[idx] = buf; } }
void nvim_tabpage_set_first_diff(tabpage_T *tp, diff_T *dp) { if (tp != NULL) { tp->tp_first_diff = dp; } }
void nvim_diff_set_next(diff_T *dp, diff_T *next) { if (dp != NULL) { dp->df_next = next; } }
void nvim_diffblock_clear_and_free(diff_T *dp) { if (dp != NULL) { ga_clear(&dp->df_changes); xfree(dp); } }
void nvim_diffblock_init_changes(diff_T *dp) { if (dp != NULL) { dp->has_changes = false; ga_init(&dp->df_changes, sizeof(diffline_change_T), 20); } }
void nvim_diffblock_init_new(diff_T *dp) { if (dp != NULL) { dp->is_linematched = false; dp->has_changes = false; ga_init(&dp->df_changes, sizeof(diffline_change_T), 20); } }
void nvim_set_need_diff_redraw(bool val) { need_diff_redraw = val; }
int nvim_diff_get_linematch_lines(void) { return linematch_lines; }
int nvim_diff_get_diff_flags(void) { return diff_flags; }
void nvim_diff_redraw(bool dofold) { diff_redraw(dofold); }
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
bool nvim_diff_get_need_scrollbind(void) { return diff_need_scrollbind; }
void nvim_diff_set_need_scrollbind(bool val) { diff_need_scrollbind = val; }
linenr_T nvim_diff_maxlnum(void) { return MAXLNUM; }
int nvim_diff_get_algorithm(void) { return diff_algorithm; }
void nvim_diff_set_options(int flags, int context, int linematch, int foldcol, int algorithm) { diff_flags = flags; diff_context = context; linematch_lines = linematch; diff_foldcolumn = foldcol; diff_algorithm = algorithm; }
void nvim_diff_check_scrollbind(void) { check_scrollbind(0, 0); }
int nvim_diff_parse_diffanchors(void) { return parse_diffanchors(true, curbuf, NULL, NULL); }
const char *nvim_diff_get_p_dip(void) { return p_dip; }
void *nvim_diffio_new(bool use_internal) { diffio_T *dio = xcalloc(1, sizeof(diffio_T)); dio->dio_internal = use_internal ? 1 : 0; return dio; }
void nvim_diffio_free(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; xfree(dio); }
bool nvim_diffio_is_internal(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; return dio != NULL && dio->dio_internal; }
void nvim_diffio_init_ga(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio != NULL) { ga_init(&dio->dio_diff.dout_ga, sizeof(diffhunk_T), 100); } }
bool nvim_diffio_alloc_tempfiles(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return false; } dio->dio_orig.din_fname = vim_tempname(); dio->dio_new.din_fname = vim_tempname(); dio->dio_diff.dout_fname = vim_tempname(); return (dio->dio_orig.din_fname != NULL && dio->dio_new.din_fname != NULL && dio->dio_diff.dout_fname != NULL); }
void nvim_diffio_free_tempfiles(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return; } xfree(dio->dio_orig.din_fname); xfree(dio->dio_new.din_fname); xfree(dio->dio_diff.dout_fname); dio->dio_orig.din_fname = NULL; dio->dio_new.din_fname = NULL; dio->dio_diff.dout_fname = NULL; }
int nvim_diffio_write_orig(void *dio_ptr, buf_T *buf, linenr_T start, linenr_T end) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL || buf == NULL) { return FAIL; } return diff_write(buf, &dio->dio_orig, start, end); }
int nvim_diffio_write_new(void *dio_ptr, buf_T *buf, linenr_T start, linenr_T end) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL || buf == NULL) { return FAIL; } return diff_write(buf, &dio->dio_new, start, end); }
int nvim_diffio_run_diff(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return FAIL; } return diff_file(dio); }
int nvim_diffio_check_external(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio == NULL) { return FAIL; } return check_external_diff(dio); }
void nvim_diffio_clear_new(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio != NULL) { clear_diffin(&dio->dio_new); } }
void nvim_diffio_clear_output(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio != NULL) { clear_diffout(&dio->dio_diff); } }
void nvim_diffio_clear_orig(void *dio_ptr) { diffio_T *dio = (diffio_T *)dio_ptr; if (dio != NULL) { clear_diffin(&dio->dio_orig); } }
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
int nvim_diff_parse_buf_anchors(buf_T *buf, linenr_T *anchors, int max_anchors) { if (buf == NULL) { return -1; } int num = 0; if (parse_diffanchors(false, buf, anchors, &num) != OK) { return -1; } return num; }
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
void nvim_diff_run_linematch(diff_T *dp) { run_linematch_algorithm(dp); }
bool nvim_diffblock_get_has_changes(diff_T *dp) { if (dp == NULL) { return false; } return dp->has_changes; }
void nvim_diffblock_set_has_changes(diff_T *dp, bool val) { if (dp != NULL) { dp->has_changes = val; } }
void nvim_diffblock_reset_changes_len(diff_T *dp) { if (dp != NULL) { dp->df_changes.ga_len = 0; } }
diffline_change_T *nvim_diff_get_simple_change(void) { return &simple_diffline_change; }
void nvim_diff_compute_inline(diff_T *dp) { if (dp != NULL) { diff_find_change_inline_diff(dp); } }
int nvim_diffchange_get_start_lnum_off(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_start_lnum_off[idx]; }
int nvim_diffchange_get_end_lnum_off(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_end_lnum_off[idx]; }
colnr_T nvim_diffchange_get_start(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_start[idx]; }
colnr_T nvim_diffchange_get_end(diffline_change_T *change, int idx) { if (change == NULL || idx < 0 || idx >= DB_COUNT) { return 0; } return change->dc_end[idx]; }
bool nvim_diff_is_simple_change(diffline_change_T *change) { return change == &simple_diffline_change; }
const char *nvim_diff_skipwhite(const char *p) { return skipwhite(p); }
// Phase 1 accessors: xdiff_out and f_diff_filler
void nvim_diffout_append_hunk(void *dout, linenr_T lnum_orig, int count_orig, linenr_T lnum_new, int count_new) { diffout_T *d = (diffout_T *)dout; if (d == NULL) { return; } GA_APPEND(diffhunk_T, &(d->dout_ga), ((diffhunk_T){ .lnum_orig = lnum_orig, .count_orig = count_orig, .lnum_new = lnum_new, .count_new = count_new, })); }
linenr_T nvim_diff_tv_get_lnum(typval_T *argvars) { return tv_get_lnum(argvars); }
// Phase 2 accessors: nv_diffgetput and ex_diffthis
void nvim_vim_beep_operator(void) { vim_beep(kOptBoFlagOperator); }
void nvim_diff_win_options(win_T *wp, bool addbuf) { diff_win_options(wp, addbuf); }
void nvim_call_ex_diffgetput(int cmdidx, const char *arg, int addr_count, linenr_T line1, linenr_T line2) { exarg_T ea; CLEAR_FIELD(ea); ea.cmdidx = (cmdidx_T)cmdidx; ea.arg = (char *)arg; ea.addr_count = addr_count; ea.line1 = line1; ea.line2 = line2; ex_diffgetput(&ea); }
// Phase 3 accessors: f_diff_hlID
int64_t nvim_curbuf_changedtick_i64(void) { return (int64_t)buf_get_changedtick(curbuf); }
int nvim_diff_tv_get_number_idx(typval_T *argvars, int idx) { return (int)tv_get_number(&argvars[idx]); }
int nvim_diff_hlf_add(void) { return (int)HLF_ADD; }
int nvim_diff_hlf_chd(void) { return (int)HLF_CHD; }
int nvim_diff_hlf_txd(void) { return (int)HLF_TXD; }
int nvim_diff_hlf_txa(void) { return (int)HLF_TXA; }
int nvim_diff_diffline_num_changes(diffline_T *dl) { return dl ? dl->num_changes : 0; }
int nvim_diff_diffline_bufidx(diffline_T *dl) { return dl ? dl->bufidx : 0; }
diffline_change_T *nvim_diff_diffline_get_change(diffline_T *dl, int i) { if (!dl || i < 0 || i >= dl->num_changes) { return NULL; } return &dl->changes[i]; }
colnr_T nvim_diff_change_dc_start(diffline_change_T *dc, int idx) { if (!dc || idx < 0 || idx >= DB_COUNT) { return 0; } return dc->dc_start[idx]; }
colnr_T nvim_diff_change_dc_end(diffline_change_T *dc, int idx) { if (!dc || idx < 0 || idx >= DB_COUNT) { return 0; } return dc->dc_end[idx]; }

