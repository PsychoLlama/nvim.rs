//! Reading a `.spl` file, and the `.sug` beside it.
//!
//! This is a parser over bytes nobody vouched for: a `.spl` is just a file
//! on `'runtimepath'`, and every length, count and index in it is whatever
//! the file says. Each reader therefore returns one of the `SP_*` codes
//! rather than trusting what it read:
//!
//! - [`SP_TRUNCERROR`] — the file ended early.
//! - [`SP_FORMERROR`] — the bytes are there but do not make sense.
//! - [`SP_OTHERERROR`] — the read itself failed.
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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::semsg;
use crate::smsg;
use crate::spell::{WordFlags, WordTree};
use core::ffi::{c_char, c_int, c_uint};

use crate::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::fileio::{get2c, get3c, get4c, get8ctime, read_string};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::main::{curwin, got_int, p_verbose};
use crate::memline::ml_append_buf;
use crate::memory::{xfree, xstrdup};
use crate::message::{emsg, verbose_enter, verbose_leave};
use crate::message_fmt::c_str;
use crate::os::cshim::{getc, gettext, gettext_ptr, strstr};
use crate::os::fs::os_fopen;
use crate::os::input::fast_breakcheck;
use crate::path::{path_fnamecmp, path_full_compare, path_tail};
use crate::runtime::{estack_pop, estack_push};
use crate::spell::{
    e_format, first_lang, init_syl_tab, open_spellbuf, parse_spelllang, slang_alloc, slang_clear,
    slang_clear_sug, slang_free,
};
use crate::types::{
    FILE, NUL, OptInt, colnr_T, garray_T, idx_T, langp_T, linenr_T, size_t, slang_T, time_t,
    uint8_t,
};
use ::libc::{fclose, feof, ferror, fread, strcpy, strerror, strrchr};

use super::sections::{
    read_charflags_section, read_compound, read_prefcond_section, read_region_section,
    read_rep_section, read_sal_section, read_sofo_section, read_words_section, set_map_str,
};
use super::{
    BY_FLAGS, BY_FLAGS2, BY_INDEX, BY_NOFLAGS, BY_SPECIAL, ETYPE_SPELL, MAXWLEN, OK, SN_CHARFLAGS,
    SN_COMPOUND, SN_END, SN_INFO, SN_MAP, SN_MIDWORD, SN_NOBREAK, SN_NOCOMPOUNDSUGS,
    SN_NOSPLITSUGS, SN_PREFCOND, SN_REGION, SN_REP, SN_REPSAL, SN_SAL, SN_SOFO, SN_SUGFILE,
    SN_SYLLABLE, SN_WORDS, SNF_REQUIRED, SP_FORMERROR, SP_OTHERERROR, SP_TRUNCERROR, SPL_FNAME_ADD,
    VIMSPELLMAGIC, VIMSPELLMAGICL, VIMSPELLVERSION, VIMSUGMAGIC, VIMSUGMAGICL, VIMSUGVERSION,
    e_spell_trunc, kEqualFiles,
};

/// Marks a tree index that already points at a shared sub-tree, so the
/// second pass over a node's bytes knows not to descend into it again.
pub(super) const SHARED_MASK: c_int = 0x8000000;

/// Read exactly `n` bytes, or say why not.
///
/// # Safety
///
/// `buf` must have room for `n` bytes.
pub(super) unsafe fn read_bytes(fd: *mut FILE, buf: *mut c_char, n: usize) -> Result<(), c_int> {
    // SAFETY: the caller promises the buffer; `fd` is open.
    if unsafe { fread(buf.cast(), 1, n, fd) } as usize == n {
        return Ok(());
    }
    Err(if unsafe { feof(fd) } != 0 {
        SP_TRUNCERROR
    } else {
        SP_OTHERERROR
    })
}

/// As [`read_bytes`], and reject an embedded NUL.
///
/// Several sections hold text that later gets treated as a C string, so a
/// NUL inside would silently truncate it; the file is malformed instead.
///
/// # Safety
///
/// `buf` must have room for `n` bytes.
pub(super) unsafe fn read_nonnul_bytes(
    fd: *mut FILE,
    buf: *mut c_char,
    n: usize,
) -> Result<(), c_int> {
    // SAFETY: as above.
    unsafe { read_bytes(fd, buf, n) }?;
    if unsafe { core::slice::from_raw_parts(buf.cast::<u8>(), n) }.contains(&0) {
        return Err(SP_FORMERROR);
    }
    Ok(())
}

/// Check the eight magic bytes a `.spl` starts with.
///
/// # Safety
///
/// `fd` must be open for reading.
unsafe fn spell_check_magic_string(fd: *mut FILE) -> c_int {
    // SAFETY: `fd` is open; `buf` is the size the read asks for.
    let mut buf: [c_char; VIMSPELLMAGICL] = [0; VIMSPELLMAGICL];
    if let Err(e) = unsafe { read_bytes(fd, buf.as_mut_ptr(), VIMSPELLMAGICL) } {
        return e;
    }
    if cstr::as_bytes(&buf) != VIMSPELLMAGIC.to_bytes() {
        return SP_FORMERROR;
    }
    0
}

/// What reading one section decided about the rest of the file.
enum Section {
    /// Carry on; the code is `0` or an `SP_*` the caller reports.
    Code(c_int),
    /// The file ended in the middle of a section being skipped.
    Truncated,
    /// Stop reading this file; the reason has already been reported.
    Failed,
}

/// Load a spell file, returning the language it describes or null.
///
/// With `old_lp` given, that language is refilled in place — this is how a
/// `.add` file is folded into the language it extends. Otherwise a fresh
/// one is allocated, and registered in the global list when `lang` names
/// it.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path; `lang`, when given, must point at
/// a writable language name.
pub unsafe fn spell_load_file(
    fname: *mut c_char,
    lang: *mut c_char,
    old_lp: *mut slang_T,
    silent: bool,
) -> *mut slang_T {
    // SAFETY: the caller promises the path and the language buffer.
    let fd = unsafe { os_fopen(fname, c"r".as_ptr()) };
    let mut lp: *mut slang_T = core::ptr::null_mut();
    let mut did_estack_push = false;

    let (out, pushed) = (&mut lp, &mut did_estack_push);
    let ok = unsafe { load_spl(fd, fname, lang, old_lp, silent, out, pushed) };
    if !ok {
        // The language name is cleared even when the file could not be
        // opened, so the caller stops trying this name.
        if !lang.is_null() {
            unsafe { *lang = NUL as c_char };
        }
        if !lp.is_null() && old_lp.is_null() {
            unsafe { slang_free(lp) };
        }
        lp = core::ptr::null_mut();
    }

    if !fd.is_null() {
        unsafe { fclose(fd) };
    }
    if did_estack_push {
        estack_pop();
    }
    lp
}

/// The body of [`spell_load_file`]: true when the language is usable.
///
/// # Safety
///
/// As [`spell_load_file`].
unsafe fn load_spl(
    fd: *mut FILE,
    fname: *mut c_char,
    lang: *mut c_char,
    old_lp: *mut slang_T,
    silent: bool,
    lpp: &mut *mut slang_T,
    did_estack_push: &mut bool,
) -> bool {
    // SAFETY: as the caller's contract.
    if fd.is_null() {
        if !silent {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = unsafe { c_str(fname) };
            semsg!("E484: Can't open file {fname}");
        } else if p_verbose.get() > 2 as OptInt {
            unsafe { verbose_enter() };
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = unsafe { c_str(fname) };
            smsg!(0, "E484: Can't open file {fname}");
            unsafe { verbose_leave() };
        }
        return false;
    }
    if p_verbose.get() > 2 as OptInt {
        unsafe { verbose_enter() };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        smsg!(0, "Reading spell file \"{fname}\"");
        unsafe { verbose_leave() };
    }

    let lp = if old_lp.is_null() {
        let lp = unsafe { slang_alloc(lang) };
        unsafe { (*lp).sl_fname = xstrdup(fname) };
        // ".add.spl" files add to an existing language rather than
        // defining one.
        unsafe { (*lp).sl_add = !strstr(path_tail(fname), SPL_FNAME_ADD.as_ptr()).is_null() };
        lp
    } else {
        old_lp
    };
    *lpp = lp;

    estack_push(ETYPE_SPELL, fname, 0 as linenr_T);
    *did_estack_push = true;

    match unsafe { spell_check_magic_string(fd) } {
        SP_FORMERROR | SP_TRUNCERROR => {
            let fmt = gettext(c"E757: This does not look like a spell file");
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(fmt.as_ptr()) };
            semsg!("{arg0}");
            return false;
        }
        SP_OTHERERROR => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (fname, arg1) = unsafe { (c_str(fname), c_str(strerror(ferror(fd)))) };
            semsg!("E5042: Failed to read spell file {fname}: {arg1}");
            return false;
        }
        _ => {}
    }

    let version = unsafe { getc(fd) };
    if version < VIMSPELLVERSION {
        let text = c"E771: Old spell file, needs to be updated";
        emsg(gettext(text));
        return false;
    }
    if version > VIMSPELLVERSION {
        let text = c"E772: Spell file is for newer version of Vim";
        emsg(gettext(text));
        return false;
    }

    loop {
        let id = unsafe { getc(fd) };
        let mut res = 0;

        if id == SN_END as c_int {
            res = unsafe { read_trees(fd, lp) };
            if res == 0 {
                if old_lp.is_null() && !lang.is_null() {
                    unsafe { (*lp).sl_next = first_lang.get() };
                    first_lang.set(lp);
                }
                return true;
            }
        } else {
            let flags = unsafe { getc(fd) };
            let len = unsafe { get4c(fd) };
            if len < 0 {
                break;
            }
            match unsafe { read_section(fd, lp, id, flags, len) } {
                Section::Code(c) => res = c,
                Section::Truncated => break,
                Section::Failed => return false,
            }
        }

        if res == SP_FORMERROR {
            unsafe { emsg(gettext_ptr(e_format.get())) };
            return false;
        }
        if res == SP_TRUNCERROR {
            break;
        }
        if res == SP_OTHERERROR {
            return false;
        }
    }

    unsafe { emsg(gettext_ptr(e_spell_trunc.get())) };
    false
}

/// Dispatch one section by its id.
///
/// # Safety
///
/// `fd` must be positioned at the section's payload and `lp` be live.
unsafe fn read_section(
    fd: *mut FILE,
    lp: *mut slang_T,
    id: c_int,
    flags: c_int,
    mut len: c_int,
) -> Section {
    // SAFETY: the caller promises the file position and the language.
    let code = match id as u32 {
        SN_INFO => {
            unsafe { xfree((*lp).sl_info.cast()) };
            unsafe { (*lp).sl_info = read_string(fd, len as size_t) };
            if unsafe { (*lp).sl_info }.is_null() {
                return Section::Failed;
            }
            0
        }
        SN_REGION => unsafe { read_region_section(fd, lp, len) },
        SN_CHARFLAGS => unsafe { read_charflags_section(fd) },
        SN_MIDWORD => {
            unsafe { (*lp).sl_midword = read_string(fd, len as size_t) };
            if unsafe { (*lp).sl_midword }.is_null() {
                return Section::Failed;
            }
            0
        }
        SN_PREFCOND => unsafe { read_prefcond_section(fd, lp) },
        SN_REP => unsafe { read_rep_section(fd, &mut (*lp).sl_rep, &mut (*lp).sl_rep_first) },
        SN_REPSAL => unsafe {
            read_rep_section(fd, &mut (*lp).sl_repsal, &mut (*lp).sl_repsal_first)
        },
        SN_SAL => unsafe { read_sal_section(fd, lp) },
        SN_SOFO => unsafe { read_sofo_section(fd, lp) },
        SN_MAP => {
            let p = unsafe { read_string(fd, len as size_t) };
            if p.is_null() {
                return Section::Failed;
            }
            unsafe { set_map_str(lp, p) };
            unsafe { xfree(p.cast()) };
            0
        }
        SN_WORDS => unsafe { read_words_section(fd, lp, len) },
        SN_SUGFILE => {
            unsafe { (*lp).sl_sugtime = get8ctime(fd) };
            0
        }
        SN_NOSPLITSUGS => {
            unsafe { (*lp).sl_nosplitsugs = true };
            0
        }
        SN_NOCOMPOUNDSUGS => {
            unsafe { (*lp).sl_nocompoundsugs = true };
            0
        }
        SN_COMPOUND => unsafe { read_compound(fd, lp, len) },
        SN_NOBREAK => {
            unsafe { (*lp).sl_nobreak = true };
            0
        }
        SN_SYLLABLE => {
            unsafe { (*lp).sl_syllable = read_string(fd, len as size_t) };
            if unsafe { (*lp).sl_syllable }.is_null() || unsafe { init_syl_tab(lp) } != OK {
                return Section::Failed;
            }
            0
        }
        _ => {
            // An unknown section is only fatal when the file insists
            // it is understood; otherwise step over it.
            if flags & SNF_REQUIRED != 0 {
                emsg(gettext(c"E770: Unsupported section in spell file"));
                return Section::Failed;
            }
            loop {
                len -= 1;
                if len < 0 {
                    break;
                }
                if unsafe { getc(fd) } < 0 {
                    return Section::Truncated;
                }
            }
            0
        }
    };
    Section::Code(code)
}

/// Read the three trees that close a `.spl`: case-folded words, keep-case
/// words, and prefixes.
///
/// # Safety
///
/// `fd` must be positioned at the first tree and `lp` be live.
unsafe fn read_trees(fd: *mut FILE, lp: *mut slang_T) -> c_int {
    // SAFETY: the caller promises the position and the language.
    let res = unsafe { spell_read_tree(fd, &mut (*lp).sl_fold_tree, false, 0) };
    if res != 0 {
        return res;
    }
    let res = unsafe { spell_read_tree(fd, &mut (*lp).sl_keep_tree, false, 0) };
    if res != 0 {
        return res;
    }
    // The prefix tree's entries name a prefix condition by number, so
    // it can only be read once SN_PREFCOND has said how many there are.
    let conds = unsafe { (*lp).sl_prefixcnt };
    unsafe { spell_read_tree(fd, &mut (*lp).sl_prefix_tree, true, conds) }
}

/// Load the `.sug` file for every language of the current window that has
/// one and has not tried yet.
///
/// A `.sug` is optional and best-effort: anything wrong with it is
/// reported and the language carries on without sound-a-like suggestions.
pub unsafe fn suggest_load_files() {
    // SAFETY: `b_langp` holds `ga_len` live `langp_T`s.
    let langp = unsafe { (*(*curwin.get()).w_s).b_langp };
    for lpi in 0..langp.ga_len {
        let lp = unsafe { langp.ga_data.cast::<langp_T>().offset(lpi as isize) };
        let slang = unsafe { (*lp).lp_slang };
        if unsafe { (*slang).sl_sugtime } == 0 as time_t || unsafe { (*slang).sl_sugloaded } {
            continue;
        }
        // One attempt per language, successful or not.
        unsafe { (*slang).sl_sugloaded = true };

        // The `.sug` sits beside the `.spl` under the same stem; the
        // name is edited in place and put back afterwards.
        let dotp = unsafe { strrchr((*slang).sl_fname, b'.' as c_int) };
        if dotp.is_null() || unsafe { path_fnamecmp(dotp, c".spl".as_ptr()) } != 0 {
            continue;
        }
        unsafe { strcpy(dotp, c".sug".as_ptr().cast_mut()) };

        let fd = unsafe { os_fopen((*slang).sl_fname, c"r".as_ptr()) };
        if !fd.is_null() {
            unsafe { load_sug(fd, slang) };
            unsafe { fclose(fd) };
        }
        unsafe { strcpy(dotp, c".spl".as_ptr().cast_mut()) };
    }
}

/// Read one `.sug` file into `slang`.
///
/// # Safety
///
/// `fd` must be open at the start of the file and `slang` be live.
unsafe fn load_sug(fd: *mut FILE, slang: *mut slang_T) {
    // SAFETY: the caller promises the file and the language.
    let mut buf: [c_char; MAXWLEN] = [0; MAXWLEN];
    for b in buf.iter_mut().take(VIMSUGMAGICL as usize) {
        *b = unsafe { getc(fd) } as c_char;
    }
    if !(unsafe { cstr::prefix_eq(buf.as_ptr(), VIMSUGMAGIC.as_ptr(), VIMSUGMAGICL as size_t) }) {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let sl_fname = unsafe { c_str((*slang).sl_fname) };
        semsg!("E778: This does not look like a .sug file: {sl_fname}");
        return;
    }
    let version = unsafe { getc(fd) };
    if version < VIMSUGVERSION {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let sl_fname = unsafe { c_str((*slang).sl_fname) };
        semsg!("E779: Old .sug file, needs to be updated: {sl_fname}");
        return;
    }
    if version > VIMSUGVERSION {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let sl_fname = unsafe { c_str((*slang).sl_fname) };
        semsg!("E780: .sug file is for newer version of Vim: {sl_fname}");
        return;
    }
    // The `.spl` stamped both files; a mismatch means the pair is
    // stale and the word numbers would point at the wrong words.
    if unsafe { get8ctime(fd) } != unsafe { (*slang).sl_sugtime } {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let sl_fname = unsafe { c_str((*slang).sl_fname) };
        semsg!("E781: .sug file doesn't match .spl file: {sl_fname}");
        return;
    }

    if unsafe { read_sug_body(fd, slang) } {
        return;
    }
    // SAFETY: the loaded language's own file name.
    let fname = unsafe { c_str((*slang).sl_fname) };
    semsg!("E782: Error while reading .sug file: {fname}");
    unsafe { slang_clear_sug(slang) };
}

/// The sound-fold tree and the word-number lines behind it.
///
/// # Safety
///
/// `fd` must be positioned just past the `.sug` header.
unsafe fn read_sug_body(fd: *mut FILE, slang: *mut slang_T) -> bool {
    // SAFETY: the caller promises the position and the language.
    if unsafe { spell_read_tree(fd, &mut (*slang).sl_sound_tree, false, 0) } != 0 {
        return false;
    }

    unsafe { (*slang).sl_sugbuf = open_spellbuf() };
    let wcount = unsafe { get4c(fd) };
    if wcount < 0 {
        return false;
    }

    // One line per word end, each a NUL-terminated run of encoded word
    // numbers. They go into a scratch buffer so the suggestion search
    // can index them by line.
    let mut ga: garray_T = unsafe { core::mem::zeroed() };
    unsafe { ga_init(&raw mut ga, 1, 100) };
    let mut ok = true;
    for wordnr in 0..wcount {
        ga.ga_len = 0;
        loop {
            let c = unsafe { getc(fd) };
            if c < 0 {
                ok = false;
                break;
            }
            unsafe { ga_grow(&raw mut ga, 1) };
            unsafe { *ga.ga_data.cast::<uint8_t>().offset(ga.ga_len as isize) = c as uint8_t };
            ga.ga_len += 1;
            if c == NUL {
                break;
            }
        }
        let sugbuf = unsafe { (*slang).sl_sugbuf };
        let (line, len) = (ga.ga_data.cast::<c_char>(), ga.ga_len as colnr_T);
        let at = wordnr as linenr_T;
        if !ok || unsafe { ml_append_buf(sugbuf, at, line, len, true) }.is_err() {
            ok = false;
            break;
        }
    }
    unsafe { ga_clear(&raw mut ga) };
    if !ok {
        return false;
    }

    // Both trees get their word counts filled in, which is what turns
    // a position in a tree into a word number.
    // SAFETY: the caller's language.
    unsafe {
        tree_count_words(&mut (*slang).sl_fold_tree);
        tree_count_words(&mut (*slang).sl_sound_tree);
    }
    true
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
    let mut wordcount = [0 as idx_T; MAXWLEN + 1];

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
            arridx[depth] = usize::try_from(tree.idx(n)).unwrap_or(0);
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
/// `out` receives the tree, or is left empty when the file says the
/// language has none.
///
/// # Safety
///
/// `fd` must be positioned at a tree.
unsafe fn spell_read_tree(
    fd: *mut FILE,
    out: &mut WordTree,
    prefixtree: bool,
    prefixcnt: c_int,
) -> c_int {
    // SAFETY: the caller promises the position.
    let len = unsafe { get4c(fd) };
    if len < 0 {
        return SP_TRUNCERROR;
    }
    let Ok(len_usize) = usize::try_from(len) else {
        return SP_FORMERROR;
    };
    // The index array is `len` ints; refuse a length that could not be
    // allocated rather than wrapping the multiplication.
    if len_usize > usize::MAX / size_of::<c_int>() {
        return SP_FORMERROR;
    }
    if len == 0 {
        return 0;
    }

    // Both arrays are `len` entries and nothing else reaches them until
    // the tree is read, so every index a corrupt file names is checked by
    // Rust rather than by review.
    let mut byts = vec![0u8; len_usize].into_boxed_slice();
    let mut idxs = vec![0 as idx_T; len_usize].into_boxed_slice();
    let idx = unsafe { read_tree_node(fd, &mut byts, &mut idxs, 0, prefixtree, prefixcnt, 0) };
    if idx < 0 {
        return idx;
    }
    // Every byte of the array has to be accounted for; anything else
    // means the node lengths and the tree length disagree.
    if idx != len {
        return SP_FORMERROR;
    }
    *out = WordTree::from_parts(byts, idxs);
    0
}

/// Read one node — a length byte and that many entries — and recurse into
/// the children. Returns the next free index, or a negative `SP_*`.
///
/// The two arrays arrive as slices, so every index a corrupt file could
/// name is bounds-checked by Rust rather than by review; the explicit
/// `startidx + len >= byts.len()` test below is kept because it must answer
/// `SP_FORMERROR` rather than panic. Nesting is capped at [`MAXWLEN`], so
/// the tree cannot be made to nest without end either.
///
/// # Safety
///
/// `fd` must be open and positioned at a node.
unsafe fn read_tree_node(
    fd: *mut FILE,
    byts: &mut [uint8_t],
    idxs: &mut [idx_T],
    startidx: idx_T,
    prefixtree: bool,
    maxprefcondnr: c_int,
    depth: c_int,
) -> idx_T {
    // SAFETY: the caller promises the stream; the arrays are checked.
    let maxidx = byts.len() as c_int;
    let mut idx = startidx;
    if depth > MAXWLEN as c_int {
        return SP_FORMERROR;
    }
    let len = unsafe { getc(fd) };
    if len <= 0 {
        return SP_TRUNCERROR;
    }
    if startidx as i64 + len as i64 >= maxidx as i64 {
        return SP_FORMERROR;
    }

    byts[idx as usize] = len as uint8_t;
    idx += 1;

    for _ in 1..=len {
        let mut c = unsafe { getc(fd) };
        if c < 0 {
            return SP_TRUNCERROR;
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
                        (unsafe { getc(fd) }) << 24
                    } else {
                        0
                    };
                    c |= unsafe { getc(fd) };
                    let n = unsafe { get2c(fd) };
                    if n >= maxprefcondnr {
                        return SP_FORMERROR;
                    }
                    c |= n << 8;
                } else {
                    let kind = c;
                    c = unsafe { getc(fd) };
                    if kind == BY_FLAGS2 as c_int {
                        c += unsafe { getc(fd) } << 8;
                    }
                    if WordFlags::from_bits(c).has(WordFlags::REGION) {
                        c += unsafe { getc(fd) } << 16;
                    }
                    if WordFlags::from_bits(c).has(WordFlags::AFX) {
                        c += unsafe { getc(fd) } << 24;
                    }
                }
                idxs[idx as usize] = c as idx_T;
                c = 0;
            } else {
                // A reference to a sub-tree written earlier.
                let n = unsafe { get3c(fd) };
                if n < 0 || n >= maxidx {
                    return SP_FORMERROR;
                }
                idxs[idx as usize] = n.wrapping_add(SHARED_MASK) as idx_T;
                c = unsafe { getc(fd) };
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
        idx = unsafe { read_tree_node(fd, byts, idxs, idx, prefixtree, maxprefcondnr, deeper) };
        if idx < 0 {
            break;
        }
    }
    idx
}

/// Re-read every loaded language that came from `fname`, and redraw the
/// windows that were spell-checking with it.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path.
pub(super) unsafe fn spell_reload_one(fname: *mut c_char, added_word: bool) {
    // SAFETY: the caller promises the path; the language list is global
    // and only walked here.
    let mut didit = false;
    let mut slang = first_lang.get();
    while !slang.is_null() {
        if unsafe { path_full_compare(fname, (*slang).sl_fname, false, true) } as c_uint
            == kEqualFiles as c_uint
        {
            unsafe { slang_clear(slang) };
            // Reload in place, so every window pointing at this
            // language keeps its pointer.
            if unsafe { spell_load_file(fname, core::ptr::null_mut(), slang, false) }.is_null() {
                // Leave it empty rather than half-read.
                unsafe { slang_clear(slang) };
            }
            unsafe { redraw_all_later(UPD_SOME_VALID) };
            didit = true;
        }
        slang = unsafe { (*slang).sl_next };
    }

    // A word was added to a file no window had loaded; re-resolving
    // 'spelllang' is what picks it up.
    if added_word && !didit {
        unsafe { parse_spelllang(curwin.get()) };
    }
}
