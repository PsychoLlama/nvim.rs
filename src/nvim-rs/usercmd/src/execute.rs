//! User command execution handling
//!
//! This module provides Rust implementations for user command execution,
//! including execution context, modifiers, and result handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

use crate::define::EX_NOSPC;
use crate::{ucmds, CmdmodHandle, ExargHandle, UcmdT};

/// Line number type
type LinenrT = i32;

// =============================================================================
// CMOD_* Constants — from ex_cmds_defs.h
// =============================================================================

pub const CMOD_SANDBOX: c_int = 0x0001;
pub const CMOD_SILENT: c_int = 0x0002;
pub const CMOD_ERRSILENT: c_int = 0x0004;
pub const CMOD_UNSILENT: c_int = 0x0008;
pub const CMOD_NOAUTOCMD: c_int = 0x0010;
pub const CMOD_HIDE: c_int = 0x0020;
pub const CMOD_BROWSE: c_int = 0x0040;
pub const CMOD_CONFIRM: c_int = 0x0080;
pub const CMOD_KEEPALT: c_int = 0x0100;
pub const CMOD_KEEPMARKS: c_int = 0x0200;
pub const CMOD_KEEPJUMPS: c_int = 0x0400;
pub const CMOD_LOCKMARKS: c_int = 0x0800;
pub const CMOD_KEEPPATTERNS: c_int = 0x1000;
pub const CMOD_NOSWAPFILE: c_int = 0x2000;

// =============================================================================
// WSP_* Constants — from window.h
// =============================================================================

pub const WSP_ROOM: c_int = 0x01;
pub const WSP_VERT: c_int = 0x02;
pub const WSP_HOR: c_int = 0x04;
pub const WSP_TOP: c_int = 0x08;
pub const WSP_BOT: c_int = 0x10;
pub const WSP_HELP: c_int = 0x20;
pub const WSP_BELOW: c_int = 0x40;
pub const WSP_ABOVE: c_int = 0x80;
pub const WSP_NEWLOC: c_int = 0x100;
pub const WSP_NOENTER: c_int = 0x200;

// =============================================================================
// K_SPECIAL / KS_SPECIAL / KE_FILLER — from keycodes.h
// =============================================================================

pub const K_SPECIAL: u8 = 0x80;
pub const KS_SPECIAL: u8 = 254;
pub const KE_FILLER: u8 = b'X'; // 0x58

// =============================================================================
// Execution Modifiers (internal Rust flags)
// =============================================================================

/// Command modifier flags (internal Rust tracking, NOT matching C)
pub const MOD_SILENT: u32 = 0x0001;
pub const MOD_VERTICAL: u32 = 0x0002;
pub const MOD_HORIZONTAL: u32 = 0x0004;
pub const MOD_TOPLEFT: u32 = 0x0008;
pub const MOD_BOTRIGHT: u32 = 0x0010;
pub const MOD_LEFTABOVE: u32 = 0x0020;
pub const MOD_RIGHTBELOW: u32 = 0x0040;
pub const MOD_TAB: u32 = 0x0080;
pub const MOD_CONFIRM: u32 = 0x0100;
pub const MOD_KEEPALT: u32 = 0x0200;
pub const MOD_KEEPJUMPS: u32 = 0x0400;
pub const MOD_KEEPMARKS: u32 = 0x0800;
pub const MOD_KEEPPATTERNS: u32 = 0x1000;
pub const MOD_LOCKMARKS: u32 = 0x2000;
pub const MOD_NOAUTOCMD: u32 = 0x4000;
pub const MOD_NOSWAPFILE: u32 = 0x8000;
pub const MOD_HIDE: u32 = 0x10000;
pub const MOD_BROWSE: u32 = 0x20000;

/// Command modifiers wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdModifiers {
    flags: u32,
    /// Tab page number for :tab
    tab: c_int,
    /// Count before modifier
    count: c_int,
}

impl CmdModifiers {
    /// Create with no modifiers
    pub const fn none() -> Self {
        Self {
            flags: 0,
            tab: 0,
            count: 0,
        }
    }

    /// Create from raw flags
    pub const fn from_raw(flags: u32) -> Self {
        Self {
            flags,
            tab: 0,
            count: 0,
        }
    }

    /// Get raw flags
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if silent
    pub const fn is_silent(self) -> bool {
        (self.flags & MOD_SILENT) != 0
    }

    /// Check if vertical split
    pub const fn is_vertical(self) -> bool {
        (self.flags & MOD_VERTICAL) != 0
    }

    /// Check if horizontal split
    pub const fn is_horizontal(self) -> bool {
        (self.flags & MOD_HORIZONTAL) != 0
    }

    /// Check if top-left position
    pub const fn is_topleft(self) -> bool {
        (self.flags & MOD_TOPLEFT) != 0
    }

    /// Check if bottom-right position
    pub const fn is_botright(self) -> bool {
        (self.flags & MOD_BOTRIGHT) != 0
    }

    /// Check if new tab
    pub const fn is_tab(self) -> bool {
        (self.flags & MOD_TAB) != 0
    }

    /// Check if confirm mode
    pub const fn is_confirm(self) -> bool {
        (self.flags & MOD_CONFIRM) != 0
    }

    /// Check if keepalt
    pub const fn is_keepalt(self) -> bool {
        (self.flags & MOD_KEEPALT) != 0
    }

    /// Check if keepjumps
    pub const fn is_keepjumps(self) -> bool {
        (self.flags & MOD_KEEPJUMPS) != 0
    }

    /// Check if noautocmd
    pub const fn is_noautocmd(self) -> bool {
        (self.flags & MOD_NOAUTOCMD) != 0
    }

    /// Check if browse mode
    pub const fn is_browse(self) -> bool {
        (self.flags & MOD_BROWSE) != 0
    }

    /// Get tab number
    pub const fn tab_number(self) -> c_int {
        self.tab
    }

    /// Set silent flag
    pub fn set_silent(&mut self, value: bool) {
        if value {
            self.flags |= MOD_SILENT;
        } else {
            self.flags &= !MOD_SILENT;
        }
    }

    /// Set vertical flag
    pub fn set_vertical(&mut self, value: bool) {
        if value {
            self.flags |= MOD_VERTICAL;
        } else {
            self.flags &= !MOD_VERTICAL;
        }
    }
}

// =============================================================================
// Execution Context
// =============================================================================

/// Context for user command execution
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExecContext {
    /// Command modifiers
    pub modifiers: CmdModifiers,
    /// First line of range
    pub line1: LinenrT,
    /// Last line of range
    pub line2: LinenrT,
    /// Whether range was given
    pub range_given: bool,
    /// Whether bang (!) was used
    pub bang: bool,
    /// Count value (-1 if not given)
    pub count: c_int,
    /// Register name (0 if not given)
    pub reg: u8,
}

impl Default for ExecContext {
    fn default() -> Self {
        Self {
            modifiers: CmdModifiers::none(),
            line1: 1,
            line2: 1,
            range_given: false,
            bang: false,
            count: -1,
            reg: 0,
        }
    }
}

impl ExecContext {
    /// Create a new execution context
    pub const fn new() -> Self {
        Self {
            modifiers: CmdModifiers::none(),
            line1: 1,
            line2: 1,
            range_given: false,
            bang: false,
            count: -1,
            reg: 0,
        }
    }

    /// Check if a range was given
    pub const fn has_range(&self) -> bool {
        self.range_given
    }

    /// Check if a count was given
    pub const fn has_count(&self) -> bool {
        self.count >= 0
    }

    /// Check if a register was given
    pub const fn has_register(&self) -> bool {
        self.reg != 0
    }

    /// Get the number of lines in the range
    pub const fn line_count(&self) -> LinenrT {
        if self.range_given {
            self.line2 - self.line1 + 1
        } else {
            1
        }
    }

    /// Check if this is a single line range
    pub const fn is_single_line(&self) -> bool {
        self.line1 == self.line2
    }
}

// =============================================================================
// Execution Result
// =============================================================================

/// Result of command execution
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecResult {
    /// Command executed successfully
    Success = 0,
    /// Command failed
    Failure = 1,
    /// Command was interrupted
    Interrupted = 2,
    /// Command not found
    NotFound = 3,
    /// Invalid arguments
    InvalidArgs = 4,
    /// Permission denied
    Permission = 5,
    /// Range error
    RangeError = 6,
    /// Command is disabled
    Disabled = 7,
}

impl ExecResult {
    /// Check if execution was successful
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if execution failed
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }

    /// Convert to raw integer
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Success,
            1 => Self::Failure,
            2 => Self::Interrupted,
            3 => Self::NotFound,
            4 => Self::InvalidArgs,
            5 => Self::Permission,
            6 => Self::RangeError,
            7 => Self::Disabled,
            _ => Self::Failure,
        }
    }
}

// =============================================================================
// Execution State
// =============================================================================

/// State during command execution
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExecState {
    /// Whether currently executing
    pub executing: bool,
    /// Nesting level (for recursive commands)
    pub level: c_int,
    /// Whether output is being captured
    pub capturing: bool,
    /// Whether errors should be suppressed
    pub silent_errors: bool,
}

impl Default for ExecState {
    fn default() -> Self {
        Self {
            executing: false,
            level: 0,
            capturing: false,
            silent_errors: false,
        }
    }
}

impl ExecState {
    /// Check if at top level
    pub const fn is_top_level(&self) -> bool {
        self.level == 0
    }

    /// Check if nested
    pub const fn is_nested(&self) -> bool {
        self.level > 0
    }
}

// =============================================================================
// Special Values for <q-args>, <f-args>, etc.
// =============================================================================

/// Special argument expansion type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialArg {
    /// <args> - raw arguments
    Args = 0,
    /// <q-args> - quoted arguments
    QArgs = 1,
    /// <f-args> - function arguments (split)
    FArgs = 2,
    /// <bang> - bang (!)
    Bang = 3,
    /// <line1> - first line
    Line1 = 4,
    /// <line2> - last line
    Line2 = 5,
    /// <count> - count value
    Count = 6,
    /// <reg> - register
    Reg = 7,
    /// <mods> - modifiers
    Mods = 8,
    /// <lt> - literal <
    Lt = 9,
}

impl SpecialArg {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Args),
            1 => Some(Self::QArgs),
            2 => Some(Self::FArgs),
            3 => Some(Self::Bang),
            4 => Some(Self::Line1),
            5 => Some(Self::Line2),
            6 => Some(Self::Count),
            7 => Some(Self::Reg),
            8 => Some(Self::Mods),
            9 => Some(Self::Lt),
            _ => None,
        }
    }

    /// Get the placeholder name (without <>)
    pub const fn name(self) -> &'static str {
        match self {
            Self::Args => "args",
            Self::QArgs => "q-args",
            Self::FArgs => "f-args",
            Self::Bang => "bang",
            Self::Line1 => "line1",
            Self::Line2 => "line2",
            Self::Count => "count",
            Self::Reg => "reg",
            Self::Mods => "mods",
            Self::Lt => "lt",
        }
    }

    /// Check if this requires the execution context
    pub const fn needs_context(self) -> bool {
        !matches!(self, Self::Lt)
    }
}

// =============================================================================
// C Accessor Functions (Phase 3 — modifier string generation)
// =============================================================================

extern "C" {
    /// Get cmod->cmod_split field
    fn nvim_uc_cmod_get_split(cmod: CmdmodHandle) -> c_int;
    /// Get cmod->cmod_flags field
    fn nvim_uc_cmod_get_flags(cmod: CmdmodHandle) -> c_int;
    /// Get cmod->cmod_tab field
    fn nvim_uc_cmod_get_tab(cmod: CmdmodHandle) -> c_int;
    /// Get cmod->cmod_verbose field
    fn nvim_uc_cmod_get_verbose(cmod: CmdmodHandle) -> c_int;
    fn nvim_get_curtab() -> *mut std::ffi::c_void;
    #[link_name = "rs_tabpage_index"]
    fn nvim_rs_tabpage_index(tp: *mut std::ffi::c_void) -> c_int;
}

// =============================================================================
// C Accessor Functions (Phase 4 — argument expansion)
// =============================================================================

extern "C" {
    // exarg_T accessors
    /// Get eap->arg (const char *)
    fn nvim_uc_eap_get_arg(eap: ExargHandle) -> *const c_char;
    /// Get eap->argt (uint32_t)
    fn nvim_uc_eap_get_argt(eap: ExargHandle) -> u32;
    /// Get eap->forceit (int, used as bool)
    fn nvim_uc_eap_get_forceit(eap: ExargHandle) -> c_int;
    /// Get eap->line1 (linenr_T = int32)
    fn nvim_uc_eap_get_line1(eap: ExargHandle) -> c_int;
    /// Get eap->line2 (linenr_T = int32)
    fn nvim_uc_eap_get_line2(eap: ExargHandle) -> c_int;
    /// Get eap->addr_count (int)
    fn nvim_uc_eap_get_addr_count(eap: ExargHandle) -> c_int;
    /// Get eap->regname (int)
    fn nvim_uc_eap_get_regname(eap: ExargHandle) -> c_int;
    /// Get eap->args (char **)
    fn nvim_uc_eap_get_args(eap: ExargHandle) -> *const *const c_char;
    /// Get eap->arglens (size_t *)
    fn nvim_uc_eap_get_arglens(eap: ExargHandle) -> *const usize;
    /// Get eap->argc (size_t)
    fn nvim_uc_eap_get_argc(eap: ExargHandle) -> usize;

    // Multibyte helpers
    /// utfc_ptr2len(p) — byte length of a UTF-8 character
    #[link_name = "utfc_ptr2len"]
    fn nvim_uc_utfc_ptr2len(p: *const c_char) -> c_int;
    /// mb_copy_char(pp, qq) — copy one multi-byte char, advance both pointers
    #[link_name = "mb_copy_char"]
    fn nvim_uc_mb_copy_char(pp: *mut *const c_char, qq: *mut *mut c_char);

    // Memory allocation
    /// xmalloc(size) — returns *mut c_void (same as C void *)
    #[link_name = "xmalloc"]
    fn nvim_uc_xmalloc(size: usize) -> *mut c_void;

    // Global cmdmod pointer
    /// Get pointer to global cmdmod struct
    fn nvim_uc_get_cmdmod() -> CmdmodHandle;
}

// =============================================================================
// Modifier String Generation (Phase 3)
// =============================================================================

/// Table of simple flag-based modifier entries for `uc_mods`.
/// Order matches the C `mod_entries[]` table exactly.
const MOD_ENTRIES: &[(c_int, &[u8])] = &[
    (CMOD_BROWSE, b"browse"),
    (CMOD_CONFIRM, b"confirm"),
    (CMOD_HIDE, b"hide"),
    (CMOD_KEEPALT, b"keepalt"),
    (CMOD_KEEPJUMPS, b"keepjumps"),
    (CMOD_KEEPMARKS, b"keepmarks"),
    (CMOD_KEEPPATTERNS, b"keeppatterns"),
    (CMOD_LOCKMARKS, b"lockmarks"),
    (CMOD_NOSWAPFILE, b"noswapfile"),
    (CMOD_UNSILENT, b"unsilent"),
    (CMOD_NOAUTOCMD, b"noautocmd"),
    (CMOD_SANDBOX, b"sandbox"),
];

/// Internal helper: append a modifier string, preceded by a space if
/// `*multi_mods` is true.  Returns the number of bytes that would be
/// (or were) written.
///
/// * `buf` – if `Some`, the modifier text is appended via `strcat`-style
///   writes (the buffer must already be NUL-terminated).
///   If `None`, only the length is computed (measure pass).
/// * `mod_str` – the modifier keyword (e.g. `b"keepalt"`).
/// * `multi_mods` – set to `true` after the first call so subsequent
///   modifiers are separated by a space.
fn add_cmd_modifier(buf: *mut c_char, mod_str: &[u8], multi_mods: &mut bool) -> usize {
    let mut result = mod_str.len();
    if *multi_mods {
        result += 1;
    }

    if !buf.is_null() {
        unsafe {
            // Find the NUL terminator (equivalent to strcat finding the end)
            let mut p = buf;
            while *p != 0 {
                p = p.add(1);
            }
            // Append space separator if needed
            if *multi_mods {
                *p = b' ' as c_char;
                p = p.add(1);
            }
            // Append the modifier string
            std::ptr::copy_nonoverlapping(mod_str.as_ptr().cast::<c_char>(), p, mod_str.len());
            p = p.add(mod_str.len());
            // NUL-terminate
            *p = 0;
        }
    }

    *multi_mods = true;
    result
}

/// Generate window-split modifier strings from `cmod->cmod_split` and
/// `cmod->cmod_tab`.
///
/// When `buf` is non-NULL the text is appended (the buffer must be
/// NUL-terminated on entry).  Returns the number of bytes added.
fn add_win_cmd_modifiers_impl(
    buf: *mut c_char,
    cmod: CmdmodHandle,
    multi_mods: &mut bool,
) -> usize {
    let split = unsafe { nvim_uc_cmod_get_split(cmod) };
    let cmod_tab = unsafe { nvim_uc_cmod_get_tab(cmod) };
    let mut result: usize = 0;

    // :aboveleft / :leftabove
    if split & WSP_ABOVE != 0 {
        result += add_cmd_modifier(buf, b"aboveleft", multi_mods);
    }
    // :belowright / :rightbelow
    if split & WSP_BELOW != 0 {
        result += add_cmd_modifier(buf, b"belowright", multi_mods);
    }
    // :botright
    if split & WSP_BOT != 0 {
        result += add_cmd_modifier(buf, b"botright", multi_mods);
    }

    // :tab  (cmod_tab > 0 means ":tab" was used; value is tab_number + 1)
    if cmod_tab > 0 {
        let tabnr = cmod_tab - 1;
        let curtab_idx = unsafe { nvim_rs_tabpage_index(nvim_get_curtab()) };
        if tabnr == curtab_idx {
            result += add_cmd_modifier(buf, b"tab", multi_mods);
        } else {
            // Format "<N>tab" into a stack buffer
            let mut tab_buf = [0u8; 68]; // NUMBUFLEN(65) + 3
            let s = format_int_suffix(tabnr, b"tab", &mut tab_buf);
            result += add_cmd_modifier(buf, s, multi_mods);
        }
    }

    // :topleft
    if split & WSP_TOP != 0 {
        result += add_cmd_modifier(buf, b"topleft", multi_mods);
    }
    // :vertical
    if split & WSP_VERT != 0 {
        result += add_cmd_modifier(buf, b"vertical", multi_mods);
    }
    // :horizontal
    if split & WSP_HOR != 0 {
        result += add_cmd_modifier(buf, b"horizontal", multi_mods);
    }

    result
}

/// Generate the full modifier string for `<mods>` / `<q-mods>` expansion.
///
/// When `buf` is non-NULL the text is written into it.  Returns the total
/// number of bytes (including optional surrounding quotes).
fn uc_mods_impl(buf: *mut c_char, cmod: CmdmodHandle, quote: bool) -> usize {
    let flags = unsafe { nvim_uc_cmod_get_flags(cmod) };
    let cmod_verbose = unsafe { nvim_uc_cmod_get_verbose(cmod) };
    let mut multi_mods = false;

    // Start with space for the quote characters (if any).
    let mut result: usize = if quote { 2 } else { 0 };

    // `work` is the pointer where modifier text will be appended.
    // If quoting, it is advanced past the opening '"' character.
    let work: *mut c_char = if buf.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe {
            let mut p = buf;
            if quote {
                *p = b'"' as c_char;
                p = p.add(1);
            }
            // NUL-terminate so strcat-style appending works
            *p = 0;
            p
        }
    };

    // Simple flag-based modifiers
    for &(flag, name) in MOD_ENTRIES {
        if flags & flag != 0 {
            result += add_cmd_modifier(work, name, &mut multi_mods);
        }
    }

    // :silent  (may be "silent!")
    if flags & CMOD_SILENT != 0 {
        if flags & CMOD_ERRSILENT != 0 {
            result += add_cmd_modifier(work, b"silent!", &mut multi_mods);
        } else {
            result += add_cmd_modifier(work, b"silent", &mut multi_mods);
        }
    }

    // :verbose
    if cmod_verbose > 0 {
        let verbose_value = cmod_verbose - 1;
        if verbose_value == 1 {
            result += add_cmd_modifier(work, b"verbose", &mut multi_mods);
        } else {
            let mut verbose_buf = [0u8; 65]; // NUMBUFLEN
            let s = format_int_suffix(verbose_value, b"verbose", &mut verbose_buf);
            result += add_cmd_modifier(work, s, &mut multi_mods);
        }
    }

    // Window-split modifiers
    result += add_win_cmd_modifiers_impl(work, cmod, &mut multi_mods);

    // Closing quote — overwrites the NUL that strcat left at the end
    if quote && !buf.is_null() {
        unsafe {
            // The opening quote consumed 1 byte, so the modifier text starts
            // at buf+1 and is (result - 2) bytes long.  The closing quote
            // goes right after the modifier text.
            let closing = buf.add(result - 1);
            *closing = b'"' as c_char;
        }
    }

    result
}

/// Format `<number><suffix>` into `out` and return the used slice.
///
/// Example: `format_int_suffix(3, b"tab", &mut buf)` produces `b"3tab"`.
fn format_int_suffix<'a>(n: c_int, suffix: &[u8], out: &'a mut [u8]) -> &'a [u8] {
    // Write the integer digits
    let mut num_buf = [0u8; 20]; // enough for any i32
    let num_str = format_c_int(n, &mut num_buf);
    let total = num_str.len() + suffix.len();
    assert!(total <= out.len(), "format_int_suffix: buffer too small");
    out[..num_str.len()].copy_from_slice(num_str);
    out[num_str.len()..total].copy_from_slice(suffix);
    &out[..total]
}

/// Format a `c_int` as decimal digits into `buf`, returning the used slice.
fn format_c_int(n: c_int, buf: &mut [u8; 20]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }
    let negative = n < 0;
    // Work with the absolute value as u32 to avoid sign-loss warnings.
    let mut val = n.unsigned_abs();
    let mut pos = buf.len();
    while val > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    if negative {
        pos -= 1;
        buf[pos] = b'-';
    }
    &buf[pos..]
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Generate window-split modifier string.
///
/// Replaces C `add_win_cmd_modifiers(buf, cmod, multi_mods)`.
/// C signature has `bool *multi_mods` (pointer to 1-byte bool).
#[export_name = "add_win_cmd_modifiers"]
pub extern "C" fn rs_add_win_cmd_modifiers(
    buf: *mut c_char,
    cmod: CmdmodHandle,
    multi_mods_ptr: *mut bool,
) -> usize {
    let mut multi_mods = unsafe { *multi_mods_ptr };
    let result = add_win_cmd_modifiers_impl(buf, cmod, &mut multi_mods);
    unsafe {
        *multi_mods_ptr = multi_mods;
    }
    result
}

/// FFI export: Generate full modifier string for `<mods>` expansion.
///
/// Replaces C `uc_mods(buf, cmod, quote)`.
/// C signature has `bool quote`.
#[export_name = "uc_mods"]
pub extern "C" fn rs_uc_mods(buf: *mut c_char, cmod: CmdmodHandle, quote: bool) -> usize {
    uc_mods_impl(buf, cmod, quote)
}

/// FFI export: Check if modifiers is silent
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_silent(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_silent())
}

/// FFI export: Check if modifiers is vertical
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_vertical(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_vertical())
}

/// FFI export: Check if modifiers is tab
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_tab(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_tab())
}

/// FFI export: Check if modifiers is noautocmd
#[no_mangle]
pub extern "C" fn rs_usercmd_mods_is_noautocmd(flags: u32) -> c_int {
    c_int::from(CmdModifiers::from_raw(flags).is_noautocmd())
}

/// FFI export: Create default execution context
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_context_new() -> ExecContext {
    ExecContext::new()
}

/// FFI export: Get line count from context
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_line_count(ctx: *const ExecContext) -> LinenrT {
    if ctx.is_null() {
        return 1;
    }
    unsafe { (*ctx).line_count() }
}

/// FFI export: Check if context has range
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_has_range(ctx: *const ExecContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*ctx).has_range() })
}

/// FFI export: Check if result is ok
#[no_mangle]
pub extern "C" fn rs_usercmd_exec_result_is_ok(result: c_int) -> c_int {
    c_int::from(ExecResult::from_raw(result).is_ok())
}

// =============================================================================
// Helpers (Phase 4)
// =============================================================================

/// Check if a byte is ASCII whitespace (space or tab), matching C `ascii_iswhite`.
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Compute the length of a NUL-terminated C string.
///
/// # Safety
/// `s` must point to a valid NUL-terminated C string.
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while unsafe { *p } != 0 {
        p = unsafe { p.add(1) };
    }
    p as usize - s as usize
}

/// Case-insensitive comparison of a raw pointer region against a byte literal.
///
/// Compares `l` bytes starting at `p` against `target`.  Returns true if
/// `target.len() == l` and the bytes match case-insensitively.
///
/// This is the equivalent of `STRNICMP(p, "keyword>", l) == 0` from C.
///
/// # Safety
/// `p` must point to at least `l` readable bytes.
unsafe fn strnicmp_eq_ptr(p: *const c_char, target: &[u8], l: usize) -> bool {
    if target.len() != l {
        return false;
    }
    let slice = unsafe { std::slice::from_raw_parts(p.cast::<u8>(), l) };
    slice.eq_ignore_ascii_case(target)
}

// =============================================================================
// Argument Expansion (Phase 4)
// =============================================================================

/// Code type for `uc_check_code` expansion — mirrors the C `enum { ct_ARGS, ... }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeType {
    Args,
    Bang,
    Count,
    Line1,
    Line2,
    Range,
    Mods,
    Register,
    Lt,
    None,
}

/// Detect the quote prefix (q/Q/f/F) and return the quote mode (0, 1, or 2)
/// plus the number of bytes consumed (0 or 2).
///
/// This matches the C logic:
/// ```c
/// if ((vim_strchr("qQfF", *p) != NULL) && p[1] == '-') {
///     quote = (*p == 'q' || *p == 'Q') ? 1 : 2;
///     p += 2; l -= 2;
/// }
/// ```
fn detect_quote_prefix(first: u8, second: u8) -> (c_int, usize) {
    match first {
        b'q' | b'Q' if second == b'-' => (1, 2),
        b'f' | b'F' if second == b'-' => (2, 2),
        _ => (0, 0),
    }
}

/// Identify the `CodeType` from a keyword region.
///
/// `p` points to the first byte of the keyword (after any quote prefix),
/// `l` is the comparison length (includes the trailing `>`).
///
/// # Safety
/// `p` must point to at least `l` readable bytes.
unsafe fn identify_code_type(p: *const c_char, l: usize) -> CodeType {
    if l <= 1 {
        return CodeType::None;
    }
    if unsafe { strnicmp_eq_ptr(p, b"args>", l) } {
        CodeType::Args
    } else if unsafe { strnicmp_eq_ptr(p, b"bang>", l) } {
        CodeType::Bang
    } else if unsafe { strnicmp_eq_ptr(p, b"count>", l) } {
        CodeType::Count
    } else if unsafe { strnicmp_eq_ptr(p, b"line1>", l) } {
        CodeType::Line1
    } else if unsafe { strnicmp_eq_ptr(p, b"line2>", l) } {
        CodeType::Line2
    } else if unsafe { strnicmp_eq_ptr(p, b"range>", l) } {
        CodeType::Range
    } else if unsafe { strnicmp_eq_ptr(p, b"lt>", l) } {
        CodeType::Lt
    } else if unsafe { strnicmp_eq_ptr(p, b"reg>", l) }
        || unsafe { strnicmp_eq_ptr(p, b"register>", l) }
    {
        CodeType::Register
    } else if unsafe { strnicmp_eq_ptr(p, b"mods>", l) } {
        CodeType::Mods
    } else {
        CodeType::None
    }
}

/// Split and quote args for `<f-args>`.
///
/// This mirrors the C `uc_split_args` function exactly.  It allocates the
/// result buffer via `nvim_uc_xmalloc` and returns ownership to the caller.
///
/// # Safety
/// All pointer arguments must be valid or NULL as described:
/// - `arg` must be a valid NUL-terminated C string.
/// - When `args` is non-NULL, `arglens` must also be non-NULL and both
///   must have at least `argc` entries.  Each `args[i]` must point to
///   `arglens[i]` readable bytes.
/// - `lenp` must be a valid pointer.
unsafe fn uc_split_args_impl(
    arg: *const c_char,
    args: *const *const c_char,
    arglens: *const usize,
    argc: usize,
    lenp: *mut usize,
) -> *mut c_char {
    // Precalculate length
    let mut len: usize = 2; // Initial and final quotes

    if args.is_null() {
        // Single-string path: process `arg` directly
        let mut p = arg;
        unsafe {
            while *p != 0 {
                let p0 = *p as u8;
                let p1 = *p.add(1) as u8;
                if p0 == b'\\' && p1 == b'\\' {
                    len += 2;
                    p = p.add(2);
                } else if p0 == b'\\' && ascii_iswhite(p1) {
                    len += 1;
                    p = p.add(2);
                } else if p0 == b'\\' || p0 == b'"' {
                    len += 2;
                    p = p.add(1);
                } else if ascii_iswhite(p0) {
                    // Skip whitespace
                    while *p != 0 && ascii_iswhite(*p as u8) {
                        p = p.add(1);
                    }
                    if *p == 0 {
                        break;
                    }
                    len += 4; // ", "
                } else {
                    let charlen = nvim_uc_utfc_ptr2len(p) as usize;
                    len += charlen;
                    p = p.add(charlen);
                }
            }
        }
    } else {
        // Pre-split args path
        unsafe {
            for i in 0..argc {
                let mut p = *args.add(i);
                let arg_end = p.add(*arglens.add(i));
                while (p as usize) < (arg_end as usize) {
                    let ch = *p as u8;
                    if ch == b'\\' || ch == b'"' {
                        len += 2;
                        p = p.add(1);
                    } else {
                        let charlen = nvim_uc_utfc_ptr2len(p) as usize;
                        len += charlen;
                        p = p.add(charlen);
                    }
                }
                if i != argc - 1 {
                    len += 4; // ", "
                }
            }
        }
    }

    // Allocate and fill
    let buf: *mut c_char = unsafe { nvim_uc_xmalloc(len + 1).cast::<c_char>() };
    let mut q = buf;

    unsafe {
        *q = b'"' as c_char;
        q = q.add(1);
    }

    if args.is_null() {
        let mut p = arg;
        unsafe {
            while *p != 0 {
                let p0 = *p as u8;
                let p1 = *p.add(1) as u8;
                if p0 == b'\\' && p1 == b'\\' {
                    *q = b'\\' as c_char;
                    q = q.add(1);
                    *q = b'\\' as c_char;
                    q = q.add(1);
                    p = p.add(2);
                } else if p0 == b'\\' && ascii_iswhite(p1) {
                    *q = p1 as c_char;
                    q = q.add(1);
                    p = p.add(2);
                } else if p0 == b'\\' || p0 == b'"' {
                    *q = b'\\' as c_char;
                    q = q.add(1);
                    *q = *p;
                    q = q.add(1);
                    p = p.add(1);
                } else if ascii_iswhite(p0) {
                    // Skip whitespace
                    while *p != 0 && ascii_iswhite(*p as u8) {
                        p = p.add(1);
                    }
                    if *p == 0 {
                        break;
                    }
                    *q = b'"' as c_char;
                    q = q.add(1);
                    *q = b',' as c_char;
                    q = q.add(1);
                    *q = b' ' as c_char;
                    q = q.add(1);
                    *q = b'"' as c_char;
                    q = q.add(1);
                } else {
                    let mut pp = p.cast::<c_char>();
                    let mut qq = q;
                    nvim_uc_mb_copy_char(&raw mut pp, &raw mut qq);
                    p = pp;
                    q = qq;
                }
            }
        }
    } else {
        unsafe {
            for i in 0..argc {
                let mut p = *args.add(i);
                let arg_end = p.add(*arglens.add(i));
                while (p as usize) < (arg_end as usize) {
                    let ch = *p as u8;
                    if ch == b'\\' || ch == b'"' {
                        *q = b'\\' as c_char;
                        q = q.add(1);
                        *q = *p;
                        q = q.add(1);
                        p = p.add(1);
                    } else {
                        let mut pp = p.cast::<c_char>();
                        let mut qq = q;
                        nvim_uc_mb_copy_char(&raw mut pp, &raw mut qq);
                        p = pp;
                        q = qq;
                    }
                }
                if i != argc - 1 {
                    *q = b'"' as c_char;
                    q = q.add(1);
                    *q = b',' as c_char;
                    q = q.add(1);
                    *q = b' ' as c_char;
                    q = q.add(1);
                    *q = b'"' as c_char;
                    q = q.add(1);
                }
            }
        }
    }

    unsafe {
        *q = b'"' as c_char;
        q = q.add(1);
        *q = 0;
        *lenp = len;
    }

    buf
}

/// Expand `<>` codes like `<args>`, `<bang>`, `<count>`, `<mods>`, etc.
///
/// This mirrors the C `uc_check_code` function exactly.
///
/// When `buf` is NULL, only the length is computed (measure pass).
/// When `buf` is non-NULL, the expanded text is written.
///
/// Returns the number of bytes produced, or `usize::MAX` (i.e. `(size_t)-1`
/// in C) when the code is not recognized.
///
/// # Safety
/// - `code` must point to a valid `<...>` sequence of at least `len` bytes.
/// - `cmd` and `eap` must be valid opaque handles.
/// - `split_buf` and `split_len` must be valid pointers.
/// - When `buf` is non-NULL it must have enough space for the expansion.
unsafe fn uc_check_code_impl(
    code: *mut c_char,
    len: usize,
    buf: *mut c_char,
    cmd: *mut UcmdT,
    eap: ExargHandle,
    split_buf: *mut *mut c_char,
    split_len: *mut usize,
) -> usize {
    let mut p = unsafe { code.add(1) };
    let mut l = len - 2;

    // Detect quote prefix
    let first = unsafe { *p as u8 };
    let second = if l >= 2 {
        unsafe { *p.add(1) as u8 }
    } else {
        0
    };
    let (quote, skip) = detect_quote_prefix(first, second);
    unsafe {
        p = p.add(skip);
    }
    l -= skip;

    // l++ in C (adjusts for the comparison including the '>')
    l += 1;

    // Identify the code type
    let code_type = unsafe { identify_code_type(p, l) };

    match code_type {
        CodeType::Args => unsafe { expand_args(buf, eap, quote, split_buf, split_len) },
        CodeType::Bang => unsafe { expand_bang(buf, eap, quote) },
        CodeType::Line1 | CodeType::Line2 | CodeType::Range | CodeType::Count => unsafe {
            expand_numeric(buf, cmd, eap, code_type, quote)
        },
        CodeType::Mods => {
            let cmod = unsafe { nvim_uc_get_cmdmod() };
            uc_mods_impl(buf, cmod, quote != 0)
        }
        CodeType::Register => unsafe { expand_register(buf, eap, quote) },
        CodeType::Lt => {
            let result = 1;
            if !buf.is_null() {
                unsafe {
                    *buf = b'<' as c_char;
                }
            }
            result
        }
        CodeType::None => {
            // Not recognized: just copy the '<' and return -1 (usize::MAX)
            if !buf.is_null() {
                unsafe {
                    *buf = b'<' as c_char;
                }
            }
            usize::MAX
        }
    }
}

/// Expand `<args>`, `<q-args>`, or `<f-args>`.
///
/// # Safety
/// All pointers must be valid.
unsafe fn expand_args(
    buf: *mut c_char,
    eap: ExargHandle,
    quote: c_int,
    split_buf: *mut *mut c_char,
    split_len: *mut usize,
) -> usize {
    let eap_arg = unsafe { nvim_uc_eap_get_arg(eap) };

    // Empty argument case
    if unsafe { *eap_arg } == 0 {
        if quote == 1 {
            let result = 2;
            if !buf.is_null() {
                unsafe {
                    *buf = b'\'' as c_char;
                    *buf.add(1) = b'\'' as c_char;
                }
            }
            return result;
        }
        return 0;
    }

    // When specified there is a single argument don't split it.
    let argt = unsafe { nvim_uc_eap_get_argt(eap) };
    let effective_quote = if (argt & EX_NOSPC) != 0 && quote == 2 {
        1
    } else {
        quote
    };

    match effective_quote {
        0 => {
            // No quoting, no splitting
            let result = unsafe { c_strlen(eap_arg) };
            if !buf.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(eap_arg.cast::<u8>(), buf.cast::<u8>(), result);
                    *buf.add(result) = 0;
                }
            }
            result
        }
        1 => {
            // Quote, but don't split
            let arg_len = unsafe { c_strlen(eap_arg) };
            let mut result = arg_len + 2; // for surrounding quotes
                                          // Count extra escapes needed
            let mut scan = eap_arg;
            unsafe {
                while *scan != 0 {
                    let ch = *scan as u8;
                    if ch == b'\\' || ch == b'"' {
                        result += 1;
                    }
                    scan = scan.add(1);
                }
            }

            if !buf.is_null() {
                let mut b = buf;
                unsafe {
                    *b = b'"' as c_char;
                    b = b.add(1);
                    scan = eap_arg;
                    while *scan != 0 {
                        let ch = *scan as u8;
                        if ch == b'\\' || ch == b'"' {
                            *b = b'\\' as c_char;
                            b = b.add(1);
                        }
                        *b = *scan;
                        b = b.add(1);
                        scan = scan.add(1);
                    }
                    *b = b'"' as c_char;
                }
            }
            result
        }
        2 => {
            // Quote and split (<f-args>)
            unsafe {
                if (*split_buf).is_null() {
                    let eap_args = nvim_uc_eap_get_args(eap);
                    let eap_arglens = nvim_uc_eap_get_arglens(eap);
                    let eap_argc = nvim_uc_eap_get_argc(eap);
                    *split_buf =
                        uc_split_args_impl(eap_arg, eap_args, eap_arglens, eap_argc, split_len);
                }
                let result = *split_len;
                if !buf.is_null() && result != 0 {
                    std::ptr::copy_nonoverlapping(
                        (*split_buf).cast::<u8>(),
                        buf.cast::<u8>(),
                        result,
                    );
                    *buf.add(result) = 0;
                }
                result
            }
        }
        _ => 0,
    }
}

/// Expand `<bang>`.
///
/// # Safety
/// All pointers must be valid.
unsafe fn expand_bang(buf: *mut c_char, eap: ExargHandle, quote: c_int) -> usize {
    let forceit = unsafe { nvim_uc_eap_get_forceit(eap) } != 0;
    let mut result: usize = usize::from(forceit);
    if quote != 0 {
        result += 2;
    }
    if !buf.is_null() {
        let mut b = buf;
        unsafe {
            if quote != 0 {
                *b = b'"' as c_char;
                b = b.add(1);
            }
            if forceit {
                *b = b'!' as c_char;
                b = b.add(1);
            }
            if quote != 0 {
                *b = b'"' as c_char;
            }
        }
    }
    result
}

/// Expand `<line1>`, `<line2>`, `<range>`, or `<count>`.
///
/// # Safety
/// All pointers must be valid.
unsafe fn expand_numeric(
    buf: *mut c_char,
    cmd: *mut UcmdT,
    eap: ExargHandle,
    code_type: CodeType,
    quote: c_int,
) -> usize {
    let num: i64 = unsafe {
        match code_type {
            CodeType::Line1 => i64::from(nvim_uc_eap_get_line1(eap)),
            CodeType::Line2 => i64::from(nvim_uc_eap_get_line2(eap)),
            CodeType::Range => i64::from(nvim_uc_eap_get_addr_count(eap)),
            CodeType::Count => {
                if nvim_uc_eap_get_addr_count(eap) > 0 {
                    i64::from(nvim_uc_eap_get_line2(eap))
                } else {
                    (*cmd).uc_def
                }
            }
            _ => 0,
        }
    };

    // Format number into a stack buffer
    let mut num_buf = [0u8; 20];
    let num_str = format_i64(num, &mut num_buf);
    let num_len = num_str.len();
    let mut result = num_len;

    if quote != 0 {
        result += 2;
    }

    if !buf.is_null() {
        let mut b = buf;
        unsafe {
            if quote != 0 {
                *b = b'"' as c_char;
                b = b.add(1);
            }
            std::ptr::copy_nonoverlapping(num_str.as_ptr().cast::<c_char>(), b, num_len);
            b = b.add(num_len);
            if quote != 0 {
                *b = b'"' as c_char;
            }
        }
    }

    result
}

/// Expand `<reg>` / `<register>`.
///
/// # Safety
/// All pointers must be valid.
unsafe fn expand_register(buf: *mut c_char, eap: ExargHandle, quote: c_int) -> usize {
    let regname = unsafe { nvim_uc_eap_get_regname(eap) };
    let mut result: usize = usize::from(regname != 0);
    if quote != 0 {
        result += 2;
    }
    if !buf.is_null() {
        let mut b = buf;
        unsafe {
            if quote != 0 {
                *b = b'\'' as c_char;
                b = b.add(1);
            }
            if regname != 0 {
                *b = regname as u8 as c_char;
                b = b.add(1);
            }
            if quote != 0 {
                *b = b'\'' as c_char;
            }
        }
    }
    result
}

/// Format an `i64` as decimal digits into `buf`, returning the used slice.
fn format_i64(n: i64, buf: &mut [u8; 20]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }
    let negative = n < 0;
    let mut val = n.unsigned_abs();
    let mut pos = buf.len();
    while val > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    if negative {
        pos -= 1;
        buf[pos] = b'-';
    }
    &buf[pos..]
}

// =============================================================================
// FFI Exports (Phase 4)
// =============================================================================

/// FFI export: Split and quote args for `<f-args>`.
///
/// Replaces C static `uc_split_args(arg, args, arglens, argc, lenp)`.
#[export_name = "uc_split_args"]
pub unsafe extern "C" fn rs_uc_split_args(
    arg: *const c_char,
    args: *const *const c_char,
    arglens: *const usize,
    argc: usize,
    lenp: *mut usize,
) -> *mut c_char {
    unsafe { uc_split_args_impl(arg, args, arglens, argc, lenp) }
}

/// FFI export: Expand `<>` codes in user commands.
///
/// Replaces C static `uc_check_code(code, len, buf, cmd, eap, split_buf, split_len)`.
#[export_name = "uc_check_code"]
pub unsafe extern "C" fn rs_uc_check_code(
    code: *mut c_char,
    len: usize,
    buf: *mut c_char,
    cmd: *mut UcmdT,
    eap: ExargHandle,
    split_buf: *mut *mut c_char,
    split_len: *mut usize,
) -> usize {
    unsafe { uc_check_code_impl(code, len, buf, cmd, eap, split_buf, split_len) }
}

// =============================================================================
// Constants (Phase 7)
// =============================================================================

/// CMD_USER from ex_cmds_enum.generated.h
const CMD_USER: c_int = -1;

// =============================================================================
// C Accessor Functions (Phase 7 — do_ucmd)
// =============================================================================

extern "C" {
    /// Get eap->cmdidx (as int)
    fn nvim_uc_eap_get_cmdidx(eap: ExargHandle) -> c_int;
    /// Get eap->useridx (int)
    fn nvim_uc_eap_get_useridx(eap: ExargHandle) -> c_int;
    /// Returns USER_CMD_GA(&prevwin_curwin()->w_buffer->b_ucmds, idx)
    fn nvim_uc_prevwin_curwin_buf_ucmd(idx: c_int) -> *mut UcmdT;
    /// Calls nlua_do_ucmd(cmd, eap, preview != 0)
    fn nvim_uc_nlua_do_ucmd(cmd: *mut UcmdT, eap: ExargHandle, preview: c_int) -> c_int;
    /// vim_strchr(p, c)
    #[link_name = "vim_strchr"]
    fn nvim_uc_vim_strchr(p: *const c_char, c: c_int) -> *mut c_char;
    /// Calls do_cmdline with sctx save/restore — kept: complex wrapper
    fn nvim_uc_do_cmdline_with_sctx(buf: *mut c_char, eap: ExargHandle, argt: u32, sc_sid: c_int);
    /// xfree(ptr)
    #[link_name = "xfree"]
    fn nvim_uc_xfree(ptr: *mut c_void);
}

// =============================================================================
// Phase 7: do_ucmd Implementation
// =============================================================================

/// Execute a user command.
///
/// This is the main user command execution function.  It:
/// 1. Looks up the command (global or buffer-local based on cmdidx)
/// 2. If preview: calls nlua_do_ucmd with preview=true, returns result
/// 3. If luaref > 0: calls nlua_do_ucmd with preview=false, returns 0
/// 4. Otherwise: expands `<>` codes in the replacement text (two-pass:
///    measure then fill), handles K_SPECIAL byte sequences, calls do_cmdline,
///    and saves/restores current_sctx
///
/// # Safety
/// `eap` must be a valid ExargHandle (pointer to exarg_T).
unsafe fn do_ucmd_impl(eap: ExargHandle, preview: bool) -> c_int {
    // Look up the command
    let cmdidx = nvim_uc_eap_get_cmdidx(eap);
    let useridx = nvim_uc_eap_get_useridx(eap);
    let cmd: *mut UcmdT = if cmdidx == CMD_USER {
        // USER_CMD(idx) = ucmds.ga_data.cast::<UcmdT>().add(idx)
        unsafe { ucmds.ga_data.cast::<UcmdT>().add(useridx as usize) }
    } else {
        nvim_uc_prevwin_curwin_buf_ucmd(useridx)
    };

    // Preview path
    if preview {
        debug_assert!(unsafe { (*cmd).uc_preview_luaref } > 0);
        return nvim_uc_nlua_do_ucmd(cmd, eap, 1);
    }

    // Lua callback path
    if unsafe { (*cmd).uc_luaref } > 0 {
        nvim_uc_nlua_do_ucmd(cmd, eap, 0);
        return 0;
    }

    // Save argt and sc_sid before the two-pass loop, since after do_cmdline
    // the cmd pointer may become invalid.
    let argt = unsafe { (*cmd).uc_argt };
    let sc_sid = unsafe { (*cmd).uc_script_ctx.sc_sid };

    let mut split_len: usize = 0;
    let mut split_buf: *mut c_char = std::ptr::null_mut();

    // Replace <> in the command by the arguments.
    // First round: buf is NULL, compute length, allocate buf.
    // Second round: copy result into buf.
    let mut buf: *mut c_char = std::ptr::null_mut();
    let uc_rep = unsafe { (*cmd).uc_rep };

    loop {
        let mut p: *mut c_char = uc_rep;
        let mut q: *mut c_char = buf;
        let mut totlen: usize = 0;
        let mut end: *mut c_char;

        loop {
            let start = nvim_uc_vim_strchr(p, c_int::from(b'<'));
            end = if start.is_null() {
                std::ptr::null_mut()
            } else {
                nvim_uc_vim_strchr(start.add(1), c_int::from(b'>'))
            };

            // K_SPECIAL handling — only during the fill pass (buf != NULL)
            if !buf.is_null() {
                // Scan for K_SPECIAL byte between p and the next < (or end of string)
                let mut ksp = p;
                while *ksp != 0 && (*ksp as u8) != K_SPECIAL {
                    ksp = ksp.add(1);
                }
                if (*ksp as u8) == K_SPECIAL
                    && (start.is_null() || (ksp as usize) < (start as usize) || end.is_null())
                    && ((*ksp.add(1) as u8) == KS_SPECIAL && (*ksp.add(2) as u8) == KE_FILLER)
                {
                    // K_SPECIAL has been put in the buffer as K_SPECIAL
                    // KS_SPECIAL KE_FILLER, like for mappings, but
                    // do_cmdline() doesn't handle that, so convert it back.
                    let len = ksp as usize - p as usize;
                    if len > 0 {
                        std::ptr::copy(p, q, len);
                        q = q.add(len);
                    }
                    *q = K_SPECIAL as c_char;
                    q = q.add(1);
                    p = ksp.add(3);
                    continue;
                }
            }

            // Break if no <item> is found
            if start.is_null() || end.is_null() {
                break;
            }

            // Include the '>'
            end = end.add(1);

            // Take everything up to the '<'
            let len = start as usize - p as usize;
            if buf.is_null() {
                totlen += len;
            } else {
                std::ptr::copy(p, q, len);
                q = q.add(len);
            }

            // Expand the <> code
            let mut expanded_len = uc_check_code_impl(
                start,
                end as usize - start as usize,
                q,
                cmd,
                eap,
                &raw mut split_buf,
                &raw mut split_len,
            );
            if expanded_len == usize::MAX {
                // No match, continue after '<'
                p = start.add(1);
                expanded_len = 1;
            } else {
                p = end;
            }
            if buf.is_null() {
                totlen += expanded_len;
            } else {
                q = q.add(expanded_len);
            }
        }

        if !buf.is_null() {
            // Second pass complete — copy trailing characters and NUL
            let trail_len = c_strlen(p);
            std::ptr::copy(p, q, trail_len + 1); // +1 for NUL
            break;
        }

        // Add trailing characters length
        totlen += c_strlen(p);
        buf = nvim_uc_xmalloc(totlen + 1).cast::<c_char>();
    }

    // Execute the command line with sctx save/restore handled in C
    nvim_uc_do_cmdline_with_sctx(buf, eap, argt, sc_sid);

    // Careful: Do not use "cmd" here, it may have become invalid if a user
    // command was added.
    nvim_uc_xfree(buf.cast::<c_void>());
    nvim_uc_xfree(split_buf.cast::<c_void>());

    0
}

// =============================================================================
// FFI Exports (Phase 7)
// =============================================================================

/// FFI export: Execute a user command.
///
/// Direct replacement for C `do_ucmd(eap, preview)`.
/// C signature has `bool preview` (1 byte).
#[export_name = "do_ucmd"]
pub unsafe extern "C" fn rs_do_ucmd(eap: ExargHandle, preview: bool) -> c_int {
    do_ucmd_impl(eap, preview)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmod_constant_values() {
        // Verify CMOD_* constants match ex_cmds_defs.h
        assert_eq!(CMOD_SANDBOX, 0x0001);
        assert_eq!(CMOD_SILENT, 0x0002);
        assert_eq!(CMOD_ERRSILENT, 0x0004);
        assert_eq!(CMOD_UNSILENT, 0x0008);
        assert_eq!(CMOD_NOAUTOCMD, 0x0010);
        assert_eq!(CMOD_HIDE, 0x0020);
        assert_eq!(CMOD_BROWSE, 0x0040);
        assert_eq!(CMOD_CONFIRM, 0x0080);
        assert_eq!(CMOD_KEEPALT, 0x0100);
        assert_eq!(CMOD_KEEPMARKS, 0x0200);
        assert_eq!(CMOD_KEEPJUMPS, 0x0400);
        assert_eq!(CMOD_LOCKMARKS, 0x0800);
        assert_eq!(CMOD_KEEPPATTERNS, 0x1000);
        assert_eq!(CMOD_NOSWAPFILE, 0x2000);
    }

    #[test]
    fn test_wsp_constant_values() {
        // Verify WSP_* constants match window.h
        assert_eq!(WSP_ROOM, 0x01);
        assert_eq!(WSP_VERT, 0x02);
        assert_eq!(WSP_HOR, 0x04);
        assert_eq!(WSP_TOP, 0x08);
        assert_eq!(WSP_BOT, 0x10);
        assert_eq!(WSP_HELP, 0x20);
        assert_eq!(WSP_BELOW, 0x40);
        assert_eq!(WSP_ABOVE, 0x80);
        assert_eq!(WSP_NEWLOC, 0x100);
        assert_eq!(WSP_NOENTER, 0x200);
    }

    #[test]
    fn test_keycode_constant_values() {
        assert_eq!(K_SPECIAL, 0x80);
        assert_eq!(KS_SPECIAL, 254);
        assert_eq!(KE_FILLER, b'X');
    }

    #[test]
    fn test_cmd_modifiers() {
        let mods = CmdModifiers::none();
        assert!(!mods.is_silent());
        assert!(!mods.is_vertical());

        let mods = CmdModifiers::from_raw(MOD_SILENT | MOD_VERTICAL);
        assert!(mods.is_silent());
        assert!(mods.is_vertical());
        assert!(!mods.is_tab());
    }

    #[test]
    fn test_cmd_modifiers_set() {
        let mut mods = CmdModifiers::none();
        mods.set_silent(true);
        assert!(mods.is_silent());

        mods.set_vertical(true);
        assert!(mods.is_vertical());

        mods.set_silent(false);
        assert!(!mods.is_silent());
    }

    #[test]
    fn test_exec_context() {
        let ctx = ExecContext::new();
        assert!(!ctx.has_range());
        assert!(!ctx.has_count());
        assert!(!ctx.has_register());
        assert_eq!(ctx.line_count(), 1);

        let ctx = ExecContext {
            line1: 10,
            line2: 20,
            range_given: true,
            ..Default::default()
        };
        assert!(ctx.has_range());
        assert_eq!(ctx.line_count(), 11);
        assert!(!ctx.is_single_line());
    }

    #[test]
    fn test_exec_result() {
        assert!(ExecResult::Success.is_ok());
        assert!(!ExecResult::Success.is_err());

        assert!(!ExecResult::Failure.is_ok());
        assert!(ExecResult::Failure.is_err());

        assert_eq!(ExecResult::from_raw(0), ExecResult::Success);
        assert_eq!(ExecResult::from_raw(1), ExecResult::Failure);
        assert_eq!(ExecResult::from_raw(100), ExecResult::Failure);
    }

    #[test]
    fn test_exec_state() {
        let state = ExecState::default();
        assert!(state.is_top_level());
        assert!(!state.is_nested());

        let nested = ExecState {
            level: 2,
            ..Default::default()
        };
        assert!(!nested.is_top_level());
        assert!(nested.is_nested());
    }

    #[test]
    fn test_special_arg() {
        assert_eq!(SpecialArg::from_raw(0), Some(SpecialArg::Args));
        assert_eq!(SpecialArg::from_raw(100), None);

        assert!(SpecialArg::Args.needs_context());
        assert!(!SpecialArg::Lt.needs_context());

        assert_eq!(SpecialArg::QArgs.name(), "q-args");
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_mods_is_silent(MOD_SILENT), 1);
        assert_eq!(rs_usercmd_mods_is_silent(0), 0);

        assert_eq!(rs_usercmd_exec_result_is_ok(0), 1);
        assert_eq!(rs_usercmd_exec_result_is_ok(1), 0);
    }

    // =========================================================================
    // Phase 3 — add_cmd_modifier tests
    // =========================================================================

    /// Helper: create a NUL-terminated C buffer of the given size, returning
    /// a pointer and the backing storage.
    fn make_buf(size: usize) -> (Vec<i8>, *mut c_char) {
        let mut v: Vec<i8> = vec![0; size];
        let p = v.as_mut_ptr();
        (v, p)
    }

    /// Read a NUL-terminated C string from a buffer into a Rust `String`.
    #[allow(clippy::cast_sign_loss)]
    fn read_cstr(buf: &[i8]) -> String {
        let bytes: Vec<u8> = buf
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as u8)
            .collect();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn test_add_cmd_modifier_single_measure() {
        // Measure-only pass (null buffer)
        let mut multi = false;
        let len = add_cmd_modifier(std::ptr::null_mut(), b"keepalt", &mut multi);
        assert_eq!(len, 7); // "keepalt".len()
        assert!(multi); // always set to true
    }

    #[test]
    fn test_add_cmd_modifier_single_write() {
        let (storage, buf) = make_buf(64);
        let mut multi = false;
        let len = add_cmd_modifier(buf, b"keepalt", &mut multi);
        assert_eq!(len, 7);
        assert!(multi);
        assert_eq!(read_cstr(&storage), "keepalt");
        // Verify NUL termination
        assert_eq!(storage[7], 0);
    }

    #[test]
    fn test_add_cmd_modifier_multiple_measure() {
        let mut multi = false;
        let len1 = add_cmd_modifier(std::ptr::null_mut(), b"browse", &mut multi);
        assert_eq!(len1, 6);
        assert!(multi);

        let len2 = add_cmd_modifier(std::ptr::null_mut(), b"confirm", &mut multi);
        // Second modifier adds a space prefix
        assert_eq!(len2, 8); // " confirm" = 1 + 7
        assert!(multi);
    }

    #[test]
    fn test_add_cmd_modifier_multiple_write() {
        let (storage, buf) = make_buf(64);
        let mut multi = false;

        let len1 = add_cmd_modifier(buf, b"browse", &mut multi);
        assert_eq!(len1, 6);
        assert_eq!(read_cstr(&storage), "browse");

        let len2 = add_cmd_modifier(buf, b"confirm", &mut multi);
        assert_eq!(len2, 8); // " confirm"
        assert_eq!(read_cstr(&storage), "browse confirm");

        let len3 = add_cmd_modifier(buf, b"hide", &mut multi);
        assert_eq!(len3, 5); // " hide"
        assert_eq!(read_cstr(&storage), "browse confirm hide");
    }

    #[test]
    fn test_add_cmd_modifier_multi_mods_starts_true() {
        // When multi_mods starts as true, even the first modifier gets a
        // leading space.
        let (storage, buf) = make_buf(64);
        let mut multi = true;
        let len = add_cmd_modifier(buf, b"silent", &mut multi);
        assert_eq!(len, 7); // " silent" = 1 + 6
        assert_eq!(read_cstr(&storage), " silent");
    }

    #[test]
    fn test_add_cmd_modifier_null_buf_does_not_crash() {
        let mut multi = true;
        let len = add_cmd_modifier(std::ptr::null_mut(), b"noautocmd", &mut multi);
        assert_eq!(len, 10); // " noautocmd" = 1 + 9
    }

    #[test]
    fn test_add_cmd_modifier_empty_string() {
        let (storage, buf) = make_buf(64);
        let mut multi = false;
        let len = add_cmd_modifier(buf, b"", &mut multi);
        assert_eq!(len, 0);
        assert_eq!(read_cstr(&storage), "");
        assert!(multi); // still set to true
    }

    #[test]
    fn test_add_cmd_modifier_silent_bang() {
        let (storage, buf) = make_buf(64);
        let mut multi = false;
        let len = add_cmd_modifier(buf, b"silent!", &mut multi);
        assert_eq!(len, 7);
        assert_eq!(read_cstr(&storage), "silent!");
    }

    // =========================================================================
    // Phase 3 — format_c_int / format_int_suffix tests
    // =========================================================================

    #[test]
    fn test_format_c_int_positive() {
        let mut buf = [0u8; 20];
        assert_eq!(format_c_int(0, &mut buf), b"0");
        assert_eq!(format_c_int(1, &mut buf), b"1");
        assert_eq!(format_c_int(42, &mut buf), b"42");
        assert_eq!(format_c_int(999, &mut buf), b"999");
    }

    #[test]
    fn test_format_c_int_negative() {
        let mut buf = [0u8; 20];
        assert_eq!(format_c_int(-1, &mut buf), b"-1");
        assert_eq!(format_c_int(-123, &mut buf), b"-123");
    }

    #[test]
    fn test_format_int_suffix_tab() {
        let mut buf = [0u8; 68];
        assert_eq!(format_int_suffix(3, b"tab", &mut buf), b"3tab");
        assert_eq!(format_int_suffix(10, b"tab", &mut buf), b"10tab");
        assert_eq!(format_int_suffix(0, b"tab", &mut buf), b"0tab");
    }

    #[test]
    fn test_format_int_suffix_verbose() {
        let mut buf = [0u8; 65];
        assert_eq!(format_int_suffix(2, b"verbose", &mut buf), b"2verbose");
        assert_eq!(format_int_suffix(99, b"verbose", &mut buf), b"99verbose");
    }

    // =========================================================================
    // Phase 3 — MOD_ENTRIES table tests
    // =========================================================================

    #[test]
    fn test_mod_entries_table_length() {
        // Must match the 12 entries in the C mod_entries[] array
        assert_eq!(MOD_ENTRIES.len(), 12);
    }

    #[test]
    fn test_mod_entries_table_order() {
        // Verify the order matches C exactly
        let expected: &[(c_int, &[u8])] = &[
            (CMOD_BROWSE, b"browse"),
            (CMOD_CONFIRM, b"confirm"),
            (CMOD_HIDE, b"hide"),
            (CMOD_KEEPALT, b"keepalt"),
            (CMOD_KEEPJUMPS, b"keepjumps"),
            (CMOD_KEEPMARKS, b"keepmarks"),
            (CMOD_KEEPPATTERNS, b"keeppatterns"),
            (CMOD_LOCKMARKS, b"lockmarks"),
            (CMOD_NOSWAPFILE, b"noswapfile"),
            (CMOD_UNSILENT, b"unsilent"),
            (CMOD_NOAUTOCMD, b"noautocmd"),
            (CMOD_SANDBOX, b"sandbox"),
        ];
        for (i, &(flag, name)) in expected.iter().enumerate() {
            assert_eq!(MOD_ENTRIES[i].0, flag, "flag mismatch at index {i}");
            assert_eq!(MOD_ENTRIES[i].1, name, "name mismatch at index {i}");
        }
    }

    // =========================================================================
    // Phase 4 — detect_quote_prefix tests
    // =========================================================================

    #[test]
    fn test_detect_quote_prefix_q_dash() {
        let (quote, skip) = detect_quote_prefix(b'q', b'-');
        assert_eq!(quote, 1);
        assert_eq!(skip, 2);
    }

    #[test]
    fn test_detect_quote_prefix_upper_q_dash() {
        let (quote, skip) = detect_quote_prefix(b'Q', b'-');
        assert_eq!(quote, 1);
        assert_eq!(skip, 2);
    }

    #[test]
    fn test_detect_quote_prefix_f_dash() {
        let (quote, skip) = detect_quote_prefix(b'f', b'-');
        assert_eq!(quote, 2);
        assert_eq!(skip, 2);
    }

    #[test]
    fn test_detect_quote_prefix_upper_f_dash() {
        let (quote, skip) = detect_quote_prefix(b'F', b'-');
        assert_eq!(quote, 2);
        assert_eq!(skip, 2);
    }

    #[test]
    fn test_detect_quote_prefix_no_prefix() {
        let (quote, skip) = detect_quote_prefix(b'a', b'-');
        assert_eq!(quote, 0);
        assert_eq!(skip, 0);
    }

    #[test]
    fn test_detect_quote_prefix_q_no_dash() {
        // q without - should not match
        let (quote, skip) = detect_quote_prefix(b'q', b'x');
        assert_eq!(quote, 0);
        assert_eq!(skip, 0);
    }

    // =========================================================================
    // Phase 4 — identify_code_type tests
    // =========================================================================

    #[test]
    fn test_identify_code_type_args() {
        let s = b"args>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Args);
    }

    #[test]
    fn test_identify_code_type_bang() {
        let s = b"bang>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Bang);
    }

    #[test]
    fn test_identify_code_type_count() {
        let s = b"count>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Count);
    }

    #[test]
    fn test_identify_code_type_line1() {
        let s = b"line1>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Line1);
    }

    #[test]
    fn test_identify_code_type_line2() {
        let s = b"line2>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Line2);
    }

    #[test]
    fn test_identify_code_type_range() {
        let s = b"range>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Range);
    }

    #[test]
    fn test_identify_code_type_lt() {
        let s = b"lt>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Lt);
    }

    #[test]
    fn test_identify_code_type_reg() {
        let s = b"reg>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Register);
    }

    #[test]
    fn test_identify_code_type_register() {
        let s = b"register>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Register);
    }

    #[test]
    fn test_identify_code_type_mods() {
        let s = b"mods>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Mods);
    }

    #[test]
    fn test_identify_code_type_case_insensitive() {
        let s = b"ARGS>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Args);

        let s = b"Bang>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Bang);

        let s = b"MODS>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::Mods);
    }

    #[test]
    fn test_identify_code_type_unknown() {
        let s = b"foo>";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::None);
    }

    #[test]
    fn test_identify_code_type_too_short() {
        // l <= 1 should return None
        let s = b"x";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), 1) };
        assert_eq!(ct, CodeType::None);

        let ct = unsafe { identify_code_type(s.as_ptr().cast(), 0) };
        assert_eq!(ct, CodeType::None);
    }

    #[test]
    fn test_identify_code_type_wrong_length() {
        // "args" without ">" should not match "args>" (length mismatch)
        let s = b"args";
        let ct = unsafe { identify_code_type(s.as_ptr().cast(), s.len()) };
        assert_eq!(ct, CodeType::None);
    }

    // =========================================================================
    // Phase 4 — strnicmp_eq_ptr tests
    // =========================================================================

    #[test]
    fn test_strnicmp_eq_ptr_basic() {
        let s = b"args>";
        assert!(unsafe { strnicmp_eq_ptr(s.as_ptr().cast(), b"args>", 5) });
    }

    #[test]
    fn test_strnicmp_eq_ptr_case_insensitive() {
        let s = b"ARGS>";
        assert!(unsafe { strnicmp_eq_ptr(s.as_ptr().cast(), b"args>", 5) });
    }

    #[test]
    fn test_strnicmp_eq_ptr_length_mismatch() {
        let s = b"args>";
        // target is 4 bytes but l is 5 — length check should fail
        assert!(!unsafe { strnicmp_eq_ptr(s.as_ptr().cast(), b"args", 5) });
    }

    #[test]
    fn test_strnicmp_eq_ptr_content_mismatch() {
        let s = b"bang>";
        assert!(!unsafe { strnicmp_eq_ptr(s.as_ptr().cast(), b"args>", 5) });
    }

    // =========================================================================
    // Phase 4 — ascii_iswhite tests
    // =========================================================================

    #[test]
    fn test_ascii_iswhite() {
        assert!(ascii_iswhite(b' '));
        assert!(ascii_iswhite(b'\t'));
        assert!(!ascii_iswhite(b'a'));
        assert!(!ascii_iswhite(b'\n'));
        assert!(!ascii_iswhite(b'\0'));
    }

    // =========================================================================
    // Phase 4 — format_i64 tests
    // =========================================================================

    #[test]
    fn test_format_i64_zero() {
        let mut buf = [0u8; 20];
        assert_eq!(format_i64(0, &mut buf), b"0");
    }

    #[test]
    fn test_format_i64_positive() {
        let mut buf = [0u8; 20];
        assert_eq!(format_i64(1, &mut buf), b"1");
        assert_eq!(format_i64(42, &mut buf), b"42");
        assert_eq!(format_i64(12345, &mut buf), b"12345");
    }

    #[test]
    fn test_format_i64_negative() {
        let mut buf = [0u8; 20];
        assert_eq!(format_i64(-1, &mut buf), b"-1");
        assert_eq!(format_i64(-999, &mut buf), b"-999");
    }

    #[test]
    fn test_format_i64_large() {
        let mut buf = [0u8; 20];
        assert_eq!(format_i64(1_000_000_000, &mut buf), b"1000000000");
        assert_eq!(format_i64(-1_000_000_000, &mut buf), b"-1000000000");
    }

    // =========================================================================
    // Phase 7 — do_ucmd constant tests
    // =========================================================================

    #[test]
    fn test_cmd_user_constant() {
        assert_eq!(CMD_USER, -1);
    }

    #[test]
    fn test_k_special_sequence() {
        // Verify the K_SPECIAL encoding constants match the expected values
        // for the K_SPECIAL → KS_SPECIAL KE_FILLER conversion in do_ucmd
        assert_eq!(K_SPECIAL, 0x80);
        assert_eq!(KS_SPECIAL, 254);
        assert_eq!(KE_FILLER, b'X');
        // The 3-byte sequence is: K_SPECIAL(0x80) KS_SPECIAL(0xFE) KE_FILLER(0x58)
        assert_eq!([K_SPECIAL, KS_SPECIAL, KE_FILLER], [0x80, 0xFE, 0x58]);
    }

    #[test]
    fn test_ex_keepscript_value() {
        // Verify EX_KEEPSCRIPT constant from define.rs
        assert_eq!(crate::define::EX_KEEPSCRIPT, 0x400_0000);
    }
}
