//! Re-checking options after something outside `:set` changed them.
//!
//! Startup, `:source`, a shada file or an autocommand can all leave an
//! option holding something no `did_set_*` callback ever saw. These are the
//! sweeps that put that right, plus the two flag lookups the sandbox rests
//! on ([`insecure_flag`], [`was_set_insecurely`]) and the redraw dispatch
//! every set goes through.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::field;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

use crate::api::extmark::nvim_create_namespace;
use crate::buffer::maketitle;
use crate::charset::init_chartab;
use crate::decoration_provider::with_decor_provider;
use crate::drawscreen::state::{need_maketitle, redraw_tabline};
use crate::drawscreen::{
    UPD_NOT_VALID, redraw_all_later, redraw_buf_later, redraw_later, status_redraw_all,
};
use crate::ex_getln::{check_opt_wim, derive_cedit_key};
use crate::global_cell::GlobalCell;
use crate::highlight::{HlAttrFlags, ns_hl_def};
use crate::highlight_group::{highlight_changed, syn_check_group};
use crate::indent::replace_tabstop_array;
use crate::r#move::changed_window_setting;
use crate::option::vars::{P_BIN, P_ET, P_ML, P_TW, P_WM, p_et, p_ml, p_tw, p_wm};
use crate::options::*;
use crate::optionstr::{
    OptString, OptStringRef, derive_breakat_flags, didset_string_options, set_window_chars_option,
};
use crate::spell::{compile_cap_prog, did_set_spell_option};
use crate::spellfile::spell_check_msm;
use crate::spellsuggest::spell_check_sps;
use crate::startup::starting;
use crate::types::{
    Buffer, HlAttrs, NS, OptIndex, OptInt, OptionSetFlags, String_0, WinOpt, uint32_t,
};
use crate::winlayer::Win;

use super::{
    HLATTRS_INIT, Local, NO_SCREEN, WinOptSet, didset_options_sctx, didset_window_options,
    kFillchars, kListchars, kOptFlagHLOnly, kOptFlagInsecure, kOptFlagRedrAll, kOptFlagRedrBuf,
    kOptFlagRedrStat, kOptFlagRedrTabl, kOptFlagRedrWin, option_is_insecure, set_option_insecure,
};

/// What 'binary' overrode, so that switching it off again restores the
/// values the user set rather than the defaults. The per-buffer copies live
/// in `Buffer`; this is the global set, saved and restored as one.
#[derive(Copy, Clone)]
pub(crate) struct BinSave {
    /// 'textwidth' and 'wrapmargin'.
    pub(crate) tw: OptInt,
    pub(crate) wm: OptInt,
    /// 'modeline' and 'expandtab'.
    pub(crate) ml: bool,
    pub(crate) et: bool,
}

static SAVED: GlobalCell<BinSave> = GlobalCell::new(BinSave {
    tw: 0,
    wm: 0,
    ml: false,
    et: false,
});

/// What 'binary' has stashed, for the buffer-local copies `copy.rs` seeds
/// from it.
pub(crate) fn bin_save() -> BinSave {
    SAVED.get()
}

/// The options 'binary' overrides while it is on, and so re-attributes to
/// whatever script set 'binary'.
const BIN_DEP_OPTS: [OptIndex; 4] = [kOptTextwidth, kOptWrapmargin, kOptModeline, kOptExpandtab];

/// Rebuild the window title, unless the screen has not started yet.
pub(crate) fn did_set_title() {
    if starting.get() != NO_SCREEN {
        maketitle();
    }
}

/// Apply what turning 'binary' on and off does to the four options it
/// overrides. Their pre-'binary' values are stashed so that turning it off
/// again restores them rather than the defaults.
pub(crate) fn set_options_bin(oldval: bool, newval: bool, opt_flags: OptionSetFlags) {
    let local = !opt_flags.has(OptionSetFlags::GLOBAL);
    let global = !opt_flags.has(OptionSetFlags::LOCAL);
    let mut buf = Buf::current();
    if newval {
        if !oldval {
            if local {
                buf.b_p_tw_nobin = buf.b_p_tw;
                buf.b_p_wm_nobin = buf.b_p_wm;
                buf.b_p_ml_nobin = buf.b_p_ml;
                buf.b_p_et_nobin = buf.b_p_et;
            }
            if global {
                SAVED.set(BinSave {
                    tw: p_tw(),
                    wm: p_wm(),
                    ml: p_ml(),
                    et: p_et(),
                });
            }
        }
        if local {
            buf.b_p_tw = 0;
            buf.b_p_wm = 0;
            buf.b_p_ml = 0;
            buf.b_p_et = 0;
        }
        if global {
            P_TW.set(0);
            P_WM.set(0);
            P_ML.set(false);
            P_ET.set(false);
            P_BIN.set(true);
        }
    } else if oldval {
        if local {
            buf.b_p_tw = buf.b_p_tw_nobin;
            buf.b_p_wm = buf.b_p_wm_nobin;
            buf.b_p_ml = buf.b_p_ml_nobin;
            buf.b_p_et = buf.b_p_et_nobin;
        }
        if global {
            let saved = SAVED.get();
            P_TW.set(saved.tw);
            P_WM.set(saved.wm);
            P_ML.set(saved.ml);
            P_ET.set(saved.et);
        }
    }
    // The four overridden options were not set by the user, so they take
    // 'binary's own script context rather than keeping their old one.
    didset_options_sctx(opt_flags, &BIN_DEP_OPTS);
}

/// The first startup sweep: everything that has to see the options as they
/// finally are, before any window or buffer has been shown.
pub(crate) fn didset_options() {
    init_chartab();
    didset_string_options();
    let _ = spell_check_msm();
    let _ = spell_check_sps();
    let _ = compile_cap_prog(&mut Win::current().syntax());
    let _ = did_set_spell_option();
    // The two callbacks these stand in for read no frame at all, and the
    // startup sweep has none to give.
    let _ = derive_cedit_key();
    derive_breakat_flags();
    didset_window_options(Win::current(), true);
}

/// The second startup sweep: what needs highlight groups, and the option
/// values that are cached as parsed arrays.
pub(crate) fn didset_options2() {
    highlight_changed();
    let win = Win::current();
    let _ = set_window_chars_option(win, kFillchars, true);
    let _ = set_window_chars_option(win, kListchars, true);
    let _ = check_opt_wim();
    let mut buf = Buf::current();
    let vsts = buf.b_p_vsts.get();
    replace_tabstop_array(&mut buf.b_p_vsts_array, Some(vsts.as_cstr()));
    let vts = buf.b_p_vts.get();
    replace_tabstop_array(&mut buf.b_p_vts_array, Some(vts.as_cstr()));
}

/// Whether the option's current value was set from an untrusted place, so
/// evaluating it has to run in the sandbox.
pub(crate) fn was_set_insecurely(
    window: Win,
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
) -> bool {
    debug_assert!(opt_idx != kOptInvalid);
    insecure_flag(Some(window), opt_idx, opt_flags).is_set()
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
    Field(Local<uint32_t>),
}

impl InsecureFlag {
    /// Whether the mark is raised.
    pub(crate) fn is_set(self) -> bool {
        match self {
            InsecureFlag::Shared(opt_idx) => option_is_insecure(opt_idx),
            InsecureFlag::Field(flags) => flags.get() & kOptFlagInsecure != 0,
        }
    }

    /// Raise or lower the mark.
    pub(crate) fn set(self, insecure: bool) {
        match self {
            InsecureFlag::Shared(opt_idx) => set_option_insecure(opt_idx, insecure),
            InsecureFlag::Field(flags) => {
                let word = flags.get();
                flags.set(match insecure {
                    true => word | kOptFlagInsecure,
                    false => word & !kOptFlagInsecure,
                });
            }
        }
    }
}

/// Which of the two homes above carries this option's mark, for the window
/// the option is about to be used from.
pub(crate) fn insecure_flag(
    window: Option<Win>,
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
) -> InsecureFlag {
    let win_word = |window: Win, set: WinOptSet, field| Some(Local::Win(window, set, field));
    let one = WinOptSet::One;
    let own = if opt_flags.has(OptionSetFlags::LOCAL) {
        let window = window.expect("a local option is set on a window");
        let buf_word = |field| Some(Local::Buf(window.buffer(), field));
        match opt_idx {
            kOptWrap => win_word(window, one, field!(WinOpt, wo_wrap_flags)),
            kOptStatusline => win_word(window, one, field!(WinOpt, wo_stl_flags)),
            kOptWinbar => win_word(window, one, field!(WinOpt, wo_wbr_flags)),
            kOptFoldexpr => win_word(window, one, field!(WinOpt, wo_fde_flags)),
            kOptFoldtext => win_word(window, one, field!(WinOpt, wo_fdt_flags)),
            kOptIndentexpr => buf_word(field!(Buffer, b_p_inde_flags)),
            kOptFormatexpr => buf_word(field!(Buffer, b_p_fex_flags)),
            kOptIncludeexpr => buf_word(field!(Buffer, b_p_inex_flags)),
            _ => None,
        }
    } else if let Some(window) = window {
        // The global value of a window-local option lives in the window's
        // second `WinOpt`. Upstream dereferences `window` here without the
        // assert the local branch has; the `Option` above leaves a caller
        // that passes none on the shared mark instead.
        let all = WinOptSet::All;
        match opt_idx {
            kOptWrap => win_word(window, all, field!(WinOpt, wo_wrap_flags)),
            kOptFoldexpr => win_word(window, all, field!(WinOpt, wo_fde_flags)),
            kOptFoldtext => win_word(window, all, field!(WinOpt, wo_fdt_flags)),
            _ => None,
        }
    } else {
        None
    };
    own.map_or(InsecureFlag::Shared(opt_idx), InsecureFlag::Field)
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
pub(crate) fn check_blending(mut window: Win) {
    window.w_grid_alloc.blending =
        window.w_onebuf_opt.wo_winbl > 0 as OptInt || (window.w_floating && window.w_config.shadow);
}

/// Parse 'winhighlight' — a comma-separated list of `from:to` group pairs —
/// and, when `window` is given, install it as that window's highlight namespace.
/// `winhl` is the value to parse, `None` for the window's own. `false` for a
/// value that does not parse; with `window` given, a failure leaves the
/// namespace half-built, exactly as upstream does.
pub(crate) fn parse_winhl_opt(winhl: Option<&[u8]>, window: Option<Win>) -> bool {
    // A copy of the window's own: defining the groups below runs the
    // highlight code, which may look at the window.
    let own;
    let value: &[u8] = match (winhl, window) {
        (Some(winhl), _) => winhl,
        (None, Some(w)) => {
            own = w.w_onebuf_opt.wo_winhl.bytes().to_vec();
            &own
        }
        (None, None) => b"",
    };

    if value.is_empty() {
        // An empty value drops the window's namespace, but only while it
        // is still the one 'winhighlight' made.
        if let Some(mut w) = window.filter(|w| w.w_ns_hl_winhl > 0 && w.w_ns_hl == w.w_ns_hl_winhl)
        {
            w.w_ns_hl = 0;
            w.w_hl_needs_update = true;
        }
        return true;
    }

    let mut ns_hl: c_int = 0;
    if let Some(mut w) = window {
        if w.w_ns_hl_winhl == 0 {
            let ns = nvim_create_namespace(String_0::NULL);
            w.w_ns_hl_winhl = c_int::try_from(ns).expect("a namespace id fits an int");
        } else {
            // Reusing the namespace: bump the generation so attributes
            // cached against it are re-resolved.
            with_decor_provider(w.w_ns_hl_winhl as NS, true, |dp| dp.hl_valid += 1);
        }
        ns_hl = w.w_ns_hl_winhl;
        if w.w_ns_hl <= 0 {
            w.w_ns_hl = w.w_ns_hl_winhl;
        }
    }

    let mut rest = value;
    while !rest.is_empty() {
        let Some(colon) = rest.iter().position(|&b| b == b':') else {
            return false;
        };
        let from = &rest[..colon];
        let after = &rest[colon + 1..];
        let comma = after.iter().position(|&b| b == b',').unwrap_or(after.len());
        let to = &after[..comma];

        // An empty target means "no highlight at all", spelled -1.
        let hl_id = if to.is_empty() {
            -1
        } else {
            syn_check_group(to)
        };
        if hl_id == 0 {
            return false;
        }
        let hl_id_link = if from.is_empty() {
            0
        } else {
            syn_check_group(from)
        };
        if hl_id_link == 0 {
            return false;
        }

        if window.is_some() {
            let mut attrs: HlAttrs = HLATTRS_INIT;
            attrs.rgb_ae_attr |= HlAttrFlags::GLOBAL;
            ns_hl_def(ns_hl as NS, hl_id_link, attrs, hl_id, None);
        }
        rest = after.get(comma + 1..).unwrap_or_default();
    }

    if let Some(mut window) = window {
        window.w_hl_needs_update = true;
    }
    true
}

/// Ask for whatever the option's redraw flags say has to be redrawn.
pub(crate) fn check_redraw_for(buffer: Buf, win: Win, flags: uint32_t) {
    // `kOptFlagRedrAll` is the two window bits together, so test for both.
    let all = flags & kOptFlagRedrAll == kOptFlagRedrAll;
    if flags & kOptFlagRedrStat != 0 || all {
        status_redraw_all();
    }
    if flags & kOptFlagRedrTabl != 0 || all {
        redraw_tabline.set(true);
    }
    if flags & (kOptFlagRedrBuf | kOptFlagRedrWin) != 0 || all {
        if flags & kOptFlagHLOnly != 0 {
            redraw_later(win, UPD_NOT_VALID);
        } else {
            changed_window_setting(win);
        }
    }
    if flags & kOptFlagRedrBuf != 0 {
        redraw_buf_later(buffer, UPD_NOT_VALID);
    }
    if all {
        redraw_all_later(UPD_NOT_VALID);
    }
}

/// [`check_redraw_for`] for the current buffer and window.
pub(crate) fn check_redraw(flags: uint32_t) {
    check_redraw_for(Buf::current(), Win::current(), flags)
}
