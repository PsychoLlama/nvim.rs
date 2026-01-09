//! Regular expression utilities for Neovim
//!
//! This crate provides Rust implementations of regex helper functions,
//! wrapping the Vim regex engines which remain in C.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

pub mod api;
pub mod bt_compile;
pub mod bt_exec;
pub mod bt_opcodes;
pub mod bt_parse;
pub mod bt_state;
pub mod char_class;
pub mod decompose;
pub mod exec_state;
pub mod line_fetch;
pub mod multiline;
pub mod nfa_compile;
pub mod nfa_exec;
pub mod nfa_match;
pub mod nfa_pattern;
pub mod nfa_states;
pub mod parser;
pub mod regsub;
pub mod scanner;
pub mod special;

pub use api::{
    rs_engine_to_int, rs_get_regflags, rs_int_to_engine, rs_is_bt_prog, rs_is_nfa_prog,
    rs_prog_in_use, rs_regmatch_get_endp, rs_regmatch_get_startp, rs_regmatch_set_endp,
    rs_regmatch_set_startp, rs_regmmatch_get_end_col, rs_regmmatch_get_end_lnum,
    rs_regmmatch_get_start_col, rs_regmmatch_get_start_lnum, rs_regprog_get_engine,
    rs_set_prog_in_use,
};
pub use bt_compile::{
    rs_bt_chain, rs_bt_compiler_free, rs_bt_compiler_is_too_long, rs_bt_compiler_new,
    rs_bt_compiler_size, rs_bt_compiler_start, rs_bt_emit_byte, rs_bt_emit_node,
    rs_bt_emit_node_arg, rs_bt_insert_node, rs_bt_next, rs_bt_op, rs_bt_operand, rs_bt_set_next,
};
pub use bt_exec::{
    rs_bt_backtrack_empty, rs_bt_match_state_advance, rs_bt_match_state_at_bol,
    rs_bt_match_state_at_eol, rs_bt_match_state_cleanup, rs_bt_match_state_clear_submatches,
    rs_bt_match_state_current_byte, rs_bt_match_state_free, rs_bt_match_state_get_endp,
    rs_bt_match_state_get_startp, rs_bt_match_state_new, rs_bt_match_state_new_multi,
    rs_bt_match_state_set_endp, rs_bt_match_state_set_input, rs_bt_match_state_set_startp,
    rs_bt_pop_backtrack, rs_bt_push_backtrack, rs_bt_regrepeat,
};
pub use bt_state::{
    rs_backpos_clear, rs_backpos_free, rs_backpos_new, rs_regstack_clear, rs_regstack_free,
    rs_regstack_is_empty, rs_regstack_new,
};
pub use char_class::rs_get_char_class;
pub use decompose::rs_mb_decompose;
pub use exec_state::{
    rs_exec_state_advance, rs_exec_state_at_bol, rs_exec_state_at_eol, rs_exec_state_current_byte,
    rs_exec_state_free, rs_exec_state_init_multi, rs_exec_state_init_single,
    rs_exec_state_load_from_rex, rs_exec_state_save_to_rex, rs_rex_in_use, rs_rex_set_in_use,
};
pub use line_fetch::{
    rs_line_cache_free, rs_line_cache_invalidate, rs_line_cache_new, rs_line_fetcher_free,
    rs_line_fetcher_from_rex, rs_line_fetcher_get_len, rs_line_fetcher_get_line,
    rs_line_fetcher_invalidate, rs_line_fetcher_new, rs_reg_getline, rs_reg_getline_len,
};
pub use multiline::{
    rs_multi_match_context_free, rs_multi_match_context_new, rs_multi_match_context_set_fetcher,
    rs_multi_match_context_set_prog, rs_multi_match_context_set_start,
    rs_multi_match_result_end_col, rs_multi_match_result_end_lnum, rs_multi_match_result_free,
    rs_multi_match_result_matched, rs_multi_match_result_new, rs_multi_match_result_start_col,
    rs_multi_match_result_start_lnum, rs_regexec_multi,
};
pub use nfa_compile::{
    rs_alloc_state, rs_append, rs_frag_new, rs_frag_stack_free, rs_frag_stack_new,
    rs_frag_stack_pop, rs_frag_stack_push, rs_list1, rs_patch, rs_state_allocator_count,
    rs_state_allocator_free, rs_state_allocator_new, rs_state_allocator_reset,
};
pub use nfa_exec::{rs_addstate, rs_addstate_here};
pub use nfa_match::{
    rs_clear_subs, rs_copy_pim, rs_copy_subs, rs_init_thread, rs_is_match_state, rs_is_split_state,
    rs_list_has_match, rs_mark_state_in_list, rs_match_context_advance, rs_match_context_at_eol,
    rs_match_context_current_byte, rs_match_context_new, rs_match_context_new_multi,
    rs_pim_matched, rs_pim_needs_exec, rs_set_pim_matched, rs_set_pim_nomatch, rs_state_in_list,
};
pub use nfa_pattern::{rs_nfa_free_postfix, rs_nfa_parse_pattern};
pub use nfa_states::{
    rs_check_char_class, rs_nfa_get_subexpr_idx, rs_nfa_is_char_class, rs_nfa_is_position_match,
    rs_nfa_is_posix_class, rs_nfa_list_clear, rs_nfa_list_count, rs_nfa_list_free, rs_nfa_list_get,
    rs_nfa_list_grow, rs_nfa_list_init, rs_nfa_list_next_id, rs_nfa_recognize_char_class,
    rs_ptrlist_append, rs_ptrlist_patch, rs_ptrlist_single,
};
pub use parser::{
    rs_nfa_re_num_cmp, rs_re_get_uint16, rs_re_get_uint32, rs_re_num_cmp, rs_re_put_uint16,
    rs_re_put_uint32, rs_read_limits, rs_use_multibytecode,
};
pub use regsub::{
    rs_do_lower, rs_do_upper, rs_is_k_special, rs_sub_context_has_room, rs_sub_context_new,
    rs_sub_context_output_len, rs_utf8_char2bytes, rs_utf8_char2len, rs_utf8_char_len,
    rs_utf8_ptr2char,
};
pub use scanner::{
    rs_getchr, rs_initchr, rs_peekchr, rs_skipchr, rs_skipchr_keepstart, rs_ungetchr,
};
pub use special::{
    rs_match_bof, rs_match_col, rs_match_cursor, rs_match_eof, rs_match_lnum, rs_match_vcol,
    rs_parse_num_cmp,
};

use std::ffi::{c_int, c_void};
use std::sync::OnceLock;

// =============================================================================
// Constants
// =============================================================================

/// ASCII control characters used in backslash_trans
const CAR: c_int = 13; // Carriage return
const TAB: c_int = 9; // Tab
const ESC: c_int = 27; // Escape
const BS: c_int = 8; // Backspace

/// Magic character offset (negative chars are magic)
const MAGIC_OFFSET: c_int = 256;

/// Return values for re_multi_type
/// Multi-operator type: not a multi operator.
pub const NOT_MULTI: c_int = 0;
const MULTI_ONE: c_int = 1;
const MULTI_MULT: c_int = 2;

/// regflags values
const RF_HASNL: u32 = 4;

/// Character class flags for the class table
const RI_DIGIT: i16 = 0x01;
const RI_HEX: i16 = 0x02;
const RI_OCTAL: i16 = 0x04;
const RI_WORD: i16 = 0x08;
const RI_HEAD: i16 = 0x10;
const RI_ALPHA: i16 = 0x20;
const RI_LOWER: i16 = 0x40;
const RI_UPPER: i16 = 0x80;
const RI_WHITE: i16 = 0x100;

/// Magic modes for regex patterns
const MAGIC_NONE: c_int = 1; // \V very nomagic
const MAGIC_OFF: c_int = 2; // \M or magic off
const MAGIC_ON: c_int = 3; // \m or magic (default)
const MAGIC_ALL: c_int = 4; // \v very magic

// Re-export CLASS_NONE for use within this module
use char_class::CLASS_NONE;

/// Characters always special in [] range after '\'
const REGEXP_INRANGE: &[u8] = b"]^-n\\";

/// Abbreviation characters after '\'
const REGEXP_ABBR: &[u8] = b"nrtebdoxuU";

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to regprog_T (compiled regular expression program).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegprogHandle(*mut std::ffi::c_void);

impl RegprogHandle {
    /// Create a handle from a raw pointer.
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to regmatch_T (single-line match result).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegmatchHandle(*mut std::ffi::c_void);

impl RegmatchHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to regmmatch_T (multi-line match result).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegmmatchHandle(*mut std::ffi::c_void);

impl RegmmatchHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to win_T (window).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinHandle(*mut std::ffi::c_void);

impl WinHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to buf_T (buffer).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufHandle(*mut std::ffi::c_void);

impl BufHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to lpos_T (line/column position).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LposHandle(*mut std::ffi::c_void);

impl LposHandle {
    #[inline]
    pub const fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// Type aliases for C types
#[allow(dead_code)]
type LineNr = c_int; // linenr_T
#[allow(dead_code)]
type ColNr = c_int; // colnr_T

// =============================================================================
// FFI Declarations
// =============================================================================

#[allow(dead_code)] // Phase 4 accessors are infrastructure for future phases
extern "C" {
    /// Get the regflags field from a regprog_T.
    fn nvim_regprog_get_regflags(prog: RegprogHandle) -> c_int;

    // UTF-8 functions
    /// Get byte length of UTF-8 character at pointer (including composing chars).
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    /// Get byte length of UTF-8 character at pointer (base character only).
    fn utf_ptr2len(p: *const c_char) -> c_int;

    /// Get Unicode codepoint from UTF-8 pointer.
    fn utf_ptr2char(p: *const c_char) -> c_int;

    /// Get reg_cpo_lit flag (whether 'cpoptions' contains 'l').
    fn nvim_get_reg_cpo_lit() -> c_int;

    /// Get character class for POSIX class name.
    fn nvim_get_char_class(pp: *mut *mut c_char) -> c_int;

    // Phase 3: Substitution helpers
    /// Get previous substitution pattern.
    fn nvim_get_reg_prev_sub() -> *mut c_char;

    /// Get previous substitution pattern length.
    fn nvim_get_reg_prev_sublen() -> usize;

    /// Set previous substitution pattern (takes ownership of s).
    fn nvim_set_reg_prev_sub(s: *mut c_char, len: usize);

    /// Allocate memory.
    fn xmalloc(size: usize) -> *mut c_char;

    /// Free memory.
    fn xfree(ptr: *mut c_void);

    /// Allocate and copy string (with length).
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    /// Display error message.
    fn emsg(s: *const c_char);

    // =========================================================================
    // Phase 4: rex structure accessors
    // =========================================================================

    // Current position accessors
    fn nvim_rex_get_lnum() -> LineNr;
    fn nvim_rex_set_lnum(lnum: LineNr);
    fn nvim_rex_get_line() -> *mut u8;
    fn nvim_rex_set_line(line: *mut u8);
    fn nvim_rex_get_input() -> *mut u8;
    fn nvim_rex_set_input(input: *mut u8);

    // Match state accessors
    fn nvim_rex_get_reg_match() -> RegmatchHandle;
    fn nvim_rex_set_reg_match(m: RegmatchHandle);
    fn nvim_rex_get_reg_mmatch() -> RegmmatchHandle;
    fn nvim_rex_set_reg_mmatch(m: RegmmatchHandle);

    // Submatch position accessors
    fn nvim_rex_get_reg_startp() -> *mut *mut u8;
    fn nvim_rex_set_reg_startp(p: *mut *mut u8);
    fn nvim_rex_get_reg_endp() -> *mut *mut u8;
    fn nvim_rex_set_reg_endp(p: *mut *mut u8);
    fn nvim_rex_get_reg_startpos() -> LposHandle;
    fn nvim_rex_set_reg_startpos(p: LposHandle);
    fn nvim_rex_get_reg_endpos() -> LposHandle;
    fn nvim_rex_set_reg_endpos(p: LposHandle);

    // Buffer/window context accessors
    fn nvim_rex_get_reg_win() -> WinHandle;
    fn nvim_rex_set_reg_win(win: WinHandle);
    fn nvim_rex_get_reg_buf() -> BufHandle;
    fn nvim_rex_set_reg_buf(buf: BufHandle);
    fn nvim_rex_get_reg_firstlnum() -> LineNr;
    fn nvim_rex_set_reg_firstlnum(lnum: LineNr);
    fn nvim_rex_get_reg_maxline() -> LineNr;
    fn nvim_rex_set_reg_maxline(lnum: LineNr);

    // Flag accessors
    fn nvim_rex_get_reg_ic() -> bool;
    fn nvim_rex_set_reg_ic(ic: bool);
    fn nvim_rex_get_reg_icombine() -> bool;
    fn nvim_rex_set_reg_icombine(ic: bool);
    fn nvim_rex_get_reg_line_lbr() -> bool;
    fn nvim_rex_set_reg_line_lbr(lbr: bool);
    fn nvim_rex_get_reg_nobreak() -> bool;
    fn nvim_rex_set_reg_nobreak(nb: bool);
    fn nvim_rex_get_reg_maxcol() -> ColNr;
    fn nvim_rex_set_reg_maxcol(col: ColNr);

    // Subexpression clearing flags
    fn nvim_rex_get_need_clear_subexpr() -> c_int;
    fn nvim_rex_set_need_clear_subexpr(v: c_int);
    fn nvim_rex_get_need_clear_zsubexpr() -> c_int;
    fn nvim_rex_set_need_clear_zsubexpr(v: c_int);

    // NFA engine state accessors
    fn nvim_rex_get_nfa_has_zend() -> c_int;
    fn nvim_rex_set_nfa_has_zend(v: c_int);
    fn nvim_rex_get_nfa_has_backref() -> c_int;
    fn nvim_rex_set_nfa_has_backref(v: c_int);
    fn nvim_rex_get_nfa_nsubexpr() -> c_int;
    fn nvim_rex_set_nfa_nsubexpr(v: c_int);
    fn nvim_rex_get_nfa_listid() -> c_int;
    fn nvim_rex_set_nfa_listid(v: c_int);
    fn nvim_rex_get_nfa_alt_listid() -> c_int;
    fn nvim_rex_set_nfa_alt_listid(v: c_int);
    fn nvim_rex_get_nfa_has_zsubexpr() -> c_int;
    fn nvim_rex_set_nfa_has_zsubexpr(v: c_int);

    // rex_in_use flag
    fn nvim_rex_in_use() -> bool;
    fn nvim_rex_set_in_use(in_use: bool);

    // =========================================================================
    // Phase 5: Parse state accessors
    // =========================================================================

    // regparse - input scan pointer
    fn nvim_parse_get_regparse() -> *mut c_char;
    fn nvim_parse_set_regparse(p: *mut c_char);

    // prevchr_len - byte length of previous char
    fn nvim_parse_get_prevchr_len() -> c_int;
    fn nvim_parse_set_prevchr_len(len: c_int);

    // curchr - currently parsed character
    fn nvim_parse_get_curchr() -> c_int;
    fn nvim_parse_set_curchr(c: c_int);

    // prevchr - previous character
    fn nvim_parse_get_prevchr() -> c_int;
    fn nvim_parse_set_prevchr(c: c_int);

    // prevprevchr - previous-previous character
    fn nvim_parse_get_prevprevchr() -> c_int;
    fn nvim_parse_set_prevprevchr(c: c_int);

    // nextchr - used for ungetchr()
    fn nvim_parse_get_nextchr() -> c_int;
    fn nvim_parse_set_nextchr(c: c_int);

    // at_start - true when on first character
    fn nvim_parse_get_at_start() -> c_int;
    fn nvim_parse_set_at_start(v: c_int);

    // prev_at_start - true when on second character
    fn nvim_parse_get_prev_at_start() -> c_int;
    fn nvim_parse_set_prev_at_start(v: c_int);

    // regnpar - parenthesis count
    fn nvim_parse_get_regnpar() -> c_int;
    fn nvim_parse_set_regnpar(n: c_int);

    // reg_magic - magicness of pattern
    fn nvim_parse_get_reg_magic() -> c_int;
    fn nvim_parse_set_reg_magic(m: c_int);

    // Helper functions for number parsing
    fn nvim_hex2nr(c: c_int) -> c_int;
    fn nvim_ascii_isxdigit(c: c_int) -> c_int;
}

// MAXCOL constant - maximum column number
const MAXCOL: usize = 0x7fff_ffff;

// =============================================================================
// Magic Functions
// =============================================================================

/// Check if a character is magic (negative value).
#[inline]
const fn is_magic(x: c_int) -> bool {
    x < 0
}

/// Convert a magic character back to its ASCII value.
#[inline]
const fn un_magic(x: c_int) -> c_int {
    x + MAGIC_OFFSET
}

/// Convert an ASCII character to its magic form.
#[inline]
const fn magic(x: c_int) -> c_int {
    x - MAGIC_OFFSET
}

/// Remove magic from a character.
/// If it's magic, convert it back to regular. Otherwise return as-is.
#[inline]
const fn no_magic_impl(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        x
    }
}

/// Toggle the magic state of a character.
/// If magic, make it regular. If regular, make it magic.
#[inline]
const fn toggle_magic_impl(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        magic(x)
    }
}

/// Return the type of "multi" operator for character c.
/// NOT_MULTI (0) if not a multi operator.
/// MULTI_ONE (1) if single multi operator (@, =, ?).
/// MULTI_MULT (2) if multi multi operator (*, +, {).
#[inline]
pub const fn re_multi_type_impl(c: c_int) -> c_int {
    // Magic('@') = '@' - 256 = 64 - 256 = -192
    // Magic('=') = '=' - 256 = 61 - 256 = -195
    // Magic('?') = '?' - 256 = 63 - 256 = -193
    // Magic('*') = '*' - 256 = 42 - 256 = -214
    // Magic('+') = '+' - 256 = 43 - 256 = -213
    // Magic('{') = '{' - 256 = 123 - 256 = -133
    let magic_at = magic(b'@' as c_int);
    let magic_eq = magic(b'=' as c_int);
    let magic_q = magic(b'?' as c_int);
    let magic_star = magic(b'*' as c_int);
    let magic_plus = magic(b'+' as c_int);
    let magic_brace = magic(b'{' as c_int);

    if c == magic_at || c == magic_eq || c == magic_q {
        MULTI_ONE
    } else if c == magic_star || c == magic_plus || c == magic_brace {
        MULTI_MULT
    } else {
        NOT_MULTI
    }
}

/// Translate '\x' to its control character, except "\n" which is Magic.
#[inline]
const fn backslash_trans_impl(c: c_int) -> c_int {
    match c as u8 {
        b'r' => CAR,
        b't' => TAB,
        b'e' => ESC,
        b'b' => BS,
        _ => c,
    }
}

// =============================================================================
// Character Class Table
// =============================================================================

/// Initialize the character class table.
fn init_class_tab() -> [i16; 256] {
    let mut tab = [0i16; 256];

    for (i, entry) in tab.iter_mut().enumerate() {
        *entry = match i as u8 {
            b'0'..=b'7' => RI_DIGIT | RI_HEX | RI_OCTAL | RI_WORD,
            b'8'..=b'9' => RI_DIGIT | RI_HEX | RI_WORD,
            b'a'..=b'f' => RI_HEX | RI_WORD | RI_HEAD | RI_ALPHA | RI_LOWER,
            b'g'..=b'z' => RI_WORD | RI_HEAD | RI_ALPHA | RI_LOWER,
            b'A'..=b'F' => RI_HEX | RI_WORD | RI_HEAD | RI_ALPHA | RI_UPPER,
            b'G'..=b'Z' => RI_WORD | RI_HEAD | RI_ALPHA | RI_UPPER,
            b'_' => RI_WORD | RI_HEAD,
            b' ' | b'\t' => RI_WHITE,
            _ => 0,
        };
    }

    tab
}

/// Get a reference to the character class table (lazily initialized).
fn class_tab() -> &'static [i16; 256] {
    static CLASS_TAB: OnceLock<[i16; 256]> = OnceLock::new();
    CLASS_TAB.get_or_init(init_class_tab)
}

/// Check if character is a digit (0-9).
#[inline]
fn ri_digit(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_DIGIT) != 0
}

/// Check if character is a hexadecimal digit (0-9, a-f, A-F).
#[inline]
fn ri_hex(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_HEX) != 0
}

/// Check if character is an octal digit (0-7).
#[inline]
fn ri_octal(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_OCTAL) != 0
}

/// Check if character is a word character (a-z, A-Z, 0-9, _).
#[inline]
fn ri_word(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_WORD) != 0
}

/// Check if character can start an identifier (a-z, A-Z, _).
#[inline]
fn ri_head(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_HEAD) != 0
}

/// Check if character is alphabetic (a-z, A-Z).
#[inline]
fn ri_alpha(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_ALPHA) != 0
}

/// Check if character is lowercase (a-z).
#[inline]
fn ri_lower(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_LOWER) != 0
}

/// Check if character is uppercase (A-Z).
#[inline]
fn ri_upper(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_UPPER) != 0
}

/// Check if character is whitespace (space or tab).
#[inline]
fn ri_white(c: c_int) -> bool {
    (c as u32) < 256 && (class_tab()[c as usize] & RI_WHITE) != 0
}

// =============================================================================
// Query Functions
// =============================================================================

/// Check if compiled regex can match a newline.
///
/// # Safety
/// The handle must point to a valid regprog_T.
#[inline]
unsafe fn re_multiline_impl(prog: RegprogHandle) -> bool {
    let flags = nvim_regprog_get_regflags(prog);
    (flags as u32 & RF_HASNL) != 0
}

// =============================================================================
// Phase 2: Pattern Parsing Utilities
// =============================================================================

use std::ffi::c_char;

/// Helper to check if a byte is in a byte slice.
#[inline]
fn byte_in_slice(b: u8, slice: &[u8]) -> bool {
    slice.contains(&b)
}

/// Check for an equivalence class name "[=a=]". `p` points to the '['.
/// Returns the character representing the class, or 0 if not recognized.
/// Advances `p` past the item if recognized.
///
/// # Safety
/// `p` must point to a valid null-terminated string.
unsafe fn get_equi_class_impl(p: *mut *mut c_char) -> c_int {
    let ptr = *p;

    // Check for [= pattern
    if *ptr.add(1) as u8 != b'=' || *ptr.add(2) == 0 {
        return 0;
    }

    // Get the character length at position 2
    let char_len = utfc_ptr2len(ptr.add(2)) as usize;

    // Check for =] after the character
    if *ptr.add(char_len + 2) as u8 == b'=' && *ptr.add(char_len + 3) as u8 == b']' {
        let c = utf_ptr2char(ptr.add(2));
        *p = ptr.add(char_len + 4);
        return c;
    }

    0
}

/// Check for a collating element "[.a.]". `p` points to the '['.
/// Returns the character, or 0 if not recognized.
/// Advances `p` past the item if recognized.
///
/// # Safety
/// `p` must point to a valid null-terminated string.
unsafe fn get_coll_element_impl(p: *mut *mut c_char) -> c_int {
    let ptr = *p;

    // Check for [. pattern
    if *ptr == 0 || *ptr.add(1) as u8 != b'.' || *ptr.add(2) == 0 {
        return 0;
    }

    // Get the character length at position 2
    let char_len = utfc_ptr2len(ptr.add(2)) as usize;

    // Check for .] after the character
    if *ptr.add(char_len + 2) as u8 == b'.' && *ptr.add(char_len + 3) as u8 == b']' {
        let c = utf_ptr2char(ptr.add(2));
        *p = ptr.add(char_len + 4);
        return c;
    }

    0
}

/// Skip over a "[]" range. `p` must point to the character after the '['.
/// Returns pointer to the matching ']', or the terminating NUL.
///
/// # Safety
/// `p` must point to a valid null-terminated string.
unsafe fn skip_anyof_impl(mut p: *mut c_char) -> *mut c_char {
    let reg_cpo_lit = nvim_get_reg_cpo_lit() != 0;

    // Complement of range
    if *p as u8 == b'^' {
        p = p.add(1);
    }

    // ] or - at start are literal
    if *p as u8 == b']' || *p as u8 == b'-' {
        p = p.add(1);
    }

    while *p != 0 && *p as u8 != b']' {
        let char_len = utfc_ptr2len(p) as usize;

        if char_len > 1 {
            // Multi-byte character
            p = p.add(char_len);
        } else if *p as u8 == b'-' {
            p = p.add(1);
            if *p as u8 != b']' && *p != 0 {
                // Skip the character after -
                let next_len = utfc_ptr2len(p) as usize;
                p = p.add(next_len.max(1));
            }
        } else if *p as u8 == b'\\' {
            let next_byte = *p.add(1) as u8;
            // Check if next char is in REGEXP_INRANGE or (if not cpo_lit) in REGEXP_ABBR
            if byte_in_slice(next_byte, REGEXP_INRANGE)
                || (!reg_cpo_lit && byte_in_slice(next_byte, REGEXP_ABBR))
            {
                p = p.add(2);
            } else {
                p = p.add(1);
            }
        } else if *p as u8 == b'[' {
            // Try character class, equivalence class, or collating element
            let mut pp = p;
            if nvim_get_char_class(&mut pp) == CLASS_NONE
                && get_equi_class_impl(&mut pp) == 0
                && get_coll_element_impl(&mut pp) == 0
                && *pp != 0
            {
                // Not a class, just a literal [
                p = p.add(1);
            } else {
                p = pp;
            }
        } else {
            p = p.add(1);
        }
    }

    p
}

/// Skip past regular expression.
/// Stop at end of string or where `delim` is found ('/', '?', etc).
/// Takes care of characters with a backslash in front.
/// Skips strings inside [ and ].
///
/// Returns pointer to the delimiter or end of string, and optionally
/// the effective magic mode via `magic_val`.
///
/// # Safety
/// `startp` must point to a valid null-terminated string.
unsafe fn skip_regexp_ex_impl(
    startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
    magic_val: *mut c_int,
) -> *mut c_char {
    let mut mymagic = if magic != 0 { MAGIC_ON } else { MAGIC_OFF };
    let mut p = startp;

    while *p != 0 {
        let byte = *p as u8;

        // Found end of regexp
        if byte as c_int == dirc {
            break;
        }

        // Check for character class
        if (byte == b'[' && mymagic >= MAGIC_ON)
            || (byte == b'\\' && *p.add(1) as u8 == b'[' && mymagic <= MAGIC_OFF)
        {
            p = skip_anyof_impl(p.add(1));
            if *p == 0 {
                break;
            }
        } else if byte == b'\\' && *p.add(1) != 0 {
            // Skip backslash and next character
            p = p.add(1);

            // Track magic mode changes
            let next = *p as u8;
            if next == b'v' {
                mymagic = MAGIC_ALL;
            } else if next == b'V' {
                mymagic = MAGIC_NONE;
            }
        }

        // Advance to next character
        let char_len = utfc_ptr2len(p) as usize;
        p = p.add(char_len.max(1));
    }

    if !magic_val.is_null() {
        *magic_val = mymagic;
    }

    p
}

/// Skip past regular expression (simple version).
///
/// # Safety
/// `startp` must point to a valid null-terminated string.
#[inline]
unsafe fn skip_regexp_impl(startp: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char {
    skip_regexp_ex_impl(startp, delim, magic, std::ptr::null_mut())
}

// =============================================================================
// Phase 3: Substitution Helpers
// =============================================================================

/// Error message for text too long
static E_RESULTING_TEXT_TOO_LONG: &[u8] = b"E1240: Resulting text too long\0";

/// Replace tildes in the pattern by the old pattern.
///
/// The tilde stands for the previous replacement pattern. If that previous
/// pattern also contains a ~ we should go back a step further.
///
/// Returns the (possibly new) source string. If a new string was allocated,
/// it must be freed by the caller. The returned pointer equals `source` if
/// no allocation occurred.
///
/// # Safety
/// `source` must point to a valid null-terminated string.
unsafe fn regtilde_impl(source: *mut c_char, magic: c_int, preview: bool) -> *mut c_char {
    let mut newsub = source;
    let mut newsublen: usize = 0;

    // Tilde pattern depends on magic mode
    let (tilde, tildelen): (&[u8], usize) = if magic != 0 {
        (b"~\0", 1)
    } else {
        (b"\\~\0", 2)
    };

    let mut error = false;
    let mut p = newsub;

    while *p != 0 {
        // Check for tilde pattern
        let tilde_ptr = tilde.as_ptr().cast::<c_char>();
        if libc::strncmp(p, tilde_ptr, tildelen) == 0 {
            let prefixlen = p.offset_from(newsub) as usize;
            let postfix = p.add(tildelen);

            if newsublen == 0 {
                newsublen = libc::strlen(newsub);
            }
            newsublen -= tildelen;
            let postfixlen = newsublen - prefixlen;

            let reg_prev_sub = nvim_get_reg_prev_sub();
            let reg_prev_sublen = nvim_get_reg_prev_sublen();
            let tmpsublen = prefixlen + reg_prev_sublen + postfixlen;

            if tmpsublen > 0 && !reg_prev_sub.is_null() {
                // Avoid making the text longer than MAXCOL
                if tmpsublen > MAXCOL {
                    emsg(E_RESULTING_TEXT_TOO_LONG.as_ptr().cast());
                    error = true;
                    break;
                }

                let tmpsub = xmalloc(tmpsublen + 1);
                // copy prefix
                libc::memmove(tmpsub.cast(), newsub.cast(), prefixlen);
                // interpret tilde - insert previous substitution
                libc::memmove(
                    tmpsub.add(prefixlen).cast(),
                    reg_prev_sub.cast(),
                    reg_prev_sublen,
                );
                // copy postfix (including NUL)
                libc::memmove(
                    tmpsub.add(prefixlen + reg_prev_sublen).cast(),
                    postfix.cast(),
                    postfixlen + 1,
                );

                if newsub != source {
                    // We allocated newsub before, free it
                    xfree(newsub.cast());
                }
                newsub = tmpsub;
                newsublen = tmpsublen;
                p = newsub.add(prefixlen + reg_prev_sublen);
            } else {
                // No previous sub, just remove the tilde
                libc::memmove(p.cast(), postfix.cast(), postfixlen + 1);
            }
            p = p.sub(1);
        } else {
            // Skip escaped characters
            if *p as u8 == b'\\' && *p.add(1) != 0 {
                p = p.add(1);
            }
            // Advance by UTF-8 character length
            let char_len = utfc_ptr2len(p) as usize;
            p = p.add(char_len.saturating_sub(1));
        }
        p = p.add(1);
    }

    if error {
        if newsub != source {
            xfree(newsub.cast());
        }
        return source;
    }

    // Only update reg_prev_sub when not previewing
    if !preview {
        newsublen = p.offset_from(newsub) as usize;
        if newsublen == 0 {
            nvim_set_reg_prev_sub(std::ptr::null_mut(), 0);
        } else {
            let new_prev = xstrnsave(newsub, newsublen);
            nvim_set_reg_prev_sub(new_prev, newsublen);
        }
    }

    newsub
}

// =============================================================================
// Phase 5: Number Parsing Functions
// =============================================================================

/// Get and return the value of a hex string at the current position.
/// Returns -1 if there is no valid hex number.
/// The regparse position is updated.
///
/// The parameter controls the maximum number of input characters:
/// - 2 for \%x20 sequence
/// - 4 for \%u20AC sequence
/// - 8 for \%U12345678 sequence
///
/// # Safety
/// regparse must point to a valid null-terminated string.
unsafe fn gethexchrs_impl(maxinputlen: c_int) -> i64 {
    let mut nr: i64 = 0;
    let mut regparse = nvim_parse_get_regparse();

    let mut i = 0;
    while i < maxinputlen {
        let c = *regparse as u8 as c_int;
        if nvim_ascii_isxdigit(c) == 0 {
            break;
        }
        nr <<= 4;
        nr |= nvim_hex2nr(c) as i64;
        regparse = regparse.add(1);
        i += 1;
    }

    nvim_parse_set_regparse(regparse);

    if i == 0 {
        -1
    } else {
        nr
    }
}

/// Get and return the value of a decimal string immediately after the
/// current position. Returns -1 for invalid. Consumes all digits.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
unsafe fn getdecchrs_impl() -> i64 {
    let mut nr: i64 = 0;
    let mut regparse = nvim_parse_get_regparse();

    let mut i = 0;
    loop {
        let c = *regparse as u8;
        if !c.is_ascii_digit() {
            break;
        }
        nr *= 10;
        nr += (c - b'0') as i64;
        regparse = regparse.add(1);
        nvim_parse_set_curchr(-1); // no longer valid
        i += 1;
    }

    nvim_parse_set_regparse(regparse);

    if i == 0 {
        -1
    } else {
        nr
    }
}

/// Get and return the value of an octal string immediately after the current
/// position. Returns -1 for invalid, or 0-255 for valid.
/// Smart enough to handle numbers > 377 correctly (e.g., 400 is treated as 40)
/// and doesn't treat 8 or 9 as recognized characters.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
unsafe fn getoctchrs_impl() -> i64 {
    let mut nr: i64 = 0;
    let mut regparse = nvim_parse_get_regparse();

    // Maximum 3 octal digits, and nr < 0o40 (32 decimal) to stay <= 255
    let mut i = 0;
    while i < 3 && nr < 0o40 {
        let c = *regparse as u8;
        if !(b'0'..=b'7').contains(&c) {
            break;
        }
        nr <<= 3;
        nr |= nvim_hex2nr(c as c_int) as i64;
        regparse = regparse.add(1);
        i += 1;
    }

    nvim_parse_set_regparse(regparse);

    if i == 0 {
        -1
    } else {
        nr
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Remove magic from a character.
#[no_mangle]
pub extern "C" fn rs_no_magic(x: c_int) -> c_int {
    no_magic_impl(x)
}

/// Toggle the magic state of a character.
#[no_mangle]
pub extern "C" fn rs_toggle_magic(x: c_int) -> c_int {
    toggle_magic_impl(x)
}

/// Return the type of multi operator.
#[no_mangle]
pub extern "C" fn rs_re_multi_type(c: c_int) -> c_int {
    re_multi_type_impl(c)
}

/// Translate backslash escape to control character.
#[no_mangle]
pub extern "C" fn rs_backslash_trans(c: c_int) -> c_int {
    backslash_trans_impl(c)
}

/// Check if character is a digit.
#[no_mangle]
pub extern "C" fn rs_ri_digit(c: c_int) -> c_int {
    c_int::from(ri_digit(c))
}

/// Check if character is a hex digit.
#[no_mangle]
pub extern "C" fn rs_ri_hex(c: c_int) -> c_int {
    c_int::from(ri_hex(c))
}

/// Check if character is an octal digit.
#[no_mangle]
pub extern "C" fn rs_ri_octal(c: c_int) -> c_int {
    c_int::from(ri_octal(c))
}

/// Check if character is a word character.
#[no_mangle]
pub extern "C" fn rs_ri_word(c: c_int) -> c_int {
    c_int::from(ri_word(c))
}

/// Check if character can start an identifier.
#[no_mangle]
pub extern "C" fn rs_ri_head(c: c_int) -> c_int {
    c_int::from(ri_head(c))
}

/// Check if character is alphabetic.
#[no_mangle]
pub extern "C" fn rs_ri_alpha(c: c_int) -> c_int {
    c_int::from(ri_alpha(c))
}

/// Check if character is lowercase.
#[no_mangle]
pub extern "C" fn rs_ri_lower(c: c_int) -> c_int {
    c_int::from(ri_lower(c))
}

/// Check if character is uppercase.
#[no_mangle]
pub extern "C" fn rs_ri_upper(c: c_int) -> c_int {
    c_int::from(ri_upper(c))
}

/// Check if character is whitespace.
#[no_mangle]
pub extern "C" fn rs_ri_white(c: c_int) -> c_int {
    c_int::from(ri_white(c))
}

/// Check if compiled regex can match a newline.
///
/// # Safety
/// The handle must point to a valid regprog_T.
#[no_mangle]
pub unsafe extern "C" fn rs_re_multiline(prog: RegprogHandle) -> c_int {
    c_int::from(re_multiline_impl(prog))
}

// -----------------------------------------------------------------------------
// Phase 2: Pattern Parsing FFI Exports
// -----------------------------------------------------------------------------

/// Check for an equivalence class name "[=a=]".
///
/// # Safety
/// `pp` must point to a valid pointer to a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_equi_class(pp: *mut *mut c_char) -> c_int {
    get_equi_class_impl(pp)
}

/// Check for a collating element "[.a.]".
///
/// # Safety
/// `pp` must point to a valid pointer to a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_coll_element(pp: *mut *mut c_char) -> c_int {
    get_coll_element_impl(pp)
}

/// Skip over a "[]" range.
///
/// # Safety
/// `p` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_anyof(p: *mut c_char) -> *mut c_char {
    skip_anyof_impl(p)
}

/// Skip past regular expression to delimiter.
///
/// # Safety
/// `startp` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_regexp(
    startp: *mut c_char,
    delim: c_int,
    magic: c_int,
) -> *mut c_char {
    skip_regexp_impl(startp, delim, magic)
}

/// Skip past regular expression with magic value tracking.
///
/// # Safety
/// `startp` must point to a valid null-terminated string.
/// `newp` if non-null must be valid for writes.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_regexp_ex(
    startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
    newp: *mut c_int,
) -> *mut c_char {
    skip_regexp_ex_impl(startp, dirc, magic, newp)
}

// -----------------------------------------------------------------------------
// Phase 3: Substitution FFI Exports
// -----------------------------------------------------------------------------

/// Replace tildes in the pattern by the old pattern.
///
/// Returns the (possibly new) source string. If a new string was allocated,
/// it must be freed by the caller.
///
/// # Safety
/// `source` must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_regtilde(
    source: *mut c_char,
    magic: c_int,
    preview: c_int,
) -> *mut c_char {
    regtilde_impl(source, magic, preview != 0)
}

// -----------------------------------------------------------------------------
// Phase 5: Number Parsing FFI Exports
// -----------------------------------------------------------------------------

/// Get and return the value of a hex string at the current position.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_gethexchrs(maxinputlen: c_int) -> i64 {
    gethexchrs_impl(maxinputlen)
}

/// Get and return the value of a decimal string at the current position.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_getdecchrs() -> i64 {
    getdecchrs_impl()
}

/// Get and return the value of an octal string at the current position.
///
/// # Safety
/// regparse must point to a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_getoctchrs() -> i64 {
    getoctchrs_impl()
}

// =============================================================================
// Phase 6: Public API Wrappers
// =============================================================================

/// Number of subexpressions supported (matches C NSUBEXP)
pub const NSUBEXP: usize = 10;

/// Regex compilation flags
pub mod re_flags {
    use std::ffi::c_int;

    /// Very nomagic (\V)
    pub const RE_MAGIC: c_int = 1;
    /// Case-insensitive
    pub const RE_NOCASE: c_int = 2;
    /// Pattern contains \ze
    pub const RE_HASNL: c_int = 4;
}

// C API function declarations
#[allow(dead_code)]
extern "C" {
    /// Compile a regular expression pattern.
    fn vim_regcomp(expr: *const c_char, re_flags: c_int) -> RegprogHandle;

    /// Free a compiled regular expression.
    fn vim_regfree(prog: RegprogHandle);

    /// Execute regex against a string (single-line).
    fn vim_regexec(rmp: *mut RegmatchRaw, line: *const c_char, col: ColNr) -> bool;

    /// Execute regex against a string with newline support.
    fn vim_regexec_nl(rmp: *mut RegmatchRaw, line: *const c_char, col: ColNr) -> bool;
}

/// Raw regmatch_T structure for FFI.
/// This matches the C regmatch_T layout.
#[repr(C)]
#[derive(Clone)]
pub struct RegmatchRaw {
    /// Compiled regex program
    pub regprog: RegprogHandle,
    /// Start positions for each submatch
    pub startp: [*mut c_char; NSUBEXP],
    /// End positions for each submatch
    pub endp: [*mut c_char; NSUBEXP],
    /// Match start without \zs
    pub rm_matchcol: ColNr,
    /// Ignore case flag
    pub rm_ic: bool,
}

impl Default for RegmatchRaw {
    fn default() -> Self {
        Self {
            regprog: RegprogHandle::from_ptr(std::ptr::null_mut()),
            startp: [std::ptr::null_mut(); NSUBEXP],
            endp: [std::ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

/// A compiled regular expression with RAII semantics.
///
/// This type owns the compiled regex program and automatically
/// frees it when dropped.
pub struct CompiledRegex {
    prog: RegprogHandle,
}

impl CompiledRegex {
    /// Compile a regular expression pattern.
    ///
    /// Returns `None` if compilation fails (invalid pattern).
    ///
    /// # Safety
    /// Must be called from a context where Neovim's memory allocator is available.
    #[inline]
    pub unsafe fn compile(pattern: *const c_char, flags: c_int) -> Option<Self> {
        let prog = vim_regcomp(pattern, flags);
        if prog.is_null() {
            None
        } else {
            Some(Self { prog })
        }
    }

    /// Get the underlying program handle.
    #[inline]
    pub fn handle(&self) -> RegprogHandle {
        self.prog
    }

    /// Check if the compiled regex can match a newline.
    ///
    /// # Safety
    /// The handle must be valid.
    #[inline]
    pub unsafe fn can_match_newline(&self) -> bool {
        re_multiline_impl(self.prog)
    }

    /// Execute this regex against a string.
    ///
    /// Returns match result if successful.
    ///
    /// # Safety
    /// `line` must point to a valid null-terminated string.
    #[inline]
    pub unsafe fn exec(
        &mut self,
        line: *const c_char,
        col: ColNr,
        ignore_case: bool,
    ) -> Option<MatchResult> {
        let mut rmp = RegmatchRaw {
            regprog: self.prog,
            rm_ic: ignore_case,
            ..Default::default()
        };

        if vim_regexec(&mut rmp, line, col) {
            // Update our handle in case it was reallocated
            self.prog = rmp.regprog;
            Some(MatchResult::from_raw(&rmp, line))
        } else {
            self.prog = rmp.regprog;
            None
        }
    }

    /// Execute this regex against a string, treating \n as line break.
    ///
    /// Returns match result if successful.
    ///
    /// # Safety
    /// `line` must point to a valid null-terminated string.
    #[inline]
    pub unsafe fn exec_nl(
        &mut self,
        line: *const c_char,
        col: ColNr,
        ignore_case: bool,
    ) -> Option<MatchResult> {
        let mut rmp = RegmatchRaw {
            regprog: self.prog,
            rm_ic: ignore_case,
            ..Default::default()
        };

        if vim_regexec_nl(&mut rmp, line, col) {
            self.prog = rmp.regprog;
            Some(MatchResult::from_raw(&rmp, line))
        } else {
            self.prog = rmp.regprog;
            None
        }
    }
}

impl Drop for CompiledRegex {
    fn drop(&mut self) {
        if !self.prog.is_null() {
            // SAFETY: We own this program and it's valid
            unsafe {
                vim_regfree(self.prog);
            }
        }
    }
}

/// Result of a successful regex match.
///
/// Contains the start and end positions of the overall match
/// and any captured subgroups.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Start offset of the full match (in bytes from line start)
    pub start: usize,
    /// End offset of the full match (in bytes from line start)
    pub end: usize,
    /// Start column without \zs
    pub match_col: ColNr,
    /// Submatch positions: (start, end) offsets, None if not matched
    pub submatches: [Option<(usize, usize)>; NSUBEXP],
}

impl MatchResult {
    /// Create a MatchResult from a raw regmatch_T.
    ///
    /// # Safety
    /// `rmp` must contain valid pointers from a successful match,
    /// and `line` must be the original line that was matched against.
    unsafe fn from_raw(rmp: &RegmatchRaw, line: *const c_char) -> Self {
        let mut submatches = [None; NSUBEXP];

        for (i, submatch) in submatches.iter_mut().enumerate() {
            if !rmp.startp[i].is_null() && !rmp.endp[i].is_null() {
                let start = rmp.startp[i].offset_from(line) as usize;
                let end = rmp.endp[i].offset_from(line) as usize;
                *submatch = Some((start, end));
            }
        }

        // The full match is submatch 0
        let (start, end) = submatches[0].unwrap_or((0, 0));

        Self {
            start,
            end,
            match_col: rmp.rm_matchcol,
            submatches,
        }
    }

    /// Get the matched text length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if the match is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get a specific submatch by index (0 = full match).
    #[inline]
    pub fn submatch(&self, n: usize) -> Option<(usize, usize)> {
        self.submatches.get(n).copied().flatten()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_functions() {
        // Test is_magic
        assert!(is_magic(-1));
        assert!(is_magic(-100));
        assert!(!is_magic(0));
        assert!(!is_magic(100));

        // Test magic/un_magic round-trip
        assert_eq!(un_magic(magic(65)), 65); // 'A'
        assert_eq!(un_magic(magic(42)), 42); // '*'
    }

    #[test]
    fn test_no_magic() {
        // Magic character should be unmagicked
        assert_eq!(no_magic_impl(magic(b'*' as c_int)), b'*' as c_int);
        // Non-magic should stay the same
        assert_eq!(no_magic_impl(b'a' as c_int), b'a' as c_int);
    }

    #[test]
    fn test_toggle_magic() {
        let star = b'*' as c_int;
        let magic_star = magic(star);

        // Toggle regular -> magic
        assert_eq!(toggle_magic_impl(star), magic_star);
        // Toggle magic -> regular
        assert_eq!(toggle_magic_impl(magic_star), star);
    }

    #[test]
    fn test_re_multi_type() {
        // Single multi operators
        assert_eq!(re_multi_type_impl(magic(b'@' as c_int)), MULTI_ONE);
        assert_eq!(re_multi_type_impl(magic(b'=' as c_int)), MULTI_ONE);
        assert_eq!(re_multi_type_impl(magic(b'?' as c_int)), MULTI_ONE);

        // Multi multi operators
        assert_eq!(re_multi_type_impl(magic(b'*' as c_int)), MULTI_MULT);
        assert_eq!(re_multi_type_impl(magic(b'+' as c_int)), MULTI_MULT);
        assert_eq!(re_multi_type_impl(magic(b'{' as c_int)), MULTI_MULT);

        // Non-multi
        assert_eq!(re_multi_type_impl(magic(b'a' as c_int)), NOT_MULTI);
        assert_eq!(re_multi_type_impl(b'*' as c_int), NOT_MULTI); // non-magic star
    }

    #[test]
    fn test_backslash_trans() {
        assert_eq!(backslash_trans_impl(b'r' as c_int), CAR);
        assert_eq!(backslash_trans_impl(b't' as c_int), TAB);
        assert_eq!(backslash_trans_impl(b'e' as c_int), ESC);
        assert_eq!(backslash_trans_impl(b'b' as c_int), BS);
        // Other characters should pass through
        assert_eq!(backslash_trans_impl(b'n' as c_int), b'n' as c_int);
        assert_eq!(backslash_trans_impl(b'x' as c_int), b'x' as c_int);
    }

    #[test]
    fn test_class_tab() {
        // Test digit detection
        assert!(ri_digit(b'0' as c_int));
        assert!(ri_digit(b'5' as c_int));
        assert!(ri_digit(b'9' as c_int));
        assert!(!ri_digit(b'a' as c_int));

        // Test hex detection
        assert!(ri_hex(b'0' as c_int));
        assert!(ri_hex(b'a' as c_int));
        assert!(ri_hex(b'F' as c_int));
        assert!(!ri_hex(b'g' as c_int));
        assert!(!ri_hex(b'G' as c_int));

        // Test octal detection
        assert!(ri_octal(b'0' as c_int));
        assert!(ri_octal(b'7' as c_int));
        assert!(!ri_octal(b'8' as c_int));

        // Test word detection
        assert!(ri_word(b'a' as c_int));
        assert!(ri_word(b'Z' as c_int));
        assert!(ri_word(b'5' as c_int));
        assert!(ri_word(b'_' as c_int));
        assert!(!ri_word(b'-' as c_int));

        // Test head detection
        assert!(ri_head(b'a' as c_int));
        assert!(ri_head(b'_' as c_int));
        assert!(!ri_head(b'0' as c_int));

        // Test alpha detection
        assert!(ri_alpha(b'a' as c_int));
        assert!(ri_alpha(b'Z' as c_int));
        assert!(!ri_alpha(b'0' as c_int));
        assert!(!ri_alpha(b'_' as c_int));

        // Test case detection
        assert!(ri_lower(b'a' as c_int));
        assert!(!ri_lower(b'A' as c_int));
        assert!(ri_upper(b'A' as c_int));
        assert!(!ri_upper(b'a' as c_int));

        // Test whitespace
        assert!(ri_white(b' ' as c_int));
        assert!(ri_white(b'\t' as c_int));
        assert!(!ri_white(b'\n' as c_int));

        // Test out of range
        assert!(!ri_digit(256));
        assert!(!ri_digit(-1));
    }

    #[test]
    fn test_re_flags() {
        // Test that flag constants are defined correctly
        // These match the values in regexp_defs.h
        assert_eq!(re_flags::RE_MAGIC, 1);
        assert_eq!(re_flags::RE_NOCASE, 2);
        assert_eq!(re_flags::RE_HASNL, 4);
    }

    #[test]
    fn test_regmatch_raw_default() {
        let rmp = RegmatchRaw::default();

        // Check that default initializes correctly
        assert!(rmp.regprog.is_null());
        assert_eq!(rmp.rm_matchcol, 0);
        assert!(!rmp.rm_ic);

        // All pointers should be null
        for startp in &rmp.startp {
            assert!(startp.is_null());
        }
        for endp in &rmp.endp {
            assert!(endp.is_null());
        }
    }

    #[test]
    fn test_match_result_methods() {
        // Create a mock MatchResult
        let mut submatches = [None; NSUBEXP];
        submatches[0] = Some((5, 10)); // Full match at positions 5-10
        submatches[1] = Some((6, 8)); // Submatch 1 at positions 6-8
        submatches[2] = Some((8, 9)); // Submatch 2 at positions 8-9

        let result = MatchResult {
            start: 5,
            end: 10,
            match_col: 5,
            submatches,
        };

        // Test len
        assert_eq!(result.len(), 5);

        // Test is_empty
        assert!(!result.is_empty());

        // Test submatch
        assert_eq!(result.submatch(0), Some((5, 10)));
        assert_eq!(result.submatch(1), Some((6, 8)));
        assert_eq!(result.submatch(2), Some((8, 9)));
        assert_eq!(result.submatch(3), None);
        assert_eq!(result.submatch(100), None); // Out of bounds

        // Test empty match
        let empty_submatches = [None; NSUBEXP];
        let empty_result = MatchResult {
            start: 0,
            end: 0,
            match_col: 0,
            submatches: empty_submatches,
        };
        assert!(empty_result.is_empty());
        assert_eq!(empty_result.len(), 0);
    }

    #[test]
    fn test_nsubexp_constant() {
        // NSUBEXP should be 10 (matches C definition)
        assert_eq!(NSUBEXP, 10);
    }

    #[test]
    fn test_magic_constants() {
        // Test magic mode constants
        assert_eq!(MAGIC_NONE, 1); // \V very nomagic
        assert_eq!(MAGIC_OFF, 2); // \M or magic off
        assert_eq!(MAGIC_ON, 3); // \m or magic (default)
        assert_eq!(MAGIC_ALL, 4); // \v very magic

        // Magic offset
        assert_eq!(MAGIC_OFFSET, 256);
    }

    #[test]
    fn test_multi_type_constants() {
        assert_eq!(NOT_MULTI, 0);
        assert_eq!(MULTI_ONE, 1);
        assert_eq!(MULTI_MULT, 2);
    }

    #[test]
    fn test_control_char_constants() {
        assert_eq!(CAR, 13);
        assert_eq!(TAB, 9);
        assert_eq!(ESC, 27);
        assert_eq!(BS, 8);
    }

    #[test]
    fn test_magic_un_magic_roundtrip() {
        // Any character should roundtrip through magic/un_magic
        for c in 0..256 {
            let magic_c = magic(c);
            let back = un_magic(magic_c);
            assert_eq!(back, c, "Roundtrip failed for {c}");
        }
    }

    #[test]
    fn test_magic_produces_negative() {
        // magic() should always produce negative values for ASCII
        for c in 0..=127 {
            let m = magic(c);
            assert!(m < 0, "magic({c}) = {m} should be negative");
        }
    }

    #[test]
    fn test_is_magic_matches_magic_function() {
        // Characters produced by magic() should test as is_magic()
        for c in 0..256 {
            let m = magic(c);
            assert!(is_magic(m), "is_magic(magic({c})) should be true");
        }

        // Regular positive characters should not be magic
        for c in 0..256 {
            assert!(!is_magic(c), "is_magic({c}) should be false");
        }
    }

    #[test]
    fn test_regprog_handle() {
        use std::ptr;

        // Test null handle
        let null_handle = RegprogHandle::from_ptr(ptr::null_mut());
        assert!(null_handle.is_null());
        assert_eq!(null_handle.as_ptr(), ptr::null_mut());

        // Test non-null handle
        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = RegprogHandle::from_ptr(ptr);
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);

        // Test equality
        let handle2 = RegprogHandle::from_ptr(ptr);
        assert_eq!(handle, handle2);
    }

    #[test]
    fn test_regmatch_handle() {
        use std::ptr;

        let null_handle = RegmatchHandle::from_ptr(ptr::null_mut());
        assert!(null_handle.is_null());

        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = RegmatchHandle::from_ptr(ptr);
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);
    }

    #[test]
    fn test_regmmatch_handle() {
        use std::ptr;

        let null_handle = RegmmatchHandle::from_ptr(ptr::null_mut());
        assert!(null_handle.is_null());

        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = RegmmatchHandle::from_ptr(ptr);
        assert!(!handle.is_null());
    }

    #[test]
    fn test_win_handle() {
        use std::ptr;

        let null_handle = WinHandle::from_ptr(ptr::null_mut());
        assert!(null_handle.is_null());

        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = WinHandle::from_ptr(ptr);
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);
    }

    #[test]
    fn test_buf_handle() {
        use std::ptr;

        let null_handle = BufHandle::from_ptr(ptr::null_mut());
        assert!(null_handle.is_null());

        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = BufHandle::from_ptr(ptr);
        assert!(!handle.is_null());
    }

    #[test]
    fn test_lpos_handle() {
        use std::ptr;

        let null_handle = LposHandle::from_ptr(ptr::null_mut());
        assert!(null_handle.is_null());

        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = LposHandle::from_ptr(ptr);
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);
    }

    #[test]
    fn test_byte_in_slice() {
        assert!(byte_in_slice(b'a', b"abc"));
        assert!(byte_in_slice(b'b', b"abc"));
        assert!(byte_in_slice(b'c', b"abc"));
        assert!(!byte_in_slice(b'd', b"abc"));
        assert!(!byte_in_slice(b'a', b""));
    }

    #[test]
    fn test_regexp_inrange() {
        // REGEXP_INRANGE should contain "]^-n\\"
        assert!(byte_in_slice(b']', REGEXP_INRANGE));
        assert!(byte_in_slice(b'^', REGEXP_INRANGE));
        assert!(byte_in_slice(b'-', REGEXP_INRANGE));
        assert!(byte_in_slice(b'n', REGEXP_INRANGE));
        assert!(byte_in_slice(b'\\', REGEXP_INRANGE));
        assert!(!byte_in_slice(b'a', REGEXP_INRANGE));
    }

    #[test]
    fn test_regexp_abbr() {
        // REGEXP_ABBR should contain "nrtebdoxuU"
        assert!(byte_in_slice(b'n', REGEXP_ABBR));
        assert!(byte_in_slice(b'r', REGEXP_ABBR));
        assert!(byte_in_slice(b't', REGEXP_ABBR));
        assert!(byte_in_slice(b'e', REGEXP_ABBR));
        assert!(byte_in_slice(b'b', REGEXP_ABBR));
        assert!(byte_in_slice(b'd', REGEXP_ABBR));
        assert!(byte_in_slice(b'o', REGEXP_ABBR));
        assert!(byte_in_slice(b'x', REGEXP_ABBR));
        assert!(byte_in_slice(b'u', REGEXP_ABBR));
        assert!(byte_in_slice(b'U', REGEXP_ABBR));
        assert!(!byte_in_slice(b'a', REGEXP_ABBR));
    }

    #[test]
    fn test_class_tab_initialization() {
        let tab = init_class_tab();

        // Verify size
        assert_eq!(tab.len(), 256);

        // Check digits 0-7 have all digit flags
        for c in b'0'..=b'7' {
            let flags = tab[c as usize];
            assert_ne!(
                flags & RI_DIGIT,
                0,
                "Digit '{}' should have RI_DIGIT",
                c as char
            );
            assert_ne!(
                flags & RI_HEX,
                0,
                "Digit '{}' should have RI_HEX",
                c as char
            );
            assert_ne!(
                flags & RI_OCTAL,
                0,
                "Digit '{}' should have RI_OCTAL",
                c as char
            );
            assert_ne!(
                flags & RI_WORD,
                0,
                "Digit '{}' should have RI_WORD",
                c as char
            );
        }

        // Check digits 8-9 have digit but not octal
        for c in b'8'..=b'9' {
            let flags = tab[c as usize];
            assert_ne!(flags & RI_DIGIT, 0);
            assert_ne!(flags & RI_HEX, 0);
            assert_eq!(
                flags & RI_OCTAL,
                0,
                "Digit '{}' should NOT have RI_OCTAL",
                c as char
            );
        }

        // Check lowercase letters
        for c in b'a'..=b'z' {
            let flags = tab[c as usize];
            assert_ne!(flags & RI_WORD, 0);
            assert_ne!(flags & RI_HEAD, 0);
            assert_ne!(flags & RI_ALPHA, 0);
            assert_ne!(flags & RI_LOWER, 0);
            assert_eq!(flags & RI_UPPER, 0);
        }

        // Check uppercase letters
        for c in b'A'..=b'Z' {
            let flags = tab[c as usize];
            assert_ne!(flags & RI_WORD, 0);
            assert_ne!(flags & RI_HEAD, 0);
            assert_ne!(flags & RI_ALPHA, 0);
            assert_ne!(flags & RI_UPPER, 0);
            assert_eq!(flags & RI_LOWER, 0);
        }

        // Check underscore
        let underscore_flags = tab[b'_' as usize];
        assert_ne!(underscore_flags & RI_WORD, 0);
        assert_ne!(underscore_flags & RI_HEAD, 0);
        assert_eq!(underscore_flags & RI_ALPHA, 0);

        // Check whitespace
        assert_ne!(tab[b' ' as usize] & RI_WHITE, 0);
        assert_ne!(tab[b'\t' as usize] & RI_WHITE, 0);
        assert_eq!(tab[b'\n' as usize] & RI_WHITE, 0);
    }

    #[test]
    fn test_class_flag_constants() {
        // Verify flag constants are unique powers of 2
        assert_eq!(RI_DIGIT, 0x01);
        assert_eq!(RI_HEX, 0x02);
        assert_eq!(RI_OCTAL, 0x04);
        assert_eq!(RI_WORD, 0x08);
        assert_eq!(RI_HEAD, 0x10);
        assert_eq!(RI_ALPHA, 0x20);
        assert_eq!(RI_LOWER, 0x40);
        assert_eq!(RI_UPPER, 0x80);
        assert_eq!(RI_WHITE, 0x100);
    }

    #[test]
    fn test_maxcol_constant() {
        assert_eq!(MAXCOL, 0x7fff_ffff);
    }

    #[test]
    fn test_class_none_constant() {
        assert_eq!(CLASS_NONE, 99);
    }
}
