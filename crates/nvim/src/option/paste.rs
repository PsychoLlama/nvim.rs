//! 'paste', and everything it switches off.
//!
//! The option is a bundle: while it is on, every option that would reformat
//! or re-indent inserted text is forced off, globally and in every buffer.
//! Switching it off puts back what was there — which is why the saved copies
//! are only taken on the transition *into* 'paste'. Setting it again while
//! it is already on must not overwrite them with the suppressed values.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::drawscreen::status_redraw_all;
use crate::global_cell::GlobalCell;
use crate::indent::tabstop_set;
use crate::main::{p_ai, p_et, p_paste, p_ri, p_ru, p_sm, p_sta, p_sts, p_tw, p_vsts, p_wm};
use crate::memory::{xfree, xstrdup};
use crate::options::{
    kOptAutoindent, kOptExpandtab, kOptRevins, kOptRuler, kOptShowmatch, kOptSmarttab,
    kOptSofttabstop, kOptTextwidth, kOptVarsofttabstop, kOptWrapmargin,
};
use crate::optionstr::{empty_option, free_string_option, is_empty_option};
use crate::types::{OptIndex, OptInt, OptionSetFlags, colnr_T, optset_T};

use crate::types::buf_T;
use crate::winlayer::buffers;

use super::{didset_options_sctx, field_ptr};

/// What 'paste' overrode, so that switching it off again restores the
/// values the user set. The per-buffer copies live in `buf_T`; these are
/// the global ones.
pub(crate) static p_ai_nopaste: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_et_nopaste: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_sts_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_tw_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wm_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_vsts_nopaste: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// The options 'paste' overrides while it is on, and so re-attributes to
/// whatever script set 'paste'.
const PASTE_DEP_OPTS: [OptIndex; 10] = [
    kOptAutoindent,
    kOptExpandtab,
    kOptRuler,
    kOptShowmatch,
    kOptSmarttab,
    kOptSofttabstop,
    kOptTextwidth,
    kOptWrapmargin,
    kOptRevins,
    kOptVarsofttabstop,
];

/// 'paste': switch off everything that would reformat pasted text, and
/// remember what to switch back on.
/// Where a buffer keeps its parsed 'varsofttabstop' stops.
const VSTS_ARRAY: usize = core::mem::offset_of!(buf_T, b_p_vsts_array);

pub(crate) unsafe fn did_set_paste(_args: &mut optset_T) -> Option<&CStr> {
    static old_p_paste: GlobalCell<c_int> = GlobalCell::new(0);
    static save_sm: GlobalCell<c_int> = GlobalCell::new(0);
    static save_sta: GlobalCell<c_int> = GlobalCell::new(0);
    static save_ru: GlobalCell<c_int> = GlobalCell::new(0);
    static save_ri: GlobalCell<c_int> = GlobalCell::new(0);

    // SAFETY: the buffer list is the editor's own, and every string handled
    // here is either the shared empty string or an allocation this option
    // owns.
    if p_paste.get() != 0 {
        if old_p_paste.get() == 0 {
            for mut buf in buffers() {
                buf.b_p_tw_nopaste = buf.b_p_tw;
                buf.b_p_wm_nopaste = buf.b_p_wm;
                buf.b_p_sts_nopaste = buf.b_p_sts;
                buf.b_p_ai_nopaste = buf.b_p_ai;
                buf.b_p_et_nopaste = buf.b_p_et;
                if !buf.b_p_vsts_nopaste.is_null() {
                    unsafe { xfree(buf.b_p_vsts_nopaste.cast::<c_void>()) };
                }
                buf.b_p_vsts_nopaste = unsafe { saved_copy(buf.b_p_vsts) };
            }
            save_sm.set(p_sm.get());
            save_sta.set(p_sta.get());
            save_ru.set(p_ru.get());
            save_ri.set(p_ri.get());
            p_ai_nopaste.set(p_ai.get());
            p_et_nopaste.set(p_et.get());
            p_sts_nopaste.set(p_sts.get());
            p_tw_nopaste.set(p_tw.get());
            p_wm_nopaste.set(p_wm.get());
            if !p_vsts_nopaste.get().is_null() {
                unsafe { xfree(p_vsts_nopaste.get().cast::<c_void>()) };
            }
            p_vsts_nopaste.set(unsafe { saved_copy(p_vsts.get()) });
        }

        for mut buf in buffers() {
            buf.b_p_tw = 0;
            buf.b_p_wm = 0;
            buf.b_p_sts = 0;
            buf.b_p_ai = 0;
            buf.b_p_et = 0;
            if !buf.b_p_vsts.is_null() {
                unsafe { free_string_option(buf.b_p_vsts) };
            }
            buf.b_p_vsts = empty_option();
            unsafe { xfree(buf.b_p_vsts_array.cast::<c_void>()) };
            buf.b_p_vsts_array = ptr::null_mut();
        }
        p_sm.set(0);
        p_sta.set(0);
        if p_ru.get() != 0 {
            unsafe { status_redraw_all() };
        }
        p_ru.set(0);
        p_ri.set(0);
        p_tw.set(0);
        p_wm.set(0);
        p_sts.set(0);
        p_ai.set(0);
        p_et.set(0);
        if !p_vsts.get().is_null() {
            unsafe { free_string_option(p_vsts.get()) };
        }
        p_vsts.set(empty_option());
    } else if old_p_paste.get() != 0 {
        for mut buf in buffers() {
            buf.b_p_tw = buf.b_p_tw_nopaste;
            buf.b_p_wm = buf.b_p_wm_nopaste;
            buf.b_p_sts = buf.b_p_sts_nopaste;
            buf.b_p_ai = buf.b_p_ai_nopaste;
            buf.b_p_et = buf.b_p_et_nopaste;
            if !buf.b_p_vsts.is_null() {
                unsafe { free_string_option(buf.b_p_vsts) };
            }
            buf.b_p_vsts = unsafe { restored_copy(buf.b_p_vsts_nopaste) };
            unsafe { xfree(buf.b_p_vsts_array.cast::<c_void>()) };
            if !buf.b_p_vsts.is_null() && !is_empty_option(buf.b_p_vsts) {
                // The array's address is the buffer's plus a constant, so
                // naming it reads nothing.
                let array = field_ptr(buf.raw(), VSTS_ARRAY, |b: &buf_T| &b.b_p_vsts_array);
                unsafe { tabstop_set(buf.b_p_vsts, array) };
            } else {
                buf.b_p_vsts_array = ptr::null_mut::<colnr_T>();
            }
        }
        p_sm.set(save_sm.get());
        p_sta.set(save_sta.get());
        if p_ru.get() != save_ru.get() {
            unsafe { status_redraw_all() };
        }
        p_ru.set(save_ru.get());
        p_ri.set(save_ri.get());
        p_ai.set(p_ai_nopaste.get());
        p_et.set(p_et_nopaste.get());
        p_sts.set(p_sts_nopaste.get());
        p_tw.set(p_tw_nopaste.get());
        p_wm.set(p_wm_nopaste.get());
        if !p_vsts.get().is_null() {
            unsafe { free_string_option(p_vsts.get()) };
        }
        p_vsts.set(unsafe { restored_copy(p_vsts_nopaste.get()) });
    }
    old_p_paste.set(p_paste.get());
    didset_options_sctx(
        OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL,
        &PASTE_DEP_OPTS,
    );
    None
}

/// What 'paste' stashes for a 'varsofttabstop' value: null for a value that
/// was not set, so the restore knows to put the shared empty string back.
///
/// # Safety
///
/// `value` must be a string option's value.
unsafe fn saved_copy(value: *mut c_char) -> *mut c_char {
    if value.is_null() || is_empty_option(value) {
        return ptr::null_mut();
    }
    // SAFETY: the caller's `value` is a NUL-terminated option value.
    unsafe { xstrdup(value) }
}

/// The inverse of [`saved_copy`].
///
/// # Safety
///
/// `saved` must be what [`saved_copy`] returned.
unsafe fn restored_copy(saved: *mut c_char) -> *mut c_char {
    if saved.is_null() {
        return empty_option();
    }
    // SAFETY: the caller's `saved` is a NUL-terminated allocation.
    unsafe { xstrdup(saved) }
}
