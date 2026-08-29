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

use crate::semsg;
use core::ffi::{c_char, c_int, c_void};

use crate::fileio::{put_bytes, put_time};
use crate::hashtab::hash_removed;
use crate::main::e_write;
use crate::mbyte::utf_char2bytes;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, putc};
use crate::os::fs::os_fopen;
use crate::spell::{spelltab_fold, spelltab_isu, spelltab_isw};
use crate::types::{FILE, NUL, fromto_T, garray_T, size_t, time_t, uintmax_t};
use ::libc::{fclose, fputc, fwrite, qsort, strcmp, strlen, time};

use super::wordtree::wordnode_T;
use super::{
    BY_FLAGS, BY_FLAGS2, BY_INDEX, BY_NOFLAGS, CF_UPPER, CF_WORD, EOF, FAIL, OK, PFX_FLAGS,
    SAL_COLLAPSE, SAL_F0LLOWUP, SAL_REM_ACCENTS, SN_CHARFLAGS, SN_COMPOUND, SN_END, SN_INFO,
    SN_MAP, SN_MIDWORD, SN_NOBREAK, SN_NOCOMPOUNDSUGS, SN_NOSPLITSUGS, SN_PREFCOND, SN_REGION,
    SN_REP, SN_REPSAL, SN_SAL, SN_SOFO, SN_SUGFILE, SN_SYLLABLE, SN_WORDS, SNF_REQUIRED,
    VIMSPELLMAGIC, VIMSPELLMAGICL, VIMSPELLVERSION, WF_AFX, WF_REGION, spellinfo_T,
};

/// A `.spl` file being written.
struct SplWriter {
    fd: *mut FILE,
    /// Cleared by the first payload write that does not report one item
    /// written. Checked once, when the file is closed.
    ok: bool,
}

impl SplWriter {
    fn byte(&self, c: c_int) -> c_int {
        // SAFETY: `fd` is open for the writer's whole lifetime.
        unsafe { putc(c, self.fd) }
    }

    fn u32(&self, v: usize) {
        // SAFETY: as above.
        unsafe { put_bytes(self.fd, v as uintmax_t, 4) };
    }

    fn u16(&self, v: usize) {
        // SAFETY: as above.
        unsafe { put_bytes(self.fd, v as uintmax_t, 2) };
    }

    /// Write `len` bytes of payload.
    ///
    /// # Safety
    ///
    /// `p` must point at `len` readable bytes.
    unsafe fn payload(&mut self, p: *const c_void, len: usize) {
        // SAFETY: the caller promises the range; `fd` is open.
        self.ok &= unsafe { fwrite(p, len, 1, self.fd) } == 1;
    }

    /// Write a NUL-terminated string's bytes, without the terminator.
    ///
    /// # Safety
    ///
    /// `p` must point at a NUL-terminated string.
    unsafe fn payload_str(&mut self, p: *const c_char) -> usize {
        // SAFETY: the caller promises a terminated string.
        let len = unsafe { strlen(p) };
        unsafe { self.payload(p.cast(), len) };
        len
    }

    /// Open a section: its id, its flags, and the payload length that
    /// follows.
    fn section(&self, id: c_int, flags: c_int, len: usize) {
        self.byte(id);
        self.byte(flags);
        self.u32(len);
    }
}

/// Order `REP`/`REPSAL` entries by what they match, so the reader can stop
/// searching once it passes the first byte.
///
/// Kept on the C ABI for `qsort`: entries that compare equal have no
/// defined order, so which of them ends up first is the sort's choice, and
/// a Rust sort would be free to choose differently.
pub(super) unsafe extern "C" fn rep_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: qsort passes elements of the `fromto_T` array it was given.
    unsafe {
        strcmp(
            (*s1.cast::<fromto_T>()).ft_from,
            (*s2.cast::<fromto_T>()).ft_from,
        )
    }
}

/// Write the whole `.spl` file.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path and `spin` must hold finished,
/// compressed trees.
pub(super) unsafe fn write_vim_spell(spin: &mut spellinfo_T, fname: *mut c_char) -> c_int {
    // SAFETY: every pointer read below comes from `spin`, whose strings are
    // arena-allocated and NUL-terminated, or from the word trees.
    let fd = unsafe { os_fopen(fname, c"w".as_ptr()) };
    if fd.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E484: Can't open file {fname}");
        return FAIL;
    }
    let mut w = SplWriter { fd, ok: true };
    let mut retval = OK;

    unsafe { w.payload(VIMSPELLMAGIC.as_ptr().cast(), VIMSPELLMAGICL) };
    // A failed magic write means the file is unusable; the C skipped
    // straight to the close and so does this.
    if w.ok {
        w.byte(VIMSPELLVERSION);

        unsafe { put_info(&mut w, spin) };
        let regionmask = unsafe { put_region(&mut w, spin) };
        unsafe { put_charflags(&mut w, spin) };
        unsafe { put_midword(&mut w, spin) };
        unsafe { put_prefcond(&mut w, spin) };
        unsafe { put_rep_and_sal(&mut w, spin) };
        unsafe { put_sofo(&mut w, spin) };
        unsafe { put_words(&mut w, spin) };
        unsafe { put_map(&mut w, spin) };
        unsafe { put_sugfile(&mut w, spin) };
        unsafe { put_flag_sections(&mut w, spin) };
        unsafe { put_compound(&mut w, spin) };
        unsafe { put_syllable(&mut w, spin) };

        w.byte(SN_END as c_int);
        unsafe { put_trees(&mut w, spin, regionmask) };

        // The trailing byte the reader uses to tell a complete file
        // from a truncated one.
        if w.byte(0) == EOF {
            retval = FAIL;
        }
    }

    if unsafe { fclose(fd) } == EOF {
        retval = FAIL;
    }
    if !w.ok {
        retval = FAIL;
    }
    if retval == FAIL {
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
    let len = unsafe { strlen(spin.si_info) };
    w.section(SN_INFO as c_int, 0, len);
    unsafe { w.payload(spin.si_info.cast(), len) };
}

/// `SN_REGION`: the two-letter region names. Returns the mask of all
/// regions, which the tree writer uses to spot words that are in every one.
unsafe fn put_region(w: &mut SplWriter, spin: &spellinfo_T) -> c_int {
    if spin.si_region_count <= 1 {
        return 0;
    }
    // SAFETY: `si_region_name` holds `si_region_count` two-byte names.
    let len = spin.si_region_count as usize * 2;
    w.section(SN_REGION as c_int, SNF_REQUIRED, len);
    unsafe { w.payload((&raw const spin.si_region_name).cast(), len) };
    (1 << spin.si_region_count) - 1
}

/// `SN_CHARFLAGS`: which of the bytes 128..255 are word characters and
/// which are upper case, plus the folded form of each.
///
/// Only meaningful for a non-ASCII base dictionary; an `.add` file inherits
/// the table from the file it extends.
unsafe fn put_charflags(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_ascii != 0 || spin.si_add != 0 {
        return;
    }
    // SAFETY: `folchars` has room for 128 characters at up to four bytes
    // each, well past what the fold table can produce, and `spelltab` is
    // initialised before any spell file is written.
    let mut folchars: [c_char; 1024] = [0; 1024];
    let mut folen = 0usize;
    for i in 128..256 {
        folen +=
            unsafe { utf_char2bytes(spelltab_fold(i) as c_int, folchars.as_mut_ptr().add(folen)) }
                as usize;
    }

    w.section(SN_CHARFLAGS as c_int, SNF_REQUIRED, 1 + 128 + 2 + folen);
    unsafe { fputc(128, w.fd) };
    for i in 128..256 {
        let mut flags = 0;
        if spelltab_isw(i) {
            flags |= CF_WORD as c_int;
        }
        if spelltab_isu(i) {
            flags |= CF_UPPER as c_int;
        }
        unsafe { fputc(flags, w.fd) };
    }
    w.u16(folen);
    unsafe { w.payload(folchars.as_ptr().cast(), folen) };
}

/// `SN_MIDWORD`: characters that may appear inside a word without ending
/// it, such as an apostrophe.
unsafe fn put_midword(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_midword.is_null() {
        return;
    }
    // SAFETY: `si_midword` is a NUL-terminated arena string.
    let len = unsafe { strlen(spin.si_midword) };
    w.section(SN_MIDWORD as c_int, SNF_REQUIRED, len);
    unsafe { w.payload(spin.si_midword.cast(), len) };
}

/// `SN_PREFCOND`: the regexps a prefix's condition compiles from, one per
/// prefix id. Measured with a null file first, since the length has to
/// precede the payload.
unsafe fn put_prefcond(w: &mut SplWriter, spin: &mut spellinfo_T) {
    if spin.si_prefcond.ga_len <= 0 {
        return;
    }
    // SAFETY: `si_prefcond` holds NUL-terminated strings and nulls.
    let mut ok = w.ok;
    let len =
        unsafe { write_spell_prefcond(core::ptr::null_mut(), &raw mut spin.si_prefcond, &mut ok) };
    w.section(SN_PREFCOND as c_int, SNF_REQUIRED, len as usize);
    unsafe { write_spell_prefcond(w.fd, &raw mut spin.si_prefcond, &mut ok) };
    w.ok = ok;
}

/// `SN_REP`, `SN_SAL` and `SN_REPSAL`: the three from/to tables.
///
/// `REP` and `REPSAL` are sorted so the reader can index them by first
/// byte; `SAL` must keep the order the affix file gave, because sound
/// folding applies its rules in sequence. `SAL` is skipped entirely when
/// the language uses a `SOFOFROM`/`SOFOTO` pair instead.
unsafe fn put_rep_and_sal(w: &mut SplWriter, spin: &mut spellinfo_T) {
    // SAFETY: each garray holds `ga_len` `fromto_T`s of NUL-terminated
    // arena strings.
    let sofo = !spin.si_sofofr.is_null() && !spin.si_sofoto.is_null();
    for round in 1..=3 {
        let (gap, sect_id) = match round {
            1 => (&raw mut spin.si_rep, SN_REP),
            2 if sofo => continue,
            2 => (&raw mut spin.si_sal, SN_SAL),
            _ => (&raw mut spin.si_repsal, SN_REPSAL),
        };
        if unsafe { (*gap).ga_len } <= 0 {
            continue;
        }
        if round != 2 {
            unsafe {
                qsort(
                    (*gap).ga_data,
                    (*gap).ga_len as size_t,
                    size_of::<fromto_T>(),
                    Some(rep_compare),
                )
            };
        }
        debug_assert!(unsafe { (*gap).ga_len } >= 0);

        // Two length-prefixed strings per entry, plus the count.
        let entries = unsafe { (*gap).ga_data }.cast::<fromto_T>();
        let mut len = 2usize;
        for i in 0..unsafe { (*gap).ga_len } as usize {
            let ftp = unsafe { entries.add(i) };
            len += 1 + unsafe { strlen((*ftp).ft_from) };
            len += 1 + unsafe { strlen((*ftp).ft_to) };
        }
        if round == 2 {
            // The extra flags byte SAL carries.
            len += 1;
        }
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
        w.u16(unsafe { (*gap).ga_len } as usize);

        for i in 0..unsafe { (*gap).ga_len } as usize {
            let ftp = unsafe { entries.add(i) };
            for p in [unsafe { (*ftp).ft_from }, unsafe { (*ftp).ft_to }] {
                let l = unsafe { strlen(p) };
                debug_assert!(l < c_int::MAX as usize);
                w.byte(l as c_int);
                // A zero-length half is legitimate here, and writing
                // zero items would wrongly mark the file failed.
                if l > 0 {
                    unsafe { w.payload(p.cast(), l) };
                }
            }
        }
    }
}

/// `SN_SOFO`: the simple character-mapping alternative to `SAL`.
unsafe fn put_sofo(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_sofofr.is_null() || spin.si_sofoto.is_null() {
        return;
    }
    // SAFETY: both are NUL-terminated arena strings.
    let from_len = unsafe { strlen(spin.si_sofofr) };
    let to_len = unsafe { strlen(spin.si_sofoto) };
    // Two length-prefixed strings.
    w.section(SN_SOFO as c_int, 0, from_len + to_len + 4);
    w.u16(from_len);
    unsafe { w.payload(spin.si_sofofr.cast(), from_len) };
    w.u16(to_len);
    unsafe { w.payload(spin.si_sofoto.cast(), to_len) };
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
    for round in 1..=2 {
        let mut todo = spin.si_commonwords.ht_used;
        let mut hi = spin.si_commonwords.ht_array;
        let mut len = 0usize;
        while todo > 0 {
            if !unsafe { (*hi).hi_key }.is_null()
                && unsafe { (*hi).hi_key } != (&raw const hash_removed).cast_mut().cast()
            {
                // Keys go out with their terminator, so the reader can
                // split them apart again.
                let l = unsafe { strlen((*hi).hi_key) } + 1;
                len += l;
                if round == 2 {
                    unsafe { w.payload((*hi).hi_key.cast(), l) };
                }
                todo -= 1;
            }
            hi = unsafe { hi.add(1) };
        }
        if round == 1 {
            w.u32(len);
        }
    }
}

/// `SN_MAP`: groups of characters that count as near-equivalent when
/// scoring a suggestion.
unsafe fn put_map(w: &mut SplWriter, spin: &spellinfo_T) {
    if spin.si_map.ga_len <= 0 {
        return;
    }
    // SAFETY: `ga_data` holds `ga_len` bytes.
    let len = spin.si_map.ga_len as usize;
    w.section(SN_MAP as c_int, 0, len);
    unsafe { w.payload(spin.si_map.ga_data, len) };
}

/// `SN_SUGFILE`: a timestamp stamped into both this file and the `.sug`
/// beside it, so a stale `.sug` can be spotted and ignored.
unsafe fn put_sugfile(w: &mut SplWriter, spin: &mut spellinfo_T) {
    let wanted = spin.si_sal.ga_len > 0 || (!spin.si_sofofr.is_null() && !spin.si_sofoto.is_null());
    if spin.si_nosugfile != 0 || !wanted {
        return;
    }
    // SAFETY: `fd` is open.
    w.section(SN_SUGFILE as c_int, 0, 8);
    spin.si_sugtime = unsafe { time(core::ptr::null_mut::<time_t>()) };
    unsafe { put_time(w.fd, spin.si_sugtime) };
}

/// The sections that are pure on/off flags, carrying no payload.
unsafe fn put_flag_sections(w: &mut SplWriter, spin: &spellinfo_T) {
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
    // SAFETY: `si_comppat` holds `ga_len` NUL-terminated strings and
    // `si_compflags` is one too.
    debug_assert!(spin.si_comppat.ga_len >= 0);
    let patterns = spin.si_comppat.ga_data.cast::<*mut c_char>();
    let count = spin.si_comppat.ga_len as usize;

    let mut len = unsafe { strlen(spin.si_compflags) };
    for i in 0..count {
        len += unsafe { strlen(*patterns.add(i)) } + 1;
    }
    // Five limit bytes, a spare, and the two-byte pattern count.
    w.section(SN_COMPOUND as c_int, 0, len + 7);

    w.byte(spin.si_compmax);
    w.byte(spin.si_compminlen);
    w.byte(spin.si_compsylmax);
    w.byte(0);
    w.byte(spin.si_compoptions);
    w.u16(count);
    for i in 0..count {
        let p = unsafe { *patterns.add(i) };
        debug_assert!(unsafe { strlen(p) } < c_int::MAX as usize);
        w.byte(unsafe { strlen(p) } as c_int);
        unsafe { w.payload_str(p) };
    }
    unsafe { w.payload_str(spin.si_compflags) };
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
    let len = unsafe { strlen(spin.si_syllable) };
    w.section(SN_SYLLABLE as c_int, 0, len);
    unsafe { w.payload(spin.si_syllable.cast(), len) };
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
        let nodecount =
            unsafe { put_node(core::ptr::null_mut(), tree, 0, regionmask, prefixtree) } as usize;
        w.u32(nodecount);
        debug_assert!(nodecount + nodecount * size_of::<c_int>() < c_int::MAX as usize);
        spin.si_memtot += (nodecount + nodecount * size_of::<c_int>()) as c_int;

        unsafe { put_node(w.fd, tree, 0, regionmask, prefixtree) };
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
    fd: *mut FILE,
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
    if !fd.is_null() {
        unsafe { putc(siblingcount, fd) };
    }

    let mut np = node;
    while !np.is_null() {
        if unsafe { (*np).wn_byte } as c_int == 0 {
            if !fd.is_null() {
                unsafe { put_word_end(fd, np, regionmask, prefixtree) };
            }
        } else {
            let child = unsafe { (*np).wn_child };
            if unsafe { (*child).wn_index } != 0 && unsafe { (*child).wn_link } != node {
                // Already written under a different parent.
                if !fd.is_null() {
                    unsafe { putc(BY_INDEX as c_int, fd) };
                    unsafe { put_bytes(fd, (*child).wn_index as uintmax_t, 3) };
                }
            } else if unsafe { (*child).wn_link }.is_null() {
                // Claim it: this chain will write it out below.
                unsafe { (*child).wn_link = node };
            }
            if !fd.is_null() && unsafe { putc((*np).wn_byte as c_int, fd) } == EOF {
                emsg(gettext(e_write));
                return 0;
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
            newindex = unsafe { put_node(fd, (*np).wn_child, newindex, regionmask, prefixtree) };
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
/// `np` must be a live node whose byte is NUL, and `fd` open.
unsafe fn put_word_end(fd: *mut FILE, np: *mut wordnode_T, regionmask: c_int, prefixtree: bool) {
    // SAFETY: the caller promises a live node and an open file.
    if prefixtree {
        // Prefix ids carry their own flag set; the common case has
        // none of the interesting bits and needs no flags byte.
        if unsafe { (*np).wn_flags } as c_int == PFX_FLAGS as u16 as c_int {
            unsafe { putc(BY_NOFLAGS as c_int, fd) };
        } else {
            unsafe { putc(BY_FLAGS as c_int, fd) };
            unsafe { putc((*np).wn_flags as c_int, fd) };
        }
        unsafe { putc((*np).wn_affixID as c_int, fd) };
        unsafe { put_bytes(fd, (*np).wn_region as uintmax_t, 2) };
        return;
    }

    // Region and affix bytes only follow when the word is not in every
    // region, or does take an affix.
    let mut flags = unsafe { (*np).wn_flags } as c_int;
    if regionmask != 0 && unsafe { (*np).wn_region } as c_int != regionmask {
        flags |= WF_REGION as c_int;
    }
    if unsafe { (*np).wn_affixID } as c_int != 0 {
        flags |= WF_AFX as c_int;
    }
    if flags == 0 {
        unsafe { putc(BY_NOFLAGS as c_int, fd) };
        return;
    }
    if unsafe { (*np).wn_flags } as c_int >= 0x100 {
        unsafe { putc(BY_FLAGS2 as c_int, fd) };
        unsafe { putc(flags, fd) };
        unsafe { putc((flags as core::ffi::c_uint >> 8) as c_int, fd) };
    } else {
        unsafe { putc(BY_FLAGS as c_int, fd) };
        unsafe { putc(flags, fd) };
    }
    if flags & WF_REGION as c_int != 0 {
        unsafe { putc((*np).wn_region as c_int, fd) };
    }
    if flags & WF_AFX as c_int != 0 {
        unsafe { putc((*np).wn_affixID as c_int, fd) };
    }
}

/// Measure or write the `SN_PREFCOND` payload: a count, then one
/// length-prefixed regexp per prefix id, with a zero length where an id has
/// no condition.
///
/// With a null `fd` nothing is written and only the total is returned.
///
/// # Safety
///
/// `gap` must hold `ga_len` pointers, each null or NUL-terminated.
pub(super) unsafe fn write_spell_prefcond(
    fd: *mut FILE,
    gap: *mut garray_T,
    ok: &mut bool,
) -> c_int {
    // SAFETY: the caller promises the array's shape.
    debug_assert!(unsafe { (*gap).ga_len } >= 0);
    if !fd.is_null() {
        unsafe { put_bytes(fd, (*gap).ga_len as uintmax_t, 2) };
    }

    // The count, plus one length byte per entry.
    let mut totlen = 2 + unsafe { (*gap).ga_len } as usize;
    let entries = unsafe { (*gap).ga_data }.cast::<*mut c_char>();
    for i in 0..unsafe { (*gap).ga_len } as usize {
        let p = unsafe { *entries.add(i) };
        if p.is_null() {
            if !fd.is_null() {
                unsafe { fputc(0, fd) };
            }
            continue;
        }
        let len = unsafe { strlen(p) };
        if !fd.is_null() {
            debug_assert!(len <= c_int::MAX as usize);
            unsafe { fputc(len as c_int, fd) };
            *ok &= unsafe { fwrite(p.cast(), len, 1, fd) } == 1;
        }
        totlen += len;
    }
    debug_assert!(totlen <= c_int::MAX as usize);
    totlen as c_int
}
