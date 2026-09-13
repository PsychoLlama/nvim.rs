//! The `sign_*()` Vimscript functions.
//!
//! The same operations the `:sign` command performs, addressed by
//! dictionary rather than by command line, plus the two report functions
//! (`sign_getdefined()`, `sign_getplaced()`) that answer with dictionaries
//! of their own. The `*_from_dict` helpers are shared between the single
//! and the `*list()` bulk forms.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::{DictRef, ListRef, NumBuf, list_items};
use crate::narrow::number_as_int;
use crate::types::{VAR_DICT, VAR_LIST, kListLenMayKnow};
use core::ptr;

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
unsafe fn put_str(d: *mut Dict, key: &str, val: *const ::core::ffi::c_char) {
    // SAFETY: the caller's dictionary and value.
    let _ = unsafe { tv_dict_add_str(d, key.as_ptr().cast(), key.len(), val) };
}

/// `tv_dict_add_nr` with a Rust key; see [`put_str`].
///
/// # Safety
/// `d` must be a live dictionary.
unsafe fn put_nr(d: *mut Dict, key: &str, nr: VarNumber) {
    // SAFETY: the caller's dictionary.
    let _ = unsafe { tv_dict_add_nr(d, key.as_ptr().cast(), key.len(), nr) };
}

/// `NULL`, for the many optional pointers in this file.
fn null<T>() -> *mut T {
    ::core::ptr::null_mut()
}

/// The value stored under `key`, or `None` when the dictionary has no such
/// key.
///
/// # Safety
/// `d` must be null or a live dictionary; the answer borrows from it.
unsafe fn key<'a>(d: *const Dict, key: &str) -> Option<&'a TypVal> {
    // SAFETY: the caller's dictionary.
    let di: *mut DictItem = unsafe {
        tv_dict_find(
            d,
            key.as_ptr().cast(),
            ptrdiff_t::try_from(key.len()).expect("a key literal is short"),
        )
    };
    // SAFETY: a non-null answer is a live item of that dictionary. No read
    // happens here.
    (!di.is_null()).then(|| unsafe { &(*di).di_tv })
}

/// Argument `i` as a dictionary, or null when it was not supplied.
///
/// # Safety
/// The caller must already have checked that a supplied argument `i` is a
/// dictionary -- `tv_check_for_*_dict_arg` is what does that.
unsafe fn dict_arg(args: &[TypVal], i: usize) -> *mut Dict {
    if args.len() <= i {
        return null();
    }
    args[i].dict_or_null()
}

/// A `group` argument: `None` when it does not read as a string at all, and
/// null for the empty string, which names the global group.
///
/// # Safety
/// `tv` must be a live typval.
unsafe fn group_arg(tv: &TypVal, numbuf: &mut NumBuf) -> Option<*mut c_char> {
    // SAFETY: the caller's typval.
    let group = numbuf.string_ptr_chk(tv).cast_mut();
    if group.is_null() {
        return None;
    }
    // SAFETY: a non-null answer is a NUL-terminated string.
    Some(if unsafe { *group } == 0 {
        null()
    } else {
        group
    })
}

/// The name of highlight group `id`, or `"NONE"` when it has none.
///
/// # Safety
/// None beyond `get_highlight_name_ext`'s.
unsafe fn hl_name(id: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    // SAFETY: the null `Expand` is the "no completion context" argument.
    let p = unsafe { get_highlight_name_ext(::core::ptr::null_mut(), id - 1, false) };
    if p.is_null() { c"NONE".as_ptr() } else { p }
}

/// Walks a `List`, yielding each item's value in order.
///
/// # Safety
/// `l` must be null or a live list the body does not modify.
unsafe fn item_values<'a>(l: *const List) -> impl Iterator<Item = *mut TypVal> + 'a {
    // SAFETY: the caller's list, which the body does not modify.
    list_items(unsafe { l.as_ref() })
        .iter()
        .map(|li| ::core::ptr::from_ref(&li.li_tv).cast_mut())
}

/// Runs `one` over every dictionary in `l`, appending what it answers to
/// `retlist`. An entry that is not a dictionary is E715 and answers -1.
///
/// # Safety
/// `l` and `retlist` must be live lists.
unsafe fn each_dict(retlist: *mut List, l: *const List, mut one: impl FnMut(*mut Dict) -> c_int) {
    // SAFETY: the caller's lists.
    unsafe {
        for tv in item_values(l) {
            let retval = if (*tv).v_type() == VAR_DICT {
                one((*tv).dict_or_null())
            } else {
                emsg(gettext(e_dictreq));
                -1
            };
            (*retlist).push_number(VarNumber::from(retval));
        }
    };
}

/// The body `sign_placelist()` and `sign_unplacelist()` share: a list of
/// what `one` answered for each dictionary in the argument list.
///
/// The return list is allocated *before* the type check, so a non-list
/// argument still answers `[]` and not `0`.
///
/// # Safety
/// `args` and `result` are the frame's.
unsafe fn each_dict_arg(args: &[TypVal], result: &mut TypVal, one: impl FnMut(*mut Dict) -> c_int) {
    // SAFETY: the frame's return slot.
    let retlist = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    if !args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
        emsg(gettext(e_listreq));
        return;
    }
    // SAFETY: the tag says the list arm is live, and `retlist` was just
    // allocated.
    unsafe { each_dict(retlist, args[0].list_or_null(), one) };
}

/// `sign_getdefined()`'s dictionary for one defined sign.
pub(crate) fn sign_get_info_dict(sign: SignRef) -> DictRef {
    // SAFETY: a definition's name, icon and cells are its own.
    let d_held = tv_dict_alloc();
    let d = d_held.as_ptr();
    unsafe { put_str(d, "name", sign.sn_name) };
    if !sign.sn_icon.is_null() {
        unsafe { put_str(d, "icon", sign.sn_icon) };
    }
    if sign.sn_text[0] != 0 {
        let mut buf = [0 as ::core::ffi::c_char; SIGN_TEXT_BUF];
        unsafe { describe_sign_text(buf.as_mut_ptr(), sign.cells()) };
        unsafe { put_str(d, "text", buf.as_ptr()) };
    }
    if sign.sn_priority > 0 {
        unsafe { put_nr(d, "priority", VarNumber::from(sign.sn_priority)) };
    }
    let ids = [
        sign.sn_line_hl,
        sign.sn_text_hl,
        sign.sn_cul_hl,
        sign.sn_num_hl,
    ];
    for (key, id) in HL_KEYS.iter().zip(ids) {
        if id > 0 {
            unsafe { put_str(d, key, hl_name(id)) };
        }
    }
    d_held
}

/// `sign_getplaced()`'s dictionary for one placed sign.
pub(crate) fn sign_get_placed_info_dict(mark: MTKey) -> DictRef {
    // SAFETY: the caller's mark, and the decoration the store names for it.
    let d_held = tv_dict_alloc();
    let d = d_held.as_ptr();
    let sh = unsafe { Sh::new(decor_find_sign(mt_decor(mark))) };
    unsafe { put_str(d, "name", sign_get_name(sh.raw())) };
    unsafe { put_nr(d, "id", VarNumber::from(mark.id.cast_signed())) };
    unsafe { put_str(d, "group", describe_ns(mark.ns.cast_signed(), c"".as_ptr())) };
    unsafe { put_nr(d, "lnum", VarNumber::from(mark.pos.row + 1)) };
    unsafe { put_nr(d, "priority", VarNumber::from(sh.priority)) };
    d_held
}

/// Every sign placed in `buffer`, in marktree order — `getbufinfo()`'s `signs`.
pub(crate) fn get_buffer_signs(buffer: Buf) -> ListRef {
    let signs = placed_signs(buffer, 0, ALL_GROUPS, |_| Keep::Yes);
    let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    for mark in signs {
        unsafe { (*l.as_ptr()).push_dict(Some(sign_get_placed_info_dict(mark))) };
    }
    l
}

/// Appends `buffer`'s `{ bufnr, signs }` entry to `retlist`, filtered by `lnum`,
/// `sign_id` and `group`.
///
/// A zero `lnum` or `sign_id` means "any"; the two combine, so naming both
/// asks for one specific sign on one specific line.
///
/// # Safety
/// `buffer` and `retlist` must be live; `group` must be null or NUL-terminated.
unsafe fn sign_get_placed_in_buf(
    buffer: Buf,
    lnum: LineNr,
    sign_id: ::core::ffi::c_int,
    group: *const ::core::ffi::c_char,
    retlist: *mut List,
) {
    // SAFETY: the caller's buffer.
    let cbuf = buffer;
    let d_held = tv_dict_alloc();
    let d = d_held.as_ptr();
    // SAFETY: the caller's list, and the buffer handle it reports.
    let l = unsafe {
        (*retlist).push_dict(Some(d_held));
        put_nr(d, "bufnr", VarNumber::from(cbuf.handle));
        let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        // A borrow of the list the dictionary owns from here on.
        let into = l.as_ptr();
        let _ = tv_dict_add_list(d, "signs".as_ptr().cast(), "signs".len(), Some(l));
        into
    };

    // SAFETY: the caller's buffer and group name.
    let ns = unsafe { group_get_ns(group) };
    if !buf_has_signs(buffer) || ns < 0 {
        return;
    }

    let first_row = if lnum != 0 { lnum - 1 } else { 0 };
    let mut signs = placed_signs(cbuf, first_row, ns, |mark| {
        if lnum != 0 && mark.pos.row >= lnum {
            // The tree is in row order, so nothing past this row can match.
            return Keep::Stop;
        }
        // A zero `lnum` or `sign_id` means "any"; the two combine.
        let on_line = lnum == 0 || lnum == mark.pos.row + 1;
        let is_id = sign_id == 0 || sign_id == mark.id.cast_signed();
        if on_line && is_id {
            Keep::Yes
        } else {
            Keep::No
        }
    });

    // SAFETY: every mark the walk kept carries a live sign decoration.
    sort_signs(&mut signs);
    for mark in signs {
        unsafe { (*l).push_dict(Some(sign_get_placed_info_dict(mark))) };
    }
}

/// Appends the placed-sign report for `buffer`, or for every buffer that has
/// signs when `buffer` is null.
///
/// # Safety
/// `buffer` must be null or live; `retlist` must be live.
unsafe fn sign_get_placed(
    buffer: Option<Buf>,
    lnum: LineNr,
    id: ::core::ffi::c_int,
    group: *const ::core::ffi::c_char,
    retlist: *mut List,
) {
    if let Some(buffer) = buffer {
        // SAFETY: the caller's buffer and list.
        unsafe { sign_get_placed_in_buf(buffer, lnum, id, group, retlist) };
        return;
    }
    for cbuf in buffers() {
        // SAFETY: a live buffer from the editor's own list, and the caller's
        // list.
        if buf_has_signs(cbuf) {
            // `lnum` is deliberately dropped: an all-buffers query
            // reports every line whatever line was asked for.
            unsafe { sign_get_placed_in_buf(cbuf, 0, id, group, retlist) };
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
    dict: *mut Dict,
) -> ::core::ffi::c_int {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let mut numbuf5 = NumBuf::new();
    let mut numbuf6 = NumBuf::new();
    let mut numbuf7 = NumBuf::new();
    // SAFETY: the caller's name and dictionary.
    let mut name = name;
    if name.is_null() {
        name = unsafe { numbuf.dict_string(dict, c"name".as_ptr()) }.cast_mut();
        if name.is_null() || unsafe { *name } == 0 {
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
        icon = unsafe { numbuf2.dict_string(dict, c"icon".as_ptr()) }.cast_mut();
        linehl = unsafe { numbuf3.dict_string(dict, c"linehl".as_ptr()) }.cast_mut();
        text = unsafe { numbuf4.dict_string(dict, c"text".as_ptr()) }.cast_mut();
        texthl = unsafe { numbuf5.dict_string(dict, c"texthl".as_ptr()) }.cast_mut();
        culhl = unsafe { numbuf6.dict_string(dict, c"culhl".as_ptr()) }.cast_mut();
        numhl = unsafe { numbuf7.dict_string(dict, c"numhl".as_ptr()) }.cast_mut();
        prio = number_as_int(unsafe { tv_dict_get_number_def(dict, c"priority".as_ptr(), -1) });
    }
    let defined =
        unsafe { sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio) };
    if defined.is_ok() { 0 } else { -1 }
}

/// `sign_define()`.
pub(crate) fn f_sign_define(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if args[0].v_type() == VAR_LIST && args.len() <= 1 {
        // SAFETY: the frame's return slot, and a list the evaluator owns.
        unsafe {
            let retlist = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
            each_dict(retlist, args[0].list_or_null(), |d| {
                sign_define_from_dict(null(), d)
            });
        };
        return;
    }

    result.write_number(-1);
    // SAFETY: the argument slots the frame named.
    let name = numbuf.string_ptr_chk(&args[0]).cast_mut();
    // SAFETY: as above.
    if name.is_null() || tv_check_for_opt_dict_arg(args, 1).is_err() {
        return;
    }
    // SAFETY: the tag says the dictionary arm is live.
    let d = unsafe { dict_arg(args, 1) };
    // SAFETY: the name and dictionary just read out of the frame.
    result.write_number(VarNumber::from(unsafe { sign_define_from_dict(name, d) }));
}

/// `sign_getdefined()`.
pub(crate) fn f_sign_getdefined(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the frame's return slot and argument.
    unsafe {
        let l = tv_list_alloc_ret(result, 0);
        let defs = if !args.is_empty() {
            sign_find(numbuf.string_ptr(&args[0])).into_iter().collect()
        } else {
            sign_defs()
        };
        for sp in defs {
            (*l).push_dict(Some(sign_get_info_dict(sp)));
        }
    };
}

/// `sign_getplaced()`.
pub(crate) fn f_sign_getplaced(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the frame's return slot and argument slots.
    unsafe {
        let mut buf = None;
        let mut lnum: LineNr = 0;
        let mut sign_id = 0;
        let mut group: *const ::core::ffi::c_char = ::core::ptr::null();

        let l = tv_list_alloc_ret(result, 0);

        if !args.is_empty() {
            buf = get_buf_arg(&args[0]);
            if buf.is_none() {
                return;
            }
            if args.len() > 1 {
                if tv_check_for_nonnull_dict_arg(args, 1).is_err() {
                    return;
                }
                let dict = args[1].dict_or_null();

                if let Some(tv) = key(dict, "lnum") {
                    lnum = tv_get_lnum(tv);
                    if lnum <= 0 {
                        return;
                    }
                }
                if let Some(tv) = key(dict, "id") {
                    let Ok(given) = tv_get_number_chk(tv) else {
                        return;
                    };
                    sign_id = number_as_int(given);
                }
                if let Some(tv) = key(dict, "group") {
                    group = numbuf.string_ptr_chk(tv);
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
    };
}

/// `sign_jump()`.
pub(crate) fn f_sign_jump(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    let Ok(given) = tv_get_number_chk(&args[0]) else {
        return;
    };
    let id = number_as_int(given);
    if id <= 0 {
        emsg(gettext(e_invarg));
        return;
    }

    // SAFETY: the frame's argument slots.
    let Some(group) = (unsafe { group_arg(&args[1], &mut numbuf) }) else {
        return;
    };
    let buf = get_buf_arg(&args[2]);
    if buf.is_none() {
        return;
    }

    // SAFETY: a live buffer and a group name the argument owns.
    let jumped = unsafe { sign_jump(id, group, buf.expect("a live handle")) };
    result.write_number(VarNumber::from(jumped));
}

/// The named key's value, or the positional typval when there is one.
///
/// # Safety
/// `tv` and `dict` must be null or live.
unsafe fn slot<'a>(tv: Option<&'a TypVal>, dict: *mut Dict, name: &str) -> Option<&'a TypVal> {
    if tv.is_some() {
        return tv;
    }
    // SAFETY: the caller's dictionary.
    unsafe { key(dict, name) }
}

/// Places one sign described by a dictionary; answers its id, or −1.
///
/// The four `*_tv` arguments are `sign_place()`'s positional ones and are
/// null for `sign_placelist()`, where the dictionary carries them instead.
///
/// # Safety
/// The typvals and `dict` must be null or live.
unsafe fn sign_place_from_dict(
    id_tv: Option<&TypVal>,
    group_tv: Option<&TypVal>,
    name_tv: Option<&TypVal>,
    buf_tv: Option<&TypVal>,
    dict: *mut Dict,
) -> ::core::ffi::c_int {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's typvals and dictionary.
    let mut id = 0;
    if let Some(tv) = unsafe { slot(id_tv, dict, "id") } {
        let Ok(given) = tv_get_number_chk(tv) else {
            return -1;
        };
        id = number_as_int(given);
        if id < 0 {
            emsg(gettext(e_invarg));
            return -1;
        }
    }

    let mut group: *mut c_char = null();
    if let Some(tv) = unsafe { slot(group_tv, dict, "group") } {
        match unsafe { group_arg(tv, &mut numbuf) } {
            Some(named) => group = named,
            None => return -1,
        }
    }

    let __v = unsafe { slot(name_tv, dict, "name") };

    let Some(name_tv) = __v else {
        return -1;
    };
    let name = numbuf.string_ptr_chk(name_tv).cast_mut();
    if name.is_null() {
        return -1;
    }

    let __v = unsafe { slot(buf_tv, dict, "buffer") };

    let Some(buf_tv) = __v else {
        return -1;
    };
    let buf = get_buf_arg(buf_tv);
    if buf.is_none() {
        return -1;
    }

    let mut lnum: LineNr = 0;
    if let Some(tv) = unsafe { key(dict, "lnum") } {
        lnum = tv_get_lnum(tv);
        if lnum <= 0 {
            emsg(gettext(e_invarg));
            return -1;
        }
    }

    let mut prio = -1;
    if let Some(tv) = unsafe { key(dict, "priority") } {
        let Ok(given) = tv_get_number_chk(tv) else {
            return -1;
        };
        prio = number_as_int(given);
    }

    // `sign_place` writes the id back when it was zero (auto-allocate).
    let mut uid = id.cast_unsigned();
    if unsafe {
        sign_place(
            &raw mut uid,
            group,
            name,
            buf.expect("a live handle"),
            lnum,
            prio,
        )
    }
    .is_ok()
    {
        uid.cast_signed()
    } else {
        -1
    }
}

/// `sign_place()`.
pub(crate) fn f_sign_place(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    let mut dict = null();
    if args.len() > 4 {
        // SAFETY: the frame's argument slots.
        if tv_check_for_nonnull_dict_arg(args, 4).is_err() {
            return;
        }
        dict = args[4].dict_or_null();
    }
    // SAFETY: the frame's argument slots and the dictionary just read.
    let (id_tv, group_tv) = (Some(&args[0]), Some(&args[1]));
    let (name_tv, buf_tv) = (Some(&args[2]), Some(&args[3]));
    let id = unsafe { sign_place_from_dict(id_tv, group_tv, name_tv, buf_tv, dict) };
    result.write_number(VarNumber::from(id));
}

/// `sign_placelist()`.
pub(crate) fn f_sign_placelist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the frame's return slot and argument.
    unsafe {
        each_dict_arg(args, result, |d| {
            sign_place_from_dict(None, None, None, None, d)
        });
    };
}

/// `sign_undefine()`.
pub(crate) fn f_sign_undefine(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    if args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) && args.len() <= 1 {
        // SAFETY: the frame's return slot, and a list the evaluator owns.
        unsafe {
            let retlist = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
            for tv in item_values(args[0].list_or_null()) {
                let name = numbuf.string_ptr_chk(&*tv);
                let ok = !name.is_null() && sign_undefine_by_name(name).is_ok();
                (*retlist).push_number(if ok { 0 } else { -1 });
            }
        };
        return;
    }

    result.write_number(-1);
    if args.is_empty() {
        free_signs();
        result.write_number(0);
        return;
    }
    // SAFETY: the frame's argument slot.
    let name = numbuf2.string_ptr_chk(&args[0]);
    // SAFETY: a name the argument owns, NUL-terminated.
    if !name.is_null() && unsafe { sign_undefine_by_name(name) }.is_ok() {
        result.write_number(0);
    }
}

/// Removes the signs a dictionary describes; 0 on success, −1 on failure.
///
/// `group_tv` is `sign_unplace()`'s positional group and is null for
/// `sign_unplacelist()`, where the dictionary carries it.
///
/// # Safety
/// The typval and `dict` must be null or live.
unsafe fn sign_unplace_from_dict(group_tv: Option<&TypVal>, dict: *mut Dict) -> ::core::ffi::c_int {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY: the caller's typval and dictionary.
    let mut id = 0;
    let mut buf = ::core::ptr::null_mut();
    let mut group = match group_tv {
        Some(tv) => numbuf.string_ptr(tv),
        None => unsafe { numbuf2.dict_string(dict, c"group".as_ptr()) },
    };
    if !group.is_null() && unsafe { *group } == 0 {
        group = ::core::ptr::null();
    }

    if !dict.is_null() {
        if let Some(tv) = unsafe { key(dict, "buffer") } {
            buf = get_buf_arg(tv).map_or(ptr::null_mut(), Buf::raw);
            if buf.is_null() {
                return -1;
            }
        }
        if unsafe { key(dict, "id") }.is_some() {
            id = number_as_int(unsafe { tv_dict_get_number(dict, c"id".as_ptr()) });
            if id <= 0 {
                emsg(gettext(e_invarg));
                return -1;
            }
        }
    }

    unsafe { sign_unplace(Buf::from_raw(buf), id, group, 0) - 1 }
}

/// `sign_unplace()`.
pub(crate) fn f_sign_unplace(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    // SAFETY: the frame's argument slots.
    if tv_check_for_string_arg(args, 0).is_err() || tv_check_for_opt_dict_arg(args, 1).is_err() {
        return;
    }
    // SAFETY: the check above says the dictionary arm is live if it is set.
    let dict = unsafe { dict_arg(args, 1) };
    // SAFETY: the frame's first argument and the dictionary just read.
    let unplaced = unsafe { sign_unplace_from_dict(Some(&args[0]), dict) };
    result.write_number(VarNumber::from(unplaced));
}

/// `sign_unplacelist()`.
pub(crate) fn f_sign_unplacelist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the frame's return slot and argument.
    unsafe { each_dict_arg(args, result, |d| sign_unplace_from_dict(None, d)) };
}
