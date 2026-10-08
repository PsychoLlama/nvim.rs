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
    api_try, dict_get_value, dict_set_var, find_buffer_by_handle, find_tab_by_handle,
    find_window_by_handle, scope_vars,
};
use crate::api::vim::nvim_get_current_win;
use crate::window::tab_index;

use crate::api_error;
use crate::guard::Suppress;
use crate::message::e_cmdwin;
use crate::narrow::number_as_int;
use crate::types::{
    Array, Boolean, BufferHandle, Error, Integer, KeyDict_tabpage_config, Object, String_0,
    TabpageHandle, WindowHandle, kErrorTypeException, size_t,
};
use crate::window::{
    tabpage_win_valid, valid_tab, valid_tabpage, win_goto, win_new_tabpage, win_set_buf,
};
use crate::winlayer::graph::{cmdwin_buf, cmdwin_type};
use crate::winlayer::{Win, windows_in_tab};
use ::libc::abort;

/// The windows of `tabpage`, oldest first.
pub fn nvim_tabpage_list_wins(tabpage: TabpageHandle) -> Result<Array, Error> {
    let mut rv = Array::EMPTY;
    let Some(tab) = find_tab_by_handle(tabpage)?.filter(|&t| valid_tabpage(t.id())) else {
        return Ok(rv);
    };
    // Counted first, so the array is allocated once.
    let n = windows_in_tab(tab).count() as size_t;
    rv = Array::with_capacity(n);
    for wp in windows_in_tab(tab) {
        rv.push(Object::window(wp.handle));
    }
    Ok(rv)
}

/// The tab-scoped variable `name`.
pub fn nvim_tabpage_get_var(tabpage: TabpageHandle, name: String_0) -> Result<Object, Error> {
    let Some(tab) = find_tab_by_handle(tabpage)? else {
        return Ok(Object::Nil);
    };
    dict_get_value(&scope_vars(&tab.tp_winvar), &name)
}

/// Set the tab-scoped variable `name`.
pub fn nvim_tabpage_set_var(
    tabpage: TabpageHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let Some(tab) = find_tab_by_handle(tabpage)? else {
        return Ok(());
    };
    let vars = scope_vars(&tab.tp_winvar);
    dict_set_var(&vars, &name, value, false, false).map(|_| ())
}

/// Remove the tab-scoped variable `name`.
pub fn nvim_tabpage_del_var(tabpage: TabpageHandle, name: String_0) -> Result<(), Error> {
    let Some(tab) = find_tab_by_handle(tabpage)? else {
        return Ok(());
    };
    let vars = scope_vars(&tab.tp_winvar);
    dict_set_var(&vars, &name, Object::Nil, true, false).map(|_| ())
}

/// The window `tabpage` is showing.
pub fn nvim_tabpage_get_win(tabpage: TabpageHandle) -> Result<WindowHandle, Error> {
    let Some(tab) = find_tab_by_handle(tabpage)?.filter(|&t| valid_tabpage(t.id())) else {
        return Ok(0 as WindowHandle);
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
    let Some(tp) = find_tab_by_handle(tabpage)? else {
        return Ok(());
    };
    let Some(wp) = find_window_by_handle(win)? else {
        return Ok(());
    };
    if !tabpage_win_valid(tp, wp.id()) {
        let handle = tp.handle;
        return Err(api_error!(
            kErrorTypeException,
            "Window does not belong to tabpage {handle}"
        ));
    }
    if tp.is_current() {
        api_try(|| win_goto(wp))?;
    } else if tp.tp_curwin != Some(wp.id()) {
        let mut tp = tp;
        tp.tp_prevwin = tp.tp_curwin;
        tp.tp_curwin = Some(wp.id());
    }
    Ok(())
}

/// `tabpage`'s 1-based position in the tab line.
pub fn nvim_tabpage_get_number(tabpage: TabpageHandle) -> Result<Integer, Error> {
    let Some(tab) = find_tab_by_handle(tabpage)? else {
        return Ok(0 as Integer);
    };
    Ok(Integer::from(tab_index(tab)))
}

/// Whether `tabpage` still names a tab page.
pub fn nvim_tabpage_is_valid(tabpage: TabpageHandle) -> Boolean {
    // A handle that names nothing is not an error here, only a `false`.
    find_tab_by_handle(tabpage).unwrap_or_default().is_some()
}

/// Open a new tab page showing `buf`.
pub fn nvim_open_tabpage(
    buf: BufferHandle,
    enter: Boolean,
    config: &mut KeyDict_tabpage_config,
) -> Result<TabpageHandle, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as TabpageHandle);
    };
    if cmdwin_type.get() != 0 && enter || cmdwin_buf.get() == Some(b.id()) {
        return Err(Error::exception(e_cmdwin));
    }
    let after = config.after.map_or(-1, number_as_int);

    let mut wp: Option<Win> = None;
    let tp = api_try(|| win_new_tabpage(after + 1, None, enter, Some(&mut wp)))?;
    let Some(tp) = tp else {
        return Err(Error::exception(c"Failed to create new tabpage"));
    };
    // `win_new_tabpage` fires `TabNew`, which can close what it just opened,
    // so the tab page is looked up again by the identity it was born with.
    let Some(tp) = valid_tab(tp.id()) else {
        return Err(tabpage_closed());
    };

    let new_win = wp
        .and_then(|w| windows_in_tab(tp).find(|live| live.raw() == w.raw()))
        .filter(|w| w.w_buffer != b);
    if let Some(w) = new_win {
        // `win_set_buf` fires `BufEnter`/`BufLeave` only for the window the
        // user is in; a tab page opened without entering it must not.
        let quiet = (Win::current_raw() != w.raw()).then(Suppress::win_enter_leave_autocmds);
        let set = win_set_buf(w, b);
        drop(quiet);
        if !valid_tabpage(tp.id()) {
            return Err(tabpage_closed());
        }
        set?;
    }
    Ok(tp.handle as TabpageHandle)
}

/// "The tab page went away", which `nvim_open_tabpage` reports at both points
/// a `BufEnter` autocommand could have closed what it just opened -- in
/// preference to whatever else went wrong there.
fn tabpage_closed() -> Error {
    Error::exception(c"Tabpage was closed immediately")
}
