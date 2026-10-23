//! Turning diff output back into blocks.
//!
//! [`diff_read`] walks whatever the engine produced -- unified or `ed`-style
//! text from an external diff, or the hunks the internal one collected -- and
//! [`process_hunk`] merges each into a block list, growing, splitting or
//! joining the existing blocks as the new range overlaps them.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::getdigits_int_at;
use crate::cstr;
use crate::fileio::Fgets;
use core::ffi::c_int;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

/// Which format an external diff answered in.
///
/// Not known until the first line that says so, which is why it is threaded
/// through [`extract_hunk`] rather than decided once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DiffStyle {
    /// Not yet decided.
    Unknown,
    /// `2,4c2,4`.
    Ed,
    /// `@@ -2,3 +2,3 @@`.
    Unified,
}

/// Where the walk over the block list has got to.
///
/// `at` is the block the walk is on (the list's length past the end); the
/// one before it is always the block it stepped past last. `notset` says
/// the block at `at` has *not* yet had `idx_new`'s range written into it,
/// which is what tells the merge whether it is looking at a block from an
/// earlier pass or one it just made.
struct Walk {
    at: usize,
    notset: bool,
}

impl Walk {
    /// Step past the block at `at`, filling in its `idx_new` range first if
    /// no hunk did.
    fn step(&mut self, blocks: &mut [DiffBlock], idx_orig: usize, idx_new: usize) {
        if self.notset {
            diff_copy_entry(blocks, self.at, idx_orig, idx_new);
        }
        self.at += 1;
        self.notset = true;
    }
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
/// dropped), or it touches none (a new block).
fn process_hunk(
    blocks: &mut Vec<DiffBlock>,
    slots: &DiffSlots,
    walk: &mut Walk,
    idx_orig: usize,
    idx_new: usize,
    hunk: &DiffHunk,
) {
    let end_orig = hunk.lnum_orig + hunk.count_orig;

    // Blocks entirely above the hunk: they keep whatever an earlier pass
    // gave them, but `idx_new` still has to be filled in.
    while walk.at < blocks.len() && hunk.lnum_orig > blocks[walk.at].end(idx_orig) {
        walk.step(blocks, idx_orig, idx_new);
    }

    let at = walk.at;
    let overlaps = blocks
        .get(at)
        .is_some_and(|dp| hunk.lnum_orig <= dp.end(idx_orig) && end_orig >= dp.df_lnum[idx_orig]);
    if !overlaps {
        // A change of its own.
        let mut dp = DiffBlock::default();
        dp.df_lnum[idx_orig] = hunk.lnum_orig;
        dp.df_count[idx_orig] = hunk.count_orig;
        dp.df_lnum[idx_new] = hunk.lnum_new;
        dp.df_count[idx_new] = hunk.count_new;
        blocks.insert(at, dp);
        for i in used_slots(slots).filter(|i| (idx_orig + 1..idx_new).contains(i)) {
            diff_copy_entry(blocks, at, idx_orig, i);
        }
        walk.notset = false;
        return;
    }

    // The last existing block this hunk reaches; everything from `at` to
    // `last` becomes one block.
    let mut last = at;
    while blocks
        .get(last + 1)
        .is_some_and(|next| end_orig >= next.df_lnum[idx_orig])
    {
        last += 1;
    }

    let dp = &mut blocks[at];
    let mut off = dp.df_lnum[idx_orig] - hunk.lnum_orig;
    if off > 0 {
        // The hunk starts above the block: every buffer up to `idx_new`
        // grows upwards by the same amount.
        for i in used_slots(slots).filter(|i| (idx_orig..idx_new).contains(i)) {
            dp.df_lnum[i] -= off;
            dp.df_count[i] += off;
        }
        dp.df_lnum[idx_new] = hunk.lnum_new;
        dp.df_count[idx_new] = hunk.count_new;
    } else if walk.notset {
        // First hunk to touch this block: it starts `off` lines into it.
        dp.df_lnum[idx_new] = hunk.lnum_new + off;
        dp.df_count[idx_new] = hunk.count_new - off;
    } else {
        // A second hunk inside a block this pass already wrote: extend
        // `idx_new` by however much longer the new text is than the part
        // of the block the hunk covers.
        let orig_size_in_dp = hunk.count_orig.min(dp.end(idx_orig) - hunk.lnum_orig);
        dp.df_count[idx_new] += hunk.count_new - orig_size_in_dp;
        let past = hunk.lnum_new + hunk.count_new - dp.end(idx_new);
        if past > 0 {
            dp.df_count[idx_new] += past;
        }
    }

    // How far the hunk reaches past the last block it touches -- which may
    // be the one just widened, read as it now stands.
    let (last_lnum, last_count) = (blocks[last].df_lnum, blocks[last].df_count);
    let dp = &mut blocks[at];
    off = end_orig - (last_lnum[idx_orig] + last_count[idx_orig]);
    if off < 0 {
        if walk.notset || at != last {
            dp.df_count[idx_new] += -off;
        }
        off = 0;
    }
    for i in idx_orig..idx_new {
        if slots[i].is_some() {
            dp.df_count[i] = last_lnum[i] + last_count[i] - dp.df_lnum[i] + off;
        }
    }

    // Everything after `at` up to `last` is now covered by it.
    blocks.drain(at + 1..=last);
    walk.notset = false;
}

/// Where a run of hunks comes from: the internal engine's answer, or an
/// external diff's output file.
pub(crate) enum HunkSource<'a> {
    Internal(core::slice::Iter<'a, DiffHunk>),
    External(Fgets, DiffStyle),
}

impl<'a> HunkSource<'a> {
    /// The hunks of `dio`'s run, `None` (and E98) when the external diff's
    /// output cannot be opened.
    pub(crate) fn open(dio: &'a DiffIo) -> Option<Self> {
        if dio.internal {
            return Some(Self::Internal(dio.diff.hunks.iter()));
        }
        let file = dio
            .diff
            .fname
            .as_ref()
            .and_then(|name| File::open(OsStr::from_bytes(name.as_cstr().to_bytes())).ok());
        let Some(file) = file else {
            emsg(gettext(c"E98: Cannot read diff output"));
            return None;
        };
        Some(Self::External(Fgets::new(file), DiffStyle::Unknown))
    }
}

impl Iterator for HunkSource<'_> {
    type Item = DiffHunk;

    fn next(&mut self) -> Option<DiffHunk> {
        match self {
            Self::Internal(hunks) => hunks.next().copied(),
            Self::External(fd, diffstyle) => extract_hunk(fd, diffstyle),
        }
    }
}

/// Read a whole diff's worth of hunks into `blocks`.
///
/// `slots` says which buffers are in the diff. Both are the caller's, so the
/// inline diff can run this over a scratch list of its own.
pub(crate) fn diff_read(
    blocks: &mut Vec<DiffBlock>,
    slots: &DiffSlots,
    idx_orig: c_int,
    idx_new: c_int,
    hunks: HunkSource<'_>,
) {
    let (idx_orig, idx_new) = (idx_orig as usize, idx_new as usize);
    let mut walk = Walk {
        at: 0,
        notset: true,
    };
    for hunk in hunks {
        process_hunk(blocks, slots, &mut walk, idx_orig, idx_new, &hunk);
    }

    // Blocks below the last hunk still need `idx_new` filled in.
    while walk.at < blocks.len() {
        walk.step(blocks, idx_orig, idx_new);
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
