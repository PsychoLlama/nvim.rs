//! `xtypes.h` and `xdiff.h`'s flags: the model the whole engine works over.
//!
//! Upstream keeps a prepared file as five parallel heap blocks plus a bump
//! allocator (`chastore_t`) handing out `xrecord_t`s that are then threaded
//! onto two singly-linked lists. Only one of those lists is ever *read* —
//! `xdfile_t.rhash` and `xrecord_t.next` are filled by
//! [`super::xprepare::classify_record`] and never looked at again by anything
//! in the library — so this port keeps the records in a `Vec` and drops the
//! chaining, the arena and the hash side table with it.
//!
//! A record is a byte range rather than a pointer pair, which is what makes
//! [`XdFile::span`] (the sub-file `fall_back_diff` hands back to the classic
//! algorithm) a plain reslice instead of pointer arithmetic across two
//! records.
//!
//! Ported from LibXDiff by Davide Libenzi (File Differential Library),
//! Copyright (C) 2003 Davide Libenzi. LibXDiff is LGPL-2.1-or-later, and
//! this port stays under that license (text: licenses/LGPL-2.1.txt).

#![forbid(unsafe_code)]

/// `xpparam_t.flags`: ask for the minimal edit script rather than a
/// heuristic one.
pub const XDF_NEED_MINIMAL: u64 = 1 << 0;
/// Compare lines with every whitespace run elided (`-w`).
pub const XDF_IGNORE_WHITESPACE: u64 = 1 << 1;
/// Compare lines with interior whitespace runs collapsed (`-b`).
pub const XDF_IGNORE_WHITESPACE_CHANGE: u64 = 1 << 2;
/// Compare lines ignoring trailing whitespace.
pub const XDF_IGNORE_WHITESPACE_AT_EOL: u64 = 1 << 3;
/// Compare lines ignoring a CR immediately before the line's newline. The
/// only spelling of this flag in the tree is `vim.diff{ignore_cr_at_eol=}`;
/// `'diffopt'` has no name for it.
pub const XDF_IGNORE_CR_AT_EOL: u64 = 1 << 4;
/// The four flags above. Their presence is what switches
/// [`super::xutils::recmatch`] and [`super::xutils::hash_record`] out of
/// their fast byte-exact paths.
pub const XDF_WHITESPACE_FLAGS: u64 = XDF_IGNORE_WHITESPACE
    | XDF_IGNORE_WHITESPACE_CHANGE
    | XDF_IGNORE_WHITESPACE_AT_EOL
    | XDF_IGNORE_CR_AT_EOL;
/// Do not report a hunk whose every line is blank.
pub const XDF_IGNORE_BLANK_LINES: u64 = 1 << 7;
/// Use the patience algorithm ([`super::xpatience`]).
pub const XDF_PATIENCE_DIFF: u64 = 1 << 14;
/// Use the histogram algorithm ([`super::xhistogram`]).
pub const XDF_HISTOGRAM_DIFF: u64 = 1 << 15;
/// The bits [`Params::algorithm`] reads. Both clear means Myers.
pub const XDF_DIFF_ALGORITHM_MASK: u64 = XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF;
/// Slide a shiftable hunk to the position the indent heuristic scores best.
pub const XDF_INDENT_HEURISTIC: u64 = 1 << 23;

/// `xdemitconf_t.flags`: emit the hunk bodies with no `@@` header.
pub const XDL_EMIT_NO_HUNK_HDR: u64 = 1 << 1;

/// Which of the three engines [`super::xdiffi::do_diff`] dispatches to.
///
/// The mapping is upstream's `XDF_DIFF_ALG`, which reads the two bits as a
/// *value*, not as a pair of independent switches: patience wins when both
/// are set, because its bit is tested first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    /// The default: Myers' O(ND) walk, in [`super::xdiffi`].
    Myers,
    /// [`super::xpatience`].
    Patience,
    /// [`super::xhistogram`].
    Histogram,
}

/// `xpparam_t`, with the anchors already read off the C `char **`.
pub struct Params<'a> {
    /// The `XDF_*` bits above.
    pub flags: u64,
    /// Line prefixes that patience must keep as anchors. Always empty in
    /// nvim: `'diffanchors'` is implemented in `diff.rs` by splitting the
    /// buffers, and neither caller of [`super::ffi::xdl_diff`] fills
    /// `xpparam_t.anchors`.
    pub anchors: Vec<&'a [u8]>,
}

impl Params<'_> {
    /// `XDF_DIFF_ALG(flags)`.
    pub fn algorithm(&self) -> Algorithm {
        match self.flags & XDF_DIFF_ALGORITHM_MASK {
            XDF_PATIENCE_DIFF => Algorithm::Patience,
            XDF_HISTOGRAM_DIFF => Algorithm::Histogram,
            // Both bits set is `XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF`,
            // which upstream's `if (alg == XDF_PATIENCE_DIFF) ... else if
            // (alg == XDF_HISTOGRAM_DIFF)` chain answers *neither* of — it
            // falls through to Myers. Reproduced, not tidied.
            _ => Algorithm::Myers,
        }
    }

    /// The parameters a fall-back to the classic algorithm runs under: the
    /// caller's flags with the engine bits cleared, and no anchors.
    pub fn without_algorithm(&self) -> Params<'static> {
        Params {
            flags: self.flags & !XDF_DIFF_ALGORITHM_MASK,
            anchors: Vec::new(),
        }
    }
}

/// `xdemitconf_t`, less the two `#if 0`-ed function-name hooks.
#[derive(Clone, Copy)]
pub struct EmitConf {
    /// Unchanged lines to show either side of a hunk.
    pub ctxlen: i64,
    /// Unchanged lines two hunks may share before they are emitted as one.
    pub interhunkctxlen: i64,
    /// The `XDL_EMIT_*` bits.
    pub flags: u64,
}

/// A callback answered with a negative number, which is the only way any of
/// this fails: every allocation goes through `xmalloc`, which aborts.
#[derive(Clone, Copy, Debug)]
pub struct Aborted;

/// The result every emit path carries.
pub type XdResult<T = ()> = Result<T, Aborted>;

/// One line of a prepared file: where it starts in [`XdFile::text`], how long
/// it is (its newline included, when it has one) and its hash.
#[derive(Clone, Copy, Debug)]
pub struct Rec {
    /// Byte offset into the owning file's text.
    pub start: usize,
    /// Byte length, the trailing newline included.
    pub size: usize,
    /// Out of [`super::xutils::hash_record`], then replaced by
    /// [`super::xprepare::classify_record`] with a dense class id — small,
    /// starting at 0, and shared between the two files, which is what lets
    /// [`super::xpatience`] use it directly as a hash key. The histogram
    /// engine skips classification, so there `ha` stays the content hash.
    pub ha: u64,
}

/// `xdfile_t.rchg`, one flag per line, addressable over `-1 ..= nrec`.
///
/// Upstream allocates `nrec + 2` bytes and hands out a pointer one past the
/// start, so index `-1` and index `nrec` are permanent zero sentinels. Every
/// group walk in [`super::xdiffi::change_compact`] leans on them: they are
/// what makes `while (rchg[end]) end++` terminate without a bounds test.
#[derive(Default)]
pub struct Changed(Vec<u8>);

impl Changed {
    /// `nrec` lines, all unchanged, with the two sentinels.
    pub fn new(nrec: usize) -> Self {
        Self(vec![0; nrec + 2])
    }

    /// Is line `i` changed? Out-of-range answers `false`, which is what the
    /// sentinels give upstream — and `build_script`'s `i1 >= 0 || i2 >= 0`
    /// loop condition can in principle step one index further still.
    pub fn get(&self, i: i64) -> bool {
        // The `as usize` of a negative i + 1 wraps to a huge index, which
        // `get` answers None for; that is the "further still" case.
        self.0.get((i + 1) as usize).is_some_and(|&c| c != 0)
    }

    /// Mark line `i`. Panics outside `-1 ..= nrec`, which no caller reaches.
    pub fn set(&mut self, i: i64, changed: bool) {
        self.0[(i + 1) as usize] = u8::from(changed);
    }

    /// The `count` flags starting at line `from`, for
    /// [`super::xutils::fall_back_diff`]'s copy-back.
    pub fn slice(&self, from: i64, count: i64) -> &[u8] {
        let base = (from + 1) as usize;
        &self.0[base..base + count as usize]
    }

    /// Overwrite the `flags.len()` flags starting at line `from`.
    pub fn write(&mut self, from: i64, flags: &[u8]) {
        let base = (from + 1) as usize;
        self.0[base..base + flags.len()].copy_from_slice(flags);
    }
}

/// `xdfile_t`: one side of a diff, prepared.
pub struct XdFile<'a> {
    /// The whole input file. Every [`Rec`] indexes into this.
    pub text: &'a [u8],
    /// The file's lines, in order; `recs.len()` is upstream's `nrec`.
    pub recs: Vec<Rec>,
    /// First line the two files disagree about, per
    /// [`super::xprepare::trim_ends`]. 0 when the engine skips trimming.
    pub dstart: i64,
    /// Last such line; `nrec - 1` when the engine skips trimming.
    pub dend: i64,
    /// Which lines the engine decided are changed. The answer.
    pub rchg: Changed,
    /// For each of the `nreff` lines that survived
    /// [`super::xprepare::cleanup_records`], its index in [`Self::recs`].
    pub rindex: Vec<i64>,
    /// Those lines' hashes, so the Myers walk compares two flat arrays.
    pub ha: Vec<u64>,
    /// How many of [`Self::rindex`] and [`Self::ha`] are filled. Zero for
    /// patience and histogram, which do not reduce the problem this way.
    pub nreff: i64,
}

impl<'a> XdFile<'a> {
    /// Upstream's `nrec`.
    pub fn nrec(&self) -> i64 {
        self.recs.len() as i64
    }

    /// Line `i`'s bytes.
    pub fn line(&self, i: i64) -> &'a [u8] {
        let rec = self.recs[i as usize];
        &self.text[rec.start..rec.start + rec.size]
    }

    /// Line `i`'s hash or class id.
    pub fn ha_at(&self, i: i64) -> u64 {
        self.recs[i as usize].ha
    }

    /// Lines `first ..= last` as one slice. The records tile the text with
    /// no gaps and no overlap, so a run of them is contiguous — which is the
    /// whole reason [`super::xutils::fall_back_diff`] can build a sub-file
    /// without copying.
    pub fn span(&self, first: i64, last: i64) -> &'a [u8] {
        let a = self.recs[first as usize];
        let b = self.recs[last as usize];
        &self.text[a.start..b.start + b.size]
    }

    /// The three arrays [`super::xdiffi::recs_cmp`] works over, borrowed
    /// apart so the walk can write `rchg` while reading the other two.
    pub fn diff_data(&mut self) -> DiffData<'_> {
        DiffData {
            nrec: self.nreff,
            ha: &self.ha,
            rindex: &self.rindex,
            rchg: &mut self.rchg,
        }
    }
}

/// `diffdata_t`: the reduced view of an [`XdFile`] the Myers walk sees.
pub struct DiffData<'a> {
    /// How many entries of [`Self::ha`] and [`Self::rindex`] are live.
    pub nrec: i64,
    /// The surviving lines' hashes, densely packed.
    pub ha: &'a [u64],
    /// Where each of them sits in the full file.
    pub rindex: &'a [i64],
    /// The full file's changed flags, written through [`Self::rindex`].
    pub rchg: &'a mut Changed,
}

impl DiffData<'_> {
    /// Mark the reduced-index line `i` changed in the full file.
    pub fn mark(&mut self, i: i64) {
        self.rchg.set(self.rindex[i as usize], true);
    }
}

/// `xdfenv_t`: both prepared sides.
pub struct Env<'a> {
    /// The "before" file.
    pub xdf1: XdFile<'a>,
    /// The "after" file.
    pub xdf2: XdFile<'a>,
}

/// `xdchange_t`: one run of changed lines on each side.
///
/// Upstream builds these as a singly-linked list, prepending while it walks
/// the files backwards; the list therefore comes out in increasing `i1`
/// order, which is the order this `Vec` is in.
#[derive(Clone, Copy, Debug)]
pub struct Change {
    /// First changed line in file 1.
    pub i1: i64,
    /// First changed line in file 2.
    pub i2: i64,
    /// How many lines file 1 loses.
    pub chg1: i64,
    /// How many lines file 2 gains.
    pub chg2: i64,
    /// Every line of the change is blank and `XDF_IGNORE_BLANK_LINES` is on,
    /// so [`super::xemit::get_hunk`] may fold it away.
    pub ignore: bool,
}

/// A sub-problem: the same lines of both files, 1-based and inclusive of
/// `line1`/`line2`, as [`super::xpatience`] and [`super::xhistogram`] carry
/// it down their recursions.
#[derive(Clone, Copy)]
pub struct Block {
    /// First line of file 1 in the block.
    pub line1: i64,
    /// How many lines of file 1 it covers.
    pub count1: i64,
    /// First line of file 2.
    pub line2: i64,
    /// How many lines of file 2 it covers.
    pub count2: i64,
}

impl Block {
    /// `LINE_END(1)`: the last line of file 1 in the block.
    pub fn end1(&self) -> i64 {
        self.line1 + self.count1 - 1
    }

    /// `LINE_END(2)`.
    pub fn end2(&self) -> i64 {
        self.line2 + self.count2 - 1
    }
}
