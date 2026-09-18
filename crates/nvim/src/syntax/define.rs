//! `:syntax match`, `:syntax region` and `:syntax include`.
//!
//! The three subcommands that add a pattern-based item, plus
//! [`read_pattern`], which parses one `/pat/` with its `ms=`/`me=`/... offset
//! suffixes into a `SynPat`. `:syntax include` is here too: it sources another
//! syntax file under an inclusion tag so its toplevel items become contained
//! ones.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::message_fmt::msg_bytes;
use crate::option::SavedCpo;
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
pub(crate) fn syn_cmd_include(args: &mut ExArg, _syncing: c_int) {
    let mut sgl_id = 1;

    args.line.next = args.line.find_next(args.line.arg);
    if args.skip {
        return;
    }

    if args.line.byte_at(args.line.arg) == b'@' {
        let at = args.line.arg + 1;
        let line = args.line.rest_of(at);
        let Some(name) = split_group_name(line) else {
            emsg(gettext(c"E397: Filename required"));
            return;
        };
        sgl_id = syn_check_cluster(&line[..name.len]);
        if sgl_id == 0 {
            return;
        }
        // `separate_nextcmd` and `expand_filename` depend on this.
        args.line.arg = at + name.rest;
    }

    // Everything left, up to the next command, is the file to include.
    args.argt |= ExArgt::XFILE | ExArgt::NOSPC;
    separate_nextcmd(args);

    // An absolute path, "$VIM/.." or "<sfile>.." is `:source`d, which needs
    // the name expanded first; everything else goes through `:runtime!`.
    let source = matches!(args.line.byte_at(args.line.arg), b'<' | b'$')
        || path_is_absolute(args.line.cstr_from(args.line.arg));
    if source {
        let mut errormsg = None;
        if expand_filename(args, &mut errormsg).is_err() {
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

    let arg = args.line.cstr_from(args.line.arg);
    let none = ::core::ptr::null_mut();
    let failed = if source {
        // SAFETY: sourcing the file the user named.
        unsafe { do_source(arg.as_ptr().cast_mut(), false, DOSO_NONE as c_int, none) == FAIL }
    } else {
        // SAFETY: as above -- the name is NUL-terminated and only read.
        unsafe { source_runtime(arg.as_ptr().cast_mut(), RuntimeOpts::ALL) }.is_err()
    };
    if failed {
        let arg = msg_bytes(args.line.arg());
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
fn item_opt(takes_sync_idx: bool) -> SynOptArg {
    SynOptArg {
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
pub(crate) fn syn_cmd_match(args: &mut ExArg, syncing: c_int) {
    let line = args.line.arg().to_vec();
    let mut conceal_char: c_int = NUL;
    let mut opt = item_opt(syncing != 0);
    let mut item = EMPTY_SYNPAT;

    // Isolate the group name, then the options before the pattern, the
    // pattern, and the options after it.
    let name = split_group_name(&line);
    let mut end = name.as_ref().and_then(|name| {
        let at = read_item_options(&line, name.rest, &mut opt, &mut conceal_char, args.skip)?;
        let at = read_pattern(&line, at, &mut item)?;
        if vim_regcomp_had_eol() != 0 && !opt.flags.has(SynFlags::EXCLUDENL) {
            opt.flags |= SynFlags::HAS_EOL;
        }
        read_item_options(&line, at, &mut opt, &mut conceal_char, args.skip)
    });

    let mut stored = false;
    if let Some(at) = end {
        // Check for a trailing command and illegal trailing arguments.
        args.line.next = args.line.check_next(args.line.arg + at);
        if ends_excmd(c_int::from(cstr::byte_at(&line, at))) == 0 || args.skip {
            end = None;
        } else {
            let name_len = name.as_ref().map_or(0, |name| name.len);
            let syn_id = syn_check_group(&line[..name_len]);
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
    if !stored && end.is_none() {
        let shown = msg_bytes(&line);
        semsg!("E475: Invalid argument: {shown}");
    }
}

/// One start/skip/end pattern of a `:syntax region`, with the `matchgroup=`
/// that was in force when it was read.
struct RegionPat {
    pat: SynPat,
    matchgroup_id: c_int,
}

/// What `:syntax region` parsing produced, or why it stopped.
struct RegionArgs {
    /// The start, skip and end patterns, indexed by `ITEM_*`, each in
    /// **reverse** command order: upstream prepends to a linked list because
    /// "the list is used from end to start".
    pats: [Vec<RegionPat>; 3],
    opt: SynOptArg,
    conceal_char: c_int,
    /// Where parsing stopped, or `None` after an error.
    end: Option<usize>,
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
///
/// `line` is the whole rest of the command line and `at` the offset of the
/// first argument after the group name.
fn parse_region_args(args: &mut ExArg, line: &[u8], at: Option<usize>) -> RegionArgs {
    let mut out = RegionArgs {
        pats: [Vec::new(), Vec::new(), Vec::new()],
        opt: item_opt(false),
        conceal_char: NUL,
        end: at,
        not_enough: false,
    };
    let mut matchgroup_id = 0;
    let mut illegal = false;
    let byte = |at: usize| c_int::from(cstr::byte_at(line, at));

    let mut cursor = at;
    while let Some(mut at) = cursor.filter(|&at| ends_excmd(byte(at)) == 0) {
        // Options may appear anywhere between the patterns.
        cursor = read_item_options(line, at, &mut out.opt, &mut out.conceal_char, args.skip);
        match cursor {
            Some(next) if ends_excmd(byte(next)) == 0 => at = next,
            _ => break,
        }

        // Must be a pattern keyword or `matchgroup` then.
        let mut key_end = at;
        while !matches!(cstr::byte_at(line, key_end), 0 | b'=') && !ascii_iswhite(byte(key_end)) {
            key_end += 1;
        }
        let Some(item) = region_item(&line[at..key_end]) else {
            break;
        };
        if item == ITEM_SKIP && !out.pats[ITEM_SKIP as usize].is_empty() {
            illegal = true; // Only one skip pattern is allowed.
            break;
        }

        at = key_end + skip::white(&line[key_end..]);
        if byte(at) != '=' as c_int {
            cursor = None;
            let shown = msg_bytes(line);
            semsg!("E398: Missing '=': {shown}");
            break;
        }
        at += 1;
        at += skip::white(&line[at..]);
        if byte(at) == NUL {
            out.not_enough = true;
            break;
        }

        if item == ITEM_MATCHGROUP {
            let name_end = at + skip::to_white(&line[at..]);
            if &line[at..name_end] == b"NONE" || args.skip {
                matchgroup_id = 0;
            } else {
                matchgroup_id = syn_check_group(&line[at..name_end]);
                if matchgroup_id == 0 {
                    illegal = true;
                    break;
                }
            }
            cursor = Some(name_end + skip::white(&line[name_end..]));
            continue;
        }

        // Enable the appropriate `\z` specials: a start pattern defines the
        // external matches, skip and end patterns use them.
        reg_do_extmatch.set(if item == ITEM_START { REX_SET } else { REX_USE });
        let mut pat = EMPTY_SYNPAT;
        cursor = read_pattern(line, at, &mut pat);
        reg_do_extmatch.set(0);
        if item == ITEM_END && vim_regcomp_had_eol() != 0 && !out.opt.flags.has(SynFlags::EXCLUDENL)
        {
            pat.sp_flags |= SynFlags::HAS_EOL;
        }
        out.pats[item as usize].insert(0, RegionPat { pat, matchgroup_id });
    }

    // An `illegal` stop is reported as E390, which is what upstream's
    // "rest = NULL" here and its `illegal || rest == NULL` test below say.
    out.end = if illegal || out.not_enough {
        None
    } else {
        cursor
    };
    out
}

/// `:syntax region {group} [matchgroup={group}] start={pat} .. [skip={pat}]
/// end={pat} .. [{options}]`.
pub(crate) fn syn_cmd_region(args: &mut ExArg, syncing: c_int) {
    let line = args.line.arg().to_vec();

    // Isolate the group name, check for validity.
    let name = split_group_name(&line);

    let mut parsed = parse_region_args(args, &line, name.as_ref().map(|name| name.rest));
    let mut end = parsed.end;

    // Must have a "start" and an "end" pattern.
    if end.is_some()
        && (parsed.pats[ITEM_START as usize].is_empty()
            || parsed.pats[ITEM_END as usize].is_empty())
    {
        parsed.not_enough = true;
        end = None;
    }

    if let Some(at) = end {
        // Check for trailing garbage or a command; if OK, add the item.
        args.line.next = args.line.check_next(args.line.arg + at);
        if ends_excmd(c_int::from(cstr::byte_at(&line, at))) == 0 || args.skip {
            end = None;
        } else {
            let name_len = name.as_ref().map_or(0, |name| name.len);
            let syn_id = syn_check_group(&line[..name_len]);
            if syn_id != 0 {
                syn_incl_toplevel(syn_id, &mut parsed.opt.flags);
                store_region(parsed, syn_id, syncing != 0);
                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all(cur_syn_block()); // Need to recompute all.
                return; // the patterns and the lists belong to the block now
            }
        }
    }

    // Nothing was stored: dropping `parsed` releases every parsed pattern, its
    // compiled program and the three lists.
    let shown = msg_bytes(&line);
    if parsed.not_enough {
        semsg!("E399: Not enough arguments: syntax region {shown}");
    } else if end.is_none() {
        semsg!("E475: Invalid argument: {shown}");
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

/// Read the delimited pattern at `line[at..]`, plus its offsets, into `ci`.
///
/// Answers the offset of what follows it, or `None` after reporting an error.
pub(crate) fn read_pattern(line: &[u8], at: usize, ci: &mut SynPat) -> Option<usize> {
    // Need at least three characters: two delimiters and something between.
    let body = line.get(at..)?;
    if body.len() < 3 {
        return None;
    }
    let delimiter = body[0];

    // One writable NUL-terminated copy of the rest of the line. `skip_regexp`
    // and `getdigits_int` are pointer walks with no slice form, and this is
    // what they walk: offsets into it are offsets into `line[at..]`, and
    // nothing borrowed from the caller is handed to a raw pointer.
    let mut tail: Vec<u8> = Vec::with_capacity(body.len() + 1);
    tail.extend_from_slice(body);
    tail.push(NUL as u8);

    // SAFETY: `tail` is NUL-terminated and outlives the walk.
    let stop = unsafe { skip_regexp(tail.as_mut_ptr().add(1).cast(), delimiter as c_int, 1) };
    // SAFETY: both pointers are into `tail`, the base first.
    let end = unsafe { stop.cast::<u8>().offset_from(tail.as_ptr()) } as usize;
    if cstr::byte_at(&tail, end) != delimiter {
        let shown = msg_bytes(body);
        semsg!("E401: Pattern delimiter not found: {shown}");
        return None;
    }

    // Store the pattern and its compiled program. 'cpoptions' is emptied
    // first, to avoid the 'l' flag.
    let pattern = cstr::owned(&tail[1..end]);
    let _cpo = SavedCpo::empty();
    // SAFETY: an owned NUL-terminated copy of the pattern text.
    ci.sp_prog = unsafe { vim_regcomp(pattern.as_ptr().cast_mut(), RE_MAGIC) };
    ci.sp_pattern = Some(pattern);
    if ci.sp_prog.is_null() {
        return None;
    }
    ci.sp_ic = cur_syn_block().b_syn_ic;
    syn_clear_time(&mut ci.sp_time);

    let end = read_pattern_offsets(ci, &mut tail, end + 1);
    let after = c_int::from(cstr::byte_at(&tail, end));
    if ends_excmd(after) == 0 && !ascii_iswhite(after) {
        let shown = msg_bytes(body);
        semsg!("E402: Garbage after pattern: {shown}");
        return None;
    }
    Some(at + end + skip::white(&line[at + end..]))
}

/// The offset names, indexed by `SPO_*`.
pub(crate) static SPO_NAME_TAB: [&CStr; SPO_COUNT as usize] =
    [c"ms=", c"me=", c"hs=", c"he=", c"rs=", c"re=", c"lc="];

/// Which `SPO_*` offset the three bytes at `at` name.
fn offset_name(text: &[u8], at: usize) -> Option<c_int> {
    let head = text.get(at..at + 3)?;
    let idx = SPO_NAME_TAB
        .iter()
        .rposition(|name| name.to_bytes() == head)?;
    Some(idx as c_int)
}

/// Read the comma-separated `ms=s+1,he=e-2,lc=3` offsets after a pattern.
///
/// Answers the offset of the first byte that is not part of them. An
/// unrecognised name, an unrecognised `s`/`b`/`e` suffix or a missing comma
/// ends the list; the caller diagnoses whatever is left.
fn read_pattern_offsets(ci: &mut SynPat, text: &mut [u8], mut at: usize) -> usize {
    loop {
        let Some(mut idx) = offset_name(text, at) else {
            return at;
        };
        let slot = idx as usize;

        // An offset applies to the match's start unless it names `e`, which
        // selects the second half of the flag word.
        if idx != SPO_LC_OFF {
            match cstr::byte_at(text, at + 3) {
                b's' | b'b' => {}
                b'e' => idx += SPO_COUNT,
                _ => return at,
            }
        }
        ci.sp_off_flags |= (1 << idx) as int16_t;

        if idx == SPO_LC_OFF {
            // lc=99
            let (n, past) = getdigits_int_at(text, at + 3, true, 0);
            at = past;
            ci.sp_offsets[slot] = n;
            // An "lc=" offset automatically sets the "ms=" offset.
            if ci.sp_off_flags as c_int & (1 << SPO_MS_OFF) == 0 {
                ci.sp_off_flags |= (1 << SPO_MS_OFF) as int16_t;
                ci.sp_offsets[SPO_MS_OFF as usize] = n;
            }
        } else {
            // yy=x+99
            at += 4;
            let sign = match cstr::byte_at(text, at) {
                b'+' => 1,
                b'-' => -1,
                _ => 0,
            };
            if sign != 0 {
                let (n, past) = getdigits_int_at(text, at + 1, true, 0);
                at = past;
                ci.sp_offsets[slot] = sign * n;
            }
        }

        if cstr::byte_at(text, at) != b',' {
            return at;
        }
        at += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The offsets `text` names, read as `read_pattern` would read them.
    fn offsets(text: &str) -> (SynPat, usize) {
        let mut buffer: Vec<u8> = text.bytes().chain([NUL as u8]).collect();
        let mut pat = EMPTY_SYNPAT;
        let at = read_pattern_offsets(&mut pat, &mut buffer, 0);
        (pat, at)
    }

    #[test]
    fn a_start_offset_sets_its_own_slot() {
        let (pat, at) = offsets("ms=s+1");
        assert_eq!(at, 6);
        assert_eq!(pat.sp_offsets[SPO_MS_OFF as usize], 1);
        assert_eq!(
            pat.sp_off_flags as c_int & (1 << SPO_MS_OFF),
            1 << SPO_MS_OFF
        );
    }

    #[test]
    fn an_end_suffix_selects_the_second_half_of_the_flag_word() {
        let (pat, at) = offsets("he=e-2");
        assert_eq!(at, 6);
        assert_eq!(pat.sp_offsets[SPO_HE_OFF as usize], -2);
        let from_end = SPO_HE_OFF + SPO_COUNT;
        assert_eq!(pat.sp_off_flags as c_int & (1 << from_end), 1 << from_end);
        assert_eq!(pat.sp_off_flags as c_int & (1 << SPO_HE_OFF), 0);
    }

    #[test]
    fn several_offsets_separated_by_commas_all_land() {
        let (pat, at) = offsets("ms=s+1,me=e-1,hs=s,re=e+3");
        assert_eq!(at, 25);
        assert_eq!(pat.sp_offsets[SPO_MS_OFF as usize], 1);
        assert_eq!(pat.sp_offsets[SPO_ME_OFF as usize], -1);
        // A bare `s` with no number leaves the offset at zero.
        assert_eq!(pat.sp_offsets[SPO_HS_OFF as usize], 0);
        assert_eq!(pat.sp_offsets[SPO_RE_OFF as usize], 3);
    }

    #[test]
    fn lc_takes_no_suffix_and_seeds_ms() {
        let (pat, at) = offsets("lc=3");
        assert_eq!(at, 4);
        assert_eq!(pat.sp_offsets[SPO_LC_OFF as usize], 3);
        assert_eq!(pat.sp_offsets[SPO_MS_OFF as usize], 3);
        assert_eq!(
            pat.sp_off_flags as c_int & (1 << SPO_MS_OFF),
            1 << SPO_MS_OFF
        );
    }

    #[test]
    fn an_ms_already_given_is_not_overwritten_by_lc() {
        let (pat, _) = offsets("ms=s+9,lc=3");
        assert_eq!(pat.sp_offsets[SPO_MS_OFF as usize], 9);
        assert_eq!(pat.sp_offsets[SPO_LC_OFF as usize], 3);
    }

    #[test]
    fn the_list_stops_at_anything_it_does_not_recognise() {
        // An unknown name.
        assert_eq!(offsets("xx=s+1").1, 0);
        // A known name with an unknown suffix.
        assert_eq!(offsets("ms=q+1").1, 0);
        // No comma after a complete offset: the caller diagnoses the rest.
        assert_eq!(offsets("ms=s+1 contained").1, 6);
        // Nothing at all, and a name the line is too short to hold.
        assert_eq!(offsets("").1, 0);
        assert_eq!(offsets("ms").1, 0);
    }

    #[test]
    fn every_region_keyword_is_recognised_ignoring_case() {
        assert_eq!(region_item(b"matchgroup"), Some(ITEM_MATCHGROUP));
        assert_eq!(region_item(b"MatchGroup"), Some(ITEM_MATCHGROUP));
        assert_eq!(region_item(b"start"), Some(ITEM_START));
        assert_eq!(region_item(b"skip"), Some(ITEM_SKIP));
        assert_eq!(region_item(b"END"), Some(ITEM_END));
        assert_eq!(region_item(b"contained"), None);
        assert_eq!(region_item(b""), None);
    }
}
