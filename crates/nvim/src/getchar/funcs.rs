//! The Vimscript `getchar()` family.
//!
//! [`getchar_common`] backs `getchar()`, `getcharstr()` and `getcharmod()`:
//! it reads one key with mappings disabled, optionally without blocking, and
//! renders it as a number, a string or a modifier mask.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::{NumBuf, Unconvertible};
use crate::guard::{Keys, Suppress};
use crate::keycodes::{SpecialKey, key_escape};
use crate::mbyte::Codepoint;
use crate::mbyte::encode_char;
use crate::memory::ThinCString;
use crate::message_fmt::msg_cstr;
use crate::narrow::len_as_integer;
use crate::os::input::{InputEvents, main_loop_has_events};
use crate::semsg;
use crate::types::{Key, VAR_DICT, VAR_STRING};
use crate::winlayer::windows;

/// What the `cursor` option asked for while the key is awaited.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Debug)]
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
/// `result` alone, which is what upstream's `CALLED_EMSG` comparison decides.
fn getchar_opts(args: &[TypVal], allow_number: bool) -> Option<GetcharOpts> {
    let mut numbuf = NumBuf::new();
    let mut opts = GetcharOpts {
        allow_number,
        simplify: true,
        cursor: CursorFlag::Default,
    };
    let called_emsg_start = CALLED_EMSG.get();

    // SAFETY (this body): the Vimscript call convention -- `args` is a live
    // argument vector running to a `VAR_UNKNOWN`, so every slot tested here is
    // there, and `numbuf` outlives the strings it lends back.
    if !args.is_empty() && tv_check_for_opt_dict_arg(args, 1).is_err() {
        return None;
    }
    if !args.is_empty() && args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT) {
        let d = args[1].dict_ref();

        if opts.allow_number {
            opts.allow_number = dict_get_bool(d, b"number", 1) != 0;
        } else if dict_has_key(d, b"number") {
            // getcharstr() never answers a number, so asking is an error.
            semsg!("E475: Invalid argument: {}", "number");
        }

        opts.simplify = dict_get_bool(d, b"simplify", 1) != 0;

        if let Some(cursor) = numbuf.dict_string(d, b"cursor") {
            opts.cursor = match cursor.to_bytes() {
                b"hide" => CursorFlag::Hide,
                b"keep" => CursorFlag::Keep,
                b"msg" => CursorFlag::Msg,
                _ => {
                    let cursor = msg_cstr(cursor);
                    semsg!("E475: Invalid value for argument {}: {cursor}", "cursor");
                    CursorFlag::Default
                }
            };
        }
    }

    if CALLED_EMSG.get() != called_emsg_start {
        return None;
    }
    Some(opts)
}

/// Read one key for `getchar()` / `getcharstr()`.
///
/// `argvars[0]` decides how: absent or -1 blocks, 1 only peeks, 0 takes a key
/// if one is there. Keys nothing can act on are skipped.
fn getchar_read(args: &[TypVal], cursor: CursorFlag) -> Key {
    loop {
        if cursor == CursorFlag::Msg || (cursor == CursorFlag::Default && MSG_COL.get() > 0) {
            ui_cursor_goto(MSG_ROW.get(), MSG_COL.get());
        }

        // SAFETY (this body): reads one key through the ordinary input stack;
        // the buffers it fills are this frame's own.
        let blocking = args.is_empty() || args[0].as_number() == Some(-1);
        // Read once: the coercion reports its own message, and asking twice
        // would report it twice.
        let how = (!blocking).then(|| tv_get_number_chk(&args[0]));
        let n = if blocking {
            // getchar(): blocking wait.
            // TODO(bfredl): deduplicate the shared logic with state_enter?
            if !char_avail() {
                ui_flush(); // flush screen updates before blocking
                input_get(&mut [], -1, typeahead().change_cnt(), InputEvents::MainLoop);
                if input_available() == 0 && main_loop_has_events() {
                    state_handle_k_event();
                    continue;
                }
            }
            safe_vgetc()
        } else if how == Some(Ok(1)) {
            // getchar(1): only check whether a character is available.
            vpeekc_any()
        } else if how == Some(Err(Unconvertible)) || vpeekc_any().is_nul() {
            // An illegal argument, or getchar(0) with nothing there.
            Key::NUL
        } else {
            // getchar(0) with something there. Note that `vpeekc_any`
            // answers K_SPECIAL for K_IGNORE.
            safe_vgetc()
        };

        if !matches!(
            n.special(),
            Some(
                SpecialKey::Ignore
                    | SpecialKey::Mousemove
                    | SpecialKey::VerScrollbar
                    | SpecialKey::HorScrollbar
            )
        ) {
            return n;
        }
    }
}

/// Set `v:mouse_win`, `v:mouse_winid`, `v:mouse_lnum` and `v:mouse_col` from
/// the position a mouse key was received at.
fn set_mouse_vars() {
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
    set_vim_var_nr(Vv::MouseWin, len_as_integer(winnr));
    set_vim_var_nr(Vv::MouseWinid, VarNumber::from(win.handle.get()));
    set_vim_var_nr(Vv::MouseLnum, i64::from(lnum));
    set_vim_var_nr(Vv::MouseCol, VarNumber::from(pos.col + 1));
}

/// `getchar()` and `getcharstr()`.
pub(crate) fn getchar_common(args: &[TypVal], result: &mut TypVal, allow_number: bool) {
    let Some(opts) = getchar_opts(args, allow_number) else {
        return;
    };

    if opts.cursor == CursorFlag::Hide {
        ui_busy_start();
    }
    let raw_key = Keys::unmapped_with_codes();
    let unsimplified = (!opts.simplify).then(|| Suppress::counter(NO_REDUCE_KEYS));

    let n = getchar_read(args, opts.cursor);

    drop(raw_key);
    drop(unsimplified);
    if opts.cursor == CursorFlag::Hide {
        ui_busy_stop();
    }

    set_vim_var_nr(Vv::MouseWin, 0);
    set_vim_var_nr(Vv::MouseWinid, 0);
    set_vim_var_nr(Vv::MouseLnum, 0);
    set_vim_var_nr(Vv::MouseCol, 0);

    if !n.is_nul() && (!opts.allow_number || n.is_special() || !MOD_MASK.get().is_empty()) {
        // Render the key as a string: modifier prefix, then either the
        // key code's three bytes or the character's UTF-8 ones.
        let mut temp = [0u8; 10]; // modifier 3 + mbyte char 6 + NUL
        let mut i = 0;
        if !MOD_MASK.get().is_empty() {
            temp[0] = K_SPECIAL;
            temp[1] = KS_MODIFIER;
            temp[2] = MOD_MASK.get().byte();
            i = 3;
        }
        if n.is_special() {
            temp[i..i + 3].copy_from_slice(&key_escape(n));
            i += 3;
        } else {
            i += encode_char(Codepoint::new(n.code()), &mut temp[i..]);
        }
        debug_assert!(i < temp.len());

        (*result).write_string(Some(ThinCString::from_bytes(&temp[..i])));

        if is_mouse_key(n) {
            set_mouse_vars();
        }
    } else if !opts.allow_number {
        (*result).write_empty(VAR_STRING);
    } else {
        (*result).write_number(VarNumber::from(n.code()));
    }
}

/// The `getchar()` Vimscript function.
///
/// The eval function table holds it as a `VimLFunc` pointer.
pub fn f_getchar(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getchar_common(args, result, true);
}

/// The `getcharstr()` Vimscript function.
pub fn f_getcharstr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY (this body): as [`f_getchar`].
    getchar_common(args, result, false);
}

/// The `getcharmod()` Vimscript function: the modifiers of the last key.
pub(crate) fn f_getcharmod(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY (this body): as [`f_getchar`].
    result.write_number(VarNumber::from(MOD_MASK.get().bits()));
}
