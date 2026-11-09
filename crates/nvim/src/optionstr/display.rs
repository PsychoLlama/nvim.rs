//! The callbacks for options that decide what the screen looks like.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::field;
use crate::memory::XString;
use crate::option::HIGHLIGHT_INIT;
use crate::strings::has_char;
use crate::types::Handle;
use crate::window::Z_INDEX_FLOAT_DEFAULT;
use crate::winfloat::ALIGN_LEFT;
use crate::winfloat::WIN_SPLIT_LEFT;
use crate::winfloat::WIN_STYLE_UNUSED;
use core::ffi::{CStr, c_int, c_uint};

use crate::api::win_config::parse_winborder;
use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int_at, init_chartab, ptr2cells_at};
use crate::cursor::coladvance;
use crate::cursor_shape::{SHAPE_CURSOR, parse_shape_opt};
use crate::decoration::defs::VirtText;
use crate::drawscreen::{
    UPD_INVERTED, UPD_NOT_VALID, comp_col, redraw_all_later, redraw_curbuf_later, redraw_win_line,
};
use crate::eval::vars::{do_unlet, var_string_value};
use crate::ex_getln::check_opt_wim;
use crate::ex_getln::state::CMDPREVIEW;
use crate::highlight_group::init_highlight;
use crate::indent::briopt_check;
use crate::mbyte::c_cluster_len;
use crate::message::E_UNSUPPORTEDOPTION;
use crate::message::{messagesopt_changed, msg_grid_validate};
use crate::r#move::validate_virtcol;
use crate::option::defs::{OptError, OptSet, OptionSetFlags};
use crate::option::vars::{
    BREAKAT_FLAGS, P_BG, P_MOUSESCROLL, P_MOUSESCROLL_HOR, P_MOUSESCROLL_VERT, VE_FLAGS, p_breakat,
    p_km, p_pumborder, p_winborder,
};
use crate::option::{fill_culopt_flags, parse_winhl_opt};
use crate::options::{kOptAmbiwidth, opt_ve_values};
use crate::optionstr::BreakAt;
use crate::state::mode::{KM_STARTSEL, KM_STOPSEL};
use crate::types::{
    ColNr, FLOAT_RELATIVE_EDITOR, FloatAnchor, LPos, LineNr, OptInt, WinConfig, WinOpt,
};
use crate::window::check_colorcolumn;

use super::frame::{invalid, local_window, new_bytes, new_value, old_bytes, varp, win};
use super::{
    COCU_ALL, E_SHOWBREAK_CONTAINS_UNPRINTABLE_OR_WIDE_CHARACTER, INT_MAX, MOUSESCROLL_HOR_DFLT,
    MOUSESCROLL_VERT_DFLT, WW_ALL, check_chars_options, check_signcolumn, check_str_opt,
    did_set_option_listflag, did_set_statustabline_rulerformat, did_set_str_generic,
    opt_strings_mask, terminal_notify_theme,
};
use crate::decoration::SCL_NUM;
use crate::normal::visual_active;
use crate::winlayer::{Win, buffers};

/// 'ambiwidth' decides how wide an ambiguous-width character is drawn, so
/// the two character options have to be re-checked against the new answer.
pub fn did_set_ambiwidth(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
    check_chars_options()
}

/// 'emoji' has the same reach as 'ambiwidth', so it re-checks the same
/// things — including 'ambiwidth' itself, whose mask depends on it.
pub fn did_set_emoji(_args: &mut OptSet) -> Result<(), OptError> {
    if check_str_opt(kOptAmbiwidth, None).is_err() {
        return invalid();
    }
    check_chars_options()
}

/// Reload the highlight groups for a new 'background'.
///
/// The colour scheme may set 'background' back from under us while it is
/// being reloaded; when it does, the scheme is disowned (`g:colors_name`
/// unset), the value the user asked for is restored, and the highlighting
/// is built once more from the built-in defaults.
pub fn did_set_background(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
    // Only the first byte distinguishes "dark" from "light".
    if old_bytes(args).first().copied().unwrap_or(0) == P_BG.first_byte() {
        return Ok(());
    }

    let dark = P_BG.first_byte() == b'd';
    init_highlight(false, false);

    // The global may have been changed by `init_highlight`.
    if dark != (P_BG.first_byte() == b'd') && var_string_value(b"g:colors_name").is_some() {
        let name = c"g:colors_name";
        // SAFETY: the name is a C string of the length given.
        let _ = do_unlet(name.to_bytes(), true);
        P_BG.set(XString::from_cstr(if dark { c"dark" } else { c"light" }));
        init_highlight(false, false);
    }

    // Terminal buffers pick their palette from the background.
    for buf in buffers() {
        terminal_notify_theme(buf, dark);
    }
    Ok(())
}

/// 'breakat' is consulted per character while wrapping, so it is kept as a
/// 256-entry lookup rather than re-scanned.
pub fn did_set_breakat(_args: &mut OptSet) -> Result<(), OptError> {
    derive_breakat_flags();
    Ok(())
}

/// The `'breakat'` character set itself, for the startup sweep, which has
/// no option frame to hand a callback.
pub(crate) fn derive_breakat_flags() {
    let mut chars = BreakAt::NONE;
    p_breakat(|value| {
        for &byte in value.to_bytes() {
            chars.insert(byte);
        }
    });
    BREAKAT_FLAGS.set(chars);
}

pub fn did_set_breakindentopt(args: &mut OptSet) -> Result<(), OptError> {
    let (wp, varp) = (win(args), varp(args));
    let local = field!(WinOpt, wo_briopt);
    let for_window = local_window(varp, wp, local);
    if !briopt_check(Some(new_value(args).as_cstr()), for_window) {
        return invalid();
    }
    // A window whose 'breakindentopt' asks for list indenting affects how
    // every other window's shared buffer wraps.
    if for_window.is_some() && wp.w_briopt_list != 0 {
        redraw_all_later(UPD_NOT_VALID);
    }
    Ok(())
}

pub fn did_set_colorcolumn(args: &mut OptSet) -> Result<(), OptError> {
    let (wp, varp) = (win(args), varp(args));
    let local = field!(WinOpt, wo_cc);
    check_colorcolumn(
        Some(new_value(args).as_cstr()),
        local_window(varp, wp, local),
    )
}

pub fn did_set_concealcursor(args: &mut OptSet) -> Result<(), OptError> {
    did_set_option_listflag(new_bytes(args), COCU_ALL)
}

pub fn did_set_cursorlineopt(args: &mut OptSet) -> Result<(), OptError> {
    let (wp, value) = (win(args), new_value(args));
    // An empty 'cursorlineopt' is not "no highlighting", it is no answer at
    // all.
    if value.is_empty() || fill_culopt_flags(Some(value.as_cstr()), wp).is_err() {
        return invalid();
    }
    Ok(())
}

pub fn did_set_display(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
    // "uhex" changes how an unprintable character is drawn, and "msgsep"
    // changes whether the message area is its own grid.
    init_chartab();
    msg_grid_validate();
    Ok(())
}

pub fn did_set_guicursor(_args: &mut OptSet) -> Result<(), OptError> {
    parse_shape_opt(SHAPE_CURSOR)?;
    // The Visual-mode cursor shape is drawn as part of the line.
    if visual_active() {
        redraw_win_line(Win::current(), Win::current().w_cursor.lnum);
    }
    Ok(())
}

/// 'highlight' is Vim's highlight-group mapping, which nvim does not
/// implement: only its default value is accepted.
pub fn did_set_highlight(args: &mut OptSet) -> Result<(), OptError> {
    if new_bytes(args) != HIGHLIGHT_INIT.to_bytes() {
        return Err((E_UNSUPPORTEDOPTION).into());
    }
    Ok(())
}

/// 'inccommand' cannot change while a preview is on screen, because the
/// preview was set up under the old value.
pub fn did_set_inccommand(args: &mut OptSet) -> Result<(), OptError> {
    if CMDPREVIEW.get() {
        return invalid();
    }
    did_set_str_generic(args)
}

pub fn did_set_keymodel(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
    KM_STOPSEL.set(p_km(|value| has_char(value, c_int::from(b'o'))));
    KM_STARTSEL.set(p_km(|value| has_char(value, c_int::from(b'a'))));
    Ok(())
}

pub fn did_set_messagesopt(_args: &mut OptSet) -> Result<(), OptError> {
    if messagesopt_changed().is_err() {
        return invalid();
    }
    Ok(())
}

pub fn did_set_mouse(args: &mut OptSet) -> Result<(), OptError> {
    did_set_option_listflag(new_bytes(args), super::MOUSE_ALL)
}

/// 'mousescroll' is `ver:<n>` and/or `hor:<n>`, each at most once. A
/// direction the value does not mention keeps its built-in default rather
/// than whatever the previous value set.
pub fn did_set_mousescroll(_args: &mut OptSet) -> Result<(), OptError> {
    // A copy: the parse below outlives the projection's borrow.
    let mousescroll = P_MOUSESCROLL.get();
    let value = &*mousescroll;
    let mut vertical: Option<OptInt> = None;
    let mut horizontal: Option<OptInt> = None;

    let mut offset = 0;
    for part in value.split(|&b| b == b',') {
        // "ver:" or "hor:" plus at least one digit.
        if part.len() <= 4 {
            return invalid();
        }
        let (prefix, digits) = part.split_at(4);
        let direction = match prefix {
            b"ver:" => &mut vertical,
            b"hor:" => &mut horizontal,
            _ => return invalid(),
        };
        // Naming a direction twice is an error, not a last-one-wins.
        if direction.is_some() {
            return invalid();
        }
        if !digits.iter().all(|b| ascii_isdigit(c_int::from(*b))) {
            return Err((c"E5080: Digit expected").into());
        }
        // The digits are read as `getdigits_int` reads them, which is what
        // rejects a number too large for an `int`.
        let (number, _) = getdigits_int_at(value, offset + 4, false, -1);
        if number == -1 {
            return invalid();
        }
        *direction = Some(OptInt::from(number));
        offset += part.len() + 1;
    }

    P_MOUSESCROLL_VERT.set(vertical.unwrap_or(OptInt::from(MOUSESCROLL_VERT_DFLT)));
    P_MOUSESCROLL_HOR.set(horizontal.unwrap_or(OptInt::from(MOUSESCROLL_HOR_DFLT)));
    Ok(())
}

pub fn did_set_selection(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
    // Whether the character under the cursor is inside the selection just
    // changed.
    if visual_active() {
        redraw_curbuf_later(UPD_INVERTED);
    }
    Ok(())
}

/// Every character of 'showbreak' has to fit in one cell, because the
/// leader is drawn in a fixed-width margin.
pub fn did_set_showbreak(args: &mut OptSet) -> Result<(), OptError> {
    // The walk steps by the length of the character it just measured.
    let mut rest = new_bytes(args);
    while !rest.is_empty() {
        if ptr2cells_at(rest) != 1 {
            return Err((E_SHOWBREAK_CONTAINS_UNPRINTABLE_OR_WIDE_CHARACTER).into());
        }
        rest = &rest[c_cluster_len(rest)..];
    }
    Ok(())
}

pub fn did_set_showcmdloc(args: &mut OptSet) -> Result<(), OptError> {
    let errmsg = did_set_str_generic(args);
    if errmsg.is_ok() {
        // The pending-command display shares the last line with the ruler.
        comp_col();
    }
    errmsg
}

pub fn did_set_signcolumn(args: &mut OptSet) -> Result<(), OptError> {
    let (mut wp, varp) = (win(args), varp(args));
    let local = field!(WinOpt, wo_scl);
    if check_signcolumn(Some(new_bytes(args)), local_window(varp, wp, local)).is_err() {
        return invalid();
    }
    // "number" shares the sign column with the number column, so
    // leaving or entering it invalidates the cached number width.
    if old_bytes(args).starts_with(b"nu") || wp.w_minscwidth == SCL_NUM {
        wp.w_nrwidth_line_count = LineNr::ZERO;
    }
    Ok(())
}

/// 'virtualedit' keeps a mask beside its string, and the window-local one
/// uses an empty value to mean "no override" rather than "no virtual edit".
pub fn did_set_virtualedit(args: &mut OptSet) -> Result<(), OptError> {
    let mut wp = win(args);
    let local = args.os_flags.has(OptionSetFlags::LOCAL);
    // The variable being set is the window's own copy for `:setlocal` and
    // the global one otherwise -- which is the value the frame holds.
    let value = new_bytes(args);
    let mut store = |mask: c_uint| {
        if local {
            wp.w_onebuf_opt.wo_ve_flags = mask;
        } else {
            VE_FLAGS.set(mask);
        }
    };

    if local && value.is_empty() {
        store(0 as c_uint);
        return Ok(());
    }
    let Some(mask) = opt_strings_mask(value, &opt_ve_values, true) else {
        return invalid();
    };
    store(mask);
    if value != old_bytes(args) {
        // What column the cursor may sit in just changed.
        validate_virtcol(wp);
        coladvance(wp, wp.w_virtcol);
    }
    Ok(())
}

pub fn did_set_whichwrap(args: &mut OptSet) -> Result<(), OptError> {
    // 'whichwrap' is spelled as a comma-separated list but checked as a set
    // of letters, so the comma is one of the accepted letters.
    const WW_AND_COMMA: &CStr = c"bshl<>[]~,";
    debug_assert!(WW_AND_COMMA.to_bytes().starts_with(WW_ALL.to_bytes()));
    did_set_option_listflag(new_bytes(args), WW_AND_COMMA)
}

pub fn did_set_wildmode(_args: &mut OptSet) -> Result<(), OptError> {
    if check_opt_wim().is_err() {
        return invalid();
    }
    Ok(())
}

pub fn did_set_winbar(args: &mut OptSet) -> Result<(), OptError> {
    did_set_statustabline_rulerformat(args, false, false)
}

/// Would `border_opt` be accepted as a floating window's border?
///
/// The API's parser is reused, which needs a whole window configuration to
/// write into and an error slot to report through; both are discarded.
pub(crate) fn parse_border_opt(border_opt: &CStr) -> bool {
    let mut fconfig = WinConfig {
        window: Handle::NONE,
        bufpos: LPos {
            lnum: LineNr::new(-1),
            col: ColNr::ZERO,
        },
        height: 0,
        width: 0,
        row: 0.0,
        col: 0.0,
        anchor: 0 as FloatAnchor,
        relative: FLOAT_RELATIVE_EDITOR,
        external: false,
        focusable: true,
        mouse: true,
        split: WIN_SPLIT_LEFT,
        zindex: c_int::try_from(Z_INDEX_FLOAT_DEFAULT).expect("the default z-index is small"),
        style: WIN_STYLE_UNUSED,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: ALIGN_LEFT,
        title_chunks: VirtText::new(),
        title_width: 0,
        footer: false,
        footer_pos: ALIGN_LEFT,
        footer_chunks: VirtText::new(),
        footer_width: 0,
        noautocmd: false,
        fixed: false,
        hide: false,
        _cmdline_offset: INT_MAX,
    };
    // Whatever the call refused with is dropped rather than reported.
    parse_winborder(&mut fconfig, border_opt).unwrap_or(false)
}

pub fn did_set_winborder(_args: &mut OptSet) -> Result<(), OptError> {
    if !p_winborder(parse_border_opt) {
        return invalid();
    }
    Ok(())
}

pub fn did_set_pumborder(_args: &mut OptSet) -> Result<(), OptError> {
    if !p_pumborder(parse_border_opt) {
        return invalid();
    }
    Ok(())
}

pub fn did_set_winhighlight(args: &mut OptSet) -> Result<(), OptError> {
    let (wp, varp) = (win(args), varp(args));
    let local = field!(WinOpt, wo_winhl);
    if !parse_winhl_opt(Some(new_bytes(args)), local_window(varp, wp, local)) {
        return invalid();
    }
    Ok(())
}
