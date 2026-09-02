//! The callbacks for options that decide what the screen looks like.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

use crate::api::win_config::parse_winborder;
use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int, init_chartab, ptr2cells};
use crate::cursor::coladvance;
use crate::cursor_shape::{SHAPE_CURSOR, parse_shape_opt};
use crate::drawscreen::{
    UPD_INVERTED, UPD_NOT_VALID, comp_col, redraw_all_later, redraw_curbuf_later, redraw_win_line,
};
use crate::eval::vars::{do_unlet, get_var_value};
use crate::ex_getln::check_opt_wim;
use crate::highlight_group::init_highlight;
use crate::indent::briopt_check;
use crate::main::{
    breakat_flags, cmdpreview, curwin, e_unsupportedoption, km_startsel, km_stopsel, p_bg,
    p_breakat, p_km, p_mousescroll, p_mousescroll_hor, p_mousescroll_vert, p_pumborder, p_ve,
    p_winborder, ve_flags,
};
use crate::mbyte::utfc_ptr2len;
use crate::memory::xstrdup;
use crate::message::{messagesopt_changed, msg_grid_validate};
use crate::r#move::validate_virtcol;
use crate::option::{answer_err, fill_culopt_flags, parse_winhl_opt};
use crate::options::{kOptAmbiwidth, opt_ve_values};
use crate::strings::vim_strchr;
use crate::types::{
    BreakAt, Error, FAIL, FloatAnchor, NUL, OptInt, OptionSetFlags, VirtText, WinConfig, colnr_T,
    kFloatRelativeEditor, linenr_T, lpos_T, optset_T,
};
use crate::window::check_colorcolumn;

use super::frame::{errbuf, invalid, local_window, old_value, varp, win};
use super::{
    COCU_ALL, HIGHLIGHT_INIT, INT_MAX, MOUSESCROLL_HOR_DFLT, MOUSESCROLL_VERT_DFLT, WW_ALL,
    check_chars_options, check_signcolumn, check_str_opt, did_set_option_listflag,
    did_set_statustabline_rulerformat, did_set_str_generic,
    e_showbreak_contains_unprintable_or_wide_character, empty_option, free_string_option,
    kAlignLeft, kWinSplitLeft, kWinStyleUnused, kZIndexFloatDefault, opt_strings_mask,
    terminal_notify_theme,
};
use crate::decoration::SCL_NUM;
use crate::eval::typval::NumBuf;
use crate::normal::visual_active;
use crate::winlayer::{Win, buffers};

/// 'ambiwidth' decides how wide an ambiguous-width character is drawn, so
/// the two character options have to be re-checked against the new answer.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_ambiwidth(args: &mut optset_T) -> Option<&CStr> {
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    unsafe { check_chars_options() }
}

/// 'emoji' has the same reach as 'ambiwidth', so it re-checks the same
/// things — including 'ambiwidth' itself, whose mask depends on it.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_emoji(_args: &mut optset_T) -> Option<&CStr> {
    if unsafe { check_str_opt(kOptAmbiwidth, ptr::null_mut()) }.is_err() {
        return invalid();
    }
    unsafe { check_chars_options() }
}

/// Reload the highlight groups for a new 'background'.
///
/// The colour scheme may set 'background' back from under us while it is
/// being reloaded; when it does, the scheme is disowned (`g:colors_name`
/// unset), the value the user asked for is restored, and the highlighting
/// is built once more from the built-in defaults.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_background(args: &mut optset_T) -> Option<&CStr> {
    let mut numbuf = NumBuf::new();
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    // SAFETY: both are C strings; only the first byte distinguishes "dark"
    // from "light".
    if unsafe { *old_value(args) == *p_bg.get() } {
        return None;
    }

    let dark = unsafe { *p_bg.get() } == b'd' as c_char;
    // SAFETY: `init_highlight` reads the editor's own state.
    unsafe { init_highlight(false, false) };

    // SAFETY: reading the global that `init_highlight` may have changed,
    // and the editor's own variable dictionary.
    if unsafe {
        dark != (*p_bg.get() == b'd' as c_char)
            && !get_var_value(c"g:colors_name".as_ptr(), &mut numbuf).is_null()
    } {
        let name = c"g:colors_name";
        // SAFETY: the name is a C string of the length given, and `p_bg` is
        // this process's own option variable.
        let _ = unsafe { do_unlet(name.as_ptr(), name.to_bytes().len(), true) };
        unsafe { free_string_option(p_bg.get()) };
        p_bg.set(unsafe {
            xstrdup(if dark {
                c"dark".as_ptr()
            } else {
                c"light".as_ptr()
            })
        });
        // `check_string_option` for a cell: `xstrdup` never answers
        // null, but upstream guards anyway.
        if p_bg.get().is_null() {
            p_bg.set(empty_option());
        }
        unsafe { init_highlight(false, false) };
    }

    // Terminal buffers pick their palette from the background.
    for buf in buffers() {
        if !buf.terminal.is_null() {
            // SAFETY: a live buffer's own terminal.
            unsafe { terminal_notify_theme(buf.terminal, dark) };
        }
    }
    None
}

/// 'breakat' is consulted per character while wrapping, so it is kept as a
/// 256-entry lookup rather than re-scanned.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_breakat(_args: &mut optset_T) -> Option<&'static CStr> {
    // SAFETY: the option's own value is a C string.
    unsafe { derive_breakat_flags() };
    None
}

/// The `'breakat'` character set itself, for the startup sweep, which has
/// no option frame to hand a callback.
///
/// # Safety
/// `'breakat'`'s value is null or a C string.
pub(crate) unsafe fn derive_breakat_flags() {
    let value = p_breakat.get();
    let mut chars = BreakAt::NONE;
    if !value.is_null() {
        // SAFETY: the option's own value is a C string.
        for &byte in unsafe { CStr::from_ptr(value) }.to_bytes() {
            chars.insert(byte);
        }
    }
    breakat_flags.set(chars);
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_breakindentopt(args: &mut optset_T) -> Option<&CStr> {
    let (wp, varp) = (win(args), varp(args));
    // SAFETY: the frame's window.
    let local = unsafe { &raw mut (*wp).w_onebuf_opt.wo_briopt };
    let for_window = unsafe { local_window(varp, wp, local) };
    // SAFETY: the option's value is a C string.
    if unsafe { briopt_check(*varp, for_window) } as c_int == FAIL {
        return invalid();
    }
    // A window whose 'breakindentopt' asks for list indenting affects how
    // every other window's shared buffer wraps.
    if !for_window.is_null() && unsafe { (*wp).w_briopt_list } != 0 {
        // SAFETY: marks the editor's own windows.
        unsafe { redraw_all_later(UPD_NOT_VALID) };
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_colorcolumn(args: &mut optset_T) -> Option<&CStr> {
    let (wp, varp) = (win(args), varp(args));
    // SAFETY: the frame's window, and the option's C string value.
    let local = unsafe { &raw mut (*wp).w_onebuf_opt.wo_cc };
    unsafe { check_colorcolumn(*varp, local_window(varp, wp, local)) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_concealcursor(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), COCU_ALL.as_ptr(), buf, len) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_cursorlineopt(args: &mut optset_T) -> Option<&CStr> {
    let (wp, varp) = (win(args), varp(args));
    // An empty 'cursorlineopt' is not "no highlighting", it is no answer at
    // all.
    // SAFETY: the option's C string value, and the frame's window, which
    // `optset_T` names for exactly this call.
    let win = unsafe { Win::new(wp) };
    if unsafe { c_int::from(**varp) } == NUL
        || unsafe { fill_culopt_flags(Some(CStr::from_ptr(*varp)), win) }.is_err()
    {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_display(args: &mut optset_T) -> Option<&CStr> {
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    // "uhex" changes how an unprintable character is drawn, and "msgsep"
    // changes whether the message area is its own grid.
    // SAFETY: both read the editor's own state.
    unsafe { init_chartab() };
    unsafe { msg_grid_validate() };
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_guicursor(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: reads the option's own value.
    let errmsg = unsafe { parse_shape_opt(SHAPE_CURSOR) };
    if errmsg.is_some() {
        return errmsg;
    }
    // The Visual-mode cursor shape is drawn as part of the line.
    if visual_active() {
        // SAFETY: the current window is live.
        unsafe { redraw_win_line(curwin.get(), (*curwin.get()).w_cursor.lnum) };
    }
    None
}

/// 'highlight' is Vim's highlight-group mapping, which nvim does not
/// implement: only its default value is accepted.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_highlight(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: both are C strings.
    if !unsafe { cstr::eq(*varp(args), HIGHLIGHT_INIT.as_ptr()) } {
        return Some(e_unsupportedoption);
    }
    None
}

/// 'inccommand' cannot change while a preview is on screen, because the
/// preview was set up under the old value.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_inccommand(args: &mut optset_T) -> Option<&CStr> {
    if cmdpreview.get() {
        return invalid();
    }
    unsafe { did_set_str_generic(args) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_keymodel(args: &mut optset_T) -> Option<&CStr> {
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    // SAFETY: the option's C string value; `vim_strchr` only reads it.
    km_stopsel.set(!unsafe { vim_strchr(p_km.get(), c_int::from(b'o')) }.is_null());
    km_startsel.set(!unsafe { vim_strchr(p_km.get(), c_int::from(b'a')) }.is_null());
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_messagesopt(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: reads the option's own value.
    if unsafe { messagesopt_changed() }.is_err() {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_mouse(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), super::MOUSE_ALL.as_ptr(), buf, len) }
}

/// 'mousescroll' is `ver:<n>` and/or `hor:<n>`, each at most once. A
/// direction the value does not mention keeps its built-in default rather
/// than whatever the previous value set.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_mousescroll(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the option's own value is a C string.
    let value = unsafe { CStr::from_ptr(p_mousescroll.get()) }.to_bytes();
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
            return Some(c"E5080: Digit expected");
        }
        // The digits are read with `getdigits_int`, which is what rejects a
        // number too large for an `int`.
        // SAFETY: `at` points into the option's own C string, at the digits
        // just vetted.
        let mut at = unsafe { p_mousescroll.get().add(offset + 4) };
        let number = unsafe { getdigits_int(&raw mut at, false, -1) };
        if number == -1 {
            return invalid();
        }
        *direction = Some(OptInt::from(number));
        offset += part.len() + 1;
    }

    p_mousescroll_vert.set(vertical.unwrap_or(MOUSESCROLL_VERT_DFLT as OptInt));
    p_mousescroll_hor.set(horizontal.unwrap_or(MOUSESCROLL_HOR_DFLT as OptInt));
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_selection(args: &mut optset_T) -> Option<&CStr> {
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    // Whether the character under the cursor is inside the selection just
    // changed.
    if visual_active() {
        // SAFETY: marks the current buffer's windows.
        redraw_curbuf_later(UPD_INVERTED);
    }
    None
}

/// Every character of 'showbreak' has to fit in one cell, because the
/// leader is drawn in a fixed-width margin.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_showbreak(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's value is a C string, and the walk steps by the
    // length of the character it just measured.
    let mut s = unsafe { *varp(args) };
    while unsafe { *s } != 0 {
        if unsafe { ptr2cells(s) } != 1 {
            return Some(e_showbreak_contains_unprintable_or_wide_character);
        }
        s = unsafe { s.add(utfc_ptr2len(s) as usize) };
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_showcmdloc(args: &mut optset_T) -> Option<&CStr> {
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_none() {
        // The pending-command display shares the last line with the ruler.
        // SAFETY: recomputes a global from the editor's own state.
        unsafe { comp_col() };
    }
    errmsg
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_signcolumn(args: &mut optset_T) -> Option<&CStr> {
    let (wp, varp) = (win(args), varp(args));
    // SAFETY: the frame's window and value.
    let local = unsafe { &raw mut (*wp).w_onebuf_opt.wo_scl };
    if unsafe { check_signcolumn(*varp, local_window(varp, wp, local)) }.is_err() {
        return invalid();
    }
    // "number" shares the sign column with the number column, so
    // leaving or entering it invalidates the cached number width.
    let old = old_value(args);
    if (unsafe { *old } == b'n' as c_char && unsafe { *old.add(1) } == b'u' as c_char)
        || unsafe { (*wp).w_minscwidth } == SCL_NUM
    {
        unsafe { (*wp).w_nrwidth_line_count = 0 as linenr_T };
    }
    None
}

/// 'virtualedit' keeps a mask beside its string, and the window-local one
/// uses an empty value to mean "no override" rather than "no virtual edit".
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_virtualedit(args: &mut optset_T) -> Option<&CStr> {
    let wp = win(args);
    // SAFETY: the caller's frame and window.
    let local = args.os_flags.has(OptionSetFlags::LOCAL);
    let value = unsafe {
        if local {
            (*wp).w_onebuf_opt.wo_ve
        } else {
            p_ve.get()
        }
    };
    let store = |mask: c_uint| {
        if local {
            // SAFETY: the frame's window.
            unsafe { (*wp).w_onebuf_opt.wo_ve_flags = mask };
        } else {
            ve_flags.set(mask);
        }
    };

    // SAFETY: the option's C string value.
    if local && unsafe { c_int::from(*value) } == NUL {
        store(0 as c_uint);
        return None;
    }
    // SAFETY: the same C string, against the table's own word list.
    let Some(mask) = (unsafe { opt_strings_mask(value, &opt_ve_values, true) }) else {
        return invalid();
    };
    store(mask);
    // SAFETY: the frame's old value and window.
    if !unsafe { cstr::eq(value, old_value(args)) } {
        // What column the cursor may sit in just changed.
        validate_virtcol(unsafe { Win::new(wp) });
        coladvance(unsafe { Win::new(wp) }, unsafe { (*wp).w_virtcol });
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_whichwrap(args: &mut optset_T) -> Option<&CStr> {
    // 'whichwrap' is spelled as a comma-separated list but checked as a set
    // of letters, so the comma is one of the accepted letters.
    const WW_AND_COMMA: &CStr = c"bshl<>[]~,";
    debug_assert!(WW_AND_COMMA.to_bytes().starts_with(WW_ALL.to_bytes()));
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), WW_AND_COMMA.as_ptr(), buf, len) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_wildmode(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: reads the option's own value.
    if unsafe { check_opt_wim() }.is_err() {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_winbar(args: &mut optset_T) -> Option<&CStr> {
    unsafe { answer_err(args, did_set_statustabline_rulerformat(args, false, false)) }
}

/// Would `border_opt` be accepted as a floating window's border?
///
/// The API's parser is reused, which needs a whole window configuration to
/// write into and an error slot to report through; both are discarded.
///
/// # Safety
/// `border_opt` is a C string.
pub(crate) unsafe fn parse_border_opt(border_opt: *mut c_char) -> bool {
    let mut fconfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0,
        width: 0,
        row: 0.0,
        col: 0.0,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false,
        focusable: true,
        mouse: true,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        },
        footer_width: 0,
        noautocmd: false,
        fixed: false,
        hide: false,
        _cmdline_offset: INT_MAX,
    };
    let mut err = Error::none();
    // SAFETY: the caller's C string, and two locals the parser writes into.
    let ok = unsafe { parse_winborder(&raw mut fconfig, border_opt, &mut err) };
    // Whatever the call left behind is dropped rather than reported.
    err.clear();
    ok
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_winborder(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the option's own C string value.
    if !unsafe { parse_border_opt(p_winborder.get()) } {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_pumborder(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the option's own C string value.
    if !unsafe { parse_border_opt(p_pumborder.get()) } {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_winhighlight(args: &mut optset_T) -> Option<&CStr> {
    let (wp, varp) = (win(args), varp(args));
    // SAFETY: the frame's window and C string value.
    let local = unsafe { &raw mut (*wp).w_onebuf_opt.wo_winhl };
    if !unsafe { parse_winhl_opt(*varp, local_window(varp, wp, local)) } {
        return invalid();
    }
    None
}
