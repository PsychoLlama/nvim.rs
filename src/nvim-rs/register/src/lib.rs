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

/// Position in file or buffer, matching C's `pos_T` exactly (12 bytes).
///
/// Layout matches `pos_defs.h`:
/// ```c
/// typedef struct { linenr_T lnum; colnr_T col; colnr_T coladd; } pos_T;
/// ```
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

const _: () = assert!(std::mem::size_of::<PosT>() == 12);

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

    // Global variables (Phase 1)
    static mut got_int: bool;

    // Accessors for get_spec_reg (Phase 1)
    fn check_fname() -> c_int;
    fn rs_getaltfname(errmsg: bool) -> *mut c_char;
    fn file_name_at_cursor(options: c_int, count: c_int, file_lnum: *mut c_int) -> *mut c_char;
    fn nvim_register_get_curbuf_fname() -> *mut c_char;
    fn nvim_register_ml_get_buf_curwin_lnum() -> *mut c_char;
    fn nvim_emsg_nolastcmd();
    fn nvim_emsg_noprevre();
    fn nvim_emsg_noinstext();
    fn rs_find_ident_under_cursor(text: *mut *mut c_char, find_type: c_int) -> usize;

    // Accessors for insert_reg (Phase 1)
    fn stuffescaped(arg: *const c_char, literally: bool);
    fn stuff_inserted(c: c_int, count: i64, no_esc: bool) -> c_int;
    fn stuffcharReadbuff(c: c_int);
    fn AppendCharToRedobuff(c: c_int);
    fn del_chars(count: c_int, fixpos: bool) -> c_int;
    fn mb_charlen(str: *const c_char) -> c_int;
    fn oneright() -> c_int;
    fn u_save_cursor() -> c_int;
    fn nvim_register_get_State() -> c_int;
    fn nvim_register_get_curwin_cursor(pos: *mut PosT);
    fn nvim_register_set_curwin_cursor(pos: *const PosT);
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
const CTRL_L: c_int = 12;
const CTRL_P: c_int = 16;
const CTRL_R: c_int = 18;
#[allow(dead_code)]
const CTRL_U: c_int = 21;
const CTRL_W: c_int = 23;

/// FNAME_MESS|FNAME_HYP|FNAME_EXP constants from file_search.h.
const FNAME_MESS: c_int = 1;
const FNAME_EXP: c_int = 2;
const FNAME_HYP: c_int = 4;

/// FIND_IDENT|FIND_STRING constants from normal.h.
const FIND_IDENT: c_int = 1;
const FIND_STRING: c_int = 2;

/// REPLACE_FLAG from state_defs.h.
const REPLACE_FLAG: c_int = 0x100;

/// Direction constants from vim_defs.h.
const BACKWARD: c_int = -1;
const FORWARD: c_int = 1;

/// PUT_CURSEND flag from register_defs.h.
const PUT_CURSEND: c_int = 2;

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
// BlockDef struct – matches C's `struct block_def` (register_defs.h), 64 bytes.
// ---------------------------------------------------------------------------

/// Blockwise operation data, matching C's `struct block_def` (64 bytes on 64-bit).
///
/// Layout (verified by static assertion below):
/// ```c
/// struct block_def {
///   int startspaces;     // 0
///   int endspaces;       // 4
///   int textlen;         // 8
///   // 4 bytes padding
///   char *textstart;     // 16
///   colnr_T textcol;     // 24
///   colnr_T start_vcol;  // 28
///   colnr_T end_vcol;    // 32
///   int is_short;        // 36
///   int is_MAX;          // 40
///   int is_oneChar;      // 44
///   int pre_whitesp;     // 48
///   int pre_whitesp_c;   // 52
///   colnr_T end_char_vcols;    // 56
///   colnr_T start_char_vcols;  // 60
/// };  // total = 64
/// ```
#[repr(C)]
struct BlockDef {
    startspaces: c_int,
    endspaces: c_int,
    textlen: c_int,
    _pad: c_int,
    textstart: *mut c_char,
    textcol: c_int,
    start_vcol: c_int,
    end_vcol: c_int,
    is_short: c_int,
    is_max: c_int,
    is_one_char: c_int,
    pre_whitesp: c_int,
    pre_whitesp_c: c_int,
    end_char_vcols: c_int,
    start_char_vcols: c_int,
}

const _: () = assert!(std::mem::size_of::<BlockDef>() == 64);

impl BlockDef {
    fn zeroed() -> Self {
        Self {
            startspaces: 0,
            endspaces: 0,
            textlen: 0,
            _pad: 0,
            textstart: std::ptr::null_mut(),
            textcol: 0,
            start_vcol: 0,
            end_vcol: 0,
            is_short: 0,
            is_max: 0,
            is_one_char: 0,
            pre_whitesp: 0,
            pre_whitesp_c: 0,
            end_char_vcols: 0,
            start_char_vcols: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Yank group FFI declarations
// ---------------------------------------------------------------------------

extern "C" {
    // oparg_T accessors
    fn nvim_oap_get_motion_type(oap: *mut c_void) -> c_int;
    fn nvim_oap_get_start(oap: *mut c_void, pos: *mut PosT);
    fn nvim_oap_get_end(oap: *mut c_void, pos: *mut PosT);
    fn nvim_oap_get_inclusive(oap: *mut c_void) -> bool;
    fn nvim_oap_get_is_VIsual(oap: *mut c_void) -> bool;
    fn nvim_oap_get_line_count(oap: *mut c_void) -> c_int;
    fn nvim_oap_get_start_vcol(oap: *mut c_void) -> c_int;
    fn nvim_oap_get_end_vcol(oap: *mut c_void) -> c_int;
    fn nvim_oap_get_regname(oap: *mut c_void) -> c_int;
    fn nvim_oap_get_excl_tr_ws(oap: *mut c_void) -> bool;
    fn nvim_oap_get_op_type(oap: *mut c_void) -> c_int;

    // Block ops
    fn block_prep(oap: *mut c_void, bd: *mut BlockDef, lnum: i32, cflag: bool);
    fn charwise_block_prep(start: PosT, end: PosT, bd: *mut BlockDef, lnum: i32, inclusive: bool);

    // memline
    fn ml_get(lnum: i32) -> *mut c_char;
    fn ml_get_len(lnum: i32) -> c_int;

    // String construction
    fn cbuf_to_string(buf: *const c_char, len: usize) -> NvimString;
    fn nvim_register_cbuf_as_string(buf: *mut c_char, len: usize) -> NvimString;

    // Display
    fn update_topline(wp: *mut c_void);
    fn update_screen();

    // Autocmd helpers
    fn has_event(event: c_int) -> c_int;
    fn tv_dict_add_bool(d: *mut c_void, key: *const c_char, key_len: usize, val: c_int) -> c_int;
    fn tv_dict_add_list(
        d: *mut c_void,
        key: *const c_char,
        key_len: usize,
        list: *mut c_void,
    ) -> c_int;
    fn nvim_register_tv_list_set_lock_fixed(list: *mut c_void);
    fn get_op_char(optype: c_int) -> c_int;

    // Multibyte / ascii for yank_copy_line
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // op_yank helpers
    fn beep_flush();

    // Options (Phase 2)
    fn nvim_get_p_sel() -> *const c_char;
    fn nvim_get_p_report() -> i64;
    fn nvim_register_p_cpo_has_regappend() -> bool;
    fn nvim_register_cmod_lockmarks() -> bool;
    fn nvim_must_redraw() -> c_int;

    // curwin/curbuf accessors (Phase 2)
    fn nvim_register_get_curwin_curswant() -> c_int;
    fn nvim_register_curbuf_set_op_start(pos: *const PosT);
    fn nvim_register_curbuf_set_op_end(pos: *const PosT);
    fn nvim_register_curbuf_set_op_start_col(col: c_int);
    fn nvim_register_curbuf_set_op_end_col(col: c_int);
    fn nvim_register_curbuf_decl_op_end();

    // Yank message (NGETTEXT wrapper)
    fn nvim_register_yank_msg(yanklines: usize, namebuf: *const c_char, is_block: bool);
    fn nvim_register_yank_namebuf(regname: c_int, buf: *mut c_char, bufsz: usize);

    // textlock inc/dec
    fn nvim_inc_textlock();
    fn nvim_dec_textlock();

    // curwin ptr (for update_topline)
    fn nvim_register_get_curwin() -> *mut c_void;
}

/// EVENT_TEXTYANKPOST value (from autocmd/src/event.rs: TextYankPost = 125).
const EVENT_TEXTYANKPOST: c_int = 125;

/// kBoolVarTrue/kBoolVarFalse values from eval/typval_defs.h.
const K_BOOL_VAR_FALSE: c_int = 0;
const K_BOOL_VAR_TRUE: c_int = 1;

// MAXCOL constant from pos_defs.h.
const MAXCOL: c_int = 0x7fff_ffff_u32 as c_int;

// ---------------------------------------------------------------------------
// get_spec_reg and insert_reg (Phase 1 migration)
// ---------------------------------------------------------------------------

/// If `regname` is a special register, return true and set *argp to its value.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(export_name = "get_spec_reg")]
pub unsafe extern "C" fn rs_get_spec_reg(
    regname: c_int,
    argp: *mut *mut c_char,
    allocated: *mut bool,
    errmsg: bool,
) -> bool {
    *argp = std::ptr::null_mut();
    *allocated = false;

    match regname {
        r if r == c_int::from(b'%') => {
            // file name
            if errmsg {
                check_fname(); // gives emsg if not set
            }
            *argp = nvim_register_get_curbuf_fname();
            true
        }
        r if r == c_int::from(b'#') => {
            // alternate file name
            *argp = rs_getaltfname(errmsg); // may give emsg if not set
            true
        }
        r if r == c_int::from(b'=') => {
            // result of expression
            *argp = rs_get_expr_line();
            *allocated = true;
            true
        }
        r if r == c_int::from(b':') => {
            // last command line
            if last_cmdline.is_null() && errmsg {
                nvim_emsg_nolastcmd();
            }
            *argp = last_cmdline;
            true
        }
        r if r == c_int::from(b'/') => {
            // last search-pattern
            let sp = last_search_pat();
            if sp.is_null() && errmsg {
                nvim_emsg_noprevre();
            }
            *argp = sp as *mut c_char;
            true
        }
        r if r == c_int::from(b'.') => {
            // last inserted text
            *argp = get_last_insert_save();
            *allocated = true;
            if (*argp).is_null() && errmsg {
                nvim_emsg_noinstext();
            }
            true
        }
        r if r == CTRL_F || r == CTRL_P => {
            // Filename/path under cursor
            if !errmsg {
                return false;
            }
            let opts = FNAME_MESS | FNAME_HYP | if regname == CTRL_P { FNAME_EXP } else { 0 };
            *argp = file_name_at_cursor(opts, 1, std::ptr::null_mut());
            *allocated = true;
            true
        }
        r if r == CTRL_W || r == CTRL_A => {
            // word/WORD under cursor
            if !errmsg {
                return false;
            }
            let find_type = if regname == CTRL_W {
                FIND_IDENT | FIND_STRING
            } else {
                FIND_STRING
            };
            let cnt = rs_find_ident_under_cursor(argp, find_type);
            *argp = if cnt > 0 {
                xmemdupz(*argp as *const c_void, cnt) as *mut c_char
            } else {
                std::ptr::null_mut()
            };
            *allocated = true;
            true
        }
        r if r == CTRL_L => {
            // Line under cursor
            if !errmsg {
                return false;
            }
            *argp = nvim_register_ml_get_buf_curwin_lnum();
            true
        }
        r if r == c_int::from(b'_') => {
            // black hole: always empty
            *argp = c"".as_ptr() as *mut c_char;
            true
        }
        _ => false,
    }
}

/// Insert a yank register: copy it into the Read buffer.
/// Used by CTRL-R command and middle mouse button in insert mode.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(export_name = "insert_reg")]
pub unsafe extern "C" fn rs_insert_reg(
    regname: c_int,
    reg: *mut YankReg,
    literally_arg: bool,
) -> c_int {
    let mut retval = OK;
    let literally = literally_arg || rs_is_literal_register(regname) != 0;

    // It is possible to get into an endless loop by having CTRL-R a in
    // register a and then, in insert mode, doing CTRL-R a.
    // If you hit CTRL-C, the loop will be broken here.
    os_breakcheck();
    if got_int {
        return FAIL;
    }

    // check for valid regname
    if regname != NUL && !rs_valid_yank_reg(regname, false) {
        return FAIL;
    }

    let mut arg: *mut c_char = std::ptr::null_mut();
    let mut allocated = false;

    if regname == c_int::from(b'.') {
        // Insert last inserted text.
        retval = stuff_inserted(NUL, 1, true);
    } else if rs_get_spec_reg(regname, &raw mut arg, &raw mut allocated, true) {
        if arg.is_null() {
            return FAIL;
        }
        stuffescaped(arg, literally);
        if allocated {
            xfree(arg as *mut c_void);
        }
    } else {
        // Name or number register.
        let reg = if reg.is_null() {
            rs_get_yank_register(regname, 0 /* YREG_PASTE */)
        } else {
            reg
        };
        if (*reg).y_array.is_null() {
            retval = FAIL;
        } else {
            let size = (*reg).y_size;
            for i in 0..size {
                if regname == c_int::from(b'-') && (*reg).y_type == K_MT_CHAR_WISE {
                    let mut dir = BACKWARD;
                    let state = nvim_register_get_State();
                    if (state & REPLACE_FLAG) != 0 {
                        let mut curpos = PosT::default();
                        if u_save_cursor() == FAIL {
                            return FAIL;
                        }
                        del_chars(mb_charlen((*(*reg).y_array).data), true);
                        nvim_register_get_curwin_cursor(&raw mut curpos);
                        if oneright() == FAIL {
                            // hit end of line, need to put forward
                            dir = FORWARD;
                        }
                        nvim_register_set_curwin_cursor(&raw const curpos);
                    }
                    AppendCharToRedobuff(CTRL_R);
                    AppendCharToRedobuff(regname);
                    rs_do_put(regname, std::ptr::null_mut(), dir, 1, PUT_CURSEND);
                } else {
                    stuffescaped((*(*reg).y_array.add(i)).data, literally);
                    // Insert a newline between lines and after last line if
                    // y_type is kMTLineWise.
                    if (*reg).y_type == K_MT_LINE_WISE || i < size - 1 {
                        stuffcharReadbuff(c_int::from(b'\n'));
                    }
                }
            }
        }
    }

    retval
}

// ---------------------------------------------------------------------------
// Phase 2: Yank group – yank_copy_line, op_yank_reg, do_autocmd_textyankpost, op_yank
// ---------------------------------------------------------------------------

/// Copy a block range line into a register, handling trailing space exclusion.
///
/// # Safety
///
/// `reg` and `bd` must be valid. `reg->y_array[y_idx]` must be writable.
unsafe fn rs_yank_copy_line(
    reg: *mut YankReg,
    bd: *mut BlockDef,
    y_idx: usize,
    exclude_trailing_space: bool,
) {
    if exclude_trailing_space {
        (*bd).endspaces = 0;
    }
    let size = (*bd).startspaces + (*bd).endspaces + (*bd).textlen;
    assert!(size >= 0);
    let pnew_start = xmallocz(size as usize);
    (*(*reg).y_array.add(y_idx)).data = pnew_start;

    let mut pnew = pnew_start;
    // Fill startspaces.
    std::ptr::write_bytes(pnew, b' ', (*bd).startspaces as usize);
    pnew = pnew.add((*bd).startspaces as usize);
    // Copy text.
    std::ptr::copy((*bd).textstart, pnew, (*bd).textlen as usize);
    pnew = pnew.add((*bd).textlen as usize);
    // Fill endspaces.
    std::ptr::write_bytes(pnew, b' ', (*bd).endspaces as usize);
    pnew = pnew.add((*bd).endspaces as usize);

    if exclude_trailing_space {
        let mut s = (*bd).textlen + (*bd).endspaces;
        while s > 0
            && rs_ascii_iswhite(c_int::from(*(*bd).textstart.add(s as usize - 1) as u8)) != 0
        {
            s -= utf_head_off((*bd).textstart, (*bd).textstart.add(s as usize - 1)) + 1;
            pnew = pnew.sub(1);
        }
    }
    *pnew = 0; // NUL terminator
    (*(*reg).y_array.add(y_idx)).size = pnew.offset_from(pnew_start) as usize;
}

/// Core yank logic: fills register from oap, handles append, displays message, sets marks.
///
/// # Safety
///
/// `oap` and `reg` must be valid.
#[unsafe(export_name = "op_yank_reg")]
pub unsafe extern "C" fn rs_op_yank_reg(
    oap: *mut c_void,
    message: bool,
    reg: *mut YankReg,
    append: bool,
) {
    let mut newreg = YankReg::ZERO;
    let mut yank_type = nvim_oap_get_motion_type(oap);

    // Read oap fields.
    let mut start = PosT::default();
    let mut end = PosT::default();
    nvim_oap_get_start(oap, &raw mut start);
    nvim_oap_get_end(oap, &raw mut end);
    let inclusive = nvim_oap_get_inclusive(oap);
    let is_visual = nvim_oap_get_is_VIsual(oap);
    let line_count = nvim_oap_get_line_count(oap);
    let start_vcol = nvim_oap_get_start_vcol(oap);
    let end_vcol = nvim_oap_get_end_vcol(oap);
    let regname = nvim_oap_get_regname(oap);
    let excl_tr_ws = nvim_oap_get_excl_tr_ws(oap);

    let mut yanklines = line_count as usize;
    let mut yankendlnum = end.lnum;

    let curr = reg; // copy of current register
                    // Decide whether to append or free.
    let reg = if append && !(*reg).y_array.is_null() {
        // Append to existing: fill newreg first.
        &raw mut newreg
    } else {
        rs_free_register(reg);
        reg
    };

    // If the cursor was in column 1 before and after the movement, and the
    // operator is not inclusive, the yank is always linewise.
    let p_sel_char = *nvim_get_p_sel() as u8;
    if yank_type == K_MT_CHAR_WISE
        && start.col == 0
        && !inclusive
        && (!is_visual || p_sel_char == b'o')
        && end.col == 0
        && yanklines > 1
    {
        yank_type = K_MT_LINE_WISE;
        yankendlnum -= 1;
        yanklines -= 1;
    }

    (*reg).y_size = yanklines;
    (*reg).y_type = yank_type;
    (*reg).y_width = 0;
    (*reg).y_array = xcalloc(yanklines, std::mem::size_of::<NvimString>()) as *mut NvimString;
    (*reg).additional_data = std::ptr::null_mut();
    (*reg).timestamp = os_time();

    let mut y_idx: usize = 0;
    let mut lnum = start.lnum;

    if yank_type == K_MT_BLOCK_WISE {
        (*reg).y_width = end_vcol - start_vcol;
        let curswant = nvim_register_get_curwin_curswant();
        if curswant == MAXCOL && (*reg).y_width > 0 {
            (*reg).y_width -= 1;
        }
    }

    while lnum <= yankendlnum {
        let mut bd = BlockDef::zeroed();
        match yank_type {
            K_MT_BLOCK_WISE => {
                block_prep(oap, &raw mut bd, lnum, false);
                rs_yank_copy_line(reg, &raw mut bd, y_idx, excl_tr_ws);
            }
            K_MT_LINE_WISE => {
                *(*reg).y_array.add(y_idx) =
                    cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as usize);
            }
            K_MT_CHAR_WISE => {
                charwise_block_prep(start, end, &raw mut bd, lnum, inclusive);
                // make sure bd.textlen is not longer than the text
                let tmp = libc::strlen(bd.textstart) as c_int;
                if tmp < bd.textlen {
                    bd.textlen = tmp;
                }
                rs_yank_copy_line(reg, &raw mut bd, y_idx, false);
            }
            _ => {
                libc::abort();
            }
        }
        lnum += 1;
        y_idx += 1;
    }

    if !std::ptr::eq(curr, reg) {
        // append the new block to the old block
        let new_ptr = xmalloc(std::mem::size_of::<NvimString>() * ((*curr).y_size + (*reg).y_size))
            as *mut NvimString;
        for j in 0..(*curr).y_size {
            *new_ptr.add(j) = *(*curr).y_array.add(j);
        }
        xfree((*curr).y_array as *mut c_void);
        (*curr).y_array = new_ptr;

        if yank_type == K_MT_LINE_WISE {
            // kMTLineWise overrides kMTCharWise and kMTBlockWise
            (*curr).y_type = K_MT_LINE_WISE;
        }

        // Concatenate the last line of the old block with the first line of the new block,
        // unless being Vi compatible.
        let mut concat_start_idx: usize;
        if (*curr).y_type == K_MT_CHAR_WISE && !nvim_register_p_cpo_has_regappend() {
            let j = (*curr).y_size - 1;
            let old_last = &*(*curr).y_array.add(j);
            let new_first = &*(*reg).y_array;
            let pnew = xmalloc(old_last.size + new_first.size + 1) as *mut c_char;
            std::ptr::copy_nonoverlapping(old_last.data, pnew, old_last.size);
            std::ptr::copy_nonoverlapping(new_first.data, pnew.add(old_last.size), new_first.size);
            *pnew.add(old_last.size + new_first.size) = 0;
            xfree(old_last.data as *mut c_void);
            let new_size = old_last.size + new_first.size;
            *(*curr).y_array.add(j) = nvim_register_cbuf_as_string(pnew, new_size);
            // Clear reg->y_array[0].
            if !(*(*reg).y_array).data.is_null() {
                xfree((*(*reg).y_array).data as *mut c_void);
                (*(*reg).y_array).data = std::ptr::null_mut();
                (*(*reg).y_array).size = 0;
            }
            concat_start_idx = 1;
        } else {
            concat_start_idx = 0;
        }
        let mut j = (*curr).y_size;
        while concat_start_idx < (*reg).y_size {
            *(*curr).y_array.add(j) = *(*reg).y_array.add(concat_start_idx);
            j += 1;
            concat_start_idx += 1;
        }
        (*curr).y_size = j;
        xfree((*reg).y_array as *mut c_void);
    }

    if message {
        // Display message about yank?
        if yank_type == K_MT_CHAR_WISE && yanklines == 1 {
            yanklines = 0;
        }
        // Some versions of Vi use ">=" here, some don't...
        if yanklines > nvim_get_p_report() as usize {
            // Fill name buffer.
            let mut namebuf = [0u8; 100];
            nvim_register_yank_namebuf(regname, namebuf.as_mut_ptr() as *mut c_char, 100);

            // Redisplay now, so message is not deleted.
            update_topline(nvim_register_get_curwin());
            if nvim_must_redraw() != 0 {
                update_screen();
            }
            nvim_register_yank_msg(
                yanklines,
                namebuf.as_ptr() as *const c_char,
                yank_type == K_MT_BLOCK_WISE,
            );
        }
    }

    if !nvim_register_cmod_lockmarks() {
        // Set "'[" and "']" marks.
        nvim_register_curbuf_set_op_start(&raw const start);
        nvim_register_curbuf_set_op_end(&raw const end);
        if yank_type == K_MT_LINE_WISE {
            nvim_register_curbuf_set_op_start_col(0);
            nvim_register_curbuf_set_op_end_col(MAXCOL);
        }
        if yank_type != K_MT_LINE_WISE && !inclusive {
            // Exclude the end position.
            nvim_register_curbuf_decl_op_end();
        }
    }
}

/// Execute autocommands for TextYankPost.
///
/// # Safety
///
/// `oap` and `reg` must be valid.
#[unsafe(export_name = "do_autocmd_textyankpost")]
pub unsafe extern "C" fn rs_do_autocmd_textyankpost(oap: *mut c_void, reg: *mut YankReg) {
    static mut RECURSIVE: bool = false;

    if RECURSIVE || has_event(EVENT_TEXTYANKPOST) == 0 {
        return;
    }

    RECURSIVE = true;

    let mut save_v_event = std::mem::MaybeUninit::<[u8; SAVE_V_EVENT_SIZE]>::uninit();
    let dict = get_v_event(save_v_event.as_mut_ptr() as *mut c_void);

    // The yanked text contents.
    let list = tv_list_alloc((*reg).y_size as isize);
    for i in 0..(*reg).y_size {
        tv_list_append_string(list, (*(*reg).y_array.add(i)).data, -1);
    }
    nvim_register_tv_list_set_lock_fixed(list);
    tv_dict_add_list(dict, c"regcontents".as_ptr(), 11, list);

    // Register type.
    let mut buf = [0u8; NUMBUFLEN + 2];
    rs_format_reg_type(
        (*reg).y_type,
        (*reg).y_width,
        buf.as_mut_ptr() as *mut c_char,
        NUMBUFLEN + 2,
    );
    tv_dict_add_str(dict, c"regtype".as_ptr(), 7, buf.as_ptr() as *const c_char);

    // Name of requested register, or empty string for unnamed operation.
    let regname = nvim_oap_get_regname(oap);
    buf[0] = regname as u8;
    buf[1] = 0;
    tv_dict_add_str(dict, c"regname".as_ptr(), 7, buf.as_ptr() as *const c_char);

    // Motion type: inclusive or exclusive.
    let inclusive = nvim_oap_get_inclusive(oap);
    tv_dict_add_bool(
        dict,
        c"inclusive".as_ptr(),
        9,
        if inclusive {
            K_BOOL_VAR_TRUE
        } else {
            K_BOOL_VAR_FALSE
        },
    );

    // Kind of operation: yank, delete, change.
    let op_type = nvim_oap_get_op_type(oap);
    buf[0] = get_op_char(op_type) as u8;
    buf[1] = 0;
    tv_dict_add_str(dict, c"operator".as_ptr(), 8, buf.as_ptr() as *const c_char);

    // Selection type: visual or not.
    let is_visual = nvim_oap_get_is_VIsual(oap);
    tv_dict_add_bool(
        dict,
        c"visual".as_ptr(),
        6,
        if is_visual {
            K_BOOL_VAR_TRUE
        } else {
            K_BOOL_VAR_FALSE
        },
    );

    tv_dict_set_keys_readonly(dict);
    nvim_inc_textlock();
    apply_autocmds(
        EVENT_TEXTYANKPOST,
        std::ptr::null(),
        std::ptr::null(),
        false,
        curbuf,
    );
    nvim_dec_textlock();
    restore_v_event(dict, save_v_event.as_mut_ptr() as *mut c_void);

    RECURSIVE = false;
}

/// Thin wrapper: validates register, calls op_yank_reg, clipboard, autocmd.
///
/// # Safety
///
/// `oap` must be valid.
#[unsafe(export_name = "op_yank")]
pub unsafe extern "C" fn rs_op_yank(oap: *mut c_void, message: bool) -> bool {
    let regname = nvim_oap_get_regname(oap);

    // check for read-only register
    if regname != NUL && !rs_valid_yank_reg(regname, true) {
        beep_flush();
        return false;
    }
    if regname == c_int::from(b'_') {
        return true; // black hole: nothing to do
    }

    let reg = rs_get_yank_register(regname, 1 /* YREG_YANK */);
    rs_op_yank_reg(oap, message, reg, rs_is_append_register(regname) != 0);
    set_clipboard(regname, reg);
    rs_do_autocmd_textyankpost(oap, reg);
    true
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
    if rs_get_spec_reg(regname, &raw mut retval, &raw mut allocated, false) {
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
// do_put FFI declarations and implementation (Phase 1)
// ---------------------------------------------------------------------------

/// Mirror of C `CharSize` from `plines.h`: `{int width; int head;}`.
#[repr(C)]
struct DpCharSize {
    width: c_int,
    head: c_int,
}

/// Mirror of C `CharInfo` from `mbyte_defs.h`: `{int32_t value; int len;}`.
#[repr(C)]
#[derive(Clone, Copy)]
struct DpCharInfo {
    value: i32,
    len: c_int,
}

/// Mirror of C `StrCharInfo` from `mbyte_defs.h`: `{char *ptr; CharInfo chr;}`.
#[repr(C)]
#[derive(Clone, Copy)]
struct DpStrCharInfo {
    ptr: *mut c_char,
    chr: DpCharInfo,
}

/// Opaque stack buffer for `CharsizeArg` (320 bytes, align 8 -- same as cursor crate).
#[repr(C, align(8))]
struct DpCharsizeArgBuf([u8; 320]);

extern "C" {
    // --- charsize FFI (same functions as cursor crate uses) ---
    fn charsize_fast(
        csarg: *mut c_void,
        cur: *const c_char,
        vcol: c_int,
        cur_char: i32,
    ) -> DpCharSize;
    fn charsize_regular(
        csarg: *mut c_void,
        cur: *const c_char,
        vcol: c_int,
        cur_char: i32,
    ) -> DpCharSize;
    fn utfc_next_impl(cur: DpStrCharInfo) -> DpStrCharInfo;
    fn utf_ptr2CharInfo_impl(p: *const u8, len: usize) -> i32;
    static utf8len_tab: [u8; 256];

    // --- do_put accessor wrappers (added to register.c) ---
    fn nvim_dp_get_op_start(pos: *mut PosT);
    fn nvim_dp_get_op_end(pos: *mut PosT);
    fn nvim_dp_set_op_start_lnum(lnum: c_int);
    fn nvim_dp_set_op_end_lnum(lnum: c_int);
    fn nvim_dp_set_op_end_col(col: c_int);
    fn nvim_dp_set_op_end_coladd(coladd: c_int);
    fn nvim_dp_set_cursor_to_b_visual_vi_end();
    fn nvim_dp_get_ml_line_count() -> c_int;
    fn nvim_dp_getvcol_cursor(start: *mut c_int, endcol2: *mut c_int);
    fn nvim_dp_getvcol_cursor_end_only(col: *mut c_int);
    fn nvim_dp_getvcol_pos(lnum: c_int, col: c_int, coladd: c_int, vcol_mid: *mut c_int) -> c_int;
    fn nvim_dp_getvpos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int, wcol: c_int)
        -> c_int;
    fn nvim_dp_buf_updates_send_changes(lnum: c_int, num_added: i64, num_removed: i64);
    fn nvim_dp_changed_lines(lnum: c_int, col: c_int, lnume: c_int, xtra: c_int);
    fn nvim_dp_changed_bytes(lnum: c_int, col: c_int);
    fn nvim_dp_mark_adjust(line1: c_int, nr_lines: c_int, kind: c_int);
    fn nvim_dp_extmark_splice(
        start_row: c_int,
        start_col: c_int,
        old_row: c_int,
        old_col: c_int,
        new_row: c_int,
        new_col: c_int,
        totsize: c_long,
        kind: c_int,
    );
    fn nvim_dp_extmark_splice_cols(
        start_row: c_int,
        start_col: c_int,
        old_col: c_int,
        new_col: c_int,
        lines_appended: c_int,
        kind: c_int,
    );
    fn nvim_dp_terminal_paste(count: c_int, y_array: *mut NvimString, y_size: usize);
    fn nvim_dp_get_b_p_ts() -> i64;
    fn nvim_dp_get_b_p_vts_array() -> *const c_int;
    fn nvim_dp_tabstop_padding(col: c_int, ts: i64, vts: *const c_int) -> c_int;
    fn nvim_dp_set_indent(size: c_int) -> bool;
    fn nvim_dp_get_indent() -> c_int;
    fn nvim_dp_preprocs_left() -> bool;
    fn nvim_dp_beginline();
    fn u_save(top: c_int, bot: c_int) -> c_int;
    fn nvim_dp_hasFolding_backward(lnum: *mut c_int);
    fn nvim_dp_hasFolding_forward(lnum: *mut c_int);
    fn nvim_dp_buf_is_empty() -> bool;
    fn nvim_dp_get_cursor_line_len() -> c_int;
    fn nvim_dp_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_dp_changed_cline_bef_curs();
    fn nvim_dp_invalidate_botline();
    fn nvim_dp_msgmore(n: c_int);
    fn nvim_dp_gchar_cursor() -> c_int;
    fn nvim_dp_get_cursor_lnum() -> c_int;
    fn nvim_dp_set_cursor_lnum(lnum: c_int);
    fn nvim_dp_get_cursor_col() -> c_int;
    fn nvim_dp_set_cursor_col(col: c_int);
    fn nvim_dp_set_cursor_coladd(coladd: c_int);
    fn nvim_dp_get_cursor_coladd() -> c_int;
    fn nvim_dp_set_curswant();
    fn nvim_dp_get_cursor(pos: *mut PosT);
    fn nvim_dp_set_cursor(pos: *const PosT);
    fn ml_append(lnum: c_int, line: *mut c_char, len: c_int, newfile: bool) -> c_int;
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: bool) -> c_int;
    fn nvim_dp_get_b_visual_vi_start_lnum() -> c_int;
    fn nvim_dp_get_b_visual_vi_end_lnum() -> c_int;
    static mut VIsual_mode: c_int;
    fn nvim_dp_init_charsize_arg(csarg: *mut c_void, lnum: c_int, line: *const c_char) -> bool;
    fn nvim_dp_init_charsize_arg_lnum0(csarg: *mut c_void, line: *const c_char) -> bool;
    fn nvim_dp_get_op_start_lnum() -> c_int;
    fn nvim_dp_get_e_resulting_text_too_long() -> *const c_char;
    fn nvim_dp_op_end_col_add(delta: c_int);
    fn nvim_dp_adjust_cursor_eol();
    fn nvim_dp_getviscol() -> c_int;
    fn nvim_dp_coladvance_force(viscol: c_int) -> c_int;
    fn nvim_dp_get_ve_flags() -> c_uint;
    fn nvim_dp_get_cursor_pos_ptr() -> *mut c_char;
    fn nvim_dp_get_cursor_pos_len() -> c_int;
    fn nvim_dp_semsg_E353(regname: c_int);

    // already declared above but need here too
    // fn xfree -- already declared
    // fn xmalloc -- already declared
    // fn u_save_cursor -- already declared
    // fn emsg -- already declared
    // fn semsg -- already declared
}

use std::ffi::c_long;
use std::ffi::c_uint;

/// PUT_* flag constants from `register_defs.h`.
const PUT_FIXINDENT: c_int = 1;
// PUT_CURSEND = 2 already defined above
const PUT_CURSLINE: c_int = 4;
const PUT_LINE: c_int = 8;
const PUT_LINE_SPLIT: c_int = 16;
const PUT_LINE_FORWARD: c_int = 32;
const PUT_BLOCK_INNER: c_int = 64;

/// MotionType constants (matches K_MT_* and kMT*).
const K_MT_CHAR_WISE_V: c_int = K_MT_CHAR_WISE;
const K_MT_LINE_WISE_V: c_int = K_MT_LINE_WISE;
const K_MT_BLOCK_WISE_V: c_int = K_MT_BLOCK_WISE;

/// kOptVeFlagAll from option_vars.generated.h.
const K_OPT_VE_FLAG_ALL: c_uint = 0x04;
/// kOptVeFlagOnemore from option_vars.generated.h.
const K_OPT_VE_FLAG_ONEMORE: c_uint = 0x08;

/// kExtmarkUndo = 1, kExtmarkNOOP = 0.
const K_EXTMARK_UNDO: c_int = 1;
const K_EXTMARK_NOOP: c_int = 0;

#[allow(dead_code)]
/// MAXLNUM from pos_defs.h.
const MAXLNUM: c_int = 0x7fff_ffff_u32 as c_int;

/// TAB character.
const TAB: c_int = 9;

/// Dispatch to `charsize_fast` or `charsize_regular`.
#[inline]
unsafe fn dp_win_charsize(
    cstype: bool,
    vcol: c_int,
    ptr: *const c_char,
    chr: i32,
    csarg: *mut c_void,
) -> DpCharSize {
    if cstype {
        charsize_regular(csarg, ptr, vcol, chr)
    } else {
        charsize_fast(csarg, ptr, vcol, chr)
    }
}

/// Inline `utf_ptr2StrCharInfo`: construct `DpStrCharInfo` for the character at `ptr`.
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
unsafe fn dp_utf_ptr2str_char_info(ptr: *mut c_char) -> DpStrCharInfo {
    let p = ptr.cast::<u8>();
    let first = *p;
    if first < 0x80 {
        DpStrCharInfo {
            ptr,
            chr: DpCharInfo {
                value: i32::from(first),
                len: 1,
            },
        }
    } else {
        let len = utf8len_tab[first as usize] as usize;
        let code_point = utf_ptr2CharInfo_impl(p, len);
        let (code_point, len) = if code_point < 0 {
            (code_point, 1)
        } else {
            (code_point, len as c_int)
        };
        DpStrCharInfo {
            ptr,
            chr: DpCharInfo {
                value: code_point,
                len,
            },
        }
    }
}

/// Inline `utfc_next`: advance to the next character in a string.
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn dp_utfc_next(cur: DpStrCharInfo) -> DpStrCharInfo {
    let first = *cur.ptr as u8;
    if first < 0x80 {
        let next_ptr = cur.ptr.add(1);
        let next_first = *next_ptr as u8;
        if next_first < 0x80 {
            return DpStrCharInfo {
                ptr: next_ptr,
                chr: DpCharInfo {
                    value: i32::from(next_first),
                    len: 1,
                },
            };
        }
    }
    utfc_next_impl(cur)
}

/// Read `curwin->w_cursor` and add `delta` to its `col` field, then write it back.
///
/// Replaces the C `nvim_dp_cursor_col_add` wrapper.
#[inline]
unsafe fn dp_cursor_col_add(delta: c_int) {
    let mut pos = PosT::default();
    nvim_dp_get_cursor(&raw mut pos);
    pos.col += delta;
    nvim_dp_set_cursor(&raw const pos);
}

/// Copy `curwin->w_cursor` into `curbuf->b_op_start`.
///
/// Replaces the C `nvim_dp_set_op_start_cursor` wrapper.
#[inline]
unsafe fn dp_set_op_start_cursor() {
    let mut pos = PosT::default();
    nvim_dp_get_cursor(&raw mut pos);
    nvim_register_curbuf_set_op_start(&raw const pos);
}

/// Copy `curwin->w_cursor` into `curbuf->b_op_end`.
///
/// Replaces the C `nvim_dp_set_op_end_cursor` wrapper.
#[inline]
unsafe fn dp_set_op_end_cursor() {
    let mut pos = PosT::default();
    nvim_dp_get_cursor(&raw mut pos);
    nvim_register_curbuf_set_op_end(&raw const pos);
}

/// Put contents of register `regname` into the text.
///
/// Caller must check `regname` to be valid.
///
/// # Safety
///
/// Reads/writes global Neovim state via FFI.
#[unsafe(export_name = "do_put")]
#[allow(unused_assignments, unused_mut, dead_code)]
pub unsafe extern "C" fn rs_do_put(
    regname: c_int,
    reg: *mut YankReg,
    dir: c_int,
    count: c_int,
    flags: c_int,
) {
    let mut totlen: usize = 0;
    let mut lnum: c_int = 0;
    let mut y_type: c_int;
    let mut y_size: usize;
    let mut y_width: c_int = 0;
    let mut vcol: c_int = 0;
    let mut y_array: *mut NvimString = std::ptr::null_mut();
    let mut nr_lines: c_int = 0;
    let mut allocated = false;
    let mut orig_start = PosT::default();
    let mut orig_end = PosT::default();
    nvim_dp_get_op_start(&raw mut orig_start);
    nvim_dp_get_op_end(&raw mut orig_end);
    let cur_ve_flags = nvim_dp_get_ve_flags();

    // default for '[ and '] marks
    dp_set_op_start_cursor();
    dp_set_op_end_cursor();

    // Using inserted text works differently, because the register includes
    // special characters (newlines, etc.).
    if regname == c_int::from(b'.') && reg.is_null() {
        let visual_active = VIsual_active;
        let non_linewise_vis = visual_active && VIsual_mode != c_int::from(b'V');

        // PUT_LINE has special handling below which means we use 'i' to start.
        let command_start_char: c_int = if non_linewise_vis {
            c_int::from(b'c')
        } else if flags & PUT_LINE != 0 {
            c_int::from(b'i')
        } else if dir == FORWARD {
            c_int::from(b'a')
        } else {
            c_int::from(b'i')
        };

        // To avoid 'autoindent' on linewise puts, create a new line with `:put _`.
        if flags & PUT_LINE != 0 {
            rs_do_put(c_int::from(b'_'), std::ptr::null_mut(), dir, 1, PUT_LINE);
        }

        // If given a count when putting linewise, we stuff the readbuf with the
        // dot register 'count' times split by newlines.
        if flags & PUT_LINE != 0 {
            stuffcharReadbuff(command_start_char);
            let mut cnt = count;
            while cnt > 0 {
                stuff_inserted(NUL, 1, cnt != 1);
                if cnt != 1 {
                    stuffReadbuff(c"\n ".as_ptr());
                    stuffcharReadbuff(CTRL_U);
                }
                cnt -= 1;
            }
        } else {
            stuff_inserted(command_start_char, count as i64, false);
        }

        // Putting the text is done later, so can't move the cursor to the next
        // character. Simulate it with motion commands after the insert.
        if flags & PUT_CURSEND != 0 {
            if flags & PUT_LINE != 0 {
                stuffReadbuff(c"j0".as_ptr());
            } else {
                let cursor_pos = nvim_dp_get_cursor_pos_ptr();
                let one_past_line = *cursor_pos == 0;
                let mut eol = false;
                if !one_past_line {
                    let len = utfc_ptr2len(cursor_pos);
                    eol = *cursor_pos.add(len as usize) == 0;
                }
                let ve_allows =
                    cur_ve_flags == K_OPT_VE_FLAG_ALL || cur_ve_flags == K_OPT_VE_FLAG_ONEMORE;
                let eof = nvim_dp_get_ml_line_count() == nvim_dp_get_cursor_lnum() && one_past_line;
                if ve_allows || !(eol || eof) {
                    stuffcharReadbuff(c_int::from(b'l'));
                }
            }
        } else if flags & PUT_LINE != 0 {
            stuffReadbuff(c"g'[".as_ptr());
        }

        // So the 'u' command restores cursor position after ".p, save the cursor
        // position now (though not saving any text).
        if command_start_char == c_int::from(b'a') {
            let lnum_now = nvim_dp_get_cursor_lnum();
            if u_save(lnum_now, lnum_now + 1) == FAIL {
                return;
            }
        }
        return;
    }

    // For special registers '%', '#', ':', etc. we have to create a fake yank register.
    let mut insert_string = NvimString::default();
    if reg.is_null()
        && rs_get_spec_reg(
            regname,
            &raw mut insert_string.data,
            &raw mut allocated,
            true,
        )
        && insert_string.data.is_null()
    {
        return;
    }

    // Autocommands may be executed when saving lines for undo. This might
    // make y_array invalid, so we start undo now to avoid that.
    // (but only if not a terminal buffer)
    // We check terminal status separately:
    let is_terminal = nvim_dp_curbuf_is_terminal();
    if !is_terminal {
        let cursor_lnum = nvim_dp_get_cursor_lnum();
        if u_save(cursor_lnum, cursor_lnum + 1) == FAIL {
            return;
        }
    }

    if !insert_string.data.is_null() {
        insert_string.size = libc_strlen(insert_string.data);
        y_type = K_MT_CHAR_WISE_V;
        if regname == c_int::from(b'=') {
            // For the = register we need to split the string at NL characters.
            // Loop twice: count the number of lines and save them.
            loop {
                y_size = 0;
                let mut ptr = insert_string.data;
                let mut ptrlen = insert_string.size;
                while !ptr.is_null() {
                    if !y_array.is_null() {
                        (*y_array.add(y_size)).data = ptr;
                    }
                    y_size += 1;
                    let tmp = libc_strchr(ptr, b'\n');
                    if tmp.is_null() {
                        if !y_array.is_null() {
                            (*y_array.add(y_size - 1)).size = ptrlen;
                        }
                    } else {
                        if !y_array.is_null() {
                            *tmp = 0;
                            (*y_array.add(y_size - 1)).size = tmp.offset_from(ptr) as usize;
                            ptrlen -= (*y_array.add(y_size - 1)).size + 1;
                        }
                        let tmp_next = tmp.add(1);
                        // A trailing '\n' makes the register linewise.
                        if *tmp_next == 0 {
                            y_type = K_MT_LINE_WISE_V;
                            break;
                        }
                        ptr = tmp_next;
                        continue;
                    }
                    break;
                }
                if !y_array.is_null() {
                    break;
                }
                y_array = xmalloc(y_size * std::mem::size_of::<NvimString>()) as *mut NvimString;
            }
        } else {
            y_size = 1; // use fake one-line yank register
            y_array = &raw mut insert_string;
        }
    } else {
        // in case of replacing visually selected text
        // the yankreg might already have been saved to avoid
        // just restoring the deleted text.
        let effective_reg = if reg.is_null() {
            rs_get_yank_register(regname, 0 /* YREG_PASTE */)
        } else {
            reg
        };

        y_type = (*effective_reg).y_type;
        y_width = (*effective_reg).y_width;
        y_size = (*effective_reg).y_size;
        y_array = (*effective_reg).y_array;
    }

    if is_terminal {
        nvim_dp_terminal_paste(count, y_array, y_size);
        return;
    }

    // We need a 'cleanup' section -- use a closure that always runs
    // after the main body (to replicate C's 'goto end' cleanup).
    // We use a labeled block approach.
    let mut split_pos: c_int = 0;

    // The 'error' and 'end' labels in C become early returns from this inner block.
    // We collect whether we reached the normal path or an error.
    let _: () = 'do_put_body: {
        if y_type == K_MT_LINE_WISE_V {
            if flags & PUT_LINE_SPLIT != 0 {
                // "p" or "P" in Visual mode: split the lines to put the text in between.
                if u_save_cursor() == FAIL {
                    break 'do_put_body;
                }
                let curline = nvim_dp_get_cursor_line_ptr();
                let p_orig = nvim_dp_get_cursor_pos_ptr();
                let mut p = p_orig;
                let plen = nvim_dp_get_cursor_pos_len() as usize;
                if dir == FORWARD && *p != 0 {
                    // MB_PTR_ADV
                    let adv = utfc_ptr2len(p);
                    p = p.add(adv as usize);
                }
                // split_pos = p - curline
                split_pos = p.offset_from(curline) as c_int;

                let part_len = plen - p.offset_from(p_orig) as usize;
                let ptr = xmemdupz(p as *const c_void, part_len) as *mut c_char;
                let cursor_lnum = nvim_dp_get_cursor_lnum();
                ml_append(cursor_lnum, ptr, 0, false);
                xfree(ptr as *mut c_void);

                // After ml_append, re-fetch line ptr (may have been reallocated).
                let curline2 = nvim_dp_get_cursor_line_ptr();
                let ptr2 = xmemdupz(curline2 as *const c_void, split_pos as usize) as *mut c_char;
                ml_replace(cursor_lnum, ptr2, false);
                nr_lines += 1;
                // C: dir = FORWARD; -- handled via effective_dir above
                nvim_dp_buf_updates_send_changes(cursor_lnum, 1, 1);
            }
            if flags & PUT_LINE_FORWARD != 0 {
                // Must be "p" for a Visual block, put lines below the block.
                nvim_dp_set_cursor_to_b_visual_vi_end();
            }
            dp_set_op_start_cursor(); // for mark_adjust()
            dp_set_op_end_cursor();
        }

        // Effective direction after linewise adjustments.
        // C mutates `dir = FORWARD` for both PUT_LINE_SPLIT and PUT_LINE_FORWARD.
        let effective_dir = if y_type == K_MT_LINE_WISE_V
            && (flags & PUT_LINE_SPLIT != 0 || flags & PUT_LINE_FORWARD != 0)
        {
            FORWARD
        } else {
            dir
        };

        if flags & PUT_LINE != 0 {
            // :put command or "p" in Visual line mode.
            y_type = K_MT_LINE_WISE_V;
        }

        if y_size == 0 || y_array.is_null() {
            nvim_dp_semsg_E353(regname);
            break 'do_put_body;
        }

        if y_type == K_MT_BLOCK_WISE_V {
            lnum = nvim_dp_get_cursor_lnum() + y_size as c_int + 1;
            lnum = lnum.min(nvim_dp_get_ml_line_count() + 1);
            let cursor_lnum = nvim_dp_get_cursor_lnum();
            if u_save(cursor_lnum - 1, lnum) == FAIL {
                break 'do_put_body;
            }
        } else if y_type == K_MT_LINE_WISE_V {
            lnum = nvim_dp_get_cursor_lnum();
            // Correct line number for closed fold. Don't move the cursor yet.
            if effective_dir == BACKWARD {
                nvim_dp_hasFolding_backward(&raw mut lnum);
            } else {
                nvim_dp_hasFolding_forward(&raw mut lnum);
            }
            if effective_dir == FORWARD {
                lnum += 1;
            }
            // In an empty buffer the empty line is going to be replaced.
            let save_failed = if nvim_dp_buf_is_empty() {
                u_save(0, 2) == FAIL
            } else {
                u_save(lnum - 1, lnum) == FAIL
            };
            if save_failed {
                break 'do_put_body;
            }
            if effective_dir == FORWARD {
                nvim_dp_set_cursor_lnum(lnum - 1);
            } else {
                nvim_dp_set_cursor_lnum(lnum);
            }
            dp_set_op_start_cursor(); // for mark_adjust()
        } else if u_save_cursor() == FAIL {
            break 'do_put_body;
        }

        if cur_ve_flags == K_OPT_VE_FLAG_ALL && y_type == K_MT_CHAR_WISE_V {
            if nvim_dp_gchar_cursor() == TAB {
                let viscol = nvim_dp_getviscol();
                let ts = nvim_dp_get_b_p_ts();
                let vts = nvim_dp_get_b_p_vts_array();
                // Don't need to insert spaces when "p" on the last position of a
                // tab or "P" on the first position.
                if if dir == FORWARD {
                    nvim_dp_tabstop_padding(viscol, ts, vts) != 1
                } else {
                    nvim_dp_get_cursor_coladd() > 0
                } {
                    nvim_dp_coladvance_force(viscol);
                } else {
                    nvim_dp_set_cursor_coladd(0);
                }
            } else if nvim_dp_get_cursor_coladd() > 0 || nvim_dp_gchar_cursor() == NUL {
                nvim_dp_coladvance_force(nvim_dp_getviscol() + if dir == FORWARD { 1 } else { 0 });
            }
        }

        lnum = nvim_dp_get_cursor_lnum();
        let mut col = nvim_dp_get_cursor_col();

        // Block mode
        if y_type == K_MT_BLOCK_WISE_V {
            let mut incr: c_int = 0;
            let mut bd = BlockDef::zeroed();
            let c = nvim_dp_gchar_cursor();
            let mut endcol2: c_int = 0;

            if dir == FORWARD && c != NUL {
                if cur_ve_flags == K_OPT_VE_FLAG_ALL {
                    nvim_dp_getvcol_cursor(&raw mut col, &raw mut endcol2);
                } else {
                    nvim_dp_getvcol_cursor_end_only(&raw mut col);
                }
                // move to start of next multi-byte character
                let adv = utfc_ptr2len(nvim_dp_get_cursor_pos_ptr());
                dp_cursor_col_add(adv);
                col += 1;
            } else {
                nvim_dp_getvcol_cursor(&raw mut col, &raw mut endcol2);
            }

            col += nvim_dp_get_cursor_coladd();
            if cur_ve_flags == K_OPT_VE_FLAG_ALL
                && (nvim_dp_get_cursor_coladd() > 0 || endcol2 == nvim_dp_get_cursor_col())
            {
                if dir == FORWARD && c == NUL {
                    col += 1;
                }
                if dir != FORWARD && c != NUL && nvim_dp_get_cursor_coladd() > 0 {
                    dp_cursor_col_add(1);
                }
                if c == TAB {
                    if dir == BACKWARD && nvim_dp_get_cursor_col() != 0 {
                        dp_cursor_col_add(-1);
                    }
                    if dir == FORWARD && col - 1 == endcol2 {
                        dp_cursor_col_add(1);
                    }
                }
            }
            nvim_dp_set_cursor_coladd(0);
            bd.textcol = 0;

            let mut i = 0usize;
            while i < y_size {
                let mut spaces: c_int = 0;
                let mut lines_appended: c_int = 0;

                bd.startspaces = 0;
                bd.endspaces = 0;
                vcol = 0;
                let mut delcount: c_int = 0;

                // add a new line if necessary
                if nvim_dp_get_cursor_lnum() > nvim_dp_get_ml_line_count() {
                    if ml_append(
                        nvim_dp_get_ml_line_count(),
                        c"".as_ptr() as *mut c_char,
                        0,
                        false,
                    ) == FAIL
                    {
                        break;
                    }
                    nr_lines += 1;
                    lines_appended = 1;
                }

                // get the old line and advance to the position to insert at
                let oldp = nvim_dp_get_cursor_line_ptr();
                let oldlen = nvim_dp_get_cursor_line_len();

                let mut csarg_buf = DpCharsizeArgBuf([0u8; 320]);
                let csarg = csarg_buf.0.as_mut_ptr() as *mut c_void;
                let cursor_lnum = nvim_dp_get_cursor_lnum();
                let cstype = nvim_dp_init_charsize_arg(csarg, cursor_lnum, oldp);
                let mut ci = dp_utf_ptr2str_char_info(oldp);
                vcol = 0;
                while vcol < col && *ci.ptr != 0 {
                    incr = dp_win_charsize(cstype, vcol, ci.ptr, ci.chr.value, csarg).width;
                    vcol += incr;
                    ci = dp_utfc_next(ci);
                }
                let ptr = ci.ptr;
                bd.textcol = ptr.offset_from(oldp) as c_int;

                let shortline = vcol < col || (vcol == col && *ptr == 0);

                if vcol < col {
                    // line too short, pad with spaces
                    bd.startspaces = col - vcol;
                } else if vcol > col {
                    bd.endspaces = vcol - col;
                    bd.startspaces = incr - bd.endspaces;
                    bd.textcol -= 1;
                    delcount = 1;
                    bd.textcol -= utf_head_off(oldp, oldp.add(bd.textcol as usize));
                    if *oldp.add(bd.textcol as usize) != b'\t' as c_char {
                        // Only a Tab can be split into spaces.
                        delcount = 0;
                        bd.endspaces = 0;
                    }
                }

                let yanklen = (*y_array.add(i)).size as c_int;

                if flags & PUT_BLOCK_INNER == 0 {
                    // calculate number of spaces required to fill right side of block
                    spaces = y_width + 1;

                    let mut csarg_buf2 = DpCharsizeArgBuf([0u8; 320]);
                    let csarg2 = csarg_buf2.0.as_mut_ptr() as *mut c_void;
                    let yline = (*y_array.add(i)).data;
                    nvim_dp_init_charsize_arg_lnum0(csarg2, yline);
                    let mut ci2 = dp_utf_ptr2str_char_info(yline);
                    while *ci2.ptr != 0 {
                        spaces -= dp_win_charsize(cstype, 0, ci2.ptr, ci2.chr.value, csarg2).width;
                        ci2 = dp_utfc_next(ci2);
                    }
                    spaces = spaces.max(0);
                }

                // Insert the new text.
                // First check for multiplication overflow.
                if yanklen + spaces != 0
                    && count > ((c_int::MAX - (bd.startspaces + bd.endspaces)) / (yanklen + spaces))
                {
                    emsg(nvim_dp_get_e_resulting_text_too_long());
                    break;
                }

                totlen = count as usize * (yanklen + spaces) as usize
                    + bd.startspaces as usize
                    + bd.endspaces as usize;
                let newp = xmalloc(totlen + oldlen as usize + 1) as *mut c_char;

                // copy part up to cursor to new line
                let mut out_ptr = newp;
                std::ptr::copy_nonoverlapping(oldp, out_ptr, bd.textcol as usize);
                out_ptr = out_ptr.add(bd.textcol as usize);

                // may insert some spaces before the new text
                std::ptr::write_bytes(out_ptr, b' ', bd.startspaces as usize);
                out_ptr = out_ptr.add(bd.startspaces as usize);

                // insert the new text
                for j in 0..count {
                    let ydata = (*y_array.add(i)).data;
                    std::ptr::copy_nonoverlapping(ydata, out_ptr, yanklen as usize);
                    out_ptr = out_ptr.add(yanklen as usize);

                    // insert block's trailing spaces only if there's text behind
                    if (j < count - 1 || !shortline) && spaces > 0 {
                        std::ptr::write_bytes(out_ptr, b' ', spaces as usize);
                        out_ptr = out_ptr.add(spaces as usize);
                    } else {
                        totlen -= spaces as usize; // didn't use these spaces
                    }
                }

                // may insert some spaces after the new text
                std::ptr::write_bytes(out_ptr, b' ', bd.endspaces as usize);
                out_ptr = out_ptr.add(bd.endspaces as usize);

                // move the text after the cursor to the end of the line.
                let columns = oldlen - bd.textcol - delcount + 1;
                std::ptr::copy_nonoverlapping(
                    oldp.add((bd.textcol + delcount) as usize),
                    out_ptr,
                    columns as usize,
                );
                ml_replace(nvim_dp_get_cursor_lnum(), newp, false);
                nvim_dp_extmark_splice_cols(
                    nvim_dp_get_cursor_lnum() - 1,
                    bd.textcol,
                    delcount,
                    totlen as c_int,
                    lines_appended,
                    K_EXTMARK_UNDO,
                );

                nvim_dp_set_cursor_lnum(nvim_dp_get_cursor_lnum() + 1);
                if i == 0 {
                    dp_cursor_col_add(bd.startspaces);
                }

                i += 1;
            }

            nvim_dp_changed_lines(
                lnum,
                0,
                nvim_dp_get_op_start_lnum() + y_size as c_int - nr_lines,
                nr_lines,
            );

            // Set '[ mark.
            let cursor_lnum_end = nvim_dp_get_cursor_lnum();
            dp_set_op_start_cursor();
            nvim_dp_set_op_start_lnum(lnum);

            // adjust '] mark
            nvim_dp_set_op_end_lnum(cursor_lnum_end - 1);
            nvim_dp_set_op_end_col(0.max(bd.textcol + totlen as c_int - 1));
            nvim_dp_set_op_end_coladd(0);

            if flags & PUT_CURSEND != 0 {
                let mut op_end = PosT::default();
                nvim_dp_get_op_end(&raw mut op_end);
                nvim_dp_set_cursor(&raw const op_end);
                dp_cursor_col_add(1);

                // in Insert mode we might be after the NUL, correct for that
                let len = nvim_dp_get_cursor_line_len();
                let cur_col = nvim_dp_get_cursor_col();
                if cur_col > len {
                    nvim_dp_set_cursor_col(len);
                }
            } else {
                nvim_dp_set_cursor_lnum(lnum);
            }
        } else {
            let yanklen = (*y_array).size as c_int;

            // Character or Line mode
            if y_type == K_MT_CHAR_WISE_V {
                // if type is kMTCharWise, FORWARD is the same as BACKWARD on the next char
                if effective_dir == FORWARD && nvim_dp_gchar_cursor() != NUL {
                    let bytelen = utfc_ptr2len(nvim_dp_get_cursor_pos_ptr());
                    // put it on the next of the multi-byte character.
                    col += bytelen;
                    if yanklen != 0 {
                        dp_cursor_col_add(bytelen);
                        nvim_dp_op_end_col_add(bytelen);
                    }
                }
                dp_set_op_start_cursor();
            } else if effective_dir == BACKWARD {
                // Line mode: BACKWARD is the same as FORWARD on the previous line
                lnum -= 1;
            }

            // save cursor position for multi-line insert
            let mut new_cursor = PosT::default();
            nvim_dp_get_cursor(&raw mut new_cursor);

            // simple case: insert into one line at a time
            if y_type == K_MT_CHAR_WISE_V && y_size == 1 {
                let mut end_lnum: c_int = 0;
                let start_lnum = lnum;
                let mut first_byte_off: c_int = 0;

                let visual_active_charwise = VIsual_active;
                if visual_active_charwise {
                    let vi_end_lnum = nvim_dp_get_b_visual_vi_end_lnum();
                    let vi_start_lnum = nvim_dp_get_b_visual_vi_start_lnum();
                    end_lnum = vi_end_lnum.max(vi_start_lnum);
                    if end_lnum > start_lnum {
                        // "col" is valid for the first line, in following lines
                        // the virtual column needs to be used.
                        let mut vcol_mid = 0;
                        nvim_dp_getvcol_pos(lnum, col, 0, &raw mut vcol_mid);
                        vcol = vcol_mid;
                    }
                }

                if count == 0 || yanklen == 0 {
                    if visual_active_charwise {
                        lnum = end_lnum;
                    }
                } else if count > c_int::MAX / yanklen {
                    // multiplication overflow
                    emsg(nvim_dp_get_e_resulting_text_too_long());
                } else {
                    totlen = count as usize * yanklen as usize;
                    loop {
                        let oldp = ml_get(lnum);
                        let oldlen = ml_get_len(lnum);
                        if lnum > start_lnum {
                            let mut pos_lnum = lnum;
                            let mut pos_col = 0;
                            let mut pos_coladd = 0;
                            if nvim_dp_getvpos(
                                &raw mut pos_lnum,
                                &raw mut pos_col,
                                &raw mut pos_coladd,
                                vcol,
                            ) == OK
                            {
                                col = pos_col;
                            } else {
                                col = MAXCOL;
                            }
                        }
                        if visual_active_charwise && col > oldlen {
                            lnum += 1;
                            if !visual_active_charwise || lnum > end_lnum {
                                break;
                            }
                            continue;
                        }
                        let newp = xmalloc(totlen + oldlen as usize + 1) as *mut c_char;
                        std::ptr::copy_nonoverlapping(oldp, newp, col as usize);
                        let mut out_ptr = newp.add(col as usize);
                        for _ in 0..count {
                            let ydata = (*y_array).data;
                            std::ptr::copy_nonoverlapping(ydata, out_ptr, yanklen as usize);
                            out_ptr = out_ptr.add(yanklen as usize);
                        }
                        std::ptr::copy_nonoverlapping(
                            oldp.add(col as usize),
                            out_ptr,
                            (oldlen - col + 1) as usize, // +1 for NUL
                        );
                        ml_replace(lnum, newp, false);

                        // compute the byte offset for the last character
                        first_byte_off = utf_head_off(newp, out_ptr.sub(1));

                        // Place cursor on last putted char.
                        if lnum == nvim_dp_get_cursor_lnum() {
                            // make sure curwin->w_virtcol is updated
                            nvim_dp_changed_cline_bef_curs();
                            nvim_dp_invalidate_botline();
                            dp_cursor_col_add(totlen as c_int - 1);
                        }
                        nvim_dp_changed_bytes(lnum, col);
                        nvim_dp_extmark_splice_cols(
                            lnum - 1,
                            col,
                            0,
                            totlen as c_int,
                            0,
                            K_EXTMARK_UNDO,
                        );
                        if visual_active_charwise {
                            lnum += 1;
                        }

                        if !visual_active_charwise || lnum > end_lnum {
                            break;
                        }
                    }

                    if visual_active_charwise {
                        // reset lnum to the last visual line
                        lnum -= 1;
                    }
                }

                // put '] at the first byte of the last character
                dp_set_op_end_cursor();
                nvim_dp_op_end_col_add(-first_byte_off);

                // For "CTRL-O p" in Insert mode, put cursor after last char
                if totlen != 0 && (restart_edit != 0 || flags & PUT_CURSEND != 0) {
                    dp_cursor_col_add(1);
                } else {
                    dp_cursor_col_add(-first_byte_off);
                }
            } else {
                // Multi-line charwise or linewise insert
                let mut new_lnum = new_cursor.lnum;
                let mut indent: c_int;
                let mut orig_indent: c_int = 0;
                let mut indent_diff: c_int = 0;
                let mut first_indent = true;
                let mut lendiff: c_int = 0;

                if flags & PUT_FIXINDENT != 0 {
                    orig_indent = nvim_dp_get_indent();
                }

                // Insert at least one line. When y_type is kMTCharWise, break the first
                // line in two.
                let mut cnt = 1;
                let mut error_break = false;
                while cnt <= count {
                    let mut i = 0usize;
                    if y_type == K_MT_CHAR_WISE_V {
                        // Split the current line in two at the insert position.
                        // First insert y_array[size - 1] in front of second line.
                        // Then append y_array[0] to first line.
                        lnum = new_cursor.lnum;
                        let cur_line_ptr = ml_get(lnum);
                        let ptr = cur_line_ptr.add(col as usize);
                        let ptrlen = (ml_get_len(lnum) - col) as usize;
                        let last_entry = y_array.add(y_size - 1);
                        totlen = (*last_entry).size;
                        let newp = xmalloc(ptrlen + totlen + 1) as *mut c_char;
                        // STRCPY(newp, y_array[y_size - 1].data)
                        std::ptr::copy_nonoverlapping((*last_entry).data, newp, totlen);
                        // STRCPY(newp + totlen, ptr)
                        std::ptr::copy_nonoverlapping(ptr, newp.add(totlen), ptrlen + 1);
                        // insert second line
                        ml_append(lnum, newp, 0, false);
                        new_lnum += 1;
                        xfree(newp as *mut c_void);

                        let oldp = ml_get(lnum);
                        let newp2 = xmalloc(col as usize + yanklen as usize + 1) as *mut c_char;
                        // copy first part of line
                        std::ptr::copy_nonoverlapping(oldp, newp2, col as usize);
                        // append to first line
                        std::ptr::copy_nonoverlapping(
                            (*y_array).data,
                            newp2.add(col as usize),
                            yanklen as usize + 1,
                        );
                        ml_replace(lnum, newp2, false);

                        nvim_dp_set_cursor_lnum(lnum);
                        i = 1;
                    }

                    while i < y_size {
                        if y_type != K_MT_CHAR_WISE_V || i < y_size - 1 {
                            if ml_append(lnum, (*y_array.add(i)).data, 0, false) == FAIL {
                                error_break = true;
                                break;
                            }
                            new_lnum += 1;
                        }
                        lnum += 1;
                        nr_lines += 1;
                        if flags & PUT_FIXINDENT != 0 {
                            let old_lnum = nvim_dp_get_cursor_lnum();
                            let old_col = nvim_dp_get_cursor_col();
                            let old_coladd = nvim_dp_get_cursor_coladd();
                            nvim_dp_set_cursor_lnum(lnum);
                            let line_ptr = ml_get(lnum);
                            if cnt == count && i == y_size - 1 {
                                lendiff = ml_get_len(lnum);
                            }
                            if *line_ptr == b'#' as c_char && nvim_dp_preprocs_left() {
                                indent = 0; // Leave # lines at start
                            } else if *line_ptr == 0 {
                                indent = 0; // Ignore empty lines
                            } else if first_indent {
                                indent_diff = orig_indent - nvim_dp_get_indent();
                                indent = orig_indent;
                                first_indent = false;
                            } else {
                                indent = nvim_dp_get_indent() + indent_diff;
                                if indent < 0 {
                                    indent = 0;
                                }
                            }
                            nvim_dp_set_indent(indent);
                            // restore cursor
                            nvim_dp_set_cursor_lnum(old_lnum);
                            nvim_dp_set_cursor_col(old_col);
                            nvim_dp_set_cursor_coladd(old_coladd);
                            // remember how many chars were removed
                            if cnt == count && i == y_size - 1 {
                                lendiff -= ml_get_len(lnum);
                            }
                        }
                        i += 1;
                    }
                    if error_break {
                        break;
                    }

                    // extmark splice
                    let mut totsize: i64 = 0;
                    let mut lastsize: c_int = 0;
                    if y_type == K_MT_CHAR_WISE_V
                        || (y_type == K_MT_LINE_WISE_V && flags & PUT_LINE_SPLIT != 0)
                    {
                        for ii in 0..y_size - 1 {
                            totsize += (*y_array.add(ii)).size as i64 + 1;
                        }
                        lastsize = (*y_array.add(y_size - 1)).size as c_int;
                        totsize += lastsize as i64;
                    }
                    if y_type == K_MT_CHAR_WISE_V {
                        nvim_dp_extmark_splice(
                            new_cursor.lnum - 1,
                            col,
                            0,
                            0,
                            y_size as c_int - 1,
                            lastsize,
                            totsize,
                            K_EXTMARK_UNDO,
                        );
                    } else if y_type == K_MT_LINE_WISE_V && flags & PUT_LINE_SPLIT != 0 {
                        // Account for last pasted NL + last NL
                        nvim_dp_extmark_splice(
                            new_cursor.lnum - 1,
                            split_pos,
                            0,
                            0,
                            y_size as c_int + 1,
                            0,
                            totsize + 2,
                            K_EXTMARK_UNDO,
                        );
                    }

                    if cnt == 1 {
                        new_lnum = lnum;
                    }

                    cnt += 1;
                }

                // error: label in C -- Adjust marks.
                if y_type == K_MT_LINE_WISE_V {
                    nvim_dp_set_op_start_lnum(nvim_dp_get_op_start_lnum()); // no-op if already set
                                                                            // curbuf->b_op_start.col = 0
                                                                            // dp_set_op_start_cursor sets from curwin which isn't right for
                                                                            // linewise put -- col must be 0.
                    dp_set_op_start_cursor();
                    nvim_register_curbuf_set_op_start_col(0);
                    if effective_dir == FORWARD {
                        nvim_dp_set_op_start_lnum(nvim_dp_get_op_start_lnum() + 1);
                    }
                }

                let kind = if y_type == K_MT_LINE_WISE_V && flags & PUT_LINE_SPLIT == 0 {
                    K_EXTMARK_UNDO
                } else {
                    K_EXTMARK_NOOP
                };
                let mark_lnum1 =
                    nvim_dp_get_op_start_lnum() + if y_type == K_MT_CHAR_WISE_V { 1 } else { 0 };
                nvim_dp_mark_adjust(mark_lnum1, nr_lines, kind);

                // note changed text for displaying and folding
                if y_type == K_MT_CHAR_WISE_V {
                    nvim_dp_changed_lines(
                        nvim_dp_get_cursor_lnum(),
                        col,
                        nvim_dp_get_cursor_lnum() + 1,
                        nr_lines,
                    );
                } else {
                    nvim_dp_changed_lines(
                        nvim_dp_get_op_start_lnum(),
                        0,
                        nvim_dp_get_op_start_lnum(),
                        nr_lines,
                    );
                }

                // Put the '] mark on the first byte of the last inserted character.
                // Correct the length for change in indent.
                nvim_dp_set_op_end_lnum(new_lnum);
                let last_entry_size = (*y_array.add(y_size - 1)).size as c_int;
                let raw_col = 0.max(last_entry_size - lendiff);
                if raw_col > 1 {
                    nvim_dp_set_op_end_col(raw_col - 1);
                    if (*y_array.add(y_size - 1)).size > 0 {
                        let last_data = (*y_array.add(y_size - 1)).data;
                        let last_size = (*y_array.add(y_size - 1)).size;
                        nvim_dp_set_op_end_col(
                            (raw_col - 1) - utf_head_off(last_data, last_data.add(last_size - 1)),
                        );
                    }
                } else {
                    nvim_dp_set_op_end_col(0);
                }

                if flags & PUT_CURSLINE != 0 {
                    // ":put": put cursor on last inserted line
                    nvim_dp_set_cursor_lnum(lnum);
                    nvim_dp_beginline();
                } else if flags & PUT_CURSEND != 0 {
                    // put cursor after inserted text
                    if y_type == K_MT_LINE_WISE_V {
                        if lnum >= nvim_dp_get_ml_line_count() {
                            nvim_dp_set_cursor_lnum(nvim_dp_get_ml_line_count());
                        } else {
                            nvim_dp_set_cursor_lnum(lnum + 1);
                        }
                        nvim_dp_set_cursor_col(0);
                    } else {
                        nvim_dp_set_cursor_lnum(new_lnum);
                        nvim_dp_set_cursor_col(raw_col);
                        dp_set_op_end_cursor();
                        if raw_col > 1 {
                            nvim_dp_set_op_end_col(raw_col - 1);
                        }
                    }
                } else if y_type == K_MT_LINE_WISE_V {
                    // put cursor on first non-blank in first inserted line
                    nvim_dp_set_cursor_col(0);
                    if effective_dir == FORWARD {
                        nvim_dp_set_cursor_lnum(nvim_dp_get_cursor_lnum() + 1);
                    }
                    nvim_dp_beginline();
                } else {
                    // put cursor on first inserted character
                    nvim_dp_set_cursor(&raw const new_cursor);
                }
            }
        }

        nvim_dp_msgmore(nr_lines);
        nvim_dp_set_curswant();

        // Make sure the cursor is not after the NUL.
        let len = nvim_dp_get_cursor_line_len();
        let cur_col = nvim_dp_get_cursor_col();
        if cur_col > len {
            if cur_ve_flags == K_OPT_VE_FLAG_ALL {
                nvim_dp_set_cursor_coladd(cur_col - len);
            }
            nvim_dp_set_cursor_col(len);
        }
    }; // end 'do_put_body

    // Cleanup (corresponds to C's "end:" label -- always runs).
    if nvim_register_cmod_lockmarks() {
        nvim_register_curbuf_set_op_start(&raw const orig_start);
        nvim_register_curbuf_set_op_end(&raw const orig_end);
    }
    if allocated {
        xfree(insert_string.data as *mut c_void);
    }
    if regname == c_int::from(b'=') {
        xfree(y_array as *mut c_void);
    }

    VIsual_active = false;

    // If the cursor is past the end of the line put it at the end.
    nvim_dp_adjust_cursor_eol();
}

extern "C" {
    fn nvim_dp_curbuf_is_terminal() -> bool;
    fn stuffReadbuff(s: *const c_char);
}

/// Compute the byte length of a C string using the libc strlen convention.
#[inline]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Find the first occurrence of `ch` in the C string `s`.
#[inline]
unsafe fn libc_strchr(s: *mut c_char, ch: u8) -> *mut c_char {
    let mut p = s;
    while *p != 0 {
        if *p == ch as c_char {
            return p;
        }
        p = p.add(1);
    }
    std::ptr::null_mut()
}

// ---------------------------------------------------------------------------
// Phase 28: f_setreg, f_getregion, f_getregionpos — real Rust implementations
//
// Logic ported branch-for-branch from the original C bodies (funcs_shim.c /
// register.c).  C is retained only for the small accessor functions that wrap
// struct field access and gettext-formatted error messages.
// ---------------------------------------------------------------------------

extern "C" {
    // --- global get/set (C globals, not accessible directly from Rust) ---
    fn nvim_register_get_curbuf() -> *mut c_void; // buf_T*
    fn nvim_register_set_curbuf(buf: *mut c_void);
    fn nvim_register_set_curwin_buffer(buf: *mut c_void);
    fn nvim_register_get_virtual_op() -> c_int;
    fn nvim_register_set_virtual_op(v: c_int);
    fn nvim_register_set_virtual_op_from_curwin(); // virtual_op = virtual_active(curwin)

    // --- buffer / line accessors (struct-nested, no direct Rust equivalent) ---
    fn nvim_register_buf_ml_mfp_is_null(buf: *mut c_void) -> bool;
    fn nvim_register_buf_line_count(buf: *mut c_void) -> c_int;
    fn nvim_register_curbuf_fnum() -> c_int;
    fn ml_get_buf_len(buf: *mut c_void, lnum: c_int) -> c_int;

    // --- position helpers requiring curwin or combining calls ---
    fn nvim_register_getvvcol(pos: *mut PosT, sc: *mut c_int, ec: *mut c_int); // needs curwin

    // --- oparg_T setter (oparg_T layout is opaque to Rust) ---
    fn nvim_oap_set_for_blockwise(
        oap: *mut c_void,
        start: *mut PosT,
        end: *mut PosT,
        start_vcol: c_int,
        end_vcol: c_int,
    );

    // --- tv_dict_len is static inline; keep a C wrapper ---
    fn nvim_register_tv_dict_len(d: *mut c_void) -> c_int;

    // --- setreg list-write helper (TV_LIST_ITER_CONST is a C macro) ---
    fn nvim_register_setreg_write_lst(
        regname: c_int,
        ll: *mut c_void,
        append: bool,
        yank_type: c_int,
        block_len: c_int,
    ) -> c_int;

    // --- kListLenMayKnow wrapper (constant not accessible from Rust) ---
    fn nvim_register_tv_list_alloc_ret(rettv: *mut c_void);

    // --- error message emitters (keep gettext _() macro in C) ---
    fn nvim_register_emsg_buffer_not_loaded();
    fn nvim_register_semsg_invalid_line(lnum: c_int);
    fn nvim_register_semsg_invalid_col(col: c_int);
    fn nvim_register_semsg_invargNval_type(val: *const c_char);
    fn nvim_register_semsg_invargval_value();
    fn nvim_register_semsg_toomanyarg_setreg();

    // --- direct C externs (no wrapper needed) ---
    fn unadjust_for_sel_inner(p: *mut PosT) -> bool;
    fn tv_check_for_list_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_opt_dict_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn tv_dict_get_string(d: *const c_void, key: *const c_char, save: bool) -> *mut c_char;
    fn tv_dict_get_number(d: *const c_void, key: *const c_char) -> i64;
    fn tv_dict_get_bool(d: *const c_void, key: *const c_char, def: c_int) -> c_int;
    fn tv_list_append_list(l: *mut c_void, itemlist: *mut c_void);
    fn tv_list_append_number(l: *mut c_void, nr: i64);
    fn tv_get_string_chk(tv: *const c_void) -> *const c_char;
    /// Returns &argvars[idx] — the typval_T* at the given index.
    fn nvim_typval_array_get(args: *const c_void, idx: c_int) -> *const c_void;
    fn ml_get_pos(pos: *const PosT) -> *mut c_char;
    fn mb_prevptr(line: *mut c_char, p: *mut c_char) -> *mut c_char;
    fn list2fpos(
        arg: *const c_void,
        pos: *mut PosT,
        fnum: *mut c_int,
        curswantp: *mut c_int,
        charcol: bool,
    ) -> c_int;
    /// tv_dict_find returns dictitem_T* where di_tv is the first field (offset 0).
    /// Safe to cast to *mut TypvalT to access the typval_T directly.
    fn tv_dict_find(d: *const c_void, key: *const c_char, len: isize) -> *mut TypvalT;
}

/// Thin mirror of C's typval_T (16 bytes on 64-bit platforms).
///
/// Layout: `{ VarType v_type (4 bytes), VarLockStatus v_lock (4 bytes), union vval (8 bytes) }`
/// Only the fields we read are accessed; the union is opaque beyond `v_type`.
#[repr(C)]
struct TypvalT {
    v_type: c_int, // VarType enum (int)
    v_lock: c_int, // VarLockStatus (int)
    vval: u64,     // union — 8 bytes; interpret via v_type
}
const _: () = assert!(std::mem::size_of::<TypvalT>() == 16);

impl TypvalT {
    /// Return the v_list pointer (valid when v_type == VAR_LIST).
    fn as_list(&self) -> *mut c_void {
        // The union starts at offset 8; a pointer is at offset 8 on 64-bit.
        self.vval as usize as *mut c_void
    }
    /// Return the v_dict pointer (valid when v_type == VAR_DICT).
    fn as_dict(&self) -> *mut c_void {
        self.vval as usize as *mut c_void
    }
    /// Set v_number (the int64_t union member).
    unsafe fn set_number(ptr: *mut Self, n: i64) {
        (*ptr).vval = n as u64;
    }
}

// VAR_* type constants matching eval/typval_defs.h.
const VAR_UNKNOWN: c_int = 0;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;

// CTRL-V byte value (0x16 = 22), already declared above as CTRL_V.

/// Convenience: get `argvars[idx]` as `*const TypvalT`.
unsafe fn tv_at(argvars: *const c_void, idx: c_int) -> *const TypvalT {
    nvim_typval_array_get(argvars, idx).cast::<TypvalT>()
}
/// Convenience: get `argvars[idx]` as `*mut TypvalT`.
unsafe fn tv_at_mut(argvars: *mut c_void, idx: c_int) -> *mut TypvalT {
    nvim_typval_array_get(argvars.cast_const(), idx)
        .cast::<TypvalT>()
        .cast_mut()
}

/// Parse a regtype string character into (MotionType, block_len_delta, advance).
/// Returns `Some((motion_type, block_len))` or `None` on failure.
/// On success the caller should advance the pointer by 1 (the consumed char);
/// for blockwise with digits the pointer is returned already past the digits.
///
/// # Safety
///
/// `pp` must point to a valid NUL-terminated C string.
unsafe fn parse_yank_type(pp: *mut *mut c_char) -> Option<(c_int, c_int)> {
    let stropt = *pp;
    let first = *stropt as u8;
    match first {
        b'v' | b'c' => Some((K_MT_CHAR_WISE, -1)),
        b'V' | b'l' => Some((K_MT_LINE_WISE, -1)),
        b'b' | 0x16 /* Ctrl_V */ => {
            let mut block_len: c_int = -1;
            let next = stropt.add(1);
            if (*next as u8).is_ascii_digit() {
                // advance into the digit sequence
                *pp = next;
                block_len = getdigits_int(pp, false, 0) - 1;
                // pp now points one past the last digit; we back up one so
                // the outer loop's stropt++ lands on the right position.
                *pp = (*pp).sub(1);
            }
            Some((K_MT_BLOCK_WISE, block_len))
        }
        _ => None,
    }
}

/// "setreg()" function — sets register contents from a VimL value/dict.
///
/// # Safety
///
/// Caller must provide valid pointers to typval_T arrays.
#[unsafe(export_name = "f_setreg")]
pub unsafe extern "C" fn rs_f_setreg(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let mut append = false;
    let mut block_len: c_int = -1;
    let mut yank_type: c_int = K_MT_UNKNOWN;

    TypvalT::set_number(rettv.cast::<TypvalT>(), 1); // FAIL is default.

    let strregname = tv_get_string_chk(nvim_typval_array_get(argvars, 0));
    if strregname.is_null() {
        return;
    }
    let mut regname = *strregname as u8;
    if regname == 0 || regname == b'@' {
        regname = b'"';
    }

    // argvars[1]: either a dict or the regcontents value directly.
    let arg1 = tv_at(argvars, 1);
    let arg1_type = (*arg1).v_type;
    let mut regcontents_tv: *mut TypvalT = std::ptr::null_mut();
    let mut pointreg: u8 = 0;

    if arg1_type == VAR_DICT {
        let d = (*arg1).as_dict();

        if nvim_register_tv_dict_len(d) == 0 {
            // Empty dict: clear the register with an empty list.
            let mut empty: [*mut c_char; 2] = [std::ptr::null_mut(), std::ptr::null_mut()];
            rs_write_reg_contents_lst(
                c_int::from(regname),
                empty.as_mut_ptr(),
                false,
                K_MT_UNKNOWN,
                -1,
            );
            return;
        }

        let di = tv_dict_find(d, c"regcontents".as_ptr(), -1);
        if !di.is_null() {
            regcontents_tv = di;
        }

        let stropt = tv_dict_get_string(d, c"regtype".as_ptr(), false);
        if !stropt.is_null() {
            let mut p = stropt as *mut c_char;
            match parse_yank_type(&raw mut p) {
                None => {
                    nvim_register_semsg_invargval_value();
                    return;
                }
                Some((yt, bl)) => {
                    // advance p by 1 for the consumed character
                    p = p.add(1);
                    if *p != 0 {
                        nvim_register_semsg_invargval_value();
                        return;
                    }
                    yank_type = yt;
                    if bl >= 0 {
                        block_len = bl;
                    }
                }
            }
        }

        if regname == b'"' {
            let pt = tv_dict_get_string(d, c"points_to".as_ptr(), false);
            if !pt.is_null() {
                pointreg = *pt as u8;
                regname = pointreg;
            }
        } else if tv_dict_get_number(d, c"isunnamed".as_ptr()) != 0 {
            pointreg = regname;
        }
    } else {
        // argvars[1] is the regcontents value directly.
        regcontents_tv = tv_at(argvars, 1).cast_mut();
    }

    let mut set_unnamed = false;
    let arg2 = tv_at(argvars, 2);
    let arg2_type = (*arg2).v_type;
    if arg2_type != VAR_UNKNOWN {
        if yank_type != K_MT_UNKNOWN {
            nvim_register_semsg_toomanyarg_setreg();
            return;
        }
        let stropt = tv_get_string_chk(arg2.cast::<c_void>());
        if stropt.is_null() {
            return;
        }
        let mut p = stropt as *mut c_char;
        while *p != 0 {
            match *p as u8 {
                b'a' | b'A' => {
                    append = true;
                }
                b'u' | b'"' => {
                    set_unnamed = true;
                }
                _ => {
                    if let Some((yt, bl)) = parse_yank_type(&raw mut p) {
                        yank_type = yt;
                        if bl >= 0 {
                            block_len = bl;
                        }
                    }
                }
            }
            p = p.add(1);
        }
    }

    if !regcontents_tv.is_null() {
        let tv_type = (*regcontents_tv).v_type;
        if tv_type == VAR_LIST {
            let ll = (*regcontents_tv).as_list();
            let ret = nvim_register_setreg_write_lst(
                c_int::from(regname),
                ll,
                append,
                yank_type,
                block_len,
            );
            if ret != 0 {
                // FAIL: type error in list items
                return;
            }
        } else {
            let strval = tv_get_string_chk(regcontents_tv.cast::<c_void>());
            if strval.is_null() {
                return;
            }
            rs_write_reg_contents_ex(
                c_int::from(regname),
                strval,
                -1, // use strlen
                append,
                yank_type,
                block_len,
            );
        }
    }

    if pointreg != 0 {
        rs_get_yank_register(c_int::from(pointreg), 1 /* YREG_YANK */);
    }
    TypvalT::set_number(rettv.cast::<TypvalT>(), 0); // success

    if set_unnamed {
        rs_op_reg_set_previous(regname as c_char);
    }
}

/// Opaque oparg_T storage — 160 bytes, enough to hold oparg_T on any platform.
/// The actual C struct is accessed only through nvim_oap_* accessors.
const OPARG_T_SIZE: usize = 160;
type OpaqueOap = [u8; OPARG_T_SIZE];

/// Core region resolver — ported from funcs_shim.c `getregionpos`.
///
/// Parses two list positions + opts dict, validates buf/line/col,
/// mutates curbuf/curwin->w_buffer/virtual_op (callers save/restore),
/// fills `oap` for blockwise.
///
/// Returns `true` on success, `false` on failure (rettv list already allocated
/// by nvim_register_tv_list_alloc_ret before calling).
///
/// # Safety
///
/// `argvars`, `p1`, `p2`, `inclusive`, `region_type`, `oap` must all be valid.
unsafe fn region_resolve(
    argvars: *const c_void,
    rettv: *mut c_void,
    p1: *mut PosT,
    p2: *mut PosT,
    inclusive: *mut bool,
    region_type: *mut c_int,
    oap: *mut OpaqueOap,
) -> bool {
    nvim_register_tv_list_alloc_ret(rettv);

    // validate arg types: list, list, opt-dict.
    // FAIL == -1 in C; tv_check_for_* emit the error message themselves.
    if tv_check_for_list_arg(argvars, 0) == -1
        || tv_check_for_list_arg(argvars, 1) == -1
        || tv_check_for_opt_dict_arg(argvars, 2) == -1
    {
        return false;
    }

    let mut fnum1: c_int = -1;
    let mut fnum2: c_int = -1;
    let arg0 = nvim_typval_array_get(argvars, 0);
    let arg1 = nvim_typval_array_get(argvars, 1);
    if list2fpos(arg0, p1, &raw mut fnum1, std::ptr::null_mut(), false) != 0
        || list2fpos(arg1, p2, &raw mut fnum2, std::ptr::null_mut(), false) != 0
        || fnum1 != fnum2
    {
        return false;
    }

    // Read opts dict for exclusive / type.
    let arg2 = tv_at(argvars, 2);
    let arg2_type = (*arg2).v_type;
    let is_exclusive: bool;
    let type_str: *const c_char;

    if arg2_type == VAR_DICT {
        let d = (*arg2).as_dict();
        // default exclusive = (*p_sel == 'e')
        let p_sel_char = *nvim_get_p_sel() as u8;
        is_exclusive =
            tv_dict_get_bool(d, c"exclusive".as_ptr(), c_int::from(p_sel_char == b'e')) != 0;
        type_str = tv_dict_get_string(d, c"type".as_ptr(), false);
    } else {
        let p_sel_char = *nvim_get_p_sel() as u8;
        is_exclusive = p_sel_char == b'e';
        type_str = std::ptr::null();
    }

    // Parse the type string.
    let type_bytes: &[u8] = if type_str.is_null() {
        b"v"
    } else {
        let len = libc::strlen(type_str);
        std::slice::from_raw_parts(type_str as *const u8, len + 1)
    };

    let mut block_width: c_int = 0;
    if type_bytes[0] == b'v' && type_bytes[1] == 0 {
        *region_type = K_MT_CHAR_WISE;
    } else if type_bytes[0] == b'V' && type_bytes[1] == 0 {
        *region_type = K_MT_LINE_WISE;
    } else if type_bytes[0] == CTRL_V {
        *region_type = K_MT_BLOCK_WISE;
        if type_bytes[1] != 0 {
            let mut p = type_str.add(1) as *mut c_char;
            let digits = getdigits_int(&raw mut p, false, 0);
            if digits <= 0 || *p != 0 {
                nvim_register_semsg_invargNval_type(type_str);
                return false;
            }
            block_width = digits;
        }
    } else {
        nvim_register_semsg_invargNval_type(type_str);
        return false;
    }

    // Find the buffer.
    let findbuf: *mut c_void = if fnum1 != 0 {
        buflist_findnr(fnum1)
    } else {
        nvim_register_get_curbuf()
    };
    if findbuf.is_null() || nvim_register_buf_ml_mfp_is_null(findbuf) {
        nvim_register_emsg_buffer_not_loaded();
        return false;
    }

    // Validate p1.
    if (*p1).lnum < 1 || (*p1).lnum > nvim_register_buf_line_count(findbuf) {
        nvim_register_semsg_invalid_line((*p1).lnum);
        return false;
    }
    if (*p1).col == MAXCOL {
        (*p1).col = ml_get_buf_len(findbuf, (*p1).lnum) + 1;
    } else if (*p1).col < 1 || (*p1).col > ml_get_buf_len(findbuf, (*p1).lnum) + 1 {
        nvim_register_semsg_invalid_col((*p1).col);
        return false;
    }

    // Validate p2.
    if (*p2).lnum < 1 || (*p2).lnum > nvim_register_buf_line_count(findbuf) {
        nvim_register_semsg_invalid_line((*p2).lnum);
        return false;
    }
    if (*p2).col == MAXCOL {
        (*p2).col = ml_get_buf_len(findbuf, (*p2).lnum) + 1;
    } else if (*p2).col < 1 || (*p2).col > ml_get_buf_len(findbuf, (*p2).lnum) + 1 {
        nvim_register_semsg_invalid_col((*p2).col);
        return false;
    }

    // Set curbuf / curwin->w_buffer / virtual_op.
    nvim_register_set_curbuf(findbuf);
    nvim_register_set_curwin_buffer(findbuf);
    nvim_register_set_virtual_op_from_curwin();

    // Adjust col from 1-based external to 0-based internal.
    (*p1).col -= 1;
    (*p2).col -= 1;

    // Swap if p1 > p2.
    if !pos_lt(*p1, *p2) {
        std::ptr::swap(p1, p2);
    }

    if *region_type == K_MT_CHAR_WISE {
        if is_exclusive && !pos_equal(*p1, *p2) {
            // When backing up to previous line, inclusive becomes false.
            *inclusive = !unadjust_for_sel_inner(p2);
        }
        // If p2 is on NUL (end of line) and not virtual, inclusive becomes false.
        if *inclusive
            && nvim_register_get_virtual_op() == 0 /* kFalse */
            && *ml_get_pos(p2) == 0
        // NUL
        {
            *inclusive = false;
        }
    } else if *region_type == K_MT_BLOCK_WISE {
        let mut sc1: c_int = 0;
        let mut ec1: c_int = 0;
        let mut sc2: c_int = 0;
        let mut ec2: c_int = 0;
        nvim_register_getvvcol(p1, &raw mut sc1, &raw mut ec1);
        nvim_register_getvvcol(p2, &raw mut sc2, &raw mut ec2);

        let start_vcol = sc1.min(sc2);
        let end_vcol = if block_width > 0 {
            start_vcol + block_width - 1
        } else if is_exclusive && ec1 < sc2 && 0 < sc2 && ec2 > ec1 {
            sc2 - 1
        } else {
            ec1.max(ec2)
        };

        nvim_oap_set_for_blockwise(oap as *mut c_void, p1, p2, start_vcol, end_vcol);
    }

    // Include the trailing byte of a multi-byte char.
    let l = utfc_ptr2len(ml_get_pos(p2));
    if l > 1 {
        (*p2).col += l - 1;
    }

    true
}

/// pos_T less-than (lnum first, then col).
fn pos_lt(a: PosT, b: PosT) -> bool {
    a.lnum < b.lnum || (a.lnum == b.lnum && a.col < b.col)
}

/// pos_T equality.
fn pos_equal(a: PosT, b: PosT) -> bool {
    a.lnum == b.lnum && a.col == b.col
}

/// Render a `BlockDef` into a heap-allocated NUL-terminated string.
/// Ownership is transferred to the caller (must be freed with xfree).
unsafe fn block_def_to_string(bd: *const BlockDef) -> *mut c_char {
    let size = ((*bd).startspaces + (*bd).endspaces + (*bd).textlen) as usize;
    let ret = xmallocz(size) as *mut c_char;
    let mut p = ret;
    std::ptr::write_bytes(p, b' ', (*bd).startspaces as usize);
    p = p.add((*bd).startspaces as usize);
    std::ptr::copy((*bd).textstart, p, (*bd).textlen as usize);
    p = p.add((*bd).textlen as usize);
    std::ptr::write_bytes(p, b' ', (*bd).endspaces as usize);
    // xmallocz already wrote the NUL terminator.
    ret
}

/// "getregion()" function — returns a list of strings from a region.
///
/// # Safety
///
/// Caller must provide valid pointers to typval_T arrays.
#[unsafe(export_name = "f_getregion")]
pub unsafe extern "C" fn rs_f_getregion(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let save_curbuf = nvim_register_get_curbuf();
    let save_virtual = nvim_register_get_virtual_op();

    let mut p1 = PosT::default();
    let mut p2 = PosT::default();
    let mut inclusive = true;
    let mut region_type: c_int = K_MT_UNKNOWN;
    let mut oap_storage: OpaqueOap = [0u8; OPARG_T_SIZE];

    if !region_resolve(
        argvars,
        rettv,
        &raw mut p1,
        &raw mut p2,
        &raw mut inclusive,
        &raw mut region_type,
        &raw mut oap_storage,
    ) {
        return;
    }

    let oap = oap_storage.as_mut_ptr() as *mut c_void;
    let result_list = (*tv_at_mut(rettv, 0)).as_list();

    let mut lnum = p1.lnum;
    while lnum <= p2.lnum {
        let akt: *mut c_char = if region_type == K_MT_LINE_WISE {
            xstrdup(ml_get(lnum))
        } else if region_type == K_MT_BLOCK_WISE {
            let mut bd = BlockDef::zeroed();
            block_prep(oap, &raw mut bd, lnum, false);
            block_def_to_string(&raw const bd)
        } else if p1.lnum < lnum && lnum < p2.lnum {
            xstrdup(ml_get(lnum))
        } else {
            let mut bd = BlockDef::zeroed();
            charwise_block_prep(p1, p2, &raw mut bd, lnum, inclusive);
            block_def_to_string(&raw const bd)
        };

        tv_list_append_allocated_string(result_list, akt);
        lnum += 1;
    }

    nvim_register_set_curbuf(save_curbuf);
    nvim_register_set_curwin_buffer(save_curbuf);
    nvim_register_set_virtual_op(save_virtual);
}

/// Append `[[buf,lnum,col,coladd],[buf,lnum,col,coladd]]` to the result list.
///
/// # Safety
///
/// `result_list` must be a valid list_T*.
unsafe fn add_regionpos_range(result_list: *mut c_void, fnum: c_int, ret_p1: PosT, ret_p2: PosT) {
    let l1 = tv_list_alloc(2);
    tv_list_append_list(result_list, l1);

    let l2 = tv_list_alloc(4);
    tv_list_append_list(l1, l2);
    let l3 = tv_list_alloc(4);
    tv_list_append_list(l1, l3);

    tv_list_append_number(l2, i64::from(fnum));
    tv_list_append_number(l2, i64::from(ret_p1.lnum));
    tv_list_append_number(l2, i64::from(ret_p1.col));
    tv_list_append_number(l2, i64::from(ret_p1.coladd));

    tv_list_append_number(l3, i64::from(fnum));
    tv_list_append_number(l3, i64::from(ret_p2.lnum));
    tv_list_append_number(l3, i64::from(ret_p2.col));
    tv_list_append_number(l3, i64::from(ret_p2.coladd));
}

/// "getregionpos()" function — returns positions of a region.
///
/// # Safety
///
/// Caller must provide valid pointers to typval_T arrays.
#[unsafe(export_name = "f_getregionpos")]
pub unsafe extern "C" fn rs_f_getregionpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let save_curbuf = nvim_register_get_curbuf();
    let save_virtual = nvim_register_get_virtual_op();

    let mut p1 = PosT::default();
    let mut p2 = PosT::default();
    let mut inclusive = true;
    let mut region_type: c_int = K_MT_UNKNOWN;
    let mut oap_storage: OpaqueOap = [0u8; OPARG_T_SIZE];

    if !region_resolve(
        argvars,
        rettv,
        &raw mut p1,
        &raw mut p2,
        &raw mut inclusive,
        &raw mut region_type,
        &raw mut oap_storage,
    ) {
        return;
    }

    let oap = oap_storage.as_mut_ptr() as *mut c_void;

    let allow_eol: bool = if (*tv_at(argvars, 2)).v_type == VAR_DICT {
        let d = (*tv_at(argvars, 2)).as_dict();
        tv_dict_get_bool(d, c"eol".as_ptr(), 0) != 0
    } else {
        false
    };

    let result_list = (*tv_at_mut(rettv, 0)).as_list();
    let fnum = nvim_register_curbuf_fnum();

    let mut lnum = p1.lnum;
    while lnum <= p2.lnum {
        let line: *mut c_char = ml_get(lnum);
        let line_len: c_int = ml_get_len(lnum);

        let (ret_p1, ret_p2): (PosT, PosT) = if region_type == K_MT_LINE_WISE {
            (
                PosT {
                    lnum,
                    col: 1,
                    coladd: 0,
                },
                PosT {
                    lnum,
                    col: MAXCOL,
                    coladd: 0,
                },
            )
        } else {
            let mut bd = BlockDef::zeroed();
            if region_type == K_MT_BLOCK_WISE {
                block_prep(oap, &raw mut bd, lnum, false);
            } else {
                charwise_block_prep(p1, p2, &raw mut bd, lnum, inclusive);
            }

            let oap_start_vcol = nvim_oap_get_start_vcol(oap);

            let mut rp1 = PosT::default();
            let mut rp2 = PosT::default();

            if bd.is_one_char != 0 {
                if region_type == K_MT_BLOCK_WISE {
                    rp1.col = mb_prevptr(line, bd.textstart).offset_from(line) as c_int + 1;
                    rp1.coladd = bd.start_char_vcols - (bd.start_vcol - oap_start_vcol);
                } else {
                    rp1.col = p1.col + 1;
                    rp1.coladd = p1.coladd;
                }
            } else if region_type == K_MT_BLOCK_WISE && oap_start_vcol > bd.start_vcol {
                rp1.col = MAXCOL;
                rp1.coladd = oap_start_vcol - bd.start_vcol;
                bd.is_one_char = 1; // bd.is_oneChar = true (flag)
            } else if bd.startspaces > 0 {
                rp1.col = mb_prevptr(line, bd.textstart).offset_from(line) as c_int + 1;
                rp1.coladd = bd.start_char_vcols - bd.startspaces;
            } else {
                rp1.col = bd.textcol + 1;
                rp1.coladd = 0;
            }

            if bd.is_one_char != 0 {
                rp2.col = rp1.col;
                rp2.coladd = rp1.coladd + bd.startspaces + bd.endspaces;
            } else if bd.endspaces > 0 {
                rp2.col = bd.textcol + bd.textlen + 1;
                rp2.coladd = bd.endspaces;
            } else {
                rp2.col = bd.textcol + bd.textlen;
                rp2.coladd = 0;
            }

            (rp1, rp2)
        };

        // Clamp ret_p1.
        let mut ret_p1 = ret_p1;
        let mut ret_p2 = ret_p2;
        if !allow_eol && ret_p1.col > line_len {
            ret_p1.col = 0;
            ret_p1.coladd = 0;
        } else if ret_p1.col > line_len + 1 {
            ret_p1.col = line_len + 1;
        }

        // Clamp ret_p2.
        if !allow_eol && ret_p2.col > line_len {
            ret_p2.col = if ret_p1.col == 0 { 0 } else { line_len };
            ret_p2.coladd = 0;
        } else if ret_p2.col > line_len + 1 {
            ret_p2.col = line_len + 1;
        }

        ret_p1.lnum = lnum;
        ret_p2.lnum = lnum;

        add_regionpos_range(result_list, fnum, ret_p1, ret_p2);

        lnum += 1;
    }

    nvim_register_set_curbuf(save_curbuf);
    nvim_register_set_curwin_buffer(save_curbuf);
    nvim_register_set_virtual_op(save_virtual);
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
