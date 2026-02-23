//! Listing/display subsystem for `:syntax list` and `:syntax` (no args).
//!
//! Migrated from syntax_accessors.c: syn_cmd_list, syn_list_one,
//! syn_list_keywords, put_pattern, put_id_list, syn_list_flags,
//! syn_list_cluster, syn_lines_msg, syn_match_msg.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    KeyEntryHandle, SynPatHandle, HL_CONCEAL, HL_CONCEALENDS, HL_CONTAINED, HL_DISPLAY,
    HL_EXCLUDENL, HL_EXTEND, HL_FOLD, HL_KEEPEND, HL_ONELINE, HL_SKIPEMPTY, HL_SKIPNL,
    HL_SKIPWHITE, HL_SYNC_HERE, HL_SYNC_THERE, HL_TRANSP, SF_CCOMMENT, SF_MATCH, SPO_COUNT,
    SYNID_ALLBUT, SYNID_CLUSTER, SYNID_CONTAINED, SYNID_TOP,
};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // EAP accessors
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_eap_check_nextcmd(eap: *mut c_void, arg: *mut c_char);

    // String helpers
    fn nvim_syn_skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_skiptowhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_ends_excmd(c: c_int) -> c_int;

    // Syntax presence and state
    fn nvim_syn_syntax_present_curwin() -> c_int;
    fn nvim_syn_get_got_int() -> c_int;

    // syn_list_header wrapper (from highlight_group.c)
    fn nvim_syn_list_header(did_header: c_int, outlen: c_int, id: c_int, force: c_int) -> c_int;

    // Highlight group accessors
    fn nvim_syn_highlight_num_groups() -> c_int;
    fn nvim_syn_highlight_group_name(idx: c_int) -> *mut c_char;
    fn nvim_syn_highlight_link_id(id: c_int) -> c_int;

    // Cluster accessors for curwin
    fn nvim_curwin_syncluster_count() -> c_int;
    fn nvim_curwin_syncluster_name(idx: c_int) -> *mut c_char;
    fn nvim_curwin_syncluster_list(idx: c_int) -> *mut i16;

    // Pattern accessors for curwin
    fn nvim_curwin_synpat_count() -> c_int;
    fn nvim_curwin_synpat_at(idx: c_int) -> SynPatHandle;

    // Synpat field accessors
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_syncing(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_flags(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_syn_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_syn_match_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_sync_idx(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_pattern(pat: SynPatHandle) -> *const c_char;
    fn nvim_synpat_get_off_flags(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_offset(pat: SynPatHandle, idx: c_int) -> c_int;
    fn nvim_synpat_get_cont_list(pat: SynPatHandle) -> *mut i16;
    fn nvim_synpat_get_next_list(pat: SynPatHandle) -> *mut i16;
    fn nvim_synpat_get_cont_in_list(pat: SynPatHandle) -> *mut i16;

    // Keyword hashtable iteration
    fn nvim_synblock_keywtab_ptr(block: crate::types::SynBlockHandle) -> *mut c_void;
    fn nvim_synblock_keywtab_ic_ptr(block: crate::types::SynBlockHandle) -> *mut c_void;
    fn nvim_ht_get_array_size(ht: *const c_void) -> usize;
    fn nvim_ht_get_used(ht: *const c_void) -> usize;
    fn nvim_ht_item_at(ht: *const c_void, idx: usize) -> KeyEntryHandle;

    // Keyentry field accessors
    fn nvim_keyentry_get_next(ke: KeyEntryHandle) -> KeyEntryHandle;
    fn nvim_keyentry_get_syn_id(ke: KeyEntryHandle) -> i16;
    fn nvim_keyentry_get_flags(ke: KeyEntryHandle) -> c_int;
    fn nvim_keyentry_get_keyword(ke: KeyEntryHandle) -> *const c_char;
    fn nvim_keyentry_get_next_list(ke: KeyEntryHandle) -> *mut i16;
    fn nvim_keyentry_get_cont_in_list(ke: KeyEntryHandle) -> *mut i16;

    // Sync field accessors for curwin
    fn nvim_curwin_syn_sync_flags() -> c_int;
    fn nvim_curwin_syn_sync_minlines() -> c_int;
    fn nvim_curwin_syn_sync_maxlines() -> c_int;
    fn nvim_curwin_syn_sync_linebreaks() -> c_int;

    // Group name/ID lookup
    fn nvim_syn_scl_namen2id(arg: *const c_char, len: c_int) -> c_int;
    fn nvim_syn_name2id_len(name: *const c_char, len: c_int) -> c_int;

    // vim_strchr wrapper
    fn nvim_syn_vim_strchr(s: *const c_char, c: c_int) -> c_int;

    // Message output
    fn msg_puts(s: *const c_char);
    fn msg_puts_title(s: *const c_char);
    fn msg_putchar(c: c_int);
    fn msg_outnum(n: c_int);
    fn msg_advance(col: c_int);
    fn msg_outtrans(s: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn nvim_msg_puts_hl_syn(s: *const c_char, hl_id: c_int, hist: bool);
    fn nvim_get_msg_col_syn() -> c_int;

    // Error / info message
    fn semsg(fmt: *const c_char, ...);
    fn msg(s: *const c_char, hl_id: c_int) -> c_int;

    // curwin synblock
    fn nvim_get_curwin_synblock() -> crate::types::SynBlockHandle;
}

// =============================================================================
// Constants
// =============================================================================

// HLF_D = 5 in hlf_T enum (0=NONE, 1=HLF_8, 2=EOB, 3=TERM, 4=AT, 5=D)
const HLF_D: c_int = 5;

// MAXLNUM for sync from-first-line detection
const MAXLNUM: c_int = 0x7FFF_FFFF;

// spo_name_tab: offset name prefixes (3 chars + NUL)
static SPO_NAME_TAB: [&[u8]; 7] = [
    b"ms=\0", b"me=\0", b"hs=\0", b"he=\0", b"rs=\0", b"re=\0", b"lc=\0",
];

// SPO_LC_OFF index (last entry in spo_name_tab)
const SPO_LC_OFF: usize = 6;

// Separator characters for pattern output
static SEPCHARS: &[u8] = b"/+=-#@\"|'^&";

// =============================================================================
// Flag name tables (flag_value, static name)
// =============================================================================

struct FlagEntry {
    flag: c_int,
    name: &'static [u8],
}

static NAMELIST1: &[FlagEntry] = &[
    FlagEntry {
        flag: HL_DISPLAY,
        name: b"display\0",
    },
    FlagEntry {
        flag: HL_CONTAINED,
        name: b"contained\0",
    },
    FlagEntry {
        flag: HL_ONELINE,
        name: b"oneline\0",
    },
    FlagEntry {
        flag: HL_KEEPEND,
        name: b"keepend\0",
    },
    FlagEntry {
        flag: HL_EXTEND,
        name: b"extend\0",
    },
    FlagEntry {
        flag: HL_EXCLUDENL,
        name: b"excludenl\0",
    },
    FlagEntry {
        flag: HL_TRANSP,
        name: b"transparent\0",
    },
    FlagEntry {
        flag: HL_FOLD,
        name: b"fold\0",
    },
    FlagEntry {
        flag: HL_CONCEAL,
        name: b"conceal\0",
    },
    FlagEntry {
        flag: HL_CONCEALENDS,
        name: b"concealends\0",
    },
];

static NAMELIST2: &[FlagEntry] = &[
    FlagEntry {
        flag: HL_SKIPWHITE,
        name: b"skipwhite\0",
    },
    FlagEntry {
        flag: HL_SKIPNL,
        name: b"skipnl\0",
    },
    FlagEntry {
        flag: HL_SKIPEMPTY,
        name: b"skipempty\0",
    },
];

// =============================================================================
// Internal helpers
// =============================================================================

/// Output sync line count info (if any).
unsafe fn syn_lines_msg() {
    let maxlines = nvim_curwin_syn_sync_maxlines();
    let minlines = nvim_curwin_syn_sync_minlines();
    if maxlines > 0 || minlines > 0 {
        msg_puts(c"; ".as_ptr());
        if minlines == MAXLNUM {
            msg_puts(c"from the first line".as_ptr());
        } else {
            if minlines > 0 {
                msg_puts(c"minimal ".as_ptr());
                msg_outnum(minlines);
                if maxlines > 0 {
                    msg_puts(c", ".as_ptr());
                }
            }
            if maxlines > 0 {
                msg_puts(c"maximal ".as_ptr());
                msg_outnum(maxlines);
            }
            msg_puts(c" lines before top line".as_ptr());
        }
    }
}

/// Output sync line-break match info (if any).
unsafe fn syn_match_msg() {
    let linebreaks = nvim_curwin_syn_sync_linebreaks();
    if linebreaks > 0 {
        msg_puts(c"; match ".as_ptr());
        msg_outnum(linebreaks);
        msg_puts(c" line breaks".as_ptr());
    }
}

/// Output flag names from a flag table.
unsafe fn syn_list_flags(nlist: &[FlagEntry], flags: c_int, hl_id: c_int) {
    for entry in nlist {
        if flags & entry.flag != 0 {
            nvim_msg_puts_hl_syn(entry.name.as_ptr().cast(), hl_id, false);
            msg_putchar(b' ' as c_int);
        }
    }
}

/// Output an id list (contains=, containedin=, nextgroup=, cluster=).
unsafe fn put_id_list(name: &[u8], list: *const i16, hl_id: c_int) {
    nvim_msg_puts_hl_syn(name.as_ptr().cast(), hl_id, false);
    msg_putchar(b'=' as c_int);
    let mut p = list;
    loop {
        let id = i32::from(*p);
        if id == 0 {
            break;
        }
        if id >= SYNID_ALLBUT && id < SYNID_TOP {
            if *p.add(1) != 0 {
                msg_puts(c"ALLBUT".as_ptr());
            } else {
                msg_puts(c"ALL".as_ptr());
            }
        } else if id >= SYNID_TOP && id < SYNID_CONTAINED {
            msg_puts(c"TOP".as_ptr());
        } else if id >= SYNID_CONTAINED && id < SYNID_CLUSTER {
            msg_puts(c"CONTAINED".as_ptr());
        } else if id >= SYNID_CLUSTER {
            let scl_id = id - SYNID_CLUSTER;
            msg_putchar(b'@' as c_int);
            let name_ptr = nvim_curwin_syncluster_name(scl_id);
            if !name_ptr.is_null() {
                msg_outtrans(name_ptr, 0, false);
            }
        } else {
            let group_name = nvim_syn_highlight_group_name(id - 1);
            if !group_name.is_null() {
                msg_outtrans(group_name, 0, false);
            }
        }
        p = p.add(1);
        if *p != 0 {
            msg_putchar(b',' as c_int);
        }
    }
    msg_putchar(b' ' as c_int);
}

/// Output a pattern definition (match/start/skip/end).
unsafe fn put_pattern(
    s: &[u8],
    c: u8,
    spp: SynPatHandle,
    hl_id: c_int,
    last_matchgroup: &mut c_int,
) {
    // May need to output "matchgroup=group"
    let syn_match_id = i32::from(nvim_synpat_get_syn_match_id(spp));
    if *last_matchgroup != syn_match_id {
        *last_matchgroup = syn_match_id;
        nvim_msg_puts_hl_syn(c"matchgroup".as_ptr(), hl_id, false);
        msg_putchar(b'=' as c_int);
        if *last_matchgroup == 0 {
            msg_outtrans(c"NONE".as_ptr(), 0, false);
        } else {
            let group_name = nvim_syn_highlight_group_name(*last_matchgroup - 1);
            if !group_name.is_null() {
                msg_outtrans(group_name, 0, false);
            }
        }
        msg_putchar(b' ' as c_int);
    }

    // Output the name of the pattern and separator char
    nvim_msg_puts_hl_syn(s.as_ptr().cast(), hl_id, false);
    msg_putchar(c as c_int);

    // Find a separator char not in the pattern
    let pattern = nvim_synpat_get_pattern(spp);
    let mut sep_idx = 0usize;
    if !pattern.is_null() {
        loop {
            let sep_char = SEPCHARS[sep_idx] as c_int;
            if nvim_syn_vim_strchr(pattern, sep_char) == 0 {
                break;
            }
            sep_idx += 1;
            if sep_idx >= SEPCHARS.len() {
                sep_idx = 0;
                break;
            }
        }
    }
    let sep = SEPCHARS[sep_idx];
    msg_putchar(sep as c_int);
    if !pattern.is_null() {
        msg_outtrans(pattern, 0, false);
    }
    msg_putchar(sep as c_int);

    // Output any pattern offset options
    let off_flags = i32::from(nvim_synpat_get_off_flags(spp));
    let mut first = true;
    for (i, spo_name) in SPO_NAME_TAB.iter().enumerate() {
        let mask = 1i32 << i;
        let mask2 = mask << (SPO_COUNT as usize);
        if off_flags & (mask | mask2) == 0 {
            continue;
        }
        if !first {
            msg_putchar(b',' as c_int);
        }
        msg_puts(spo_name.as_ptr().cast());
        let n = nvim_synpat_get_offset(spp, i as c_int);
        if i != SPO_LC_OFF {
            if off_flags & mask != 0 {
                msg_putchar(b's' as c_int);
            } else {
                msg_putchar(b'e' as c_int);
            }
            if n > 0 {
                msg_putchar(b'+' as c_int);
            }
        }
        if n != 0 || i == SPO_LC_OFF {
            // lc= always shows the number
            msg_outnum(n);
        }
        first = false;
    }
    msg_putchar(b' ' as c_int);
}

/// List keywords for one syntax group.
/// Returns true if the header was printed.
unsafe fn syn_list_keywords(
    id: c_int,
    ht: *const c_void,
    mut did_header: bool,
    hl_id: c_int,
) -> bool {
    let mut prev_contained: c_int = 0;
    let mut prev_next_list: *const i16 = std::ptr::null();
    let mut prev_cont_in_list: *const i16 = std::ptr::null();
    let mut prev_skipnl: c_int = 0;
    let mut prev_skipwhite: c_int = 0;
    let mut prev_skipempty: c_int = 0;

    let array_size = nvim_ht_get_array_size(ht);
    let mut todo = nvim_ht_get_used(ht);

    let mut i = 0usize;
    while i < array_size && todo > 0 && nvim_syn_get_got_int() == 0 {
        let kp_start = nvim_ht_item_at(ht, i);
        i += 1;
        if kp_start.is_null() {
            continue;
        }
        todo -= 1;
        let mut kp = kp_start;
        while !kp.is_null() && nvim_syn_get_got_int() == 0 {
            let kp_next = nvim_keyentry_get_next(kp);
            if i32::from(nvim_keyentry_get_syn_id(kp)) == id {
                let kp_flags = nvim_keyentry_get_flags(kp);
                let kp_next_list = nvim_keyentry_get_next_list(kp);
                let kp_cont_in_list = nvim_keyentry_get_cont_in_list(kp);

                let kp_contained = kp_flags & HL_CONTAINED;
                let kp_skipnl = kp_flags & HL_SKIPNL;
                let kp_skipwhite = kp_flags & HL_SKIPWHITE;
                let kp_skipempty = kp_flags & HL_SKIPEMPTY;

                let force_newline = prev_contained != kp_contained
                    || prev_skipnl != kp_skipnl
                    || prev_skipwhite != kp_skipwhite
                    || prev_skipempty != kp_skipempty
                    || !std::ptr::eq(prev_cont_in_list, kp_cont_in_list as *const i16)
                    || !std::ptr::eq(prev_next_list, kp_next_list as *const i16);

                let outlen = if force_newline {
                    0
                } else {
                    let keyword = nvim_keyentry_get_keyword(kp);
                    if keyword.is_null() {
                        0
                    } else {
                        libc_strlen(keyword) as c_int
                    }
                };

                // syn_list_header returns non-zero if it reset state (newline printed)
                let reset =
                    nvim_syn_list_header(did_header as c_int, outlen, id, force_newline as c_int);
                if reset != 0 {
                    prev_contained = 0;
                    prev_next_list = std::ptr::null();
                    prev_cont_in_list = std::ptr::null();
                    prev_skipnl = 0;
                    prev_skipwhite = 0;
                    prev_skipempty = 0;
                }
                did_header = true;

                if prev_contained != kp_contained {
                    nvim_msg_puts_hl_syn(c"contained".as_ptr(), hl_id, false);
                    msg_putchar(b' ' as c_int);
                    prev_contained = kp_contained;
                }
                if !std::ptr::eq(prev_cont_in_list, kp_cont_in_list as *const i16) {
                    if !kp_cont_in_list.is_null() {
                        put_id_list(b"containedin\0", kp_cont_in_list, hl_id);
                        msg_putchar(b' ' as c_int);
                    }
                    prev_cont_in_list = kp_cont_in_list as *const i16;
                }
                if !std::ptr::eq(prev_next_list, kp_next_list as *const i16) {
                    if !kp_next_list.is_null() {
                        put_id_list(b"nextgroup\0", kp_next_list, hl_id);
                        msg_putchar(b' ' as c_int);
                        if kp_skipnl != 0 {
                            nvim_msg_puts_hl_syn(c"skipnl".as_ptr(), hl_id, false);
                            msg_putchar(b' ' as c_int);
                            prev_skipnl = kp_skipnl;
                        }
                        if kp_skipwhite != 0 {
                            nvim_msg_puts_hl_syn(c"skipwhite".as_ptr(), hl_id, false);
                            msg_putchar(b' ' as c_int);
                            prev_skipwhite = kp_skipwhite;
                        }
                        if kp_skipempty != 0 {
                            nvim_msg_puts_hl_syn(c"skipempty".as_ptr(), hl_id, false);
                            msg_putchar(b' ' as c_int);
                            prev_skipempty = kp_skipempty;
                        }
                    }
                    prev_next_list = kp_next_list as *const i16;
                }
                let keyword = nvim_keyentry_get_keyword(kp);
                if !keyword.is_null() {
                    msg_outtrans(keyword, 0, false);
                }
            }
            kp = kp_next;
        }
    }

    did_header
}

/// List one syntax group (patterns + keywords + link).
unsafe fn syn_list_one(id: c_int, syncing: bool, link_only: bool) {
    let hl_id = HLF_D;

    let mut did_header = false;
    // last_matchgroup tracks the current matchgroup within a region's start/skip/end sequence.
    // Initialized to 0; reset to 0 at the start of each top-level pattern item.
    #[allow(unused_assignments)]
    let mut last_matchgroup: c_int = 0;

    // List keywords (not for syncing items)
    if !syncing {
        let curwin_block = nvim_get_curwin_synblock();
        let ht = nvim_synblock_keywtab_ptr(curwin_block);
        did_header = syn_list_keywords(id, ht, did_header, hl_id);
        let ht_ic = nvim_synblock_keywtab_ic_ptr(curwin_block);
        did_header = syn_list_keywords(id, ht_ic, did_header, hl_id);
    }

    // List patterns for this id
    let pat_count = nvim_curwin_synpat_count();
    let mut idx = 0;
    while idx < pat_count && nvim_syn_get_got_int() == 0 {
        let spp = nvim_curwin_synpat_at(idx);
        if spp.is_null() {
            idx += 1;
            continue;
        }
        let sp_syn_id = i32::from(nvim_synpat_get_syn_id(spp));
        let sp_syncing = nvim_synpat_get_syncing(spp) != 0;
        if sp_syn_id != id || sp_syncing != syncing {
            idx += 1;
            continue;
        }

        nvim_syn_list_header(did_header as c_int, 0, id, 1);
        did_header = true;
        last_matchgroup = 0;

        let sp_type = nvim_synpat_get_type(spp);
        if sp_type == crate::types::SPTYPE_MATCH {
            put_pattern(b"match\0", b' ', spp, hl_id, &mut last_matchgroup);
            msg_putchar(b' ' as c_int);
        } else if sp_type == crate::types::SPTYPE_START {
            // Collect start/skip/end patterns
            while idx < pat_count {
                let cur = nvim_curwin_synpat_at(idx);
                if cur.is_null() || nvim_synpat_get_type(cur) != crate::types::SPTYPE_START {
                    break;
                }
                put_pattern(b"start\0", b'=', cur, hl_id, &mut last_matchgroup);
                idx += 1;
            }
            if idx < pat_count {
                let cur = nvim_curwin_synpat_at(idx);
                if !cur.is_null() && nvim_synpat_get_type(cur) == crate::types::SPTYPE_SKIP {
                    put_pattern(b"skip\0", b'=', cur, hl_id, &mut last_matchgroup);
                    idx += 1;
                }
            }
            while idx < pat_count {
                let cur = nvim_curwin_synpat_at(idx);
                if cur.is_null() || nvim_synpat_get_type(cur) != crate::types::SPTYPE_END {
                    break;
                }
                put_pattern(b"end\0", b'=', cur, hl_id, &mut last_matchgroup);
                idx += 1;
            }
            idx -= 1; // outer loop will increment
            msg_putchar(b' ' as c_int);
        }

        let sp_flags = nvim_synpat_get_flags(spp);
        syn_list_flags(NAMELIST1, sp_flags, hl_id);

        let cont_list = nvim_synpat_get_cont_list(spp);
        if !cont_list.is_null() {
            put_id_list(b"contains\0", cont_list, hl_id);
        }

        let cont_in_list = nvim_synpat_get_cont_in_list(spp);
        if !cont_in_list.is_null() {
            put_id_list(b"containedin\0", cont_in_list, hl_id);
        }

        let next_list = nvim_synpat_get_next_list(spp);
        if !next_list.is_null() {
            put_id_list(b"nextgroup\0", next_list, hl_id);
            syn_list_flags(NAMELIST2, sp_flags, hl_id);
        }

        if sp_flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
            if sp_flags & HL_SYNC_HERE != 0 {
                nvim_msg_puts_hl_syn(c"grouphere".as_ptr(), hl_id, false);
            } else {
                nvim_msg_puts_hl_syn(c"groupthere".as_ptr(), hl_id, false);
            }
            msg_putchar(b' ' as c_int);
            let sync_idx = nvim_synpat_get_sync_idx(spp);
            if sync_idx >= 0 {
                let sync_pat = nvim_curwin_synpat_at(sync_idx);
                if !sync_pat.is_null() {
                    let group_name = nvim_syn_highlight_group_name(
                        i32::from(nvim_synpat_get_syn_id(sync_pat)) - 1,
                    );
                    if !group_name.is_null() {
                        msg_outtrans(group_name, 0, false);
                    }
                }
            } else {
                msg_puts(c"NONE".as_ptr());
            }
            msg_putchar(b' ' as c_int);
        }

        idx += 1;
    }

    // List the link, if there is one
    let link_id = nvim_syn_highlight_link_id(id - 1);
    if link_id != 0 && (did_header || link_only) && nvim_syn_get_got_int() == 0 {
        nvim_syn_list_header(did_header as c_int, 0, id, 1);
        nvim_msg_puts_hl_syn(c"links to".as_ptr(), hl_id, false);
        msg_putchar(b' ' as c_int);
        let group_name = nvim_syn_highlight_group_name(link_id - 1);
        if !group_name.is_null() {
            msg_outtrans(group_name, 0, false);
        }
    }
}

/// List one syntax cluster.
unsafe fn syn_list_cluster(id: c_int) {
    let endcol = 15;

    // Slightly duplicate the guts of syn_list_header
    msg_putchar(b'\n' as c_int);
    let name = nvim_curwin_syncluster_name(id);
    if !name.is_null() {
        msg_outtrans(name, 0, false);
    }

    let col = nvim_get_msg_col_syn();
    let advance_col = if col >= endcol { col + 1 } else { endcol };
    msg_advance(advance_col);

    let list = nvim_curwin_syncluster_list(id);
    if !list.is_null() && *list != 0 {
        put_id_list(b"cluster\0", list, HLF_D);
    } else {
        nvim_msg_puts_hl_syn(c"cluster".as_ptr(), HLF_D, false);
        msg_puts(c"=NONE".as_ptr());
    }
}

/// Compute strlen of a C string (without libc).
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// Main implementation
// =============================================================================

/// Rust implementation of syn_cmd_list.
unsafe fn syn_cmd_list_impl(eap: *mut c_void, syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);

    if nvim_syn_get_eap_skip(eap) != 0 {
        // Even on skip, set nextcmd
        nvim_syn_eap_check_nextcmd(eap, arg);
        return;
    }

    if nvim_syn_syntax_present_curwin() == 0 {
        msg(c"No Syntax items defined for this buffer".as_ptr(), 0);
        return;
    }

    let sync_flags = nvim_curwin_syn_sync_flags();
    let minlines = nvim_curwin_syn_sync_minlines();

    if syncing != 0 {
        if sync_flags & SF_CCOMMENT != 0 {
            msg_puts(c"syncing on C-style comments".as_ptr());
            syn_lines_msg();
            syn_match_msg();
            return;
        } else if sync_flags & SF_MATCH == 0 {
            if minlines == 0 {
                msg_puts(c"no syncing".as_ptr());
            } else {
                if minlines == MAXLNUM {
                    msg_puts(c"syncing starts at the first line".as_ptr());
                } else {
                    msg_puts(c"syncing starts ".as_ptr());
                    msg_outnum(minlines);
                    msg_puts(c" lines before top line".as_ptr());
                }
                syn_match_msg();
            }
            return;
        }
        msg_puts_title(c"\n--- Syntax sync items ---".as_ptr());
        let maxlines = nvim_curwin_syn_sync_maxlines();
        let linebreaks = nvim_curwin_syn_sync_linebreaks();
        if minlines > 0 || maxlines > 0 || linebreaks > 0 {
            msg_puts(c"\nsyncing on items".as_ptr());
            syn_lines_msg();
            syn_match_msg();
        }
    } else {
        msg_puts_title(c"\n--- Syntax items ---".as_ptr());
    }

    // Track current position for nextcmd at end
    let mut cur = arg;

    if nvim_syn_ends_excmd(*cur as c_int) != 0 {
        // No argument: list all groups and clusters
        let num_groups = nvim_syn_highlight_num_groups();
        let mut id = 1;
        while id <= num_groups && nvim_syn_get_got_int() == 0 {
            syn_list_one(id, syncing != 0, false);
            id += 1;
        }
        let cluster_count = nvim_curwin_syncluster_count();
        let mut cid = 0;
        while cid < cluster_count && nvim_syn_get_got_int() == 0 {
            syn_list_cluster(cid);
            cid += 1;
        }
    } else {
        // List specific groups/clusters from argument
        while nvim_syn_ends_excmd(*cur as c_int) == 0 && nvim_syn_get_got_int() == 0 {
            let arg_end = nvim_syn_skiptowhite(cur);
            if *cur == b'@' as c_char {
                // Cluster reference
                let len = arg_end.offset_from(cur) as c_int - 1;
                let cluster_id = nvim_syn_scl_namen2id(cur.add(1), len);
                if cluster_id == 0 {
                    semsg(c"E392: No such syntax cluster: %s".as_ptr(), cur);
                } else {
                    syn_list_cluster(cluster_id - SYNID_CLUSTER);
                }
            } else {
                // Group reference
                let len = arg_end.offset_from(cur) as c_int;
                let group_id = nvim_syn_name2id_len(cur, len);
                if group_id == 0 {
                    semsg(c"E28: No such highlight group name: %s".as_ptr(), cur);
                } else {
                    syn_list_one(group_id, syncing != 0, true);
                }
            }
            cur = nvim_syn_skipwhite(arg_end);
        }
    }

    nvim_syn_eap_check_nextcmd(eap, cur);
}

// =============================================================================
// FFI export
// =============================================================================

/// Main entry point: `:syntax list` dispatcher.
///
/// # Safety
/// Must be called from the main thread with a valid `exarg_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_list(eap: *mut c_void, syncing: c_int) {
    syn_cmd_list_impl(eap, syncing);
}
