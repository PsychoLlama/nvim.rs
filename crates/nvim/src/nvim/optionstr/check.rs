//! Vetting a string option's value, and the sweeps that re-vet every
//! buffer's copy of one.
//!
//! Two things live here that the rest of the tree leans on.
//!
//! The **empty string option**. A string option's variable is never null:
//! out of memory, `xstrdup` may hand back null, and everything downstream
//! dereferences the variable, so a null one is replaced by the shared
//! `empty_string_option`. That one allocation is aliased by every option
//! holding "", which is why freeing a string option has to go through
//! [`free_string_option`] rather than `xfree`.
//!
//! The **error buffer**. A check that has to name what it disliked formats
//! into a caller-supplied `errbuf`; the caller may decline one by passing
//! null, and upstream then answers with the shared empty string rather than
//! null, so the set still fails but reports nothing.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::transchar;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::main::{empty_string_option, secure};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::option::{kOptFlagNDname, kOptFlagNFname, valid_name};
use crate::src::nvim::options::{
    kOptBackupcopy, kOptBelloff, kOptCasemap, kOptClipboard, kOptCompleteopt, kOptDisplay,
    kOptFoldopen, kOptJumpoptions, kOptRedrawdebug, kOptSessionoptions, kOptSwitchbuf,
    kOptTabclose, kOptTagcase, kOptTermpastefilter, kOptViewoptions, kOptVirtualedit,
    kOptWildoptions, opt_scl_values,
};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{buf_T, size_t, uint32_t, win_T};

use super::{
    FAIL, NUL, OK, SCL_NO, check_str_opt, e_illegal_character_after_chr, e_unbalanced_groups,
    e_unclosed_expression_sequence, opt_strings_flags,
};
use crate::src::nvim::decoration::SCL_NUM;

/// The options whose bitmask is derived from a value the startup sequence
/// may have installed without going through `:set`.
pub unsafe extern "C" fn didset_string_options() {
    for idx in [
        kOptCasemap,
        kOptBackupcopy,
        kOptBelloff,
        kOptCompleteopt,
        kOptSessionoptions,
        kOptViewoptions,
        kOptFoldopen,
        kOptDisplay,
        kOptJumpoptions,
        kOptRedrawdebug,
        kOptTagcase,
        kOptTermpastefilter,
        kOptVirtualedit,
        kOptSwitchbuf,
        kOptTabclose,
        kOptWildoptions,
        kOptClipboard,
    ] {
        // SAFETY: a null `varp` asks for the option's own global variable.
        unsafe { check_str_opt(idx, ptr::null_mut()) };
    }
}

/// "E539: Illegal character <x>", formatted into the caller's buffer.
///
/// # Safety
/// `errbuf` is null or points at `errbuflen` writable bytes.
pub unsafe fn illegal_char(errbuf: *mut c_char, errbuflen: size_t, c: c_int) -> *mut c_char {
    if errbuf.is_null() {
        return c"".as_ptr().cast_mut();
    }
    // SAFETY: the caller's buffer, and `transchar` returns a C string.
    unsafe {
        vim_snprintf(
            errbuf,
            errbuflen,
            gettext(c"E539: Illegal character <%s>".as_ptr()),
            transchar(c),
        );
    }
    errbuf
}

/// "E535: Illegal character after <%c>", for the options that spell a field
/// as a character followed by a value.
///
/// # Safety
/// As [`illegal_char`].
pub(crate) unsafe fn illegal_char_after_chr(
    errbuf: *mut c_char,
    errbuflen: size_t,
    c: c_int,
) -> *mut c_char {
    if errbuf.is_null() {
        return c"".as_ptr().cast_mut();
    }
    // SAFETY: the caller's buffer; the format takes one `int`.
    unsafe {
        vim_snprintf(
            errbuf,
            errbuflen,
            gettext(e_illegal_character_after_chr.as_ptr()),
            c,
        );
    }
    errbuf
}

/// Give every string option of a buffer the empty string in place of a null.
///
/// # Safety
/// `buf` points at a live buffer.
pub unsafe extern "C" fn check_buf_options(buf: *mut buf_T) {
    // SAFETY: the caller's buffer; each field is one of its `char *`
    // options, and `parse_cino` re-derives the 'cinoptions' cache from the
    // string this just made non-null.
    unsafe {
        for field in [
            &raw mut (*buf).b_p_bh,
            &raw mut (*buf).b_p_bt,
            &raw mut (*buf).b_p_fenc,
            &raw mut (*buf).b_p_ff,
            &raw mut (*buf).b_p_def,
            &raw mut (*buf).b_p_inc,
            &raw mut (*buf).b_p_inex,
            &raw mut (*buf).b_p_inde,
            &raw mut (*buf).b_p_indk,
            &raw mut (*buf).b_p_fp,
            &raw mut (*buf).b_p_fex,
            &raw mut (*buf).b_p_kp,
            &raw mut (*buf).b_p_mps,
            &raw mut (*buf).b_p_fo,
            &raw mut (*buf).b_p_flp,
            &raw mut (*buf).b_p_isk,
            &raw mut (*buf).b_p_com,
            &raw mut (*buf).b_p_cms,
            &raw mut (*buf).b_p_nf,
            &raw mut (*buf).b_p_qe,
            &raw mut (*buf).b_p_syn,
            &raw mut (*buf).b_s.b_syn_isk,
            &raw mut (*buf).b_s.b_p_spc,
            &raw mut (*buf).b_s.b_p_spf,
            &raw mut (*buf).b_s.b_p_spl,
            &raw mut (*buf).b_s.b_p_spo,
            &raw mut (*buf).b_p_sua,
            &raw mut (*buf).b_p_cink,
            &raw mut (*buf).b_p_cino,
        ] {
            check_string_option(field);
        }
        parse_cino(buf);
        for field in [
            &raw mut (*buf).b_p_lop,
            &raw mut (*buf).b_p_ft,
            &raw mut (*buf).b_p_cinw,
            &raw mut (*buf).b_p_cinsd,
            &raw mut (*buf).b_p_cot,
            &raw mut (*buf).b_p_cpt,
            &raw mut (*buf).b_p_cfu,
            &raw mut (*buf).b_p_ofu,
            &raw mut (*buf).b_p_keymap,
            &raw mut (*buf).b_p_gefm,
            &raw mut (*buf).b_p_gp,
            &raw mut (*buf).b_p_mp,
            &raw mut (*buf).b_p_efm,
            &raw mut (*buf).b_p_ep,
            &raw mut (*buf).b_p_path,
            &raw mut (*buf).b_p_tags,
            &raw mut (*buf).b_p_ffu,
            &raw mut (*buf).b_p_tfu,
            &raw mut (*buf).b_p_tc,
            &raw mut (*buf).b_p_dict,
            &raw mut (*buf).b_p_dia,
            &raw mut (*buf).b_p_tsr,
            &raw mut (*buf).b_p_tsrfu,
            &raw mut (*buf).b_p_lw,
            &raw mut (*buf).b_p_bkc,
            &raw mut (*buf).b_p_menc,
            &raw mut (*buf).b_p_vsts,
            &raw mut (*buf).b_p_vts,
        ] {
            check_string_option(field);
        }
    }
}

/// Free a string option's value, unless it is the shared empty string every
/// option holding "" points at.
///
/// # Safety
/// `p` is null, the shared empty string, or an allocation this owns.
pub unsafe fn free_string_option(p: *mut c_char) {
    if p != empty_string_option.ptr().cast::<c_char>() {
        // SAFETY: as documented above.
        unsafe { xfree(p.cast::<c_void>()) };
    }
}

/// Free a string option's value and leave the variable holding the shared
/// empty string.
///
/// # Safety
/// `pp` points at a string option's variable.
pub unsafe fn clear_string_option(pp: *mut *mut c_char) {
    // SAFETY: the caller's variable, holding a value `free_string_option`
    // accepts.
    unsafe {
        free_string_option(*pp);
        *pp = empty_string_option.ptr().cast::<c_char>();
    }
}

/// Replace a null option value with the shared empty string, so that
/// everything downstream can dereference it.
///
/// # Safety
/// `pp` points at a string option's variable.
pub unsafe fn check_string_option(pp: *mut *mut c_char) {
    // SAFETY: the caller's variable.
    unsafe {
        if (*pp).is_null() {
            *pp = empty_string_option.ptr().cast::<c_char>();
        }
    }
}

/// Is `val` a name 'filetype', 'syntax' or 'keymap' will accept?
///
/// # Safety
/// `val` is a C string.
pub(crate) unsafe fn valid_filetype(val: *const c_char) -> bool {
    // SAFETY: `valid_name` only reads the string.
    unsafe { valid_name(val, c".-_".as_ptr()) }
}

/// Parse 'signcolumn' and, given a window, store the width range it asks
/// for.
///
/// `scl` overrides the window's own value; a null `wp` only validates. The
/// two halves are separate grammars: everything the generated table lists
/// ("no", "yes", "yes:1".."yes:9", "auto", "auto:1".."auto:9", "number"),
/// and then the `auto:<min>-<max>` range, which the table cannot enumerate
/// and which is parsed by hand.
///
/// # Safety
/// `scl` is null or a C string; `wp` is null or a live window.
pub unsafe extern "C" fn check_signcolumn(scl: *mut c_char, wp: *mut win_T) -> c_int {
    let val = if !scl.is_null() {
        scl.cast_const()
    } else if !wp.is_null() {
        // SAFETY: the caller's window.
        unsafe { (*wp).w_onebuf_opt.wo_scl }
    } else {
        empty_string_option.ptr().cast::<c_char>()
    };
    // SAFETY: an option value is a C string.
    let val = unsafe { CStr::from_ptr(val) }.to_bytes();
    if val.is_empty() {
        return FAIL;
    }

    // SAFETY: the table's own array, and no mask is wanted.
    let listed = unsafe {
        opt_strings_flags(
            val.as_ptr().cast::<c_char>(),
            opt_scl_values.ptr().cast::<*const c_char>(),
            ptr::null_mut(),
            false,
        )
    } == OK;

    let (min, max) = if listed {
        if wp.is_null() {
            return OK;
        }
        // SAFETY: the caller's window; 'number' only wins when the window
        // is actually showing numbers.
        let numbered = unsafe { (*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0 };
        match val {
            [b'n', b'o', ..] => (SCL_NO, SCL_NO),
            [b'n', b'u', ..] if numbered => (SCL_NUM, SCL_NUM),
            [b'y', b'e', b's', b':', n, ..] => (digit(*n), digit(*n)),
            [b'y', ..] => (1, 1),
            [b'a', b'u', b't', b'o', b':', n, ..] => (0, digit(*n)),
            _ => (0, 1),
        }
    } else {
        // "auto:<min>-<max>", the one spelling the table cannot list.
        let [b'a', b'u', b't', b'o', b':', min, b'-', max] = val else {
            return FAIL;
        };
        if !ascii_isdigit(c_int::from(*min)) || !ascii_isdigit(c_int::from(*max)) {
            return FAIL;
        }
        let (min, max) = (digit(*min), digit(*max));
        if min < 1 || max < 2 || min > 8 || min >= max {
            return FAIL;
        }
        if wp.is_null() {
            return OK;
        }
        (min, max)
    };

    // SAFETY: the caller's window, which the null tests above ruled out.
    unsafe {
        (*wp).w_minscwidth = min;
        (*wp).w_maxscwidth = max;
        // Keep the width the window is currently drawing inside the new
        // range, without widening it on its own.
        let held = if min <= 0 {
            0
        } else {
            max.min((*wp).w_scwidth)
        };
        (*wp).w_scwidth = min.max(held);
    }
    OK
}

/// One ASCII digit as its value. Every caller has already established that
/// the byte is a digit, or is in the half of 'signcolumn' the option table
/// vetted.
fn digit(byte: u8) -> c_int {
    c_int::from(byte) - c_int::from(b'0')
}

/// Every item 'statusline' and its relatives accept after a `%`.
///
/// The three tab-page items repeat at the end because upstream builds
/// `STL_ALL` by concatenating the item list with the tab-page one, which
/// already contains them. Duplicates do not matter to a membership test;
/// the set is kept exactly as upstream spells it.
const STL_ALL: &CStr = c"fFtcvVlLnkoObBrRhHyYwWmMqpPaNSCs{=<*#$TX@TX@";

/// Check a 'statusline'-format value. Returns an untranslated message, or
/// null when the format is good.
///
/// # Safety
/// `s` is a C string.
pub unsafe extern "C" fn check_stl_option(s: *mut c_char) -> *const c_char {
    // The message needs to outlive this call, and the caller passes no
    // buffer, so upstream formats into a function-local static.
    static ERRBUF: GlobalCell<[c_char; 80]> = GlobalCell::new([0; 80]);
    let errbuf = ERRBUF.ptr().cast::<c_char>();
    let errbuflen = size_of::<[c_char; 80]>();

    // SAFETY: the caller's C string.
    let mut rest = unsafe { CStr::from_ptr(s) }.to_bytes();
    let mut groupdepth: c_int = 0;

    loop {
        let Some(at) = rest.iter().position(|&b| b == b'%') else {
            break;
        };
        // Past the `%`. The value may end here, in which case the item is
        // the terminator and the membership test below rejects it.
        rest = &rest[at + 1..];
        match rest.first() {
            // "%%", the truncation mark and the item separator take no
            // width, no precision and no argument.
            Some(&b'%' | &b'<' | &b'=') => {
                rest = &rest[1..];
                continue;
            }
            Some(&b')') => {
                rest = &rest[1..];
                groupdepth -= 1;
                if groupdepth < 0 {
                    break;
                }
                continue;
            }
            _ => {}
        }
        // A minimum width, optionally left-aligned.
        rest = rest.strip_prefix(b"-").unwrap_or(rest);
        rest = &rest[rest.iter().take_while(|b| b.is_ascii_digit()).count()..];
        // A user highlight group takes the width as its number and stops
        // there.
        if rest.first() == Some(&b'*') {
            continue;
        }
        // A maximum width.
        if let Some(after) = rest.strip_prefix(b".") {
            rest = &after[after.iter().take_while(|b| b.is_ascii_digit()).count()..];
        }
        if rest.first() == Some(&b'(') {
            groupdepth += 1;
            continue;
        }
        let Some(&item) = rest.first() else {
            // SAFETY: the module's own buffer.
            return unsafe { illegal_char(errbuf, errbuflen, NUL) };
        };
        // SAFETY: `STL_ALL` is a C string; `vim_strchr` only reads it.
        if unsafe { vim_strchr(STL_ALL.as_ptr(), c_int::from(item)) }.is_null() {
            // SAFETY: the module's own buffer.
            return unsafe { illegal_char(errbuf, errbuflen, c_int::from(item)) };
        }
        if item == b'{' {
            rest = &rest[1..];
            // "%{%…%}" re-evaluates its result as another format, and its
            // terminator is "%}" rather than a bare "}".
            let reevaluate = rest.first() == Some(&b'%');
            if reevaluate {
                rest = &rest[1..];
                if rest.first() == Some(&b'}') {
                    // SAFETY: the module's own buffer.
                    return unsafe { illegal_char(errbuf, errbuflen, c_int::from(b'}')) };
                }
            }
            let close = if reevaluate {
                rest.windows(2).position(|w| w == b"%}").map(|at| at + 1)
            } else {
                rest.iter().position(|&b| b == b'}')
            };
            let Some(close) = close else {
                return e_unclosed_expression_sequence.as_ptr();
            };
            rest = &rest[close..];
        }
    }

    if groupdepth != 0 {
        return e_unbalanced_groups.as_ptr();
    }
    ptr::null()
}

/// Does `val` hold a character an option marked as a file or directory name
/// refuses? The set is wider while 'secure' is on.
///
/// # Safety
/// `val` is a C string.
pub unsafe extern "C" fn check_illegal_path_names(val: *mut c_char, flags: uint32_t) -> bool {
    // SAFETY: the caller's C string.
    let val = unsafe { CStr::from_ptr(val) }.to_bytes();
    let holds = |set: &[u8]| val.iter().any(|b| set.contains(b));
    (flags & kOptFlagNFname as uint32_t != 0
        && holds(if secure.get() != 0 {
            b"/\\*?[|;&<>\r\n"
        } else {
            b"/\\*?[<>\r\n"
        }))
        || (flags & kOptFlagNDname as uint32_t != 0 && holds(b"*?[|;&<>\r\n"))
}
