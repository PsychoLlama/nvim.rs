//! Strings and buffer text: splitting an API string into lines, and the
//! line-index arithmetic the buffer API shares.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{CAR, NL};
use crate::api::private::validate::err_out_of_range;

use crate::os::libc::int64_t;
use crate::pos::MAXLNUM;
use crate::types::{ApiString, Array, Error, LineNr, Object};
use crate::winlayer::Buf;

// -- Strings ---------------------------------------------------------------

/// Split `input` into one array item per line, each line its own string.
///
/// Line breaks are `\n`, or `\r` and `\r\n` as well with `crlf`. A NUL in
/// the text stands for a newline, as it does everywhere a buffer line is
/// passed as a C string, and is turned back into one. Text that ends *with*
/// a break gets a trailing empty item, so that the array round-trips.
pub(crate) fn string_to_array(input: &ApiString, crlf: bool) -> Array {
    let bytes = input.as_bytes();
    let is_break = |byte: u8| byte == NL || (crlf && byte == CAR);
    let mut items: Vec<Object> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let line_len = bytes[at..]
            .iter()
            .position(|&byte| is_break(byte))
            .unwrap_or(bytes.len() - at);
        // A NUL in the line stands for the newline it was read as.
        let line = bytes[at..at + line_len].iter();
        let line: Vec<u8> = line.map(|&b| if b == 0 { NL } else { b }).collect();
        items.push(Object::string(ApiString::from_vec(line)));

        at += line_len;
        let ends_line = at < bytes.len();
        // A CRLF counts as one break, so the LF is stepped over as well.
        if crlf && ends_line && bytes[at] == CAR && bytes.get(at + 1) == Some(&(NL)) {
            at += 1;
        }
        if at + 1 == bytes.len() && ends_line {
            // Text that ends with a break round-trips through a trailing
            // empty item.
            items.push(Object::string(ApiString::NULL));
        }
        at += 1;
    }
    Array::from(items)
}

// -- Buffer text -----------------------------------------------------------

/// Turn a signed, end-relative line index into a line number,
/// clamping it into the buffer and reporting through `oob` that it had to.
///
/// `end_exclusive` allows one past the last line, which is what an
/// end-of-range index means.
pub(crate) fn normalize_index(
    buffer: Buf,
    index: int64_t,
    end_exclusive: bool,
    oob: &mut bool,
) -> LineNr {
    // The callers have checked that `buffer` is loaded.
    let line_count = buffer.b_ml.ml_line_count;
    debug_assert!(line_count > LineNr::ZERO);
    let max_index = int64_t::from(line_count + i32::from(end_exclusive) - 1);
    let mut index = if index < 0 {
        max_index + index + 1
    } else {
        index
    };
    if index > max_index {
        *oob = true;
        index = max_index;
    } else if index < 0 {
        *oob = true;
        index = 0;
    }
    LineNr::try_from(index + 1).expect("clamped into the buffer")
}

/// The text of line `lnum` between the two columns, copied out of the
/// buffer's own line. Negative columns count back from the end.
pub(crate) fn buf_get_text(
    buffer: Buf,
    lnum: LineNr,
    start_col: int64_t,
    end_col: int64_t,
) -> Result<ApiString, Error> {
    if lnum >= MAXLNUM {
        return Err(err_out_of_range(c"line index"));
    }
    let mut lines = buffer.lines();
    let bufstr = lines.line(lnum);
    let line_length = int64_t::try_from(bufstr.len()).unwrap_or(0);

    let relative = |col: int64_t| if col < 0 { line_length + col + 1 } else { col };
    let start_col = relative(start_col).clamp(0, line_length);
    let end_col = relative(end_col).clamp(0, line_length);
    if start_col > end_col {
        let why = c"start_col must be less than or equal to end_col";
        return Err(Error::validation(why));
    }
    // `start_col` and `end_col` were clamped into the line.
    let in_line = |col: int64_t| usize::try_from(col).expect("clamped into the line");
    let (from, to) = (in_line(start_col), in_line(end_col));
    Ok(ApiString::from_bytes(&bufstr[from..to]))
}
