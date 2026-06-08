//! Encode Vimscript `typval_T` values to string/echo/JSON/msgpack forms.
//!
//! Port of `src/nvim/eval/encode.c` and the X-macro template
//! `src/nvim/eval/typval_encode.c.h` (string, echo, json, msgpack instantiations).

#![allow(unsafe_code)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::if_not_else)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_assert)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::redundant_else)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unreadable_literal)]
// FFI offset-based casts require pointer alignment exceptions.
#![allow(clippy::cast_ptr_alignment)]
// Saturating subtraction in bounds checks is intentional.
#![allow(clippy::implicit_saturating_sub)]
// Late initialization of `cur_tv` is intentional in the encoder loop.
#![allow(clippy::needless_late_init)]
// Comparing raw pointers with == is intentional (pointer identity).
#![allow(clippy::ptr_eq)]
// Unused dead code: extern declarations kept for future use or documentation.
#![allow(dead_code)]
// Mutable static references are single-threaded (nvim main thread).
#![allow(static_mut_refs)]

use std::ffi::{c_char, c_int, c_void};

use crate::GarrayT;

// =============================================================================
// Opaque handle aliases
// =============================================================================
type ListHandle = *mut c_void;
type ListItemHandle = *mut c_void;
type TypevalHandle = *mut c_void;
type DictHandle = *mut c_void;
type HashItemHandle = *mut c_void;
type BlobHandle = *mut c_void;
type PartialHandle = *mut c_void;

// =============================================================================
// Constants (matching C)
// =============================================================================
const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = -1;

const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;
const VAR_PARTIAL: c_int = 9;
const VAR_BLOB: c_int = 10;

const K_BOOL_VAR_FALSE: c_int = 0;
const K_BOOL_VAR_TRUE: c_int = 1;
const K_SPECIAL_VAR_NULL: c_int = 0;

// MessagePackType indices
const KMP_NIL: usize = 0;
const KMP_BOOLEAN: usize = 1;
const KMP_INTEGER: usize = 2;
const KMP_FLOAT: usize = 3;
const KMP_STRING: usize = 4;
const KMP_ARRAY: usize = 5;
const KMP_MAP: usize = 6;
const KMP_EXT: usize = 7;
const KMP_COUNT: usize = 8;

const FP_NAN: c_int = 0;
const FP_INFINITE: c_int = 1;

const NUMBUFLEN: usize = 65;
const IOSIZE: usize = 1025;

// =============================================================================
// PackerBuffer mirror (matches C packer_defs.h)
// =============================================================================

/// Rust mirror of C `PackerBuffer`.
#[repr(C)]
pub struct PackerBuffer {
    pub startptr: *mut c_char,
    pub ptr: *mut c_char,
    pub endptr: *mut c_char,
    pub anydata: *mut c_void,
    pub anyint: i64,
    pub packer_flush: Option<unsafe extern "C" fn(*mut PackerBuffer)>,
}

// =============================================================================
// C function declarations
// =============================================================================

extern "C" {
    // List
    fn nvim_list_get_first(l: ListHandle) -> ListItemHandle;
    fn nvim_list_get_last(l: ListHandle) -> ListItemHandle;
    fn nvim_list_get_len(l: ListHandle) -> c_int;
    fn nvim_list_get_copyid(l: ListHandle) -> c_int;
    fn nvim_list_set_copyid(l: ListHandle, copyid: c_int);
    fn nvim_listitem_get_next(li: ListItemHandle) -> ListItemHandle;
    fn nvim_listitem_get_tv(li: ListItemHandle) -> TypevalHandle;
    fn tv_list_idx_of_item(l: ListHandle, li: ListItemHandle) -> c_int;
    fn tv_list_append_allocated_string(list: ListHandle, s: *mut c_char);

    // Dict
    fn nvim_dict_get_ht_used(d: DictHandle) -> usize;
    fn nvim_dict_get_ht_array(d: DictHandle) -> HashItemHandle;
    fn nvim_hash_removed_ptr() -> *const c_char;
    fn nvim_hashitem_to_dictitem(hi: HashItemHandle) -> *mut c_void; // dictitem_T*
    fn nvim_dictitem_get_key(di: *mut c_void) -> *const c_char;
    fn nvim_dictitem_di_tv(di: *mut c_void) -> TypevalHandle;
    /// `len` is `ptrdiff_t`/`isize`, NOT `c_int` — same as decode.rs declaration.
    fn tv_dict_find(d: DictHandle, key: *const c_char, len: isize) -> *mut c_void; // dictitem_T*

    // Blob
    fn nvim_blob_get_len(b: BlobHandle) -> c_int;
    fn nvim_blob_get_byte(b: BlobHandle, idx: c_int) -> u8;
    fn nvim_blob_get_ga_data(b: BlobHandle) -> *mut u8;

    // Partial (via rs_partial_name, plus direct struct access via offset helpers)
    fn rs_partial_name(pt: PartialHandle) -> *mut c_char;

    // Garray
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_clear(gap: *mut GarrayT);
    fn ga_concat(gap: *mut GarrayT, data: *const c_char);
    fn ga_append(gap: *mut GarrayT, c: u8);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn ga_concat_len(gap: *mut GarrayT, data: *const c_char, len: usize);

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
    fn xmemscan(addr: *const c_void, c: c_char, size: usize) -> *mut c_void;
    fn memchrsub(data: *mut c_void, c: c_char, x: c_char, len: usize);
    #[link_name = "strlen"]
    fn libc_strlen(s: *const c_char) -> usize;
    #[link_name = "memcpy"]
    fn libc_memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // Messages
    fn semsg(fmt: *const c_char, ...);
    fn emsg(fmt: *const c_char) -> c_int;
    fn internal_error(msg: *const c_char);

    // Copy ID
    fn rs_get_copyID() -> c_int;

    // Msgpack type list accessor
    fn nvim_eval_msgpack_type_list(idx: c_int) -> ListHandle;

    // Float classification
    fn xfpclassify(d: f64) -> c_int;

    // vim_snprintf
    fn vim_snprintf(dst: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;

    // IObuff global
    #[link_name = "IObuff"]
    static mut IOBUFF: [c_char; IOSIZE];

    // Packer (msgpack)
    fn mpack_check_buffer(packer: *mut PackerBuffer);
    fn mpack_integer(ptr: *mut *mut c_char, val: i64);
    fn mpack_float8(ptr: *mut *mut c_char, val: f64);
    fn mpack_uint64(ptr: *mut *mut c_char, val: u64);
    fn rs_mpack_nil(ptr: *mut *mut u8);
    fn rs_mpack_bool(ptr: *mut *mut u8, val: c_int);
    fn rs_mpack_array(ptr: *mut *mut u8, size: u32);
    fn rs_mpack_map(ptr: *mut *mut u8, size: u32);
    fn rs_mpack_bin(data: *const u8, len: usize, packer: *mut PackerBuffer);
    fn rs_mpack_str(data: *const u8, len: usize, packer: *mut PackerBuffer);
    #[link_name = "mpack_ext"]
    fn rs_mpack_ext(buf: *const c_char, len: usize, ty: i8, packer: *mut PackerBuffer);
    fn nvim_packer_get_ptr(packer: *mut PackerBuffer) -> *mut u8;
    fn nvim_packer_set_ptr(packer: *mut PackerBuffer, ptr: *mut u8);

    // rs_convert_to_json_string (already in Rust eval_codec lib.rs)
    fn rs_convert_to_json_string(gap: *mut GarrayT, buf: *const c_char, len: usize) -> c_int;
}

// =============================================================================
// ListReaderState (exported under its C name)
// =============================================================================

/// Rust mirror of C `ListReaderState` from encode.h.
///
/// Layout matches C struct exactly:
/// - list      (*const list_T)  at offset 0
/// - li        (*const listitem_T) at offset 8
/// - offset    (size_t) at offset 16
/// - li_length (size_t) at offset 24
#[repr(C)]
pub struct ListReaderState {
    pub list: ListHandle,
    pub li: ListItemHandle,
    pub offset: usize,
    pub li_length: usize,
}

// =============================================================================
// did_echo_string_emsg — exported as C-compatible bool
// =============================================================================

/// Abort-conversion latch exported to C as `did_echo_string_emsg`.
///
/// C code resets this to false; Rust code reads and writes it.
/// Using `static mut bool` because C reads it as a plain bool.
///
/// # Safety
/// Only accessed from single-threaded nvim main thread.
#[unsafe(no_mangle)]
pub static mut did_echo_string_emsg: bool = false;

// =============================================================================
// typval_T inline field accessors (offset-based, no C call overhead)
// =============================================================================
// Layout: v_type(i32, off0), v_lock(i32, off4), vval(8 bytes, off8)

#[inline]
unsafe fn tv_get_vtype(tv: TypevalHandle) -> c_int {
    *(tv as *const c_int)
}

#[inline]
unsafe fn tv_get_vstring(tv: TypevalHandle) -> *mut c_char {
    *(tv as *const u8).add(8).cast::<*mut c_char>()
}

#[inline]
unsafe fn tv_get_vnumber(tv: TypevalHandle) -> i64 {
    *(tv as *const u8).add(8).cast::<i64>()
}

#[inline]
unsafe fn tv_get_vfloat(tv: TypevalHandle) -> f64 {
    *(tv as *const u8).add(8).cast::<f64>()
}

#[inline]
unsafe fn tv_get_vbool(tv: TypevalHandle) -> c_int {
    *(tv as *const u8).add(8).cast::<c_int>()
}

#[inline]
unsafe fn tv_get_vspecial(tv: TypevalHandle) -> c_int {
    *(tv as *const u8).add(8).cast::<c_int>()
}

#[inline]
unsafe fn tv_get_vlist(tv: TypevalHandle) -> ListHandle {
    *(tv as *const u8).add(8).cast::<ListHandle>()
}

#[inline]
unsafe fn tv_get_vdict(tv: TypevalHandle) -> DictHandle {
    *(tv as *const u8).add(8).cast::<DictHandle>()
}

#[inline]
unsafe fn tv_get_vpartial(tv: TypevalHandle) -> PartialHandle {
    *(tv as *const u8).add(8).cast::<PartialHandle>()
}

#[inline]
unsafe fn tv_get_vblob(tv: TypevalHandle) -> BlobHandle {
    *(tv as *const u8).add(8).cast::<BlobHandle>()
}

#[inline]
unsafe fn tv_strlen_inner(tv: TypevalHandle) -> usize {
    let s = tv_get_vstring(tv);
    if s.is_null() {
        0
    } else {
        libc_strlen(s)
    }
}

// =============================================================================
// partial_T field accessors (layout from eval_struct_check.c)
// pt_argc at offset 28, pt_argv at offset 32, pt_dict at offset 40
// =============================================================================

#[inline]
unsafe fn partial_argc(pt: PartialHandle) -> c_int {
    *(pt as *const u8).add(28).cast::<c_int>()
}

#[inline]
unsafe fn partial_argv(pt: PartialHandle) -> TypevalHandle {
    *(pt as *const u8).add(32).cast::<TypevalHandle>()
}

#[inline]
unsafe fn partial_dict(pt: PartialHandle) -> DictHandle {
    *(pt as *const u8).add(40).cast::<DictHandle>()
}

// typval_T is 16 bytes; advance argv by 16
#[inline]
unsafe fn argv_next(argv: TypevalHandle) -> TypevalHandle {
    (argv as *mut u8).add(16) as TypevalHandle
}

// =============================================================================
// dict_T layout helpers (from eval_struct_check.c)
// dv_copyID at offset 12
// dv_hashtab at offset 16
// hashtab_T: ht_mask(8), ht_used(8), ht_filled(8), ht_changed(4), ht_locked(4), ht_array(8)
// => ht_used at offset 16+8=24, ht_array at offset 16+32=48
// =============================================================================

#[inline]
unsafe fn dict_get_copyid(d: DictHandle) -> c_int {
    *(d as *const u8).add(12).cast::<c_int>()
}

#[inline]
unsafe fn dict_set_copyid(d: DictHandle, id: c_int) {
    *(d as *mut u8).add(12).cast::<c_int>() = id;
}

#[inline]
unsafe fn dict_get_ht_used(d: DictHandle) -> usize {
    *(d as *const u8).add(24).cast::<usize>()
}

#[inline]
unsafe fn dict_get_ht_array(d: DictHandle) -> HashItemHandle {
    *(d as *const u8).add(48).cast::<HashItemHandle>()
}

// hashitem_T: hi_hash(usize=8), hi_key(*char=8) => hi_key at offset 8, total 16 bytes

#[inline]
unsafe fn hi_get_key(hi: HashItemHandle) -> *const c_char {
    *(hi as *const u8).add(8).cast::<*const c_char>()
}

#[inline]
fn hi_is_empty(hi: HashItemHandle, removed_ptr: *const c_char) -> bool {
    let key = unsafe { hi_get_key(hi) };
    key.is_null() || key == removed_ptr
}

// dictitem_T: di_tv(16), di_flags(1), di_key(flex @ offset 17)
// hi_key points to di_key[0], so dictitem_T* = hi_key - 17

#[inline]
unsafe fn hi_to_dictitem(hi: HashItemHandle) -> *mut c_void {
    let hi_key = hi_get_key(hi) as *const u8;
    hi_key.sub(17) as *mut c_void
}

#[inline]
unsafe fn hi_to_key(hi: HashItemHandle) -> *const c_char {
    hi_get_key(hi)
}

// Advance hashitem by 16 bytes (sizeof hashitem_T)
#[inline]
unsafe fn hi_advance(hi: HashItemHandle) -> HashItemHandle {
    (hi as *mut u8).add(16) as HashItemHandle
}

// list_T: lv_first at offset 0, lv_copyID at offset 68

#[inline]
unsafe fn list_get_copyid_raw(l: ListHandle) -> c_int {
    *(l as *const u8).add(68).cast::<c_int>()
}

#[inline]
unsafe fn list_set_copyid_raw(l: ListHandle, id: c_int) {
    *(l as *mut u8).add(68).cast::<c_int>() = id;
}

// listitem_T: li_next(8), li_prev(8), li_tv(16) => li_prev at offset 8

#[inline]
unsafe fn listitem_get_prev(li: ListItemHandle) -> ListItemHandle {
    *(li as *const u8).add(8).cast::<ListItemHandle>()
}

// =============================================================================
// Stack machine types
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
enum StackEntryType {
    Dict,
    List,
    Pairs,
    Partial,
    PartialList,
}

struct StackEntry {
    ty: StackEntryType,
    tv: TypevalHandle,
    saved_copyid: c_int,
    data: StackData,
}

enum StackData {
    Dict {
        dict: DictHandle,
        /// pointer to the dict field (tv->vval.v_dict or pt->pt_dict)
        dictp: *mut DictHandle,
        hi: HashItemHandle,
        todo: usize,
    },
    List {
        list: ListHandle,
        li: ListItemHandle,
    },
    Pairs {
        list: ListHandle,
        li: ListItemHandle,
    },
    Partial {
        stage: PartialStage,
        pt: PartialHandle,
    },
    PartialList {
        arg: TypevalHandle,
        argv: TypevalHandle,
        todo: usize,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum PartialStage {
    Args,
    Self_,
    End,
}

// =============================================================================
// Encoder trait
// =============================================================================

trait Encoder {
    unsafe fn check_before(&mut self);
    unsafe fn conv_nil(&mut self, tv: TypevalHandle);
    unsafe fn conv_bool(&mut self, tv: TypevalHandle, val: bool);
    unsafe fn conv_number(&mut self, tv: TypevalHandle, num: i64);
    unsafe fn conv_unsigned_number(&mut self, tv: TypevalHandle, num: u64);
    unsafe fn conv_float(&mut self, tv: TypevalHandle, flt: f64) -> c_int;
    unsafe fn conv_string(&mut self, tv: TypevalHandle, buf: *const c_char, len: usize) -> c_int;
    unsafe fn conv_str_string(
        &mut self,
        tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
    ) -> c_int;
    unsafe fn conv_ext_string(
        &mut self,
        tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
        ty: i64,
    ) -> c_int;
    unsafe fn conv_blob(&mut self, tv: TypevalHandle, blob: BlobHandle, len: c_int) -> c_int;
    unsafe fn conv_func_start(
        &mut self,
        tv: TypevalHandle,
        fun: *const c_char,
        mpstack: &[StackEntry],
        objname: *const c_char,
    ) -> c_int;
    unsafe fn conv_func_before_args(&mut self, tv: TypevalHandle, len: c_int);
    unsafe fn conv_func_before_self(&mut self, tv: TypevalHandle, len: isize);
    unsafe fn conv_func_end(&mut self, tv: TypevalHandle);
    unsafe fn conv_empty_list(&mut self, tv: TypevalHandle);
    unsafe fn conv_list_start(&mut self, tv: TypevalHandle, len: c_int);
    unsafe fn conv_real_list_after_start(&mut self, tv: TypevalHandle);
    unsafe fn conv_list_between_items(&mut self, tv: TypevalHandle);
    unsafe fn conv_list_end(&mut self, tv: TypevalHandle);
    unsafe fn conv_empty_dict(&mut self, tv: TypevalHandle, dict: DictHandle);
    unsafe fn conv_dict_start(&mut self, tv: TypevalHandle, dict: DictHandle, len: usize);
    unsafe fn conv_real_dict_after_start(&mut self, tv: TypevalHandle, dict: DictHandle);
    unsafe fn conv_dict_after_key(&mut self, tv: TypevalHandle, dict: DictHandle);
    unsafe fn conv_dict_between_items(&mut self, tv: TypevalHandle, dict: DictHandle);
    unsafe fn conv_dict_end(&mut self, tv: TypevalHandle, dict: DictHandle);
    unsafe fn conv_recurse(
        &mut self,
        val: *const c_void,
        conv_type: StackEntryType,
        mpstack: &[StackEntry],
        objname: *const c_char,
    ) -> c_int;
    unsafe fn special_dict_key_check(&mut self, key_tv: TypevalHandle) -> bool;
    fn allow_specials(&self) -> bool;
}

// =============================================================================
// conv_error — shared error path formatter
// =============================================================================

unsafe fn conv_error(msg: *const c_char, mpstack: &[StackEntry], objname: *const c_char) -> c_int {
    let mut msg_ga = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init(&mut msg_ga, 1, 80);

    let key_msg = b"key %s\0".as_ptr() as *const c_char;
    let key_pair_msg = b"key %s at index %i from special map\0".as_ptr() as *const c_char;
    let idx_msg = b"index %i\0".as_ptr() as *const c_char;
    let partial_arg_msg = b"partial\0".as_ptr() as *const c_char;
    let partial_arg_i_msg = b"argument %i\0".as_ptr() as *const c_char;
    let partial_self_msg = b"partial self dictionary\0".as_ptr() as *const c_char;

    for (i, entry) in mpstack.iter().enumerate() {
        if i != 0 {
            ga_concat(&mut msg_ga, b", \0".as_ptr() as *const c_char);
        }
        match &entry.data {
            StackData::Dict { dict, hi, .. } => {
                // Key that was just processed = *(hi - 1)
                // In C: (v.data.d.hi == NULL ? v.data.d.dict->dv_hashtab.ht_array : (v.data.d.hi-1))->hi_key
                let key_hi: HashItemHandle = if (*hi).is_null() {
                    dict_get_ht_array(*dict)
                } else {
                    ((*hi) as *const u8).sub(16) as HashItemHandle
                };
                let hi_key = hi_get_key(key_hi);
                // Build a stack-allocated VAR_STRING typval for encode_tv2string
                let mut tv_bytes = [0u8; 16usize];
                // v_type = VAR_STRING = 2
                *(tv_bytes.as_mut_ptr() as *mut c_int) = VAR_STRING;
                // vval.v_string at offset 8
                *(tv_bytes.as_mut_ptr().add(8) as *mut *const c_char) = hi_key;
                let key_tv = tv_bytes.as_ptr() as TypevalHandle;
                let key = rs_encode_tv2string(key_tv, std::ptr::null_mut());
                vim_snprintf(IOBUFF.as_mut_ptr(), IOSIZE, key_msg, key);
                xfree(key as *mut c_void);
                ga_concat(&mut msg_ga, IOBUFF.as_ptr());
            }
            StackData::List { list, li } | StackData::Pairs { list, li } => {
                let is_pairs = matches!(entry.data, StackData::Pairs { .. });
                let first = nvim_list_get_first(*list);
                let idx = if *li == first {
                    0
                } else if (*li).is_null() {
                    nvim_list_get_len(*list) - 1
                } else {
                    let prev = listitem_get_prev(*li);
                    tv_list_idx_of_item(*list, prev)
                };
                let last_li = if (*li).is_null() {
                    nvim_list_get_last(*list)
                } else {
                    listitem_get_prev(*li)
                };
                let use_idx = !is_pairs || last_li.is_null() || {
                    let tv = nvim_listitem_get_tv(last_li);
                    if tv_get_vtype(tv) != VAR_LIST {
                        true
                    } else {
                        let sub = tv_get_vlist(tv);
                        sub.is_null() || nvim_list_get_len(sub) <= 0
                    }
                };
                if use_idx {
                    vim_snprintf(IOBUFF.as_mut_ptr(), IOSIZE, idx_msg, idx);
                    ga_concat(&mut msg_ga, IOBUFF.as_ptr());
                } else {
                    let tv = nvim_listitem_get_tv(last_li);
                    let sub = tv_get_vlist(tv);
                    let first_item = nvim_list_get_first(sub);
                    let key_tv = nvim_listitem_get_tv(first_item);
                    let key = rs_encode_tv2echo(key_tv, std::ptr::null_mut());
                    vim_snprintf(IOBUFF.as_mut_ptr(), IOSIZE, key_pair_msg, key, idx);
                    xfree(key as *mut c_void);
                    ga_concat(&mut msg_ga, IOBUFF.as_ptr());
                }
            }
            StackData::Partial { stage, .. } => match stage {
                PartialStage::Args => {
                    std::process::abort(); // C has abort() here
                }
                PartialStage::Self_ => {
                    ga_concat(&mut msg_ga, partial_arg_msg);
                }
                PartialStage::End => {
                    ga_concat(&mut msg_ga, partial_self_msg);
                }
            },
            StackData::PartialList { arg, argv, .. } => {
                let idx = (*arg as *const u8).offset_from(*argv as *const u8) / 16 - 1;
                vim_snprintf(IOBUFF.as_mut_ptr(), IOSIZE, partial_arg_i_msg, idx as c_int);
                ga_concat(&mut msg_ga, IOBUFF.as_ptr());
            }
        }
    }

    let itself = b"itself\0".as_ptr() as *const c_char;
    let stack_msg = if mpstack.is_empty() {
        itself
    } else {
        msg_ga.ga_data as *const c_char
    };
    semsg(msg, objname, stack_msg);
    ga_clear(&mut msg_ga);
    FAIL
}

// =============================================================================
// convert_one_value
// =============================================================================

unsafe fn convert_one_value<E: Encoder>(
    enc: &mut E,
    mpstack: &mut Vec<StackEntry>,
    tv: TypevalHandle,
    copyid: c_int,
    objname: *const c_char,
) -> c_int {
    enc.check_before();

    match tv_get_vtype(tv) {
        t if t == VAR_STRING => {
            let s = tv_get_vstring(tv);
            let len = tv_strlen_inner(tv);
            if enc.conv_string(tv, s, len) != OK {
                return FAIL;
            }
        }
        t if t == VAR_NUMBER => {
            enc.conv_number(tv, tv_get_vnumber(tv));
        }
        t if t == VAR_FLOAT => {
            if enc.conv_float(tv, tv_get_vfloat(tv)) != OK {
                return FAIL;
            }
        }
        t if t == VAR_BLOB => {
            let blob = tv_get_vblob(tv);
            let len = if blob.is_null() {
                0
            } else {
                nvim_blob_get_len(blob)
            };
            if enc.conv_blob(tv, blob, len) != OK {
                return FAIL;
            }
        }
        t if t == VAR_FUNC => {
            let fun = tv_get_vstring(tv);
            // NOTDONE means "handled, skip before_args/before_self/end"
            // (models C's goto typval_encode_stop_converting_one_item).
            let ret = enc.conv_func_start(tv, fun, mpstack, objname);
            if ret == FAIL {
                return FAIL;
            }
            if ret == OK {
                enc.conv_func_before_args(tv, 0);
                enc.conv_func_before_self(tv, -1);
                enc.conv_func_end(tv);
            }
        }
        t if t == VAR_PARTIAL => {
            let pt = tv_get_vpartial(tv);
            let fun: *const c_char = if pt.is_null() {
                std::ptr::null()
            } else {
                rs_partial_name(pt) as *const c_char
            };
            // NOTDONE means "handled, skip partial stack push"
            let ret = enc.conv_func_start(tv, fun, mpstack, objname);
            if ret == FAIL {
                return FAIL;
            }
            if ret == OK {
                mpstack.push(StackEntry {
                    ty: StackEntryType::Partial,
                    tv,
                    saved_copyid: copyid - 1,
                    data: StackData::Partial {
                        stage: PartialStage::Args,
                        pt,
                    },
                });
            }
            // NOTDONE: encoder handled the function reference, skip partial push.
        }
        t if t == VAR_LIST => {
            let list = tv_get_vlist(tv);
            if list.is_null() || nvim_list_get_len(list) == 0 {
                enc.conv_empty_list(tv);
            } else {
                let saved_copyid = list_get_copyid_raw(list);
                let lv_copyid_ptr = (list as *mut u8).add(68) as *mut c_int;
                if *lv_copyid_ptr == copyid {
                    return enc.conv_recurse(list.cast(), StackEntryType::List, mpstack, objname);
                }
                *lv_copyid_ptr = copyid;
                enc.conv_list_start(tv, nvim_list_get_len(list));
                let li = nvim_list_get_first(list);
                mpstack.push(StackEntry {
                    ty: StackEntryType::List,
                    tv,
                    saved_copyid,
                    data: StackData::List { list, li },
                });
                enc.conv_real_list_after_start(tv);
            }
        }
        t if t == VAR_BOOL => {
            let bval = tv_get_vbool(tv);
            enc.conv_bool(tv, bval == K_BOOL_VAR_TRUE);
        }
        t if t == VAR_SPECIAL => {
            enc.conv_nil(tv);
        }
        t if t == VAR_DICT => {
            let dict = tv_get_vdict(tv);
            if dict.is_null() || dict_get_ht_used(dict) == 0 {
                enc.conv_empty_dict(tv, dict);
            } else {
                // Try special dict if allowed
                if enc.allow_specials() {
                    if dict_get_ht_used(dict) == 2 {
                        let type_di = tv_dict_find(dict, b"_TYPE\0".as_ptr() as *const c_char, 5);
                        if !type_di.is_null() {
                            let type_tv = nvim_dictitem_di_tv(type_di);
                            if tv_get_vtype(type_tv) == VAR_LIST {
                                let type_list = tv_get_vlist(type_tv);
                                let val_di =
                                    tv_dict_find(dict, b"_VAL\0".as_ptr() as *const c_char, 4);
                                if !val_di.is_null() {
                                    let mut mp_type: Option<usize> = None;
                                    for i in 0..KMP_COUNT {
                                        if nvim_eval_msgpack_type_list(i as c_int) == type_list {
                                            mp_type = Some(i);
                                            break;
                                        }
                                    }
                                    if let Some(mp_i) = mp_type {
                                        enc.check_before();
                                        let val_tv = nvim_dictitem_di_tv(val_di);
                                        let ret = handle_special_dict(
                                            enc, mpstack, tv, val_tv, mp_i, copyid, objname,
                                        );
                                        if ret != NOTDONE {
                                            return ret;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Regular dict
                let saved_copyid = dict_get_copyid(dict);
                let dv_copyid_ptr = (dict as *mut u8).add(12) as *mut c_int;
                if *dv_copyid_ptr == copyid {
                    return enc.conv_recurse(dict.cast(), StackEntryType::Dict, mpstack, objname);
                }
                *dv_copyid_ptr = copyid;
                let ht_used = dict_get_ht_used(dict);
                enc.conv_dict_start(tv, dict, ht_used);
                let hi = dict_get_ht_array(dict);
                let dictp = (tv as *mut u8).add(8) as *mut DictHandle;
                mpstack.push(StackEntry {
                    ty: StackEntryType::Dict,
                    tv,
                    saved_copyid,
                    data: StackData::Dict {
                        dict,
                        dictp,
                        hi,
                        todo: ht_used,
                    },
                });
                enc.conv_real_dict_after_start(tv, dict);
            }
        }
        t if t == VAR_UNKNOWN => {
            internal_error(b"encode_vim_to_*(): VAR_UNKNOWN\0".as_ptr() as *const c_char);
            return FAIL;
        }
        _ => {
            internal_error(b"encode_vim_to_*(): unexpected vtype\0".as_ptr() as *const c_char);
            return FAIL;
        }
    }
    OK
}

/// Handle a special dictionary (_TYPE/_VAL pair).
///
/// Returns OK, FAIL, or NOTDONE (fall through to regular dict).
unsafe fn handle_special_dict<E: Encoder>(
    enc: &mut E,
    mpstack: &mut Vec<StackEntry>,
    tv: TypevalHandle,
    val_tv: TypevalHandle,
    mp_type: usize,
    copyid: c_int,
    objname: *const c_char,
) -> c_int {
    match mp_type {
        KMP_NIL => {
            enc.conv_nil(tv);
            OK
        }
        KMP_BOOLEAN => {
            if tv_get_vtype(val_tv) != VAR_NUMBER {
                return NOTDONE;
            }
            enc.conv_bool(tv, tv_get_vnumber(val_tv) != 0);
            OK
        }
        KMP_INTEGER => {
            if tv_get_vtype(val_tv) != VAR_LIST {
                return NOTDONE;
            }
            let val_list = tv_get_vlist(val_tv);
            if nvim_list_get_len(val_list) != 4 {
                return NOTDONE;
            }
            let li0 = nvim_list_get_first(val_list);
            let tv0 = nvim_listitem_get_tv(li0);
            if tv_get_vtype(tv0) != VAR_NUMBER {
                return NOTDONE;
            }
            let sign = tv_get_vnumber(tv0);
            if sign == 0 {
                return NOTDONE;
            }
            let li1 = nvim_listitem_get_next(li0);
            let tv1 = nvim_listitem_get_tv(li1);
            if tv_get_vtype(tv1) != VAR_NUMBER || tv_get_vnumber(tv1) < 0 {
                return NOTDONE;
            }
            let highest_bits = tv_get_vnumber(tv1);
            let li2 = nvim_listitem_get_next(li1);
            let tv2 = nvim_listitem_get_tv(li2);
            if tv_get_vtype(tv2) != VAR_NUMBER || tv_get_vnumber(tv2) < 0 {
                return NOTDONE;
            }
            let high_bits = tv_get_vnumber(tv2);
            let li3 = nvim_list_get_last(val_list);
            let tv3 = nvim_listitem_get_tv(li3);
            if tv_get_vtype(tv3) != VAR_NUMBER || tv_get_vnumber(tv3) < 0 {
                return NOTDONE;
            }
            let low_bits = tv_get_vnumber(tv3);
            let number: u64 =
                ((highest_bits as u64) << 62) | ((high_bits as u64) << 31) | (low_bits as u64);
            if sign > 0 {
                enc.conv_unsigned_number(tv, number);
            } else {
                enc.conv_number(tv, -(number as i64));
            }
            OK
        }
        KMP_FLOAT => {
            if tv_get_vtype(val_tv) != VAR_FLOAT {
                return NOTDONE;
            }
            enc.conv_float(tv, tv_get_vfloat(val_tv))
        }
        KMP_STRING => {
            if tv_get_vtype(val_tv) != VAR_LIST {
                return NOTDONE;
            }
            let mut len: usize = 0;
            let mut buf: *mut c_char = std::ptr::null_mut();
            if !rs_encode_vim_list_to_buf(tv_get_vlist(val_tv), &mut len, &mut buf) {
                return NOTDONE;
            }
            let ret = enc.conv_str_string(tv, buf, len);
            xfree(buf as *mut c_void);
            ret
        }
        KMP_ARRAY => {
            if tv_get_vtype(val_tv) != VAR_LIST {
                return NOTDONE;
            }
            let inner = tv_get_vlist(val_tv);
            let lv_copyid_ptr = (inner as *mut u8).add(68) as *mut c_int;
            let saved = *lv_copyid_ptr;
            if *lv_copyid_ptr == copyid {
                return enc.conv_recurse(inner.cast(), StackEntryType::List, mpstack, objname);
            }
            *lv_copyid_ptr = copyid;
            enc.conv_list_start(tv, nvim_list_get_len(inner));
            let li = nvim_list_get_first(inner);
            mpstack.push(StackEntry {
                ty: StackEntryType::List,
                tv,
                saved_copyid: saved,
                data: StackData::List { list: inner, li },
            });
            OK
        }
        KMP_MAP => {
            if tv_get_vtype(val_tv) != VAR_LIST {
                return NOTDONE;
            }
            let val_list = tv_get_vlist(val_tv);
            if val_list.is_null() || nvim_list_get_len(val_list) == 0 {
                enc.conv_empty_dict(tv, std::ptr::null_mut());
                return OK;
            }
            // Validate all items are 2-element lists
            let mut check = nvim_list_get_first(val_list);
            while !check.is_null() {
                let item_tv = nvim_listitem_get_tv(check);
                if tv_get_vtype(item_tv) != VAR_LIST {
                    return NOTDONE;
                }
                if nvim_list_get_len(tv_get_vlist(item_tv)) != 2 {
                    return NOTDONE;
                }
                check = nvim_listitem_get_next(check);
            }
            let lv_copyid_ptr = (val_list as *mut u8).add(68) as *mut c_int;
            let saved = *lv_copyid_ptr;
            if *lv_copyid_ptr == copyid {
                return enc.conv_recurse(val_list.cast(), StackEntryType::Pairs, mpstack, objname);
            }
            *lv_copyid_ptr = copyid;
            enc.conv_dict_start(
                tv,
                std::ptr::null_mut(),
                nvim_list_get_len(val_list) as usize,
            );
            let li = nvim_list_get_first(val_list);
            mpstack.push(StackEntry {
                ty: StackEntryType::Pairs,
                tv,
                saved_copyid: saved,
                data: StackData::Pairs { list: val_list, li },
            });
            OK
        }
        KMP_EXT => {
            if tv_get_vtype(val_tv) != VAR_LIST {
                return NOTDONE;
            }
            let val_list = tv_get_vlist(val_tv);
            if nvim_list_get_len(val_list) != 2 {
                return NOTDONE;
            }
            let first_item = nvim_list_get_first(val_list);
            let type_tv = nvim_listitem_get_tv(first_item);
            if tv_get_vtype(type_tv) != VAR_NUMBER {
                return NOTDONE;
            }
            let ext_type = tv_get_vnumber(type_tv);
            if ext_type > i8::MAX as i64 || ext_type < i8::MIN as i64 {
                return NOTDONE;
            }
            let last_item = nvim_list_get_last(val_list);
            let data_tv = nvim_listitem_get_tv(last_item);
            if tv_get_vtype(data_tv) != VAR_LIST {
                return NOTDONE;
            }
            let mut len: usize = 0;
            let mut buf: *mut c_char = std::ptr::null_mut();
            if !rs_encode_vim_list_to_buf(tv_get_vlist(data_tv), &mut len, &mut buf) {
                return NOTDONE;
            }
            let ret = enc.conv_ext_string(tv, buf, len, ext_type);
            xfree(buf as *mut c_void);
            ret
        }
        _ => NOTDONE,
    }
}

// =============================================================================
// Main encoder loop
// =============================================================================

unsafe fn encode_vim_to<E: Encoder>(
    enc: &mut E,
    top_tv: TypevalHandle,
    objname: *const c_char,
) -> c_int {
    let copyid = rs_get_copyID();
    let mut mpstack: Vec<StackEntry> = Vec::with_capacity(8);

    if convert_one_value(enc, &mut mpstack, top_tv, copyid, objname) == FAIL {
        return FAIL;
    }

    'outer: loop {
        if mpstack.is_empty() {
            break;
        }

        // Extract all needed state from the top stack entry BEFORE any operation
        // that might need to borrow mpstack again (push, pop, or encoder calls that
        // may recurse). We do a single mutable-borrow block to update the mutable
        // fields, then drop the borrow before calling encoder methods or pushing.
        let last_idx = mpstack.len() - 1;
        let entry_tv = mpstack[last_idx].tv;
        let saved_copyid_top = mpstack[last_idx].saved_copyid;

        // `Action` encodes what we want to do after releasing the borrow.
        enum Action {
            DictEnd {
                dict: DictHandle,
                saved: c_int,
                dp: *mut DictHandle,
            },
            DictItem {
                dict_tv: TypevalHandle,
                dp: *mut DictHandle,
                di: TypevalHandle,
                di_key: *const c_char,
                key_len: usize,
                is_first: bool,
            },
            ListEnd {
                list: ListHandle,
                saved: c_int,
            },
            ListItem {
                list_tv: TypevalHandle,
                is_first: bool,
                item_tv: TypevalHandle,
            },
            PairsEnd {
                list: ListHandle,
                saved: c_int,
            },
            PairsItem {
                list_tv: TypevalHandle,
                is_first: bool,
                kv_pair: ListHandle,
                /// The list item (of the outer `_VAL` list) that this pair came from.
                /// Used to temporarily restore the stack's `li` to the C-equivalent
                /// pre-advance position while processing the key, so that error paths
                /// (conv_recurse → conv_error) produce the correct "index N" message.
                cur_li: ListItemHandle,
                key_tv: TypevalHandle,
                val_tv: TypevalHandle,
            },
            PartialArgs {
                pt: PartialHandle,
                argc: c_int,
            },
            PartialSelf {
                pt: PartialHandle,
                dict: DictHandle,
            },
            PartialEnd,
            PartialListEnd,
            PartialListItem {
                is_first: bool,
                item_tv: TypevalHandle,
            },
        }

        let action = {
            let top = &mut mpstack[last_idx];
            match &mut top.data {
                StackData::Dict {
                    dict,
                    dictp,
                    hi,
                    todo,
                } => {
                    if *todo == 0 {
                        Action::DictEnd {
                            dict: *dict,
                            saved: saved_copyid_top,
                            dp: *dictp,
                        }
                    } else {
                        let dict_h = *dict;
                        let ht_used = dict_get_ht_used(dict_h);
                        let is_first = *todo == ht_used;
                        let removed = nvim_hash_removed_ptr();
                        while hi_is_empty(*hi, removed) {
                            *hi = hi_advance(*hi);
                        }
                        let cur_hi = *hi;
                        *todo -= 1;
                        *hi = hi_advance(cur_hi);
                        let cur_dictp = *dictp;
                        let di = hi_to_dictitem(cur_hi);
                        let di_key = hi_to_key(cur_hi);
                        let key_len = libc_strlen(di_key);
                        Action::DictItem {
                            dict_tv: top.tv,
                            dp: cur_dictp,
                            di,
                            di_key,
                            key_len,
                            is_first,
                        }
                    }
                }
                StackData::List { list, li } => {
                    if (*li).is_null() {
                        Action::ListEnd {
                            list: *list,
                            saved: saved_copyid_top,
                        }
                    } else {
                        let list_h = *list;
                        let first = nvim_list_get_first(list_h);
                        let is_first = *li == first;
                        let item_tv = nvim_listitem_get_tv(*li);
                        *li = nvim_listitem_get_next(*li);
                        Action::ListItem {
                            list_tv: top.tv,
                            is_first,
                            item_tv,
                        }
                    }
                }
                StackData::Pairs { list, li } => {
                    if (*li).is_null() {
                        Action::PairsEnd {
                            list: *list,
                            saved: saved_copyid_top,
                        }
                    } else {
                        let list_h = *list;
                        let first = nvim_list_get_first(list_h);
                        let cur_li = *li;
                        let is_first = cur_li == first;
                        let next_li = nvim_listitem_get_next(cur_li);
                        *li = next_li;
                        let kv_pair_tv = nvim_listitem_get_tv(cur_li);
                        let kv_pair = tv_get_vlist(kv_pair_tv);
                        let key_li = nvim_list_get_first(kv_pair);
                        let key_tv = nvim_listitem_get_tv(key_li);
                        let val_li = nvim_list_get_last(kv_pair);
                        let val_tv = nvim_listitem_get_tv(val_li);
                        Action::PairsItem {
                            list_tv: top.tv,
                            is_first,
                            kv_pair,
                            cur_li,
                            key_tv,
                            val_tv,
                        }
                    }
                }
                StackData::Partial { stage, pt } => {
                    let stage_v = *stage;
                    let pt_h = *pt;
                    match stage_v {
                        PartialStage::Args => {
                            let argc = if pt_h.is_null() {
                                0
                            } else {
                                partial_argc(pt_h)
                            };
                            top.data = StackData::Partial {
                                stage: PartialStage::Self_,
                                pt: pt_h,
                            };
                            Action::PartialArgs { pt: pt_h, argc }
                        }
                        PartialStage::Self_ => {
                            let dict = if pt_h.is_null() {
                                std::ptr::null_mut()
                            } else {
                                partial_dict(pt_h)
                            };
                            top.data = StackData::Partial {
                                stage: PartialStage::End,
                                pt: pt_h,
                            };
                            Action::PartialSelf { pt: pt_h, dict }
                        }
                        PartialStage::End => Action::PartialEnd,
                    }
                }
                StackData::PartialList { arg, argv, todo } => {
                    if *todo == 0 {
                        Action::PartialListEnd
                    } else {
                        let first_arg = *argv;
                        let is_first = *arg == first_arg;
                        let item_tv = *arg;
                        *arg = argv_next(*arg);
                        *todo -= 1;
                        Action::PartialListItem { is_first, item_tv }
                    }
                }
            }
        }; // borrow of mpstack[last_idx] ends here

        // Now execute the action with no outstanding borrows on mpstack.
        let cur_tv: TypevalHandle;
        match action {
            Action::DictEnd { dict, saved, dp } => {
                mpstack.pop();
                dict_set_copyid(dict, saved);
                enc.conv_dict_end(entry_tv, *dp);
                continue 'outer;
            }
            Action::DictItem {
                dict_tv,
                dp,
                di,
                di_key,
                key_len,
                is_first,
            } => {
                if !is_first {
                    enc.conv_dict_between_items(dict_tv, *dp);
                }
                if enc.conv_str_string(std::ptr::null_mut(), di_key, key_len) != OK {
                    return FAIL;
                }
                enc.conv_dict_after_key(dict_tv, *dp);
                cur_tv = di;
            }
            Action::ListEnd { list, saved } => {
                mpstack.pop();
                nvim_list_set_copyid(list, saved);
                enc.conv_list_end(entry_tv);
                continue 'outer;
            }
            Action::ListItem {
                list_tv,
                is_first,
                item_tv,
            } => {
                if !is_first {
                    enc.conv_list_between_items(list_tv);
                }
                cur_tv = item_tv;
            }
            Action::PairsEnd { list, saved } => {
                mpstack.pop();
                nvim_list_set_copyid(list, saved);
                enc.conv_dict_end(entry_tv, std::ptr::null_mut());
                continue 'outer;
            }
            Action::PairsItem {
                list_tv,
                is_first,
                kv_pair: _,
                cur_li,
                key_tv,
                val_tv,
            } => {
                if !is_first {
                    enc.conv_dict_between_items(list_tv, std::ptr::null_mut());
                }
                if !enc.special_dict_key_check(key_tv) {
                    return FAIL;
                }
                // Temporarily restore `li` to the pre-advance state (cur_li)
                // while the key is being processed.  This mirrors C's state
                // machine, which advances cur_mpsv->data.l.li AFTER the entire
                // pair (both key and value).  If conv_recurse fires during key
                // processing, conv_error computes prev(cur_li) — which is NULL
                // for the first pair — and correctly emits "index N" rather than
                // "key K at index N from special map".
                //
                // We record the Pairs entry index here (before any nested pushes)
                // so we can reliably find it again after convert_one_value returns.
                let pairs_stack_idx = mpstack.len() - 1;
                if let StackData::Pairs { li, .. } = &mut mpstack[pairs_stack_idx].data {
                    *li = cur_li;
                }
                if convert_one_value(enc, &mut mpstack, key_tv, copyid, objname) == FAIL {
                    return FAIL;
                }
                // Advance li past cur_li now that the key has been processed.
                // Use the saved index — it is still valid because the Pairs entry
                // is never popped inside convert_one_value (only PairsEnd pops it).
                if let StackData::Pairs { li, .. } = &mut mpstack[pairs_stack_idx].data {
                    *li = nvim_listitem_get_next(cur_li);
                }
                enc.conv_dict_after_key(list_tv, std::ptr::null_mut());
                cur_tv = val_tv;
            }
            Action::PartialArgs { pt, argc } => {
                enc.conv_func_before_args(entry_tv, argc);
                if !pt.is_null() && argc > 0 {
                    let argv = partial_argv(pt);
                    enc.conv_list_start(std::ptr::null_mut(), argc);
                    mpstack.push(StackEntry {
                        ty: StackEntryType::PartialList,
                        tv: std::ptr::null_mut(),
                        saved_copyid: copyid - 1,
                        data: StackData::PartialList {
                            arg: argv,
                            argv,
                            todo: argc as usize,
                        },
                    });
                }
                continue 'outer;
            }
            Action::PartialSelf { pt, dict } => {
                if !dict.is_null() {
                    let ht_used = dict_get_ht_used(dict);
                    enc.conv_func_before_self(entry_tv, ht_used as isize);
                    if ht_used == 0 {
                        enc.conv_empty_dict(std::ptr::null_mut(), dict);
                        continue 'outer;
                    }
                    let dv_copyid_ptr = (dict as *mut u8).add(12) as *mut c_int;
                    let saved = *dv_copyid_ptr;
                    if *dv_copyid_ptr == copyid {
                        let ret =
                            enc.conv_recurse(dict.cast(), StackEntryType::Dict, &mpstack, objname);
                        if ret == FAIL {
                            return FAIL;
                        }
                        continue 'outer;
                    }
                    *dv_copyid_ptr = copyid;
                    enc.conv_dict_start(std::ptr::null_mut(), dict, ht_used);
                    let hi = dict_get_ht_array(dict);
                    let dictp = if !pt.is_null() {
                        (pt as *mut u8).add(40) as *mut DictHandle
                    } else {
                        std::ptr::null_mut()
                    };
                    mpstack.push(StackEntry {
                        ty: StackEntryType::Dict,
                        tv: std::ptr::null_mut(),
                        saved_copyid: saved,
                        data: StackData::Dict {
                            dict,
                            dictp,
                            hi,
                            todo: ht_used,
                        },
                    });
                    enc.conv_real_dict_after_start(std::ptr::null_mut(), dict);
                } else {
                    enc.conv_func_before_self(entry_tv, -1);
                }
                continue 'outer;
            }
            Action::PartialEnd => {
                mpstack.pop();
                enc.conv_func_end(entry_tv);
                continue 'outer;
            }
            Action::PartialListEnd => {
                mpstack.pop();
                enc.conv_list_end(std::ptr::null_mut());
                continue 'outer;
            }
            Action::PartialListItem { is_first, item_tv } => {
                if !is_first {
                    enc.conv_list_between_items(std::ptr::null_mut());
                }
                cur_tv = item_tv;
            }
        }

        if convert_one_value(enc, &mut mpstack, cur_tv, copyid, objname) == FAIL {
            return FAIL;
        }
    }

    OK
}

// =============================================================================
// single-quote escaper for string/echo output
// =============================================================================

unsafe fn string_conv_string(gap: *mut GarrayT, buf: *const c_char, len: usize) {
    if buf.is_null() {
        ga_concat(gap, b"''\0".as_ptr() as *const c_char);
        return;
    }
    let mut quote_count: usize = 0;
    for i in 0..len {
        if *buf.add(i) == b'\'' as i8 {
            quote_count += 1;
        }
    }
    ga_grow(gap, (2 + len + quote_count) as c_int);
    ga_append(gap, b'\'');
    for i in 0..len {
        let c = *buf.add(i) as u8;
        if c == b'\'' {
            ga_append(gap, b'\'');
        }
        ga_append(gap, c);
    }
    ga_append(gap, b'\'');
}

// =============================================================================
// StringEncoder
// =============================================================================

struct StringEncoder {
    gap: *mut GarrayT,
}

impl Encoder for StringEncoder {
    unsafe fn check_before(&mut self) {}

    unsafe fn conv_nil(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b"v:null\0".as_ptr() as *const c_char);
    }

    unsafe fn conv_bool(&mut self, _tv: TypevalHandle, val: bool) {
        let s: &[u8] = if val { b"v:true\0" } else { b"v:false\0" };
        ga_concat(self.gap, s.as_ptr() as *const c_char);
    }

    unsafe fn conv_number(&mut self, _tv: TypevalHandle, num: i64) {
        let mut buf = [0i8; NUMBUFLEN];
        vim_snprintf(
            buf.as_mut_ptr(),
            NUMBUFLEN,
            b"%lld\0".as_ptr() as *const c_char,
            num,
        );
        ga_concat(self.gap, buf.as_ptr());
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: TypevalHandle, _num: u64) {}

    unsafe fn conv_float(&mut self, _tv: TypevalHandle, flt: f64) -> c_int {
        let cls = xfpclassify(flt);
        if cls == FP_NAN {
            ga_concat(self.gap, b"str2float('nan')\0".as_ptr() as *const c_char);
        } else if cls == FP_INFINITE {
            if flt < 0.0 {
                ga_append(self.gap, b'-');
            }
            ga_concat(self.gap, b"str2float('inf')\0".as_ptr() as *const c_char);
        } else {
            let mut buf = [0i8; NUMBUFLEN];
            vim_snprintf(
                buf.as_mut_ptr(),
                NUMBUFLEN,
                b"%g\0".as_ptr() as *const c_char,
                flt,
            );
            ga_concat(self.gap, buf.as_ptr());
        }
        OK
    }

    unsafe fn conv_string(&mut self, _tv: TypevalHandle, buf: *const c_char, len: usize) -> c_int {
        string_conv_string(self.gap, buf, len);
        OK
    }

    unsafe fn conv_str_string(
        &mut self,
        tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
    ) -> c_int {
        self.conv_string(tv, buf, len)
    }

    unsafe fn conv_ext_string(
        &mut self,
        _tv: TypevalHandle,
        _buf: *const c_char,
        _len: usize,
        _ty: i64,
    ) -> c_int {
        OK
    }

    unsafe fn conv_blob(&mut self, _tv: TypevalHandle, blob: BlobHandle, len: c_int) -> c_int {
        if len == 0 {
            ga_concat(self.gap, b"0z\0".as_ptr() as *const c_char);
        } else {
            ga_grow(self.gap, 2 + 2 * len + (len - 1) / 4);
            ga_concat(self.gap, b"0z\0".as_ptr() as *const c_char);
            let mut numbuf = [0i8; NUMBUFLEN];
            for i in 0..len {
                if i > 0 && i.trailing_zeros() >= 2 {
                    ga_append(self.gap, b'.');
                }
                let b = nvim_blob_get_byte(blob, i) as c_int;
                vim_snprintf(
                    numbuf.as_mut_ptr(),
                    NUMBUFLEN,
                    b"%02X\0".as_ptr() as *const c_char,
                    b,
                );
                ga_concat(self.gap, numbuf.as_ptr());
            }
        }
        OK
    }

    unsafe fn conv_func_start(
        &mut self,
        _tv: TypevalHandle,
        fun: *const c_char,
        _mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        if fun.is_null() {
            internal_error(b"string(): NULL function name\0".as_ptr() as *const c_char);
            ga_concat(self.gap, b"function(NULL\0".as_ptr() as *const c_char);
        } else {
            ga_concat(self.gap, b"function(\0".as_ptr() as *const c_char);
            let len = libc_strlen(fun);
            string_conv_string(self.gap, fun, len);
        }
        OK
    }

    unsafe fn conv_func_before_args(&mut self, _tv: TypevalHandle, len: c_int) {
        if len != 0 {
            ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
        }
    }

    unsafe fn conv_func_before_self(&mut self, _tv: TypevalHandle, len: isize) {
        if len != -1 {
            ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
        }
    }

    unsafe fn conv_func_end(&mut self, _tv: TypevalHandle) {
        ga_append(self.gap, b')');
    }

    unsafe fn conv_empty_list(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b"[]\0".as_ptr() as *const c_char);
    }

    unsafe fn conv_list_start(&mut self, _tv: TypevalHandle, _len: c_int) {
        ga_append(self.gap, b'[');
    }

    unsafe fn conv_real_list_after_start(&mut self, _tv: TypevalHandle) {}

    unsafe fn conv_list_between_items(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
    }

    unsafe fn conv_list_end(&mut self, _tv: TypevalHandle) {
        ga_append(self.gap, b']');
    }

    unsafe fn conv_empty_dict(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b"{}\0".as_ptr() as *const c_char);
    }

    unsafe fn conv_dict_start(&mut self, _tv: TypevalHandle, _dict: DictHandle, _len: usize) {
        ga_append(self.gap, b'{');
    }

    unsafe fn conv_real_dict_after_start(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}

    unsafe fn conv_dict_after_key(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b": \0".as_ptr() as *const c_char);
    }

    unsafe fn conv_dict_between_items(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
    }

    unsafe fn conv_dict_end(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_append(self.gap, b'}');
    }

    unsafe fn conv_recurse(
        &mut self,
        val: *const c_void,
        conv_type: StackEntryType,
        mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        if !did_echo_string_emsg {
            did_echo_string_emsg = true;
            emsg(
                b"E724: unable to correctly dump variable with self-referencing container\0"
                    .as_ptr() as *const c_char,
            );
        }
        let mut ebuf = [0i8; NUMBUFLEN + 7];
        let backref = find_backref(val, conv_type, mpstack);
        vim_snprintf(
            ebuf.as_mut_ptr(),
            NUMBUFLEN + 7,
            b"{E724@%zu}\0".as_ptr() as *const c_char,
            backref,
        );
        ga_concat(self.gap, ebuf.as_ptr());
        OK
    }

    unsafe fn special_dict_key_check(&mut self, _: TypevalHandle) -> bool {
        true
    }
    fn allow_specials(&self) -> bool {
        false
    }
}

fn find_backref(val: *const c_void, conv_type: StackEntryType, mpstack: &[StackEntry]) -> usize {
    for (i, entry) in mpstack.iter().enumerate() {
        match conv_type {
            StackEntryType::Dict => {
                if let StackData::Dict { dict, .. } = &entry.data {
                    if *dict as *const c_void == val {
                        return i;
                    }
                }
            }
            StackEntryType::List => {
                if let StackData::List { list, .. } = &entry.data {
                    if *list as *const c_void == val {
                        return i;
                    }
                }
            }
            StackEntryType::Pairs => {
                if let StackData::Pairs { list, .. } = &entry.data {
                    if *list as *const c_void == val {
                        return i;
                    }
                }
            }
            _ => {}
        }
    }
    0
}

// =============================================================================
// EchoEncoder
// =============================================================================

struct EchoEncoder {
    gap: *mut GarrayT,
}

impl Encoder for EchoEncoder {
    unsafe fn check_before(&mut self) {}

    unsafe fn conv_nil(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b"v:null\0".as_ptr() as *const c_char);
    }

    unsafe fn conv_bool(&mut self, _tv: TypevalHandle, val: bool) {
        let s: &[u8] = if val { b"v:true\0" } else { b"v:false\0" };
        ga_concat(self.gap, s.as_ptr() as *const c_char);
    }

    unsafe fn conv_number(&mut self, _tv: TypevalHandle, num: i64) {
        let mut buf = [0i8; NUMBUFLEN];
        vim_snprintf(
            buf.as_mut_ptr(),
            NUMBUFLEN,
            b"%lld\0".as_ptr() as *const c_char,
            num,
        );
        ga_concat(self.gap, buf.as_ptr());
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: TypevalHandle, _num: u64) {}

    unsafe fn conv_float(&mut self, _tv: TypevalHandle, flt: f64) -> c_int {
        let cls = xfpclassify(flt);
        if cls == FP_NAN {
            ga_concat(self.gap, b"str2float('nan')\0".as_ptr() as *const c_char);
        } else if cls == FP_INFINITE {
            if flt < 0.0 {
                ga_append(self.gap, b'-');
            }
            ga_concat(self.gap, b"str2float('inf')\0".as_ptr() as *const c_char);
        } else {
            let mut buf = [0i8; NUMBUFLEN];
            vim_snprintf(
                buf.as_mut_ptr(),
                NUMBUFLEN,
                b"%g\0".as_ptr() as *const c_char,
                flt,
            );
            ga_concat(self.gap, buf.as_ptr());
        }
        OK
    }

    unsafe fn conv_string(&mut self, _tv: TypevalHandle, buf: *const c_char, len: usize) -> c_int {
        string_conv_string(self.gap, buf, len);
        OK
    }

    unsafe fn conv_str_string(
        &mut self,
        tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
    ) -> c_int {
        self.conv_string(tv, buf, len)
    }

    unsafe fn conv_ext_string(
        &mut self,
        _tv: TypevalHandle,
        _buf: *const c_char,
        _len: usize,
        _ty: i64,
    ) -> c_int {
        OK
    }

    unsafe fn conv_blob(&mut self, tv: TypevalHandle, blob: BlobHandle, len: c_int) -> c_int {
        let mut se = StringEncoder { gap: self.gap };
        se.conv_blob(tv, blob, len)
    }

    unsafe fn conv_func_start(
        &mut self,
        _tv: TypevalHandle,
        fun: *const c_char,
        _mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        if fun.is_null() {
            internal_error(b"echo: NULL function name\0".as_ptr() as *const c_char);
            ga_concat(self.gap, b"function(NULL\0".as_ptr() as *const c_char);
        } else {
            ga_concat(self.gap, b"function(\0".as_ptr() as *const c_char);
            let len = libc_strlen(fun);
            string_conv_string(self.gap, fun, len);
        }
        OK
    }

    unsafe fn conv_func_before_args(&mut self, _tv: TypevalHandle, len: c_int) {
        if len != 0 {
            ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
        }
    }

    unsafe fn conv_func_before_self(&mut self, _tv: TypevalHandle, len: isize) {
        if len != -1 {
            ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
        }
    }

    unsafe fn conv_func_end(&mut self, _tv: TypevalHandle) {
        ga_append(self.gap, b')');
    }
    unsafe fn conv_empty_list(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b"[]\0".as_ptr() as *const c_char);
    }
    unsafe fn conv_list_start(&mut self, _tv: TypevalHandle, _len: c_int) {
        ga_append(self.gap, b'[');
    }
    unsafe fn conv_real_list_after_start(&mut self, _tv: TypevalHandle) {}
    unsafe fn conv_list_between_items(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
    }
    unsafe fn conv_list_end(&mut self, _tv: TypevalHandle) {
        ga_append(self.gap, b']');
    }
    unsafe fn conv_empty_dict(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b"{}\0".as_ptr() as *const c_char);
    }
    unsafe fn conv_dict_start(&mut self, _tv: TypevalHandle, _dict: DictHandle, _len: usize) {
        ga_append(self.gap, b'{');
    }
    unsafe fn conv_real_dict_after_start(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}
    unsafe fn conv_dict_after_key(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b": \0".as_ptr() as *const c_char);
    }
    unsafe fn conv_dict_between_items(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
    }
    unsafe fn conv_dict_end(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_append(self.gap, b'}');
    }

    unsafe fn conv_recurse(
        &mut self,
        val: *const c_void,
        conv_type: StackEntryType,
        mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        let mut ebuf = [0i8; NUMBUFLEN + 7];
        let backref = find_backref(val, conv_type, mpstack);
        match conv_type {
            StackEntryType::Dict => {
                vim_snprintf(
                    ebuf.as_mut_ptr(),
                    NUMBUFLEN + 7,
                    b"{...@%zu}\0".as_ptr() as *const c_char,
                    backref,
                );
            }
            _ => {
                vim_snprintf(
                    ebuf.as_mut_ptr(),
                    NUMBUFLEN + 7,
                    b"[...@%zu]\0".as_ptr() as *const c_char,
                    backref,
                );
            }
        }
        ga_concat(self.gap, ebuf.as_ptr());
        OK
    }

    unsafe fn special_dict_key_check(&mut self, _: TypevalHandle) -> bool {
        true
    }
    fn allow_specials(&self) -> bool {
        false
    }
}

// =============================================================================
// JsonEncoder
// =============================================================================

struct JsonEncoder {
    gap: *mut GarrayT,
}

impl Encoder for JsonEncoder {
    unsafe fn check_before(&mut self) {}

    unsafe fn conv_nil(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b"null\0".as_ptr() as *const c_char);
    }

    unsafe fn conv_bool(&mut self, _tv: TypevalHandle, val: bool) {
        let s: &[u8] = if val { b"true\0" } else { b"false\0" };
        ga_concat(self.gap, s.as_ptr() as *const c_char);
    }

    unsafe fn conv_number(&mut self, _tv: TypevalHandle, num: i64) {
        let mut buf = [0i8; NUMBUFLEN];
        vim_snprintf(
            buf.as_mut_ptr(),
            NUMBUFLEN,
            b"%lld\0".as_ptr() as *const c_char,
            num,
        );
        ga_concat(self.gap, buf.as_ptr());
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: TypevalHandle, num: u64) {
        let mut buf = [0i8; NUMBUFLEN];
        vim_snprintf(
            buf.as_mut_ptr(),
            NUMBUFLEN,
            b"%llu\0".as_ptr() as *const c_char,
            num,
        );
        ga_concat(self.gap, buf.as_ptr());
    }

    unsafe fn conv_float(&mut self, _tv: TypevalHandle, flt: f64) -> c_int {
        let cls = xfpclassify(flt);
        if cls == FP_NAN {
            emsg(b"E474: Unable to represent NaN value in JSON\0".as_ptr() as *const c_char);
            return FAIL;
        }
        if cls == FP_INFINITE {
            emsg(b"E474: Unable to represent infinity in JSON\0".as_ptr() as *const c_char);
            return FAIL;
        }
        let mut buf = [0i8; NUMBUFLEN];
        vim_snprintf(
            buf.as_mut_ptr(),
            NUMBUFLEN,
            b"%g\0".as_ptr() as *const c_char,
            flt,
        );
        ga_concat(self.gap, buf.as_ptr());
        OK
    }

    unsafe fn conv_string(&mut self, _tv: TypevalHandle, buf: *const c_char, len: usize) -> c_int {
        rs_convert_to_json_string(self.gap, buf, len)
    }

    unsafe fn conv_str_string(
        &mut self,
        tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
    ) -> c_int {
        self.conv_string(tv, buf, len)
    }

    unsafe fn conv_ext_string(
        &mut self,
        _tv: TypevalHandle,
        _buf: *const c_char,
        _len: usize,
        _ty: i64,
    ) -> c_int {
        // Do NOT free buf here. In the original C, the macro did `xfree(buf);
        // return FAIL;` (exiting the function, so the outer xfree was skipped).
        // In Rust, handle_special_dict is the sole owner and always frees buf.
        emsg(b"E474: Unable to convert EXT string to JSON\0".as_ptr() as *const c_char);
        FAIL
    }

    unsafe fn conv_blob(&mut self, _tv: TypevalHandle, blob: BlobHandle, len: c_int) -> c_int {
        if len == 0 {
            ga_concat(self.gap, b"[]\0".as_ptr() as *const c_char);
        } else {
            ga_append(self.gap, b'[');
            let mut numbuf = [0i8; NUMBUFLEN];
            for i in 0..len {
                if i > 0 {
                    ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
                }
                let b = nvim_blob_get_byte(blob, i) as c_int;
                vim_snprintf(
                    numbuf.as_mut_ptr(),
                    NUMBUFLEN,
                    b"%d\0".as_ptr() as *const c_char,
                    b,
                );
                ga_concat(self.gap, numbuf.as_ptr());
            }
            ga_append(self.gap, b']');
        }
        OK
    }

    unsafe fn conv_func_start(
        &mut self,
        _tv: TypevalHandle,
        _fun: *const c_char,
        mpstack: &[StackEntry],
        objname: *const c_char,
    ) -> c_int {
        conv_error(
            b"E474: Error while dumping %s, %s: attempt to dump function reference\0".as_ptr()
                as *const c_char,
            mpstack,
            objname,
        )
    }

    unsafe fn conv_func_before_args(&mut self, _tv: TypevalHandle, _len: c_int) {}
    unsafe fn conv_func_before_self(&mut self, _tv: TypevalHandle, _len: isize) {}
    unsafe fn conv_func_end(&mut self, _tv: TypevalHandle) {}
    unsafe fn conv_empty_list(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b"[]\0".as_ptr() as *const c_char);
    }
    unsafe fn conv_list_start(&mut self, _tv: TypevalHandle, _len: c_int) {
        ga_append(self.gap, b'[');
    }
    unsafe fn conv_real_list_after_start(&mut self, _tv: TypevalHandle) {}
    unsafe fn conv_list_between_items(&mut self, _tv: TypevalHandle) {
        ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
    }
    unsafe fn conv_list_end(&mut self, _tv: TypevalHandle) {
        ga_append(self.gap, b']');
    }
    unsafe fn conv_empty_dict(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b"{}\0".as_ptr() as *const c_char);
    }
    unsafe fn conv_dict_start(&mut self, _tv: TypevalHandle, _dict: DictHandle, _len: usize) {
        ga_append(self.gap, b'{');
    }
    unsafe fn conv_real_dict_after_start(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}
    unsafe fn conv_dict_after_key(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b": \0".as_ptr() as *const c_char);
    }
    unsafe fn conv_dict_between_items(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_concat(self.gap, b", \0".as_ptr() as *const c_char);
    }
    unsafe fn conv_dict_end(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        ga_append(self.gap, b'}');
    }

    unsafe fn conv_recurse(
        &mut self,
        _val: *const c_void,
        _conv_type: StackEntryType,
        _mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        if !did_echo_string_emsg {
            did_echo_string_emsg = true;
            emsg(
                b"E724: unable to correctly dump variable with self-referencing container\0"
                    .as_ptr() as *const c_char,
            );
        }
        OK
    }

    unsafe fn special_dict_key_check(&mut self, key_tv: TypevalHandle) -> bool {
        let ok = rs_encode_check_json_key(key_tv);
        if !ok {
            emsg(b"E474: Invalid key in special dictionary\0".as_ptr() as *const c_char);
        }
        ok
    }

    fn allow_specials(&self) -> bool {
        true
    }
}

// =============================================================================
// encode_check_json_key
// =============================================================================

/// Check whether a key can be used in json_encode().
///
/// Exported as `encode_check_json_key`.
#[unsafe(export_name = "encode_check_json_key")]
pub unsafe extern "C" fn rs_encode_check_json_key(tv: TypevalHandle) -> bool {
    let vtype = tv_get_vtype(tv);
    if vtype == VAR_STRING {
        return true;
    }
    if vtype != VAR_DICT {
        return false;
    }
    let spdict = tv_get_vdict(tv);
    if nvim_dict_get_ht_used(spdict) != 2 {
        return false;
    }
    let type_di = tv_dict_find(spdict, b"_TYPE\0".as_ptr() as *const c_char, 5);
    if type_di.is_null() {
        return false;
    }
    let type_tv = nvim_dictitem_di_tv(type_di);
    if tv_get_vtype(type_tv) != VAR_LIST {
        return false;
    }
    if tv_get_vlist(type_tv) != nvim_eval_msgpack_type_list(KMP_STRING as c_int) {
        return false;
    }
    let val_di = tv_dict_find(spdict, b"_VAL\0".as_ptr() as *const c_char, 4);
    if val_di.is_null() {
        return false;
    }
    let val_tv = nvim_dictitem_di_tv(val_di);
    if tv_get_vtype(val_tv) != VAR_LIST {
        return false;
    }
    let val_list = tv_get_vlist(val_tv);
    if val_list.is_null() {
        return true;
    }
    let mut li = nvim_list_get_first(val_list);
    while !li.is_null() {
        if tv_get_vtype(nvim_listitem_get_tv(li)) != VAR_STRING {
            return false;
        }
        li = nvim_listitem_get_next(li);
    }
    true
}

// =============================================================================
// MsgpackEncoder
// =============================================================================

struct MsgpackEncoder {
    packer: *mut PackerBuffer,
}

impl Encoder for MsgpackEncoder {
    unsafe fn check_before(&mut self) {
        mpack_check_buffer(self.packer);
    }

    unsafe fn conv_nil(&mut self, _tv: TypevalHandle) {
        let mut ptr = nvim_packer_get_ptr(self.packer);
        rs_mpack_nil(&mut ptr);
        nvim_packer_set_ptr(self.packer, ptr);
    }

    unsafe fn conv_bool(&mut self, _tv: TypevalHandle, val: bool) {
        let mut ptr = nvim_packer_get_ptr(self.packer);
        rs_mpack_bool(&mut ptr, val as c_int);
        nvim_packer_set_ptr(self.packer, ptr);
    }

    unsafe fn conv_number(&mut self, _tv: TypevalHandle, num: i64) {
        let mut ptr = (*self.packer).ptr;
        mpack_integer(&mut ptr, num);
        (*self.packer).ptr = ptr;
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: TypevalHandle, num: u64) {
        let mut ptr = (*self.packer).ptr;
        mpack_uint64(&mut ptr, num);
        (*self.packer).ptr = ptr;
    }

    unsafe fn conv_float(&mut self, _tv: TypevalHandle, flt: f64) -> c_int {
        let mut ptr = (*self.packer).ptr;
        mpack_float8(&mut ptr, flt);
        (*self.packer).ptr = ptr;
        OK
    }

    unsafe fn conv_string(&mut self, _tv: TypevalHandle, buf: *const c_char, len: usize) -> c_int {
        rs_mpack_bin(buf as *const u8, len, self.packer);
        OK
    }

    unsafe fn conv_str_string(
        &mut self,
        _tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
    ) -> c_int {
        rs_mpack_str(buf as *const u8, len, self.packer);
        OK
    }

    unsafe fn conv_ext_string(
        &mut self,
        _tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
        ty: i64,
    ) -> c_int {
        rs_mpack_ext(buf, len, ty as i8, self.packer);
        OK
    }

    unsafe fn conv_blob(&mut self, _tv: TypevalHandle, blob: BlobHandle, len: c_int) -> c_int {
        let data: *const u8 = if blob.is_null() {
            std::ptr::null()
        } else {
            nvim_blob_get_ga_data(blob) as *const u8
        };
        rs_mpack_bin(data, len as usize, self.packer);
        OK
    }

    unsafe fn conv_func_start(
        &mut self,
        _tv: TypevalHandle,
        _fun: *const c_char,
        mpstack: &[StackEntry],
        objname: *const c_char,
    ) -> c_int {
        conv_error(
            b"E5004: Error while dumping %s, %s: attempt to dump function reference\0".as_ptr()
                as *const c_char,
            mpstack,
            objname,
        )
    }

    unsafe fn conv_func_before_args(&mut self, _tv: TypevalHandle, _len: c_int) {}
    unsafe fn conv_func_before_self(&mut self, _tv: TypevalHandle, _len: isize) {}
    unsafe fn conv_func_end(&mut self, _tv: TypevalHandle) {}

    unsafe fn conv_empty_list(&mut self, _tv: TypevalHandle) {
        let mut ptr = nvim_packer_get_ptr(self.packer);
        rs_mpack_array(&mut ptr, 0);
        nvim_packer_set_ptr(self.packer, ptr);
    }

    unsafe fn conv_list_start(&mut self, _tv: TypevalHandle, len: c_int) {
        let mut ptr = nvim_packer_get_ptr(self.packer);
        rs_mpack_array(&mut ptr, len as u32);
        nvim_packer_set_ptr(self.packer, ptr);
    }

    unsafe fn conv_real_list_after_start(&mut self, _tv: TypevalHandle) {}
    unsafe fn conv_list_between_items(&mut self, _tv: TypevalHandle) {}
    unsafe fn conv_list_end(&mut self, _tv: TypevalHandle) {}

    unsafe fn conv_empty_dict(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        let mut ptr = nvim_packer_get_ptr(self.packer);
        rs_mpack_map(&mut ptr, 0);
        nvim_packer_set_ptr(self.packer, ptr);
    }

    unsafe fn conv_dict_start(&mut self, _tv: TypevalHandle, _dict: DictHandle, len: usize) {
        let mut ptr = nvim_packer_get_ptr(self.packer);
        rs_mpack_map(&mut ptr, len as u32);
        nvim_packer_set_ptr(self.packer, ptr);
    }

    unsafe fn conv_real_dict_after_start(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}
    unsafe fn conv_dict_after_key(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}
    unsafe fn conv_dict_between_items(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}
    unsafe fn conv_dict_end(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}

    unsafe fn conv_recurse(
        &mut self,
        _val: *const c_void,
        _conv_type: StackEntryType,
        mpstack: &[StackEntry],
        objname: *const c_char,
    ) -> c_int {
        conv_error(
            b"E5005: Unable to dump %s: container references itself in %s\0".as_ptr()
                as *const c_char,
            mpstack,
            objname,
        )
    }

    unsafe fn special_dict_key_check(&mut self, _: TypevalHandle) -> bool {
        true
    }
    fn allow_specials(&self) -> bool {
        true
    }
}

// =============================================================================
// Phase 1: Utility helpers
// =============================================================================

/// Msgpack write-to-list callback.
///
/// Exported as `encode_list_write`.
#[unsafe(export_name = "encode_list_write")]
pub unsafe extern "C" fn rs_encode_list_write(data: *mut c_void, buf: *const c_char, len: usize) {
    if len == 0 {
        return;
    }
    let list = data as ListHandle;
    let end = buf.add(len);
    let mut line_end = buf;
    let li = nvim_list_get_last(list);

    if !li.is_null() {
        let scan = xmemscan(buf.cast(), b'\n' as c_char, len).cast::<c_char>();
        line_end = scan;
        if line_end != buf {
            let line_length = line_end.offset_from(buf) as usize;
            let tv = nvim_listitem_get_tv(li);
            let str_ptr = (tv as *mut u8).add(8) as *mut *mut c_char;
            let old_str = *str_ptr;
            let li_len = if old_str.is_null() {
                0usize
            } else {
                libc_strlen(old_str)
            };
            let new_str = xmalloc(li_len + line_length + 1).cast::<c_char>();
            if !old_str.is_null() {
                libc_memcpy(new_str.cast(), old_str.cast(), li_len);
            }
            libc_memcpy(new_str.add(li_len).cast(), buf.cast(), line_length);
            *new_str.add(li_len + line_length) = 0;
            memchrsub(new_str.add(li_len).cast(), 0, b'\n' as c_char, line_length);
            xfree(old_str.cast());
            *str_ptr = new_str;
        }
        line_end = line_end.add(1);
    }

    while line_end < end {
        let line_start = line_end;
        let remaining = end.offset_from(line_start) as usize;
        let scan = xmemscan(line_start.cast(), b'\n' as c_char, remaining).cast::<c_char>();
        line_end = scan;
        let s: *mut c_char = if line_end != line_start {
            let line_length = line_end.offset_from(line_start) as usize;
            let dup = xmemdupz(line_start.cast(), line_length);
            memchrsub(dup.cast(), 0, b'\n' as c_char, line_length);
            dup
        } else {
            std::ptr::null_mut()
        };
        tv_list_append_allocated_string(list, s);
        line_end = line_end.add(1);
    }
    if line_end == end {
        tv_list_append_allocated_string(list, std::ptr::null_mut());
    }
}

/// Initialize a `ListReaderState`.
///
/// Exported as `encode_init_lrstate`.
#[unsafe(export_name = "encode_init_lrstate")]
pub unsafe extern "C" fn rs_encode_init_lrstate(list: ListHandle) -> ListReaderState {
    let li = nvim_list_get_first(list);
    let li_length = if li.is_null() {
        0
    } else {
        let tv = nvim_listitem_get_tv(li);
        let v_str = tv_get_vstring(tv);
        if v_str.is_null() {
            0
        } else {
            libc_strlen(v_str)
        }
    };
    ListReaderState {
        list,
        li,
        offset: 0,
        li_length,
    }
}

/// Read bytes from a readfile()-style list.
///
/// Exported as `encode_read_from_list`.
#[unsafe(export_name = "encode_read_from_list")]
pub unsafe extern "C" fn rs_encode_read_from_list(
    state: *mut ListReaderState,
    buf: *mut c_char,
    nbuf: usize,
    read_bytes: *mut usize,
) -> c_int {
    let buf_end = buf.add(nbuf);
    let mut p = buf;
    let s = &mut *state;

    while p < buf_end {
        while s.offset < s.li_length && p < buf_end {
            let tv = nvim_listitem_get_tv(s.li);
            let v_str = tv_get_vstring(tv);
            let ch = *v_str.add(s.offset) as u8;
            *p = if ch == b'\n' { 0i8 } else { ch as i8 };
            p = p.add(1);
            s.offset += 1;
        }
        if p < buf_end {
            s.li = nvim_listitem_get_next(s.li);
            if s.li.is_null() {
                *read_bytes = p.offset_from(buf) as usize;
                return OK;
            }
            *p = b'\n' as i8;
            p = p.add(1);
            let tv = nvim_listitem_get_tv(s.li);
            if tv_get_vtype(tv) != VAR_STRING {
                *read_bytes = p.offset_from(buf) as usize;
                return FAIL;
            }
            s.offset = 0;
            let v_str = tv_get_vstring(tv);
            s.li_length = if v_str.is_null() {
                0
            } else {
                libc_strlen(v_str)
            };
        }
    }
    *read_bytes = nbuf;
    let has_more = s.offset < s.li_length || !nvim_listitem_get_next(s.li).is_null();
    if has_more {
        NOTDONE
    } else {
        OK
    }
}

/// Convert a readfile()-style list to an owned byte buffer.
///
/// Exported as `encode_vim_list_to_buf`.
#[unsafe(export_name = "encode_vim_list_to_buf")]
pub unsafe extern "C" fn rs_encode_vim_list_to_buf(
    list: ListHandle,
    ret_len: *mut usize,
    ret_buf: *mut *mut c_char,
) -> bool {
    let mut len: usize = 0;
    // Mirror C's TV_LIST_ITER_CONST: skip the loop entirely for a NULL list.
    if list.is_null() {
        *ret_len = 0;
        *ret_buf = std::ptr::null_mut();
        return true;
    }
    let mut li = nvim_list_get_first(list);
    while !li.is_null() {
        let tv = nvim_listitem_get_tv(li);
        if tv_get_vtype(tv) != VAR_STRING {
            return false;
        }
        len += 1;
        let v_str = tv_get_vstring(tv);
        if !v_str.is_null() {
            len += libc_strlen(v_str);
        }
        li = nvim_listitem_get_next(li);
    }
    if len > 0 {
        len -= 1;
    }
    *ret_len = len;
    if len == 0 {
        *ret_buf = std::ptr::null_mut();
        return true;
    }
    let mut lrstate = rs_encode_init_lrstate(list);
    let buf = xmalloc(len).cast::<c_char>();
    let mut read_bytes: usize = 0;
    let ret = rs_encode_read_from_list(&mut lrstate, buf, len, &mut read_bytes);
    if ret == FAIL {
        std::process::abort();
    }
    debug_assert_eq!(len, read_bytes);
    *ret_buf = buf;
    true
}

// =============================================================================
// LuaEncoder: encode typval_T to Lua stack values
// =============================================================================

/// Opaque Lua interpreter state (same layout as lua::state::LuaState).
#[repr(C)]
pub struct LuaState {
    _private: [u8; 0],
}

// FC_LUAREF flag: ufunc has a Lua reference instead of a VimL body.
const FC_LUAREF: c_int = 0x800;
// kNluaPushSpecial flag: use typed tables for special values.
const K_NLUA_PUSH_SPECIAL: c_int = 0x01;

extern "C" {
    // Lua C API (used only by LuaEncoder)
    fn lua_pushnil(lstate: *mut LuaState);
    fn lua_pushboolean(lstate: *mut LuaState, b: c_int);
    fn lua_pushnumber(lstate: *mut LuaState, n: f64);
    fn lua_pushlstring(lstate: *mut LuaState, s: *const c_char, len: usize);
    fn lua_createtable(lstate: *mut LuaState, narr: c_int, nrec: c_int);
    fn lua_rawset(lstate: *mut LuaState, idx: c_int);
    fn lua_rawseti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64;
    fn lua_setmetatable(lstate: *mut LuaState, objindex: c_int) -> c_int;
    fn lua_checkstack(lstate: *mut LuaState, extra: c_int) -> c_int;
    fn lua_gettop(lstate: *mut LuaState) -> c_int;
    fn lua_pushvalue(lstate: *mut LuaState, idx: c_int);

    // Neovim nlua (Rust symbols from nvim-lua crate)
    fn nlua_pushref(lstate: *mut LuaState, ref_: c_int);
    fn rs_nlua_get_nil_ref(lstate: *mut LuaState) -> c_int;
    fn rs_nlua_get_empty_dict_ref(lstate: *mut LuaState) -> c_int;

    // User function lookup
    fn find_func(name: *const c_char) -> *mut c_void; // ufunc_T*
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_luaref(fp: *const c_void) -> c_int;
}

/// Encoder that converts typval_T values to Lua stack values.
///
/// Mirrors the C macro-generated `encode_vim_to_lua` function from converter.c.
struct LuaEncoder {
    lstate: *mut LuaState,
    /// True when kNluaPushSpecial is set (use typed tables for special values).
    allow_special: bool,
}

impl LuaEncoder {
    /// Push the type-index key (true) onto the stack.
    #[inline]
    unsafe fn push_type_idx(&self) {
        lua_pushboolean(self.lstate, 1);
    }

    /// Push the type number onto the stack.
    #[inline]
    unsafe fn push_type_num(&self, ty: c_int) {
        lua_pushnumber(self.lstate, f64::from(ty));
    }

    /// Create a typed table: {[type_idx] = ty}.
    #[inline]
    unsafe fn create_typed_table(&self, narr: c_int, nrec: c_int, ty: c_int) {
        lua_createtable(self.lstate, narr, 1 + nrec);
        self.push_type_idx();
        self.push_type_num(ty);
        lua_rawset(self.lstate, -3);
    }
}

// kObjectType constants mirrored from api/private/defs.h
const K_OBJECT_TYPE_FLOAT_LUA: c_int = 3;
const K_OBJECT_TYPE_ARRAY_LUA: c_int = 5;
const K_OBJECT_TYPE_DICT_LUA: c_int = 6;

impl Encoder for LuaEncoder {
    unsafe fn check_before(&mut self) {}

    unsafe fn conv_nil(&mut self, _tv: TypevalHandle) {
        if self.allow_special {
            lua_pushnil(self.lstate);
        } else {
            let nil_ref = rs_nlua_get_nil_ref(self.lstate);
            nlua_pushref(self.lstate, nil_ref);
        }
    }

    unsafe fn conv_bool(&mut self, _tv: TypevalHandle, val: bool) {
        lua_pushboolean(self.lstate, c_int::from(val));
    }

    unsafe fn conv_number(&mut self, _tv: TypevalHandle, num: i64) {
        #[allow(clippy::cast_precision_loss)]
        lua_pushnumber(self.lstate, num as f64);
    }

    unsafe fn conv_unsigned_number(&mut self, tv: TypevalHandle, num: u64) {
        #[allow(clippy::cast_precision_loss)]
        self.conv_number(tv, num as i64);
    }

    unsafe fn conv_float(&mut self, _tv: TypevalHandle, flt: f64) -> c_int {
        lua_pushnumber(self.lstate, flt);
        OK
    }

    unsafe fn conv_string(&mut self, _tv: TypevalHandle, buf: *const c_char, len: usize) -> c_int {
        lua_pushlstring(self.lstate, buf, len);
        OK
    }

    unsafe fn conv_str_string(
        &mut self,
        tv: TypevalHandle,
        buf: *const c_char,
        len: usize,
    ) -> c_int {
        self.conv_string(tv, buf, len)
    }

    unsafe fn conv_ext_string(
        &mut self,
        tv: TypevalHandle,
        _buf: *const c_char,
        _len: usize,
        _ty: i64,
    ) -> c_int {
        self.conv_nil(tv);
        OK
    }

    unsafe fn conv_blob(&mut self, _tv: TypevalHandle, blob: BlobHandle, len: c_int) -> c_int {
        let data: *const c_char = if blob.is_null() || len == 0 {
            b"\0".as_ptr() as *const c_char
        } else {
            nvim_blob_get_ga_data(blob) as *const c_char
        };
        lua_pushlstring(self.lstate, data, len as usize);
        OK
    }

    /// Push a function reference onto the Lua stack.
    ///
    /// Returns NOTDONE if the function was handled (modelling C's
    /// `goto typval_encode_stop_converting_one_item`), or OK/FAIL otherwise.
    unsafe fn conv_func_start(
        &mut self,
        tv: TypevalHandle,
        fun: *const c_char,
        _mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        let fp = if fun.is_null() {
            std::ptr::null_mut()
        } else {
            find_func(fun)
        };
        if !fp.is_null() && (nvim_ufunc_get_flags(fp) & FC_LUAREF) != 0 {
            let luaref = nvim_ufunc_get_luaref(fp);
            nlua_pushref(self.lstate, luaref);
        } else {
            self.conv_nil(tv);
        }
        // NOTDONE: we've already pushed the value; tell the template to skip
        // the remaining conv_func_before_args/before_self/end calls.
        NOTDONE
    }

    unsafe fn conv_func_before_args(&mut self, _tv: TypevalHandle, _len: c_int) {}
    unsafe fn conv_func_before_self(&mut self, _tv: TypevalHandle, _len: isize) {}
    unsafe fn conv_func_end(&mut self, _tv: TypevalHandle) {}

    unsafe fn conv_empty_list(&mut self, _tv: TypevalHandle) {
        lua_createtable(self.lstate, 0, 0);
    }

    unsafe fn conv_list_start(&mut self, _tv: TypevalHandle, len: c_int) {
        if lua_checkstack(self.lstate, lua_gettop(self.lstate) + 3) == 0 {
            semsg(
                b"E5102: Lua failed to grow stack to %i\0".as_ptr() as *const c_char,
                lua_gettop(self.lstate) + 3,
            );
        }
        lua_createtable(self.lstate, len, 0);
        lua_pushnumber(self.lstate, 1.0); // current list index (pushed as sentinel)
    }

    unsafe fn conv_real_list_after_start(&mut self, _tv: TypevalHandle) {}

    unsafe fn conv_list_between_items(&mut self, _tv: TypevalHandle) {
        // Stack: [..., table, idx, value]
        // Read idx from -2, rawset table[-3][idx] = value, then push idx+1.
        let idx = lua_tonumber(self.lstate, -2);
        lua_rawset(self.lstate, -3); // table[idx] = value; pops key and value
        lua_pushnumber(self.lstate, idx + 1.0);
    }

    unsafe fn conv_list_end(&mut self, _tv: TypevalHandle) {
        // Stack: [..., table, idx, last_value]
        // rawset(-3): table[idx] = last_value; pops both idx and last_value.
        // Stack after: [..., table]
        lua_rawset(self.lstate, -3);
    }

    unsafe fn conv_empty_dict(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        if self.allow_special {
            self.create_typed_table(0, 0, K_OBJECT_TYPE_DICT_LUA);
        } else {
            lua_createtable(self.lstate, 0, 0);
            let empty_ref = rs_nlua_get_empty_dict_ref(self.lstate);
            nlua_pushref(self.lstate, empty_ref);
            lua_setmetatable(self.lstate, -2);
        }
    }

    unsafe fn conv_dict_start(&mut self, _tv: TypevalHandle, _dict: DictHandle, len: usize) {
        if lua_checkstack(self.lstate, lua_gettop(self.lstate) + 3) == 0 {
            semsg(
                b"E5102: Lua failed to grow stack to %i\0".as_ptr() as *const c_char,
                lua_gettop(self.lstate) + 3,
            );
        }
        #[allow(clippy::cast_possible_truncation)]
        lua_createtable(self.lstate, 0, len as c_int);
    }

    unsafe fn conv_real_dict_after_start(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}

    unsafe fn conv_dict_after_key(&mut self, _tv: TypevalHandle, _dict: DictHandle) {}

    unsafe fn conv_dict_between_items(&mut self, _tv: TypevalHandle, _dict: DictHandle) {
        lua_rawset(self.lstate, -3);
    }

    unsafe fn conv_dict_end(&mut self, tv: TypevalHandle, dict: DictHandle) {
        self.conv_dict_between_items(tv, dict);
    }

    unsafe fn conv_recurse(
        &mut self,
        val: *const c_void,
        conv_type: StackEntryType,
        mpstack: &[StackEntry],
        _objname: *const c_char,
    ) -> c_int {
        // Find the backref index (0-based from start of mpstack).
        let backref_i = find_backref(val, conv_type, mpstack);
        // C formula: lua_pushvalue(lstate, -((kv_size(*mpstack) - backref + 1) * 2))
        // where backref is 1-based from end, i.e. backref = mpstack.len() - backref_i.
        // Simplified: offset = -((backref_i + 1) * 2)
        // Each mpstack entry adds 2 Lua stack slots (container + key/index counter).
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let offset = -(((backref_i + 1) * 2) as c_int);
        lua_pushvalue(self.lstate, offset);
        OK
    }

    unsafe fn special_dict_key_check(&mut self, _key_tv: TypevalHandle) -> bool {
        false
    }

    fn allow_specials(&self) -> bool {
        self.allow_special
    }
}

// =============================================================================
// Public exported entry points
// =============================================================================

/// Return a string representation for `string()` builtin.
///
/// Exported as `encode_tv2string`.
#[unsafe(export_name = "encode_tv2string")]
pub unsafe extern "C" fn rs_encode_tv2string(tv: TypevalHandle, len: *mut usize) -> *mut c_char {
    let mut ga = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init(&mut ga, 1, 80);
    let mut enc = StringEncoder { gap: &mut ga };
    let _ = encode_vim_to(
        &mut enc,
        tv,
        b"encode_tv2string() argument\0".as_ptr() as *const c_char,
    );
    did_echo_string_emsg = false;
    if !len.is_null() {
        *len = ga.ga_len as usize;
    }
    ga_append(&mut ga, 0u8);
    ga.ga_data as *mut c_char
}

/// Return a string representation for `:echo`.
///
/// Exported as `encode_tv2echo`.
#[unsafe(export_name = "encode_tv2echo")]
pub unsafe extern "C" fn rs_encode_tv2echo(tv: TypevalHandle, len: *mut usize) -> *mut c_char {
    let mut ga = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init(&mut ga, 1, 80);
    let vtype = tv_get_vtype(tv);
    if vtype == VAR_STRING || vtype == VAR_FUNC {
        let s = tv_get_vstring(tv);
        if !s.is_null() {
            ga_concat(&mut ga, s);
        }
    } else {
        let mut enc = EchoEncoder { gap: &mut ga };
        let _ = encode_vim_to(&mut enc, tv, b":echo argument\0".as_ptr() as *const c_char);
    }
    if !len.is_null() {
        *len = ga.ga_len as usize;
    }
    ga_append(&mut ga, 0u8);
    ga.ga_data as *mut c_char
}

/// Return a JSON string representation.
///
/// Exported as `encode_tv2json`.
#[unsafe(export_name = "encode_tv2json")]
pub unsafe extern "C" fn rs_encode_tv2json(tv: TypevalHandle, len: *mut usize) -> *mut c_char {
    let mut ga = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init(&mut ga, 1, 80);
    let mut enc = JsonEncoder { gap: &mut ga };
    let evj_ret = encode_vim_to(
        &mut enc,
        tv,
        b"encode_tv2json() argument\0".as_ptr() as *const c_char,
    );
    if evj_ret == FAIL {
        ga_clear(&mut ga);
    }
    did_echo_string_emsg = false;
    if !len.is_null() {
        *len = ga.ga_len as usize;
    }
    ga_append(&mut ga, 0u8);
    ga.ga_data as *mut c_char
}

/// Encode a typval to garray using string form.
///
/// Exported as `encode_vim_to_string`.
#[unsafe(export_name = "encode_vim_to_string")]
pub unsafe extern "C" fn rs_encode_vim_to_string(
    gap: *mut GarrayT,
    tv: TypevalHandle,
    objname: *const c_char,
) -> c_int {
    let mut enc = StringEncoder { gap };
    encode_vim_to(&mut enc, tv, objname)
}

/// Encode a typval to garray using echo form.
///
/// Exported as `encode_vim_to_echo`.
#[unsafe(export_name = "encode_vim_to_echo")]
pub unsafe extern "C" fn rs_encode_vim_to_echo(
    gap: *mut GarrayT,
    tv: TypevalHandle,
    objname: *const c_char,
) -> c_int {
    let mut enc = EchoEncoder { gap };
    encode_vim_to(&mut enc, tv, objname)
}

/// Encode a typval to garray using JSON form.
///
/// Exported as `encode_vim_to_json`.
#[unsafe(export_name = "encode_vim_to_json")]
pub unsafe extern "C" fn rs_encode_vim_to_json(
    gap: *mut GarrayT,
    tv: TypevalHandle,
    objname: *const c_char,
) -> c_int {
    let mut enc = JsonEncoder { gap };
    encode_vim_to(&mut enc, tv, objname)
}

/// Encode a typval to a msgpack packer buffer.
///
/// Exported as `encode_vim_to_msgpack`.
#[unsafe(export_name = "encode_vim_to_msgpack")]
pub unsafe extern "C" fn rs_encode_vim_to_msgpack(
    packer: *mut PackerBuffer,
    tv: TypevalHandle,
    objname: *const c_char,
) -> c_int {
    let mut enc = MsgpackEncoder { packer };
    encode_vim_to(&mut enc, tv, objname)
}

/// Encode a typval to the Lua stack.
///
/// Pushes exactly one value onto the Lua stack. `allow_special` is true when
/// `kNluaPushSpecial` is set in the caller's flags (types become typed tables).
///
/// Exported as `encode_vim_to_lua`.
#[unsafe(export_name = "encode_vim_to_lua")]
pub unsafe extern "C" fn rs_encode_vim_to_lua(
    lstate: *mut LuaState,
    tv: TypevalHandle,
    objname: *const c_char,
    allow_special: bool,
) -> c_int {
    let mut enc = LuaEncoder {
        lstate,
        allow_special,
    };
    encode_vim_to(&mut enc, tv, objname)
}
