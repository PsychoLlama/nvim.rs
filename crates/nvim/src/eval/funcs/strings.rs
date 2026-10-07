//! Building and reshaping strings: escaping, formatting, splitting,
//! substituting, hashing, time formatting and spelling.
#![forbid(unsafe_code)]

use super::wrappers::{arg_bool_chk, arg_number, arg_number_chk, blob_alloc_ret, non_zero_arg};
use super::{NSUBEXP, VSE_NONE};
use crate::eval::do_string_sub;
use crate::eval::typval::{
    NumBuf, blob_bytes, list_extend, list_len, tv_check_for_nonempty_string_arg,
    tv_check_for_string_arg, tv_check_num, tv_list_alloc_ret,
};
use crate::ex_getln::fnameescape;
use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::keycodes::escape_ks;
use crate::mbyte::{Converter, char_at, cluster_len, enc_locale, encode_char};
use crate::memline::Lines;
use crate::memory::{ThinCString, XString};
use crate::message::e_no_spell;
use crate::message::{emsg, str2special_bytes};
use crate::option::SavedCpo;
use crate::option::vars::p_enc;
use crate::optionstr::OptString;
use crate::os::cshim::{gettext, gettext_owned};
use crate::os::time::{
    os_localtime_r, os_mktime, os_strftime, os_strptime, os_time_raw, tm_zeroed,
};
use crate::regexp::{OwnedProg, RE_MAGIC, RE_STRING, reg_submatch, reg_submatch_list};
use crate::search::FORWARD;
use crate::semsg;
use crate::sha256::hex_digest;
use crate::spell::{SMT_ALL, eval_soundfold, parse_spelllang, spell_check_text, spell_move_to};
use crate::spellsuggest::spell_suggest_list;
use crate::strings::{escaped_bytes, format_typvals, shellescape_of};
use crate::types::{
    EvalFuncData, Hlf, List, TypVal, VAR_BLOB, VAR_LIST, VAR_STRING, VarNumber, kListLenMayKnow,
    time_t, tm,
};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};

/// `char2nr({string} [, {utf8}])` — the first character's code point. The
/// second argument only has to type-check; nvim is always UTF-8.
pub fn f_char2nr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if args.len() > 1 && !tv_check_num(&args[1]) {
        return;
    }
    result.write_number(VarNumber::from(char_at(numbuf.bytes(&args[0]))));
}

/// `escape({string}, {chars})` — backslash every byte listed in `chars`.
pub fn f_escape(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = NumBuf::new();
    let text = numbuf.string(&args[0]);
    let chars = buf.string(&args[1]);
    result.write_string(Some(escaped_bytes(text, chars).into()));
}

/// `fnameescape({string})`.
pub fn f_fnameescape(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let name = numbuf.string(&args[0]);
    result.write_string(Some(fnameescape(name, VSE_NONE as c_int).into()));
}

/// `gettext({string})` — a no-op while no message catalogs ship, but it
/// still requires a non-empty String.
pub fn f_gettext(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if tv_check_for_nonempty_string_arg(args, 0).is_err() {
        return;
    }
    // The check above proved argument 0 is a non-empty String.
    let msgid = args[0].string_cstr().unwrap_or(c"");
    result.write_string(Some(gettext_owned(msgid).into()));
}

/// `keytrans({string})` — the readable spelling of a key sequence.
pub fn f_keytrans(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_empty(VAR_STRING);
    if tv_check_for_string_arg(args, 0).is_err() {
        return;
    }
    let Some(text) = args[0].string_ref() else {
        return;
    };
    let escaped = escape_ks(text.as_cstr());
    let readable = str2special_bytes(escaped.as_cstr(), true, true);
    result.write_string(Some(readable.into()));
}

/// `nr2char({number} [, {utf8}])`.
pub fn f_nr2char(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut error = false;
    if args.len() > 1 && !tv_check_num(&args[1]) {
        return;
    }
    let num = arg_number_chk(&args[0], Some(&mut error));
    if error {
        return;
    }
    if num < 0 {
        let msg = c"E5070: Character number must not be less than zero";
        emsg(gettext(msg));
        return;
    }
    if num > c_int::MAX as VarNumber {
        semsg!(
            "E5071: Character number must not be greater than INT_MAX ({})",
            c_int::MAX
        );
        return;
    }
    let mut buf = [0u8; 6];
    let len = encode_char(num as c_int, &mut buf);
    result.write_string(Some(ThinCString::from_bytes(&buf[..len])));
}

/// `printf({fmt}, ...)` — measured, then formatted into an exact
/// allocation; a format that reports an error answers the null String.
pub fn f_printf(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(format_typvals(&args[0], &args[1..]).map(ThinCString::from));
}

/// `repeat({expr}, {count})` — for a List, a Blob or a String.
pub fn f_repeat(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let n = arg_number(&args[1]);
    match args[0].v_type() {
        VAR_LIST => repeat_list(args, result, n),
        VAR_BLOB => repeat_blob(args, result, n),
        _ => repeat_string(args, result, n),
    }
}

fn repeat_list(args: &[TypVal], result: &mut TypVal, n: VarNumber) {
    let src = args[0].list_shared();
    // The length hint is upstream's; a non-positive count contributes
    // nothing rather than a negative capacity.
    let hint = VarNumber::from(n > 0) * n * VarNumber::from(list_len(args[0].list_ref()));
    tv_list_alloc_ret(result, hint as isize);
    let Some(out) = result.list_shared() else {
        return;
    };
    for _ in 0..n.max(0) {
        list_extend(out, src, None);
    }
}

fn repeat_blob(args: &[TypVal], result: &mut TypVal, n: VarNumber) {
    let src = blob_bytes(args[0].blob_ref());
    let out = blob_alloc_ret(result);
    if src.is_empty() || n <= 0 {
        return;
    }
    // Upstream computes the total in `int`; a product that does not fit
    // reads as non-positive and the repeat is dropped.
    let len = (src.len() as VarNumber * n) as c_int;
    if len <= 0 {
        return;
    }
    let len = usize::try_from(len).expect("a positive length");
    for room in out.claim(len).chunks_mut(src.len()) {
        room.copy_from_slice(&src[..room.len()]);
    }
}

fn repeat_string(args: &[TypVal], result: &mut TypVal, n: VarNumber) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);
    if n <= 0 {
        return;
    }
    let p = numbuf.bytes(&args[0]);
    let slen = p.len();
    if slen == 0 {
        return;
    }
    // Upstream's overflow guard, in `size_t` as upstream writes it: a
    // product that wrapped does not divide back to the source length.
    // A product that fits is asked for in earnest, and reports E41 if
    // the allocation fails.
    let len = slen.wrapping_mul(n as usize);
    if len.wrapping_div(n as usize) != slen {
        return;
    }
    result.write_string(Some(p.repeat(n as usize).into()));
}

/// `sha256({string})` — also accepts a Blob, whose bytes are hashed as they
/// are rather than up to the first NUL.
pub fn f_sha256(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    let hash = if args[0].v_type() == VAR_BLOB {
        hex_digest(args[0].blob_ref().map_or(&[][..], |b| b.bytes()))
    } else {
        hex_digest(numbuf.bytes(&args[0]))
    };
    result.write_string(Some(ThinCString::from_bytes(hash.as_bytes())));
}

/// `shellescape({string} [, {special}])`.
pub fn f_shellescape(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let do_special = args.get(1).is_some_and(non_zero_arg);
    let text = numbuf.string(&args[0]);
    result.write_string(Some(shellescape_of(text, do_special, do_special).into()));
}

/// `soundfold({word})`.
pub fn f_soundfold(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let word = numbuf.string(&args[0]);
    result.write_string(Some(eval_soundfold(word).into()));
}

/// Turn 'spell' on for the duration of `body`, loading the spell languages
/// if they are not loaded yet, and report E756 if none is configured.
///
/// Both spelling builtins open this way, and both must put the window's own
/// 'spell' back on every path out — including the error one.
fn with_spell(body: impl FnOnce()) {
    let mut win = Win::current();
    let saved = win.w_onebuf_opt.wo_spell;
    if win.w_onebuf_opt.wo_spell == 0 {
        let _ = parse_spelllang(win);
        win.w_onebuf_opt.wo_spell = 1;
    }
    if win.syntax().b_p_spl.first_byte() == 0 {
        emsg(gettext(e_no_spell));
    } else {
        body();
    }
    win.w_onebuf_opt.wo_spell = saved;
}

/// `spellbadword([{sentence}])` — the first misspelling and why it is one.
pub fn f_spellbadword(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // The misspelt word, copied out: the cursor line's text when the search
    // moved the cursor onto one, the argument's otherwise.
    let mut word: Vec<u8> = Vec::new();
    let mut attr: Hlf = HLF_COUNT;
    let mut reported = false;
    with_spell(|| {
        reported = true;
        if args.is_empty() {
            let len = spell_move_to(Win::current(), FORWARD, SMT_ALL, true, Some(&mut attr));
            if len != 0 {
                let cursor = Win::current().w_cursor;
                let col = usize::try_from(cursor.col).unwrap_or(0);
                let line = Lines::current().line(cursor.lnum).to_vec();
                word = line[col..(col + len).min(line.len())].to_vec();
                Win::current().w_set_curswant = true;
            }
        } else if Buf::current().b_s.b_p_spl.first_byte() != 0 {
            let text = numbuf.string_chk(&args[0]);
            let mut capcol: c_int = -1;
            if let Some(text) = text {
                let end = text.count_bytes();
                let mut offset = 0;
                while offset < end {
                    let rest = &text[offset..];
                    let len = spell_check_text(Win::current(), rest, &mut attr, &mut capcol, false);
                    if attr != HLF_COUNT {
                        word = rest.to_bytes()[..len.min(rest.count_bytes())].to_vec();
                        break;
                    }
                    offset += len;
                    capcol -= len as c_int;
                }
            }
        }
    });
    if !reported {
        return;
    }
    let list = tv_list_alloc_ret(result, 2);
    list.push_bytes(Some(&word));
    let reason: Option<&CStr> = match attr {
        HLF_SPB => Some(c"bad"),
        HLF_SPR => Some(c"rare"),
        HLF_SPL => Some(c"local"),
        HLF_SPC => Some(c"caps"),
        _ => None,
    };
    match reason {
        Some(r) => list.push_str(Some(r)),
        None => list.push_bytes(None),
    }
}

/// `spellsuggest({word} [, {max} [, {capital}]])`.
pub fn f_spellsuggest(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut found: Vec<XString> = Vec::new();
    let mut reported = false;
    with_spell(|| {
        reported = true;
        let str = numbuf.string(&args[0]);
        let mut typeerr = false;
        let (maxcount, need_capital) = if args.len() <= 1 {
            (25, false)
        } else {
            let maxcount = arg_number_chk(&args[1], Some(&mut typeerr)) as c_int;
            // A non-positive maximum leaves the list empty, and does so
            // before the type error from argument 2 could be reported.
            if maxcount <= 0 {
                return;
            }
            let need_capital = args.len() > 2 && arg_number_chk(&args[2], Some(&mut typeerr)) != 0;
            if typeerr {
                return;
            }
            (maxcount, need_capital)
        };
        found = spell_suggest_list(str, maxcount, need_capital, false);
    });
    if !reported {
        return;
    }
    let list = tv_list_alloc_ret(result, found.len() as isize);
    for word in found {
        list.push(TypVal::string(Some(word.into())));
    }
}

/// `split({string} [, {pattern} [, {keepempty}]])`.
pub fn f_split(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut patbuf = NumBuf::new();
    // 'cpoptions' is cleared around the split so that its flags cannot
    // change what the pattern means.
    let _cpo = SavedCpo::empty();
    let str = numbuf.string(&args[0]);
    let mut typeerr = false;
    let mut keepempty = false;
    let mut pat = None;
    if args.len() > 1 {
        pat = patbuf.string_chk(&args[1]);
        if pat.is_none() {
            typeerr = true;
        }
        if args.len() > 2 {
            keepempty = arg_bool_chk(&args[2], &mut typeerr) != 0;
        }
    }
    // An absent or empty pattern splits on runs of whitespace.
    let pat = pat.filter(|pat| !pat.is_empty()).unwrap_or(c"[\\x01- ]\\+");
    let list = tv_list_alloc_ret(result, kListLenMayKnow as isize);
    if !typeerr && let Some(mut prog) = OwnedProg::compile(pat, RE_MAGIC + RE_STRING) {
        split_into(list, str, &mut prog, keepempty);
    }
}

/// Append `str`'s pieces to `list`, separating on `prog`.
///
/// The empty-piece rule is the subtle part and is upstream's: a piece is
/// kept when `keepempty` is set, when it is non-empty, or when it is an
/// empty piece produced by a *widening* match in the middle of the string
/// (`end < endp[0]`) after at least one piece is already there. That last
/// clause is what makes `split("aXbXc", "X*", 1)` differ from the plain
/// form.
fn split_into(list: &mut List, subject: &CStr, prog: &mut OwnedProg, keepempty: bool) {
    // Where the piece being cut starts, and how far into it the next match
    // may begin. The pattern is run against the *tail* rather than the whole
    // subject, because a `^` in it anchors at each piece.
    let mut at = 0;
    let mut col = 0;
    loop {
        let tail = &subject[at..];
        let rest = tail.to_bytes();
        if rest.is_empty() && !keepempty {
            break;
        }
        let found = if rest.is_empty() {
            None
        } else {
            prog.exec_nl(tail, col, false)
        };
        let matched = found.is_some();
        let (start, match_end) = match found.as_ref().and_then(|m| m.group(0)) {
            Some(span) => (span.start, span.end),
            None => (rest.len(), 0),
        };
        if keepempty
            || start > 0
            || (list_len(Some(list)) > 0 && !rest.is_empty() && matched && start < match_end)
        {
            list.push_bytes(Some(&rest[..start]));
        }
        if !matched {
            break;
        }
        // An empty match would not advance, so the next attempt starts
        // one character further in while the piece stays put.
        col = if match_end > 0 { 0 } else { cluster_len(rest) };
        at += match_end;
    }
}

/// `strftime({format} [, {time}])`.
/// The conversions between 'encoding' and the locale's own encoding, one
/// way and the other; `None` where no conversion is needed.
fn locale_converters() -> (Option<Converter>, Option<Converter>) {
    let locale = enc_locale();
    let locale = locale.as_ref().map_or(c"", XString::as_cstr);
    let to_locale = p_enc(|value| Converter::new(value, locale));
    let from_locale = p_enc(|value| Converter::new(locale, value));
    (to_locale, from_locale)
}

/// `strftime({format} [, {time}])`.
pub fn f_strftime(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    let format = numbuf.string(&args[0]);
    let seconds: time_t = if args.len() > 1 {
        arg_number(&args[1]) as time_t
    } else {
        os_time_raw()
    };
    let mut curtime: tm = tm_zeroed();
    if !os_localtime_r(seconds, &mut curtime) {
        result.write_string(Some(ThinCString::from_cstr(gettext(c"(Invalid)"))));
        return;
    }
    let (to_locale, from_locale) = locale_converters();
    // A format that will not convert formats nothing.
    let out = match &to_locale {
        Some(conv) => conv
            .convert(format.to_bytes())
            .map(|format| os_strftime(XString::from_bytes(&format).as_cstr(), &curtime))
            .unwrap_or_default(),
        None => os_strftime(format, &curtime),
    };
    result.write_string(match &from_locale {
        Some(conv) => conv.convert(&out).map(ThinCString::from),
        None => Some(ThinCString::from(out)),
    });
}

/// `strptime({format}, {timestring})`.
pub fn f_strptime(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut fmt_buf = NumBuf::new();
    let mut str_buf = NumBuf::new();
    // strptime() is asked to determine DST itself.
    let mut tmval: tm = tm {
        tm_isdst: -1,
        ..tm_zeroed()
    };
    let format = fmt_buf.string(&args[0]);
    let text = str_buf.string(&args[1]);
    let (to_locale, _) = locale_converters();
    let converted = to_locale.map(|conv| {
        conv.convert(format.to_bytes())
            .map(|f| XString::from_bytes(&f))
    });
    let format = match &converted {
        Some(converted) => converted.as_ref().map(XString::as_cstr),
        None => Some(format),
    };
    // `mktime` reporting -1 is indistinguishable from a genuine
    // timestamp of -1, and upstream treats both as failure.
    let parsed = format.is_some_and(|format| !os_strptime(text, format, &mut tmval).is_null());
    let seconds = if parsed {
        os_mktime(&mut tmval) as VarNumber
    } else {
        -1
    };
    result.write_number(if seconds == -1 { 0 } else { seconds });
}

/// `submatch({nr} [, {list}])`.
pub fn f_submatch(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut error = false;
    // SAFETY throughout: the arguments are live typvals.
    let no = arg_number_chk(&args[0], Some(&mut error)) as c_int;
    if error {
        return;
    }
    if !(0..NSUBEXP as c_int).contains(&no) {
        semsg!("E935: Invalid submatch number: {no}");
        return;
    }
    let as_list = if args.len() > 1 {
        let flag = arg_number_chk(&args[1], Some(&mut error));
        if error {
            return;
        }
        flag != 0
    } else {
        false
    };
    if as_list {
        result.write_list(reg_submatch_list(no));
    } else {
        result.write_string(reg_submatch(no).map(ThinCString::from));
    }
}

/// `substitute({string}, {pat}, {sub}, {flags})` — `{sub}` may be a Funcref,
/// in which case it is handed on rather than read as a String.
pub fn f_substitute(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut patbuf = NumBuf::new();
    let mut subbuf = NumBuf::new();
    let mut flagsbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    let str = numbuf.string_chk(&args[0]);
    let pat = patbuf.string_chk(&args[1]);
    let flg = flagsbuf.string_chk(&args[3]);
    let (sub, expr) = if args[2].is_func() {
        (None, Some(&args[2]))
    } else {
        (subbuf.string_chk(&args[2]), None)
    };
    result.write_string(match (str, pat, flg) {
        (Some(str), Some(pat), Some(flg)) if sub.is_some() || expr.is_some() => {
            Some(do_string_sub(str, pat, sub, expr, flg).into())
        }
        _ => None,
    });
}
