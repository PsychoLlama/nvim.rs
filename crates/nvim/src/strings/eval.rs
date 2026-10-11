//! The remaining Vimscript string builtins.
//!
//! Measurement (`strlen`, `strchars`, `strcharlen`, `strwidth`,
//! `strdisplaywidth`), search (`stridx`, `strridx`), conversion (`str2nr`,
//! `str2list`, `string`, `strtrans`) and transformation (`tolower`, `toupper`,
//! `tr`, `trim`).  `strchar_common` is the character count `strchars()` and
//! `strcharlen()` share, differing only in whether composing characters count
//! separately.

#![forbid(unsafe_code)]

use crate::semsg;
use core::ffi::c_int;

use super::{find_bytes, strcase_copy, strict_bool_arg};
use crate::charset::display::transstr_owned;
use crate::charset::{Str2NrBases, skip, str2nr_scan};
use crate::cstr::byte_at;
use crate::eval::encode::encode_tv2string;
use crate::eval::typval::{
    NumBuf, tv_check_for_opt_string_arg, tv_get_bool, tv_get_number, tv_get_number_chk,
    tv_list_alloc_ret,
};
use crate::mbyte::{
    c_string_cells, char_at, char_count, char_len, cluster_len, clusters, head_off,
};
use crate::memory::ThinCString;
use crate::message::e_invarg;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::os::cshim::gettext;
use crate::plines::linetabsize_text;
use crate::types::{EvalFuncData, TypVal, VarNumber, kListLenUnknown, ptrdiff_t};

/// "str2list()" function: the string as a list of code points.
pub fn f_str2list(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let list = tv_list_alloc_ret(result, kListLenUnknown as ptrdiff_t);
    let bytes = numbuf.bytes(&args[0]);
    let mut at = 0;
    while at < bytes.len() {
        let rest = &bytes[at..];
        list.push_number(VarNumber::from(char_at(rest)));
        at += char_len(rest);
    }
}

/// "str2nr()" function.
///
/// The sign is handled here rather than by `vim_str2nr`, so that a base
/// prefix may follow it and so that whitespace between the two is allowed.
/// Text after the number is silently ignored.
pub fn f_str2nr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut base = 10;
    let mut what = Str2NrBases::NONE;
    if args.len() > 1 {
        base = tv_get_number(&args[1]) as c_int;
        if !matches!(base, 2 | 8 | 10 | 16) {
            emsg(gettext(e_invarg));
            return;
        }
        if args.len() > 2 && tv_get_bool(&args[2]) != 0 {
            what |= Str2NrBases::QUOTE;
        }
    }

    let text = numbuf.string(&args[0]).to_bytes();
    let mut at = skip::white(text);
    let isneg = byte_at(text, at) == b'-';
    if matches!(byte_at(text, at), b'+' | b'-') {
        at += 1;
        at += skip::white(&text[at..]);
    }

    // An explicit base forces that radix; base 10 accepts none.
    what |= match base {
        2 => Str2NrBases::BIN | Str2NrBases::FORCE,
        8 => Str2NrBases::OCT_ANY | Str2NrBases::FORCE,
        16 => Str2NrBases::HEX | Str2NrBases::FORCE,
        _ => Str2NrBases::NONE,
    };

    // Only the number matters here: the prefix length, the digit count,
    // the unsigned value and the overflow flag go unread.
    let n = str2nr_scan(&text[at..], 0, what, false, true).value;
    result.write_number(if isneg { -n } else { n });
}

/// "stridx()" function: the byte index of the first occurrence.
pub fn f_stridx(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    let mut buf = NumBuf::new();
    let needle = numbuf.string_chk(&args[1]);
    let haystack = buf.string_chk(&args[0]);
    let (Some(needle), Some(haystack)) = (needle, haystack) else {
        return;
    };
    let (needle, haystack) = (needle.to_bytes(), haystack.to_bytes());
    let mut start = 0;

    if args.len() > 2 {
        let Ok(start_idx) = tv_get_number_chk(&args[2]) else {
            return;
        };
        if start_idx >= haystack.len() as VarNumber {
            return;
        }
        // A negative start is ignored, not counted from the end.
        start = usize::try_from(start_idx).unwrap_or(0);
    }

    if let Some(pos) = find_bytes(&haystack[start..], needle) {
        // Reported against the whole string, not against the start.
        result.write_number((start + pos) as VarNumber);
    }
}

/// "strridx()" function: the byte index of the last occurrence at or
/// before `end_idx`.
pub fn f_strridx(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    let mut buf = NumBuf::new();
    let needle = numbuf.string_chk(&args[1]);
    let haystack = buf.string_chk(&args[0]);
    let (Some(needle), Some(haystack)) = (needle, haystack) else {
        return;
    };
    let (needle, haystack) = (needle.to_bytes(), haystack.to_bytes());

    let last_allowed = if args.len() > 2 {
        let Ok(idx) = usize::try_from(tv_get_number_chk(&args[2]).unwrap_or(-1)) else {
            return;
        };
        idx
    } else {
        haystack.len()
    };

    let lastmatch = if needle.is_empty() {
        // The empty needle matches at the end of the range -- which may lie
        // past the string's end.
        Some(last_allowed)
    } else {
        let mut found = None;
        let mut rest = 0;
        while rest < haystack.len() {
            match find_bytes(&haystack[rest..], needle) {
                Some(at) if rest + at <= last_allowed => {
                    found = Some(rest + at);
                    rest += at + 1;
                }
                _ => break,
            }
        }
        found
    };

    if let Some(at) = lastmatch {
        result.write_number(at as VarNumber);
    }
}

/// "string()" function.
pub fn f_string(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(Some(encode_tv2string(&args[0])));
}

/// "strlen()" function: the length in bytes.
pub fn f_strlen(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(numbuf.bytes(&args[0]).len() as VarNumber);
}

/// The character count `strchars()` and `strcharlen()` share.
///
/// `skipcc` folds a composing character into the base character it
/// follows; without it each one counts on its own.
fn strchar_common(args: &[TypVal], result: &mut TypVal, skipcc: bool) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.string(&args[0]).to_bytes();
    let mut len: VarNumber = 0;
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        at += if skipcc {
            cluster_len(rest)
        } else {
            char_len(rest)
        };
        len += 1;
    }
    (*result).write_number(len);
}

/// "strcharlen()" function: characters, composing characters folded in.
pub fn f_strcharlen(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    strchar_common(args, result, true)
}

/// "strchars()" function: characters, composing ones counted unless the
/// optional `skipcc` argument says otherwise.
pub fn f_strchars(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let skipcc = if args.len() > 1 {
        match strict_bool_arg(&args[1]) {
            Some(flag) => flag,
            None => return,
        }
    } else {
        false
    };
    strchar_common(args, result, skipcc);
}

/// "strdisplaywidth()" function: screen cells, tabs expanded against the
/// optional starting column.
pub fn f_strdisplaywidth(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.string(&args[0]);
    let col = if args.len() > 1 {
        tv_get_number(&args[1]) as c_int
    } else {
        0
    };
    result.write_number(VarNumber::from(linetabsize_text(col, text) - col));
}

/// "strwidth()" function: screen cells, with a tab counting as one.
pub fn f_strwidth(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.string(&args[0]).to_bytes();
    result.write_number(c_string_cells(text) as VarNumber);
}

/// "strtrans()" function: unprintable characters as `^X`/`<xx>`.
pub fn f_strtrans(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.string(&args[0]).to_bytes();
    result.write_string(Some(transstr_owned(text, true).into()));
}

/// "tolower()" function.
pub fn f_tolower(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.string(&args[0]).to_bytes();
    result.write_string(Some(strcase_copy(text, false).into()));
}

/// "toupper()" function.
pub fn f_toupper(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.string(&args[0]).to_bytes();
    result.write_string(Some(strcase_copy(text, true).into()));
}

/// "tr()" function: character-wise translation.
///
/// `fromstr` and `tostr` must hold the same number of characters, which is
/// checked lazily -- either when a mapped character's counterpart runs off
/// the end of `tostr`, or once, the first time an input character is *not*
/// in `fromstr` and the counts can be compared directly. So
/// `tr('a', 'ab', 'x')` is an error but `tr('a', 'a', 'x')` is not.
pub fn f_tr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = NumBuf::new();
    let mut buf2 = NumBuf::new();
    let in_bytes = numbuf.bytes(&args[0]);
    let fromstr = buf.string_chk(&args[1]);
    let tostr = buf2.bytes_chk(&args[2]);

    result.write_string(None);
    let (Some(fromstr), Some(to_bytes)) = (fromstr, tostr) else {
        return; // Type error; the message is already out.
    };
    let from_bytes = fromstr.to_bytes();

    /// The `n`-th character of a set, as its bytes.
    fn nth_char(set: &[u8], n: usize) -> Option<&[u8]> {
        let mut at = 0;
        for _ in 0..n {
            at += cluster_len(set.get(at..).filter(|rest| !rest.is_empty())?);
        }
        let rest = set.get(at..).filter(|rest| !rest.is_empty())?;
        Some(&rest[..cluster_len(rest)])
    }

    let mut out = Vec::<u8>::new();

    let mut lengths_checked = false;
    let mut at = 0;
    'error: {
        while at < in_bytes.len() {
            let cur = &in_bytes[at..];
            let ch = &cur[..cluster_len(cur)];

            // Which character of `fromstr` is this, if any?
            let mut idx = 0;
            let mut found = false;
            let mut from_at = 0;
            while from_at < from_bytes.len() {
                let from_rest = &from_bytes[from_at..];
                let from_ch = &from_rest[..cluster_len(from_rest)];
                if from_ch == ch {
                    found = true;
                    break;
                }
                idx += 1;
                from_at += from_ch.len();
            }

            let replacement = if found {
                match nth_char(to_bytes, idx) {
                    Some(hit) => hit,
                    None => break 'error, // tostr is shorter than fromstr
                }
            } else {
                if !lengths_checked {
                    lengths_checked = true;
                    // `idx` is now `fromstr`'s character count.
                    if char_count(to_bytes) != idx {
                        break 'error;
                    }
                }
                ch
            };

            out.extend_from_slice(replacement);
            at += ch.len();
        }
        result.write_string(Some(out.into()));
        return;
    }
    let fromstr = msg_cstr(fromstr);
    semsg!("E475: Invalid argument: {fromstr}");
}

/// "trim()" function.
///
/// `dir` is 0 (both ends, the default), 1 (leading) or 2 (trailing). With
/// no mask the set trimmed is whitespace plus U+00A0; with one it is
/// exactly the mask's characters, and an empty mask reverts to the
/// default.
pub fn f_trim(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf1 = NumBuf::new();
    let mut buf2 = NumBuf::new();
    let head = buf1.bytes_chk(&args[0]);
    let mut mask = None;
    let mut dir = 0;

    result.write_string(None);
    let Some(bytes) = head else { return };
    if tv_check_for_opt_string_arg(args, 1).is_err() {
        return;
    }

    if args.get(1).is_some_and(TypVal::is_string) {
        mask = buf2.bytes_chk(&args[1]).filter(|mask| !mask.is_empty());
        if args.len() > 2 {
            let Ok(given) = tv_get_number_chk(&args[2]) else {
                return;
            };
            dir = given as c_int;
            if !(0..=2).contains(&dir) {
                let arg0 = msg_cstr(numbuf.string(&args[2]));
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
        }
    }

    // Whitespace and NBSP by default, else exactly the mask's set.
    let trimmable = |c: c_int| -> bool {
        let Some(mask) = mask else {
            return c <= c_int::from(b' ') || c == 0xa0;
        };
        clusters(mask).any(|(_, mask_char)| mask_char == c)
    };

    let mut start = 0;
    if dir == 0 || dir == 1 {
        while start < bytes.len() && trimmable(char_at(&bytes[start..])) {
            start += cluster_len(&bytes[start..]);
        }
    }

    let mut end = bytes.len();
    if dir == 0 || dir == 2 {
        while end > start {
            // Step back over one whole character.
            let prev = end - 1 - head_off(bytes, end - 1);
            if !trimmable(char_at(&bytes[prev..])) {
                break;
            }
            end = prev;
        }
    }

    result.write_string(Some(ThinCString::from_bytes(&bytes[start..end])));
}
