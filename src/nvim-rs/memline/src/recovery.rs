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

use std::ffi::{c_char, c_int, c_void};

use crate::types::BufHandle;

// =============================================================================
// C Implementation Declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // Recovery Operations (C implementations)
    // -------------------------------------------------------------------------

    /// Recover from a swap file
    fn ml_recover(checkext: c_int);

    /// Write swap file info to a dictionary
    fn swapfile_dict(fname: *const c_char, d: *mut c_void);

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

    /// Print swap file info (handles StringBuilder internally)
    fn nvim_swapfile_info_and_print(fname: *mut c_char);

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
// Recovery Functions
// =============================================================================

/// Recover the contents of a buffer from its swap file.
///
/// This reads the swap file and reconstructs the buffer content.
/// If `checkext` is true, verify the file extension is correct.
///
/// # Safety
/// Modifies buffer state and may read from corrupted files.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_recover(checkext: c_int) {
    ml_recover(checkext);
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
                    nvim_swapfile_info_and_print(*files.add(i as usize));
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
/// dictionary with version, user, host, filename, pid, mtime, etc.
///
/// # Arguments
/// * `fname` - Path to the swap file
/// * `d` - Dictionary to populate
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `d` must be a valid dict_T pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_swapfile_dict(fname: *const c_char, d: *mut c_void) {
    if fname.is_null() || d.is_null() {
        return;
    }
    swapfile_dict(fname, d);
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
    fn nvim_b0_get_version_size() -> usize;

    /// Get pointer to b0_uname field (B0_UNAME_SIZE bytes)
    fn nvim_b0_get_uname_ptr(b0: *const c_void) -> *const c_char;

    /// Get pointer to b0_hname field (B0_HNAME_SIZE bytes)
    fn nvim_b0_get_hname_ptr(b0: *const c_void) -> *const c_char;

    /// Get pointer to b0_fname field (B0_FNAME_SIZE_ORG bytes)
    fn nvim_b0_get_fname_ptr(b0: *const c_void) -> *const c_char;

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
    let version_size = nvim_b0_get_version_size();
    let uname = nvim_b0_get_uname_ptr(b0);
    let hname = nvim_b0_get_hname_ptr(b0);
    let fname = nvim_b0_get_fname_ptr(b0);

    has_nul(version, version_size)
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
