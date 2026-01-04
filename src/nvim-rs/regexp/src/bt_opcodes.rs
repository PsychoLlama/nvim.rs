//! Backtracking (BT) regex engine opcodes.
//!
//! This module defines the opcodes used by the BT regex engine's bytecode.
//! The bytecode format uses a linked list structure where each instruction
//! consists of an opcode byte followed by a 2-byte "next" pointer.

use std::ffi::c_int;

// =============================================================================
// Core Opcodes
// =============================================================================

/// End of program or NOMATCH operand.
pub const END: c_int = 0;

/// Match "" at beginning of line.
pub const BOL: c_int = 1;

/// Match "" at end of line.
pub const EOL: c_int = 2;

/// Match this alternative, or the next.
/// Node has "next" pointing to the next BRANCH.
pub const BRANCH: c_int = 3;

/// Match "", "next" ptr points backward.
pub const BACK: c_int = 4;

/// Match this exact string (operand follows).
pub const EXACTLY: c_int = 5;

/// Match empty string.
pub const NOTHING: c_int = 6;

/// Match this (simple) thing 0 or more times.
pub const STAR: c_int = 7;

/// Match this (simple) thing 1 or more times.
pub const PLUS: c_int = 8;

/// Match the operand zero-width.
pub const MATCH: c_int = 9;

/// Check for no match with operand.
pub const NOMATCH: c_int = 10;

/// Look behind for a match with operand.
pub const BEHIND: c_int = 11;

/// Look behind for no match with operand.
pub const NOBEHIND: c_int = 12;

/// Match the operand here.
pub const SUBPAT: c_int = 13;

/// Match this (simple) thing between m and n times `\{m,n\}`.
pub const BRACE_SIMPLE: c_int = 14;

/// Match "" after `[^a-zA-Z0-9_]` (beginning of word).
pub const BOW: c_int = 15;

/// Match "" at `[^a-zA-Z0-9_]` (end of word).
pub const EOW: c_int = 16;

/// Define the min & max for BRACE_SIMPLE and BRACE_COMPLEX.
/// Operand: nr nr
pub const BRACE_LIMITS: c_int = 17;

/// Match line-break.
pub const NEWL: c_int = 18;

/// End position for BEHIND or NOBEHIND.
pub const BHPOS: c_int = 19;

// =============================================================================
// Character Classes (20-48 normal, 50-78 include line-break)
// =============================================================================

/// Offset to add for NL-including variants.
pub const ADD_NL: c_int = 30;

/// Match any one character.
pub const ANY: c_int = 20;

/// Match any character in this string (operand: str).
pub const ANYOF: c_int = 21;

/// Match any character NOT in this string (operand: str).
pub const ANYBUT: c_int = 22;

/// Match identifier char.
pub const IDENT: c_int = 23;

/// Match identifier char but no digit.
pub const SIDENT: c_int = 24;

/// Match keyword char.
pub const KWORD: c_int = 25;

/// Match word char but no digit.
pub const SKWORD: c_int = 26;

/// Match file name char.
pub const FNAME: c_int = 27;

/// Match file name char but no digit.
pub const SFNAME: c_int = 28;

/// Match printable char.
pub const PRINT: c_int = 29;

/// Match printable char but no digit.
pub const SPRINT: c_int = 30;

/// Match whitespace char.
pub const WHITE: c_int = 31;

/// Match non-whitespace char.
pub const NWHITE: c_int = 32;

/// Match digit char.
pub const DIGIT: c_int = 33;

/// Match non-digit char.
pub const NDIGIT: c_int = 34;

/// Match hex char.
pub const HEX: c_int = 35;

/// Match non-hex char.
pub const NHEX: c_int = 36;

/// Match octal char.
pub const OCTAL: c_int = 37;

/// Match non-octal char.
pub const NOCTAL: c_int = 38;

/// Match word char.
pub const WORD: c_int = 39;

/// Match non-word char.
pub const NWORD: c_int = 40;

/// Match head char (can start identifier).
pub const HEAD: c_int = 41;

/// Match non-head char.
pub const NHEAD: c_int = 42;

/// Match alpha char.
pub const ALPHA: c_int = 43;

/// Match non-alpha char.
pub const NALPHA: c_int = 44;

/// Match lowercase char.
pub const LOWER: c_int = 45;

/// Match non-lowercase char.
pub const NLOWER: c_int = 46;

/// Match uppercase char.
pub const UPPER: c_int = 47;

/// Match non-uppercase char.
pub const NUPPER: c_int = 48;

/// First opcode that includes NL matching.
pub const FIRST_NL: c_int = ANY + ADD_NL;

/// Last opcode that includes NL matching.
pub const LAST_NL: c_int = NUPPER + ADD_NL;

/// Check if opcode includes newline matching.
#[inline]
pub const fn with_nl(op: c_int) -> bool {
    op >= FIRST_NL && op <= LAST_NL
}

// =============================================================================
// Subexpression Markers
// =============================================================================

/// Mark start of `\( ... \)` subexpr (80-89).
/// MOPEN + 0 marks start of match.
pub const MOPEN: c_int = 80;

/// Mark end of `\( ... \)` subexpr (90-99).
/// MCLOSE + 0 marks end of match.
pub const MCLOSE: c_int = 90;

/// Match same string again `\1`-`\9` (100-109).
pub const BACKREF: c_int = 100;

/// Mark start of `\z( ... \)` subexpr (110-119).
pub const ZOPEN: c_int = 110;

/// Mark end of `\z( ... \)` subexpr (120-129).
pub const ZCLOSE: c_int = 120;

/// Match external submatch `\z1`-`\z9` (130-139).
pub const ZREF: c_int = 130;

/// Match nodes between m & n times (140-149).
pub const BRACE_COMPLEX: c_int = 140;

/// Mark start of `\%( ... \)` subexpr.
pub const NOPEN: c_int = 150;

/// Mark end of `\%( ... \)` subexpr.
pub const NCLOSE: c_int = 151;

// =============================================================================
// Special Opcodes
// =============================================================================

/// Match one multi-byte character.
pub const MULTIBYTECODE: c_int = 200;

/// Match "" at beginning of file.
pub const RE_BOF: c_int = 201;

/// Match "" at end of file.
pub const RE_EOF: c_int = 202;

/// Match location of cursor.
pub const CURSOR: c_int = 203;

/// Match line number (operand: nr cmp).
pub const RE_LNUM: c_int = 204;

/// Match column number (operand: nr cmp).
pub const RE_COL: c_int = 205;

/// Match virtual column number (operand: nr cmp).
pub const RE_VCOL: c_int = 206;

/// Match mark position (operand: mark cmp).
pub const RE_MARK: c_int = 207;

/// Match Visual area.
pub const RE_VISUAL: c_int = 208;

/// Match any composing characters.
pub const RE_COMPOSING: c_int = 209;

// =============================================================================
// Flags (passed up and down during compilation)
// =============================================================================

/// Known never to match null string.
pub const HASWIDTH: c_int = 0x1;

/// Simple enough to be STAR/PLUS operand.
pub const SIMPLE: c_int = 0x2;

/// Starts with * or +.
pub const SPSTART: c_int = 0x4;

/// Contains some `\n`.
pub const HASNL: c_int = 0x8;

/// Contains `\@<=` or `\@<!`.
pub const HASLOOKBH: c_int = 0x10;

/// Worst case (no flags).
pub const WORST: c_int = 0;

// =============================================================================
// Stack/Table Sizes
// =============================================================================

/// Initial size of regstack.
pub const REGSTACK_INITIAL: usize = 2048;

/// Initial size of backpos table.
pub const BACKPOS_INITIAL: usize = 64;

// =============================================================================
// Character Class Lookup
// =============================================================================

/// Character class specifiers as a byte slice.
/// Maps to classcodes array by index.
pub const CLASSCHARS: &[u8] = b".iIkKfFpPsSdDxXoOwWhHaAlLuU";

/// Opcodes corresponding to CLASSCHARS.
pub const CLASSCODES: [c_int; 27] = [
    ANY, IDENT, SIDENT, KWORD, SKWORD, FNAME, SFNAME, PRINT, SPRINT, WHITE, NWHITE, DIGIT, NDIGIT,
    HEX, NHEX, OCTAL, NOCTAL, WORD, NWORD, HEAD, NHEAD, ALPHA, NALPHA, LOWER, NLOWER, UPPER,
    NUPPER,
];

/// Look up the opcode for a character class specifier.
///
/// Returns `None` if the character is not a valid class specifier.
#[inline]
pub fn classcode_for_char(c: u8) -> Option<c_int> {
    CLASSCHARS
        .iter()
        .position(|&ch| ch == c)
        .map(|idx| CLASSCODES[idx])
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_values() {
        // Verify core opcode values match C definitions
        assert_eq!(END, 0);
        assert_eq!(BOL, 1);
        assert_eq!(EOL, 2);
        assert_eq!(BRANCH, 3);
        assert_eq!(BACK, 4);
        assert_eq!(EXACTLY, 5);
        assert_eq!(NOTHING, 6);
        assert_eq!(STAR, 7);
        assert_eq!(PLUS, 8);
    }

    #[test]
    fn test_character_class_opcodes() {
        assert_eq!(ANY, 20);
        assert_eq!(ANYOF, 21);
        assert_eq!(ANYBUT, 22);
        assert_eq!(WHITE, 31);
        assert_eq!(DIGIT, 33);
        assert_eq!(UPPER, 47);
        assert_eq!(NUPPER, 48);
    }

    #[test]
    fn test_add_nl() {
        assert_eq!(ADD_NL, 30);
        assert_eq!(FIRST_NL, ANY + ADD_NL);
        assert_eq!(FIRST_NL, 50);
        assert_eq!(LAST_NL, NUPPER + ADD_NL);
        assert_eq!(LAST_NL, 78);
    }

    #[test]
    fn test_with_nl() {
        // Normal opcodes don't include NL
        assert!(!with_nl(ANY));
        assert!(!with_nl(DIGIT));
        assert!(!with_nl(UPPER));

        // NL variants do
        assert!(with_nl(ANY + ADD_NL));
        assert!(with_nl(DIGIT + ADD_NL));
        assert!(with_nl(UPPER + ADD_NL));

        // Boundary cases
        assert!(with_nl(FIRST_NL));
        assert!(with_nl(LAST_NL));
        assert!(!with_nl(FIRST_NL - 1));
        assert!(!with_nl(LAST_NL + 1));
    }

    #[test]
    fn test_subexpr_markers() {
        assert_eq!(MOPEN, 80);
        assert_eq!(MCLOSE, 90);
        assert_eq!(BACKREF, 100);
        assert_eq!(ZOPEN, 110);
        assert_eq!(ZCLOSE, 120);
        assert_eq!(ZREF, 130);
        assert_eq!(BRACE_COMPLEX, 140);
        assert_eq!(NOPEN, 150);
        assert_eq!(NCLOSE, 151);
    }

    #[test]
    fn test_special_opcodes() {
        assert_eq!(MULTIBYTECODE, 200);
        assert_eq!(RE_BOF, 201);
        assert_eq!(RE_EOF, 202);
        assert_eq!(CURSOR, 203);
        assert_eq!(RE_LNUM, 204);
        assert_eq!(RE_COL, 205);
        assert_eq!(RE_VCOL, 206);
        assert_eq!(RE_MARK, 207);
        assert_eq!(RE_VISUAL, 208);
        assert_eq!(RE_COMPOSING, 209);
    }

    #[test]
    fn test_flags() {
        assert_eq!(HASWIDTH, 0x1);
        assert_eq!(SIMPLE, 0x2);
        assert_eq!(SPSTART, 0x4);
        assert_eq!(HASNL, 0x8);
        assert_eq!(HASLOOKBH, 0x10);
        assert_eq!(WORST, 0);

        // Flags can be combined
        let combined = HASWIDTH | SIMPLE | HASNL;
        assert_eq!(combined, 0x1 | 0x2 | 0x8);
    }

    #[test]
    fn test_classchars_classcodes_correspondence() {
        assert_eq!(CLASSCHARS.len(), CLASSCODES.len());

        // Test specific mappings
        assert_eq!(classcode_for_char(b'.'), Some(ANY));
        assert_eq!(classcode_for_char(b'i'), Some(IDENT));
        assert_eq!(classcode_for_char(b'I'), Some(SIDENT));
        assert_eq!(classcode_for_char(b'd'), Some(DIGIT));
        assert_eq!(classcode_for_char(b'D'), Some(NDIGIT));
        assert_eq!(classcode_for_char(b'w'), Some(WORD));
        assert_eq!(classcode_for_char(b'W'), Some(NWORD));
        assert_eq!(classcode_for_char(b's'), Some(WHITE));
        assert_eq!(classcode_for_char(b'S'), Some(NWHITE));

        // Invalid character
        assert_eq!(classcode_for_char(b'z'), None);
        assert_eq!(classcode_for_char(b'0'), None);
    }

    #[test]
    fn test_stack_sizes() {
        assert_eq!(REGSTACK_INITIAL, 2048);
        assert_eq!(BACKPOS_INITIAL, 64);
    }
}
