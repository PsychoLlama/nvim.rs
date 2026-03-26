// Environment inspection

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <uv.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/eval.h"
#include "nvim/eval/fs.h"
#include "nvim/eval/vars.h"
#include "nvim/globals.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"

#ifdef MSWIN
# include "nvim/mbyte.h"
#endif

#ifdef BACKSLASH_IN_FILENAME
# include "nvim/fileio.h"
#endif

#ifdef __APPLE__
# include <mach/task.h>
#endif

#ifdef HAVE__NSGETENVIRON
# include <crt_externs.h>
#endif

#ifdef HAVE_SYS_UTSNAME_H
# include <sys/utsname.h>
#endif

#include "auto/pathdef.h"

#include "os/env.c.generated.h"


/// Like getenv(), but returns a pointer to `NameBuff` instead of allocating, or NULL on failure.
/// Value is truncated if it exceeds sizeof(NameBuff).
/// @see os_env_exists
char *os_getenv_noalloc(const char *name)
  FUNC_ATTR_NONNULL_ALL
{
  if (name[0] == NUL) {
    return NULL;
  }

  size_t size = sizeof(NameBuff);
  int r = uv_os_getenv(name, NameBuff, &size);
  if (r == UV_ENOBUFS) {
    char *e = xmalloc(size);
    r = uv_os_getenv(name, e, &size);
    if (r == 0 && size != 0 && e[0] != NUL) {
      xmemcpyz(NameBuff, e, sizeof(NameBuff) - 1);
    }
    xfree(e);
  }

  if (r != 0 || size == 0 || NameBuff[0] == NUL) {
    if (r != 0 && r != UV_ENOENT && r != UV_UNKNOWN) {
      ELOG("uv_os_getenv(%s) failed: %d %s", name, r, uv_err_name(r));
    }
    return NULL;
  }
  return NameBuff;
}





/// The "real" home directory as determined by `init_homedir`.
static char *homedir = NULL;
static char *os_uv_homedir(void);

/// Gets the "real", resolved user home directory as determined by `init_homedir`.
const char *os_homedir(void)
{
  if (!homedir) {
    emsg("os_homedir failed: homedir not initialized");
    return NULL;
  }
  return homedir;
}

/// Sets `homedir` to the "real", resolved user home directory, as follows:
///   1. get value of $HOME
///   2. if $HOME is not set, try the following
/// For Windows:
///   1. assemble homedir using HOMEDRIVE and HOMEPATH
///   2. try os_uv_homedir()
///   3. resolve a direct reference to another system variable
///   4. guess C drive
/// For Unix:
///   1. try os_uv_homedir()
///   2. go to that directory
///     This also works with mounts and links.
///     Don't do this for Windows, it will change the "current dir" for a drive.
///   3. fall back to current working directory as a last resort
void init_homedir(void)
{
  // In case we are called a second time.
  xfree(homedir);
  homedir = NULL;

  char *var = os_getenv("HOME");
  char *tofree = var;

#ifdef MSWIN
  // Typically, $HOME is not defined on Windows, unless the user has
  // specifically defined it for Vim's sake. However, on Windows NT
  // platforms, $HOMEDRIVE and $HOMEPATH are automatically defined for
  // each user. Try constructing $HOME from these.
  if (var == NULL) {
    char *homedrive = os_getenv("HOMEDRIVE");
    char *homepath = os_getenv("HOMEPATH");
    if (homepath == NULL) {
      homepath = xstrdup("\\");
    }
    if (homedrive != NULL
        && strlen(homedrive) + strlen(homepath) < MAXPATHL) {
      snprintf(os_buf, MAXPATHL, "%s%s", homedrive, homepath);
      if (os_buf[0] != NUL) {
        var = os_buf;
      }
    }
    xfree(homepath);
    xfree(homedrive);
  }
  if (var == NULL) {
    var = os_uv_homedir();
  }

  // Weird but true: $HOME may contain an indirect reference to another
  // variable, esp. "%USERPROFILE%".  Happens when $USERPROFILE isn't set
  // when $HOME is being set.
  if (var != NULL && *var == '%') {
    const char *p = strchr(var + 1, '%');
    if (p != NULL) {
      vim_snprintf(os_buf, (size_t)(p - var), "%s", var + 1);
      var = NULL;
      char *exp = os_getenv(os_buf);
      if (exp != NULL) {
        if (*exp != NUL && strlen(exp) + strlen(p) < MAXPATHL) {
          vim_snprintf(os_buf, MAXPATHL, "%s%s", exp, p + 1);
          var = os_buf;
        }
        xfree(exp);
      }
    }
  }

  // Default home dir is C:/
  // Best assumption we can make in such a situation.
  if (var == NULL
      // Empty means "undefined"
      || *var == NUL) {
    var = "C:/";
  }
#endif

#ifdef UNIX
  if (var == NULL) {
    var = os_uv_homedir();
  }

  // Get the actual path.  This resolves links.
  if (var != NULL && os_realpath(var, IObuff, IOSIZE) != NULL) {
    var = IObuff;
  }

  // Fall back to current working directory if home is not found
  if ((var == NULL || *var == NUL)
      && os_dirname(os_buf, sizeof(os_buf)) == OK) {
    var = os_buf;
  }
#endif
  if (var != NULL) {
    homedir = xstrdup(var);
  }
  xfree(tofree);
}

static char homedir_buf[MAXPATHL];

static char *os_uv_homedir(void)
{
  homedir_buf[0] = NUL;
  size_t homedir_size = MAXPATHL;
  // http://docs.libuv.org/en/v1.x/misc.html#c.uv_os_homedir
  int ret_value = uv_os_homedir(homedir_buf, &homedir_size);
  if (ret_value == 0 && homedir_size < MAXPATHL) {
    return homedir_buf;
  }
  ELOG("uv_os_homedir() failed %d: %s", ret_value, os_strerror(ret_value));
  homedir_buf[0] = NUL;
  return NULL;
}

#if defined(EXITFREE)

void free_homedir(void) { xfree(homedir); }

#endif


/// Expand environment variable with path name and escaping.
/// @see expand_env
///
/// @param srcp       Input string e.g. "$HOME/vim.hlp"
/// @param dst[out]   Where to put the result
/// @param dstlen     Maximum length of the result
/// @param esc        Escape spaces in expanded variables
/// @param one        `srcp` is a single filename
/// @param prefix     Start again after this (can be NULL)
size_t expand_env_esc(const char *restrict srcp, char *restrict dst, int dstlen, bool esc, bool one,
                      char *prefix)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  char *tail;
  char *var;
  bool copy_char;
  bool mustfree;  // var was allocated, need to free it later
  bool at_start = true;  // at start of a name
  char *const dst_start = dst;

  int prefix_len = (prefix == NULL) ? 0 : (int)strlen(prefix);

  char *src = skipwhite(srcp);
  dstlen--;  // leave one char space for "\,"
  while (*src && dstlen > 0) {
    // Skip over `=expr`.
    if (src[0] == '`' && src[1] == '=') {
      var = src;
      src += 2;
      skip_expr(&src, NULL);
      if (*src == '`') {
        src++;
      }
      size_t len = (size_t)(src - var);
      if (len > (size_t)dstlen) {
        len = (size_t)dstlen;
      }
      memcpy(dst, var, len);
      dst += len;
      dstlen -= (int)len;
      continue;
    }

    copy_char = true;
    if ((*src == '$') || (*src == '~' && at_start)) {
      mustfree = false;

      // The variable name is copied into dst temporarily, because it may
      // be a string in read-only memory and a NUL needs to be appended.
      if (*src != '~') {  // environment var
        tail = src + 1;
        var = dst;
        int c = dstlen - 1;

#ifdef UNIX
        // Unix has ${var-name} type environment vars
        if (*tail == '{' && !vim_isIDc('{')) {
          tail++;               // ignore '{'
          while (c-- > 0 && *tail != NUL && *tail != '}') {
            *var++ = *tail++;
          }
        } else
#endif
        {
          while (c-- > 0 && *tail != NUL && vim_isIDc((uint8_t)(*tail))) {
            *var++ = *tail++;
          }
        }

#if defined(UNIX)
        // Verify that we have found the end of a Unix ${VAR} style variable
        if (src[1] == '{' && *tail != '}') {
          var = NULL;
        } else {
          if (src[1] == '{') {
            tail++;
          }
#endif
        *var = NUL;
        var = vim_getenv(dst);
        mustfree = true;
#if defined(UNIX)
      }
#endif
      } else if (src[1] == NUL  // home directory
                 || vim_ispathsep(src[1])
                 || vim_strchr(" ,\t\n", (uint8_t)src[1]) != NULL) {
        var = homedir;
        tail = src + 1;
      } else {  // user directory
#if defined(UNIX)
        // Copy ~user to dst[], so we can put a NUL after it.
        tail = src;
        var = dst;
        int c = dstlen - 1;
        while (c-- > 0
               && *tail
               && vim_isfilec((uint8_t)(*tail))
               && !vim_ispathsep(*tail)) {
          *var++ = *tail++;
        }
        *var = NUL;
        // Get the user directory. If this fails the shell is used to expand
        // ~user, which is slower and may fail on old versions of /bin/sh.
        var = (*dst == NUL) ? NULL
                            : os_get_userdir(dst + 1);
        mustfree = true;
        if (var == NULL) {
          expand_T xpc;

          ExpandInit(&xpc);
          xpc.xp_context = EXPAND_FILES;
          var = ExpandOne(&xpc, dst, NULL,
                          WILD_ADD_SLASH|WILD_SILENT, WILD_EXPAND_FREE);
          mustfree = true;
        }
#else
        // cannot expand user's home directory, so don't try
        var = NULL;
        tail = "";  // for gcc
#endif  // UNIX
      }

#ifdef BACKSLASH_IN_FILENAME
      // If 'shellslash' is set change backslashes to forward slashes.
      // Can't use slash_adjust(), p_ssl may be set temporarily.
      if (p_ssl && var != NULL && vim_strchr(var, '\\') != NULL) {
        char *p = xstrdup(var);

        if (mustfree) {
          xfree(var);
        }
        var = p;
        mustfree = true;
        forward_slash(var);
      }
#endif

      // If "var" contains white space, escape it with a backslash.
      // Required for ":e ~/tt" when $HOME includes a space.
      if (esc && var != NULL && strpbrk(var, " \t") != NULL) {
        char *p = vim_strsave_escaped(var, " \t");

        if (mustfree) {
          xfree(var);
        }
        var = p;
        mustfree = true;
      }

      if (var != NULL && *var != NUL) {
        int c = (int)strlen(var);
        if ((size_t)c + strlen(tail) + 1 < (unsigned)dstlen) {
          STRCPY(dst, var);
          dstlen -= c;
          // if var[] ends in a path separator and tail[] starts
          // with it, skip a character
          if (after_pathsep(dst, dst + c)
#if defined(BACKSLASH_IN_FILENAME)
              && dst[c - 1] != ':'
#endif
              && vim_ispathsep(*tail)) {
            tail++;
          }
          dst += c;
          src = tail;
          copy_char = false;
        }
      }
      if (mustfree) {
        xfree(var);
      }
    }

    if (copy_char) {  // copy at least one char
      // Recognize the start of a new name, for '~'.
      // Don't do this when "one" is true, to avoid expanding "~" in
      // ":edit foo ~ foo".
      at_start = false;
      if (src[0] == '\\' && src[1] != NUL) {
        *dst++ = *src++;
        dstlen--;
      } else if ((src[0] == ' ' || src[0] == ',') && !one) {
        at_start = true;
      }
      if (dstlen > 0) {
        *dst++ = *src++;
        dstlen--;

        if (prefix != NULL
            && src - prefix_len >= srcp
            && strncmp(src - prefix_len, prefix, (size_t)prefix_len) == 0) {
          at_start = true;
        }
      }
    }
  }
  *dst = NUL;

  return (size_t)(dst - dst_start);
}

/// Check if the directory "vimdir/runtime" exists.
/// Return NULL if not, return its name in allocated memory otherwise.
/// @param vimdir directory to test
static char *vim_runtime_dir(const char *vimdir)
{
  if (vimdir == NULL || *vimdir == NUL) {
    return NULL;
  }
  char *p = concat_fnames(vimdir, RUNTIME_DIRNAME, true);
  if (os_isdir(p)) {
    return p;
  }
  xfree(p);
  return NULL;
}

/// If `dirname + "/"` precedes `pend` in the path, return the pointer to
/// `dirname + "/" + pend`.  Otherwise return `pend`.
///
/// Examples (path = /usr/local/share/nvim/runtime/doc/help.txt):
///
///   pend    = help.txt
///   dirname = doc
///   -> doc/help.txt
///
///   pend    = doc/help.txt
///   dirname = runtime
///   -> runtime/doc/help.txt
///
///   pend    = runtime/doc/help.txt
///   dirname = vim74
///   -> runtime/doc/help.txt
///
/// @param path    Path to a file
/// @param pend    A suffix of the path
/// @param dirname The immediate path fragment before the pend
/// @return The new pend including dirname or just pend
static char *remove_tail(char *path, char *pend, char *dirname)
{
  size_t len = strlen(dirname);
  char *new_tail = pend - len - 1;

  if (new_tail >= path
      && path_fnamencmp(new_tail, dirname, len) == 0
      && (new_tail == path || after_pathsep(path, new_tail))) {
    return new_tail;
  }
  return pend;
}


/// @param[out] exe_name should be at least MAXPATHL in size
void vim_get_prefix_from_exepath(char *exe_name)
{
  // TODO(bfredl): param could have been written as "char exe_name[MAXPATHL]"
  // but c_grammar.lua does not recognize it (yet).
  xstrlcpy(exe_name, get_vim_var_str(VV_PROGPATH), MAXPATHL * sizeof(*exe_name));
  char *path_end = path_tail_with_sep(exe_name);
  *path_end = NUL;  // remove the trailing "nvim.exe"
  path_end = path_tail(exe_name);
  *path_end = NUL;  // remove the trailing "bin/"
}

/// Vim getenv() wrapper with special handling of $HOME, $VIM, $VIMRUNTIME,
/// allowing the user to override the Nvim runtime directory at runtime.
/// Result must be freed by the caller.
///
/// @param name Environment variable to expand
/// @return [allocated] Expanded environment variable, or NULL
char *vim_getenv(const char *name)
{
  // init_path() should have been called before now.
  assert(get_vim_var_str(VV_PROGPATH)[0] != NUL);

#ifdef MSWIN
  if (strcmp(name, "HOME") == 0) {
    return xstrdup(homedir);
  }
#endif

  char *kos_env_path = os_getenv(name);
  if (kos_env_path != NULL) {
    return kos_env_path;
  }

  bool vimruntime = (strcmp(name, "VIMRUNTIME") == 0);
  if (!vimruntime && strcmp(name, "VIM") != 0) {
    return NULL;
  }

  // When expanding $VIMRUNTIME fails, try using $VIM/vim<version> or $VIM.
  // Don't do this when default_vimruntime_dir is non-empty.
  char *vim_path = NULL;
  if (vimruntime
      && *default_vimruntime_dir == NUL) {
    kos_env_path = os_getenv("VIM");    // kos_env_path was NULL.
    if (kos_env_path != NULL) {
      vim_path = vim_runtime_dir(kos_env_path);
      if (vim_path == NULL) {
        vim_path = kos_env_path;
      } else {
        xfree(kos_env_path);
      }
    }
  }

  // When expanding $VIM or $VIMRUNTIME fails, try using:
  // - the directory name from 'helpfile' (unless it contains '$')
  // - the executable name from argv[0]
  if (vim_path == NULL) {
    if (p_hf != NULL && vim_strchr(p_hf, '$') == NULL) {
      vim_path = p_hf;
    }

    char exe_name[MAXPATHL];
    // Find runtime path relative to the nvim binary: ../share/nvim/runtime
    if (vim_path == NULL) {
      vim_get_prefix_from_exepath(exe_name);
      if (append_path(exe_name, "share/nvim/runtime/", MAXPATHL) == OK) {
        vim_path = exe_name;
      }
    }

    if (vim_path != NULL) {
      // remove the file name
      char *vim_path_end = path_tail(vim_path);

      // remove "doc/" from 'helpfile', if present
      if (vim_path == p_hf) {
        vim_path_end = remove_tail(vim_path, vim_path_end, "doc");
      }

      // for $VIM, remove "runtime/", if present
      if (!vimruntime) {
        vim_path_end = remove_tail(vim_path, vim_path_end, RUNTIME_DIRNAME);
      }

      // remove trailing path separator
      if (vim_path_end > vim_path && after_pathsep(vim_path, vim_path_end)) {
        vim_path_end--;
      }

      // check that the result is a directory name
      assert(vim_path_end >= vim_path);
      vim_path = xmemdupz(vim_path, (size_t)(vim_path_end - vim_path));

      if (!os_isdir(vim_path)) {
        xfree(vim_path);
        vim_path = NULL;
      }
    }
    assert(vim_path != exe_name);
  }

  // When there is a pathdef.c file we can use default_vim_dir and
  // default_vimruntime_dir
  if (vim_path == NULL) {
    // Only use default_vimruntime_dir when it is not empty
    if (vimruntime && *default_vimruntime_dir != NUL) {
      vim_path = xstrdup(default_vimruntime_dir);
    } else if (*default_vim_dir != NUL) {
      if (vimruntime
          && (vim_path = vim_runtime_dir(default_vim_dir)) == NULL) {
        vim_path = xstrdup(default_vim_dir);
      }
    }
  }

  // Set the environment variable, so that the new value can be found fast
  // next time, and others can also use it (e.g. Perl).
  if (vim_path != NULL) {
    if (vimruntime) {
      os_setenv("VIMRUNTIME", vim_path, 1);
      didset_vimruntime = true;
    } else {
      os_setenv("VIM", vim_path, 1);
      didset_vim = true;
    }
  }
  return vim_path;
}

/// Replace home directory by "~" in each space or comma separated file name in
/// 'src'.
///
/// Replace home directory with tilde in each file name
///
/// If anything fails (except when out of space) dst equals src.
///
/// @param[in]  buf  When not NULL, uses this buffer to check whether it is
///                  a help file. If it is then path to file is removed
///                  completely, `one` is ignored and assumed to be true.
/// @param[in]  src  Input file names. Assumed to be a space/comma separated
///                  list unless `one` is true.
/// @param[out]  dst  Where to put the result.
/// @param[in]  dstlen  Destination length.
/// @param[in]  one  If true, assumes source is a single file name and not
///                  a list of them.
///
/// @return length of the string put into dst, does not include NUL byte.
size_t home_replace(const buf_T *const buf, const char *src, char *const dst, size_t dstlen,
                    const bool one)
  FUNC_ATTR_NONNULL_ARG(3)
{
  size_t dirlen = 0;
  size_t envlen = 0;

  if (src == NULL) {
    *dst = NUL;
    return 0;
  }

  if (buf != NULL && buf->b_help) {
    const size_t dlen = xstrlcpy(dst, path_tail(src), dstlen);
    return MIN(dlen, dstlen - 1);
  }

  // We check both the value of the $HOME environment variable and the
  // "real" home directory.
  if (homedir != NULL) {
    dirlen = strlen(homedir);
  }

  char *homedir_env = os_getenv("HOME");
#ifdef MSWIN
  if (homedir_env == NULL) {
    homedir_env = os_getenv("USERPROFILE");
  }
#endif
  char *homedir_env_mod = homedir_env;
  bool must_free = false;

  if (homedir_env_mod != NULL && *homedir_env_mod == '~') {
    must_free = true;
    size_t usedlen = 0;
    size_t flen = strlen(homedir_env_mod);
    char *fbuf = NULL;
    modify_fname(":p", false, &usedlen, &homedir_env_mod, &fbuf, &flen);
    flen = strlen(homedir_env_mod);
    assert(homedir_env_mod != homedir_env);
    if (vim_ispathsep(homedir_env_mod[flen - 1])) {
      // Remove the trailing / that is added to a directory.
      homedir_env_mod[flen - 1] = NUL;
    }
  }

  if (homedir_env_mod != NULL) {
    envlen = strlen(homedir_env_mod);
  }

  if (!one) {
    src = skipwhite(src);
  }
  char *dst_p = dst;
  while (*src && dstlen > 0) {
    // Here we are at the beginning of a file name.
    // First, check to see if the beginning of the file name matches
    // $HOME or the "real" home directory. Check that there is a '/'
    // after the match (so that if e.g. the file is "/home/pieter/bla",
    // and the home directory is "/home/piet", the file does not end up
    // as "~er/bla" (which would seem to indicate the file "bla" in user
    // er's home directory)).
    char *p = homedir;
    size_t len = dirlen;
    while (true) {
      if (len
          && path_fnamencmp(src, p, len) == 0
          && (vim_ispathsep(src[len])
              || (!one && (src[len] == ',' || src[len] == ' '))
              || src[len] == NUL)) {
        src += len;
        if (--dstlen > 0) {
          *dst_p++ = '~';
        }

        // Do not add directory separator into dst, because dst is
        // expected to just return the directory name without the
        // directory separator '/'.
        break;
      }
      if (p == homedir_env_mod) {
        break;
      }
      p = homedir_env_mod;
      len = envlen;
    }

    if (dstlen == 0) {
      break;  // Avoid overflowing below.
    }
    // if (!one) skip to separator: space or comma.
    while (*src && (one || (*src != ',' && *src != ' ')) && --dstlen > 0) {
      *dst_p++ = *src++;
    }
    if (dstlen == 0) {
      break;  // Avoid overflowing below.
    }
    // Skip separator.
    while ((*src == ' ' || *src == ',') && --dstlen > 0) {
      *dst_p++ = *src++;
    }
  }
  // If (dstlen == 0) out of space, what to do???

  *dst_p = NUL;

  xfree(homedir_env);

  if (must_free) {
    xfree(homedir_env_mod);
  }
  return (size_t)(dst_p - dst);
}

/// Like home_replace, store the replaced string in allocated memory.
/// @param buf When not NULL, check for help files
/// @param src Input file name
