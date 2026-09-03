//! The `.spl` file format, from both ends.
//!
//! Nothing in the tree asserted a `.spl` byte before this file: the old
//! suite's `mkspell` cases read the result back through the editor, so a
//! writer and a reader that were wrong in the same way agreed with each
//! other. These cases pin the bytes.
//!
//! # The golden
//!
//! `tests/fixtures/spell/one-utf8.spl` is 424 bytes and is what upstream's
//! `Test_spellfile_verbose` builds: a `.dic` whose first line is the word
//! count (`1`) and whose only word is `one`, and an empty `.aff`. It is
//! reproducible because the one timestamp the format carries is written by
//! `put_sugfile`, which an `.aff` without `SAL`/`SOFOFROM`/`SOFOTO` never
//! reaches. Regenerate it with
//!
//! ```text
//! printf '1\none\n' > Xdet.dic && : > Xdet.aff
//! nvim --clean -es -c 'set mkspellmem=460000,2000,500' \
//!      -c 'mkspell! one-utf8.spl Xdet' -c q
//! ```
//!
//! `'mkspellmem'` has to be pinned because it is what decides when the word
//! tree is compressed, and a tree compressed at a different point is a
//! different set of shared sub-trees — the same words, different bytes.
//! The default *value* is already `460000,2000,500`, but the limits it names
//! are installed by `didset_options()`, which a unit test never reaches, so
//! [`spell_check_msm`] is called here for the same reason `:set` calls it.
//!
//! # The `.sug` pair
//!
//! `sal-utf8.spl` and `sal-utf8.sug` are the same dictionary with an `.aff`
//! carrying two `SAL` rules, which is the smallest input that produces a
//! `.sug` file. Those two are *not* reproducible — both carry the same
//! `time(NULL)` — so the checked-in copies have those eight bytes zeroed
//! and the case masks the same eight before comparing, after asserting the
//! two live values are equal (which is the cross-check the reader makes).
//! Regenerate them with the recipe above over `SAL n n` / `SAL e _`, then
//! zero `SN_SUGFILE`'s payload in the `.spl` and bytes 7..15 of the `.sug`.
//!
//! # What the goldens do not pin
//!
//! One region, one encoding, no affixes, no `REP`/`MAP`/`COMMON` section,
//! and a word tree too small to compress. The sections they carry are
//! `SN_CHARFLAGS`, `SN_SAL`, `SN_SUGFILE` and the three trees; every other
//! section id is covered only by the read side, against
//! `runtime/spell/en.utf-8.spl`.

#![cfg(not(miri))]

use std::ffi::{CString, c_char, c_int};

use neovim::garray::{ga_clear, ga_grow, ga_init};
use neovim::main::curwin;
use neovim::spell::{REGION_ALL, init_spell_chartab, spell_check};
use neovim::spellfile::{mkspell, spell_check_msm, spell_load_file};
use neovim::types::{langp_T, slang_T};

use crate::support::{Sandbox, cstr};

/// The `.spl` every case below is written against.
const GOLDEN: &[u8] = include_bytes!("../fixtures/spell/one-utf8.spl");

/// The word the golden's dictionary holds. Its `.dic` reads `1\none\n`:
/// the first line of a Hunspell dictionary is the word count, not a word.
const WORD: &str = "one";

/// The language the tree ships, read by the cases that need a real file
/// with every section in it.
const SHIPPED: &str = "runtime/spell/en.utf-8.spl";

/// The header of a `.spl`, and the id and length of each section, as far as
/// the bytes are readable. Only ever used to describe a mismatch, so it
/// stops at the first thing it cannot make sense of rather than failing.
///
/// A byte-for-byte `assert_eq!` over 424 bytes prints two hex dumps and
/// leaves the reader to find the difference; this says which *section*
/// moved, which is the question a format change actually raises.
fn describe(spl: &[u8]) -> String {
    let mut out = String::new();
    if spl.len() < 9 || &spl[..8] != b"VIMspell" {
        return format!("{} bytes, no VIMspell magic", spl.len());
    }
    out.push_str(&format!("VIMspell version {}", spl[8]));
    let mut at = 9;
    loop {
        let Some(&id) = spl.get(at) else {
            out.push_str(", truncated in the section list");
            return out;
        };
        if id == 255 {
            out.push_str(&format!(", SN_END at {at}"));
            break;
        }
        let Some(len) = spl.get(at + 2..at + 6) else {
            out.push_str(&format!(", section {id} truncated at {at}"));
            return out;
        };
        let len = u32::from_be_bytes(len.try_into().expect("four bytes")) as usize;
        out.push_str(&format!(", section {id} flags {} len {len}", spl[at + 1]));
        at += 6 + len;
    }
    out.push_str(&format!(", {} bytes of trees", spl.len() - at - 1));
    out
}

/// Every word in one of the loaded trees, in the order the tree holds them
/// (which is sorted, because the writer sorts siblings).
///
/// This is a second decoder for the in-memory shape `read_tree_node` builds
/// — `byts[n]` is the child count, the `n+1..n+1+count` bytes are the
/// children, a zero byte is a word end and its `idxs` entry the word's
/// flags, and any other byte's `idxs` entry is the child node. Walking it
/// here rather than calling `find_word` is the point: a reader that stored
/// the tree wrongly and a lookup that read it back the same wrong way would
/// agree, and this does not go through the lookup.
///
fn tree_words(byts: &[u8], idxs: &[i32]) -> Vec<String> {
    let mut out = Vec::new();
    if byts.is_empty() {
        return out;
    }
    let mut stack = vec![(0usize, Vec::<u8>::new())];
    while let Some((node, prefix)) = stack.pop() {
        let count = byts[node] as usize;
        // Reversed, so that a stack pops the children in tree order.
        for i in (0..count).rev() {
            let at = node + 1 + i;
            if byts[at] == 0 {
                out.push(String::from_utf8_lossy(&prefix).into_owned());
            } else {
                let mut deeper = prefix.clone();
                deeper.push(byts[at]);
                stack.push((idxs[at] as usize, deeper));
            }
        }
    }
    out
}

/// The words of a language's case-folded tree.
///
/// # Safety
///
/// `lp` must be a language `spell_load_file` answered.
unsafe fn fold_words(lp: *const slang_T) -> Vec<String> {
    // SAFETY: the caller promises the language.
    let (byts, idxs) = unsafe { (*lp).sl_fold_tree.as_slices() };
    tree_words(byts, idxs)
}

/// Build the golden's inputs in the sandbox and run the writer over them,
/// answering the bytes it wrote.
fn write_one_spl(sandbox: &Sandbox, stem: &str) -> Vec<u8> {
    sandbox.write(&format!("{stem}.dic"), b"1\none\n");
    sandbox.write(&format!("{stem}.aff"), b"");
    // The limits `'mkspellmem'` names, which only `didset_options()` would
    // otherwise install. Its default is the value the golden was made with.
    spell_check_msm().expect("the default 'mkspellmem' parses");

    let out = cstr(format!("{stem}-utf8.spl"));
    let input = cstr(stem);
    let mut names: [*mut c_char; 2] = [out.as_ptr().cast_mut(), input.as_ptr().cast_mut()];
    // SAFETY: both paths are this frame's, NUL-terminated, and live across
    // the call; `over_write` lets a rerun replace its own output and
    // `added_word` keeps the progress messages off stdout.
    unsafe { mkspell(2, names.as_mut_ptr(), false, true, true) };
    std::fs::read(sandbox.path(&format!("{stem}-utf8.spl"))).expect("the writer wrote a file")
}

/// The writer's output for the fixed inputs is the checked-in golden, byte
/// for byte. A change to the format has to land here, in a diff a reviewer
/// can see, rather than only in a `.spl` nobody reads.
#[test]
fn the_writer_reproduces_the_golden_byte_for_byte() {
    let sandbox = Sandbox::dir("spellfile-golden");
    let written = write_one_spl(&sandbox, "Xdet");
    assert_eq!(
        written,
        GOLDEN,
        "\n  wrote  {}\n  golden {}",
        describe(&written),
        describe(GOLDEN)
    );
}

/// Twice from the same inputs is twice the same bytes: the format carries
/// no timestamp unless the affix file asks for a `.sug` file, and this one
/// does not.
#[test]
fn the_writer_is_deterministic() {
    let sandbox = Sandbox::dir("spellfile-deterministic");
    let first = write_one_spl(&sandbox, "Xdet");
    let second = write_one_spl(&sandbox, "Xdet");
    assert_eq!(first, second);
}

/// The header the reader refuses a file over, spelled out where a change to
/// it is visible.
#[test]
fn the_golden_carries_the_documented_magic_and_version() {
    assert_eq!(&GOLDEN[..8], b"VIMspell");
    assert_eq!(GOLDEN[8], 50, "VIMSPELLVERSION");
    // `SN_CHARFLAGS` is the only section a plain word list produces, and it
    // carries `SNF_REQUIRED`: a reader that skipped it would fold case with
    // the wrong table, so it must refuse the file instead.
    assert_eq!(GOLDEN[9], 1, "SN_CHARFLAGS");
    assert_eq!(GOLDEN[10], 1, "SNF_REQUIRED");
    assert!(describe(GOLDEN).contains("SN_END"), "{}", describe(GOLDEN));
}

/// Round trip: the golden goes back through the reader and the word is in
/// the tree it built, and nothing else is.
#[test]
fn the_golden_round_trips_through_the_reader() {
    let sandbox = Sandbox::dir("spellfile-roundtrip");
    let at = sandbox.write("one-utf8.spl", GOLDEN);
    let path = CString::new(at.to_str().expect("a temp path is text")).expect("no interior NUL");
    // SAFETY: the path is this frame's and NUL-terminated; no language name
    // and no old language means a fresh one this case owns.
    let lp = unsafe {
        spell_load_file(
            path.as_ptr().cast_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            true,
        )
    };
    assert!(!lp.is_null(), "the golden loads");
    // SAFETY: the language was just allocated and is freed below.
    let words = unsafe { fold_words(lp) };
    assert_eq!(words, vec![WORD.to_string()]);
    // The keep-case tree of a lower-case-only word list is empty.
    // SAFETY: as above.
    assert!(unsafe { (*lp).sl_keep_tree.as_slices().0.is_empty() });
    // SAFETY: as above; nothing else holds this language.
    unsafe { neovim::spell::slang_free(lp) };
}

/// What the writer wrote is what the reader reads: the same tree comes back
/// from the bytes the writer produced in this process, not only from the
/// checked-in copy.
#[test]
fn the_writer_and_the_reader_agree() {
    let sandbox = Sandbox::dir("spellfile-writer-reader");
    write_one_spl(&sandbox, "Xdet");
    let at = sandbox.path("Xdet-utf8.spl");
    let path = CString::new(at.to_str().expect("a temp path is text")).expect("no interior NUL");
    // SAFETY: as the round-trip case above.
    let lp = unsafe {
        spell_load_file(
            path.as_ptr().cast_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            true,
        )
    };
    assert!(!lp.is_null(), "what the writer wrote loads");
    // SAFETY: the language was just allocated and is freed below.
    assert_eq!(unsafe { fold_words(lp) }, vec![WORD.to_string()]);
    // SAFETY: nothing else holds this language.
    unsafe { neovim::spell::slang_free(lp) };
}

/// The language the tree ships is 621,617 bytes of every section the format
/// has, and it is git-tracked — so it is a read-side golden that costs
/// nothing to keep. This is the case that would notice a section parser
/// going wrong; the 424-byte golden has only two of them.
#[test]
fn the_shipped_english_language_loads_with_its_sections() {
    let _sandbox = Sandbox::globals();
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SHIPPED)
        .canonicalize()
        .expect("the shipped language is in the tree");
    let path = CString::new(at.to_str().expect("a repo path is text")).expect("no interior NUL");
    // SAFETY: the path is this frame's and NUL-terminated.
    let lp = unsafe {
        spell_load_file(
            path.as_ptr().cast_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            true,
        )
    };
    assert!(!lp.is_null(), "en.utf-8.spl loads");
    assert_eq!(
        describe(&std::fs::read(&at).expect("the shipped language is readable")),
        "VIMspell version 50, section 0 flags 1 len 10, section 1 flags 1 len 387, \
         section 2 flags 1 len 1, section 4 flags 0 len 560, section 5 flags 0 len 858, \
         section 12 flags 0 len 12, section 13 flags 0 len 1284, section 7 flags 0 len 74, \
         section 11 flags 0 len 8, SN_END at 3257, 618359 bytes of trees"
    );

    // SAFETY: the language was just allocated and is freed below. Each
    // field below is one section of the file, so a section the reader
    // stopped filling in shows up here as a missing feature rather than as
    // a mis-spelled word three suites away.
    unsafe {
        // SN_REGION: the five two-letter regions, lower-cased as the
        // reader stores them, in the order the file lists them.
        let regions = std::ffi::CStr::from_ptr((*lp).sl_regions.as_ptr());
        assert_eq!(regions.to_bytes(), b"usaucagbnz");
        // SN_MIDWORD, one character long.
        assert!(!(*lp).sl_midword.is_null(), "SN_MIDWORD");
        // SN_SOFO is absent and SN_SAL present, which is what makes
        // sound-folding fall to the `SAL` rules.
        assert!(!(*lp).sl_sofo);
        assert!(!(*lp).sl_sal.is_empty(), "SN_SAL");
        assert!(!(*lp).sl_rep.is_empty(), "SN_REP");
        assert!(!(*lp).sl_repsal.is_empty(), "SN_REPSAL");
        assert!((*lp).sl_has_map, "SN_MAP");
        assert!((*lp).sl_wordcount.ht_used > 0, "SN_WORDS (the COMMON list)");
        // SN_SUGFILE: the timestamp a `.sug` file would have to match.
        // There is no `.sug` in the tree, so this is the only place the
        // field is seen at all.
        assert_ne!((*lp).sl_sugtime, 0, "SN_SUGFILE");
        // Sections the file does *not* carry, named so that a reader that
        // started inventing them is caught too.
        assert!((*lp).sl_info.is_null(), "no SN_INFO");
        assert_eq!((*lp).sl_prefixcnt, 0, "no SN_PREFCOND");
        // `slang_alloc`'s default, untouched: no SN_COMPOUND.
        assert_eq!((*lp).sl_compmax, 254, "no SN_COMPOUND");
        assert!(!(*lp).sl_nobreak, "no SN_NOBREAK");
        assert!((*lp).sl_syllable.is_null(), "no SN_SYLLABLE");
        assert!(!(*lp).sl_fold_tree.as_slices().0.is_empty());
        assert!(
            !(*lp).sl_keep_tree.as_slices().0.is_empty(),
            "a keep-case tree"
        );
    }

    // SAFETY: the language was just allocated.
    let words = unsafe { fold_words(lp) };
    // Sorted, and only weakly: a node may carry more than one word end,
    // one per set of flags the word has (region, affix, rare), and the
    // walk reports each. That is why the two counts differ, and why
    // `find_word` loops over the zero bytes rather than taking the first.
    assert!(
        words.windows(2).all(|w| w[0] <= w[1]),
        "the tree is in sorted order"
    );
    let mut distinct = words.clone();
    distinct.dedup();
    // Both counts are goldens of their own: they change only when the
    // shipped file changes, and the shipped file is checked in.
    assert_eq!((words.len(), distinct.len()), (174_788, 171_580));
    for word in ["the", "hello", "spelling", "colour", "color"] {
        assert!(
            words.binary_search(&word.to_string()).is_ok(),
            "{word} is in the shipped word list"
        );
    }
    for word in ["asdfgh", "qwertyu"] {
        assert!(
            words.binary_search(&word.to_string()).is_err(),
            "{word} is not"
        );
    }

    // SAFETY: nothing else holds this language.
    unsafe { neovim::spell::slang_free(lp) };
}

/// The reader's tree, seen from where the editor sees it: `spell_check`.
///
/// The tree walk above proves the bytes came back; this proves the lookup
/// agrees, which is the level every other suite observes and none of them
/// observes in isolation. The language is installed into the window by
/// hand rather than through `'spelllang'`, because resolving that name
/// wants a runtime path and a `.spl` in it, and neither says anything about
/// the format.
#[test]
fn the_golden_answers_spell_check() {
    let sandbox = Sandbox::dir("spellfile-spell-check");
    let at = sandbox.write("one-utf8.spl", GOLDEN);
    let path = CString::new(at.to_str().expect("a temp path is text")).expect("no interior NUL");
    // SAFETY: the path is this frame's and NUL-terminated.
    let lp = unsafe {
        spell_load_file(
            path.as_ptr().cast_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            true,
        )
    };
    assert!(!lp.is_null(), "the golden loads");
    init_spell_chartab();

    let wp = curwin.get();
    assert!(!wp.is_null(), "`early_init` left a window");
    // SAFETY: the window is the editor's own and the sandbox holds the
    // lock, so nothing else is reading `b_langp` while it is swapped out.
    // The whole garray is saved and put back, so the case leaves the
    // window's language list exactly as it found it.
    let saved = unsafe { (*(*wp).w_s).b_langp };
    let langp = unsafe { &raw mut (*(*wp).w_s).b_langp };
    // SAFETY: as above; one entry, this language, every region.
    unsafe {
        ga_init(langp, size_of::<langp_T>() as c_int, 1);
        ga_grow(langp, 1);
        (*langp).ga_data.cast::<langp_T>().write(langp_T {
            lp_slang: lp,
            lp_sallang: lp,
            lp_replang: lp,
            lp_region: REGION_ALL,
        });
        (*langp).ga_len = 1;
    }

    // `spell_check` writes a highlight id only when the word is *not*
    // good, so a sentinel that is not a highlight id says which happened
    // without this file having to name `HLF_SPB`.
    const UNTOUCHED: c_int = -1;
    let verdict = |word: &str| -> (usize, c_int) {
        let text = cstr(word);
        let mut attr: c_int = UNTOUCHED;
        // SAFETY: the text is this frame's and NUL-terminated, the window
        // is the editor's, `attr` is a local, and no capital column is
        // asked for.
        let len = unsafe {
            spell_check(
                wp,
                text.as_ptr().cast_mut(),
                &raw mut attr,
                std::ptr::null_mut(),
                false,
            )
        };
        (len, attr)
    };

    assert_eq!(verdict(WORD), (3, UNTOUCHED), "`one` is in the tree");
    // The whole word is consumed either way, so that the caller can step
    // over a bad word as easily as a good one.
    let (len, attr) = verdict("onx");
    assert_eq!(len, 3, "`onx` is consumed whole");
    assert_ne!(attr, UNTOUCHED, "`onx` is not in the tree");
    // Case: the tree holds the folded word, so the capitalised form is
    // good and the mixed-case form is not.
    assert_eq!(verdict("One").1, UNTOUCHED, "`One` is `one` capitalised");
    assert_ne!(verdict("oNe").1, UNTOUCHED, "`oNe` is not");

    // SAFETY: as above — the borrowed list goes, the window's comes back.
    unsafe {
        ga_clear(langp);
        (*(*wp).w_s).b_langp = saved;
        neovim::spell::slang_free(lp);
    }
}

/// The `.sug` pair, with the timestamp masked out.
///
/// `.sug` files are the one part of the format that is not reproducible:
/// `put_sugfile` stamps `time(NULL)` into the `.spl`'s `SN_SUGFILE` section
/// and `sug_write` writes the same value into the `.sug`, and the reader
/// cross-checks them so that a stale `.sug` is ignored rather than believed.
/// Masking the eight bytes in both files leaves everything else pinned, and
/// asserting the two unmasked values are *equal* pins the thing the mask
/// gave up.
///
/// The fixtures are the `one` dictionary again with an `.aff` that carries
/// two `SAL` rules — the smallest thing that makes a `.sug` file at all.
#[test]
fn a_sug_file_matches_its_spl_once_the_timestamp_is_masked() {
    const SAL_SPL: &[u8] = include_bytes!("../fixtures/spell/sal-utf8.spl");
    const SAL_SUG: &[u8] = include_bytes!("../fixtures/spell/sal-utf8.sug");

    let sandbox = Sandbox::dir("spellfile-sug");
    sandbox.write("Xsug.dic", b"1\none\n");
    sandbox.write("Xsug.aff", b"SAL n n\nSAL e _\n");
    spell_check_msm().expect("the default 'mkspellmem' parses");
    let out = cstr("Xsug-utf8.spl");
    let input = cstr("Xsug");
    let mut names: [*mut c_char; 2] = [out.as_ptr().cast_mut(), input.as_ptr().cast_mut()];
    // SAFETY: as `write_one_spl` — this frame's paths, live across the call.
    unsafe { mkspell(2, names.as_mut_ptr(), false, true, true) };

    let mut spl = std::fs::read(sandbox.path("Xsug-utf8.spl")).expect("a .spl");
    let mut sug = std::fs::read(sandbox.path("Xsug-utf8.sug")).expect("a .sug");
    assert_eq!(&sug[..6], b"VIMsug", "the .sug magic");
    assert_eq!(sug[6], 1, "SUG version");

    // The `.spl`'s copy lives in the payload of section 11, which the
    // section walk finds by length rather than by a fixed offset.
    let at = sugfile_section(&spl).expect("SN_SUGFILE");
    let stamped = spl[at..at + 8].to_vec();
    assert_eq!(&sug[7..15], &stamped[..], "the two timestamps agree");
    assert_ne!(stamped, [0; 8], "and are a real time");

    spl[at..at + 8].fill(0);
    sug[7..15].fill(0);
    assert_eq!(
        spl,
        SAL_SPL,
        "\n  wrote  {}\n  golden {}",
        describe(&spl),
        describe(SAL_SPL)
    );
    assert_eq!(sug, SAL_SUG);
}

/// Where `SN_SUGFILE`'s eight timestamp bytes start in a `.spl`, by walking
/// the section list rather than assuming an offset.
fn sugfile_section(spl: &[u8]) -> Option<usize> {
    let mut at = 9;
    while *spl.get(at)? != 255 {
        let len = u32::from_be_bytes(spl.get(at + 2..at + 6)?.try_into().ok()?) as usize;
        if spl[at] == 11 {
            assert_eq!(len, 8, "SN_SUGFILE is one eight-byte time");
            return Some(at + 6);
        }
        at += 6 + len;
    }
    None
}
