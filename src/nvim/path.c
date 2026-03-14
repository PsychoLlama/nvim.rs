#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/eval.h"
#include "nvim/ex_docmd.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/shell.h"
#include "nvim/path.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/strings.h"
#include "nvim/vim_defs.h"

enum {
  URL_SLASH = 1,      // path_is_url() has found ":/"
  URL_BACKSLASH = 2,  // path_is_url() has found ":\\"
};

#ifdef gen_expand_wildcards
# undef gen_expand_wildcards
#endif

// pstrcmp is implemented in Rust (path crate)
extern int pstrcmp(const void *a, const void *b);

// Forward declarations (defined below, after #include "path.c.generated.h")
static int expand_backtick(garray_T *gap, char *pat, int flags);
static int expand_in_path(garray_T *gap, char *pattern, int flags);

// C accessor functions for Rust
int nvim_path_os_isdir(const char *name) {
  return os_isdir(name);
}

void *nvim_path_xmalloc(size_t size) {
  return xmalloc(size);
}

void *nvim_path_xrealloc(void *ptr, size_t size) {
  return xrealloc(ptr, size);
}

void nvim_path_xfree(void *ptr) {
  xfree(ptr);
}

int nvim_path_os_dirname(char *buf, size_t len) {
  return os_dirname(buf, len);
}

char *nvim_path_os_realpath(const char *name, char *buf, size_t len) {
  return os_realpath(name, buf, len);
}

char *nvim_path_xstrdup(const char *str) {
  return xstrdup(str);
}

size_t nvim_path_xstrlcpy(char *dst, const char *src, size_t dsize) {
  return xstrlcpy(dst, src, dsize);
}

// Phase 3 C accessor functions for Rust
void nvim_path_expand_env(const char *src, char *dst, size_t dstlen) {
  expand_env((char *)src, dst, (int)dstlen);
}

int nvim_path_os_fileid(const char *fname, uint8_t *id_out) {
  _Static_assert(sizeof(FileID) == 16, "FileID must be 16 bytes");
  return os_fileid(fname, (FileID *)id_out) ? 1 : 0;
}

int nvim_path_os_fileid_equal(const uint8_t *a, const uint8_t *b) {
  return os_fileid_equal((const FileID *)a, (const FileID *)b) ? 1 : 0;
}

int nvim_path_file_exists_link(const char *name) {
  FileInfo info;
  return os_fileinfo_link(name, &info) ? 1 : 0;
}

int nvim_path_scandir_open(const char *path, void **dir_out) {
  Directory *dir = xmalloc(sizeof(Directory));
  if (os_scandir(dir, path)) {
    *dir_out = dir;
    return 1;
  }
  xfree(dir);
  *dir_out = NULL;
  return 0;
}

const char *nvim_path_scandir_next(void *dir) {
  return os_scandir_next((Directory *)dir);
}

void nvim_path_scandir_close(void *dir) {
  os_closedir((Directory *)dir);
  xfree(dir);
}

int nvim_path_STRICMP(const char *a, const char *b) {
  return STRICMP(a, b);
}

size_t nvim_path_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep) {
  return copy_option_part(option, buf, (int)maxlen, (char *)sep);
}

const char *nvim_path_get_p_su(void) {
  return p_su;
}

char *nvim_path_os_getenv(const char *name) {
  return os_getenv(name);
}

int nvim_path_os_can_exe(const char *name, char **abspath, int use_path) {
  return os_can_exe(name, abspath, use_path != 0) ? 1 : 0;
}

const void *nvim_path_vim_env_iter(char sep, const char *val, const void *iter,
                                   const char **dir, size_t *len) {
  return vim_env_iter(sep, val, iter, dir, len);
}

char *nvim_path_get_NameBuff(void) {
  return NameBuff;
}

size_t nvim_path_get_NameBuff_size(void) {
  return sizeof(NameBuff);
}

size_t nvim_path_xstrlcat(char *dst, const char *src, size_t dstsize) {
  return xstrlcat(dst, src, dstsize);
}

void nvim_path_xmemcpyz(char *dst, const char *src, size_t len) {
  xmemcpyz(dst, src, len);
}

int nvim_path_ga_len(const void *gap) {
  return ((const garray_T *)gap)->ga_len;
}

const char *nvim_path_ga_get_string(const void *gap, int i) {
  return ((const char **)((const garray_T *)gap)->ga_data)[i];
}

// Phase 4 C accessor functions
int nvim_path_os_path_exists(const char *fname) {
  return os_path_exists(fname) ? 1 : 0;
}

void nvim_path_ga_append_string(void *gap, char *s) {
  GA_APPEND(char *, (garray_T *)gap, s);
}

// Phase 5 C accessor functions for Rust
int nvim_path_os_fileinfo(const char *fname, uint8_t *info_out) {
  _Static_assert(sizeof(FileInfo) <= 256, "FileInfo must fit in 256-byte opaque buffer");
  return os_fileinfo(fname, (FileInfo *)info_out) ? 1 : 0;
}

int nvim_path_os_fileinfo_link(const char *fname, uint8_t *info_out) {
  return os_fileinfo_link(fname, (FileInfo *)info_out) ? 1 : 0;
}

int nvim_path_os_fileinfo_id_equal(const uint8_t *a, const uint8_t *b) {
  return os_fileinfo_id_equal((const FileInfo *)a, (const FileInfo *)b) ? 1 : 0;
}

// Phase 6 C accessor functions for Rust
void *nvim_path_compile_pattern(const char *pat, int flags, int ic) {
  regmatch_T *rm = xmalloc(sizeof(regmatch_T));
  rm->rm_ic = ic != 0;
  rm->regprog = vim_regcomp(pat, flags);
  if (rm->regprog == NULL) {
    xfree(rm);
    return NULL;
  }
  return rm;
}

int nvim_path_match_pattern(const void *handle, const char *s) {
  return vim_regexec((regmatch_T *)handle, s, 0) ? 1 : 0;
}

void nvim_path_free_pattern(void *handle) {
  if (handle != NULL) {
    regmatch_T *rm = (regmatch_T *)handle;
    vim_regfree(rm->regprog);
    xfree(rm);
  }
}

void nvim_path_os_breakcheck(void) {
  os_breakcheck();
}

int nvim_path_get_got_int(void) {
  return got_int ? 1 : 0;
}

int nvim_path_get_p_fic(void) {
  return p_fic;
}

int nvim_path_os_file_is_readable(const char *fname) {
  return os_file_is_readable(fname) ? 1 : 0;
}

char *nvim_path_file_pat_to_reg_pat(const char *pat, const char *pat_end,
                                     char *allow_dirs, int no_bslash) {
  return file_pat_to_reg_pat(pat, pat_end, allow_dirs, no_bslash);
}

int nvim_path_rem_backslash(const char *s) {
  return rem_backslash(s) ? 1 : 0;
}

void nvim_path_backslash_halve(char *s) {
  backslash_halve(s);
}

void nvim_path_emsg_silent_inc(void) {
  emsg_silent++;
}

void nvim_path_emsg_silent_dec(void) {
  emsg_silent--;
}

int nvim_path_mb_isalpha(int c) {
  return mb_isalpha(c) ? 1 : 0;
}

int nvim_path_utf_ptr2char(const char *p) {
  return utf_ptr2char(p);
}

void nvim_path_ga_sort_strings(void *gap, int start) {
  garray_T *ga = (garray_T *)gap;
  size_t count = (size_t)(ga->ga_len - start);
  if (count > 1) {
    qsort(((char **)ga->ga_data) + start, count, sizeof(char *), pstrcmp);
  }
}

// --- Phase 7 Rust declarations ---
extern void rs_expand_path_option(char *curdir, char *path_option, void *gap);

// --- Phase 7 accessor functions ---

const char *nvim_path_get_path_option(void) {
  return *curbuf->b_p_path == NUL ? p_path : curbuf->b_p_path;
}

const char *nvim_path_curbuf_ffname(void) {
  return curbuf->b_ffname;
}

char *nvim_path_expand_env_save_opt(char *src, int one) {
  return expand_env_save_opt(src, one != 0);
}

char *nvim_path_backslash_halve_save(const char *s) {
  return backslash_halve_save(s);
}

const char *nvim_path_get_p_wig(void) {
  return p_wig;
}

int nvim_path_match_file_list(const char *list, const char *fname, const char *ffname) {
  return match_file_list((char *)list, (char *)fname, (char *)ffname) ? 1 : 0;
}

int nvim_path_os_expand_wildcards(int num_pat, char **pat, int *num_file,
                                   char ***file, int flags) {
  return os_expand_wildcards(num_pat, pat, num_file, file, flags);
}

int nvim_path_expand_backtick(void *gap, char *pat, int flags) {
  return expand_backtick((garray_T *)gap, pat, flags);
}

int nvim_path_expand_in_path(void *gap, char *pat, int flags) {
  return expand_in_path((garray_T *)gap, pat, flags);
}

void *nvim_path_ga_alloc_strings(int growsize) {
  garray_T *gap = xmalloc(sizeof(garray_T));
  ga_init(gap, (int)sizeof(char *), growsize);
  return gap;
}

void nvim_path_ga_free_handle(void *gap) {
  xfree(gap);
}

void nvim_path_ga_clear_strings(void *gap) {
  ga_clear_strings((garray_T *)gap);
}

void nvim_path_ga_remove_duplicate_strings(void *gap) {
  ga_remove_duplicate_strings((garray_T *)gap);
}

char **nvim_path_ga_get_data(const void *gap) {
  return (char **)((const garray_T *)gap)->ga_data;
}

void nvim_path_ga_set_string(void *gap, int i, char *s) {
  ((char **)((garray_T *)gap)->ga_data)[i] = s;
}

char *nvim_path_xmemdupz(const char *s, size_t len) {
  return xmemdupz(s, len);
}

void *nvim_path_xcalloc(size_t count, size_t size) {
  return xcalloc(count, size);
}

void *nvim_path_ga_get_data_ptr(const void *gap) {
  return ((const garray_T *)gap)->ga_data;
}

// Static assertions for constants used in Rust
_Static_assert(sizeof(int) == 4, "int must be 4 bytes");
_Static_assert(RE_STRING == 2, "RE_STRING");
_Static_assert(sizeof(garray_T) <= 64, "garray_T must fit in 64 bytes");
_Static_assert(EW_DIR == 0x01, "EW_DIR");
_Static_assert(EW_FILE == 0x02, "EW_FILE");
_Static_assert(EW_NOTFOUND == 0x04, "EW_NOTFOUND");
_Static_assert(EW_ADDSLASH == 0x08, "EW_ADDSLASH");
_Static_assert(EW_KEEPALL == 0x10, "EW_KEEPALL");
_Static_assert(EW_SILENT == 0x20, "EW_SILENT");
_Static_assert(EW_EXEC == 0x40, "EW_EXEC");
_Static_assert(EW_PATH == 0x80, "EW_PATH");
_Static_assert(EW_ICASE == 0x100, "EW_ICASE");
_Static_assert(EW_NOERROR == 0x200, "EW_NOERROR");
_Static_assert(EW_NOTWILD == 0x400, "EW_NOTWILD");
_Static_assert(EW_KEEPDOLLAR == 0x800, "EW_KEEPDOLLAR");
_Static_assert(EW_ALLLINKS == 0x1000, "EW_ALLLINKS");
_Static_assert(EW_SHELLCMD == 0x2000, "EW_SHELLCMD");
_Static_assert(EW_DODOT == 0x4000, "EW_DODOT");
_Static_assert(EW_EMPTYOK == 0x8000, "EW_EMPTYOK");
_Static_assert(EW_NOTENV == 0x10000, "EW_NOTENV");
_Static_assert(EW_CDPATH == 0x20000, "EW_CDPATH");
_Static_assert(EW_NOBREAK == 0x40000, "EW_NOBREAK");
_Static_assert(kEqualFiles == 1, "kEqualFiles");
_Static_assert(kDifferentFiles == 2, "kDifferentFiles");
_Static_assert(kBothFilesMissing == 4, "kBothFilesMissing");
_Static_assert(kOneFileMissing == 6, "kOneFileMissing");
_Static_assert(kEqualFileNames == 7, "kEqualFileNames");
_Static_assert(OK == 1, "OK");
_Static_assert(FAIL == 0, "FAIL");
_Static_assert(MAXPATHL >= 4096, "MAXPATHL");
_Static_assert(sizeof(FileID) == 16, "FileID must be 16 bytes for Rust opaque buffer");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC");
_Static_assert(RE_NOBREAK == 16, "RE_NOBREAK");

#include "path.c.generated.h"

/// Calls globpath() with 'path' values for the given pattern and stores the
/// result in "gap".
/// Returns the total number of matches.
///
/// @param flags  EW_* flags
static int expand_in_path(garray_T *const gap, char *const pattern, const int flags)
  FUNC_ATTR_NONNULL_ALL
{
  garray_T path_ga;
  char *path_option = *curbuf->b_p_path == NUL ? p_path : curbuf->b_p_path;

  char *const curdir = xmalloc(MAXPATHL);
  os_dirname(curdir, MAXPATHL);

  ga_init(&path_ga, (int)sizeof(char *), 1);
  if (flags & EW_CDPATH) {
    rs_expand_path_option(curdir, p_cdpath, &path_ga);
  } else {
    rs_expand_path_option(curdir, path_option, &path_ga);
  }
  xfree(curdir);
  if (GA_EMPTY(&path_ga)) {
    return 0;
  }

  char *const paths = ga_concat_strings(&path_ga, ",");
  ga_clear_strings(&path_ga);

  int glob_flags = 0;
  if (flags & EW_ICASE) {
    glob_flags |= WILD_ICASE;
  }
  if (flags & EW_ADDSLASH) {
    glob_flags |= WILD_ADD_SLASH;
  }
  globpath(paths, pattern, gap, glob_flags, !!(flags & EW_CDPATH));
  xfree(paths);

  return gap->ga_len;
}

/// Expand an item in `backticks` by executing it as a command.
/// Currently only works when pat[] starts and ends with a `.
/// Returns number of file names found, -1 if an error is encountered.
///
/// @param flags  EW_* flags
static int expand_backtick(garray_T *gap, char *pat, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  char *p;
  char *buffer;
  int cnt = 0;

  // Create the command: lop off the backticks.
  char *cmd = xmemdupz(pat + 1, strlen(pat) - 2);

  if (*cmd == '=') {          // `={expr}`: Expand expression
    buffer = eval_to_string(cmd + 1, true, false);
  } else {
    buffer = get_cmd_output(cmd, NULL, (flags & EW_SILENT) ? kShellOptSilent : 0, NULL);
  }
  xfree(cmd);
  if (buffer == NULL) {
    return -1;
  }

  cmd = buffer;
  while (*cmd != NUL) {
    cmd = skipwhite(cmd);               // skip over white space
    p = cmd;
    while (*p != NUL && *p != '\r' && *p != '\n') {  // skip over entry
      p++;
    }
    // add an entry if it is not empty
    if (p > cmd) {
      char i = *p;
      *p = NUL;
      addfile(gap, cmd, flags);
      *p = i;
      cnt++;
    }
    cmd = p;
    while (*cmd != NUL && (*cmd == '\r' || *cmd == '\n')) {
      cmd++;
    }
  }

  xfree(buffer);
  return cnt;
}

/// Try to find a shortname by comparing the fullname with `dir_name`.
///
/// Invoke expand_wildcards() for one pattern
///
/// One should expand items like "%:h" before the expansion.
///
/// @param[in]   pat       Pointer to the input pattern.
/// @param[out]  num_file  Resulting number of files.
/// @param[out]  file      Array of resulting files.
/// @param[in]   flags     Flags passed to expand_wildcards().
///
/// @returns               OK when *file is set to allocated array of matches
///                        and *num_file (can be zero) to the number of matches.
///                        If FAIL is returned, *num_file and *file are either
///                        unchanged or *num_file is set to 0 and *file is set
///                        to NULL or points to "".
int expand_wildcards_eval(char **pat, int *num_file, char ***file, int flags)
{
  int ret = FAIL;
  char *eval_pat = NULL;
  char *exp_pat = *pat;
  const char *ignored_msg;
  size_t usedlen;
  const bool is_cur_alt_file = *exp_pat == '%' || *exp_pat == '#';
  bool star_follows = false;

  if (is_cur_alt_file || *exp_pat == '<') {
    emsg_off++;
    eval_pat = eval_vars(exp_pat, exp_pat, &usedlen, NULL, &ignored_msg,
                         NULL,
                         true);
    emsg_off--;
    if (eval_pat != NULL) {
      star_follows = strcmp(exp_pat + usedlen, "*") == 0;
      exp_pat = concat_str(eval_pat, exp_pat + usedlen);
    }
  }

  if (exp_pat != NULL) {
    ret = expand_wildcards(1, &exp_pat, num_file, file, flags);
  }

  if (eval_pat != NULL) {
    if (*num_file == 0 && is_cur_alt_file && star_follows) {
      // Expanding "%" or "#" and the file does not exist: Add the
      // pattern anyway (without the star) so that this works for remote
      // files and non-file buffer names.
      *file = xmalloc(sizeof(char *));
      **file = eval_pat;
      eval_pat = NULL;
      *num_file = 1;
      ret = OK;
    }
    xfree(exp_pat);
    xfree(eval_pat);
  }

  return ret;
}

