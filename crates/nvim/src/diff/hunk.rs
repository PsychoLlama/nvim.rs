//! Turning diff output back into blocks.
//!
//! [`diff_read`] walks whatever the engine produced -- unified or `ed`-style
//! text from an external diff, or the hunks `xdiff_out` collected from the
//! internal one -- and [`process_hunk`] merges each into the tabpage's block
//! list, growing, splitting or joining the existing blocks as the new range
//! overlaps them.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::charset::getdigits_int_at;
use crate::cstr;
use crate::fileio::Fgets;
use crate::winlayer::TabPage;
use core::ffi::c_int;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

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
    dp: *mut DiffBlock,
    dprev: *mut DiffBlock,
    notset: bool,
}

/// The next line of the external diff's output, as the C string
/// `vim_fgets` would have left in a `LBUFLEN` buffer: capped, and cut at a
/// NUL.  `None` at the end of the file.
fn next_line<'a>(fd: &mut Fgets, buf: &'a mut Vec<u8>) -> Option<&'a [u8]> {
    fd.line_capped(buf, LBUFLEN as usize)
        .then(|| cstr::until_nul(buf))
}

/// Read lines from `fd` until one parses as a hunk header.  `None` at the
/// end of input.
fn extract_hunk(fd: &mut Fgets, diffstyle: &mut DiffStyle) -> Option<DiffHunk> {
    let mut buf = Vec::new();
    loop {
        let line = next_line(fd, &mut buf)?;
        if *diffstyle == DiffStyle::Unknown {
            if line.first().is_some_and(u8::is_ascii_digit) {
                *diffstyle = DiffStyle::Ed;
            } else if line.starts_with(b"@@ ") {
                *diffstyle = DiffStyle::Unified;
            } else if line.starts_with(b"--- ")
                && next_line(fd, &mut buf).is_some_and(|line| line.starts_with(b"+++ "))
                && next_line(fd, &mut buf).is_some_and(|line| line.starts_with(b"@@ "))
            {
                // A unified diff with its file header still attached.
                *diffstyle = DiffStyle::Unified;
            } else {
                continue;
            }
        }
        let line = cstr::until_nul(&buf);
        let parsed = if *diffstyle == DiffStyle::Ed {
            line.first()
                .is_some_and(u8::is_ascii_digit)
                .then(|| parse_diff_ed(line))
                .flatten()
        } else {
            debug_assert_eq!(*diffstyle, DiffStyle::Unified);
            parse_diff_unified(line)
        };
        if parsed.is_some() {
            return parsed;
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
///
/// # Safety
///
/// `walk`'s blocks must be live blocks of the current tab page's list.
unsafe fn process_hunk(walk: &mut Walk, idx_orig: usize, idx_new: usize, hunk: &DiffHunk) {
    let tp = TabPage::current();
    let end_orig = hunk.lnum_orig + hunk.count_orig;

    // Blocks entirely above the hunk: they keep whatever an earlier pass
    // gave them, but `idx_new` still has to be filled in.
    while !walk.dp.is_null()
        && hunk.lnum_orig
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
        && hunk.lnum_orig
            <= unsafe { (*dp).df_lnum[idx_orig] } + unsafe { (*dp).df_count[idx_orig] }
        && end_orig >= unsafe { (*dp).df_lnum[idx_orig] };
    if !overlaps {
        // A change of its own.
        let dp = unsafe { diff_alloc_new(tp, walk.dprev, dp) };
        unsafe { (*dp).df_lnum[idx_orig] = hunk.lnum_orig };
        unsafe { (*dp).df_count[idx_orig] = hunk.count_orig };
        unsafe { (*dp).df_lnum[idx_new] = hunk.lnum_new };
        unsafe { (*dp).df_count[idx_new] = hunk.count_new };
        for i in idx_orig + 1..idx_new {
            if tp.tp_diffbuf[i].is_some() {
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

    let mut off = unsafe { (*dp).df_lnum[idx_orig] } - hunk.lnum_orig;
    if off > 0 {
        // The hunk starts above the block: every buffer up to `idx_new`
        // grows upwards by the same amount.
        for i in idx_orig..idx_new {
            if tp.tp_diffbuf[i].is_some() {
                unsafe { (*dp).df_lnum[i] -= off };
                unsafe { (*dp).df_count[i] += off };
            }
        }
        unsafe { (*dp).df_lnum[idx_new] = hunk.lnum_new };
        unsafe { (*dp).df_count[idx_new] = hunk.count_new };
    } else if walk.notset {
        // First hunk to touch this block: it starts `off` lines into it.
        unsafe { (*dp).df_lnum[idx_new] = hunk.lnum_new + off };
        unsafe { (*dp).df_count[idx_new] = hunk.count_new - off };
    } else {
        // A second hunk inside a block this pass already wrote: extend
        // `idx_new` by however much longer the new text is than the part
        // of the block the hunk covers.
        let orig_size_in_dp = hunk.count_orig.min(
            unsafe { (*dp).df_lnum[idx_orig] } + unsafe { (*dp).df_count[idx_orig] }
                - hunk.lnum_orig,
        );
        unsafe { (*dp).df_count[idx_new] += hunk.count_new - orig_size_in_dp };
        let past = hunk.lnum_new + hunk.count_new
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
        if tp.tp_diffbuf[i].is_some() {
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
pub(crate) fn diff_read(idx_orig: c_int, idx_new: c_int, dio: &DiffIo) {
    let (idx_orig, idx_new) = (idx_orig as usize, idx_new as usize);
    let mut fd = None;
    if !dio.internal {
        let file = dio
            .diff
            .fname
            .as_ref()
            .and_then(|name| File::open(OsStr::from_bytes(name.as_cstr().to_bytes())).ok());
        let Some(file) = file else {
            emsg(gettext(c"E98: Cannot read diff output"));
            return;
        };
        fd = Some(Fgets::new(file));
    }

    let mut walk = Walk {
        dp: TabPage::current().tp_first_diff,
        dprev: ::core::ptr::null_mut(),
        notset: true,
    };
    let mut internal_hunks = dio.diff.hunks.iter();
    let mut diffstyle = DiffStyle::Unknown;
    loop {
        let next = match &mut fd {
            None => internal_hunks.next().copied(),
            Some(fd) => extract_hunk(fd, &mut diffstyle),
        };
        let Some(hunk) = next else {
            break;
        };
        unsafe { process_hunk(&mut walk, idx_orig, idx_new, &hunk) };
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
}

/// `[f1[,l1]](a|c|d)f2[,l2]` -- one `ed`-style hunk header.
///
/// An `a` hunk adds after `f1`, so its original range is empty and starts on
/// the *next* line; a `d` hunk is the mirror image.
fn parse_diff_ed(line: &[u8]) -> Option<DiffHunk> {
    let (f1, at) = getdigits_int_at(line, 0, true, 0);
    let (l1, at) = number_after_comma(line, at, f1);
    let difftype = *line.get(at).filter(|c| matches!(c, b'a' | b'c' | b'd'))?;
    let (f2, at) = getdigits_int_at(line, at + 1, true, 0);
    let (l2, _) = number_after_comma(line, at, f2);
    if l1 < f1 || l2 < f2 {
        return None;
    }
    Some(DiffHunk {
        lnum_orig: if difftype == b'a' { f1 + 1 } else { f1 },
        count_orig: if difftype == b'a' { 0 } else { l1 - f1 + 1 },
        lnum_new: if difftype == b'd' { f2 + 1 } else { f2 },
        count_new: if difftype == b'd' { 0 } else { l2 - f2 + 1 },
    })
}

/// `,N` at `at`: `N` and the offset past it, else `default` and `at`.
fn number_after_comma(line: &[u8], at: usize, default: c_int) -> (c_int, usize) {
    if line.get(at) == Some(&b',') {
        getdigits_int_at(line, at + 1, true, 0)
    } else {
        (default, at)
    }
}

/// `@@ -f1[,c1] +f2[,c2] @@` -- one unified hunk header.
///
/// An omitted count is 1, and a count of *zero* means the hunk adds or
/// deletes at that point rather than covering it, which shifts the line
/// number by one.
fn parse_diff_unified(line: &[u8]) -> Option<DiffHunk> {
    let rest = line.strip_prefix(b"@@ -")?;
    let (mut oldline, at) = getdigits_int_at(rest, 0, true, 0);
    let (oldcount, at) = number_after_comma(rest, at, 1);
    let rest = rest[at..].strip_prefix(b" +")?;
    let (mut newline, at) = getdigits_int_at(rest, 0, true, 0);
    let (newcount, _) = number_after_comma(rest, at, 1);
    if oldcount == 0 {
        oldline += 1;
    }
    if newcount == 0 {
        newline += 1;
    }
    Some(DiffHunk {
        lnum_orig: oldline,
        count_orig: oldcount,
        // `@@ -1,2 +0,0 @@` deletes the whole file; line 0 is not a line.
        lnum_new: newline.max(1),
        count_new: newcount,
    })
}
