//! Strings and buffer text: splitting an API string into lines, and the
//! line-index arithmetic the buffer API shares.

#![forbid(unsafe_code)]

use super::{CAR, NL};
use crate::api::private::validate::err_out_of_range;

use crate::pos::MAXLNUM;
use crate::types::{Array, Error, LineNr, NUL, Object, String_0, int64_t};
use crate::winlayer::Buf;

// -- Strings ---------------------------------------------------------------

/// Split `input` into one array item per line, each line its own string.
///
/// Line breaks are `\n`, or `\r` and `\r\n` as well with `crlf`. A NUL in
/// the text stands for a newline, as it does everywhere a buffer line is
/// passed as a C string, and is turned back into one. Text that ends *with*
/// a break gets a trailing empty item, so that the array round-trips.
pub(crate) fn string_to_array(input: &String_0, crlf: bool) -> Array {
    let bytes = input.as_bytes();
    let is_break = |byte: u8| byte == NL as u8 || (crlf && byte == CAR as u8);
    let mut items: Vec<Object> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let line_len = bytes[at..]
            .iter()
            .position(|&byte| is_break(byte))
            .unwrap_or(bytes.len() - at);
        // A NUL in the line stands for the newline it was read as.
        let line = bytes[at..at + line_len].iter();
        let line: Vec<u8> = line
            .map(|&b| if b == NUL as u8 { NL as u8 } else { b })
            .collect();
        items.push(Object::string(String_0::from_vec(line)));

        at += line_len;
        let ends_line = at < bytes.len();
        // A CRLF counts as one break, so the LF is stepped over as well.
        if crlf && ends_line && bytes[at] == CAR as u8 && bytes.get(at + 1) == Some(&(NL as u8)) {
            at += 1;
        }
        if at + 1 == bytes.len() && ends_line {
            // Text that ends with a break round-trips through a trailing
            // empty item.
            items.push(Object::string(String_0::NULL));
        }
        at += 1;
    }
    Array::from(items)
}

// -- Buffer text -----------------------------------------------------------

/// Turn a signed, end-relative line index into a 1-based line number,
/// clamping it into the buffer and reporting through `oob` that it had to.
///
/// `end_exclusive` allows one past the last line, which is what an
/// end-of-range index means.
pub(crate) fn normalize_index(
    buffer: Buf,
    index: int64_t,
    end_exclusive: bool,
    oob: &mut bool,
) -> int64_t {
    // The callers have checked that `buffer` is loaded.
    let line_count = buffer.b_ml.ml_line_count;
    debug_assert!(line_count > 0);
    let max_index = (line_count + end_exclusive as LineNr - 1) as int64_t;
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
    index + 1
}

/// The text of line `lnum` between the two columns, copied out of the
/// buffer's own line. Negative columns count back from the end.
pub(crate) fn buf_get_text(
    buffer: Buf,
    lnum: int64_t,
    start_col: int64_t,
    end_col: int64_t,
) -> Result<String_0, Error> {
    if lnum >= i64::from(MAXLNUM) {
        return Err(err_out_of_range(c"line index"));
    }
    let mut lines = buffer.lines();
    let bufstr = lines.line(lnum as LineNr);
    let line_length = int64_t::try_from(bufstr.len()).unwrap_or(0);

    let relative = |col: int64_t| if col < 0 { line_length + col + 1 } else { col };
    let start_col = relative(start_col).clamp(0, line_length);
    let end_col = relative(end_col).clamp(0, line_length);
    if start_col > end_col {
        let why = c"start_col must be less than or equal to end_col";
        return Err(Error::validation(why));
    }
    // `start_col` and `end_col` were clamped into the line.
    let (from, to) = (start_col as usize, end_col as usize);
    Ok(String_0::from_bytes(&bufstr[from..to]))
}
