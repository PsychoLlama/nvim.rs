use crate::src::nvim::charset::{rem_backslash, skipwhite, transstr, vim_str2nr};
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::typval::{
    tv_check_for_number_arg, tv_check_for_opt_bool_arg, tv_check_for_opt_number_arg,
    tv_check_for_opt_string_arg, tv_check_for_string_arg, tv_get_bool, tv_get_bool_chk,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_get_string_chk,
    tv_list_alloc_ret, tv_list_append_number,
};
use crate::src::nvim::ex_docmd::find_cmdline_var;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{e_invarg, e_invarg2, e_using_number_as_bool_nr, e_val_too_large_len};
use crate::src::nvim::mbyte::{
    mb_copy_char, mb_cptr2char_adv, mb_ptr2char_adv, mb_string2cells, mb_tolower, mb_toupper,
    utf8len_tab, utf_char2bytes, utf_char2len, utf_head_off, utf_ptr2CharInfo_impl, utf_ptr2cells,
    utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memory::{
    arena_alloc, arena_alloc_block, xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xmemscan,
    xrealloc, xstrchrnul, xstrlcpy,
};
use crate::src::nvim::message::{emsg, semsg, siemsg};
use crate::src::nvim::option::{csh_like_shell, fish_like_shell};
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, log10, memcpy, memmove, memset, qsort, snprintf, strcasecmp, strchr,
    strcmp, strcpy, strlen, strncmp, strstr, vsnprintf,
};
use crate::src::nvim::plines::linetabsize_col;
pub use crate::src::nvim::types::{
    __builtin_va_list, __compar_fn_t, __gnuc_va_list, __va_list_tag, blob_T, blobvar_S, dict_T,
    dictvar_S, float_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T,
    hash_T, hashitem_T, hashtab_T, int16_t, int32_t, int64_t, intmax_t, keyvalue_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, partial_S, partial_T,
    proftime_T, ptrdiff_t, queue, scid_T, sctx_T, size_t, ssize_t, typval_T, typval_vval_union,
    ufunc_S, ufunc_T, uint16_t, uint64_t, uint8_t, uintmax_t, uintptr_t, uvarnumber_T, va_list,
    varnumber_T, Arena, BoolVarValue, CharInfo, EvalFuncData, ListLenSpecials, LuaRef,
    MsgpackRpcRequestHandler, ScopeDictDictItem, ScopeType, SpecialVarValue, StringBuilder,
    String_0, VarLockStatus, VarType, QUEUE,
};
use core::ffi::{c_char, c_int, CStr};
use core::ptr;
use core::slice;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const TYPE_FLOAT: C2Rust_Unnamed_1 = 12;
pub const TYPE_SIZET: C2Rust_Unnamed_1 = 7;
pub const TYPE_UNSIGNEDLONGLONGINT: C2Rust_Unnamed_1 = 6;
pub const TYPE_UNSIGNEDLONGINT: C2Rust_Unnamed_1 = 5;
pub const TYPE_UNSIGNEDINT: C2Rust_Unnamed_1 = 4;
pub const TYPE_SIGNEDSIZET: C2Rust_Unnamed_1 = 3;
pub const TYPE_LONGLONGINT: C2Rust_Unnamed_1 = 2;
pub const TYPE_LONGINT: C2Rust_Unnamed_1 = 1;
pub const TYPE_INT: C2Rust_Unnamed_1 = 0;
pub const TYPE_POINTER: C2Rust_Unnamed_1 = 8;
pub const TYPE_STRING: C2Rust_Unnamed_1 = 11;
pub const TYPE_CHAR: C2Rust_Unnamed_1 = 10;
pub const TYPE_UNKNOWN: C2Rust_Unnamed_1 = -1;
pub const TYPE_PERCENT: C2Rust_Unnamed_1 = 9;
pub const MAX_ALLOWED_STRING_WIDTH: C2Rust_Unnamed_2 = 1048576;
pub const STR2NR_FORCE: C2Rust_Unnamed_0 = 128;
pub const STR2NR_HEX: C2Rust_Unnamed_0 = 4;
pub const STR2NR_OOCT: C2Rust_Unnamed_0 = 8;
pub const STR2NR_OCT: C2Rust_Unnamed_0 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_0 = 1;
pub const STR2NR_QUOTE: C2Rust_Unnamed_0 = 16;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_0 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_0 = 15;
pub const STR2NR_DEC: C2Rust_Unnamed_0 = 0;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
fn ascii_iswhite(c: ::core::ffi::c_int) -> bool {
    c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int
}
#[inline(always)]
fn ascii_isdigit(c: ::core::ffi::c_int) -> bool {
    c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
}
static e_cannot_mix_positional_and_non_positional_str: GlobalCell<[::core::ffi::c_char; 62]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
            *b"E1500: Cannot mix positional and non-positional arguments: %s\0",
        )
    });
static e_fmt_arg_nr_unused_str: GlobalCell<[::core::ffi::c_char; 55]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
        *b"E1501: format argument %d unused in $-style format: %s\0",
    )
});
static e_positional_num_field_spec_reused_str_str: GlobalCell<[::core::ffi::c_char; 82]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 82], [::core::ffi::c_char; 82]>(
            *b"E1502: Positional argument %d used as field width reused as different type: %s/%s\0",
        )
    });
static e_positional_nr_out_of_bounds_str: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E1503: Positional argument %d out of bounds: %s\0",
        )
    });
static e_positional_arg_num_type_inconsistent_str_str: GlobalCell<[::core::ffi::c_char; 62]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
            *b"E1504: Positional argument %d type used inconsistently: %s/%s\0",
        )
    });
static e_invalid_format_specifier_str: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E1505: Invalid format specifier: %s\0",
        )
    });
static e_aptypes_is_null_nr_str: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E1507: Internal error: ap_types or ap_types[idx] is NULL: %d: %s\0",
    )
});
static typename_unknown: GlobalCell<[::core::ffi::c_char; 8]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"unknown\0")
});
static typename_int: GlobalCell<[::core::ffi::c_char; 4]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"int\0")
});
static typename_longint: GlobalCell<[::core::ffi::c_char; 9]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"long int\0")
});
static typename_longlongint: GlobalCell<[::core::ffi::c_char; 14]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"long long int\0")
});
static typename_signedsizet: GlobalCell<[::core::ffi::c_char; 14]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"signed size_t\0")
});
static typename_unsignedint: GlobalCell<[::core::ffi::c_char; 13]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"unsigned int\0")
});
static typename_unsignedlongint: GlobalCell<[::core::ffi::c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"unsigned long int\0")
});
static typename_unsignedlonglongint: GlobalCell<[::core::ffi::c_char; 23]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"unsigned long long int\0")
    });
static typename_sizet: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"size_t\0")
});
static typename_pointer: GlobalCell<[::core::ffi::c_char; 8]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"pointer\0")
});
static typename_percent: GlobalCell<[::core::ffi::c_char; 8]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"percent\0")
});
static typename_char: GlobalCell<[::core::ffi::c_char; 5]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"char\0")
});
static typename_string: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"string\0")
});
static typename_float: GlobalCell<[::core::ffi::c_char; 6]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"float\0")
});
// ── The vim_str* family: safe cores + C-ABI shims ─────────────────────────
//
// Byte-level logic (unquoting, ASCII case mapping, comparison, scanning)
// lives in safe functions; the shims confine the raw-pointer plumbing.
// Multibyte-aware functions (vim_strsave_escaped_ext, shellescape,
// strcase_save, vim_strchr) still call the transpiled mbyte/ex_docmd
// machinery through raw pointers and remain shims throughout. Results are
// allocated with the xmalloc family so ownership crosses the C ABI as
// before.

/// `strnlen`: bytes before the terminator, reading at most `maxlen` bytes.
unsafe fn strnlen(s: *const c_char, maxlen: size_t) -> size_t {
    let mut n = 0;
    while n < maxlen && *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// ASCII-uppercase `s` in place (bytes ≥ 0x80 untouched).
fn ascii_upcase(s: &mut [u8]) {
    for b in s {
        if b.is_ascii_lowercase() {
            *b -= 0x20;
        }
    }
}

/// Any byte outside 7-bit ASCII?
fn any_non_ascii(s: &[u8]) -> bool {
    s.iter().any(|&b| b >= 0x80)
}

/// Walk `src` as `vim_strnsave_unquoted` reads it, feeding kept bytes to
/// `emit`: unescaped double quotes toggle quote mode and vanish; inside
/// quotes `\\` and `\"` collapse to the escaped byte; everything else
/// (including other backslash sequences) passes through untouched.
fn unquote(src: &[u8], emit: &mut impl FnMut(u8)) {
    let mut inquote = false;
    let mut i = 0;
    while i < src.len() {
        let b = src[i];
        if b == b'"' {
            inquote = !inquote;
        } else if b == b'\\' && inquote && i + 1 < src.len() && matches!(src[i + 1], b'\\' | b'"') {
            i += 1;
            emit(src[i]);
        } else {
            emit(b);
        }
        i += 1;
    }
}

/// Index where removable trailing whitespace starts: spaces/tabs not
/// preceded by a backslash or Ctrl-V, never including the first byte.
fn trailing_spaces_start(s: &[u8]) -> usize {
    let mut end = s.len();
    while end > 1
        && matches!(s[end - 1], b' ' | b'\t')
        && s[end - 2] != b'\\'
        && s[end - 2] != Ctrl_V as u8
    {
        end -= 1;
    }
    end
}

/// ASCII-case-insensitive `strncmp`. Bytes fold exactly as the C code's
/// signed chars did: only A–Z map (down) to lowercase, and bytes ≥ 0x80
/// compare negative.
fn strnicmp_asc(a: &[u8], b: &[u8], len: size_t) -> c_int {
    let fold = |c: u8| -> c_int {
        let c = c as i8 as c_int;
        if !('A' as c_int..='Z' as c_int).contains(&c) {
            c
        } else {
            c + 0x20
        }
    };
    let mut diff = 0;
    for k in 0..len {
        let ca = a.get(k).copied().unwrap_or(0);
        let cb = b.get(k).copied().unwrap_or(0);
        diff = fold(ca) - fold(cb);
        if diff != 0 || ca == 0 {
            break;
        }
    }
    diff
}

/// Copy at most `len` bytes of `string` into a fresh NUL-terminated
/// buffer, zero-filling the remainder (strncpy semantics).
pub unsafe extern "C" fn xstrnsave(string: *const c_char, len: size_t) -> *mut c_char {
    let n = strnlen(string, len);
    let ret = xmallocz(len) as *mut c_char;
    let out = slice::from_raw_parts_mut(ret as *mut u8, len);
    if n != 0 {
        out[..n].copy_from_slice(slice::from_raw_parts(string as *const u8, n));
    }
    out[n..].fill(0);
    ret
}

pub unsafe extern "C" fn vim_strsave_escaped(
    string: *const c_char,
    esc_chars: *const c_char,
) -> *mut c_char {
    vim_strsave_escaped_ext(string, esc_chars, b'\\' as c_char, false)
}

/// Copy `string`, prefixing `cc` to every byte in `esc_chars` (and, with
/// `bsl`, to the backslashes `rem_backslash` flags). Multibyte characters
/// are copied whole and never escaped.
pub unsafe extern "C" fn vim_strsave_escaped_ext(
    string: *const c_char,
    esc_chars: *const c_char,
    cc: c_char,
    bsl: bool,
) -> *mut c_char {
    // First pass: measure (1 for the terminating NUL).
    let mut length: size_t = 1;
    let mut p = string;
    while *p != 0 {
        let l = utfc_ptr2len(p) as size_t;
        if l > 1 {
            length = length.wrapping_add(l);
            p = p.add(l);
            continue;
        }
        if !vim_strchr(esc_chars, *p as u8 as c_int).is_null() || (bsl && rem_backslash(p)) {
            length = length.wrapping_add(1);
        }
        length = length.wrapping_add(1);
        p = p.add(1);
    }

    let escaped_string = xmalloc(length) as *mut c_char;
    let mut p2 = escaped_string;
    let mut p = string;
    while *p != 0 {
        let l = utfc_ptr2len(p) as size_t;
        if l > 1 {
            ptr::copy_nonoverlapping(p, p2, l);
            p2 = p2.add(l);
            p = p.add(l);
            continue;
        }
        if !vim_strchr(esc_chars, *p as u8 as c_int).is_null() || (bsl && rem_backslash(p)) {
            *p2 = cc;
            p2 = p2.add(1);
        }
        *p2 = *p;
        p2 = p2.add(1);
        p = p.add(1);
    }
    *p2 = 0;
    escaped_string
}

/// Copy `length` bytes of `string` with shell-style double-quoting
/// resolved (see `unquote`), NUL-terminated.
pub unsafe extern "C" fn vim_strnsave_unquoted(
    string: *const c_char,
    length: size_t,
) -> *mut c_char {
    if length == 0 {
        return xmallocz(0) as *mut c_char;
    }
    let src = slice::from_raw_parts(string as *const u8, length);
    let mut n: size_t = 0;
    unquote(src, &mut |_| n += 1);
    let ret = xmallocz(n) as *mut c_char;
    let out = slice::from_raw_parts_mut(ret as *mut u8, n);
    let mut o = 0;
    unquote(src, &mut |b| {
        out[o] = b;
        o += 1;
    });
    ret
}

/// Single-quote `string` for the shell, doubling embedded quotes
/// (`'` → `'\''`) and — depending on the shell flavor and flags — escaping
/// newlines, `!`, `\`, and `%`/`#` cmdline specials.
pub unsafe extern "C" fn vim_strsave_shellescape(
    string: *const c_char,
    do_special: bool,
    do_newline: bool,
) -> *mut c_char {
    let csh_like = csh_like_shell() != 0;
    let fish_like = fish_like_shell();
    let mut l: size_t = 0;

    // First pass: measure (3 = the surrounding quotes plus NUL).
    let mut length: size_t = strlen(string).wrapping_add(3);
    let mut p = string;
    while *p != 0 {
        if *p == b'\'' as c_char {
            length = length.wrapping_add(3);
        }
        if (*p == b'\n' as c_char && (csh_like || do_newline))
            || (*p == b'!' as c_char && (csh_like || do_special))
        {
            length = length.wrapping_add(1);
            if csh_like && do_special {
                length = length.wrapping_add(1);
            }
        }
        if do_special && find_cmdline_var(p, &mut l) >= 0 {
            length = length.wrapping_add(1); // insert backslash
            p = p.add(l.wrapping_sub(1));
        }
        if *p == b'\\' as c_char && fish_like {
            length = length.wrapping_add(1);
        }
        p = p.add(utfc_ptr2len(p) as usize);
    }

    let escaped_string = xmalloc(length) as *mut c_char;
    let mut d = escaped_string;
    *d = b'\'' as c_char;
    d = d.add(1);
    let mut p = string;
    while *p != 0 {
        if *p == b'\'' as c_char {
            // A single-quoted string cannot contain a quote: close it,
            // emit an escaped quote, and reopen.
            for &b in b"'\\''" {
                *d = b as c_char;
                d = d.add(1);
            }
            p = p.add(1);
            continue;
        }
        if (*p == b'\n' as c_char && (csh_like || do_newline))
            || (*p == b'!' as c_char && (csh_like || do_special))
        {
            *d = b'\\' as c_char;
            d = d.add(1);
            if csh_like && do_special {
                *d = b'\\' as c_char;
                d = d.add(1);
            }
            *d = *p;
            d = d.add(1);
            p = p.add(1);
            continue;
        }
        if do_special && find_cmdline_var(p, &mut l) >= 0 {
            *d = b'\\' as c_char; // insert backslash
            d = d.add(1);
            ptr::copy_nonoverlapping(p, d, l); // copy the var
            d = d.add(l);
            p = p.add(l);
            continue;
        }
        if *p == b'\\' as c_char && fish_like {
            *d = b'\\' as c_char;
            d = d.add(1);
            *d = *p;
            d = d.add(1);
            p = p.add(1);
            continue;
        }
        mb_copy_char(&mut p, &mut d);
    }
    *d = b'\'' as c_char;
    d = d.add(1);
    *d = 0;
    escaped_string
}

/// ASCII-uppercased copy of `string`.
pub unsafe extern "C" fn vim_strsave_up(string: *const c_char) -> *mut c_char {
    let p1 = xmalloc(strlen(string).wrapping_add(1)) as *mut c_char;
    vim_strcpy_up(p1, string);
    p1
}

/// ASCII-uppercased copy of at most `len` bytes of `string`.
pub unsafe extern "C" fn vim_strnsave_up(string: *const c_char, len: size_t) -> *mut c_char {
    let p1 = xmalloc(len.wrapping_add(1)) as *mut c_char;
    vim_strncpy_up(p1, string, len);
    p1
}

/// ASCII-uppercase the C string in place.
pub unsafe extern "C" fn vim_strup(p: *mut c_char) {
    let len = CStr::from_ptr(p).to_bytes().len();
    ascii_upcase(slice::from_raw_parts_mut(p as *mut u8, len));
}

/// `strcpy` that ASCII-uppercases while copying.
pub unsafe extern "C" fn vim_strcpy_up(dst: *mut c_char, src: *const c_char) {
    let bytes = CStr::from_ptr(src).to_bytes_with_nul();
    let out = slice::from_raw_parts_mut(dst as *mut u8, bytes.len());
    out.copy_from_slice(bytes);
    ascii_upcase(&mut out[..bytes.len() - 1]);
}

/// Like `vim_strcpy_up` but copies at most `n` bytes; always terminates.
pub unsafe extern "C" fn vim_strncpy_up(dst: *mut c_char, src: *const c_char, n: size_t) {
    let len = strnlen(src, n);
    let out = slice::from_raw_parts_mut(dst as *mut u8, len + 1);
    if len != 0 {
        out[..len].copy_from_slice(slice::from_raw_parts(src as *const u8, len));
        ascii_upcase(&mut out[..len]);
    }
    out[len] = 0;
}

/// `memcpy` that ASCII-uppercases while copying: exactly `n` bytes, no
/// terminator.
pub unsafe extern "C" fn vim_memcpy_up(dst: *mut c_char, src: *const c_char, n: size_t) {
    if n == 0 {
        return;
    }
    let out = slice::from_raw_parts_mut(dst as *mut u8, n);
    out.copy_from_slice(slice::from_raw_parts(src as *const u8, n));
    ascii_upcase(out);
}

/// Case-fold `orig` per character (multibyte-aware), growing the result
/// when a folded character encodes longer than its original.
pub unsafe extern "C" fn strcase_save(orig: *const c_char, upper: bool) -> *mut c_char {
    let mut orig_len = strlen(orig);
    let mut res = xmalloc(orig_len.wrapping_add(1)) as *mut c_char;
    let mut res_index: size_t = 0;
    let mut p = orig;
    while *p != 0 {
        let char_info = utf_ptr2CharInfo(p);
        let c = if char_info.value < 0 {
            *p as u8 as c_int
        } else {
            char_info.value as c_int
        };
        let newc = if upper { mb_toupper(c) } else { mb_tolower(c) };
        let newl = utf_char2len(newc) as size_t;
        if res_index.wrapping_add(newl) > orig_len {
            let new_size = res_index.wrapping_add(newl).wrapping_add(1);
            res = xrealloc(res as *mut ::core::ffi::c_void, new_size) as *mut c_char;
            orig_len = new_size.wrapping_sub(1);
        }
        utf_char2bytes(newc, res.add(res_index));
        res_index = res_index.wrapping_add(newl);
        p = p.add(char_info.len as usize);
    }
    *res.add(res_index) = 0;
    res
}

/// Truncate unescaped trailing spaces and tabs in place.
pub unsafe extern "C" fn del_trailing_spaces(ptr: *mut c_char) {
    let len = CStr::from_ptr(ptr).to_bytes().len();
    let s = slice::from_raw_parts_mut(ptr as *mut u8, len);
    let end = trailing_spaces_start(s);
    s[end..].fill(0);
}

/// Case-insensitive `strcmp` equality where NULL only equals NULL.
/// strcasecmp is locale-aware, so the libc call stays.
pub unsafe extern "C" fn striequal(a: *const c_char, b: *const c_char) -> bool {
    (a.is_null() && b.is_null()) || (!a.is_null() && !b.is_null() && strcasecmp(a, b) == 0)
}

pub unsafe extern "C" fn vim_strnicmp_asc(
    s1: *const c_char,
    s2: *const c_char,
    len: size_t,
) -> c_int {
    strnicmp_asc(
        CStr::from_ptr(s1).to_bytes(),
        CStr::from_ptr(s2).to_bytes(),
        len,
    )
}

/// Find character `c` (a codepoint, not a byte) in `string`.
pub unsafe extern "C" fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char {
    if c <= 0 {
        ptr::null_mut()
    } else if c < 0x80 {
        strchr(string, c)
    } else {
        let mut u8char = [0 as c_char; 22];
        let len = utf_char2bytes(c, u8char.as_mut_ptr());
        u8char[len as usize] = 0;
        strstr(string, u8char.as_ptr())
    }
}

unsafe extern "C" fn sort_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> c_int {
    strcmp(*(s1 as *const *const c_char), *(s2 as *const *const c_char))
}

pub unsafe extern "C" fn sort_strings(files: *mut *mut c_char, count: c_int) {
    qsort(
        files as *mut ::core::ffi::c_void,
        count as size_t,
        ::core::mem::size_of::<*mut c_char>(),
        Some(
            sort_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> c_int,
        ),
    );
}

pub unsafe extern "C" fn has_non_ascii(s: *const c_char) -> bool {
    !s.is_null() && any_non_ascii(CStr::from_ptr(s).to_bytes())
}

/// Freshly allocated `str1 ++ str2`, NUL-terminated.
pub unsafe extern "C" fn concat_str(str1: *const c_char, str2: *const c_char) -> *mut c_char {
    let a = CStr::from_ptr(str1).to_bytes();
    let b = CStr::from_ptr(str2).to_bytes_with_nul();
    let dest = xmalloc(a.len() + b.len()) as *mut c_char;
    let out = slice::from_raw_parts_mut(dest as *mut u8, a.len() + b.len());
    out[..a.len()].copy_from_slice(a);
    out[a.len()..].copy_from_slice(b);
    dest
}
static e_printf: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E766: Insufficient arguments for printf()\0".as_ptr() as *const ::core::ffi::c_char,
);
unsafe extern "C" fn tv_nr(
    mut tvs: *mut typval_T,
    mut idxp: *mut ::core::ffi::c_int,
) -> varnumber_T {
    let mut idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
    let mut n: varnumber_T = 0 as varnumber_T;
    if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(e_printf.get()));
    } else {
        *idxp += 1;
        let mut err: bool = false_0 != 0;
        n = tv_get_number_chk(tvs.offset(idx as isize), &raw mut err);
        if err {
            n = 0 as varnumber_T;
        }
    }
    return n;
}
unsafe extern "C" fn tv_str(
    mut tvs: *mut typval_T,
    mut idxp: *mut ::core::ffi::c_int,
    tofree: *mut *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(e_printf.get()));
    } else {
        *idxp += 1;
        if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            s = tv_get_string_chk(tvs.offset(idx as isize));
            *tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            *tofree = encode_tv2echo(tvs.offset(idx as isize), ::core::ptr::null_mut::<size_t>());
            s = *tofree;
        }
    }
    return s;
}
unsafe extern "C" fn tv_ptr(
    tvs: *const typval_T,
    idxp: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_void {
    let idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
    if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(e_printf.get()));
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *idxp += 1;
    return (*tvs.offset(idx as isize)).vval.v_string as *const ::core::ffi::c_void;
}
unsafe extern "C" fn tv_float(tvs: *mut typval_T, idxp: *mut ::core::ffi::c_int) -> float_T {
    let mut idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
    let mut f: float_T = 0 as ::core::ffi::c_int as float_T;
    if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(e_printf.get()));
    } else {
        *idxp += 1;
        if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            f = (*tvs.offset(idx as isize)).vval.v_float;
        } else if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            f = (*tvs.offset(idx as isize)).vval.v_number as float_T;
        } else {
            emsg(gettext(
                b"E807: Expected Float argument for printf()\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        }
    }
    return f;
}
pub unsafe extern "C" fn vim_snprintf_add(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let len: size_t = strlen(str);
    let mut space: size_t = 0;
    if str_m <= len {
        space = 0 as size_t;
    } else {
        space = str_m.wrapping_sub(len);
    }
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    let str_l: ::core::ffi::c_int = vim_vsnprintf(str.offset(len as isize), space, fmt, ap);
    return str_l;
}
pub unsafe extern "C" fn vim_snprintf(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    let str_l: ::core::ffi::c_int = vim_vsnprintf(str, str_m, fmt, ap);
    return str_l;
}
unsafe extern "C" fn infinity_str(
    mut positive: bool,
    mut fmt_spec: ::core::ffi::c_char,
    mut force_sign: ::core::ffi::c_int,
    mut space_for_positive: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static table: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
        b"-inf\0".as_ptr() as *const ::core::ffi::c_char,
        b"inf\0".as_ptr() as *const ::core::ffi::c_char,
        b"+inf\0".as_ptr() as *const ::core::ffi::c_char,
        b" inf\0".as_ptr() as *const ::core::ffi::c_char,
        b"-INF\0".as_ptr() as *const ::core::ffi::c_char,
        b"INF\0".as_ptr() as *const ::core::ffi::c_char,
        b"+INF\0".as_ptr() as *const ::core::ffi::c_char,
        b" INF\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut idx: ::core::ffi::c_int = positive as ::core::ffi::c_int
        * (1 as ::core::ffi::c_int + force_sign + force_sign * space_for_positive);
    if fmt_spec as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && fmt_spec as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        idx += 4 as ::core::ffi::c_int;
    }
    return (*table.ptr())[idx as usize];
}
pub unsafe extern "C" fn vim_snprintf_safelen(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> size_t {
    let mut ap: ::core::ffi::VaList;
    let mut str_l: ::core::ffi::c_int = 0;
    if str_m == 0 as size_t {
        return 0 as size_t;
    }
    ap = c2rust_args.clone();
    str_l = vim_vsnprintf_typval(str, str_m, fmt, ap, ::core::ptr::null_mut::<typval_T>());
    if str_l < 0 as ::core::ffi::c_int {
        *str = NUL as ::core::ffi::c_char;
        return 0 as size_t;
    }
    return if str_l as size_t >= str_m {
        str_m.wrapping_sub(1 as size_t)
    } else {
        str_l as size_t
    };
}
pub unsafe extern "C" fn vim_vsnprintf(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> ::core::ffi::c_int {
    return vim_vsnprintf_typval(str, str_m, fmt, ap, ::core::ptr::null_mut::<typval_T>());
}
unsafe extern "C" fn format_typeof(mut type_0: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut length_modifier: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut fmt_spec: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    if *type_0 as ::core::ffi::c_int == 'h' as ::core::ffi::c_int
        || *type_0 as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
        || *type_0 as ::core::ffi::c_int == 'z' as ::core::ffi::c_int
    {
        length_modifier = *type_0;
        type_0 = type_0.offset(1);
        if length_modifier as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
            && *type_0 as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
        {
            length_modifier = 'L' as ::core::ffi::c_char;
            type_0 = type_0.offset(1);
        }
    }
    fmt_spec = *type_0;
    match fmt_spec as ::core::ffi::c_int {
        105 => {
            fmt_spec = 'd' as ::core::ffi::c_char;
        }
        42 => {
            fmt_spec = 'd' as ::core::ffi::c_char;
            length_modifier = 'h' as ::core::ffi::c_char;
        }
        68 => {
            fmt_spec = 'd' as ::core::ffi::c_char;
            length_modifier = 'l' as ::core::ffi::c_char;
        }
        85 => {
            fmt_spec = 'u' as ::core::ffi::c_char;
            length_modifier = 'l' as ::core::ffi::c_char;
        }
        79 => {
            fmt_spec = 'o' as ::core::ffi::c_char;
            length_modifier = 'l' as ::core::ffi::c_char;
        }
        _ => {}
    }
    match fmt_spec as ::core::ffi::c_int {
        37 => return TYPE_PERCENT as ::core::ffi::c_int,
        99 => return TYPE_CHAR as ::core::ffi::c_int,
        115 | 83 => return TYPE_STRING as ::core::ffi::c_int,
        100 | 117 | 98 | 66 | 111 | 120 | 88 | 112 => {
            if fmt_spec as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
                return TYPE_POINTER as ::core::ffi::c_int;
            } else if fmt_spec as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                || fmt_spec as ::core::ffi::c_int == 'B' as ::core::ffi::c_int
            {
                return TYPE_UNSIGNEDLONGLONGINT as ::core::ffi::c_int;
            } else if fmt_spec as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                match length_modifier as ::core::ffi::c_int {
                    NUL | 104 => return TYPE_INT as ::core::ffi::c_int,
                    108 => return TYPE_LONGINT as ::core::ffi::c_int,
                    76 => return TYPE_LONGLONGINT as ::core::ffi::c_int,
                    122 => return TYPE_SIGNEDSIZET as ::core::ffi::c_int,
                    _ => {}
                }
            } else {
                match length_modifier as ::core::ffi::c_int {
                    NUL | 104 => return TYPE_UNSIGNEDINT as ::core::ffi::c_int,
                    108 => return TYPE_UNSIGNEDLONGINT as ::core::ffi::c_int,
                    76 => return TYPE_UNSIGNEDLONGLONGINT as ::core::ffi::c_int,
                    122 => return TYPE_SIZET as ::core::ffi::c_int,
                    _ => {}
                }
            }
        }
        102 | 70 | 101 | 69 | 103 | 71 => return TYPE_FLOAT as ::core::ffi::c_int,
        _ => {}
    }
    return TYPE_UNKNOWN as ::core::ffi::c_int;
}
unsafe extern "C" fn format_typename(
    mut type_0: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    match format_typeof(type_0) {
        0 => return gettext((typename_int.ptr() as *const _) as *const ::core::ffi::c_char),
        1 => return gettext((typename_longint.ptr() as *const _) as *const ::core::ffi::c_char),
        2 => {
            return gettext((typename_longlongint.ptr() as *const _) as *const ::core::ffi::c_char);
        }
        4 => {
            return gettext((typename_unsignedint.ptr() as *const _) as *const ::core::ffi::c_char);
        }
        3 => {
            return gettext((typename_signedsizet.ptr() as *const _) as *const ::core::ffi::c_char);
        }
        5 => {
            return gettext(
                (typename_unsignedlongint.ptr() as *const _) as *const ::core::ffi::c_char,
            );
        }
        6 => {
            return gettext(
                (typename_unsignedlonglongint.ptr() as *const _) as *const ::core::ffi::c_char,
            );
        }
        7 => return gettext((typename_sizet.ptr() as *const _) as *const ::core::ffi::c_char),
        8 => return gettext((typename_pointer.ptr() as *const _) as *const ::core::ffi::c_char),
        9 => return gettext((typename_percent.ptr() as *const _) as *const ::core::ffi::c_char),
        10 => return gettext((typename_char.ptr() as *const _) as *const ::core::ffi::c_char),
        11 => return gettext((typename_string.ptr() as *const _) as *const ::core::ffi::c_char),
        12 => return gettext((typename_float.ptr() as *const _) as *const ::core::ffi::c_char),
        _ => {}
    }
    return gettext((typename_unknown.ptr() as *const _) as *const ::core::ffi::c_char);
}
unsafe extern "C" fn adjust_types(
    mut ap_types: *mut *mut *const ::core::ffi::c_char,
    mut arg: ::core::ffi::c_int,
    mut num_posarg: *mut ::core::ffi::c_int,
    mut type_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if arg <= 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                (e_invalid_format_specifier_str.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            type_0,
        );
        return FAIL;
    }
    if (*ap_types).is_null() || *num_posarg < arg {
        let mut new_types: *mut *const ::core::ffi::c_char = (if (*ap_types).is_null() {
            xcalloc(
                arg as size_t,
                ::core::mem::size_of::<*const ::core::ffi::c_char>(),
            )
        } else {
            xrealloc(
                *ap_types as *mut ::core::ffi::c_void,
                (arg as size_t).wrapping_mul(::core::mem::size_of::<*const ::core::ffi::c_char>()),
            )
        })
            as *mut *const ::core::ffi::c_char;
        let mut idx: ::core::ffi::c_int = *num_posarg;
        while idx < arg {
            *new_types.offset(idx as isize) = ::core::ptr::null::<::core::ffi::c_char>();
            idx += 1;
        }
        *ap_types = new_types;
        *num_posarg = arg;
    }
    if !(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize)).is_null() {
        if *(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize))
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            || *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
        {
            let mut pt: *const ::core::ffi::c_char = type_0;
            if *pt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
            {
                pt = *(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize);
            }
            if *pt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '*' as ::core::ffi::c_int
            {
                match *pt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                    100 | 105 => {}
                    _ => {
                        semsg(
                            gettext(
                                (e_positional_num_field_spec_reused_str_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            arg,
                            format_typename(
                                *(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize),
                            ),
                            format_typename(type_0),
                        );
                        return FAIL;
                    }
                }
            }
        } else if format_typeof(type_0)
            != format_typeof(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize))
        {
            semsg(
                gettext(
                    (e_positional_arg_num_type_inconsistent_str_str.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                arg,
                format_typename(type_0),
                format_typename(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize)),
            );
            return FAIL;
        }
    }
    *(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize) = type_0;
    return OK;
}
unsafe extern "C" fn format_overflow_error(mut pstart: *const ::core::ffi::c_char) {
    let mut p: *const ::core::ffi::c_char = pstart;
    while ascii_isdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    semsg(
        gettext(&raw const e_val_too_large_len as *const ::core::ffi::c_char),
        p.offset_from(pstart) as ::core::ffi::c_int,
        pstart,
    );
}
unsafe extern "C" fn get_unsigned_int(
    mut pstart: *const ::core::ffi::c_char,
    mut p: *mut *const ::core::ffi::c_char,
    mut uj: *mut ::core::ffi::c_uint,
    mut overflow_err: bool,
) -> ::core::ffi::c_int {
    *uj = (**p as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as ::core::ffi::c_uint;
    *p = (*p).offset(1);
    while ascii_isdigit(**p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && *uj < MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *uj = (10 as ::core::ffi::c_uint).wrapping_mul(*uj).wrapping_add(
            (**p as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as ::core::ffi::c_uint,
        );
        *p = (*p).offset(1);
    }
    if *uj > MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int as ::core::ffi::c_uint {
        if overflow_err {
            format_overflow_error(pstart);
            return FAIL;
        } else {
            *uj = MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
    }
    return OK;
}
unsafe extern "C" fn parse_fmt_types(
    mut ap_types: *mut *mut *const ::core::ffi::c_char,
    mut num_posarg: *mut ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut tvs: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = fmt;
    let mut arg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut any_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut any_arg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if p.is_null() {
        return OK;
    }
    '_error: {
        while *p as ::core::ffi::c_int != NUL {
            if *p as ::core::ffi::c_int != '%' as ::core::ffi::c_int {
                let mut n: size_t = xstrchrnul(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    '%' as ::core::ffi::c_char,
                )
                .offset_from(p) as size_t;
                p = p.offset(n as isize);
            } else {
                let mut length_modifier: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                let mut pos_arg: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut pstart: *const ::core::ffi::c_char =
                    p.offset(1 as ::core::ffi::c_int as isize);
                p = p.offset(1);
                let mut ptype: *const ::core::ffi::c_char = p;
                while ascii_isdigit(*ptype as ::core::ffi::c_int) {
                    ptype = ptype.offset(1);
                }
                if *ptype as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                    if *p as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                (e_invalid_format_specifier_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            fmt,
                        );
                        break '_error;
                    } else {
                        let mut uj: ::core::ffi::c_uint = 0;
                        if get_unsigned_int(pstart, &raw mut p, &raw mut uj, !tvs.is_null()) == FAIL
                        {
                            break '_error;
                        }
                        pos_arg = uj as ::core::ffi::c_int;
                        any_pos = 1 as ::core::ffi::c_int;
                        if any_pos != 0 && any_arg != 0 {
                            semsg(
                                gettext(
                                    (e_cannot_mix_positional_and_non_positional_str.ptr()
                                        as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        } else {
                            p = p.offset(1);
                        }
                    }
                }
                while *p as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                {
                    match *p as ::core::ffi::c_int {
                        48 | 45 | 43 | 32 | 35 | 39 | _ => {}
                    }
                    p = p.offset(1);
                }
                arg = p;
                if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                    p = p.offset(1);
                    if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let mut uj_0: ::core::ffi::c_uint = 0;
                        if get_unsigned_int(
                            arg.offset(1 as ::core::ffi::c_int as isize),
                            &raw mut p,
                            &raw mut uj_0,
                            !tvs.is_null(),
                        ) == FAIL
                        {
                            break '_error;
                        }
                        if *p as ::core::ffi::c_int != '$' as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    (e_invalid_format_specifier_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        } else {
                            p = p.offset(1);
                            any_pos = 1 as ::core::ffi::c_int;
                            if any_pos != 0 && any_arg != 0 {
                                semsg(
                                    gettext(
                                        (e_cannot_mix_positional_and_non_positional_str.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            } else if adjust_types(
                                ap_types,
                                uj_0 as ::core::ffi::c_int,
                                num_posarg,
                                arg,
                            ) == FAIL
                            {
                                break '_error;
                            }
                        }
                    } else {
                        any_arg = 1 as ::core::ffi::c_int;
                        if any_pos != 0 && any_arg != 0 {
                            semsg(
                                gettext(
                                    (e_cannot_mix_positional_and_non_positional_str.ptr()
                                        as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        }
                    }
                } else if ascii_isdigit(*p as ::core::ffi::c_int) {
                    let mut digstart: *const ::core::ffi::c_char = p;
                    let mut uj_1: ::core::ffi::c_uint = 0;
                    if get_unsigned_int(digstart, &raw mut p, &raw mut uj_1, !tvs.is_null()) == FAIL
                    {
                        break '_error;
                    }
                    if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                (e_invalid_format_specifier_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            fmt,
                        );
                        break '_error;
                    }
                }
                if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                    p = p.offset(1);
                    arg = p;
                    if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                        p = p.offset(1);
                        if ascii_isdigit(*p as ::core::ffi::c_int) {
                            let mut uj_2: ::core::ffi::c_uint = 0;
                            if get_unsigned_int(
                                arg.offset(1 as ::core::ffi::c_int as isize),
                                &raw mut p,
                                &raw mut uj_2,
                                !tvs.is_null(),
                            ) == FAIL
                            {
                                break '_error;
                            }
                            if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                                any_pos = 1 as ::core::ffi::c_int;
                                if any_pos != 0 && any_arg != 0 {
                                    semsg(
                                        gettext(
                                            (e_cannot_mix_positional_and_non_positional_str.ptr()
                                                as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        fmt,
                                    );
                                    break '_error;
                                } else {
                                    p = p.offset(1);
                                    if adjust_types(
                                        ap_types,
                                        uj_2 as ::core::ffi::c_int,
                                        num_posarg,
                                        arg,
                                    ) == FAIL
                                    {
                                        break '_error;
                                    }
                                }
                            } else {
                                semsg(
                                    gettext(
                                        (e_invalid_format_specifier_str.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            }
                        } else {
                            any_arg = 1 as ::core::ffi::c_int;
                            if any_pos != 0 && any_arg != 0 {
                                semsg(
                                    gettext(
                                        (e_cannot_mix_positional_and_non_positional_str.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            }
                        }
                    } else if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let mut digstart_0: *const ::core::ffi::c_char = p;
                        let mut uj_3: ::core::ffi::c_uint = 0;
                        if get_unsigned_int(digstart_0, &raw mut p, &raw mut uj_3, !tvs.is_null())
                            == FAIL
                        {
                            break '_error;
                        }
                        if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    (e_invalid_format_specifier_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        }
                    }
                }
                if pos_arg != -1 as ::core::ffi::c_int {
                    any_pos = 1 as ::core::ffi::c_int;
                    if any_pos != 0 && any_arg != 0 {
                        semsg(
                            gettext(
                                (e_cannot_mix_positional_and_non_positional_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            fmt,
                        );
                        break '_error;
                    } else {
                        ptype = p;
                    }
                }
                if *p as ::core::ffi::c_int == 'h' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == 'z' as ::core::ffi::c_int
                {
                    length_modifier = *p;
                    p = p.offset(1);
                    if length_modifier as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                }
                match *p as ::core::ffi::c_int {
                    105 | 42 | 100 | 117 | 111 | 68 | 85 | 79 | 120 | 88 | 98 | 66 | 99 | 115
                    | 83 | 112 | 102 | 70 | 101 | 69 | 103 | 71 => {
                        if pos_arg != -1 as ::core::ffi::c_int {
                            if adjust_types(ap_types, pos_arg, num_posarg, ptype) == FAIL {
                                break '_error;
                            }
                        } else {
                            any_arg = 1 as ::core::ffi::c_int;
                            if any_pos != 0 && any_arg != 0 {
                                semsg(
                                    gettext(
                                        (e_cannot_mix_positional_and_non_positional_str.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            }
                        }
                    }
                    _ => {
                        if pos_arg != -1 as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    (e_cannot_mix_positional_and_non_positional_str.ptr()
                                        as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        }
                    }
                }
                if *p as ::core::ffi::c_int != NUL {
                    p = p.offset(1);
                }
            }
        }
        let mut arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while arg_idx < *num_posarg {
            if (*(*ap_types).offset(arg_idx as isize)).is_null() {
                semsg(
                    gettext(
                        (e_fmt_arg_nr_unused_str.ptr() as *const _) as *const ::core::ffi::c_char,
                    ),
                    arg_idx + 1 as ::core::ffi::c_int,
                    fmt,
                );
                break '_error;
            } else if !tvs.is_null()
                && (*tvs.offset(arg_idx as isize)).v_type as ::core::ffi::c_uint
                    == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                semsg(
                    gettext(
                        (e_positional_nr_out_of_bounds_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    arg_idx + 1 as ::core::ffi::c_int,
                    fmt,
                );
                break '_error;
            } else {
                arg_idx += 1;
            }
        }
        return OK;
    }
    xfree(*ap_types as *mut ::core::ffi::c_void);
    *ap_types = ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    *num_posarg = 0 as ::core::ffi::c_int;
    return FAIL;
}
// Hand-ported from neovim's static `skip_to_arg` in src/nvim/strings.c.
// c2rust drops this definition (it takes `va_list` by value, which its
// variadic support cannot translate) yet still emits the 17 call sites in
// `vim_vsnprintf_typval` below. This faithful port keeps the positional
// (`%N$`) printf path correct. The signature matches exactly what those call
// sites pass: `ap_start` is a fresh `va_copy` (`ap_start.clone()`) of the
// argument list's start, and `ap` is a pointer to the working `VaList`.
unsafe extern "C" fn skip_to_arg<'f>(
    ap_types: *mut *const ::core::ffi::c_char,
    ap_start: ::core::ffi::VaList<'f>,
    ap: *mut ::core::ffi::VaList<'f>,
    arg_idx: *mut ::core::ffi::c_int,
    arg_cur: *mut ::core::ffi::c_int,
    fmt: *const ::core::ffi::c_char,
) {
    let mut arg_min: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *arg_cur + 1 as ::core::ffi::c_int == *arg_idx {
        *arg_cur += 1;
        *arg_idx += 1;
        return;
    }
    if *arg_cur >= *arg_idx {
        // Reset ap to ap_start and skip arg_idx - 1 types (va_end + va_copy).
        *ap = ap_start.clone();
    } else {
        // Skip over any we should skip.
        arg_min = *arg_cur;
    }
    *arg_cur = arg_min;
    while *arg_cur < *arg_idx - 1 as ::core::ffi::c_int {
        if ap_types.is_null() || (*ap_types.offset(*arg_cur as isize)).is_null() {
            siemsg(
                (*e_aptypes_is_null_nr_str.ptr()).as_ptr() as *const ::core::ffi::c_char,
                fmt,
                *arg_cur,
            );
            return;
        }
        let p: *const ::core::ffi::c_char = *ap_types.offset(*arg_cur as isize);
        let fmt_type: ::core::ffi::c_int = format_typeof(p);
        // get parameter value, do initial processing (consume one va_arg)
        match fmt_type {
            TYPE_PERCENT | TYPE_UNKNOWN => {}
            TYPE_CHAR => {
                (*ap).next_arg::<::core::ffi::c_int>();
            }
            TYPE_STRING => {
                (*ap).next_arg::<*const ::core::ffi::c_char>();
            }
            TYPE_POINTER => {
                (*ap).next_arg::<*mut ::core::ffi::c_void>();
            }
            TYPE_INT => {
                (*ap).next_arg::<::core::ffi::c_int>();
            }
            TYPE_LONGINT => {
                (*ap).next_arg::<::core::ffi::c_long>();
            }
            TYPE_LONGLONGINT => {
                (*ap).next_arg::<::core::ffi::c_longlong>();
            }
            TYPE_SIGNEDSIZET => {
                // implementation-defined, usually ptrdiff_t
                (*ap).next_arg::<isize>();
            }
            TYPE_UNSIGNEDINT => {
                (*ap).next_arg::<::core::ffi::c_uint>();
            }
            TYPE_UNSIGNEDLONGINT => {
                (*ap).next_arg::<::core::ffi::c_ulong>();
            }
            TYPE_UNSIGNEDLONGLONGINT => {
                (*ap).next_arg::<::core::ffi::c_ulonglong>();
            }
            TYPE_SIZET => {
                (*ap).next_arg::<size_t>();
            }
            TYPE_FLOAT => {
                (*ap).next_arg::<::core::ffi::c_double>();
            }
            _ => {}
        }
        *arg_cur += 1;
    }
    // Because we know that after we return from this call, a va_arg() call is
    // made, we can pre-emptively increment the current argument index.
    *arg_cur += 1;
    *arg_idx += 1;
}
pub unsafe extern "C" fn vim_vsnprintf_typval(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut ap_start: ::core::ffi::VaList,
    tvs: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut str_l: size_t = 0 as size_t;
    let mut str_avail: bool = str_l < str_m;
    let mut p: *const ::core::ffi::c_char = fmt;
    let mut arg_cur: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut num_posarg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut arg_idx: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut ap: ::core::ffi::VaList;
    let mut ap_types: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    if parse_fmt_types(&raw mut ap_types, &raw mut num_posarg, fmt, tvs) == FAIL {
        return 0 as ::core::ffi::c_int;
    }
    ap = ap_start.clone();
    if p.is_null() {
        p = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    '_error: {
        while *p != 0 {
            if *p as ::core::ffi::c_int != '%' as ::core::ffi::c_int {
                let mut n: size_t = xstrchrnul(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    '%' as ::core::ffi::c_char,
                )
                .offset_from(p) as size_t;
                if str_avail {
                    let mut avail: size_t = str_m.wrapping_sub(str_l);
                    memmove(
                        str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        if n < avail { n } else { avail },
                    );
                    str_avail = n < avail;
                }
                p = p.offset(n as isize);
                '_c2rust_label: {
                    if n <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                    } else {
                        __assert_fail(
                            b"n <= SIZE_MAX - str_l\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/strings.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1486 as ::core::ffi::c_uint,
                            b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                str_l = str_l.wrapping_add(n);
            } else {
                let mut min_field_width: size_t = 0 as size_t;
                let mut precision: size_t = 0 as size_t;
                let mut zero_padding: bool = false_0 != 0;
                let mut precision_specified: bool = false_0 != 0;
                let mut justify_left: bool = false_0 != 0;
                let mut alternate_form: bool = false_0 != 0;
                let mut force_sign: bool = false_0 != 0;
                let mut space_for_positive: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut length_modifier: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                let mut tmp: [::core::ffi::c_char; 350] = [0; 350];
                let mut str_arg: *const ::core::ffi::c_char =
                    ::core::ptr::null::<::core::ffi::c_char>();
                let mut str_arg_l: size_t = 0;
                let mut uchar_arg: ::core::ffi::c_uchar = 0;
                let mut number_of_zeros_to_pad: size_t = 0 as size_t;
                let mut zero_padding_insertion_ind: size_t = 0 as size_t;
                let mut fmt_spec: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                let mut tofree: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut pos_arg: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                p = p.offset(1);
                let mut ptype: *const ::core::ffi::c_char = p;
                while ascii_isdigit(*ptype as ::core::ffi::c_int) {
                    ptype = ptype.offset(1);
                }
                if *ptype as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                    let mut digstart: *const ::core::ffi::c_char = p;
                    let mut uj: ::core::ffi::c_uint = 0;
                    if get_unsigned_int(digstart, &raw mut p, &raw mut uj, !tvs.is_null()) == FAIL {
                        break '_error;
                    }
                    pos_arg = uj as ::core::ffi::c_int;
                    p = p.offset(1);
                }
                loop {
                    match *p as ::core::ffi::c_int {
                        48 => {
                            zero_padding = true_0 != 0;
                            p = p.offset(1);
                        }
                        45 => {
                            justify_left = true_0 != 0;
                            p = p.offset(1);
                        }
                        43 => {
                            force_sign = true_0 != 0;
                            space_for_positive = 0 as ::core::ffi::c_int;
                            p = p.offset(1);
                        }
                        32 => {
                            force_sign = true_0 != 0;
                            p = p.offset(1);
                        }
                        35 => {
                            alternate_form = true_0 != 0;
                            p = p.offset(1);
                        }
                        39 => {
                            p = p.offset(1);
                        }
                        _ => {
                            break;
                        }
                    }
                }
                if *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                    let mut digstart_0: *const ::core::ffi::c_char =
                        p.offset(1 as ::core::ffi::c_int as isize);
                    p = p.offset(1);
                    if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let mut uj_0: ::core::ffi::c_uint = 0;
                        if get_unsigned_int(digstart_0, &raw mut p, &raw mut uj_0, !tvs.is_null())
                            == FAIL
                        {
                            break '_error;
                        }
                        arg_idx = uj_0 as ::core::ffi::c_int;
                        p = p.offset(1);
                    }
                    let mut j: ::core::ffi::c_int = if !tvs.is_null() {
                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                    } else {
                        skip_to_arg(
                            ap_types,
                            ap_start.clone(),
                            &raw mut ap,
                            &raw mut arg_idx,
                            &raw mut arg_cur,
                            fmt,
                        );
                        ap.next_arg::<::core::ffi::c_int>()
                    };
                    if j > MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int {
                        if !tvs.is_null() {
                            format_overflow_error(digstart_0);
                            break '_error;
                        } else {
                            j = MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int;
                        }
                    }
                    if j >= 0 as ::core::ffi::c_int {
                        min_field_width = j as size_t;
                    } else {
                        min_field_width = -j as size_t;
                        justify_left = true_0 != 0;
                    }
                } else if ascii_isdigit(*p as ::core::ffi::c_int) {
                    let mut digstart_1: *const ::core::ffi::c_char = p;
                    let mut uj_1: ::core::ffi::c_uint = 0;
                    if get_unsigned_int(digstart_1, &raw mut p, &raw mut uj_1, !tvs.is_null())
                        == FAIL
                    {
                        break '_error;
                    }
                    min_field_width = uj_1 as size_t;
                }
                if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                    p = p.offset(1);
                    precision_specified = true_0 != 0;
                    if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let mut digstart_2: *const ::core::ffi::c_char = p;
                        let mut uj_2: ::core::ffi::c_uint = 0;
                        if get_unsigned_int(digstart_2, &raw mut p, &raw mut uj_2, !tvs.is_null())
                            == FAIL
                        {
                            break '_error;
                        }
                        precision = uj_2 as size_t;
                    } else if *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                        let mut digstart_3: *const ::core::ffi::c_char = p;
                        p = p.offset(1);
                        if ascii_isdigit(*p as ::core::ffi::c_int) {
                            let mut uj_3: ::core::ffi::c_uint = 0;
                            if get_unsigned_int(
                                digstart_3,
                                &raw mut p,
                                &raw mut uj_3,
                                !tvs.is_null(),
                            ) == FAIL
                            {
                                break '_error;
                            }
                            arg_idx = uj_3 as ::core::ffi::c_int;
                            p = p.offset(1);
                        }
                        let mut j_0: ::core::ffi::c_int = if !tvs.is_null() {
                            tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                        } else {
                            skip_to_arg(
                                ap_types,
                                ap_start.clone(),
                                &raw mut ap,
                                &raw mut arg_idx,
                                &raw mut arg_cur,
                                fmt,
                            );
                            ap.next_arg::<::core::ffi::c_int>()
                        };
                        if j_0 > MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int {
                            if !tvs.is_null() {
                                format_overflow_error(digstart_3);
                                break '_error;
                            } else {
                                j_0 = MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_int;
                            }
                        }
                        if j_0 >= 0 as ::core::ffi::c_int {
                            precision = j_0 as size_t;
                        } else {
                            precision_specified = false_0 != 0;
                            precision = 0 as size_t;
                        }
                    }
                }
                if *p as ::core::ffi::c_int == 'h' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == 'z' as ::core::ffi::c_int
                {
                    length_modifier = *p;
                    p = p.offset(1);
                    if length_modifier as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    {
                        length_modifier = 'L' as ::core::ffi::c_char;
                        p = p.offset(1);
                    }
                }
                fmt_spec = *p;
                match fmt_spec as ::core::ffi::c_int {
                    105 => {
                        fmt_spec = 'd' as ::core::ffi::c_char;
                    }
                    68 => {
                        fmt_spec = 'd' as ::core::ffi::c_char;
                        length_modifier = 'l' as ::core::ffi::c_char;
                    }
                    85 => {
                        fmt_spec = 'u' as ::core::ffi::c_char;
                        length_modifier = 'l' as ::core::ffi::c_char;
                    }
                    79 => {
                        fmt_spec = 'o' as ::core::ffi::c_char;
                        length_modifier = 'l' as ::core::ffi::c_char;
                    }
                    _ => {}
                }
                match fmt_spec as ::core::ffi::c_int {
                    100 | 117 | 111 | 120 | 88 => {
                        if !tvs.is_null() && length_modifier as ::core::ffi::c_int == NUL {
                            length_modifier = 'L' as ::core::ffi::c_char;
                        }
                    }
                    _ => {}
                }
                if pos_arg != -1 as ::core::ffi::c_int {
                    arg_idx = pos_arg;
                }
                match fmt_spec as ::core::ffi::c_int {
                    37 | 99 | 115 | 83 => {
                        str_arg_l = 1 as size_t;
                        match fmt_spec as ::core::ffi::c_int {
                            37 => {
                                str_arg = p;
                            }
                            99 => {
                                let j_1: ::core::ffi::c_int = if !tvs.is_null() {
                                    tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                                } else {
                                    skip_to_arg(
                                        ap_types,
                                        ap_start.clone(),
                                        &raw mut ap,
                                        &raw mut arg_idx,
                                        &raw mut arg_cur,
                                        fmt,
                                    );
                                    ap.next_arg::<::core::ffi::c_int>()
                                };
                                uchar_arg = j_1 as ::core::ffi::c_uchar;
                                str_arg = &raw mut uchar_arg as *mut ::core::ffi::c_char;
                            }
                            115 | 83 => {
                                str_arg = if !tvs.is_null() {
                                    tv_str(tvs, &raw mut arg_idx, &raw mut tofree)
                                } else {
                                    skip_to_arg(
                                        ap_types,
                                        ap_start.clone(),
                                        &raw mut ap,
                                        &raw mut arg_idx,
                                        &raw mut arg_cur,
                                        fmt,
                                    );
                                    ap.next_arg::<*const ::core::ffi::c_char>()
                                };
                                if str_arg.is_null() {
                                    str_arg = b"[NULL]\0".as_ptr() as *const ::core::ffi::c_char;
                                    str_arg_l = 6 as size_t;
                                } else if !precision_specified {
                                    str_arg_l = strlen(str_arg);
                                } else if precision == 0 as size_t {
                                    str_arg_l = 0 as size_t;
                                } else {
                                    str_arg_l = (xmemscan(
                                        str_arg as *const ::core::ffi::c_void,
                                        NUL as ::core::ffi::c_char,
                                        if precision < 0x7fffffff as ::core::ffi::c_int as size_t {
                                            precision
                                        } else {
                                            0x7fffffff as ::core::ffi::c_int as size_t
                                        },
                                    )
                                        as *mut ::core::ffi::c_char)
                                        .offset_from(str_arg)
                                        as size_t;
                                }
                                if fmt_spec as ::core::ffi::c_int == 'S' as ::core::ffi::c_int {
                                    let mut p1: *const ::core::ffi::c_char =
                                        ::core::ptr::null::<::core::ffi::c_char>();
                                    let mut i: size_t = 0;
                                    i = 0 as size_t;
                                    p1 = str_arg;
                                    while *p1 != 0 {
                                        let mut cell: size_t = utf_ptr2cells(p1) as size_t;
                                        if precision_specified as ::core::ffi::c_int != 0
                                            && i.wrapping_add(cell) > precision
                                        {
                                            break;
                                        }
                                        i = i.wrapping_add(cell);
                                        p1 = p1.offset(utfc_ptr2len(p1) as isize);
                                    }
                                    str_arg_l = p1.offset_from(str_arg) as size_t;
                                    if min_field_width != 0 as size_t {
                                        min_field_width =
                                            min_field_width.wrapping_add(str_arg_l.wrapping_sub(i));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    100 | 117 | 98 | 66 | 111 | 120 | 88 | 112 => {
                        let mut arg_sign: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut arg: intmax_t = 0 as intmax_t;
                        let mut uarg: uintmax_t = 0 as uintmax_t;
                        let mut ptr_arg: *const ::core::ffi::c_void =
                            ::core::ptr::null::<::core::ffi::c_void>();
                        if fmt_spec as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
                            ptr_arg = if !tvs.is_null() {
                                tv_ptr(tvs, &raw mut arg_idx)
                            } else {
                                skip_to_arg(
                                    ap_types,
                                    ap_start.clone(),
                                    &raw mut ap,
                                    &raw mut arg_idx,
                                    &raw mut arg_cur,
                                    fmt,
                                );
                                ap.next_arg::<*mut ::core::ffi::c_void>()
                                    as *const ::core::ffi::c_void
                            };
                            if !ptr_arg.is_null() {
                                arg_sign = 1 as ::core::ffi::c_int;
                            }
                        } else if fmt_spec as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                            || fmt_spec as ::core::ffi::c_int == 'B' as ::core::ffi::c_int
                        {
                            uarg = (if !tvs.is_null() {
                                tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_ulonglong
                            } else {
                                skip_to_arg(
                                    ap_types,
                                    ap_start.clone(),
                                    &raw mut ap,
                                    &raw mut arg_idx,
                                    &raw mut arg_cur,
                                    fmt,
                                );
                                ap.next_arg::<::core::ffi::c_ulonglong>()
                            }) as uintmax_t;
                            arg_sign = (uarg != 0 as uintmax_t) as ::core::ffi::c_int;
                        } else if fmt_spec as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                            match length_modifier as ::core::ffi::c_int {
                                NUL => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_int>()
                                    }) as intmax_t;
                                }
                                104 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_int>()
                                    }) as int16_t
                                        as intmax_t;
                                }
                                108 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_long
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_long>()
                                    }) as intmax_t;
                                }
                                76 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_longlong
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_longlong>()
                                    }) as intmax_t;
                                }
                                122 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ptrdiff_t
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<ptrdiff_t>()
                                    }) as intmax_t;
                                }
                                _ => {}
                            }
                            if arg > 0 as intmax_t {
                                arg_sign = 1 as ::core::ffi::c_int;
                            } else if arg < 0 as intmax_t {
                                arg_sign = -1 as ::core::ffi::c_int;
                            }
                        } else {
                            match length_modifier as ::core::ffi::c_int {
                                NUL => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_uint
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_uint>()
                                    }) as uintmax_t;
                                }
                                104 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_uint
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_uint>()
                                    }) as uint16_t
                                        as uintmax_t;
                                }
                                108 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_ulong
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_ulong>()
                                    }) as uintmax_t;
                                }
                                76 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_ulonglong
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_ulonglong>()
                                    }) as uintmax_t;
                                }
                                122 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as size_t
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<size_t>()
                                    }) as uintmax_t;
                                }
                                _ => {}
                            }
                            arg_sign = (uarg != 0 as uintmax_t) as ::core::ffi::c_int;
                        }
                        str_arg = &raw mut tmp as *mut ::core::ffi::c_char;
                        str_arg_l = 0 as size_t;
                        if precision_specified {
                            zero_padding = false_0 != 0;
                        }
                        if fmt_spec as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                            if force_sign as ::core::ffi::c_int != 0
                                && arg_sign >= 0 as ::core::ffi::c_int
                            {
                                let c2rust_fresh27 = str_arg_l;
                                str_arg_l = str_arg_l.wrapping_add(1);
                                tmp[c2rust_fresh27 as usize] = (if space_for_positive != 0 {
                                    ' ' as ::core::ffi::c_int
                                } else {
                                    '+' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                            }
                        } else if alternate_form {
                            if arg_sign != 0 as ::core::ffi::c_int
                                && (fmt_spec as ::core::ffi::c_int == 'x' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'X' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'B' as ::core::ffi::c_int)
                            {
                                let c2rust_fresh28 = str_arg_l;
                                str_arg_l = str_arg_l.wrapping_add(1);
                                tmp[c2rust_fresh28 as usize] = '0' as ::core::ffi::c_char;
                                let c2rust_fresh29 = str_arg_l;
                                str_arg_l = str_arg_l.wrapping_add(1);
                                tmp[c2rust_fresh29 as usize] = fmt_spec;
                            }
                        }
                        zero_padding_insertion_ind = str_arg_l;
                        if !precision_specified {
                            precision = 1 as size_t;
                        }
                        if !(precision == 0 as size_t && arg_sign == 0 as ::core::ffi::c_int) {
                            match fmt_spec as ::core::ffi::c_int {
                                112 => {
                                    str_arg_l = str_arg_l.wrapping_add(snprintf(
                                        (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(str_arg_l as isize),
                                        ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                            .wrapping_sub(str_arg_l),
                                        b"%p\0".as_ptr() as *const ::core::ffi::c_char,
                                        ptr_arg,
                                    )
                                        as size_t);
                                }
                                100 => {
                                    str_arg_l = str_arg_l.wrapping_add(snprintf(
                                        (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(str_arg_l as isize),
                                        ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                            .wrapping_sub(str_arg_l),
                                        b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                        arg,
                                    )
                                        as size_t);
                                }
                                98 | 66 => {
                                    let mut bits: size_t = 0 as size_t;
                                    bits = ::core::mem::size_of::<uintmax_t>()
                                        .wrapping_mul(8 as usize)
                                        as size_t;
                                    while bits > 0 as size_t {
                                        if uarg >> bits.wrapping_sub(1 as size_t) & 0x1 as uintmax_t
                                            != 0
                                        {
                                            break;
                                        }
                                        bits = bits.wrapping_sub(1);
                                    }
                                    while bits > 0 as size_t {
                                        bits = bits.wrapping_sub(1);
                                        let c2rust_fresh30 = str_arg_l;
                                        str_arg_l = str_arg_l.wrapping_add(1);
                                        tmp[c2rust_fresh30 as usize] =
                                            (if uarg >> bits & 0x1 as uintmax_t != 0 {
                                                '1' as ::core::ffi::c_int
                                            } else {
                                                '0' as ::core::ffi::c_int
                                            })
                                                as ::core::ffi::c_char;
                                    }
                                }
                                _ => {
                                    let mut f: [::core::ffi::c_char; 4] = ::core::mem::transmute::<
                                        [u8; 4],
                                        [::core::ffi::c_char; 4],
                                    >(
                                        *b"%lu\0"
                                    );
                                    f[::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                                        .wrapping_sub(1 as usize)
                                        .wrapping_sub(1 as usize)
                                        as usize] = fmt_spec;
                                    '_c2rust_label_0: {
                                        if ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(
                                            *b"lu\0",
                                        )
                                            [::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as usize)
                                                .wrapping_sub(1 as usize)
                                                as usize]
                                            as ::core::ffi::c_int
                                            == 'u' as ::core::ffi::c_int
                                        {
                                        } else {
                                            __assert_fail(
                                                b"PRIuMAX[sizeof(PRIuMAX) - 1 - 1] == 'u'\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/strings.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2001 as ::core::ffi::c_uint,
                                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    str_arg_l = str_arg_l.wrapping_add(snprintf(
                                        (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(str_arg_l as isize),
                                        ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                            .wrapping_sub(str_arg_l),
                                        &raw mut f as *mut ::core::ffi::c_char,
                                        uarg,
                                    )
                                        as size_t);
                                }
                            }
                            '_c2rust_label_1: {
                                if str_arg_l < ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                {
                                } else {
                                    __assert_fail(
                                        b"str_arg_l < sizeof(tmp)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/strings.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2008 as ::core::ffi::c_uint,
                                        b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            if zero_padding_insertion_ind < str_arg_l
                                && tmp[zero_padding_insertion_ind as usize] as ::core::ffi::c_int
                                    == '-' as ::core::ffi::c_int
                            {
                                zero_padding_insertion_ind =
                                    zero_padding_insertion_ind.wrapping_add(1);
                            }
                            if zero_padding_insertion_ind.wrapping_add(1 as size_t) < str_arg_l
                                && tmp[zero_padding_insertion_ind as usize] as ::core::ffi::c_int
                                    == '0' as ::core::ffi::c_int
                                && (tmp
                                    [zero_padding_insertion_ind.wrapping_add(1 as size_t) as usize]
                                    as ::core::ffi::c_int
                                    == 'x' as ::core::ffi::c_int
                                    || tmp[zero_padding_insertion_ind.wrapping_add(1 as size_t)
                                        as usize]
                                        as ::core::ffi::c_int
                                        == 'X' as ::core::ffi::c_int
                                    || tmp[zero_padding_insertion_ind.wrapping_add(1 as size_t)
                                        as usize]
                                        as ::core::ffi::c_int
                                        == 'b' as ::core::ffi::c_int
                                    || tmp[zero_padding_insertion_ind.wrapping_add(1 as size_t)
                                        as usize]
                                        as ::core::ffi::c_int
                                        == 'B' as ::core::ffi::c_int)
                            {
                                zero_padding_insertion_ind =
                                    zero_padding_insertion_ind.wrapping_add(2 as size_t);
                            }
                        }
                        let mut num_of_digits: size_t =
                            str_arg_l.wrapping_sub(zero_padding_insertion_ind);
                        if alternate_form as ::core::ffi::c_int != 0
                            && fmt_spec as ::core::ffi::c_int == 'o' as ::core::ffi::c_int
                            && !(zero_padding_insertion_ind < str_arg_l
                                && tmp[zero_padding_insertion_ind as usize] as ::core::ffi::c_int
                                    == '0' as ::core::ffi::c_int)
                        {
                            if !precision_specified
                                || precision < num_of_digits.wrapping_add(1 as size_t)
                            {
                                precision = num_of_digits.wrapping_add(1 as size_t);
                            }
                        }
                        if num_of_digits < precision {
                            number_of_zeros_to_pad = precision.wrapping_sub(num_of_digits);
                        }
                        if !justify_left && zero_padding as ::core::ffi::c_int != 0 {
                            let n_0: ::core::ffi::c_int = min_field_width
                                .wrapping_sub(str_arg_l.wrapping_add(number_of_zeros_to_pad))
                                as ::core::ffi::c_int;
                            if n_0 > 0 as ::core::ffi::c_int {
                                number_of_zeros_to_pad =
                                    number_of_zeros_to_pad.wrapping_add(n_0 as size_t);
                            }
                        }
                    }
                    102 | 70 | 101 | 69 | 103 | 71 => {
                        let mut format: [::core::ffi::c_char; 40] = [0; 40];
                        let mut remove_trailing_zeroes: bool = false_0 != 0;
                        let mut f_0: ::core::ffi::c_double = if !tvs.is_null() {
                            tv_float(tvs, &raw mut arg_idx)
                        } else {
                            skip_to_arg(
                                ap_types,
                                ap_start.clone(),
                                &raw mut ap,
                                &raw mut arg_idx,
                                &raw mut arg_cur,
                                fmt,
                            );
                            ap.next_arg::<::core::ffi::c_double>()
                        };
                        let mut abs_f: ::core::ffi::c_double =
                            if f_0 < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
                                -f_0
                            } else {
                                f_0
                            };
                        if fmt_spec as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                            || fmt_spec as ::core::ffi::c_int == 'G' as ::core::ffi::c_int
                        {
                            if abs_f >= 0.001f64 && abs_f < 10000000.0f64 || abs_f == 0.0f64 {
                                fmt_spec = (if fmt_spec as ::core::ffi::c_uint
                                    >= 'A' as ::core::ffi::c_uint
                                    && fmt_spec as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                {
                                    'F' as ::core::ffi::c_int
                                } else {
                                    'f' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                            } else {
                                fmt_spec =
                                    (if fmt_spec as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                                    {
                                        'e' as ::core::ffi::c_int
                                    } else {
                                        'E' as ::core::ffi::c_int
                                    }) as ::core::ffi::c_char;
                            }
                            remove_trailing_zeroes = true_0 != 0;
                        }
                        if f_0.is_infinite()
                            || !strchr(
                                b"fF\0".as_ptr() as *const ::core::ffi::c_char,
                                fmt_spec as ::core::ffi::c_int,
                            )
                            .is_null()
                                && abs_f > 1.0e307f64
                        {
                            xstrlcpy(
                                &raw mut tmp as *mut ::core::ffi::c_char,
                                infinity_str(
                                    f_0 > 0.0f64,
                                    fmt_spec,
                                    force_sign as ::core::ffi::c_int,
                                    space_for_positive,
                                ),
                                ::core::mem::size_of::<[::core::ffi::c_char; 350]>(),
                            );
                            str_arg_l = strlen(&raw mut tmp as *mut ::core::ffi::c_char);
                            zero_padding = false_0 != 0;
                        } else if f_0.is_nan() {
                            memmove(
                                &raw mut tmp as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (if fmt_spec as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                                    && fmt_spec as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                {
                                    b"NAN\0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"nan\0".as_ptr() as *const ::core::ffi::c_char
                                }) as *const ::core::ffi::c_void,
                                4 as size_t,
                            );
                            str_arg_l = 3 as size_t;
                            zero_padding = false_0 != 0;
                        } else {
                            format[0 as ::core::ffi::c_int as usize] = '%' as ::core::ffi::c_char;
                            let mut l: size_t = 1 as size_t;
                            if force_sign {
                                let c2rust_fresh31 = l;
                                l = l.wrapping_add(1);
                                format[c2rust_fresh31 as usize] = (if space_for_positive != 0 {
                                    ' ' as ::core::ffi::c_int
                                } else {
                                    '+' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                            }
                            if precision_specified {
                                let mut max_prec: size_t =
                                    (TMP_LEN - 10 as ::core::ffi::c_int) as size_t;
                                if (fmt_spec as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'F' as ::core::ffi::c_int)
                                    && abs_f > 1.0f64
                                {
                                    max_prec = max_prec.wrapping_sub(log10(abs_f) as size_t);
                                }
                                if precision > max_prec {
                                    precision = max_prec;
                                }
                                l = l.wrapping_add(snprintf(
                                    (&raw mut format as *mut ::core::ffi::c_char)
                                        .offset(l as isize),
                                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                                        .wrapping_sub(l),
                                    b".%d\0".as_ptr() as *const ::core::ffi::c_char,
                                    precision as ::core::ffi::c_int,
                                ) as size_t);
                            }
                            '_c2rust_label_2: {
                                if l.wrapping_add(1 as size_t)
                                    < ::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                                {
                                } else {
                                    __assert_fail(
                                        b"l + 1 < sizeof(format)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/strings.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2119 as ::core::ffi::c_uint,
                                        b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            format[l as usize] =
                                (if fmt_spec as ::core::ffi::c_int == 'F' as ::core::ffi::c_int {
                                    'f' as ::core::ffi::c_int
                                } else {
                                    fmt_spec as ::core::ffi::c_int
                                }) as ::core::ffi::c_char;
                            format[l.wrapping_add(1 as size_t) as usize] =
                                NUL as ::core::ffi::c_char;
                            str_arg_l = snprintf(
                                &raw mut tmp as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 350]>(),
                                &raw mut format as *mut ::core::ffi::c_char,
                                f_0,
                            ) as size_t;
                            '_c2rust_label_3: {
                                if str_arg_l < ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                {
                                } else {
                                    __assert_fail(
                                        b"str_arg_l < sizeof(tmp)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/strings.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2124 as ::core::ffi::c_uint,
                                        b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            if remove_trailing_zeroes {
                                let mut tp: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                if fmt_spec as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'F' as ::core::ffi::c_int
                                {
                                    tp = (&raw mut tmp as *mut ::core::ffi::c_char)
                                        .offset(str_arg_l as isize)
                                        .offset(-(1 as ::core::ffi::c_int as isize));
                                } else {
                                    tp = vim_strchr(
                                        &raw mut tmp as *mut ::core::ffi::c_char,
                                        if fmt_spec as ::core::ffi::c_int
                                            == 'e' as ::core::ffi::c_int
                                        {
                                            'e' as ::core::ffi::c_int
                                        } else {
                                            'E' as ::core::ffi::c_int
                                        },
                                    );
                                    if !tp.is_null() {
                                        if *tp.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '+' as ::core::ffi::c_int
                                        {
                                            memmove(
                                                tp.offset(1 as ::core::ffi::c_int as isize)
                                                    as *mut ::core::ffi::c_void,
                                                tp.offset(2 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(tp.offset(2 as ::core::ffi::c_int as isize))
                                                    .wrapping_add(1 as size_t),
                                            );
                                            str_arg_l = str_arg_l.wrapping_sub(1);
                                        }
                                        let mut i_0: ::core::ffi::c_int = if *tp
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '-' as ::core::ffi::c_int
                                        {
                                            2 as ::core::ffi::c_int
                                        } else {
                                            1 as ::core::ffi::c_int
                                        };
                                        while *tp.offset(i_0 as isize) as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        {
                                            memmove(
                                                tp.offset(i_0 as isize) as *mut ::core::ffi::c_void,
                                                tp.offset(i_0 as isize)
                                                    .offset(1 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(
                                                    tp.offset(i_0 as isize)
                                                        .offset(1 as ::core::ffi::c_int as isize),
                                                )
                                                .wrapping_add(1 as size_t),
                                            );
                                            str_arg_l = str_arg_l.wrapping_sub(1);
                                        }
                                        tp = tp.offset(-1);
                                    }
                                }
                                if !tp.is_null() && !precision_specified {
                                    while tp
                                        > (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(2 as ::core::ffi::c_int as isize)
                                        && *tp as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                                        && *tp.offset(-1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            != '.' as ::core::ffi::c_int
                                    {
                                        memmove(
                                            tp as *mut ::core::ffi::c_void,
                                            tp.offset(1 as ::core::ffi::c_int as isize)
                                                as *const ::core::ffi::c_void,
                                            strlen(tp.offset(1 as ::core::ffi::c_int as isize))
                                                .wrapping_add(1 as size_t),
                                        );
                                        tp = tp.offset(-1);
                                        str_arg_l = str_arg_l.wrapping_sub(1);
                                    }
                                }
                            } else {
                                let mut tp_0: *mut ::core::ffi::c_char = vim_strchr(
                                    &raw mut tmp as *mut ::core::ffi::c_char,
                                    if fmt_spec as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                                        'e' as ::core::ffi::c_int
                                    } else {
                                        'E' as ::core::ffi::c_int
                                    },
                                );
                                if !tp_0.is_null()
                                    && (*tp_0.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '+' as ::core::ffi::c_int
                                        || *tp_0.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '-' as ::core::ffi::c_int)
                                    && *tp_0.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '0' as ::core::ffi::c_int
                                    && ascii_isdigit(*tp_0.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                    && ascii_isdigit(*tp_0.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    memmove(
                                        tp_0.offset(2 as ::core::ffi::c_int as isize)
                                            as *mut ::core::ffi::c_void,
                                        tp_0.offset(3 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        strlen(tp_0.offset(3 as ::core::ffi::c_int as isize))
                                            .wrapping_add(1 as size_t),
                                    );
                                    str_arg_l = str_arg_l.wrapping_sub(1);
                                }
                            }
                        }
                        if zero_padding as ::core::ffi::c_int != 0
                            && min_field_width > str_arg_l
                            && (tmp[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                                == '-' as ::core::ffi::c_int
                                || force_sign as ::core::ffi::c_int != 0)
                        {
                            number_of_zeros_to_pad = min_field_width.wrapping_sub(str_arg_l);
                            zero_padding_insertion_ind = 1 as size_t;
                        }
                        str_arg = &raw mut tmp as *mut ::core::ffi::c_char;
                    }
                    _ => {
                        zero_padding = false_0 != 0;
                        justify_left = true_0 != 0;
                        min_field_width = 0 as size_t;
                        str_arg = p;
                        str_arg_l = 0 as size_t;
                        if *p != 0 {
                            str_arg_l = str_arg_l.wrapping_add(1);
                        }
                    }
                }
                if *p != 0 {
                    p = p.offset(1);
                }
                if !justify_left {
                    '_c2rust_label_4: {
                        if str_arg_l
                            <= (18446744073709551615 as size_t).wrapping_sub(number_of_zeros_to_pad)
                        {
                        } else {
                            __assert_fail(
                                b"str_arg_l <= SIZE_MAX - number_of_zeros_to_pad\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2204 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if min_field_width > str_arg_l.wrapping_add(number_of_zeros_to_pad) {
                        let mut pn: size_t = min_field_width
                            .wrapping_sub(str_arg_l.wrapping_add(number_of_zeros_to_pad));
                        if str_avail {
                            let mut avail_0: size_t = str_m.wrapping_sub(str_l);
                            memset(
                                str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                                if zero_padding as ::core::ffi::c_int != 0 {
                                    '0' as ::core::ffi::c_int
                                } else {
                                    ' ' as ::core::ffi::c_int
                                },
                                if pn < avail_0 { pn } else { avail_0 },
                            );
                            str_avail = pn < avail_0;
                        }
                        '_c2rust_label_5: {
                            if pn <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                            } else {
                                __assert_fail(
                                    b"pn <= SIZE_MAX - str_l\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/strings.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2213 as ::core::ffi::c_uint,
                                    b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        str_l = str_l.wrapping_add(pn);
                    }
                }
                if number_of_zeros_to_pad == 0 as size_t {
                    zero_padding_insertion_ind = 0 as size_t;
                } else {
                    if zero_padding_insertion_ind > 0 as size_t {
                        let mut zn: size_t = zero_padding_insertion_ind;
                        if str_avail {
                            let mut avail_1: size_t = str_m.wrapping_sub(str_l);
                            memmove(
                                str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                                str_arg as *const ::core::ffi::c_void,
                                if zn < avail_1 { zn } else { avail_1 },
                            );
                            str_avail = zn < avail_1;
                        }
                        '_c2rust_label_6: {
                            if zn <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                            } else {
                                __assert_fail(
                                    b"zn <= SIZE_MAX - str_l\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/strings.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2233 as ::core::ffi::c_uint,
                                    b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        str_l = str_l.wrapping_add(zn);
                    }
                    let mut zn_0: size_t = number_of_zeros_to_pad;
                    if str_avail {
                        let mut avail_2: size_t = str_m.wrapping_sub(str_l);
                        memset(
                            str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                            '0' as ::core::ffi::c_int,
                            if zn_0 < avail_2 { zn_0 } else { avail_2 },
                        );
                        str_avail = zn_0 < avail_2;
                    }
                    '_c2rust_label_7: {
                        if zn_0 <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                        } else {
                            __assert_fail(
                                b"zn <= SIZE_MAX - str_l\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2244 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    str_l = str_l.wrapping_add(zn_0);
                }
                if str_arg_l > zero_padding_insertion_ind {
                    let mut sn: size_t = str_arg_l.wrapping_sub(zero_padding_insertion_ind);
                    if str_avail {
                        let mut avail_3: size_t = str_m.wrapping_sub(str_l);
                        memmove(
                            str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                            str_arg.offset(zero_padding_insertion_ind as isize)
                                as *const ::core::ffi::c_void,
                            if sn < avail_3 { sn } else { avail_3 },
                        );
                        str_avail = sn < avail_3;
                    }
                    '_c2rust_label_8: {
                        if sn <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                        } else {
                            __assert_fail(
                                b"sn <= SIZE_MAX - str_l\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2259 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    str_l = str_l.wrapping_add(sn);
                }
                if justify_left {
                    '_c2rust_label_9: {
                        if str_arg_l
                            <= (18446744073709551615 as size_t).wrapping_sub(number_of_zeros_to_pad)
                        {
                        } else {
                            __assert_fail(
                                b"str_arg_l <= SIZE_MAX - number_of_zeros_to_pad\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2265 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if min_field_width > str_arg_l.wrapping_add(number_of_zeros_to_pad) {
                        let mut pn_0: size_t = min_field_width
                            .wrapping_sub(str_arg_l.wrapping_add(number_of_zeros_to_pad));
                        if str_avail {
                            let mut avail_4: size_t = str_m.wrapping_sub(str_l);
                            memset(
                                str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                if pn_0 < avail_4 { pn_0 } else { avail_4 },
                            );
                            str_avail = pn_0 < avail_4;
                        }
                        '_c2rust_label_10: {
                            if pn_0 <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                            } else {
                                __assert_fail(
                                    b"pn <= SIZE_MAX - str_l\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/strings.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2274 as ::core::ffi::c_uint,
                                    b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        str_l = str_l.wrapping_add(pn_0);
                    }
                }
                xfree(tofree as *mut ::core::ffi::c_void);
            }
        }
        if str_m > 0 as size_t {
            *str.offset(
                (if str_l <= str_m.wrapping_sub(1 as size_t) {
                    str_l
                } else {
                    str_m.wrapping_sub(1 as size_t)
                }) as isize,
            ) = NUL as ::core::ffi::c_char;
        }
        if !tvs.is_null()
            && (*tvs.offset(
                (if num_posarg != 0 as ::core::ffi::c_int {
                    num_posarg
                } else {
                    arg_idx - 1 as ::core::ffi::c_int
                }) as isize,
            ))
            .v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(
                b"E767: Too many arguments to printf()\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    xfree(ap_types as *mut ::core::ffi::c_void);
    return str_l as ::core::ffi::c_int;
}
pub const TMP_LEN: ::core::ffi::c_int = 350 as ::core::ffi::c_int;
pub unsafe extern "C" fn kv_do_printf(
    mut str: *mut StringBuilder,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let mut remaining: size_t = (*str).capacity.wrapping_sub((*str).size);
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    let mut printed: ::core::ffi::c_int = vsnprintf(
        if !(*str).items.is_null() {
            (*str).items.offset((*str).size as isize)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        },
        remaining,
        fmt,
        ap,
    );
    if printed < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if printed as size_t >= remaining {
        if (*str).capacity
            < (*str)
                .size
                .wrapping_add(printed as size_t)
                .wrapping_add(1 as size_t)
        {
            (*str).capacity = (*str)
                .size
                .wrapping_add(printed as size_t)
                .wrapping_add(1 as size_t);
            (*str).capacity = (*str).capacity.wrapping_sub(1);
            (*str).capacity |= (*str).capacity >> 1 as ::core::ffi::c_int;
            (*str).capacity |= (*str).capacity >> 2 as ::core::ffi::c_int;
            (*str).capacity |= (*str).capacity >> 4 as ::core::ffi::c_int;
            (*str).capacity |= (*str).capacity >> 8 as ::core::ffi::c_int;
            (*str).capacity |= (*str).capacity >> 16 as ::core::ffi::c_int;
            (*str).capacity = (*str).capacity.wrapping_add(1);
            (*str).capacity = (*str).capacity;
            (*str).items = xrealloc(
                (*str).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*str).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*str).items.is_null() {
            } else {
                __assert_fail(
                    b"str->items != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/strings.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2321 as ::core::ffi::c_uint,
                    b"int kv_do_printf(StringBuilder *, const char *, ...)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        ap = c2rust_args.clone();
        printed = vsnprintf(
            (*str).items.offset((*str).size as isize),
            (*str).capacity.wrapping_sub((*str).size),
            fmt,
            ap,
        );
        if printed < 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
    }
    (*str).size = (*str).size.wrapping_add(printed as size_t);
    return printed;
}
pub unsafe extern "C" fn arena_printf(
    mut arena: *mut Arena,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> String_0 {
    let mut remaining: size_t = 0 as size_t;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !arena.is_null() {
        if (*arena).cur_blk.is_null() {
            arena_alloc_block(arena);
        }
        remaining = (*arena).size.wrapping_sub((*arena).pos);
        buf = (*arena).cur_blk.offset((*arena).pos as isize);
    }
    let mut ap: ::core::ffi::VaList;
    ap = c2rust_args.clone();
    let mut printed: ::core::ffi::c_int = vsnprintf(buf, remaining, fmt, ap);
    if printed < 0 as ::core::ffi::c_int {
        return String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
    }
    if printed as size_t >= remaining {
        buf = arena_alloc(
            arena,
            (printed as size_t).wrapping_add(1 as size_t),
            false_0 != 0,
        ) as *mut ::core::ffi::c_char;
        ap = c2rust_args.clone();
        printed = vsnprintf(buf, (printed as size_t).wrapping_add(1 as size_t), fmt, ap);
        if printed < 0 as ::core::ffi::c_int {
            return String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            };
        }
    } else {
        (*arena).pos = (*arena)
            .pos
            .wrapping_add((printed as size_t).wrapping_add(1 as size_t));
    }
    return String_0 {
        data: buf,
        size: printed as size_t,
    };
}
#[no_mangle]
pub unsafe extern "C" fn reverse_text(mut s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(s);
    let mut rev: *mut ::core::ffi::c_char =
        xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut s_i: size_t = 0 as size_t;
    let mut rev_i: size_t = len;
    while s_i < len {
        let mb_len: ::core::ffi::c_int = utfc_ptr2len(s.offset(s_i as isize));
        rev_i = rev_i.wrapping_sub(mb_len as size_t);
        memmove(
            rev.offset(rev_i as isize) as *mut ::core::ffi::c_void,
            s.offset(s_i as isize) as *const ::core::ffi::c_void,
            mb_len as size_t,
        );
        s_i = s_i.wrapping_add((mb_len as size_t).wrapping_sub(1 as size_t));
        s_i = s_i.wrapping_add(1);
    }
    *rev.offset(len as isize) = NUL as ::core::ffi::c_char;
    return rev;
}
pub unsafe extern "C" fn strrep(
    mut src: *const ::core::ffi::c_char,
    mut what: *const ::core::ffi::c_char,
    mut rep: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut pos: *const ::core::ffi::c_char = src;
    let mut whatlen: size_t = strlen(what);
    let mut count: size_t = 0 as size_t;
    loop {
        pos = strstr(pos, what);
        if pos.is_null() {
            break;
        }
        count = count.wrapping_add(1);
        pos = pos.offset(whatlen as isize);
    }
    if count == 0 as size_t {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut replen: size_t = strlen(rep);
    let mut ret: *mut ::core::ffi::c_char = xmalloc(
        strlen(src)
            .wrapping_add(count.wrapping_mul(replen.wrapping_sub(whatlen)))
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    let mut ptr: *mut ::core::ffi::c_char = ret;
    loop {
        pos = strstr(src, what);
        if pos.is_null() {
            break;
        }
        let mut idx: size_t = pos.offset_from(src) as size_t;
        memcpy(
            ptr as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            idx,
        );
        ptr = ptr.offset(idx as isize);
        strcpy(ptr, rep as *mut ::core::ffi::c_char);
        ptr = ptr.offset(replen as isize);
        src = pos.offset(whatlen as isize);
    }
    strcpy(ptr, src as *mut ::core::ffi::c_char);
    return ret;
}
unsafe extern "C" fn byteidx_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut comp: bool,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut idx: varnumber_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
    if str.is_null() || idx < 0 as varnumber_T {
        return;
    }
    let mut utf16idx: varnumber_T = false_0 as varnumber_T;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        utf16idx = tv_get_bool_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            return;
        }
        if utf16idx < 0 as varnumber_T || utf16idx > 1 as varnumber_T {
            semsg(
                gettext(&raw const e_using_number_as_bool_nr as *const ::core::ffi::c_char),
                utf16idx,
            );
            return;
        }
    }
    let mut ptr2len: Option<
        unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
    > = None;
    if comp {
        ptr2len = Some(
            utf_ptr2len as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
            as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    } else {
        ptr2len = Some(
            utfc_ptr2len as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
            as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    }
    let mut t: *const ::core::ffi::c_char = str;
    while idx > 0 as varnumber_T {
        if *t as ::core::ffi::c_int == NUL {
            return;
        }
        if utf16idx != 0 {
            let clen: ::core::ffi::c_int = ptr2len.expect("non-null function pointer")(t);
            let c: ::core::ffi::c_int = if clen > 1 as ::core::ffi::c_int {
                utf_ptr2char(t)
            } else {
                *t as ::core::ffi::c_int
            };
            if c > 0xffff as ::core::ffi::c_int {
                idx -= 1;
            }
            if idx > 0 as varnumber_T {
                t = t.offset(clen as isize);
            }
        } else if idx > 0 as varnumber_T {
            t = t.offset(ptr2len.expect("non-null function pointer")(t) as isize);
        }
        idx -= 1;
    }
    (*rettv).vval.v_number = t.offset_from(str) as varnumber_T;
}
pub unsafe extern "C" fn f_byteidx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    byteidx_common(argvars, rettv, false_0 != 0);
}
pub unsafe extern "C" fn f_byteidxcomp(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    byteidx_common(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_charidx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_bool_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_check_for_opt_bool_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut idx: varnumber_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
    if str.is_null() || idx < 0 as varnumber_T {
        return;
    }
    let mut countcc: varnumber_T = false_0 as varnumber_T;
    let mut utf16idx: varnumber_T = false_0 as varnumber_T;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        countcc = tv_get_bool(argvars.offset(2 as ::core::ffi::c_int as isize));
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            utf16idx = tv_get_bool(argvars.offset(3 as ::core::ffi::c_int as isize));
        }
    }
    let mut ptr2len: Option<
        unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
    > = None;
    if countcc != 0 {
        ptr2len = Some(
            utf_ptr2len as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
            as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    } else {
        ptr2len = Some(
            utfc_ptr2len as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
            as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    }
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    p = str;
    len = 0 as ::core::ffi::c_int;
    while if utf16idx != 0 {
        (idx >= 0 as varnumber_T) as ::core::ffi::c_int
    } else {
        (p <= str.offset(idx as isize)) as ::core::ffi::c_int
    } != 0
    {
        if *p as ::core::ffi::c_int == NUL {
            if if utf16idx != 0 {
                (idx == 0 as varnumber_T) as ::core::ffi::c_int
            } else {
                (p == str.offset(idx as isize)) as ::core::ffi::c_int
            } != 0
            {
                (*rettv).vval.v_number = len as varnumber_T;
            }
            return;
        }
        if utf16idx != 0 {
            idx -= 1;
            let clen: ::core::ffi::c_int = ptr2len.expect("non-null function pointer")(p);
            let c: ::core::ffi::c_int = if clen > 1 as ::core::ffi::c_int {
                utf_ptr2char(p)
            } else {
                *p as ::core::ffi::c_int
            };
            if c > 0xffff as ::core::ffi::c_int {
                idx -= 1;
            }
        }
        p = p.offset(ptr2len.expect("non-null function pointer")(p) as isize);
        len += 1;
    }
    (*rettv).vval.v_number = (if len > 0 as ::core::ffi::c_int {
        len - 1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as varnumber_T;
}
pub unsafe extern "C" fn f_str2list(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    while *p as ::core::ffi::c_int != NUL {
        tv_list_append_number((*rettv).vval.v_list, utf_ptr2char(p) as varnumber_T);
        p = p.offset(utf_ptr2len(p) as isize);
    }
}
pub unsafe extern "C" fn f_str2nr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut base: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        base =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if base != 2 as ::core::ffi::c_int
            && base != 8 as ::core::ffi::c_int
            && base != 10 as ::core::ffi::c_int
            && base != 16 as ::core::ffi::c_int
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_get_bool(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0
        {
            what |= STR2NR_QUOTE as ::core::ffi::c_int;
        }
    }
    let mut p: *mut ::core::ffi::c_char = skipwhite(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    let mut isneg: bool = *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
    }
    match base {
        2 => {
            what |= STR2NR_BIN as ::core::ffi::c_int | STR2NR_FORCE as ::core::ffi::c_int;
        }
        8 => {
            what |= STR2NR_OCT as ::core::ffi::c_int
                | STR2NR_OOCT as ::core::ffi::c_int
                | STR2NR_FORCE as ::core::ffi::c_int;
        }
        16 => {
            what |= STR2NR_HEX as ::core::ffi::c_int | STR2NR_FORCE as ::core::ffi::c_int;
        }
        _ => {}
    }
    let mut n: varnumber_T = 0;
    vim_str2nr(
        p,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        what,
        &raw mut n,
        ::core::ptr::null_mut::<uvarnumber_T>(),
        0 as ::core::ffi::c_int,
        false_0 != 0,
        ::core::ptr::null_mut::<bool>(),
    );
    if isneg {
        (*rettv).vval.v_number = -n;
    } else {
        (*rettv).vval.v_number = n;
    };
}
pub unsafe extern "C" fn f_strgetchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if str.is_null() {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut charidx: varnumber_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if error {
        return;
    }
    let len: size_t = strlen(str);
    let mut byteidx: size_t = 0 as size_t;
    while charidx >= 0 as varnumber_T && byteidx < len {
        if charidx == 0 as varnumber_T {
            (*rettv).vval.v_number = utf_ptr2char(str.offset(byteidx as isize)) as varnumber_T;
            break;
        } else {
            charidx -= 1;
            byteidx = byteidx.wrapping_add(utf_ptr2len(str.offset(byteidx as isize)) as size_t);
        }
    }
}
pub unsafe extern "C" fn f_stridx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let needle: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut haystack: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    let haystack_start: *const ::core::ffi::c_char = haystack;
    if needle.is_null() || haystack.is_null() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        let start_idx: ptrdiff_t = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ptrdiff_t;
        if error as ::core::ffi::c_int != 0 || start_idx >= strlen(haystack) as ptrdiff_t {
            return;
        }
        if start_idx >= 0 as ptrdiff_t {
            haystack = haystack.offset(start_idx as isize);
        }
    }
    let mut pos: *const ::core::ffi::c_char = strstr(haystack, needle);
    if !pos.is_null() {
        (*rettv).vval.v_number = pos.offset_from(haystack_start) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_string(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = encode_tv2string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<size_t>(),
    );
}
pub unsafe extern "C" fn f_strlen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = strlen(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
unsafe extern "C" fn strchar_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut skipcc: bool,
) {
    let mut s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut len: varnumber_T = 0 as varnumber_T;
    let mut func_mb_ptr2char_adv: Option<
        unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
    > = None;
    func_mb_ptr2char_adv = (if skipcc as ::core::ffi::c_int != 0 {
        Some(
            mb_ptr2char_adv
                as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
    } else {
        Some(
            mb_cptr2char_adv
                as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
    })
        as Option<unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    while *s as ::core::ffi::c_int != NUL {
        func_mb_ptr2char_adv.expect("non-null function pointer")(&raw mut s);
        len += 1;
    }
    (*rettv).vval.v_number = len;
}
pub unsafe extern "C" fn f_strcharlen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    strchar_common(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_strchars(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut skipcc: varnumber_T = false_0 as varnumber_T;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        skipcc = tv_get_bool_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            return;
        }
        if skipcc < 0 as varnumber_T || skipcc > 1 as varnumber_T {
            semsg(
                gettext(&raw const e_using_number_as_bool_nr as *const ::core::ffi::c_char),
                skipcc,
            );
            return;
        }
    }
    strchar_common(argvars, rettv, skipcc != 0);
}
pub unsafe extern "C" fn f_strutf16len(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_bool_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut countcc: varnumber_T = false_0 as varnumber_T;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        countcc = tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    let mut s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut len: varnumber_T = 0 as varnumber_T;
    let mut func_mb_ptr2char_adv: Option<
        unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
    > = None;
    func_mb_ptr2char_adv = (if countcc != 0 {
        Some(
            mb_cptr2char_adv
                as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
    } else {
        Some(
            mb_ptr2char_adv
                as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
    })
        as Option<unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    while *s as ::core::ffi::c_int != NUL {
        let ch: ::core::ffi::c_int =
            func_mb_ptr2char_adv.expect("non-null function pointer")(&raw mut s);
        if ch > 0xffff as ::core::ffi::c_int {
            len += 1;
        }
        len += 1;
    }
    (*rettv).vval.v_number = len;
}
pub unsafe extern "C" fn f_strdisplaywidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        col = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number =
        (linetabsize_col(col, s as *mut ::core::ffi::c_char) - col) as varnumber_T;
}
pub unsafe extern "C" fn f_strwidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = mb_string2cells(s) as varnumber_T;
}
pub unsafe extern "C" fn f_strcharpart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let slen: size_t = strlen(p);
    let mut nbyte: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skipcc: varnumber_T = false_0 as varnumber_T;
    let mut error: bool = false_0 != 0;
    let mut nchar: varnumber_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if !error {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            skipcc = tv_get_bool_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
            if error {
                return;
            }
            if skipcc < 0 as varnumber_T || skipcc > 1 as varnumber_T {
                semsg(
                    gettext(&raw const e_using_number_as_bool_nr as *const ::core::ffi::c_char),
                    skipcc,
                );
                return;
            }
        }
        if nchar > 0 as varnumber_T {
            while nchar > 0 as varnumber_T && (nbyte as size_t) < slen {
                if skipcc != 0 {
                    nbyte += utfc_ptr2len(p.offset(nbyte as isize));
                } else {
                    nbyte += utf_ptr2len(p.offset(nbyte as isize));
                }
                nchar -= 1;
            }
        } else {
            nbyte = nchar as ::core::ffi::c_int;
        }
    }
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut charlen: ::core::ffi::c_int =
            tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        while charlen > 0 as ::core::ffi::c_int && nbyte + len < slen as ::core::ffi::c_int {
            let mut off: ::core::ffi::c_int = nbyte + len;
            if off < 0 as ::core::ffi::c_int {
                len += 1 as ::core::ffi::c_int;
            } else if skipcc != 0 {
                len += utfc_ptr2len(p.offset(off as isize));
            } else {
                len += utf_ptr2len(p.offset(off as isize));
            }
            charlen -= 1;
        }
    } else {
        len = slen as ::core::ffi::c_int - nbyte;
    }
    if nbyte < 0 as ::core::ffi::c_int {
        len += nbyte;
        nbyte = 0 as ::core::ffi::c_int;
    } else if nbyte as size_t > slen {
        nbyte = slen as ::core::ffi::c_int;
    }
    if len < 0 as ::core::ffi::c_int {
        len = 0 as ::core::ffi::c_int;
    } else if nbyte + len > slen as ::core::ffi::c_int {
        len = slen as ::core::ffi::c_int - nbyte;
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xmemdupz(
        p.offset(nbyte as isize) as *const ::core::ffi::c_void,
        len as size_t,
    ) as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn f_strpart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut error: bool = false_0 != 0;
    let p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let slen: size_t = strlen(p);
    let mut n: varnumber_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    let mut len: varnumber_T = 0;
    if error {
        len = 0 as varnumber_T;
    } else if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        len = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize));
    } else {
        len = slen as varnumber_T - n;
    }
    if n < 0 as varnumber_T {
        len += n;
        n = 0 as varnumber_T;
    } else if n > slen as varnumber_T {
        n = slen as varnumber_T;
    }
    if len < 0 as varnumber_T {
        len = 0 as varnumber_T;
    } else if n + len > slen as varnumber_T {
        len = slen as varnumber_T - n;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut off: int64_t = 0;
        off = n as int64_t;
        while off < slen as int64_t && len > 0 as varnumber_T {
            off += utfc_ptr2len(p.offset(off as isize)) as int64_t;
            len -= 1;
        }
        len = (off - n as int64_t) as varnumber_T;
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xmemdupz(
        p.offset(n as isize) as *const ::core::ffi::c_void,
        len as size_t,
    ) as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn f_strridx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let needle: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    let haystack: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    (*rettv).vval.v_number = -1 as varnumber_T;
    if needle.is_null() || haystack.is_null() {
        return;
    }
    let haystack_len: size_t = strlen(haystack);
    let mut end_idx: ptrdiff_t = 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end_idx = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ptrdiff_t;
        if end_idx < 0 as ptrdiff_t {
            return;
        }
    } else {
        end_idx = haystack_len as ptrdiff_t;
    }
    let mut lastmatch: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *needle as ::core::ffi::c_int == NUL {
        lastmatch = haystack.offset(end_idx as isize);
    } else {
        let mut rest: *const ::core::ffi::c_char = haystack;
        while *rest as ::core::ffi::c_int != NUL {
            rest = strstr(rest, needle);
            if rest.is_null() || rest > haystack.offset(end_idx as isize) {
                break;
            }
            lastmatch = rest;
            rest = rest.offset(1);
        }
    }
    if !lastmatch.is_null() {
        (*rettv).vval.v_number = lastmatch.offset_from(haystack) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_strtrans(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = transstr(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        true_0 != 0,
    );
}
pub unsafe extern "C" fn f_utf16idx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_bool_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_check_for_opt_bool_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut idx: varnumber_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
    if str.is_null() || idx < 0 as varnumber_T {
        return;
    }
    let mut countcc: varnumber_T = false_0 as varnumber_T;
    let mut charidx: varnumber_T = false_0 as varnumber_T;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        countcc = tv_get_bool(argvars.offset(2 as ::core::ffi::c_int as isize));
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            charidx = tv_get_bool(argvars.offset(3 as ::core::ffi::c_int as isize));
        }
    }
    let mut ptr2len: Option<
        unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
    > = None;
    if countcc != 0 {
        ptr2len = Some(
            utf_ptr2len as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
            as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    } else {
        ptr2len = Some(
            utfc_ptr2len as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        )
            as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
    }
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut utf16idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    p = str;
    len = 0 as ::core::ffi::c_int;
    while if charidx != 0 {
        (idx >= 0 as varnumber_T) as ::core::ffi::c_int
    } else {
        (p <= str.offset(idx as isize)) as ::core::ffi::c_int
    } != 0
    {
        if *p as ::core::ffi::c_int == NUL {
            if if charidx != 0 {
                (idx == 0 as varnumber_T) as ::core::ffi::c_int
            } else {
                (p == str.offset(idx as isize)) as ::core::ffi::c_int
            } != 0
            {
                (*rettv).vval.v_number = len as varnumber_T;
            }
            return;
        }
        utf16idx = len;
        let clen: ::core::ffi::c_int = ptr2len.expect("non-null function pointer")(p);
        let c: ::core::ffi::c_int = if clen > 1 as ::core::ffi::c_int {
            utf_ptr2char(p)
        } else {
            *p as ::core::ffi::c_int
        };
        if c > 0xffff as ::core::ffi::c_int {
            len += 1;
        }
        p = p.offset(clen as isize);
        if charidx != 0 {
            idx -= 1;
        }
        len += 1;
    }
    (*rettv).vval.v_number = utf16idx as varnumber_T;
}
pub unsafe extern "C" fn f_tolower(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = strcase_save(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        false_0 != 0,
    );
}
pub unsafe extern "C" fn f_toupper(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = strcase_save(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        true_0 != 0,
    );
}
pub unsafe extern "C" fn f_tr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let mut in_str: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut fromstr: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    let mut tostr: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if fromstr.is_null() || tostr.is_null() {
        return;
    }
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let mut first: bool = true_0 != 0;
    '_error: {
        while *in_str as ::core::ffi::c_int != NUL {
            let mut cpstr: *const ::core::ffi::c_char = in_str;
            let inlen: ::core::ffi::c_int = utfc_ptr2len(in_str);
            let mut cplen: ::core::ffi::c_int = inlen;
            let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut fromlen: ::core::ffi::c_int = 0;
            let mut p: *const ::core::ffi::c_char = fromstr;
            while *p as ::core::ffi::c_int != NUL {
                fromlen = utfc_ptr2len(p);
                if fromlen == inlen
                    && strncmp(in_str, p, inlen as size_t) == 0 as ::core::ffi::c_int
                {
                    let mut tolen: ::core::ffi::c_int = 0;
                    p = tostr;
                    while *p as ::core::ffi::c_int != NUL {
                        tolen = utfc_ptr2len(p);
                        let c2rust_fresh32 = idx;
                        idx = idx - 1;
                        if c2rust_fresh32 == 0 as ::core::ffi::c_int {
                            cplen = tolen;
                            cpstr = p;
                            break;
                        } else {
                            p = p.offset(tolen as isize);
                        }
                    }
                    if *p as ::core::ffi::c_int == NUL {
                        break '_error;
                    } else {
                        break;
                    }
                } else {
                    idx += 1;
                    p = p.offset(fromlen as isize);
                }
            }
            if first as ::core::ffi::c_int != 0 && cpstr == in_str {
                first = false_0 != 0;
                let mut tolen_0: ::core::ffi::c_int = 0;
                let mut p_0: *const ::core::ffi::c_char = tostr;
                while *p_0 as ::core::ffi::c_int != NUL {
                    tolen_0 = utfc_ptr2len(p_0);
                    idx -= 1;
                    p_0 = p_0.offset(tolen_0 as isize);
                }
                if idx != 0 as ::core::ffi::c_int {
                    break '_error;
                }
            }
            ga_grow(&raw mut ga, cplen);
            memmove(
                (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize)
                    as *mut ::core::ffi::c_void,
                cpstr as *const ::core::ffi::c_void,
                cplen as size_t,
            );
            ga.ga_len += cplen;
            in_str = in_str.offset(inlen as isize);
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
        return;
    }
    semsg(
        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
        fromstr,
    );
    ga_clear(&raw mut ga);
}
pub unsafe extern "C" fn f_trim(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let mut head: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    let mut mask: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prev: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut dir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if head.is_null() {
        return;
    }
    if tv_check_for_opt_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        mask = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf2 as *mut ::core::ffi::c_char,
        );
        if *mask as ::core::ffi::c_int == NUL {
            mask = ::core::ptr::null::<::core::ffi::c_char>();
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut error: bool = false_0 != 0;
            dir = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if error {
                return;
            }
            if dir < 0 as ::core::ffi::c_int || dir > 2 as ::core::ffi::c_int {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    tv_get_string(argvars.offset(2 as ::core::ffi::c_int as isize)),
                );
                return;
            }
        }
    }
    if dir == 0 as ::core::ffi::c_int || dir == 1 as ::core::ffi::c_int {
        while *head as ::core::ffi::c_int != NUL {
            let mut c1: ::core::ffi::c_int = utf_ptr2char(head);
            if mask.is_null() {
                if c1 > ' ' as ::core::ffi::c_int && c1 != 0xa0 as ::core::ffi::c_int {
                    break;
                }
            } else {
                p = mask;
                while *p as ::core::ffi::c_int != NUL {
                    if c1 == utf_ptr2char(p) {
                        break;
                    }
                    p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                }
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
            }
            head = head.offset(utfc_ptr2len(head as *mut ::core::ffi::c_char) as isize);
        }
    }
    let mut tail: *const ::core::ffi::c_char = head.offset(strlen(head) as isize);
    if dir == 0 as ::core::ffi::c_int || dir == 2 as ::core::ffi::c_int {
        while tail > head {
            prev = tail;
            prev = prev.offset(
                -((utf_head_off(
                    head as *mut ::core::ffi::c_char,
                    (prev as *mut ::core::ffi::c_char).offset(-(1 as ::core::ffi::c_int as isize)),
                ) + 1 as ::core::ffi::c_int) as isize),
            );
            let mut c1_0: ::core::ffi::c_int = utf_ptr2char(prev);
            if mask.is_null() {
                if c1_0 > ' ' as ::core::ffi::c_int && c1_0 != 0xa0 as ::core::ffi::c_int {
                    break;
                }
            } else {
                p = mask;
                while *p as ::core::ffi::c_int != NUL {
                    if c1_0 == utf_ptr2char(p) {
                        break;
                    }
                    p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                }
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
            }
            tail = prev;
        }
    }
    (*rettv).vval.v_string = xstrnsave(head, tail.offset_from(head) as size_t);
}
pub unsafe extern "C" fn cmp_keyvalue_value_n(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut kv1: *mut keyvalue_T = a as *mut keyvalue_T;
    let mut kv2: *mut keyvalue_T = b as *mut keyvalue_T;
    return strncmp(
        (*kv1).value,
        (*kv2).value,
        if (*kv1).length > (*kv2).length {
            (*kv1).length
        } else {
            (*kv2).length
        },
    );
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int =
            (*utf8len_tab.ptr())[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[cfg(test)]
mod tests {
    use super::{any_non_ascii, ascii_upcase, strnicmp_asc, trailing_spaces_start, unquote};

    fn unquote_all(src: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        unquote(src, &mut |b| out.push(b));
        out
    }

    #[test]
    fn unquote_mirrors_the_unit_spec_cases() {
        assert_eq!(unquote_all(b"abc"), b"abc"); // unquoted copies as-is
        assert_eq!(unquote_all(br#""abc""#), b"abc"); // fully quoted word
        assert_eq!(unquote_all(br#"a"b"c"#), b"abc"); // partially quoted
        assert_eq!(unquote_all(br#"a""b"#), b"ab"); // removes ""
        assert_eq!(unquote_all(br#""a\"b""#), br#"a"b"#); // unescapes \"
        assert_eq!(unquote_all(br#""a\\b""#), br#"a\b"#); // unescapes doubled backslash
        assert_eq!(unquote_all(br#"a\\b"#), br#"a\\b"#); // but not outside quotes
        assert_eq!(unquote_all(br#""a\nb""#), br#"a\nb"#); // \n is not unescaped
        assert_eq!(unquote_all(br#""abc"#), b"abc"); // unpaired quote stripped
        assert_eq!(unquote_all(br#"a\"#), br#"a\"#); // may end with one backslash
    }

    #[test]
    fn strnicmp_folds_only_ascii_and_stops_at_len_diff_or_nul() {
        assert_eq!(strnicmp_asc(b"abc", b"ABC", 3), 0);
        assert!(strnicmp_asc(b"abc", b"abd", 3) < 0);
        assert_eq!(strnicmp_asc(b"abX", b"abY", 2), 0); // len clamps the compare
        assert!(strnicmp_asc(b"ab", b"abc", 5) < 0); // terminator vs 'c'
        assert_eq!(strnicmp_asc(b"", b"", 4), 0);
        // Bytes >= 0x80 compare as signed chars, exactly like the C code.
        assert!(strnicmp_asc(b"\x80", b"\x7f", 1) < 0);
    }

    #[test]
    fn trailing_spaces_respect_escapes_and_never_take_byte_zero() {
        assert_eq!(trailing_spaces_start(b"ab  "), 2);
        assert_eq!(trailing_spaces_start(b"ab\t "), 2);
        assert_eq!(trailing_spaces_start(b"ab\\  "), 4); // escaped space stays
        assert_eq!(trailing_spaces_start(&[b'a', 22, b' ']), 3); // Ctrl-V escapes
        assert_eq!(trailing_spaces_start(b" "), 1); // first byte never stripped
        assert_eq!(trailing_spaces_start(b""), 0);
    }

    #[test]
    fn upcase_and_ascii_scan() {
        let mut buf = *b"aZ9\x80!";
        ascii_upcase(&mut buf);
        assert_eq!(&buf, b"AZ9\x80!");
        assert!(any_non_ascii(b"caf\xc3\xa9"));
        assert!(!any_non_ascii(b"cafe"));
    }
}
