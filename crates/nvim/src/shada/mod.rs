#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::dispatch::{
    key_dict__shada_buflist_item_get_field, key_dict__shada_mark_get_field,
    key_dict__shada_register_get_field, key_dict__shada_search_pat_get_field,
};
use crate::api::private::helpers::{api_free_dict, api_free_string, copy_string, cstr_as_string};
use crate::ascii::ascii_isdigit;
use crate::buffer::{buf_is_quickfix, buf_is_terminal, buflist_new, buflist_setfpos, find_buf};
use crate::cmdhist::{HistShadaEntry, hist_shada_replace, hist_shada_take, hist_shada_view};
use crate::eval::decode::{decode_string, unpack_typval};
use crate::eval::encode::encode_vim_to_msgpack;
use crate::eval::typval::{
    tv_clear, tv_copy, tv_list_alloc, tv_list_append_allocated_string, tv_list_len,
};
use crate::eval::vars::{get_globvar_ht, get_vim_var_list, set_vim_var_list};
use crate::eval::{get_copy_id, set_ref_in_ht, set_ref_in_list_items, var_flavour, var_set_global};
use crate::event::libuv::uv_strerror;
use crate::ex_cmds::{sub_get_replacement, sub_set_replacement};
use crate::ex_docmd::set_no_hlsearch;
use crate::fileio::{modname, vim_rename};
use crate::global_cell::GlobalCell;
use crate::main::{
    curbuf, curwin, no_hlsearch, p_enc, p_fs, p_hi, p_shada, p_shadafile, p_verbose,
};
use crate::map::{
    map_del_cstr_t_ptr_t, map_put_ref_cstr_t_ptr_t, map_ref_cstr_t_ptr_t, mh_get_cstr_t,
    mh_get_ptr_t, mh_put_cstr_t, mh_put_ptr_t,
};
use crate::mark::{
    cleanup_jumplist, free_fmark, free_xfmark, mark_buffer_iter, mark_get, mark_global_iter,
    mark_jumplist_iter, mark_set_global, mark_set_local, set_last_cursor, setpcmark,
};
use crate::mbyte::mb_strnicmp;
use crate::memory::{strequal, xcalloc, xfree, xmalloc, xmemdup, xmemdupz, xrealloc, xstrdup};
use crate::message::{verbose_enter, verbose_leave};
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_bin, mpack_bool, mpack_check_buffer, mpack_integer, mpack_map, mpack_raw,
    mpack_remaining, mpack_str, mpack_uint, mpack_uint64, packer_string_buffer, packer_take_string,
};
use crate::msgpack_rpc::unpacker::{
    push_additional_data, unpack_array, unpack_integer, unpack_keydict, unpack_skip, unpack_string,
};
use crate::option::{copy_option_part, magic_isset};
use crate::os::env::{expand_env, home_replace, home_replace_save, os_get_pid};
use crate::os::fileio::{
    FileOpenFlags, file_close, file_flush, file_open, file_open_buffer, file_read, file_skip,
    file_try_read_buffered,
};
use crate::os::fs::{os_fchown, os_fileinfo, os_getperm, os_isdir, os_mkdir_recurse, os_remove};
use crate::os::stdpaths::stdpaths_user_state_subpath;
use crate::os::time::os_time;
use crate::path::{
    concat_fnames_realloc, path_fnamecmp, path_tail_with_sep, path_try_shorten_fname,
};
use crate::pos::MAXLNUM;
use crate::regexp::regtilde;
use crate::register::{op_global_reg_iter, op_reg_get, op_reg_index, op_reg_set};
use crate::search::{
    get_search_pattern, get_substitute_pattern, search_was_last_used, set_last_used_pattern,
    set_search_pattern, set_substitute_pattern,
};
use crate::strings::vim_strchr;
use crate::types::{
    AdditionalData, AdditionalDataBuilder, Arena, Dict, FileDescriptor, FileInfo, HistoryType,
    Integer, KeyDict__shada_buflist_item, KeyDict__shada_mark, KeyDict__shada_register,
    KeyDict__shada_search_pat, KeyValuePair, MHPutStatus, Map_cstr_t_ptr_t, MapHash, MarkGet,
    MotionType, OptionalKeys, PackerBuffer, SearchOffset, SearchPattern, Set_cstr_t, Set_ptr_t,
    String_0, StringArray, SubReplacementString, Timestamp, VAR_UNKNOWN, VarLock, bln_values,
    buf_T, colnr_T, cstr_t, dictitem_T, fmark_T, fmarkv_T, int64_t, linenr_T, list_T, pos_T, ptr_t,
    ptrdiff_t, size_t, ssize_t, typval_T, typval_vval_union, uid_t, uint8_t, uint32_t, uint64_t,
    uintmax_t, uv_gid_t, uv_uid_t, var_flavour_T, xfmark_T, yankreg_T,
};
use crate::version::LONG_VERSION;
use crate::winlayer::{buffers, tab_windows};
use ::libc::{atoi, getgid, getuid, qsort};

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
pub const kMHExisting: MHPutStatus = 0;
pub const kMarkBufLocal: MarkGet = 0;
pub const BLN_LISTED: bln_values = 2;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_COUNT: ::core::ffi::c_uint = 5;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const kFileTruncate: FileOpenFlags = 32;
pub const kFileCreateOnly: FileOpenFlags = 16;
pub const kFileNoSymlink: FileOpenFlags = 8;
pub const kFileCreate: FileOpenFlags = 2;
pub const kFileReadOnly: FileOpenFlags = 1;
pub const kShaDaMissingError: ::core::ffi::c_uint = 16;
pub const kShaDaGetOldfiles: ::core::ffi::c_uint = 8;
pub const kShaDaForceit: ::core::ffi::c_uint = 4;
pub const kShaDaWantMarks: ::core::ffi::c_uint = 2;
pub const kShaDaWantInfo: ::core::ffi::c_uint = 1;
pub const kSDWriteReadNotShada: ShaDaWriteResult = 1;
pub type ShaDaWriteResult = ::core::ffi::c_uint;
pub const kSDWriteIgnError: ShaDaWriteResult = 3;
pub const kSDWriteFailed: ShaDaWriteResult = 2;
pub const kSDWriteSuccessful: ShaDaWriteResult = 0;
#[derive(Clone)]
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
pub struct ShadaEntry {
    pub can_free_entry: bool,
    pub timestamp: Timestamp,
    pub data: ShadaEntryData,
    pub additional_data: *mut AdditionalData,
}

impl ShadaEntry {
    /// An empty slot: what a merger's arrays are filled with, and what an
    /// entry that failed to read is left as.
    pub(crate) const MISSING: ShadaEntry = ShadaEntry {
        can_free_entry: false,
        timestamp: 0,
        data: ShadaEntryData::Missing,
        additional_data: ::core::ptr::null_mut(),
    };

    /// Which kind of entry this is — the number the format writes for it.
    pub(crate) fn kind(&self) -> ShadaEntryType {
        self.data.kind()
    }
}

/// One entry's payload, and by which variant it is, the kind of entry.
///
/// Upstream this is an untagged union beside a separate `type` field; the
/// two are never written apart, so they are one value here and the reads
/// that used to have to trust the tag are `match` arms.
#[derive(Copy, Clone)]
pub enum ShadaEntryData {
    /// No entry at all. An empty slot in a merger's arrays, and what a
    /// malformed entry is reduced to.
    Missing,
    Header(Dict),
    SearchPattern(KeyDict__shada_search_pat),
    SubString(sub_string),
    HistoryEntry(history_item),
    Register(reg),
    Variable(global_var),
    GlobalMark(shada_filemark),
    Jump(shada_filemark),
    BufferList(buffer_list),
    LocalMark(shada_filemark),
    Change(shada_filemark),
    /// An entry of a type this Nvim does not know, kept byte for byte so
    /// that writing the file back does not lose it.
    Unknown(unknown_item),
}

impl ShadaEntryData {
    /// The number the format writes for this kind of entry. An unknown
    /// entry answers [`kSDItemUnknown`], not the type it arrived with.
    pub(crate) fn kind(&self) -> ShadaEntryType {
        match self {
            ShadaEntryData::Missing => kSDItemMissing,
            ShadaEntryData::Header(_) => kSDItemHeader,
            ShadaEntryData::SearchPattern(_) => kSDItemSearchPattern,
            ShadaEntryData::SubString(_) => kSDItemSubString,
            ShadaEntryData::HistoryEntry(_) => kSDItemHistoryEntry,
            ShadaEntryData::Register(_) => kSDItemRegister,
            ShadaEntryData::Variable(_) => kSDItemVariable,
            ShadaEntryData::GlobalMark(_) => kSDItemGlobalMark,
            ShadaEntryData::Jump(_) => kSDItemJump,
            ShadaEntryData::BufferList(_) => kSDItemBufferList,
            ShadaEntryData::LocalMark(_) => kSDItemLocalMark,
            ShadaEntryData::Change(_) => kSDItemChange,
            ShadaEntryData::Unknown(_) => kSDItemUnknown,
        }
    }

    /// Whether this is an empty slot.
    pub(crate) fn is_missing(&self) -> bool {
        matches!(self, ShadaEntryData::Missing)
    }

    /// What the entry's fields default to when the file leaves them out.
    /// Both sides of the format agree on these, which is what lets a
    /// writer omit them.
    pub(crate) fn default_for(kind: ShadaEntryType) -> ShadaEntryData {
        match kind {
            kSDItemHeader => ShadaEntryData::Header(EMPTY_DICT),
            kSDItemSearchPattern => ShadaEntryData::SearchPattern(DEFAULT_SEARCH_PATTERN),
            kSDItemSubString => ShadaEntryData::SubString(DEFAULT_SUB_STRING),
            kSDItemHistoryEntry => ShadaEntryData::HistoryEntry(DEFAULT_HISTORY_ITEM),
            kSDItemRegister => ShadaEntryData::Register(DEFAULT_REGISTER),
            kSDItemVariable => ShadaEntryData::Variable(DEFAULT_VARIABLE),
            kSDItemGlobalMark => ShadaEntryData::GlobalMark(default_filemark(kind)),
            kSDItemJump => ShadaEntryData::Jump(default_filemark(kind)),
            kSDItemBufferList => ShadaEntryData::BufferList(DEFAULT_BUFFER_LIST),
            kSDItemLocalMark => ShadaEntryData::LocalMark(default_filemark(kind)),
            kSDItemChange => ShadaEntryData::Change(default_filemark(kind)),
            _ => ShadaEntryData::Missing,
        }
    }

    /// The mark a global mark, local mark, jump or change entry carries.
    pub(crate) fn filemark(&self) -> shada_filemark {
        match self {
            ShadaEntryData::GlobalMark(mark)
            | ShadaEntryData::Jump(mark)
            | ShadaEntryData::LocalMark(mark)
            | ShadaEntryData::Change(mark) => *mark,
            other => unreachable!("shada: entry type {} carries no mark", other.kind()),
        }
    }

    /// [`Self::filemark`], to write to.
    pub(crate) fn filemark_mut(&mut self) -> &mut shada_filemark {
        match self {
            ShadaEntryData::GlobalMark(mark)
            | ShadaEntryData::Jump(mark)
            | ShadaEntryData::LocalMark(mark)
            | ShadaEntryData::Change(mark) => mark,
            other => unreachable!("shada: entry type {} carries no mark", other.kind()),
        }
    }

    /// The search or substitute pattern a search-pattern entry carries.
    pub(crate) fn search_pattern_mut(&mut self) -> &mut KeyDict__shada_search_pat {
        match self {
            ShadaEntryData::SearchPattern(pattern) => pattern,
            other => unreachable!("shada: entry type {} is not a search pattern", other.kind()),
        }
    }

    /// The line a history entry carries.
    pub(crate) fn history(&self) -> history_item {
        match self {
            ShadaEntryData::HistoryEntry(item) => *item,
            other => unreachable!("shada: entry type {} is not a history entry", other.kind()),
        }
    }

    /// [`Self::history`], to write to.
    pub(crate) fn history_mut(&mut self) -> &mut history_item {
        match self {
            ShadaEntryData::HistoryEntry(item) => item,
            other => unreachable!("shada: entry type {} is not a history entry", other.kind()),
        }
    }

    /// The register a register entry carries.
    pub(crate) fn register_mut(&mut self) -> &mut reg {
        match self {
            ShadaEntryData::Register(reg) => reg,
            other => unreachable!("shada: entry type {} is not a register", other.kind()),
        }
    }

    /// The variable a variable entry carries.
    pub(crate) fn variable_mut(&mut self) -> &mut global_var {
        match self {
            ShadaEntryData::Variable(var) => var,
            other => unreachable!("shada: entry type {} is not a variable", other.kind()),
        }
    }

    /// The replacement string a sub-string entry carries.
    pub(crate) fn sub_string_mut(&mut self) -> &mut sub_string {
        match self {
            ShadaEntryData::SubString(sub) => sub,
            other => unreachable!("shada: entry type {} is not a sub string", other.kind()),
        }
    }

    /// The buffer list a buffer-list entry carries.
    pub(crate) fn buffer_list(&self) -> buffer_list {
        match self {
            ShadaEntryData::BufferList(list) => *list,
            other => unreachable!("shada: entry type {} is not a buffer list", other.kind()),
        }
    }

    /// [`Self::buffer_list`], to write to.
    pub(crate) fn buffer_list_mut(&mut self) -> &mut buffer_list {
        match self {
            ShadaEntryData::BufferList(list) => list,
            other => unreachable!("shada: entry type {} is not a buffer list", other.kind()),
        }
    }

    /// The bytes an entry of an unrecognised type arrived as.
    pub(crate) fn unknown_mut(&mut self) -> &mut unknown_item {
        match self {
            ShadaEntryData::Unknown(item) => item,
            other => unreachable!("shada: entry type {} is a known one", other.kind()),
        }
    }
}
#[derive(Copy, Clone)]
pub struct buffer_list {
    pub size: size_t,
    pub buffers: *mut buffer_list_buffer,
}
#[derive(Copy, Clone)]
pub struct buffer_list_buffer {
    pub pos: pos_T,
    pub fname: *mut ::core::ffi::c_char,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
pub struct sub_string {
    pub sub: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct unknown_item {
    pub type_0: uint64_t,
    pub contents: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
pub struct global_var {
    pub name: *mut ::core::ffi::c_char,
    pub value: typval_T,
}
#[derive(Copy, Clone)]
pub struct reg {
    pub name: ::core::ffi::c_char,
    pub type_0: MotionType,
    pub contents: *mut String_0,
    pub is_unnamed: bool,
    pub contents_size: size_t,
    pub width: size_t,
}
#[derive(Copy, Clone)]
pub struct history_item {
    pub histtype: uint8_t,
    pub string: *mut ::core::ffi::c_char,
    pub sep: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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

impl HMLList {
    /// A ring with no array yet: what a merger holds before `hmll_init`.
    const EMPTY: HMLList = HMLList {
        entries: ::core::ptr::null_mut(),
        first: ::core::ptr::null_mut(),
        last: ::core::ptr::null_mut(),
        free_entry: ::core::ptr::null_mut(),
        last_free_entry: ::core::ptr::null_mut(),
        size: 0,
        num_entries: 0,
        contained_entries: MAP_INIT,
    };
}

impl HistoryMergerState {
    /// A merger that has not been started. `hms_init` makes a usable one.
    const EMPTY: HistoryMergerState = HistoryMergerState {
        hmll: HMLList::EMPTY,
        do_merge: false,
        reading: false,
        pending: ::core::ptr::null_mut(),
        pending_len: 0,
        pending_pos: 0,
        history_type: 0,
    };
}

impl WriteMergerState {
    /// Nothing collected yet. Every slot is [`ShadaEntry::MISSING`], which
    /// is what marks one as empty.
    pub(crate) const EMPTY: WriteMergerState = WriteMergerState {
        hms: [HistoryMergerState::EMPTY; 5],
        global_marks: [ShadaEntry::MISSING; 26],
        numbered_marks: [ShadaEntry::MISSING; 10],
        registers: [ShadaEntry::MISSING; 37],
        jumps: [ShadaEntry::MISSING; 100],
        jumps_size: 0,
        search_pattern: ShadaEntry::MISSING,
        sub_search_pattern: ShadaEntry::MISSING,
        replacement: ShadaEntry::MISSING,
        dumped_variables: SET_CSTR_INIT,
        file_marks: MAP_INIT,
    };
}

/// One `value` on the heap, for the aggregates the merge allocates.
///
/// `xmalloc` rather than `Box` because these are released with `xfree`, and
/// written whole rather than `xcalloc`ed because an all-zero
/// [`ShadaEntryData`] is not a value of it — [`ShadaEntry::MISSING`] is.
fn shada_heap<T>(value: T) -> *mut T {
    // SAFETY: fresh storage of exactly `T`'s size, written before it is read.
    let ptr = unsafe { xmalloc(size_of::<T>()) }.cast::<T>();
    unsafe { ptr.write(value) };
    ptr
}

pub type HMLListEntry = hm_llist_entry;
pub struct hm_llist_entry {
    pub data: ShadaEntry,
    pub next: *mut hm_llist_entry,
    pub prev: *mut hm_llist_entry,
}
pub struct FileMarks {
    pub marks: [ShadaEntry; 29],
    pub changes: [ShadaEntry; 100],
    pub changes_size: size_t,
    pub additional_marks: *mut ShadaEntry,
    pub additional_marks_size: size_t,
    pub greatest_timestamp: Timestamp,
}

impl FileMarks {
    /// One file's marks before any have been collected.
    pub(crate) const EMPTY: FileMarks = FileMarks {
        marks: [ShadaEntry::MISSING; 29],
        changes: [ShadaEntry::MISSING; 100],
        changes_size: 0,
        additional_marks: ::core::ptr::null_mut(),
        additional_marks_size: 0,
        greatest_timestamp: 0,
    };
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
pub type SearchPatternGetter = Option<unsafe fn(*mut SearchPattern) -> ()>;
pub const kSDReadBufferList: SRNIFlags = 512;
pub type SRNIFlags = ::core::ffi::c_uint;
pub const PTRDIFF_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
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
const SET_CSTR_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
const SET_PTR_INIT: Set_ptr_t = Set_ptr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: SET_CSTR_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = u32::MAX;
#[inline]
/// Add a key to a set, answering whether it was not already there. When
/// `key_alloc` is given it is pointed at the set's own copy of the key,
/// which the caller may then replace with an owned one.
unsafe fn set_put_cstr_t(set: *mut Set_cstr_t, key: cstr_t, key_alloc: *mut *mut cstr_t) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let k = unsafe { mh_put_cstr_t(set, key, &raw mut status) };
    if !key_alloc.is_null() {
        unsafe { *key_alloc = (*set).keys.add(k as usize) };
    }
    status != kMHExisting
}

/// Whether a set holds a key.
unsafe fn set_has_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> bool {
    unsafe { mh_get_cstr_t(set, key) != MH_TOMBSTONE }
}

/// [`set_put_cstr_t`] for a set of pointers.
unsafe fn set_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, key_alloc: *mut *mut ptr_t) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let k = unsafe { mh_put_ptr_t(set, key, &raw mut status) };
    if !key_alloc.is_null() {
        unsafe { *key_alloc = (*set).keys.add(k as usize) };
    }
    status != kMHExisting
}

/// [`set_has_cstr_t`] for a set of pointers.
unsafe fn set_has_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> bool {
    unsafe { mh_get_ptr_t(set, key) != MH_TOMBSTONE }
}

/// Free a set's own allocations, leaving it empty. What the keys point at
/// is the caller's business.
unsafe fn set_destroy_cstr_t(set: *mut Set_cstr_t) {
    unsafe { xfree((*set).keys.cast()) };
    unsafe { xfree((*set).h.hash.cast()) };
    unsafe { *set = SET_CSTR_INIT };
}

/// [`set_destroy_cstr_t`] for a set of pointers.
unsafe fn set_destroy_ptr_t(set: *mut Set_ptr_t) {
    unsafe { xfree((*set).keys.cast()) };
    unsafe { xfree((*set).h.hash.cast()) };
    unsafe { *set = SET_PTR_INIT };
}

/// [`set_destroy_cstr_t`] for a map. Neither the keys nor the values are
/// freed; both belong to whoever put them in.
unsafe fn map_destroy_cstr_t_ptr_t(map: *mut Map_cstr_t_ptr_t) {
    unsafe { set_destroy_cstr_t(&raw mut (*map).set) };
    unsafe { xfree((*map).values.cast()) };
    unsafe { (*map).values = ::core::ptr::null_mut() };
}

pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const JUMPLISTSIZE: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
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
/// The empty dictionary a header entry starts as.
const EMPTY_DICT: Dict = Dict {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};

/// What a search-pattern entry's fields default to. A pattern is magic and
/// was the last one used unless the file says otherwise.
const DEFAULT_SEARCH_PATTERN: KeyDict__shada_search_pat = KeyDict__shada_search_pat {
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
    pat: String_0::NULL,
};

/// What a sub-string entry defaults to.
const DEFAULT_SUB_STRING: sub_string = sub_string {
    sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};

/// What a history entry defaults to: the command-line history, no separator.
const DEFAULT_HISTORY_ITEM: history_item = history_item {
    histtype: HIST_CMD as uint8_t,
    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sep: 0,
};

/// What a register entry defaults to: charwise, unnamed, no width.
const DEFAULT_REGISTER: reg = reg {
    name: 0,
    type_0: kMTCharWise,
    contents: ::core::ptr::null_mut::<String_0>(),
    is_unnamed: false,
    contents_size: 0 as size_t,
    width: 0 as size_t,
};

/// What a variable entry defaults to.
const DEFAULT_VARIABLE: global_var = global_var {
    name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    value: typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union {
            v_string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
    },
};

/// What a buffer-list entry defaults to.
const DEFAULT_BUFFER_LIST: buffer_list = buffer_list {
    size: 0 as size_t,
    buffers: ::core::ptr::null_mut::<buffer_list_buffer>(),
};

/// What a mark entry's fields default to. Only the name differs between the
/// kinds: a jump and a change have none, while a mark the file does not name
/// is the `"` one.
const fn default_filemark(kind: ShadaEntryType) -> shada_filemark {
    shada_filemark {
        name: match kind {
            kSDItemGlobalMark | kSDItemLocalMark => b'"' as ::core::ffi::c_char,
            _ => 0,
        },
        mark: DEFAULT_POS,
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }
}

pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
