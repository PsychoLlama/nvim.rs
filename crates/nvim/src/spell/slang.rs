//! The lifetime of one loaded language.
//!
//! A [`SpellLang`] is everything a single `.spl` file turned into: three word
//! trees, the affix and compound rules, the sound-folding tables, the REP
//! list, and — once a `.sug` file has been read — a fourth tree of
//! sound-folded forms. They are chained on `sl_next` from the global
//! `first_lang`, and shared by every window whose `'spelllang'` names them.
//!
//! Only three things happen to one here: it is allocated
//! ([`slang_alloc`]), emptied so the file can be read again
//! ([`slang_clear`]), or freed ([`slang_free`]). The reader in `spellfile`
//! fills it in between.
//!
//! Two per-language tables that nothing else owns also live here: the
//! `COMMON` word counts ([`count_common_word`]), used to prefer suggestions
//! that are ordinary words, and the syllable table
//! ([`init_syl_tab`]/[`count_syllables`]) that `COMPOUNDSYLMAX` is measured
//! against.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_void};

use crate::allocator::Owned;
use crate::buffer::alloc_unregistered_buffer;
use crate::hashtab::{
    hash_add_item, hash_clear_all, hash_hash, hash_init, hash_lookup, hash_reset,
};
use crate::log::{LOGLVL_ERR, logmsg};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_close, ml_open, ml_open_file};
use crate::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xstrdup};
use crate::regexp::vim_regfree;
use crate::strings::vim_strchr;
use crate::types::{HashValue, NUL, OK, RegProg, SpellLang, WordCount, size_t, uint8_t, uint16_t};

use super::{MAXWLEN, MAXWORDCOUNT, SP_FORMERROR, SY_MAXLEN, SylItem, WC_KEY_OFF, WordTree};

/// Free `*p` and null it.
unsafe fn xfree_clear<T>(p: *mut *mut T) {
    unsafe { xfree(*p as *mut c_void) };
    unsafe { *p = core::ptr::null_mut() };
}

/// Allocate an empty language named `lang` (which may be null). The caller
/// fills in `sl_next`.
pub unsafe fn slang_alloc(lang: *mut c_char) -> *mut SpellLang {
    let lp = unsafe { xcalloc(1, size_of::<SpellLang>()) } as *mut SpellLang;

    // The four trees are owned values, and an all-zero `Box<[u8]>` is not
    // one: its pointer would be null where the type promises it is not.
    // Every other field of a `SpellLang` is happy zeroed.
    // SAFETY: `xcalloc` just handed back room for one `SpellLang`, so these
    // are the addresses of four uninitialised fields of it.
    unsafe {
        core::ptr::write(&raw mut (*lp).sl_fold_tree, WordTree::default());
        core::ptr::write(&raw mut (*lp).sl_keep_tree, WordTree::default());
        core::ptr::write(&raw mut (*lp).sl_prefix_tree, WordTree::default());
        core::ptr::write(&raw mut (*lp).sl_sound_tree, WordTree::default());
        core::ptr::write(&raw mut (*lp).sl_sal, Vec::new());
        core::ptr::write(&raw mut (*lp).sl_sofo_map, Vec::new());
        core::ptr::write(&raw mut (*lp).sl_rep, Vec::new());
        core::ptr::write(&raw mut (*lp).sl_repsal, Vec::new());
        core::ptr::write(&raw mut (*lp).sl_comppat, Vec::new());
        core::ptr::write(&raw mut (*lp).sl_syl_items, Vec::new());
    }

    if !lang.is_null() {
        unsafe { (*lp).sl_name = xstrdup(lang) };
    }
    unsafe { (*lp).sl_compmax = MAXWLEN as c_int };
    unsafe { (*lp).sl_compsylmax = MAXWLEN as c_int };
    // All three tables, not just the one the caller is about to fill: a
    // `SpellLang` reached by reference must be a valid value throughout, and
    // `slang_clear` empties `sl_map_hash` whether or not a MAP section was
    // ever read.
    unsafe { hash_init(&raw mut (*lp).sl_wordcount) };
    unsafe { hash_init(&raw mut (*lp).sl_map_hash) };
    unsafe { hash_init(&raw mut (*lp).sl_sounddone) };

    lp
}

/// Free a language and everything it owns.
pub unsafe fn slang_free(slang: *mut SpellLang) {
    unsafe { xfree((*slang).sl_name as *mut c_void) };
    unsafe { xfree((*slang).sl_fname as *mut c_void) };
    unsafe { slang_clear(slang) };
    unsafe { xfree(slang as *mut c_void) };
}

/// Empty a language so its file can be read again, leaving the struct
/// itself usable and its name and chain link intact.
pub unsafe fn slang_clear(slang: *mut SpellLang) {
    // SAFETY: the caller's language. Assigning drops the old tree.
    unsafe {
        (*slang).sl_fold_tree = WordTree::default();
        (*slang).sl_keep_tree = WordTree::default();
        (*slang).sl_prefix_tree = WordTree::default();
    }

    // SAFETY: the caller's language. Assigning drops what was there.
    unsafe {
        (*slang).sl_rep = Vec::new();
        (*slang).sl_repsal = Vec::new();
    }

    // SAFETY: the caller's language. Assigning drops what was there.
    unsafe {
        (*slang).sl_sal = Vec::new();
        (*slang).sl_sofo_map = Vec::new();
    }

    for i in 0..unsafe { (*slang).sl_prefixcnt } {
        unsafe { vim_regfree(*(*slang).sl_prefprog.offset(i as isize)) };
    }
    unsafe { (*slang).sl_prefixcnt = 0 };
    unsafe { xfree_clear(&raw mut (*slang).sl_prefprog) };
    unsafe { xfree_clear(&raw mut (*slang).sl_info) };
    unsafe { xfree_clear(&raw mut (*slang).sl_midword) };

    unsafe { vim_regfree((*slang).sl_compprog) };
    unsafe { (*slang).sl_compprog = core::ptr::null_mut::<RegProg>() };
    unsafe { xfree_clear(&raw mut (*slang).sl_comprules) };
    unsafe { xfree_clear(&raw mut (*slang).sl_compstartflags) };
    unsafe { xfree_clear(&raw mut (*slang).sl_compallflags) };

    unsafe { xfree_clear(&raw mut (*slang).sl_syllable) };
    unsafe { (*slang).sl_syl_items = Vec::new() };

    unsafe { (*slang).sl_comppat = Vec::new() };

    unsafe { hash_clear_all(&raw mut (*slang).sl_wordcount, WC_KEY_OFF as u32) };
    // SAFETY: the caller's language.
    hash_reset(unsafe { &mut (*slang).sl_wordcount });

    unsafe { hash_clear_all(&raw mut (*slang).sl_map_hash, 0) };

    unsafe { slang_clear_sug(slang) };

    unsafe { (*slang).sl_compmax = MAXWLEN as c_int };
    unsafe { (*slang).sl_compminlen = 0 };
    unsafe { (*slang).sl_compsylmax = MAXWLEN as c_int };
    unsafe { (*slang).sl_regions[0] = NUL as c_char };
}

/// Drop what the `.sug` file contributed, so it can be read again.
pub unsafe fn slang_clear_sug(slang: *mut SpellLang) {
    // SAFETY: the caller's language. Assigning drops the old tree.
    unsafe { (*slang).sl_sound_tree = WordTree::default() };
    unsafe { close_spellbuf(Buf::from_raw((*slang).sl_sugbuf)) };
    unsafe { (*slang).sl_sugbuf = core::ptr::null_mut() };
    unsafe { (*slang).sl_sugloaded = false };
    unsafe { (*slang).sl_sugtime = 0 };
}

/// Note that `word` is a `COMMON` word of `slang`, or bump its count if it is
/// already known.
///
/// `len` is the word's length, or -1 when it is NUL terminated. `count` is
/// 1 to count one use and 10 to seed a word the `.spl` file declared
/// common. The count saturates rather than wrapping.
pub unsafe fn count_common_word(
    slang: *mut SpellLang,
    word: *mut c_char,
    len: c_int,
    count: uint8_t,
) {
    let mut buf = [0 as c_char; MAXWLEN];
    let p = if len == -1 {
        word
    } else if len >= MAXWLEN as c_int {
        return;
    } else {
        let to = buf.as_mut_ptr() as *mut c_void;
        unsafe { xmemcpyz(to, word as *const c_void, len as size_t) };
        buf.as_mut_ptr()
    };

    let hash: HashValue = unsafe { hash_hash(p) };
    let p_len = unsafe { cstr::bytes_at(p) }.len();
    let hi = unsafe { hash_lookup(&raw mut (*slang).sl_wordcount, p, p_len, hash) };
    if !hi.is_kept() {
        let wc = unsafe { xmalloc(WC_KEY_OFF as size_t + p_len + 1) } as *mut WordCount;
        let key = unsafe { &raw mut (*wc).wc_word }.cast::<c_char>();
        let into = key.cast::<u8>();
        unsafe { into.copy_from_nonoverlapping(p.cast(), p_len + 1) };
        unsafe { (*wc).wc_count = count as uint16_t };
        unsafe { hash_add_item(&raw mut (*slang).sl_wordcount, hi, key, hash) };
    } else {
        let wc = unsafe { hi.hi_key.offset(-(WC_KEY_OFF as isize)) } as *mut WordCount;
        // The C adds and then checks for the wrap, which is a saturate
        // spelled the long way round.
        let total = unsafe { (*wc).wc_count }.wrapping_add(count as uint16_t);
        let capped = if (total as c_int) < count as c_int {
            MAXWORDCOUNT as uint16_t
        } else {
            total
        };
        unsafe { (*wc).wc_count = capped };
    }
}

/// Split `sl_syllable` at its slashes: the part before the first becomes
/// the set of single syllable characters, and each part after it becomes an
/// entry in `sl_syl_items`.
///
/// Returns `SP_FORMERROR` for an entry longer than [`SY_MAXLEN`].
pub unsafe fn init_syl_tab(slang: *mut SpellLang) -> c_int {
    // SAFETY: the caller's language, whose `sl_syllable` is a live
    // NUL-terminated string this splits in place.
    let mut items: Vec<SylItem> = Vec::new();
    let mut p = unsafe { vim_strchr((*slang).sl_syllable, '/' as c_int) };
    while !p.is_null() {
        unsafe { *p = NUL as c_char };
        p = unsafe { p.offset(1) };
        if unsafe { *p } == NUL as c_char {
            break; // trailing slash
        }
        let s = p;
        p = unsafe { vim_strchr(p, '/' as c_int) };
        let l = if p.is_null() {
            unsafe { cstr::bytes_at(s).len() }
        } else {
            unsafe { p.offset_from(s) as usize }
        };
        if l >= SY_MAXLEN as usize {
            return SP_FORMERROR;
        }

        let mut syl = SylItem {
            sy_chars: [0; SY_MAXLEN as usize],
            sy_len: l as c_int,
        };
        // SAFETY: as above; `l` is under `SY_MAXLEN`, which is the room
        // `sy_chars` has, and those bytes are the item's own.
        // SAFETY: as above.
        syl.sy_chars[..l].copy_from_slice(unsafe { &core::slice::from_raw_parts(s, l)[..l] });
        items.push(syl);
    }
    // SAFETY: the caller's language.
    unsafe { (*slang).sl_syl_items = items };
    OK
}

/// How many syllables `word` has, by the language's syllable definition.
///
/// A space resets the count, so what is returned is the count after the
/// last space. Zero means the language defines no syllables.
pub(super) unsafe fn count_syllables(slang: *mut SpellLang, word: *const c_char) -> c_int {
    if unsafe { (*slang).sl_syllable }.is_null() {
        return 0;
    }

    let mut cnt = 0;
    let mut skip = false;
    let mut p = word;
    while unsafe { *p } != 0 {
        if unsafe { *p } == b' ' as c_char {
            cnt = 0;
            p = unsafe { p.offset(1) };
            continue;
        }

        // The longest matching syllable item wins.
        let mut len = 0;
        for syl in unsafe { &(*slang).sl_syl_items } {
            // SAFETY: `p` walks the caller's NUL-terminated word, and the
            // item's own length is what is compared.
            if syl.sy_len > len
                && unsafe { cstr::prefix_eq(p, syl.sy_chars.as_ptr(), syl.sy_len as size_t) }
            {
                len = syl.sy_len;
            }
        }

        if len != 0 {
            cnt += 1;
            skip = false;
        } else {
            // No item matched; a bare syllable character still counts,
            // but only the first of a run.
            let c = unsafe { utf_ptr2char(p) };
            len = unsafe { utfc_ptr2len(p) };
            if unsafe { vim_strchr((*slang).sl_syllable, c) }.is_null() {
                skip = false;
            } else if !skip {
                cnt += 1;
                skip = true;
            }
        }
        p = unsafe { p.offset(len as isize) };
    }
    cnt
}

/// Open a nameless, unlisted buffer holding nothing but text lines, backed
/// by a swap file so that a big `.sug` word list need not stay in memory.
///
/// Most of its fields are invalid: string options are null and there is no
/// undo information.
pub unsafe fn open_spellbuf() -> Option<Buf> {
    // Never registered and never on the buffer list -- see
    // `alloc_unregistered_buffer`.
    // The allocation travels as a bare address: it is stored in a
    // `SpellLang`'s `sl_sugbuf`, and `close_spellbuf` takes it back.
    let buf = alloc_unregistered_buffer().into_raw();

    unsafe { (*buf).b_spell = true };
    unsafe { (*buf).b_p_swf = 1 };
    if unsafe { ml_open(Buf::new(buf)) }.is_err() {
        logmsg!(
            LOGLVL_ERR,
            c"open_spellbuf",
            line!() as c_int,
            "Error opening a new memline"
        );
    }
    unsafe { ml_open_file(Buf::new(buf)) }; // create the swap file now

    unsafe { Buf::from_raw(buf) }
}

/// Close a buffer from [`open_spellbuf`].
pub unsafe fn close_spellbuf(buffer: Option<Buf>) {
    let Some(buffer) = buffer else {
        return;
    };
    unsafe { ml_close(buffer, 1) };
    // The free: `Buffer`'s destructor runs and the memory goes back.
    // SAFETY: `open_spellbuf` gave up this address and nothing else
    // takes it back -- `sl_sugbuf`/`si_spellbuf` are cleared right after
    // this call.
    drop(unsafe { Owned::from_raw(buffer.raw()) });
}
