//! Listing buffers, windows and tabpages, and switching between them.
//!
//! Ten accessors of one shape -- `nvim_list_*` walks the editor's own list
//! into an `Array` of handles, `nvim_get_current_*` reads the pointer and
//! `nvim_set_current_*` moves it -- plus `nvim_create_buf`, which is the
//! only one that builds something.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::{
    api_try, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
};
use crate::types::OptionSetFlags;
use core::ffi::CStr;
use core::ptr;

use crate::buffer::BufRef;
use crate::winlayer::{Buf, TabPage, Win, buffers, tab_windows, tabs};

/// One `String` option's value, borrowing the literal's bytes.
fn string_optval(value: &'static CStr) -> OptVal {
    OptVal::static_string(value)
}

/// Every listed and unlisted buffer's handle.
///
/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_list_bufs() -> Array {
    let n: size_t = buffers().count();
    let mut rv: Array = Array::with_capacity(n);
    for buf in buffers() {
        // SAFETY: `rv` is the block `arena` just sized for every buffer.
        rv.push(Object::buffer(buf.handle));
    }
    rv
}

/// The current buffer's handle.
pub fn nvim_get_current_buf() -> BufferHandle {
    Buf::current().handle
}

/// Make `buf` the current buffer, as `:buffer` does.
pub fn nvim_set_current_buf(buf: BufferHandle) -> Result<(), Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(());
    };
    let handle = b.handle;
    api_try(|| {
        let _ = do_buffer(
            DOBUF_GOTO as ::core::ffi::c_int,
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            handle,
            0 as ::core::ffi::c_int,
        );
    })?;
    Ok(())
}

/// Every window of the current tab page, in layout order.
///
/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_list_wins() -> Array {
    let n: size_t = tab_windows().count();
    let mut rv: Array = Array::with_capacity(n);
    for win in tab_windows() {
        // SAFETY: `rv` is the block `arena` just sized for every window.
        rv.push(Object::window(win.handle));
    }
    rv
}

/// The current window's handle.
pub fn nvim_get_current_win() -> WindowHandle {
    Win::current().handle
}

/// Make `win` the current window, entering its tab page if need be.
pub fn nvim_set_current_win(win: WindowHandle) -> Result<(), Error> {
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    api_try(|| {
        if w.w_buffer != Buf::current_raw() {
            reset_visual_and_resel();
        }
        let tab = win_find_tabpage(w.id()).expect("a live window is on a tab page");
        goto_tabpage_win(tab, w);
    })?;
    Ok(())
}

/// A new empty buffer: `listed` for one `:ls` shows, `scratch` for one with
/// `'buftype'` `nofile`, `'bufhidden'` `hide` and no swap file.
pub fn nvim_create_buf(listed: Boolean, scratch: Boolean) -> Result<BufferHandle, Error> {
    let ret = api_try(|| create_buf(listed, scratch))?;
    if ret == 0 {
        return Err(Error::exception(c"Failed to create buffer"));
    }
    Ok(ret)
}

/// [`nvim_create_buf`]'s body, inside the try/catch bracket.
fn create_buf(listed: Boolean, scratch: Boolean) -> BufferHandle {
    block_autocmds();
    let flags = BLN_NOOPT as ::core::ffi::c_int
        | BLN_NEW as ::core::ffi::c_int
        | if listed {
            BLN_LISTED as ::core::ffi::c_int
        } else {
            0
        };
    let no_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
    // SAFETY: a new buffer with neither a file name nor a short name.
    let buf = unsafe { buflist_new(no_name, no_name, 0 as LineNr, flags) };
    let opened = buf.is_some() && ml_open(buf.expect("a live handle")).is_ok();
    if !opened {
        unblock_autocmds();
        return 0;
    }
    let mut b = buf.expect("`buflist_new` answered a buffer");
    let tick = buf_get_changedtick(b);
    b.b_last_changedtick = tick;
    b.b_last_changedtick_i = tick;
    b.b_last_changedtick_pum = tick;
    buf_copy_options(
        buf.expect("a live handle"),
        BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
    );
    if scratch {
        let local = OptionSetFlags::LOCAL;
        let hide = string_optval(c"hide");
        set_option_direct_for(kOptBufhidden, hide, local, 0, OptionTarget::Buf(b));
        let nofile = string_optval(c"nofile");
        set_option_direct_for(kOptBuftype, nofile, local, 0, OptionTarget::Buf(b));
        debug_assert!(
            // SAFETY: a buffer `ml_open` answered for has a memfile.
            unsafe { (*b.b_ml.ml_mfp).mf_fd } < 0 as ::core::ffi::c_int,
            "buf->b_ml.ml_mfp->mf_fd < 0"
        );
        b.b_p_swf = 0;
        b.b_p_ml = 0;
    }
    unblock_autocmds();
    let bufref = BufRef::of_opt(Some(b));
    // SAFETY: `buf` is live, and the event has neither a file name nor a
    // pattern. A handler may wipe the buffer, which is what `bufref` checks.
    let (no_fname, no_fname_io) = (ptr::null_mut(), ptr::null_mut());
    let event = AutoEvent::BufNew;
    let wiped =
        unsafe { apply_autocmds(event, no_fname, no_fname_io, false, buf) } && !bufref.valid();
    if wiped {
        return 0;
    }
    let event = AutoEvent::BufAdd;
    let wiped = listed
        && unsafe { apply_autocmds(event, no_fname, no_fname_io, false, buf) }
        && !bufref.valid();
    if wiped {
        return 0;
    }
    // The autocommands above left the buffer alive.
    buf.map_or(0, |b| b.handle)
}

/// Every tab page's handle, in order.
///
/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
pub unsafe fn nvim_list_tabpages() -> Array {
    let n: size_t = tabs().count();
    let mut rv: Array = Array::with_capacity(n);
    for tp in tabs() {
        // SAFETY: `rv` is the block `arena` just sized for every tab page.
        rv.push(Object::tabpage(tp.handle));
    }
    rv
}

/// The current tab page's handle.
pub fn nvim_get_current_tabpage() -> TabpageHandle {
    TabPage::current().handle
}

/// Make `tabpage` the current one, as `:tabnext` does.
pub fn nvim_set_current_tabpage(tabpage: TabpageHandle) -> Result<(), Error> {
    let Some(tp) = find_tab_by_handle(tabpage)? else {
        return Ok(());
    };
    api_try(|| {
        goto_tabpage_tp(tp, true, true);
    })?;
    Ok(())
}
