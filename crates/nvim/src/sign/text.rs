//! A sign's `text=`: parsing it into cells, and rendering it back.
//!
//! A sign is drawn in exactly [`SIGN_WIDTH`] cells, each holding one
//! `schar_T`. [`init_sign_text`] turns a `text=` value into those cells and
//! [`describe_sign_text`] turns them back into bytes for `:sign list`,
//! `sign_getdefined()` and `'statuscolumn'`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

/// Room for [`describe_sign_text`]'s answer: SIGN_WIDTH cells of up to
/// `MAX_SCHAR_SIZE` bytes each, the last of which carries the NUL
/// `schar_get` writes.
pub(crate) const SIGN_TEXT_BUF: usize = SIGN_WIDTH as usize * MAX_SCHAR_SIZE as usize;

/// Renders `sign_text` back into `buf` and answers how many bytes it wrote.
/// No extra `+ 1` is needed on [`SIGN_TEXT_BUF`]: a cell that renders empty
/// stops the walk, and `schar_get` has already written its NUL.
///
/// # Safety
/// `buf` must have room for [`SIGN_TEXT_BUF`] bytes and `sign_text` for
/// `SIGN_WIDTH` cells.
pub(crate) unsafe fn describe_sign_text(buf: *mut c_char, sign_text: *mut schar_T) -> size_t {
    // SAFETY: `sign_text` holds SIGN_WIDTH cells, per the caller.
    let cells = unsafe { slice::from_raw_parts(sign_text, SIGN_WIDTH as usize) };
    let mut at = 0;
    for &cell in cells {
        // SAFETY: `buf` has room for SIGN_TEXT_BUF bytes, and `at` is how
        // many of them the cells so far have used.
        let len = unsafe {
            schar_get(buf.add(at), cell);
            strlen(buf.add(at))
        };
        if len == 0 {
            break;
        }
        at += len;
    }
    at
}

/// Removes one level of backslash escaping from `text`, in place, and
/// answers the length of what is left.
///
/// `text` is the string's bytes **plus its NUL**, which is moved along with
/// the rest so the result is still NUL-terminated; the answer does not count
/// it. A backslash is dropped and whatever follows it kept, so `text=\ x`
/// can carry a space and `text=\\` a backslash. The **last** byte is never
/// examined, so a value ending in a lone backslash keeps it -- upstream's
/// loop stops one short, and `:sign define x text=a\` shows `a\`.
fn unescape(text: &mut [u8]) -> usize {
    // The NUL's index is also the current length.
    let mut end = text.len() - 1;
    let mut at = 0;
    while at + 1 < end {
        if text[at] == b'\\' {
            text.copy_within(at + 1..=end, at);
            end -= 1;
        }
        at += 1;
    }
    end
}

/// Parses a sign's `text=` into `sign_text`; `FAIL` when it does not fit.
///
/// `from_define` distinguishes the `:sign define` / `sign_define()` caller,
/// which unescapes backslashes (see [`unescape`]) and diagnoses a bad value,
/// from `nvim_buf_set_extmark`, which does neither. The unescaping happens
/// **in place**, in the caller's own buffer.
///
/// A one-cell text is padded to two with a space, and a two-cell character
/// blanks the second cell so the drawing code knows not to emit it.
///
/// # Safety
/// `text` must be a writable NUL-terminated string and `sign_text` must have
/// room for `SIGN_WIDTH` cells.
pub(crate) unsafe fn init_sign_text(
    text: *mut c_char,
    sign_text: *mut schar_T,
    from_define: bool,
) -> c_int {
    // SAFETY: the caller's text, NUL-terminated and writable.
    let len = unsafe { strlen(text) };
    let len = if from_define {
        // SAFETY: as above; the shift copies the NUL along with the rest, so
        // the string stays NUL-terminated at its new length.
        unsafe { unescape(slice::from_raw_parts_mut(text.cast::<u8>(), len + 1)) }
    } else {
        len
    };
    // SAFETY: as above.
    let endp = unsafe { text.add(len) };
    // SAFETY: the caller's cell array, `SIGN_WIDTH` wide.
    let out = unsafe { slice::from_raw_parts_mut(sign_text, SIGN_WIDTH as usize) };

    // Count display cells, stopping at the first unprintable character.
    let width_limit = usize::try_from(SIGN_WIDTH).expect("a sign is a positive number of cells");
    let mut cells = 0usize;
    let mut s = text;
    while s < endp {
        // SAFETY: `s` points into the text, which is NUL-terminated.
        let (sc, c, width, step) = unsafe {
            let mut c: c_int = 0;
            let sc = utfc_ptr2schar(s, &raw mut c);
            let width = usize::try_from(utf_ptr2cells(s)).expect("a cell width is not negative");
            let step = usize::try_from(utfc_ptr2len(s)).expect("a character is at least one byte");
            (sc, c, width, step)
        };
        // `sign_text` holds SIGN_WIDTH cells but this walk runs to the end
        // of `text` and only tests the width afterwards, so upstream
        // (v0.12.4) overruns the array for anything wider: on the heap via
        // `:sign define x text=xxx`, on the STACK via
        // nvim_buf_set_extmark{sign_text=..}. Dropping the out-of-range
        // stores is unobservable — every path that gets here with
        // `cells >= SIGN_WIDTH` goes on to fail and discard the array.
        if cells < width_limit {
            out[cells] = sc;
        }
        // SAFETY: `c` is the codepoint just decoded; the printability test
        // reaches the editor's character tables.
        if !unsafe { vim_isprintc(c) } {
            break;
        }
        if width == 2 && cells + 1 < width_limit {
            out[cells + 1] = 0;
        }
        cells += width;
        // SAFETY: `step` is the length of the character at `s`, which is
        // inside the text.
        s = unsafe { s.add(step) };
    }

    // Must be empty, one cell or two; `s != endp` means the walk stopped on
    // an unprintable character.
    if s != endp || cells > width_limit {
        if from_define {
            // SAFETY: the caller's text, and a format the message takes.
            unsafe { semsg_c!(gettext(c"E239: Invalid sign text: %s".as_ptr()), text) };
        }
        return FAIL;
    }

    if cells < 1 {
        out[0] = 0;
    } else if cells == 1 {
        out[1] = schar_T::from(b' ');
    }
    OK
}

#[cfg(test)]
mod tests {
    use super::unescape;

    /// `unescape` on a scratch buffer, the way `init_sign_text` calls it:
    /// the string's bytes plus the NUL.
    fn unescaped(text: &str) -> String {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        let len = unescape(&mut bytes);
        assert_eq!(0, bytes[len], "the NUL moved with the rest");
        String::from_utf8(bytes[..len].to_vec()).expect("still UTF-8")
    }

    #[test]
    fn a_backslash_escapes_the_byte_after_it() {
        assert_eq!(" x", unescaped("\\ x"));
        assert_eq!("a\\b", unescaped("a\\\\b"));
    }

    #[test]
    fn text_with_no_backslash_is_left_alone() {
        assert_eq!("ab", unescaped("ab"));
        assert_eq!("", unescaped(""));
    }

    /// The loop stops one byte short of the end, so a value ending in a lone
    /// backslash keeps it. Upstream's behaviour, pinned here because it is
    /// the one thing about this loop that looks like a bug and is not ours.
    #[test]
    fn a_trailing_backslash_survives() {
        assert_eq!("a\\", unescaped("a\\"));
        assert_eq!("\\", unescaped("\\"));
    }

    #[test]
    fn every_backslash_in_a_run_is_taken_in_turn() {
        assert_eq!("\\\\", unescaped("\\\\\\\\"));
        assert_eq!("ab", unescaped("\\a\\b"));
    }
}
