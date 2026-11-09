//! Whole-line reads and replacements.
//!
//! `nvim_buf_get_lines`/`nvim_buf_set_lines` are the line-granular half of
//! the buffer API, and `nvim_buf_get_text` the read that takes a byte range
//! but still hands back whole lines.  `push_linestr` is the shared
//! line-to-`ApiString` step every one of them ends in, and
//! `buf_collect_lines` the loop over a range that `buffer_updates.rs`
//! reaches too, to build the `nvim_buf_attach` change notifications.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::api_try;
use crate::extmark::kExtmarkNOOP;
use crate::extmark::kExtmarkUndo;
use crate::lua::stack::{At, Stack};
use crate::mark::kMarkAdjustApi;
use crate::memory::memchr_in;
use crate::narrow::{len_as_int, len_as_integer};
use crate::normal::{visual_active, visual_anchor, with_visual_anchor};
use crate::winlayer::{Buf, tab_windows};

pub fn nvim_buf_line_count(buf: BufferHandle) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0);
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(0);
    }
    Ok(i64::from(b.line_count()))
}

/// Lines `start..end` of `buf`. Called from Lua (`stack` is `Some`), the
/// lines go straight into a table on the Lua stack and the answer is empty.
pub fn nvim_buf_get_lines(
    channel_id: uint64_t,
    buf: BufferHandle,
    start: Integer,
    end: Integer,
    strict_indexing: Boolean,
    stack: Option<&mut Stack>,
) -> Result<Array, Error> {
    let mut rv = Vec::new();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv.into());
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(rv.into());
    }
    let mut oob: bool = false;
    let start = normalize_index(b, start, true, &mut oob);
    let end = normalize_index(b, end, true, &mut oob);
    if strict_indexing && oob {
        return Err(Error::validation(c"Index out of bounds"));
    }
    if start >= end {
        return Ok(rv.into());
    }
    let size = line_span(start, end);
    let nl = channel_id != VIML_INTERNAL_CALL;
    let mut lines = LineSink::new(stack, &mut rv, size);
    b.collect_lines(size, start, 0, nl, &mut lines);
    Ok(rv.into())
}

/// How many lines `start..end` holds: `end` is not before `start`.
#[inline]
pub(super) fn line_span(start: LineNr, end: LineNr) -> usize {
    usize::try_from(end - start).expect("the range runs forwards")
}

/// A length in bytes as the extmark code's byte count. Nothing in memory
/// holds more than `isize::MAX` bytes, so this cannot fail.
#[inline]
pub(super) fn byte_count(len: usize) -> BCount {
    BCount::try_from(len).expect("an allocation fits in isize")
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

/// The line `offset` lines below `start`, if it is before `MAXLNUM`.
#[inline]
pub(super) fn line_below(start: LineNr, offset: usize) -> Result<LineNr, Error> {
    let lnum = i64::from(start) + len_as_integer(offset);
    LineNr::try_from(lnum)
        .ok()
        .filter(|&lnum| lnum < MAXLNUM)
        .ok_or_else(|| Error::validation(c"Index out of bounds"))
}

/// Replace lines `start..end` of `buf` with `replacement`.
pub fn nvim_buf_set_lines(
    channel_id: uint64_t,
    buf: BufferHandle,
    start: Integer,
    end: Integer,
    strict_indexing: Boolean,
    replacement: Array,
) -> Result<(), Error> {
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(());
    };
    let mut oob: bool = false;
    let start = normalize_index(buffer, start, true, &mut oob);
    let end = normalize_index(buffer, end, true, &mut oob);
    if strict_indexing && oob {
        return Err(Error::validation(c"Index out of bounds"));
    }
    if start > end {
        return Err(Error::validation(c"'start' is higher than 'end'"));
    }
    let disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    check_string_array(&replacement, c"replacement string", disallow_nl)?;
    let new_len = replacement.len();
    let old_len = line_span(start, end);
    let mut extra: i32 = 0;
    // Every item is a String: `check_string_array` above turned anything
    // else into an error. The memline borrows each line (`noalloc`), so they
    // live in this frame until the edit is done. Each is NUL-terminated, and
    // `api_line` turned every NUL inside it into a newline.
    let lines: Vec<Vec<u8>> = replacement
        .iter()
        .map(|item| {
            let l = item.as_string();
            api_line(
                l.expect("check_string_array accepted only Strings")
                    .as_bytes(),
            )
        })
        .collect();
    // What the bracket caught outranks the body's own refusal.
    api_try(|| -> Result<(), Error> {
        if buffer.b_p_ma == 0 {
            return Err(Error::exception(c"Buffer is not 'modifiable'"));
        }
        if u_save_buf(buffer, start - 1, end).is_err() {
            return Err(Error::exception(c"Failed to save undo information"));
        }
        let deleted_bytes: BCount =
            get_region_bytecount(buffer, start, end, ColNr::ZERO, ColNr::ZERO);
        let to_delete = old_len.saturating_sub(new_len);
        for _ in 0..to_delete {
            if ml_delete_buf(buffer, start, false).is_err() {
                return Err(Error::exception(c"Failed to delete line"));
            }
        }
        extra -= len_as_int(to_delete);
        let to_replace = old_len.min(new_len);
        let mut inserted_bytes: BCount = 0;
        for (i, line) in lines.iter().enumerate().take(to_replace) {
            let lnum = line_below(start, i)?;
            if ml_replace_buf_now(buffer, lnum, line).is_err() {
                return Err(Error::exception(c"Failed to replace line"));
            }
            // The line and its NUL.
            inserted_bytes += byte_count(line.len());
        }
        for (i, line) in lines.iter().enumerate().skip(to_replace) {
            // Appended below the line before it.
            let lnum = line_below(start - 1, i)?;
            if ml_append_buf_terminated(buffer, lnum, line, false).is_err() {
                return Err(Error::exception(c"Failed to insert line"));
            }
            // The line and its NUL.
            inserted_bytes += byte_count(line.len());
            extra += 1;
        }
        let adjust: i32 = if end > start { MAXLNUM.get() } else { 0 };
        mark_adjust_buf(
            buffer,
            start,
            end - 1,
            adjust,
            extra,
            true,
            kMarkAdjustApi,
            kExtmarkNOOP,
        );
        if visual_active()
            && Some(buffer) == Buf::current_or_none()
            && visual_anchor().lnum >= start
        {
            if visual_anchor().lnum >= end {
                with_visual_anchor(|a| a.lnum += extra);
            }
            check_visual_pos();
        }
        extmark_splice(
            buffer,
            start.get() - 1,
            ColNr::ZERO,
            end - start,
            ColNr::ZERO,
            deleted_bytes,
            len_as_int(new_len),
            ColNr::ZERO,
            inserted_bytes,
            kExtmarkUndo,
        );
        changed_lines(buffer, start, ColNr::ZERO, end, extra, true);
        for win in tab_windows() {
            if win.w_buffer == buffer {
                fix_cursor(win, start, end, extra);
            }
        }
        Ok(())
    })?
}

/// The text from `(start_row, start_col)` to `(end_row, end_col)`, a line
/// per row. Called from Lua (`stack` is `Some`), the lines go straight into a
/// table on the Lua stack and the answer is empty.
pub fn nvim_buf_get_text(
    channel_id: uint64_t,
    buf: BufferHandle,
    start_row: Integer,
    start_col: Integer,
    end_row: Integer,
    end_col: Integer,
    _opts: &mut KeyDict_empty,
    stack: Option<&mut Stack>,
) -> Result<Array, Error> {
    let mut rv = Vec::new();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv.into());
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(rv.into());
    }
    let mut oob: bool = false;
    let first = normalize_index(b, start_row, false, &mut oob);
    let last = normalize_index(b, end_row, false, &mut oob);
    if oob {
        return Err(Error::validation(c"Index out of bounds"));
    }
    if first > last {
        return Err(Error::validation(c"'start' is higher than 'end'"));
    }
    let replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
    let size = line_span(first, last) + 1;
    let mut lines = LineSink::new(stack, &mut rv, size);
    if first == last {
        let line: ApiString = buf_get_text(b, first, start_col, end_col)?;
        lines.push(line.as_bytes(), 0, replace_nl);
        return Ok(rv.into());
    }
    let line = buf_get_text(b, first, start_col, Integer::from(MAXCOL - 1))?;
    lines.push(line.as_bytes(), 0, replace_nl);
    if size > 2 {
        b.collect_lines(size - 2, first + 1, 1, replace_nl, &mut lines);
    }
    let line = buf_get_text(b, last, 0, end_col)?;
    lines.push(line.as_bytes(), len_as_int(size - 1), replace_nl);
    Ok(rv.into())
}

pub fn nvim_buf_get_offset(buf: BufferHandle, index: Integer) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0);
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(-1);
    }
    let Some(lnum) = LineNr::try_from(index)
        .ok()
        .filter(|&i| i >= LineNr::ZERO && i <= b.line_count())
    else {
        return Err(Error::validation(c"Index out of bounds"));
    };
    // -1 is the API's answer for "no offset".
    let offset = b.find_line_or_offset(lnum + 1, None, true).unwrap_or(-1);
    Ok(Integer::from(offset))
}

/// Where collected lines go: straight into the Lua table on top of the
/// stack, when Lua asked, or into an API array.
#[derive(Debug)]
pub(crate) enum LineSink<'a> {
    Lua(&'a mut Stack),
    Api(&'a mut Vec<Object>),
}

impl<'a> LineSink<'a> {
    /// A sink for `size` lines: a fresh Lua table pushed for them when there
    /// is a `stack`, else `array` reserved for them.
    pub(crate) fn new(
        stack: Option<&'a mut Stack>,
        array: &'a mut Vec<Object>,
        size: size_t,
    ) -> Self {
        match stack {
            Some(stack) => {
                stack.new_table(size, 0);
                LineSink::Lua(stack)
            }
            None => {
                *array = Vec::with_capacity(size);
                LineSink::Api(array)
            }
        }
    }

    /// Store line `bytes` as item `idx` (0-based), each newline in it --
    /// which is how a buffer line spells NUL -- turned back into a NUL when
    /// `replace_nl`.
    fn push(&mut self, bytes: &[u8], idx: ::core::ffi::c_int, replace_nl: bool) {
        // The question is only whether the line holds a NL: the C library's
        // vectorised `memchr` answers it in one early-exiting pass (core's
        // costs several times as much), and a line without one (860,000 a run
        // on the memline bench) is handed on as it is.
        let restored = (replace_nl && memchr_in(bytes, b'\n').is_some()).then(|| -> Vec<u8> {
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
                    ApiString::NULL
                } else {
                    ApiString::from_bytes(line)
                };
                array.push(Object::string(line));
            }
        }
    }
}

impl Buf {
    /// Collect `n` lines of `self` from `start` into `lines`, as items
    /// `start_idx..`.
    pub(crate) fn collect_lines(
        self,
        n: size_t,
        start: LineNr,
        start_idx: ::core::ffi::c_int,
        replace_nl: bool,
        lines: &mut LineSink<'_>,
    ) {
        let mut text = self.lines();
        for i in 0..n {
            let offset = len_as_int(i);
            lines.push(text.line(start + offset), start_idx + offset, replace_nl);
        }
    }
}
