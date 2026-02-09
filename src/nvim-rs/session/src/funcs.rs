//! Migrated C function implementations for ex_session.c
//!
//! Each `rs_*` function is the Rust implementation of a corresponding C function.
//! The C function body is replaced with a thin wrapper that calls the `rs_*` version.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_cast_constness)]

use std::ffi::{c_char, c_int, c_uint, CStr};
use std::fmt::Write as _;

use crate::ffi;

/// OK return value (matches C OK = 1)
pub const OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0)
pub const FAIL: c_int = 0;

/// Session option flag constants (verified via _Static_assert in C)
pub const K_OPT_SSOP_FLAG_BLANK: c_uint = 0x80;
pub const K_OPT_SSOP_FLAG_HELP: c_uint = 0x40;
pub const K_OPT_SSOP_FLAG_TERMINAL: c_uint = 0x10000;
pub const K_OPT_SSOP_FLAG_CURDIR: c_uint = 0x1000;
pub const K_OPT_SSOP_FLAG_SESDIR: c_uint = 0x800;
pub const K_OPT_SSOP_FLAG_WINSIZE: c_uint = 0x08;

/// Frame layout constants (verified via _Static_assert in C)
pub const FR_LEAF: c_int = 0;
pub const FR_ROW: c_int = 1;
pub const FR_COL: c_int = 2;

/// MAXCOL - maximal column number (verified via _Static_assert in C)
pub const MAXCOL: c_int = 0x7FFF_FFFF;

/// MAXPATHL - maximum path length (verified via _Static_assert in C)
pub const MAXPATHL: usize = 4096;

// =============================================================================
// Phase 2: Window/Frame Predicates
// =============================================================================

/// Check if window "wp" should be stored in the session.
///
/// Returns non-zero (truthy) if the window should be saved.
/// Matches the logic of `ses_do_win()` in ex_session.c.
///
/// # Safety
/// `wp` must be a valid win_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_do_win(wp: ffi::WinPtr) -> c_int {
    // Skip floating windows to avoid issues when restoring the Session. #18432
    if ffi::nvim_ses_win_get_floating(wp) {
        return 0; // false
    }

    let buf = ffi::nvim_ses_win_get_buffer(wp);
    let fname = ffi::nvim_ses_buf_get_fname(buf);
    let ssop_flags = ffi::nvim_ses_get_ssop_flags();

    if fname.is_null() || (!ffi::nvim_ses_buf_is_terminal(buf) && ffi::nvim_ses_bt_nofilename(buf))
    {
        // When 'buftype' is "nofile" can't restore the window contents.
        return (ssop_flags & K_OPT_SSOP_FLAG_BLANK) as c_int;
    }
    if ffi::nvim_ses_bt_help(buf) {
        return (ssop_flags & K_OPT_SSOP_FLAG_HELP) as c_int;
    }
    if ffi::nvim_ses_bt_terminal(buf) {
        return (ssop_flags & K_OPT_SSOP_FLAG_TERMINAL) as c_int;
    }
    1 // true
}

/// Check if frame "fr" has a window somewhere that we want to save in the Session.
///
/// # Safety
/// `fr` must be a valid frame_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_do_frame(fr: ffi::FramePtr) -> bool {
    let layout = ffi::nvim_ses_frame_get_layout(fr);
    if layout == FR_LEAF {
        let win = ffi::nvim_ses_frame_get_win(fr);
        return rs_ses_do_win(win) != 0;
    }
    // Iterate children (FOR_ALL_FRAMES equivalent)
    let mut frc = ffi::nvim_ses_frame_get_child(fr);
    while !frc.is_null() {
        if rs_ses_do_frame(frc) {
            return true;
        }
        frc = ffi::nvim_ses_frame_get_next(frc);
    }
    false
}

/// Skip frames that don't contain windows we want to save in the Session.
///
/// Returns NULL when there are none.
///
/// # Safety
/// `fr` must be a valid frame_T pointer or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_skipframe(fr: ffi::FramePtr) -> ffi::FramePtr {
    // FOR_ALL_FRAMES(frc, fr) { if (ses_do_frame(frc)) break; }
    let mut frc = fr;
    while !frc.is_null() {
        if rs_ses_do_frame(frc) {
            break;
        }
        frc = ffi::nvim_ses_frame_get_next(frc);
    }
    frc
}

// =============================================================================
// Phase 3: Filename Helpers
// =============================================================================

/// Get the buffer name for `buf`.
///
/// Uses the short file name when the current directory is known at session
/// source time (session flags with curdir/sesdir, no acd, no lcd).
/// Otherwise uses the full file name.
///
/// # Safety
/// `buf` must be a valid buf_T pointer. `flagp` must be a valid pointer to unsigned.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_get_fname(buf: ffi::BufPtr, flagp: *const c_uint) -> *const c_char {
    let sfname = ffi::nvim_ses_buf_get_sfname(buf);
    let ssop_ptr = ffi::nvim_ses_get_ssop_flags_ptr();

    // Use the short file name if:
    // - Buffer has b_sfname
    // - flagp == &ssop_flags (session mode, not view)
    // - (ssop_flags & (CURDIR | SESDIR)) is set
    // - !p_acd
    // - !did_lcd
    if !sfname.is_null()
        && flagp == ssop_ptr
        && (ffi::nvim_ses_get_ssop_flags() & (K_OPT_SSOP_FLAG_CURDIR | K_OPT_SSOP_FLAG_SESDIR)) != 0
        && ffi::nvim_ses_get_p_acd() == 0
        && ffi::nvim_ses_get_did_lcd() == 0
    {
        return sfname;
    }
    ffi::nvim_ses_buf_get_ffname(buf)
}

/// Escape a filename for session writing.
///
/// Replaces backslashes with forward slashes (always kOptSsopFlagSlash)
/// and escapes special characters.
///
/// Returns an allocated string (caller must free with `nvim_ses_xfree`).
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_escape_fname(
    name: *mut c_char,
    _flagp: *mut c_uint,
) -> *mut c_char {
    let sname = ffi::nvim_ses_home_replace_save(name);

    // Always kOptSsopFlagSlash: change all backslashes to forward slashes.
    // Use MB_PTR_ADV equivalent (utfc_ptr2len) to advance past multibyte chars.
    let mut p = sname;
    while *p != 0 {
        if *p == b'\\' as c_char {
            *p = b'/' as c_char;
        }
        let len = ffi::nvim_ses_utfc_ptr2len(p);
        p = p.add(if len > 0 { len as usize } else { 1 });
    }

    // Escape special characters.
    let result = ffi::nvim_ses_vim_strsave_fnameescape(sname);
    ffi::nvim_ses_xfree(sname.cast());
    result
}

/// Write a file name to the session file.
///
/// Takes care of the "slash" option and escapes special characters.
/// Returns OK or FAIL.
///
/// # Safety
/// `fd` must be a valid FILE*. `name` must be a valid C string. `flagp` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_put_fname(
    fd: *mut libc::FILE,
    name: *mut c_char,
    flagp: *mut c_uint,
) -> c_int {
    let p = rs_ses_escape_fname(name, flagp);
    let mut w = crate::writer::SessionWriter::new(fd);

    // Write the escaped filename (fputs equivalent)
    let cstr = std::ffi::CStr::from_ptr(p);
    let retval = if w.write_bytes(cstr.to_bytes()) {
        OK
    } else {
        FAIL
    };
    ffi::nvim_ses_xfree(p.cast());
    retval
}

/// Write a buffer name to the session file, optionally ending the line.
///
/// Returns OK or FAIL.
///
/// # Safety
/// `fd` must be a valid FILE*. `buf` must be a valid buf_T pointer. `flagp` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_fname(
    fd: *mut libc::FILE,
    buf: ffi::BufPtr,
    flagp: *mut c_uint,
    add_eol: bool,
) -> c_int {
    let name = rs_ses_get_fname(buf, flagp);
    if rs_ses_put_fname(fd, name as *mut c_char, flagp) == FAIL {
        return FAIL;
    }
    if add_eol {
        let mut w = crate::writer::SessionWriter::new(fd);
        if !w.write_bytes(b"\n") {
            return FAIL;
        }
    }
    OK
}

// =============================================================================
// Phase 4: Layout Writers
// =============================================================================

/// Write cursor position command for a window.
///
/// If curswant == MAXCOL, writes `{spaces}normal! $`.
/// Otherwise, writes `{spaces}normal! 0{virtcol+1}|`.
///
/// # Safety
/// `fd` must be a valid FILE*. `wp` must be a valid win_T*. `spaces` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_put_view_curpos(
    fd: *mut libc::FILE,
    wp: ffi::WinPtr,
    spaces: *const c_char,
) -> c_int {
    let mut w = crate::writer::SessionWriter::new(fd);
    let sp = if spaces.is_null() {
        ""
    } else {
        CStr::from_ptr(spaces).to_str().unwrap_or("")
    };

    let curswant = ffi::nvim_ses_win_get_curswant(wp);
    let mut buf = String::new();
    if curswant == MAXCOL {
        let _ = writeln!(buf, "{sp}normal! $");
    } else {
        let col = ffi::nvim_ses_win_get_virtcol(wp) + 1;
        let _ = writeln!(buf, "{sp}normal! 0{col}|");
    }

    c_int::from(w.write_bytes(buf.as_bytes()))
}

/// Write window size restoration commands.
///
/// When `restore_size` and `kOptSsopFlagWinsize` are set, writes proportional
/// resize commands for each window. Otherwise writes `wincmd =`.
///
/// # Safety
/// `fd` must be a valid FILE*. `tab_firstwin` must be a valid win_T* or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_winsizes(
    fd: *mut libc::FILE,
    restore_size: bool,
    tab_firstwin: ffi::WinPtr,
) -> c_int {
    let mut w = crate::writer::SessionWriter::new(fd);
    let ssop_flags = ffi::nvim_ses_get_ssop_flags();

    if restore_size && (ssop_flags & K_OPT_SSOP_FLAG_WINSIZE) != 0 {
        let mut n: c_int = 0;
        let mut wp = tab_firstwin;
        while !wp.is_null() {
            if rs_ses_do_win(wp) == 0 {
                wp = ffi::nvim_ses_win_get_next(wp);
                continue;
            }
            n += 1;

            let win_height = ffi::nvim_ses_win_get_height(wp);
            let hsep_height = ffi::nvim_ses_win_get_hsep_height(wp);
            let status_height = ffi::nvim_ses_win_get_status_height(wp);
            let topframe_height = ffi::nvim_ses_topframe_get_height();
            let rows = ffi::nvim_ses_get_Rows();

            // Restore height when not full height
            if win_height + hsep_height + status_height < topframe_height {
                let h = i64::from(win_height);
                let r = i64::from(rows);
                let r2 = r / 2;
                let mut buf = String::new();
                let _ = writeln!(buf, "exe '{n}resize ' . ((&lines * {h} + {r2}) / {r})");
                if !w.write_bytes(buf.as_bytes()) {
                    return FAIL;
                }
            }

            // Restore width when not full width
            let win_width = ffi::nvim_ses_win_get_width(wp);
            let columns = ffi::nvim_ses_get_Columns();
            if win_width < columns {
                let ww = i64::from(win_width);
                let c = i64::from(columns);
                let c2 = c / 2;
                let mut buf = String::new();
                let _ = writeln!(
                    buf,
                    "exe 'vert {n}resize ' . ((&columns * {ww} + {c2}) / {c})"
                );
                if !w.write_bytes(buf.as_bytes()) {
                    return FAIL;
                }
            }

            wp = ffi::nvim_ses_win_get_next(wp);
        }
    } else {
        // Just equalize window sizes.
        if w.put_line(b"wincmd =") == FAIL {
            return FAIL;
        }
    }
    OK
}

/// Write commands to recursively create windows for frame `fr`.
///
/// Writes split commands for horizontally/vertically split frames.
/// After the commands the last window in the frame is the current window.
///
/// # Safety
/// `fd` must be a valid FILE*. `fr` must be a valid frame_T*.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_win_rec(fd: *mut libc::FILE, fr: ffi::FramePtr) -> c_int {
    let mut w = crate::writer::SessionWriter::new(fd);
    let layout = ffi::nvim_ses_frame_get_layout(fr);

    if layout == FR_LEAF {
        return OK;
    }

    // Find first frame that's not skipped and then create a window for
    // each following one (first frame is already there).
    let mut count: c_int = 0;
    let first_frc = rs_ses_skipframe(ffi::nvim_ses_frame_get_child(fr));
    if !first_frc.is_null() {
        let mut frc = rs_ses_skipframe(ffi::nvim_ses_frame_get_next(first_frc));
        while !frc.is_null() {
            // Make window as big as possible so that we have lots of room to split.
            let split_cmd = if layout == FR_COL {
                "wincmd _ | wincmd |\nsplit\n"
            } else {
                "wincmd _ | wincmd |\nvsplit\n"
            };
            if !w.write_bytes(split_cmd.as_bytes()) {
                return FAIL;
            }
            count += 1;
            frc = rs_ses_skipframe(ffi::nvim_ses_frame_get_next(frc));
        }
    }

    // Go back to the first window.
    if count > 0 {
        let mut buf = String::new();
        if layout == FR_COL {
            let _ = writeln!(buf, "{count}wincmd k");
        } else {
            let _ = writeln!(buf, "{count}wincmd h");
        }
        if !w.write_bytes(buf.as_bytes()) {
            return FAIL;
        }
    }

    // Recursively create frames/windows in each window of this column or row.
    let mut frc = rs_ses_skipframe(ffi::nvim_ses_frame_get_child(fr));
    while !frc.is_null() {
        rs_ses_win_rec(fd, frc);
        frc = rs_ses_skipframe(ffi::nvim_ses_frame_get_next(frc));
        // Go to next window.
        if !frc.is_null() && w.put_line(b"wincmd w") == FAIL {
            return FAIL;
        }
    }

    OK
}

/// Write an argument list to the session file.
///
/// Writes the command, then `%argdel`, then `$argadd` for each entry.
///
/// # Safety
/// `fd` must be a valid FILE*. `cmd` must be a valid C string.
/// `gap` must be a valid garray_T*. `flagp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_arglist(
    fd: *mut libc::FILE,
    cmd: *const c_char,
    gap: ffi::GarrayPtr,
    fullname: bool,
    flagp: *mut c_uint,
) -> c_int {
    let mut w = crate::writer::SessionWriter::new(fd);

    // Write the command and %argdel
    let cmd_str = if cmd.is_null() {
        ""
    } else {
        CStr::from_ptr(cmd).to_str().unwrap_or("")
    };
    let header = format!("{cmd_str}\n%argdel\n");
    if !w.write_bytes(header.as_bytes()) {
        return FAIL;
    }

    let ga_len = ffi::nvim_ses_ga_get_len(gap);
    for i in 0..ga_len {
        // NULL file names are skipped (only happens when out of memory).
        let s = ffi::nvim_ses_alist_name_at(gap, i);
        if s.is_null() {
            continue;
        }

        let mut buf_ptr: *mut c_char = std::ptr::null_mut();
        let name = if fullname {
            buf_ptr = ffi::nvim_ses_xmalloc(MAXPATHL);
            ffi::nvim_ses_vim_FullName(s, buf_ptr, MAXPATHL, false);
            buf_ptr
        } else {
            s
        };

        let fname_esc = rs_ses_escape_fname(name, flagp);
        let escaped = CStr::from_ptr(fname_esc).to_str().unwrap_or("");
        let line = format!("$argadd {escaped}\n");
        let ok = w.write_bytes(line.as_bytes());
        ffi::nvim_ses_xfree(fname_esc.cast());
        if !buf_ptr.is_null() {
            ffi::nvim_ses_xfree(buf_ptr.cast());
        }
        if !ok {
            return FAIL;
        }
    }
    OK
}
