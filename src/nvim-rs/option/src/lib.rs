//! Option system for Neovim
//!
//! This crate provides Rust implementations of Neovim's option handling
//! functionality from `src/nvim/option.c`. It handles option types, scopes,
//! validation, and option value manipulation.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)] // Getters don't need #[must_use]
#![allow(clippy::missing_const_for_fn)] // FFI functions can't be const
#![allow(clippy::cast_sign_loss)] // FFI with C char types
#![allow(clippy::cast_possible_wrap)] // FFI u8/i8 casts for c_char comparisons
#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

pub mod accessors;
pub mod apply;
pub mod buf_opt_index;
pub mod bufcopy;
pub mod callbacks;
pub mod completion;
pub mod copy;
pub mod defaults;
pub mod expand;
pub mod format;
pub mod getset;
pub mod groups;
pub mod index;
pub mod init;
pub mod modeline;
pub mod opt_index;
pub mod parsing;
pub mod query;
pub mod scope;
pub mod security;
pub mod session;
pub mod set;
pub mod setcmd;
pub mod setops;
pub mod show;
pub mod sideeffect;
pub mod storage;
pub mod types;
pub mod validate;
pub mod value;
pub mod varp;
pub mod winopt;

use std::ffi::{c_char, c_int, c_uint};

/// OptInt type (matches C's OptInt = int64_t)
pub type OptInt = i64;

// =============================================================================
// Constants
// =============================================================================

/// Function succeeded
pub const OK: c_int = 1;

/// Function failed
pub const FAIL: c_int = 0;

// buf_copy_options flags (BCO_* from option.h)
pub const BCO_ENTER: c_int = 1;
pub const BCO_ALWAYS: c_int = 2;
pub const BCO_NOHELP: c_int = 4;

// 'cpoptions' characters relevant to buf_copy_options (from option_vars.h)
pub const CPO_BUFOPTGLOB: c_int = b'S' as c_int;
pub const CPO_BUFOPT: c_int = b's' as c_int;

// cmdmod flag for :noswapfile (from ex_cmds_defs.h)
pub const CMOD_NOSWAPFILE: c_int = 0x2000;

// Script context: don't set scriptID (from globals.h)
pub const SID_NONE: c_int = -6;

// kOptFlag* bitmask constants (from option_defs.h)
pub const K_OPT_FLAG_UI_OPTION: c_uint = 1 << 5;
pub const K_OPT_FLAG_REDR_WIN: c_uint = 1 << 8;
pub const K_OPT_FLAG_REDR_BUF: c_uint = 1 << 9;
pub const K_OPT_FLAG_REDR_ALL: c_uint = K_OPT_FLAG_REDR_BUF | K_OPT_FLAG_REDR_WIN;
pub const K_OPT_FLAG_SECURE: c_uint = 1 << 14;
pub const K_OPT_FLAG_INSECURE: c_uint = 1 << 18;
pub const K_OPT_FLAG_CURSWANT: c_uint = 1 << 21;
pub const K_OPT_FLAG_HL_ONLY: c_uint = 1 << 23;

// OPT_MODELINE flag (from option.h)
pub const OPT_MODELINE: c_int = 0x04;

// UPD_NOT_VALID redraw level (from drawscreen.h)
pub const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// Option Value Types
// =============================================================================

/// Option value type.
///
/// Corresponds to `OptValType` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptValType {
    /// Nil/unset value
    Nil = -1,
    /// Boolean option (true/false/none)
    Boolean = 0,
    /// Numeric option (integer)
    Number = 1,
    /// String option
    String = 2,
}

// =============================================================================
// Option Scopes
// =============================================================================

/// Scopes that an option can support.
///
/// Corresponds to `OptScope` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptScope {
    /// Global option value
    Global = 0,
    /// Window-local option value
    Win = 1,
    /// Buffer-local option value
    Buf = 2,
}

impl OptScope {
    /// Number of option scopes
    pub const COUNT: usize = 3;
}

// =============================================================================
// Option Flags
// =============================================================================

/// Option flags.
///
/// Corresponds to `OptFlags` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptFlags(pub u32);

impl OptFlags {
    /// Environment expansion
    pub const EXPAND: Self = Self(1 << 0);
    /// Don't expand default value
    pub const NO_DEF_EXP: Self = Self(1 << 1);
    /// Don't set to default value
    pub const NO_DEFAULT: Self = Self(1 << 2);
    /// Option has been set/reset
    pub const WAS_SET: Self = Self(1 << 3);
    /// Don't include in :mkvimrc output
    pub const NO_MKRC: Self = Self(1 << 4);
    /// Send option to remote UI
    pub const UI_OPTION: Self = Self(1 << 5);
    /// Redraw tabline
    pub const REDR_TABL: Self = Self(1 << 6);
    /// Redraw status lines
    pub const REDR_STAT: Self = Self(1 << 7);
    /// Redraw current window and recompute text
    pub const REDR_WIN: Self = Self(1 << 8);
    /// Redraw current buffer and recompute text
    pub const REDR_BUF: Self = Self(1 << 9);
    /// Comma-separated list
    pub const COMMA: Self = Self(1 << 10);
    /// Don't allow duplicate strings
    pub const NO_DUP: Self = Self(1 << 12);
    /// List of single-char flags
    pub const FLAG_LIST: Self = Self(1 << 13);
    /// Cannot change in modeline or secure mode
    pub const SECURE: Self = Self(1 << 14);
    /// Expand default value with _()
    pub const GETTEXT: Self = Self(1 << 15);
    /// Do not use local value for global vimrc
    pub const NO_GLOB: Self = Self(1 << 16);
    /// Only normal file name chars allowed
    pub const N_FNAME: Self = Self(1 << 17);
    /// Option was set from a modeline
    pub const INSECURE: Self = Self(1 << 18);
    /// Priority for :mkvimrc
    pub const PRI_MKRC: Self = Self(1 << 19);
    /// Not allowed in modeline
    pub const NO_ML: Self = Self(1 << 20);
    /// Update curswant required
    pub const CURSWANT: Self = Self(1 << 21);
    /// Only normal directory name chars allowed
    pub const N_DNAME: Self = Self(1 << 22);
    /// Option only changes highlight, not text
    pub const HL_ONLY: Self = Self(1 << 23);
    /// Under control of 'modelineexpr'
    pub const MLE: Self = Self(1 << 24);
    /// Accept a function reference or a lambda
    pub const FUNC: Self = Self(1 << 25);
    /// Values use colons to create sublists
    pub const COLON: Self = Self(1 << 26);

    /// Redraw all windows and recompute text
    pub const REDR_ALL: Self = Self(Self::REDR_BUF.0 | Self::REDR_WIN.0);
    /// Clear and redraw all and recompute text
    pub const REDR_CLEAR: Self = Self(Self::REDR_ALL.0 | Self::REDR_STAT.0);
    /// Comma-separated list that cannot have two consecutive commas
    pub const ONE_COMMA: Self = Self((1 << 11) | Self::COMMA.0);

    /// Check if a flag is set
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two flag sets
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// =============================================================================
// :set Operator Types
// =============================================================================

/// :set operator types.
///
/// Corresponds to `set_op_T` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOp {
    /// No operator
    None = 0,
    /// "opt+=arg"
    Adding = 1,
    /// "opt^=arg"
    Prepending = 2,
    /// "opt-=arg"
    Removing = 3,
}

// =============================================================================
// :set Boolean Option Prefix
// =============================================================================

/// :set boolean option prefix.
///
/// Corresponds to `set_prefix_T` in option.c.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPrefix {
    /// "no" prefix
    No = 0,
    /// No prefix
    None = 1,
    /// "inv" prefix
    Inv = 2,
}

// =============================================================================
// Option Setting Flags
// =============================================================================

/// Flags for option-setting functions.
///
/// Corresponds to `OptionSetFlags` in option.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSetFlags(pub u32);

impl OptionSetFlags {
    /// Use global value
    pub const GLOBAL: Self = Self(0x01);
    /// Use local value
    pub const LOCAL: Self = Self(0x02);
    /// Option in modeline
    pub const MODELINE: Self = Self(0x04);
    /// Only set window-local options
    pub const WINONLY: Self = Self(0x08);
    /// Don't set window-local options
    pub const NOWIN: Self = Self(0x10);
    /// List options one per line
    pub const ONECOLUMN: Self = Self(0x20);
    /// Ignore redraw flags on option
    pub const NO_REDRAW: Self = Self(0x40);
    /// "skiprtp" in 'sessionoptions'
    pub const SKIPRTP: Self = Self(0x80);

    /// Check if a flag is set
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two flag sets
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

// =============================================================================
// C Accessor Functions (FFI)
// =============================================================================

extern "C" {
    // String option globals (char * -- accessed as *mut c_char, cast to *const c_char at use sites)
    pub static mut p_sh: *mut c_char;
    pub static mut p_ffs: *mut c_char;
    pub static mut p_cpo: *mut c_char;
    pub static mut p_isk: *mut c_char;
    pub static mut p_isf: *mut c_char;
    pub static mut p_isp: *mut c_char;
    pub static mut p_isi: *mut c_char;
    pub static mut p_breakat: *mut c_char;
    pub static mut p_sel: *mut c_char;
    pub static mut p_enc: *mut c_char;
    pub static mut p_ff: *mut c_char;
    pub static mut p_fo: *mut c_char;
    pub static mut p_mps: *mut c_char;
    pub static mut p_nf: *mut c_char;
    pub static mut p_ww: *mut c_char;
    pub static mut p_mouse: *mut c_char;
    pub static mut p_shm: *mut c_char;

    // Boolean/int option globals
    pub static mut p_ai: c_int;
    pub static mut p_et: c_int;
    pub static mut p_ic: c_int;
    pub static mut p_scs: c_int;
    pub static mut p_hls: c_int;
    pub static mut p_is: c_int;
    pub static mut p_magic: c_int;
    pub static mut p_fic: c_int;
    pub static mut p_ml: c_int;
    pub static mut p_mle: c_int;
    pub static mut p_paste: c_int;
    pub static mut p_ri: c_int;
    pub static mut p_ws: c_int;
    pub static mut p_gd: c_int;
    pub static mut p_ea: c_int;
    pub static mut p_hid: c_int;
    pub static mut p_sm: c_int;
    pub static mut p_lz: c_int;
    pub static mut p_to: c_int;

    // Numeric option globals (OptInt = i64)
    pub static mut p_sw: OptInt;
    pub static mut p_ts: OptInt;
    pub static mut p_sts: OptInt;
    pub static mut p_tw: OptInt;
    pub static mut p_wm: OptInt;
    pub static mut p_so: OptInt;
    pub static mut p_siso: OptInt;
    pub static mut p_columns: OptInt;
    pub static mut p_lines: OptInt;
    pub static mut p_ch: OptInt;
    pub static mut p_report: OptInt;
    pub static mut p_mat: OptInt;
    pub static mut p_ut: OptInt;
    pub static mut p_tm: OptInt;
    pub static mut p_hi: OptInt;
    pub static mut p_ls: OptInt;
    pub static mut p_stal: OptInt;
    pub static mut p_re: OptInt;
    pub static mut p_wmh: OptInt;
    pub static mut p_wh: OptInt;
    pub static mut p_wmw: OptInt;
    pub static mut p_wiw: OptInt;
    pub static mut p_uc: OptInt;
    pub static mut p_hh: OptInt;
    pub static mut p_pb: OptInt;
    pub static mut p_ul: OptInt;
    pub static mut p_window: OptInt;
    pub static mut p_ss: OptInt;
    pub static mut p_rdt: OptInt;
    pub static mut p_cwh: OptInt;

    // Flag option globals (unsigned)
    pub static mut cot_flags: c_uint;
    pub static mut fdo_flags: c_uint;
    pub static mut dy_flags: c_uint;
    pub static mut cb_flags: c_uint;

    // Special globals from globals.h
    pub static mut magic_overruled: c_int;
    pub static mut secure: c_int;
    pub static mut starting: c_int;
    pub static mut no_hlsearch: bool;
    pub static mut full_screen: bool;
    pub static mut Rows: c_int;
    pub static mut clear_cmdline: bool;
    pub static mut readonlymode: bool;
    pub static mut km_stopsel: bool;
    pub static mut km_startsel: bool;
    pub static mut silent_mode: bool;
    pub static mut info_message: bool;
    pub static mut no_wait_return: c_int;
}

// =============================================================================
// Safe Rust Wrappers for Option Access
// =============================================================================

/// Get the 'shell' option value.
#[inline]
pub fn get_shell() -> *const c_char {
    unsafe { p_sh.cast_const() }
}

/// Get the 'fileformats' option value.
#[inline]
pub fn get_fileformats() -> *const c_char {
    unsafe { p_ffs.cast_const() }
}

/// Get the 'cpoptions' option value.
#[inline]
pub fn get_cpoptions() -> *const c_char {
    unsafe { p_cpo.cast_const() }
}

/// Get the 'iskeyword' option value.
#[inline]
pub fn get_iskeyword() -> *const c_char {
    unsafe { p_isk.cast_const() }
}

/// Get the 'isfname' option value.
#[inline]
pub fn get_isfname() -> *const c_char {
    unsafe { p_isf.cast_const() }
}

/// Get the 'isprint' option value.
#[inline]
pub fn get_isprint() -> *const c_char {
    unsafe { p_isp.cast_const() }
}

/// Get the 'isident' option value.
#[inline]
pub fn get_isident() -> *const c_char {
    unsafe { p_isi.cast_const() }
}

/// Get the 'breakat' option value.
#[inline]
pub fn get_breakat() -> *const c_char {
    unsafe { p_breakat.cast_const() }
}

/// Get the 'selection' option value.
#[inline]
pub fn get_selection() -> *const c_char {
    unsafe { p_sel.cast_const() }
}

/// Get the 'encoding' option value.
#[inline]
pub fn get_encoding() -> *const c_char {
    unsafe { p_enc.cast_const() }
}

/// Get the 'fileformat' option value.
#[inline]
pub fn get_fileformat() -> *const c_char {
    unsafe { p_ff.cast_const() }
}

/// Get the 'formatoptions' option value.
#[inline]
pub fn get_formatoptions() -> *const c_char {
    unsafe { p_fo.cast_const() }
}

/// Get the 'matchpairs' option value.
#[inline]
pub fn get_matchpairs() -> *const c_char {
    unsafe { p_mps.cast_const() }
}

/// Get the 'nrformats' option value.
#[inline]
pub fn get_nrformats() -> *const c_char {
    unsafe { p_nf.cast_const() }
}

/// Get the 'whichwrap' option value.
#[inline]
pub fn get_whichwrap() -> *const c_char {
    unsafe { p_ww.cast_const() }
}

/// Get the 'mouse' option value.
#[inline]
pub fn get_mouse() -> *const c_char {
    unsafe { p_mouse.cast_const() }
}

/// Get the 'shortmess' option value.
#[inline]
pub fn get_shortmess() -> *const c_char {
    unsafe { p_shm.cast_const() }
}

/// Get the 'autoindent' option value.
#[inline]
pub fn get_autoindent() -> bool {
    unsafe { p_ai != 0 }
}

/// Get the 'expandtab' option value.
#[inline]
pub fn get_expandtab() -> bool {
    unsafe { p_et != 0 }
}

/// Get the 'ignorecase' option value.
#[inline]
pub fn get_ignorecase() -> bool {
    unsafe { p_ic != 0 }
}

/// Get the 'smartcase' option value.
#[inline]
pub fn get_smartcase() -> bool {
    unsafe { p_scs != 0 }
}

/// Get the 'hlsearch' option value.
#[inline]
pub fn get_hlsearch() -> bool {
    unsafe { p_hls != 0 }
}

/// Get the 'incsearch' option value.
#[inline]
pub fn get_incsearch() -> bool {
    unsafe { p_is != 0 }
}

/// Get the 'magic' option value.
#[inline]
pub fn get_magic() -> bool {
    unsafe { p_magic != 0 }
}

/// Get the 'fileignorecase' option value.
#[inline]
pub fn get_fileignorecase() -> bool {
    unsafe { p_fic != 0 }
}

/// Get the 'modeline' option value.
#[inline]
pub fn get_modeline() -> bool {
    unsafe { p_ml != 0 }
}

/// Get the 'modelineexpr' option value.
#[inline]
pub fn get_modelineexpr() -> bool {
    unsafe { p_mle != 0 }
}

/// Get the 'paste' option value.
#[inline]
pub fn get_paste() -> bool {
    unsafe { p_paste != 0 }
}

/// Get the 'revins' option value.
#[inline]
pub fn get_revins() -> bool {
    unsafe { p_ri != 0 }
}

/// Get the 'wrapscan' option value.
#[inline]
pub fn get_wrapscan() -> bool {
    unsafe { p_ws != 0 }
}

/// Get the 'gdefault' option value.
#[inline]
pub fn get_gdefault() -> bool {
    unsafe { p_gd != 0 }
}

/// Get the 'equalalways' option value.
#[inline]
pub fn get_equalalways() -> bool {
    unsafe { p_ea != 0 }
}

/// Get the 'hidden' option value.
#[inline]
pub fn get_hidden() -> bool {
    unsafe { p_hid != 0 }
}

/// Get the 'showmatch' option value.
#[inline]
pub fn get_showmatch() -> bool {
    unsafe { p_sm != 0 }
}

/// Get the 'lazyredraw' option value.
#[inline]
pub fn get_lazyredraw() -> bool {
    unsafe { p_lz != 0 }
}

/// Get the 'tildeop' option value.
#[inline]
pub fn get_tildeop() -> bool {
    unsafe { p_to != 0 }
}

/// Get the 'shiftwidth' option value.
#[inline]
pub fn get_shiftwidth() -> OptInt {
    unsafe { p_sw }
}

/// Get the 'tabstop' option value.
#[inline]
pub fn get_tabstop() -> OptInt {
    unsafe { p_ts }
}

/// Get the 'softtabstop' option value.
#[inline]
pub fn get_softtabstop() -> OptInt {
    unsafe { p_sts }
}

/// Get the 'textwidth' option value.
#[inline]
pub fn get_textwidth() -> OptInt {
    unsafe { p_tw }
}

/// Get the 'wrapmargin' option value.
#[inline]
pub fn get_wrapmargin() -> OptInt {
    unsafe { p_wm }
}

/// Get the 'scrolloff' option value.
#[inline]
pub fn get_scrolloff() -> OptInt {
    unsafe { p_so }
}

/// Get the 'sidescrolloff' option value.
#[inline]
pub fn get_sidescrolloff() -> OptInt {
    unsafe { p_siso }
}

/// Get the 'columns' option value.
#[inline]
pub fn get_columns() -> OptInt {
    unsafe { p_columns }
}

/// Get the 'lines' option value.
#[inline]
pub fn get_lines() -> OptInt {
    unsafe { p_lines }
}

/// Get the 'cmdheight' option value.
#[inline]
pub fn get_cmdheight() -> OptInt {
    unsafe { p_ch }
}

/// Get the 'report' option value.
#[inline]
pub fn get_report() -> OptInt {
    unsafe { p_report }
}

/// Get the 'matchtime' option value.
#[inline]
pub fn get_matchtime() -> OptInt {
    unsafe { p_mat }
}

/// Get the 'updatetime' option value.
#[inline]
pub fn get_updatetime() -> OptInt {
    unsafe { p_ut }
}

/// Get the 'timeoutlen' option value.
#[inline]
pub fn get_timeoutlen() -> OptInt {
    unsafe { p_tm }
}

/// Get the 'history' option value.
#[inline]
pub fn get_history() -> OptInt {
    unsafe { p_hi }
}

/// Get the 'laststatus' option value.
#[inline]
pub fn get_laststatus() -> OptInt {
    unsafe { p_ls }
}

/// Get the 'showtabline' option value.
#[inline]
pub fn get_showtabline() -> OptInt {
    unsafe { p_stal }
}

/// Get the 'regexpengine' option value.
#[inline]
pub fn get_regexpengine() -> OptInt {
    unsafe { p_re }
}

/// Get the 'completeopt' flags.
#[inline]
pub fn get_completeopt_flags() -> c_uint {
    unsafe { cot_flags }
}

/// Get the 'foldopen' flags.
#[inline]
pub fn get_foldopen_flags() -> c_uint {
    unsafe { fdo_flags }
}

/// Get the 'display' flags.
#[inline]
pub fn get_display_flags() -> c_uint {
    unsafe { dy_flags }
}

/// Get the 'clipboard' flags.
#[inline]
pub fn get_clipboard_flags() -> c_uint {
    unsafe { cb_flags }
}

/// Get the magic_overruled value.
#[inline]
pub fn get_magic_overruled() -> c_int {
    unsafe { magic_overruled }
}

// =============================================================================
// Safe Rust Wrappers for Option Setting
// =============================================================================

/// Set the 'autoindent' option value.
#[inline]
pub fn set_autoindent(value: bool) {
    unsafe { p_ai = c_int::from(value) }
}

/// Set the 'expandtab' option value.
#[inline]
pub fn set_expandtab(value: bool) {
    unsafe { p_et = c_int::from(value) }
}

/// Set the 'ignorecase' option value.
#[inline]
pub fn set_ignorecase(value: bool) {
    unsafe { p_ic = c_int::from(value) }
}

/// Set the 'smartcase' option value.
#[inline]
pub fn set_smartcase(value: bool) {
    unsafe { p_scs = c_int::from(value) }
}

/// Set the 'hlsearch' option value.
#[inline]
pub fn set_hlsearch(value: bool) {
    unsafe { p_hls = c_int::from(value) }
}

/// Set the 'incsearch' option value.
#[inline]
pub fn set_incsearch(value: bool) {
    unsafe { p_is = c_int::from(value) }
}

/// Set the 'magic' option value.
#[inline]
pub fn set_magic(value: bool) {
    unsafe { p_magic = c_int::from(value) }
}

/// Set the 'modeline' option value.
#[inline]
pub fn set_modeline(value: bool) {
    unsafe { p_ml = c_int::from(value) }
}

/// Set the 'paste' option value.
#[inline]
pub fn set_paste(value: bool) {
    unsafe { p_paste = c_int::from(value) }
}

/// Set the 'revins' option value.
#[inline]
pub fn set_revins(value: bool) {
    unsafe { p_ri = c_int::from(value) }
}

/// Set the 'wrapscan' option value.
#[inline]
pub fn set_wrapscan(value: bool) {
    unsafe { p_ws = c_int::from(value) }
}

/// Set the 'gdefault' option value.
#[inline]
pub fn set_gdefault(value: bool) {
    unsafe { p_gd = c_int::from(value) }
}

/// Set the 'equalalways' option value.
#[inline]
pub fn set_equalalways(value: bool) {
    unsafe { p_ea = c_int::from(value) }
}

/// Set the 'hidden' option value.
#[inline]
pub fn set_hidden(value: bool) {
    unsafe { p_hid = c_int::from(value) }
}

/// Set the 'showmatch' option value.
#[inline]
pub fn set_showmatch(value: bool) {
    unsafe { p_sm = c_int::from(value) }
}

/// Set the 'lazyredraw' option value.
#[inline]
pub fn set_lazyredraw(value: bool) {
    unsafe { p_lz = c_int::from(value) }
}

/// Set the 'tildeop' option value.
#[inline]
pub fn set_tildeop(value: bool) {
    unsafe { p_to = c_int::from(value) }
}

/// Set the 'shiftwidth' option value.
#[inline]
pub fn set_shiftwidth(value: OptInt) {
    unsafe { p_sw = value }
}

/// Set the 'tabstop' option value.
#[inline]
pub fn set_tabstop(value: OptInt) {
    unsafe { p_ts = value }
}

/// Set the 'softtabstop' option value.
#[inline]
pub fn set_softtabstop(value: OptInt) {
    unsafe { p_sts = value }
}

/// Set the 'textwidth' option value.
#[inline]
pub fn set_textwidth(value: OptInt) {
    unsafe { p_tw = value }
}

/// Set the 'wrapmargin' option value.
#[inline]
pub fn set_wrapmargin(value: OptInt) {
    unsafe { p_wm = value }
}

/// Set the 'scrolloff' option value.
#[inline]
pub fn set_scrolloff(value: OptInt) {
    unsafe { p_so = value }
}

/// Set the 'sidescrolloff' option value.
#[inline]
pub fn set_sidescrolloff(value: OptInt) {
    unsafe { p_siso = value }
}

/// Set the 'report' option value.
#[inline]
pub fn set_report(value: OptInt) {
    unsafe { p_report = value }
}

/// Set the 'matchtime' option value.
#[inline]
pub fn set_matchtime(value: OptInt) {
    unsafe { p_mat = value }
}

/// Set the 'updatetime' option value.
#[inline]
pub fn set_updatetime(value: OptInt) {
    unsafe { p_ut = value }
}

/// Set the 'timeoutlen' option value.
#[inline]
pub fn set_timeoutlen(value: OptInt) {
    unsafe { p_tm = value }
}

/// Set the 'history' option value.
#[inline]
pub fn set_history(value: OptInt) {
    unsafe { p_hi = value }
}

/// Set the 'regexpengine' option value.
#[inline]
pub fn set_regexpengine(value: OptInt) {
    unsafe { p_re = value }
}

/// Set the magic_overruled value.
#[inline]
pub fn set_magic_overruled(value: c_int) {
    unsafe { magic_overruled = value }
}

// =============================================================================
// Validation Utilities
// =============================================================================

/// Error codes for option validation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    /// No error
    Ok = 0,
    /// Value must be positive
    NeedPositive = 1,
    /// Invalid argument
    InvalidArg = 2,
    /// Value out of range
    OutOfRange = 3,
}

/// Validate that a numeric option value is non-negative.
#[no_mangle]
pub extern "C" fn rs_validate_nonnegative(value: OptInt) -> c_int {
    if value < 0 {
        ValidationError::NeedPositive as c_int
    } else {
        ValidationError::Ok as c_int
    }
}

/// Validate that a numeric option value is positive (>= 1).
#[no_mangle]
pub extern "C" fn rs_validate_positive(value: OptInt) -> c_int {
    if value < 1 {
        ValidationError::NeedPositive as c_int
    } else {
        ValidationError::Ok as c_int
    }
}

/// Validate that a numeric option value is within a range (inclusive).
#[no_mangle]
pub extern "C" fn rs_validate_range(value: OptInt, min: OptInt, max: OptInt) -> c_int {
    if value < min || value > max {
        ValidationError::OutOfRange as c_int
    } else {
        ValidationError::Ok as c_int
    }
}

/// Clamp a value to a range (inclusive).
#[no_mangle]
pub extern "C" fn rs_clamp_value(value: OptInt, min: OptInt, max: OptInt) -> OptInt {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Validate the 'regexpengine' option value (must be 0, 1, or 2).
#[no_mangle]
pub extern "C" fn rs_validate_regexpengine(value: OptInt) -> c_int {
    if (0..=2).contains(&value) {
        ValidationError::Ok as c_int
    } else {
        ValidationError::InvalidArg as c_int
    }
}

/// Validate the 'history' option value (must be 0-10000).
#[no_mangle]
pub extern "C" fn rs_validate_history(value: OptInt) -> c_int {
    if value < 0 {
        ValidationError::NeedPositive as c_int
    } else if value > 10000 {
        ValidationError::InvalidArg as c_int
    } else {
        ValidationError::Ok as c_int
    }
}

/// Validate percentage values (must be 0-100).
#[no_mangle]
pub extern "C" fn rs_validate_percentage(value: OptInt) -> c_int {
    if (0..=100).contains(&value) {
        ValidationError::Ok as c_int
    } else {
        ValidationError::OutOfRange as c_int
    }
}

/// Clamp percentage values to 0-100.
#[no_mangle]
pub extern "C" fn rs_clamp_percentage(value: OptInt) -> OptInt {
    rs_clamp_value(value, 0, 100)
}

// =============================================================================
// Option Parsing Utilities
// =============================================================================

/// Check if a string starts with "no" prefix (for boolean options).
/// Returns the length of the prefix (2) if found, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_has_no_prefix(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    let b0 = *s as u8;
    let b1 = *s.add(1) as u8;
    if b0 == b'n' && b1 == b'o' {
        2
    } else {
        0
    }
}

/// Check if a string starts with "inv" prefix (for boolean options).
/// Returns the length of the prefix (3) if found, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_has_inv_prefix(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    let b0 = *s as u8;
    let b1 = *s.add(1) as u8;
    let b2 = *s.add(2) as u8;
    if b0 == b'i' && b1 == b'n' && b2 == b'v' {
        3
    } else {
        0
    }
}

/// Parse the boolean prefix from an option name.
/// Returns: 0 = no prefix, 1 = "no" prefix, 2 = "inv" prefix
#[no_mangle]
pub unsafe extern "C" fn rs_parse_bool_prefix(s: *const c_char) -> c_int {
    if s.is_null() {
        return SetPrefix::None as c_int;
    }
    if rs_option_has_no_prefix(s) > 0 {
        SetPrefix::No as c_int
    } else if rs_option_has_inv_prefix(s) > 0 {
        SetPrefix::Inv as c_int
    } else {
        SetPrefix::None as c_int
    }
}

/// Check if a string contains only characters from an allowed set.
/// Returns 1 if valid, 0 if invalid.
#[no_mangle]
pub unsafe extern "C" fn rs_option_chars_valid(s: *const c_char, allowed: *const c_char) -> c_int {
    if s.is_null() || allowed.is_null() {
        return 1;
    }

    let mut p = s;
    while *p != 0 {
        let ch = *p as u8;

        // Check if character is in allowed set
        let mut found = false;
        let mut a = allowed;
        while *a != 0 {
            if *a as u8 == ch {
                found = true;
                break;
            }
            a = a.add(1);
        }

        if !found {
            return 0;
        }
        p = p.add(1);
    }
    1
}

/// Check if a string matches a specific keyword (case-sensitive).
/// Returns 1 if matches, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_match_keyword(
    s: *const c_char,
    keyword: *const c_char,
    len: usize,
) -> c_int {
    if s.is_null() || keyword.is_null() || len == 0 {
        return 0;
    }

    for i in 0..len {
        let sc = *s.add(i) as u8;
        let kc = *keyword.add(i) as u8;
        if sc != kc {
            return 0;
        }
    }

    // Check that there's no more alphanumeric characters after
    let next = *s.add(len) as u8;
    if next.is_ascii_alphanumeric() || next == b'_' {
        return 0;
    }

    1
}

/// Skip leading whitespace in an option argument.
#[no_mangle]
pub unsafe extern "C" fn rs_option_skip_whitespace(s: *const c_char) -> *const c_char {
    if s.is_null() {
        return s;
    }
    let mut p = s;
    while (*p as u8) == b' ' || (*p as u8) == b'\t' {
        p = p.add(1);
    }
    p
}

/// Find the end of an option argument (stops at whitespace or delimiter).
#[no_mangle]
pub unsafe extern "C" fn rs_option_find_arg_end(s: *const c_char) -> *const c_char {
    if s.is_null() {
        return s;
    }
    let mut p = s;
    while *p != 0 {
        let ch = *p as u8;
        if ch == b' ' || ch == b'\t' || ch == b',' || ch == b':' {
            break;
        }
        p = p.add(1);
    }
    p
}

// =============================================================================
// Option Name Validation
// =============================================================================

/// Check if a character is valid as the first character of an option name.
/// Option names must start with a letter or underscore.
#[inline]
#[must_use]
pub const fn is_valid_option_name_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

/// Check if a character is valid in an option name (after first character).
/// Option names can contain letters, digits, and underscores.
#[inline]
#[must_use]
pub const fn is_valid_option_name_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Validate an option name string.
/// Returns 1 if the name is valid, 0 otherwise.
///
/// A valid option name:
/// - Is not empty
/// - Starts with a letter or underscore
/// - Contains only letters, digits, and underscores
#[no_mangle]
pub unsafe extern "C" fn rs_validate_option_name(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let first = *name as u8;
    if first == 0 || !is_valid_option_name_start(first) {
        return 0;
    }

    let mut p = name.add(1);
    while *p != 0 {
        if !is_valid_option_name_char(*p as u8) {
            return 0;
        }
        p = p.add(1);
    }

    1
}

/// Get the length of an option name starting at the given pointer.
/// Stops at the first invalid character.
#[no_mangle]
pub unsafe extern "C" fn rs_option_name_length(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }

    let first = *s as u8;
    if !is_valid_option_name_start(first) {
        return 0;
    }

    let mut len: usize = 1;
    let mut p = s.add(1);
    while *p != 0 && is_valid_option_name_char(*p as u8) {
        len += 1;
        p = p.add(1);
    }

    len
}

/// Parse a :set operator from a string.
/// Returns the operator type and updates the pointer past the operator.
///
/// Operators:
/// - '=' or ':' -> SetOp::None (simple assignment)
/// - '+=' -> SetOp::Adding
/// - '^=' -> SetOp::Prepending
/// - '-=' -> SetOp::Removing
#[no_mangle]
pub unsafe extern "C" fn rs_parse_set_operator(p: *mut *const c_char) -> c_int {
    if p.is_null() || (*p).is_null() {
        return SetOp::None as c_int;
    }

    let mut s = *p;
    let c = *s as u8;

    let op = match c {
        b'+' => {
            s = s.add(1);
            if *s as u8 == b'=' {
                s = s.add(1);
                SetOp::Adding
            } else {
                SetOp::None
            }
        }
        b'^' => {
            s = s.add(1);
            if *s as u8 == b'=' {
                s = s.add(1);
                SetOp::Prepending
            } else {
                SetOp::None
            }
        }
        b'-' => {
            s = s.add(1);
            if *s as u8 == b'=' {
                s = s.add(1);
                SetOp::Removing
            } else {
                SetOp::None
            }
        }
        b'=' | b':' => {
            s = s.add(1);
            SetOp::None
        }
        _ => SetOp::None,
    };

    *p = s;
    op as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_val_type_repr() {
        assert_eq!(OptValType::Nil as i32, -1);
        assert_eq!(OptValType::Boolean as i32, 0);
        assert_eq!(OptValType::Number as i32, 1);
        assert_eq!(OptValType::String as i32, 2);
    }

    #[test]
    fn test_opt_scope_repr() {
        assert_eq!(OptScope::Global as i32, 0);
        assert_eq!(OptScope::Win as i32, 1);
        assert_eq!(OptScope::Buf as i32, 2);
    }

    #[test]
    fn test_opt_flags() {
        let flags = OptFlags::EXPAND.union(OptFlags::COMMA);
        assert!(flags.contains(OptFlags::EXPAND));
        assert!(flags.contains(OptFlags::COMMA));
        assert!(!flags.contains(OptFlags::SECURE));
    }

    #[test]
    fn test_set_op_repr() {
        assert_eq!(SetOp::None as i32, 0);
        assert_eq!(SetOp::Adding as i32, 1);
        assert_eq!(SetOp::Prepending as i32, 2);
        assert_eq!(SetOp::Removing as i32, 3);
    }

    #[test]
    fn test_set_prefix_repr() {
        assert_eq!(SetPrefix::No as i32, 0);
        assert_eq!(SetPrefix::None as i32, 1);
        assert_eq!(SetPrefix::Inv as i32, 2);
    }

    #[test]
    fn test_option_set_flags() {
        let flags = OptionSetFlags::GLOBAL.union(OptionSetFlags::LOCAL);
        assert!(flags.contains(OptionSetFlags::GLOBAL));
        assert!(flags.contains(OptionSetFlags::LOCAL));
        assert!(!flags.contains(OptionSetFlags::MODELINE));
    }

    #[test]
    fn test_validate_nonnegative() {
        assert_eq!(rs_validate_nonnegative(0), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_nonnegative(100), ValidationError::Ok as c_int);
        assert_eq!(
            rs_validate_nonnegative(-1),
            ValidationError::NeedPositive as c_int
        );
    }

    #[test]
    fn test_validate_positive() {
        assert_eq!(rs_validate_positive(1), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_positive(100), ValidationError::Ok as c_int);
        assert_eq!(
            rs_validate_positive(0),
            ValidationError::NeedPositive as c_int
        );
        assert_eq!(
            rs_validate_positive(-1),
            ValidationError::NeedPositive as c_int
        );
    }

    #[test]
    fn test_validate_range() {
        assert_eq!(rs_validate_range(5, 0, 10), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_range(0, 0, 10), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_range(10, 0, 10), ValidationError::Ok as c_int);
        assert_eq!(
            rs_validate_range(-1, 0, 10),
            ValidationError::OutOfRange as c_int
        );
        assert_eq!(
            rs_validate_range(11, 0, 10),
            ValidationError::OutOfRange as c_int
        );
    }

    #[test]
    fn test_clamp_value() {
        assert_eq!(rs_clamp_value(5, 0, 10), 5);
        assert_eq!(rs_clamp_value(-5, 0, 10), 0);
        assert_eq!(rs_clamp_value(15, 0, 10), 10);
        assert_eq!(rs_clamp_value(0, 0, 10), 0);
        assert_eq!(rs_clamp_value(10, 0, 10), 10);
    }

    #[test]
    fn test_validate_regexpengine() {
        assert_eq!(rs_validate_regexpengine(0), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_regexpengine(1), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_regexpengine(2), ValidationError::Ok as c_int);
        assert_eq!(
            rs_validate_regexpengine(-1),
            ValidationError::InvalidArg as c_int
        );
        assert_eq!(
            rs_validate_regexpengine(3),
            ValidationError::InvalidArg as c_int
        );
    }

    #[test]
    fn test_validate_history() {
        assert_eq!(rs_validate_history(0), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_history(100), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_history(10000), ValidationError::Ok as c_int);
        assert_eq!(
            rs_validate_history(-1),
            ValidationError::NeedPositive as c_int
        );
        assert_eq!(
            rs_validate_history(10001),
            ValidationError::InvalidArg as c_int
        );
    }

    #[test]
    fn test_validate_percentage() {
        assert_eq!(rs_validate_percentage(0), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_percentage(50), ValidationError::Ok as c_int);
        assert_eq!(rs_validate_percentage(100), ValidationError::Ok as c_int);
        assert_eq!(
            rs_validate_percentage(-1),
            ValidationError::OutOfRange as c_int
        );
        assert_eq!(
            rs_validate_percentage(101),
            ValidationError::OutOfRange as c_int
        );
    }

    #[test]
    fn test_clamp_percentage() {
        assert_eq!(rs_clamp_percentage(50), 50);
        assert_eq!(rs_clamp_percentage(-10), 0);
        assert_eq!(rs_clamp_percentage(150), 100);
    }

    #[test]
    fn test_option_has_no_prefix() {
        use std::ffi::CString;

        unsafe {
            let no_number = CString::new("nonumber").unwrap();
            let number = CString::new("number").unwrap();
            let inv_number = CString::new("invnumber").unwrap();

            assert_eq!(rs_option_has_no_prefix(no_number.as_ptr()), 2);
            assert_eq!(rs_option_has_no_prefix(number.as_ptr()), 0);
            assert_eq!(rs_option_has_no_prefix(inv_number.as_ptr()), 0);
            assert_eq!(rs_option_has_no_prefix(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_option_has_inv_prefix() {
        use std::ffi::CString;

        unsafe {
            let inv_number = CString::new("invnumber").unwrap();
            let no_number = CString::new("nonumber").unwrap();
            let number = CString::new("number").unwrap();

            assert_eq!(rs_option_has_inv_prefix(inv_number.as_ptr()), 3);
            assert_eq!(rs_option_has_inv_prefix(no_number.as_ptr()), 0);
            assert_eq!(rs_option_has_inv_prefix(number.as_ptr()), 0);
            assert_eq!(rs_option_has_inv_prefix(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_parse_bool_prefix() {
        use std::ffi::CString;

        unsafe {
            let no_number = CString::new("nonumber").unwrap();
            let inv_number = CString::new("invnumber").unwrap();
            let number = CString::new("number").unwrap();

            assert_eq!(
                rs_parse_bool_prefix(no_number.as_ptr()),
                SetPrefix::No as c_int
            );
            assert_eq!(
                rs_parse_bool_prefix(inv_number.as_ptr()),
                SetPrefix::Inv as c_int
            );
            assert_eq!(
                rs_parse_bool_prefix(number.as_ptr()),
                SetPrefix::None as c_int
            );
        }
    }

    #[test]
    fn test_option_chars_valid() {
        use std::ffi::CString;

        unsafe {
            let flags = CString::new("abc").unwrap();
            let allowed = CString::new("abcdef").unwrap();
            let invalid = CString::new("xyz").unwrap();

            assert_eq!(rs_option_chars_valid(flags.as_ptr(), allowed.as_ptr()), 1);
            assert_eq!(rs_option_chars_valid(invalid.as_ptr(), allowed.as_ptr()), 0);
        }
    }

    #[test]
    fn test_option_match_keyword() {
        use std::ffi::CString;

        unsafe {
            let all = CString::new("all").unwrap();
            let all_more = CString::new("all:test").unwrap();
            let all_continued = CString::new("allmore").unwrap();
            let keyword = CString::new("all").unwrap();

            assert_eq!(
                rs_option_match_keyword(all.as_ptr(), keyword.as_ptr(), 3),
                1
            );
            assert_eq!(
                rs_option_match_keyword(all_more.as_ptr(), keyword.as_ptr(), 3),
                1
            );
            // "allmore" doesn't match because there's more alphanumeric after
            assert_eq!(
                rs_option_match_keyword(all_continued.as_ptr(), keyword.as_ptr(), 3),
                0
            );
        }
    }

    #[test]
    fn test_option_skip_whitespace() {
        use std::ffi::CString;

        unsafe {
            let with_spaces = CString::new("  \t  test").unwrap();
            let no_spaces = CString::new("test").unwrap();

            let result = rs_option_skip_whitespace(with_spaces.as_ptr());
            assert_eq!(*result as u8, b't');

            let result = rs_option_skip_whitespace(no_spaces.as_ptr());
            assert_eq!(*result as u8, b't');
        }
    }

    #[test]
    fn test_option_find_arg_end() {
        use std::ffi::CString;

        unsafe {
            let with_space = CString::new("arg1 arg2").unwrap();
            let with_comma = CString::new("arg1,arg2").unwrap();
            let no_delim = CString::new("arg1").unwrap();

            let result = rs_option_find_arg_end(with_space.as_ptr());
            assert_eq!(result.offset_from(with_space.as_ptr()), 4);

            let result = rs_option_find_arg_end(with_comma.as_ptr());
            assert_eq!(result.offset_from(with_comma.as_ptr()), 4);

            let result = rs_option_find_arg_end(no_delim.as_ptr());
            assert_eq!(*result, 0); // points to NUL
        }
    }

    #[test]
    fn test_is_valid_option_name_start() {
        // Valid start characters
        assert!(is_valid_option_name_start(b'a'));
        assert!(is_valid_option_name_start(b'z'));
        assert!(is_valid_option_name_start(b'A'));
        assert!(is_valid_option_name_start(b'Z'));
        assert!(is_valid_option_name_start(b'_'));

        // Invalid start characters
        assert!(!is_valid_option_name_start(b'0'));
        assert!(!is_valid_option_name_start(b'9'));
        assert!(!is_valid_option_name_start(b'-'));
        assert!(!is_valid_option_name_start(b' '));
    }

    #[test]
    fn test_is_valid_option_name_char() {
        // Valid characters
        assert!(is_valid_option_name_char(b'a'));
        assert!(is_valid_option_name_char(b'z'));
        assert!(is_valid_option_name_char(b'A'));
        assert!(is_valid_option_name_char(b'Z'));
        assert!(is_valid_option_name_char(b'0'));
        assert!(is_valid_option_name_char(b'9'));
        assert!(is_valid_option_name_char(b'_'));

        // Invalid characters
        assert!(!is_valid_option_name_char(b'-'));
        assert!(!is_valid_option_name_char(b' '));
        assert!(!is_valid_option_name_char(b'.'));
    }

    #[test]
    fn test_validate_option_name() {
        use std::ffi::CString;

        unsafe {
            // Valid names
            let valid1 = CString::new("autoindent").unwrap();
            let valid2 = CString::new("_private").unwrap();
            let valid3 = CString::new("opt123").unwrap();
            let valid4 = CString::new("a").unwrap();

            assert_eq!(rs_validate_option_name(valid1.as_ptr()), 1);
            assert_eq!(rs_validate_option_name(valid2.as_ptr()), 1);
            assert_eq!(rs_validate_option_name(valid3.as_ptr()), 1);
            assert_eq!(rs_validate_option_name(valid4.as_ptr()), 1);

            // Invalid names
            let invalid1 = CString::new("123abc").unwrap(); // starts with digit
            let invalid2 = CString::new("-opt").unwrap(); // starts with hyphen
            let invalid3 = CString::new("opt-name").unwrap(); // contains hyphen
            let empty = CString::new("").unwrap();

            assert_eq!(rs_validate_option_name(invalid1.as_ptr()), 0);
            assert_eq!(rs_validate_option_name(invalid2.as_ptr()), 0);
            assert_eq!(rs_validate_option_name(invalid3.as_ptr()), 0);
            assert_eq!(rs_validate_option_name(empty.as_ptr()), 0);
            assert_eq!(rs_validate_option_name(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_option_name_length() {
        use std::ffi::CString;

        unsafe {
            let full = CString::new("autoindent").unwrap();
            let with_extra = CString::new("option=value").unwrap();
            let with_space = CString::new("opt ion").unwrap();
            let digit_start = CString::new("123abc").unwrap();

            assert_eq!(rs_option_name_length(full.as_ptr()), 10);
            assert_eq!(rs_option_name_length(with_extra.as_ptr()), 6);
            assert_eq!(rs_option_name_length(with_space.as_ptr()), 3);
            assert_eq!(rs_option_name_length(digit_start.as_ptr()), 0);
            assert_eq!(rs_option_name_length(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_parse_set_operator() {
        use std::ffi::CString;

        unsafe {
            // Test += (Adding)
            let adding = CString::new("+=value").unwrap();
            let mut p = adding.as_ptr();
            let op = rs_parse_set_operator(&raw mut p);
            assert_eq!(op, SetOp::Adding as c_int);
            assert_eq!(*p as u8, b'v');

            // Test ^= (Prepending)
            let prepending = CString::new("^=value").unwrap();
            let mut p = prepending.as_ptr();
            let op = rs_parse_set_operator(&raw mut p);
            assert_eq!(op, SetOp::Prepending as c_int);
            assert_eq!(*p as u8, b'v');

            // Test -= (Removing)
            let removing = CString::new("-=value").unwrap();
            let mut p = removing.as_ptr();
            let op = rs_parse_set_operator(&raw mut p);
            assert_eq!(op, SetOp::Removing as c_int);
            assert_eq!(*p as u8, b'v');

            // Test = (None - assignment)
            let assign = CString::new("=value").unwrap();
            let mut p = assign.as_ptr();
            let op = rs_parse_set_operator(&raw mut p);
            assert_eq!(op, SetOp::None as c_int);
            assert_eq!(*p as u8, b'v');

            // Test : (None - assignment alternative)
            let colon = CString::new(":value").unwrap();
            let mut p = colon.as_ptr();
            let op = rs_parse_set_operator(&raw mut p);
            assert_eq!(op, SetOp::None as c_int);
            assert_eq!(*p as u8, b'v');
        }
    }
}
