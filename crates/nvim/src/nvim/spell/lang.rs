//! Turning `'spelllang'` into a list of loaded languages.
//!
//! `'spelllang'` is a comma-separated list of names — `en`, `en_us`,
//! `de`, `cjk`, or a path ending in `.spl`. [`parse_spelllang`] resolves
//! each of them to a [`slang_T`], loading it from `'runtimepath'` if it is
//! not already in the global chain, and leaves the result in the window's
//! `b_langp` as a list of [`langp_T`]: a language plus the region mask
//! selected for it.
//!
//! `'spellfile'` entries are appended to the same list, along with the
//! internal word list `zg` writes to.
//!
//! # Regions
//!
//! A name like `en_us` asks for one region of a language whose `.spl` file
//! covers several. The region becomes a bit in `lp_region`, checked against
//! each word's own region mask at lookup time. A region named in
//! `'spelllang'` is also used for `'spellfile'` entries, but only while
//! every name agrees on it.
//!
//! # Borrowing between languages
//!
//! Sound folding and REP items are expensive to define and often missing
//! from a derived language. After the list is built, each entry that lacks
//! them borrows from the first entry whose name starts with the same two
//! letters — so `en-math` uses `en`'s.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use crate::src::nvim::autocmd::{EVENT_SPELLFILEMISSING, apply_autocmds};
use crate::src::nvim::buffer::{bufref_valid, set_bufref};
use crate::src::nvim::charset::vim_is_fname_char;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curbuf, curwin, e_invarg, firstbuf, firstwin, p_enc, starting};
use crate::src::nvim::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmemcpyz, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::smsg;
use crate::src::nvim::option::{copy_option_part, valid_name};
use crate::src::nvim::os::fs::os_remove;
use crate::src::nvim::os::libc::{
    gettext, memmove, snprintf, strcasecmp, strcmp, strcpy, strlen, strncmp,
};
use crate::src::nvim::path::{path_fnamecmp, path_full_compare, path_tail};
use crate::src::nvim::regexp::{vim_regcomp, vim_regfree};
use crate::src::nvim::spellfile::spell_load_file;
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::types::{
    bufref_T, garray_T, langp_T, regprog_T, size_t, slang_T, synblock_T, win_T,
};
use crate::src::nvim::window::win_valid_any_tab;

use super::chartab::init_spell_chartab;
use super::slang::slang_free;
use super::{
    DIP_ALL, MAXPATHL, MAXWLEN, NUL, RE_MAGIC, REGION_ALL, SPL_FNAME_TMPL, first_lang,
    int_wordlist, kEqualFiles, repl_from, repl_to, spelload_T,
};

/// `ASCII_ISALPHA`: an unaccented Latin letter.
fn ascii_isalpha(c: c_int) -> bool {
    // Unsigned, so that a negative byte fails both ranges rather than
    // wrapping into one of them.
    let c = c as core::ffi::c_uint;
    (c >= 'A' as core::ffi::c_uint && c <= 'Z' as core::ffi::c_uint)
        || (c >= 'a' as core::ffi::c_uint && c <= 'z' as core::ffi::c_uint)
}

/// The encoding spell files are named after: `'encoding'`, except that
/// `latin9` uses `latin1`'s files, and anything implausibly long falls back
/// to `latin1`.
pub unsafe fn spell_enc() -> *mut c_char {
    unsafe {
        if strlen(p_enc.get()) < 60 && strcmp(p_enc.get(), c"iso-8859-15".as_ptr()) != 0 {
            return p_enc.get();
        }
        c"latin1".as_ptr() as *mut c_char
    }
}

/// The `.spl` file name for the internal word list, into `fname[MAXPATHL]`.
unsafe fn int_wordlist_spl(fname: *mut c_char) {
    unsafe {
        vim_snprintf(
            fname,
            MAXPATHL as size_t,
            SPL_FNAME_TMPL.as_ptr(),
            int_wordlist.get(),
            spell_enc(),
        );
    }
}

/// Load every spell file for language `lang` (a name without a region)
/// found in `'runtimepath'`, then every `.add.spl` alongside them.
///
/// When nothing is found, a `SpellFileMissing` autocommand gets one chance
/// to produce the file, and the whole search is retried. Failing that, at
/// startup an autocommand is queued to offer downloading it, and otherwise
/// a warning is printed.
unsafe fn spell_load_lang(lang: *mut c_char) {
    unsafe {
        let mut fname_enc = [0 as c_char; 85];
        let mut sl: spelload_T = core::mem::zeroed();

        // The name is passed to spell_load_cb() as a cookie, and truncated
        // there when an error is found.
        strcpy(sl.sl_lang.as_mut_ptr(), lang);
        sl.sl_slang = core::ptr::null_mut();
        sl.sl_nobreak = 0;

        // Autocommands could otherwise delete the buffer and free "lang".
        (*curbuf.get()).b_locked += 1;

        let mut r = 0;
        for round in 1..=2 {
            vim_snprintf(
                fname_enc.as_mut_ptr(),
                fname_enc.len() as size_t - 5,
                c"spell/%s.%s.spl".as_ptr(),
                lang,
                spell_enc(),
            );
            r = do_in_runtimepath_cb(fname_enc.as_mut_ptr(), 0, &raw mut sl);

            if r == FAIL_I && sl.sl_lang[0] != 0 {
                // Fall back on the ASCII version.
                vim_snprintf(
                    fname_enc.as_mut_ptr(),
                    fname_enc.len() as size_t - 5,
                    c"spell/%s.ascii.spl".as_ptr(),
                    lang,
                );
                r = do_in_runtimepath_cb(fname_enc.as_mut_ptr(), 0, &raw mut sl);

                if r == FAIL_I
                    && sl.sl_lang[0] != 0
                    && round == 1
                    && apply_autocmds(
                        EVENT_SPELLFILEMISSING,
                        lang,
                        (*curbuf.get()).b_fname,
                        false,
                        curbuf.get(),
                    )
                {
                    continue;
                }
            }
            break;
        }

        if r == FAIL_I {
            if starting.get() != 0 {
                // Plugins are not loaded yet, so nvim/spellfile.lua cannot
                // offer the download itself. #3027
                let mut autocmd_buf = [0 as c_char; 512];
                snprintf(
                    autocmd_buf.as_mut_ptr(),
                    autocmd_buf.len(),
                    c"autocmd VimEnter * call v:lua.require'nvim.spellfile'.get('%s')|set spell"
                        .as_ptr(),
                    lang,
                );
                do_cmdline_cmd(autocmd_buf.as_ptr());
            } else {
                smsg(
                    0,
                    gettext(
                        c"Warning: Cannot find word list \"%s.%s.spl\" or \"%s.ascii.spl\""
                            .as_ptr(),
                    ),
                    lang,
                    spell_enc(),
                    lang,
                );
            }
        } else if !sl.sl_slang.is_null() {
            // At least one file loaded; now take all the additions.
            strcpy(
                fname_enc.as_mut_ptr().add(strlen(fname_enc.as_ptr()) - 3),
                c"add.spl".as_ptr(),
            );
            do_in_runtimepath_cb(fname_enc.as_mut_ptr(), DIP_ALL as c_int, &raw mut sl);
        }

        (*curbuf.get()).b_locked -= 1;
    }
}

const FAIL_I: c_int = 0;

/// `do_in_runtimepath` with [`spell_load_cb`] as the callback.
unsafe fn do_in_runtimepath_cb(name: *mut c_char, flags: c_int, sl: *mut spelload_T) -> c_int {
    unsafe {
        crate::src::nvim::runtime::do_in_runtimepath(
            name,
            flags,
            Some(spell_load_cb),
            sl as *mut c_void,
        )
    }
}

/// Load the spell files `do_in_runtimepath` found, keeping the last one in
/// the cookie.
///
/// NOBREAK is sticky in both directions: a `.add` file inherits it from the
/// base language, and a base language that declares it passes it on.
unsafe extern "C" fn spell_load_cb(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    unsafe {
        let slp = cookie as *mut spelload_T;
        for i in 0..num_fnames {
            let slang = spell_load_file(
                *fnames.offset(i as isize),
                (*slp).sl_lang.as_mut_ptr(),
                core::ptr::null_mut(),
                false,
            );
            if slang.is_null() {
                continue;
            }

            if (*slp).sl_nobreak != 0 && (*slang).sl_add {
                (*slang).sl_nobreak = true;
            } else if (*slang).sl_nobreak {
                (*slp).sl_nobreak = 1;
            }

            (*slp).sl_slang = slang;

            if !all {
                break;
            }
        }

        num_fnames > 0
    }
}

/// Guard against re-entering [`parse_spelllang`]: a `SpellFileMissing`
/// autocommand can open a new buffer with `'spell'` set.
static recursive: GlobalCell<bool> = GlobalCell::new(false);

/// Parse `'spelllang'` and fill `wp->w_s->b_langp`.
///
/// Returns null on success, or an untranslated error message.
pub unsafe fn parse_spelllang(wp: *mut win_T) -> *mut c_char {
    unsafe {
        if recursive.get() {
            return core::ptr::null_mut();
        }
        recursive.set(true);

        let mut region_cp = [0 as c_char; 3];
        let mut lang = [0 as c_char; MAXWLEN + 1];
        let mut spf_name = [0 as c_char; MAXPATHL as usize];
        let mut use_region: *mut c_char = core::ptr::null_mut();
        let mut dont_use_region = false;
        let mut nobreak = false;
        let mut ret_msg: *mut c_char = core::ptr::null_mut();

        let mut bufref: bufref_T = core::mem::zeroed();
        set_bufref(&raw mut bufref, (*wp).w_buffer);

        let mut ga: garray_T = core::mem::zeroed();
        ga_init(&raw mut ga, size_of::<langp_T>() as c_int, 2);
        clear_midword(wp);

        // The SpellFileMissing autocommands may change 'spelllang' underfoot.
        let spl_copy = xstrdup((*(*wp).w_s).b_p_spl);

        (*(*wp).w_s).b_cjk = 0;

        let mut splp = spl_copy;
        'names: while *splp != 0 {
            let len = copy_option_part(
                &raw mut splp,
                lang.as_mut_ptr(),
                MAXWLEN as size_t,
                c",".as_ptr() as *mut c_char,
            ) as c_int;
            let mut region: *mut c_char = core::ptr::null_mut();

            if !valid_spelllang(lang.as_ptr()) {
                continue;
            }

            if strcmp(lang.as_ptr(), c"cjk".as_ptr()) == 0 {
                (*(*wp).w_s).b_cjk = 1;
                continue;
            }

            let mut slang: *mut slang_T;
            let filename;
            if len > 4
                && path_fnamecmp(lang.as_ptr().offset(len as isize - 4), c".spl".as_ptr()) == 0
            {
                // The name is a file name; a region in it is pulled out.
                filename = true;

                let p = vim_strchr(path_tail(lang.as_mut_ptr()), '_' as c_int);
                if !p.is_null()
                    && ascii_isalpha(*p.offset(1) as c_int)
                    && ascii_isalpha(*p.offset(2) as c_int)
                    && !ascii_isalpha(*p.offset(3) as c_int)
                {
                    xstrlcpy(region_cp.as_mut_ptr(), p.offset(1), 3);
                    memmove(
                        p as *mut c_void,
                        p.offset(3) as *const c_void,
                        (len as isize - p.offset_from(lang.as_ptr()) - 2) as size_t,
                    );
                    region = region_cp.as_mut_ptr();
                } else {
                    dont_use_region = true;
                }

                slang = first_lang.get();
                while !slang.is_null() {
                    if path_full_compare(lang.as_mut_ptr(), (*slang).sl_fname, false, true)
                        == kEqualFiles
                    {
                        break;
                    }
                    slang = (*slang).sl_next;
                }
            } else {
                filename = false;
                if len > 3 && lang[(len - 3) as usize] == b'_' as c_char {
                    region = lang.as_mut_ptr().offset(len as isize - 2);
                    lang[(len - 3) as usize] = NUL as c_char;
                } else {
                    dont_use_region = true;
                }

                slang = first_lang.get();
                while !slang.is_null() {
                    if strcasecmp(lang.as_ptr(), (*slang).sl_name) == 0 {
                        break;
                    }
                    slang = (*slang).sl_next;
                }
            }

            if !region.is_null() {
                // A region that disagrees with an earlier one disqualifies
                // regions for 'spellfile'.
                if !use_region.is_null() && strcmp(region, use_region) != 0 {
                    dont_use_region = true;
                }
                use_region = region;
            }

            // Not loaded yet: load it now.
            if slang.is_null() {
                if filename {
                    spell_load_file(
                        lang.as_mut_ptr(),
                        lang.as_mut_ptr(),
                        core::ptr::null_mut(),
                        false,
                    );
                } else {
                    spell_load_lang(lang.as_mut_ptr());
                    // The autocommands may have destroyed the buffer being
                    // used, or closed the window.
                    if !bufref_valid(&raw mut bufref) || !win_valid_any_tab(wp) {
                        ret_msg = c"E797: SpellFileMissing autocommand deleted buffer".as_ptr()
                            as *mut c_char;
                        break 'names;
                    }
                }
            }

            // There can be several files for one language.
            slang = first_lang.get();
            while !slang.is_null() {
                let matches = if filename {
                    path_full_compare(lang.as_mut_ptr(), (*slang).sl_fname, false, true)
                        == kEqualFiles
                } else {
                    strcasecmp(lang.as_ptr(), (*slang).sl_name) == 0
                };
                if matches {
                    let mut region_mask = REGION_ALL;
                    if !filename && !region.is_null() {
                        let c = find_region((*slang).sl_regions.as_ptr(), region);
                        if c == REGION_ALL {
                            if (*slang).sl_add {
                                if (*slang).sl_regions[0] != 0 {
                                    // This addition file covers other regions.
                                    region_mask = 0;
                                }
                            } else {
                                // Probably a mistake; warn but accept the
                                // words anyway.
                                smsg(
                                    0,
                                    gettext(c"Warning: region %s not supported".as_ptr()),
                                    region,
                                );
                            }
                        } else {
                            region_mask = 1 << c;
                        }
                    }

                    if region_mask != 0 {
                        let p_ =
                            ga_append_via_ptr(&raw mut ga, size_of::<langp_T>()) as *mut langp_T;
                        (*p_).lp_slang = slang;
                        (*p_).lp_region = region_mask;

                        use_midword(slang, wp);
                        if (*slang).sl_nobreak {
                            nobreak = true;
                        }
                    }
                }
                slang = (*slang).sl_next;
            }
        }

        if ret_msg.is_null() {
            // Round 0 is the internal word list; each round after that is one
            // entry of 'spellfile'.
            let mut spf = (*(*curwin.get()).w_s).b_p_spf;
            let mut round = 0;
            while round == 0 || *spf != 0 {
                if round == 0 {
                    if (*int_wordlist.ptr()).is_null() {
                        round += 1;
                        continue;
                    }
                    int_wordlist_spl(spf_name.as_mut_ptr());
                } else {
                    let len = copy_option_part(
                        &raw mut spf,
                        spf_name.as_mut_ptr(),
                        MAXPATHL as size_t - 4,
                        c",".as_ptr() as *mut c_char,
                    ) as c_int;
                    strcpy(spf_name.as_mut_ptr().offset(len as isize), c".spl".as_ptr());

                    // Skip it if the loop above already took it.
                    let mut c = 0;
                    while c < ga.ga_len {
                        let p =
                            (*(*(ga.ga_data as *mut langp_T).offset(c as isize)).lp_slang).sl_fname;
                        if !p.is_null()
                            && path_full_compare(spf_name.as_mut_ptr(), p, false, true)
                                == kEqualFiles
                        {
                            break;
                        }
                        c += 1;
                    }
                    if c < ga.ga_len {
                        round += 1;
                        continue;
                    }
                }

                let mut slang = first_lang.get();
                while !slang.is_null() {
                    if path_full_compare(spf_name.as_mut_ptr(), (*slang).sl_fname, false, true)
                        == kEqualFiles
                    {
                        break;
                    }
                    slang = (*slang).sl_next;
                }

                if slang.is_null() {
                    // The language name includes the region; the region is
                    // ignored for these. The internal word list gets an
                    // arbitrary name.
                    if round == 0 {
                        strcpy(lang.as_mut_ptr(), c"internal wordlist".as_ptr());
                    } else {
                        xstrlcpy(
                            lang.as_mut_ptr(),
                            path_tail(spf_name.as_mut_ptr()),
                            MAXWLEN + 1,
                        );
                        let p = vim_strchr(lang.as_mut_ptr(), '.' as c_int);
                        if !p.is_null() {
                            *p = NUL as c_char; // truncate at ".encoding.add"
                        }
                    }
                    slang = spell_load_file(
                        spf_name.as_mut_ptr(),
                        lang.as_mut_ptr(),
                        core::ptr::null_mut(),
                        true,
                    );

                    // If any language has NOBREAK assume the additions do too.
                    if !slang.is_null() && nobreak {
                        (*slang).sl_nobreak = true;
                    }
                }

                if !slang.is_null() {
                    let mut region_mask = REGION_ALL;
                    if !use_region.is_null() && !dont_use_region {
                        let c = find_region((*slang).sl_regions.as_ptr(), use_region);
                        if c != REGION_ALL {
                            region_mask = 1 << c;
                        } else if (*slang).sl_regions[0] != 0 {
                            // This spell file covers other regions.
                            region_mask = 0;
                        }
                    }

                    if region_mask != 0 {
                        let p_ =
                            ga_append_via_ptr(&raw mut ga, size_of::<langp_T>()) as *mut langp_T;
                        (*p_).lp_slang = slang;
                        (*p_).lp_sallang = core::ptr::null_mut();
                        (*p_).lp_replang = core::ptr::null_mut();
                        (*p_).lp_region = region_mask;

                        use_midword(slang, wp);
                    }
                }
                round += 1;
            }

            // Everything worked; publish the new list.
            ga_clear(&raw mut (*(*wp).w_s).b_langp);
            (*(*wp).w_s).b_langp = ga;

            // A language with no sound folding or no REP items of its own
            // borrows from the first similarly-named one that has them, so
            // that "en-math" gets "en"'s.
            let entries = ga.ga_data as *mut langp_T;
            for i in 0..ga.ga_len {
                let lp = entries.offset(i as isize);

                if (*(*lp).lp_slang).sl_sal.ga_len > 0 {
                    (*lp).lp_sallang = (*lp).lp_slang;
                } else {
                    for j in 0..ga.ga_len {
                        let lp2 = entries.offset(j as isize);
                        if (*(*lp2).lp_slang).sl_sal.ga_len > 0
                            && strncmp((*(*lp).lp_slang).sl_name, (*(*lp2).lp_slang).sl_name, 2)
                                == 0
                        {
                            (*lp).lp_sallang = (*lp2).lp_slang;
                            break;
                        }
                    }
                }

                if (*(*lp).lp_slang).sl_rep.ga_len > 0 {
                    (*lp).lp_replang = (*lp).lp_slang;
                } else {
                    for j in 0..ga.ga_len {
                        let lp2 = entries.offset(j as isize);
                        if (*(*lp2).lp_slang).sl_rep.ga_len > 0
                            && strncmp((*(*lp).lp_slang).sl_name, (*(*lp2).lp_slang).sl_name, 2)
                                == 0
                        {
                            (*lp).lp_replang = (*lp2).lp_slang;
                            break;
                        }
                    }
                }
            }
            redraw_later(wp, UPD_NOT_VALID);
        }

        xfree(spl_copy as *mut c_void);
        recursive.set(false);
        ret_msg
    }
}

/// Forget the midword characters recorded for `wp`.
fn clear_midword(wp: *mut win_T) {
    unsafe {
        (*(*wp).w_s).b_spell_ismw = [false; 256];
        xfree((*(*wp).w_s).b_spell_ismw_mb as *mut c_void);
        (*(*wp).w_s).b_spell_ismw_mb = core::ptr::null_mut();
    }
}

/// The index of region `region[..2]` in `rp` (which is `sl_regions`, two
/// characters per region), or `REGION_ALL` when it is not there.
unsafe fn find_region(rp: *const c_char, region: *const c_char) -> c_int {
    unsafe {
        let mut i = 0;
        loop {
            if *rp.offset(i as isize) == 0 {
                return REGION_ALL;
            }
            if *rp.offset(i as isize) == *region && *rp.offset(i as isize + 1) == *region.offset(1)
            {
                return i / 2;
            }
            i += 2;
        }
    }
}

/// Delete the internal word list and its compiled `.spl`.
pub unsafe fn spell_delete_wordlist() {
    unsafe {
        if (*int_wordlist.ptr()).is_null() {
            return;
        }

        let mut fname = [0 as c_char; MAXPATHL as usize];
        os_remove(int_wordlist.get());
        int_wordlist_spl(fname.as_mut_ptr());
        os_remove(fname.as_mut_ptr());
        xfree(int_wordlist.get() as *mut c_void);
        int_wordlist.set(core::ptr::null_mut());
    }
}

/// Free every loaded language and everything derived from them.
pub unsafe fn spell_free_all() {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            ga_clear(&raw mut (*buf).b_s.b_langp);
            buf = (*buf).b_next;
        }

        while !(*first_lang.ptr()).is_null() {
            let slang = first_lang.get();
            first_lang.set((*slang).sl_next);
            slang_free(slang);
        }

        spell_delete_wordlist();

        xfree(repl_to.get() as *mut c_void);
        repl_to.set(core::ptr::null_mut());
        xfree(repl_from.get() as *mut c_void);
        repl_from.set(core::ptr::null_mut());
    }
}

/// Drop every spelling table and load them again, after `'encoding'`
/// changed or `:mkspell` ran.
pub unsafe fn spell_reload() {
    unsafe {
        init_spell_chartab();
        spell_free_all();

        // Only load word lists where 'spelllang' is set and some window on
        // the buffer has 'spell' on. The walk is over the current tab, which
        // always starts at `firstwin`.
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if *(*(*wp).w_s).b_p_spl != 0 && (*wp).w_onebuf_opt.wo_spell != 0 {
                parse_spelllang(wp);
                break;
            }
            wp = (*wp).w_next;
        }
    }
}

/// Whether `val` is a usable `'spelllang'` value.
pub unsafe fn valid_spelllang(val: *const c_char) -> bool {
    unsafe { valid_name(val, c".-_,@".as_ptr()) }
}

/// Whether `val` is a usable `'spellfile'` value: a comma-separated list of
/// file names, each ending in `.add` and made of file-name characters.
pub unsafe fn valid_spellfile(val: *const c_char) -> bool {
    unsafe {
        let mut spf_name = [0 as c_char; MAXPATHL as usize];
        let mut spf = val as *mut c_char;
        while *spf != 0 {
            let l = copy_option_part(
                &raw mut spf,
                spf_name.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr() as *mut c_char,
            );
            if l >= MAXPATHL as size_t - 4
                || l < 4
                || strcmp(spf_name.as_ptr().add(l - 4), c".add".as_ptr()) != 0
            {
                return false;
            }
            let mut s = spf_name.as_ptr();
            while *s != 0 {
                if !vim_is_fname_char(*s as u8 as c_int) {
                    return false;
                }
                s = s.offset(1);
            }
        }
        true
    }
}

/// Re-parse `'spelllang'` for the current buffer after a spell option
/// changed.
pub unsafe fn did_set_spell_option() -> *const c_char {
    unsafe {
        let mut errmsg: *const c_char = core::ptr::null();
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if (*wp).w_buffer == curbuf.get() && (*wp).w_onebuf_opt.wo_spell != 0 {
                errmsg = parse_spelllang(wp);
                break;
            }
            wp = (*wp).w_next;
        }
        errmsg
    }
}

/// Compile `'spellcapcheck'` into `b_cap_prog`, anchored so that it can
/// only match at one column.
///
/// Returns an error message when the pattern does not compile, leaving the
/// previous program in place.
pub unsafe fn compile_cap_prog(synblock: *mut synblock_T) -> *const c_char {
    unsafe {
        let rp: *mut regprog_T = (*synblock).b_cap_prog;

        if (*synblock).b_p_spc.is_null() || *(*synblock).b_p_spc == 0 {
            (*synblock).b_cap_prog = core::ptr::null_mut();
        } else {
            let re = concat_str(c"^".as_ptr(), (*synblock).b_p_spc);
            (*synblock).b_cap_prog = vim_regcomp(re, RE_MAGIC as c_int);
            xfree(re as *mut c_void);
            if (*synblock).b_cap_prog.is_null() {
                (*synblock).b_cap_prog = rp; // keep the previous program
                return e_invarg.ptr() as *const c_char;
            }
        }

        vim_regfree(rp);
        core::ptr::null()
    }
}

/// Record `lp`'s `MIDWORD` characters in `wp`, so that [`spell_iswordp`]
/// treats them as part of a word when a word character follows.
///
/// Characters below 256 that take at most two bytes go in the flat
/// `b_spell_ismw` table; anything wider is appended to the
/// `b_spell_ismw_mb` string, which is scanned instead.
///
/// [`spell_iswordp`]: super::chartab::spell_iswordp
unsafe fn use_midword(lp: *mut slang_T, wp: *mut win_T) {
    unsafe {
        if (*lp).sl_midword.is_null() {
            return;
        }

        let mut p = (*lp).sl_midword;
        while *p != 0 {
            let c = utf_ptr2char(p);
            let l = utfc_ptr2len(p);
            if c < 256 && l <= 2 {
                (*(*wp).w_s).b_spell_ismw[c as usize] = true;
            } else if (*(*wp).w_s).b_spell_ismw_mb.is_null() {
                (*(*wp).w_s).b_spell_ismw_mb =
                    xmemdupz(p as *const c_void, l as size_t) as *mut c_char;
            } else {
                let n = strlen((*(*wp).w_s).b_spell_ismw_mb) as c_int;
                let bp = xstrnsave((*(*wp).w_s).b_spell_ismw_mb, (n + l) as size_t);
                xfree((*(*wp).w_s).b_spell_ismw_mb as *mut c_void);
                (*(*wp).w_s).b_spell_ismw_mb = bp;
                xmemcpyz(
                    bp.offset(n as isize) as *mut c_void,
                    p as *const c_void,
                    l as size_t,
                );
            }
            p = p.offset(l as isize);
        }
    }
}
