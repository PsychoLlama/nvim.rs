//! Re-checking options after something outside `:set` changed them.
//!
//! Startup, `:source`, a shada file or an autocommand can all leave an
//! option holding something no `did_set_*` callback ever saw. These are the
//! sweeps that put that right, plus the two flag lookups the sandbox rests
//! on ([`insecure_flag`], [`was_set_insecurely`]) and the redraw dispatch
//! every set goes through.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::api::extmark::nvim_create_namespace;
use crate::buffer::maketitle;
use crate::charset::init_chartab;
use crate::decoration_provider::get_decor_provider;
use crate::drawscreen::{
    UPD_NOT_VALID, redraw_all_later, redraw_buf_later, redraw_later, status_redraw_all,
};
use crate::ex_getln::{check_opt_wim, did_set_cedit};
use crate::global_cell::GlobalCell;
use crate::highlight::{HlAttrFlags, ns_hl_def};
use crate::highlight_group::{highlight_changed, syn_check_group};
use crate::indent::tabstop_set;
use crate::main::{
    curbuf, curwin, need_maketitle, p_bin, p_et, p_ml, p_tw, p_wm, redraw_tabline, starting,
};
use crate::memory::{xfree, xstrchrnul};
use crate::r#move::changed_window_setting;
use crate::options::*;
use crate::optionstr::{
    check_string_option, did_set_breakat, didset_string_options, empty_option, set_chars_option,
};
use crate::os::cshim::strchr;
use crate::spell::{compile_cap_prog, did_set_spell_option};
use crate::spellfile::spell_check_msm;
use crate::spellsuggest::spell_check_sps;
use crate::types::{
    DecorProvider, HlAttrs, NS, OptIndex, OptInt, OptionSetFlags, String_0, buf_T, optset_T,
    size_t, uint32_t, win_T,
};
use crate::winlayer::Win;

use super::{
    HLATTRS_INIT, NO_SCREEN, didset_options_sctx, didset_window_options, get_option, get_varp,
    kFillchars, kListchars, kOptFlagHLOnly, kOptFlagInsecure, kOptFlagRedrAll, kOptFlagRedrBuf,
    kOptFlagRedrStat, kOptFlagRedrTabl, kOptFlagRedrWin, kOptValTypeString, option_has_type,
    option_is_insecure, set_option_insecure,
};

/// What 'binary' overrode, so that switching it off again restores the
/// values the user set rather than the defaults. The per-buffer copies live
/// in `buf_T`; these are the global ones.
pub(crate) static p_tw_nobin: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_wm_nobin: GlobalCell<OptInt> = GlobalCell::new(0);
pub(crate) static p_ml_nobin: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static p_et_nobin: GlobalCell<c_int> = GlobalCell::new(0);

/// The options 'binary' overrides while it is on, and so re-attributes to
/// whatever script set 'binary'.
const BIN_DEP_OPTS: [OptIndex; 4] = [kOptTextwidth, kOptWrapmargin, kOptModeline, kOptExpandtab];

/// Rebuild the window title, unless the screen has not started yet.
pub(crate) fn did_set_title() {
    if starting.get() != NO_SCREEN {
        // SAFETY: the screen is up, so the buffers it names are live.
        unsafe { maketitle() };
    }
}

/// Apply what turning 'binary' on and off does to the four options it
/// overrides. Their pre-'binary' values are stashed so that turning it off
/// again restores them rather than the defaults.
pub(crate) fn set_options_bin(oldval: bool, newval: bool, opt_flags: OptionSetFlags) {
    let local = !opt_flags.has(OptionSetFlags::GLOBAL);
    let global = !opt_flags.has(OptionSetFlags::LOCAL);
    // SAFETY: `curbuf` is live.
    let buf = curbuf.get();
    if newval {
        if !oldval {
            if local {
                unsafe { (*buf).b_p_tw_nobin = (*buf).b_p_tw };
                unsafe { (*buf).b_p_wm_nobin = (*buf).b_p_wm };
                unsafe { (*buf).b_p_ml_nobin = (*buf).b_p_ml };
                unsafe { (*buf).b_p_et_nobin = (*buf).b_p_et };
            }
            if global {
                p_tw_nobin.set(p_tw.get());
                p_wm_nobin.set(p_wm.get());
                p_ml_nobin.set(p_ml.get());
                p_et_nobin.set(p_et.get());
            }
        }
        if local {
            unsafe { (*buf).b_p_tw = 0 };
            unsafe { (*buf).b_p_wm = 0 };
            unsafe { (*buf).b_p_ml = 0 };
            unsafe { (*buf).b_p_et = 0 };
        }
        if global {
            p_tw.set(0);
            p_wm.set(0);
            p_ml.set(0);
            p_et.set(0);
            p_bin.set(1);
        }
    } else if oldval {
        if local {
            unsafe { (*buf).b_p_tw = (*buf).b_p_tw_nobin };
            unsafe { (*buf).b_p_wm = (*buf).b_p_wm_nobin };
            unsafe { (*buf).b_p_ml = (*buf).b_p_ml_nobin };
            unsafe { (*buf).b_p_et = (*buf).b_p_et_nobin };
        }
        if global {
            p_tw.set(p_tw_nobin.get());
            p_wm.set(p_wm_nobin.get());
            p_ml.set(p_ml_nobin.get());
            p_et.set(p_et_nobin.get());
        }
    }
    // The four overridden options were not set by the user, so they take
    // 'binary's own script context rather than keeping their old one.
    didset_options_sctx(opt_flags, &BIN_DEP_OPTS);
}

/// The first startup sweep: everything that has to see the options as they
/// finally are, before any window or buffer has been shown.
pub(crate) fn didset_options() {
    // SAFETY: `curwin`/`curbuf` are live by the time this runs.
    unsafe { init_chartab() };
    unsafe { didset_string_options() };
    spell_check_msm();
    unsafe { spell_check_sps() };
    unsafe { compile_cap_prog(cur_win().w_s) };
    unsafe { did_set_spell_option() };
    unsafe { did_set_cedit(ptr::null_mut::<optset_T>()) };
    unsafe { did_set_breakat(ptr::null_mut::<optset_T>()) };
    unsafe { didset_window_options(curwin.get(), true) };
}

/// The second startup sweep: what needs highlight groups, and the option
/// values that are cached as parsed arrays.
pub(crate) fn didset_options2() {
    // SAFETY: `curwin`/`curbuf` are live by the time this runs.
    unsafe { highlight_changed() };
    let win = curwin.get();
    let no_err = ptr::null_mut::<c_char>();
    let fcs = unsafe { (*win).w_onebuf_opt.wo_fcs };
    unsafe { set_chars_option(win, fcs, kFillchars, true, no_err, 0) };
    let lcs = unsafe { (*win).w_onebuf_opt.wo_lcs };
    unsafe { set_chars_option(win, lcs, kListchars, true, no_err, 0) };
    unsafe { check_opt_wim() };
    let buf = curbuf.get();
    unsafe { xfree((*buf).b_p_vsts_array.cast::<c_void>()) };
    unsafe { tabstop_set((*buf).b_p_vsts, &raw mut (*buf).b_p_vsts_array) };
    unsafe { xfree((*buf).b_p_vts_array.cast::<c_void>()) };
    unsafe { tabstop_set((*buf).b_p_vts, &raw mut (*buf).b_p_vts_array) };
}

/// Replace a null string option with the shared empty string, for every
/// option that has a global variable. A `:source`d script can leave one.
pub(crate) fn check_options() {
    // SAFETY: `get_varp` hands back the variable of a string option, which
    // is a `*mut c_char`.
    for opt_idx in kOptAleph..kOptCount {
        if option_has_type(opt_idx, kOptValTypeString) && get_option(opt_idx).var.has_global() {
            unsafe { check_string_option(get_varp(opt_idx).string_var()) };
        }
    }
}

/// Whether the option's current value was set from an untrusted place, so
/// evaluating it has to run in the sandbox.
///
/// # Safety
///
/// `wp` must be live for the options that keep their flag in a window.
pub(crate) unsafe fn was_set_insecurely(
    wp: *mut win_T,
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
) -> bool {
    debug_assert!(opt_idx != kOptInvalid);
    // SAFETY: the caller's window is live.
    unsafe { insecure_flag(wp, opt_idx, opt_flags).is_set() }
}

/// Where an option's `kOptFlagInsecure` mark lives.
///
/// The options whose value is evaluated as an expression keep the mark per
/// window or per buffer, because one window may be showing a file whose
/// modeline set it while another is not; everything else shares one mark per
/// option. Upstream answers both with a `uint32_t *` and lets the caller
/// poke it; here the two homes are arms, and reading or writing the mark is
/// [`is_set`](Self::is_set)/[`set`](Self::set).
#[derive(Copy, Clone)]
pub(crate) enum InsecureFlag {
    /// The one mark the option shares, in `crate::option::state`.
    Shared(OptIndex),
    /// A window's or buffer's own flag word.
    Field(*mut uint32_t),
}

impl InsecureFlag {
    /// Whether the mark is raised.
    pub(crate) fn is_set(self) -> bool {
        match self {
            InsecureFlag::Shared(opt_idx) => option_is_insecure(opt_idx),
            // SAFETY: the address came from the live window or buffer the
            // caller passed to `insecure_flag`.
            InsecureFlag::Field(flags) => unsafe { *flags & kOptFlagInsecure != 0 },
        }
    }

    /// Raise or lower the mark.
    pub(crate) fn set(self, insecure: bool) {
        match self {
            InsecureFlag::Shared(opt_idx) => set_option_insecure(opt_idx, insecure),
            // SAFETY: as above.
            InsecureFlag::Field(flags) => {
                // SAFETY: as above.
                let word = unsafe { *flags };
                let word = match insecure {
                    true => word | kOptFlagInsecure,
                    false => word & !kOptFlagInsecure,
                };
                // SAFETY: as above.
                unsafe { *flags = word };
            }
        }
    }
}

/// Which of the two homes above carries this option's mark, for the window
/// the option is about to be used from.
///
/// # Safety
///
/// `wp` must be live for the options that keep their own copy.
pub(crate) unsafe fn insecure_flag(
    wp: *mut win_T,
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
) -> InsecureFlag {
    let own: *mut uint32_t = if opt_flags.has(OptionSetFlags::LOCAL) {
        debug_assert!(!wp.is_null());
        // SAFETY: the caller's window, and its buffer, are live.
        match opt_idx {
            kOptWrap => unsafe { &raw mut (*wp).w_onebuf_opt.wo_wrap_flags },
            kOptStatusline => unsafe { &raw mut (*wp).w_onebuf_opt.wo_stl_flags },
            kOptWinbar => unsafe { &raw mut (*wp).w_onebuf_opt.wo_wbr_flags },
            kOptFoldexpr => unsafe { &raw mut (*wp).w_onebuf_opt.wo_fde_flags },
            kOptFoldtext => unsafe { &raw mut (*wp).w_onebuf_opt.wo_fdt_flags },
            kOptIndentexpr => unsafe { &raw mut (*(*wp).w_buffer).b_p_inde_flags },
            kOptFormatexpr => unsafe { &raw mut (*(*wp).w_buffer).b_p_fex_flags },
            kOptIncludeexpr => unsafe { &raw mut (*(*wp).w_buffer).b_p_inex_flags },
            _ => ptr::null_mut(),
        }
    } else if !wp.is_null() {
        // The global value of a window-local option lives in the window's
        // second `winopt_T`. Upstream dereferences `wp` here without the
        // assert the local branch has; the null test above leaves a caller
        // that passes none on the shared mark instead.
        // SAFETY: the caller's window is live.
        match opt_idx {
            kOptWrap => unsafe { &raw mut (*wp).w_allbuf_opt.wo_wrap_flags },
            kOptFoldexpr => unsafe { &raw mut (*wp).w_allbuf_opt.wo_fde_flags },
            kOptFoldtext => unsafe { &raw mut (*wp).w_allbuf_opt.wo_fdt_flags },
            _ => ptr::null_mut(),
        }
    } else {
        ptr::null_mut()
    };
    match own.is_null() {
        true => InsecureFlag::Shared(opt_idx),
        false => InsecureFlag::Field(own),
    }
}

/// Ask for the window title and the tabline to be rebuilt.
pub(crate) fn redraw_titles() {
    need_maketitle.set(true);
    redraw_tabline.set(true);
}

/// Whether every byte of `val` is a letter, a digit, or one of `allowed` —
/// the test a 'filetype'/'syntax'/'keymap' value has to pass.
///
///
/// `allowed` is ASCII at every call site, so the byte search here is the
/// same answer as upstream's `vim_strchr`, which only differs above 0x7f.
pub(crate) fn valid_name(val: &CStr, allowed: &[u8]) -> bool {
    val.to_bytes()
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || allowed.contains(b))
}

/// Whether the window's grid has to be composed with what is under it —
/// 'winblend', or a float with a shadow.
///
/// # Safety
///
/// `wp` must be live.
pub(crate) unsafe fn check_blending(wp: *mut win_T) {
    // SAFETY: the caller's window is live.
    // SAFETY: the caller's window.
    let mut wp = unsafe { Win::new(wp) };
    wp.w_grid_alloc.blending =
        wp.w_onebuf_opt.wo_winbl > 0 as OptInt || (wp.w_floating && wp.w_config.shadow);
}

/// Parse 'winhighlight' — a comma-separated list of `from:to` group pairs —
/// and, when `wp` is given, install it as that window's highlight namespace.
/// `false` for a value that does not parse; with `wp` given, a failure
/// leaves the namespace half-built, exactly as upstream does.
///
/// # Safety
///
/// `winhl`, when non-null, must be NUL-terminated; `wp`, when non-null, must
/// be live.
pub(crate) unsafe fn parse_winhl_opt(winhl: *const c_char, wp: *mut win_T) -> bool {
    // SAFETY: the caller's string is NUL-terminated and its window is live.
    let mut p: *const c_char = if !winhl.is_null() {
        winhl
    } else if !wp.is_null() {
        unsafe { (*wp).w_onebuf_opt.wo_winhl }
    } else {
        empty_option()
    };

    if unsafe { *p } == 0 {
        // An empty value drops the window's namespace, but only while it
        // is still the one 'winhighlight' made.
        if !wp.is_null()
            && unsafe { (*wp).w_ns_hl_winhl } > 0
            && unsafe { (*wp).w_ns_hl } == unsafe { (*wp).w_ns_hl_winhl }
        {
            unsafe { (*wp).w_ns_hl = 0 };
            unsafe { (*wp).w_hl_needs_update = true };
        }
        return true;
    }

    let mut ns_hl: c_int = 0;
    if !wp.is_null() {
        if unsafe { (*wp).w_ns_hl_winhl } == 0 {
            unsafe { (*wp).w_ns_hl_winhl = nvim_create_namespace(String_0::NULL) as c_int };
        } else {
            // Reusing the namespace: bump the generation so attributes
            // cached against it are re-resolved.
            let dp: *mut DecorProvider =
                unsafe { get_decor_provider((*wp).w_ns_hl_winhl as NS, true) };
            unsafe { (*dp).hl_valid += 1 };
        }
        ns_hl = unsafe { (*wp).w_ns_hl_winhl };
        if unsafe { (*wp).w_ns_hl } <= 0 {
            unsafe { (*wp).w_ns_hl = (*wp).w_ns_hl_winhl };
        }
    }

    while unsafe { *p } != 0 {
        let colon = unsafe { strchr(p, ':' as c_int) };
        if colon.is_null() {
            return false;
        }
        let from_len = unsafe { colon.offset_from(p) } as size_t;
        let to = unsafe { colon.add(1) };
        let comma = unsafe { xstrchrnul(to, ',' as c_char) };
        let to_len = unsafe { comma.offset_from(to) } as size_t;

        // An empty target means "no highlight at all", spelled -1.
        let hl_id = if to_len != 0 {
            unsafe { syn_check_group(to, to_len) }
        } else {
            -1
        };
        if hl_id == 0 {
            return false;
        }
        let hl_id_link = if from_len != 0 {
            unsafe { syn_check_group(p, from_len) }
        } else {
            0
        };
        if hl_id_link == 0 {
            return false;
        }

        if !wp.is_null() {
            let mut attrs: HlAttrs = HLATTRS_INIT;
            attrs.rgb_ae_attr |= HlAttrFlags::GLOBAL;
            unsafe { ns_hl_def(ns_hl as NS, hl_id_link, attrs, hl_id, None) };
        }
        p = if unsafe { *comma } != 0 {
            unsafe { comma.add(1) }
        } else {
            c"".as_ptr()
        };
    }

    if !wp.is_null() {
        unsafe { (*wp).w_hl_needs_update = true };
    }
    true
}

/// Ask for whatever the option's redraw flags say has to be redrawn.
///
/// # Safety
///
/// `buf` and `win` must be live.
pub(crate) unsafe fn check_redraw_for(buf: *mut buf_T, win: *mut win_T, flags: uint32_t) {
    // `kOptFlagRedrAll` is the two window bits together, so test for both.
    let all = flags & kOptFlagRedrAll == kOptFlagRedrAll;
    // SAFETY: the caller's buffer and window are live.
    if flags & kOptFlagRedrStat != 0 || all {
        unsafe { status_redraw_all() };
    }
    if flags & kOptFlagRedrTabl != 0 || all {
        redraw_tabline.set(true);
    }
    if flags & (kOptFlagRedrBuf | kOptFlagRedrWin) != 0 || all {
        if flags & kOptFlagHLOnly != 0 {
            unsafe { redraw_later(win, UPD_NOT_VALID) };
        } else {
            changed_window_setting(unsafe { Win::new(win) });
        }
    }
    if flags & kOptFlagRedrBuf != 0 {
        unsafe { redraw_buf_later(buf, UPD_NOT_VALID) };
    }
    if all {
        unsafe { redraw_all_later(UPD_NOT_VALID) };
    }
}

/// [`check_redraw_for`] for the current buffer and window.
pub(crate) fn check_redraw(flags: uint32_t) {
    // SAFETY: `curbuf`/`curwin` are live.
    unsafe { check_redraw_for(curbuf.get(), curwin.get(), flags) }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
