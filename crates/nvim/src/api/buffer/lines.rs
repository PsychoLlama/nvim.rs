//! Whole-line reads and replacements.
//!
//! `nvim_buf_get_lines`/`nvim_buf_set_lines` are the line-granular half of
//! the buffer API, and `nvim_buf_get_text` the read that takes a byte range
//! but still hands back whole lines.  `push_linestr` is the shared
//! line-to-`String_0` step every one of them ends in, and
//! `buf_collect_lines` the loop over a range that `buffer_updates.rs`
//! reaches too, to build the `nvim_buf_attach` change notifications.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::lua::stack::{At, Stack};
use crate::normal::{visual_active, visual_anchor, with_visual_anchor};
use crate::winlayer::{Buf, tab_windows};

pub fn nvim_buf_line_count(buf: BufferHandle) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as Integer);
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(0 as Integer);
    }
    Ok(b.line_count() as Integer)
}

/// Lines `start..end` of `buf`. Called from Lua (`stack` is `Some`), the
/// lines go straight into a table on the Lua stack and the answer is empty.
pub fn nvim_buf_get_lines(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start: Integer,
    mut end: Integer,
    strict_indexing: Boolean,
    stack: Option<&mut Stack>,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut rv: Array = Array::EMPTY;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    // SAFETY: non-null, so the handle named a live buffer.
    if b.b_ml.ml_mfp.is_none() {
        return rv.reported(error);
    }
    let mut oob: bool = false;
    start = normalize_index(b, start as int64_t, true, &mut oob) as Integer;
    end = normalize_index(b, end as int64_t, true, &mut oob) as Integer;
    if !(!strict_indexing || !oob) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return rv.reported(error);
    }
    if start >= end {
        return rv.reported(error);
    }
    let size: size_t = (end - start) as size_t;
    let at = start as LineNr;
    let nl = channel_id != VIML_INTERNAL_CALL;
    let mut lines = LineSink::new(stack, &mut rv, size);
    buf_collect_lines(b, size, at, 0, nl, &mut lines);
    rv.reported(error)
}

/// An API string as a buffer line: NUL-terminated, with each NUL it holds
/// turned back into the newline it stands for.
pub(crate) fn api_line(text: &[u8]) -> Vec<u8> {
    let mut line = Vec::with_capacity(text.len() + 1);
    api_text_into(&mut line, text);
    line.push(0);
    line
}

/// Append an API string's bytes to a buffer line, each NUL turned back into
/// the newline it stands for.
pub(crate) fn api_text_into(line: &mut Vec<u8>, text: &[u8]) {
    line.extend(text.iter().map(|&b| if b == 0 { b'\n' } else { b }));
}

/// Replace lines `start..end` of `buf` with `replacement`.
pub fn nvim_buf_set_lines(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start: Integer,
    mut end: Integer,
    strict_indexing: Boolean,
    replacement: Array,
) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(());
    };
    let mut oob: bool = false;
    start = normalize_index(buffer, start as int64_t, true, &mut oob) as Integer;
    end = normalize_index(buffer, end as int64_t, true, &mut oob) as Integer;
    if !(!strict_indexing || !oob) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return ().reported(error);
    }
    if !(start <= end) {
        let why = c"'start' is higher than 'end'";
        error = Error::validation(why);
        return ().reported(error);
    }
    let disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    check_string_array(&replacement, c"replacement string", disallow_nl)?;
    let new_len: size_t = replacement.len();
    let old_len: size_t = (end - start) as size_t;
    let mut extra: ptrdiff_t = 0 as ptrdiff_t;
    // Every item is a String: `check_string_array` above turned anything
    // else into an error. The memline borrows each line (`noalloc`), so they
    // live in this frame until the edit is done. Each is NUL-terminated, and
    // `api_line` turned every NUL inside it into a newline.
    let lines: Vec<Vec<u8>> = (0..new_len)
        .map(|i| {
            let l = replacement[i].as_string();
            api_line(
                l.expect("check_string_array accepted only Strings")
                    .as_bytes(),
            )
        })
        .collect();
    let mut tstate: TryState = TryState::INIT;
    try_enter(&mut tstate);
    let buf = buffer;
    's_382: {
        if buf.b_p_ma == 0 {
            let why = c"Buffer is not 'modifiable'";
            error = Error::exception(why);
        } else if u_save_buf(buf, (start - 1 as Integer) as LineNr, end as LineNr).is_err() {
            let why = c"Failed to save undo information";
            error = Error::exception(why);
        } else {
            let deleted_bytes: BCount =
                get_region_bytecount(buf, start as LineNr, end as LineNr, 0 as ColNr, 0 as ColNr);
            let to_delete: size_t = if new_len < old_len {
                old_len.wrapping_sub(new_len)
            } else {
                0 as size_t
            };
            let mut i_0: size_t = 0 as size_t;
            while i_0 < to_delete {
                if ml_delete_buf(buffer, start as LineNr, false).is_err() {
                    let why = c"Failed to delete line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    i_0 = i_0.wrapping_add(1);
                }
            }
            if to_delete > 0 as size_t {
                extra -= to_delete as ptrdiff_t;
            }
            let to_replace: size_t = if old_len < new_len { old_len } else { new_len };
            let mut inserted_bytes: BCount = 0 as BCount;
            let mut i_1: size_t = 0 as size_t;
            while i_1 < to_replace {
                let lnum: int64_t = start as int64_t + i_1 as int64_t;
                if !(lnum < MAXLNUM as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_382;
                } else if ml_replace_buf_now(buffer, lnum as LineNr, &lines[i_1]).is_err() {
                    let why = c"Failed to replace line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    // The line and its NUL.
                    inserted_bytes += lines[i_1].len() as BCount;
                    i_1 = i_1.wrapping_add(1);
                }
            }
            let mut i_2: size_t = to_replace;
            while i_2 < new_len {
                let lnum_0: int64_t = start as int64_t + i_2 as int64_t - 1 as int64_t;
                if !(lnum_0 < MAXLNUM as int64_t) {
                    let why = c"Index out of bounds";
                    error = Error::validation(why);
                    break 's_382;
                } else if ml_append_buf_terminated(buffer, lnum_0 as LineNr, &lines[i_2], false)
                    .is_err()
                {
                    let why = c"Failed to insert line";
                    error = Error::exception(why);
                    break 's_382;
                } else {
                    // The line and its NUL.
                    inserted_bytes += lines[i_2].len() as BCount;
                    extra += 1;
                    i_2 = i_2.wrapping_add(1);
                }
            }
            let adjust: LineNr = if end > start {
                MAXLNUM as LineNr
            } else {
                0 as LineNr
            };
            mark_adjust_buf(
                buffer,
                start as LineNr,
                (end - 1 as Integer) as LineNr,
                adjust,
                extra as LineNr,
                true,
                kMarkAdjustApi,
                kExtmarkNOOP,
            );
            if visual_active() as ::core::ffi::c_int != 0
                && Some(buffer) == Buf::current_or_none()
                && visual_anchor().lnum >= start as LineNr
            {
                if visual_anchor().lnum >= end as LineNr {
                    with_visual_anchor(|a| a.lnum += extra as LineNr);
                }
                check_visual_pos();
            }
            extmark_splice(
                buffer,
                start as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as ColNr,
                (end - start) as ::core::ffi::c_int,
                0 as ColNr,
                deleted_bytes,
                new_len as ::core::ffi::c_int,
                0 as ColNr,
                inserted_bytes,
                kExtmarkUndo,
            );
            changed_lines(
                buffer,
                start as LineNr,
                0 as ColNr,
                end as LineNr,
                extra as LineNr,
                true,
            );
            for win in tab_windows() {
                if win.w_buffer == buffer {
                    let (lo, hi) = (start as LineNr, end as LineNr);
                    fix_cursor(win, lo, hi, extra as LineNr);
                }
            }
        }
    }
    // The bracket outranks whatever the body wrote into `error`, which is
    // the order the two had when both went through one slot.
    try_leave(&tstate)?;
    ().reported(error)
}

/// The text from `(start_row, start_col)` to `(end_row, end_col)`, a line
/// per row. Called from Lua (`stack` is `Some`), the lines go straight into a
/// table on the Lua stack and the answer is empty.
pub fn nvim_buf_get_text(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start_row: Integer,
    start_col: Integer,
    mut end_row: Integer,
    end_col: Integer,
    _opts: &mut KeyDict_empty,
    stack: Option<&mut Stack>,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let mut rv: Array = Array::EMPTY;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    // SAFETY: non-null, so the handle named a live buffer.
    if b.b_ml.ml_mfp.is_none() {
        return rv.reported(error);
    }
    let mut oob: bool = false;
    start_row = normalize_index(b, start_row as int64_t, false, &mut oob) as Integer;
    end_row = normalize_index(b, end_row as int64_t, false, &mut oob) as Integer;
    if oob {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return rv.reported(error);
    }
    if !(start_row <= end_row) {
        let why = c"'start' is higher than 'end'";
        error = Error::validation(why);
        return rv.reported(error);
    }
    let replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
    let size: size_t = ((end_row - start_row) as size_t).wrapping_add(1 as size_t);
    let mut lines = LineSink::new(stack, &mut rv, size);
    let first = start_row as int64_t;
    if start_row == end_row {
        let (from, to) = (start_col as int64_t, end_col as int64_t);
        let line: String_0 = buf_get_text(b, first, from, to)?;
        lines.push(line.as_bytes(), 0, replace_nl);
        return Ok(rv);
    }
    let from = start_col as int64_t;
    let to = (MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int64_t;
    let line = buf_get_text(b, first, from, to)?;
    lines.push(line.as_bytes(), 0, replace_nl);
    if size > 2 as size_t {
        let n = size.wrapping_sub(2 as size_t);
        let at = start_row as LineNr + 1 as LineNr;
        buf_collect_lines(b, n, at, 1, replace_nl, &mut lines);
    }
    let last = end_row as int64_t;
    let to = end_col as int64_t;
    let line = buf_get_text(b, last, 0 as int64_t, to)?;
    let at = size.wrapping_sub(1 as size_t) as ::core::ffi::c_int;
    lines.push(line.as_bytes(), at, replace_nl);
    Ok(rv)
}

pub fn nvim_buf_get_offset(buf: BufferHandle, index: Integer) -> Result<Integer, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as Integer);
    };
    if b.b_ml.ml_mfp.is_none() {
        return (-1 as Integer).reported(error);
    }
    if !(index >= 0 as Integer && index <= b.line_count() as Integer) {
        let why = c"Index out of bounds";
        error = Error::validation(why);
        return (0 as Integer).reported(error);
    }
    let lnum = index as LineNr + 1 as LineNr;
    let offset = ml_find_line_or_offset(b, lnum, None, true);
    (offset as Integer).reported(error)
}

/// Where collected lines go: straight into the Lua table on top of the
/// stack, when Lua asked, or into an API array.
pub(crate) enum LineSink<'a> {
    Lua(&'a mut Stack),
    Api(&'a mut Array),
}

impl<'a> LineSink<'a> {
    /// A sink for `size` lines: a fresh Lua table pushed for them when there
    /// is a `stack`, else `array` reserved for them.
    pub(crate) fn new(stack: Option<&'a mut Stack>, array: &'a mut Array, size: size_t) -> Self {
        match stack {
            Some(stack) => {
                stack.new_table(size, 0);
                LineSink::Lua(stack)
            }
            None => {
                *array = Array::with_capacity(size);
                LineSink::Api(array)
            }
        }
    }

    /// Store line `bytes` as item `idx` (0-based), each newline in it --
    /// which is how a buffer line spells NUL -- turned back into a NUL when
    /// `replace_nl`.
    fn push(&mut self, bytes: &[u8], idx: ::core::ffi::c_int, replace_nl: bool) {
        // The question is only whether the line holds a NL: `contains` answers
        // it in one early-exiting `memchr` pass, and a line without one (860,000
        // a run on the memline bench) is handed on as it is.
        let restored = (replace_nl && bytes.contains(&b'\n')).then(|| -> Vec<u8> {
            bytes
                .iter()
                .map(|&b| if b == b'\n' { 0 } else { b })
                .collect()
        });
        let line = restored.as_deref().unwrap_or(bytes);
        match self {
            LineSink::Lua(stack) => {
                stack.push_bytes(line);
                stack.raw_set_index(At::from_top(2), idx + 1);
            }
            LineSink::Api(array) => {
                // An empty line is the null string, as upstream's
                // `STRING_INIT` is.
                let line = if line.is_empty() {
                    String_0::NULL
                } else {
                    String_0::from_bytes(line)
                };
                array.push(Object::string(line));
            }
        }
    }
}

/// Collect `n` lines of `buffer` from `start` into `lines`, as items
/// `start_idx..`.
pub(crate) fn buf_collect_lines(
    buffer: Buf,
    n: size_t,
    start: LineNr,
    start_idx: ::core::ffi::c_int,
    replace_nl: bool,
    lines: &mut LineSink<'_>,
) {
    let mut text = buffer.lines();
    for i in 0..n {
        let lnum: LineNr = start + i as LineNr;
        let at = start_idx + i as ::core::ffi::c_int;
        lines.push(text.line(lnum), at, replace_nl);
    }
}
