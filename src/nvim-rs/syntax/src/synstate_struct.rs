//! Repr(C) mirror of `synstate_T` (syntax state at start of a line).
//!
//! Field offsets are validated at compile time by `_Static_assert` checks in
//! `src/nvim/syntax_struct_check.c`.
//!
//! # Safety
//!
//! `SynStateStruct` is `#[repr(C)]` and must match the C `synstate_T` layout
//! exactly. Any mismatch causes undefined behavior (SIGSEGV). The layout is
//! verified by `_Static_assert` at C compile time.

use std::ffi::c_int;
use std::mem::size_of;

use nvim_collections::garray::GArray;

use crate::ffi_types::BufState;
use crate::types::{SynStateHandle, SST_FIX_STATES};

/// Repr(C) mirror of `synstate_T` (216 bytes).
///
/// Layout:
///   offset   0: sst_next (*mut SynStateStruct, 8 bytes)
///   offset   8: sst_lnum (i32, 4 bytes)
///   offset  12: _pad12 (4 bytes, alignment before union)
///   offset  16: sst_union ([u8; 168]) - union of sst_stack[7] or sst_ga
///   offset 184: sst_next_flags (c_int, 4 bytes)
///   offset 188: sst_stacksize (c_int, 4 bytes)
///   offset 192: sst_next_list (*mut i16, 8 bytes)
///   offset 200: sst_tick (u64 = disptick_T, 8 bytes)
///   offset 208: sst_change_lnum (i32, 4 bytes)
///   offset 212: _pad212 (4 bytes to reach 216)
///
/// Offsets validated by `_Static_assert` in `syntax_struct_check.c`.
#[repr(C)]
pub struct SynStateStruct {
    /// offset   0: sst_next (synstate_T*)
    pub sst_next: *mut SynStateStruct,
    /// offset   8: sst_lnum (linenr_T = i32)
    pub sst_lnum: i32,
    /// offset  12: 4 bytes padding before the union at 16
    pub _pad12: [u8; 4],
    /// offset  16: sst_union (union, 168 bytes -- bufstate_T[7] or garray_T)
    pub sst_union: [u8; 168],
    /// offset 184: sst_next_flags (int)
    pub sst_next_flags: c_int,
    /// offset 188: sst_stacksize (int)
    pub sst_stacksize: c_int,
    /// offset 192: sst_next_list (int16_t*)
    pub sst_next_list: *mut i16,
    /// offset 200: sst_tick (disptick_T = uint64_t)
    pub sst_tick: u64,
    /// offset 208: sst_change_lnum (linenr_T = i32)
    pub sst_change_lnum: i32,
    /// offset 212: 4 bytes padding to reach total of 216
    pub _pad212: [u8; 4],
}

// Compile-time size assertion
const _: () = {
    assert!(
        size_of::<SynStateStruct>() == 216,
        "SynStateStruct size mismatch: expected 216 bytes"
    );
};

impl SynStateStruct {
    /// Get the short state stack (valid when `sst_stacksize <= SST_FIX_STATES`).
    ///
    /// # Safety
    ///
    /// The caller must ensure `sst_stacksize <= SST_FIX_STATES` (7).
    #[inline]
    pub unsafe fn sst_stack_ptr(&self) -> *mut BufState {
        self.sst_union.as_ptr().cast::<BufState>().cast_mut()
    }

    /// Get a reference to the garray (valid when `sst_stacksize > SST_FIX_STATES`).
    ///
    /// # Safety
    ///
    /// The caller must ensure `sst_stacksize > SST_FIX_STATES` (7).
    #[inline]
    pub unsafe fn sst_ga(&self) -> &GArray {
        unsafe { &*(self.sst_union.as_ptr().cast::<GArray>()) }
    }

    /// Get a mutable reference to the garray (valid when `sst_stacksize > SST_FIX_STATES`).
    ///
    /// # Safety
    ///
    /// The caller must ensure `sst_stacksize > SST_FIX_STATES` (7).
    #[inline]
    pub unsafe fn sst_ga_mut(&mut self) -> &mut GArray {
        unsafe { &mut *(self.sst_union.as_mut_ptr().cast::<GArray>()) }
    }

    /// Get a pointer to the bufstate at `idx` in the state stack.
    ///
    /// Handles both the short (inline array) and long (garray) variants.
    ///
    /// # Safety
    ///
    /// `idx` must be `>= 0` and `< sst_stacksize`. The caller must verify
    /// this before calling.
    #[inline]
    pub unsafe fn bufstate_at(&self, idx: c_int) -> *mut BufState {
        if self.sst_stacksize > SST_FIX_STATES {
            let ga = unsafe { self.sst_ga() };
            let data = ga.ga_data as *mut BufState;
            unsafe { data.add(idx as usize) }
        } else {
            unsafe { self.sst_stack_ptr().add(idx as usize) }
        }
    }
}

/// Cast a `SynStateHandle` to a shared reference.
///
/// # Safety
///
/// `handle` must be a valid, non-null pointer to a `synstate_T` that will
/// remain valid for the lifetime `'a`. The caller must ensure no mutable
/// aliasing occurs.
#[inline]
pub(crate) unsafe fn synstate_ref<'a>(handle: SynStateHandle) -> &'a SynStateStruct {
    unsafe { &*(handle.0.cast::<SynStateStruct>()) }
}

/// Cast a `SynStateHandle` to a mutable reference.
///
/// # Safety
///
/// `handle` must be a valid, non-null pointer to a `synstate_T` that will
/// remain valid for the lifetime `'a`. The caller must ensure exclusive
/// access during the borrow.
#[inline]
pub(crate) unsafe fn synstate_mut<'a>(handle: SynStateHandle) -> &'a mut SynStateStruct {
    unsafe { &mut *(handle.0.cast::<SynStateStruct>()) }
}
