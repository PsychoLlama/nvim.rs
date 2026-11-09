//! Byte offsets: `line2byte`, `byte2line` and `'go'`'s
//! character count.
//!
//! The tree does not store byte offsets, so `ml_find_line_or_offset` walks it
//! adding up block sizes. `Buf::update_line_chunks` (upstream's `ml_updatechunk`) maintains the `b_ml.ml_chunks`
//! accelerator that keeps that walk from being O(lines) on every call: a run
//! of between `MLCS_MINL` and `MLCS_MAXL` consecutive lines, with their total
//! byte size, so the walk can skip whole runs and only visit blocks inside
//! the run the answer falls in.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::winlayer::Buf;
use crate::winlayer::BufId;
use core::ffi::c_int;

use super::*;
use crate::winlayer::Win;

/// Where `Buf::update_line_chunks` last left off. Appending runs down the buffer in
/// order, so the next call almost always wants the same chunk or the next
/// one, and the search from chunk zero can be skipped.
///
/// Setting `ML_UPD_LASTBUF` to `None` invalidates the lot. An id, not an
/// address: a buffer allocated where a wiped one was must not inherit its
/// cached position.
static ML_UPD_LASTBUF: GlobalCell<Option<BufId>> = GlobalCell::new(None);
static ML_UPD_LASTLINE: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
static ML_UPD_LASTCURLINE: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
static ML_UPD_LASTCURIX: GlobalCell<usize> = GlobalCell::new(0);

impl Buf {
    /// Keep the chunk table up to date for a line that was added, removed or
    /// resized.
    ///
    /// `updtype` is [`ML_CHNK_ADDLINE`] (add `len` to the chunk, splitting it if
    /// it got too long — careful, this can call `ml_find_line`),
    /// [`ML_CHNK_DELLINE`] (subtract `len`, possibly merging the chunk away) or
    /// [`ML_CHNK_UPDLINE`] (add `len` as a signed quantity).
    pub(crate) fn update_line_chunks(self, line: LineNr, len_arg: c_int, updtype: c_int) {
        // SAFETY: the caller's buffer, reached through a handle that
        // borrows it for the one access that asked and no longer.
        let mut b = self;
        let mut curline = ML_UPD_LASTCURLINE.get();
        let mut curix = ML_UPD_LASTCURIX.get();

        if b.b_ml.ml_chunks.is_off() || len_arg == 0 {
            return;
        }
        if !b.b_ml.ml_chunks.is_built() {
            b.b_ml.ml_chunks.build();
        }

        if updtype == ML_CHNK_UPDLINE && b.b_ml.ml_line_count == LineNr::ONE {
            // First line in an empty buffer, from ml_flush_line: reset.
            let textlen = b.b_ml.cached_len();
            b.b_ml.ml_chunks.reset_to_one(textlen.get());
            return;
        }

        // Find the chunk the line belongs to; `curline` ends up at the start
        // of it.
        let key = self.try_id();
        if key.is_none()
            || ML_UPD_LASTBUF.get() != key
            || line != ML_UPD_LASTLINE.get() + 1
            || updtype != ML_CHNK_ADDLINE
        {
            curline = LineNr::ONE;
            curix = 0;
            while curix + 1 < b.b_ml.ml_chunks.len()
                && line >= curline + b.b_ml.ml_chunks.lines(curix)
            {
                curline += b.b_ml.ml_chunks.lines(curix);
                curix += 1;
            }
        } else if curix + 1 < b.b_ml.ml_chunks.len()
            && line >= curline + b.b_ml.ml_chunks.lines(curix)
        {
            // The cached position is one chunk stale; step it on.
            curline += b.b_ml.ml_chunks.lines(curix);
            curix += 1;
        }

        let len = if updtype == ML_CHNK_DELLINE {
            -len_arg
        } else {
            len_arg
        };
        b.b_ml.ml_chunks.add_size(curix, len);

        if updtype == ML_CHNK_ADDLINE {
            if !ml_chunk_addline(self, line, curline, curix) {
                return;
            }
        } else if updtype == ML_CHNK_DELLINE {
            ML_UPD_LASTBUF.set(None); // force a recalc
            b.b_ml.ml_chunks.delete_line(curix, MLCS_MINL);
            return;
        }

        ML_UPD_LASTBUF.set(key);
        ML_UPD_LASTLINE.set(line);
        ML_UPD_LASTCURLINE.set(curline);
        ML_UPD_LASTCURIX.set(curix);
    }
}

/// A line was added to chunk `curix`. Returns false if the caller must return
/// without caching its position — either the chunk was split (which
/// invalidates the cache) or the walk failed.
///
/// # Panics
/// When the chunk index has no chunk `curix`.
fn ml_chunk_addline(mut b: Buf, line: LineNr, curline: LineNr, curix: usize) -> bool {
    b.b_ml.ml_chunks.add_lines(curix, 1);

    if b.b_ml.ml_chunks.lines(curix) >= MLCS_MAXL {
        return ml_chunk_split(b, curix, curline);
    }

    if b.b_ml.ml_chunks.lines(curix) >= MLCS_MINL
        && curix + 1 == b.b_ml.ml_chunks.len()
        && b.b_ml.ml_line_count - line <= 1
    {
        // This is the last chunk and starting another one after it is
        // cheap right now, so do it and save the walk later on.
        b.b_ml.ml_chunks.push_empty();
        if line != b.b_ml.ml_line_count {
            // The line is just before the last one, so move the last
            // line's size over. This is the common case while loading a
            // file.
            let Some(id) = ml_find_line(b, b.b_ml.ml_line_count, ML_FIND) else {
                b.b_ml.ml_chunks.switch_off();
                return false;
            };
            let data = DataBlock(b.b_ml.page(id));
            let rest = if data.line_count() == 1 {
                data.txt_end().wrapping_sub(data.txt_start()).cast_signed()
            } else {
                let last_but_one =
                    usize::try_from(data.line_count() - 2).expect("two lines or more");
                data.line_start(last_but_one).cast_signed() - data.txt_start().cast_signed()
            };
            b.b_ml.ml_chunks.set(curix + 1, 1, rest);
            b.b_ml.ml_chunks.add_size(curix, -rest);
            b.b_ml.ml_chunks.add_lines(curix, -1);
        }
    }
    true
}

/// Chunk `curix` grew past [`MLCS_MAXL`] lines: cut it in two at
/// [`MLCS_MINL`], measuring the first half by walking the data blocks it
/// covers.
///
/// Always returns false — either it failed, or it invalidated the cached
/// position. Both mean the caller returns without caching.
///
/// # Panics
/// When the chunk index has no chunk `curix`.
fn ml_chunk_split(mut b: Buf, curix: usize, curline_arg: LineNr) -> bool {
    b.b_ml.ml_chunks.split_at(curix);

    // Total size of the first MLCS_MINL lines of the chunk.
    let mut curline = curline_arg;
    let mut size = 0;
    let mut linecnt = 0;
    while curline < b.b_ml.ml_line_count && linecnt < MLCS_MINL {
        let Some(id) = ml_find_line(b, curline, ML_FIND) else {
            b.b_ml.ml_chunks.switch_off();
            return false;
        };
        let count = b.b_ml.locked_high() - b.b_ml.locked_low() + 1;
        let idx = curline - b.b_ml.locked_low();
        curline = b.b_ml.locked_high() + 1;

        // Index of the last line of this block that still counts.
        let rest = count - idx;
        let end_idx = if linecnt + rest > MLCS_MINL {
            let end_idx = idx + MLCS_MINL - linecnt - 1;
            linecnt = MLCS_MINL;
            end_idx
        } else {
            linecnt += rest;
            count - 1
        };

        // First line in the block has its text at the end of it.
        let data = DataBlock(b.b_ml.page(id));
        let text_end = data.line_end(idx.cast_unsigned() as usize).cast_signed();
        size += text_end
            - data
                .line_start(end_idx.cast_unsigned() as usize)
                .cast_signed();
    }

    b.b_ml.ml_chunks.add_lines(curix + 1, -linecnt);
    b.b_ml.ml_chunks.add_size(curix + 1, -size);
    b.b_ml.ml_chunks.set(curix, linecnt, size);
    ML_UPD_LASTBUF.set(None); // force a recalc
    false
}

impl Buf {
    /// Translate between line numbers and byte offsets, in whichever direction
    /// the caller left blank.
    ///
    /// With `lnum > 0` this returns the byte offset of the start of that line
    /// (and `offp` should be `None`). With `lnum == 0` it returns the line holding
    /// byte offset `*offp`, and writes the remaining column offset back through
    /// `offp`.
    ///
    /// `no_ff` counts one byte per line break whatever `'fileformat'` says, which
    /// is what byte tracking wants.
    ///
    /// `None` when there is nothing to answer from: no memline, a line past
    /// the end, or an offset one byte beyond it.
    ///
    /// The running byte count is 64 bits wide. Upstream's is an `int`, which a
    /// buffer past 2 GiB overflows: C wraps (by accident of the compiler) and a
    /// debug build of the port panicked. The walk now counts and compares
    /// without overflowing, and only the answer is cut to the `int` the callers
    /// take — wrapping, as upstream's did, so a file past 2 GiB still answers
    /// what it always answered for the offset of a line beyond that point.
    pub fn find_line_or_offset(
        self,
        lnum: LineNr,
        mut offp: Option<&mut c_int>,
        no_ff: bool,
    ) -> Option<c_int> {
        let mut b = self;
        let ffdos = Wide::from(!no_ff && get_fileformat(b) == EOL_DOS);
        let mut extra: i32 = 0;

        // Take care of the cached line first, and only if it is before the
        // requested line. The value for the cached line is then cached in
        // turn: the extmark code wants the byte offset of the line being
        // edited, so a run of small edits to one line computes it once.
        //
        // Caching does not work with 'fileformat', which is no problem for
        // byte tracking (it ignores 'fileformat'), but a line2byte() call
        // does invalidate the cache for the time being.
        let can_cache = lnum != LineNr::ZERO && ffdos == 0 && b.b_ml.cached_lnum() == lnum;
        if lnum == LineNr::ZERO || b.b_ml.cached_lnum() < lnum || !no_ff {
            ml_flush_line(Buf::current());
        } else if can_cache && b.b_ml.cached_offset() > 0 {
            // Cut to the callers' `int`, as below.
            return Some(
                low32(i64::try_from(b.b_ml.cached_offset()).unwrap_or(i64::MAX)).cast_signed(),
            );
        }

        if b.b_ml.ml_chunks.is_off() || !b.b_ml.ml_chunks.is_built() || lnum < LineNr::ZERO {
            // The memline is empty. If it is open at all it still behaves as
            // though it held one empty line.
            if no_ff && b.b_ml.is_open() && (lnum == LineNr::ONE || lnum == LineNr::new(2)) {
                return Some((lnum - 1).get());
            }
            return None;
        }

        let offset = Wide::from(offp.as_deref().copied().unwrap_or(0));
        if lnum == LineNr::ZERO && offset <= 0 {
            // Not a "find offset", and offset 0 must be in line 1.
            return Some(1);
        }

        // Skip whole chunks, up to but not including the one the answer is
        // in. The last chunk is special: it never qualifies.
        let chunks = &self.b_ml.ml_chunks;
        let lines_in = |at: usize| Wide::from(chunks.lines(at));
        let size_of_chunk = |at: usize| Wide::from(chunks.size(at));
        let mut curline: LineNr = LineNr::ONE;
        let mut curix = 0usize;
        let mut size: Wide = 0;
        while curix + 1 < chunks.len()
            && (lnum != LineNr::ZERO && lnum >= curline + chunks.lines(curix)
                || offset != 0 && offset > size + size_of_chunk(curix) + ffdos * lines_in(curix))
        {
            curline += chunks.lines(curix);
            size += size_of_chunk(curix);
            if offset != 0 && ffdos != 0 {
                size += lines_in(curix);
            }
            curix += 1;
        }

        // Then walk the data blocks of that chunk line by line.
        while lnum != LineNr::ZERO && curline < lnum || offset != 0 && size < offset {
            // The bounds test has to come first: ml_find_line complains about
            // a line past the end.
            if curline > b.b_ml.ml_line_count {
                return None;
            }
            let id = ml_find_line(self, curline, ML_FIND)?;
            let count = b.b_ml.locked_high() - b.b_ml.locked_low() + 1;
            let start_idx = curline - b.b_ml.locked_low();
            let mut idx = start_idx;
            let page = self.b_ml.page(id);
            let data = DataBlock(page);
            let starts = data.index_entries(count.cast_unsigned() as usize);
            let start_of = |idx: c_int| {
                Wide::from(u32::from_ne_bytes(starts[idx.cast_unsigned() as usize]) & DB_INDEX_MASK)
            };
            // First line in the block has its text at the end of it.
            let text_end = Wide::from(data.line_end(idx.cast_unsigned() as usize));

            // Index of the last line of this block to account for.
            if lnum != LineNr::ZERO {
                if curline + (count - idx) >= lnum {
                    idx += lnum - curline - 1;
                } else {
                    idx = count - 1;
                }
            } else {
                extra = 0;
                while offset >= size + text_end - start_of(idx) + ffdos {
                    if ffdos != 0 {
                        size += 1;
                    }
                    if idx == count - 1 {
                        // The offset is past the last line of the block.
                        extra = 1;
                        break;
                    }
                    idx += 1;
                }
            }

            let len = text_end - start_of(idx);
            size += len;
            if offset != 0 && size >= offset {
                // A non-zero offset was read through `offp`, so it is there.
                let offp = offp.as_deref_mut().expect("an offset to answer through");
                // A column inside one line of a block, so it fits.
                *offp = if size + ffdos == offset {
                    0
                } else if idx == start_idx {
                    low32(offset - size + len).cast_signed()
                } else {
                    low32(offset - size + len - (text_end - start_of(idx - 1))).cast_signed()
                };
                curline += idx - start_idx + extra;
                if curline > b.b_ml.ml_line_count {
                    return None; // exactly one byte beyond the end
                }
                return Some(curline.get());
            }
            curline = b.b_ml.locked_high() + 1;
        }

        if lnum != LineNr::ZERO {
            // Count the extra CR characters.
            if ffdos != 0 {
                size += Wide::from(lnum - 1);
            }
            // Do not count the last line break with 'noeol' and either 'bin'
            // or 'nofixeol'.
            if (b.b_p_fixeol == 0 || b.b_p_bin != 0)
                && b.b_p_eol == 0
                && lnum > b.b_ml.ml_line_count
            {
                size -= ffdos + 1;
            }
        }

        if can_cache && size > 0 {
            b.b_ml
                .set_cached_offset(usize::try_from(size).expect("checked positive"));
        }
        // Cut to the callers' `int`, wrapping as upstream's arithmetic did.
        Some(low32(size).cast_signed())
    }
}

/// The byte count [`Buf::find_line_or_offset`] keeps while it walks.
type Wide = i64;

/// Move the cursor to byte `cnt` of the buffer.
pub fn goto_byte(cnt: c_int) {
    let mut boff = cnt;
    ml_flush_line(Buf::current()); // the cached line may be dirty
    setpcmark();
    if boff != 0 {
        boff -= 1;
    }
    let found = Buf::current().find_line_or_offset(LineNr::ZERO, Some(&mut boff), false);
    if let Some(lnum) = found.filter(|&lnum| lnum >= 1) {
        Win::current().w_cursor.lnum = LineNr::new(lnum);
        Win::current().w_cursor.col = ColNr::new(boff);
        Win::current().w_cursor.coladd = 0;
        Win::current().w_set_curswant = true;
    } else {
        // Past the end.
        Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
        Win::current().w_curswant = ColNr::MAX;
        coladvance(Win::current(), ColNr::MAX);
    }
    check_cursor(Win::current());

    // Make sure the cursor is on the first byte of a multi-byte char.
    mb_adjust_cursor();
}
