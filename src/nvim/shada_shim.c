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
extern int rs_shada_hist_type2char(int hist_type);
extern int rs_shada_hist_char2type(int c);
extern int rs_get_shada_parameter(int typ);
extern const char *rs_find_shada_parameter(int typ);
extern int rs_shada_entry_type_valid(int entry_type);
extern const char *rs_shada_entry_type_name(int entry_type);
extern uint64_t rs_vim_be64toh(uint64_t big_endian_64_bits);
extern uint64_t rs_vim_htobe64(uint64_t host_64_bits);
extern int rs_shada_entry_type_from_raw(uint64_t raw_type);
extern int rs_shada_is_unknown_entry(uint64_t entry_type);
extern int rs_shada_should_write_entry(size_t packed_size, size_t max_kbyte);
extern int rs_marks_equal(pos_T a, pos_T b);
extern int rs_marklist_insert(void *jumps_arr, size_t jump_size, int jl_len, int i);
extern int rs_compare_file_marks(const void *a, const void *b);
extern int rs_shada_removable(const char *name);
extern int rs_ignore_buf(const void *buf, const void *removable_bufs);
extern void rs_find_removable_bufs(void *removable_bufs);
extern const void *rs_shada_hist_iter(const void *iter, uint8_t history_type,
                                      int zero, ShadaEntry *hist);
extern void rs_add_search_pattern(ShadaEntry *ret_pse, int is_substitute,
                                  int search_last_used, int search_highlighted);
extern ShadaEntry rs_shada_get_buflist(void *removable_bufs);
extern size_t rs_shada_init_jumps(ShadaEntry *jumps, void *removable_bufs);
extern void rs_close_file(void *cookie);
extern const char *rs_shada_get_default_file(void);
extern int rs_shada_read_file(const char *file, int flags);
extern int rs_shada_read_marks(void);
extern int rs_shada_read_everything(const char *fname, bool forceit, bool missing_ok);
extern void rs_check_marks_read(void);
extern var_flavour_T rs_var_flavour(const char *varname);
extern bool rs_set_ref_in_ht(hashtab_T *ht, int copyID, list_stack_T **list_stack);
extern bool rs_set_ref_in_list_items(list_T *l, int copyID, ht_stack_T **ht_stack);
extern int rs_get_copyID(void);

#define rs_hist_type2char rs_shada_hist_type2char

#ifdef HAVE_BE64TOH
# define _BSD_SOURCE 1  // NOLINT(bugprone-reserved-identifier)
# define _DEFAULT_SOURCE 1  // NOLINT(bugprone-reserved-identifier)
# include ENDIAN_INCLUDE_FILE
#endif

#define SEARCH_KEY_MAGIC sm
#define SEARCH_KEY_SMARTCASE sc
#define SEARCH_KEY_HAS_LINE_OFFSET sl
#define SEARCH_KEY_PLACE_CURSOR_AT_END se
#define SEARCH_KEY_IS_LAST_USED su
#define SEARCH_KEY_IS_SUBSTITUTE_PATTERN ss
#define SEARCH_KEY_HIGHLIGHTED sh
#define SEARCH_KEY_OFFSET so
#define SEARCH_KEY_PAT sp
#define SEARCH_KEY_BACKWARD sb

#define REG_KEY_TYPE rt
#define REG_KEY_WIDTH rw
#define REG_KEY_CONTENTS rc
#define REG_KEY_UNNAMED ru

#define KEY_LNUM l
#define KEY_COL c
#define KEY_FILE f
#define KEY_NAME_CHAR n

// Error messages formerly used by viminfo code:
//   E136: viminfo: Too many errors, skipping rest of file
//   E137: Viminfo file is not writable: %s
//   E138: Can't write viminfo file %s!
//   E195: Cannot open ShaDa file for reading
//   E574: Unknown register type %d
//   E575: Illegal starting char
//   E576: Missing '>'
//   E577: Illegal register name
//   E886: Can't rename viminfo file to %s!
//   E929: Too many viminfo temp files, like %s!
// Now only six of them are used:
//   E137: ShaDa file is not writeable (for pre-open checks)
//   E929: All %s.tmp.X files exist, cannot write ShaDa file!
//   RCERR (E576) for critical read errors.
//   RNERR (E136) for various errors when renaming.
//   RERR (E575) for various errors inside read ShaDa file.
//   SERR (E886) for various “system” errors (always contains output of
//   strerror)
//   WERR (E574) for various ignorable write errors

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

// Rust rs_* declarations (need HMLList, HMLListEntry, HistoryMergerState types)
extern void rs_hmll_init(HMLList *hmll, size_t size);
extern void rs_hmll_remove(HMLList *hmll, HMLListEntry *entry);
extern void rs_hmll_insert(HMLList *hmll, HMLListEntry *after, ShadaEntry data);
extern void rs_hmll_dealloc(HMLList *hmll);
extern void rs_hms_init(HistoryMergerState *hms_p, uint8_t history_type,
                        size_t num_elements, int do_merge, int reading);
extern void rs_hms_insert(HistoryMergerState *hms_p, ShadaEntry entry, int do_iter);
extern void rs_hms_insert_whole_neovim_history(HistoryMergerState *hms_p);
extern void rs_hms_dealloc(HistoryMergerState *hms_p);
extern void rs_hms_to_he_array(const HistoryMergerState *hms_p, void *hist_array,
                               int *new_hisidx, int *new_hisnum);

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

// Rust rs_* declarations (need WriteMergerState type)
extern void rs_replace_numbered_mark(WriteMergerState *wms, size_t idx, ShadaEntry entry);
extern void rs_shada_initialize_registers(WriteMergerState *wms, int max_reg_lines);

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

/// Iterate over HMLList in forward direction
///
/// @param  hmll       Pointer to the list.
/// @param  cur_entry  Name of the variable to iterate over.
/// @param  code       Code to execute on each iteration.
///
/// @return `for` cycle header (use `HMLL_FORALL(hmll, cur_entry) {body}`).
#define HMLL_FORALL(hmll, cur_entry, code) \
  for (HMLListEntry *(cur_entry) = (hmll)->first; (cur_entry) != NULL; \
       (cur_entry) = (cur_entry)->next) { \
    code \
  } \

/// Wrapper for read that can be used when lseek cannot be used
///
/// E.g. when trying to read from a pipe.
///
/// @param[in,out]  sd_reader  File read.
/// @param[in]      offset     Amount of bytes to skip.
///
/// @return kSDReadStatusReadError, kSDReadStatusNotShaDa or
///         kSDReadStatusSuccess.
static ShaDaReadResult sd_reader_skip(FileDescriptor *const sd_reader, const size_t offset)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_NONNULL_ALL
{
  const ptrdiff_t skip_bytes = file_skip(sd_reader, offset);
  if (skip_bytes < 0) {
    semsg(_(SERR "System error while skipping in ShaDa file: %s"), os_strerror((int)skip_bytes));
    return kSDReadStatusReadError;
  } else if (skip_bytes != (ptrdiff_t)offset) {
    assert(skip_bytes < (ptrdiff_t)offset);
    if (file_eof(sd_reader)) {
      semsg(_(RCERR "Reading ShaDa file: last entry specified that it occupies %" PRIu64 " bytes, "
              "but file ended earlier"),
            (uint64_t)offset);
    } else {
      semsg(_(SERR "System error while skipping in ShaDa file: %s"), _("too few bytes read"));
    }
    return kSDReadStatusNotShaDa;
  }
  return kSDReadStatusSuccess;
}

/// Iterate over all history entries in history merger, in order
///
/// @param[in]   hms_p      Merger structure to iterate over.
/// @param[out]  cur_entry  Name of the iterator variable.
/// @param       code       Code to execute on each iteration.
///
/// @return for cycle header. Use `HMS_ITER(hms_p, cur_entry) {body}`.
#define HMS_ITER(hms_p, cur_entry, code) \
  HMLL_FORALL(&((hms_p)->hmll), cur_entry, code)

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

/// Read data from ShaDa file
///
/// @param[in]  sd_reader  Structure containing file reader definition.
/// @param[in]  flags      What to read, see ShaDaReadFileFlags enum.
static void shada_read(FileDescriptor *const sd_reader, const int flags)
  FUNC_ATTR_NONNULL_ALL
{
  list_T *oldfiles_list = get_vim_var_list(VV_OLDFILES);
  const bool force = flags & kShaDaForceit;
  const bool get_old_files = (flags & (kShaDaGetOldfiles | kShaDaForceit)
                              && (force || tv_list_len(oldfiles_list) == 0));
  const bool want_marks = flags & kShaDaWantMarks;
  const unsigned srni_flags =
    (unsigned)(
               (flags & kShaDaWantInfo
                ? (kSDReadUndisableableData
                   | kSDReadRegisters
                   | kSDReadGlobalMarks
                   | (p_hi ? kSDReadHistory : 0)
                   | (find_shada_parameter('!') != NULL
                      ? kSDReadVariables
                      : 0)
                   | (find_shada_parameter('%') != NULL
                      && ARGCOUNT == 0
                      ? kSDReadBufferList
                      : 0))
                : 0)
               | (want_marks && get_shada_parameter('\'') > 0
                  ? kSDReadLocalMarks | kSDReadChanges
                  : 0)
               | (get_old_files
                  ? kSDReadLocalMarks
                  : 0));
  if (srni_flags == 0) {
    // Nothing to do.
    return;
  }
  HistoryMergerState hms[HIST_COUNT];
  if (srni_flags & kSDReadHistory) {
    for (int i = 0; i < HIST_COUNT; i++) {
      rs_hms_init(&hms[i], (uint8_t)i, (size_t)p_hi, 1, 1);
    }
  }
  ShadaEntry cur_entry;
  Set(ptr_t) cl_bufs = SET_INIT;
  PMap(cstr_t) fname_bufs = MAP_INIT;
  Set(cstr_t) oldfiles_set = SET_INIT;
  if (get_old_files && (oldfiles_list == NULL || force)) {
    oldfiles_list = tv_list_alloc(kListLenUnknown);
    set_vim_var_list(VV_OLDFILES, oldfiles_list);
  }
  ShaDaReadResult srni_ret;
  while ((srni_ret = shada_read_next_item(sd_reader, &cur_entry, srni_flags, 0))
         != kSDReadStatusFinished) {
    switch (srni_ret) {
    case kSDReadStatusSuccess:
      break;
    case kSDReadStatusFinished:
      // Should be handled by the while condition.
      abort();
    case kSDReadStatusNotShaDa:
    case kSDReadStatusReadError:
      goto shada_read_main_cycle_end;
    case kSDReadStatusMalformed:
      continue;
    }
    switch (cur_entry.type) {
    case kSDItemMissing:
      abort();
    case kSDItemUnknown:
      break;
    case kSDItemHeader:
      shada_free_shada_entry(&cur_entry);
      break;
    case kSDItemSearchPattern:
      if (!force) {
        SearchPattern pat;
        if (cur_entry.data.search_pattern.is_substitute_pattern) {
          get_substitute_pattern(&pat);
        } else {
          get_search_pattern(&pat);
        }
        if (pat.pat != NULL && pat.timestamp >= cur_entry.timestamp) {
          shada_free_shada_entry(&cur_entry);
          break;
        }
      }

      SearchPattern spat = (SearchPattern) {
        .magic = cur_entry.data.search_pattern.magic,
        .no_scs = !cur_entry.data.search_pattern.smartcase,
        .off = {
          .dir = cur_entry.data.search_pattern.search_backward ? '?' : '/',
          .line = cur_entry.data.search_pattern.has_line_offset,
          .end = cur_entry.data.search_pattern.place_cursor_at_end,
          .off = cur_entry.data.search_pattern.offset,
        },
        .pat = cur_entry.data.search_pattern.pat.data,
        .patlen = cur_entry.data.search_pattern.pat.size,
        .additional_data = cur_entry.additional_data,
        .timestamp = cur_entry.timestamp,
      };

      if (cur_entry.data.search_pattern.is_substitute_pattern) {
        set_substitute_pattern(spat);
      } else {
        set_search_pattern(spat);
      }

      if (cur_entry.data.search_pattern.is_last_used) {
        set_last_used_pattern(cur_entry.data.search_pattern.is_substitute_pattern);
        set_no_hlsearch(!cur_entry.data.search_pattern.highlighted);
      }
      // Do not free shada entry: its allocated memory was saved above.
      break;
    case kSDItemSubString:
      if (!force) {
        SubReplacementString sub;
        sub_get_replacement(&sub);
        if (sub.sub != NULL && sub.timestamp >= cur_entry.timestamp) {
          shada_free_shada_entry(&cur_entry);
          break;
        }
      }
      sub_set_replacement((SubReplacementString) {
        .sub = cur_entry.data.sub_string.sub,
        .timestamp = cur_entry.timestamp,
        .additional_data = cur_entry.additional_data,
      });
      // Without using regtilde and without / &cpo flag previous substitute
      // string is close to useless: you can only use it with :& or :~ and
      // that’s all because s//~ is not available until the first call to
      // regtilde. Vim was not calling this for some reason.
      regtilde(cur_entry.data.sub_string.sub, rs_magic_isset(), false);
      // Do not free shada entry: its allocated memory was saved above.
      break;
    case kSDItemHistoryEntry:
      if (cur_entry.data.history_item.histtype >= HIST_COUNT) {
        shada_free_shada_entry(&cur_entry);
        break;
      }
      rs_hms_insert(hms + cur_entry.data.history_item.histtype, cur_entry, 1);
      // Do not free shada entry: its allocated memory was saved above.
      break;
    case kSDItemRegister:
      if (cur_entry.data.reg.type != kMTCharWise
          && cur_entry.data.reg.type != kMTLineWise
          && cur_entry.data.reg.type != kMTBlockWise) {
        shada_free_shada_entry(&cur_entry);
        break;
      }
      if (!force) {
        const yankreg_T *const reg = op_reg_get(cur_entry.data.reg.name);
        if (reg == NULL || reg->timestamp >= cur_entry.timestamp) {
          shada_free_shada_entry(&cur_entry);
          break;
        }
      }
      if (!op_reg_set(cur_entry.data.reg.name, (yankreg_T) {
        .y_array = cur_entry.data.reg.contents,
        .y_size = cur_entry.data.reg.contents_size,
        .y_type = cur_entry.data.reg.type,
        .y_width = (colnr_T)cur_entry.data.reg.width,
        .timestamp = cur_entry.timestamp,
        .additional_data = cur_entry.additional_data,
      }, cur_entry.data.reg.is_unnamed)) {
        shada_free_shada_entry(&cur_entry);
      }
      // Do not free shada entry: its allocated memory was saved above.
      break;
    case kSDItemVariable:
      var_set_global(cur_entry.data.global_var.name,
                     cur_entry.data.global_var.value);
      cur_entry.data.global_var.value.v_type = VAR_UNKNOWN;
      shada_free_shada_entry(&cur_entry);
      break;
    case kSDItemJump:
    case kSDItemGlobalMark: {
      buf_T *buf = find_buffer(&fname_bufs, cur_entry.data.filemark.fname);
      if (buf != NULL) {
        XFREE_CLEAR(cur_entry.data.filemark.fname);
      }
      xfmark_T fm = (xfmark_T) {
        .fname = buf == NULL ? cur_entry.data.filemark.fname : NULL,
        .fmark = {
          .mark = cur_entry.data.filemark.mark,
          .fnum = (buf == NULL ? 0 : buf->b_fnum),
          .timestamp = cur_entry.timestamp,
          .view = INIT_FMARKV,
          .additional_data = cur_entry.additional_data,
        },
      };
      if (cur_entry.type == kSDItemGlobalMark) {
        if (!mark_set_global(cur_entry.data.filemark.name, fm, !force)) {
          shada_free_shada_entry(&cur_entry);
          break;
        }
      } else {
        int i;
        for (i = curwin->w_jumplistlen; i > 0; i--) {
          const xfmark_T jl_entry = curwin->w_jumplist[i - 1];
          if (jl_entry.fmark.timestamp <= cur_entry.timestamp) {
            if (rs_marks_equal(jl_entry.fmark.mark, cur_entry.data.filemark.mark) != 0
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
          if (curwin->w_jumplistidx >= i && curwin->w_jumplistidx + 1 <= curwin->w_jumplistlen) {
            curwin->w_jumplistidx++;
          }
        } else {
          shada_free_shada_entry(&cur_entry);
        }
      }

      // Do not free shada entry: its allocated memory was saved above.
      break;
    }
    case kSDItemBufferList:
      for (size_t i = 0; i < cur_entry.data.buffer_list.size; i++) {
        char *const sfname =
          path_try_shorten_fname(cur_entry.data.buffer_list.buffers[i].fname);
        buf_T *const buf =
          buflist_new(cur_entry.data.buffer_list.buffers[i].fname, sfname, 0, BLN_LISTED);
        if (buf != NULL) {
          fmarkv_T view = INIT_FMARKV;
          RESET_FMARK(&buf->b_last_cursor,
                      cur_entry.data.buffer_list.buffers[i].pos, 0, view);
          buflist_setfpos(buf, curwin, buf->b_last_cursor.mark.lnum,
                          buf->b_last_cursor.mark.col, false);

          xfree(buf->additional_data);
          buf->additional_data = cur_entry.data.buffer_list.buffers[i].additional_data;
          cur_entry.data.buffer_list.buffers[i].additional_data = NULL;
        }
      }
      shada_free_shada_entry(&cur_entry);
      break;
    case kSDItemChange:
    case kSDItemLocalMark: {
      if (get_old_files && !set_has(cstr_t, &oldfiles_set, cur_entry.data.filemark.fname)) {
        char *fname = cur_entry.data.filemark.fname;
        if (want_marks) {
          // Do not bother with allocating memory for the string if already
          // allocated string from cur_entry can be used. It cannot be used if
          // want_marks is set because this way it may be used for a mark.
          fname = xstrdup(fname);
        }
        set_put(cstr_t, &oldfiles_set, fname);
        tv_list_append_allocated_string(oldfiles_list, fname);
        if (!want_marks) {
          // Avoid free because this string was already used.
          cur_entry.data.filemark.fname = NULL;
        }
      }
      if (!want_marks) {
        shada_free_shada_entry(&cur_entry);
        break;
      }
      buf_T *buf = find_buffer(&fname_bufs, cur_entry.data.filemark.fname);
      if (buf == NULL) {
        shada_free_shada_entry(&cur_entry);
        break;
      }
      const fmark_T fm = (fmark_T) {
        .mark = cur_entry.data.filemark.mark,
        .fnum = 0,
        .timestamp = cur_entry.timestamp,
        .view = INIT_FMARKV,
        .additional_data = cur_entry.additional_data,
      };
      if (cur_entry.type == kSDItemLocalMark) {
        if (!mark_set_local(cur_entry.data.filemark.name, buf, fm, !force)) {
          shada_free_shada_entry(&cur_entry);
          break;
        }
      } else {
        set_put(ptr_t, &cl_bufs, buf);
        int i;
        for (i = buf->b_changelistlen; i > 0; i--) {
          const fmark_T jl_entry = buf->b_changelist[i - 1];
          if (jl_entry.timestamp <= cur_entry.timestamp) {
            if (rs_marks_equal(jl_entry.mark, cur_entry.data.filemark.mark) != 0) {
              i = -1;
            }
            break;
          }
        }
        if (i > 0 && buf->b_changelistlen == JUMPLISTSIZE) {
          free_fmark(buf->b_changelist[0]);
        }
        i = rs_marklist_insert(buf->b_changelist, sizeof(*buf->b_changelist), buf->b_changelistlen, i);
        if (i != -1) {
          buf->b_changelist[i] = fm;
          if (buf->b_changelistlen < JUMPLISTSIZE) {
            buf->b_changelistlen++;
          }
        } else {
          xfree(fm.additional_data);
        }
      }
      // only free fname part of shada entry, as additional_data was saved or freed above.
      xfree(cur_entry.data.filemark.fname);
      break;
    }
    }
  }
shada_read_main_cycle_end:
  // Warning: rs_shada_hist_iter returns ShadaEntry elements which use strings from
  //          original history list. This means that once such entry is removed
  //          from the history Neovim array will no longer be valid. To reduce
  //          amount of memory allocations ShaDa file reader allocates enough
  //          memory for the history string itself and separator character which
  //          may be assigned right away.
  if (srni_flags & kSDReadHistory) {
    for (int i = 0; i < HIST_COUNT; i++) {
      rs_hms_insert_whole_neovim_history(&hms[i]);
      clr_history(i);
      int *new_hisidx;
      int *new_hisnum;
      histentry_T *hist = hist_get_array((uint8_t)i, &new_hisidx, &new_hisnum);
      if (hist != NULL) {
        rs_hms_to_he_array(&hms[i], hist, new_hisidx, new_hisnum);
      }
      rs_hms_dealloc(&hms[i]);
    }
  }
  if (cl_bufs.h.n_occupied) {
    FOR_ALL_TAB_WINDOWS(tp, wp) {
      (void)tp;
      if (set_has(ptr_t, &cl_bufs, wp->w_buffer)) {
        wp->w_changelistidx = wp->w_buffer->b_changelistlen;
      }
    }
  }
  set_destroy(ptr_t, &cl_bufs);
  const char *key;
  map_foreach_key(&fname_bufs, key, {
    xfree((char *)key);
  })
  map_destroy(cstr_t, &fname_bufs);
  set_destroy(cstr_t, &oldfiles_set);
}


/// Get the ShaDa file name to use
///
/// If "file" is given and not empty, use it (has already been expanded by
/// cmdline functions). Otherwise use "-i file_name", value from 'shada' or the
/// default, and expand environment variables.
///
/// @param[in]  file  Forced file name or NULL.
///
/// @return  An allocated string containing shada file name,
///          or NULL if shada file should not be used.
static char *shada_filename(const char *file)
  FUNC_ATTR_MALLOC FUNC_ATTR_WARN_UNUSED_RESULT
{
  if (file == NULL || *file == NUL) {
    if (p_shadafile != NULL && *p_shadafile != NUL) {
      // Check if writing to ShaDa file was disabled ("-i NONE" or "--clean").
      if (!strequal(p_shadafile, "NONE")) {
        file = p_shadafile;
      } else {
        return NULL;
      }
    } else {
      if ((file = find_shada_parameter('n')) == NULL || *file == NUL) {
        file = rs_shada_get_default_file();
      }
      // XXX It used to be one level lower, so that whatever is in
      //     `p_shadafile` was expanded. I intentionally moved it here
      //     because various expansions must have already be done by the shell.
      //     If shell is not performing them then they should be done in main.c
      //     where arguments are parsed, *not here*.
      size_t len = expand_env((char *)file, &(NameBuff[0]), MAXPATHL);
      file = &(NameBuff[0]);
      return xmemdupz(file, len);
    }
  }
  return xstrdup(file);
}

#define KEY_NAME_(s) #s
#define PACK_KEY(s) mpack_str(STATIC_CSTR_AS_STRING(KEY_NAME_(s)), &sbuf);
#define KEY_NAME(s) KEY_NAME_(s)

#define SHADA_MPACK_FREE_SPACE (4 * MPACK_ITEM_SIZE)

static void shada_check_buffer(PackerBuffer *packer)
{
  if (mpack_remaining(packer) < SHADA_MPACK_FREE_SPACE) {
    packer->packer_flush(packer);
  }
}

static uint32_t additional_data_len(AdditionalData *src)
{
  return src ? src->nitems : 0;
}

static void dump_additional_data(AdditionalData *src, PackerBuffer *sbuf)
{
  if (src != NULL) {
    mpack_raw(src->data, src->nbytes, sbuf);
  }
}

/// Write single ShaDa entry
///
/// @param[in]  packer     Packer used to write entry.
/// @param[in]  entry      Entry written.
/// @param[in]  max_kbyte  Maximum size of an item in KiB. Zero means no
///                        restrictions.
///
/// @return kSDWriteSuccessful, kSDWriteFailed or kSDWriteIgnError.
static ShaDaWriteResult shada_pack_entry(PackerBuffer *const packer, ShadaEntry entry,
                                         const size_t max_kbyte)
  FUNC_ATTR_NONNULL_ALL
{
  ShaDaWriteResult ret = kSDWriteFailed;
  PackerBuffer sbuf = packer_string_buffer();

#define CHECK_DEFAULT(entry, attr) \
  (sd_default_values[(entry).type].data.attr == (entry).data.attr)
#define ONE_IF_NOT_DEFAULT(entry, attr) \
  ((uint32_t)(!CHECK_DEFAULT(entry, attr)))

#define PACK_BOOL(entry, name, attr) \
  do { \
    if (!CHECK_DEFAULT(entry, search_pattern.attr)) { \
      PACK_KEY(name); \
      mpack_bool(&sbuf.ptr, !sd_default_values[(entry).type].data.search_pattern.attr); \
    } \
  } while (0)

  shada_check_buffer(&sbuf);
  switch (entry.type) {
  case kSDItemMissing:
    abort();
  case kSDItemUnknown:
    mpack_raw(entry.data.unknown_item.contents, entry.data.unknown_item.size, &sbuf);
    break;
  case kSDItemHistoryEntry: {
    const bool is_hist_search =
      entry.data.history_item.histtype == HIST_SEARCH;
    uint32_t arr_size = (2 + (uint32_t)is_hist_search
                         + additional_data_len(entry.additional_data));
    mpack_array(&sbuf.ptr, arr_size);
    mpack_uint(&sbuf.ptr, entry.data.history_item.histtype);
    mpack_bin(cstr_as_string(entry.data.history_item.string), &sbuf);
    if (is_hist_search) {
      mpack_uint(&sbuf.ptr, (uint8_t)entry.data.history_item.sep);
    }
    dump_additional_data(entry.additional_data, &sbuf);
    break;
  }
  case kSDItemVariable: {
    bool is_blob = (entry.data.global_var.value.v_type == VAR_BLOB);
    uint32_t arr_size = 2 + (is_blob ? 1 : 0) + additional_data_len(entry.additional_data);
    mpack_array(&sbuf.ptr, arr_size);
    const String varname = cstr_as_string(entry.data.global_var.name);
    mpack_bin(varname, &sbuf);
    char vardesc[256] = "variable g:";
    memcpy(&vardesc[sizeof("variable g:") - 1], varname.data,
           varname.size + 1);
    if (encode_vim_to_msgpack(&sbuf, &entry.data.global_var.value, vardesc)
        == FAIL) {
      ret = kSDWriteIgnError;
      semsg(_(WERR "Failed to write variable %s"),
            entry.data.global_var.name);
      goto shada_pack_entry_error;
    }
    if (is_blob) {
      mpack_check_buffer(&sbuf);
      mpack_integer(&sbuf.ptr, VAR_TYPE_BLOB);
    }
    dump_additional_data(entry.additional_data, &sbuf);
    break;
  }
  case kSDItemSubString: {
    uint32_t arr_size = 1 + additional_data_len(entry.additional_data);
    mpack_array(&sbuf.ptr, arr_size);
    mpack_bin(cstr_as_string(entry.data.sub_string.sub), &sbuf);
    dump_additional_data(entry.additional_data, &sbuf);
    break;
  }
  case kSDItemSearchPattern: {
    uint32_t entry_map_size = (1  // Search pattern is always present
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.magic)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.is_last_used)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.smartcase)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.has_line_offset)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.place_cursor_at_end)
                               + ONE_IF_NOT_DEFAULT(entry,
                                                    search_pattern.is_substitute_pattern)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.highlighted)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.offset)
                               + ONE_IF_NOT_DEFAULT(entry, search_pattern.search_backward)
                               + additional_data_len(entry.additional_data));
    mpack_map(&sbuf.ptr, entry_map_size);
    PACK_KEY(SEARCH_KEY_PAT);
    mpack_bin(entry.data.search_pattern.pat, &sbuf);
    PACK_BOOL(entry, SEARCH_KEY_MAGIC, magic);
    PACK_BOOL(entry, SEARCH_KEY_IS_LAST_USED, is_last_used);
    PACK_BOOL(entry, SEARCH_KEY_SMARTCASE, smartcase);
    PACK_BOOL(entry, SEARCH_KEY_HAS_LINE_OFFSET, has_line_offset);
    PACK_BOOL(entry, SEARCH_KEY_PLACE_CURSOR_AT_END, place_cursor_at_end);
    PACK_BOOL(entry, SEARCH_KEY_IS_SUBSTITUTE_PATTERN, is_substitute_pattern);
    PACK_BOOL(entry, SEARCH_KEY_HIGHLIGHTED, highlighted);
    PACK_BOOL(entry, SEARCH_KEY_BACKWARD, search_backward);
    if (!CHECK_DEFAULT(entry, search_pattern.offset)) {
      PACK_KEY(SEARCH_KEY_OFFSET);
      mpack_integer(&sbuf.ptr, entry.data.search_pattern.offset);
    }
#undef PACK_BOOL
    dump_additional_data(entry.additional_data, &sbuf);
    break;
  }
  case kSDItemChange:
  case kSDItemGlobalMark:
  case kSDItemLocalMark:
  case kSDItemJump: {
    size_t entry_map_size = (1  // File name
                             + ONE_IF_NOT_DEFAULT(entry, filemark.mark.lnum)
                             + ONE_IF_NOT_DEFAULT(entry, filemark.mark.col)
                             + ONE_IF_NOT_DEFAULT(entry, filemark.name)
                             + additional_data_len(entry.additional_data));
    mpack_map(&sbuf.ptr, (uint32_t)entry_map_size);
    PACK_KEY(KEY_FILE);
    mpack_bin(cstr_as_string(entry.data.filemark.fname), &sbuf);
    if (!CHECK_DEFAULT(entry, filemark.mark.lnum)) {
      PACK_KEY(KEY_LNUM);
      mpack_integer(&sbuf.ptr, entry.data.filemark.mark.lnum);
    }
    if (!CHECK_DEFAULT(entry, filemark.mark.col)) {
      PACK_KEY(KEY_COL);
      mpack_integer(&sbuf.ptr, entry.data.filemark.mark.col);
    }
    assert(entry.type == kSDItemJump || entry.type == kSDItemChange
           ? CHECK_DEFAULT(entry, filemark.name)
           : true);
    if (!CHECK_DEFAULT(entry, filemark.name)) {
      PACK_KEY(KEY_NAME_CHAR);
      mpack_uint(&sbuf.ptr, (uint8_t)entry.data.filemark.name);
    }
    dump_additional_data(entry.additional_data, &sbuf);
    break;
  }
  case kSDItemRegister: {
    uint32_t entry_map_size = (2  // Register contents and name
                               + ONE_IF_NOT_DEFAULT(entry, reg.type)
                               + ONE_IF_NOT_DEFAULT(entry, reg.width)
                               + ONE_IF_NOT_DEFAULT(entry, reg.is_unnamed)
                               + additional_data_len(entry.additional_data));

    mpack_map(&sbuf.ptr, entry_map_size);
    PACK_KEY(REG_KEY_CONTENTS);
    mpack_array(&sbuf.ptr, (uint32_t)entry.data.reg.contents_size);
    for (size_t i = 0; i < entry.data.reg.contents_size; i++) {
      mpack_bin(entry.data.reg.contents[i], &sbuf);
    }
    PACK_KEY(KEY_NAME_CHAR);
    mpack_uint(&sbuf.ptr, (uint8_t)entry.data.reg.name);
    if (!CHECK_DEFAULT(entry, reg.type)) {
      PACK_KEY(REG_KEY_TYPE);
      mpack_uint(&sbuf.ptr, (uint8_t)entry.data.reg.type);
    }
    if (!CHECK_DEFAULT(entry, reg.width)) {
      PACK_KEY(REG_KEY_WIDTH);
      mpack_uint64(&sbuf.ptr, (uint64_t)entry.data.reg.width);
    }
    if (!CHECK_DEFAULT(entry, reg.is_unnamed)) {
      PACK_KEY(REG_KEY_UNNAMED);
      mpack_bool(&sbuf.ptr, entry.data.reg.is_unnamed);
    }
    dump_additional_data(entry.additional_data, &sbuf);
    break;
  }
  case kSDItemBufferList:
    mpack_array(&sbuf.ptr, (uint32_t)entry.data.buffer_list.size);
    for (size_t i = 0; i < entry.data.buffer_list.size; i++) {
      size_t entry_map_size = (1  // Buffer name
                               + (size_t)(entry.data.buffer_list.buffers[i].pos.lnum
                                          != default_pos.lnum)
                               + (size_t)(entry.data.buffer_list.buffers[i].pos.col
                                          != default_pos.col)
                               + additional_data_len(entry.data.buffer_list.buffers[i].
                                                     additional_data));
      mpack_map(&sbuf.ptr, (uint32_t)entry_map_size);
      PACK_KEY(KEY_FILE);
      mpack_bin(cstr_as_string(entry.data.buffer_list.buffers[i].fname), &sbuf);
      if (entry.data.buffer_list.buffers[i].pos.lnum != 1) {
        PACK_KEY(KEY_LNUM);
        mpack_uint64(&sbuf.ptr, (uint64_t)entry.data.buffer_list.buffers[i].pos.lnum);
      }
      if (entry.data.buffer_list.buffers[i].pos.col != 0) {
        PACK_KEY(KEY_COL);
        mpack_uint64(&sbuf.ptr, (uint64_t)entry.data.buffer_list.buffers[i].pos.col);
      }
      dump_additional_data(entry.data.buffer_list.buffers[i].additional_data, &sbuf);
    }
    break;
  case kSDItemHeader:
    mpack_map(&sbuf.ptr, (uint32_t)entry.data.header.size);
    for (size_t i = 0; i < entry.data.header.size; i++) {
      mpack_str(entry.data.header.items[i].key, &sbuf);
      const Object obj = entry.data.header.items[i].value;
      switch (obj.type) {
      case kObjectTypeString:
        mpack_bin(obj.data.string, &sbuf);
        break;
      case kObjectTypeInteger:
        mpack_integer(&sbuf.ptr, obj.data.integer);
        break;
      default:
        abort();
      }
    }
    break;
  }
#undef CHECK_DEFAULT
#undef ONE_IF_NOT_DEFAULT
  String packed = packer_take_string(&sbuf);
  if (!max_kbyte || packed.size <= max_kbyte * 1024) {
    shada_check_buffer(packer);

    if (entry.type == kSDItemUnknown) {
      mpack_uint64(&packer->ptr, entry.data.unknown_item.type);
    } else {
      mpack_uint64(&packer->ptr, (uint64_t)entry.type);
    }
    mpack_uint64(&packer->ptr, (uint64_t)entry.timestamp);
    if (packed.size > 0) {
      mpack_uint64(&packer->ptr, (uint64_t)packed.size);
      mpack_raw(packed.data, packed.size, packer);
    }

    if (packer->anyint != 0) {  // error code
      goto shada_pack_entry_error;
    }
  }
  ret = kSDWriteSuccessful;
shada_pack_entry_error:
  xfree(sbuf.startptr);
  return ret;
}

/// Write single ShaDa entry and free it afterwards
///
/// Will not free if entry could not be freed.
///
/// @param[in]  packer     Packer used to write entry.
/// @param[in]  entry      Entry written.
/// @param[in]  max_kbyte  Maximum size of an item in KiB. Zero means no
///                        restrictions.
static inline ShaDaWriteResult shada_pack_pfreed_entry(PackerBuffer *const packer, ShadaEntry entry,
                                                       const size_t max_kbyte)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_ALWAYS_INLINE
{
  ShaDaWriteResult ret = shada_pack_entry(packer, entry, max_kbyte);
  shada_free_shada_entry(&entry);
  return ret;
}

/// Compare two FileMarks structure to order them by greatest_timestamp.
/// Delegated to Rust rs_compare_file_marks.
static int compare_file_marks(const void *a, const void *b)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return rs_compare_file_marks(a, b);
}

/// Parse msgpack object that has given length
///
/// @param[in]   sd_reader     Structure containing file reader definition.
/// @param[in]   length        Object length.
/// @param[out]  ret_unpacked  Location where read result should be saved. If
///                            NULL then unpacked data will be freed. Must be
///                            NULL if `ret_buf` is NULL.
/// @param[out]  ret_buf       Buffer containing parsed string.
///
/// @return kSDReadStatusNotShaDa, kSDReadStatusReadError or
///         kSDReadStatusSuccess.
static ShaDaReadResult shada_check_status(uintmax_t initial_fpos, int status, size_t remaining)
  FUNC_ATTR_WARN_UNUSED_RESULT
{
  switch (status) {
  case MPACK_OK:
    if (remaining) {
      semsg(_(RCERR "Failed to parse ShaDa file: extra bytes in msgpack string "
              "at position %" PRIu64),
            (uint64_t)initial_fpos);
      return kSDReadStatusNotShaDa;
    }
    return kSDReadStatusSuccess;
  case MPACK_EOF:
    semsg(_(RCERR "Failed to parse ShaDa file: incomplete msgpack string "
            "at position %" PRIu64),
          (uint64_t)initial_fpos);
    return kSDReadStatusNotShaDa;
  default:
    semsg(_(RCERR "Failed to parse ShaDa file due to a msgpack parser error "
            "at position %" PRIu64),
          (uint64_t)initial_fpos);
    return kSDReadStatusNotShaDa;
  }
}

/// Format shada entry for debugging purposes
///
/// @param[in]  entry  ShaDa entry to format.
///
/// @return string representing ShaDa entry in a static buffer.
static const char *shada_format_entry(const ShadaEntry entry)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_UNUSED FUNC_ATTR_NONNULL_RET
{
  static char ret[1024];
  ret[0] = 0;
  vim_snprintf(S_LEN(ret), "%s", "[ ] ts=%" PRIu64 " ");
  //                         ^ Space for `can_free_entry`
#define FORMAT_MARK_ENTRY(entry_name, name_fmt, name_fmt_arg) \
  do { \
    vim_snprintf_add(S_LEN(ret), \
                     entry_name " {" name_fmt " file=[%zu]\"%.512s\", " \
                     "pos={l=%" PRIdLINENR ",c=%" PRIdCOLNR ",a=%" PRIdCOLNR "}, " \
                     "}", \
                     name_fmt_arg, \
                     strlen(entry.data.filemark.fname), \
                     entry.data.filemark.fname, \
                     entry.data.filemark.mark.lnum, \
                     entry.data.filemark.mark.col, \
                     entry.data.filemark.mark.coladd); \
  } while (0)
  switch (entry.type) {
  case kSDItemMissing:
    vim_snprintf_add(S_LEN(ret), "Missing");
    break;
  case kSDItemHeader:
    vim_snprintf_add(S_LEN(ret), "Header { TODO }");
    break;
  case kSDItemBufferList:
    vim_snprintf_add(S_LEN(ret), "BufferList { TODO }");
    break;
  case kSDItemUnknown:
    vim_snprintf_add(S_LEN(ret), "Unknown { TODO }");
    break;
  case kSDItemSearchPattern:
    vim_snprintf_add(S_LEN(ret), "SearchPattern { TODO }");
    break;
  case kSDItemSubString:
    vim_snprintf_add(S_LEN(ret), "SubString { TODO }");
    break;
  case kSDItemHistoryEntry:
    vim_snprintf_add(S_LEN(ret), "HistoryEntry { TODO }");
    break;
  case kSDItemRegister:
    vim_snprintf_add(S_LEN(ret), "Register { TODO }");
    break;
  case kSDItemVariable:
    vim_snprintf_add(S_LEN(ret), "Variable { TODO }");
    break;
  case kSDItemGlobalMark:
    FORMAT_MARK_ENTRY("GlobalMark", " name='%c',", entry.data.filemark.name);
    break;
  case kSDItemChange:
    FORMAT_MARK_ENTRY("Change", "%s", "");
    break;
  case kSDItemLocalMark:
    FORMAT_MARK_ENTRY("LocalMark", " name='%c',", entry.data.filemark.name);
    break;
  case kSDItemJump:
    FORMAT_MARK_ENTRY("Jump", "%s", "");
    break;
#undef FORMAT_MARK_ENTRY
  }
  ret[1] = (entry.can_free_entry ? 'T' : 'F');
  return ret;
}

/// Read and merge in ShaDa file, used when writing
///
/// @param[in]      sd_reader   Structure containing file reader definition.
/// @param[in]      srni_flags  Flags determining what to read.
/// @param[in]      max_kbyte   Maximum size of one element.
/// @param[in,out]  ret_wms     Location where results are saved.
/// @param[out]     packer      MessagePack packer for entries which are not
///                             merged.
static inline ShaDaWriteResult shada_read_when_writing(FileDescriptor *const sd_reader,
                                                       const unsigned srni_flags,
                                                       const size_t max_kbyte,
                                                       WriteMergerState *const wms,
                                                       PackerBuffer *const packer)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  ShaDaWriteResult ret = kSDWriteSuccessful;
  ShadaEntry entry;
  ShaDaReadResult srni_ret;

#define COMPARE_WITH_ENTRY(wms_entry_, entry) \
  do { \
    ShadaEntry *const wms_entry = (wms_entry_); \
    if (wms_entry->type != kSDItemMissing) { \
      if (wms_entry->timestamp >= (entry).timestamp) { \
        shada_free_shada_entry(&entry); \
        break; \
      } \
      shada_free_shada_entry(wms_entry); \
    } \
    *wms_entry = entry; \
  } while (0)

  while ((srni_ret = shada_read_next_item(sd_reader, &entry, srni_flags,
                                          max_kbyte))
         != kSDReadStatusFinished) {
    switch (srni_ret) {
    case kSDReadStatusSuccess:
      break;
    case kSDReadStatusFinished:
      // Should be handled by the while condition.
      abort();
    case kSDReadStatusNotShaDa:
      ret = kSDWriteReadNotShada;
      FALLTHROUGH;
    case kSDReadStatusReadError:
      return ret;
    case kSDReadStatusMalformed:
      continue;
    }
    switch (entry.type) {
    case kSDItemMissing:
      break;
    case kSDItemHeader:
    case kSDItemBufferList:
      abort();
    case kSDItemUnknown:
      ret = shada_pack_entry(packer, entry, 0);
      shada_free_shada_entry(&entry);
      break;
    case kSDItemSearchPattern:
      COMPARE_WITH_ENTRY((entry.data.search_pattern.is_substitute_pattern
                          ? &wms->sub_search_pattern
                          : &wms->search_pattern), entry);
      break;
    case kSDItemSubString:
      COMPARE_WITH_ENTRY(&wms->replacement, entry);
      break;
    case kSDItemHistoryEntry:
      if (entry.data.history_item.histtype >= HIST_COUNT) {
        ret = shada_pack_entry(packer, entry, 0);
        shada_free_shada_entry(&entry);
        break;
      }
      if (wms->hms[entry.data.history_item.histtype].hmll.size != 0) {
        rs_hms_insert(&wms->hms[entry.data.history_item.histtype], entry, 1);
      } else {
        shada_free_shada_entry(&entry);
      }
      break;
    case kSDItemRegister: {
      const int idx = op_reg_index(entry.data.reg.name);
      if (idx < 0) {
        ret = shada_pack_entry(packer, entry, 0);
        shada_free_shada_entry(&entry);
        break;
      }
      COMPARE_WITH_ENTRY(&wms->registers[idx], entry);
      break;
    }
    case kSDItemVariable:
      if (!set_has(cstr_t, &wms->dumped_variables, entry.data.global_var.name)) {
        ret = shada_pack_entry(packer, entry, 0);
      }
      shada_free_shada_entry(&entry);
      break;
    case kSDItemGlobalMark:
      if (ascii_isdigit(entry.data.filemark.name)) {
        bool processed_mark = false;
        // Completely ignore numbered mark names, make a list sorted by
        // timestamp.
        for (size_t i = ARRAY_SIZE(wms->numbered_marks); i > 0; i--) {
          ShadaEntry wms_entry = wms->numbered_marks[i - 1];
          if (wms_entry.type != kSDItemGlobalMark) {
            continue;
          }
          // Ignore duplicates.
          if (wms_entry.timestamp == entry.timestamp
              && (wms_entry.additional_data == NULL
                  && entry.additional_data == NULL)
              && rs_marks_equal(wms_entry.data.filemark.mark,
                                entry.data.filemark.mark) != 0
              && strcmp(wms_entry.data.filemark.fname,
                        entry.data.filemark.fname) == 0) {
            shada_free_shada_entry(&entry);
            processed_mark = true;
            break;
          }
          if (wms_entry.timestamp >= entry.timestamp) {
            processed_mark = true;
            if (i < ARRAY_SIZE(wms->numbered_marks)) {
              rs_replace_numbered_mark(wms, i, entry);
            } else {
              shada_free_shada_entry(&entry);
            }
            break;
          }
        }
        if (!processed_mark) {
          rs_replace_numbered_mark(wms, 0, entry);
        }
      } else {
        const int idx = mark_global_index(entry.data.filemark.name);
        if (idx < 0) {
          ret = shada_pack_entry(packer, entry, 0);
          shada_free_shada_entry(&entry);
          break;
        }

        // Global or numbered mark.
        ShadaEntry *mark = idx < 26 ? &wms->global_marks[idx] : &wms->numbered_marks[idx - 26];

        if (mark->type == kSDItemMissing) {
          if (namedfm[idx].fmark.timestamp >= entry.timestamp) {
            shada_free_shada_entry(&entry);
            break;
          }
        }
        COMPARE_WITH_ENTRY(mark, entry);
      }
      break;
    case kSDItemChange:
    case kSDItemLocalMark: {
      if (rs_shada_removable(entry.data.filemark.fname) != 0) {
        shada_free_shada_entry(&entry);
        break;
      }
      const char *const fname = entry.data.filemark.fname;
      cstr_t *key = NULL;
      bool new_item = false;
      ptr_t *val = pmap_put_ref(cstr_t)(&wms->file_marks, fname, &key, &new_item);
      if (new_item) {
        *key = xstrdup(fname);
      }
      if (*val == NULL) {
        *val = xcalloc(1, sizeof(FileMarks));
      }
      FileMarks *const filemarks = *val;
      if (entry.timestamp > filemarks->greatest_timestamp) {
        filemarks->greatest_timestamp = entry.timestamp;
      }
      if (entry.type == kSDItemLocalMark) {
        const int idx = mark_local_index(entry.data.filemark.name);
        if (idx < 0) {
          filemarks->additional_marks = xrealloc(filemarks->additional_marks,
                                                 (++filemarks->additional_marks_size
                                                  * sizeof(filemarks->additional_marks[0])));
          filemarks->additional_marks[filemarks->additional_marks_size - 1] =
            entry;
        } else {
          ShadaEntry *const wms_entry = &filemarks->marks[idx];
          bool set_wms = true;
          if (wms_entry->type != kSDItemMissing) {
            if (wms_entry->timestamp >= entry.timestamp) {
              shada_free_shada_entry(&entry);
              break;
            }
            if (wms_entry->can_free_entry) {
              if (*key == wms_entry->data.filemark.fname) {
                *key = entry.data.filemark.fname;
              }
              shada_free_shada_entry(wms_entry);
            }
          } else {
            FOR_ALL_BUFFERS(buf) {
              if (buf->b_ffname != NULL
                  && path_fnamecmp(entry.data.filemark.fname, buf->b_ffname) == 0) {
                fmark_T fm;
                mark_get(buf, curwin, &fm, kMarkBufLocal, (int)entry.data.filemark.name);
                if (fm.timestamp >= entry.timestamp) {
                  set_wms = false;
                  shada_free_shada_entry(&entry);
                  break;
                }
              }
            }
          }
          if (set_wms) {
            *wms_entry = entry;
          }
        }
      } else {
        int i;
        for (i = (int)filemarks->changes_size; i > 0; i--) {
          const ShadaEntry jl_entry = filemarks->changes[i - 1];
          if (jl_entry.timestamp <= (entry).timestamp) {
            if (rs_marks_equal(jl_entry.data.filemark.mark, entry.data.filemark.mark) != 0) {
              i = -1;
            }
            break;
          }
        }
        if (i > 0 && filemarks->changes_size == JUMPLISTSIZE) {
          shada_free_shada_entry(&filemarks->changes[0]);
        }
        i = rs_marklist_insert(filemarks->changes, sizeof(*filemarks->changes),
                            (int)filemarks->changes_size, i);
        if (i != -1) {
          filemarks->changes[i] = entry;
          if (filemarks->changes_size < JUMPLISTSIZE) {
            filemarks->changes_size++;
          }
        } else {
          shada_free_shada_entry(&(entry));
        }
      }
      break;
    }
    case kSDItemJump:
      ;
      int i;
      for (i = (int)wms->jumps_size; i > 0; i--) {
        const ShadaEntry jl_entry = wms->jumps[i - 1];
        if (jl_entry.timestamp <= entry.timestamp) {
          if (rs_marks_equal(jl_entry.data.filemark.mark, entry.data.filemark.mark) != 0
              && strcmp(jl_entry.data.filemark.fname, entry.data.filemark.fname) == 0) {
            i = -1;
          }
          break;
        }
      }
      if (i > 0 && wms->jumps_size == JUMPLISTSIZE) {
        shada_free_shada_entry(&wms->jumps[0]);
      }
      i = rs_marklist_insert(wms->jumps, sizeof(*wms->jumps), (int)wms->jumps_size, i);
      if (i != -1) {
        wms->jumps[i] = entry;
        if (wms->jumps_size < JUMPLISTSIZE) {
          wms->jumps_size++;
        }
      } else {
        shada_free_shada_entry(&entry);
      }
      break;
    }
  }
#undef COMPARE_WITH_ENTRY
  return ret;
}

static PackerBuffer packer_buffer_for_file(FileDescriptor *file)
{
  if (file_space(file) < SHADA_MPACK_FREE_SPACE) {
    file_flush(file);
  }
  return (PackerBuffer) {
    .startptr = file->buffer,
    .ptr = file->write_pos,
    .endptr = file->buffer + ARENA_BLOCK_SIZE,
    .anydata = file,
    .anyint = 0,  // set to nonzero if error
    .packer_flush = flush_file_buffer,
  };
}

static void flush_file_buffer(PackerBuffer *buffer)
{
  FileDescriptor *fd = buffer->anydata;
  fd->write_pos = buffer->ptr;
  buffer->anyint = file_flush(fd);
  buffer->ptr = fd->write_pos;
}

/// Write ShaDa file
///
/// @param[in]  sd_writer  Structure containing file writer definition.
/// @param[in]  sd_reader  Structure containing file reader definition. If it is
///                        not NULL then contents of this file will be merged
///                        with current Neovim runtime.
static ShaDaWriteResult shada_write(FileDescriptor *const sd_writer,
                                    FileDescriptor *const sd_reader)
  FUNC_ATTR_NONNULL_ARG(1)
{
  ShaDaWriteResult ret = kSDWriteSuccessful;
  int max_kbyte_i = get_shada_parameter('s');
  if (max_kbyte_i < 0) {
    max_kbyte_i = 10;
  }
  if (max_kbyte_i == 0) {
    return ret;
  }

  WriteMergerState *const wms = xcalloc(1, sizeof(*wms));
  bool dump_one_history[HIST_COUNT];
  const bool dump_global_vars = (find_shada_parameter('!') != NULL);
  int max_reg_lines = get_shada_parameter('<');
  if (max_reg_lines < 0) {
    max_reg_lines = get_shada_parameter('"');
  }
  const bool dump_registers = (max_reg_lines != 0);
  Set(ptr_t) removable_bufs = SET_INIT;
  const size_t max_kbyte = (size_t)max_kbyte_i;
  const size_t num_marked_files = (size_t)get_shada_parameter('\'');
  const bool dump_global_marks = get_shada_parameter('f') != 0;
  bool dump_history = false;

  // Initialize history merger
  for (int i = 0; i < HIST_COUNT; i++) {
    int num_saved = get_shada_parameter(rs_hist_type2char(i));
    if (num_saved == -1) {
      num_saved = (int)p_hi;
    }
    if (num_saved > 0) {
      dump_history = true;
      dump_one_history[i] = true;
      rs_hms_init(&wms->hms[i], (uint8_t)i, (size_t)num_saved, sd_reader != NULL ? 1 : 0, 0);
    } else {
      dump_one_history[i] = false;
    }
  }

  const unsigned srni_flags = (unsigned)(kSDReadUndisableableData
                                         | kSDReadUnknown
                                         | (dump_history ? kSDReadHistory : 0)
                                         | (dump_registers ? kSDReadRegisters : 0)
                                         | (dump_global_vars ? kSDReadVariables : 0)
                                         | (dump_global_marks ? kSDReadGlobalMarks : 0)
                                         | (num_marked_files ? kSDReadLocalMarks |
                                            kSDReadChanges : 0));

  PackerBuffer packer = packer_buffer_for_file(sd_writer);

  // Set b_last_cursor for all the buffers that have a window.
  //
  // It is needed to correctly save '"' mark on exit. Has a side effect of
  // setting '"' mark in all windows on :wshada to the current cursor
  // position (basically what :wviminfo used to do).
  FOR_ALL_TAB_WINDOWS(tp, wp) {
    set_last_cursor(wp);
  }

  rs_find_removable_bufs(&removable_bufs);

  // Write header
  if (shada_pack_entry(&packer, (ShadaEntry) {
    .type = kSDItemHeader,
    .timestamp = os_time(),
    .data = {
      .header = {
        .size = 5,
        .capacity = 5,
        .items = ((KeyValuePair[]) {
          { STATIC_CSTR_AS_STRING("generator"),
            STATIC_CSTR_AS_OBJ("nvim") },
          { STATIC_CSTR_AS_STRING("version"),
            CSTR_AS_OBJ(longVersion) },
          { STATIC_CSTR_AS_STRING("max_kbyte"),
            INTEGER_OBJ((Integer)max_kbyte) },
          { STATIC_CSTR_AS_STRING("pid"),
            INTEGER_OBJ((Integer)os_get_pid()) },
          { STATIC_CSTR_AS_STRING("encoding"),
            CSTR_AS_OBJ(p_enc) },
        }),
      }
    }
  }, 0) == kSDWriteFailed) {
    ret = kSDWriteFailed;
    goto shada_write_exit;
  }

  // Write buffer list
  if (find_shada_parameter('%') != NULL) {
    ShadaEntry buflist_entry = rs_shada_get_buflist(&removable_bufs);
    if (shada_pack_entry(&packer, buflist_entry, 0) == kSDWriteFailed) {
      xfree(buflist_entry.data.buffer_list.buffers);
      ret = kSDWriteFailed;
      goto shada_write_exit;
    }
    xfree(buflist_entry.data.buffer_list.buffers);
  }

  // Write some of the variables
  if (dump_global_vars) {
    const void *var_iter = NULL;
    const Timestamp cur_timestamp = os_time();
    do {
      typval_T vartv;
      const char *name = NULL;
      var_iter = var_shada_iter(var_iter, &name, &vartv, VAR_FLAVOUR_SHADA);
      if (name == NULL) {
        break;
      }
      switch (vartv.v_type) {
      case VAR_FUNC:
      case VAR_PARTIAL:
        tv_clear(&vartv);
        continue;
      case VAR_DICT: {
        dict_T *di = vartv.vval.v_dict;
        int copyID = rs_get_copyID();
        if (!rs_set_ref_in_ht(&di->dv_hashtab, copyID, NULL)
            && copyID == di->dv_copyID) {
          tv_clear(&vartv);
          continue;
        }
        break;
      }
      case VAR_LIST: {
        list_T *l = vartv.vval.v_list;
        int copyID = rs_get_copyID();
        if (!rs_set_ref_in_list_items(l, copyID, NULL)
            && copyID == l->lv_copyID) {
          tv_clear(&vartv);
          continue;
        }
        break;
      }
      default:
        break;
      }
      typval_T tgttv;
      tv_copy(&vartv, &tgttv);
      ShaDaWriteResult spe_ret;
      if ((spe_ret = shada_pack_entry(&packer, (ShadaEntry) {
        .type = kSDItemVariable,
        .timestamp = cur_timestamp,
        .data = {
          .global_var = {
            .name = (char *)name,
            .value = tgttv,
          }
        },
        .additional_data = NULL,
      }, max_kbyte)) == kSDWriteFailed) {
        tv_clear(&vartv);
        tv_clear(&tgttv);
        ret = kSDWriteFailed;
        goto shada_write_exit;
      }
      tv_clear(&vartv);
      tv_clear(&tgttv);
      if (spe_ret == kSDWriteSuccessful) {
        set_put(cstr_t, &wms->dumped_variables, name);
      }
    } while (var_iter != NULL);
  }

  if (num_marked_files > 0) {  // Skip if '0 in 'shada'
    // Initialize jump list
    wms->jumps_size = rs_shada_init_jumps(wms->jumps, &removable_bufs);
  }

  if (dump_one_history[HIST_SEARCH] > 0) {  // Skip if /0 in 'shada'
    const bool search_highlighted = !(no_hlsearch
                                      || find_shada_parameter('h') != NULL);
    const bool search_last_used = search_was_last_used();

    // Initialize search pattern
    rs_add_search_pattern(&wms->search_pattern, 0,
                          search_last_used ? 1 : 0, search_highlighted ? 1 : 0);

    // Initialize substitute search pattern
    rs_add_search_pattern(&wms->sub_search_pattern, 1,
                          search_last_used ? 1 : 0, search_highlighted ? 1 : 0);

    // Initialize substitute replacement string
    SubReplacementString sub;
    sub_get_replacement(&sub);
    if (sub.sub != NULL) {  // Don't store empty replacement string
      wms->replacement = (ShadaEntry) {
        .can_free_entry = false,
        .type = kSDItemSubString,
        .timestamp = sub.timestamp,
        .data = {
          .sub_string = {
            .sub = sub.sub,
          }
        },
        .additional_data = sub.additional_data,
      };
    }
  }

  // Initialize global marks
  if (dump_global_marks) {
    const void *global_mark_iter = NULL;
    size_t digit_mark_idx = 0;
    do {
      char name = NUL;
      xfmark_T fm;
      global_mark_iter = mark_global_iter(global_mark_iter, &name, &fm);
      if (name == NUL) {
        break;
      }
      const char *fname;
      if (fm.fmark.fnum == 0) {
        assert(fm.fname != NULL);
        if (rs_shada_removable(fm.fname) != 0) {
          continue;
        }
        fname = fm.fname;
      } else {
        const buf_T *const buf = buflist_findnr(fm.fmark.fnum);
        if (buf == NULL || buf->b_ffname == NULL
            || set_has(ptr_t, &removable_bufs, (ptr_t)buf)) {
          continue;
        }
        fname = buf->b_ffname;
      }
      const ShadaEntry entry = {
        .can_free_entry = false,
        .type = kSDItemGlobalMark,
        .timestamp = fm.fmark.timestamp,
        .data = {
          .filemark = {
            .mark = fm.fmark.mark,
            .name = name,
            .fname = (char *)fname,
          }
        },
        .additional_data = fm.fmark.additional_data,
      };
      if (ascii_isdigit(name)) {
        rs_replace_numbered_mark(wms, digit_mark_idx++, entry);
      } else {
        wms->global_marks[mark_global_index(name)] = entry;
      }
    } while (global_mark_iter != NULL);
  }

  // Initialize registers
  if (dump_registers) {
    rs_shada_initialize_registers(wms, max_reg_lines);
  }

  // Initialize buffers
  if (num_marked_files > 0) {
    FOR_ALL_BUFFERS(buf) {
      if (rs_ignore_buf(buf, &removable_bufs) != 0) {
        continue;
      }
      const void *local_marks_iter = NULL;
      const char *const fname = buf->b_ffname;
      cstr_t *map_key = NULL;
      bool new_item = false;
      ptr_t *val = pmap_put_ref(cstr_t)(&wms->file_marks, fname, &map_key, &new_item);
      if (new_item) {
        *map_key = xstrdup(fname);
      }
      if (*val == NULL) {
        *val = xcalloc(1, sizeof(FileMarks));
      }
      FileMarks *const filemarks = *val;
      do {
        fmark_T fm;
        char name = NUL;
        local_marks_iter = mark_buffer_iter(local_marks_iter, buf, &name, &fm);
        if (name == NUL) {
          break;
        }
        filemarks->marks[mark_local_index(name)] = (ShadaEntry) {
          .can_free_entry = false,
          .type = kSDItemLocalMark,
          .timestamp = fm.timestamp,
          .data = {
            .filemark = {
              .mark = fm.mark,
              .name = name,
              .fname = (char *)fname,
            }
          },
          .additional_data = fm.additional_data,
        };
        if (fm.timestamp > filemarks->greatest_timestamp) {
          filemarks->greatest_timestamp = fm.timestamp;
        }
      } while (local_marks_iter != NULL);
      for (int i = 0; i < buf->b_changelistlen; i++) {
        const fmark_T fm = buf->b_changelist[i];
        filemarks->changes[i] = (ShadaEntry) {
          .can_free_entry = false,
          .type = kSDItemChange,
          .timestamp = fm.timestamp,
          .data = {
            .filemark = {
              .mark = fm.mark,
              .fname = (char *)fname,
            }
          },
          .additional_data = fm.additional_data,
        };
        if (fm.timestamp > filemarks->greatest_timestamp) {
          filemarks->greatest_timestamp = fm.timestamp;
        }
      }
      filemarks->changes_size = (size_t)buf->b_changelistlen;
    }
  }

  if (sd_reader != NULL) {
    const ShaDaWriteResult srww_ret = shada_read_when_writing(sd_reader, srni_flags, max_kbyte, wms,
                                                              &packer);
    if (srww_ret != kSDWriteSuccessful) {
      ret = srww_ret;
    }
  }

  // Update numbered marks: replace '0 mark with the current position,
  // remove '9 and shift all other marks. Skip if f0 in 'shada'.
  if (dump_global_marks && rs_ignore_buf(curbuf, &removable_bufs) == 0 && curwin->w_cursor.lnum != 0) {
    rs_replace_numbered_mark(wms, 0, (ShadaEntry) {
      .can_free_entry = false,
      .type = kSDItemGlobalMark,
      .timestamp = os_time(),
      .data = {
        .filemark = {
          .mark = curwin->w_cursor,
          .name = '0',
          .fname = curbuf->b_ffname,
        }
      },
      .additional_data = NULL,
    });
  }

  // Write the rest
#define PACK_WMS_ARRAY(wms_array) \
  do { \
    for (size_t i_ = 0; i_ < ARRAY_SIZE(wms_array); i_++) { \
      if ((wms_array)[i_].type != kSDItemMissing) { \
        if (shada_pack_pfreed_entry(&packer, (wms_array)[i_], max_kbyte) \
            == kSDWriteFailed) { \
          ret = kSDWriteFailed; \
          goto shada_write_exit; \
        } \
      } \
    } \
  } while (0)
  PACK_WMS_ARRAY(wms->global_marks);
  PACK_WMS_ARRAY(wms->numbered_marks);
  PACK_WMS_ARRAY(wms->registers);
  for (size_t i = 0; i < wms->jumps_size; i++) {
    if (shada_pack_pfreed_entry(&packer, wms->jumps[i], max_kbyte)
        == kSDWriteFailed) {
      ret = kSDWriteFailed;
      goto shada_write_exit;
    }
  }
#define PACK_WMS_ENTRY(wms_entry) \
  do { \
    if ((wms_entry).type != kSDItemMissing) { \
      if (shada_pack_pfreed_entry(&packer, wms_entry, max_kbyte) \
          == kSDWriteFailed) { \
        ret = kSDWriteFailed; \
        goto shada_write_exit; \
      } \
    } \
  } while (0)
  PACK_WMS_ENTRY(wms->search_pattern);
  PACK_WMS_ENTRY(wms->sub_search_pattern);
  PACK_WMS_ENTRY(wms->replacement);
#undef PACK_WMS_ENTRY

  const size_t file_markss_size = map_size(&wms->file_marks);
  FileMarks **const all_file_markss =
    xmalloc(file_markss_size * sizeof(*all_file_markss));
  FileMarks **cur_file_marks = all_file_markss;
  ptr_t val;
  map_foreach_value(&wms->file_marks, val, {
    *cur_file_marks++ = val;
  })
  qsort((void *)all_file_markss, file_markss_size, sizeof(*all_file_markss),
        &compare_file_marks);
  const size_t file_markss_to_dump = MIN(num_marked_files, file_markss_size);
  for (size_t i = 0; i < file_markss_to_dump; i++) {
    PACK_WMS_ARRAY(all_file_markss[i]->marks);
    for (size_t j = 0; j < all_file_markss[i]->changes_size; j++) {
      if (shada_pack_pfreed_entry(&packer, all_file_markss[i]->changes[j],
                                  max_kbyte) == kSDWriteFailed) {
        ret = kSDWriteFailed;
        goto shada_write_exit;
      }
    }
    for (size_t j = 0; j < all_file_markss[i]->additional_marks_size; j++) {
      if (shada_pack_entry(&packer, all_file_markss[i]->additional_marks[j],
                           0) == kSDWriteFailed) {
        shada_free_shada_entry(&all_file_markss[i]->additional_marks[j]);
        ret = kSDWriteFailed;
        goto shada_write_exit;
      }
      shada_free_shada_entry(&all_file_markss[i]->additional_marks[j]);
    }
    xfree(all_file_markss[i]->additional_marks);
  }
  xfree(all_file_markss);
#undef PACK_WMS_ARRAY

  if (dump_history) {
    for (int i = 0; i < HIST_COUNT; i++) {
      if (dump_one_history[i]) {
        rs_hms_insert_whole_neovim_history(&wms->hms[i]);
        HMS_ITER(&wms->hms[i], cur_entry, {
          if (shada_pack_pfreed_entry(&packer, cur_entry->data, max_kbyte) == kSDWriteFailed) {
            ret = kSDWriteFailed;
            break;
          }
        })
        if (ret == kSDWriteFailed) {
          goto shada_write_exit;
        }
      }
    }
  }

shada_write_exit:
  for (int i = 0; i < HIST_COUNT; i++) {
    if (dump_one_history[i]) {
      rs_hms_dealloc(&wms->hms[i]);
    }
  }
  const char *stored_key = NULL;
  map_foreach(&wms->file_marks, stored_key, val, {
    xfree((char *)stored_key);
    xfree(val);
  })
  map_destroy(cstr_t, &wms->file_marks);
  set_destroy(ptr_t, &removable_bufs);
  packer.packer_flush(&packer);
  set_destroy(cstr_t, &wms->dumped_variables);
  xfree(wms);
  return ret;
}

#undef PACK_KEY

/// Write ShaDa file to a given location
///
/// @param[in]  fname    File to write to. If it is NULL or empty then default
///                      location is used.
/// @param[in]  nomerge  If true then old file is ignored.
///
/// @return OK if writing was successful, FAIL otherwise.
int shada_write_file(const char *const file, bool nomerge)
{
  char *const fname = shada_filename(file);
  if (fname == NULL) {
    return FAIL;
  }

  char *tempname = NULL;
  FileDescriptor sd_writer;
  FileDescriptor sd_reader;
  bool did_open_writer = false;
  bool did_open_reader = false;

  if (!nomerge) {
    int error;
    if ((error = file_open(&sd_reader, fname, kFileReadOnly, 0)) != 0) {
      if (error != UV_ENOENT) {
        semsg(_(SERR "System error while opening ShaDa file %s for reading "
                "to merge before writing it: %s"),
              fname, os_strerror(error));
        // Try writing the file even if opening it emerged any issues besides
        // file not existing: maybe writing will succeed nevertheless.
      }
      nomerge = true;
      goto shada_write_file_nomerge;
    } else {
      did_open_reader = true;
    }
    tempname = modname(fname, ".tmp.a", false);
    if (tempname == NULL) {
      nomerge = true;
      goto shada_write_file_nomerge;
    }

    // Save permissions from the original file, with modifications:
    int perm = (int)os_getperm(fname);
    perm = (perm >= 0) ? ((perm & 0777) | 0600) : 0600;
    //                 ^3         ^1       ^2      ^2,3
    // 1: Strip SUID bit if any.
    // 2: Make sure that user can always read and write the result.
    // 3: If somebody happened to delete the file after it was opened for
    //    reading use u=rw permissions.
shada_write_file_open: {}
    error = file_open(&sd_writer, tempname, kFileCreateOnly|kFileNoSymlink, perm);
    if (error) {
      if (error == UV_EEXIST || error == UV_ELOOP) {
        // File already exists, try another name
        char *const wp = tempname + strlen(tempname) - 1;
        if (*wp == 'z') {
          // Tried names from .tmp.a to .tmp.z, all failed. Something must be
          // wrong then.
          semsg(_("E138: All %s.tmp.X files exist, cannot write ShaDa file!"),
                fname);
          xfree(fname);
          xfree(tempname);
          if (did_open_reader) {
            rs_close_file(&sd_reader);
          }
          return FAIL;
        }
        (*wp)++;
        goto shada_write_file_open;
      } else {
        semsg(_(SERR "System error while opening temporary ShaDa file %s "
                "for writing: %s"), tempname, os_strerror(error));
      }
    } else {
      did_open_writer = true;
    }
  }
  if (nomerge) {
shada_write_file_nomerge: {}
    char *const tail = path_tail_with_sep(fname);
    if (tail != fname) {
      const char tail_save = *tail;
      *tail = NUL;
      if (!os_isdir(fname)) {
        int ret;
        char *failed_dir;
        if ((ret = os_mkdir_recurse(fname, 0700, &failed_dir, NULL)) != 0) {
          semsg(_(SERR "Failed to create directory %s "
                  "for writing ShaDa file: %s"),
                failed_dir, os_strerror(ret));
          xfree(fname);
          xfree(failed_dir);
          return FAIL;
        }
      }
      *tail = tail_save;
    }
    int error = file_open(&sd_writer, fname, kFileCreate|kFileTruncate, 0600);
    if (error) {
      semsg(_(SERR "System error while opening ShaDa file %s for writing: %s"),
            fname, os_strerror(error));
    } else {
      did_open_writer = true;
    }
  }

  if (!did_open_writer) {
    xfree(fname);
    xfree(tempname);
    if (did_open_reader) {
      rs_close_file(&sd_reader);
    }
    return FAIL;
  }

  if (p_verbose > 1) {
    verbose_enter();
    smsg(0, _("Writing ShaDa file \"%s\""), fname);
    verbose_leave();
  }

  const ShaDaWriteResult sw_ret = shada_write(&sd_writer, (nomerge ? NULL : &sd_reader));
  assert(sw_ret != kSDWriteIgnError);
  if (!nomerge) {
    if (did_open_reader) {
      rs_close_file(&sd_reader);
    }
    bool did_remove = false;
    if (sw_ret == kSDWriteSuccessful) {
      FileInfo old_info;
      if (!os_fileinfo(fname, &old_info)
          || S_ISDIR(old_info.stat.st_mode)
#ifdef UNIX
          // For Unix we check the owner of the file.  It's not very nice
          // to overwrite a user's viminfo file after a "su root", with a
          // viminfo file that the user can't read.
          || (getuid() != ROOT_UID
              && !(old_info.stat.st_uid == getuid()
                   ? (old_info.stat.st_mode & 0200)
                   : (old_info.stat.st_gid == getgid()
                      ? (old_info.stat.st_mode & 0020)
                      : (old_info.stat.st_mode & 0002))))
#endif
          ) {
        semsg(_("E137: ShaDa file is not writable: %s"), fname);
        goto shada_write_file_did_not_remove;
      }
#ifdef UNIX
      if (getuid() == ROOT_UID) {
        if (old_info.stat.st_uid != ROOT_UID
            || old_info.stat.st_gid != getgid()) {
          const uv_uid_t old_uid = (uv_uid_t)old_info.stat.st_uid;
          const uv_gid_t old_gid = (uv_gid_t)old_info.stat.st_gid;
          const int fchown_ret = os_fchown(file_fd(&sd_writer),
                                           old_uid, old_gid);
          if (fchown_ret != 0) {
            semsg(_(RNERR "Failed setting uid and gid for file %s: %s"),
                  tempname, os_strerror(fchown_ret));
            goto shada_write_file_did_not_remove;
          }
        }
      }
#endif
      if (vim_rename(tempname, fname) == -1) {
        semsg(_(RNERR "Can't rename ShaDa file from %s to %s!"),
              tempname, fname);
      } else {
        did_remove = true;
        os_remove(tempname);
      }
    } else {
      if (sw_ret == kSDWriteReadNotShada) {
        semsg(_(RNERR "Did not rename %s because %s "
                "does not look like a ShaDa file"), tempname, fname);
      } else {
        semsg(_(RNERR "Did not rename %s to %s because there were errors "
                "during writing it"), tempname, fname);
      }
    }
    if (!did_remove) {
shada_write_file_did_not_remove:
      semsg(_(RNERR "Do not forget to remove %s or rename it manually to %s."),
            tempname, fname);
    }
    xfree(tempname);
  }
  rs_close_file(&sd_writer);

  xfree(fname);
  return OK;
}

static void shada_free_shada_entry(ShadaEntry *const entry)
{
  if (entry == NULL || !entry->can_free_entry) {
    return;
  }
  switch (entry->type) {
  case kSDItemMissing:
    break;
  case kSDItemUnknown:
    xfree(entry->data.unknown_item.contents);
    break;
  case kSDItemHeader:
    api_free_dict(entry->data.header);
    break;
  case kSDItemChange:
  case kSDItemJump:
  case kSDItemGlobalMark:
  case kSDItemLocalMark:
    xfree(entry->data.filemark.fname);
    break;
  case kSDItemSearchPattern:
    api_free_string(entry->data.search_pattern.pat);
    break;
  case kSDItemRegister:
    for (size_t i = 0; i < entry->data.reg.contents_size; i++) {
      api_free_string(entry->data.reg.contents[i]);
    }
    xfree(entry->data.reg.contents);
    break;
  case kSDItemHistoryEntry:
    xfree(entry->data.history_item.string);
    break;
  case kSDItemVariable:
    xfree(entry->data.global_var.name);
    tv_clear(&entry->data.global_var.value);
    break;
  case kSDItemSubString:
    xfree(entry->data.sub_string.sub);
    break;
  case kSDItemBufferList:
    for (size_t i = 0; i < entry->data.buffer_list.size; i++) {
      xfree(entry->data.buffer_list.buffers[i].fname);
      xfree(entry->data.buffer_list.buffers[i].additional_data);
    }
    xfree(entry->data.buffer_list.buffers);
    break;
  }
  XFREE_CLEAR(entry->additional_data);
}

#ifndef HAVE_BE64TOH
// Use Rust implementation for byte order conversion
static inline uint64_t vim_be64toh(uint64_t big_endian_64_bits)
{
  return rs_vim_be64toh(big_endian_64_bits);
}
# define be64toh vim_be64toh
#endif

/// Read given number of bytes into given buffer, display error if needed
///
/// @param[in]   sd_reader  Structure containing file reader definition.
/// @param[out]  buffer     Where to save the results.
/// @param[in]   length     How many bytes should be read.
///
/// @return kSDReadStatusSuccess if everything was OK, kSDReadStatusNotShaDa if
///         there were not enough bytes to read or kSDReadStatusReadError if
///         there was some error while reading.
static ShaDaReadResult fread_len(FileDescriptor *const sd_reader, char *const buffer,
                                 const size_t length)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  const ptrdiff_t read_bytes = file_read(sd_reader, buffer, length);
  if (read_bytes < 0) {
    semsg(_(SERR "System error while reading ShaDa file: %s"),
          os_strerror((int)read_bytes));
    return kSDReadStatusReadError;
  }

  if (read_bytes != (ptrdiff_t)length) {
    semsg(_(RCERR "Error while reading ShaDa file: "
            "last entry specified that it occupies %" PRIu64 " bytes, "
            "but file ended earlier"),
          (uint64_t)length);
    return kSDReadStatusNotShaDa;
  }
  return kSDReadStatusSuccess;
}

/// Read next unsigned integer from file
///
/// Errors out if the result is not an unsigned integer.
///
/// Unlike msgpack own function this one works with `FILE *` and reads *exactly*
/// as much bytes as needed, making it possible to avoid both maintaining own
/// buffer and calling `fseek`.
///
/// One byte from file stream is always consumed, even if it is not correct.
///
/// @param[in]   sd_reader  Structure containing file reader definition.
/// @param[out]  result     Location where result is saved.
///
/// @return kSDReadStatusSuccess if reading was successful,
///         kSDReadStatusNotShaDa if there were not enough bytes to read or
///         kSDReadStatusReadError if reading failed for whatever reason.
///         kSDReadStatusFinished if eof and that was allowed
static ShaDaReadResult msgpack_read_uint64(FileDescriptor *const sd_reader, bool allow_eof,
                                           uint64_t *const result)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  const uintmax_t fpos = sd_reader->bytes_read;

  uint8_t ret;
  ptrdiff_t read_bytes = file_read(sd_reader, (char *)&ret, 1);

  if (read_bytes < 0) {
    semsg(_(SERR "System error while reading integer from ShaDa file: %s"),
          os_strerror((int)read_bytes));
    return kSDReadStatusReadError;
  } else if (read_bytes == 0) {
    if (allow_eof && file_eof(sd_reader)) {
      return kSDReadStatusFinished;
    }
    semsg(_(RCERR "Error while reading ShaDa file: "
            "expected positive integer at position %" PRIu64
            ", but got nothing"),
          (uint64_t)fpos);
    return kSDReadStatusNotShaDa;
  }

  int first_char = (int)ret;
  if (~first_char & 0x80) {
    // Positive fixnum
    *result = (uint64_t)((uint8_t)first_char);
  } else {
    size_t length = 0;
    switch (first_char) {
    case 0xCC:    // uint8
      length = 1;
      break;
    case 0xCD:    // uint16
      length = 2;
      break;
    case 0xCE:    // uint32
      length = 4;
      break;
    case 0xCF:    // uint64
      length = 8;
      break;
    default:
      semsg(_(RCERR "Error while reading ShaDa file: "
              "expected positive integer at position %" PRIu64),
            (uint64_t)fpos);
      return kSDReadStatusNotShaDa;
    }
    uint64_t buf = 0;
    char *buf_u8 = (char *)&buf;
    ShaDaReadResult fl_ret;
    if ((fl_ret = fread_len(sd_reader, &(buf_u8[sizeof(buf) - length]), length))
        != kSDReadStatusSuccess) {
      return fl_ret;
    }
    *result = be64toh(buf);
  }
  return kSDReadStatusSuccess;
}

#define READERR(entry_name, error_desc) \
  RERR "Error while reading ShaDa file: " \
  entry_name " entry at position %" PRIu64 " " \
  error_desc

/// Iterate over shada file contents
///
/// @param[in]   sd_reader  Structure containing file reader definition.
/// @param[out]  entry      Address where next entry contents will be saved.
/// @param[in]   flags      Flags, determining whether and which items should be
///                         skipped (see SRNIFlags enum).
/// @param[in]   max_kbyte  If non-zero, skip reading entries which have length
///                         greater then given.
///
/// @return Any value from ShaDaReadResult enum.
static ShaDaReadResult shada_read_next_item(FileDescriptor *const sd_reader,
                                            ShadaEntry *const entry, const unsigned flags,
                                            const size_t max_kbyte)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  ShaDaReadResult ret = kSDReadStatusMalformed;
shada_read_next_item_start:
  // Set entry type to kSDItemMissing and also make sure that all pointers in
  // data union are NULL so they are safe to xfree(). This is needed in case
  // somebody calls goto shada_read_next_item_error before anything is set in
  // the switch.
  CLEAR_POINTER(entry);
  if (file_eof(sd_reader)) {
    return kSDReadStatusFinished;
  }

  bool verify_but_ignore = false;

  // First: manually unpack type, timestamp and length.
  // This is needed to avoid both seeking and having to maintain a buffer.
  uint64_t type_u64 = (uint64_t)kSDItemMissing;
  uint64_t timestamp_u64;
  uint64_t length_u64;

  const uint64_t initial_fpos = sd_reader->bytes_read;
  AdditionalDataBuilder ad = KV_INITIAL_VALUE;
  uint32_t read_additional_array_elements = 0;
  char *error_alloc = NULL;

  ShaDaReadResult mru_ret;
  if (((mru_ret = msgpack_read_uint64(sd_reader, true, &type_u64))
       != kSDReadStatusSuccess)
      || ((mru_ret = msgpack_read_uint64(sd_reader, false,
                                         &timestamp_u64))
          != kSDReadStatusSuccess)
      || ((mru_ret = msgpack_read_uint64(sd_reader, false,
                                         &length_u64))
          != kSDReadStatusSuccess)) {
    return mru_ret;
  }

  if (length_u64 > PTRDIFF_MAX) {
    semsg(_(RCERR "Error while reading ShaDa file: "
            "there is an item at position %" PRIu64 " "
            "that is stated to be too long"),
          initial_fpos);
    return kSDReadStatusNotShaDa;
  }

  const size_t length = (size_t)length_u64;
  entry->timestamp = (Timestamp)timestamp_u64;
  entry->can_free_entry = true;  // all allocations are owned by the entry

  if (type_u64 == 0) {
    // kSDItemUnknown cannot possibly pass that far because it is -1 and that
    // will fail in msgpack_read_uint64. But kSDItemMissing may and it will
    // otherwise be skipped because (1 << 0) will never appear in flags.
    semsg(_(RCERR "Error while reading ShaDa file: "
            "there is an item at position %" PRIu64 " "
            "that must not be there: Missing items are "
            "for internal uses only"),
          initial_fpos);
    return kSDReadStatusNotShaDa;
  }

  if ((type_u64 > SHADA_LAST_ENTRY
       ? !(flags & kSDReadUnknown)
       : !((unsigned)(1 << type_u64) & flags))
      || (max_kbyte && length > max_kbyte * 1024)) {
    // First entry is unknown or equal to "\n" (10)? Most likely this means that
    // current file is not a ShaDa file because first item should normally be
    // a header (excluding tests where first item is tested item). Check this by
    // parsing entry contents: in non-ShaDa files this will most likely result
    // in incomplete MessagePack string.
    if (initial_fpos == 0
        && (type_u64 == '\n' || type_u64 > SHADA_LAST_ENTRY)) {
      verify_but_ignore = true;
    } else {
      const ShaDaReadResult srs_ret = sd_reader_skip(sd_reader, length);
      if (srs_ret != kSDReadStatusSuccess) {
        return srs_ret;
      }
      goto shada_read_next_item_start;
    }
  }

  const uint64_t parse_pos = sd_reader->bytes_read;
  bool buf_allocated = false;
  // try to avoid allocation for small items which fits entirely
  // in the internal buffer of sd_reader
  char *buf = file_try_read_buffered(sd_reader, length);
  if (!buf) {
    buf_allocated = true;
    buf = xmalloc(length);
    const ShaDaReadResult fl_ret = fread_len(sd_reader, buf, length);
    if (fl_ret != kSDReadStatusSuccess) {
      ret = fl_ret;
      goto shada_read_next_item_error;
    }
  }

  const char *read_ptr = buf;
  size_t read_size = length;

  if (verify_but_ignore) {
    int status = unpack_skip(&read_ptr, &read_size);
    ShaDaReadResult spm_ret = shada_check_status(parse_pos, status, read_size);
    if (buf_allocated) {
      xfree(buf);
    }
    if (spm_ret != kSDReadStatusSuccess) {
      return spm_ret;
    }
    goto shada_read_next_item_start;
  }

  if (type_u64 > SHADA_LAST_ENTRY) {
    entry->type = kSDItemUnknown;
    entry->data.unknown_item.size = length;
    entry->data.unknown_item.type = type_u64;
    if (initial_fpos == 0) {
      int status = unpack_skip(&read_ptr, &read_size);
      ShaDaReadResult spm_ret = shada_check_status(parse_pos, status, read_size);
      if (spm_ret != kSDReadStatusSuccess) {
        if (buf_allocated) {
          xfree(buf);
        }
        entry->type = kSDItemMissing;
        return spm_ret;
      }
    }
    entry->data.unknown_item.contents = buf_allocated ? buf : xmemdup(buf, length);
    return kSDReadStatusSuccess;
  }

  entry->data = sd_default_values[type_u64].data;
  switch ((ShadaEntryType)type_u64) {
  case kSDItemHeader:
    // TODO(bfredl): header is written to file and provides useful debugging
    // info. It is never read by nvim (earlier we parsed it back to a
    // Dict, but that value was never used)
    break;
  case kSDItemSearchPattern: {
    Dict(_shada_search_pat) *it = &entry->data.search_pattern;
    if (!unpack_keydict(it, DictHash(_shada_search_pat), &ad, &read_ptr, &read_size,
                        &error_alloc)) {
      semsg(_(READERR("search pattern", "%s")), initial_fpos, error_alloc);
      it->pat = NULL_STRING;
      goto shada_read_next_item_error;
    }

    if (!HAS_KEY(it, _shada_search_pat, sp)) {  // SEARCH_KEY_PAT
      semsg(_(READERR("search pattern", "has no pattern")), initial_fpos);
      goto shada_read_next_item_error;
    }
    entry->data.search_pattern.pat = copy_string(entry->data.search_pattern.pat, NULL);

    break;
  }
  case kSDItemChange:
  case kSDItemJump:
  case kSDItemGlobalMark:
  case kSDItemLocalMark: {
    Dict(_shada_mark) it = { 0 };
    if (!unpack_keydict(&it, DictHash(_shada_mark), &ad, &read_ptr, &read_size, &error_alloc)) {
      semsg(_(READERR("mark", "%s")), initial_fpos, error_alloc);
      goto shada_read_next_item_error;
    }

    if (HAS_KEY(&it, _shada_mark, n)) {
      if (type_u64 == kSDItemJump || type_u64 == kSDItemChange) {
        semsg(_(READERR("mark", "has n key which is only valid for "
                        "local and global mark entries")), initial_fpos);
        goto shada_read_next_item_error;
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
      goto shada_read_next_item_error;
    }
    if (entry->data.filemark.mark.lnum <= 0) {
      semsg(_(READERR("mark", "has invalid line number")), initial_fpos);
      goto shada_read_next_item_error;
    }
    if (entry->data.filemark.mark.col < 0) {
      semsg(_(READERR("mark", "has invalid column number")), initial_fpos);
      goto shada_read_next_item_error;
    }
    break;
  }
  case kSDItemRegister: {
    Dict(_shada_register) it = { 0 };
    if (!unpack_keydict(&it, DictHash(_shada_register), &ad, &read_ptr, &read_size, &error_alloc)) {
      semsg(_(READERR("register", "%s")), initial_fpos, error_alloc);
      kv_destroy(it.rc);
      goto shada_read_next_item_error;
    }
    if (it.rc.size == 0) {
      semsg(_(READERR("register",
                      "has " KEY_NAME(REG_KEY_CONTENTS) " key with missing or empty array")),
            initial_fpos);
      goto shada_read_next_item_error;
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
    break;
  }
  case kSDItemHistoryEntry: {
    ssize_t len = unpack_array(&read_ptr, &read_size);

    if (len < 2) {
      semsg(_(READERR("history", "is not an array with enough elements")), initial_fpos);
      goto shada_read_next_item_error;
    }
    Integer hist_type;
    if (!unpack_integer(&read_ptr, &read_size, &hist_type)) {
      semsg(_(READERR("history", "has wrong history type type")), initial_fpos);
      goto shada_read_next_item_error;
    }
    const String item = unpack_string(&read_ptr, &read_size);
    if (!item.data) {
      semsg(_(READERR("history", "has wrong history string type")), initial_fpos);
      goto shada_read_next_item_error;
    }
    if (memchr(item.data, 0, item.size) != NULL) {
      semsg(_(READERR("history", "contains string with zero byte inside")), initial_fpos);
      goto shada_read_next_item_error;
    }
    entry->data.history_item.histtype = (uint8_t)hist_type;
    const bool is_hist_search = entry->data.history_item.histtype == HIST_SEARCH;
    if (is_hist_search) {
      if (len < 3) {
        semsg(_(READERR("search history",
                        "does not have separator character")), initial_fpos);
        goto shada_read_next_item_error;
      }
      Integer sep_type;
      if (!unpack_integer(&read_ptr, &read_size, &sep_type)) {
        semsg(_(READERR("search history", "has wrong history separator type")), initial_fpos);
        goto shada_read_next_item_error;
      }
      entry->data.history_item.sep = (char)sep_type;
    }
    size_t strsize = (item.size
                      + 1  // Zero byte
                      + 1);  // Separator character
    entry->data.history_item.string = xmalloc(strsize);
    memcpy(entry->data.history_item.string, item.data, item.size);
    entry->data.history_item.string[strsize - 2] = 0;
    entry->data.history_item.string[strsize - 1] = entry->data.history_item.sep;
    read_additional_array_elements = (uint32_t)(len - (2 + is_hist_search));
    break;
  }
  case kSDItemVariable: {
    ssize_t len = unpack_array(&read_ptr, &read_size);

    if (len < 2) {
      semsg(_(READERR("variable", "is not an array with enough elements")), initial_fpos);
      goto shada_read_next_item_error;
    }

    String name = unpack_string(&read_ptr, &read_size);

    if (!name.data) {
      semsg(_(READERR("variable", "has wrong variable name type")), initial_fpos);
      goto shada_read_next_item_error;
    }
    entry->data.global_var.name = xmemdupz(name.data, name.size);

    String binval = unpack_string(&read_ptr, &read_size);

    bool is_blob = false;
    if (binval.data) {
      if (len > 2) {
        // A msgpack BIN could be a String or Blob; an additional VAR_TYPE_BLOB
        // element is stored with Blobs which can be used to differentiate them
        Integer type;
        if (!unpack_integer(&read_ptr, &read_size, &type) || type != VAR_TYPE_BLOB) {
          semsg(_(READERR("variable", "has wrong variable type")),
                initial_fpos);
          goto shada_read_next_item_error;
        }
        is_blob = true;
      }
      entry->data.global_var.value = decode_string(binval.data, binval.size, is_blob, false);
    } else {
      int status = unpack_typval(&read_ptr, &read_size, &entry->data.global_var.value);
      if (status != MPACK_OK) {
        semsg(_(READERR("variable", "has value that cannot "
                        "be converted to the Vimscript value")), initial_fpos);
        goto shada_read_next_item_error;
      }
    }
    read_additional_array_elements = (uint32_t)(len - 2 - (is_blob ? 1 : 0));
    break;
  }
  case kSDItemSubString: {
    ssize_t len = unpack_array(&read_ptr, &read_size);

    if (len < 1) {
      semsg(_(READERR("sub string", "is not an array with enough elements")), initial_fpos);
      goto shada_read_next_item_error;
    }

    String sub = unpack_string(&read_ptr, &read_size);
    if (!sub.data) {
      semsg(_(READERR("sub string", "has wrong sub string type")), initial_fpos);
      goto shada_read_next_item_error;
    }
    entry->data.sub_string.sub = xmemdupz(sub.data, sub.size);
    read_additional_array_elements = (uint32_t)(len - 1);
    break;
  }
  case kSDItemBufferList: {
    ssize_t len = unpack_array(&read_ptr, &read_size);
    if (len < 0) {
      semsg(_(READERR("buffer list", "is not an array")), initial_fpos);
      goto shada_read_next_item_error;
    }
    if (len == 0) {
      break;
    }
    entry->data.buffer_list.buffers = xcalloc((size_t)len,
                                              sizeof(*entry->data.buffer_list.buffers));
    for (size_t i = 0; i < (size_t)len; i++) {
      entry->data.buffer_list.size++;
      Dict(_shada_buflist_item) it = { 0 };
      AdditionalDataBuilder it_ad = KV_INITIAL_VALUE;
      if (!unpack_keydict(&it, DictHash(_shada_buflist_item), &it_ad, &read_ptr, &read_size,
                          &error_alloc)) {
        semsg(_(RERR "Error while reading ShaDa file: "
                "buffer list at position %" PRIu64 " contains entry that %s"),
              initial_fpos, error_alloc);
        kv_destroy(it_ad);
        goto shada_read_next_item_error;
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
                "contains entry with invalid line number"),
              initial_fpos);
        goto shada_read_next_item_error;
      }
      if (e->pos.col < 0) {
        semsg(_(RERR "Error while reading ShaDa file: "
                "buffer list at position %" PRIu64 " "
                "contains entry with invalid column number"),
              initial_fpos);
        goto shada_read_next_item_error;
      }
      if (e->fname == NULL) {
        semsg(_(RERR "Error while reading ShaDa file: "
                "buffer list at position %" PRIu64 " "
                "contains entry that does not have a file name"),
              initial_fpos);
        goto shada_read_next_item_error;
      }
    }
    break;
  }
  case kSDItemMissing:
  case kSDItemUnknown:
    abort();
  }

  for (uint32_t i = 0; i < read_additional_array_elements; i++) {
    const char *item_start = read_ptr;
    int status = unpack_skip(&read_ptr, &read_size);
    if (status) {
      goto shada_read_next_item_error;
    }

    push_additional_data(&ad, item_start, (size_t)(read_ptr - item_start));
  }

  if (read_size) {
    semsg(_(READERR("item", "additional bytes")), initial_fpos);
    goto shada_read_next_item_error;
  }

  entry->type = (ShadaEntryType)type_u64;
  entry->additional_data = (AdditionalData *)ad.items;
  ret = kSDReadStatusSuccess;
shada_read_next_item_end:
  if (buf_allocated) {
    xfree(buf);
  }
  return ret;
shada_read_next_item_error:
  entry->type = (ShadaEntryType)type_u64;
  shada_free_shada_entry(entry);
  entry->type = kSDItemMissing;
  xfree(error_alloc);
  kv_destroy(ad);
  goto shada_read_next_item_end;
}

/// Write registers ShaDa entries in given msgpack_sbuffer.
///
/// @param[in]  sbuf  target msgpack_sbuffer to write to.
String shada_encode_regs(void)
  FUNC_ATTR_NONNULL_ALL
{
  WriteMergerState *const wms = xcalloc(1, sizeof(*wms));
  rs_shada_initialize_registers(wms, -1);
  PackerBuffer packer = packer_string_buffer();
  for (size_t i = 0; i < ARRAY_SIZE(wms->registers); i++) {
    if (wms->registers[i].type == kSDItemRegister) {
      if (kSDWriteFailed
          == shada_pack_pfreed_entry(&packer, wms->registers[i], 0)) {
        abort();
      }
    }
  }
  xfree(wms);
  return packer_take_string(&packer);
}

/// Write jumplist ShaDa entries in given msgpack_sbuffer.
///
/// @param[in]  sbuf            target msgpack_sbuffer to write to.
String shada_encode_jumps(void)
  FUNC_ATTR_NONNULL_ALL
{
  Set(ptr_t) removable_bufs = SET_INIT;
  rs_find_removable_bufs(&removable_bufs);
  ShadaEntry jumps[JUMPLISTSIZE];
  size_t jumps_size = rs_shada_init_jumps(jumps, &removable_bufs);
  PackerBuffer packer = packer_string_buffer();
  for (size_t i = 0; i < jumps_size; i++) {
    if (kSDWriteFailed == shada_pack_pfreed_entry(&packer, jumps[i], 0)) {
      abort();
    }
  }
  return packer_take_string(&packer);
}

/// Write buffer list ShaDa entry in given msgpack_sbuffer.
///
/// @param[in]  sbuf            target msgpack_sbuffer to write to.
String shada_encode_buflist(void)
  FUNC_ATTR_NONNULL_ALL
{
  Set(ptr_t) removable_bufs = SET_INIT;
  rs_find_removable_bufs(&removable_bufs);
  ShadaEntry buflist_entry = rs_shada_get_buflist(&removable_bufs);

  PackerBuffer packer = packer_string_buffer();
  if (kSDWriteFailed == shada_pack_entry(&packer, buflist_entry, 0)) {
    abort();
  }
  xfree(buflist_entry.data.buffer_list.buffers);
  return packer_take_string(&packer);
}

/// Write global variables ShaDa entries in given msgpack_sbuffer.
///
/// @param[in]  sbuf            target msgpack_sbuffer to write to.
String shada_encode_gvars(void)
  FUNC_ATTR_NONNULL_ALL
{
  PackerBuffer packer = packer_string_buffer();
  const void *var_iter = NULL;
  const Timestamp cur_timestamp = os_time();
  do {
    typval_T vartv;
    const char *name = NULL;
    var_iter = var_shada_iter(var_iter, &name, &vartv,
                              VAR_FLAVOUR_DEFAULT | VAR_FLAVOUR_SESSION | VAR_FLAVOUR_SHADA);
    if (name == NULL) {
      break;
    }
    if (vartv.v_type != VAR_FUNC && vartv.v_type != VAR_PARTIAL) {
      typval_T tgttv;
      tv_copy(&vartv, &tgttv);
      ShaDaWriteResult r = shada_pack_entry(&packer, (ShadaEntry) {
        .type = kSDItemVariable,
        .timestamp = cur_timestamp,
        .data = {
          .global_var = {
            .name = (char *)name,
            .value = tgttv,
          }
        },
        .additional_data = NULL,
      }, 0);
      if (kSDWriteFailed == r) {
        abort();
      }
      tv_clear(&tgttv);
    }
    tv_clear(&vartv);
  } while (var_iter != NULL);
  return packer_take_string(&packer);
}

/// Read ShaDa from String.
///
/// @param[in]  string   string to read from.
/// @param[in]  flags  Flags, see ShaDaReadFileFlags enum.
void shada_read_string(String string, const int flags)
  FUNC_ATTR_NONNULL_ALL
{
  if (string.size == 0) {
    return;
  }
  FileDescriptor sd_reader;
  file_open_buffer(&sd_reader, string.data, string.size);
  shada_read(&sd_reader, flags);
  rs_close_file(&sd_reader);
}

/// Find the parameter represented by the given character (eg ', :, ", or /),
/// and return its associated value in the 'shada' string.
/// Only works for number parameters, not for 'r' or 'n'.
/// If the parameter is not specified in the string or there is no following
/// number, return -1.
int get_shada_parameter(int type)
{
  char *p = find_shada_parameter(type);
  if (p != NULL && ascii_isdigit(*p)) {
    return atoi(p);
  }
  return -1;
}

/// Find the parameter represented by the given character (eg ''', ':', '"', or
/// '/') in the 'shada' option and return a pointer to the string after it.
/// Return NULL if the parameter is not specified in the string.
char *find_shada_parameter(int type)
{
  for (char *p = p_shada; *p; p++) {
    if (*p == type) {
      return p + 1;
    }
    if (*p == 'n') {                // 'n' is always the last one
      break;
    }
    p = vim_strchr(p, ',');         // skip until next ','
    if (p == NULL) {                // hit the end without finding parameter
      break;
    }
  }
  return NULL;
}

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

/// Read a string from shada data
void nvim_shada_read_string(String string, int flags)
{
  shada_read_string(string, flags);
}

// Wrapper functions for existing shada functions (nvim_ prefix for Rust FFI)
int nvim_get_shada_parameter(int type) { return get_shada_parameter(type); }
char *nvim_find_shada_parameter(int type) { return find_shada_parameter(type); }

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

// Wrapper for shada_free_shada_entry — delegates to Rust rs_shada_free_entry_contents
// which handles all entry types (delegating back to C for Variable and Register
// due to struct layout differences).
extern void rs_shada_free_entry_contents(ShadaEntry *entry);
void nvim_shada_free_shada_entry(ShadaEntry *entry)
{
  rs_shada_free_entry_contents(entry);
}

// Accessor for rs_shada_hist_iter
const void *nvim_shada_hist_iter(const void *iter, uint8_t history_type,
                                  int reading, ShadaEntry *out_entry)
{
  return rs_shada_hist_iter(iter, history_type, reading, out_entry);
}

// Accessor for rs_shada_get_default_file
const char *nvim_shada_get_default_file(void)
{
  return rs_shada_get_default_file();
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

// Shada encode wrappers (forward declarations)
extern String shada_encode_regs(void);
extern String shada_encode_jumps(void);
extern String shada_encode_buflist(void);
extern String shada_encode_gvars(void);

String nvim_shada_encode_regs(void) { return shada_encode_regs(); }
String nvim_shada_encode_jumps(void) { return shada_encode_jumps(); }
String nvim_shada_encode_buflist(void) { return shada_encode_buflist(); }
String nvim_shada_encode_gvars(void) { return shada_encode_gvars(); }

// Phase 1: FileMarks accessor for Rust FFI
Timestamp nvim_filemarks_get_greatest_timestamp(const void *fm_ptr)
{
  const FileMarks *fm = fm_ptr;
  return fm ? fm->greatest_timestamp : 0;
}

// Phase 2: Buffer/path filtering accessors for Rust FFI
const char *nvim_shada_get_p_shada(void) { return p_shada; }
char *nvim_shada_home_replace_save(const void *buf, const char *src)
{
  return home_replace_save((buf_T *)buf, src);
}
void nvim_shada_home_replace(const void *buf, const char *src, char *dst, size_t dstlen, int one)
{
  home_replace((buf_T *)buf, src, dst, dstlen, one != 0);
}
size_t nvim_shada_copy_option_part(char **option, char *buf, size_t maxlen, const char *sep_chars)
{
  return copy_option_part(option, buf, maxlen, (char *)sep_chars);
}
int nvim_shada_mb_strnicmp(const char *s1, const char *s2, size_t n)
{
  return mb_strnicmp(s1, s2, n);
}
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
int nvim_shada_buf_is_listed(const void *buf)
{
  return buf ? ((const buf_T *)buf)->b_p_bl : 0;
}
int nvim_shada_buf_is_quickfix(const void *buf)
{
  return buf ? bt_quickfix((const buf_T *)buf) : 0;
}
int nvim_shada_buf_is_terminal(const void *buf)
{
  return buf ? bt_terminal((const buf_T *)buf) : 0;
}

// Set(ptr_t) operations for Rust FFI
void *nvim_shada_set_init_ptr(void)
{
  Set(ptr_t) *s = xcalloc(1, sizeof(Set(ptr_t)));
  *s = (Set(ptr_t))SET_INIT;
  return s;
}
int nvim_shada_set_has_ptr(const void *set, const void *ptr)
{
  return set ? set_has(ptr_t, (Set(ptr_t) *)set, (ptr_t)ptr) : 0;
}
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

// Phase 3: Data collection accessors for Rust FFI

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
int nvim_shada_get_percent_param(void) { return get_shada_parameter('%'); }
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

// Phase 4: FFI wrappers for shada_free_shada_entry consolidation

/// Free a Dict (api_free_dict wrapper)
void nvim_shada_api_free_dict(Dict value)
{
  api_free_dict(value);
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

// Phase 5: File I/O wrappers

/// Open a file for reading. Returns 0 on success, error code on failure.
int nvim_shada_file_open(void *fd, const char *fname)
{
  return file_open((FileDescriptor *)fd, fname, kFileReadOnly, 0);
}

/// Read shada data from an open file descriptor.
void nvim_shada_read(void *fd, int flags)
{
  shada_read((FileDescriptor *)fd, flags);
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
int nvim_shada_get_p_verbose(void)
{
  return (int)p_verbose;
}

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
int nvim_shada_get_p_fs(void)
{
  return !!p_fs;
}

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
size_t nvim_shada_file_descriptor_size(void)
{
  return sizeof(FileDescriptor);
}

// Phase 6: curbuf accessors for check_marks_read

int nvim_shada_curbuf_marks_read(void)
{
  return curbuf->b_marks_read;
}

void nvim_shada_curbuf_set_marks_read(int val)
{
  curbuf->b_marks_read = val;
}

const char *nvim_shada_curbuf_ffname(void)
{
  return curbuf->b_ffname;
}

// Phase 7: histentry_T accessor for hms_to_he_array

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
