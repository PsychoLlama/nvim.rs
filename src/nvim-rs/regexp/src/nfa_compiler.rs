//! NFA Compilation Engine
//!
//! This module provides the complete NFA compilation pipeline that converts
//! regex patterns into executable NFA state machines. It consolidates:
//!
//! - Pattern parsing (pattern to postfix conversion)
//! - NFA construction (Thompson's construction algorithm)
//! - Post-processing optimization
//!
//! # Compilation Flow
//!
//! ```text
//! Pattern String
//!       |
//!       v
//!   nfa_regcomp_start() - Initialize compilation state
//!       |
//!       v
//!   re2post() - Parse pattern to postfix notation
//!       |
//!       v
//!   post2nfa() - Convert postfix to NFA (two passes)
//!       |
//!       v
//!   nfa_postprocess() - Apply optimizations
//!       |
//!       v
//!   NFA Program (nfa_regprog_T)
//! ```

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::nfa_compile::{append, frag, list1, patch, FragStack};
use crate::nfa_states::{
    Frag, NfaState, NFA_ALPHA, NFA_ANY, NFA_ANY_COMPOSING, NFA_BACKREF1, NFA_BACKREF9, NFA_BOF,
    NFA_BOL, NFA_BOW, NFA_CLASS_ALNUM, NFA_CLASS_ALPHA, NFA_CLASS_BACKSPACE, NFA_CLASS_BLANK,
    NFA_CLASS_CNTRL, NFA_CLASS_DIGIT, NFA_CLASS_ESCAPE, NFA_CLASS_FNAME, NFA_CLASS_GRAPH,
    NFA_CLASS_IDENT, NFA_CLASS_KEYWORD, NFA_CLASS_LOWER, NFA_CLASS_PRINT, NFA_CLASS_PUNCT,
    NFA_CLASS_RETURN, NFA_CLASS_SPACE, NFA_CLASS_TAB, NFA_CLASS_UPPER, NFA_CLASS_XDIGIT, NFA_COL,
    NFA_COL_GT, NFA_COL_LT, NFA_COMPOSING, NFA_CONCAT, NFA_CURSOR, NFA_DIGIT, NFA_EMPTY,
    NFA_END_COLL, NFA_END_COMPOSING, NFA_END_INVISIBLE, NFA_END_INVISIBLE_NEG, NFA_END_NEG_COLL,
    NFA_END_PATTERN, NFA_EOF, NFA_EOL, NFA_EOW, NFA_FNAME, NFA_HEAD, NFA_HEX, NFA_IDENT, NFA_KWORD,
    NFA_LNUM, NFA_LNUM_GT, NFA_LNUM_LT, NFA_LOWER, NFA_LOWER_IC, NFA_MARK, NFA_MARK_GT,
    NFA_MARK_LT, NFA_MATCH, NFA_MCLOSE, NFA_MCLOSE9, NFA_MOPEN, NFA_MOPEN1, NFA_MOPEN9, NFA_NALPHA,
    NFA_NCLOSE, NFA_NDIGIT, NFA_NEWL, NFA_NHEAD, NFA_NHEX, NFA_NLOWER, NFA_NLOWER_IC, NFA_NOCTAL,
    NFA_NOPEN, NFA_NUPPER, NFA_NUPPER_IC, NFA_NWHITE, NFA_NWORD, NFA_OCTAL, NFA_OPT_CHARS, NFA_OR,
    NFA_PREV_ATOM_JUST_BEFORE, NFA_PREV_ATOM_JUST_BEFORE_NEG, NFA_PREV_ATOM_LIKE_PATTERN,
    NFA_PREV_ATOM_NO_WIDTH, NFA_PREV_ATOM_NO_WIDTH_NEG, NFA_PRINT, NFA_QUEST, NFA_QUEST_NONGREEDY,
    NFA_RANGE, NFA_RANGE_MAX, NFA_RANGE_MIN, NFA_SFNAME, NFA_SIDENT, NFA_SKIP, NFA_SKWORD,
    NFA_SPLIT, NFA_SPRINT, NFA_STAR, NFA_START_COLL, NFA_START_INVISIBLE,
    NFA_START_INVISIBLE_BEFORE, NFA_START_INVISIBLE_BEFORE_FIRST, NFA_START_INVISIBLE_BEFORE_NEG,
    NFA_START_INVISIBLE_BEFORE_NEG_FIRST, NFA_START_INVISIBLE_FIRST, NFA_START_INVISIBLE_NEG,
    NFA_START_INVISIBLE_NEG_FIRST, NFA_START_NEG_COLL, NFA_START_PATTERN, NFA_STAR_NONGREEDY,
    NFA_UPPER, NFA_UPPER_IC, NFA_VCOL, NFA_VCOL_GT, NFA_VCOL_LT, NFA_VISUAL, NFA_WHITE, NFA_WORD,
    NFA_ZCLOSE, NFA_ZCLOSE9, NFA_ZEND, NFA_ZOPEN, NFA_ZOPEN9, NFA_ZREF1, NFA_ZREF9, NFA_ZSTART,
    NSUBEXP,
};

// =============================================================================
// Constants
// =============================================================================

/// Maximum recursion depth for NFA analysis
const MAX_DEPTH: c_int = 4;

/// Maximum bytes in a multibyte character
const MB_MAXBYTES: c_int = 6;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Memory allocation
    fn xmalloc(size: usize) -> *mut c_char;

    // Error reporting
    fn emsg(msg: *const c_char);

    // UTF-8
    fn utf_char2len(c: c_int) -> c_int;
}

// =============================================================================
// Error Messages
// =============================================================================

const E_NFA_STACK_ERROR: &[u8] = b"E874: (NFA) Could not pop the stack!\0";
const E_NFA_TOO_MANY_STATES: &[u8] =
    b"E875: (NFA regexp) (While converting from postfix to NFA), too many states left on stack\0";
const E_NFA_NOT_ENOUGH_SPACE: &[u8] =
    b"E876: (NFA regexp) Not enough space to store the whole NFA\0";

// =============================================================================
// NFA Construction - post2nfa
// =============================================================================

/// Convert postfix form to NFA using Thompson's construction.
///
/// This function implements the core Thompson's construction algorithm.
/// It processes the postfix representation and builds an NFA state machine.
///
/// # Arguments
/// * `postfix` - Pointer to postfix buffer start
/// * `end` - Pointer to end of postfix data
/// * `nfa_calc_size` - If true, only count states without building NFA
/// * `state_ptr` - Pointer to state array (ignored if counting)
/// * `nstate` - Total states available (ignored if counting)
///
/// # Returns
/// Start state of NFA, or null on error. If counting, returns a non-null
/// value on success (the actual pointer is meaningless).
///
/// # Safety
/// All pointers must be valid. `state_ptr` must point to an array of at
/// least `nstate` NfaState structs.
pub unsafe fn post2nfa(
    postfix: *const c_int,
    end: *const c_int,
    nfa_calc_size: bool,
    state_ptr: *mut NfaState,
    nstate: c_int,
    out_nstate: *mut c_int,
) -> *mut NfaState {
    if postfix.is_null() {
        return ptr::null_mut();
    }

    let mut istate: c_int = 0;
    let mut stack = FragStack::with_capacity(if nfa_calc_size {
        100
    } else {
        nstate as usize + 1
    });

    // Macro-like helpers for stack operations
    let push = |stack: &mut FragStack, f: Frag| {
        stack.push(f);
    };

    let mut p = postfix;
    while p < end {
        let op = *p;
        p = p.add(1);

        match op {
            NFA_CONCAT => {
                // Concatenation: patch e1's outputs to e2's start
                if nfa_calc_size {
                    // No new states needed
                    continue;
                }
                let e2 = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let e1 = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                patch(e1.out, e2.start);
                push(&mut stack, frag(e1.start, e2.out));
            }

            NFA_OR => {
                // Alternation: create SPLIT state
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let e2 = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let e1 = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_SPLIT,
                    e1.start,
                    e2.start,
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                push(&mut stack, frag(s, append(e1.out, e2.out)));
            }

            NFA_STAR => {
                // Zero or more (greedy)
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let e = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_SPLIT,
                    e.start,
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                patch(e.out, s);
                push(&mut stack, frag(s, list1(&mut (*s).out1)));
            }

            NFA_STAR_NONGREEDY => {
                // Zero or more (non-greedy)
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let e = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_SPLIT,
                    ptr::null_mut(),
                    e.start,
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                patch(e.out, s);
                push(&mut stack, frag(s, list1(&mut (*s).out)));
            }

            NFA_QUEST => {
                // Zero or one (greedy)
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let e = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_SPLIT,
                    e.start,
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                push(&mut stack, frag(s, append(e.out, list1(&mut (*s).out1))));
            }

            NFA_QUEST_NONGREEDY => {
                // Zero or one (non-greedy)
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let e = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_SPLIT,
                    ptr::null_mut(),
                    e.start,
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                push(&mut stack, frag(s, append(e.out, list1(&mut (*s).out))));
            }

            NFA_END_COLL | NFA_END_NEG_COLL => {
                // End of character collection
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let e = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_END_COLL,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                patch(e.out, s);
                (*e.start).out1 = s;
                push(&mut stack, frag(e.start, list1(&mut (*s).out)));
            }

            NFA_RANGE => {
                // Character range
                if nfa_calc_size {
                    continue;
                }
                let e2 = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                let e1 = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };
                (*e2.start).val = (*e2.start).c;
                (*e2.start).c = NFA_RANGE_MAX;
                (*e1.start).val = (*e1.start).c;
                (*e1.start).c = NFA_RANGE_MIN;
                patch(e1.out, e2.start);
                push(&mut stack, frag(e1.start, e2.out));
            }

            NFA_EMPTY => {
                // Empty transition
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_EMPTY,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                push(&mut stack, frag(s, list1(&mut (*s).out)));
            }

            NFA_OPT_CHARS => {
                // Optional characters \%[abc]
                let n = *p;
                p = p.add(1);
                if nfa_calc_size {
                    istate += n;
                    continue;
                }
                let mut s: *mut NfaState = ptr::null_mut();
                let mut e1 = Frag::empty();
                e1.out = ptr::null_mut();
                let mut s1: *mut NfaState = ptr::null_mut();
                for _ in 0..n {
                    let e = match stack.pop() {
                        Some(f) => f,
                        None => {
                            emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                            return ptr::null_mut();
                        }
                    };
                    s = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        NFA_SPLIT,
                        e.start,
                        ptr::null_mut(),
                    );
                    if s.is_null() {
                        return ptr::null_mut();
                    }
                    if e1.out.is_null() {
                        e1 = e;
                    }
                    patch(e.out, s1);
                    e1.out = append(e1.out, list1(&mut (*s).out1));
                    s1 = s;
                }
                push(&mut stack, frag(s, e1.out));
            }

            NFA_PREV_ATOM_NO_WIDTH
            | NFA_PREV_ATOM_NO_WIDTH_NEG
            | NFA_PREV_ATOM_JUST_BEFORE
            | NFA_PREV_ATOM_JUST_BEFORE_NEG
            | NFA_PREV_ATOM_LIKE_PATTERN => {
                // Lookahead/lookbehind assertions
                let before = op == NFA_PREV_ATOM_JUST_BEFORE || op == NFA_PREV_ATOM_JUST_BEFORE_NEG;
                let pattern = op == NFA_PREV_ATOM_LIKE_PATTERN;

                let (start_state, end_state) = match op {
                    NFA_PREV_ATOM_NO_WIDTH => (NFA_START_INVISIBLE, NFA_END_INVISIBLE),
                    NFA_PREV_ATOM_NO_WIDTH_NEG => (NFA_START_INVISIBLE_NEG, NFA_END_INVISIBLE_NEG),
                    NFA_PREV_ATOM_JUST_BEFORE => (NFA_START_INVISIBLE_BEFORE, NFA_END_INVISIBLE),
                    NFA_PREV_ATOM_JUST_BEFORE_NEG => {
                        (NFA_START_INVISIBLE_BEFORE_NEG, NFA_END_INVISIBLE_NEG)
                    }
                    NFA_PREV_ATOM_LIKE_PATTERN => (NFA_START_PATTERN, NFA_END_PATTERN),
                    _ => unreachable!(),
                };

                let n = if before {
                    let val = *p;
                    p = p.add(1);
                    val
                } else {
                    0
                };

                if nfa_calc_size {
                    istate += if pattern { 4 } else { 2 };
                    continue;
                }

                let e = match stack.pop() {
                    Some(f) => f,
                    None => {
                        emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                        return ptr::null_mut();
                    }
                };

                let s1 = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    end_state,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s1.is_null() {
                    return ptr::null_mut();
                }

                let s = alloc_state(state_ptr, &mut istate, nstate, start_state, e.start, s1);
                if s.is_null() {
                    return ptr::null_mut();
                }

                if pattern {
                    let skip = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        NFA_SKIP,
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );
                    if skip.is_null() {
                        return ptr::null_mut();
                    }
                    let zend = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        NFA_ZEND,
                        s1,
                        ptr::null_mut(),
                    );
                    if zend.is_null() {
                        return ptr::null_mut();
                    }
                    (*s1).out = skip;
                    patch(e.out, zend);
                    push(&mut stack, frag(s, list1(&mut (*skip).out)));
                } else {
                    patch(e.out, s1);
                    push(&mut stack, frag(s, list1(&mut (*s1).out)));
                    if before {
                        (*s).val = n;
                    }
                }
            }

            NFA_COMPOSING | NFA_MOPEN..=NFA_MOPEN9 | NFA_ZOPEN..=NFA_ZOPEN9 | NFA_NOPEN => {
                // Subexpression open
                if nfa_calc_size {
                    istate += 2;
                    continue;
                }

                let mopen = op;
                let mclose = match mopen {
                    NFA_NOPEN => NFA_NCLOSE,
                    NFA_COMPOSING => NFA_END_COMPOSING,
                    _ if (NFA_ZOPEN..=NFA_ZOPEN9).contains(&mopen) => {
                        // NFA_ZOPEN + n -> NFA_ZCLOSE + n
                        NFA_ZCLOSE + (mopen - NFA_ZOPEN)
                    }
                    _ => {
                        // NFA_MOPEN + n -> NFA_MCLOSE + n
                        mopen + NSUBEXP as c_int
                    }
                };

                // Allow empty groups
                if stack.is_empty() {
                    let s = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        mopen,
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );
                    if s.is_null() {
                        return ptr::null_mut();
                    }
                    let s1 = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        mclose,
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );
                    if s1.is_null() {
                        return ptr::null_mut();
                    }
                    patch(list1(&mut (*s).out), s1);
                    push(&mut stack, frag(s, list1(&mut (*s1).out)));
                } else {
                    let e = match stack.pop() {
                        Some(f) => f,
                        None => {
                            emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
                            return ptr::null_mut();
                        }
                    };
                    let s = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        mopen,
                        e.start,
                        ptr::null_mut(),
                    );
                    if s.is_null() {
                        return ptr::null_mut();
                    }
                    let s1 = alloc_state(
                        state_ptr,
                        &mut istate,
                        nstate,
                        mclose,
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );
                    if s1.is_null() {
                        return ptr::null_mut();
                    }
                    patch(e.out, s1);
                    if mopen == NFA_COMPOSING {
                        patch(list1(&mut (*s).out1), s1);
                    }
                    push(&mut stack, frag(s, list1(&mut (*s1).out)));
                }
            }

            NFA_BACKREF1..=NFA_BACKREF9 | NFA_ZREF1..=NFA_ZREF9 => {
                // Backreference
                if nfa_calc_size {
                    istate += 2;
                    continue;
                }
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    op,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                let s1 = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    NFA_SKIP,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s1.is_null() {
                    return ptr::null_mut();
                }
                patch(list1(&mut (*s).out), s1);
                push(&mut stack, frag(s, list1(&mut (*s1).out)));
            }

            NFA_LNUM | NFA_LNUM_GT | NFA_LNUM_LT | NFA_VCOL | NFA_VCOL_GT | NFA_VCOL_LT
            | NFA_COL | NFA_COL_GT | NFA_COL_LT | NFA_MARK | NFA_MARK_GT | NFA_MARK_LT => {
                // Position match with value
                let n = *p;
                p = p.add(1);
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    op,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                (*s).val = n;
                push(&mut stack, frag(s, list1(&mut (*s).out)));
            }

            // Default: operands (literal characters, anchors, etc.)
            _ => {
                if nfa_calc_size {
                    istate += 1;
                    continue;
                }
                let s = alloc_state(
                    state_ptr,
                    &mut istate,
                    nstate,
                    op,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                if s.is_null() {
                    return ptr::null_mut();
                }
                push(&mut stack, frag(s, list1(&mut (*s).out)));
            }
        }
    }

    if nfa_calc_size {
        // Add one for the match state
        *out_nstate = istate + 1;
        // Return non-null to indicate success (actual pointer doesn't matter)
        return std::ptr::dangling_mut::<NfaState>();
    }

    // Pop final fragment
    let e = match stack.pop() {
        Some(f) => f,
        None => {
            emsg(E_NFA_STACK_ERROR.as_ptr() as *const c_char);
            return ptr::null_mut();
        }
    };

    // Check that stack is empty
    if !stack.is_empty() {
        emsg(E_NFA_TOO_MANY_STATES.as_ptr() as *const c_char);
        return ptr::null_mut();
    }

    // Check state count
    if istate >= nstate {
        emsg(E_NFA_NOT_ENOUGH_SPACE.as_ptr() as *const c_char);
        return ptr::null_mut();
    }

    // Create match state
    let matchstate = alloc_state(
        state_ptr,
        &mut istate,
        nstate,
        NFA_MATCH,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    if matchstate.is_null() {
        return ptr::null_mut();
    }
    (*matchstate).id = 0;

    // Patch final fragment to match state
    patch(e.out, matchstate);

    *out_nstate = istate;
    e.start
}

/// Allocate and initialize an NFA state.
#[inline]
unsafe fn alloc_state(
    state_ptr: *mut NfaState,
    istate: &mut c_int,
    nstate: c_int,
    c: c_int,
    out: *mut NfaState,
    out1: *mut NfaState,
) -> *mut NfaState {
    if *istate >= nstate {
        return ptr::null_mut();
    }

    let s = state_ptr.add(*istate as usize);
    *istate += 1;

    (*s).c = c;
    (*s).out = out;
    (*s).out1 = out1;
    (*s).val = 0;
    (*s).id = *istate;
    (*s).lastlist[0] = 0;
    (*s).lastlist[1] = 0;

    s
}

// =============================================================================
// NFA Pattern Analysis
// =============================================================================

/// Check if NFA is anchored at start of line.
///
/// Returns 1 if pattern always matches at BOL/BOF, 0 otherwise.
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
pub unsafe fn nfa_get_reganch(start: *mut NfaState, depth: c_int) -> c_int {
    if start.is_null() || depth > MAX_DEPTH {
        return 0;
    }

    let mut p = start;
    while !p.is_null() {
        let c = (*p).c;

        match c {
            NFA_BOL | NFA_BOF => return 1,

            NFA_ZSTART
            | NFA_ZEND
            | NFA_CURSOR
            | NFA_VISUAL
            | NFA_MOPEN..=NFA_MOPEN9
            | NFA_NOPEN
            | NFA_ZOPEN..=NFA_ZOPEN9 => {
                p = (*p).out;
            }

            NFA_SPLIT => {
                let r1 = nfa_get_reganch((*p).out, depth + 1);
                let r2 = nfa_get_reganch((*p).out1, depth + 1);
                return if r1 != 0 && r2 != 0 { 1 } else { 0 };
            }

            _ => return 0,
        }
    }
    0
}

/// Get the first mandatory character of the pattern.
///
/// Returns the character code, or 0 if no fixed start character.
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
pub unsafe fn nfa_get_regstart(start: *mut NfaState, depth: c_int) -> c_int {
    if start.is_null() || depth > MAX_DEPTH {
        return 0;
    }

    let mut p = start;
    while !p.is_null() {
        let c = (*p).c;

        // Literal character (positive values < 256 typically)
        if c > 0 && c < 256 {
            return c;
        }

        match c {
            // Transparent states
            NFA_BOL
            | NFA_BOF
            | NFA_BOW
            | NFA_MOPEN
            | NFA_NOPEN
            | NFA_MOPEN1..=NFA_MOPEN9
            | NFA_ZOPEN..=NFA_ZOPEN9
            | NFA_ZSTART
            | NFA_ZEND
            | NFA_CURSOR
            | NFA_VISUAL
            | NFA_LNUM
            | NFA_LNUM_GT
            | NFA_LNUM_LT
            | NFA_COL
            | NFA_COL_GT
            | NFA_COL_LT
            | NFA_VCOL
            | NFA_VCOL_GT
            | NFA_VCOL_LT
            | NFA_MARK
            | NFA_MARK_GT
            | NFA_MARK_LT
            | NFA_EOW => {
                p = (*p).out;
            }

            NFA_SPLIT => {
                let c1 = nfa_get_regstart((*p).out, depth + 1);
                let c2 = nfa_get_regstart((*p).out1, depth + 1);
                return if c1 == c2 { c1 } else { 0 };
            }

            _ => return 0,
        }
    }
    0
}

/// Extract literal match text from NFA if pattern is simple enough.
///
/// Returns pointer to allocated match text, or null if pattern is complex.
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
pub unsafe fn nfa_get_match_text(start: *mut NfaState) -> *mut u8 {
    if start.is_null() {
        return ptr::null_mut();
    }

    // Skip transparent states at start
    let mut p = start;
    loop {
        if p.is_null() {
            return ptr::null_mut();
        }
        let c = (*p).c;

        // Skip MOPEN/NOPEN/ZOPEN
        if c == NFA_MOPEN
            || c == NFA_NOPEN
            || (NFA_MOPEN..=NFA_MOPEN9).contains(&c)
            || (NFA_ZOPEN..=NFA_ZOPEN9).contains(&c)
        {
            p = (*p).out;
            continue;
        }

        // Found a non-transparent state
        break;
    }

    // Check if it's a simple sequence of literals
    let mut len = 0;
    let mut check = p;
    while !check.is_null() {
        let c = (*check).c;

        // Only accept literal characters
        if c > 0 && c < 0x10000 {
            len += utf_char2len(c);
            check = (*check).out;

            // Check for end states
            if !check.is_null() {
                let next_c = (*check).c;
                if next_c == NFA_MCLOSE
                    || (NFA_MCLOSE..=NFA_MCLOSE9).contains(&next_c)
                    || next_c == NFA_NCLOSE
                    || (NFA_ZCLOSE..=NFA_ZCLOSE9).contains(&next_c)
                {
                    check = (*check).out;
                    if !check.is_null() && (*check).c == NFA_MATCH {
                        break;
                    }
                    return ptr::null_mut();
                }
            }
        } else if c == NFA_MATCH {
            break;
        } else {
            return ptr::null_mut();
        }
    }

    if len == 0 {
        return ptr::null_mut();
    }

    // Allocate and fill the match text
    let text = xmalloc((len + 1) as usize) as *mut u8;
    if text.is_null() {
        return ptr::null_mut();
    }

    let mut pos = 0;
    check = p;
    while !check.is_null() {
        let c = (*check).c;
        if c > 0 && c < 0x10000 {
            // Write UTF-8 encoded character
            let clen = utf_char2len(c) as usize;
            if c < 0x80 {
                *text.add(pos) = c as u8;
            } else if c < 0x800 {
                *text.add(pos) = (0xC0 | (c >> 6)) as u8;
                *text.add(pos + 1) = (0x80 | (c & 0x3F)) as u8;
            } else if c < 0x10000 {
                *text.add(pos) = (0xE0 | (c >> 12)) as u8;
                *text.add(pos + 1) = (0x80 | ((c >> 6) & 0x3F)) as u8;
                *text.add(pos + 2) = (0x80 | (c & 0x3F)) as u8;
            }
            pos += clen;
            check = (*check).out;
        } else {
            break;
        }
    }
    *text.add(pos) = 0;

    text
}

/// Estimate maximum byte length of anything matching the state.
///
/// Returns -1 if unknown or unlimited.
///
/// # Safety
/// `startstate` must be a valid NFA state pointer or null.
pub unsafe fn nfa_max_width(startstate: *mut NfaState, depth: c_int) -> c_int {
    if startstate.is_null() || depth > MAX_DEPTH {
        return -1;
    }

    let mut state = startstate;
    let mut len = 0;

    while !state.is_null() {
        let c = (*state).c;

        match c {
            NFA_END_INVISIBLE | NFA_END_INVISIBLE_NEG => return len,

            NFA_SPLIT => {
                let l = nfa_max_width((*state).out, depth + 1);
                let r = nfa_max_width((*state).out1, depth + 1);
                if l < 0 || r < 0 {
                    return -1;
                }
                return len + l.max(r);
            }

            NFA_ANY => {
                len += MB_MAXBYTES;
            }

            // ASCII character classes
            NFA_CLASS_DIGIT | NFA_CLASS_SPACE | NFA_CLASS_XDIGIT => {
                len += 1;
            }

            // Potentially multibyte character classes
            NFA_CLASS_ALNUM | NFA_CLASS_ALPHA | NFA_CLASS_LOWER | NFA_CLASS_UPPER
            | NFA_CLASS_PRINT | NFA_CLASS_PUNCT | NFA_CLASS_GRAPH | NFA_CLASS_CNTRL
            | NFA_CLASS_BLANK | NFA_CLASS_IDENT | NFA_CLASS_KEYWORD | NFA_CLASS_FNAME
            | NFA_CLASS_TAB | NFA_CLASS_RETURN | NFA_CLASS_BACKSPACE | NFA_CLASS_ESCAPE => {
                len += 3;
            }

            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_NEG => {
                // Zero-width, skip to after END
                state = (*(*state).out1).out;
                continue;
            }

            // Backreferences - unknown width
            NFA_BACKREF1..=NFA_BACKREF9 | NFA_ZREF1..=NFA_ZREF9 | NFA_SKIP => {
                return -1;
            }

            // Zero-width matches
            NFA_BOL
            | NFA_EOL
            | NFA_BOF
            | NFA_EOF
            | NFA_BOW
            | NFA_EOW
            | NFA_MOPEN..=NFA_MOPEN9
            | NFA_ZOPEN..=NFA_ZOPEN9
            | NFA_ZCLOSE..=NFA_ZCLOSE9
            | NFA_MCLOSE..=NFA_MCLOSE9
            | NFA_NOPEN
            | NFA_NCLOSE
            | NFA_LNUM
            | NFA_LNUM_GT
            | NFA_LNUM_LT
            | NFA_COL
            | NFA_COL_GT
            | NFA_COL_LT
            | NFA_VCOL
            | NFA_VCOL_GT
            | NFA_VCOL_LT
            | NFA_MARK
            | NFA_MARK_GT
            | NFA_MARK_LT
            | NFA_VISUAL
            | NFA_CURSOR
            | NFA_ZSTART
            | NFA_ZEND
            | NFA_EMPTY
            | NFA_START_PATTERN
            | NFA_END_PATTERN
            | NFA_COMPOSING
            | NFA_END_COMPOSING => {
                // Zero-width, continue
            }

            _ => {
                if c < 0 {
                    return -1;
                }
                // Normal character
                len += utf_char2len(c);
            }
        }

        state = (*state).out;
    }

    -1
}

// =============================================================================
// NFA Post-processing Optimization
// =============================================================================

/// Maximum recursion depth for match_follows
const MATCH_FOLLOWS_MAX_DEPTH: c_int = 10;

/// Maximum recursion depth for failure_chance
const FAILURE_CHANCE_MAX_DEPTH: c_int = 4;

/// Check if a match immediately follows the current state.
///
/// Returns true if the pattern could match without consuming input.
///
/// # Safety
/// `startstate` must be a valid NFA state pointer or null.
pub unsafe fn match_follows(startstate: *const NfaState, depth: c_int) -> bool {
    if startstate.is_null() || depth > MATCH_FOLLOWS_MAX_DEPTH {
        return false;
    }

    let mut state = startstate;
    while !state.is_null() {
        let c = (*state).c;

        match c {
            NFA_MATCH
            | NFA_MCLOSE
            | NFA_END_INVISIBLE
            | NFA_END_INVISIBLE_NEG
            | NFA_END_PATTERN => {
                return true;
            }

            NFA_SPLIT => {
                return match_follows((*state).out, depth + 1)
                    || match_follows((*state).out1, depth + 1);
            }

            NFA_START_INVISIBLE
            | NFA_START_INVISIBLE_FIRST
            | NFA_START_INVISIBLE_BEFORE
            | NFA_START_INVISIBLE_BEFORE_FIRST
            | NFA_START_INVISIBLE_NEG
            | NFA_START_INVISIBLE_NEG_FIRST
            | NFA_START_INVISIBLE_BEFORE_NEG
            | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
            | NFA_COMPOSING => {
                // Skip ahead to next state
                state = (*(*state).out1).out;
                continue;
            }

            // States that advance input
            NFA_ANY | NFA_IDENT | NFA_SIDENT | NFA_KWORD | NFA_SKWORD | NFA_FNAME | NFA_SFNAME
            | NFA_PRINT | NFA_SPRINT | NFA_WHITE | NFA_NWHITE | NFA_DIGIT | NFA_NDIGIT
            | NFA_HEX | NFA_NHEX | NFA_OCTAL | NFA_NOCTAL | NFA_WORD | NFA_NWORD | NFA_HEAD
            | NFA_NHEAD | NFA_ALPHA | NFA_NALPHA | NFA_LOWER | NFA_NLOWER | NFA_UPPER
            | NFA_NUPPER | NFA_LOWER_IC | NFA_NLOWER_IC | NFA_UPPER_IC | NFA_NUPPER_IC
            | NFA_START_COLL | NFA_START_NEG_COLL | NFA_NEWL => {
                return false;
            }

            _ => {
                if c > 0 {
                    // State will advance input
                    return false;
                }
                // Zero-width, continue looking
            }
        }
        state = (*state).out;
    }
    false
}

/// Estimate the failure chance of a pattern.
///
/// Returns a value from 0 (always succeeds) to 99 (likely to fail).
///
/// # Safety
/// `state` must be a valid NFA state pointer or null.
pub unsafe fn failure_chance(state: *mut NfaState, depth: c_int) -> c_int {
    if state.is_null() || depth > FAILURE_CHANCE_MAX_DEPTH {
        return 1;
    }

    let c = (*state).c;

    match c {
        NFA_SPLIT => {
            if (*(*state).out).c == NFA_SPLIT || (*(*state).out1).c == NFA_SPLIT {
                // Avoid recursive stuff
                return 1;
            }
            // Two alternatives, use the lowest failure chance
            let l = failure_chance((*state).out, depth + 1);
            let r = failure_chance((*state).out1, depth + 1);
            if l < r {
                l
            } else {
                r
            }
        }

        NFA_ANY => 1, // Matches anything, unlikely to fail

        NFA_MATCH | NFA_MCLOSE | NFA_ANY_COMPOSING => 0, // Empty match works always

        NFA_START_INVISIBLE
        | NFA_START_INVISIBLE_FIRST
        | NFA_START_INVISIBLE_NEG
        | NFA_START_INVISIBLE_NEG_FIRST
        | NFA_START_INVISIBLE_BEFORE
        | NFA_START_INVISIBLE_BEFORE_FIRST
        | NFA_START_INVISIBLE_BEFORE_NEG
        | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
        | NFA_START_PATTERN => 5, // Recursive regmatch is expensive

        NFA_BOL | NFA_EOL | NFA_BOF | NFA_EOF | NFA_NEWL => 99,

        NFA_BOW | NFA_EOW => 90,

        // Transparent states
        NFA_MOPEN..=NFA_MOPEN9
        | NFA_ZOPEN..=NFA_ZOPEN9
        | NFA_ZCLOSE..=NFA_ZCLOSE9
        | NFA_NOPEN
        | NFA_MCLOSE..=NFA_MCLOSE9
        | NFA_NCLOSE => failure_chance((*state).out, depth + 1),

        // Backreferences
        NFA_BACKREF1..=NFA_BACKREF9 | NFA_ZREF1..=NFA_ZREF9 => 40,

        NFA_LNUM | NFA_VCOL | NFA_COL | NFA_MARK | NFA_CURSOR | NFA_VISUAL => 90,

        // Default: unknown
        _ => 50,
    }
}

/// Post-process NFA to add optimization hints.
///
/// This examines invisible match states and decides whether to execute
/// them immediately or postpone them based on failure chance heuristics.
///
/// # Safety
/// `states` must point to an array of `nstate` NfaState structs.
pub unsafe fn nfa_postprocess(states: *mut NfaState, nstate: c_int) {
    for i in 0..nstate {
        let state = states.add(i as usize);
        let c = (*state).c;

        if c == NFA_START_INVISIBLE
            || c == NFA_START_INVISIBLE_NEG
            || c == NFA_START_INVISIBLE_BEFORE
            || c == NFA_START_INVISIBLE_BEFORE_NEG
        {
            // Determine whether to execute directly or postpone
            let directly = if match_follows((*(*state).out1).out, 0) {
                // Do it directly when what follows is possibly the end of the match
                true
            } else {
                let ch_invisible = failure_chance((*state).out, 0);
                let ch_follows = failure_chance((*(*state).out1).out, 0);

                if c == NFA_START_INVISIBLE_BEFORE || c == NFA_START_INVISIBLE_BEFORE_NEG {
                    // "before" matches are very expensive when unbounded
                    if (*state).val <= 0 && ch_follows > 0 {
                        false
                    } else {
                        ch_follows * 10 < ch_invisible
                    }
                } else {
                    // Normal invisible, first do the one with highest failure chance
                    ch_follows < ch_invisible
                }
            };

            if directly {
                // Switch to the _FIRST state variant
                (*state).c += 1;
            }
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Convert postfix to NFA (FFI entry point).
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_post2nfa(
    postfix: *const c_int,
    end: *const c_int,
    nfa_calc_size: c_int,
    state_ptr: *mut NfaState,
    nstate: c_int,
    out_nstate: *mut c_int,
) -> *mut NfaState {
    post2nfa(
        postfix,
        end,
        nfa_calc_size != 0,
        state_ptr,
        nstate,
        out_nstate,
    )
}

/// Get pattern anchor status (FFI entry point).
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_reganch_full(start: *mut NfaState, depth: c_int) -> c_int {
    nfa_get_reganch(start, depth)
}

/// Get pattern start character (FFI entry point).
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_regstart_full(start: *mut NfaState, depth: c_int) -> c_int {
    nfa_get_regstart(start, depth)
}

/// Get match text (FFI entry point).
///
/// # Safety
/// `start` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_get_match_text_full(start: *mut NfaState) -> *mut u8 {
    nfa_get_match_text(start)
}

/// Get max width of pattern (FFI entry point).
///
/// # Safety
/// `startstate` must be a valid NFA state pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_max_width(startstate: *mut NfaState, depth: c_int) -> c_int {
    nfa_max_width(startstate, depth)
}

/// Post-process NFA for optimization on raw state array.
///
/// This is the core implementation that operates directly on the state array.
/// Use `rs_nfa_postprocess` (in nfa_compile.rs) for the FFI entry point that
/// takes a prog pointer.
///
/// # Safety
/// `states` must point to an array of `nstate` NfaState structs.
pub unsafe fn nfa_postprocess_states(states: *mut NfaState, nstate: c_int) {
    nfa_postprocess(states, nstate)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify MOPEN/MCLOSE relationship
        assert_eq!(NFA_MOPEN9 - NFA_MOPEN, 9);
        assert_eq!(NFA_ZOPEN9 - NFA_ZOPEN, 9);
        assert_eq!(NFA_ZCLOSE9 - NFA_ZCLOSE, 9);
    }
}
