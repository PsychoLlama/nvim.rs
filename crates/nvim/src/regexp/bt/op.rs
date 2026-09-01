//! The opcodes a backtracking program is written in.
//!
//! One node of a compiled program is an opcode byte followed by a two-byte
//! offset to the next node, so every code here fits in a `u8` and the
//! discriminants are upstream's numbers unchanged — the compiler and the
//! matcher both do arithmetic on them, and `\1`..`\9`, the ten capture
//! opens, the ten closes and the ten complex-brace slots are consecutive
//! runs that a group number indexes.
//!
//! The one code shape that is *not* a variant is upstream's `ADD_NL` band.
//! `\_x` is written as `x`'s opcode plus thirty, which upstream then
//! subtracts back off before dispatching; here [`BtOp::decode`] answers the
//! plain opcode and a `bool`, so the twenty-nine doubled codes never need
//! naming. Their numbers stay a hole in the enum so that everything above
//! them keeps its value.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

/// One node's opcode.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub(crate) enum BtOp {
    /// The end of the program: reaching it is a match.
    End = 0,
    Bol = 1,
    Eol = 2,
    /// One alternative of a `\|`; the operand is the next one.
    Branch = 3,
    /// A jump backwards, which is how a loop closes.
    Back = 4,
    /// A run of literal text, NUL-terminated in the operand.
    Exactly = 5,
    Nothing = 6,
    Star = 7,
    Plus = 8,
    /// `\@=`: the operand must match here, and consumes nothing.
    Match = 9,
    /// `\@!`: the operand must not match here.
    Nomatch = 10,
    /// `\@<=`: the operand must match ending here.
    Behind = 11,
    /// `\@<!`: the operand must not match ending here.
    Nobehind = 12,
    /// `\%[]`: the operand's longest prefix matches.
    Subpat = 13,
    /// `\{n,m}` over an item that cannot backtrack into itself.
    BraceSimple = 14,
    Bow = 15,
    Eow = 16,
    /// The two bounds of the `BRACE_COMPLEX` that follows.
    BraceLimits = 17,
    /// A line break as an atom of its own.
    Newl = 18,
    /// Where a `\@<=` look-behind has to end.
    Bhpos = 19,

    // The classes: every one of these has a `\_x` form thirty codes up.
    Any = 20,
    Anyof = 21,
    Anybut = 22,
    Ident = 23,
    Sident = 24,
    Kword = 25,
    Skword = 26,
    Fname = 27,
    Sfname = 28,
    Print = 29,
    Sprint = 30,
    White = 31,
    Nwhite = 32,
    Digit = 33,
    Ndigit = 34,
    Hex = 35,
    Nhex = 36,
    Octal = 37,
    Noctal = 38,
    Word = 39,
    Nword = 40,
    Head = 41,
    Nhead = 42,
    Alpha = 43,
    Nalpha = 44,
    Lower = 45,
    Nlower = 46,
    Upper = 47,
    Nupper = 48,
    // 50..=78 are the `\_x` forms of `Any`..`Nupper`; see `decode`.
    /// `\(`: opens capture group 0, the whole match.
    Mopen = 80,
    Mopen1 = 81,
    Mopen2 = 82,
    Mopen3 = 83,
    Mopen4 = 84,
    Mopen5 = 85,
    Mopen6 = 86,
    Mopen7 = 87,
    Mopen8 = 88,
    Mopen9 = 89,
    Mclose = 90,
    Mclose1 = 91,
    Mclose2 = 92,
    Mclose3 = 93,
    Mclose4 = 94,
    Mclose5 = 95,
    Mclose6 = 96,
    Mclose7 = 97,
    Mclose8 = 98,
    Mclose9 = 99,
    /// `\0` is not a back-reference; the code exists so that `\1`..`\9` sit
    /// at `Backref + n`.
    Backref = 100,
    Backref1 = 101,
    Backref2 = 102,
    Backref3 = 103,
    Backref4 = 104,
    Backref5 = 105,
    Backref6 = 106,
    Backref7 = 107,
    Backref8 = 108,
    Backref9 = 109,
    /// `\z(`: opens external capture group 0, which is never used.
    Zopen = 110,
    Zopen1 = 111,
    Zopen2 = 112,
    Zopen3 = 113,
    Zopen4 = 114,
    Zopen5 = 115,
    Zopen6 = 116,
    Zopen7 = 117,
    Zopen8 = 118,
    Zopen9 = 119,
    Zclose = 120,
    Zclose1 = 121,
    Zclose2 = 122,
    Zclose3 = 123,
    Zclose4 = 124,
    Zclose5 = 125,
    Zclose6 = 126,
    Zclose7 = 127,
    Zclose8 = 128,
    Zclose9 = 129,
    Zref = 130,
    Zref1 = 131,
    Zref2 = 132,
    Zref3 = 133,
    Zref4 = 134,
    Zref5 = 135,
    Zref6 = 136,
    Zref7 = 137,
    Zref8 = 138,
    Zref9 = 139,
    /// `\{n,m}` over an item that can: the code carries which of the ten
    /// counters it uses.
    BraceComplex = 140,
    BraceComplex1 = 141,
    BraceComplex2 = 142,
    BraceComplex3 = 143,
    BraceComplex4 = 144,
    BraceComplex5 = 145,
    BraceComplex6 = 146,
    BraceComplex7 = 147,
    BraceComplex8 = 148,
    BraceComplex9 = 149,
    /// `\%(`: a group that captures nothing.
    Nopen = 150,
    Nclose = 151,
    /// One character that has to be matched whole rather than as bytes.
    Multibytecode = 200,
    ReBof = 201,
    ReEof = 202,
    /// `\%#`: the cursor.
    Cursor = 203,
    ReLnum = 204,
    ReCol = 205,
    ReVcol = 206,
    ReMark = 207,
    ReVisual = 208,
    /// `\%C`: any combining characters, matching nothing else.
    ReComposing = 209,
}

/// A program byte that names no opcode.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct NotAnOpcode;

impl BtOp {
    /// The ten markers that open a capture group, `\(` through `\9(`.
    pub(crate) const MOPEN: [BtOp; 10] = [
        BtOp::Mopen,
        BtOp::Mopen1,
        BtOp::Mopen2,
        BtOp::Mopen3,
        BtOp::Mopen4,
        BtOp::Mopen5,
        BtOp::Mopen6,
        BtOp::Mopen7,
        BtOp::Mopen8,
        BtOp::Mopen9,
    ];

    /// The ten markers that close a capture group.
    pub(crate) const MCLOSE: [BtOp; 10] = [
        BtOp::Mclose,
        BtOp::Mclose1,
        BtOp::Mclose2,
        BtOp::Mclose3,
        BtOp::Mclose4,
        BtOp::Mclose5,
        BtOp::Mclose6,
        BtOp::Mclose7,
        BtOp::Mclose8,
        BtOp::Mclose9,
    ];

    /// The `\1`..`\9` back-references, and the unused zeroth code.
    pub(crate) const BACKREF: [BtOp; 10] = [
        BtOp::Backref,
        BtOp::Backref1,
        BtOp::Backref2,
        BtOp::Backref3,
        BtOp::Backref4,
        BtOp::Backref5,
        BtOp::Backref6,
        BtOp::Backref7,
        BtOp::Backref8,
        BtOp::Backref9,
    ];

    /// The ten markers that open an external capture group, `\z(`.
    pub(crate) const ZOPEN: [BtOp; 10] = [
        BtOp::Zopen,
        BtOp::Zopen1,
        BtOp::Zopen2,
        BtOp::Zopen3,
        BtOp::Zopen4,
        BtOp::Zopen5,
        BtOp::Zopen6,
        BtOp::Zopen7,
        BtOp::Zopen8,
        BtOp::Zopen9,
    ];

    /// The ten markers that close an external capture group.
    pub(crate) const ZCLOSE: [BtOp; 10] = [
        BtOp::Zclose,
        BtOp::Zclose1,
        BtOp::Zclose2,
        BtOp::Zclose3,
        BtOp::Zclose4,
        BtOp::Zclose5,
        BtOp::Zclose6,
        BtOp::Zclose7,
        BtOp::Zclose8,
        BtOp::Zclose9,
    ];

    /// The `\z1`..`\z9` references to an external capture, and the unused
    /// zeroth code.
    pub(crate) const ZREF: [BtOp; 10] = [
        BtOp::Zref,
        BtOp::Zref1,
        BtOp::Zref2,
        BtOp::Zref3,
        BtOp::Zref4,
        BtOp::Zref5,
        BtOp::Zref6,
        BtOp::Zref7,
        BtOp::Zref8,
        BtOp::Zref9,
    ];

    /// The ten complex-repeat slots.
    pub(crate) const BRACE_COMPLEX: [BtOp; 10] = [
        BtOp::BraceComplex,
        BtOp::BraceComplex1,
        BtOp::BraceComplex2,
        BtOp::BraceComplex3,
        BtOp::BraceComplex4,
        BtOp::BraceComplex5,
        BtOp::BraceComplex6,
        BtOp::BraceComplex7,
        BtOp::BraceComplex8,
        BtOp::BraceComplex9,
    ];

    /// The number this opcode is stored as.
    #[inline(always)]
    pub(crate) const fn code(self) -> c_int {
        self as c_int
    }

    /// Where this opcode stands in `run`, which for the capture runs is the
    /// group number and for the reference runs the reference number.
    ///
    /// Every run is ten consecutive codes, so this is a subtraction rather
    /// than a search: it runs inside the match loop, which the test suites
    /// build at opt-level 0.
    #[inline(always)]
    pub(crate) fn index_in(self, first: BtOp) -> Option<usize> {
        let offset = self.code() - first.code();
        // Two comparisons, not `Range::contains`, which is a call at
        // opt-level 0.
        #[allow(clippy::manual_range_contains)]
        if offset < 0 || offset >= 10 {
            return None;
        }
        Some(offset.cast_unsigned() as usize)
    }

    /// One of the ten complex-repeat slots.
    #[inline(always)]
    pub(crate) fn is_complex_brace(self) -> bool {
        self.index_in(BtOp::BraceComplex).is_some()
    }

    /// Is there a `\_x` form of this opcode — a code thirty above it that
    /// also matches a line break?
    #[inline(always)]
    pub(crate) fn has_newline_form(self) -> bool {
        const FIRST: c_int = BtOp::Any.code();
        const LAST: c_int = BtOp::Nupper.code();
        #[allow(clippy::manual_range_contains)]
        {
            self.code() >= FIRST && self.code() <= LAST
        }
    }

    /// The byte a node holds for this opcode, `\_x` form or not.
    ///
    /// Only the classes have a `\_x` form; asking for one of anything else
    /// is a bug in the emitter.
    pub(crate) fn encode(self, crosses_lines: bool) -> u8 {
        let code = if crosses_lines {
            debug_assert!(self.has_newline_form(), "no `\\_` form of {self:?}");
            self.code() + NEWLINE_OFFSET
        } else {
            self.code()
        };
        u8::try_from(code).expect("an opcode fits a program byte")
    }

    /// Read a node's opcode byte: the opcode, and whether it is the `\_x`
    /// form that also matches a line break.
    #[inline(always)]
    pub(crate) fn decode(byte: u8) -> Result<(BtOp, bool), NotAnOpcode> {
        let code = c_int::from(byte);
        // Two comparisons, not `RangeInclusive::contains`, which is a call
        // at opt-level 0 — and this runs once per node of the walk.
        #[allow(clippy::manual_range_contains)]
        if code >= FIRST_NEWLINE && code <= LAST_NEWLINE {
            return Ok((by_code(code - NEWLINE_OFFSET)?, true));
        }
        Ok((by_code(code)?, false))
    }
}

/// What `\_x` adds to `x`'s opcode.
const NEWLINE_OFFSET: c_int = 30;
const FIRST_NEWLINE: c_int = BtOp::Any.code() + NEWLINE_OFFSET;
const LAST_NEWLINE: c_int = BtOp::Nupper.code() + NEWLINE_OFFSET;

/// The opcode `code` names, if it names one.
#[inline(always)]
fn by_code(code: c_int) -> Result<BtOp, NotAnOpcode> {
    const FIRST: c_int = BtOp::End.code();
    const LAST: c_int = BtOp::ReComposing.code();
    #[allow(clippy::manual_range_contains)]
    if code < FIRST || code > LAST {
        return Err(NotAnOpcode);
    }
    // The check above puts `code` in `0..=LAST`.
    match BY_CODE[code.cast_unsigned() as usize] {
        Some(op) => Ok(op),
        None => Err(NotAnOpcode),
    }
}

/// Every opcode by its code, for [`by_code`]. The holes are the `\_x` band
/// and the gaps upstream left between the runs.
static BY_CODE: [Option<BtOp>; 210] = build_by_code();

const fn build_by_code() -> [Option<BtOp>; 210] {
    let mut table: [Option<BtOp>; 210] = [None; 210];
    let mut i = 0;
    while i < ALL.len() {
        let op = ALL[i];
        table[op.code().cast_unsigned() as usize] = Some(op);
        i += 1;
    }
    table
}

/// Every opcode, once. Only [`build_by_code`] reads it.
static ALL: [BtOp; 131] = [
    BtOp::End,
    BtOp::Bol,
    BtOp::Eol,
    BtOp::Branch,
    BtOp::Back,
    BtOp::Exactly,
    BtOp::Nothing,
    BtOp::Star,
    BtOp::Plus,
    BtOp::Match,
    BtOp::Nomatch,
    BtOp::Behind,
    BtOp::Nobehind,
    BtOp::Subpat,
    BtOp::BraceSimple,
    BtOp::Bow,
    BtOp::Eow,
    BtOp::BraceLimits,
    BtOp::Newl,
    BtOp::Bhpos,
    BtOp::Any,
    BtOp::Anyof,
    BtOp::Anybut,
    BtOp::Ident,
    BtOp::Sident,
    BtOp::Kword,
    BtOp::Skword,
    BtOp::Fname,
    BtOp::Sfname,
    BtOp::Print,
    BtOp::Sprint,
    BtOp::White,
    BtOp::Nwhite,
    BtOp::Digit,
    BtOp::Ndigit,
    BtOp::Hex,
    BtOp::Nhex,
    BtOp::Octal,
    BtOp::Noctal,
    BtOp::Word,
    BtOp::Nword,
    BtOp::Head,
    BtOp::Nhead,
    BtOp::Alpha,
    BtOp::Nalpha,
    BtOp::Lower,
    BtOp::Nlower,
    BtOp::Upper,
    BtOp::Nupper,
    BtOp::Mopen,
    BtOp::Mopen1,
    BtOp::Mopen2,
    BtOp::Mopen3,
    BtOp::Mopen4,
    BtOp::Mopen5,
    BtOp::Mopen6,
    BtOp::Mopen7,
    BtOp::Mopen8,
    BtOp::Mopen9,
    BtOp::Mclose,
    BtOp::Mclose1,
    BtOp::Mclose2,
    BtOp::Mclose3,
    BtOp::Mclose4,
    BtOp::Mclose5,
    BtOp::Mclose6,
    BtOp::Mclose7,
    BtOp::Mclose8,
    BtOp::Mclose9,
    BtOp::Backref,
    BtOp::Backref1,
    BtOp::Backref2,
    BtOp::Backref3,
    BtOp::Backref4,
    BtOp::Backref5,
    BtOp::Backref6,
    BtOp::Backref7,
    BtOp::Backref8,
    BtOp::Backref9,
    BtOp::Zopen,
    BtOp::Zopen1,
    BtOp::Zopen2,
    BtOp::Zopen3,
    BtOp::Zopen4,
    BtOp::Zopen5,
    BtOp::Zopen6,
    BtOp::Zopen7,
    BtOp::Zopen8,
    BtOp::Zopen9,
    BtOp::Zclose,
    BtOp::Zclose1,
    BtOp::Zclose2,
    BtOp::Zclose3,
    BtOp::Zclose4,
    BtOp::Zclose5,
    BtOp::Zclose6,
    BtOp::Zclose7,
    BtOp::Zclose8,
    BtOp::Zclose9,
    BtOp::Zref,
    BtOp::Zref1,
    BtOp::Zref2,
    BtOp::Zref3,
    BtOp::Zref4,
    BtOp::Zref5,
    BtOp::Zref6,
    BtOp::Zref7,
    BtOp::Zref8,
    BtOp::Zref9,
    BtOp::BraceComplex,
    BtOp::BraceComplex1,
    BtOp::BraceComplex2,
    BtOp::BraceComplex3,
    BtOp::BraceComplex4,
    BtOp::BraceComplex5,
    BtOp::BraceComplex6,
    BtOp::BraceComplex7,
    BtOp::BraceComplex8,
    BtOp::BraceComplex9,
    BtOp::Nopen,
    BtOp::Nclose,
    BtOp::Multibytecode,
    BtOp::ReBof,
    BtOp::ReEof,
    BtOp::Cursor,
    BtOp::ReLnum,
    BtOp::ReCol,
    BtOp::ReVcol,
    BtOp::ReMark,
    BtOp::ReVisual,
    BtOp::ReComposing,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newline_band_decodes_to_its_plain_form() {
        for op in ALL {
            if !op.has_newline_form() {
                continue;
            }
            assert_eq!(BtOp::decode(op.encode(true)), Ok((op, true)));
        }
    }

    #[test]
    fn every_opcode_round_trips() {
        for op in ALL {
            assert_eq!(BtOp::decode(op.encode(false)), Ok((op, false)));
        }
    }

    #[test]
    fn the_gaps_between_the_runs_name_nothing() {
        for code in [49, 79, 152, 199] {
            assert_eq!(BtOp::decode(code), Err(NotAnOpcode));
        }
    }
}
