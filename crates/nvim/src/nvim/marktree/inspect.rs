//! Rendering a tree for a human: `:lua vim.api.nvim__buf_debug_extmarks()`.
//!
//! Two formats. The default is a nested parenthesised dump, one node per line,
//! with each node's intersection set in braces. With `dot` it emits a Graphviz
//! digraph instead, one HTML table per node, which is the only practical way to
//! see what a rebalancing did.

use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mt_inspect(
    mut b: *mut MarkTree,
    mut keys: bool,
    mut dot: bool,
) -> String_0 {
    let mut ga: [garray_T; 1] = [garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    }; 1];
    ga_init(
        &raw mut ga as *mut garray_T,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let mut p: MTPos = MTPos {
        row: 0 as int32_t,
        col: 0 as int32_t,
    };
    if !(*b).root.is_null() {
        if dot {
            ga_concat(
                &raw mut ga as *mut garray_T,
                c"digraph D {\n\n".as_ptr() as *mut ::core::ffi::c_char,
            );
            mt_inspect_dotfile_node(
                &raw mut ga as *mut garray_T,
                (*b).root,
                p,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
            ga_concat(
                &raw mut ga as *mut garray_T,
                c"\n}".as_ptr() as *mut ::core::ffi::c_char,
            );
        } else {
            mt_inspect_node(&raw mut ga as *mut garray_T, keys, (*b).root, p);
        }
    }
    return ga_take_string(&raw mut ga as *mut garray_T);
}

#[inline]
pub unsafe extern "C" fn mt_dbg_id(mut id: uint64_t) -> uint64_t {
    return id >> 1 as ::core::ffi::c_int & 0xffffffff as uint64_t;
}

pub unsafe extern "C" fn mt_inspect_node(
    mut ga: *mut garray_T,
    mut keys: bool,
    mut n: *mut MTNode,
    mut off: MTPos,
) {
    static buf: GlobalCell<[::core::ffi::c_char; 1024]> = GlobalCell::new([0; 1024]);
    ga_concat(ga, c"[".as_ptr() as *mut ::core::ffi::c_char);
    if keys as ::core::ffi::c_int != 0 && !ix(n).is_empty() {
        for (i, &id) in ix(n).as_slice().iter().enumerate() {
            let sep = if i == 0 { c"{" } else { c";" };
            ga_concat(ga, sep.as_ptr() as *mut ::core::ffi::c_char);
            snprintf(
                buf.ptr() as *mut ::core::ffi::c_char,
                size_of::<[::core::ffi::c_char; 1024]>(),
                c"%lu".as_ptr(),
                mt_dbg_id(id),
            );
            ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
        }
        ga_concat(ga, c"},".as_ptr() as *mut ::core::ffi::c_char);
    }
    if (*n).level != 0 {
        mt_inspect_node(
            ga,
            keys,
            (*inner(n)).i_ptr[0 as ::core::ffi::c_int as usize],
            off,
        );
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i_0 as int32_t) < (*n).n {
        let mut p: MTPos = (*n).key[i_0 as usize].pos;
        unrelative(off, &mut p);
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            c"%d/%d".as_ptr(),
            p.row,
            p.col,
        );
        ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
        if keys {
            let mut key: MTKey = (*n).key[i_0 as usize];
            ga_concat(ga, c":".as_ptr() as *mut ::core::ffi::c_char);
            if mt_start(key) {
                ga_concat(ga, c"<".as_ptr() as *mut ::core::ffi::c_char);
            }
            snprintf(
                buf.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
                c"%u".as_ptr(),
                key.id,
            );
            ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
            if mt_end(key) {
                ga_concat(ga, c">".as_ptr() as *mut ::core::ffi::c_char);
            }
        }
        if (*n).level != 0 {
            mt_inspect_node(
                ga,
                keys,
                (*inner(n)).i_ptr[(i_0 + 1 as ::core::ffi::c_int) as usize],
                p,
            );
        } else {
            ga_concat(ga, c",".as_ptr());
        }
        i_0 += 1;
    }
    ga_concat(ga, c"]".as_ptr());
}

pub unsafe extern "C" fn mt_inspect_dotfile_node(
    mut ga: *mut garray_T,
    mut n: *mut MTNode,
    mut off: MTPos,
    mut parent: *mut ::core::ffi::c_char,
) {
    static buf: GlobalCell<[::core::ffi::c_char; 1024]> = GlobalCell::new([0; 1024]);
    let mut namebuf: [::core::ffi::c_char; 64] = [0; 64];
    if !parent.is_null() {
        snprintf(
            &raw mut namebuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            c"%s_%c%d".as_ptr(),
            parent,
            'a' as ::core::ffi::c_int + (*n).level as ::core::ffi::c_int,
            (*n).p_idx as ::core::ffi::c_int,
        );
    } else {
        snprintf(
            &raw mut namebuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            c"MTNode".as_ptr(),
        );
    }
    snprintf(
        buf.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
        c"  %s[shape=plaintext, label=<\n".as_ptr(),
        &raw mut namebuf as *mut ::core::ffi::c_char,
    );
    ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
    ga_concat(
        ga,
        c"    <table border='0' cellborder='1' cellspacing='0'>\n".as_ptr()
            as *mut ::core::ffi::c_char,
    );
    if !ix(n).is_empty() {
        ga_concat(ga, c"    <tr><td>".as_ptr() as *mut ::core::ffi::c_char);
        for (i, &id) in ix(n).as_slice().iter().enumerate() {
            if i > 0 {
                ga_concat(ga, c", ".as_ptr() as *mut ::core::ffi::c_char);
            }
            snprintf(
                buf.ptr() as *mut ::core::ffi::c_char,
                size_of::<[::core::ffi::c_char; 1024]>(),
                c"%lu".as_ptr(),
                mt_dbg_id(id),
            );
            ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
        }
        ga_concat(ga, c"</td></tr>\n".as_ptr() as *mut ::core::ffi::c_char);
    }
    ga_concat(ga, c"    <tr><td>".as_ptr() as *mut ::core::ffi::c_char);
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i_0 as int32_t) < (*n).n {
        let mut k: MTKey = (*n).key[i_0 as usize];
        if i_0 > 0 as ::core::ffi::c_int {
            ga_concat(ga, c", ".as_ptr() as *mut ::core::ffi::c_char);
        }
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            c"%d".as_ptr(),
            k.id,
        );
        ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
        if mt_paired(k) {
            ga_concat(
                ga,
                (if mt_end(k) as ::core::ffi::c_int != 0 {
                    c"e".as_ptr()
                } else {
                    c"s".as_ptr()
                }) as *mut ::core::ffi::c_char,
            );
        }
        i_0 += 1;
    }
    ga_concat(ga, c"</td></tr>\n".as_ptr() as *mut ::core::ffi::c_char);
    ga_concat(ga, c"    </table>\n".as_ptr() as *mut ::core::ffi::c_char);
    ga_concat(ga, c">];\n".as_ptr() as *mut ::core::ffi::c_char);
    if !parent.is_null() {
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            c"  %s -> %s\n".as_ptr(),
            parent,
            &raw mut namebuf as *mut ::core::ffi::c_char,
        );
        ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
    }
    if (*n).level != 0 {
        mt_inspect_dotfile_node(
            ga,
            (*inner(n)).i_ptr[0 as ::core::ffi::c_int as usize],
            off,
            &raw mut namebuf as *mut ::core::ffi::c_char,
        );
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i_1 as int32_t) < (*n).n {
        let mut p: MTPos = (*n).key[i_1 as usize].pos;
        unrelative(off, &mut p);
        if (*n).level != 0 {
            mt_inspect_dotfile_node(
                ga,
                (*inner(n)).i_ptr[(i_1 + 1 as ::core::ffi::c_int) as usize],
                p,
                &raw mut namebuf as *mut ::core::ffi::c_char,
            );
        }
        i_1 += 1;
    }
}
