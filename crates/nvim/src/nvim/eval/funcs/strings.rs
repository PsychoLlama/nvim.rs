//! Building and reshaping strings: escaping, formatting, splitting,
//! substituting, hashing, time formatting and spelling.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::non_zero_arg;
use super::{
    CONV_NONE, FAIL, FORWARD, GA_EMPTY_INIT_VALUE, HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR,
    NSUBEXP, NUL, RE_MAGIC, RE_STRING, SMT_ALL, VAR_BLOB, VAR_LIST, VAR_STRING, VSE_NONE,
    kListLenMayKnow,
};
use crate::semsg;
use crate::src::nvim::cursor::get_cursor_pos_ptr;
use crate::src::nvim::eval::typval::{
    tv_blob_alloc_ret, tv_blob_get, tv_blob_set_range, tv_check_for_nonempty_string_arg,
    tv_check_for_string_arg, tv_check_num, tv_get_bool_chk, tv_get_number, tv_get_number_chk,
    tv_get_string, tv_get_string_buf, tv_get_string_buf_chk, tv_get_string_chk, tv_is_func,
    tv_list_alloc_ret, tv_list_append_allocated_string, tv_list_append_string, tv_list_extend,
    tv_list_len,
};
use crate::src::nvim::eval_1::do_string_sub;
use crate::src::nvim::ex_getln::vim_strsave_fnameescape;
use crate::src::nvim::garray::{ga_clear, ga_grow};
use crate::src::nvim::keycodes::vim_strsave_escape_ks;
use crate::src::nvim::main::{
    curbuf, curwin, did_emsg, e_no_spell, empty_string_option, p_cpo, p_enc,
};
use crate::src::nvim::mbyte::{
    convert_setup, enc_locale, string_convert, utf_char2bytes, utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memory::{xfree, xmalloc, xmallocz, xmemdupz, xstrdup};
use crate::src::nvim::message::{emsg, str2special_save};
use crate::src::nvim::os::libc::{gettext, memmove, mktime, strftime, strlen, time};
use crate::src::nvim::os::time::{os_localtime_r, os_strptime, tm_zeroed};
use crate::src::nvim::regexp::{
    reg_submatch, reg_submatch_list, vim_regcomp, vim_regexec_nl, vim_regfree,
};
use crate::src::nvim::sha256::hex_digest;
use crate::src::nvim::spell::{eval_soundfold, parse_spelllang, spell_check, spell_move_to};
use crate::src::nvim::spellsuggest::spell_suggest_list;
use crate::src::nvim::strings::{
    vim_strsave_escaped, vim_strsave_shellescape, vim_vsnprintf_typval,
};
use crate::src::nvim::types::{
    EvalFuncData, blob_T, colnr_T, garray_T, hlf_T, list_T, regmatch_T, regprog_T, time_t, tm,
    typval_T, varnumber_T, vimconv_T,
};
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

/// The scratch buffer `tv_get_string_buf` fills for a value that is not
/// already a String. `NUMBUFLEN` in the C.
type NumBuf = [c_char; 65];
const NUM_BUF: NumBuf = [0; 65];

/// A conversion descriptor that has not been set up yet.
const CONV_NONE_INIT: vimconv_T = vimconv_T {
    vc_type: CONV_NONE as c_int,
    vc_factor: 0,
    vc_fd: ptr::null_mut(),
    vc_fail: false,
};

/// `char2nr({string} [, {utf8}])` — the first character's code point. The
/// second argument only has to type-check; nvim is always UTF-8.
pub unsafe extern "C" fn f_char2nr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    unsafe {
        if args.has(1) && !tv_check_num(args.ptr(1)) {
            return;
        }
        rettv.vval.v_number = utf_ptr2char(tv_get_string(args.ptr(0))) as varnumber_T;
    }
}

/// `escape({string}, {chars})` — backslash every byte listed in `chars`.
pub unsafe extern "C" fn f_escape(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut buf: NumBuf = NUM_BUF;
    // SAFETY: the arguments are live typvals and `buf` outlives the call
    // `tv_get_string_buf` may fill it for.
    rettv.vval.v_string = unsafe {
        vim_strsave_escaped(
            tv_get_string(args.ptr(0)),
            tv_get_string_buf(args.ptr(1), buf.as_mut_ptr()),
        )
    };
    rettv.v_type = VAR_STRING;
}

/// `fnameescape({string})`.
pub unsafe extern "C" fn f_fnameescape(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `args.ptr(0)` is a live typval.
    rettv.vval.v_string =
        unsafe { vim_strsave_fnameescape(tv_get_string(args.ptr(0)), VSE_NONE as c_int) };
    rettv.v_type = VAR_STRING;
}

/// `gettext({string})` — a no-op while no message catalogs ship, but it
/// still requires a non-empty String.
pub unsafe extern "C" fn f_gettext(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `args.ptr(0)` is a live typval.
    if unsafe { tv_check_for_nonempty_string_arg(args.ptr(0), 0) } == FAIL {
        return;
    }
    rettv.v_type = VAR_STRING;
    // SAFETY: the check above proved argument 0 is a non-empty String, so
    // the union holds a live NUL-terminated pointer.
    rettv.vval.v_string = unsafe { xstrdup(gettext(args.get(0).vval.v_string)) };
}

/// `keytrans({string})` — the readable spelling of a key sequence.
pub unsafe extern "C" fn f_keytrans(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: `args.ptr(0)` is a live typval; after the check the union
    // holds a String pointer, which may still be null.
    unsafe {
        if tv_check_for_string_arg(args.ptr(0), 0) == FAIL || args.get(0).vval.v_string.is_null() {
            return;
        }
        let escaped = vim_strsave_escape_ks(args.get(0).vval.v_string);
        rettv.vval.v_string = str2special_save(escaped, true, true);
        xfree(escaped.cast::<c_void>());
    }
}

/// `nr2char({number} [, {utf8}])`.
pub unsafe extern "C" fn f_nr2char(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut error = false;
    // SAFETY: the arguments are live typvals.
    let num = unsafe {
        if args.has(1) && !tv_check_num(args.ptr(1)) {
            return;
        }
        tv_get_number_chk(args.ptr(0), &raw mut error)
    };
    if error {
        return;
    }
    if num < 0 {
        // SAFETY: a literal message.
        unsafe {
            emsg(gettext(
                c"E5070: Character number must not be less than zero".as_ptr(),
            ))
        };
        return;
    }
    if num > c_int::MAX as varnumber_T {
        semsg!(
            "E5071: Character number must not be greater than INT_MAX ({})",
            c_int::MAX
        );
        return;
    }
    let mut buf: [c_char; 6] = [0; 6];
    // SAFETY: `buf` has room for the longest UTF-8 sequence
    // `utf_char2bytes` writes, and the returned length is what it wrote.
    rettv.vval.v_string = unsafe {
        let len = utf_char2bytes(num as c_int, buf.as_mut_ptr());
        xmemdupz(buf.as_ptr().cast::<c_void>(), len as usize).cast::<c_char>()
    };
    rettv.v_type = VAR_STRING;
}

/// `printf({fmt}, ...)` — measure, then format into an exact allocation.
///
/// The `did_emsg` dance is load-bearing: the measuring pass is where a bad
/// format reports, and the formatting pass is skipped when it did. The
/// caller's own `did_emsg` is restored by OR-ing it back, so an error
/// raised before this call is not lost and one raised inside it is.
pub unsafe extern "C" fn f_printf(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();

    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);
    let mut buf: NumBuf = NUM_BUF;
    // SAFETY: `buf` outlives both passes, and `args.ptr(1)` is the start of
    // the remaining arguments — `vim_vsnprintf_typval` stops at the
    // `VAR_UNKNOWN` terminator. The `dummy_ap` va_list is never read
    // because the typval overload is selected by the non-null argument
    // list, which is how every caller of this entry point uses it.
    unsafe {
        let fmt = tv_get_string_buf(args.ptr(0), buf.as_mut_ptr());
        let len = vim_vsnprintf_typval(ptr::null_mut(), 0, fmt, dummy_ap(), args.ptr(1));
        if did_emsg.get() == 0 {
            let s = xmalloc(len as usize + 1).cast::<c_char>();
            rettv.vval.v_string = s;
            vim_vsnprintf_typval(s, len as usize + 1, fmt, dummy_ap(), args.ptr(1));
        }
    }
    did_emsg.set(did_emsg.get() | saved_did_emsg);
}

/// `repeat({expr}, {count})` — for a List, a Blob or a String.
pub unsafe extern "C" fn f_repeat(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    let n = unsafe { tv_get_number(args.ptr(1)) };
    match args.ty(0) {
        VAR_LIST => unsafe { repeat_list(args, rettv, n) },
        VAR_BLOB => unsafe { repeat_blob(args, rettv, n) },
        _ => unsafe { repeat_string(args, rettv, n) },
    }
}

/// # Safety
/// Argument 0 is a live List typval and `rettv` is the cleared return value.
unsafe fn repeat_list(args: Args<'_>, rettv: &mut typval_T, n: varnumber_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        let src = args.get(0).vval.v_list;
        // The length hint is upstream's; a non-positive count contributes
        // nothing rather than a negative capacity.
        let hint = varnumber_T::from(n > 0) * n * varnumber_T::from(tv_list_len(src));
        let out = tv_list_alloc_ret(rettv, hint as isize);
        for _ in 0..n.max(0) {
            tv_list_extend(out, src, ptr::null_mut());
        }
    }
}

/// # Safety
/// Argument 0 is a live Blob typval and `rettv` is the cleared return value.
unsafe fn repeat_blob(args: Args<'_>, rettv: &mut typval_T, n: varnumber_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        tv_blob_alloc_ret(rettv);
        let src: *mut blob_T = args.get(0).vval.v_blob;
        if src.is_null() || n <= 0 {
            return;
        }
        let slen = (*src).bv_ga.ga_len;
        // Upstream computes the total in `int`; a product that does not fit
        // reads as non-positive and the repeat is dropped.
        let len = (slen as varnumber_T * n) as c_int;
        if len <= 0 {
            return;
        }
        let out = rettv.vval.v_blob;
        ga_grow(&raw mut (*out).bv_ga, len);
        (*out).bv_ga.ga_len = len;
        // An all-zero source needs no copying: `ga_grow` already zeroed the
        // destination. This is upstream's shortcut, not an optimisation
        // added here.
        if (0..slen).all(|i| tv_blob_get(src, i) == 0) {
            return;
        }
        for i in 0..len / slen {
            tv_blob_set_range(
                out,
                (i * slen) as varnumber_T,
                ((i + 1) * slen - 1) as varnumber_T,
                args.ptr(0),
            );
        }
    }
}

/// # Safety
/// Argument 0 is a live typval and `rettv` is the cleared return value.
unsafe fn repeat_string(args: Args<'_>, rettv: &mut typval_T, n: varnumber_T) {
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    if n <= 0 {
        return;
    }
    // SAFETY: the caller's obligation; `p` is NUL-terminated and outlives
    // the copies made from it.
    unsafe {
        let p = tv_get_string(args.ptr(0));
        let slen = strlen(p);
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
        let r = xmallocz(len).cast::<c_char>();
        for i in 0..n as usize {
            memmove(r.add(i * slen).cast::<c_void>(), p.cast::<c_void>(), slen);
        }
        rettv.vval.v_string = r;
    }
}

/// `sha256({string})` — also accepts a Blob, whose bytes are hashed as they
/// are rather than up to the first NUL.
pub unsafe extern "C" fn f_sha256(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: `args.ptr(0)` is a live typval. For the Blob branch the tag
    // proves the union holds a blob pointer, which may be null (an empty
    // literal) or hold a null buffer; both read as no bytes. For the String
    // branch `tv_get_string` hands back a NUL-terminated buffer.
    let hash = unsafe {
        if args.ty(0) == VAR_BLOB {
            let blob = args.get(0).vval.v_blob;
            let bytes = match blob.is_null() || (*blob).bv_ga.ga_data.is_null() {
                true => &[][..],
                false => core::slice::from_raw_parts(
                    (*blob).bv_ga.ga_data.cast::<u8>(),
                    (*blob).bv_ga.ga_len as usize,
                ),
            };
            hex_digest(bytes)
        } else {
            let p = tv_get_string(args.ptr(0));
            hex_digest(core::slice::from_raw_parts(p.cast::<u8>(), strlen(p)))
        }
    };
    // SAFETY: `hash` is a live buffer of `hash.len()` bytes.
    rettv.vval.v_string =
        unsafe { xmemdupz(hash.as_ptr().cast::<c_void>(), hash.len()).cast::<c_char>() };
}

/// `shellescape({string} [, {special}])`.
pub unsafe extern "C" fn f_shellescape(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    unsafe {
        let do_special = non_zero_arg(args.ptr(1));
        rettv.vval.v_string =
            vim_strsave_shellescape(tv_get_string(args.ptr(0)), do_special, do_special);
    }
    rettv.v_type = VAR_STRING;
}

/// `soundfold({word})`.
pub unsafe extern "C" fn f_soundfold(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: `args.ptr(0)` is a live typval.
    rettv.vval.v_string = unsafe { eval_soundfold(tv_get_string(args.ptr(0))) };
}

/// Turn 'spell' on for the duration of `body`, loading the spell languages
/// if they are not loaded yet, and report E756 if none is configured.
///
/// Both spelling builtins open this way, and both must put the window's own
/// 'spell' back on every path out — including the error one.
///
/// # Safety
/// Runs with `curwin` live, which is every builtin's situation.
unsafe fn with_spell(body: impl FnOnce()) {
    // SAFETY: the caller's obligation.
    unsafe {
        let win = curwin.get();
        let saved = (*win).w_onebuf_opt.wo_spell;
        if (*win).w_onebuf_opt.wo_spell == 0 {
            parse_spelllang(win);
            (*win).w_onebuf_opt.wo_spell = 1;
        }
        if *(*(*win).w_s).b_p_spl == NUL as c_char {
            emsg(gettext(e_no_spell.ptr() as *const c_char));
        } else {
            body();
        }
        (*win).w_onebuf_opt.wo_spell = saved;
    }
}

/// `spellbadword([{sentence}])` — the first misspelling and why it is one.
pub unsafe extern "C" fn f_spellbadword(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut word: *const c_char = c"".as_ptr();
    let mut attr: hlf_T = HLF_COUNT;
    let mut len: usize = 0;
    let mut reported = false;
    // SAFETY: `curwin`/`curbuf` are live, and `args.ptr(0)` is a live
    // typval. `spell_check` advances `str` by the length it reports, which
    // never passes the terminator.
    unsafe {
        with_spell(|| {
            reported = true;
            if !args.has(0) {
                len = spell_move_to(curwin.get(), FORWARD, SMT_ALL, true, &raw mut attr);
                if len != 0 {
                    word = get_cursor_pos_ptr();
                    (*curwin.get()).w_set_curswant = 1;
                }
            } else if *(*curbuf.get()).b_s.b_p_spl != NUL as c_char {
                let mut str = tv_get_string_chk(args.ptr(0));
                let mut capcol: c_int = -1;
                if !str.is_null() {
                    while *str != NUL as c_char {
                        len = spell_check(
                            curwin.get(),
                            str as *mut c_char,
                            &raw mut attr,
                            &raw mut capcol,
                            false,
                        );
                        if attr != HLF_COUNT {
                            word = str;
                            break;
                        }
                        str = str.add(len);
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
        let list = tv_list_alloc_ret(rettv, 2);
        tv_list_append_string(list, word, len as isize);
        let reason: Option<&CStr> = match attr {
            HLF_SPB => Some(c"bad"),
            HLF_SPR => Some(c"rare"),
            HLF_SPL => Some(c"local"),
            HLF_SPC => Some(c"caps"),
            _ => None,
        };
        match reason {
            Some(r) => tv_list_append_string(list, r.as_ptr(), r.count_bytes() as isize),
            None => tv_list_append_string(list, ptr::null(), -1),
        }
    }
}

/// `spellsuggest({word} [, {max} [, {capital}]])`.
pub unsafe extern "C" fn f_spellsuggest(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut ga: garray_T = GA_EMPTY_INIT_VALUE;
    let mut reported = false;
    // SAFETY: `curwin` is live and the arguments are live typvals; `ga` is
    // a local garray that `spell_suggest_list` fills and `ga_clear` frees.
    unsafe {
        with_spell(|| {
            reported = true;
            let str = tv_get_string(args.ptr(0));
            let mut typeerr = false;
            let (maxcount, need_capital) = if !args.has(1) {
                (25, false)
            } else {
                let maxcount = tv_get_number_chk(args.ptr(1), &raw mut typeerr) as c_int;
                // A non-positive maximum leaves the list empty, and does so
                // before the type error from argument 2 could be reported.
                if maxcount <= 0 {
                    return;
                }
                let need_capital =
                    args.has(2) && tv_get_number_chk(args.ptr(2), &raw mut typeerr) != 0;
                if typeerr {
                    return;
                }
                (maxcount, need_capital)
            };
            spell_suggest_list(
                &raw mut ga,
                str as *mut c_char,
                maxcount,
                need_capital,
                false,
            );
        });
        if !reported {
            return;
        }
        let list = tv_list_alloc_ret(rettv, ga.ga_len as isize);
        for i in 0..ga.ga_len {
            tv_list_append_allocated_string(
                list,
                *ga.ga_data.cast::<*mut c_char>().offset(i as isize),
            );
        }
        ga_clear(&raw mut ga);
    }
}

/// `split({string} [, {pattern} [, {keepempty}]])`.
pub unsafe extern "C" fn f_split(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut patbuf: NumBuf = NUM_BUF;
    // 'cpoptions' is cleared around the split so that its flags cannot
    // change what the pattern means.
    let save_cpo = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut c_char);
    // SAFETY: the arguments are live typvals, `patbuf` outlives the calls
    // that may fill it, and the compiled program is freed before returning.
    unsafe {
        let str = tv_get_string(args.ptr(0));
        let mut typeerr = false;
        let mut keepempty = false;
        let mut pat: *const c_char = ptr::null();
        if args.has(1) {
            pat = tv_get_string_buf_chk(args.ptr(1), patbuf.as_mut_ptr());
            if pat.is_null() {
                typeerr = true;
            }
            if args.has(2) {
                keepempty = tv_get_bool_chk(args.ptr(2), &raw mut typeerr) != 0;
            }
        }
        // An absent or empty pattern splits on runs of whitespace.
        if pat.is_null() || *pat == NUL as c_char {
            pat = c"[\\x01- ]\\+".as_ptr();
        }
        let list = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        if !typeerr {
            let prog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            if !prog.is_null() {
                split_into(list, str, prog, keepempty);
                vim_regfree(prog);
            }
        }
    }
    p_cpo.set(save_cpo);
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
unsafe fn split_into(
    list: *mut list_T,
    mut str: *const c_char,
    prog: *mut regprog_T,
    keepempty: bool,
) {
    // SAFETY: the caller's obligation. The match positions come back
    // pointing into `str`, so every pointer difference below is within one
    // allocation.
    unsafe {
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: prog,
            startp: [ptr::null_mut(); 10],
            endp: [ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut col: colnr_T = 0;
        while *str != NUL as c_char || keepempty {
            let matched = *str != NUL as c_char && vim_regexec_nl(&raw mut regmatch, str, col);
            let end: *const c_char = if matched {
                regmatch.startp[0]
            } else {
                str.add(strlen(str))
            };
            if keepempty
                || end > str
                || (tv_list_len(list) > 0
                    && *str != NUL as c_char
                    && matched
                    && end < regmatch.endp[0] as *const c_char)
            {
                tv_list_append_string(list, str, end.offset_from(str) as isize);
            }
            if !matched {
                break;
            }
            // An empty match would not advance, so the next attempt starts
            // one character further in while `str` stays put.
            col = if regmatch.endp[0] > str as *mut c_char {
                0
            } else {
                utfc_ptr2len(regmatch.endp[0]) as colnr_T
            };
            str = regmatch.endp[0];
        }
    }
}

/// `strftime({format} [, {time}])`.
pub unsafe extern "C" fn f_strftime(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: the arguments are live typvals; the two conversion
    // descriptors are opened and closed here, and `enc` is freed on every
    // path out.
    unsafe {
        let mut p = tv_get_string(args.ptr(0)) as *mut c_char;
        let seconds: time_t = if args.has(1) {
            tv_get_number(args.ptr(1)) as time_t
        } else {
            time(ptr::null_mut())
        };
        let mut curtime: tm = tm_zeroed();
        if !os_localtime_r(seconds, &mut curtime) {
            rettv.vval.v_string = xstrdup(gettext(c"(Invalid)".as_ptr()));
            return;
        }
        let mut conv: vimconv_T = CONV_NONE_INIT;
        let enc = enc_locale();
        convert_setup(&raw mut conv, p_enc.get(), enc);
        if conv.vc_type != CONV_NONE as c_int {
            p = string_convert(&raw mut conv, p, ptr::null_mut());
        }
        let mut out: [c_char; 256] = [0; 256];
        if p.is_null() || strftime(out.as_mut_ptr(), out.len(), p, &raw mut curtime) == 0 {
            out[0] = NUL as c_char;
        }
        if conv.vc_type != CONV_NONE as c_int {
            xfree(p.cast::<c_void>());
        }
        // The reverse conversion reuses `conv`, so it must be set up again
        // in the other direction before the result is converted back.
        convert_setup(&raw mut conv, enc, p_enc.get());
        rettv.vval.v_string = if conv.vc_type != CONV_NONE as c_int {
            string_convert(&raw mut conv, out.as_mut_ptr(), ptr::null_mut())
        } else {
            xstrdup(out.as_mut_ptr())
        };
        convert_setup(&raw mut conv, ptr::null_mut(), ptr::null_mut());
        xfree(enc.cast::<c_void>());
    }
}

/// `strptime({format}, {timestring})`.
pub unsafe extern "C" fn f_strptime(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut fmt_buf: NumBuf = NUM_BUF;
    let mut str_buf: NumBuf = NUM_BUF;
    // strptime() is asked to determine DST itself.
    let mut tmval: tm = tm {
        tm_isdst: -1,
        ..tm_zeroed()
    };
    // SAFETY: the arguments are live typvals, the two scratch buffers
    // outlive the calls that may fill them, and `enc` and the converted
    // format are freed on every path out.
    unsafe {
        let mut fmt = tv_get_string_buf(args.ptr(0), fmt_buf.as_mut_ptr()) as *mut c_char;
        let str = tv_get_string_buf(args.ptr(1), str_buf.as_mut_ptr()) as *mut c_char;
        let mut conv: vimconv_T = CONV_NONE_INIT;
        let enc = enc_locale();
        convert_setup(&raw mut conv, p_enc.get(), enc);
        if conv.vc_type != CONV_NONE as c_int {
            fmt = string_convert(&raw mut conv, fmt, ptr::null_mut());
        }
        // `mktime` reporting -1 is indistinguishable from a genuine
        // timestamp of -1, and upstream treats both as failure.
        let parsed = !fmt.is_null()
            && !os_strptime(CStr::from_ptr(str), CStr::from_ptr(fmt), &mut tmval).is_null();
        rettv.vval.v_number = match parsed {
            true => mktime(&raw mut tmval) as varnumber_T,
            false => -1,
        };
        if rettv.vval.v_number == -1 {
            rettv.vval.v_number = 0;
        }
        if conv.vc_type != CONV_NONE as c_int {
            xfree(fmt.cast::<c_void>());
        }
        convert_setup(&raw mut conv, ptr::null_mut(), ptr::null_mut());
        xfree(enc.cast::<c_void>());
    }
}

/// `submatch({nr} [, {list}])`.
pub unsafe extern "C" fn f_submatch(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut error = false;
    // SAFETY: the arguments are live typvals.
    let no = unsafe { tv_get_number_chk(args.ptr(0), &raw mut error) } as c_int;
    if error {
        return;
    }
    if !(0..NSUBEXP as c_int).contains(&no) {
        semsg!("E935: Invalid submatch number: {no}");
        return;
    }
    let as_list = if args.has(1) {
        // SAFETY: `args.ptr(1)` is a live typval.
        let flag = unsafe { tv_get_number_chk(args.ptr(1), &raw mut error) };
        if error {
            return;
        }
        flag != 0
    } else {
        false
    };
    // SAFETY: both readers answer for the match state the caller is in,
    // returning null outside a substitution.
    unsafe {
        if as_list {
            rettv.v_type = VAR_LIST;
            rettv.vval.v_list = reg_submatch_list(no);
        } else {
            rettv.v_type = VAR_STRING;
            rettv.vval.v_string = reg_submatch(no);
        }
    }
}

/// `substitute({string}, {pat}, {sub}, {flags})` — `{sub}` may be a Funcref,
/// in which case it is handed on rather than read as a String.
pub unsafe extern "C" fn f_substitute(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut patbuf: NumBuf = NUM_BUF;
    let mut subbuf: NumBuf = NUM_BUF;
    let mut flagsbuf: NumBuf = NUM_BUF;
    rettv.v_type = VAR_STRING;
    // SAFETY: the arguments are live typvals and the three scratch buffers
    // outlive the calls that fill them and the `do_string_sub` that reads
    // what they hold.
    unsafe {
        let str = tv_get_string_chk(args.ptr(0));
        let pat = tv_get_string_buf_chk(args.ptr(1), patbuf.as_mut_ptr());
        let flg = tv_get_string_buf_chk(args.ptr(3), flagsbuf.as_mut_ptr());
        let mut sub: *const c_char = ptr::null();
        let mut expr: *mut typval_T = ptr::null_mut();
        if tv_is_func(*args.get(2)) {
            expr = args.ptr(2);
        } else {
            sub = tv_get_string_buf_chk(args.ptr(2), subbuf.as_mut_ptr());
        }
        rettv.vval.v_string =
            if str.is_null() || pat.is_null() || (sub.is_null() && expr.is_null()) || flg.is_null()
            {
                ptr::null_mut()
            } else {
                do_string_sub(
                    str as *mut c_char,
                    strlen(str),
                    pat as *mut c_char,
                    sub as *mut c_char,
                    expr,
                    flg as *mut c_char,
                    ptr::null_mut(),
                )
            };
    }
}
