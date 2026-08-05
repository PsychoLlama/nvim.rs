//! The `sign_*()` Vimscript functions.
//!
//! The same operations the `:sign` command performs, addressed by
//! dictionary rather than by command line, plus the two report functions
//! (`sign_getdefined()`, `sign_getplaced()`) that answer with dictionaries
//! of their own. The `*_from_dict` helpers are shared between the single
//! and the `*list()` bulk forms.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::types::{VAR_DICT, VAR_LIST, VAR_UNKNOWN, kListLenMayKnow};

/// The four highlight keys a sign definition carries, in the order every
/// reader in this family reports them.
const HL_KEYS: [&str; 4] = ["linehl", "texthl", "culhl", "numhl"];

/// `tv_dict_add_str` with a Rust key.
///
/// `tv_dict_item_alloc_len` copies exactly the length it is given, so the key
/// is a plain `&str` and the transpile's `b"name\0"` plus
/// `size_of::<[c_char; 5]>() - 1` goes.
///
/// # Safety
/// `d` must be a live dictionary and `val` a NUL-terminated string.
unsafe fn put_str(d: *mut dict_T, key: &str, val: *const ::core::ffi::c_char) {
    // SAFETY: the caller's dictionary and value.
    unsafe {
        tv_dict_add_str(d, key.as_ptr().cast(), key.len(), val);
    }
}

/// `tv_dict_add_nr` with a Rust key; see [`put_str`].
///
/// # Safety
/// `d` must be a live dictionary.
unsafe fn put_nr(d: *mut dict_T, key: &str, nr: varnumber_T) {
    // SAFETY: the caller's dictionary.
    unsafe {
        tv_dict_add_nr(d, key.as_ptr().cast(), key.len(), nr);
    }
}

/// `tv_dict_find` with a Rust key; null when the key is absent.
///
/// # Safety
/// `d` must be null or a live dictionary.
unsafe fn find(d: *const dict_T, key: &str) -> *mut dictitem_T {
    // SAFETY: the caller's dictionary.
    unsafe { tv_dict_find(d, key.as_ptr().cast(), key.len() as ptrdiff_t) }
}

/// The name of highlight group `id`, or `"NONE"` when it has none.
///
/// # Safety
/// None beyond `get_highlight_name_ext`'s.
unsafe fn hl_name(id: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    // SAFETY: the null `expand_T` is the "no completion context" argument.
    unsafe {
        let p = get_highlight_name_ext(::core::ptr::null_mut(), id - 1, false);
        if p.is_null() { c"NONE".as_ptr() } else { p }
    }
}

/// Walks a `list_T`, yielding each item in order.
///
/// # Safety
/// `l` must be null or a live list the body does not modify.
unsafe fn list_items(l: *const list_T) -> impl Iterator<Item = *mut listitem_T> {
    // SAFETY: the caller's list.
    let mut at = unsafe { tv_list_first(l) };
    ::core::iter::from_fn(move || {
        if at.is_null() {
            return None;
        }
        let item = at;
        // SAFETY: `item` is a live element of the caller's list.
        at = unsafe { (*item).li_next };
        Some(item)
    })
}

/// `sign_getdefined()`'s dictionary for one defined sign.
///
/// # Safety
/// `sp` must be a live sign definition.
pub(crate) unsafe fn sign_get_info_dict(sp: *mut sign_T) -> *mut dict_T {
    // SAFETY: the caller's definition.
    unsafe {
        let d = tv_dict_alloc();
        put_str(d, "name", (*sp).sn_name);
        if !(*sp).sn_icon.is_null() {
            put_str(d, "icon", (*sp).sn_icon);
        }
        if (*sp).sn_text[0] != 0 {
            let mut buf = [0 as ::core::ffi::c_char; SIGN_TEXT_BUF];
            describe_sign_text(buf.as_mut_ptr(), (&raw mut (*sp).sn_text).cast());
            put_str(d, "text", buf.as_ptr());
        }
        if (*sp).sn_priority > 0 {
            put_nr(d, "priority", (*sp).sn_priority as varnumber_T);
        }
        let ids = [
            (*sp).sn_line_hl,
            (*sp).sn_text_hl,
            (*sp).sn_cul_hl,
            (*sp).sn_num_hl,
        ];
        for (key, id) in HL_KEYS.iter().zip(ids) {
            if id > 0 {
                put_str(d, key, hl_name(id));
            }
        }
        d
    }
}

/// `sign_getplaced()`'s dictionary for one placed sign.
///
/// # Safety
/// `mark` must carry a live sign decoration.
pub(crate) unsafe fn sign_get_placed_info_dict(mark: MTKey) -> *mut dict_T {
    // SAFETY: the caller's mark.
    unsafe {
        let d = tv_dict_alloc();
        let sh = decor_find_sign(mt_decor(mark));
        put_str(d, "name", sign_get_name(sh));
        put_nr(d, "id", mark.id as ::core::ffi::c_int as varnumber_T);
        put_str(d, "group", describe_ns(mark.ns as NS, c"".as_ptr()));
        put_nr(d, "lnum", (mark.pos.row + 1) as varnumber_T);
        put_nr(d, "priority", (*sh).priority as varnumber_T);
        d
    }
}

/// Every sign placed in `buf`, in marktree order — `getbufinfo()`'s `signs`.
///
/// # Safety
/// `buf` must be live.
pub unsafe fn get_buffer_signs(buf: *mut buf_T) -> *mut list_T {
    // SAFETY: the caller's buffer.
    unsafe {
        let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        let tree: *mut MarkTree = (&raw mut (*buf).b_marktree).cast();
        let mut itr = MarkTreeIter::default();
        marktree_itr_get(tree, 0, 0, &raw mut itr);
        while !itr.x.is_null() {
            let mark = marktree_itr_current(&raw mut itr);
            if !mt_end(mark) && mt_decor_sign(mark) {
                tv_list_append_dict(l, sign_get_placed_info_dict(mark));
            }
            marktree_itr_next(tree, &raw mut itr);
        }
        l
    }
}

/// Appends `buf`'s `{ bufnr, signs }` entry to `retlist`, filtered by `lnum`,
/// `sign_id` and `group`.
///
/// A zero `lnum` or `sign_id` means "any"; the two combine, so naming both
/// asks for one specific sign on one specific line.
///
/// # Safety
/// `buf` and `retlist` must be live; `group` must be null or NUL-terminated.
unsafe fn sign_get_placed_in_buf(
    buf: *mut buf_T,
    lnum: linenr_T,
    sign_id: ::core::ffi::c_int,
    group: *const ::core::ffi::c_char,
    retlist: *mut list_T,
) {
    // SAFETY: the caller's buffer, group and list.
    unsafe {
        let d = tv_dict_alloc();
        tv_list_append_dict(retlist, d);
        put_nr(d, "bufnr", (*buf).handle as varnumber_T);
        let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        tv_dict_add_list(d, "signs".as_ptr().cast(), "signs".len(), l);

        let ns = group_get_ns(group);
        if !buf_has_signs(buf) || ns < 0 {
            return;
        }

        let tree: *mut MarkTree = (&raw mut (*buf).b_marktree).cast();
        let mut itr = MarkTreeIter::default();
        let mut signs: Vec<MTKey> = Vec::new();
        let first_row = if lnum != 0 { lnum - 1 } else { 0 };
        marktree_itr_get(tree, first_row, 0, &raw mut itr);

        while !itr.x.is_null() {
            let mark = marktree_itr_current(&raw mut itr);
            if lnum != 0 && mark.pos.row >= lnum {
                break;
            }
            let wanted = (lnum == 0 && sign_id == 0)
                || (sign_id == 0 && lnum == mark.pos.row + 1)
                || (lnum == 0 && sign_id == mark.id as ::core::ffi::c_int)
                || (lnum == mark.pos.row + 1 && sign_id == mark.id as ::core::ffi::c_int);
            if !mt_end(mark)
                && (ns == ALL_GROUPS || ns == mark.ns as int64_t)
                && wanted
                && mt_decor_sign(mark)
            {
                signs.push(mark);
            }
            marktree_itr_next(tree, &raw mut itr);
        }

        sort_signs(&mut signs);
        for mark in signs {
            tv_list_append_dict(l, sign_get_placed_info_dict(mark));
        }
    }
}

/// Appends the placed-sign report for `buf`, or for every buffer that has
/// signs when `buf` is null.
///
/// # Safety
/// `buf` must be null or live; `retlist` must be live.
unsafe fn sign_get_placed(
    buf: *mut buf_T,
    lnum: linenr_T,
    id: ::core::ffi::c_int,
    group: *const ::core::ffi::c_char,
    retlist: *mut list_T,
) {
    // SAFETY: the caller's buffer and list.
    unsafe {
        if !buf.is_null() {
            sign_get_placed_in_buf(buf, lnum, id, group, retlist);
            return;
        }
        let mut cbuf = firstbuf.get();
        while !cbuf.is_null() {
            if buf_has_signs(cbuf) {
                // `lnum` is deliberately dropped: an all-buffers query
                // reports every line whatever line was asked for.
                sign_get_placed_in_buf(cbuf, 0, id, group, retlist);
            }
            cbuf = (*cbuf).b_next;
        }
    }
}

/// Defines one sign from a dictionary; 0 on success, −1 on failure.
///
/// `name` is null for the list form, where the name is the dictionary's own
/// `name` key.
///
/// # Safety
/// `name` must be null or NUL-terminated; `dict` must be null or live.
unsafe fn sign_define_from_dict(
    name: *mut ::core::ffi::c_char,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's name and dictionary.
    unsafe {
        let mut name = name;
        if name.is_null() {
            name = tv_dict_get_string(dict, c"name".as_ptr(), false);
            if name.is_null() || *name == 0 {
                return -1;
            }
        }
        let null = ::core::ptr::null_mut();
        let (mut icon, mut text) = (null, null);
        let (mut linehl, mut texthl, mut culhl, mut numhl) = (null, null, null, null);
        let mut prio = -1;
        if !dict.is_null() {
            // `tv_dict_get_string(.., false)` hands back the dictionary's own
            // buffer, which `init_sign_text` then unescapes IN PLACE — see
            // the note on `sign_define_by_name`.
            icon = tv_dict_get_string(dict, c"icon".as_ptr(), false);
            linehl = tv_dict_get_string(dict, c"linehl".as_ptr(), false);
            text = tv_dict_get_string(dict, c"text".as_ptr(), false);
            texthl = tv_dict_get_string(dict, c"texthl".as_ptr(), false);
            culhl = tv_dict_get_string(dict, c"culhl".as_ptr(), false);
            numhl = tv_dict_get_string(dict, c"numhl".as_ptr(), false);
            prio = tv_dict_get_number_def(dict, c"priority".as_ptr(), -1) as ::core::ffi::c_int;
        }
        sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio) - 1
    }
}

/// `sign_define()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_define(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        if (*argvars).v_type == VAR_LIST && (*argvars.offset(1)).v_type == VAR_UNKNOWN {
            let retlist = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
            for li in list_items((*argvars).vval.v_list) {
                let tv = &raw mut (*li).li_tv;
                let retval = if (*tv).v_type == VAR_DICT {
                    sign_define_from_dict(::core::ptr::null_mut(), (*tv).vval.v_dict)
                } else {
                    emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                    -1
                };
                tv_list_append_number(retlist, retval as varnumber_T);
            }
            return;
        }

        (*rettv).vval.v_number = -1;
        let name = tv_get_string_chk(argvars) as *mut ::core::ffi::c_char;
        if name.is_null() || tv_check_for_opt_dict_arg(argvars, 1) == FAIL {
            return;
        }
        let d = if (*argvars.offset(1)).v_type == VAR_DICT {
            (*argvars.offset(1)).vval.v_dict
        } else {
            ::core::ptr::null_mut()
        };
        (*rettv).vval.v_number = sign_define_from_dict(name, d) as varnumber_T;
    }
}

/// `sign_getdefined()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_getdefined(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let l = tv_list_alloc_ret(rettv, 0);
        if (*argvars).v_type == VAR_UNKNOWN {
            for sp in sign_defs() {
                tv_list_append_dict(l, sign_get_info_dict(sp));
            }
        } else {
            let sp = sign_find(tv_get_string(argvars));
            if !sp.is_null() {
                tv_list_append_dict(l, sign_get_info_dict(sp));
            }
        }
    }
}

/// `sign_getplaced()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_getplaced(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let mut buf = ::core::ptr::null_mut();
        let mut lnum: linenr_T = 0;
        let mut sign_id = 0;
        let mut group: *const ::core::ffi::c_char = ::core::ptr::null();

        let l = tv_list_alloc_ret(rettv, 0);

        if (*argvars).v_type != VAR_UNKNOWN {
            buf = get_buf_arg(argvars);
            if buf.is_null() {
                return;
            }
            if (*argvars.offset(1)).v_type != VAR_UNKNOWN {
                if tv_check_for_nonnull_dict_arg(argvars, 1) == FAIL {
                    return;
                }
                let dict = (*argvars.offset(1)).vval.v_dict;

                let di = find(dict, "lnum");
                if !di.is_null() {
                    lnum = tv_get_lnum(&raw mut (*di).di_tv);
                    if lnum <= 0 {
                        return;
                    }
                }
                let di = find(dict, "id");
                if !di.is_null() {
                    let mut notanum = false;
                    sign_id = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum)
                        as ::core::ffi::c_int;
                    if notanum {
                        return;
                    }
                }
                let di = find(dict, "group");
                if !di.is_null() {
                    group = tv_get_string_chk(&raw mut (*di).di_tv);
                    if group.is_null() {
                        return;
                    }
                    if *group == 0 {
                        // The empty string means the global group.
                        group = ::core::ptr::null();
                    }
                }
            }
        }

        sign_get_placed(buf, lnum, sign_id, group, l);
    }
}

/// `sign_jump()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_jump(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        (*rettv).vval.v_number = -1;

        let mut notanum = false;
        let id = tv_get_number_chk(argvars, &raw mut notanum) as ::core::ffi::c_int;
        if notanum {
            return;
        }
        if id <= 0 {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }

        let mut group = tv_get_string_chk(argvars.offset(1)) as *mut ::core::ffi::c_char;
        if group.is_null() {
            return;
        }
        if *group == 0 {
            group = ::core::ptr::null_mut();
        }

        let buf = get_buf_arg(argvars.offset(2));
        if buf.is_null() {
            return;
        }

        (*rettv).vval.v_number = sign_jump(id, group, buf) as varnumber_T;
    }
}

/// The named key's value, or the positional typval when there is one.
///
/// # Safety
/// `tv` and `dict` must be null or live.
unsafe fn slot(tv: *mut typval_T, dict: *mut dict_T, key: &str) -> *mut typval_T {
    // SAFETY: the caller's typval and dictionary.
    unsafe {
        if !tv.is_null() {
            return tv;
        }
        let di = find(dict, key);
        if di.is_null() {
            ::core::ptr::null_mut()
        } else {
            &raw mut (*di).di_tv
        }
    }
}

/// Places one sign described by a dictionary; answers its id, or −1.
///
/// The four `*_tv` arguments are `sign_place()`'s positional ones and are
/// null for `sign_placelist()`, where the dictionary carries them instead.
///
/// # Safety
/// The typvals and `dict` must be null or live.
unsafe fn sign_place_from_dict(
    id_tv: *mut typval_T,
    group_tv: *mut typval_T,
    name_tv: *mut typval_T,
    buf_tv: *mut typval_T,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's typvals and dictionary.
    unsafe {
        let mut notanum = false;

        let mut id = 0;
        let id_tv = slot(id_tv, dict, "id");
        if !id_tv.is_null() {
            id = tv_get_number_chk(id_tv, &raw mut notanum) as ::core::ffi::c_int;
            if notanum {
                return -1;
            }
            if id < 0 {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return -1;
            }
        }

        let mut group: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
        let group_tv = slot(group_tv, dict, "group");
        if !group_tv.is_null() {
            group = tv_get_string_chk(group_tv) as *mut ::core::ffi::c_char;
            if group.is_null() {
                return -1;
            }
            if *group == 0 {
                group = ::core::ptr::null_mut();
            }
        }

        let name_tv = slot(name_tv, dict, "name");
        if name_tv.is_null() {
            return -1;
        }
        let name = tv_get_string_chk(name_tv) as *mut ::core::ffi::c_char;
        if name.is_null() {
            return -1;
        }

        let buf_tv = slot(buf_tv, dict, "buffer");
        if buf_tv.is_null() {
            return -1;
        }
        let buf = get_buf_arg(buf_tv);
        if buf.is_null() {
            return -1;
        }

        let mut lnum: linenr_T = 0;
        let di = find(dict, "lnum");
        if !di.is_null() {
            lnum = tv_get_lnum(&raw mut (*di).di_tv);
            if lnum <= 0 {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return -1;
            }
        }

        let mut prio = -1;
        let di = find(dict, "priority");
        if !di.is_null() {
            prio = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum) as ::core::ffi::c_int;
            if notanum {
                return -1;
            }
        }

        // `sign_place` writes the id back when it was zero (auto-allocate).
        let mut uid = id as uint32_t;
        if sign_place(&raw mut uid, group, name, buf, lnum, prio) == OK {
            uid as ::core::ffi::c_int
        } else {
            -1
        }
    }
}

/// `sign_place()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_place(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let mut dict = ::core::ptr::null_mut();
        (*rettv).vval.v_number = -1;
        if (*argvars.offset(4)).v_type != VAR_UNKNOWN {
            if tv_check_for_nonnull_dict_arg(argvars, 4) == FAIL {
                return;
            }
            dict = (*argvars.offset(4)).vval.v_dict;
        }
        (*rettv).vval.v_number = sign_place_from_dict(
            argvars,
            argvars.offset(1),
            argvars.offset(2),
            argvars.offset(3),
            dict,
        ) as varnumber_T;
    }
}

/// `sign_placelist()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_placelist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let retlist = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
        if (*argvars).v_type != VAR_LIST {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        for li in list_items((*argvars).vval.v_list) {
            let tv = &raw mut (*li).li_tv;
            let sign_id = if (*tv).v_type == VAR_DICT {
                let null = ::core::ptr::null_mut();
                sign_place_from_dict(null, null, null, null, (*tv).vval.v_dict)
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                -1
            };
            tv_list_append_number(retlist, sign_id as varnumber_T);
        }
    }
}

/// `sign_undefine()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_undefine(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        if (*argvars).v_type == VAR_LIST && (*argvars.offset(1)).v_type == VAR_UNKNOWN {
            let retlist = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
            for li in list_items((*argvars).vval.v_list) {
                let name = tv_get_string_chk(&raw mut (*li).li_tv);
                let ok = !name.is_null() && sign_undefine_by_name(name) == OK;
                tv_list_append_number(retlist, if ok { 0 } else { -1 });
            }
            return;
        }

        (*rettv).vval.v_number = -1;
        if (*argvars).v_type == VAR_UNKNOWN {
            free_signs();
            (*rettv).vval.v_number = 0;
            return;
        }
        let name = tv_get_string_chk(argvars);
        if name.is_null() {
            return;
        }
        if sign_undefine_by_name(name) == OK {
            (*rettv).vval.v_number = 0;
        }
    }
}

/// Removes the signs a dictionary describes; 0 on success, −1 on failure.
///
/// `group_tv` is `sign_unplace()`'s positional group and is null for
/// `sign_unplacelist()`, where the dictionary carries it.
///
/// # Safety
/// The typval and `dict` must be null or live.
unsafe fn sign_unplace_from_dict(group_tv: *mut typval_T, dict: *mut dict_T) -> ::core::ffi::c_int {
    // SAFETY: the caller's typval and dictionary.
    unsafe {
        let mut id = 0;
        let mut buf = ::core::ptr::null_mut();
        let mut group = if !group_tv.is_null() {
            tv_get_string(group_tv) as *mut ::core::ffi::c_char
        } else {
            tv_dict_get_string(dict, c"group".as_ptr(), false)
        };
        if !group.is_null() && *group == 0 {
            group = ::core::ptr::null_mut();
        }

        if !dict.is_null() {
            let di = find(dict, "buffer");
            if !di.is_null() {
                buf = get_buf_arg(&raw mut (*di).di_tv);
                if buf.is_null() {
                    return -1;
                }
            }
            if !find(dict, "id").is_null() {
                id = tv_dict_get_number(dict, c"id".as_ptr()) as ::core::ffi::c_int;
                if id <= 0 {
                    emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                    return -1;
                }
            }
        }

        sign_unplace(buf, id, group, 0) - 1
    }
}

/// `sign_unplace()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_unplace(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        (*rettv).vval.v_number = -1;
        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_opt_dict_arg(argvars, 1) == FAIL
        {
            return;
        }
        let dict = if (*argvars.offset(1)).v_type != VAR_UNKNOWN {
            (*argvars.offset(1)).vval.v_dict
        } else {
            ::core::ptr::null_mut()
        };
        (*rettv).vval.v_number = sign_unplace_from_dict(argvars, dict) as varnumber_T;
    }
}

/// `sign_unplacelist()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe extern "C" fn f_sign_unplacelist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let retlist = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
        if (*argvars).v_type != VAR_LIST {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        for li in list_items((*argvars).vval.v_list) {
            let tv = &raw mut (*li).li_tv;
            let retval = if (*tv).v_type == VAR_DICT {
                sign_unplace_from_dict(::core::ptr::null_mut(), (*tv).vval.v_dict)
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                -1
            };
            tv_list_append_number(retlist, retval as varnumber_T);
        }
    }
}
