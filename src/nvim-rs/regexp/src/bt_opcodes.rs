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
// Magic Character Handling
// =============================================================================

/// Magic value to distinguish special chars from literal bytes.
/// Magic characters are stored as negative values: Magic(x) = x - 256.
pub const MAGIC_OFFSET: c_int = 256;

/// Convert a character to its "magic" form.
/// This is used to distinguish metacharacters from literals in the pattern.
#[inline]
pub const fn magic(x: c_int) -> c_int {
    x - MAGIC_OFFSET
}

/// Convert a "magic" value back to the original character.
#[inline]
pub const fn un_magic(x: c_int) -> c_int {
    x + MAGIC_OFFSET
}

/// Check if a value is a "magic" (special) character.
#[inline]
pub const fn is_magic(x: c_int) -> bool {
    x < 0
}

/// Magic byte used to identify compiled regexp programs.
pub const REGMAGIC: u8 = 0o234;

// =============================================================================
// Opcode Range Checking
// =============================================================================

/// Check if opcode is an MOPEN (0-9).
#[inline]
pub const fn is_mopen(op: c_int) -> bool {
    op >= MOPEN && op < MOPEN + 10
}

/// Check if opcode is an MCLOSE (0-9).
#[inline]
pub const fn is_mclose(op: c_int) -> bool {
    op >= MCLOSE && op < MCLOSE + 10
}

/// Check if opcode is a BACKREF (1-9).
#[inline]
pub const fn is_backref(op: c_int) -> bool {
    op > BACKREF && op < BACKREF + 10
}

/// Check if opcode is a ZOPEN (0-9).
#[inline]
pub const fn is_zopen(op: c_int) -> bool {
    op >= ZOPEN && op < ZOPEN + 10
}

/// Check if opcode is a ZCLOSE (0-9).
#[inline]
pub const fn is_zclose(op: c_int) -> bool {
    op >= ZCLOSE && op < ZCLOSE + 10
}

/// Check if opcode is a ZREF (1-9).
#[inline]
pub const fn is_zref(op: c_int) -> bool {
    op > ZREF && op < ZREF + 10
}

/// Check if opcode is a BRACE_COMPLEX (0-9).
#[inline]
pub const fn is_brace_complex(op: c_int) -> bool {
    op >= BRACE_COMPLEX && op < BRACE_COMPLEX + 10
}

/// Get the subexpr number from an MOPEN opcode.
#[inline]
pub const fn get_mopen_num(op: c_int) -> c_int {
    op - MOPEN
}

/// Get the subexpr number from an MCLOSE opcode.
#[inline]
pub const fn get_mclose_num(op: c_int) -> c_int {
    op - MCLOSE
}

/// Get the backref number from a BACKREF opcode.
#[inline]
pub const fn get_backref_num(op: c_int) -> c_int {
    op - BACKREF
}

/// Get the zref number from a ZREF opcode.
#[inline]
pub const fn get_zref_num(op: c_int) -> c_int {
    op - ZREF
}

/// Get the subexpr number from a ZOPEN opcode.
#[inline]
pub const fn get_zopen_num(op: c_int) -> c_int {
    op - ZOPEN
}

/// Get the subexpr number from a ZCLOSE opcode.
#[inline]
pub const fn get_zclose_num(op: c_int) -> c_int {
    op - ZCLOSE
}

/// Get the brace number from a BRACE_COMPLEX opcode.
#[inline]
pub const fn get_brace_complex_num(op: c_int) -> c_int {
    op - BRACE_COMPLEX
}

// =============================================================================
// Bytecode Access Functions
// =============================================================================

/// Get the opcode at a bytecode position.
///
/// # Safety
/// Pointer must be valid and point to valid bytecode.
#[inline]
pub unsafe fn op(p: *const u8) -> c_int {
    *p as c_int
}

/// Get the "next" pointer offset from a bytecode position.
/// The offset is stored as a big-endian 16-bit value at p+1 and p+2.
///
/// # Safety
/// Pointer must be valid and point to at least 3 bytes of valid bytecode.
#[inline]
pub unsafe fn next(p: *const u8) -> c_int {
    ((*p.add(1) as c_int & 0o377) << 8) + (*p.add(2) as c_int & 0o377)
}

/// Get a pointer to the operand of a bytecode instruction.
/// The operand starts at p+3 (after opcode and next pointer).
///
/// # Safety
/// Pointer must be valid and point to at least 3 bytes of valid bytecode.
#[inline]
pub unsafe fn operand(p: *const u8) -> *const u8 {
    p.add(3)
}

/// Get a mutable pointer to the operand of a bytecode instruction.
///
/// # Safety
/// Pointer must be valid and point to at least 3 bytes of valid bytecode.
#[inline]
pub unsafe fn operand_mut(p: *mut u8) -> *mut u8 {
    p.add(3)
}

/// Read a 64-bit minimum value from BRACE_LIMITS operand.
/// Format: 4 bytes at operand position (big-endian).
///
/// # Safety
/// Pointer must point to valid BRACE_LIMITS bytecode.
#[inline]
pub unsafe fn operand_min(p: *const u8) -> i64 {
    let op = operand(p);
    (((*op) as i64) << 24)
        + (((*op.add(1)) as i64) << 16)
        + (((*op.add(2)) as i64) << 8)
        + ((*op.add(3)) as i64)
}

/// Read a 64-bit maximum value from BRACE_LIMITS operand.
/// The max comes after the min (4 bytes later).
///
/// # Safety
/// Pointer must point to valid BRACE_LIMITS bytecode.
#[inline]
pub unsafe fn operand_max(p: *const u8) -> i64 {
    operand_min(p.add(4))
}

/// Read the comparison operator from position operand (RE_LNUM, RE_COL, etc.).
///
/// # Safety
/// Pointer must point to valid position comparison bytecode.
#[inline]
pub unsafe fn operand_cmp(p: *const u8) -> u8 {
    *p.add(7)
}

// =============================================================================
// Register Parens Constants
// =============================================================================

/// REG_NOPAREN - toplevel reg() (no parens).
pub const REG_NOPAREN: c_int = 0;

/// REG_PAREN - \(\) capturing group.
pub const REG_PAREN: c_int = 1;

/// REG_ZPAREN - \z(\) external capturing group.
pub const REG_ZPAREN: c_int = 2;

/// REG_NPAREN - \%(\) non-capturing group.
pub const REG_NPAREN: c_int = 3;

// =============================================================================
// Regex Flags (RF_*)
// =============================================================================

/// Ignore case during matching.
pub const RF_ICASE: c_int = 1;

/// Don't ignore case (explicit).
pub const RF_NOICASE: c_int = 2;

/// Pattern can match a newline.
pub const RF_HASNL: c_int = 4;

/// Ignore combining characters.
pub const RF_ICOMBINE: c_int = 8;

/// Uses `\@<=` or `\@<!`.
pub const RF_LOOKBH: c_int = 16;

// =============================================================================
// Multi-line Matching Constants
// =============================================================================

/// Not a multi-line quantifier.
pub const NOT_MULTI: c_int = 0;

/// Multi-line quantifier matching one.
pub const MULTI_ONE: c_int = 1;

/// Multi-line quantifier matching multiple.
pub const MULTI_MULT: c_int = 2;

// =============================================================================
// Regmatch Return Codes
// =============================================================================

/// Regmatch: something failed, abort.
pub const RA_FAIL: c_int = 1;

/// Regmatch: continue in inner loop.
pub const RA_CONT: c_int = 2;

/// Regmatch: break inner loop.
pub const RA_BREAK: c_int = 3;

/// Regmatch: successful match.
pub const RA_MATCH: c_int = 4;

/// Regmatch: didn't match.
pub const RA_NOMATCH: c_int = 5;

// =============================================================================
// FFI Exports
// =============================================================================
//
// Note: Some FFI exports (rs_bt_op, rs_bt_next, rs_bt_operand) are defined in
// bt_compile.rs to avoid duplication. These functions remain as internal Rust
// functions here.

/// Check if opcode includes newline matching.
#[no_mangle]
pub extern "C" fn rs_bt_with_nl(op: c_int) -> c_int {
    c_int::from(with_nl(op))
}

/// Check if opcode is an MOPEN.
#[no_mangle]
pub extern "C" fn rs_bt_is_mopen(op: c_int) -> c_int {
    c_int::from(is_mopen(op))
}

/// Check if opcode is an MCLOSE.
#[no_mangle]
pub extern "C" fn rs_bt_is_mclose(op: c_int) -> c_int {
    c_int::from(is_mclose(op))
}

/// Check if opcode is a BACKREF.
#[no_mangle]
pub extern "C" fn rs_bt_is_backref(op: c_int) -> c_int {
    c_int::from(is_backref(op))
}

/// Get the character class opcode for a specifier character.
/// Returns -1 if not a valid specifier.
#[no_mangle]
pub extern "C" fn rs_bt_classcode_for_char(c: u8) -> c_int {
    classcode_for_char(c).unwrap_or(-1)
}

/// Convert character to magic form.
#[no_mangle]
pub extern "C" fn rs_bt_magic(x: c_int) -> c_int {
    magic(x)
}

/// Convert magic form back to character.
#[no_mangle]
pub extern "C" fn rs_bt_un_magic(x: c_int) -> c_int {
    un_magic(x)
}

/// Check if value is a magic character.
#[no_mangle]
pub extern "C" fn rs_bt_is_magic(x: c_int) -> c_int {
    c_int::from(is_magic(x))
}

/// Check if opcode is a ZOPEN.
#[no_mangle]
pub extern "C" fn rs_bt_is_zopen(op: c_int) -> c_int {
    c_int::from(is_zopen(op))
}

/// Check if opcode is a ZCLOSE.
#[no_mangle]
pub extern "C" fn rs_bt_is_zclose(op: c_int) -> c_int {
    c_int::from(is_zclose(op))
}

/// Check if opcode is a ZREF.
#[no_mangle]
pub extern "C" fn rs_bt_is_zref(op: c_int) -> c_int {
    c_int::from(is_zref(op))
}

/// Check if opcode is a BRACE_COMPLEX.
#[no_mangle]
pub extern "C" fn rs_bt_is_brace_complex(op: c_int) -> c_int {
    c_int::from(is_brace_complex(op))
}

/// Get the subexpr number from an MOPEN opcode.
#[no_mangle]
pub extern "C" fn rs_bt_get_mopen_num(op: c_int) -> c_int {
    get_mopen_num(op)
}

/// Get the subexpr number from an MCLOSE opcode.
#[no_mangle]
pub extern "C" fn rs_bt_get_mclose_num(op: c_int) -> c_int {
    get_mclose_num(op)
}

/// Get the backref number from a BACKREF opcode.
#[no_mangle]
pub extern "C" fn rs_bt_get_backref_num(op: c_int) -> c_int {
    get_backref_num(op)
}

/// Get the subexpr number from a ZOPEN opcode.
#[no_mangle]
pub extern "C" fn rs_bt_get_zopen_num(op: c_int) -> c_int {
    get_zopen_num(op)
}

/// Get the subexpr number from a ZCLOSE opcode.
#[no_mangle]
pub extern "C" fn rs_bt_get_zclose_num(op: c_int) -> c_int {
    get_zclose_num(op)
}

/// Get the brace number from a BRACE_COMPLEX opcode.
#[no_mangle]
pub extern "C" fn rs_bt_get_brace_complex_num(op: c_int) -> c_int {
    get_brace_complex_num(op)
}

/// Get the REGMAGIC constant value.
#[no_mangle]
pub extern "C" fn rs_bt_regmagic() -> c_int {
    REGMAGIC as c_int
}

/// Get the ADD_NL constant value.
#[no_mangle]
pub extern "C" fn rs_bt_add_nl() -> c_int {
    ADD_NL
}

/// Get the FIRST_NL constant value.
#[no_mangle]
pub extern "C" fn rs_bt_first_nl() -> c_int {
    FIRST_NL
}

/// Get the LAST_NL constant value.
#[no_mangle]
pub extern "C" fn rs_bt_last_nl() -> c_int {
    LAST_NL
}

/// Get the REGSTACK_INITIAL constant value.
#[no_mangle]
pub extern "C" fn rs_bt_regstack_initial() -> c_int {
    REGSTACK_INITIAL as c_int
}

/// Get the BACKPOS_INITIAL constant value.
#[no_mangle]
pub extern "C" fn rs_bt_backpos_initial() -> c_int {
    BACKPOS_INITIAL as c_int
}

/// Get the HASWIDTH flag value.
#[no_mangle]
pub extern "C" fn rs_bt_haswidth() -> c_int {
    HASWIDTH
}

/// Get the SIMPLE flag value.
#[no_mangle]
pub extern "C" fn rs_bt_simple() -> c_int {
    SIMPLE
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

    #[test]
    fn test_opcode_ranges() {
        // Verify MOPEN/MCLOSE ranges
        assert!(is_mopen(MOPEN));
        assert!(is_mopen(MOPEN + 9));
        assert!(!is_mopen(MOPEN - 1));
        assert!(!is_mopen(MOPEN + 10));

        assert!(is_mclose(MCLOSE));
        assert!(is_mclose(MCLOSE + 9));
        assert!(!is_mclose(MCLOSE - 1));
        assert!(!is_mclose(MCLOSE + 10));
    }

    #[test]
    fn test_backref_range() {
        assert!(is_backref(BACKREF + 1));
        assert!(is_backref(BACKREF + 9));
        assert!(!is_backref(BACKREF));
        assert!(!is_backref(BACKREF + 10));
    }

    #[test]
    fn test_zopen_zclose_ranges() {
        assert!(is_zopen(ZOPEN));
        assert!(is_zopen(ZOPEN + 9));
        assert!(!is_zopen(ZOPEN - 1));
        assert!(!is_zopen(ZOPEN + 10));

        assert!(is_zclose(ZCLOSE));
        assert!(is_zclose(ZCLOSE + 9));
        assert!(!is_zclose(ZCLOSE - 1));
        assert!(!is_zclose(ZCLOSE + 10));
    }

    #[test]
    fn test_brace_complex_range() {
        assert!(is_brace_complex(BRACE_COMPLEX));
        assert!(is_brace_complex(BRACE_COMPLEX + 9));
        assert!(!is_brace_complex(BRACE_COMPLEX - 1));
        assert!(!is_brace_complex(BRACE_COMPLEX + 10));
    }

    #[test]
    fn test_get_subexpr_num() {
        assert_eq!(get_mopen_num(MOPEN), 0);
        assert_eq!(get_mopen_num(MOPEN + 5), 5);
        assert_eq!(get_mopen_num(MOPEN + 9), 9);

        assert_eq!(get_mclose_num(MCLOSE), 0);
        assert_eq!(get_mclose_num(MCLOSE + 3), 3);
    }

    #[test]
    fn test_magic_conversion() {
        // '*' = 42, magic(42) = 42 - 256 = -214
        assert_eq!(magic(b'*' as c_int), -214);
        assert!(is_magic(-214));
        assert!(!is_magic(b'*' as c_int));
        assert_eq!(un_magic(-214), b'*' as c_int);
    }

    #[test]
    fn test_operand_access() {
        // Simulate a bytecode sequence: opcode at [0], next at [1,2], operand at [3+]
        let bytecode: [u8; 10] = [
            EXACTLY as u8, // opcode
            0x01,          // next high byte
            0x20,          // next low byte
            b'h',          // operand bytes
            b'e',
            b'l',
            b'l',
            b'o',
            0,
            0,
        ];
        let ptr = bytecode.as_ptr();
        unsafe {
            assert_eq!(op(ptr), EXACTLY);
            assert_eq!(next(ptr), 0x0120); // big-endian
            assert_eq!(*operand(ptr), b'h');
        }
    }

    #[test]
    fn test_regmagic() {
        assert_eq!(REGMAGIC, 0o234);
    }

    #[test]
    fn test_reg_paren_constants() {
        assert_eq!(REG_NOPAREN, 0);
        assert_eq!(REG_PAREN, 1);
        assert_eq!(REG_ZPAREN, 2);
        assert_eq!(REG_NPAREN, 3);
    }

    #[test]
    fn test_rf_flags() {
        assert_eq!(RF_ICASE, 1);
        assert_eq!(RF_NOICASE, 2);
        assert_eq!(RF_HASNL, 4);
        assert_eq!(RF_ICOMBINE, 8);
        assert_eq!(RF_LOOKBH, 16);
    }

    #[test]
    fn test_ra_codes() {
        assert_eq!(RA_FAIL, 1);
        assert_eq!(RA_CONT, 2);
        assert_eq!(RA_BREAK, 3);
        assert_eq!(RA_MATCH, 4);
        assert_eq!(RA_NOMATCH, 5);
    }
}
