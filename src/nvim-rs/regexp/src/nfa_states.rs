//! NFA regex engine state types.
//!
//! This module defines the state types used by the NFA regex engine.
//! The NFA engine uses a parallel state tracking approach rather than
//! the backtracking used by the BT engine.

use std::ffi::c_int;

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
/// Match cursor position.
pub const NFA_CURSOR: c_int = NFA_NUPPER_IC + 1;
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
}
