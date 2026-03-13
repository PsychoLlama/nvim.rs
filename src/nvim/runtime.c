/// @file runtime.c
///
/// Management of runtime files (including packages)

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <uv.h>

#include "klib/kvec.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/debugger.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/garray.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/stdpaths_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/usercmd.h"
#include "nvim/vim_defs.h"
#ifdef USE_CRNL
# include "nvim/highlight.h"
#endif

/// Structure used to store info for each sourced file.
/// It is shared between do_source() and getsourceline().
/// This is required, because it needs to be handed to do_cmdline() and
/// sourcing can be done recursively.
typedef struct {
  FILE *fp;                     ///< opened file for sourcing
  char *nextline;               ///< if not NULL: line that was read ahead
  linenr_T sourcing_lnum;       ///< line number of the source file
  bool finished;                ///< ":finish" used
  bool source_from_buf_or_str;  ///< true if sourcing from a buffer or string
  int buf_lnum;                 ///< line number in the buffer or string
  garray_T buflines;            ///< lines in the buffer or string
#ifdef USE_CRNL
  int fileformat;               ///< EOL_UNKNOWN, EOL_UNIX or EOL_DOS
  bool error;                   ///< true if LF found after CR-LF
#endif
  linenr_T breakpoint;          ///< next line with breakpoint or zero
  char *fname;                  ///< name of sourced file
  int dbg_tick;                 ///< debug_tick when breakpoint was set
  int level;                    ///< top nesting level of sourced file
  vimconv_T conv;               ///< type of conversion
} source_cookie_T;

typedef struct {
  char *path;
  bool after;
  TriState has_lua;
} SearchPathItem;

typedef kvec_t(SearchPathItem) RuntimeSearchPath;
typedef kvec_t(char *) CharVec;

#include "runtime.c.generated.h"

// Rust FFI forward declarations
extern bool rs_source_callback(int num_fnames, char **fnames, bool all, void *cookie);

garray_T exestack = { 0, 0, sizeof(estack_T), 50, NULL };
garray_T script_items = { 0, 0, sizeof(scriptitem_T *), 20, NULL };

/// The names of packages that once were loaded are remembered.
static garray_T ga_loaded = { 0, 0, sizeof(char *), 4, NULL };

/// last used sequence number for sourcing scripts (current_sctx.sc_seq)
static int last_current_SID_seq = 0;

// Rust implementations of execution stack functions (Phase 1)
extern void rs_estack_init(void);
extern estack_T *rs_estack_push(int type, char *name, linenr_T lnum);
extern void rs_estack_push_ufunc(ufunc_T *ufunc, linenr_T lnum);
extern void rs_estack_pop(void);
extern char *rs_estack_sfile(int which);
extern list_T *rs_stacktrace_create(void);
extern void rs_f_getstacktrace(typval_T *argvars, typval_T *rettv, void *fptr);

// Rust implementations of script registry functions (Phase 2)
extern scriptitem_T *rs_new_script_item(char *name, scid_T *sid_out);
extern int rs_find_script_by_name(const char *name);
extern bool rs_script_is_lua(scid_T sid);
extern char *rs_get_scriptname(int sc_sid, uint64_t sc_chan, bool *should_free);
extern linenr_T rs_get_sourced_lnum(void *fgetline, void *cookie);
extern void *rs_get_script_local_funcs(scid_T sid);
extern void rs_f_getscriptinfo(void *argvars, void *rettv, void *fptr);

// Rust implementations of path utilities and runtimepath (Phase 3)
extern char *rs_get_lib_dir(void);
extern char *rs_runtimepath_default(bool clean_arg);
extern bool rs_path_is_after(const char *buf, size_t buflen);

// C helpers called by Rust for functions that access static variables.
// These live here (not in runtime_ffi.c) because ga_loaded is static.

/// Helper for rs_free_autoload_scriptnames: clear ga_loaded.
void nvim_rt_ga_clear_loaded(void)
{
  ga_clear_strings(&ga_loaded);
}

/// Helper for rs_free_scriptnames: full cleanup of script_items.
void nvim_rt_free_scriptnames(void)
{
  profile_reset();

#define FREE_SCRIPTNAME_RS(item) \
  do { \
    scriptitem_T *_si = *(item); \
    xfree(_si->sn_vars); \
    xfree(_si->sn_name); \
    ga_clear(&_si->sn_prl_ga); \
    xfree(_si); \
  } while (0)

  GA_DEEP_CLEAR(&script_items, scriptitem_T *, FREE_SCRIPTNAME_RS);
}

/// Helper for rs_get_sourced_lnum: compare function pointer to getsourceline.
linenr_T nvim_rt_get_sourced_lnum(LineGetter fgetline, void *cookie)
{
  return fgetline == getsourceline
         ? ((source_cookie_T *)cookie)->sourcing_lnum
         : SOURCING_LNUM;
}

/// Helper for rs_get_script_local_funcs: get script-local functions as a list.
list_T *nvim_rt_get_script_local_funcs(scid_T sid)
{
  hashtab_T *const functbl = func_tbl_get();
  list_T *l = tv_list_alloc((ptrdiff_t)functbl->ht_used);

  HASHTAB_ITER(functbl, hi, {
    const ufunc_T *const fp = HI2UF(hi);
    if (fp->uf_script_ctx.sc_sid == sid) {
      const char *const name = fp->uf_name_exp != NULL
                               ? fp->uf_name_exp : fp->uf_name;
      tv_list_append_string(l, name, -1);
    }
  });

  return l;
}

// =============================================================================
// Phase 3: ga_loaded accessors (ga_loaded is static, so these must live here)
// =============================================================================

/// Get ga_loaded length.
int nvim_rt_ga_loaded_len(void)
{
  return ga_loaded.ga_len;
}

/// Get loaded script name at index.
const char *nvim_rt_ga_loaded_get(int idx)
{
  return ((char **)ga_loaded.ga_data)[idx];
}

/// Append a string to ga_loaded (takes ownership).
void nvim_rt_ga_loaded_append(char *name)
{
  GA_APPEND(char *, &ga_loaded, name);
}

/// do_in_runtimepath wrapper that passes rs_source_callback.
int nvim_rt_do_in_runtimepath_source(const char *name, int flags, void *cookie)
{
  return do_in_runtimepath((char *)name, flags, rs_source_callback, cookie);
}

/// Initialize the execution stack.
void estack_init(void)
{
  rs_estack_init();
}

/// Add an item to the execution stack.
/// @return  the new entry
estack_T *estack_push(etype_T type, char *name, linenr_T lnum)
{
  return rs_estack_push((int)type, name, lnum);
}

/// Add a user function to the execution stack.
void estack_push_ufunc(ufunc_T *ufunc, linenr_T lnum)
{
  rs_estack_push_ufunc(ufunc, lnum);
}

/// Take an item off of the execution stack.
void estack_pop(void)
{
  rs_estack_pop();
}

/// Get the current value for <sfile> in allocated memory.
/// @param which  ESTACK_SFILE for <sfile>, ESTACK_STACK for <stack> or
///               ESTACK_SCRIPT for <script>.
char *estack_sfile(estack_arg_T which)
{
  return rs_estack_sfile((int)which);
}

/// Create the stacktrace from exestack.
list_T *stacktrace_create(void)
{
  return rs_stacktrace_create();
}

/// getstacktrace() function
void f_getstacktrace(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_getstacktrace(argvars, rettv, &fptr);
}

static bool runtime_search_path_valid = false;
static int *runtime_search_path_ref = NULL;
static RuntimeSearchPath runtime_search_path;
static RuntimeSearchPath runtime_search_path_thread;
static uv_mutex_t runtime_search_path_mutex;

// Rust implementations of search path management functions
extern const char *rs_did_set_runtimepackpath(optset_T *args);
extern void rs_runtime_search_path_validate(void);
extern void rs_runtime_search_path_get_cached(int *ref);
extern bool rs_runtime_search_path_unref(const int *ref);

// =============================================================================
// Phase 4: Search path global state accessors (globals are static)
// =============================================================================

/// Get runtime_search_path_valid.
bool nvim_rt_sp_get_valid(void)
{
  return runtime_search_path_valid;
}

/// Set runtime_search_path_valid.
void nvim_rt_sp_set_valid(bool valid)
{
  runtime_search_path_valid = valid;
}

/// Get runtime_search_path_ref.
int *nvim_rt_sp_get_ref(void)
{
  return runtime_search_path_ref;
}

/// Set runtime_search_path_ref.
void nvim_rt_sp_set_ref(int *ref_ptr)
{
  runtime_search_path_ref = ref_ptr;
}

/// Initialize runtime_search_path_mutex.
void nvim_rt_sp_mutex_init(void)
{
  uv_mutex_init(&runtime_search_path_mutex);
}

/// Lock runtime_search_path_mutex.
void nvim_rt_sp_mutex_lock(void)
{
  uv_mutex_lock(&runtime_search_path_mutex);
}

/// Unlock runtime_search_path_mutex.
void nvim_rt_sp_mutex_unlock(void)
{
  uv_mutex_unlock(&runtime_search_path_mutex);
}

/// Check if deferred execution is safe.
bool nvim_rt_nlua_is_deferred_safe(void)
{
  return nlua_is_deferred_safe();
}

static void runtime_search_path_free(RuntimeSearchPath path);
static RuntimeSearchPath copy_runtime_search_path(const RuntimeSearchPath src);
static RuntimeSearchPath runtime_search_path_build(void);

/// Free the global runtime_search_path (items + kvec).
void nvim_rt_sp_free_path(void)
{
  runtime_search_path_free(runtime_search_path);
  runtime_search_path = (RuntimeSearchPath)KV_INITIAL_VALUE;
}

/// Build a new search path and set it as the global.
void nvim_rt_sp_build_and_set(void)
{
  runtime_search_path = runtime_search_path_build();
}

/// Copy global search path to thread-safe copy (frees old thread copy first).
void nvim_rt_sp_copy_to_thread(void)
{
  runtime_search_path_free(runtime_search_path_thread);
  runtime_search_path_thread = copy_runtime_search_path(runtime_search_path);
}


/// Find the patterns in "name" in all directories in "path" and invoke
/// "callback(fname, cookie)".
/// "prefix" is prepended to each pattern in "name".
/// When "flags" has DIP_ALL: source all files, otherwise only the first one.
/// When "flags" has DIP_DIR: find directories instead of files.
/// When "flags" has DIP_ERR: give an error message if there is no match.
///
/// Return FAIL when no file could be sourced, OK otherwise.
int do_in_path(const char *path, const char *prefix, char *name, int flags,
               DoInRuntimepathCB callback, void *cookie)
  FUNC_ATTR_NONNULL_ARG(1, 2)
{
  bool did_one = false;

  // Make a copy of 'runtimepath'.  Invoking the callback may change the
  // value.
  char *rtp_copy = xstrdup(path);
  char *buf = xmallocz(MAXPATHL);
  {
    char *tail;
    if (p_verbose > 10 && name != NULL) {
      verbose_enter();
      if (*prefix != NUL) {
        smsg(0, _("Searching for \"%s\" under \"%s\" in \"%s\""), name, prefix, path);
      } else {
        smsg(0, _("Searching for \"%s\" in \"%s\""), name, path);
      }
      verbose_leave();
    }

    bool do_all = (flags & DIP_ALL) != 0;

    // Loop over all entries in 'runtimepath'.
    char *rtp = rtp_copy;
    while (*rtp != NUL && (do_all || !did_one)) {
      // Copy the path from 'runtimepath' to buf[].
      copy_option_part(&rtp, buf, MAXPATHL, ",");
      size_t buflen = strlen(buf);

      // Skip after or non-after directories.
      if (flags & (DIP_NOAFTER | DIP_AFTER)) {
        bool is_after = path_is_after(buf, buflen);

        if ((is_after && (flags & DIP_NOAFTER))
            || (!is_after && (flags & DIP_AFTER))) {
          continue;
        }
      }

      if (name == NULL) {
        (*callback)(1, &buf, do_all, cookie);
        did_one = true;
      } else if (buflen + 2 + strlen(prefix) + strlen(name) < MAXPATHL) {
        add_pathsep(buf);
        strcat(buf, prefix);
        tail = buf + strlen(buf);

        // Loop over all patterns in "name"
        char *np = name;
        while (*np != NUL && (do_all || !did_one)) {
          // Append the pattern from "name" to buf[].
          assert(MAXPATHL >= (tail - buf));
          copy_option_part(&np, tail, (size_t)(MAXPATHL - (tail - buf)), "\t ");

          if (p_verbose > 10) {
            verbose_enter();
            smsg(0, _("Searching for \"%s\""), buf);
            verbose_leave();
          }

          int ew_flags = ((flags & DIP_DIR) ? EW_DIR : EW_FILE)
                         | ((flags & DIP_DIRFILE) ? (EW_DIR|EW_FILE) : 0);

          did_one |= gen_expand_wildcards_and_cb(1, &buf, ew_flags, do_all, callback,
                                                 cookie) == OK;
        }
      }
    }
  }
  xfree(buf);
  xfree(rtp_copy);
  if (!did_one && name != NULL) {
    char *basepath = path == p_rtp ? "runtimepath" : "packpath";

    if (flags & DIP_ERR) {
      semsg(_(e_dirnotf), basepath, name);
    } else if (p_verbose > 1) {
      verbose_enter();
      smsg(0, _("not found in '%s': \"%s\""), basepath, name);
      verbose_leave();
    }
  }

  return did_one ? OK : FAIL;
}

static RuntimeSearchPath runtime_search_path_get_cached(int *ref)
  FUNC_ATTR_NONNULL_ALL
{
  rs_runtime_search_path_get_cached(ref);
  return runtime_search_path;
}

static RuntimeSearchPath copy_runtime_search_path(const RuntimeSearchPath src)
{
  RuntimeSearchPath dst = KV_INITIAL_VALUE;
  for (size_t j = 0; j < kv_size(src); j++) {
    SearchPathItem src_item = kv_A(src, j);
    kv_push(dst, ((SearchPathItem){ xstrdup(src_item.path), src_item.after, src_item.has_lua }));
  }

  return dst;
}

static void runtime_search_path_unref(RuntimeSearchPath path, const int *ref)
  FUNC_ATTR_NONNULL_ALL
{
  if (rs_runtime_search_path_unref(ref)) {
    runtime_search_path_free(path);
  }
}

/// Find the file "name" in all directories in "path" and invoke
/// "callback(fname, cookie)".
/// "name" can contain wildcards.
/// When "flags" has DIP_ALL: source all files, otherwise only the first one.
/// When "flags" has DIP_DIR: find directories instead of files.
/// When "flags" has DIP_ERR: give an error message if there is no match.
///
/// return FAIL when no file could be sourced, OK otherwise.
int do_in_cached_path(char *name, int flags, DoInRuntimepathCB callback, void *cookie)
{
  bool did_one = false;

  char buf[MAXPATHL];

  if (p_verbose > 10 && name != NULL) {
    verbose_enter();
    smsg(0, _("Searching for \"%s\" in runtime path"), name);
    verbose_leave();
  }

  int ref;
  RuntimeSearchPath path = runtime_search_path_get_cached(&ref);

  bool do_all = (flags & DIP_ALL) != 0;

  // Loop over all entries in cached path
  for (size_t j = 0; j < kv_size(path); j++) {
    SearchPathItem item = kv_A(path, j);
    size_t buflen = strlen(item.path);

    // Skip after or non-after directories.
    if (flags & (DIP_NOAFTER | DIP_AFTER)) {
      if ((item.after && (flags & DIP_NOAFTER))
          || (!item.after && (flags & DIP_AFTER))) {
        continue;
      }
    }

    if (name == NULL) {
      (*callback)(1, &item.path, do_all, cookie);
    } else if (buflen + strlen(name) + 2 < MAXPATHL) {
      STRCPY(buf, item.path);
      add_pathsep(buf);
      char *tail = buf + strlen(buf);

      // Loop over all patterns in "name"
      char *np = name;

      while (*np != NUL && (do_all || !did_one)) {
        // Append the pattern from "name" to buf[].
        assert(MAXPATHL >= (tail - buf));
        copy_option_part(&np, tail, (size_t)(MAXPATHL - (tail - buf)), "\t ");

        if (p_verbose > 10) {
          verbose_enter();
          smsg(0, _("Searching for \"%s\""), buf);
          verbose_leave();
        }

        int ew_flags = ((flags & DIP_DIR) ? EW_DIR : EW_FILE)
                       | ((flags & DIP_DIRFILE) ? (EW_DIR|EW_FILE) : 0)
                       | EW_NOBREAK;

        // Expand wildcards, invoke the callback for each match.
        char *(pat[]) = { buf };
        did_one |= gen_expand_wildcards_and_cb(1, pat, ew_flags, do_all, callback, cookie) == OK;
      }
    }
  }

  if (!did_one && name != NULL) {
    if (flags & DIP_ERR) {
      semsg(_(e_dirnotf), "runtime path", name);
    } else if (p_verbose > 1) {
      verbose_enter();
      smsg(0, _("not found in runtime path: \"%s\""), name);
      verbose_leave();
    }
  }

  runtime_search_path_unref(path, &ref);

  return did_one ? OK : FAIL;
}

Array runtime_inspect(Arena *arena)
{
  RuntimeSearchPath path = runtime_search_path;
  Array rv = arena_array(arena, kv_size(path));

  for (size_t i = 0; i < kv_size(path); i++) {
    SearchPathItem *item = &kv_A(path, i);
    Array entry = arena_array(arena, 3);
    ADD_C(entry, CSTR_AS_OBJ(item->path));
    ADD_C(entry, BOOLEAN_OBJ(item->after));
    if (item->has_lua != kNone) {
      ADD_C(entry, BOOLEAN_OBJ(item->has_lua == kTrue));
    }
    ADD_C(rv, ARRAY_OBJ(entry));
  }
  return rv;
}

ArrayOf(String) runtime_get_named(bool lua, Array pat, bool all, Arena *arena)
{
  int ref;
  RuntimeSearchPath path = runtime_search_path_get_cached(&ref);
  static char buf[MAXPATHL];

  ArrayOf(String) rv = runtime_get_named_common(lua, pat, all, path, buf, sizeof buf, arena);

  runtime_search_path_unref(path, &ref);
  return rv;
}

ArrayOf(String) runtime_get_named_thread(bool lua, Array pat, bool all)
{
  // TODO(bfredl): avoid contention between multiple worker threads?
  uv_mutex_lock(&runtime_search_path_mutex);
  static char buf[MAXPATHL];
  ArrayOf(String) rv = runtime_get_named_common(lua, pat, all, runtime_search_path_thread,
                                                buf, sizeof buf, NULL);
  uv_mutex_unlock(&runtime_search_path_mutex);
  return rv;
}

static ArrayOf(String) runtime_get_named_common(bool lua, Array pat, bool all,
                                                RuntimeSearchPath path, char *buf, size_t buf_len,
                                                Arena *arena)
{
  ArrayOf(String) rv = arena_array(arena, kv_size(path) * pat.size);
  for (size_t i = 0; i < kv_size(path); i++) {
    SearchPathItem *item = &kv_A(path, i);
    if (lua) {
      if (item->has_lua == kNone) {
        size_t size = (size_t)snprintf(buf, buf_len, "%s/lua/", item->path);
        item->has_lua = (size < buf_len && os_isdir(buf));
      }
      if (item->has_lua == kFalse) {
        continue;
      }
    }

    for (size_t j = 0; j < pat.size; j++) {
      Object pat_item = pat.items[j];
      if (pat_item.type == kObjectTypeString) {
        size_t size = (size_t)snprintf(buf, buf_len, "%s/%s",
                                       item->path, pat_item.data.string.data);
        if (size < buf_len) {
          if (os_file_is_readable(buf)) {
            ADD_C(rv, CSTR_TO_ARENA_OBJ(arena, buf));
            if (!all) {
              goto done;
            }
          }
        }
      }
    }
  }
done:
  return rv;
}

/// Find "name" in "path".  When found, invoke the callback function for
/// it: callback(fname, "cookie")
/// When "flags" has DIP_ALL repeat for all matches, otherwise only the first
/// one is used.
/// Returns OK when at least one match found, FAIL otherwise.
/// If "name" is NULL calls callback for each entry in "path". Cookie is
/// passed by reference in this case, setting it to NULL indicates that callback
/// has done its job.

static void push_path(RuntimeSearchPath *search_path, Set(String) *rtp_used, char *entry,
                      bool after)
{
  String *key_alloc;
  if (set_put_ref(String, rtp_used, cstr_as_string(entry), &key_alloc)) {
    *key_alloc = cstr_to_string(entry);
    kv_push(*search_path, ((SearchPathItem){ key_alloc->data, after, kNone }));
  }
}

static void expand_rtp_entry(RuntimeSearchPath *search_path, Set(String) *rtp_used, char *entry,
                             bool after)
{
  if (set_has(String, rtp_used, cstr_as_string(entry))) {
    return;
  }

  if (!*entry) {
    push_path(search_path, rtp_used, entry, after);
  }

  int num_files;
  char **files;
  char *(pat[]) = { entry };
  if (gen_expand_wildcards(1, pat, &num_files, &files, EW_DIR | EW_NOBREAK) == OK) {
    for (int i = 0; i < num_files; i++) {
      push_path(search_path, rtp_used, files[i], after);
    }
    FreeWild(num_files, files);
  }
}

static void expand_pack_entry(RuntimeSearchPath *search_path, Set(String) *rtp_used,
                              CharVec *after_path, char *pack_entry, size_t pack_entry_len)
{
  static char buf[MAXPATHL];
  char *(start_pat[]) = { "/pack/*/start/*", "/start/*" };  // NOLINT
  for (int i = 0; i < 2; i++) {
    if (pack_entry_len + strlen(start_pat[i]) + 1 > sizeof buf) {
      continue;
    }
    xstrlcpy(buf, pack_entry, sizeof buf);
    xstrlcpy(buf + pack_entry_len, start_pat[i], sizeof buf - pack_entry_len);
    expand_rtp_entry(search_path, rtp_used, buf, false);
    size_t after_size = strlen(buf) + 7;
    char *after = xmallocz(after_size);
    xstrlcpy(after, buf, after_size);
    xstrlcat(after, "/after", after_size);
    kv_push(*after_path, after);
  }
}

static bool path_is_after(char *buf, size_t buflen)
{
  return rs_path_is_after(buf, buflen);
}

static RuntimeSearchPath runtime_search_path_build(void)
{
  kvec_t(String) pack_entries = KV_INITIAL_VALUE;
  Map(String, int) pack_used = MAP_INIT;
  Set(String) rtp_used = SET_INIT;
  RuntimeSearchPath search_path = KV_INITIAL_VALUE;
  CharVec after_path = KV_INITIAL_VALUE;

  static char buf[MAXPATHL];
  for (char *entry = p_pp; *entry != NUL;) {
    char *cur_entry = entry;
    copy_option_part(&entry, buf, MAXPATHL, ",");

    String the_entry = { .data = cur_entry, .size = strlen(buf) };

    kv_push(pack_entries, the_entry);
    map_put(String, int)(&pack_used, the_entry, 0);
  }

  char *rtp_entry;
  for (rtp_entry = p_rtp; *rtp_entry != NUL;) {
    char *cur_entry = rtp_entry;
    copy_option_part(&rtp_entry, buf, MAXPATHL, ",");
    size_t buflen = strlen(buf);

    if (path_is_after(buf, buflen)) {
      rtp_entry = cur_entry;
      break;
    }

    // fact: &rtp entries can contain wild chars
    expand_rtp_entry(&search_path, &rtp_used, buf, false);

    handle_T *h = map_ref(String, int)(&pack_used, cstr_as_string(buf), NULL);
    if (h) {
      (*h)++;
      expand_pack_entry(&search_path, &rtp_used, &after_path, buf, buflen);
    }
  }

  for (size_t i = 0; i < kv_size(pack_entries); i++) {
    String item = kv_A(pack_entries, i);
    handle_T h = map_get(String, int)(&pack_used, item);
    if (h == 0) {
      expand_pack_entry(&search_path, &rtp_used, &after_path, item.data, item.size);
    }
  }

  // "after" packages
  for (size_t i = 0; i < kv_size(after_path); i++) {
    expand_rtp_entry(&search_path, &rtp_used, kv_A(after_path, i), true);
    xfree(kv_A(after_path, i));
  }

  // "after" dirs in rtp
  for (; *rtp_entry != NUL;) {
    copy_option_part(&rtp_entry, buf, MAXPATHL, ",");
    expand_rtp_entry(&search_path, &rtp_used, buf, path_is_after(buf, strlen(buf)));
  }

  // strings are not owned
  kv_destroy(pack_entries);
  kv_destroy(after_path);
  map_destroy(String, &pack_used);
  set_destroy(String, &rtp_used);

  return search_path;
}

static void runtime_search_path_free(RuntimeSearchPath path)
{
  for (size_t j = 0; j < kv_size(path); j++) {
    SearchPathItem item = kv_A(path, j);
    xfree(item.path);
  }
  kv_destroy(path);
}

void runtime_search_path_validate(void)
{
  rs_runtime_search_path_validate();
}


// Phase 3: strcpy_comma_escaped, compute_double_env_sep_len, add_env_sep_dirs,
// and add_dir have been migrated to Rust as internal helpers of
// rs_runtimepath_default. They are no longer needed in C.

char *get_lib_dir(void)
{
  return rs_get_lib_dir();
}

char *runtimepath_default(bool clean_arg)
{
  return rs_runtimepath_default(clean_arg);
}

static void cmd_source(char *fname, exarg_T *eap)
{
  if (*fname != NUL && eap != NULL && eap->addr_count > 0) {
    // if a filename is specified to :source, then a range is not allowed
    emsg(_(e_norange));
    return;
  }

  if (eap != NULL && *fname == NUL) {
    if (eap->forceit) {
      // a file name is needed to source normal mode commands
      emsg(_(e_argreq));
    } else {
      // source ex commands from the current buffer
      cmd_source_buffer(eap, false);
    }
  } else if (eap != NULL && eap->forceit) {
    // ":source!": read Normal mode commands
    // Need to execute the commands directly.  This is required at least
    // for:
    // - ":g" command busy
    // - after ":argdo", ":windo" or ":bufdo"
    // - another command follows
    // - inside a loop
    openscript(fname, global_busy || listcmd_busy || eap->nextcmd != NULL
               || eap->cstack->cs_idx >= 0);

    // ":source" read ex commands
  } else if (do_source(fname, false, DOSO_NONE, NULL) == FAIL) {
    semsg(_(e_notopen), fname);
  }
}

/// ":source [{fname}]"
void ex_source(exarg_T *eap)
{
  cmd_source(eap->arg, eap);
}

/// ":options"
void ex_options(exarg_T *eap)
{
  char buf[500];
  bool multi_mods = 0;

  buf[0] = NUL;
  add_win_cmd_modifiers(buf, &cmdmod, &multi_mods);

  os_setenv("OPTWIN_CMD", buf, 1);
  cmd_source(SYS_OPTWIN_FILE, NULL);
}

/// ":source" and associated commands.
///
/// @return address holding the next breakpoint line for a source cookie
linenr_T *source_breakpoint(void *cookie)
{
  return &((source_cookie_T *)cookie)->breakpoint;
}

/// @return  the address holding the debug tick for a source cookie.
int *source_dbg_tick(void *cookie)
{
  return &((source_cookie_T *)cookie)->dbg_tick;
}

/// @return  the nesting level for a source cookie.
int source_level(void *cookie)
  FUNC_ATTR_PURE
{
  return ((source_cookie_T *)cookie)->level;
}

/// Special function to open a file without handle inheritance.
/// If possible the handle is closed on exec().
static FILE *fopen_noinh_readbin(char *filename)
{
#ifdef MSWIN
  int fd_tmp = os_open(filename, O_RDONLY | O_BINARY | O_NOINHERIT, 0);
#else
  int fd_tmp = os_open(filename, O_RDONLY, 0);
#endif

  if (fd_tmp < 0) {
    return NULL;
  }

  os_set_cloexec(fd_tmp);

  return fdopen(fd_tmp, READBIN);
}

/// Concatenate Vimscript line if it starts with a line continuation into a growarray
/// (excluding the continuation chars and leading whitespace)
///
/// @note Growsize of the growarray may be changed to speed up concatenations!
///
/// @param ga  the growarray to append to
/// @param init_growsize  the starting growsize value of the growarray
/// @param p  pointer to the beginning of the line to consider
/// @param len  the length of this line
///
/// @return true if this line did begin with a continuation (the next line
///         should also be considered, if it exists); false otherwise
static bool concat_continued_line(garray_T *const ga, const int init_growsize, const char *const p,
                                  size_t len)
  FUNC_ATTR_NONNULL_ALL
{
  const char *const line = skipwhite_len(p, len);
  len -= (size_t)(line - p);
  // Skip lines starting with '\" ', concat lines starting with '\'
  if (len >= 3 && strncmp(line, "\"\\ ", 3) == 0) {
    return true;
  } else if (len == 0 || line[0] != '\\') {
    return false;
  }
  if (ga->ga_len > init_growsize) {
    ga_set_growsize(ga, MIN(ga->ga_len, 8000));
  }
  ga_concat_len(ga, line + 1, len - 1);
  return true;
}

/// Create a new script item and allocate script-local vars. @see new_script_vars
///
/// @param  name  File name of the script. NULL for anonymous :source.
/// @param[out]  sid_out  SID of the new item.
///
/// @return  pointer to the created script item.
scriptitem_T *new_script_item(char *const name, scid_T *const sid_out)
  FUNC_ATTR_NONNULL_RET
{
  return rs_new_script_item(name, sid_out);
}

/// Initialization for sourcing lines from the current buffer. Reads all the
/// lines from the buffer and stores it in the cookie grow array.
/// Returns a pointer to the name ":source buffer=<n>" on success and NULL on failure.
static char *do_source_buffer_init(source_cookie_T *sp, const exarg_T *eap, bool ex_lua)
  FUNC_ATTR_NONNULL_ALL
{
  if (curbuf == NULL) {
    return NULL;
  }

  char *fname;
  if (curbuf->b_ffname != NULL) {
    fname = xstrdup(curbuf->b_ffname);
  } else {
    if (ex_lua) {
      // Use ":{range}lua buffer=<num>" as the script name
      snprintf(IObuff, IOSIZE, ":{range}lua buffer=%d", curbuf->b_fnum);
    } else {
      // Use ":source buffer=<num>" as the script name
      snprintf(IObuff, IOSIZE, ":source buffer=%d", curbuf->b_fnum);
    }
    fname = xstrdup(IObuff);
  }

  ga_init(&sp->buflines, sizeof(char *), 100);
  // Copy the lines from the buffer into a grow array
  for (linenr_T curr_lnum = eap->line1; curr_lnum <= eap->line2; curr_lnum++) {
    GA_APPEND(char *, &sp->buflines, xstrdup(ml_get(curr_lnum)));
  }
  sp->buf_lnum = 0;
  sp->source_from_buf_or_str = true;
  // When sourcing a range of lines from a buffer, use buffer line number.
  sp->sourcing_lnum = eap->line1 - 1;

  return fname;
}

/// Initialization for sourcing lines from a string. Reads all the
/// lines from the string and stores it in the cookie grow array.
static void do_source_str_init(source_cookie_T *sp, const char *str)
  FUNC_ATTR_NONNULL_ALL
{
  ga_init(&sp->buflines, sizeof(char *), 100);
  // Copy the lines from the string into a grow array
  while (*str != NUL) {
    const char *eol = skip_to_newline(str);
    GA_APPEND(char *, &sp->buflines, xmemdupz(str, (size_t)(eol - str)));
    str = eol + (*eol != NUL);
  }
  sp->buf_lnum = 0;
  sp->source_from_buf_or_str = true;
}

void cmd_source_buffer(const exarg_T *const eap, bool ex_lua)
  FUNC_ATTR_NONNULL_ALL
{
  do_source_ext(NULL, false, DOSO_NONE, NULL, eap, ex_lua, NULL);
}

/// Executes lines in `str` as Ex commands.
///
/// @see do_source_ext()
int do_source_str(const char *str, char *traceback_name)
  FUNC_ATTR_NONNULL_ALL
{
  char *const sourcing_name = SOURCING_NAME;
  const linenr_T sourcing_lnum = SOURCING_LNUM;
  char sname_buf[256];
  if (sourcing_name != NULL) {
    snprintf(sname_buf, sizeof(sname_buf), "%s called at %s:%" PRIdLINENR,
             traceback_name, sourcing_name, sourcing_lnum);
    traceback_name = sname_buf;
  }
  return do_source_ext(traceback_name, false, DOSO_NONE, NULL, NULL, false, str);
}

/// When fname is a .lua file nlua_exec_file() is invoked to source it.
/// Otherwise reads the file `fname` and executes its lines as Ex commands.
///
/// This function may be called recursively!
///
/// @see do_source_str
///
/// @param fname        if NULL, source from the current buffer
/// @param check_other  check for .vimrc and _vimrc
/// @param is_vimrc     DOSO_ value
/// @param ret_sid      if not NULL and we loaded the script before, don't load it again
/// @param eap          used when sourcing lines from a buffer instead of a file
/// @param str          if not NULL, source from the given string
///
/// @return  FAIL if file could not be opened, OK otherwise
///
/// If a scriptitem_T was found or created "*ret_sid" is set to the SID.
static int do_source_ext(char *const fname, const bool check_other, const int is_vimrc,
                         int *const ret_sid, const exarg_T *const eap, const bool ex_lua,
                         const char *const str)
{
  source_cookie_T cookie;
  uint8_t *firstline = NULL;
  int retval = FAIL;
  int save_debug_break_level = debug_break_level;
  scriptitem_T *si = NULL;
  proftime_T wait_start;
  bool trigger_source_post = false;

  CLEAR_FIELD(cookie);
  char *fname_exp = NULL;
  if (fname == NULL) {
    assert(str == NULL);
    // sourcing lines from a buffer
    fname_exp = do_source_buffer_init(&cookie, eap, ex_lua);
    if (fname_exp == NULL) {
      return FAIL;
    }
  } else if (str != NULL) {
    do_source_str_init(&cookie, str);
    fname_exp = xstrdup(fname);
  } else {
    char *p = expand_env_save(fname);
    if (p == NULL) {
      return retval;
    }
    fname_exp = fix_fname(p);
    xfree(p);
    if (fname_exp == NULL) {
      return retval;
    }
    if (os_isdir(fname_exp)) {
      smsg(0, _("Cannot source a directory: \"%s\""), fname);
      goto theend;
    }
  }

  // See if we loaded this script before.
  int sid = str != NULL ? SID_STR : find_script_by_name(fname_exp);
  if (sid > 0 && ret_sid != NULL) {
    // Already loaded and no need to load again, return here.
    *ret_sid = sid;
    retval = OK;
    goto theend;
  }

  if (str == NULL) {
    // Apply SourceCmd autocommands, they should get the file and source it.
    if (has_autocmd(EVENT_SOURCECMD, fname_exp, NULL)
        && apply_autocmds(EVENT_SOURCECMD, fname_exp, fname_exp,
                          false, curbuf)) {
      retval = aborting() ? FAIL : OK;
      if (retval == OK) {
        // Apply SourcePost autocommands.
        apply_autocmds(EVENT_SOURCEPOST, fname_exp, fname_exp, false, curbuf);
      }
      goto theend;
    }

    // Apply SourcePre autocommands, they may get the file.
    apply_autocmds(EVENT_SOURCEPRE, fname_exp, fname_exp, false, curbuf);
  }

  if (!cookie.source_from_buf_or_str) {
    cookie.fp = fopen_noinh_readbin(fname_exp);
  }
  if (cookie.fp == NULL && check_other) {
    // Try again, replacing file name ".nvimrc" by "_nvimrc" or vice versa,
    // and ".exrc" by "_exrc" or vice versa.
    char *p = path_tail(fname_exp);
    if ((*p == '.' || *p == '_')
        && (STRICMP(p + 1, "nvimrc") == 0 || STRICMP(p + 1, "exrc") == 0)) {
      *p = (*p == '_') ? '.' : '_';
      cookie.fp = fopen_noinh_readbin(fname_exp);
    }
  }

  if (cookie.fp == NULL && !cookie.source_from_buf_or_str) {
    if (p_verbose > 1) {
      verbose_enter();
      if (SOURCING_NAME == NULL) {
        smsg(0, _("could not source \"%s\""), fname);
      } else {
        smsg(0, _("line %" PRId64 ": could not source \"%s\""),
             (int64_t)SOURCING_LNUM, fname);
      }
      verbose_leave();
    }
    goto theend;
  }

  // The file exists.
  // - In verbose mode, give a message.
  // - For a vimrc file, may want to call vimrc_found().
  if (p_verbose > 1) {
    verbose_enter();
    if (SOURCING_NAME == NULL) {
      smsg(0, _("sourcing \"%s\""), fname);
    } else {
      smsg(0, _("line %" PRId64 ": sourcing \"%s\""), (int64_t)SOURCING_LNUM, fname);
    }
    verbose_leave();
  }
  if (is_vimrc == DOSO_VIMRC) {
    vimrc_found(fname_exp, "MYVIMRC");
  }

#ifdef USE_CRNL
  // If no automatic file format: Set default to CR-NL.
  if (*p_ffs == NUL) {
    cookie.fileformat = EOL_DOS;
  } else {
    cookie.fileformat = EOL_UNKNOWN;
  }
#endif

  // Check if this script has a breakpoint.
  cookie.breakpoint = dbg_find_breakpoint(true, fname_exp, 0);
  cookie.fname = fname_exp;
  cookie.dbg_tick = debug_tick;

  cookie.level = ex_nesting_level;

  // start measuring script load time if --startuptime was passed and
  // time_fd was successfully opened afterwards.
  proftime_T rel_time;
  proftime_T start_time;
  FILE * const l_time_fd = time_fd;
  if (l_time_fd != NULL) {
    time_push(&rel_time, &start_time);
  }

  const int l_do_profiling = do_profiling;
  if (l_do_profiling == PROF_YES) {
    prof_child_enter(&wait_start);    // entering a child now
  }

  // Don't use local function variables, if called from a function.
  // Also starts profiling timer for nested script.
  funccal_entry_T funccalp_entry;
  save_funccal(&funccalp_entry);

  const sctx_T save_current_sctx = current_sctx;

  // Always use a new sequence number.
  current_sctx.sc_seq = ++last_current_SID_seq;

  if (sid > 0) {
    // loading the same script again
    si = SCRIPT_ITEM(sid);
  } else if (str == NULL) {
    // It's new, generate a new SID.
    si = new_script_item(fname_exp, &sid);
    si->sn_lua = path_with_extension(fname_exp, "lua");
    fname_exp = xstrdup(si->sn_name);  // used for autocmd
    if (ret_sid != NULL) {
      *ret_sid = sid;
    }
  }
  // Sourcing a string doesn't allocate a script item immediately.
  assert((si != NULL) == (str == NULL));

  // Don't change sc_sid to SID_STR when sourcing a string from a Lua script,
  // as keeping the current sc_sid allows more useful :verbose messages.
  if (str == NULL || !script_is_lua(current_sctx.sc_sid)) {
    current_sctx.sc_sid = sid;
    current_sctx.sc_lnum = 0;
  }

  // Keep the sourcing name/lnum, for recursive calls.
  estack_push(ETYPE_SCRIPT, si != NULL ? si->sn_name : fname_exp, 0);

  if (l_do_profiling == PROF_YES && si != NULL) {
    bool forceit = false;

    // Check if we do profiling for this script.
    if (!si->sn_prof_on && has_profiling(true, si->sn_name, &forceit)) {
      profile_init(si);
      si->sn_pr_force = forceit;
    }
    if (si->sn_prof_on) {
      si->sn_pr_count++;
      si->sn_pr_start = profile_start();
      si->sn_pr_children = profile_zero();
    }
  }

  cookie.conv.vc_type = CONV_NONE;              // no conversion

  if (fname == NULL
      && (ex_lua || strequal(curbuf->b_p_ft, "lua")
          || (curbuf->b_fname && path_with_extension(curbuf->b_fname, "lua")))) {
    // Source lines from the current buffer as lua
    nlua_exec_ga(&cookie.buflines, fname_exp);
  } else if (si != NULL && si->sn_lua) {
    // Source the file as lua
    nlua_exec_file(fname_exp);
  } else {
    // Read the first line so we can check for a UTF-8 BOM.
    firstline = (uint8_t *)getsourceline(0, (void *)&cookie, 0, true);
    if (firstline != NULL && strlen((char *)firstline) >= 3 && firstline[0] == 0xef
        && firstline[1] == 0xbb && firstline[2] == 0xbf) {
      // Found BOM; setup conversion, skip over BOM and recode the line.
      convert_setup(&cookie.conv, "utf-8", p_enc);
      char *p = string_convert(&cookie.conv, (char *)firstline + 3, NULL);
      if (p == NULL) {
        p = xstrdup((char *)firstline + 3);
      }
      xfree(firstline);
      firstline = (uint8_t *)p;
    }
    // Call do_cmdline, which will call getsourceline() to get the lines.
    do_cmdline((char *)firstline, getsourceline, (void *)&cookie,
               DOCMD_VERBOSE|DOCMD_NOWAIT|DOCMD_REPEAT);
  }
  retval = OK;

  if (l_do_profiling == PROF_YES && si != NULL) {
    // Get "si" again, "script_items" may have been reallocated.
    si = SCRIPT_ITEM(current_sctx.sc_sid);
    if (si->sn_prof_on) {
      si->sn_pr_start = profile_end(si->sn_pr_start);
      si->sn_pr_start = profile_sub_wait(wait_start, si->sn_pr_start);
      si->sn_pr_total = profile_add(si->sn_pr_total, si->sn_pr_start);
      si->sn_pr_self = profile_self(si->sn_pr_self, si->sn_pr_start,
                                    si->sn_pr_children);
    }
  }

  if (got_int) {
    emsg(_(e_interr));
  }
  estack_pop();
  if (p_verbose > 1) {
    verbose_enter();
    smsg(0, _("finished sourcing %s"), fname);
    if (SOURCING_NAME != NULL) {
      smsg(0, _("continuing in %s"), SOURCING_NAME);
    }
    verbose_leave();
  }

  if (l_time_fd != NULL) {
    vim_snprintf(IObuff, IOSIZE, "sourcing %s", fname);
    time_msg(IObuff, &start_time);
    time_pop(rel_time);
  }

  if (!got_int) {
    trigger_source_post = true;
  }

  // After a "finish" in debug mode, need to break at first command of next
  // sourced file.
  if (save_debug_break_level > ex_nesting_level
      && debug_break_level == ex_nesting_level) {
    debug_break_level++;
  }

  current_sctx = save_current_sctx;
  restore_funccal();
  if (l_do_profiling == PROF_YES) {
    prof_child_exit(&wait_start);    // leaving a child now
  }
  if (cookie.fp != NULL) {
    fclose(cookie.fp);
  }
  if (cookie.source_from_buf_or_str) {
    ga_clear_strings(&cookie.buflines);
  }
  xfree(cookie.nextline);
  xfree(firstline);
  convert_setup(&cookie.conv, NULL, NULL);

  if (str == NULL && trigger_source_post) {
    apply_autocmds(EVENT_SOURCEPOST, fname_exp, fname_exp, false, curbuf);
  }

theend:
  xfree(fname_exp);
  return retval;
}

/// @param check_other  check for .vimrc and _vimrc
/// @param is_vimrc     DOSO_ value
int do_source(char *fname, bool check_other, int is_vimrc, int *ret_sid)
{
  return do_source_ext(fname, check_other, is_vimrc, ret_sid, NULL, false, NULL);
}

/// Checks if the script with the given script ID is a Lua script.
bool script_is_lua(scid_T sid)
{
  return rs_script_is_lua(sid);
}

/// Find an already loaded script "name".
/// If found returns its script ID.  If not found returns -1.
int find_script_by_name(char *name)
{
  return rs_find_script_by_name(name);
}

linenr_T get_sourced_lnum(LineGetter fgetline, void *cookie)
  FUNC_ATTR_PURE
{
  return fgetline == getsourceline
         ? ((source_cookie_T *)cookie)->sourcing_lnum
         : SOURCING_LNUM;
}

/// Get a pointer to a script name.  Used for ":verbose set".
/// Message appended to "Last set from "
///
/// @param should_free  if non-NULL and the script name is a file path, call
///                     home_replace_save() on it and set *should_free to true.
char *get_scriptname(sctx_T script_ctx, bool *should_free)
{
  return rs_get_scriptname(script_ctx.sc_sid, script_ctx.sc_chan, should_free);
}

/// "getscriptinfo()" function
void f_getscriptinfo(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_getscriptinfo(argvars, rettv, &fptr);
}

/// Get one full line from a sourced file.
/// Called by do_cmdline() when it's called from do_source().
///
/// @return pointer to the line in allocated memory, or NULL for end-of-file or
///         some error.
char *getsourceline(int c, void *cookie, int indent, bool do_concat)
{
  source_cookie_T *sp = (source_cookie_T *)cookie;
  char *line;

  // If breakpoints have been added/deleted need to check for it.
  if ((sp->dbg_tick < debug_tick) && !sp->source_from_buf_or_str) {
    sp->breakpoint = dbg_find_breakpoint(true, sp->fname, SOURCING_LNUM);
    sp->dbg_tick = debug_tick;
  }
  if (do_profiling == PROF_YES) {
    script_line_end();
  }
  // Set the current sourcing line number.
  SOURCING_LNUM = sp->sourcing_lnum + 1;
  // Get current line.  If there is a read-ahead line, use it, otherwise get
  // one now.  "fp" is NULL if actually using a string.
  if (sp->finished || (!sp->source_from_buf_or_str && sp->fp == NULL)) {
    line = NULL;
  } else if (sp->nextline == NULL) {
    line = get_one_sourceline(sp);
  } else {
    line = sp->nextline;
    sp->nextline = NULL;
    sp->sourcing_lnum++;
  }
  if (line != NULL && do_profiling == PROF_YES) {
    script_line_start();
  }

  // Only concatenate lines starting with a \ when 'cpoptions' doesn't
  // contain the 'C' flag.
  if (line != NULL && do_concat && (vim_strchr(p_cpo, CPO_CONCAT) == NULL)) {
    char *p;
    // compensate for the one line read-ahead
    sp->sourcing_lnum--;

    // Get the next line and concatenate it when it starts with a
    // backslash. We always need to read the next line, keep it in
    // sp->nextline.
    // Also check for a comment in between continuation lines: "\ .
    sp->nextline = get_one_sourceline(sp);
    if (sp->nextline != NULL
        && (*(p = skipwhite(sp->nextline)) == '\\'
            || (p[0] == '"' && p[1] == '\\' && p[2] == ' '))) {
      garray_T ga;

      ga_init(&ga, (int)sizeof(char), 400);
      ga_concat(&ga, line);
      while (sp->nextline != NULL
             && concat_continued_line(&ga, 400, sp->nextline, strlen(sp->nextline))) {
        xfree(sp->nextline);
        sp->nextline = get_one_sourceline(sp);
      }
      ga_append(&ga, NUL);
      xfree(line);
      line = ga.ga_data;
    }
  }

  if (line != NULL && sp->conv.vc_type != CONV_NONE) {
    // Convert the encoding of the script line.
    char *s = string_convert(&sp->conv, line, NULL);
    if (s != NULL) {
      xfree(line);
      line = s;
    }
  }

  // Did we encounter a breakpoint?
  if (!sp->source_from_buf_or_str
      && sp->breakpoint != 0 && sp->breakpoint <= SOURCING_LNUM) {
    dbg_breakpoint(sp->fname, SOURCING_LNUM);
    // Find next breakpoint.
    sp->breakpoint = dbg_find_breakpoint(true, sp->fname, SOURCING_LNUM);
    sp->dbg_tick = debug_tick;
  }

  return line;
}

static char *get_one_sourceline(source_cookie_T *sp)
{
  garray_T ga;
  int len;
  int c;
  char *buf;
#ifdef USE_CRNL
  bool has_cr;                           // CR-LF found
#endif
  bool have_read = false;

  // use a growarray to store the sourced line
  ga_init(&ga, 1, 250);

  // Loop until there is a finished line (or end-of-file).
  sp->sourcing_lnum++;
  while (true) {
    // make room to read at least 120 (more) characters
    ga_grow(&ga, 120);
    if (sp->source_from_buf_or_str) {
      if (sp->buf_lnum >= sp->buflines.ga_len) {
        break;              // all the lines are processed
      }
      ga_concat(&ga, ((char **)sp->buflines.ga_data)[sp->buf_lnum]);
      sp->buf_lnum++;
      ga_grow(&ga, 1);
      buf = (char *)ga.ga_data;
      buf[ga.ga_len++] = NUL;
      len = ga.ga_len;
    } else {
      buf = ga.ga_data;
retry:
      errno = 0;
      if (fgets(buf + ga.ga_len, ga.ga_maxlen - ga.ga_len, sp->fp) == NULL) {
        if (errno == EINTR) {
          goto retry;
        }
        break;
      }
      len = ga.ga_len + (int)strlen(buf + ga.ga_len);
    }
#ifdef USE_CRNL
    // Ignore a trailing CTRL-Z, when in Dos mode. Only recognize the
    // CTRL-Z by its own, or after a NL.
    if ((len == 1 || (len >= 2 && buf[len - 2] == '\n'))
        && sp->fileformat == EOL_DOS
        && buf[len - 1] == Ctrl_Z) {
      buf[len - 1] = NUL;
      break;
    }
#endif

    have_read = true;
    ga.ga_len = len;

    // If the line was longer than the buffer, read more.
    if (ga.ga_maxlen - ga.ga_len == 1 && buf[len - 1] != '\n') {
      continue;
    }

    if (len >= 1 && buf[len - 1] == '\n') {     // remove trailing NL
#ifdef USE_CRNL
      has_cr = (len >= 2 && buf[len - 2] == '\r');
      if (sp->fileformat == EOL_UNKNOWN) {
        if (has_cr) {
          sp->fileformat = EOL_DOS;
        } else {
          sp->fileformat = EOL_UNIX;
        }
      }

      if (sp->fileformat == EOL_DOS) {
        if (has_cr) {               // replace trailing CR
          buf[len - 2] = '\n';
          len--;
          ga.ga_len--;
        } else {          // lines like ":map xx yy^M" will have failed
          if (!sp->error) {
            msg_source(HL_ATTR(HLF_W));
            emsg(_("W15: Warning: Wrong line separator, ^M may be missing"));
          }
          sp->error = true;
          sp->fileformat = EOL_UNIX;
        }
      }
#endif
      // The '\n' is escaped if there is an odd number of ^V's just
      // before it, first set "c" just before the 'V's and then check
      // len&c parities (is faster than ((len-c)%2 == 0)) -- Acevedo
      for (c = len - 2; c >= 0 && buf[c] == Ctrl_V; c--) {}
      if ((len & 1) != (c & 1)) {       // escaped NL, read more
        sp->sourcing_lnum++;
        continue;
      }

      buf[len - 1] = NUL;               // remove the NL
    }

    // Check for ^C here now and then, so recursive :so can be broken.
    line_breakcheck();
    break;
  }

  if (have_read) {
    return ga.ga_data;
  }

  xfree(ga.ga_data);
  return NULL;
}

/// Returns true if sourcing a script either from a file or a buffer or a string.
/// Otherwise returns false.
int sourcing_a_script(exarg_T *eap)
{
  return getline_equal(eap->ea_getline, eap->cookie, getsourceline);
}

/// ":scriptencoding": Set encoding conversion for a sourced script.
/// Without the multi-byte feature it's simply ignored.
void ex_scriptencoding(exarg_T *eap)
{
  if (!sourcing_a_script(eap)) {
    emsg(_("E167: :scriptencoding used outside of a sourced file"));
    return;
  }

  char *name = (*eap->arg != NUL) ? enc_canonize(eap->arg) : eap->arg;

  // Setup for conversion from the specified encoding to 'encoding'.
  source_cookie_T *sp = (source_cookie_T *)getline_cookie(eap->ea_getline, eap->cookie);
  convert_setup(&sp->conv, name, p_enc);

  if (name != eap->arg) {
    xfree(name);
  }
}

/// ":finish": Mark a sourced file as finished.
void ex_finish(exarg_T *eap)
{
  if (sourcing_a_script(eap)) {
    do_finish(eap, false);
  } else {
    emsg(_("E168: :finish used outside of a sourced file"));
  }
}

/// Mark a sourced file as finished.  Possibly makes the ":finish" pending.
/// Also called for a pending finish at the ":endtry" or after returning from
/// an extra do_cmdline().  "reanimate" is used in the latter case.
void do_finish(exarg_T *eap, bool reanimate)
{
  if (reanimate) {
    ((source_cookie_T *)getline_cookie(eap->ea_getline, eap->cookie))->finished = false;
  }

  // Cleanup (and deactivate) conditionals, but stop when a try conditional
  // not in its finally clause (which then is to be executed next) is found.
  // In this case, make the ":finish" pending for execution at the ":endtry".
  // Otherwise, finish normally.
  int idx = cleanup_conditionals(eap->cstack, 0, true);
  if (idx >= 0) {
    eap->cstack->cs_pending[idx] = CSTP_FINISH;
    report_make_pending(CSTP_FINISH, NULL);
  } else {
    ((source_cookie_T *)getline_cookie(eap->ea_getline, eap->cookie))->finished = true;
  }
}

/// @return  true when a sourced file had the ":finish" command: Don't give error
///          message for missing ":endif".
///          false when not sourcing a file.
bool source_finished(LineGetter fgetline, void *cookie)
{
  return getline_equal(fgetline, cookie, getsourceline)
         && ((source_cookie_T *)getline_cookie(fgetline, cookie))->finished;
}

