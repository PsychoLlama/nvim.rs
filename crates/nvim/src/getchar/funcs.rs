//! The Vimscript `getchar()` family.
//!
//! [`getchar_common`] backs `getchar()`, `getcharstr()` and `getcharmod()`:
//! it reads one key with mappings disabled, optionally without blocking, and
//! renders it as a number, a string or a modifier mask.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{K_IGNORE, K_MOUSEMOVE, key_escape};
#[allow(unused_imports)]
use crate::semsg_c;
use crate::types::{VAR_DICT, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};
use core::ffi::{c_char, c_int};
use core::ptr;

/// What the `cursor` option asked for while the key is awaited.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorFlag {
    /// No option given: move the cursor to the message only if something has
    /// already been written there.
    Default,
    /// `"hide"`: show the busy cursor.
    Hide,
    /// `"keep"`: leave the cursor where it is.
    Keep,
    /// `"msg"`: move it to the message area.
    Msg,
}

/// The options dictionary `getchar()` and `getcharstr()` accept.
struct GetcharOpts {
    /// Whether an ordinary character may be answered as a number.
    allow_number: bool,
    /// Whether a Ctrl modifier may be folded into the key.
    simplify: bool,
    cursor: CursorFlag,
}

/// Read the `{opts}` dictionary, if there is one.
///
/// Answers `None` when an argument was rejected — the caller must then leave
/// `rettv` alone, which is what upstream's `called_emsg` comparison decides.
///
/// # Safety
/// `argvars` must be a valid argument vector.
unsafe fn getchar_opts(argvars: *mut typval_T, allow_number: bool) -> Option<GetcharOpts> {
    unsafe {
        let mut opts = GetcharOpts {
            allow_number,
            simplify: true,
            cursor: CursorFlag::Default,
        };
        let called_emsg_start = called_emsg.get();

        if (*argvars).v_type != VAR_UNKNOWN && tv_check_for_opt_dict_arg(argvars, 1) == FAIL {
            return None;
        }
        if (*argvars).v_type != VAR_UNKNOWN && (*argvars.add(1)).v_type == VAR_DICT {
            let d = (*argvars.add(1)).vval.v_dict;

            if opts.allow_number {
                opts.allow_number = tv_dict_get_bool(d, c"number".as_ptr(), true_0) != 0;
            } else if tv_dict_has_key(d, c"number".as_ptr()) {
                // getcharstr() never answers a number, so asking is an error.
                semsg_c!(
                    gettext(&raw const e_invarg2 as *const c_char),
                    c"number".as_ptr(),
                );
            }

            opts.simplify = tv_dict_get_bool(d, c"simplify".as_ptr(), true_0) != 0;

            let cursor = tv_dict_get_string(d, c"cursor".as_ptr(), false);
            if !cursor.is_null() {
                opts.cursor = if strcmp(cursor, c"hide".as_ptr()) == 0 {
                    CursorFlag::Hide
                } else if strcmp(cursor, c"keep".as_ptr()) == 0 {
                    CursorFlag::Keep
                } else if strcmp(cursor, c"msg".as_ptr()) == 0 {
                    CursorFlag::Msg
                } else {
                    semsg_c!(
                        gettext(&raw const e_invargNval as *const c_char),
                        c"cursor".as_ptr(),
                        cursor,
                    );
                    CursorFlag::Default
                };
            }
        }

        if called_emsg.get() != called_emsg_start {
            return None;
        }
        Some(opts)
    }
}

/// Read one key for `getchar()` / `getcharstr()`.
///
/// `argvars[0]` decides how: absent or -1 blocks, 1 only peeks, 0 takes a key
/// if one is there. Keys nothing can act on are skipped.
///
/// # Safety
/// `argvars` must be a valid argument vector.
unsafe fn getchar_read(argvars: *mut typval_T, cursor: CursorFlag) -> varnumber_T {
    unsafe {
        let mut error = false;
        loop {
            if cursor == CursorFlag::Msg || (cursor == CursorFlag::Default && msg_col.get() > 0) {
                ui_cursor_goto(msg_row.get(), msg_col.get());
            }

            let blocking = (*argvars).v_type == VAR_UNKNOWN
                || ((*argvars).v_type == VAR_NUMBER && (*argvars).vval.v_number == -1);
            let n: varnumber_T = if blocking {
                // getchar(): blocking wait.
                // TODO(bfredl): deduplicate the shared logic with state_enter?
                if !char_avail() {
                    ui_flush(); // flush screen updates before blocking
                    input_get(
                        ptr::null_mut(),
                        0,
                        -1,
                        (*typebuf.ptr()).tb_change_cnt,
                        (*main_loop.ptr()).events,
                    );
                    if input_available() == 0 && !multiqueue_empty((*main_loop.ptr()).events) {
                        state_handle_k_event();
                        continue;
                    }
                }
                safe_vgetc() as varnumber_T
            } else if tv_get_number_chk(argvars, &raw mut error) == 1 {
                // getchar(1): only check whether a character is available.
                vpeekc_any() as varnumber_T
            } else if error || vpeekc_any() == NUL {
                // An illegal argument, or getchar(0) with nothing there.
                0
            } else {
                // getchar(0) with something there. Note that `vpeekc_any`
                // answers K_SPECIAL for K_IGNORE.
                safe_vgetc() as varnumber_T
            };

            let n = n as c_int;
            if n != K_IGNORE && n != K_MOUSEMOVE && n != K_VER_SCROLLBAR && n != K_HOR_SCROLLBAR {
                return n as varnumber_T;
            }
        }
    }
}

/// Set `v:mouse_win`, `v:mouse_winid`, `v:mouse_lnum` and `v:mouse_col` from
/// the position a mouse key was received at.
///
/// # Safety
/// Callable at any time.
unsafe fn set_mouse_vars() {
    unsafe {
        let mut pos = MousePos::current();
        if pos.row < 0 || pos.col < 0 {
            return;
        }

        // Find the window under the mouse and turn the coordinates into a
        // text position.
        let Some(win) = find_win_inner(&mut pos) else {
            return;
        };
        let (lnum, _) = comp_pos(win, &mut pos.row, &mut pos.col);

        let mut winnr = 1;
        let mut wp = firstwin.get();
        while wp != win.raw() {
            winnr += 1;
            wp = (*wp).w_next;
        }
        set_vim_var_nr(VV_MOUSE_WIN, winnr as varnumber_T);
        set_vim_var_nr(VV_MOUSE_WINID, (*wp).handle as varnumber_T);
        set_vim_var_nr(VV_MOUSE_LNUM, lnum as varnumber_T);
        set_vim_var_nr(VV_MOUSE_COL, (pos.col + 1) as varnumber_T);
    }
}

/// `getchar()` and `getcharstr()`.
///
/// # Safety
/// `argvars` and `rettv` must be a valid argument vector and return slot.
pub(crate) unsafe fn getchar_common(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    allow_number: bool,
) {
    unsafe {
        let Some(opts) = getchar_opts(argvars, allow_number) else {
            return;
        };

        if opts.cursor == CursorFlag::Hide {
            ui_busy_start();
        }
        *no_mapping.ptr() += 1;
        *allow_keys.ptr() += 1;
        if !opts.simplify {
            *no_reduce_keys.ptr() += 1;
        }

        let n = getchar_read(argvars, opts.cursor);

        *no_mapping.ptr() -= 1;
        *allow_keys.ptr() -= 1;
        if !opts.simplify {
            *no_reduce_keys.ptr() -= 1;
        }
        if opts.cursor == CursorFlag::Hide {
            ui_busy_stop();
        }

        set_vim_var_nr(VV_MOUSE_WIN, 0);
        set_vim_var_nr(VV_MOUSE_WINID, 0);
        set_vim_var_nr(VV_MOUSE_LNUM, 0);
        set_vim_var_nr(VV_MOUSE_COL, 0);

        if n != 0 && (!opts.allow_number || n < 0 || mod_mask.get() != 0) {
            // Render the key as a string: modifier prefix, then either the
            // key code's three bytes or the character's UTF-8 ones.
            let mut temp = [0 as c_char; 10]; // modifier 3 + mbyte char 6 + NUL
            let mut i = 0;
            if mod_mask.get() != 0 {
                temp[0] = K_SPECIAL as c_char;
                temp[1] = KS_MODIFIER as c_char;
                temp[2] = mod_mask.get() as c_char;
                i = 3;
            }
            if n < 0 {
                for (at, byte) in key_escape(n as c_int).into_iter().enumerate() {
                    temp[i + at] = byte as c_char;
                }
                i += 3;
            } else {
                i += utf_char2bytes(n as c_int, temp.as_mut_ptr().add(i)) as usize;
            }
            debug_assert!(i < temp.len());
            temp[i] = 0;

            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = xmemdupz(temp.as_ptr().cast(), i).cast();

            if is_mouse_key(n as c_int) {
                set_mouse_vars();
            }
        } else if !opts.allow_number {
            (*rettv).v_type = VAR_STRING;
        } else {
            (*rettv).vval.v_number = n;
        }
    }
}

/// The `getchar()` Vimscript function.
///
/// Keeps the `extern "C"` ABI: the eval function table holds it as an
/// `extern "C"` pointer.
///
/// # Safety
/// As [`getchar_common`].
pub unsafe extern "C" fn f_getchar(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { getchar_common(argvars, rettv, true) };
}

/// The `getcharstr()` Vimscript function.
///
/// # Safety
/// As [`getchar_common`].
pub unsafe extern "C" fn f_getcharstr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { getchar_common(argvars, rettv, false) };
}

/// The `getcharmod()` Vimscript function: the modifiers of the last key.
///
/// # Safety
/// `rettv` must be a valid return slot.
pub unsafe extern "C" fn f_getcharmod(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { (*rettv).vval.v_number = mod_mask.get() as varnumber_T };
}
