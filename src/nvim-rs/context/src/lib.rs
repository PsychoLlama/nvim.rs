//! Context stack handling for Neovim
//!
//! Provides Rust implementations of context stack functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::borrow_as_ptr)]
#![allow(unsafe_code)]

use std::ffi::c_char;

mod convert;
pub mod ffi;
mod restore;
mod save;
mod stack;

pub use convert::*;
pub use restore::*;
pub use save::*;
pub use stack::*;

/// String type from the Neovim API
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

/// Object type enum
pub const K_OBJECT_TYPE_NIL: i32 = 0;
pub const K_OBJECT_TYPE_BOOLEAN: i32 = 1;
pub const K_OBJECT_TYPE_INTEGER: i32 = 2;
pub const K_OBJECT_TYPE_FLOAT: i32 = 3;
pub const K_OBJECT_TYPE_STRING: i32 = 4;
pub const K_OBJECT_TYPE_ARRAY: i32 = 5;
pub const K_OBJECT_TYPE_DICT: i32 = 6;
pub const K_OBJECT_TYPE_LUAREF: i32 = 7;

/// Object data union
#[repr(C)]
#[derive(Copy, Clone)]
pub union ObjectData {
    pub boolean: bool,
    pub integer: i64,
    pub floating: f64,
    pub string: NvimString,
    pub array: Array,
    pub dict: Dict,
    pub luaref: i32,
}

/// Object type
#[repr(C)]
pub struct Object {
    pub obj_type: i32,
    pub data: ObjectData,
}

/// Key-value pair for Dict
#[repr(C)]
pub struct KeyValuePair {
    pub key: NvimString,
    pub value: Object,
}

/// Array type
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Array {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut Object,
}

/// Dict type
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dict {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut KeyValuePair,
}

/// Context struct matching C definition
#[repr(C)]
pub struct Context {
    pub regs: NvimString,
    pub jumps: NvimString,
    pub bufs: NvimString,
    pub gvars: NvimString,
    pub funcs: Array,
}

/// Returns the size of the context stack.
#[no_mangle]
pub unsafe extern "C" fn rs_ctx_size() -> usize {
    ffi::nvim_get_ctx_stack_size()
}

/// Returns pointer to Context object with given zero-based index from the top
/// of context stack or NULL if index is out of bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_ctx_get(index: usize) -> *mut Context {
    ffi::nvim_get_ctx_at_index(index)
}

/// Free resources used by a Context object.
#[no_mangle]
pub unsafe extern "C" fn rs_ctx_free(ctx: *mut Context) {
    if ctx.is_null() {
        return;
    }

    let ctx = &mut *ctx;

    ffi::rs_api_free_string(std::ptr::read(&ctx.regs));
    ffi::rs_api_free_string(std::ptr::read(&ctx.jumps));
    ffi::rs_api_free_string(std::ptr::read(&ctx.bufs));
    ffi::rs_api_free_string(std::ptr::read(&ctx.gvars));

    ffi::rs_api_free_array(std::ptr::read(&ctx.funcs));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_type_constants() {
        assert_eq!(K_OBJECT_TYPE_NIL, 0);
        assert_eq!(K_OBJECT_TYPE_BOOLEAN, 1);
        assert_eq!(K_OBJECT_TYPE_INTEGER, 2);
        assert_eq!(K_OBJECT_TYPE_FLOAT, 3);
        assert_eq!(K_OBJECT_TYPE_STRING, 4);
        assert_eq!(K_OBJECT_TYPE_ARRAY, 5);
        assert_eq!(K_OBJECT_TYPE_DICT, 6);
        assert_eq!(K_OBJECT_TYPE_LUAREF, 7);
    }

    #[test]
    fn test_object_type_sequential() {
        let types = [
            K_OBJECT_TYPE_NIL,
            K_OBJECT_TYPE_BOOLEAN,
            K_OBJECT_TYPE_INTEGER,
            K_OBJECT_TYPE_FLOAT,
            K_OBJECT_TYPE_STRING,
            K_OBJECT_TYPE_ARRAY,
            K_OBJECT_TYPE_DICT,
            K_OBJECT_TYPE_LUAREF,
        ];
        for (i, &t) in types.iter().enumerate() {
            assert_eq!(t, i32::try_from(i).unwrap());
        }
    }

    #[test]
    fn test_nvim_string_size() {
        assert_eq!(std::mem::size_of::<NvimString>(), 16);
    }

    #[test]
    fn test_array_size() {
        assert_eq!(std::mem::size_of::<Array>(), 24);
    }

    #[test]
    fn test_dict_size() {
        assert_eq!(std::mem::size_of::<Dict>(), 24);
    }

    #[test]
    fn test_nvim_string_default() {
        let s = NvimString {
            data: std::ptr::null_mut(),
            size: 0,
        };
        assert!(s.data.is_null());
        assert_eq!(s.size, 0);
    }

    #[test]
    fn test_array_default() {
        let a = Array {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
        assert_eq!(a.size, 0);
        assert_eq!(a.capacity, 0);
        assert!(a.items.is_null());
    }

    #[test]
    fn test_object_type_distinct() {
        let types = [
            K_OBJECT_TYPE_NIL,
            K_OBJECT_TYPE_BOOLEAN,
            K_OBJECT_TYPE_INTEGER,
            K_OBJECT_TYPE_FLOAT,
            K_OBJECT_TYPE_STRING,
            K_OBJECT_TYPE_ARRAY,
            K_OBJECT_TYPE_DICT,
            K_OBJECT_TYPE_LUAREF,
        ];
        for (i, &type_a) in types.iter().enumerate() {
            for (j, &type_b) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(type_a, type_b);
                }
            }
        }
    }

    #[test]
    fn test_dict_default() {
        let d = Dict {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
        assert_eq!(d.size, 0);
        assert_eq!(d.capacity, 0);
        assert!(d.items.is_null());
    }

    #[test]
    fn test_context_struct_size() {
        assert_eq!(std::mem::size_of::<Context>(), 88);
    }

    #[test]
    fn test_context_field_offsets() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Context, regs), 0);
        assert_eq!(offset_of!(Context, jumps), 16);
        assert_eq!(offset_of!(Context, bufs), 32);
        assert_eq!(offset_of!(Context, gvars), 48);
        assert_eq!(offset_of!(Context, funcs), 64);
    }
}
