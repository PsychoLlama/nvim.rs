use crate::buffer::buflist_findnr;
use crate::main::{IObuff, cmdmod, curbuf, curwin, global_busy, got_int, jop_flags, listcmd_busy};
use crate::memory::{xfree, xstrdup};
use crate::message::{
    message_filtered, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts, msg_puts_title,
};
use crate::os::cshim::{gettext, memmove, snprintf};
use crate::os::input::os_breakcheck;
use crate::os::time::os_time;
use crate::pos::{MAXLNUM, equalpos};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use super::show::*;
use super::*;
use crate::highlight_group::HLF_D;
use crate::types::CMOD_KEEPJUMPS;

/// Set the previous context mark to the current position and add it to the
/// jump list.
pub unsafe extern "C" fn setpcmark() {
    let mut fm: *mut xfmark_T = ptr::null_mut();
    if global_busy.get() != 0
        || listcmd_busy.get()
        || (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as c_int != 0
    {
        return;
    }
    (*curwin.get()).w_prev_pcmark = (*curwin.get()).w_pcmark;
    (*curwin.get()).w_pcmark = (*curwin.get()).w_cursor;
    if (*curwin.get()).w_pcmark.lnum == 0 {
        (*curwin.get()).w_pcmark.lnum = 1;
    }
    if jop_flags.get() & kOptJopFlagStack as c_int as c_uint != 0 {
        if (*curwin.get()).w_jumplistidx < (*curwin.get()).w_jumplistlen - 1 {
            (*curwin.get()).w_jumplistlen = (*curwin.get()).w_jumplistidx + 1;
        }
    }
    (*curwin.get()).w_jumplistlen += 1;
    if (*curwin.get()).w_jumplistlen > JUMPLISTSIZE {
        (*curwin.get()).w_jumplistlen = JUMPLISTSIZE;
        free_xfmark((*curwin.get()).w_jumplist[0]);
        memmove(
            (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T).offset(0) as *mut c_void,
            (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T).offset(1) as *const c_void,
            ((JUMPLISTSIZE - 1) as size_t).wrapping_mul(size_of::<xfmark_T>()),
        );
    }
    (*curwin.get()).w_jumplistidx = (*curwin.get()).w_jumplistlen;
    fm = (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
        .offset(((*curwin.get()).w_jumplistlen - 1) as isize);
    let mut view: fmarkv_T = mark_view_make(curwin.get(), (*curwin.get()).w_pcmark);
    let xfmarkp__: *mut xfmark_T = fm;
    (*xfmarkp__).fname = ptr::null_mut();
    let fmarkp__: *mut fmark_T = &raw mut (*xfmarkp__).fmark;
    (*fmarkp__).mark = (*curwin.get()).w_pcmark;
    (*fmarkp__).fnum = (*curbuf.get()).handle as c_int;
    (*fmarkp__).timestamp = os_time();
    (*fmarkp__).view = view;
    (*fmarkp__).additional_data = ptr::null_mut();
}

/// To change context, call setpcmark(), then move the current position to
/// where ever, then call checkpcmark().  This ensures that the previous
/// context will only be changed if the cursor moved to a different line.
/// If pcmark was deleted (with "dG") the previous mark is restored.
pub unsafe extern "C" fn checkpcmark() {
    if (*curwin.get()).w_prev_pcmark.lnum != 0
        && (equalpos((*curwin.get()).w_pcmark, (*curwin.get()).w_cursor)
            || (*curwin.get()).w_pcmark.lnum == 0)
    {
        (*curwin.get()).w_pcmark = (*curwin.get()).w_prev_pcmark;
    }
    (*curwin.get()).w_prev_pcmark.lnum = 0;
}

/// Get mark in "count" position in the |jumplist| relative to the current index.
///
/// If the mark is in a different buffer, it will be skipped unless the buffer exists.
///
/// @note cleanup_jumplist() is run, which removes duplicate marks, and
///       changes win->w_jumplistidx.
/// `win` — window to get jumplist from.
/// `count` — count to move may be negative.
///
/// Returns mark, NULL if out of jumplist bounds.
pub unsafe extern "C" fn get_jumplist(mut win: *mut win_T, mut count: c_int) -> *mut fmark_T {
    let mut jmp: *mut xfmark_T = ptr::null_mut();
    cleanup_jumplist(win, true);
    if (*win).w_jumplistlen == 0 {
        return ptr::null_mut();
    }
    loop {
        if (*win).w_jumplistidx + count < 0 || (*win).w_jumplistidx + count >= (*win).w_jumplistlen
        {
            return ptr::null_mut();
        }
        if (*win).w_jumplistidx == (*win).w_jumplistlen {
            setpcmark();
            (*win).w_jumplistidx -= 1;
            if (*win).w_jumplistidx + count < 0 {
                return ptr::null_mut();
            }
        }
        (*win).w_jumplistidx += count;
        jmp = (&raw mut (*win).w_jumplist as *mut xfmark_T).offset((*win).w_jumplistidx as isize);
        if (*jmp).fmark.fnum == 0 {
            fname2fnum(jmp);
        }
        if (*jmp).fmark.fnum == (*curbuf.get()).handle {
            break;
        }
        if !buflist_findnr((*jmp).fmark.fnum).is_null() {
            break;
        }
        count += if count < 0 { -1 } else { 1 };
    }
    return &raw mut (*jmp).fmark;
}

/// Get mark in "count" position in the |changelist| relative to the current index.
///
/// @note  Changes the win->w_changelistidx.
/// `win` — window to get jumplist from.
/// `count` — count to move may be negative.
///
/// Returns mark, NULL if out of bounds.
pub unsafe extern "C" fn get_changelist(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut count: c_int,
) -> *mut fmark_T {
    let mut n: c_int = 0;
    let mut fm: *mut fmark_T = ptr::null_mut();
    if (*buf).b_changelistlen == 0 {
        return ptr::null_mut();
    }
    n = (*win).w_changelistidx;
    if n + count < 0 {
        if n == 0 {
            return ptr::null_mut();
        }
        n = 0;
    } else if n + count >= (*buf).b_changelistlen {
        if n == (*buf).b_changelistlen - 1 {
            return ptr::null_mut();
        }
        n = (*buf).b_changelistlen - 1;
    } else {
        n += count;
    }
    (*win).w_changelistidx = n;
    fm = (&raw mut (*buf).b_changelist as *mut fmark_T).offset(n as isize);
    (*fm).fnum = (*curbuf.get()).handle as c_int;
    return (&raw mut (*buf).b_changelist as *mut fmark_T).offset(n as isize);
}

/// Remove every jump list entry referring to a given buffer.
/// This function will also adjust the current jump list index.
pub unsafe extern "C" fn mark_jumplist_forget_file(mut wp: *mut win_T, mut fnum: c_int) {
    let mut i: c_int = (*wp).w_jumplistlen - 1;
    while i >= 0 {
        if (*wp).w_jumplist[i as usize].fmark.fnum == fnum {
            free_xfmark((*wp).w_jumplist[i as usize]);
            if (*wp).w_jumplistidx > i {
                (*wp).w_jumplistidx -= 1;
            }
            (*wp).w_jumplistlen -= 1;
            memmove(
                (&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(i as isize) as *mut c_void,
                (&raw mut (*wp).w_jumplist as *mut xfmark_T).offset((i + 1) as isize)
                    as *const c_void,
                (((*wp).w_jumplistlen - i) as size_t).wrapping_mul(size_of::<xfmark_T>()),
            );
        }
        i -= 1;
    }
}

/// When deleting lines, this may create duplicate marks in the
/// jumplist. They will be removed here for the specified window.
/// When "loadfiles" is true first ensure entries have the "fnum" field set
/// (this may be a bit slow).
pub unsafe extern "C" fn cleanup_jumplist(mut wp: *mut win_T, mut loadfiles: bool) {
    let mut i: c_int = 0;
    if loadfiles {
        i = 0;
        while i < (*wp).w_jumplistlen {
            if (*wp).w_jumplist[i as usize].fmark.fnum == 0
                && (*wp).w_jumplist[i as usize].fmark.mark.lnum != 0
            {
                fname2fnum((&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(i as isize));
            }
            i += 1;
        }
    }
    let mut to: c_int = 0;
    let mut from: c_int = 0;
    while from < (*wp).w_jumplistlen {
        if (*wp).w_jumplistidx == from {
            (*wp).w_jumplistidx = to;
        }
        i = from + 1;
        while i < (*wp).w_jumplistlen {
            if (*wp).w_jumplist[i as usize].fmark.fnum == (*wp).w_jumplist[from as usize].fmark.fnum
                && (*wp).w_jumplist[from as usize].fmark.fnum != 0
                && (*wp).w_jumplist[i as usize].fmark.mark.lnum
                    == (*wp).w_jumplist[from as usize].fmark.mark.lnum
            {
                break;
            }
            i += 1;
        }
        let mut mustfree: bool = false;
        if i >= (*wp).w_jumplistlen {
            mustfree = false;
        } else if i > from + 1 {
            mustfree = jop_flags.get() & kOptJopFlagStack as c_int as c_uint == 0;
        } else {
            mustfree = true;
        }
        if mustfree {
            xfree((*wp).w_jumplist[from as usize].fname as *mut c_void);
        } else {
            if to != from {
                (*wp).w_jumplist[to as usize] = (*wp).w_jumplist[from as usize];
            }
            to += 1;
        }
        from += 1;
    }
    if (*wp).w_jumplistidx == (*wp).w_jumplistlen {
        (*wp).w_jumplistidx = to;
    }
    (*wp).w_jumplistlen = to;
    if loadfiles && (*wp).w_jumplistlen != 0 && (*wp).w_jumplistidx == (*wp).w_jumplistlen {
        let mut fm_last: *const xfmark_T =
            (&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(((*wp).w_jumplistlen - 1) as isize);
        if (*fm_last).fmark.fnum == (*curbuf.get()).handle
            && (*fm_last).fmark.mark.lnum == (*wp).w_cursor.lnum
        {
            xfree((*fm_last).fname as *mut c_void);
            (*wp).w_jumplistlen -= 1;
            (*wp).w_jumplistidx -= 1;
        }
    }
}

/// Copy the jumplist from window "from" to window "to".
pub unsafe extern "C" fn copy_jumplist(mut from: *mut win_T, mut to: *mut win_T) {
    let mut i: c_int = 0;
    while i < (*from).w_jumplistlen {
        (*to).w_jumplist[i as usize] = (*from).w_jumplist[i as usize];
        if !(*from).w_jumplist[i as usize].fname.is_null() {
            (*to).w_jumplist[i as usize].fname = xstrdup((*from).w_jumplist[i as usize].fname);
        }
        i += 1;
    }
    (*to).w_jumplistlen = (*from).w_jumplistlen;
    (*to).w_jumplistidx = (*from).w_jumplistidx;
}

/// Free items in the jumplist of window "wp".
pub unsafe extern "C" fn free_jumplist(mut wp: *mut win_T) {
    let mut i: c_int = 0;
    while i < (*wp).w_jumplistlen {
        free_xfmark((*wp).w_jumplist[i as usize]);
        i += 1;
    }
    (*wp).w_jumplistlen = 0;
}

/// print the jumplist
pub unsafe fn ex_jumps(mut _eap: *mut exarg_T) {
    cleanup_jumplist(curwin.get(), true);
    msg_ext_set_kind(c"list_cmd".as_ptr());
    msg_puts_title(gettext(c"\n jump line  col file/text".as_ptr()));
    let mut i: c_int = 0;
    while i < (*curwin.get()).w_jumplistlen && !got_int.get() {
        if (*curwin.get()).w_jumplist[i as usize].fmark.mark.lnum != 0 {
            let mut name: *mut c_char = fm_getname(
                &raw mut (*(&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
                    .offset(i as isize))
                .fmark,
                16,
            );
            if name.is_null() && i == (*curwin.get()).w_jumplistidx {
                name = xstrdup(c"-invalid-".as_ptr());
            }
            if name.is_null() || message_filtered(name) {
                xfree(name as *mut c_void);
            } else {
                msg_putchar('\n' as c_int);
                if got_int.get() {
                    xfree(name as *mut c_void);
                    break;
                } else {
                    snprintf(
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        c"%c %2d %5d %4d ".as_ptr(),
                        if i == (*curwin.get()).w_jumplistidx {
                            '>' as c_int
                        } else {
                            ' ' as c_int
                        },
                        if i > (*curwin.get()).w_jumplistidx {
                            i - (*curwin.get()).w_jumplistidx
                        } else {
                            (*curwin.get()).w_jumplistidx - i
                        },
                        (*curwin.get()).w_jumplist[i as usize].fmark.mark.lnum,
                        (*curwin.get()).w_jumplist[i as usize].fmark.mark.col,
                    );
                    msg_outtrans(IObuff.ptr() as *mut c_char, 0, false);
                    msg_outtrans(
                        name,
                        if (*curwin.get()).w_jumplist[i as usize].fmark.fnum
                            == (*curbuf.get()).handle
                        {
                            HLF_D
                        } else {
                            0
                        },
                        false,
                    );
                    xfree(name as *mut c_void);
                    os_breakcheck();
                }
            }
        }
        i += 1;
    }
    if (*curwin.get()).w_jumplistidx == (*curwin.get()).w_jumplistlen {
        msg_puts(c"\n>".as_ptr());
    }
}

pub unsafe fn ex_clearjumps(mut _eap: *mut exarg_T) {
    free_jumplist(curwin.get());
    (*curwin.get()).w_jumplistlen = 0;
    (*curwin.get()).w_jumplistidx = 0;
}

/// print the changelist
pub unsafe fn ex_changes(mut _eap: *mut exarg_T) {
    msg_ext_set_kind(c"list_cmd".as_ptr());
    msg_puts_title(gettext(c"\nchange line  col text".as_ptr()));
    let mut i: c_int = 0;
    while i < (*curbuf.get()).b_changelistlen && !got_int.get() {
        if (*curbuf.get()).b_changelist[i as usize].mark.lnum != 0 {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                break;
            }
            snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                c"%c %3d %5d %4d ".as_ptr(),
                if i == (*curwin.get()).w_changelistidx {
                    '>' as c_int
                } else {
                    ' ' as c_int
                },
                if i > (*curwin.get()).w_changelistidx {
                    i - (*curwin.get()).w_changelistidx
                } else {
                    (*curwin.get()).w_changelistidx - i
                },
                (*curbuf.get()).b_changelist[i as usize].mark.lnum,
                (*curbuf.get()).b_changelist[i as usize].mark.col,
            );
            msg_outtrans(IObuff.ptr() as *mut c_char, 0, false);
            let mut name: *mut c_char = mark_line(
                &raw mut (*(&raw mut (*curbuf.get()).b_changelist as *mut fmark_T)
                    .offset(i as isize))
                .mark,
                17,
            );
            msg_outtrans(name, HLF_D, false);
            xfree(name as *mut c_void);
            os_breakcheck();
        }
        i += 1;
    }
    if (*curwin.get()).w_changelistidx == (*curbuf.get()).b_changelistlen {
        msg_puts(c"\n>".as_ptr());
    }
}

/// Iterate over jumplist items
///
/// @warning No jumplist-editing functions must be called while iteration is in
///          progress.
///
/// `iter` — Iterator. Pass NULL to start iteration.
/// `win` — Window for which jump list is processed.
/// `fm` — Item definition.
///
/// Returns pointer that needs to be passed to next `mark_jumplist_iter` call or
///         NULL if iteration is over.
pub unsafe extern "C" fn mark_jumplist_iter(
    iter: *const c_void,
    win: *const win_T,
    fm: *mut xfmark_T,
) -> *const c_void {
    if iter.is_null() && (*win).w_jumplistlen == 0 {
        *fm = xfmark_T {
            fmark: fmark_T {
                mark: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                fnum: 0,
                timestamp: 0 as Timestamp,
                view: fmarkv_T {
                    topline_offset: MAXLNUM as c_int,
                    skipcol: 0,
                },
                additional_data: ptr::null_mut(),
            },
            fname: ptr::null_mut(),
        };
        return ptr::null();
    }
    let iter_mark: *const xfmark_T = if iter.is_null() {
        (&raw const (*win).w_jumplist as *const xfmark_T).offset(0)
    } else {
        iter as *const xfmark_T
    };
    *fm = *iter_mark;
    if iter_mark
        == (&raw const (*win).w_jumplist as *const xfmark_T)
            .offset(((*win).w_jumplistlen - 1) as isize)
    {
        return ptr::null();
    }
    return iter_mark.offset(1) as *const c_void;
}
