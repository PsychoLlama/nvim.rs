//! `:all` and `:sall`: lay the argument list out over windows.

use super::*;
use crate::src::nvim::window::{WSP_BELOW, WSP_ROOM};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct arg_all_state_T {
    pub alist: *mut alist_T,
    pub had_tab: c_int,
    pub keep_tabs: bool,
    pub forceit: bool,
    pub use_firstwin: bool,
    pub opened: *mut uint8_t,
    pub opened_len: c_int,
    pub new_curwin: *mut win_T,
    pub new_curtab: *mut tabpage_T,
}
unsafe extern "C" fn arg_all_close_unused_windows(mut aall: *mut arg_all_state_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curtab: *mut tabpage_T = curtab.get();
    if (*aall).had_tab > 0 {
        goto_tabpage_tp(first_tabpage.get(), true, true);
    }
    (*tabpage_move_disallowed.ptr()) += 1;
    loop {
        let mut wpnext: *mut win_T = ptr::null_mut();
        let mut tpnext: *mut tabpage_T = (*curtab.get()).tp_next;
        let mut wp: *mut win_T = if (*lastwin.get()).w_floating {
            lastwin.get()
        } else {
            firstwin.get()
        };
        while !wp.is_null() {
            let mut i: c_int = 0;
            wpnext = if (*wp).w_floating {
                if (*(*wp).w_prev).w_floating {
                    (*wp).w_prev
                } else {
                    firstwin.get()
                }
            } else if (*wp).w_next.is_null() || (*(*wp).w_next).w_floating {
                ptr::null_mut()
            } else {
                (*wp).w_next
            };
            let mut buf: *mut buf_T = (*wp).w_buffer;
            if (*buf).b_ffname.is_null()
                || !(*aall).keep_tabs
                    && ((*buf).b_nwindows > 1
                        || (*wp).w_width != Columns.get()
                        || (*wp).w_floating && !is_aucmd_win(wp))
            {
                i = (*aall).opened_len;
            } else {
                i = 0;
                while i < (*aall).opened_len {
                    if i < alist_count((*aall).alist)
                        && ((*alist_arg((*aall).alist, i)).ae_fnum == (*buf).handle
                            || path_full_compare(
                                alist_name(alist_arg((*aall).alist, i)),
                                (*buf).b_ffname,
                                true,
                                true,
                            ) as c_uint
                                & kEqualFiles as c_int as c_uint
                                != 0)
                    {
                        let mut weight: c_int = 1;
                        if old_curtab == curtab.get() {
                            weight += 1;
                            if old_curwin == wp {
                                weight += 1;
                            }
                        }
                        if weight > *(*aall).opened.offset(i as isize) as c_int {
                            *(*aall).opened.offset(i as isize) = weight as uint8_t;
                            if i == 0 {
                                if !(*aall).new_curwin.is_null() {
                                    (*(*aall).new_curwin).w_arg_idx = (*aall).opened_len;
                                }
                                (*aall).new_curwin = wp;
                                (*aall).new_curtab = curtab.get();
                            }
                        } else if (*aall).keep_tabs {
                            i = (*aall).opened_len;
                        }
                        if (*wp).w_alist != (*aall).alist {
                            alist_unlink((*wp).w_alist);
                            (*wp).w_alist = (*aall).alist;
                            (*(*wp).w_alist).al_refcount += 1;
                        }
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            (*wp).w_arg_idx = i;
            's_31: {
                if i == (*aall).opened_len && !(*aall).keep_tabs {
                    if buf_hide(buf)
                        || (*aall).forceit
                        || (*buf).b_nwindows > 1
                        || !bufIsChanged(buf)
                    {
                        if !buf_hide(buf) && (*buf).b_nwindows <= 1 && bufIsChanged(buf) {
                            let mut bufref: bufref_T = bufref_T::default();
                            set_bufref(&raw mut bufref, buf);
                            autowrite(buf, false);
                            if !win_valid(wp) || !bufref_valid(&raw mut bufref) {
                                wpnext = if (*lastwin.get()).w_floating {
                                    lastwin.get()
                                } else {
                                    firstwin.get()
                                };
                                break 's_31;
                            }
                        }
                        if firstwin.get() == lastwin.get()
                            && ((*first_tabpage.get()).tp_next.is_null() || (*aall).had_tab == 0)
                        {
                            (*aall).use_firstwin = true;
                        } else {
                            win_close(wp, !buf_hide(buf) && !bufIsChanged(buf), false);
                            if !win_valid(wpnext) {
                                wpnext = if (*lastwin.get()).w_floating {
                                    lastwin.get()
                                } else {
                                    firstwin.get()
                                };
                            }
                        }
                    }
                }
            }
            wp = wpnext;
        }
        if (*aall).had_tab == 0 || tpnext.is_null() {
            break;
        }
        if !valid_tabpage(tpnext) {
            tpnext = first_tabpage.get();
        }
        goto_tabpage_tp(tpnext, true, true);
    }
    (*tabpage_move_disallowed.ptr()) -= 1;
}
unsafe extern "C" fn arg_all_open_windows(mut aall: *mut arg_all_state_T, mut count: c_int) {
    let mut tab_drop_empty_window: bool = false;
    if (*aall).keep_tabs
        && buf_is_empty(curbuf.get())
        && (*curbuf.get()).b_nwindows == 1
        && (*curbuf.get()).b_ffname.is_null()
        && (*curbuf.get()).b_changed == 0
    {
        (*aall).use_firstwin = true;
        tab_drop_empty_window = true;
    }
    let mut split_ret: c_int = OK;
    let mut i: c_int = 0;
    while i < count && !got_int.get() {
        if (*aall).alist == global_alist.ptr() && i == alist_count(global_alist.ptr()) - 1 {
            arg_had_last.set(true);
        }
        's_23: {
            if *(*aall).opened.offset(i as isize) as c_int > 0 {
                if (*curwin.get()).w_arg_idx != i {
                    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                        firstwin.get()
                    } else {
                        (*curtab.get()).tp_firstwin
                    };
                    while !wp.is_null() {
                        if (*wp).w_arg_idx == i {
                            if (*aall).keep_tabs {
                                (*aall).new_curwin = wp;
                                (*aall).new_curtab = curtab.get();
                                break;
                            } else {
                                if (*wp).w_floating {
                                    break;
                                }
                                if (*(*wp).w_frame).fr_parent
                                    != (*(*curwin.get()).w_frame).fr_parent
                                {
                                    emsg(gettext(
                                        c"E249: Window layout changed unexpectedly".as_ptr(),
                                    ));
                                    i = count;
                                    break;
                                } else {
                                    win_move_after(wp, curwin.get());
                                    break;
                                }
                            }
                        } else {
                            wp = (*wp).w_next;
                        }
                    }
                }
            } else if split_ret == OK {
                if tab_drop_empty_window && i == count - 1 {
                    (*autocmd_no_enter.ptr()) -= 1;
                }
                if !(*aall).use_firstwin {
                    let mut p_ea_save: bool = p_ea.get() != 0;
                    p_ea.set(true_0);
                    split_ret = win_split(0, WSP_ROOM as c_int | WSP_BELOW as c_int);
                    p_ea.set(p_ea_save as c_int);
                    if split_ret == FAIL {
                        break 's_23;
                    }
                } else {
                    (*autocmd_no_leave.ptr()) -= 1;
                }
                (*curwin.get()).w_arg_idx = i;
                if i == 0 {
                    (*aall).new_curwin = curwin.get();
                    (*aall).new_curtab = curtab.get();
                }
                do_ecmd(
                    0,
                    alist_name(alist_arg((*aall).alist, i)),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ECMD_ONE as c_int as linenr_T,
                    (if buf_hide((*curwin.get()).w_buffer) || bufIsChanged((*curwin.get()).w_buffer)
                    {
                        ECMD_HIDE as c_int
                    } else {
                        0
                    }) + ECMD_OLDBUF as c_int,
                    curwin.get(),
                );
                if tab_drop_empty_window && i == count - 1 {
                    (*autocmd_no_enter.ptr()) += 1;
                }
                if (*aall).use_firstwin {
                    (*autocmd_no_leave.ptr()) += 1;
                }
                (*aall).use_firstwin = false;
            }
            os_breakcheck();
            if (*aall).had_tab > 0 && tabpage_index(ptr::null_mut()) as OptInt <= p_tpm.get() {
                (*cmdmod.ptr()).cmod_tab = 9999;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn do_arg_all(mut count: c_int, mut forceit: c_int, mut keep_tabs: c_int) {
    let mut last_curwin: *mut win_T = ptr::null_mut();
    let mut last_curtab: *mut tabpage_T = ptr::null_mut();
    let mut prev_arglist_locked: bool = arglist_locked.get();
    '_c2rust_label: {
        if !(*firstwin.ptr()).is_null() {
        } else {
            __assert_fail(
                c"firstwin != NULL".as_ptr(),
                c"src/nvim/arglist.rs".as_ptr(),
                1068,
                c"void do_arg_all(int, int, int)".as_ptr(),
            );
        }
    };
    if cmdwin_type.get() != 0 {
        emsg(gettext(&raw const e_cmdwin as *const c_char));
        return;
    }
    if argcount() <= 0 {
        return;
    }
    setpcmark();
    let mut aall: arg_all_state_T = arg_all_state_T {
        alist: ptr::null_mut(),
        had_tab: (*cmdmod.ptr()).cmod_tab,
        keep_tabs: keep_tabs != 0,
        forceit: forceit != 0,
        use_firstwin: false,
        opened: xcalloc(argcount() as size_t, 1 as size_t) as *mut uint8_t,
        opened_len: argcount(),
        new_curwin: ptr::null_mut(),
        new_curtab: ptr::null_mut(),
    };
    aall.alist = (*curwin.get()).w_alist;
    (*aall.alist).al_refcount += 1;
    arglist_locked.set(true);
    let new_lu_tp: *mut tabpage_T = curtab.get();
    reset_VIsual_and_resel();
    arg_all_close_unused_windows(&raw mut aall);
    if count > aall.opened_len || count <= 0 {
        count = aall.opened_len;
    }
    (*autocmd_no_enter.ptr()) += 1;
    (*autocmd_no_leave.ptr()) += 1;
    last_curwin = curwin.get();
    last_curtab = curtab.get();
    win_enter(lastwin_nofloating(ptr::null_mut()), false);
    arg_all_open_windows(&raw mut aall, count);
    alist_unlink(aall.alist);
    arglist_locked.set(prev_arglist_locked);
    (*autocmd_no_enter.ptr()) -= 1;
    if last_curtab != aall.new_curtab {
        if valid_tabpage(last_curtab) {
            goto_tabpage_tp(last_curtab, true, true);
        }
        if win_valid(last_curwin) {
            win_enter(last_curwin, false);
        }
    }
    if valid_tabpage(aall.new_curtab) {
        goto_tabpage_tp(aall.new_curtab, true, true);
    }
    if valid_tabpage(new_lu_tp) {
        lastused_tabpage.set(new_lu_tp);
    }
    if win_valid(aall.new_curwin) {
        win_enter(aall.new_curwin, false);
    }
    (*autocmd_no_leave.ptr()) -= 1;
    xfree(aall.opened as *mut c_void);
}
pub unsafe fn ex_all(mut eap: *mut exarg_T) {
    if (*eap).addr_count == 0 {
        (*eap).line2 = 9999 as linenr_T;
    }
    do_arg_all(
        (*eap).line2 as c_int,
        (*eap).forceit,
        ((*eap).cmdidx as c_int == CMD_drop as c_int) as c_int,
    );
}
pub unsafe extern "C" fn arg_all() -> *mut c_char {
    let mut retval: *mut c_char = ptr::null_mut();
    loop {
        let mut len: c_int = 0;
        let mut idx: c_int = 0;
        while idx < argcount() {
            let mut p: *mut c_char = arg_name(idx);
            if !p.is_null() {
                if len > 0 {
                    if !retval.is_null() {
                        *retval.offset(len as isize) = ' ' as c_char;
                    }
                    len += 1;
                }
                while *p as c_int != NUL {
                    if *p as c_int == ' ' as c_int
                        || *p as c_int == '\\' as c_int
                        || *p as c_int == '`' as c_int
                    {
                        if !retval.is_null() {
                            *retval.offset(len as isize) = '\\' as c_char;
                        }
                        len += 1;
                    }
                    if !retval.is_null() {
                        *retval.offset(len as isize) = *p;
                    }
                    len += 1;
                    p = p.offset(1);
                }
            }
            idx += 1;
        }
        if !retval.is_null() {
            *retval.offset(len as isize) = NUL as c_char;
            break;
        } else {
            retval = xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut c_char;
        }
    }
    return retval;
}
