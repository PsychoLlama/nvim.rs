//! Option value expansion and completions
//!
//! This module provides Rust implementations for option value
//! expansion including environment variables, special characters,
//! and completion candidates.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

use crate::{OptFlags, OptValType};

// =============================================================================
// Expansion Types
// =============================================================================

/// Type of expansion to perform.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExpandType {
    /// No expansion
    #[default]
    None = 0,
    /// Expand environment variables
    Env = 1,
    /// Expand ~ to home directory
    Home = 2,
    /// Expand file names
    File = 3,
    /// Expand directory names
    Dir = 4,
    /// Expand option values
    OptVal = 5,
    /// Expand buffer names
    Buffer = 6,
    /// Expand tag names
    Tag = 7,
}

impl ExpandType {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Env,
            2 => Self::Home,
            3 => Self::File,
            4 => Self::Dir,
            5 => Self::OptVal,
            6 => Self::Buffer,
            7 => Self::Tag,
            _ => Self::None,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this type expands file system entries.
    #[must_use]
    pub const fn is_filesystem(self) -> bool {
        matches!(self, Self::File | Self::Dir)
    }
}

// =============================================================================
// Expansion Context
// =============================================================================

/// Context for expansion operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExpandContext {
    /// Type of expansion
    pub expand_type: c_int,
    /// Starting column of text to expand
    pub start_col: c_int,
    /// Ending column
    pub end_col: c_int,
    /// Whether expansion is at command start
    pub at_start: bool,
    /// Whether to show all matches
    pub show_all: bool,
    /// Option flags affecting expansion
    pub opt_flags: u32,
}

impl ExpandContext {
    /// Create a new expansion context.
    #[must_use]
    pub const fn new(expand_type: ExpandType) -> Self {
        Self {
            expand_type: expand_type.to_c_int(),
            start_col: 0,
            end_col: 0,
            at_start: false,
            show_all: false,
            opt_flags: 0,
        }
    }

    /// Get expansion type.
    #[must_use]
    pub const fn get_type(&self) -> ExpandType {
        ExpandType::from_c_int(self.expand_type)
    }

    /// Check if environment expansion is needed.
    #[must_use]
    pub const fn needs_env(&self) -> bool {
        self.opt_flags & OptFlags::EXPAND.0 != 0
    }
}

// =============================================================================
// Environment Variable Expansion
// =============================================================================

/// Check if position is at start of an environment variable.
/// Looks for $NAME or ${NAME} patterns.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_at_env_var(str: *const c_char, pos: c_int) -> c_int {
    if str.is_null() || pos < 0 {
        return 0;
    }

    let p = str.add(pos as usize);
    c_int::from(*p as u8 == b'$')
}

/// Find start of environment variable at position.
/// Scans backward to find the $ character.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_env_start(str: *const c_char, pos: c_int) -> c_int {
    if str.is_null() || pos <= 0 {
        return -1;
    }

    let mut i = pos - 1;
    while i >= 0 {
        let c = *str.add(i as usize) as u8;
        if c == b'$' {
            return i;
        }
        // Stop at path separators or whitespace
        if c == b'/' || c == b'\\' || c == b' ' || c == b'\t' {
            break;
        }
        i -= 1;
    }

    -1
}

/// Check if character is valid in environment variable name.
#[no_mangle]
pub extern "C" fn rs_is_env_char(c: c_int) -> c_int {
    let ch = c as u8;
    c_int::from(ch.is_ascii_alphanumeric() || ch == b'_')
}

/// Extract environment variable name from string.
/// Returns length of the variable name (excluding $).
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_env_name_len(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let mut p = str;

    // Skip $
    if *p as u8 == b'$' {
        p = p.add(1);
    }

    // Handle ${NAME} form
    let brace = *p as u8 == b'{';
    if brace {
        p = p.add(1);
    }

    let start = p;

    // Count valid env chars
    while *p != 0 {
        let c = *p as u8;
        if brace {
            if c == b'}' {
                break;
            }
        } else if !c.is_ascii_alphanumeric() && c != b'_' {
            break;
        }
        p = p.add(1);
    }

    (p as usize - start as usize) as c_int
}

// =============================================================================
// Home Directory Expansion
// =============================================================================

/// Check if string starts with ~ for home expansion.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_needs_home_expand(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }
    c_int::from(*str as u8 == b'~')
}

/// Get length of ~user prefix.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_tilde_user_len(str: *const c_char) -> c_int {
    if str.is_null() || *str as u8 != b'~' {
        return 0;
    }

    let mut p = str.add(1);
    let start = p;

    // Count username characters
    while *p != 0 {
        let c = *p as u8;
        if c == b'/' || c == b'\\' || c == b' ' || c == b'\t' {
            break;
        }
        p = p.add(1);
    }

    // Add 1 for the ~
    1 + (p as usize - start as usize) as c_int
}

// =============================================================================
// Path Expansion
// =============================================================================

/// Check if character is a path separator.
#[no_mangle]
pub extern "C" fn rs_opt_is_path_sep(c: c_int) -> c_int {
    let ch = c as u8;
    c_int::from(ch == b'/' || ch == b'\\')
}

/// Find last path separator in string.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_last_path_sep(str: *const c_char) -> c_int {
    if str.is_null() {
        return -1;
    }

    let mut last: c_int = -1;
    let mut i: c_int = 0;
    let mut p = str;

    while *p != 0 {
        let c = *p as u8;
        if c == b'/' || c == b'\\' {
            last = i;
        }
        p = p.add(1);
        i += 1;
    }

    last
}

/// Check if path is absolute.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_opt_is_absolute_path(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let c = *str as u8;

    // Unix absolute path
    if c == b'/' {
        return 1;
    }

    // Windows absolute path (C:\...)
    if c.is_ascii_alphabetic() && *str.add(1) as u8 == b':' {
        return 1;
    }

    0
}

// =============================================================================
// Comma-Separated Values
// =============================================================================

/// Count items in a comma-separated list.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_count_csv_items(str: *const c_char) -> c_int {
    if str.is_null() || *str == 0 {
        return 0;
    }

    let mut count: c_int = 1;
    let mut p = str;
    let mut in_escape = false;

    while *p != 0 {
        let c = *p as u8;
        if in_escape {
            in_escape = false;
        } else if c == b'\\' {
            in_escape = true;
        } else if c == b',' {
            count += 1;
        }
        p = p.add(1);
    }

    count
}

/// Find start of item at given index in CSV.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_csv_item(str: *const c_char, index: c_int) -> c_int {
    if str.is_null() || index < 0 {
        return -1;
    }

    let mut current: c_int = 0;
    let mut pos: c_int = 0;
    let mut p = str;
    let mut in_escape = false;

    while *p != 0 {
        if current == index {
            return pos;
        }

        let c = *p as u8;
        if in_escape {
            in_escape = false;
        } else if c == b'\\' {
            in_escape = true;
        } else if c == b',' {
            current += 1;
        }

        p = p.add(1);
        pos += 1;
    }

    if current == index {
        pos
    } else {
        -1
    }
}

/// Get length of CSV item starting at position.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_csv_item_len(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let mut len: c_int = 0;
    let mut p = str;
    let mut in_escape = false;

    while *p != 0 {
        let c = *p as u8;
        if in_escape {
            in_escape = false;
        } else if c == b'\\' {
            in_escape = true;
        } else if c == b',' {
            break;
        }

        p = p.add(1);
        len += 1;
    }

    len
}

// =============================================================================
// Value Completion
// =============================================================================

/// Option value completion info.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueCompInfo {
    /// Completion start position
    pub start: c_int,
    /// Completion end position
    pub end: c_int,
    /// Type of completion
    pub comp_type: c_int,
    /// Whether in quoted string
    pub in_quote: bool,
}

/// Determine completion info for option value.
///
/// # Safety
/// `value` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_value_comp_info(
    value: *const c_char,
    pos: c_int,
    opt_type: c_int,
) -> ValueCompInfo {
    if value.is_null() || pos < 0 {
        return ValueCompInfo::default();
    }

    let mut info = ValueCompInfo {
        start: 0,
        end: pos,
        comp_type: 0,
        in_quote: false,
    };

    // For string options, find word boundaries
    if opt_type == OptValType::String as c_int {
        let mut i = pos;

        // Scan backward for word start
        while i > 0 {
            let c = *value.add((i - 1) as usize) as u8;
            if c == b',' || c == b' ' || c == b'\t' {
                break;
            }
            i -= 1;
        }
        info.start = i;

        // Check for env var - scan from info.start to pos
        let mut scan_pos = info.start;
        while scan_pos < pos {
            if *value.add(scan_pos as usize) as u8 == b'$' {
                info.comp_type = ExpandType::Env as c_int;
                info.start = scan_pos;
                break;
            }
            scan_pos += 1;
        }

        // Check for path
        if info.comp_type == 0 {
            let first = *value.add(info.start as usize) as u8;
            if first == b'~' || first == b'/' || first == b'.' {
                info.comp_type = ExpandType::File as c_int;
            }
        }
    }

    info
}

// =============================================================================
// Expand Context FFI
// =============================================================================

/// FFI: Create expansion context.
#[no_mangle]
pub extern "C" fn rs_expand_context_new(expand_type: c_int) -> ExpandContext {
    ExpandContext::new(ExpandType::from_c_int(expand_type))
}

/// FFI: Check if context needs env expansion.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_needs_env(ctx: *const ExpandContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).needs_env())
}

/// FFI: Check if expand type is filesystem.
#[no_mangle]
pub extern "C" fn rs_expand_is_filesystem(expand_type: c_int) -> c_int {
    c_int::from(ExpandType::from_c_int(expand_type).is_filesystem())
}

// =============================================================================
// ExpandSettingSubtract migration
// =============================================================================

/// Growing array structure matching C `garray_T`.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GArray {
    const fn new() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 1,
            ga_data: std::ptr::null_mut(),
        }
    }
}

use crate::index::OptIndex;
use crate::opt_index::K_OPT_INVALID;
use crate::{BufHandle, WinHandle, FAIL, OK};

// =============================================================================
// expand_option_* static state (moved from C option_shim.c)
// These are the expand_option_idx, expand_option_name, expand_option_flags
// statics from option_shim.c. expand_option_start_col and expand_option_append
// remain in C because nvim_option_invoke_expand_cb reads them directly.
// =============================================================================

/// Current option index being expanded (kOptInvalid for terminal options).
static mut EXPAND_OPTION_IDX: OptIndex = K_OPT_INVALID;
/// Terminal option name buffer: "t_XX\0" (index 2+3 hold the key chars).
static mut EXPAND_OPTION_NAME: [c_char; 5] = [b't' as c_char, b'_' as c_char, 0, 0, 0];
/// Option flags for the option being expanded.
static mut EXPAND_OPTION_FLAGS: c_int = 0;

pub(crate) unsafe fn get_expand_option_idx() -> OptIndex {
    EXPAND_OPTION_IDX
}
pub(crate) unsafe fn set_expand_option_idx(val: OptIndex) {
    EXPAND_OPTION_IDX = val;
}
pub(crate) unsafe fn get_expand_option_flags() -> c_int {
    EXPAND_OPTION_FLAGS
}
pub(crate) unsafe fn set_expand_option_flags(val: c_int) {
    EXPAND_OPTION_FLAGS = val;
}
pub(crate) unsafe fn get_expand_option_name_ptr() -> *const c_char {
    std::ptr::addr_of!(EXPAND_OPTION_NAME).cast::<c_char>()
}
pub(crate) unsafe fn set_expand_option_name_chars(c2: c_char, c3: c_char) {
    EXPAND_OPTION_NAME[2] = c2;
    EXPAND_OPTION_NAME[3] = c3;
}

extern "C" {
    // option varp scope accessor
    fn get_option_varp_scope_from(
        opt_idx: OptIndex,
        opt_flags: c_int,
        buf: BufHandle,
        win: WinHandle,
    ) -> *mut c_void;

    // option type checking
    fn option_has_type(opt_idx: OptIndex, val_type: c_int) -> c_int;

    // option flags
    #[link_name = "rs_get_option_flags"]
    fn get_option_flags(opt_idx: OptIndex) -> u32;

    // xp_pattern accessor
    fn nvim_xp_get_pattern(xp: *mut c_void) -> *mut c_char;

    // vim_strchr: search for character in string
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // vim_regexec: single-line regex match (regmatch_T* passed as *mut c_void)
    fn vim_regexec(rmp: *mut c_void, line: *const c_char, col: c_int) -> bool;

    // curbuf/curwin globals
    static mut curbuf: BufHandle;
    static mut curwin: WinHandle;

    // garray operations
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GArray, n: c_int);

    // xfree for cleanup (use *mut c_char to match other declarations in this crate)
    fn xfree(ptr: *mut c_char);

    // xstrdup, xmemdupz, xmalloc for allocation
    fn xstrdup(str_: *const c_char) -> *mut c_char;
    fn xmemdupz(data: *const c_char, len: usize) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_char;
}

/// kOptValTypeNumber value (must match C kOptValTypeNumber = 1)
const K_OPT_VAL_TYPE_NUMBER: c_int = 1;

/// Rust port of `ExpandSettingSubtract`.
///
/// Expansion handler for `:set-=`. Splits comma-separated or flag-list option
/// values and returns matching completions filtered by `regmatch`.
///
/// # Safety
/// All pointer arguments must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_setting_subtract(
    xp: *mut c_void,
    regmatch: *mut c_void, // regmatch_T* passed as opaque pointer
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let expand_option_idx = get_expand_option_idx();

    if expand_option_idx == K_OPT_INVALID {
        // Terminal option: delegate to rs_expand_old_setting.
        return rs_expand_old_setting(num_matches, matches);
    }

    let expand_option_flags = get_expand_option_flags();

    let option_val_ptr =
        get_option_varp_scope_from(expand_option_idx, expand_option_flags, curbuf, curwin);
    let option_val = *(option_val_ptr.cast::<*mut c_char>());

    let option_flags = OptFlags(get_option_flags(expand_option_idx));

    if option_has_type(expand_option_idx, K_OPT_VAL_TYPE_NUMBER) != 0 {
        return rs_expand_old_setting(num_matches, matches);
    } else if option_flags.contains(OptFlags::COMMA) {
        // Split by comma (respecting "\," escapes), filter by regexec, escape each.
        // kOptFlagComma check goes first because 'whichwrap' has both COMMA and FLAG_LIST.

        if *option_val == 0 {
            return FAIL;
        }

        // Make a copy as we need to null-terminate items destructively.
        let option_copy = xstrdup(option_val);
        let mut next_val = option_copy;

        let mut ga = GArray::new();
        ga_init(
            std::ptr::addr_of_mut!(ga),
            c_int::try_from(std::mem::size_of::<*mut c_char>()).unwrap_or(8),
            10,
        );

        loop {
            let item = next_val;

            // Find next comma (skipping escaped commas "\,")
            let mut comma = vim_strchr(next_val, c_int::from(b','));
            while !comma.is_null() && comma != next_val && *comma.offset(-1) == b'\\' as c_char {
                comma = vim_strchr(comma.add(1), c_int::from(b','));
            }

            if comma.is_null() {
                next_val = std::ptr::null_mut();
            } else {
                // Null-terminate this item.
                *comma = 0;
                next_val = comma.add(1);
            }

            if *item == 0 {
                // Empty value, skip.
                if next_val.is_null() {
                    break;
                }
                continue;
            }

            if !vim_regexec(regmatch, item, 0) {
                if next_val.is_null() {
                    break;
                }
                continue;
            }

            let escaped = rs_escape_option_str_cmdline(item);
            // GA_APPEND: grow by 1 and store pointer.
            ga_grow(std::ptr::addr_of_mut!(ga), 1);
            #[allow(clippy::cast_ptr_alignment)]
            let slot = ga.ga_data.cast::<*mut c_char>().add(ga.ga_len as usize);
            *slot = escaped;
            ga.ga_len += 1;

            if next_val.is_null() {
                break;
            }
        }

        xfree(option_copy);

        #[allow(clippy::cast_ptr_alignment)]
        {
            *matches = ga.ga_data.cast::<*mut c_char>();
        }
        *num_matches = ga.ga_len;
        return OK;
    } else if option_flags.contains(OptFlags::FLAG_LIST) {
        // Flag-list: expose individual flags (and full value) as choices.
        // Only when xp_pattern is empty.

        let xp_pattern = nvim_xp_get_pattern(xp);
        if *xp_pattern != 0 {
            // Non-empty pattern: don't suggest anything.
            return FAIL;
        }

        let mut len = 0usize;
        let mut p = option_val;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }

        if len == 0 {
            return FAIL;
        }

        // Allocate: 1 for full value + len individual flags (+ 1 for safety)
        let arr_size = if len > 1 { len + 1 } else { 1 };
        #[allow(clippy::cast_ptr_alignment)]
        let arr = xmalloc(arr_size * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();

        let mut count: usize = 0;

        // First entry: the whole current value
        *arr.add(count) = xmemdupz(option_val, len);
        count += 1;

        if len > 1 {
            // Individual flag characters
            let mut flag = option_val;
            while *flag != 0 {
                *arr.add(count) = xmemdupz(flag, 1);
                count += 1;
                flag = flag.add(1);
            }
        }

        *matches = arr;
        *num_matches = count as c_int;
        return OK;
    }

    rs_expand_old_setting(num_matches, matches)
}

// =============================================================================
// Phase 3: escape_option_str_cmdline, ExpandOldSetting, ExpandStringSetting
// =============================================================================

extern "C" {
    fn vim_strsave_escaped(s: *const c_char, esc: *const c_char) -> *mut c_char;
    static escape_chars: *const c_char;
    fn find_option(name: *const c_char) -> OptIndex;
    static mut NameBuff: [c_char; 4096];
    fn nvim_option_has_expand_cb(opt_idx: OptIndex) -> c_int;
    fn nvim_option_invoke_expand_cb(
        opt_idx: OptIndex,
        opt_flags: c_int,
        xp: *mut c_void,
        regmatch: *mut c_void,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
}

// Call rs_option_value2string (same library, #[no_mangle] in session.rs)
extern "C" {
    fn rs_option_value2string(opt_idx: c_int, opt_flags: c_int);
}

/// Rust port of `escape_option_str_cmdline`.
///
/// Escapes option value for command-line display by prepending backslashes
/// before special characters. Caller must free the returned string.
///
/// # Safety
/// `var` must be a valid null-terminated C string.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_escape_option_str_cmdline(var: *const c_char) -> *mut c_char {
    // Call vim_strsave_escaped(var, escape_chars)
    vim_strsave_escaped(var, escape_chars)
    // Note: BACKSLASH_IN_FILENAME path is Windows-only, skipped on Linux.
}

/// Rust port of `ExpandOldSetting`.
///
/// Expansion handler for `:set=` when filling in with the existing value.
///
/// # Safety
/// All pointer arguments must be valid.
#[allow(clippy::must_use_candidate)]
#[export_name = "ExpandOldSetting"]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_expand_old_setting(
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    *num_matches = 0;
    #[allow(clippy::cast_ptr_alignment)]
    let matches_arr = xmalloc(std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    *matches = matches_arr;

    let mut expand_option_idx = get_expand_option_idx();
    let expand_option_flags = get_expand_option_flags();

    // For a terminal key code expand_option_idx is kOptInvalid.
    if expand_option_idx == K_OPT_INVALID {
        let name = get_expand_option_name_ptr();
        expand_option_idx = find_option(name);
        set_expand_option_idx(expand_option_idx);
    }

    let var: *const c_char = if expand_option_idx == K_OPT_INVALID {
        c"".as_ptr()
    } else {
        // Put string of option value in NameBuff.
        rs_option_value2string(expand_option_idx as c_int, expand_option_flags);
        (&raw const NameBuff).cast::<c_char>()
    };

    let buf = rs_escape_option_str_cmdline(var);
    *(*matches) = buf;
    *num_matches = 1;
    OK
}

/// Rust port of `ExpandStringSetting`.
///
/// Expansion handler for `:set=`/`:set+=` when the option has a custom callback.
///
/// # Safety
/// All pointer arguments must be valid.
#[allow(clippy::must_use_candidate)]
#[export_name = "ExpandStringSetting"]
pub unsafe extern "C" fn rs_expand_string_setting(
    xp: *mut c_void,
    regmatch: *mut c_void,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let expand_option_idx = get_expand_option_idx();
    let expand_option_flags = get_expand_option_flags();

    if nvim_option_has_expand_cb(expand_option_idx) == 0 {
        // Not supposed to reach here.
        return FAIL;
    }

    nvim_option_invoke_expand_cb(
        expand_option_idx,
        expand_option_flags,
        xp,
        regmatch,
        num_matches,
        matches,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_type() {
        assert_eq!(ExpandType::from_c_int(0), ExpandType::None);
        assert_eq!(ExpandType::from_c_int(1), ExpandType::Env);
        assert!(ExpandType::File.is_filesystem());
        assert!(ExpandType::Dir.is_filesystem());
        assert!(!ExpandType::Env.is_filesystem());
    }

    #[test]
    fn test_expand_context() {
        let ctx = ExpandContext::new(ExpandType::Env);
        assert_eq!(ctx.get_type(), ExpandType::Env);
    }

    #[test]
    fn test_at_env_var() {
        unsafe {
            assert_eq!(rs_at_env_var(c"$HOME".as_ptr(), 0), 1);
            assert_eq!(rs_at_env_var(c"/path/$VAR".as_ptr(), 0), 0);
            assert_eq!(rs_at_env_var(c"/path/$VAR".as_ptr(), 6), 1);
        }
    }

    #[test]
    fn test_env_name_len() {
        unsafe {
            assert_eq!(rs_env_name_len(c"$HOME/path".as_ptr()), 4);
            assert_eq!(rs_env_name_len(c"${HOME}/path".as_ptr()), 4);
            assert_eq!(rs_env_name_len(c"$VAR_NAME".as_ptr()), 8);
        }
    }

    #[test]
    fn test_needs_home_expand() {
        unsafe {
            assert_eq!(rs_needs_home_expand(c"~/.config".as_ptr()), 1);
            assert_eq!(rs_needs_home_expand(c"/home/user".as_ptr()), 0);
        }
    }

    #[test]
    fn test_tilde_user_len() {
        unsafe {
            assert_eq!(rs_tilde_user_len(c"~/".as_ptr()), 1);
            assert_eq!(rs_tilde_user_len(c"~user/".as_ptr()), 5);
            assert_eq!(rs_tilde_user_len(c"/path".as_ptr()), 0);
        }
    }

    #[test]
    fn test_is_path_sep() {
        assert_eq!(rs_opt_is_path_sep(c_int::from(b'/')), 1);
        assert_eq!(rs_opt_is_path_sep(c_int::from(b'\\')), 1);
        assert_eq!(rs_opt_is_path_sep(c_int::from(b'a')), 0);
    }

    #[test]
    fn test_find_last_path_sep() {
        unsafe {
            assert_eq!(rs_find_last_path_sep(c"/a/b/c".as_ptr()), 4);
            assert_eq!(rs_find_last_path_sep(c"nopath".as_ptr()), -1);
        }
    }

    #[test]
    fn test_is_absolute_path() {
        unsafe {
            assert_eq!(rs_opt_is_absolute_path(c"/home".as_ptr()), 1);
            assert_eq!(rs_opt_is_absolute_path(c"relative".as_ptr()), 0);
        }
    }

    #[test]
    fn test_count_csv_items() {
        unsafe {
            assert_eq!(rs_count_csv_items(c"".as_ptr()), 0);
            assert_eq!(rs_count_csv_items(c"one".as_ptr()), 1);
            assert_eq!(rs_count_csv_items(c"one,two,three".as_ptr()), 3);
            assert_eq!(rs_count_csv_items(c"a\\,b,c".as_ptr()), 2); // Escaped comma
        }
    }

    #[test]
    fn test_find_csv_item() {
        unsafe {
            let s = c"one,two,three".as_ptr();
            assert_eq!(rs_find_csv_item(s, 0), 0);
            assert_eq!(rs_find_csv_item(s, 1), 4);
            assert_eq!(rs_find_csv_item(s, 2), 8);
        }
    }

    #[test]
    fn test_csv_item_len() {
        unsafe {
            assert_eq!(rs_csv_item_len(c"one,two".as_ptr()), 3);
            assert_eq!(rs_csv_item_len(c"a\\,b,c".as_ptr()), 4); // "a\,b"
        }
    }
}
