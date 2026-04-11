// Various routines dealing with allocation and deallocation of memory.

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "nvim/api/extmark.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/ui.h"
#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/channel.h"
#include "nvim/context.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawline.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight.h"
#include "nvim/highlight_group.h"
#include "nvim/insexpand.h"
#include "nvim/lua/executor.h"
#include "nvim/main.h"
#include "nvim/map_defs.h"
#include "nvim/mapping.h"
#include "nvim/memfile.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/sign.h"
#include "nvim/state_defs.h"
#include "nvim/statusline.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_client.h"
#include "nvim/ui_compositor.h"
#include "nvim/usercmd.h"

#ifdef UNIT_TESTING
# define malloc(size) mem_malloc(size)
# define calloc(count, size) mem_calloc(count, size)
# define realloc(ptr, size) mem_realloc(ptr, size)
# define free(ptr) mem_free(ptr)
MemMalloc mem_malloc = &malloc;
MemFree mem_free = &free;
MemCalloc mem_calloc = &calloc;
MemRealloc mem_realloc = &realloc;
#endif

#include "memory.c.generated.h"

// Rust FFI declarations (tag module)
extern void rs_free_tag_stuff(void);

extern void rs_diff_clear(tabpage_T *tp);

#ifdef EXITFREE
bool entered_free_all_mem = false;
#endif

/// Try to free memory. Used when trying to recover from out of memory errors.
/// @see {xmalloc}
static void try_to_free_memory(void)
{
  static bool trying_to_free = false;
  // avoid recursive calls
  if (trying_to_free) {
    return;
  }
  trying_to_free = true;

  // free any scrollback text
  clear_sb_text(true);
  // Try to save all buffers and release as many blocks as possible
  mf_release_all();

  arena_free_reuse_blks();

  trying_to_free = false;
}

/// Avoid repeating the error message many times (they take 1 second each).
/// `did_outofmem_msg` is reset when a character is read.
static void do_outofmem_msg(size_t size)
{
  if (did_outofmem_msg) {
    return;
  }

  // Don't hide this message
  emsg_silent = 0;

  // Must come first to avoid coming back here when printing the error
  // message fails, e.g. when setting v:errmsg.
  did_outofmem_msg = true;

  semsg(_("E342: Out of memory!  (allocating %" PRIu64 " bytes)"), (uint64_t)size);
}

/// malloc() wrapper
///
/// try_malloc() is a malloc() wrapper that tries to free some memory before
/// trying again.
///
/// @see {try_to_free_memory}
/// @param size
/// @return pointer to allocated space. NULL if out of memory
void *try_malloc(size_t size) FUNC_ATTR_MALLOC FUNC_ATTR_ALLOC_SIZE(1)
{
  size_t allocated_size = size ? size : 1;
  void *ret = malloc(allocated_size);
  if (!ret) {
    try_to_free_memory();
    ret = malloc(allocated_size);
  }
  return ret;
}

/// try_malloc() wrapper that shows an out-of-memory error message to the user
/// before returning NULL
///
/// @see {try_malloc}
/// @param size
/// @return pointer to allocated space. NULL if out of memory
void *verbose_try_malloc(size_t size) FUNC_ATTR_MALLOC FUNC_ATTR_ALLOC_SIZE(1)
{
  void *ret = try_malloc(size);
  if (!ret) {
    do_outofmem_msg(size);
  }
  return ret;
}

/// malloc() wrapper that never returns NULL
///
/// xmalloc() succeeds or gracefully aborts when out of memory.
/// Before aborting try to free some memory and call malloc again.
///
/// @see {try_to_free_memory}
/// @param size
/// @return pointer to allocated space. Never NULL
void *xmalloc(size_t size)
  FUNC_ATTR_MALLOC FUNC_ATTR_ALLOC_SIZE(1) FUNC_ATTR_NONNULL_RET
{
  void *ret = try_malloc(size);
  if (!ret) {
    preserve_exit(e_outofmem);
  }
  return ret;
}

/// free() wrapper that delegates to the backing memory manager
///
/// @note Use XFREE_CLEAR() instead, if possible.
void xfree(void *ptr) { free(ptr); }

/// calloc() wrapper
///
/// @see {xmalloc}
/// @param count
/// @param size
/// @return pointer to allocated space. Never NULL
void *xcalloc(size_t count, size_t size)
  FUNC_ATTR_MALLOC FUNC_ATTR_ALLOC_SIZE_PROD(1, 2) FUNC_ATTR_NONNULL_RET
{
  size_t allocated_count = count && size ? count : 1;
  size_t allocated_size = count && size ? size : 1;
  void *ret = calloc(allocated_count, allocated_size);
  if (!ret) {
    try_to_free_memory();
    ret = calloc(allocated_count, allocated_size);
    if (!ret) {
      preserve_exit(e_outofmem);
    }
  }
  return ret;
}

/// realloc() wrapper
///
/// @see {xmalloc}
/// @param size
/// @return pointer to reallocated space. Never NULL
void *xrealloc(void *ptr, size_t size)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_ALLOC_SIZE(2) FUNC_ATTR_NONNULL_RET
{
  size_t allocated_size = size ? size : 1;
  void *ret = realloc(ptr, allocated_size);
  if (!ret) {
    try_to_free_memory();
    ret = realloc(ptr, allocated_size);
    if (!ret) {
      preserve_exit(e_outofmem);
    }
  }
  return ret;
}

/// xmalloc() wrapper that allocates size + 1 bytes and zeroes the last byte
///
/// Commonly used to allocate strings, e.g. `char *s = xmallocz(len)`.
///
/// @see {xmalloc}
/// @param size
/// @return pointer to allocated space. Never NULL
void *xmallocz(size_t size)
  FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_RET FUNC_ATTR_WARN_UNUSED_RESULT
{
  size_t total_size = size + 1;
  if (total_size < size) {
    preserve_exit(_("Nvim: Data too large to fit into virtual memory space\n"));
  }

  void *ret = xmalloc(total_size);
  ((char *)ret)[size] = NUL;

  return ret;
}

/// Allocates (len + 1) bytes of memory, duplicates `len` bytes of
/// `data` to the allocated memory, zero terminates the allocated memory,
/// and returns a pointer to the allocated memory. If the allocation fails,
/// the program dies.
///
/// @see {xmalloc}
/// @param data Pointer to the data that will be copied
/// @param len number of bytes that will be copied
void *xmemdupz(const void *data, size_t len)
  FUNC_ATTR_MALLOC FUNC_ATTR_NONNULL_RET FUNC_ATTR_WARN_UNUSED_RESULT
  FUNC_ATTR_NONNULL_ALL
{
  return memcpy(xmallocz(len), data, len);
}

/// Copies `len` bytes of `src` to `dst` and zero terminates it.
///
/// @see {xstrlcpy}
/// @param[out]  dst  Buffer to store the result.
/// @param[in]  src  Buffer to be copied.
/// @param[in]  len  Number of bytes to be copied.
void *xmemcpyz(void *dst, const void *src, size_t len)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_NONNULL_RET
{
  memcpy(dst, src, len);
  ((char *)dst)[len] = NUL;
  return dst;
}

/// strdup() wrapper
///
/// @see {xmalloc}
/// @param str 0-terminated string that will be copied
/// @return pointer to a copy of the string
char *xstrdup(const char *str)
  FUNC_ATTR_MALLOC FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_RET
  FUNC_ATTR_NONNULL_ALL
{
  return xmemdupz(str, strlen(str));
}

/// strdup() wrapper
///
/// Unlike xstrdup() allocates a new empty string if it receives NULL.
char *xstrdupnul(const char *const str)
  FUNC_ATTR_MALLOC FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_RET
{
  if (str == NULL) {
    return xmallocz(0);
  }
  return xstrdup(str);
}

/// strndup() wrapper
///
/// @see {xmalloc}
/// @param str 0-terminated string that will be copied
/// @return pointer to a copy of the string
char *xstrndup(const char *str, size_t len)
  FUNC_ATTR_MALLOC FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_RET
  FUNC_ATTR_NONNULL_ALL
{
  char *p = memchr(str, NUL, len);
  return xmemdupz(str, p ? (size_t)(p - str) : len);
}

/// Duplicates a chunk of memory using xmalloc
///
/// @see {xmalloc}
/// @param data pointer to the chunk
/// @param len size of the chunk
/// @return a pointer
void *xmemdup(const void *data, size_t len)
  FUNC_ATTR_MALLOC FUNC_ATTR_ALLOC_SIZE(2) FUNC_ATTR_NONNULL_RET
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  return memcpy(xmalloc(len), data, len);
}

/// Iterative merge sort for doubly linked list.
/// O(NlogN) worst case, and stable.
///  - The list is divided into blocks of increasing size (1, 2, 4, 8, ...).
///  - Each pair of blocks is merged in sorted order.
///  - Merged blocks are reconnected to build the sorted list.
void *mergesort_list(void *head, MergeSortGetFunc get_next, MergeSortSetFunc set_next,
                     MergeSortGetFunc get_prev, MergeSortSetFunc set_prev,
                     MergeSortCompareFunc compare)
{
  if (!head || !get_next(head)) {
    return head;
  }

  // Count length
  int n = 0;
  void *curr = head;
  while (curr) {
    n++;
    curr = get_next(curr);
  }

  for (int size = 1; size < n; size *= 2) {
    void *new_head = NULL;
    void *tail = NULL;
    curr = head;

    while (curr) {
      // Split two runs
      void *left = curr;
      void *right = left;
      for (int i = 0; i < size && right; i++) {
        right = get_next(right);
      }

      void *next = right;
      for (int i = 0; i < size && next; i++) {
        next = get_next(next);
      }

      // Break links
      void *l_end = right ? get_prev(right) : NULL;
      if (l_end) {
        set_next(l_end, NULL);
      }
      if (right) {
        set_prev(right, NULL);
      }

      void *r_end = next ? get_prev(next) : NULL;
      if (r_end) {
        set_next(r_end, NULL);
      }
      if (next) {
        set_prev(next, NULL);
      }

      // Merge
      void *merged = NULL;
      void *merged_tail = NULL;

      while (left || right) {
        void *chosen = NULL;
        if (!left) {
          chosen = right;
          right = get_next(right);
        } else if (!right) {
          chosen = left;
          left = get_next(left);
        } else if (compare(left, right) <= 0) {
          chosen = left;
          left = get_next(left);
        } else {
          chosen = right;
          right = get_next(right);
        }

        if (merged_tail) {
          set_next(merged_tail, chosen);
          set_prev(chosen, merged_tail);
          merged_tail = chosen;
        } else {
          merged = merged_tail = chosen;
          set_prev(chosen, NULL);
        }
      }

      // Connect to full list
      if (!new_head) {
        new_head = merged;
      } else {
        set_next(tail, merged);
        set_prev(merged, tail);
      }

      // Move tail to end
      while (get_next(merged_tail)) {
        merged_tail = get_next(merged_tail);
      }
      tail = merged_tail;

      curr = next;
    }

    head = new_head;
  }

  return head;
}


#if defined(EXITFREE)

# include "nvim/autocmd.h"
# include "nvim/buffer.h"
# include "nvim/cmdhist.h"
# include "nvim/diff.h"
# include "nvim/edit.h"
# include "nvim/ex_cmds.h"
# include "nvim/ex_docmd.h"
# include "nvim/file_search.h"
# include "nvim/getchar.h"
# include "nvim/grid.h"
# include "nvim/mark.h"
# include "nvim/msgpack_rpc/channel.h"
# include "nvim/option.h"
# include "nvim/os/os.h"
# include "nvim/quickfix.h"
# include "nvim/regexp.h"
# include "nvim/register.h"
# include "nvim/search.h"
# include "nvim/spell.h"
# include "nvim/tag.h"
# include "nvim/window.h"

// Free everything that we allocated.
// Can be used to detect memory leaks, e.g., with ccmalloc.
// NOTE: This is tricky!  Things are freed that functions depend on.  Don't be
// surprised if Vim crashes...
// Some things can't be freed, esp. things local to a library function.
void free_all_mem(void)
{
  buf_T *buf, *nextbuf;

  // When we cause a crash here it is caught and Vim tries to exit cleanly.
  // Don't try freeing everything again.
  if (entered_free_all_mem) {
    return;
  }
  entered_free_all_mem = true;
  // Don't want to trigger autocommands from here on.
  block_autocmds();

  // Close all tabs and windows.  Reset 'equalalways' to avoid redraws.
  p_ea = false;
  if (first_tabpage != NULL && first_tabpage->tp_next != NULL) {
    do_cmdline_cmd("tabonly!");
  }

  // Free all spell info.
  spell_free_all();

  // Clear user commands (before deleting buffers).
  ex_comclear(NULL);

  if (curbuf != NULL) {
    // Clear menus.
    do_cmdline_cmd("aunmenu *");
    do_cmdline_cmd("tlunmenu *");
    do_cmdline_cmd("menutranslate clear");

    // Clear mappings, abbreviations, breakpoints.
    // NB: curbuf not used with local=false arg
    map_clear_mode(curbuf, MAP_ALL_MODES, false, false);
    map_clear_mode(curbuf, MAP_ALL_MODES, false, true);
    do_cmdline_cmd("breakdel *");
    do_cmdline_cmd("profdel *");
    do_cmdline_cmd("set keymap=");
  }

  free_titles();
  free_findfile();

  // Obviously named calls.
  free_all_autocmds();
  free_all_marks();
  alist_clear(&global_alist);
  free_homedir();
  free_users();
  free_search_patterns();
  free_old_sub();
  free_last_insert();
  free_insexpand_stuff();
  free_prev_shellcmd();
  free_regexp_stuff();
  rs_free_tag_stuff();
  free_cd_dir();
  free_signs();
  set_expr_line(NULL);
  if (curtab != NULL) {
    rs_diff_clear(curtab);
  }
  clear_sb_text(true);            // free any scrollback text

  // Free some global vars.
  xfree(last_cmdline);
  xfree(new_last_cmdline);
  set_keep_msg(NULL, 0);

  // Clear cmdline history.
  p_hi = 0;
  init_history();

  free_quickfix();

  // Close all script inputs.
  close_all_scripts();

  if (curwin != NULL) {
    // Destroy all windows.  Must come before freeing buffers.
    win_free_all();
  }

  // Free all option values.  Must come after closing windows.
  free_all_options();

  // Free all buffers.  Reset 'autochdir' to avoid accessing things that
  // were freed already.
  // Must be after eval_clear to avoid it trying to access b:changedtick after
  // freeing it.
  p_acd = false;
  for (buf = firstbuf; buf != NULL;) {
    bufref_T bufref;
    set_bufref(&bufref, buf);
    nextbuf = buf->b_next;

    // Since options (in addition to other stuff) have been freed above we need to ensure no
    // callbacks are called, so free them before closing the buffer.
    buf_free_callbacks(buf);

    close_buffer(NULL, buf, DOBUF_WIPE, false, false);
    // Didn't work, try next one.
    buf = bufref_valid(&bufref) ? nextbuf : firstbuf;
  }

  // Clear registers.
  clear_registers();
  ResetRedobuff();
  ResetRedobuff();

  // highlight info
  free_highlight();

  reset_last_sourcing();

  if (first_tabpage != NULL) {
    free_tabpage(first_tabpage);
    first_tabpage = NULL;
  }

  // message history
  msg_hist_clear(0);

  channel_free_all_mem();
  eval_clear();
  api_extmark_free_all_mem();
  ctx_free_all();

  map_destroy(int, &buffer_handles);
  map_destroy(int, &window_handles);
  map_destroy(int, &tabpage_handles);

  // free screenlines (can't display anything now!)
  grid_free_all_mem();
  stl_clear_click_defs(tab_page_click_defs, tab_page_click_defs_size);
  xfree(tab_page_click_defs);

  clear_hl_tables(false);

  check_quickfix_busy();

  decor_free_all_mem();
  drawline_free_all_mem();

  if (ui_client_channel_id) {
    ui_client_free_all_mem();
  }

  remote_ui_free_all_mem();
  ui_free_all_mem();
  ui_comp_free_all_mem();
  nlua_free_all_mem();
  rpc_free_all_mem();

  // should be last, in case earlier free functions deallocates arenas
  arena_free_reuse_blks();
}

#endif
