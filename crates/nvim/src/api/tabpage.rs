//! `nvim_tabpage_*`: the tab page entry points.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{
    Reported, api_try, arena_array, array_add, dict_get_value, dict_set_var, find_buffer_by_handle,
    find_tab_by_handle, find_window_by_handle, has_key,
};
use crate::api::vim::nvim_get_current_win;
use crate::window::tab_index;

use crate::api_error;
use crate::guard::Suppress;
use crate::message::e_cmdwin;
use crate::narrow::number_as_int;
use crate::types::{
    Arena, Array, Boolean, BufferHandle, Error, Integer, KeyDict_tabpage_config, Object, String_0,
    TabpageHandle, WindowHandle, kErrorTypeException, size_t,
};
use crate::window::{
    tabpage_win_valid, valid_tab, valid_tabpage, win_goto, win_new_tabpage, win_set_buf,
};
use crate::winlayer::graph::{cmdwin_buf, cmdwin_type};
use crate::winlayer::{Win, windows_in_tab};
use ::libc::abort;
use core::ffi::CStr;
use core::ptr;

/// The windows of `tabpage`, oldest first.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_tabpage_list_wins(
    tabpage: TabpageHandle,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut err = Error::none();
    let mut rv = Array::EMPTY;
    let Some(tab) = find_tab_by_handle(tabpage, &mut err).filter(|&t| valid_tabpage(t.id())) else {
        return rv.reported(err);
    };
    // Counted first, because the arena block has to be sized before it is
    // filled and `array_add` asserts against its capacity.
    let n = windows_in_tab(tab).count() as size_t;
    // SAFETY: `arena` is the caller's, and `rv` is the block it just handed
    // back, sized for exactly the windows appended below.
    rv = arena_array(arena, n);
    for wp in windows_in_tab(tab) {
        unsafe { array_add(&mut rv, Object::window(wp.handle)) };
    }
    Ok(rv)
}

/// The tab-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes, and `arena` must be the caller's.
pub unsafe fn nvim_tabpage_get_var(
    tabpage: TabpageHandle,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut err = Error::none();
    let Some(tab) = find_tab_by_handle(tabpage, &mut err) else {
        return Object::Nil.reported(err);
    };
    // SAFETY: `tab` is a live tabpage, so `tp_vars` is its own dictionary;
    // `name` and `arena` are the caller's, per this function's contract.
    let value = unsafe { dict_get_value(tab.tp_vars, name, arena, &mut err) };
    value.reported(err)
}

/// Set the tab-scoped variable `name`.
///
/// # Safety
/// `name` and `value` must own their bytes: the store takes them over.
pub unsafe fn nvim_tabpage_set_var(
    tabpage: TabpageHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut err = Error::none();
    let Some(tab) = find_tab_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: as `nvim_tabpage_get_var`; `value` is the caller's and the
    // store takes it over.
    let no_arena = ptr::null_mut::<Arena>();
    let vars = tab.tp_vars;
    unsafe { dict_set_var(vars, name, value, false, false, no_arena, &mut err) };
    ().reported(err)
}

/// Remove the tab-scoped variable `name`.
///
/// # Safety
/// `name` must point at its own bytes.
pub unsafe fn nvim_tabpage_del_var(tabpage: TabpageHandle, name: String_0) -> Result<(), Error> {
    let mut err = Error::none();
    let Some(tab) = find_tab_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: as `nvim_tabpage_set_var`, with the deleting flag set.
    let no_arena = ptr::null_mut::<Arena>();
    let vars = tab.tp_vars;
    unsafe { dict_set_var(vars, name, Object::Nil, true, false, no_arena, &mut err) };
    ().reported(err)
}

/// The window `tabpage` is showing.
pub fn nvim_tabpage_get_win(tabpage: TabpageHandle) -> Result<WindowHandle, Error> {
    let mut err = Error::none();
    let Some(tab) = find_tab_by_handle(tabpage, &mut err).filter(|&t| valid_tabpage(t.id())) else {
        return (0 as WindowHandle).reported(err);
    };
    if tab.is_current() {
        return Ok(nvim_get_current_win());
    }
    let curwin_of_tab = tab.current_window();
    match windows_in_tab(tab).find(|&wp| curwin_of_tab == Some(wp)) {
        Some(wp) => Ok(wp.handle as WindowHandle),
        // A tab page that is not current always has a `tp_curwin` in its own
        // window list; upstream aborts here rather than answer a handle it
        // cannot justify.
        //
        // SAFETY: `abort` returns nothing and touches nothing.
        None => unsafe { abort() },
    }
}

/// Make `win` the window `tp` shows.
pub fn nvim_tabpage_set_win(tabpage: TabpageHandle, win: WindowHandle) -> Result<(), Error> {
    let mut err = Error::none();
    let Some(tp) = find_tab_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    let Some(wp) = find_window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    // SAFETY: both handles named a live object, which is all these ask.
    if !tabpage_win_valid(tp, wp.id()) {
        let handle = tp.handle;
        return Err(api_error!(
            kErrorTypeException,
            "Window does not belong to tabpage {handle}"
        ));
    }
    if tp.is_current() {
        api_try(&mut err, |_| win_goto(wp));
    } else if tp.tp_curwin != Some(wp.id()) {
        let mut tp = tp;
        tp.tp_prevwin = tp.tp_curwin;
        tp.tp_curwin = Some(wp.id());
    }
    ().reported(err)
}

/// `tabpage`'s 1-based position in the tab line.
pub fn nvim_tabpage_get_number(tabpage: TabpageHandle) -> Result<Integer, Error> {
    let mut err = Error::none();
    let Some(tab) = find_tab_by_handle(tabpage, &mut err) else {
        return (0 as Integer).reported(err);
    };
    Ok(Integer::from(tab_index(tab)))
}

/// Whether `tabpage` still names a tab page.
pub fn nvim_tabpage_is_valid(tabpage: TabpageHandle) -> Boolean {
    let mut stub: Error = Error::none();
    let ret = find_tab_by_handle(tabpage, &mut stub).is_some();
    // The message the lookup may have left behind is dropped rather than
    // reported.
    stub.clear();
    ret
}

/// Open a new tab page showing `buf`.
///
/// # Safety
/// `config` must point at a filled-in `KeyDict_tabpage_config`.
pub unsafe fn nvim_open_tabpage(
    buf: BufferHandle,
    enter: Boolean,
    config: *mut KeyDict_tabpage_config,
) -> Result<TabpageHandle, Error> {
    // `after`'s index in `config`'s `is_set` mask. Function-local so that it
    // cannot collide in the flat namespace `tools/ffigen` renders
    // module-level constants into.
    const OPTIDX_AFTER: ::core::ffi::c_int = 1;

    let mut err = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut err) else {
        return (0 as TabpageHandle).reported(err);
    };
    if cmdwin_type.get() != 0 && enter || cmdwin_buf.get() == Some(b.id()) {
        return Err(Error::exception(e_cmdwin));
    }
    // SAFETY: `config` is the caller's, per this function's contract.
    let after = unsafe {
        if has_key((*config).is_set__tabpage_config_, OPTIDX_AFTER) {
            number_as_int((*config).after)
        } else {
            -1
        }
    };

    let mut wp: Option<Win> = None;
    // SAFETY: `wp` is this frame's own out-parameter and `b` is live.
    let tp = api_try(&mut err, |_| {
        let filename = ptr::null_mut::<::core::ffi::c_char>();
        // SAFETY: `wp` is this frame's own out-parameter.
        unsafe { win_new_tabpage(after + 1, filename, enter, Some(&mut wp)) }
    });
    let Some(tp) = tp else {
        if !err.is_set() {
            set_msg(&mut err, c"Failed to create new tabpage");
        }
        return Err(err);
    };
    // `win_new_tabpage` fires `TabNew`, which can close what it just opened,
    // so the tab page is looked up again by the identity it was born with.
    let Some(tp) = valid_tab(tp.id()) else {
        return Err(tabpage_closed(err));
    };

    let new_win = wp
        .and_then(|w| windows_in_tab(tp).find(|live| live.raw() == w.raw()))
        .filter(|w| w.w_buffer != b.raw());
    if let Some(w) = new_win {
        // `win_set_buf` fires `BufEnter`/`BufLeave` only for the window the
        // user is in; a tab page opened without entering it must not.
        let quiet = (Win::current_raw() != w.raw()).then(Suppress::win_enter_leave_autocmds);
        win_set_buf(w, b, &mut err);
        drop(quiet);
        if !valid_tabpage(tp.id()) {
            return Err(tabpage_closed(err));
        }
    }
    (tp.handle as TabpageHandle).reported(err)
}

/// Replace whatever `err` was carrying with "the tab page went away", which
/// `nvim_open_tabpage` reports at both points a `BufEnter` autocommand could
/// have closed what it just opened.
fn tabpage_closed(mut err: Error) -> Error {
    // SAFETY: `err` is the caller's, moved in.
    err.clear();
    set_msg(&mut err, c"Tabpage was closed immediately");
    err
}

/// An exception whose whole message is `msg`.
fn set_msg(err: &mut Error, msg: &CStr) {
    *err = Error::exception(msg);
}
