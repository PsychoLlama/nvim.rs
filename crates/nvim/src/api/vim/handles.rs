//! Listing buffers, windows and tabpages, and switching between them.
//!
//! Ten accessors of one shape -- `nvim_list_*` walks the editor's own list
//! into an `Array` of handles, `nvim_get_current_*` reads the pointer and
//! `nvim_set_current_*` moves it -- plus `nvim_create_buf`, which is the
//! only one that builds something.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{
    ERROR_INIT, Reported, api_try, array_add, buffer_by_handle, tabpage_by_handle, window_by_handle,
};
use crate::types::OptionSetFlags;
use core::ffi::CStr;
use core::ptr;

use crate::buffer::BufRef;
use crate::winlayer::{Buf, TabPage, Win, buffers, tab_windows, tabs};

/// The current buffer, which exists from startup to exit.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` names a live buffer for the editor's whole run.
    unsafe { Buf::current() }
}

/// The current window, which exists from startup to exit.
fn cur_win() -> Win {
    // SAFETY: `curwin` names a live window for the editor's whole run.
    unsafe { Win::current() }
}

/// The current tab page, which exists from startup to exit.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` names a live tab page for the editor's whole run.
    unsafe { TabPage::current() }
}

/// One `String` option's value, borrowing the literal's bytes.
fn string_optval(value: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0::from_cstr(value),
        },
    }
}

/// Every listed and unlisted buffer's handle.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_list_bufs(arena: *mut Arena) -> Array {
    let n: size_t = buffers().count();
    let mut rv: Array = arena_array(arena, n);
    for buf in buffers() {
        // SAFETY: `rv` is the block `arena` just sized for every buffer.
        unsafe { array_add(&mut rv, Object::buffer(buf.handle)) };
    }
    rv
}

/// The current buffer's handle.
///
/// # Safety
/// The editor must be running: there is a current buffer from startup to
/// exit.
pub unsafe fn nvim_get_current_buf() -> Buffer {
    cur_buf().handle
}

/// Make `buf` the current buffer, as `:buffer` does.
///
/// # Safety
/// The editor must be running.
pub unsafe fn nvim_set_current_buf(buf: Buffer) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(b) = buffer_by_handle(buf, &mut err) else {
        return ().reported(err);
    };
    let handle = b.handle;
    api_try(&mut err, |_| {
        do_buffer(
            DOBUF_GOTO as ::core::ffi::c_int,
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            handle,
            0 as ::core::ffi::c_int,
        );
    });
    ().reported(err)
}

/// Every window of the current tab page, in layout order.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_list_wins(arena: *mut Arena) -> Array {
    let n: size_t = tab_windows().count();
    let mut rv: Array = arena_array(arena, n);
    for win in tab_windows() {
        // SAFETY: `rv` is the block `arena` just sized for every window.
        unsafe { array_add(&mut rv, Object::window(win.handle)) };
    }
    rv
}

/// The current window's handle.
///
/// # Safety
/// The editor must be running: there is a current window from startup to
/// exit.
pub unsafe fn nvim_get_current_win() -> Window {
    cur_win().handle
}

/// Make `win` the current window, entering its tab page if need be.
///
/// # Safety
/// The editor must be running.
pub unsafe fn nvim_set_current_win(win: Window) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(w) = window_by_handle(win, &mut err) else {
        return ().reported(err);
    };
    api_try(&mut err, |_| {
        if w.w_buffer != curbuf.get() {
            reset_VIsual_and_resel();
        }
        // SAFETY: `w` is the live window just found.
        unsafe { goto_tabpage_win(win_find_tabpage(w.raw()), w.raw()) };
    });
    ().reported(err)
}

/// A new empty buffer: `listed` for one `:ls` shows, `scratch` for one with
/// `'buftype'` `nofile`, `'bufhidden'` `hide` and no swap file.
///
/// # Safety
/// The editor must be running.
pub unsafe fn nvim_create_buf(listed: Boolean, scratch: Boolean) -> Result<Buffer, Error> {
    let mut err = ERROR_INIT;
    let ret = api_try(&mut err, |_| create_buf(listed, scratch));
    if ret == 0 && !err.is_set() {
        err = Error::exception(c"Failed to create buffer");
    }
    ret.reported(err)
}

/// [`nvim_create_buf`]'s body, inside the try/catch bracket.
fn create_buf(listed: Boolean, scratch: Boolean) -> Buffer {
    // SAFETY: paired with the `unblock_autocmds` on both paths below.
    unsafe { block_autocmds() };
    let flags = BLN_NOOPT as ::core::ffi::c_int
        | BLN_NEW as ::core::ffi::c_int
        | if listed {
            BLN_LISTED as ::core::ffi::c_int
        } else {
            0
        };
    let no_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
    // SAFETY: a new buffer with neither a file name nor a short name.
    let buf = unsafe { buflist_new(no_name, no_name, 0 as linenr_T, flags) };
    // SAFETY: `buf` is the buffer just made, or null.
    let opened = !buf.is_null() && unsafe { ml_open(buf) } != 0 as ::core::ffi::c_int;
    if !opened {
        // SAFETY: paired with the `block_autocmds` above.
        unsafe { unblock_autocmds() };
        return 0;
    }
    // SAFETY: `buf` is the live buffer just made.
    let mut b = unsafe { Buf::new(buf) };
    let tick = buf_get_changedtick(b);
    b.b_last_changedtick = tick;
    b.b_last_changedtick_i = tick;
    b.b_last_changedtick_pum = tick;
    // SAFETY: as above.
    unsafe {
        buf_copy_options(
            buf,
            BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
        );
    }
    if scratch {
        let target = buf.cast::<::core::ffi::c_void>();
        let local = OptionSetFlags::LOCAL;
        // SAFETY: as above; the two values borrow static literals.
        unsafe {
            let hide = string_optval(c"hide");
            set_option_direct_for(kOptBufhidden, hide, local, 0, kOptScopeBuf, target);
            let nofile = string_optval(c"nofile");
            set_option_direct_for(kOptBuftype, nofile, local, 0, kOptScopeBuf, target);
        }
        debug_assert!(
            // SAFETY: a buffer `ml_open` answered for has a memfile.
            unsafe { (*b.b_ml.ml_mfp).mf_fd } < 0 as ::core::ffi::c_int,
            "buf->b_ml.ml_mfp->mf_fd < 0"
        );
        b.b_p_swf = 0;
        b.b_p_ml = 0;
    }
    // SAFETY: paired with the `block_autocmds` above.
    unsafe { unblock_autocmds() };
    let bufref = BufRef::of_opt(Some(b));
    // SAFETY: `buf` is live, and the event has neither a file name nor a
    // pattern. A handler may wipe the buffer, which is what `bufref` checks.
    let wiped =
        unsafe { apply_autocmds(EVENT_BUFNEW, ptr::null_mut(), ptr::null_mut(), false, buf) }
            && !bufref.valid();
    if wiped {
        return 0;
    }
    // SAFETY: as above.
    let wiped = listed
        && unsafe { apply_autocmds(EVENT_BUFADD, ptr::null_mut(), ptr::null_mut(), false, buf) }
        && !bufref.valid();
    if wiped {
        return 0;
    }
    // SAFETY: the autocommands above left the buffer alive.
    unsafe { (*buf).handle }
}

/// Every tab page's handle, in order.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_list_tabpages(arena: *mut Arena) -> Array {
    let n: size_t = tabs().count();
    let mut rv: Array = arena_array(arena, n);
    for tp in tabs() {
        // SAFETY: `rv` is the block `arena` just sized for every tab page.
        unsafe { array_add(&mut rv, Object::tabpage(tp.handle)) };
    }
    rv
}

/// The current tab page's handle.
///
/// # Safety
/// The editor must be running: there is a current tab page from startup to
/// exit.
pub unsafe fn nvim_get_current_tabpage() -> Tabpage {
    cur_tab().handle
}

/// Make `tabpage` the current one, as `:tabnext` does.
///
/// # Safety
/// The editor must be running.
pub unsafe fn nvim_set_current_tabpage(tabpage: Tabpage) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let Some(tp) = tabpage_by_handle(tabpage, &mut err) else {
        return ().reported(err);
    };
    api_try(&mut err, |_| {
        // SAFETY: `tp` is the live tab page just found.
        unsafe { goto_tabpage_tp(tp.raw(), true, true) };
    });
    ().reported(err)
}
