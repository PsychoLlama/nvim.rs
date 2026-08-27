//! The `:syntax` item options, and the containment test.
//!
//! [`get_syn_options`] parses the flag words (`contained`, `oneline`, `keepend`,
//! `conceal`, `nextgroup=`, ...) that may follow any item definition, and
//! [`get_id_list`] parses a group list (`contains=a,b,@cl,ALLBUT,TOP`) into the
//! `int16_t` id array the state machine tests against. [`in_id_list`] is that
//! test -- it runs once per candidate pattern per column, so it is on the
//! per-cell path even though the rest of this module is not.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::*;
use crate::regexp::RE_MAGIC;
use crate::types::{FAIL, NUL, OK};
use crate::winlayer::Win;

/// Split off a `:syntax` command's group-name argument.
///
/// `name_end` is left at the end of the name; the answer is the first argument
/// after it, or NULL when the command ended instead. The first argument may be
/// a pattern, in which which `|` is allowed, so only a NUL counts as the end.
pub(crate) unsafe fn get_group_name(arg: *mut c_char, name_end: &mut *mut c_char) -> *mut c_char {
    unsafe {
        *name_end = skiptowhite(arg);
        let rest = skipwhite(*name_end);
        if ends_excmd(*arg as c_int) != 0 || *rest as c_int == NUL {
            return ::core::ptr::null_mut();
        }
        rest
    }
}

/// What an option word takes after its name.
#[derive(Copy, Clone, PartialEq, Eq)]
enum OptArg {
    /// Nothing: a bare flag word that ORs its `HL_*` into the item's flags.
    Flag,
    /// `contains=`
    Contains,
    /// `containedin=`
    ContainedIn,
    /// `nextgroup=`
    NextGroup,
    /// `cchar=`
    Cchar,
}

impl OptArg {
    /// Whether the name must be followed by `=` rather than by the end of the
    /// argument.
    fn takes_value(self) -> bool {
        self != OptArg::Flag
    }
}

/// One recognised option word.
struct SynFlag {
    name: &'static CStr,
    arg: OptArg,
    /// The `HL_*` a bare flag word sets; 0 for the ones that take a value.
    flags: SynFlags,
}

/// A `const fn` constructor keeps each entry on one line under rustfmt.
const fn flag(name: &'static CStr, arg: OptArg, flags: SynFlags) -> SynFlag {
    SynFlag { name, arg, flags }
}

/// Every option word any `:syntax` item definition accepts.
///
/// Searched **last to first**, which is the order upstream's `--fidx` loop
/// walks it. Nothing here depends on that order — every name that is a prefix
/// of another (`conceal`/`concealends`, `contained`/`containedin`) is
/// separated by the "followed by white space, `=` or the end of the command"
/// test below — but the order is kept as documentation of that fact.
static FLAG_TAB: [SynFlag; 19] = [
    flag(c"contained", OptArg::Flag, SynFlags::CONTAINED),
    flag(c"oneline", OptArg::Flag, SynFlags::ONELINE),
    flag(c"keepend", OptArg::Flag, SynFlags::KEEPEND),
    flag(c"extend", OptArg::Flag, SynFlags::EXTEND),
    flag(c"excludenl", OptArg::Flag, SynFlags::EXCLUDENL),
    flag(c"transparent", OptArg::Flag, SynFlags::TRANSP),
    flag(c"skipnl", OptArg::Flag, SynFlags::SKIPNL),
    flag(c"skipwhite", OptArg::Flag, SynFlags::SKIPWHITE),
    flag(c"skipempty", OptArg::Flag, SynFlags::SKIPEMPTY),
    flag(c"grouphere", OptArg::Flag, SynFlags::SYNC_HERE),
    flag(c"groupthere", OptArg::Flag, SynFlags::SYNC_THERE),
    flag(c"display", OptArg::Flag, SynFlags::DISPLAY),
    flag(c"fold", OptArg::Flag, SynFlags::FOLD),
    flag(c"conceal", OptArg::Flag, SynFlags::CONCEAL),
    flag(c"concealends", OptArg::Flag, SynFlags::CONCEALENDS),
    flag(c"cchar", OptArg::Cchar, SynFlags::NONE),
    flag(c"contains", OptArg::Contains, SynFlags::NONE),
    flag(c"containedin", OptArg::ContainedIn, SynFlags::NONE),
    flag(c"nextgroup", OptArg::NextGroup, SynFlags::NONE),
];

/// Could `c` start an option word? A cheap reject, because this runs for every
/// word of a `:syntax keyword` command with a large keyword list.
///
/// Upstream spells this as `strchr(first_letters, *arg)`, which answers the
/// terminator for a NUL `*arg` and so scans the whole table once for nothing;
/// the outcome is the same either way.
fn starts_option(c: u8) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        b'c' | b'o' | b'k' | b'e' | b't' | b's' | b'g' | b'd' | b'f' | b'n'
    )
}

/// Does `arg` begin with `f`'s name, followed by what `f` requires?
///
/// The comparison is ASCII-case-insensitive, which is what upstream's
/// doubled-case name table (`"cCoOnNtTaAiInNeEdD"`) spells out a byte at a
/// time. A NUL never matches a letter, so the walk stops at the terminator.
unsafe fn flag_matches(arg: *const c_char, f: &SynFlag) -> bool {
    unsafe {
        let name = f.name.to_bytes();
        for (i, &want) in name.iter().enumerate() {
            if !(*arg.add(i) as u8).eq_ignore_ascii_case(&want) {
                return false;
            }
        }
        let after = *arg.add(name.len()) as c_int;
        ascii_iswhite(after)
            || if f.arg.takes_value() {
                after == '=' as c_int
            } else {
                ends_excmd(after) != 0
            }
    }
}

/// Which option word `arg` names, if any.
///
/// `keyword` is set while parsing `:syntax keyword`, where `display`, `fold`
/// and `extend` are keywords rather than options — a match on one of those is
/// reported as no match at all, which stops option parsing right there.
unsafe fn find_flag(arg: *const c_char, keyword: bool) -> Option<&'static SynFlag> {
    unsafe {
        let f = FLAG_TAB.iter().rev().find(|f| flag_matches(arg, f))?;
        if keyword
            && (f.flags == SynFlags::DISPLAY
                || f.flags == SynFlags::FOLD
                || f.flags == SynFlags::EXTEND)
        {
            return None;
        }
        Some(f)
    }
}

/// Read the item options at `arg`, answering the first argument that is not
/// one, or NULL on any error.
///
/// Callable at any point in an argument list and repeatedly, so that options
/// before, between and after the patterns of a `:syntax region` all land in
/// the same [`syn_opt_arg_T`].
pub(crate) unsafe fn get_syn_options(
    mut arg: *mut c_char,
    opt: &mut syn_opt_arg_T,
    conceal_char: &mut c_int,
    skip: c_int,
) -> *mut c_char {
    unsafe {
        if arg.is_null() {
            return ::core::ptr::null_mut(); // already detected error
        }
        if (*cur_syn_block()).b_syn_conceal != 0 {
            opt.flags |= SynFlags::CONCEAL;
        }

        while starts_option(*arg as u8) {
            let Some(f) = find_flag(arg, opt.keyword) else {
                break;
            };
            match f.arg {
                OptArg::Contains => {
                    if !opt.has_cont_list {
                        emsg(gettext(E_CONTAINS_NOT_ACCEPTED_HERE.as_ptr()));
                        return ::core::ptr::null_mut();
                    }
                    if get_id_list(&mut arg, 8, &mut opt.cont_list, skip != 0) == FAIL {
                        return ::core::ptr::null_mut();
                    }
                }
                OptArg::ContainedIn => {
                    if get_id_list(&mut arg, 11, &mut opt.cont_in_list, skip != 0) == FAIL {
                        return ::core::ptr::null_mut();
                    }
                }
                OptArg::NextGroup => {
                    if get_id_list(&mut arg, 9, &mut opt.next_list, skip != 0) == FAIL {
                        return ::core::ptr::null_mut();
                    }
                }
                OptArg::Cchar => {
                    // `cchar` is five letters and `flag_matches` already
                    // required the `=`, so the character starts at arg[6].
                    *conceal_char = utf_ptr2char(arg.add(6));
                    arg = arg.add(utfc_ptr2len(arg.add(6)) as usize - 1);
                    if !vim_isprintc(*conceal_char) {
                        emsg(gettext(E_INVALID_CCHAR_VALUE.as_ptr()));
                        return ::core::ptr::null_mut();
                    }
                    arg = skipwhite(arg.add(7));
                }
                OptArg::Flag => {
                    opt.flags |= f.flags;
                    arg = skipwhite(arg.add(f.name.count_bytes()));
                    if f.flags == SynFlags::SYNC_HERE || f.flags == SynFlags::SYNC_THERE {
                        arg = sync_group_arg(arg, opt);
                        if arg.is_null() {
                            return ::core::ptr::null_mut();
                        }
                    } else if f.flags == SynFlags::FOLD && foldmethod_is_syntax(Win::current()) {
                        fold_update_all(Win::current()); // Need to update folds later.
                    }
                }
            }
        }
        arg
    }
}

/// Read the group name after `grouphere`/`groupthere` and record the pattern
/// index it names in `opt.sync_idx`.
///
/// Answers what follows it, or NULL after reporting an error.
unsafe fn sync_group_arg(mut arg: *mut c_char, opt: &syn_opt_arg_T) -> *mut c_char {
    unsafe {
        if opt.sync_idx.is_null() {
            emsg(gettext(c"E393: group[t]here not accepted here".as_ptr()));
            return ::core::ptr::null_mut();
        }
        let gname_start = arg;
        arg = skiptowhite(arg);
        if gname_start == arg {
            return ::core::ptr::null_mut();
        }
        let gname = xstrnsave(gname_start, arg.offset_from(gname_start) as size_t);

        if strcmp(gname, c"NONE".as_ptr()) == 0 {
            *opt.sync_idx = NONE_IDX;
        } else {
            // The named group has to already have a region START item: this is
            // an index into the pattern array, not an id.
            let syn_id = syn_name2id(gname);
            let mut i = cur_pattern_count();
            let found = loop {
                i -= 1;
                if i < 0 {
                    break false;
                }
                let spp = cur_pattern(i);
                if (*spp).sp_syn.id as c_int == syn_id && (*spp).sp_type as c_int == SPTYPE_START {
                    *opt.sync_idx = i;
                    break true;
                }
            };
            if !found {
                semsg_c!(
                    gettext(c"E394: Didn't find region item for %s".as_ptr()),
                    gname,
                );
                xfree(gname as *mut c_void);
                return ::core::ptr::null_mut();
            }
        }

        xfree(gname as *mut c_void);
        skipwhite(arg)
    }
}

/// What one pass of [`parse_id_list`] found.
struct IdListPass {
    /// The ids, in the order the list named them.
    ids: Vec<int16_t>,
    /// Where the scan stopped. Written back to the caller's `arg` even on
    /// failure — `:syntax cluster` reports the error against it.
    end: *mut c_char,
    /// An error was reported and the whole list is to be discarded.
    failed: bool,
}

/// Turn a `contains=`-style group list into a list of ids.
///
/// `arg` points at the keyword and is advanced past the list. The argument is
/// modified in passing (the parse writes NULs into it). Answers `FAIL` on any
/// error; an existing `*list` is kept and the new one discarded.
pub(crate) unsafe fn get_id_list(
    arg: &mut *mut c_char,
    keylen: c_int,
    list: &mut *mut int16_t,
    skip: bool,
) -> c_int {
    unsafe {
        // The list is parsed more than once. A name that is a regexp matches
        // the group table as it stands, and a *later* name in the same list
        // can create a group that the regexp would also have matched
        // ("contains=a.*b,axb"), so the pass has to be repeated until it stops
        // growing. Upstream spells this as a two-round loop that resets its
        // own counter back to round 1.
        let mut previous: Option<usize> = None;
        let pass = loop {
            let pass = parse_id_list(*arg, keylen, skip);
            if pass.failed {
                break pass;
            }
            match previous {
                Some(n) if pass.ids.len() <= n => break pass,
                _ => previous = Some(pass.ids.len()),
            }
        };

        *arg = pass.end;
        if pass.failed {
            return FAIL;
        }
        // An already-parsed list is kept; upstream allocates the second one
        // and frees it again.
        if list.is_null() {
            *list = alloc_ids(&pass.ids);
        }
        OK
    }
}

/// Copy `ids` into the `xmalloc`ed, NUL-terminated array the item structs hold.
unsafe fn alloc_ids(ids: &[int16_t]) -> *mut int16_t {
    unsafe {
        let out = xmalloc((ids.len() + 1) * ::core::mem::size_of::<int16_t>()) as *mut int16_t;
        ::core::ptr::copy_nonoverlapping(ids.as_ptr(), out, ids.len());
        *out.add(ids.len()) = 0;
        out
    }
}

/// One pass over `keyword=a,b,@cl` starting at `arg`.
unsafe fn parse_id_list(arg: *mut c_char, keylen: c_int, skip: bool) -> IdListPass {
    unsafe {
        let mut ids: Vec<int16_t> = Vec::new();

        let mut p = skipwhite(arg.offset(keylen as isize));
        if *p as c_int != '=' as c_int {
            semsg_c!(gettext(c"E405: Missing equal sign: %s".as_ptr()), arg);
            return IdListPass {
                ids,
                end: p,
                failed: true,
            };
        }
        p = skipwhite(p.add(1));
        if ends_excmd(*p as c_int) != 0 {
            semsg_c!(gettext(c"E406: Empty argument: %s".as_ptr()), arg);
            return IdListPass {
                ids,
                end: p,
                failed: true,
            };
        }

        loop {
            let mut end = p;
            while *end as c_int != 0
                && !ascii_iswhite(*end as c_int)
                && *end as c_int != ',' as c_int
            {
                end = end.add(1);
            }

            match parse_id_name(arg, p, end, skip, &mut ids) {
                Ok(Some(id)) => ids.push(id as int16_t),
                // A regexp name pushed its own matches, or `skip` is on.
                Ok(None) => {}
                Err(()) => {
                    return IdListPass {
                        ids,
                        end: p,
                        failed: true,
                    };
                }
            }

            p = skipwhite(end);
            if *p as c_int != ',' as c_int {
                break;
            }
            p = skipwhite(p.add(1)); // skip comma in between arguments
            if ends_excmd(*p as c_int) != 0 {
                break;
            }
        }

        IdListPass {
            ids,
            end: p,
            failed: false,
        }
    }
}

/// Resolve one name of a group list.
///
/// Answers the id to add, `None` when the name added its own (a regexp) or
/// added nothing (`@cluster` while skipping), and `Err` when a message has
/// been given.
unsafe fn parse_id_name(
    arg: *mut c_char,
    p: *mut c_char,
    end: *mut c_char,
    skip: bool,
    ids: &mut Vec<int16_t>,
) -> Result<Option<c_int>, ()> {
    unsafe {
        let text_len = end.offset_from(p) as usize;
        // Leave room in front for the `^` and behind for the `$` the regexp
        // form needs.
        let mut name: Vec<u8> = Vec::with_capacity(text_len + 3);
        name.push(b'^');
        name.extend_from_slice(::core::slice::from_raw_parts(p as *const u8, text_len));
        name.push(0);
        let plain = name.as_ptr().add(1) as *const c_char;
        let text = &name[1..1 + text_len];

        if text == b"ALLBUT" || text == b"ALL" || text == b"TOP" || text == b"CONTAINED" {
            // Only `contains=` and `containedin=` accept these, which is what
            // upstream tests by the keyword's first letter.
            if !(*arg as u8).eq_ignore_ascii_case(&b'C') {
                semsg_c!(gettext(c"E407: %s not allowed here".as_ptr()), plain);
                return Err(());
            }
            if !ids.is_empty() {
                semsg_c!(
                    gettext(c"E408: %s must be first in contains list".as_ptr()),
                    plain,
                );
                return Err(());
            }
            let base = match text[0] {
                b'A' => SYNID_ALLBUT,
                b'T' => SYNID_TOP,
                _ => SYNID_CONTAINED,
            };
            return Ok(Some(base + current_syn_inc_tag.get()));
        }

        if text.first() == Some(&b'@') {
            if skip {
                return Ok(None);
            }
            let id = syn_check_cluster(plain.add(1), text_len as c_int - 1);
            return if id == 0 {
                semsg_c!(gettext(c"E409: Unknown group name: %s".as_ptr()), p);
                Err(())
            } else {
                Ok(Some(id))
            };
        }

        if strpbrk(plain, c"\\.*^$~[".as_ptr()).is_null() {
            let id = syn_check_group(plain, text_len as size_t);
            return if id == 0 {
                semsg_c!(gettext(c"E409: Unknown group name: %s".as_ptr()), p);
                Err(())
            } else {
                Ok(Some(id))
            };
        }

        // A regexp matching group names: add every group it matches.
        name.pop();
        name.push(b'$');
        name.push(0);
        let mut regmatch = regmatch_T {
            regprog: vim_regcomp(name.as_ptr() as *const c_char, RE_MAGIC),
            startp: [::core::ptr::null_mut(); 10],
            endp: [::core::ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: true,
        };
        if regmatch.regprog.is_null() {
            return Err(());
        }
        let mut matched = false;
        let mut i = highlight_num_groups();
        while i > 0 {
            i -= 1;
            if vim_regexec(&raw mut regmatch, highlight_group_name(i), 0) {
                ids.push((i + 1) as int16_t);
                matched = true;
            }
        }
        vim_regfree(regmatch.regprog);
        if !matched {
            semsg_c!(gettext(c"E409: Unknown group name: %s".as_ptr()), p);
            return Err(());
        }
        Ok(None)
    }
}

/// Copy an id list, which is a NUL-terminated `int16_t` array.
pub(crate) unsafe fn copy_id_list(list: *const int16_t) -> *mut int16_t {
    unsafe {
        if list.is_null() {
            return ::core::ptr::null_mut();
        }
        let mut count = 0;
        while *list.add(count) != 0 {
            count += 1;
        }
        let len = (count + 1) * ::core::mem::size_of::<int16_t>();
        let retval = xmalloc(len) as *mut int16_t;
        memmove(retval as *mut c_void, list as *const c_void, len);
        retval
    }
}

/// Is the syntax group `ssp` in the id list `list` of `cur_si`?
///
/// `cur_si` is the current item, or NULL when the `containedin` list is not
/// being checked. This runs once per candidate pattern per column: keep it
/// fast.
pub(crate) unsafe fn in_id_list(
    cur_si: *mut stateitem_T,
    list: *mut int16_t,
    ssp: *mut sp_syn,
    flags: SynFlags,
) -> bool {
    unsafe {
        // If `ssp` has a `containedin` list and `cur_si` is in it, it is
        // admitted whatever `list` says.
        if !cur_si.is_null()
            && !(*ssp).cont_in_list.is_null()
            && !(*cur_si).si_flags.has(SynFlags::MATCH)
        {
            // Ignore transparent items without a contains argument, double
            // checking that we don't go back past the first one.
            let mut si = cur_si;
            while (*si).si_flags.has(SynFlags::TRANS_CONT) && si > state_at(0) {
                si = si.offset(-1);
            }
            // si_idx is -1 for keywords, which never contain anything.
            if (*si).si_idx >= 0 {
                let spp = syn_pattern((*si).si_idx);
                if id_list_has(
                    (*ssp).cont_in_list,
                    &raw mut (*spp).sp_syn,
                    (*spp).sp_flags,
                    0,
                ) {
                    return true;
                }
            }
        }
        id_list_has(list, ssp, flags, 0)
    }
}

/// The list half of [`in_id_list`], with the cluster recursion depth threaded
/// through rather than kept in a static.
///
/// A cluster that includes itself indirectly would recurse forever, so the
/// depth is capped at 30.
unsafe fn id_list_has(
    mut list: *mut int16_t,
    ssp: *mut sp_syn,
    flags: SynFlags,
    depth: c_int,
) -> bool {
    unsafe {
        if list.is_null() {
            return false;
        }
        // ID_LIST_ALL means a transparent item that is not inside anything:
        // only not-contained groups are admitted.
        if list == ID_LIST_ALL {
            return !flags.has(SynFlags::CONTAINED);
        }

        // Is this top-level (i.e. not `contained`) in the file it was declared
        // in? For an included file that is not the same as SynFlags::CONTAINED, which
        // is set unconditionally there.
        let toplevel = !flags.has(SynFlags::CONTAINED) || flags.has(SynFlags::INCLUDED_TOPLEVEL);

        // A leading ALLBUT/TOP/CONTAINED inverts the answer, and requires the
        // group to be at the same `:syntax include` level as the list.
        let id = (*ssp).id;
        let mut item = *list;
        let mut retval = true;
        if item as c_int >= SYNID_ALLBUT && (item as c_int) < SYNID_CLUSTER {
            let level = if (item as c_int) < SYNID_TOP {
                // ALL or ALLBUT: accept all groups in the same file.
                item as c_int - SYNID_ALLBUT
            } else if (item as c_int) < SYNID_CONTAINED {
                // TOP: accept all not-contained groups in the same file.
                if !toplevel {
                    return false;
                }
                item as c_int - SYNID_TOP
            } else {
                // CONTAINED: accept all contained groups in the same file.
                if toplevel {
                    return false;
                }
                item as c_int - SYNID_CONTAINED
            };
            if level != (*ssp).inc_tag {
                return false;
            }
            list = list.add(1);
            item = *list;
            retval = false;
        }

        while item != 0 {
            if item == id {
                return retval;
            }
            if item as c_int >= SYNID_CLUSTER {
                let scl_list = (*cluster_of(item as c_int - SYNID_CLUSTER)).scl_list;
                if !scl_list.is_null() && depth < 30 && id_list_has(scl_list, ssp, flags, depth + 1)
                {
                    return retval;
                }
            }
            list = list.add(1);
            item = *list;
        }
        !retval
    }
}

/// The cluster at `idx` in the block being *parsed*.
#[inline(always)]
unsafe fn cluster_of(idx: c_int) -> *mut syn_cluster_T {
    unsafe {
        ((*syn_block.get()).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(idx as isize)
    }
}
