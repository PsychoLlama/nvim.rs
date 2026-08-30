//! The Vimscript `getchar()` family.
//!
//! [`getchar_common`] backs `getchar()`, `getcharstr()` and `getcharmod()`:
//! it reads one key with mappings disabled, optionally without blocking, and
//! renders it as a number, a string or a modifier mask.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::guard::{Keys, Suppress};
use crate::keycodes::{K_IGNORE, K_MOUSEMOVE, key_escape};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{NUL, VAR_DICT, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};
use crate::winlayer::windows;
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
    let mut numbuf = NumBuf::new();
    let mut opts = GetcharOpts {
        allow_number,
        simplify: true,
        cursor: CursorFlag::Default,
    };
    let called_emsg_start = called_emsg.get();

    // SAFETY (this body): the Vimscript call convention -- `argvars` is a live
    // argument vector running to a `VAR_UNKNOWN`, so every slot tested here is
    // there, and `numbuf` outlives the strings it lends back.
    if unsafe { (*argvars).v_type } != VAR_UNKNOWN
        && unsafe { tv_check_for_opt_dict_arg(argvars, 1) }.is_err()
    {
        return None;
    }
    if unsafe { (*argvars).v_type } != VAR_UNKNOWN
        && unsafe { (*argvars.add(1)).v_type } == VAR_DICT
    {
        let d = unsafe { (*argvars.add(1)).vval.v_dict };

        if opts.allow_number {
            opts.allow_number = unsafe { tv_dict_get_bool(d, c"number".as_ptr(), 1) } != 0;
        } else if unsafe { tv_dict_has_key(d, c"number".as_ptr()) } {
            // getcharstr() never answers a number, so asking is an error.
            semsg!("E475: Invalid argument: {}", "number");
        }

        opts.simplify = unsafe { tv_dict_get_bool(d, c"simplify".as_ptr(), 1) } != 0;

        let cursor = unsafe { numbuf.dict_string(d, c"cursor".as_ptr()) };
        if !cursor.is_null() {
            opts.cursor = if unsafe { cstr::bytes_at(cursor) == b"hide" } {
                CursorFlag::Hide
            } else if unsafe { cstr::bytes_at(cursor) == b"keep" } {
                CursorFlag::Keep
            } else if unsafe { cstr::bytes_at(cursor) == b"msg" } {
                CursorFlag::Msg
            } else {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let cursor = unsafe { c_str(cursor) };
                semsg!("E475: Invalid value for argument {}: {cursor}", "cursor");
                CursorFlag::Default
            };
        }
    }

    if called_emsg.get() != called_emsg_start {
        return None;
    }
    Some(opts)
}

/// Read one key for `getchar()` / `getcharstr()`.
///
/// `argvars[0]` decides how: absent or -1 blocks, 1 only peeks, 0 takes a key
/// if one is there. Keys nothing can act on are skipped.
///
/// # Safety
/// `argvars` must be a valid argument vector.
unsafe fn getchar_read(argvars: *mut typval_T, cursor: CursorFlag) -> varnumber_T {
    let mut error = false;
    loop {
        if cursor == CursorFlag::Msg || (cursor == CursorFlag::Default && msg_col.get() > 0) {
            ui_cursor_goto(msg_row.get(), msg_col.get());
        }

        // SAFETY (this body): reads one key through the ordinary input stack;
        // the buffers it fills are this frame's own.
        let blocking = unsafe { (*argvars).v_type } == VAR_UNKNOWN
            || (unsafe { (*argvars).v_type } == VAR_NUMBER
                && unsafe { (*argvars).vval.v_number } == -1);
        let n: varnumber_T = if blocking {
            // getchar(): blocking wait.
            // TODO(bfredl): deduplicate the shared logic with state_enter?
            if !char_avail() {
                unsafe { ui_flush() }; // flush screen updates before blocking
                unsafe {
                    input_get(
                        ptr::null_mut(),
                        0,
                        -1,
                        typeahead().change_cnt(),
                        (*main_loop.ptr()).events,
                    )
                };
                if input_available() == 0 && !unsafe { multiqueue_empty((*main_loop.ptr()).events) }
                {
                    unsafe { state_handle_k_event() };
                    continue;
                }
            }
            safe_vgetc() as varnumber_T
        } else if unsafe { tv_get_number_chk(argvars, &raw mut error) } == 1 {
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

/// Set `v:mouse_win`, `v:mouse_winid`, `v:mouse_lnum` and `v:mouse_col` from
/// the position a mouse key was received at.
///
/// # Safety
/// Callable at any time.
unsafe fn set_mouse_vars() {
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

    // Upstream stops this walk on `win` itself rather than on the end of
    // the list, and `v:mouse_win` is one more than the number of windows
    // before it. `win` came out of a walk of the same list, so the "not
    // in it" arm upstream walks into a null pointer on is unreachable.
    let winnr = windows().take_while(|wp| wp.raw() != win.raw()).count() + 1;
    // SAFETY (this body): `curwin` is set from startup to exit, and the vim
    // variables set here are the editor's own.
    unsafe { set_vim_var_nr(Vv::MouseWin, winnr as varnumber_T) };
    unsafe { set_vim_var_nr(Vv::MouseWinid, win.handle as varnumber_T) };
    unsafe { set_vim_var_nr(Vv::MouseLnum, lnum as varnumber_T) };
    unsafe { set_vim_var_nr(Vv::MouseCol, (pos.col + 1) as varnumber_T) };
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
    // SAFETY (this body): as [`getchar_opts`] -- a live argument vector and a
    // writable `rettv`; the scratch buffers are this frame's own.
    let Some(opts) = (unsafe { getchar_opts(argvars, allow_number) }) else {
        return;
    };

    if opts.cursor == CursorFlag::Hide {
        ui_busy_start();
    }
    let raw_key = Keys::unmapped_with_codes();
    let unsimplified = (!opts.simplify).then(|| Suppress::counter(&no_reduce_keys));

    let n = unsafe { getchar_read(argvars, opts.cursor) };

    drop(raw_key);
    drop(unsimplified);
    if opts.cursor == CursorFlag::Hide {
        ui_busy_stop();
    }

    unsafe { set_vim_var_nr(Vv::MouseWin, 0) };
    unsafe { set_vim_var_nr(Vv::MouseWinid, 0) };
    unsafe { set_vim_var_nr(Vv::MouseLnum, 0) };
    unsafe { set_vim_var_nr(Vv::MouseCol, 0) };

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
            i += unsafe { utf_char2bytes(n as c_int, temp.as_mut_ptr().add(i)) } as usize;
        }
        debug_assert!(i < temp.len());
        temp[i] = 0;

        unsafe { (*rettv).v_type = VAR_STRING };
        unsafe { (*rettv).vval.v_string = xmemdupz(temp.as_ptr().cast(), i).cast() };

        if is_mouse_key(n as c_int) {
            unsafe { set_mouse_vars() };
        }
    } else if !opts.allow_number {
        unsafe { (*rettv).v_type = VAR_STRING };
    } else {
        unsafe { (*rettv).vval.v_number = n };
    }
}

/// The `getchar()` Vimscript function.
///
/// The eval function table holds it as a `VimLFunc` pointer.
///
/// # Safety
/// As [`getchar_common`].
pub unsafe fn f_getchar(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY (this body): the Vimscript call convention, passed straight
    // through.
    unsafe { getchar_common(argvars, rettv, true) };
}

/// The `getcharstr()` Vimscript function.
///
/// # Safety
/// As [`getchar_common`].
pub unsafe fn f_getcharstr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY (this body): as [`f_getchar`].
    unsafe { getchar_common(argvars, rettv, false) };
}

/// The `getcharmod()` Vimscript function: the modifiers of the last key.
///
/// # Safety
/// `rettv` must be a valid return slot.
pub unsafe fn f_getcharmod(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY (this body): as [`f_getchar`].
    unsafe { (*rettv).vval.v_number = mod_mask.get() as varnumber_T };
}
