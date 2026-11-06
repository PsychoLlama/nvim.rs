//! Why a JSON document would not decode.
//!
//! The scanner and the value stack answer one of these where they refuse,
//! and [`json_decode_string`](super::json_decode_string) says it once, as
//! the boundary every caller goes through. Each refusal quotes the document
//! from where it stopped, as upstream's `%.*s` did -- up to the first NUL,
//! which is where the message ended.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::ThinCString;
use crate::message_fmt::msg_cstr;
use crate::tr;

/// A refusal, carrying the rest of the document it names.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DecodeError {
    #[error("{}", tr!("E474: Attempt to decode a blank string"))]
    BlankString,
    #[error("{}", tr!("E474: No container to close: {}", msg_cstr(.0.as_cstr())))]
    NoContainer(ThinCString),
    #[error("{}", tr!("E474: Closing list with curly bracket: {}", msg_cstr(.0.as_cstr())))]
    CloseListCurly(ThinCString),
    #[error("{}", tr!("E474: Closing dictionary with square bracket: {}", msg_cstr(.0.as_cstr())))]
    CloseDictSquare(ThinCString),
    #[error("{}", tr!("E474: Trailing comma: {}", msg_cstr(.0.as_cstr())))]
    TrailingComma(ThinCString),
    #[error("{}", tr!("E474: Expected value after colon: {}", msg_cstr(.0.as_cstr())))]
    ValueAfterColon(ThinCString),
    #[error("{}", tr!("E474: Expected value: {}", msg_cstr(.0.as_cstr())))]
    ExpectedValue(ThinCString),
    #[error("{}", tr!("E474: Comma not inside container: {}", msg_cstr(.0.as_cstr())))]
    CommaOutside(ThinCString),
    #[error("{}", tr!("E474: Duplicate comma: {}", msg_cstr(.0.as_cstr())))]
    DuplicateComma(ThinCString),
    #[error("{}", tr!("E474: Comma after colon: {}", msg_cstr(.0.as_cstr())))]
    CommaAfterColon(ThinCString),
    #[error("{}", tr!("E474: Using comma in place of colon: {}", msg_cstr(.0.as_cstr())))]
    CommaForColon(ThinCString),
    #[error("{}", tr!("E474: Leading comma: {}", msg_cstr(.0.as_cstr())))]
    LeadingComma(ThinCString),
    #[error("{}", tr!("E474: Colon after comma: {}", msg_cstr(.0.as_cstr())))]
    ColonAfterComma(ThinCString),
    #[error("{}", tr!("E474: Colon not inside container: {}", msg_cstr(.0.as_cstr())))]
    ColonOutside(ThinCString),
    #[error("{}", tr!("E474: Using colon not in dictionary: {}", msg_cstr(.0.as_cstr())))]
    ColonNotInDict(ThinCString),
    #[error("{}", tr!("E474: Unexpected colon: {}", msg_cstr(.0.as_cstr())))]
    UnexpectedColon(ThinCString),
    #[error("{}", tr!("E474: Duplicate colon: {}", msg_cstr(.0.as_cstr())))]
    DuplicateColon(ThinCString),
    #[error("{}", tr!("E474: Expected null: {}", msg_cstr(.0.as_cstr())))]
    ExpectedNull(ThinCString),
    #[error("{}", tr!("E474: Expected true: {}", msg_cstr(.0.as_cstr())))]
    ExpectedTrue(ThinCString),
    #[error("{}", tr!("E474: Expected false: {}", msg_cstr(.0.as_cstr())))]
    ExpectedFalse(ThinCString),
    #[error("{}", tr!("E474: Unidentified byte: {}", msg_cstr(.0.as_cstr())))]
    UnidentifiedByte(ThinCString),
    #[error("{}", tr!("E474: Trailing characters: {}", msg_cstr(.0.as_cstr())))]
    TrailingCharacters(ThinCString),
    #[error("{}", tr!("E474: Unexpected end of input: {}", msg_cstr(.0.as_cstr())))]
    UnexpectedEnd(ThinCString),
    #[error("{}", tr!("E474: Unfinished escape sequence: {}", msg_cstr(.0.as_cstr())))]
    UnfinishedEscape(ThinCString),
    #[error("{}", tr!("E474: Unfinished unicode escape sequence: {}", msg_cstr(.0.as_cstr())))]
    UnfinishedUnicode(ThinCString),
    #[error("{}", tr!("E474: Expected four hex digits after \\u: {}", msg_cstr(.0.as_cstr())))]
    FourHexDigits(ThinCString),
    #[error("{}", tr!("E474: Unknown escape sequence: {}", msg_cstr(.0.as_cstr())))]
    UnknownEscape(ThinCString),
    #[error("{}", tr!("E474: ASCII control characters cannot be present inside string: {}", msg_cstr(.0.as_cstr())))]
    ControlChars(ThinCString),
    #[error("{}", tr!("E474: Only UTF-8 strings allowed: {}", msg_cstr(.0.as_cstr())))]
    OnlyUtf8(ThinCString),
    #[error("{}", tr!("E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: {}", msg_cstr(.0.as_cstr())))]
    Above10ffff(ThinCString),
    #[error("{}", tr!("E474: Expected string end: {}", msg_cstr(.0.as_cstr())))]
    StringEnd(ThinCString),
    #[error("{}", tr!("E474: Leading zeroes are not allowed: {}", msg_cstr(.0.as_cstr())))]
    LeadingZeroes(ThinCString),
    #[error("{}", tr!("E474: Missing number after minus sign: {}", msg_cstr(.0.as_cstr())))]
    AfterMinus(ThinCString),
    #[error("{}", tr!("E474: Missing number after decimal dot: {}", msg_cstr(.0.as_cstr())))]
    AfterDot(ThinCString),
    #[error("{}", tr!("E474: Missing exponent: {}", msg_cstr(.0.as_cstr())))]
    MissingExponent(ThinCString),
    #[error("{}", expected_comma_before_item(.0))]
    CommaBeforeItem(ThinCString),
    #[error("{}", expected_colon_before_value(.0))]
    ColonBeforeValue(ThinCString),
    #[error("{}", expected_string_key(.0))]
    StringKey(ThinCString),
    #[error("{}", expected_comma_before_key(.0))]
    CommaBeforeKey(ThinCString),
}

const _: () = assert!(size_of::<DecodeError>() <= 16);

// The value stack's four refusals keep the `{arg0}` their msgids were
// written with.

fn expected_comma_before_item(rest: &ThinCString) -> String {
    let arg0 = msg_cstr(rest.as_cstr());
    tr!("E474: Expected comma before list item: {arg0}")
}

fn expected_colon_before_value(rest: &ThinCString) -> String {
    let arg0 = msg_cstr(rest.as_cstr());
    tr!("E474: Expected colon before dictionary value: {arg0}")
}

fn expected_string_key(rest: &ThinCString) -> String {
    let arg0 = msg_cstr(rest.as_cstr());
    tr!("E474: Expected string key: {arg0}")
}

fn expected_comma_before_key(rest: &ThinCString) -> String {
    let arg0 = msg_cstr(rest.as_cstr());
    tr!("E474: Expected comma before dictionary key: {arg0}")
}
