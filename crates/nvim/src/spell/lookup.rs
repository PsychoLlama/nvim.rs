//! Looking a word up in a language's word tree.
//!
//! This is the innermost loop of spell checking: [`find_word`] runs once
//! per candidate word per language, and again for every prefix and every
//! compound continuation, so it is by some margin the hottest code in the
//! subsystem.
//!
//! # The tree
//!
//! A `.spl` file stores words as a trie flattened into two parallel arrays.
//! `byts` holds, for each node, a count followed by that many child bytes
//! in sorted order; `idxs` holds the matching child node indices. Walking
//! is therefore: read the count, binary-search the bytes for the next
//! character of the word, follow `idxs` to the child.
//!
//! A child byte of zero does not mean a character — it means "a word ends
//! here", and the `idxs` entry beside it holds that word's `WF_*` flags and
//! region mask instead of a node index. There can be several such zero
//! entries in a row, one per flag/region combination the same spelling has.
//!
//! Three trees exist per language, and `mode` says which one to walk:
//! the case-folded tree (`FIND_FOLDWORD`), the keep-case tree for words
//! whose capitalisation cannot be described by a flag (`FIND_KEEPWORD`),
//! and the prefix tree ([`find_prefix`], which walks it and then calls
//! [`find_word`] with `FIND_PREFIX` for the remainder).
//!
//! # Endings
//!
//! The walk goes as deep as it can and *then* works back out, because the
//! longest match wins. Every "word ends here" node passed on the way down
//! is pushed onto `endidx`/`endlen`, and the second half of [`find_word`]
//! pops them longest-first, checking case, region, prefix conditions and
//! compounding rules until one is accepted.
//!
//! # Compounding
//!
//! When a word may be followed by another to form a compound, [`find_word`]
//! recurses on the remainder with `FIND_COMPOUND`. The flags of the parts
//! collected so far live in `mi_compflags`, and [`can_compound`] /
//! [`match_compoundrule`] decide whether that sequence is one the language
//! allows.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

use crate::mbyte::{mb_charlen_len, utf_char2bytes, utf_head_off, utfc_ptr2len};
use crate::memory::xmemcpyz;
use crate::message::emsg;
use crate::os::cshim::{gettext, strncmp};
use crate::regexp::vim_regexec_prog;
use crate::strings::vim_strchr;
use crate::types::{NUL, garray_T, idx_T, langp_T, regprog_T, slang_T, uint8_t, uint32_t};
use ::libc::strlen;

use super::chartab::{
    byte_in_str, captype, nofold_len, spell_casefold, spell_iswordp, spell_iswordp_nmw,
};
use super::{
    FIND_COMPOUND, FIND_FOLDWORD, FIND_KEEPCOMPOUND, FIND_KEEPWORD, FIND_PREFIX, MAXWLEN, SP_BAD,
    SP_BANNED, SP_LOCAL, SP_OK, SP_RARE, TAB, WF_ALLCAP, WF_BANNED, WF_COMPROOT, WF_FIXCAP,
    WF_HAS_AFF, WF_KEEPCAP, WF_NEEDCOMP, WF_NOCOMPAFT, WF_NOCOMPBEF, WF_ONECAP, WF_PFX_NC, WF_RARE,
    WF_RAREPFX, WF_REGION, count_syllables, e_format, matchinf_T, spell_valid_case,
};

/// Advance `p` past one character.
macro_rules! mb_ptr_adv {
    ($p:expr) => {
        $p = $p.offset(utfc_ptr2len($p) as isize)
    };
}

/// Check whether the word at `mip.mi_word` is in one of `mip`'s language's
/// trees, updating `mi_result` and `mi_end` for a match.
///
/// `mode` picks the tree and says where in the word to start:
///
/// * `FIND_FOLDWORD` — the case-folded tree, from the start.
/// * `FIND_KEEPWORD` — the keep-case tree, from the start.
/// * `FIND_PREFIX` — the case-folded tree, after a prefix [`find_prefix`]
///   already matched.
/// * `FIND_COMPOUND` / `FIND_KEEPCOMPOUND` — either tree, after the
///   compound parts found so far.
pub(super) unsafe fn find_word(mip: *mut matchinf_T, mode: c_int) {
    unsafe {
        let slang = (*(*mip).mi_lp).lp_slang;

        let ptr;
        let mut flen;
        let byts;
        let idxs;
        let mut wlen = 0;
        if mode == FIND_KEEPWORD || mode == FIND_KEEPCOMPOUND {
            // The keep-case tree is matched against the word as written, so
            // no folding is needed and there are always enough bytes.
            ptr = (*mip).mi_word;
            flen = 9999;
            byts = (*slang).sl_kbyts;
            idxs = (*slang).sl_kidxs;

            if mode == FIND_KEEPCOMPOUND {
                wlen += (*mip).mi_compoff;
            }
        } else {
            ptr = &raw mut (*mip).mi_fword as *mut c_char;
            flen = (*mip).mi_fwordlen;
            byts = (*slang).sl_fbyts;
            idxs = (*slang).sl_fidxs;

            if mode == FIND_PREFIX {
                wlen = (*mip).mi_prefixlen;
                flen -= (*mip).mi_prefixlen;
            } else if mode == FIND_COMPOUND {
                wlen = (*mip).mi_compoff;
                flen -= (*mip).mi_compoff;
            }
        }

        if byts.is_null() {
            return; // this language has no such tree
        }

        let mut arridx: idx_T = 0;
        // Where each "a word could end here" node was, and how far into the
        // word it sat.
        let mut endlen = [0 as c_int; MAXWLEN];
        let mut endidx = [0 as idx_T; MAXWLEN];
        let mut endidxcnt = 0usize;

        // Descend until a byte does not match, the tree runs out, or the
        // word does.
        loop {
            if flen <= 0 && *(*mip).mi_fend != 0 {
                flen = fold_more(mip);
            }

            let mut len = *byts.offset(arridx as isize) as c_int;
            arridx += 1;

            // A leading zero byte means a word ends here. Remember the spot
            // and carry on; the longest match is preferred, so the endings
            // are only examined once the descent is done.
            if *byts.offset(arridx as isize) == 0 {
                if endidxcnt == MAXWLEN {
                    // Only a corrupted spell file can nest this deep.
                    emsg(gettext(e_format.get()));
                    return;
                }
                endlen[endidxcnt] = wlen;
                endidx[endidxcnt] = arridx;
                endidxcnt += 1;
                arridx += 1;
                len -= 1;

                // Skip the rest of the zeros: one per flag/region variant.
                while len > 0 && *byts.offset(arridx as isize) == 0 {
                    arridx += 1;
                    len -= 1;
                }
                if len == 0 {
                    break; // no children, the word must end here
                }
            }

            if *ptr.offset(wlen as isize) == 0 {
                break; // end of the line
            }

            let mut c = *ptr.offset(wlen as isize) as uint8_t as c_int;
            if c == TAB {
                c = ' ' as c_int; // a tab counts as a space
            }

            // Binary search the sorted child bytes.
            let mut lo = arridx;
            let mut hi = arridx + len - 1;
            while lo < hi {
                let m = (lo + hi) / 2;
                let b = *byts.offset(m as isize) as c_int;
                if b > c {
                    hi = m - 1;
                } else if b < c {
                    lo = m + 1;
                } else {
                    lo = m;
                    hi = m;
                    break;
                }
            }
            if hi < lo || *byts.offset(lo as isize) as c_int != c {
                break; // no matching byte
            }

            arridx = *idxs.offset(lo as isize);
            wlen += 1;
            flen -= 1;

            // One space in the dictionary word may stand for a run of
            // spaces and tabs in the text.
            if c == ' ' as c_int {
                loop {
                    if flen <= 0 && *(*mip).mi_fend != 0 {
                        flen = fold_more(mip);
                    }
                    let ch = *ptr.offset(wlen as isize) as c_int;
                    if ch != ' ' as c_int && ch != TAB {
                        break;
                    }
                    wlen += 1;
                    flen -= 1;
                }
            }
        }

        // Now try the endings, longest first.
        while endidxcnt > 0 {
            endidxcnt -= 1;
            arridx = endidx[endidxcnt];
            wlen = endlen[endidxcnt];

            if utf_head_off(ptr, ptr.offset(wlen as isize)) > 0 {
                continue; // not on a character boundary
            }

            // A word character right after the match means this is not the
            // end of a word — unless the language compounds or does not
            // break, in which case what follows may continue it.

            let word_ends = if spell_iswordp(ptr.offset(wlen as isize), (*mip).mi_win) {
                if (*slang).sl_compprog.is_null() && !(*slang).sl_nobreak {
                    continue;
                }
                false
            } else {
                true
            };

            // The prefix flag comes before the compound flags; once a valid
            // prefix has been found the rest are tried as compound flags.
            let mut prefix_found = false;

            if mode != FIND_KEEPWORD {
                // Translate the length back into the unfolded word, since
                // folding can change how many bytes a character takes. The
                // comparison is a shortcut for the common case where it did
                // not change anything.
                let mut p = (*mip).mi_word;
                if strncmp(ptr, p, wlen as usize) != 0 {
                    let end = ptr.offset(wlen as isize);
                    let mut s = ptr;
                    while s < end {
                        mb_ptr_adv!(s);
                        mb_ptr_adv!(p);
                    }
                    wlen = p.offset_from((*mip).mi_word) as c_int;
                }
            }

            // Try each flag/region variant recorded for this spelling.
            let mut len = *byts.offset(arridx as isize - 1) as c_int;
            'variants: while len > 0 && *byts.offset(arridx as isize) == 0 {
                // `break 'variant` is C's `continue`: on to the next
                // flag/region entry. `break 'variants` is C's `break`: this
                // ending is settled.
                'variant: {
                    let mut flags = *idxs.offset(arridx as isize) as uint32_t;

                    if mode == FIND_FOLDWORD {
                        // The fold-case tree records what case the word must be
                        // written in; the keep-case tree is right by
                        // construction, and prefixes are not worth checking.
                        if (*mip).mi_cend != (*mip).mi_word.offset(wlen as isize) {
                            (*mip).mi_cend = (*mip).mi_word.offset(wlen as isize);
                            (*mip).mi_capflags = captype((*mip).mi_word, (*mip).mi_cend);
                        }

                        if (*mip).mi_capflags == WF_KEEPCAP as c_int
                            || !spell_valid_case((*mip).mi_capflags, flags as c_int)
                        {
                            break 'variant;
                        }
                    } else if mode == FIND_PREFIX && !prefix_found {
                        // The word has to accept one of the prefixes
                        // find_prefix() left at mi_prefarridx.
                        let c = valid_word_prefix(
                            (*mip).mi_prefcnt,
                            (*mip).mi_prefarridx,
                            flags as c_int,
                            (*mip).mi_word.offset((*mip).mi_cprefixlen as isize),
                            slang,
                            false,
                        );
                        if c == 0 {
                            break 'variant;
                        }
                        if c & WF_RAREPFX as c_int != 0 {
                            flags |= WF_RARE;
                        }
                        prefix_found = true;
                    }

                    if (*slang).sl_nobreak {
                        if (mode == FIND_COMPOUND || mode == FIND_KEEPCOMPOUND)
                            && flags & WF_BANNED == 0
                        {
                            // NOBREAK: a valid word follows, which is all the
                            // caller wanted to know.
                            (*mip).mi_result = SP_OK;
                            break 'variants;
                        }
                    } else if mode == FIND_COMPOUND || mode == FIND_KEEPCOMPOUND || !word_ends {
                        if !compound_part_allowed(
                            mip,
                            ptr,
                            wlen,
                            flags,
                            word_ends,
                            mode,
                            endlen[endidxcnt],
                        ) {
                            break 'variant;
                        }
                    } else if flags & WF_NEEDCOMP != 0 {
                        // Only valid as part of a compound.
                        break 'variant;
                    }

                    let mut nobreak_result = SP_OK;

                    if !word_ends {
                        let save_result = (*mip).mi_result;
                        let save_end = (*mip).mi_end;
                        let save_lp = (*mip).mi_lp;

                        // Check that a valid word follows. When compounding, the
                        // recursion sets mi_result itself and there is nothing
                        // left to do here; for NOBREAK only its existence
                        // matters.
                        if (*slang).sl_nobreak {
                            (*mip).mi_result = SP_BAD;
                        }

                        (*mip).mi_compoff = endlen[endidxcnt];
                        if mode == FIND_KEEPWORD {
                            // Translate the keep-case length into a case-folded
                            // one, again short-cutting when folding changed
                            // nothing.
                            let mut p = &raw mut (*mip).mi_fword as *mut c_char;
                            if strncmp(ptr, p, wlen as usize) != 0 {
                                let end = ptr.offset(wlen as isize);
                                let mut s = ptr;
                                while s < end {
                                    mb_ptr_adv!(s);
                                    mb_ptr_adv!(p);
                                }
                                (*mip).mi_compoff =
                                    p.offset_from(&raw mut (*mip).mi_fword as *mut c_char) as c_int;
                            }
                        }
                        (*mip).mi_complen += 1;
                        if flags & WF_COMPROOT != 0 {
                            (*mip).mi_compextra += 1;
                        }

                        // For NOBREAK every language has to be tried, if only to
                        // reach the ".add" files.
                        let langp_data = (*(*(*mip).mi_win).w_s).b_langp.ga_data as *mut langp_T;
                        let langp_len = (*(*(*mip).mi_win).w_s).b_langp.ga_len;
                        for lpi in 0..langp_len {
                            if (*slang).sl_nobreak {
                                (*mip).mi_lp = langp_data.offset(lpi as isize);
                                if (*(*(*mip).mi_lp).lp_slang).sl_fidxs.is_null()
                                    || !(*(*(*mip).mi_lp).lp_slang).sl_nobreak
                                {
                                    continue;
                                }
                            }

                            find_word(mip, FIND_COMPOUND);

                            // Under NOBREAK any match will do; otherwise the
                            // longest one is wanted, so the keep-case tree is
                            // tried as well.
                            if !(*slang).sl_nobreak || (*mip).mi_result == SP_BAD {
                                (*mip).mi_compoff = wlen;
                                find_word(mip, FIND_KEEPCOMPOUND);
                            }

                            if !(*slang).sl_nobreak {
                                break;
                            }
                        }
                        (*mip).mi_complen -= 1;
                        if flags & WF_COMPROOT != 0 {
                            (*mip).mi_compextra -= 1;
                        }
                        (*mip).mi_lp = save_lp;

                        if (*slang).sl_nobreak {
                            nobreak_result = (*mip).mi_result;
                            (*mip).mi_result = save_result;
                            (*mip).mi_end = save_end;
                        } else if (*mip).mi_result == SP_OK {
                            break 'variants;
                        } else {
                            break 'variant;
                        }
                    }

                    let res = if flags & WF_BANNED != 0 {
                        SP_BANNED
                    } else if flags & WF_REGION != 0 {
                        if (*(*mip).mi_lp).lp_region as c_uint & (flags >> 16) != 0 {
                            SP_OK
                        } else {
                            SP_LOCAL
                        }
                    } else if flags & WF_RARE != 0 {
                        SP_RARE
                    } else {
                        SP_OK
                    };

                    // Keep the longest match with the best result. NOBREAK keeps
                    // the longest match *without* a following good word
                    // separately, as a fall-back.
                    let end = (*mip).mi_word.offset(wlen as isize);
                    if nobreak_result == SP_BAD {
                        if (*mip).mi_result2 > res {
                            (*mip).mi_result2 = res;
                            (*mip).mi_end2 = end;
                        } else if (*mip).mi_result2 == res && (*mip).mi_end2 < end {
                            (*mip).mi_end2 = end;
                        }
                    } else if (*mip).mi_result > res {
                        (*mip).mi_result = res;
                        (*mip).mi_end = end;
                    } else if (*mip).mi_result == res && (*mip).mi_end < end {
                        (*mip).mi_end = end;
                    }

                    if (*mip).mi_result == SP_OK {
                        break 'variants;
                    }
                }
                len -= 1;
                arridx += 1;
            }

            if (*mip).mi_result == SP_OK {
                break;
            }
        }
    }
}

/// Whether this word may be one part of a compound, given the flags the
/// tree recorded for it and the parts collected in `mi_compflags` so far.
///
/// Split out of [`find_word`] only to keep that function readable; it is
/// the body of its `FIND_COMPOUND` arm and updates `mi_compflags` on the
/// way through.
#[inline]
unsafe fn compound_part_allowed(
    mip: *mut matchinf_T,
    ptr: *mut c_char,
    wlen: c_int,
    flags: uint32_t,
    word_ends: bool,
    mode: c_int,
    endlen: c_int,
) -> bool {
    unsafe {
        let slang = (*(*mip).mi_lp).lp_slang;
        // No compound flag, or shorter than COMPOUNDMIN, rejects quickly.
        // (Myspell compatibility requires accepting a compound flag on a
        // word that is too short to use it.)
        if flags >> 24 == 0 || wlen - (*mip).mi_compoff < (*slang).sl_compminlen {
            return false;
        }
        // COMPOUNDMIN counts characters, not bytes.
        if (*slang).sl_compminlen > 0
            && mb_charlen_len(
                (*mip).mi_word.offset((*mip).mi_compoff as isize),
                wlen - (*mip).mi_compoff,
            ) < (*slang).sl_compminlen
        {
            return false;
        }

        // COMPOUNDWORDMAX caps the number of parts, unless a syllable
        // maximum was given instead.
        if !word_ends
            && (*mip).mi_complen + (*mip).mi_compextra + 2 > (*slang).sl_compmax
            && (*slang).sl_compsylmax == MAXWLEN as c_int
        {
            return false;
        }

        // Compounding is not allowed on a side where an affix was added,
        // unless COMPOUNDPERMITFLAG said so.
        if (*mip).mi_complen > 0 && flags & WF_NOCOMPBEF != 0 {
            return false;
        }
        if !word_ends && flags & WF_NOCOMPAFT != 0 {
            return false;
        }

        // Is this flag usable in this position at all?
        let usable = if (*mip).mi_complen == 0 {
            (*slang).sl_compstartflags
        } else {
            (*slang).sl_compallflags
        };
        if !byte_in_str(usable, (flags >> 24) as c_int) {
            return false;
        }

        if match_checkcompoundpattern(ptr, wlen, &raw mut (*slang).sl_comppat) {
            return false;
        }

        if mode == FIND_COMPOUND {
            // Check the capitalisation of the part being appended.
            let mut p;
            if strncmp(ptr, (*mip).mi_word, (*mip).mi_compoff as usize) != 0 {
                // Folding changed the length.
                p = (*mip).mi_word;
                let end = ptr.offset((*mip).mi_compoff as isize);
                let mut s = ptr;
                while s < end {
                    mb_ptr_adv!(s);
                    mb_ptr_adv!(p);
                }
            } else {
                p = (*mip).mi_word.offset((*mip).mi_compoff as isize);
            }
            let capflags = captype(p, (*mip).mi_word.offset(wlen as isize));
            if capflags == WF_KEEPCAP as c_int
                || (capflags == WF_ALLCAP as c_int && flags & WF_FIXCAP != 0)
            {
                return false;
            }

            if capflags != WF_ALLCAP as c_int {
                // A Onecap part is not accepted after a word character. A
                // no-caps part is accepted even where the dictionary word
                // says ONECAP.
                p = p.offset(-(utf_head_off((*mip).mi_word, p.offset(-1)) as isize + 1));
                let reject = if spell_iswordp_nmw(p, (*mip).mi_win) {
                    capflags == WF_ONECAP as c_int
                } else {
                    flags & WF_ONECAP != 0 && capflags != WF_ONECAP as c_int
                };
                if reject {
                    return false;
                }
            }
        }

        // Record this part's flag, then check the sequence: against
        // COMPOUNDRULE for a complete word, or against the rules' prefixes
        // for a word still being built.
        (*mip).mi_compflags[(*mip).mi_complen as usize] = (flags >> 24) as uint8_t;
        (*mip).mi_compflags[((*mip).mi_complen + 1) as usize] = NUL as uint8_t;
        if word_ends {
            let mut fword = [0 as c_char; MAXWLEN];

            if (*slang).sl_compsylmax < MAXWLEN as c_int {
                // Only syllable counting needs the word itself.
                if ptr == (*mip).mi_word {
                    spell_casefold(
                        (*mip).mi_win,
                        ptr,
                        wlen,
                        fword.as_mut_ptr(),
                        MAXWLEN as c_int,
                    );
                } else {
                    xmemcpyz(
                        fword.as_mut_ptr() as *mut ::core::ffi::c_void,
                        ptr as *const ::core::ffi::c_void,
                        endlen as usize,
                    );
                }
            }
            if !can_compound(slang, fword.as_ptr(), (*mip).mi_compflags.as_ptr()) {
                return false;
            }
        } else if !(*slang).sl_comprules.is_null()
            && !match_compoundrule(slang, (*mip).mi_compflags.as_ptr())
        {
            return false;
        }

        true
    }
}

/// Whether joining another word onto `ptr[..wlen]` would hit a
/// CHECKCOMPOUNDPATTERN rule, which forbids the join.
///
/// A rule is a pair: the first part has to match at the end of the word so
/// far, the second at the start of what follows.
pub unsafe fn match_checkcompoundpattern(
    ptr: *mut c_char,
    wlen: c_int,
    gap: *mut garray_T,
) -> bool {
    unsafe {
        let pats = (*gap).ga_data as *mut *mut c_char;
        let mut i = 0;
        while i + 1 < (*gap).ga_len {
            let second = *pats.offset((i + 1) as isize);
            if strncmp(ptr.offset(wlen as isize), second, strlen(second)) == 0 {
                let first = *pats.offset(i as isize);
                let len = strlen(first) as c_int;
                if len <= wlen
                    && strncmp(ptr.offset((wlen - len) as isize), first, len as usize) == 0
                {
                    return true;
                }
            }
            i += 2;
        }
        false
    }
}

/// Whether `flags` is a sequence of compound flags the language allows and
/// `word` does not have too many syllables.
///
/// The COMPOUNDRULE patterns are compiled into one regexp at load time, so
/// the check is: widen the one-byte flags to characters and match. Syllable
/// counting is much slower, so it is left until last, and a word over
/// COMPOUNDSYLMAX is still accepted while it has fewer parts than
/// COMPOUNDWORDMAX.
pub unsafe fn can_compound(
    slang: *mut slang_T,
    word: *const c_char,
    flags: *const uint8_t,
) -> bool {
    unsafe {
        if (*slang).sl_compprog.is_null() {
            return false;
        }

        let mut uflags = [0 as c_char; MAXWLEN * 2];
        let mut p = uflags.as_mut_ptr();
        let mut i = 0;
        while *flags.offset(i) != 0 {
            p = p.offset(utf_char2bytes(*flags.offset(i) as c_int, p) as isize);
            i += 1;
        }
        *p = NUL as c_char;

        if !vim_regexec_prog(&raw mut (*slang).sl_compprog, false, uflags.as_ptr(), 0) {
            return false;
        }

        if (*slang).sl_compsylmax < MAXWLEN as c_int
            && count_syllables(slang, word) > (*slang).sl_compsylmax
        {
            return (strlen(flags as *const c_char) as c_int) < (*slang).sl_compmax;
        }
        true
    }
}

/// Whether the compound flags collected so far are a prefix of any
/// COMPOUNDRULE, so that it is still worth extending the compound.
///
/// The caller must have checked that `sl_comprules` is not null. A rule is
/// a sequence of flags, `[abc]` standing for any one of them, with `/`
/// separating rules.
pub unsafe fn match_compoundrule(slang: *mut slang_T, compflags: *const uint8_t) -> bool {
    unsafe {
        let mut p = (*slang).sl_comprules as *mut c_char;
        while *p != 0 {
            let mut i = 0;
            loop {
                let c = *compflags.offset(i) as c_int;
                if c == NUL {
                    // Every flag so far matched, and the rule has more to
                    // give: this compound can still work out.
                    return true;
                }
                if *p == b'/' as c_char || *p == 0 {
                    break; // the rule ran out first
                }
                if *p == b'[' as c_char {
                    let mut matched = false;
                    p = p.offset(1);
                    while *p != b']' as c_char && *p != 0 {
                        if *p as uint8_t as c_int == c {
                            matched = true;
                        }
                        p = p.offset(1);
                    }
                    if !matched {
                        break;
                    }
                } else if *p as uint8_t as c_int != c {
                    break;
                }
                p = p.offset(1);
                i += 1;
            }

            // On to the next rule.
            p = vim_strchr(p, '/' as c_int);
            if p.is_null() {
                break;
            }
        }
        false
    }
}

/// Whether one of the `totprefcnt` prefixes listed at `arridx` in
/// `sl_pidxs` may be used with the word `word` carrying `flags`.
///
/// Returns the prefix's own flags (non-zero) on a match, including
/// `WF_RAREPFX` when the prefix is rare, or zero when none applies.
///
/// A prefix entry packs its ID in the low byte and the index of its
/// condition regexp in the two bytes above.
pub unsafe fn valid_word_prefix(
    totprefcnt: c_int,
    arridx: c_int,
    flags: c_int,
    word: *mut c_char,
    slang: *mut slang_T,
    cond_req: bool,
) -> c_int {
    unsafe {
        let prefid = (flags as c_uint >> 24) as c_int;
        for prefcnt in (0..totprefcnt).rev() {
            let pidx = *(*slang).sl_pidxs.offset((arridx + prefcnt) as isize);

            if prefid != pidx & 0xff {
                continue;
            }

            // A non-combining prefix cannot go on a word that already has a
            // suffix.
            if flags & WF_HAS_AFF as c_int != 0 && pidx & WF_PFX_NC as c_int != 0 {
                continue;
            }

            let rp: *mut *mut regprog_T = (*slang)
                .sl_prefprog
                .offset(((pidx as c_uint >> 8) & 0xffff) as isize);
            if !(*rp).is_null() {
                if !vim_regexec_prog(rp, false, word, 0) {
                    continue;
                }
            } else if cond_req {
                continue;
            }

            return pidx;
        }
        0
    }
}

/// Check whether the word at `mip.mi_word` starts with a known prefix, and
/// if so look the remainder up with [`find_word`].
///
/// `FIND_COMPOUND` does the same after the compound parts found so far.
pub(super) unsafe fn find_prefix(mip: *mut matchinf_T, mode: c_int) {
    unsafe {
        let slang = (*(*mip).mi_lp).lp_slang;
        let byts = (*slang).sl_pbyts;
        if byts.is_null() {
            return; // this language has no prefixes
        }
        let idxs = (*slang).sl_pidxs;

        // Prefixes are always stored case-folded.
        let mut ptr = &raw mut (*mip).mi_fword as *mut c_char;
        let mut flen = (*mip).mi_fwordlen;
        if mode == FIND_COMPOUND {
            ptr = ptr.offset((*mip).mi_compoff as isize);
            flen -= (*mip).mi_compoff;
        }

        let mut arridx: idx_T = 0;
        let mut wlen = 0;
        loop {
            if flen == 0 && *(*mip).mi_fend != 0 {
                flen = fold_more(mip);
            }

            let mut len = *byts.offset(arridx as isize) as c_int;
            arridx += 1;

            // A leading zero byte means a prefix ends here. Several prefixes
            // can share a spelling with different conditions, and which one
            // gives the longest match is not known yet, so find_word() gets
            // the whole list to try.
            if *byts.offset(arridx as isize) == 0 {
                (*mip).mi_prefarridx = arridx;
                (*mip).mi_prefcnt = len;
                while len > 0 && *byts.offset(arridx as isize) == 0 {
                    arridx += 1;
                    len -= 1;
                }
                (*mip).mi_prefcnt -= len;

                (*mip).mi_prefixlen = wlen;
                if mode == FIND_COMPOUND {
                    (*mip).mi_prefixlen += (*mip).mi_compoff;
                }

                // The folded length may differ from the original.
                (*mip).mi_cprefixlen = nofold_len(
                    &raw mut (*mip).mi_fword as *mut c_char,
                    (*mip).mi_prefixlen,
                    (*mip).mi_word,
                );
                find_word(mip, FIND_PREFIX);

                if len == 0 {
                    break; // no children, the prefix must end here
                }
            }

            if *ptr.offset(wlen as isize) == 0 {
                break; // end of the line
            }

            let c = *ptr.offset(wlen as isize) as uint8_t as c_int;
            let mut lo = arridx;
            let mut hi = arridx + len - 1;
            while lo < hi {
                let m = (lo + hi) / 2;
                let b = *byts.offset(m as isize) as c_int;
                if b > c {
                    hi = m - 1;
                } else if b < c {
                    lo = m + 1;
                } else {
                    lo = m;
                    hi = m;
                    break;
                }
            }
            if hi < lo || *byts.offset(lo as isize) as c_int != c {
                break;
            }

            arridx = *idxs.offset(lo as isize);
            wlen += 1;
            flen -= 1;
        }
    }
}

/// Fold one more character of the word being checked into `mi_fword`, and
/// return how many bytes that added.
///
/// Folding runs to the next non-word character rather than one character at
/// a time, and includes that character, so that the caller can see where
/// the word ends.
unsafe fn fold_more(mip: *mut matchinf_T) -> c_int {
    unsafe {
        let p = (*mip).mi_fend;
        loop {
            mb_ptr_adv!((*mip).mi_fend);
            if *(*mip).mi_fend == 0 || !spell_iswordp((*mip).mi_fend, (*mip).mi_win) {
                break;
            }
        }

        // Include the non-word character, so the word end is visible.
        if *(*mip).mi_fend != 0 {
            mb_ptr_adv!((*mip).mi_fend);
        }

        let fwordlen = (*mip).mi_fwordlen;
        let tail = (&raw mut (*mip).mi_fword as *mut c_char).offset(fwordlen as isize);
        spell_casefold(
            (*mip).mi_win,
            p,
            (*mip).mi_fend.offset_from(p) as c_int,
            tail,
            MAXWLEN as c_int - fwordlen,
        );
        let flen = strlen(tail) as c_int;
        (*mip).mi_fwordlen += flen;
        flen
    }
}
