#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "mpack/mpack_core.h"
#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/cmdhist.h"
#include "nvim/eval.h"
#include "nvim/eval/decode.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/fileio.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/msgpack_rpc/packer.h"
#include "nvim/msgpack_rpc/packer_defs.h"
#include "nvim/msgpack_rpc/unpacker.h"
#include "nvim/normal_defs.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/fileio.h"
#include "nvim/os/fileio_defs.h"
#include "nvim/os/fs.h"
#include "nvim/os/fs_defs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/shada.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"

// Rust rs_* function declarations (from src/nvim-rs/shada/src/lib.rs)
extern int rs_magic_isset(void);
extern int rs_marks_equal(pos_T a, pos_T b);
extern int rs_marklist_insert(void *jumps_arr, size_t jump_size, int jl_len, int i);
extern int rs_compare_file_marks(const void *a, const void *b);
extern void rs_shada_free_entry_contents(ShadaEntry *entry);
extern int rs_shada_write_file(const char *file, bool nomerge);
extern void rs_shada_read(void *sd_reader, int flags);
extern var_flavour_T rs_var_flavour(const char *varname);
extern int rs_shada_pack_entry(PackerBuffer *packer, const ShadaEntry *entry, size_t max_kbyte);

/// Generic semsg wrapper: one string argument.
void nvim_shada_semsg_1s(const char *fmt, const char *arg)
{
  semsg(fmt, arg);
}

/// Generic semsg wrapper: two string arguments.
void nvim_shada_semsg_2s(const char *fmt, const char *a, const char *b)
{
  semsg(fmt, a, b);
}

/// Generic semsg wrapper: one uint64 argument (cast to unsigned long long for portability).
void nvim_shada_semsg_u64(const char *fmt, uint64_t val)
{
  semsg(fmt, (unsigned long long)val);
}

/// Generic semsg wrapper: two strings + uint64 + string (for readerr pattern).
void nvim_shada_semsg_2s_u64(const char *fmt, const char *a, uint64_t val, const char *b)
{
  semsg(fmt, a, (unsigned long long)val, b);
}

/// Generic smsg wrapper: one string argument (for verbose writing message).
void nvim_shada_smsg_1s(const char *fmt, const char *arg)
{
  smsg(0, fmt, arg);
}

/// Generic siemsg wrapper: one string argument.
void nvim_shada_siemsg_1s(const char *fmt, const char *arg)
{
  siemsg(fmt, arg);
}

/// Possible ShaDa entry types
///
/// @warning Enum values are part of the API and must not be altered.
///
/// All values that are not in enum are ignored.
typedef enum {
  kSDItemUnknown = -1,       ///< Unknown item.
  kSDItemMissing = 0,        ///< Missing value. Should never appear in a file.
  kSDItemHeader = 1,         ///< Header. Present for debugging purposes.
  kSDItemSearchPattern = 2,  ///< Last search pattern (*not* history item).
                             ///< Comes from user searches (e.g. when typing
                             ///< "/pat") or :substitute command calls.
  kSDItemSubString = 3,      ///< Last substitute replacement string.
  kSDItemHistoryEntry = 4,   ///< History item.
  kSDItemRegister = 5,       ///< Register.
  kSDItemVariable = 6,       ///< Global variable.
  kSDItemGlobalMark = 7,     ///< Global mark definition.
  kSDItemJump = 8,           ///< Item from jump list.
  kSDItemBufferList = 9,     ///< Buffer list.
  kSDItemLocalMark = 10,     ///< Buffer-local mark.
  kSDItemChange = 11,        ///< Item from buffer change list.
} ShadaEntryType;
#define SHADA_LAST_ENTRY ((uint64_t)kSDItemChange)

/// Possible results when reading ShaDa file
typedef enum {
  kSDReadStatusSuccess,    ///< Reading was successful.
  kSDReadStatusFinished,   ///< Nothing more to read.
  kSDReadStatusReadError,  ///< Failed to read from file.
  kSDReadStatusNotShaDa,   ///< Input is most likely not a ShaDa file.
  kSDReadStatusMalformed,  ///< Error in the currently read item.
} ShaDaReadResult;

/// Possible results of shada_write function.
typedef enum {
  kSDWriteSuccessful,    ///< Writing was successful.
  kSDWriteReadNotShada,  ///< Writing was successful, but when reading it
                         ///< attempted to read file that did not look like
                         ///< a ShaDa file.
  kSDWriteFailed,        ///< Writing was not successful (e.g. because there
                         ///< was no space left on device).
  kSDWriteIgnError,      ///< Writing resulted in a error which can be ignored
                         ///< (e.g. when trying to dump a function reference or
                         ///< self-referencing container in a variable).
} ShaDaWriteResult;

/// Flags for shada_read_next_item
enum SRNIFlags {
  kSDReadHeader = (1 << kSDItemHeader),  ///< Determines whether header should
                                         ///< be read (it is usually ignored).
  kSDReadUndisableableData = (
                              (1 << kSDItemSearchPattern)
                              | (1 << kSDItemSubString)
                              | (1 << kSDItemJump)),  ///< Data reading which cannot be disabled by
                                                      ///< &shada or other options except for disabling
                                                      ///< reading ShaDa as a whole.
  kSDReadRegisters = (1 << kSDItemRegister),  ///< Determines whether registers
                                              ///< should be read (may only be
                                              ///< disabled when writing, but
                                              ///< not when reading).
  kSDReadHistory = (1 << kSDItemHistoryEntry),  ///< Determines whether history
                                                ///< should be read (can only be
                                                ///< disabled by &history).
  kSDReadVariables = (1 << kSDItemVariable),  ///< Determines whether variables
                                              ///< should be read (disabled by
                                              ///< removing ! from &shada).
  kSDReadBufferList = (1 << kSDItemBufferList),  ///< Determines whether buffer
                                                 ///< list should be read
                                                 ///< (disabled by removing
                                                 ///< % entry from &shada).
  kSDReadUnknown = (1 << (SHADA_LAST_ENTRY + 1)),  ///< Determines whether
                                                   ///< unknown items should be
                                                   ///< read (usually disabled).
  kSDReadGlobalMarks = (1 << kSDItemGlobalMark),  ///< Determines whether global
                                                  ///< marks should be read. Can
                                                  ///< only be disabled by
                                                  ///< having f0 in &shada when
                                                  ///< writing.
  kSDReadLocalMarks = (1 << kSDItemLocalMark),  ///< Determines whether local
                                                ///< marks should be read. Can
                                                ///< only be disabled by
                                                ///< disabling &shada or putting
                                                ///< '0 there. Is also used for
                                                ///< v:oldfiles.
  kSDReadChanges = (1 << kSDItemChange),  ///< Determines whether change list
                                          ///< should be read. Can only be
                                          ///< disabled by disabling &shada or
                                          ///< putting '0 there.
};
// Note: SRNIFlags enum name was created only to make it possible to reference
// it. This name is not actually used anywhere outside of the documentation.

/// Structure defining a single ShaDa file entry
typedef struct ShadaEntry {
  ShadaEntryType type;
  // If the entry was read from file, string data will be allocated and needs to be freed.
  // Entries can also be constructed from nvim internal data structures (like registers)
  // and reference their allocated strings. then shada code must not attempt to free these.
  bool can_free_entry;
  Timestamp timestamp;
  union {
    Dict header;
    struct shada_filemark {
      char name;
      pos_T mark;
      char *fname;
    } filemark;
    Dict(_shada_search_pat) search_pattern;
    struct history_item {
      uint8_t histtype;
      char *string;
      char sep;
    } history_item;
    struct reg {  // yankreg_T
      char name;
      MotionType type;
      String *contents;
      bool is_unnamed;
      size_t contents_size;
      size_t width;
    } reg;
    struct global_var {
      char *name;
      typval_T value;
    } global_var;
    struct {
      uint64_t type;
      char *contents;
      size_t size;
    } unknown_item;
    struct sub_string {
      char *sub;
    } sub_string;
    struct buffer_list {
      size_t size;
      struct buffer_list_buffer {
        pos_T pos;
        char *fname;
        AdditionalData *additional_data;
      } *buffers;
    } buffer_list;
  } data;
  AdditionalData *additional_data;
} ShadaEntry;

/// One entry in sized linked list
typedef struct hm_llist_entry {
  ShadaEntry data;              ///< Entry data.
  struct hm_llist_entry *next;  ///< Pointer to next entry or NULL.
  struct hm_llist_entry *prev;  ///< Pointer to previous entry or NULL.
} HMLListEntry;

/// Sized linked list structure for history merger
typedef struct {
  HMLListEntry *entries;  ///< Pointer to the start of the allocated array of
                          ///< entries.
  HMLListEntry *first;    ///< First entry in the list (is not necessary start
                          ///< of the array) or NULL.
  HMLListEntry *last;     ///< Last entry in the list or NULL.
  HMLListEntry *free_entry;  ///< Last free entry removed by hmll_remove.
  HMLListEntry *last_free_entry;  ///< Last unused element in entries array.
  size_t size;            ///< Number of allocated entries.
  size_t num_entries;     ///< Number of entries already used.
  PMap(cstr_t) contained_entries;  ///< Map all history entry strings to
                                   ///< corresponding entry pointers.
} HMLList;

typedef struct {
  HMLList hmll;
  bool do_merge;
  bool reading;
  const void *iter;
  ShadaEntry last_hist_entry;
  uint8_t history_type;
} HistoryMergerState;


/// Structure that holds one file marks.
typedef struct {
  ShadaEntry marks[NLOCALMARKS];  ///< All file marks.
  ShadaEntry changes[JUMPLISTSIZE];  ///< All file changes.
  size_t changes_size;  ///< Number of changes occupied.
  ShadaEntry *additional_marks;  ///< All marks with unknown names.
  size_t additional_marks_size;  ///< Size of the additional_marks array.
  Timestamp greatest_timestamp;  ///< Greatest timestamp among marks.
} FileMarks;

/// State structure used by shada_write
///
/// Before actually writing most of the data is read to this structure.
typedef struct {
  HistoryMergerState hms[HIST_COUNT];  ///< Structures for history merging.
  ShadaEntry global_marks[NMARKS];  ///< Named global marks.
  ShadaEntry numbered_marks[EXTRA_MARKS];  ///< Numbered marks.
  ShadaEntry registers[NUM_SAVED_REGISTERS];  ///< All registers.
  ShadaEntry jumps[JUMPLISTSIZE];  ///< All dumped jumps.
  size_t jumps_size;  ///< Number of jumps occupied.
  ShadaEntry search_pattern;  ///< Last search pattern.
  ShadaEntry sub_search_pattern;  ///< Last s/ search pattern.
  ShadaEntry replacement;  ///< Last s// replacement string.
  Set(cstr_t) dumped_variables;  ///< Names of already dumped variables.
  PMap(cstr_t) file_marks;  ///< All file marks.
} WriteMergerState;


#include "shada_shim.c.generated.h"

/// Find buffer for given buffer name (cached).
///
/// @param[in,out]  fname_bufs_handle  Opaque PMap(cstr_t) handle.
/// @param[in]      fname              File name to find.
///
/// @return Pointer to the buffer or NULL.
buf_T *nvim_shada_find_buffer(void *const fname_bufs_handle, const char *const fname)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  PMap(cstr_t) *const fname_bufs = (PMap(cstr_t) *)fname_bufs_handle;
  cstr_t *key_alloc = NULL;
  bool new_item = false;
  buf_T **ref = (buf_T **)pmap_put_ref(cstr_t)(fname_bufs, fname, &key_alloc, &new_item);
  if (new_item) {
    *key_alloc = xstrdup(fname);
  } else {
    return *ref;  // item already existed (can be a NULL value)
  }

  FOR_ALL_BUFFERS(buf) {
    if (buf->b_ffname != NULL) {
      if (path_fnamecmp(fname, buf->b_ffname) == 0) {
        *ref = buf;
        return buf;
      }
    }
  }
  *ref = NULL;
  return NULL;
}


#define SHADA_MPACK_FREE_SPACE (4 * MPACK_ITEM_SIZE)


/// Decode a msgpack binary string to a typval_T at dst.
/// Wrapper for decode_string() that writes the result to an existing buffer.
void nvim_shada_decode_string_into(const char *s, size_t len, bool force_blob, void *dst)
{
  typval_T tv = decode_string(s, len, force_blob, false);
  memcpy(dst, &tv, sizeof(typval_T));
}

// Map operations for Rust hmll implementation (operate on inline PMap(cstr_t))
void nvim_hmll_map_init(PMap(cstr_t) *map)
{
  *map = (PMap(cstr_t)) MAP_INIT;
}

void nvim_hmll_map_destroy(PMap(cstr_t) *map)
{
  if (map) {
    map_destroy(cstr_t, map);
  }
}

void *nvim_hmll_map_get(PMap(cstr_t) *map, const char *key)
{
  if (!map || !key) {
    return NULL;
  }
  return pmap_get(cstr_t)(map, key);
}

void nvim_hmll_map_put(PMap(cstr_t) *map, const char *key, void *entry)
{
  if (!map || !key) {
    return;
  }
  bool new_item = false;
  ptr_t *val = pmap_put_ref(cstr_t)(map, key, NULL, &new_item);
  if (val) {
    *val = entry;
  }
}

void nvim_hmll_map_del(PMap(cstr_t) *map, const char *key)
{
  if (!map || !key) {
    return;
  }
  pmap_del(cstr_t)(map, key, NULL);
}

// Option value accessors for shada
int64_t nvim_get_p_hi(void) { return p_hi; }
const char *nvim_get_p_shadafile(void) { return p_shadafile; }

// Utility wrappers
size_t nvim_expand_env(const char *src, char *dst, size_t dstlen)
{
  if (!src || !dst || dstlen == 0) {
    return 0;
  }
  expand_env((char *)src, dst, (int)dstlen);
  return strlen(dst);
}

char *nvim_xmemdupz(const char *s, size_t len)
{
  return xmemdupz(s, len);
}



const char *nvim_shada_get_p_shada(void) { return p_shada; }
char *nvim_shada_home_replace_save(const void *buf, const char *src)
{
  return home_replace_save((buf_T *)buf, src);
}
void nvim_shada_home_replace(const void *buf, const char *src, char *dst, size_t dstlen, int one)
{
  home_replace((buf_T *)buf, src, dst, dstlen, one != 0);
}
size_t nvim_shada_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep_chars) { return copy_option_part(option, buf, maxlen, (char *)sep_chars); }
int nvim_shada_mb_strnicmp(const char *s1, const char *s2, size_t n) { return mb_strnicmp(s1, s2, n); }
char *nvim_shada_get_namebuff(void) { return NameBuff; }
const void *nvim_shada_buf_first(void) { return firstbuf; }
const void *nvim_shada_buf_next(const void *buf)
{
  return buf ? ((const buf_T *)buf)->b_next : NULL;
}
const char *nvim_shada_buf_get_ffname(const void *buf)
{
  return buf ? ((const buf_T *)buf)->b_ffname : NULL;
}
/// Returns 1 if the buffer should be skipped (not listed, quickfix, or terminal), 0 otherwise.
int nvim_shada_buf_should_skip(const void *buf)
{
  if (!buf) {
    return 1;
  }
  if (!((const buf_T *)buf)->b_p_bl) {
    return 1;
  }
  if (bt_quickfix((const buf_T *)buf)) {
    return 1;
  }
  if (bt_terminal((const buf_T *)buf)) {
    return 1;
  }
  return 0;
}

// Set(ptr_t) operations for Rust FFI
void *nvim_shada_set_init_ptr(void)
{
  Set(ptr_t) *s = xcalloc(1, sizeof(Set(ptr_t)));
  *s = (Set(ptr_t))SET_INIT;
  return s;
}
int nvim_shada_set_has_ptr(const void *set, const void *ptr) { return set ? set_has(ptr_t, (Set(ptr_t) *)set, (ptr_t)ptr) : 0; }
void nvim_shada_set_put_ptr(void *set, const void *ptr)
{
  if (set) {
    set_put(ptr_t, (Set(ptr_t) *)set, (ptr_t)ptr);
  }
}
void nvim_shada_set_destroy_ptr(void *set)
{
  if (set) {
    set_destroy(ptr_t, (Set(ptr_t) *)set);
    xfree(set);
  }
}

// hist_iter wrapper that returns individual fields instead of histentry_T
const void *nvim_shada_hist_iter_raw(const void *iter, uint8_t history_type, int zero,
                                     char **out_str, size_t *out_strlen, Timestamp *out_ts,
                                     void **out_additional_data)
{
  histentry_T hist_he;
  const void *ret = hist_iter(iter, history_type, zero != 0, &hist_he);
  *out_str = hist_he.hisstr;
  *out_strlen = hist_he.hisstrlen;
  *out_ts = hist_he.timestamp;
  *out_additional_data = hist_he.additional_data;
  return ret;
}

// Search/substitute pattern accessor (is_substitute=1 → substitute pattern)
void nvim_shada_get_search_pattern(int is_substitute, char **out_pat, int *out_magic,
                                   int *out_no_scs, Timestamp *out_ts, int *out_off_line,
                                   int *out_off_end, int64_t *out_off_off, char *out_off_dir,
                                   void **out_additional_data)
{
  SearchPattern pat;
  if (is_substitute) {
    get_substitute_pattern(&pat);
  } else {
    get_search_pattern(&pat);
  }
  *out_pat = pat.pat;
  *out_magic = pat.magic;
  *out_no_scs = pat.no_scs;
  *out_ts = pat.timestamp;
  *out_off_line = pat.off.line;
  *out_off_end = pat.off.end;
  *out_off_off = pat.off.off;
  *out_off_dir = pat.off.dir;
  *out_additional_data = pat.additional_data;
}

int nvim_shada_search_was_last_used(void) { return search_was_last_used(); }
int nvim_shada_no_hlsearch(void) { return no_hlsearch; }

// Register iteration accessor
const void *nvim_shada_reg_iter(const void *iter, char *out_name, int *out_type,
                                String **out_contents, size_t *out_size,
                                size_t *out_width, int *out_is_unnamed,
                                Timestamp *out_ts, void **out_additional_data)
{
  yankreg_T reg;
  bool is_unnamed = false;
  const void *ret = op_global_reg_iter(iter, out_name, &reg, &is_unnamed);
  *out_type = reg.y_type;
  *out_contents = reg.y_array;
  *out_size = reg.y_size;
  *out_width = (size_t)(reg.y_type == kMTBlockWise ? reg.y_width : 0);
  *out_is_unnamed = is_unnamed;
  *out_ts = reg.timestamp;
  *out_additional_data = reg.additional_data;
  return ret;
}

int nvim_shada_op_reg_index(char name) { return op_reg_index(name); }

// Buffer list accessors

/// Get last cursor position and additional data for buffer list entry.
void nvim_shada_buf_get_buflist_info(const void *buf, pos_T *out_pos,
                                     void **out_additional_data)
{
  if (buf) {
    *out_pos = ((const buf_T *)buf)->b_last_cursor.mark;
    *out_additional_data = ((const buf_T *)buf)->additional_data;
  }
}
Timestamp nvim_shada_os_time(void) { return os_time(); }

// Jump list accessors
void nvim_shada_setpcmark(void) { setpcmark(); }
void nvim_shada_cleanup_jumplist(void *wp, int loadfiles)
{
  cleanup_jumplist((win_T *)wp, loadfiles != 0);
}
void *nvim_shada_curwin(void) { return curwin; }

// mark_jumplist_iter wrapper
const void *nvim_shada_jumplist_iter(const void *iter, void *wp,
                                     pos_T *out_mark, int *out_fnum,
                                     Timestamp *out_ts, char **out_fname,
                                     void **out_additional_data)
{
  xfmark_T fm;
  const void *ret = mark_jumplist_iter(iter, (win_T *)wp, &fm);
  *out_mark = fm.fmark.mark;
  *out_fnum = fm.fmark.fnum;
  *out_ts = fm.fmark.timestamp;
  *out_fname = fm.fname;
  *out_additional_data = fm.fmark.additional_data;
  return ret;
}

const void *nvim_shada_buflist_findnr(int nr) { return buflist_findnr(nr); }

/// Free a Header ShadaEntry's dict (api_free_dict wrapper for Header entries).
/// Called from Rust rs_shada_free_entry_contents when entry_type == Header.
void nvim_shada_free_header_entry(ShadaEntry *entry)
{
  api_free_dict(entry->data.header);
}

/// Clear a typval_T (tv_clear wrapper)
void nvim_shada_tv_clear(typval_T *tv)
{
  tv_clear(tv);
}


/// Free the value portion of a global_var entry.
/// Takes a pointer to the typval_T within the ShadaEntry union.
void nvim_shada_free_variable(ShadaEntry *entry)
{
  xfree(entry->data.global_var.name);
  tv_clear(&entry->data.global_var.value);
}

/// Open a file for reading. Returns 0 on success, error code on failure.
int nvim_shada_file_open(void *fd, const char *fname) { return file_open((FileDescriptor *)fd, fname, kFileReadOnly, 0); }

/// Initialize a FileDescriptor to read from an in-memory buffer (no fd backing).
void nvim_shada_file_open_buffer(void *fd, char *data, size_t len) { file_open_buffer((FileDescriptor *)fd, data, len); }

/// Read shada data from an open file descriptor.
/// Delegates to Rust rs_shada_read implementation.
void nvim_shada_read(void *fd, int flags)
{
  rs_shada_read(fd, flags);
}

/// Get the os_strerror() message for an error code.
const char *nvim_shada_os_strerror(int err)
{
  return os_strerror(err);
}

/// Wrapper for verbose_enter()
void nvim_shada_verbose_enter(void)
{
  verbose_enter();
}

/// Wrapper for verbose_leave()
void nvim_shada_verbose_leave(void)
{
  verbose_leave();
}

/// Get p_verbose value
int nvim_shada_get_p_verbose(void) { return (int)p_verbose; }

/// Non-variadic smsg wrapper for "Reading ShaDa file" verbose message
void nvim_shada_smsg_reading(const char *fname, int want_info, int want_marks,
                             int get_oldfiles, int failed)
{
  smsg(0, _("Reading ShaDa file \"%s\"%s%s%s%s"),
       fname,
       want_info ? _(" info") : "",
       want_marks ? _(" marks") : "",
       get_oldfiles ? _(" oldfiles") : "",
       failed ? _(" FAILED") : "");
}

/// Get p_fs value (for file sync)
int nvim_shada_get_p_fs(void) { return !!p_fs; }

/// Wrapper for stdpaths_user_state_subpath + concat_fnames_realloc
/// to build the default shada file path.
char *nvim_shada_build_default_path(void)
{
  char *shada_dir = stdpaths_user_state_subpath("shada", 0, false);
  return concat_fnames_realloc(shada_dir, "main.shada", true);
}

/// Get the size of FileDescriptor struct for Rust allocation
size_t nvim_shada_file_descriptor_size(void) { return sizeof(FileDescriptor); }

int nvim_shada_curbuf_marks_read(void) { return curbuf->b_marks_read; }

void nvim_shada_curbuf_set_marks_read(int val)
{
  curbuf->b_marks_read = val;
}

const char *nvim_shada_curbuf_ffname(void)
{
  return curbuf->b_ffname;
}

/// Set a histentry_T element at the given index.
void nvim_shada_set_histentry(void *hist_array, int idx, uint64_t ts,
                              char *hisstr, void *additional_data)
{
  histentry_T *he = &((histentry_T *)hist_array)[idx];
  he->timestamp = ts;
  he->hisnum = idx + 1;
  he->hisstr = hisstr;
  he->hisstrlen = strlen(hisstr);
  he->additional_data = additional_data;
}

// =============================================================================
// Phase 2 (plan 11dd3cf4): shada_write_file migration accessors
// =============================================================================

/// Wrapper for modname() used by rs_shada_write_file.
char *nvim_shada_modname(const char *fname, const char *ext, bool prepend_dot)
{
  return modname(fname, ext, prepend_dot);
}

/// Wrapper for os_getperm() used by rs_shada_write_file.
int nvim_shada_os_getperm(const char *fname)
{
  return (int)os_getperm(fname);
}

/// Wrapper for file_open() with write flags used by rs_shada_write_file.
/// flags: combination of FileOpenFlags bits (int).
int nvim_shada_file_open_write(void *fd, const char *fname, int flags, int perm)
{
  return file_open((FileDescriptor *)fd, fname, (int)flags, perm);
}

/// Return the byte offset from fname to path_tail_with_sep(fname).
/// Used by rs_shada_write_file to find the directory portion.
size_t nvim_shada_path_tail_with_sep_offset(const char *fname)
{
  const char *tail = path_tail_with_sep((char *)fname);
  return (size_t)(tail - fname);
}

/// Wrapper for os_isdir() used by rs_shada_write_file.
int nvim_shada_os_isdir(const char *fname)
{
  return os_isdir(fname) ? 1 : 0;
}

/// Wrapper for os_mkdir_recurse() used by rs_shada_write_file.
/// Returns error code; sets *out_failed_dir to the failed directory (caller must free).
int nvim_shada_os_mkdir_recurse(const char *fname, int perm, char **out_failed_dir)
{
  return os_mkdir_recurse(fname, (int)perm, out_failed_dir, NULL);
}


/// Wrapper for vim_rename() used by rs_shada_write_file.
int nvim_shada_vim_rename(const char *from, const char *to)
{
  return vim_rename(from, to);
}

/// Wrapper for os_remove() used by rs_shada_write_file.
void nvim_shada_os_remove(const char *fname)
{
  os_remove(fname);
}

// =============================================================================
// Phase 4 (plan fd426e0f): nvim_shada_platform_check_writable migration accessors
// =============================================================================

/// Get stat fields (mode, uid, gid) for a file via os_fileinfo.
/// Returns 1 if file exists and is not a directory, 0 otherwise.
/// @param fname      File path.
/// @param out_mode   Output: st_mode (or 0 on failure).
/// @param out_uid    Output: st_uid as uint64_t.
/// @param out_gid    Output: st_gid as uint64_t.
int nvim_shada_os_fileinfo(const char *fname, uint64_t *out_mode,
                           uint64_t *out_uid, uint64_t *out_gid)
{
  FileInfo info;
  if (!os_fileinfo(fname, &info)) {
    *out_mode = 0;
    *out_uid = 0;
    *out_gid = 0;
    return 0;
  }
  *out_mode = (uint64_t)info.stat.st_mode;
  *out_uid  = (uint64_t)info.stat.st_uid;
  *out_gid  = (uint64_t)info.stat.st_gid;
  if (S_ISDIR(info.stat.st_mode)) {
    return 0;
  }
  return 1;
}

/// Wrapper for os_fchown on an open FileDescriptor.
/// @param sd_writer   FileDescriptor pointer (passed by Rust as void*).
/// @param uid         New owner uid.
/// @param gid         New owner gid.
/// @return Return value of os_fchown (0 on success, errno on failure).
int nvim_shada_os_fchown(void *sd_writer, uint64_t uid, uint64_t gid)
{
  return os_fchown(file_fd((FileDescriptor *)sd_writer),
                   (uv_uid_t)uid, (uv_gid_t)gid);
}

// =============================================================================
// Phase 3 (plan 11dd3cf4): shada_read migration accessors
// =============================================================================

/// Allocate and initialize a PMap(cstr_t) for fname_bufs caching.
void *nvim_shada_fname_bufs_new(void)
{
  PMap(cstr_t) *m = xcalloc(1, sizeof(PMap(cstr_t)));
  *m = (PMap(cstr_t))MAP_INIT;
  return m;
}

/// Destroy a PMap(cstr_t) and free all keys and the struct.
void nvim_shada_fname_bufs_destroy(void *handle)
{
  PMap(cstr_t) *m = (PMap(cstr_t) *)handle;
  const char *key;
  map_foreach_key(m, key, { xfree((char *)key); })
  map_destroy(cstr_t, m);
  xfree(m);
}

/// Allocate and initialize a Set(cstr_t) for oldfiles dedup.
void *nvim_shada_oldfiles_set_new(void)
{
  Set(cstr_t) *s = xcalloc(1, sizeof(Set(cstr_t)));
  *s = (Set(cstr_t))SET_INIT;
  return s;
}

/// Destroy a Set(cstr_t) and free the struct.
void nvim_shada_oldfiles_set_destroy(void *handle)
{
  Set(cstr_t) *s = (Set(cstr_t) *)handle;
  set_destroy(cstr_t, s);
  xfree(s);
}

/// Get VV_OLDFILES list (returns NULL if not set).
void *nvim_shada_get_oldfiles_list(void)
{
  return get_vim_var_list(VV_OLDFILES);
}

/// Get the length of a list_T.
int nvim_shada_tv_list_len(void *list)
{
  return (int)tv_list_len((list_T *)list);
}

/// Allocate a new list and set it as VV_OLDFILES. Returns the new list.
void *nvim_shada_create_oldfiles_list(void)
{
  list_T *list = tv_list_alloc(kListLenUnknown);
  set_vim_var_list(VV_OLDFILES, list);
  return list;
}

/// Return ARGCOUNT.
int nvim_shada_argcount(void)
{
  return ARGCOUNT;
}

/// Update w_changelistidx for all windows that have buffers in cl_bufs.
/// Called after shada_read completes.
void nvim_shada_for_all_tab_windows_update_changelist(void *cl_bufs_handle)
{
  Set(ptr_t) *cl_bufs = (Set(ptr_t) *)cl_bufs_handle;
  if (!cl_bufs->h.n_occupied) {
    return;
  }
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    (void)tp;
    if (set_has(ptr_t, cl_bufs, wp->w_buffer)) {
      wp->w_changelistidx = wp->w_buffer->b_changelistlen;
    }
  }
}

/// Clear a history type. Wrapper for clr_history().
void nvim_shada_clr_history(int i)
{
  clr_history(i);
}

/// Get the histentry_T array for history type i.
/// Sets *out_hisidx and *out_hisnum to pointers within the array.
/// Returns NULL if not available.
void *nvim_shada_hist_get_array(int i, int **out_hisidx, int **out_hisnum)
{
  return hist_get_array((uint8_t)i, out_hisidx, out_hisnum);
}

// =============================================================================
// Phase 1 accessors: shada_pack_entry migration
// =============================================================================

/// Wrapper for encode_vim_to_msgpack (for encoding typval_T variables).
int nvim_encode_vim_to_msgpack(PackerBuffer *packer, void *tv, const char *desc)
{
  return encode_vim_to_msgpack(packer, (typval_T *)tv, desc);
}

/// Return the number of additional data items in an AdditionalData struct.
uint32_t nvim_shada_additional_data_len(const void *ad_ptr)
{
  const AdditionalData *ad = (const AdditionalData *)ad_ptr;
  return ad ? ad->nitems : 0;
}

/// Write additional data raw bytes to a packer buffer.
void nvim_shada_dump_additional_data(const void *ad_ptr, PackerBuffer *sbuf)
{
  const AdditionalData *ad = (const AdditionalData *)ad_ptr;
  if (ad != NULL) {
    mpack_raw(ad->data, ad->nbytes, sbuf);
  }
}

/// Check if a ShadaEntry variable is a blob (v_type == VAR_BLOB).
int nvim_shada_entry_is_blob_var(const ShadaEntry *entry)
{
  return (entry && entry->data.global_var.value.v_type == VAR_BLOB) ? 1 : 0;
}

/// Get the pointer to the typval_T in a variable entry (for encode_vim_to_msgpack).
void *nvim_shada_entry_var_value_ptr(ShadaEntry *entry)
{
  return entry ? &entry->data.global_var.value : NULL;
}

/// Pack a header entry's Dict into a packer buffer (compound replacement for 7 individual
/// header field accessors). Writes a msgpack map with all header key-value pairs.
/// Called from Rust rs_shada_pack_entry for ShadaEntryType::Header.
void nvim_shada_pack_header_dict(const ShadaEntry *entry, PackerBuffer *sbuf)
{
  if (!entry) {
    return;
  }
  size_t header_size = entry->data.header.size;
  char *ptr = sbuf->ptr;
  mpack_map((uint8_t **)&ptr, (uint32_t)header_size);
  sbuf->ptr = ptr;
  for (size_t i = 0; i < header_size; i++) {
    mpack_check_buffer(sbuf);
    KeyValuePair *kv = &entry->data.header.items[i];
    mpack_str((String){ .data = kv->key.data, .size = kv->key.size }, sbuf);
    if (kv->value.type == kObjectTypeString) {
      mpack_bin((String){ .data = kv->value.data.string.data, .size = kv->value.data.string.size },
                sbuf);
    } else if (kv->value.type == kObjectTypeInteger) {
      mpack_integer(&sbuf->ptr, kv->value.data.integer);
    }
    // Other types are skipped (same as Rust behaviour).
  }
}

// Global variable iteration accessor (inlines var_shada_iter; plan b499a5d0 Phase 5).
// flavour is a bitmask of VAR_FLAVOUR_DEFAULT | VAR_FLAVOUR_SESSION | VAR_FLAVOUR_SHADA.
// On each call: *out_name set to variable name (static, do not free); *out_tv set to a
// freshly xmalloc'd typval_T copy (caller must pass to nvim_shada_build_gvar_entry which frees it).
// Returns the next iter pointer (NULL when exhausted).
const void *nvim_shada_var_shada_iter(const void *iter, const char **out_name, void **out_tv,
                                      unsigned flavour)
{
  hashtab_T *globvarht = get_globvar_ht();
  const hashitem_T *hifirst = globvarht->ht_array;
  const size_t hinum = (size_t)globvarht->ht_mask + 1;
  *out_name = NULL;
  const hashitem_T *hi;
  if (iter == NULL) {
    hi = globvarht->ht_array;
    while ((size_t)(hi - hifirst) < hinum
           && (HASHITEM_EMPTY(hi)
               || !(rs_var_flavour(hi->hi_key) & (var_flavour_T)flavour))) {
      hi++;
    }
    if ((size_t)(hi - hifirst) == hinum) {
      *out_tv = NULL;
      return NULL;
    }
  } else {
    hi = (const hashitem_T *)iter;
  }
  *out_name = TV_DICT_HI2DI(hi)->di_key;
  typval_T vartv;
  tv_copy(&TV_DICT_HI2DI(hi)->di_tv, &vartv);
  typval_T *tv_copy_ptr = xmalloc(sizeof(typval_T));
  *tv_copy_ptr = vartv;
  *out_tv = tv_copy_ptr;
  while ((size_t)(++hi - hifirst) < hinum) {
    if (!HASHITEM_EMPTY(hi) && (rs_var_flavour(hi->hi_key) & (var_flavour_T)flavour)) {
      return hi;
    }
  }
  return NULL;
}

// Get the v_type field of a typval_T pointer.
int nvim_shada_tv_get_type(const void *tv)
{
  return tv ? ((const typval_T *)tv)->v_type : 0;
}

// Build a ShadaEntry for a global variable into *out (C layout: inline typval_T).
// Copies tv into out->data.global_var.value, clears and frees tv (which was xmalloc'd
// by nvim_shada_var_shada_iter). Caller must call nvim_shada_clear_gvar_entry_value
// after rs_shada_pack_entry to release the copied typval. (plan b499a5d0 Phase 5)
void nvim_shada_build_gvar_entry(const char *name, void *tv, Timestamp ts, ShadaEntry *out)
{
  typval_T tgttv;
  tv_copy((typval_T *)tv, &tgttv);
  tv_clear((typval_T *)tv);
  xfree(tv);
  *out = (ShadaEntry){
    .type = kSDItemVariable,
    .timestamp = ts,
    .data = {
      .global_var = {
        .name = (char *)name,
        .value = tgttv,
      }
    },
    .additional_data = NULL,
  };
}

// Clear the inline typval_T of a global variable ShadaEntry built by nvim_shada_build_gvar_entry.
// Must be called after rs_shada_pack_entry to release the copied typval. (plan b499a5d0 Phase 5)
void nvim_shada_clear_gvar_entry_value(ShadaEntry *entry)
{
  tv_clear(&entry->data.global_var.value);
}

// =============================================================================
// Phase 2 accessor functions: shada_write / shada_read_when_writing migration
// =============================================================================

/// Set b_last_cursor for all windows in all tabs (wraps FOR_ALL_TAB_WINDOWS loop).
void nvim_shada_set_all_last_cursors(void)
{
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    set_last_cursor(wp);
  }
}

/// Get the longVersion string (Neovim version string).
const char *nvim_shada_get_longversion(void) { return longVersion; }

/// Get the current process ID.
int64_t nvim_shada_os_get_pid(void) { return os_get_pid(); }

/// Get the current encoding option (p_enc).
const char *nvim_shada_get_p_enc(void) { return p_enc; }

/// Iterate over global marks.
/// @param iter       Previous iterator or NULL to start.
/// @param out_name   Output: mark name character.
/// @param out_lnum   Output: mark line number.
/// @param out_col    Output: mark column.
/// @param out_fnum   Output: file number (0 if fname-based).
/// @param out_ts     Output: timestamp.
/// @param out_fname  Output: file name (used when fnum == 0, may be NULL).
/// @param out_additional Output: additional data pointer (may be NULL).
/// @return Next iterator value, or NULL when done.
const void *nvim_shada_mark_global_iter(const void *iter,
                                        char *out_name,
                                        int64_t *out_lnum,
                                        int32_t *out_col,
                                        int *out_fnum,
                                        uint64_t *out_ts,
                                        const char **out_fname,
                                        void **out_additional)
{
  xfmark_T fm;
  *out_name = NUL;
  const void *next = mark_global_iter(iter, out_name, &fm);
  if (*out_name != NUL) {
    *out_lnum = (int64_t)fm.fmark.mark.lnum;
    *out_col = (int32_t)fm.fmark.mark.col;
    *out_fnum = fm.fmark.fnum;
    *out_ts = fm.fmark.timestamp;
    *out_fname = fm.fname;
    *out_additional = fm.fmark.additional_data;
  }
  return next;
}

/// Get the global mark index for a mark name character.
int nvim_shada_mark_global_index(int name)
{
  return mark_global_index((char)name);
}

/// Get the local mark index for a mark name character.
int nvim_shada_mark_local_index(int name)
{
  return mark_local_index((char)name);
}

/// Get the timestamp of a named global mark (namedfm[idx].fmark.timestamp).
uint64_t nvim_shada_named_mark_timestamp(int idx)
{
  if (idx < 0 || idx >= NGLOBALMARKS) {
    return 0;
  }
  return namedfm[idx].fmark.timestamp;
}

/// Iterate over buffer-local marks.
/// @param iter         Previous iterator or NULL to start.
/// @param buf          Buffer to iterate.
/// @param out_name     Output: mark name character.
/// @param out_lnum     Output: mark line number.
/// @param out_col      Output: mark column.
/// @param out_ts       Output: timestamp.
/// @param out_additional Output: additional data pointer (may be NULL).
/// @return Next iterator value, or NULL when done.
const void *nvim_shada_mark_buffer_iter(const void *iter,
                                        const void *buf,
                                        char *out_name,
                                        int64_t *out_lnum,
                                        int32_t *out_col,
                                        uint64_t *out_ts,
                                        void **out_additional)
{
  fmark_T fm;
  *out_name = NUL;
  const void *next = mark_buffer_iter(iter, (const buf_T *)buf, out_name, &fm);
  if (*out_name != NUL) {
    *out_lnum = (int64_t)fm.mark.lnum;
    *out_col = (int32_t)fm.mark.col;
    *out_ts = fm.timestamp;
    *out_additional = fm.additional_data;
  }
  return next;
}

/// Get the changelist length of a buffer.
int nvim_shada_buf_changelist_len(const void *buf)
{
  if (!buf) {
    return 0;
  }
  return ((const buf_T *)buf)->b_changelistlen;
}

/// Get a changelist entry from a buffer.
/// @param buf          Buffer handle.
/// @param idx          Index into changelist.
/// @param out_lnum     Output: line number.
/// @param out_col      Output: column.
/// @param out_ts       Output: timestamp.
/// @param out_additional Output: additional data pointer.
void nvim_shada_buf_changelist_entry(const void *buf, int idx,
                                     int64_t *out_lnum, int32_t *out_col,
                                     uint64_t *out_ts, void **out_additional)
{
  if (!buf || idx < 0) {
    return;
  }
  const buf_T *b = (const buf_T *)buf;
  const fmark_T fm = b->b_changelist[idx];
  *out_lnum = (int64_t)fm.mark.lnum;
  *out_col = (int32_t)fm.mark.col;
  *out_ts = fm.timestamp;
  *out_additional = fm.additional_data;
}

/// Get the current substitute replacement string, its timestamp, and additional data.
/// @param out_sub      Output: replacement string (may be NULL if not set).
/// @param out_ts       Output: timestamp.
/// @param out_additional Output: additional data pointer (may be NULL).
void nvim_shada_sub_get_replacement(const char **out_sub, uint64_t *out_ts,
                                    void **out_additional)
{
  SubReplacementString sub;
  sub_get_replacement(&sub);
  *out_sub = sub.sub;
  *out_ts = sub.timestamp;
  *out_additional = sub.additional_data;
}

/// Get curwin->w_cursor.lnum.
int64_t nvim_shada_curwin_lnum(void)
{
  return (int64_t)curwin->w_cursor.lnum;
}

/// Get curwin->w_cursor as a Position.
/// @param out_lnum Output: line number.
/// @param out_col  Output: column.
void nvim_shada_curwin_cursor(int64_t *out_lnum, int32_t *out_col)
{
  *out_lnum = (int64_t)curwin->w_cursor.lnum;
  *out_col = (int32_t)curwin->w_cursor.col;
}

/// Put a file-marks entry into the WriteMergerState's file_marks PMap.
/// If the key is new, xstrdup's the fname. Returns pointer to the FileMarks*.
/// @param wms    WriteMergerState pointer (as void*).
/// @param fname  File name to use as key.
/// @param is_new Output: true if this is a new entry.
/// @param out_key Output: pointer to the key in the map (for ownership transfer).
/// @return pointer to the ptr_t value slot in the PMap.
void **nvim_shada_wms_file_marks_put_ref(void *wms_opaque, const char *fname,
                                         bool *is_new, const char **out_key)
{
  WriteMergerState *wms = (WriteMergerState *)wms_opaque;
  if (!wms || !fname) {
    return NULL;
  }
  ptr_t *val = pmap_put_ref(cstr_t)(&wms->file_marks, fname, (cstr_t **)out_key, is_new);
  return (void **)val;
}


/// Collect all FileMarks from the PMap, sort by greatest_timestamp (descending),
/// and return as an allocated array of void* pointers.
/// Caller must xfree the returned array.
void **nvim_shada_wms_file_marks_get_sorted(const void *wms_opaque, size_t *out_size)
{
  const WriteMergerState *wms = (const WriteMergerState *)wms_opaque;
  if (!wms) {
    *out_size = 0;
    return NULL;
  }
  size_t sz = map_size(&wms->file_marks);
  *out_size = sz;
  if (sz == 0) {
    return NULL;
  }
  void **arr = xmalloc(sz * sizeof(*arr));
  size_t i = 0;
  ptr_t val;
  map_foreach_value((PMap(cstr_t) *)&wms->file_marks, val, {
    arr[i++] = val;
  })
  qsort(arr, sz, sizeof(*arr), &rs_compare_file_marks);
  return arr;
}

/// Destroy the file_marks PMap in the WriteMergerState.
/// Frees all keys and FileMarks values.
void nvim_shada_wms_file_marks_destroy(void *wms_opaque)
{
  WriteMergerState *wms = (WriteMergerState *)wms_opaque;
  if (!wms) {
    return;
  }
  const char *key = NULL;
  ptr_t val;
  map_foreach(&wms->file_marks, key, val, {
    xfree((char *)key);
    xfree(val);
  })
  map_destroy(cstr_t, &wms->file_marks);
}

/// Check if a variable name is in the dumped_variables set.
bool nvim_shada_wms_dumped_vars_has(const void *wms_opaque, const char *name)
{
  const WriteMergerState *wms = (const WriteMergerState *)wms_opaque;
  if (!wms || !name) {
    return false;
  }
  return set_has(cstr_t, (Set(cstr_t) *)&wms->dumped_variables, name);
}

/// Add a variable name to the dumped_variables set.
void nvim_shada_wms_dumped_vars_put(void *wms_opaque, const char *name)
{
  WriteMergerState *wms = (WriteMergerState *)wms_opaque;
  if (!wms || !name) {
    return;
  }
  set_put(cstr_t, &wms->dumped_variables, name);
}

/// Destroy the dumped_variables set in the WriteMergerState.
void nvim_shada_wms_dumped_vars_destroy(void *wms_opaque)
{
  WriteMergerState *wms = (WriteMergerState *)wms_opaque;
  if (wms) {
    set_destroy(cstr_t, &wms->dumped_variables);
  }
}


/// Get the (lnum, col) of the mark returned by mark_get for a local mark.
/// Returns 1 if a mark was found with timestamp >= entry_ts, 0 otherwise.
int nvim_shada_mark_get_cmp(const void *buf, const void *win, int name, uint64_t entry_ts)
{
  if (!buf) {
    return 0;
  }
  fmark_T fm_storage;
  fmark_T *fm = mark_get((buf_T *)buf, (win_T *)win, &fm_storage,
                          kMarkBufLocal, name);
  if (fm == NULL) {
    return 0;
  }
  return (fm->timestamp >= entry_ts) ? 1 : 0;
}

/// Wrapper for nvim_mark_path_fnamecmp (path comparison for marks).
int nvim_shada_path_fnamecmp(const char *a, const char *b)
{
  return nvim_mark_path_fnamecmp(a, b);
}


/// Flush the packer buffer.

/// Internal flush callback for file-backed PackerBuffers.
static void nvim_shada_flush_file_buffer_(PackerBuffer *buffer)
{
  FileDescriptor *fd = buffer->anydata;
  fd->write_pos = buffer->ptr;
  buffer->anyint = file_flush(fd);
  buffer->ptr = fd->write_pos;
}

/// Initialize a PackerBuffer for writing to a FileDescriptor (by pointer).
/// @param fd     FileDescriptor to write to.
/// @param out    Output: initialized PackerBuffer.
void nvim_shada_packer_init_for_file(void *fd, PackerBuffer *out)
{
  FileDescriptor *file = (FileDescriptor *)fd;
  if (file_space(file) < SHADA_MPACK_FREE_SPACE) {
    file_flush(file);
  }
  *out = (PackerBuffer) {
    .startptr = file->buffer,
    .ptr = file->write_pos,
    .endptr = file->buffer + ARENA_BLOCK_SIZE,
    .anydata = file,
    .anyint = 0,
    .packer_flush = nvim_shada_flush_file_buffer_,
  };
}

// =============================================================================
// Phase 3 (plan fd426e0f): nvim_shada_pack_all_gvars migration accessors
// =============================================================================

/// Get refcheck info from a typval for circular-reference detection.
/// @param tv           xmalloc'd typval_T pointer.
/// @param out_vtype    Output: v_type of the typval.
/// @param out_container Output: for VAR_DICT: &dv_hashtab; for VAR_LIST: v_list; else NULL.
/// @param out_copy_id  Output: dv_copyID or lv_copyID (0 if container is NULL).
void nvim_shada_tv_get_refcheck_info(const void *tv, int *out_vtype,
                                     void **out_container, int *out_copy_id)
{
  const typval_T *t = (const typval_T *)tv;
  if (!t) {
    *out_vtype = 0;
    *out_container = NULL;
    *out_copy_id = 0;
    return;
  }
  *out_vtype = t->v_type;
  if (t->v_type == VAR_DICT) {
    if (t->vval.v_dict) {
      *out_container = &t->vval.v_dict->dv_hashtab;
      *out_copy_id = t->vval.v_dict->dv_copyID;
    } else {
      *out_container = NULL;
      *out_copy_id = 0;
    }
  } else if (t->v_type == VAR_LIST) {
    if (t->vval.v_list) {
      *out_container = t->vval.v_list;
      *out_copy_id = t->vval.v_list->lv_copyID;
    } else {
      *out_container = NULL;
      *out_copy_id = 0;
    }
  } else {
    *out_container = NULL;
    *out_copy_id = 0;
  }
}

// =============================================================================
// Phase 1 (plan b499a5d0): thin C accessors for search/sub apply migration
// =============================================================================

/// Get current search or substitute pattern timestamp (0 if no pattern set).
uint64_t nvim_shada_get_search_pattern_timestamp(int is_substitute)
{
  SearchPattern pat;
  if (is_substitute) {
    get_substitute_pattern(&pat);
  } else {
    get_search_pattern(&pat);
  }
  return pat.pat != NULL ? (uint64_t)pat.timestamp : 0;
}

/// Build SearchPattern from entry fields and call set_search_pattern or
/// set_substitute_pattern depending on is_substitute.
/// Memory ownership: entry's pat.data and additional_data are consumed.
void nvim_shada_set_search_pattern_from_entry(ShadaEntry *entry, int is_substitute)
{
  SearchPattern spat = (SearchPattern) {
    .magic = entry->data.search_pattern.magic,
    .no_scs = !entry->data.search_pattern.smartcase,
    .off = {
      .dir = entry->data.search_pattern.search_backward ? '?' : '/',
      .line = entry->data.search_pattern.has_line_offset,
      .end = entry->data.search_pattern.place_cursor_at_end,
      .off = entry->data.search_pattern.offset,
    },
    .pat = entry->data.search_pattern.pat.data,
    .patlen = entry->data.search_pattern.pat.size,
    .additional_data = entry->additional_data,
    .timestamp = entry->timestamp,
  };
  if (is_substitute) {
    set_substitute_pattern(spat);
  } else {
    set_search_pattern(spat);
  }
}

/// Wrap set_last_used_pattern(is_substitute).
void nvim_shada_set_last_used_pattern(int is_substitute)
{
  set_last_used_pattern((bool)is_substitute);
}

/// Wrap set_no_hlsearch(val).
void nvim_shada_set_no_hlsearch(int val)
{
  set_no_hlsearch((bool)val);
}

/// Get current substitute replacement string timestamp (0 if no sub set).
uint64_t nvim_shada_get_sub_replacement_timestamp(void)
{
  SubReplacementString sub;
  sub_get_replacement(&sub);
  return sub.sub != NULL ? (uint64_t)sub.timestamp : 0;
}

/// Build SubReplacementString from entry and call sub_set_replacement + regtilde.
/// Memory ownership: entry's sub_string.sub and additional_data are consumed.
void nvim_shada_set_sub_replacement_from_entry(ShadaEntry *entry)
{
  sub_set_replacement((SubReplacementString) {
    .sub = entry->data.sub_string.sub,
    .timestamp = entry->timestamp,
    .additional_data = entry->additional_data,
  });
  regtilde(entry->data.sub_string.sub, rs_magic_isset(), false);
}

// Phase 2 (plan b499a5d0): thin C accessors for register/variable apply migration

/// Return 1 if register type is char/line/block-wise (valid for ShaDa), 0 otherwise.
int nvim_shada_entry_get_reg_type_valid(const ShadaEntry *entry)
{
  return (entry->data.reg.type == kMTCharWise
          || entry->data.reg.type == kMTLineWise
          || entry->data.reg.type == kMTBlockWise) ? 1 : 0;
}

/// Get timestamp of named register (0 if register is NULL).
uint64_t nvim_shada_op_reg_get_timestamp(char name)
{
  const yankreg_T *const reg = op_reg_get(name);
  return reg != NULL ? (uint64_t)reg->timestamp : 0;
}

/// Build yankreg_T from entry fields and call op_reg_set.
/// Returns 1 if memory was consumed (do not free entry), 0 if op_reg_set rejected.
int nvim_shada_op_reg_set_from_entry(ShadaEntry *entry)
{
  return op_reg_set(entry->data.reg.name, (yankreg_T) {
    .y_array = entry->data.reg.contents,
    .y_size = entry->data.reg.contents_size,
    .y_type = entry->data.reg.type,
    .y_width = (colnr_T)entry->data.reg.width,
    .timestamp = entry->timestamp,
    .additional_data = entry->additional_data,
  }, entry->data.reg.is_unnamed) ? 1 : 0;
}

/// Call var_set_global with entry's name and value, then clear the typval.
/// After this call, the entry's value field is zeroed (VAR_UNKNOWN).
void nvim_shada_var_set_global_from_entry(ShadaEntry *entry)
{
  var_set_global(entry->data.global_var.name, &entry->data.global_var.value);
  entry->data.global_var.value.v_type = VAR_UNKNOWN;
}

// Phase 4 (plan b499a5d0): thin C accessors for mark/jump and local/change apply migration

/// Build xfmark_T from entry fields and call mark_set_global.
/// Handles buf lookup, XFREE_CLEAR of fname when buf found.
/// @param no_overwrite  Pass !force to mark_set_global.
/// @return 1 if mark was set (memory consumed), 0 if not set.
int nvim_shada_mark_set_global_from_entry(ShadaEntry *entry, void *fname_bufs_handle,
                                          int no_overwrite)
{
  buf_T *buf = nvim_shada_find_buffer(fname_bufs_handle, entry->data.filemark.fname);
  if (buf != NULL) {
    XFREE_CLEAR(entry->data.filemark.fname);
  }
  xfmark_T fm = (xfmark_T) {
    .fname = buf == NULL ? entry->data.filemark.fname : NULL,
    .fmark = {
      .mark = entry->data.filemark.mark,
      .fnum = (buf == NULL ? 0 : buf->b_fnum),
      .timestamp = entry->timestamp,
      .view = INIT_FMARKV,
      .additional_data = entry->additional_data,
    },
  };
  return mark_set_global(entry->data.filemark.name, fm, (bool)no_overwrite) ? 1 : 0;
}

/// Return curwin->w_jumplistlen.
int nvim_shada_jumplist_len(void) { return curwin->w_jumplistlen; }

/// Return all fields of curwin->w_jumplist[idx] in one call.
void nvim_shada_jumplist_get_entry(int idx, uint64_t *out_ts, int64_t *out_lnum,
                                   int32_t *out_col, int *out_fnum,
                                   const char **out_fname)
{
  const xfmark_T *jl = &curwin->w_jumplist[idx];
  *out_ts = (uint64_t)jl->fmark.timestamp;
  *out_lnum = (int64_t)jl->fmark.mark.lnum;
  *out_col = (int32_t)jl->fmark.mark.col;
  *out_fnum = jl->fmark.fnum;
  *out_fname = jl->fname;
}

/// Insert a jumplist entry at position i from a ShadaEntry.
/// Frees curwin->w_jumplist[0] if needed (when i > 0 && jl_len == JUMPLISTSIZE),
/// builds and assigns xfmark_T from entry, then updates len and idx.
void nvim_shada_jumplist_insert_entry(int i, ShadaEntry *entry,
                                      void *fname_bufs_handle, int jl_len)
{
  if (i > 0 && jl_len == JUMPLISTSIZE) {
    free_xfmark(curwin->w_jumplist[0]);
  }
  buf_T *buf = nvim_shada_find_buffer(fname_bufs_handle, entry->data.filemark.fname);
  if (buf != NULL) {
    XFREE_CLEAR(entry->data.filemark.fname);
  }
  curwin->w_jumplist[i] = (xfmark_T) {
    .fname = buf == NULL ? entry->data.filemark.fname : NULL,
    .fmark = {
      .mark = entry->data.filemark.mark,
      .fnum = (buf == NULL ? 0 : buf->b_fnum),
      .timestamp = entry->timestamp,
      .view = INIT_FMARKV,
      .additional_data = entry->additional_data,
    },
  };
  if (curwin->w_jumplistlen < JUMPLISTSIZE) {
    curwin->w_jumplistlen++;
  }
  if (curwin->w_jumplistidx >= i
      && curwin->w_jumplistidx + 1 <= curwin->w_jumplistlen) {
    curwin->w_jumplistidx++;
  }
}

// Phase 3 (plan b499a5d0): thin C accessors for buffer list apply migration

/// Wrap path_try_shorten_fname. Returns shortened fname (may be original pointer).
char *nvim_shada_path_try_shorten_fname(const char *fname)
{
  return path_try_shorten_fname((char *)fname);
}

/// Wrap buflist_new(fname, sfname, 0, BLN_LISTED). Returns buf_T* or NULL.
void *nvim_shada_buflist_new(const char *fname, const char *sfname)
{
  return buflist_new((char *)fname, (char *)sfname, 0, BLN_LISTED);
}

/// Set buffer cursor position and additional data from a buffer list entry.
/// Combines RESET_FMARK, buflist_setfpos, and additional_data ownership transfer.
/// @param buf_handle  buf_T* (non-NULL).
/// @param entry       ShadaEntry* containing buffer_list.
/// @param i           Index into entry->data.buffer_list.buffers.
void nvim_shada_buf_set_cursor_and_data(void *buf_handle, ShadaEntry *entry, size_t i)
{
  buf_T *const buf = (buf_T *)buf_handle;
  fmarkv_T view = INIT_FMARKV;
  RESET_FMARK(&buf->b_last_cursor,
              entry->data.buffer_list.buffers[i].pos, 0, view);
  buflist_setfpos(buf, curwin, buf->b_last_cursor.mark.lnum,
                  buf->b_last_cursor.mark.col, false);
  xfree(buf->additional_data);
  buf->additional_data = entry->data.buffer_list.buffers[i].additional_data;
  entry->data.buffer_list.buffers[i].additional_data = NULL;
}

/// Handle oldfiles set/list update for a filemark entry.
/// Adds fname to oldfiles_set and oldfiles_list if get_old_files is true and fname not yet seen.
/// If want_marks is false, takes ownership of fname (sets entry->data.filemark.fname = NULL).
/// If want_marks is true, duplicates fname for the set; the entry retains its fname.
void nvim_shada_oldfiles_add(void *oldfiles_set_handle, void *oldfiles_list,
                             ShadaEntry *entry, int want_marks)
{
  Set(cstr_t) *oldfiles_set = (Set(cstr_t) *)oldfiles_set_handle;
  list_T *old_list = (list_T *)oldfiles_list;
  char *fname = entry->data.filemark.fname;
  if (want_marks) {
    fname = xstrdup(fname);
  }
  set_put(cstr_t, oldfiles_set, fname);
  tv_list_append_allocated_string(old_list, fname);
  if (!want_marks) {
    entry->data.filemark.fname = NULL;
  }
}

/// Return 1 if entry->data.filemark.fname is already in oldfiles_set, 0 otherwise.
int nvim_shada_oldfiles_has(void *oldfiles_set_handle, const ShadaEntry *entry)
{
  Set(cstr_t) *oldfiles_set = (Set(cstr_t) *)oldfiles_set_handle;
  return set_has(cstr_t, oldfiles_set, entry->data.filemark.fname) ? 1 : 0;
}

/// Build fmark_T from entry and call mark_set_local.
/// @param no_overwrite  Pass !force.
/// @return 1 if mark was set (memory consumed), 0 if not set.
int nvim_shada_mark_set_local_from_entry(ShadaEntry *entry, void *buf_handle, int no_overwrite)
{
  buf_T *buf = (buf_T *)buf_handle;
  fmark_T fm = (fmark_T) {
    .mark = entry->data.filemark.mark,
    .fnum = 0,
    .timestamp = entry->timestamp,
    .view = INIT_FMARKV,
    .additional_data = entry->additional_data,
  };
  return mark_set_local(entry->data.filemark.name, buf, fm, (bool)no_overwrite) ? 1 : 0;
}

/// call set_put(ptr_t, cl_bufs, buf).
void nvim_shada_cl_bufs_set_put(void *cl_bufs_handle, void *buf_handle)
{
  Set(ptr_t) *cl_bufs = (Set(ptr_t) *)cl_bufs_handle;
  set_put(ptr_t, cl_bufs, buf_handle);
}

/// Return buf->b_changelistlen.
int nvim_shada_buf_get_changelistlen(const void *buf_handle)
{
  return ((const buf_T *)buf_handle)->b_changelistlen;
}

/// Return all fields of buf->b_changelist[idx] in one call.
void nvim_shada_changelist_get_entry(const void *buf_handle, int idx,
                                     uint64_t *out_ts, int64_t *out_lnum,
                                     int32_t *out_col)
{
  const fmark_T *fm = &((const buf_T *)buf_handle)->b_changelist[idx];
  *out_ts = (uint64_t)fm->timestamp;
  *out_lnum = (int64_t)fm->mark.lnum;
  *out_col = (int32_t)fm->mark.col;
}

/// Insert a changelist entry at position i from a ShadaEntry.
/// Frees buf->b_changelist[0] if needed (when i > 0 && cl_len == JUMPLISTSIZE),
/// builds and assigns fmark_T from entry, then updates b_changelistlen.
void nvim_shada_changelist_insert_entry(void *buf_handle, int i,
                                        ShadaEntry *entry, int cl_len)
{
  buf_T *buf = (buf_T *)buf_handle;
  if (i > 0 && cl_len == JUMPLISTSIZE) {
    free_fmark(buf->b_changelist[0]);
  }
  buf->b_changelist[i] = (fmark_T) {
    .mark = entry->data.filemark.mark,
    .fnum = 0,
    .timestamp = entry->timestamp,
    .view = INIT_FMARKV,
    .additional_data = entry->additional_data,
  };
  if (buf->b_changelistlen < JUMPLISTSIZE) {
    buf->b_changelistlen++;
  }
}

/// Free entry->data.filemark.fname via xfree.
void nvim_shada_fm_xfree_fname(ShadaEntry *entry)
{
  xfree(entry->data.filemark.fname);
  entry->data.filemark.fname = NULL;
}

/// Return buf->b_fnum.
int nvim_shada_buf_get_fnum(const void *buf_handle)
{
  return ((const buf_T *)buf_handle)->b_fnum;
}

/// Call rs_marklist_insert on curwin->w_jumplist.
/// Returns the new insertion index, or -1 if duplicate.
int nvim_shada_jumplist_marklist_insert(int i)
{
  return rs_marklist_insert(curwin->w_jumplist, sizeof(*curwin->w_jumplist),
                            curwin->w_jumplistlen, i);
}

/// Call rs_marklist_insert on buf->b_changelist.
/// Returns the new insertion index, or -1 if duplicate.
int nvim_shada_changelist_marklist_insert(void *buf_handle, int i)
{
  buf_T *buf = (buf_T *)buf_handle;
  return rs_marklist_insert(buf->b_changelist, sizeof(*buf->b_changelist),
                            buf->b_changelistlen, i);
}

// =============================================================================
// Phase 2 (plan 92c8078e): shada_read_next_item migration accessors
// =============================================================================

/// Thin wrapper for file_try_read_buffered.
/// Returns pointer to buffered data or NULL if not available.
char *nvim_shada_file_try_read_buffered(void *fd, size_t len)
{
  return file_try_read_buffered((FileDescriptor *)fd, len);
}

/// Thin wrapper to read bytes_read from a FileDescriptor.
uint64_t nvim_shada_file_bytes_read(void *fd)
{
  return (uint64_t)((FileDescriptor *)fd)->bytes_read;
}
