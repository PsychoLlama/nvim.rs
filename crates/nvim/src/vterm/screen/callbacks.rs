//! The state machine's callback table, and the twelve entry points in it.
//!
//! Split out of [`super`] for the file-size cap.  Every one of these is a
//! C function pointer the state machine calls back through, so each takes
//! the `user` payload [`super::screen_new`] installed and does nothing but
//! turn it back into a [`Screen`] and call the method next door.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::{c_int, c_void};

use super::Screen;
use super::resize::resize;
use crate::types::{
    VTermAttr, VTermGlyphInfo, VTermLineInfo, VTermPos, VTermProp, VTermRect, VTermStateCallbacks,
    VTermValue,
};

/// # Safety
///
/// `info` must point at a live `VTermGlyphInfo`, unaliased for the call.
/// `user` must be the payload this callback was registered with, live for the
/// call.
unsafe extern "C" fn putglyph(
    info: *mut VTermGlyphInfo,
    pos: VTermPos,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns the glyph for the length of the call.
    let (mut screen, info) = unsafe { (Screen::of(user), &*info) };
    screen.put_glyph(info, pos)
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
unsafe extern "C" fn movecursor(
    pos: VTermPos,
    oldpos: VTermPos,
    visible: c_int,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.report(
        |host| host.movecursor,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |movecursor, data| unsafe { movecursor(pos, oldpos, visible, data) },
        0,
    )
}

/// # Safety
///
/// `val` must point at a live `VTermValue`, unaliased for the call. `user`
/// must be the payload this callback was registered with, live for the call.
unsafe extern "C" fn setpenattr(attr: VTermAttr, val: *mut VTermValue, user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns the value for the length of the call.
    let (mut screen, val) = unsafe { (Screen::of(user), &*val) };
    screen.set_pen_attr(attr, val)
}

/// # Safety
///
/// `val` must point at a live `VTermValue`, unaliased for the call. `user`
/// must be the payload this callback was registered with, live for the call.
unsafe extern "C" fn settermprop(
    prop: VTermProp,
    val: *mut VTermValue,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as for `setpenattr`; the raw value is passed on to the host,
    // which is why it is kept alongside the boolean arm.
    let (mut screen, boolean) = unsafe { (Screen::of(user), (*val).boolean) };
    screen.set_termprop(prop, boolean, val)
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
unsafe extern "C" fn bell(user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.report(
        |host| host.bell,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |bell, data| unsafe { bell(data) },
        0,
    )
}

/// # Safety
///
/// `dark` must point at a writable `bool` the caller owns. `user` must be the
/// payload this callback was registered with, live for the call.
unsafe extern "C" fn theme(dark: *mut bool, user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns the flag for the length of the call.
    let mut screen = unsafe { Screen::of(user) };
    screen.report(
        |host| host.theme,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |theme, data| unsafe { theme(dark, data) },
        1,
    )
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
unsafe extern "C" fn sb_clear(user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    let cleared = screen.report(
        |host| host.sb_clear,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |sb_clear, data| unsafe { sb_clear(data) },
        0,
    );
    (cleared != 0) as c_int
}

/// # Safety
///
/// `newinfo` must point at a live `VTermLineInfo`. `oldinfo` must point at a
/// live `VTermLineInfo`. `user` must be the payload this callback was
/// registered with, live for the call.
unsafe extern "C" fn setlineinfo(
    row: c_int,
    newinfo: *const VTermLineInfo,
    oldinfo: *const VTermLineInfo,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns both line infos for the length of the call.
    let (mut screen, new, old) = unsafe { (Screen::of(user), &*newinfo, &*oldinfo) };
    screen.set_lineinfo(row, new, old)
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
pub(super) unsafe extern "C" fn moverect_internal(
    dest: VTermRect,
    src: VTermRect,
    user: *mut c_void,
) -> c_int {
    // SAFETY: `vterm_scroll_rect` passes on the pointer it was handed, which
    // is the screen this callback was installed for.
    let mut screen = unsafe { Screen::of(user) };
    screen.move_cells(dest, src);
    1
}

/// Tells the host about a move it may be able to perform itself, falling back
/// to reporting the destination as damaged.
///
/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
pub(super) unsafe extern "C" fn moverect_user(
    dest: VTermRect,
    src: VTermRect,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as for `moverect_internal`.
    let mut screen = unsafe { Screen::of(user) };
    if !screen.report_moverect(dest, src) {
        screen.damage(dest);
    }
    1
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
pub(super) unsafe extern "C" fn erase_internal(
    rect: VTermRect,
    selective: c_int,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as for `moverect_internal`.
    let mut screen = unsafe { Screen::of(user) };
    screen.erase_cells(rect, selective != 0);
    1
}

/// The reporting half of an erase: the cells themselves are another pass.
///
/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
pub(super) unsafe extern "C" fn erase_user(
    rect: VTermRect,
    _selective: c_int,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as for `moverect_internal`.
    let mut screen = unsafe { Screen::of(user) };
    screen.damage(rect);
    1
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
unsafe extern "C" fn erase(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.erase_cells(rect, selective != 0);
    screen.damage(rect);
    1
}

/// # Safety
///
/// `user` must be the payload this callback was registered with, live for the
/// call.
unsafe extern "C" fn scrollrect(
    region: VTermRect,
    downward: c_int,
    rightward: c_int,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.scroll_rect(region, downward, rightward);
    1
}

pub(super) static STATE_CALLBACKS: VTermStateCallbacks = VTermStateCallbacks {
    putglyph: Some(putglyph),
    movecursor: Some(movecursor),
    scrollrect: Some(scrollrect),
    moverect: None,
    erase: Some(erase),
    initpen: None,
    setpenattr: Some(setpenattr),
    settermprop: Some(settermprop),
    bell: Some(bell),
    resize: Some(resize),
    theme: Some(theme),
    setlineinfo: Some(setlineinfo),
    sb_clear: Some(sb_clear),
};
