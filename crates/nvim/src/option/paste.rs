//! 'paste', and everything it switches off.
//!
//! The option is a bundle: while it is on, every option that would reformat
//! or re-indent inserted text is forced off, globally and in every buffer.
//! Switching it off puts back what was there — which is why the saved copies
//! are only taken on the transition *into* 'paste'. Setting it again while
//! it is already on must not overwrite them with the suppressed values.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_void};
use core::ptr;

use crate::drawscreen::status_redraw_all;
use crate::global_cell::GlobalCell;
use crate::indent::tabstop_set;
use crate::memory::{XString, xfree, xstrdup};
use crate::option::vars::{
    P_AI, P_ET, P_RI, P_RU, P_SM, P_STA, P_STS, P_TW, P_VSTS, P_WM, p_ai, p_et, p_paste, p_ri,
    p_ru, p_sm, p_sta, p_sts, p_tw, p_vsts, p_wm,
};
use crate::options::{
    kOptAutoindent, kOptExpandtab, kOptRevins, kOptRuler, kOptShowmatch, kOptSmarttab,
    kOptSofttabstop, kOptTextwidth, kOptVarsofttabstop, kOptWrapmargin,
};
use crate::optionstr::{empty_option, free_string_option, is_empty_option};
use crate::types::{ColNr, OptIndex, OptInt, OptSet, OptionSetFlags};

use crate::types::Buffer;
use crate::winlayer::buffers;

use super::{didset_options_sctx, field_ptr};

/// What 'paste' overrode, so that switching it off again restores the
/// values the user set. The per-buffer copies live in `Buffer`; this is the
/// global set.
///
/// Upstream keeps one global per saved option and a tenth for "is 'paste'
/// already on"; they are only ever read and written together, on the two
/// transitions, and a saved value means nothing without `on` to say whether
/// it was ever taken. One record says that.
#[derive(Copy, Clone)]
pub(crate) struct PasteSave {
    /// Whether 'paste' was on last time the callback ran — which is what
    /// makes setting 'paste' twice keep the first save.
    on: bool,
    /// 'showmatch', 'smarttab', 'ruler' and 'revins'.
    sm: bool,
    sta: bool,
    ru: bool,
    ri: bool,
    /// 'autoindent' and 'expandtab'.
    pub(crate) ai: bool,
    pub(crate) et: bool,
    /// 'softtabstop', 'textwidth' and 'wrapmargin'.
    pub(crate) sts: OptInt,
    pub(crate) tw: OptInt,
    pub(crate) wm: OptInt,
    /// 'varsofttabstop', or null for a value that was not set. Owned: the
    /// next save frees it.
    pub(crate) vsts: *mut c_char,
}

impl PasteSave {
    /// Nothing saved yet.
    const NONE: Self = PasteSave {
        on: false,
        sm: false,
        sta: false,
        ru: false,
        ri: false,
        ai: false,
        et: false,
        sts: 0,
        tw: 0,
        wm: 0,
        vsts: ptr::null_mut(),
    };
}

static SAVED: GlobalCell<PasteSave> = GlobalCell::new(PasteSave::NONE);

/// What 'paste' has stashed, for the buffer-local copies `copy.rs` seeds
/// from it.
pub(crate) fn paste_save() -> PasteSave {
    SAVED.get()
}

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
const VSTS_ARRAY: usize = core::mem::offset_of!(Buffer, b_p_vsts_array);

pub(crate) fn did_set_paste(_args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the buffer list is the editor's own, and every string handled
    // here is either the shared empty string or an allocation this option
    // owns.
    if p_paste() {
        if !SAVED.get().on {
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
            let stale = SAVED.get().vsts;
            if !stale.is_null() {
                unsafe { xfree(stale.cast::<c_void>()) };
            }
            SAVED.set(PasteSave {
                on: false,
                sm: p_sm(),
                sta: p_sta(),
                ru: p_ru(),
                ri: p_ri(),
                ai: p_ai(),
                et: p_et(),
                sts: p_sts(),
                tw: p_tw(),
                wm: p_wm(),
                vsts: p_vsts(|value| unsafe { saved_copy(value.as_ptr().cast_mut()) }),
            });
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
        P_SM.set(false);
        P_STA.set(false);
        if p_ru() {
            status_redraw_all();
        }
        P_RU.set(false);
        P_RI.set(false);
        P_TW.set(0);
        P_WM.set(0);
        P_STS.set(0);
        P_AI.set(false);
        P_ET.set(false);
        P_VSTS.clear();
    } else if SAVED.get().on {
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
                let array = field_ptr(buf.raw(), VSTS_ARRAY, |b: &Buffer| &b.b_p_vsts_array);
                unsafe { tabstop_set(buf.b_p_vsts, array) };
            } else {
                buf.b_p_vsts_array = ptr::null_mut::<ColNr>();
            }
        }
        let saved = SAVED.get();
        P_SM.set(saved.sm);
        P_STA.set(saved.sta);
        if p_ru() != saved.ru {
            status_redraw_all();
        }
        P_RU.set(saved.ru);
        P_RI.set(saved.ri);
        P_AI.set(saved.ai);
        P_ET.set(saved.et);
        P_STS.set(saved.sts);
        P_TW.set(saved.tw);
        P_WM.set(saved.wm);
        // SAFETY: the paste record's saved value is null or a live
        // allocation; the option takes a copy of it.
        P_VSTS.restore(unsafe { restored_owned(saved.vsts) });
    }
    SAVED.with_mut(|saved| saved.on = p_paste());
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

/// [`restored_copy`] for a global value, which owns its string: nothing
/// saved is the option owning nothing, which is what the shared empty
/// string stood for.
///
/// # Safety
///
/// As [`restored_copy`].
unsafe fn restored_owned(saved: *mut c_char) -> Option<XString> {
    // SAFETY: the caller's `saved` is null or a NUL-terminated allocation.
    (!saved.is_null()).then(|| XString::from_cstr(unsafe { cstr::at(saved) }))
}
