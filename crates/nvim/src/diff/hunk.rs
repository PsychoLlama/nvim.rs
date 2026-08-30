//! Turning diff output back into blocks.
//!
//! [`diff_read`] walks whatever the engine produced -- unified or `ed`-style
//! text from an external diff, or the hunks `xdiff_out` collected from the
//! internal one -- and [`process_hunk`] merges each into the tabpage's block
//! list, growing, splitting or joining the existing blocks as the new range
//! overlaps them.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::Failed;
use crate::winlayer::TabPage;
use core::ffi::{c_char, c_int};

/// Which format an external diff answered in.
///
/// Not known until the first line that says so, which is why it is threaded
/// through [`extract_hunk`] rather than decided once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DiffStyle {
    /// Not yet decided.
    Unknown,
    /// `2,4c2,4`.
    Ed,
    /// `@@ -2,3 +2,3 @@`.
    Unified,
}

/// Where the walk over the block list has got to.
///
/// `notset` says the block `dp` points at has *not* yet had `idx_new`'s
/// range written into it, which is what tells the merge whether it is
/// looking at a block from an earlier pass or one it just made.
struct Walk {
    dp: *mut diff_T,
    dprev: *mut diff_T,
    notset: bool,
}

/// Take the next hunk the internal engine produced.  Answers end of input.
unsafe fn extract_hunk_internal(
    dout: *mut diffout_T,
    hunk: *mut diffhunk_T,
    line_idx: &mut c_int,
) -> bool {
    if *line_idx >= unsafe { (*dout).dout_ga.ga_len } {
        return true;
    }
    unsafe { *hunk = *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset(*line_idx as isize) };
    *line_idx += 1;
    false
}

/// Read lines from `fd` until one parses as a hunk header.  Answers end of
/// input.
unsafe fn extract_hunk(fd: *mut FILE, hunk: *mut diffhunk_T, diffstyle: &mut DiffStyle) -> bool {
    loop {
        let mut line = [0 as c_char; LBUFLEN as usize];
        if unsafe { vim_fgets(line.as_mut_ptr(), LBUFLEN, fd) } {
            return true;
        }
        if *diffstyle == DiffStyle::Unknown {
            if (line[0] as u8).is_ascii_digit() {
                *diffstyle = DiffStyle::Ed;
            } else if unsafe { cstr::starts_with(line.as_ptr(), b"@@ ") } {
                *diffstyle = DiffStyle::Unified;
            } else if unsafe { cstr::starts_with(line.as_ptr(), b"--- ") }
                && !unsafe { vim_fgets(line.as_mut_ptr(), LBUFLEN, fd) }
                && unsafe { cstr::starts_with(line.as_ptr(), b"+++ ") }
                && !unsafe { vim_fgets(line.as_mut_ptr(), LBUFLEN, fd) }
                && unsafe { cstr::starts_with(line.as_ptr(), b"@@ ") }
            {
                // A unified diff with its file header still attached.
                *diffstyle = DiffStyle::Unified;
            } else {
                continue;
            }
        }
        let parsed = if *diffstyle == DiffStyle::Ed {
            (line[0] as u8).is_ascii_digit()
                && unsafe { parse_diff_ed(line.as_ptr(), hunk) }.is_ok()
        } else {
            debug_assert_eq!(*diffstyle, DiffStyle::Unified);
            unsafe {
                cstr::starts_with(line.as_ptr(), b"@@ ")
                    && parse_diff_unified(line.as_ptr(), hunk).is_ok()
            }
        };
        if parsed {
            return false;
        }
    }
}

/// Merge one hunk into the block list.
///
/// The hunk names a range in `idx_orig` and the range it became in
/// `idx_new`.  Three things can happen: the hunk falls past the blocks the
/// walk has reached (they are copied forward and skipped), it overlaps one or
/// more existing blocks (they are widened to cover it and the extra ones
/// freed), or it touches none (a new block).
unsafe fn process_hunk(walk: &mut Walk, idx_orig: usize, idx_new: usize, hunk: *mut diffhunk_T) {
    // SAFETY: `curtab` is set from startup to exit.
    let tp = unsafe { TabPage::current() };
    let end_orig = unsafe { (*hunk).lnum_orig } + unsafe { (*hunk).count_orig };

    // Blocks entirely above the hunk: they keep whatever an earlier pass
    // gave them, but `idx_new` still has to be filled in.
    while !walk.dp.is_null()
        && unsafe { (*hunk).lnum_orig }
            > unsafe { (*walk.dp).df_lnum[idx_orig] } + unsafe { (*walk.dp).df_count[idx_orig] }
    {
        if walk.notset {
            unsafe { diff_copy_entry(walk.dprev, walk.dp, idx_orig, idx_new) };
        }
        walk.dprev = walk.dp;
        walk.dp = unsafe { (*walk.dp).df_next };
        walk.notset = true;
    }

    let dp = walk.dp;
    let overlaps = !dp.is_null()
        && unsafe { (*hunk).lnum_orig }
            <= unsafe { (*dp).df_lnum[idx_orig] } + unsafe { (*dp).df_count[idx_orig] }
        && end_orig >= unsafe { (*dp).df_lnum[idx_orig] };
    if !overlaps {
        // A change of its own.
        let dp = unsafe { diff_alloc_new(tp, walk.dprev, dp) };
        unsafe { (*dp).df_lnum[idx_orig] = (*hunk).lnum_orig };
        unsafe { (*dp).df_count[idx_orig] = (*hunk).count_orig };
        unsafe { (*dp).df_lnum[idx_new] = (*hunk).lnum_new };
        unsafe { (*dp).df_count[idx_new] = (*hunk).count_new };
        for i in idx_orig + 1..idx_new {
            if !tp.tp_diffbuf[i].is_null() {
                unsafe { diff_copy_entry(walk.dprev, dp, idx_orig, i) };
            }
        }
        walk.dp = dp;
        walk.notset = false;
        return;
    }

    // The last existing block this hunk reaches; everything from `dp` to
    // `dpl` becomes one block.
    let mut dpl = dp;
    while !unsafe { (*dpl).df_next }.is_null()
        && end_orig >= unsafe { (*(*dpl).df_next).df_lnum[idx_orig] }
    {
        dpl = unsafe { (*dpl).df_next };
    }

    let mut off = unsafe { (*dp).df_lnum[idx_orig] } - unsafe { (*hunk).lnum_orig };
    if off > 0 {
        // The hunk starts above the block: every buffer up to `idx_new`
        // grows upwards by the same amount.
        for i in idx_orig..idx_new {
            if !tp.tp_diffbuf[i].is_null() {
                unsafe { (*dp).df_lnum[i] -= off };
                unsafe { (*dp).df_count[i] += off };
            }
        }
        unsafe { (*dp).df_lnum[idx_new] = (*hunk).lnum_new };
        unsafe { (*dp).df_count[idx_new] = (*hunk).count_new };
    } else if walk.notset {
        // First hunk to touch this block: it starts `off` lines into it.
        unsafe { (*dp).df_lnum[idx_new] = (*hunk).lnum_new + off };
        unsafe { (*dp).df_count[idx_new] = (*hunk).count_new - off };
    } else {
        // A second hunk inside a block this pass already wrote: extend
        // `idx_new` by however much longer the new text is than the part
        // of the block the hunk covers.
        let orig_size_in_dp = unsafe { *hunk }.count_orig.min(
            unsafe { (*dp).df_lnum[idx_orig] } + unsafe { (*dp).df_count[idx_orig] }
                - unsafe { (*hunk).lnum_orig },
        );
        unsafe { (*dp).df_count[idx_new] += (*hunk).count_new - orig_size_in_dp };
        let past = unsafe { (*hunk).lnum_new } + unsafe { (*hunk).count_new }
            - (unsafe { (*dp).df_lnum[idx_new] } + unsafe { (*dp).df_count[idx_new] });
        if past > 0 {
            unsafe { (*dp).df_count[idx_new] += past };
        }
    }

    // How far the hunk reaches past the last block it touches.
    off = end_orig - (unsafe { (*dpl).df_lnum[idx_orig] } + unsafe { (*dpl).df_count[idx_orig] });
    if off < 0 {
        if walk.notset || dp != dpl {
            unsafe { (*dp).df_count[idx_new] += -off };
        }
        off = 0;
    }
    for i in idx_orig..idx_new {
        if !tp.tp_diffbuf[i].is_null() {
            unsafe {
                (*dp).df_count[i] = (*dpl).df_lnum[i] + (*dpl).df_count[i] - (*dp).df_lnum[i] + off
            };
        }
    }

    // Everything between `dp` and `dpl` is now covered by `dp`.
    let mut dn = unsafe { (*dp).df_next };
    unsafe { (*dp).df_next = (*dpl).df_next };
    while dn != unsafe { (*dp).df_next } {
        let next = unsafe { (*dn).df_next };
        unsafe { clear_diffblock(dn) };
        dn = next;
    }
    walk.dp = dp;
    walk.notset = false;
}

/// Read a whole diff's worth of hunks into the current tabpage's block list.
pub(crate) unsafe fn diff_read(idx_orig: c_int, idx_new: c_int, dio: *mut diffio_T) {
    let (idx_orig, idx_new) = (idx_orig as usize, idx_new as usize);
    let dout = unsafe { &raw mut (*dio).dio_diff };
    let internal = unsafe { (*dio).dio_internal } != 0;
    let mut fd = ::core::ptr::null_mut::<FILE>();
    if !internal {
        fd = unsafe { os_fopen((*dout).dout_fname, c"r".as_ptr()) };
        if fd.is_null() {
            emsg(gettext(c"E98: Cannot read diff output"));
            return;
        }
    }

    let mut walk = Walk {
        dp: unsafe { (*curtab.get()).tp_first_diff },
        dprev: ::core::ptr::null_mut(),
        notset: true,
    };
    let mut line_hunk_idx = 0;
    let mut diffstyle = DiffStyle::Unknown;
    loop {
        let mut hunk = diffhunk_T {
            lnum_orig: 0,
            count_orig: 0,
            lnum_new: 0,
            count_new: 0,
        };
        let eof = if internal {
            unsafe { extract_hunk_internal(dout, &raw mut hunk, &mut line_hunk_idx) }
        } else {
            unsafe { extract_hunk(fd, &raw mut hunk, &mut diffstyle) }
        };
        if eof {
            break;
        }
        unsafe { process_hunk(&mut walk, idx_orig, idx_new, &raw mut hunk) };
    }

    // Blocks below the last hunk still need `idx_new` filled in.
    while !walk.dp.is_null() {
        if walk.notset {
            unsafe { diff_copy_entry(walk.dprev, walk.dp, idx_orig, idx_new) };
        }
        walk.dprev = walk.dp;
        walk.dp = unsafe { (*walk.dp).df_next };
        walk.notset = true;
    }
    if !fd.is_null() {
        unsafe { fclose(fd) };
    }
}

/// `[f1[,l1]](a|c|d)f2[,l2]` -- one `ed`-style hunk header.
///
/// An `a` hunk adds after `f1`, so its original range is empty and starts on
/// the *next* line; a `d` hunk is the mirror image.
unsafe fn parse_diff_ed(line: *const c_char, hunk: *mut diffhunk_T) -> Result<(), Failed> {
    let mut p = line as *mut c_char;
    let f1 = unsafe { getdigits_int32(&raw mut p, true, 0) };
    let l1 = if unsafe { *p } == b',' as c_char {
        p = unsafe { p.offset(1) };
        unsafe { getdigits_int(&raw mut p, true, 0) }
    } else {
        f1
    };
    let difftype = unsafe { *p } as u8;
    if !matches!(difftype, b'a' | b'c' | b'd') {
        return Err(Failed);
    }
    p = unsafe { p.offset(1) };
    let f2 = unsafe { getdigits_int(&raw mut p, true, 0) };
    let l2 = if unsafe { *p } == b',' as c_char {
        p = unsafe { p.offset(1) };
        unsafe { getdigits_int(&raw mut p, true, 0) }
    } else {
        f2
    };
    if l1 < f1 || l2 < f2 {
        return Err(Failed);
    }
    unsafe {
        *hunk = diffhunk_T {
            lnum_orig: if difftype == b'a' { f1 + 1 } else { f1 },
            count_orig: if difftype == b'a' { 0 } else { l1 - f1 + 1 },
            lnum_new: if difftype == b'd' { f2 + 1 } else { f2 },
            count_new: if difftype == b'd' { 0 } else { l2 - f2 + 1 },
        }
    };
    Ok(())
}

/// `@@ -f1[,c1] +f2[,c2] @@` -- one unified hunk header.
///
/// An omitted count is 1, and a count of *zero* means the hunk adds or
/// deletes at that point rather than covering it, which shifts the line
/// number by one.
unsafe fn parse_diff_unified(line: *const c_char, hunk: *mut diffhunk_T) -> Result<(), Failed> {
    let mut p = line as *mut c_char;
    if !unsafe { cstr::starts_with(p, b"@@ -") } {
        return Err(Failed);
    }
    p = unsafe { p.add(4) };
    let mut oldline = unsafe { getdigits_int32(&raw mut p, true, 0) };
    let oldcount = if unsafe { *p } == b',' as c_char {
        p = unsafe { p.offset(1) };
        unsafe { getdigits_int(&raw mut p, true, 0) }
    } else {
        1
    };
    if !unsafe { cstr::starts_with(p, b" +") } {
        return Err(Failed);
    }
    p = unsafe { p.add(2) };
    let mut newline = unsafe { getdigits_int(&raw mut p, true, 0) };
    let newcount = if unsafe { *p } == b',' as c_char {
        p = unsafe { p.offset(1) };
        unsafe { getdigits_int(&raw mut p, true, 0) }
    } else {
        1
    };
    if oldcount == 0 {
        oldline += 1;
    }
    if newcount == 0 {
        newline += 1;
    }
    unsafe {
        *hunk = diffhunk_T {
            lnum_orig: oldline,
            count_orig: oldcount,
            // `@@ -1,2 +0,0 @@` deletes the whole file; line 0 is not a line.
            lnum_new: newline.max(1),
            count_new: newcount,
        }
    };
    Ok(())
}
