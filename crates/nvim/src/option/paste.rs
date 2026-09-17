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

use core::ffi::c_void;
use core::ptr;

use crate::drawscreen::status_redraw_all;
use crate::global_cell::GlobalCell;
use crate::indent::tabstop_set;
use crate::memory::{XString, xfree};
use crate::option::vars::{
    P_AI, P_ET, P_RI, P_RU, P_SM, P_STA, P_STS, P_TW, P_VSTS, P_WM, p_ai, p_et, p_paste, p_ri,
    p_ru, p_sm, p_sta, p_sts, p_tw, p_wm,
};
use crate::options::{
    kOptAutoindent, kOptExpandtab, kOptRevins, kOptRuler, kOptShowmatch, kOptSmarttab,
    kOptSofttabstop, kOptTextwidth, kOptVarsofttabstop, kOptWrapmargin,
};
use crate::types::{ColNr, OptError, OptIndex, OptInt, OptSet, OptionSetFlags};

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
    };
}

static SAVED: GlobalCell<PasteSave> = GlobalCell::new(PasteSave::NONE);

/// What 'paste' stashed for 'varsofttabstop'. Its own cell rather than a
/// field of [`PasteSave`]: the record is `Copy` and read out by value a
/// dozen times, and an owned string is neither.
///
/// `None` is "the option was not set", which is what the restore puts back
/// as the option owning nothing.
static SAVED_VSTS: GlobalCell<Option<XString>> = GlobalCell::new(None);

/// What 'paste' has stashed, for the buffer-local copies `copy.rs` seeds
/// from it.
pub(crate) fn paste_save() -> PasteSave {
    SAVED.get()
}

/// A copy of what 'paste' stashed for 'varsofttabstop'.
pub(crate) fn paste_saved_vsts() -> Option<XString> {
    SAVED_VSTS.with(Clone::clone)
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

pub(crate) fn did_set_paste(_args: &mut OptSet) -> Result<(), OptError> {
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
                buf.b_p_vsts_nopaste = buf.b_p_vsts.clone();
            }
            // The stale save goes with the write.
            SAVED_VSTS.set((!P_VSTS.is_unset()).then(|| P_VSTS.get()));
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
            });
        }

        for mut buf in buffers() {
            buf.b_p_tw = 0;
            buf.b_p_wm = 0;
            buf.b_p_sts = 0;
            buf.b_p_ai = 0;
            buf.b_p_et = 0;
            buf.b_p_vsts = None;
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
            buf.b_p_vsts = buf.b_p_vsts_nopaste.clone();
            unsafe { xfree(buf.b_p_vsts_array.cast::<c_void>()) };
            if let Some(vsts) = buf.b_p_vsts.as_ref() {
                // The array's address is the buffer's plus a constant, so
                // naming it reads nothing.
                let array = field_ptr(buf.raw(), VSTS_ARRAY, |b: &Buffer| &b.b_p_vsts_array);
                unsafe { tabstop_set(vsts.as_ptr().cast_mut(), array) };
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
        P_VSTS.restore(paste_saved_vsts());
    }
    SAVED.with_mut(|saved| saved.on = p_paste());
    didset_options_sctx(
        OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL,
        &PASTE_DEP_OPTS,
    );
    Ok(())
}
