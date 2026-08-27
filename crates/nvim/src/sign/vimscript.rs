//! The `sign_*()` Vimscript functions.
//!
//! The same operations the `:sign` command performs, addressed by
//! dictionary rather than by command line, plus the two report functions
//! (`sign_getdefined()`, `sign_getplaced()`) that answer with dictionaries
//! of their own. The `*_from_dict` helpers are shared between the single
//! and the `*list()` bulk forms.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::funcs::args::{Args, frame};
use crate::eval::typval::NumBuf;
use crate::narrow::number_as_int;
use crate::types::{FAIL, OK, VAR_DICT, VAR_LIST, kListLenMayKnow};

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

/// `NULL`, for the many optional pointers in this file.
fn null<T>() -> *mut T {
    ::core::ptr::null_mut()
}

/// The value stored under `key`, or `None` when the dictionary has no such
/// key.
///
/// # Safety
/// `d` must be null or a live dictionary; the answer borrows from it.
unsafe fn key(d: *const dict_T, key: &str) -> Option<*mut typval_T> {
    // SAFETY: the caller's dictionary.
    let di: *mut dictitem_T = unsafe {
        tv_dict_find(
            d,
            key.as_ptr().cast(),
            ptrdiff_t::try_from(key.len()).expect("a key literal is short"),
        )
    };
    // SAFETY: a non-null answer is a live item of that dictionary. No read
    // happens here.
    (!di.is_null()).then(|| unsafe { &raw mut (*di).di_tv })
}

/// Argument `i` as a dictionary, or null when it was not supplied.
///
/// # Safety
/// The caller must already have checked that a supplied argument `i` is a
/// dictionary -- `tv_check_for_*_dict_arg` is what does that.
unsafe fn dict_arg(args: Args<'_>, i: usize) -> *mut dict_T {
    if !args.has(i) {
        return null();
    }
    // SAFETY: the caller's check says the dictionary arm is live.
    unsafe { args.get(i).vval.v_dict }
}

/// A `group` argument: `None` when it does not read as a string at all, and
/// null for the empty string, which names the global group.
///
/// # Safety
/// `tv` must be a live typval.
unsafe fn group_arg(tv: *mut typval_T, numbuf: &mut NumBuf) -> Option<*mut c_char> {
    // SAFETY: the caller's typval.
    let group = unsafe { numbuf.string_chk(tv) }.cast_mut();
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
    // SAFETY: the null `expand_T` is the "no completion context" argument.
    unsafe {
        let p = get_highlight_name_ext(::core::ptr::null_mut(), id - 1, false);
        if p.is_null() { c"NONE".as_ptr() } else { p }
    }
}

/// Walks a `list_T`, yielding each item's value in order.
///
/// # Safety
/// `l` must be null or a live list the body does not modify.
unsafe fn list_items(l: *const list_T) -> impl Iterator<Item = *mut typval_T> {
    // SAFETY: the caller's list.
    let mut at = unsafe { tv_list_first(l) };
    ::core::iter::from_fn(move || {
        if at.is_null() {
            return None;
        }
        let item = at;
        // SAFETY: `item` is a live element of the caller's list.
        at = unsafe { (*item).li_next };
        // SAFETY: as above. No read happens here.
        Some(unsafe { &raw mut (*item).li_tv })
    })
}

/// Runs `one` over every dictionary in `l`, appending what it answers to
/// `retlist`. An entry that is not a dictionary is E715 and answers -1.
///
/// # Safety
/// `l` and `retlist` must be live lists.
unsafe fn each_dict(
    retlist: *mut list_T,
    l: *const list_T,
    mut one: impl FnMut(*mut dict_T) -> c_int,
) {
    // SAFETY: the caller's lists.
    unsafe {
        for tv in list_items(l) {
            let retval = if (*tv).v_type == VAR_DICT {
                one((*tv).vval.v_dict)
            } else {
                emsg(gettext((&raw const e_dictreq).cast::<c_char>()));
                -1
            };
            tv_list_append_number(retlist, varnumber_T::from(retval));
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
/// `args` and `rettv` are the frame's.
unsafe fn each_dict_arg(
    args: Args<'_>,
    rettv: &mut typval_T,
    one: impl FnMut(*mut dict_T) -> c_int,
) {
    // SAFETY: the frame's return slot.
    let retlist = unsafe { tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t) };
    if args.ty(0) != VAR_LIST {
        // SAFETY: a static message.
        unsafe { emsg(gettext((&raw const e_listreq).cast::<c_char>())) };
        return;
    }
    // SAFETY: the tag says the list arm is live, and `retlist` was just
    // allocated.
    unsafe { each_dict(retlist, args.get(0).vval.v_list, one) };
}

/// `sign_getdefined()`'s dictionary for one defined sign.
///
/// # Safety
/// `sp` must be a live sign definition.
pub(crate) unsafe fn sign_get_info_dict(sp: Sign) -> *mut dict_T {
    // SAFETY: a definition's name, icon and cells are its own.
    unsafe {
        let d = tv_dict_alloc();
        put_str(d, "name", sp.sn_name);
        if !sp.sn_icon.is_null() {
            put_str(d, "icon", sp.sn_icon);
        }
        if sp.sn_text[0] != 0 {
            let mut buf = [0 as ::core::ffi::c_char; SIGN_TEXT_BUF];
            describe_sign_text(buf.as_mut_ptr(), sp.cells());
            put_str(d, "text", buf.as_ptr());
        }
        if sp.sn_priority > 0 {
            put_nr(d, "priority", varnumber_T::from(sp.sn_priority));
        }
        let ids = [sp.sn_line_hl, sp.sn_text_hl, sp.sn_cul_hl, sp.sn_num_hl];
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
    // SAFETY: the caller's mark, and the decoration the store names for it.
    unsafe {
        let d = tv_dict_alloc();
        let sh = Sh::new(decor_find_sign(mt_decor(mark)));
        put_str(d, "name", sign_get_name(sh.raw()));
        put_nr(d, "id", varnumber_T::from(mark.id.cast_signed()));
        put_str(d, "group", describe_ns(mark.ns.cast_signed(), c"".as_ptr()));
        put_nr(d, "lnum", varnumber_T::from(mark.pos.row + 1));
        put_nr(d, "priority", varnumber_T::from(sh.priority));
        d
    }
}

/// Every sign placed in `buf`, in marktree order — `getbufinfo()`'s `signs`.
///
/// # Safety
/// `buf` must be live.
pub(crate) unsafe fn get_buffer_signs(buf: *mut buf_T) -> *mut list_T {
    // SAFETY: the caller's buffer.
    let signs = placed_signs(unsafe { Buf::new(buf) }, 0, ALL_GROUPS, |_| Keep::Yes);
    // SAFETY: every mark the walk kept carries a live sign decoration.
    unsafe {
        let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        for mark in signs {
            tv_list_append_dict(l, sign_get_placed_info_dict(mark));
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
    // SAFETY: the caller's buffer.
    let cbuf = unsafe { Buf::new(buf) };
    // SAFETY: the caller's list, and the buffer handle it reports.
    let l = unsafe {
        let d = tv_dict_alloc();
        tv_list_append_dict(retlist, d);
        put_nr(d, "bufnr", varnumber_T::from(cbuf.handle));
        let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        tv_dict_add_list(d, "signs".as_ptr().cast(), "signs".len(), l);
        l
    };

    // SAFETY: the caller's buffer and group name.
    let ns = unsafe { group_get_ns(group) };
    if !unsafe { buf_has_signs(buf) } || ns < 0 {
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
    unsafe {
        sort_signs(&mut signs);
        for mark in signs {
            tv_list_append_dict(l, sign_get_placed_info_dict(mark));
        }
    };
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
    if !buf.is_null() {
        // SAFETY: the caller's buffer and list.
        unsafe { sign_get_placed_in_buf(buf, lnum, id, group, retlist) };
        return;
    }
    for cbuf in buffers() {
        // SAFETY: a live buffer from the editor's own list, and the caller's
        // list.
        unsafe {
            if buf_has_signs(cbuf.raw()) {
                // `lnum` is deliberately dropped: an all-buffers query
                // reports every line whatever line was asked for.
                sign_get_placed_in_buf(cbuf.raw(), 0, id, group, retlist);
            }
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
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let mut numbuf5 = NumBuf::new();
    let mut numbuf6 = NumBuf::new();
    let mut numbuf7 = NumBuf::new();
    // SAFETY: the caller's name and dictionary.
    unsafe {
        let mut name = name;
        if name.is_null() {
            name = numbuf.dict_string(dict, c"name".as_ptr()).cast_mut();
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
            icon = numbuf2.dict_string(dict, c"icon".as_ptr()).cast_mut();
            linehl = numbuf3.dict_string(dict, c"linehl".as_ptr()).cast_mut();
            text = numbuf4.dict_string(dict, c"text".as_ptr()).cast_mut();
            texthl = numbuf5.dict_string(dict, c"texthl".as_ptr()).cast_mut();
            culhl = numbuf6.dict_string(dict, c"culhl".as_ptr()).cast_mut();
            numhl = numbuf7.dict_string(dict, c"numhl".as_ptr()).cast_mut();
            prio = number_as_int(tv_dict_get_number_def(dict, c"priority".as_ptr(), -1));
        }
        sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio) - 1
    }
}

/// `sign_define()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_define(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    if args.ty(0) == VAR_LIST && !args.has(1) {
        // SAFETY: the frame's return slot, and a list the evaluator owns.
        unsafe {
            let retlist = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
            each_dict(retlist, args.get(0).vval.v_list, |d| {
                sign_define_from_dict(null(), d)
            });
        };
        return;
    }

    rettv.vval.v_number = -1;
    // SAFETY: the argument slots the frame named.
    let name = unsafe { numbuf.string_chk(args.ptr(0)) }.cast_mut();
    // SAFETY: as above.
    if name.is_null() || unsafe { tv_check_for_opt_dict_arg(args.ptr(0), 1) } == FAIL {
        return;
    }
    // SAFETY: the tag says the dictionary arm is live.
    let d = unsafe { dict_arg(args, 1) };
    // SAFETY: the name and dictionary just read out of the frame.
    rettv.vval.v_number = varnumber_T::from(unsafe { sign_define_from_dict(name, d) });
}

/// `sign_getdefined()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_getdefined(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's return slot and argument.
    unsafe {
        let l = tv_list_alloc_ret(rettv, 0);
        let defs = if args.has(0) {
            sign_find(numbuf.string(args.ptr(0))).into_iter().collect()
        } else {
            sign_defs()
        };
        for sp in defs {
            tv_list_append_dict(l, sign_get_info_dict(sp));
        }
    };
}

/// `sign_getplaced()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_getplaced(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's return slot and argument slots.
    unsafe {
        let mut buf = null();
        let mut lnum: linenr_T = 0;
        let mut sign_id = 0;
        let mut group: *const ::core::ffi::c_char = ::core::ptr::null();

        let l = tv_list_alloc_ret(rettv, 0);

        if args.has(0) {
            buf = get_buf_arg(args.ptr(0));
            if buf.is_null() {
                return;
            }
            if args.has(1) {
                if tv_check_for_nonnull_dict_arg(args.ptr(0), 1) == FAIL {
                    return;
                }
                let dict = args.get(1).vval.v_dict;

                if let Some(tv) = key(dict, "lnum") {
                    lnum = tv_get_lnum(tv);
                    if lnum <= 0 {
                        return;
                    }
                }
                if let Some(tv) = key(dict, "id") {
                    let mut notanum = false;
                    sign_id = number_as_int(tv_get_number_chk(tv, &raw mut notanum));
                    if notanum {
                        return;
                    }
                }
                if let Some(tv) = key(dict, "group") {
                    group = numbuf.string_chk(tv);
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
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_jump(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;

    let mut notanum = false;
    // SAFETY: the frame's argument slots.
    let id = number_as_int(unsafe { tv_get_number_chk(args.ptr(0), &raw mut notanum) });
    if notanum {
        return;
    }
    if id <= 0 {
        // SAFETY: a static message.
        unsafe { emsg(gettext((&raw const e_invarg).cast::<c_char>())) };
        return;
    }

    // SAFETY: the frame's argument slots.
    let Some(group) = (unsafe { group_arg(args.ptr(1), &mut numbuf) }) else {
        return;
    };
    // SAFETY: as above.
    let buf = unsafe { get_buf_arg(args.ptr(2)) };
    if buf.is_null() {
        return;
    }

    // SAFETY: a live buffer and a group name the argument owns.
    rettv.vval.v_number = varnumber_T::from(unsafe { sign_jump(id, group, buf) });
}

/// The named key's value, or the positional typval when there is one.
///
/// # Safety
/// `tv` and `dict` must be null or live.
unsafe fn slot(tv: *mut typval_T, dict: *mut dict_T, name: &str) -> Option<*mut typval_T> {
    if !tv.is_null() {
        return Some(tv);
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
    id_tv: *mut typval_T,
    group_tv: *mut typval_T,
    name_tv: *mut typval_T,
    buf_tv: *mut typval_T,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's typvals and dictionary.
    unsafe {
        let mut notanum = false;

        let mut id = 0;
        if let Some(tv) = slot(id_tv, dict, "id") {
            id = number_as_int(tv_get_number_chk(tv, &raw mut notanum));
            if notanum {
                return -1;
            }
            if id < 0 {
                emsg(gettext((&raw const e_invarg).cast::<c_char>()));
                return -1;
            }
        }

        let mut group: *mut c_char = null();
        if let Some(tv) = slot(group_tv, dict, "group") {
            match group_arg(tv, &mut numbuf) {
                Some(named) => group = named,
                None => return -1,
            }
        }

        let Some(name_tv) = slot(name_tv, dict, "name") else {
            return -1;
        };
        let name = numbuf.string_chk(name_tv).cast_mut();
        if name.is_null() {
            return -1;
        }

        let Some(buf_tv) = slot(buf_tv, dict, "buffer") else {
            return -1;
        };
        let buf = get_buf_arg(buf_tv);
        if buf.is_null() {
            return -1;
        }

        let mut lnum: linenr_T = 0;
        if let Some(tv) = key(dict, "lnum") {
            lnum = tv_get_lnum(tv);
            if lnum <= 0 {
                emsg(gettext((&raw const e_invarg).cast::<c_char>()));
                return -1;
            }
        }

        let mut prio = -1;
        if let Some(tv) = key(dict, "priority") {
            prio = number_as_int(tv_get_number_chk(tv, &raw mut notanum));
            if notanum {
                return -1;
            }
        }

        // `sign_place` writes the id back when it was zero (auto-allocate).
        let mut uid = id.cast_unsigned();
        if sign_place(&raw mut uid, group, name, buf, lnum, prio) == OK {
            uid.cast_signed()
        } else {
            -1
        }
    }
}

/// `sign_place()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_place(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    let mut dict = null();
    if args.has(4) {
        // SAFETY: the frame's argument slots.
        if unsafe { tv_check_for_nonnull_dict_arg(args.ptr(0), 4) } == FAIL {
            return;
        }
        // SAFETY: the check above says the dictionary arm is live.
        dict = unsafe { args.get(4).vval.v_dict };
    }
    // SAFETY: the frame's argument slots and the dictionary just read.
    let id =
        unsafe { sign_place_from_dict(args.ptr(0), args.ptr(1), args.ptr(2), args.ptr(3), dict) };
    rettv.vval.v_number = varnumber_T::from(id);
}

/// `sign_placelist()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_placelist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's return slot and argument.
    unsafe {
        each_dict_arg(args, rettv, |d| {
            sign_place_from_dict(null(), null(), null(), null(), d)
        });
    };
}

/// `sign_undefine()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_undefine(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    if args.ty(0) == VAR_LIST && !args.has(1) {
        // SAFETY: the frame's return slot, and a list the evaluator owns.
        unsafe {
            let retlist = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
            for tv in list_items(args.get(0).vval.v_list) {
                let name = numbuf.string_chk(tv);
                let ok = !name.is_null() && sign_undefine_by_name(name) == OK;
                tv_list_append_number(retlist, if ok { 0 } else { -1 });
            }
        };
        return;
    }

    rettv.vval.v_number = -1;
    if !args.has(0) {
        free_signs();
        rettv.vval.v_number = 0;
        return;
    }
    // SAFETY: the frame's argument slot.
    let name = unsafe { numbuf2.string_chk(args.ptr(0)) };
    // SAFETY: a name the argument owns, NUL-terminated.
    if !name.is_null() && unsafe { sign_undefine_by_name(name) } == OK {
        rettv.vval.v_number = 0;
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
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY: the caller's typval and dictionary.
    unsafe {
        let mut id = 0;
        let mut buf = ::core::ptr::null_mut();
        let mut group = if !group_tv.is_null() {
            numbuf.string(group_tv)
        } else {
            numbuf2.dict_string(dict, c"group".as_ptr())
        };
        if !group.is_null() && *group == 0 {
            group = ::core::ptr::null();
        }

        if !dict.is_null() {
            if let Some(tv) = key(dict, "buffer") {
                buf = get_buf_arg(tv);
                if buf.is_null() {
                    return -1;
                }
            }
            if key(dict, "id").is_some() {
                id = number_as_int(tv_dict_get_number(dict, c"id".as_ptr()));
                if id <= 0 {
                    emsg(gettext((&raw const e_invarg).cast::<c_char>()));
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
pub(crate) unsafe fn f_sign_unplace(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the frame's argument slots.
    if unsafe { tv_check_for_string_arg(args.ptr(0), 0) } == FAIL
        || unsafe { tv_check_for_opt_dict_arg(args.ptr(0), 1) } == FAIL
    {
        return;
    }
    // SAFETY: the check above says the dictionary arm is live if it is set.
    let dict = unsafe { dict_arg(args, 1) };
    // SAFETY: the frame's first argument and the dictionary just read.
    rettv.vval.v_number = varnumber_T::from(unsafe { sign_unplace_from_dict(args.ptr(0), dict) });
}

/// `sign_unplacelist()`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub(crate) unsafe fn f_sign_unplacelist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's return slot and argument.
    unsafe { each_dict_arg(args, rettv, |d| sign_unplace_from_dict(null(), d)) };
}
