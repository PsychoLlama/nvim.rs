//! Recomputing a tabpage's diff.
//!
//! [`ex_diffupdate`] is `:diffupdate`; [`diff_try_update`] is the body it and
//! every implicit recompute share -- write each buffer out
//! ([`diff_write_buffer`] for the internal engine, [`diff_write`] through a
//! temp file for the external one), run the diff, read the hunks back.
//!
//! `'diffanchors'` is implemented here rather than in the engine: the
//! buffers are split at the anchor lines and each segment is diffed on its
//! own, then the resulting block lists are shifted back into place and
//! concatenated.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_flags, cmdmod_set_flags};
use crate::memline::MlFlags;
use crate::types::{FAIL, NUL, OK};
use crate::winlayer::{Buf, Live, TabPage, Win};
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use std::ffi::CStr;

/// One block of a tab page's diff list, as a pointer the caller has promised
/// is live.
///
/// The list hangs off `tabpage_T::tp_first_diff` and is chained through
/// `df_next`; every reader of it walks the chain asking each block for its
/// `df_lnum`/`df_count` in one buffer or another. Naming the pointer makes
/// those ordinary field accesses, and leaves the two places the chain is
/// really dereferenced -- [`Df::next`] and [`diff_blocks`] -- as the only
/// unchecked ones.
///
/// A `Df` is a record of the caller's promise and not evidence for one: the
/// list is rebuilt by `ex_diffupdate` and blocks are freed by `diff_free`, so
/// a `Df` held across either is dangling. See [`Live`]'s module docs.
pub(crate) type Df = Live<diff_T>;

impl Df {
    /// The block `dp` names, `None` for null.
    ///
    /// # Safety
    /// `dp` must be null, or stay a live block for as long as the value is
    /// used.
    pub(crate) unsafe fn from_raw(dp: *mut diff_T) -> Option<Self> {
        // SAFETY: the caller's promise, for the non-null half.
        (!dp.is_null()).then(|| unsafe { Self::new(dp) })
    }

    /// The first block of `tp`'s list, if any.
    pub(crate) fn first(tp: TabPage) -> Option<Self> {
        // SAFETY: a live tab page's block list is live.
        unsafe { Self::from_raw(tp.tp_first_diff) }
    }

    /// The next block in the list, if any.
    pub(crate) fn next(self) -> Option<Self> {
        // SAFETY: a live block's `df_next` is a live block or null.
        unsafe { Self::from_raw(self.df_next) }
    }

    /// The line one past this block's last, in buffer `idx`.
    ///
    /// A block covers `df_count[idx]` lines from `df_lnum[idx]`, so this is
    /// the line the next block may start at -- the sum the walks all compare
    /// against.
    pub(crate) fn end(self, idx: usize) -> linenr_T {
        self.df_lnum[idx] + self.df_count[idx]
    }

    /// The block's longest side, in lines: how many screen rows it takes in
    /// every window showing it. [`get_max_diff_length`].
    pub(crate) fn max_len(self) -> c_int {
        // SAFETY: a live block.
        unsafe { get_max_diff_length(self.raw()) }
    }

    /// Whether buffers `idx1` and `idx2` hold the same text over this block.
    /// [`diff_equal_entry`].
    pub(crate) fn equal_entry(self, idx1: usize, idx2: usize) -> bool {
        // SAFETY: a live block.
        unsafe { diff_equal_entry(self.raw(), idx1, idx2) }
    }

    /// Whether the block's line numbers still fit the buffers it names.
    /// [`diff_check_sanity`].
    pub(crate) fn is_sane(self, tp: TabPage) -> bool {
        // SAFETY: a live block and a live tab page.
        unsafe { diff_check_sanity(tp, self.raw()) != 0 }
    }
}

/// A tab page's diff blocks, first to last.
///
/// Nothing re-reads `tp_first_diff`, exactly as the C's own walks do not: a
/// loop that can free or rebuild the list underneath itself keeps its own
/// cursor instead.
pub(crate) fn diff_blocks(tp: TabPage) -> impl Iterator<Item = Df> {
    core::iter::successors(Df::first(tp), |dp| dp.next())
}

/// `buf`'s slot in `tp`'s diff, or `DB_COUNT` if it has none.
///
/// [`diff_buf_idx`] with its promise discharged: it only compares `buf`
/// against the tab page's eight `tp_diffbuf` slots and never dereferences it,
/// so a live tab page is the whole precondition -- which is what a [`TabPage`]
/// argument says. That matters because half the callers ask about a buffer
/// they are not otherwise sure of.
pub(crate) fn diff_slot(buf: Buf, tp: TabPage) -> c_int {
    diff_buf_idx(buf, tp)
}

/// Whether `dp` is still in the current tab page's block list.
///
/// [`valid_diff`], likewise safe: the walk compares `dp` against the live
/// list without reading it, which is the point -- it is asked exactly when
/// `dp` may have been freed underneath the caller.
pub(crate) fn diff_still_listed(dp: *mut diff_T) -> bool {
    // SAFETY: the current tab page is live; `dp` is compared, never read.
    unsafe { valid_diff(dp) }
}

/// Release one side of a diff: the temp file if there is one, else the
/// memory image.
///
/// # Safety
/// `din` must be a live input side of a diff run.
unsafe fn clear_diffin(din: *mut diffin_T) {
    // SAFETY: the caller's input side.
    let mut din = unsafe { Live::<diffin_T>::new(din) };
    if din.din_fname.is_null() {
        // SAFETY: the memory image is this module's own allocation.
        unsafe { xfree(din.din_mmfile.ptr.cast()) };
        din.din_mmfile.ptr = ::core::ptr::null_mut();
    } else {
        // SAFETY: one of this module's own temp file names.
        unsafe { os_remove(din.din_fname) };
    }
}

/// Release the diff's output: the temp file if there is one, else the hunk
/// array.
///
/// # Safety
/// `dout` must be a live output side of a diff run.
pub(crate) unsafe fn clear_diffout(dout: *mut diffout_T) {
    // SAFETY: the caller's output side.
    let dout = unsafe { Live::<diffout_T>::new(dout) };
    if dout.dout_fname.is_null() {
        let ga = dout.field_ptr(offset_of!(diffout_T, dout_ga));
        // SAFETY: the hunk array is `dout`'s own field.
        unsafe { ga_clear(ga) };
    } else {
        // SAFETY: one of this module's own temp file names.
        unsafe { os_remove(dout.dout_fname) };
    }
}

/// Write lines `start`..`end` of `buf` into a memory image for `xdl_diff`.
///
/// The image is one NL-terminated line after another.  A NL *inside* a line
/// stands for a NUL byte in the file -- `ml_get_buf` answers the two swapped
/// -- and is written back as one, so the terminators are unambiguous.
/// `icase` is applied here, by folding each character, because xdiff has no
/// flag for it.
///
/// # Safety
/// `m` must be a writable `mmfile_t`.
pub(crate) unsafe fn diff_write_buffer(
    buf: Buf,
    m: *mut mmfile_t,
    start: linenr_T,
    mut end: linenr_T,
) -> c_int {
    if end < 0 {
        end = buf.b_ml.ml_line_count;
    }
    if buf.b_ml.ml_flags.has(MlFlags::EMPTY) || end < start {
        // SAFETY: the caller's out-parameter.
        unsafe { *m = MMFILE_INIT };
        return OK;
    }

    let len = (start..=end)
        .map(|lnum| {
            // SAFETY: a live buffer, and a line number inside it.
            let n = unsafe { ml_get_buf_len(buf.raw(), lnum) };
            n as usize + 1
        })
        .sum::<usize>();
    // SAFETY: `xmalloc` aborts rather than answer null.
    let ptr = unsafe { xmalloc(len) }.cast::<c_char>();
    let image = mmfile_t {
        ptr,
        size: len as c_int,
    };
    // SAFETY: the caller's out-parameter.
    unsafe { *m = image };
    // SAFETY: `len` bytes were just allocated, and nothing else holds them
    // until the caller frees the image.
    let out = unsafe { ::core::slice::from_raw_parts_mut(ptr.cast::<u8>(), len) };

    let mut at = 0;
    for lnum in start..=end {
        // SAFETY: a live buffer, and a line number inside it.
        let line = unsafe { CStr::from_ptr(ml_get_buf(buf.raw(), lnum)) }.to_bytes();
        if diff_flags.get() & DIFF_ICASE == 0 {
            out[at..at + line.len()].copy_from_slice(line);
            let from = out[at..].as_mut_ptr().cast();
            // SAFETY: the `line.len()` bytes just copied to `at`.
            unsafe { memchrsub(from, NL as c_char, NUL as c_char, line.len()) };
            at += line.len();
        } else {
            at += fold_line(line, &mut out[at..]);
        }
        out[at] = NL as u8;
        at += 1;
    }
    OK
}

/// Copy `line` into `out` with every character case-folded, answering how
/// many bytes were written.
///
/// Always exactly `line.len()`: where the folded form is a different length
/// from the original the *original* is written instead, which is what keeps
/// the caller's precomputed allocation exact.  A NUL byte stands for a NL,
/// and folds to itself.
fn fold_line(line: &[u8], out: &mut [u8]) -> usize {
    let mut at = 0;
    while at < line.len() {
        let s = line[at..].as_ptr().cast::<c_char>();
        let (folded, c_len) = if line[at] == NL as u8 {
            (NUL, 1)
        } else {
            // SAFETY: `s` points into a NUL-terminated line, which is what
            // the multibyte readers walk.
            let c = unsafe { utf_ptr2char(s) };
            (utf_fold(c), utf_char2len(c))
        };
        // SAFETY: as above.
        let orig_len = unsafe { utfc_ptr2len(s) } as usize;
        // MB_MAXBYTES + 1.
        let mut cbuf = [0u8; 22];
        let dst = cbuf.as_mut_ptr().cast();
        // SAFETY: `cbuf` is longer than the longest encoding.
        let same_len = unsafe { utf_char2bytes(folded, dst) } == c_len;
        if same_len {
            let c_len = c_len as usize;
            out[at..at + c_len].copy_from_slice(&cbuf[..c_len]);
            if orig_len > c_len {
                // Composing characters follow; they are not folded.
                out[at + c_len..at + orig_len].copy_from_slice(&line[at + c_len..at + orig_len]);
            }
        } else {
            out[at..at + orig_len].copy_from_slice(&line[at..at + orig_len]);
        }
        at += orig_len;
    }
    at
}

/// Write lines `start`..`end` of `buf` out for the external diff.
///
/// The internal engine wants a memory image, which is what `din_fname` being
/// NULL selects; otherwise the lines go through a temp file.
///
/// # Safety
/// `din` must be a live input side.
unsafe fn diff_write(
    mut buf: Buf,
    din: *mut diffin_T,
    start: linenr_T,
    mut end: linenr_T,
) -> c_int {
    // SAFETY: the caller's input side.
    let mut din = unsafe { Live::<diffin_T>::new(din) };
    if din.din_fname.is_null() {
        let image = din.field_ptr(offset_of!(diffin_T, din_mmfile));
        // SAFETY: the caller's buffer, and `din`'s own image field.
        return unsafe { diff_write_buffer(buf, image, start, end) };
    }
    // Writing a buffer runs `aucmd_prepbuf`/`aucmd_restbuf`, which can
    // change the window layout -- and re-entering `winframe_remove` is a
    // use after free.
    if frames_locked() {
        return FAIL;
    }
    if end < 0 {
        end = buf.b_ml.ml_line_count;
    }

    let was_empty = buf.b_ml.ml_flags.masked(MlFlags::EMPTY);
    let save_ff = buf.b_p_ff;
    // The diff must see the file the way the buffer holds it.
    // SAFETY: a static string; `xstrdup` aborts rather than fail.
    buf.b_p_ff = unsafe { xstrdup(c"unix".as_ptr()) };
    // Writing the buffer is an implementation detail of the diff, so it
    // must not move the '[ and '] marks.
    //
    // Upstream saves the whole `cmod_flags` bit set into a `bool` and
    // restores it from there, so every flag that was set comes back as
    // the single bit 1. Reproduced; see O-B15-17.
    let save_cmod_flags = !cmdmod_flags().is_empty();
    cmdmod_add_flags(CmdModFlags::LOCKMARKS);
    if end < start {
        // The range names a completely empty file.
        end = start;
        buf.b_ml.ml_flags |= MlFlags::EMPTY;
    }
    let name = din.din_fname;
    let req = WriteRequest::filter();
    let noshort = ::core::ptr::null_mut::<c_char>();
    let noeap = ::core::ptr::null_mut::<exarg_T>();
    // SAFETY: a live buffer and one of this module's temp file names; no
    // short name and no `exarg_T` are wanted.
    let r = unsafe { buf_write(buf.raw(), name, noshort, start, end, noeap, req) };
    cmdmod_set_flags(CmdModFlags::SANDBOX.when(save_cmod_flags));
    // SAFETY: the option string the buffer itself holds.
    unsafe { free_string_option(buf.b_p_ff) };
    buf.b_p_ff = save_ff;
    buf.b_ml.ml_flags = buf.b_ml.ml_flags.without(MlFlags::EMPTY) | was_empty;
    r
}

/// Recompute the current tabpage's blocks with `idx_orig` as the reference
/// buffer.
///
/// Every other buffer is diffed against that one in turn, and `diff_read`
/// merges each answer into the shared block list.  With `'diffopt'`'s
/// `anchor` the whole thing runs once per segment between anchors, and the
/// segments' block lists are shifted back into place and chained together.
///
/// # Safety
/// `dio` must be a live diff run, and `eap` null or a live command.
unsafe fn diff_try_update(dio: *mut diffio_T, idx_orig: c_int, eap: *mut exarg_T) {
    // SAFETY: the caller's diff run.
    let mut dio = unsafe { Live::<diffio_T>::new(dio) };
    let orig_in: *mut diffin_T = dio.field_ptr(offset_of!(diffio_T, dio_orig));
    let new_in: *mut diffin_T = dio.field_ptr(offset_of!(diffio_T, dio_new));
    let diff_out: *mut diffout_T = dio.field_ptr(offset_of!(diffio_T, dio_diff));
    let mut tp = cur_tab();
    let idx_orig = idx_orig as usize;
    let mut anchors = [[0 as linenr_T; MAX_DIFF_ANCHORS as usize]; DB_COUNT as usize];
    'theend: {
        if dio.dio_internal != 0 {
            let ga = dio.field_ptr(offset_of!(diffio_T, dio_diff.dout_ga));
            let item = ::core::mem::size_of::<diffhunk_T>() as c_int;
            // SAFETY: the hunk array is `dio`'s own field.
            unsafe { ga_init(ga, item, 100) };
        } else {
            // SAFETY: the editor exists, for all three names.
            dio.dio_orig.din_fname = unsafe { vim_tempname() };
            dio.dio_new.din_fname = unsafe { vim_tempname() };
            dio.dio_diff.dout_fname = unsafe { vim_tempname() };
            if dio.dio_orig.din_fname.is_null()
                || dio.dio_new.din_fname.is_null()
                || dio.dio_diff.dout_fname.is_null()
                // SAFETY: the caller's diff run.
                || unsafe { check_external_diff(dio.raw()) } == FAIL
            {
                break 'theend;
            }
        }

        // `:diffupdate!` re-reads any buffer that changed on disk first.
        // SAFETY: the caller's command, when there is one.
        let forceit = !eap.is_null() && unsafe { (*eap).forceit } != 0;
        if forceit {
            for idx in idx_orig..DB_COUNT as usize {
                let buf = tp.tp_diffbuf[idx];
                // SAFETY: `buf_valid` compares against the live buffer list,
                // and a valid buffer is what `buf_check_timestamp` wants.
                if unsafe { buf_valid(buf) } {
                    unsafe { buf_check_timestamp(Buf::new(buf)) };
                }
            }
        }

        // Every buffer has to supply the same number of anchors, or the
        // segments would not line up; the smallest count wins, and a
        // buffer whose `'diffanchors'` does not resolve cancels them all.
        let mut num_anchors = c_int::MAX;
        if diff_flags.get() & DIFF_ANCHOR != 0 {
            for idx in 0..DB_COUNT as usize {
                if tp.tp_diffbuf[idx].is_null() {
                    continue;
                }
                let mut buf_num_anchors = 0;
                let into = anchors[idx].as_mut_ptr();
                let count = &raw mut buf_num_anchors;
                let buf = tp.tp_diffbuf[idx];
                // SAFETY: a live buffer, and two locals of this frame with
                // room for `MAX_DIFF_ANCHORS` line numbers.
                let ok = unsafe { parse_diffanchors(false, Buf::new(buf), into, count) };
                if ok != OK {
                    let msg = &raw const e_failed_to_find_all_diff_anchors as *const c_char;
                    // SAFETY: a static message string.
                    unsafe { emsg(gettext(msg)) };
                    num_anchors = 0;
                    anchors = [[0; MAX_DIFF_ANCHORS as usize]; DB_COUNT as usize];
                    break;
                }
                num_anchors = num_anchors.min(buf_num_anchors);
                if buf_num_anchors > 0 {
                    anchors[idx][..buf_num_anchors as usize].sort_unstable();
                }
            }
        }
        if num_anchors == c_int::MAX {
            num_anchors = 0;
        }

        // One diff per segment: `[1, a1)`, `[a1, a2)`, … `[aN, end]`.
        for anchor_i in 0..=num_anchors as usize {
            let segment = |idx: usize| {
                (
                    if anchor_i == 0 {
                        1
                    } else {
                        anchors[idx][anchor_i - 1]
                    },
                    if anchor_i == num_anchors as usize {
                        -1
                    } else {
                        anchors[idx][anchor_i] - 1
                    },
                )
            };
            // Each segment builds its own list, which is appended to the
            // ones before it once its line numbers have been corrected.
            let orig_diff = if anchor_i == 0 {
                ::core::ptr::null_mut()
            } else {
                let head = tp.tp_first_diff;
                tp.tp_first_diff = ::core::ptr::null_mut();
                head
            };

            let (start, end) = segment(idx_orig);
            // SAFETY: a live buffer of the diff, and `dio`'s own input side.
            let wrote =
                unsafe { diff_write(Buf::new(tp.tp_diffbuf[idx_orig]), orig_in, start, end) };
            if wrote == FAIL {
                if !orig_diff.is_null() {
                    tp.tp_first_diff = orig_diff;
                    // SAFETY: the current tab page is live.
                    diff_clear(tp);
                }
                break 'theend;
            }
            for idx_new in idx_orig + 1..DB_COUNT as usize {
                let buf = tp.tp_diffbuf[idx_new];
                // SAFETY: a live buffer of the diff, or null.
                if buf.is_null() || unsafe { (*buf).b_ml.ml_mfp.is_null() } {
                    continue;
                }
                let (start, end) = segment(idx_new);
                // SAFETY: a live buffer, and `dio`'s own sides.
                if unsafe { diff_write(Buf::new(buf), new_in, start, end) } != FAIL
                    && unsafe { diff_file(dio.raw()) } != FAIL
                {
                    unsafe { diff_read(idx_orig as c_int, idx_new as c_int, dio.raw()) };
                    unsafe { clear_diffin(new_in) };
                    unsafe { clear_diffout(diff_out) };
                }
            }
            // SAFETY: `dio`'s own input side.
            unsafe { clear_diffin(orig_in) };

            if anchor_i == 0 {
                continue;
            }
            // This segment's diff was computed over lines starting at 1;
            // shift it down to where the segment really begins.
            for mut dp in diff_blocks(tp) {
                for (idx, row) in anchors.iter().enumerate() {
                    let anchor = row[anchor_i - 1];
                    if anchor > 0 {
                        dp.df_lnum[idx] += anchor - 1;
                    }
                }
            }
            // SAFETY: the head of the previous segments' list, which this
            // function built and has not freed.
            if let Some(head) = unsafe { Df::from_raw(orig_diff) } {
                let mut last = head;
                while let Some(next) = last.next() {
                    last = next;
                }
                last.df_next = tp.tp_first_diff;
                tp.tp_first_diff = orig_diff;
            }
        }
    }
    // SAFETY: `dio`'s own temp file names, or null.
    unsafe { xfree(dio.dio_orig.din_fname.cast()) };
    unsafe { xfree(dio.dio_new.din_fname.cast()) };
    unsafe { xfree(dio.dio_diff.dout_fname.cast()) };
}

/// Whether the built-in diff engine is what a recompute would use.
///
/// `'diffexpr'` overrides `'diffopt'`'s `internal`.
///
/// # Safety
/// The editor must be running.
pub unsafe fn diff_internal() -> c_int {
    // SAFETY: `p_dex` is the `'diffexpr'` option string.
    let no_diffexpr = unsafe { *p_dex.get() } == 0;
    c_int::from(diff_flags.get() & DIFF_INTERNAL != 0 && no_diffexpr)
}

/// `:diffupdate`, and every implicit recompute.
///
/// # Safety
/// `eap` must be null or a live command.
pub unsafe fn ex_diffupdate(eap: *mut exarg_T) {
    // A recompute asked for from inside `:diffget`/`:diffput` is deferred
    // to that command's tail, where `diff_need_update` is read.
    if diff_busy.get() {
        diff_need_update.set(true);
        return;
    }
    let mut tp = cur_tab();
    let had_diffs = !tp.tp_first_diff.is_null();
    // SAFETY: the current tab page is live.
    diff_clear(tp);
    tp.tp_diff_invalid = 0;

    // The first two buffers in the tabpage: everything is diffed against
    // the first, so there is nothing to do without a second.
    let first_two = (0..DB_COUNT)
        .find(|&i| !tp.tp_diffbuf[i as usize].is_null())
        .filter(|&idx_orig| (idx_orig + 1..DB_COUNT).any(|i| !tp.tp_diffbuf[i as usize].is_null()));
    if let Some(idx_orig) = first_two {
        // SAFETY: the editor exists.
        let internal = unsafe { diff_internal() };
        let mut diffio = diffio_T {
            dio_orig: DIFFIN_INIT,
            dio_new: DIFFIN_INIT,
            dio_diff: diffout_T {
                dout_fname: ::core::ptr::null_mut(),
                dout_ga: GA_EMPTY_INIT_VALUE,
            },
            dio_internal: internal,
        };
        // SAFETY: `diffio` is a local, and `eap` is the caller's command.
        unsafe { diff_try_update(&raw mut diffio, idx_orig, eap) };
        cur_win().w_valid_cursor.lnum = 0;
    }

    if had_diffs || !tp.tp_first_diff.is_null() {
        let nul = ::core::ptr::null_mut::<c_char>();
        // SAFETY: the editor exists; `DiffUpdated` takes no file name.
        unsafe { diff_redraw(true) };
        unsafe { apply_autocmds(EVENT_DIFFUPDATED, nul, nul, false, curbuf.get()) };
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}
