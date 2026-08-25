//! `nvim_tabpage_*`: the tab page entry points.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{
    ERROR_INIT, NIL, Reported, api_clear_error, api_set_error, api_try, arena_array, array_add,
    buffer_by_handle, dict_get_value, dict_set_var, has_key, tabpage_by_handle, window_by_handle,
};
use crate::api::vim::nvim_get_current_win;

use crate::main::{autocmd_no_enter, autocmd_no_leave, cmdwin_buf, cmdwin_type, curwin, e_cmdwin};
use crate::narrow::number_as_int;
use crate::types::{
    Arena, Array, Boolean, Buffer, Error, Integer, KeyDict_tabpage_config, Object, String_0,
    Tabpage, Window, kErrorTypeException, kErrorTypeNone, size_t, tabpage_T, win_T,
};
use crate::window::{
    tabpage_index, tabpage_win_valid, valid_tabpage, win_goto, win_new_tabpage, win_set_buf,
};
use crate::winlayer::{TabPage, Win, windows_in_tab};
use ::libc::abort;
use core::ptr;

/// The windows of `tp`, oldest first.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_tabpage_list_wins(tabpage: Tabpage, arena: *mut Arena) -> Result<Array, Error> {
    let mut err = ERROR_INIT;
    let mut rv = Array::EMPTY;
    let Some(tab) = tabpage_by_handle(tabpage, &mut err).filter(|t| valid_tabpage(t.raw())) else {
        return rv.reported(err);
    };
    // Counted first, because the arena block has to be sized before it is
    // filled and `array_add` asserts against its capacity.
    let n = windows_in_tab(tab).count() as size_t;
    // SAFETY: `arena` is the caller's, and `rv` is the block it just handed
    // back, sized for exactly the windows appended below.
    unsafe {
        rv = arena_array(arena, n);
        for wp in windows_in_tab(tab) {
            array_add(&mut rv, Object::window(wp.handle));
        }
    }
    Ok(rv)
}

/// The tab-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes, and `arena` must be the caller's.
pub unsafe fn nvim_tabpage_get_var(
    tabpage: Tabpage,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut err = ERROR_INIT;
    let Some(tab) = tabpage_by_handle(tabpage, &mut err) else {
        return NIL.reported(err);
    };
    // SAFETY: `tab` is a live tabpage, so `tp_vars` is its own dictionary;
    // `name` and `arena` are the caller's, per this function's contract.
    let value = unsafe { dict_get_value(tab.tp_vars, name, arena, &raw mut err) };
    value.reported(err)
}

/// Set the tab-scoped variable `name`.
///
/// # Safety
/// `name` and `value` must own their bytes: the store takes them over.
pub unsafe fn nvim_tabpage_set_var(
    tabpage: Tabpage,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(tab) = tabpage_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: as `nvim_tabpage_get_var`; `value` is the caller's and the
    // store takes it over.
    let no_arena = ptr::null_mut::<Arena>();
    unsafe {
        dict_set_var(
            tab.tp_vars,
            name,
            value,
            false,
            false,
            no_arena,
            &raw mut err,
        )
    };
    ().reported(err)
}

/// Remove the tab-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes.
pub unsafe fn nvim_tabpage_del_var(tabpage: Tabpage, name: String_0) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(tab) = tabpage_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: as `nvim_tabpage_set_var`, with the deleting flag set.
    let no_arena = ptr::null_mut::<Arena>();
    unsafe { dict_set_var(tab.tp_vars, name, NIL, true, false, no_arena, &raw mut err) };
    ().reported(err)
}

/// The window `tp` is showing.
pub fn nvim_tabpage_get_win(tabpage: Tabpage) -> Result<Window, Error> {
    let mut err = ERROR_INIT;
    let Some(tab) = tabpage_by_handle(tabpage, &mut err).filter(|t| valid_tabpage(t.raw())) else {
        return (0 as Window).reported(err);
    };
    if tab.is_current() {
        // SAFETY: the current window is whatever `curwin` names.
        return Ok(unsafe { nvim_get_current_win() });
    }
    let curwin_of_tab: *mut win_T = tab.tp_curwin;
    match windows_in_tab(tab).find(|wp| wp.raw() == curwin_of_tab) {
        Some(wp) => Ok(wp.handle as Window),
        // A tab page that is not current always has a `tp_curwin` in its own
        // window list; upstream aborts here rather than answer a handle it
        // cannot justify.
        //
        // SAFETY: `abort` returns nothing and touches nothing.
        None => unsafe { abort() },
    }
}

/// Make `win` the window `tp` shows.
pub fn nvim_tabpage_set_win(tabpage: Tabpage, win: Window) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(tp) = tabpage_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    let Some(wp) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: both handles named a live object, which is all these ask.
    if !unsafe { tabpage_win_valid(tp.raw(), wp.raw()) } {
        // SAFETY: `err` is this frame's own and the format takes one `int`.
        unsafe {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                c"Window does not belong to tabpage %d".as_ptr(),
                tp.handle,
            );
        }
        return Err(err);
    }
    if tp.is_current() {
        // SAFETY: `wp` is live, and `err` is this frame's own.
        api_try(&mut err, |_| unsafe { win_goto(wp.raw()) });
    } else if tp.tp_curwin != wp.raw() {
        let mut tp = tp;
        tp.tp_prevwin = tp.tp_curwin;
        tp.tp_curwin = wp.raw();
    }
    ().reported(err)
}

/// `tp`'s 1-based position in the tab line.
pub fn nvim_tabpage_get_number(tabpage: Tabpage) -> Result<Integer, Error> {
    let mut err = ERROR_INIT;
    let Some(tab) = tabpage_by_handle(tabpage, &mut err) else {
        return (0 as Integer).reported(err);
    };
    Ok(Integer::from(tabpage_index(tab.raw())))
}

/// Whether `tabpage` still names a tab page.
pub fn nvim_tabpage_is_valid(tabpage: Tabpage) -> Boolean {
    let mut stub: Error = ERROR_INIT;
    let ret = tabpage_by_handle(tabpage, &mut stub).is_some();
    // SAFETY: `stub` is this frame's own; the message the lookup may have
    // left behind is freed here rather than reported.
    unsafe { api_clear_error(&raw mut stub) };
    ret
}

/// Open a new tab page showing `buf`.
///
/// # Safety
/// `config` must point at a filled-in `KeyDict_tabpage_config`.
pub unsafe fn nvim_open_tabpage(
    buf: Buffer,
    enter: Boolean,
    config: *mut KeyDict_tabpage_config,
) -> Result<Tabpage, Error> {
    // `after`'s index in `config`'s `is_set` mask. Function-local so that it
    // cannot collide in the flat namespace `tools/ffigen` renders
    // module-level constants into.
    const OPTIDX_AFTER: ::core::ffi::c_int = 1;

    let mut err = ERROR_INIT;
    let Some(b) = buffer_by_handle(buf, &mut err) else {
        return (0 as Tabpage).reported(err);
    };
    if cmdwin_type.get() != 0 && enter || b.raw() == cmdwin_buf.get() {
        // SAFETY: `err` is this frame's own and `e_cmdwin` is a static
        // message.
        unsafe {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                c"%s".as_ptr(),
                (&raw const e_cmdwin).cast::<::core::ffi::c_char>(),
            );
        }
        return Err(err);
    }
    // SAFETY: `config` is the caller's, per this function's contract.
    let after = unsafe {
        if has_key((*config).is_set__tabpage_config_, OPTIDX_AFTER) {
            number_as_int((*config).after)
        } else {
            -1
        }
    };

    let mut wp: *mut win_T = ptr::null_mut();
    // SAFETY: `wp` is this frame's own out-parameter and `b` is live.
    let tp: *mut tabpage_T = api_try(&mut err, |_| unsafe {
        win_new_tabpage(
            after + 1,
            ptr::null_mut::<::core::ffi::c_char>(),
            enter,
            &raw mut wp,
        )
    });
    if tp.is_null() {
        if err.type_0 == kErrorTypeNone {
            // SAFETY: `err` is this frame's own and the format takes nothing.
            unsafe {
                api_set_error(
                    &raw mut err,
                    kErrorTypeException,
                    c"Failed to create new tabpage".as_ptr(),
                );
            }
        }
        return Err(err);
    }
    if !valid_tabpage(tp) {
        return Err(tabpage_closed(err));
    }
    // SAFETY: `tp` is live, and `win_new_tabpage` filled `wp` in.
    let tp = unsafe { TabPage::new(tp) };

    // SAFETY: as above; `tabpage_win_valid` reads both lists and nothing else.
    let new_win = unsafe { Win::from_raw(wp) }
        .filter(|w| unsafe { tabpage_win_valid(tp.raw(), w.raw()) })
        .filter(|w| w.w_buffer != b.raw());
    if let Some(w) = new_win {
        // `win_set_buf` fires `BufEnter`/`BufLeave` only for the window the
        // user is in; a tab page opened without entering it must not.
        let au_no_enter_leave = curwin.get() != w.raw();
        // SAFETY: the two counters are the editor's own `int` globals.
        unsafe {
            if au_no_enter_leave {
                autocmd_no_enter.set(autocmd_no_enter.get() + 1);
                autocmd_no_leave.set(autocmd_no_leave.get() + 1);
            }
            win_set_buf(w.raw(), b.raw(), &raw mut err);
            if au_no_enter_leave {
                autocmd_no_enter.set(autocmd_no_enter.get() - 1);
                autocmd_no_leave.set(autocmd_no_leave.get() - 1);
            }
        }
        if !valid_tabpage(tp.raw()) {
            return Err(tabpage_closed(err));
        }
    }
    (tp.handle as Tabpage).reported(err)
}

/// Replace whatever `err` was carrying with "the tab page went away", which
/// `nvim_open_tabpage` reports at both points a `BufEnter` autocommand could
/// have closed what it just opened.
fn tabpage_closed(mut err: Error) -> Error {
    // SAFETY: `err` is the caller's, moved in, and the format takes nothing.
    unsafe {
        api_clear_error(&raw mut err);
        api_set_error(
            &raw mut err,
            kErrorTypeException,
            c"Tabpage was closed immediately".as_ptr(),
        );
    }
    err
}
