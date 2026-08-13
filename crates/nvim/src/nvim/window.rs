//! Windows, frames and tab pages: the layout the editor draws into.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`cmd`] | `do_window()` -- the CTRL-W commands |
//! | [`config`] | which buffer a window shows, and what the UI is told |
//! | [`split`] | `win_split_ins()` -- making a new window |
//! | [`order`] | exchange, rotate, move to an edge |
//! | [`equal`] | `win_equal()` and `'equalalways'` |
//! | [`close`] | may this window close, and `:only` |
//! | [`winclose`] | `win_close()` -- closing one window |
//! | [`frame`] | the frame tree: remove a leaf, give its room away |
//! | [`framesize`] | frame arithmetic and the minimum sizes |
//! | [`alloc`] | allocating windows and frames, and the lists they live on |
//! | [`tabpage`] | tab pages -- create, switch, close |
//! | [`goto`] | entering a window, and the directional moves |
//! | [`screensize`] | the screen resized, and `WinScrolled`/`WinResized` |
//! | [`size`] | `:resize`, and dragging a separator |
//! | [`resize`] | applying a size, and the rows that are not text |
//! | [`snapshot`] | saving and restoring a layout, and revalidation |
//!
//! What stays here is the constant alphabet the sixteen share (`WSP_*`,
//! `FR_*`, `WEE_*`, `SNAP_*`), the layout locks (`window_layout_lock` and the
//! `split_disallowed`/`close_disallowed`/`frame_locked` counters that back
//! them), the window-handle lookups every caller starts from (`win_valid`,
//! `win_valid_any_tab`, `win_find_by_handle`, `win_count`), and the small
//! globals the children read (`last_win_id`, `min_set_ch`,
//! `command_frame_height`).
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{api_clear_error, api_set_error};
use crate::src::nvim::ex_getln::is_in_cmdwin;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    c_bytes, curtab, curwin, e_not_allowed_to_change_window_layout_in_this_autocmd,
    e_winfixbuf_cannot_go_to_buffer, first_tabpage, firstwin, prevwin, swb_flags,
};
use crate::src::nvim::map::map_put_ref_int_ptr_t;
use crate::src::nvim::message::emsg;
use crate::src::nvim::options::{kOptSwbFlagUseopen, kOptSwbFlagUsetab};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::{
    AlignTextPos, CMD_tabnew, CdCause, Direction, Error, Map_int_ptr_t, MapHash, MotionType,
    OptInt, OptValType, Set_uint32_t, WinSplit, WinStyle, aucmdwin_T, bln_values, buf_T,
    cmd_addr_T, cmdidx_T, dobuf_action_values, dobuf_start_values, getf_values, handle_T,
    kErrorTypeException, kErrorTypeNone, ptr_t, size_t, tabpage_T, uint32_t, win_T,
};

// The carve of the transpiled module; see each child's docs.
mod alloc;
mod close;
mod cmd;
mod config;
mod equal;
mod frame;
mod framesize;
mod goto;
mod order;
mod resize;
mod screensize;
mod size;
mod snapshot;
mod split;
mod tabpage;
mod winclose;

pub use self::alloc::*;
pub use self::close::*;
pub use self::cmd::*;
pub use self::config::*;
pub use self::equal::*;
pub use self::frame::*;
pub use self::framesize::*;
pub use self::goto::*;
pub use self::order::*;
pub use self::resize::*;
pub use self::screensize::*;
pub use self::size::*;
pub use self::snapshot::*;
pub use self::split::*;
pub use self::tabpage::*;
pub use self::winclose::*;

unsafe extern "C" {}
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kZIndexMessages: C2Rust_Unnamed_14 = 200;
pub const kZIndexFloatDefault: C2Rust_Unnamed_14 = 50;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub const kDirectionNotSet: Direction = 0;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const kWinOptScroll: C2Rust_Unnamed_16 = 33;
pub const kWinOptFoldtext: C2Rust_Unnamed_16 = 22;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_24 = 4;
pub const BL_SOL: C2Rust_Unnamed_24 = 2;
pub const BL_WHITE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_25 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_25 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_25 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_25 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_25 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_25 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_26 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_26 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_26 = 0;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const FIND_EVAL: C2Rust_Unnamed_28 = 4;
pub const FIND_STRING: C2Rust_Unnamed_28 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const BCO_NOHELP: C2Rust_Unnamed_29 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_30 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_30 = 2;
pub const FIND_ANY: C2Rust_Unnamed_30 = 1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_31 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_31 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_31 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_31 = 1;
pub const WSP_QUICKFIX: ::core::ffi::c_uint = 1024;
pub const WSP_NOENTER: ::core::ffi::c_uint = 512;
pub const WSP_NEWLOC: ::core::ffi::c_uint = 256;
pub const WSP_ABOVE: ::core::ffi::c_uint = 128;
pub const WSP_BELOW: ::core::ffi::c_uint = 64;
pub const WSP_HELP: ::core::ffi::c_uint = 32;
pub const WSP_BOT: ::core::ffi::c_uint = 16;
pub const WSP_TOP: ::core::ffi::c_uint = 8;
pub const WSP_HOR: ::core::ffi::c_uint = 4;
pub const WSP_VERT: ::core::ffi::c_uint = 2;
pub const WSP_ROOM: ::core::ffi::c_uint = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const STATUS_HEIGHT: C2Rust_Unnamed_33 = 1;
pub const MIN_LINES: C2Rust_Unnamed_33 = 2;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const LOWEST_WIN_ID: C2Rust_Unnamed_34 = 1000;
pub const WEE_TRIGGER_LEAVE_AUTOCMDS: C2Rust_Unnamed_35 = 16;
pub const WEE_TRIGGER_ENTER_AUTOCMDS: C2Rust_Unnamed_35 = 8;
pub const WEE_UNDO_SYNC: C2Rust_Unnamed_35 = 1;
pub const WEE_TRIGGER_NEW_AUTOCMDS: C2Rust_Unnamed_35 = 4;
pub const WEE_CURWIN_INVALID: C2Rust_Unnamed_35 = 2;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SNAP_HELP_IDX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SNAP_QUICKFIX_IDX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SNAP_COUNT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
#[inline]
unsafe extern "C" fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    unsafe {
        let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
            map,
            key,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = 9;
pub const CAR: ::core::ffi::c_int = 13;
pub const SID_WINLAYOUT: ::core::ffi::c_int = -7 as ::core::ffi::c_int;
pub const NOWIN: *mut win_T = -1 as ::core::ffi::c_int as *mut win_T;
static e_cannot_close_last_window: [::core::ffi::c_char; 31] =
    c_bytes(b"E444: Cannot close last window\0");
static e_cannot_split_window_when_closing_buffer: [::core::ffi::c_char; 53] =
    c_bytes(b"E1159: Cannot split a window when closing the buffer\0");
static m_onlyone: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(c"Already only one window".as_ptr() as *mut ::core::ffi::c_char);
static split_disallowed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static close_disallowed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static frame_locked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn window_layout_lock() {
    unsafe {
        (*split_disallowed.ptr()) += 1;
        (*close_disallowed.ptr()) += 1;
    }
}
pub unsafe extern "C" fn window_layout_unlock() {
    unsafe {
        (*split_disallowed.ptr()) -= 1;
        (*close_disallowed.ptr()) -= 1;
    }
}
pub unsafe extern "C" fn frames_locked() -> bool {
    return frame_locked.get() != 0;
}
pub unsafe extern "C" fn window_layout_locked(mut cmd: cmdidx_T) -> bool {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let locked: bool = window_layout_locked_err(cmd, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            emsg(gettext(err.msg));
            api_clear_error(&raw mut err);
        }
        return locked;
    }
}
pub unsafe extern "C" fn window_layout_locked_err(mut cmd: cmdidx_T, mut err: *mut Error) -> bool {
    unsafe {
        if split_disallowed.get() > 0 as ::core::ffi::c_int
            || close_disallowed.get() > 0 as ::core::ffi::c_int
        {
            if close_disallowed.get() == 0 as ::core::ffi::c_int
                && cmd as ::core::ffi::c_int == CMD_tabnew as ::core::ffi::c_int
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"%s".as_ptr(),
                    e_cannot_split_window_when_closing_buffer.as_ptr(),
                );
            } else {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"%s".as_ptr(),
                    &raw const e_not_allowed_to_change_window_layout_in_this_autocmd
                        as *const ::core::ffi::c_char,
                );
            }
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}
pub unsafe extern "C" fn check_can_set_curbuf_disabled() -> bool {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
            emsg(gettext(
                &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}
pub unsafe extern "C" fn check_can_set_curbuf_forceit(mut forceit: ::core::ffi::c_int) -> bool {
    unsafe {
        if forceit == 0 && (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
            emsg(gettext(
                &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}
pub unsafe extern "C" fn prevwin_curwin() -> *mut win_T {
    unsafe {
        return if is_in_cmdwin() as ::core::ffi::c_int != 0 && !(*prevwin.ptr()).is_null() {
            prevwin.get()
        } else {
            curwin.get()
        };
    }
}
pub unsafe extern "C" fn swbuf_goto_win_with_buf(mut buf: *mut buf_T) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if buf.is_null() {
            return wp;
        }
        if swb_flags.get() & kOptSwbFlagUseopen as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            wp = buf_jump_open_win(buf);
        }
        if wp.is_null()
            && swb_flags.get() & kOptSwbFlagUsetab as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            wp = buf_jump_open_tab(buf);
        }
        return wp;
    }
}
static min_set_ch: GlobalCell<OptInt> = GlobalCell::new(1 as OptInt);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_valid(mut win: *const win_T) -> bool {
    unsafe {
        return tabpage_win_valid(curtab.get(), win);
    }
}
pub unsafe extern "C" fn tabpage_win_valid(
    mut tp: *const tabpage_T,
    mut win: *const win_T,
) -> bool {
    unsafe {
        if win.is_null() {
            return false_0 != 0;
        }
        let mut wp: *mut win_T = if tp == curtab.get() as *const tabpage_T {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if wp == win as *mut win_T {
                return true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        return false_0 != 0;
    }
}
pub unsafe extern "C" fn win_find_by_handle(mut handle: handle_T) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).handle == handle {
                return wp;
            }
            wp = (*wp).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}
pub unsafe extern "C" fn win_valid_any_tab(mut win: *mut win_T) -> bool {
    unsafe {
        if win.is_null() {
            return false_0 != 0;
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if wp == win {
                    return true_0 != 0;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return false_0 != 0;
    }
}
pub unsafe extern "C" fn win_count() -> ::core::ffi::c_int {
    unsafe {
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            count += 1;
            wp = (*wp).w_next;
        }
        return count;
    }
}
static command_frame_height: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static last_win_id: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(LOWEST_WIN_ID as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
static did_initial_scroll_size_snapshot: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const FRACTION_MULT: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
