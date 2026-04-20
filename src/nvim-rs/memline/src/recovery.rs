//! Recovery and attention file handling for the memline system.
//!
//! This module provides Rust wrappers for swap file recovery operations,
//! including:
//! - Finding and listing swap files for recovery
//! - Reading swap file metadata (block 0 info)
//! - Handling the "ATTENTION - swap file exists" dialog
//! - Swap file name resolution
//!
//! # Recovery Process
//!
//! When Neovim encounters an existing swap file, it can:
//! 1. List available swap files with `recover_names()`
//! 2. Show swap file info with `swapfile_info()` / `swapfile_dict()`
//! 3. Present the ATTENTION dialog with `attention_message()`
//! 4. Recover content with `ml_recover()`
//!
//! # Safety
//!
//! Recovery operations read from potentially corrupted files and should
//! be prepared to handle invalid data gracefully.

use std::ffi::{c_char, c_int, c_uint, c_void, CString};

use nvim_buffer::buf_struct::BufStruct;

use crate::block_ops;
use crate::types::{
    BlockNr, BufHandle, InfoPtr, B0_FF_MASK, B0_FNAME_SIZE_NOCRYPT, B0_HAS_FENC, B0_VERSION_SIZE,
    DATA_BLOCK_HEADER_SIZE, DATA_ID, HLF_E, MIN_SWAP_PAGE_SIZE, PTR_ID, UPD_NOT_VALID,
};

// =============================================================================
// C Implementation Declarations
// =============================================================================

extern "C" {
    static mut got_int: bool;
    // -------------------------------------------------------------------------
    // recover_names helpers (called from rs_recover_names)
    // -------------------------------------------------------------------------

    /// Get the p_dir global option string
    fn nvim_get_p_dir() -> *mut c_char;

    /// Get the swap file name of the current buffer's memfile (or NULL)
    fn nvim_buf_get_ml_mfp_fname(buf: *mut BufHandle) -> *mut c_char;

    /// Get current buffer pointer
    fn nvim_get_curbuf() -> *mut BufHandle;

    /// Iterate through comma-separated option parts
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: c_int,
        sep_chars: *const c_char,
    ) -> c_int;

    /// Concatenate file names with optional separator
    fn concat_fnames(fname1: *const c_char, fname2: *const c_char, sep: c_int) -> *mut c_char;

    /// Generate a modified file name (change/append extension)
    fn modname(fname: *const c_char, ext: *const c_char, prepend_dot: c_int) -> *mut c_char;

    /// Return pointer to tail of file name (past last path separator)
    fn path_tail(fname: *const c_char) -> *mut c_char;

    /// Return true if `p` points after a path separator in string starting at `b`
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;

    /// Compare two file paths by identity (returns FileComparison bitmask)
    fn path_full_compare(
        s1: *mut c_char,
        s2: *mut c_char,
        checkname: c_int,
        expandenv: c_int,
    ) -> c_int;

    /// Check whether a path exists on the filesystem
    fn os_path_exists(name: *const c_char) -> c_int;

    /// Expand wildcards in a list of patterns
    fn expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_files: *mut c_int,
        files: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;

    /// Free an array of file names returned by expand_wildcards
    fn FreeWild(count: c_int, files: *mut *mut c_char);

    /// Print a message to start scrolling
    fn msg(s: *const c_char, hl_id: c_int) -> c_int;

    /// Output a single character to the message area
    fn msg_putchar(c: c_int);

    /// Output a string to the message area
    fn msg_puts(s: *const c_char);

    /// Output a number to the message area
    fn msg_outnum(n: c_int);

    /// Output a file name with home directory replaced by ~
    fn msg_home_replace(fname: *const c_char);

    /// Flush the UI
    fn ui_flush();

    /// Append an allocated string to a Vim list (takes ownership)
    fn tv_list_append_allocated_string(list: *mut c_void, s: *mut c_char);

    // nvim_swapfile_info_and_print migrated to Rust (swapfile_info_and_print)

    // -------------------------------------------------------------------------
    // Memory allocation
    // -------------------------------------------------------------------------
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

// EW_* flags for expand_wildcards (from path.h)
const EW_FILE: c_int = 0x02;
const EW_KEEPALL: c_int = 0x10;
const EW_SILENT: c_int = 0x20;

/// Bitmask value indicating two files are equal (kEqualFiles from path.h)
const K_EQUAL_FILES: c_int = 1;

/// MAXPATHL constant
const MAXPATHL: usize = 4096;

// C FAIL constant
const FAIL: c_int = 0;

// =============================================================================
// Recovery Functions — foreign declarations for rs_ml_recover
// =============================================================================

extern "C" {
    /// Translate a message string (gettext wrapper)
    fn gettext(msgid: *const c_char) -> *const c_char;
    /// Display an error message
    fn emsg(s: *const c_char) -> bool;
    /// Start a multi-part message
    fn msg_start();
    /// End a multi-part message
    fn msg_end() -> bool;
    /// Output a string with highlight
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    /// Output a translated filename with highlight
    fn msg_outtrans(str_arg: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    /// no_wait_return global counter
    static mut no_wait_return: c_int;
}

extern "C" {
    fn nvim_set_recoverymode(val: c_int);
    fn nvim_get_called_from_main() -> c_int;
    fn nvim_get_curbuf_b_fname() -> *const c_char;
    fn nvim_get_curbuf_ffname() -> *const c_char;
    fn nvim_mf_open_rdonly(fname: *mut c_char) -> *mut c_void;
    fn nvim_mf_close_nodelete(mfp: *mut c_void);
    fn nvim_mf_get_block(mfp: *mut c_void, bnum: i64, page_count: c_uint) -> *mut c_void;
    #[link_name = "mf_put"]
    fn nvim_mf_put_block(mfp: *mut c_void, hp: *mut c_void, dirty: bool, infile: bool);
    fn nvim_mf_get_page_size(mfp: *mut c_void) -> c_uint;
    #[link_name = "mf_new_page_size"]
    fn nvim_mf_new_page_size_wrapper(mfp: *mut c_void, new_size: c_uint);
    fn nvim_mf_get_file_size(mfp: *mut c_void) -> i64;
    fn nvim_mf_set_blocknr_max(mfp: *mut c_void, val: i64);
    fn nvim_mf_set_infile_count(mfp: *mut c_void, val: i64);
    fn nvim_mf_get_fd(mfp: *mut c_void) -> c_int;
    fn nvim_mf_get_fname(mfp: *mut c_void) -> *mut c_char;
    fn nvim_bhdr_get_bh_data(hp: *mut c_void) -> *mut c_void;
    fn nvim_bhdr_set_bh_data(hp: *mut c_void, data: *mut c_void);
    fn nvim_b0_get_page_size_ptr(b0p: *const c_void) -> *const c_char;
    fn nvim_b0_set_fname0(b0p: *mut c_void);
    fn nvim_curbuf_set_b_flags_recovered();
    fn nvim_get_curbuf_ml_line_count() -> i64;
    fn nvim_get_curbuf_ml_flags() -> c_int;
    fn nvim_curbuf_get_b_changed() -> c_int;
    fn nvim_ml_delete_last_curbuf();
    fn nvim_ml_delete_first_curbuf();
    fn nvim_changed_internal_curbuf();
    fn nvim_unchanged_curbuf();
    fn nvim_ml_open_curbuf() -> c_int;
    fn nvim_ml_close_curbuf_true();
    fn nvim_setfname_for_recovery(name: *const c_char) -> c_int;
    fn nvim_buf_spname_curbuf() -> *const c_char;
    fn nvim_home_replace_curbuf_ffname_into_namebuff();
    fn nvim_xstrlcpy_namebuff(src: *const c_char);
    fn nvim_get_namebuff_ptr() -> *const c_char;
    fn nvim_smsg_using_swap_file();
    fn nvim_smsg_original_file();
    fn nvim_home_replace_into_namebuff(fname: *const c_char);
    fn nvim_expand_env_into_namebuff(src: *const c_char);
    fn nvim_recover_check_timestamps(mfp: *mut c_void, mtime_b0: c_int) -> c_int;
    fn nvim_get_buf_t_size() -> usize;
    fn rs_ml_add_stack(buf: *mut BufHandle) -> c_int;
    fn nvim_buf_reset_ml_stack(buf: *mut c_void);
    fn nvim_ip_set_bnum(ip: *mut InfoPtr, bnum: BlockNr);
    fn nvim_ip_set_index(ip: *mut InfoPtr, idx: c_int);
    fn nvim_ip_get_bnum(ip: *const InfoPtr) -> BlockNr;
    fn nvim_ip_get_index(ip: *const InfoPtr) -> c_int;
    fn nvim_getout_one();
    fn nvim_readfile_for_recovery(fname: *const c_char) -> c_int;
    fn nvim_readfile_from_original(
        fname: *const c_char,
        lnum: i64,
        topline: i64,
        line_count: i64,
    ) -> c_int;
    fn nvim_set_fileformat_local(ff: c_int);
    fn nvim_set_fenc_local(fenc: *const c_char);
    fn nvim_ml_append_recovery(lnum: i64, line: *const c_char, is_new: bool) -> c_int;
    fn nvim_ml_get(lnum: i64) -> *mut c_char;
    #[link_name = "ml_get_len"]
    fn nvim_ml_get_len(lnum: i64) -> i32;
    #[link_name = "xstrnsave"]
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    #[link_name = "buf_inc_changedtick"]
    fn nvim_buf_inc_changedtick(buf: *mut c_void);
    fn nvim_check_cursor();
    fn redraw_curbuf_later(redraw_type: c_int);
    fn line_breakcheck();
    fn nvim_prompt_for_recovery() -> c_int;
    // nvim_recover_check_proc_and_print migrated to Rust
    fn nvim_set_cmdline_row_to_msg_row();
    fn nvim_apply_autocmds_bufreadpost();
    fn nvim_apply_autocmds_bufwinenter();
}

// C OK/NOTDONE constants used in recovery
const OK_C: c_int = 1;
const ML_EMPTY_FLAG: c_int = 0x0001;

// recover_msg_id_T enum values
const RECOVER_MSG_E305_NO_SWAP: c_int = 0;
const RECOVER_MSG_E306_CANNOT_OPEN: c_int = 1;
const RECOVER_MSG_E307_NOT_SWAP: c_int = 2;
const RECOVER_MSG_E309_BLOCK1: c_int = 3;
const RECOVER_MSG_E310_BLOCK1_ID: c_int = 4;
const RECOVER_MSG_BLOCK0_UNREADABLE: c_int = 5;
const RECOVER_MSG_VIM3: c_int = 6;
const RECOVER_MSG_WRONG_BYTE_ORDER: c_int = 7;
const RECOVER_MSG_PAGE_SIZE_TOO_SMALL: c_int = 8;
const RECOVER_MSG_E308_ORIGINAL_CHANGED: c_int = 9;
const RECOVER_MSG_PTR_BLOCK_CORRUPTED: c_int = 10;
const RECOVER_MSG_E311_INTERRUPTED: c_int = 11;
const RECOVER_MSG_SUCCESS: c_int = 12;
const RECOVER_MSG_ERRORS: c_int = 13;

// =============================================================================
// Recovery message dispatch (replaces nvim_recover_msg C shim)
// =============================================================================

/// Format and display a recovery-related message.
///
/// This replaces the C `nvim_recover_msg` dispatch function.
/// # Safety
/// `fname` and `extra` must be valid C strings or NULL.
#[allow(clippy::cast_possible_wrap, clippy::too_many_lines)]
unsafe fn recover_msg(
    msg_id: c_int,
    fname: *const c_char,
    extra: *const c_char,
    hl_id: c_int,
    int_arg: c_int,
) {
    // Helper: translate a literal string via gettext
    macro_rules! t {
        ($s:expr) => {
            gettext($s.as_ptr())
        };
    }
    // Helper: format a C-string interpolation and call emsg
    macro_rules! semsg_fmt {
        ($fmt:literal, $arg:expr) => {{
            let s = $arg;
            let fmtted = if s.is_null() {
                CString::new(format!($fmt, "")).unwrap_or_default()
            } else {
                let cstr = std::ffi::CStr::from_ptr(s);
                CString::new(format!($fmt, cstr.to_string_lossy())).unwrap_or_default()
            };
            emsg(fmtted.as_ptr());
        }};
    }
    match msg_id {
        0 /* E305_NO_SWAP */ => {
            semsg_fmt!("E305: No swap file found for {}", fname);
        }
        1 /* E306_CANNOT_OPEN */ => {
            semsg_fmt!("E306: Cannot open {}", fname);
        }
        2 /* E307_NOT_SWAP */ => {
            semsg_fmt!("E307: {} does not look like a Nvim swap file", fname);
        }
        3 /* E309_BLOCK1 */ => {
            semsg_fmt!("E309: Unable to read block 1 from {}", fname);
        }
        4 /* E310_BLOCK1_ID */ => {
            semsg_fmt!("E310: Block 1 ID wrong ({} not a .swp file?)", fname);
        }
        5 /* BLOCK0_UNREADABLE */ => {
            msg_start();
            msg_puts_hl(t!(c"Unable to read block 0 from "), hl_id, true);
            msg_outtrans(fname, hl_id, true);
            msg_puts_hl(
                t!(c"\nMaybe no changes were made or Nvim did not update the swap file."),
                hl_id,
                true,
            );
            msg_end();
        }
        6 /* VIM3 */ => {
            msg_start();
            msg_outtrans(fname, 0, true);
            msg_puts_hl(t!(c" cannot be used with this version of Nvim.\n"), 0, true);
            msg_puts_hl(t!(c"Use Vim version 3.0.\n"), 0, true);
            msg_end();
        }
        7 /* WRONG_BYTE_ORDER */ => {
            msg_start();
            msg_outtrans(fname, hl_id, true);
            msg_puts_hl(t!(c" cannot be used on this computer.\n"), hl_id, true);
            msg_puts_hl(t!(c"The file was created on "), hl_id, true);
            msg_puts_hl(extra, hl_id, true);
            msg_puts_hl(t!(c",\nor the file has been damaged."), hl_id, true);
            msg_end();
        }
        8 /* PAGE_SIZE_TOO_SMALL */ => {
            msg_start();
            msg_outtrans(fname, hl_id, true);
            msg_puts_hl(
                t!(c" has been damaged (page size is smaller than minimum value).\n"),
                hl_id,
                true,
            );
            msg_end();
        }
        9 /* E308_ORIGINAL_CHANGED */ => {
            emsg(t!(c"E308: Warning: Original file may have been changed"));
        }
        10 /* PTR_BLOCK_CORRUPTED */ => {
            emsg(t!(c"E1364: Warning: Pointer block corrupted"));
        }
        11 /* E311_INTERRUPTED */ => {
            emsg(t!(c"E311: Recovery Interrupted"));
        }
        12 /* SUCCESS */ => {
            if int_arg != 0 {
                msg(t!(c"Recovery completed. You should check if everything is OK."), 0);
                msg_puts(t!(c"\n(You might want to write out this file under another name\n"));
                msg_puts(t!(c"and run diff with the original file to check for changes)"));
            } else {
                msg(t!(c"Recovery completed. Buffer contents equals file contents."), 0);
            }
            msg_puts(t!(c"\nYou may want to delete the .swp file now."));
        }
        13 /* ERRORS */ => {
            no_wait_return += 1;
            msg(c">>>>>>>>>>>>>".as_ptr(), 0);
            emsg(t!(
                c"E312: Errors detected while recovering; look for lines starting with ???"
            ));
            no_wait_return -= 1;
            msg(t!(c"See \":help E312\" for more information."), 0);
            msg(c">>>>>>>>>>>>>".as_ptr(), 0);
        }
        _ => {}
    }
}

// =============================================================================
// Recovery utility functions (migrated from C shim)
// =============================================================================

/// Check if the swap file owner process is still running and print a note.
///
/// Opens the swap file, reads block 0, and if the owner process is still
/// running, prints a message to the user.
///
/// # Safety
/// `fname_used` must be a valid C string.
unsafe fn recover_check_proc_and_print(fname_used: *const c_char) -> c_int {
    let fd = os_open(fname_used, 0, 0); // O_RDONLY = 0
    if fd < 0 {
        return 0;
    }
    let b0_size = nvim_b0_get_struct_size();
    let mut b0_buf = vec![0u8; b0_size];
    let n = read_eintr(fd, b0_buf.as_mut_ptr().cast(), b0_size);
    close(fd);
    #[allow(clippy::cast_sign_loss)]
    if n as usize != b0_size {
        return 0;
    }
    let b0 = b0_buf.as_ptr().cast::<c_void>();
    if rs_swapfile_proc_running(b0, fname_used) != 0 {
        msg_puts(gettext(c"\nNote: process STILL RUNNING: ".as_ptr()));
        let pid_ptr = nvim_b0_get_pid_ptr(b0);
        let pid_val = rs_char_to_long(pid_ptr);
        #[allow(clippy::cast_possible_truncation)]
        msg_outnum(pid_val as c_int);
        return 1;
    }
    0
}

// =============================================================================
// Recovery Functions
// =============================================================================

/// Recover the contents of a buffer from its swap file.
///
/// This is the Rust port of the C `ml_recover` function.
/// If `checkext` is true, verify the file extension is correct before recovery.
///
/// # Safety
/// Modifies global editor state; may read from corrupted swap files.
#[export_name = "ml_recover"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_ml_recover(checkext: c_int) {
    let mut buf: *mut c_void = std::ptr::null_mut();
    let mut mfp: *mut c_void = std::ptr::null_mut();
    let mut fname_used: *mut c_char = std::ptr::null_mut();
    let mut hp: *mut c_void = std::ptr::null_mut();
    let mut b0_fenc: *mut c_char = std::ptr::null_mut();
    let mut serious_error = true;
    let mut orig_file_status: c_int = -1; // NOTDONE

    nvim_set_recoverymode(1);
    let called_from_main = nvim_get_called_from_main();

    // Get curbuf's file name (empty string if NULL)
    let fname_raw = nvim_get_curbuf_b_fname();
    let fname: *const c_char = if fname_raw.is_null() {
        c"".as_ptr()
    } else {
        fname_raw
    };

    let directly: bool;

    'cleanup: {
        // Determine if fname itself is a swap file (extension ".s[a-w][a-z]")
        if checkext != 0 && is_swap_file_ext(fname) {
            directly = true;
            fname_used = xstrdup(fname);
        } else {
            directly = false;
            let len = rs_recover_names(fname, 0, std::ptr::null_mut(), 0, std::ptr::null_mut());
            if len == 0 {
                recover_msg(RECOVER_MSG_E305_NO_SWAP, fname, std::ptr::null(), 0, 0);
                break 'cleanup;
            }
            let chosen: c_int = if len == 1 {
                1
            } else {
                rs_recover_names(fname, 1, std::ptr::null_mut(), 0, std::ptr::null_mut());
                msg_putchar(c_int::from(b'\n'));
                let i = nvim_prompt_for_recovery();
                if i < 1 || i > len {
                    break 'cleanup;
                }
                i
            };
            rs_recover_names(fname, 0, std::ptr::null_mut(), chosen, &raw mut fname_used);
        }

        if fname_used.is_null() {
            break 'cleanup;
        }

        // When called from main(), initialize storage structure
        if called_from_main != 0 && nvim_ml_open_curbuf() != OK_C {
            nvim_getout_one();
        }

        // Allocate and init the recovery buf_T (zeroed)
        let buf_size = nvim_get_buf_t_size();
        buf = xmalloc(buf_size);
        std::ptr::write_bytes(buf.cast::<u8>(), 0, buf_size);
        block_ops::buf_init_ml_empty(buf.cast::<BufHandle>());

        // Open the swap file (mf_open() consumes fname_used, so save a copy)
        let fname_copy = xstrdup(fname_used);
        mfp = nvim_mf_open_rdonly(fname_used);
        fname_used = fname_copy;

        if mfp.is_null() || nvim_mf_get_fd(mfp) < 0 {
            recover_msg(
                RECOVER_MSG_E306_CANNOT_OPEN,
                fname_used,
                std::ptr::null(),
                0,
                0,
            );
            break 'cleanup;
        }
        (*buf.cast::<BufHandle>().cast::<BufStruct>()).ml_mfp = mfp;

        // Use minimum page size to be able to read block 0
        nvim_mf_new_page_size_wrapper(mfp, MIN_SWAP_PAGE_SIZE);
        let hl_id = HLF_E;

        // Read block 0
        hp = nvim_mf_get_block(mfp, 0, 1);
        if hp.is_null() {
            recover_msg(
                RECOVER_MSG_BLOCK0_UNREADABLE,
                nvim_mf_get_fname(mfp),
                std::ptr::null(),
                hl_id,
                0,
            );
            break 'cleanup;
        }

        let mut b0p = nvim_bhdr_get_bh_data(hp);

        // VIM 3.0 swap file?
        if b0_is_vim3_native(b0p) {
            recover_msg(
                RECOVER_MSG_VIM3,
                nvim_mf_get_fname(mfp),
                std::ptr::null(),
                0,
                0,
            );
            break 'cleanup;
        }
        // Bad block 0 ID?
        if rs_ml_check_b0_id(b0p) != 0 {
            recover_msg(
                RECOVER_MSG_E307_NOT_SWAP,
                nvim_mf_get_fname(mfp),
                std::ptr::null(),
                0,
                0,
            );
            break 'cleanup;
        }
        // Wrong byte order?
        if rs_b0_magic_wrong(b0p) != 0 {
            nvim_b0_set_fname0(b0p);
            let hname = nvim_b0_get_hname_ptr(b0p);
            recover_msg(
                RECOVER_MSG_WRONG_BYTE_ORDER,
                nvim_mf_get_fname(mfp),
                hname,
                hl_id,
                0,
            );
            break 'cleanup;
        }

        // If page size in swap file differs from what we assumed, recalculate
        let current_page_size = nvim_mf_get_page_size(mfp);
        let b0_page_size = rs_char_to_long(nvim_b0_get_page_size_ptr(b0p)) as c_uint;
        if current_page_size != b0_page_size {
            let previous_page_size = current_page_size;
            nvim_mf_new_page_size_wrapper(mfp, b0_page_size);
            let new_page_size = nvim_mf_get_page_size(mfp);
            if new_page_size < previous_page_size {
                recover_msg(
                    RECOVER_MSG_PAGE_SIZE_TOO_SMALL,
                    nvim_mf_get_fname(mfp),
                    std::ptr::null(),
                    hl_id,
                    0,
                );
                break 'cleanup;
            }
            let file_size = nvim_mf_get_file_size(mfp);
            let blocknr_max = if file_size <= 0 {
                0i64
            } else {
                file_size / i64::from(new_page_size)
            };
            nvim_mf_set_blocknr_max(mfp, blocknr_max);
            nvim_mf_set_infile_count(mfp, blocknr_max);

            // Reallocate block 0 data buffer to new page size
            let new_data = xmalloc(new_page_size as usize);
            let old_data = nvim_bhdr_get_bh_data(hp);
            std::ptr::copy_nonoverlapping(
                old_data.cast::<u8>(),
                new_data.cast::<u8>(),
                previous_page_size as usize,
            );
            xfree(old_data);
            nvim_bhdr_set_bh_data(hp, new_data);
            b0p = new_data; // b0p now points to new allocation
        }

        // If swap file was given directly, set buffer name from block 0 fname
        if directly {
            nvim_expand_env_into_namebuff(nvim_b0_get_fname_ptr(b0p));
            if nvim_setfname_for_recovery(nvim_get_namebuff_ptr()) != OK_C {
                break 'cleanup;
            }
        }

        // Extract fileformat and encoding from block 0 before releasing it
        let b0_ff = b0_get_ff_native(b0p);
        b0_fenc = b0_extract_fenc_native(b0p);
        let mtime_b0 = rs_char_to_long(nvim_b0_get_mtime(b0p).cast_const()) as c_int;

        // Display swap file and original file names
        nvim_home_replace_into_namebuff(nvim_mf_get_fname(mfp));
        nvim_smsg_using_swap_file();
        let spname = nvim_buf_spname_curbuf();
        if spname.is_null() {
            nvim_home_replace_curbuf_ffname_into_namebuff();
        } else {
            nvim_xstrlcpy_namebuff(spname);
        }
        nvim_smsg_original_file();
        msg_putchar(c_int::from(b'\n'));

        // Warn if original file was modified since swap file was written
        if nvim_recover_check_timestamps(mfp, mtime_b0) != 0 {
            recover_msg(
                RECOVER_MSG_E308_ORIGINAL_CHANGED,
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
            );
        }
        ui_flush();

        // Release block 0 — we have everything we need
        nvim_mf_put_block(mfp, hp, false, false);
        hp = std::ptr::null_mut();

        // Clear current buffer contents
        while nvim_get_curbuf_ml_flags() & ML_EMPTY_FLAG == 0 {
            nvim_ml_delete_first_curbuf();
        }

        // Read original file to get fileformat/encoding (errors ignored)
        let buf_ffname = nvim_get_curbuf_ffname();
        if !buf_ffname.is_null() {
            orig_file_status = nvim_readfile_for_recovery(buf_ffname);
        }

        // Apply fileformat and encoding from swap file
        if b0_ff != 0 {
            nvim_set_fileformat_local(b0_ff - 1);
        }
        if !b0_fenc.is_null() {
            nvim_set_fenc_local(b0_fenc);
            xfree(b0_fenc.cast());
            b0_fenc = std::ptr::null_mut();
        }
        nvim_unchanged_curbuf();

        // Walk the B-tree and append all recovered lines to curbuf
        let (lnum, error) = recover_btree(buf.cast::<BufHandle>(), mfp, &mut hp);

        // Negative error = fatal error during traversal
        if error < 0 {
            break 'cleanup;
        }

        // Determine if recovered content differs from original
        let curbuf_ptr = nvim_get_curbuf();
        let total_lines = nvim_get_curbuf_ml_line_count();
        if orig_file_status != OK_C || total_lines != lnum * 2 + 1 {
            // Empty file special case: 2 lines with first line empty → not modified
            let is_empty_recovery = total_lines == 2 && {
                let first = nvim_ml_get(1);
                !first.is_null() && *first == 0
            };
            if !is_empty_recovery {
                nvim_changed_internal_curbuf();
                nvim_buf_inc_changedtick(curbuf_ptr.cast());
            }
        } else {
            // Compare recovered lines vs original line by line
            let mut cidx = 1i64;
            while cidx <= lnum {
                let line_ptr = nvim_ml_get(cidx);
                let line_len = nvim_ml_get_len(cidx) as usize;
                let p = nvim_xstrnsave(line_ptr, line_len);
                let other = nvim_ml_get(cidx + lnum);
                let diff = libc_strcmp(p, other);
                xfree(p.cast());
                if diff != 0 {
                    nvim_changed_internal_curbuf();
                    nvim_buf_inc_changedtick(curbuf_ptr.cast());
                    break;
                }
                cidx += 1;
            }
        }

        // Delete the original lines (now after recovered lines) and dummy empty line
        while nvim_get_curbuf_ml_line_count() > lnum
            && nvim_get_curbuf_ml_flags() & ML_EMPTY_FLAG == 0
        {
            nvim_ml_delete_last_curbuf();
        }
        nvim_curbuf_set_b_flags_recovered();
        nvim_check_cursor();

        nvim_set_recoverymode(0);
        serious_error = false;

        // Final status messages
        if unsafe { got_int } {
            recover_msg(
                RECOVER_MSG_E311_INTERRUPTED,
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
            );
        } else if error > 0 {
            recover_msg(RECOVER_MSG_ERRORS, std::ptr::null(), std::ptr::null(), 0, 0);
        } else {
            recover_msg(
                RECOVER_MSG_SUCCESS,
                std::ptr::null(),
                std::ptr::null(),
                0,
                nvim_curbuf_get_b_changed(),
            );
            recover_check_proc_and_print(fname_used);
            msg_puts(c"\n\n".as_ptr());
            nvim_set_cmdline_row_to_msg_row();
        }
        redraw_curbuf_later(UPD_NOT_VALID);
    } // end 'cleanup

    // Always-run cleanup
    xfree(fname_used.cast());
    nvim_set_recoverymode(0);
    if !b0_fenc.is_null() {
        xfree(b0_fenc.cast());
    }
    if !mfp.is_null() {
        if !hp.is_null() {
            nvim_mf_put_block(mfp, hp, false, false);
        }
        nvim_mf_close_nodelete(mfp);
    }
    if !buf.is_null() {
        xfree((*buf.cast::<BufHandle>().cast::<BufStruct>()).ml_stack);
        xfree(buf);
    }
    if serious_error && called_from_main != 0 {
        nvim_ml_close_curbuf_true();
    } else {
        nvim_apply_autocmds_bufreadpost();
        nvim_apply_autocmds_bufwinenter();
    }
}

/// Check if `fname` has a swap file extension: `.s[a-w][a-z]`
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
unsafe fn is_swap_file_ext(fname: *const c_char) -> bool {
    let len = libc_strlen(fname);
    if len < 4 {
        return false;
    }
    if *fname.add(len - 4) != b'.' as c_char {
        return false;
    }
    let c2 = (*fname.add(len - 3) as u8).to_ascii_lowercase();
    if c2 != b's' {
        return false;
    }
    let c3 = (*fname.add(len - 2) as u8).to_ascii_lowercase();
    if !(b'a'..=b'w').contains(&c3) {
        return false;
    }
    (*fname.add(len - 1) as u8)
        .to_ascii_lowercase()
        .is_ascii_alphabetic()
}

// =============================================================================
// B-tree traversal for recovery
// =============================================================================

/// Walk the swap file B-tree and append recovered lines to curbuf.
///
/// Returns `(lnum, error_count)` where `error_count < 0` signals a fatal error.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::too_many_lines
)]
unsafe fn recover_btree(
    buf: *mut BufHandle,
    mfp: *mut c_void,
    hp_out: &mut *mut c_void,
) -> (i64, i64) {
    let mut bnum: i64 = 1;
    let mut page_count: c_uint = 1;
    let mut lnum: i64 = 0;
    let mut line_count: i64 = 0;
    let mut idx: c_int = 0;
    let mut error: i64 = 0;
    let mut hp: *mut c_void = std::ptr::null_mut();

    nvim_buf_reset_ml_stack(buf.cast::<c_void>());
    let mut cannot_open = nvim_get_curbuf_ffname().is_null();

    'traverse: loop {
        if unsafe { got_int } {
            break 'traverse;
        }
        line_breakcheck();

        if !hp.is_null() {
            nvim_mf_put_block(mfp, hp, false, false);
        }
        hp = nvim_mf_get_block(mfp, bnum, page_count);

        if hp.is_null() {
            if bnum == 1 {
                recover_msg(
                    RECOVER_MSG_E309_BLOCK1,
                    nvim_mf_get_fname(mfp),
                    std::ptr::null(),
                    0,
                    0,
                );
                *hp_out = std::ptr::null_mut();
                return (lnum, -1); // fatal
            }
            error += 1;
            nvim_ml_append_recovery(lnum, c"???MANY LINES MISSING".as_ptr(), true);
            lnum += 1;
        } else {
            let data = nvim_bhdr_get_bh_data(hp);

            if block_ops::pp_get_id(data) == PTR_ID {
                // ---- Pointer block ----
                let expected_max = block_ops::pp_count_max_for_mfp(mfp);
                let actual_max = block_ops::pp_get_count_max(data);
                let mut ptr_block_error = false;
                if actual_max != expected_max {
                    ptr_block_error = true;
                    block_ops::pp_set_count_max(data, expected_max);
                }
                let pb_count = block_ops::pp_get_count(data);
                if pb_count > expected_max {
                    ptr_block_error = true;
                    block_ops::pp_set_count(data, expected_max);
                }
                if ptr_block_error {
                    recover_msg(
                        RECOVER_MSG_PTR_BLOCK_CORRUPTED,
                        std::ptr::null(),
                        std::ptr::null(),
                        0,
                        0,
                    );
                }

                // Re-read pb_count after potential correction
                let pb_count_now = block_ops::pp_get_count(data);

                // Line count check on first entry into this pointer block
                if idx == 0 && line_count != 0 {
                    let mut lc = line_count;
                    for i in 0..c_int::from(pb_count_now) {
                        lc -= block_ops::pp_pe_get_line_count(data, i);
                    }
                    if lc != 0 {
                        error += 1;
                        nvim_ml_append_recovery(lnum, c"???LINE COUNT WRONG".as_ptr(), true);
                        lnum += 1;
                    }
                    line_count = 0;
                }

                if pb_count_now == 0 {
                    error += 1;
                    nvim_ml_append_recovery(lnum, c"???EMPTY BLOCK".as_ptr(), true);
                    lnum += 1;
                } else if idx < c_int::from(pb_count_now) {
                    let pe_bnum = block_ops::pp_pe_get_bnum(data, idx);
                    if pe_bnum < 0 {
                        // Negative block num: try reading from original file
                        if !cannot_open {
                            line_count = block_ops::pp_pe_get_line_count(data, idx);
                            let topline = block_ops::pp_pe_get_old_lnum(data, idx) - 1;
                            let ffname = nvim_get_curbuf_ffname();
                            if nvim_readfile_from_original(ffname, lnum, topline, line_count)
                                == OK_C
                            {
                                lnum += line_count;
                            } else {
                                cannot_open = true;
                            }
                        }
                        if cannot_open {
                            error += 1;
                            nvim_ml_append_recovery(lnum, c"???LINES MISSING".as_ptr(), true);
                            lnum += 1;
                        }
                        idx += 1;
                        continue 'traverse;
                    }

                    // Push current position and descend
                    let top = rs_ml_add_stack(buf);
                    let ip = (*buf.cast::<BufStruct>())
                        .ml_stack
                        .cast::<InfoPtr>()
                        .add((top) as usize);
                    nvim_ip_set_bnum(ip, bnum);
                    nvim_ip_set_index(ip, idx);

                    bnum = pe_bnum;
                    line_count = block_ops::pp_pe_get_line_count(data, idx);
                    page_count = block_ops::pp_pe_get_page_count_uint(data, idx);
                    idx = 0;
                    continue 'traverse;
                }
                // idx >= pb_count_now: fall through to stack pop
            } else {
                // ---- Data block ----
                if block_ops::dp_get_id(data) != DATA_ID {
                    if bnum == 1 {
                        recover_msg(
                            RECOVER_MSG_E310_BLOCK1_ID,
                            nvim_mf_get_fname(mfp),
                            std::ptr::null(),
                            0,
                            0,
                        );
                        *hp_out = hp;
                        return (lnum, -1); // fatal
                    }
                    error += 1;
                    nvim_ml_append_recovery(lnum, c"???BLOCK MISSING".as_ptr(), true);
                    lnum += 1;
                } else {
                    let mut has_error = false;
                    let page_size = nvim_mf_get_page_size(mfp);
                    let expected_txt_end = page_count * page_size;
                    let txt_end = block_ops::dp_get_txt_end(data);

                    if expected_txt_end != txt_end {
                        nvim_ml_append_recovery(
                            lnum,
                            c"??? from here until ???END lines may be messed up".as_ptr(),
                            true,
                        );
                        lnum += 1;
                        error += 1;
                        has_error = true;
                        block_ops::dp_set_txt_end(data, expected_txt_end);
                    }
                    block_ops::dp_write_nul_at_txt_end(data);

                    let dp_line_count = block_ops::dp_get_line_count(data);
                    if line_count != dp_line_count {
                        nvim_ml_append_recovery(
                            lnum,
                            c"??? from here until ???END lines may have been inserted/deleted"
                                .as_ptr(),
                            true,
                        );
                        lnum += 1;
                        error += 1;
                        has_error = true;
                    }

                    let mut did_questions = false;
                    let mut i: c_int = 0;
                    while i < dp_line_count as c_int {
                        if block_ops::dp_index_overruns_txt(data, i) != 0 {
                            error += 1;
                            nvim_ml_append_recovery(
                                lnum,
                                c"??? lines may be missing".as_ptr(),
                                true,
                            );
                            lnum += 1;
                            break;
                        }
                        let txt_start = block_ops::dp_get_index_masked(data, i);
                        let header_size = DATA_BLOCK_HEADER_SIZE as c_uint;
                        let dp_txt_end = block_ops::dp_get_txt_end(data);
                        let line_ptr: *const c_char =
                            if txt_start <= header_size || txt_start >= dp_txt_end {
                                error += 1;
                                if did_questions {
                                    i += 1;
                                    continue;
                                }
                                did_questions = true;
                                c"???".as_ptr()
                            } else {
                                did_questions = false;
                                block_ops::dp_get_txt_ptr(data, txt_start)
                            };
                        nvim_ml_append_recovery(lnum, line_ptr, true);
                        lnum += 1;
                        i += 1;
                    }
                    if has_error {
                        nvim_ml_append_recovery(lnum, c"???END".as_ptr(), true);
                        lnum += 1;
                    }
                }
            }
        } // end block processing

        // Check if traversal stack is empty (finished)
        if (*buf.cast::<BufStruct>()).ml_stack_top == 0 {
            break 'traverse;
        }

        // Pop one level and advance to next sibling
        let bs = buf.cast::<BufStruct>();
        (*bs).ml_stack_top -= 1;
        let new_top = (*bs).ml_stack_top;
        let ip = (*buf.cast::<BufStruct>())
            .ml_stack
            .cast::<InfoPtr>()
            .add((new_top) as usize);
        bnum = nvim_ip_get_bnum(ip);
        idx = nvim_ip_get_index(ip) + 1;
        page_count = 1;
    }

    *hp_out = hp;
    (lnum, error)
}

/// Get the number of swap files for a given file name.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
///
/// # Returns
/// Number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_count(fname: *const c_char) -> c_int {
    rs_recover_names(fname, 0, std::ptr::null_mut(), 0, std::ptr::null_mut())
}

/// List all swap files for a given file name.
///
/// Displays the list to the user via messaging.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
///
/// # Safety
/// - `fname` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_list(fname: *const c_char) {
    rs_recover_names(fname, 1, std::ptr::null_mut(), 0, std::ptr::null_mut());
}

/// Get the Nth swap file name for a given file.
///
/// # Arguments
/// * `fname` - File name to search for
/// * `nr` - Which swap file to return (1-based)
/// * `fname_out` - Output: the swap file name (caller must free)
///
/// # Returns
/// Number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `fname_out` must be a valid pointer
/// - The returned string must be freed by the caller
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_get(
    fname: *const c_char,
    nr: c_int,
    fname_out: *mut *mut c_char,
) -> c_int {
    if fname_out.is_null() {
        return 0;
    }
    rs_recover_names(fname, 0, std::ptr::null_mut(), nr, fname_out)
}

/// Get the list of swap files as a Vim list.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
/// * `ret_list` - Vim list_T to populate
///
/// # Returns
/// Number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `ret_list` must be a valid list_T pointer
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_to_list(
    fname: *const c_char,
    ret_list: *mut c_void,
) -> c_int {
    if ret_list.is_null() {
        return 0;
    }
    rs_recover_names(fname, 0, ret_list, 0, std::ptr::null_mut())
}

// =============================================================================
// Native Rust Implementation of recover_names
// =============================================================================

/// Generate swap file name patterns for wildcard matching.
///
/// This is the Rust port of the C `recov_file_names` static function.
/// Fills `names` with up to 2 patterns for swap file names based on `path`.
///
/// If `prepend_dot` is true, also adds `modname(path, ".sw?", true)`.
/// Always adds `concat_fnames(path, ".sw?", false)`.
///
/// Returns the number of patterns added.
///
/// # Safety
/// - `names` must point to an array of at least 2 `*mut c_char` pointers
/// - `path` must be a valid C string
#[allow(clippy::cast_sign_loss)]
unsafe fn recov_file_names(
    names: *mut *mut c_char,
    path: *const c_char,
    prepend_dot: bool,
) -> c_int {
    let mut num_names: c_int = 0;

    // May also add the file name with a dot prepended, for swapfile in same
    // dir as original file.
    if prepend_dot {
        let name = modname(path, c".sw?".as_ptr(), 1);
        if name.is_null() {
            return num_names;
        }
        *names.add(num_names as usize) = name;
        num_names += 1;
    }

    // Form the normal swapfile name pattern by appending ".sw?".
    let new_name = concat_fnames(path, c".sw?".as_ptr(), 0);
    if num_names >= 1 {
        // Check if we have the same name twice
        let prev = *names.add((num_names - 1) as usize);
        let prev_len = libc_strlen(prev);
        let new_len = libc_strlen(new_name);
        // file name has been expanded to full path if prev is longer
        let p = if prev_len > new_len {
            prev.add(prev_len - new_len)
        } else {
            prev
        };
        if libc_strcmp(p, new_name) != 0 {
            *names.add(num_names as usize) = new_name;
            num_names += 1;
        } else {
            xfree(new_name.cast());
        }
    } else {
        *names.add(num_names as usize) = new_name;
        num_names += 1;
    }

    num_names
}

/// Portable strlen for raw C strings
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Portable strcmp for raw C strings (returns 0 if equal)
#[allow(clippy::cast_sign_loss)]
unsafe fn libc_strcmp(a: *const c_char, b: *const c_char) -> c_int {
    let mut i = 0;
    loop {
        let ca = (*a.add(i)).unsigned_abs();
        let cb = (*b.add(i)).unsigned_abs();
        if ca != cb {
            return c_int::from(ca) - c_int::from(cb);
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

/// Scan swap file directories and list/count/retrieve swap files for recovery.
///
/// This is the Rust port of the C `recover_names` function.
///
/// Iterates over entries in the 'directory' option (`p_dir`), generating
/// wildcard patterns for each entry, expanding them, and either counting,
/// listing, or retrieving the swap files found.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
/// * `do_list` - If nonzero, display swap file info to the user
/// * `ret_list` - If non-NULL, populate this Vim list_T with swap file names
/// * `nr` - If >0, retrieve the Nth swap file name into `fname_out`
/// * `fname_out` - Output: the Nth swap file name (if `nr > 0`)
///
/// # Returns
/// Total number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `ret_list` must be a valid list_T* or NULL
/// - `fname_out` must be a valid `*mut *mut c_char` or NULL
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_recover_names(
    fname: *const c_char,
    do_list: c_int,
    ret_list: *mut c_void,
    nr: c_int,
    fname_out: *mut *mut c_char,
) -> c_int {
    // Resolve symlink in file name if provided (swap file uses actual path)
    let mut fname_buf = [0i8; MAXPATHL];
    let fname_res: *const c_char = if fname.is_null() {
        std::ptr::null()
    } else if crate::swap::rs_resolve_symlink(fname, fname_buf.as_mut_ptr()) == 1 {
        // rs_resolve_symlink returns OK (1) on success
        fname_buf.as_ptr()
    } else {
        fname
    };

    if do_list != 0 {
        // Use msg() to start the scrolling properly
        msg(c"Swap files found:".as_ptr(), 0);
        msg_putchar(c_int::from(b'\n'));
    }

    // Do the loop for every directory in 'directory'.
    // First allocate some memory to put the directory name in.
    let p_dir = nvim_get_p_dir();
    let dir_name_size = libc_strlen(p_dir) + 1;
    let dir_name = xmalloc(dir_name_size).cast::<c_char>();
    let mut dirp = p_dir;
    let mut file_count: c_int = 0;

    while *dirp != 0 {
        // Isolate a directory name from *dirp and put it in dir_name.
        // Advance dirp to next directory name.
        copy_option_part(&raw mut dirp, dir_name, 31000, c",".as_ptr());

        // names[] holds up to 6 patterns
        let mut names: [*mut c_char; 6] = [std::ptr::null_mut(); 6];
        let num_names: c_int;

        // Check if this is "." (current directory)
        if *dir_name == b'.' as c_char && *dir_name.add(1) == 0 {
            if fname.is_null() {
                names[0] = xstrdup(c"*.sw?".as_ptr());
                names[1] = xstrdup(c".*.sw?".as_ptr());
                names[2] = xstrdup(c".sw?".as_ptr());
                num_names = 3;
            } else {
                num_names = recov_file_names(names.as_mut_ptr(), fname_res, true);
            }
        } else {
            // Check directory dir_name
            if fname.is_null() {
                names[0] = concat_fnames(dir_name, c"*.sw?".as_ptr(), 1);
                names[1] = concat_fnames(dir_name, c".*.sw?".as_ptr(), 1);
                names[2] = concat_fnames(dir_name, c".sw?".as_ptr(), 1);
                num_names = 3;
            } else {
                let len = libc_strlen(dir_name);
                let p = dir_name.add(len);
                let tail = if after_pathsep(dir_name, p) != 0 && len > 1 && *p.sub(1) == *p.sub(2) {
                    // Ends with '//', use full path for swap name
                    crate::swap::rs_make_percent_swname(dir_name, p, fname_res)
                } else {
                    let t = path_tail(fname_res);
                    concat_fnames(dir_name, t, 1)
                };
                num_names = recov_file_names(names.as_mut_ptr(), tail, false);
                xfree(tail.cast());
            }
        }

        // Expand wildcards
        let mut num_files: c_int = 0;
        let mut files: *mut *mut c_char = std::ptr::null_mut();
        if num_names == 0
            || expand_wildcards(
                num_names,
                names.as_mut_ptr(),
                &raw mut num_files,
                &raw mut files,
                EW_KEEPALL | EW_FILE | EW_SILENT,
            ) == FAIL
        {
            num_files = 0;
        }

        // When no swapfile found, try simply adding ".swp" to the file name.
        if *dirp == 0 && file_count + num_files == 0 && !fname.is_null() {
            let swapname = modname(fname_res, c".swp".as_ptr(), 1);
            if !swapname.is_null() {
                if os_path_exists(swapname) != 0 {
                    files = xmalloc(std::mem::size_of::<*mut c_char>()).cast();
                    *files = swapname;
                    // swapname is now owned by files[0], don't free it separately
                    num_files = 1;
                } else {
                    xfree(swapname.cast());
                }
            }
        }

        // Remove swapfile name of the current buffer (must be ignored),
        // but keep it for swapfilelist().
        if ret_list.is_null() {
            let curbuf: *mut BufHandle = nvim_get_curbuf();
            let cur_mfp_fname = nvim_buf_get_ml_mfp_fname(curbuf);
            if !cur_mfp_fname.is_null() {
                let mut i = 0;
                while i < num_files {
                    if path_full_compare(cur_mfp_fname, *files.add(i as usize), 1, 0)
                        & K_EQUAL_FILES
                        != 0
                    {
                        // Remove this entry. Move further entries down.
                        xfree((*files.add(i as usize)).cast());
                        num_files -= 1;
                        if num_files == 0 {
                            xfree(files.cast());
                            files = std::ptr::null_mut();
                        } else {
                            let mut j = i;
                            while j < num_files {
                                *files.add(j as usize) = *files.add((j + 1) as usize);
                                j += 1;
                            }
                        }
                        // Don't advance i — the next entry is now at position i
                    } else {
                        i += 1;
                    }
                }
            }
        }

        if nr > 0 {
            file_count += num_files;
            if nr <= file_count {
                *fname_out = xstrdup(*files.add((nr - 1 + num_files - file_count) as usize));
                // Stop searching
                *dirp = 0;
            }
        } else if do_list != 0 {
            if *dir_name == b'.' as c_char && *dir_name.add(1) == 0 {
                if fname.is_null() {
                    msg_puts(c"   In current directory:\n".as_ptr());
                } else {
                    msg_puts(c"   Using specified name:\n".as_ptr());
                }
            } else {
                msg_puts(c"   In directory ".as_ptr());
                msg_home_replace(dir_name);
                msg_puts(c":\n".as_ptr());
            }

            if num_files > 0 {
                let mut i = 0;
                while i < num_files {
                    // print the swapfile name
                    file_count += 1;
                    msg_outnum(file_count);
                    msg_puts(c".    ".as_ptr());
                    msg_puts(path_tail(*files.add(i as usize)));
                    msg_putchar(c_int::from(b'\n'));
                    swapfile_info_and_print(*files.add(i as usize));
                    i += 1;
                }
            } else {
                msg_puts(c"      -- none --\n".as_ptr());
            }
            ui_flush();
        } else if !ret_list.is_null() {
            let mut i = 0;
            while i < num_files {
                let name = concat_fnames(dir_name, *files.add(i as usize), 1);
                tv_list_append_allocated_string(ret_list, name);
                i += 1;
            }
        } else {
            file_count += num_files;
        }

        // Free the pattern names
        let mut i = 0;
        while i < num_names {
            xfree(names[i as usize].cast());
            i += 1;
        }
        if num_files > 0 && !files.is_null() {
            FreeWild(num_files, files);
        }
    }

    xfree(dir_name.cast());
    file_count
}

// =============================================================================
// Swap File Information
// =============================================================================

/// Write swap file information to a dictionary.
///
/// Reads the block 0 header from the swap file and populates the
/// dictionary with version, user, host, filename, pid, mtime, dirty, inode.
///
/// # Arguments
/// * `fname` - Path to the swap file
/// * `d` - Dictionary to populate
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `d` must be a valid dict_T pointer or NULL
#[export_name = "swapfile_dict"]
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_swapfile_dict(fname: *const c_char, d: *mut c_void) {
    if fname.is_null() || d.is_null() {
        return;
    }

    // O_RDONLY = 0
    let fd = os_open(fname, 0, 0);
    if fd < 0 {
        tv_dict_add_str(d, c"error".as_ptr(), 5, c"Cannot open file".as_ptr());
        return;
    }

    let b0_size = nvim_b0_get_struct_size();
    let b0_ptr = xmalloc(b0_size);
    let bytes_read = read_eintr(fd, b0_ptr, b0_size);
    close(fd);

    #[allow(clippy::cast_sign_loss)]
    if bytes_read as usize != b0_size {
        xfree(b0_ptr);
        tv_dict_add_str(d, c"error".as_ptr(), 5, c"Cannot read file".as_ptr());
        return;
    }

    let b0 = b0_ptr.cast::<c_void>();

    if rs_ml_check_b0_id(b0) != 0 {
        tv_dict_add_str(d, c"error".as_ptr(), 5, c"Not a swap file".as_ptr());
    } else if rs_b0_magic_wrong(b0) != 0 {
        tv_dict_add_str(d, c"error".as_ptr(), 5, c"Magic number mismatch".as_ptr());
    } else {
        // version: 10 bytes
        let version_ptr = nvim_b0_get_version_ptr(b0);
        tv_dict_add_str_len(d, c"version".as_ptr(), 7, version_ptr, 10);

        // user: B0_UNAME_SIZE bytes
        let uname_ptr = nvim_b0_get_uname_ptr(b0);
        tv_dict_add_str_len(
            d,
            c"user".as_ptr(),
            4,
            uname_ptr,
            crate::types::B0_UNAME_SIZE as c_int,
        );

        // host: B0_HNAME_SIZE bytes
        let hname_ptr = nvim_b0_get_hname_ptr(b0);
        tv_dict_add_str_len(
            d,
            c"host".as_ptr(),
            4,
            hname_ptr,
            crate::types::B0_HNAME_SIZE as c_int,
        );

        // fname: B0_FNAME_SIZE_ORG bytes
        let fname_ptr = nvim_b0_get_fname_ptr(b0);
        tv_dict_add_str_len(
            d,
            c"fname".as_ptr(),
            5,
            fname_ptr,
            crate::types::B0_FNAME_SIZE_ORG as c_int,
        );

        // pid: process ID derived from b0_pid
        let pid = rs_swapfile_proc_running(b0, fname);
        tv_dict_add_nr(d, c"pid".as_ptr(), 3, i64::from(pid));

        // mtime: file modification time from b0_mtime
        let mtime_ptr = nvim_b0_get_mtime(b0_ptr);
        let mtime = rs_char_to_long(mtime_ptr);
        tv_dict_add_nr(d, c"mtime".as_ptr(), 5, mtime);

        // dirty: whether the file was unsaved
        let dirty = nvim_b0_get_dirty(b0);
        tv_dict_add_nr(d, c"dirty".as_ptr(), 5, i64::from(i32::from(dirty != 0)));

        // inode: stored inode number from b0_ino
        let ino_ptr = nvim_b0_get_ino(b0_ptr);
        let inode = rs_char_to_long(ino_ptr);
        tv_dict_add_nr(d, c"inode".as_ptr(), 5, inode);
    }

    xfree(b0_ptr);
}

/// Load info from swap file `fname` and append it to the StringBuilder `sb`.
///
/// Returns the swap file's mtime (0 if unknown), and sets `*proc_running_out`
/// to the PID of the swap file owner if that process is still running (else 0).
///
/// This is the Rust port of the C `swapfile_info` static function.
///
/// # Safety
/// - `fname` must be a valid C string
/// - `sb` must be a valid `StringBuilder *` (opaque from C)
/// - `proc_running_out` must be a valid pointer
/// # Panics
/// Panics if `fname` is null.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_swapfile_info(
    fname: *mut c_char,
    sb: *mut c_void,
    proc_running_out: *mut c_int,
) -> i64 {
    assert!(!fname.is_null());
    *proc_running_out = 0;

    let mut mtime: i64 = 0;

    // Get file date and owner (UNIX: get owner name)
    let (file_mtime, uname_opt) = swapfile_get_file_info(fname);
    if file_mtime != 0 {
        mtime = file_mtime;
        if let Some(ref uname_buf) = uname_opt {
            sb_append_owned_by(sb, uname_buf.as_ptr());
        }
        sb_append_dated(sb, uname_opt.is_some());
        sb_append_ctime(sb, mtime);
    }

    // Open swap file and read block 0
    // O_RDONLY = 0
    let fd = os_open(fname, 0, 0);
    if fd < 0 {
        nvim_sb_append_str(sb, gettext(c"         [cannot be opened]".as_ptr()));
        nvim_sb_append_str(sb, c"\n".as_ptr());
        return mtime;
    }

    let b0_size = nvim_b0_get_struct_size();
    let b0_ptr = xmalloc(b0_size);
    let bytes_read = read_eintr(fd, b0_ptr, b0_size);
    close(fd);

    if bytes_read as usize != b0_size {
        // cast_sign_loss: allowed because bytes_read < 0 means error (mismatch with b0_size)
        xfree(b0_ptr);
        nvim_sb_append_str(sb, gettext(c"         [cannot be read]".as_ptr()));
        nvim_sb_append_str(sb, c"\n".as_ptr());
        return mtime;
    }

    let b0 = b0_ptr.cast::<c_void>();

    // Check if this is an old Vim 3.0 swap file
    let version_ptr = nvim_b0_get_version_ptr(b0);
    // Check "VIM 3.0" (7 bytes)
    let vim3 = b"VIM 3.0";
    #[allow(clippy::cast_sign_loss)]
    let is_vim3 = (0..7).all(|i| *version_ptr.add(i) as u8 == vim3[i]);

    if is_vim3 {
        nvim_sb_append_str(sb, gettext(c"         [from Vim version 3.0]".as_ptr()));
    } else if rs_ml_check_b0_id(b0) != 0 {
        nvim_sb_append_str(
            sb,
            gettext(c"         [does not look like a Nvim swap file]".as_ptr()),
        );
    } else if rs_ml_check_b0_strings(b0) != 0 {
        nvim_sb_append_str(
            sb,
            gettext(c"         [garbled strings (not nul terminated)]".as_ptr()),
        );
    } else {
        // Print filename
        let fname_ptr = nvim_b0_get_fname_ptr(b0);
        nvim_sb_append_str(sb, gettext(c"         file name: ".as_ptr()));
        if *fname_ptr == 0 {
            nvim_sb_append_str(sb, gettext(c"[No Name]".as_ptr()));
        } else {
            nvim_sb_append_str(sb, fname_ptr);
        }

        // Print modified status
        let dirty = nvim_b0_get_dirty(b0);
        nvim_sb_append_str(sb, gettext(c"\n          modified: ".as_ptr()));
        nvim_sb_append_str(
            sb,
            gettext(if dirty != 0 {
                c"YES".as_ptr()
            } else {
                c"no".as_ptr()
            }),
        );

        // Print user name if present
        let uname_ptr = nvim_b0_get_uname_ptr(b0);
        let uname_present = *uname_ptr != 0;
        if uname_present {
            nvim_sb_append_str(sb, gettext(c"\n         user name: ".as_ptr()));
            nvim_sb_append_str(sb, uname_ptr);
        }

        // Print host name if present
        let hname_ptr = nvim_b0_get_hname_ptr(b0);
        let hname_present = *hname_ptr != 0;
        if hname_present {
            if uname_present {
                nvim_sb_append_str(sb, gettext(c"   host name: ".as_ptr()));
            } else {
                nvim_sb_append_str(sb, gettext(c"\n         host name: ".as_ptr()));
            }
            nvim_sb_append_str(sb, hname_ptr);
        }

        // Print process ID if present
        let pid_ptr = nvim_b0_get_pid_ptr(b0);
        let pid_val = rs_char_to_long(pid_ptr);
        if pid_val != 0 {
            nvim_sb_append_str(sb, gettext(c"\n        process ID: ".as_ptr()));
            // Format the PID as decimal
            let pid_cstr = CString::new(format!("{pid_val}")).unwrap_or_default();
            nvim_sb_append_str(sb, pid_cstr.as_ptr());
            let running = rs_swapfile_proc_running(b0, fname);
            *proc_running_out = running;
            if running != 0 {
                nvim_sb_append_str(sb, gettext(c" (STILL RUNNING)".as_ptr()));
            }
        }

        // Print if not usable on this computer (wrong byte order)
        if rs_b0_magic_wrong(b0) != 0 {
            nvim_sb_append_str(
                sb,
                gettext(c"\n         [not usable on this computer]".as_ptr()),
            );
        }
    }

    xfree(b0_ptr);
    nvim_sb_append_str(sb, c"\n".as_ptr());
    mtime
}

// =============================================================================
// Block 0 Validation
// =============================================================================

extern "C" {
    /// Get b0_magic_long field from ZeroBlock
    fn nvim_b0_get_magic_long(b0: *const c_void) -> i64;

    /// Get b0_magic_int field from ZeroBlock
    fn nvim_b0_get_magic_int(b0: *const c_void) -> i32;

    /// Get b0_magic_short field from ZeroBlock
    fn nvim_b0_get_magic_short(b0: *const c_void) -> i16;

    /// Get b0_magic_char field from ZeroBlock
    fn nvim_b0_get_magic_char(b0: *const c_void) -> u8;

    /// Get b0_id[idx] byte from ZeroBlock
    fn nvim_b0_get_id(b0: *const c_void, idx: c_int) -> u8;

    /// Get pointer to b0_version field (10 bytes)
    fn nvim_b0_get_version_ptr(b0: *const c_void) -> *const c_char;

    /// Get size of b0_version field

    /// Get pointer to b0_uname field (B0_UNAME_SIZE bytes)
    fn nvim_b0_get_uname_ptr(b0: *const c_void) -> *const c_char;

    /// Get pointer to b0_hname field (B0_HNAME_SIZE bytes)
    fn nvim_b0_get_hname_ptr(b0: *const c_void) -> *const c_char;

    /// Get pointer to b0_fname field (B0_FNAME_SIZE_ORG bytes)
    fn nvim_b0_get_fname_ptr(b0: *const c_void) -> *const c_char;

    /// Get b0_flags byte from ZeroBlock
    fn nvim_b0_get_flags_byte(b0: *const c_void) -> u8;

    /// Get file inode number, or 0 if file doesn't exist
    fn nvim_get_file_inode(fname: *const c_char) -> u64;

    /// Resolve full path (vim_FullName): returns OK(1) on success
    fn vim_FullName(fname: *const c_char, buf: *mut c_char, len: c_int, force: c_int) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 2: Swap file utility accessors
    // -------------------------------------------------------------------------

    /// Get mtime of a file (returns 0 if not found)
    fn nvim_get_file_mtime(fname: *const c_char) -> i64;

    /// Get the current uptime in seconds
    fn uv_uptime(uptime: *mut f64) -> c_int;

    /// Get current time in seconds
    fn os_time() -> i64;

    /// Check if process with pid is running; returns nonzero if yes
    fn os_proc_running(pid: c_int) -> c_int;

    /// Open file (returns fd or negative on error)
    fn os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int;

    /// Read from fd with EINTR retry (returns bytes read)
    fn read_eintr(fd: c_int, buf: *mut c_void, count: usize) -> isize;

    /// Close file descriptor
    fn close(fd: c_int) -> c_int;

    /// Get the size of a ZeroBlock struct
    fn nvim_b0_get_struct_size() -> usize;

    /// Get b0_dirty byte from ZeroBlock
    fn nvim_b0_get_dirty(b0: *const c_void) -> u8;

    /// Force NUL at end of b0_hname (for corruption safety)
    fn nvim_b0_set_hname_end(b0: *mut c_void);

    /// Get current hostname
    fn os_get_hostname(hostname: *mut c_char, size: usize);

    /// Case-insensitive string comparison
    fn strcasecmp(a: *const c_char, b: *const c_char) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 1 (pass 3): Vimscript dict helpers for swapfile_dict
    // -------------------------------------------------------------------------

    /// Add a string entry to a Vim dictionary (copies val)
    fn tv_dict_add_str(
        d: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
    ) -> c_int;

    /// Add a string entry with explicit length to a Vim dictionary
    fn tv_dict_add_str_len(
        d: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
        len: c_int,
    ) -> c_int;

    /// Add an integer (number) entry to a Vim dictionary
    fn tv_dict_add_nr(d: *mut c_void, key: *const c_char, key_len: usize, nr: i64) -> c_int;

    /// Get pointer to b0_mtime field (4 bytes, char_to_long encoding)
    fn nvim_b0_get_mtime(b0: *mut c_void) -> *mut c_char;

    /// Get pointer to b0_ino field (4 bytes, char_to_long encoding)
    fn nvim_b0_get_ino(b0: *mut c_void) -> *mut c_char;

    // -------------------------------------------------------------------------
    fn nvim_sb_append_str(sb: *mut c_void, s: *const c_char);

    /// Format a time_t as ctime string (with trailing newline)
    fn os_ctime_r(
        clock: *const i64,
        result: *mut c_char,
        result_len: usize,
        add_newline: bool,
    ) -> *mut c_char;

    /// Print a multi-line message to the message area
    fn nvim_msg_multiline(s: *const c_char, hl_id: c_int);
}

#[cfg(unix)]
extern "C" {
    /// Get username for file owner (UNIX only). Returns 1 on success.
    fn nvim_swapfile_get_uname(
        fname: *const c_char,
        uname_buf: *mut c_char,
        uname_len: usize,
    ) -> c_int;
}

// =============================================================================
// Swapfile info StringBuilder helpers (replace C dispatch functions)
// =============================================================================

/// Append "owned by: <uname>" to a StringBuilder.
unsafe fn sb_append_owned_by(sb: *mut c_void, uname: *const c_char) {
    nvim_sb_append_str(sb, gettext(c"          owned by: ".as_ptr()));
    nvim_sb_append_str(sb, uname);
}

/// Append "dated: " or "             dated: " to a StringBuilder.
unsafe fn sb_append_dated(sb: *mut c_void, after_uname: bool) {
    if after_uname {
        nvim_sb_append_str(sb, gettext(c"   dated: ".as_ptr()));
    } else {
        nvim_sb_append_str(sb, gettext(c"             dated: ".as_ptr()));
    }
}

/// Append ctime string for mtime to a StringBuilder.
unsafe fn sb_append_ctime(sb: *mut c_void, mtime: i64) {
    let mut ctime_buf = [0i8; 100];
    let s = os_ctime_r(
        &raw const mtime,
        ctime_buf.as_mut_ptr(),
        ctime_buf.len(),
        true,
    );
    nvim_sb_append_str(sb, s);
}

/// Get file mtime and owner (UNIX only) for swapfile_info display.
///
/// Returns `(mtime, uname)` where uname is Some if the owner name was found.
/// # Safety
/// `fname` must be a valid C string.
unsafe fn swapfile_get_file_info(fname: *const c_char) -> (i64, Option<[i8; 40]>) {
    let mtime = nvim_get_file_mtime(fname);
    if mtime == 0 {
        return (0, None);
    }
    #[cfg(unix)]
    {
        let mut uname_buf = [0i8; 40];
        if nvim_swapfile_get_uname(fname, uname_buf.as_mut_ptr(), uname_buf.len()) != 0 {
            return (mtime, Some(uname_buf));
        }
    }
    (mtime, None)
}

/// Print swapfile info to the message area.
///
/// Allocates a StringBuilder, calls rs_swapfile_info, then prints via msg_multiline.
/// # Safety
/// `fname` must be a valid C string.
unsafe fn swapfile_info_and_print(fname: *mut c_char) {
    extern "C" {
        fn nvim_alloc_stringbuilder_iosize() -> *mut c_void;
        fn nvim_sb_get_items(sb: *mut c_void) -> *const c_char;
        fn nvim_free_stringbuilder(sb: *mut c_void);
    }
    let sb = nvim_alloc_stringbuilder_iosize();
    let mut proc_running_unused: c_int = 0;
    rs_swapfile_info(fname, sb, &raw mut proc_running_unused);
    let items = nvim_sb_get_items(sb);
    nvim_msg_multiline(items, 0);
    nvim_free_stringbuilder(sb);
}

use crate::types::{
    B0_FNAME_SIZE_CRYPT, B0_HNAME_SIZE, B0_MAGIC_CHAR, B0_MAGIC_INT, B0_MAGIC_LONG, B0_MAGIC_SHORT,
    B0_UNAME_SIZE, BLOCK0_ID0, BLOCK0_ID1,
};

/// Native 4-byte check: is b0_magic_wrong?
///
/// Returns non-zero if any magic number doesn't match the expected value,
/// indicating the swap file was created on a system with a different byte order.
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
unsafe fn b0_magic_wrong_native(b0: *const c_void) -> bool {
    nvim_b0_get_magic_long(b0) != B0_MAGIC_LONG
        || nvim_b0_get_magic_int(b0) != B0_MAGIC_INT
        || nvim_b0_get_magic_short(b0) != B0_MAGIC_SHORT
        || nvim_b0_get_magic_char(b0) != B0_MAGIC_CHAR
}

/// Native check: does b0 have valid ID bytes?
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
unsafe fn ml_check_b0_id_native(b0: *const c_void) -> bool {
    nvim_b0_get_id(b0, 0) == BLOCK0_ID0 && nvim_b0_get_id(b0, 1) == BLOCK0_ID1
}

/// Native check: are all NUL-terminated strings in b0 valid?
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
unsafe fn ml_check_b0_strings_native(b0: *const c_void) -> bool {
    let version = nvim_b0_get_version_ptr(b0);
    let uname = nvim_b0_get_uname_ptr(b0);
    let hname = nvim_b0_get_hname_ptr(b0);
    let fname = nvim_b0_get_fname_ptr(b0);

    has_nul(version, B0_VERSION_SIZE)
        && has_nul(uname, B0_UNAME_SIZE)
        && has_nul(hname, B0_HNAME_SIZE)
        && has_nul(fname, B0_FNAME_SIZE_CRYPT)
}

/// Check whether a byte sequence of `len` bytes contains a NUL byte.
unsafe fn has_nul(ptr: *const c_char, len: usize) -> bool {
    for i in 0..len {
        if *ptr.add(i) == 0 {
            return true;
        }
    }
    false
}

/// Check if a block 0 has valid identification bytes.
///
/// Block 0 must start with BLOCK0_ID0 and BLOCK0_ID1.
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_ml_check_b0_id(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 1; // FAIL
    }
    c_int::from(!ml_check_b0_id_native(b0))
}

/// Check if block 0 strings are valid (NUL-terminated).
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_ml_check_b0_strings(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 1; // FAIL
    }
    c_int::from(!ml_check_b0_strings_native(b0))
}

/// Check if block 0 has wrong byte order (magic number check).
///
/// Returns non-zero if the magic numbers don't match expected values,
/// indicating the swap file was created on a system with different
/// byte order.
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_b0_magic_wrong(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 1; // true, it's wrong
    }
    c_int::from(b0_magic_wrong_native(b0))
}

// =============================================================================
// Byte Order Utilities
// =============================================================================

/// Convert a long integer to a 4-byte array for swap file storage.
///
/// Writes the 4 low bytes of `n` in little-endian order, matching the
/// C `long_to_char()` behavior (which only encodes 4 bytes).
///
/// # Arguments
/// * `n` - The number to convert (only low 32 bits used)
/// * `s` - Output buffer (must be at least 4 bytes)
///
/// # Safety
/// - `s` must be a valid buffer of at least 4 bytes
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_long_to_char(n: i64, s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // Match C behavior: write 4 bytes, low byte first (little-endian)
    *s.add(0) = (n & 0xFF) as c_char;
    *s.add(1) = ((n >> 8) & 0xFF) as c_char;
    *s.add(2) = ((n >> 16) & 0xFF) as c_char;
    *s.add(3) = ((n >> 24) & 0xFF) as c_char;
}

// =============================================================================
// Phase 5 migrated helpers (formerly C functions in memline_shim.c)
// =============================================================================

/// Get the fileformat flags from block 0 (b0_flags & B0_FF_MASK).
///
/// # Safety
/// - `b0p` must be a valid pointer to a ZeroBlock
unsafe fn b0_get_ff_native(b0p: *const c_void) -> c_int {
    c_int::from(nvim_b0_get_flags_byte(b0p) & B0_FF_MASK)
}

/// Check if block 0 is from VIM 3.0 (version string starts with "VIM 3.0").
///
/// # Safety
/// - `b0p` must be a valid pointer to a ZeroBlock
unsafe fn b0_is_vim3_native(b0p: *const c_void) -> bool {
    let version = nvim_b0_get_version_ptr(b0p).cast::<u8>();
    let vim3 = b"VIM 3.0";
    for (i, &byte) in vim3.iter().enumerate() {
        if *version.add(i) != byte {
            return false;
        }
    }
    true
}

/// Extract the fileencoding from block 0 b0_fname area (B0_HAS_FENC flag).
///
/// Returns a newly-allocated C string with the encoding, or NULL if not present.
///
/// # Safety
/// - `b0p` must be a valid pointer to a ZeroBlock
unsafe fn b0_extract_fenc_native(b0p: *const c_void) -> *mut c_char {
    if nvim_b0_get_flags_byte(b0p) & B0_HAS_FENC == 0 {
        return std::ptr::null_mut();
    }
    let fname = nvim_b0_get_fname_ptr(b0p);
    let fnsize = B0_FNAME_SIZE_NOCRYPT;
    // Search backwards from end of fname area for the start of fenc string
    let mut p = fname.add(fnsize);
    while p > fname && *p.sub(1) != 0 {
        p = p.sub(1);
    }
    let len = fname.add(fnsize) as usize - p as usize;
    nvim_xstrnsave(p, len)
}

/// Convert a 4-byte array from swap file storage to a long integer.
///
/// Reverses the `rs_long_to_char()` operation, reading 4 bytes in
/// little-endian order, matching C `char_to_long()`.
///
/// # Arguments
/// * `s` - Input buffer (must be at least 4 bytes)
///
/// # Returns
/// The decoded integer value
///
/// # Safety
/// - `s` must be a valid buffer of at least 4 bytes
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_char_to_long(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }
    // Match C behavior: read 4 bytes, little-endian
    let b0 = *s.add(0) as u8;
    let b1 = *s.add(1) as u8;
    let b2 = *s.add(2) as u8;
    let b3 = *s.add(3) as u8;
    i64::from(b3) << 24 | i64::from(b2) << 16 | i64::from(b1) << 8 | i64::from(b0)
}

// =============================================================================
// File Name Comparison
// =============================================================================

/// Compare two file names, considering inode number.
///
/// Used during recovery to match the current file with the file
/// recorded in the swap file's block 0.  Returns non-zero if the files
/// are different (i.e. they do NOT match).
///
/// # Arguments
/// * `fname_c` - Current file name
/// * `fname_s` - File name from swap file
/// * `ino_block0` - Inode stored in block 0
///
/// # Safety
/// - Both file name pointers must be valid C strings or NULL
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_fnamecmp_ino(
    fname_c: *const c_char,
    fname_s: *const c_char,
    ino_block0: i64,
) -> c_int {
    if fname_c.is_null() || fname_s.is_null() {
        return 1; // differ = true when pointers invalid
    }

    // Get inodes for both files
    let ino_c = nvim_get_file_inode(fname_c);
    let ino_s_live = nvim_get_file_inode(fname_s);
    // Use live inode for fname_s if available, fall back to block 0 value
    #[allow(clippy::cast_sign_loss)]
    let ino_s: u64 = if ino_s_live != 0 {
        ino_s_live
    } else {
        ino_block0 as u64
    };

    // Both inodes known: compare directly
    if ino_c != 0 && ino_s != 0 {
        return c_int::from(ino_c != ino_s);
    }

    // Fall back to full path comparison
    let mut buf_c = [0i8; MAXPATHL];
    let mut buf_s = [0i8; MAXPATHL];
    #[allow(clippy::cast_possible_truncation)]
    let ok_c = vim_FullName(fname_c, buf_c.as_mut_ptr(), MAXPATHL as c_int, 1);
    #[allow(clippy::cast_possible_truncation)]
    let ok_s = vim_FullName(fname_s, buf_s.as_mut_ptr(), MAXPATHL as c_int, 1);

    // OK == 1 in Vim (not 0 which would be FAIL)
    if ok_c == 1 && ok_s == 1 {
        // Compare the resolved paths
        return c_int::from(libc_strcmp(buf_c.as_ptr(), buf_s.as_ptr()) != 0);
    }

    // Both unknown inodes, both full name resolutions failed: compare raw names
    if ino_s == 0 && ino_c == 0 && ok_c != 1 && ok_s != 1 {
        return c_int::from(libc_strcmp(fname_c, fname_s) != 0);
    }

    // Can't determine equivalence; assume different
    1
}

// =============================================================================
// Phase 2: Swap File Utility Functions
// =============================================================================

/// Check if the process that owns a swap file is still running.
///
/// Returns the PID if the process is running, 0 otherwise.
/// Also returns 0 if the swap file predates the last system reboot.
///
/// # Safety
/// - `b0p` must be a valid ZeroBlock pointer
/// - `swap_fname` must be a valid C string
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_swapfile_proc_running(
    b0p: *const c_void,
    swap_fname: *const c_char,
) -> c_int {
    // If the system rebooted after the swapfile was written, the process
    // can't be running now.
    let file_mtime = nvim_get_file_mtime(swap_fname);
    if file_mtime > 0 {
        let mut uptime: f64 = 0.0;
        if uv_uptime(&raw mut uptime) == 0 {
            let now = os_time();
            if file_mtime < now - uptime as i64 {
                return 0;
            }
        }
    }

    // Get PID from block 0 and check if it's still running
    let b0_pid_ptr = nvim_b0_get_pid_ptr(b0p);
    let pid = rs_char_to_long(b0_pid_ptr) as c_int;
    if os_proc_running(pid) != 0 {
        pid
    } else {
        0
    }
}

extern "C" {
    /// Get pointer to b0_pid field (4 bytes, char_to_long encoding)
    fn nvim_b0_get_pid_ptr(b0: *const c_void) -> *const c_char;
}

/// Inner check logic for `rs_swapfile_unchanged`. Called after b0 is read.
/// Returns true if block 0 is valid, not dirty, hostname matches, and process is not running.
unsafe fn check_b0_unchanged(b0_ptr: *const c_void, fname: *mut c_char) -> bool {
    // ID and magic must be correct
    if !ml_check_b0_id_native(b0_ptr) || b0_magic_wrong_native(b0_ptr) {
        return false;
    }

    // Must be unchanged (b0_dirty == 0)
    if nvim_b0_get_dirty(b0_ptr) != 0 {
        return false;
    }

    // Host name must be known and match current host
    let hname_ptr = nvim_b0_get_hname_ptr(b0_ptr);
    if *hname_ptr == 0 {
        return false;
    }
    let mut cur_hostname = [0i8; 40]; // B0_HNAME_SIZE
    os_get_hostname(cur_hostname.as_mut_ptr(), 40);
    cur_hostname[39] = 0; // ensure NUL-terminated
                          // Force NUL at end of stored name (in case of corruption)
    nvim_b0_set_hname_end(b0_ptr.cast_mut());
    if strcasecmp(hname_ptr, cur_hostname.as_ptr()) != 0 {
        return false;
    }

    // Process must be known and not running
    if rs_char_to_long(nvim_b0_get_pid_ptr(b0_ptr)) == 0
        || rs_swapfile_proc_running(b0_ptr, fname) != 0
    {
        return false;
    }

    true
}

/// Check if a swap file has no changes (is unchanged since last save).
///
/// Returns true if the swap file exists, has valid block 0, is not dirty,
/// the hostname matches, and the owning process is not running.
///
/// Only called from `findswapname()`.
///
/// # Safety
/// - `fname` must be a valid C string
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_swapfile_unchanged(fname: *mut c_char) -> bool {
    // Swap file must exist
    if os_path_exists(fname) == 0 {
        return false;
    }

    // Must be able to read the first block
    // O_RDONLY = 0
    let fd = os_open(fname, 0, 0);
    if fd < 0 {
        return false;
    }

    // Allocate a zero block on the stack-equivalent heap allocation
    let b0_size = nvim_b0_get_struct_size();
    let b0_ptr = xmalloc(b0_size).cast::<c_void>();
    let bytes_read = read_eintr(fd, b0_ptr, b0_size);
    close(fd);

    if bytes_read as usize != b0_size {
        xfree(b0_ptr);
        return false;
    }

    // All checks below; cleanup b0_ptr on exit via a helper result
    let ok = check_b0_unchanged(b0_ptr, fname);
    xfree(b0_ptr);
    ok
}

// =============================================================================
// Recovery State Helpers
// =============================================================================

use crate::types::{
    SEA_CHOICE_ABORT, SEA_CHOICE_DELETE, SEA_CHOICE_EDIT, SEA_CHOICE_NONE, SEA_CHOICE_QUIT,
    SEA_CHOICE_READONLY, SEA_CHOICE_RECOVER,
};

/// Check if a swap file can be recovered.
///
/// A swap file can be recovered if:
/// - It has valid block 0 identification
/// - The strings in block 0 are valid (NUL-terminated)
/// - The byte order magic numbers are correct
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_swap_file_recoverable(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 0;
    }

    // Check ID bytes
    if !ml_check_b0_id_native(b0) {
        return 0;
    }

    // Check strings are valid
    if !ml_check_b0_strings_native(b0) {
        return 0;
    }

    // Check byte order
    if b0_magic_wrong_native(b0) {
        return 0;
    }

    1
}

/// Get the swap file attention choice from a user response.
///
/// Maps user input characters to SEA_CHOICE constants:
/// - 'O' or 'o' -> SEA_CHOICE_READONLY (1)
/// - 'E' or 'e' -> SEA_CHOICE_EDIT (2)
/// - 'R' or 'r' -> SEA_CHOICE_RECOVER (3)
/// - 'D' or 'd' -> SEA_CHOICE_DELETE (4)
/// - 'Q' or 'q' -> SEA_CHOICE_QUIT (5)
/// - 'A' or 'a' -> SEA_CHOICE_ABORT (6)
/// - Other -> SEA_CHOICE_NONE (0)
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // Intentional: interpreting c_int as ASCII char code
pub extern "C" fn rs_sea_choice_from_char(c: c_int) -> c_int {
    match c as u8 {
        b'O' | b'o' => SEA_CHOICE_READONLY,
        b'E' | b'e' => SEA_CHOICE_EDIT,
        b'R' | b'r' => SEA_CHOICE_RECOVER,
        b'D' | b'd' => SEA_CHOICE_DELETE,
        b'Q' | b'q' => SEA_CHOICE_QUIT,
        b'A' | b'a' => SEA_CHOICE_ABORT,
        _ => SEA_CHOICE_NONE,
    }
}

/// Get a descriptive string for a SEA_CHOICE value.
///
/// Returns a static string describing the choice, or NULL for invalid values.
///
/// # Safety
/// Returns a pointer to static string data.
#[no_mangle]
pub extern "C" fn rs_sea_choice_name(choice: c_int) -> *const c_char {
    // Static strings for each choice
    static NONE: &[u8] = b"None\0";
    static READONLY: &[u8] = b"Open Read-Only\0";
    static EDIT: &[u8] = b"Edit anyway\0";
    static RECOVER: &[u8] = b"Recover\0";
    static DELETE: &[u8] = b"Delete it\0";
    static QUIT: &[u8] = b"Quit\0";
    static ABORT: &[u8] = b"Abort\0";

    match choice {
        x if x == SEA_CHOICE_NONE => NONE.as_ptr().cast(),
        x if x == SEA_CHOICE_READONLY => READONLY.as_ptr().cast(),
        x if x == SEA_CHOICE_EDIT => EDIT.as_ptr().cast(),
        x if x == SEA_CHOICE_RECOVER => RECOVER.as_ptr().cast(),
        x if x == SEA_CHOICE_DELETE => DELETE.as_ptr().cast(),
        x if x == SEA_CHOICE_QUIT => QUIT.as_ptr().cast(),
        x if x == SEA_CHOICE_ABORT => ABORT.as_ptr().cast(),
        _ => std::ptr::null(),
    }
}

/// Check if a SEA_CHOICE value means "proceed with editing".
///
/// Returns true for: EDIT, RECOVER
#[no_mangle]
pub extern "C" fn rs_sea_choice_proceeds(choice: c_int) -> c_int {
    c_int::from(choice == SEA_CHOICE_EDIT || choice == SEA_CHOICE_RECOVER)
}

/// Check if a SEA_CHOICE value means "abort operation".
///
/// Returns true for: QUIT, ABORT
#[no_mangle]
pub extern "C" fn rs_sea_choice_aborts(choice: c_int) -> c_int {
    c_int::from(choice == SEA_CHOICE_QUIT || choice == SEA_CHOICE_ABORT)
}

// =============================================================================
// Byte Order Conversion
// =============================================================================

/// Portable conversion of long to byte array.
///
/// Stores the value in a byte order independent format.
/// This is used for swap file portability across different architectures.
///
/// # Implementation
///
/// Each byte stores 8 bits of the value, starting with the least significant.
///
/// # Safety
/// - `s` must be a valid pointer to at least 8 bytes
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // Intentional: storing bytes
pub unsafe extern "C" fn rs_long_to_char_portable(n: i64, s: *mut c_char) {
    if s.is_null() {
        return;
    }

    let mut val = n;
    for i in 0..8 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = (val & 0xFF) as c_char;
        *s.add(i) = byte;
        val >>= 8;
    }
}

/// Portable conversion of byte array to long.
///
/// Reverses the conversion done by `rs_long_to_char_portable`.
///
/// # Safety
/// - `s` must be a valid pointer to at least 8 bytes
#[no_mangle]
#[allow(clippy::cast_sign_loss)] // Intentional: reading bytes
pub unsafe extern "C" fn rs_char_to_long_portable(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }

    let mut result: i64 = 0;
    for i in (0..8).rev() {
        let byte = i64::from(*s.add(i) as u8);
        result = (result << 8) | byte;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sea_choice_constants() {
        // Verify SEA_CHOICE constants are accessible
        assert_eq!(SEA_CHOICE_NONE, 0);
        assert_eq!(SEA_CHOICE_READONLY, 1);
        assert_eq!(SEA_CHOICE_EDIT, 2);
        assert_eq!(SEA_CHOICE_RECOVER, 3);
        assert_eq!(SEA_CHOICE_DELETE, 4);
        assert_eq!(SEA_CHOICE_QUIT, 5);
        assert_eq!(SEA_CHOICE_ABORT, 6);
    }

    #[test]
    fn test_sea_choice_from_char() {
        assert_eq!(
            rs_sea_choice_from_char(c_int::from(b'O')),
            SEA_CHOICE_READONLY
        );
        assert_eq!(
            rs_sea_choice_from_char(c_int::from(b'o')),
            SEA_CHOICE_READONLY
        );
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'E')), SEA_CHOICE_EDIT);
        assert_eq!(
            rs_sea_choice_from_char(c_int::from(b'R')),
            SEA_CHOICE_RECOVER
        );
        assert_eq!(
            rs_sea_choice_from_char(c_int::from(b'D')),
            SEA_CHOICE_DELETE
        );
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'Q')), SEA_CHOICE_QUIT);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'A')), SEA_CHOICE_ABORT);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'X')), SEA_CHOICE_NONE);
    }

    #[test]
    fn test_sea_choice_proceeds() {
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_NONE), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_READONLY), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_EDIT), 1);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_RECOVER), 1);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_DELETE), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_QUIT), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_ABORT), 0);
    }

    #[test]
    fn test_sea_choice_aborts() {
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_NONE), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_READONLY), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_EDIT), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_RECOVER), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_DELETE), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_QUIT), 1);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_ABORT), 1);
    }

    #[test]
    fn test_long_char_conversion() {
        let mut buf = [0i8; 8];

        unsafe {
            // Test positive value
            rs_long_to_char_portable(0x1234_5678_9ABC_DEF0, buf.as_mut_ptr());
            let result = rs_char_to_long_portable(buf.as_ptr());
            assert_eq!(result, 0x1234_5678_9ABC_DEF0);

            // Test negative value
            rs_long_to_char_portable(-1, buf.as_mut_ptr());
            let result = rs_char_to_long_portable(buf.as_ptr());
            assert_eq!(result, -1);

            // Test zero
            rs_long_to_char_portable(0, buf.as_mut_ptr());
            let result = rs_char_to_long_portable(buf.as_ptr());
            assert_eq!(result, 0);
        }
    }
}
