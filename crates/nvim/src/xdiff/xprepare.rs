//! `xprepare.c`: split both files into lines, then reduce the problem.
//!
//! Three passes, in this order:
//!
//! 1. [`prepare_ctx`] splits a file into [`Rec`]s and hashes each line.
//! 2. [`Classifier`] replaces every hash with a dense *class id* shared
//!    between the two files, so the rest of the engine compares small
//!    integers and never re-reads a line. The histogram engine skips this —
//!    it wants the content hash — and so does nothing else.
//! 3. [`trim_ends`] and [`cleanup_records`] drop the matching head and tail
//!    and then every line with no counterpart at all, leaving `rindex`/`ha`:
//!    the reduced arrays the Myers walk actually runs over. Patience and
//!    histogram skip this too; they reduce the problem their own way.

#![forbid(unsafe_code)]

use crate::src::xdiff::xtypes::{Algorithm, Changed, Env, Params, Rec, XdFile};
use crate::src::xdiff::xutils::{bogosqrt, guess_lines, hash_record, hashbits, recmatch};

/// A run of similar lines shorter than this fraction is not worth keeping;
/// see the ratio at the end of [`clean_mmatch`].
const XDL_KPDIS_RUN: i64 = 4;
/// A line matching at least this many lines on the other side is "too
/// common" to be evidence of anything.
const XDL_MAX_EQLIMIT: i64 = 1024;
/// How far either side of a multi-match line [`clean_mmatch`] looks. Without
/// it the scan runs to the ends of the file on pathological input.
const XDL_SIMSCAN_WINDOW: i64 = 100;
/// Lines to sample when guessing a file's length.
const XDL_GUESS_NLINES1: i64 = 256;
/// The same, for the histogram engine: it never grows a hash table off the
/// guess, so a poorer estimate costs nothing.
const XDL_GUESS_NLINES2: i64 = 20;

/// One equivalence class of lines: every line in both files that matches
/// this one under the whitespace flags.
struct Class<'a> {
    /// The class's hash, as [`hash_record`] computed it.
    ha: u64,
    /// The first line that landed here, kept so [`recmatch`] has something
    /// to compare a candidate against.
    line: &'a [u8],
    /// How many lines of file 1 are in this class.
    len1: i64,
    /// How many lines of file 2 are.
    len2: i64,
}

/// `xdlclassifier_t`: hash-to-class-id, shared across both files.
struct Classifier<'a> {
    /// Width of [`Self::buckets`], in bits.
    hbits: u32,
    /// Class ids per hash bucket, oldest first. Upstream chains newest-first
    /// and takes the first match, so the search here walks in reverse — the
    /// order decides which class a line joins when two classes both match
    /// it, which the whitespace flavours make possible.
    buckets: Vec<Vec<u32>>,
    /// Every class, in the order they were first seen; the index *is* the
    /// class id, which is what makes the ids dense and comparable.
    classes: Vec<Class<'a>>,
    /// The `XDF_*` flags [`recmatch`] runs under.
    flags: u64,
}

impl<'a> Classifier<'a> {
    fn new(size: i64, flags: u64) -> Self {
        let hbits = hashbits(size as u32);
        Self {
            hbits,
            buckets: vec![Vec::new(); 1usize << hbits],
            classes: Vec::with_capacity(size.max(0) as usize),
            flags,
        }
    }

    /// Give `rec` its class id, counting it against the class's per-file
    /// tally. `pass` is 1 for the first file and 2 for the second.
    fn classify(&mut self, pass: u32, line: &'a [u8], rec: &mut Rec) {
        let bucket = hashlong(rec.ha, self.hbits);
        let found = self.buckets[bucket].iter().rev().copied().find(|&id| {
            let class = &self.classes[id as usize];
            class.ha == rec.ha && recmatch(class.line, line, self.flags)
        });

        let id = match found {
            Some(id) => id,
            None => {
                let id = self.classes.len() as u32;
                self.classes.push(Class {
                    ha: rec.ha,
                    line,
                    len1: 0,
                    len2: 0,
                });
                self.buckets[bucket].push(id);
                id
            }
        };

        let class = &mut self.classes[id as usize];
        if pass == 1 {
            class.len1 += 1;
        } else {
            class.len2 += 1;
        }
        rec.ha = u64::from(id);
    }
}

/// `XDL_HASHLONG`: fold the high bits down and mask to `bits`.
fn hashlong(v: u64, bits: u32) -> usize {
    (v.wrapping_add(v >> bits) & ((1u64 << bits) - 1)) as usize
}

/// Split `text` into lines, hash each, and classify it unless the caller is
/// the histogram engine.
fn prepare_ctx<'a>(
    pass: u32,
    text: &'a [u8],
    narec: i64,
    xpp: &Params<'_>,
    cf: Option<&mut Classifier<'a>>,
) -> XdFile<'a> {
    let mut recs: Vec<Rec> = Vec::with_capacity(narec.max(0) as usize);
    let mut cf = cf;
    let mut cur = 0usize;

    while cur < text.len() {
        let (ha, used) = hash_record(&text[cur..], xpp.flags);
        let mut rec = Rec {
            start: cur,
            size: used,
            ha,
        };
        if let Some(cf) = cf.as_deref_mut() {
            cf.classify(pass, &text[cur..cur + used], &mut rec);
        }
        recs.push(rec);
        cur += used;
    }

    let nrec = recs.len();
    XdFile {
        text,
        recs,
        dstart: 0,
        dend: nrec as i64 - 1,
        rchg: Changed::new(nrec),
        rindex: vec![0; nrec + 1],
        ha: vec![0; nrec + 1],
        nreff: 0,
    }
}

/// Prepare both files: split, classify, and (for the classic algorithm)
/// reduce.
pub fn prepare_env<'a>(text1: &'a [u8], text2: &'a [u8], xpp: &Params<'_>) -> Env<'a> {
    let algorithm = xpp.algorithm();

    // For histogram diff we can afford a smaller sample and thus a poorer
    // estimate of the number of lines: its hash table is sized from the
    // real count, not from the guess.
    let sample = if algorithm == Algorithm::Histogram {
        XDL_GUESS_NLINES2
    } else {
        XDL_GUESS_NLINES1
    };
    let enl1 = guess_lines(text1, sample) + 1;
    let enl2 = guess_lines(text2, sample) + 1;

    let mut cf =
        (algorithm != Algorithm::Histogram).then(|| Classifier::new(enl1 + enl2 + 1, xpp.flags));

    let mut xdf1 = prepare_ctx(1, text1, enl1, xpp, cf.as_mut());
    let mut xdf2 = prepare_ctx(2, text2, enl2, xpp, cf.as_mut());

    if let Some(cf) = &cf
        && algorithm == Algorithm::Myers
    {
        trim_ends(&mut xdf1, &mut xdf2);
        cleanup_records(cf, &mut xdf1, &mut xdf2);
    }

    Env { xdf1, xdf2 }
}

/// Should the multi-match line `i` be discarded?
///
/// Only when it sits in the middle of a run of lines that have no match at
/// all: a multi-match line surrounded by other multi-match lines is evidence
/// of a repeated block, which is worth keeping.
fn clean_mmatch(dis: &[i8], i: i64, mut s: i64, mut e: i64) -> bool {
    // Bound the window. The loops below stop at the first line that *does*
    // have a unique match, but on data that has none they would otherwise
    // walk to the ends of the file.
    if i - s > XDL_SIMSCAN_WINDOW {
        s = i - XDL_SIMSCAN_WINDOW;
    }
    if e - i > XDL_SIMSCAN_WINDOW {
        e = i + XDL_SIMSCAN_WINDOW;
    }

    // Scan back from `i` over lines that have no match (0) or many (2).
    let (mut rdis0, mut rpdis0) = (0i64, 1i64);
    let mut r = 1;
    while i - r >= s {
        match dis[(i - r) as usize] {
            0 => rdis0 += 1,
            2 => rpdis0 += 1,
            _ => break,
        }
        r += 1;
    }
    // A run of nothing but multi-match lines: keep `i`.
    if rdis0 == 0 {
        return false;
    }

    let (mut rdis1, mut rpdis1) = (0i64, 1i64);
    let mut r = 1;
    while i + r <= e {
        match dis[(i + r) as usize] {
            0 => rdis1 += 1,
            2 => rpdis1 += 1,
            _ => break,
        }
        r += 1;
    }
    if rdis1 == 0 {
        return false;
    }

    rdis1 += rdis0;
    rpdis1 += rpdis0;
    rpdis1 * XDL_KPDIS_RUN < rpdis1 + rdis1
}

/// Discard records that have no match on the other side, and the
/// multi-match ones [`clean_mmatch`] rejects; what is left is `rindex`/`ha`.
fn cleanup_records(cf: &Classifier<'_>, xdf1: &mut XdFile<'_>, xdf2: &mut XdFile<'_>) {
    let dis1 = match_counts(cf, xdf1, |class| class.len2);
    let dis2 = match_counts(cf, xdf2, |class| class.len1);
    keep_matched(xdf1, &dis1);
    keep_matched(xdf2, &dis2);
}

/// Per line of `xdf`, how interesting it is: 0 for no match on the other
/// side, 1 for a usable number, 2 for too many to mean anything.
fn match_counts(
    cf: &Classifier<'_>,
    xdf: &XdFile<'_>,
    other_side: impl Fn(&Class<'_>) -> i64,
) -> Vec<i8> {
    let mlim = bogosqrt(xdf.nrec()).min(XDL_MAX_EQLIMIT);
    let mut dis = vec![0i8; xdf.recs.len() + 1];
    for i in xdf.dstart..=xdf.dend {
        let nm = other_side(&cf.classes[xdf.ha_at(i) as usize]);
        dis[i as usize] = if nm == 0 {
            0
        } else if nm >= mlim {
            2
        } else {
            1
        };
    }
    dis
}

/// Fill `rindex`/`ha` with the lines worth diffing, and mark the rest
/// changed outright — they cannot match anything, so no walk will pair them.
fn keep_matched(xdf: &mut XdFile<'_>, dis: &[i8]) {
    let mut nreff = 0i64;
    for i in xdf.dstart..=xdf.dend {
        let d = dis[i as usize];
        if d == 1 || (d == 2 && !clean_mmatch(dis, i, xdf.dstart, xdf.dend)) {
            xdf.rindex[nreff as usize] = i;
            xdf.ha[nreff as usize] = xdf.ha_at(i);
            nreff += 1;
        } else {
            xdf.rchg.set(i, true);
        }
    }
    xdf.nreff = nreff;
}

/// Early-trim the matching head and tail, narrowing `dstart`/`dend`.
fn trim_ends(xdf1: &mut XdFile<'_>, xdf2: &mut XdFile<'_>) {
    let (nrec1, nrec2) = (xdf1.nrec(), xdf2.nrec());
    let mut lim = nrec1.min(nrec2);

    let mut i = 0i64;
    while i < lim && xdf1.ha_at(i) == xdf2.ha_at(i) {
        i += 1;
    }
    xdf1.dstart = i;
    xdf2.dstart = i;

    lim -= i;
    let mut i = 0i64;
    while i < lim && xdf1.ha_at(nrec1 - 1 - i) == xdf2.ha_at(nrec2 - 1 - i) {
        i += 1;
    }
    xdf1.dend = nrec1 - i - 1;
    xdf2.dend = nrec2 - i - 1;
}
