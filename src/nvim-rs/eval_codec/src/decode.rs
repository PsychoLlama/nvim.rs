//! Decode JSON and MessagePack into Vimscript `typval_T` values.
//!
//! Port of `src/nvim/eval/decode.c`.

#![allow(unsafe_code)]
#![allow(unused_assignments)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_assert)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::manual_while_let_some)]
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

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// Opaque handle aliases
// =============================================================================
type TypevalHandle = *mut c_void;
type ListHandle = *mut c_void;
type DictHandle = *mut c_void;
type DictItemHandle = *mut c_void;
type BlobHandle = *mut c_void;

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;

const VAR_UNLOCKED: c_int = 0;

const K_BOOL_VAR_TRUE: c_int = 1;
const K_BOOL_VAR_FALSE: c_int = 0;
const K_SPECIAL_VAR_NULL: c_int = 0;

const K_MP_INTEGER: c_int = 2;
const K_MP_MAP: c_int = 6;
const K_MP_EXT: c_int = 7;

const VARNUMBER_MAX: u64 = i64::MAX as u64;
const K_LIST_LEN_MAY_KNOW: isize = -3;

const SURROGATE_HI_START: u32 = 0xD800;
const SURROGATE_HI_END: u32 = 0xDBFF;
const SURROGATE_LO_START: u32 = 0xDC00;
const SURROGATE_LO_END: u32 = 0xDFFF;
const SURROGATE_FIRST_CHAR: u32 = 0x10000;

const STR2NR_HEX: c_int = 0x04; // charset.h: 1 << 2
const STR2NR_FORCE: c_int = 0x80; // charset.h: 1 << 7

// mpack token types
const MPACK_TOKEN_NIL: c_int = 1;
const MPACK_TOKEN_BOOLEAN: c_int = 2;
const MPACK_TOKEN_UINT: c_int = 3;
const MPACK_TOKEN_SINT: c_int = 4;
const MPACK_TOKEN_FLOAT: c_int = 5;
const MPACK_TOKEN_CHUNK: c_int = 6;
const MPACK_TOKEN_ARRAY: c_int = 7;
const MPACK_TOKEN_MAP: c_int = 8;
const MPACK_TOKEN_BIN: c_int = 9;
const MPACK_TOKEN_STR: c_int = 10;
const MPACK_TOKEN_EXT: c_int = 11;

const MPACK_OK: c_int = 0;

// sizeof(typval_T): v_type(4) + v_lock(4) + vval(8) = 16 on 64-bit
const TV_SIZE: usize = 16;

// =============================================================================
// Inline replacements for C static-inline helpers
// =============================================================================

#[inline]
fn ascii_isdigit(c: c_int) -> bool {
    (b'0' as c_int..=b'9' as c_int).contains(&c)
}

#[inline]
fn ascii_isxdigit(c: c_int) -> bool {
    ascii_isdigit(c)
        || (b'a' as c_int..=b'f' as c_int).contains(&c)
        || (b'A' as c_int..=b'F' as c_int).contains(&c)
}

// =============================================================================
// C function declarations
// =============================================================================

extern "C" {
    fn nvim_tv_set_type(tv: TypevalHandle, v_type: c_int);
    fn nvim_tv_set_lock(tv: TypevalHandle, lock: c_int);
    fn nvim_tv_set_list(tv: TypevalHandle, l: ListHandle);
    fn nvim_tv_set_dict(tv: TypevalHandle, d: DictHandle);
    fn nvim_tv_set_string(tv: TypevalHandle, s: *mut c_char);
    fn nvim_tv_set_bool(tv: TypevalHandle, val: c_int);
    fn nvim_tv_set_float(tv: TypevalHandle, f: f64);
    fn nvim_tv_set_number(tv: TypevalHandle, n: i64);
    fn nvim_tv_set_special(tv: TypevalHandle, val: c_int);

    fn tv_clear(tv: TypevalHandle);

    fn tv_list_alloc(len: isize) -> ListHandle;
    // tv_list_ref is static inline; use the C-callable nvim_list_ref wrapper.
    #[link_name = "nvim_list_ref"]
    fn tv_list_ref(l: ListHandle);
    // tv_list_len is static inline; use the C-callable nvim_list_get_len wrapper.
    #[link_name = "nvim_list_get_len"]
    fn tv_list_len(l: ListHandle) -> c_int;
    fn tv_list_append_list(l: ListHandle, itemlist: ListHandle);
    fn tv_list_append_number(l: ListHandle, n: i64);
    /// Move a typval (by pointer) into a new list item (Phase 1 accessor).
    fn nvim_tv_list_append_typval_ptr(l: ListHandle, tv: TypevalHandle);
    /// Append VAR_UNKNOWN to list and return pointer to the item's typval
    /// (needed by the mpack array enter handler, Phase 3 accessor).
    fn nvim_tv_list_append_unknown_and_get(l: ListHandle) -> TypevalHandle;

    fn tv_dict_alloc() -> DictHandle;
    fn tv_dict_item_alloc_len(key: *const c_char, len: usize) -> DictItemHandle;
    fn tv_dict_item_alloc(key: *const c_char) -> DictItemHandle;
    fn tv_dict_add(d: DictHandle, item: DictItemHandle) -> c_int;
    /// `keylen` must match `ptrdiff_t` / `isize` — NOT c_int. Passing c_int
    /// causes the upper 32 bits of the register to be undefined (zero-extended
    /// on x86-64), so `-1i32` arrives as `4294967295isize` (positive), which
    /// directs `nvim_dict_find` to call `hash_find_len` with a ~4 GB length
    /// and immediately segfaults.
    fn tv_dict_find(d: DictHandle, key: *const c_char, keylen: isize) -> DictItemHandle;
    fn nvim_dictitem_di_tv(di: DictItemHandle) -> TypevalHandle;
    fn nvim_dict_inc_refcount(d: DictHandle);
    fn nvim_dict_get_ht_used(d: DictHandle) -> usize;
    fn nvim_dict_item_free(item: DictItemHandle);

    fn tv_blob_alloc_ret(tv: TypevalHandle) -> BlobHandle;
    fn nvim_blob_set_ga_data(b: BlobHandle, data: *mut u8);
    fn nvim_blob_set_ga_len(b: BlobHandle, len: c_int);
    fn nvim_blob_set_ga_maxlen(b: BlobHandle, n: c_int);

    fn xmalloc(size: usize) -> *mut c_void;
    fn xmallocz(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
    #[link_name = "memchr"]
    fn libc_memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void;
    #[link_name = "strlen"]
    fn libc_strlen(s: *const c_char) -> usize;

    fn semsg(fmt: *const c_char, ...);
    fn emsg(fmt: *const c_char) -> c_int;

    fn nvim_eval_msgpack_type_list(idx: c_int) -> ListHandle;

    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    // ascii_isdigit / ascii_isxdigit are static inline in C; implemented inline below.

    fn vim_str2nr(
        start: *const c_char,
        prep: *mut c_int,
        len: *mut c_int,
        what: c_int,
        nptr: *mut i64,
        unptr: *mut u64,
        maxlen: c_int,
        strict: bool,
        overflow: *mut bool,
    );

    fn rs_string2float(text: *const c_char, ret_value: *mut f64) -> usize;

    fn encode_list_write(data: *mut c_void, buf: *const c_char, len: usize);

    fn mpack_parse(
        parser: *mut MpackParser,
        data: *mut *const c_char,
        size: *mut usize,
        enter_cb: unsafe extern "C" fn(*mut MpackParser, *mut MpackNode),
        exit_cb: unsafe extern "C" fn(*mut MpackParser, *mut MpackNode),
    ) -> c_int;
    fn mpack_parser_init(parser: *mut MpackParser, capacity: u32);
    fn mpack_unpack_boolean(tok: MpackToken) -> bool;
    fn mpack_unpack_sint(tok: MpackToken) -> i64;
    fn mpack_unpack_uint(tok: MpackToken) -> u64;
    // mpack_unpack_float is a macro for mpack_unpack_float_fast; call the real symbol.
    #[link_name = "mpack_unpack_float_fast"]
    fn mpack_unpack_float(tok: MpackToken) -> f64;

    fn nvim_mpack_parser_data_ptr(parser: *mut MpackParser) -> *mut *mut c_void;
    fn nvim_mpack_parser_size(parser: *mut MpackParser) -> u32;
    fn nvim_mpack_parser_item(parser: *mut MpackParser, idx: u32) -> *mut MpackNode;
    fn nvim_mpack_parser_alloc_size() -> usize;
}

extern "C" {
    // blob_T* == garray_T* (bv_ga is first field); call via nvim_blob_ga_concat_len
    // which is a C wrapper in decode.c that avoids the clashing-extern-declarations
    // issue (lib.rs also declares ga_concat_len with the concrete GarrayT type).
    fn nvim_blob_ga_concat_len(gap: *mut c_void, data: *const c_char, len: usize);
}

// =============================================================================
// mpack struct mirrors
// =============================================================================

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MpackValue {
    pub lo: u32,
    pub hi: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union MpackTokenData {
    pub value: MpackValue,
    pub chunk_ptr: *const c_char,
    pub ext_type: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MpackToken {
    pub tok_type: c_int,
    pub length: u32,
    pub data: MpackTokenData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union MpackData {
    pub p: *mut c_void,
    pub u: u64,
    pub i: i64,
    pub d: f64,
}

#[repr(C)]
pub struct MpackNode {
    pub tok: MpackToken,
    pub pos: usize,
    pub key_visited: c_int,
    pub data: [MpackData; 2],
}

/// Opaque `mpack_parser_t` — never mirror the variable-length layout.
#[repr(C)]
pub struct MpackParser {
    _opaque: [u8; 0],
}

// =============================================================================
// typval_T helper constructors (raw bytes, 16 bytes on 64-bit)
// =============================================================================

/// Read `v_type` (offset 0, 4 bytes).
#[inline]
fn tv_get_type(b: &[u8; TV_SIZE]) -> c_int {
    i32::from_ne_bytes(b[0..4].try_into().unwrap())
}

/// Read the vval pointer (offset 8, 8 bytes).
#[inline]
fn tv_get_vptr(b: &[u8; TV_SIZE]) -> *mut c_void {
    usize::from_ne_bytes(b[8..16].try_into().unwrap()) as *mut c_void
}

#[inline]
fn make_tv_list(list: ListHandle) -> [u8; TV_SIZE] {
    let mut b = [0u8; TV_SIZE];
    b[0..4].copy_from_slice(&VAR_LIST.to_ne_bytes());
    b[8..16].copy_from_slice(&(list as usize).to_ne_bytes());
    b
}

#[inline]
fn make_tv_dict(dict: DictHandle) -> [u8; TV_SIZE] {
    let mut b = [0u8; TV_SIZE];
    b[0..4].copy_from_slice(&VAR_DICT.to_ne_bytes());
    b[8..16].copy_from_slice(&(dict as usize).to_ne_bytes());
    b
}

#[inline]
fn make_tv_number(n: i64) -> [u8; TV_SIZE] {
    let mut b = [0u8; TV_SIZE];
    b[0..4].copy_from_slice(&VAR_NUMBER.to_ne_bytes());
    b[8..16].copy_from_slice(&n.to_ne_bytes());
    b
}

#[inline]
fn make_tv_float(f: f64) -> [u8; TV_SIZE] {
    let mut b = [0u8; TV_SIZE];
    b[0..4].copy_from_slice(&VAR_FLOAT.to_ne_bytes());
    b[8..16].copy_from_slice(&f.to_ne_bytes());
    b
}

#[inline]
fn make_tv_bool(val: c_int) -> [u8; TV_SIZE] {
    let mut b = [0u8; TV_SIZE];
    b[0..4].copy_from_slice(&VAR_BOOL.to_ne_bytes());
    b[8..12].copy_from_slice(&val.to_ne_bytes());
    b
}

#[inline]
fn make_tv_special(val: c_int) -> [u8; TV_SIZE] {
    let mut b = [0u8; TV_SIZE];
    b[0..4].copy_from_slice(&VAR_SPECIAL.to_ne_bytes());
    b[8..12].copy_from_slice(&val.to_ne_bytes());
    b
}

// =============================================================================
// Phase 1: create_special_dict, decode_create_map_special_dict, decode_string
// =============================================================================

/// Allocate `{_TYPE: eval_msgpack_type_lists[mp_type], _VAL: *val_tv}`.
/// `val_tv` bytes are copied into the dict item (ownership transfer).
unsafe fn create_special_dict(rettv: TypevalHandle, mp_type: c_int, val_tv: &[u8; TV_SIZE]) {
    let dict = tv_dict_alloc();
    nvim_dict_inc_refcount(dict);

    let type_di = tv_dict_item_alloc_len(b"_TYPE\0".as_ptr().cast(), 5);
    let type_di_tv = nvim_dictitem_di_tv(type_di);
    let mp_list = nvim_eval_msgpack_type_list(mp_type);
    nvim_tv_set_type(type_di_tv, VAR_LIST);
    nvim_tv_set_lock(type_di_tv, VAR_UNLOCKED);
    nvim_tv_set_list(type_di_tv, mp_list);
    tv_list_ref(mp_list);
    tv_dict_add(dict, type_di);

    let val_di = tv_dict_item_alloc_len(b"_VAL\0".as_ptr().cast(), 4);
    let val_di_tv = nvim_dictitem_di_tv(val_di);
    ptr::copy_nonoverlapping(val_tv.as_ptr(), val_di_tv as *mut u8, TV_SIZE);
    tv_dict_add(dict, val_di);

    nvim_tv_set_type(rettv, VAR_DICT);
    nvim_tv_set_lock(rettv, VAR_UNLOCKED);
    nvim_tv_set_dict(rettv, dict);
}

/// Build a new `{_TYPE: msgpack_map, _VAL: list}` special dict and return the list.
#[unsafe(export_name = "decode_create_map_special_dict")]
pub unsafe extern "C" fn rs_decode_create_map_special_dict(
    ret_tv: TypevalHandle,
    len: isize,
) -> ListHandle {
    let list = tv_list_alloc(len);
    tv_list_ref(list);
    let val_tv = make_tv_list(list);
    create_special_dict(ret_tv, K_MP_MAP, &val_tv);
    list
}

/// Convert `(s, len)` to a typval (VAR_BLOB or VAR_STRING) at `*rettv`.
///
/// Pointer-out form avoids passing `typval_T` by value over FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_decode_string_into(
    s: *const c_char,
    len: usize,
    force_blob: bool,
    s_allocated: bool,
    rettv: TypevalHandle,
) {
    debug_assert!(!rettv.is_null());
    debug_assert!(!s.is_null() || len == 0);

    let use_blob = force_blob || (!s.is_null() && !libc_memchr(s.cast(), 0, len).is_null());

    if use_blob {
        let blob = tv_blob_alloc_ret(rettv);
        if s_allocated {
            nvim_blob_set_ga_data(blob, s as *mut u8);
            nvim_blob_set_ga_len(blob, len as c_int);
            nvim_blob_set_ga_maxlen(blob, len as c_int);
        } else {
            // blob_T* == garray_T* (bv_ga is first field)
            nvim_blob_ga_concat_len(blob, s, len);
        }
    } else {
        let str_ptr: *mut c_char = if s.is_null() {
            ptr::null_mut()
        } else if s_allocated {
            s as *mut c_char
        } else {
            xmemdupz(s.cast(), len)
        };
        nvim_tv_set_type(rettv, VAR_STRING);
        nvim_tv_set_lock(rettv, VAR_UNLOCKED);
        nvim_tv_set_string(rettv, str_ptr);
    }
}

// =============================================================================
// Phase 2: JSON decoder
// =============================================================================

struct ValuesStackItem {
    is_special_string: bool,
    didcomma: bool,
    didcolon: bool,
    val: [u8; TV_SIZE],
}

struct ContainerStackItem {
    stack_index: usize,
    special_val: ListHandle,
    s: *const c_char,
    container: [u8; TV_SIZE],
}

unsafe impl Send for ContainerStackItem {}
unsafe impl Send for ValuesStackItem {}

/// Mirrors C `json_decoder_pop`.
unsafe fn json_decoder_pop(
    obj: ValuesStackItem,
    stack: &mut Vec<ValuesStackItem>,
    container_stack: &mut Vec<ContainerStackItem>,
    pp: &mut *const c_char,
    next_map_special: &mut bool,
    didcomma: &mut bool,
    didcolon: &mut bool,
) -> Result<(), ()> {
    if container_stack.is_empty() {
        stack.push(obj);
        return Ok(());
    }

    // Determine val_location and the active last_container.
    // In C: if obj == last_container's container, pop it first.
    let val_location: *const c_char;
    {
        let lc = container_stack.last().unwrap();
        let (obj_type, obj_ptr) = (tv_get_type(&obj.val), tv_get_vptr(&obj.val));
        let (cont_type, cont_ptr) = (tv_get_type(&lc.container), tv_get_vptr(&lc.container));
        if obj_type == cont_type && obj_ptr == cont_ptr {
            // Closing the container itself.
            let popped = container_stack.pop().unwrap();
            val_location = popped.s;
            // Fall through with the NEW last container.
        } else {
            val_location = *pp;
        }
    }

    if container_stack.is_empty() {
        // After popping, no container remains; push obj.
        stack.push(obj);
        return Ok(());
    }

    let lc_type = tv_get_type(&container_stack.last().unwrap().container);
    let lc_ptr = tv_get_vptr(&container_stack.last().unwrap().container);
    let lc_special_val = container_stack.last().unwrap().special_val;
    let lc_stack_index = container_stack.last().unwrap().stack_index;

    if lc_type == VAR_LIST {
        if tv_list_len(lc_ptr) != 0 && !obj.didcomma {
            semsg(
                b"E474: Expected comma before list item: %s\0"
                    .as_ptr()
                    .cast(),
                val_location,
            );
            tv_clear(obj.val.as_ptr().cast_mut().cast());
            return Err(());
        }
        debug_assert!(lc_special_val.is_null());
        nvim_tv_list_append_typval_ptr(lc_ptr, obj.val.as_ptr().cast_mut().cast());
    } else if lc_stack_index == stack.len().wrapping_sub(2) {
        // Dict value.
        if !obj.didcolon {
            semsg(
                b"E474: Expected colon before dictionary value: %s\0"
                    .as_ptr()
                    .cast(),
                val_location,
            );
            tv_clear(obj.val.as_ptr().cast_mut().cast());
            return Err(());
        }
        let key = stack.pop().unwrap();
        // Re-read lc after pop (size changed).
        let lc2 = container_stack.last().unwrap();
        if lc2.special_val.is_null() {
            debug_assert!(!key.is_special_string);
            let key_str = tv_get_vptr(&key.val) as *const c_char;
            debug_assert!(!key_str.is_null());
            let obj_di = tv_dict_item_alloc(key_str);
            tv_clear(key.val.as_ptr().cast_mut().cast());
            let dict = tv_get_vptr(&lc2.container);
            if tv_dict_add(dict, obj_di) == FAIL {
                panic!("tv_dict_add failed for key that should be unique");
            }
            let di_tv = nvim_dictitem_di_tv(obj_di);
            ptr::copy_nonoverlapping(obj.val.as_ptr(), di_tv as *mut u8, TV_SIZE);
        } else {
            let special_val = lc2.special_val;
            let kv_pair = tv_list_alloc(2);
            tv_list_append_list(special_val, kv_pair);
            nvim_tv_list_append_typval_ptr(kv_pair, key.val.as_ptr().cast_mut().cast());
            nvim_tv_list_append_typval_ptr(kv_pair, obj.val.as_ptr().cast_mut().cast());
        }
    } else {
        // Key position.
        let obj_type = tv_get_type(&obj.val);
        if !obj.is_special_string && obj_type != VAR_STRING {
            semsg(b"E474: Expected string key: %s\0".as_ptr().cast(), *pp);
            tv_clear(obj.val.as_ptr().cast_mut().cast());
            return Err(());
        } else if !obj.didcomma && lc_special_val.is_null() && lc_type == VAR_DICT {
            if nvim_dict_get_ht_used(lc_ptr) != 0 {
                semsg(
                    b"E474: Expected comma before dictionary key: %s\0"
                        .as_ptr()
                        .cast(),
                    val_location,
                );
                tv_clear(obj.val.as_ptr().cast_mut().cast());
                return Err(());
            }
        }

        // Restart check.
        let key_str = if obj_type == VAR_STRING {
            tv_get_vptr(&obj.val) as *const c_char
        } else {
            ptr::null()
        };
        let should_restart = lc_special_val.is_null()
            && (obj.is_special_string
                || key_str.is_null()
                || !tv_dict_find(lc_ptr, key_str, -1).is_null());

        if should_restart {
            tv_clear(obj.val.as_ptr().cast_mut().cast());
            let popped_lc = container_stack.pop().unwrap();
            let (saved_comma, saved_colon) = {
                let saved = &stack[popped_lc.stack_index];
                (saved.didcomma, saved.didcolon)
            };
            while stack.len() > popped_lc.stack_index {
                let top = stack.pop().unwrap();
                tv_clear(top.val.as_ptr().cast_mut().cast());
            }
            *pp = popped_lc.s;
            *didcomma = saved_comma;
            *didcolon = saved_colon;
            *next_map_special = true;
            return Ok(());
        }

        stack.push(obj);
    }
    Ok(())
}

/// Mirrors C `parse_json_string`.
unsafe fn parse_json_string(
    buf: *const c_char,
    buf_len: usize,
    pp: &mut *const c_char,
    stack: &mut Vec<ValuesStackItem>,
    container_stack: &mut Vec<ContainerStackItem>,
    next_map_special: &mut bool,
    didcomma: &mut bool,
    didcolon: &mut bool,
) -> Result<(), ()> {
    let e = buf.add(buf_len);
    let mut p = (*pp).add(1); // skip '"'
    let s = p;
    let mut len: usize = 0;

    // First pass: validate + compute length
    while p < e && *p != b'"' as c_char {
        if *p == b'\\' as c_char {
            p = p.add(1);
            if p == e {
                semsg(
                    b"E474: Unfinished escape sequence: %.*s\0".as_ptr().cast(),
                    buf_len as c_int,
                    buf,
                );
                *pp = p;
                return Err(());
            }
            match *p as u8 {
                b'u' => {
                    if p.add(4) >= e {
                        semsg(
                            b"E474: Unfinished unicode escape sequence: %.*s\0"
                                .as_ptr()
                                .cast(),
                            buf_len as c_int,
                            buf,
                        );
                        *pp = p;
                        return Err(());
                    }
                    if !ascii_isxdigit(*p.add(1) as c_int)
                        || !ascii_isxdigit(*p.add(2) as c_int)
                        || !ascii_isxdigit(*p.add(3) as c_int)
                        || !ascii_isxdigit(*p.add(4) as c_int)
                    {
                        semsg(
                            b"E474: Expected four hex digits after \\u: %.*s\0"
                                .as_ptr()
                                .cast(),
                            (e as isize - p.sub(1) as isize) as c_int,
                            p.sub(1),
                        );
                        *pp = p;
                        return Err(());
                    }
                    len += 3;
                    p = p.add(5);
                }
                b'\\' | b'/' | b'"' | b't' | b'b' | b'n' | b'r' | b'f' => {
                    len += 1;
                    p = p.add(1);
                }
                _ => {
                    semsg(
                        b"E474: Unknown escape sequence: %.*s\0".as_ptr().cast(),
                        (e as isize - p.sub(1) as isize) as c_int,
                        p.sub(1),
                    );
                    *pp = p;
                    return Err(());
                }
            }
        } else {
            let pb = *p as u8;
            if pb < 0x20 {
                semsg(
                    b"E474: ASCII control characters cannot be present inside string: %.*s\0"
                        .as_ptr()
                        .cast(),
                    (e as isize - p as isize) as c_int,
                    p,
                );
                *pp = p;
                return Err(());
            }
            let ch = utf_ptr2char(p);
            if ch >= 0x80
                && pb == ch as u8
                && !(ch == 0xC3 && p.add(1) < e && *p.add(1) as u8 == 0x83)
            {
                semsg(
                    b"E474: Only UTF-8 strings allowed: %.*s\0".as_ptr().cast(),
                    (e as isize - p as isize) as c_int,
                    p,
                );
                *pp = p;
                return Err(());
            } else if ch > 0x10FFFF {
                semsg(b"E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: %.*s\0"
                    .as_ptr().cast(), (e as isize - p as isize) as c_int, p);
                *pp = p;
                return Err(());
            }
            let ch_len = utf_char2len(ch) as usize;
            len += ch_len;
            p = p.add(ch_len);
        }
    }

    if p == e || *p != b'"' as c_char {
        semsg(
            b"E474: Expected string end: %.*s\0".as_ptr().cast(),
            buf_len as c_int,
            buf,
        );
        *pp = p;
        return Err(());
    }

    let str_out = xmalloc(len + 1) as *mut c_char;
    let mut fst_in_pair: u32 = 0;
    let mut str_end = str_out;

    macro_rules! flush_fst {
        () => {
            if fst_in_pair != 0 {
                str_end = str_end.add(utf_char2bytes(fst_in_pair as c_int, str_end) as usize);
                fst_in_pair = 0;
            }
        };
    }

    // Second pass: write
    let mut t = s;
    while t < p {
        if !(*t == b'\\' as c_char && t.add(1) < p && *t.add(1) == b'u' as c_char) {
            flush_fst!();
        }
        if *t == b'\\' as c_char {
            t = t.add(1);
            match *t as u8 {
                b'u' => {
                    let ubuf: [c_char; 4] = [*t.add(1), *t.add(2), *t.add(3), *t.add(4)];
                    t = t.add(4);
                    let mut ch_val: u64 = 0;
                    vim_str2nr(
                        ubuf.as_ptr(),
                        ptr::null_mut(),
                        ptr::null_mut(),
                        STR2NR_HEX | STR2NR_FORCE,
                        ptr::null_mut(),
                        &mut ch_val,
                        4,
                        true,
                        ptr::null_mut(),
                    );
                    let ch = ch_val as u32;
                    if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&ch) {
                        flush_fst!();
                        fst_in_pair = ch;
                    } else if (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&ch)
                        && fst_in_pair != 0
                    {
                        let full = (ch - SURROGATE_LO_START)
                            + ((fst_in_pair - SURROGATE_HI_START) << 10)
                            + SURROGATE_FIRST_CHAR;
                        str_end = str_end.add(utf_char2bytes(full as c_int, str_end) as usize);
                        fst_in_pair = 0;
                    } else {
                        flush_fst!();
                        str_end = str_end.add(utf_char2bytes(ch as c_int, str_end) as usize);
                    }
                    t = t.add(1);
                    continue;
                }
                c => {
                    let out_byte: u8 = match c {
                        b'\\' => b'\\',
                        b'/' => b'/',
                        b'"' => b'"',
                        b't' => b'\t',
                        b'b' => 0x08,
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b'f' => 0x0C,
                        _ => unreachable!(),
                    };
                    *str_end = out_byte as c_char;
                    str_end = str_end.add(1);
                }
            }
        } else {
            *str_end = *t;
            str_end = str_end.add(1);
        }
        t = t.add(1);
    }
    flush_fst!();
    *str_end = 0;

    let out_len = str_end as usize - str_out as usize;
    let mut obj_tv = [0u8; TV_SIZE];
    rs_decode_string_into(str_out, out_len, false, true, obj_tv.as_mut_ptr().cast());
    let is_sp = tv_get_type(&obj_tv) != VAR_STRING;

    let item = ValuesStackItem {
        is_special_string: is_sp,
        didcomma: *didcomma,
        didcolon: *didcolon,
        val: obj_tv,
    };
    *pp = p;
    json_decoder_pop(
        item,
        stack,
        container_stack,
        pp,
        next_map_special,
        didcomma,
        didcolon,
    )
}

/// Mirrors C `parse_json_number`.
unsafe fn parse_json_number(
    buf: *const c_char,
    buf_len: usize,
    pp: &mut *const c_char,
    stack: &mut Vec<ValuesStackItem>,
    container_stack: &mut Vec<ContainerStackItem>,
    next_map_special: &mut bool,
    didcomma: &mut bool,
    didcolon: &mut bool,
) -> Result<(), ()> {
    let e = buf.add(buf_len);
    let mut p = *pp;
    let s = p;
    let mut fracs: *const c_char = ptr::null();
    let mut exps: *const c_char = ptr::null();
    let mut exps_s: *const c_char = ptr::null();

    if *p == b'-' as c_char {
        p = p.add(1);
    }
    let ints = p;

    while p < e && ascii_isdigit(*p as c_int) {
        p = p.add(1);
    }

    if p != ints.add(1) && !ints.is_null() && *ints == b'0' as c_char {
        semsg(
            b"E474: Leading zeroes are not allowed: %.*s\0"
                .as_ptr()
                .cast(),
            (e as isize - s as isize) as c_int,
            s,
        );
        *pp = p;
        return Err(());
    }

    if p < e && p != ints && *p == b'.' as c_char {
        p = p.add(1);
        fracs = p;
        while p < e && ascii_isdigit(*p as c_int) {
            p = p.add(1);
        }
    }

    if p < e && (*p == b'e' as c_char || *p == b'E' as c_char) {
        p = p.add(1);
        exps_s = p;
        if p < e && (*p == b'-' as c_char || *p == b'+' as c_char) {
            p = p.add(1);
        }
        exps = p;
        while p < e && ascii_isdigit(*p as c_int) {
            p = p.add(1);
        }
    }

    if p == ints {
        semsg(
            b"E474: Missing number after minus sign: %.*s\0"
                .as_ptr()
                .cast(),
            (e as isize - s as isize) as c_int,
            s,
        );
        *pp = p;
        return Err(());
    } else if p == fracs || (!fracs.is_null() && !exps_s.is_null() && exps_s == fracs.add(1)) {
        semsg(
            b"E474: Missing number after decimal dot: %.*s\0"
                .as_ptr()
                .cast(),
            (e as isize - s as isize) as c_int,
            s,
        );
        *pp = p;
        return Err(());
    } else if !exps.is_null() && p == exps {
        semsg(
            b"E474: Missing exponent: %.*s\0".as_ptr().cast(),
            (e as isize - s as isize) as c_int,
            s,
        );
        *pp = p;
        return Err(());
    }

    let exp_num_len = p as usize - s as usize;
    let tv = if !fracs.is_null() || !exps.is_null() {
        let mut f_val = 0.0f64;
        let num_len = rs_string2float(s, &mut f_val);
        if exp_num_len != num_len {
            semsg(b"E685: internal error: while converting number \"%.*s\" to float string2float consumed %zu bytes in place of %zu\0"
                .as_ptr().cast(), exp_num_len as c_int, s, num_len, exp_num_len);
        }
        make_tv_float(f_val)
    } else {
        let mut nr: i64 = 0;
        let mut num_len_out: c_int = 0;
        vim_str2nr(
            s,
            ptr::null_mut(),
            &mut num_len_out,
            0,
            &mut nr,
            ptr::null_mut(),
            (p as isize - s as isize) as c_int,
            true,
            ptr::null_mut(),
        );
        if exp_num_len != num_len_out as usize {
            semsg(b"E685: internal error: while converting number \"%.*s\" to integer vim_str2nr consumed %i bytes in place of %zu\0"
                .as_ptr().cast(), exp_num_len as c_int, s, num_len_out, exp_num_len);
        }
        make_tv_number(nr)
    };

    let item = ValuesStackItem {
        is_special_string: false,
        didcomma: *didcomma,
        didcolon: *didcolon,
        val: tv,
    };
    json_decoder_pop(
        item,
        stack,
        container_stack,
        &mut p,
        next_map_special,
        didcomma,
        didcolon,
    )?;
    if !*next_map_special {
        p = p.sub(1);
    }
    *pp = p;
    Ok(())
}

/// Mirrors C `json_decode_string`.
#[unsafe(export_name = "json_decode_string")]
pub unsafe extern "C" fn rs_json_decode_string(
    buf: *const c_char,
    buf_len: usize,
    rettv: TypevalHandle,
) -> c_int {
    let e = buf.add(buf_len);
    let mut p = buf;

    while p < e && matches!(*p as u8, b' ' | b'\t' | b'\n' | b'\r') {
        p = p.add(1);
    }
    if p == e {
        emsg(b"E474: Attempt to decode a blank string\0".as_ptr().cast());
        return FAIL;
    }

    nvim_tv_set_type(rettv, VAR_UNKNOWN);
    let mut stack: Vec<ValuesStackItem> = Vec::new();
    let mut container_stack: Vec<ContainerStackItem> = Vec::new();
    let mut didcomma = false;
    let mut didcolon = false;
    let mut next_map_special = false;
    let mut succeeded = false;

    'outer: loop {
        // Mirror the C `for (; p < e; p++)` guard: exit loop when end of
        // input is reached.  The after-cycle code below emits
        // "Unexpected end of input" when the container stack is non-empty
        // (same as C's json_decode_string_after_cycle path).
        if p >= e {
            succeeded = true;
            break 'outer;
        }

        debug_assert!(*p == b'{' as c_char || !next_map_special);

        let needs_advance: bool = match *p as u8 {
            b'}' | b']' => {
                if container_stack.is_empty() {
                    semsg(
                        b"E474: No container to close: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                let lc = container_stack.last().unwrap();
                let lc_type = tv_get_type(&lc.container);
                let lc_si = lc.stack_index;
                if *p == b'}' as c_char && lc_type != VAR_DICT {
                    semsg(
                        b"E474: Closing list with curly bracket: %.*s\0"
                            .as_ptr()
                            .cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if *p == b']' as c_char && lc_type != VAR_LIST {
                    semsg(
                        b"E474: Closing dictionary with square bracket: %.*s\0"
                            .as_ptr()
                            .cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if didcomma {
                    semsg(
                        b"E474: Trailing comma: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if didcolon {
                    semsg(
                        b"E474: Expected value after colon: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if lc_si != stack.len().wrapping_sub(1) {
                    semsg(
                        b"E474: Expected value: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                if stack.len() == 1 {
                    p = p.add(1);
                    container_stack.pop();
                    succeeded = true;
                    break 'outer;
                } else {
                    let top = stack.pop().unwrap();
                    if json_decoder_pop(
                        top,
                        &mut stack,
                        &mut container_stack,
                        &mut p,
                        &mut next_map_special,
                        &mut didcomma,
                        &mut didcolon,
                    )
                    .is_err()
                    {
                        break 'outer;
                    }
                    debug_assert!(!next_map_special);
                    false
                }
            }
            b',' => {
                if container_stack.is_empty() {
                    semsg(
                        b"E474: Comma not inside container: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                let lc = container_stack.last().unwrap();
                let lc_type = tv_get_type(&lc.container);
                let lc_ptr = tv_get_vptr(&lc.container);
                let lc_si = lc.stack_index;
                let lc_sv = lc.special_val;
                if didcomma {
                    semsg(
                        b"E474: Duplicate comma: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if didcolon {
                    semsg(
                        b"E474: Comma after colon: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if lc_type == VAR_DICT && lc_si != stack.len().wrapping_sub(1) {
                    semsg(
                        b"E474: Using comma in place of colon: %.*s\0"
                            .as_ptr()
                            .cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else {
                    let is_empty = if lc_sv.is_null() {
                        if lc_type == VAR_DICT {
                            nvim_dict_get_ht_used(lc_ptr) == 0
                        } else {
                            tv_list_len(lc_ptr) == 0
                        }
                    } else {
                        tv_list_len(lc_sv) == 0
                    };
                    if is_empty {
                        semsg(
                            b"E474: Leading comma: %.*s\0".as_ptr().cast(),
                            (e as isize - p as isize) as c_int,
                            p,
                        );
                        break 'outer;
                    }
                }
                didcomma = true;
                p = p.add(1);
                continue 'outer;
            }
            b':' => {
                if container_stack.is_empty() {
                    semsg(
                        b"E474: Colon not inside container: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                let lc = container_stack.last().unwrap();
                let lc_type = tv_get_type(&lc.container);
                let lc_si = lc.stack_index;
                if lc_type != VAR_DICT {
                    semsg(
                        b"E474: Using colon not in dictionary: %.*s\0"
                            .as_ptr()
                            .cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if lc_si != stack.len().wrapping_sub(2) {
                    semsg(
                        b"E474: Unexpected colon: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if didcomma {
                    semsg(
                        b"E474: Colon after comma: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                } else if didcolon {
                    semsg(
                        b"E474: Duplicate colon: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                didcolon = true;
                p = p.add(1);
                continue 'outer;
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                p = p.add(1);
                continue 'outer;
            }
            b'n' => {
                if p.add(3) >= e
                    || *p.add(1) != b'u' as c_char
                    || *p.add(2) != b'l' as c_char
                    || *p.add(3) != b'l' as c_char
                {
                    semsg(
                        b"E474: Expected null: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                p = p.add(3);
                let item = ValuesStackItem {
                    is_special_string: false,
                    didcomma,
                    didcolon,
                    val: make_tv_special(K_SPECIAL_VAR_NULL),
                };
                if json_decoder_pop(
                    item,
                    &mut stack,
                    &mut container_stack,
                    &mut p,
                    &mut next_map_special,
                    &mut didcomma,
                    &mut didcolon,
                )
                .is_err()
                {
                    break 'outer;
                }
                if next_map_special {
                    // json_decoder_pop already set didcomma/didcolon correctly
                    // via the restart mechanism. Do NOT reset them here — this
                    // mirrors C's `goto json_decode_string_cycle_start` which
                    // does not touch didcomma/didcolon.
                    continue 'outer;
                }
                true
            }
            b't' => {
                if p.add(3) >= e
                    || *p.add(1) != b'r' as c_char
                    || *p.add(2) != b'u' as c_char
                    || *p.add(3) != b'e' as c_char
                {
                    semsg(
                        b"E474: Expected true: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                p = p.add(3);
                let item = ValuesStackItem {
                    is_special_string: false,
                    didcomma,
                    didcolon,
                    val: make_tv_bool(K_BOOL_VAR_TRUE),
                };
                if json_decoder_pop(
                    item,
                    &mut stack,
                    &mut container_stack,
                    &mut p,
                    &mut next_map_special,
                    &mut didcomma,
                    &mut didcolon,
                )
                .is_err()
                {
                    break 'outer;
                }
                if next_map_special {
                    // json_decoder_pop already set didcomma/didcolon correctly
                    // via the restart mechanism. Do NOT reset them here — this
                    // mirrors C's `goto json_decode_string_cycle_start` which
                    // does not touch didcomma/didcolon.
                    continue 'outer;
                }
                true
            }
            b'f' => {
                if p.add(4) >= e
                    || *p.add(1) != b'a' as c_char
                    || *p.add(2) != b'l' as c_char
                    || *p.add(3) != b's' as c_char
                    || *p.add(4) != b'e' as c_char
                {
                    semsg(
                        b"E474: Expected false: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    break 'outer;
                }
                p = p.add(4);
                let item = ValuesStackItem {
                    is_special_string: false,
                    didcomma,
                    didcolon,
                    val: make_tv_bool(K_BOOL_VAR_FALSE),
                };
                if json_decoder_pop(
                    item,
                    &mut stack,
                    &mut container_stack,
                    &mut p,
                    &mut next_map_special,
                    &mut didcomma,
                    &mut didcolon,
                )
                .is_err()
                {
                    break 'outer;
                }
                if next_map_special {
                    // json_decoder_pop already set didcomma/didcolon correctly
                    // via the restart mechanism. Do NOT reset them here — this
                    // mirrors C's `goto json_decode_string_cycle_start` which
                    // does not touch didcomma/didcolon.
                    continue 'outer;
                }
                true
            }
            b'"' => {
                if parse_json_string(
                    buf,
                    buf_len,
                    &mut p,
                    &mut stack,
                    &mut container_stack,
                    &mut next_map_special,
                    &mut didcomma,
                    &mut didcolon,
                )
                .is_err()
                {
                    break 'outer;
                }
                if next_map_special {
                    // json_decoder_pop already set didcomma/didcolon correctly
                    // via the restart mechanism. Do NOT reset them here — this
                    // mirrors C's `goto json_decode_string_cycle_start` which
                    // does not touch didcomma/didcolon.
                    continue 'outer;
                }
                // parse_json_string leaves *pp pointing at the closing '"'.
                // The outer loop must advance past it (mirrors the C for loop's p++).
                true
            }
            b'-' | b'0'..=b'9' => {
                if parse_json_number(
                    buf,
                    buf_len,
                    &mut p,
                    &mut stack,
                    &mut container_stack,
                    &mut next_map_special,
                    &mut didcomma,
                    &mut didcolon,
                )
                .is_err()
                {
                    break 'outer;
                }
                if next_map_special {
                    // json_decoder_pop already set didcomma/didcolon correctly
                    // via the restart mechanism. Do NOT reset them here — this
                    // mirrors C's `goto json_decode_string_cycle_start` which
                    // does not touch didcomma/didcolon.
                    continue 'outer;
                }
                true
            }
            b'[' => {
                let list = tv_list_alloc(K_LIST_LEN_MAY_KNOW);
                tv_list_ref(list);
                let ctv = make_tv_list(list);
                container_stack.push(ContainerStackItem {
                    stack_index: stack.len(),
                    special_val: ptr::null_mut(),
                    s: p,
                    container: ctv,
                });
                stack.push(ValuesStackItem {
                    is_special_string: false,
                    didcomma,
                    didcolon,
                    val: ctv,
                });
                false
            }
            b'{' => {
                let (ctv, val_list) = if next_map_special {
                    next_map_special = false;
                    let mut tmp = [0u8; TV_SIZE];
                    let vl = rs_decode_create_map_special_dict(
                        tmp.as_mut_ptr().cast(),
                        K_LIST_LEN_MAY_KNOW,
                    );
                    (tmp, vl)
                } else {
                    let dict = tv_dict_alloc();
                    nvim_dict_inc_refcount(dict);
                    (make_tv_dict(dict), ptr::null_mut())
                };
                container_stack.push(ContainerStackItem {
                    stack_index: stack.len(),
                    special_val: val_list,
                    s: p,
                    container: ctv,
                });
                stack.push(ValuesStackItem {
                    is_special_string: false,
                    didcomma,
                    didcolon,
                    val: ctv,
                });
                false
            }
            _ => {
                semsg(
                    b"E474: Unidentified byte: %.*s\0".as_ptr().cast(),
                    (e as isize - p as isize) as c_int,
                    p,
                );
                break 'outer;
            }
        };

        didcomma = false;
        didcolon = false;
        if container_stack.is_empty() {
            if needs_advance {
                p = p.add(1);
            }
            succeeded = true;
            break 'outer;
        }
        p = p.add(1);
    }

    // After-cycle: check trailing chars.
    if succeeded {
        while p < e {
            match *p as u8 {
                b'\n' | b' ' | b'\t' | b'\r' => {}
                _ => {
                    semsg(
                        b"E474: Trailing characters: %.*s\0".as_ptr().cast(),
                        (e as isize - p as isize) as c_int,
                        p,
                    );
                    succeeded = false;
                    break;
                }
            }
            p = p.add(1);
        }
    }

    if succeeded && stack.len() == 1 && container_stack.is_empty() {
        let top = stack.pop().unwrap();
        ptr::copy_nonoverlapping(top.val.as_ptr(), rettv as *mut u8, TV_SIZE);
        return OK;
    }

    if succeeded {
        // stack.len() != 1 means unexpected EOF
        semsg(
            b"E474: Unexpected end of input: %.*s\0".as_ptr().cast(),
            buf_len as c_int,
            buf,
        );
    }

    while !stack.is_empty() {
        let top = stack.pop().unwrap();
        tv_clear(top.val.as_ptr().cast_mut().cast());
    }
    FAIL
}

// =============================================================================
// Phase 3: mpack -> typval pipeline
// =============================================================================

unsafe fn positive_integer_to_special_typval(rettv: TypevalHandle, val: u64) {
    if val <= VARNUMBER_MAX {
        nvim_tv_set_type(rettv, VAR_NUMBER);
        nvim_tv_set_lock(rettv, VAR_UNLOCKED);
        nvim_tv_set_number(rettv, val as i64);
    } else {
        let list = tv_list_alloc(4);
        tv_list_ref(list);
        let val_tv = make_tv_list(list);
        create_special_dict(rettv, K_MP_INTEGER, &val_tv);
        tv_list_append_number(list, 1);
        tv_list_append_number(list, ((val >> 62) & 0x3) as i64);
        tv_list_append_number(list, ((val >> 31) & 0x7FFF_FFFF) as i64);
        tv_list_append_number(list, (val & 0x7FFF_FFFF) as i64);
    }
}

/// MPACK_PARENT_NODE equivalent.
unsafe fn mpack_parent_node(node: *mut MpackNode) -> *mut MpackNode {
    let prev = node.sub(1);
    if (*prev).pos == usize::MAX {
        ptr::null_mut()
    } else {
        prev
    }
}

unsafe extern "C" fn typval_parse_enter(parser: *mut MpackParser, node: *mut MpackNode) {
    let node = &mut *node;
    node.data[0].p = ptr::null_mut();
    node.data[1].p = ptr::null_mut();

    let parent = mpack_parent_node(node as *mut MpackNode);

    let result: TypevalHandle = if !parent.is_null() {
        let par = &mut *parent;
        match par.tok.tok_type {
            MPACK_TOKEN_ARRAY => {
                let list = par.data[1].p as ListHandle;
                nvim_tv_list_append_unknown_and_get(list)
            }
            MPACK_TOKEN_MAP => {
                let items_ptr = par.data[1].p as *mut u8;
                let idx = par.pos * 2 + par.key_visited as usize;
                items_ptr.add(idx * TV_SIZE).cast::<c_void>()
            }
            MPACK_TOKEN_STR | MPACK_TOKEN_BIN | MPACK_TOKEN_EXT => {
                debug_assert_eq!(node.tok.tok_type, MPACK_TOKEN_CHUNK);
                let data = par.data[1].p as *mut u8;
                let chunk_ptr = node.tok.data.chunk_ptr;
                let chunk_len = node.tok.length as usize;
                ptr::copy_nonoverlapping(chunk_ptr as *const u8, data.add(par.pos), chunk_len);
                return;
            }
            _ => panic!("typval_parse_enter: unexpected parent token type"),
        }
    } else {
        *nvim_mpack_parser_data_ptr(parser) as TypevalHandle
    };

    node.data[0].p = result;

    match node.tok.tok_type {
        MPACK_TOKEN_NIL => {
            nvim_tv_set_type(result, VAR_SPECIAL);
            nvim_tv_set_lock(result, VAR_UNLOCKED);
            nvim_tv_set_special(result, K_SPECIAL_VAR_NULL);
        }
        MPACK_TOKEN_BOOLEAN => {
            nvim_tv_set_type(result, VAR_BOOL);
            nvim_tv_set_lock(result, VAR_UNLOCKED);
            nvim_tv_set_bool(
                result,
                if mpack_unpack_boolean(node.tok) {
                    K_BOOL_VAR_TRUE
                } else {
                    K_BOOL_VAR_FALSE
                },
            );
        }
        MPACK_TOKEN_SINT => {
            nvim_tv_set_type(result, VAR_NUMBER);
            nvim_tv_set_lock(result, VAR_UNLOCKED);
            nvim_tv_set_number(result, mpack_unpack_sint(node.tok));
        }
        MPACK_TOKEN_UINT => {
            positive_integer_to_special_typval(result, mpack_unpack_uint(node.tok));
        }
        MPACK_TOKEN_FLOAT => {
            nvim_tv_set_type(result, VAR_FLOAT);
            nvim_tv_set_lock(result, VAR_UNLOCKED);
            nvim_tv_set_float(result, mpack_unpack_float(node.tok));
        }
        MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
            node.data[1].p = xmallocz(node.tok.length as usize);
        }
        MPACK_TOKEN_CHUNK => { /* handled in parent branch */ }
        MPACK_TOKEN_ARRAY => {
            let list = tv_list_alloc(node.tok.length as isize);
            tv_list_ref(list);
            nvim_tv_set_type(result, VAR_LIST);
            nvim_tv_set_lock(result, VAR_UNLOCKED);
            nvim_tv_set_list(result, list);
            node.data[1].p = list;
        }
        MPACK_TOKEN_MAP => {
            let nbytes = node.tok.length as usize * 2 * TV_SIZE;
            node.data[1].p = xmallocz(nbytes);
        }
        _ => {}
    }
}

unsafe extern "C" fn typval_parse_exit(_parser: *mut MpackParser, node: *mut MpackNode) {
    let node = &mut *node;
    let result = node.data[0].p as TypevalHandle;

    match node.tok.tok_type {
        MPACK_TOKEN_BIN | MPACK_TOKEN_STR => {
            rs_decode_string_into(
                node.data[1].p as *const c_char,
                node.tok.length as usize,
                false,
                true,
                result,
            );
            node.data[1].p = ptr::null_mut();
        }
        MPACK_TOKEN_EXT => {
            let list = tv_list_alloc(2);
            tv_list_ref(list);
            tv_list_append_number(list, node.tok.data.ext_type as i64);
            let ext_val_list = tv_list_alloc(K_LIST_LEN_MAY_KNOW);
            tv_list_append_list(list, ext_val_list);
            let val_tv = make_tv_list(list);
            create_special_dict(result, K_MP_EXT, &val_tv);
            encode_list_write(
                ext_val_list as *mut c_void,
                node.data[1].p as *const c_char,
                node.tok.length as usize,
            );
            xfree(node.data[1].p);
            node.data[1].p = ptr::null_mut();
        }
        MPACK_TOKEN_MAP => {
            let items_ptr = node.data[1].p as *mut u8;
            let n = node.tok.length as usize;

            let mut all_valid = true;
            for i in 0..n {
                let key_tv: &[u8; TV_SIZE] =
                    &*(items_ptr.add(i * 2 * TV_SIZE) as *const [u8; TV_SIZE]);
                if tv_get_type(key_tv) != VAR_STRING {
                    all_valid = false;
                    break;
                }
                let ks = tv_get_vptr(key_tv) as *const c_char;
                if ks.is_null() || *ks == 0 {
                    all_valid = false;
                    break;
                }
            }

            let mut built_dict = false;
            if all_valid {
                let dict = tv_dict_alloc();
                nvim_dict_inc_refcount(dict);
                nvim_tv_set_type(result, VAR_DICT);
                nvim_tv_set_lock(result, VAR_UNLOCKED);
                nvim_tv_set_dict(result, dict);

                let mut had_dup = false;
                // Track added dict items so we can neutralize their values before
                // tv_clear() if we hit a duplicate key.  Mirrors C's TV_DICT_ITER
                // loop that sets d->di_tv = VAR_SPECIAL/kSpecialVarNull before
                // tv_clear(result) to prevent double-free of values in items_ptr.
                let mut added_dis: Vec<DictItemHandle> = Vec::with_capacity(n);
                for i in 0..n {
                    let key_tv: &[u8; TV_SIZE] =
                        &*(items_ptr.add(i * 2 * TV_SIZE) as *const [u8; TV_SIZE]);
                    let ks = tv_get_vptr(key_tv) as *const c_char;
                    let kl = libc_strlen(ks);
                    let di = tv_dict_item_alloc_len(ks, kl);
                    let di_tv = nvim_dictitem_di_tv(di);
                    nvim_tv_set_type(di_tv, VAR_UNKNOWN);
                    if tv_dict_add(dict, di) == FAIL {
                        // Duplicate key: neutralize all already-added items' di_tv
                        // values so tv_clear() below does not free them; they are
                        // still referenced by items_ptr and will be consumed by the
                        // generic map fallback below.
                        for &prev_di in &added_dis {
                            let prev_di_tv = nvim_dictitem_di_tv(prev_di);
                            nvim_tv_set_type(prev_di_tv, VAR_SPECIAL);
                            nvim_tv_set_special(prev_di_tv, K_SPECIAL_VAR_NULL);
                        }
                        nvim_dict_item_free(di);
                        tv_clear(result);
                        had_dup = true;
                        break;
                    }
                    let val_ptr = items_ptr.add((i * 2 + 1) * TV_SIZE);
                    ptr::copy_nonoverlapping(val_ptr, di_tv as *mut u8, TV_SIZE);
                    added_dis.push(di);
                }

                if !had_dup {
                    for i in 0..n {
                        let key_tv: &[u8; TV_SIZE] =
                            &*(items_ptr.add(i * 2 * TV_SIZE) as *const [u8; TV_SIZE]);
                        xfree(tv_get_vptr(key_tv));
                    }
                    xfree(node.data[1].p);
                    node.data[1].p = ptr::null_mut();
                    built_dict = true;
                }
            }

            if !built_dict {
                let list = rs_decode_create_map_special_dict(result, n as isize);
                for i in 0..n {
                    let kv_pair = tv_list_alloc(2);
                    tv_list_append_list(list, kv_pair);
                    nvim_tv_list_append_typval_ptr(kv_pair, items_ptr.add(i * 2 * TV_SIZE).cast());
                    nvim_tv_list_append_typval_ptr(
                        kv_pair,
                        items_ptr.add((i * 2 + 1) * TV_SIZE).cast(),
                    );
                }
                xfree(node.data[1].p);
                node.data[1].p = ptr::null_mut();
            }
        }
        _ => {}
    }
}

#[unsafe(export_name = "mpack_parse_typval")]
pub unsafe extern "C" fn rs_mpack_parse_typval(
    parser: *mut MpackParser,
    data: *mut *const c_char,
    size: *mut usize,
) -> c_int {
    mpack_parse(parser, data, size, typval_parse_enter, typval_parse_exit)
}

#[unsafe(export_name = "typval_parser_error_free")]
pub unsafe extern "C" fn rs_typval_parser_error_free(parser: *mut MpackParser) {
    let size = nvim_mpack_parser_size(parser);
    for i in 0..size {
        let node = &mut *nvim_mpack_parser_item(parser, i);
        match node.tok.tok_type {
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT | MPACK_TOKEN_MAP => {
                if !node.data[1].p.is_null() {
                    xfree(node.data[1].p);
                    node.data[1].p = ptr::null_mut();
                }
            }
            _ => {}
        }
    }
}

#[unsafe(export_name = "unpack_typval")]
pub unsafe extern "C" fn rs_unpack_typval(
    data: *mut *const c_char,
    size: *mut usize,
    ret: TypevalHandle,
) -> c_int {
    nvim_tv_set_type(ret, VAR_UNKNOWN);
    let parser_size = nvim_mpack_parser_alloc_size();
    let parser = xmalloc(parser_size) as *mut MpackParser;
    mpack_parser_init(parser, 0);
    *nvim_mpack_parser_data_ptr(parser) = ret;
    let status = rs_mpack_parse_typval(parser, data, size);
    if status != MPACK_OK {
        rs_typval_parser_error_free(parser);
        tv_clear(ret);
    }
    xfree(parser as *mut c_void);
    status
}
