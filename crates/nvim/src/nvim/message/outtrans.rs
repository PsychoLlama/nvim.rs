//! Turning bytes into something displayable.
//!
//! The `msg_outtrans*` half renders unprintable bytes as `<xx>` and multibyte
//! sequences as themselves; the `str2special*` half renders key codes as
//! `<C-X>` notation, which is what mapping listings and `keytrans()` show.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::{termcap_key, termcap_name};
use crate::src::nvim::types::MB_MAXCHAR;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The `<xx>` form of an unprintable byte gets its own highlight so it can be
/// told apart from the same characters typed literally.
const SPECIAL_HL: c_int = HLF_8;

/// Show one character.
pub unsafe fn msg_putchar(c: c_int) {
    unsafe { msg_putchar_hl(c, 0) }
}

/// Show one character with a highlight id.
///
/// A special key is put back into the three-byte `K_SPECIAL` form it arrived
/// as, because that is what [`msg_outtrans_len`] and `str2special` downstream
/// know how to read.
pub unsafe fn msg_putchar_hl(c: c_int, hl_id: c_int) {
    unsafe {
        let mut buf = [0 as c_char; MB_MAXCHAR + 1];
        if c < 0 {
            // `K_SECOND`/`K_THIRD`, less their `c == K_SPECIAL`/`c == NUL`
            // arms: both of those codes are positive, so neither is reachable
            // here.
            let name = termcap_name(c);
            buf[0] = K_SPECIAL as c_char;
            buf[1] = name[0] as c_char;
            buf[2] = name[1] as c_char;
        } else {
            let len = utf_char2bytes(c, buf.as_mut_ptr());
            buf[len as usize] = 0;
        }
        msg_puts_hl(buf.as_ptr(), hl_id, false)
    }
}

/// Show a number in decimal.
pub unsafe fn msg_outnum(n: c_int) {
    // Filled from the right so the digits come out in order; the last byte
    // stays zero and terminates it.
    let mut buf = [0u8; 16];
    let mut at = buf.len() - 1;
    let mut rest = n.unsigned_abs();
    loop {
        at -= 1;
        buf[at] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    if n < 0 {
        at -= 1;
        buf[at] = b'-';
    }
    unsafe { msg_puts(buf[at..].as_ptr().cast()) }
}

/// Show a file name with `$HOME` folded back to `~`.
pub unsafe fn msg_home_replace(fname: *const c_char) {
    unsafe { msg_home_replace_hl(fname, 0) }
}

pub(crate) unsafe fn msg_home_replace_hl(fname: *const c_char, hl_id: c_int) {
    unsafe {
        let name = home_replace_save(ptr::null_mut(), fname);
        msg_outtrans(name, hl_id, false);
        xfree(name.cast());
    }
}

/// Show a NUL-terminated string, translating what cannot be displayed.
///
/// Answers how many screen cells it took.
pub unsafe fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int {
    unsafe { msg_outtrans_len(str, strlen(str) as c_int, hl_id, hist) }
}

/// Show the one character at `p`, answering a pointer to the next one.
pub unsafe fn msg_outtrans_one(p: *const c_char, hl_id: c_int, hist: bool) -> *const c_char {
    unsafe {
        let len = utfc_ptr2len(p);
        if len > 1 {
            msg_outtrans_len(p, len, hl_id, hist);
            return p.add(len as usize);
        }
        msg_puts_hl(
            transchar_byte_buf(ptr::null(), *p as u8 as c_int),
            hl_id,
            hist,
        );
        p.add(1)
    }
}

/// Show `len` bytes of `msgstr`, NULs included, translating what cannot be
/// displayed.
///
/// Printable runs are handed to [`msg_puts_len`] whole; only the characters
/// that need a `<xx>` or `<C-X>` rendering are emitted one at a time, in the
/// `SPECIAL_HL` highlight.
///
/// Answers how many screen cells it took.
///
/// # Safety
/// `msgstr` must point at `len` readable bytes.
pub unsafe fn msg_outtrans_len(
    msgstr: *const c_char,
    len: c_int,
    hl_id: c_int,
    hist: bool,
) -> c_int {
    unsafe {
        let mut cells = 0;
        let mut str = msgstr;
        // Start of the run of printable bytes not yet emitted.
        let mut plain_start = msgstr;
        // Only quit when got_int was set in here.
        let save_got_int = got_int.get();
        got_int.set(false);

        if hist {
            msg_hist_add(str, len, hl_id);
        }

        // When drawing over the command line there is no need to clear it
        // later or to remove the mode message.
        if msg_silent.get() == 0
            && len > 0
            && msg_row.get() >= cmdline_row.get()
            && msg_col.get() == 0
        {
            clear_cmdline.set(false);
            mode_displayed.set(false);
        }

        // `left` is how many bytes follow the one being looked at, which is
        // what utfc_ptr2len_len needs as its bound.
        let mut left = len;
        loop {
            left -= 1;
            if left < 0 || got_int.get() {
                break;
            }
            let flush_plain = |upto: *const c_char, plain_start: *const c_char| {
                if upto > plain_start {
                    msg_puts_len(plain_start, upto.offset_from(plain_start), hl_id, hist);
                }
            };
            // Don't include composing chars after the end.
            let mb_len = utfc_ptr2len_len(str, left + 1);
            if mb_len > 1 {
                let c = utf_ptr2char(str);
                if vim_isprintc(c) {
                    cells += utf_ptr2cells(str);
                } else {
                    flush_plain(str, plain_start);
                    plain_start = str.add(mb_len as usize);
                    msg_puts_hl(transchar_buf(ptr::null(), c), special_hl(hl_id), false);
                    cells += char2cells(c);
                }
                left -= mb_len - 1;
                str = str.add(mb_len as usize);
            } else {
                let rendered = transchar_byte_buf(ptr::null(), *str as u8 as c_int);
                if *rendered.add(1) != 0 {
                    // Unprintable: emit the printable run so far, then it.
                    flush_plain(str, plain_start);
                    plain_start = str.add(1);
                    msg_puts_hl(rendered, special_hl(hl_id), false);
                    cells += strlen(rendered) as c_int;
                } else {
                    cells += 1;
                }
                str = str.add(1);
            }
        }

        // The printable characters at the end -- or, for an empty string, the
        // empty message the callers rely on being emitted.
        if (str > plain_start || plain_start == msgstr) && !got_int.get() {
            msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
        }

        got_int.set(got_int.get() | save_got_int);
        cells
    }
}

/// Unprintable characters take `SPECIAL_HL` unless the caller asked for a
/// highlight of its own.
fn special_hl(hl_id: c_int) -> c_int {
    if hl_id == 0 { SPECIAL_HL } else { hl_id }
}

/// `:smile`.
pub unsafe fn msg_make(arg: *const c_char) {
    // The command name backwards, and the answer with every byte shifted up
    // by three -- both so that neither reads as itself in the binary.
    const REVERSED: &[u8] = b"eeffoc";
    const SHIFTED: &[u8] = b"Plon#dqg#vxjduB";

    unsafe {
        let mut arg = skipwhite(arg);
        let mut at = REVERSED.len() as isize - 1;
        while *arg != 0 && at >= 0 {
            let byte = *arg as u8;
            arg = arg.add(1);
            if byte != REVERSED[at as usize] {
                break;
            }
            at -= 1;
        }
        if at < 0 {
            msg_putchar(NL);
            for &byte in SHIFTED {
                msg_putchar((byte - 3) as c_int);
            }
        }
    }
}

/// Show a string with key codes rendered as `<C-X>` notation, the way a
/// mapping is listed.
///
/// A leading or trailing space is shown as `<Space>` so it cannot be missed.
/// Stops before exceeding `maxlen` screen columns; 0 means unlimited.
///
/// @param from  true for the left-hand side of a mapping
pub unsafe fn msg_outtrans_special(strstart: *const c_char, from: bool, maxlen: c_int) -> c_int {
    if strstart.is_null() {
        return 0;
    }
    unsafe {
        let mut str = strstart;
        let mut cells = 0;
        while *str != 0 {
            let mut text = if *str == b' ' as c_char && (str == strstart || *str.add(1) == 0) {
                str = str.add(1);
                c"<Space>".as_ptr()
            } else {
                str2special(&raw mut str, from, false)
            };
            if *text != 0 && *text.add(1) == 0 {
                // Single-byte character, or an illegal byte.
                text = transchar_byte_buf(ptr::null(), *text as u8 as c_int);
            }
            let len = vim_strsize(text);
            if maxlen > 0 && cells + len >= maxlen {
                break;
            }
            // Highlight the ones that came out as a `<>` name.
            let hl_id = if len > 1 && utfc_ptr2len(text) <= 1 {
                SPECIAL_HL
            } else {
                0
            };
            msg_puts_hl(text, hl_id, false);
            cells += len;
        }
        cells
    }
}

/// [`str2special`] over a whole string, into a freshly allocated one.
///
/// The caller owns the result and frees it with `xfree`.
pub unsafe fn str2special_save(
    str: *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *mut c_char {
    unsafe {
        let mut ga = garray_T::default();
        ga_init(&raw mut ga, 1, 40);
        let mut p = str;
        while *p != 0 {
            ga_concat(
                &raw mut ga,
                str2special(&raw mut p, replace_spaces, replace_lt),
            );
        }
        ga_append(&raw mut ga, 0);
        ga.ga_data.cast()
    }
}

/// [`str2special`] over a whole string, into `arena`.
///
/// Measures first and copies second, because the conversion buffer is one
/// shared static and cannot hold two answers at once.
pub unsafe fn str2special_arena(
    str: *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
    arena: *mut Arena,
) -> *mut c_char {
    unsafe {
        let mut len: size_t = 0;
        let mut p = str;
        while *p != 0 {
            len += strlen(str2special(&raw mut p, replace_spaces, replace_lt));
        }

        let buf: *mut c_char = arena_alloc(arena, len + 1, false).cast();
        let mut at: size_t = 0;
        p = str;
        while *p != 0 {
            let piece = str2special(&raw mut p, replace_spaces, replace_lt);
            let piece_len = strlen(piece);
            ptr::copy_nonoverlapping(piece, buf.add(at), piece_len);
            at += piece_len;
        }
        *buf.add(at) = 0;
        buf
    }
}

/// Render one key code as printable text, advancing `*sp` past it.
///
/// Special keys and C0 control characters come out in `<>` form;
/// `replace_spaces` and `replace_lt` extend that to `<Space>` and `<lt>`,
/// which is what a mapping's left-hand side and `keytrans()` want and its
/// right-hand side does not.
///
/// The answer lives in **one shared static buffer**, so it has to be copied
/// somewhere before the next call. An illegal byte comes back as itself.
///
/// # Safety
/// `sp` must point at a readable pointer into a NUL-terminated string.
pub unsafe fn str2special(
    sp: *mut *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *const c_char {
    static BUF: GlobalCell<[c_char; 7]> = GlobalCell::new([0; 7]);

    unsafe {
        // A multi-byte character escaped into the stream comes back whole.
        let unescaped = mb_unescape(sp);
        if !unescaped.is_null() {
            return unescaped;
        }

        let mut str = *sp;
        let mut c = *str as u8 as c_int;
        let mut modifiers = 0;
        let mut special = false;
        if c == K_SPECIAL && *str.add(1) != 0 && *str.add(2) != 0 {
            if *str.add(1) as u8 as c_int == KS_MODIFIER {
                modifiers = *str.add(2) as u8 as c_int;
                str = str.add(3);
                c = *str as u8 as c_int;
            }
            if c == K_SPECIAL && *str.add(1) != 0 && *str.add(2) != 0 {
                c = to_special(*str.add(1) as u8, *str.add(2) as u8);
                str = str.add(2);
            }
            if c < 0 || modifiers != 0 {
                special = true;
            }
        }

        if c >= 0 && utf8len_tab[c as usize] > 1 {
            *sp = str;
            // Try to un-escape a multi-byte character after the modifiers.
            let unescaped = mb_unescape(sp);
            if unescaped.is_null() {
                // Illegal byte.
                *sp = str.add(1);
            } else {
                // `special` is set, so get_special_key_name() renders it.
                c = utf_ptr2char(unescaped);
            }
        } else {
            // Single-byte character, NUL or illegal byte.
            *sp = str.add(usize::from(*str != 0));
        }

        if special
            || c < b' ' as c_int
            || (replace_spaces && c == b' ' as c_int)
            || (replace_lt && c == b'<' as c_int)
        {
            return get_special_key_name(c, modifiers);
        }
        BUF.with_mut(|buf| {
            buf[0] = c as c_char;
            buf[1] = 0;
        });
        BUF.ptr().cast()
    }
}

/// The key code a two-byte termcap name stands for (C's `TO_SPECIAL`).
fn to_special(second: u8, third: u8) -> c_int {
    match second as c_int {
        KS_SPECIAL => K_SPECIAL,
        KS_ZERO => K_ZERO,
        _ => termcap_key([second, third]),
    }
}

/// Show a string, cutting the middle out with `...` if it would not fit on the
/// rest of the line.
///
/// Does not handle multi-byte characters.
pub unsafe fn msg_outtrans_long(longstr: *const c_char, hl_id: c_int) {
    unsafe {
        let len = strlen(longstr) as c_int;
        let mut tail = len;
        let room = Columns.get() - msg_col.get();
        if !ui_has(kUIMessages) && len > room && room >= 20 {
            tail = (room - 3) / 2;
            msg_outtrans_len(longstr, tail, hl_id, false);
            msg_puts_hl(c"...".as_ptr(), SPECIAL_HL, false);
        }
        msg_outtrans_len(longstr.offset((len - tail) as isize), tail, hl_id, false);
    }
}
