//! The `~` shorthand going back out: a path with the home directory
//! written as `~`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::fs::modify_fname;
use crate::memory::XString;
use crate::os::env::{getenv_owned, with_home_dir};
use crate::path::{path_fnamencmp, path_tail, vim_ispathsep};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

/// `$HOME` as a directory name: its value, or -- when that is itself
/// written with a `~` -- the full path it stands for, without the trailing
/// separator a directory gets.
fn home_env() -> Option<Vec<u8>> {
    let home = getenv_owned(c"HOME")?;
    if home.first() != Some(&b'~') {
        return Some(home.to_vec());
    }
    let home = home.as_cstr();
    let mut full = modify_fname(c":p", &mut 0, false, home, home.count_bytes()).1;
    // The name up to its first NUL, as the C measured it.
    full.truncate(full.iter().position(|&b| b == 0).unwrap_or(full.len()));
    if full.last().is_some_and(|&b| vim_ispathsep(c_int::from(b))) {
        // Drop the '/' that gets added to a directory.
        full.pop();
    }
    Some(full)
}

/// `src` with the home directory written as `~`, cut where upstream's
/// `room`-byte buffer (NUL included) would cut it.
///
/// `one` treats `src` as a single file name rather than a space/comma
/// separated list. A help buffer's name is shortened to its tail alone.
/// Both the resolved home directory and the *value* of `$HOME` count, but
/// only up to a path separator: with a home of "/home/piet",
/// "/home/pieter/bla" must not come back as "~er/bla".
pub(crate) fn home_to_tilde(buffer: Option<Buf>, src: &CStr, room: usize, one: bool) -> XString {
    if room == 0 {
        return XString::new();
    }
    if buffer.is_some_and(|b| b.b_help) {
        let tail = path_tail(src).to_bytes();
        return XString::from_bytes(&tail[..tail.len().min(room - 1)]);
    }
    let home_env = home_env();
    with_home_dir(|homedir| tilde_into(src.to_bytes(), [homedir, home_env.as_deref()], room, one))
}

/// [`home_to_tilde`]'s copy loop, against the two spellings of the home
/// directory. `room` counts down with every byte written, and running out
/// truncates.
fn tilde_into(src: &[u8], homes: [Option<&[u8]>; 2], room: usize, one: bool) -> XString {
    let at = |i: usize| src.get(i).copied().unwrap_or(0);
    let is_separator = |b: u8| b == b',' || b == b' ';
    let mut out = XString::with_capacity(src.len().min(room));
    let mut room = room;
    let mut i = if one {
        0
    } else {
        src.iter()
            .position(|&b| b != b' ' && b != b'\t')
            .unwrap_or(src.len())
    };
    while i < src.len() && room > 0 {
        // At the start of a file name: either home directory may match.
        for home in homes.into_iter().flatten() {
            let len = home.len();
            let next = at(i + len);
            if len != 0
                && path_fnamencmp(&src[i..], home, len) == 0
                && (vim_ispathsep(c_int::from(next)) || (!one && is_separator(next)) || next == 0)
            {
                i += len;
                room -= 1;
                if room > 0 {
                    out.push_byte(b'~');
                }
                // No separator goes out: the caller wants the directory name
                // without one.
                break;
            }
        }
        if room == 0 {
            break;
        }
        // Copy the name, up to the separator unless `one`.
        while i < src.len() && (one || !is_separator(src[i])) {
            room -= 1;
            if room == 0 {
                break;
            }
            out.push_byte(src[i]);
            i += 1;
        }
        if room == 0 {
            break;
        }
        // And the separator itself.
        while i < src.len() && is_separator(src[i]) {
            room -= 1;
            if room == 0 {
                break;
            }
            out.push_byte(src[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tilde(src: &str, home: &str, room: usize, one: bool) -> String {
        let out = tilde_into(src.as_bytes(), [Some(home.as_bytes()), None], room, one);
        String::from_utf8(out.to_vec()).unwrap()
    }

    #[test]
    fn the_home_directory_becomes_a_tilde_only_up_to_a_separator() {
        assert_eq!(tilde("/home/piet/bla", "/home/piet", 100, true), "~/bla");
        assert_eq!(
            tilde("/home/pieter/bla", "/home/piet", 100, true),
            "/home/pieter/bla"
        );
        assert_eq!(tilde("/home/piet", "/home/piet", 100, true), "~");
        assert_eq!(
            tilde("/home/piet/a,/home/piet/b", "/home/piet", 100, false),
            "~/a,~/b"
        );
    }

    #[test]
    fn running_out_of_room_truncates() {
        assert_eq!(tilde("/home/piet/bla", "/home/piet", 4, true), "~/b");
        assert_eq!(tilde("abcdef", "/home/piet", 3, true), "ab");
    }
}
