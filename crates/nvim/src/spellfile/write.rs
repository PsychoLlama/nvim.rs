//! Writing a `.spl` file.
//!
//! The format is a short header — magic, version — followed by a run of
//! optional sections and then the three word trees. Each section is an id
//! byte, a flags byte, a four-byte length, and that many bytes of payload;
//! a reader that does not know an id can skip it by its length, unless the
//! flags say [`SNF_REQUIRED`], in which case it must refuse the file.
//! [`SN_END`] closes the list.
//!
//! [`write_vim_spell`] emits the sections in id order and then calls
//! [`put_node`] three times, for the case-folded, keep-case and prefix
//! trees.
//!
//! # Two passes per tree
//!
//! Nodes are referred to by their byte offset in the written tree, so a
//! node's index is not known until everything before it has been sized.
//! [`put_node`] therefore runs twice: once with a null file to count, then
//! again to write. Sections with a computed length do the same —
//! [`write_spell_prefcond`] measures on the first call and writes on the
//! second.
//!
//! # The write-failure flag
//!
//! The C accumulated success in a `size_t` it bitwise-ANDed with every
//! `fwrite` return, and only checked at the very end. [`SplWriter`] keeps
//! that as a boolean, including its quirk: `fwrite` of *zero* items reports
//! zero, so a zero-length payload marks the whole file failed. The callers
//! that could hit that guard against it themselves, and the ones that do
//! not are the ones the C left exposed too.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::semsg;
use crate::spell::WordFlags;
use core::ffi::{c_char, c_int};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::main::e_write;
use crate::mbyte::encode_char;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::spell::{spelltab_fold, spelltab_isu, spelltab_isw};
use crate::types::{Failed, NUL, time_t};
use ::libc::time;

use super::wordtree::wordnode_T;
use super::{
    BY_FLAGS, BY_FLAGS2, BY_INDEX, BY_NOFLAGS, CF_UPPER, CF_WORD, PFX_FLAGS, SAL_COLLAPSE,
    SAL_F0LLOWUP, SAL_REM_ACCENTS, SN_CHARFLAGS, SN_COMPOUND, SN_END, SN_INFO, SN_MAP, SN_MIDWORD,
    SN_NOBREAK, SN_NOCOMPOUNDSUGS, SN_NOSPLITSUGS, SN_PREFCOND, SN_REGION, SN_REP, SN_REPSAL,
    SN_SAL, SN_SOFO, SN_SUGFILE, SN_SYLLABLE, SN_WORDS, SNF_REQUIRED, VIMSPELLMAGIC,
    VIMSPELLVERSION, spellinfo_T,
};

/// A `.spl` or `.sug` file being written.
pub(super) struct SplWriter {
    out: BufWriter<File>,
    /// Cleared by the first write that fails. Checked once, when the file
    /// is closed.
    ok: bool,
}

impl SplWriter {
    pub(super) fn new(file: File) -> Self {
        Self {
            out: BufWriter::new(file),
            ok: true,
        }
    }

    /// Has every write so far landed?
    pub(super) fn landed(&self) -> bool {
        self.ok
    }

    /// Flush what is left and say whether every write landed.
    pub(super) fn finish(&mut self) -> bool {
        self.ok &= self.out.flush().is_ok();
        self.ok
    }

    pub(super) fn byte(&mut self, c: c_int) {
        self.bytes(&[c as u8]);
    }

    pub(super) fn u32(&mut self, v: usize) {
        self.bytes(&(v as u32).to_be_bytes());
    }

    fn u16(&mut self, v: usize) {
        self.bytes(&(v as u16).to_be_bytes());
    }

    /// Write a payload.
    pub(super) fn bytes(&mut self, b: &[u8]) {
        self.ok &= self.out.write_all(b).is_ok();
    }

    /// Open a section: its id, its flags, and the payload length that
    /// follows.
    fn section(&mut self, id: c_int, flags: c_int, len: usize) {
        self.byte(id);
        self.byte(flags);
        self.u32(len);
    }
}

/// Write the whole `.spl` file.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path and `spin` must hold finished,
/// compressed trees.
pub(super) unsafe fn write_vim_spell(
    spin: &mut spellinfo_T,
    fname: *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller promises the path.
    let path = Path::new(OsStr::from_bytes(unsafe { cstr::bytes_at(fname) }));
    let Ok(file) = File::create(path) else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E484: Can't open file {fname}");
        return Err(Failed);
    };
    let mut w = SplWriter::new(file);
    let mut retval = Ok(());

    w.bytes(VIMSPELLMAGIC.to_bytes());
    // A failed magic write means the file is unusable; the C skipped
    // straight to the close and so does this.
    if w.ok {
        w.byte(VIMSPELLVERSION);

        // SAFETY: every string read below is one of `spin`'s own
        // NUL-terminated arena strings, and the trees are its own.
        unsafe {
            put_info(&mut w, spin);
            let regionmask = put_region(&mut w, spin);
            put_charflags(&mut w, spin);
            put_midword(&mut w, spin);
            put_prefcond(&mut w, spin);
            put_rep_and_sal(&mut w, spin);
            put_sofo(&mut w, spin);
            put_words(&mut w, spin);
            put_map(&mut w, spin);
            put_sugfile(&mut w, spin);
            put_flag_sections(&mut w, spin);
            put_compound(&mut w, spin);
            put_syllable(&mut w, spin);

            w.byte(SN_END as c_int);
            put_trees(&mut w, spin, regionmask);
        }

        // The trailing byte the reader uses to tell a complete file
        // from a truncated one.
        w.byte(0);
    }

    if !w.finish() {
        retval = Err(Failed);
    }
    if retval.is_err() {
        emsg(gettext(e_write));
    }
    retval
}

/// `SN_INFO`: the free-form text `:spellinfo` shows.
unsafe fn put_info(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_info.is_null() {
        return;
    }
    // SAFETY: `si_info` is a NUL-terminated arena string.
    let text = unsafe { cstr::bytes_at(spin.si_info) };
    w.section(SN_INFO as c_int, 0, text.len());
    w.bytes(text);
}

/// `SN_REGION`: the two-letter region names. Returns the mask of all
/// regions, which the tree writer uses to spot words that are in every one.
fn put_region(w: &mut SplWriter, spin: &spellinfo_T) -> c_int {
    if spin.si_region_count <= 1 {
        return 0;
    }
    // `si_region_name` holds `si_region_count` two-byte names.
    let len = spin.si_region_count as usize * 2;
    w.section(SN_REGION as c_int, SNF_REQUIRED, len);
    let names: &[u8; 17] = &spin.si_region_name.map(i8::cast_unsigned);
    w.bytes(&names[..len]);
    (1 << spin.si_region_count) - 1
}

/// `SN_CHARFLAGS`: which of the bytes 128..255 are word characters and
/// which are upper case, plus the folded form of each.
///
/// Only meaningful for a non-ASCII base dictionary; an `.add` file inherits
/// the table from the file it extends.
fn put_charflags(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_ascii != 0 || spin.si_add != 0 {
        return;
    }
    let mut folchars: Vec<u8> = Vec::with_capacity(512);
    let mut buf = [0u8; 8];
    for i in 128..256 {
        let n = encode_char(spelltab_fold(i) as c_int, &mut buf);
        folchars.extend_from_slice(&buf[..n]);
    }

    w.section(
        SN_CHARFLAGS as c_int,
        SNF_REQUIRED,
        1 + 128 + 2 + folchars.len(),
    );
    w.byte(128);
    let flags: Vec<u8> = (128..256)
        .map(|i| {
            let mut f = 0u8;
            if spelltab_isw(i) {
                f |= CF_WORD as u8;
            }
            if spelltab_isu(i) {
                f |= CF_UPPER as u8;
            }
            f
        })
        .collect();
    w.bytes(&flags);
    w.u16(folchars.len());
    w.bytes(&folchars);
}

/// `SN_MIDWORD`: characters that may appear inside a word without ending
/// it, such as an apostrophe.
unsafe fn put_midword(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_midword.is_null() {
        return;
    }
    // SAFETY: `si_midword` is a NUL-terminated arena string.
    let text = unsafe { cstr::bytes_at(spin.si_midword) };
    w.section(SN_MIDWORD as c_int, SNF_REQUIRED, text.len());
    w.bytes(text);
}

/// `SN_PREFCOND`: the regexps a prefix's condition compiles from, one per
/// prefix id. Measured with a null file first, since the length has to
/// precede the payload.
fn put_prefcond(w: &mut SplWriter, spin: &mut spellinfo_T) {
    if spin.si_prefcond.is_empty() {
        return;
    }
    let len = write_spell_prefcond(None, &spin.si_prefcond);
    w.section(SN_PREFCOND as c_int, SNF_REQUIRED, len);
    write_spell_prefcond(Some(w), &spin.si_prefcond);
}

/// `SN_REP`, `SN_SAL` and `SN_REPSAL`: the three from/to tables.
///
/// `REP` and `REPSAL` are sorted so the reader can index them by first
/// byte; `SAL` must keep the order the affix file gave, because sound
/// folding applies its rules in sequence. `SAL` is skipped entirely when
/// the language uses a `SOFOFROM`/`SOFOTO` pair instead.
fn put_rep_and_sal(w: &mut SplWriter, spin: &mut spellinfo_T) {
    let sofo = !spin.si_sofofr.is_null() && !spin.si_sofoto.is_null();
    for round in 1..=3 {
        let (table, sect_id) = match round {
            1 => (&mut spin.si_rep, SN_REP),
            2 if sofo => continue,
            2 => (&mut spin.si_sal, SN_SAL),
            _ => (&mut spin.si_repsal, SN_REPSAL),
        };
        if table.is_empty() {
            continue;
        }
        if round != 2 {
            // Entries that match the same text have no order of their own;
            // a stable sort leaves them as the affix file gave them, where
            // `qsort` left it to the implementation.
            table.sort_by(|a, b| a.from.cmp(&b.from));
        }

        // Two length-prefixed strings per entry, plus the count.
        let mut len = 2usize;
        for item in table.iter() {
            len += 2 + item.from.len() + item.to.len();
        }
        if round == 2 {
            // The extra flags byte SAL carries.
            len += 1;
        }
        let count = table.len();
        w.section(sect_id as c_int, 0, len);

        if round == 2 {
            let mut flags = 0;
            if spin.si_followup != 0 {
                flags |= SAL_F0LLOWUP as c_int;
            }
            if spin.si_collapse != 0 {
                flags |= SAL_COLLAPSE as c_int;
            }
            if spin.si_rem_accents != 0 {
                flags |= SAL_REM_ACCENTS as c_int;
            }
            w.byte(flags);
        }
        w.u16(count);

        let table = match round {
            1 => &spin.si_rep,
            2 => &spin.si_sal,
            _ => &spin.si_repsal,
        };
        let halves: Vec<u8> = table
            .iter()
            .flat_map(|item| [&item.from, &item.to])
            .flat_map(|half| {
                debug_assert!(half.len() < c_int::MAX as usize);
                core::iter::once(half.len() as u8).chain(half.iter().copied())
            })
            .collect();
        w.bytes(&halves);
    }
}

/// `SN_SOFO`: the simple character-mapping alternative to `SAL`.
unsafe fn put_sofo(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_sofofr.is_null() || spin.si_sofoto.is_null() {
        return;
    }
    // SAFETY: both are NUL-terminated arena strings.
    let (from, to) = unsafe {
        (
            cstr::bytes_at(spin.si_sofofr),
            cstr::bytes_at(spin.si_sofoto),
        )
    };
    // Two length-prefixed strings.
    w.section(SN_SOFO as c_int, 0, from.len() + to.len() + 4);
    w.u16(from.len());
    w.bytes(from);
    w.u16(to.len());
    w.bytes(to);
}

/// `SN_WORDS`: the `COMMON` word list, which makes suggestions of everyday
/// words score better. Counted on the first pass, written on the second.
unsafe fn put_words(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_commonwords.ht_used == 0 {
        return;
    }
    // SAFETY: the hash table's array holds `ht_used` live keys, each a
    // NUL-terminated string.
    w.byte(SN_WORDS as c_int);
    w.byte(0);
    // Keys go out with their terminator, so the reader can split them
    // apart again.
    let mut payload: Vec<u8> = Vec::new();
    for hi in spin.si_commonwords.items() {
        // SAFETY: every live key is a NUL-terminated string.
        payload.extend_from_slice(unsafe { cstr::bytes_at(hi.hi_key) });
        payload.push(NUL as u8);
    }
    w.u32(payload.len());
    w.bytes(&payload);
}

/// `SN_MAP`: groups of characters that count as near-equivalent when
/// scoring a suggestion.
fn put_map(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_map.is_empty() {
        return;
    }
    w.section(SN_MAP as c_int, 0, spin.si_map.len());
    w.bytes(&spin.si_map);
}

/// `SN_SUGFILE`: a timestamp stamped into both this file and the `.sug`
/// beside it, so a stale `.sug` can be spotted and ignored.
unsafe fn put_sugfile(w: &mut SplWriter, spin: &mut spellinfo_T) {
    let wanted =
        !spin.si_sal.is_empty() || (!spin.si_sofofr.is_null() && !spin.si_sofoto.is_null());
    if spin.si_nosugfile != 0 || !wanted {
        return;
    }
    w.section(SN_SUGFILE as c_int, 0, 8);
    // SAFETY: `time` with a null argument only returns the time.
    spin.si_sugtime = unsafe { time(core::ptr::null_mut::<time_t>()) };
    w.bytes(&spin.si_sugtime.to_be_bytes());
}

/// The sections that are pure on/off flags, carrying no payload.
fn put_flag_sections(w: &mut SplWriter, spin: &spellinfo_T) {
    for (set, id) in [
        (spin.si_nosplitsugs != 0, SN_NOSPLITSUGS),
        (spin.si_nocompoundsugs != 0, SN_NOCOMPOUNDSUGS),
    ] {
        if set {
            w.section(id as c_int, 0, 0);
        }
    }
}

/// `SN_COMPOUND`: the compounding limits, the `CHECKCOMPOUNDPATTERN` pairs
/// and the flags that say which words may join.
unsafe fn put_compound(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_compflags.is_null() {
        return;
    }
    // SAFETY: `si_compflags` is a NUL-terminated arena string.
    let compflags = unsafe { cstr::bytes_at(spin.si_compflags) };
    let patterns = &spin.si_comppat;

    let mut len = compflags.len();
    for pat in patterns {
        len += pat.len() + 1;
    }
    // Five limit bytes, a spare, and the two-byte pattern count.
    w.section(SN_COMPOUND as c_int, 0, len + 7);

    w.byte(spin.si_compmax);
    w.byte(spin.si_compminlen);
    w.byte(spin.si_compsylmax);
    w.byte(0);
    w.byte(spin.si_compoptions);
    w.u16(patterns.len());
    for pat in patterns {
        debug_assert!(pat.len() < c_int::MAX as usize);
        w.byte(pat.len() as c_int);
        w.bytes(pat);
    }
    w.bytes(compflags);
}

/// `SN_SYLLABLE`: the character groups that count as one syllable, for
/// `COMPOUNDSYLMAX`. Emitted after `SN_NOBREAK`, which shares this test.
unsafe fn put_syllable(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_nobreak != 0 {
        w.section(SN_NOBREAK as c_int, 0, 0);
    }
    if spin.si_syllable.is_null() {
        return;
    }
    // SAFETY: `si_syllable` is a NUL-terminated arena string.
    let text = unsafe { cstr::bytes_at(spin.si_syllable) };
    w.section(SN_SYLLABLE as c_int, 0, text.len());
    w.bytes(text);
}

/// The three word trees, each preceded by its node count.
///
/// The count is what [`put_node`] returns from a null-file pass, and it
/// also feeds the "estimated runtime memory use" figure `:mkspell` prints:
/// one byte plus one `int` per node is what the reader will allocate.
unsafe fn put_trees(w: &mut SplWriter, spin: &mut spellinfo_T, regionmask: c_int) {
    // SAFETY: the three roots are live and their trees compressed.
    spin.si_memtot = 0;
    for (round, root) in [spin.si_foldroot, spin.si_keeproot, spin.si_prefroot]
        .into_iter()
        .enumerate()
    {
        // The root node itself is only a holder; the tree hangs off it.
        let tree = unsafe { (*root).wn_sibling };
        let prefixtree = round == 2;

        unsafe { clear_node(tree) };
        let nodecount = unsafe { put_node(None, tree, 0, regionmask, prefixtree) } as usize;
        w.u32(nodecount);
        debug_assert!(nodecount + nodecount * size_of::<c_int>() < c_int::MAX as usize);
        spin.si_memtot += (nodecount + nodecount * size_of::<c_int>()) as c_int;

        unsafe { put_node(Some(w), tree, 0, regionmask, prefixtree) };
    }
}

/// Forget the indices and back-pointers a previous [`put_node`] pass left,
/// so the counting and writing passes agree.
///
/// # Safety
///
/// `node` must be null or head a live sibling chain.
pub(super) unsafe fn clear_node(node: *mut wordnode_T) {
    // SAFETY: the caller promises the chain; recursion stays inside it.
    let mut np = node;
    while !np.is_null() {
        unsafe { (*np).wn_index = 0 };
        unsafe { (*np).wn_link = core::ptr::null_mut() };
        if unsafe { (*np).wn_byte } as c_int != NUL {
            unsafe { clear_node((*np).wn_child) };
        }
        np = unsafe { (*np).wn_sibling };
    }
}

/// Write one sibling chain and everything below it, returning the index the
/// next chain would start at.
///
/// With a null `fd` nothing is written and the return value is just the
/// node count, which is what the caller needs before it can write.
///
/// A shared sub-tree is written once, under whichever parent reaches it
/// first; the others emit a [`BY_INDEX`] reference to it.
/// [`wn_link`](wordnode_T::wn_link) records that first parent, so the
/// second pass makes the same choice as the first.
///
/// # Safety
///
/// `node` must be null or head a live sibling chain that [`clear_node`] has
/// just been run over.
pub(super) unsafe fn put_node(
    mut w: Option<&mut SplWriter>,
    node: *mut wordnode_T,
    idx: c_int,
    regionmask: c_int,
    prefixtree: bool,
) -> c_int {
    // SAFETY: the caller promises the chain; every child dereferenced below
    // belongs to a node whose byte is not NUL, which is exactly when a
    // child exists.
    if node.is_null() {
        return 0;
    }
    unsafe { (*node).wn_index = idx };

    let mut siblingcount = 0;
    let mut np = node;
    while !np.is_null() {
        siblingcount += 1;
        np = unsafe { (*np).wn_sibling };
    }
    if let Some(w) = w.as_deref_mut() {
        w.byte(siblingcount);
    }

    let mut np = node;
    while !np.is_null() {
        if unsafe { (*np).wn_byte } as c_int == 0 {
            if let Some(w) = w.as_deref_mut() {
                // SAFETY: the byte is NUL, so this is a word end.
                unsafe { put_word_end(w, np, regionmask, prefixtree) };
            }
        } else {
            let child = unsafe { (*np).wn_child };
            if unsafe { (*child).wn_index } != 0 && unsafe { (*child).wn_link } != node {
                // Already written under a different parent.
                if let Some(w) = w.as_deref_mut() {
                    w.byte(BY_INDEX as c_int);
                    let at = unsafe { (*child).wn_index } as u32;
                    w.bytes(&at.to_be_bytes()[1..]);
                }
            } else if unsafe { (*child).wn_link }.is_null() {
                // Claim it: this chain will write it out below.
                unsafe { (*child).wn_link = node };
            }
            if let Some(w) = w.as_deref_mut() {
                w.byte(unsafe { (*np).wn_byte } as c_int);
                if !w.ok {
                    emsg(gettext(e_write));
                    return 0;
                }
            }
        }
        np = unsafe { (*np).wn_sibling };
    }

    // Children come after the whole chain, so a chain's nodes are
    // contiguous and one byte count covers them.
    let mut newindex = idx + siblingcount + 1;
    let mut np = node;
    while !np.is_null() {
        if unsafe { (*np).wn_byte } as c_int != 0 && unsafe { (*(*np).wn_child).wn_link } == node {
            let child = unsafe { (*np).wn_child };
            let sink = w.as_deref_mut();
            newindex = unsafe { put_node(sink, child, newindex, regionmask, prefixtree) };
        }
        np = unsafe { (*np).wn_sibling };
    }
    newindex
}

/// Write the properties of one word end: how many bytes follow and what
/// they mean.
///
/// # Safety
///
/// `np` must be a live node whose byte is NUL.
unsafe fn put_word_end(
    w: &mut SplWriter,
    np: *mut wordnode_T,
    regionmask: c_int,
    prefixtree: bool,
) {
    // SAFETY: the caller promises a live node.
    if prefixtree {
        // Prefix ids carry their own flag set; the common case has
        // none of the interesting bits and needs no flags byte.
        if unsafe { (*np).wn_flags } as c_int == PFX_FLAGS as u16 as c_int {
            w.byte(BY_NOFLAGS as c_int);
        } else {
            w.byte(BY_FLAGS as c_int);
            w.byte(unsafe { (*np).wn_flags } as c_int);
        }
        w.byte(unsafe { (*np).wn_affixID } as c_int);
        w.bytes(&unsafe { (*np).wn_region }.to_be_bytes());
        return;
    }

    // Region and affix bytes only follow when the word is not in every
    // region, or does take an affix.
    let mut flags = WordFlags::from_bits(unsafe { (*np).wn_flags } as c_int);
    if regionmask != 0 && unsafe { (*np).wn_region } as c_int != regionmask {
        flags |= WordFlags::REGION;
    }
    if unsafe { (*np).wn_affixID } as c_int != 0 {
        flags |= WordFlags::AFX;
    }
    if flags.is_empty() {
        w.byte(BY_NOFLAGS as c_int);
        return;
    }
    if unsafe { (*np).wn_flags } as c_int >= 0x100 {
        w.byte(BY_FLAGS2 as c_int);
        w.byte(flags.bits());
        w.byte((flags.bits() as core::ffi::c_uint >> 8) as c_int);
    } else {
        w.byte(BY_FLAGS as c_int);
        w.byte(flags.bits());
    }
    if flags.has(WordFlags::REGION) {
        w.byte(unsafe { (*np).wn_region } as c_int);
    }
    if flags.has(WordFlags::AFX) {
        w.byte(unsafe { (*np).wn_affixID } as c_int);
    }
}

/// Measure or write the `SN_PREFCOND` payload: a count, then one
/// length-prefixed regexp per prefix id, with a zero length where an id has
/// no condition.
///
/// With no writer nothing is written and only the total is returned.
pub(super) fn write_spell_prefcond(
    mut w: Option<&mut SplWriter>,
    conds: &[Option<Box<[u8]>>],
) -> usize {
    if let Some(w) = w.as_deref_mut() {
        w.u16(conds.len());
    }

    // The count, plus one length byte per entry.
    let mut totlen = 2 + conds.len();
    for cond in conds {
        let bytes = cond.as_deref().unwrap_or_default();
        if let Some(w) = w.as_deref_mut() {
            debug_assert!(bytes.len() <= c_int::MAX as usize);
            w.byte(bytes.len() as c_int);
            w.bytes(bytes);
        }
        totlen += bytes.len();
    }
    debug_assert!(totlen <= c_int::MAX as usize);
    totlen
}
