//! MessagePack serialization for Neovim
//!
//! This crate provides MessagePack packing functions that match the C implementations
//! in msgpack_rpc/packer.c. These are low-level primitives that write directly to
//! a byte buffer.
//!
//! # MessagePack Format
//!
//! MessagePack is a binary serialization format. Key format bytes:
//! - 0x00-0x7f: positive fixint
//! - 0x80-0x8f: fixmap
//! - 0x90-0x9f: fixarray
//! - 0xa0-0xbf: fixstr
//! - 0xc0: nil
//! - 0xc2: false
//! - 0xc3: true
//! - 0xc4-0xc6: bin 8/16/32
//! - 0xc7-0xc9: ext 8/16/32
//! - 0xca: float 32
//! - 0xcb: float 64
//! - 0xcc-0xcf: uint 8/16/32/64
//! - 0xd0-0xd3: int 8/16/32/64
//! - 0xd4-0xd8: fixext 1/2/4/8/16
//! - 0xd9-0xdb: str 8/16/32
//! - 0xdc-0xdd: array 16/32
//! - 0xde-0xdf: map 16/32
//! - 0xe0-0xff: negative fixint

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::branches_sharing_code)]

use std::ffi::{c_char, c_int};

/// Write a single byte to the buffer and advance the pointer.
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 1 byte remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_w(ptr: *mut *mut u8, byte: u8) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    **ptr = byte;
    *ptr = (*ptr).add(1);
}

/// Write a 2-byte big-endian value to the buffer.
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 2 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_w2(ptr: *mut *mut u8, val: u32) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    let bytes = (val as u16).to_be_bytes();
    **ptr = bytes[0];
    *ptr = (*ptr).add(1);
    **ptr = bytes[1];
    *ptr = (*ptr).add(1);
}

/// Write a 4-byte big-endian value to the buffer.
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 4 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_w4(ptr: *mut *mut u8, val: u32) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    let bytes = val.to_be_bytes();
    for byte in bytes {
        **ptr = byte;
        *ptr = (*ptr).add(1);
    }
}

/// Write an 8-byte big-endian value to the buffer.
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 8 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_w8(ptr: *mut *mut u8, data: *const u8) {
    if ptr.is_null() || (*ptr).is_null() || data.is_null() {
        return;
    }
    // Write in big-endian order (reverse on little-endian systems)
    #[cfg(target_endian = "little")]
    {
        for i in (0..8).rev() {
            **ptr = *data.add(i);
            *ptr = (*ptr).add(1);
        }
    }
    #[cfg(target_endian = "big")]
    {
        for i in 0..8 {
            **ptr = *data.add(i);
            *ptr = (*ptr).add(1);
        }
    }
}

/// Pack nil value.
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 1 byte remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_nil(ptr: *mut *mut u8) {
    rs_mpack_w(ptr, 0xc0);
}

/// Pack a boolean value.
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 1 byte remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_bool(ptr: *mut *mut u8, val: c_int) {
    rs_mpack_w(ptr, if val != 0 { 0xc3 } else { 0xc2 });
}

/// Pack an unsigned integer (up to 32 bits).
///
/// Uses the most compact representation:
/// - 0x00-0x7f: positive fixint (1 byte)
/// - 0xcc: uint 8 (2 bytes)
/// - 0xcd: uint 16 (3 bytes)
/// - 0xce: uint 32 (5 bytes)
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 5 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_uint(ptr: *mut *mut u8, val: u32) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if val <= 0x7f {
        rs_mpack_w(ptr, val as u8);
    } else if val <= 0xff {
        rs_mpack_w(ptr, 0xcc);
        rs_mpack_w(ptr, val as u8);
    } else if val <= 0xffff {
        rs_mpack_w(ptr, 0xcd);
        rs_mpack_w2(ptr, val);
    } else {
        rs_mpack_w(ptr, 0xce);
        rs_mpack_w4(ptr, val);
    }
}

/// Pack an unsigned 64-bit integer.
///
/// Uses the most compact representation:
/// - Delegates to `mpack_uint` for values <= 0x0fff_ffff
/// - 0xcf: uint 64 (9 bytes) for larger values
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 9 bytes remaining.
#[unsafe(export_name = "mpack_uint64")]
pub unsafe extern "C" fn rs_mpack_uint64(ptr: *mut *mut c_char, val: u64) {
    let ptr = ptr.cast::<*mut u8>();
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if val > 0x0fff_ffff {
        rs_mpack_w(ptr, 0xcf);
        rs_mpack_w8(ptr, (&raw const val).cast::<u8>());
    } else {
        rs_mpack_uint(ptr, val as u32);
    }
}

/// Pack a signed integer.
///
/// Uses the most compact representation:
/// - For non-negative: delegates to `mpack_uint64`
/// - 0xe0-0xff: negative fixint (-32 to -1)
/// - 0xd0: int 8
/// - 0xd1: int 16
/// - 0xd2: int 32
/// - 0xd3: int 64
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 9 bytes remaining.
#[unsafe(export_name = "mpack_integer")]
pub unsafe extern "C" fn rs_mpack_integer(ptr: *mut *mut c_char, val: i64) {
    let ptr = ptr.cast::<*mut u8>();
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if val >= 0 {
        rs_mpack_uint64(ptr.cast(), val as u64);
    } else if val < -0x8000_0000 {
        // int 64
        rs_mpack_w(ptr, 0xd3);
        rs_mpack_w8(ptr, (&raw const val).cast::<u8>());
    } else if val < -0x8000 {
        // int 32
        rs_mpack_w(ptr, 0xd2);
        rs_mpack_w4(ptr, val as u32);
    } else if val < -0x80 {
        // int 16
        rs_mpack_w(ptr, 0xd1);
        rs_mpack_w2(ptr, val as u32);
    } else if val < -0x20 {
        // int 8
        rs_mpack_w(ptr, 0xd0);
        rs_mpack_w(ptr, val as u8);
    } else {
        // negative fixint
        rs_mpack_w(ptr, val as u8);
    }
}

/// Pack a 64-bit float (double).
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 9 bytes remaining.
#[unsafe(export_name = "mpack_float8")]
pub unsafe extern "C" fn rs_mpack_float8(ptr: *mut *mut c_char, val: f64) {
    let ptr = ptr.cast::<*mut u8>();
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    rs_mpack_w(ptr, 0xcb);
    rs_mpack_w8(ptr, (&raw const val).cast::<u8>());
}

/// Pack an array header (the number of elements that follow).
///
/// - 0x90-0x9f: fixarray (up to 15 elements)
/// - 0xdc: array 16 (up to 65535 elements)
/// - 0xdd: array 32 (up to 2^32-1 elements)
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 5 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_array(ptr: *mut *mut u8, size: u32) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if size < 16 {
        rs_mpack_w(ptr, 0x90 | (size as u8));
    } else if size <= 0xffff {
        rs_mpack_w(ptr, 0xdc);
        rs_mpack_w2(ptr, size);
    } else {
        rs_mpack_w(ptr, 0xdd);
        rs_mpack_w4(ptr, size);
    }
}

/// Pack a map header (the number of key-value pairs that follow).
///
/// - 0x80-0x8f: fixmap (up to 15 pairs)
/// - 0xde: map 16 (up to 65535 pairs)
/// - 0xdf: map 32 (up to 2^32-1 pairs)
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 5 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_map(ptr: *mut *mut u8, size: u32) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if size < 16 {
        rs_mpack_w(ptr, 0x80 | (size as u8));
    } else if size <= 0xffff {
        rs_mpack_w(ptr, 0xde);
        rs_mpack_w2(ptr, size);
    } else {
        rs_mpack_w(ptr, 0xdf);
        rs_mpack_w4(ptr, size);
    }
}

/// Pack a string header (the length of the string data that follows).
///
/// Note: This only writes the header. The actual string bytes must be written separately.
///
/// - 0xa0-0xbf: fixstr (up to 31 bytes)
/// - 0xd9: str 8 (up to 255 bytes)
/// - 0xda: str 16 (up to 65535 bytes)
/// - 0xdb: str 32 (up to 2^32-1 bytes)
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 5 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_str_header(ptr: *mut *mut u8, len: usize) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if len < 32 {
        rs_mpack_w(ptr, 0xa0 | (len as u8));
    } else if len < 0xff {
        rs_mpack_w(ptr, 0xd9);
        rs_mpack_w(ptr, len as u8);
    } else if len < 0xffff {
        rs_mpack_w(ptr, 0xda);
        rs_mpack_w2(ptr, len as u32);
    } else if len < 0xffff_ffff {
        rs_mpack_w(ptr, 0xdb);
        rs_mpack_w4(ptr, len as u32);
    }
    // Larger strings abort in C - we silently do nothing for safety
}

/// Pack a binary data header (the length of the binary data that follows).
///
/// Note: This only writes the header. The actual binary bytes must be written separately.
///
/// - 0xc4: bin 8 (up to 255 bytes)
/// - 0xc5: bin 16 (up to 65535 bytes)
/// - 0xc6: bin 32 (up to 2^32-1 bytes)
///
/// # Safety
///
/// `ptr` must point to a valid mutable pointer to a buffer with at least 5 bytes remaining.
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_bin_header(ptr: *mut *mut u8, len: usize) {
    if ptr.is_null() || (*ptr).is_null() {
        return;
    }
    if len < 0xff {
        rs_mpack_w(ptr, 0xc4);
        rs_mpack_w(ptr, len as u8);
    } else if len < 0xffff {
        rs_mpack_w(ptr, 0xc5);
        rs_mpack_w2(ptr, len as u32);
    } else if len < 0xffff_ffff {
        rs_mpack_w(ptr, 0xc6);
        rs_mpack_w4(ptr, len as u32);
    }
    // Larger binaries abort in C - we silently do nothing for safety
}

// =============================================================================
// PackerBuffer-aware functions
// =============================================================================

/// Max possible length: bytecode + 8 int/float bytes
/// Ext objects are maximum 8=3+5 (nested uint32 payload)
pub const MPACK_ITEM_SIZE: usize = 9;

/// PackerBuffer structure matching C packer_buffer_t
///
/// This is an opaque handle - we use C accessors to interact with it.
#[repr(C)]
pub struct PackerBuffer {
    _opaque: [u8; 0],
}

/// Callback type for buffer flush
pub type PackerBufferFlush = Option<unsafe extern "C" fn(*mut PackerBuffer)>;

// C accessor functions for PackerBuffer
extern "C" {
    fn nvim_packer_get_ptr(packer: *mut PackerBuffer) -> *mut u8;
    fn nvim_packer_set_ptr(packer: *mut PackerBuffer, ptr: *mut u8);
    fn nvim_packer_get_endptr(packer: *mut PackerBuffer) -> *mut u8;
    fn nvim_packer_flush(packer: *mut PackerBuffer);
}

/// Get remaining space in the packer buffer.
///
/// # Safety
///
/// `packer` must be a valid PackerBuffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_remaining(packer: *mut PackerBuffer) -> usize {
    if packer.is_null() {
        return 0;
    }
    let ptr = nvim_packer_get_ptr(packer);
    let endptr = nvim_packer_get_endptr(packer);
    if ptr.is_null() || endptr.is_null() {
        return 0;
    }
    (endptr as usize).saturating_sub(ptr as usize)
}

/// Check if buffer needs flushing and flush if necessary.
///
/// Ensures at least 2 * MPACK_ITEM_SIZE bytes are available.
///
/// # Safety
///
/// `packer` must be a valid PackerBuffer pointer
#[unsafe(export_name = "mpack_check_buffer")]
pub unsafe extern "C" fn rs_mpack_check_buffer(packer: *mut PackerBuffer) {
    if packer.is_null() {
        return;
    }
    if rs_mpack_remaining(packer) < 2 * MPACK_ITEM_SIZE {
        nvim_packer_flush(packer);
    }
}

/// Pack raw bytes into the buffer, handling flush as needed.
///
/// This copies data in chunks, flushing the buffer when full.
///
/// # Safety
///
/// - `data` must be valid for `len` bytes
/// - `packer` must be a valid PackerBuffer pointer
#[unsafe(export_name = "mpack_raw")]
pub unsafe extern "C" fn rs_mpack_raw(data: *const c_char, len: usize, packer: *mut PackerBuffer) {
    let data = data.cast::<u8>();
    if packer.is_null() || (data.is_null() && len > 0) {
        return;
    }

    let mut pos: usize = 0;
    while pos < len {
        let ptr = nvim_packer_get_ptr(packer);
        let endptr = nvim_packer_get_endptr(packer);
        let remaining = (endptr as usize).saturating_sub(ptr as usize);
        let to_copy = std::cmp::min(len - pos, remaining);

        if to_copy > 0 {
            std::ptr::copy_nonoverlapping(data.add(pos), ptr.cast(), to_copy);
            nvim_packer_set_ptr(packer, ptr.add(to_copy));
        }
        pos += to_copy;

        if pos < len {
            nvim_packer_flush(packer);
        }
    }
    rs_mpack_check_buffer(packer);
}

/// Pack a string (header + data) into the buffer.
///
/// # Safety
///
/// - `data` must be valid for `len` bytes
/// - `packer` must be a valid PackerBuffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_str(data: *const u8, len: usize, packer: *mut PackerBuffer) {
    if packer.is_null() {
        return;
    }

    // Write header
    let mut ptr = nvim_packer_get_ptr(packer);
    if len < 32 {
        rs_mpack_w(&mut ptr, 0xa0 | (len as u8));
    } else if len < 0xff {
        rs_mpack_w(&mut ptr, 0xd9);
        rs_mpack_w(&mut ptr, len as u8);
    } else if len < 0xffff {
        rs_mpack_w(&mut ptr, 0xda);
        rs_mpack_w2(&mut ptr, len as u32);
    } else if len < 0xffff_ffff {
        rs_mpack_w(&mut ptr, 0xdb);
        rs_mpack_w4(&mut ptr, len as u32);
    } else {
        // C would abort() here - we just return without writing
        return;
    }
    nvim_packer_set_ptr(packer, ptr);

    // Write data
    rs_mpack_raw(data.cast(), len, packer);
}

/// Pack binary data (header + data) into the buffer.
///
/// # Safety
///
/// - `data` must be valid for `len` bytes
/// - `packer` must be a valid PackerBuffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_mpack_bin(data: *const u8, len: usize, packer: *mut PackerBuffer) {
    if packer.is_null() {
        return;
    }

    // Write header
    let mut ptr = nvim_packer_get_ptr(packer);
    if len < 0xff {
        rs_mpack_w(&mut ptr, 0xc4);
        rs_mpack_w(&mut ptr, len as u8);
    } else if len < 0xffff {
        rs_mpack_w(&mut ptr, 0xc5);
        rs_mpack_w2(&mut ptr, len as u32);
    } else if len < 0xffff_ffff {
        rs_mpack_w(&mut ptr, 0xc6);
        rs_mpack_w4(&mut ptr, len as u32);
    } else {
        // C would abort() here - we just return without writing
        return;
    }
    nvim_packer_set_ptr(packer, ptr);

    // Write data
    rs_mpack_raw(data.cast(), len, packer);
}

/// Pack an extension type (header + type byte + data) into the buffer.
///
/// # Safety
///
/// - `buf` must be valid for `len` bytes
/// - `packer` must be a valid PackerBuffer pointer
#[unsafe(export_name = "mpack_ext")]
pub unsafe extern "C" fn rs_mpack_ext(
    buf: *const c_char,
    len: usize,
    ext_type: i8,
    packer: *mut PackerBuffer,
) {
    if packer.is_null() {
        return;
    }

    // Write header based on length
    let mut ptr = nvim_packer_get_ptr(packer);
    if len == 1 {
        rs_mpack_w(&mut ptr, 0xd4);
    } else if len == 2 {
        rs_mpack_w(&mut ptr, 0xd5);
    } else if len <= 0xff {
        rs_mpack_w(&mut ptr, 0xc7);
        rs_mpack_w(&mut ptr, len as u8);
    } else if len < 0xffff {
        rs_mpack_w(&mut ptr, 0xc8);
        rs_mpack_w2(&mut ptr, len as u32);
    } else if len < 0xffff_ffff {
        rs_mpack_w(&mut ptr, 0xc9);
        rs_mpack_w4(&mut ptr, len as u32);
    } else {
        // C would abort() here
        return;
    }
    rs_mpack_w(&mut ptr, ext_type as u8);
    nvim_packer_set_ptr(packer, ptr);

    // Write data
    rs_mpack_raw(buf, len, packer);
}

/// Shift value for EXT object types (kObjectTypeBuffer = 8)
/// Enum values: Nil=0, Boolean=1, Integer=2, Float=3, String=4, Array=5, Dict=6, LuaRef=7, Buffer=8
pub const EXT_OBJECT_TYPE_SHIFT: i32 = 8;

/// Pack a Neovim handle (Buffer/Window/Tabpage) as a msgpack extension.
///
/// The handle is packed as a fixext or ext8 depending on the value range.
/// Small handles in range [-31, 127] use fixext 1, larger handles use ext8
/// with a variable-length unsigned int encoding.
///
/// # Safety
///
/// `packer` must be a valid PackerBuffer pointer
#[unsafe(export_name = "mpack_handle")]
pub unsafe extern "C" fn rs_mpack_handle(
    object_type: c_int,
    handle: c_int,
    packer: *mut PackerBuffer,
) {
    if packer.is_null() {
        return;
    }

    let ext_type = (object_type - EXT_OBJECT_TYPE_SHIFT) as u8;
    let mut ptr = nvim_packer_get_ptr(packer);

    if (-0x1f..=0x7f).contains(&handle) {
        // fixext 1: small handles fit in a single byte
        rs_mpack_w(&mut ptr, 0xd4);
        rs_mpack_w(&mut ptr, ext_type);
        rs_mpack_w(&mut ptr, handle as u8);
        nvim_packer_set_ptr(packer, ptr);
    } else {
        // Larger handles need ext8 with uint encoding
        // Pack the handle as uint into a temporary buffer
        let mut buf = [0u8; MPACK_ITEM_SIZE];
        let mut pos = buf.as_mut_ptr();
        rs_mpack_uint(&mut pos, handle as u32);
        let packsize = (pos as usize) - (buf.as_ptr() as usize);

        // Write ext8 header
        rs_mpack_w(&mut ptr, 0xc7);
        rs_mpack_w(&mut ptr, packsize as u8);
        rs_mpack_w(&mut ptr, ext_type);

        // Copy packed uint data
        std::ptr::copy_nonoverlapping(buf.as_ptr(), ptr, packsize);
        ptr = ptr.add(packsize);
        nvim_packer_set_ptr(packer, ptr);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn pack_to_vec<F>(f: F) -> Vec<u8>
    where
        F: FnOnce(*mut *mut u8),
    {
        let mut buf = vec![0u8; 16];
        let mut ptr = buf.as_mut_ptr();
        f(&mut ptr);
        let len = (ptr as usize) - (buf.as_ptr() as usize);
        buf.truncate(len);
        buf
    }

    #[test]
    fn test_mpack_nil() {
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_nil(ptr) });
        assert_eq!(result, vec![0xc0]);
    }

    #[test]
    fn test_mpack_bool() {
        let result_true = pack_to_vec(|ptr| unsafe { rs_mpack_bool(ptr, 1) });
        assert_eq!(result_true, vec![0xc3]);

        let result_false = pack_to_vec(|ptr| unsafe { rs_mpack_bool(ptr, 0) });
        assert_eq!(result_false, vec![0xc2]);
    }

    #[test]
    fn test_mpack_uint_fixint() {
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_uint(ptr, 42) });
        assert_eq!(result, vec![42]);
    }

    #[test]
    fn test_mpack_uint_8() {
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_uint(ptr, 200) });
        assert_eq!(result, vec![0xcc, 200]);
    }

    #[test]
    fn test_mpack_uint_16() {
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_uint(ptr, 1000) });
        assert_eq!(result, vec![0xcd, 0x03, 0xe8]);
    }

    #[test]
    fn test_mpack_uint_32() {
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_uint(ptr, 100_000) });
        assert_eq!(result, vec![0xce, 0x00, 0x01, 0x86, 0xa0]);
    }

    #[test]
    fn test_mpack_integer_positive() {
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_integer(ptr.cast(), 42) });
        assert_eq!(result, vec![42]);
    }

    #[test]
    fn test_mpack_integer_negative_fixint() {
        // -1 is 0xff as negative fixint
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_integer(ptr.cast(), -1) });
        assert_eq!(result, vec![0xff]);

        // -31 is 0xe1 as negative fixint
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_integer(ptr.cast(), -31) });
        assert_eq!(result, vec![0xe1]);
    }

    #[test]
    fn test_mpack_integer_int8() {
        // -33 requires int 8
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_integer(ptr.cast(), -100) });
        assert_eq!(result, vec![0xd0, 0x9c]); // 0x9c = -100 as i8
    }

    #[test]
    fn test_mpack_array() {
        // fixarray
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_array(ptr, 5) });
        assert_eq!(result, vec![0x95]);

        // array 16
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_array(ptr, 100) });
        assert_eq!(result, vec![0xdc, 0x00, 0x64]);
    }

    #[test]
    fn test_mpack_map() {
        // fixmap
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_map(ptr, 3) });
        assert_eq!(result, vec![0x83]);

        // map 16
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_map(ptr, 20) });
        assert_eq!(result, vec![0xde, 0x00, 0x14]);
    }

    #[test]
    fn test_mpack_str_header() {
        // fixstr
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_str_header(ptr, 10) });
        assert_eq!(result, vec![0xaa]);

        // str 8
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_str_header(ptr, 100) });
        assert_eq!(result, vec![0xd9, 100]);
    }

    #[test]
    fn test_mpack_bin_header() {
        // bin 8
        let result = pack_to_vec(|ptr| unsafe { rs_mpack_bin_header(ptr, 50) });
        assert_eq!(result, vec![0xc4, 50]);
    }
}
