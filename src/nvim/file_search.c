// File searching functions for 'path', 'tags' and 'cdpath' options.
//
// External visible functions:
//   vim_findfile_init()          creates/initialises the search context
//   vim_findfile_free_visited()  free list of visited files/dirs of search
//                                context
//   vim_findfile()               find a file in the search context
//   vim_findfile_cleanup()       cleanup/free search context created by
//                                vim_findfile_init()
//
// All static functions and variables start with 'ff_'
//
// In general it works like this:
// First you create yourself a search context by calling vim_findfile_init().
// It is possible to give a search context from a previous call to
// vim_findfile_init(), so it can be reused. After this you call vim_findfile()
// until you are satisfied with the result or it returns NULL. On every call it
// returns the next file which matches the conditions given to
// vim_findfile_init(). If it doesn't find a next file it returns NULL.
//
// It is possible to call vim_findfile_init() again to reinitialise your search
// with some new parameters. Don't forget to pass your old search context to
// it, so it can reuse it and especially reuse the list of already visited
// directories. If you want to delete the list of already visited directories
// simply call vim_findfile_free_visited().
//
// When you are done call vim_findfile_cleanup() to free the search context.
//
// The function vim_findfile_init() has a long comment, which describes the
// needed parameters.
//
//
//
// ATTENTION:
// ==========
// We use an allocated search context, these functions are NOT thread-safe!!!!!
//
// To minimize parameter passing (or because I'm too lazy), only the
// external visible functions get a search context as a parameter. This is
// then assigned to a static global, which is used throughout the local
// functions.

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/file_search.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/macros_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/normal.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/strings.h"
#include "nvim/vim_defs.h"

// Accessor functions for Rust
int VIsual_active_get(void) { return VIsual_active ? 1 : 0; }

static String ff_expand_buffer = STRING_INIT;  // used for expanding filenames

// type for the directory search stack
typedef struct ff_stack {
  struct ff_stack *ffs_prev;

  // the fix part (no wildcards) and the part containing the wildcards
  // of the search path
  String ffs_fix_path;
  String ffs_wc_path;

  // files/dirs found in the above directory, matched by the first wildcard
  // of wc_part
  char **ffs_filearray;
  int ffs_filearray_size;
  int ffs_filearray_cur;                  // needed for partly handled dirs

  // to store status of partly handled directories
  // 0: we work on this directory for the first time
  // 1: this directory was partly searched in an earlier step
  int ffs_stage;

  // How deep are we in the directory tree?
  // Counts backward from value of level parameter to vim_findfile_init
  int ffs_level;

  // Did we already expand '**' to an empty string?
  int ffs_star_star_empty;
} ff_stack_T;

// type for already visited directories or files.
typedef struct ff_visited {
  struct ff_visited *ffv_next;

  // Visited directories are different if the wildcard string are
  // different. So we have to save it.
  char *ffv_wc_path;

  // use FileID for comparison (needed because of links), else use filename.
  bool file_id_valid;
  FileID file_id;
  // The memory for this struct is allocated according to the length of
  // ffv_fname.
  char ffv_fname[];
} ff_visited_T;

// We might have to manage several visited lists during a search.
// This is especially needed for the tags option. If tags is set to:
//      "./++/tags,./++/TAGS,++/tags"  (replace + with *)
// So we have to do 3 searches:
//   1) search from the current files directory downward for the file "tags"
//   2) search from the current files directory downward for the file "TAGS"
//   3) search from Vims current directory downwards for the file "tags"
// As you can see, the first and the third search are for the same file, so for
// the third search we can use the visited list of the first search. For the
// second search we must start from an empty visited list.
// The struct ff_visited_list_hdr is used to manage a linked list of already
// visited lists.
typedef struct ff_visited_list_hdr {
  struct ff_visited_list_hdr *ffvl_next;

  // the filename the attached visited list is for
  char *ffvl_filename;

  ff_visited_T *ffvl_visited_list;
} ff_visited_list_hdr_T;

// '**' can be expanded to several directory levels.
// Set the default maximum depth.
#define FF_MAX_STAR_STAR_EXPAND 30

// The search context:
//   ffsc_stack_ptr:    the stack for the dirs to search
//   ffsc_visited_list: the currently active visited list
//   ffsc_dir_visited_list: the currently active visited list for search dirs
//   ffsc_visited_lists_list: the list of all visited lists
//   ffsc_dir_visited_lists_list: the list of all visited lists for search dirs
//   ffsc_file_to_search:     the file to search for
//   ffsc_start_dir:    the starting directory, if search path was relative
//   ffsc_fix_path:     the fix part of the given path (without wildcards)
//                      Needed for upward search.
//   ffsc_wc_path:      the part of the given path containing wildcards
//   ffsc_level:        how many levels of dirs to search downwards
//   ffsc_stopdirs_v:   array of stop directories for upward search
//   ffsc_find_what:    FINDFILE_BOTH, FINDFILE_DIR or FINDFILE_FILE
//   ffsc_tagfile:      searching for tags file, don't use 'suffixesadd'
typedef struct {
  ff_stack_T *ffsc_stack_ptr;
  ff_visited_list_hdr_T *ffsc_visited_list;
  ff_visited_list_hdr_T *ffsc_dir_visited_list;
  ff_visited_list_hdr_T *ffsc_visited_lists_list;
  ff_visited_list_hdr_T *ffsc_dir_visited_lists_list;
  String ffsc_file_to_search;
  String ffsc_start_dir;
  String ffsc_fix_path;
  String ffsc_wc_path;
  int ffsc_level;
  String *ffsc_stopdirs_v;
  int ffsc_find_what;
  int ffsc_tagfile;
} ff_search_ctx_T;



/// Evaluate 'includeexpr' and return the result (caller must free).
char *eval_includeexpr(const char *const ptr, const size_t len)
{
  const sctx_T save_sctx = current_sctx;
  set_vim_var_string(VV_FNAME, ptr, (ptrdiff_t)len);
  current_sctx = curbuf->b_p_script_ctx[kBufOptIncludeexpr];

  char *res = eval_to_string_safe(curbuf->b_p_inex,
                                  was_set_insecurely(curwin, kOptIncludeexpr, OPT_LOCAL),
                                  true);

  set_vim_var_string(VV_FNAME, NULL, 0);
  current_sctx = save_sctx;
  return res;
}


void do_autocmd_dirchanged(char *new_dir, CdScope scope, CdCause cause, bool pre)
{
  static bool recursive = false;

  event_T event = pre ? EVENT_DIRCHANGEDPRE : EVENT_DIRCHANGED;

  if (recursive || !has_event(event)) {
    // No autocommand was defined or we changed
    // the directory from this autocommand.
    return;
  }

  recursive = true;

  save_v_event_T save_v_event;
  dict_T *dict = get_v_event(&save_v_event);
  char buf[8];

  switch (scope) {
  case kCdScopeGlobal:
    snprintf(buf, sizeof(buf), "global");
    break;
  case kCdScopeTabpage:
    snprintf(buf, sizeof(buf), "tabpage");
    break;
  case kCdScopeWindow:
    snprintf(buf, sizeof(buf), "window");
    break;
  case kCdScopeInvalid:
    // Should never happen.
    abort();
  }

#ifdef BACKSLASH_IN_FILENAME
  char new_dir_buf[MAXPATHL];
  STRCPY(new_dir_buf, new_dir);
  slash_adjust(new_dir_buf);
  new_dir = new_dir_buf;
#endif

  if (pre) {
    tv_dict_add_str(dict, S_LEN("directory"), new_dir);
  } else {
    tv_dict_add_str(dict, S_LEN("cwd"), new_dir);
  }
  tv_dict_add_str(dict, S_LEN("scope"), buf);
  tv_dict_add_bool(dict, S_LEN("changed_window"), cause == kCdCauseWindow);
  tv_dict_set_keys_readonly(dict);

  switch (cause) {
  case kCdCauseManual:
  case kCdCauseWindow:
    break;
  case kCdCauseAuto:
    snprintf(buf, sizeof(buf), "auto");
    break;
  case kCdCauseOther:
    // Should never happen.
    abort();
  }

  apply_autocmds(event, buf, new_dir, false, curbuf);

  restore_v_event(dict, &save_v_event);

  recursive = false;
}

