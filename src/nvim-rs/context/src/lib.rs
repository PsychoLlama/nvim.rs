//! Context stack handling for Neovim
//!
//! Provides Rust implementations of context stack functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::borrow_as_ptr)]
#![allow(unsafe_code)]

use std::ffi::c_char;

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

extern "C" {
    /// Get the size of the context stack
    fn nvim_get_ctx_stack_size() -> usize;

    /// Free a string
    fn rs_api_free_string(value: NvimString);

    /// Free an array
    fn rs_api_free_array(value: Array);
}

/// Returns the size of the context stack.
///
/// # Safety
/// Calls C accessor function for `ctx_stack`.
#[no_mangle]
pub unsafe extern "C" fn rs_ctx_size() -> usize {
    nvim_get_ctx_stack_size()
}

/// Free resources used by a Context object.
///
/// # Safety
/// The context pointer must be valid and the strings/arrays must be
/// properly initialized.
///
/// # Arguments
/// * `ctx` - Pointer to Context object to free
#[no_mangle]
pub unsafe extern "C" fn rs_ctx_free(ctx: *mut Context) {
    if ctx.is_null() {
        return;
    }

    let ctx = &mut *ctx;

    // Free the string fields
    rs_api_free_string(std::ptr::read(&ctx.regs));
    rs_api_free_string(std::ptr::read(&ctx.jumps));
    rs_api_free_string(std::ptr::read(&ctx.bufs));
    rs_api_free_string(std::ptr::read(&ctx.gvars));

    // Free the funcs array
    rs_api_free_array(std::ptr::read(&ctx.funcs));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_type_constants() {
        // Verify ObjectType constants match C definitions
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
        // Ensure types are sequential for indexing
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
        // NvimString should be ptr + size_t = 16 bytes on 64-bit
        assert_eq!(std::mem::size_of::<NvimString>(), 16);
    }

    #[test]
    fn test_array_size() {
        // Array should be 3 * size_t = 24 bytes on 64-bit
        assert_eq!(std::mem::size_of::<Array>(), 24);
    }

    #[test]
    fn test_dict_size() {
        // Dict should be 3 * size_t = 24 bytes on 64-bit
        assert_eq!(std::mem::size_of::<Dict>(), 24);
    }

    #[test]
    fn test_nvim_string_default() {
        // Test that we can create a null NvimString
        let s = NvimString {
            data: std::ptr::null_mut(),
            size: 0,
        };
        assert!(s.data.is_null());
        assert_eq!(s.size, 0);
    }

    #[test]
    fn test_array_default() {
        // Test that we can create an empty Array
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
        // All object types should have distinct values
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
        // Test that we can create an empty Dict
        let d = Dict {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
        assert_eq!(d.size, 0);
        assert_eq!(d.capacity, 0);
        assert!(d.items.is_null());
    }
}
