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

// C accessor functions for Rust
void *nvim_path_xmalloc(size_t size) {
  return xmalloc(size);
}

void *nvim_path_xrealloc(void *ptr, size_t size) {
  return xrealloc(ptr, size);
}

void nvim_path_xfree(void *ptr) {
  xfree(ptr);
}

char *nvim_path_xstrdup(const char *str) {
  return xstrdup(str);
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

const char *nvim_path_get_p_su(void) {
  return p_su;
}

int nvim_path_os_can_exe(const char *name, char **abspath, int use_path) {
  return os_can_exe(name, abspath, use_path != 0) ? 1 : 0;
}

char *nvim_path_get_NameBuff(void) {
  return NameBuff;
}

size_t nvim_path_get_NameBuff_size(void) {
  return sizeof(NameBuff);
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

void nvim_path_emsg_silent_inc(void) {
  emsg_silent++;
}

void nvim_path_emsg_silent_dec(void) {
  emsg_silent--;
}

void nvim_path_emsg_off_inc(void) {
  emsg_off++;
}

void nvim_path_emsg_off_dec(void) {
  emsg_off--;
}

void nvim_path_ga_sort_strings(void *gap, int start) {
  garray_T *ga = (garray_T *)gap;
  size_t count = (size_t)(ga->ga_len - start);
  if (count > 1) {
    qsort(((char **)ga->ga_data) + start, count, sizeof(char *), pstrcmp);
  }
}

// Rust declarations
extern void rs_expand_path_option(char *curdir, char *path_option, void *gap);

const char *nvim_path_get_path_option(void) {
  return *curbuf->b_p_path == NUL ? p_path : curbuf->b_p_path;
}

const char *nvim_path_curbuf_ffname(void) {
  return curbuf->b_ffname;
}

const char *nvim_path_get_p_wig(void) {
  return p_wig;
}

const char *nvim_path_get_p_cdpath(void) {
  return p_cdpath;
}

int nvim_path_os_expand_wildcards(int num_pat, char **pat, int *num_file,
                                   char ***file, int flags) {
  return os_expand_wildcards(num_pat, pat, num_file, file, flags);
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

