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

use crate::cstr;
use crate::message_fmt::c_str;
use crate::smsg;
use crate::types::AutoEvent;
use ::libc::strcasecmp;
use core::ffi::{CStr, c_char, c_int, c_void};

use crate::autocmd::apply_autocmds;
use crate::buffer::BufRef;
use crate::charset::vim_is_fname_char;
use crate::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, curwin, e_invarg, p_enc, starting};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memory::{xfree, xmemcpyz, xmemdupz, xstrdup, xstrlcpy};
use crate::option::{copy_option_part, valid_name};
use crate::os::cshim::snprintf;
use crate::os::fs::os_remove;
use crate::path::{path_fnamecmp, path_full_compare, path_tail};
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::spellfile::spell_load_file;
use crate::strings::{concat_str, vim_snprintf, vim_strchr, xstrnsave};
use crate::types::{
    Failed, MAXPATHL, NUL, SPL_FNAME_TMPL, garray_T, langp_T, regprog_T, size_t, slang_T,
    synblock_T, win_T,
};
use crate::window::win_valid_any_tab;

use super::chartab::init_spell_chartab;
use super::slang::slang_free;
use super::{
    MAXWLEN, REGION_ALL, first_lang, int_wordlist, kEqualFiles, repl_from, repl_to, spelload_T,
};
use crate::runtime::RuntimeOpts;
use crate::winlayer::{Buf, buffers, windows};

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
    if unsafe { cstr::bytes_at(p_enc.get()) }.len() < 60
        && unsafe { !cstr::eq_bytes(p_enc.get(), b"iso-8859-15") }
    {
        return p_enc.get();
    }
    c"latin1".as_ptr() as *mut c_char
}

/// The `.spl` file name for the internal word list, into `fname[MAXPATHL]`.
unsafe fn int_wordlist_spl(fname: *mut c_char) {
    let fmt = SPL_FNAME_TMPL.as_ptr();
    let (list, enc) = (int_wordlist.get(), unsafe { spell_enc() });
    unsafe { vim_snprintf(fname, MAXPATHL as size_t, fmt, list, enc) };
}

/// Load every spell file for language `lang` (a name without a region)
/// found in `'runtimepath'`, then every `.add.spl` alongside them.
///
/// When nothing is found, a `SpellFileMissing` autocommand gets one chance
/// to produce the file, and the whole search is retried. Failing that, at
/// startup an autocommand is queued to offer downloading it, and otherwise
/// a warning is printed.
unsafe fn spell_load_lang(lang: *mut c_char) {
    let mut fname_enc = [0 as c_char; 85];
    let mut sl: spelload_T = unsafe { core::mem::zeroed() };

    // The name is passed to spell_load_cb() as a cookie, and truncated
    // there when an error is found.
    let (into, room) = (sl.sl_lang.as_mut_ptr(), sl.sl_lang.len());
    unsafe { xstrlcpy(into, lang, room) };
    sl.sl_slang = core::ptr::null_mut();
    sl.sl_nobreak = 0;

    // Autocommands could otherwise delete the buffer and free "lang".
    unsafe { (*curbuf.get()).b_locked += 1 };

    let mut r = Err(Failed);
    for round in 1..=2 {
        let (buf, room) = (fname_enc.as_mut_ptr(), fname_enc.len() as size_t - 5);
        let fmt = c"spell/%s.%s.spl".as_ptr();
        let enc = unsafe { spell_enc() };
        unsafe { vim_snprintf(buf, room, fmt, lang, enc) };
        r = unsafe { do_in_runtimepath_cb(fname_enc.as_mut_ptr(), RuntimeOpts::NONE, &raw mut sl) };

        if r.is_err() && sl.sl_lang[0] != 0 {
            // Fall back on the ASCII version.
            let (buf, room) = (fname_enc.as_mut_ptr(), fname_enc.len() as size_t - 5);
            let fmt = c"spell/%s.ascii.spl".as_ptr();
            unsafe { vim_snprintf(buf, room, fmt, lang) };
            r = unsafe {
                do_in_runtimepath_cb(fname_enc.as_mut_ptr(), RuntimeOpts::NONE, &raw mut sl)
            };

            if r.is_err() && sl.sl_lang[0] != 0 && round == 1 && {
                let buf = curbuf.get();
                let fname = unsafe { (*buf).b_fname };
                let event = AutoEvent::SpellFileMissing;
                unsafe { apply_autocmds(event, lang, fname, false, buf) }
            } {
                continue;
            }
        }
        break;
    }

    if r.is_err() {
        if starting.get() != 0 {
            // Plugins are not loaded yet, so nvim/spellfile.lua cannot
            // offer the download itself. #3027
            let mut autocmd_buf = [0 as c_char; 512];
            let (buf, room) = (autocmd_buf.as_mut_ptr(), autocmd_buf.len());
            let fmt = c"autocmd VimEnter * call v:lua.require'nvim.spellfile'.get('%s')|set spell"
                .as_ptr();
            unsafe { snprintf(buf, room, fmt, lang) };
            let _ = unsafe { do_cmdline_cmd(autocmd_buf.as_ptr()) };
        } else {
            // SAFETY: the language name and the encoding are NUL-terminated.
            let (lang, enc) = unsafe { (c_str(lang), c_str(spell_enc())) };
            smsg!(
                0,
                "Warning: Cannot find word list \"{lang}.{enc}.spl\" or \"{lang}.ascii.spl\""
            );
        }
    } else if !sl.sl_slang.is_null() {
        // At least one file loaded; now take all the additions.
        let ptr_len = unsafe { cstr::bytes_at(fname_enc.as_ptr()) }.len();
        let at = unsafe { fname_enc.as_mut_ptr().add(ptr_len - 3) };
        unsafe { xstrlcpy(at, c"add.spl".as_ptr(), fname_enc.len() - (ptr_len - 3)) };
        let _ =
            unsafe { do_in_runtimepath_cb(fname_enc.as_mut_ptr(), RuntimeOpts::ALL, &raw mut sl) };
    }

    unsafe { (*curbuf.get()).b_locked -= 1 };
}

/// `do_in_runtimepath` with [`spell_load_cb`] as the callback.
unsafe fn do_in_runtimepath_cb(
    name: *mut c_char,
    flags: RuntimeOpts,
    sl: *mut spelload_T,
) -> Result<(), Failed> {
    unsafe {
        crate::runtime::do_in_runtimepath(name, flags, Some(spell_load_cb), sl as *mut c_void)
    }
}

/// Load the spell files `do_in_runtimepath` found, keeping the last one in
/// the cookie.
///
/// NOBREAK is sticky in both directions: a `.add` file inherits it from the
/// base language, and a base language that declares it passes it on.
unsafe fn spell_load_cb(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    let slp = cookie as *mut spelload_T;
    for i in 0..num_fnames {
        let fname = unsafe { *fnames.offset(i as isize) };
        let lang = unsafe { (*slp).sl_lang.as_mut_ptr() };
        let none = core::ptr::null_mut();
        let slang = unsafe { spell_load_file(fname, lang, none, false) };
        if slang.is_null() {
            continue;
        }

        if unsafe { (*slp).sl_nobreak } != 0 && unsafe { (*slang).sl_add } {
            unsafe { (*slang).sl_nobreak = true };
        } else if unsafe { (*slang).sl_nobreak } {
            unsafe { (*slp).sl_nobreak = 1 };
        }

        unsafe { (*slp).sl_slang = slang };

        if !all {
            break;
        }
    }

    num_fnames > 0
}

/// Guard against re-entering [`parse_spelllang`]: a `SpellFileMissing`
/// autocommand can open a new buffer with `'spell'` set.
static recursive: GlobalCell<bool> = GlobalCell::new(false);

/// Parse `'spelllang'` and fill `wp->w_s->b_langp`.
///
/// Returns null on success, or an untranslated error message.
pub unsafe fn parse_spelllang(wp: *mut win_T) -> Option<&'static CStr> {
    if recursive.get() {
        return None;
    }
    recursive.set(true);

    let mut region_cp = [0 as c_char; 3];
    let mut lang = [0 as c_char; MAXWLEN + 1];
    let mut spf_name = [0 as c_char; MAXPATHL as usize];
    let mut use_region: *mut c_char = core::ptr::null_mut();
    let mut dont_use_region = false;
    let mut nobreak = false;
    let mut ret_msg: Option<&'static CStr> = None;

    let bufref = BufRef::of_opt(unsafe { Buf::from_raw((*wp).w_buffer) });

    let mut ga: garray_T = unsafe { core::mem::zeroed() };
    unsafe { ga_init(&raw mut ga, size_of::<langp_T>() as c_int, 2) };
    clear_midword(wp);

    // The SpellFileMissing autocommands may change 'spelllang' underfoot.
    let spl_copy = unsafe { xstrdup((*(*wp).w_s).b_p_spl) };

    unsafe { (*(*wp).w_s).b_cjk = 0 };

    let mut splp = spl_copy;
    'names: while unsafe { *splp } != 0 {
        let (buf, room) = (lang.as_mut_ptr(), MAXWLEN as size_t);
        let sep = c",".as_ptr() as *mut c_char;
        let len = unsafe { copy_option_part(&raw mut splp, buf, room, sep) } as c_int;
        let mut region: *mut c_char = core::ptr::null_mut();

        if !valid_spelllang(cstr::in_chars(&lang)) {
            continue;
        }

        if unsafe { cstr::eq_bytes(lang.as_ptr(), b"cjk") } {
            unsafe { (*(*wp).w_s).b_cjk = 1 };
            continue;
        }

        let mut slang: *mut slang_T;
        let filename;
        if len > 4
            && unsafe { path_fnamecmp(lang.as_ptr().offset(len as isize - 4), c".spl".as_ptr()) }
                == 0
        {
            // The name is a file name; a region in it is pulled out.
            filename = true;

            let p = unsafe { vim_strchr(path_tail(lang.as_mut_ptr()), '_' as c_int) };
            if !p.is_null()
                && ascii_isalpha(unsafe { *p.offset(1) } as c_int)
                && ascii_isalpha(unsafe { *p.offset(2) } as c_int)
                && !ascii_isalpha(unsafe { *p.offset(3) } as c_int)
            {
                unsafe { xstrlcpy(region_cp.as_mut_ptr(), p.offset(1), 3) };
                let after = unsafe { p.offset(3) } as *const c_void;
                let rest = unsafe { len as isize - p.offset_from(lang.as_ptr()) - 2 };
                unsafe { p.cast::<u8>().copy_from(after.cast(), rest as size_t) };
                region = region_cp.as_mut_ptr();
            } else {
                dont_use_region = true;
            }

            slang = first_lang.get();
            while !slang.is_null() {
                if unsafe { path_full_compare(lang.as_mut_ptr(), (*slang).sl_fname, false, true) }
                    == kEqualFiles
                {
                    break;
                }
                slang = unsafe { (*slang).sl_next };
            }
        } else {
            filename = false;
            if len > 3 && lang[(len - 3) as usize] == b'_' as c_char {
                region = unsafe { lang.as_mut_ptr().offset(len as isize - 2) };
                lang[(len - 3) as usize] = NUL as c_char;
            } else {
                dont_use_region = true;
            }

            slang = first_lang.get();
            while !slang.is_null() {
                if unsafe { strcasecmp(lang.as_ptr(), (*slang).sl_name) } == 0 {
                    break;
                }
                slang = unsafe { (*slang).sl_next };
            }
        }

        if !region.is_null() {
            // A region that disagrees with an earlier one disqualifies
            // regions for 'spellfile'.
            if !use_region.is_null() && !unsafe { cstr::eq(region, use_region) } {
                dont_use_region = true;
            }
            use_region = region;
        }

        // Not loaded yet: load it now.
        if slang.is_null() {
            if filename {
                let name = lang.as_mut_ptr();
                unsafe { spell_load_file(name, name, core::ptr::null_mut(), false) };
            } else {
                unsafe { spell_load_lang(lang.as_mut_ptr()) };
                // The autocommands may have destroyed the buffer being
                // used, or closed the window.
                if !bufref.valid() || !win_valid_any_tab(wp) {
                    ret_msg = Some(c"E797: SpellFileMissing autocommand deleted buffer");
                    break 'names;
                }
            }
        }

        // There can be several files for one language.
        slang = first_lang.get();
        while !slang.is_null() {
            let matches = if filename {
                let fname = unsafe { (*slang).sl_fname };
                let cmp = unsafe { path_full_compare(lang.as_mut_ptr(), fname, false, true) };
                cmp == kEqualFiles
            } else {
                unsafe { strcasecmp(lang.as_ptr(), (*slang).sl_name) == 0 }
            };
            if matches {
                let mut region_mask = REGION_ALL;
                if !filename && !region.is_null() {
                    let c = unsafe { find_region((*slang).sl_regions.as_ptr(), region) };
                    if c == REGION_ALL {
                        if unsafe { (*slang).sl_add } {
                            if unsafe { (*slang).sl_regions[0] } != 0 {
                                // This addition file covers other regions.
                                region_mask = 0;
                            }
                        } else {
                            // SAFETY: a message argument the caller holds as a NUL-terminated string.
                            let region = unsafe { c_str(region) };
                            smsg!(0, "Warning: region {region} not supported");
                        }
                    } else {
                        region_mask = 1 << c;
                    }
                }

                if region_mask != 0 {
                    let p_ = unsafe { ga_append_via_ptr(&raw mut ga, size_of::<langp_T>()) }
                        as *mut langp_T;
                    unsafe { (*p_).lp_slang = slang };
                    unsafe { (*p_).lp_region = region_mask };

                    unsafe { use_midword(slang, wp) };
                    if unsafe { (*slang).sl_nobreak } {
                        nobreak = true;
                    }
                }
            }
            slang = unsafe { (*slang).sl_next };
        }
    }

    if ret_msg.is_none() {
        // Round 0 is the internal word list; each round after that is one
        // entry of 'spellfile'.
        let mut spf = unsafe { (*(*curwin.get()).w_s).b_p_spf };
        let mut round = 0;
        while round == 0 || unsafe { *spf } != 0 {
            if round == 0 {
                if int_wordlist.get().is_null() {
                    round += 1;
                    continue;
                }
                unsafe { int_wordlist_spl(spf_name.as_mut_ptr()) };
            } else {
                let (buf, room) = (spf_name.as_mut_ptr(), MAXPATHL as size_t - 4);
                let sep = c",".as_ptr() as *mut c_char;
                let len = unsafe { copy_option_part(&raw mut spf, buf, room, sep) } as c_int;
                let tail = unsafe { buf.offset(len as isize) };
                unsafe { xstrlcpy(tail, c".spl".as_ptr(), MAXPATHL as usize - len as usize) };

                // Skip it if the loop above already took it.
                let mut c = 0;
                while c < ga.ga_len {
                    let entry = ga.ga_data as *mut langp_T;
                    let p = unsafe { (*(*entry.offset(c as isize)).lp_slang).sl_fname };
                    if !p.is_null()
                        && unsafe { path_full_compare(spf_name.as_mut_ptr(), p, false, true) }
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
                let fname = unsafe { (*slang).sl_fname };
                let name = spf_name.as_mut_ptr();
                if unsafe { path_full_compare(name, fname, false, true) } == kEqualFiles {
                    break;
                }
                slang = unsafe { (*slang).sl_next };
            }

            if slang.is_null() {
                // The language name includes the region; the region is
                // ignored for these. The internal word list gets an
                // arbitrary name.
                if round == 0 {
                    let (into, name) = (lang.as_mut_ptr(), c"internal wordlist");
                    unsafe { xstrlcpy(into, name.as_ptr(), MAXWLEN + 1) };
                } else {
                    let tail = unsafe { path_tail(spf_name.as_mut_ptr()) };
                    unsafe { xstrlcpy(lang.as_mut_ptr(), tail, MAXWLEN + 1) };
                    let p = unsafe { vim_strchr(lang.as_mut_ptr(), '.' as c_int) };
                    if !p.is_null() {
                        unsafe { *p = NUL as c_char }; // truncate at ".encoding.add"
                    }
                }
                let (file, name) = (spf_name.as_mut_ptr(), lang.as_mut_ptr());
                slang = unsafe { spell_load_file(file, name, core::ptr::null_mut(), true) };

                // If any language has NOBREAK assume the additions do too.
                if !slang.is_null() && nobreak {
                    unsafe { (*slang).sl_nobreak = true };
                }
            }

            if !slang.is_null() {
                let mut region_mask = REGION_ALL;
                if !use_region.is_null() && !dont_use_region {
                    let c = unsafe { find_region((*slang).sl_regions.as_ptr(), use_region) };
                    if c != REGION_ALL {
                        region_mask = 1 << c;
                    } else if unsafe { (*slang).sl_regions[0] } != 0 {
                        // This spell file covers other regions.
                        region_mask = 0;
                    }
                }

                if region_mask != 0 {
                    let p_ = unsafe { ga_append_via_ptr(&raw mut ga, size_of::<langp_T>()) }
                        as *mut langp_T;
                    unsafe { (*p_).lp_slang = slang };
                    unsafe { (*p_).lp_sallang = core::ptr::null_mut() };
                    unsafe { (*p_).lp_replang = core::ptr::null_mut() };
                    unsafe { (*p_).lp_region = region_mask };

                    unsafe { use_midword(slang, wp) };
                }
            }
            round += 1;
        }

        // Everything worked; publish the new list.
        unsafe { ga_clear(&raw mut (*(*wp).w_s).b_langp) };
        unsafe { (*(*wp).w_s).b_langp = ga };

        // A language with no sound folding or no REP items of its own
        // borrows from the first similarly-named one that has them, so
        // that "en-math" gets "en"'s.
        let entries = ga.ga_data as *mut langp_T;
        for i in 0..ga.ga_len {
            let lp = unsafe { entries.offset(i as isize) };
            // The first two bytes of `sl_name` are the language; a region
            // suffix past them does not have to match.
            let lang = unsafe { (*(*lp).lp_slang).sl_name };

            if unsafe { (*(*lp).lp_slang).has_soundfold() } {
                unsafe { (*lp).lp_sallang = (*lp).lp_slang };
            } else {
                for j in 0..ga.ga_len {
                    let lp2 = unsafe { entries.offset(j as isize) };
                    let lang2 = unsafe { (*(*lp2).lp_slang).sl_name };
                    if unsafe { (*(*lp2).lp_slang).has_soundfold() }
                        && unsafe { cstr::prefix_eq(lang, lang2, 2) }
                    {
                        unsafe { (*lp).lp_sallang = (*lp2).lp_slang };
                        break;
                    }
                }
            }

            if unsafe { !(*(*lp).lp_slang).sl_rep.is_empty() } {
                unsafe { (*lp).lp_replang = (*lp).lp_slang };
            } else {
                for j in 0..ga.ga_len {
                    let lp2 = unsafe { entries.offset(j as isize) };
                    let lang2 = unsafe { (*(*lp2).lp_slang).sl_name };
                    if unsafe { !(*(*lp2).lp_slang).sl_rep.is_empty() }
                        && unsafe { cstr::prefix_eq(lang, lang2, 2) }
                    {
                        unsafe { (*lp).lp_replang = (*lp2).lp_slang };
                        break;
                    }
                }
            }
        }
        unsafe { redraw_later(wp, UPD_NOT_VALID) };
    }

    unsafe { xfree(spl_copy as *mut c_void) };
    recursive.set(false);
    ret_msg
}

/// Forget the midword characters recorded for `wp`.
fn clear_midword(wp: *mut win_T) {
    unsafe { (*(*wp).w_s).b_spell_ismw = [false; 256] };
    unsafe { xfree((*(*wp).w_s).b_spell_ismw_mb as *mut c_void) };
    unsafe { (*(*wp).w_s).b_spell_ismw_mb = core::ptr::null_mut() };
}

/// The index of region `region[..2]` in `rp` (which is `sl_regions`, two
/// characters per region), or `REGION_ALL` when it is not there.
unsafe fn find_region(rp: *const c_char, region: *const c_char) -> c_int {
    let mut i = 0;
    loop {
        if unsafe { *rp.offset(i as isize) } == 0 {
            return REGION_ALL;
        }
        if unsafe { *rp.offset(i as isize) } == unsafe { *region }
            && unsafe { *rp.offset(i as isize + 1) } == unsafe { *region.offset(1) }
        {
            return i / 2;
        }
        i += 2;
    }
}

/// Delete the internal word list and its compiled `.spl`.
pub unsafe fn spell_delete_wordlist() {
    if int_wordlist.get().is_null() {
        return;
    }

    let mut fname = [0 as c_char; MAXPATHL as usize];
    unsafe { os_remove(int_wordlist.get()) };
    unsafe { int_wordlist_spl(fname.as_mut_ptr()) };
    unsafe { os_remove(fname.as_mut_ptr()) };
    unsafe { xfree(int_wordlist.get() as *mut c_void) };
    int_wordlist.set(core::ptr::null_mut());
}

/// Free every loaded language and everything derived from them.
pub unsafe fn spell_free_all() {
    for buf in buffers() {
        // SAFETY: a live buffer from the editor's own list, and its own
        // growarray. The address is taken from the raw pointer rather than
        // through `DerefMut`, so no `&mut buf_T` is formed.
        unsafe { ga_clear(&raw mut (*buf.raw()).b_s.b_langp) };
    }

    while !first_lang.get().is_null() {
        let slang = first_lang.get();
        first_lang.set(unsafe { (*slang).sl_next });
        unsafe { slang_free(slang) };
    }

    unsafe { spell_delete_wordlist() };

    unsafe { xfree(repl_to.get() as *mut c_void) };
    repl_to.set(core::ptr::null_mut());
    unsafe { xfree(repl_from.get() as *mut c_void) };
    repl_from.set(core::ptr::null_mut());
}

/// Drop every spelling table and load them again, after `'encoding'`
/// changed or `:mkspell` ran.
pub unsafe fn spell_reload() {
    // SAFETY: on the main thread, as every caller of this is.
    init_spell_chartab();
    unsafe { spell_free_all() };

    // Only load word lists where 'spelllang' is set and some window on
    // the buffer has 'spell' on. The walk is over the current tab, which
    // always starts at `firstwin`.
    for wp in windows() {
        // SAFETY: a live window of the current tab page, and the synblock it
        // points at.
        if unsafe { *(*wp.w_s).b_p_spl } != 0 && wp.w_onebuf_opt.wo_spell != 0 {
            unsafe { parse_spelllang(wp.raw()) };
            break;
        }
    }
}

/// Whether `val` is a usable `'spelllang'` value.
pub fn valid_spelllang(val: &CStr) -> bool {
    valid_name(val, b".-_,@")
}

/// Whether `val` is a usable `'spellfile'` value: a comma-separated list of
/// file names, each ending in `.add` and made of file-name characters.
pub unsafe fn valid_spellfile(val: *const c_char) -> bool {
    let mut spf_name = [0 as c_char; MAXPATHL as usize];
    let mut spf = val as *mut c_char;
    while unsafe { *spf } != 0 {
        let (buf, room) = (spf_name.as_mut_ptr(), MAXPATHL as size_t);
        let sep = c",".as_ptr() as *mut c_char;
        let l = unsafe { copy_option_part(&raw mut spf, buf, room, sep) };
        if l >= MAXPATHL as size_t - 4
            || l < 4
            || unsafe { !cstr::eq_bytes(spf_name.as_ptr().add(l - 4), b".add") }
        {
            return false;
        }
        let mut s = spf_name.as_ptr();
        while unsafe { *s } != 0 {
            if !unsafe { vim_is_fname_char(*s as u8 as c_int) } {
                return false;
            }
            s = unsafe { s.offset(1) };
        }
    }
    true
}

/// Re-parse `'spelllang'` for the current buffer after a spell option
/// changed.
pub unsafe fn did_set_spell_option() -> Option<&'static CStr> {
    let mut errmsg = None;
    for wp in windows() {
        if wp.w_buffer == curbuf.get() && wp.w_onebuf_opt.wo_spell != 0 {
            // SAFETY: a live window of the current tab page.
            errmsg = unsafe { parse_spelllang(wp.raw()) };
            break;
        }
    }
    errmsg
}

/// Compile `'spellcapcheck'` into `b_cap_prog`, anchored so that it can
/// only match at one column.
///
/// Returns an error message when the pattern does not compile, leaving the
/// previous program in place.
pub unsafe fn compile_cap_prog(synblock: *mut synblock_T) -> Option<&'static CStr> {
    let rp: *mut regprog_T = unsafe { (*synblock).b_cap_prog };

    if unsafe { (*synblock).b_p_spc }.is_null() || unsafe { *(*synblock).b_p_spc } == 0 {
        unsafe { (*synblock).b_cap_prog = core::ptr::null_mut() };
    } else {
        let re = unsafe { concat_str(c"^".as_ptr(), (*synblock).b_p_spc) };
        unsafe { (*synblock).b_cap_prog = vim_regcomp(re, RE_MAGIC as c_int) };
        unsafe { xfree(re as *mut c_void) };
        if unsafe { (*synblock).b_cap_prog }.is_null() {
            unsafe { (*synblock).b_cap_prog = rp }; // keep the previous program
            return Some(e_invarg);
        }
    }

    unsafe { vim_regfree(rp) };
    None
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
    if unsafe { (*lp).sl_midword }.is_null() {
        return;
    }

    let mut p = unsafe { (*lp).sl_midword };
    while unsafe { *p } != 0 {
        let c = unsafe { utf_ptr2char(p) };
        let l = unsafe { utfc_ptr2len(p) };
        if c < 256 && l <= 2 {
            unsafe { (*(*wp).w_s).b_spell_ismw[c as usize] = true };
        } else if unsafe { (*(*wp).w_s).b_spell_ismw_mb }.is_null() {
            let copy = unsafe { xmemdupz(p as *const c_void, l as size_t) };
            unsafe { (*(*wp).w_s).b_spell_ismw_mb = copy as *mut c_char };
        } else {
            let n = unsafe { cstr::bytes_at((*(*wp).w_s).b_spell_ismw_mb) }.len() as c_int;
            let bp = unsafe { xstrnsave((*(*wp).w_s).b_spell_ismw_mb, (n + l) as size_t) };
            unsafe { xfree((*(*wp).w_s).b_spell_ismw_mb as *mut c_void) };
            unsafe { (*(*wp).w_s).b_spell_ismw_mb = bp };
            let at = unsafe { bp.offset(n as isize) } as *mut c_void;
            unsafe { xmemcpyz(at, p as *const c_void, l as size_t) };
        }
        p = unsafe { p.offset(l as isize) };
    }
}
