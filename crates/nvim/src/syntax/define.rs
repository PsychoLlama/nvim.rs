//! `:syntax match`, `:syntax region` and `:syntax include`.
//!
//! The three subcommands that add a pattern-based item, plus
//! [`get_syn_pattern`], which parses one `/pat/` with its `ms=`/`me=`/... offset
//! suffixes into a `synpat_T`. `:syntax include` is here too: it sources another
//! syntax file under an inclusion tag so its toplevel items become contained
//! ones.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::optionstr::empty_option;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::*;
use crate::regexp::RE_MAGIC;
use crate::runtime::RuntimeOpts;
use crate::types::{ExArgt, FAIL, NUL};

/// Adjust an item's flags when it is declared in a `:syntax include`d file.
///
/// Sets the contained flag, and if the item is not already contained adds it to
/// the top-level cluster the `:syntax include` named, if any.
pub(crate) unsafe fn syn_incl_toplevel(id: c_int, flags: &mut SynFlags) {
    if flags.has(SynFlags::CONTAINED) || cur_syn_block().b_syn_topgrp == 0 {
        return;
    }
    *flags |= SynFlags::CONTAINED | SynFlags::INCLUDED_TOPLEVEL;
    if cur_syn_block().b_syn_topgrp >= SYNID_CLUSTER {
        // Allocated, because `syn_combine_list` consumes it.
        let mut grp_list =
            unsafe { xmalloc(2 * ::core::mem::size_of::<int16_t>()) } as *mut int16_t;
        unsafe { *grp_list = id as int16_t };
        unsafe { *grp_list.add(1) = 0 };
        let tlg_id = cur_syn_block().b_syn_topgrp - SYNID_CLUSTER;
        // SAFETY: `tlg_id` is a live cluster index; the address comes off the
        // pointer rather than a `Deref` borrow.
        let list = unsafe { &mut (*cur_cluster(tlg_id).raw()).scl_list };
        unsafe { syn_combine_list(list, &mut grp_list, CLUSTER_ADD) };
    }
}

/// `:syntax include [@{cluster}] {file}`.
pub(crate) unsafe fn syn_cmd_include(eap: *mut exarg_T, _syncing: c_int) {
    let mut arg = unsafe { (*eap).arg };
    let mut sgl_id = 1;

    unsafe { (*eap).nextcmd = find_nextcmd(arg) };
    if unsafe { (*eap).skip } != 0 {
        return;
    }

    if unsafe { *arg } as c_int == '@' as c_int {
        arg = unsafe { arg.add(1) };
        let mut group_name_end = ::core::ptr::null_mut::<c_char>();
        let rest = unsafe { get_group_name(arg, &mut group_name_end) };
        if rest.is_null() {
            emsg(gettext(c"E397: Filename required"));
            return;
        }
        sgl_id = unsafe { syn_check_cluster(arg, group_name_end.offset_from(arg) as c_int) };
        if sgl_id == 0 {
            return;
        }
        // `separate_nextcmd` and `expand_filename` depend on this.
        unsafe { (*eap).arg = rest };
    }

    // Everything left, up to the next command, is the file to include.
    unsafe { (*eap).argt |= ExArgt::XFILE | ExArgt::NOSPC };
    unsafe { separate_nextcmd(eap) };

    // An absolute path, "$VIM/.." or "<sfile>.." is `:source`d, which needs
    // the name expanded first; everything else goes through `:runtime!`.
    let source = unsafe { *(*eap).arg } as c_int == '<' as c_int
        || unsafe { *(*eap).arg } as c_int == '$' as c_int
        || unsafe { path_is_absolute((*eap).arg) };
    if source {
        let mut errormsg = None;
        if unsafe { expand_filename(eap, syn_cmdlinep.get(), &mut errormsg) } == FAIL {
            if let Some(msg) = &errormsg {
                emsg(msg);
            }
            return;
        }
    }

    if running_syn_inc_tag.get() >= MAX_SYN_INC_TAG {
        emsg(gettext(c"E847: Too many syntax includes"));
        return;
    }

    // Save and restore the top-level group and the `:syntax include` tag
    // around the inclusion itself.
    let prev_syn_inc_tag = current_syn_inc_tag.get();
    running_syn_inc_tag.set(running_syn_inc_tag.get() + 1);
    current_syn_inc_tag.set(running_syn_inc_tag.get());
    let prev_toplvl_grp = cur_syn_block().b_syn_topgrp;
    cur_syn_block().b_syn_topgrp = sgl_id;

    // SAFETY: the caller's command.
    let arg = unsafe { (*eap).arg };
    let failed = if source {
        // SAFETY: sourcing the file the user named.
        unsafe { do_source(arg, false, DOSO_NONE as c_int, ::core::ptr::null_mut()) == FAIL }
    } else {
        unsafe { source_runtime((*eap).arg, RuntimeOpts::ALL) == FAIL }
    };
    if failed {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str((*eap).arg) };
        semsg!("E484: Can't open file {arg}");
    }

    cur_syn_block().b_syn_topgrp = prev_toplvl_grp;
    current_syn_inc_tag.set(prev_syn_inc_tag);
}

/// First call for this window: init the pattern array.
pub(crate) unsafe fn init_syn_patterns() {
    cur_syn_block().b_syn_patterns.ga_itemsize = ::core::mem::size_of::<synpat_T>() as c_int;
    unsafe { ga_set_growsize(syn_field!(cur_syn_block(), b_syn_patterns), 10) };
}

/// A zeroed pattern, which is what upstream's `CLEAR_FIELD`/`xcalloc` leave.
unsafe fn empty_synpat() -> synpat_T {
    unsafe { ::core::mem::zeroed() }
}

/// The default options for `:syntax match` and `:syntax region`, both of which
/// accept a `contains=` list.
fn item_opt(sync_idx: *mut c_int) -> syn_opt_arg_T {
    syn_opt_arg_T {
        flags: SynFlags::NONE,
        keyword: false,
        sync_idx,
        has_cont_list: true,
        cont_list: ::core::ptr::null_mut(),
        cont_in_list: ::core::ptr::null_mut(),
        next_list: ::core::ptr::null_mut(),
    }
}

/// Free the two allocations a half-built pattern owns.
unsafe fn free_synpat(item: &synpat_T) {
    unsafe { vim_regfree(item.sp_prog) };
    unsafe { xfree(item.sp_pattern as *mut c_void) };
}

/// Free the three id lists an abandoned option set owns.
unsafe fn free_opt_lists(opt: &syn_opt_arg_T) {
    unsafe { xfree(opt.cont_list as *mut c_void) };
    unsafe { xfree(opt.cont_in_list as *mut c_void) };
    unsafe { xfree(opt.next_list as *mut c_void) };
}

/// `:syntax match {group} [{options}] {pattern} [{options}]`, and
/// `:syntax sync match {group} [[grouphere|groupthere] {group}] ..`.
pub(crate) unsafe fn syn_cmd_match(eap: *mut exarg_T, syncing: c_int) {
    let arg = unsafe { (*eap).arg };
    let mut group_name_end = ::core::ptr::null_mut::<c_char>();
    let mut sync_idx: c_int = 0;
    let mut conceal_char: c_int = NUL;

    // Isolate the group name, check for validity.
    let mut rest = unsafe { get_group_name(arg, &mut group_name_end) };

    let mut opt = item_opt(if syncing != 0 {
        &raw mut sync_idx
    } else {
        ::core::ptr::null_mut()
    });

    // Options before the pattern, the pattern, then options after it.
    rest = unsafe { get_syn_options(rest, &mut opt, &mut conceal_char, (*eap).skip) };
    unsafe { init_syn_patterns() };
    let mut item = unsafe { empty_synpat() };
    rest = unsafe { get_syn_pattern(rest, &mut item) };
    if vim_regcomp_had_eol() != 0 && !opt.flags.has(SynFlags::EXCLUDENL) {
        opt.flags |= SynFlags::HAS_EOL;
    }
    rest = unsafe { get_syn_options(rest, &mut opt, &mut conceal_char, (*eap).skip) };

    let mut stored = false;
    if !rest.is_null() {
        // Check for a trailing command and illegal trailing arguments.
        unsafe { (*eap).nextcmd = check_nextcmd(rest) };
        if ends_excmd(unsafe { *rest } as c_int) == 0 || unsafe { (*eap).skip } != 0 {
            rest = ::core::ptr::null_mut();
        } else {
            let syn_id = unsafe { syn_check_group(arg, group_name_end.offset_from(arg) as size_t) };
            if syn_id != 0 {
                unsafe { syn_incl_toplevel(syn_id, &mut opt.flags) };
                // Store the pattern in the item list.
                // SAFETY: `ga_append_via_ptr` answered a fresh slot in the
                // pattern array, which nothing grows before the writes below.
                let mut spp = unsafe {
                    Pat::new(ga_append_via_ptr(
                        syn_field!(cur_syn_block(), b_syn_patterns),
                        ::core::mem::size_of::<synpat_T>(),
                    ) as *mut synpat_T)
                };
                *spp = item;
                spp.sp_syncing = syncing != 0;
                spp.sp_type = SPTYPE_MATCH as c_char;
                spp.sp_syn.id = syn_id as int16_t;
                spp.sp_syn.inc_tag = current_syn_inc_tag.get();
                spp.sp_flags = opt.flags;
                spp.sp_sync_idx = sync_idx;
                spp.sp_cont_list = opt.cont_list;
                spp.sp_syn.cont_in_list = opt.cont_in_list;
                spp.sp_cchar = conceal_char;
                if !opt.cont_in_list.is_null() {
                    cur_syn_block().b_syn_containedin = 1;
                }
                spp.sp_next_list = opt.next_list;

                // Remember that we found a match to sync on.
                if opt.flags.has(SynFlags::SYNC_HERE | SynFlags::SYNC_THERE) {
                    cur_syn_block().b_syn_sync_flags |= SF_MATCH;
                }
                if opt.flags.has(SynFlags::FOLD) {
                    cur_syn_block().b_syn_folditems += 1;
                }

                redraw_curbuf_later(UPD_SOME_VALID);
                unsafe { syn_stack_free_all(cur_syn_block().raw()) }; // Need to recompute all.
                stored = true;
            }
        }
    }

    // Something failed: the pattern and the lists are still ours to free.
    if !stored {
        unsafe { free_synpat(&item) };
        unsafe { free_opt_lists(&opt) };
        if rest.is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E475: Invalid argument: {arg}");
        }
    }
}

/// One start/skip/end pattern of a `:syntax region`, with the `matchgroup=`
/// that was in force when it was read.
struct RegionPat {
    pat: synpat_T,
    matchgroup_id: c_int,
}

/// What `:syntax region` parsing produced, or why it stopped.
struct RegionArgs {
    /// The start, skip and end patterns, indexed by `ITEM_*`, each in
    /// **reverse** command order: upstream prepends to a linked list because
    /// "the list is used from end to start".
    pats: [Vec<RegionPat>; 3],
    opt: syn_opt_arg_T,
    conceal_char: c_int,
    /// Where parsing stopped, or NULL after an error.
    rest: *mut c_char,
    /// A required argument was missing, which is E399 rather than E390.
    not_enough: bool,
}

/// Which of the four keywords `key` (already upper-cased) names.
unsafe fn region_item(key: *const c_char) -> Option<c_int> {
    for (name, item) in [
        (c"MATCHGROUP", ITEM_MATCHGROUP),
        (c"START", ITEM_START),
        (c"END", ITEM_END),
        (c"SKIP", ITEM_SKIP),
    ] {
        if unsafe { strcmp(key, name.as_ptr()) } == 0 {
            return Some(item);
        }
    }
    None
}

/// Read the options, patterns and `matchgroup=`s of a `:syntax region`.
unsafe fn parse_region_args(eap: *mut exarg_T, mut rest: *mut c_char) -> RegionArgs {
    let mut out = RegionArgs {
        pats: [Vec::new(), Vec::new(), Vec::new()],
        opt: item_opt(::core::ptr::null_mut()),
        conceal_char: NUL,
        rest,
        not_enough: false,
    };
    let mut matchgroup_id = 0;
    let mut illegal = false;
    let mut key = ::core::ptr::null_mut::<c_char>();

    while !rest.is_null() && ends_excmd(unsafe { *rest } as c_int) == 0 {
        // Options may appear anywhere between the patterns.
        rest = unsafe { get_syn_options(rest, &mut out.opt, &mut out.conceal_char, (*eap).skip) };
        if rest.is_null() || ends_excmd(unsafe { *rest } as c_int) != 0 {
            break;
        }

        // Must be a pattern keyword or `matchgroup` then.
        let mut key_end = rest;
        while unsafe { *key_end } as c_int != 0
            && !ascii_iswhite(unsafe { *key_end } as c_int)
            && unsafe { *key_end } as c_int != '=' as c_int
        {
            key_end = unsafe { key_end.add(1) };
        }
        unsafe { xfree(key as *mut c_void) };
        key = unsafe { vim_strnsave_up(rest, key_end.offset_from(rest) as size_t) };
        let Some(item) = (unsafe { region_item(key) }) else {
            break;
        };
        if item == ITEM_SKIP && !out.pats[ITEM_SKIP as usize].is_empty() {
            illegal = true; // Only one skip pattern is allowed.
            break;
        }

        rest = unsafe { skipwhite(key_end) };
        if unsafe { *rest } as c_int != '=' as c_int {
            rest = ::core::ptr::null_mut();
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str((*eap).arg) };
            semsg!("E398: Missing '=': {arg}");
            break;
        }
        rest = unsafe { skipwhite(rest.add(1)) };
        if unsafe { *rest } as c_int == NUL {
            out.not_enough = true;
            break;
        }

        if item == ITEM_MATCHGROUP {
            let p = unsafe { skiptowhite(rest) };
            if (unsafe { p.offset_from(rest) } == 4
                && unsafe { strncmp(rest, c"NONE".as_ptr(), 4) } == 0)
                || unsafe { (*eap).skip } != 0
            {
                matchgroup_id = 0;
            } else {
                matchgroup_id = unsafe { syn_check_group(rest, p.offset_from(rest) as size_t) };
                if matchgroup_id == 0 {
                    illegal = true;
                    break;
                }
            }
            rest = unsafe { skipwhite(p) };
            continue;
        }

        // Enable the appropriate `\z` specials: a start pattern defines the
        // external matches, skip and end patterns use them.
        reg_do_extmatch.set(if item == ITEM_START { REX_SET } else { REX_USE });
        let mut pat = unsafe { empty_synpat() };
        rest = unsafe { get_syn_pattern(rest, &mut pat) };
        reg_do_extmatch.set(0);
        if item == ITEM_END && vim_regcomp_had_eol() != 0 && !out.opt.flags.has(SynFlags::EXCLUDENL)
        {
            pat.sp_flags |= SynFlags::HAS_EOL;
        }
        out.pats[item as usize].insert(0, RegionPat { pat, matchgroup_id });
    }

    unsafe { xfree(key as *mut c_void) };
    // An `illegal` stop is reported as E390, which is what upstream's
    // "rest = NULL" here and its `illegal || rest == NULL` test below say.
    out.rest = if illegal || out.not_enough {
        ::core::ptr::null_mut()
    } else {
        rest
    };
    out
}

/// `:syntax region {group} [matchgroup={group}] start={pat} .. [skip={pat}]
/// end={pat} .. [{options}]`.
pub(crate) unsafe fn syn_cmd_region(eap: *mut exarg_T, syncing: c_int) {
    let arg = unsafe { (*eap).arg };
    let mut group_name_end = ::core::ptr::null_mut::<c_char>();

    // Isolate the group name, check for validity.
    let rest = unsafe { get_group_name(arg, &mut group_name_end) };
    unsafe { init_syn_patterns() };

    let mut args = unsafe { parse_region_args(eap, rest) };
    let mut rest = args.rest;

    // Must have a "start" and an "end" pattern.
    if !rest.is_null()
        && (args.pats[ITEM_START as usize].is_empty() || args.pats[ITEM_END as usize].is_empty())
    {
        args.not_enough = true;
        rest = ::core::ptr::null_mut();
    }

    let mut success = false;
    if !rest.is_null() {
        // Check for trailing garbage or a command; if OK, add the item.
        unsafe { (*eap).nextcmd = check_nextcmd(rest) };
        if ends_excmd(unsafe { *rest } as c_int) == 0 || unsafe { (*eap).skip } != 0 {
            rest = ::core::ptr::null_mut();
        } else {
            let pat_count: c_int = args.pats.iter().map(|v| v.len() as c_int).sum();
            unsafe { ga_grow(syn_field!(cur_syn_block(), b_syn_patterns), pat_count) };
            let syn_id = unsafe { syn_check_group(arg, group_name_end.offset_from(arg) as size_t) };
            if syn_id != 0 {
                unsafe { syn_incl_toplevel(syn_id, &mut args.opt.flags) };
                unsafe { store_region(&args, syn_id, syncing != 0) };
                redraw_curbuf_later(UPD_SOME_VALID);
                unsafe { syn_stack_free_all(cur_syn_block().raw()) }; // Need to recompute all.
                success = true; // don't free the progs and patterns now
            }
        }
    }

    if !success {
        for list in &args.pats {
            for entry in list {
                unsafe { free_synpat(&entry.pat) };
            }
        }
        unsafe { free_opt_lists(&args.opt) };
        if args.not_enough {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E399: Not enough arguments: syntax region {arg}");
        } else if rest.is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E475: Invalid argument: {arg}");
        }
    }
}

/// Copy the parsed start/skip/end patterns into the block's pattern array as
/// consecutive entries.
///
/// The `contains=`/`containedin=`/`nextgroup=` lists go on the START entries
/// only, and are handed over rather than copied — which is why the caller must
/// not free them once this has run.
unsafe fn store_region(args: &RegionArgs, syn_id: c_int, syncing: bool) {
    let patterns: *mut garray_T = syn_field!(cur_syn_block(), b_syn_patterns);
    let mut idx = unsafe { (*patterns).ga_len };
    for item in [ITEM_START, ITEM_SKIP, ITEM_END] {
        for entry in &args.pats[item as usize] {
            let mut spp = unsafe { cur_pattern(idx) };
            *spp = entry.pat;
            spp.sp_syncing = syncing;
            spp.sp_type = if item == ITEM_START {
                SPTYPE_START
            } else if item == ITEM_SKIP {
                SPTYPE_SKIP
            } else {
                SPTYPE_END
            } as c_char;
            spp.sp_flags |= args.opt.flags;
            spp.sp_syn.id = syn_id as int16_t;
            spp.sp_syn.inc_tag = current_syn_inc_tag.get();
            spp.sp_syn_match_id = entry.matchgroup_id as int16_t;
            spp.sp_cchar = args.conceal_char;
            if item == ITEM_START {
                spp.sp_cont_list = args.opt.cont_list;
                spp.sp_syn.cont_in_list = args.opt.cont_in_list;
                if !args.opt.cont_in_list.is_null() {
                    cur_syn_block().b_syn_containedin = 1;
                }
                spp.sp_next_list = args.opt.next_list;
            }
            unsafe { (*patterns).ga_len += 1 };
            idx += 1;
            if args.opt.flags.has(SynFlags::FOLD) {
                cur_syn_block().b_syn_folditems += 1;
            }
        }
    }
}

/// Read one delimited pattern plus its offsets into `ci`.
///
/// Answers what follows it, or NULL after reporting an error.
pub(crate) unsafe fn get_syn_pattern(arg: *mut c_char, ci: &mut synpat_T) -> *mut c_char {
    // Need at least three characters: two delimiters and something between.
    if arg.is_null()
        || unsafe { *arg } as c_int == NUL
        || unsafe { *arg.add(1) } as c_int == NUL
        || unsafe { *arg.add(2) } as c_int == NUL
    {
        return ::core::ptr::null_mut();
    }

    let mut end = unsafe { skip_regexp(arg.add(1), *arg as c_int, 1) };
    if unsafe { *end } as c_int != unsafe { *arg } as c_int {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E401: Pattern delimiter not found: {arg}");
        return ::core::ptr::null_mut();
    }

    // Store the pattern and its compiled program. 'cpoptions' is emptied
    // first, to avoid the 'l' flag.
    ci.sp_pattern = unsafe { xstrnsave(arg.add(1), end.offset_from(arg) as size_t - 1) };
    let cpo_save = p_cpo.get();
    p_cpo.set(empty_option());
    ci.sp_prog = unsafe { vim_regcomp(ci.sp_pattern, RE_MAGIC) };
    p_cpo.set(cpo_save);
    if ci.sp_prog.is_null() {
        return ::core::ptr::null_mut();
    }
    ci.sp_ic = cur_syn_block().b_syn_ic;
    unsafe { syn_clear_time(&mut ci.sp_time) };

    let end = unsafe { read_pattern_offsets(ci, end.add(1)) };
    if ends_excmd(unsafe { *end } as c_int) == 0 && !ascii_iswhite(unsafe { *end } as c_int) {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E402: Garbage after pattern: {arg}");
        return ::core::ptr::null_mut();
    }
    unsafe { skipwhite(end) }
}

/// The offset names, indexed by `SPO_*`.
pub(crate) static SPO_NAME_TAB: [&CStr; SPO_COUNT as usize] =
    [c"ms=", c"me=", c"hs=", c"he=", c"rs=", c"re=", c"lc="];

/// Which `SPO_*` offset the three characters at `end` name.
unsafe fn offset_name(end: *const c_char) -> Option<c_int> {
    let mut idx = SPO_COUNT;
    loop {
        idx -= 1;
        if idx < 0 {
            return None;
        }
        if unsafe { strncmp(end, SPO_NAME_TAB[idx as usize].as_ptr(), 3) } == 0 {
            return Some(idx);
        }
    }
}

/// Read the comma-separated `ms=s+1,he=e-2,lc=3` offsets after a pattern.
///
/// Answers the first character that is not part of them. An unrecognised name,
/// an unrecognised `s`/`b`/`e` suffix or a missing comma ends the list; the
/// caller diagnoses whatever is left.
unsafe fn read_pattern_offsets(ci: &mut synpat_T, mut end: *mut c_char) -> *mut c_char {
    loop {
        let Some(mut idx) = (unsafe { offset_name(end) }) else {
            return end;
        };
        let slot = idx as usize;

        // An offset applies to the match's start unless it names `e`, which
        // selects the second half of the flag word.
        if idx != SPO_LC_OFF {
            match unsafe { *end.add(3) } as u8 {
                b's' | b'b' => {}
                b'e' => idx += SPO_COUNT,
                _ => return end,
            }
        }
        ci.sp_off_flags |= (1 << idx) as int16_t;

        if idx == SPO_LC_OFF {
            // lc=99
            end = unsafe { end.add(3) };
            let n = unsafe { getdigits_int(&raw mut end, true, 0) };
            ci.sp_offsets[slot] = n;
            // An "lc=" offset automatically sets the "ms=" offset.
            if ci.sp_off_flags as c_int & (1 << SPO_MS_OFF) == 0 {
                ci.sp_off_flags |= (1 << SPO_MS_OFF) as int16_t;
                ci.sp_offsets[SPO_MS_OFF as usize] = n;
            }
        } else {
            // yy=x+99
            end = unsafe { end.add(4) };
            if unsafe { *end } as c_int == '+' as c_int {
                end = unsafe { end.add(1) };
                ci.sp_offsets[slot] = unsafe { getdigits_int(&raw mut end, true, 0) };
            } else if unsafe { *end } as c_int == '-' as c_int {
                end = unsafe { end.add(1) };
                ci.sp_offsets[slot] = -unsafe { getdigits_int(&raw mut end, true, 0) };
            }
        }

        if unsafe { *end } as c_int != ',' as c_int {
            return end;
        }
        end = unsafe { end.add(1) };
    }
}
