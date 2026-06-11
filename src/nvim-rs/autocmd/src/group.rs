//! Autocommand group management helpers
//!
//! This module provides helpers for working with autocommand groups,
//! including group creation, deletion, and membership tracking.
//!
//! Group map state (name→id, id→name) is owned entirely in Rust.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::manual_range_contains)]
// Neovim is single-threaded; mutable statics are safe in this context.
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::ffi::{c_char, c_int, CString};
use std::sync::OnceLock;

/// Deleted group sentinel value.
pub const AUGROUP_DELETED: c_int = -4;

// =============================================================================
// Rust-owned group map state
// =============================================================================

/// Name → ID map. Owned by Rust.
// SAFETY: Neovim is single-threaded; all access is from the main thread.
static mut AUGROUP_NAME_TO_ID: Option<HashMap<String, c_int>> = None;
/// ID → name map. CString values have stable heap addresses even when HashMap reallocs.
/// This is critical: callers receive `*const c_char` pointers that must remain valid.
static mut AUGROUP_ID_TO_NAME: Option<HashMap<c_int, CString>> = None;
/// Next group ID counter.
static mut NEXT_AUGROUP_ID: c_int = 1;
/// Lazy-init translated "--Deleted--" sentinel.
static DELETED_AUGROUP: OnceLock<CString> = OnceLock::new();

// C functions needed for map operations
extern "C" {
    fn gettext(msgid: *const c_char) -> *const c_char;
    static mut current_augroup: c_int;
    // Messaging
    fn msg_start();
    fn msg_end();
    fn msg_clr_eos();
    fn msg_puts(s: *const c_char);
    fn msg_ext_set_kind(kind: *const c_char);
    // Error/warning reporting
    fn nvim_autocmd_semsg_str(fmt: *const c_char, arg: *const c_char);
    #[link_name = "emsg"]
    fn group_emsg(msg: *const c_char);
    fn give_warning(message: *const c_char, hl: bool);
    // Autocmd ops used by augroup_del
    fn nvim_autocmd_get_pat_info(event: c_int, idx: usize) -> crate::AutoPatInfo;
    fn nvim_get_autocmds_count(event: c_int) -> usize;
    fn nvim_autocmd_del_at(event: c_int, idx: usize);
    // au_cleanup exported from lib.rs under C name
    #[link_name = "au_cleanup"]
    fn group_au_cleanup();
}

#[inline]
fn name_to_id_map() -> &'static mut HashMap<String, c_int> {
    // SAFETY: single-threaded
    unsafe { AUGROUP_NAME_TO_ID.get_or_insert_with(HashMap::new) }
}

#[inline]
fn id_to_name_map() -> &'static mut HashMap<c_int, CString> {
    // SAFETY: single-threaded
    unsafe { AUGROUP_ID_TO_NAME.get_or_insert_with(HashMap::new) }
}

/// Look up a group id by name. Returns 0 (not found), AUGROUP_DELETED, or a positive ID.
pub(crate) fn augroup_name_to_id(name: &str) -> c_int {
    name_to_id_map().get(name).copied().unwrap_or(0)
}

/// Look up a group name by id. Returns a stable pointer to the NUL-terminated name, or null.
///
/// The returned pointer is stable because CString allocates the string separately on the heap.
/// Even when the HashMap is reallocated on insertion, the CString's string buffer doesn't move.
pub(crate) fn augroup_id_to_name_ptr(id: c_int) -> *const c_char {
    id_to_name_map()
        .get(&id)
        .map_or(std::ptr::null(), |s| s.as_ptr())
}

/// Insert a group into both maps. The name→id map uses String keys (for lookup);
/// the id→name map uses CString values (for stable pointer returns).
pub(crate) fn augroup_put(name: &str, id: c_int) {
    name_to_id_map().insert(name.to_owned(), id);
    let cname = CString::new(name).unwrap_or_default();
    id_to_name_map().insert(id, cname);
}

/// Remove a group from both maps.
pub(crate) fn augroup_map_del(id: c_int, name: Option<&str>) {
    if let Some(n) = name {
        name_to_id_map().remove(n);
    }
    if id > 0 {
        id_to_name_map().remove(&id);
    }
}

/// Return and increment the next augroup ID counter.
pub(crate) fn inc_next_augroup_id() -> c_int {
    // SAFETY: single-threaded
    unsafe {
        let id = NEXT_AUGROUP_ID;
        NEXT_AUGROUP_ID += 1;
        id
    }
}

/// Return the current next augroup ID without incrementing.
pub(crate) fn next_augroup_id() -> c_int {
    // SAFETY: single-threaded
    unsafe { NEXT_AUGROUP_ID }
}

/// Return a pointer to the translated "--Deleted--" sentinel.
/// The CString is lazily initialised on first call.
pub(crate) fn get_deleted_augroup() -> *const c_char {
    DELETED_AUGROUP
        .get_or_init(|| {
            // SAFETY: gettext returns a pointer to a translated string (static or thread-local).
            // We copy it into an owned CString so the lifetime is 'static.
            let translated = unsafe {
                let raw = gettext(c"--Deleted--".as_ptr());
                std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned()
            };
            CString::new(translated).unwrap_or_else(|_| CString::new("--Deleted--").unwrap())
        })
        .as_ptr()
}

/// Free all group map state. Called from `free_all_autocmds` during EXITFREE.
#[unsafe(no_mangle)]
pub extern "C" fn rs_free_augroup_maps() {
    // SAFETY: single-threaded; called during shutdown
    unsafe {
        AUGROUP_NAME_TO_ID = None;
        AUGROUP_ID_TO_NAME = None;
        NEXT_AUGROUP_ID = 1;
    }
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
    let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
    let existing_id = augroup_name_to_id(name_str.as_ref());
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
    let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
    let existing_id = rs_augroup_find(name);
    if existing_id > 0 {
        return existing_id;
    }

    if existing_id == AUGROUP_DELETED {
        augroup_map_del(existing_id, Some(name_str.as_ref()));
    }

    let next_id = inc_next_augroup_id();
    augroup_put(name_str.as_ref(), next_id);

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
        return get_deleted_augroup();
    }

    if group == AUGROUP_ALL {
        group = current_augroup;
    }

    let nid = next_augroup_id();

    // "END" is always considered the last augroup ID
    if group == nid {
        return c"END".as_ptr();
    }

    // Beyond the valid range
    if group > nid {
        return std::ptr::null();
    }

    let name_ptr = augroup_id_to_name_ptr(group);
    if !name_ptr.is_null() {
        return name_ptr;
    }

    // Not in the map anymore, must have been deleted
    get_deleted_augroup()
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

/// Delete the augroup that matches name.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "augroup_del")]
pub unsafe extern "C" fn rs_augroup_del(name: *const c_char, stupid_legacy_mode: bool) {
    use crate::NUM_EVENTS;

    let group = rs_augroup_find(name);
    if group == AUGROUP_ERROR {
        nvim_autocmd_semsg_str(c"E367: No such group: \"%s\"".as_ptr(), name);
        return;
    }
    if group == current_augroup {
        group_emsg(c"E936: Cannot delete the current group".as_ptr());
        return;
    }

    if stupid_legacy_mode {
        'outer: {
            for event in 0..NUM_EVENTS {
                let size = nvim_get_autocmds_count(event);
                for i in 0..size {
                    let info = nvim_autocmd_get_pat_info(event, i);
                    if info.is_null == 0 && info.group == group {
                        give_warning(c"W19: Deleting augroup that is still in use".as_ptr(), true);
                        // Mark the name entry as AUGROUP_DELETED in name→id map
                        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
                        name_to_id_map().insert(name_str.into_owned(), AUGROUP_DELETED);
                        // Remove from id→name map only
                        augroup_map_del(info.group, None);
                        break 'outer;
                    }
                }
            }
            // No autocmds found: remove the group entirely
            let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
            augroup_map_del(group, Some(name_str.as_ref()));
        }
    } else {
        for event in 0..NUM_EVENTS {
            let size = nvim_get_autocmds_count(event);
            for i in 0..size {
                let info = nvim_autocmd_get_pat_info(event, i);
                if info.is_null == 0 && info.group == group {
                    nvim_autocmd_del_at(event, i);
                }
            }
        }
        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
        augroup_map_del(group, Some(name_str.as_ref()));
        group_au_cleanup();
    }
}

/// Iterate the augroup name→id map, calling msg_puts for each entry.
/// Called by `do_augroup` when arg is empty.
pub(crate) unsafe fn list_group_names() {
    msg_start();
    msg_ext_set_kind(c"list_cmd".as_ptr());

    // Collect names to avoid holding mutable borrow during msg_puts
    let names: Vec<(c_int, String)> = name_to_id_map()
        .iter()
        .map(|(k, &v)| (v, k.clone()))
        .collect();

    for (value, name) in &names {
        if *value > 0 {
            let cname = CString::new(name.as_str()).unwrap_or_default();
            msg_puts(cname.as_ptr());
        } else {
            msg_puts(rs_augroup_name(*value));
        }
        msg_puts(c"  ".as_ptr());
    }

    msg_clr_eos();
    msg_end();
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
