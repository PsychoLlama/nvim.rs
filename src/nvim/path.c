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

// Rust implementations - declarations
extern int rs_vim_ispathsep(int c);
extern int rs_vim_ispathsep_nocolon(int c);
extern int rs_vim_ispathlistsep(int c);
extern int rs_path_head_length(void);
extern int rs_is_path_head(const char *path);
extern const char *rs_get_past_head(const char *path);
extern int rs_path_is_absolute(const char *path);
extern int rs_path_is_url(const char *p);
extern const char *rs_path_tail(const char *fname);
extern int rs_path_has_drive_letter(const char *p, size_t path_len);
extern int rs_path_with_url(const char *fname);
extern int rs_vim_isAbsName(const char *name);
extern int rs_after_pathsep(const char *b, const char *p);
extern int rs_path_fnamecmp(const char *fname1, const char *fname2);
extern int rs_path_fnamencmp(const char *fname1, const char *fname2, size_t len);
extern const char *rs_gettail_dir(const char *fname);
extern const char *rs_path_next_component(const char *fname);
extern const char *rs_path_tail_with_sep(const char *fname);
extern const char *rs_invocation_path_tail(const char *invocation, size_t *len);
extern int rs_path_has_wildcard(const char *p);
extern int rs_path_has_exp_wildcard(const char *p);
extern int rs_pathcmp(const char *p, const char *q, int maxlen);
extern int rs_add_pathsep(char *p);
extern int rs_append_path(char *path, const char *to_append, size_t max_len);
extern int rs_path_with_extension(const char *path, const char *extension);
extern char *rs_path_shorten_fname(char *full_path, char *dir_name);
extern void rs_shorten_dir_len(char *str, int trim_len);
extern void rs_shorten_dir(char *str);
extern int rs_pstrcmp(const void *a, const void *b);
extern int rs_vim_backtick(const char *p);
extern int rs_has_env_var(const char *p);
extern void rs_FreeWild(int count, char **files);
extern int rs_dir_of_file_exists(char *fname);
extern char *rs_do_concat_fnames(char *fname1, size_t len1, const char *fname2, size_t len2, int sep);
extern char *rs_concat_fnames(const char *fname1, const char *fname2, int sep);
extern char *rs_concat_fnames_realloc(char *fname1, const char *fname2, int sep);
extern int rs_path_full_dir_name(const char *directory, char *buffer, size_t len);
extern int rs_path_to_absolute(const char *fname, char *buf, size_t len, int force);
extern int rs_vim_FullName(const char *fname, char *buf, size_t len, int force);
extern char *rs_FullName_save(const char *fname, int force);
extern char *rs_save_abs_path(const char *name);
extern char *rs_fix_fname(const char *fname);
extern int rs_same_directory(const char *f1, const char *f2);
extern char *rs_path_try_shorten_fname(char *full_path);
extern void rs_slash_adjust(char *p);
extern int rs_path_full_compare(const char *s1, const char *s2, int checkname, int expandenv);
extern void rs_path_fix_case(char *name);
extern int rs_match_suffix(const char *fname);
extern void rs_path_guess_exepath(const char *argv0, char *buf, size_t bufsize);
extern int rs_find_previous_pathsep(const char *path, char **psep);
extern int rs_is_unique(const char *maybe_unique, const void *gap, int i);
extern int rs_has_special_wildchar(const char *p, int flags);
extern const char *rs_get_path_cutoff(const char *fname, const void *gap);
extern const char *rs_scandir_next_with_dots(void *dir);
extern void rs_addfile(void *gap, const char *f, int flags);
extern size_t rs_simplify_filename(char *filename);
extern size_t rs_path_expand(void *gap, const char *path, int flags);
extern size_t rs_do_path_expand(void *gap, const char *path, size_t wildoff, int flags, int didstar);

// Forward declarations (defined below, after #include "path.c.generated.h")
static int pstrcmp(const void *a, const void *b);
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
extern void rs_uniquefy_paths(void *gap, char *pattern, char *path_option);
extern int rs_gen_expand_wildcards(int num_pat, char **pat, int *num_file,
                                    char ***file, int flags);
extern int rs_expand_wildcards(int num_pat, char **pat, int *num_files,
                                char ***files, int flags);

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

int nvim_path_ga_get_len(const void *gap) {
  return ((const garray_T *)gap)->ga_len;
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

/// Compare two file names.
///
/// @param s1 First file name. Environment variables in this name will be expanded.
/// @param s2 Second file name.
/// @param checkname When both files don't exist, only compare their names.
/// @param expandenv Whether to expand environment variables in file names.
/// @return Enum of type FileComparison. @see FileComparison.
FileComparison path_full_compare(char *const s1, char *const s2, const bool checkname,
                                 const bool expandenv)
  FUNC_ATTR_NONNULL_ALL
{
  return (FileComparison)rs_path_full_compare(s1, s2, checkname ? 1 : 0, expandenv ? 1 : 0);
}

/// Gets the tail (filename segment) of path `fname`.
///
/// Examples:
/// - "dir/file.txt" => "file.txt"
/// - "file.txt" => "file.txt"
/// - "dir/" => ""
///
/// @return pointer just past the last path separator (empty string, if fname
///         ends in a slash), or empty string if fname is NULL.
char *path_tail(const char *fname)
  FUNC_ATTR_NONNULL_RET
{
  return (char *)rs_path_tail(fname);
}

/// Get pointer to tail of "fname", including path separators.
///
/// Takes care of "c:/" and "//". That means `path_tail_with_sep("dir///file.txt")`
/// will return a pointer to `"///file.txt"`.
/// @param fname A file path. (Must be != NULL.)
/// @return
///   - Pointer to the last path separator of `fname`, if there is any.
///   - `fname` if it contains no path separator.
///   - Never NULL.
char *path_tail_with_sep(char *fname)
  FUNC_ATTR_NONNULL_ALL
{
  return (char *)rs_path_tail_with_sep(fname);
}

/// Finds the path tail (or executable) in an invocation.
///
/// @param[in]  invocation A program invocation in the form:
///                        "path/to/exe [args]".
/// @param[out] len Stores the length of the executable name.
///
/// @post if `len` is not null, stores the length of the executable name.
///
/// @return The position of the last path separator + 1.
const char *invocation_path_tail(const char *invocation, size_t *len)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_NONNULL_ARG(1)
{
  return rs_invocation_path_tail(invocation, len);
}

/// Get the next path component of a path name.
///
/// @param fname A file path. (Must be != NULL.)
/// @return Pointer to first found path separator + 1.
/// An empty string, if `fname` doesn't contain a path separator,
const char *path_next_component(const char *fname)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_next_component(fname);
}

/// Returns the length of the path head on the current platform.
/// @return
///   - 3 on windows
///   - 1 otherwise
int path_head_length(void)
{
  return rs_path_head_length();
}

/// Returns true if path begins with characters denoting the head of a path
/// (e.g. '/' on linux and 'D:' on windows).
/// @param path The path to be checked.
/// @return
///   - True if path begins with a path head
///   - False otherwise
bool is_path_head(const char *path)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_is_path_head(path) != 0;
}

/// Get a pointer to one character past the head of a path name.
/// Unix: after "/"; Win: after "c:\"
/// If there is no head, path is returned.
char *get_past_head(const char *path)
  FUNC_ATTR_NONNULL_ALL
{
  return (char *)rs_get_past_head(path);
}

/// @return true if 'c' is a path separator.
/// Note that for MS-Windows this includes the colon.
bool vim_ispathsep(int c)
{
  return rs_vim_ispathsep(c) != 0;
}

// Like vim_ispathsep(c), but exclude the colon for MS-Windows.
bool vim_ispathsep_nocolon(int c)
{
  return rs_vim_ispathsep_nocolon(c) != 0;
}

/// @return true if 'c' is a path list separator.
bool vim_ispathlistsep(int c)
{
  return rs_vim_ispathlistsep(c) != 0;
}

/// Shorten the path of a file from "~/foo/../.bar/fname" to "~/f/../.b/fname"
/// "trim_len" specifies how many characters to keep for each directory.
/// Must be 1 or more.
/// It's done in-place.
void shorten_dir_len(char *str, int trim_len)
  FUNC_ATTR_NONNULL_ALL
{
  rs_shorten_dir_len(str, trim_len);
}

/// Shorten the path of a file from "~/foo/../.bar/fname" to "~/f/../.b/fname"
/// It's done in-place.
void shorten_dir(char *str)
  FUNC_ATTR_NONNULL_ALL
{
  rs_shorten_dir(str);
}

/// Return true if the directory of "fname" exists, false otherwise.
/// Also returns true if there is no directory name.
/// "fname" must be writable!.
bool dir_of_file_exists(char *fname)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_dir_of_file_exists(fname) != 0;
}

/// Compare two file names
///
/// On some systems case in a file name does not matter, on others it does.
///
/// @note Does not account for maximum name lengths and things like "../dir",
///       thus it is not 100% accurate. OS may also use different algorithm for
///       case-insensitive comparison.
///
/// Handles '/' and '\\' correctly and deals with &fileignorecase option.
///
/// @param[in]  fname1  First file name.
/// @param[in]  fname2  Second file name.
///
/// @return 0 if they are equal, non-zero otherwise.
int path_fnamecmp(const char *fname1, const char *fname2)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_path_fnamecmp(fname1, fname2);
}

/// Compare two file names
///
/// Handles '/' and '\\' correctly and deals with &fileignorecase option.
///
/// @param[in]  fname1  First file name.
/// @param[in]  fname2  Second file name.
/// @param[in]  len  Compare at most len bytes.
///
/// @return 0 if they are equal, non-zero otherwise.
int path_fnamencmp(const char *const fname1, const char *const fname2, size_t len)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_path_fnamencmp(fname1, fname2, len);
}

/// Append fname2 to fname1
///
/// @param[in]  fname1  First fname to append to.
/// @param[in]  len1    Length of fname1.
/// @param[in]  fname2  Second part of the file name.
/// @param[in]  len2    Length of fname2.
/// @param[in]  sep     If true and fname1 does not end with a path separator,
///                     add a path separator before fname2.
///
/// @return fname1
static inline char *do_concat_fnames(char *fname1, const size_t len1, const char *fname2,
                                     const size_t len2, const bool sep)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  return rs_do_concat_fnames(fname1, len1, fname2, len2, sep ? 1 : 0);
}

/// Concatenate file names fname1 and fname2 into allocated memory.
///
/// Only add a '/' or '\\' when 'sep' is true and it is necessary.
///
/// @param fname1 is the first part of the path or filename
/// @param fname2 is the second half of the path or filename
/// @param sep    is a flag to indicate a path separator should be added
///               if necessary
/// @return [allocated] Concatenation of fname1 and fname2.
char *concat_fnames(const char *fname1, const char *fname2, bool sep)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  return rs_concat_fnames(fname1, fname2, sep ? 1 : 0);
}

/// Concatenate file names fname1 and fname2
///
/// Like concat_fnames(), but in place of allocating new memory it reallocates
/// fname1. For this reason fname1 must be allocated with xmalloc, and can no
/// longer be used after running concat_fnames_realloc.
///
/// @param fname1 is the first part of the path or filename
/// @param fname2 is the second half of the path or filename
/// @param sep    is a flag to indicate a path separator should be added
///               if necessary
/// @return [allocated] Concatenation of fname1 and fname2.
char *concat_fnames_realloc(char *fname1, const char *fname2, bool sep)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  return rs_concat_fnames_realloc(fname1, fname2, sep ? 1 : 0);
}

/// Adds a path separator to a filename, unless it already ends in one.
///
/// @return `true` if the path separator was added or already existed.
///         `false` if the filename is too long.
bool add_pathsep(char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_add_pathsep(p) != 0;
}

/// Get an allocated copy of the full path to a file.
///
/// @param fname is the filename to save
/// @param force is a flag to expand `fname` even if it looks absolute
///
/// @return [allocated] Copy of absolute path to `fname` or NULL when
///                     `fname` is NULL.
char *FullName_save(const char *fname, bool force)
  FUNC_ATTR_MALLOC
{
  return rs_FullName_save(fname, force ? 1 : 0);
}

/// Saves the absolute path.
/// @param name An absolute or relative path.
/// @return The absolute path of `name`.
char *save_abs_path(const char *name)
  FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_ALL
{
  return rs_save_abs_path(name);
}

/// Checks if a path has a wildcard character including '~', unless at the end.
/// @param p  The path to expand.
/// @returns Unix: True if it contains one of "?[{`'$".
/// @returns Windows: True if it contains one of "*?$[".
bool path_has_wildcard(const char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_has_wildcard(p) != 0;
}

static int pstrcmp(const void *a, const void *b)
{
  return rs_pstrcmp(a, b);
}

/// Checks if a path has a character path_expand can expand.
/// @param p  The path to expand.
/// @returns Unix: True if it contains one of *?[{.
/// @returns Windows: True if it contains one of *?[.
bool path_has_exp_wildcard(const char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_has_exp_wildcard(p) != 0;
}

/// Recursively expands one path component into all matching files and/or
/// directories. Handles "*", "?", "[a-z]", "**", etc.
/// @remark "**" in `path` requests recursive expansion.
///
/// @param[out] gap  The matches found.
/// @param path     The path to search.
/// @param flags    Flags for regexp expansion.
///   - EW_ICASE: Ignore case.
///   - EW_NOERROR: Silence error messages.
///   - EW_NOTWILD: Add matches literally.
/// @returns the number of matches found.
static size_t path_expand(garray_T *gap, const char *path, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_expand(gap, path, flags);
}

static const char *scandir_next_with_dots(Directory *dir)
{
  return rs_scandir_next_with_dots(dir);
}

/// Implementation of path_expand().
///
/// Chars before `path + wildoff` do not get expanded.
static size_t do_path_expand(garray_T *gap, const char *path, size_t wildoff, int flags,
                             bool didstar)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_do_path_expand(gap, path, wildoff, flags, didstar ? 1 : 0);
}

// Moves "*psep" back to the previous path separator in "path".
// Returns FAIL is "*psep" ends up at the beginning of "path".
static int find_previous_pathsep(char *path, char **psep)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_find_previous_pathsep(path, psep);
}

/// Returns true if "maybe_unique" is unique wrt other_paths in "gap".
/// "maybe_unique" is the end portion of "((char **)gap->ga_data)[i]".
static bool is_unique(char *maybe_unique, garray_T *gap, int i)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_is_unique(maybe_unique, gap, i) != 0;
}

/// Split the 'path' option into an array of strings in garray_T.  Relative
/// paths are expanded to their equivalent fullpath.  This includes the "."
/// (relative to current buffer directory) and empty path (relative to current
/// directory) notations.
///
/// @param path_option  p_path or p_cdpath
///
/// TODO(vim): handle upward search (;) and path limiter (**N) notations by
/// expanding each into their equivalent path(s).
static void expand_path_option(char *curdir, char *path_option, garray_T *gap)
  FUNC_ATTR_NONNULL_ALL
{
  rs_expand_path_option(curdir, path_option, gap);
}

// Returns a pointer to the file or directory name in "fname" that matches the
// longest path in "ga"p, or NULL if there is no match. For example:
//
//    path: /foo/bar/baz
//   fname: /foo/bar/baz/quux.txt
// returns:              ^this
static char *get_path_cutoff(char *fname, garray_T *gap)
  FUNC_ATTR_NONNULL_ALL
{
  return (char *)rs_get_path_cutoff(fname, gap);
}

/// Sorts, removes duplicates and modifies all the fullpath names in "gap" so
/// that they are unique with respect to each other while conserving the part
/// that matches the pattern. Beware, this is at least O(n^2) wrt "gap->ga_len".
///
/// @param path_option  p_path or p_cdpath
static void uniquefy_paths(garray_T *gap, char *pattern, char *path_option)
  FUNC_ATTR_NONNULL_ALL
{
  rs_uniquefy_paths(gap, pattern, path_option);
}

/// Find end of the directory name
///
/// @param[in]  fname  File name to process.
///
/// @return end of the directory name, on the first path separator:
///
///            "/path/file", "/path/dir/", "/path//dir", "/file"
///                  ^             ^             ^        ^
const char *gettail_dir(const char *const fname)
  FUNC_ATTR_PURE FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return rs_gettail_dir(fname);
}

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
    expand_path_option(curdir, p_cdpath, &path_ga);
  } else {
    expand_path_option(curdir, path_option, &path_ga);
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

/// Return true if "p" contains what looks like an environment variable.
/// Allowing for escaping.
static bool has_env_var(char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_has_env_var(p) != 0;
}

#ifdef SPECIAL_WILDCHAR

// Return true if "p" contains a special wildcard character, one that Vim
// cannot expand, requires using a shell.
static bool has_special_wildchar(char *p, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_has_special_wildchar(p, flags) != 0;
}
#endif

/// Generic wildcard expansion code.
///
/// Characters in pat that should not be expanded must be preceded with a
/// backslash. E.g., "/path\ with\ spaces/my\*star*".
///
/// @param      num_pat  is number of input patterns.
/// @param      pat      is an array of pointers to input patterns.
/// @param[out] num_file is pointer to number of matched file names.
/// @param[out] file     is pointer to array of pointers to matched file names.
/// @param      flags    is a combination of EW_* flags used in
///                      expand_wildcards().
///
/// @returns             OK when some files were found. *num_file is set to the
///                      number of matches, *file to the allocated array of
///                      matches. Call FreeWild() later.
///                      If FAIL is returned, *num_file and *file are either
///                      unchanged or *num_file is set to 0 and *file is set
///                      to NULL or points to "".
int gen_expand_wildcards(int num_pat, char **pat, int *num_file, char ***file, int flags)
{
  return rs_gen_expand_wildcards(num_pat, pat, num_file, file, flags);
}

/// Free the list of files returned by expand_wildcards() or other expansion functions.
void FreeWild(int count, char **files)
{
  rs_FreeWild(count, files);
}

/// @return  true if we can expand this backtick thing here.
static bool vim_backtick(char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_vim_backtick(p) != 0;
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

#ifdef BACKSLASH_IN_FILENAME
/// Replace all slashes by backslashes.
void slash_adjust(char *p)
  FUNC_ATTR_NONNULL_ALL
{
  rs_slash_adjust(p);
}
#endif

/// Add a file to a file list.  Accepted flags:
/// EW_DIR      add directories
/// EW_FILE     add files
/// EW_EXEC     add executable files
/// EW_NOTFOUND add even when it doesn't exist
/// EW_ADDSLASH add slash after directory name
/// EW_ALLLINKS add symlink also when the referred file does not exist
///
/// @param f  filename
void addfile(garray_T *gap, char *f, int flags)
  FUNC_ATTR_NONNULL_ALL
{
  rs_addfile(gap, f, flags);
}

// Converts a file name into a canonical form. It simplifies a file name into
// its simplest form by stripping out unneeded components, if any.  The
// resulting file name is simplified in place and will either be the same
// length as that supplied, or shorter.
size_t simplify_filename(char *filename)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_simplify_filename(filename);
}

/// Checks for a Windows drive letter ("C:/") at the start of the path.
///
/// @see https://url.spec.whatwg.org/#start-with-a-windows-drive-letter
bool path_has_drive_letter(const char *p, size_t path_len)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_has_drive_letter(p, path_len) != 0;
}

// Check if the ":/" of a URL is at the pointer, return URL_SLASH.
// Also check for ":\\", which MS Internet Explorer accepts, return
// URL_BACKSLASH.
int path_is_url(const char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_is_url(p);
}

/// Check if "fname" starts with "name://" or "name:\\".
///
/// @param  fname         is the filename to test
/// @return URL_SLASH for "name://", URL_BACKSLASH for "name:\\", zero otherwise.
int path_with_url(const char *fname)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_with_url(fname);
}

bool path_with_extension(const char *path, const char *extension)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_with_extension(path, extension) != 0;
}

/// Return true if "name" is a full (absolute) path name or URL.
bool vim_isAbsName(const char *name)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_vim_isAbsName(name) != 0;
}

/// Save absolute file name to "buf[len]".
///
/// @param      fname filename to evaluate
/// @param[out] buf   contains `fname` absolute path, or:
///                   - truncated `fname` if longer than `len`
///                   - unmodified `fname` if absolute path fails or is a URL
/// @param      len   length of `buf`
/// @param      force flag to force expanding even if the path is absolute
///
/// @return           FAIL for failure, OK otherwise
int vim_FullName(const char *fname, char *buf, size_t len, bool force)
  FUNC_ATTR_NONNULL_ARG(2)
{
  return rs_vim_FullName(fname, buf, len, force ? 1 : 0);
}

/// Get the full resolved path for `fname`
///
/// Even filenames that appear to be absolute based on starting from
/// the root may have relative paths (like dir/../subdir) or symlinks
/// embedded, or even extra separators (//).  This function addresses
/// those possibilities, returning a resolved absolute path.
/// For MS-Windows, this also expands names like "longna~1".
///
/// @param fname is the filename to expand
/// @return [allocated] Full path (NULL for failure).
char *fix_fname(const char *fname)
{
  return rs_fix_fname(fname);
}

/// Set the case of the file name, if it already exists.  This will cause the
/// file name to remain exactly the same.
/// Only required for file systems where case is ignored and preserved.
// TODO(SplinterOfChaos): Could also be used when mounting case-insensitive
// file systems.
void path_fix_case(char *name)
  FUNC_ATTR_NONNULL_ALL
{
  rs_path_fix_case(name);
}

/// Return true if "p" points to just after a path separator.
/// Takes care of multi-byte characters.
/// "b" must point to the start of the file name
int after_pathsep(const char *b, const char *p)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_after_pathsep(b, p);
}

/// Return true if file names "f1" and "f2" are in the same directory.
/// "f1" may be a short name, "f2" must be a full path.
bool same_directory(char *f1, char *f2)
{
  return rs_same_directory(f1, f2) != 0;
}

// Compare path "p[]" to "q[]".
// If `maxlen` >= 0 compare `p[maxlen]` to `q[maxlen]`
// Return value like strcmp(p, q), but consider path separators.
//
// See also `path_full_compare`.
int pathcmp(const char *p, const char *q, int maxlen)
{
  return rs_pathcmp(p, q, maxlen);
}

/// Try to find a shortname by comparing the fullname with the current
/// directory.
///
/// @param full_path The full path of the file.
/// @return
///   - Pointer into `full_path` if shortened.
///   - `full_path` unchanged if no shorter name is possible.
///   - NULL if `full_path` is NULL.
char *path_try_shorten_fname(char *full_path)
{
  return rs_path_try_shorten_fname(full_path);
}

/// Try to find a shortname by comparing the fullname with `dir_name`.
///
/// @param full_path The full path of the file.
/// @param dir_name The directory to shorten relative to.
/// @return
///   - Pointer into `full_path` if shortened.
///   - NULL if no shorter name is possible.
char *path_shorten_fname(char *full_path, char *dir_name)
{
  return rs_path_shorten_fname(full_path, dir_name);
}

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

/// Expand wildcards. Calls gen_expand_wildcards() and removes files matching
/// 'wildignore'.
///
/// @param      num_pat  is number of input patterns.
/// @param      pat      is an array of pointers to input patterns.
/// @param[out] num_file is pointer to number of matched file names.
/// @param[out] file     is pointer to array of pointers to matched file names.
/// @param      flags    is a combination of EW_* flags.
///
/// @returns             OK when *file is set to allocated array of matches
///                      and *num_file (can be zero) to the number of matches.
///                      If FAIL is returned, *num_file and *file are either
///                      unchanged or *num_file is set to 0 and *file is set to
///                      NULL or points to "".
int expand_wildcards(int num_pat, char **pat, int *num_files, char ***files, int flags)
{
  return rs_expand_wildcards(num_pat, pat, num_files, files, flags);
}

/// @return  true if "fname" matches with an entry in 'suffixes'.
bool match_suffix(char *fname)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_match_suffix(fname) != 0;
}

/// Get the absolute name of the given relative directory.
///
/// @param directory Directory name, relative to current directory.
/// @return `FAIL` for failure, `OK` for success.
int path_full_dir_name(char *directory, char *buffer, size_t len)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_full_dir_name(directory, buffer, len);
}

// Append to_append to path with a slash in between.
int append_path(char *path, const char *to_append, size_t max_len)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_append_path(path, to_append, max_len);
}

/// Used by `vim_FullName` and `fix_fname` to expand a filename to its full path.
///
/// @param  fname  Filename to expand.
/// @param  buf    Where to store the absolute path of "fname".
/// @param  len    Length of `buf`.
/// @param  force  Also expand when `fname` is already absolute.
///
/// @return FAIL for failure, OK for success.
static int path_to_absolute(const char *fname, char *buf, size_t len, int force)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_to_absolute(fname, buf, len, force);
}

/// Check if file `fname` is a full (absolute) path.
///
/// @return `true` if "fname" is absolute.
bool path_is_absolute(const char *fname)
  FUNC_ATTR_NONNULL_ALL
{
  return rs_path_is_absolute(fname) != 0;
}

/// Builds a full path from an invocation name `argv0`, based on heuristics.
///
/// @param[in]  argv0     Name by which Nvim was invoked.
/// @param[out] buf       Guessed full path to `argv0`.
/// @param[in]  bufsize   Size of `buf`.
///
/// @see os_exepath
void path_guess_exepath(const char *argv0, char *buf, size_t bufsize)
  FUNC_ATTR_NONNULL_ALL
{
  rs_path_guess_exepath(argv0, buf, bufsize);
}
