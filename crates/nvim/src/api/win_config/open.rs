//! `nvim_open_win()`: creating a window from a config.
//!
//! The entry point for both kinds of window the config describes: a float,
//! which is created directly, and a split, which goes through `win_split_dir`
//! and `win_split_flags` to turn the `split`/`vertical` keys into the
//! `WSP_*` flags `win_split_ins` takes.  `win_can_move_tp` and
//! `win_find_altwin` are the checks a window has to pass before it can be
//! moved to another tabpage.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};

pub unsafe fn nvim_open_win(
    buf: Buffer,
    enter: Boolean,
    config: *mut KeyDict_win_config,
) -> Result<Window, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut bufref: bufref_T = bufref_T::default();
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return (0 as Window).reported(error);
        }
        if cmdwin_type.get() != 0 as ::core::ffi::c_int && enter as ::core::ffi::c_int != 0
            || b == cmdwin_buf.get()
        {
            api_set_error(
                err,
                kErrorTypeException,
                c"%s".as_ptr(),
                &raw const e_cmdwin as *const ::core::ffi::c_char,
            );
            return (0 as Window).reported(error);
        }
        let mut fconfig: WinConfig = WinConfig {
            window: 0,
            bufpos: lpos_T {
                lnum: -1 as linenr_T,
                col: 0 as colnr_T,
            },
            height: 0 as ::core::ffi::c_int,
            width: 0 as ::core::ffi::c_int,
            row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
            col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
            anchor: 0 as FloatAnchor,
            relative: kFloatRelativeEditor,
            external: false,
            focusable: true,
            mouse: true,
            split: kWinSplitLeft,
            zindex: kZIndexFloatDefault as ::core::ffi::c_int,
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
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            title_width: 0,
            footer: false,
            footer_pos: kAlignLeft,
            footer_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            footer_width: 0,
            noautocmd: false,
            fixed: false,
            hide: false,
            _cmdline_offset: INT_MAX,
        };
        if !parse_win_config(
            ::core::ptr::null_mut::<win_T>(),
            config,
            &raw mut fconfig,
            false,
            err,
        ) {
            return (0 as Window).reported(error);
        }
        let keys_set = (*config).is_set__win_config_;
        let mut is_split = has_key(keys_set, KEYSET_OPTIDX_win_config__split)
            || has_key(keys_set, KEYSET_OPTIDX_win_config__vertical);
        let mut rv: Window = 0 as Window;
        // Read before the config is handed to the window: whichever branch
        // below runs moves it, and both of these are wanted afterwards.
        let noautocmd = fconfig.noautocmd;
        let style = fconfig.style;
        let cmdline_offset = fconfig._cmdline_offset;
        if noautocmd {
            block_autocmds();
        }
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut tp: *mut tabpage_T = curtab.get();
        debug_assert!(!curwin.get().is_null(), "curwin != NULL");
        let mut parent: *mut win_T = if (*config).win == 0 as ::core::ffi::c_int {
            curwin.get()
        } else {
            ::core::ptr::null_mut::<win_T>()
        };
        '_cleanup: {
            if (*config).win > 0 as ::core::ffi::c_int {
                parent = find_window_by_handle(fconfig.window, err);
                if parent.is_null() {
                    break '_cleanup;
                } else if is_split as ::core::ffi::c_int != 0
                    && (*parent).w_floating as ::core::ffi::c_int != 0
                {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        c"Cannot split a floating window".as_ptr(),
                    );
                    break '_cleanup;
                }
                tp = win_find_tabpage(parent);
            }
            if is_split {
                if !check_split_disallowed_err(
                    if !parent.is_null() {
                        parent
                    } else {
                        curwin.get()
                    },
                    err,
                ) {
                    break '_cleanup;
                }
                if has_key(keys_set, KEYSET_OPTIDX_win_config__vertical)
                    && !has_key(keys_set, KEYSET_OPTIDX_win_config__split)
                {
                    if (*config).vertical {
                        fconfig.split = (if p_spr.get() != 0 {
                            kWinSplitRight as ::core::ffi::c_int
                        } else {
                            kWinSplitLeft as ::core::ffi::c_int
                        }) as WinSplit;
                    } else {
                        fconfig.split = (if p_sb.get() != 0 {
                            kWinSplitBelow as ::core::ffi::c_int
                        } else {
                            kWinSplitAbove as ::core::ffi::c_int
                        }) as WinSplit;
                    }
                }
                let mut flags: ::core::ffi::c_int =
                    win_split_flags(fconfig.split, parent.is_null())
                        | WSP_NOENTER as ::core::ffi::c_int;
                let mut size: ::core::ffi::c_int = if flags & WSP_VERT as ::core::ffi::c_int != 0 {
                    fconfig.width
                } else {
                    fconfig.height
                };
                let mut tstate: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate);
                if parent.is_null() || parent == curwin.get() {
                    wp = win_split_ins(
                        size,
                        flags,
                        ::core::ptr::null_mut::<win_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<frame_T>(),
                    );
                } else {
                    let mut switchwin: switchwin_T = switchwin_T {
                        sw_curwin: ::core::ptr::null_mut::<win_T>(),
                        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                        sw_same_win: false,
                        sw_visual_active: false,
                    };
                    let result: ::core::ffi::c_int =
                        switch_win(&raw mut switchwin, parent, tp, true);
                    debug_assert!(result == 1 as ::core::ffi::c_int, "result == OK");
                    wp = win_split_ins(
                        size,
                        flags,
                        ::core::ptr::null_mut::<win_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<frame_T>(),
                    );
                    restore_win(&raw mut switchwin, true);
                }
                try_leave(&raw mut tstate, err);
                if !wp.is_null() {
                    (*wp).w_config = fconfig;
                    if size > 0 as ::core::ffi::c_int {
                        if flags & WSP_VERT as ::core::ffi::c_int != 0 && (*wp).w_width != size {
                            win_setwidth_win(size, wp);
                        } else if flags & WSP_VERT as ::core::ffi::c_int == 0
                            && (*wp).w_height != size
                        {
                            win_setheight_win(size, wp);
                        }
                    }
                }
            } else if (*(*curwin.get()).w_buffer).b_locked_split != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"E1159: Cannot open a float when closing the buffer".as_ptr(),
                );
                break '_cleanup;
            } else {
                wp = win_new_float(::core::ptr::null_mut::<win_T>(), false, fconfig, err);
            }
            if wp.is_null() {
                if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        c"Failed to create window".as_ptr(),
                    );
                }
            } else {
                if cmdline_offset < INT_MAX {
                    cmdline_win.set(wp);
                }
                bufref = bufref_T::default();
                set_bufref(&raw mut bufref, b);
                if !noautocmd {
                    let mut switchwin_0: switchwin_T = switchwin_T {
                        sw_curwin: ::core::ptr::null_mut::<win_T>(),
                        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                        sw_same_win: false,
                        sw_visual_active: false,
                    };
                    let result_0: ::core::ffi::c_int =
                        switch_win_noblock(&raw mut switchwin_0, wp, tp, true);
                    debug_assert!(result_0 == 1 as ::core::ffi::c_int, "result == OK");
                    if apply_autocmds(
                        EVENT_WINNEW,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false,
                        curbuf.get(),
                    ) {
                        tp = win_find_tabpage(wp);
                    }
                    restore_win_noblock(&raw mut switchwin_0, true);
                }
                if !tp.is_null() && enter as ::core::ffi::c_int != 0 {
                    goto_tabpage_win(tp, wp);
                    tp = win_find_tabpage(wp);
                }
                if !tp.is_null()
                    && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                    && b != (*wp).w_buffer
                {
                    let au_no_enter_leave: bool = curwin.get() != wp && !noautocmd;
                    if au_no_enter_leave {
                        (*autocmd_no_enter.ptr()) += 1;
                        (*autocmd_no_leave.ptr()) += 1;
                    }
                    win_set_buf(wp, b, err);
                    if !noautocmd {
                        tp = win_find_tabpage(wp);
                    }
                    if au_no_enter_leave {
                        (*autocmd_no_enter.ptr()) -= 1;
                        (*autocmd_no_leave.ptr()) -= 1;
                    }
                }
                if tp.is_null() {
                    api_clear_error(err);
                    api_set_error(
                        err,
                        kErrorTypeException,
                        c"Window was closed immediately".as_ptr(),
                    );
                } else {
                    if style as ::core::ffi::c_uint
                        == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        win_set_minimal_style(wp);
                        didset_window_options(wp, true);
                        changed_window_setting(wp);
                    }
                    rv = (*wp).handle as Window;
                }
            }
        }
        if noautocmd {
            unblock_autocmds();
        }
        rv.reported(error)
    }
}

pub(crate) unsafe fn win_split_dir(win: *mut win_T) -> WinSplit {
    unsafe {
        if (*win).w_frame.is_null() || (*(*win).w_frame).fr_parent.is_null() {
            return kWinSplitLeft;
        }
        // A window in a column was split off the one below it when it still
        // has a sibling ahead of it, and off the one above it otherwise; in a
        // row the same test reads left/right.
        let column = (*(*(*win).w_frame).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL;
        match ((*(*win).w_frame).fr_next.is_null(), column) {
            (false, true) => kWinSplitAbove,
            (true, true) => kWinSplitBelow,
            (false, false) => kWinSplitLeft,
            (true, false) => kWinSplitRight,
        }
    }
}

pub(crate) fn win_split_flags(mut split: WinSplit, mut toplevel: bool) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= WSP_HOR as ::core::ffi::c_int;
    } else {
        flags |= WSP_VERT as ::core::ffi::c_int;
    }
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitLeft as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_TOP as ::core::ffi::c_int
        } else {
            WSP_ABOVE as ::core::ffi::c_int
        };
    } else {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_BOT as ::core::ffi::c_int
        } else {
            WSP_BELOW as ::core::ffi::c_int
        };
    }
    flags
}

pub(crate) unsafe fn win_can_move_tp(
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
    mut err: *mut Error,
) -> bool {
    unsafe {
        if one_window(
            wp,
            if tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                tp
            },
        ) {
            api_set_error(
                err,
                kErrorTypeException,
                c"Cannot move last non-floating window".as_ptr(),
            );
            return false;
        }
        if win_locked(wp) != 0 {
            api_set_error(
                err,
                kErrorTypeException,
                c"Cannot move window to another tabpage whilst in use".as_ptr(),
            );
            return false;
        }
        if window_layout_locked_err(CMD_SIZE, err) {
            return false;
        }
        if textlock.get() != 0 || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                err,
                kErrorTypeException,
                c"%s".as_ptr(),
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
            return false;
        }
        if is_aucmd_win(wp) {
            api_set_error(
                err,
                kErrorTypeException,
                c"Cannot move autocmd window to another tabpage".as_ptr(),
            );
            return false;
        }
        if wp == cmdwin_win.get() || wp == cmdwin_old_curwin.get() {
            api_set_error(
                err,
                kErrorTypeException,
                c"%s".as_ptr(),
                &raw const e_cmdwin as *const ::core::ffi::c_char,
            );
            return false;
        }
        true
    }
}

pub(crate) unsafe fn win_find_altwin(mut win: *mut win_T, mut tp: *mut tabpage_T) -> *mut win_T {
    unsafe {
        if (*win).w_floating {
            win_float_find_altwin(
                win,
                if tp == curtab.get() {
                    ::core::ptr::null_mut::<tabpage_T>()
                } else {
                    tp
                },
            )
        } else {
            let mut dir: ::core::ffi::c_int = 0;
            winframe_find_altwin(
                win,
                &raw mut dir,
                if tp == curtab.get() {
                    ::core::ptr::null_mut::<tabpage_T>()
                } else {
                    tp
                },
                ::core::ptr::null_mut::<*mut frame_T>(),
            )
        }
    }
}
