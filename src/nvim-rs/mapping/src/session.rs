//! Session serialization functions for key mappings.
//!
//! Provides `put_escstr` (write an escaped key string to a FILE*)
//! and `makemap` (write map commands for current mappings to an .exrc file).

use std::ffi::{c_char, c_int};

use crate::{
    completion::{
        rs_makemap_mode_chars as makemap_mode_chars, rs_makemap_needs_cpo as makemap_needs_cpo,
        rs_makemap_should_skip as makemap_should_skip,
        rs_put_escstr_escape_type as put_escstr_escape_type,
    },
    BufHandle, MapblockHandle,
};

// =============================================================================
// Constants
// =============================================================================

const NUL: u8 = 0;
const NL: u8 = 10;
const K_SPECIAL: u8 = 0x80;
const KS_MODIFIER: u8 = 252;
/// `TO_SPECIAL` KS_SPECIAL byte
const KS_SPECIAL: u8 = 254;
const KS_ZERO: u8 = 255;
const KE_FILLER: u8 = b'X';
const OK: c_int = 1;
const FAIL: c_int = 0;
const REMAP_YES: c_int = 0;
// Ctrl-V = 22
const CTRL_V: u8 = 22;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    /// Unescape a K_SPECIAL-encoded multi-byte character.
    /// Advances `*pp` past the escaped bytes and returns a pointer to the
    /// decoded UTF-8 string (static storage), or NULL if not a multi-byte sequence.
    fn mb_unescape(pp: *mut *const c_char) -> *const c_char;

    /// Get a printable name for a special key (e.g. "<Up>").
    fn get_special_key_name(c: c_int, modifiers: c_int) -> *mut c_char;

    /// Read a UTF-8 codepoint from a byte string.
    #[link_name = "utf_ptr2char"]
    fn utf_ptr2char(p: *const c_char) -> c_int;

    /// Write a newline to the session file.
    fn put_eol(fd: *mut libc::FILE) -> c_int;

    /// Show an internal error message.
    fn iemsg(msg: *const c_char);

    // Mapping hash accessors
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
}

// =============================================================================
// Helper: TO_SPECIAL
// =============================================================================

/// Decode a K_SPECIAL two-byte sequence to a key code.
///
/// Mirrors the C macro `TO_SPECIAL(a, b)`.
#[inline]
const fn to_special(a: u8, b: u8) -> c_int {
    if a == KS_SPECIAL {
        K_SPECIAL as c_int
    } else if a == KS_ZERO {
        // termcap2key(KS_ZERO, KE_FILLER)
        -(KS_ZERO as c_int + ((KE_FILLER as c_int) << 8))
    } else {
        -(a as c_int + ((b as c_int) << 8))
    }
}

/// True when `c` is a special (negative) key code.
#[inline]
const fn is_special(c: c_int) -> bool {
    c < 0
}

// =============================================================================
// put_escstr
// =============================================================================

/// Write an escaped key string to a FILE.
///
/// `what`: 0 = :map lhs, 1 = :map rhs, 2 = :set
///
/// Returns OK on success, FAIL on I/O error.
///
/// # Safety
/// `fd` must be a valid writable FILE*. `strstart` must be a valid
/// NUL-terminated C string.
#[export_name = "put_escstr"]
pub unsafe extern "C" fn rs_put_escstr(
    fd: *mut libc::FILE,
    strstart: *const c_char,
    what: c_int,
) -> c_int {
    let mut str = strstart.cast::<u8>();

    // :map xx <Nop>
    if *str == NUL && what == 1 {
        if libc::fputs(c"<Nop>".as_ptr(), fd) < 0 {
            return FAIL;
        }
        return OK;
    }

    while *str != NUL {
        // Check for a multi-byte character which may contain escaped K_SPECIAL bytes.
        let mut cpp: *const c_char = str.cast::<c_char>();
        let p = mb_unescape(std::ptr::addr_of_mut!(cpp));
        if !p.is_null() {
            // mb_unescape advanced cpp past the consumed bytes; update str.
            str = cpp.cast::<u8>();
            // Write the unescaped bytes.
            let mut q = p.cast::<u8>();
            while *q != NUL {
                if libc::fputc(c_int::from(*q), fd) < 0 {
                    return FAIL;
                }
                q = q.add(1);
            }
            // str already points at the next unprocessed byte; continue the loop.
            continue;
        }

        let mut c = c_int::from(*str);

        // Special key codes have to be translated.
        if c == c_int::from(K_SPECIAL) && what != 2 {
            let mut modifiers: c_int = 0;
            if *str.add(1) == KS_MODIFIER {
                modifiers = c_int::from(*str.add(2));
                str = str.add(3);

                // Modifiers can apply to multi-byte characters too.
                let mut cpp2: *const c_char = str.cast::<c_char>();
                let p2 = mb_unescape(std::ptr::addr_of_mut!(cpp2));
                if p2.is_null() {
                    c = c_int::from(*str);
                } else {
                    // Retrieve codepoint from unescaped string.
                    c = utf_ptr2char(p2);
                    str = cpp2.cast::<u8>().sub(1);
                }
            }
            if c == c_int::from(K_SPECIAL) {
                c = to_special(*str.add(1), *str.add(2));
                str = str.add(2);
            }
            if is_special(c) || modifiers != 0 {
                let name = get_special_key_name(c, modifiers);
                if libc::fputs(name, fd) < 0 {
                    return FAIL;
                }
                str = str.add(1);
                continue;
            }
        }

        // A '\n' in a map command should be written as <NL>.
        // A '\n' in a set command should be written as \^V^J.
        if c == c_int::from(NL) {
            if what == 2 {
                // "\\\026\n" = backslash, Ctrl-V, newline
                if libc::fputc(c_int::from(b'\\'), fd) < 0
                    || libc::fputc(c_int::from(CTRL_V), fd) < 0
                    || libc::fputc(c_int::from(b'\n'), fd) < 0
                {
                    return FAIL;
                }
            } else if libc::fputs(c"<NL>".as_ptr(), fd) < 0 {
                return FAIL;
            }
            str = str.add(1);
            continue;
        }

        let is_first = i32::from(str == strstart.cast::<u8>());
        let esc = put_escstr_escape_type(what, c, is_first);
        let prefix_byte: Option<u8> = if esc == 1 {
            Some(b'\\')
        } else if esc == 2 {
            Some(CTRL_V)
        } else {
            None
        };
        if let Some(b) = prefix_byte {
            if libc::fputc(c_int::from(b), fd) < 0 {
                return FAIL;
            }
        }
        if libc::fputc(c, fd) < 0 {
            return FAIL;
        }
        str = str.add(1);
    }
    OK
}

// =============================================================================
// makemap
// =============================================================================

/// Write map commands for the current mappings to an .exrc file.
///
/// Returns FAIL on error, OK otherwise.
///
/// `buf`: buffer for local mappings, or null for global mappings.
///
/// # Safety
/// `fd` must be a valid writable FILE*. `buf` may be null.
#[export_name = "makemap"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_makemap(fd: *mut libc::FILE, buf: BufHandle) -> c_int {
    let mut did_cpo = false;

    // Loop twice: once for mappings (abbr=0), once for abbreviations (abbr=1).
    for abbr in 0..2i32 {
        for hash in 0..256i32 {
            let mp: MapblockHandle = if abbr != 0 {
                if hash > 0 {
                    break; // only one abbreviation list
                }
                if buf.is_null() {
                    nvim_get_first_abbr()
                } else {
                    nvim_buf_get_first_abbr(buf)
                }
            } else if buf.is_null() {
                nvim_get_maphash_entry(hash)
            } else {
                nvim_buf_get_maphash_entry(buf, hash)
            };

            let mut cur = mp;
            while !cur.is_null() {
                if makemap_should_skip(cur) != 0 {
                    cur = (*cur).m_next;
                    continue;
                }

                // Decompose mode into prefix characters.
                let mr = makemap_mode_chars((*cur).m_mode, abbr);
                if mr.error != 0 {
                    iemsg(c"E228: makemap: Illegal mode".as_ptr());
                    return FAIL;
                }
                let mut c1 = mr.c1;
                let mut c2 = mr.c2;
                let mut c3 = mr.c3;
                let cmd: *const c_char = if mr.use_bang != 0 {
                    c"map!".as_ptr()
                } else if abbr != 0 {
                    c"abbr".as_ptr()
                } else {
                    c"map".as_ptr()
                };

                // do this twice if c2 is set, 3 times with c3
                loop {
                    if !did_cpo && makemap_needs_cpo(cur) != 0 {
                        did_cpo = true;
                        if libc::fprintf(fd, c"let s:cpo_save=&cpo".as_ptr()) < 0
                            || put_eol(fd) < 0
                            || libc::fprintf(fd, c"set cpo&vim".as_ptr()) < 0
                            || put_eol(fd) < 0
                        {
                            return FAIL;
                        }
                    }
                    if c1 != 0 && libc::fputc(c_int::from(c1 as u8), fd) < 0 {
                        return FAIL;
                    }
                    if (*cur).m_noremap != REMAP_YES && libc::fputs(c"nore".as_ptr(), fd) < 0 {
                        return FAIL;
                    }
                    if libc::fputs(cmd, fd) < 0 {
                        return FAIL;
                    }
                    if !buf.is_null() && libc::fputs(c" <buffer>".as_ptr(), fd) < 0 {
                        return FAIL;
                    }
                    if (*cur).m_nowait != 0 && libc::fputs(c" <nowait>".as_ptr(), fd) < 0 {
                        return FAIL;
                    }
                    if (*cur).m_silent != 0 && libc::fputs(c" <silent>".as_ptr(), fd) < 0 {
                        return FAIL;
                    }
                    if (*cur).m_expr != 0 && libc::fputs(c" <expr>".as_ptr(), fd) < 0 {
                        return FAIL;
                    }

                    if libc::fputc(c_int::from(b' '), fd) < 0
                        || rs_put_escstr(fd, (*cur).m_keys, 0) == FAIL
                        || libc::fputc(c_int::from(b' '), fd) < 0
                        || rs_put_escstr(fd, (*cur).m_str, 1) == FAIL
                        || put_eol(fd) < 0
                    {
                        return FAIL;
                    }
                    c1 = c2;
                    c2 = c3;
                    c3 = 0;
                    if c1 == 0 {
                        break;
                    }
                }

                cur = (*cur).m_next;
            }
        }
    }

    if did_cpo
        && (libc::fprintf(fd, c"let &cpo=s:cpo_save".as_ptr()) < 0
            || put_eol(fd) < 0
            || libc::fprintf(fd, c"unlet s:cpo_save".as_ptr()) < 0
            || put_eol(fd) < 0)
    {
        return FAIL;
    }
    OK
}
