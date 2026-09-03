//! `:syntax cluster` — named groups of syntax groups.
//!
//! A cluster is an id above `SYNID_CLUSTER` that stands for a list of other
//! ids, so `contains=@Foo` can be written once and edited in one place.
//! [`syn_combine_list`] is the `contains=`/`add=`/`remove=` set arithmetic, and
//! [`syn_check_cluster`] resolves a name to an id, creating it if needed.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};
use std::ffi::CString;

use super::*;

/// Combine two cluster lists in place: `*clstr1 <op>= clstr2`.
///
/// `list_op` is `CLUSTER_REPLACE`, `CLUSTER_ADD` (union) or
/// `CLUSTER_SUBTRACT` (difference); `clstr2` is consumed either way.
pub(crate) fn syn_combine_list(clstr1: &mut IdList, clstr2: IdList, list_op: c_int) {
    // Degenerate cases: nothing to combine with, or nothing to combine.
    if clstr2.is_none() {
        return;
    }
    if clstr1.is_none() || list_op == CLUSTER_REPLACE {
        if list_op == CLUSTER_REPLACE || list_op == CLUSTER_ADD {
            *clstr1 = clstr2;
        }
        return;
    }

    // Sorting both lets the merge below be linear. `sort_unstable` is
    // sound where a general qsort replacement is not: two equal `int16_t`s
    // are indistinguishable, so every permutation of them is the same
    // array.
    let mut a: Vec<int16_t> = clstr1.ids().into();
    let mut b: Vec<int16_t> = clstr2.ids().into();
    a.sort_unstable();
    b.sort_unstable();

    *clstr1 = IdList::from_ids_or_none(&merge_id_lists(&a, &b, list_op));
}

/// Merge two sorted id lists, taking everything from `a` and — only when
/// adding — everything from `b`.
///
/// Subtracting drops an id of `a` that `b` also has by advancing past both.
fn merge_id_lists(a: &[int16_t], b: &[int16_t], list_op: c_int) -> Vec<int16_t> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            out.push(a[i]);
            i += 1;
            continue;
        }
        if list_op == CLUSTER_ADD {
            out.push(b[j]);
        }
        if a[i] == b[j] {
            i += 1;
        }
        j += 1;
    }
    out.extend_from_slice(&a[i..]);
    if list_op == CLUSTER_ADD {
        out.extend_from_slice(&b[j..]);
    }
    out
}

/// The name upper-cased, which is the form a cluster lookup compares:
/// `stricmp` is slow on some systems, so each cluster stores both.
fn upper(name: &CStr) -> CString {
    let mut bytes = name.to_bytes().to_vec();
    bytes.make_ascii_uppercase();
    cstr::owned(&bytes)
}

/// Look a cluster name up, answering its id or 0.
fn scl_name2id(name: &CStr) -> c_int {
    let name_u = upper(name);
    let block = cur_syn_block();
    match block
        .clusters()
        .iter()
        .rposition(|c| c.scl_name_u == name_u)
    {
        Some(i) => i as c_int + SYNID_CLUSTER,
        None => 0,
    }
}

/// Look up the cluster named by `len` bytes at `linep`, answering 0 when there
/// is no such cluster.
pub(crate) unsafe fn syn_scl_namen2id(linep: *const c_char, len: c_int) -> c_int {
    // SAFETY: the caller's promise -- `len` readable bytes.
    scl_name2id(&unsafe { name_at(linep, len as usize) })
}

/// Like [`syn_scl_namen2id`], but create the cluster when it does not exist.
///
/// Answers 0 only when there is no room for another cluster.
pub(crate) unsafe fn syn_check_cluster(pp: *const c_char, len: c_int) -> c_int {
    // SAFETY: the caller's promise -- `len` readable bytes.
    let name = unsafe { name_at(pp, len as usize) };
    match scl_name2id(&name) {
        0 => syn_add_cluster(name),
        id => id,
    }
}

/// Add a cluster with no members, answering its id, or 0 when the table is
/// full.
fn syn_add_cluster(name: CString) -> c_int {
    let mut block = cur_syn_block();
    let len = block.clusters().len() as c_int;
    if len >= MAX_CLUSTER_ID {
        emsg(gettext(c"E848: Too many syntax clusters"));
        return 0;
    }

    // The two clusters the spell checker asks about by id.
    if name.to_bytes().eq_ignore_ascii_case(b"Spell") {
        block.b_spell_cluster_id = len + SYNID_CLUSTER;
    }
    if name.to_bytes().eq_ignore_ascii_case(b"NoSpell") {
        block.b_nospell_cluster_id = len + SYNID_CLUSTER;
    }

    let scl_name_u = upper(&name);
    block.clusters_mut().push(syn_cluster_T {
        scl_name: name,
        scl_name_u,
        scl_list: IdList::NONE,
    });
    len + SYNID_CLUSTER
}

/// The three list operations `:syntax cluster` accepts, longest name first in
/// no particular order: no two are prefixes of each other.
const CLUSTER_OPS: [(&CStr, c_int); 3] = [
    (c"add", CLUSTER_ADD),
    (c"remove", CLUSTER_SUBTRACT),
    (c"contains", CLUSTER_REPLACE),
];

/// Which operation `rest` names, and how many bytes its keyword took.
///
/// A keyword must be followed by white space or `=`, and a failed test falls
/// through to the next candidate rather than claiming the argument.
unsafe fn cluster_op(rest: *const c_char) -> Option<(c_int, c_int)> {
    for (name, op) in CLUSTER_OPS {
        let len = name.count_bytes();
        if unsafe { strncasecmp(rest, name.as_ptr(), len) } != 0 {
            continue;
        }
        let after = unsafe { *rest.add(len) } as c_int;
        if ascii_iswhite(after) || after == '=' as c_int {
            return Some((len as c_int, op));
        }
    }
    None
}

/// `:syntax cluster {name} [contains=..] [add=..] [remove=..]`.
pub(crate) fn syn_cmd_cluster(eap: &mut exarg_T, _syncing: c_int) {
    let arg = eap.arg;
    let mut group_name_end = ::core::ptr::null_mut::<c_char>();
    let mut got_clstr = false;

    eap.nextcmd = unsafe { find_nextcmd(arg) };
    if eap.skip != 0 {
        return;
    }

    let mut rest = unsafe { get_group_name(arg, &mut group_name_end) };
    if !rest.is_null() {
        let scl_id = unsafe { syn_check_cluster(arg, group_name_end.offset_from(arg) as c_int) };
        if scl_id == 0 {
            return;
        }
        // Always a valid index: `syn_check_cluster` answers either 0,
        // handled above, or an id at or above `SYNID_CLUSTER`. Upstream
        // tests `scl_id >= 0` here and frees the list on the other branch;
        // that branch is unreachable.
        let scl_id = scl_id - SYNID_CLUSTER;

        while let Some((opt_len, list_op)) = unsafe { cluster_op(rest) } {
            let mut clstr_list = IdList::NONE;
            if unsafe { get_id_list(&mut rest, opt_len, &mut clstr_list, eap.skip != 0) }.is_err() {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let rest = unsafe { c_str(rest) };
                semsg!("E475: Invalid argument: {rest}");
                break;
            }
            let mut block = cur_syn_block();
            let list = &mut block.clusters_mut()[scl_id as usize].scl_list;
            syn_combine_list(list, clstr_list, list_op);
            got_clstr = true;
        }

        if got_clstr {
            redraw_curbuf_later(UPD_SOME_VALID);
            unsafe { syn_stack_free_all(cur_syn_block().raw()) }; // Need to recompute all.
        }
    }

    if !got_clstr {
        emsg(gettext(c"E400: No cluster specified"));
    }
    if rest.is_null() || ends_excmd(unsafe { *rest } as c_int) == 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {arg}");
    }
}
