//! `<>` key notation: the spelling of a key as text, and its encoding as
//! bytes.
//!
//! A key that is not a printable character travels through the editor as a
//! negative `c_int` and through a byte stream as `K_SPECIAL` plus two bytes;
//! modifiers ride along as a `K_SPECIAL KS_MODIFIER <bits>` prefix. This
//! module is the two translations between that and the text a user writes:
//! [`special_key_at`] and [`replace_termcodes`] read `<C-S-Up>`,
//! [`get_special_key_name`] writes it.
//!
//! The codes are in [`codes`] and the tables that name them in [`tables`];
//! everything here is about the notation, over `&[u8]`.
//!
//! Two escaping helpers live here for want of a better home:
//! [`escape_ks_bytes`] and [`unescape_ks_in`], which hide and reveal a
//! literal 0x80 byte in text that is about to be, or has just been, treated
//! as a key sequence.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::Str2NrBases;
use crate::cstr;
use crate::narrow::number_as_int;
use crate::strings::has_char;
use crate::types::Failed;
use crate::types::Key;
use core::ffi::{CStr, c_int};

use crate::ascii::{ascii_isdigit, ascii_isident};
use crate::charset::{str2nr_in, transchar, vim_isprintc};
use crate::eval::vars::var_string_value;
use crate::mbyte::MB_MAXBYTES;
use crate::mbyte::{char_at, char_len, cluster_len, encode_char};
use crate::message::emsg;
use crate::message::{E_INVARG, E_USINGSID};
use crate::option::defs::CpoFlag;
use crate::os::cshim::gettext;
use crate::runtime::state::current_sctx;
use crate::types::ScriptId;

mod codes;
pub use self::codes::*;
mod tables;
pub use self::tables::*;

/// The key code of a `KS_EXTRA` key — everything with no termcap name of its
/// own, which is most of what a modern terminal sends.
const fn extra(ke: KeyExtra) -> c_int {
    termcap(KS_EXTRA, ke)
}

/// The key code of a two-character termcap name (C's `TERMCAP2KEY`).
const fn termcap(first: u8, second: u8) -> c_int {
    -c_int::from_le_bytes([first, second, 0, 0])
}

/// C's `TOUPPER_ASC`: upper-case an ASCII letter and leave everything else —
/// a byte above 0x7f included — alone.
fn to_upper_ascii(key: Key) -> Key {
    match key.as_byte() {
        Some(b) if b.is_ascii_lowercase() => Key::byte(b.to_ascii_uppercase()),
        _ => key,
    }
}

/// C's `ASCII_ISALPHA`, which tests the *unsigned* value and so says no to
/// anything outside 7-bit ASCII.
fn is_ascii_alpha(key: Key) -> bool {
    key.as_byte().is_some_and(|b| b.is_ascii_alphabetic())
}

/// Fold `modifiers` into `key` where the terminal has a code for the
/// combination: `<S-Up>` is a key of its own, not `Up` plus a shift bit.
/// `modifiers` is left holding whatever could not be folded in.
pub fn simplify_key(key: Key, modifiers: &mut ModMask) -> Key {
    if !modifiers.has(ModMask::SHIFT | ModMask::CTRL) {
        return key;
    }
    // TAB is the one key with a shifted code but no unshifted termcap row.
    if key == Key::TAB && modifiers.has(ModMask::SHIFT) {
        modifiers.clear(ModMask::SHIFT);
        return SpecialKey::STab.into();
    }
    match simplify(key, *modifiers) {
        Some((simplified, left)) => {
            *modifiers = left;
            simplified
        }
        None => key,
    }
}

/// The inverse of [`simplify_key`]: a special key that is itself a shifted or
/// ctrl'ed code (`<S-Del>`, `<C-Left>`) becomes the plain key, with the
/// modifier it carried added to `modifiers`. Anything else comes back as is.
pub(crate) fn unsimplify_key(key: Key, modifiers: ModMask) -> (Key, ModMask) {
    if key.is_special()
        && let Some((plain, bit)) = unshift(key)
    {
        return (plain, modifiers | bit);
    }
    (key, modifiers)
}

/// [`simplify_key`] applied to the global `mod_mask`.
pub(crate) fn simplify_mod_mask(key: Key) -> Key {
    let mut modifiers = crate::getchar::state::MOD_MASK.get();
    let simplified = simplify_key(key, &mut modifiers);
    crate::getchar::state::MOD_MASK.set(modifiers);
    simplified
}

/// `<xKey>` to `<Key>`: the DEC and vt100 spellings of keys that also have an
/// ordinary termcap name.
fn handle_x_keys(key: Key) -> Key {
    let plain = match key.special() {
        Some(SpecialKey::Xup) => SpecialKey::Up,
        Some(SpecialKey::Xdown) => SpecialKey::Down,
        Some(SpecialKey::Xleft) => SpecialKey::Left,
        Some(SpecialKey::Xright) => SpecialKey::Right,
        Some(SpecialKey::Xhome | SpecialKey::Zhome) => SpecialKey::Home,
        Some(SpecialKey::Xend | SpecialKey::Zend) => SpecialKey::End,
        Some(SpecialKey::Xf1) => SpecialKey::F1,
        Some(SpecialKey::Xf2) => SpecialKey::F2,
        Some(SpecialKey::Xf3) => SpecialKey::F3,
        Some(SpecialKey::Xf4) => SpecialKey::F4,
        Some(SpecialKey::SXf1) => SpecialKey::SF1,
        Some(SpecialKey::SXf2) => SpecialKey::SF2,
        Some(SpecialKey::SXf3) => SpecialKey::SF3,
        Some(SpecialKey::SXf4) => SpecialKey::SF4,
        _ => return key,
    };
    plain.into()
}

/// A `<...>` key spelling, NUL-terminated within its own storage.
///
/// Upstream answers a pointer into one static buffer, valid only until the
/// next call, which is why `str2special_arena` has to measure and copy in
/// two passes and why two callers in one test tread on each other.
pub(crate) type SpecialKeyName = [u8; MAX_KEY_NAME_LEN + 1];

/// The `<...>` spelling of `key` with `modifiers` held down.
pub fn get_special_key_name(key: Key, modifiers: ModMask) -> SpecialKeyName {
    let mut key = key;
    let mut modifiers = modifiers;
    // A key that stands for a normal character.
    if key.is_special() && termcap_name(key)[0] == KS_KEY {
        key = Key::byte(termcap_name(key)[1]);
    }
    // A shifted or ctrl'ed special key becomes the plain key and a bit.
    (key, modifiers) = unsimplify_key(key, modifiers);

    let mut name = name_of_code(key);
    // Not a known key and not printable: try to read modifiers off the byte.
    // (A one-byte character is below 0x80, so the Alt half never applies; it
    // is upstream's, kept as it reads.)
    if let Some(mut byte) = key.as_byte().filter(|&b| b > 0 && key.utf8_len() == 1) {
        if name.is_none()
            && (!vim_isprintc(c_int::from(byte)) || byte & 0x7f == b' ')
            && byte & 0x80 != 0
        {
            byte &= 0x7f;
            modifiers |= ModMask::ALT;
            name = name_of_code(Key::byte(byte));
        }
        if name.is_none() && !vim_isprintc(c_int::from(byte)) && byte < b' ' {
            byte += b'@';
            modifiers |= ModMask::CTRL;
        }
        key = Key::byte(byte);
    }

    let mut out: SpecialKeyName = [0; MAX_KEY_NAME_LEN + 1];
    out[0] = b'<';
    let mut at = 1;
    for letter in printed_modifiers(modifiers) {
        out[at] = letter;
        out[at + 1] = b'-';
        at += 2;
    }
    match name {
        // An unnamed special key prints as its termcap name.
        None if key.is_special() => {
            let code = termcap_name(key);
            out[at..at + 4].copy_from_slice(&[b't', b'_', code[0], code[1]]);
            at += 4;
        }
        // Not a special key at all: only modifiers, printed directly.
        None => {
            let len = key.utf8_len();
            match key.as_byte() {
                Some(byte) if len == 1 && vim_isprintc(key.code()) => {
                    out[at] = byte;
                    at += 1;
                }
                _ if len > 1 => {
                    let mut wide = [0u8; MB_MAXBYTES + 1];
                    let len = encode_char(key.code(), &mut wide);
                    out[at..at + len].copy_from_slice(&wide[..len]);
                    at += len;
                }
                _ => {
                    let display = transchar(key.code());
                    let shown = cstr::in_bytes(&display).to_bytes();
                    out[at..at + shown.len()].copy_from_slice(shown);
                    at += shown.len();
                }
            }
        }
        // A named key, as long as the name fits.
        Some(name) => {
            if name.len() + at + 2 <= MAX_KEY_NAME_LEN {
                out[at..at + name.len()].copy_from_slice(name.as_bytes());
                at += name.len();
            }
        }
    }
    out[at] = b'>';
    out
}

/// The bytes one key is encoded as, held inline: room for a modifier prefix
/// and a character whose every byte is an escaped `K_SPECIAL`.
#[derive(Debug)]
pub struct KeyBytes {
    bytes: [u8; 3 * MB_MAXBYTES + 4],
    len: usize,
}

impl KeyBytes {
    #[inline]
    const fn new() -> Self {
        KeyBytes {
            bytes: [0; 3 * MB_MAXBYTES + 4],
            len: 0,
        }
    }

    #[inline]
    fn extend(&mut self, bytes: &[u8]) {
        self.bytes[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
    }

    /// The encoding.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

/// The byte sequence for `key` with `modifiers` held down: how a key is
/// encoded in a string.
///
/// With `escape_ks` set, a literal 0x80 byte inside the character is escaped
/// the way the typeahead buffer needs.
pub fn special_key_bytes(key: Key, modifiers: ModMask, escape_ks: bool) -> KeyBytes {
    let mut out = KeyBytes::new();
    if !modifiers.is_empty() {
        out.extend(&[K_SPECIAL, KS_MODIFIER, modifiers.byte()]);
    }
    if key.is_special() {
        let code = termcap_name(key);
        out.extend(&[K_SPECIAL, code[0], code[1]]);
    } else if escape_ks {
        escape_char_into(key, &mut out);
    } else {
        let mut encoded = [0u8; MB_MAXBYTES + 1];
        let len = encode_char(key.code(), &mut encoded);
        out.extend(&encoded[..len]);
    }
    out
}

/// [`special_key_bytes`], appended to `out`.
pub fn push_special_key(key: Key, modifiers: ModMask, escape_ks: bool, out: &mut Vec<u8>) {
    out.extend_from_slice(special_key_bytes(key, modifiers, escape_ks).as_bytes());
}

/// The length of the character (with its composing marks) at `text[i]`,
/// zero on a NUL or at the end -- `utfc_ptr2len`'s answer for a NUL.
fn cluster_len_at(text: &[u8], i: usize) -> usize {
    match text.get(i..) {
        Some(rest @ [first, ..]) if *first != 0 => cluster_len(rest),
        _ => 0,
    }
}

/// C's `STRNICMP(p, lit, strlen(lit)) == 0` over a slice: does `text` open
/// with `lit`, ASCII case ignored?
fn opens_with_ignoring_case(text: &[u8], lit: &[u8]) -> bool {
    text.get(..lit.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(lit))
}

/// One `<>` name at the start of `text`: the key code, the modifiers that
/// could not be folded into it, and how many bytes of `text` the name took.
/// `Ok(None)` when `text` does not start with a name this understands, and
/// `Err` when it starts with a `<Char->` name whose number is missing, E474
/// said -- which every caller then reads as no name, keeping the text.
///
/// The end of `text` reads as a NUL, which is where the pointer form's
/// readers stopped. `did_simplify`, when given, is set if `FSK_SIMPLIFY`
/// collapsed a Ctrl modifier into the key.
pub fn special_key_at(
    text: &[u8],
    flags: c_int,
    did_simplify: Option<&mut bool>,
) -> Result<Option<(Key, ModMask, usize)>, Failed> {
    let at = |i: usize| cstr::byte_at(text, i);
    // The offset of the last byte; the walk below never steps past it.
    let Some(end) = text.len().checked_sub(1) else {
        return Ok(None);
    };
    // How many bytes past `bp` the last one is -- the C's `end - bp`, which
    // is never asked once `bp` has passed it.
    let gap = |bp: usize| end.saturating_sub(bp);
    let in_string = flags & FSK_IN_STRING != 0;
    let mut src = 0;
    if at(src) != b'<' {
        return Ok(None);
    }
    if at(src + 1) == b'*' {
        src += 1; // <*xxx>: do not simplify
    }

    // Find the end of the modifier list.
    let mut last_dash = src;
    let mut bp = src + 1;
    while bp <= end && (at(bp) == b'-' || ascii_isident(c_int::from(at(bp)))) {
        if at(bp) == b'-' {
            last_dash = bp;
            if bp < end {
                let len = cluster_len_at(text, bp + 1);
                // Anything is accepted, as in <C-?>. In a string <C-"> and
                // <M-"> are not, because " ends the string; <M-\"> works.
                if gap(bp) > len && !(in_string && at(bp + 1) == b'"') && at(bp + len + 1) == b'>' {
                    bp += len;
                } else if gap(bp) > 2
                    && in_string
                    && at(bp + 1) == b'\\'
                    && at(bp + 2) == b'"'
                    && at(bp + 3) == b'>'
                {
                    bp += 2;
                }
            }
        }
        if gap(bp) > 3 && at(bp) == b't' && at(bp + 1) == b'_' {
            bp += 3; // skip t_xx; xx may be '-' or '>'
        } else if gap(bp) > 4 && opens_with_ignoring_case(&text[bp..], b"char-") {
            let digits = str2nr_in(&text[bp + 5..], Str2NrBases::ALL, true).len;
            if digits == 0 {
                emsg(gettext(E_INVARG));
                return Err(Failed);
            }
            bp += digits + 5;
            break;
        }
        bp += 1;
    }

    if bp > end || at(bp) != b'>' {
        return Ok(None);
    }
    let end_of_name = bp + 1;

    // Which modifiers are given?
    let mut modifiers = ModMask::NONE;
    for &byte in &text[src + 1..last_dash.max(src + 1)] {
        if byte != b'-' {
            let bit = name_to_mod_mask(byte);
            if bit.is_empty() {
                return Ok(None); // illegal modifier name
            }
            modifiers |= bit;
        }
    }

    let after_dash = &text[last_dash + 1..];
    let mut key = if opens_with_ignoring_case(after_dash, b"char-")
        && ascii_isdigit(c_int::from(at(last_dash + 6)))
    {
        // <Char-123>, <Char-033> or <Char-0x33>.
        let number = str2nr_in(&text[last_dash + 6..], Str2NrBases::ALL, true);
        if number.len == 0 {
            emsg(gettext(E_INVARG));
            return Err(Failed);
        }
        // Upstream's `(int)` of the number: the low 32 bits.
        Key::from_code(number_as_int(number.magnitude.cast_signed()))
    } else {
        // A single-letter modifier, or a special key name.
        let (len, off) = if in_string && at(last_dash + 1) == b'\\' && at(last_dash + 2) == b'"' {
            // In a double-quoted string, `"` is written `\"`.
            (2, 2)
        } else {
            (cluster_len_at(text, last_dash + 1), 1)
        };
        let name = &text[last_dash + off..];
        if !modifiers.is_empty() && at(last_dash + len + 1) == b'>' {
            Key::from_code(char_at(name))
        } else {
            let code = special_key_code(name);
            if flags & FSK_KEEP_X_KEY == 0 {
                handle_x_keys(code)
            } else {
                code
            }
        }
    };

    // The name lookup answers NUL for a name it does not know.
    if key.is_nul() {
        return Ok(None);
    }
    // Only keep a modifier that no key code already includes.
    key = simplify_key(key, &mut modifiers);
    if flags & FSK_KEYCODE == 0 {
        // No key code wanted: answer with the single-byte code.
        if key == SpecialKey::Bs {
            key = Key::BS;
        } else if key == SpecialKey::Del || key == SpecialKey::Kdel {
            key = Key::DEL;
        }
    }
    // A normal character with a modifier: try to make one byte of the two,
    // Alt and Meta excepted.
    if !key.is_special() {
        let simplify = flags & FSK_SIMPLIFY != 0;
        key = extract_modifiers(key, &mut modifiers, simplify, did_simplify);
    }
    Ok(Some((key, modifiers, end_of_name)))
}

/// Append the encoding of the `<>` name at the start of `text` to `out` and
/// answer how many bytes of `text` the name took. `Ok(None)`, appending
/// nothing, when `text` does not start with a name this understands; `Err`
/// as [`special_key_at`]'s.
pub fn trans_special_into(
    text: &[u8],
    flags: c_int,
    escape_ks: bool,
    out: &mut Vec<u8>,
) -> Result<Option<usize>, Failed> {
    let Some((key, modifiers, used)) = special_key_at(text, flags, None)? else {
        return Ok(None);
    };
    push_special_key(key, modifiers, escape_ks, out);
    Ok(Some(used))
}

/// Fold the modifiers a single byte can carry into `key`: `Shift-a` becomes
/// `A`, `Ctrl-@` becomes `<Nul>`. Alt and Meta are never folded.
///
/// With `simplify` clear the Ctrl half is skipped, which is how a caller
/// keeps both spellings of `<C-H>`; `did_simplify` says whether it happened.
///
fn extract_modifiers(
    key: Key,
    modifiers: &mut ModMask,
    simplify: bool,
    did_simplify: Option<&mut bool>,
) -> Key {
    let mut key = key;

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
    let ctrl_able = |key: Key| key.as_byte().is_some_and(|b| (b'?'..=b'_').contains(&b));
    if simplify && modifiers.has(ModMask::CTRL) && (ctrl_able(key) || is_ascii_alpha(key)) {
        let upper = to_upper_ascii(key).as_byte().expect("an ASCII character");
        key = Key::byte(upper ^ 0x40);
        modifiers.clear(ModMask::CTRL);
        if key.is_nul() {
            key = SpecialKey::Zero.into(); // <C-@> is <Nul>
        }
        if let Some(did_simplify) = did_simplify {
            *did_simplify = true;
        }
    }
    key
}

/// The code of the special key whose name `text` starts with, or 0 when
/// there is no such key.
///
/// The name ends at the first non-identifier byte or at the end of `text`,
/// so a caller may hand this the middle of a larger string. A `t_xx` name is
/// a raw termcap code and never reaches the table.
pub fn special_key_code(text: &[u8]) -> Key {
    let at = |i: usize| cstr::byte_at(text, i);
    if at(0) == b't' && at(1) == b'_' && at(2) != 0 && at(3) != 0 {
        return termcap_key([at(2), at(3)]);
    }
    let len = text
        .iter()
        .take_while(|&&b| ascii_isident(c_int::from(b)))
        .count();
    code_for_name(&text[..len])
}

/// Which button a mouse key is about, and whether it was a click or a drag.
/// Read off the key's second termcap byte alone, as upstream's
/// `KEY2TERMCAP1` is; a key that is not a mouse event answers button 0,
/// neither.
pub(crate) fn mouse_button(key: Key) -> (c_int, bool, bool) {
    let code = termcap_name(key)[1];
    match mouse_event(code) {
        Some(event) => (event.button, event.is_click, event.is_drag),
        None => (0, false, false),
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
/// Also handles `<C-v>` and backslash escapes (per `cpo`),
/// `<Leader>`/`<LocalLeader>` expansion, `<SID>` script-id substitution, and —
/// unless `REPTERM_NO_SIMPLIFY` — simplifications such as `<C-H>` to 0x08.
///
/// `sid_arg` is the script id `<SID>` stands for, or 0 to take the current
/// one. Only `CpoFlag::BSLASH` is read out of `cpo`.
///
/// `room` is the fixed buffer a mapping's lhs is read into (128 bytes): the
/// answer is `None` when the translation would not leave 64 bytes of it free
/// at the start of a step, which is upstream's test. With no `room` the
/// answer is never `None`. `did_simplify` is set when a Ctrl modifier was
/// folded into a key.
pub fn replace_termcodes(
    from: &[u8],
    sid_arg: ScriptId,
    flags: c_int,
    mut did_simplify: Option<&mut bool>,
    cpo: &CStr,
    room: Option<usize>,
) -> Option<Vec<u8>> {
    let at = |i: usize| cstr::byte_at(from, i);
    // How many bytes past `i` the last one is -- the C's `end - src`.
    let gap = |i: usize| from.len().saturating_sub(i + 1);
    let opens = |i: usize, lit: &[u8]| {
        from.get(i..i + lit.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(lit))
    };
    // A backslash is a special character unless 'cpoptions' contains B.
    let do_backslash = !has_char(cpo, CpoFlag::BSLASH.as_c_int());
    let do_special = flags & REPTERM_NO_SPECIAL == 0;
    let mut out: Vec<u8> = Vec::with_capacity(room.unwrap_or(from.len() * 6 + 1));

    let mut src = 0;
    while src < from.len() {
        if room.is_some_and(|room| out.len() + 64 > room) {
            return None;
        }
        // Check for special <> keycodes, like "<C-S-LeftMouse>". Every name
        // below opens with a `<`, so nothing else needs the comparisons.
        let opens_name = at(src) == b'<';
        if do_special
            && opens_name
            && (flags & REPTERM_DO_LT != 0
                || (gap(src) >= 3 && from.get(src..src + 4) != Some(b"<lt>")))
        {
            // <SID>Func becomes K_SNR <script-nr> _Func, which is how a
            // script-local function's name is spelled.
            if gap(src) >= 4 && opens(src, b"<SID>") {
                if sid_arg < 0 || (sid_arg == 0 && current_sctx.get().sc_sid <= 0) {
                    emsg(gettext(E_USINGSID));
                } else {
                    let sid = if sid_arg != 0 {
                        sid_arg
                    } else {
                        current_sctx.get().sc_sid
                    };
                    src += 5;
                    out.extend_from_slice(&[K_SPECIAL, KS_EXTRA, KE_SNR]);
                    out.extend_from_slice(sid.to_string().as_bytes());
                    out.push(b'_');
                    continue;
                }
            }

            let simplify = if flags & REPTERM_NO_SIMPLIFY != 0 {
                0
            } else {
                FSK_SIMPLIFY
            };
            let how = FSK_KEYCODE | simplify;
            if let Ok(Some((key, modifiers, used))) =
                special_key_at(&from[src..], how, did_simplify.as_deref_mut())
            {
                push_special_key(key, modifiers, true, &mut out);
                src += used;
                continue;
            }
        }

        if do_special && opens_name {
            // <Leader> and <LocalLeader> take the value of "mapleader" and
            // "maplocalleader"; a backslash stands in when either is unset.
            let (len, value) = if gap(src) >= 7 && opens(src, b"<Leader>") {
                (8, var_string_value(b"g:mapleader"))
            } else if gap(src) >= 12 && opens(src, b"<LocalLeader>") {
                (13, var_string_value(b"g:maplocalleader"))
            } else {
                (0, None)
            };
            if len != 0 {
                // Up to 8 * 6 characters of "mapleader" are allowed.
                let leader: &[u8] = match &value {
                    Some(value) if !value.is_empty() && value.len() <= 8 * 6 => value,
                    _ => b"\\",
                };
                out.extend_from_slice(leader);
                src += len;
                continue;
            }
        }

        // Remove CTRL-V and take the next character literally. On the "from"
        // side a trailing CTRL-V is kept, on the "to" side it is dropped, so
        // that ":map xx ^V" maps xx to nothing. Without 'B' in 'cpoptions' a
        // backslash does the same job.
        let quoted = at(src);
        if Key::byte(quoted) == Key::ctrl(b'V') || (do_backslash && quoted == b'\\') {
            src += 1;
            if src >= from.len() {
                if flags & REPTERM_FROM_PART != 0 {
                    out.push(quoted);
                }
                break;
            }
        }

        // Copy one whole character, hiding a literal K_SPECIAL byte. A NUL
        // is a byte of its own, copied like any other.
        let len = cluster_len(&from[src..]);
        for &byte in &from[src..src + len] {
            if byte == K_SPECIAL {
                out.extend_from_slice(&[K_SPECIAL, KS_SPECIAL, KE_FILLER]);
            } else {
                out.push(byte);
            }
        }
        src += len;
    }
    Some(out)
}

/// `c`'s UTF-8 bytes, with a literal `K_SPECIAL` byte escaped the way the
/// typeahead buffer needs.
fn escape_char_into(c: Key, out: &mut KeyBytes) {
    let mut encoded = [0u8; MB_MAXBYTES + 1];
    let len = encode_char(c.code(), &mut encoded);
    for &byte in &encoded[..len] {
        if byte == K_SPECIAL {
            out.extend(&[K_SPECIAL, KS_SPECIAL, KE_FILLER]);
        } else {
            out.extend(&[byte]);
        }
    }
}

/// Append `c` to `out`, escaping a literal `K_SPECIAL` byte the way the
/// typeahead buffer needs.
pub fn push_char_escaped(c: Key, out: &mut Vec<u8>) {
    let mut escaped = KeyBytes::new();
    escape_char_into(c, &mut escaped);
    out.extend_from_slice(escaped.as_bytes());
}

/// `text` with every literal `K_SPECIAL` byte escaped, up to its first NUL
/// (and up to a NUL its decoding produces),
/// so the result can go into the typeahead buffer. A special key already
/// spelled out (`K_SPECIAL` and two more bytes) is copied unchanged, and an
/// illegal byte is taken as the character of the same number.
pub fn escape_ks_bytes(text: &[u8]) -> Vec<u8> {
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    // Room for three times as much, four in case of an illegal utf-8 byte:
    // 0xc0 -> 0xc3 - 0x80 -> 0xc3 K_SPECIAL KS_SPECIAL KE_FILLER.
    let mut out = Vec::with_capacity(text.len() * 4 + 1);
    let mut src = 0;
    while src < text.len() {
        let rest = &text[src..];
        // ASCII is itself, and is all most text is.
        if rest[0] < 0x80 {
            out.push(rest[0]);
            src += 1;
            continue;
        }
        if rest[0] == K_SPECIAL && rest.len() > 2 {
            // Copy a special key unchanged.
            out.extend_from_slice(&rest[..3]);
            src += 3;
        } else {
            // Add the character, possibly multi-byte, escaping K_SPECIAL.
            push_char_escaped(Key::from_code(char_at(rest)), &mut out);
            src += char_len(rest);
        }
    }
    // An overlong NUL (`C0 80`) decodes to 0 and is written as one: the
    // string a C reader sees ends there, and so does this one.
    out.truncate(out.iter().position(|&b| b == 0).unwrap_or(out.len()));
    out
}

/// Undo [`escape_ks_bytes`] in place, answering the new length; the bytes
/// past it are left as they were. Never longer than what it was given. All
/// of `text` is the text: a caller holding a C string's buffer hands over the
/// bytes before its NUL.
pub fn unescape_ks_in(text: &mut [u8]) -> usize {
    let end = text.len();
    let (mut src, mut dst) = (0, 0);
    while src < end {
        if text[src] == K_SPECIAL
            && src + 2 < end
            && text[src + 1] == KS_SPECIAL
            && text[src + 2] == KE_FILLER
        {
            text[dst] = K_SPECIAL;
            src += 3;
        } else {
            text[dst] = text[src];
            src += 1;
        }
        dst += 1;
    }
    dst
}

/// [`unescape_ks_in`] of a copy of `text`.
pub(crate) fn unescape_ks(text: &[u8]) -> Vec<u8> {
    let mut out = text.to_vec();
    let len = unescape_ks_in(&mut out);
    out.truncate(len);
    out
}

pub type KeyExtra = u8;
