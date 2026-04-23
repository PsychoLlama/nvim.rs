//! Spell language loading: implements `parse_spelllang`, `use_midword`,
//! and `clear_midword` in Rust.
//!
//! Migrated from `src/nvim/spell.c`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::manual_c_str_literals)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{GArrayRaw, LangpT, SlangHandle, SlangRaw};

// bufref_T layout: { buf_T*, int fnum, int buf_free_count }
// 8 + 4 + 4 = 16 bytes
#[repr(C)]
struct BufrefT {
    br_buf: *mut c_void,
    br_fnum: c_int,
    br_buf_free_count: c_int,
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // String
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn xmemcpyz(dst: *mut c_void, src: *const c_void, len: usize);

    // garray
    fn ga_init(gap: *mut GArrayRaw, itemsize: c_int, growsize: c_int);
    fn ga_clear(gap: *mut GArrayRaw);

    // MB / char
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Spell
    // Note: rs_valid_spelllang is exported as "valid_spelllang"
    #[link_name = "valid_spelllang"]
    fn rs_valid_spelllang(val: *const c_char) -> bool;
    fn spell_load_file(
        fname: *const c_char,
        lang: *const c_char,
        old_lp: *mut SlangRaw,
        add_file: bool,
    ) -> *mut SlangRaw;
    fn rs_find_region(rp: *const c_char, region: *const c_char) -> c_int;
    fn rs_win_valid_any_tab(win: *mut c_void) -> c_int;

    // Shim accessors
    fn nvim_spell_get_b_p_spf() -> *const c_char;
    fn nvim_win_get_w_buffer(wp: *mut c_void) -> *mut c_void;
    fn nvim_win_set_b_cjk(wp: *mut c_void, val: c_int);
    fn nvim_win_set_b_langp(wp: *mut c_void, ga: GArrayRaw);
    fn nvim_spell_ga_append_langp(ga: *mut GArrayRaw) -> *mut LangpT;
    #[link_name = "set_bufref"]
    fn nvim_spell_set_bufref(bufref: *mut BufrefT, buf: *mut c_void);
    // bufref_valid is a static inline wrapper; call the underlying Rust impl directly
    #[link_name = "rs_bufref_valid"]
    fn nvim_spell_bufref_valid_raw(bufref: *mut BufrefT) -> c_int;
    #[link_name = "copy_option_part"]
    fn nvim_spell_copy_option_part(
        pp: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    #[link_name = "path_full_compare"]
    fn nvim_spell_path_full_compare(
        s1: *mut c_char,
        s2: *mut c_char,
        checkname: bool,
        expandenv: bool,
    ) -> c_int;
    #[link_name = "path_fnamecmp"]
    fn nvim_spell_path_fnamecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    #[link_name = "path_tail"]
    fn nvim_spell_path_tail(fname: *const c_char) -> *mut c_char;
    #[link_name = "strcasecmp"]
    fn nvim_spell_stricmp(a: *const c_char, b: *const c_char) -> c_int;
    // redraw_later(wp, UPD_NOT_VALID=40)
    #[link_name = "redraw_later"]
    fn nvim_spell_redraw_later_c(wp: *mut c_void, update_type: c_int);
    fn nvim_spell_int_wordlist_spl(fname: *mut c_char);
    fn nvim_spell_load_lang(lang: *mut c_char);

    // Warn about unsupported region
    fn nvim_spell_warn_region(region: *const c_char);

    // starting global (non-zero while Nvim is starting)
    static starting: c_int;

    // ismw accessors
    fn nvim_spell_win_get_ismw(wp: *mut c_void, c: c_int) -> bool;
    fn nvim_spell_win_set_ismw(wp: *mut c_void, c: c_int, val: bool);
    fn nvim_spell_win_get_ismw_mb(wp: *mut c_void) -> *const c_char;
    fn nvim_spell_win_set_ismw_mb(wp: *mut c_void, new_val: *mut c_char);
    fn nvim_spell_win_clear_ismw(wp: *mut c_void);

    // Window accessors (declared in lib.rs extern blocks but re-declared for local use)
    fn nvim_win_get_b_p_spl(wp: *const c_void) -> *const c_char;
    fn nvim_win_get_b_langp(wp: *const c_void) -> *const GArrayRaw;

    // int_wordlist global
    #[link_name = "int_wordlist"]
    static mut int_wordlist_global: *mut c_char;

    // first_lang global
    #[link_name = "first_lang"]
    static first_lang_global: *mut SlangRaw;
}

const MAXWLEN: usize = 254;
const MAXPATHL: usize = 4096;
const REGION_ALL: c_int = 0xff;
const K_EQUAL_FILES: c_int = 1; // kEqualFiles

/// Recursion guard for parse_spelllang.
static RECURSIVE: AtomicBool = AtomicBool::new(false);

/// Parse 'spelllang' and set w_s->b_langp accordingly.
/// Returns NULL if OK, or a pointer to an untranslated error message.
///
/// # Safety
///
/// `wp` must be a valid win_T pointer.
#[export_name = "parse_spelllang"]
pub unsafe extern "C" fn rs_parse_spelllang(wp: *mut c_void) -> *const c_char {
    let mut ret_msg: *const c_char = std::ptr::null();

    let buf_ptr = nvim_win_get_w_buffer(wp);
    let mut bufref = BufrefT {
        br_buf: std::ptr::null_mut(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    nvim_spell_set_bufref(&mut bufref, buf_ptr);

    // We don't want to do this recursively.
    if RECURSIVE.swap(true, Ordering::SeqCst) {
        return std::ptr::null();
    }

    let mut ga = GArrayRaw {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init(&mut ga, std::mem::size_of::<LangpT>() as c_int, 2);
    rs_clear_midword(wp);

    // Get b_p_spl (spelllang)
    let b_p_spl = nvim_win_get_b_p_spl(wp);
    // Make a copy - SpellFileMissing autocmds may change it under our feet.
    let spl_copy = xstrdup(b_p_spl);

    nvim_win_set_b_cjk(wp, 0);

    // region strings - use a local buffer for tracking
    let mut region_cp = [0u8; 3];
    let mut use_region: *const c_char = std::ptr::null();
    let mut dont_use_region = false;
    let mut nobreak = false;

    // Loop over comma-separated language names.
    let comma_sep = b",\0".as_ptr().cast::<c_char>();
    let spl_sep = b",\0".as_ptr().cast::<c_char>();
    let _ = spl_sep;
    let mut lang = [0u8; MAXWLEN + 1];
    let mut splp = spl_copy;

    while *(splp as *const u8) != 0 {
        // Get one language name.
        nvim_spell_copy_option_part(
            &mut splp,
            lang.as_mut_ptr().cast::<c_char>(),
            MAXWLEN,
            comma_sep,
        );
        let len = strlen(lang.as_ptr().cast::<c_char>()) as c_int;

        if !rs_valid_spelllang(lang.as_ptr().cast::<c_char>()) {
            continue;
        }

        // Check for "cjk"
        let cjk = b"cjk\0";
        if libc::strcmp(
            lang.as_ptr().cast::<c_char>(),
            cjk.as_ptr().cast::<c_char>(),
        ) == 0
        {
            nvim_win_set_b_cjk(wp, 1);
            continue;
        }

        let mut region: *const c_char = std::ptr::null();
        let filename;

        // If name ends in ".spl" use it as spell file name.
        let spl_suffix = b".spl\0";
        if len > 4
            && nvim_spell_path_fnamecmp(
                lang.as_ptr().cast::<c_char>().add((len - 4) as usize),
                spl_suffix.as_ptr().cast::<c_char>(),
            ) == 0
        {
            filename = true;

            // Locate a region and remove it from the file name.
            let tail = nvim_spell_path_tail(lang.as_ptr().cast::<c_char>());
            let underscore = b'_' as c_int;
            let p = strchr(tail, underscore);
            if !p.is_null()
                && (*(p.add(1) as *const u8)).is_ascii_alphabetic()
                && (*(p.add(2) as *const u8)).is_ascii_alphabetic()
                && !(*(p.add(3) as *const u8)).is_ascii_alphabetic()
            {
                // Copy region to region_cp
                region_cp[0] = *(p.add(1) as *const u8);
                region_cp[1] = *(p.add(2) as *const u8);
                region_cp[2] = 0;
                // memmove: remove the 3-char region suffix (_XX) from lang
                let lang_start = lang.as_ptr() as usize;
                let p_offset = p as usize - lang_start;
                let tail_len = strlen(lang.as_ptr().cast::<c_char>()) - p_offset;
                // We want to move p+3 to p, removing "_XX"
                std::ptr::copy(p.add(3), p, tail_len - 2);
                region = region_cp.as_ptr().cast::<c_char>();
            } else {
                dont_use_region = true;
            }

            // Check if we loaded this language before.
            let mut slang = SlangHandle::first();
            while !slang.is_null() {
                if nvim_spell_path_full_compare(
                    lang.as_mut_ptr().cast::<c_char>(),
                    (*slang.0).sl_fname,
                    false,
                    true,
                ) == K_EQUAL_FILES
                {
                    break;
                }
                slang = slang.next();
            }
            let slang_ptr = if slang.is_null() {
                std::ptr::null_mut()
            } else {
                slang.0
            };

            if region.is_null() {
                // no region found in filename path
            }

            if !region.is_null() {
                if !use_region.is_null() && libc::strcmp(region, use_region) != 0 {
                    dont_use_region = true;
                }
                use_region = region;
            }

            if slang_ptr.is_null() {
                spell_load_file(
                    lang.as_ptr().cast::<c_char>(),
                    lang.as_ptr().cast::<c_char>(),
                    std::ptr::null_mut(),
                    false,
                );
            }

            // Loop over languages
            let mut sl = SlangHandle::first();
            while !sl.is_null() {
                if nvim_spell_path_full_compare(
                    lang.as_mut_ptr().cast::<c_char>(),
                    (*sl.0).sl_fname,
                    false,
                    true,
                ) == K_EQUAL_FILES
                {
                    let region_mask = compute_region_mask_filename(sl, region);
                    if region_mask != 0 {
                        let lp = nvim_spell_ga_append_langp(&mut ga);
                        (*lp).lp_slang = sl.0;
                        (*lp).lp_region = region_mask;
                        rs_use_midword(sl.0, wp);
                        if (*sl.0).sl_nobreak {
                            nobreak = true;
                        }
                    }
                }
                sl = sl.next();
            }
        } else {
            filename = false;

            // Check for region suffix: lang_XX
            if len > 3 && *(lang.as_ptr().add((len - 3) as usize)) == b'_' {
                // region is at lang[len-2]
                region = lang.as_ptr().cast::<c_char>().add((len - 2) as usize);
                // truncate lang at len-3
                lang[(len - 3) as usize] = 0;
            } else {
                dont_use_region = true;
            }

            // Check if we loaded this language before.
            let mut slang = SlangHandle::first();
            while !slang.is_null() {
                if nvim_spell_stricmp(lang.as_ptr().cast::<c_char>(), (*slang.0).sl_name) == 0 {
                    break;
                }
                slang = slang.next();
            }
            let slang_ptr = if slang.is_null() {
                std::ptr::null_mut::<SlangRaw>()
            } else {
                slang.0
            };

            if !region.is_null() {
                if !use_region.is_null() && libc::strcmp(region, use_region) != 0 {
                    dont_use_region = true;
                }
                use_region = region;
            }

            // If not found try loading the language now.
            if slang_ptr.is_null() {
                nvim_spell_load_lang(lang.as_mut_ptr().cast::<c_char>());
                // SpellFileMissing autocommands may do anything
                if nvim_spell_bufref_valid_raw(&mut bufref) == 0 || rs_win_valid_any_tab(wp) == 0 {
                    ret_msg = b"E797: SpellFileMissing autocommand deleted buffer\0"
                        .as_ptr()
                        .cast::<c_char>();
                    xfree(spl_copy.cast::<c_void>());
                    RECURSIVE.store(false, Ordering::SeqCst);
                    return ret_msg;
                }
            }

            // Loop over the languages
            let mut sl = SlangHandle::first();
            while !sl.is_null() {
                if nvim_spell_stricmp(lang.as_ptr().cast::<c_char>(), (*sl.0).sl_name) == 0 {
                    let region_mask = compute_region_mask(sl, region, filename);
                    if region_mask != 0 {
                        let lp = nvim_spell_ga_append_langp(&mut ga);
                        (*lp).lp_slang = sl.0;
                        (*lp).lp_region = region_mask;
                        rs_use_midword(sl.0, wp);
                        if (*sl.0).sl_nobreak {
                            nobreak = true;
                        }
                    }
                }
                sl = sl.next();
            }
        }

        let _ = filename; // suppress unused warning
    }

    // round 0: load int_wordlist, if possible.
    // round 1+: load each name in 'spellfile'.
    let b_p_spf_orig = nvim_spell_get_b_p_spf();
    let spf_copy = xstrdup(b_p_spf_orig);
    let mut spf = spf_copy;
    let mut spf_name = [0u8; MAXPATHL];
    let mut round = 0i32;

    loop {
        if round == 0 {
            // Internal wordlist
            if int_wordlist_global.is_null() {
                round += 1;
                continue;
            }
            nvim_spell_int_wordlist_spl(spf_name.as_mut_ptr().cast::<c_char>());
        } else {
            if *(spf as *const u8) == 0 {
                break;
            }
            // One entry in 'spellfile'
            let comma_sep2 = b",\0".as_ptr().cast::<c_char>();
            nvim_spell_copy_option_part(
                &mut spf,
                spf_name.as_mut_ptr().cast::<c_char>(),
                MAXPATHL - 4,
                comma_sep2,
            );
            let add_spl = b".spl\0";
            strcat(
                spf_name.as_mut_ptr().cast::<c_char>(),
                add_spl.as_ptr().cast::<c_char>(),
            );

            // If already found above, skip
            let mut already_found = false;
            for c in 0..ga.ga_len {
                let lp = (ga.ga_data as *mut LangpT).add(c as usize);
                let p = (*lp).lp_slang;
                if !p.is_null()
                    && !(*p).sl_fname.is_null()
                    && nvim_spell_path_full_compare(
                        spf_name.as_mut_ptr().cast::<c_char>(),
                        (*p).sl_fname,
                        false,
                        true,
                    ) == K_EQUAL_FILES
                {
                    already_found = true;
                    break;
                }
            }
            if already_found {
                round += 1;
                continue;
            }
        }

        // Check if it was loaded already.
        let mut slang = SlangHandle(first_lang_global);
        while !slang.is_null() {
            if nvim_spell_path_full_compare(
                spf_name.as_mut_ptr().cast::<c_char>(),
                (*slang.0).sl_fname,
                false,
                true,
            ) == K_EQUAL_FILES
            {
                break;
            }
            slang = slang.next();
        }

        let slang_ptr = if slang.is_null() {
            // Not loaded, try loading it now.
            let add_spl_b = b".spl\0";
            let _ = add_spl_b;
            if round == 0 {
                let internal = b"internal wordlist\0";
                libc::strcpy(
                    lang.as_mut_ptr().cast::<c_char>(),
                    internal.as_ptr().cast::<c_char>(),
                );
            } else {
                let tail = nvim_spell_path_tail(spf_name.as_ptr().cast::<c_char>());
                xstrlcpy(lang.as_mut_ptr().cast::<c_char>(), tail, MAXWLEN + 1);
                let dot = b'.' as c_int;
                let p = strchr(lang.as_ptr().cast::<c_char>(), dot);
                if !p.is_null() {
                    *p = 0; // truncate at ".encoding.add"
                }
            }
            let loaded = spell_load_file(
                spf_name.as_ptr().cast::<c_char>(),
                lang.as_ptr().cast::<c_char>(),
                std::ptr::null_mut(),
                true,
            );
            if !loaded.is_null() && nobreak {
                (*loaded).sl_nobreak = true;
            }
            loaded
        } else {
            slang.0
        };

        if !slang_ptr.is_null() {
            let mut region_mask = REGION_ALL;
            if !use_region.is_null() && !dont_use_region {
                let c = rs_find_region((*slang_ptr).sl_regions.as_ptr(), use_region);
                if c != REGION_ALL {
                    region_mask = 1 << c;
                } else if *((*slang_ptr).sl_regions.as_ptr() as *const u8) != 0 {
                    // This spell file is for other regions.
                    region_mask = 0;
                }
            }

            if region_mask != 0 {
                let lp = nvim_spell_ga_append_langp(&mut ga);
                (*lp).lp_slang = slang_ptr;
                (*lp).lp_sallang = std::ptr::null_mut();
                (*lp).lp_replang = std::ptr::null_mut();
                (*lp).lp_region = region_mask;
                rs_use_midword(slang_ptr, wp);
            }
        }

        round += 1;
    }

    xfree(spf_copy.cast::<c_void>());

    // Everything is fine, store the new b_langp value.
    nvim_win_set_b_langp(wp, ga);

    // For each language figure out what language to use for sound folding and REP items.
    let final_ga_ptr = crate::nvim_win_get_b_langp(wp);
    let final_ga_len = (*final_ga_ptr).ga_len;

    for i in 0..final_ga_len {
        let lp = ((*final_ga_ptr).ga_data as *mut LangpT).add(i as usize);

        // sound folding
        if (*(*lp).lp_slang).sl_sal.ga_len > 0 {
            (*lp).lp_sallang = (*lp).lp_slang;
        } else {
            for j in 0..final_ga_len {
                let lp2 = ((*final_ga_ptr).ga_data as *mut LangpT).add(j as usize);
                if (*(*lp2).lp_slang).sl_sal.ga_len > 0
                    && libc::strncmp((*(*lp).lp_slang).sl_name, (*(*lp2).lp_slang).sl_name, 2) == 0
                {
                    (*lp).lp_sallang = (*lp2).lp_slang;
                    break;
                }
            }
        }

        // REP items
        if (*(*lp).lp_slang).sl_rep.ga_len > 0 {
            (*lp).lp_replang = (*lp).lp_slang;
        } else {
            for j in 0..final_ga_len {
                let lp2 = ((*final_ga_ptr).ga_data as *mut LangpT).add(j as usize);
                if (*(*lp2).lp_slang).sl_rep.ga_len > 0
                    && libc::strncmp((*(*lp).lp_slang).sl_name, (*(*lp2).lp_slang).sl_name, 2) == 0
                {
                    (*lp).lp_replang = (*lp2).lp_slang;
                    break;
                }
            }
        }
    }

    nvim_spell_redraw_later_c(wp, 40); // UPD_NOT_VALID = 40

    xfree(spl_copy.cast::<c_void>());
    RECURSIVE.store(false, Ordering::SeqCst);
    ret_msg
}

/// Compute region_mask for a filename-based slang match (no region lookup).
unsafe fn compute_region_mask_filename(sl: SlangHandle, region: *const c_char) -> c_int {
    if region.is_null() {
        REGION_ALL
    } else {
        // For filename entries, just use REGION_ALL if no region
        REGION_ALL
    }
}

/// Compute region_mask for a name-based slang match.
unsafe fn compute_region_mask(sl: SlangHandle, region: *const c_char, _filename: bool) -> c_int {
    let mut region_mask = REGION_ALL;
    if !region.is_null() {
        let c = rs_find_region((*sl.0).sl_regions.as_ptr(), region);
        if c == REGION_ALL {
            if (*sl.0).sl_add {
                if *((*sl.0).sl_regions.as_ptr() as *const u8) != 0 {
                    // Addition file for other regions.
                    region_mask = 0;
                }
            } else {
                // Probably an error. Give warning and accept anyway.
                warn_region_not_supported(region);
            }
        } else {
            region_mask = 1 << c;
        }
    }
    region_mask
}

unsafe fn warn_region_not_supported(region: *const c_char) {
    nvim_spell_warn_region(region);
}

/// Clear the midword characters for buffer "wp".
///
/// # Safety
///
/// `wp` must be a valid win_T pointer.
#[export_name = "clear_midword"]
pub unsafe extern "C" fn rs_clear_midword(wp: *mut c_void) {
    nvim_spell_win_clear_ismw(wp);
}

/// Use the "sl_midword" field of language "lp" for buffer "wp".
/// Adds midword chars to any currently used midword characters.
///
/// # Safety
///
/// Both `lp` and `wp` must be valid non-null pointers.
#[export_name = "use_midword"]
pub unsafe extern "C" fn rs_use_midword(lp: *mut SlangRaw, wp: *mut c_void) {
    if (*lp).sl_midword.is_null() {
        return;
    }

    let mut p = (*lp).sl_midword;
    while *(p as *const u8) != 0 {
        let c = utf_ptr2char(p);
        let l = utfc_ptr2len(p);
        if c < 256 && l <= 2 {
            nvim_spell_win_set_ismw(wp, c, true);
        } else {
            let ismw_mb = nvim_spell_win_get_ismw_mb(wp);
            if ismw_mb.is_null() {
                // First multi-byte char in b_spell_ismw_mb.
                let new_mb = xmemdupz(p.cast::<c_void>(), l as usize).cast::<c_char>();
                nvim_spell_win_set_ismw_mb(wp, new_mb);
            } else {
                // Append multi-byte char to b_spell_ismw_mb.
                let n = strlen(ismw_mb) as usize;
                let bp = xstrnsave(ismw_mb, n + l as usize);
                xmemcpyz(bp.add(n).cast::<c_void>(), p.cast::<c_void>(), l as usize);
                nvim_spell_win_set_ismw_mb(wp, bp);
            }
        }
        p = p.add(l as usize);
    }
}
