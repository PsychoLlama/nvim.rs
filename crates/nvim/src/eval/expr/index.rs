//! `[]`, `[:]` and `.` applied to a value the evaluator already has.
//!
//! Two indexing vocabularies live here and they are not the same. The
//! subscript the *grammar* produces (`s[1]`, `s[1:2]`) counts **bytes** in a
//! String and includes its end; `slice()` counts **characters** and excludes
//! its end. `exclusive` is the flag that tells them apart, and it also
//! switches the String arm onto the character walkers.

#![forbid(unsafe_code)]

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::{CStr, c_int};

use crate::eval::typval::{
    DictRef, NumBuf, blob_slice_or_index, list_slice_or_index, tv_check_str, tv_clear, tv_copy,
    tv_get_number,
};
use crate::eval::userfunc::set_selfdict;
use crate::eval::{
    Cursor, VARNUMBER_MAX, call_func_rettv, char_len_at, check_luafunc_name,
    e_cannot_index_a_funcref, e_cannot_index_special_variable, e_cannot_slice_dictionary,
    e_missbrac, eval_isdictc, eval_lambda, eval_method, eval1, tv_is_luafunc,
};
use crate::ex_eval::aborting;
use crate::mbyte::head_off;
use crate::message::e_using_float_as_string;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{
    EvalFuncData, Failed, TypVal, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST,
    VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarNumber,
};

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// `expr[idx]`, `expr[first : last]` or `dict.key`, with the cursor on the
/// `[` or the `.`. Leaves the cursor after the `]` or the key.
pub(crate) fn eval_index(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    verbose: bool,
) -> Result<(), Failed> {
    let mut empty1 = false;
    let mut empty2 = false;
    let mut range = false;
    let mut key: Option<&[u8]> = None;

    check_can_index(result, evaluate, verbose)?;

    let mut var1 = UNSET_TV;
    let mut var2 = UNSET_TV;
    if cursor.byte() == b'.' {
        // dict.name
        let rest = &cursor.rest()[1..];
        let keylen = rest
            .iter()
            .take_while(|&&b| eval_isdictc(c_int::from(b)))
            .count();
        if keylen == 0 {
            return Err(Failed);
        }
        key = Some(&rest[..keylen]);
        cursor.bump(1 + keylen);
        cursor.skip_white();
    } else {
        // The first index, from inside the brackets.
        cursor.bump(1);
        cursor.skip_white();
        if cursor.byte() == b':' {
            empty1 = true;
        } else if eval1(cursor, &mut var1, evaluate).is_err() {
            return Err(Failed);
        } else if evaluate && !tv_check_str(&var1) {
            tv_clear(&mut var1);
            return Err(Failed);
        }

        // The second index, from inside the `[ : ]`.
        if cursor.byte() == b':' {
            range = true;
            cursor.bump(1);
            cursor.skip_white();
            if cursor.byte() == b']' {
                empty2 = true;
            } else if eval1(cursor, &mut var2, evaluate).is_err() {
                if !empty1 {
                    tv_clear(&mut var1);
                }
                return Err(Failed);
            } else if evaluate && !tv_check_str(&var2) {
                if !empty1 {
                    tv_clear(&mut var1);
                }
                tv_clear(&mut var2);
                return Err(Failed);
            }
        }

        if cursor.byte() != b']' {
            if verbose {
                emsg(gettext(e_missbrac));
            }
            // Not guarded by `empty1`: an unread `var1` is still unset.
            tv_clear(&mut var1);
            if range {
                tv_clear(&mut var2);
            }
            return Err(Failed);
        }
        cursor.bump(1);
        cursor.skip_white();
    }

    if !evaluate {
        return Ok(());
    }
    // An empty half of a `[a:b]` is *absent*, not an unset value.
    let one = (!empty1).then_some(&var1);
    let two = (!empty2).then_some(&var2);
    let res = eval_index_inner(result, range, one, two, false, key, verbose);
    if !empty1 {
        tv_clear(&mut var1);
    }
    if range {
        tv_clear(&mut var2);
    }
    res
}

/// Can `result` carry an `[index]` or a `[sli:ce]` at all?
pub(crate) fn check_can_index(
    result: &TypVal,
    evaluate: bool,
    verbose: bool,
) -> Result<(), Failed> {
    let message: &'static CStr = match result.v_type() {
        VAR_FUNC | VAR_PARTIAL => e_cannot_index_a_funcref,
        VAR_FLOAT => e_using_float_as_string,
        VAR_BOOL | VAR_SPECIAL => e_cannot_index_special_variable,
        // Not evaluating: the subscript is only being skipped over, and an
        // unset value is what an unevaluated operand looks like.
        VAR_UNKNOWN if !evaluate => return Ok(()),
        // Reported whether or not the caller asked to be verbose.
        VAR_UNKNOWN => {
            emsg(gettext(e_cannot_index_special_variable));
            return Err(Failed);
        }
        _ => return Ok(()),
    };
    if verbose {
        emsg(gettext(message));
    }
    Err(Failed)
}

/// `slice()`
pub(crate) fn f_slice(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if check_can_index(&args[0], true, false).is_err() {
        return;
    }
    tv_copy(&args[0], result);
    let (first, end) = (Some(&args[1]), args.get(2));
    let _ = eval_index_inner(result, true, first, end, true, None, false);
}

/// Apply an index or a range to `result`, in place.
///
/// `var1` is the first index and is absent for `[:expr]`; `var2` is the
/// second and is absent for `[expr]` and `[expr:]`. `exclusive` is
/// `slice()`'s: the second index is excluded and a String is indexed by
/// character. `key`, when there is one, is the Dict index instead of `var1`.
pub(crate) fn eval_index_inner(
    result: &mut TypVal,
    is_range: bool,
    var1: Option<&TypVal>,
    var2: Option<&TypVal>,
    exclusive: bool,
    key: Option<&[u8]>,
    verbose: bool,
) -> Result<(), Failed> {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut n1: VarNumber = 0;
    let mut n2: VarNumber = 0;
    if let Some(var1) = var1
        && result.v_type() != VAR_DICT
    {
        n1 = tv_get_number(var1);
    }
    if is_range {
        if result.v_type() == VAR_DICT {
            if verbose {
                emsg(gettext(e_cannot_slice_dictionary));
            }
            return Err(Failed);
        }
        n2 = match var2 {
            None => VARNUMBER_MAX,
            Some(var2) => tv_get_number(var2),
        };
    }

    match result.v_type() {
        VAR_NUMBER | VAR_STRING => {
            let s = numbuf.string(result).to_bytes();
            let len = s.len() as c_int as VarNumber;
            let v = if exclusive {
                // slice(): character indexes, second one excluded.
                if is_range {
                    string_slice(s, n1, n2, exclusive)
                } else {
                    char_from_string(s, n1)
                }
            } else if is_range {
                // A substring. Out-of-range indexes give an empty result.
                if n1 < 0 {
                    n1 = (len + n1).max(0);
                }
                if n2 < 0 {
                    n2 += len;
                } else if n2 >= len {
                    n2 = len;
                }
                if n1 >= len || n2 < 0 || n1 > n2 {
                    None
                } else {
                    // `n2` may be the length itself, whose byte is the end.
                    Some(&s[n1 as usize..((n2 + 1) as usize).min(s.len())])
                }
            } else if n1 >= len || n1 < 0 {
                // A one-byte String; too big or negative gives an empty one.
                None
            } else {
                Some(&s[n1 as usize..=n1 as usize])
            };
            let v = v.map(XString::from_bytes);
            tv_clear(result);
            result.write_string(v.map_or(::core::ptr::null_mut(), XString::into_raw));
        }
        VAR_BLOB => {
            let _ = blob_slice_or_index(is_range, n1, n2, exclusive, result);
        }
        VAR_LIST => {
            if var1.is_none() {
                n1 = 0;
            }
            if var2.is_none() {
                n2 = VARNUMBER_MAX;
            }
            list_slice_or_index(is_range, n1, n2, exclusive, result, verbose)?;
        }
        VAR_DICT => {
            let key = match key {
                Some(key) => key,
                None => match numbuf2.string_chk(var1.expect("checked")) {
                    Some(key) => key.to_bytes(),
                    None => return Err(Failed),
                },
            };
            // `v:_null_dict` holds no key at all. The value is copied out
            // of the item before `result` -- which owns the Dict the item
            // lives in -- is cleared.
            let mut tmp = UNSET_TV;
            match result.dict_ref().and_then(|d| d.find(key)) {
                Some(item) if !tv_is_luafunc(&item.di_tv) => tv_copy(&item.di_tv, &mut tmp),
                Some(_) => return Err(Failed),
                None => {
                    if verbose {
                        let key = msg_bytes(key);
                        semsg!("E716: Key not present in Dictionary: \"{key}\"");
                    }
                    return Err(Failed);
                }
            }
            tv_clear(result);
            *result = tmp;
        }
        // Not evaluating: skipping over the subscript.
        _ => {}
    }
    Ok(())
}

/// `text[index]` by character index, composing characters included; `None`
/// when `index` is out of range.
pub(crate) fn char_from_string(text: &[u8], index: VarNumber) -> Option<&[u8]> {
    let mut nchar = index;

    // As for a List, a negative index counts from the end — but unlike a
    // List, running off the start is an empty string rather than an error.
    if index < 0 {
        let mut clen: c_int = 0;
        let mut nbyte = 0;
        while nbyte < text.len() {
            nbyte += char_len_at(text, nbyte);
            clen += 1;
        }
        nchar = VarNumber::from(clen) + index;
        if nchar < 0 {
            return None;
        }
    }

    let mut nbyte = 0;
    while nchar > 0 && nbyte < text.len() {
        nbyte += char_len_at(text, nbyte);
        nchar -= 1;
    }
    if nbyte >= text.len() {
        return None;
    }
    Some(&text[nbyte..nbyte + char_len_at(text, nbyte)])
}

/// The byte index of character index `idx` in `text`, composing characters
/// included. Answers `text.len()` for an index past the end and -1 for one
/// before the start.
pub(crate) fn char_idx2byte(text: &[u8], idx: VarNumber) -> isize {
    let mut nchar = idx;
    let mut nbyte = 0;
    if nchar >= 0 {
        while nchar > 0 && nbyte < text.len() {
            nbyte += char_len_at(text, nbyte);
            nchar -= 1;
        }
    } else {
        nbyte = text.len();
        while nchar < 0 && nbyte > 0 {
            nbyte -= 1;
            nbyte -= head_off(text, nbyte);
            nchar += 1;
        }
        if nchar < 0 {
            return -1;
        }
    }
    nbyte as isize
}

/// `text[first : last]` by character index, composing characters included.
/// `exclusive` is `slice()`'s. `None` when the result is empty.
pub(crate) fn string_slice(
    text: &[u8],
    first: VarNumber,
    last: VarNumber,
    exclusive: bool,
) -> Option<&[u8]> {
    let slen = text.len() as isize;
    // A very negative first index starts at zero rather than failing.
    let start_byte = char_idx2byte(text, first).max(0);
    let end_byte = if (last == -1 && !exclusive) || last == VARNUMBER_MAX {
        slen
    } else {
        let mut end = char_idx2byte(text, last);
        if !exclusive && end >= 0 && end < slen {
            // The end index is inclusive here.
            end += char_len_at(text, end as usize) as isize;
        }
        end
    };

    if start_byte >= slen || end_byte <= start_byte {
        return None;
    }
    Some(&text[start_byte as usize..end_byte as usize])
}

/// Everything that can follow a completed operand, in any order:
/// `expr[idx]`, `expr[a:b]`, `.name`, a call through a Funcref, and
/// `expr->method()`. `dict.func(expr)[idx]['func'](expr)->len()` is one run
/// of this loop.
pub(crate) fn handle_subscript(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    verbose: bool,
) -> Result<(), Failed> {
    let mut ret = Ok(());
    let mut selfdict: Option<DictRef> = None;
    let mut lua_name: Option<usize> = None;

    if tv_is_luafunc(result) {
        if !evaluate {
            tv_clear(result);
        }
        if cursor.byte() != b'.' {
            tv_clear(result);
            ret = Err(Failed);
        } else {
            cursor.bump(1);
            lua_name = Some(cursor.offset());
            let len = check_luafunc_name(cursor.rest(), true);
            if len == 0 {
                tv_clear(result);
                ret = Err(Failed);
            }
            cursor.bump(len);
        }
    }

    // Whether another subscript follows. An opening `[`, `.` or `(` right
    // after white space is not one: `a [b]` is two operands.
    let more = |cursor: &Cursor<'_>, result: &TypVal| {
        let c = cursor.byte();
        let opens = c == b'['
            || (c == b'.' && result.v_type() == VAR_DICT)
            || (c == b'(' && (!evaluate || result.is_func()));
        let before = cursor
            .offset()
            .checked_sub(1)
            .map_or(0, |at| cursor.text()[at]);
        (opens && !matches!(before, b' ' | b'\t')) || (c == b'-' && cursor.at(1) == b'>')
    };

    while ret.is_ok() && more(cursor, result) {
        if cursor.byte() == b'(' {
            ret = call_func_rettv(cursor, result, evaluate, selfdict.as_ref(), None, lua_name);
            // Stop evaluating on an immediate abort, an interrupt, or an
            // exception that was thrown and not caught.
            if aborting() {
                if ret.is_ok() {
                    tv_clear(result);
                }
                ret = Err(Failed);
            }
            selfdict = None;
        } else if cursor.byte() == b'-' {
            ret = if cursor.at(2) == b'{' {
                // expr->{lambda}()
                eval_lambda(cursor, result, evaluate, verbose)
            } else {
                // expr->name()
                eval_method(cursor, result, evaluate, verbose)
            };
        } else {
            // `[` or `.`: a Dict being subscripted is the `self` a
            // Funcref found in it would be bound to.
            selfdict = match &*result {
                TypVal::Dict(dict) => (**dict).clone(),
                _ => None,
            };
            if eval_index(cursor, result, evaluate, verbose).is_err() {
                tv_clear(result);
                ret = Err(Failed);
            }
        }
    }

    // Turn "dict.Func" into a partial for "Func" bound to "dict".
    if let Some(dict) = selfdict.as_mut()
        && result.is_func()
    {
        set_selfdict(result, dict);
    }
    ret
}
