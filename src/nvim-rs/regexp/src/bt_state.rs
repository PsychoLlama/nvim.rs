//! Backtracking (BT) regex engine state types.
//!
//! This module defines the state types used by the BT regex engine,
//! including the backtrack stack and save/restore mechanisms.
//!
//! # Architecture
//!
//! The BT engine uses a stack-based recursive descent matching algorithm:
//! - [`RegItem`]: Stack item representing a match attempt state
//! - [`RegBehind`]: State saved for look-behind matching
//! - [`RegStar`]: State for `*`, `+`, and `{n,m}` repetition
//! - [`BackPos`]: Tracks positions for BACK (loop detection)
//!
//! Unlike the NFA engine which tracks all possible states in parallel,
//! the BT engine explores one path at a time and backtracks on failure.

use std::ffi::c_int;
use std::ptr;

use crate::bt_opcodes::{BACKPOS_INITIAL, REGSTACK_INITIAL};

// =============================================================================
// Number of subexpressions
// =============================================================================

/// Maximum number of subexpressions (\1 through \9 plus the whole match).
pub const NSUBEXP: usize = 10;

// =============================================================================
// RegState - State types for regstack items
// =============================================================================

/// State types for regstack items.
///
/// These values indicate what kind of backtracking operation is being
/// attempted at each point on the stack.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RegState {
    /// NOPEN and NCLOSE (non-capturing group)
    #[default]
    Nopen = 0,
    /// MOPEN + [0-9] (start of capturing group)
    Mopen = 1,
    /// MCLOSE + [0-9] (end of capturing group)
    Mclose = 2,
    /// ZOPEN + [0-9] (external submatch start)
    Zopen = 3,
    /// ZCLOSE + [0-9] (external submatch end)
    Zclose = 4,
    /// BRANCH (alternation)
    Branch = 5,
    /// BRACE_COMPLEX trying one more match
    BrcplxMore = 6,
    /// BRACE_COMPLEX trying longest match
    BrcplxLong = 7,
    /// BRACE_COMPLEX trying shortest match
    BrcplxShort = 8,
    /// NOMATCH (negative lookahead)
    Nomatch = 9,
    /// BEHIND/NOBEHIND matching rest
    Behind1 = 10,
    /// BEHIND/NOBEHIND matching behind part
    Behind2 = 11,
    /// STAR/PLUS/BRACE_SIMPLE longest match
    StarLong = 12,
    /// STAR/PLUS/BRACE_SIMPLE shortest match
    StarShort = 13,
}

impl From<c_int> for RegState {
    fn from(v: c_int) -> Self {
        match v {
            0 => Self::Nopen,
            1 => Self::Mopen,
            2 => Self::Mclose,
            3 => Self::Zopen,
            4 => Self::Zclose,
            5 => Self::Branch,
            6 => Self::BrcplxMore,
            7 => Self::BrcplxLong,
            8 => Self::BrcplxShort,
            9 => Self::Nomatch,
            10 => Self::Behind1,
            11 => Self::Behind2,
            12 => Self::StarLong,
            13 => Self::StarShort,
            _ => Self::Nopen, // Default for unknown values
        }
    }
}

// =============================================================================
// Line number and column types (match C typedefs)
// =============================================================================

/// Line number type (1-indexed).
pub type LineNr = c_int;

/// Column number type (0-indexed byte offset).
pub type ColNr = c_int;

// =============================================================================
// Position saving structures
// =============================================================================

/// Saved position for single-line matching.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RegSavePos {
    /// Position in input line (pointer into string).
    pub pos: *mut u8,
}

/// Saved position for multi-line matching.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RegSaveMulti {
    /// Current line number.
    pub line: LineNr,
    /// Current column.
    pub col: ColNr,
}

/// Saved match position union.
///
/// Used to save and restore the current match position.
/// The correct variant is chosen based on `rex.reg_multi`.
#[repr(C)]
pub union RegSave {
    /// Single-line position (pointer).
    pub pos: RegSavePos,
    /// Multi-line position (line, col).
    pub multi: RegSaveMulti,
}

impl Default for RegSave {
    fn default() -> Self {
        Self {
            pos: RegSavePos::default(),
        }
    }
}

impl Clone for RegSave {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for RegSave {}

/// Saved submatch position.
///
/// Used to save/restore submatch start/end positions during backtracking.
#[repr(C)]
pub union SaveSe {
    /// Pointer for single-line matching.
    pub ptr: *mut u8,
    /// Line/column for multi-line matching.
    pub pos: RegSaveMulti,
}

impl Default for SaveSe {
    fn default() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }
}

impl Clone for SaveSe {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for SaveSe {}

// =============================================================================
// RegItem - Backtrack stack item
// =============================================================================

/// Union for regitem data storage.
#[repr(C)]
pub union RegItemData {
    /// Saved position for restoring.
    pub regsave: RegSave,
}

impl Default for RegItemData {
    fn default() -> Self {
        Self {
            regsave: RegSave::default(),
        }
    }
}

impl Clone for RegItemData {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for RegItemData {}

/// Backtrack stack item.
///
/// Each item on the regstack represents a point where backtracking might
/// be needed. When a match fails, we pop items and retry alternatives.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegItem {
    /// Type of stack item (what operation was being attempted).
    pub rs_state: c_int,
    /// Current scan position in bytecode.
    pub rs_scan: *mut u8,
    /// Additional data for restoring state.
    pub rs_un: RegItemData,
}

impl Default for RegItem {
    fn default() -> Self {
        Self {
            rs_state: RegState::Nopen as c_int,
            rs_scan: ptr::null_mut(),
            rs_un: RegItemData::default(),
        }
    }
}

impl RegItem {
    /// Create a new stack item.
    pub fn new(state: RegState, scan: *mut u8) -> Self {
        Self {
            rs_state: state as c_int,
            rs_scan: scan,
            rs_un: RegItemData::default(),
        }
    }

    /// Get the state as an enum.
    pub fn state(&self) -> RegState {
        RegState::from(self.rs_state)
    }
}

// =============================================================================
// RegBehind - Look-behind state
// =============================================================================

/// State saved for look-behind matching.
///
/// When processing `\@<=` (positive look-behind) or `\@<!` (negative look-behind),
/// we need to save the current state, match backwards, and restore.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegBehind {
    /// Position after the look-behind pattern.
    pub save_after: RegSave,
    /// Position where look-behind matching started.
    pub save_behind: RegSave,
    /// Whether subexpressions need to be cleared.
    pub save_need_clear_subexpr: c_int,
    /// Saved submatch start positions.
    pub save_start: [SaveSe; NSUBEXP],
    /// Saved submatch end positions.
    pub save_end: [SaveSe; NSUBEXP],
}

impl Default for RegBehind {
    fn default() -> Self {
        Self {
            save_after: RegSave::default(),
            save_behind: RegSave::default(),
            save_need_clear_subexpr: 0,
            save_start: [SaveSe::default(); NSUBEXP],
            save_end: [SaveSe::default(); NSUBEXP],
        }
    }
}

// =============================================================================
// RegStar - Repetition state
// =============================================================================

/// State for repetition operators (`*`, `+`, `{n,m}`).
///
/// Tracks the count of matches and the min/max limits.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RegStar {
    /// Current match count.
    pub count: i64,
    /// Minimum number of matches required.
    pub minval: i64,
    /// Maximum number of matches allowed.
    pub maxval: i64,
}

impl RegStar {
    /// Create a new repetition state.
    pub fn new(minval: i64, maxval: i64) -> Self {
        Self {
            count: 0,
            minval,
            maxval,
        }
    }

    /// Check if minimum matches have been made.
    pub fn has_min(&self) -> bool {
        self.count >= self.minval
    }

    /// Check if maximum matches have been reached.
    pub fn at_max(&self) -> bool {
        self.count >= self.maxval
    }

    /// Increment the count.
    pub fn increment(&mut self) {
        self.count += 1;
    }
}

// =============================================================================
// BackPos - Loop detection
// =============================================================================

/// Position tracking for BACK (loop detection).
///
/// Used to detect when a zero-width match is made in a loop,
/// which would otherwise cause infinite looping.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BackPos {
    /// Scan position where BACK was encountered.
    pub bp_scan: *mut u8,
    /// Input position at that point.
    pub bp_pos: RegSave,
}

impl Default for BackPos {
    fn default() -> Self {
        Self {
            bp_scan: ptr::null_mut(),
            bp_pos: RegSave::default(),
        }
    }
}

// =============================================================================
// RegStack - The backtrack stack
// =============================================================================

/// The backtrack stack for the BT regex engine.
///
/// This is implemented as a growable array that can hold `RegItem`,
/// `RegStar`, and `RegBehind` structures. Items are pushed/popped
/// as the match progresses/backtracks.
pub struct RegStack {
    /// Raw storage for stack items.
    data: Vec<u8>,
    /// Current stack size in bytes.
    len: usize,
    /// Initial capacity (for shrink-to-fit).
    initial_capacity: usize,
}

impl RegStack {
    /// Create a new backtrack stack.
    pub fn new() -> Self {
        let initial = REGSTACK_INITIAL * std::mem::size_of::<RegItem>();
        Self {
            data: Vec::with_capacity(initial),
            len: 0,
            initial_capacity: initial,
        }
    }

    /// Push a RegItem onto the stack.
    ///
    /// Returns a mutable reference to the pushed item, or None if allocation fails.
    pub fn push_item(&mut self, state: RegState, scan: *mut u8) -> Option<&mut RegItem> {
        let size = std::mem::size_of::<RegItem>();
        if self.grow(size) {
            let offset = self.len;
            self.len += size;

            // SAFETY: We just grew the buffer to accommodate this item
            unsafe {
                let ptr = self.data.as_mut_ptr().add(offset) as *mut RegItem;
                *ptr = RegItem::new(state, scan);
                Some(&mut *ptr)
            }
        } else {
            None
        }
    }

    /// Pop a RegItem from the stack.
    ///
    /// Returns the scan pointer from the popped item.
    pub fn pop_item(&mut self) -> Option<*mut u8> {
        let size = std::mem::size_of::<RegItem>();
        if self.len >= size {
            self.len -= size;
            // SAFETY: We just verified there's enough data
            unsafe {
                let ptr = self.data.as_ptr().add(self.len) as *const RegItem;
                Some((*ptr).rs_scan)
            }
        } else {
            None
        }
    }

    /// Get the top RegItem without popping.
    pub fn peek_item(&self) -> Option<&RegItem> {
        let size = std::mem::size_of::<RegItem>();
        if self.len >= size {
            // SAFETY: We just verified there's enough data
            unsafe {
                let ptr = self.data.as_ptr().add(self.len - size) as *const RegItem;
                Some(&*ptr)
            }
        } else {
            None
        }
    }

    /// Push a RegStar before the current position.
    ///
    /// RegStar is pushed before RegItem in the stack layout.
    pub fn push_star(&mut self, star: RegStar) -> bool {
        let size = std::mem::size_of::<RegStar>();
        if self.grow(size) {
            let offset = self.len;
            self.len += size;

            // SAFETY: We just grew the buffer
            unsafe {
                let ptr = self.data.as_mut_ptr().add(offset) as *mut RegStar;
                *ptr = star;
            }
            true
        } else {
            false
        }
    }

    /// Pop a RegStar from the stack.
    pub fn pop_star(&mut self) -> Option<RegStar> {
        let size = std::mem::size_of::<RegStar>();
        if self.len >= size {
            self.len -= size;
            // SAFETY: We just verified there's enough data
            unsafe {
                let ptr = self.data.as_ptr().add(self.len) as *const RegStar;
                Some(*ptr)
            }
        } else {
            None
        }
    }

    /// Push a RegBehind before the current position.
    pub fn push_behind(&mut self, behind: RegBehind) -> bool {
        let size = std::mem::size_of::<RegBehind>();
        if self.grow(size) {
            let offset = self.len;
            self.len += size;

            // SAFETY: We just grew the buffer
            unsafe {
                let ptr = self.data.as_mut_ptr().add(offset) as *mut RegBehind;
                *ptr = behind;
            }
            true
        } else {
            false
        }
    }

    /// Pop a RegBehind from the stack.
    pub fn pop_behind(&mut self) -> Option<RegBehind> {
        let size = std::mem::size_of::<RegBehind>();
        if self.len >= size {
            self.len -= size;
            // SAFETY: We just verified there's enough data
            unsafe {
                let ptr = self.data.as_ptr().add(self.len) as *const RegBehind;
                Some(*ptr)
            }
        } else {
            None
        }
    }

    /// Grow the buffer to accommodate `additional` bytes.
    fn grow(&mut self, additional: usize) -> bool {
        let required = self.len + additional;
        if required > self.data.len() {
            // Grow by factor of 2 or 8 if small
            let new_cap = if self.data.len() < 256 {
                (required * 8).max(256)
            } else {
                required * 2
            };
            self.data.resize(new_cap, 0);
        }
        true
    }

    /// Check if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the current stack size in bytes.
    pub fn len_bytes(&self) -> usize {
        self.len
    }

    /// Clear the stack.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Shrink the stack if it has grown beyond initial capacity.
    pub fn shrink_if_grown(&mut self) {
        if self.data.capacity() > self.initial_capacity * 2 {
            self.data.shrink_to(self.initial_capacity);
        }
    }
}

impl Default for RegStack {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// BackPosTable - BACK position tracking
// =============================================================================

/// Table of BACK positions for loop detection.
pub struct BackPosTable {
    /// Storage for positions.
    data: Vec<BackPos>,
    /// Initial capacity.
    initial_capacity: usize,
}

impl BackPosTable {
    /// Create a new BackPos table.
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(BACKPOS_INITIAL),
            initial_capacity: BACKPOS_INITIAL,
        }
    }

    /// Add a new position entry.
    pub fn push(&mut self, scan: *mut u8, pos: RegSave) {
        self.data.push(BackPos {
            bp_scan: scan,
            bp_pos: pos,
        });
    }

    /// Find an entry by scan position.
    pub fn find(&self, scan: *const u8) -> Option<&BackPos> {
        self.data.iter().find(|bp| std::ptr::eq(bp.bp_scan, scan))
    }

    /// Find a mutable entry by scan position.
    pub fn find_mut(&mut self, scan: *const u8) -> Option<&mut BackPos> {
        self.data
            .iter_mut()
            .find(|bp| std::ptr::eq(bp.bp_scan, scan))
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Shrink if grown beyond initial capacity.
    pub fn shrink_if_grown(&mut self) {
        if self.data.capacity() > self.initial_capacity * 2 {
            self.data.shrink_to(self.initial_capacity);
        }
    }
}

impl Default for BackPosTable {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new backtrack stack.
#[no_mangle]
pub extern "C" fn rs_regstack_new() -> *mut RegStack {
    Box::into_raw(Box::new(RegStack::new()))
}

/// Free a backtrack stack.
///
/// # Safety
/// `stack` must be a valid pointer from `rs_regstack_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_regstack_free(stack: *mut RegStack) {
    if !stack.is_null() {
        drop(Box::from_raw(stack));
    }
}

/// Clear a backtrack stack.
///
/// # Safety
/// `stack` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regstack_clear(stack: *mut RegStack) {
    if !stack.is_null() {
        (*stack).clear();
    }
}

/// Check if the backtrack stack is empty.
///
/// # Safety
/// `stack` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_regstack_is_empty(stack: *const RegStack) -> c_int {
    if stack.is_null() {
        1
    } else {
        c_int::from((*stack).is_empty())
    }
}

/// Create a new backpos table.
#[no_mangle]
pub extern "C" fn rs_backpos_new() -> *mut BackPosTable {
    Box::into_raw(Box::new(BackPosTable::new()))
}

/// Free a backpos table.
///
/// # Safety
/// `table` must be a valid pointer from `rs_backpos_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_backpos_free(table: *mut BackPosTable) {
    if !table.is_null() {
        drop(Box::from_raw(table));
    }
}

/// Clear a backpos table.
///
/// # Safety
/// `table` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_backpos_clear(table: *mut BackPosTable) {
    if !table.is_null() {
        (*table).clear();
    }
}

/// Get the NSUBEXP constant.
#[no_mangle]
pub extern "C" fn rs_bt_nsubexp() -> c_int {
    NSUBEXP as c_int
}

/// Convert a RegState enum to int.
#[no_mangle]
pub extern "C" fn rs_regstate_nopen() -> c_int {
    RegState::Nopen as c_int
}

/// Get RegState::Mopen value.
#[no_mangle]
pub extern "C" fn rs_regstate_mopen() -> c_int {
    RegState::Mopen as c_int
}

/// Get RegState::Mclose value.
#[no_mangle]
pub extern "C" fn rs_regstate_mclose() -> c_int {
    RegState::Mclose as c_int
}

/// Get RegState::Branch value.
#[no_mangle]
pub extern "C" fn rs_regstate_branch() -> c_int {
    RegState::Branch as c_int
}

/// Get RegState::StarLong value.
#[no_mangle]
pub extern "C" fn rs_regstate_star_long() -> c_int {
    RegState::StarLong as c_int
}

/// Get RegState::StarShort value.
#[no_mangle]
pub extern "C" fn rs_regstate_star_short() -> c_int {
    RegState::StarShort as c_int
}

/// Get RegState::Nomatch value.
#[no_mangle]
pub extern "C" fn rs_regstate_nomatch() -> c_int {
    RegState::Nomatch as c_int
}

/// Get RegState::Behind1 value.
#[no_mangle]
pub extern "C" fn rs_regstate_behind1() -> c_int {
    RegState::Behind1 as c_int
}

/// Get RegState::Behind2 value.
#[no_mangle]
pub extern "C" fn rs_regstate_behind2() -> c_int {
    RegState::Behind2 as c_int
}

/// Get the length of the backtrack stack.
///
/// # Safety
/// `stack` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_regstack_len(stack: *const RegStack) -> c_int {
    if stack.is_null() {
        0
    } else {
        (*stack).data.len() as c_int
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regstate_values() {
        assert_eq!(RegState::Nopen as c_int, 0);
        assert_eq!(RegState::Mopen as c_int, 1);
        assert_eq!(RegState::Mclose as c_int, 2);
        assert_eq!(RegState::Branch as c_int, 5);
        assert_eq!(RegState::StarLong as c_int, 12);
        assert_eq!(RegState::StarShort as c_int, 13);
    }

    #[test]
    fn test_regstate_from_int() {
        assert_eq!(RegState::from(0), RegState::Nopen);
        assert_eq!(RegState::from(5), RegState::Branch);
        assert_eq!(RegState::from(12), RegState::StarLong);
        assert_eq!(RegState::from(99), RegState::Nopen); // Unknown -> default
    }

    #[test]
    fn test_regstar() {
        let mut star = RegStar::new(1, 5);
        assert_eq!(star.count, 0);
        assert_eq!(star.minval, 1);
        assert_eq!(star.maxval, 5);
        assert!(!star.has_min());
        assert!(!star.at_max());

        star.increment();
        assert!(star.has_min());
        assert!(!star.at_max());

        star.count = 5;
        assert!(star.at_max());
    }

    #[test]
    fn test_regstack_basic() {
        let mut stack = RegStack::new();
        assert!(stack.is_empty());

        // Push and pop item
        let item = stack.push_item(RegState::Branch, ptr::null_mut());
        assert!(item.is_some());
        assert!(!stack.is_empty());

        let scan = stack.pop_item();
        assert!(scan.is_some());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_regstack_star() {
        let mut stack = RegStack::new();

        let star = RegStar::new(0, 10);
        assert!(stack.push_star(star));
        assert!(!stack.is_empty());

        let popped = stack.pop_star();
        assert!(popped.is_some());
        let popped = popped.unwrap();
        assert_eq!(popped.minval, 0);
        assert_eq!(popped.maxval, 10);
    }

    #[test]
    fn test_backpos_table() {
        let mut table = BackPosTable::new();
        assert!(table.is_empty());

        // Add an entry
        let mut dummy: u8 = 0;
        let scan = &mut dummy as *mut u8;
        table.push(scan, RegSave::default());
        assert_eq!(table.len(), 1);

        // Find it
        let found = table.find(scan);
        assert!(found.is_some());
        assert_eq!(found.unwrap().bp_scan, scan);

        // Clear
        table.clear();
        assert!(table.is_empty());
    }

    #[test]
    fn test_nsubexp_constant() {
        assert_eq!(NSUBEXP, 10);
    }
}
