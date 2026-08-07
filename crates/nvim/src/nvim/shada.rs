#![deny(unsafe_op_in_unsafe_fn)]

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
use crate::src::nvim::os::libc::{atoi, getgid, gettext, getuid, qsort, strcmp, strlen};
use crate::src::nvim::os::stdpaths::stdpaths_user_state_subpath;
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::path::{
    concat_fnames_realloc, path_fnamecmp, path_tail_with_sep, path_try_shorten_fname,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::regexp::regtilde;
use crate::src::nvim::register::op_global_reg_iter;
use crate::src::nvim::register::op_reg_get;
use crate::src::nvim::register::op_reg_index;
use crate::src::nvim::register::op_reg_set;
use crate::src::nvim::search::{
    get_search_pattern, get_substitute_pattern, search_was_last_used, set_last_used_pattern,
    set_search_pattern, set_substitute_pattern,
};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    __uid_t, AdditionalData, AdditionalDataBuilder, Arena, Dict, FileDescriptor, FileInfo, Integer,
    KeyDict__shada_buflist_item, KeyDict__shada_mark, KeyDict__shada_register,
    KeyDict__shada_search_pat, KeyValuePair, MHPutStatus, Map_cstr_t_ptr_t, MapHash, MarkGet,
    MotionType, OptionalKeys, PackerBuffer, SearchOffset, SearchPattern, Set_cstr_t, Set_ptr_t,
    String_0, StringArray, SubReplacementString, Timestamp, VAR_UNKNOWN, VAR_UNLOCKED, bln_values,
    buf_T, colnr_T, cstr_t, dictitem_T, fmark_T, fmarkv_T, hashitem_T, int64_t, kObjectTypeInteger,
    kObjectTypeString, linenr_T, list_T, pos_T, ptr_t, ptrdiff_t, size_t, ssize_t, tabpage_T,
    typval_T, typval_vval_union, uint8_t, uint32_t, uint64_t, uintmax_t, uv_gid_t, uv_uid_t,
    var_flavour_T, win_T, xfmark_T, yankreg_T,
};
use crate::src::nvim::version::longVersion;

// The carve of the transpiled module; see each child's docs.
mod file;
pub use self::file::*;
mod pack;
pub(crate) use self::pack::*;
mod parse;
pub(crate) use self::parse::*;
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
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const kMHExisting: MHPutStatus = 0;
pub const kMarkBufLocal: MarkGet = 0;
pub const BLN_LISTED: bln_values = 2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_17 = 1;
pub const HIST_CMD: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const HIST_COUNT: C2Rust_Unnamed_18 = 5;
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
pub const PTRDIFF_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = 4096;
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
pub const MH_TOMBSTONE: ::core::ffi::c_uint = u32::MAX;
#[inline]
/// Add a key to a set, answering whether it was not already there. When
/// `key_alloc` is given it is pointed at the set's own copy of the key,
/// which the caller may then replace with an owned one.
unsafe fn set_put_cstr_t(set: *mut Set_cstr_t, key: cstr_t, key_alloc: *mut *mut cstr_t) -> bool {
    unsafe {
        let mut status: MHPutStatus = kMHExisting;
        let k = mh_put_cstr_t(set, key, &raw mut status);
        if !key_alloc.is_null() {
            *key_alloc = (*set).keys.add(k as usize);
        }
        status != kMHExisting
    }
}

/// Whether a set holds a key.
unsafe fn set_has_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> bool {
    unsafe { mh_get_cstr_t(set, key) != MH_TOMBSTONE }
}

/// [`set_put_cstr_t`] for a set of pointers.
unsafe fn set_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, key_alloc: *mut *mut ptr_t) -> bool {
    unsafe {
        let mut status: MHPutStatus = kMHExisting;
        let k = mh_put_ptr_t(set, key, &raw mut status);
        if !key_alloc.is_null() {
            *key_alloc = (*set).keys.add(k as usize);
        }
        status != kMHExisting
    }
}

/// [`set_has_cstr_t`] for a set of pointers.
unsafe fn set_has_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> bool {
    unsafe { mh_get_ptr_t(set, key) != MH_TOMBSTONE }
}

/// Free a set's own allocations, leaving it empty. What the keys point at
/// is the caller's business.
unsafe fn set_destroy_cstr_t(set: *mut Set_cstr_t) {
    unsafe {
        xfree((*set).keys.cast());
        xfree((*set).h.hash.cast());
        *set = Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut(),
        };
    }
}

/// [`set_destroy_cstr_t`] for a set of pointers.
unsafe fn set_destroy_ptr_t(set: *mut Set_ptr_t) {
    unsafe {
        xfree((*set).keys.cast());
        xfree((*set).h.hash.cast());
        *set = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut(),
        };
    }
}

/// [`set_destroy_cstr_t`] for a map. Neither the keys nor the values are
/// freed; both belong to whoever put them in.
unsafe fn map_destroy_cstr_t_ptr_t(map: *mut Map_cstr_t_ptr_t) {
    unsafe {
        set_destroy_cstr_t(&raw mut (*map).set);
        xfree((*map).values.cast());
        (*map).values = ::core::ptr::null_mut();
    }
}

/// Every window in every tabpage.
///
/// The current tabpage keeps its window list in `firstwin` rather than in
/// the tabpage struct, which is why this is not a plain walk of
/// `tp_firstwin`.
unsafe fn all_windows() -> impl Iterator<Item = *mut win_T> {
    let mut tp = first_tabpage.get() as *mut tabpage_T;
    let mut wp: *mut win_T = ::core::ptr::null_mut();
    ::core::iter::from_fn(move || {
        // SAFETY: walking the editor's window lists on the main thread. No
        // caller restructures them while iterating.
        unsafe {
            while wp.is_null() {
                if tp.is_null() {
                    return None;
                }
                wp = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            let found = wp;
            wp = (*found).w_next;
            Some(found)
        }
    })
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
/// Where a global mark's letter lives in `namedfm`: `A`-`Z` first, then the
/// ten numbered marks. −1 for a name that is neither.
fn mark_global_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    match name as u8 {
        b'A'..=b'Z' => name as ::core::ffi::c_int - 'A' as ::core::ffi::c_int,
        b'0'..=b'9' => NMARKS + (name as ::core::ffi::c_int - '0' as ::core::ffi::c_int),
        _ => -1,
    }
}

/// Where a buffer-local mark's name lives in a buffer's mark array: `a`-`z`
/// first, then the three special ones. −1 for a name that is none of them.
fn mark_local_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    match name as u8 {
        b'a'..=b'z' => name as ::core::ffi::c_int - 'a' as ::core::ffi::c_int,
        b'"' => NMARKS,
        b'^' => NMARKS + 1,
        b'.' => NMARKS + 2,
        _ => -1,
    }
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
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
