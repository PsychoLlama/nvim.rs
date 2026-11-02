//! Whole-line reads and replacements.
//!
//! `nvim_buf_get_lines`/`nvim_buf_set_lines` are the line-granular half of
//! the buffer API, and `nvim_buf_get_text` the read that takes a byte range
//! but still hands back whole lines.  `push_linestr` is the shared
//! line-to-`ApiString` step every one of them ends in, and
//! `buf_collect_lines` the loop over a range that `buffer_updates.rs`
//! reaches too, to build the `nvim_buf_attach` change notifications.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::helpers::api_try;
use crate::lua::stack::{At, Stack};
use crate::memory::memchr_in;
use crate::narrow::number_as_int;
use crate::normal::{visual_active, visual_anchor, with_visual_anchor};
use crate::winlayer::{Buf, tab_windows};

pub fn nvim_buf_line_count(buf: BufferHandle) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as Integer);
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(0 as Integer);
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
    let size: size_t = (end - start) as size_t;
    let at = LineNr::new(number_as_int(start));
    let nl = channel_id != VIML_INTERNAL_CALL;
    let mut lines = LineSink::new(stack, &mut rv, size);
    buf_collect_lines(b, size, at, 0, nl, &mut lines);
    Ok(rv.into())
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
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(());
    };
    let mut oob: bool = false;
    start = normalize_index(buffer, start, true, &mut oob);
    end = normalize_index(buffer, end, true, &mut oob);
    if strict_indexing && oob {
        return Err(Error::validation(c"Index out of bounds"));
    }
    if start > end {
        return Err(Error::validation(c"'start' is higher than 'end'"));
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
    // What the bracket caught outranks the body's own refusal.
    api_try(|| -> Result<(), Error> {
        if buffer.b_p_ma == 0 {
            return Err(Error::exception(c"Buffer is not 'modifiable'"));
        }
        if u_save_buf(
            buffer,
            LineNr::new(number_as_int(start - 1)),
            LineNr::new(number_as_int(end)),
        )
        .is_err()
        {
            return Err(Error::exception(c"Failed to save undo information"));
        }
        let deleted_bytes: BCount = get_region_bytecount(
            buffer,
            LineNr::new(number_as_int(start)),
            LineNr::new(number_as_int(end)),
            0 as ColNr,
            0 as ColNr,
        );
        let to_delete: size_t = if new_len < old_len {
            old_len.wrapping_sub(new_len)
        } else {
            0 as size_t
        };
        let mut i_0: size_t = 0 as size_t;
        while i_0 < to_delete {
            if ml_delete_buf(buffer, LineNr::new(number_as_int(start)), false).is_err() {
                return Err(Error::exception(c"Failed to delete line"));
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
            if !(lnum < i64::from(MAXLNUM)) {
                return Err(Error::validation(c"Index out of bounds"));
            } else if ml_replace_buf_now(buffer, LineNr::new(number_as_int(lnum)), &lines[i_1])
                .is_err()
            {
                return Err(Error::exception(c"Failed to replace line"));
            } else {
                // The line and its NUL.
                inserted_bytes += lines[i_1].len() as BCount;
                i_1 = i_1.wrapping_add(1);
            }
        }
        let mut i_2: size_t = to_replace;
        while i_2 < new_len {
            let lnum_0: int64_t = start as int64_t + i_2 as int64_t - 1 as int64_t;
            if !(lnum_0 < i64::from(MAXLNUM)) {
                return Err(Error::validation(c"Index out of bounds"));
            } else if ml_append_buf_terminated(
                buffer,
                LineNr::new(number_as_int(lnum_0)),
                &lines[i_2],
                false,
            )
            .is_err()
            {
                return Err(Error::exception(c"Failed to insert line"));
            } else {
                // The line and its NUL.
                inserted_bytes += lines[i_2].len() as BCount;
                extra += 1;
                i_2 = i_2.wrapping_add(1);
            }
        }
        let adjust: i32 = if end > start { MAXLNUM.get() } else { 0 };
        mark_adjust_buf(
            buffer,
            LineNr::new(number_as_int(start)),
            LineNr::new(number_as_int(end - 1 as Integer)),
            adjust,
            extra as i32,
            true,
            kMarkAdjustApi,
            kExtmarkNOOP,
        );
        if visual_active() as ::core::ffi::c_int != 0
            && Some(buffer) == Buf::current_or_none()
            && visual_anchor().lnum >= LineNr::new(number_as_int(start))
        {
            if visual_anchor().lnum >= LineNr::new(number_as_int(end)) {
                with_visual_anchor(|a| a.lnum += extra as i32);
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
            LineNr::new(number_as_int(start)),
            0 as ColNr,
            LineNr::new(number_as_int(end)),
            extra as i32,
            true,
        );
        for win in tab_windows() {
            if win.w_buffer == buffer {
                let (lo, hi) = (
                    LineNr::new(number_as_int(start)),
                    LineNr::new(number_as_int(end)),
                );
                fix_cursor(win, lo, hi, extra as i32);
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
    mut start_row: Integer,
    start_col: Integer,
    mut end_row: Integer,
    end_col: Integer,
    _opts: &mut KeyDict_empty,
    stack: Option<&mut Stack>,
) -> Result<Array, Error> {
    let mut rv = Vec::new();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv.into());
    };
    // SAFETY: non-null, so the handle named a live buffer.
    if b.b_ml.ml_mfp.is_none() {
        return Ok(rv.into());
    }
    let mut oob: bool = false;
    start_row = normalize_index(b, start_row as int64_t, false, &mut oob) as Integer;
    end_row = normalize_index(b, end_row as int64_t, false, &mut oob) as Integer;
    if oob {
        return Err(Error::validation(c"Index out of bounds"));
    }
    if !(start_row <= end_row) {
        return Err(Error::validation(c"'start' is higher than 'end'"));
    }
    let replace_nl: bool = channel_id != VIML_INTERNAL_CALL;
    let size: size_t = ((end_row - start_row) as size_t).wrapping_add(1 as size_t);
    let mut lines = LineSink::new(stack, &mut rv, size);
    let first = start_row as int64_t;
    if start_row == end_row {
        let (from, to) = (start_col as int64_t, end_col as int64_t);
        let line: ApiString = buf_get_text(b, first, from, to)?;
        lines.push(line.as_bytes(), 0, replace_nl);
        return Ok(rv.into());
    }
    let from = start_col as int64_t;
    let to = (MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int64_t;
    let line = buf_get_text(b, first, from, to)?;
    lines.push(line.as_bytes(), 0, replace_nl);
    if size > 2 as size_t {
        let n = size.wrapping_sub(2 as size_t);
        let at = LineNr::new(number_as_int(start_row)) + 1;
        buf_collect_lines(b, n, at, 1, replace_nl, &mut lines);
    }
    let last = end_row as int64_t;
    let to = end_col as int64_t;
    let line = buf_get_text(b, last, 0 as int64_t, to)?;
    let at = size.wrapping_sub(1 as size_t) as ::core::ffi::c_int;
    lines.push(line.as_bytes(), at, replace_nl);
    Ok(rv.into())
}

pub fn nvim_buf_get_offset(buf: BufferHandle, index: Integer) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0 as Integer);
    };
    if b.b_ml.ml_mfp.is_none() {
        return Ok(-1 as Integer);
    }
    if !(index >= 0 as Integer && index <= i64::from(b.line_count())) {
        return Err(Error::validation(c"Index out of bounds"));
    }
    let lnum = LineNr::new(number_as_int(index)) + 1;
    let offset = ml_find_line_or_offset(b, lnum, None, true);
    Ok(offset as Integer)
}

/// Where collected lines go: straight into the Lua table on top of the
/// stack, when Lua asked, or into an API array.
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
        let lnum: LineNr = start + i as i32;
        let at = start_idx + i as ::core::ffi::c_int;
        lines.push(text.line(lnum), at, replace_nl);
    }
}
