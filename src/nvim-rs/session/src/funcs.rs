//! Migrated C function implementations for ex_session.c
//!
//! Each `rs_*` function is the Rust implementation of a corresponding C function.
//! The C function body is replaced with a thin wrapper that calls the `rs_*` version.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_cast_constness)]

use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
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

/// Whether ":lcd" or ":tcd" was produced for a session (formerly C static `did_lcd`).
/// Neovim is single-threaded so a plain static mut is safe here.
static mut DID_LCD: c_int = 0;

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
        && DID_LCD == 0
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

// =============================================================================
// Phase 5: Utility Functions
// =============================================================================

/// Callback function invoked by `nvim_ses_foreach_session_global` for each
/// session-flavoured global variable.
///
/// `var_type`: 0 = number, 1 = string, 2 = float.
/// For types 0/1, `escaped_val` is the escaped string representation.
/// For type 2, `float_val` and `float_sign` are the float value and sign char.
/// `ud` is the FILE* to write to.
unsafe extern "C" fn session_global_callback(
    key: *const c_char,
    var_type: c_int,
    escaped_val: *const c_char,
    float_val: f64,
    float_sign: c_int,
    ud: *mut std::ffi::c_void,
) -> c_int {
    let fd = ud.cast::<libc::FILE>();
    let mut w = crate::writer::SessionWriter::new(fd);
    let key_str = CStr::from_ptr(key).to_str().unwrap_or("");

    match var_type {
        0 => {
            // VAR_NUMBER: let key =  val
            let val = CStr::from_ptr(escaped_val).to_str().unwrap_or("");
            let line = format!("let {key_str} =  {val} \n");
            if !w.write_bytes(line.as_bytes()) {
                return FAIL;
            }
        }
        1 => {
            // VAR_STRING: let key = "val"
            let val = CStr::from_ptr(escaped_val).to_str().unwrap_or("");
            let line = format!("let {key_str} = \"{val}\"\n");
            if !w.write_bytes(line.as_bytes()) {
                return FAIL;
            }
        }
        2 => {
            // VAR_FLOAT: let key = <sign><f>
            #[allow(clippy::cast_possible_truncation)]
            let sign = char::from(float_sign as u8);
            // Use .6 precision to match C's %f format
            let line = format!("let {key_str} = {sign}{float_val:.6}\n");
            if !w.write_bytes(line.as_bytes()) {
                return FAIL;
            }
        }
        _ => {}
    }
    OK
}

/// Store session-flavoured global variables to the session file.
///
/// # Safety
/// `fd` must be a valid FILE*.
#[no_mangle]
pub unsafe extern "C" fn rs_store_session_globals(fd: *mut libc::FILE) -> c_int {
    ffi::nvim_ses_foreach_session_global(session_global_callback, fd.cast())
}

/// Get the view file path for the current buffer.
///
/// Encodes the buffer's full filename into a safe path under 'viewdir':
///   - path separators → "=+"
///   - "=" → "=="
///   - appends "=<c>.vim"
///
/// Returns an xmalloc'd string, or NULL on error (no file name).
///
/// # Safety
/// Accesses global state (curbuf, p_vdir).
#[no_mangle]
pub unsafe extern "C" fn rs_get_view_file(c: c_char) -> *mut c_char {
    let ffname = ffi::nvim_ses_get_curbuf_ffname();
    if ffname.is_null() {
        ffi::nvim_ses_emsg_noname();
        return std::ptr::null_mut();
    }

    let sname = ffi::nvim_ses_home_replace_save(ffname);

    // Count extra bytes needed for escaping
    let mut extra: usize = 0;
    let mut p = sname;
    while *p != 0 {
        if *p == b'=' as c_char || ffi::nvim_ses_vim_ispathsep(c_int::from(*p as u8)) {
            extra += 1;
        }
        p = p.add(1);
    }

    let sname_len = p.offset_from(sname) as usize;
    let vdir = ffi::nvim_ses_get_p_vdir();
    let vdir_len = CStr::from_ptr(vdir).to_bytes().len();

    // Allocate: vdir + separator + encoded_name + "=<c>.vim" + NUL
    // The +9 accounts for separator(1) + "=" (1) + c(1) + ".vim"(4) + NUL(1) + spare(1)
    let alloc_size = vdir_len + sname_len + extra + 9;
    let retval = ffi::nvim_ses_xmalloc(alloc_size);

    // Copy viewdir
    std::ptr::copy_nonoverlapping(vdir.cast::<u8>(), retval.cast::<u8>(), vdir_len);
    *retval.add(vdir_len) = 0; // NUL terminate for add_pathsep
    ffi::nvim_ses_add_pathsep(retval);

    // Find end of retval (after add_pathsep)
    let mut s = retval;
    while *s != 0 {
        s = s.add(1);
    }

    // Encode sname
    p = sname;
    while *p != 0 {
        if *p == b'=' as c_char {
            *s = b'=' as c_char;
            s = s.add(1);
            *s = b'=' as c_char;
        } else if ffi::nvim_ses_vim_ispathsep(c_int::from(*p as u8)) {
            *s = b'=' as c_char;
            s = s.add(1);
            // On Unix, BACKSLASH_IN_FILENAME is not defined, always use '+'
            *s = b'+' as c_char;
        } else {
            *s = *p;
        }
        s = s.add(1);
        p = p.add(1);
    }

    // Append "=<c>.vim\0"
    *s = b'=' as c_char;
    s = s.add(1);
    *s = c;
    s = s.add(1);
    let suffix = b".vim\0";
    std::ptr::copy_nonoverlapping(suffix.as_ptr(), s.cast::<u8>(), suffix.len());

    ffi::nvim_ses_xfree(sname.cast());
    retval
}

// =============================================================================
// Phase 6: put_view (View Writer)
// =============================================================================

/// Additional session option flag constants (verified via _Static_assert in C)
pub const K_OPT_SSOP_FLAG_CURSOR: c_uint = 0x4000;
pub const K_OPT_SSOP_FLAG_OPTIONS: c_uint = 0x20;
pub const K_OPT_SSOP_FLAG_LOCALOPTIONS: c_uint = 0x10;
pub const K_OPT_SSOP_FLAG_FOLDS: c_uint = 0x2000;

/// OPT_LOCAL flag for makeset (verified via _Static_assert in C)
const OPT_LOCAL: c_int = 0x02;

/// Write commands to restore the view of a window.
///
/// # Safety
/// All pointers must be valid. `flagp` must point to either ssop_flags or vop_flags.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_put_view(
    fd: *mut libc::FILE,
    wp: ffi::WinPtr,
    tp: ffi::TabpagePtr,
    add_edit: bool,
    flagp: *mut c_uint,
    current_arg_idx: c_int,
) -> c_int {
    let mut w = crate::writer::SessionWriter::new(fd);
    let ssop_ptr = ffi::nvim_ses_get_ssop_flags_ptr();
    let vop_ptr = ffi::nvim_ses_get_vop_flags_ptr();
    let opt_flags = *flagp;
    let is_session = std::ptr::eq(flagp.cast_const(), ssop_ptr);

    // Always restore cursor position for ":mksession". For ":mkview" only
    // when 'viewoptions' contains "cursor".
    let mut do_cursor = is_session || (opt_flags & K_OPT_SSOP_FLAG_CURSOR) != 0;

    // Local argument list.
    if ffi::nvim_ses_win_uses_global_alist(wp) {
        if w.put_line(b"argglobal") == FAIL {
            return FAIL;
        }
    } else {
        let is_vop = std::ptr::eq(flagp.cast_const(), vop_ptr);
        let fullname = is_vop
            || (opt_flags & K_OPT_SSOP_FLAG_CURDIR) == 0
            || !ffi::nvim_ses_tp_get_localdir(tp).is_null()
            || !ffi::nvim_ses_win_get_localdir(wp).is_null();
        let ga = ffi::nvim_ses_win_get_alist_ga(wp);
        if rs_ses_arglist(fd, c"arglocal".as_ptr(), ga, fullname, flagp) == FAIL {
            return FAIL;
        }
    }

    // Restore the argument index (session mode only).
    let mut did_next = false;
    let arg_idx = ffi::nvim_ses_win_get_arg_idx(wp);
    if arg_idx != current_arg_idx && arg_idx < ffi::nvim_ses_win_wargcount(wp) && is_session {
        let mut buf = String::new();
        let idx = i64::from(arg_idx) + 1;
        let _ = writeln!(buf, "{idx}argu");
        if !w.write_bytes(buf.as_bytes()) {
            return FAIL;
        }
        did_next = true;
    }

    // Edit the file. Skip this when ":next" already did it.
    let buf_handle = ffi::nvim_ses_win_get_buffer(wp);
    if add_edit && (!did_next || ffi::nvim_ses_win_get_arg_idx_invalid(wp)) {
        let fname_raw = rs_ses_get_fname(buf_handle, flagp);
        let fname_esc = rs_ses_escape_fname(fname_raw as *mut c_char, flagp);

        if ffi::nvim_ses_bt_help(buf_handle) {
            // Help buffer
            let mut curtag_ptr: *const c_char = c"".as_ptr();
            let tsidx = ffi::nvim_ses_win_get_tagstackidx(wp);
            let tslen = ffi::nvim_ses_win_get_tagstacklen(wp);
            if tsidx > 0 && tsidx <= tslen {
                curtag_ptr = ffi::nvim_ses_win_get_tagname(wp, tsidx - 1);
            }

            if w.put_line(b"enew | setl bt=help") == FAIL {
                ffi::nvim_ses_xfree(fname_esc.cast());
                return FAIL;
            }
            let curtag = CStr::from_ptr(curtag_ptr).to_str().unwrap_or("");
            let line = format!("help {curtag}");
            if !w.write_bytes(line.as_bytes()) || w.put_eol() == FAIL {
                ffi::nvim_ses_xfree(fname_esc.cast());
                return FAIL;
            }
        } else if !ffi::nvim_ses_buf_get_ffname(buf_handle).is_null()
            && (!ffi::nvim_ses_bt_nofilename(buf_handle)
                || ffi::nvim_ses_buf_is_terminal(buf_handle))
        {
            // File buffer: use :edit or :buffer
            let fe = CStr::from_ptr(fname_esc).to_str().unwrap_or("");
            let block = format!(
                "if bufexists(fnamemodify(\"{fe}\", \":p\")) | buffer {fe} | else | edit {fe} | endif\n\
                 if &buftype ==# 'terminal'\n\
                   silent file {fe}\n\
                 endif\n"
            );
            if !w.write_bytes(block.as_bytes()) {
                ffi::nvim_ses_xfree(fname_esc.cast());
                return FAIL;
            }
        } else {
            // No file, just enew
            if w.put_line(b"enew") == FAIL {
                ffi::nvim_ses_xfree(fname_esc.cast());
                return FAIL;
            }
            if !ffi::nvim_ses_buf_get_ffname(buf_handle).is_null() {
                let fe = CStr::from_ptr(fname_esc).to_str().unwrap_or("");
                let line = format!("file {fe}\n");
                if !w.write_bytes(line.as_bytes()) {
                    ffi::nvim_ses_xfree(fname_esc.cast());
                    return FAIL;
                }
            }
            do_cursor = false;
        }
        ffi::nvim_ses_xfree(fname_esc.cast());
    }

    // Alternate file
    let alt_fnum = ffi::nvim_ses_win_get_alt_fnum(wp);
    if alt_fnum != 0 {
        let alt = ffi::nvim_ses_buflist_findnr(alt_fnum);
        if is_session
            && !alt.is_null()
            && !ffi::nvim_ses_buf_get_fname(alt).is_null()
            && *ffi::nvim_ses_buf_get_fname(alt) != 0
            && ffi::nvim_ses_buf_get_p_bl(alt)
            && !(ffi::nvim_ses_bt_terminal(alt)
                && (ffi::nvim_ses_get_ssop_flags() & K_OPT_SSOP_FLAG_TERMINAL) == 0)
            && (!w.write_bytes(b"balt ") || rs_ses_fname(fd, alt, flagp, true) == FAIL)
        {
            return FAIL;
        }
    }

    // Local mappings and abbreviations.
    if (opt_flags & (K_OPT_SSOP_FLAG_OPTIONS | K_OPT_SSOP_FLAG_LOCALOPTIONS)) != 0
        && ffi::nvim_ses_makemap(fd, buf_handle) == FAIL
    {
        return FAIL;
    }

    // Local options. Need to go to the window temporarily.
    let save_curwin = ffi::nvim_ses_get_curwin();
    ffi::nvim_ses_set_curwin(wp);
    let f = if (opt_flags & (K_OPT_SSOP_FLAG_OPTIONS | K_OPT_SSOP_FLAG_LOCALOPTIONS)) != 0 {
        let is_vop = std::ptr::eq(flagp.cast_const(), vop_ptr);
        let local_only = is_vop || (opt_flags & K_OPT_SSOP_FLAG_OPTIONS) == 0;
        ffi::nvim_ses_makeset(fd, OPT_LOCAL, local_only)
    } else if (opt_flags & K_OPT_SSOP_FLAG_FOLDS) != 0 {
        ffi::nvim_ses_makefoldset(fd)
    } else {
        OK
    };
    ffi::nvim_ses_set_curwin(save_curwin);
    if f == FAIL {
        return FAIL;
    }

    // Save folds when 'buftype' is empty and for help files.
    if (opt_flags & K_OPT_SSOP_FLAG_FOLDS) != 0
        && !ffi::nvim_ses_buf_get_ffname(buf_handle).is_null()
        && (ffi::nvim_ses_bt_normal(buf_handle) || ffi::nvim_ses_bt_help(buf_handle))
        && ffi::nvim_ses_put_folds(fd, wp) == FAIL
    {
        return FAIL;
    }

    // Set the cursor after creating folds, since that moves the cursor.
    if do_cursor {
        let cursor_lnum = ffi::nvim_ses_win_get_cursor_lnum(wp);
        let view_height = ffi::nvim_ses_win_get_view_height(wp);

        if view_height <= 0 {
            let line = format!("let s:l = {cursor_lnum}\n");
            if !w.write_bytes(line.as_bytes()) {
                return FAIL;
            }
        } else {
            let topline = ffi::nvim_ses_win_get_topline(wp);
            let diff = cursor_lnum - topline;
            let half = view_height / 2;
            let line = format!(
                "let s:l = {cursor_lnum} - (({diff} * winheight(0) + {half}) / {view_height})\n"
            );
            if !w.write_bytes(line.as_bytes()) {
                return FAIL;
            }
        }

        let block = format!(
            "if s:l < 1 | let s:l = 1 | endif\n\
             keepjumps exe s:l\n\
             normal! zt\n\
             keepjumps {cursor_lnum}\n"
        );
        if !w.write_bytes(block.as_bytes()) {
            return FAIL;
        }

        // Restore cursor column and left offset when not wrapping.
        let cursor_col = ffi::nvim_ses_win_get_cursor_col(wp);
        if cursor_col == 0 {
            if w.put_line(b"normal! 0") == FAIL {
                return FAIL;
            }
        } else {
            let wrap = ffi::nvim_ses_win_get_p_wrap(wp);
            let leftcol = ffi::nvim_ses_win_get_leftcol(wp);
            let win_width = ffi::nvim_ses_win_get_width(wp);
            let virtcol = ffi::nvim_ses_win_get_virtcol(wp);

            if !wrap && leftcol > 0 && win_width > 0 {
                let vc1 = i64::from(virtcol) + 1;
                let vl = i64::from(virtcol - leftcol);
                let wh = i64::from(win_width) / 2;
                let ww = i64::from(win_width);
                let block = format!(
                    "let s:c = {vc1} - (({vl} * winwidth(0) + {wh}) / {ww})\n\
                     if s:c > 0\n\
                       exe 'normal! ' . s:c . '|zs' . {vc1} . '|'\n\
                     else\n"
                );
                if !w.write_bytes(block.as_bytes())
                    || rs_put_view_curpos(fd, wp, c"  ".as_ptr()) == 0
                    || w.put_line(b"endif") == FAIL
                {
                    return FAIL;
                }
            } else if rs_put_view_curpos(fd, wp, c"".as_ptr()) == 0 {
                return FAIL;
            }
        }
    }

    // Local directory
    let localdir = ffi::nvim_ses_win_get_localdir(wp);
    let is_vop = std::ptr::eq(flagp.cast_const(), vop_ptr);
    if !localdir.is_null() && (!is_vop || (opt_flags & K_OPT_SSOP_FLAG_CURDIR) != 0) {
        if !w.write_bytes(b"lcd ")
            || rs_ses_put_fname(fd, localdir, flagp) == FAIL
            || !w.write_bytes(b"\n")
        {
            return FAIL;
        }
        DID_LCD = 1;
    }

    OK
}

// =============================================================================
// Phase 7: makeopens (Session Orchestrator)
// =============================================================================

/// Additional session option flag constants (verified via _Static_assert in C)
pub const K_OPT_SSOP_FLAG_BUFFERS: c_uint = 0x01;
pub const K_OPT_SSOP_FLAG_GLOBALS: c_uint = 0x100;
pub const K_OPT_SSOP_FLAG_TABPAGES: c_uint = 0x8000;
pub const K_OPT_SSOP_FLAG_RESIZE: c_uint = 0x04;

/// State passed through buffer iteration callback.
struct BufIterState {
    fd: *mut libc::FILE,
    ssop_flags: c_uint,
    only_save_windows: bool,
}

/// Callback for `nvim_ses_foreach_buffer` — writes `badd +N fname` for each saveable buffer.
unsafe extern "C" fn makeopens_buf_callback(
    buf: ffi::BufPtr,
    _only_save_windows: bool,
    ud: *mut std::ffi::c_void,
) -> c_int {
    let state = &*(ud.cast::<BufIterState>());
    let ssop = state.ssop_flags;

    // Filter: skip buffers we don't want to save
    if state.only_save_windows && ffi::nvim_ses_buf_get_nwindows(buf) == 0 {
        return OK;
    }
    if ffi::nvim_ses_buf_is_help(buf) && (ssop & K_OPT_SSOP_FLAG_HELP) == 0 {
        return OK;
    }
    if ffi::nvim_ses_bt_terminal(buf) && (ssop & K_OPT_SSOP_FLAG_TERMINAL) == 0 {
        return OK;
    }
    if ffi::nvim_ses_buf_get_fname(buf).is_null() {
        return OK;
    }
    if !ffi::nvim_ses_buf_get_p_bl(buf) {
        return OK;
    }

    let mut w = crate::writer::SessionWriter::new(state.fd);
    let lnum = ffi::nvim_ses_buf_get_wininfo_lnum(buf);
    let header = format!("badd +{lnum} ");
    if !w.write_bytes(header.as_bytes())
        || rs_ses_fname(
            state.fd,
            buf,
            ffi::nvim_ses_get_ssop_flags_ptr() as *mut c_uint,
            true,
        ) == FAIL
    {
        return FAIL;
    }
    OK
}

/// Write commands for restoring the current buffers/windows/tabs for :mksession.
///
/// # Safety
/// `fd` must be a valid FILE*. `dirnow` must be a valid C string.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_makeopens(fd: *mut libc::FILE, dirnow: *mut c_char) -> c_int {
    let mut w = crate::writer::SessionWriter::new(fd);
    let ssop = ffi::nvim_ses_get_ssop_flags();
    let ssop_ptr = ffi::nvim_ses_get_ssop_flags_ptr() as *mut c_uint;

    let only_save_windows = (ssop & K_OPT_SSOP_FLAG_BUFFERS) == 0;
    let mut restore_size = true;
    let mut edited_win: ffi::WinPtr = std::ptr::null_mut();
    let mut cur_arg_idx: c_int = 0;
    let mut next_arg_idx: c_int = 0;

    // Begin by setting v:this_session, and then other sessionable variables.
    if w.put_line(b"let v:this_session=expand(\"<sfile>:p\")") == FAIL {
        return FAIL;
    }
    if (ssop & K_OPT_SSOP_FLAG_GLOBALS) != 0 && rs_store_session_globals(fd) == FAIL {
        return FAIL;
    }

    // Close all windows and tabs but one.
    if w.put_line(b"silent only") == FAIL {
        return FAIL;
    }
    if (ssop & K_OPT_SSOP_FLAG_TABPAGES) != 0 && w.put_line(b"silent tabonly") == FAIL {
        return FAIL;
    }

    // Now a :cd command to the session directory or the current directory
    if (ssop & K_OPT_SSOP_FLAG_SESDIR) != 0 {
        if w.put_line(b"exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')") == FAIL {
            return FAIL;
        }
    } else if (ssop & K_OPT_SSOP_FLAG_CURDIR) != 0 {
        let gdir = ffi::nvim_ses_get_globaldir();
        let dir = if gdir.is_null() {
            dirnow
        } else {
            gdir as *mut c_char
        };
        let sname = ffi::nvim_ses_home_replace_save(dir);
        let fname_esc = rs_ses_escape_fname(sname, ssop_ptr);
        let fe = CStr::from_ptr(fname_esc).to_str().unwrap_or("");
        let line = format!("cd {fe}\n");
        let ok = w.write_bytes(line.as_bytes());
        ffi::nvim_ses_xfree(fname_esc.cast());
        ffi::nvim_ses_xfree(sname.cast());
        if !ok {
            return FAIL;
        }
    }

    // Check for empty unnamed buffer
    if !w.write_bytes(
        b"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n\
  let s:wipebuf = bufnr('%')\nendif\n",
    ) {
        return FAIL;
    }

    // Save 'shortmess' if not storing options.
    if (ssop & K_OPT_SSOP_FLAG_OPTIONS) == 0
        && w.put_line(b"let s:shortmess_save = &shortmess") == FAIL
    {
        return FAIL;
    }

    if w.put_line(b"set shortmess+=aoO") == FAIL {
        return FAIL;
    }

    // Save all buffers into the buffer list.
    let mut buf_state = BufIterState {
        fd,
        ssop_flags: ssop,
        only_save_windows,
    };
    if ffi::nvim_ses_foreach_buffer(
        makeopens_buf_callback,
        only_save_windows,
        std::ptr::from_mut(&mut buf_state).cast(),
    ) == FAIL
    {
        return FAIL;
    }

    // The global argument list
    let ga = ffi::nvim_ses_get_global_alist_ga();
    let fullname = (ssop & K_OPT_SSOP_FLAG_CURDIR) == 0;
    if rs_ses_arglist(fd, c"argglobal".as_ptr(), ga, fullname, ssop_ptr) == FAIL {
        return FAIL;
    }

    // Resize
    if (ssop & K_OPT_SSOP_FLAG_RESIZE) != 0 {
        let rows = i64::from(ffi::nvim_ses_get_Rows());
        let cols = i64::from(ffi::nvim_ses_get_Columns());
        let line = format!("set lines={rows} columns={cols}\n");
        if !w.write_bytes(line.as_bytes()) {
            return FAIL;
        }
    }

    // Showtabline workaround
    let mut restore_stal = false;
    let first_tp = ffi::nvim_ses_get_first_tabpage();
    if ffi::nvim_ses_get_p_stal() == 1 && !ffi::nvim_ses_tp_get_next(first_tp).is_null() {
        if w.put_line(b"set stal=2") == FAIL {
            return FAIL;
        }
        restore_stal = true;
    }

    // Pre-populate tab pages
    let do_tabpages = (ssop & K_OPT_SSOP_FLAG_TABPAGES) != 0;
    if do_tabpages {
        let mut tp_iter = first_tp;
        while !tp_iter.is_null() {
            if !ffi::nvim_ses_tp_get_next(tp_iter).is_null()
                && w.put_line(b"tabnew +setlocal\\ bufhidden=wipe") == FAIL
            {
                return FAIL;
            }
            tp_iter = ffi::nvim_ses_tp_get_next(tp_iter);
        }
        if !ffi::nvim_ses_tp_get_next(first_tp).is_null() && w.put_line(b"tabrewind") == FAIL {
            return FAIL;
        }
    }

    // Main tab page loop
    let mut restore_height_width = false;
    let curtab = ffi::nvim_ses_get_curtab();
    let curwin_ptr = ffi::nvim_ses_get_curwin();
    let mut tp = if do_tabpages { first_tp } else { curtab };

    loop {
        let mut need_tabnext = false;
        let mut cnr: c_int = 1;
        let tab_firstwin;
        let tab_topframe;

        if do_tabpages {
            if std::ptr::eq(tp, curtab) {
                tab_firstwin = ffi::nvim_ses_get_firstwin();
                tab_topframe = ffi::nvim_ses_get_topframe();
            } else {
                tab_firstwin = ffi::nvim_ses_tp_get_firstwin(tp);
                tab_topframe = ffi::nvim_ses_tp_get_topframe(tp);
            }
            if !std::ptr::eq(tp, first_tp) {
                need_tabnext = true;
            }
        } else {
            tab_firstwin = ffi::nvim_ses_get_firstwin();
            tab_topframe = ffi::nvim_ses_get_topframe();
        }

        // Try loading one file first.
        let mut wp = tab_firstwin;
        while !wp.is_null() {
            let buf = ffi::nvim_ses_win_get_buffer(wp);
            if rs_ses_do_win(wp) != 0
                && !ffi::nvim_ses_buf_get_ffname(buf).is_null()
                && !ffi::nvim_ses_bt_help(buf)
                && !ffi::nvim_ses_bt_nofilename(buf)
            {
                if need_tabnext && w.put_line(b"tabnext") == FAIL {
                    return FAIL;
                }
                need_tabnext = false;

                if !w.write_bytes(b"edit ") || rs_ses_fname(fd, buf, ssop_ptr, true) == FAIL {
                    return FAIL;
                }
                if !ffi::nvim_ses_win_get_arg_idx_invalid(wp) {
                    edited_win = wp;
                }
                break;
            }
            wp = ffi::nvim_ses_win_get_next(wp);
        }

        // If no file got edited create an empty tab page.
        if need_tabnext && w.put_line(b"tabnext") == FAIL {
            return FAIL;
        }

        if ffi::nvim_ses_frame_get_layout(tab_topframe) != FR_LEAF
            && (w.put_line(b"let s:save_splitbelow = &splitbelow") == FAIL
                || w.put_line(b"let s:save_splitright = &splitright") == FAIL
                || w.put_line(b"set splitbelow splitright") == FAIL
                || rs_ses_win_rec(fd, tab_topframe) == FAIL
                || w.put_line(b"let &splitbelow = s:save_splitbelow") == FAIL
                || w.put_line(b"let &splitright = s:save_splitright") == FAIL)
        {
            return FAIL;
        }

        // Count windows, check sizes can be restored
        let mut nr: c_int = 0;
        wp = tab_firstwin;
        while !wp.is_null() {
            if rs_ses_do_win(wp) != 0 {
                nr += 1;
            } else {
                restore_size = false;
            }
            if std::ptr::eq(curwin_ptr, wp) {
                cnr = nr;
            }
            wp = ffi::nvim_ses_win_get_next(wp);
        }

        if !tab_firstwin.is_null() && !ffi::nvim_ses_win_get_next(tab_firstwin).is_null() {
            if w.put_line(b"wincmd t") == FAIL
                || w.put_line(b"let s:save_winminheight = &winminheight") == FAIL
                || w.put_line(b"let s:save_winminwidth = &winminwidth") == FAIL
                || !w.write_bytes(
                    b"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1\n",
                )
            {
                return FAIL;
            }
            restore_height_width = true;
        }
        if nr > 1 && rs_ses_winsizes(fd, restore_size, tab_firstwin) == FAIL {
            return FAIL;
        }

        // Tab-local working directory
        if (ssop & K_OPT_SSOP_FLAG_CURDIR) != 0 && !ffi::nvim_ses_tp_get_localdir(tp).is_null() {
            let tpdir = ffi::nvim_ses_tp_get_localdir(tp);
            if !w.write_bytes(b"tcd ")
                || rs_ses_put_fname(fd, tpdir, ssop_ptr) == FAIL
                || w.put_eol() == FAIL
            {
                return FAIL;
            }
            DID_LCD = 1;
        }

        // Restore view of each window
        wp = tab_firstwin;
        while !wp.is_null() {
            if rs_ses_do_win(wp) == 0 {
                wp = ffi::nvim_ses_win_get_next(wp);
                continue;
            }
            if rs_put_view(
                fd,
                wp,
                tp,
                !std::ptr::eq(wp, edited_win),
                ssop_ptr,
                cur_arg_idx,
            ) == FAIL
            {
                return FAIL;
            }
            if nr > 1 && w.put_line(b"wincmd w") == FAIL {
                return FAIL;
            }
            next_arg_idx = ffi::nvim_ses_win_get_arg_idx(wp);
            wp = ffi::nvim_ses_win_get_next(wp);
        }

        cur_arg_idx = next_arg_idx;

        // Restore cursor to the current window
        if cnr > 1 {
            let line = format!("{cnr}wincmd w\n");
            if !w.write_bytes(line.as_bytes()) {
                return FAIL;
            }
        }

        // Restore window sizes again
        if nr > 1 && rs_ses_winsizes(fd, restore_size, tab_firstwin) == FAIL {
            return FAIL;
        }

        if !do_tabpages {
            break;
        }
        tp = ffi::nvim_ses_tp_get_next(tp);
        if tp.is_null() {
            break;
        }
    }

    if do_tabpages {
        let idx = ffi::nvim_ses_tabpage_index(curtab);
        let line = format!("tabnext {idx}\n");
        if !w.write_bytes(line.as_bytes()) {
            return FAIL;
        }
    }
    if restore_stal && w.put_line(b"set stal=1") == FAIL {
        return FAIL;
    }

    // Wipe out an empty unnamed buffer we started in.
    if !w.write_bytes(
        b"if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0\
 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'\n\
  silent exe 'bwipe ' . s:wipebuf\n\
endif\n\
unlet! s:wipebuf\n",
    ) {
        return FAIL;
    }

    // Re-apply 'winheight' and 'winwidth'.
    let wh = ffi::nvim_ses_get_p_wh();
    let wiw = ffi::nvim_ses_get_p_wiw();
    let line = format!("set winheight={wh} winwidth={wiw}\n");
    if !w.write_bytes(line.as_bytes()) {
        return FAIL;
    }

    // Restore 'shortmess'.
    if (ssop & K_OPT_SSOP_FLAG_OPTIONS) != 0 {
        let shm = CStr::from_ptr(ffi::nvim_ses_get_p_shm())
            .to_str()
            .unwrap_or("");
        let line = format!("set shortmess={shm}\n");
        if !w.write_bytes(line.as_bytes()) {
            return FAIL;
        }
    } else if w.put_line(b"let &shortmess = s:shortmess_save") == FAIL {
        return FAIL;
    }

    if restore_height_width
        && (w.put_line(b"let &winminheight = s:save_winminheight") == FAIL
            || w.put_line(b"let &winminwidth = s:save_winminwidth") == FAIL)
    {
        return FAIL;
    }

    // Lastly, execute the x.vim file if it exists.
    if !w.write_bytes(
        b"let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"\n\
if filereadable(s:sx)\n\
  exe \"source \" . fnameescape(s:sx)\n\
endif\n",
    ) {
        return FAIL;
    }

    OK
}

// =============================================================================
// Phase 8: Public Entry Points (ex_mkrc, ex_loadview)
// =============================================================================

/// Phase 8 constants (verified via _Static_assert in C)
const K_OPT_SSOP_FLAG_SKIPRTP: c_uint = 0x20000;
const OPT_GLOBAL: c_int = 0x01;
const OPT_SKIPRTP: c_int = 0x80;

/// Filename constants (matching C macros VIMRC_FILE, SESSION_FILE, EXRC_FILE)
const VIMRC_FILE: *const c_char = c".nvimrc".as_ptr();
const SESSION_FILE: *const c_char = c"Session.vim".as_ptr();
const EXRC_FILE: *const c_char = c".exrc".as_ptr();

/// `:loadview [nr]` — load a view file.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_loadview"]
pub unsafe extern "C" fn rs_ex_loadview(eap: ffi::ExargPtr) {
    let arg = ffi::nvim_ses_eap_get_arg(eap);
    let c = *arg;
    let fname = rs_get_view_file(c);
    if fname.is_null() {
        return;
    }

    if ffi::nvim_ses_do_source(fname) == FAIL {
        let e_notopen = ffi::nvim_ses_get_e_notopen();
        ffi::nvim_ses_semsg(e_notopen, fname);
    }
    ffi::nvim_ses_xfree(fname.cast::<c_void>());
}

/// `:mkexrc`, `:mkvimrc`, `:mkview`, `:mksession`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_mkrc"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_ex_mkrc(eap: ffi::ExargPtr) {
    let cmd_mksession = ffi::nvim_ses_get_CMD_mksession();
    let cmd_mkview = ffi::nvim_ses_get_CMD_mkview();
    let cmd_mkvimrc = ffi::nvim_ses_get_CMD_mkvimrc();

    let cmdidx = ffi::nvim_ses_eap_get_cmdidx(eap);

    let view_session = cmdidx == cmd_mksession || cmdidx == cmd_mkview;

    // Use the short file name until ":lcd" is used.
    DID_LCD = 0;

    let mut view_file: *mut c_char = std::ptr::null_mut();
    let mut using_vdir = false;

    let arg = ffi::nvim_ses_eap_get_arg(eap);
    let arg_byte = *arg as u8;

    let fname: *mut c_char;

    // ":mkview" or ":mkview 9": generate file name with 'viewdir'
    if cmdidx == cmd_mkview && (arg_byte == 0 || (arg_byte.is_ascii_digit() && *arg.add(1) == 0)) {
        ffi::nvim_ses_eap_set_forceit(eap, true);
        let f = rs_get_view_file(*arg);
        if f.is_null() {
            return;
        }
        fname = f;
        view_file = f;
        using_vdir = true;
    } else if arg_byte != 0 {
        fname = arg;
    } else if cmdidx == cmd_mkvimrc {
        fname = VIMRC_FILE as *mut c_char;
    } else if cmdidx == cmd_mksession {
        fname = SESSION_FILE as *mut c_char;
    } else {
        fname = EXRC_FILE as *mut c_char;
    }

    // When using 'viewdir' may have to create the directory.
    if using_vdir && !ffi::nvim_ses_os_isdir(ffi::nvim_ses_get_p_vdir()) {
        ffi::nvim_ses_vim_mkdir_emsg(ffi::nvim_ses_get_p_vdir(), 0o755);
    }

    #[allow(clippy::as_ptr_cast_mut)]
    let mode = c"wb".as_ptr() as *mut c_char;
    let fd =
        ffi::nvim_ses_open_exfile(fname, c_int::from(ffi::nvim_ses_eap_get_forceit(eap)), mode);
    if !fd.is_null() {
        let mut failed = false;
        let flagp: *mut c_uint = if cmdidx == cmd_mkview {
            ffi::nvim_ses_get_vop_flags_ptr() as *mut c_uint
        } else {
            ffi::nvim_ses_get_ssop_flags_ptr() as *mut c_uint
        };

        let mut w = crate::SessionWriter::new(fd);

        // Write the version command for :mkvimrc
        if cmdidx == cmd_mkvimrc {
            w.put_line(b"version 6.0");
        }

        if cmdidx == cmd_mksession && w.put_line(b"let SessionLoad = 1") == FAIL {
            failed = true;
        }

        if !view_session || (cmdidx == cmd_mksession && (*flagp & K_OPT_SSOP_FLAG_OPTIONS) != 0) {
            let mut opt_flags = OPT_GLOBAL;
            if cmdidx == cmd_mksession && (*flagp & K_OPT_SSOP_FLAG_SKIPRTP) != 0 {
                opt_flags |= OPT_SKIPRTP;
            }
            failed |= ffi::nvim_ses_makemap(fd, std::ptr::null_mut()) == FAIL
                || ffi::nvim_ses_makeset(fd, opt_flags, false) == FAIL;
        }

        if !failed && view_session {
            if w.put_line(
                b"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1",
            ) == FAIL
            {
                failed = true;
            }
            if cmdidx == cmd_mksession {
                let dirnow = ffi::nvim_ses_xmalloc(MAXPATHL);

                // Change to session file's dir.
                if ffi::nvim_ses_os_dirname(dirnow, MAXPATHL) == FAIL
                    || ffi::nvim_ses_os_chdir(dirnow) != 0
                {
                    *dirnow = 0;
                }

                let ssop = ffi::nvim_ses_get_ssop_flags();
                let gdir = ffi::nvim_ses_get_globaldir();

                if *dirnow != 0 && (ssop & K_OPT_SSOP_FLAG_SESDIR) != 0 {
                    if ffi::nvim_ses_vim_chdirfile(fname) == OK {
                        ffi::nvim_ses_shorten_fnames(1);
                    }
                } else if *dirnow != 0
                    && (ssop & K_OPT_SSOP_FLAG_CURDIR) != 0
                    && !gdir.is_null()
                    && ffi::nvim_ses_os_chdir(gdir) == 0
                {
                    ffi::nvim_ses_shorten_fnames(1);
                }

                failed |= rs_makeopens(fd, dirnow) == FAIL;

                // restore original dir
                if *dirnow != 0
                    && ((ssop & K_OPT_SSOP_FLAG_SESDIR) != 0
                        || ((ssop & K_OPT_SSOP_FLAG_CURDIR) != 0 && !gdir.is_null()))
                {
                    if ffi::nvim_ses_os_chdir(dirnow) != 0 {
                        ffi::nvim_ses_emsg(ffi::nvim_ses_get_e_prev_dir());
                    }
                    ffi::nvim_ses_shorten_fnames(1);
                }
                ffi::nvim_ses_xfree(dirnow.cast::<c_void>());
            } else {
                let curwin = ffi::nvim_ses_get_curwin();
                let curtab = ffi::nvim_ses_get_curtab();
                failed |= rs_put_view(fd, curwin, curtab, !using_vdir, flagp, -1) == FAIL;
            }
            if !w.write_bytes(b"let &g:so = s:so_save | let &g:siso = s:siso_save\n") {
                failed = true;
            }
            if ffi::nvim_ses_get_p_hls() && !w.write_bytes(b"set hlsearch\n") {
                failed = true;
            }
            if ffi::nvim_ses_get_no_hlsearch() && !w.write_bytes(b"nohlsearch\n") {
                failed = true;
            }
            if !w.write_bytes(b"doautoall SessionLoadPost\n") {
                failed = true;
            }
            if cmdidx == cmd_mksession && !w.write_bytes(b"unlet SessionLoad\n") {
                failed = true;
            }
        }
        if w.put_line(b"\" vim: set ft=vim :") == FAIL {
            failed = true;
        }

        failed |= libc::fclose(fd) != 0;

        if failed {
            ffi::nvim_ses_emsg(ffi::nvim_ses_get_e_write());
        } else if cmdidx == cmd_mksession {
            // successful session write - set v:this_session
            let tbuf = ffi::nvim_ses_xmalloc(MAXPATHL);
            if ffi::nvim_ses_vim_FullName(fname, tbuf, MAXPATHL, false) == OK {
                ffi::nvim_ses_set_vim_var_string(tbuf);
            }
            ffi::nvim_ses_xfree(tbuf.cast::<c_void>());
        }
    }

    ffi::nvim_ses_xfree(view_file.cast::<c_void>());

    ffi::nvim_ses_apply_autocmds_session();
}
