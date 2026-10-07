//! Resolving a buffer argument — number, name, `#`, `%` — and the questions
//! about one: `bufnr()`, `bufname()`, `bufwinid()`, ...

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::{buflist_add_name, find_buffer_by_name};
use crate::eval::typval::NumBuf;
use crate::guard::Suppress;
use crate::memory::ThinCString;
use crate::types::{VAR_NUMBER, VAR_STRING};

/// The buffer `avar` names, by number or by exact name.
pub fn find_buffer(avar: &TypVal) -> Option<Buf> {
    match avar.v_type() {
        VAR_NUMBER => find_buf(number_as_int(avar.number_or_zero())),
        VAR_STRING => {
            let name = avar.string_cstr()?;
            if let Some(found) = find_buffer_by_name(name) {
                return Some(found);
            }
            // A buffer with no file of its own — a URL, or a scratch
            // buffer — is not in the name index, so it is matched
            // literally instead.
            buffers().find(|b| {
                b.name.shown().is_some_and(|shown| {
                    (path_with_url(shown) != 0 || buf_is_nofilename(Some(*b))) && shown == name
                })
            })
        }
        _ => None,
    }
}

/// `bufadd({name})` — the number of the buffer, creating it if need be.
pub fn f_bufadd(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let name = numbuf.string(&args[0]);
    // An empty name asks for an unnamed buffer.
    let name = (!name.is_empty()).then_some(name);
    let buffer = buflist_add_name(name, 0, 0);
    result.write_number(buffer.map_or(0, |b| VarNumber::from(b.handle)));
}

/// `bufexists({buf})`.
pub fn f_bufexists(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let buf = find_buffer(&args[0]);
    result.write_number(VarNumber::from(buf.is_some()));
}

/// `buflisted({buf})`.
pub fn f_buflisted(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let listed = find_buffer(&args[0]).is_some_and(|b| b.b_p_bl != 0);
    result.write_number(VarNumber::from(listed));
}

/// `bufload({buf})` — read the file in if the buffer is not loaded yet.
pub fn f_bufload(args: &[TypVal], _unused: &mut TypVal, _fptr: EvalFuncData) {
    let buf = get_buf_arg(&args[0]);
    if buf.is_none() {
        return;
    }
    // A swap file found while loading must not leave the standing
    // "read-only" answer behind unless that is what it already was.
    if swap_exists_action.get() != SEA_READONLY {
        swap_exists_action.set(SEA_NONE);
    }
    buf_ensure_loaded(buf.expect("a live handle"));
}

/// `bufloaded({buf})`.
pub fn f_bufloaded(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let loaded = find_buffer(&args[0]).is_some_and(|b| !b.b_ml.ml_mfp.is_null());
    result.write_number(VarNumber::from(loaded));
}

/// `bufname([{buf}])` — the buffer's short name, empty when it has none.
pub fn f_bufname(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    let buf = if !args.is_empty() {
        tv_get_buf_from_arg(&args[0])
    } else {
        Some(Buf::current())
    };
    if let Some(buf) = buf
        && !buf.name.is_unnamed()
    {
        result.write_string(buf.name.shown().map(ThinCString::from_cstr));
    }
}

/// `bufnr([{buf} [, {create}]])` — -1 when there is no such buffer and it was
/// not asked to be created.
pub fn f_bufnr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);
    let mut buf = if args.is_empty() {
        Buf::current_or_none()
    } else {
        if !tv_check_str_or_nr(&args[0]) {
            return;
        }
        // The lookup itself must not report "no such buffer": a second
        // argument asks for the buffer to be created instead.
        let _no_emsg = Suppress::emsg();
        arg_buf(args, 0, 0)
    };
    if buf.is_none()
        && args.len() > 1
        && tv_get_number_chk(&args[1]).is_ok_and(|create| create != 0)
        && let Some(name) = numbuf.string_chk(&args[0])
    {
        buf = buflist_add_name(Some(name), 1, 0);
    }
    if let Some(buf) = buf {
        result.write_number(VarNumber::from(buf.handle));
    }
}

/// `bufwinid()` and `bufwinnr()`: the first window of the current tab page
/// showing the buffer, as an id or as a `winnr()` ordinal.
fn buf_win_common(args: &[TypVal], result: &mut TypVal, get_nr: bool) {
    let buf = arg_buf_chk(args, 0);
    if buf.is_none() {
        result.write_number(-1);
        return;
    }
    let tp = TabPage::current();
    // `bufwinnr()` skips a window the numbering has no number for;
    // `bufwinid()` still answers its id.
    let mut winnr = 0;
    let found = windows_in_tab(tp).find(|wp| {
        winnr += c_int::from(wp.has_winnr(tp));
        Some(wp.buffer()) == buf && (!get_nr || wp.has_winnr(tp))
    });
    result.write_number(match found {
        Some(wp) => VarNumber::from(if get_nr { winnr } else { wp.handle }),
        None => -1,
    });
}

/// `bufwinid({buf})`.
pub fn f_bufwinid(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    buf_win_common(args, result, false);
}

/// `bufwinnr({buf})`.
pub fn f_bufwinnr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    buf_win_common(args, result, true);
}
