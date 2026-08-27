//! Operands that are written out: numbers, the three string forms,
//! `&option` and `$ENV`.
//!
//! The two quoted forms are each parsed twice — once to find the closing
//! quote and size the result, once to fill it — and the two passes must
//! agree byte for byte. What keeps them in step is a small correction the
//! measuring pass accumulates: `extra` in `eval_string`, `reduce` in
//! `eval_lit_string`. Both count how much longer or shorter the result is
//! than the source text it was read from.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{hex2nr, skipdigits, vim_str2nr};
use crate::eval::typval::{tv_blob_alloc, tv_blob_set_ret, tv_clear};
use crate::eval::vars::{eval_one_expr_in_str, optval_as_tv};
use crate::eval::{
    BS, CAR, ESC, FF, FSK_IN_STRING, FSK_KEYCODE, FSK_SIMPLIFY, NL, STR2NR_ALL, TAB,
    find_option_var_end, get_env_len, kOptValTypeNil,
};
use crate::garray::{ga_append, ga_clear, ga_concat, ga_init};
use crate::keycodes::{find_special_key, trans_special};
use crate::main::{e_invexpr2, e_stray_closing_curly_str};
use crate::mbyte::{mb_copy_char, utf_char2bytes, utfc_ptr2len};
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg, iemsg};
use crate::option::{get_option_value, get_tty_option, is_option_hidden, is_tty_option};
use crate::options::{kOptAleph, kOptInvalid};
use crate::os::cshim::{gettext, strncasecmp};
use crate::os::env::{expand_env_save, vim_getenv};
use crate::types::{
    FAIL, NUL, OK, OptIndex, OptVal, OptionSetFlags, VAR_FLOAT, VAR_NUMBER, VAR_STRING,
    VAR_UNKNOWN, VarLock, blob_T, float_T, garray_T, size_t, typval_T, typval_vval_union, uint8_t,
    varnumber_T,
};
use ::libc::{strlen, strtod, toupper};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// An empty growable array of bytes.
const UNSET_GA: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: null_mut(),
};

/// `&option`, `&l:option`, `&g:option` or `+option`, with the cursor on the
/// `&` or the `+`. Leaves it after the option name.
///
/// A null `rettv` means "only say whether this names an option"; that is
/// `has("+option")`, which is also the only caller `working` is true for.
///
/// # Safety
/// `arg` must point at the cursor into a writable, NUL-terminated
/// expression; `rettv` must be null or valid.
pub(crate) unsafe fn eval_option(
    arg: *mut *const c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    unsafe {
        let working = **arg == b'+' as c_char; // has("+option")
        let mut opt_idx: OptIndex = kOptAleph;
        let mut opt_flags: OptionSetFlags = OptionSetFlags::NONE;

        // Isolate the option name and find its value.
        let option_end =
            find_option_var_end(arg, &raw mut opt_idx, &raw mut opt_flags) as *mut c_char;
        if option_end.is_null() {
            if !rettv.is_null() {
                semsg_c!(gettext(c"E112: Option name missing: %s".as_ptr()), *arg);
            }
            return FAIL;
        }
        if !evaluate {
            *arg = option_end;
            return OK;
        }

        // The name is terminated in place for the lookup and put back
        // afterwards, because the error messages want the whole expression.
        let c = *option_end;
        *option_end = NUL as c_char;

        let opt_name = CStr::from_ptr(*arg);
        let is_tty_opt = is_tty_option(opt_name);
        let ret = if opt_idx == kOptInvalid && !is_tty_opt {
            // Only report it when the result is going to be used.
            if !rettv.is_null() {
                semsg_c!(gettext(c"E113: Unknown option: %s".as_ptr()), *arg);
            }
            FAIL
        } else if !rettv.is_null() {
            let value: OptVal = if is_tty_opt {
                get_tty_option(opt_name)
            } else {
                get_option_value(opt_idx, opt_flags)
            };
            debug_assert!(value.type_0 != kOptValTypeNil);
            *rettv = optval_as_tv(value, true);
            OK
        } else if working && !is_tty_opt && is_option_hidden(opt_idx) {
            FAIL
        } else {
            OK
        };

        *option_end = c;
        *arg = option_end;
        ret
    }
}

/// A Number, a Float or a `0z` Blob literal, with the cursor on the first
/// digit. `want_string` suppresses the Float reading, so that `1.2` in a
/// context that wants a string is the Number 1 followed by `.2`.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `rettv` must be valid when `evaluate`.
pub(crate) unsafe fn eval_number(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
    want_string: bool,
) -> c_int {
    unsafe {
        let mut p = skipdigits((*arg).add(1));

        // A Float is accepted only for the exact `1.2`, `1.2e3` shapes: a
        // digit either side of the dot, and nothing alphabetic or a second
        // dot after what was read.
        let mut get_float = false;
        if !want_string && *p == b'.' as c_char && ascii_isdigit(*p.add(1) as c_int) {
            get_float = true;
            p = skipdigits(p.add(2));
            if *p == b'e' as c_char || *p == b'E' as c_char {
                p = p.add(1);
                if *p == b'-' as c_char || *p == b'+' as c_char {
                    p = p.add(1);
                }
                if !ascii_isdigit(*p as c_int) {
                    get_float = false;
                } else {
                    p = skipdigits(p.add(1));
                }
            }
            let after = *p as u8;
            if after.is_ascii_alphabetic() || after == b'.' {
                get_float = false;
            }
        }

        if get_float {
            let mut f: float_T = 0.;
            *arg = (*arg).add(string2float(*arg, &raw mut f) as usize);
            if evaluate {
                (*rettv).v_type = VAR_FLOAT;
                (*rettv).vval.v_float = f;
            }
        } else if **arg == b'0' as c_char
            && (*(*arg).add(1) == b'z' as c_char || *(*arg).add(1) == b'Z' as c_char)
        {
            let mut blob: *mut blob_T = if evaluate {
                tv_blob_alloc()
            } else {
                null_mut()
            };
            let mut bp = (*arg).add(2);
            while ascii_isxdigit(*bp as c_int) {
                if !ascii_isxdigit(*bp.add(1) as c_int) {
                    if !blob.is_null() {
                        emsg(gettext(
                            c"E973: Blob literal should have an even number of hex characters"
                                .as_ptr(),
                        ));
                        ga_clear(&raw mut (*blob).bv_ga);
                        xfree(blob.cast());
                    }
                    return FAIL;
                }
                if !blob.is_null() {
                    ga_append(
                        &raw mut (*blob).bv_ga,
                        ((hex2nr(*bp as c_int) << 4) + hex2nr(*bp.add(1) as c_int)) as uint8_t,
                    );
                }
                // A dot may separate byte pairs: `0z00.11.22`.
                if *bp.add(2) == b'.' as c_char && ascii_isxdigit(*bp.add(3) as c_int) {
                    bp = bp.add(1);
                }
                bp = bp.add(2);
            }
            if !blob.is_null() {
                tv_blob_set_ret(rettv, blob);
            }
            *arg = bp;
        } else {
            let mut len: c_int = 0;
            let mut n: varnumber_T = 0;
            vim_str2nr(
                *arg,
                null_mut(),
                &raw mut len,
                STR2NR_ALL as c_int,
                &raw mut n,
                null_mut(),
                0,
                true,
                null_mut(),
            );
            if len == 0 {
                if evaluate {
                    semsg_c!(gettext(e_invexpr2.as_ptr()), *arg);
                }
                return FAIL;
            }
            *arg = (*arg).offset(len as isize);
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = n;
            }
        }
        OK
    }
}

/// A double-quoted string, with the cursor on the quote — or, when
/// `interpolate` is set, on the first character of a `$"..."` piece, which
/// ends at the closing quote or at a single `{`.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `rettv` must be valid when `evaluate`.
pub(crate) unsafe fn eval_string(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    unsafe {
        let arg_end = (*arg).add(strlen(*arg) as usize) as *const c_char;
        let off = if interpolate { 0 } else { 1 };
        // How much longer the result is than the text it is read from. The
        // 1 an interpolated piece starts with is the terminator it writes;
        // a doubled brace gives a byte back. It is `unsigned` upstream and
        // may go negative here for the same reason it wraps there — the
        // sum with the source length is what is used, and stays positive.
        let mut extra: isize = if interpolate { 1 } else { 0 };

        // Find the end of the string, skipping backslashed characters.
        let mut p = (*arg).add(off);
        while *p as c_int != NUL && *p != b'"' as c_char {
            if *p == b'\\' as c_char && *p.add(1) as c_int != NUL {
                p = p.add(1);
                if *p == b'<' as c_char {
                    // A `\<x>` form is at least 4 characters and produces up
                    // to 9 (6 for the character, 3 for a modifier): reserve
                    // five extra.
                    extra += 5;
                    let mut modifiers: c_int = 0;
                    let mut flags = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                    if *p.add(1) != b'*' as c_char {
                        flags |= FSK_SIMPLIFY as c_int;
                    }
                    // Skip to the `>` so a `{` inside is not read as the
                    // start of an interpolated expression.
                    if find_special_key(
                        &raw mut p as *mut *const c_char,
                        arg_end.offset_from(p) as size_t,
                        &raw mut modifiers,
                        flags,
                        null_mut(),
                    ) != 0
                    {
                        p = p.sub(1); // leave `p` on the `>`
                    }
                }
            } else if interpolate && (*p == b'{' as c_char || *p == b'}' as c_char) {
                if *p == b'{' as c_char && *p.add(1) != b'{' as c_char {
                    break; // start of an expression
                }
                p = p.add(1);
                if *p.sub(1) == b'}' as c_char && *p != b'}' as c_char {
                    semsg_c!(gettext(e_stray_closing_curly_str.as_ptr()), *arg);
                    return FAIL;
                }
                extra -= 1; // `{{` becomes `{`, `}}` becomes `}`
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }

        if *p != b'"' as c_char && !(interpolate && *p == b'{' as c_char) {
            semsg_c!(gettext(c"E114: Missing quote: %s".as_ptr()), *arg);
            return FAIL;
        }
        if !evaluate {
            *arg = p.add(off);
            return OK;
        }

        // Copy the string into allocated memory, resolving the escapes.
        (*rettv).v_type = VAR_STRING;
        let len = (p.offset_from(*arg) + extra) as c_int;
        (*rettv).vval.v_string = xmalloc(len as size_t) as *mut c_char;
        let mut end = (*rettv).vval.v_string;

        p = (*arg).add(off);
        while *p as c_int != NUL && *p != b'"' as c_char {
            if *p != b'\\' as c_char {
                if interpolate && (*p == b'{' as c_char || *p == b'}' as c_char) {
                    if *p == b'{' as c_char && *p.add(1) != b'{' as c_char {
                        break; // start of an expression
                    }
                    p = p.add(1); // reduce `{{` to `{` and `}}` to `}`
                }
                mb_copy_char(&raw mut p as *mut *const c_char, &raw mut end);
                continue;
            }

            p = p.add(1);
            // Every arm that handles the escape itself leaves `handled` set;
            // the rest — including `\<` that did not name a key — fall
            // through to copying the character after the backslash.
            let mut handled = true;
            match *p as u8 {
                b'b' => {
                    *end = BS as c_char;
                    end = end.add(1);
                    p = p.add(1);
                }
                b'e' => {
                    *end = ESC as c_char;
                    end = end.add(1);
                    p = p.add(1);
                }
                b'f' => {
                    *end = FF as c_char;
                    end = end.add(1);
                    p = p.add(1);
                }
                b'n' => {
                    *end = NL as c_char;
                    end = end.add(1);
                    p = p.add(1);
                }
                b'r' => {
                    *end = CAR as c_char;
                    end = end.add(1);
                    p = p.add(1);
                }
                b't' => {
                    *end = TAB as c_char;
                    end = end.add(1);
                    p = p.add(1);
                }
                // hex `\x1`/`\x12`, Unicode `\u0023`/`\U0001f600`. With no
                // hex digit after it the letter itself is copied, by the
                // next pass of the loop rather than here.
                b'X' | b'x' | b'u' | b'U' => {
                    if ascii_isxdigit(*p.add(1) as c_int) {
                        let c = toupper(*p as u8 as c_int);
                        let mut n = if c == 'X' as c_int {
                            2
                        } else if *p == b'u' as c_char {
                            4
                        } else {
                            8
                        };
                        let mut nr: c_int = 0;
                        loop {
                            n -= 1;
                            if n < 0 || !ascii_isxdigit(*p.add(1) as c_int) {
                                break;
                            }
                            p = p.add(1);
                            nr = (nr << 4) + hex2nr(*p as c_int);
                        }
                        p = p.add(1);
                        // `\u` stores the character in the current encoding;
                        // `\x` stores the byte.
                        if c != 'X' as c_int {
                            end = end.offset(utf_char2bytes(nr, end) as isize);
                        } else {
                            *end = nr as c_char;
                            end = end.add(1);
                        }
                    }
                }
                // octal `\1`, `\12`, `\123`
                b'0'..=b'7' => {
                    *end = (*p as c_int - '0' as c_int) as c_char;
                    p = p.add(1);
                    if *p >= b'0' as c_char && *p <= b'7' as c_char {
                        *end = (((*end as c_int) << 3) + *p as c_int - '0' as c_int) as c_char;
                        p = p.add(1);
                        if *p >= b'0' as c_char && *p <= b'7' as c_char {
                            *end = (((*end as c_int) << 3) + *p as c_int - '0' as c_int) as c_char;
                            p = p.add(1);
                        }
                    }
                    end = end.add(1);
                }
                // a special key, e.g. `\<C-W>`
                b'<' => {
                    let mut flags = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                    if *p.add(1) != b'*' as c_char {
                        flags |= FSK_SIMPLIFY as c_int;
                    }
                    let written = trans_special(
                        &raw mut p as *mut *const c_char,
                        arg_end.offset_from(p) as size_t,
                        end,
                        flags,
                        false,
                        null_mut(),
                    );
                    if written != 0 {
                        end = end.offset(written as isize);
                        if end >= (*rettv).vval.v_string.offset(len as isize) {
                            iemsg(c"eval_string() used more space than allocated".as_ptr());
                        }
                    } else {
                        handled = false;
                    }
                }
                _ => handled = false,
            }
            if !handled {
                mb_copy_char(&raw mut p as *mut *const c_char, &raw mut end);
            }
        }

        *end = NUL as c_char;
        if *p == b'"' as c_char && !interpolate {
            p = p.add(1);
        }
        *arg = p;
        OK
    }
}

/// A single-quoted string, in which the only escape is a doubled quote —
/// or, when `interpolate` is set, a `$'...'` piece, which also reduces a
/// doubled brace and stops at a single `{`.
///
/// # Safety
/// As `eval_string`.
pub(crate) unsafe fn eval_lit_string(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    unsafe {
        let off = if interpolate { 0 } else { 1 };
        // How much *shorter* the result is than the text: one byte per
        // doubled quote or brace, less the terminator an interpolated piece
        // writes. The sign is the opposite of `eval_string`'s `extra`.
        let mut reduce: c_int = if interpolate { -1 } else { 0 };

        // Find the end of the string, skipping `''`.
        let mut p = (*arg).add(off);
        while *p as c_int != NUL {
            if *p == b'\'' as c_char {
                if *p.add(1) != b'\'' as c_char {
                    break;
                }
                reduce += 1;
                p = p.add(1);
            } else if interpolate {
                if *p == b'{' as c_char {
                    if *p.add(1) != b'{' as c_char {
                        break; // start of an expression
                    }
                    p = p.add(1);
                    reduce += 1;
                } else if *p == b'}' as c_char {
                    p = p.add(1);
                    if *p != b'}' as c_char {
                        semsg_c!(gettext(e_stray_closing_curly_str.as_ptr()), *arg);
                        return FAIL;
                    }
                    reduce += 1;
                }
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }

        if *p != b'\'' as c_char && !(interpolate && *p == b'{' as c_char) {
            semsg_c!(gettext(c"E115: Missing quote: %s".as_ptr()), *arg);
            return FAIL;
        }
        if !evaluate {
            *arg = p.add(off);
            return OK;
        }

        let mut str = xmalloc((p.offset_from(*arg) - reduce as isize) as size_t) as *mut c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = str;
        p = (*arg).add(off);
        while *p as c_int != NUL {
            if *p == b'\'' as c_char {
                if *p.add(1) != b'\'' as c_char {
                    break;
                }
                p = p.add(1);
            } else if interpolate && (*p == b'{' as c_char || *p == b'}' as c_char) {
                if *p == b'{' as c_char && *p.add(1) != b'{' as c_char {
                    break; // start of an expression
                }
                p = p.add(1);
            }
            mb_copy_char(&raw mut p as *mut *const c_char, &raw mut str);
        }
        *str = NUL as c_char;
        *arg = p.add(off);
        OK
    }
}

/// `$"..."` or `$'...'`, with the cursor on the `$`: alternating literal
/// pieces and `{expr}` substitutions, joined into one String.
///
/// Answers `OK` even for a piece that failed — upstream's; `rettv` then
/// holds whatever was assembled before the error, which may be null.
///
/// # Safety
/// As `eval_string`.
pub(crate) unsafe fn eval_interp_string(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    unsafe {
        let mut ret = OK;
        let mut ga = UNSET_GA;
        ga_init(&raw mut ga, 1, 80);

        // `*arg` is on the `$`; move it to the first string character.
        *arg = (*arg).add(1);
        let quote = **arg as u8;
        *arg = (*arg).add(1);

        loop {
            // The piece up to the matching quote or to a single `{`; `arg`
            // is left on whichever it was.
            let mut tv = UNSET_TV;
            ret = if quote == b'"' {
                eval_string(arg, &raw mut tv, evaluate, true)
            } else {
                eval_lit_string(arg, &raw mut tv, evaluate, true)
            };
            if ret == FAIL {
                break;
            }
            if evaluate {
                ga_concat(&raw mut ga, tv.vval.v_string);
                tv_clear(&raw mut tv);
            }
            if **arg != b'{' as c_char {
                // Found the terminating quote.
                *arg = (*arg).add(1);
                break;
            }
            let p = eval_one_expr_in_str(*arg, &raw mut ga, evaluate);
            if p.is_null() {
                ret = FAIL;
                break;
            }
            *arg = p;
        }

        (*rettv).v_type = VAR_STRING;
        if ret != FAIL && evaluate {
            ga_append(&raw mut ga, NUL as uint8_t);
        }
        (*rettv).vval.v_string = ga.ga_data as *mut c_char;
        OK
    }
}

/// Read a Float out of `text`, answering how many bytes it consumed. The
/// three named values are recognised ahead of `strtod`, which does not know
/// them in every locale.
///
/// # Safety
/// `text` must be NUL-terminated and `ret_value` valid.
pub(crate) unsafe fn string2float(text: *const c_char, ret_value: *mut float_T) -> size_t {
    unsafe {
        for (name, len, value) in [
            (c"inf", 3, f64::INFINITY),
            (c"-inf", 4, f64::NEG_INFINITY),
            (c"nan", 3, f64::NAN),
        ] {
            if strncasecmp(
                text as *mut c_char,
                name.as_ptr() as *mut c_char,
                len as size_t,
            ) == 0
            {
                *ret_value = value as float_T;
                return len as size_t;
            }
        }
        let mut s: *mut c_char = null_mut();
        *ret_value = strtod(text, &raw mut s) as float_T;
        s.offset_from(text) as size_t
    }
}

/// `$NAME`, with the cursor on the `$`.
///
/// # Safety
/// `arg` must point at the cursor into a writable, NUL-terminated
/// expression; `rettv` must be valid when `evaluate`.
pub(crate) unsafe fn eval_env_var(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    unsafe {
        *arg = (*arg).add(1);
        let name = *arg;
        let len = get_env_len(arg as *mut *const c_char);
        if !evaluate {
            return OK;
        }
        if len == 0 {
            return FAIL;
        }

        // The name is terminated in place across the two lookups.
        let cc = *name.offset(len as isize);
        *name.offset(len as isize) = NUL as c_char;
        let mut string = vim_getenv(name);
        if string.is_null() || *string as c_int == NUL {
            xfree(string as *mut c_void);
            // Not in the environment: let `expand_env` have it, which knows
            // the names nvim answers itself. A result that still starts with
            // `$` is the name coming back unexpanded.
            string = expand_env_save(name.sub(1));
            if !string.is_null() && *string == b'$' as c_char {
                xfree(string as *mut c_void);
                string = null_mut();
            }
        }
        *name.offset(len as isize) = cc;

        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = string;
        (*rettv).v_lock = VarLock::Unlocked;
        OK
    }
}
