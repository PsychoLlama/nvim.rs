//! What a pattern compile refuses, and how the refusal is said.
//!
//! The two compilers and the parsing they share answer a [`CompileError`]
//! where upstream printed and returned null; [`super::vim_regcomp_ext`] is
//! the one place that says it, once, and then tells its caller what kind of
//! refusal went out ([`Refused`]). The texts are upstream's, byte for byte,
//! and so is the split between the refusals that blame the pattern and the
//! ones that do not (upstream's `rc_did_emsg`), which is what makes a search
//! add "E383: Invalid search string".

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use core::fmt;

use crate::message::e_nopresub;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::os::cshim::gettext;
use crate::tr;

/// How a message spells a magic character: bare (`(`) or with the backslash
/// the current 'magic' level needs (`\(`). Each message decides which by its
/// own test, as upstream's do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Spelled {
    Bare,
    Escaped,
}

impl Spelled {
    /// [`Spelled::Bare`] when `bare`.
    pub(crate) fn bare_if(bare: bool) -> Self {
        if bare { Self::Bare } else { Self::Escaped }
    }

    fn prefix(self) -> &'static str {
        match self {
            Self::Bare => "",
            Self::Escaped => "\\",
        }
    }
}

/// Which compiler refused: the two blame the pattern for different things.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Engine {
    Backtracking,
    Nfa,
}

/// Why a pattern does not compile. A character in a variant is the byte the
/// message shows.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CompileError {
    /// `\z(` past the ninth.
    TooManyZGroups,
    /// `\(` past the ninth.
    TooManyGroups(Spelled),
    /// A `\z(` without its `\)`.
    UnmatchedZGroup,
    /// A `\%(` without its `\)`.
    UnmatchedNoCapture(Spelled),
    /// A `\(` without its `\)`.
    UnmatchedGroup(Spelled),
    /// A `\)` without its `\(`.
    UnmatchedClose(Spelled),
    /// The parser stopped before the end of the pattern ("can't happen").
    Trailing,
    /// `\@` followed by something that is not a lookaround.
    InvalidAfterAt(Spelled),
    /// More than ten complex `\{n,m}`.
    TooManyComplexBraces(Spelled),
    /// A multi after a multi: `**`.
    NestedStar(Spelled),
    /// A multi after a multi, the second one shown.
    Nested(Spelled, u8),
    /// `\_` followed by something that is not a class.
    InvalidUnderscore,
    /// A multi with nothing in front of it.
    FollowsNothing(Spelled, u8),
    /// `\N` before group N closed.
    IllegalBackReference,
    /// `\z(` outside a syntax region's start pattern.
    ZGroupNotAllowed,
    /// `\z1`..`\z9` outside a syntax region's skip and end patterns.
    ZRefNotAllowed,
    /// `\z` followed by something the backtracker does not know.
    InvalidAfterZ,
    /// A `\%[` without its `]`.
    MissingOptionalEnd(Spelled),
    /// `\%[]`.
    EmptyOptional(Spelled),
    /// `\%` followed by something the backtracker does not know.
    InvalidAfterPercent(Spelled),
    /// More `[` than the parser keeps track of ("cannot happen").
    TooManyBrackets,
    /// The program does not fit the backtracker's offsets.
    TooLong,
    /// A group or an alternation inside `\%[]`.
    InvalidInOptional(Spelled),
    /// The parser lost track of where an alternative ends.
    Internal,
    /// A `\{` whose bound does not parse.
    BraceSyntax(Spelled),
    /// A `\%d`/`\%x`/`\%o`/`\%u`/`\%U` without a number that fits.
    InvalidNumber(Spelled),
    /// A `[` without its `]`, under `RE_STRICT`.
    MissingBracket(Spelled),
    /// `\zs` or `\ze` with a multi.
    CannotRepeat { ze: bool },
    /// A range whose end comes before its start.
    ReverseRange,
    /// A range wider than the backtracker's collection allows.
    RangeTooLarge,
    /// A character above U+10FFFF in a collection.
    CodepointTooLarge,
    /// `\%.` followed by a number.
    NumberAfterDot(u8),
    /// `\%#=` inside the pattern, which has the digit.
    EngineNotAtStart(u8),
    /// `~` with no previous substitute string.
    NoPreviousSubstitute,
    /// The end of the pattern in the middle of an atom.
    PrematureEnd,
    /// A multi or a closing token where an atom belongs.
    Misplaced(u8),
    /// `\z` followed by something the NFA engine does not know.
    UnknownZOperator(u8),
    /// `\%` followed by something the NFA engine does not know.
    UnknownPercentOperator(u8),
    /// `\@` followed by something that is not a lookaround.
    UnknownAtOperator(u8),
    /// A `\{` whose bound does not parse, which the shared reader has said
    /// first.
    RepetitionLimits(Spelled),
    /// A multi after a multi.
    MultiAfterMulti,
    /// `\(` past the ninth.
    NfaTooManyGroups,
    /// The parser stopped before the end of the pattern.
    Termination,
    /// The postfix program asked for an operand that is not there.
    CannotPop,
    /// The postfix program left fragments over.
    StatesLeft,
    /// The second pass needed more states than the first counted.
    NotEnoughStates,
    /// `\_` followed by something that is not a class: its code.
    InvalidClass(i64),
    /// A class code the NFA engine has no item for.
    UnknownClass(c_int),
    /// `\z(` past the ninth.
    NfaTooManyZGroups,
    /// A `\%23l`-style number too large for its unit.
    PositionTooLarge,
    /// `\%l`, `\%c` or `\%v` with neither a number nor a `.`.
    MissingValue(u8),
    /// The NFA engine gave up without a word: a bound too wide to be worth
    /// its states under the automatic engine, which then hands the pattern
    /// to the backtracker, or a program that outgrew its first pass's count.
    Declined,
}

const _: () = assert!(size_of::<CompileError>() <= 16);

/// What a pattern a compile refused was told; whatever there was to say has
/// been said.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refused {
    /// The message blamed the pattern.
    Invalid,
    /// The message did not blame the pattern (`\%#=` misplaced, one of the
    /// NFA engine's operator complaints), or the NFA engine declined without
    /// one. A search says "E383: Invalid search string" after it.
    Unexplained,
}

impl CompileError {
    /// Put the message out, if the refusal has one.
    pub(crate) fn report(&self) {
        match self {
            Self::Declined => {}
            Self::RepetitionLimits(spelled) => {
                Self::BraceSyntax(*spelled).report();
                emsg_text(tr!("E870: (NFA regexp) Error reading repetition limits"));
            }
            _ => {
                emsg_text(self.to_string());
            }
        }
    }

    /// Whether `engine`'s refusal blames the pattern: what upstream's
    /// `rc_did_emsg` records. The two engines differ on `\%.5l` and on `~`.
    pub(crate) fn blames_pattern(&self, engine: Engine) -> bool {
        match self {
            Self::RepetitionLimits(_) => true,
            Self::NumberAfterDot(_) | Self::NoPreviousSubstitute => engine == Engine::Backtracking,
            Self::EngineNotAtStart(_)
            | Self::Misplaced(_)
            | Self::UnknownZOperator(_)
            | Self::UnknownPercentOperator(_)
            | Self::UnknownAtOperator(_)
            | Self::UnknownClass(_)
            | Self::CannotPop
            | Self::PositionTooLarge
            | Self::MissingValue(_)
            | Self::Declined => false,
            _ => true,
        }
    }

    /// What its caller is told once it has been said.
    pub(crate) fn refused(&self, engine: Engine) -> Refused {
        if self.blames_pattern(engine) {
            Refused::Invalid
        } else {
            Refused::Unexplained
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each literal is the message as upstream's `semsg()` spelled it,
        // named arguments and all: it is the msgid.
        let text = match *self {
            Self::TooManyZGroups => tr!("E50: Too many \\z("),
            Self::TooManyGroups(spelled) => {
                let prefix = spelled.prefix();
                tr!("E51: Too many {prefix}(")
            }
            Self::UnmatchedZGroup => tr!("E52: Unmatched \\z("),
            Self::UnmatchedNoCapture(spelled) => {
                let prefix = spelled.prefix();
                tr!("E53: Unmatched {prefix}%(")
            }
            Self::UnmatchedGroup(spelled) => {
                let prefix = spelled.prefix();
                tr!("E54: Unmatched {prefix}(")
            }
            Self::UnmatchedClose(spelled) => {
                let prefix = spelled.prefix();
                tr!("E55: Unmatched {prefix})")
            }
            Self::Trailing => tr!("E488: Trailing characters"),
            Self::InvalidAfterAt(spelled) => {
                let prefix = spelled.prefix();
                tr!("E59: Invalid character after {prefix}@")
            }
            Self::TooManyComplexBraces(spelled) => {
                let prefix = spelled.prefix();
                tr!("E60: Too many complex {prefix}{{...}}s")
            }
            Self::NestedStar(spelled) => {
                let prefix = spelled.prefix();
                tr!("E61: Nested {prefix}*")
            }
            Self::Nested(spelled, c) => {
                let prefix = spelled.prefix();
                let c = char::from(c);
                tr!("E62: Nested {prefix}{c}")
            }
            Self::InvalidUnderscore => tr!("E63: Invalid use of \\_"),
            Self::FollowsNothing(spelled, c) => {
                let prefix = spelled.prefix();
                let c = char::from(c);
                tr!("E64: {prefix}{c} follows nothing")
            }
            Self::IllegalBackReference => tr!("E65: Illegal back reference"),
            Self::ZGroupNotAllowed => tr!("E66: \\z( not allowed here"),
            Self::ZRefNotAllowed => tr!("E67: \\z1 - \\z9 not allowed here"),
            Self::InvalidAfterZ => tr!("E68: Invalid character after \\z"),
            Self::MissingOptionalEnd(spelled) => {
                let prefix = spelled.prefix();
                tr!("E69: Missing ] after {prefix}%[")
            }
            Self::EmptyOptional(spelled) => {
                let prefix = spelled.prefix();
                tr!("E70: Empty {prefix}%[]")
            }
            Self::InvalidAfterPercent(spelled) => {
                let prefix = spelled.prefix();
                tr!("E71: Invalid character after {prefix}%")
            }
            Self::TooManyBrackets => tr!("E76: Too many ["),
            Self::TooLong => tr!("E339: Pattern too long"),
            Self::InvalidInOptional(spelled) => {
                let prefix = spelled.prefix();
                tr!("E369: Invalid item in {prefix}%[]")
            }
            Self::Internal => tr!("E473: Internal error in regexp"),
            Self::BraceSyntax(spelled) | Self::RepetitionLimits(spelled) => {
                let prefix = spelled.prefix();
                tr!("E554: Syntax error in {prefix}{{...}}")
            }
            Self::InvalidNumber(spelled) => {
                let prefix = spelled.prefix();
                tr!("E678: Invalid character after {prefix}%[dxouU]")
            }
            Self::MissingBracket(spelled) => {
                let prefix = spelled.prefix();
                tr!("E769: Missing ] after {prefix}[")
            }
            Self::CannotRepeat { ze } => {
                let what = if ze { "\\ze" } else { "\\zs" };
                tr!("E888: (NFA regexp) cannot repeat {what}")
            }
            Self::ReverseRange => tr!("E944: Reverse range in character class"),
            Self::RangeTooLarge => tr!("E945: Range too large in character class"),
            Self::CodepointTooLarge => {
                tr!("E1541: Value too large, max Unicode codepoint is U+10FFFF")
            }
            Self::NumberAfterDot(c) => {
                let c = char::from(c);
                tr!("E1204: No Number allowed after .: '\\%{c}'")
            }
            Self::EngineNotAtStart(which) => {
                let which = char::from(which);
                tr!("E1281: Atom '\\%#={which}' must be at the start of the pattern")
            }
            Self::NoPreviousSubstitute => msg_cstr(gettext(e_nopresub)).to_string(),
            Self::PrematureEnd => tr!("E865: (NFA) Regexp end encountered prematurely"),
            Self::Misplaced(c) => {
                let c = char::from(c);
                tr!("E866: (NFA regexp) Misplaced {c}")
            }
            Self::UnknownZOperator(c) => {
                let c = char::from(c);
                tr!("E867: (NFA) Unknown operator '\\z{c}'")
            }
            Self::UnknownPercentOperator(c) => {
                let c = char::from(c);
                tr!("E867: (NFA) Unknown operator '\\%{c}'")
            }
            Self::UnknownAtOperator(op) => {
                let op = char::from(op);
                tr!("E869: (NFA) Unknown operator '\\@{op}'")
            }
            Self::MultiAfterMulti => tr!("E871: (NFA regexp) Can't have a multi follow a multi"),
            Self::NfaTooManyGroups => tr!("E872: (NFA regexp) Too many '('"),
            Self::Termination => tr!("E873: (NFA regexp) proper termination error"),
            Self::CannotPop => tr!("E874: (NFA) Could not pop the stack!"),
            Self::StatesLeft => tr!(
                "E875: (NFA regexp) (While converting from postfix to NFA),too many states left on stack"
            ),
            Self::NotEnoughStates => {
                tr!("E876: (NFA regexp) Not enough space to store the whole NFA ")
            }
            Self::InvalidClass(c) => tr!("E877: (NFA regexp) Invalid character class: {c}"),
            Self::UnknownClass(c) => tr!("INTERNAL: Unknown character class char: {}", c),
            Self::NfaTooManyZGroups => tr!("E879: (NFA regexp) Too many \\z("),
            Self::PositionTooLarge => tr!("E951: \\% value too large"),
            Self::MissingValue(c) => {
                let c = char::from(c);
                tr!("E1273: (NFA regexp) missing value in '\\%{c}'")
            }
            Self::Declined => String::new(),
        };
        f.write_str(&text)
    }
}

/// What a compile step answers: what it built, or why the pattern is
/// refused.
pub(crate) type Compiled<T = ()> = Result<T, CompileError>;
