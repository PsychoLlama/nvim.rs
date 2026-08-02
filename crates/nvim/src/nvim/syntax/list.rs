//! `:syntax list` — the report.
//!
//! [`syn_cmd_list`] with no argument lists every item, with one lists the items
//! of a group. [`syn_list_one`] renders one group's keyword, match and region
//! items, [`put_pattern`] one pattern with its offsets, [`put_id_list`] a
//! `contains=`/`nextgroup=` list, and [`syn_list_cluster`] a cluster.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_cmd_list(
    mut eap: *mut exarg_T,
    mut syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        if !syntax_present(curwin.get()) {
            msg(gettext(MSG_NO_ITEMS.as_ptr()), 0 as ::core::ffi::c_int);
            return;
        }
        if syncing != 0 {
            if (*(*curwin.get()).w_s).b_syn_sync_flags & SF_CCOMMENT != 0 {
                msg_puts(gettext(
                    b"syncing on C-style comments\0".as_ptr() as *const ::core::ffi::c_char
                ));
                syn_lines_msg();
                syn_match_msg();
            } else if (*(*curwin.get()).w_s).b_syn_sync_flags & SF_MATCH != 0 {
                msg_puts_title(gettext(
                    b"\n--- Syntax sync items ---\0".as_ptr() as *const ::core::ffi::c_char
                ));
                if (*(*curwin.get()).w_s).b_syn_sync_minlines > 0 as linenr_T
                    || (*(*curwin.get()).w_s).b_syn_sync_maxlines > 0 as linenr_T
                    || (*(*curwin.get()).w_s).b_syn_sync_linebreaks > 0 as linenr_T
                {
                    msg_puts(gettext(
                        b"\nsyncing on items\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    syn_lines_msg();
                    syn_match_msg();
                }
                let mut id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while id <= highlight_num_groups() && !got_int.get() {
                    syn_list_one(id, syncing != 0, false_0 != 0);
                    id += 1;
                }
            } else if (*(*curwin.get()).w_s).b_syn_sync_minlines == 0 as linenr_T {
                msg_puts(gettext(
                    b"no syncing\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                if (*(*curwin.get()).w_s).b_syn_sync_minlines
                    == MAXLNUM as ::core::ffi::c_int as linenr_T
                {
                    msg_puts(gettext(b"syncing starts at the first line\0".as_ptr()
                        as *const ::core::ffi::c_char));
                } else {
                    msg_puts(gettext(
                        b"syncing starts \0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    msg_outnum((*(*curwin.get()).w_s).b_syn_sync_minlines as ::core::ffi::c_int);
                    msg_puts(gettext(
                        b" lines before top line\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                syn_match_msg();
            }
            return;
        }
        msg_puts_title(gettext(
            b"\n--- Syntax items ---\0".as_ptr() as *const ::core::ffi::c_char
        ));
        if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
            let mut id_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while id_0 <= highlight_num_groups() && !got_int.get() {
                syn_list_one(id_0, syncing != 0, false_0 != 0);
                id_0 += 1;
            }
            let mut id_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while id_1 < (*(*curwin.get()).w_s).b_syn_clusters.ga_len && !got_int.get() {
                syn_list_cluster(id_1);
                id_1 += 1;
            }
        } else {
            while ends_excmd(*arg as ::core::ffi::c_int) == 0 && !got_int.get() {
                arg_end = skiptowhite(arg);
                if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                    let mut id_2: ::core::ffi::c_int = syn_scl_namen2id(
                        arg.offset(1 as ::core::ffi::c_int as isize),
                        (arg_end.offset_from(arg) - 1 as isize) as ::core::ffi::c_int,
                    );
                    if id_2 == 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E392: No such syntax cluster: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            arg,
                        );
                    } else {
                        syn_list_cluster(id_2 - SYNID_CLUSTER);
                    }
                } else {
                    let mut id_3: ::core::ffi::c_int =
                        syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                    if id_3 == 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(&raw const e_nogroup as *const ::core::ffi::c_char),
                            arg,
                        );
                    } else {
                        syn_list_one(id_3, syncing != 0, true_0 != 0);
                    }
                }
                arg = skipwhite(arg_end);
            }
        }
        (*eap).nextcmd = check_nextcmd(arg);
    }
}

pub(crate) unsafe extern "C" fn syn_lines_msg() {
    unsafe {
        if (*(*curwin.get()).w_s).b_syn_sync_maxlines > 0 as linenr_T
            || (*(*curwin.get()).w_s).b_syn_sync_minlines > 0 as linenr_T
        {
            msg_puts(b"; \0".as_ptr() as *const ::core::ffi::c_char);
            if (*(*curwin.get()).w_s).b_syn_sync_minlines
                == MAXLNUM as ::core::ffi::c_int as linenr_T
            {
                msg_puts(gettext(
                    b"from the first line\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                if (*(*curwin.get()).w_s).b_syn_sync_minlines > 0 as linenr_T {
                    msg_puts(gettext(b"minimal \0".as_ptr() as *const ::core::ffi::c_char));
                    msg_outnum((*(*curwin.get()).w_s).b_syn_sync_minlines as ::core::ffi::c_int);
                    if (*(*curwin.get()).w_s).b_syn_sync_maxlines != 0 {
                        msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
                    }
                }
                if (*(*curwin.get()).w_s).b_syn_sync_maxlines > 0 as linenr_T {
                    msg_puts(gettext(b"maximal \0".as_ptr() as *const ::core::ffi::c_char));
                    msg_outnum((*(*curwin.get()).w_s).b_syn_sync_maxlines as ::core::ffi::c_int);
                }
                msg_puts(gettext(
                    b" lines before top line\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        }
    }
}

pub(crate) unsafe extern "C" fn syn_match_msg() {
    unsafe {
        if (*(*curwin.get()).w_s).b_syn_sync_linebreaks > 0 as linenr_T {
            msg_puts(gettext(b"; match \0".as_ptr() as *const ::core::ffi::c_char));
            msg_outnum((*(*curwin.get()).w_s).b_syn_sync_linebreaks as ::core::ffi::c_int);
            msg_puts(gettext(
                b" line breaks\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
}

pub(crate) static last_matchgroup: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

pub(crate) unsafe extern "C" fn syn_list_one(
    id: ::core::ffi::c_int,
    syncing: bool,
    link_only: bool,
) {
    unsafe {
        let mut did_header: bool = false_0 != 0;
        let hl_id: ::core::ffi::c_int = HLF_D;
        if !syncing {
            did_header = syn_list_keywords(
                id,
                &raw mut (*(*curwin.get()).w_s).b_keywtab,
                false_0 != 0,
                hl_id,
            );
            did_header = syn_list_keywords(
                id,
                &raw mut (*(*curwin.get()).w_s).b_keywtab_ic,
                did_header,
                hl_id,
            );
        }
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len && !got_int.get() {
            let spp: *const synpat_T = ((*(*curwin.get()).w_s).b_syn_patterns.ga_data
                as *mut synpat_T)
                .offset(idx as isize);
            if !((*spp).sp_syn.id as ::core::ffi::c_int != id
                || (*spp).sp_syncing as ::core::ffi::c_int != syncing as ::core::ffi::c_int)
            {
                syn_list_header(did_header, 0 as ::core::ffi::c_int, id, true_0 != 0);
                did_header = true_0 != 0;
                last_matchgroup.set(0 as ::core::ffi::c_int);
                if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH {
                    put_pattern(
                        b"match\0".as_ptr() as *const ::core::ffi::c_char,
                        ' ' as ::core::ffi::c_int,
                        spp,
                        hl_id,
                    );
                    msg_putchar(' ' as ::core::ffi::c_int);
                } else if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START {
                    while (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_START
                    {
                        let c2rust_fresh8 = idx;
                        idx = idx + 1;
                        put_pattern(
                            b"start\0".as_ptr() as *const ::core::ffi::c_char,
                            '=' as ::core::ffi::c_int,
                            ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(c2rust_fresh8 as isize),
                            hl_id,
                        );
                    }
                    if (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_SKIP
                    {
                        let c2rust_fresh9 = idx;
                        idx = idx + 1;
                        put_pattern(
                            b"skip\0".as_ptr() as *const ::core::ffi::c_char,
                            '=' as ::core::ffi::c_int,
                            ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(c2rust_fresh9 as isize),
                            hl_id,
                        );
                    }
                    while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len
                        && (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_type as ::core::ffi::c_int
                            == SPTYPE_END
                    {
                        let c2rust_fresh10 = idx;
                        idx = idx + 1;
                        put_pattern(
                            b"end\0".as_ptr() as *const ::core::ffi::c_char,
                            '=' as ::core::ffi::c_int,
                            ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(c2rust_fresh10 as isize),
                            hl_id,
                        );
                    }
                    idx -= 1;
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                syn_list_flags(
                    namelist1.ptr() as *mut keyvalue_T,
                    ::core::mem::size_of::<[keyvalue_T; 10]>()
                        .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                        .wrapping_div(
                            (::core::mem::size_of::<[keyvalue_T; 10]>()
                                .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                                == 0) as ::core::ffi::c_int as size_t,
                        ),
                    (*spp).sp_flags,
                    hl_id,
                );
                if !(*spp).sp_cont_list.is_null() {
                    put_id_list(
                        b"contains\0".as_ptr() as *const ::core::ffi::c_char,
                        (*spp).sp_cont_list,
                        hl_id,
                    );
                }
                if !(*spp).sp_syn.cont_in_list.is_null() {
                    put_id_list(
                        b"containedin\0".as_ptr() as *const ::core::ffi::c_char,
                        (*spp).sp_syn.cont_in_list,
                        hl_id,
                    );
                }
                if !(*spp).sp_next_list.is_null() {
                    put_id_list(
                        b"nextgroup\0".as_ptr() as *const ::core::ffi::c_char,
                        (*spp).sp_next_list,
                        hl_id,
                    );
                    syn_list_flags(
                        namelist2.ptr() as *mut keyvalue_T,
                        ::core::mem::size_of::<[keyvalue_T; 3]>()
                            .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                            .wrapping_div(
                                (::core::mem::size_of::<[keyvalue_T; 3]>()
                                    .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            ),
                        (*spp).sp_flags,
                        hl_id,
                    );
                }
                if (*spp).sp_flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
                    if (*spp).sp_flags & HL_SYNC_HERE != 0 {
                        msg_puts_hl(
                            b"grouphere\0".as_ptr() as *const ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                    } else {
                        msg_puts_hl(
                            b"groupthere\0".as_ptr() as *const ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                    }
                    msg_putchar(' ' as ::core::ffi::c_int);
                    if (*spp).sp_sync_idx >= 0 as ::core::ffi::c_int {
                        msg_outtrans(
                            highlight_group_name(
                                (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                    .offset((*spp).sp_sync_idx as isize))
                                .sp_syn
                                .id as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int,
                            ),
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                    } else {
                        msg_puts(b"NONE\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
            }
            idx += 1;
        }
        if highlight_link_id(id - 1 as ::core::ffi::c_int) != 0
            && (did_header as ::core::ffi::c_int != 0 || link_only as ::core::ffi::c_int != 0)
            && !got_int.get()
        {
            syn_list_header(did_header, 0 as ::core::ffi::c_int, id, true_0 != 0);
            msg_puts_hl(
                b"links to\0".as_ptr() as *const ::core::ffi::c_char,
                hl_id,
                false_0 != 0,
            );
            msg_putchar(' ' as ::core::ffi::c_int);
            msg_outtrans(
                highlight_group_name(
                    highlight_link_id(id - 1 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn syn_list_flags(
    mut nlist: *mut keyvalue_T,
    mut nr_entries: size_t,
    mut flags: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
) {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < nr_entries {
            if flags & (*nlist.offset(i as isize)).key != 0 {
                msg_puts_hl((*nlist.offset(i as isize)).value, hl_id, false_0 != 0);
                msg_putchar(' ' as ::core::ffi::c_int);
            }
            i = i.wrapping_add(1);
        }
    }
}

pub(crate) unsafe extern "C" fn syn_list_cluster(mut id: ::core::ffi::c_int) {
    unsafe {
        let mut endcol: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
        msg_putchar('\n' as ::core::ffi::c_int);
        msg_outtrans(
            (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                .offset(id as isize))
            .scl_name,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
        if msg_col.get() >= endcol {
            endcol = msg_col.get() + 1 as ::core::ffi::c_int;
        }
        if Columns.get() <= endcol {
            endcol = Columns.get() - 1 as ::core::ffi::c_int;
        }
        msg_advance(endcol);
        if !(*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
            .offset(id as isize))
        .scl_list
        .is_null()
        {
            put_id_list(
                b"cluster\0".as_ptr() as *const ::core::ffi::c_char,
                (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                    .offset(id as isize))
                .scl_list,
                HLF_D,
            );
        } else {
            msg_puts_hl(
                b"cluster\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_D,
                false_0 != 0,
            );
            msg_puts(b"=NONE\0".as_ptr() as *const ::core::ffi::c_char);
        };
    }
}

pub(crate) unsafe extern "C" fn put_id_list(
    name: *const ::core::ffi::c_char,
    list: *const int16_t,
    hl_id: ::core::ffi::c_int,
) {
    unsafe {
        msg_puts_hl(name, hl_id, false_0 != 0);
        msg_putchar('=' as ::core::ffi::c_int);
        let mut p: *const int16_t = list;
        while *p != 0 {
            if *p as ::core::ffi::c_int >= MAX_HL_ID as ::core::ffi::c_int
                && (*p as ::core::ffi::c_int) < SYNID_TOP
            {
                if *p.offset(1 as ::core::ffi::c_int as isize) != 0 {
                    msg_puts(b"ALLBUT\0".as_ptr() as *const ::core::ffi::c_char);
                } else {
                    msg_puts(b"ALL\0".as_ptr() as *const ::core::ffi::c_char);
                }
            } else if *p as ::core::ffi::c_int >= SYNID_TOP
                && (*p as ::core::ffi::c_int) < SYNID_CONTAINED
            {
                msg_puts(b"TOP\0".as_ptr() as *const ::core::ffi::c_char);
            } else if *p as ::core::ffi::c_int >= SYNID_CONTAINED
                && (*p as ::core::ffi::c_int) < SYNID_CLUSTER
            {
                msg_puts(b"CONTAINED\0".as_ptr() as *const ::core::ffi::c_char);
            } else if *p as ::core::ffi::c_int >= SYNID_CLUSTER {
                let mut scl_id: ::core::ffi::c_int = *p as ::core::ffi::c_int - SYNID_CLUSTER;
                msg_putchar('@' as ::core::ffi::c_int);
                msg_outtrans(
                    (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                        .offset(scl_id as isize))
                    .scl_name,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            } else {
                msg_outtrans(
                    highlight_group_name(*p as ::core::ffi::c_int - 1 as ::core::ffi::c_int),
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) != 0 {
                msg_putchar(',' as ::core::ffi::c_int);
            }
            p = p.offset(1);
        }
        msg_putchar(' ' as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn put_pattern(
    s: *const ::core::ffi::c_char,
    c: ::core::ffi::c_int,
    spp: *const synpat_T,
    hl_id: ::core::ffi::c_int,
) {
    unsafe {
        static sepchars: GlobalCell<*const ::core::ffi::c_char> =
            GlobalCell::new(b"/+=-#@\"|'^&\0".as_ptr() as *const ::core::ffi::c_char);
        let mut i: ::core::ffi::c_int = 0;
        if last_matchgroup.get() != (*spp).sp_syn_match_id as ::core::ffi::c_int {
            last_matchgroup.set((*spp).sp_syn_match_id as ::core::ffi::c_int);
            msg_puts_hl(
                b"matchgroup\0".as_ptr() as *const ::core::ffi::c_char,
                hl_id,
                false_0 != 0,
            );
            msg_putchar('=' as ::core::ffi::c_int);
            if last_matchgroup.get() == 0 as ::core::ffi::c_int {
                msg_outtrans(
                    b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            } else {
                msg_outtrans(
                    highlight_group_name(last_matchgroup.get() - 1 as ::core::ffi::c_int),
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        msg_puts_hl(s, hl_id, false_0 != 0);
        msg_putchar(c);
        i = 0 as ::core::ffi::c_int;
        while !vim_strchr(
            (*spp).sp_pattern,
            *(*sepchars.ptr()).offset(i as isize) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            i += 1;
            if *(*sepchars.ptr()).offset(i as isize) as ::core::ffi::c_int != NUL {
                continue;
            }
            i = 0 as ::core::ffi::c_int;
            break;
        }
        msg_putchar(*(*sepchars.ptr()).offset(i as isize) as ::core::ffi::c_int);
        msg_outtrans((*spp).sp_pattern, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_putchar(*(*sepchars.ptr()).offset(i as isize) as ::core::ffi::c_int);
        let mut first: bool = true_0 != 0;
        i = 0 as ::core::ffi::c_int;
        while i < SPO_COUNT {
            let mask: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << i;
            if (*spp).sp_off_flags as ::core::ffi::c_int & mask + (mask << SPO_COUNT) != 0 {
                if !first {
                    msg_putchar(',' as ::core::ffi::c_int);
                }
                msg_puts((*spo_name_tab.ptr())[i as usize] as *const ::core::ffi::c_char);
                let n: ::core::ffi::c_int = (*spp).sp_offsets[i as usize];
                if i != SPO_LC_OFF {
                    if (*spp).sp_off_flags as ::core::ffi::c_int & mask != 0 {
                        msg_putchar('s' as ::core::ffi::c_int);
                    } else {
                        msg_putchar('e' as ::core::ffi::c_int);
                    }
                    if n > 0 as ::core::ffi::c_int {
                        msg_putchar('+' as ::core::ffi::c_int);
                    }
                }
                if n != 0 || i == SPO_LC_OFF {
                    msg_outnum(n);
                }
                first = false_0 != 0;
            }
            i += 1;
        }
        msg_putchar(' ' as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn syn_list_keywords(
    id: ::core::ffi::c_int,
    ht: *const hashtab_T,
    mut did_header: bool,
    hl_id: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut prev_contained: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut prev_next_list: *const int16_t = ::core::ptr::null::<int16_t>();
        let mut prev_cont_in_list: *const int16_t = ::core::ptr::null::<int16_t>();
        let mut prev_skipnl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut prev_skipwhite: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut prev_skipempty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut todo: size_t = (*ht).ht_used;
        let mut hi: *const hashitem_T = (*ht).ht_array;
        while todo > 0 as size_t && !got_int.get() {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo = todo.wrapping_sub(1);
                let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                    -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                        .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                        as isize),
                ) as *mut keyentry_T;
                while !kp.is_null() && !got_int.get() {
                    if (*kp).k_syn.id as ::core::ffi::c_int == id {
                        let mut outlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut force_newline: bool = false_0 != 0;
                        if prev_contained != (*kp).flags & HL_CONTAINED
                            || prev_skipnl != (*kp).flags & HL_SKIPNL
                            || prev_skipwhite != (*kp).flags & HL_SKIPWHITE
                            || prev_skipempty != (*kp).flags & HL_SKIPEMPTY
                            || prev_cont_in_list != (*kp).k_syn.cont_in_list as *const int16_t
                            || prev_next_list != (*kp).next_list as *const int16_t
                        {
                            force_newline = true_0 != 0;
                        } else {
                            outlen = strlen(&raw mut (*kp).keyword as *mut ::core::ffi::c_char)
                                as ::core::ffi::c_int;
                        }
                        if syn_list_header(did_header, outlen, id, force_newline) {
                            prev_contained = 0 as ::core::ffi::c_int;
                            prev_next_list = ::core::ptr::null::<int16_t>();
                            prev_cont_in_list = ::core::ptr::null::<int16_t>();
                            prev_skipnl = 0 as ::core::ffi::c_int;
                            prev_skipwhite = 0 as ::core::ffi::c_int;
                            prev_skipempty = 0 as ::core::ffi::c_int;
                        }
                        did_header = true_0 != 0;
                        if prev_contained != (*kp).flags & HL_CONTAINED {
                            msg_puts_hl(
                                b"contained\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_contained = (*kp).flags & HL_CONTAINED;
                        }
                        if (*kp).k_syn.cont_in_list != prev_cont_in_list as *mut int16_t {
                            put_id_list(
                                b"containedin\0".as_ptr() as *const ::core::ffi::c_char,
                                (*kp).k_syn.cont_in_list,
                                hl_id,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_cont_in_list = (*kp).k_syn.cont_in_list;
                        }
                        if (*kp).next_list != prev_next_list as *mut int16_t {
                            put_id_list(
                                b"nextgroup\0".as_ptr() as *const ::core::ffi::c_char,
                                (*kp).next_list,
                                hl_id,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_next_list = (*kp).next_list;
                            if (*kp).flags & HL_SKIPNL != 0 {
                                msg_puts_hl(
                                    b"skipnl\0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                                msg_putchar(' ' as ::core::ffi::c_int);
                                prev_skipnl = (*kp).flags & HL_SKIPNL;
                            }
                            if (*kp).flags & HL_SKIPWHITE != 0 {
                                msg_puts_hl(
                                    b"skipwhite\0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                                msg_putchar(' ' as ::core::ffi::c_int);
                                prev_skipwhite = (*kp).flags & HL_SKIPWHITE;
                            }
                            if (*kp).flags & HL_SKIPEMPTY != 0 {
                                msg_puts_hl(
                                    b"skipempty\0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                                msg_putchar(' ' as ::core::ffi::c_int);
                                prev_skipempty = (*kp).flags & HL_SKIPEMPTY;
                            }
                        }
                        msg_outtrans(
                            &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                    }
                    kp = (*kp).ke_next;
                }
            }
            hi = hi.offset(1);
        }
        return did_header;
    }
}
