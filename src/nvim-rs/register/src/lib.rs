//! Register utilities for Neovim
//!
//! This crate provides functions for managing yank registers.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

pub mod blockwise;

pub use blockwise::*;

use std::ffi::{c_char, c_int, c_void};

/// MotionType values matching `normal_defs.h`.
pub const K_MT_CHAR_WISE: c_int = 0;
pub const K_MT_LINE_WISE: c_int = 1;
pub const K_MT_BLOCK_WISE: c_int = 2;
pub const K_MT_UNKNOWN: c_int = -1;

/// CTRL-V character (0x16 = 22 decimal).
const CTRL_V: u8 = 0x16;

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

/// Register struct matching C's `yankreg_T` exactly (40 bytes on 64-bit).
///
/// Layout matches `register_defs.h`:
/// ```c
/// typedef struct {
///   String *y_array;          // Pointer to an array of Strings.
///   size_t y_size;            // Number of lines in y_array.
///   MotionType y_type;        // Register type
///   colnr_T y_width;          // Register width (only valid for blockwise).
///   Timestamp timestamp;      // Time when register was last modified.
///   AdditionalData *additional_data;  // Additional data from ShaDa file.
/// } yankreg_T;
/// ```
#[repr(C)]
#[derive(Clone, Copy)]
pub struct YankReg {
    pub y_array: *mut NvimString,
    pub y_size: usize,
    pub y_type: c_int,  // MotionType
    pub y_width: c_int, // colnr_T
    pub timestamp: u64, // Timestamp
    pub additional_data: *mut c_void,
}

const _: () = assert!(std::mem::size_of::<YankReg>() == 40);

impl YankReg {
    /// A zero-initialised register (empty, no array).
    pub const ZERO: Self = Self {
        y_array: std::ptr::null_mut(),
        y_size: 0,
        y_type: K_MT_CHAR_WISE,
        y_width: 0,
        timestamp: 0,
        additional_data: std::ptr::null_mut(),
    };

    /// Return true when the register holds no meaningful text.
    pub fn is_empty(&self) -> bool {
        self.y_array.is_null()
            || self.y_size == 0
            || (self.y_size == 1
                && self.y_type == K_MT_CHAR_WISE
                && unsafe { (*self.y_array).size == 0 })
    }
}

// ---------------------------------------------------------------------------
// Global register state – owned by Rust, referenced by C via `extern`.
// ---------------------------------------------------------------------------

/// All yank registers.
#[no_mangle]
pub static mut y_regs: [YankReg; 39] = [YankReg::ZERO; 39];

/// Pointer to the last-written register ("" unnamed paste source).
#[no_mangle]
pub static mut y_previous: *mut YankReg = std::ptr::null_mut();

/// The saved expression-register source line.
#[no_mangle]
pub static mut expr_line: *mut c_char = std::ptr::null_mut();

/// Last character used with `@` (for `@@` repeat).
#[no_mangle]
pub static mut execreg_lastc: c_int = 0;

// ---------------------------------------------------------------------------
// FFI declarations – real C functions, no wrapper overhead.
// ---------------------------------------------------------------------------

extern "C" {
    // Memory management
    fn xfree(ptr: *mut c_void);
    fn xstrdup(str: *const c_char) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xmallocz(size: usize) -> *mut c_char;
    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    #[allow(dead_code)]
    fn xmemdup(src: *const c_void, len: usize) -> *mut c_void;

    // String helpers
    fn cstr_to_string(str: *const c_char) -> NvimString;
    fn copy_string(s: NvimString, arena: *mut c_void) -> NvimString;

    // Time
    fn os_time() -> u64;

    // Expression evaluation
    fn eval_to_string(expr: *mut c_char, want_retval: bool, in_sandbox: bool) -> *mut c_char;

    // Multibyte
    fn mb_string2cells_len(str: *const c_char, size: usize) -> usize;
    fn mb_string2cells(str: *const c_char) -> usize;
    fn utf_ptr2cells_len(p: *const c_char, size: c_int) -> c_int;
    fn utf_ptr2len_len(p: *const c_char, size: c_int) -> c_int;

    // Memory utilities
    fn memcnt(str: *const c_char, c: c_char, len: usize) -> usize;
    fn memchrsub(data: *mut c_char, from: c_char, to: c_char, len: usize);

    // Clipboard
    fn get_clipboard(name: c_int, target: *mut *mut YankReg, quiet: bool) -> bool;
    fn set_clipboard(name: c_int, reg: *mut YankReg);

    // Error messages
    fn emsg_invreg(name: c_int);

    // Buffer list (for write_reg_contents_ex)
    fn buflist_findnr(nr: c_int) -> *mut c_void;
    fn buflist_findpat(
        pat: *const c_char,
        patend: *const c_char,
        unlisted: bool,
        diffmode: bool,
        curtab_only: bool,
    ) -> c_int;

    // Search pattern
    fn set_last_search_pat(
        s: *const c_char,
        which: c_int,
        keep_capitalize: bool,
        update_prev: bool,
    );

    // Command line
    fn getcmdline(firstc: c_int, count: i64, indent: c_int, do_concat: bool) -> *mut c_char;
}

/// Register index constants (matching `register_defs.h`).
pub const DELETION_REGISTER: c_int = 36;
pub const NUM_SAVED_REGISTERS: c_int = 37;
pub const STAR_REGISTER: c_int = 37;
pub const PLUS_REGISTER: c_int = 38;
pub const NUM_REGISTERS: c_int = 39;

/// RE_SEARCH constant for set_last_search_pat.
const RE_SEARCH: c_int = 0;

/// NUL character.
const NUL: c_int = 0;

// ---------------------------------------------------------------------------
// Pure helper functions (no FFI, no unsafe).
// ---------------------------------------------------------------------------

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

/// Check if a character appears in a byte string literal.
#[inline]
fn strchr(s: &[u8], c: u8) -> bool {
    s.contains(&c)
}

// ---------------------------------------------------------------------------
// Register name / index helpers (exported with original C names via Phase 2).
// ---------------------------------------------------------------------------

/// Check if register should be inserted literally (selection or clipboard).
pub fn is_literal_register(c: u8) -> bool {
    c == b'*' || c == b'+' || ascii_isalnum(c)
}

/// FFI wrapper.
#[no_mangle]
pub extern "C" fn rs_is_literal_register(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return 0;
    };
    c_int::from(is_literal_register(c))
}

/// Convert register name into register index.
///
/// Returns the index in the `y_regs` array, or -1 if the register name is not recognised.
#[no_mangle]
pub extern "C" fn rs_op_reg_index(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return -1;
    };
    if ascii_isdigit(c) {
        c_int::from(c - b'0')
    } else if ascii_islower(c) {
        c_int::from(c - b'a') + 10
    } else if ascii_isupper(c) {
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
/// Exported as both `rs_valid_yank_reg` (legacy alias kept for Rust internal use)
/// and `valid_yank_reg` (the canonical C-visible name).
#[unsafe(export_name = "valid_yank_reg")]
pub extern "C" fn rs_valid_yank_reg(regname: c_int, writing: bool) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    if regname > 0 && ascii_isalnum(c) {
        return true;
    }
    if !writing && strchr(b"/.%:=", c) {
        return true;
    }
    matches!(c, b'#' | b'"' | b'-' | b'_' | b'*' | b'+')
}

// ---------------------------------------------------------------------------
// Global state accessors
// ---------------------------------------------------------------------------

/// Get the index of the register that "" points to.
///
/// Returns the index of `y_previous` in `y_regs`, or -1 if NULL.
///
/// # Safety
///
/// Reads `y_regs` and `y_previous` globals.
#[unsafe(export_name = "get_unname_register")]
pub unsafe extern "C" fn rs_get_unname_register() -> c_int {
    if y_previous.is_null() {
        return -1;
    }
    let base = (&raw const y_regs).cast::<YankReg>();
    let idx = y_previous.offset_from(base);
    if idx >= 0 && idx < NUM_REGISTERS as isize {
        idx as c_int
    } else {
        -1
    }
}

/// Get a pointer to `y_regs[reg]`.
///
/// # Safety
///
/// `reg` must be in `0..NUM_REGISTERS`.
#[unsafe(export_name = "get_y_register")]
pub unsafe extern "C" fn rs_get_y_register(reg: c_int) -> *mut YankReg {
    &raw mut y_regs[reg as usize]
}

/// Get the previous yank register pointer.
///
/// # Safety
///
/// Reads `y_previous`.
#[unsafe(export_name = "get_y_previous")]
pub unsafe extern "C" fn rs_get_y_previous() -> *mut YankReg {
    y_previous
}

// ---------------------------------------------------------------------------
// Register iteration (ported from C op_reg_iter / op_global_reg_iter).
// ---------------------------------------------------------------------------

/// Iterate over registers in `regs`.
///
/// Pass `iter = NULL` to start.  Returns NULL when done.
///
/// # Safety
///
/// All pointer arguments must be valid.
#[unsafe(export_name = "op_reg_iter")]
pub unsafe extern "C" fn rs_op_reg_iter(
    iter: *const c_void,
    regs: *const YankReg,
    name: *mut c_char,
    reg: *mut YankReg,
    is_unnamed: *mut bool,
) -> *const c_void {
    *name = b'\0' as c_char;

    let mut iter_reg: *const YankReg = if iter.is_null() {
        regs
    } else {
        iter as *const YankReg
    };

    // Advance past empty registers.
    while iter_reg.offset_from(regs) < NUM_SAVED_REGISTERS as isize && (*iter_reg).is_empty() {
        iter_reg = iter_reg.add(1);
    }

    if iter_reg.offset_from(regs) == NUM_SAVED_REGISTERS as isize || (*iter_reg).is_empty() {
        return std::ptr::null();
    }

    let iter_off = iter_reg.offset_from(regs) as c_int;
    *name = rs_get_register_name(iter_off) as c_char;
    *reg = *iter_reg;
    *is_unnamed = std::ptr::eq(iter_reg, y_previous as *const _);

    // Find next non-empty register.
    iter_reg = iter_reg.add(1);
    while iter_reg.offset_from(regs) < NUM_SAVED_REGISTERS as isize {
        if !(*iter_reg).is_empty() {
            return iter_reg as *const c_void;
        }
        iter_reg = iter_reg.add(1);
    }
    std::ptr::null()
}

/// Iterate over global registers.
///
/// # Safety
///
/// All pointer arguments must be valid.
#[unsafe(export_name = "op_global_reg_iter")]
pub unsafe extern "C" fn rs_op_global_reg_iter(
    iter: *const c_void,
    name: *mut c_char,
    reg: *mut YankReg,
    is_unnamed: *mut bool,
) -> *const c_void {
    rs_op_reg_iter(
        iter,
        (&raw const y_regs).cast::<YankReg>(),
        name,
        reg,
        is_unnamed,
    )
}

/// Get number of non-empty registers.
///
/// # Safety
///
/// Reads `y_regs`.
#[unsafe(export_name = "op_reg_amount")]
pub unsafe extern "C" fn rs_op_reg_amount() -> usize {
    y_regs[..NUM_SAVED_REGISTERS as usize]
        .iter()
        .filter(|r| !r.is_empty())
        .count()
}

/// Set register `name` to `reg`, optionally marking it as unnamed.
///
/// # Safety
///
/// Modifies `y_regs` and optionally `y_previous`.
#[unsafe(export_name = "op_reg_set")]
pub unsafe extern "C" fn rs_op_reg_set(name: c_char, reg: YankReg, is_unnamed: bool) -> bool {
    let i = rs_op_reg_index(c_int::from(name as u8));
    if i == -1 {
        return false;
    }
    rs_free_register(&raw mut y_regs[i as usize]);
    y_regs[i as usize] = reg;
    if is_unnamed {
        y_previous = &raw mut y_regs[i as usize];
    }
    true
}

/// Get register with the given name.
///
/// Returns a pointer to the register contents, or NULL if the name is invalid.
///
/// # Safety
///
/// Reads `y_regs`.
#[unsafe(export_name = "op_reg_get")]
pub unsafe extern "C" fn rs_op_reg_get(name: c_char) -> *mut YankReg {
    let i = rs_op_reg_index(c_int::from(name as u8));
    if i == -1 {
        return std::ptr::null_mut();
    }
    &raw mut y_regs[i as usize]
}

/// Set the previous yank register.
///
/// Returns true on success, false if the register name is invalid.
///
/// # Safety
///
/// Modifies `y_previous`.
#[unsafe(export_name = "op_reg_set_previous")]
pub unsafe extern "C" fn rs_op_reg_set_previous(name: c_char) -> bool {
    let i = rs_op_reg_index(c_int::from(name as u8));
    if i == -1 {
        return false;
    }
    y_previous = &raw mut y_regs[i as usize];
    true
}

// ---------------------------------------------------------------------------
// get_yank_register (ported from C).
// ---------------------------------------------------------------------------

/// Return a pointer to the register to use for `regname`.
///
/// Cannot handle the `_` (black hole) register.
/// Must only be called with a valid register name!
///
/// # Safety
///
/// Reads/writes `y_regs` and `y_previous`.
#[unsafe(export_name = "get_yank_register")]
pub unsafe extern "C" fn rs_get_yank_register(regname: c_int, mode: c_int) -> *mut YankReg {
    const YREG_PASTE: c_int = 0;
    const YREG_YANK: c_int = 1;
    const YREG_PUT: c_int = 2;

    // Try clipboard for paste/put modes.
    if mode == YREG_PASTE || mode == YREG_PUT {
        let mut reg: *mut YankReg = std::ptr::null_mut();
        if get_clipboard(regname, &raw mut reg, false) {
            return reg;
        }
    }

    if mode == YREG_PUT && (regname == c_int::from(b'*') || regname == c_int::from(b'+')) {
        // Clipboard not available; return an empty register.
        static mut EMPTY_REG: YankReg = YankReg::ZERO;
        return &raw mut EMPTY_REG;
    }

    if mode != YREG_YANK
        && (regname == 0
            || regname == c_int::from(b'"')
            || regname == c_int::from(b'*')
            || regname == c_int::from(b'+'))
        && !y_previous.is_null()
    {
        return y_previous;
    }

    let mut i = rs_op_reg_index(regname);
    if i == -1 {
        i = 0;
    }
    let reg = &raw mut y_regs[i as usize];

    if mode == YREG_YANK {
        y_previous = reg;
    }
    reg
}

// ---------------------------------------------------------------------------
// clear_registers (EXITFREE cleanup).
// ---------------------------------------------------------------------------

/// Free all registers.
///
/// # Safety
///
/// Modifies `y_regs`.
#[unsafe(export_name = "clear_registers")]
pub unsafe extern "C" fn rs_clear_registers() {
    let base = (&raw mut y_regs).cast::<YankReg>();
    for i in 0..NUM_REGISTERS as usize {
        rs_free_register(base.add(i));
    }
}

// ---------------------------------------------------------------------------
// update_yankreg_width
// ---------------------------------------------------------------------------

/// Updates the `y_width` of a blockwise register based on its contents.
///
/// # Safety
///
/// `reg` must be a valid pointer to a `YankReg`.
#[unsafe(export_name = "update_yankreg_width")]
pub unsafe extern "C" fn rs_update_yankreg_width(reg: *mut YankReg) {
    if reg.is_null() {
        return;
    }
    let r = &mut *reg;
    if r.y_type != K_MT_BLOCK_WISE {
        return;
    }
    let mut maxlen: usize = 0;
    for i in 0..r.y_size {
        let s = &*r.y_array.add(i);
        let rowlen = mb_string2cells_len(s.data, s.size);
        maxlen = maxlen.max(rowlen);
    }
    let new_width = if maxlen > 0 { (maxlen - 1) as c_int } else { 0 };
    r.y_width = r.y_width.max(new_width);
}

// ---------------------------------------------------------------------------
// Expression register
// ---------------------------------------------------------------------------

/// Set the expression for the `=` register (takes ownership of `new_line`).
///
/// # Safety
///
/// `new_line` must be a C-allocated string or NULL.
#[unsafe(export_name = "set_expr_line")]
pub unsafe extern "C" fn rs_set_expr_line(new_line: *mut c_char) {
    if !expr_line.is_null() {
        xfree(expr_line as *mut c_void);
    }
    expr_line = new_line;
}

/// Get the `=` register expression itself, without evaluating it.
///
/// Returns a newly allocated copy, or NULL.
///
/// # Safety
///
/// Returns a C-allocated string that the caller must free.
#[unsafe(export_name = "get_expr_line_src")]
pub unsafe extern "C" fn rs_get_expr_line_src() -> *mut c_char {
    if expr_line.is_null() {
        return std::ptr::null_mut();
    }
    xstrdup(expr_line)
}

/// Get the result of the `=` register expression.
///
/// Returns a newly allocated evaluated string, or NULL for failure.
///
/// # Safety
///
/// Returns a C-allocated string that the caller must free.
#[unsafe(export_name = "get_expr_line")]
pub unsafe extern "C" fn rs_get_expr_line() -> *mut c_char {
    static mut NESTED: i32 = 0;

    if expr_line.is_null() {
        return std::ptr::null_mut();
    }

    // Copy so that evaluation cannot corrupt the stored expression.
    let expr_copy = xstrdup(expr_line);

    if NESTED >= 10 {
        return expr_copy;
    }

    NESTED += 1;
    let rv = eval_to_string(expr_copy, true, false);
    NESTED -= 1;
    xfree(expr_copy as *mut c_void);
    rv
}

// ---------------------------------------------------------------------------
// free_register
// ---------------------------------------------------------------------------

/// Free a `YankReg`'s contents (does not free the struct itself).
///
/// # Safety
///
/// `reg` must be a valid pointer to a `YankReg`.
#[unsafe(export_name = "free_register")]
pub unsafe extern "C" fn rs_free_register(reg: *mut YankReg) {
    let r = &mut *reg;

    // Free additional_data.
    if !r.additional_data.is_null() {
        xfree(r.additional_data);
        r.additional_data = std::ptr::null_mut();
    }

    if r.y_array.is_null() {
        return;
    }

    // Free each string entry from last to first.
    for i in (0..r.y_size).rev() {
        let s = &mut *r.y_array.add(i);
        if !s.data.is_null() {
            xfree(s.data as *mut c_void);
            s.data = std::ptr::null_mut();
        }
    }

    // Free the array itself.
    xfree(r.y_array as *mut c_void);
    r.y_array = std::ptr::null_mut();
}

// ---------------------------------------------------------------------------
// init_write_reg / finish_write_reg
// ---------------------------------------------------------------------------

/// Initialise a register for writing.
///
/// # Safety
///
/// `old_y_previous` must be valid or NULL.
#[unsafe(export_name = "init_write_reg")]
pub unsafe extern "C" fn rs_init_write_reg(
    name: c_int,
    old_y_previous: *mut *mut YankReg,
    must_append: bool,
) -> *mut YankReg {
    if !rs_valid_yank_reg(name, true) {
        emsg_invreg(name);
        return std::ptr::null_mut();
    }

    if !old_y_previous.is_null() {
        *old_y_previous = y_previous;
    }

    let reg = rs_get_yank_register(name, 1 /* YREG_YANK */);

    if rs_is_append_register(name) == 0 && !must_append {
        rs_free_register(reg);
    }

    reg
}

/// Finalise a register write operation.
///
/// # Safety
///
/// `reg` must be valid.
#[unsafe(export_name = "finish_write_reg")]
pub unsafe extern "C" fn rs_finish_write_reg(
    name: c_int,
    reg: *mut YankReg,
    old_y_previous: *mut YankReg,
) {
    set_clipboard(name, reg);

    if name != c_int::from(b'"') {
        y_previous = old_y_previous;
    }
}

// ---------------------------------------------------------------------------
// yank_register_mline / get_reg_type
// ---------------------------------------------------------------------------

// Control key constants from ascii_defs.h
const CTRL_A: c_int = 1;
const CTRL_F: c_int = 6;
const CTRL_P: c_int = 16;
#[allow(dead_code)]
const CTRL_R: c_int = 18;
#[allow(dead_code)]
const CTRL_U: c_int = 21;
const CTRL_W: c_int = 23;

/// Return values.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Check if the current yank register has `kMTLineWise` type.
///
/// # Safety
///
/// `reg` output pointer must be valid.
#[unsafe(export_name = "yank_register_mline")]
pub unsafe extern "C" fn rs_yank_register_mline(regname: c_int, reg: *mut *mut YankReg) -> bool {
    if !reg.is_null() {
        *reg = std::ptr::null_mut();
    }

    if regname != 0 && !rs_valid_yank_reg(regname, false) {
        return false;
    }

    if regname == c_int::from(b'_') {
        return false;
    }

    let yankreg = rs_get_yank_register(regname, 0 /* YREG_PASTE */);

    if !reg.is_null() {
        *reg = yankreg;
    }

    (*yankreg).y_type == K_MT_LINE_WISE
}

/// Get the type of a register.
///
/// # Safety
///
/// `reg_width` must be valid or NULL.
#[unsafe(export_name = "get_reg_type")]
pub unsafe extern "C" fn rs_get_reg_type(regname: c_int, reg_width: *mut c_int) -> c_int {
    match regname {
        r if r == c_int::from(b'%') => return K_MT_CHAR_WISE,
        r if r == c_int::from(b'#') => return K_MT_CHAR_WISE,
        r if r == c_int::from(b'=') => return K_MT_CHAR_WISE,
        r if r == c_int::from(b':') => return K_MT_CHAR_WISE,
        r if r == c_int::from(b'/') => return K_MT_CHAR_WISE,
        r if r == c_int::from(b'.') => return K_MT_CHAR_WISE,
        r if r == CTRL_F => return K_MT_CHAR_WISE,
        r if r == CTRL_P => return K_MT_CHAR_WISE,
        r if r == CTRL_W => return K_MT_CHAR_WISE,
        r if r == CTRL_A => return K_MT_CHAR_WISE,
        r if r == c_int::from(b'_') => return K_MT_CHAR_WISE,
        _ => {}
    }

    if regname != NUL && !rs_valid_yank_reg(regname, false) {
        return K_MT_UNKNOWN;
    }

    let reg = rs_get_yank_register(regname, 0 /* YREG_PASTE */);

    if (*reg).is_empty() {
        return K_MT_UNKNOWN;
    }

    let reg_type = (*reg).y_type;

    if !reg_width.is_null() && reg_type == K_MT_BLOCK_WISE {
        *reg_width = (*reg).y_width;
    }

    reg_type
}

/// Format the register type as a string.
///
/// # Safety
///
/// `buf` must be valid and at least `buf_len` bytes.
#[unsafe(export_name = "format_reg_type")]
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
            let width = reg_width + 1;
            let formatted = format!("{}", width);
            let formatted_bytes = formatted.as_bytes();
            buf_slice[0] = CTRL_V;
            let copy_len = formatted_bytes.len().min(buf_len - 2);
            buf_slice[1..1 + copy_len].copy_from_slice(&formatted_bytes[..copy_len]);
            buf_slice[1 + copy_len] = 0;
        }
        _ => {
            buf_slice[0] = 0;
        }
    }
}

// ---------------------------------------------------------------------------
// stuff_yank
// ---------------------------------------------------------------------------

/// Stuff string `p` into yank register `regname` as a single line.
///
/// # Safety
///
/// `p` must be a valid C-allocated string.
#[unsafe(export_name = "stuff_yank")]
pub unsafe extern "C" fn rs_stuff_yank(regname: c_int, p: *mut c_char) -> c_int {
    if regname != 0 && !rs_valid_yank_reg(regname, true) {
        xfree(p as *mut c_void);
        return FAIL;
    }

    if regname == c_int::from(b'_') {
        xfree(p as *mut c_void);
        return OK;
    }

    let plen = libc::strlen(p as *const _);
    let reg = rs_get_yank_register(regname, 1 /* YREG_YANK */);
    let r = &mut *reg;

    if rs_is_append_register(regname) != 0 && !r.y_array.is_null() {
        // Append to the last line.
        let last = &*r.y_array.add(r.y_size - 1);
        let last_size = last.size;
        let tmplen = last_size + plen;
        let tmp = xmalloc(tmplen + 1) as *mut c_char;
        std::ptr::copy_nonoverlapping(last.data, tmp, last_size);
        std::ptr::copy_nonoverlapping(p, tmp.add(last_size), plen);
        *tmp.add(tmplen) = 0;
        xfree(p as *mut c_void);
        // Replace last line.
        let last_mut = &mut *r.y_array.add(r.y_size - 1);
        xfree(last_mut.data as *mut c_void);
        *last_mut = NvimString {
            data: tmp,
            size: tmplen,
        };
    } else {
        rs_free_register(reg);
        r.additional_data = std::ptr::null_mut();
        r.y_array = xmalloc(std::mem::size_of::<NvimString>()) as *mut NvimString;
        *r.y_array = NvimString {
            data: p,
            size: plen,
        };
        r.y_size = 1;
        r.y_type = K_MT_CHAR_WISE;
    }

    r.timestamp = os_time();
    OK
}

// ---------------------------------------------------------------------------
// copy_register
// ---------------------------------------------------------------------------

/// Copy a register and return a pointer to a newly allocated copy.
///
/// # Safety
///
/// Caller must free the returned register.
#[unsafe(export_name = "copy_register")]
pub unsafe extern "C" fn rs_copy_register(name: c_int) -> *mut YankReg {
    let src = rs_get_yank_register(name, 0 /* YREG_PASTE */);

    let copy = xmalloc(std::mem::size_of::<YankReg>()) as *mut YankReg;
    // Shallow copy.
    *copy = *src;

    let size = (*src).y_size;
    if size == 0 {
        (*copy).y_array = std::ptr::null_mut();
    } else {
        let array = xcalloc(size, std::mem::size_of::<NvimString>()) as *mut NvimString;
        (*copy).y_array = array;
        for i in 0..size {
            *array.add(i) = copy_string(*(*src).y_array.add(i), std::ptr::null_mut());
        }
    }

    copy
}

// ---------------------------------------------------------------------------
// shift_delete_registers
// ---------------------------------------------------------------------------

/// Shift the delete registers: "9 is cleared, "8 becomes "9, etc.
///
/// # Safety
///
/// Modifies `y_regs` and `y_previous`.
#[unsafe(export_name = "shift_delete_registers")]
pub unsafe extern "C" fn rs_shift_delete_registers(y_append: bool) {
    // Free register "9.
    rs_free_register(&raw mut y_regs[9]);

    // Shift: 9 <- 8 <- ... <- 2 <- 1.
    for n in (2..=9usize).rev() {
        y_regs[n] = y_regs[n - 1];
    }

    if !y_append {
        y_previous = &raw mut y_regs[1];
    }

    // Clear register "1 (will be set by caller).
    y_regs[1] = YankReg::ZERO;
}

// ---------------------------------------------------------------------------
// str_to_reg
// ---------------------------------------------------------------------------

/// NL / CR / NUL constants.
const NL: c_char = b'\n' as c_char;
const CAR: c_char = b'\r' as c_char;
const NUL_CHAR: c_char = 0;

/// Convert a string to register contents.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(export_name = "str_to_reg")]
pub unsafe extern "C" fn rs_str_to_reg(
    y_ptr: *mut YankReg,
    mut yank_type: c_int,
    str: *const c_char,
    len: usize,
    blocklen: c_int,
    str_list: bool,
) {
    let r = &mut *y_ptr;

    if r.y_array.is_null() {
        r.y_size = 0;
    }

    // Determine yank type if unknown.
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

    if str_list {
        let mut ss = str as *const *const c_char;
        while !(*ss).is_null() {
            newlines += 1;
            ss = ss.add(1);
        }
    } else {
        newlines = memcnt(str, b'\n' as c_char, len);
        if yank_type == K_MT_CHAR_WISE || len == 0 || *str.add(len - 1) != b'\n' as c_char {
            extraline = true;
            newlines += 1;
        }
        if r.y_size > 0 && r.y_type == K_MT_CHAR_WISE {
            append = true;
            newlines -= 1;
        }
    }

    let y_size = r.y_size;

    if y_size + newlines == 0 {
        xfree(r.y_array as *mut c_void);
        r.y_array = std::ptr::null_mut();
        return;
    }

    // Grow the array.
    r.y_array = xrealloc(
        r.y_array as *mut c_void,
        (y_size + newlines) * std::mem::size_of::<NvimString>(),
    ) as *mut NvimString;

    let mut lnum = y_size;
    let mut maxlen: usize = 0;

    if str_list {
        let mut ss = str as *const *const c_char;
        while !(*ss).is_null() {
            let s = cstr_to_string(*ss);
            *r.y_array.add(lnum) = s;
            if yank_type == K_MT_BLOCK_WISE {
                let charlen = mb_string2cells(*ss);
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
        let extraline_offset: isize = if extraline { 1 } else { 0 };

        while start < end.offset(extraline_offset) {
            let mut charlen: c_int = 0;
            let mut line_end = start;

            while line_end < end {
                if *line_end == b'\n' as c_char {
                    break;
                }
                if yank_type == K_MT_BLOCK_WISE {
                    charlen +=
                        utf_ptr2cells_len(line_end, (end as isize - line_end as isize) as c_int);
                }

                if *line_end == NUL_CHAR {
                    line_end = line_end.add(1);
                } else {
                    line_end = line_end.add(utf_ptr2len_len(
                        line_end,
                        (end as isize - line_end as isize) as c_int,
                    ) as usize);
                }
            }

            let line_len = (line_end as usize) - (start as usize);
            if (charlen as usize) > maxlen {
                maxlen = charlen as usize;
            }

            let extra = if append {
                lnum -= 1;
                (*r.y_array.add(lnum)).size
            } else {
                0
            };

            let s = xmallocz(line_len + extra);
            if extra > 0 {
                let prev_data = (*r.y_array.add(lnum)).data;
                std::ptr::copy_nonoverlapping(prev_data, s, extra);
            }
            if line_len > 0 {
                std::ptr::copy_nonoverlapping(start, s.add(extra), line_len);
            }
            let s_len = extra + line_len;

            if append {
                let prev = &mut *r.y_array.add(lnum);
                xfree(prev.data as *mut c_void);
                append = false;
            }

            let new_string = NvimString {
                data: s,
                size: s_len,
            };
            *r.y_array.add(lnum) = new_string;

            // Convert NULs to '\n' to prevent truncation.
            memchrsub(s, NUL_CHAR, b'\n' as c_char, s_len);

            lnum += 1;
            start = line_end.add(1);
        }
    }

    r.y_type = yank_type;
    r.y_size = lnum;
    // Free additional_data.
    if !r.additional_data.is_null() {
        xfree(r.additional_data);
        r.additional_data = std::ptr::null_mut();
    }
    r.timestamp = os_time();

    if yank_type == K_MT_BLOCK_WISE {
        r.y_width = if blocklen == -1 {
            (maxlen as c_int) - 1
        } else {
            blocklen
        };
    } else {
        r.y_width = 0;
    }
}

// ---------------------------------------------------------------------------
// get_default_register_name (Phase 3: thin wrapper over adjust_clipboard_name)
// ---------------------------------------------------------------------------

extern "C" {
    fn adjust_clipboard_name(name: *mut c_int, quiet: bool, writing: bool) -> *mut YankReg;
}

/// Check if the default register should be a clipboard register.
///
/// Returns the clipboard register name, or NUL if none.
///
/// # Safety
///
/// Calls `adjust_clipboard_name` which reads global state.
#[unsafe(export_name = "get_default_register_name")]
pub unsafe extern "C" fn rs_get_default_register_name() -> c_int {
    let mut name: c_int = 0;
    adjust_clipboard_name(&raw mut name, true, false);
    name
}

// ---------------------------------------------------------------------------
// get_expr_register
// ---------------------------------------------------------------------------

/// Get an expression for the `"=expr1"` or `CTRL-R =expr1`.
///
/// Returns `'='` when OK, NUL otherwise.
///
/// # Safety
///
/// Calls `getcmdline`.
#[unsafe(export_name = "get_expr_register")]
pub unsafe extern "C" fn rs_get_expr_register() -> c_int {
    let new_line = getcmdline(c_int::from(b'='), 0, 0, true);
    if new_line.is_null() {
        return 0; // NUL
    }
    if *new_line == 0 {
        // use previous line
        xfree(new_line as *mut c_void);
    } else {
        rs_set_expr_line(new_line);
    }
    c_int::from(b'=')
}

// ---------------------------------------------------------------------------
// write_reg_contents family
// ---------------------------------------------------------------------------

extern "C" {
    fn semsg(msg: *const c_char, ...);
    fn emsg(msg: *const c_char);
}

/// Store `str` in register `name`.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(export_name = "write_reg_contents")]
pub unsafe extern "C" fn rs_write_reg_contents(
    name: c_int,
    str: *const c_char,
    len: isize,
    must_append: c_int,
) {
    rs_write_reg_contents_ex(name, str, len, must_append != 0, K_MT_UNKNOWN, 0);
}

/// Store a list of strings in register `name`.
///
/// # Safety
///
/// All pointers must be valid. `strings` must be NULL-terminated.
#[unsafe(export_name = "write_reg_contents_lst")]
pub unsafe extern "C" fn rs_write_reg_contents_lst(
    name: c_int,
    strings: *mut *mut c_char,
    must_append: bool,
    yank_type: c_int,
    block_len: c_int,
) {
    if name == c_int::from(b'/') || name == c_int::from(b'=') {
        let s = if (*strings).is_null() {
            b"\0" as *const u8 as *const c_char
        } else if !(*strings.add(1)).is_null() {
            // E883: search pattern and expression register may not contain two or more lines
            static E883: &[u8] =
                b"E883: Search pattern and expression register may not contain two or more lines\0";
            emsg(E883.as_ptr() as *const c_char);
            return;
        } else {
            *strings as *const c_char
        };
        rs_write_reg_contents_ex(name, s, -1, must_append, yank_type, block_len);
        return;
    }

    if name == c_int::from(b'_') {
        return;
    }

    let mut old_y_previous: *mut YankReg = std::ptr::null_mut();
    let reg = rs_init_write_reg(name, &raw mut old_y_previous, must_append);
    if reg.is_null() {
        return;
    }

    // str_to_reg with str_list=true interprets str as char**; len is unused in that mode.
    rs_str_to_reg(reg, yank_type, strings as *const c_char, 0, block_len, true);
    rs_finish_write_reg(name, reg, old_y_previous);
}

/// Write `str` (length `len`) to register `name`.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(export_name = "write_reg_contents_ex")]
pub unsafe extern "C" fn rs_write_reg_contents_ex(
    name: c_int,
    str: *const c_char,
    len: isize,
    must_append: bool,
    yank_type: c_int,
    block_len: c_int,
) {
    let len: usize = if len < 0 {
        libc::strlen(str)
    } else {
        len as usize
    };

    if name == c_int::from(b'/') {
        set_last_search_pat(str, RE_SEARCH, true, true);
        return;
    }

    if name == c_int::from(b'#') {
        // Set alternate file number.
        let first_char = *str as u8;
        if ascii_isdigit(first_char) {
            // parse decimal number from str
            let mut num: c_int = 0;
            let mut p = str;
            while ascii_isdigit(*p as u8) {
                num = num * 10 + (*p as c_int - b'0' as c_int);
                p = p.add(1);
            }
            let buf = buflist_findnr(num);
            if buf.is_null() {
                static FMTBUF: &[u8] = b"E86: Buffer %lld does not exist\0";
                semsg(FMTBUF.as_ptr() as *const c_char, num as i64);
            } else {
                // Set curwin->w_alt_fnum.
                set_alt_fnum(buf);
            }
        } else {
            let buf_nr = buflist_findpat(str, str.add(len), true, false, false);
            let buf = buflist_findnr(buf_nr);
            if !buf.is_null() {
                set_alt_fnum(buf);
            }
        }
        return;
    }

    if name == c_int::from(b'=') {
        let offset: usize;
        let totlen: usize;
        if must_append && !expr_line.is_null() {
            let exprlen = libc::strlen(expr_line);
            totlen = exprlen + len;
            offset = exprlen;
        } else {
            totlen = len;
            offset = 0;
        }
        expr_line = xrealloc(expr_line as *mut c_void, totlen + 1) as *mut c_char;
        std::ptr::copy_nonoverlapping(str, expr_line.add(offset), len);
        *expr_line.add(totlen) = 0;
        return;
    }

    if name == c_int::from(b'_') {
        return;
    }

    let mut old_y_previous: *mut YankReg = std::ptr::null_mut();
    let reg = rs_init_write_reg(name, &raw mut old_y_previous, must_append);
    if reg.is_null() {
        return;
    }
    rs_str_to_reg(reg, yank_type, str, len, block_len, false);
    rs_finish_write_reg(name, reg, old_y_previous);
}

// Helper: set curwin->w_alt_fnum from a buf_T pointer.
// We reach into C for this tiny detail.
extern "C" {
    fn nvim_register_set_alt_fnum(buf: *mut c_void);
}

unsafe fn set_alt_fnum(buf: *mut c_void) {
    nvim_register_set_alt_fnum(buf);
}

// ---------------------------------------------------------------------------
// prepare_yankreg_from_object / finish_yankreg_from_object
// ---------------------------------------------------------------------------

extern "C" {
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
}

/// Prepare a `YankReg` from an object (API setreg).
///
/// # Safety
///
/// `reg` must point to an empty `YankReg`. `regtype` is a `NvimString` by value.
#[unsafe(export_name = "prepare_yankreg_from_object")]
pub unsafe extern "C" fn rs_prepare_yankreg_from_object(
    reg: *mut YankReg,
    regtype: NvimString,
    _lines: usize,
) -> bool {
    let r = &mut *reg;
    let type_char = if !regtype.data.is_null() && regtype.size > 0 {
        *regtype.data as u8
    } else {
        0
    };

    r.y_type = match type_char {
        0 => K_MT_UNKNOWN,
        b'v' | b'c' => K_MT_CHAR_WISE,
        b'V' | b'l' => K_MT_LINE_WISE,
        b'b' | 0x16 /* Ctrl_V */ => K_MT_BLOCK_WISE,
        _ => return false,
    };

    r.y_width = 0;
    if regtype.size > 1 {
        if r.y_type != K_MT_BLOCK_WISE {
            return false;
        }
        if !ascii_isdigit(*regtype.data.add(1) as u8) {
            return false;
        }
        let mut p = regtype.data.add(1);
        r.y_width = getdigits_int(&raw mut p, false, 1) - 1;
        if regtype.size > (p as usize - regtype.data as usize) {
            return false;
        }
    }

    r.additional_data = std::ptr::null_mut();
    r.timestamp = 0;
    true
}

/// Adjust a `YankReg` after object set.
///
/// # Safety
///
/// `reg` must be valid.
#[unsafe(export_name = "finish_yankreg_from_object")]
pub unsafe extern "C" fn rs_finish_yankreg_from_object(reg: *mut YankReg, clipboard_adjust: bool) {
    let r = &mut *reg;

    if r.y_size > 0 && (*r.y_array.add(r.y_size - 1)).size == 0 {
        if r.y_type != K_MT_CHAR_WISE {
            if r.y_type == K_MT_UNKNOWN || clipboard_adjust {
                r.y_size -= 1;
            }
            if r.y_type == K_MT_UNKNOWN {
                r.y_type = K_MT_LINE_WISE;
            }
        }
    } else if r.y_type == K_MT_UNKNOWN {
        r.y_type = K_MT_CHAR_WISE;
    }

    rs_update_yankreg_width(reg);
}

// ---------------------------------------------------------------------------
// Utility functions kept for callers in other crates.
// ---------------------------------------------------------------------------

/// Check if a register is a clipboard register (* or +).
#[no_mangle]
pub extern "C" fn rs_register_is_clipboard(regname: c_int) -> bool {
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };
    c == b'*' || c == b'+'
}

// ---------------------------------------------------------------------------
// Additional helpers used by shada / ops callers.
// ---------------------------------------------------------------------------

/// Check if a register contains any text.
///
/// # Safety
///
/// Accesses global register state.
#[no_mangle]
pub unsafe extern "C" fn rs_register_has_content(regname: c_int) -> bool {
    let i = rs_op_reg_index(regname);
    if i == -1 {
        return false;
    }
    !y_regs[i as usize].is_empty()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yankreg_size() {
        assert_eq!(std::mem::size_of::<YankReg>(), 40);
    }

    #[test]
    fn test_valid_yank_reg_named() {
        assert!(rs_valid_yank_reg(c_int::from(b'a'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'z'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'A'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'Z'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'a'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'0'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'9'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'5'), true));
    }

    #[test]
    fn test_valid_yank_reg_readonly() {
        assert!(rs_valid_yank_reg(c_int::from(b'.'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'/'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'%'), false));
        assert!(rs_valid_yank_reg(c_int::from(b':'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'='), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'.'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'/'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'%'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b':'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'='), true));
    }

    #[test]
    fn test_valid_yank_reg_special() {
        assert!(rs_valid_yank_reg(c_int::from(b'#'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'"'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'-'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'_'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'*'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'+'), false));
    }

    #[test]
    fn test_valid_yank_reg_invalid() {
        assert!(!rs_valid_yank_reg(c_int::from(b' '), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'!'), false));
        assert!(!rs_valid_yank_reg(-1, false));
        assert!(!rs_valid_yank_reg(256, false));
        assert!(!rs_valid_yank_reg(0, false));
    }

    #[test]
    fn test_is_literal_register() {
        assert_ne!(rs_is_literal_register(c_int::from(b'a')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'Z')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'0')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'*')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'+')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'-')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'"')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'#')), 0);
    }

    #[test]
    fn test_op_reg_index() {
        assert_eq!(rs_op_reg_index(c_int::from(b'0')), 0);
        assert_eq!(rs_op_reg_index(c_int::from(b'9')), 9);
        assert_eq!(rs_op_reg_index(c_int::from(b'a')), 10);
        assert_eq!(rs_op_reg_index(c_int::from(b'z')), 35);
        assert_eq!(rs_op_reg_index(c_int::from(b'A')), 10);
        assert_eq!(rs_op_reg_index(c_int::from(b'Z')), 35);
        assert_eq!(rs_op_reg_index(c_int::from(b'-')), DELETION_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'*')), STAR_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'+')), PLUS_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'@')), -1);
        assert_eq!(rs_op_reg_index(-1), -1);
        assert_eq!(rs_op_reg_index(256), -1);
    }
}
