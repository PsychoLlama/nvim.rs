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

/// Structure passed around between undofile functions.
typedef struct {
  buf_T *bi_buf;
  FILE *bi_fp;
} bufinfo_T;

// Rust FFI function declarations
extern bool rs_bufIsChanged(buf_T *buf);
extern bool rs_anyBufIsChanged(void);
extern bool rs_curbufIsChanged(void);
extern void rs_u_clearall(buf_T *buf);
extern void rs_u_clearline(buf_T *buf);
extern void rs_u_freeentry(u_entry_T *uep, int n);
extern void rs_u_freeentries(buf_T *buf, u_header_T *uhp, u_header_T **uhpp);
extern void rs_u_freeheader(buf_T *buf, u_header_T *uhp, u_header_T **uhpp);
extern void rs_u_freebranch(buf_T *buf, u_header_T *uhp, u_header_T **uhpp);
extern u_entry_T *rs_u_get_headentry(buf_T *buf);
extern void rs_u_getbot(buf_T *buf);
extern void rs_u_blockfree(buf_T *buf);
extern void rs_u_sync(bool force);
extern void rs_u_clearallandblockfree(buf_T *buf);
extern void rs_u_unch_branch(u_header_T *uhp);
extern void rs_u_unchanged(buf_T *buf);
extern void rs_u_update_save_nr(buf_T *buf);
extern void rs_u_free_uhp(u_header_T *uhp);
extern bool rs_undo_allowed(buf_T *buf);
extern void rs_ex_undojoin(void);
extern void rs_u_undo(int count);
extern void rs_u_redo(int count);
extern bool rs_u_undo_and_forget(int count, bool do_buf_event);
extern void rs_u_doit(int startcount, bool quiet, bool do_buf_event);
extern int rs_u_savecommon(buf_T *buf, linenr_T top, linenr_T bot, linenr_T newbot, bool reload);
extern int rs_u_save_cursor(void);
extern int rs_u_save(linenr_T top, linenr_T bot);
extern int rs_u_save_buf(buf_T *buf, linenr_T top, linenr_T bot);
extern int rs_u_savesub(linenr_T lnum);
extern int rs_u_inssub(linenr_T lnum);
extern int rs_u_savedel(linenr_T lnum, linenr_T nlines);
extern void rs_u_find_first_changed(void);
extern u_header_T *rs_u_force_get_undo_header(buf_T *buf);
extern void rs_u_undoline(void);
extern void rs_undo_time(int step, bool sec, bool file, bool absolute);
extern void rs_u_compute_hash(buf_T *buf, uint8_t *hash);
extern void rs_u_write_undo(const char *name, bool forceit, buf_T *buf, const uint8_t *hash);
extern void rs_u_read_undo(const char *name, const uint8_t *hash, const char *orig_name);
extern void rs_ex_undolist(exarg_T *eap);
extern list_T *rs_u_eval_tree(buf_T *buf, const u_header_T *first_uhp);
extern char *rs_f_undofile(const char *fname);

#include "undo.c.generated.h"

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

/// Save the current line for both the "u" and "U" command.
/// Careful: may trigger autocommands that reload the buffer.
/// Returns OK or FAIL.
int u_save_cursor(void)
{
  return rs_u_save_cursor();
}

/// Save the lines between "top" and "bot" for both the "u" and "U" command.
/// "top" may be 0 and "bot" may be curbuf->b_ml.ml_line_count + 1.
/// Careful: may trigger autocommands that reload the buffer.
/// Returns FAIL when lines could not be saved, OK otherwise.
int u_save(linenr_T top, linenr_T bot)
{
  return rs_u_save(top, bot);
}

int u_save_buf(buf_T *buf, linenr_T top, linenr_T bot)
{
  return rs_u_save_buf(buf, top, bot);
}

/// Save the line "lnum" (used by ":s" and "~" command).
/// The line is replaced, so the new bottom line is lnum + 1.
/// Careful: may trigger autocommands that reload the buffer.
/// Returns FAIL when lines could not be saved, OK otherwise.
int u_savesub(linenr_T lnum)
{
  return rs_u_savesub(lnum);
}

/// A new line is inserted before line "lnum" (used by :s command).
/// The line is inserted, so the new bottom line is lnum + 1.
/// Careful: may trigger autocommands that reload the buffer.
/// Returns FAIL when lines could not be saved, OK otherwise.
int u_inssub(linenr_T lnum)
{
  return rs_u_inssub(lnum);
}

/// Save the lines "lnum" - "lnum" + nlines (used by delete command).
/// The lines are deleted, so the new bottom line is lnum, unless the buffer
/// becomes empty.
/// Careful: may trigger autocommands that reload the buffer.
/// Returns FAIL when lines could not be saved, OK otherwise.
int u_savedel(linenr_T lnum, linenr_T nlines)
{
  return rs_u_savedel(lnum, nlines);
}

/// Return true when undo is allowed. Otherwise print an error message and
/// return false.
///
/// @return true if undo is allowed.
bool undo_allowed(buf_T *buf)
{
  return rs_undo_allowed(buf);
}

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

/// Common code for various ways to save text before a change.
/// "top" is the line above the first changed line.
/// "bot" is the line below the last changed line.
/// "newbot" is the new bottom line.  Use zero when not known.
/// "reload" is true when saving for a buffer reload.
/// Careful: may trigger autocommands that reload the buffer.
/// Returns FAIL when lines could not be saved, OK otherwise.
int u_savecommon(buf_T *buf, linenr_T top, linenr_T bot, linenr_T newbot, bool reload)
{
  return rs_u_savecommon(buf, top, bot, newbot, reload);
}

// magic at start of entry
#define UF_ENTRY_MAGIC         0xf518
// magic after last entry
#define UF_ENTRY_END_MAGIC     0x3581

// extra fields for uhp
#define UHP_SAVE_NR            1

/// Compute the hash for a buffer text into hash[UNDO_HASH_SIZE].
///
/// @param[in] buf The buffer used to compute the hash
/// @param[in] hash Array of size UNDO_HASH_SIZE in which to store the value of
///                 the hash
void u_compute_hash(buf_T *buf, uint8_t *hash)
{
  rs_u_compute_hash(buf, hash);
}

/// Return an allocated string of the full path of the target undofile.
///
/// @param[in]  buf_ffname  Full file name for which undo file location should
///                         be found.
/// @param[in]  reading  If true, find the file to read by traversing all of the
///                      directories in &undodir. If false use the first
///                      existing directory. If none of the directories in
///                      &undodir option exist then last directory in the list
///                      will be automatically created.
///
/// @return [allocated] File name to read from/write to or NULL.
char *u_get_undo_file_name(const char *const buf_ffname, const bool reading)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  const char *ffname = buf_ffname;

  if (ffname == NULL) {
    return NULL;
  }

#ifdef HAVE_READLINK
  char fname_buf[MAXPATHL];
  // Expand symlink in the file name, so that we put the undo file with the
  // actual file instead of with the symlink.
  if (resolve_symlink(ffname, fname_buf) == OK) {
    ffname = fname_buf;
  }
#endif

  char dir_name[MAXPATHL + 1];
  char *munged_name = NULL;
  char *undo_file_name = NULL;

  // Loop over 'undodir'.  When reading find the first file that exists.
  // When not reading use the first directory that exists or ".".
  char *dirp = p_udir;
  while (*dirp != NUL) {
    size_t dir_len = copy_option_part(&dirp, dir_name, MAXPATHL, ",");
    if (dir_len == 1 && dir_name[0] == '.') {
      // Use same directory as the ffname,
      // "dir/name" -> "dir/.name.un~"
      const size_t ffname_len = strlen(ffname);
      undo_file_name = xmalloc(ffname_len + 6);
      memmove(undo_file_name, ffname, ffname_len + 1);
      char *const tail = path_tail(undo_file_name);
      const size_t tail_len = strlen(tail);
      memmove(tail + 1, tail, tail_len + 1);
      *tail = '.';
      memmove(tail + tail_len + 1, ".un~", sizeof(".un~"));
    } else {
      dir_name[dir_len] = NUL;

      // Remove trailing pathseps from directory name
      char *p = &dir_name[dir_len - 1];
      while (vim_ispathsep(*p)) {
        *p-- = NUL;
      }

      bool has_directory = os_isdir(dir_name);
      if (!has_directory && *dirp == NUL && !reading) {
        // Last directory in the list does not exist, create it.
        int ret;
        char *failed_dir;
        if ((ret = os_mkdir_recurse(dir_name, 0755, &failed_dir, NULL)) != 0) {
          semsg(_("E5003: Unable to create directory \"%s\" for undo file: %s"),
                failed_dir, os_strerror(ret));
          xfree(failed_dir);
        } else {
          has_directory = true;
        }
      }
      if (has_directory) {
        if (munged_name == NULL) {
          munged_name = xstrdup(ffname);
          for (char *c = munged_name; *c != NUL; MB_PTR_ADV(c)) {
            if (vim_ispathsep(*c)) {
              *c = '%';
            }
          }
        }
        undo_file_name = concat_fnames(dir_name, munged_name, true);
      }
    }

    // When reading check if the file exists.
    if (undo_file_name != NULL
        && (!reading || os_path_exists(undo_file_name))) {
      break;
    }
    XFREE_CLEAR(undo_file_name);
  }

  xfree(munged_name);
  return undo_file_name;
}

/// Display an error for corrupted undo file
///
/// @param[in]  mesg  Identifier of the corruption kind.
/// @param[in]  file_name  File in which error occurred.
static void corruption_error(const char *const mesg, const char *const file_name)
  FUNC_ATTR_NONNULL_ALL
{
  semsg(_("E825: Corrupted undo file (%s): %s"), mesg, file_name);
}

static void u_free_uhp(u_header_T *uhp)
{
  rs_u_free_uhp(uhp);
}

static u_header_T *unserialize_uhp(bufinfo_T *bi, const char *file_name)
{
  u_header_T *uhp = xmalloc(sizeof(u_header_T));
  CLEAR_POINTER(uhp);
#ifdef U_DEBUG
  uhp->uh_magic = UH_MAGIC;
#endif
  uhp->uh_next.seq = undo_read_4c(bi);
  uhp->uh_prev.seq = undo_read_4c(bi);
  uhp->uh_alt_next.seq = undo_read_4c(bi);
  uhp->uh_alt_prev.seq = undo_read_4c(bi);
  uhp->uh_seq = undo_read_4c(bi);
  if (uhp->uh_seq <= 0) {
    corruption_error("uh_seq", file_name);
    xfree(uhp);
    return NULL;
  }
  unserialize_pos(bi, &uhp->uh_cursor);
  uhp->uh_cursor_vcol = undo_read_4c(bi);
  uhp->uh_flags = undo_read_2c(bi);
  const Timestamp cur_timestamp = os_time();
  for (size_t i = 0; i < (size_t)NMARKS; i++) {
    unserialize_pos(bi, &uhp->uh_namedm[i].mark);
    uhp->uh_namedm[i].timestamp = cur_timestamp;
    uhp->uh_namedm[i].fnum = 0;
  }
  unserialize_visualinfo(bi, &uhp->uh_visual);
  uhp->uh_time = undo_read_time(bi);

  // Unserialize optional fields.
  while (true) {
    int len = undo_read_byte(bi);

    if (len == EOF) {
      corruption_error("truncated", file_name);
      u_free_uhp(uhp);
      return NULL;
    }
    if (len == 0) {
      break;
    }
    int what = undo_read_byte(bi);
    switch (what) {
    case UHP_SAVE_NR:
      uhp->uh_save_nr = undo_read_4c(bi);
      break;
    default:
      // Field not supported, skip it.
      while (--len >= 0) {
        undo_read_byte(bi);
      }
    }
  }

  // Unserialize the uep list.
  u_entry_T *last_uep = NULL;
  int c;
  while ((c = undo_read_2c(bi)) == UF_ENTRY_MAGIC) {
    bool error = false;
    u_entry_T *uep = unserialize_uep(bi, &error, file_name);
    if (last_uep == NULL) {
      uhp->uh_entry = uep;
    } else {
      last_uep->ue_next = uep;
    }
    last_uep = uep;
    if (uep == NULL || error) {
      u_free_uhp(uhp);
      return NULL;
    }
  }
  if (c != UF_ENTRY_END_MAGIC) {
    corruption_error("entry end", file_name);
    u_free_uhp(uhp);
    return NULL;
  }

  // Unserialize all extmark undo information
  kv_init(uhp->uh_extmark);

  while ((c = undo_read_2c(bi)) == UF_ENTRY_MAGIC) {
    bool error = false;
    ExtmarkUndoObject *extup = unserialize_extmark(bi, &error, file_name);
    if (error) {
      kv_destroy(uhp->uh_extmark);
      xfree(extup);
      return NULL;
    }
    kv_push(uhp->uh_extmark, *extup);
    xfree(extup);
  }
  if (c != UF_ENTRY_END_MAGIC) {
    corruption_error("entry end", file_name);
    u_free_uhp(uhp);
    return NULL;
  }

  return uhp;
}

static ExtmarkUndoObject *unserialize_extmark(bufinfo_T *bi, bool *error, const char *filename)
{
  uint8_t *buf = NULL;

  ExtmarkUndoObject *extup = xmalloc(sizeof(ExtmarkUndoObject));

  UndoObjectType type = (UndoObjectType)undo_read_4c(bi);
  extup->type = type;
  if (type == kExtmarkSplice) {
    size_t n_elems = (size_t)sizeof(ExtmarkSplice) / sizeof(uint8_t);
    buf = xcalloc(n_elems, sizeof(uint8_t));
    if (!undo_read(bi, buf, n_elems)) {
      goto error;
    }
    extup->data.splice = *(ExtmarkSplice *)buf;
  } else if (type == kExtmarkMove) {
    size_t n_elems = (size_t)sizeof(ExtmarkMove) / sizeof(uint8_t);
    buf = xcalloc(n_elems, sizeof(uint8_t));
    if (!undo_read(bi, buf, n_elems)) {
      goto error;
    }
    extup->data.move = *(ExtmarkMove *)buf;
  } else {
    goto error;
  }

  xfree(buf);

  return extup;

error:
  xfree(extup);
  if (buf) {
    xfree(buf);
  }
  *error = true;
  return NULL;
}

static u_entry_T *unserialize_uep(bufinfo_T *bi, bool *error, const char *file_name)
{
  u_entry_T *uep = xmalloc(sizeof(u_entry_T));
  CLEAR_POINTER(uep);
#ifdef U_DEBUG
  uep->ue_magic = UE_MAGIC;
#endif
  uep->ue_top = undo_read_4c(bi);
  uep->ue_bot = undo_read_4c(bi);
  uep->ue_lcount = undo_read_4c(bi);
  uep->ue_size = undo_read_4c(bi);

  char **array = NULL;
  if (uep->ue_size > 0) {
    if ((size_t)uep->ue_size < SIZE_MAX / sizeof(char *)) {
      array = xmalloc(sizeof(char *) * (size_t)uep->ue_size);
      memset(array, 0, sizeof(char *) * (size_t)uep->ue_size);
    }
  }
  uep->ue_array = array;

  for (size_t i = 0; i < (size_t)uep->ue_size; i++) {
    int line_len = undo_read_4c(bi);
    char *line;
    if (line_len >= 0) {
      line = undo_read_string(bi, (size_t)line_len);
    } else {
      line = NULL;
      corruption_error("line length", file_name);
    }
    if (line == NULL) {
      *error = true;
      return uep;
    }
    array[i] = line;
  }
  return uep;
}

/// Unserializes the pos_T at the current position.
static void unserialize_pos(bufinfo_T *bi, pos_T *pos)
{
  pos->lnum = undo_read_4c(bi);
  pos->lnum = MAX(pos->lnum, 0);
  pos->col = undo_read_4c(bi);
  pos->col = MAX(pos->col, 0);
  pos->coladd = undo_read_4c(bi);
  pos->coladd = MAX(pos->coladd, 0);
}

/// Unserializes the visualinfo_T at the current position.
static void unserialize_visualinfo(bufinfo_T *bi, visualinfo_T *info)
{
  unserialize_pos(bi, &info->vi_start);
  unserialize_pos(bi, &info->vi_end);
  info->vi_mode = undo_read_4c(bi);
  info->vi_curswant = undo_read_4c(bi);
}

/// Write the undo tree in an undo file.
///
/// @param[in]  name  Name of the undo file or NULL if this function needs to
///                   generate the undo file name based on buf->b_ffname.
/// @param[in]  forceit  True for `:wundo!`, false otherwise.
/// @param[in]  buf  Buffer for which undo file is written.
/// @param[in]  hash  Hash value of the buffer text. Must have #UNDO_HASH_SIZE
///                   size.
void u_write_undo(const char *const name, const bool forceit, buf_T *const buf, uint8_t *const hash)
  FUNC_ATTR_NONNULL_ARG(3, 4)
{
  // Call the Rust implementation
  rs_u_write_undo(name, forceit, buf, hash);
}

/// Loads the undo tree from an undo file.
/// If "name" is not NULL use it as the undo file name. This also means being
/// a bit more verbose.
/// Otherwise use curbuf->b_ffname to generate the undo file name.
/// "hash[UNDO_HASH_SIZE]" must be the hash value of the buffer text.
void u_read_undo(char *name, const uint8_t *hash, const char *orig_name FUNC_ATTR_UNUSED)
  FUNC_ATTR_NONNULL_ARG(2)
{
  // Call the Rust implementation
  rs_u_read_undo(name, hash, orig_name);
}


static int undo_read_4c(bufinfo_T *bi)
{
  return get4c(bi->bi_fp);
}

static int undo_read_2c(bufinfo_T *bi)
{
  return get2c(bi->bi_fp);
}

static int undo_read_byte(bufinfo_T *bi)
{
  return getc(bi->bi_fp);
}

static time_t undo_read_time(bufinfo_T *bi)
{
  return get8ctime(bi->bi_fp);
}

/// Reads "buffer[size]" from the undo file.
///
/// @param bi     The buffer info
/// @param buffer Character buffer to read data into
/// @param size   The size of the character buffer
///
/// @returns false in case of an error.
static bool undo_read(bufinfo_T *bi, uint8_t *buffer, size_t size)
  FUNC_ATTR_NONNULL_ARG(1)
{
  const bool retval = fread(buffer, size, 1, bi->bi_fp) == 1;
  if (!retval) {
    // Error may be checked for only later.  Fill with zeros,
    // so that the reader won't use garbage.
    memset(buffer, 0, size);
  }
  return retval;
}

/// Reads a string of length "len" from "bi->bi_fd" and appends a zero to it.
///
/// @param len can be zero to allocate an empty line.
///
/// @returns a pointer to allocated memory or NULL in case of an error.
static char *undo_read_string(bufinfo_T *bi, size_t len)
{
  char *ptr = xmallocz(len);
  if (len > 0 && !undo_read(bi, (uint8_t *)ptr, len)) {
    xfree(ptr);
    return NULL;
  }
  return ptr;
}

/// If 'cpoptions' contains 'u': Undo the previous undo or redo (vi compatible).
/// If 'cpoptions' does not contain 'u': Always undo.
void u_undo(int count)
{
  rs_u_undo(count);
}

/// If 'cpoptions' contains 'u': Repeat the previous undo or redo.
/// If 'cpoptions' does not contain 'u': Always redo.
void u_redo(int count)
{
  rs_u_redo(count);
}

/// Undo and remove the branch from the undo tree.
/// Also moves the cursor (as a "normal" undo would).
///
/// @param do_buf_event If `true`, send the changedtick with the buffer updates
bool u_undo_and_forget(int count, bool do_buf_event)
{
  return rs_u_undo_and_forget(count, do_buf_event);
}

/// Undo or redo, depending on `undo_undoes`, `count` times.
///
/// @param startcount How often to undo or redo
/// @param quiet If `true`, don't show messages
/// @param do_buf_event If `true`, send the changedtick with the buffer updates
static void u_doit(int startcount, bool quiet, bool do_buf_event)
{
  rs_u_doit(startcount, quiet, do_buf_event);
}

// Undo or redo over the timeline.
// When "step" is negative go back in time, otherwise goes forward in time.
// When "sec" is false make "step" steps, when "sec" is true use "step" as
// seconds.
// When "file" is true use "step" as a number of file writes.
// When "absolute" is true use "step" as the sequence number to jump to.
// "sec" must be false then.
void undo_time(int step, bool sec, bool file, bool absolute)
{
  // Call the Rust implementation
  rs_undo_time(step, sec, file, absolute);
}


/// u_undoredo: common code for undo and redo
///
/// The lines in the file are replaced by the lines in the entry list at
/// curbuf->b_u_curhead. The replaced lines in the file are saved in the entry
/// list for the next undo/redo.
///
/// @param undo If `true`, go up the tree. Down if `false`.
/// @param do_buf_event If `true`, send buffer updates.
static void u_undoredo(bool undo, bool do_buf_event)
{
  char **newarray = NULL;
  linenr_T newlnum = MAXLNUM;
  pos_T new_curpos = curwin->w_cursor;
  u_entry_T *nuep;
  u_entry_T *newlist = NULL;
  fmark_T namedm[NMARKS];
  u_header_T *curhead = curbuf->b_u_curhead;

  // Don't want autocommands using the undo structures here, they are
  // invalid till the end.
  block_autocmds();

#ifdef U_DEBUG
  u_check(false);
#endif
  int old_flags = curhead->uh_flags;
  int new_flags = (curbuf->b_changed ? UH_CHANGED : 0)
                  | ((curbuf->b_ml.ml_flags & ML_EMPTY) ? UH_EMPTYBUF : 0)
                  | (old_flags & UH_RELOAD);
  setpcmark();

  // save marks before undo/redo
  zero_fmark_additional_data(curbuf->b_namedm);
  memmove(namedm, curbuf->b_namedm, sizeof(curbuf->b_namedm[0]) * NMARKS);
  visualinfo_T visualinfo = curbuf->b_visual;
  curbuf->b_op_start.lnum = curbuf->b_ml.ml_line_count;
  curbuf->b_op_start.col = 0;
  curbuf->b_op_end.lnum = 0;
  curbuf->b_op_end.col = 0;

  for (u_entry_T *uep = curhead->uh_entry; uep != NULL; uep = nuep) {
    linenr_T top = uep->ue_top;
    linenr_T bot = uep->ue_bot;
    if (bot == 0) {
      bot = curbuf->b_ml.ml_line_count + 1;
    }
    if (top > curbuf->b_ml.ml_line_count || top >= bot
        || bot > curbuf->b_ml.ml_line_count + 1) {
      unblock_autocmds();
      iemsg(_("E438: u_undo: line numbers wrong"));
      changed(curbuf);                // don't want UNCHANGED now
      return;
    }

    linenr_T oldsize = bot - top - 1;        // number of lines before undo
    linenr_T newsize = uep->ue_size;         // number of lines after undo

    // Decide about the cursor position, depending on what text changed.
    // Don't set it yet, it may be invalid if lines are going to be added.
    if (top < newlnum) {
      // If the saved cursor is somewhere in this undo block, move it to
      // the remembered position.  Makes "gwap" put the cursor back
      // where it was.
      linenr_T lnum = curhead->uh_cursor.lnum;
      if (lnum >= top && lnum <= top + newsize + 1) {
        new_curpos = curhead->uh_cursor;
        newlnum = new_curpos.lnum - 1;
      } else {
        // Use the first line that actually changed.  Avoids that
        // undoing auto-formatting puts the cursor in the previous
        // line.
        int i;
        for (i = 0; i < newsize && i < oldsize; i++) {
          if (strcmp(uep->ue_array[i], ml_get(top + 1 + (linenr_T)i)) != 0) {
            break;
          }
        }
        if (i == newsize && newlnum == MAXLNUM && uep->ue_next == NULL) {
          newlnum = top;
          new_curpos.lnum = newlnum + 1;
        } else if (i < newsize) {
          newlnum = top + (linenr_T)i;
          new_curpos.lnum = newlnum + 1;
        }
      }
    }

    bool empty_buffer = false;

    // Delete the lines between top and bot and save them in newarray.
    if (oldsize > 0) {
      newarray = xmalloc(sizeof(char *) * (size_t)oldsize);
      // delete backwards, it goes faster in most cases
      int i;
      linenr_T lnum;
      for (lnum = bot - 1, i = oldsize; --i >= 0; lnum--) {
        // what can we do when we run out of memory?
        newarray[i] = u_save_line(lnum);
        // remember we deleted the last line in the buffer, and a
        // dummy empty line will be inserted
        if (curbuf->b_ml.ml_line_count == 1) {
          empty_buffer = true;
        }
        ml_delete(lnum);  // ML_DEL_UNDO
      }
    } else {
      newarray = NULL;
    }

    // make sure the cursor is on a valid line after the deletions
    check_cursor_lnum(curwin);

    // Insert the lines in u_array between top and bot.
    if (newsize) {
      int i;
      linenr_T lnum;
      for (lnum = top, i = 0; i < newsize; i++, lnum++) {
        // If the file is empty, there is an empty line 1 that we
        // should get rid of, by replacing it with the new line
        if (empty_buffer && lnum == 0) {
          ml_replace(1, uep->ue_array[i], true);
        } else {
          ml_append_flags(lnum, uep->ue_array[i], 0, 0);  // ML_APPEND_UNDO
        }
        xfree(uep->ue_array[i]);
      }
      xfree(uep->ue_array);
    }

    // Adjust marks
    if (oldsize != newsize) {
      mark_adjust(top + 1, top + oldsize, MAXLNUM, newsize - oldsize, kExtmarkNOOP);
      if (curbuf->b_op_start.lnum > top + oldsize) {
        curbuf->b_op_start.lnum += newsize - oldsize;
      }
      if (curbuf->b_op_end.lnum > top + oldsize) {
        curbuf->b_op_end.lnum += newsize - oldsize;
      }
    }

    if (oldsize > 0 || newsize > 0) {
      changed_lines(curbuf, top + 1, 0, bot, newsize - oldsize, do_buf_event);
      // When text has been changed, possibly the start of the next line
      // may have SpellCap that should be removed or it needs to be
      // displayed.  Schedule the next line for redrawing just in case.
      if (spell_check_window(curwin) && bot <= curbuf->b_ml.ml_line_count) {
        redrawWinline(curwin, bot);
      }
    }

    // Set the '[ mark.
    curbuf->b_op_start.lnum = MIN(curbuf->b_op_start.lnum, top + 1);
    // Set the '] mark.
    if (newsize == 0 && top + 1 > curbuf->b_op_end.lnum) {
      curbuf->b_op_end.lnum = top + 1;
    } else if (top + newsize > curbuf->b_op_end.lnum) {
      curbuf->b_op_end.lnum = top + newsize;
    }

    u_newcount += newsize;
    u_oldcount += oldsize;
    uep->ue_size = oldsize;
    uep->ue_array = newarray;
    uep->ue_bot = top + newsize + 1;

    // insert this entry in front of the new entry list
    nuep = uep->ue_next;
    uep->ue_next = newlist;
    newlist = uep;
  }

  // Ensure the '[ and '] marks are within bounds.
  curbuf->b_op_start.lnum = MIN(curbuf->b_op_start.lnum, curbuf->b_ml.ml_line_count);
  curbuf->b_op_end.lnum = MIN(curbuf->b_op_end.lnum, curbuf->b_ml.ml_line_count);

  // Adjust Extmarks
  if (undo) {
    for (int i = (int)kv_size(curhead->uh_extmark) - 1; i > -1; i--) {
      extmark_apply_undo(kv_A(curhead->uh_extmark, i), undo);
    }
    // redo
  } else {
    for (int i = 0; i < (int)kv_size(curhead->uh_extmark); i++) {
      extmark_apply_undo(kv_A(curhead->uh_extmark, i), undo);
    }
  }
  if (curhead->uh_flags & UH_RELOAD) {
    // TODO(bfredl): this is a bit crude. When 'undoreload' is used we
    // should have all info to send a buffer-reloaing on_lines/on_bytes event
    buf_updates_unload(curbuf, true);
  }
  // Finish adjusting extmarks

  // Set the cursor to the desired position.  Check that the line is valid.
  curwin->w_cursor = new_curpos;
  check_cursor_lnum(curwin);

  curhead->uh_entry = newlist;
  curhead->uh_flags = new_flags;
  if ((old_flags & UH_EMPTYBUF) && buf_is_empty(curbuf)) {
    curbuf->b_ml.ml_flags |= ML_EMPTY;
  }
  if (old_flags & UH_CHANGED) {
    changed(curbuf);
  } else {
    unchanged(curbuf, false, true);
  }

  // because the calls to changed()/unchanged() above will bump changedtick
  // again, we need to send a nvim_buf_lines_event with just the new value of
  // b:changedtick
  if (do_buf_event) {
    buf_updates_changedtick(curbuf);
  }

  // restore marks from before undo/redo
  for (int i = 0; i < NMARKS; i++) {
    if (curhead->uh_namedm[i].mark.lnum != 0) {
      free_fmark(curbuf->b_namedm[i]);
      curbuf->b_namedm[i] = curhead->uh_namedm[i];
    }
    if (namedm[i].mark.lnum != 0) {
      curhead->uh_namedm[i] = namedm[i];
    } else {
      curhead->uh_namedm[i].mark.lnum = 0;
    }
  }
  if (curhead->uh_visual.vi_start.lnum != 0) {
    curbuf->b_visual = curhead->uh_visual;
    curhead->uh_visual = visualinfo;
  }

  // If the cursor is only off by one line, put it at the same position as
  // before starting the change (for the "o" command).
  // Otherwise the cursor should go to the first undone line.
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
    // We get here with the current cursor line being past the end (eg
    // after adding lines at the end of the file, and then undoing it).
    // check_cursor() will move the cursor to the last line.  Move it to
    // the first column here.
    curwin->w_cursor.col = 0;
    curwin->w_cursor.coladd = 0;
  }

  // Make sure the cursor is on an existing line and column.
  check_cursor(curwin);

  // Remember where we are for "g-" and ":earlier 10s".
  curbuf->b_u_seq_cur = curhead->uh_seq;
  if (undo) {
    // We are below the previous undo.  However, to make ":earlier 1s"
    // work we compute this as being just above the just undone change.
    curbuf->b_u_seq_cur = curhead->uh_next.ptr
                          ? curhead->uh_next.ptr->uh_seq : 0;
  }

  // Remember where we are for ":earlier 1f" and ":later 1f".
  if (curhead->uh_save_nr != 0) {
    if (undo) {
      curbuf->b_u_save_nr_cur = curhead->uh_save_nr - 1;
    } else {
      curbuf->b_u_save_nr_cur = curhead->uh_save_nr;
    }
  }

  // The timestamp can be the same for multiple changes, just use the one of
  // the undone/redone change.
  curbuf->b_u_time_cur = curhead->uh_time;

  unblock_autocmds();
#ifdef U_DEBUG
  u_check(false);
#endif
}

/// If we deleted or added lines, report the number of less/more lines.
/// Otherwise, report the number of changes (this may be incorrect
/// in some cases, but it's better than nothing).
///
/// @param did_undo  just did an undo
/// @param absolute  used ":undo N"
static void u_undo_end(bool did_undo, bool absolute, bool quiet)
{
  if ((fdo_flags & kOptFdoFlagUndo) && KeyTyped) {
    foldOpenCursor();
  }

  if (quiet
      || global_busy        // no messages until global is finished
      || !messaging()) {    // 'lazyredraw' set, don't do messages now
    return;
  }

  if (curbuf->b_ml.ml_flags & ML_EMPTY) {
    u_newcount--;
  }

  u_oldcount -= u_newcount;
  char *msgstr;
  if (u_oldcount == -1) {
    msgstr = N_("more line");
  } else if (u_oldcount < 0) {
    msgstr = N_("more lines");
  } else if (u_oldcount == 1) {
    msgstr = N_("line less");
  } else if (u_oldcount > 1) {
    msgstr = N_("fewer lines");
  } else {
    u_oldcount = u_newcount;
    if (u_newcount == 1) {
      msgstr = N_("change");
    } else {
      msgstr = N_("changes");
    }
  }

  u_header_T *uhp;
  if (curbuf->b_u_curhead != NULL) {
    // For ":undo N" we prefer a "after #N" message.
    if (absolute && curbuf->b_u_curhead->uh_next.ptr != NULL) {
      uhp = curbuf->b_u_curhead->uh_next.ptr;
      did_undo = false;
    } else if (did_undo) {
      uhp = curbuf->b_u_curhead;
    } else {
      uhp = curbuf->b_u_curhead->uh_next.ptr;
    }
  } else {
    uhp = curbuf->b_u_newhead;
  }

  char msgbuf[80];
  if (uhp == NULL) {
    *msgbuf = NUL;
  } else {
    undo_fmt_time(msgbuf, sizeof(msgbuf), uhp->uh_time);
  }

  {
    FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
      if (wp->w_buffer == curbuf && wp->w_p_cole > 0) {
        redraw_later(wp, UPD_NOT_VALID);
      }
    }
  }

  if (VIsual_active) {
    check_pos(curbuf, &VIsual);
  }

  smsg_keep(0, _("%" PRId64 " %s; %s #%" PRId64 "  %s"),
            u_oldcount < 0 ? (int64_t)-u_oldcount : (int64_t)u_oldcount,
            _(msgstr),
            did_undo ? _("before") : _("after"),
            uhp == NULL ? 0 : (int64_t)uhp->uh_seq,
            msgbuf);
}

/// Put the timestamp of an undo header in "buf[buflen]" in a nice format.
void undo_fmt_time(char *buf, size_t buflen, time_t tt)
{
  if (time(NULL) - tt >= 100) {
    struct tm curtime;
    os_localtime_r(&tt, &curtime);
    size_t n;
    if (time(NULL) - tt < (60 * 60 * 12)) {
      // within 12 hours
      n = strftime(buf, buflen, "%H:%M:%S", &curtime);
    } else {
      // longer ago
      n = strftime(buf, buflen, "%Y/%m/%d %H:%M:%S", &curtime);
    }
    if (n == 0) {
      buf[0] = NUL;
    }
  } else {
    int64_t seconds = time(NULL) - tt;
    vim_snprintf(buf, buflen,
                 NGETTEXT("%" PRId64 " second ago",
                          "%" PRId64 " seconds ago", (uint32_t)seconds),
                 seconds);
  }
}

/// u_sync: stop adding to the current entry list
///
/// @param force  if true, also sync when no_u_sync is set.
void u_sync(bool force)
{
  rs_u_sync(force);
}

/// ":undolist": List the leafs of the undo tree
void ex_undolist(exarg_T *eap)
{
  // Call the Rust implementation
  rs_ex_undolist(eap);
}

/// ":undojoin": continue adding to the last entry list
void ex_undojoin(exarg_T *eap)
{
  (void)eap;  // unused
  rs_ex_undojoin();
}

/// Called after writing or reloading the file and setting b_changed to false.
/// Now an undo means that the buffer is modified.
void u_unchanged(buf_T *buf)
{
  rs_u_unchanged(buf);
}

/// After reloading a buffer which was saved for 'undoreload': Find the first
/// line that was changed and set the cursor there.
void u_find_first_changed(void)
{
  rs_u_find_first_changed();
}

/// Increase the write count, store it in the last undo header, what would be
/// used for "u".
void u_update_save_nr(buf_T *buf)
{
  rs_u_update_save_nr(buf);
}

static void u_unch_branch(u_header_T *uhp)
{
  rs_u_unch_branch(uhp);
}

/// Get pointer to last added entry.
/// If it's not valid, give an error message and return NULL.
static u_entry_T *u_get_headentry(buf_T *buf)
{
  return rs_u_get_headentry(buf);
}

/// u_getbot(): compute the line number of the previous u_save
///              It is called only when b_u_synced is false.
static void u_getbot(buf_T *buf)
{
  rs_u_getbot(buf);
}

/// Free one header "uhp" and its entry list and adjust the pointers.
///
/// @param uhpp  if not NULL reset when freeing this header
static void u_freeheader(buf_T *buf, u_header_T *uhp, u_header_T **uhpp)
{
  rs_u_freeheader(buf, uhp, uhpp);
}

/// Free an alternate branch and any following alternate branches.
///
/// @param uhpp  if not NULL reset when freeing this header
static void u_freebranch(buf_T *buf, u_header_T *uhp, u_header_T **uhpp)
{
  rs_u_freebranch(buf, uhp, uhpp);
}

/// Free all the undo entries for one header and the header itself.
/// This means that "uhp" is invalid when returning.
///
/// @param uhpp  if not NULL reset when freeing this header
static void u_freeentries(buf_T *buf, u_header_T *uhp, u_header_T **uhpp)
{
  rs_u_freeentries(buf, uhp, uhpp);
}

/// free entry 'uep' and 'n' lines in uep->ue_array[]
static void u_freeentry(u_entry_T *uep, int n)
{
  rs_u_freeentry(uep, n);
}

/// invalidate the undo buffer; called when storage has already been released
void u_clearall(buf_T *buf)
{
  rs_u_clearall(buf);
}

/// Free all allocated memory blocks for the buffer 'buf'.
void u_blockfree(buf_T *buf)
{
  rs_u_blockfree(buf);
}

/// Free all allocated memory blocks for the buffer 'buf'.
/// and invalidate the undo buffer
void u_clearallandblockfree(buf_T *buf)
{
  rs_u_clearallandblockfree(buf);
}

/// Save the line "lnum" for the "U" command.
static void u_saveline(buf_T *buf, linenr_T lnum)
{
  if (lnum == buf->b_u_line_lnum) {      // line is already saved
    return;
  }
  if (lnum < 1 || lnum > buf->b_ml.ml_line_count) {  // should never happen
    return;
  }
  u_clearline(buf);
  buf->b_u_line_lnum = lnum;
  if (curwin->w_buffer == buf && curwin->w_cursor.lnum == lnum) {
    buf->b_u_line_colnr = curwin->w_cursor.col;
  } else {
    buf->b_u_line_colnr = 0;
  }
  buf->b_u_line_ptr = u_save_line_buf(buf, lnum);
}

/// clear the line saved for the "U" command
/// (this is used externally for crossing a line while in insert mode)
void u_clearline(buf_T *buf)
{
  rs_u_clearline(buf);
}

/// Implementation of the "U" command.
/// Differentiation from vi: "U" can be undone with the next "U".
/// We also allow the cursor to be in another line.
/// Careful: may trigger autocommands that reload the buffer.
void u_undoline(void)
{
  rs_u_undoline();
}

/// Allocate memory and copy curbuf line into it.
///
/// @param lnum the line to copy
static char *u_save_line(linenr_T lnum)
{
  return u_save_line_buf(curbuf, lnum);
}

/// Allocate memory and copy line into it
///
/// @param lnum line to copy
/// @param buf buffer to copy from
static char *u_save_line_buf(buf_T *buf, linenr_T lnum)
{
  return xstrdup(ml_get_buf(buf, lnum));
}

/// Check if the 'modified' flag is set, or 'ff' has changed (only need to
/// check the first character, because it can only be "dos", "unix" or "mac").
/// "nofile" and "scratch" type buffers are considered to always be unchanged.
///
/// @param buf The buffer to check
///
/// @return true if the buffer has changed
bool bufIsChanged(buf_T *buf)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_bufIsChanged(buf);
}

// Return true if any buffer has changes.  Also buffers that are not written.
bool anyBufIsChanged(void)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_anyBufIsChanged();
}

/// @see bufIsChanged
/// @return true if the current buffer has changed
bool curbufIsChanged(void)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_curbufIsChanged();
}

/// Append the list of undo blocks to a newly allocated list
///
/// For use in undotree(). Recursive.
///
/// @param[in]  first_uhp  Undo blocks list to start with.
///
/// @return [allocated] List with a representation of undo blocks.
static list_T *u_eval_tree(buf_T *const buf, const u_header_T *const first_uhp)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_RET
{
  return rs_u_eval_tree(buf, first_uhp);
}

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

  tv_dict_add_list(dict, S_LEN("entries"), u_eval_tree(buf, buf->b_u_oldhead));
}

// Given the buffer, Return the undo header. If none is set, set one first.
// NULL will be returned if e.g undolevels = -1 (undo disabled)
u_header_T *u_force_get_undo_header(buf_T *buf)
{
  return rs_u_force_get_undo_header(buf);
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

bool nvim_bt_prompt(buf_T *buf)
{
  return bt_prompt(buf);
}

bool nvim_file_ff_differs(buf_T *buf, bool strict)
{
  return file_ff_differs(buf, strict);
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

// u_header_T field accessors
u_header_T *nvim_uhp_get_next(u_header_T *uhp)
{
  return uhp->uh_next.ptr;
}

u_header_T *nvim_uhp_get_prev(u_header_T *uhp)
{
  return uhp->uh_prev.ptr;
}

u_header_T *nvim_uhp_get_alt_next(u_header_T *uhp)
{
  return uhp->uh_alt_next.ptr;
}

u_header_T *nvim_uhp_get_alt_prev(u_header_T *uhp)
{
  return uhp->uh_alt_prev.ptr;
}

int nvim_uhp_get_seq(u_header_T *uhp)
{
  return uhp->uh_seq;
}

int nvim_uhp_get_walk(u_header_T *uhp)
{
  return uhp->uh_walk;
}

u_entry_T *nvim_uhp_get_entry(u_header_T *uhp)
{
  return uhp->uh_entry;
}

u_entry_T *nvim_uhp_get_getbot_entry(u_header_T *uhp)
{
  return uhp->uh_getbot_entry;
}

time_t nvim_uhp_get_time(u_header_T *uhp)
{
  return uhp->uh_time;
}

int nvim_uhp_get_flags(u_header_T *uhp)
{
  return uhp->uh_flags;
}

int nvim_uhp_get_save_nr(u_header_T *uhp)
{
  return uhp->uh_save_nr;
}

void nvim_uhp_set_next(u_header_T *uhp, u_header_T *val)
{
  uhp->uh_next.ptr = val;
}

void nvim_uhp_set_prev(u_header_T *uhp, u_header_T *val)
{
  uhp->uh_prev.ptr = val;
}

void nvim_uhp_set_alt_next(u_header_T *uhp, u_header_T *val)
{
  uhp->uh_alt_next.ptr = val;
}

void nvim_uhp_set_alt_prev(u_header_T *uhp, u_header_T *val)
{
  uhp->uh_alt_prev.ptr = val;
}

void nvim_uhp_set_seq(u_header_T *uhp, int val)
{
  uhp->uh_seq = val;
}

void nvim_uhp_set_walk(u_header_T *uhp, int val)
{
  uhp->uh_walk = val;
}

void nvim_uhp_set_entry(u_header_T *uhp, u_entry_T *val)
{
  uhp->uh_entry = val;
}

void nvim_uhp_set_getbot_entry(u_header_T *uhp, u_entry_T *val)
{
  uhp->uh_getbot_entry = val;
}

void nvim_uhp_set_time(u_header_T *uhp, time_t val)
{
  uhp->uh_time = val;
}

void nvim_uhp_set_flags(u_header_T *uhp, int val)
{
  uhp->uh_flags = val;
}

void nvim_uhp_set_save_nr(u_header_T *uhp, int val)
{
  uhp->uh_save_nr = val;
}

// u_entry_T field accessors
u_entry_T *nvim_uep_get_next(u_entry_T *uep)
{
  return uep->ue_next;
}

linenr_T nvim_uep_get_top(u_entry_T *uep)
{
  return uep->ue_top;
}

linenr_T nvim_uep_get_bot(u_entry_T *uep)
{
  return uep->ue_bot;
}

linenr_T nvim_uep_get_lcount(u_entry_T *uep)
{
  return uep->ue_lcount;
}

linenr_T nvim_uep_get_size(u_entry_T *uep)
{
  return uep->ue_size;
}

char **nvim_uep_get_array(u_entry_T *uep)
{
  return uep->ue_array;
}

void nvim_uep_set_next(u_entry_T *uep, u_entry_T *val)
{
  uep->ue_next = val;
}

void nvim_uep_set_top(u_entry_T *uep, linenr_T val)
{
  uep->ue_top = val;
}

void nvim_uep_set_bot(u_entry_T *uep, linenr_T val)
{
  uep->ue_bot = val;
}

void nvim_uep_set_lcount(u_entry_T *uep, linenr_T val)
{
  uep->ue_lcount = val;
}

void nvim_uep_set_size(u_entry_T *uep, linenr_T val)
{
  uep->ue_size = val;
}

void nvim_uep_set_array(u_entry_T *uep, char **val)
{
  uep->ue_array = val;
}

char *nvim_uep_get_array_element(u_entry_T *uep, linenr_T idx)
{
  return uep->ue_array[idx];
}

void nvim_uep_set_array_element(u_entry_T *uep, linenr_T idx, char *val)
{
  uep->ue_array[idx] = val;
}

// Memory allocation - return pointer for Rust to use with xfree
u_entry_T *nvim_alloc_u_entry(void)
{
  return xcalloc(1, sizeof(u_entry_T));
}

u_header_T *nvim_alloc_u_header(void)
{
  return xcalloc(1, sizeof(u_header_T));
}

// Destroy extmark vector in u_header_T (replaces kv_destroy macro)
void nvim_uhp_destroy_extmark(u_header_T *uhp)
{
  kv_destroy(uhp->uh_extmark);
}

// Error message wrappers
void nvim_iemsg_undo_list_corrupt(void)
{
  iemsg(_(e_undo_list_corrupt));
}

void nvim_iemsg_undo_line_missing(void)
{
  iemsg(_(e_undo_line_missing));
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

void nvim_u_doit(int count, bool quiet, bool do_buf_event)
{
  u_doit(count, quiet, do_buf_event);
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

void nvim_u_undoredo(bool undo, bool do_buf_event)
{
  u_undoredo(undo, do_buf_event);
}

void nvim_u_undo_end(bool did_undo, bool absolute, bool quiet)
{
  u_undo_end(did_undo, absolute, quiet);
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

// Initialize extmark vector in undo header
void nvim_uhp_init_extmark(u_header_T *uhp)
{
  kv_init(uhp->uh_extmark);
}

// Copy marks and visual from buffer to undo header
void nvim_uhp_copy_marks_visual(buf_T *buf, u_header_T *uhp)
{
  zero_fmark_additional_data(buf->b_namedm);
  memmove(uhp->uh_namedm, buf->b_namedm, sizeof(buf->b_namedm[0]) * NMARKS);
  uhp->uh_visual = buf->b_visual;
}

// Set undo header cursor position
void nvim_uhp_set_cursor(u_header_T *uhp, linenr_T lnum, colnr_T col, colnr_T coladd)
{
  uhp->uh_cursor.lnum = lnum;
  uhp->uh_cursor.col = col;
  uhp->uh_cursor.coladd = coladd;
}

void nvim_uhp_set_cursor_vcol(u_header_T *uhp, colnr_T vcol)
{
  uhp->uh_cursor_vcol = vcol;
}

// Allocate and copy line array element
void nvim_uep_alloc_array(u_entry_T *uep, linenr_T size)
{
  uep->ue_array = xmalloc(sizeof(char *) * (size_t)size);
}

void nvim_uep_set_array_from_buf(u_entry_T *uep, linenr_T idx, buf_T *buf, linenr_T lnum)
{
  uep->ue_array[idx] = xstrdup(ml_get_buf(buf, lnum));
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

// u_saveline wrapper
void nvim_u_saveline(buf_T *buf, linenr_T lnum)
{
  u_saveline(buf, lnum);
}

// Get/set undo_undoes global
void nvim_set_undo_undoes_false(void)
{
  undo_undoes = false;
}

// Compare buffer line with ue_array element, returns true if different
bool nvim_uep_compare_line_with_array(u_entry_T *uep, linenr_T idx, buf_T *buf, linenr_T lnum)
{
  return strcmp(ml_get_buf(buf, lnum), uep->ue_array[idx]) != 0;
}

// Clear uh_cursor position
void nvim_uhp_clear_cursor(u_header_T *uhp)
{
  clearpos(&(uhp->uh_cursor));
}

// Set uh_cursor.lnum only
void nvim_uhp_set_cursor_lnum_only(u_header_T *uhp, linenr_T lnum)
{
  uhp->uh_cursor.lnum = lnum;
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

int nvim_get_lastmark(void)
{
  return lastmark;
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

void nvim_set_lastmark(int val)
{
  lastmark = val;
}

// ============================================================================
// Undo File I/O FFI Functions
// ============================================================================

// File operations
FILE *nvim_undo_fopen(const char *path, const char *mode)
{
  return os_fopen(path, mode);
}

int nvim_undo_fclose(FILE *fp)
{
  return fclose(fp);
}

size_t nvim_undo_fwrite(const void *ptr, size_t size, size_t count, FILE *fp)
{
  return fwrite(ptr, size, count, fp);
}

size_t nvim_undo_fread(void *ptr, size_t size, size_t count, FILE *fp)
{
  return fread(ptr, size, count, fp);
}

int nvim_undo_fflush(FILE *fp)
{
  return fflush(fp);
}

int nvim_undo_fgetc(FILE *fp)
{
  return getc(fp);
}

// File I/O helpers (reading from C file handle)
int nvim_undo_get2c(FILE *fp)
{
  return get2c(fp);
}

int nvim_undo_get4c(FILE *fp)
{
  return get4c(fp);
}

time_t nvim_undo_get8ctime(FILE *fp)
{
  return get8ctime(fp);
}

// Undo file path helper
char *nvim_u_get_undo_file_name(const char *ffname, bool reading)
{
  return u_get_undo_file_name(ffname, reading);
}

// File system operations
bool nvim_os_path_exists(const char *path)
{
  return os_path_exists(path);
}

int nvim_os_remove(const char *path)
{
  return os_remove(path);
}

int nvim_os_open(const char *path, int flags, int mode)
{
  return os_open(path, flags, mode);
}

int nvim_os_close(int fd)
{
  return os_close(fd);
}

int nvim_os_getperm(const char *path)
{
  return os_getperm(path);
}

int nvim_os_setperm(const char *path, int perm)
{
  return os_setperm(path, perm);
}

int nvim_os_fsync(int fd)
{
  return os_fsync(fd);
}

FILE *nvim_fdopen(int fd, const char *mode)
{
  return fdopen(fd, mode);
}

int nvim_fileno(FILE *fp)
{
  return fileno(fp);
}

// Message functions for undo file I/O
void nvim_undo_verbose_enter(void)
{
  verbose_enter();
}

void nvim_undo_verbose_leave(void)
{
  verbose_leave();
}

void nvim_undo_smsg(const char *msg, const char *arg)
{
  smsg(0, msg, arg);
}

void nvim_undo_semsg(const char *msg, const char *arg)
{
  semsg(msg, arg);
}

void nvim_undo_give_warning(const char *msg, bool serious)
{
  give_warning(msg, serious);
}

void nvim_undo_verb_msg(const char *msg)
{
  verb_msg(msg);
}

// Option accessors
int nvim_get_p_verbose(void)
{
  return p_verbose;
}

bool nvim_get_p_fs(void)
{
  return p_fs;
}

// u_sync wrapper
void nvim_u_sync(bool force)
{
  u_sync(force);
}

// Buffer line count and line accessors for hash computation
linenr_T nvim_buf_get_b_ml_line_count(buf_T *buf)
{
  return buf->b_ml.ml_line_count;
}

const char *nvim_ml_get_buf_line(buf_T *buf, linenr_T lnum)
{
  return ml_get_buf(buf, lnum);
}

// ACL operations (Unix)
vim_acl_T nvim_os_get_acl(const char *path)
{
  return os_get_acl(path);
}

void nvim_os_set_acl(const char *path, vim_acl_T acl)
{
  os_set_acl(path, acl);
}

void nvim_os_free_acl(vim_acl_T acl)
{
  os_free_acl(acl);
}

// Hash computation wrapper
void nvim_u_compute_hash(buf_T *buf, uint8_t *hash)
{
  u_compute_hash(buf, hash);
}

// File info for Unix ownership checks
#ifdef UNIX
bool nvim_undo_check_file_owner(const char *orig_path, const char *undo_path)
{
  FileInfo file_info_orig;
  FileInfo file_info_undo;
  if (os_fileinfo(orig_path, &file_info_orig)
      && os_fileinfo(undo_path, &file_info_undo)
      && file_info_orig.stat.st_uid != file_info_undo.stat.st_uid
      && file_info_undo.stat.st_uid != getuid()) {
    return false;  // Owner mismatch, not safe
  }
  return true;  // Safe to read
}

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
bool nvim_undo_check_file_owner(const char *orig_path, const char *undo_path)
{
  (void)orig_path;
  (void)undo_path;
  return true;  // Always safe on non-Unix
}

int nvim_undo_set_file_group(int fd, const char *orig_path, const char *undo_path, int perm)
{
  (void)fd;
  (void)orig_path;
  (void)undo_path;
  return perm;
}
#endif

// Read helper for errno handling
ssize_t nvim_read_eintr(int fd, void *buf, size_t count)
{
  return read_eintr(fd, buf, count);
}

// Extmark serialization
size_t nvim_uhp_get_extmark_count(u_header_T *uhp)
{
  return kv_size(uhp->uh_extmark);
}

int nvim_uhp_get_extmark_type(u_header_T *uhp, size_t idx)
{
  if (idx >= kv_size(uhp->uh_extmark)) {
    return -1;
  }
  return (int)kv_A(uhp->uh_extmark, idx).type;
}

void nvim_uhp_get_extmark_data(u_header_T *uhp, size_t idx, uint8_t *buf, size_t size)
{
  if (idx >= kv_size(uhp->uh_extmark)) {
    memset(buf, 0, size);
    return;
  }
  ExtmarkUndoObject *extup = &kv_A(uhp->uh_extmark, idx);
  if (extup->type == kExtmarkSplice) {
    size_t copy_size = MIN(size, sizeof(ExtmarkSplice));
    memcpy(buf, &extup->data.splice, copy_size);
  } else if (extup->type == kExtmarkMove) {
    size_t copy_size = MIN(size, sizeof(ExtmarkMove));
    memcpy(buf, &extup->data.move, copy_size);
  } else {
    memset(buf, 0, size);
  }
}

// Named mark and visual info serialization
linenr_T nvim_uhp_get_namedm_lnum(u_header_T *uhp, int idx)
{
  if (idx < 0 || idx >= NMARKS) {
    return 0;
  }
  return uhp->uh_namedm[idx].mark.lnum;
}

colnr_T nvim_uhp_get_namedm_col(u_header_T *uhp, int idx)
{
  if (idx < 0 || idx >= NMARKS) {
    return 0;
  }
  return uhp->uh_namedm[idx].mark.col;
}

colnr_T nvim_uhp_get_namedm_coladd(u_header_T *uhp, int idx)
{
  if (idx < 0 || idx >= NMARKS) {
    return 0;
  }
  return uhp->uh_namedm[idx].mark.coladd;
}

linenr_T nvim_uhp_get_visual_start_lnum(u_header_T *uhp)
{
  return uhp->uh_visual.vi_start.lnum;
}

colnr_T nvim_uhp_get_visual_start_col(u_header_T *uhp)
{
  return uhp->uh_visual.vi_start.col;
}

colnr_T nvim_uhp_get_visual_start_coladd(u_header_T *uhp)
{
  return uhp->uh_visual.vi_start.coladd;
}

linenr_T nvim_uhp_get_visual_end_lnum(u_header_T *uhp)
{
  return uhp->uh_visual.vi_end.lnum;
}

colnr_T nvim_uhp_get_visual_end_col(u_header_T *uhp)
{
  return uhp->uh_visual.vi_end.col;
}

colnr_T nvim_uhp_get_visual_end_coladd(u_header_T *uhp)
{
  return uhp->uh_visual.vi_end.coladd;
}

int nvim_uhp_get_visual_mode(u_header_T *uhp)
{
  return uhp->uh_visual.vi_mode;
}

colnr_T nvim_uhp_get_visual_curswant(u_header_T *uhp)
{
  return uhp->uh_visual.vi_curswant;
}

// Cursor position from header
linenr_T nvim_uhp_get_cursor_lnum(u_header_T *uhp)
{
  return uhp->uh_cursor.lnum;
}

colnr_T nvim_uhp_get_cursor_col(u_header_T *uhp)
{
  return uhp->uh_cursor.col;
}

colnr_T nvim_uhp_get_cursor_coladd(u_header_T *uhp)
{
  return uhp->uh_cursor.coladd;
}

colnr_T nvim_uhp_get_cursor_vcol(u_header_T *uhp)
{
  return uhp->uh_cursor_vcol;
}

// Sequence number accessors for serialization
int nvim_uhp_get_next_seq(u_header_T *uhp)
{
  return uhp->uh_next.ptr ? uhp->uh_next.ptr->uh_seq : 0;
}

int nvim_uhp_get_prev_seq(u_header_T *uhp)
{
  return uhp->uh_prev.ptr ? uhp->uh_prev.ptr->uh_seq : 0;
}

int nvim_uhp_get_alt_next_seq(u_header_T *uhp)
{
  return uhp->uh_alt_next.ptr ? uhp->uh_alt_next.ptr->uh_seq : 0;
}

int nvim_uhp_get_alt_prev_seq(u_header_T *uhp)
{
  return uhp->uh_alt_prev.ptr ? uhp->uh_alt_prev.ptr->uh_seq : 0;
}

// Allocate memory with zero terminator
void *nvim_undo_xmallocz(size_t size)
{
  return xmallocz(size);
}

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

FILE *nvim_os_fopen(const char *path, const char *mode)
{
  return os_fopen(path, mode);
}

void nvim_u_blockfree(buf_T *buf)
{
  u_blockfree(buf);
}

void nvim_u_free_uhp(u_header_T *uhp)
{
  u_free_uhp(uhp);
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

u_header_T *nvim_unserialize_uhp(FILE *fp, const char *file_name)
{
  bufinfo_T bi = {
    .bi_buf = curbuf,
    .bi_fp = fp,
  };
  return unserialize_uhp(&bi, file_name);
}

int nvim_uhp_get_next_seq_for_swizzle(u_header_T *uhp)
{
  return uhp->uh_next.seq;
}

int nvim_uhp_get_prev_seq_for_swizzle(u_header_T *uhp)
{
  return uhp->uh_prev.seq;
}

int nvim_uhp_get_alt_next_seq_for_swizzle(u_header_T *uhp)
{
  return uhp->uh_alt_next.seq;
}

int nvim_uhp_get_alt_prev_seq_for_swizzle(u_header_T *uhp)
{
  return uhp->uh_alt_prev.seq;
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

void nvim_undo_msg_putchar(int c)
{
  msg_putchar(c);
}

void nvim_undo_msg_puts(const char *s)
{
  msg_puts(s);
}

char *nvim_undo_xstrdup(const char *s)
{
  return xstrdup(s);
}

void nvim_undolist_format_entry(u_header_T *uhp, int changes, char *buf, size_t buf_size)
{
  vim_snprintf(buf, buf_size, "%6d %7d  ", uhp->uh_seq, changes);
  undo_fmt_time(buf + strlen(buf), buf_size - strlen(buf), uhp->uh_time);
  if (uhp->uh_save_nr > 0) {
    while (strlen(buf) < 33) {
      xstrlcat(buf, " ", buf_size);
    }
    vim_snprintf_add(buf, buf_size, "  %3d", uhp->uh_save_nr);
  }
}

// ============================================================================
// Phase 5: VimL function FFI wrappers
// ============================================================================

list_T *nvim_tv_list_alloc(void)
{
  return tv_list_alloc(kListLenMayKnow);
}

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

/// Append line after 'lnum' in current buffer. Returns OK/FAIL.
int nvim_ml_append_lnum(linenr_T lnum, const char *line, colnr_T len, bool newfile)
{
  return ml_append(lnum, (char *)line, len, newfile);
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

/// Set pc mark for jump list
void nvim_undo_setpcmark(void)
{
  setpcmark();
}

/// Check cursor line number validity and adjust if needed
void nvim_undo_check_cursor_lnum(win_T *win)
{
  check_cursor_lnum(win);
}

/// Mark adjust for undo
void nvim_undo_mark_adjust(linenr_T top, linenr_T bot, linenr_T amount, linenr_T amount_after)
{
  mark_adjust(top, bot, amount, amount_after, kExtmarkNOOP);
}

/// Changed lines notification
void nvim_undo_changed_lines(buf_T *buf, linenr_T top, colnr_T col, linenr_T bot, linenr_T xtra,
                             bool do_buf_event)
{
  changed_lines(buf, top, col, bot, xtra, do_buf_event);
}

/// Mark buffer as changed
void nvim_buf_changed(buf_T *buf)
{
  changed(buf);
}

/// Mark buffer as unchanged
void nvim_buf_unchanged(buf_T *buf, bool ff, bool always_strstruc)
{
  unchanged(buf, ff, always_strstruc);
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

/// Buffer updates unload
void nvim_buf_updates_unload(buf_T *buf, bool force)
{
  buf_updates_unload(buf, force);
}

/// Check position validity
void nvim_check_pos(buf_T *buf, pos_T *pos)
{
  check_pos(buf, pos);
}

/// Buffer is empty check
bool nvim_buf_is_empty(buf_T *buf)
{
  return buf_is_empty(buf);
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
  return u_save_line(lnum);
}

/// Get global_busy flag
bool nvim_get_global_busy(void)
{
  return global_busy;
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

/// Fold open cursor
void nvim_undo_foldOpenCursor(void)
{
  foldOpenCursor();
}

/// Check VIsual_active
bool nvim_undo_get_visual_active(void)
{
  return VIsual_active;
}

/// Get VIsual position
void nvim_undo_get_visual_pos(linenr_T *lnum, colnr_T *col, colnr_T *coladd)
{
  *lnum = VIsual.lnum;
  *col = VIsual.col;
  *coladd = VIsual.coladd;
}
