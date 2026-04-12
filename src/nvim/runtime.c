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

extern bool rs_has_autocmd(int event, const char *sfname, int buf_fnum);

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

// Rust implementations (still needed by C)
extern void rs_f_getstacktrace(typval_T *argvars, typval_T *rettv, void *fptr);
extern char *rs_get_scriptname(int sc_sid, uint64_t sc_chan, bool *should_free);
extern void *rs_get_script_local_funcs(scid_T sid);
extern void rs_f_getscriptinfo(void *argvars, void *rettv, void *fptr);

// C helpers called by Rust for functions that access static variables.
// These live here (not in runtime_ffi.c) because ga_loaded is static.

/// Helper for rs_free_autoload_scriptnames: clear ga_loaded.
void nvim_rt_ga_clear_loaded(void) { ga_clear_strings(&ga_loaded); }

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

// Phase 4: source_cookie_T accessors and static variable wrappers
// These must live in runtime.c because source_cookie_T and static vars are here.

// nvim_rt_fopen_noinh_readbin: migrated to Rust (dosource.rs).

// last_current_SID_seq increment (static var, must be here).
int nvim_rt_next_script_seq(void) { return ++last_current_SID_seq; }

// Allocate a source_cookie_T.
void *nvim_rt_cookie_alloc(void) { return xcalloc(1, sizeof(source_cookie_T)); }

// nvim_rt_cookie_free_full: migrated to Rust (dosource.rs).

void *nvim_rt_cookie_get_buflines_ga(void *cookie) { return &((source_cookie_T *)cookie)->buflines; }
void nvim_rt_cookie_set_fp(void *cookie, void *fp) { ((source_cookie_T *)cookie)->fp = (FILE *)fp; }
void *nvim_rt_cookie_get_fp(void *cookie) { return ((source_cookie_T *)cookie)->fp; }
bool nvim_rt_cookie_get_src_from_buf_or_str(void *cookie) { return ((source_cookie_T *)cookie)->source_from_buf_or_str; }
void nvim_rt_cookie_set_src_from_buf_or_str(void *cookie, bool val) { ((source_cookie_T *)cookie)->source_from_buf_or_str = val; }
int nvim_rt_cookie_get_buf_lnum(void *cookie) { return ((source_cookie_T *)cookie)->buf_lnum; }
void nvim_rt_cookie_set_buf_lnum(void *cookie, int val) { ((source_cookie_T *)cookie)->buf_lnum = val; }
void nvim_rt_cookie_set_sourcing_lnum(void *cookie, int val) { ((source_cookie_T *)cookie)->sourcing_lnum = val; }
char *nvim_rt_cookie_get_nextline(void *cookie) { return ((source_cookie_T *)cookie)->nextline; }
void nvim_rt_cookie_set_nextline(void *cookie, char *val) { ((source_cookie_T *)cookie)->nextline = val; }
const char *nvim_rt_cookie_get_fname(void *cookie) { return ((source_cookie_T *)cookie)->fname; }
void nvim_rt_cookie_set_fname(void *cookie, char *val) { ((source_cookie_T *)cookie)->fname = val; }
void nvim_rt_cookie_set_dbg_tick(void *cookie, int val) { ((source_cookie_T *)cookie)->dbg_tick = val; }
int nvim_rt_cookie_get_dbg_tick(void *cookie) { return ((source_cookie_T *)cookie)->dbg_tick; }
void nvim_rt_cookie_set_breakpoint(void *cookie, int val) { ((source_cookie_T *)cookie)->breakpoint = (linenr_T)val; }
void nvim_rt_cookie_set_level(void *cookie, int val) { ((source_cookie_T *)cookie)->level = val; }
void nvim_rt_cookie_set_conv_type_none(void *cookie) { ((source_cookie_T *)cookie)->conv.vc_type = CONV_NONE; }
void nvim_rt_cookie_clear_buflines(void *cookie) { ga_clear_strings(&((source_cookie_T *)cookie)->buflines); }
void nvim_rt_cookie_free_nextline(void *cookie) { xfree(((source_cookie_T *)cookie)->nextline); ((source_cookie_T *)cookie)->nextline = NULL; }
void nvim_rt_cookie_teardown_conv(void *cookie) { convert_setup(&((source_cookie_T *)cookie)->conv, NULL, NULL); }

/// cookie sourcing_lnum getter.
int nvim_rt_cookie_get_sourcing_lnum(void *cookie) { return ((source_cookie_T *)cookie)->sourcing_lnum; }

/// cookie sourcing_lnum incrementor.
void nvim_rt_cookie_inc_sourcing_lnum(void *cookie) { ((source_cookie_T *)cookie)->sourcing_lnum++; }

/// cookie sourcing_lnum decrement (for continuation line handling).
void nvim_rt_cookie_dec_sourcing_lnum(void *cookie) { ((source_cookie_T *)cookie)->sourcing_lnum--; }

/// cookie buflines.ga_len getter.
int nvim_rt_cookie_get_buflines_len(void *cookie) { return ((source_cookie_T *)cookie)->buflines.ga_len; }

/// Get a line from cookie->buflines at index.
const char *nvim_rt_cookie_get_bufline(void *cookie, int idx)
{
  return ((char **)((source_cookie_T *)cookie)->buflines.ga_data)[idx];
}

/// cookie breakpoint value getter (not pointer).
int nvim_rt_cookie_get_breakpoint(void *cookie) { return (int)((source_cookie_T *)cookie)->breakpoint; }

/// cookie buf_lnum increment.
void nvim_rt_cookie_inc_buf_lnum(void *cookie) { ((source_cookie_T *)cookie)->buf_lnum++; }

/// do_cmdline via getsourceline (accesses static getsourceline).
int nvim_rt_do_cmdline_source(char *firstline, void *cookie, int flags)
{
  return do_cmdline(firstline, getsourceline, cookie, flags);
}

/// getsourceline function pointer (for comparison or passing to do_cmdline).
LineGetter nvim_rt_getsourceline_ptr(void) { return getsourceline; }

// Phase 3: ga_loaded accessors (ga_loaded is static, so these must live here)

/// Get ga_loaded length.
int nvim_rt_ga_loaded_len(void) { return ga_loaded.ga_len; }

/// Get loaded script name at index.
const char *nvim_rt_ga_loaded_get(int idx) { return ((char **)ga_loaded.ga_data)[idx]; }

/// Append a string to ga_loaded (takes ownership).
void nvim_rt_ga_loaded_append(char *name) { GA_APPEND(char *, &ga_loaded, name); }

/// do_in_runtimepath wrapper that passes rs_source_callback.
int nvim_rt_do_in_runtimepath_source(const char *name, int flags, void *cookie)
{
  return do_in_runtimepath((char *)name, flags, rs_source_callback, cookie);
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
extern void rs_runtime_search_path_get_cached(int *ref);
extern bool rs_runtime_search_path_unref(const int *ref);

// Phase 1: Rust implementations of runtime_inspect / runtime_get_named*
extern Array rs_runtime_inspect(Arena *arena);
extern Array rs_runtime_get_named(bool lua, Array pat, bool all, Arena *arena);
extern Array rs_runtime_get_named_thread(bool lua, Array pat, bool all);

// Phase 4: Search path global state accessors (globals are static)

/// Get runtime_search_path_valid.
bool nvim_rt_sp_get_valid(void) { return runtime_search_path_valid; }

/// Set runtime_search_path_valid.
void nvim_rt_sp_set_valid(bool valid) { runtime_search_path_valid = valid; }

/// Get runtime_search_path_ref.
int *nvim_rt_sp_get_ref(void) { return runtime_search_path_ref; }

/// Set runtime_search_path_ref.
void nvim_rt_sp_set_ref(int *ref_ptr) { runtime_search_path_ref = ref_ptr; }

/// Initialize runtime_search_path_mutex.
void nvim_rt_sp_mutex_init(void) { uv_mutex_init(&runtime_search_path_mutex); }

/// Lock runtime_search_path_mutex.
void nvim_rt_sp_mutex_lock(void) { uv_mutex_lock(&runtime_search_path_mutex); }

/// Unlock runtime_search_path_mutex.
void nvim_rt_sp_mutex_unlock(void) { uv_mutex_unlock(&runtime_search_path_mutex); }

/// Check if deferred execution is safe.
bool nvim_rt_nlua_is_deferred_safe(void) { return nlua_is_deferred_safe(); }

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
void nvim_rt_sp_build_and_set(void) { runtime_search_path = runtime_search_path_build(); }

/// Copy global search path to thread-safe copy (frees old thread copy first).
void nvim_rt_sp_copy_to_thread(void)
{
  runtime_search_path_free(runtime_search_path_thread);
  runtime_search_path_thread = copy_runtime_search_path(runtime_search_path);
}

// Phase 5: RuntimeSearchPath accessors for do_in_cached_path in Rust.
// These expose the cached RuntimeSearchPath kvec to Rust via an opaque ref.

/// Get the cached runtime search path and return its size.
/// @param ref  output: reference token (pass to nvim_rt_rsp_unref when done)
/// @return     number of entries in the cached path
size_t nvim_rt_rsp_get_cached_size(int *ref)
{
  rs_runtime_search_path_get_cached(ref);
  return kv_size(runtime_search_path);
}

/// Get the path string of item at index in the cached runtime search path.
/// Must be called between nvim_rt_rsp_get_cached_size and nvim_rt_rsp_unref.
const char *nvim_rt_rsp_get_item_path(size_t idx)
{
  return kv_A(runtime_search_path, idx).path;
}

/// Get the 'after' flag of item at index in the cached runtime search path.
bool nvim_rt_rsp_get_item_after(size_t idx)
{
  return kv_A(runtime_search_path, idx).after;
}

/// Unref the cached runtime search path (mirrors runtime_search_path_unref).
void nvim_rt_rsp_unref(const int *ref)
{
  // We pass the global path but we only free it if unref says so.
  if (rs_runtime_search_path_unref(ref)) {
    runtime_search_path_free(runtime_search_path);
    runtime_search_path = (RuntimeSearchPath)KV_INITIAL_VALUE;
  }
}

// Phase 1: Accessors for the global (non-cached) runtime_search_path used by runtime_inspect.

/// Get the size of the global (non-cached) runtime search path.
size_t nvim_rt_rsp_get_path_size(void) { return kv_size(runtime_search_path); }

/// Get path string of item at index in the global runtime_search_path.
const char *nvim_rt_rsp_get_path_item_path(size_t idx)
{
  return kv_A(runtime_search_path, idx).path;
}

/// Get the 'after' flag of item at index in the global runtime_search_path.
bool nvim_rt_rsp_get_path_item_after(size_t idx)
{
  return kv_A(runtime_search_path, idx).after;
}

/// Get has_lua of item at index in the global runtime_search_path as int (-1/0/1).
int nvim_rt_rsp_get_path_item_has_lua(size_t idx)
{
  return (int)kv_A(runtime_search_path, idx).has_lua;
}

/// Get has_lua of item at index in the cached runtime_search_path as int (-1/0/1).
int nvim_rt_rsp_get_item_has_lua(size_t idx)
{
  return (int)kv_A(runtime_search_path, idx).has_lua;
}

/// Set has_lua of item at index in the cached runtime_search_path.
void nvim_rt_rsp_set_item_has_lua(size_t idx, int val)
{
  kv_A(runtime_search_path, idx).has_lua = (TriState)val;
}

// Phase 1: Accessors for the thread-safe copy (runtime_search_path_thread).

/// Get the size of the thread-safe runtime search path.
size_t nvim_rt_rsp_get_thread_size(void) { return kv_size(runtime_search_path_thread); }

/// Get path string of item at index in runtime_search_path_thread.
const char *nvim_rt_rsp_get_thread_item_path(size_t idx)
{
  return kv_A(runtime_search_path_thread, idx).path;
}

/// Get the 'after' flag of item at index in runtime_search_path_thread.
bool nvim_rt_rsp_get_thread_item_after(size_t idx)
{
  return kv_A(runtime_search_path_thread, idx).after;
}

/// Get has_lua of item at index in runtime_search_path_thread as int (-1/0/1).
int nvim_rt_rsp_get_thread_item_has_lua(size_t idx)
{
  return (int)kv_A(runtime_search_path_thread, idx).has_lua;
}

/// Set has_lua of item at index in runtime_search_path_thread.
void nvim_rt_rsp_set_thread_item_has_lua(size_t idx, int val)
{
  kv_A(runtime_search_path_thread, idx).has_lua = (TriState)val;
}

/// Get p_rtp pointer value (for pointer comparison in do_in_path).
const char *nvim_rt_get_p_rtp(void) { return p_rtp; }


static RuntimeSearchPath copy_runtime_search_path(const RuntimeSearchPath src)
{
  RuntimeSearchPath dst = KV_INITIAL_VALUE;
  for (size_t j = 0; j < kv_size(src); j++) {
    SearchPathItem src_item = kv_A(src, j);
    kv_push(dst, ((SearchPathItem){ xstrdup(src_item.path), src_item.after, src_item.has_lua }));
  }

  return dst;
}

Array runtime_inspect(Arena *arena) { return rs_runtime_inspect(arena); }

ArrayOf(String) runtime_get_named(bool lua, Array pat, bool all, Arena *arena)
{
  return rs_runtime_get_named(lua, pat, all, arena);
}

ArrayOf(String) runtime_get_named_thread(bool lua, Array pat, bool all)
{
  return rs_runtime_get_named_thread(lua, pat, all);
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

// source_cookie_T field accessors (for Rust FFI)
linenr_T *nvim_rt_cookie_get_breakpoint_ptr(void *cookie) { return &((source_cookie_T *)cookie)->breakpoint; }
int *nvim_rt_cookie_get_dbg_tick_ptr(void *cookie) { return &((source_cookie_T *)cookie)->dbg_tick; }
int nvim_rt_cookie_get_level(void *cookie) { return ((source_cookie_T *)cookie)->level; }
bool nvim_rt_cookie_get_finished(void *cookie) { return ((source_cookie_T *)cookie)->finished; }
void nvim_rt_cookie_set_finished(void *cookie, bool val) { ((source_cookie_T *)cookie)->finished = val; }
void *nvim_rt_cookie_get_conv(void *cookie) { return &((source_cookie_T *)cookie)->conv; }



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

