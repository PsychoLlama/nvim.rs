//! Buffer Ex commands (:buffer, :bdelete, :bunload, :bwipeout, :buffers)
//!
//! This module implements Ex commands for buffer management and navigation.

use std::ffi::c_int;

// =============================================================================
// Buffer Action Types
// =============================================================================

/// Action to perform on a buffer.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferAction {
    /// Switch to buffer (:buffer, :sbuffer)
    #[default]
    Switch = 0,
    /// Unload buffer (:bunload)
    Unload = 1,
    /// Delete buffer (:bdelete)
    Delete = 2,
    /// Wipe buffer (:bwipeout)
    Wipe = 3,
}

impl BufferAction {
    /// Create from raw integer (matching DOBUF_* constants).
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Unload,
            2 => Self::Delete,
            3 => Self::Wipe,
            _ => Self::Switch,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this action removes the buffer from the list.
    #[must_use]
    pub const fn removes_from_list(self) -> bool {
        matches!(self, Self::Delete | Self::Wipe)
    }

    /// Check if this action is destructive (wipe).
    #[must_use]
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::Wipe)
    }
}

// =============================================================================
// Buffer List Flags
// =============================================================================

/// Flags for :buffers command display.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferListFlags {
    /// Show unlisted buffers
    pub unlisted: bool,
    /// Show buffer numbers
    pub numbers: bool,
    /// Show modified indicator
    pub modified: bool,
    /// Show full path
    pub full_path: bool,
}

impl BufferListFlags {
    /// Create from argument string.
    #[must_use]
    pub fn from_args(args: &[u8]) -> Self {
        let mut flags = Self::default();

        for &c in args {
            match c {
                b'u' => flags.unlisted = true,
                b'!' => flags.unlisted = true,
                b'+' => flags.modified = true,
                _ => {}
            }
        }

        flags
    }
}

// =============================================================================
// Buffer Navigation
// =============================================================================

/// Direction for buffer navigation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferNavDirection {
    /// Next buffer (:bnext)
    #[default]
    Next = 0,
    /// Previous buffer (:bprevious)
    Previous = 1,
    /// First buffer (:bfirst, :brewind)
    First = 2,
    /// Last buffer (:blast)
    Last = 3,
}

impl BufferNavDirection {
    /// Create from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Previous,
            2 => Self::First,
            3 => Self::Last,
            _ => Self::Next,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// Calculate the next buffer number for navigation.
///
/// # Arguments
/// * `current` - Current buffer number
/// * `count` - Count (how many to skip)
/// * `max_bufnr` - Maximum buffer number
/// * `direction` - Navigation direction
///
/// # Returns
/// Next buffer number to try (may not exist)
#[must_use]
pub const fn calc_next_bufnr(
    current: c_int,
    count: c_int,
    max_bufnr: c_int,
    direction: BufferNavDirection,
) -> c_int {
    match direction {
        BufferNavDirection::Next => {
            let next = current + count;
            if next > max_bufnr {
                1
            } else {
                next
            }
        }
        BufferNavDirection::Previous => {
            let prev = current - count;
            if prev < 1 {
                max_bufnr
            } else {
                prev
            }
        }
        BufferNavDirection::First => 1,
        BufferNavDirection::Last => max_bufnr,
    }
}

// =============================================================================
// Buffer Argument Parsing
// =============================================================================

/// Result of parsing buffer specification.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferSpec {
    /// Buffer number (0 if not specified by number)
    pub bufnr: c_int,
    /// Whether a number was specified
    pub has_number: bool,
    /// Whether to use alternate buffer (#)
    pub alternate: bool,
    /// Count for navigation commands
    pub count: c_int,
}

/// Check if a buffer argument is a number.
#[must_use]
pub fn is_numeric_bufarg(arg: &[u8]) -> bool {
    if arg.is_empty() {
        return false;
    }

    for &c in arg {
        if !c.is_ascii_digit() {
            return false;
        }
    }
    true
}

/// Parse a numeric buffer argument.
#[must_use]
pub fn parse_bufnr(arg: &[u8]) -> c_int {
    if arg.is_empty() {
        return 0;
    }

    let mut result: c_int = 0;
    for &c in arg {
        if c.is_ascii_digit() {
            result = result
                .saturating_mul(10)
                .saturating_add((c - b'0') as c_int);
        } else {
            break;
        }
    }
    result
}

// =============================================================================
// Buffer Modification Check
// =============================================================================

/// Check if buffer action should be blocked due to modifications.
///
/// # Arguments
/// * `action` - Action being attempted
/// * `force` - Whether ! was used
/// * `modified` - Whether buffer is modified
///
/// # Returns
/// true if action should be blocked
#[must_use]
pub const fn should_block_modified(action: BufferAction, force: bool, modified: bool) -> bool {
    if force {
        return false;
    }

    modified && action.removes_from_list()
}

/// Check if buffer can be unloaded.
///
/// # Arguments
/// * `modified` - Whether buffer is modified
/// * `force` - Whether ! was used
/// * `hidden` - Whether 'hidden' option is set
///
/// # Returns
/// true if buffer can be unloaded
#[must_use]
pub const fn can_unload_buffer(modified: bool, force: bool, hidden: bool) -> bool {
    !modified || force || hidden
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get buffer action from raw value.
#[no_mangle]
pub extern "C" fn rs_buffer_action_from_raw(value: c_int) -> c_int {
    BufferAction::from_raw(value).to_raw()
}

/// FFI: Check if action removes from list.
#[no_mangle]
pub extern "C" fn rs_buffer_action_removes_from_list(action: c_int) -> c_int {
    c_int::from(BufferAction::from_raw(action).removes_from_list())
}

/// FFI: Check if action is destructive.
#[no_mangle]
pub extern "C" fn rs_buffer_action_is_destructive(action: c_int) -> c_int {
    c_int::from(BufferAction::from_raw(action).is_destructive())
}

/// FFI: Get buffer navigation direction from raw value.
#[no_mangle]
pub extern "C" fn rs_buffer_nav_from_raw(value: c_int) -> c_int {
    BufferNavDirection::from_raw(value).to_raw()
}

/// FFI: Calculate next buffer number.
#[no_mangle]
pub extern "C" fn rs_calc_next_bufnr(
    current: c_int,
    count: c_int,
    max_bufnr: c_int,
    direction: c_int,
) -> c_int {
    calc_next_bufnr(
        current,
        count,
        max_bufnr,
        BufferNavDirection::from_raw(direction),
    )
}

/// FFI: Check if should block modified buffer.
#[no_mangle]
pub extern "C" fn rs_should_block_modified(action: c_int, force: c_int, modified: c_int) -> c_int {
    c_int::from(should_block_modified(
        BufferAction::from_raw(action),
        force != 0,
        modified != 0,
    ))
}

/// FFI: Check if buffer can be unloaded.
#[no_mangle]
pub extern "C" fn rs_can_unload_buffer(modified: c_int, force: c_int, hidden: c_int) -> c_int {
    c_int::from(can_unload_buffer(modified != 0, force != 0, hidden != 0))
}

/// FFI: Parse buffer number from string.
///
/// # Safety
/// `arg` must be a valid null-terminated string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_bufnr(arg: *const u8, len: c_int) -> c_int {
    if arg.is_null() || len <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(arg, len as usize);
    parse_bufnr(slice)
}

/// FFI: Check if buffer argument is numeric.
///
/// # Safety
/// `arg` must be a valid null-terminated string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_is_numeric_bufarg(arg: *const u8, len: c_int) -> c_int {
    if arg.is_null() || len <= 0 {
        return 0;
    }

    let slice = std::slice::from_raw_parts(arg, len as usize);
    c_int::from(is_numeric_bufarg(slice))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_action() {
        assert!(!BufferAction::Switch.removes_from_list());
        assert!(!BufferAction::Unload.removes_from_list());
        assert!(BufferAction::Delete.removes_from_list());
        assert!(BufferAction::Wipe.removes_from_list());

        assert!(!BufferAction::Delete.is_destructive());
        assert!(BufferAction::Wipe.is_destructive());
    }

    #[test]
    fn test_buffer_action_roundtrip() {
        for i in 0..4 {
            let action = BufferAction::from_raw(i);
            assert_eq!(action.to_raw(), i);
        }
    }

    #[test]
    fn test_buffer_nav_direction() {
        assert_eq!(BufferNavDirection::from_raw(0), BufferNavDirection::Next);
        assert_eq!(
            BufferNavDirection::from_raw(1),
            BufferNavDirection::Previous
        );
        assert_eq!(BufferNavDirection::from_raw(2), BufferNavDirection::First);
        assert_eq!(BufferNavDirection::from_raw(3), BufferNavDirection::Last);
    }

    #[test]
    fn test_calc_next_bufnr() {
        // Next
        assert_eq!(calc_next_bufnr(5, 1, 10, BufferNavDirection::Next), 6);
        assert_eq!(calc_next_bufnr(10, 1, 10, BufferNavDirection::Next), 1); // Wrap

        // Previous
        assert_eq!(calc_next_bufnr(5, 1, 10, BufferNavDirection::Previous), 4);
        assert_eq!(calc_next_bufnr(1, 1, 10, BufferNavDirection::Previous), 10); // Wrap

        // First/Last
        assert_eq!(calc_next_bufnr(5, 1, 10, BufferNavDirection::First), 1);
        assert_eq!(calc_next_bufnr(5, 1, 10, BufferNavDirection::Last), 10);
    }

    #[test]
    fn test_buffer_list_flags() {
        let flags = BufferListFlags::from_args(b"");
        assert!(!flags.unlisted);

        let flags = BufferListFlags::from_args(b"u");
        assert!(flags.unlisted);

        let flags = BufferListFlags::from_args(b"!");
        assert!(flags.unlisted);

        let flags = BufferListFlags::from_args(b"+");
        assert!(flags.modified);
    }

    #[test]
    fn test_is_numeric_bufarg() {
        assert!(is_numeric_bufarg(b"123"));
        assert!(is_numeric_bufarg(b"1"));
        assert!(!is_numeric_bufarg(b"abc"));
        assert!(!is_numeric_bufarg(b"12a"));
        assert!(!is_numeric_bufarg(b""));
    }

    #[test]
    fn test_parse_bufnr() {
        assert_eq!(parse_bufnr(b"123"), 123);
        assert_eq!(parse_bufnr(b"1"), 1);
        assert_eq!(parse_bufnr(b"0"), 0);
        assert_eq!(parse_bufnr(b""), 0);
        assert_eq!(parse_bufnr(b"12abc"), 12);
    }

    #[test]
    fn test_should_block_modified() {
        // Modified buffer, delete without force - blocked
        assert!(should_block_modified(BufferAction::Delete, false, true));

        // Modified buffer, delete with force - allowed
        assert!(!should_block_modified(BufferAction::Delete, true, true));

        // Unmodified buffer, delete - allowed
        assert!(!should_block_modified(BufferAction::Delete, false, false));

        // Switch never blocks
        assert!(!should_block_modified(BufferAction::Switch, false, true));
        assert!(!should_block_modified(BufferAction::Unload, false, true));
    }

    #[test]
    fn test_can_unload_buffer() {
        // Unmodified - always can unload
        assert!(can_unload_buffer(false, false, false));

        // Modified, no force, no hidden - cannot
        assert!(!can_unload_buffer(true, false, false));

        // Modified with force - can
        assert!(can_unload_buffer(true, true, false));

        // Modified with hidden - can
        assert!(can_unload_buffer(true, false, true));
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_buffer_action_removes_from_list(2), 1);
        assert_eq!(rs_buffer_action_removes_from_list(0), 0);
        assert_eq!(rs_calc_next_bufnr(5, 1, 10, 0), 6);
    }
}
