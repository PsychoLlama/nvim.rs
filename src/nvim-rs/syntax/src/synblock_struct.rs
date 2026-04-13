//! Repr(C) mirror of `synblock_T` (syntax block attached to buffer/window).
//!
//! Field offsets are validated at compile time by `_Static_assert` checks in
//! `src/nvim/syntax_struct_check.c`.
//!
//! # Safety
//!
//! `SynBlockStruct` is `#[repr(C)]` and must match the C `synblock_T` layout
//! exactly. Any mismatch causes undefined behavior (SIGSEGV). The layout is
//! verified by `_Static_assert` at C compile time.

use std::ffi::{c_char, c_int, c_uint, c_void};
use std::mem::size_of;

use nvim_collections::garray::GArray;
use nvim_collections::hashtab::HashTab;

use crate::ffi_types::SynTime;
use crate::synstate_struct::SynStateStruct;
use crate::types::SynBlockHandle;

/// Repr(C) mirror of `synblock_T` (1168 bytes, 44 fields).
///
/// Offsets validated by `_Static_assert` in `syntax_struct_check.c`.
///
/// # Safety
///
/// Must be cast from a valid `synblock_T *` pointer only.
#[repr(C)]
pub struct SynBlockStruct {
    /// offset   0: b_keywtab (hashtab_T, 296 bytes)
    pub b_keywtab: HashTab,
    /// offset 296: b_keywtab_ic (hashtab_T, 296 bytes)
    pub b_keywtab_ic: HashTab,
    /// offset 592: b_syn_error (bool)
    pub b_syn_error: bool,
    /// offset 593: b_syn_slow (bool)
    pub b_syn_slow: bool,
    /// offset 594: 2 bytes padding to align b_syn_ic (int) at offset 596
    pub _pad594: [u8; 2],
    /// offset 596: b_syn_ic (int)
    pub b_syn_ic: c_int,
    /// offset 600: b_syn_foldlevel (int)
    pub b_syn_foldlevel: c_int,
    /// offset 604: b_syn_spell (int)
    pub b_syn_spell: c_int,
    /// offset 608: b_syn_patterns (garray_T, 24 bytes)
    pub b_syn_patterns: GArray,
    /// offset 632: b_syn_clusters (garray_T, 24 bytes)
    pub b_syn_clusters: GArray,
    /// offset 656: b_spell_cluster_id (int)
    pub b_spell_cluster_id: c_int,
    /// offset 660: b_nospell_cluster_id (int)
    pub b_nospell_cluster_id: c_int,
    /// offset 664: b_syn_containedin (int)
    pub b_syn_containedin: c_int,
    /// offset 668: b_syn_sync_flags (int)
    pub b_syn_sync_flags: c_int,
    /// offset 672: b_syn_sync_id (int16_t)
    pub b_syn_sync_id: i16,
    /// offset 674: 2 bytes padding before b_syn_sync_minlines (i32) at 676
    pub _pad674: [u8; 2],
    /// offset 676: b_syn_sync_minlines (linenr_T = i32)
    pub b_syn_sync_minlines: i32,
    /// offset 680: b_syn_sync_maxlines (linenr_T)
    pub b_syn_sync_maxlines: i32,
    /// offset 684: b_syn_sync_linebreaks (linenr_T)
    pub b_syn_sync_linebreaks: i32,
    /// offset 688: b_syn_linecont_pat (char*)
    pub b_syn_linecont_pat: *mut c_char,
    /// offset 696: b_syn_linecont_prog (regprog_T*)
    pub b_syn_linecont_prog: *mut c_void,
    /// offset 704: b_syn_linecont_time (syn_time_T, 24 bytes)
    pub b_syn_linecont_time: SynTime,
    /// offset 728: b_syn_linecont_ic (int)
    pub b_syn_linecont_ic: c_int,
    /// offset 732: b_syn_topgrp (int)
    pub b_syn_topgrp: c_int,
    /// offset 736: b_syn_conceal (int)
    pub b_syn_conceal: c_int,
    /// offset 740: b_syn_folditems (int)
    pub b_syn_folditems: c_int,
    /// offset 744: b_sst_array (synstate_T*)
    pub b_sst_array: *mut SynStateStruct,
    /// offset 752: b_sst_len (int)
    pub b_sst_len: c_int,
    /// offset 756: 4 bytes padding before b_sst_first pointer at 760
    pub _pad756: [u8; 4],
    /// offset 760: b_sst_first (synstate_T*)
    pub b_sst_first: *mut SynStateStruct,
    /// offset 768: b_sst_firstfree (synstate_T*)
    pub b_sst_firstfree: *mut SynStateStruct,
    /// offset 776: b_sst_freecount (int)
    pub b_sst_freecount: c_int,
    /// offset 780: b_sst_check_lnum (linenr_T = i32)
    pub b_sst_check_lnum: i32,
    /// offset 784: b_sst_lasttick (disptick_T = uint64_t)
    pub b_sst_lasttick: u64,
    /// offset 792: b_langp (garray_T, 24 bytes)
    pub b_langp: GArray,
    /// offset 816: b_spell_ismw (bool[256])
    pub b_spell_ismw: [u8; 256],
    /// offset 1072: b_spell_ismw_mb (char*)
    pub b_spell_ismw_mb: *mut c_char,
    /// offset 1080: b_p_spc (char*)
    pub b_p_spc: *mut c_char,
    /// offset 1088: b_cap_prog (regprog_T*)
    pub b_cap_prog: *mut c_void,
    /// offset 1096: b_p_spf (char*)
    pub b_p_spf: *mut c_char,
    /// offset 1104: b_p_spl (char*)
    pub b_p_spl: *mut c_char,
    /// offset 1112: b_p_spo (char*)
    pub b_p_spo: *mut c_char,
    /// offset 1120: b_p_spo_flags (unsigned)
    pub b_p_spo_flags: c_uint,
    /// offset 1124: b_cjk (int)
    pub b_cjk: c_int,
    /// offset 1128: b_syn_chartab (uint8_t[32])
    pub b_syn_chartab: [u8; 32],
    /// offset 1160: b_syn_isk (char*)
    pub b_syn_isk: *mut c_char,
}

// Compile-time size assertion
const _: () = {
    assert!(
        size_of::<SynBlockStruct>() == 1168,
        "SynBlockStruct size mismatch: expected 1168 bytes"
    );
};

/// Cast a `SynBlockHandle` to a shared reference.
///
/// # Safety
///
/// `handle` must be a valid, non-null pointer to a `synblock_T` that will
/// remain valid for the lifetime `'a`. The caller must ensure no mutable
/// aliasing occurs.
#[inline]
pub(crate) unsafe fn synblock_ref<'a>(handle: SynBlockHandle) -> &'a SynBlockStruct {
    unsafe { &*(handle.0.cast::<SynBlockStruct>()) }
}

/// Cast a `SynBlockHandle` to a mutable reference.
///
/// # Safety
///
/// `handle` must be a valid, non-null pointer to a `synblock_T` that will
/// remain valid for the lifetime `'a`. The caller must ensure exclusive
/// access during the borrow.
#[inline]
pub(crate) unsafe fn synblock_mut<'a>(handle: SynBlockHandle) -> &'a mut SynBlockStruct {
    unsafe { &mut *(handle.0.cast::<SynBlockStruct>()) }
}
