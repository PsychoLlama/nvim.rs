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

use crate::charset::Str2NrBases;
use crate::cstr;
use crate::types::BS;
use crate::types::DEL;
use crate::types::TAB;
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::{ptr, slice};

use crate::ascii::{ascii_isdigit, ascii_isident};
use crate::charset::{transchar, vim_isprintc, vim_str2nr};
use crate::eval::typval::NumBuf;
use crate::eval::vars::get_var_value;
use crate::main::{current_sctx, e_invarg, e_usingsid};
use crate::mbyte::{
    utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len,
};
use crate::memory::{xmalloc, xrealloc};
use crate::message::emsg;
use crate::os::cshim::{gettext, snprintf, strncasecmp};
use crate::strings::vim_strchr;
use crate::types::{
    CpoFlag, MB_MAXBYTES, NUL, key_extra, scid_T, size_t, uvarnumber_T, varnumber_T,
};

mod codes;
pub use self::codes::*;
mod tables;
pub use self::tables::*;

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

/// A read cursor over the text a `<>` name is being parsed out of.
///
/// [`find_special_key`] and [`replace_termcodes`] walk a `*const c_char` byte
/// by byte and peek a handful of bytes ahead of it; every one of those reads
/// was its own `unsafe` before. Capturing the pointer in a newtype moves the
/// promise to where the cursor is *built*, once per entry point, and leaves
/// the walk itself ordinary checked code — p23's shape 3.
///
/// Ordered by address, so `bp <= end` keeps reading the way the C does.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Cursor(*const c_char);

impl Cursor {
    /// # Safety
    /// `p` must stay readable at every offset the walk reaches. In practice
    /// that is a NUL-terminated string: the NUL is what stops the walk, and
    /// the few bytes peeked past the cursor are guarded by the caller's own
    /// length checks.
    const unsafe fn new(p: *const c_char) -> Self {
        Self(p)
    }

    /// The byte under the cursor.
    fn byte(self) -> c_char {
        self.at(0)
    }

    /// The byte `n` further on — the C's `bp[n]`.
    fn at(self, n: isize) -> c_char {
        // SAFETY: the constructor's promise.
        unsafe { *self.0.offset(n) }
    }

    /// The byte `n` further on, unsigned — C's `(uint8_t)p[n]`. A `c_char` is
    /// signed here, so which of the two a comparison uses is load-bearing
    /// once the byte is above 0x7f: `K_SPECIAL` is 0x80.
    fn u8_at(self, n: isize) -> u8 {
        self.at(n) as u8
    }

    /// The cursor `n` bytes further on.
    fn skip(self, n: isize) -> Self {
        Self(self.0.wrapping_offset(n))
    }

    /// How many bytes this cursor is ahead of `other` — the C's `end - bp`.
    fn gap(self, other: Self) -> isize {
        self.0.addr().wrapping_sub(other.0.addr()) as isize
    }

    /// The pointer back, for the callees that still take one.
    fn raw(self) -> *const c_char {
        self.0
    }

    /// The cursor's own storage, for the callees that advance it in place —
    /// C's `const char **`. Taking the address of a field is not a read, so
    /// this is safe; what the callee then writes through it is its own
    /// contract.
    fn slot(&mut self) -> *mut *const c_char {
        &raw mut self.0
    }
}

/// C's `STRNICMP(p, lit, strlen(lit)) == 0`: does the text at `p` open with
/// `lit`, ASCII case ignored?
fn starts_with_ignoring_case(p: Cursor, lit: &CStr) -> bool {
    let n = lit.to_bytes().len();
    // SAFETY: `p` is a cursor into a NUL-terminated string and `lit` is one
    // too, so the comparison stops at the shorter of the two.
    unsafe { strncasecmp(p.raw(), lit.as_ptr(), n) == 0 }
}

/// C's `strncmp(p, lit, strlen(lit)) == 0`: the case-sensitive half.
fn starts_with(p: Cursor, lit: &CStr) -> bool {
    let n = lit.to_bytes().len();
    // SAFETY: as [`starts_with_ignoring_case`].
    unsafe { cstr::prefix_eq(p.raw(), lit.as_ptr(), n) }
}

/// [`vim_str2nr`] as the `<>` parsers ask for it: the unsigned value at `p`
/// and the number of bytes it spans, zero when there is no number there.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
unsafe fn number_at(p: Cursor) -> (uvarnumber_T, c_int) {
    let mut number: uvarnumber_T = 0;
    let mut len: c_int = 0;
    // Bound out here so the call itself fits on one line; nine arguments
    // spread over nine lines would be nine unchecked lines.
    let (start, prep, lenp) = (p.raw(), ptr::null_mut(), &raw mut len);
    let (nptr, unptr) = (ptr::null_mut::<varnumber_T>(), &raw mut number);
    let (overflow, all) = (ptr::null_mut(), Str2NrBases::ALL);
    // SAFETY: the caller's promise, and every out-parameter is a live local
    // or null. Upstream passes null for `unptr` at the one call site that
    // does not want the value; writing a local it then ignores is the same.
    unsafe { vim_str2nr(start, prep, lenp, all, nptr, unptr, 0, true, overflow) };
    (number, len)
}

/// Fold `modifiers` into `key` where the terminal has a code for the
/// combination: `<S-Up>` is a key of its own, not `Up` plus a shift bit.
/// `modifiers` is left holding whatever could not be folded in.
pub fn simplify_key(key: c_int, modifiers: &mut ModMask) -> c_int {
    if !modifiers.has(ModMask::SHIFT | ModMask::CTRL) {
        return key;
    }
    // TAB is the one key with a shifted code but no unshifted termcap row.
    if key == TAB && modifiers.has(ModMask::SHIFT) {
        modifiers.clear(ModMask::SHIFT);
        return Key::STab.code();
    }
    match simplify(key, *modifiers) {
        Some((simplified, left)) => {
            *modifiers = left;
            simplified
        }
        None => key,
    }
}

/// [`simplify_key`] applied to the global `mod_mask`.
pub(crate) fn simplify_mod_mask(key: c_int) -> c_int {
    let mut modifiers = crate::main::mod_mask.get();
    let simplified = simplify_key(key, &mut modifiers);
    crate::main::mod_mask.set(modifiers);
    simplified
}

/// `<xKey>` to `<Key>`: the DEC and vt100 spellings of keys that also have an
/// ordinary termcap name.
fn handle_x_keys(key: c_int) -> c_int {
    match Key::try_from(key) {
        Ok(Key::Xup) => Key::Up.code(),
        Ok(Key::Xdown) => Key::Down.code(),
        Ok(Key::Xleft) => Key::Left.code(),
        Ok(Key::Xright) => Key::Right.code(),
        Ok(Key::Xhome | Key::Zhome) => Key::Home.code(),
        Ok(Key::Xend | Key::Zend) => Key::End.code(),
        Ok(Key::Xf1) => Key::F1.code(),
        Ok(Key::Xf2) => Key::F2.code(),
        Ok(Key::Xf3) => Key::F3.code(),
        Ok(Key::Xf4) => Key::F4.code(),
        Ok(Key::SXf1) => Key::SF1.code(),
        Ok(Key::SXf2) => Key::SF2.code(),
        Ok(Key::SXf3) => Key::SF3.code(),
        Ok(Key::SXf4) => Key::SF4.code(),
        _ => key,
    }
}

/// A `<...>` key spelling, NUL-terminated within its own storage.
///
/// Upstream answers a pointer into one static buffer, valid only until the
/// next call, which is why `str2special_arena` has to measure and copy in
/// two passes and why two callers in one test tread on each other.
pub(crate) type SpecialKeyName = [c_char; MAX_KEY_NAME_LEN as usize + 1];

/// The `<...>` spelling of `key` with `modifiers` held down.
pub fn get_special_key_name(key: c_int, modifiers: ModMask) -> SpecialKeyName {
    let mut key = key;
    let mut modifiers = modifiers;
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
            && (!unsafe { vim_isprintc(key) } || key & 0x7f == ' ' as c_int)
            && key & 0x80 != 0
        {
            key &= 0x7f;
            modifiers |= ModMask::ALT;
            name = name_of_code(key);
        }
        if name.is_none() && !unsafe { vim_isprintc(key) } && key < ' ' as c_int {
            key += '@' as c_int;
            modifiers |= ModMask::CTRL;
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
            if len == 1 && unsafe { vim_isprintc(key) } {
                out[at] = key as u8;
                at += 1;
            } else if len > 1 {
                let mut wide = [0u8; MB_MAXBYTES + 1];
                let len = unsafe { utf_char2bytes(key, wide.as_mut_ptr().cast()) } as usize;
                out[at..at + len].copy_from_slice(&wide[..len]);
                at += len;
            } else {
                let display = unsafe { transchar(key) };
                let shown = unsafe { CStr::from_ptr(display.as_ptr()) }.to_bytes();
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

    let mut name: SpecialKeyName = [0; MAX_KEY_NAME_LEN as usize + 1];
    for (slot, byte) in name.iter_mut().zip(out) {
        *slot = byte as c_char;
    }
    name
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
    let mut modifiers = ModMask::NONE;
    let key = unsafe { find_special_key(srcp, src_len, &raw mut modifiers, flags, did_simplify) };
    if key == 0 {
        return 0;
    }
    unsafe { special_to_buf(key, modifiers, escape_ks, dst) }
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
    modifiers: ModMask,
    escape_ks: bool,
    dst: *mut c_char,
) -> c_uint {
    let mut dlen = 0;
    if !modifiers.is_empty() {
        let prefix = [K_SPECIAL as u8, KS_MODIFIER as u8, modifiers.bits() as u8];
        // SAFETY: the caller's promise -- room for the modifier prefix.
        dlen = unsafe { put_bytes(dst, dlen, &prefix) };
    }
    if key < 0 {
        let code = termcap_name(key);
        let name = [K_SPECIAL as u8, code[0], code[1]];
        // SAFETY: as above -- three bytes into the caller's buffer.
        dlen = unsafe { put_bytes(dst, dlen, &name) };
    } else if escape_ks {
        // SAFETY: as above; `dst + dlen` is what is left of the buffer, and
        // the caller promised three times `MB_MAXBYTES` for this case.
        let after = unsafe { add_char2buf(key, dst.add(dlen)) };
        // SAFETY: `after` came out of `dst`, so both name the same object.
        let written = unsafe { after.offset_from(dst) };
        debug_assert!(written >= 0 && written as usize <= c_uint::MAX as usize);
        dlen = written as usize;
    } else {
        dlen += unsafe { utf_char2bytes(key, dst.add(dlen)) } as usize;
    }
    dlen as c_uint
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
    modp: *mut ModMask,
    flags: c_int,
    did_simplify: *mut bool,
) -> c_int {
    if src_len == 0 {
        return 0;
    }
    let in_string = flags & FSK_IN_STRING != 0;
    // SAFETY: the caller's promise -- `*srcp` names a readable buffer of
    // `src_len` bytes, NUL-terminated, which is what bounds the walk below.
    let mut src = unsafe { Cursor::new(*srcp) };
    let end = src.skip(src_len as isize - 1);
    if src.byte() != b'<' as c_char {
        return 0;
    }
    if src.at(1) == b'*' as c_char {
        src = src.skip(1); // <*xxx>: do not simplify
    }

    // Find the end of the modifier list.
    let mut last_dash = src;
    let mut bp = src.skip(1);
    let mut len: c_int = 0;
    while bp <= end && (bp.byte() == b'-' as c_char || ascii_isident(c_int::from(bp.byte()))) {
        if bp.byte() == b'-' as c_char {
            last_dash = bp;
            if bp.skip(1) <= end {
                let (after_dash, left) = (bp.skip(1).raw(), end.gap(bp) as c_int + 1);
                // SAFETY: `after_dash` is inside the caller's buffer and
                // `left` is what is left of it from there.
                len = unsafe { utfc_ptr2len_len(after_dash, left) };
                // Anything is accepted, as in <C-?>. In a string <C-"> and
                // <M-"> are not, because " ends the string; <M-\"> works.
                if end.gap(bp) > len as isize
                    && !(in_string && bp.at(1) == b'"' as c_char)
                    && bp.at((len + 1) as isize) == b'>' as c_char
                {
                    bp = bp.skip(len as isize);
                } else if end.gap(bp) > 2
                    && in_string
                    && bp.at(1) == b'\\' as c_char
                    && bp.at(2) == b'"' as c_char
                    && bp.at(3) == b'>' as c_char
                {
                    bp = bp.skip(2);
                }
            }
        }
        if end.gap(bp) > 3 && bp.byte() == b't' as c_char && bp.at(1) == b'_' as c_char {
            bp = bp.skip(3); // skip t_xx; xx may be '-' or '>'
        } else if end.gap(bp) > 4 && starts_with_ignoring_case(bp, c"char-") {
            // SAFETY: `bp + 5` is inside the caller's NUL-terminated buffer.
            len = unsafe { number_at(bp.skip(5)) }.1;
            if len == 0 {
                emsg(gettext(e_invarg));
                return 0;
            }
            bp = bp.skip((len + 5) as isize);
            break;
        }
        bp = bp.skip(1);
    }

    if bp > end || bp.byte() != b'>' as c_char {
        return 0;
    }
    let end_of_name = bp.skip(1);

    // Which modifiers are given?
    let mut modifiers = ModMask::NONE;
    bp = src.skip(1);
    while bp < last_dash {
        if bp.byte() != b'-' as c_char {
            let bit = name_to_mod_mask(c_int::from(bp.byte() as u8));
            if bit.is_empty() {
                return 0; // illegal modifier name
            }
            modifiers |= bit;
        }
        bp = bp.skip(1);
    }

    let mut key = if starts_with_ignoring_case(last_dash.skip(1), c"char-")
        && ascii_isdigit(c_int::from(last_dash.at(6)))
    {
        // <Char-123>, <Char-033> or <Char-0x33>.
        // SAFETY: `last_dash + 6` is inside the caller's NUL-terminated
        // buffer -- the five bytes of "char-" and a digit precede it.
        let (number, digits) = unsafe { number_at(last_dash.skip(6)) };
        len = digits;
        if len == 0 {
            emsg(gettext(e_invarg));
            return 0;
        }
        number as c_int
    } else {
        // A single-letter modifier, or a special key name.
        let mut off = 1;
        if in_string && last_dash.at(1) == b'\\' as c_char && last_dash.at(2) == b'"' as c_char {
            // In a double-quoted string, `"` is written `\"`.
            len = 2;
            off = 2;
        } else {
            // SAFETY: `last_dash + 1` is inside the caller's buffer, whose
            // NUL bounds the character `utfc_ptr2len` measures.
            len = unsafe { utfc_ptr2len(last_dash.skip(1).raw()) };
        }
        if !modifiers.is_empty() && last_dash.at((len + 1) as isize) == b'>' as c_char {
            // SAFETY: as above -- a character inside the caller's buffer.
            unsafe { utf_ptr2char(last_dash.skip(off).raw()) }
        } else {
            // SAFETY: as above; `get_special_key_code` stops at the first
            // byte that cannot be part of a name.
            let code = unsafe { get_special_key_code(last_dash.skip(off).raw()) };
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
    key = simplify_key(key, &mut modifiers);
    if flags & FSK_KEYCODE == 0 {
        // No key code wanted: answer with the single-byte code.
        if key == Key::Bs.code() {
            key = BS;
        } else if key == Key::Del.code() || key == Key::Kdel.code() {
            key = DEL;
        }
    }
    // A normal character with a modifier: try to make one byte of the two,
    // Alt and Meta excepted.
    if key >= 0 {
        let (modp_local, simplify) = (&raw mut modifiers, flags & FSK_SIMPLIFY != 0);
        // SAFETY: `modp_local` is a live local and `did_simplify` is the
        // caller's, which they promised is writable or null.
        key = unsafe { extract_modifiers(key, modp_local, simplify, did_simplify) };
    }
    // SAFETY: the caller's promise -- both out-parameters are writable.
    unsafe {
        *modp = modifiers;
        *srcp = end_of_name.raw();
    }
    key
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
    modp: *mut ModMask,
    simplify: bool,
    did_simplify: *mut bool,
) -> c_int {
    let mut key = key;
    let mut modifiers = unsafe { *modp };

    if modifiers.has(ModMask::SHIFT) && is_ascii_alpha(key) {
        key = to_upper_ascii(key);
        // <C-S-a> keeps the shift; <S-a>, <A-S-a> and <S-A> do not.
        if !modifiers.has(ModMask::CTRL) {
            modifiers.clear(ModMask::SHIFT);
        }
    }
    // <C-H> and <C-h> mean the same thing; always use "H".
    if modifiers.has(ModMask::CTRL) && is_ascii_alpha(key) {
        key = to_upper_ascii(key);
    }
    if simplify
        && modifiers.has(ModMask::CTRL)
        && ((key >= '?' as c_int && key <= '_' as c_int) || is_ascii_alpha(key))
    {
        key = to_upper_ascii(key) ^ 0x40;
        modifiers.clear(ModMask::CTRL);
        if key == NUL {
            key = Key::Zero.code(); // <C-@> is <Nul>
        }
        if !did_simplify.is_null() {
            unsafe { *did_simplify = true };
        }
    }

    unsafe { *modp = modifiers };
    key
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
    // SAFETY: the caller's promise -- a NUL-terminated string, whose NUL
    // stops both the `t_xx` peek and the identifier walk below.
    let name = unsafe { Cursor::new(name) };
    if name.byte() == b't' as c_char
        && name.at(1) == b'_' as c_char
        && name.at(2) != 0
        && name.at(3) != 0
    {
        return termcap_key([name.at(2) as u8, name.at(3) as u8]);
    }
    let mut len = 0;
    while ascii_isident(c_int::from(name.at(len))) {
        len += 1;
    }
    // SAFETY: `len` bytes of identifier were just read one at a time.
    code_for_name(unsafe { slice::from_raw_parts(name.raw().cast::<u8>(), len as usize) })
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
            unsafe { *is_click = event.is_click };
            unsafe { *is_drag = event.is_drag };
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
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise -- `from` is readable for `from_len`
    // bytes, and every read below stays at or before `end`.
    let mut src = unsafe { Cursor::new(from) };
    let end = src.skip(from_len as isize - 1);
    // A backslash is a special character unless 'cpoptions' contains B.
    // SAFETY: `cpo_val` is the caller's NUL-terminated option string.
    let do_backslash = unsafe { vim_strchr(cpo_val, CpoFlag::BSLASH.as_c_int()) }.is_null();
    let do_special = flags & REPTERM_NO_SPECIAL == 0;
    // SAFETY: the caller's promise -- `bufp` is readable and writable.
    let given = unsafe { *bufp };
    let allocated = given.is_null();
    // Worst case one character becomes six bytes (a shifted special key),
    // plus a NUL at the end.
    let buf_len = if allocated { from_len * 6 + 1 } else { 128 };
    let result = if allocated {
        // SAFETY: `xmalloc` either answers `buf_len` writable bytes or dies.
        unsafe { xmalloc(buf_len) }.cast::<c_char>()
    } else {
        given
    };

    let mut dlen: usize = 0;
    while src <= end {
        if !allocated && dlen + 64 > buf_len {
            return ptr::null_mut();
        }
        // Check for special <> keycodes, like "<C-S-LeftMouse>".
        if do_special
            && (flags & REPTERM_DO_LT != 0 || (end.gap(src) >= 3 && !starts_with(src, c"<lt>")))
        {
            // <SID>Func becomes K_SNR <script-nr> _Func, which is how a
            // script-local function's name is spelled.
            // (Room: 5 * 6 = 30 bytes; needed: 3 + <nr> + 1 <= 14.)
            if end.gap(src) >= 4 && starts_with_ignoring_case(src, c"<SID>") {
                if sid_arg < 0 || (sid_arg == 0 && current_sctx.get().sc_sid <= 0) {
                    emsg(gettext(e_usingsid));
                } else {
                    let sid = if sid_arg != 0 {
                        sid_arg
                    } else {
                        current_sctx.get().sc_sid
                    };
                    src = src.skip(5);
                    let snr = [K_SPECIAL as u8, KS_EXTRA as u8, KE_SNR as u8];
                    // SAFETY: `result` has `buf_len` bytes and the comment
                    // above accounts for the 14 this branch writes.
                    dlen = unsafe { put_bytes(result, dlen, &snr) };
                    let (at, room) = (result.wrapping_add(dlen), buf_len - dlen);
                    // SAFETY: `at` has `room` writable bytes, and the format
                    // takes exactly the one `c_int` argument given.
                    unsafe { snprintf(at, room, c"%d".as_ptr(), sid) };
                    // SAFETY: `snprintf` NUL-terminated what it wrote.
                    dlen += unsafe { cstr::bytes_at(at) }.len();
                    // SAFETY: as above -- one byte, still inside `buf_len`.
                    dlen = unsafe { put_bytes(result, dlen, b"_") };
                    continue;
                }
            }

            let left = end.gap(src) as size_t + 1;
            let simplify = if flags & REPTERM_NO_SIMPLIFY != 0 {
                0
            } else {
                FSK_SIMPLIFY
            };
            let (slot, at) = (src.slot(), result.wrapping_add(dlen));
            // SAFETY: `slot` is this frame's cursor, `left` the bytes left of
            // the caller's buffer, and `at` has at least the 64 bytes the
            // guard above kept free.
            let written = unsafe {
                trans_special(slot, left, at, FSK_KEYCODE | simplify, true, did_simplify)
            };
            if written != 0 {
                dlen += written as usize;
                continue;
            }
        }

        if do_special {
            // <Leader> and <LocalLeader> take the value of "mapleader" and
            // "maplocalleader"; a backslash stands in when either is unset.
            // SAFETY: both names are static, and `numbuf` is a live local.
            let (len, value) = if end.gap(src) >= 7 && starts_with_ignoring_case(src, c"<Leader>") {
                (8, unsafe {
                    get_var_value(c"g:mapleader".as_ptr(), &mut numbuf)
                })
            } else if end.gap(src) >= 12 && starts_with_ignoring_case(src, c"<LocalLeader>") {
                (13, unsafe {
                    get_var_value(c"g:maplocalleader".as_ptr(), &mut numbuf)
                })
            } else {
                (0, ptr::null_mut())
            };
            if len != 0 {
                // Up to 8 * 6 characters of "mapleader" are allowed.
                // SAFETY: `get_var_value` answers null or a NUL-terminated
                // string that outlives this loop.
                let too_long = !value.is_null() && unsafe { cstr::bytes_at(value) }.len() > 8 * 6;
                // SAFETY: the option's value, or the static backslash.
                let mut leader = unsafe { Cursor::new(value.cast_const()) };
                if value.is_null() || leader.byte() == 0 || too_long {
                    // SAFETY: a static NUL-terminated string.
                    leader = unsafe { Cursor::new(c"\\".as_ptr()) };
                }
                while leader.byte() != 0 {
                    // SAFETY: the 64-byte guard above; the leader is capped
                    // at 48 bytes by `too_long`.
                    unsafe { *result.add(dlen) = leader.byte() };
                    dlen += 1;
                    leader = leader.skip(1);
                }
                src = src.skip(len);
                continue;
            }
        }

        // Remove CTRL-V and take the next character literally. On the "from"
        // side a trailing CTRL-V is kept, on the "to" side it is dropped, so
        // that ":map xx ^V" maps xx to nothing. Without 'B' in 'cpoptions' a
        // backslash does the same job.
        let quoted = src.byte();
        if c_int::from(quoted) == Ctrl_V || (do_backslash && quoted == b'\\' as c_char) {
            src = src.skip(1);
            if src > end {
                if flags & REPTERM_FROM_PART != 0 {
                    // SAFETY: the 64-byte guard above.
                    unsafe { *result.add(dlen) = quoted };
                    dlen += 1;
                }
                break;
            }
        }

        // Copy one whole character, hiding a literal K_SPECIAL byte.
        let (at, left) = (src.raw(), end.gap(src) as c_int + 1);
        // SAFETY: `at` is inside the caller's buffer with `left` bytes left.
        let char_len = unsafe { utfc_ptr2len_len(at, left) };
        for _ in 0..char_len {
            if src.byte() == K_SPECIAL as u8 as c_char {
                let escaped = [K_SPECIAL as u8, KS_SPECIAL as u8, KE_FILLER as u8];
                // SAFETY: the 64-byte guard above.
                dlen = unsafe { put_bytes(result, dlen, &escaped) };
            } else {
                // SAFETY: as above.
                unsafe { *result.add(dlen) = src.byte() };
                dlen += 1;
            }
            src = src.skip(1);
        }
    }
    // SAFETY: `dlen` never passed `buf_len - 1`; see the guard above and the
    // six-bytes-per-character sizing of an allocated buffer.
    unsafe { *result.add(dlen) = 0 };

    if allocated {
        // SAFETY: `result` is this function's own allocation, and `bufp` is
        // the caller's writable out-parameter.
        unsafe { *bufp = xrealloc(result.cast(), dlen + 1).cast() };
    }
    // SAFETY: as above -- `bufp` is readable.
    unsafe { *bufp }
}

/// Append `c` to `s`, escaping a literal `K_SPECIAL` byte the way the
/// typeahead buffer needs, and answer a pointer past what was written.
///
/// # Safety
/// `s` must have room for `MB_MAXBYTES + 1` bytes.
pub unsafe fn add_char2buf(c: c_int, s: *mut c_char) -> *mut c_char {
    let mut encoded = [0u8; MB_MAXBYTES + 1];
    let dst = encoded.as_mut_ptr().cast();
    // SAFETY: `encoded` is a live local with room for any one character.
    let len = unsafe { utf_char2bytes(c, dst) } as usize;
    let escaped = [K_SPECIAL as u8, KS_SPECIAL as u8, KE_FILLER as u8];
    let mut at = 0;
    for &byte in &encoded[..len] {
        let written: &[u8] = if c_int::from(byte) == K_SPECIAL {
            &escaped
        } else {
            slice::from_ref(&byte)
        };
        // SAFETY: the caller's promise -- room for the escaped character.
        at = unsafe { put_bytes(s, at, written) };
    }
    // SAFETY: as above -- `at` is what was just written into `s`.
    unsafe { s.add(at) }
}

/// A copy of `p` with every literal `K_SPECIAL` byte escaped, so the result
/// can go into the typeahead buffer. The caller owns the allocation.
///
/// # Safety
/// `p` must point at a NUL-terminated string.
pub unsafe fn vim_strsave_escape_ks(p: *mut c_char) -> *mut c_char {
    // Room for three times as much, four in case of an illegal utf-8 byte:
    // 0xc0 -> 0xc3 - 0x80 -> 0xc3 K_SPECIAL KS_SPECIAL KE_FILLER.
    // SAFETY: the caller's promise -- `p` is NUL-terminated, so `strlen`
    // measures it and the NUL stops the walk below.
    let res = unsafe { xmalloc(cstr::bytes_at(p).len() * 4 + 1) }.cast::<c_char>();
    let mut dst = res;
    // SAFETY: as above.
    let mut src = unsafe { Cursor::new(p) };
    while src.byte() != 0 {
        if c_int::from(src.byte() as u8) == K_SPECIAL && src.at(1) != 0 && src.at(2) != 0 {
            // Copy a special key unchanged.
            // SAFETY: the three bytes were just read, and `res` was sized at
            // four times the input.
            unsafe { ptr::copy_nonoverlapping(src.raw(), dst, 3) };
            src = src.skip(3);
            dst = dst.wrapping_add(3);
        } else {
            // Add the character, possibly multi-byte, escaping K_SPECIAL.
            // Careful: it can be an illegal byte.
            // SAFETY: as above -- a character inside the caller's string,
            // and `res` has room for its escaped form.
            let (decoded, len) = unsafe { (utf_ptr2char(src.raw()), utf_ptr2len(src.raw())) };
            // SAFETY: as above.
            dst = unsafe { add_char2buf(decoded, dst) };
            src = src.skip(len as isize);
        }
    }
    // SAFETY: `res` was sized to hold the NUL as well.
    unsafe { *dst = 0 };
    res
}

/// Undo [`vim_strsave_escape_ks`], in place.
///
/// # Safety
/// `p` must point at a NUL-terminated, writable string.
pub unsafe fn vim_unescape_ks(p: *mut c_char) {
    // SAFETY: the caller's promise -- `p` is NUL-terminated. `src[1]` is
    // read only when `src[0]` is not the NUL and `src[2]` only when `src[1]`
    // is not either, so the walk never passes the terminator.
    let mut src = unsafe { Cursor::new(p) };
    let mut dst = p.cast::<u8>();
    while src.byte() != 0 {
        if c_int::from(src.u8_at(0)) == K_SPECIAL
            && c_int::from(src.u8_at(1)) == KS_SPECIAL
            && c_int::from(src.u8_at(2)) == KE_FILLER
        {
            // SAFETY: `dst` never passes `src`, so it stays inside `p`.
            unsafe { *dst = K_SPECIAL as u8 };
            dst = dst.wrapping_add(1);
            src = src.skip(3);
        } else {
            // SAFETY: as above.
            unsafe { *dst = src.u8_at(0) };
            dst = dst.wrapping_add(1);
            src = src.skip(1);
        }
    }
    // SAFETY: as above -- the result is never longer than the input.
    unsafe { *dst = 0 };
}
