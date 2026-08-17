//! `:syntax list` — the report.
//!
//! [`syn_cmd_list`] with no argument lists every item, with one lists the items
//! of a group. [`syn_list_one`] renders one group's keyword, match and region
//! items, [`put_pattern`] one pattern with its offsets, [`put_id_list`] a
//! `contains=`/`nextgroup=` list, and [`syn_list_cluster`] a cluster.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int};

use super::*;

/// Everything is listed in the "directory" highlight, as `:highlight` does.
const LIST_HL: c_int = HLF_D;

/// `:syntax [list] [{group}|@{cluster}] ..` and `:syntax sync` with no
/// argument.
pub(crate) unsafe extern "C" fn syn_cmd_list(eap: *mut exarg_T, syncing: c_int) {
    unsafe {
        let mut arg = (*eap).arg;

        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }

        msg_ext_set_kind(c"list_cmd".as_ptr());
        if !syntax_present(curwin.get()) {
            msg(gettext(MSG_NO_ITEMS.as_ptr()), 0);
            return;
        }

        if syncing != 0 {
            list_sync_items();
            return;
        }

        msg_puts_title(gettext(c"\n--- Syntax items ---".as_ptr()));
        if ends_excmd(*arg as c_int) != 0 {
            // No argument: list every group id, then every cluster.
            let mut id = 1;
            while id <= highlight_num_groups() && !got_int.get() {
                syn_list_one(id, false, false);
                id += 1;
            }
            let mut id = 0;
            while id < cur_cluster_count() && !got_int.get() {
                syn_list_cluster(id);
                id += 1;
            }
        } else {
            // List the groups and clusters the argument names.
            while ends_excmd(*arg as c_int) == 0 && !got_int.get() {
                let arg_end = skiptowhite(arg);
                if *arg as c_int == '@' as c_int {
                    let id = syn_scl_namen2id(arg.add(1), arg_end.offset_from(arg) as c_int - 1);
                    if id == 0 {
                        semsg_c!(gettext(c"E392: No such syntax cluster: %s".as_ptr()), arg);
                    } else {
                        syn_list_cluster(id - SYNID_CLUSTER);
                    }
                } else {
                    let id = syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                    if id == 0 {
                        semsg_c!(gettext(&raw const e_nogroup as *const c_char), arg);
                    } else {
                        syn_list_one(id, false, true);
                    }
                }
                arg = skipwhite(arg_end);
            }
        }
        (*eap).nextcmd = check_nextcmd(arg);
    }
}

/// The `:syntax sync` half of the listing: how this buffer synchronises.
unsafe fn list_sync_items() {
    unsafe {
        let block = cur_syn_block();
        if (*block).b_syn_sync_flags & SF_CCOMMENT != 0 {
            msg_puts(gettext(c"syncing on C-style comments".as_ptr()));
            syn_lines_msg();
            syn_match_msg();
        } else if (*block).b_syn_sync_flags & SF_MATCH != 0 {
            msg_puts_title(gettext(c"\n--- Syntax sync items ---".as_ptr()));
            if (*block).b_syn_sync_minlines > 0
                || (*block).b_syn_sync_maxlines > 0
                || (*block).b_syn_sync_linebreaks > 0
            {
                msg_puts(gettext(c"\nsyncing on items".as_ptr()));
                syn_lines_msg();
                syn_match_msg();
            }
            let mut id = 1;
            while id <= highlight_num_groups() && !got_int.get() {
                syn_list_one(id, true, false);
                id += 1;
            }
        } else if (*block).b_syn_sync_minlines == 0 {
            msg_puts(gettext(c"no syncing".as_ptr()));
        } else {
            if (*block).b_syn_sync_minlines == MAXLNUM as linenr_T {
                msg_puts(gettext(c"syncing starts at the first line".as_ptr()));
            } else {
                msg_puts(gettext(c"syncing starts ".as_ptr()));
                msg_outnum((*block).b_syn_sync_minlines);
                msg_puts(gettext(c" lines before top line".as_ptr()));
            }
            syn_match_msg();
        }
    }
}

/// "; minimal 5, maximal 10 lines before top line".
unsafe fn syn_lines_msg() {
    unsafe {
        let block = cur_syn_block();
        if (*block).b_syn_sync_maxlines <= 0 && (*block).b_syn_sync_minlines <= 0 {
            return;
        }
        msg_puts(c"; ".as_ptr());
        if (*block).b_syn_sync_minlines == MAXLNUM as linenr_T {
            msg_puts(gettext(c"from the first line".as_ptr()));
            return;
        }
        if (*block).b_syn_sync_minlines > 0 {
            msg_puts(gettext(c"minimal ".as_ptr()));
            msg_outnum((*block).b_syn_sync_minlines);
            if (*block).b_syn_sync_maxlines != 0 {
                msg_puts(c", ".as_ptr());
            }
        }
        if (*block).b_syn_sync_maxlines > 0 {
            msg_puts(gettext(c"maximal ".as_ptr()));
            msg_outnum((*block).b_syn_sync_maxlines);
        }
        msg_puts(gettext(c" lines before top line".as_ptr()));
    }
}

/// "; match 3 line breaks".
unsafe fn syn_match_msg() {
    unsafe {
        let linebreaks = (*cur_syn_block()).b_syn_sync_linebreaks;
        if linebreaks > 0 {
            msg_puts(gettext(c"; match ".as_ptr()));
            msg_outnum(linebreaks);
            msg_puts(gettext(c" line breaks".as_ptr()));
        }
    }
}

/// The item flags that are printed after the patterns.
const ITEM_FLAG_NAMES: [(c_int, &CStr); 10] = [
    (HL_DISPLAY, c"display"),
    (HL_CONTAINED, c"contained"),
    (HL_ONELINE, c"oneline"),
    (HL_KEEPEND, c"keepend"),
    (HL_EXTEND, c"extend"),
    (HL_EXCLUDENL, c"excludenl"),
    (HL_TRANSP, c"transparent"),
    (HL_FOLD, c"fold"),
    (HL_CONCEAL, c"conceal"),
    (HL_CONCEALENDS, c"concealends"),
];

/// The flags that are printed after a `nextgroup=` list.
const NEXTGROUP_FLAG_NAMES: [(c_int, &CStr); 3] = [
    (HL_SKIPWHITE, c"skipwhite"),
    (HL_SKIPNL, c"skipnl"),
    (HL_SKIPEMPTY, c"skipempty"),
];

/// Print the names of every flag of `names` that `flags` has.
unsafe fn syn_list_flags(names: &[(c_int, &CStr)], flags: c_int, hl_id: c_int) {
    unsafe {
        for (flag, name) in names {
            if flags & flag != 0 {
                msg_puts_hl(name.as_ptr(), hl_id, false);
                msg_putchar(' ' as c_int);
            }
        }
    }
}

/// List one syntax group: its keywords, its items, and the group it links to.
///
/// `syncing` lists the `:syntax sync` items instead of the ordinary ones;
/// `link_only` prints a group that has nothing but a link.
unsafe fn syn_list_one(id: c_int, syncing: bool, link_only: bool) {
    unsafe {
        let mut did_header = false;

        // The keywords of `id`, from both tables.
        if !syncing {
            did_header = syn_list_keywords(id, &raw const (*cur_syn_block()).b_keywtab, false);
            did_header =
                syn_list_keywords(id, &raw const (*cur_syn_block()).b_keywtab_ic, did_header);
        }

        // The patterns of `id`.
        let mut idx = 0;
        while idx < cur_pattern_count() && !got_int.get() {
            let spp = cur_pattern(idx);
            if (*spp).sp_syn.id as c_int != id || (*spp).sp_syncing != syncing {
                idx += 1;
                continue;
            }
            syn_list_header(did_header, 0, id, true);
            did_header = true;
            idx = put_item_patterns(idx);
            syn_list_flags(&ITEM_FLAG_NAMES, (*spp).sp_flags, LIST_HL);

            if !(*spp).sp_cont_list.is_null() {
                put_id_list(c"contains", (*spp).sp_cont_list, LIST_HL);
            }
            if !(*spp).sp_syn.cont_in_list.is_null() {
                put_id_list(c"containedin", (*spp).sp_syn.cont_in_list, LIST_HL);
            }
            if !(*spp).sp_next_list.is_null() {
                put_id_list(c"nextgroup", (*spp).sp_next_list, LIST_HL);
                syn_list_flags(&NEXTGROUP_FLAG_NAMES, (*spp).sp_flags, LIST_HL);
            }
            if (*spp).sp_flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
                put_sync_group(spp);
            }
            idx += 1;
        }

        // The link, if there is one.
        let link = highlight_link_id(id - 1);
        if link != 0 && (did_header || link_only) && !got_int.get() {
            syn_list_header(did_header, 0, id, true);
            msg_puts_hl(c"links to".as_ptr(), LIST_HL, false);
            msg_putchar(' ' as c_int);
            msg_outtrans(highlight_group_name(link - 1), 0, false);
        }
    }
}

/// Print the pattern(s) of the item at `idx`, answering the index of its last
/// pattern.
///
/// A match is one pattern; a region is a run of consecutive entries —
/// start(s), an optional skip, then end(s) — which are printed together.
unsafe fn put_item_patterns(mut idx: c_int) -> c_int {
    unsafe {
        let mut last_matchgroup = 0;
        let count = cur_pattern_count();
        let sp_type = |i: c_int| (*cur_pattern(i)).sp_type as c_int;

        if sp_type(idx) == SPTYPE_MATCH {
            put_pattern(
                &mut last_matchgroup,
                c"match",
                ' ' as c_int,
                cur_pattern(idx),
            );
            msg_putchar(' ' as c_int);
        } else if sp_type(idx) == SPTYPE_START {
            // The three loops bound themselves on `count`; upstream bounds only
            // the last of them and reads past the array if a region's END
            // entries are ever missing.
            while idx < count && sp_type(idx) == SPTYPE_START {
                put_pattern(
                    &mut last_matchgroup,
                    c"start",
                    '=' as c_int,
                    cur_pattern(idx),
                );
                idx += 1;
            }
            if idx < count && sp_type(idx) == SPTYPE_SKIP {
                put_pattern(
                    &mut last_matchgroup,
                    c"skip",
                    '=' as c_int,
                    cur_pattern(idx),
                );
                idx += 1;
            }
            while idx < count && sp_type(idx) == SPTYPE_END {
                put_pattern(&mut last_matchgroup, c"end", '=' as c_int, cur_pattern(idx));
                idx += 1;
            }
            idx -= 1;
            msg_putchar(' ' as c_int);
        }
        idx
    }
}

/// Print `grouphere`/`groupthere` and the region item it names.
unsafe fn put_sync_group(spp: *const synpat_T) {
    unsafe {
        if (*spp).sp_flags & HL_SYNC_HERE != 0 {
            msg_puts_hl(c"grouphere".as_ptr(), LIST_HL, false);
        } else {
            msg_puts_hl(c"groupthere".as_ptr(), LIST_HL, false);
        }
        msg_putchar(' ' as c_int);
        if (*spp).sp_sync_idx >= 0 {
            let target = cur_pattern((*spp).sp_sync_idx);
            msg_outtrans(
                highlight_group_name((*target).sp_syn.id as c_int - 1),
                0,
                false,
            );
        } else {
            msg_puts(c"NONE".as_ptr());
        }
        msg_putchar(' ' as c_int);
    }
}

/// List one cluster and its members.
unsafe fn syn_list_cluster(id: c_int) {
    unsafe {
        // Slight hack: roughly duplicate the guts of `syn_list_header`.
        let mut endcol = 15;
        msg_putchar('\n' as c_int);
        msg_outtrans((*cur_cluster(id)).scl_name, 0, false);

        if msg_col.get() >= endcol {
            endcol = msg_col.get() + 1; // output at least one space
        }
        if Columns.get() <= endcol {
            endcol = Columns.get() - 1; // avoid a hang for a tiny window
        }
        msg_advance(endcol);

        let list = (*cur_cluster(id)).scl_list;
        if !list.is_null() {
            put_id_list(c"cluster", list, LIST_HL);
        } else {
            msg_puts_hl(c"cluster".as_ptr(), LIST_HL, false);
            msg_puts(c"=NONE".as_ptr());
        }
    }
}

/// Print `name=a,b,@cl` for a `contains=`/`containedin=`/`nextgroup=` list.
unsafe fn put_id_list(name: &CStr, list: *const int16_t, hl_id: c_int) {
    unsafe {
        msg_puts_hl(name.as_ptr(), hl_id, false);
        msg_putchar('=' as c_int);
        let mut p = list;
        while *p != 0 {
            let item = *p as c_int;
            let more = *p.add(1) != 0;
            if item >= SYNID_ALLBUT && item < SYNID_TOP {
                // ALLBUT is the same marker as ALL, told apart by whether the
                // list goes on to name exceptions.
                msg_puts(if more { c"ALLBUT" } else { c"ALL" }.as_ptr());
            } else if item >= SYNID_TOP && item < SYNID_CONTAINED {
                msg_puts(c"TOP".as_ptr());
            } else if item >= SYNID_CONTAINED && item < SYNID_CLUSTER {
                msg_puts(c"CONTAINED".as_ptr());
            } else if item >= SYNID_CLUSTER {
                msg_putchar('@' as c_int);
                msg_outtrans((*cur_cluster(item - SYNID_CLUSTER)).scl_name, 0, false);
            } else {
                msg_outtrans(highlight_group_name(item - 1), 0, false);
            }
            if more {
                msg_putchar(',' as c_int);
            }
            p = p.add(1);
        }
        msg_putchar(' ' as c_int);
    }
}

/// The delimiters `put_pattern` will wrap a pattern in, best first.
const SEPCHARS: &[u8] = b"/+=-#@\"|'^&";

/// Print one pattern of an item: its keyword, the pattern between delimiters,
/// and its offsets.
///
/// `last_matchgroup` carries the `matchgroup=` in force across the patterns of
/// one item, so a change is printed once rather than per pattern.
unsafe fn put_pattern(last_matchgroup: &mut c_int, s: &CStr, c: c_int, spp: *const synpat_T) {
    unsafe {
        // May have to write "matchgroup=group".
        if *last_matchgroup != (*spp).sp_syn_match_id as c_int {
            *last_matchgroup = (*spp).sp_syn_match_id as c_int;
            msg_puts_hl(c"matchgroup".as_ptr(), LIST_HL, false);
            msg_putchar('=' as c_int);
            if *last_matchgroup == 0 {
                msg_outtrans(c"NONE".as_ptr(), 0, false);
            } else {
                msg_outtrans(highlight_group_name(*last_matchgroup - 1), 0, false);
            }
            msg_putchar(' ' as c_int);
        }

        // The name of the pattern and an '=' or ' '.
        msg_puts_hl(s.as_ptr(), LIST_HL, false);
        msg_putchar(c);

        // The pattern, wrapped in the first delimiter it does not itself
        // contain — or the first one of all, if it contains every one.
        let mut i = 0;
        while !vim_strchr((*spp).sp_pattern, SEPCHARS[i] as c_int).is_null() {
            i += 1;
            if i == SEPCHARS.len() {
                i = 0;
                break;
            }
        }
        msg_putchar(SEPCHARS[i] as c_int);
        msg_outtrans((*spp).sp_pattern, 0, false);
        msg_putchar(SEPCHARS[i] as c_int);

        put_pattern_offsets(spp);
        msg_putchar(' ' as c_int);
    }
}

/// Print the `ms=s+1,he=e-2,lc=3` offsets of one pattern.
unsafe fn put_pattern_offsets(spp: *const synpat_T) {
    unsafe {
        let mut first = true;
        for i in 0..SPO_COUNT {
            // A start offset and an end offset share one name; the flag word
            // holds the start half in its low `SPO_COUNT` bits.
            let mask = 1 << i;
            if (*spp).sp_off_flags as c_int & (mask + (mask << SPO_COUNT)) == 0 {
                continue;
            }
            if !first {
                msg_putchar(',' as c_int); // separate with commas
            }
            msg_puts(SPO_NAME_TAB[i as usize].as_ptr());
            let n = (*spp).sp_offsets[i as usize];
            if i != SPO_LC_OFF {
                if (*spp).sp_off_flags as c_int & mask != 0 {
                    msg_putchar('s' as c_int);
                } else {
                    msg_putchar('e' as c_int);
                }
                if n > 0 {
                    msg_putchar('+' as c_int);
                }
            }
            if n != 0 || i == SPO_LC_OFF {
                msg_outnum(n);
            }
            first = false;
        }
    }
}

/// What the previous keyword of a listing printed, so that a run of keywords
/// sharing their options prints them once.
#[derive(PartialEq)]
struct KeywordOpts {
    contained: c_int,
    skipnl: c_int,
    skipwhite: c_int,
    skipempty: c_int,
    cont_in_list: *const int16_t,
    next_list: *const int16_t,
}

impl KeywordOpts {
    /// Nothing printed yet, which is also what a new header resets to.
    const fn none() -> Self {
        KeywordOpts {
            contained: 0,
            skipnl: 0,
            skipwhite: 0,
            skipempty: 0,
            cont_in_list: ::core::ptr::null(),
            next_list: ::core::ptr::null(),
        }
    }

    /// What one keyword needs printed before it.
    unsafe fn of(kp: *const keyentry_T) -> Self {
        unsafe {
            KeywordOpts {
                contained: (*kp).flags & HL_CONTAINED,
                skipnl: (*kp).flags & HL_SKIPNL,
                skipwhite: (*kp).flags & HL_SKIPWHITE,
                skipempty: (*kp).flags & HL_SKIPEMPTY,
                cont_in_list: (*kp).k_syn.cont_in_list,
                next_list: (*kp).next_list,
            }
        }
    }
}

/// List the keywords of group `id` in `ht`, answering whether a header has now
/// been printed.
///
/// The keywords come out in hash order, not alphabetically, which is why the
/// options are re-printed whenever two neighbours disagree.
unsafe fn syn_list_keywords(id: c_int, ht: *const hashtab_T, mut did_header: bool) -> bool {
    unsafe {
        let mut prev = KeywordOpts::none();

        let mut todo = (*ht).ht_used;
        let mut hi = (*ht).ht_array;
        while todo > 0 && !got_int.get() {
            if !(*hi).is_kept() {
                hi = hi.offset(1);
                continue;
            }
            todo -= 1;
            let mut kp = key_to_entry((*hi).hi_key);
            while !kp.is_null() && !got_int.get() {
                if (*kp).k_syn.id as c_int == id {
                    did_header = put_keyword(kp, id, did_header, &mut prev);
                }
                kp = (*kp).ke_next;
            }
            hi = hi.offset(1);
        }
        did_header
    }
}

/// Print one keyword, preceded by whatever of its options the previous one did
/// not already print.
///
/// A keyword whose options differ from its neighbour's forces a new line,
/// which makes `syn_list_header` answer true and resets `prev` — that reset is
/// what keeps a NULL `containedin=`/`nextgroup=` from ever reaching
/// [`put_id_list`], which would walk it.
unsafe fn put_keyword(
    kp: *mut keyentry_T,
    id: c_int,
    did_header: bool,
    prev: &mut KeywordOpts,
) -> bool {
    unsafe {
        let opts = KeywordOpts::of(kp);
        let force_newline = opts != *prev;
        let outlen = if force_newline {
            0
        } else {
            strlen(entry_to_key(kp)) as c_int
        };
        if syn_list_header(did_header, outlen, id, force_newline) {
            *prev = KeywordOpts::none();
        }

        if prev.contained != opts.contained {
            msg_puts_hl(c"contained".as_ptr(), LIST_HL, false);
            msg_putchar(' ' as c_int);
            prev.contained = opts.contained;
        }
        if prev.cont_in_list != opts.cont_in_list {
            put_id_list(c"containedin", opts.cont_in_list, LIST_HL);
            msg_putchar(' ' as c_int);
            prev.cont_in_list = opts.cont_in_list;
        }
        if prev.next_list != opts.next_list {
            put_id_list(c"nextgroup", opts.next_list, LIST_HL);
            msg_putchar(' ' as c_int);
            prev.next_list = opts.next_list;
            // The three skip flags are only meaningful with a `nextgroup=`,
            // and upstream only ever prints them here.
            if opts.skipnl != 0 {
                msg_puts_hl(c"skipnl".as_ptr(), LIST_HL, false);
                msg_putchar(' ' as c_int);
                prev.skipnl = opts.skipnl;
            }
            if opts.skipwhite != 0 {
                msg_puts_hl(c"skipwhite".as_ptr(), LIST_HL, false);
                msg_putchar(' ' as c_int);
                prev.skipwhite = opts.skipwhite;
            }
            if opts.skipempty != 0 {
                msg_puts_hl(c"skipempty".as_ptr(), LIST_HL, false);
                msg_putchar(' ' as c_int);
                prev.skipempty = opts.skipempty;
            }
        }
        msg_outtrans(entry_to_key(kp), 0, false);
        true
    }
}
