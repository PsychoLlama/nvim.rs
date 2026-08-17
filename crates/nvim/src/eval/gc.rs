//! Heads of the intrusive lists that let the VimL garbage collector reach
//! every live list and dict, however deeply they are nested.
//!
//! `tv_list_alloc`/`tv_dict_alloc` push onto these; `tv_list_free`/
//! `tv_dict_free` unlink. `garbage_collect` (eval.rs) walks them to find
//! the objects nothing references any more.

// Not `forbid(unsafe_code)`: that lint rejects the name-mangling override
// below, and the symbol it exports is pinned by a functional spec.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::global_cell::GlobalCell;
use crate::types::{dict_T, list_T};

/// Most recently allocated dict.
pub static gc_first_dict: GlobalCell<*mut dict_T> =
    GlobalCell::new(::core::ptr::null_mut::<dict_T>());

/// Most recently allocated list. Exported because
/// `test/functional/core/job_spec.lua` reads it through the LuaJIT FFI to
/// prove a list handed to a job callback is freed again.
#[unsafe(no_mangle)]
pub static gc_first_list: GlobalCell<*mut list_T> =
    GlobalCell::new(::core::ptr::null_mut::<list_T>());
