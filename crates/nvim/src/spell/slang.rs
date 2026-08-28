//! The lifetime of one loaded language.
//!
//! A [`slang_T`] is everything a single `.spl` file turned into: three word
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

use core::ffi::{c_char, c_int, c_void};

use crate::allocator::Owned;
use crate::buffer::alloc_unregistered_buffer;
use crate::garray::{ga_append_via_ptr, ga_clear, ga_clear_strings, ga_init};
use crate::hashtab::{
    hash_add_item, hash_clear_all, hash_hash, hash_init, hash_lookup, hash_removed,
};
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_close, ml_open, ml_open_file};
use crate::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xstrdup};
use crate::os::cshim::strncmp;
use crate::regexp::vim_regfree;
use crate::strings::vim_strchr;
use crate::types::{
    NUL, OK, buf_T, fromto_T, garray_T, hash_T, hashitem_T, regprog_T, salitem_T, size_t, slang_T,
    uint8_t, uint16_t, wordcount_T,
};
use ::libc::{memcpy, strlen};

use super::{FAIL, MAXWLEN, MAXWORDCOUNT, SP_FORMERROR, SY_MAXLEN, WC_KEY_OFF, syl_item_T};

/// Free the contents of `gap`, which holds `T` items each needing `drop`,
/// then the array itself.
unsafe fn ga_deep_clear<T>(gap: *mut garray_T, drop: unsafe fn(*mut T)) {
    if !unsafe { (*gap).ga_data }.is_null() {
        for i in 0..unsafe { (*gap).ga_len } {
            unsafe { drop(((*gap).ga_data as *mut T).offset(i as isize)) };
        }
    }
    unsafe { ga_clear(gap) };
}

/// Free `*p` and null it.
unsafe fn xfree_clear<T>(p: *mut *mut T) {
    unsafe { xfree(*p as *mut c_void) };
    unsafe { *p = core::ptr::null_mut() };
}

/// Allocate an empty language named `lang` (which may be null). The caller
/// fills in `sl_next`.
pub unsafe fn slang_alloc(lang: *mut c_char) -> *mut slang_T {
    let lp = unsafe { xcalloc(1, size_of::<slang_T>()) } as *mut slang_T;

    if !lang.is_null() {
        unsafe { (*lp).sl_name = xstrdup(lang) };
    }
    unsafe { ga_init(&raw mut (*lp).sl_rep, size_of::<fromto_T>() as c_int, 10) };
    unsafe { ga_init(&raw mut (*lp).sl_repsal, size_of::<fromto_T>() as c_int, 10) };
    unsafe { (*lp).sl_compmax = MAXWLEN as c_int };
    unsafe { (*lp).sl_compsylmax = MAXWLEN as c_int };
    unsafe { hash_init(&raw mut (*lp).sl_wordcount) };

    lp
}

/// Free a language and everything it owns.
pub unsafe fn slang_free(lp: *mut slang_T) {
    unsafe { xfree((*lp).sl_name as *mut c_void) };
    unsafe { xfree((*lp).sl_fname as *mut c_void) };
    unsafe { slang_clear(lp) };
    unsafe { xfree(lp as *mut c_void) };
}

/// Free one sound-folding rule.
unsafe fn free_salitem(smp: *mut salitem_T) {
    unsafe { xfree((*smp).sm_lead as *mut c_void) };
    // sm_oneof and sm_rules point into sm_lead, so they are already gone.
    unsafe { xfree((*smp).sm_to as *mut c_void) };
    unsafe { xfree((*smp).sm_lead_w as *mut c_void) };
    unsafe { xfree((*smp).sm_oneof_w as *mut c_void) };
    unsafe { xfree((*smp).sm_to_w as *mut c_void) };
}

/// Free one REP or REPSAL pair.
unsafe fn free_fromto(ftp: *mut fromto_T) {
    unsafe { xfree((*ftp).ft_from as *mut c_void) };
    unsafe { xfree((*ftp).ft_to as *mut c_void) };
}

/// Empty a language so its file can be read again, leaving the struct
/// itself usable and its name and chain link intact.
pub unsafe fn slang_clear(lp: *mut slang_T) {
    unsafe { xfree_clear(&raw mut (*lp).sl_fbyts) };
    unsafe { xfree_clear(&raw mut (*lp).sl_kbyts) };
    unsafe { xfree_clear(&raw mut (*lp).sl_pbyts) };

    unsafe { xfree_clear(&raw mut (*lp).sl_fidxs) };
    unsafe { xfree_clear(&raw mut (*lp).sl_kidxs) };
    unsafe { xfree_clear(&raw mut (*lp).sl_pidxs) };

    unsafe { ga_deep_clear(&raw mut (*lp).sl_rep, free_fromto) };
    unsafe { ga_deep_clear(&raw mut (*lp).sl_repsal, free_fromto) };

    let gap = unsafe { &raw mut (*lp).sl_sal };
    if unsafe { (*lp).sl_sofo } {
        // The SOFO table sets ga_len to 1 without adding an item for
        // latin1, so its entries are bare pointers.
        unsafe { ga_deep_clear::<*mut c_void>(gap, xfree_ptr) };
    } else {
        unsafe { ga_deep_clear(gap, free_salitem) };
    }

    for i in 0..unsafe { (*lp).sl_prefixcnt } {
        unsafe { vim_regfree(*(*lp).sl_prefprog.offset(i as isize)) };
    }
    unsafe { (*lp).sl_prefixcnt = 0 };
    unsafe { xfree_clear(&raw mut (*lp).sl_prefprog) };
    unsafe { xfree_clear(&raw mut (*lp).sl_info) };
    unsafe { xfree_clear(&raw mut (*lp).sl_midword) };

    unsafe { vim_regfree((*lp).sl_compprog) };
    unsafe { (*lp).sl_compprog = core::ptr::null_mut::<regprog_T>() };
    unsafe { xfree_clear(&raw mut (*lp).sl_comprules) };
    unsafe { xfree_clear(&raw mut (*lp).sl_compstartflags) };
    unsafe { xfree_clear(&raw mut (*lp).sl_compallflags) };

    unsafe { xfree_clear(&raw mut (*lp).sl_syllable) };
    unsafe { ga_clear(&raw mut (*lp).sl_syl_items) };

    unsafe { ga_clear_strings(&raw mut (*lp).sl_comppat) };

    unsafe { hash_clear_all(&raw mut (*lp).sl_wordcount, WC_KEY_OFF as u32) };
    unsafe { hash_init(&raw mut (*lp).sl_wordcount) };

    unsafe { hash_clear_all(&raw mut (*lp).sl_map_hash, 0) };

    unsafe { slang_clear_sug(lp) };

    unsafe { (*lp).sl_compmax = MAXWLEN as c_int };
    unsafe { (*lp).sl_compminlen = 0 };
    unsafe { (*lp).sl_compsylmax = MAXWLEN as c_int };
    unsafe { (*lp).sl_regions[0] = NUL as c_char };
}

/// Free one bare pointer held in a garray.
unsafe fn xfree_ptr(p: *mut *mut c_void) {
    unsafe { xfree(*p) }
}

/// Drop what the `.sug` file contributed, so it can be read again.
pub unsafe fn slang_clear_sug(lp: *mut slang_T) {
    unsafe { xfree_clear(&raw mut (*lp).sl_sbyts) };
    unsafe { xfree_clear(&raw mut (*lp).sl_sidxs) };
    unsafe { close_spellbuf((*lp).sl_sugbuf) };
    unsafe { (*lp).sl_sugbuf = core::ptr::null_mut() };
    unsafe { (*lp).sl_sugloaded = false };
    unsafe { (*lp).sl_sugtime = 0 };
}

/// Note that `word` is a `COMMON` word of `lp`, or bump its count if it is
/// already known.
///
/// `len` is the word's length, or -1 when it is NUL terminated. `count` is
/// 1 to count one use and 10 to seed a word the `.spl` file declared
/// common. The count saturates rather than wrapping.
pub unsafe fn count_common_word(lp: *mut slang_T, word: *mut c_char, len: c_int, count: uint8_t) {
    let mut buf = [0 as c_char; MAXWLEN];
    let p = if len == -1 {
        word
    } else if len >= MAXWLEN as c_int {
        return;
    } else {
        unsafe {
            xmemcpyz(
                buf.as_mut_ptr() as *mut c_void,
                word as *const c_void,
                len as size_t,
            )
        };
        buf.as_mut_ptr()
    };

    let hash: hash_T = unsafe { hash_hash(p) };
    let p_len = unsafe { strlen(p) };
    let hi: *mut hashitem_T = unsafe { hash_lookup(&raw mut (*lp).sl_wordcount, p, p_len, hash) };
    if unsafe { (*hi).hi_key }.is_null()
        || core::ptr::eq(unsafe { (*hi).hi_key }, &raw const hash_removed)
    {
        let wc = unsafe { xmalloc(WC_KEY_OFF as size_t + p_len + 1) } as *mut wordcount_T;
        let key = unsafe { &raw mut (*wc).wc_word } as *mut c_char;
        unsafe { memcpy(key as *mut c_void, p as *const c_void, p_len + 1) };
        unsafe { (*wc).wc_count = count as uint16_t };
        unsafe { hash_add_item(&raw mut (*lp).sl_wordcount, hi, key, hash) };
    } else {
        let wc = unsafe { (*hi).hi_key.offset(-(WC_KEY_OFF as isize)) } as *mut wordcount_T;
        unsafe { (*wc).wc_count = (*wc).wc_count.wrapping_add(count as uint16_t) };
        if (unsafe { (*wc).wc_count } as c_int) < count as c_int {
            unsafe { (*wc).wc_count = MAXWORDCOUNT as uint16_t };
        }
    }
}

/// Split `sl_syllable` at its slashes: the part before the first becomes
/// the set of single syllable characters, and each part after it becomes an
/// entry in `sl_syl_items`.
///
/// Returns `SP_FORMERROR` for an entry longer than [`SY_MAXLEN`].
pub unsafe fn init_syl_tab(slang: *mut slang_T) -> c_int {
    unsafe {
        ga_init(
            &raw mut (*slang).sl_syl_items,
            size_of::<syl_item_T>() as c_int,
            4,
        )
    };
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
            unsafe { strlen(s) as c_int }
        } else {
            unsafe { p.offset_from(s) as c_int }
        };
        if l >= SY_MAXLEN {
            return SP_FORMERROR;
        }

        let syl =
            unsafe { ga_append_via_ptr(&raw mut (*slang).sl_syl_items, size_of::<syl_item_T>()) }
                as *mut syl_item_T;
        unsafe {
            xmemcpyz(
                &raw mut (*syl).sy_chars as *mut c_void,
                s as *const c_void,
                l as size_t,
            )
        };
        unsafe { (*syl).sy_len = l };
    }
    OK
}

/// How many syllables `word` has, by the language's syllable definition.
///
/// A space resets the count, so what is returned is the count after the
/// last space. Zero means the language defines no syllables.
pub(super) unsafe fn count_syllables(slang: *mut slang_T, word: *const c_char) -> c_int {
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
        for i in 0..unsafe { (*slang).sl_syl_items.ga_len } {
            let syl =
                unsafe { ((*slang).sl_syl_items.ga_data as *mut syl_item_T).offset(i as isize) };
            if unsafe { (*syl).sy_len } > len
                && unsafe { strncmp(p, (*syl).sy_chars.as_ptr(), (*syl).sy_len as size_t) } == 0
            {
                len = unsafe { (*syl).sy_len };
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
pub unsafe fn open_spellbuf() -> *mut buf_T {
    // Never registered and never on the buffer list -- see
    // `alloc_unregistered_buffer`.
    // The allocation travels as a bare address: it is stored in a
    // `slang_T`'s `sl_sugbuf`, and `close_spellbuf` takes it back.
    let buf = alloc_unregistered_buffer().into_raw();

    unsafe { (*buf).b_spell = true };
    unsafe { (*buf).b_p_swf = 1 };
    if unsafe { ml_open(buf) } == FAIL {
        unsafe {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"open_spellbuf".as_ptr(),
                line!() as c_int,
                true,
                c"Error opening a new memline".as_ptr(),
            )
        };
    }
    unsafe { ml_open_file(buf) }; // create the swap file now

    buf
}

/// Close a buffer from [`open_spellbuf`].
pub unsafe fn close_spellbuf(buf: *mut buf_T) {
    if buf.is_null() {
        return;
    }
    unsafe { ml_close(buf, 1) };
    // The free: `buf_T`'s destructor runs and the memory goes back.
    // SAFETY: `open_spellbuf` gave up this address and nothing else
    // takes it back -- `sl_sugbuf`/`si_spellbuf` are cleared right after
    // this call.
    drop(unsafe { Owned::from_raw(buf) });
}
