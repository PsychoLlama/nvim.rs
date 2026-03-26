//! Autocommand group management helpers
//!
//! This module provides helpers for working with autocommand groups,
//! including group creation, deletion, and membership tracking.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::manual_range_contains)]

use std::ffi::{c_char, c_int};

/// Deleted group sentinel value.
pub const AUGROUP_DELETED: c_int = -4;

// C accessors for group map operations
extern "C" {
    fn nvim_augroup_name_to_id(name: *const c_char) -> c_int;
    fn nvim_augroup_id_to_name(id: c_int) -> *const c_char;
    fn nvim_augroup_put(name: *const c_char, id: c_int);
    fn nvim_augroup_map_del_c(id: c_int, name: *const c_char);
    fn nvim_get_next_augroup_id() -> c_int;
    fn nvim_inc_next_augroup_id() -> c_int;
    fn nvim_autocmd_get_current_augroup() -> c_int;
    fn nvim_get_deleted_augroup() -> *const c_char;
}

// =============================================================================
// Group Constants
// =============================================================================

/// Default/anonymous group ID.
pub const AUGROUP_DEFAULT: c_int = -1;

/// Invalid/error group ID.
pub const AUGROUP_ERROR: c_int = -2;

/// All groups (for deletion/query operations).
pub const AUGROUP_ALL: c_int = -3;

/// Maximum length of a group name.
pub const AUGROUP_NAME_MAX: usize = 200;

// =============================================================================
// Group Flags
// =============================================================================

/// Flags for autocommand groups.
pub mod group_flags {
    use std::ffi::c_int;

    /// Group has been deleted
    pub const AU_GROUP_DELETED: c_int = 0x01;
    /// Group is currently being cleared
    pub const AU_GROUP_CLEARING: c_int = 0x02;
    /// Group was created with ++clear
    pub const AU_GROUP_CLEAR: c_int = 0x04;
}

/// Check if group flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_group_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set a group flag.
#[must_use]
#[inline]
pub const fn set_group_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear a group flag.
#[must_use]
#[inline]
pub const fn clear_group_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Group State
// =============================================================================

/// State for an autocommand group.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GroupState {
    /// Group ID
    pub id: c_int,
    /// Group flags
    pub flags: c_int,
    /// Number of autocommands in this group
    pub aucmd_count: c_int,
}

impl GroupState {
    /// Create a new group state.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            id,
            flags: 0,
            aucmd_count: 0,
        }
    }

    /// Check if group is valid (not deleted or error).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.id >= 0 && !self.is_deleted()
    }

    /// Check if group is deleted.
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        has_group_flag(self.flags, group_flags::AU_GROUP_DELETED)
    }

    /// Check if group is being cleared.
    #[must_use]
    pub const fn is_clearing(&self) -> bool {
        has_group_flag(self.flags, group_flags::AU_GROUP_CLEARING)
    }

    /// Check if group is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.aucmd_count == 0
    }

    /// Mark group as deleted.
    pub fn mark_deleted(&mut self) {
        self.flags = set_group_flag(self.flags, group_flags::AU_GROUP_DELETED);
    }

    /// Mark group as clearing.
    pub fn set_clearing(&mut self, clearing: bool) {
        if clearing {
            self.flags = set_group_flag(self.flags, group_flags::AU_GROUP_CLEARING);
        } else {
            self.flags = clear_group_flag(self.flags, group_flags::AU_GROUP_CLEARING);
        }
    }
}

// =============================================================================
// Group ID Validation
// =============================================================================

/// Result of group ID validation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupIdResult {
    /// Valid group ID
    Valid = 0,
    /// Default/anonymous group
    Default = 1,
    /// Invalid/error group ID
    Invalid = 2,
    /// Group was deleted
    Deleted = 3,
}

impl GroupIdResult {
    /// Check if the result represents a usable group.
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        matches!(self, Self::Valid | Self::Default)
    }
}

/// Validate a group ID.
#[must_use]
pub const fn validate_group_id(group: c_int) -> GroupIdResult {
    if group == AUGROUP_DEFAULT {
        GroupIdResult::Default
    } else if group == AUGROUP_ERROR {
        GroupIdResult::Invalid
    } else if group < AUGROUP_ALL {
        GroupIdResult::Invalid
    } else if group >= 0 {
        GroupIdResult::Valid
    } else {
        // Special negative values (like AUGROUP_ALL)
        GroupIdResult::Valid
    }
}

// =============================================================================
// Group Name Validation
// =============================================================================

/// Result of group name validation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupNameResult {
    /// Valid group name
    Valid = 0,
    /// Name is empty
    Empty = 1,
    /// Name is too long
    TooLong = 2,
    /// Name contains invalid characters
    InvalidChars = 3,
}

impl GroupNameResult {
    /// Check if the name is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Check if a character is valid in a group name.
///
/// Valid characters are: alphanumeric, underscore, hyphen.
#[must_use]
#[inline]
pub const fn is_valid_group_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}

/// Validate a group name (Rust slice version).
#[must_use]
pub fn validate_group_name(name: &[u8]) -> GroupNameResult {
    if name.is_empty() {
        return GroupNameResult::Empty;
    }

    if name.len() > AUGROUP_NAME_MAX {
        return GroupNameResult::TooLong;
    }

    // First character must be alphanumeric or underscore (not hyphen)
    if !name[0].is_ascii_alphanumeric() && name[0] != b'_' {
        return GroupNameResult::InvalidChars;
    }

    // Check all characters
    for &c in name {
        if c == 0 {
            break; // Null terminator
        }
        if !is_valid_group_char(c) {
            return GroupNameResult::InvalidChars;
        }
    }

    GroupNameResult::Valid
}

// =============================================================================
// Pattern Matching
// =============================================================================

/// Pattern match types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PatternType {
    /// Regular file pattern (glob)
    #[default]
    File = 0,
    /// Buffer-local pattern (<buffer>)
    BufLocal = 1,
    /// Event pattern (for User events)
    Event = 2,
}

impl PatternType {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::BufLocal,
            2 => Self::Event,
            _ => Self::File,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// Pattern match result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PatternMatch {
    /// Whether the pattern matched
    pub matched: bool,
    /// Pattern type that matched
    pub pat_type: PatternType,
    /// Buffer number for buffer-local matches
    pub bufnr: c_int,
}

impl PatternMatch {
    /// Create a non-match.
    #[must_use]
    pub const fn no_match() -> Self {
        Self {
            matched: false,
            pat_type: PatternType::File,
            bufnr: 0,
        }
    }

    /// Create a file pattern match.
    #[must_use]
    pub const fn file_match() -> Self {
        Self {
            matched: true,
            pat_type: PatternType::File,
            bufnr: 0,
        }
    }

    /// Create a buffer-local match.
    #[must_use]
    pub const fn buflocal_match(bufnr: c_int) -> Self {
        Self {
            matched: true,
            pat_type: PatternType::BufLocal,
            bufnr,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if group ID is the default/anonymous group.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_augroup_default(group: c_int) -> c_int {
    c_int::from(group == AUGROUP_DEFAULT)
}

/// Check if group ID indicates an error.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_augroup_error(group: c_int) -> c_int {
    c_int::from(group == AUGROUP_ERROR)
}

/// Validate a group ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_validate_augroup(group: c_int) -> c_int {
    validate_group_id(group) as c_int
}

/// Check if group flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_group_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_group_flag(flags, flag))
}

/// Set a group flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_group_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_group_flag(flags, flag)
}

/// Clear a group flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_group_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_group_flag(flags, flag)
}

/// Validate a group name.
///
/// Returns 0 for valid, positive values for various error conditions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_validate_augroup_name(name: *const c_char) -> c_int {
    if name.is_null() {
        return GroupNameResult::Empty as c_int;
    }

    // Convert to slice
    let mut len = 0usize;
    while *name.add(len) != 0 && len < AUGROUP_NAME_MAX + 1 {
        len += 1;
    }

    if len == 0 {
        return GroupNameResult::Empty as c_int;
    }

    let slice = std::slice::from_raw_parts(name.cast::<u8>(), len);
    validate_group_name(slice) as c_int
}

/// Check if a character is valid for group names.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_augroup_char(c: c_int) -> c_int {
    if c < 0 || c > 127 {
        return 0;
    }
    c_int::from(is_valid_group_char(c as u8))
}

// =============================================================================
// Phase 3: Group Management FFI Exports
// =============================================================================

/// Find an autocmd group by name.
///
/// Returns the group ID if found, `AUGROUP_DELETED` if the group was deleted,
/// or `AUGROUP_ERROR` if not found.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "augroup_find")]
pub unsafe extern "C" fn rs_augroup_find(name: *const c_char) -> c_int {
    let existing_id = nvim_augroup_name_to_id(name);
    if existing_id == AUGROUP_DELETED {
        return existing_id;
    }
    if existing_id > 0 {
        return existing_id;
    }
    AUGROUP_ERROR
}

/// Add a new autocmd group or return the existing ID.
///
/// If the group already exists, returns its ID. If it was previously deleted,
/// removes the old entry first and creates a new one.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string, and must not be "end".
#[unsafe(export_name = "augroup_add")]
pub unsafe extern "C" fn rs_augroup_add(name: *const c_char) -> c_int {
    let existing_id = rs_augroup_find(name);
    if existing_id > 0 {
        return existing_id;
    }

    if existing_id == AUGROUP_DELETED {
        nvim_augroup_map_del_c(existing_id, name);
    }

    let next_id = nvim_inc_next_augroup_id();
    nvim_augroup_put(name, next_id);

    next_id
}

/// Get the name of an autocmd group.
///
/// Returns a pointer to the group name string, "END" for next_augroup_id,
/// the deleted sentinel string for deleted groups, or NULL if the group
/// ID is beyond the valid range.
///
/// # Safety
/// The returned pointer is valid as long as the group map entry exists.
#[unsafe(export_name = "augroup_name")]
pub unsafe extern "C" fn rs_augroup_name(mut group: c_int) -> *const c_char {
    if group == AUGROUP_DELETED {
        return nvim_get_deleted_augroup();
    }

    if group == AUGROUP_ALL {
        group = nvim_autocmd_get_current_augroup();
    }

    let next_id = nvim_get_next_augroup_id();

    // "END" is always considered the last augroup ID
    if group == next_id {
        return c"END".as_ptr();
    }

    // Beyond the valid range
    if group > next_id {
        return std::ptr::null();
    }

    let name = nvim_augroup_id_to_name(group);
    if !name.is_null() {
        return name;
    }

    // Not in the map anymore, must have been deleted
    nvim_get_deleted_augroup()
}

/// Check if an autocmd group name exists.
///
/// Returns 1 if the group exists, 0 otherwise.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[export_name = "augroup_exists"]
pub unsafe extern "C" fn rs_augroup_exists(name: *const c_char) -> bool {
    rs_augroup_find(name) > 0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_constants() {
        assert_eq!(AUGROUP_DEFAULT, -1);
        assert_eq!(AUGROUP_ERROR, -2);
        assert_eq!(AUGROUP_ALL, -3);
    }

    #[test]
    fn test_group_flags() {
        let flags = 0;
        assert!(!has_group_flag(flags, group_flags::AU_GROUP_DELETED));

        let flags = set_group_flag(flags, group_flags::AU_GROUP_DELETED);
        assert!(has_group_flag(flags, group_flags::AU_GROUP_DELETED));

        let flags = clear_group_flag(flags, group_flags::AU_GROUP_DELETED);
        assert!(!has_group_flag(flags, group_flags::AU_GROUP_DELETED));
    }

    #[test]
    fn test_group_state() {
        let mut state = GroupState::new(5);
        assert!(state.is_valid());
        assert!(!state.is_deleted());
        assert!(state.is_empty());

        state.mark_deleted();
        assert!(!state.is_valid());
        assert!(state.is_deleted());
    }

    #[test]
    fn test_validate_group_id() {
        assert_eq!(validate_group_id(0), GroupIdResult::Valid);
        assert_eq!(validate_group_id(100), GroupIdResult::Valid);
        assert_eq!(validate_group_id(AUGROUP_DEFAULT), GroupIdResult::Default);
        assert_eq!(validate_group_id(AUGROUP_ERROR), GroupIdResult::Invalid);
        assert_eq!(validate_group_id(-100), GroupIdResult::Invalid);

        assert!(GroupIdResult::Valid.is_usable());
        assert!(GroupIdResult::Default.is_usable());
        assert!(!GroupIdResult::Invalid.is_usable());
    }

    #[test]
    fn test_validate_group_name() {
        assert_eq!(validate_group_name(b""), GroupNameResult::Empty);
        assert_eq!(validate_group_name(b"mygroup"), GroupNameResult::Valid);
        assert_eq!(validate_group_name(b"my_group"), GroupNameResult::Valid);
        assert_eq!(validate_group_name(b"my-group"), GroupNameResult::Valid);
        assert_eq!(validate_group_name(b"MyGroup123"), GroupNameResult::Valid);
        assert_eq!(
            validate_group_name(b"-invalid"),
            GroupNameResult::InvalidChars
        );
        assert_eq!(
            validate_group_name(b"has space"),
            GroupNameResult::InvalidChars
        );
        assert_eq!(
            validate_group_name(b"has.dot"),
            GroupNameResult::InvalidChars
        );

        // Too long
        let long_name = [b'a'; AUGROUP_NAME_MAX + 1];
        assert_eq!(validate_group_name(&long_name), GroupNameResult::TooLong);
    }

    #[test]
    fn test_is_valid_group_char() {
        assert!(is_valid_group_char(b'a'));
        assert!(is_valid_group_char(b'Z'));
        assert!(is_valid_group_char(b'0'));
        assert!(is_valid_group_char(b'_'));
        assert!(is_valid_group_char(b'-'));
        assert!(!is_valid_group_char(b' '));
        assert!(!is_valid_group_char(b'.'));
        assert!(!is_valid_group_char(b'@'));
    }

    #[test]
    fn test_pattern_type() {
        assert_eq!(PatternType::from_raw(0), PatternType::File);
        assert_eq!(PatternType::from_raw(1), PatternType::BufLocal);
        assert_eq!(PatternType::from_raw(2), PatternType::Event);
        assert_eq!(PatternType::from_raw(99), PatternType::File);

        assert_eq!(PatternType::File.to_raw(), 0);
        assert_eq!(PatternType::BufLocal.to_raw(), 1);
    }

    #[test]
    fn test_pattern_match() {
        let no_match = PatternMatch::no_match();
        assert!(!no_match.matched);

        let file_match = PatternMatch::file_match();
        assert!(file_match.matched);
        assert_eq!(file_match.pat_type, PatternType::File);

        let buflocal_match = PatternMatch::buflocal_match(42);
        assert!(buflocal_match.matched);
        assert_eq!(buflocal_match.pat_type, PatternType::BufLocal);
        assert_eq!(buflocal_match.bufnr, 42);
    }
}
