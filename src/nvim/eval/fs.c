// eval/fs.c: Filesystem related builtin functions

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "auto/config.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/fs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_docmd.h"
#include "nvim/file_search.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/os/fileio.h"
#include "nvim/os/fileio_defs.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/os.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "eval/fs.c.generated.h"

// Rust FFI declarations (window wrappers removed)
extern tabpage_T *rs_find_tabpage(int n);

static const char e_error_while_writing_str[] = N_("E80: Error while writing: %s");

/// Adjust a filename, according to a string of modifiers.
/// *fnamep must be NUL terminated when called.  When returning, the length is
/// determined by *fnamelen.
/// Returns VALID_ flags or -1 for failure.
/// When there is an error, *fnamep is set to NULL.
///
/// @param src  string with modifiers
/// @param tilde_file  "~" is a file name, not $HOME
/// @param usedlen  characters after src that are used
/// @param fnamep  file name so far
/// @param bufp  buffer for allocated file name or NULL
/// @param fnamelen  length of fnamep
int modify_fname(char *src, bool tilde_file, size_t *usedlen, char **fnamep, char **bufp,
                 size_t *fnamelen)
{
  int valid = 0;
  char *s, *p, *pbuf;
  char dirname[MAXPATHL];
  bool has_fullname = false;
  bool has_homerelative = false;

repeat:
  // ":p" - full path/file_name
  if (src[*usedlen] == ':' && src[*usedlen + 1] == 'p') {
    has_fullname = true;

    valid |= VALID_PATH;
    *usedlen += 2;

    // Expand "~/path" for all systems and "~user/path" for Unix
    if ((*fnamep)[0] == '~'
#if !defined(UNIX)
        && ((*fnamep)[1] == '/'
# ifdef BACKSLASH_IN_FILENAME
            || (*fnamep)[1] == '\\'
# endif
            || (*fnamep)[1] == NUL)
#endif
        && !(tilde_file && (*fnamep)[1] == NUL)) {
      *fnamep = expand_env_save(*fnamep);
      xfree(*bufp);          // free any allocated file name
      *bufp = *fnamep;
      if (*fnamep == NULL) {
        return -1;
      }
    }

    // When "/." or "/.." is used: force expansion to get rid of it.
    for (p = *fnamep; *p != NUL; MB_PTR_ADV(p)) {
      if (vim_ispathsep(*p)
          && p[1] == '.'
          && (p[2] == NUL
              || vim_ispathsep(p[2])
              || (p[2] == '.'
                  && (p[3] == NUL || vim_ispathsep(p[3]))))) {
        break;
      }
    }

    // FullName_save() is slow, don't use it when not needed.
    if (*p != NUL || !vim_isAbsName(*fnamep)) {
      *fnamep = FullName_save(*fnamep, *p != NUL);
      xfree(*bufp);          // free any allocated file name
      *bufp = *fnamep;
      if (*fnamep == NULL) {
        return -1;
      }
    }

    // Append a path separator to a directory.
    if (os_isdir(*fnamep)) {
      // Make room for one or two extra characters.
      *fnamep = xstrnsave(*fnamep, strlen(*fnamep) + 2);
      xfree(*bufp);          // free any allocated file name
      *bufp = *fnamep;
      add_pathsep(*fnamep);
    }
  }

  int c;

  // ":." - path relative to the current directory
  // ":~" - path relative to the home directory
  // ":8" - shortname path - postponed till after
  while (src[*usedlen] == ':'
         && ((c = (uint8_t)src[*usedlen + 1]) == '.' || c == '~' || c == '8')) {
    *usedlen += 2;
    if (c == '8') {
      continue;
    }
    pbuf = NULL;
    // Need full path first (use expand_env() to remove a "~/")
    if (!has_fullname && !has_homerelative) {
      if (**fnamep == '~') {
        p = pbuf = expand_env_save(*fnamep);
      } else {
        p = pbuf = FullName_save(*fnamep, false);
      }
    } else {
      p = *fnamep;
    }

    has_fullname = false;

    if (p != NULL) {
      if (c == '.') {
        os_dirname(dirname, MAXPATHL);
        if (has_homerelative) {
          s = xstrdup(dirname);
          home_replace(NULL, s, dirname, MAXPATHL, true);
          xfree(s);
        }
        size_t namelen = strlen(dirname);

        // Do not call shorten_fname() here since it removes the prefix
        // even though the path does not have a prefix.
        if (path_fnamencmp(p, dirname, namelen) == 0) {
          p += namelen;
          if (vim_ispathsep(*p)) {
            while (*p && vim_ispathsep(*p)) {
              p++;
            }
            *fnamep = p;
            if (pbuf != NULL) {
              // free any allocated file name
              xfree(*bufp);
              *bufp = pbuf;
              pbuf = NULL;
            }
          }
        }
      } else {
        home_replace(NULL, p, dirname, MAXPATHL, true);
        // Only replace it when it starts with '~'
        if (*dirname == '~') {
          s = xstrdup(dirname);
          assert(s != NULL);  // suppress clang "Argument with 'nonnull' attribute passed null"
          *fnamep = s;
          xfree(*bufp);
          *bufp = s;
          has_homerelative = true;
        }
      }
      xfree(pbuf);
    }
  }

  char *tail = path_tail(*fnamep);
  *fnamelen = strlen(*fnamep);

  // ":h" - head, remove "/file_name", can be repeated
  // Don't remove the first "/" or "c:\"
  while (src[*usedlen] == ':' && src[*usedlen + 1] == 'h') {
    valid |= VALID_HEAD;
    *usedlen += 2;
    s = get_past_head(*fnamep);
    while (tail > s && after_pathsep(s, tail)) {
      MB_PTR_BACK(*fnamep, tail);
    }
    *fnamelen = (size_t)(tail - *fnamep);
    if (*fnamelen == 0) {
      // Result is empty.  Turn it into "." to make ":cd %:h" work.
      xfree(*bufp);
      *bufp = *fnamep = tail = xstrdup(".");
      *fnamelen = 1;
    } else {
      while (tail > s && !after_pathsep(s, tail)) {
        MB_PTR_BACK(*fnamep, tail);
      }
    }
  }

  // ":8" - shortname
  if (src[*usedlen] == ':' && src[*usedlen + 1] == '8') {
    *usedlen += 2;
  }

  // ":t" - tail, just the basename
  if (src[*usedlen] == ':' && src[*usedlen + 1] == 't') {
    *usedlen += 2;
    *fnamelen -= (size_t)(tail - *fnamep);
    *fnamep = tail;
  }

  // ":e" - extension, can be repeated
  // ":r" - root, without extension, can be repeated
  while (src[*usedlen] == ':'
         && (src[*usedlen + 1] == 'e' || src[*usedlen + 1] == 'r')) {
    // find a '.' in the tail:
    // - for second :e: before the current fname
    // - otherwise: The last '.'
    const bool is_second_e = *fnamep > tail;
    if (src[*usedlen + 1] == 'e' && is_second_e) {
      s = (*fnamep) - 2;
    } else {
      s = (*fnamep) + *fnamelen - 1;
    }

    for (; s > tail; s--) {
      if (s[0] == '.') {
        break;
      }
    }
    if (src[*usedlen + 1] == 'e') {
      if (s > tail || (0 && is_second_e && s == tail)) {
        // we stopped at a '.' (so anchor to &'.' + 1)
        char *newstart = s + 1;
        size_t distance_stepped_back = (size_t)(*fnamep - newstart);
        *fnamelen += distance_stepped_back;
        *fnamep = newstart;
      } else if (*fnamep <= tail) {
        *fnamelen = 0;
      }
    } else {
      // :r - Remove one extension
      //
      // Ensure that `s` doesn't go before `*fnamep`,
      // since then we're taking too many roots:
      //
      // "path/to/this.file.ext" :e:e:r:r
      //          ^    ^-------- *fnamep
      //          +------------- tail
      //
      // Also ensure `s` doesn't go before `tail`,
      // since then we're taking too many roots again:
      //
      // "path/to/this.file.ext" :r:r:r
      //  ^       ^------------- tail
      //  +--------------------- *fnamep
      if (s > MAX(tail, *fnamep)) {
        *fnamelen = (size_t)(s - *fnamep);
      }
    }
    *usedlen += 2;
  }

  // ":s?pat?foo?" - substitute
  // ":gs?pat?foo?" - global substitute
  if (src[*usedlen] == ':'
      && (src[*usedlen + 1] == 's'
          || (src[*usedlen + 1] == 'g' && src[*usedlen + 2] == 's'))) {
    bool didit = false;

    char *flags = "";
    s = src + *usedlen + 2;
    if (src[*usedlen + 1] == 'g') {
      flags = "g";
      s++;
    }

    int sep = (uint8_t)(*s++);
    if (sep) {
      // find end of pattern
      p = vim_strchr(s, sep);
      if (p != NULL) {
        char *const pat = xmemdupz(s, (size_t)(p - s));
        s = p + 1;
        // find end of substitution
        p = vim_strchr(s, sep);
        if (p != NULL) {
          char *const sub = xmemdupz(s, (size_t)(p - s));
          char *const str = xmemdupz(*fnamep, *fnamelen);
          *usedlen = (size_t)(p + 1 - src);
          size_t slen;
          s = do_string_sub(str, *fnamelen, pat, sub, NULL, flags, &slen);
          *fnamep = s;
          *fnamelen = slen;
          xfree(*bufp);
          *bufp = s;
          didit = true;
          xfree(sub);
          xfree(str);
        }
        xfree(pat);
      }
      // after using ":s", repeat all the modifiers
      if (didit) {
        goto repeat;
      }
    }
  }

  if (src[*usedlen] == ':' && src[*usedlen + 1] == 'S') {
    // vim_strsave_shellescape() needs a NUL terminated string.
    c = (uint8_t)(*fnamep)[*fnamelen];
    if (c != NUL) {
      (*fnamep)[*fnamelen] = NUL;
    }
    p = vim_strsave_shellescape(*fnamep, false, false);
    if (c != NUL) {
      (*fnamep)[*fnamelen] = (char)c;
    }
    xfree(*bufp);
    *bufp = *fnamep = p;
    *fnamelen = strlen(p);
    *usedlen += 2;
  }

  return valid;
}
