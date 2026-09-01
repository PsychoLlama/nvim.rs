//! The opcodes an NFA program is written in.
//!
//! Upstream keeps them in one anonymous C enum whose values are negative so
//! that a state's `c` can hold either an opcode or a literal character, and
//! that seam survives here: `nfa_state.c` and the postfix program are still
//! `c_int`, because a character is not an opcode. [`NfaOp`] is the *named*
//! half of that space and `try_from` is the only way in.
//!
//! Every discriminant is written out because the parser does arithmetic on
//! them: `\1`..`\9` are nine consecutive opcodes, and so are the ten capture
//! opens and the ten closes. Those runs are the [`NfaOp::MOPEN`]-style arrays
//! below, which is how a group number indexes one. The thirty-one numbers
//! between [`NfaOp::NupperIc`] and [`NfaOp::Cursor`] are a hole: upstream
//! reserves them for `class + NFA_ADD_NL`, the `\_x` form of a character
//! class. Nothing here builds one -- the parser carries "and a line break"
//! as a `bool` beside the class instead -- but the hole stays so that the
//! numbers below it keep their values.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

/// One opcode of an NFA program.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub(crate) enum NfaOp {
    Split = -1024,
    Match = -1023,
    Empty = -1022,
    StartColl = -1021,
    EndColl = -1020,
    StartNegColl = -1019,
    EndNegColl = -1018,
    Range = -1017,
    RangeMin = -1016,
    RangeMax = -1015,
    Concat = -1014,
    Or = -1013,
    Star = -1012,
    StarNongreedy = -1011,
    Quest = -1010,
    QuestNongreedy = -1009,
    Bol = -1008,
    Eol = -1007,
    Bow = -1006,
    Eow = -1005,
    Bof = -1004,
    Eof = -1003,
    Newl = -1002,
    Zstart = -1001,
    Zend = -1000,
    Nopen = -999,
    Nclose = -998,
    StartInvisible = -997,
    StartInvisibleFirst = -996,
    StartInvisibleNeg = -995,
    StartInvisibleNegFirst = -994,
    StartInvisibleBefore = -993,
    StartInvisibleBeforeFirst = -992,
    StartInvisibleBeforeNeg = -991,
    StartInvisibleBeforeNegFirst = -990,
    StartPattern = -989,
    EndInvisible = -988,
    EndInvisibleNeg = -987,
    EndPattern = -986,
    Composing = -985,
    EndComposing = -984,
    AnyComposing = -983,
    OptChars = -982,
    PrevAtomNoWidth = -981,
    PrevAtomNoWidthNeg = -980,
    PrevAtomJustBefore = -979,
    PrevAtomJustBeforeNeg = -978,
    PrevAtomLikePattern = -977,
    Backref1 = -976,
    Backref2 = -975,
    Backref3 = -974,
    Backref4 = -973,
    Backref5 = -972,
    Backref6 = -971,
    Backref7 = -970,
    Backref8 = -969,
    Backref9 = -968,
    Zref1 = -967,
    Zref2 = -966,
    Zref3 = -965,
    Zref4 = -964,
    Zref5 = -963,
    Zref6 = -962,
    Zref7 = -961,
    Zref8 = -960,
    Zref9 = -959,
    Skip = -958,
    Mopen = -957,
    Mopen1 = -956,
    Mopen2 = -955,
    Mopen3 = -954,
    Mopen4 = -953,
    Mopen5 = -952,
    Mopen6 = -951,
    Mopen7 = -950,
    Mopen8 = -949,
    Mopen9 = -948,
    Mclose = -947,
    Mclose1 = -946,
    Mclose2 = -945,
    Mclose3 = -944,
    Mclose4 = -943,
    Mclose5 = -942,
    Mclose6 = -941,
    Mclose7 = -940,
    Mclose8 = -939,
    Mclose9 = -938,
    Zopen = -937,
    Zopen1 = -936,
    Zopen2 = -935,
    Zopen3 = -934,
    Zopen4 = -933,
    Zopen5 = -932,
    Zopen6 = -931,
    Zopen7 = -930,
    Zopen8 = -929,
    Zopen9 = -928,
    Zclose = -927,
    Zclose1 = -926,
    Zclose2 = -925,
    Zclose3 = -924,
    Zclose4 = -923,
    Zclose5 = -922,
    Zclose6 = -921,
    Zclose7 = -920,
    Zclose8 = -919,
    Zclose9 = -918,
    Any = -917,
    Ident = -916,
    Sident = -915,
    Kword = -914,
    Skword = -913,
    Fname = -912,
    Sfname = -911,
    Print = -910,
    Sprint = -909,
    White = -908,
    Nwhite = -907,
    Digit = -906,
    Ndigit = -905,
    Hex = -904,
    Nhex = -903,
    Octal = -902,
    Noctal = -901,
    Word = -900,
    Nword = -899,
    Head = -898,
    Nhead = -897,
    Alpha = -896,
    Nalpha = -895,
    Lower = -894,
    Nlower = -893,
    Upper = -892,
    Nupper = -891,
    LowerIc = -890,
    NlowerIc = -889,
    UpperIc = -888,
    NupperIc = -887,
    Cursor = -855,
    Lnum = -854,
    LnumGt = -853,
    LnumLt = -852,
    Col = -851,
    ColGt = -850,
    ColLt = -849,
    Vcol = -848,
    VcolGt = -847,
    VcolLt = -846,
    Mark = -845,
    MarkGt = -844,
    MarkLt = -843,
    Visual = -842,
    ClassAlnum = -841,
    ClassAlpha = -840,
    ClassBlank = -839,
    ClassCntrl = -838,
    ClassDigit = -837,
    ClassGraph = -836,
    ClassLower = -835,
    ClassPrint = -834,
    ClassPunct = -833,
    ClassSpace = -832,
    ClassUpper = -831,
    ClassXdigit = -830,
    ClassTab = -829,
    ClassReturn = -828,
    ClassBackspace = -827,
    ClassEscape = -826,
    ClassIdent = -825,
    ClassKeyword = -824,
    ClassFname = -823,
}

impl NfaOp {
    /// `Mopen` and its nine numbered siblings, indexed by group number.
    pub(crate) const MOPEN: [NfaOp; 10] = [
        NfaOp::Mopen,
        NfaOp::Mopen1,
        NfaOp::Mopen2,
        NfaOp::Mopen3,
        NfaOp::Mopen4,
        NfaOp::Mopen5,
        NfaOp::Mopen6,
        NfaOp::Mopen7,
        NfaOp::Mopen8,
        NfaOp::Mopen9,
    ];

    /// `Mclose` and its nine numbered siblings, indexed by group number.
    pub(crate) const MCLOSE: [NfaOp; 10] = [
        NfaOp::Mclose,
        NfaOp::Mclose1,
        NfaOp::Mclose2,
        NfaOp::Mclose3,
        NfaOp::Mclose4,
        NfaOp::Mclose5,
        NfaOp::Mclose6,
        NfaOp::Mclose7,
        NfaOp::Mclose8,
        NfaOp::Mclose9,
    ];

    /// `Zopen` and its nine numbered siblings, indexed by group number.
    pub(crate) const ZOPEN: [NfaOp; 10] = [
        NfaOp::Zopen,
        NfaOp::Zopen1,
        NfaOp::Zopen2,
        NfaOp::Zopen3,
        NfaOp::Zopen4,
        NfaOp::Zopen5,
        NfaOp::Zopen6,
        NfaOp::Zopen7,
        NfaOp::Zopen8,
        NfaOp::Zopen9,
    ];

    /// `Zclose` and its nine numbered siblings, indexed by group number.
    pub(crate) const ZCLOSE: [NfaOp; 10] = [
        NfaOp::Zclose,
        NfaOp::Zclose1,
        NfaOp::Zclose2,
        NfaOp::Zclose3,
        NfaOp::Zclose4,
        NfaOp::Zclose5,
        NfaOp::Zclose6,
        NfaOp::Zclose7,
        NfaOp::Zclose8,
        NfaOp::Zclose9,
    ];

    /// The nine `\1`..`\9` back-references, indexed by group number minus one.
    pub(crate) const BACKREFS: [NfaOp; 9] = [
        NfaOp::Backref1,
        NfaOp::Backref2,
        NfaOp::Backref3,
        NfaOp::Backref4,
        NfaOp::Backref5,
        NfaOp::Backref6,
        NfaOp::Backref7,
        NfaOp::Backref8,
        NfaOp::Backref9,
    ];

    /// The nine `\z1`..`\z9` external references, indexed the same way.
    pub(crate) const ZREFS: [NfaOp; 9] = [
        NfaOp::Zref1,
        NfaOp::Zref2,
        NfaOp::Zref3,
        NfaOp::Zref4,
        NfaOp::Zref5,
        NfaOp::Zref6,
        NfaOp::Zref7,
        NfaOp::Zref8,
        NfaOp::Zref9,
    ];

    /// The marker that opens capture group `number`, `\\(` through `\\9(`.
    pub(crate) fn mopen(number: c_int) -> NfaOp {
        NfaOp::MOPEN[group(number)]
    }

    /// The marker that opens external capture group `number`, `\\z(`.
    pub(crate) fn zopen(number: c_int) -> NfaOp {
        NfaOp::ZOPEN[group(number)]
    }

    /// The `\\1`..`\\9` back-reference to group `number`.
    pub(crate) fn backref(number: c_int) -> NfaOp {
        NfaOp::BACKREFS[group(number) - 1]
    }

    /// The `\\z1`..`\\z9` reference to external group `number`.
    pub(crate) fn zref(number: c_int) -> NfaOp {
        NfaOp::ZREFS[group(number) - 1]
    }

    /// The number this opcode is stored as.
    pub(crate) const fn code(self) -> c_int {
        self as c_int
    }

    /// Where a group's open or close marker stands in `run`, which is the
    /// group number for the capture runs and one less for the reference runs.
    pub(crate) fn index_in(self, run: &[NfaOp]) -> Option<usize> {
        run.iter().position(|op| *op == self)
    }

    /// One of the forty markers that open or close a capture: `MOPEN` through
    /// `ZCLOSE9`, which upstream matches as one range.
    pub(crate) fn is_capture_marker(self) -> bool {
        (NfaOp::Mopen.code()..=NfaOp::Zclose9.code()).contains(&self.code())
    }

    /// One of the eighteen `\1`-style references, back or external.
    pub(crate) fn is_reference(self) -> bool {
        (NfaOp::Backref1.code()..=NfaOp::Zref9.code()).contains(&self.code())
    }

    /// Does this opcode take an inline operand -- the number that follows it
    /// in the postfix program?
    ///
    /// `\\%[abc]` counts its members, `\\@123<=` its width, and the position
    /// assertions carry the line, column or mark they are about.
    pub(crate) fn has_inline_operand(self) -> bool {
        matches!(
            self,
            NfaOp::OptChars | NfaOp::PrevAtomJustBefore | NfaOp::PrevAtomJustBeforeNeg
        ) || (NfaOp::Lnum.code()..=NfaOp::MarkLt.code()).contains(&self.code())
    }

    /// One of the thirty character classes `\i` through `[^A-Z]`, the ones
    /// [`class_matches`](super::classes::class_matches) answers for.
    ///
    /// `Any` sits just below the run and is deliberately outside it: it is
    /// not a class test, it accepts anything.
    pub(crate) fn is_class(self) -> bool {
        (NfaOp::Ident.code()..=NfaOp::NupperIc.code()).contains(&self.code())
    }
}

/// The number is not one of the named opcodes -- most often because it is a
/// literal character, which shares the space.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct NotAnOpcode;

impl TryFrom<c_int> for NfaOp {
    type Error = NotAnOpcode;

    fn try_from(code: c_int) -> Result<NfaOp, NotAnOpcode> {
        Ok(match code {
            -1024 => NfaOp::Split,
            -1023 => NfaOp::Match,
            -1022 => NfaOp::Empty,
            -1021 => NfaOp::StartColl,
            -1020 => NfaOp::EndColl,
            -1019 => NfaOp::StartNegColl,
            -1018 => NfaOp::EndNegColl,
            -1017 => NfaOp::Range,
            -1016 => NfaOp::RangeMin,
            -1015 => NfaOp::RangeMax,
            -1014 => NfaOp::Concat,
            -1013 => NfaOp::Or,
            -1012 => NfaOp::Star,
            -1011 => NfaOp::StarNongreedy,
            -1010 => NfaOp::Quest,
            -1009 => NfaOp::QuestNongreedy,
            -1008 => NfaOp::Bol,
            -1007 => NfaOp::Eol,
            -1006 => NfaOp::Bow,
            -1005 => NfaOp::Eow,
            -1004 => NfaOp::Bof,
            -1003 => NfaOp::Eof,
            -1002 => NfaOp::Newl,
            -1001 => NfaOp::Zstart,
            -1000 => NfaOp::Zend,
            -999 => NfaOp::Nopen,
            -998 => NfaOp::Nclose,
            -997 => NfaOp::StartInvisible,
            -996 => NfaOp::StartInvisibleFirst,
            -995 => NfaOp::StartInvisibleNeg,
            -994 => NfaOp::StartInvisibleNegFirst,
            -993 => NfaOp::StartInvisibleBefore,
            -992 => NfaOp::StartInvisibleBeforeFirst,
            -991 => NfaOp::StartInvisibleBeforeNeg,
            -990 => NfaOp::StartInvisibleBeforeNegFirst,
            -989 => NfaOp::StartPattern,
            -988 => NfaOp::EndInvisible,
            -987 => NfaOp::EndInvisibleNeg,
            -986 => NfaOp::EndPattern,
            -985 => NfaOp::Composing,
            -984 => NfaOp::EndComposing,
            -983 => NfaOp::AnyComposing,
            -982 => NfaOp::OptChars,
            -981 => NfaOp::PrevAtomNoWidth,
            -980 => NfaOp::PrevAtomNoWidthNeg,
            -979 => NfaOp::PrevAtomJustBefore,
            -978 => NfaOp::PrevAtomJustBeforeNeg,
            -977 => NfaOp::PrevAtomLikePattern,
            -976 => NfaOp::Backref1,
            -975 => NfaOp::Backref2,
            -974 => NfaOp::Backref3,
            -973 => NfaOp::Backref4,
            -972 => NfaOp::Backref5,
            -971 => NfaOp::Backref6,
            -970 => NfaOp::Backref7,
            -969 => NfaOp::Backref8,
            -968 => NfaOp::Backref9,
            -967 => NfaOp::Zref1,
            -966 => NfaOp::Zref2,
            -965 => NfaOp::Zref3,
            -964 => NfaOp::Zref4,
            -963 => NfaOp::Zref5,
            -962 => NfaOp::Zref6,
            -961 => NfaOp::Zref7,
            -960 => NfaOp::Zref8,
            -959 => NfaOp::Zref9,
            -958 => NfaOp::Skip,
            -957 => NfaOp::Mopen,
            -956 => NfaOp::Mopen1,
            -955 => NfaOp::Mopen2,
            -954 => NfaOp::Mopen3,
            -953 => NfaOp::Mopen4,
            -952 => NfaOp::Mopen5,
            -951 => NfaOp::Mopen6,
            -950 => NfaOp::Mopen7,
            -949 => NfaOp::Mopen8,
            -948 => NfaOp::Mopen9,
            -947 => NfaOp::Mclose,
            -946 => NfaOp::Mclose1,
            -945 => NfaOp::Mclose2,
            -944 => NfaOp::Mclose3,
            -943 => NfaOp::Mclose4,
            -942 => NfaOp::Mclose5,
            -941 => NfaOp::Mclose6,
            -940 => NfaOp::Mclose7,
            -939 => NfaOp::Mclose8,
            -938 => NfaOp::Mclose9,
            -937 => NfaOp::Zopen,
            -936 => NfaOp::Zopen1,
            -935 => NfaOp::Zopen2,
            -934 => NfaOp::Zopen3,
            -933 => NfaOp::Zopen4,
            -932 => NfaOp::Zopen5,
            -931 => NfaOp::Zopen6,
            -930 => NfaOp::Zopen7,
            -929 => NfaOp::Zopen8,
            -928 => NfaOp::Zopen9,
            -927 => NfaOp::Zclose,
            -926 => NfaOp::Zclose1,
            -925 => NfaOp::Zclose2,
            -924 => NfaOp::Zclose3,
            -923 => NfaOp::Zclose4,
            -922 => NfaOp::Zclose5,
            -921 => NfaOp::Zclose6,
            -920 => NfaOp::Zclose7,
            -919 => NfaOp::Zclose8,
            -918 => NfaOp::Zclose9,
            -917 => NfaOp::Any,
            -916 => NfaOp::Ident,
            -915 => NfaOp::Sident,
            -914 => NfaOp::Kword,
            -913 => NfaOp::Skword,
            -912 => NfaOp::Fname,
            -911 => NfaOp::Sfname,
            -910 => NfaOp::Print,
            -909 => NfaOp::Sprint,
            -908 => NfaOp::White,
            -907 => NfaOp::Nwhite,
            -906 => NfaOp::Digit,
            -905 => NfaOp::Ndigit,
            -904 => NfaOp::Hex,
            -903 => NfaOp::Nhex,
            -902 => NfaOp::Octal,
            -901 => NfaOp::Noctal,
            -900 => NfaOp::Word,
            -899 => NfaOp::Nword,
            -898 => NfaOp::Head,
            -897 => NfaOp::Nhead,
            -896 => NfaOp::Alpha,
            -895 => NfaOp::Nalpha,
            -894 => NfaOp::Lower,
            -893 => NfaOp::Nlower,
            -892 => NfaOp::Upper,
            -891 => NfaOp::Nupper,
            -890 => NfaOp::LowerIc,
            -889 => NfaOp::NlowerIc,
            -888 => NfaOp::UpperIc,
            -887 => NfaOp::NupperIc,
            -855 => NfaOp::Cursor,
            -854 => NfaOp::Lnum,
            -853 => NfaOp::LnumGt,
            -852 => NfaOp::LnumLt,
            -851 => NfaOp::Col,
            -850 => NfaOp::ColGt,
            -849 => NfaOp::ColLt,
            -848 => NfaOp::Vcol,
            -847 => NfaOp::VcolGt,
            -846 => NfaOp::VcolLt,
            -845 => NfaOp::Mark,
            -844 => NfaOp::MarkGt,
            -843 => NfaOp::MarkLt,
            -842 => NfaOp::Visual,
            -841 => NfaOp::ClassAlnum,
            -840 => NfaOp::ClassAlpha,
            -839 => NfaOp::ClassBlank,
            -838 => NfaOp::ClassCntrl,
            -837 => NfaOp::ClassDigit,
            -836 => NfaOp::ClassGraph,
            -835 => NfaOp::ClassLower,
            -834 => NfaOp::ClassPrint,
            -833 => NfaOp::ClassPunct,
            -832 => NfaOp::ClassSpace,
            -831 => NfaOp::ClassUpper,
            -830 => NfaOp::ClassXdigit,
            -829 => NfaOp::ClassTab,
            -828 => NfaOp::ClassReturn,
            -827 => NfaOp::ClassBackspace,
            -826 => NfaOp::ClassEscape,
            -825 => NfaOp::ClassIdent,
            -824 => NfaOp::ClassKeyword,
            -823 => NfaOp::ClassFname,
            _ => return Err(NotAnOpcode),
        })
    }
}

/// A capture group number as an index. The parser has already refused a
/// pattern with more than nine groups, so anything else is a bug here.
fn group(number: c_int) -> usize {
    usize::try_from(number).expect("a capture group number")
}
