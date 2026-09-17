//! Building and reshaping strings: escaping, formatting, splitting,
//! substituting, hashing, time formatting and spelling.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{
    arg_bool_chk, arg_number, arg_number_chk, arg_string, arg_string_chk, blob_alloc_ret,
    list_alloc_ret, non_zero_arg,
};
use super::{GA_EMPTY_INIT_VALUE, NSUBEXP, VSE_NONE};
use crate::cstr;
use crate::cursor::get_cursor_pos_ptr;
use crate::eval::do_string_sub;
use crate::eval::typval::{
    NumBuf, blob_bytes, list_extend, list_len, tv_check_for_nonempty_string_arg,
    tv_check_for_string_arg, tv_check_num,
};
use crate::ex_getln::vim_strsave_fnameescape;
use crate::garray::ga_clear;
use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::keycodes::vim_strsave_escape_ks;
use crate::mbyte::{
    convert_setup, enc_locale, string_convert, utf_char2bytes, utf_ptr2char, utfc_ptr2len,
};
use crate::memory::XString;
use crate::memory::{xfree, xmalloc, xmallocz, xmemdupz, xstrdup};
use crate::message::e_no_spell;
use crate::message::state::did_emsg;
use crate::message::{emsg, str2special_save};
use crate::option::SavedCpo;
use crate::option::vars::p_enc;
use crate::optionstr::LocalOptStr;
use crate::os::cshim::{gettext, gettext_ptr};
use crate::os::time::{os_localtime_r, os_strptime, tm_zeroed};
use crate::regexp::{
    RE_MAGIC, RE_STRING, reg_submatch, reg_submatch_list, vim_regcomp, vim_regexec_nl, vim_regfree,
};
use crate::search::FORWARD;
use crate::semsg;
use crate::sha256::hex_digest;
use crate::spell::{SMT_ALL, eval_soundfold, parse_spelllang, spell_check, spell_move_to};
use crate::spellsuggest::spell_suggest_list;
use crate::strings::{vim_strsave_escaped, vim_strsave_shellescape, vim_vsnprintf_typval};
use crate::types::{
    CONV_NONE, ColNr, EvalFuncData, GArray, Hlf, List, NUL, RegMatch, RegProg, TypVal, VAR_BLOB,
    VAR_LIST, VAR_STRING, VarNumber, VimConv, kListLenMayKnow, time_t, tm,
};
use crate::winlayer::{Buf, Win};
use ::libc::{mktime, strftime, time};
use core::ffi::{CStr, VaList, c_char, c_int, c_void};
use core::ptr;

/// The placeholder `va_list` the typval formatter is handed. It is never
/// read: `vim_vsnprintf_typval` takes its arguments from the typval array
/// whenever that is non-null, which is the only way this family calls it.
///
/// # Safety
/// The result must only reach `vim_vsnprintf_typval` with a non-null
/// typval argument list.
unsafe fn dummy_ap() -> VaList<'static> {
    // SAFETY: a zeroed `va_list` is inert as long as nothing reads it, and
    // the typval overload never does. This is what the transpiled body did
    // through a zeroed static.
    unsafe { core::mem::transmute::<[u8; 24], VaList<'static>>([0u8; 24]) }
}

/// A conversion descriptor that has not been set up yet.
const CONV_NONE_INIT: VimConv = VimConv {
    vc_type: CONV_NONE,
    vc_factor: 0,
    vc_fd: ptr::null_mut(),
    vc_fail: false,
};

/// `char2nr({string} [, {utf8}])` — the first character's code point. The
/// second argument only has to type-check; nvim is always UTF-8.
pub fn f_char2nr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the arguments are live typvals.
    if args.len() > 1 && !tv_check_num(&args[1]) {
        return;
    }
    result.write_number(unsafe { utf_ptr2char(arg_string(&mut numbuf, &args[0])) } as VarNumber);
}

/// `escape({string}, {chars})` — backslash every byte listed in `chars`.
pub fn f_escape(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = NumBuf::new();
    let str = arg_string(&mut numbuf, &args[0]);
    let chars = arg_string(&mut buf, &args[1]);
    // SAFETY: both are NUL-terminated and outlive the call, `chars` because
    // `buf` does.
    result.write_string(unsafe { vim_strsave_escaped(str, chars) });
}

/// `fnameescape({string})`.
pub fn f_fnameescape(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: `&args[0]` is a live typval.
    let name = arg_string(&mut numbuf, &args[0]);
    result.write_string(unsafe { vim_strsave_fnameescape(name, VSE_NONE as c_int) });
}

/// `gettext({string})` — a no-op while no message catalogs ship, but it
/// still requires a non-empty String.
pub fn f_gettext(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: `&args[0]` is a live typval.
    if tv_check_for_nonempty_string_arg(args, 0).is_err() {
        return;
    }
    // SAFETY: the check above proved argument 0 is a non-empty String, so
    // the value holds a live NUL-terminated string.
    result.write_string(unsafe { xstrdup(gettext_ptr(args[0].string_or_null()).as_ptr()) });
}

/// `keytrans({string})` — the readable spelling of a key sequence.
pub fn f_keytrans(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_empty(VAR_STRING);
    // SAFETY throughout: `&args[0]` is a live typval; after the check the union
    // holds a String pointer, which may still be null.
    if tv_check_for_string_arg(args, 0).is_err() || args[0].string_or_null().is_null() {
        return;
    }
    let escaped = unsafe { vim_strsave_escape_ks(args[0].string_or_null()) };
    result.write_string(unsafe { str2special_save(escaped, true, true) });
    unsafe { xfree(escaped.cast::<c_void>()) };
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
        // SAFETY throughout: a literal message.
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
    let mut buf: [c_char; 6] = [0; 6];
    // SAFETY: `buf` has room for the longest UTF-8 sequence
    // `utf_char2bytes` writes, and the returned length is what it wrote.
    let len = unsafe { utf_char2bytes(num as c_int, buf.as_mut_ptr()) };
    let src = buf.as_ptr().cast::<c_void>();
    result.write_string(unsafe { xmemdupz(src, len as usize) }.cast::<c_char>());
}

/// `printf({fmt}, ...)` — measure, then format into an exact allocation.
///
/// The `did_emsg` dance is load-bearing: the measuring pass is where a bad
/// format reports, and the formatting pass is skipped when it did. The
/// caller's own `did_emsg` is restored by OR-ing it back, so an error
/// raised before this call is not lost and one raised inside it is.
pub fn f_printf(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(ptr::null_mut());

    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);
    let mut buf = NumBuf::new();
    // SAFETY throughout: `buf` outlives both passes, and the conversions read
    // the arguments after the format. The `dummy_ap` va_list is never read
    // because a typval slice is what selects the Vimscript overload, which is
    // how every caller of this entry point uses it.
    let fmt = arg_string(&mut buf, &args[0]);
    let rest = Some(&args[1..]);
    let len = unsafe { vim_vsnprintf_typval(ptr::null_mut(), 0, fmt, dummy_ap(), rest) };
    if did_emsg.get() == 0 {
        let s = unsafe { xmalloc(len as usize + 1) }.cast::<c_char>();
        result.write_string(s);
        unsafe { vim_vsnprintf_typval(s, len as usize + 1, fmt, dummy_ap(), rest) };
    }
    did_emsg.set(did_emsg.get() | saved_did_emsg);
}

/// `repeat({expr}, {count})` — for a List, a Blob or a String.
pub fn f_repeat(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments are live typvals.
    let n = arg_number(&args[1]);
    match args[0].v_type() {
        VAR_LIST => repeat_list(args, result, n),
        VAR_BLOB => repeat_blob(args, result, n),
        _ => repeat_string(args, result, n),
    }
}

fn repeat_list(args: &[TypVal], result: &mut TypVal, n: VarNumber) {
    // SAFETY: the caller's obligation.
    let src = args[0].list_or_null();
    // The length hint is upstream's; a non-positive count contributes
    // nothing rather than a negative capacity.
    let hint = VarNumber::from(n > 0) * n * VarNumber::from(list_len(unsafe { src.as_ref() }));
    let out = list_alloc_ret(result, hint as isize);
    for _ in 0..n.max(0) {
        unsafe { list_extend(out, src, None) };
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
    result.write_string(ptr::null_mut());
    if n <= 0 {
        return;
    }
    // SAFETY throughout: the caller's obligation; `p` is NUL-terminated and outlives
    // the copies made from it.
    let p = arg_string(&mut numbuf, &args[0]);
    let slen = unsafe { cstr::bytes_at(p) }.len();
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
    let r = unsafe { xmallocz(len) }.cast::<c_char>();
    for i in 0..n as usize {
        unsafe { (r.add(i * slen)).cast::<u8>().copy_from(p.cast(), slen) };
    }
    result.write_string(r);
}

/// `sha256({string})` — also accepts a Blob, whose bytes are hashed as they
/// are rather than up to the first NUL.
pub fn f_sha256(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    let hash = if args[0].v_type() == VAR_BLOB {
        let blob = args[0].blob_or_null();
        let ga = if blob.is_null() {
            GA_EMPTY_INIT_VALUE
        } else {
            unsafe { (*blob).bv_ga }
        };
        let bytes = if ga.ga_data.is_null() {
            &[][..]
        } else {
            let data = ga.ga_data.cast::<u8>();
            unsafe { core::slice::from_raw_parts(data, ga.ga_len as usize) }
        };
        hex_digest(bytes)
    } else {
        // SAFETY throughout: `tv_get_string` hands back a NUL-terminated buffer.
        let p = arg_string(&mut numbuf, &args[0]);
        let bytes = unsafe { core::slice::from_raw_parts(p.cast::<u8>(), cstr::bytes_at(p).len()) };
        hex_digest(bytes)
    };
    // SAFETY throughout: `hash` is a live buffer of `hash.len()` bytes.
    let owned = unsafe { xmemdupz(hash.as_ptr().cast::<c_void>(), hash.len()) };
    result.write_string(owned.cast::<c_char>());
}

/// `shellescape({string} [, {special}])`.
pub fn f_shellescape(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the arguments are live typvals.
    let do_special = args.get(1).is_some_and(non_zero_arg);
    let str = arg_string(&mut numbuf, &args[0]);
    result.write_string(unsafe { vim_strsave_shellescape(str, do_special, do_special) });
}

/// `soundfold({word})`.
pub fn f_soundfold(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: `&args[0]` is a live typval.
    result.write_string(unsafe { eval_soundfold(arg_string(&mut numbuf, &args[0])) });
}

/// Turn 'spell' on for the duration of `body`, loading the spell languages
/// if they are not loaded yet, and report E756 if none is configured.
///
/// Both spelling builtins open this way, and both must put the window's own
/// 'spell' back on every path out — including the error one.
fn with_spell(body: impl FnOnce()) {
    // SAFETY throughout: `curwin` names a live window from startup to exit, and the
    // spell state hanging off it is initialised with the window.
    let mut win = Win::current();
    let saved = win.w_onebuf_opt.wo_spell;
    if win.w_onebuf_opt.wo_spell == 0 {
        let _ = parse_spelllang(win);
        win.w_onebuf_opt.wo_spell = 1;
    }
    if unsafe { (*win.w_s).b_p_spl.first_byte() } == 0 {
        emsg(gettext(e_no_spell));
    } else {
        body();
    }
    win.w_onebuf_opt.wo_spell = saved;
}

/// `spellbadword([{sentence}])` — the first misspelling and why it is one.
pub fn f_spellbadword(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut word: *const c_char = c"".as_ptr();
    let mut attr: Hlf = HLF_COUNT;
    let mut len: usize = 0;
    let mut reported = false;
    // SAFETY throughout the closure: `curwin`/`curbuf` are live, and an
    // argument is a live typval. `spell_check` advances `str` by the
    // length it reports, which never passes the terminator.
    with_spell(|| {
        reported = true;
        if args.is_empty() {
            let at = &raw mut attr;
            len = unsafe { spell_move_to(Win::current(), FORWARD, SMT_ALL, true, at) };
            if len != 0 {
                word = get_cursor_pos_ptr();
                Win::current().w_set_curswant = true;
            }
        } else if Buf::current().b_s.b_p_spl.first_byte() != 0 {
            let mut str = arg_string_chk(&mut numbuf, &args[0]);
            let mut capcol: c_int = -1;
            if !str.is_null() {
                while unsafe { *str } != NUL as c_char {
                    let p = str as *mut c_char;
                    let (at, cap) = (&raw mut attr, &raw mut capcol);
                    len = unsafe { spell_check(Win::current(), p, at, cap, false) };
                    if attr != HLF_COUNT {
                        word = str;
                        break;
                    }
                    str = unsafe { str.add(len) };
                    capcol -= len as c_int;
                    len = 0;
                }
            }
        }
    });
    if !reported {
        return;
    }
    debug_assert!(len <= c_int::MAX as usize);
    let list = list_alloc_ret(result, 2);
    unsafe { (*list).push_string(word, len as isize) };
    let reason: Option<&CStr> = match attr {
        HLF_SPB => Some(c"bad"),
        HLF_SPR => Some(c"rare"),
        HLF_SPL => Some(c"local"),
        HLF_SPC => Some(c"caps"),
        _ => None,
    };
    match reason {
        Some(r) => unsafe { (*list).push_string(r.as_ptr(), r.count_bytes() as isize) },
        None => unsafe { (*list).push_string(ptr::null(), -1) },
    }
}

/// `spellsuggest({word} [, {max} [, {capital}]])`.
pub fn f_spellsuggest(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut ga: GArray = GA_EMPTY_INIT_VALUE;
    let mut reported = false;
    with_spell(|| {
        reported = true;
        let str = arg_string(&mut numbuf, &args[0]);
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
        let (out, word) = (&raw mut ga, str as *mut c_char);
        // SAFETY: `ga` is a local garray that `spell_suggest_list` fills and
        // `ga_clear` frees, and `word` is the NUL-terminated argument.
        unsafe { spell_suggest_list(out, word, maxcount, need_capital, false) };
    });
    if !reported {
        return;
    }
    let list = list_alloc_ret(result, ga.ga_len as isize);
    for i in 0..ga.ga_len {
        // SAFETY: the garray holds `ga_len` allocated strings, and the list
        // takes each one over.
        let word = unsafe { *ga.ga_data.cast::<*mut c_char>().offset(i as isize) };
        unsafe { (*list).push_allocated_string(word) };
    }
    unsafe { ga_clear(&raw mut ga) };
}

/// `split({string} [, {pattern} [, {keepempty}]])`.
pub fn f_split(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut patbuf = NumBuf::new();
    // 'cpoptions' is cleared around the split so that its flags cannot
    // change what the pattern means.
    let _cpo = SavedCpo::empty();
    // SAFETY throughout: the arguments are live typvals, `patbuf` outlives the calls
    // that may fill it, and the compiled program is freed before returning.
    let str = arg_string(&mut numbuf, &args[0]);
    let mut typeerr = false;
    let mut keepempty = false;
    let mut pat: *const c_char = ptr::null();
    if args.len() > 1 {
        pat = arg_string_chk(&mut patbuf, &args[1]);
        if pat.is_null() {
            typeerr = true;
        }
        if args.len() > 2 {
            keepempty = arg_bool_chk(&args[2], &mut typeerr) != 0;
        }
    }
    // An absent or empty pattern splits on runs of whitespace.
    if pat.is_null() || unsafe { *pat } == NUL as c_char {
        pat = c"[\\x01- ]\\+".as_ptr();
    }
    let list = list_alloc_ret(result, kListLenMayKnow as isize);
    if !typeerr {
        let prog = unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) };
        if !prog.is_null() {
            unsafe { split_into(list, str, prog, keepempty) };
            unsafe { vim_regfree(prog) };
        }
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
///
/// # Safety
/// `list` is a live list, `str` is NUL-terminated, and `prog` is a compiled
/// program the caller frees.
unsafe fn split_into(list: *mut List, mut str: *const c_char, prog: *mut RegProg, keepempty: bool) {
    // SAFETY throughout: the caller's obligation. The match positions come back
    // pointing into `str`, so every pointer difference below is within one
    // allocation.
    let mut regmatch: RegMatch = RegMatch {
        regprog: prog,
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut col: ColNr = 0;
    while unsafe { *str } != NUL as c_char || keepempty {
        let matched = unsafe { *str } != NUL as c_char
            && unsafe { vim_regexec_nl(&raw mut regmatch, str, col) };
        let end: *const c_char = if matched {
            regmatch.startp[0]
        } else {
            unsafe { str.add(cstr::bytes_at(str).len()) }
        };
        if keepempty
            || end > str
            || (list_len(unsafe { list.as_ref() }) > 0
                && unsafe { *str } != NUL as c_char
                && matched
                && end < regmatch.endp[0] as *const c_char)
        {
            unsafe { (*list).push_string(str, end.offset_from(str) as isize) };
        }
        if !matched {
            break;
        }
        // An empty match would not advance, so the next attempt starts
        // one character further in while `str` stays put.
        col = if regmatch.endp[0] > str as *mut c_char {
            0
        } else {
            unsafe { utfc_ptr2len(regmatch.endp[0]) as ColNr }
        };
        str = regmatch.endp[0];
    }
}

/// `strftime({format} [, {time}])`.
pub fn f_strftime(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    // SAFETY throughout: the arguments are live typvals; the two conversion
    // descriptors are opened and closed here, and `enc` is freed on every
    // path out.
    let mut p = arg_string(&mut numbuf, &args[0]) as *mut c_char;
    let seconds: time_t = if args.len() > 1 {
        arg_number(&args[1]) as time_t
    } else {
        unsafe { time(ptr::null_mut()) }
    };
    let mut curtime: tm = tm_zeroed();
    if !os_localtime_r(seconds, &mut curtime) {
        result.write_string(unsafe { xstrdup(gettext(c"(Invalid)").as_ptr()) });
        return;
    }
    let mut conv: VimConv = CONV_NONE_INIT;
    let enc = unsafe { enc_locale() };
    let _ = p_enc(|value| unsafe { convert_setup(&raw mut conv, value.as_ptr().cast_mut(), enc) });
    if conv.vc_type != CONV_NONE {
        p = unsafe { string_convert(&raw mut conv, p, ptr::null_mut()) };
    }
    let mut out: [c_char; 256] = [0; 256];
    if p.is_null() || unsafe { strftime(out.as_mut_ptr(), out.len(), p, &raw mut curtime) } == 0 {
        out[0] = NUL as c_char;
    }
    if conv.vc_type != CONV_NONE {
        unsafe { xfree(p.cast::<c_void>()) };
    }
    // The reverse conversion reuses `conv`, so it must be set up again
    // in the other direction before the result is converted back.
    let _ = p_enc(|value| unsafe { convert_setup(&raw mut conv, enc, value.as_ptr().cast_mut()) });
    result.write_string(if conv.vc_type != CONV_NONE {
        unsafe { string_convert(&raw mut conv, out.as_mut_ptr(), ptr::null_mut()) }
    } else {
        unsafe { xstrdup(out.as_mut_ptr()) }
    });
    let _ = unsafe { convert_setup(&raw mut conv, ptr::null_mut(), ptr::null_mut()) };
    unsafe { xfree(enc.cast::<c_void>()) };
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
    // SAFETY throughout: the arguments are live typvals, the two scratch buffers
    // outlive the calls that may fill them, and `enc` and the converted
    // format are freed on every path out.
    let mut fmt = arg_string(&mut fmt_buf, &args[0]) as *mut c_char;
    let str = arg_string(&mut str_buf, &args[1]) as *mut c_char;
    let mut conv: VimConv = CONV_NONE_INIT;
    let enc = unsafe { enc_locale() };
    let _ = p_enc(|value| unsafe { convert_setup(&raw mut conv, value.as_ptr().cast_mut(), enc) });
    if conv.vc_type != CONV_NONE {
        fmt = unsafe { string_convert(&raw mut conv, fmt, ptr::null_mut()) };
    }
    // `mktime` reporting -1 is indistinguishable from a genuine
    // timestamp of -1, and upstream treats both as failure.
    let parsed = !fmt.is_null()
        && !os_strptime(
            unsafe { CStr::from_ptr(str) },
            unsafe { CStr::from_ptr(fmt) },
            &mut tmval,
        )
        .is_null();
    result.write_number(match parsed {
        true => unsafe { mktime(&raw mut tmval) as VarNumber },
        false => -1,
    });
    if result.number_or_zero() == -1 {
        result.write_number(0);
    }
    if conv.vc_type != CONV_NONE {
        unsafe { xfree(fmt.cast::<c_void>()) };
    }
    let _ = unsafe { convert_setup(&raw mut conv, ptr::null_mut(), ptr::null_mut()) };
    unsafe { xfree(enc.cast::<c_void>()) };
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
        result.write_string(reg_submatch(no).map_or(ptr::null_mut(), XString::into_raw));
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
    // SAFETY throughout: the arguments are live typvals and the three scratch buffers
    // outlive the calls that fill them and the `do_string_sub` that reads
    // what they hold.
    let str = arg_string_chk(&mut numbuf, &args[0]);
    let pat = arg_string_chk(&mut patbuf, &args[1]);
    let flg = arg_string_chk(&mut flagsbuf, &args[3]);
    let mut sub: *const c_char = ptr::null();
    let mut expr: Option<&TypVal> = None;
    if args[2].is_func() {
        expr = Some(&args[2]);
    } else {
        sub = arg_string_chk(&mut subbuf, &args[2]);
    }
    result.write_string(
        if str.is_null() || pat.is_null() || (sub.is_null() && expr.is_none()) || flg.is_null() {
            ptr::null_mut()
        } else {
            let str = str as *mut c_char;
            let pat = pat as *mut c_char;
            let sub = sub as *mut c_char;
            let flg = flg as *mut c_char;
            let out = ptr::null_mut();
            // SAFETY: every string is NUL-terminated and outlives the call,
            // and `expr` is null or argument 2.
            let len = unsafe { cstr::bytes_at(str) }.len();
            unsafe { do_string_sub(str, len, pat, sub, expr, flg, out) }
        },
    );
}
