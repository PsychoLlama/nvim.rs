//! `:syntax keyword` and the keyword hash tables.
//!
//! Keywords are not patterns: they live in one of two hash tables (case
//! sensitive and not) keyed by the keyword text, so a lookup is a hash rather
//! than a scan of every item. [`add_keyword`] fills them, [`syn_clear_keyword`]
//! and [`clear_keywtab`] empty them.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::types::NUL;

/// Where a `keyentry_T`'s trailing `keyword` array starts.
///
/// The hash tables key on that array rather than on the entry, so every walk
/// converts between the two by this offset. Upstream spells it as the `HI2KE`
/// and `KE2HIKEY` macros.
const KEYWORD_OFFSET: usize = ::core::mem::offset_of!(keyentry_T, keyword);

/// The entry a hash key points into.
#[inline]
pub(crate) unsafe fn key_to_entry(key: *mut c_char) -> *mut keyentry_T {
    unsafe { key.byte_sub(KEYWORD_OFFSET) as *mut keyentry_T }
}

/// An entry's hash key: a pointer to its trailing `keyword` array.
///
/// `&raw mut (*kp).keyword` and not `.as_ptr()`: the field is a zero-length
/// array, so an autoref would cover no bytes and the pointer could not be
/// walked.
#[inline]
pub(crate) unsafe fn entry_to_key(kp: *mut keyentry_T) -> *mut c_char {
    unsafe { (&raw mut (*kp).keyword).cast::<c_char>() }
}

/// Free one entry and the two id lists it owns.
unsafe fn free_entry(kp: *mut keyentry_T) {
    unsafe { xfree((*kp).next_list as *mut c_void) };
    unsafe { xfree((*kp).k_syn.cont_in_list as *mut c_void) };
    unsafe { xfree(kp as *mut c_void) };
}

/// Drop every keyword of group `id` from `ht`.
pub(crate) unsafe fn syn_clear_keyword(id: c_int, ht: *mut hashtab_T) {
    unsafe { hash_lock(ht) };
    let mut todo = unsafe { (*ht).ht_used } as c_int;
    let mut hi = unsafe { (*ht).slot_ptr() };
    while todo > 0 {
        if !unsafe { (*hi).is_kept() } {
            hi = unsafe { hi.offset(1) };
            continue;
        }
        todo -= 1;

        // Walk the entry chain hanging off this slot, unlinking the
        // entries of `id`. The slot's key names the chain's head, so
        // removing the head has to rewrite it.
        let mut kp_prev: *mut keyentry_T = ::core::ptr::null_mut();
        let mut kp = unsafe { key_to_entry((*hi).hi_key) };
        while !kp.is_null() {
            if unsafe { (*kp).k_syn.id } as c_int != id {
                kp_prev = kp;
                kp = unsafe { (*kp).ke_next };
                continue;
            }
            let kp_next = unsafe { (*kp).ke_next };
            if kp_prev.is_null() {
                if kp_next.is_null() {
                    unsafe { hash_remove(ht, hi) };
                } else {
                    unsafe { (*hi).hi_key = entry_to_key(kp_next) };
                }
            } else {
                unsafe { (*kp_prev).ke_next = kp_next };
            }
            unsafe { free_entry(kp) };
            kp = kp_next;
        }
        hi = unsafe { hi.offset(1) };
    }
    unsafe { hash_unlock(ht) };
}

/// Empty a whole keyword table.
pub(crate) unsafe fn clear_keywtab(ht: *mut hashtab_T) {
    let mut todo = unsafe { (*ht).ht_used } as c_int;
    let mut hi = unsafe { (*ht).slot_ptr() };
    while todo > 0 {
        if unsafe { (*hi).is_kept() } {
            todo -= 1;
            let mut kp = unsafe { key_to_entry((*hi).hi_key) };
            while !kp.is_null() {
                let kp_next = unsafe { (*kp).ke_next };
                unsafe { free_entry(kp) };
                kp = kp_next;
            }
        }
        hi = unsafe { hi.offset(1) };
    }
    // SAFETY: the caller's table.
    hash_reset(unsafe { &mut *ht });
}

/// Everything a run of `:syntax keyword` gives every keyword it defines.
struct KeywordDef {
    /// The syntax group the keywords belong to.
    id: c_int,
    /// `HL_*` flags from the options.
    flags: SynFlags,
    /// `containedin=`, copied per keyword.
    cont_in_list: *mut int16_t,
    /// `nextgroup=`, copied per keyword.
    next_list: *mut int16_t,
    /// `cchar=`, or NUL.
    conceal_char: c_int,
}

/// Add one keyword to the table its case sensitivity selects.
unsafe fn add_keyword(name: *mut c_char, namelen: size_t, def: &KeywordDef) {
    // With `:syntax case ignore` the table is keyed on the folded form,
    // and the lookup folds too.
    let ignore_case = cur_syn_block().b_syn_ic != 0;
    let mut name_folded: [c_char; MAXKEYWLEN as usize + 1] = [0; MAXKEYWLEN as usize + 1];
    let (name_ic, name_iclen) = if ignore_case {
        let folded = unsafe {
            str_foldcase(
                name,
                namelen as c_int,
                &raw mut name_folded as *mut c_char,
                MAXKEYWLEN + 1,
            )
        };
        (folded, unsafe { cstr::bytes_at(folded) }.len())
    } else {
        (name, namelen)
    };

    // The keyword text lives in the entry's trailing array, so the entry
    // is sized for it.
    let kp = unsafe { xmalloc(KEYWORD_OFFSET + name_iclen + 1) } as *mut keyentry_T;
    let key = unsafe { entry_to_key(kp) };
    unsafe { strcpy(key, name_ic) };
    unsafe { (*kp).k_syn.id = def.id as int16_t };
    unsafe { (*kp).k_syn.inc_tag = current_syn_inc_tag.get() };
    unsafe { (*kp).flags = def.flags };
    unsafe { (*kp).k_char = def.conceal_char };
    unsafe { (*kp).k_syn.cont_in_list = copy_id_list(def.cont_in_list) };
    if !def.cont_in_list.is_null() {
        cur_syn_block().b_syn_containedin = 1;
    }
    unsafe { (*kp).next_list = copy_id_list(def.next_list) };

    let hash = unsafe { hash_hash(key) };
    let ht = if ignore_case {
        syn_field!(cur_syn_block(), b_keywtab_ic)
    } else {
        syn_field!(cur_syn_block(), b_keywtab)
    };
    let hi = unsafe { hash_lookup(ht, key, cstr::bytes_at(key).len(), hash) };
    if unsafe { (*hi).is_kept() } {
        // The keyword already has entries: prepend to the chain.
        unsafe { (*kp).ke_next = key_to_entry((*hi).hi_key) };
        unsafe { (*hi).hi_key = key };
    } else {
        unsafe { (*kp).ke_next = ::core::ptr::null_mut() };
        unsafe { hash_add_item(ht, hi, key, hash) };
    }
}

/// Add the keyword at `kw` and every variant its `[optional tail]` names,
/// answering where the next keyword of the buffer starts.
///
/// `ab[cde]` defines `ab`, `abc`, `abcd` and `abcde`. The variants are made by
/// editing the buffer in place: the `[` becomes the NUL of the short form,
/// then each following character is shifted left over it. `None` means the
/// notation was malformed and the message has been given.
unsafe fn add_keyword_variants(mut kw: *mut c_char, def: &KeywordDef) -> Option<*mut c_char> {
    let mut kwlen;
    let mut p = unsafe { vim_strchr(kw, '[' as c_int) };
    loop {
        if p.is_null() {
            kwlen = unsafe { cstr::bytes_at(kw) }.len();
        } else {
            unsafe { *p = NUL as c_char };
            kwlen = unsafe { p.offset_from(kw) } as size_t;
        }
        unsafe { add_keyword(kw, kwlen, def) };
        if p.is_null() {
            break;
        }
        let next = unsafe { *p.add(1) } as c_int;
        if next == NUL {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let kw = unsafe { c_str(kw) };
            semsg!("E789: Missing ']': {kw}");
            return None;
        }
        if next == ']' as c_int {
            if unsafe { *p.add(2) } as c_int != NUL {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (kw, arg1) = unsafe { (c_str(kw), c_str(p.add(2))) };
                semsg!("E890: Trailing char after ']': {kw}]{arg1}");
                return None;
            }
            // Step over the `]`: it and the NUL after it are exactly the
            // two bytes between here and the next keyword.
            kw = unsafe { p.add(1) };
            kwlen = 1;
            break;
        }
        // Shift the next character left over the NUL, lengthening the
        // keyword by one, and look again from there.
        let l = unsafe { utfc_ptr2len(p.add(1)) };
        unsafe { p.cast::<u8>().copy_from(p.add(1).cast(), l as size_t) };
        p = unsafe { p.add(l as usize) };
    }
    Some(unsafe { kw.add(kwlen + 1) })
}

/// `:syntax keyword {group} [{options}] {keyword} ..`.
pub(crate) unsafe fn syn_cmd_keyword(eap: *mut exarg_T, _syncing: c_int) {
    let arg = unsafe { (*eap).arg };
    let mut group_name_end = ::core::ptr::null_mut::<c_char>();
    let mut conceal_char: c_int = NUL;

    let mut rest = unsafe { get_group_name(arg, &mut group_name_end) };
    if !rest.is_null() {
        let syn_id = if unsafe { (*eap).skip } != 0 {
            -1
        } else {
            unsafe { syn_check_group(arg, group_name_end.offset_from(arg) as size_t) }
        };
        if syn_id != 0 {
            // A buffer for the keywords with their backslashes removed;
            // it can only shrink, so the argument's length is enough.
            let keyword_copy = unsafe { xmalloc(cstr::bytes_at(rest).len() + 1) } as *mut c_char;
            let mut opt = syn_opt_arg_T {
                flags: SynFlags::NONE,
                keyword: true,
                sync_idx: ::core::ptr::null_mut(),
                has_cont_list: false,
                cont_list: ::core::ptr::null_mut(),
                cont_in_list: ::core::ptr::null_mut(),
                next_list: ::core::ptr::null_mut(),
            };

            // The options apply to ALL the keywords, so every option has
            // to be read before any keyword can be created. Pass 1
            // collects them and copies the keywords into the buffer.
            let mut cnt = 0;
            let mut p = keyword_copy;
            while !rest.is_null() && ends_excmd(unsafe { *rest } as c_int) == 0 {
                rest = unsafe { get_syn_options(rest, &mut opt, &mut conceal_char, (*eap).skip) };
                if rest.is_null() || ends_excmd(unsafe { *rest } as c_int) != 0 {
                    break;
                }
                while unsafe { *rest } as c_int != NUL && !ascii_iswhite(unsafe { *rest } as c_int)
                {
                    if unsafe { *rest } as c_int == '\\' as c_int
                        && unsafe { *rest.add(1) } as c_int != NUL
                    {
                        rest = unsafe { rest.add(1) };
                    }
                    unsafe { *p = *rest };
                    p = unsafe { p.add(1) };
                    rest = unsafe { rest.add(1) };
                }
                unsafe { *p = NUL as c_char };
                p = unsafe { p.add(1) };
                cnt += 1;
                rest = unsafe { skipwhite(rest) };
            }

            // Pass 2: an entry per keyword.
            if unsafe { (*eap).skip } == 0 {
                unsafe { syn_incl_toplevel(syn_id, &mut opt.flags) };
                let def = KeywordDef {
                    id: syn_id,
                    flags: opt.flags,
                    cont_in_list: opt.cont_in_list,
                    next_list: opt.next_list,
                    conceal_char,
                };
                let mut kw = keyword_copy;
                while cnt > 0 {
                    cnt -= 1;
                    match unsafe { add_keyword_variants(kw, &def) } {
                        Some(next) => kw = next,
                        None => break,
                    }
                }
            }

            unsafe { xfree(keyword_copy as *mut c_void) };
            unsafe { xfree(opt.cont_in_list as *mut c_void) };
            unsafe { xfree(opt.next_list as *mut c_void) };
        }
    }

    if rest.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {arg}");
    } else {
        unsafe { (*eap).nextcmd = check_nextcmd(rest) };
    }

    redraw_curbuf_later(UPD_SOME_VALID);
    unsafe { syn_stack_free_all(cur_syn_block().raw()) }; // Need to recompute all syntax.
}
