//! `:syntax list` — the report.
//!
//! [`syn_cmd_list`] with no argument lists every item, with one lists the items
//! of a group. [`syn_list_one`] renders one group's keyword, match and region
//! items, [`put_pattern`] one pattern with its offsets, [`put_id_list`] a
//! `contains=`/`nextgroup=` list, and [`syn_list_cluster`] a cluster.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_int};

use super::*;

/// Everything is listed in the "directory" highlight, as `:highlight` does.
const LIST_HL: c_int = HLF_D;

/// `:syntax [list] [{group}|@{cluster}] ..` and `:syntax sync` with no
/// argument.
pub(crate) fn syn_cmd_list(eap: &mut exarg_T, syncing: c_int) {
    let mut arg = eap.arg;

    eap.nextcmd = unsafe { find_nextcmd(arg) };
    if eap.skip != 0 {
        return;
    }

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    if !unsafe { syntax_present(curwin.get()) } {
        msg(gettext(MSG_NO_ITEMS), 0);
        return;
    }

    if syncing != 0 {
        unsafe { list_sync_items() };
        return;
    }

    unsafe { msg_puts_title(gettext(c"\n--- Syntax items ---").as_ptr()) };
    if ends_excmd(unsafe { *arg } as c_int) != 0 {
        // No argument: list every group id, then every cluster.
        let mut id = 1;
        while id <= highlight_num_groups() && !got_int.get() {
            unsafe { syn_list_one(id, false, false) };
            id += 1;
        }
        let mut id = 0;
        while id < cur_cluster_count() && !got_int.get() {
            unsafe { syn_list_cluster(id) };
            id += 1;
        }
    } else {
        // List the groups and clusters the argument names.
        while ends_excmd(unsafe { *arg } as c_int) == 0 && !got_int.get() {
            let arg_end = unsafe { skiptowhite(arg) };
            if unsafe { *arg } as c_int == '@' as c_int {
                let id =
                    unsafe { syn_scl_namen2id(arg.add(1), arg_end.offset_from(arg) as c_int - 1) };
                if id == 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E392: No such syntax cluster: {arg}");
                } else {
                    unsafe { syn_list_cluster(id - SYNID_CLUSTER) };
                }
            } else {
                let id = unsafe { syn_name2id_len(arg, arg_end.offset_from(arg) as size_t) };
                if id == 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E28: No such highlight group name: {arg}");
                } else {
                    unsafe { syn_list_one(id, false, true) };
                }
            }
            arg = unsafe { skipwhite(arg_end) };
        }
    }
    eap.nextcmd = unsafe { check_nextcmd(arg) };
}

/// The `:syntax sync` half of the listing: how this buffer synchronises.
unsafe fn list_sync_items() {
    let mut block = cur_syn_block();
    if block.b_syn_sync_flags & SF_CCOMMENT != 0 {
        unsafe { msg_puts(gettext(c"syncing on C-style comments").as_ptr()) };
        unsafe { syn_lines_msg() };
        unsafe { syn_match_msg() };
    } else if block.b_syn_sync_flags & SF_MATCH != 0 {
        unsafe { msg_puts_title(gettext(c"\n--- Syntax sync items ---").as_ptr()) };
        if block.b_syn_sync_minlines > 0
            || block.b_syn_sync_maxlines > 0
            || block.b_syn_sync_linebreaks > 0
        {
            unsafe { msg_puts(gettext(c"\nsyncing on items").as_ptr()) };
            unsafe { syn_lines_msg() };
            unsafe { syn_match_msg() };
        }
        let mut id = 1;
        while id <= highlight_num_groups() && !got_int.get() {
            unsafe { syn_list_one(id, true, false) };
            id += 1;
        }
    } else if block.b_syn_sync_minlines == 0 {
        unsafe { msg_puts(gettext(c"no syncing").as_ptr()) };
    } else {
        if block.b_syn_sync_minlines == MAXLNUM as linenr_T {
            unsafe { msg_puts(gettext(c"syncing starts at the first line").as_ptr()) };
        } else {
            unsafe { msg_puts(gettext(c"syncing starts ").as_ptr()) };
            unsafe { msg_outnum(block.b_syn_sync_minlines) };
            unsafe { msg_puts(gettext(c" lines before top line").as_ptr()) };
        }
        unsafe { syn_match_msg() };
    }
}

/// "; minimal 5, maximal 10 lines before top line".
unsafe fn syn_lines_msg() {
    let mut block = cur_syn_block();
    if block.b_syn_sync_maxlines <= 0 && block.b_syn_sync_minlines <= 0 {
        return;
    }
    unsafe { msg_puts(c"; ".as_ptr()) };
    if block.b_syn_sync_minlines == MAXLNUM as linenr_T {
        unsafe { msg_puts(gettext(c"from the first line").as_ptr()) };
        return;
    }
    if block.b_syn_sync_minlines > 0 {
        unsafe { msg_puts(gettext(c"minimal ").as_ptr()) };
        unsafe { msg_outnum(block.b_syn_sync_minlines) };
        if block.b_syn_sync_maxlines != 0 {
            unsafe { msg_puts(c", ".as_ptr()) };
        }
    }
    if block.b_syn_sync_maxlines > 0 {
        unsafe { msg_puts(gettext(c"maximal ").as_ptr()) };
        unsafe { msg_outnum(block.b_syn_sync_maxlines) };
    }
    unsafe { msg_puts(gettext(c" lines before top line").as_ptr()) };
}

/// "; match 3 line breaks".
unsafe fn syn_match_msg() {
    let linebreaks = cur_syn_block().b_syn_sync_linebreaks;
    if linebreaks > 0 {
        unsafe { msg_puts(gettext(c"; match ").as_ptr()) };
        unsafe { msg_outnum(linebreaks) };
        unsafe { msg_puts(gettext(c" line breaks").as_ptr()) };
    }
}

/// The item flags that are printed after the patterns.
const ITEM_FLAG_NAMES: [(SynFlags, &CStr); 10] = [
    (SynFlags::DISPLAY, c"display"),
    (SynFlags::CONTAINED, c"contained"),
    (SynFlags::ONELINE, c"oneline"),
    (SynFlags::KEEPEND, c"keepend"),
    (SynFlags::EXTEND, c"extend"),
    (SynFlags::EXCLUDENL, c"excludenl"),
    (SynFlags::TRANSP, c"transparent"),
    (SynFlags::FOLD, c"fold"),
    (SynFlags::CONCEAL, c"conceal"),
    (SynFlags::CONCEALENDS, c"concealends"),
];

/// The flags that are printed after a `nextgroup=` list.
const NEXTGROUP_FLAG_NAMES: [(SynFlags, &CStr); 3] = [
    (SynFlags::SKIPWHITE, c"skipwhite"),
    (SynFlags::SKIPNL, c"skipnl"),
    (SynFlags::SKIPEMPTY, c"skipempty"),
];

/// Print the names of every flag of `names` that `flags` has.
unsafe fn syn_list_flags(names: &[(SynFlags, &CStr)], flags: SynFlags, hl_id: c_int) {
    for (flag, name) in names {
        if flags.has(*flag) {
            unsafe { msg_puts_hl(name.as_ptr(), hl_id, false) };
            unsafe { msg_putchar(' ' as c_int) };
        }
    }
}

/// List one syntax group: its keywords, its items, and the group it links to.
///
/// `syncing` lists the `:syntax sync` items instead of the ordinary ones;
/// `link_only` prints a group that has nothing but a link.
unsafe fn syn_list_one(id: c_int, syncing: bool, link_only: bool) {
    let mut did_header = false;

    // The keywords of `id`, from both tables.
    if !syncing {
        did_header =
            unsafe { syn_list_keywords(id, syn_field!(cur_syn_block(), b_keywtab), false) };
        did_header =
            unsafe { syn_list_keywords(id, syn_field!(cur_syn_block(), b_keywtab_ic), did_header) };
    }

    // The patterns of `id`. The item's options are read off its *first*
    // entry, which for a region is a START and so the one carrying the id
    // lists; `put_item_patterns` leaves `idx` on its last.
    let block = cur_syn_block();
    let mut idx = 0;
    while idx < block.patterns().len() && !got_int.get() {
        let first = idx;
        let spp = &block.patterns()[first];
        if spp.sp_syn.id as c_int != id || spp.sp_syncing != syncing {
            idx += 1;
            continue;
        }
        unsafe { syn_list_header(did_header, 0, id, true) };
        did_header = true;
        idx = unsafe { put_item_patterns(first) };

        let spp = &block.patterns()[first];
        unsafe { syn_list_flags(&ITEM_FLAG_NAMES, spp.sp_flags, LIST_HL) };
        if !spp.sp_cont_list.is_none() {
            unsafe { put_id_list(c"contains", spp.sp_cont_list.as_ptr(), LIST_HL) };
        }
        if !spp.sp_cont_in_list.is_none() {
            unsafe { put_id_list(c"containedin", spp.sp_cont_in_list.as_ptr(), LIST_HL) };
        }
        if !spp.sp_next_list.is_none() {
            unsafe { put_id_list(c"nextgroup", spp.sp_next_list.as_ptr(), LIST_HL) };
            unsafe { syn_list_flags(&NEXTGROUP_FLAG_NAMES, spp.sp_flags, LIST_HL) };
        }
        if spp.sp_flags.has(SynFlags::SYNC_HERE | SynFlags::SYNC_THERE) {
            unsafe { put_sync_group(spp.sp_flags, spp.sp_sync_idx) };
        }
        idx += 1;
    }

    // The link, if there is one.
    let link = highlight_link_id(id - 1);
    if link != 0 && (did_header || link_only) && !got_int.get() {
        unsafe { syn_list_header(did_header, 0, id, true) };
        unsafe { msg_puts_hl(c"links to".as_ptr(), LIST_HL, false) };
        unsafe { msg_putchar(' ' as c_int) };
        unsafe { msg_outtrans(highlight_group_name(link - 1), 0, false) };
    }
}

/// Print the pattern(s) of the item at `idx`, answering the index of its last
/// pattern.
///
/// A match is one pattern; a region is a run of consecutive entries —
/// start(s), an optional skip, then end(s) — which are printed together.
unsafe fn put_item_patterns(mut idx: usize) -> usize {
    let mut last_matchgroup = 0;
    let block = cur_syn_block();
    let pats = block.patterns();
    let sp_type = |i: usize| pats[i].sp_type as c_int;

    if sp_type(idx) == SPTYPE_MATCH {
        unsafe { put_pattern(&mut last_matchgroup, c"match", ' ' as c_int, &pats[idx]) };
        unsafe { msg_putchar(' ' as c_int) };
    } else if sp_type(idx) == SPTYPE_START {
        // The three loops bound themselves on the array; upstream bounds
        // only the last of them and reads past it if a region's END
        // entries are ever missing.
        while idx < pats.len() && sp_type(idx) == SPTYPE_START {
            unsafe { put_pattern(&mut last_matchgroup, c"start", '=' as c_int, &pats[idx]) };
            idx += 1;
        }
        if idx < pats.len() && sp_type(idx) == SPTYPE_SKIP {
            unsafe { put_pattern(&mut last_matchgroup, c"skip", '=' as c_int, &pats[idx]) };
            idx += 1;
        }
        while idx < pats.len() && sp_type(idx) == SPTYPE_END {
            unsafe { put_pattern(&mut last_matchgroup, c"end", '=' as c_int, &pats[idx]) };
            idx += 1;
        }
        idx -= 1;
        unsafe { msg_putchar(' ' as c_int) };
    }
    idx
}

/// Print `grouphere`/`groupthere` and the region item it names.
unsafe fn put_sync_group(flags: SynFlags, sync_idx: c_int) {
    if flags.has(SynFlags::SYNC_HERE) {
        unsafe { msg_puts_hl(c"grouphere".as_ptr(), LIST_HL, false) };
    } else {
        unsafe { msg_puts_hl(c"groupthere".as_ptr(), LIST_HL, false) };
    }
    unsafe { msg_putchar(' ' as c_int) };
    if sync_idx >= 0 {
        let block = cur_syn_block();
        let target_id = block.patterns()[sync_idx as usize].sp_syn.id as c_int;
        unsafe { msg_outtrans(highlight_group_name(target_id - 1), 0, false) };
    } else {
        unsafe { msg_puts(c"NONE".as_ptr()) };
    }
    unsafe { msg_putchar(' ' as c_int) };
}

/// List one cluster and its members.
unsafe fn syn_list_cluster(id: c_int) {
    // Slight hack: roughly duplicate the guts of `syn_list_header`.
    let mut endcol = 15;
    let block = cur_syn_block();
    let cluster = &block.clusters()[id as usize];
    unsafe { msg_putchar('\n' as c_int) };
    unsafe { msg_outtrans(cluster.scl_name.as_ptr(), 0, false) };

    if msg_col.get() >= endcol {
        endcol = msg_col.get() + 1; // output at least one space
    }
    if Columns.get() <= endcol {
        endcol = Columns.get() - 1; // avoid a hang for a tiny window
    }
    unsafe { msg_advance(endcol) };

    if cluster.scl_list.is_none() {
        unsafe { msg_puts_hl(c"cluster".as_ptr(), LIST_HL, false) };
        unsafe { msg_puts(c"=NONE".as_ptr()) };
    } else {
        unsafe { put_id_list(c"cluster", cluster.scl_list.as_ptr(), LIST_HL) };
    }
}

/// Print `name=a,b,@cl` for a `contains=`/`containedin=`/`nextgroup=` list.
unsafe fn put_id_list(name: &CStr, list: *const int16_t, hl_id: c_int) {
    unsafe { msg_puts_hl(name.as_ptr(), hl_id, false) };
    unsafe { msg_putchar('=' as c_int) };
    let mut p = list;
    while unsafe { *p } != 0 {
        let item = unsafe { *p } as c_int;
        let more = unsafe { *p.add(1) } != 0;
        if (SYNID_ALLBUT..SYNID_TOP).contains(&item) {
            // ALLBUT is the same marker as ALL, told apart by whether the
            // list goes on to name exceptions.
            unsafe { msg_puts(if more { c"ALLBUT" } else { c"ALL" }.as_ptr()) };
        } else if (SYNID_TOP..SYNID_CONTAINED).contains(&item) {
            unsafe { msg_puts(c"TOP".as_ptr()) };
        } else if (SYNID_CONTAINED..SYNID_CLUSTER).contains(&item) {
            unsafe { msg_puts(c"CONTAINED".as_ptr()) };
        } else if item >= SYNID_CLUSTER {
            let block = cur_syn_block();
            let name = block.clusters()[(item - SYNID_CLUSTER) as usize]
                .scl_name
                .as_ptr();
            unsafe { msg_putchar('@' as c_int) };
            unsafe { msg_outtrans(name, 0, false) };
        } else {
            unsafe { msg_outtrans(highlight_group_name(item - 1), 0, false) };
        }
        if more {
            unsafe { msg_putchar(',' as c_int) };
        }
        p = unsafe { p.add(1) };
    }
    unsafe { msg_putchar(' ' as c_int) };
}

/// The delimiters `put_pattern` will wrap a pattern in, best first.
const SEPCHARS: &[u8] = b"/+=-#@\"|'^&";

/// Print one pattern of an item: its keyword, the pattern between delimiters,
/// and its offsets.
///
/// `last_matchgroup` carries the `matchgroup=` in force across the patterns of
/// one item, so a change is printed once rather than per pattern.
unsafe fn put_pattern(last_matchgroup: &mut c_int, s: &CStr, c: c_int, spp: &synpat_T) {
    // May have to write "matchgroup=group".
    if *last_matchgroup != spp.sp_syn_match_id as c_int {
        *last_matchgroup = spp.sp_syn_match_id as c_int;
        unsafe { msg_puts_hl(c"matchgroup".as_ptr(), LIST_HL, false) };
        unsafe { msg_putchar('=' as c_int) };
        if *last_matchgroup == 0 {
            unsafe { msg_outtrans(c"NONE".as_ptr(), 0, false) };
        } else {
            unsafe { msg_outtrans(highlight_group_name(*last_matchgroup - 1), 0, false) };
        }
        unsafe { msg_putchar(' ' as c_int) };
    }

    // The name of the pattern and an '=' or ' '.
    unsafe { msg_puts_hl(s.as_ptr(), LIST_HL, false) };
    unsafe { msg_putchar(c) };

    // The pattern, wrapped in the first delimiter it does not itself
    // contain — or the first one of all, if it contains every one.
    let pattern = spp.sp_pattern.as_deref().unwrap_or(c"");
    let mut i = 0;
    while pattern.to_bytes().contains(&SEPCHARS[i]) {
        i += 1;
        if i == SEPCHARS.len() {
            i = 0;
            break;
        }
    }
    unsafe { msg_putchar(SEPCHARS[i] as c_int) };
    unsafe { msg_outtrans(pattern.as_ptr(), 0, false) };
    unsafe { msg_putchar(SEPCHARS[i] as c_int) };

    unsafe { put_pattern_offsets(spp) };
    unsafe { msg_putchar(' ' as c_int) };
}

/// Print the `ms=s+1,he=e-2,lc=3` offsets of one pattern.
unsafe fn put_pattern_offsets(spp: &synpat_T) {
    let mut first = true;
    for i in 0..SPO_COUNT {
        // A start offset and an end offset share one name; the flag word
        // holds the start half in its low `SPO_COUNT` bits.
        let mask = 1 << i;
        if spp.sp_off_flags as c_int & (mask + (mask << SPO_COUNT)) == 0 {
            continue;
        }
        if !first {
            unsafe { msg_putchar(',' as c_int) }; // separate with commas
        }
        unsafe { msg_puts(SPO_NAME_TAB[i as usize].as_ptr()) };
        let n = spp.sp_offsets[i as usize];
        if i != SPO_LC_OFF {
            if spp.sp_off_flags as c_int & mask != 0 {
                unsafe { msg_putchar('s' as c_int) };
            } else {
                unsafe { msg_putchar('e' as c_int) };
            }
            if n > 0 {
                unsafe { msg_putchar('+' as c_int) };
            }
        }
        if n != 0 || i == SPO_LC_OFF {
            unsafe { msg_outnum(n) };
        }
        first = false;
    }
}

/// What the previous keyword of a listing printed, so that a run of keywords
/// sharing their options prints them once.
#[derive(PartialEq)]
struct KeywordOpts {
    contained: SynFlags,
    skipnl: SynFlags,
    skipwhite: SynFlags,
    skipempty: SynFlags,
    cont_in_list: *const int16_t,
    next_list: *const int16_t,
}

impl KeywordOpts {
    /// Nothing printed yet, which is also what a new header resets to.
    const fn none() -> Self {
        KeywordOpts {
            contained: SynFlags::NONE,
            skipnl: SynFlags::NONE,
            skipwhite: SynFlags::NONE,
            skipempty: SynFlags::NONE,
            cont_in_list: ::core::ptr::null(),
            next_list: ::core::ptr::null(),
        }
    }

    /// What one keyword needs printed before it.
    unsafe fn of(kp: *const keyentry_T) -> Self {
        KeywordOpts {
            contained: unsafe { (*kp).flags }.masked(SynFlags::CONTAINED),
            skipnl: unsafe { (*kp).flags }.masked(SynFlags::SKIPNL),
            skipwhite: unsafe { (*kp).flags }.masked(SynFlags::SKIPWHITE),
            skipempty: unsafe { (*kp).flags }.masked(SynFlags::SKIPEMPTY),
            cont_in_list: unsafe { (*kp).cont_in_list },
            next_list: unsafe { (*kp).next_list },
        }
    }
}

/// List the keywords of group `id` in `ht`, answering whether a header has now
/// been printed.
///
/// The keywords come out in hash order, not alphabetically, which is why the
/// options are re-printed whenever two neighbours disagree.
unsafe fn syn_list_keywords(id: c_int, ht: *const hashtab_T, mut did_header: bool) -> bool {
    let mut prev = KeywordOpts::none();

    // SAFETY: the caller's table, which nothing here mutates.
    for hi in unsafe { &*ht }.items() {
        if got_int.get() {
            break;
        }
        let mut kp = unsafe { key_to_entry(hi.hi_key) };
        while !kp.is_null() && !got_int.get() {
            if unsafe { (*kp).k_syn.id } as c_int == id {
                did_header = unsafe { put_keyword(kp, id, did_header, &mut prev) };
            }
            kp = unsafe { (*kp).ke_next };
        }
    }
    did_header
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
    let opts = unsafe { KeywordOpts::of(kp) };
    let force_newline = opts != *prev;
    let outlen = if force_newline {
        0
    } else {
        unsafe { cstr::bytes_at(entry_to_key(kp)).len() as c_int }
    };
    if unsafe { syn_list_header(did_header, outlen, id, force_newline) } {
        *prev = KeywordOpts::none();
    }

    if prev.contained != opts.contained {
        unsafe { msg_puts_hl(c"contained".as_ptr(), LIST_HL, false) };
        unsafe { msg_putchar(' ' as c_int) };
        prev.contained = opts.contained;
    }
    if prev.cont_in_list != opts.cont_in_list {
        unsafe { put_id_list(c"containedin", opts.cont_in_list, LIST_HL) };
        unsafe { msg_putchar(' ' as c_int) };
        prev.cont_in_list = opts.cont_in_list;
    }
    if prev.next_list != opts.next_list {
        unsafe { put_id_list(c"nextgroup", opts.next_list, LIST_HL) };
        unsafe { msg_putchar(' ' as c_int) };
        prev.next_list = opts.next_list;
        // The three skip flags are only meaningful with a `nextgroup=`,
        // and upstream only ever prints them here.
        if opts.skipnl != SynFlags::NONE {
            unsafe { msg_puts_hl(c"skipnl".as_ptr(), LIST_HL, false) };
            unsafe { msg_putchar(' ' as c_int) };
            prev.skipnl = opts.skipnl;
        }
        if opts.skipwhite != SynFlags::NONE {
            unsafe { msg_puts_hl(c"skipwhite".as_ptr(), LIST_HL, false) };
            unsafe { msg_putchar(' ' as c_int) };
            prev.skipwhite = opts.skipwhite;
        }
        if opts.skipempty != SynFlags::NONE {
            unsafe { msg_puts_hl(c"skipempty".as_ptr(), LIST_HL, false) };
            unsafe { msg_putchar(' ' as c_int) };
            prev.skipempty = opts.skipempty;
        }
    }
    unsafe { msg_outtrans(entry_to_key(kp), 0, false) };
    true
}
