//! `<>` key notation: the spelling of a key as text, and its encoding as
//! bytes.
//!
//! A key that is not a printable character travels through the editor as a
//! negative `c_int` and through a byte stream as `K_SPECIAL` plus two bytes;
//! modifiers ride along as a `K_SPECIAL KS_MODIFIER <bits>` prefix. This
//! module is the two translations between that and the text a user writes:
//! [`find_special_key`] and [`replace_termcodes`] read `<C-S-Up>`,
//! [`get_special_key_name`] writes it.
//!
//! The codes are in [`codes`] and the tables that name them in [`tables`];
//! everything here is about the notation. The raw pointers stop at this
//! level — both children are safe code.
//!
//! Two escaping helpers live here for want of a better home:
//! [`vim_strsave_escape_ks`] and [`vim_unescape_ks`], which hide and reveal a
//! literal 0x80 byte in text that is about to be, or has just been, treated
//! as a key sequence.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint};
use core::{ptr, slice};

use crate::ascii::{ascii_isdigit, ascii_isident};
use crate::charset::{transchar, vim_isprintc, vim_str2nr};
use crate::eval::vars::get_var_value;
use crate::global_cell::GlobalCell;
use crate::main::{current_sctx, e_invarg, e_usingsid};
use crate::mbyte::{
    utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len,
};
use crate::memory::{xmalloc, xrealloc};
use crate::message::emsg;
use crate::os::cshim::{gettext, snprintf, strncasecmp, strncmp};
use crate::strings::vim_strchr;
use crate::types::{
    CpoFlag, MB_MAXBYTES, NUL, key_extra, scid_T, size_t, uvarnumber_T, varnumber_T,
};
use ::libc::strlen;

mod codes;
pub use self::codes::*;
mod tables;
pub use self::tables::*;

/// [`vim_str2nr`] flags: accept decimal, octal, hex and binary alike.
const STR2NR_ALL: c_int = 15;

/// The key code of a `KS_EXTRA` key — everything with no termcap name of its
/// own, which is most of what a modern terminal sends.
const fn extra(ke: key_extra) -> c_int {
    -(KS_EXTRA + ((ke as c_int) << 8))
}

/// The key code of a two-character termcap name (C's `TERMCAP2KEY`).
const fn termcap(first: u8, second: u8) -> c_int {
    -((first as c_int) + ((second as c_int) << 8))
}

/// C's `TOUPPER_ASC`: upper-case an ASCII letter and leave everything else —
/// a byte above 0x7f included — alone.
const fn to_upper_ascii(c: c_int) -> c_int {
    if c < 'a' as c_int || c > 'z' as c_int {
        c
    } else {
        c - ('a' as c_int - 'A' as c_int)
    }
}

/// C's `ASCII_ISALPHA`, which tests the *unsigned* value and so says no to
/// anything outside 7-bit ASCII.
const fn is_ascii_alpha(key: c_int) -> bool {
    (key >= 'A' as c_int && key <= 'Z' as c_int) || (key >= 'a' as c_int && key <= 'z' as c_int)
}

/// Fold `modifiers` into `key` where the terminal has a code for the
/// combination: `<S-Up>` is a key of its own, not `Up` plus a shift bit.
/// `modifiers` is left holding whatever could not be folded in.
///
/// # Safety
/// `modifiers` must point at a writable `c_int`.
pub unsafe fn simplify_key(key: c_int, modifiers: *mut c_int) -> c_int {
    unsafe {
        if *modifiers & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0 {
            return key;
        }
        // TAB is the one key with a shifted code but no unshifted termcap row.
        if key == TAB && *modifiers & MOD_MASK_SHIFT != 0 {
            *modifiers &= !MOD_MASK_SHIFT;
            return K_S_TAB;
        }
        match tables::simplify(key, *modifiers) {
            Some((simplified, left)) => {
                *modifiers = left;
                simplified
            }
            None => key,
        }
    }
}

/// `<xKey>` to `<Key>`: the DEC and vt100 spellings of keys that also have an
/// ordinary termcap name.
fn handle_x_keys(key: c_int) -> c_int {
    match key {
        K_XUP => K_UP,
        K_XDOWN => K_DOWN,
        K_XLEFT => K_LEFT,
        K_XRIGHT => K_RIGHT,
        K_XHOME | K_ZHOME => K_HOME,
        K_XEND | K_ZEND => K_END,
        K_XF1 => K_F1,
        K_XF2 => K_F2,
        K_XF3 => K_F3,
        K_XF4 => K_F4,
        K_S_XF1 => K_S_F1,
        K_S_XF2 => K_S_F2,
        K_S_XF3 => K_S_F3,
        K_S_XF4 => K_S_F4,
        _ => key,
    }
}

/// The `<...>` spelling of `key` with `modifiers` held down.
///
/// The answer lives in one static buffer and is only valid until the next
/// call — which is also why every caller of this in a test has to be one
/// test.
pub fn get_special_key_name(key: c_int, modifiers: c_int) -> *mut c_char {
    /// Where the answer lives between calls.
    static NAME: GlobalCell<[c_char; MAX_KEY_NAME_LEN as usize + 1]> =
        GlobalCell::new([0; MAX_KEY_NAME_LEN as usize + 1]);

    let mut key = key;
    let mut modifiers = modifiers;
    unsafe {
        // A key that stands for a normal character.
        if key < 0 && c_int::from(termcap_name(key)[0]) == KS_KEY {
            key = c_int::from(termcap_name(key)[1]);
        }
        // A shifted or ctrl'ed special key becomes the plain key and a bit.
        if key < 0
            && let Some((plain, bit)) = unshift(key)
        {
            modifiers |= bit;
            key = plain;
        }

        let mut name = name_of_code(key);
        // Not a known key and not printable: try to read modifiers off the byte.
        if key > 0 && utf_char2len(key) == 1 {
            if name.is_none()
                && (!vim_isprintc(key) || key & 0x7f == ' ' as c_int)
                && key & 0x80 != 0
            {
                key &= 0x7f;
                modifiers |= MOD_MASK_ALT;
                name = name_of_code(key);
            }
            if name.is_none() && !vim_isprintc(key) && key < ' ' as c_int {
                key += '@' as c_int;
                modifiers |= MOD_MASK_CTRL;
            }
        }

        let mut out = [0u8; MAX_KEY_NAME_LEN as usize + 1];
        out[0] = b'<';
        let mut at = 1;
        for letter in printed_modifiers(modifiers) {
            out[at] = letter;
            out[at + 1] = b'-';
            at += 2;
        }
        match name {
            // An unnamed special key prints as its termcap name.
            None if key < 0 => {
                let code = termcap_name(key);
                out[at..at + 4].copy_from_slice(&[b't', b'_', code[0], code[1]]);
                at += 4;
            }
            // Not a special key at all: only modifiers, printed directly.
            None => {
                let len = utf_char2len(key);
                if len == 1 && vim_isprintc(key) {
                    out[at] = key as u8;
                    at += 1;
                } else if len > 1 {
                    let mut wide = [0u8; MB_MAXBYTES + 1];
                    let len = utf_char2bytes(key, wide.as_mut_ptr().cast()) as usize;
                    out[at..at + len].copy_from_slice(&wide[..len]);
                    at += len;
                } else {
                    let shown = CStr::from_ptr(transchar(key)).to_bytes();
                    out[at..at + shown.len()].copy_from_slice(shown);
                    at += shown.len();
                }
            }
            // A named key, as long as the name fits.
            Some(name) => {
                if (name.len() + at + 2) as c_int <= MAX_KEY_NAME_LEN {
                    out[at..at + name.len()].copy_from_slice(name.as_bytes());
                    at += name.len();
                }
            }
        }
        out[at] = b'>';

        NAME.with_mut(|dst| {
            for (slot, byte) in dst.iter_mut().zip(out) {
                *slot = byte as c_char;
            }
        });
        NAME.ptr().cast()
    }
}

/// Translate one `<>` name at `*srcp` into `dst`, advancing `*srcp` past it.
///
/// Answers the number of bytes written, zero when `*srcp` does not start with
/// a name this understands.
///
/// # Safety
/// `srcp` must point at a readable pointer into a buffer of `src_len` bytes,
/// and `dst` must have room for one key's encoding (19 bytes is enough).
pub unsafe fn trans_special(
    srcp: *mut *const c_char,
    src_len: size_t,
    dst: *mut c_char,
    flags: c_int,
    escape_ks: bool,
    did_simplify: *mut bool,
) -> c_uint {
    unsafe {
        let mut modifiers = 0;
        let key = find_special_key(srcp, src_len, &raw mut modifiers, flags, did_simplify);
        if key == 0 {
            return 0;
        }
        special_to_buf(key, modifiers, escape_ks, dst)
    }
}

/// Write the byte sequence for `key` with `modifiers` into `dst` and answer
/// its length. Not NUL-terminated; this is how a key is encoded in a string.
///
/// With `escape_ks` set, a literal 0x80 byte inside the character is escaped
/// the way the typeahead buffer needs.
///
/// # Safety
/// `dst` must have room for three modifier bytes plus the key: `MB_MAXBYTES`,
/// or three times that with `escape_ks` set.
pub unsafe fn special_to_buf(
    key: c_int,
    modifiers: c_int,
    escape_ks: bool,
    dst: *mut c_char,
) -> c_uint {
    unsafe {
        let mut dlen = 0;
        if modifiers != 0 {
            dlen = put_bytes(
                dst,
                dlen,
                &[K_SPECIAL as u8, KS_MODIFIER as u8, modifiers as u8],
            );
        }
        if key < 0 {
            let code = termcap_name(key);
            dlen = put_bytes(dst, dlen, &[K_SPECIAL as u8, code[0], code[1]]);
        } else if escape_ks {
            let after = add_char2buf(key, dst.add(dlen));
            let written = after.offset_from(dst);
            debug_assert!(written >= 0 && written as usize <= c_uint::MAX as usize);
            dlen = written as usize;
        } else {
            dlen += utf_char2bytes(key, dst.add(dlen)) as usize;
        }
        dlen as c_uint
    }
}

/// Copy `bytes` to `dst[at..]` and answer the offset past them.
///
/// # Safety
/// `dst[at..at + bytes.len()]` must be writable.
unsafe fn put_bytes(dst: *mut c_char, at: usize, bytes: &[u8]) -> usize {
    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr().cast(), dst.add(at), bytes.len()) };
    at + bytes.len()
}

/// Read one `<>` name at `*srcp`, advancing `*srcp` past it.
///
/// Answers the key code, with the modifiers that could not be folded into it
/// left in `*modp`, or 0 when there is no name there. `did_simplify`, when
/// given, is set if `FSK_SIMPLIFY` collapsed a Ctrl modifier into the key.
///
/// # Safety
/// `srcp` must point at a readable pointer into a buffer of `src_len` bytes,
/// and `modp` at a writable `c_int`.
pub unsafe fn find_special_key(
    srcp: *mut *const c_char,
    src_len: size_t,
    modp: *mut c_int,
    flags: c_int,
    did_simplify: *mut bool,
) -> c_int {
    unsafe {
        if src_len == 0 {
            return 0;
        }
        let in_string = flags & FSK_IN_STRING != 0;
        let end = (*srcp).add(src_len).sub(1);
        let mut src = *srcp;
        if *src != b'<' as c_char {
            return 0;
        }
        if *src.add(1) == b'*' as c_char {
            src = src.add(1); // <*xxx>: do not simplify
        }

        // Find the end of the modifier list.
        let mut last_dash = src;
        let mut bp = src.add(1);
        let mut len: c_int = 0;
        while bp <= end && (*bp == b'-' as c_char || ascii_isident(c_int::from(*bp))) {
            if *bp == b'-' as c_char {
                last_dash = bp;
                if bp.add(1) <= end {
                    len = utfc_ptr2len_len(bp.add(1), end.offset_from(bp) as c_int + 1);
                    // Anything is accepted, as in <C-?>. In a string <C-"> and
                    // <M-"> are not, because " ends the string; <M-\"> works.
                    if end.offset_from(bp) > len as isize
                        && !(in_string && *bp.add(1) == b'"' as c_char)
                        && *bp.offset((len + 1) as isize) == b'>' as c_char
                    {
                        bp = bp.offset(len as isize);
                    } else if end.offset_from(bp) > 2
                        && in_string
                        && *bp.add(1) == b'\\' as c_char
                        && *bp.add(2) == b'"' as c_char
                        && *bp.add(3) == b'>' as c_char
                    {
                        bp = bp.add(2);
                    }
                }
            }
            if end.offset_from(bp) > 3 && *bp == b't' as c_char && *bp.add(1) == b'_' as c_char {
                bp = bp.add(3); // skip t_xx; xx may be '-' or '>'
            } else if end.offset_from(bp) > 4 && strncasecmp(bp, c"char-".as_ptr(), 5) == 0 {
                vim_str2nr(
                    bp.add(5),
                    ptr::null_mut(),
                    &raw mut len,
                    STR2NR_ALL,
                    ptr::null_mut::<varnumber_T>(),
                    ptr::null_mut::<uvarnumber_T>(),
                    0,
                    true,
                    ptr::null_mut(),
                );
                if len == 0 {
                    emsg(gettext(&raw const e_invarg as *const c_char));
                    return 0;
                }
                bp = bp.offset((len + 5) as isize);
                break;
            }
            bp = bp.add(1);
        }

        if bp > end || *bp != b'>' as c_char {
            return 0;
        }
        let end_of_name = bp.add(1);

        // Which modifiers are given?
        let mut modifiers = 0;
        bp = src.add(1);
        while bp < last_dash {
            if *bp != b'-' as c_char {
                let bit = name_to_mod_mask(c_int::from(*bp as u8));
                if bit == 0 {
                    return 0; // illegal modifier name
                }
                modifiers |= bit;
            }
            bp = bp.add(1);
        }

        let mut key = if strncasecmp(last_dash.add(1), c"char-".as_ptr(), 5) == 0
            && ascii_isdigit(c_int::from(*last_dash.add(6)))
        {
            // <Char-123>, <Char-033> or <Char-0x33>.
            let mut number: uvarnumber_T = 0;
            vim_str2nr(
                last_dash.add(6),
                ptr::null_mut(),
                &raw mut len,
                STR2NR_ALL,
                ptr::null_mut::<varnumber_T>(),
                &raw mut number,
                0,
                true,
                ptr::null_mut(),
            );
            if len == 0 {
                emsg(gettext(&raw const e_invarg as *const c_char));
                return 0;
            }
            number as c_int
        } else {
            // A single-letter modifier, or a special key name.
            let mut off = 1;
            if in_string
                && *last_dash.add(1) == b'\\' as c_char
                && *last_dash.add(2) == b'"' as c_char
            {
                // In a double-quoted string, `"` is written `\"`.
                len = 2;
                off = 2;
            } else {
                len = utfc_ptr2len(last_dash.add(1));
            }
            if modifiers != 0 && *last_dash.offset((len + 1) as isize) == b'>' as c_char {
                utf_ptr2char(last_dash.offset(off as isize))
            } else {
                let code = get_special_key_code(last_dash.offset(off as isize));
                if flags & FSK_KEEP_X_KEY == 0 {
                    handle_x_keys(code)
                } else {
                    code
                }
            }
        };

        // get_special_key_code() answers NUL for a name it does not know.
        if key == NUL {
            return 0;
        }
        // Only keep a modifier that no key code already includes.
        key = simplify_key(key, &raw mut modifiers);
        if flags & FSK_KEYCODE == 0 {
            // No key code wanted: answer with the single-byte code.
            if key == K_BS {
                key = BS;
            } else if key == K_DEL || key == K_KDEL {
                key = DEL;
            }
        }
        // A normal character with a modifier: try to make one byte of the two,
        // Alt and Meta excepted.
        if key >= 0 {
            key = extract_modifiers(
                key,
                &raw mut modifiers,
                flags & FSK_SIMPLIFY != 0,
                did_simplify,
            );
        }
        *modp = modifiers;
        *srcp = end_of_name;
        key
    }
}

/// Fold the modifiers a single byte can carry into `key`: `Shift-a` becomes
/// `A`, `Ctrl-@` becomes `<Nul>`. Alt and Meta are never folded.
///
/// With `simplify` clear the Ctrl half is skipped, which is how a caller
/// keeps both spellings of `<C-H>`; `did_simplify` says whether it happened.
///
/// # Safety
/// `modp` must point at a writable `c_int`, and `did_simplify` at a writable
/// `bool` or be null.
unsafe fn extract_modifiers(
    key: c_int,
    modp: *mut c_int,
    simplify: bool,
    did_simplify: *mut bool,
) -> c_int {
    unsafe {
        let mut key = key;
        let mut modifiers = *modp;

        if modifiers & MOD_MASK_SHIFT != 0 && is_ascii_alpha(key) {
            key = to_upper_ascii(key);
            // <C-S-a> keeps the shift; <S-a>, <A-S-a> and <S-A> do not.
            if modifiers & MOD_MASK_CTRL == 0 {
                modifiers &= !MOD_MASK_SHIFT;
            }
        }
        // <C-H> and <C-h> mean the same thing; always use "H".
        if modifiers & MOD_MASK_CTRL != 0 && is_ascii_alpha(key) {
            key = to_upper_ascii(key);
        }
        if simplify
            && modifiers & MOD_MASK_CTRL != 0
            && ((key >= '?' as c_int && key <= '_' as c_int) || is_ascii_alpha(key))
        {
            key = to_upper_ascii(key) ^ 0x40;
            modifiers &= !MOD_MASK_CTRL;
            if key == NUL {
                key = K_ZERO; // <C-@> is <Nul>
            }
            if !did_simplify.is_null() {
                *did_simplify = true;
            }
        }

        *modp = modifiers;
        key
    }
}

/// The code of the special key called `name`, or 0 when there is no such key.
///
/// The name ends at the first non-identifier byte rather than at the NUL, so
/// a caller may point this into the middle of a larger string. A `t_xx` name
/// is a raw termcap code and never reaches the table.
///
/// # Safety
/// `name` must point at a NUL-terminated string.
pub unsafe fn get_special_key_code(name: *const c_char) -> c_int {
    unsafe {
        if *name == b't' as c_char
            && *name.add(1) == b'_' as c_char
            && *name.add(2) != 0
            && *name.add(3) != 0
        {
            return termcap_key([*name.add(2) as u8, *name.add(3) as u8]);
        }
        let mut len = 0;
        while ascii_isident(c_int::from(*name.add(len))) {
            len += 1;
        }
        code_for_name(slice::from_raw_parts(name.cast::<u8>(), len))
    }
}

/// Which button a mouse pseudo-code is about, and whether it was a click or a
/// drag. Answers 0 for a code that is not a mouse event, leaving the flags
/// untouched.
///
/// # Safety
/// `is_click` and `is_drag` must point at writable `bool`s.
pub unsafe fn get_mouse_button(code: c_int, is_click: *mut bool, is_drag: *mut bool) -> c_int {
    match mouse_event(code) {
        Some(event) => {
            unsafe {
                *is_click = event.is_click;
                *is_drag = event.is_drag;
            }
            event.button
        }
        None => 0,
    }
}

/// Encode `<key>` notation into the editor's internal key representation.
/// (This does *not* process raw terminal escape sequences, despite the legacy
/// "termcode" name.)
///
/// Parses `<C-Up>`, `<CR>`, `<Esc>`, `<F1>`, `<Leader>`, `<SID>` and the rest,
/// and emits the `K_SPECIAL` byte sequences for them; a literal `K_SPECIAL`
/// byte in the input is escaped as `K_SPECIAL KS_SPECIAL KE_FILLER`. Used for
/// both sides of a mapping, the rhs of a menu command, and `feedkeys()` input.
///
/// Also handles `<C-v>` and backslash escapes (per `cpo_val`),
/// `<Leader>`/`<LocalLeader>` expansion, `<SID>` script-id substitution, and —
/// unless `REPTERM_NO_SIMPLIFY` — simplifications such as `<C-H>` to 0x08.
///
/// `sid_arg` is the script id `<SID>` stands for, or 0 to take the current
/// one. Only `CpoFlag::BSLASH` is read out of `cpo_val`.
///
/// # Safety
/// `from` must be readable for `from_len` bytes and `bufp` must be writable.
/// When `*bufp` is non-null it is used directly and is assumed to be 128 bytes
/// long, enough for the lhs of a mapping; otherwise the result is allocated
/// and `*bufp` is set to it. `did_simplify`, when non-null, must be writable.
pub unsafe fn replace_termcodes(
    from: *const c_char,
    from_len: size_t,
    bufp: *mut *mut c_char,
    sid_arg: scid_T,
    flags: c_int,
    did_simplify: *mut bool,
    cpo_val: *const c_char,
) -> *mut c_char {
    unsafe {
        let end = from.add(from_len).sub(1);
        // A backslash is a special character unless 'cpoptions' contains B.
        let do_backslash = vim_strchr(cpo_val, CpoFlag::BSLASH.as_c_int()).is_null();
        let do_special = flags & REPTERM_NO_SPECIAL == 0;
        let allocated = (*bufp).is_null();
        // Worst case one character becomes six bytes (a shifted special key),
        // plus a NUL at the end.
        let buf_len = if allocated { from_len * 6 + 1 } else { 128 };
        let result = if allocated {
            xmalloc(buf_len).cast::<c_char>()
        } else {
            *bufp
        };

        let mut dlen: usize = 0;
        let mut src = from;
        while src <= end {
            if !allocated && dlen + 64 > buf_len {
                return ptr::null_mut();
            }
            // Check for special <> keycodes, like "<C-S-LeftMouse>".
            if do_special
                && (flags & REPTERM_DO_LT != 0
                    || (end.offset_from(src) >= 3 && strncmp(src, c"<lt>".as_ptr(), 4) != 0))
            {
                // <SID>Func becomes K_SNR <script-nr> _Func, which is how a
                // script-local function's name is spelled.
                // (Room: 5 * 6 = 30 bytes; needed: 3 + <nr> + 1 <= 14.)
                if end.offset_from(src) >= 4 && strncasecmp(src, c"<SID>".as_ptr(), 5) == 0 {
                    if sid_arg < 0 || (sid_arg == 0 && (*current_sctx.ptr()).sc_sid <= 0) {
                        emsg(gettext(&raw const e_usingsid as *const c_char));
                    } else {
                        let sid = if sid_arg != 0 {
                            sid_arg
                        } else {
                            (*current_sctx.ptr()).sc_sid
                        };
                        src = src.add(5);
                        dlen = put_bytes(
                            result,
                            dlen,
                            &[K_SPECIAL as u8, KS_EXTRA as u8, KE_SNR as u8],
                        );
                        snprintf(result.add(dlen), buf_len - dlen, c"%d".as_ptr(), sid);
                        dlen += strlen(result.add(dlen));
                        dlen = put_bytes(result, dlen, b"_");
                        continue;
                    }
                }

                let written = trans_special(
                    &raw mut src,
                    end.offset_from(src) as size_t + 1,
                    result.add(dlen),
                    FSK_KEYCODE
                        | if flags & REPTERM_NO_SIMPLIFY != 0 {
                            0
                        } else {
                            FSK_SIMPLIFY
                        },
                    true,
                    did_simplify,
                ) as usize;
                if written != 0 {
                    dlen += written;
                    continue;
                }
            }

            if do_special {
                // <Leader> and <LocalLeader> take the value of "mapleader" and
                // "maplocalleader"; a backslash stands in when either is unset.
                let (len, value) = if end.offset_from(src) >= 7
                    && strncasecmp(src, c"<Leader>".as_ptr(), 8) == 0
                {
                    (8, get_var_value(c"g:mapleader".as_ptr()))
                } else if end.offset_from(src) >= 12
                    && strncasecmp(src, c"<LocalLeader>".as_ptr(), 13) == 0
                {
                    (13, get_var_value(c"g:maplocalleader".as_ptr()))
                } else {
                    (0, ptr::null_mut())
                };
                if len != 0 {
                    // Up to 8 * 6 characters of "mapleader" are allowed.
                    let mut leader = if value.is_null() || *value == 0 || strlen(value) > 8 * 6 {
                        c"\\".as_ptr()
                    } else {
                        value.cast_const()
                    };
                    while *leader != 0 {
                        *result.add(dlen) = *leader;
                        dlen += 1;
                        leader = leader.add(1);
                    }
                    src = src.add(len);
                    continue;
                }
            }

            // Remove CTRL-V and take the next character literally. On the "from"
            // side a trailing CTRL-V is kept, on the "to" side it is dropped, so
            // that ":map xx ^V" maps xx to nothing. Without 'B' in 'cpoptions' a
            // backslash does the same job.
            let quoted = *src;
            if c_int::from(quoted) == Ctrl_V || (do_backslash && quoted == b'\\' as c_char) {
                src = src.add(1);
                if src > end {
                    if flags & REPTERM_FROM_PART != 0 {
                        *result.add(dlen) = quoted;
                        dlen += 1;
                    }
                    break;
                }
            }

            // Copy one whole character, hiding a literal K_SPECIAL byte.
            for _ in 0..utfc_ptr2len_len(src, end.offset_from(src) as c_int + 1) {
                if *src == K_SPECIAL as u8 as c_char {
                    dlen = put_bytes(
                        result,
                        dlen,
                        &[K_SPECIAL as u8, KS_SPECIAL as u8, KE_FILLER as u8],
                    );
                } else {
                    *result.add(dlen) = *src;
                    dlen += 1;
                }
                src = src.add(1);
            }
        }
        *result.add(dlen) = 0;

        if allocated {
            *bufp = xrealloc(result.cast(), dlen + 1).cast();
        }
        *bufp
    }
}

/// Append `c` to `s`, escaping a literal `K_SPECIAL` byte the way the
/// typeahead buffer needs, and answer a pointer past what was written.
///
/// # Safety
/// `s` must have room for `MB_MAXBYTES + 1` bytes.
pub unsafe fn add_char2buf(c: c_int, s: *mut c_char) -> *mut c_char {
    unsafe {
        let mut encoded = [0u8; MB_MAXBYTES + 1];
        let len = utf_char2bytes(c, encoded.as_mut_ptr().cast()) as usize;
        let mut at = 0;
        for &byte in &encoded[..len] {
            at = if c_int::from(byte) == K_SPECIAL {
                put_bytes(s, at, &[K_SPECIAL as u8, KS_SPECIAL as u8, KE_FILLER as u8])
            } else {
                put_bytes(s, at, &[byte])
            };
        }
        s.add(at)
    }
}

/// A copy of `p` with every literal `K_SPECIAL` byte escaped, so the result
/// can go into the typeahead buffer. The caller owns the allocation.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub unsafe fn vim_strsave_escape_ks(p: *mut c_char) -> *mut c_char {
    unsafe {
        // Room for three times as much, four in case of an illegal utf-8 byte:
        // 0xc0 -> 0xc3 - 0x80 -> 0xc3 K_SPECIAL KS_SPECIAL KE_FILLER.
        let res = xmalloc(strlen(p) * 4 + 1).cast::<c_char>();
        let mut dst = res;
        let mut src = p;
        while *src != 0 {
            if c_int::from(*src as u8) == K_SPECIAL && *src.add(1) != 0 && *src.add(2) != 0 {
                // Copy a special key unchanged.
                ptr::copy_nonoverlapping(src, dst, 3);
                src = src.add(3);
                dst = dst.add(3);
            } else {
                // Add the character, possibly multi-byte, escaping K_SPECIAL.
                // Careful: it can be an illegal byte.
                dst = add_char2buf(utf_ptr2char(src), dst);
                src = src.offset(utf_ptr2len(src) as isize);
            }
        }
        *dst = 0;
        res
    }
}

/// Undo [`vim_strsave_escape_ks`], in place.
///
/// # Safety
/// `p` must point at a NUL-terminated, writable string.
pub unsafe fn vim_unescape_ks(p: *mut c_char) {
    unsafe {
        let mut src = p.cast::<u8>();
        let mut dst = p.cast::<u8>();
        while *src != 0 {
            if c_int::from(*src) == K_SPECIAL
                && c_int::from(*src.add(1)) == KS_SPECIAL
                && c_int::from(*src.add(2)) == KE_FILLER
            {
                *dst = K_SPECIAL as u8;
                dst = dst.add(1);
                src = src.add(3);
            } else {
                *dst = *src;
                dst = dst.add(1);
                src = src.add(1);
            }
        }
        *dst = 0;
    }
}
