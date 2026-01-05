//! Register utilities for Neovim
//!
//! This crate provides functions for validating register names and operations.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int};

/// MotionType values matching `normal_defs.h`.
pub const K_MT_CHAR_WISE: c_int = 0;
pub const K_MT_LINE_WISE: c_int = 1;
pub const K_MT_BLOCK_WISE: c_int = 2;
pub const K_MT_UNKNOWN: c_int = -1;

/// CTRL-V character (0x16 = 22 decimal).
const CTRL_V: u8 = 0x16;

/// Opaque handle to yankreg_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct YankRegHandle(*mut std::ffi::c_void);

/// String type matching C's `String` struct (data pointer + size).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

impl Default for NvimString {
    fn default() -> Self {
        Self {
            data: std::ptr::null_mut(),
            size: 0,
        }
    }
}

// FFI declarations for C accessor functions
extern "C" {
    /// Get the index of the unnamed register (y_previous - y_regs), or -1 if NULL.
    fn nvim_get_y_previous_index() -> c_int;

    // yankreg_T accessors
    fn nvim_yankreg_get_size(reg: YankRegHandle) -> usize;
    fn nvim_yankreg_get_type(reg: YankRegHandle) -> c_int;
    fn nvim_yankreg_get_width(reg: YankRegHandle) -> c_int;
    fn nvim_yankreg_set_width(reg: YankRegHandle, width: c_int);
    fn nvim_yankreg_get_line_data(reg: YankRegHandle, idx: usize) -> *const c_char;
    fn nvim_yankreg_get_line_size(reg: YankRegHandle, idx: usize) -> usize;
    fn nvim_yankreg_is_empty(reg: YankRegHandle) -> bool;

    // Global register array accessors
    fn nvim_get_y_regs_ptr(idx: c_int) -> YankRegHandle;
    fn nvim_set_y_previous_by_index(idx: c_int);
    fn nvim_free_register(reg: YankRegHandle);
    fn nvim_copy_yankreg(dst: YankRegHandle, src: YankRegHandle);
    fn nvim_clear_yankreg_array(reg: YankRegHandle);

    // mbyte function for calculating string width
    fn rs_mb_string2cells_len(str: *const c_char, size: usize) -> usize;

    // Expression register accessors
    fn nvim_get_expr_line() -> *const c_char;
    fn nvim_set_expr_line_ptr(new_line: *mut c_char);
    fn nvim_xfree(ptr: *mut std::ffi::c_void);
    fn nvim_xstrdup(str: *const c_char) -> *mut c_char;
    fn nvim_eval_to_string(expr: *const c_char, want_retval: bool, in_sandbox: bool)
        -> *mut c_char;

    // Phase 4 accessors: init_write_reg / finish_write_reg support
    fn nvim_get_yank_register_for_write(regname: c_int) -> YankRegHandle;
    fn nvim_emsg_invreg(name: c_int);
    fn nvim_get_y_previous() -> YankRegHandle;
    fn nvim_set_y_previous(reg: YankRegHandle);
    fn nvim_set_clipboard(name: c_int, reg: YankRegHandle);

    // Phase 5 accessors: get_reg_type support
    fn nvim_get_yank_register_for_paste(regname: c_int) -> YankRegHandle;

    // Phase 7/8 accessors: free_register and stuff_yank support
    fn nvim_xmalloc(size: usize) -> *mut std::ffi::c_void;
    fn nvim_os_time() -> u64;
    fn nvim_yankreg_has_array(reg: YankRegHandle) -> bool;
    fn nvim_yankreg_free_additional_data(reg: YankRegHandle);
    fn nvim_yankreg_clear_string_at(reg: YankRegHandle, idx: usize);
    fn nvim_yankreg_free_array(reg: YankRegHandle);
    fn nvim_yankreg_set_size(reg: YankRegHandle, size: usize);
    fn nvim_yankreg_set_type(reg: YankRegHandle, reg_type: c_int);
    fn nvim_yankreg_set_timestamp(reg: YankRegHandle, ts: u64);
    fn nvim_yankreg_null_additional_data(reg: YankRegHandle);
    fn nvim_yankreg_alloc_array(reg: YankRegHandle, count: usize);
    fn nvim_yankreg_set_line(reg: YankRegHandle, idx: usize, data: *mut c_char, len: usize);
    fn nvim_yankreg_get_last_line_data(reg: YankRegHandle) -> *const c_char;
    fn nvim_yankreg_get_last_line_size(reg: YankRegHandle) -> usize;
    fn nvim_yankreg_replace_last_line(reg: YankRegHandle, data: *mut c_char, len: usize);

    // Phase 8 accessors: copy_register support
    fn nvim_alloc_yankreg() -> YankRegHandle;
    fn nvim_xcalloc(count: usize, size: usize) -> *mut std::ffi::c_void;
    fn nvim_copy_yankreg_line(
        dst: YankRegHandle,
        dst_idx: usize,
        src: YankRegHandle,
        src_idx: usize,
    );
    fn nvim_yankreg_set_array_ptr(reg: YankRegHandle, array: *mut std::ffi::c_void);

    // Phase 9 accessors: str_to_reg support
    fn nvim_memcnt(str: *const c_char, c: c_char, len: usize) -> usize;
    fn nvim_xmallocz(size: usize) -> *mut c_char;
    fn nvim_memchrsub(data: *mut c_char, from: c_char, to: c_char, len: usize);
    fn nvim_cstr_to_string(str: *const c_char) -> NvimString;
    fn nvim_mb_string2cells(str: *const c_char) -> usize;
    fn nvim_utf_ptr2cells_len(p: *const c_char, size: c_int) -> c_int;
    fn nvim_utf_ptr2len_len(p: *const c_char, size: c_int) -> c_int;
    fn nvim_yankreg_realloc_array(reg: YankRegHandle, count: usize);
    fn nvim_yankreg_set_string_at(reg: YankRegHandle, idx: usize, s: NvimString);
    fn nvim_yankreg_get_data_at(reg: YankRegHandle, idx: usize) -> *mut c_char;
    fn nvim_yankreg_get_size_at(reg: YankRegHandle, idx: usize) -> usize;
    fn nvim_yankreg_free_data_at(reg: YankRegHandle, idx: usize);
}

/// Register index constants (matching `register_defs.h`).
pub const DELETION_REGISTER: c_int = 36;
pub const NUM_SAVED_REGISTERS: c_int = 37;
pub const STAR_REGISTER: c_int = 37;
pub const PLUS_REGISTER: c_int = 38;
pub const NUM_REGISTERS: c_int = 39;

/// Check if a character is an ASCII alphanumeric character (A-Z, a-z, 0-9).
#[inline]
const fn ascii_isalnum(c: u8) -> bool {
    (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || (c >= b'0' && c <= b'9')
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if a character is an ASCII lowercase letter (a-z).
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character appears in a string.
#[inline]
fn strchr(s: &[u8], c: u8) -> bool {
    s.contains(&c)
}

/// Check if register should be inserted literally (selection or clipboard).
///
/// Returns true for '*', '+', or any alphanumeric register name.
#[no_mangle]
pub extern "C" fn rs_is_literal_register(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return 0;
    };
    c_int::from(c == b'*' || c == b'+' || ascii_isalnum(c))
}

/// Convert register name into register index.
///
/// Returns the index in the `y_regs` array, or -1 if the register name is not recognized.
#[no_mangle]
pub extern "C" fn rs_op_reg_index(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return -1;
    };
    if ascii_isdigit(c) {
        // Digits 0-9 map to indices 0-9
        c_int::from(c - b'0')
    } else if ascii_islower(c) {
        // Lowercase a-z maps to indices 10-35
        c_int::from(c - b'a') + 10
    } else if ascii_isupper(c) {
        // Uppercase A-Z maps to indices 10-35 (same as lowercase)
        c_int::from(c - b'A') + 10
    } else if c == b'-' {
        DELETION_REGISTER
    } else if c == b'*' {
        STAR_REGISTER
    } else if c == b'+' {
        PLUS_REGISTER
    } else {
        -1
    }
}

/// Check if register name indicates append mode (uppercase letter).
#[no_mangle]
pub extern "C" fn rs_is_append_register(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return 0;
    };
    c_int::from(ascii_isupper(c))
}

/// Get the index of the register that "" points to.
///
/// Returns the index of `y_previous` in the `y_regs` array, or -1 if
/// `y_previous` is NULL (no previous yank).
///
/// # Safety
///
/// Calls external C function to access global register state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_unname_register() -> c_int {
    nvim_get_y_previous_index()
}

/// Get the character name of the register with the given index.
///
/// Returns the register character name, or '"' for index -1.
#[no_mangle]
pub extern "C" fn rs_get_register_name(num: c_int) -> c_int {
    if num == -1 {
        c_int::from(b'"')
    } else if num < 10 {
        num + c_int::from(b'0')
    } else if num == DELETION_REGISTER {
        c_int::from(b'-')
    } else if num == STAR_REGISTER {
        c_int::from(b'*')
    } else if num == PLUS_REGISTER {
        c_int::from(b'+')
    } else {
        num + c_int::from(b'a') - 10
    }
}

/// Check if `regname` is a valid name of a yank register.
///
/// There is no check for 0 (default register), caller should do this.
/// The black hole register '_' is regarded as valid.
///
/// # Arguments
///
/// * `regname` - name of register (as a character code)
/// * `writing` - allow only writable registers
///
/// # Returns
///
/// `true` if the register name is valid
#[no_mangle]
pub extern "C" fn rs_valid_yank_reg(regname: c_int, writing: bool) -> bool {
    // Convert to u8, invalid values return false
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };

    // Named registers (a-z, A-Z, 0-9)
    if regname > 0 && ascii_isalnum(c) {
        return true;
    }

    // Read-only registers (only valid when not writing): . / % : =
    if !writing && strchr(b"/.%:=", c) {
        return true;
    }

    // Special registers: # " - _ * +
    matches!(c, b'#' | b'"' | b'-' | b'_' | b'*' | b'+')
}

/// Updates the "y_width" of a blockwise register based on its contents.
/// Does nothing on a non-blockwise register.
///
/// # Safety
///
/// The `reg` handle must be a valid pointer to a yankreg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_update_yankreg_width(reg: YankRegHandle) {
    if reg.0.is_null() {
        return;
    }

    let reg_type = nvim_yankreg_get_type(reg);
    if reg_type != K_MT_BLOCK_WISE {
        return;
    }

    let y_size = nvim_yankreg_get_size(reg);
    let mut maxlen: usize = 0;

    for i in 0..y_size {
        let data = nvim_yankreg_get_line_data(reg, i);
        let size = nvim_yankreg_get_line_size(reg, i);
        let rowlen = rs_mb_string2cells_len(data, size);
        maxlen = maxlen.max(rowlen);
    }

    let current_width = nvim_yankreg_get_width(reg);
    // maxlen - 1, but maxlen can be 0
    let new_width = if maxlen > 0 { (maxlen - 1) as c_int } else { 0 };
    nvim_yankreg_set_width(reg, current_width.max(new_width));
}

/// Get the number of non-empty registers.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_op_reg_amount() -> usize {
    let mut count: usize = 0;
    for i in 0..NUM_SAVED_REGISTERS {
        let reg = nvim_get_y_regs_ptr(i);
        if !nvim_yankreg_is_empty(reg) {
            count += 1;
        }
    }
    count
}

/// Get register with the given name.
///
/// Returns a pointer to the register contents, or NULL if the register name is invalid.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_op_reg_get(name: c_char) -> YankRegHandle {
    let i = rs_op_reg_index(c_int::from(name));
    if i == -1 {
        return YankRegHandle(std::ptr::null_mut());
    }
    nvim_get_y_regs_ptr(i)
}

/// Get the previous yank register.
///
/// Returns the `y_previous` pointer (the register that "" points to).
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_get_y_previous() -> YankRegHandle {
    nvim_get_y_previous()
}

/// Set the previous yank register.
///
/// Returns true on success, false if the register name is invalid.
///
/// # Safety
///
/// Modifies global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_op_reg_set_previous(name: c_char) -> bool {
    let i = rs_op_reg_index(c_int::from(name));
    if i == -1 {
        return false;
    }
    nvim_set_y_previous_by_index(i);
    true
}

/// Shift the delete registers: "9 is cleared, "8 becomes "9, etc.
///
/// # Safety
///
/// Modifies global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_shift_delete_registers(y_append: bool) {
    // Free register "9
    let reg9 = nvim_get_y_regs_ptr(9);
    nvim_free_register(reg9);

    // Shift registers: 9 <- 8 <- 7 <- ... <- 2 <- 1
    for n in (2..=9).rev() {
        let dst = nvim_get_y_regs_ptr(n);
        let src = nvim_get_y_regs_ptr(n - 1);
        nvim_copy_yankreg(dst, src);
    }

    // Set y_previous to register "1 if not appending
    if !y_append {
        nvim_set_y_previous_by_index(1);
    }

    // Set register "1 to empty
    let reg1 = nvim_get_y_regs_ptr(1);
    nvim_clear_yankreg_array(reg1);
}

/// Set the expression for the '=' register.
/// Argument must be a C-allocated string (takes ownership).
///
/// # Safety
///
/// The `new_line` pointer must be a valid C-allocated string or NULL.
/// This function takes ownership of the string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_expr_line(new_line: *mut c_char) {
    // Free the old expression line
    let old_line = nvim_get_expr_line();
    if !old_line.is_null() {
        nvim_xfree(old_line as *mut std::ffi::c_void);
    }
    // Set the new expression line
    nvim_set_expr_line_ptr(new_line);
}

/// Get the '=' register expression itself, without evaluating it.
/// Returns a newly allocated copy, or NULL if no expression is set.
///
/// # Safety
///
/// Returns a C-allocated string that must be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn rs_get_expr_line_src() -> *mut c_char {
    let expr_line = nvim_get_expr_line();
    if expr_line.is_null() {
        return std::ptr::null_mut();
    }
    nvim_xstrdup(expr_line)
}

/// Get the result of the '=' register expression.
/// Returns a newly allocated string with the evaluated result, or NULL for failure.
///
/// When invoked recursively (more than 10 levels), returns the expression as-is.
///
/// # Safety
///
/// Returns a C-allocated string that must be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn rs_get_expr_line() -> *mut c_char {
    // Use a static counter for recursion depth
    static mut NESTED: i32 = 0;

    let expr_line = nvim_get_expr_line();
    if expr_line.is_null() {
        return std::ptr::null_mut();
    }

    // Make a copy of the expression, because evaluating it may cause it to be changed
    let expr_copy = nvim_xstrdup(expr_line);

    // When we are invoked recursively limit the evaluation to 10 levels
    // Then return the string as-is
    if NESTED >= 10 {
        return expr_copy;
    }

    NESTED += 1;
    let rv = nvim_eval_to_string(expr_copy, true, false);
    NESTED -= 1;
    nvim_xfree(expr_copy as *mut std::ffi::c_void);
    rv
}

/// Initialize a register for writing.
///
/// Validates the register name, saves the old y_previous, gets the register,
/// and optionally frees it if not appending.
///
/// # Arguments
///
/// * `name` - Register name character.
/// * `old_y_previous` - Output pointer to save the old y_previous.
/// * `must_append` - If true, don't free the register even for non-append registers.
///
/// # Returns
///
/// Pointer to the register, or NULL if the register name is invalid.
///
/// # Safety
///
/// The `old_y_previous` pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_init_write_reg(
    name: c_int,
    old_y_previous: *mut YankRegHandle,
    must_append: bool,
) -> YankRegHandle {
    // Check for valid register name
    if !rs_valid_yank_reg(name, true) {
        nvim_emsg_invreg(name);
        return YankRegHandle(std::ptr::null_mut());
    }

    // Save old y_previous - don't want to change the current (unnamed) register
    if !old_y_previous.is_null() {
        *old_y_previous = nvim_get_y_previous();
    }

    // Get the register for writing
    let reg = nvim_get_yank_register_for_write(name);

    // Free the register if not appending
    if rs_is_append_register(name) == 0 && !must_append {
        nvim_free_register(reg);
    }

    reg
}

/// Finalize a register write operation.
///
/// Sends the register to the clipboard and restores y_previous if needed.
///
/// # Arguments
///
/// * `name` - Register name character.
/// * `reg` - The register that was written.
/// * `old_y_previous` - The saved y_previous to restore.
///
/// # Safety
///
/// The `reg` handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_finish_write_reg(
    name: c_int,
    reg: YankRegHandle,
    old_y_previous: YankRegHandle,
) {
    // Send text of clipboard register to the clipboard
    nvim_set_clipboard(name, reg);

    // ':let @" = "val"' should change the meaning of the "" register
    if name != c_int::from(b'"') {
        nvim_set_y_previous(old_y_previous);
    }
}

// Control key constants from ascii_defs.h
const CTRL_A: c_int = 1;
const CTRL_F: c_int = 6;
const CTRL_P: c_int = 16;
const CTRL_W: c_int = 23;
const NUL: c_int = 0;

/// Check if the current yank register has kMTLineWise register type.
///
/// For valid, non-blackhole registers also provides pointer to the register
/// structure prepared for pasting.
///
/// # Arguments
///
/// * `regname` - The name of the register used or 0 for the unnamed register.
/// * `reg` - Output pointer to store the register handle.
///
/// # Returns
///
/// True if the register is linewise, false otherwise.
/// Sets `*reg` to the register handle, or NULL for invalid/blackhole registers.
///
/// # Safety
///
/// The `reg` output pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_yank_register_mline(regname: c_int, reg: *mut YankRegHandle) -> bool {
    // Set output to NULL initially
    if !reg.is_null() {
        *reg = YankRegHandle(std::ptr::null_mut());
    }

    // Validate register name (0 is allowed for unnamed register)
    if regname != 0 && !rs_valid_yank_reg(regname, false) {
        return false;
    }

    // Black hole register is always empty
    if regname == c_int::from(b'_') {
        return false;
    }

    // Get the register for pasting
    let yankreg = nvim_get_yank_register_for_paste(regname);

    // Set output register pointer
    if !reg.is_null() {
        *reg = yankreg;
    }

    // Return whether it's linewise
    nvim_yankreg_get_type(yankreg) == K_MT_LINE_WISE
}

/// Get the type of a register.
///
/// Used for getregtype().
///
/// # Arguments
///
/// * `regname` - The register name character.
/// * `reg_width` - Output pointer for block width (only set for blockwise registers).
///
/// # Returns
///
/// The MotionType of the register, or kMTUnknown for error.
///
/// # Safety
///
/// The `reg_width` pointer must be valid or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_get_reg_type(regname: c_int, reg_width: *mut c_int) -> c_int {
    // Special registers that are always character-wise
    match regname {
        r if r == c_int::from(b'%') => return K_MT_CHAR_WISE, // file name
        r if r == c_int::from(b'#') => return K_MT_CHAR_WISE, // alternate file name
        r if r == c_int::from(b'=') => return K_MT_CHAR_WISE, // expression
        r if r == c_int::from(b':') => return K_MT_CHAR_WISE, // last command line
        r if r == c_int::from(b'/') => return K_MT_CHAR_WISE, // last search-pattern
        r if r == c_int::from(b'.') => return K_MT_CHAR_WISE, // last inserted text
        r if r == CTRL_F => return K_MT_CHAR_WISE,            // Filename under cursor
        r if r == CTRL_P => return K_MT_CHAR_WISE,            // Path under cursor
        r if r == CTRL_W => return K_MT_CHAR_WISE,            // word under cursor
        r if r == CTRL_A => return K_MT_CHAR_WISE,            // WORD under cursor
        r if r == c_int::from(b'_') => return K_MT_CHAR_WISE, // black hole: always empty
        _ => {}
    }

    // Check for valid register name
    if regname != NUL && !rs_valid_yank_reg(regname, false) {
        return K_MT_UNKNOWN;
    }

    // Get the register for pasting
    let reg = nvim_get_yank_register_for_paste(regname);

    // Check if register has content
    if nvim_yankreg_is_empty(reg) {
        return K_MT_UNKNOWN;
    }

    let reg_type = nvim_yankreg_get_type(reg);

    // Set width for blockwise registers
    if !reg_width.is_null() && reg_type == K_MT_BLOCK_WISE {
        *reg_width = nvim_yankreg_get_width(reg);
    }

    reg_type
}

/// Format the register type as a string.
///
/// # Arguments
///
/// * `reg_type` - The register type (MotionType).
/// * `reg_width` - The width, only used if "reg_type" is kMTBlockWise.
/// * `buf` - Buffer to store formatted string.
/// * `buf_len` - The allocated size of the buffer.
///
/// # Safety
///
/// The `buf` pointer must be valid and point to a buffer of at least `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_format_reg_type(
    reg_type: c_int,
    reg_width: c_int,
    buf: *mut c_char,
    buf_len: usize,
) {
    if buf.is_null() || buf_len < 2 {
        return;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf as *mut u8, buf_len);

    match reg_type {
        K_MT_LINE_WISE => {
            buf_slice[0] = b'V';
            buf_slice[1] = 0;
        }
        K_MT_CHAR_WISE => {
            buf_slice[0] = b'v';
            buf_slice[1] = 0;
        }
        K_MT_BLOCK_WISE => {
            // Format as ^V{width+1}
            let width = reg_width + 1;
            let formatted = format!("{}", width);
            let formatted_bytes = formatted.as_bytes();

            buf_slice[0] = CTRL_V;
            let copy_len = formatted_bytes.len().min(buf_len - 2);
            buf_slice[1..1 + copy_len].copy_from_slice(&formatted_bytes[..copy_len]);
            buf_slice[1 + copy_len] = 0;
        }
        _ => {
            // kMTUnknown or invalid
            buf_slice[0] = 0;
        }
    }
}

/// Return values matching nvim/vim_defs.h
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Free a yankreg_T register's contents.
///
/// Frees additional_data and all lines in y_array.
///
/// # Safety
///
/// The `reg` handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_free_register(reg: YankRegHandle) {
    // Free additional_data
    nvim_yankreg_free_additional_data(reg);

    // If y_array is NULL, nothing more to do
    if !nvim_yankreg_has_array(reg) {
        return;
    }

    // Free each line from y_size - 1 to 0
    let size = nvim_yankreg_get_size(reg);
    for i in (0..size).rev() {
        nvim_yankreg_clear_string_at(reg, i);
    }

    // Free the array itself
    nvim_yankreg_free_array(reg);
}

/// Stuff string `p` in a yank register.
///
/// Used by the `setreg()` function to store data.
/// If the `'<` or `'>` register is written to, the text is inserted in the
/// buffer instead.
///
/// # Arguments
///
/// * `regname` - Register name character.
/// * `p` - String to store (takes ownership, will be freed).
///
/// # Returns
///
/// OK on success, FAIL on error.
///
/// # Safety
///
/// The `p` pointer must be valid and point to a C string allocated with xmalloc.
#[no_mangle]
pub unsafe extern "C" fn rs_stuff_yank(regname: c_int, p: *mut c_char) -> c_int {
    // Check for read-only register
    if regname != 0 && !rs_valid_yank_reg(regname, true) {
        nvim_xfree(p as *mut std::ffi::c_void);
        return FAIL;
    }

    // Black hole register: don't do anything
    if regname == c_int::from(b'_') {
        nvim_xfree(p as *mut std::ffi::c_void);
        return OK;
    }

    // Calculate string length
    let plen = libc::strlen(p as *const _);

    // Get the register for writing
    let reg = nvim_get_yank_register_for_write(regname);

    // Check if we should append
    if rs_is_append_register(regname) != 0 && nvim_yankreg_has_array(reg) {
        // Append to last line
        let last_data = nvim_yankreg_get_last_line_data(reg);
        let last_size = nvim_yankreg_get_last_line_size(reg);
        let tmplen = last_size + plen;

        // Allocate new buffer
        let tmp = nvim_xmalloc(tmplen + 1) as *mut c_char;

        // Copy existing data
        std::ptr::copy_nonoverlapping(last_data, tmp, last_size);
        // Copy new data
        std::ptr::copy_nonoverlapping(p, tmp.add(last_size), plen);
        // Null-terminate
        *tmp.add(tmplen) = 0;

        // Free p since we took ownership
        nvim_xfree(p as *mut std::ffi::c_void);

        // Replace the last line (this frees the old data)
        nvim_yankreg_replace_last_line(reg, tmp, tmplen);
    } else {
        // Replace register contents
        rs_free_register(reg);
        nvim_yankreg_null_additional_data(reg);
        nvim_yankreg_alloc_array(reg, 1);
        nvim_yankreg_set_line(reg, 0, p, plen);
        nvim_yankreg_set_size(reg, 1);
        nvim_yankreg_set_type(reg, K_MT_CHAR_WISE);
    }

    nvim_yankreg_set_timestamp(reg, nvim_os_time());
    OK
}

/// Size of the String struct (data pointer + size_t).
const STRING_SIZE: usize = std::mem::size_of::<usize>() * 2;

/// Copy a register and return a pointer to a newly allocated register.
///
/// # Arguments
///
/// * `name` - Register name character.
///
/// # Returns
///
/// Pointer to the newly allocated copy.
///
/// # Safety
///
/// The returned register must be freed by the caller.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_register(name: c_int) -> YankRegHandle {
    // Get the source register
    let src = nvim_get_yank_register_for_paste(name);

    // Allocate a new register
    let copy = nvim_alloc_yankreg();

    // Shallow copy all fields using nvim_copy_yankreg
    nvim_copy_yankreg(copy, src);

    // Get the size
    let size = nvim_yankreg_get_size(copy);

    if size == 0 {
        // Set y_array to NULL
        nvim_yankreg_set_array_ptr(copy, std::ptr::null_mut());
    } else {
        // Allocate new array
        let array = nvim_xcalloc(size, STRING_SIZE);
        nvim_yankreg_set_array_ptr(copy, array);

        // Deep copy each string
        for i in 0..size {
            nvim_copy_yankreg_line(copy, i, src, i);
        }
    }

    copy
}

/// NL (newline) character.
const NL: c_char = b'\n' as c_char;
/// CAR (carriage return) character.
const CAR: c_char = b'\r' as c_char;
/// NUL character.
const NUL_CHAR: c_char = 0;

/// Convert a string to register contents.
///
/// This function handles two modes:
/// - str_list=true: str is actually a char** (NULL-terminated array of C strings)
/// - str_list=false: str is a regular string with embedded newlines
///
/// # Safety
///
/// All pointers must be valid. If str_list is true, str must be a valid char**.
#[no_mangle]
pub unsafe extern "C" fn rs_str_to_reg(
    y_ptr: YankRegHandle,
    mut yank_type: c_int,
    str: *const c_char,
    len: usize,
    blocklen: c_int,
    str_list: bool,
) {
    // If y_array is NULL, set y_size to 0
    if !nvim_yankreg_has_array(y_ptr) {
        nvim_yankreg_set_size(y_ptr, 0);
    }

    // Determine yank type if unknown
    if yank_type == K_MT_UNKNOWN {
        if str_list {
            yank_type = K_MT_LINE_WISE;
        } else if len > 0 {
            let last_char = *str.add(len - 1);
            if last_char == NL || last_char == CAR {
                yank_type = K_MT_LINE_WISE;
            } else {
                yank_type = K_MT_CHAR_WISE;
            }
        } else {
            yank_type = K_MT_CHAR_WISE;
        }
    }

    let mut newlines: usize = 0;
    let mut extraline = false;
    let mut append = false;

    // Count the number of lines within the string
    if str_list {
        // str is actually a char**
        let mut ss = str as *const *const c_char;
        while !(*ss).is_null() {
            newlines += 1;
            ss = ss.add(1);
        }
    } else {
        newlines = nvim_memcnt(str, b'\n' as c_char, len);
        if yank_type == K_MT_CHAR_WISE || len == 0 || *str.add(len - 1) != b'\n' as c_char {
            extraline = true;
            newlines += 1;
        }
        let y_size = nvim_yankreg_get_size(y_ptr);
        let y_type = nvim_yankreg_get_type(y_ptr);
        if y_size > 0 && y_type == K_MT_CHAR_WISE {
            append = true;
            newlines -= 1;
        }
    }

    let y_size = nvim_yankreg_get_size(y_ptr);

    // Without any lines make the register empty
    if y_size + newlines == 0 {
        nvim_yankreg_free_array(y_ptr);
        return;
    }

    // Grow the register array to hold the pointers to the new lines
    nvim_yankreg_realloc_array(y_ptr, y_size + newlines);

    let mut lnum = y_size;
    let mut maxlen: usize = 0;

    // Find the end of each line and save it into the array
    if str_list {
        let mut ss = str as *const *const c_char;
        while !(*ss).is_null() {
            let s = nvim_cstr_to_string(*ss);
            nvim_yankreg_set_string_at(y_ptr, lnum, s);
            if yank_type == K_MT_BLOCK_WISE {
                let charlen = nvim_mb_string2cells(*ss);
                if charlen > maxlen {
                    maxlen = charlen;
                }
            }
            lnum += 1;
            ss = ss.add(1);
        }
    } else {
        let end = str.add(len);
        let mut start = str;
        let extraline_offset = if extraline { 1isize } else { 0isize };

        while start < end.offset(extraline_offset) {
            let mut charlen: c_int = 0;
            let mut line_end = start;

            // Find the end of the line
            while line_end < end {
                if *line_end == b'\n' as c_char {
                    break;
                }
                if yank_type == K_MT_BLOCK_WISE {
                    charlen += nvim_utf_ptr2cells_len(
                        line_end,
                        (end as isize - line_end as isize) as c_int,
                    );
                }

                if *line_end == NUL_CHAR {
                    line_end = line_end.add(1);
                } else {
                    line_end = line_end.add(nvim_utf_ptr2len_len(
                        line_end,
                        (end as isize - line_end as isize) as c_int,
                    ) as usize);
                }
            }

            let line_len = (line_end as usize) - (start as usize);
            if (charlen as usize) > maxlen {
                maxlen = charlen as usize;
            }

            // When appending, copy the previous line and free it after
            let extra = if append {
                lnum -= 1;
                nvim_yankreg_get_size_at(y_ptr, lnum)
            } else {
                0
            };

            let s = nvim_xmallocz(line_len + extra);
            if extra > 0 {
                let prev_data = nvim_yankreg_get_data_at(y_ptr, lnum);
                std::ptr::copy_nonoverlapping(prev_data, s, extra);
            }
            if line_len > 0 {
                std::ptr::copy_nonoverlapping(start, s.add(extra), line_len);
            }
            let s_len = extra + line_len;

            if append {
                nvim_yankreg_free_data_at(y_ptr, lnum);
                append = false;
            }

            // Set the string
            let new_string = NvimString {
                data: s,
                size: s_len,
            };
            nvim_yankreg_set_string_at(y_ptr, lnum, new_string);

            // Convert NULs to '\n' to prevent truncation
            nvim_memchrsub(s, NUL_CHAR, b'\n' as c_char, s_len);

            lnum += 1;
            start = line_end.add(1);
        }
    }

    nvim_yankreg_set_type(y_ptr, yank_type);
    nvim_yankreg_set_size(y_ptr, lnum);
    nvim_yankreg_free_additional_data(y_ptr);
    nvim_yankreg_set_timestamp(y_ptr, nvim_os_time());

    if yank_type == K_MT_BLOCK_WISE {
        let width = if blocklen == -1 {
            (maxlen as c_int) - 1
        } else {
            blocklen
        };
        nvim_yankreg_set_width(y_ptr, width);
    } else {
        nvim_yankreg_set_width(y_ptr, 0);
    }
}

// =============================================================================
// Phase 3: Register System Foundation - Additional Functions
// =============================================================================

/// Get the register index for the unnamed register ("").
/// Returns the index of y_previous, or -1 if not set.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_get_unnamed_register_index() -> c_int {
    nvim_get_y_previous_index()
}

/// Check if a register contains any text.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_has_content(regname: c_int) -> bool {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return false;
    }
    let reg = nvim_get_y_regs_ptr(i);
    !nvim_yankreg_is_empty(reg)
}

/// Get the line count of a register.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_get_line_count(regname: c_int) -> usize {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return 0;
    }
    let reg = nvim_get_y_regs_ptr(i);
    nvim_yankreg_get_size(reg)
}

/// Get the motion type of a register (linewise, charwise, blockwise).
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_get_motion_type(regname: c_int) -> c_int {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return K_MT_UNKNOWN;
    }
    let reg = nvim_get_y_regs_ptr(i);
    if nvim_yankreg_is_empty(reg) {
        return K_MT_UNKNOWN;
    }
    nvim_yankreg_get_type(reg)
}

/// Get the block width of a register (only meaningful for blockwise).
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_get_block_width(regname: c_int) -> c_int {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return 0;
    }
    let reg = nvim_get_y_regs_ptr(i);
    nvim_yankreg_get_width(reg)
}

/// Check if a register is linewise.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_is_linewise(regname: c_int) -> bool {
    rs_register_get_motion_type(regname) == K_MT_LINE_WISE
}

/// Check if a register is charwise.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_is_charwise(regname: c_int) -> bool {
    rs_register_get_motion_type(regname) == K_MT_CHAR_WISE
}

/// Check if a register is blockwise.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_is_blockwise(regname: c_int) -> bool {
    rs_register_get_motion_type(regname) == K_MT_BLOCK_WISE
}

/// Check if a register is a clipboard register (* or +).
#[no_mangle]
pub extern "C" fn rs_register_is_clipboard(regname: c_int) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    c == b'*' || c == b'+'
}

/// Check if a register is a special register (not alphanumeric).
#[no_mangle]
pub extern "C" fn rs_register_is_special(regname: c_int) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    !ascii_isalnum(c)
}

/// Check if a register is a named register (a-z).
#[no_mangle]
pub extern "C" fn rs_register_is_named(regname: c_int) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    ascii_islower(c)
}

/// Check if a register is a numbered register (0-9).
#[no_mangle]
pub extern "C" fn rs_register_is_numbered(regname: c_int) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    ascii_isdigit(c)
}

/// Check if a register is read-only.
#[no_mangle]
pub extern "C" fn rs_register_is_readonly(regname: c_int) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    // Read-only registers: . / % : =
    strchr(b"/.%:=", c)
}

/// Check if a register is the black hole register (_).
#[no_mangle]
pub extern "C" fn rs_register_is_blackhole(regname: c_int) -> bool {
    regname == c_int::from(b'_')
}

/// Check if a register is the expression register (=).
#[no_mangle]
pub extern "C" fn rs_register_is_expression(regname: c_int) -> bool {
    regname == c_int::from(b'=')
}

/// Check if a register is the search register (/).
#[no_mangle]
pub extern "C" fn rs_register_is_search(regname: c_int) -> bool {
    regname == c_int::from(b'/')
}

/// Check if a register is the command register (:).
#[no_mangle]
pub extern "C" fn rs_register_is_command(regname: c_int) -> bool {
    regname == c_int::from(b':')
}

/// Check if a register is the filename register (%).
#[no_mangle]
pub extern "C" fn rs_register_is_filename(regname: c_int) -> bool {
    regname == c_int::from(b'%')
}

/// Check if a register is the alternate filename register (#).
#[no_mangle]
pub extern "C" fn rs_register_is_altfile(regname: c_int) -> bool {
    regname == c_int::from(b'#')
}

/// Check if a register is the insertion register (.).
#[no_mangle]
pub extern "C" fn rs_register_is_insertion(regname: c_int) -> bool {
    regname == c_int::from(b'.')
}

/// Check if a register is the unnamed register (").
#[no_mangle]
pub extern "C" fn rs_register_is_unnamed(regname: c_int) -> bool {
    regname == c_int::from(b'"')
}

/// Check if a register is the small delete register (-).
#[no_mangle]
pub extern "C" fn rs_register_is_small_delete(regname: c_int) -> bool {
    regname == c_int::from(b'-')
}

// =============================================================================
// Phase 4: Register Operations - Additional Functions
// =============================================================================

/// Clear a register's contents.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_clear(regname: c_int) -> c_int {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return FAIL;
    }
    let reg = nvim_get_y_regs_ptr(i);
    nvim_free_register(reg);
    nvim_clear_yankreg_array(reg);
    OK
}

/// Get a line from a register by index.
///
/// # Safety
///
/// Accesses global register state via C FFI.
/// The returned pointer is only valid while the register is unchanged.
#[no_mangle]
pub unsafe extern "C" fn rs_register_get_line(regname: c_int, idx: usize) -> *const c_char {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return std::ptr::null();
    }
    let reg = nvim_get_y_regs_ptr(i);
    let size = nvim_yankreg_get_size(reg);
    if idx >= size {
        return std::ptr::null();
    }
    nvim_yankreg_get_line_data(reg, idx)
}

/// Get the size of a line in a register.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_get_line_size(regname: c_int, idx: usize) -> usize {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return 0;
    }
    let reg = nvim_get_y_regs_ptr(i);
    let size = nvim_yankreg_get_size(reg);
    if idx >= size {
        return 0;
    }
    nvim_yankreg_get_line_size(reg, idx)
}

/// Get the total character count of a register's contents.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_get_total_size(regname: c_int) -> usize {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return 0;
    }
    let reg = nvim_get_y_regs_ptr(i);
    let line_count = nvim_yankreg_get_size(reg);

    let mut total: usize = 0;
    for idx in 0..line_count {
        total += nvim_yankreg_get_line_size(reg, idx);
        // Add 1 for newline (except for the last line in charwise mode)
        if idx < line_count - 1 || nvim_yankreg_get_type(reg) == K_MT_LINE_WISE {
            total += 1;
        }
    }
    total
}

/// Check if a register name is valid for reading.
#[no_mangle]
pub extern "C" fn rs_register_valid_for_read(regname: c_int) -> bool {
    rs_valid_yank_reg(regname, false) || regname == 0
}

/// Check if a register name is valid for writing.
#[no_mangle]
pub extern "C" fn rs_register_valid_for_write(regname: c_int) -> bool {
    rs_valid_yank_reg(regname, true)
}

/// Convert register character to lowercase (for named registers).
#[no_mangle]
pub extern "C" fn rs_register_to_lowercase(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return regname;
    };
    if ascii_isupper(c) {
        c_int::from(c - b'A' + b'a')
    } else {
        regname
    }
}

/// Get the number of non-empty named registers (a-z).
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_count_named() -> c_int {
    let mut count: c_int = 0;
    for c in b'a'..=b'z' {
        let i = rs_op_reg_index(c_int::from(c));
        if i != -1 {
            let reg = nvim_get_y_regs_ptr(i);
            if !nvim_yankreg_is_empty(reg) {
                count += 1;
            }
        }
    }
    count
}

/// Get the number of non-empty numbered registers (0-9).
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_register_count_numbered() -> c_int {
    let mut count: c_int = 0;
    for c in b'0'..=b'9' {
        let i = rs_op_reg_index(c_int::from(c));
        if i != -1 {
            let reg = nvim_get_y_regs_ptr(i);
            if !nvim_yankreg_is_empty(reg) {
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_yank_reg_named() {
        // Alphabetic registers (a-z, A-Z)
        assert!(rs_valid_yank_reg(c_int::from(b'a'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'z'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'A'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'Z'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'a'), true));

        // Numeric registers (0-9)
        assert!(rs_valid_yank_reg(c_int::from(b'0'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'9'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'5'), true));
    }

    #[test]
    fn test_valid_yank_reg_readonly() {
        // Read-only registers: . / % : =
        // These are only valid when NOT writing
        assert!(rs_valid_yank_reg(c_int::from(b'.'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'/'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'%'), false));
        assert!(rs_valid_yank_reg(c_int::from(b':'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'='), false));

        // Not valid when writing
        assert!(!rs_valid_yank_reg(c_int::from(b'.'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'/'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'%'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b':'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'='), true));
    }

    #[test]
    fn test_valid_yank_reg_special() {
        // Special registers: # " - _ * +
        assert!(rs_valid_yank_reg(c_int::from(b'#'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'"'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'-'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'_'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'*'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'+'), false));

        // Also valid when writing
        assert!(rs_valid_yank_reg(c_int::from(b'#'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'"'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'-'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'_'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'*'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'+'), true));
    }

    #[test]
    fn test_valid_yank_reg_invalid() {
        // Invalid register names
        assert!(!rs_valid_yank_reg(c_int::from(b' '), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'!'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'@'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'$'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'^'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'&'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'('), false));
        assert!(!rs_valid_yank_reg(c_int::from(b')'), false));

        // Negative values
        assert!(!rs_valid_yank_reg(-1, false));

        // Values > 255
        assert!(!rs_valid_yank_reg(256, false));

        // Zero (not handled by this function - caller's responsibility)
        assert!(!rs_valid_yank_reg(0, false));
    }

    #[test]
    fn test_is_literal_register() {
        // Alphanumeric registers are literal
        assert_ne!(rs_is_literal_register(c_int::from(b'a')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'Z')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'0')), 0);

        // Star and plus are literal
        assert_ne!(rs_is_literal_register(c_int::from(b'*')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'+')), 0);

        // Other special registers are not literal
        assert_eq!(rs_is_literal_register(c_int::from(b'-')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'"')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'#')), 0);
    }

    #[test]
    fn test_op_reg_index() {
        // Digits map to 0-9
        assert_eq!(rs_op_reg_index(c_int::from(b'0')), 0);
        assert_eq!(rs_op_reg_index(c_int::from(b'9')), 9);

        // Lowercase letters map to 10-35
        assert_eq!(rs_op_reg_index(c_int::from(b'a')), 10);
        assert_eq!(rs_op_reg_index(c_int::from(b'z')), 35);

        // Uppercase letters also map to 10-35
        assert_eq!(rs_op_reg_index(c_int::from(b'A')), 10);
        assert_eq!(rs_op_reg_index(c_int::from(b'Z')), 35);

        // Special registers
        assert_eq!(rs_op_reg_index(c_int::from(b'-')), DELETION_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'*')), STAR_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'+')), PLUS_REGISTER);

        // Invalid returns -1
        assert_eq!(rs_op_reg_index(c_int::from(b'@')), -1);
        assert_eq!(rs_op_reg_index(-1), -1);
        assert_eq!(rs_op_reg_index(256), -1);
    }

    #[test]
    fn test_is_append_register() {
        // Uppercase letters are append registers
        assert_ne!(rs_is_append_register(c_int::from(b'A')), 0);
        assert_ne!(rs_is_append_register(c_int::from(b'Z')), 0);

        // Lowercase letters are not append registers
        assert_eq!(rs_is_append_register(c_int::from(b'a')), 0);
        assert_eq!(rs_is_append_register(c_int::from(b'z')), 0);

        // Other characters are not append registers
        assert_eq!(rs_is_append_register(c_int::from(b'0')), 0);
        assert_eq!(rs_is_append_register(c_int::from(b'-')), 0);
    }

    #[test]
    fn test_get_register_name() {
        // -1 returns '"'
        assert_eq!(rs_get_register_name(-1), c_int::from(b'"'));

        // 0-9 return '0'-'9'
        assert_eq!(rs_get_register_name(0), c_int::from(b'0'));
        assert_eq!(rs_get_register_name(9), c_int::from(b'9'));

        // 10-35 return 'a'-'z'
        assert_eq!(rs_get_register_name(10), c_int::from(b'a'));
        assert_eq!(rs_get_register_name(35), c_int::from(b'z'));

        // Special registers
        assert_eq!(rs_get_register_name(DELETION_REGISTER), c_int::from(b'-'));
        assert_eq!(rs_get_register_name(STAR_REGISTER), c_int::from(b'*'));
        assert_eq!(rs_get_register_name(PLUS_REGISTER), c_int::from(b'+'));
    }

    #[test]
    fn test_format_reg_type() {
        let mut buf = [0u8; 20];

        // kMTLineWise -> 'V'
        unsafe {
            rs_format_reg_type(
                K_MT_LINE_WISE,
                0,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], b'V');
        assert_eq!(buf[1], 0);

        // kMTCharWise -> 'v'
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(
                K_MT_CHAR_WISE,
                0,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], b'v');
        assert_eq!(buf[1], 0);

        // kMTBlockWise -> ^V{width+1}
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(
                K_MT_BLOCK_WISE,
                9,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], CTRL_V);
        assert_eq!(buf[1], b'1');
        assert_eq!(buf[2], b'0');
        assert_eq!(buf[3], 0);

        // kMTBlockWise with width 0 -> ^V1
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(
                K_MT_BLOCK_WISE,
                0,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], CTRL_V);
        assert_eq!(buf[1], b'1');
        assert_eq!(buf[2], 0);

        // kMTUnknown -> empty
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(K_MT_UNKNOWN, 0, buf.as_mut_ptr() as *mut c_char, buf.len());
        }
        assert_eq!(buf[0], 0);
    }

    #[test]
    fn test_motion_type_constants() {
        // Verify motion type constants match C definitions
        assert_eq!(K_MT_CHAR_WISE, 0);
        assert_eq!(K_MT_LINE_WISE, 1);
        assert_eq!(K_MT_BLOCK_WISE, 2);
        assert_eq!(K_MT_UNKNOWN, -1);
    }

    #[test]
    fn test_register_index_constants() {
        // Verify register index constants match C definitions
        assert_eq!(DELETION_REGISTER, 36);
        assert_eq!(NUM_SAVED_REGISTERS, 37);
        assert_eq!(STAR_REGISTER, 37);
        assert_eq!(PLUS_REGISTER, 38);
        assert_eq!(NUM_REGISTERS, 39);
    }

    #[test]
    fn test_ctrl_key_constants() {
        // Verify control key constants match C definitions
        assert_eq!(CTRL_V, 0x16);
        assert_eq!(CTRL_A, 1);
        assert_eq!(CTRL_F, 6);
        assert_eq!(CTRL_P, 16);
        assert_eq!(CTRL_W, 23);
        assert_eq!(NUL, 0);
    }

    #[test]
    fn test_return_value_constants() {
        // Verify OK/FAIL constants match C definitions
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
    }

    #[test]
    fn test_char_constants() {
        // Verify character constants match C definitions
        assert_eq!(NL, b'\n' as c_char);
        assert_eq!(CAR, b'\r' as c_char);
        assert_eq!(NUL_CHAR, 0);
    }

    #[test]
    fn test_register_type_checks() {
        // Clipboard registers
        assert!(rs_register_is_clipboard(c_int::from(b'*')));
        assert!(rs_register_is_clipboard(c_int::from(b'+')));
        assert!(!rs_register_is_clipboard(c_int::from(b'a')));

        // Named registers
        assert!(rs_register_is_named(c_int::from(b'a')));
        assert!(rs_register_is_named(c_int::from(b'z')));
        assert!(!rs_register_is_named(c_int::from(b'A')));
        assert!(!rs_register_is_named(c_int::from(b'0')));

        // Numbered registers
        assert!(rs_register_is_numbered(c_int::from(b'0')));
        assert!(rs_register_is_numbered(c_int::from(b'9')));
        assert!(!rs_register_is_numbered(c_int::from(b'a')));

        // Special registers
        assert!(rs_register_is_blackhole(c_int::from(b'_')));
        assert!(rs_register_is_expression(c_int::from(b'=')));
        assert!(rs_register_is_search(c_int::from(b'/')));
        assert!(rs_register_is_command(c_int::from(b':')));
        assert!(rs_register_is_filename(c_int::from(b'%')));
        assert!(rs_register_is_altfile(c_int::from(b'#')));
        assert!(rs_register_is_insertion(c_int::from(b'.')));
        assert!(rs_register_is_unnamed(c_int::from(b'"')));
        assert!(rs_register_is_small_delete(c_int::from(b'-')));
    }

    #[test]
    fn test_register_to_lowercase() {
        assert_eq!(
            rs_register_to_lowercase(c_int::from(b'A')),
            c_int::from(b'a')
        );
        assert_eq!(
            rs_register_to_lowercase(c_int::from(b'Z')),
            c_int::from(b'z')
        );
        assert_eq!(
            rs_register_to_lowercase(c_int::from(b'a')),
            c_int::from(b'a')
        );
        assert_eq!(
            rs_register_to_lowercase(c_int::from(b'0')),
            c_int::from(b'0')
        );
    }

    #[test]
    fn test_register_validation() {
        // Valid for reading but not writing
        assert!(rs_register_valid_for_read(c_int::from(b'.')));
        assert!(!rs_register_valid_for_write(c_int::from(b'.')));

        // Valid for both
        assert!(rs_register_valid_for_read(c_int::from(b'a')));
        assert!(rs_register_valid_for_write(c_int::from(b'a')));

        // Unnamed register (0) is valid for reading
        assert!(rs_register_valid_for_read(0));
    }
}
