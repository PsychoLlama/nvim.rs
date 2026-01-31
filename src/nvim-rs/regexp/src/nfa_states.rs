//! NFA regex engine state types.
//!
//! This module defines the state types used by the NFA regex engine.
//! The NFA engine uses a parallel state tracking approach rather than
//! the backtracking used by the BT engine.
//!
//! # Architecture
//!
//! The NFA engine uses Thompson's construction for pattern compilation and
//! simulated NFA execution for matching. Key structures:
//!
//! - [`NfaState`]: A single NFA state with character and transitions
//! - [`NfaThread`]: Execution thread tracking state and submatches
//! - [`NfaList`]: List of active threads for parallel execution
//! - [`NfaPim`]: Postponed Invisible Match for lookahead/lookbehind

use std::ffi::{c_int, c_void};
use std::ptr;

// =============================================================================
// NFA State Constants
// =============================================================================

// Note: NFA states start from -1024 and increment.
// The enum values must match the C enum in regexp.c.

/// NFA split state - represents an alternation point.
pub const NFA_SPLIT: c_int = -1024;

/// NFA match state - successful pattern match.
pub const NFA_MATCH: c_int = NFA_SPLIT + 1; // -1023

/// NFA empty state - matches 0-length.
pub const NFA_EMPTY: c_int = NFA_MATCH + 1; // -1022

// Collection states
/// Start of `[abc]` collection.
pub const NFA_START_COLL: c_int = NFA_EMPTY + 1;
/// End of `[abc]` collection.
pub const NFA_END_COLL: c_int = NFA_START_COLL + 1;
/// Start of `[^abc]` negated collection.
pub const NFA_START_NEG_COLL: c_int = NFA_END_COLL + 1;
/// End of `[^abc]` negated collection (postfix only).
pub const NFA_END_NEG_COLL: c_int = NFA_START_NEG_COLL + 1;
/// Range of two previous items (postfix only).
pub const NFA_RANGE: c_int = NFA_END_NEG_COLL + 1;
/// Low end of a range.
pub const NFA_RANGE_MIN: c_int = NFA_RANGE + 1;
/// High end of a range.
pub const NFA_RANGE_MAX: c_int = NFA_RANGE_MIN + 1;

// Postfix operators
/// Concatenate two previous items (postfix only).
pub const NFA_CONCAT: c_int = NFA_RANGE_MAX + 1;
/// `\|` alternation (postfix only).
pub const NFA_OR: c_int = NFA_CONCAT + 1;
/// Greedy `*` (postfix only).
pub const NFA_STAR: c_int = NFA_OR + 1;
/// Non-greedy `*` (postfix only).
pub const NFA_STAR_NONGREEDY: c_int = NFA_STAR + 1;
/// Greedy `\?` (postfix only).
pub const NFA_QUEST: c_int = NFA_STAR_NONGREEDY + 1;
/// Non-greedy `\?` (postfix only).
pub const NFA_QUEST_NONGREEDY: c_int = NFA_QUEST + 1;

// Anchors
/// `^` Begin line.
pub const NFA_BOL: c_int = NFA_QUEST_NONGREEDY + 1;
/// `$` End line.
pub const NFA_EOL: c_int = NFA_BOL + 1;
/// `\<` Begin word.
pub const NFA_BOW: c_int = NFA_EOL + 1;
/// `\>` End word.
pub const NFA_EOW: c_int = NFA_BOW + 1;
/// `\%^` Begin file.
pub const NFA_BOF: c_int = NFA_EOW + 1;
/// `\%$` End file.
pub const NFA_EOF: c_int = NFA_BOF + 1;
/// Newline.
pub const NFA_NEWL: c_int = NFA_EOF + 1;
/// Used for `\zs`.
pub const NFA_ZSTART: c_int = NFA_NEWL + 1;
/// Used for `\ze`.
pub const NFA_ZEND: c_int = NFA_ZSTART + 1;
/// Start of subexpression marked with `\%(`.
pub const NFA_NOPEN: c_int = NFA_ZEND + 1;
/// End of subexpr. marked with `\%( ... \)`.
pub const NFA_NCLOSE: c_int = NFA_NOPEN + 1;

// Invisible/lookaround states
/// Start invisible match.
pub const NFA_START_INVISIBLE: c_int = NFA_NCLOSE + 1;
/// Start invisible match (first).
pub const NFA_START_INVISIBLE_FIRST: c_int = NFA_START_INVISIBLE + 1;
/// Start invisible negative match.
pub const NFA_START_INVISIBLE_NEG: c_int = NFA_START_INVISIBLE_FIRST + 1;
/// Start invisible negative match (first).
pub const NFA_START_INVISIBLE_NEG_FIRST: c_int = NFA_START_INVISIBLE_NEG + 1;
/// Start invisible before match.
pub const NFA_START_INVISIBLE_BEFORE: c_int = NFA_START_INVISIBLE_NEG_FIRST + 1;
/// Start invisible before match (first).
pub const NFA_START_INVISIBLE_BEFORE_FIRST: c_int = NFA_START_INVISIBLE_BEFORE + 1;
/// Start invisible before negative match.
pub const NFA_START_INVISIBLE_BEFORE_NEG: c_int = NFA_START_INVISIBLE_BEFORE_FIRST + 1;
/// Start invisible before negative match (first).
pub const NFA_START_INVISIBLE_BEFORE_NEG_FIRST: c_int = NFA_START_INVISIBLE_BEFORE_NEG + 1;
/// Start pattern.
pub const NFA_START_PATTERN: c_int = NFA_START_INVISIBLE_BEFORE_NEG_FIRST + 1;
/// End invisible match.
pub const NFA_END_INVISIBLE: c_int = NFA_START_PATTERN + 1;
/// End invisible negative match.
pub const NFA_END_INVISIBLE_NEG: c_int = NFA_END_INVISIBLE + 1;
/// End pattern.
pub const NFA_END_PATTERN: c_int = NFA_END_INVISIBLE_NEG + 1;

// Composing character states
/// Next nodes are part of composing multibyte char.
pub const NFA_COMPOSING: c_int = NFA_END_PATTERN + 1;
/// End of composing char in NFA.
pub const NFA_END_COMPOSING: c_int = NFA_COMPOSING + 1;
/// `\%C`: Any composing characters.
pub const NFA_ANY_COMPOSING: c_int = NFA_END_COMPOSING + 1;
/// `\%[abc]` optional characters.
pub const NFA_OPT_CHARS: c_int = NFA_ANY_COMPOSING + 1;

// Postfix-only lookaround atoms
/// Used for `\@=`.
pub const NFA_PREV_ATOM_NO_WIDTH: c_int = NFA_OPT_CHARS + 1;
/// Used for `\@!`.
pub const NFA_PREV_ATOM_NO_WIDTH_NEG: c_int = NFA_PREV_ATOM_NO_WIDTH + 1;
/// Used for `\@<=`.
pub const NFA_PREV_ATOM_JUST_BEFORE: c_int = NFA_PREV_ATOM_NO_WIDTH_NEG + 1;
/// Used for `\@<!`.
pub const NFA_PREV_ATOM_JUST_BEFORE_NEG: c_int = NFA_PREV_ATOM_JUST_BEFORE + 1;
/// Used for `\@>`.
pub const NFA_PREV_ATOM_LIKE_PATTERN: c_int = NFA_PREV_ATOM_JUST_BEFORE_NEG + 1;

// Backreferences
/// `\1` backreference.
pub const NFA_BACKREF1: c_int = NFA_PREV_ATOM_LIKE_PATTERN + 1;
/// `\2` backreference.
pub const NFA_BACKREF2: c_int = NFA_BACKREF1 + 1;
/// `\3` backreference.
pub const NFA_BACKREF3: c_int = NFA_BACKREF2 + 1;
/// `\4` backreference.
pub const NFA_BACKREF4: c_int = NFA_BACKREF3 + 1;
/// `\5` backreference.
pub const NFA_BACKREF5: c_int = NFA_BACKREF4 + 1;
/// `\6` backreference.
pub const NFA_BACKREF6: c_int = NFA_BACKREF5 + 1;
/// `\7` backreference.
pub const NFA_BACKREF7: c_int = NFA_BACKREF6 + 1;
/// `\8` backreference.
pub const NFA_BACKREF8: c_int = NFA_BACKREF7 + 1;
/// `\9` backreference.
pub const NFA_BACKREF9: c_int = NFA_BACKREF8 + 1;

// External submatches
/// `\z1` external submatch.
pub const NFA_ZREF1: c_int = NFA_BACKREF9 + 1;
/// `\z2` external submatch.
pub const NFA_ZREF2: c_int = NFA_ZREF1 + 1;
/// `\z3` external submatch.
pub const NFA_ZREF3: c_int = NFA_ZREF2 + 1;
/// `\z4` external submatch.
pub const NFA_ZREF4: c_int = NFA_ZREF3 + 1;
/// `\z5` external submatch.
pub const NFA_ZREF5: c_int = NFA_ZREF4 + 1;
/// `\z6` external submatch.
pub const NFA_ZREF6: c_int = NFA_ZREF5 + 1;
/// `\z7` external submatch.
pub const NFA_ZREF7: c_int = NFA_ZREF6 + 1;
/// `\z8` external submatch.
pub const NFA_ZREF8: c_int = NFA_ZREF7 + 1;
/// `\z9` external submatch.
pub const NFA_ZREF9: c_int = NFA_ZREF8 + 1;

/// Skip characters.
pub const NFA_SKIP: c_int = NFA_ZREF9 + 1;

// Subexpression open markers (MOPEN)
/// `\(` start of subexpr 0 (whole match).
pub const NFA_MOPEN: c_int = NFA_SKIP + 1;
/// `\(` start of subexpr 1.
pub const NFA_MOPEN1: c_int = NFA_MOPEN + 1;
/// `\(` start of subexpr 2.
pub const NFA_MOPEN2: c_int = NFA_MOPEN1 + 1;
/// `\(` start of subexpr 3.
pub const NFA_MOPEN3: c_int = NFA_MOPEN2 + 1;
/// `\(` start of subexpr 4.
pub const NFA_MOPEN4: c_int = NFA_MOPEN3 + 1;
/// `\(` start of subexpr 5.
pub const NFA_MOPEN5: c_int = NFA_MOPEN4 + 1;
/// `\(` start of subexpr 6.
pub const NFA_MOPEN6: c_int = NFA_MOPEN5 + 1;
/// `\(` start of subexpr 7.
pub const NFA_MOPEN7: c_int = NFA_MOPEN6 + 1;
/// `\(` start of subexpr 8.
pub const NFA_MOPEN8: c_int = NFA_MOPEN7 + 1;
/// `\(` start of subexpr 9.
pub const NFA_MOPEN9: c_int = NFA_MOPEN8 + 1;

// Subexpression close markers (MCLOSE)
/// `\)` end of subexpr 0.
pub const NFA_MCLOSE: c_int = NFA_MOPEN9 + 1;
/// `\)` end of subexpr 1.
pub const NFA_MCLOSE1: c_int = NFA_MCLOSE + 1;
/// `\)` end of subexpr 2.
pub const NFA_MCLOSE2: c_int = NFA_MCLOSE1 + 1;
/// `\)` end of subexpr 3.
pub const NFA_MCLOSE3: c_int = NFA_MCLOSE2 + 1;
/// `\)` end of subexpr 4.
pub const NFA_MCLOSE4: c_int = NFA_MCLOSE3 + 1;
/// `\)` end of subexpr 5.
pub const NFA_MCLOSE5: c_int = NFA_MCLOSE4 + 1;
/// `\)` end of subexpr 6.
pub const NFA_MCLOSE6: c_int = NFA_MCLOSE5 + 1;
/// `\)` end of subexpr 7.
pub const NFA_MCLOSE7: c_int = NFA_MCLOSE6 + 1;
/// `\)` end of subexpr 8.
pub const NFA_MCLOSE8: c_int = NFA_MCLOSE7 + 1;
/// `\)` end of subexpr 9.
pub const NFA_MCLOSE9: c_int = NFA_MCLOSE8 + 1;

// External subexpr open markers (ZOPEN)
/// `\z(` start of external subexpr 0.
pub const NFA_ZOPEN: c_int = NFA_MCLOSE9 + 1;
/// `\z(` start of external subexpr 1.
pub const NFA_ZOPEN1: c_int = NFA_ZOPEN + 1;
/// `\z(` start of external subexpr 2.
pub const NFA_ZOPEN2: c_int = NFA_ZOPEN1 + 1;
/// `\z(` start of external subexpr 3.
pub const NFA_ZOPEN3: c_int = NFA_ZOPEN2 + 1;
/// `\z(` start of external subexpr 4.
pub const NFA_ZOPEN4: c_int = NFA_ZOPEN3 + 1;
/// `\z(` start of external subexpr 5.
pub const NFA_ZOPEN5: c_int = NFA_ZOPEN4 + 1;
/// `\z(` start of external subexpr 6.
pub const NFA_ZOPEN6: c_int = NFA_ZOPEN5 + 1;
/// `\z(` start of external subexpr 7.
pub const NFA_ZOPEN7: c_int = NFA_ZOPEN6 + 1;
/// `\z(` start of external subexpr 8.
pub const NFA_ZOPEN8: c_int = NFA_ZOPEN7 + 1;
/// `\z(` start of external subexpr 9.
pub const NFA_ZOPEN9: c_int = NFA_ZOPEN8 + 1;

// External subexpr close markers (ZCLOSE)
/// `\z)` end of external subexpr 0.
pub const NFA_ZCLOSE: c_int = NFA_ZOPEN9 + 1;
/// `\z)` end of external subexpr 1.
pub const NFA_ZCLOSE1: c_int = NFA_ZCLOSE + 1;
/// `\z)` end of external subexpr 2.
pub const NFA_ZCLOSE2: c_int = NFA_ZCLOSE1 + 1;
/// `\z)` end of external subexpr 3.
pub const NFA_ZCLOSE3: c_int = NFA_ZCLOSE2 + 1;
/// `\z)` end of external subexpr 4.
pub const NFA_ZCLOSE4: c_int = NFA_ZCLOSE3 + 1;
/// `\z)` end of external subexpr 5.
pub const NFA_ZCLOSE5: c_int = NFA_ZCLOSE4 + 1;
/// `\z)` end of external subexpr 6.
pub const NFA_ZCLOSE6: c_int = NFA_ZCLOSE5 + 1;
/// `\z)` end of external subexpr 7.
pub const NFA_ZCLOSE7: c_int = NFA_ZCLOSE6 + 1;
/// `\z)` end of external subexpr 8.
pub const NFA_ZCLOSE8: c_int = NFA_ZCLOSE7 + 1;
/// `\z)` end of external subexpr 9.
pub const NFA_ZCLOSE9: c_int = NFA_ZCLOSE8 + 1;

// Character classes (NFA_ANY through NFA_NUPPER_IC)
// Note: NFA_FIRST_NL = NFA_ANY + NFA_ADD_NL

/// Match any one character.
pub const NFA_ANY: c_int = NFA_ZCLOSE9 + 1;
/// Match identifier char.
pub const NFA_IDENT: c_int = NFA_ANY + 1;
/// Match identifier char but no digit.
pub const NFA_SIDENT: c_int = NFA_IDENT + 1;
/// Match keyword char.
pub const NFA_KWORD: c_int = NFA_SIDENT + 1;
/// Match word char but no digit.
pub const NFA_SKWORD: c_int = NFA_KWORD + 1;
/// Match file name char.
pub const NFA_FNAME: c_int = NFA_SKWORD + 1;
/// Match file name char but no digit.
pub const NFA_SFNAME: c_int = NFA_FNAME + 1;
/// Match printable char.
pub const NFA_PRINT: c_int = NFA_SFNAME + 1;
/// Match printable char but no digit.
pub const NFA_SPRINT: c_int = NFA_PRINT + 1;
/// Match whitespace char.
pub const NFA_WHITE: c_int = NFA_SPRINT + 1;
/// Match non-whitespace char.
pub const NFA_NWHITE: c_int = NFA_WHITE + 1;
/// Match digit char.
pub const NFA_DIGIT: c_int = NFA_NWHITE + 1;
/// Match non-digit char.
pub const NFA_NDIGIT: c_int = NFA_DIGIT + 1;
/// Match hex char.
pub const NFA_HEX: c_int = NFA_NDIGIT + 1;
/// Match non-hex char.
pub const NFA_NHEX: c_int = NFA_HEX + 1;
/// Match octal char.
pub const NFA_OCTAL: c_int = NFA_NHEX + 1;
/// Match non-octal char.
pub const NFA_NOCTAL: c_int = NFA_OCTAL + 1;
/// Match word char.
pub const NFA_WORD: c_int = NFA_NOCTAL + 1;
/// Match non-word char.
pub const NFA_NWORD: c_int = NFA_WORD + 1;
/// Match head char.
pub const NFA_HEAD: c_int = NFA_NWORD + 1;
/// Match non-head char.
pub const NFA_NHEAD: c_int = NFA_HEAD + 1;
/// Match alpha char.
pub const NFA_ALPHA: c_int = NFA_NHEAD + 1;
/// Match non-alpha char.
pub const NFA_NALPHA: c_int = NFA_ALPHA + 1;
/// Match lowercase char.
pub const NFA_LOWER: c_int = NFA_NALPHA + 1;
/// Match non-lowercase char.
pub const NFA_NLOWER: c_int = NFA_LOWER + 1;
/// Match uppercase char.
pub const NFA_UPPER: c_int = NFA_NLOWER + 1;
/// Match non-uppercase char.
pub const NFA_NUPPER: c_int = NFA_UPPER + 1;
/// Match `[a-z]` case-insensitive.
pub const NFA_LOWER_IC: c_int = NFA_NUPPER + 1;
/// Match `[^a-z]` case-insensitive.
pub const NFA_NLOWER_IC: c_int = NFA_LOWER_IC + 1;
/// Match `[A-Z]` case-insensitive.
pub const NFA_UPPER_IC: c_int = NFA_NLOWER_IC + 1;
/// Match `[^A-Z]` case-insensitive.
pub const NFA_NUPPER_IC: c_int = NFA_UPPER_IC + 1;

// NL variants
/// Offset to add for NL-including variants.
pub const NFA_ADD_NL: c_int = 31;
/// First opcode that includes NL matching.
pub const NFA_FIRST_NL: c_int = NFA_ANY + NFA_ADD_NL;
/// Last opcode that includes NL matching.
pub const NFA_LAST_NL: c_int = NFA_NUPPER_IC + NFA_ADD_NL;

// Position matching
// Note: These come after NFA_LAST_NL in the C enum, not after NFA_NUPPER_IC
/// Match cursor position.
pub const NFA_CURSOR: c_int = NFA_LAST_NL + 1;
/// Match line number.
pub const NFA_LNUM: c_int = NFA_CURSOR + 1;
/// Match > line number.
pub const NFA_LNUM_GT: c_int = NFA_LNUM + 1;
/// Match < line number.
pub const NFA_LNUM_LT: c_int = NFA_LNUM_GT + 1;
/// Match cursor column.
pub const NFA_COL: c_int = NFA_LNUM_LT + 1;
/// Match > cursor column.
pub const NFA_COL_GT: c_int = NFA_COL + 1;
/// Match < cursor column.
pub const NFA_COL_LT: c_int = NFA_COL_GT + 1;
/// Match cursor virtual column.
pub const NFA_VCOL: c_int = NFA_COL_LT + 1;
/// Match > cursor virtual column.
pub const NFA_VCOL_GT: c_int = NFA_VCOL + 1;
/// Match < cursor virtual column.
pub const NFA_VCOL_LT: c_int = NFA_VCOL_GT + 1;
/// Match mark.
pub const NFA_MARK: c_int = NFA_VCOL_LT + 1;
/// Match > mark.
pub const NFA_MARK_GT: c_int = NFA_MARK + 1;
/// Match < mark.
pub const NFA_MARK_LT: c_int = NFA_MARK_GT + 1;
/// Match Visual area.
pub const NFA_VISUAL: c_int = NFA_MARK_LT + 1;

// POSIX character classes
/// `[:alnum:]` character class.
pub const NFA_CLASS_ALNUM: c_int = NFA_VISUAL + 1;
/// `[:alpha:]` character class.
pub const NFA_CLASS_ALPHA: c_int = NFA_CLASS_ALNUM + 1;
/// `[:blank:]` character class.
pub const NFA_CLASS_BLANK: c_int = NFA_CLASS_ALPHA + 1;
/// `[:cntrl:]` character class.
pub const NFA_CLASS_CNTRL: c_int = NFA_CLASS_BLANK + 1;
/// `[:digit:]` character class.
pub const NFA_CLASS_DIGIT: c_int = NFA_CLASS_CNTRL + 1;
/// `[:graph:]` character class.
pub const NFA_CLASS_GRAPH: c_int = NFA_CLASS_DIGIT + 1;
/// `[:lower:]` character class.
pub const NFA_CLASS_LOWER: c_int = NFA_CLASS_GRAPH + 1;
/// `[:print:]` character class.
pub const NFA_CLASS_PRINT: c_int = NFA_CLASS_LOWER + 1;
/// `[:punct:]` character class.
pub const NFA_CLASS_PUNCT: c_int = NFA_CLASS_PRINT + 1;
/// `[:space:]` character class.
pub const NFA_CLASS_SPACE: c_int = NFA_CLASS_PUNCT + 1;
/// `[:upper:]` character class.
pub const NFA_CLASS_UPPER: c_int = NFA_CLASS_SPACE + 1;
/// `[:xdigit:]` character class.
pub const NFA_CLASS_XDIGIT: c_int = NFA_CLASS_UPPER + 1;
/// `[:tab:]` character class.
pub const NFA_CLASS_TAB: c_int = NFA_CLASS_XDIGIT + 1;
/// `[:return:]` character class.
pub const NFA_CLASS_RETURN: c_int = NFA_CLASS_TAB + 1;
/// `[:backspace:]` character class.
pub const NFA_CLASS_BACKSPACE: c_int = NFA_CLASS_RETURN + 1;
/// `[:escape:]` character class.
pub const NFA_CLASS_ESCAPE: c_int = NFA_CLASS_BACKSPACE + 1;
/// `[:ident:]` character class.
pub const NFA_CLASS_IDENT: c_int = NFA_CLASS_ESCAPE + 1;
/// `[:keyword:]` character class.
pub const NFA_CLASS_KEYWORD: c_int = NFA_CLASS_IDENT + 1;
/// `[:fname:]` character class.
pub const NFA_CLASS_FNAME: c_int = NFA_CLASS_KEYWORD + 1;

// =============================================================================
// PIM (Postponed Invisible Match) states
// =============================================================================

/// PIM not used.
pub const NFA_PIM_UNUSED: c_int = 0;
/// PIM not done yet.
pub const NFA_PIM_TODO: c_int = 1;
/// PIM executed, matches.
pub const NFA_PIM_MATCH: c_int = 2;
/// PIM executed, no match.
pub const NFA_PIM_NOMATCH: c_int = 3;

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if NFA state includes newline matching.
#[inline]
pub const fn nfa_with_nl(state: c_int) -> bool {
    state >= NFA_FIRST_NL && state <= NFA_LAST_NL
}

/// Convert a normal NFA char class to its NL-including variant.
#[inline]
pub const fn nfa_add_nl(state: c_int) -> c_int {
    state + NFA_ADD_NL
}

/// Convert an NL-including NFA char class to its normal variant.
#[inline]
pub const fn nfa_remove_nl(state: c_int) -> c_int {
    state - NFA_ADD_NL
}

/// Get the MOPEN state for a given subexpr number (0-9).
#[inline]
pub const fn nfa_mopen(n: c_int) -> c_int {
    NFA_MOPEN + n
}

/// Get the MCLOSE state for a given subexpr number (0-9).
#[inline]
pub const fn nfa_mclose(n: c_int) -> c_int {
    NFA_MCLOSE + n
}

/// Get the BACKREF state for a given backref number (1-9).
#[inline]
pub const fn nfa_backref(n: c_int) -> c_int {
    NFA_BACKREF1 + n - 1
}

/// Check if state is an MOPEN state (0-9).
#[inline]
pub const fn is_nfa_mopen(state: c_int) -> bool {
    state >= NFA_MOPEN && state <= NFA_MOPEN9
}

/// Check if state is an MCLOSE state (0-9).
#[inline]
pub const fn is_nfa_mclose(state: c_int) -> bool {
    state >= NFA_MCLOSE && state <= NFA_MCLOSE9
}

/// Check if state is a ZOPEN state (0-9).
#[inline]
pub const fn is_nfa_zopen(state: c_int) -> bool {
    state >= NFA_ZOPEN && state <= NFA_ZOPEN9
}

/// Check if state is a ZCLOSE state (0-9).
#[inline]
pub const fn is_nfa_zclose(state: c_int) -> bool {
    state >= NFA_ZCLOSE && state <= NFA_ZCLOSE9
}

/// Get the subexpression number from an MOPEN/MCLOSE state.
#[inline]
pub const fn nfa_get_subexpr_num(state: c_int) -> c_int {
    if is_nfa_mopen(state) {
        state - NFA_MOPEN
    } else if is_nfa_mclose(state) {
        state - NFA_MCLOSE
    } else if is_nfa_zopen(state) {
        state - NFA_ZOPEN
    } else if is_nfa_zclose(state) {
        state - NFA_ZCLOSE
    } else {
        -1
    }
}

// =============================================================================
// NFA State Machine Structures
// =============================================================================

/// Number of subexpressions supported.
pub const NSUBEXP: usize = 10;

/// Line number type (matches linenr_T in C).
pub type LineNr = c_int;

/// Column number type (matches colnr_T in C).
pub type ColNr = c_int;

/// Opaque handle to an NFA state (nfa_state_T*).
///
/// This is used for FFI calls to C code that manages the actual state memory.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NfaStateHandle(*mut c_void);

impl NfaStateHandle {
    /// Create a handle from a raw pointer.
    #[inline]
    pub const fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

impl Default for NfaStateHandle {
    fn default() -> Self {
        Self::null()
    }
}

/// NFA state structure matching the C nfa_state_T.
///
/// Represents a single state in the NFA. An NFA state may have:
/// - No outgoing edges (when c == NFA_MATCH, matching state)
/// - One unlabeled edge to `out` (when c == NFA_EMPTY or character match)
/// - Two unlabeled edges to `out` and `out1` (when c == NFA_SPLIT)
#[repr(C)]
#[derive(Debug)]
pub struct NfaState {
    /// Character/opcode for this state.
    /// - If c >= 0: labeled edge matching character c
    /// - If c == NFA_SPLIT: split state with edges to out and out1
    /// - If c == NFA_MATCH: accepting state (no edges)
    /// - Other negative values: special opcodes (see NFA_* constants)
    pub c: c_int,
    /// Primary outgoing edge (may be NULL).
    pub out: *mut NfaState,
    /// Secondary outgoing edge for split states (may be NULL).
    pub out1: *mut NfaState,
    /// State ID (for debugging and state tracking).
    pub id: c_int,
    /// Last list ID this state was added to.
    /// Index 0: normal matching, Index 1: recursive matching.
    pub lastlist: [c_int; 2],
    /// Value storage for the state (context-dependent).
    pub val: c_int,
}

impl NfaState {
    /// Check if this is a match (accepting) state.
    #[inline]
    pub fn is_match(&self) -> bool {
        self.c == NFA_MATCH
    }

    /// Check if this is a split state.
    #[inline]
    pub fn is_split(&self) -> bool {
        self.c == NFA_SPLIT
    }

    /// Check if this is an empty transition state.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.c == NFA_EMPTY
    }

    /// Check if this state matches a literal character.
    #[inline]
    pub fn is_char(&self) -> bool {
        self.c >= 0
    }

    /// Check if this state was already visited in the given list.
    #[inline]
    pub fn in_list(&self, listid: c_int, recursive: bool) -> bool {
        let idx = if recursive { 1 } else { 0 };
        self.lastlist[idx] == listid
    }

    /// Mark this state as visited in the given list.
    #[inline]
    pub fn mark_in_list(&mut self, listid: c_int, recursive: bool) {
        let idx = if recursive { 1 } else { 0 };
        self.lastlist[idx] = listid;
    }
}

/// Line position structure (matches lpos_T in C).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LPos {
    /// Line number (1-based).
    pub lnum: LineNr,
    /// Column number (0-based byte offset).
    pub col: ColNr,
}

impl LPos {
    /// Create a new line position.
    #[inline]
    pub const fn new(lnum: LineNr, col: ColNr) -> Self {
        Self { lnum, col }
    }

    /// Check if this position is before another.
    #[inline]
    pub fn is_before(&self, other: &LPos) -> bool {
        self.lnum < other.lnum || (self.lnum == other.lnum && self.col < other.col)
    }

    /// Check if this position is after another.
    #[inline]
    pub fn is_after(&self, other: &LPos) -> bool {
        self.lnum > other.lnum || (self.lnum == other.lnum && self.col > other.col)
    }
}

/// Multi-line submatch position.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MultiPos {
    /// Start line number.
    pub start_lnum: LineNr,
    /// End line number.
    pub end_lnum: LineNr,
    /// Start column.
    pub start_col: ColNr,
    /// End column.
    pub end_col: ColNr,
}

/// Single-line submatch position.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LinePos {
    /// Start pointer.
    pub start: *mut u8,
    /// End pointer.
    pub end: *mut u8,
}

/// Submatch positions union (multi-line or single-line).
#[repr(C)]
pub union SubPos {
    /// Multi-line positions.
    pub multi: [MultiPos; NSUBEXP],
    /// Single-line positions.
    pub line: [LinePos; NSUBEXP],
}

/// Submatch tracking structure.
#[repr(C)]
pub struct RegSub {
    /// Number of subexpressions with useful info.
    pub in_use: c_int,
    /// Submatch positions (multi-line or single-line).
    pub list: SubPos,
    /// Original start column without \zs adjustment.
    pub orig_start_col: ColNr,
}

impl Default for RegSub {
    fn default() -> Self {
        Self {
            in_use: 0,
            list: SubPos {
                multi: [MultiPos::default(); NSUBEXP],
            },
            orig_start_col: 0,
        }
    }
}

/// Combined submatch structures for normal and syntax subexpressions.
#[repr(C)]
#[derive(Default)]
pub struct RegSubs {
    /// Normal \( .. \) matches.
    pub norm: RegSub,
    /// Syntax \z( .. \) matches.
    pub synt: RegSub,
}

/// Postponed Invisible Match position union.
#[repr(C)]
#[derive(Clone, Copy)]
pub union PimEnd {
    /// Position for multi-line matching.
    pub pos: LPos,
    /// Pointer for single-line matching.
    pub ptr: *mut u8,
}

/// Postponed Invisible Match (PIM) structure.
///
/// Used for lookahead and lookbehind assertions that need to be
/// evaluated after the main match has progressed.
#[repr(C)]
pub struct NfaPim {
    /// Result of the PIM: NFA_PIM_UNUSED, NFA_PIM_TODO, NFA_PIM_MATCH, or NFA_PIM_NOMATCH.
    pub result: c_int,
    /// The invisible match start state.
    pub state: *mut NfaState,
    /// Submatch info (partially used).
    pub subs: RegSubs,
    /// Where the match must end.
    pub end: PimEnd,
}

impl Default for NfaPim {
    fn default() -> Self {
        Self {
            result: NFA_PIM_UNUSED,
            state: ptr::null_mut(),
            subs: RegSubs::default(),
            end: PimEnd {
                ptr: ptr::null_mut(),
            },
        }
    }
}

impl NfaPim {
    /// Check if this PIM is in use.
    #[inline]
    pub fn is_used(&self) -> bool {
        self.result != NFA_PIM_UNUSED
    }

    /// Check if this PIM needs to be executed.
    #[inline]
    pub fn is_todo(&self) -> bool {
        self.result == NFA_PIM_TODO
    }

    /// Check if this PIM has matched.
    #[inline]
    pub fn has_matched(&self) -> bool {
        self.result == NFA_PIM_MATCH
    }

    /// Mark this PIM as unused.
    #[inline]
    pub fn clear(&mut self) {
        self.result = NFA_PIM_UNUSED;
        self.state = ptr::null_mut();
    }
}

/// NFA execution thread.
///
/// Contains the state and submatch info for one path through the NFA
/// during parallel simulation.
#[repr(C)]
pub struct NfaThread {
    /// Current NFA state.
    pub state: *mut NfaState,
    /// Visit count (for cycle detection).
    pub count: c_int,
    /// Postponed invisible match (if result != NFA_PIM_UNUSED).
    pub pim: NfaPim,
    /// Submatch info (partially used).
    pub subs: RegSubs,
}

impl Default for NfaThread {
    fn default() -> Self {
        Self {
            state: ptr::null_mut(),
            count: 0,
            pim: NfaPim::default(),
            subs: RegSubs::default(),
        }
    }
}

/// List of NFA execution threads.
///
/// The NFA engine maintains two lists: one for the current input position
/// and one for the next position. States are added to the "next" list
/// when they can consume the current character.
#[repr(C)]
pub struct NfaList {
    /// Allocated array of threads.
    pub t: *mut NfaThread,
    /// Number of threads currently in the list.
    pub n: c_int,
    /// Maximum capacity of the thread array.
    pub len: c_int,
    /// List ID (incremented to track visited states).
    pub id: c_int,
    /// True if any thread has a PIM.
    pub has_pim: c_int,
}

impl NfaList {
    /// Check if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Check if the list is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.n >= self.len
    }

    /// Get the number of threads in the list.
    #[inline]
    pub fn count(&self) -> c_int {
        self.n
    }

    /// Clear the list (set count to 0, keep capacity).
    #[inline]
    pub fn clear(&mut self) {
        self.n = 0;
        self.has_pim = 0;
    }
}

// =============================================================================
// NFA Fragment for Pattern Compilation
// =============================================================================

/// Pointer list for patching NFA fragments.
///
/// During NFA construction, we maintain lists of outgoing pointers
/// that need to be patched to point to subsequent states.
#[repr(C)]
pub union Ptrlist {
    /// Next pointer in the list.
    pub next: *mut Ptrlist,
    /// State pointer (when list element is used).
    pub s: *mut NfaState,
}

/// A partially built NFA fragment.
///
/// Frag.start points to the start state.
/// Frag.out is a list of places that need to be connected
/// to the next state for this fragment.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Frag {
    /// Start state of the fragment.
    pub start: *mut NfaState,
    /// List of outgoing pointers to patch.
    pub out: *mut Ptrlist,
}

impl Frag {
    /// Create a new fragment.
    #[inline]
    pub const fn new(start: *mut NfaState, out: *mut Ptrlist) -> Self {
        Self { start, out }
    }

    /// Create an empty fragment.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            start: ptr::null_mut(),
            out: ptr::null_mut(),
        }
    }

    /// Check if this fragment is empty/invalid.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start.is_null()
    }
}

impl Default for Frag {
    fn default() -> Self {
        Self::empty()
    }
}

// =============================================================================
// FFI Declarations for C Accessors
// =============================================================================

use std::ffi::c_char;

extern "C" {
    /// Allocate memory using Neovim's allocator.
    /// Note: Returns *mut c_char to match existing declaration in lib.rs.
    fn xmalloc(size: usize) -> *mut c_char;

    /// Reallocate memory using Neovim's allocator.
    fn xrealloc(ptr: *mut c_char, size: usize) -> *mut c_char;

    /// Free memory using Neovim's allocator.
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// NFA List Management Functions
// =============================================================================

/// Initialize an NFA list with the given capacity.
///
/// # Safety
/// The returned list must be freed with `nfa_list_free`.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_init(capacity: c_int) -> *mut NfaList {
    let list = xmalloc(std::mem::size_of::<NfaList>()).cast::<NfaList>();
    if list.is_null() {
        return ptr::null_mut();
    }

    let t = if capacity > 0 {
        xmalloc(std::mem::size_of::<NfaThread>() * capacity as usize).cast::<NfaThread>()
    } else {
        ptr::null_mut()
    };

    (*list).t = t;
    (*list).n = 0;
    (*list).len = if t.is_null() && capacity > 0 {
        0
    } else {
        capacity
    };
    (*list).id = 1;
    (*list).has_pim = 0;

    list
}

/// Free an NFA list and its thread array.
///
/// # Safety
/// `list` must be a valid pointer returned by `rs_nfa_list_init`.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_free(list: *mut NfaList) {
    if !list.is_null() {
        if !(*list).t.is_null() {
            xfree((*list).t.cast::<c_void>());
        }
        xfree(list.cast::<c_void>());
    }
}

/// Clear an NFA list (reset count, keep capacity).
///
/// # Safety
/// `list` must be a valid NfaList pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_clear(list: *mut NfaList) {
    if !list.is_null() {
        (*list).n = 0;
        (*list).has_pim = 0;
    }
}

/// Grow an NFA list to accommodate more threads.
///
/// Returns true on success, false on allocation failure.
///
/// # Safety
/// `list` must be a valid NfaList pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_grow(list: *mut NfaList, min_capacity: c_int) -> c_int {
    if list.is_null() {
        return 0;
    }

    if (*list).len >= min_capacity {
        return 1; // Already big enough
    }

    // Grow by doubling or to min_capacity, whichever is larger
    let new_len = std::cmp::max((*list).len * 2, min_capacity);
    let new_size = std::mem::size_of::<NfaThread>() * new_len as usize;

    let new_t = if (*list).t.is_null() {
        xmalloc(new_size).cast::<NfaThread>()
    } else {
        xrealloc((*list).t.cast::<c_char>(), new_size).cast::<NfaThread>()
    };

    if new_t.is_null() {
        return 0;
    }

    (*list).t = new_t;
    (*list).len = new_len;
    1
}

/// Get the current count of threads in a list.
///
/// # Safety
/// `list` must be a valid NfaList pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_count(list: *const NfaList) -> c_int {
    if list.is_null() {
        0
    } else {
        (*list).n
    }
}

/// Get a thread from the list by index.
///
/// # Safety
/// `list` must be a valid NfaList pointer and `index` must be < n.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_get(list: *mut NfaList, index: c_int) -> *mut NfaThread {
    if list.is_null() || index < 0 || index >= (*list).n {
        return ptr::null_mut();
    }
    (*list).t.add(index as usize)
}

/// Increment the list ID (used to track visited states).
///
/// # Safety
/// `list` must be a valid NfaList pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_list_next_id(list: *mut NfaList) -> c_int {
    if list.is_null() {
        return 0;
    }
    (*list).id += 1;
    (*list).id
}

// =============================================================================
// Ptrlist Operations for NFA Construction
// =============================================================================

/// Create a singleton pointer list containing just one output pointer.
///
/// # Safety
/// `outp` must be a valid pointer to a state pointer field.
#[no_mangle]
pub unsafe extern "C" fn rs_ptrlist_single(outp: *mut *mut NfaState) -> *mut Ptrlist {
    let l = outp.cast::<Ptrlist>();
    (*l).next = ptr::null_mut();
    l
}

/// Patch all pointers in the list to point to the given state.
///
/// # Safety
/// `l` must be a valid Ptrlist or null. `s` must be a valid state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ptrlist_patch(mut l: *mut Ptrlist, s: *mut NfaState) {
    while !l.is_null() {
        let next = (*l).next;
        (*l).s = s;
        l = next;
    }
}

/// Append two pointer lists, returning the combined list.
///
/// # Safety
/// Both `l1` and `l2` must be valid Ptrlist pointers or null.
#[no_mangle]
pub unsafe extern "C" fn rs_ptrlist_append(l1: *mut Ptrlist, l2: *mut Ptrlist) -> *mut Ptrlist {
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
// NFA State Helpers
// =============================================================================

/// Check if an NFA state is a character class state (includes NL variants).
#[no_mangle]
pub extern "C" fn rs_nfa_is_char_class(c: c_int) -> c_int {
    let base = if (NFA_FIRST_NL..=NFA_LAST_NL).contains(&c) {
        c - NFA_ADD_NL
    } else {
        c
    };

    c_int::from((NFA_ANY..=NFA_NUPPER_IC).contains(&base))
}

/// Check if an NFA state is a position match state (line/column/mark).
#[no_mangle]
pub extern "C" fn rs_nfa_is_position_match(c: c_int) -> c_int {
    c_int::from((NFA_CURSOR..=NFA_VISUAL).contains(&c))
}

/// Check if an NFA state is a POSIX character class state.
#[no_mangle]
pub extern "C" fn rs_nfa_is_posix_class(c: c_int) -> c_int {
    c_int::from((NFA_CLASS_ALNUM..=NFA_CLASS_FNAME).contains(&c))
}

/// Get the subexpression index from an MOPEN/MCLOSE/ZOPEN/ZCLOSE state.
/// Returns -1 if not a subexpression state.
#[no_mangle]
pub extern "C" fn rs_nfa_get_subexpr_idx(state: c_int) -> c_int {
    nfa_get_subexpr_num(state)
}

// =============================================================================
// Character Class Recognition
// =============================================================================

// Flags for nfa_recognize_char_class
const CLASS_NOT: u8 = 0x80;
const CLASS_AF: u8 = 0x40;
const CLASS_AF_UPPER: u8 = 0x20;
const CLASS_AZ: u8 = 0x10;
const CLASS_AZ_UPPER: u8 = 0x08;
const CLASS_O7: u8 = 0x04;
const CLASS_O9: u8 = 0x02;
const CLASS_UNDERSCORE: u8 = 0x01;

/// FAIL value (matches vim_defs.h)
const FAIL: c_int = 0;

/// Recognize a character class in expanded form like [0-9] or [a-zA-Z_0-9].
///
/// On success, return the NFA state id for the character class.
/// On failure, return FAIL (0).
///
/// # Arguments
/// * `start` - Points to the first char inside the brackets (after '[')
/// * `end` - Points to the closing bracket ']'
/// * `extra_newl` - If true, add NFA_ADD_NL to result
///
/// # Safety
/// `start` and `end` must be valid pointers within the same buffer.
pub unsafe fn nfa_recognize_char_class_impl(
    start: *const u8,
    end: *const u8,
    extra_newl: c_int,
) -> c_int {
    if end.is_null() || start.is_null() {
        return FAIL;
    }

    if *end != b']' {
        return FAIL;
    }

    let mut config: u8 = 0;
    let mut newl = extra_newl != 0;
    let mut p = start;

    // Check for negation
    if *p == b'^' {
        config |= CLASS_NOT;
        p = p.add(1);
    }

    while p < end {
        // Check for range pattern: x-y
        if p.add(2) < end && *p.add(1) == b'-' {
            match *p {
                b'0' => {
                    let end_char = *p.add(2);
                    if end_char == b'9' {
                        config |= CLASS_O9;
                    } else if end_char == b'7' {
                        config |= CLASS_O7;
                    } else {
                        return FAIL;
                    }
                }
                b'a' => {
                    let end_char = *p.add(2);
                    if end_char == b'z' {
                        config |= CLASS_AZ;
                    } else if end_char == b'f' {
                        config |= CLASS_AF;
                    } else {
                        return FAIL;
                    }
                }
                b'A' => {
                    let end_char = *p.add(2);
                    if end_char == b'Z' {
                        config |= CLASS_AZ_UPPER;
                    } else if end_char == b'F' {
                        config |= CLASS_AF_UPPER;
                    } else {
                        return FAIL;
                    }
                }
                _ => return FAIL,
            }
            p = p.add(3);
        } else if p.add(1) < end && *p == b'\\' && *p.add(1) == b'n' {
            newl = true;
            p = p.add(2);
        } else if *p == b'_' {
            config |= CLASS_UNDERSCORE;
            p = p.add(1);
        } else if *p == b'\n' {
            newl = true;
            p = p.add(1);
        } else {
            return FAIL;
        }
    }

    if p != end {
        return FAIL;
    }

    let extra = if newl { NFA_ADD_NL } else { 0 };

    match config {
        CLASS_O9 => extra + NFA_DIGIT,
        c if c == CLASS_NOT | CLASS_O9 => extra + NFA_NDIGIT,
        c if c == CLASS_AF | CLASS_AF_UPPER | CLASS_O9 => extra + NFA_HEX,
        c if c == CLASS_NOT | CLASS_AF | CLASS_AF_UPPER | CLASS_O9 => extra + NFA_NHEX,
        CLASS_O7 => extra + NFA_OCTAL,
        c if c == CLASS_NOT | CLASS_O7 => extra + NFA_NOCTAL,
        c if c == CLASS_AZ | CLASS_AZ_UPPER | CLASS_O9 | CLASS_UNDERSCORE => extra + NFA_WORD,
        c if c == CLASS_NOT | CLASS_AZ | CLASS_AZ_UPPER | CLASS_O9 | CLASS_UNDERSCORE => {
            extra + NFA_NWORD
        }
        c if c == CLASS_AZ | CLASS_AZ_UPPER | CLASS_UNDERSCORE => extra + NFA_HEAD,
        c if c == CLASS_NOT | CLASS_AZ | CLASS_AZ_UPPER | CLASS_UNDERSCORE => extra + NFA_NHEAD,
        c if c == CLASS_AZ | CLASS_AZ_UPPER => extra + NFA_ALPHA,
        c if c == CLASS_NOT | CLASS_AZ | CLASS_AZ_UPPER => extra + NFA_NALPHA,
        CLASS_AZ => extra + NFA_LOWER_IC,
        c if c == CLASS_NOT | CLASS_AZ => extra + NFA_NLOWER_IC,
        CLASS_AZ_UPPER => extra + NFA_UPPER_IC,
        c if c == CLASS_NOT | CLASS_AZ_UPPER => extra + NFA_NUPPER_IC,
        _ => FAIL,
    }
}

/// FFI export: Recognize a character class in expanded form.
///
/// # Safety
/// `start` and `end` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_recognize_char_class(
    start: *const u8,
    end: *const u8,
    extra_newl: c_int,
) -> c_int {
    nfa_recognize_char_class_impl(start, end, extra_newl)
}

// =============================================================================
// Character Class Membership Checking
// =============================================================================

// FFI declarations for character classification helpers
extern "C" {
    /// Check if character is a valid identifier character.
    fn vim_isIDc(c: c_int) -> c_int;

    /// Check if character is a keyword character for the current buffer.
    fn vim_iswordc_buf(c: c_int, buf: *mut std::ffi::c_void) -> c_int;

    /// Check if character is valid in a file name.
    fn vim_isfilec(c: c_int) -> c_int;

    /// Check if character is lowercase (multibyte aware).
    fn mb_islower(c: c_int) -> c_int;

    /// Check if character is uppercase (multibyte aware).
    fn mb_isupper(c: c_int) -> c_int;

    /// Check if character is printable.
    fn vim_isprintc(c: c_int) -> c_int;

    /// Get the current regex buffer (rex.reg_buf).
    fn nvim_rex_get_reg_buf() -> *mut std::ffi::c_void;
}

/// Result codes
const OK: c_int = 1;

/// Escape character (ASCII 27)
const ESC: c_int = 27;

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: c_int) -> bool {
    c >= b'0' as c_int && c <= b'9' as c_int
}

/// Check if a character is an ASCII hex digit (0-9, a-f, A-F).
#[inline]
const fn ascii_isxdigit(c: c_int) -> bool {
    ascii_isdigit(c)
        || (c >= b'a' as c_int && c <= b'f' as c_int)
        || (c >= b'A' as c_int && c <= b'F' as c_int)
}

/// Check if a character belongs to a POSIX character class.
///
/// # Arguments
/// * `cls` - The NFA_CLASS_* constant
/// * `c` - The character to check
///
/// # Returns
/// * `OK` (1) if the character is in the class
/// * `FAIL` (0) if not
///
/// # Safety
/// For NFA_CLASS_KEYWORD, requires rex.reg_buf to be valid.
pub unsafe fn check_char_class_impl(cls: c_int, c: c_int) -> c_int {
    match cls {
        NFA_CLASS_ALNUM => {
            if (1..128).contains(&c) && libc::isalnum(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_ALPHA => {
            if (1..128).contains(&c) && libc::isalpha(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_BLANK => {
            if c == b' ' as c_int || c == b'\t' as c_int {
                return OK;
            }
        }
        NFA_CLASS_CNTRL => {
            if (1..=127).contains(&c) && libc::iscntrl(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_DIGIT => {
            if ascii_isdigit(c) {
                return OK;
            }
        }
        NFA_CLASS_GRAPH => {
            if (1..=127).contains(&c) && libc::isgraph(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_LOWER => {
            // Exclude special characters 170 and 186 per C implementation
            if mb_islower(c) != 0 && c != 170 && c != 186 {
                return OK;
            }
        }
        NFA_CLASS_PRINT => {
            if vim_isprintc(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_PUNCT => {
            if (1..128).contains(&c) && libc::ispunct(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_SPACE => {
            // Tab (9), newline (10), vertical tab (11), form feed (12), carriage return (13), space
            if (9..=13).contains(&c) || c == b' ' as c_int {
                return OK;
            }
        }
        NFA_CLASS_UPPER => {
            if mb_isupper(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_XDIGIT => {
            if ascii_isxdigit(c) {
                return OK;
            }
        }
        NFA_CLASS_TAB => {
            if c == b'\t' as c_int {
                return OK;
            }
        }
        NFA_CLASS_RETURN => {
            if c == b'\r' as c_int {
                return OK;
            }
        }
        NFA_CLASS_BACKSPACE => {
            if c == 8 {
                // '\b' = ASCII 8
                return OK;
            }
        }
        NFA_CLASS_ESCAPE => {
            if c == ESC {
                return OK;
            }
        }
        NFA_CLASS_IDENT => {
            if vim_isIDc(c) != 0 {
                return OK;
            }
        }
        NFA_CLASS_KEYWORD => {
            let buf = nvim_rex_get_reg_buf();
            if vim_iswordc_buf(c, buf) != 0 {
                return OK;
            }
        }
        NFA_CLASS_FNAME => {
            if vim_isfilec(c) != 0 {
                return OK;
            }
        }
        _ => {
            // Invalid class - return FAIL
            return FAIL;
        }
    }
    FAIL
}

/// FFI export: Check if a character belongs to a POSIX character class.
///
/// # Safety
/// For NFA_CLASS_KEYWORD, requires rex.reg_buf to be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_check_char_class(cls: c_int, c: c_int) -> c_int {
    check_char_class_impl(cls, c)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa_split_value() {
        assert_eq!(NFA_SPLIT, -1024);
    }

    #[test]
    fn test_nfa_sequential_values() {
        // Verify sequential ordering
        assert_eq!(NFA_MATCH, NFA_SPLIT + 1);
        assert_eq!(NFA_EMPTY, NFA_MATCH + 1);
        assert_eq!(NFA_START_COLL, NFA_EMPTY + 1);
    }

    #[test]
    fn test_nfa_with_nl() {
        assert!(!nfa_with_nl(NFA_ANY));
        assert!(!nfa_with_nl(NFA_DIGIT));
        assert!(nfa_with_nl(NFA_FIRST_NL));
        assert!(nfa_with_nl(NFA_LAST_NL));
        assert!(nfa_with_nl(NFA_ANY + NFA_ADD_NL));
    }

    #[test]
    fn test_nfa_add_remove_nl() {
        assert_eq!(nfa_add_nl(NFA_ANY), NFA_ANY + NFA_ADD_NL);
        assert_eq!(nfa_remove_nl(NFA_ANY + NFA_ADD_NL), NFA_ANY);
    }

    #[test]
    fn test_nfa_mopen_mclose() {
        assert_eq!(nfa_mopen(0), NFA_MOPEN);
        assert_eq!(nfa_mopen(1), NFA_MOPEN1);
        assert_eq!(nfa_mopen(9), NFA_MOPEN9);

        assert_eq!(nfa_mclose(0), NFA_MCLOSE);
        assert_eq!(nfa_mclose(1), NFA_MCLOSE1);
        assert_eq!(nfa_mclose(9), NFA_MCLOSE9);
    }

    #[test]
    fn test_is_nfa_mopen_mclose() {
        assert!(is_nfa_mopen(NFA_MOPEN));
        assert!(is_nfa_mopen(NFA_MOPEN5));
        assert!(is_nfa_mopen(NFA_MOPEN9));
        assert!(!is_nfa_mopen(NFA_MCLOSE));
        assert!(!is_nfa_mopen(NFA_ANY));

        assert!(is_nfa_mclose(NFA_MCLOSE));
        assert!(is_nfa_mclose(NFA_MCLOSE5));
        assert!(is_nfa_mclose(NFA_MCLOSE9));
        assert!(!is_nfa_mclose(NFA_MOPEN));
    }

    #[test]
    fn test_nfa_backref() {
        assert_eq!(nfa_backref(1), NFA_BACKREF1);
        assert_eq!(nfa_backref(5), NFA_BACKREF5);
        assert_eq!(nfa_backref(9), NFA_BACKREF9);
    }

    #[test]
    fn test_nfa_pim_states() {
        assert_eq!(NFA_PIM_UNUSED, 0);
        assert_eq!(NFA_PIM_TODO, 1);
        assert_eq!(NFA_PIM_MATCH, 2);
        assert_eq!(NFA_PIM_NOMATCH, 3);
    }

    #[test]
    fn test_nfa_add_nl_constant() {
        assert_eq!(NFA_ADD_NL, 31);
    }

    #[test]
    fn test_nfa_class_values() {
        // Just verify some key class values are defined and in sequence
        // Using variables to avoid assertions_on_constants lint
        let class_alnum = NFA_CLASS_ALNUM;
        let visual = NFA_VISUAL;
        assert!(class_alnum > visual);
        assert_eq!(NFA_CLASS_ALPHA, NFA_CLASS_ALNUM + 1);
        assert_eq!(NFA_CLASS_FNAME, NFA_CLASS_KEYWORD + 1);
    }

    // =========================================================================
    // nfa_recognize_char_class tests
    // =========================================================================

    /// Helper to test nfa_recognize_char_class with a string
    unsafe fn test_char_class(s: &[u8], extra_newl: c_int) -> c_int {
        if s.is_empty() {
            return FAIL;
        }
        let start = s.as_ptr();
        let end = s.as_ptr().add(s.len() - 1); // Point to ']'
        nfa_recognize_char_class_impl(start, end, extra_newl)
    }

    #[test]
    fn test_recognize_digit() {
        unsafe {
            // [0-9] -> NFA_DIGIT
            assert_eq!(test_char_class(b"0-9]", 0), NFA_DIGIT);
            // [^0-9] -> NFA_NDIGIT
            assert_eq!(test_char_class(b"^0-9]", 0), NFA_NDIGIT);
        }
    }

    #[test]
    fn test_recognize_octal() {
        unsafe {
            // [0-7] -> NFA_OCTAL
            assert_eq!(test_char_class(b"0-7]", 0), NFA_OCTAL);
            // [^0-7] -> NFA_NOCTAL
            assert_eq!(test_char_class(b"^0-7]", 0), NFA_NOCTAL);
        }
    }

    #[test]
    fn test_recognize_hex() {
        unsafe {
            // [0-9a-fA-F] -> NFA_HEX
            assert_eq!(test_char_class(b"0-9a-fA-F]", 0), NFA_HEX);
            // [^0-9a-fA-F] -> NFA_NHEX
            assert_eq!(test_char_class(b"^0-9a-fA-F]", 0), NFA_NHEX);
        }
    }

    #[test]
    fn test_recognize_word() {
        unsafe {
            // [a-zA-Z0-9_] -> NFA_WORD
            assert_eq!(test_char_class(b"a-zA-Z0-9_]", 0), NFA_WORD);
            // [^a-zA-Z0-9_] -> NFA_NWORD
            assert_eq!(test_char_class(b"^a-zA-Z0-9_]", 0), NFA_NWORD);
        }
    }

    #[test]
    fn test_recognize_head() {
        unsafe {
            // [a-zA-Z_] -> NFA_HEAD
            assert_eq!(test_char_class(b"a-zA-Z_]", 0), NFA_HEAD);
            // [^a-zA-Z_] -> NFA_NHEAD
            assert_eq!(test_char_class(b"^a-zA-Z_]", 0), NFA_NHEAD);
        }
    }

    #[test]
    fn test_recognize_alpha() {
        unsafe {
            // [a-zA-Z] -> NFA_ALPHA
            assert_eq!(test_char_class(b"a-zA-Z]", 0), NFA_ALPHA);
            // [^a-zA-Z] -> NFA_NALPHA
            assert_eq!(test_char_class(b"^a-zA-Z]", 0), NFA_NALPHA);
        }
    }

    #[test]
    fn test_recognize_lower_upper() {
        unsafe {
            // [a-z] -> NFA_LOWER_IC
            assert_eq!(test_char_class(b"a-z]", 0), NFA_LOWER_IC);
            // [^a-z] -> NFA_NLOWER_IC
            assert_eq!(test_char_class(b"^a-z]", 0), NFA_NLOWER_IC);
            // [A-Z] -> NFA_UPPER_IC
            assert_eq!(test_char_class(b"A-Z]", 0), NFA_UPPER_IC);
            // [^A-Z] -> NFA_NUPPER_IC
            assert_eq!(test_char_class(b"^A-Z]", 0), NFA_NUPPER_IC);
        }
    }

    #[test]
    fn test_recognize_with_newline() {
        unsafe {
            // [0-9] with extra_newl = 1 -> NFA_DIGIT + NFA_ADD_NL
            assert_eq!(test_char_class(b"0-9]", 1), NFA_DIGIT + NFA_ADD_NL);
            // [0-9\n] -> NFA_DIGIT + NFA_ADD_NL
            assert_eq!(test_char_class(b"0-9\\n]", 0), NFA_DIGIT + NFA_ADD_NL);
        }
    }

    #[test]
    fn test_recognize_invalid() {
        unsafe {
            // Invalid: just letters
            assert_eq!(test_char_class(b"abc]", 0), FAIL);
            // Invalid: no closing bracket
            assert_eq!(test_char_class(b"0-9", 0), FAIL);
            // Invalid: wrong range
            assert_eq!(test_char_class(b"0-5]", 0), FAIL);
        }
    }

    #[test]
    fn test_recognize_char_class_flags() {
        // Verify flag constants
        assert_eq!(CLASS_NOT, 0x80);
        assert_eq!(CLASS_AF, 0x40);
        assert_eq!(CLASS_AF_UPPER, 0x20);
        assert_eq!(CLASS_AZ, 0x10);
        assert_eq!(CLASS_AZ_UPPER, 0x08);
        assert_eq!(CLASS_O7, 0x04);
        assert_eq!(CLASS_O9, 0x02);
        assert_eq!(CLASS_UNDERSCORE, 0x01);
    }

    // =========================================================================
    // NFA State Machine Structure Tests
    // =========================================================================

    #[test]
    fn test_nsubexp_constant() {
        assert_eq!(NSUBEXP, 10);
    }

    #[test]
    fn test_nfa_state_handle() {
        let null_handle = NfaStateHandle::null();
        assert!(null_handle.is_null());
        assert_eq!(null_handle, NfaStateHandle::default());

        let mut dummy: i32 = 42;
        let ptr = (&mut dummy as *mut i32).cast::<c_void>();
        let handle = NfaStateHandle::from_ptr(ptr);
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), ptr);
    }

    #[test]
    fn test_lpos_methods() {
        let pos1 = LPos::new(1, 5);
        let pos2 = LPos::new(1, 10);
        let pos3 = LPos::new(2, 0);

        assert!(pos1.is_before(&pos2));
        assert!(pos2.is_before(&pos3));
        assert!(!pos2.is_before(&pos1));

        assert!(pos2.is_after(&pos1));
        assert!(pos3.is_after(&pos2));
        assert!(!pos1.is_after(&pos2));

        // Same position
        let pos4 = LPos::new(1, 5);
        assert!(!pos1.is_before(&pos4));
        assert!(!pos1.is_after(&pos4));
    }

    #[test]
    fn test_nfa_pim_methods() {
        let mut pim = NfaPim::default();
        assert!(!pim.is_used());
        assert!(!pim.is_todo());
        assert!(!pim.has_matched());

        pim.result = NFA_PIM_TODO;
        assert!(pim.is_used());
        assert!(pim.is_todo());
        assert!(!pim.has_matched());

        pim.result = NFA_PIM_MATCH;
        assert!(pim.is_used());
        assert!(!pim.is_todo());
        assert!(pim.has_matched());

        pim.clear();
        assert!(!pim.is_used());
        assert!(pim.state.is_null());
    }

    #[test]
    fn test_frag_methods() {
        let empty = Frag::empty();
        assert!(empty.is_empty());
        assert!(empty.start.is_null());
        assert!(empty.out.is_null());

        let default = Frag::default();
        assert!(default.is_empty());
    }

    #[test]
    fn test_is_nfa_zopen_zclose() {
        assert!(is_nfa_zopen(NFA_ZOPEN));
        assert!(is_nfa_zopen(NFA_ZOPEN5));
        assert!(is_nfa_zopen(NFA_ZOPEN9));
        assert!(!is_nfa_zopen(NFA_MOPEN));
        assert!(!is_nfa_zopen(NFA_ZCLOSE));

        assert!(is_nfa_zclose(NFA_ZCLOSE));
        assert!(is_nfa_zclose(NFA_ZCLOSE5));
        assert!(is_nfa_zclose(NFA_ZCLOSE9));
        assert!(!is_nfa_zclose(NFA_MCLOSE));
        assert!(!is_nfa_zclose(NFA_ZOPEN));
    }

    #[test]
    fn test_nfa_get_subexpr_num() {
        // MOPEN states
        assert_eq!(nfa_get_subexpr_num(NFA_MOPEN), 0);
        assert_eq!(nfa_get_subexpr_num(NFA_MOPEN1), 1);
        assert_eq!(nfa_get_subexpr_num(NFA_MOPEN9), 9);

        // MCLOSE states
        assert_eq!(nfa_get_subexpr_num(NFA_MCLOSE), 0);
        assert_eq!(nfa_get_subexpr_num(NFA_MCLOSE5), 5);
        assert_eq!(nfa_get_subexpr_num(NFA_MCLOSE9), 9);

        // ZOPEN states
        assert_eq!(nfa_get_subexpr_num(NFA_ZOPEN), 0);
        assert_eq!(nfa_get_subexpr_num(NFA_ZOPEN3), 3);

        // ZCLOSE states
        assert_eq!(nfa_get_subexpr_num(NFA_ZCLOSE), 0);
        assert_eq!(nfa_get_subexpr_num(NFA_ZCLOSE7), 7);

        // Non-subexpr state
        assert_eq!(nfa_get_subexpr_num(NFA_SPLIT), -1);
        assert_eq!(nfa_get_subexpr_num(NFA_ANY), -1);
    }

    #[test]
    fn test_rs_nfa_is_char_class() {
        // Basic char classes
        assert_eq!(rs_nfa_is_char_class(NFA_ANY), 1);
        assert_eq!(rs_nfa_is_char_class(NFA_DIGIT), 1);
        assert_eq!(rs_nfa_is_char_class(NFA_WORD), 1);
        assert_eq!(rs_nfa_is_char_class(NFA_NUPPER_IC), 1);

        // NL variants
        assert_eq!(rs_nfa_is_char_class(NFA_ANY + NFA_ADD_NL), 1);
        assert_eq!(rs_nfa_is_char_class(NFA_DIGIT + NFA_ADD_NL), 1);

        // Non-char class states
        assert_eq!(rs_nfa_is_char_class(NFA_SPLIT), 0);
        assert_eq!(rs_nfa_is_char_class(NFA_MATCH), 0);
        assert_eq!(rs_nfa_is_char_class(NFA_CURSOR), 0);
    }

    #[test]
    fn test_rs_nfa_is_position_match() {
        assert_eq!(rs_nfa_is_position_match(NFA_CURSOR), 1);
        assert_eq!(rs_nfa_is_position_match(NFA_LNUM), 1);
        assert_eq!(rs_nfa_is_position_match(NFA_COL), 1);
        assert_eq!(rs_nfa_is_position_match(NFA_VCOL), 1);
        assert_eq!(rs_nfa_is_position_match(NFA_MARK), 1);
        assert_eq!(rs_nfa_is_position_match(NFA_VISUAL), 1);

        // Non-position states
        assert_eq!(rs_nfa_is_position_match(NFA_ANY), 0);
        assert_eq!(rs_nfa_is_position_match(NFA_SPLIT), 0);
    }

    #[test]
    fn test_rs_nfa_is_posix_class() {
        assert_eq!(rs_nfa_is_posix_class(NFA_CLASS_ALNUM), 1);
        assert_eq!(rs_nfa_is_posix_class(NFA_CLASS_DIGIT), 1);
        assert_eq!(rs_nfa_is_posix_class(NFA_CLASS_SPACE), 1);
        assert_eq!(rs_nfa_is_posix_class(NFA_CLASS_FNAME), 1);

        // Non-POSIX states
        assert_eq!(rs_nfa_is_posix_class(NFA_ANY), 0);
        assert_eq!(rs_nfa_is_posix_class(NFA_DIGIT), 0);
    }
}
