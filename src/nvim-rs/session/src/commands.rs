//! Session Commands FFI
//!
//! This module provides FFI helpers for session command handlers:
//! `:mksession`, `:mkview`, `:mkexrc`, `:mkvimrc`.
//! Phase 179 of Rust migration.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]

use std::ffi::{c_char, c_int};

use crate::SessionFlags;

// =============================================================================
// Command Type Constants
// =============================================================================

/// Command type for :mkexrc
pub const CMD_MKEXRC: c_int = 0;
/// Command type for :mkvimrc
pub const CMD_MKVIMRC: c_int = 1;
/// Command type for :mkview
pub const CMD_MKVIEW: c_int = 2;
/// Command type for :mksession
pub const CMD_MKSESSION: c_int = 3;

// =============================================================================
// Default File Names
// =============================================================================

/// Default file name for :mkexrc
static EXRC_FILE: &[u8] = b".exrc\0";
/// Default file name for :mkvimrc
static VIMRC_FILE: &[u8] = b".nvimrc\0";
/// Default file name for :mksession
static SESSION_FILE: &[u8] = b"Session.vim\0";

/// Get the default file name for a session command.
#[no_mangle]
pub extern "C" fn rs_session_default_filename(cmd_type: c_int) -> *const c_char {
    let name = match cmd_type {
        CMD_MKEXRC => EXRC_FILE,
        CMD_MKVIMRC => VIMRC_FILE,
        CMD_MKSESSION => SESSION_FILE,
        _ => b"\0",
    };
    name.as_ptr().cast::<c_char>()
}

// =============================================================================
// Command Type Queries
// =============================================================================

/// Check if the command is a view/session command (vs mkexrc/mkvimrc).
#[no_mangle]
pub extern "C" fn rs_session_is_view_session(cmd_type: c_int) -> c_int {
    c_int::from(cmd_type == CMD_MKVIEW || cmd_type == CMD_MKSESSION)
}

/// Check if the command is :mkview.
#[no_mangle]
pub extern "C" fn rs_session_is_mkview(cmd_type: c_int) -> c_int {
    c_int::from(cmd_type == CMD_MKVIEW)
}

/// Check if the command is :mksession.
#[no_mangle]
pub extern "C" fn rs_session_is_mksession(cmd_type: c_int) -> c_int {
    c_int::from(cmd_type == CMD_MKSESSION)
}

/// Check if the command is :mkvimrc.
#[no_mangle]
pub extern "C" fn rs_session_is_mkvimrc(cmd_type: c_int) -> c_int {
    c_int::from(cmd_type == CMD_MKVIMRC)
}

/// Check if the command is :mkexrc.
#[no_mangle]
pub extern "C" fn rs_session_is_mkexrc(cmd_type: c_int) -> c_int {
    c_int::from(cmd_type == CMD_MKEXRC)
}

// =============================================================================
// View File Name Generation
// =============================================================================

/// View file path separator replacement: normal path separator.
pub const VIEW_PATH_SEP_NORMAL: u8 = b'+';
/// View file path separator replacement: colon.
pub const VIEW_PATH_SEP_COLON: u8 = b'-';
/// View file path equals escape.
pub const VIEW_PATH_EQUALS: u8 = b'=';

/// Calculate the length needed for a view file name.
///
/// The view file name is the full path with:
/// - '=' -> "=="
/// - path separator -> "=+"
/// - ':' -> "=-"
///
/// Returns the additional length needed beyond the original string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_viewfile_extra_len(
    path: *const u8,
    len: c_int,
    is_path_sep: extern "C" fn(c: c_int) -> c_int,
) -> c_int {
    if path.is_null() || len <= 0 {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    let slice = std::slice::from_raw_parts(path, len as usize);
    let mut extra = 0;

    for &c in slice {
        if c == b'=' || is_path_sep(c_int::from(c)) != 0 {
            extra += 1; // Each of these becomes 2 chars
        }
    }

    extra
}

/// Encode a path for use in a view file name.
///
/// # Safety
/// - `src` must be valid for reading `src_len` bytes.
/// - `dst` must be valid for writing `src_len + extra_len` bytes, where
///   `extra_len` comes from `rs_session_viewfile_extra_len`.
///
/// Returns the number of bytes written to dst.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_session_encode_viewpath(
    src: *const u8,
    src_len: c_int,
    dst: *mut u8,
    is_path_sep: extern "C" fn(c: c_int) -> c_int,
) -> c_int {
    if src.is_null() || dst.is_null() || src_len <= 0 {
        return 0;
    }

    let src_slice = std::slice::from_raw_parts(src, src_len as usize);
    let mut written = 0;

    for &c in src_slice {
        if c == b'=' {
            *dst.add(written) = b'=';
            *dst.add(written + 1) = b'=';
            written += 2;
        } else if is_path_sep(c_int::from(c)) != 0 {
            *dst.add(written) = b'=';
            // Use '+' for normal separators, '-' for ':'
            *dst.add(written + 1) = if c == b':' {
                VIEW_PATH_SEP_COLON
            } else {
                VIEW_PATH_SEP_NORMAL
            };
            written += 2;
        } else {
            *dst.add(written) = c;
            written += 1;
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        written as c_int
    }
}

/// Get the view file suffix for a given view number.
///
/// Returns the character to append before ".vim" (e.g., '1'-'9' or NUL for default).
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub extern "C" fn rs_session_view_suffix(view_num: c_int) -> c_char {
    if (1..=9).contains(&view_num) {
        let c = b'0' + view_num as u8;
        c as c_char
    } else {
        0 // NUL for default view
    }
}

/// Check if an argument represents a view number (single digit 0-9).
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub extern "C" fn rs_session_is_view_number(arg_char: c_char, arg_next: c_char) -> c_int {
    let c = arg_char as u8;
    c_int::from(c.is_ascii_digit() && arg_next == 0)
}

// =============================================================================
// Session File Preamble/Epilogue
// =============================================================================

/// Get the version line for :mkvimrc.
#[no_mangle]
pub extern "C" fn rs_session_version_line() -> *const c_char {
    static VERSION: &[u8] = b"version 6.0\0";
    VERSION.as_ptr().cast::<c_char>()
}

/// Get the SessionLoad variable line.
#[no_mangle]
pub extern "C" fn rs_session_load_start() -> *const c_char {
    static LINE: &[u8] = b"let SessionLoad = 1\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the SessionLoad unlet line.
#[no_mangle]
pub extern "C" fn rs_session_load_end() -> *const c_char {
    static LINE: &[u8] = b"unlet SessionLoad\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the scrolloff save line.
#[no_mangle]
pub extern "C" fn rs_session_scrolloff_save() -> *const c_char {
    static LINE: &[u8] = b"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the scrolloff restore line.
#[no_mangle]
pub extern "C" fn rs_session_scrolloff_restore() -> *const c_char {
    static LINE: &[u8] = b"let &g:so = s:so_save | let &g:siso = s:siso_save\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the hlsearch set line.
#[no_mangle]
pub extern "C" fn rs_session_hlsearch_set() -> *const c_char {
    static LINE: &[u8] = b"set hlsearch\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the nohlsearch line.
#[no_mangle]
pub extern "C" fn rs_session_nohlsearch() -> *const c_char {
    static LINE: &[u8] = b"nohlsearch\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the SessionLoadPost autocmd line.
#[no_mangle]
pub extern "C" fn rs_session_loadpost_autocmd() -> *const c_char {
    static LINE: &[u8] = b"doautoall SessionLoadPost\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the modeline for session files.
#[no_mangle]
pub extern "C" fn rs_session_modeline() -> *const c_char {
    static LINE: &[u8] = b"\" vim: set ft=vim :\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the v:this_session variable set line.
#[no_mangle]
pub extern "C" fn rs_session_this_session_set() -> *const c_char {
    static LINE: &[u8] = b"let v:this_session=expand(\"<sfile>:p\")\0";
    LINE.as_ptr().cast::<c_char>()
}

// =============================================================================
// Session Content Commands
// =============================================================================

/// Get the "silent only" command.
#[no_mangle]
pub extern "C" fn rs_session_silent_only() -> *const c_char {
    static CMD: &[u8] = b"silent only\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the "silent tabonly" command.
#[no_mangle]
pub extern "C" fn rs_session_silent_tabonly() -> *const c_char {
    static CMD: &[u8] = b"silent tabonly\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the sesdir cd command.
#[no_mangle]
pub extern "C" fn rs_session_sesdir_cd() -> *const c_char {
    static CMD: &[u8] = b"exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the wipebuf detection code.
#[no_mangle]
pub extern "C" fn rs_session_wipebuf_detect() -> *const c_char {
    static CODE: &[u8] = b"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n  let s:wipebuf = bufnr('%')\nendif\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the wipebuf cleanup code.
#[no_mangle]
pub extern "C" fn rs_session_wipebuf_cleanup() -> *const c_char {
    static CODE: &[u8] = b"if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'\n  silent exe 'bwipe ' . s:wipebuf\nendif\nunlet! s:wipebuf\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the shortmess save line.
#[no_mangle]
pub extern "C" fn rs_session_shortmess_save() -> *const c_char {
    static LINE: &[u8] = b"let s:shortmess_save = &shortmess\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the shortmess set line.
#[no_mangle]
pub extern "C" fn rs_session_shortmess_set() -> *const c_char {
    static LINE: &[u8] = b"set shortmess+=aoO\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the shortmess restore line.
#[no_mangle]
pub extern "C" fn rs_session_shortmess_restore() -> *const c_char {
    static LINE: &[u8] = b"let &shortmess = s:shortmess_save\0";
    LINE.as_ptr().cast::<c_char>()
}

/// Get the x.vim source code.
#[no_mangle]
pub extern "C" fn rs_session_source_xvim() -> *const c_char {
    static CODE: &[u8] = b"let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"\nif filereadable(s:sx)\n  exe \"source \" . fnameescape(s:sx)\nendif\0";
    CODE.as_ptr().cast::<c_char>()
}

// =============================================================================
// Options Determination
// =============================================================================

/// Check if global options should be written.
#[no_mangle]
pub extern "C" fn rs_session_should_write_options(cmd_type: c_int, flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    let is_view_session = cmd_type == CMD_MKVIEW || cmd_type == CMD_MKSESSION;

    // Write options for:
    // - Non-view/session commands (mkexrc, mkvimrc)
    // - mksession with OPTIONS flag
    c_int::from(
        !is_view_session || (cmd_type == CMD_MKSESSION && f.contains(SessionFlags::OPTIONS)),
    )
}

/// Check if mappings should be written.
#[no_mangle]
pub extern "C" fn rs_session_should_write_mappings(cmd_type: c_int, flags: u32) -> c_int {
    // Same logic as options
    rs_session_should_write_options(cmd_type, flags)
}

/// Check if the skiprtp flag should be used for a given command.
#[no_mangle]
pub extern "C" fn rs_session_cmd_should_skip_rtp(cmd_type: c_int, flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(cmd_type == CMD_MKSESSION && f.contains(SessionFlags::SKIPRTP))
}

/// Check if using viewdir for the file name.
#[no_mangle]
pub extern "C" fn rs_session_uses_viewdir(
    cmd_type: c_int,
    arg_empty: c_int,
    arg_is_digit: c_int,
) -> c_int {
    // :mkview with no arg or single digit uses viewdir
    c_int::from(cmd_type == CMD_MKVIEW && (arg_empty != 0 || arg_is_digit != 0))
}

// =============================================================================
// Window State
// =============================================================================

/// Get the split save commands.
#[no_mangle]
pub extern "C" fn rs_session_split_save() -> *const c_char {
    static CODE: &[u8] = b"let s:save_splitbelow = &splitbelow\nlet s:save_splitright = &splitright\nset splitbelow splitright\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the split restore commands.
#[no_mangle]
pub extern "C" fn rs_session_split_restore() -> *const c_char {
    static CODE: &[u8] =
        b"let &splitbelow = s:save_splitbelow\nlet &splitright = s:save_splitright\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the winminheight save command.
#[no_mangle]
pub extern "C" fn rs_session_winmin_save() -> *const c_char {
    static CODE: &[u8] =
        b"let s:save_winminheight = &winminheight\nlet s:save_winminwidth = &winminwidth\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the winmin settings for session restoration.
#[no_mangle]
pub extern "C" fn rs_session_winmin_set() -> *const c_char {
    static CODE: &[u8] =
        b"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the winmin restore command.
#[no_mangle]
pub extern "C" fn rs_session_winmin_restore() -> *const c_char {
    static CODE: &[u8] =
        b"let &winminheight = s:save_winminheight\nlet &winminwidth = s:save_winminwidth\0";
    CODE.as_ptr().cast::<c_char>()
}

/// Get the maximize window command.
#[no_mangle]
pub extern "C" fn rs_session_maximize_win() -> *const c_char {
    static CMD: &[u8] = b"wincmd _ | wincmd |\0";
    CMD.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_command_type_queries() {
        assert_eq!(rs_session_is_view_session(CMD_MKVIEW), 1);
        assert_eq!(rs_session_is_view_session(CMD_MKSESSION), 1);
        assert_eq!(rs_session_is_view_session(CMD_MKEXRC), 0);
        assert_eq!(rs_session_is_view_session(CMD_MKVIMRC), 0);

        assert_eq!(rs_session_is_mkview(CMD_MKVIEW), 1);
        assert_eq!(rs_session_is_mkview(CMD_MKSESSION), 0);

        assert_eq!(rs_session_is_mksession(CMD_MKSESSION), 1);
        assert_eq!(rs_session_is_mksession(CMD_MKVIEW), 0);

        assert_eq!(rs_session_is_mkvimrc(CMD_MKVIMRC), 1);
        assert_eq!(rs_session_is_mkexrc(CMD_MKEXRC), 1);
    }

    #[test]
    fn test_default_filenames() {
        unsafe {
            let exrc = CStr::from_ptr(rs_session_default_filename(CMD_MKEXRC));
            assert_eq!(exrc.to_str().unwrap(), ".exrc");

            let vimrc = CStr::from_ptr(rs_session_default_filename(CMD_MKVIMRC));
            assert_eq!(vimrc.to_str().unwrap(), ".nvimrc");

            let session = CStr::from_ptr(rs_session_default_filename(CMD_MKSESSION));
            assert_eq!(session.to_str().unwrap(), "Session.vim");

            let view = CStr::from_ptr(rs_session_default_filename(CMD_MKVIEW));
            assert_eq!(view.to_str().unwrap(), ""); // No default for mkview
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_view_suffix() {
        assert_eq!(rs_session_view_suffix(1), b'1' as c_char);
        assert_eq!(rs_session_view_suffix(9), b'9' as c_char);
        assert_eq!(rs_session_view_suffix(0), 0);
        assert_eq!(rs_session_view_suffix(10), 0);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_is_view_number() {
        assert_eq!(rs_session_is_view_number(b'0' as c_char, 0), 1);
        assert_eq!(rs_session_is_view_number(b'9' as c_char, 0), 1);
        assert_eq!(rs_session_is_view_number(b'1' as c_char, b'2' as c_char), 0); // "12" - not single
        assert_eq!(rs_session_is_view_number(b'a' as c_char, 0), 0);
    }

    #[allow(clippy::cast_lossless)]
    extern "C" fn test_is_path_sep(c: c_int) -> c_int {
        c_int::from(c == b'/' as c_int || c == b'\\' as c_int || c == b':' as c_int)
    }

    #[test]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    fn test_viewfile_extra_len() {
        unsafe {
            // /home/user/file.txt has 3 slashes
            let path = b"/home/user/file.txt";
            let extra =
                rs_session_viewfile_extra_len(path.as_ptr(), path.len() as c_int, test_is_path_sep);
            assert_eq!(extra, 3);

            // /home=test has 1 separator + 1 equals
            let path_eq = b"/home=test";
            let extra_eq = rs_session_viewfile_extra_len(
                path_eq.as_ptr(),
                path_eq.len() as c_int,
                test_is_path_sep,
            );
            assert_eq!(extra_eq, 2);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    fn test_encode_viewpath() {
        unsafe {
            let src = b"/home/test";
            let mut dst = [0u8; 20];
            let len = rs_session_encode_viewpath(
                src.as_ptr(),
                src.len() as c_int,
                dst.as_mut_ptr(),
                test_is_path_sep,
            );
            // /home/test -> =+home=+test (12 chars)
            assert_eq!(len, 12);
            assert_eq!(&dst[..12], b"=+home=+test");
        }
    }

    #[test]
    fn test_should_write_options() {
        // mkvimrc always writes options
        assert_eq!(rs_session_should_write_options(CMD_MKVIMRC, 0), 1);
        assert_eq!(rs_session_should_write_options(CMD_MKEXRC, 0), 1);

        // mksession only with OPTIONS flag
        assert_eq!(
            rs_session_should_write_options(CMD_MKSESSION, SessionFlags::OPTIONS.bits()),
            1
        );
        assert_eq!(rs_session_should_write_options(CMD_MKSESSION, 0), 0);

        // mkview never writes global options
        assert_eq!(
            rs_session_should_write_options(CMD_MKVIEW, SessionFlags::OPTIONS.bits()),
            0
        );
    }

    #[test]
    fn test_uses_viewdir() {
        // mkview with empty arg
        assert_eq!(rs_session_uses_viewdir(CMD_MKVIEW, 1, 0), 1);
        // mkview with digit arg
        assert_eq!(rs_session_uses_viewdir(CMD_MKVIEW, 0, 1), 1);
        // mkview with other arg
        assert_eq!(rs_session_uses_viewdir(CMD_MKVIEW, 0, 0), 0);
        // mksession never uses viewdir
        assert_eq!(rs_session_uses_viewdir(CMD_MKSESSION, 1, 0), 0);
    }

    #[test]
    fn test_static_strings() {
        unsafe {
            let version = CStr::from_ptr(rs_session_version_line());
            assert_eq!(version.to_str().unwrap(), "version 6.0");

            let modeline = CStr::from_ptr(rs_session_modeline());
            assert!(modeline.to_str().unwrap().contains("vim:"));

            let this_session = CStr::from_ptr(rs_session_this_session_set());
            assert!(this_session.to_str().unwrap().contains("v:this_session"));
        }
    }
}
