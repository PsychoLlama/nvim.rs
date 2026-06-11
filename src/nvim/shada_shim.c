#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <uv.h>
#include "mpack/mpack_core.h"
#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/buffer.h"
#include "nvim/cmdhist.h"
#include "nvim/eval.h"
#include "nvim/eval/decode.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/vars.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/mark.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/msgpack_rpc/packer.h"
#include "nvim/option_vars.h"
#include "nvim/os/fileio.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/path.h"
#include "nvim/register.h"
#include "nvim/search.h"

#include "nvim/shada.h"
#include "shada_shim.c.generated.h"

extern int rs_marklist_insert(void *jumps_arr, size_t jump_size, int jl_len, int i);
extern int rs_compare_file_marks(const void *a, const void *b);
extern var_flavour_T rs_var_flavour(const char *varname);

void nvim_shada_smsg_1s(const char *fmt, const char *arg) { smsg(0, fmt, arg); }

typedef enum {
  kSDItemUnknown = -1,
  kSDItemMissing = 0,
  kSDItemHeader = 1,
  kSDItemSearchPattern = 2,
  kSDItemSubString = 3,
  kSDItemHistoryEntry = 4,
  kSDItemRegister = 5,
  kSDItemVariable = 6,
  kSDItemGlobalMark = 7,
  kSDItemJump = 8,
  kSDItemBufferList = 9,
  kSDItemLocalMark = 10,
  kSDItemChange = 11,
} ShadaEntryType;

// Convert a Rust-layout position (int64_t lnum) to pos_T (int32_t lnum).
#define RS_POS_TO_POST(p) \
  ((pos_T){ .lnum = (linenr_T)(p).lnum, .col = (colnr_T)(p).col, \
            .coladd = (colnr_T)(p).coladd })

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
      // Rust FilemarkData uses Position { lnum: i64, col: i32, coladd: i32 },
      // which is 16 bytes (vs pos_T's 12). The C layout must match exactly so
      // that fname lands at the correct offset (24, not 16).
      struct { int64_t lnum; int32_t col; int32_t coladd; } mark;
      char *fname;
    } filemark;
    Dict(_shada_search_pat) search_pattern;
    struct history_item {
      uint8_t histtype;
      char *string;
      char sep;
    } history_item;
    struct reg {  // RegisterData (must match Rust layout exactly)
      char name;
      int type;  // MotionType — same underlying int, same offset
      // Rust RegisterData uses *mut *mut c_char (thin 8-byte pointers, no inline size).
      // C yankreg_T uses String* (fat 16-byte {data,size} pairs). These are NOT layout-
      // compatible; nvim_shada_op_reg_set_from_entry converts between them.
      char **contents;
      size_t contents_size;  // before is_unnamed (matches Rust field order)
      bool is_unnamed;       // after contents_size (matches Rust field order)
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
        // Matches Rust BufferListBuffer { pos: Position { lnum: i64, col: i32, coladd: i32 } }.
        struct { int64_t lnum; int32_t col; int32_t coladd; } pos;
        char *fname;
        AdditionalData *additional_data;
      } *buffers;
    } buffer_list;
  } data;
  AdditionalData *additional_data;
} ShadaEntry;

typedef struct hm_llist_entry {
  ShadaEntry data;
  struct hm_llist_entry *next;
  struct hm_llist_entry *prev;
} HMLListEntry;

typedef struct {
  HMLListEntry *entries;
  HMLListEntry *first;
  HMLListEntry *last;
  HMLListEntry *free_entry;
  HMLListEntry *last_free_entry;
  size_t size;
  size_t num_entries;
  PMap(cstr_t) contained_entries;
} HMLList;

typedef struct {
  HMLList hmll;
  bool do_merge;
  bool reading;
  const void *iter;
  ShadaEntry last_hist_entry;
  uint8_t history_type;
} HistoryMergerState;

typedef struct {
  HistoryMergerState hms[HIST_COUNT];
  ShadaEntry global_marks[NMARKS];
  ShadaEntry numbered_marks[EXTRA_MARKS];
  ShadaEntry registers[NUM_SAVED_REGISTERS];
  ShadaEntry jumps[JUMPLISTSIZE];
  size_t jumps_size;
  ShadaEntry search_pattern;
  ShadaEntry sub_search_pattern;
  ShadaEntry replacement;
  Set(cstr_t) dumped_variables;
  PMap(cstr_t) file_marks;
} WriteMergerState;

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
void nvim_shada_decode_string_into(const char *s, size_t len, bool force_blob, void *dst) { typval_T tv = decode_string(s, len, force_blob, false); memcpy(dst, &tv, sizeof(typval_T)); }
void nvim_hmll_map_init(PMap(cstr_t) *map) { *map = (PMap(cstr_t)) MAP_INIT; }
void nvim_hmll_map_destroy(PMap(cstr_t) *map) { if (map) { map_destroy(cstr_t, map); } }
void *nvim_hmll_map_get(PMap(cstr_t) *map, const char *key) { return (map && key) ? pmap_get(cstr_t)(map, key) : NULL; }
void nvim_hmll_map_put(PMap(cstr_t) *map, const char *key, void *entry)
{ if (!map || !key) { return; } bool new_item = false; ptr_t *val = pmap_put_ref(cstr_t)(map, key, NULL, &new_item); if (val) { *val = entry; } }
void nvim_hmll_map_del(PMap(cstr_t) *map, const char *key) { if (map && key) { pmap_del(cstr_t)(map, key, NULL); } }
// Deleted: nvim_get_p_hi -- Rust uses extern static p_hi directly
// Deleted: nvim_get_p_shadafile -- Rust uses extern static p_shadafile directly
size_t nvim_expand_env(const char *src, char *dst, size_t dstlen)
{ if (!src || !dst || dstlen == 0) { return 0; } expand_env((char *)src, dst, (int)dstlen); return strlen(dst); }
char *nvim_xmemdupz(const char *s, size_t len) { return xmemdupz(s, len); }
// Deleted: nvim_shada_get_namebuff -- Rust uses extern static NameBuff directly
const void *nvim_shada_buf_next(const void *buf) { return buf ? ((const buf_T *)buf)->b_next : NULL; }
const char *nvim_shada_buf_get_ffname(const void *buf) { return buf ? ((const buf_T *)buf)->b_ffname : NULL; }
int nvim_shada_buf_should_skip(const void *buf) { const buf_T *b = (const buf_T *)buf; return (!b || !b->b_p_bl || bt_quickfix(b) || bt_terminal(b)) ? 1 : 0; }
void *nvim_shada_set_init_ptr(void)
{ Set(ptr_t) *s = xcalloc(1, sizeof(Set(ptr_t))); *s = (Set(ptr_t))SET_INIT; return s; }
int nvim_shada_set_has_ptr(const void *set, const void *ptr) { return set ? set_has(ptr_t, (Set(ptr_t) *)set, (ptr_t)ptr) : 0; }
void nvim_shada_set_put_ptr(void *set, const void *ptr) { if (set) { set_put(ptr_t, (Set(ptr_t) *)set, (ptr_t)ptr); } }
void nvim_shada_set_destroy_ptr(void *set) { if (set) { set_destroy(ptr_t, (Set(ptr_t) *)set); xfree(set); } }
const void *nvim_shada_hist_iter_raw(const void *iter, uint8_t history_type, int zero,
                                     char **out_str, size_t *out_strlen, Timestamp *out_ts,
                                     void **out_additional_data)
{ histentry_T he; const void *ret = hist_iter(iter, history_type, zero != 0, &he);
  *out_str = he.hisstr; *out_strlen = he.hisstrlen; *out_ts = he.timestamp; *out_additional_data = he.additional_data; return ret; }
void nvim_shada_get_search_pattern(int is_substitute, char **out_pat, int *out_magic,
                                   int *out_no_scs, Timestamp *out_ts, int *out_off_line,
                                   int *out_off_end, int64_t *out_off_off, char *out_off_dir,
                                   void **out_additional_data)
{ SearchPattern pat; if (is_substitute) { get_substitute_pattern(&pat); } else { get_search_pattern(&pat); }
  *out_pat = pat.pat; *out_magic = pat.magic; *out_no_scs = pat.no_scs; *out_ts = pat.timestamp;
  *out_off_line = pat.off.line; *out_off_end = pat.off.end; *out_off_off = pat.off.off; *out_off_dir = pat.off.dir; *out_additional_data = pat.additional_data; }
const void *nvim_shada_reg_iter(const void *iter, char *out_name, int *out_type,
                                String **out_contents, size_t *out_size,
                                size_t *out_width, int *out_is_unnamed,
                                Timestamp *out_ts, void **out_additional_data)
{ yankreg_T reg = { 0 }; bool is_unnamed = false; const void *ret = op_global_reg_iter(iter, out_name, &reg, &is_unnamed);
  *out_type = reg.y_type; *out_contents = reg.y_array; *out_size = reg.y_size;
  *out_width = (size_t)(reg.y_type == kMTBlockWise ? reg.y_width : 0);
  *out_is_unnamed = is_unnamed; *out_ts = reg.timestamp; *out_additional_data = reg.additional_data; return ret; }
void nvim_shada_buf_get_buflist_info(const void *buf, pos_T *out_pos, void **out_additional_data)
{ if (buf) { *out_pos = ((const buf_T *)buf)->b_last_cursor.mark; *out_additional_data = ((const buf_T *)buf)->additional_data; } }
const void *nvim_shada_jumplist_iter(const void *iter, void *wp,
                                     pos_T *out_mark, int *out_fnum,
                                     Timestamp *out_ts, char **out_fname,
                                     void **out_additional_data)
{ xfmark_T fm; const void *ret = mark_jumplist_iter(iter, (win_T *)wp, &fm);
  *out_mark = fm.fmark.mark; *out_fnum = fm.fmark.fnum; *out_ts = fm.fmark.timestamp;
  *out_fname = fm.fname; *out_additional_data = fm.fmark.additional_data; return ret; }

void nvim_shada_free_header_entry(ShadaEntry *entry) { api_free_dict(entry->data.header); }
void nvim_shada_free_variable(ShadaEntry *entry) { xfree(entry->data.global_var.name); tv_clear(&entry->data.global_var.value); }
void nvim_shada_smsg_reading(const char *fname, int want_info, int want_marks,
                             int get_oldfiles, int failed)
{ smsg(0, _("Reading ShaDa file \"%s\"%s%s%s%s"), fname,
       want_info ? _(" info") : "", want_marks ? _(" marks") : "",
       get_oldfiles ? _(" oldfiles") : "", failed ? _(" FAILED") : ""); }
// Deleted: nvim_shada_build_default_path -- Rust calls stdpaths_user_state_subpath+concat_fnames_realloc directly
// Deleted: nvim_shada_file_descriptor_size -- Rust uses FILE_DESCRIPTOR_SIZE = 48 constant (verified by _Static_assert below)
_Static_assert(sizeof(FileDescriptor) == 48, "FileDescriptor size changed; update FILE_DESCRIPTOR_SIZE in shada/src/lib.rs");
_Static_assert(offsetof(FileDescriptor, bytes_read) == 40, "FileDescriptor.bytes_read offset changed; update FILE_DESCRIPTOR_BYTES_READ_OFFSET in shada/src/lib.rs");
int nvim_shada_curbuf_marks_read(void) { return curbuf->b_marks_read; }
void nvim_shada_curbuf_set_marks_read(int val) { curbuf->b_marks_read = val; }
const char *nvim_shada_curbuf_ffname(void) { return curbuf->b_ffname; }
void nvim_shada_set_histentry(void *hist_array, int idx, uint64_t ts, char *hisstr, void *additional_data)
{ histentry_T *he = &((histentry_T *)hist_array)[idx]; he->timestamp = ts; he->hisnum = idx + 1; he->hisstr = hisstr; he->hisstrlen = strlen(hisstr); he->additional_data = additional_data; }
size_t nvim_shada_path_tail_with_sep_offset(const char *fname) { return (size_t)(path_tail_with_sep((char *)fname) - fname); }
int nvim_shada_os_fileinfo(const char *fname, uint64_t *out_mode, uint64_t *out_uid, uint64_t *out_gid)
{ FileInfo info;
  if (!os_fileinfo(fname, &info)) { *out_mode = 0; *out_uid = 0; *out_gid = 0; return 0; }
  *out_mode = (uint64_t)info.stat.st_mode; *out_uid = (uint64_t)info.stat.st_uid; *out_gid = (uint64_t)info.stat.st_gid;
  return S_ISDIR(info.stat.st_mode) ? 0 : 1; }
int nvim_shada_os_fchown(void *sd_writer, uint64_t uid, uint64_t gid) { return os_fchown(file_fd((FileDescriptor *)sd_writer), (uv_uid_t)uid, (uv_gid_t)gid); }
void *nvim_shada_fname_bufs_new(void)
{ PMap(cstr_t) *m = xcalloc(1, sizeof(PMap(cstr_t))); *m = (PMap(cstr_t))MAP_INIT; return m; }
void nvim_shada_fname_bufs_destroy(void *handle)
{ PMap(cstr_t) *m = (PMap(cstr_t) *)handle; const char *key;
  map_foreach_key(m, key, { xfree((char *)key); }) map_destroy(cstr_t, m); xfree(m); }
void *nvim_shada_oldfiles_set_new(void)
{ Set(cstr_t) *s = xcalloc(1, sizeof(Set(cstr_t))); *s = (Set(cstr_t))SET_INIT; return s; }
void nvim_shada_oldfiles_set_destroy(void *handle) { Set(cstr_t) *s = (Set(cstr_t) *)handle; set_destroy(cstr_t, s); xfree(s); }
int nvim_shada_argcount(void) { return ARGCOUNT; }
void nvim_shada_for_all_tab_windows_update_changelist(void *cl_bufs_handle)
{ Set(ptr_t) *cl_bufs = (Set(ptr_t) *)cl_bufs_handle;
  if (!cl_bufs->h.n_occupied) { return; }
  FOR_ALL_TAB_WINDOWS(tp, wp) { (void)tp;
    if (set_has(ptr_t, cl_bufs, wp->w_buffer)) { wp->w_changelistidx = wp->w_buffer->b_changelistlen; }
  }
}
uint32_t nvim_shada_additional_data_len(const void *ad_ptr) { const AdditionalData *ad = (const AdditionalData *)ad_ptr; return ad ? ad->nitems : 0; }
void nvim_shada_dump_additional_data(const void *ad_ptr, PackerBuffer *sbuf) { const AdditionalData *ad = (const AdditionalData *)ad_ptr; if (ad) { mpack_raw(ad->data, ad->nbytes, sbuf); } }
int nvim_shada_entry_is_blob_var(const ShadaEntry *entry) { return (entry && entry->data.global_var.value.v_type == VAR_BLOB) ? 1 : 0; }
void *nvim_shada_entry_var_value_ptr(ShadaEntry *entry) { return entry ? &entry->data.global_var.value : NULL; }
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
int nvim_shada_tv_get_type(const void *tv) { return tv ? ((const typval_T *)tv)->v_type : 0; }
void nvim_shada_build_gvar_entry(const char *name, void *tv, Timestamp ts, ShadaEntry *out)
{ typval_T tgttv; tv_copy((typval_T *)tv, &tgttv); tv_clear((typval_T *)tv); xfree(tv);
  *out = (ShadaEntry){ .type = kSDItemVariable, .timestamp = ts,
    .data = { .global_var = { .name = (char *)name, .value = tgttv } }, .additional_data = NULL }; }
void nvim_shada_clear_gvar_entry_value(ShadaEntry *entry) { tv_clear(&entry->data.global_var.value); }
void nvim_shada_set_all_last_cursors(void)
{ FOR_ALL_TAB_WINDOWS(tp, wp) { (void)tp; set_last_cursor(wp); } }
const void *nvim_shada_mark_global_iter(const void *iter,
                                        char *out_name, int64_t *out_lnum, int32_t *out_col,
                                        int *out_fnum, uint64_t *out_ts,
                                        const char **out_fname, void **out_additional)
{ xfmark_T fm; *out_name = NUL; const void *next = mark_global_iter(iter, out_name, &fm);
  if (*out_name != NUL) { *out_lnum = (int64_t)fm.fmark.mark.lnum; *out_col = (int32_t)fm.fmark.mark.col;
    *out_fnum = fm.fmark.fnum; *out_ts = fm.fmark.timestamp; *out_fname = fm.fname; *out_additional = fm.fmark.additional_data; }
  return next; }
uint64_t nvim_shada_named_mark_timestamp(int idx) { return (idx >= 0 && idx < NGLOBALMARKS) ? namedfm[idx].fmark.timestamp : 0; }
const void *nvim_shada_mark_buffer_iter(const void *iter,
                                        const void *buf, char *out_name,
                                        int64_t *out_lnum, int32_t *out_col,
                                        uint64_t *out_ts, void **out_additional)
{ fmark_T fm; *out_name = NUL; const void *next = mark_buffer_iter(iter, (const buf_T *)buf, out_name, &fm);
  if (*out_name != NUL) { *out_lnum = (int64_t)fm.mark.lnum; *out_col = (int32_t)fm.mark.col;
    *out_ts = fm.timestamp; *out_additional = fm.additional_data; }
  return next; }
int nvim_shada_buf_changelist_len(const void *buf) { return buf ? ((const buf_T *)buf)->b_changelistlen : 0; }
void nvim_shada_buf_changelist_entry(const void *buf, int idx, int64_t *out_lnum, int32_t *out_col, uint64_t *out_ts, void **out_additional)
{ if (!buf || idx < 0) { return; } const fmark_T fm = ((const buf_T *)buf)->b_changelist[idx];
  *out_lnum = (int64_t)fm.mark.lnum; *out_col = (int32_t)fm.mark.col; *out_ts = fm.timestamp; *out_additional = fm.additional_data; }
void nvim_shada_curwin_cursor(int64_t *out_lnum, int32_t *out_col) { *out_lnum = (int64_t)curwin->w_cursor.lnum; *out_col = (int32_t)curwin->w_cursor.col; }
void **nvim_shada_wms_file_marks_put_ref(void *wms_opaque, const char *fname, bool *is_new, const char **out_key)
{ WriteMergerState *wms = (WriteMergerState *)wms_opaque; if (!wms || !fname) { return NULL; }
  return (void **)pmap_put_ref(cstr_t)(&wms->file_marks, fname, (cstr_t **)out_key, is_new); }
void **nvim_shada_wms_file_marks_get_sorted(const void *wms_opaque, size_t *out_size)
{ const WriteMergerState *wms = (const WriteMergerState *)wms_opaque;
  if (!wms) { *out_size = 0; return NULL; }
  size_t sz = map_size(&wms->file_marks); *out_size = sz;
  if (sz == 0) { return NULL; }
  void **arr = xmalloc(sz * sizeof(*arr)); size_t i = 0; ptr_t val;
  map_foreach_value((PMap(cstr_t) *)&wms->file_marks, val, { arr[i++] = val; })
  qsort(arr, sz, sizeof(*arr), &rs_compare_file_marks); return arr; }
void nvim_shada_wms_file_marks_destroy(void *wms_opaque)
{ WriteMergerState *wms = (WriteMergerState *)wms_opaque; if (!wms) { return; }
  const char *key = NULL; ptr_t val;
  map_foreach(&wms->file_marks, key, val, { xfree((char *)key); xfree(val); })
  map_destroy(cstr_t, &wms->file_marks); }
bool nvim_shada_wms_dumped_vars_has(const void *wms_opaque, const char *name) { const WriteMergerState *wms = (const WriteMergerState *)wms_opaque; return (wms && name) ? set_has(cstr_t, (Set(cstr_t) *)&wms->dumped_variables, name) : false; }
void nvim_shada_wms_dumped_vars_put(void *wms_opaque, const char *name) { WriteMergerState *wms = (WriteMergerState *)wms_opaque; if (wms && name) { set_put(cstr_t, &wms->dumped_variables, name); } }
void nvim_shada_wms_dumped_vars_destroy(void *wms_opaque) { WriteMergerState *wms = (WriteMergerState *)wms_opaque; if (wms) { set_destroy(cstr_t, &wms->dumped_variables); } }
int nvim_shada_mark_get_cmp(const void *buf, const void *win, int name, uint64_t entry_ts)
{ if (!buf) { return 0; } fmark_T fm_storage;
  fmark_T *fm = mark_get((buf_T *)buf, (win_T *)win, &fm_storage, kMarkBufLocal, name);
  return (fm && fm->timestamp >= entry_ts) ? 1 : 0; }
static void nvim_shada_flush_file_buffer_(PackerBuffer *buffer)
{ FileDescriptor *fd = buffer->anydata; fd->write_pos = buffer->ptr;
  buffer->anyint = file_flush(fd); buffer->ptr = fd->write_pos; }
void nvim_shada_packer_init_for_file(void *fd, PackerBuffer *out)
{ FileDescriptor *file = (FileDescriptor *)fd;
  if (file_space(file) < (4 * MPACK_ITEM_SIZE)) { file_flush(file); }
  *out = (PackerBuffer){ .startptr = file->buffer, .ptr = file->write_pos,
    .endptr = file->buffer + ARENA_BLOCK_SIZE, .anydata = file, .anyint = 0,
    .packer_flush = nvim_shada_flush_file_buffer_ }; }
void nvim_shada_tv_get_refcheck_info(const void *tv, int *out_vtype, void **out_container, int *out_copy_id)
{ const typval_T *t = (const typval_T *)tv;
  if (!t) { *out_vtype = 0; *out_container = NULL; *out_copy_id = 0; return; }
  *out_vtype = t->v_type; *out_container = NULL; *out_copy_id = 0;
  if (t->v_type == VAR_DICT && t->vval.v_dict) { *out_container = &t->vval.v_dict->dv_hashtab; *out_copy_id = t->vval.v_dict->dv_copyID; }
  else if (t->v_type == VAR_LIST && t->vval.v_list) { *out_container = t->vval.v_list; *out_copy_id = t->vval.v_list->lv_copyID; } }
uint64_t nvim_shada_op_reg_get_timestamp(char name) { const yankreg_T *const reg = op_reg_get(name); return reg ? (uint64_t)reg->timestamp : 0; }
void nvim_shada_var_set_global_from_entry(ShadaEntry *entry) { var_set_global(entry->data.global_var.name, &entry->data.global_var.value); entry->data.global_var.value.v_type = VAR_UNKNOWN; }
int nvim_shada_jumplist_len(void) { return curwin->w_jumplistlen; }
void nvim_shada_jumplist_get_entry(int idx, uint64_t *out_ts, int64_t *out_lnum, int32_t *out_col, int *out_fnum, const char **out_fname)
{ const xfmark_T *jl = &curwin->w_jumplist[idx]; *out_ts = (uint64_t)jl->fmark.timestamp;
  *out_lnum = (int64_t)jl->fmark.mark.lnum; *out_col = (int32_t)jl->fmark.mark.col; *out_fnum = jl->fmark.fnum; *out_fname = jl->fname; }
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
      .mark = RS_POS_TO_POST(entry->data.filemark.mark),
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
void nvim_shada_buf_set_cursor_and_data(void *buf_handle, ShadaEntry *entry, size_t i)
{ buf_T *const buf = (buf_T *)buf_handle; fmarkv_T view = INIT_FMARKV;
  RESET_FMARK(&buf->b_last_cursor, RS_POS_TO_POST(entry->data.buffer_list.buffers[i].pos), 0, view);
  buflist_setfpos(buf, curwin, buf->b_last_cursor.mark.lnum, buf->b_last_cursor.mark.col, false);
  xfree(buf->additional_data); buf->additional_data = entry->data.buffer_list.buffers[i].additional_data;
  entry->data.buffer_list.buffers[i].additional_data = NULL; }
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
int nvim_shada_oldfiles_has(void *oldfiles_set_handle, const ShadaEntry *entry)
{ return set_has(cstr_t, (Set(cstr_t) *)oldfiles_set_handle, entry->data.filemark.fname) ? 1 : 0; }
void nvim_shada_cl_bufs_set_put(void *cl_bufs_handle, void *buf_handle)
{ set_put(ptr_t, (Set(ptr_t) *)cl_bufs_handle, buf_handle); }
void nvim_shada_changelist_get_entry(const void *buf_handle, int idx, uint64_t *out_ts, int64_t *out_lnum, int32_t *out_col)
{ const fmark_T *fm = &((const buf_T *)buf_handle)->b_changelist[idx]; *out_ts = (uint64_t)fm->timestamp; *out_lnum = (int64_t)fm->mark.lnum; *out_col = (int32_t)fm->mark.col; }
void nvim_shada_changelist_insert_entry(void *buf_handle, int i,
                                        ShadaEntry *entry, int cl_len)
{
  buf_T *buf = (buf_T *)buf_handle;
  if (i > 0 && cl_len == JUMPLISTSIZE) {
    free_fmark(buf->b_changelist[0]);
  }
  buf->b_changelist[i] = (fmark_T) {
    .mark = RS_POS_TO_POST(entry->data.filemark.mark),
    .fnum = 0,
    .timestamp = entry->timestamp,
    .view = INIT_FMARKV,
    .additional_data = entry->additional_data,
  };
  if (buf->b_changelistlen < JUMPLISTSIZE) {
    buf->b_changelistlen++;
  }
}
void nvim_shada_fm_xfree_fname(ShadaEntry *entry) { xfree(entry->data.filemark.fname); entry->data.filemark.fname = NULL; }
int nvim_shada_buf_get_fnum(const void *buf_handle) { return ((const buf_T *)buf_handle)->b_fnum; }
int nvim_shada_jumplist_marklist_insert(int i) { return rs_marklist_insert(curwin->w_jumplist, sizeof(*curwin->w_jumplist), curwin->w_jumplistlen, i); }
int nvim_shada_changelist_marklist_insert(void *buf_handle, int i) { buf_T *buf = (buf_T *)buf_handle; return rs_marklist_insert(buf->b_changelist, sizeof(*buf->b_changelist), buf->b_changelistlen, i); }
// Deleted: nvim_shada_file_try_read_buffered -- Rust uses #[link_name = "file_try_read_buffered"] directly
// Deleted: nvim_shada_file_bytes_read -- Rust reads FileDescriptor.bytes_read at offset 40 directly
void nvim_shada_semsg_u64(const char *fmt, uint64_t val) { semsg(fmt, (unsigned long long)val); }
void nvim_shada_semsg_2s_u64(const char *fmt, const char *a, uint64_t val, const char *b) { semsg(fmt, a, (unsigned long long)val, b); }
