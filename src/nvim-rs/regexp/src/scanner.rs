//! Character scanner for the regex parser.
//!
//! This module provides the lexical scanner for parsing regular expressions.
//! It handles magic characters, backslash escapes, and UTF-8 character iteration.

use std::ffi::{c_char, c_int};

// =============================================================================
// Constants
// =============================================================================

/// Magic character offset (negative chars are magic)
const MAGIC_OFFSET: c_int = 256;

/// Magic modes for regex patterns
const MAGIC_NONE: c_int = 1; // \V very nomagic
const MAGIC_OFF: c_int = 2; // \M or magic off
const MAGIC_ON: c_int = 3; // \m or magic (default)
const MAGIC_ALL: c_int = 4; // \v very magic

/// ASCII control characters
const CAR: c_int = 13; // Carriage return
const TAB: c_int = 9; // Tab
const ESC: c_int = 27; // Escape
const BS: c_int = 8; // Backspace

/// Abbreviation characters after '\'
const REGEXP_ABBR: &[u8] = b"nrtebdoxuU";

/// META_flags table: characters that may be magic when preceded by backslash.
/// Index by ASCII character code (0-126). Value of 1 means the character can be meta.
#[rustfmt::skip]
static META_FLAGS: [u8; 127] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//                 %  &     (  )  *  +        .
    0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0,
//     1  2  3  4  5  6  7  8  9        <  =  >  ?
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
//  @  A     C  D     F     H  I     K  L  M     O
    1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1,
//  P        S     U  V  W  X     Z  [           _
    1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1,
//     a     c  d     f     h  i     k  l  m  n  o
    0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1,
//  p        s     u  v  w  x     z  {  |     ~
    1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1,
];

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Parse state accessors
    fn nvim_parse_get_regparse() -> *mut c_char;
    fn nvim_parse_set_regparse(p: *mut c_char);
    fn nvim_parse_get_prevchr_len() -> c_int;
    fn nvim_parse_set_prevchr_len(len: c_int);
    fn nvim_parse_get_curchr() -> c_int;
    fn nvim_parse_set_curchr(c: c_int);
    fn nvim_parse_get_prevchr() -> c_int;
    fn nvim_parse_set_prevchr(c: c_int);
    fn nvim_parse_get_prevprevchr() -> c_int;
    fn nvim_parse_set_prevprevchr(c: c_int);
    fn nvim_parse_get_nextchr() -> c_int;
    fn nvim_parse_set_nextchr(c: c_int);
    fn nvim_parse_get_at_start() -> c_int;
    fn nvim_parse_set_at_start(v: c_int);
    fn nvim_parse_get_prev_at_start() -> c_int;
    fn nvim_parse_set_prev_at_start(v: c_int);
    fn nvim_parse_get_reg_magic() -> c_int;

    // UTF-8 functions
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
}

// =============================================================================
// Magic Functions
// =============================================================================

/// Convert an ASCII character to its magic form.
#[inline]
const fn magic(x: c_int) -> c_int {
    x - MAGIC_OFFSET
}

/// Convert a magic character back to its ASCII value.
#[inline]
const fn un_magic(x: c_int) -> c_int {
    x + MAGIC_OFFSET
}

/// Check if a character is magic (negative value).
#[inline]
const fn is_magic(x: c_int) -> bool {
    x < 0
}

/// Remove magic from a character.
#[inline]
const fn no_magic(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        x
    }
}

/// Toggle the magic state of a character.
#[inline]
const fn toggle_magic(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        magic(x)
    }
}

/// Translate '\x' to its control character, except "\n" which is Magic.
#[inline]
const fn backslash_trans(c: c_int) -> c_int {
    match c as u8 {
        b'r' => CAR,
        b't' => TAB,
        b'e' => ESC,
        b'b' => BS,
        _ => c,
    }
}

/// Check if a byte is in a byte slice.
#[inline]
fn byte_in_slice(b: u8, slice: &[u8]) -> bool {
    slice.contains(&b)
}

// =============================================================================
// Scanner Implementation
// =============================================================================

/// Initialize parsing at `str`. Sets up the scanner state.
///
/// # Safety
/// `str` must point to a valid null-terminated string.
#[inline]
pub unsafe fn initchr(str: *mut c_char) {
    nvim_parse_set_regparse(str);
    nvim_parse_set_prevchr_len(0);
    nvim_parse_set_curchr(-1);
    nvim_parse_set_prevprevchr(-1);
    nvim_parse_set_prevchr(-1);
    nvim_parse_set_nextchr(-1);
    nvim_parse_set_at_start(1); // true
    nvim_parse_set_prev_at_start(0); // false
}

/// Get the next character without advancing.
/// Handles magic characters, backslash escapes, and context-sensitive parsing.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
pub unsafe fn peekchr() -> c_int {
    // Static variable for tracking after_slash state (matches C implementation)
    static mut AFTER_SLASH: c_int = 0;

    let curchr = nvim_parse_get_curchr();
    if curchr != -1 {
        return curchr;
    }

    let regparse = nvim_parse_get_regparse();
    let reg_magic = nvim_parse_get_reg_magic();
    let at_start = nvim_parse_get_at_start() != 0;
    let prev_at_start = nvim_parse_get_prev_at_start() != 0;
    let prevchr = nvim_parse_get_prevchr();
    let prevprevchr = nvim_parse_get_prevprevchr();

    let byte = *regparse as u8;
    let mut new_curchr = byte as c_int;

    match byte {
        // Magic when 'magic' is on
        b'.' | b'[' | b'~' => {
            if reg_magic >= MAGIC_ON {
                new_curchr = magic(new_curchr);
            }
        }
        // Magic only after "\v"
        b'(' | b')' | b'{' | b'%' | b'+' | b'=' | b'?' | b'@' | b'!' | b'&' | b'|' | b'<'
        | b'>' | b'#' | b'"' | b'\'' | b',' | b'-' | b':' | b';' | b'`' | b'/' => {
            if reg_magic == MAGIC_ALL {
                new_curchr = magic(new_curchr);
            }
        }
        b'*' => {
            // * is not magic as the very first character, eg "?*ptr", when
            // after '^', eg "/^*ptr" and when after "\(", "\|", "\&".  But
            // "\(\*" is not magic, thus must be magic if "after_slash"
            if reg_magic >= MAGIC_ON
                && !at_start
                && !(prev_at_start && prevchr == magic(b'^' as c_int))
                && (AFTER_SLASH != 0
                    || (prevchr != magic(b'(' as c_int)
                        && prevchr != magic(b'&' as c_int)
                        && prevchr != magic(b'|' as c_int)))
            {
                new_curchr = magic(b'*' as c_int);
            }
        }
        b'^' => {
            // '^' is only magic as the very first character and if it's after
            // "\(", "\|", "\&' or "\n"
            if reg_magic >= MAGIC_OFF
                && (at_start
                    || reg_magic == MAGIC_ALL
                    || prevchr == magic(b'(' as c_int)
                    || prevchr == magic(b'|' as c_int)
                    || prevchr == magic(b'&' as c_int)
                    || prevchr == magic(b'n' as c_int)
                    || (no_magic(prevchr) == b'(' as c_int && prevprevchr == magic(b'%' as c_int)))
            {
                new_curchr = magic(b'^' as c_int);
                nvim_parse_set_at_start(1);
                nvim_parse_set_prev_at_start(0);
            }
        }
        b'$' => {
            // '$' is only magic as the very last char and if it's in front of
            // either "\|", "\)", "\&", or "\n"
            if reg_magic >= MAGIC_OFF {
                let mut p = regparse.add(1) as *mut u8;
                let mut is_magic_all = reg_magic == MAGIC_ALL;

                // Ignore \c \C \m \M \v \V and \Z after '$'
                while *p == b'\\' {
                    let next = *p.add(1);
                    if next == b'c'
                        || next == b'C'
                        || next == b'm'
                        || next == b'M'
                        || next == b'v'
                        || next == b'V'
                        || next == b'Z'
                    {
                        if next == b'v' {
                            is_magic_all = true;
                        } else if next == b'm' || next == b'M' || next == b'V' {
                            is_magic_all = false;
                        }
                        p = p.add(2);
                    } else {
                        break;
                    }
                }

                if *p == 0
                    || (*p == b'\\'
                        && (*p.add(1) == b'|'
                            || *p.add(1) == b'&'
                            || *p.add(1) == b')'
                            || *p.add(1) == b'n'))
                    || (is_magic_all && (*p == b'|' || *p == b'&' || *p == b')'))
                    || reg_magic == MAGIC_ALL
                {
                    new_curchr = magic(b'$' as c_int);
                }
            }
        }
        b'\\' => {
            let c = *regparse.add(1) as u8;
            if c == 0 {
                // trailing '\'
                new_curchr = b'\\' as c_int;
            } else if c <= b'~' && META_FLAGS[c as usize] != 0 {
                // META contains everything that may be magic sometimes,
                // except ^ and $ ("\^" and "\$" are only magic after
                // "\V").  We now fetch the next character and toggle its
                // magicness.  Therefore, \ is so meta-magic that it is
                // not in META.
                nvim_parse_set_curchr(-1);
                let old_prev_at_start = nvim_parse_get_prev_at_start();
                nvim_parse_set_prev_at_start(nvim_parse_get_at_start());
                nvim_parse_set_at_start(0); // be able to say "/\*ptr"
                let old_regparse = nvim_parse_get_regparse();
                nvim_parse_set_regparse(old_regparse.add(1));
                AFTER_SLASH += 1;
                let inner_chr = peekchr();
                AFTER_SLASH -= 1;
                nvim_parse_set_regparse(old_regparse);
                nvim_parse_set_at_start(nvim_parse_get_prev_at_start());
                nvim_parse_set_prev_at_start(old_prev_at_start);
                new_curchr = toggle_magic(inner_chr);
            } else if byte_in_slice(c, REGEXP_ABBR) {
                // Handle abbreviations, like "\t" for TAB -- webb
                new_curchr = backslash_trans(c as c_int);
            } else if reg_magic == MAGIC_NONE && (c == b'$' || c == b'^') {
                new_curchr = toggle_magic(c as c_int);
            } else {
                // Next character can never be (made) magic?
                // Then backslashing it won't do anything.
                new_curchr = utf_ptr2char(regparse.add(1));
            }
        }
        _ => {
            new_curchr = utf_ptr2char(regparse);
        }
    }

    nvim_parse_set_curchr(new_curchr);
    new_curchr
}

/// Eat one lexed character. Does this in a way that we can undo it.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
pub unsafe fn skipchr() {
    let regparse = nvim_parse_get_regparse();

    // peekchr() eats a backslash, do the same here
    let prevchr_len = if *regparse as u8 == b'\\' { 1 } else { 0 };
    nvim_parse_set_prevchr_len(prevchr_len);

    let mut new_prevchr_len = prevchr_len;
    if *regparse.add(prevchr_len as usize) != 0 {
        // Exclude composing chars that utfc_ptr2len does include.
        new_prevchr_len += utf_ptr2len(regparse.add(prevchr_len as usize));
    }
    nvim_parse_set_prevchr_len(new_prevchr_len);

    let new_regparse = regparse.add(new_prevchr_len as usize);
    nvim_parse_set_regparse(new_regparse);

    nvim_parse_set_prev_at_start(nvim_parse_get_at_start());
    nvim_parse_set_at_start(0);
    nvim_parse_set_prevprevchr(nvim_parse_get_prevchr());
    nvim_parse_set_prevchr(nvim_parse_get_curchr());
    nvim_parse_set_curchr(nvim_parse_get_nextchr()); // use previously unget char, or -1
    nvim_parse_set_nextchr(-1);
}

/// Skip a character while keeping the value of prev_at_start for at_start.
/// prevchr and prevprevchr are also kept.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
pub unsafe fn skipchr_keepstart() {
    let prev_at_start = nvim_parse_get_prev_at_start();
    let prevchr = nvim_parse_get_prevchr();
    let prevprevchr = nvim_parse_get_prevprevchr();

    skipchr();

    nvim_parse_set_at_start(prev_at_start);
    nvim_parse_set_prevchr(prevchr);
    nvim_parse_set_prevprevchr(prevprevchr);
}

/// Get the next character from the pattern. We know about magic and such, so
/// therefore we need a lexical analyzer.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[inline]
pub unsafe fn getchr() -> c_int {
    let chr = peekchr();
    skipchr();
    chr
}

/// Put character back. Works only once!
///
/// # Safety
/// regparse must point to a valid string and prevchr_len must be valid.
pub unsafe fn ungetchr() {
    nvim_parse_set_nextchr(nvim_parse_get_curchr());
    nvim_parse_set_curchr(nvim_parse_get_prevchr());
    nvim_parse_set_prevchr(nvim_parse_get_prevprevchr());
    nvim_parse_set_at_start(nvim_parse_get_prev_at_start());
    nvim_parse_set_prev_at_start(0);

    // Backup regparse, so that it's at the same position as before the getchr()
    let regparse = nvim_parse_get_regparse();
    let prevchr_len = nvim_parse_get_prevchr_len();
    nvim_parse_set_regparse(regparse.sub(prevchr_len as usize));
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Initialize parsing at `str`.
///
/// # Safety
/// `str` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_initchr(str: *mut c_char) {
    initchr(str);
}

/// Get the next character without advancing.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_peekchr() -> c_int {
    peekchr()
}

/// Eat one lexed character.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipchr() {
    skipchr();
}

/// Skip a character while keeping the value of prev_at_start.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_skipchr_keepstart() {
    skipchr_keepstart();
}

/// Get the next character from the pattern.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_getchr() -> c_int {
    getchr()
}

/// Put character back. Works only once!
///
/// # Safety
/// regparse must point to a valid string.
#[no_mangle]
pub unsafe extern "C" fn rs_ungetchr() {
    ungetchr();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_flags_size() {
        assert_eq!(META_FLAGS.len(), 127);
    }

    #[test]
    fn test_meta_flags_values() {
        // Check some known meta characters
        assert_eq!(META_FLAGS[b'%' as usize], 1);
        assert_eq!(META_FLAGS[b'&' as usize], 1);
        assert_eq!(META_FLAGS[b'(' as usize], 1);
        assert_eq!(META_FLAGS[b')' as usize], 1);
        assert_eq!(META_FLAGS[b'*' as usize], 1);
        assert_eq!(META_FLAGS[b'+' as usize], 1);
        assert_eq!(META_FLAGS[b'.' as usize], 1);
        assert_eq!(META_FLAGS[b'[' as usize], 1);
        assert_eq!(META_FLAGS[b'{' as usize], 1);
        assert_eq!(META_FLAGS[b'|' as usize], 1);
        assert_eq!(META_FLAGS[b'~' as usize], 1);

        // Check some non-meta characters
        assert_eq!(META_FLAGS[b'a' as usize], 1); // \a is meta
        assert_eq!(META_FLAGS[b'b' as usize], 0); // \b is not in META (it's in ABBR)
        assert_eq!(META_FLAGS[b'e' as usize], 0); // \e is not in META
        assert_eq!(META_FLAGS[b' ' as usize], 0); // space is not meta
    }

    #[test]
    fn test_magic_constants() {
        assert_eq!(MAGIC_NONE, 1);
        assert_eq!(MAGIC_OFF, 2);
        assert_eq!(MAGIC_ON, 3);
        assert_eq!(MAGIC_ALL, 4);
    }

    #[test]
    fn test_magic_functions() {
        // Test magic/un_magic roundtrip
        for c in 0..=127 {
            let m = magic(c);
            assert!(m < 0, "magic({c}) should be negative");
            assert_eq!(un_magic(m), c);
        }
    }

    #[test]
    fn test_toggle_magic() {
        let star = b'*' as c_int;
        let magic_star = magic(star);

        // Toggle regular -> magic
        assert_eq!(toggle_magic(star), magic_star);
        // Toggle magic -> regular
        assert_eq!(toggle_magic(magic_star), star);
    }

    #[test]
    fn test_no_magic() {
        let star = b'*' as c_int;
        let magic_star = magic(star);

        // Magic should be unmagicked
        assert_eq!(no_magic(magic_star), star);
        // Regular should stay the same
        assert_eq!(no_magic(star), star);
    }

    #[test]
    fn test_is_magic() {
        assert!(is_magic(-1));
        assert!(is_magic(magic(b'*' as c_int)));
        assert!(!is_magic(0));
        assert!(!is_magic(b'*' as c_int));
    }

    #[test]
    fn test_backslash_trans() {
        assert_eq!(backslash_trans(b'r' as c_int), CAR);
        assert_eq!(backslash_trans(b't' as c_int), TAB);
        assert_eq!(backslash_trans(b'e' as c_int), ESC);
        assert_eq!(backslash_trans(b'b' as c_int), BS);
        // Other characters pass through
        assert_eq!(backslash_trans(b'n' as c_int), b'n' as c_int);
        assert_eq!(backslash_trans(b'x' as c_int), b'x' as c_int);
    }

    #[test]
    fn test_byte_in_slice() {
        assert!(byte_in_slice(b'n', REGEXP_ABBR));
        assert!(byte_in_slice(b'r', REGEXP_ABBR));
        assert!(byte_in_slice(b't', REGEXP_ABBR));
        assert!(!byte_in_slice(b'a', REGEXP_ABBR));
    }
}
