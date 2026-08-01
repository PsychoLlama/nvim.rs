use crate::src::nvim::api::private::dispatch::{
    KeyDict__shada_buflist_item_get_field, KeyDict__shada_mark_get_field,
    KeyDict__shada_register_get_field, KeyDict__shada_search_pat_get_field,
};
use crate::src::nvim::api::private::helpers::{
    api_free_dict, api_free_string, copy_string, cstr_as_string,
};
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::{
    bt_quickfix, bt_terminal, buflist_findnr, buflist_new, buflist_setfpos,
};
use crate::src::nvim::cmdhist::{
    HistShadaEntry, hist_shada_replace, hist_shada_take, hist_shada_view,
};
use crate::src::nvim::eval::decode::{decode_string, unpack_typval};
use crate::src::nvim::eval::encode::encode_vim_to_msgpack;
use crate::src::nvim::eval::typval::tv_list_len;
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_list_alloc, tv_list_append_allocated_string,
};
use crate::src::nvim::eval::vars::{get_globvar_ht, get_vim_var_list, set_vim_var_list};
use crate::src::nvim::eval::{
    get_copyID, set_ref_in_ht, set_ref_in_list_items, var_flavour, var_set_global,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::{sub_get_replacement, sub_set_replacement};
use crate::src::nvim::ex_docmd::set_no_hlsearch;
use crate::src::nvim::fileio::{modname, vim_rename};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{
    NameBuff, curbuf, curtab, curwin, first_tabpage, firstbuf, firstwin, namedfm, no_hlsearch,
    p_enc, p_fs, p_hi, p_shada, p_shadafile, p_verbose,
};
use crate::src::nvim::map::{
    map_del_cstr_t_ptr_t, map_put_ref_cstr_t_ptr_t, map_ref_cstr_t_ptr_t, mh_get_cstr_t,
    mh_get_ptr_t, mh_put_cstr_t, mh_put_ptr_t,
};
use crate::src::nvim::mark::{
    cleanup_jumplist, free_fmark, free_xfmark, mark_buffer_iter, mark_get, mark_global_iter,
    mark_jumplist_iter, mark_set_global, mark_set_local, set_last_cursor, setpcmark,
};
use crate::src::nvim::mbyte::mb_strnicmp;
use crate::src::nvim::memory::{
    strequal, xcalloc, xfree, xmalloc, xmemdup, xmemdupz, xrealloc, xstrdup,
};
use crate::src::nvim::message::{semsg, siemsg, smsg, verbose_enter, verbose_leave};
use crate::src::nvim::msgpack_rpc::packer::{
    mpack_array, mpack_bin, mpack_bool, mpack_check_buffer, mpack_integer, mpack_map, mpack_raw,
    mpack_remaining, mpack_str, mpack_uint, mpack_uint64, packer_string_buffer, packer_take_string,
};
use crate::src::nvim::msgpack_rpc::unpacker::{
    push_additional_data, unpack_array, unpack_integer, unpack_keydict, unpack_skip, unpack_string,
};
use crate::src::nvim::option::{copy_option_part, magic_isset};
use crate::src::nvim::os::env::{expand_env, home_replace, home_replace_save, os_get_pid};
use crate::src::nvim::os::fileio::{
    file_close, file_flush, file_open, file_open_buffer, file_read, file_skip,
    file_try_read_buffered,
};
use crate::src::nvim::os::fs::{
    os_fchown, os_fileinfo, os_getperm, os_isdir, os_mkdir_recurse, os_remove,
};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, atoi, getgid, gettext, getuid, memchr, memcpy, memmove, memset, qsort,
    strcmp, strlen,
};
use crate::src::nvim::os::stdpaths::stdpaths_user_state_subpath;
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::path::{
    concat_fnames_realloc, path_fnamecmp, path_tail_with_sep, path_try_shorten_fname,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::regexp::regtilde;
use crate::src::nvim::register::op_reg_get;
use crate::src::nvim::register::op_reg_index;
use crate::src::nvim::search::{
    get_search_pattern, get_substitute_pattern, search_was_last_used, set_last_used_pattern,
    set_search_pattern, set_substitute_pattern,
};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    __uid_t, AdditionalData, AdditionalDataBuilder, Arena, Boolean, Dict, FileDescriptor, FileInfo,
    Integer, KeyDict__shada_buflist_item, KeyDict__shada_mark, KeyDict__shada_register,
    KeyDict__shada_search_pat, KeySetLink, KeyValuePair, ListLenSpecials, MHPutStatus,
    Map_cstr_t_ptr_t, MapHash, MarkGet, MotionType, Object, OptInt, OptionalKeys, PackerBuffer,
    SearchOffset, SearchPattern, Set_cstr_t, Set_ptr_t, String_0, StringArray,
    SubReplacementString, Timestamp, VarLockStatus, VarType, VimVarIndex, bln_values, buf_T,
    colnr_T, cstr_t, dict_T, dictitem_T, fmark_T, fmarkv_T, hashitem_T, hashtab_T, ht_stack_T,
    int32_t, int64_t, kObjectTypeInteger, kObjectTypeString, key_value_pair, linenr_T, list_T,
    list_stack_T, object, object_data as C2Rust_Unnamed_1, packer_buffer_t, pos_T, ptr_t,
    ptrdiff_t, size_t, ssize_t, tabpage_T, typval_T, typval_vval_union, uint8_t, uint32_t,
    uint64_t, uintmax_t, uv_gid_t, uv_stat_t, uv_timespec_t, uv_uid_t, var_flavour_T, win_T,
    xfmark_T, yankreg_T,
};
use crate::src::nvim::version::longVersion;

// The carve of the transpiled module; see each child's docs.
mod file;
pub use self::file::*;
mod pack;
pub(crate) use self::pack::*;
mod unpack;
pub(crate) use self::unpack::*;
mod merge;
pub(crate) use self::merge::*;
mod collect;
pub use self::collect::*;
mod read;
pub use self::read::*;
mod write;
pub(crate) use self::write::*;
unsafe extern "C" {
    fn op_global_reg_iter(
        iter: *const ::core::ffi::c_void,
        name: *mut ::core::ffi::c_char,
        reg: *mut yankreg_T,
        is_unnamed: *mut bool,
    ) -> *const ::core::ffi::c_void;
    fn op_reg_set(name: ::core::ffi::c_char, reg: yankreg_T, is_unnamed: bool) -> bool;
}
pub type __uint64_t = u64;
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const MPACK_OK: C2Rust_Unnamed_0 = 0;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_FUNC: VarType = 3;
pub const VAR_UNKNOWN: VarType = 0;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const VAR_TYPE_BLOB: C2Rust_Unnamed_16 = 10;
pub const kMHExisting: MHPutStatus = 0;
pub const kMarkBufLocal: MarkGet = 0;
pub const BLN_LISTED: bln_values = 2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_17 = 1;
pub const HIST_CMD: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const HIST_COUNT: C2Rust_Unnamed_18 = 5;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VAR_FLAVOUR_SHADA: var_flavour_T = 4;
pub const VAR_FLAVOUR_SESSION: var_flavour_T = 2;
pub const VAR_FLAVOUR_DEFAULT: var_flavour_T = 1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kFileTruncate: C2Rust_Unnamed_19 = 32;
pub const kFileCreateOnly: C2Rust_Unnamed_19 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_19 = 8;
pub const kFileCreate: C2Rust_Unnamed_19 = 2;
pub const kFileReadOnly: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kShaDaMissingError: C2Rust_Unnamed_21 = 16;
pub const kShaDaGetOldfiles: C2Rust_Unnamed_21 = 8;
pub const kShaDaForceit: C2Rust_Unnamed_21 = 4;
pub const kShaDaWantMarks: C2Rust_Unnamed_21 = 2;
pub const kShaDaWantInfo: C2Rust_Unnamed_21 = 1;
pub const kSDWriteReadNotShada: ShaDaWriteResult = 1;
pub type ShaDaWriteResult = ::core::ffi::c_uint;
pub const kSDWriteIgnError: ShaDaWriteResult = 3;
pub const kSDWriteFailed: ShaDaWriteResult = 2;
pub const kSDWriteSuccessful: ShaDaWriteResult = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WriteMergerState {
    pub hms: [HistoryMergerState; 5],
    pub global_marks: [ShadaEntry; 26],
    pub numbered_marks: [ShadaEntry; 10],
    pub registers: [ShadaEntry; 37],
    pub jumps: [ShadaEntry; 100],
    pub jumps_size: size_t,
    pub search_pattern: ShadaEntry,
    pub sub_search_pattern: ShadaEntry,
    pub replacement: ShadaEntry,
    pub dumped_variables: Set_cstr_t,
    pub file_marks: Map_cstr_t_ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ShadaEntry {
    pub type_0: ShadaEntryType,
    pub can_free_entry: bool,
    pub timestamp: Timestamp,
    pub data: C2Rust_Unnamed_22,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub header: Dict,
    pub filemark: shada_filemark,
    pub search_pattern: KeyDict__shada_search_pat,
    pub history_item: history_item,
    pub reg: reg,
    pub global_var: global_var,
    pub unknown_item: C2Rust_Unnamed_23,
    pub sub_string: sub_string,
    pub buffer_list: buffer_list,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffer_list {
    pub size: size_t,
    pub buffers: *mut buffer_list_buffer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffer_list_buffer {
    pub pos: pos_T,
    pub fname: *mut ::core::ffi::c_char,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sub_string {
    pub sub: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub type_0: uint64_t,
    pub contents: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct global_var {
    pub name: *mut ::core::ffi::c_char,
    pub value: typval_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg {
    pub name: ::core::ffi::c_char,
    pub type_0: MotionType,
    pub contents: *mut String_0,
    pub is_unnamed: bool,
    pub contents_size: size_t,
    pub width: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct history_item {
    pub histtype: uint8_t,
    pub string: *mut ::core::ffi::c_char,
    pub sep: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct shada_filemark {
    pub name: ::core::ffi::c_char,
    pub mark: pos_T,
    pub fname: *mut ::core::ffi::c_char,
}
pub type ShadaEntryType = ::core::ffi::c_int;
pub const kSDItemChange: ShadaEntryType = 11;
pub const kSDItemLocalMark: ShadaEntryType = 10;
pub const kSDItemBufferList: ShadaEntryType = 9;
pub const kSDItemJump: ShadaEntryType = 8;
pub const kSDItemGlobalMark: ShadaEntryType = 7;
pub const kSDItemVariable: ShadaEntryType = 6;
pub const kSDItemRegister: ShadaEntryType = 5;
pub const kSDItemHistoryEntry: ShadaEntryType = 4;
pub const kSDItemSubString: ShadaEntryType = 3;
pub const kSDItemSearchPattern: ShadaEntryType = 2;
pub const kSDItemHeader: ShadaEntryType = 1;
pub const kSDItemMissing: ShadaEntryType = 0;
pub const kSDItemUnknown: ShadaEntryType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HistoryMergerState {
    pub hmll: HMLList,
    pub do_merge: bool,
    pub reading: bool,
    /// Snapshot of neovim's own history (oldest first), taken at
    /// [`hms_init`] time and drained by [`hms_insert`] /
    /// [`hms_insert_whole_neovim_history`]. Boxed slice owned by this
    /// struct; freed in [`hms_dealloc`].
    pub pending: *mut ShadaEntry,
    pub pending_len: size_t,
    pub pending_pos: size_t,
    pub history_type: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HMLList {
    pub entries: *mut HMLListEntry,
    pub first: *mut HMLListEntry,
    pub last: *mut HMLListEntry,
    pub free_entry: *mut HMLListEntry,
    pub last_free_entry: *mut HMLListEntry,
    pub size: size_t,
    pub num_entries: size_t,
    pub contained_entries: Map_cstr_t_ptr_t,
}
pub type HMLListEntry = hm_llist_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hm_llist_entry {
    pub data: ShadaEntry,
    pub next: *mut hm_llist_entry,
    pub prev: *mut hm_llist_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileMarks {
    pub marks: [ShadaEntry; 29],
    pub changes: [ShadaEntry; 100],
    pub changes_size: size_t,
    pub additional_marks: *mut ShadaEntry,
    pub additional_marks_size: size_t,
    pub greatest_timestamp: Timestamp,
}
pub const kSDReadChanges: SRNIFlags = 2048;
pub const kSDReadLocalMarks: SRNIFlags = 1024;
pub const kSDReadGlobalMarks: SRNIFlags = 128;
pub const kSDReadVariables: SRNIFlags = 64;
pub const kSDReadRegisters: SRNIFlags = 32;
pub const kSDReadHistory: SRNIFlags = 16;
pub const kSDReadUnknown: SRNIFlags = 4096;
pub const kSDReadUndisableableData: SRNIFlags = 268;
pub const kSDReadStatusMalformed: ShaDaReadResult = 4;
pub const kSDReadStatusReadError: ShaDaReadResult = 2;
pub const kSDReadStatusNotShaDa: ShaDaReadResult = 3;
pub const kSDReadStatusFinished: ShaDaReadResult = 1;
pub const kSDReadStatusSuccess: ShaDaReadResult = 0;
pub type ShaDaReadResult = ::core::ffi::c_uint;
pub type SearchPatternGetter = Option<unsafe extern "C" fn(*mut SearchPattern) -> ()>;
pub const kSDReadBufferList: SRNIFlags = 512;
pub type SRNIFlags = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const PTRDIFF_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn __bswap_64(mut __bsx: __uint64_t) -> __uint64_t {
    return ((__bsx as ::core::ffi::c_ulonglong & 0xff00000000000000 as ::core::ffi::c_ulonglong)
        >> 56 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000000000 as ::core::ffi::c_ulonglong)
            >> 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000000000 as ::core::ffi::c_ulonglong)
            >> 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00000000 as ::core::ffi::c_ulonglong)
            >> 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000 as ::core::ffi::c_ulonglong)
            << 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000 as ::core::ffi::c_ulonglong)
            << 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00 as ::core::ffi::c_ulonglong)
            << 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff as ::core::ffi::c_ulonglong)
            << 56 as ::core::ffi::c_int) as __uint64_t;
}
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const ROOT_UID: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: AdditionalDataBuilder = AdditionalDataBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: Set_cstr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<cstr_t>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_put_cstr_t(
    mut set: *mut Set_cstr_t,
    mut key: cstr_t,
    mut key_alloc: *mut *mut cstr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const JUMPLISTSIZE: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_search_pat__sp: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__c: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__f: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__l: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_mark__n: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__rt: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__ru: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_register__rw: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_buflist_item__c: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_buflist_item__f: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX__shada_buflist_item__l: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MPACK_ITEM_SIZE: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mark_global_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return if name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        name as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
    } else if ascii_isdigit(name as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        NMARKS + (name as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[inline]
unsafe extern "C" fn mark_local_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return if name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        name as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
    } else if name as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        NMARKS
    } else if name as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
        NMARKS + 1 as ::core::ffi::c_int
    } else if name as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        NMARKS + 2 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub const DEFAULT_POS: pos_T = pos_T {
    lnum: 1 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
};
static default_pos: GlobalCell<pos_T> = GlobalCell::new(DEFAULT_POS);
static sd_default_values: GlobalCell<[ShadaEntry; 12]> = GlobalCell::new([
    ShadaEntry {
        type_0: kSDItemMissing,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemHeader,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            header: Dict {
                size: 0 as size_t,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemSearchPattern,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            search_pattern: KeyDict__shada_search_pat {
                is_set___shada_search_pat_: 0,
                magic: true,
                smartcase: false,
                has_line_offset: false,
                place_cursor_at_end: false,
                is_last_used: true,
                is_substitute_pattern: false,
                highlighted: false,
                search_backward: false,
                offset: 0 as Integer,
                pat: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                },
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemSubString,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            sub_string: sub_string {
                sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemHistoryEntry,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            history_item: history_item {
                histtype: HIST_CMD as ::core::ffi::c_int as uint8_t,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                sep: '\0' as ::core::ffi::c_char,
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemRegister,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            reg: reg {
                name: '\0' as ::core::ffi::c_char,
                type_0: kMTCharWise,
                contents: ::core::ptr::null_mut::<String_0>(),
                is_unnamed: false,
                contents_size: 0 as size_t,
                width: 0 as size_t,
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemVariable,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            global_var: global_var {
                name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                value: typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    },
                },
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemGlobalMark,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '"' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemJump,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '\0' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemBufferList,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            buffer_list: buffer_list {
                size: 0 as size_t,
                buffers: ::core::ptr::null_mut::<buffer_list_buffer>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemLocalMark,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '"' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    ShadaEntry {
        type_0: kSDItemChange,
        can_free_entry: false,
        timestamp: 0 as Timestamp,
        data: C2Rust_Unnamed_22 {
            filemark: shada_filemark {
                name: '\0' as ::core::ffi::c_char,
                mark: pos_T {
                    lnum: 1 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
]);
static default_shada_file: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub const SHADA_MPACK_FREE_SPACE: ::core::ffi::c_int = 4 as ::core::ffi::c_int * MPACK_ITEM_SIZE;
unsafe extern "C" fn shada_read_next_item(
    sd_reader: *mut FileDescriptor,
    entry: *mut ShadaEntry,
    flags: ::core::ffi::c_uint,
    max_kbyte: size_t,
) -> ShaDaReadResult {
    let mut verify_but_ignore: bool = false;
    let mut type_u64: uint64_t = 0;
    let mut timestamp_u64: uint64_t = 0;
    let mut length_u64: uint64_t = 0;
    let mut initial_fpos: uint64_t = 0;
    let mut ad: AdditionalDataBuilder = AdditionalDataBuilder {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut read_additional_array_elements: uint32_t = 0;
    let mut error_alloc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mru_ret: ShaDaReadResult = kSDReadStatusSuccess;
    let mut length: size_t = 0;
    let mut parse_pos: uint64_t = 0;
    let mut buf_allocated: bool = false;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut read_ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut read_size: size_t = 0;
    let mut ret: ShaDaReadResult = kSDReadStatusMalformed;
    '_shada_read_next_item_end: {
        '_shada_read_next_item_error: loop {
            memset(
                entry as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<ShadaEntry>(),
            );
            if file_eof(sd_reader) {
                return kSDReadStatusFinished;
            }
            verify_but_ignore = false_0 != 0;
            type_u64 = kSDItemMissing as ::core::ffi::c_int as uint64_t;
            timestamp_u64 = 0;
            length_u64 = 0;
            initial_fpos = (*sd_reader).bytes_read;
            ad = KV_INITIAL_VALUE;
            read_additional_array_elements = 0 as uint32_t;
            error_alloc = ::core::ptr::null_mut::<::core::ffi::c_char>();
            mru_ret = kSDReadStatusSuccess;
            mru_ret = msgpack_read_uint64(sd_reader, true_0 != 0, &raw mut type_u64);
            if mru_ret as ::core::ffi::c_uint
                != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    mru_ret = msgpack_read_uint64(sd_reader, false_0 != 0, &raw mut timestamp_u64);
                    mru_ret as ::core::ffi::c_uint
                        != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                }
                || {
                    mru_ret = msgpack_read_uint64(sd_reader, false_0 != 0, &raw mut length_u64);
                    mru_ret as ::core::ffi::c_uint
                        != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                }
            {
                return mru_ret;
            }
            if length_u64 > PTRDIFF_MAX as uint64_t {
                semsg(
                    gettext(
                        b"E576: Error while reading ShaDa file: there is an item at position %lu that is stated to be too long\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    initial_fpos,
                );
                return kSDReadStatusNotShaDa;
            }
            length = length_u64 as size_t;
            (*entry).timestamp = timestamp_u64;
            (*entry).can_free_entry = true_0 != 0;
            if type_u64 == 0 as uint64_t {
                semsg(
                    gettext(
                        b"E576: Error while reading ShaDa file: there is an item at position %lu that must not be there: Missing items are for internal uses only\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    initial_fpos,
                );
                return kSDReadStatusNotShaDa;
            }
            if (if type_u64 > kSDItemChange as ::core::ffi::c_int as uint64_t {
                (flags & kSDReadUnknown as ::core::ffi::c_int as ::core::ffi::c_uint == 0)
                    as ::core::ffi::c_int
            } else {
                (((1 as ::core::ffi::c_int) << type_u64) as ::core::ffi::c_uint & flags == 0)
                    as ::core::ffi::c_int
            }) != 0
                || max_kbyte != 0 && length > max_kbyte.wrapping_mul(1024 as size_t)
            {
                if initial_fpos == 0 as uint64_t
                    && (type_u64 == '\n' as uint64_t
                        || type_u64 > kSDItemChange as ::core::ffi::c_int as uint64_t)
                {
                    verify_but_ignore = true_0 != 0;
                } else {
                    let srs_ret: ShaDaReadResult = sd_reader_skip(sd_reader, length);
                    if srs_ret as ::core::ffi::c_uint
                        != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return srs_ret;
                    }
                    continue;
                }
            }
            parse_pos = (*sd_reader).bytes_read;
            buf_allocated = false_0 != 0;
            buf = file_try_read_buffered(sd_reader, length);
            if buf.is_null() {
                buf_allocated = true_0 != 0;
                buf = xmalloc(length) as *mut ::core::ffi::c_char;
                let fl_ret: ShaDaReadResult = fread_len(sd_reader, buf, length);
                if fl_ret as ::core::ffi::c_uint
                    != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    ret = fl_ret;
                    break;
                }
            }
            read_ptr = buf;
            read_size = length;
            if verify_but_ignore {
                let mut status: ::core::ffi::c_int =
                    unpack_skip(&raw mut read_ptr, &raw mut read_size);
                let mut spm_ret: ShaDaReadResult =
                    shada_check_status(parse_pos as uintmax_t, status, read_size);
                if buf_allocated {
                    xfree(buf as *mut ::core::ffi::c_void);
                }
                if spm_ret as ::core::ffi::c_uint
                    != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return spm_ret;
                }
            } else {
                if type_u64 > kSDItemChange as ::core::ffi::c_int as uint64_t {
                    (*entry).type_0 = kSDItemUnknown;
                    (*entry).data.unknown_item.size = length;
                    (*entry).data.unknown_item.type_0 = type_u64;
                    if initial_fpos == 0 as uint64_t {
                        let mut status_0: ::core::ffi::c_int =
                            unpack_skip(&raw mut read_ptr, &raw mut read_size);
                        let mut spm_ret_0: ShaDaReadResult =
                            shada_check_status(parse_pos as uintmax_t, status_0, read_size);
                        if spm_ret_0 as ::core::ffi::c_uint
                            != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if buf_allocated {
                                xfree(buf as *mut ::core::ffi::c_void);
                            }
                            (*entry).type_0 = kSDItemMissing;
                            return spm_ret_0;
                        }
                    }
                    (*entry).data.unknown_item.contents =
                        (if buf_allocated as ::core::ffi::c_int != 0 {
                            buf as *mut ::core::ffi::c_void
                        } else {
                            xmemdup(buf as *const ::core::ffi::c_void, length)
                        }) as *mut ::core::ffi::c_char;
                    return kSDReadStatusSuccess;
                }
                (*entry).data = (*sd_default_values.ptr())[type_u64 as usize].data;
                's_900: {
                    match type_u64 as ShadaEntryType as ::core::ffi::c_int {
                        2 => {
                            let mut it: *mut KeyDict__shada_search_pat =
                                &raw mut (*entry).data.search_pattern;
                            if !unpack_keydict(
                                it as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict__shada_search_pat_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                &raw mut ad,
                                &raw mut read_ptr,
                                &raw mut read_size,
                                &raw mut error_alloc,
                            ) {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: search pattern entry at position %lu %s\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                    error_alloc,
                                );
                                (*it).pat = NULL_STRING;
                                break '_shada_read_next_item_error;
                            } else if !((*it).is_set___shada_search_pat_
                                as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX__shada_search_pat__sp
                                != 0 as ::core::ffi::c_ulonglong)
                            {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: search pattern entry at position %lu has no pattern\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                (*entry).data.search_pattern.pat = copy_string(
                                    (*entry).data.search_pattern.pat,
                                    ::core::ptr::null_mut::<Arena>(),
                                );
                            }
                        }
                        11 | 8 | 7 | 10 => {
                            let mut it_0: KeyDict__shada_mark = KeyDict__shada_mark {
                                is_set___shada_mark_: 0 as OptionalKeys,
                                n: 0,
                                l: 0,
                                c: 0,
                                f: String_0 {
                                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    size: 0,
                                },
                            };
                            if !unpack_keydict(
                                &raw mut it_0 as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict__shada_mark_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                &raw mut ad,
                                &raw mut read_ptr,
                                &raw mut read_size,
                                &raw mut error_alloc,
                            ) {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: mark entry at position %lu %s\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                    error_alloc,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__n
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if type_u64 == kSDItemJump as ::core::ffi::c_int as uint64_t
                                        || type_u64
                                            == kSDItemChange as ::core::ffi::c_int as uint64_t
                                    {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: mark entry at position %lu has n key which is only valid for local and global mark entries\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                        );
                                        break '_shada_read_next_item_error;
                                    } else {
                                        (*entry).data.filemark.name = it_0.n as ::core::ffi::c_char;
                                    }
                                }
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__l
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.filemark.mark.lnum = it_0.l as linenr_T;
                                }
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__c
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.filemark.mark.col = it_0.c as colnr_T;
                                }
                                if it_0.is_set___shada_mark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_mark__f
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.filemark.fname = xmemdupz(
                                        it_0.f.data as *const ::core::ffi::c_void,
                                        it_0.f.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                }
                                if (*entry).data.filemark.fname.is_null() {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: mark entry at position %lu is missing file name\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else if (*entry).data.filemark.mark.lnum <= 0 as linenr_T {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: mark entry at position %lu has invalid line number\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else if (*entry).data.filemark.mark.col < 0 as ::core::ffi::c_int
                                {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: mark entry at position %lu has invalid column number\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                }
                            }
                        }
                        5 => {
                            let mut it_1: KeyDict__shada_register = KeyDict__shada_register {
                                is_set___shada_register_: 0 as OptionalKeys,
                                rc: StringArray {
                                    size: 0,
                                    capacity: 0,
                                    items: ::core::ptr::null_mut::<String_0>(),
                                },
                                ru: false,
                                rt: 0,
                                n: 0,
                                rw: 0,
                            };
                            if !unpack_keydict(
                                &raw mut it_1 as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict__shada_register_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                &raw mut ad,
                                &raw mut read_ptr,
                                &raw mut read_size,
                                &raw mut error_alloc,
                            ) {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: register entry at position %lu %s\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                    error_alloc,
                                );
                                xfree(it_1.rc.items as *mut ::core::ffi::c_void);
                                it_1.rc.capacity = 0 as size_t;
                                it_1.rc.size = it_1.rc.capacity;
                                it_1.rc.items = ::core::ptr::null_mut::<String_0>();
                                break '_shada_read_next_item_error;
                            } else if it_1.rc.size == 0 as size_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: register entry at position %lu has rc key with missing or empty array\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                (*entry).data.reg.contents_size = it_1.rc.size;
                                (*entry).data.reg.contents = xmalloc(
                                    it_1.rc
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<String_0>()),
                                )
                                    as *mut String_0;
                                let mut j: size_t = 0 as size_t;
                                while j < it_1.rc.size {
                                    *(*entry).data.reg.contents.offset(j as isize) = copy_string(
                                        *it_1.rc.items.offset(j as isize),
                                        ::core::ptr::null_mut::<Arena>(),
                                    );
                                    j = j.wrapping_add(1);
                                }
                                xfree(it_1.rc.items as *mut ::core::ffi::c_void);
                                it_1.rc.capacity = 0 as size_t;
                                it_1.rc.size = it_1.rc.capacity;
                                it_1.rc.items = ::core::ptr::null_mut::<String_0>();
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__ru
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.is_unnamed = it_1.ru;
                                }
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__rt
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.type_0 = it_1.rt as uint8_t as MotionType;
                                }
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__n
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.name = it_1.n as ::core::ffi::c_char;
                                }
                                if it_1.is_set___shada_register_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX__shada_register__rw
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*entry).data.reg.width = it_1.rw as size_t;
                                }
                            }
                        }
                        4 => {
                            let mut len: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len < 2 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: history entry at position %lu is not an array with enough elements\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                let mut hist_type: Integer = 0;
                                if !unpack_integer(
                                    &raw mut read_ptr,
                                    &raw mut read_size,
                                    &raw mut hist_type,
                                ) {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: history entry at position %lu has wrong history type type\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else {
                                    let item: String_0 =
                                        unpack_string(&raw mut read_ptr, &raw mut read_size);
                                    if item.data.is_null() {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: history entry at position %lu has wrong history string type\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                        );
                                        break '_shada_read_next_item_error;
                                    } else if !memchr(
                                        item.data as *const ::core::ffi::c_void,
                                        0 as ::core::ffi::c_int,
                                        item.size,
                                    )
                                    .is_null()
                                    {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: history entry at position %lu contains string with zero byte inside\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                        );
                                        break '_shada_read_next_item_error;
                                    } else {
                                        (*entry).data.history_item.histtype = hist_type as uint8_t;
                                        let is_hist_search: bool =
                                            (*entry).data.history_item.histtype
                                                as ::core::ffi::c_int
                                                == HIST_SEARCH as ::core::ffi::c_int;
                                        if is_hist_search {
                                            if len < 3 as ssize_t {
                                                semsg(
                                                    gettext(
                                                        b"E575: Error while reading ShaDa file: search history entry at position %lu does not have separator character\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    initial_fpos,
                                                );
                                                break '_shada_read_next_item_error;
                                            } else {
                                                let mut sep_type: Integer = 0;
                                                if !unpack_integer(
                                                    &raw mut read_ptr,
                                                    &raw mut read_size,
                                                    &raw mut sep_type,
                                                ) {
                                                    semsg(
                                                        gettext(
                                                            b"E575: Error while reading ShaDa file: search history entry at position %lu has wrong history separator type\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        ),
                                                        initial_fpos,
                                                    );
                                                    break '_shada_read_next_item_error;
                                                } else {
                                                    (*entry).data.history_item.sep =
                                                        sep_type as ::core::ffi::c_char;
                                                }
                                            }
                                        }
                                        let mut strsize: size_t = item
                                            .size
                                            .wrapping_add(1 as size_t)
                                            .wrapping_add(1 as size_t);
                                        (*entry).data.history_item.string =
                                            xmalloc(strsize) as *mut ::core::ffi::c_char;
                                        memcpy(
                                            (*entry).data.history_item.string
                                                as *mut ::core::ffi::c_void,
                                            item.data as *const ::core::ffi::c_void,
                                            item.size,
                                        );
                                        *(*entry)
                                            .data
                                            .history_item
                                            .string
                                            .offset(strsize.wrapping_sub(2 as size_t) as isize) =
                                            0 as ::core::ffi::c_char;
                                        *(*entry)
                                            .data
                                            .history_item
                                            .string
                                            .offset(strsize.wrapping_sub(1 as size_t) as isize) =
                                            (*entry).data.history_item.sep;
                                        read_additional_array_elements = (len
                                            - (2 as ::core::ffi::c_int
                                                + is_hist_search as ::core::ffi::c_int)
                                                as ssize_t)
                                            as uint32_t;
                                    }
                                }
                            }
                        }
                        6 => {
                            let mut len_0: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len_0 < 2 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: variable entry at position %lu is not an array with enough elements\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                let mut name: String_0 =
                                    unpack_string(&raw mut read_ptr, &raw mut read_size);
                                if name.data.is_null() {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable name type\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else {
                                    (*entry).data.global_var.name = xmemdupz(
                                        name.data as *const ::core::ffi::c_void,
                                        name.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    let mut binval: String_0 =
                                        unpack_string(&raw mut read_ptr, &raw mut read_size);
                                    let mut is_blob: bool = false_0 != 0;
                                    if !binval.data.is_null() {
                                        if len_0 > 2 as ssize_t {
                                            let mut type_0: Integer = 0;
                                            if !unpack_integer(
                                                &raw mut read_ptr,
                                                &raw mut read_size,
                                                &raw mut type_0,
                                            ) || type_0
                                                != VAR_TYPE_BLOB as ::core::ffi::c_int as Integer
                                            {
                                                semsg(
                                                    gettext(
                                                        b"E575: Error while reading ShaDa file: variable entry at position %lu has wrong variable type\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    initial_fpos,
                                                );
                                                break '_shada_read_next_item_error;
                                            } else {
                                                is_blob = true_0 != 0;
                                            }
                                        }
                                        (*entry).data.global_var.value = decode_string(
                                            binval.data,
                                            binval.size,
                                            is_blob,
                                            false_0 != 0,
                                        );
                                    } else {
                                        let mut status_1: ::core::ffi::c_int = unpack_typval(
                                            &raw mut read_ptr,
                                            &raw mut read_size,
                                            &raw mut (*entry).data.global_var.value,
                                        );
                                        if status_1 != MPACK_OK as ::core::ffi::c_int {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: variable entry at position %lu has value that cannot be converted to the Vimscript value\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        }
                                    }
                                    read_additional_array_elements = (len_0
                                        - 2 as ssize_t
                                        - (if is_blob as ::core::ffi::c_int != 0 {
                                            1 as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }) as ssize_t)
                                        as uint32_t;
                                }
                            }
                        }
                        3 => {
                            let mut len_1: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len_1 < 1 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: sub string entry at position %lu is not an array with enough elements\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else {
                                let mut sub: String_0 =
                                    unpack_string(&raw mut read_ptr, &raw mut read_size);
                                if sub.data.is_null() {
                                    semsg(
                                        gettext(
                                            b"E575: Error while reading ShaDa file: sub string entry at position %lu has wrong sub string type\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        ),
                                        initial_fpos,
                                    );
                                    break '_shada_read_next_item_error;
                                } else {
                                    (*entry).data.sub_string.sub =
                                        xmemdupz(sub.data as *const ::core::ffi::c_void, sub.size)
                                            as *mut ::core::ffi::c_char;
                                    read_additional_array_elements =
                                        (len_1 - 1 as ssize_t) as uint32_t;
                                }
                            }
                        }
                        9 => {
                            let mut len_2: ssize_t =
                                unpack_array(&raw mut read_ptr, &raw mut read_size);
                            if len_2 < 0 as ssize_t {
                                semsg(
                                    gettext(
                                        b"E575: Error while reading ShaDa file: buffer list entry at position %lu is not an array\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    ),
                                    initial_fpos,
                                );
                                break '_shada_read_next_item_error;
                            } else if len_2 != 0 as ssize_t {
                                (*entry).data.buffer_list.buffers = xcalloc(
                                    len_2 as size_t,
                                    ::core::mem::size_of::<buffer_list_buffer>(),
                                )
                                    as *mut buffer_list_buffer;
                                let mut i: size_t = 0 as size_t;
                                loop {
                                    if i >= len_2 as size_t {
                                        break 's_900;
                                    }
                                    (*entry).data.buffer_list.size =
                                        (*entry).data.buffer_list.size.wrapping_add(1);
                                    let mut it_2: KeyDict__shada_buflist_item =
                                        KeyDict__shada_buflist_item {
                                            is_set___shada_buflist_item_: 0 as OptionalKeys,
                                            l: 0,
                                            c: 0,
                                            f: String_0 {
                                                data: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                                size: 0,
                                            },
                                        };
                                    let mut it_ad: AdditionalDataBuilder = KV_INITIAL_VALUE;
                                    if !unpack_keydict(
                                        &raw mut it_2 as *mut ::core::ffi::c_void,
                                        Some(
                                            KeyDict__shada_buflist_item_get_field
                                                as unsafe extern "C" fn(
                                                    *const ::core::ffi::c_char,
                                                    size_t,
                                                )
                                                    -> *mut KeySetLink,
                                        ),
                                        &raw mut it_ad,
                                        &raw mut read_ptr,
                                        &raw mut read_size,
                                        &raw mut error_alloc,
                                    ) {
                                        semsg(
                                            gettext(
                                                b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            initial_fpos,
                                            error_alloc,
                                        );
                                        xfree(it_ad.items as *mut ::core::ffi::c_void);
                                        it_ad.capacity = 0 as size_t;
                                        it_ad.size = it_ad.capacity;
                                        it_ad.items =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        break '_shada_read_next_item_error;
                                    } else {
                                        let mut e: *mut buffer_list_buffer =
                                            (*entry).data.buffer_list.buffers.offset(i as isize)
                                                as *mut buffer_list_buffer;
                                        (*e).additional_data = it_ad.items as *mut AdditionalData;
                                        (*e).pos = default_pos.get();
                                        if it_2.is_set___shada_buflist_item_
                                            as ::core::ffi::c_ulonglong
                                            & (1 as ::core::ffi::c_ulonglong)
                                                << KEYSET_OPTIDX__shada_buflist_item__l
                                            != 0 as ::core::ffi::c_ulonglong
                                        {
                                            (*e).pos.lnum = it_2.l as linenr_T;
                                        }
                                        if it_2.is_set___shada_buflist_item_
                                            as ::core::ffi::c_ulonglong
                                            & (1 as ::core::ffi::c_ulonglong)
                                                << KEYSET_OPTIDX__shada_buflist_item__c
                                            != 0 as ::core::ffi::c_ulonglong
                                        {
                                            (*e).pos.col = it_2.c as colnr_T;
                                        }
                                        if it_2.is_set___shada_buflist_item_
                                            as ::core::ffi::c_ulonglong
                                            & (1 as ::core::ffi::c_ulonglong)
                                                << KEYSET_OPTIDX__shada_buflist_item__f
                                            != 0 as ::core::ffi::c_ulonglong
                                        {
                                            (*e).fname = xmemdupz(
                                                it_2.f.data as *const ::core::ffi::c_void,
                                                it_2.f.size,
                                            )
                                                as *mut ::core::ffi::c_char;
                                        }
                                        if (*e).pos.lnum <= 0 as linenr_T {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid line number\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        } else if (*e).pos.col < 0 as ::core::ffi::c_int {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry with invalid column number\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        } else if (*e).fname.is_null() {
                                            semsg(
                                                gettext(
                                                    b"E575: Error while reading ShaDa file: buffer list at position %lu contains entry that does not have a file name\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                initial_fpos,
                                            );
                                            break '_shada_read_next_item_error;
                                        } else {
                                            i = i.wrapping_add(1);
                                        }
                                    }
                                }
                            }
                        }
                        0 | -1 => {
                            abort();
                        }
                        1 | _ => {}
                    }
                }
                let mut i_0: uint32_t = 0 as uint32_t;
                while i_0 < read_additional_array_elements {
                    let mut item_start: *const ::core::ffi::c_char = read_ptr;
                    let mut status_2: ::core::ffi::c_int =
                        unpack_skip(&raw mut read_ptr, &raw mut read_size);
                    if status_2 != 0 {
                        break '_shada_read_next_item_error;
                    }
                    push_additional_data(
                        &raw mut ad,
                        item_start,
                        read_ptr.offset_from(item_start) as size_t,
                    );
                    i_0 = i_0.wrapping_add(1);
                }
                if read_size != 0 {
                    semsg(
                        gettext(
                            b"E575: Error while reading ShaDa file: item entry at position %lu additional bytes\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        initial_fpos,
                    );
                    break;
                } else {
                    (*entry).type_0 = type_u64 as ShadaEntryType;
                    (*entry).additional_data = ad.items as *mut AdditionalData;
                    ret = kSDReadStatusSuccess;
                    break '_shada_read_next_item_end;
                }
            }
        }
        (*entry).type_0 = type_u64 as ShadaEntryType;
        shada_free_shada_entry(entry);
        (*entry).type_0 = kSDItemMissing;
        xfree(error_alloc as *mut ::core::ffi::c_void);
        xfree(ad.items as *mut ::core::ffi::c_void);
        ad.capacity = 0 as size_t;
        ad.size = ad.capacity;
        ad.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if buf_allocated {
        xfree(buf as *mut ::core::ffi::c_void);
    }
    return ret;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
