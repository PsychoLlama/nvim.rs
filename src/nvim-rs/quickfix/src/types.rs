//! Quickfix data structure types
//!
//! This module provides Rust type-safe wrappers for the quickfix data structures.
//! These mirror the C structures (`qfline_T`, `qf_list_T`, `qf_info_T`) and provide
//! safe accessor methods through C FFI functions.
//!
//! # Design Pattern
//!
//! We use the "Opaque Handle" pattern defined in the project guidelines:
//! - C structures remain in C
//! - Rust accesses fields through C accessor functions
//! - Newtype wrappers provide type safety and null checks

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::missing_safety_doc)]

use crate::ffi_types::QfListPtr;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::marker::PhantomData;
use std::ptr::NonNull;

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of quickfix lists in a stack
pub const QF_LISTCOUNT: c_int = 10;

/// Invalid quickfix index
pub const QF_INVALID_IDX: c_int = -1;

/// Invalid buffer number
pub const QF_INVALID_BUFNR: c_int = 0;

// =============================================================================
// Quickfix List Type
// =============================================================================

/// Type of quickfix/location list
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QfListType {
    /// Global quickfix list
    Quickfix = 0,
    /// Window-local location list
    Location = 1,
    /// Internal temporary list
    Internal = 2,
}

impl QfListType {
    /// Convert from C integer representation
    pub const fn from_c_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Quickfix),
            1 => Some(Self::Location),
            2 => Some(Self::Internal),
            _ => None,
        }
    }

    /// Convert to C integer representation
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a location list
    pub const fn is_location_list(self) -> bool {
        matches!(self, Self::Location)
    }

    /// Check if this is a quickfix list
    pub const fn is_quickfix_list(self) -> bool {
        matches!(self, Self::Quickfix)
    }
}

// =============================================================================
// Entry Type (Error/Warning/Info/Note)
// =============================================================================

/// Quickfix entry type (E/W/I/N)
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QfEntryType {
    /// Error entry ('E')
    Error = b'E' as isize,
    /// Warning entry ('W')
    Warning = b'W' as isize,
    /// Info entry ('I')
    Info = b'I' as isize,
    /// Note entry ('N')
    Note = b'N' as isize,
    /// No type (empty)
    #[default]
    None = 0,
}

impl QfEntryType {
    /// Convert from C char representation
    pub const fn from_char(c: c_char) -> Self {
        // Handle both upper and lowercase
        let upper = if c >= b'a' as c_char && c <= b'z' as c_char {
            c - 32
        } else {
            c
        };

        match upper as u8 {
            b'E' => Self::Error,
            b'W' => Self::Warning,
            b'I' => Self::Info,
            b'N' => Self::Note,
            _ => Self::None,
        }
    }

    /// Convert to C char representation
    pub const fn to_char(self) -> c_char {
        match self {
            Self::Error => b'E' as c_char,
            Self::Warning => b'W' as c_char,
            Self::Info => b'I' as c_char,
            Self::Note => b'N' as c_char,
            Self::None => 0,
        }
    }

    /// Check if this is an error type
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    /// Check if this is a warning type
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Check if this indicates a problem (error or warning)
    pub const fn is_problem(self) -> bool {
        matches!(self, Self::Error | Self::Warning)
    }
}

// =============================================================================
// Line Number Type
// =============================================================================

/// Line number type (matches C `linenr_T`)
pub type LineNr = i32;

// =============================================================================
// External C Accessor Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Stack accessors
    fn nvim_qf_get_listcount(qi: *const c_void) -> c_int;
    fn nvim_qf_get_curlist_idx(qi: *const c_void) -> c_int;
    fn nvim_qf_get_curlist(qi: *const c_void) -> QfListPtr;
    fn nvim_qf_get_list_at(qi: *const c_void, idx: c_int) -> QfListPtr;
    fn nvim_qf_get_maxcount(qi: *const c_void) -> c_int;
    fn nvim_qf_get_bufnr(qi: *const c_void) -> c_int;
    fn nvim_qf_get_qi_type(qi: *const c_void) -> c_int;
    fn nvim_qf_get_refcount(qi: *const c_void) -> c_int;
    fn nvim_qf_is_qf_stack(qi: *const c_void) -> bool;
    fn nvim_qf_is_ll_stack(qi: *const c_void) -> bool;

    // List accessors

    // Entry accessors

    // Global quickfix stack accessor
    fn nvim_get_ql_info() -> *mut c_void;
}

// =============================================================================
// Opaque Handle: QfEntry (qfline_T)
// =============================================================================

/// Handle to a quickfix entry (`qfline_T`)
///
/// This is an opaque wrapper around the C `qfline_T` structure.
/// All field access goes through C accessor functions.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct QfEntry(NonNull<c_void>);

impl QfEntry {
    /// Create a new entry handle from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid pointer to a `qfline_T` struct.
    pub unsafe fn from_raw(ptr: *mut crate::ffi_types::QfLineRaw) -> Option<Self> {
        NonNull::new(ptr.cast::<c_void>()).map(Self)
    }

    /// Create a new entry handle from a raw pointer, or return a null handle
    ///
    /// # Safety
    /// If non-null, the pointer must be valid.
    pub unsafe fn from_raw_unchecked(ptr: *mut crate::ffi_types::QfLineRaw) -> Self {
        Self(NonNull::new_unchecked(ptr.cast::<c_void>()))
    }

    /// Get the raw pointer
    pub fn as_ptr(self) -> *mut crate::ffi_types::QfLineRaw {
        self.0.as_ptr().cast::<crate::ffi_types::QfLineRaw>()
    }

    /// Get line number
    pub fn lnum(self) -> LineNr {
        unsafe { (*self.as_ptr()).qf_lnum }
    }

    /// Get end line number (0 if no range)
    pub fn end_lnum(self) -> LineNr {
        unsafe { (*self.as_ptr()).qf_end_lnum }
    }

    /// Get column number
    pub fn col(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_col }
    }

    /// Get end column number (0 if no range)
    pub fn end_col(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_end_col }
    }

    /// Get file/buffer number
    pub fn fnum(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_fnum }
    }

    /// Get error number
    pub fn nr(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_nr }
    }

    /// Get entry type
    pub fn entry_type(self) -> QfEntryType {
        let c = unsafe { (*self.as_ptr()).qf_type };
        QfEntryType::from_char(c)
    }

    /// Get entry type as raw char
    pub fn type_char(self) -> c_char {
        unsafe { (*self.as_ptr()).qf_type }
    }

    /// Check if entry is valid
    pub fn is_valid(self) -> bool {
        unsafe { (*self.as_ptr()).qf_valid != 0 }
    }

    /// Check if entry has been cleared (line deleted)
    pub fn is_cleared(self) -> bool {
        unsafe { (*self.as_ptr()).qf_cleared != 0 }
    }

    /// Check if column is a screen column (vs byte offset)
    pub fn is_viscol(self) -> bool {
        unsafe { (*self.as_ptr()).qf_viscol != 0 }
    }

    /// Get next entry in the list
    pub fn next(self) -> Option<Self> {
        let ptr = unsafe { (*self.as_ptr()).qf_next };
        unsafe { Self::from_raw(ptr) }
    }

    /// Get previous entry in the list
    pub fn prev(self) -> Option<Self> {
        let ptr = unsafe { (*self.as_ptr()).qf_prev };
        unsafe { Self::from_raw(ptr) }
    }

    /// Get the error text/message
    ///
    /// Returns `None` if text is null
    pub fn text(self) -> Option<&'static CStr> {
        let ptr = unsafe { (*self.as_ptr()).qf_text };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Get the module name
    ///
    /// Returns `None` if module is null
    pub fn module(self) -> Option<&'static CStr> {
        let ptr = unsafe { (*self.as_ptr()).qf_module };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Get the search pattern
    ///
    /// Returns `None` if pattern is null
    pub fn pattern(self) -> Option<&'static CStr> {
        let ptr = unsafe { (*self.as_ptr()).qf_pattern };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Check if entry is active (valid and not cleared)
    pub fn is_active(self) -> bool {
        self.is_valid() && !self.is_cleared()
    }

    /// Check if entry has a line range (end_lnum != 0)
    pub fn has_line_range(self) -> bool {
        self.end_lnum() != 0
    }

    /// Check if entry has a column range (end_col != 0)
    pub fn has_col_range(self) -> bool {
        self.end_col() != 0
    }

    /// Check if entry is in a specific file
    pub fn in_file(self, bnr: c_int) -> bool {
        self.fnum() == bnr
    }

    /// Check if entry covers a specific line
    pub fn covers_line(self, line: LineNr) -> bool {
        let start = self.lnum();
        let end = self.end_lnum();
        if end == 0 {
            line == start
        } else {
            line >= start && line <= end
        }
    }
}

impl std::fmt::Debug for QfEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QfEntry")
            .field("ptr", &self.as_ptr())
            .field("lnum", &self.lnum())
            .field("col", &self.col())
            .field("fnum", &self.fnum())
            .field("valid", &self.is_valid())
            .field("type", &self.entry_type())
            .finish()
    }
}

// =============================================================================
// Nullable Entry Handle
// =============================================================================

/// Nullable handle to a quickfix entry
///
/// This is useful for FFI where we need to handle null pointers.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct QfEntryOption(*mut crate::ffi_types::QfLineRaw);

impl QfEntryOption {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Create from a raw pointer
    pub const fn from_raw(ptr: *mut crate::ffi_types::QfLineRaw) -> Self {
        Self(ptr)
    }

    /// Check if the handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer
    pub const fn as_ptr(self) -> *mut crate::ffi_types::QfLineRaw {
        self.0
    }

    /// Convert to an Option<QfEntry>
    pub fn as_option(self) -> Option<QfEntry> {
        if self.is_null() {
            None
        } else {
            unsafe { QfEntry::from_raw(self.0) }
        }
    }

    /// Get entry or return None
    pub fn get(self) -> Option<QfEntry> {
        self.as_option()
    }
}

impl From<Option<QfEntry>> for QfEntryOption {
    fn from(opt: Option<QfEntry>) -> Self {
        match opt {
            Some(e) => Self(e.as_ptr()),
            None => Self::null(),
        }
    }
}

impl std::fmt::Debug for QfEntryOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_option() {
            Some(e) => write!(f, "Some({e:?})"),
            None => write!(f, "None"),
        }
    }
}

// =============================================================================
// Opaque Handle: QfList (qf_list_T)
// =============================================================================

/// Handle to a quickfix list (`qf_list_T`)
///
/// This is an opaque wrapper around the C `qf_list_T` structure.
/// All field access goes through C accessor functions.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct QfList(NonNull<crate::ffi_types::QfListRaw>);

impl QfList {
    /// Create a new list handle from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid pointer to a `qf_list_T` struct.
    pub unsafe fn from_raw(ptr: *mut crate::ffi_types::QfListRaw) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    /// Get the raw pointer
    pub fn as_ptr(self) -> *mut crate::ffi_types::QfListRaw {
        self.0.as_ptr()
    }

    /// Get the unique list ID
    pub fn id(self) -> u32 {
        unsafe { (*self.as_ptr()).qf_id }
    }

    /// Get the number of entries
    pub fn count(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_count }
    }

    /// Get the current entry index (1-based)
    pub fn index(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_index }
    }

    /// Check if list is empty
    pub fn is_empty(self) -> bool {
        self.count() <= 0
    }

    /// Check if all entries are invalid
    pub fn is_nonevalid(self) -> bool {
        unsafe { (*self.as_ptr()).qf_nonevalid }
    }

    /// Check if list has valid entries
    pub fn has_valid_entries(self) -> bool {
        !self.is_empty() && !self.is_nonevalid()
    }

    /// Get the changedtick (modification counter)
    pub fn changedtick(self) -> c_int {
        unsafe { (*self.as_ptr()).qf_changedtick }
    }

    /// Get the list type
    pub fn list_type(self) -> QfListType {
        let t = unsafe { (*self.as_ptr()).qfl_type };
        QfListType::from_c_int(t as c_int).unwrap_or(QfListType::Quickfix)
    }

    /// Check if list has user data
    pub fn has_user_data(self) -> bool {
        unsafe { (*self.as_ptr()).qf_has_user_data }
    }

    /// Get the first entry
    pub fn first_entry(self) -> Option<QfEntry> {
        let ptr = unsafe { (*self.as_ptr()).qf_start };
        unsafe { QfEntry::from_raw(ptr) }
    }

    /// Get the last entry
    pub fn last_entry(self) -> Option<QfEntry> {
        let ptr = unsafe { (*self.as_ptr()).qf_last };
        unsafe { QfEntry::from_raw(ptr) }
    }

    /// Get the current entry (qf_ptr)
    pub fn current_entry(self) -> Option<QfEntry> {
        let ptr = unsafe { (*self.as_ptr()).qf_ptr };
        unsafe { QfEntry::from_raw(ptr) }
    }

    /// Get the list title
    pub fn title(self) -> Option<&'static CStr> {
        let ptr = unsafe { (*self.as_ptr()).qf_title };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Check if list has a title
    pub fn has_title(self) -> bool {
        !unsafe { (*self.as_ptr()).qf_title }.is_null()
    }

    /// Get multiline parsing flag
    pub fn is_multiline(self) -> bool {
        unsafe { (*self.as_ptr()).qf_multiline }
    }

    /// Get multiignore parsing flag
    pub fn is_multiignore(self) -> bool {
        unsafe { (*self.as_ptr()).qf_multiignore }
    }

    /// Get multiscan parsing flag
    pub fn is_multiscan(self) -> bool {
        unsafe { (*self.as_ptr()).qf_multiscan }
    }

    /// Create an iterator over entries
    pub fn iter(self) -> QfEntryIter {
        QfEntryIter {
            current: self.first_entry(),
            idx: 1,
            count: self.count(),
            _marker: PhantomData,
        }
    }
}

impl std::fmt::Debug for QfList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QfList")
            .field("ptr", &self.as_ptr())
            .field("id", &self.id())
            .field("count", &self.count())
            .field("index", &self.index())
            .field("type", &self.list_type())
            .finish()
    }
}

// =============================================================================
// List Iterator
// =============================================================================

/// Iterator over quickfix entries
pub struct QfEntryIter {
    current: Option<QfEntry>,
    idx: c_int,
    count: c_int,
    _marker: PhantomData<*const ()>,
}

impl Iterator for QfEntryIter {
    type Item = (c_int, QfEntry);

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.current?;
        if self.idx > self.count {
            return None;
        }
        let result = (self.idx, entry);
        self.idx += 1;
        self.current = entry.next();
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.idx + 1).max(0) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QfEntryIter {}

// =============================================================================
// Opaque Handle: QfStack (qf_info_T)
// =============================================================================

/// Handle to a quickfix stack (`qf_info_T`)
///
/// This is an opaque wrapper around the C `qf_info_T` structure.
/// All field access goes through C accessor functions.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct QfStack(NonNull<c_void>);

impl QfStack {
    /// Create a new stack handle from a raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid pointer to a `qf_info_T` struct.
    pub unsafe fn from_raw(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    /// Get the raw pointer
    pub fn as_ptr(self) -> *mut c_void {
        self.0.as_ptr()
    }

    /// Get the global quickfix stack
    ///
    /// # Safety
    /// This should only be called when Neovim is properly initialized.
    pub unsafe fn global() -> Option<Self> {
        Self::from_raw(nvim_get_ql_info())
    }

    /// Get the number of lists in the stack
    pub fn list_count(self) -> c_int {
        unsafe { nvim_qf_get_listcount(self.as_ptr()) }
    }

    /// Get the current list index (0-based)
    pub fn cur_list_idx(self) -> c_int {
        unsafe { nvim_qf_get_curlist_idx(self.as_ptr()) }
    }

    /// Get the maximum number of lists
    pub fn max_count(self) -> c_int {
        unsafe { nvim_qf_get_maxcount(self.as_ptr()) }
    }

    /// Get the quickfix window buffer number
    pub fn bufnr(self) -> c_int {
        unsafe { nvim_qf_get_bufnr(self.as_ptr()) }
    }

    /// Get the reference count (for location lists)
    pub fn refcount(self) -> c_int {
        unsafe { nvim_qf_get_refcount(self.as_ptr()) }
    }

    /// Check if stack is empty
    pub fn is_empty(self) -> bool {
        self.list_count() <= 0
    }

    /// Check if stack is full
    pub fn is_full(self) -> bool {
        self.list_count() >= self.max_count()
    }

    /// Check if this is a global quickfix stack
    pub fn is_quickfix_stack(self) -> bool {
        unsafe { nvim_qf_is_qf_stack(self.as_ptr()) }
    }

    /// Check if this is a location list stack
    pub fn is_location_stack(self) -> bool {
        unsafe { nvim_qf_is_ll_stack(self.as_ptr()) }
    }

    /// Get the current list
    pub fn current_list(self) -> Option<QfList> {
        let ptr = unsafe { nvim_qf_get_curlist(self.as_ptr()) };
        unsafe { QfList::from_raw(ptr) }
    }

    /// Get a list at a specific index
    pub fn list_at(self, idx: c_int) -> Option<QfList> {
        if idx < 0 || idx >= self.list_count() {
            return None;
        }
        let ptr = unsafe { nvim_qf_get_list_at(self.as_ptr(), idx) };
        unsafe { QfList::from_raw(ptr) }
    }

    /// Get stack type
    pub fn stack_type(self) -> QfListType {
        let t = unsafe { nvim_qf_get_qi_type(self.as_ptr()) };
        QfListType::from_c_int(t).unwrap_or(QfListType::Quickfix)
    }

    /// Check if we can navigate to an older list
    pub fn can_go_older(self) -> bool {
        self.cur_list_idx() > 0
    }

    /// Check if we can navigate to a newer list
    pub fn can_go_newer(self) -> bool {
        self.cur_list_idx() < self.list_count() - 1
    }

    /// Check if a list index is valid
    pub fn is_valid_idx(self, idx: c_int) -> bool {
        idx >= 0 && idx < self.list_count()
    }
}

impl std::fmt::Debug for QfStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QfStack")
            .field("ptr", &self.as_ptr())
            .field("list_count", &self.list_count())
            .field("cur_idx", &self.cur_list_idx())
            .field("type", &self.stack_type())
            .finish()
    }
}

// =============================================================================
// Entry Position
// =============================================================================

/// Position information from a quickfix entry
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QfPosition {
    /// Line number
    pub lnum: LineNr,
    /// Column number
    pub col: c_int,
    /// End line number (0 if no range)
    pub end_lnum: LineNr,
    /// End column number (0 if no range)
    pub end_col: c_int,
    /// File/buffer number
    pub fnum: c_int,
    /// Whether column is screen column
    pub viscol: bool,
}

impl QfPosition {
    /// Create from a quickfix entry
    pub fn from_entry(entry: QfEntry) -> Self {
        Self {
            lnum: entry.lnum(),
            col: entry.col(),
            end_lnum: entry.end_lnum(),
            end_col: entry.end_col(),
            fnum: entry.fnum(),
            viscol: entry.is_viscol(),
        }
    }

    /// Check if position has a range
    pub const fn has_range(&self) -> bool {
        self.end_lnum != 0 || self.end_col != 0
    }

    /// Check if position is in a specific file
    pub const fn in_file(&self, bnr: c_int) -> bool {
        self.fnum == bnr
    }
}

// =============================================================================
// Entry Summary
// =============================================================================

/// Summary information about a quickfix entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfEntrySummary {
    /// Position information
    pub position: QfPosition,
    /// Entry type
    pub entry_type: QfEntryType,
    /// Error number
    pub nr: c_int,
    /// Whether entry is valid
    pub valid: bool,
    /// Whether entry has been cleared
    pub cleared: bool,
}

impl QfEntrySummary {
    /// Create from a quickfix entry
    pub fn from_entry(entry: QfEntry) -> Self {
        Self {
            position: QfPosition::from_entry(entry),
            entry_type: entry.entry_type(),
            nr: entry.nr(),
            valid: entry.is_valid(),
            cleared: entry.is_cleared(),
        }
    }

    /// Check if entry is active (valid and not cleared)
    pub const fn is_active(&self) -> bool {
        self.valid && !self.cleared
    }
}

// =============================================================================
// List Statistics
// =============================================================================

/// Statistics about a quickfix list
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfListStats {
    /// Total number of entries
    pub total_entries: c_int,
    /// Number of valid entries
    pub valid_entries: c_int,
    /// Number of error entries
    pub errors: c_int,
    /// Number of warning entries
    pub warnings: c_int,
    /// Number of info entries
    pub info: c_int,
    /// Number of note entries
    pub notes: c_int,
    /// Number of distinct files
    pub file_count: c_int,
    /// Current entry index
    pub current_index: c_int,
    /// List ID
    pub list_id: u32,
    /// Changedtick
    pub changedtick: c_int,
}

impl QfListStats {
    /// Compute statistics for a list
    pub fn from_list(list: QfList) -> Self {
        let mut stats = Self {
            total_entries: list.count(),
            current_index: list.index(),
            list_id: list.id(),
            changedtick: list.changedtick(),
            ..Default::default()
        };

        let mut last_fnum = -1;

        for (_, entry) in list.iter() {
            if entry.is_valid() {
                stats.valid_entries += 1;
            }

            match entry.entry_type() {
                QfEntryType::Error => stats.errors += 1,
                QfEntryType::Warning => stats.warnings += 1,
                QfEntryType::Info => stats.info += 1,
                QfEntryType::Note => stats.notes += 1,
                QfEntryType::None => {}
            }

            let fnum = entry.fnum();
            if fnum != last_fnum && fnum > 0 {
                stats.file_count += 1;
                last_fnum = fnum;
            }
        }

        stats
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get the global quickfix stack as a raw pointer
#[no_mangle]
pub extern "C" fn rs_qf_get_global_stack() -> *const c_void {
    unsafe { nvim_get_ql_info() }
}

/// Check if a stack handle is valid (non-null)
#[no_mangle]
pub extern "C" fn rs_qf_stack_valid(qi: *const c_void) -> bool {
    !qi.is_null()
}

/// Check if a list handle is valid (non-null)
#[no_mangle]
pub extern "C" fn rs_qf_list_valid(qfl: QfListPtr) -> bool {
    !qfl.is_null()
}

/// Check if an entry handle is valid (non-null)
#[no_mangle]
pub extern "C" fn rs_qf_entry_valid(qfp: *const c_void) -> bool {
    !qfp.is_null()
}

/// Get entry type from a quickfix entry
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_get_type(qfp: *const c_void) -> c_int {
    if let Some(entry) = QfEntry::from_raw(qfp as *mut crate::ffi_types::QfLineRaw) {
        entry.entry_type() as c_int
    } else {
        QfEntryType::None as c_int
    }
}

/// Get position from a quickfix entry
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_get_position(qfp: *const c_void) -> QfPosition {
    if let Some(entry) = QfEntry::from_raw(qfp as *mut crate::ffi_types::QfLineRaw) {
        QfPosition::from_entry(entry)
    } else {
        QfPosition::default()
    }
}

/// Get summary from a quickfix entry
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_get_summary(qfp: *const c_void) -> QfEntrySummary {
    if let Some(entry) = QfEntry::from_raw(qfp as *mut crate::ffi_types::QfLineRaw) {
        QfEntrySummary::from_entry(entry)
    } else {
        QfEntrySummary {
            position: QfPosition::default(),
            entry_type: QfEntryType::None,
            nr: 0,
            valid: false,
            cleared: false,
        }
    }
}

/// Get statistics for a quickfix list
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_get_stats(qfl: QfListPtr) -> QfListStats {
    if let Some(list) = QfList::from_raw(qfl) {
        QfListStats::from_list(list)
    } else {
        QfListStats::default()
    }
}

/// Get list type
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_get_type(qfl: QfListPtr) -> c_int {
    if let Some(list) = QfList::from_raw(qfl) {
        list.list_type().to_c_int()
    } else {
        QfListType::Quickfix.to_c_int()
    }
}

/// Get stack type
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_get_type(qi: *const c_void) -> c_int {
    if let Some(stack) = QfStack::from_raw(qi as *mut c_void) {
        stack.stack_type().to_c_int()
    } else {
        QfListType::Quickfix.to_c_int()
    }
}

/// Convert entry type char to enum int value
#[no_mangle]
pub extern "C" fn rs_qf_entry_type_char_to_int(c: c_char) -> c_int {
    QfEntryType::from_char(c) as c_int
}

/// Convert entry type int value to char
#[no_mangle]
pub extern "C" fn rs_qf_entry_type_int_to_char(t: c_int) -> c_char {
    match t {
        x if x == QfEntryType::Error as c_int => QfEntryType::Error.to_char(),
        x if x == QfEntryType::Warning as c_int => QfEntryType::Warning.to_char(),
        x if x == QfEntryType::Info as c_int => QfEntryType::Info.to_char(),
        x if x == QfEntryType::Note as c_int => QfEntryType::Note.to_char(),
        _ => 0,
    }
}

/// Check if entry type is a problem (error or warning)
#[no_mangle]
pub extern "C" fn rs_qf_entry_type_is_problem(t: c_int) -> bool {
    matches!(
        t,
        x if x == QfEntryType::Error as c_int || x == QfEntryType::Warning as c_int
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_from_char() {
        assert_eq!(QfEntryType::from_char(b'E' as c_char), QfEntryType::Error);
        assert_eq!(QfEntryType::from_char(b'e' as c_char), QfEntryType::Error);
        assert_eq!(QfEntryType::from_char(b'W' as c_char), QfEntryType::Warning);
        assert_eq!(QfEntryType::from_char(b'w' as c_char), QfEntryType::Warning);
        assert_eq!(QfEntryType::from_char(b'I' as c_char), QfEntryType::Info);
        assert_eq!(QfEntryType::from_char(b'N' as c_char), QfEntryType::Note);
        assert_eq!(QfEntryType::from_char(b'X' as c_char), QfEntryType::None);
        assert_eq!(QfEntryType::from_char(0), QfEntryType::None);
    }

    #[test]
    fn test_entry_type_to_char() {
        assert_eq!(QfEntryType::Error.to_char(), b'E' as c_char);
        assert_eq!(QfEntryType::Warning.to_char(), b'W' as c_char);
        assert_eq!(QfEntryType::Info.to_char(), b'I' as c_char);
        assert_eq!(QfEntryType::Note.to_char(), b'N' as c_char);
        assert_eq!(QfEntryType::None.to_char(), 0);
    }

    #[test]
    fn test_entry_type_classification() {
        assert!(QfEntryType::Error.is_error());
        assert!(!QfEntryType::Warning.is_error());
        assert!(QfEntryType::Warning.is_warning());
        assert!(QfEntryType::Error.is_problem());
        assert!(QfEntryType::Warning.is_problem());
        assert!(!QfEntryType::Info.is_problem());
    }

    #[test]
    fn test_list_type() {
        assert_eq!(QfListType::from_c_int(0), Some(QfListType::Quickfix));
        assert_eq!(QfListType::from_c_int(1), Some(QfListType::Location));
        assert_eq!(QfListType::from_c_int(2), Some(QfListType::Internal));
        assert_eq!(QfListType::from_c_int(3), None);

        assert!(QfListType::Location.is_location_list());
        assert!(QfListType::Quickfix.is_quickfix_list());
    }

    #[test]
    fn test_position_has_range() {
        let pos_no_range = QfPosition {
            lnum: 10,
            col: 5,
            end_lnum: 0,
            end_col: 0,
            fnum: 1,
            viscol: false,
        };
        assert!(!pos_no_range.has_range());

        let pos_with_end_lnum = QfPosition {
            lnum: 10,
            col: 5,
            end_lnum: 15,
            end_col: 0,
            fnum: 1,
            viscol: false,
        };
        assert!(pos_with_end_lnum.has_range());

        let pos_with_end_col = QfPosition {
            lnum: 10,
            col: 5,
            end_lnum: 0,
            end_col: 10,
            fnum: 1,
            viscol: false,
        };
        assert!(pos_with_end_col.has_range());
    }

    #[test]
    fn test_entry_option_null() {
        let opt = QfEntryOption::null();
        assert!(opt.is_null());
        assert!(opt.get().is_none());
    }

    #[test]
    fn test_list_stats_default() {
        let stats = QfListStats::default();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.valid_entries, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.warnings, 0);
    }
}
