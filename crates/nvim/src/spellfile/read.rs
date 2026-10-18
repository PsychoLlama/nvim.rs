//! Reading a `.spl` file, and the `.sug` beside it.
//!
//! This is a parser over bytes nobody vouched for: a `.spl` is just a file
//! on `'runtimepath'`, and every length, count and index in it is whatever
//! the file says. The file is read through [`Spl`], whose every answer is a
//! `Result` carrying one of the three [`SpellReadError`]s — the old `SP_*`
//! codes, under their own type.
//!
//! [`spell_load_file`] walks the section list, hands each section to
//! [`sections`](super::sections), and finishes with the three word trees.
//! An unknown section is skipped by its length — unless its flags say
//! [`SNF_REQUIRED`], in which case the file is refused, which is how the
//! format stays extensible without silently mis-reading a newer file.
//!
//! # Bounds
//!
//! The trees are the part worth watching, because they are read into flat
//! arrays and index into themselves. [`read_tree_node`] checks every index
//! against the array length before storing it and caps nesting at
//! [`MAXWLEN`], so a cycle or a wild index cannot get through. What it
//! permits, [`tree_count_words`] then has to survive — see the note there.
//!
//! # Where the end of the file is *not* an error
//!
//! Several reads in the tree parser are deliberately unchecked upstream:
//! they take `getc`'s `-1` as a value and let a later test reject it. Two of
//! those tests are the only thing standing between a corrupt prefix tree and
//! `E759` rather than `E758`, so those sites spell the fallback out
//! (`unwrap_or(-1)`) instead of propagating the error.

#![forbid(unsafe_code)]

use crate::semsg;
use crate::smsg;
use crate::spell::{WordFlags, WordTree};
use crate::strings::has_bytes;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::getchar::state::got_int;
use crate::memline::ml_append_buf_terminated;
use crate::memory::XString;
use crate::message::{emsg, verbose_enter, verbose_leave};
use crate::message_fmt::msg_cstr;
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext;
use crate::os::input::fast_breakcheck;
use crate::path::{path_fnamecmp, path_full_compare, path_tail};
use crate::runtime::{estack_pop, estack_push};
use crate::spell::{
    e_format, init_syl_tab, languages, open_spellbuf, parse_spelllang, put_lang, slang_clear,
    slang_clear_sug, take_lang, with_lang,
};
use crate::types::{LangId, LineNr, NUL, OptInt, SpellIdx, SpellLang, time_t, uint8_t};

use super::sections::{
    read_charflags_section, read_compound, read_prefcond_section, read_region_section,
    read_rep_section, read_sal_section, read_sofo_section, read_words_section, set_map_str,
};
use super::spl::{SpellReadError, Spl, SplResult, trim_nul};
use super::{
    BY_FLAGS, BY_FLAGS2, BY_INDEX, BY_NOFLAGS, BY_SPECIAL, ETYPE_SPELL, MAXWLEN, OK, SN_CHARFLAGS,
    SN_COMPOUND, SN_END, SN_INFO, SN_MAP, SN_MIDWORD, SN_NOBREAK, SN_NOCOMPOUNDSUGS,
    SN_NOSPLITSUGS, SN_PREFCOND, SN_REGION, SN_REP, SN_REPSAL, SN_SAL, SN_SOFO, SN_SUGFILE,
    SN_SYLLABLE, SN_WORDS, SNF_REQUIRED, SPL_FNAME_ADD, VIMSPELLMAGIC, VIMSPELLMAGICL,
    VIMSPELLVERSION, VIMSUGMAGIC, VIMSUGMAGICL, VIMSUGVERSION, e_spell_trunc, kEqualFiles,
};

/// Marks a tree index that already points at a shared sub-tree, so the
/// second pass over a node's bytes knows not to descend into it again.
pub(super) const SHARED_MASK: c_int = 0x8000000;

/// Why the load stopped.
///
/// [`Stop::Silent`] is the section reader's "stop, and say nothing further":
/// either the message is already on the screen, or the case is one upstream
/// abandons the file over without a word.
enum Stop {
    /// One of the reader's own three errors, still to be reported.
    Read(SpellReadError),
    /// Nothing more to say.
    Silent,
}

impl From<SpellReadError> for Stop {
    fn from(e: SpellReadError) -> Self {
        Stop::Read(e)
    }
}

/// Check the eight magic bytes a `.spl` starts with.
fn spell_check_magic_string(spl: &mut Spl) -> SplResult<()> {
    let mut buf = [0u8; VIMSPELLMAGICL];
    spl.read_exact(&mut buf)?;
    if buf != VIMSPELLMAGIC.to_bytes() {
        return Err(SpellReadError::Format);
    }
    Ok(())
}

/// Load a spell file, returning the language it describes.
///
/// The language is named `lang` (none for a file read only to be looked
/// at); the caller lists it. A `.add` file is recognised by its name and
/// adds to the language it is loaded with rather than defining one.
pub fn spell_load_file(fname: &CStr, lang: Option<&[u8]>, silent: bool) -> Option<Box<SpellLang>> {
    let path = Path::new(OsStr::from_bytes(fname.to_bytes()));
    let opened = Spl::open(path);
    let mut slang = None;
    let mut did_estack_push = false;
    let ok = load_spl(
        opened,
        fname,
        None,
        silent,
        &mut slang,
        &mut did_estack_push,
        lang,
    );
    if did_estack_push {
        estack_pop();
    }
    if ok { slang } else { None }
}

/// Read `fname` again into `slang`, which [`slang_clear`] has emptied: the
/// reload that keeps a language where windows find it.
pub(crate) fn spell_reload_file(fname: &CStr, slang: &mut SpellLang) -> bool {
    let path = Path::new(OsStr::from_bytes(fname.to_bytes()));
    let opened = Spl::open(path);
    let mut did_estack_push = false;
    let mut fresh = None;
    let ok = load_spl(
        opened,
        fname,
        Some(slang),
        false,
        &mut fresh,
        &mut did_estack_push,
        None,
    );
    if did_estack_push {
        estack_pop();
    }
    ok
}

/// The body of [`spell_load_file`]: true when the language is usable. Reads
/// into `old` when given, else into a new language left in `fresh`.
fn load_spl(
    opened: std::io::Result<Spl>,
    fname: &CStr,
    old: Option<&mut SpellLang>,
    silent: bool,
    fresh: &mut Option<Box<SpellLang>>,
    did_estack_push: &mut bool,
    lang: Option<&[u8]>,
) -> bool {
    let Ok(mut spl) = opened else {
        if !silent {
            let fname = msg_cstr(fname);
            semsg!("E484: Can't open file {fname}");
        } else if p_verbose() > 2 as OptInt {
            verbose_enter();
            let fname = msg_cstr(fname);
            smsg!(0, "E484: Can't open file {fname}");
            verbose_leave();
        }
        return false;
    };
    if p_verbose() > 2 as OptInt {
        verbose_enter();
        let fname = msg_cstr(fname);
        smsg!(0, "Reading spell file \"{fname}\"");
        verbose_leave();
    }

    let slang = match old {
        Some(slang) => slang,
        None => {
            let mut slang = SpellLang::new(lang.unwrap_or_default(), XString::from_cstr(fname));
            // ".add.spl" files add to an existing language rather than
            // defining one.
            slang.sl_add = has_bytes(path_tail(fname), SPL_FNAME_ADD.to_bytes());
            &mut **fresh.insert(slang)
        }
    };

    estack_push(
        ETYPE_SPELL,
        crate::types::FrameName::owned(fname),
        0 as LineNr,
    );
    *did_estack_push = true;

    read_spl(&mut spl, slang, fname)
}

/// Read the header, every section and the three trees.
fn read_spl(spl: &mut Spl, slang: &mut SpellLang, fname: &CStr) -> bool {
    match spell_check_magic_string(spl) {
        Err(SpellReadError::Format | SpellReadError::Trunc) => {
            let fmt = gettext(c"E757: This does not look like a spell file");
            // SAFETY: `gettext`'s answer is a NUL-terminated string.
            let arg0 = msg_cstr(fmt);
            semsg!("{arg0}");
            return false;
        }
        Err(SpellReadError::Other) => {
            let why = spl.last_error();
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = msg_cstr(fname);
            semsg!("E5042: Failed to read spell file {fname}: {why}");
            return false;
        }
        Ok(()) => {}
    }

    // A missing version byte reads as -1, which is "too old".
    let version = spl.getc().map_or(-1, c_int::from);
    if version < VIMSPELLVERSION {
        emsg(gettext(c"E771: Old spell file, needs to be updated"));
        return false;
    }
    if version > VIMSPELLVERSION {
        emsg(gettext(c"E772: Spell file is for newer version of Vim"));
        return false;
    }

    loop {
        let id = spl.getc().map_or(-1, c_int::from);
        let res = if id == SN_END as c_int {
            match read_trees(spl, slang) {
                Ok(()) => {
                    return true;
                }
                Err(e) => Err(Stop::Read(e)),
            }
        } else {
            let flags = spl.getc().map_or(-1, c_int::from);
            let len = spl.get4c().unwrap_or(-1);
            if len < 0 {
                break;
            }
            read_section(spl, slang, id, flags, len)
        };

        match res {
            Ok(()) => {}
            Err(Stop::Read(SpellReadError::Format)) => {
                // SAFETY: the message table's own string.
                emsg(gettext(e_format));
                return false;
            }
            Err(Stop::Read(SpellReadError::Trunc)) => break,
            Err(Stop::Read(SpellReadError::Other) | Stop::Silent) => return false,
        }
    }

    // SAFETY: the message table's own string.
    emsg(gettext(e_spell_trunc));
    false
}

/// Dispatch one section by its id.
fn read_section(
    spl: &mut Spl,
    slang: &mut SpellLang,
    id: c_int,
    flags: c_int,
    mut len: c_int,
) -> Result<(), Stop> {
    match id as u32 {
        SN_INFO => {
            let text = spl.read_string(len as usize).map_err(|_| Stop::Silent)?;
            slang.sl_info = Some(XString::from_bytes(trim_nul(&text)));
        }
        SN_REGION => read_region_section(spl, slang, len)?,
        SN_CHARFLAGS => read_charflags_section(spl)?,
        SN_MIDWORD => {
            let text = spl.read_string(len as usize).map_err(|_| Stop::Silent)?;
            slang.sl_midword = trim_nul(&text).to_vec();
        }
        SN_PREFCOND => read_prefcond_section(spl, slang)?,
        SN_REP => read_rep_section(spl, &mut slang.sl_rep, &mut slang.sl_rep_first)?,
        SN_REPSAL => read_rep_section(spl, &mut slang.sl_repsal, &mut slang.sl_repsal_first)?,
        SN_SAL => read_sal_section(spl, slang)?,
        SN_SOFO => read_sofo_section(spl, slang)?,
        SN_MAP => {
            let text = spl.read_string(len as usize).map_err(|_| Stop::Silent)?;
            set_map_str(slang, trim_nul(&text));
        }
        SN_WORDS => read_words_section(spl, slang, len)?,
        SN_SUGFILE => slang.sl_sugtime = spl.get8ctime().unwrap_or(-1),
        SN_NOSPLITSUGS => slang.sl_nosplitsugs = true,
        SN_NOCOMPOUNDSUGS => slang.sl_nocompoundsugs = true,
        SN_COMPOUND => read_compound(spl, slang, len)?,
        SN_NOBREAK => slang.sl_nobreak = true,
        SN_SYLLABLE => {
            let text = spl.read_string(len as usize).map_err(|_| Stop::Silent)?;
            if init_syl_tab(slang, trim_nul(&text)) != OK {
                return Err(Stop::Silent);
            }
        }
        _ => {
            // An unknown section is only fatal when the file insists
            // it is understood; otherwise step over it.
            if flags & SNF_REQUIRED != 0 {
                emsg(gettext(c"E770: Unsupported section in spell file"));
                return Err(Stop::Silent);
            }
            loop {
                len -= 1;
                if len < 0 {
                    break;
                }
                if spl.getc().is_none() {
                    return Err(Stop::Read(SpellReadError::Trunc));
                }
            }
        }
    }
    Ok(())
}

/// Read the three trees that close a `.spl`: case-folded words, keep-case
/// words, and prefixes.
fn read_trees(spl: &mut Spl, slang: &mut SpellLang) -> SplResult<()> {
    spell_read_tree(spl, &mut slang.sl_fold_tree, false, 0)?;
    spell_read_tree(spl, &mut slang.sl_keep_tree, false, 0)?;
    // The prefix tree's entries name a prefix condition by number, so
    // it can only be read once SN_PREFCOND has said how many there are.
    let conds = slang.sl_prefixcnt;
    spell_read_tree(spl, &mut slang.sl_prefix_tree, true, conds)
}

/// Load the `.sug` file for every language of the current window that has
/// one and has not tried yet.
///
/// A `.sug` is optional and best-effort: anything wrong with it is
/// reported and the language carries on without sound-a-like suggestions.
pub fn suggest_load_files() {
    let ids: Vec<LangId> = Win::current()
        .syn_block()
        .b_langp
        .iter()
        .map(|lp| lp.slang)
        .collect();
    for id in ids {
        // Out of its slot while the file is read: reading gives messages.
        let Some(mut slang) = take_lang(id) else {
            continue;
        };
        if slang.sl_sugtime != 0 as time_t && !slang.sl_sugloaded {
            // One attempt per language, successful or not.
            slang.sl_sugloaded = true;

            // The `.sug` sits beside the `.spl` under the same stem.
            let fname = &slang.sl_fname[..];
            if let Some(dot) = fname.iter().rposition(|&b| b == b'.')
                && path_fnamecmp(&fname[dot..], b".spl") == 0
            {
                let mut sug = fname[..dot].to_vec();
                sug.extend_from_slice(b".sug");
                let path = Path::new(OsStr::from_bytes(&sug));
                if let Ok(mut spl) = Spl::open(path) {
                    load_sug(&mut spl, &mut slang);
                }
            }
        }
        put_lang(id, slang);
    }
}

/// Read one `.sug` file into `slang`.
fn load_sug(spl: &mut Spl, slang: &mut SpellLang) {
    let mut buf = [0u8; VIMSUGMAGICL as usize];
    let _ = spl.read_exact(&mut buf);
    if buf != VIMSUGMAGIC.to_bytes() {
        let sl_fname = msg_cstr(slang.sl_fname.as_cstr());
        semsg!("E778: This does not look like a .sug file: {sl_fname}");
        return;
    }
    let version = spl.getc().map_or(-1, c_int::from);
    if version < VIMSUGVERSION {
        let sl_fname = msg_cstr(slang.sl_fname.as_cstr());
        semsg!("E779: Old .sug file, needs to be updated: {sl_fname}");
        return;
    }
    if version > VIMSUGVERSION {
        let sl_fname = msg_cstr(slang.sl_fname.as_cstr());
        semsg!("E780: .sug file is for newer version of Vim: {sl_fname}");
        return;
    }
    // The `.spl` stamped both files; a mismatch means the pair is
    // stale and the word numbers would point at the wrong words.
    if spl.get8ctime().unwrap_or(-1) != slang.sl_sugtime {
        let sl_fname = msg_cstr(slang.sl_fname.as_cstr());
        semsg!("E781: .sug file doesn't match .spl file: {sl_fname}");
        return;
    }

    // SAFETY: the language is live.
    if read_sug_body(spl, slang).is_ok() {
        return;
    }
    let fname = msg_cstr(slang.sl_fname.as_cstr());
    semsg!("E782: Error while reading .sug file: {fname}");
    slang_clear_sug(slang);
}

/// The sound-fold tree and the word-number lines behind it.
fn read_sug_body(spl: &mut Spl, slang: &mut SpellLang) -> SplResult<()> {
    spell_read_tree(spl, &mut slang.sl_sound_tree, false, 0)?;

    let sugbuf = slang.sl_sugbuf.insert(open_spellbuf()).buf();
    let wcount = spl.get4c()?;
    if wcount < 0 {
        return Err(SpellReadError::Format);
    }

    // One line per word end, each a NUL-terminated run of encoded word
    // numbers. They go into a scratch buffer so the suggestion search
    // can index them by line.
    let mut line: Vec<u8> = Vec::with_capacity(100);
    for wordnr in 0..wcount {
        line.clear();
        loop {
            let c = spl.byte()?;
            line.push(c);
            if c == NUL as u8 {
                break;
            }
        }
        let at = wordnr as LineNr;
        let appended = ml_append_buf_terminated(sugbuf, at, &line, true);
        if appended.is_err() {
            return Err(SpellReadError::Other);
        }
    }

    // Both trees get their word counts filled in, which is what turns
    // a position in a tree into a word number.
    tree_count_words(&mut slang.sl_fold_tree);
    tree_count_words(&mut slang.sl_sound_tree);
    Ok(())
}

/// Replace each word end's index with the number of words in its sub-tree.
///
/// That is what later turns a position in a tree into a word number, which
/// is how a `.sug` file names the words a sound-folded form stands for.
///
/// # Depth
///
/// The stacks are one longer than [`MAXWLEN`] on purpose. [`read_tree_node`]
/// rejects a tree nested deeper than `MAXWLEN`, which still admits a node
/// *at* depth `MAXWLEN`; sizing these to `MAXWLEN` alone left the deepest
/// such tree writing one past the end of all three.
fn tree_count_words(tree: &mut WordTree) {
    if tree.is_empty() {
        return;
    }
    let mut arridx = [0usize; MAXWLEN + 1];
    let mut curi = [0usize; MAXWLEN + 1];
    let mut wordcount = [0 as SpellIdx; MAXWLEN + 1];

    let mut depth: usize = 0;
    curi[0] = 1;
    loop {
        if got_int.get() {
            break;
        }
        if curi[depth] > tree.node_len(arridx[depth]) {
            // Everything at this node is counted; publish the total
            // and hand it to the parent.
            let at = arridx[depth];
            let total = wordcount[depth];
            tree.idxs_mut()[at] = total;
            let at_root = depth == 0;
            if !at_root {
                wordcount[depth - 1] += total;
                depth -= 1;
            }
            fast_breakcheck();
            if at_root {
                break;
            }
            continue;
        }

        let mut n = arridx[depth] + curi[depth];
        curi[depth] += 1;
        if !tree.ends_word(n) {
            depth += 1;
            arridx[depth] = tree.child_node(n);
            curi[depth] = 1;
            wordcount[depth] = 0;
            continue;
        }

        wordcount[depth] += 1;
        // The same word can end several times over, once per flag
        // set; that is still one word.
        while n + 1 < tree.len() && tree.ends_word(n + 1) {
            n += 1;
            curi[depth] += 1;
        }
    }
}

/// Read one whole tree: its length, then its nodes.
///
/// `out` receives the tree, or is left as it was when the file says the
/// language has none.
fn spell_read_tree(
    spl: &mut Spl,
    out: &mut WordTree,
    prefixtree: bool,
    prefixcnt: c_int,
) -> SplResult<()> {
    let len = spl.get4c()?;
    if len < 0 {
        return Err(SpellReadError::Trunc);
    }
    let len_usize = len as usize;
    // The index array is `len` ints; refuse a length that could not be
    // allocated rather than wrapping the multiplication.
    if len_usize > usize::MAX / size_of::<c_int>() {
        return Err(SpellReadError::Format);
    }
    if len == 0 {
        return Ok(());
    }

    // Both arrays are `len` entries and nothing else reaches them until
    // the tree is read, so every index a corrupt file names is checked by
    // Rust rather than by review.
    let mut byts = vec![0u8; len_usize].into_boxed_slice();
    let mut idxs = vec![0 as SpellIdx; len_usize].into_boxed_slice();
    let idx = read_tree_node(spl, &mut byts, &mut idxs, 0, prefixtree, prefixcnt, 0)?;
    // Every byte of the array has to be accounted for; anything else
    // means the node lengths and the tree length disagree.
    if idx != len {
        return Err(SpellReadError::Format);
    }
    *out = WordTree::from_parts(byts, idxs);
    Ok(())
}

/// Read one node — a length byte and that many entries — and recurse into
/// the children. Returns the next free index.
///
/// The two arrays arrive as slices, so every index a corrupt file could
/// name is bounds-checked by Rust rather than by review; the explicit
/// `startidx + len >= byts.len()` test below is kept because it must answer
/// a format error rather than panic. Nesting is capped at [`MAXWLEN`], so
/// the tree cannot be made to nest without end either.
///
/// The reads that spell `unwrap_or(-1)` are the ones upstream leaves
/// unchecked: the value flows into a range test that rejects `-1` on its
/// own, and turning the end of the file into an error here would answer
/// `E758` where the suite pins `E759`.
fn read_tree_node(
    spl: &mut Spl,
    byts: &mut [uint8_t],
    idxs: &mut [SpellIdx],
    startidx: SpellIdx,
    prefixtree: bool,
    maxprefcondnr: c_int,
    depth: c_int,
) -> SplResult<SpellIdx> {
    let maxidx = byts.len() as c_int;
    let mut idx = startidx;
    if depth > MAXWLEN as c_int {
        return Err(SpellReadError::Format);
    }
    let len = spl.getc().map_or(-1, c_int::from);
    if len <= 0 {
        return Err(SpellReadError::Trunc);
    }
    if i64::from(startidx) + i64::from(len) >= i64::from(maxidx) {
        return Err(SpellReadError::Format);
    }

    byts[idx as usize] = len as uint8_t;
    idx += 1;

    for _ in 1..=len {
        let mut c = spl.getc().map_or(-1, c_int::from);
        if c < 0 {
            return Err(SpellReadError::Trunc);
        }
        if c <= BY_SPECIAL as c_int {
            if c == BY_NOFLAGS as c_int && !prefixtree {
                // A word end with nothing to say about it.
                idxs[idx as usize] = 0;
            } else if c != BY_INDEX as c_int {
                // A word end carrying flags; how many bytes follow
                // depends on which flags are set.
                if prefixtree {
                    c = if c == BY_FLAGS as c_int {
                        spl.getc().map_or(-1, c_int::from) << 24
                    } else {
                        0
                    };
                    c |= spl.getc().map_or(-1, c_int::from);
                    let n = spl.get2c().unwrap_or(-1);
                    if n >= maxprefcondnr {
                        return Err(SpellReadError::Format);
                    }
                    c |= n << 8;
                } else {
                    let kind = c;
                    c = spl.getc().map_or(-1, c_int::from);
                    if kind == BY_FLAGS2 as c_int {
                        c += spl.getc().map_or(-1, c_int::from) << 8;
                    }
                    if WordFlags::from_bits(c).has(WordFlags::REGION) {
                        c += spl.getc().map_or(-1, c_int::from) << 16;
                    }
                    if WordFlags::from_bits(c).has(WordFlags::AFX) {
                        c += spl.getc().map_or(-1, c_int::from) << 24;
                    }
                }
                idxs[idx as usize] = c as SpellIdx;
                c = 0;
            } else {
                // A reference to a sub-tree written earlier.
                let n = spl.get3c().unwrap_or(-1);
                if n < 0 || n >= maxidx {
                    return Err(SpellReadError::Format);
                }
                idxs[idx as usize] = n.wrapping_add(SHARED_MASK) as SpellIdx;
                c = spl.getc().map_or(-1, c_int::from);
            }
        }
        byts[idx as usize] = c as uint8_t;
        idx += 1;
    }

    // Second pass: children follow the whole node, so their indices
    // are only known now.
    for i in 1..=len {
        let at = (startidx + i) as usize;
        if byts[at] == 0 {
            continue;
        }
        if idxs[at] & SHARED_MASK != 0 {
            idxs[at] &= !SHARED_MASK;
            continue;
        }
        idxs[at] = idx;
        let deeper = depth + 1;
        idx = read_tree_node(spl, byts, idxs, idx, prefixtree, maxprefcondnr, deeper)?;
    }
    Ok(idx)
}

/// Re-read every loaded language that came from `fname`, and redraw the
/// windows that were spell-checking with it.
pub(super) fn spell_reload_one(fname: &CStr, added_word: bool) {
    let mut didit = false;
    for id in languages() {
        let same = with_lang(id, |slang| {
            path_full_compare(fname, slang.sl_fname.as_cstr(), false, true) as c_uint
                == kEqualFiles as c_uint
        });
        if same != Some(true) {
            continue;
        }
        // Reload in its slot, so every window naming this language keeps
        // naming it; out of the slot while the file is read.
        let Some(mut slang) = take_lang(id) else {
            continue;
        };
        slang_clear(&mut slang);
        if !spell_reload_file(fname, &mut slang) {
            // Leave it empty rather than half-read.
            slang_clear(&mut slang);
        }
        put_lang(id, slang);
        redraw_all_later(UPD_SOME_VALID);
        didit = true;
    }

    // A word was added to a file no window had loaded; re-resolving
    // 'spelllang' is what picks it up.
    if added_word && !didit {
        let _ = parse_spelllang(Win::current());
    }
}
