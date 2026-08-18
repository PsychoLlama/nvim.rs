//! `:syntax cluster` — named groups of syntax groups.
//!
//! A cluster is an id above `SYNID_CLUSTER` that stands for a list of other
//! ids, so `contains=@Foo` can be written once and edited in one place.
//! [`syn_combine_list`] is the `contains=`/`add=`/`remove=` set arithmetic, and
//! [`syn_check_cluster`] resolves a name to an id, creating it if needed.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::*;

/// The ids of a cluster list, which is a NUL-terminated `int16_t` array.
///
/// Empty for a NULL list: `slice::from_raw_parts` rejects a null base even at
/// length zero.
unsafe fn id_list(list: *const int16_t) -> &'static [int16_t] {
    unsafe {
        if list.is_null() {
            return &[];
        }
        let mut len = 0;
        while *list.add(len) != 0 {
            len += 1;
        }
        ::core::slice::from_raw_parts(list, len)
    }
}

/// Copy `ids` into a fresh NUL-terminated `xmalloc`ed list, or NULL when empty.
///
/// The lists live in `syn_cluster_T::scl_list` and are freed with `xfree`, so
/// they cannot be a `Vec`.
unsafe fn alloc_id_list(ids: &[int16_t]) -> *mut int16_t {
    unsafe {
        if ids.is_empty() {
            return ::core::ptr::null_mut();
        }
        let out = xmalloc((ids.len() + 1) * ::core::mem::size_of::<int16_t>()) as *mut int16_t;
        ::core::ptr::copy_nonoverlapping(ids.as_ptr(), out, ids.len());
        *out.add(ids.len()) = 0;
        out
    }
}

/// Combine two cluster lists in place: `*clstr1 <op>= *clstr2`.
///
/// Both must be allocated; both are consumed. `list_op` is `CLUSTER_REPLACE`,
/// `CLUSTER_ADD` (union) or `CLUSTER_SUBTRACT` (difference).
pub(crate) unsafe fn syn_combine_list(
    clstr1: &mut *mut int16_t,
    clstr2: &mut *mut int16_t,
    list_op: c_int,
) {
    unsafe {
        // Degenerate cases: nothing to combine with, or nothing to combine.
        if clstr2.is_null() {
            return;
        }
        if clstr1.is_null() || list_op == CLUSTER_REPLACE {
            if list_op == CLUSTER_REPLACE {
                xfree(*clstr1 as *mut c_void);
            }
            if list_op == CLUSTER_REPLACE || list_op == CLUSTER_ADD {
                *clstr1 = *clstr2;
            } else {
                xfree(*clstr2 as *mut c_void);
            }
            return;
        }

        // Sorting both lets the merge below be linear. `sort_unstable` is
        // sound where a general qsort replacement is not: two equal `int16_t`s
        // are indistinguishable, so every permutation of them is the same
        // array.
        let a = ::core::slice::from_raw_parts_mut(*clstr1, id_list(*clstr1).len());
        let b = ::core::slice::from_raw_parts_mut(*clstr2, id_list(*clstr2).len());
        a.sort_unstable();
        b.sort_unstable();

        let merged = merge_id_lists(a, b, list_op);
        let out = alloc_id_list(&merged);

        xfree(*clstr1 as *mut c_void);
        xfree(*clstr2 as *mut c_void);
        *clstr1 = out;
    }
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

/// Look a cluster name up, answering its id or 0.
///
/// The names are compared upper-cased, because `stricmp` is slow on some
/// systems and the upper-cased form is stored alongside each name.
unsafe fn scl_name2id(name: *const c_char) -> c_int {
    unsafe {
        let name_u = vim_strsave_up(name);
        let mut i = cur_cluster_count();
        let id = loop {
            i -= 1;
            if i < 0 {
                break 0;
            }
            let stored = (*cur_cluster(i)).scl_name_u;
            if !stored.is_null() && strcmp(name_u, stored) == 0 {
                break i + SYNID_CLUSTER;
            }
        };
        xfree(name_u as *mut c_void);
        id
    }
}

/// Look up the cluster named by `len` bytes at `linep`, answering 0 when there
/// is no such cluster.
pub(crate) unsafe fn syn_scl_namen2id(linep: *const c_char, len: c_int) -> c_int {
    unsafe {
        let name = xstrnsave(linep, len as size_t);
        let id = scl_name2id(name);
        xfree(name as *mut c_void);
        id
    }
}

/// Like [`syn_scl_namen2id`], but create the cluster when it does not exist.
///
/// Answers 0 only when there is no room for another cluster.
pub(crate) unsafe fn syn_check_cluster(pp: *const c_char, len: c_int) -> c_int {
    unsafe {
        let name = xstrnsave(pp, len as size_t);
        let id = scl_name2id(name);
        if id != 0 {
            xfree(name as *mut c_void);
            return id;
        }
        syn_add_cluster(name)
    }
}

/// Add a cluster with no members, answering its id, or 0 when the table is
/// full. Consumes `name`.
unsafe fn syn_add_cluster(name: *mut c_char) -> c_int {
    unsafe {
        let clusters = &raw mut (*cur_syn_block()).b_syn_clusters;
        // First call for this window: init the growing array.
        if (*clusters).ga_data.is_null() {
            (*clusters).ga_itemsize = ::core::mem::size_of::<syn_cluster_T>() as c_int;
            ga_set_growsize(clusters, 10);
        }

        let len = (*clusters).ga_len;
        if len >= MAX_CLUSTER_ID {
            emsg(gettext(c"E848: Too many syntax clusters".as_ptr()));
            xfree(name as *mut c_void);
            return 0;
        }

        let scp = ga_append_via_ptr(clusters, ::core::mem::size_of::<syn_cluster_T>())
            as *mut syn_cluster_T;
        (*scp).scl_name = name;
        (*scp).scl_name_u = vim_strsave_up(name);
        (*scp).scl_list = ::core::ptr::null_mut();

        // The two clusters the spell checker asks about by id.
        if strcasecmp(name, c"Spell".as_ptr()) == 0 {
            (*cur_syn_block()).b_spell_cluster_id = len + SYNID_CLUSTER;
        }
        if strcasecmp(name, c"NoSpell".as_ptr()) == 0 {
            (*cur_syn_block()).b_nospell_cluster_id = len + SYNID_CLUSTER;
        }

        len + SYNID_CLUSTER
    }
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
    unsafe {
        for (name, op) in CLUSTER_OPS {
            let len = name.count_bytes();
            if strncasecmp(rest, name.as_ptr(), len) != 0 {
                continue;
            }
            let after = *rest.add(len) as c_int;
            if ascii_iswhite(after) || after == '=' as c_int {
                return Some((len as c_int, op));
            }
        }
        None
    }
}

/// `:syntax cluster {name} [contains=..] [add=..] [remove=..]`.
pub(crate) unsafe fn syn_cmd_cluster(eap: *mut exarg_T, _syncing: c_int) {
    unsafe {
        let arg = (*eap).arg;
        let mut group_name_end = ::core::ptr::null_mut::<c_char>();
        let mut got_clstr = false;

        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }

        let mut rest = get_group_name(arg, &mut group_name_end);
        if !rest.is_null() {
            let scl_id = syn_check_cluster(arg, group_name_end.offset_from(arg) as c_int);
            if scl_id == 0 {
                return;
            }
            // Always a valid index: `syn_check_cluster` answers either 0,
            // handled above, or an id at or above `SYNID_CLUSTER`. Upstream
            // tests `scl_id >= 0` here and frees the list on the other branch;
            // that branch is unreachable.
            let scl_id = scl_id - SYNID_CLUSTER;

            while let Some((opt_len, list_op)) = cluster_op(rest) {
                let mut clstr_list = ::core::ptr::null_mut::<int16_t>();
                if get_id_list(&mut rest, opt_len, &mut clstr_list, (*eap).skip != 0) == FAIL {
                    semsg_c!(gettext(&raw const e_invarg2 as *const c_char), rest);
                    break;
                }
                syn_combine_list(
                    &mut (*cur_cluster(scl_id)).scl_list,
                    &mut clstr_list,
                    list_op,
                );
                got_clstr = true;
            }

            if got_clstr {
                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all(cur_syn_block()); // Need to recompute all.
            }
        }

        if !got_clstr {
            emsg(gettext(c"E400: No cluster specified".as_ptr()));
        }
        if rest.is_null() || ends_excmd(*rest as c_int) == 0 {
            semsg_c!(gettext(&raw const e_invarg2 as *const c_char), arg);
        }
    }
}
