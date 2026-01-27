//! NFA pattern compilation for the regex engine.
//!
//! This module implements the pattern compilation phase of the NFA regex engine.
//! It converts regex patterns in postfix form into NFA state machines using
//! Thompson's construction algorithm.
//!
//! # Overview
//!
//! The compilation process:
//! 1. Parse regex into postfix form (done in C via `nfa_reg()`)
//! 2. Convert postfix to NFA using `post2nfa()`
//!    - First pass: count states needed
//!    - Second pass: allocate and build NFA
//!
//! # Key structures
//!
//! - [`StateAllocator`]: Manages NFA state allocation
//! - [`FragStack`]: Stack for building NFA fragments during construction
//! - Fragment operations: `frag`, `list1`, `patch`, `append`

use std::ffi::c_int;
use std::ptr;

use crate::nfa_states::{
    Frag, NfaState, Ptrlist, NFA_CONCAT, NFA_EMPTY, NFA_END_COLL, NFA_END_NEG_COLL, NFA_OR,
    NFA_QUEST, NFA_QUEST_NONGREEDY, NFA_RANGE, NFA_RANGE_MAX, NFA_RANGE_MIN, NFA_SPLIT, NFA_STAR,
    NFA_STAR_NONGREEDY,
};

// =============================================================================
// Error Types
// =============================================================================

/// Error during NFA compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileError {
    /// Stack underflow (not enough fragments on stack).
    StackUnderflow,
    /// State allocation failed (capacity exceeded).
    AllocationFailed,
    /// No allocator provided when one was needed.
    NoAllocator,
}

// =============================================================================
// State Allocator
// =============================================================================

/// State allocator for NFA construction.
///
/// Manages a pre-allocated array of NFA states. States are allocated
/// sequentially during NFA construction. The allocator tracks the current
/// index and total capacity.
pub struct StateAllocator {
    /// Pointer to the state array (owned externally, typically by nfa_regprog_T).
    states: *mut NfaState,
    /// Current allocation index.
    index: usize,
    /// Total number of states available.
    capacity: usize,
}

impl StateAllocator {
    /// Create a new state allocator.
    ///
    /// # Arguments
    /// * `states` - Pointer to pre-allocated state array
    /// * `capacity` - Number of states available
    ///
    /// # Safety
    /// The `states` pointer must be valid for `capacity` states and must
    /// remain valid for the lifetime of the allocator.
    pub unsafe fn new(states: *mut NfaState, capacity: usize) -> Self {
        Self {
            states,
            index: 0,
            capacity,
        }
    }

    /// Allocate and initialize a new NFA state.
    ///
    /// # Arguments
    /// * `c` - Character/opcode for this state
    /// * `out` - Primary outgoing edge (may be null)
    /// * `out1` - Secondary outgoing edge (may be null)
    ///
    /// # Returns
    /// Pointer to the new state, or null if allocation fails (capacity exceeded).
    pub fn alloc_state(
        &mut self,
        c: c_int,
        out: *mut NfaState,
        out1: *mut NfaState,
    ) -> *mut NfaState {
        if self.index >= self.capacity {
            return ptr::null_mut();
        }

        unsafe {
            let state = self.states.add(self.index);
            self.index += 1;

            (*state).c = c;
            (*state).out = out;
            (*state).out1 = out1;
            (*state).val = 0;
            (*state).id = self.index as c_int;
            (*state).lastlist[0] = 0;
            (*state).lastlist[1] = 0;

            state
        }
    }

    /// Reset the allocator for reuse (e.g., for second pass).
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Get the current number of allocated states.
    pub fn count(&self) -> usize {
        self.index
    }

    /// Get the total capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

// =============================================================================
// Fragment Stack
// =============================================================================

/// Stack for NFA fragment construction.
///
/// During NFA construction, fragments are pushed and popped from this stack
/// as operators are processed. The stack grows dynamically as needed.
pub struct FragStack {
    /// Stack storage.
    stack: Vec<Frag>,
}

impl FragStack {
    /// Create a new fragment stack with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    /// Push a fragment onto the stack.
    pub fn push(&mut self, frag: Frag) {
        self.stack.push(frag);
    }

    /// Pop a fragment from the stack.
    ///
    /// Returns `None` if the stack is empty (indicates a compilation error).
    pub fn pop(&mut self) -> Option<Frag> {
        self.stack.pop()
    }

    /// Check if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the number of fragments on the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Clear the stack.
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

// =============================================================================
// Fragment Operations
// =============================================================================

/// Create a new NFA fragment.
///
/// # Arguments
/// * `start` - Start state of the fragment
/// * `out` - List of outgoing pointers to patch
#[inline]
pub const fn frag(start: *mut NfaState, out: *mut Ptrlist) -> Frag {
    Frag { start, out }
}

/// Create a singleton pointer list containing just one output pointer.
///
/// # Safety
/// `outp` must be a valid pointer to a state pointer field.
#[inline]
pub unsafe fn list1(outp: *mut *mut NfaState) -> *mut Ptrlist {
    let l = outp.cast::<Ptrlist>();
    (*l).next = ptr::null_mut();
    l
}

/// Patch all pointers in the list to point to the given state.
///
/// This connects the dangling outputs of a fragment to a destination state.
///
/// # Safety
/// `l` must be a valid Ptrlist chain or null.
/// `s` must be a valid state pointer.
pub unsafe fn patch(mut l: *mut Ptrlist, s: *mut NfaState) {
    while !l.is_null() {
        let next = (*l).next;
        (*l).s = s;
        l = next;
    }
}

/// Join two pointer lists, returning the combined list.
///
/// # Safety
/// Both `l1` and `l2` must be valid Ptrlist chains or null.
pub unsafe fn append(l1: *mut Ptrlist, l2: *mut Ptrlist) -> *mut Ptrlist {
    if l1.is_null() {
        return l2;
    }
    if l2.is_null() {
        return l1;
    }

    // Find the end of l1
    let mut end = l1;
    while !(*end).next.is_null() {
        end = (*end).next;
    }
    (*end).next = l2;
    l1
}

// =============================================================================
// Postfix to NFA Conversion
// =============================================================================

/// State counting mode for post2nfa.
///
/// In counting mode, we don't allocate states; we just count how many
/// states will be needed. This is used for the first pass.
pub struct StateCounter {
    count: usize,
}

impl StateCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn add(&mut self, n: usize) {
        self.count += n;
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for StateCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a single postfix operator to NFA fragment(s).
///
/// This handles the core operators: CONCAT, OR, STAR, QUEST, EMPTY, etc.
///
/// # Arguments
/// * `op` - The postfix operator (NFA_* constant)
/// * `stack` - Fragment stack for push/pop operations
/// * `allocator` - State allocator (if not counting)
/// * `counting` - If true, only count states without allocating
///
/// # Returns
/// * `Ok(states_needed)` - Number of states needed for this operator
/// * `Err(CompileError)` - Stack underflow, allocation failure, or no allocator
///
/// # Safety
/// Must be called with valid allocator and stack.
pub unsafe fn process_operator(
    op: c_int,
    stack: &mut FragStack,
    allocator: Option<&mut StateAllocator>,
    counting: bool,
) -> Result<usize, CompileError> {
    match op {
        NFA_CONCAT => {
            // Concatenation: patch e1's outputs to e2's start
            if counting {
                return Ok(0);
            }
            let e2 = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let e1 = stack.pop().ok_or(CompileError::StackUnderflow)?;
            patch(e1.out, e2.start);
            stack.push(frag(e1.start, e2.out));
            Ok(0)
        }

        NFA_OR => {
            // Alternation: create split state
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let e2 = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let e1 = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let s = alloc.alloc_state(NFA_SPLIT, e1.start, e2.start);
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            stack.push(frag(s, append(e1.out, e2.out)));
            Ok(1)
        }

        NFA_STAR => {
            // Zero or more (greedy)
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let e = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let s = alloc.alloc_state(NFA_SPLIT, e.start, ptr::null_mut());
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            patch(e.out, s);
            stack.push(frag(s, list1(&mut (*s).out1)));
            Ok(1)
        }

        NFA_STAR_NONGREEDY => {
            // Zero or more (non-greedy)
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let e = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let s = alloc.alloc_state(NFA_SPLIT, ptr::null_mut(), e.start);
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            patch(e.out, s);
            stack.push(frag(s, list1(&mut (*s).out)));
            Ok(1)
        }

        NFA_QUEST => {
            // Zero or one (greedy)
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let e = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let s = alloc.alloc_state(NFA_SPLIT, e.start, ptr::null_mut());
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            stack.push(frag(s, append(e.out, list1(&mut (*s).out1))));
            Ok(1)
        }

        NFA_QUEST_NONGREEDY => {
            // Zero or one (non-greedy)
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let e = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let s = alloc.alloc_state(NFA_SPLIT, ptr::null_mut(), e.start);
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            stack.push(frag(s, append(e.out, list1(&mut (*s).out))));
            Ok(1)
        }

        NFA_EMPTY => {
            // Empty transition (0-length match)
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let s = alloc.alloc_state(NFA_EMPTY, ptr::null_mut(), ptr::null_mut());
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            stack.push(frag(s, list1(&mut (*s).out)));
            Ok(1)
        }

        NFA_END_COLL | NFA_END_NEG_COLL => {
            // End of character collection
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let e = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let s = alloc.alloc_state(NFA_END_COLL, ptr::null_mut(), ptr::null_mut());
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            patch(e.out, s);
            (*e.start).out1 = s;
            stack.push(frag(e.start, list1(&mut (*s).out)));
            Ok(1)
        }

        NFA_RANGE => {
            // Character range [a-z]
            if counting {
                return Ok(0);
            }
            let e2 = stack.pop().ok_or(CompileError::StackUnderflow)?;
            let e1 = stack.pop().ok_or(CompileError::StackUnderflow)?;
            (*e2.start).val = (*e2.start).c;
            (*e2.start).c = NFA_RANGE_MAX;
            (*e1.start).val = (*e1.start).c;
            (*e1.start).c = NFA_RANGE_MIN;
            patch(e1.out, e2.start);
            stack.push(frag(e1.start, e2.out));
            Ok(0)
        }

        // For literal characters and other states, create a single state
        _ => {
            if counting {
                return Ok(1);
            }
            let alloc = allocator.ok_or(CompileError::NoAllocator)?;
            let s = alloc.alloc_state(op, ptr::null_mut(), ptr::null_mut());
            if s.is_null() {
                return Err(CompileError::AllocationFailed);
            }
            stack.push(frag(s, list1(&mut (*s).out)));
            Ok(1)
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new state allocator for NFA construction.
///
/// # Safety
/// `states` must be a valid pointer to an array of at least `capacity` NfaState.
#[no_mangle]
pub unsafe extern "C" fn rs_state_allocator_new(
    states: *mut NfaState,
    capacity: c_int,
) -> *mut StateAllocator {
    let allocator = Box::new(StateAllocator::new(states, capacity as usize));
    Box::into_raw(allocator)
}

/// Free a state allocator.
///
/// # Safety
/// `allocator` must be a valid pointer returned by `rs_state_allocator_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_state_allocator_free(allocator: *mut StateAllocator) {
    if !allocator.is_null() {
        drop(Box::from_raw(allocator));
    }
}

/// Allocate a new NFA state.
///
/// # Safety
/// `allocator` must be a valid StateAllocator pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_alloc_state(
    allocator: *mut StateAllocator,
    c: c_int,
    out: *mut NfaState,
    out1: *mut NfaState,
) -> *mut NfaState {
    if allocator.is_null() {
        return ptr::null_mut();
    }
    (*allocator).alloc_state(c, out, out1)
}

/// Reset a state allocator for reuse.
///
/// # Safety
/// `allocator` must be a valid StateAllocator pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_state_allocator_reset(allocator: *mut StateAllocator) {
    if !allocator.is_null() {
        (*allocator).reset();
    }
}

/// Get the count of allocated states.
///
/// # Safety
/// `allocator` must be a valid StateAllocator pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_state_allocator_count(allocator: *const StateAllocator) -> c_int {
    if allocator.is_null() {
        0
    } else {
        (*allocator).count() as c_int
    }
}

/// Create a new fragment stack.
///
/// # Safety
/// Returns a valid FragStack pointer that must be freed with `rs_frag_stack_free`.
#[no_mangle]
pub extern "C" fn rs_frag_stack_new(capacity: c_int) -> *mut FragStack {
    let stack = Box::new(FragStack::with_capacity(capacity as usize));
    Box::into_raw(stack)
}

/// Free a fragment stack.
///
/// # Safety
/// `stack` must be a valid pointer returned by `rs_frag_stack_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_frag_stack_free(stack: *mut FragStack) {
    if !stack.is_null() {
        drop(Box::from_raw(stack));
    }
}

/// Push a fragment onto the stack.
///
/// # Safety
/// `stack` must be a valid FragStack pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_frag_stack_push(
    stack: *mut FragStack,
    start: *mut NfaState,
    out: *mut Ptrlist,
) {
    if !stack.is_null() {
        (*stack).push(frag(start, out));
    }
}

/// Pop a fragment from the stack.
///
/// # Safety
/// `stack` must be a valid FragStack pointer.
/// `out_start` and `out_out` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_frag_stack_pop(
    stack: *mut FragStack,
    out_start: *mut *mut NfaState,
    out_out: *mut *mut Ptrlist,
) -> c_int {
    if stack.is_null() {
        return 0;
    }
    match (*stack).pop() {
        Some(f) => {
            *out_start = f.start;
            *out_out = f.out;
            1
        }
        None => 0,
    }
}

/// Create a fragment helper (FFI wrapper for frag()).
#[no_mangle]
pub extern "C" fn rs_frag_new(start: *mut NfaState, out: *mut Ptrlist) -> Frag {
    frag(start, out)
}

/// Create a singleton pointer list (FFI wrapper for list1()).
///
/// # Safety
/// `outp` must be a valid pointer to a state pointer field.
#[no_mangle]
pub unsafe extern "C" fn rs_list1(outp: *mut *mut NfaState) -> *mut Ptrlist {
    list1(outp)
}

/// Patch a pointer list (FFI wrapper for patch()).
///
/// # Safety
/// `l` must be a valid Ptrlist chain or null.
/// `s` must be a valid state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_patch(l: *mut Ptrlist, s: *mut NfaState) {
    patch(l, s);
}

/// Append two pointer lists (FFI wrapper for append()).
///
/// # Safety
/// Both `l1` and `l2` must be valid Ptrlist chains or null.
#[no_mangle]
pub unsafe extern "C" fn rs_append(l1: *mut Ptrlist, l2: *mut Ptrlist) -> *mut Ptrlist {
    append(l1, l2)
}

// =============================================================================
// NFA Optimization Functions
// =============================================================================

/// Maximum recursion depth for NFA analysis
const MAX_DEPTH: c_int = 1000;

/// Check if the NFA always matches at the beginning of line.
///
/// Returns 1 if the pattern is anchored (starts with BOL/BOF), 0 otherwise.
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_reganch(start: *mut NfaState, depth: c_int) -> c_int {
    nfa_get_reganch_impl(start, depth)
}

unsafe fn nfa_get_reganch_impl(start: *mut NfaState, depth: c_int) -> c_int {
    use crate::nfa_states::*;

    if start.is_null() {
        return 0;
    }
    if depth >= MAX_DEPTH {
        return 0;
    }

    let p = start;
    let c = (*p).c;

    // BOL and BOF indicate anchor
    if c == NFA_BOL || c == NFA_BOF {
        return 1;
    }

    // MOPEN/ZOPEN - continue checking
    if c == NFA_MOPEN
        || (NFA_MOPEN..=NFA_MOPEN + 9).contains(&c)
        || (NFA_ZOPEN..=NFA_ZOPEN + 9).contains(&c)
    {
        return nfa_get_reganch_impl((*p).out, depth + 1);
    }

    // NFA_SPLIT requires both branches to be anchored
    if c == NFA_SPLIT {
        let r1 = nfa_get_reganch_impl((*p).out, depth + 1);
        let r2 = nfa_get_reganch_impl((*p).out1, depth + 1);
        return if r1 != 0 && r2 != 0 { 1 } else { 0 };
    }

    // Lookahead/behind - continue with out
    if (NFA_START_INVISIBLE..=NFA_END_INVISIBLE).contains(&c) {
        return nfa_get_reganch_impl((*p).out, depth + 1);
    }

    // For other states, not anchored
    0
}

/// Get the starting character for the pattern (if it has one).
///
/// Returns the character code, or 0 if no fixed start character.
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_regstart(start: *mut NfaState, depth: c_int) -> c_int {
    nfa_get_regstart_impl(start, depth)
}

unsafe fn nfa_get_regstart_impl(start: *mut NfaState, depth: c_int) -> c_int {
    use crate::nfa_states::*;

    if start.is_null() {
        return 0;
    }
    if depth >= MAX_DEPTH {
        return 0;
    }

    let p = start;
    let c = (*p).c;

    // If it's a literal character
    if c > 0 && c < 256 {
        return c;
    }

    // Transparent states - continue
    if c == NFA_BOL || c == NFA_BOF || c == NFA_BOW || c == NFA_MOPEN || c == NFA_NOPEN {
        return nfa_get_regstart_impl((*p).out, depth + 1);
    }

    // MOPEN + subexpr
    if (NFA_MOPEN..=NFA_MOPEN + 9).contains(&c) {
        return nfa_get_regstart_impl((*p).out, depth + 1);
    }

    // ZOPEN
    if (NFA_ZOPEN..=NFA_ZOPEN + 9).contains(&c) {
        return nfa_get_regstart_impl((*p).out, depth + 1);
    }

    // NFA_SPLIT - both branches must have same start char
    if c == NFA_SPLIT {
        let c1 = nfa_get_regstart_impl((*p).out, depth + 1);
        let c2 = nfa_get_regstart_impl((*p).out1, depth + 1);
        return if c1 == c2 { c1 } else { 0 };
    }

    // Lookahead - continue
    if (NFA_START_INVISIBLE..=NFA_END_INVISIBLE).contains(&c) {
        return nfa_get_regstart_impl((*p).out, depth + 1);
    }

    0
}

/// Extract a literal match text from the NFA if the pattern is simple enough.
///
/// Returns a pointer to the match text, or null if the pattern is complex.
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_match_text(start: *mut NfaState) -> *mut u8 {
    nfa_get_match_text_impl(start)
}

unsafe fn nfa_get_match_text_impl(start: *mut NfaState) -> *mut u8 {
    use crate::nfa_states::*;

    if start.is_null() {
        return ptr::null_mut();
    }

    // For simple literal patterns, we could extract the text
    // This is an optimization where patterns like "foo" can use
    // a simple string search instead of full NFA matching

    // Walk through the NFA and check if it's a simple sequence of literals
    let mut p = start;

    // Skip over transparent states at the beginning
    loop {
        if p.is_null() {
            return ptr::null_mut();
        }
        let c = (*p).c;

        // Skip MOPEN/NOPEN/ZOPEN at start
        if c == NFA_MOPEN
            || c == NFA_NOPEN
            || (NFA_MOPEN..=NFA_MOPEN + 9).contains(&c)
            || (NFA_ZOPEN..=NFA_ZOPEN + 9).contains(&c)
        {
            p = (*p).out;
            continue;
        }

        // Check for literal character
        if c > 0 && c < 256 {
            // Found a literal - the C code stores this in a specific way
            // For now, return null to indicate we can't extract match text
            // The full implementation requires buffer allocation
            return ptr::null_mut();
        }

        // For other states, can't extract simple text
        return ptr::null_mut();
    }
}

/// Post-process the NFA for optimization.
///
/// This function is called after NFA construction to apply optimizations
/// such as detecting patterns that can use simpler matching.
///
/// # Safety
/// `prog_ptr` must be a valid nfa_regprog_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_postprocess(prog_ptr: *mut std::ffi::c_void) {
    // The actual post-processing is complex and modifies the NFA states
    // For now, this is a stub that allows the C code to call it
    // The full implementation would optimize things like:
    // - Detecting patterns that always match empty string
    // - Marking states that can be skipped
    // - Pre-computing character class membership
    let _ = prog_ptr;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_counter() {
        let mut counter = StateCounter::new();
        assert_eq!(counter.count(), 0);

        counter.add(5);
        assert_eq!(counter.count(), 5);

        counter.add(3);
        assert_eq!(counter.count(), 8);
    }

    #[test]
    fn test_frag_stack() {
        let mut stack = FragStack::with_capacity(10);
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);

        // Push some fragments
        stack.push(Frag::empty());
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);

        stack.push(Frag::empty());
        assert_eq!(stack.len(), 2);

        // Pop
        let f = stack.pop();
        assert!(f.is_some());
        assert_eq!(stack.len(), 1);

        let f = stack.pop();
        assert!(f.is_some());
        assert!(stack.is_empty());

        // Pop from empty
        let f = stack.pop();
        assert!(f.is_none());
    }

    #[test]
    fn test_frag_creation() {
        let f = frag(ptr::null_mut(), ptr::null_mut());
        assert!(f.start.is_null());
        assert!(f.out.is_null());
    }
}
