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
// rs_vim_be64toh extern removed (Phase 3 plan 92c8078e): vim_be64toh wrapper deleted.
extern int rs_marks_equal(pos_T a, pos_T b);
extern int rs_marklist_insert(void *jumps_arr, size_t jump_size, int jl_len, int i);
extern int rs_compare_file_marks(const void *a, const void *b);
extern void rs_shada_free_entry_contents(ShadaEntry *entry);
extern int rs_shada_write_file(const char *file, bool nomerge);
extern void rs_shada_read(void *sd_reader, int flags);
extern var_flavour_T rs_var_flavour(const char *varname);
extern int rs_shada_pack_entry(PackerBuffer *packer, const ShadaEntry *entry, size_t max_kbyte);

// HAVE_BE64TOH block deleted (Phase 3 plan 92c8078e): vim_be64toh/be64toh wrapper deleted.

// SEARCH_KEY_* macros deleted (Phase 3 plan 92c8078e): unused after shada_read_next_item migration.

#define REG_KEY_TYPE rt
#define REG_KEY_WIDTH rw
#define REG_KEY_CONTENTS rc
#define REG_KEY_UNNAMED ru

// KEY_LNUM, KEY_COL, KEY_FILE deleted (Phase 3 plan 92c8078e): unused after shada_read_next_item migration.
#define KEY_NAME_CHAR n

/// Common prefix for all errors inside ShaDa file
///
/// I.e. errors occurred while parsing, but not system errors occurred while
/// reading.
#define RERR "E575: "

/// Common prefix for critical read errors
///
/// I.e. errors that make shada_read_next_item return kSDReadStatusNotShaDa.
#define RCERR "E576: "

/// Common prefix for all “system” errors
#define SERR "E886: "

/// Common prefix for all “rename” errors
#define RNERR "E136: "

/// Common prefix for all ignorable “write” errors
#define WERR "E574: "

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

#define DEF_SDE(name, attr, ...) \
  [kSDItem##name] = { \
    .timestamp = 0, \
    .type = kSDItem##name, \
    .additional_data = NULL, \
    .data = { \
      .attr = { __VA_ARGS__ } \
    } \
  }
#define DEFAULT_POS { 1, 0, 0 }
static const pos_T default_pos = DEFAULT_POS;
static const ShadaEntry sd_default_values[] = {
  [kSDItemMissing] = { .type = kSDItemMissing, .timestamp = 0 },
  DEF_SDE(Header, header, .size = 0),
  DEF_SDE(SearchPattern, search_pattern,
          .magic = true,
          .smartcase = false,
          .has_line_offset = false,
          .place_cursor_at_end = false,
          .offset = 0,
          .is_last_used = true,
          .is_substitute_pattern = false,
          .highlighted = false,
          .search_backward = false,
          .pat = STRING_INIT),
  DEF_SDE(SubString, sub_string, .sub = NULL),
  DEF_SDE(HistoryEntry, history_item,
          .histtype = HIST_CMD,
          .string = NULL,
          .sep = NUL),
  DEF_SDE(Register, reg,
          .name = NUL,
          .type = kMTCharWise,
          .contents = NULL,
          .contents_size = 0,
          .is_unnamed = false,
          .width = 0),
  DEF_SDE(Variable, global_var,
          .name = NULL,
          .value = { .v_type = VAR_UNKNOWN, .vval = { .v_string = NULL } }),
  DEF_SDE(GlobalMark, filemark,
          .name = '"',
          .mark = DEFAULT_POS,
          .fname = NULL),
  DEF_SDE(Jump, filemark,
          .name = NUL,
          .mark = DEFAULT_POS,
          .fname = NULL),
  DEF_SDE(BufferList, buffer_list,
          .size = 0,
          .buffers = NULL),
  DEF_SDE(LocalMark, filemark,
          .name = '"',
          .mark = DEFAULT_POS,
          .fname = NULL),
  DEF_SDE(Change, filemark,
          .name = NUL,
          .mark = DEFAULT_POS,
          .fname = NULL),
};
#undef DEFAULT_POS
#undef DEF_SDE

// sd_reader_skip deleted (Phase 2 plan 92c8078e): Rust rs_sd_reader_skip_bytes replaces it.

/// Iterate over global variables
///
/// @warning No modifications to global variable Dict must be performed
///          while iteration is in progress.
///
/// @param[in]   iter   Iterator. Pass NULL to start iteration.
/// @param[out]  name   Variable name.
/// @param[out]  rettv  Variable value.
///
/// @return Pointer that needs to be passed to next `var_shada_iter` invocation
///         or NULL to indicate that iteration is over.
static const void *var_shada_iter(const void *const iter, const char **const name, typval_T *rettv,
                                  var_flavour_T flavour)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ARG(2, 3)
{
  const hashitem_T *hi;
  hashtab_T *globvarht = get_globvar_ht();
  const hashitem_T *hifirst = globvarht->ht_array;
  const size_t hinum = (size_t)globvarht->ht_mask + 1;
  *name = NULL;
  if (iter == NULL) {
    hi = globvarht->ht_array;
    while ((size_t)(hi - hifirst) < hinum
           && (HASHITEM_EMPTY(hi)
               || !(rs_var_flavour(hi->hi_key) & flavour))) {
      hi++;
    }
    if ((size_t)(hi - hifirst) == hinum) {
      return NULL;
    }
  } else {
    hi = (const hashitem_T *)iter;
  }
  *name = TV_DICT_HI2DI(hi)->di_key;
  tv_copy(&TV_DICT_HI2DI(hi)->di_tv, rettv);
  while ((size_t)(++hi - hifirst) < hinum) {
    if (!HASHITEM_EMPTY(hi) && (rs_var_flavour(hi->hi_key) & flavour)) {
      return hi;
    }
  }
  return NULL;
}

/// Find buffer for given buffer name (cached)
///
/// @param[in,out]  fname_bufs  Cache containing fname to buffer mapping.
/// @param[in]      fname       File name to find.
///
/// @return Pointer to the buffer or NULL.
static buf_T *find_buffer(PMap(cstr_t) *const fname_bufs, const char *const fname)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
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

#define KEY_NAME_(s) #s
// PACK_KEY deleted (Phase 3 plan 92c8078e): unused after shada_read_next_item migration.
#define KEY_NAME(s) KEY_NAME_(s)

#define SHADA_MPACK_FREE_SPACE (4 * MPACK_ITEM_SIZE)


// shada_check_status deleted (Phase 2 plan 92c8078e): only used by shada_read_next_item.





// vim_be64toh deleted (Phase 2 plan 92c8078e): only used by msgpack_read_uint64.

// fread_len deleted (Phase 2 plan 92c8078e): Rust rs_fread_len replaces it.

// msgpack_read_uint64 deleted (Phase 2 plan 92c8078e): Rust rs_msgpack_read_uint64 replaces it.

#define READERR(entry_name, error_desc) \
  RERR "Error while reading ShaDa file: " \
  entry_name " entry at position %" PRIu64 " " \
  error_desc

// shada_read_next_item deleted (Phase 2 plan 92c8078e): Rust rs_shada_read_next_item replaces it.

// =============================================================================
// Accessor functions for Rust shada crate (ShadaPackerBuffer == PackerBuffer)
// =============================================================================

/// Get the current write pointer from a packer buffer
uint8_t *nvim_shada_packer_get_ptr(PackerBuffer *packer)
{
  return (uint8_t *)(packer ? packer->ptr : NULL);
}

/// Set the current write pointer in a packer buffer
void nvim_shada_packer_set_ptr(PackerBuffer *packer, uint8_t *ptr)
{
  if (packer) {
    packer->ptr = (char *)ptr;
  }
}

/// Get the end pointer from a packer buffer
uint8_t *nvim_shada_packer_get_endptr(PackerBuffer *packer)
{
  return (uint8_t *)(packer ? packer->endptr : NULL);
}

/// Flush the packer buffer
void nvim_shada_packer_flush(PackerBuffer *packer)
{
  if (packer && packer->packer_flush) {
    packer->packer_flush(packer);
  }
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

bool nvim_strequal(const char *s1, const char *s2)
{
  if (s1 == s2) {
    return true;
  }
  if (!s1 || !s2) {
    return false;
  }
  return strcmp(s1, s2) == 0;
}


Timestamp nvim_filemarks_get_greatest_timestamp(const void *fm_ptr)
{
  const FileMarks *fm = fm_ptr;
  return fm ? fm->greatest_timestamp : 0;
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
int nvim_shada_buf_is_listed(const void *buf) { return buf ? ((const buf_T *)buf)->b_p_bl : 0; }
int nvim_shada_buf_is_quickfix(const void *buf) { return buf ? bt_quickfix((const buf_T *)buf) : 0; }
int nvim_shada_buf_is_terminal(const void *buf) { return buf ? bt_terminal((const buf_T *)buf) : 0; }

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

// Search pattern accessors
void nvim_shada_get_search_pattern(char **out_pat, int *out_magic, int *out_no_scs,
                                   Timestamp *out_ts, int *out_off_line, int *out_off_end,
                                   int64_t *out_off_off, char *out_off_dir,
                                   void **out_additional_data)
{
  SearchPattern pat;
  get_search_pattern(&pat);
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

void nvim_shada_get_substitute_pattern(char **out_pat, int *out_magic, int *out_no_scs,
                                       Timestamp *out_ts, int *out_off_line, int *out_off_end,
                                       int64_t *out_off_off, char *out_off_dir,
                                       void **out_additional_data)
{
  SearchPattern pat;
  get_substitute_pattern(&pat);
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
void nvim_shada_buf_get_cursor(const void *buf, pos_T *pos)
{
  if (buf) {
    *pos = ((const buf_T *)buf)->b_last_cursor.mark;
  }
}
void *nvim_shada_buf_get_additional_data(const void *buf)
{
  return buf ? ((const buf_T *)buf)->additional_data : NULL;
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

void nvim_shada_siemsg(const char *msg)
{
  siemsg("%s", msg);
}

/// Free a Dict (api_free_dict wrapper)
void nvim_shada_api_free_dict(Dict value)
{
  api_free_dict(value);
}

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

/// Free register contents — each element is a String struct ({char*, size_t}).
/// Frees each String's data and then the array itself.
void nvim_shada_free_reg_contents(void *contents_ptr, size_t contents_size)
{
  String *contents = (String *)contents_ptr;
  for (size_t i = 0; i < contents_size; i++) {
    api_free_string(contents[i]);
  }
  xfree(contents);
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

/// Error message wrapper for close_file errors
void nvim_shada_semsg_close_error(const char *strerror_msg)
{
  semsg(_(SERR "System error while closing ShaDa file: %s"), strerror_msg);
}

/// Error message wrapper for open-for-read errors
void nvim_shada_semsg_open_error(const char *fname, const char *strerror_msg)
{
  semsg(_(SERR "System error while opening ShaDa file %s for reading: %s"),
        fname, strerror_msg);
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

/// Verbose message "Writing ShaDa file" used by rs_shada_write_file.
void nvim_shada_smsg_writing(const char *fname)
{
  smsg(0, _("Writing ShaDa file \"%s\""), fname);
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

/// Error: ShaDa file is not writable (E137).
void nvim_shada_semsg_not_writable(const char *fname)
{
  semsg(_("E137: ShaDa file is not writable: %s"), fname);
}

/// Error: fchown failed while writing ShaDa file.
void nvim_shada_semsg_fchown_error(const char *tempname, const char *strerror_msg)
{
  semsg(_(RNERR "Failed setting uid and gid for file %s: %s"),
        tempname, strerror_msg);
}

/// Error: merge reader open failed (for non-ENOENT errors)
void nvim_shada_semsg_merge_read_error(const char *fname, const char *strerror_msg)
{
  semsg(_(SERR "System error while opening ShaDa file %s for reading "
          "to merge before writing it: %s"), fname, strerror_msg);
}

/// Error: temp file open failed
void nvim_shada_semsg_tempfile_open_error(const char *tempname, const char *strerror_msg)
{
  semsg(_(SERR "System error while opening temporary ShaDa file %s "
          "for writing: %s"), tempname, strerror_msg);
}

/// Error: all .tmp.X files exist
void nvim_shada_semsg_all_tmpfiles(const char *fname)
{
  semsg(_("E138: All %s.tmp.X files exist, cannot write ShaDa file!"), fname);
}

/// Error: mkdir failed
void nvim_shada_semsg_mkdir_error(const char *failed_dir, const char *strerror_msg)
{
  semsg(_(SERR "Failed to create directory %s "
          "for writing ShaDa file: %s"), failed_dir, strerror_msg);
}

/// Error: ShaDa file open for writing failed
void nvim_shada_semsg_write_open_error(const char *fname, const char *strerror_msg)
{
  semsg(_(SERR "System error while opening ShaDa file %s for writing: %s"),
        fname, strerror_msg);
}

/// Error: rename failed
void nvim_shada_semsg_rename_error(const char *tempname, const char *fname)
{
  semsg(_(RNERR "Can't rename ShaDa file from %s to %s!"), tempname, fname);
}

/// Error: did not rename (not shada)
void nvim_shada_semsg_not_shada(const char *tempname, const char *fname)
{
  semsg(_(RNERR "Did not rename %s because %s does not look like a ShaDa file"),
        tempname, fname);
}

/// Error: did not rename (write errors)
void nvim_shada_semsg_write_errors(const char *tempname, const char *fname)
{
  semsg(_(RNERR "Did not rename %s to %s because there were errors "
          "during writing it"), tempname, fname);
}

/// Reminder: do not forget to remove temp file
void nvim_shada_semsg_remove_reminder(const char *tempname, const char *fname)
{
  semsg(_(RNERR "Do not forget to remove %s or rename it manually to %s."),
        tempname, fname);
}

// =============================================================================
// Phase 3 (plan 11dd3cf4): shada_read migration accessors
// =============================================================================

// nvim_shada_read_next_item deleted (Phase 2 plan 92c8078e): Rust calls rs_shada_read_next_item directly.

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

/// Allocate and initialize a Set(ptr_t) for cl_bufs.
void *nvim_shada_cl_bufs_new(void)
{
  Set(ptr_t) *s = xcalloc(1, sizeof(Set(ptr_t)));
  *s = (Set(ptr_t))SET_INIT;
  return s;
}

/// Destroy a Set(ptr_t) and free the struct.
void nvim_shada_cl_bufs_destroy(void *handle)
{
  Set(ptr_t) *s = (Set(ptr_t) *)handle;
  set_destroy(ptr_t, s);
  xfree(s);
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

// nvim_shada_apply_entry was deleted in Phase 1 (plan 92c8078e).
// Its logic now lives in rs_shada_apply_entry (Rust) which calls the
// compound accessors nvim_shada_apply_search_pattern, nvim_shada_apply_sub_string,
// nvim_shada_apply_register, nvim_shada_apply_variable, nvim_shada_apply_mark_or_jump,
// nvim_shada_apply_buffer_list, and nvim_shada_apply_local_or_change.

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

/// Create a string-backed packer buffer. Writes the resulting buffer to *out.
void nvim_shada_packer_string_buffer(PackerBuffer *out)
{
  if (out) {
    *out = packer_string_buffer();
  }
}

/// Take the packed string from a string-backed packer buffer.
/// Returns the String value and zeroes out the buffer.
String nvim_shada_packer_take_string(PackerBuffer *buf)
{
  return buf ? packer_take_string(buf) : (String)STRING_INIT;
}

/// Wrapper for encode_vim_to_msgpack (for encoding typval_T variables).
int nvim_encode_vim_to_msgpack(PackerBuffer *packer, void *tv, const char *desc)
{
  return encode_vim_to_msgpack(packer, (typval_T *)tv, desc);
}

/// Get cstr_as_string equivalent: returns {s, strlen(s)} or STRING_INIT if NULL.
String nvim_shada_cstr_as_string(const char *s)
{
  return s ? cstr_as_string((char *)s) : (String)STRING_INIT;
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

/// Get number of items in a header Dict.
size_t nvim_shada_header_size(const ShadaEntry *entry)
{
  return entry ? entry->data.header.size : 0;
}

/// Get key data pointer for header item i.
const char *nvim_shada_header_item_key_data(const ShadaEntry *entry, size_t i)
{
  return (entry && i < entry->data.header.size) ? entry->data.header.items[i].key.data : NULL;
}

/// Get key size for header item i.
size_t nvim_shada_header_item_key_size(const ShadaEntry *entry, size_t i)
{
  return (entry && i < entry->data.header.size) ? entry->data.header.items[i].key.size : 0;
}

/// Get object type for header item i.
int nvim_shada_header_item_value_type(const ShadaEntry *entry, size_t i)
{
  return (entry && i < entry->data.header.size) ? (int)entry->data.header.items[i].value.type : 0;
}

/// Get string data pointer for a string-typed header item i.
const char *nvim_shada_header_item_value_str_data(const ShadaEntry *entry, size_t i)
{
  return (entry && i < entry->data.header.size) ? entry->data.header.items[i].value.data.string.data : NULL;
}

/// Get string size for a string-typed header item i.
size_t nvim_shada_header_item_value_str_size(const ShadaEntry *entry, size_t i)
{
  return (entry && i < entry->data.header.size) ? entry->data.header.items[i].value.data.string.size : 0;
}

/// Get integer value for an integer-typed header item i.
int64_t nvim_shada_header_item_value_integer(const ShadaEntry *entry, size_t i)
{
  return (entry && i < entry->data.header.size) ? entry->data.header.items[i].value.data.integer : 0;
}

/// Get data pointer for register contents[i] (a String struct's .data).
const char *nvim_shada_reg_contents_data(const ShadaEntry *entry, size_t i)
{
  if (!entry || i >= entry->data.reg.contents_size || !entry->data.reg.contents) {
    return NULL;
  }
  return entry->data.reg.contents[i].data;
}

/// Get size for register contents[i] (a String struct's .size).
size_t nvim_shada_reg_contents_size(const ShadaEntry *entry, size_t i)
{
  if (!entry || i >= entry->data.reg.contents_size || !entry->data.reg.contents) {
    return 0;
  }
  return entry->data.reg.contents[i].size;
}

/// Get the number of register contents entries.
size_t nvim_shada_reg_contents_count(const ShadaEntry *entry)
{
  return entry ? entry->data.reg.contents_size : 0;
}

/// Get the anyint error field from a packer buffer (non-zero means error).
int64_t nvim_shada_packer_get_anyint(PackerBuffer *packer)
{
  return packer ? packer->anyint : 0;
}

// Search pattern field accessors (Dict(_shada_search_pat) has OptionalKeys prefix)
bool nvim_shada_sp_get_magic(const ShadaEntry *e) { return e->data.search_pattern.magic; }
bool nvim_shada_sp_get_smartcase(const ShadaEntry *e) { return e->data.search_pattern.smartcase; }
bool nvim_shada_sp_get_has_line_offset(const ShadaEntry *e) { return e->data.search_pattern.has_line_offset; }
bool nvim_shada_sp_get_place_cursor_at_end(const ShadaEntry *e) { return e->data.search_pattern.place_cursor_at_end; }
bool nvim_shada_sp_get_is_last_used(const ShadaEntry *e) { return e->data.search_pattern.is_last_used; }
bool nvim_shada_sp_get_is_substitute_pattern(const ShadaEntry *e) { return e->data.search_pattern.is_substitute_pattern; }
bool nvim_shada_sp_get_highlighted(const ShadaEntry *e) { return e->data.search_pattern.highlighted; }
bool nvim_shada_sp_get_search_backward(const ShadaEntry *e) { return e->data.search_pattern.search_backward; }
int64_t nvim_shada_sp_get_offset(const ShadaEntry *e) { return e->data.search_pattern.offset; }
const char *nvim_shada_sp_get_pat_data(const ShadaEntry *e) { return e->data.search_pattern.pat.data; }
size_t nvim_shada_sp_get_pat_size(const ShadaEntry *e) { return e->data.search_pattern.pat.size; }

// Filemark field accessors (pos_T uses linenr_T=int32 but Rust Position.lnum is i64)
int64_t nvim_shada_fm_get_lnum(const ShadaEntry *e) { return (int64_t)e->data.filemark.mark.lnum; }
int32_t nvim_shada_fm_get_col(const ShadaEntry *e) { return (int32_t)e->data.filemark.mark.col; }
char nvim_shada_fm_get_name(const ShadaEntry *e) { return e->data.filemark.name; }
const char *nvim_shada_fm_get_fname(const ShadaEntry *e) { return e->data.filemark.fname; }

// Register field accessors (MotionType enum and String* layout differ from Rust)
int32_t nvim_shada_reg_get_type(const ShadaEntry *e) { return (int32_t)e->data.reg.type; }
char nvim_shada_reg_get_name(const ShadaEntry *e) { return e->data.reg.name; }
bool nvim_shada_reg_get_is_unnamed(const ShadaEntry *e) { return e->data.reg.is_unnamed; }
size_t nvim_shada_reg_get_width(const ShadaEntry *e) { return e->data.reg.width; }

// BufferList per-buffer position accessors
int64_t nvim_shada_bl_buf_get_lnum(const ShadaEntry *e, size_t i) { return (int64_t)e->data.buffer_list.buffers[i].pos.lnum; }
int32_t nvim_shada_bl_buf_get_col(const ShadaEntry *e, size_t i) { return (int32_t)e->data.buffer_list.buffers[i].pos.col; }
const char *nvim_shada_bl_buf_get_fname(const ShadaEntry *e, size_t i) { return e->data.buffer_list.buffers[i].fname; }
size_t nvim_shada_bl_buf_fname_size(const ShadaEntry *e, size_t i)
{
  const char *f = e->data.buffer_list.buffers[i].fname;
  return f ? strlen(f) : 0;
}
const void *nvim_shada_bl_buf_get_additional_data(const ShadaEntry *e, size_t i) { return e->data.buffer_list.buffers[i].additional_data; }
size_t nvim_shada_bl_get_size(const ShadaEntry *e) { return e->data.buffer_list.size; }

// UnknownItem field accessors (avoid Rust implicit autoref through union)
uint64_t nvim_shada_unknown_get_type_num(const ShadaEntry *e) { return e->data.unknown_item.type; }
const char *nvim_shada_unknown_get_contents(const ShadaEntry *e) { return e->data.unknown_item.contents; }
size_t nvim_shada_unknown_get_size(const ShadaEntry *e) { return e->data.unknown_item.size; }

// HistoryItem field accessors
uint8_t nvim_shada_hist_get_histtype(const ShadaEntry *e) { return e->data.history_item.histtype; }
const char *nvim_shada_hist_get_string(const ShadaEntry *e) { return e->data.history_item.string; }
char nvim_shada_hist_get_sep(const ShadaEntry *e) { return e->data.history_item.sep; }

// GlobalVar field accessors
const char *nvim_shada_gvar_get_name(const ShadaEntry *e) { return e->data.global_var.name; }

// SubString field accessors
const char *nvim_shada_sub_get_string(const ShadaEntry *e) { return e->data.sub_string.sub; }

// Global variable iteration accessor (wraps var_shada_iter).
// flavour is a bitmask of VAR_FLAVOUR_DEFAULT | VAR_FLAVOUR_SESSION | VAR_FLAVOUR_SHADA.
// On each call: *out_name set to variable name (static, do not free); *out_tv set to a
// freshly xmalloc'd typval_T copy (caller must pass to nvim_shada_pack_gvar_entry which frees it).
// Returns the next iter pointer (NULL when exhausted).
const void *nvim_shada_var_shada_iter(const void *iter, const char **out_name, void **out_tv,
                                      unsigned flavour)
{
  typval_T vartv;
  const char *name = NULL;
  const void *next = var_shada_iter(iter, &name, &vartv, (var_flavour_T)flavour);
  *out_name = name;
  if (name == NULL) {
    *out_tv = NULL;
    return next;
  }
  typval_T *tv_copy_ptr = xmalloc(sizeof(typval_T));
  *tv_copy_ptr = vartv;
  *out_tv = tv_copy_ptr;
  return next;
}

// Get the v_type field of a typval_T pointer.
int nvim_shada_tv_get_type(const void *tv)
{
  return tv ? ((const typval_T *)tv)->v_type : 0;
}

// Pack a global variable entry via rs_shada_pack_entry.
// Builds the ShadaEntry inline (C layout: inline typval_T), calls rs_shada_pack_entry,
// then clears and frees tv (which was xmalloc'd by nvim_shada_var_shada_iter).
// Returns SD_WRITE_SUCCESSFUL (1), SD_WRITE_FAILED (0), or SD_WRITE_IGN_ERROR (-1).
int nvim_shada_pack_gvar_entry(PackerBuffer *packer, const char *name, void *tv,
                               Timestamp ts)
{
  typval_T tgttv;
  tv_copy((typval_T *)tv, &tgttv);
  tv_clear((typval_T *)tv);
  xfree(tv);
  ShadaEntry entry = {
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
  int r = rs_shada_pack_entry(packer, &entry, 0);
  tv_clear(&entry.data.global_var.value);
  return r;
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

/// Allocate a new FileMarks and zero-initialize it.
void *nvim_shada_file_marks_alloc(void)
{
  return xcalloc(1, sizeof(FileMarks));
}

/// Get the greatest_timestamp from a FileMarks.
uint64_t nvim_shada_file_marks_greatest_ts(const void *fm_ptr)
{
  return fm_ptr ? ((const FileMarks *)fm_ptr)->greatest_timestamp : 0;
}

/// Set the greatest_timestamp in a FileMarks if ts is larger.
void nvim_shada_file_marks_update_ts(void *fm_ptr, uint64_t ts)
{
  if (fm_ptr && ts > ((FileMarks *)fm_ptr)->greatest_timestamp) {
    ((FileMarks *)fm_ptr)->greatest_timestamp = ts;
  }
}

/// Get a specific local mark entry from a FileMarks.
ShadaEntry *nvim_shada_file_marks_get_mark(void *fm_ptr, int idx)
{
  if (!fm_ptr || idx < 0 || idx >= NLOCALMARKS) {
    return NULL;
  }
  return &((FileMarks *)fm_ptr)->marks[idx];
}

/// Get a specific change entry from a FileMarks.
ShadaEntry *nvim_shada_file_marks_get_change(void *fm_ptr, int idx)
{
  if (!fm_ptr || idx < 0 || idx >= JUMPLISTSIZE) {
    return NULL;
  }
  return &((FileMarks *)fm_ptr)->changes[idx];
}

/// Get changes_size from FileMarks.
size_t nvim_shada_file_marks_changes_size(const void *fm_ptr)
{
  return fm_ptr ? ((const FileMarks *)fm_ptr)->changes_size : 0;
}

/// Set changes_size in FileMarks.
void nvim_shada_file_marks_set_changes_size(void *fm_ptr, size_t size)
{
  if (fm_ptr) {
    ((FileMarks *)fm_ptr)->changes_size = size;
  }
}

/// Get additional_marks_size from FileMarks.
size_t nvim_shada_file_marks_additional_size(const void *fm_ptr)
{
  return fm_ptr ? ((const FileMarks *)fm_ptr)->additional_marks_size : 0;
}

/// Get pointer to an additional mark entry (for packing then freeing).
ShadaEntry *nvim_shada_file_marks_get_additional(void *fm_ptr, size_t idx)
{
  if (!fm_ptr) {
    return NULL;
  }
  FileMarks *fm = (FileMarks *)fm_ptr;
  if (idx >= fm->additional_marks_size) {
    return NULL;
  }
  return &fm->additional_marks[idx];
}

/// Free the additional_marks array of a FileMarks.
void nvim_shada_file_marks_free_additional(void *fm_ptr)
{
  if (fm_ptr) {
    FileMarks *fm = (FileMarks *)fm_ptr;
    xfree(fm->additional_marks);
    fm->additional_marks = NULL;
    fm->additional_marks_size = 0;
  }
}

/// Grow the additional_marks array of a FileMarks and set the last entry.
void nvim_shada_file_marks_push_additional(void *fm_ptr, const ShadaEntry *entry)
{
  if (!fm_ptr || !entry) {
    return;
  }
  FileMarks *fm = (FileMarks *)fm_ptr;
  fm->additional_marks = xrealloc(fm->additional_marks,
                                  (++fm->additional_marks_size
                                   * sizeof(fm->additional_marks[0])));
  fm->additional_marks[fm->additional_marks_size - 1] = *entry;
}

/// Get the size of the file_marks PMap in the WriteMergerState.
size_t nvim_shada_wms_file_marks_size(const void *wms_opaque)
{
  const WriteMergerState *wms = (const WriteMergerState *)wms_opaque;
  return wms ? map_size(&wms->file_marks) : 0;
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

/// Get the hmll.size of hms[i] (to check if history merging is active).
size_t nvim_shada_wms_hms_size(const void *wms_opaque, int i)
{
  const WriteMergerState *wms = (const WriteMergerState *)wms_opaque;
  if (!wms || i < 0 || i >= HIST_COUNT) {
    return 0;
  }
  return wms->hms[i].hmll.size;
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

/// Get the xfree function pointer for use by Rust (wraps xfree).
void nvim_shada_xfree_key(void *key)
{
  xfree(key);
}

/// Allocate and initialize a WriteMergerState on the heap (returns void*).
void *nvim_shada_wms_alloc(void)
{
  return xcalloc(1, sizeof(WriteMergerState));
}

/// Free a WriteMergerState without destroying PMap/Set fields (those must be
/// destroyed separately via nvim_shada_wms_file_marks_destroy and
/// nvim_shada_wms_dumped_vars_destroy).
void nvim_shada_wms_free(void *wms)
{
  xfree(wms);
}

/// Flush the packer buffer.
void nvim_shada_packer_flush_buf(PackerBuffer *packer)
{
  if (packer && packer->packer_flush) {
    packer->packer_flush(packer);
  }
}

/// Internal flush callback for file-backed PackerBuffers.
static void nvim_shada_flush_file_buffer_(PackerBuffer *buffer)
{
  FileDescriptor *fd = buffer->anydata;
  fd->write_pos = buffer->ptr;
  buffer->anyint = file_flush(fd);
  buffer->ptr = fd->write_pos;
}

/// Create a PackerBuffer for writing to a FileDescriptor.
PackerBuffer nvim_shada_packer_buffer_for_file(void *fd)
{
  FileDescriptor *file = (FileDescriptor *)fd;
  if (file_space(file) < SHADA_MPACK_FREE_SPACE) {
    file_flush(file);
  }
  return (PackerBuffer) {
    .startptr = file->buffer,
    .ptr = file->write_pos,
    .endptr = file->buffer + ARENA_BLOCK_SIZE,
    .anydata = file,
    .anyint = 0,
    .packer_flush = nvim_shada_flush_file_buffer_,
  };
}

/// Initialize a PackerBuffer for writing to a FileDescriptor (by pointer).
/// This is the preferred accessor for Rust which cannot use by-value struct returns.
/// @param fd     FileDescriptor to write to.
/// @param out    Output: initialized PackerBuffer.
void nvim_shada_packer_init_for_file(void *fd, PackerBuffer *out)
{
  *out = nvim_shada_packer_buffer_for_file(fd);
}

// =============================================================================
// Phase 3 (plan fd426e0f): nvim_shada_pack_all_gvars migration accessors
// =============================================================================

/// Get the dv_hashtab pointer from a VAR_DICT typval (for rs_set_ref_in_ht).
/// @param tv  xmalloc'd typval_T pointer with v_type == VAR_DICT.
/// @return    pointer to tv->vval.v_dict->dv_hashtab, or NULL if dict is NULL.
void *nvim_shada_tv_get_dict_ht(const void *tv)
{
  const typval_T *t = (const typval_T *)tv;
  if (!t || !t->vval.v_dict) {
    return NULL;
  }
  return &t->vval.v_dict->dv_hashtab;
}

/// Get the dv_copyID from a VAR_DICT typval.
/// @param tv  xmalloc'd typval_T pointer with v_type == VAR_DICT.
/// @return    dict->dv_copyID, or 0 if dict is NULL.
int nvim_shada_tv_get_dict_copyid(const void *tv)
{
  const typval_T *t = (const typval_T *)tv;
  if (!t || !t->vval.v_dict) {
    return 0;
  }
  return t->vval.v_dict->dv_copyID;
}

/// Get the list_T pointer from a VAR_LIST typval (for rs_set_ref_in_list_items).
/// @param tv  xmalloc'd typval_T pointer with v_type == VAR_LIST.
/// @return    tv->vval.v_list pointer, or NULL.
void *nvim_shada_tv_get_list(const void *tv)
{
  const typval_T *t = (const typval_T *)tv;
  if (!t) {
    return NULL;
  }
  return t->vval.v_list;
}

/// Get the lv_copyID from a VAR_LIST typval.
/// @param tv  xmalloc'd typval_T pointer with v_type == VAR_LIST.
/// @return    list->lv_copyID, or 0 if list is NULL.
int nvim_shada_tv_get_list_copyid(const void *tv)
{
  const typval_T *t = (const typval_T *)tv;
  if (!t || !t->vval.v_list) {
    return 0;
  }
  return t->vval.v_list->lv_copyID;
}

// =============================================================================
// Phase 1 (plan 92c8078e): compound accessor functions for rs_shada_apply_entry
// =============================================================================

/// Apply a search pattern entry.
/// @param entry   The ShaDa entry (kSDItemSearchPattern).
/// @param force   Whether to force overwrite newer entries.
/// @return 1 if memory was consumed (do not free), 0 if entry was freed.
int nvim_shada_apply_search_pattern(ShadaEntry *entry, bool force)
{
  if (!force) {
    SearchPattern pat;
    if (entry->data.search_pattern.is_substitute_pattern) {
      get_substitute_pattern(&pat);
    } else {
      get_search_pattern(&pat);
    }
    if (pat.pat != NULL && pat.timestamp >= entry->timestamp) {
      rs_shada_free_entry_contents(entry);
      return 0;
    }
  }
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
  if (entry->data.search_pattern.is_substitute_pattern) {
    set_substitute_pattern(spat);
  } else {
    set_search_pattern(spat);
  }
  if (entry->data.search_pattern.is_last_used) {
    set_last_used_pattern(entry->data.search_pattern.is_substitute_pattern);
    set_no_hlsearch(!entry->data.search_pattern.highlighted);
  }
  // Do not free: allocated memory was saved above.
  return 1;
}

/// Apply a substitute string entry.
/// @param entry   The ShaDa entry (kSDItemSubString).
/// @param force   Whether to force overwrite newer entries.
/// @return 1 if memory was consumed (do not free), 0 if entry was freed.
int nvim_shada_apply_sub_string(ShadaEntry *entry, bool force)
{
  if (!force) {
    SubReplacementString sub;
    sub_get_replacement(&sub);
    if (sub.sub != NULL && sub.timestamp >= entry->timestamp) {
      rs_shada_free_entry_contents(entry);
      return 0;
    }
  }
  sub_set_replacement((SubReplacementString) {
    .sub = entry->data.sub_string.sub,
    .timestamp = entry->timestamp,
    .additional_data = entry->additional_data,
  });
  regtilde(entry->data.sub_string.sub, rs_magic_isset(), false);
  // Do not free: allocated memory was saved above.
  return 1;
}

/// Apply a register entry.
/// @param entry   The ShaDa entry (kSDItemRegister).
/// @param force   Whether to force overwrite newer entries.
/// @return 1 if memory was consumed by op_reg_set, 0 if entry was freed.
int nvim_shada_apply_register(ShadaEntry *entry, bool force)
{
  if (entry->data.reg.type != kMTCharWise
      && entry->data.reg.type != kMTLineWise
      && entry->data.reg.type != kMTBlockWise) {
    rs_shada_free_entry_contents(entry);
    return 0;
  }
  if (!force) {
    const yankreg_T *const reg = op_reg_get(entry->data.reg.name);
    if (reg == NULL || reg->timestamp >= entry->timestamp) {
      rs_shada_free_entry_contents(entry);
      return 0;
    }
  }
  if (!op_reg_set(entry->data.reg.name, (yankreg_T) {
    .y_array = entry->data.reg.contents,
    .y_size = entry->data.reg.contents_size,
    .y_type = entry->data.reg.type,
    .y_width = (colnr_T)entry->data.reg.width,
    .timestamp = entry->timestamp,
    .additional_data = entry->additional_data,
  }, entry->data.reg.is_unnamed)) {
    rs_shada_free_entry_contents(entry);
    return 0;
  }
  return 1;
}

/// Apply a global variable entry.
/// @param entry   The ShaDa entry (kSDItemVariable).
void nvim_shada_apply_variable(ShadaEntry *entry)
{
  var_set_global(entry->data.global_var.name, entry->data.global_var.value);
  entry->data.global_var.value.v_type = VAR_UNKNOWN;
  rs_shada_free_entry_contents(entry);
}

/// Apply a global mark or jump entry.
/// @param entry             The ShaDa entry (kSDItemGlobalMark or kSDItemJump).
/// @param fname_bufs_handle Opaque PMap(cstr_t) for buffer caching.
/// @param force             Whether to force overwrite.
/// @return 0 normally.
int nvim_shada_apply_mark_or_jump(ShadaEntry *entry, void *fname_bufs_handle, bool force)
{
  PMap(cstr_t) *fname_bufs = (PMap(cstr_t) *)fname_bufs_handle;
  buf_T *buf = find_buffer(fname_bufs, entry->data.filemark.fname);
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
  if (entry->type == kSDItemGlobalMark) {
    if (!mark_set_global(entry->data.filemark.name, fm, !force)) {
      rs_shada_free_entry_contents(entry);
    }
  } else {
    int i;
    for (i = curwin->w_jumplistlen; i > 0; i--) {
      const xfmark_T jl_entry = curwin->w_jumplist[i - 1];
      if (jl_entry.fmark.timestamp <= entry->timestamp) {
        if (rs_marks_equal(jl_entry.fmark.mark, entry->data.filemark.mark) != 0
            && (buf == NULL
                ? (jl_entry.fname != NULL && strcmp(fm.fname, jl_entry.fname) == 0)
                : fm.fmark.fnum == jl_entry.fmark.fnum)) {
          i = -1;
        }
        break;
      }
    }
    if (i > 0 && curwin->w_jumplistlen == JUMPLISTSIZE) {
      free_xfmark(curwin->w_jumplist[0]);
    }
    i = rs_marklist_insert(curwin->w_jumplist, sizeof(*curwin->w_jumplist),
                           curwin->w_jumplistlen, i);
    if (i != -1) {
      curwin->w_jumplist[i] = fm;
      if (curwin->w_jumplistlen < JUMPLISTSIZE) {
        curwin->w_jumplistlen++;
      }
      if (curwin->w_jumplistidx >= i
          && curwin->w_jumplistidx + 1 <= curwin->w_jumplistlen) {
        curwin->w_jumplistidx++;
      }
    } else {
      rs_shada_free_entry_contents(entry);
    }
  }
  return 0;
}

/// Apply a buffer list entry.
/// @param entry   The ShaDa entry (kSDItemBufferList).
void nvim_shada_apply_buffer_list(ShadaEntry *entry)
{
  for (size_t i = 0; i < entry->data.buffer_list.size; i++) {
    char *const sfname =
      path_try_shorten_fname(entry->data.buffer_list.buffers[i].fname);
    buf_T *const buf =
      buflist_new(entry->data.buffer_list.buffers[i].fname, sfname, 0, BLN_LISTED);
    if (buf != NULL) {
      fmarkv_T view = INIT_FMARKV;
      RESET_FMARK(&buf->b_last_cursor,
                  entry->data.buffer_list.buffers[i].pos, 0, view);
      buflist_setfpos(buf, curwin, buf->b_last_cursor.mark.lnum,
                      buf->b_last_cursor.mark.col, false);
      xfree(buf->additional_data);
      buf->additional_data = entry->data.buffer_list.buffers[i].additional_data;
      entry->data.buffer_list.buffers[i].additional_data = NULL;
    }
  }
  rs_shada_free_entry_contents(entry);
}

/// Apply a local mark or change entry.
/// @param entry             The ShaDa entry (kSDItemLocalMark or kSDItemChange).
/// @param fname_bufs_handle Opaque PMap(cstr_t) for buffer caching.
/// @param cl_bufs_handle    Opaque Set(ptr_t) for changelist bufs.
/// @param oldfiles_set_handle Opaque Set(cstr_t) for oldfiles dedup.
/// @param oldfiles_list     Opaque list_T for v:oldfiles.
/// @param force             Whether to force overwrite.
/// @param want_marks        Whether marks should be read.
/// @param get_old_files     Whether oldfiles should be gathered.
/// @return 0 normally.
int nvim_shada_apply_local_or_change(ShadaEntry *entry,
                                     void *fname_bufs_handle, void *cl_bufs_handle,
                                     void *oldfiles_set_handle, void *oldfiles_list,
                                     bool force, bool want_marks, bool get_old_files)
{
  PMap(cstr_t) *fname_bufs = (PMap(cstr_t) *)fname_bufs_handle;
  Set(ptr_t) *cl_bufs = (Set(ptr_t) *)cl_bufs_handle;
  Set(cstr_t) *oldfiles_set = (Set(cstr_t) *)oldfiles_set_handle;
  list_T *old_list = (list_T *)oldfiles_list;

  if (get_old_files
      && !set_has(cstr_t, oldfiles_set, entry->data.filemark.fname)) {
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
  if (!want_marks) {
    rs_shada_free_entry_contents(entry);
    return 0;
  }
  buf_T *buf = find_buffer(fname_bufs, entry->data.filemark.fname);
  if (buf == NULL) {
    rs_shada_free_entry_contents(entry);
    return 0;
  }
  const fmark_T fm = (fmark_T) {
    .mark = entry->data.filemark.mark,
    .fnum = 0,
    .timestamp = entry->timestamp,
    .view = INIT_FMARKV,
    .additional_data = entry->additional_data,
  };
  if (entry->type == kSDItemLocalMark) {
    if (!mark_set_local(entry->data.filemark.name, buf, fm, !force)) {
      rs_shada_free_entry_contents(entry);
      return 0;
    }
  } else {
    set_put(ptr_t, cl_bufs, buf);
    int i;
    for (i = buf->b_changelistlen; i > 0; i--) {
      const fmark_T jl_entry = buf->b_changelist[i - 1];
      if (jl_entry.timestamp <= entry->timestamp) {
        if (rs_marks_equal(jl_entry.mark, entry->data.filemark.mark) != 0) {
          i = -1;
        }
        break;
      }
    }
    if (i > 0 && buf->b_changelistlen == JUMPLISTSIZE) {
      free_fmark(buf->b_changelist[0]);
    }
    i = rs_marklist_insert(buf->b_changelist, sizeof(*buf->b_changelist),
                           buf->b_changelistlen, i);
    if (i != -1) {
      buf->b_changelist[i] = fm;
      if (buf->b_changelistlen < JUMPLISTSIZE) {
        buf->b_changelistlen++;
      }
    } else {
      xfree(fm.additional_data);
    }
  }
  xfree(entry->data.filemark.fname);
  return 0;
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

/// Set entry->data to the default values for the given type from sd_default_values.
void nvim_shada_set_entry_default_data(ShadaEntry *entry, uint64_t type_u64)
{
  entry->data = sd_default_values[type_u64].data;
}

/// Verify that the buffer contains a valid msgpack value (for verify_but_ignore path).
/// Returns kSDReadStatusSuccess or kSDReadStatusNotShaDa.
int nvim_shada_verify_skip(const char *buf, size_t size, uint64_t parse_pos)
{
  size_t read_size = size;
  int status = unpack_skip(&buf, &read_size);
  // Inlined from shada_check_status (deleted in Phase 2 plan 92c8078e)
  switch (status) {
  case MPACK_OK:
    if (read_size) {
      semsg(_(RCERR "Failed to parse ShaDa file: extra bytes in msgpack string "
              "at position %" PRIu64),
            (uint64_t)parse_pos);
      return (int)kSDReadStatusNotShaDa;
    }
    return (int)kSDReadStatusSuccess;
  case MPACK_EOF:
    semsg(_(RCERR "Failed to parse ShaDa file: incomplete msgpack string "
            "at position %" PRIu64),
          (uint64_t)parse_pos);
    return (int)kSDReadStatusNotShaDa;
  default:
    semsg(_(RCERR "Failed to parse ShaDa file due to a msgpack parser error "
            "at position %" PRIu64),
          (uint64_t)parse_pos);
    return (int)kSDReadStatusNotShaDa;
  }
}

/// Set entry as an unknown item, copying buf if needed.
/// buf_allocated: if true, caller owns buf; if false, a copy is made.
/// For unknown items at initial_fpos==0, also verifies the buffer is valid msgpack.
/// Returns kSDReadStatusSuccess on success, kSDReadStatusNotShaDa on parse failure.
int nvim_shada_set_unknown_item(ShadaEntry *entry, uint64_t type_u64, char *buf,
                                size_t length, bool buf_allocated,
                                const char *read_ptr, size_t read_size,
                                uint64_t initial_fpos, uint64_t parse_pos)
{
  entry->type = kSDItemUnknown;
  entry->data.unknown_item.size = length;
  entry->data.unknown_item.type = type_u64;
  if (initial_fpos == 0) {
    // Verify parse_pos is valid msgpack (inlined shada_check_status, deleted Phase 2 plan 92c8078e)
    int status = unpack_skip(&read_ptr, &read_size);
    ShaDaReadResult spm_ret;
    switch (status) {
    case MPACK_OK:
      spm_ret = read_size ? kSDReadStatusNotShaDa : kSDReadStatusSuccess;
      if (read_size) {
        semsg(_(RCERR "Failed to parse ShaDa file: extra bytes in msgpack string "
                "at position %" PRIu64),
              (uint64_t)parse_pos);
      }
      break;
    case MPACK_EOF:
      semsg(_(RCERR "Failed to parse ShaDa file: incomplete msgpack string "
              "at position %" PRIu64),
            (uint64_t)parse_pos);
      spm_ret = kSDReadStatusNotShaDa;
      break;
    default:
      semsg(_(RCERR "Failed to parse ShaDa file due to a msgpack parser error "
              "at position %" PRIu64),
            (uint64_t)parse_pos);
      spm_ret = kSDReadStatusNotShaDa;
      break;
    }
    if (spm_ret != kSDReadStatusSuccess) {
      if (buf_allocated) {
        xfree(buf);
      }
      entry->type = kSDItemMissing;
      return (int)spm_ret;
    }
  }
  entry->data.unknown_item.contents = buf_allocated ? buf : xmemdup(buf, length);
  return (int)kSDReadStatusSuccess;
}

/// Parse a SearchPattern entry body from a msgpack buffer.
/// Populates entry->data.search_pattern. Returns kSDReadStatusSuccess,
/// kSDReadStatusNotShaDa, or kSDReadStatusMalformed on error.
int nvim_shada_parse_search_pattern(ShadaEntry *entry, const char *buf,
                                    size_t size, uint64_t initial_fpos)
{
  size_t read_size = size;
  char *error_alloc = NULL;
  Dict(_shada_search_pat) *it = &entry->data.search_pattern;
  if (!unpack_keydict(it, DictHash(_shada_search_pat), NULL, &buf, &read_size, &error_alloc)) {
    semsg(_(READERR("search pattern", "%s")), initial_fpos, error_alloc);
    xfree(error_alloc);
    it->pat = NULL_STRING;
    return (int)kSDReadStatusMalformed;
  }
  if (!HAS_KEY(it, _shada_search_pat, sp)) {
    semsg(_(READERR("search pattern", "has no pattern")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  entry->data.search_pattern.pat = copy_string(entry->data.search_pattern.pat, NULL);
  return (int)kSDReadStatusSuccess;
}

/// Parse a Mark/Jump/GlobalMark/Change/LocalMark entry body from a msgpack buffer.
/// Populates entry->data.filemark. Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
int nvim_shada_parse_mark(ShadaEntry *entry, const char *buf, size_t size,
                          uint64_t initial_fpos, uint64_t type_u64)
{
  size_t read_size = size;
  char *error_alloc = NULL;
  Dict(_shada_mark) it = { 0 };
  if (!unpack_keydict(&it, DictHash(_shada_mark), NULL, &buf, &read_size, &error_alloc)) {
    semsg(_(READERR("mark", "%s")), initial_fpos, error_alloc);
    xfree(error_alloc);
    return (int)kSDReadStatusMalformed;
  }

  if (HAS_KEY(&it, _shada_mark, n)) {
    if (type_u64 == kSDItemJump || type_u64 == kSDItemChange) {
      semsg(_(READERR("mark", "has n key which is only valid for "
                      "local and global mark entries")), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
    entry->data.filemark.name = (char)it.n;
  }

  if (HAS_KEY(&it, _shada_mark, l)) {
    entry->data.filemark.mark.lnum = (linenr_T)it.l;
  }
  if (HAS_KEY(&it, _shada_mark, c)) {
    entry->data.filemark.mark.col = (colnr_T)it.c;
  }
  if (HAS_KEY(&it, _shada_mark, f)) {
    entry->data.filemark.fname = xmemdupz(it.f.data, it.f.size);
  }

  if (entry->data.filemark.fname == NULL) {
    semsg(_(READERR("mark", "is missing file name")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  if (entry->data.filemark.mark.lnum <= 0) {
    semsg(_(READERR("mark", "has invalid line number")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  if (entry->data.filemark.mark.col < 0) {
    semsg(_(READERR("mark", "has invalid column number")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  return (int)kSDReadStatusSuccess;
}

/// Parse a Register entry body from a msgpack buffer.
/// Populates entry->data.reg. Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
int nvim_shada_parse_register(ShadaEntry *entry, const char *buf, size_t size,
                              uint64_t initial_fpos)
{
  size_t read_size = size;
  char *error_alloc = NULL;
  Dict(_shada_register) it = { 0 };
  if (!unpack_keydict(&it, DictHash(_shada_register), NULL, &buf, &read_size, &error_alloc)) {
    semsg(_(READERR("register", "%s")), initial_fpos, error_alloc);
    xfree(error_alloc);
    kv_destroy(it.rc);
    return (int)kSDReadStatusMalformed;
  }
  if (it.rc.size == 0) {
    semsg(_(READERR("register",
                    "has " KEY_NAME(REG_KEY_CONTENTS) " key with missing or empty array")),
          initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  entry->data.reg.contents_size = it.rc.size;
  entry->data.reg.contents = xmalloc(it.rc.size * sizeof(String));
  for (size_t j = 0; j < it.rc.size; j++) {
    entry->data.reg.contents[j] = copy_string(it.rc.items[j], NULL);
  }
  kv_destroy(it.rc);

#define REGISTER_VAL(name, loc, type) \
  if (HAS_KEY(&it, _shada_register, name)) { \
    loc = (type)it.name; \
  }
  REGISTER_VAL(REG_KEY_UNNAMED, entry->data.reg.is_unnamed, bool)
  REGISTER_VAL(REG_KEY_TYPE, entry->data.reg.type, uint8_t)
  REGISTER_VAL(KEY_NAME_CHAR, entry->data.reg.name, char)
  REGISTER_VAL(REG_KEY_WIDTH, entry->data.reg.width, size_t)
#undef REGISTER_VAL
  return (int)kSDReadStatusSuccess;
}

/// Parse a HistoryEntry body from a msgpack buffer.
/// Populates entry->data.history_item and sets *num_additional to the number
/// of additional array elements remaining.
/// Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
int nvim_shada_parse_history(ShadaEntry *entry, const char *buf, size_t size,
                             uint64_t initial_fpos, uint32_t *num_additional,
                             const char **read_ptr_out, size_t *read_size_out)
{
  size_t read_size = size;
  ssize_t len = unpack_array(&buf, &read_size);
  if (len < 2) {
    semsg(_(READERR("history", "is not an array with enough elements")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  Integer hist_type;
  if (!unpack_integer(&buf, &read_size, &hist_type)) {
    semsg(_(READERR("history", "has wrong history type type")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  const String item = unpack_string(&buf, &read_size);
  if (!item.data) {
    semsg(_(READERR("history", "has wrong history string type")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  if (memchr(item.data, 0, item.size) != NULL) {
    semsg(_(READERR("history", "contains string with zero byte inside")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  entry->data.history_item.histtype = (uint8_t)hist_type;
  const bool is_hist_search = entry->data.history_item.histtype == HIST_SEARCH;
  if (is_hist_search) {
    if (len < 3) {
      semsg(_(READERR("search history", "does not have separator character")), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
    Integer sep_type;
    if (!unpack_integer(&buf, &read_size, &sep_type)) {
      semsg(_(READERR("search history", "has wrong history separator type")), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
    entry->data.history_item.sep = (char)sep_type;
  }
  size_t strsize = (item.size + 1 + 1);
  entry->data.history_item.string = xmalloc(strsize);
  memcpy(entry->data.history_item.string, item.data, item.size);
  entry->data.history_item.string[strsize - 2] = 0;
  entry->data.history_item.string[strsize - 1] = entry->data.history_item.sep;
  *num_additional = (uint32_t)(len - (2 + is_hist_search));
  *read_ptr_out = buf;
  *read_size_out = read_size;
  return (int)kSDReadStatusSuccess;
}

/// Parse a Variable entry body from a msgpack buffer.
/// Populates entry->data.global_var and sets *num_additional.
/// Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
int nvim_shada_parse_variable(ShadaEntry *entry, const char *buf, size_t size,
                              uint64_t initial_fpos, uint32_t *num_additional,
                              const char **read_ptr_out, size_t *read_size_out)
{
  size_t read_size = size;
  ssize_t len = unpack_array(&buf, &read_size);
  if (len < 2) {
    semsg(_(READERR("variable", "is not an array with enough elements")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  String name = unpack_string(&buf, &read_size);
  if (!name.data) {
    semsg(_(READERR("variable", "has wrong variable name type")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  entry->data.global_var.name = xmemdupz(name.data, name.size);

  String binval = unpack_string(&buf, &read_size);
  bool is_blob = false;
  if (binval.data) {
    if (len > 2) {
      Integer type;
      if (!unpack_integer(&buf, &read_size, &type) || type != VAR_TYPE_BLOB) {
        semsg(_(READERR("variable", "has wrong variable type")), initial_fpos);
        return (int)kSDReadStatusMalformed;
      }
      is_blob = true;
    }
    entry->data.global_var.value = decode_string(binval.data, binval.size, is_blob, false);
  } else {
    int status = unpack_typval(&buf, &read_size, &entry->data.global_var.value);
    if (status != MPACK_OK) {
      semsg(_(READERR("variable", "has value that cannot "
                      "be converted to the Vimscript value")), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
  }
  *num_additional = (uint32_t)(len - 2 - (is_blob ? 1 : 0));
  *read_ptr_out = buf;
  *read_size_out = read_size;
  return (int)kSDReadStatusSuccess;
}

/// Parse a SubString entry body from a msgpack buffer.
/// Populates entry->data.sub_string and sets *num_additional.
/// Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
int nvim_shada_parse_substr(ShadaEntry *entry, const char *buf, size_t size,
                            uint64_t initial_fpos, uint32_t *num_additional,
                            const char **read_ptr_out, size_t *read_size_out)
{
  size_t read_size = size;
  ssize_t len = unpack_array(&buf, &read_size);
  if (len < 1) {
    semsg(_(READERR("sub string", "is not an array with enough elements")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  String sub = unpack_string(&buf, &read_size);
  if (!sub.data) {
    semsg(_(READERR("sub string", "has wrong sub string type")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  entry->data.sub_string.sub = xmemdupz(sub.data, sub.size);
  *num_additional = (uint32_t)(len - 1);
  *read_ptr_out = buf;
  *read_size_out = read_size;
  return (int)kSDReadStatusSuccess;
}

/// Parse a BufferList entry body from a msgpack buffer.
/// Populates entry->data.buffer_list.
/// Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
int nvim_shada_parse_buflist(ShadaEntry *entry, const char *buf, size_t size,
                             uint64_t initial_fpos)
{
  size_t read_size = size;
  char *error_alloc = NULL;
  ssize_t len = unpack_array(&buf, &read_size);
  if (len < 0) {
    semsg(_(READERR("buffer list", "is not an array")), initial_fpos);
    return (int)kSDReadStatusMalformed;
  }
  if (len == 0) {
    return (int)kSDReadStatusSuccess;
  }
  entry->data.buffer_list.buffers = xcalloc((size_t)len,
                                            sizeof(*entry->data.buffer_list.buffers));
  for (size_t i = 0; i < (size_t)len; i++) {
    entry->data.buffer_list.size++;
    Dict(_shada_buflist_item) it = { 0 };
    AdditionalDataBuilder it_ad = KV_INITIAL_VALUE;
    if (!unpack_keydict(&it, DictHash(_shada_buflist_item), &it_ad, &buf, &read_size,
                        &error_alloc)) {
      semsg(_(RERR "Error while reading ShaDa file: "
              "buffer list at position %" PRIu64 " contains entry that %s"),
            initial_fpos, error_alloc);
      xfree(error_alloc);
      kv_destroy(it_ad);
      return (int)kSDReadStatusMalformed;
    }
    struct buffer_list_buffer *e = &entry->data.buffer_list.buffers[i];
    e->additional_data = (AdditionalData *)it_ad.items;
    e->pos = default_pos;
    if (HAS_KEY(&it, _shada_buflist_item, l)) {
      e->pos.lnum = (linenr_T)it.l;
    }
    if (HAS_KEY(&it, _shada_buflist_item, c)) {
      e->pos.col = (colnr_T)it.c;
    }
    if (HAS_KEY(&it, _shada_buflist_item, f)) {
      e->fname = xmemdupz(it.f.data, it.f.size);
    }
    if (e->pos.lnum <= 0) {
      semsg(_(RERR "Error while reading ShaDa file: "
              "buffer list at position %" PRIu64 " "
              "contains entry with invalid line number"), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
    if (e->pos.col < 0) {
      semsg(_(RERR "Error while reading ShaDa file: "
              "buffer list at position %" PRIu64 " "
              "contains entry with invalid column number"), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
    if (e->fname == NULL) {
      semsg(_(RERR "Error while reading ShaDa file: "
              "buffer list at position %" PRIu64 " "
              "contains entry that does not have a file name"), initial_fpos);
      return (int)kSDReadStatusMalformed;
    }
  }
  return (int)kSDReadStatusSuccess;
}

/// Push additional data elements from read_ptr into entry->additional_data.
/// num_additional is the count of extra elements. read_size is remaining bytes.
/// Returns kSDReadStatusSuccess or kSDReadStatusMalformed.
/// On success, sets *remaining_size_out to remaining bytes after additional data.
int nvim_shada_parse_additional_data(ShadaEntry *entry, const char *read_ptr,
                                     size_t read_size, uint32_t num_additional,
                                     uint64_t initial_fpos)
{
  AdditionalDataBuilder ad = KV_INITIAL_VALUE;
  for (uint32_t i = 0; i < num_additional; i++) {
    const char *item_start = read_ptr;
    int status = unpack_skip(&read_ptr, &read_size);
    if (status) {
      kv_destroy(ad);
      return (int)kSDReadStatusMalformed;
    }
    push_additional_data(&ad, item_start, (size_t)(read_ptr - item_start));
  }
  if (read_size) {
    semsg(_(READERR("item", "additional bytes")), initial_fpos);
    kv_destroy(ad);
    return (int)kSDReadStatusMalformed;
  }
  entry->additional_data = (AdditionalData *)ad.items;
  return (int)kSDReadStatusSuccess;
}

/// Emit RCERR "too long" error message (used by rs_shada_read_next_item).
void nvim_shada_semsg_rcerr_too_long(uint64_t initial_fpos)
{
  semsg(_(RCERR "Error while reading ShaDa file: "
          "there is an item at position %" PRIu64 " "
          "that is stated to be too long"),
        initial_fpos);
}

/// Emit RCERR "missing item" error message (used by rs_shada_read_next_item).
void nvim_shada_semsg_rcerr_missing(uint64_t initial_fpos)
{
  semsg(_(RCERR "Error while reading ShaDa file: "
          "there is an item at position %" PRIu64 " "
          "that must not be there: Missing items are "
          "for internal uses only"),
        initial_fpos);
}

