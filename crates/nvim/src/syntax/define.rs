//! `:syntax match`, `:syntax region` and `:syntax include`.
//!
//! The three subcommands that add a pattern-based item, plus
//! [`get_syn_pattern`], which parses one `/pat/` with its `ms=`/`me=`/... offset
//! suffixes into a `synpat_T`. `:syntax include` is here too: it sources another
//! syntax file under an inclusion tag so its toplevel items become contained
//! ones.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::optionstr::empty_option;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::regexp::RE_MAGIC;
use crate::runtime::RuntimeOpts;
use crate::types::{ExArgt, FAIL, NUL};

/// Adjust an item's flags when it is declared in a `:syntax include`d file.
///
/// Sets the contained flag, and if the item is not already contained adds it to
/// the top-level cluster the `:syntax include` named, if any.
pub(crate) fn syn_incl_toplevel(id: c_int, flags: &mut SynFlags) {
    if flags.has(SynFlags::CONTAINED) || cur_syn_block().b_syn_topgrp == 0 {
        return;
    }
    *flags |= SynFlags::CONTAINED | SynFlags::INCLUDED_TOPLEVEL;
    if cur_syn_block().b_syn_topgrp >= SYNID_CLUSTER {
        let tlg_id = (cur_syn_block().b_syn_topgrp - SYNID_CLUSTER) as usize;
        let mut block = cur_syn_block();
        let list = &mut block.clusters_mut()[tlg_id].scl_list;
        syn_combine_list(list, IdList::from_ids(&[id as int16_t]), CLUSTER_ADD);
    }
}

/// `:syntax include [@{cluster}] {file}`.
pub(crate) fn syn_cmd_include(eap: &mut exarg_T, _syncing: c_int) {
    let mut arg = eap.arg;
    let mut sgl_id = 1;

    eap.nextcmd = unsafe { find_nextcmd(arg) };
    if eap.skip != 0 {
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
        eap.arg = rest;
    }

    // Everything left, up to the next command, is the file to include.
    eap.argt |= ExArgt::XFILE | ExArgt::NOSPC;
    unsafe { separate_nextcmd(eap) };

    // An absolute path, "$VIM/.." or "<sfile>.." is `:source`d, which needs
    // the name expanded first; everything else goes through `:runtime!`.
    let source = unsafe { *eap.arg } as c_int == '<' as c_int
        || unsafe { *eap.arg } as c_int == '$' as c_int
        || unsafe { path_is_absolute(eap.arg) };
    if source {
        let mut errormsg = None;
        if unsafe { expand_filename(eap, syn_cmdlinep.get(), &mut errormsg) }.is_err() {
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
    let arg = eap.arg;
    let failed = if source {
        // SAFETY: sourcing the file the user named.
        unsafe { do_source(arg, false, DOSO_NONE as c_int, ::core::ptr::null_mut()) == FAIL }
    } else {
        unsafe { source_runtime(eap.arg, RuntimeOpts::ALL) }.is_err()
    };
    if failed {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(eap.arg) };
        semsg!("E484: Can't open file {arg}");
    }

    cur_syn_block().b_syn_topgrp = prev_toplvl_grp;
    current_syn_inc_tag.set(prev_syn_inc_tag);
}

/// The default options for `:syntax match` and `:syntax region`, both of which
/// accept a `contains=` list.
///
/// `takes_sync_idx` is what `grouphere`/`groupthere` needs, and only
/// `:syntax sync match` sets it.
fn item_opt(takes_sync_idx: bool) -> syn_opt_arg_T {
    syn_opt_arg_T {
        flags: SynFlags::NONE,
        keyword: false,
        takes_sync_idx,
        sync_idx: 0,
        has_cont_list: true,
        cont_list: IdList::NONE,
        cont_in_list: IdList::NONE,
        next_list: IdList::NONE,
    }
}

/// `:syntax match {group} [{options}] {pattern} [{options}]`, and
/// `:syntax sync match {group} [[grouphere|groupthere] {group}] ..`.
pub(crate) fn syn_cmd_match(eap: &mut exarg_T, syncing: c_int) {
    let arg = eap.arg;
    let mut group_name_end = ::core::ptr::null_mut::<c_char>();
    let mut conceal_char: c_int = NUL;

    // Isolate the group name, check for validity.
    let mut rest = unsafe { get_group_name(arg, &mut group_name_end) };

    let mut opt = item_opt(syncing != 0);

    // Options before the pattern, the pattern, then options after it.
    rest = unsafe { get_syn_options(rest, &mut opt, &mut conceal_char, eap.skip) };
    let mut item = EMPTY_SYNPAT;
    rest = unsafe { get_syn_pattern(rest, &mut item) };
    if vim_regcomp_had_eol() != 0 && !opt.flags.has(SynFlags::EXCLUDENL) {
        opt.flags |= SynFlags::HAS_EOL;
    }
    rest = unsafe { get_syn_options(rest, &mut opt, &mut conceal_char, eap.skip) };

    let mut stored = false;
    if !rest.is_null() {
        // Check for a trailing command and illegal trailing arguments.
        eap.nextcmd = unsafe { check_nextcmd(rest) };
        if ends_excmd(unsafe { *rest } as c_int) == 0 || eap.skip != 0 {
            rest = ::core::ptr::null_mut();
        } else {
            let syn_id = unsafe { syn_check_group(arg, group_name_end.offset_from(arg) as size_t) };
            if syn_id != 0 {
                syn_incl_toplevel(syn_id, &mut opt.flags);
                // Store the pattern in the item list; the three id lists are
                // handed over rather than copied.
                item.sp_syncing = syncing != 0;
                item.sp_type = SPTYPE_MATCH as c_char;
                item.sp_syn.id = syn_id as int16_t;
                item.sp_syn.inc_tag = current_syn_inc_tag.get();
                item.sp_flags = opt.flags;
                item.sp_sync_idx = opt.sync_idx;
                item.sp_cchar = conceal_char;
                if !opt.cont_in_list.is_none() {
                    cur_syn_block().b_syn_containedin = 1;
                }
                item.sp_cont_list = ::core::mem::take(&mut opt.cont_list);
                item.sp_cont_in_list = ::core::mem::take(&mut opt.cont_in_list);
                item.sp_next_list = ::core::mem::take(&mut opt.next_list);
                cur_syn_block().patterns_mut().push(item);
                stored = true;

                // Remember that we found a match to sync on.
                if opt.flags.has(SynFlags::SYNC_HERE | SynFlags::SYNC_THERE) {
                    cur_syn_block().b_syn_sync_flags |= SF_MATCH;
                }
                if opt.flags.has(SynFlags::FOLD) {
                    cur_syn_block().b_syn_folditems += 1;
                }

                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all(cur_syn_block()); // Need to recompute all.
            }
        }
    }

    // Something failed: dropping `item` and `opt` releases the pattern text,
    // the compiled program and the three lists.
    if !stored && rest.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {arg}");
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

/// Which of the four keywords `key` names, ignoring case.
fn region_item(key: &[u8]) -> Option<c_int> {
    [
        (&b"MATCHGROUP"[..], ITEM_MATCHGROUP),
        (b"START", ITEM_START),
        (b"END", ITEM_END),
        (b"SKIP", ITEM_SKIP),
    ]
    .into_iter()
    .find(|(name, _)| key.eq_ignore_ascii_case(name))
    .map(|(_, item)| item)
}

/// Read the options, patterns and `matchgroup=`s of a `:syntax region`.
fn parse_region_args(eap: &mut exarg_T, mut rest: *mut c_char) -> RegionArgs {
    let mut out = RegionArgs {
        pats: [Vec::new(), Vec::new(), Vec::new()],
        opt: item_opt(false),
        conceal_char: NUL,
        rest,
        not_enough: false,
    };
    let mut matchgroup_id = 0;
    let mut illegal = false;

    while !rest.is_null() && ends_excmd(unsafe { *rest } as c_int) == 0 {
        // Options may appear anywhere between the patterns.
        rest = unsafe { get_syn_options(rest, &mut out.opt, &mut out.conceal_char, eap.skip) };
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
        // SAFETY: both pointers are into the command line, `rest` first.
        let key = unsafe { cstr::slice_at(rest, key_end.offset_from(rest) as usize) };
        let Some(item) = region_item(key) else {
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
            let arg = unsafe { c_str(eap.arg) };
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
            if (unsafe { p.offset_from(rest) } == 4 && unsafe { cstr::starts_with(rest, b"NONE") })
                || eap.skip != 0
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
        let mut pat = EMPTY_SYNPAT;
        rest = unsafe { get_syn_pattern(rest, &mut pat) };
        reg_do_extmatch.set(0);
        if item == ITEM_END && vim_regcomp_had_eol() != 0 && !out.opt.flags.has(SynFlags::EXCLUDENL)
        {
            pat.sp_flags |= SynFlags::HAS_EOL;
        }
        out.pats[item as usize].insert(0, RegionPat { pat, matchgroup_id });
    }

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
pub(crate) fn syn_cmd_region(eap: &mut exarg_T, syncing: c_int) {
    let arg = eap.arg;
    let mut group_name_end = ::core::ptr::null_mut::<c_char>();

    // Isolate the group name, check for validity.
    let rest = unsafe { get_group_name(arg, &mut group_name_end) };

    let mut args = parse_region_args(eap, rest);
    let mut rest = args.rest;

    // Must have a "start" and an "end" pattern.
    if !rest.is_null()
        && (args.pats[ITEM_START as usize].is_empty() || args.pats[ITEM_END as usize].is_empty())
    {
        args.not_enough = true;
        rest = ::core::ptr::null_mut();
    }

    if !rest.is_null() {
        // Check for trailing garbage or a command; if OK, add the item.
        eap.nextcmd = unsafe { check_nextcmd(rest) };
        if ends_excmd(unsafe { *rest } as c_int) == 0 || eap.skip != 0 {
            rest = ::core::ptr::null_mut();
        } else {
            let syn_id = unsafe { syn_check_group(arg, group_name_end.offset_from(arg) as size_t) };
            if syn_id != 0 {
                syn_incl_toplevel(syn_id, &mut args.opt.flags);
                store_region(args, syn_id, syncing != 0);
                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all(cur_syn_block()); // Need to recompute all.
                return; // the patterns and the lists belong to the block now
            }
        }
    }

    // Nothing was stored: dropping `args` releases every parsed pattern, its
    // compiled program and the three lists.
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

/// Copy the parsed start/skip/end patterns into the block's pattern array as
/// consecutive entries.
///
/// The `contains=`/`containedin=`/`nextgroup=` lists go on the START entries
/// only, and are handed over rather than copied — which is why the caller must
/// not free them once this has run.
fn store_region(args: RegionArgs, syn_id: c_int, syncing: bool) {
    let RegionArgs {
        mut pats,
        opt,
        conceal_char,
        ..
    } = args;
    let mut block = cur_syn_block();
    if !opt.cont_in_list.is_none() {
        block.b_syn_containedin = 1;
    }
    for item in [ITEM_START, ITEM_SKIP, ITEM_END] {
        for entry in ::core::mem::take(&mut pats[item as usize]) {
            let mut spp = entry.pat;
            spp.sp_syncing = syncing;
            spp.sp_type = if item == ITEM_START {
                SPTYPE_START
            } else if item == ITEM_SKIP {
                SPTYPE_SKIP
            } else {
                SPTYPE_END
            } as c_char;
            spp.sp_flags |= opt.flags;
            spp.sp_syn.id = syn_id as int16_t;
            spp.sp_syn.inc_tag = current_syn_inc_tag.get();
            spp.sp_syn_match_id = entry.matchgroup_id as int16_t;
            spp.sp_cchar = conceal_char;
            if item == ITEM_START {
                // Every START of the region gets its own copy: upstream
                // gave them one list, owned by the first, and freed the
                // array last to first so it could tell which that was.
                spp.sp_cont_list = opt.cont_list.clone();
                spp.sp_cont_in_list = opt.cont_in_list.clone();
                spp.sp_next_list = opt.next_list.clone();
            }
            block.patterns_mut().push(spp);
            if opt.flags.has(SynFlags::FOLD) {
                block.b_syn_folditems += 1;
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
    // SAFETY: both pointers are into the command line, `arg` first.
    let pattern = unsafe { name_at(arg.add(1), end.offset_from(arg) as usize - 1) };
    let cpo_save = p_cpo.get();
    p_cpo.set(empty_option());
    ci.sp_prog = unsafe { vim_regcomp(pattern.as_ptr().cast_mut(), RE_MAGIC) };
    p_cpo.set(cpo_save);
    ci.sp_pattern = Some(pattern);
    if ci.sp_prog.is_null() {
        return ::core::ptr::null_mut();
    }
    ci.sp_ic = cur_syn_block().b_syn_ic;
    syn_clear_time(&mut ci.sp_time);

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
        if unsafe { cstr::prefix_eq(end, SPO_NAME_TAB[idx as usize].as_ptr(), 3) } {
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
