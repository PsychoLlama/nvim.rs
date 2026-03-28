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

    // Command-line paste (Phase 1)
    fn cmdline_paste_str(s: *const c_char, literally: bool);
    fn os_breakcheck();

    // Typval list operations (Phase 1)
    fn tv_list_alloc(count: isize) -> *mut c_void;
    fn tv_list_append_string(list: *mut c_void, s: *const c_char, len: isize);
    fn tv_list_append_allocated_string(list: *mut c_void, s: *mut c_char);

    // Special register contents (Phase 1)
    fn get_spec_reg(
        regname: c_int,
        argp: *mut *mut c_char,
        allocated: *mut bool,
        errmsg: bool,
    ) -> bool;

    // Global variables (Phase 1)
    static mut got_int: bool;

    // Typeahead buffer (Phase 2)
    fn ins_typebuf(
        str: *const c_char,
        noremap: c_int,
        offset: c_int,
        nottyped: bool,
        silent: c_int,
    ) -> c_int;
    fn vim_strsave_escape_ks(s: *mut c_char) -> *mut c_char;
    fn vim_strsave_escaped_ext(
        s: *const c_char,
        esc: *const c_char,
        cc: c_int,
        bsl: bool,
    ) -> *mut c_char;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn get_last_insert_save() -> *mut c_char;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;

    // GArray (Phase 2)
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_set_growsize(gap: *mut GArray, growsize: c_int);
    fn ga_concat(gap: *mut GArray, s: *const c_char);
    fn ga_append(gap: *mut GArray, c: u8);
    fn ga_clear(gap: *mut GArray);

    // Global variables (Phase 2)
    static mut reg_executing: c_int;
    static mut pending_end_reg_executing: bool;
    static mut VIsual_active: bool;
    static mut last_cmdline: *mut c_char;
    static mut new_last_cmdline: *mut c_char;
    static mut restart_edit: c_int;

    // Messaging (Phase 3)
    fn msg_puts(s: *const c_char);
    fn msg_putchar(c: c_int);
    fn msg_puts_title(s: *const c_char);
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn msg_outtrans_len(msgstr: *const c_char, len: c_int, hl_id: c_int, hist: bool) -> c_int;
    fn msg_ext_set_kind(kind: *const c_char);
    fn msg(s: *const c_char, hl_id: c_int) -> bool;
    fn ptr2cells(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn message_filtered(msg: *const c_char) -> bool;

    // Buffer list (Phase 3) -- rs_buflist_name_nr is the real implementation; the
    // C buflist_name_nr inline wrapper calls this.
    fn rs_buflist_name_nr(fnum: c_int, fname: *mut *mut c_char, lnum: *mut c_int) -> c_int;

    // Search / mbyte (Phase 3)
    fn last_search_pat() -> *const c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn mb_tolower(a: c_int) -> c_int;

    // Edit mode (Phase 3)
    fn showmode() -> c_int;

    // Autocmds (Phase 3)
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;

    // v:event (Phase 3)
    fn get_v_event(save: *mut c_void) -> *mut c_void;
    fn restore_v_event(dict: *mut c_void, save: *mut c_void);
    fn tv_dict_add_str(
        d: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
    ) -> c_int;
    fn tv_dict_set_keys_readonly(dict: *mut c_void);

    // UI query (Phase 3)
    fn ui_has(feat: c_int) -> bool;

    // Recording (Phase 3)
    fn get_recorded() -> *mut c_char;
    fn vim_unescape_ks(p: *mut c_char);
    fn get_last_insert() -> NvimString;

    // Existing accessors (Phase 3)
    fn nvim_al_curbuf_b_fname() -> *mut c_char;
    fn nvim_al_eap_get_arg(eap: *mut c_void) -> *mut c_char;

    // Global variables (Phase 3)
    static mut Columns: c_int;
    static mut redir_reg: c_int;
    static mut msg_ext_skip_flush: bool;
    static mut reg_recording: c_int;
    static mut reg_recorded: c_int;
    static mut p_ch: i64;
    static mut curbuf: *mut c_void;
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
// GArray struct matching C's garray_T (Phase 2).
// ---------------------------------------------------------------------------

/// Growable array matching C's `garray_T` (24 bytes on 64-bit).
#[repr(C)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GArray {
    fn zeroed() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: std::ptr::null_mut(),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 1: Register Contents and Command-Line Paste
// ---------------------------------------------------------------------------

/// GRegFlags constants matching `register_defs.h`.
const K_GREG_NO_EXPR: c_int = 1;
const K_GREG_EXPR_SRC: c_int = 2;
const K_GREG_LIST: c_int = 4;

/// YREG_PUT mode constant.
const YREG_PUT: c_int = 2;

/// When `flags` has `kGRegList` return a list with text `s`.
/// Otherwise just return `s`.
///
/// # Safety
///
/// `s` must be a valid C-allocated string or NULL.
unsafe fn get_reg_wrap_one_line(s: *mut c_char, flags: c_int) -> *mut c_void {
    if flags & K_GREG_LIST == 0 {
        return s as *mut c_void;
    }
    let list = tv_list_alloc(1);
    tv_list_append_allocated_string(list, s);
    list
}

/// Paste a yank register into the command line.
/// Only for non-special registers.
/// Used by CTRL-R in command-line mode.
///
/// # Safety
///
/// Reads global register state and calls C functions.
#[unsafe(export_name = "cmdline_paste_reg")]
pub unsafe extern "C" fn rs_cmdline_paste_reg(
    regname: c_int,
    literally_arg: bool,
    remcr: bool,
) -> bool {
    let literally = literally_arg || rs_is_literal_register(regname) != 0;

    let reg = rs_get_yank_register(regname, 0 /* YREG_PASTE */);
    if (*reg).y_array.is_null() {
        return false; // FAIL
    }

    let size = (*reg).y_size;
    for i in 0..size {
        let s = &*(*reg).y_array.add(i);
        cmdline_paste_str(s.data, literally);

        // Insert ^M between lines, unless `remcr` is true.
        if i < size - 1 && !remcr {
            cmdline_paste_str(c"\r".as_ptr(), literally);
        }

        // Check for CTRL-C.
        os_breakcheck();
        if got_int {
            return false; // FAIL
        }
    }
    true // OK
}

/// Gets the contents of a register.
/// Used for `@r` in expressions and for `getreg()`.
///
/// # Safety
///
/// Reads global register state and calls C functions.
#[unsafe(export_name = "get_reg_contents")]
pub unsafe extern "C" fn rs_get_reg_contents(regname: c_int, flags: c_int) -> *mut c_void {
    // Don't allow using an expression register inside an expression.
    if regname == c_int::from(b'=') {
        if flags & K_GREG_NO_EXPR != 0 {
            return std::ptr::null_mut();
        }
        if flags & K_GREG_EXPR_SRC != 0 {
            return get_reg_wrap_one_line(rs_get_expr_line_src(), flags);
        }
        return get_reg_wrap_one_line(rs_get_expr_line(), flags);
    }

    // "@@" is used for unnamed register
    let regname = if regname == c_int::from(b'@') {
        c_int::from(b'"')
    } else {
        regname
    };

    // check for valid regname
    if regname != NUL && !rs_valid_yank_reg(regname, false) {
        return std::ptr::null_mut();
    }

    let mut retval: *mut c_char = std::ptr::null_mut();
    let mut allocated = false;
    if get_spec_reg(regname, &raw mut retval, &raw mut allocated, false) {
        if retval.is_null() {
            return std::ptr::null_mut();
        }
        if allocated {
            return get_reg_wrap_one_line(retval, flags);
        }
        return get_reg_wrap_one_line(xstrdup(retval), flags);
    }

    let reg = rs_get_yank_register(regname, YREG_PUT);
    if (*reg).y_array.is_null() {
        return std::ptr::null_mut();
    }

    if flags & K_GREG_LIST != 0 {
        let list = tv_list_alloc((*reg).y_size as isize);
        for i in 0..(*reg).y_size {
            let s = &*(*reg).y_array.add(i);
            tv_list_append_string(list, s.data, -1);
        }
        return list;
    }

    // Compute length of resulting string.
    let mut len: usize = 0;
    for i in 0..(*reg).y_size {
        let s = &*(*reg).y_array.add(i);
        len += s.size;
        // Insert a newline between lines and after last line if y_type is kMTLineWise.
        if (*reg).y_type == K_MT_LINE_WISE || i < (*reg).y_size - 1 {
            len += 1;
        }
    }

    let out = xmalloc(len + 1) as *mut c_char;

    // Copy lines into string.
    let mut offset: usize = 0;
    for i in 0..(*reg).y_size {
        let s = &*(*reg).y_array.add(i);
        std::ptr::copy_nonoverlapping(s.data, out.add(offset), s.size);
        offset += s.size;
        if (*reg).y_type == K_MT_LINE_WISE || i < (*reg).y_size - 1 {
            *out.add(offset) = b'\n' as c_char;
            offset += 1;
        }
    }
    *out.add(offset) = 0;

    out as *mut c_void
}

// ---------------------------------------------------------------------------
// Phase 2: Execute Register Subsystem
// ---------------------------------------------------------------------------

/// REMAP_YES: allow remapping.
const REMAP_YES: c_int = 0;
/// REMAP_NONE: no remapping.
const REMAP_NONE: c_int = -1;

/// Ctrl_V character (0x16 = 22 decimal) -- for vim_strsave_escaped_ext.
const CTRL_V_INT: c_int = 22;

/// Error message for E748.
static E748_MSG: &[u8] = b"E748: No previously used register\0";
/// Error message for E30 (no last command line).
static E_NOLASTCMD: &[u8] = b"E30: No previous command line\0";
/// Error message for E29 (no inserted text).
static E_NOINSTEXT: &[u8] = b"E29: No inserted text yet\0";

/// If "restart_edit" is not zero, put it in the typeahead buffer, so that
/// it's used only after other typeahead has been processed.
///
/// # Safety
///
/// Reads and writes global `restart_edit`.
unsafe fn put_reedit_in_typebuf(silent: c_int) {
    if restart_edit == NUL {
        return;
    }
    let mut buf = [0u8; 3];
    let len = if restart_edit == c_int::from(b'V') {
        buf[0] = b'g';
        buf[1] = b'R';
        2usize
    } else {
        buf[0] = if restart_edit == c_int::from(b'I') {
            b'i'
        } else {
            restart_edit as u8
        };
        1usize
    };
    buf[len] = 0;
    if ins_typebuf(buf.as_ptr() as *const c_char, REMAP_NONE, 0, true, silent) == OK {
        restart_edit = NUL;
    }
}

/// Insert register contents `s` into the typeahead buffer, so that it will be
/// executed again.
///
/// `esc`: when true, escape K_SPECIAL characters and no remapping.
/// `colon`: add ':' before the line.
///
/// # Safety
///
/// Reads global state and calls C functions.
unsafe fn put_in_typebuf(s: *mut c_char, esc: bool, colon: bool, silent: c_int) -> c_int {
    let mut retval = OK;

    put_reedit_in_typebuf(silent);
    if colon {
        retval = ins_typebuf(c"\n".as_ptr(), REMAP_NONE, 0, true, silent);
    }
    if retval == OK {
        let p = if esc { vim_strsave_escape_ks(s) } else { s };
        if p.is_null() {
            retval = FAIL;
        } else {
            retval = ins_typebuf(p, if esc { REMAP_NONE } else { REMAP_YES }, 0, true, silent);
        }
        if esc && !p.is_null() {
            xfree(p as *mut c_void);
        }
    }
    if colon && retval == OK {
        retval = ins_typebuf(c":".as_ptr(), REMAP_NONE, 0, true, silent);
    }
    retval
}

/// When executing a register as a series of ex-commands, if the
/// line-continuation character is used for a line, join it with predecessor
/// lines. Lines are processed backwards.
///
/// Returns a newly allocated concatenated line. Updates `*idx` to the
/// starting line index.
///
/// # Safety
///
/// `lines` must point to valid `NvimString` entries. `idx` must be valid.
unsafe fn execreg_line_continuation(lines: *const NvimString, idx: *mut usize) -> *mut c_char {
    let mut i = *idx;
    assert!(i > 0);
    let cmd_end = i;

    let mut ga = GArray::zeroed();
    ga_init(&raw mut ga, 1 /* sizeof(char) */, 400);

    // Search backwards for the first line of this command.
    while i > 0 {
        i -= 1;
        let s = &*lines.add(i);
        let p = skipwhite(s.data);
        let p_bytes = std::slice::from_raw_parts(p as *const u8, libc::strlen(p));
        let is_continuation = p_bytes.first() == Some(&b'\\')
            || (p_bytes.len() >= 3
                && p_bytes[0] == b'"'
                && p_bytes[1] == b'\\'
                && p_bytes[2] == b' ');
        if !is_continuation {
            break;
        }
    }
    let cmd_start = i;

    // Join all the lines.
    ga_concat(&raw mut ga, (*lines.add(cmd_start)).data);
    for j in (cmd_start + 1)..=cmd_end {
        let s = &*lines.add(j);
        let p = skipwhite(s.data);
        let p_bytes = std::slice::from_raw_parts(p as *const u8, libc::strlen(p));
        if p_bytes.first() == Some(&b'\\') {
            // Adjust growsize to current length to speed up concatenating many lines.
            if ga.ga_len > 400 {
                ga_set_growsize(&raw mut ga, ga.ga_len.min(8000));
            }
            ga_concat(&raw mut ga, p.add(1));
        }
    }
    ga_append(&raw mut ga, 0); // NUL terminator

    let str_ptr = xmemdupz(ga.ga_data, ga.ga_len as usize) as *mut c_char;
    ga_clear(&raw mut ga);

    *idx = i;
    str_ptr
}

/// Execute a yank register: copy it into the stuff buffer.
///
/// `colon`:  insert ':' before each line
/// `addcr`:  always add '\n' to end of line
/// `silent`: set "silent" flag in typeahead buffer
///
/// Returns FAIL for failure, OK otherwise.
///
/// # Safety
///
/// Reads/writes global state and calls C functions.
#[unsafe(export_name = "do_execreg")]
pub unsafe extern "C" fn rs_do_execreg(
    mut regname: c_int,
    colon: c_int,
    addcr: c_int,
    silent: c_int,
) -> c_int {
    let mut retval = OK;

    if regname == c_int::from(b'@') {
        // repeat previous one
        if execreg_lastc == NUL {
            emsg(E748_MSG.as_ptr() as *const c_char);
            return FAIL;
        }
        regname = execreg_lastc;
    }

    // check for valid regname
    if regname == c_int::from(b'%')
        || regname == c_int::from(b'#')
        || !rs_valid_yank_reg(regname, false)
    {
        emsg_invreg(regname);
        return FAIL;
    }
    execreg_lastc = regname;

    if regname == c_int::from(b'_') {
        // black hole: don't stuff anything
        return OK;
    }

    if regname == c_int::from(b':') {
        // use last command line
        if last_cmdline.is_null() {
            emsg(E_NOLASTCMD.as_ptr() as *const c_char);
            return FAIL;
        }
        // don't keep the cmdline containing @:
        xfree(new_last_cmdline as *mut c_void);
        new_last_cmdline = std::ptr::null_mut();

        // Escape all control characters with CTRL-V.
        let esc_chars = b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\0";
        let p = vim_strsave_escaped_ext(
            last_cmdline,
            esc_chars.as_ptr() as *const c_char,
            CTRL_V_INT,
            false,
        );

        // When in Visual mode "'<,'>" will be prepended to the command.
        // Remove it when it's already there.
        let prefix = b"'<,'>".as_ptr() as *const c_char;
        if VIsual_active && libc::strncmp(p, prefix, 5) == 0 {
            retval = put_in_typebuf(p.add(5), true, true, silent);
        } else {
            retval = put_in_typebuf(p, true, true, silent);
        }
        xfree(p as *mut c_void);
    } else if regname == c_int::from(b'=') {
        let p = rs_get_expr_line();
        if p.is_null() {
            return FAIL;
        }
        retval = put_in_typebuf(p, true, colon != 0, silent);
        xfree(p as *mut c_void);
    } else if regname == c_int::from(b'.') {
        // use last inserted text
        let p = get_last_insert_save();
        if p.is_null() {
            emsg(E_NOINSTEXT.as_ptr() as *const c_char);
            return FAIL;
        }
        retval = put_in_typebuf(p, false, colon != 0, silent);
        xfree(p as *mut c_void);
    } else {
        let reg = rs_get_yank_register(regname, 0 /* YREG_PASTE */);
        if (*reg).y_array.is_null() {
            return FAIL;
        }

        // Disallow remapping for ":@r".
        let remap = if colon != 0 { REMAP_NONE } else { REMAP_YES };

        // Insert lines into typeahead buffer, from last one to first one.
        put_reedit_in_typebuf(silent);
        let size = (*reg).y_size;
        let mut i = size;
        while i > 0 {
            i -= 1;
            // insert NL between lines and after last line if type is kMTLineWise
            if ((*reg).y_type == K_MT_LINE_WISE || i < size - 1 || addcr != 0)
                && ins_typebuf(c"\n".as_ptr(), remap, 0, true, silent) == FAIL
            {
                return FAIL;
            }

            // Handle line-continuation for :@<register>
            let mut str_ptr = (*(*reg).y_array.add(i)).data;
            let mut free_str = false;
            if colon != 0 && i > 0 {
                let p = skipwhite(str_ptr);
                let p_bytes = std::slice::from_raw_parts(p as *const u8, libc::strlen(p));
                let is_continuation = p_bytes.first() == Some(&b'\\')
                    || (p_bytes.len() >= 3
                        && p_bytes[0] == b'"'
                        && p_bytes[1] == b'\\'
                        && p_bytes[2] == b' ');
                if is_continuation {
                    str_ptr = execreg_line_continuation((*reg).y_array, &raw mut i);
                    free_str = true;
                }
            }
            let escaped = vim_strsave_escape_ks(str_ptr);
            if free_str {
                xfree(str_ptr as *mut c_void);
            }
            retval = ins_typebuf(escaped, remap, 0, true, silent);
            xfree(escaped as *mut c_void);
            if retval == FAIL {
                return FAIL;
            }
            if colon != 0 && ins_typebuf(c":".as_ptr(), remap, 0, true, silent) == FAIL {
                return FAIL;
            }
        }
        reg_executing = if regname == 0 {
            c_int::from(b'"')
        } else {
            regname
        };
        pending_end_reg_executing = false;
    }
    retval
}

// ---------------------------------------------------------------------------
// Phase 3: Display and Recording
// ---------------------------------------------------------------------------

/// kUIMessages value (UIExtension enum).
const K_UI_MESSAGES: c_int = 4;

/// HLF_8 value (HlGroups enum, first entry after HLF_NONE=0).
const HLF_8: c_int = 1;

/// EVENT_RECORDINGENTER value (from autocmd/src/event.rs: RecordingEnter = 91).
const EVENT_RECORDINGENTER: c_int = 91;

/// EVENT_RECORDINGLEAVE value (from autocmd/src/event.rs: RecordingLeave = 92).
const EVENT_RECORDINGLEAVE: c_int = 92;

/// NUMBUFLEN value (matching NUMBUFLEN in vim.h: max size of a number-to-string buffer).
const NUMBUFLEN: usize = 65;

/// Size of save_v_event_T: bool (1) + padding (7) + hashtab_T (296) = 304 bytes.
const SAVE_V_EVENT_SIZE: usize = 304;

/// Display a string for ex_display(), truncate at end of screen line.
///
/// # Safety
///
/// `p` must be a valid C string.
unsafe fn dis_msg(p: *const c_char, skip_esc: bool) {
    let mut n = Columns - 6;
    let mut pos = p;
    loop {
        if *pos == 0 {
            break;
        }
        // Skip trailing ESC if skip_esc is set.
        if skip_esc && *pos as u8 == 0x1b && *pos.add(1) == 0 {
            break;
        }
        let cells = ptr2cells(pos);
        n -= cells;
        if n < 0 {
            break;
        }
        let l = utfc_ptr2len(pos);
        if l > 1 {
            msg_outtrans_len(pos, l, 0, false);
            pos = pos.add(l as usize);
        } else {
            msg_outtrans_len(pos, 1, 0, false);
            pos = pos.add(1);
        }
    }
    os_breakcheck();
}

/// `:dis` and `:registers`: Display the contents of the yank registers.
///
/// # Safety
///
/// Reads/writes global state and calls C functions.
#[unsafe(export_name = "ex_display")]
pub unsafe extern "C" fn rs_ex_display(eap: *mut c_void) {
    let mut p: *const c_char;
    let arg_raw = nvim_al_eap_get_arg(eap);
    // Normalize: if arg is NULL or empty string, treat as NULL (show all registers).
    let arg: *const c_char = if arg_raw.is_null() || *arg_raw == 0 {
        std::ptr::null()
    } else {
        arg_raw as *const c_char
    };

    let hl_id = HLF_8;

    msg_ext_set_kind(c"list_cmd".as_ptr());
    msg_ext_skip_flush = true;

    // Highlight title.
    msg_puts_title(c"\nType Name Content".as_ptr());

    let mut i: c_int = -1;
    while i < NUM_REGISTERS && !got_int {
        let name = rs_get_register_name(i);

        // filter by arg
        if !arg.is_null() && vim_strchr(arg, name).is_null() {
            i += 1;
            continue;
        }

        let reg_type_code = rs_get_reg_type(name, std::ptr::null_mut());
        let type_char: c_int = match reg_type_code {
            K_MT_LINE_WISE => c_int::from(b'l'),
            K_MT_CHAR_WISE => c_int::from(b'c'),
            _ => c_int::from(b'b'),
        };

        let yb: *mut YankReg = if i == -1 {
            if !y_previous.is_null() {
                y_previous
            } else {
                &raw mut y_regs[0]
            }
        } else {
            &raw mut y_regs[i as usize]
        };

        // Check clipboard.
        get_clipboard(name, &raw mut (*(yb as *mut *mut YankReg)), true);

        // Do not list register being written to.
        if name == mb_tolower(redir_reg)
            || (redir_reg == c_int::from(b'"') && std::ptr::eq(yb, y_previous as *const _))
        {
            i += 1;
            continue;
        }

        if !(*yb).y_array.is_null() {
            let mut do_show = false;
            let mut j = 0usize;
            while !do_show && j < (*yb).y_size {
                let s = &*(*yb).y_array.add(j);
                do_show = !message_filtered(s.data);
                j += 1;
            }

            if do_show || (*yb).y_size == 0 {
                msg_putchar(c_int::from(b'\n'));
                msg_puts(c"  ".as_ptr());
                msg_putchar(type_char);
                msg_puts(c"  ".as_ptr());
                msg_putchar(c_int::from(b'"'));
                msg_putchar(name);
                msg_puts(c"   ".as_ptr());

                let mut n = Columns - 11;
                let mut j = 0usize;
                while j < (*yb).y_size && n > 1 {
                    if j > 0 {
                        msg_puts_hl(c"^J".as_ptr(), hl_id, false);
                        n -= 2;
                    }
                    let line = &*(*yb).y_array.add(j);
                    p = line.data;
                    while *p != 0 {
                        let cells = ptr2cells(p);
                        n -= cells;
                        if n < 0 {
                            break;
                        }
                        let clen = utfc_ptr2len(p);
                        msg_outtrans_len(p, clen, 0, false);
                        p = p.add(clen as usize);
                    }
                    j += 1;
                }
                if n > 1 && (*yb).y_type == K_MT_LINE_WISE {
                    msg_puts_hl(c"^J".as_ptr(), hl_id, false);
                }
            }
            os_breakcheck();
        }
        i += 1;
    }

    // Display last inserted text.
    let insert = get_last_insert();
    p = insert.data;
    if !p.is_null()
        && (arg.is_null() || !vim_strchr(arg, c_int::from(b'.')).is_null())
        && !got_int
        && !message_filtered(p)
    {
        msg_puts(c"\n  c  \".   ".as_ptr());
        dis_msg(p, true);
    }

    // Display last command line.
    if !last_cmdline.is_null()
        && (arg.is_null() || !vim_strchr(arg, c_int::from(b':')).is_null())
        && !got_int
        && !message_filtered(last_cmdline)
    {
        msg_puts(c"\n  c  \":   ".as_ptr());
        dis_msg(last_cmdline, false);
    }

    // Display current file name.
    let b_fname = nvim_al_curbuf_b_fname();
    if !b_fname.is_null()
        && (arg.is_null() || !vim_strchr(arg, c_int::from(b'%')).is_null())
        && !got_int
        && !message_filtered(b_fname)
    {
        msg_puts(c"\n  c  \"%   ".as_ptr());
        dis_msg(b_fname, false);
    }

    // Display alternate file name.
    if (arg.is_null() || !vim_strchr(arg, c_int::from(b'%')).is_null()) && !got_int {
        let mut fname: *mut c_char = std::ptr::null_mut();
        let mut dummy: c_int = 0;
        if rs_buflist_name_nr(0, &raw mut fname, &raw mut dummy) != FAIL && !message_filtered(fname)
        {
            msg_puts(c"\n  c  \"#   ".as_ptr());
            dis_msg(fname, false);
        }
    }

    // Display last search pattern.
    let sp = last_search_pat();
    if !sp.is_null()
        && (arg.is_null() || !vim_strchr(arg, c_int::from(b'/')).is_null())
        && !got_int
        && !message_filtered(sp)
    {
        msg_puts(c"\n  c  \"/   ".as_ptr());
        dis_msg(sp, false);
    }

    // Display last used expression.
    if !expr_line.is_null()
        && (arg.is_null() || !vim_strchr(arg, c_int::from(b'=')).is_null())
        && !got_int
        && !message_filtered(expr_line)
    {
        msg_puts(c"\n  c  \"=   ".as_ptr());
        dis_msg(expr_line, false);
    }

    msg_ext_skip_flush = false;
}

/// Start/stop macro recording into a register.
///
/// Returns FAIL for failure, OK otherwise.
///
/// # Safety
///
/// Reads/writes global state and calls C functions.
#[unsafe(export_name = "do_record")]
pub unsafe extern "C" fn rs_do_record(c: c_int) -> c_int {
    static mut REGNAME: c_int = 0;

    let retval;

    if reg_recording == 0 {
        // start recording
        // registers 0-9, a-z and " are allowed
        if c < 0 || (!ascii_isalnum(c as u8) && c != c_int::from(b'"')) {
            retval = FAIL;
        } else {
            reg_recording = c;
            // TODO(bfredl): showmode based messaging is currently missing with cmdheight=0
            showmode();
            REGNAME = c;
            retval = OK;

            apply_autocmds(
                EVENT_RECORDINGENTER,
                std::ptr::null(),
                std::ptr::null(),
                false,
                curbuf,
            );
        }
    } else {
        // stop recording
        let mut save_v_event = std::mem::MaybeUninit::<[u8; SAVE_V_EVENT_SIZE]>::uninit();
        let dict = get_v_event(save_v_event.as_mut_ptr() as *mut c_void);

        // The recorded text contents.
        let p = get_recorded();
        if !p.is_null() {
            // Remove escaping for K_SPECIAL in multi-byte chars.
            vim_unescape_ks(p);
            tv_dict_add_str(dict, c"regcontents".as_ptr(), 11, p);
        }

        // Name of requested register.
        let mut buf = [0u8; NUMBUFLEN + 2];
        buf[0] = REGNAME as u8;
        buf[1] = 0;
        tv_dict_add_str(dict, c"regname".as_ptr(), 7, buf.as_ptr() as *const c_char);
        tv_dict_set_keys_readonly(dict);

        apply_autocmds(
            EVENT_RECORDINGLEAVE,
            std::ptr::null(),
            std::ptr::null(),
            false,
            curbuf,
        );
        restore_v_event(dict, save_v_event.as_mut_ptr() as *mut c_void);
        reg_recorded = reg_recording;
        reg_recording = 0;
        if p_ch == 0 || ui_has(K_UI_MESSAGES) {
            showmode();
        } else {
            msg(c"".as_ptr(), 0);
        }
        if p.is_null() {
            retval = FAIL;
        } else {
            // We don't want to change the default register here, so save and
            // restore the current register name.
            let old_y_previous = y_previous;
            retval = rs_stuff_yank(REGNAME, p);
            y_previous = old_y_previous;
        }
    }
    retval
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
