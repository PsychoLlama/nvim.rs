//! The `sign_*()` Vimscript functions.
//!
//! The same operations the `:sign` command performs, addressed by
//! dictionary rather than by command line, plus the two report functions
//! (`sign_getdefined()`, `sign_getplaced()`) that answer with dictionaries
//! of their own. The `*_from_dict` helpers are shared between the single
//! and the `*list()` bulk forms.

#![forbid(unsafe_code)]
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
use crate::types::Handle;
use crate::types::{VAR_DICT, VAR_LIST, kListLenMayKnow};

/// The four highlight keys a sign definition carries, in the order every
/// reader in this family reports them.
const HL_KEYS: [&str; 4] = ["linehl", "texthl", "culhl", "numhl"];

/// `tv_dict_add_str` with a Rust key; `tv_dict_item_alloc_len` copies
/// exactly the length it is given.
fn put_str(d: &mut Dict, key: &str, val: &[u8]) {
    let _ = d.add_str_len(key.as_bytes(), Some(val));
}

/// `tv_dict_add_nr` with a Rust key; see [`put_str`].
fn put_nr(d: &mut Dict, key: &str, nr: VarNumber) {
    let _ = d.add_number(key.as_bytes(), nr);
}

/// The value stored under `key`, or `None` when the dictionary has no such
/// key.
fn key<'a>(d: Option<&'a Dict>, key: &str) -> Option<&'a TypVal> {
    Some(&dict_find(d, key.as_bytes())?.di_tv)
}

/// Argument `i` as a dictionary, or `None` when it was not supplied (or is
/// `v:_null_dict`). Whether a supplied one *is* a dictionary is the
/// caller's check -- `tv_check_for_*_dict_arg` is what does that.
fn dict_arg(args: &[TypVal], i: usize) -> Option<&Dict> {
    args.get(i).and_then(TypVal::dict_ref)
}

/// A `group` argument: `None` when it does not read as a string at all, and
/// `Some(None)` for the empty string, which names the global group.
fn group_arg(tv: &TypVal) -> Option<Option<Vec<u8>>> {
    let mut numbuf = NumBuf::new();
    let group = numbuf.string_chk(tv).ok()?;
    Some((!group.is_empty()).then(|| group.to_bytes().to_vec()))
}

/// The name of highlight group `id`, or `"NONE"` when it has none.
fn hl_name(id: c_int) -> &'static CStr {
    get_highlight_name_ext(id - 1, false).unwrap_or(c"NONE")
}

/// Runs `one` over every dictionary in `l`, appending what it answers to
/// `retlist` -- a refusal reported, and `-1`. An entry that is not a
/// dictionary is E715.
///
/// Nothing `one` does here runs user code, so the list is walked as
/// borrowed.
fn each_dict(
    retlist: &mut List,
    l: Option<&List>,
    mut one: impl FnMut(Option<&Dict>) -> Result<c_int, SignError>,
) {
    for item in list_items(l) {
        let retval = if item.li_tv.v_type() == VAR_DICT {
            one(item.li_tv.dict_ref())
        } else {
            Err(SignError::DictionaryRequired)
        };
        retlist.push_number(VarNumber::from(answer_or_report(retval)));
    }
}

/// The body `sign_placelist()` and `sign_unplacelist()` share: a list of
/// what `one` answered for each dictionary in the argument list.
///
/// The return list is allocated *before* the type check, so a non-list
/// argument still answers `[]` and not `0`.
fn each_dict_arg(
    args: &[TypVal],
    result: &mut TypVal,
    one: impl FnMut(Option<&Dict>) -> Result<c_int, SignError>,
) {
    let retlist = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    match args.first() {
        Some(arg) if arg.v_type() == VAR_LIST => each_dict(retlist, arg.list_ref(), one),
        _ => SignError::ListRequired.report(),
    }
}

/// `sign_getdefined()`'s dictionary for one defined sign.
pub(crate) fn sign_get_info_dict(sign: &Sign) -> DictRef {
    let mut held = tv_dict_alloc();
    let d = &mut *held;
    put_str(d, "name", sign.name.as_bytes());
    if let Some(icon) = &sign.icon {
        put_str(d, "icon", icon);
    }
    if sign.text[0] != 0 {
        put_str(d, "text", &sign_text_bytes(&sign.text));
    }
    if sign.priority > 0 {
        put_nr(d, "priority", VarNumber::from(sign.priority));
    }
    let ids = [sign.line_hl, sign.text_hl, sign.cul_hl, sign.num_hl];
    for (key, id) in HL_KEYS.iter().zip(ids) {
        if id > 0 {
            put_str(d, key, hl_name(id).to_bytes());
        }
    }
    held
}

/// `sign_getplaced()`'s dictionary for one placed sign.
pub(crate) fn sign_get_placed_info_dict(mark: MTKey) -> DictRef {
    let mut held = tv_dict_alloc();
    let d = &mut *held;
    let sh = sign_of(mark);
    put_str(d, "name", placed_sign_name(&sh));
    put_nr(d, "id", VarNumber::from(mark.id.cast_signed()));
    put_str(
        d,
        "group",
        &describe_ns(Handle::new(mark.ns.cast_signed()), b""),
    );
    put_nr(d, "lnum", VarNumber::from(mark.pos.row + 1));
    put_nr(d, "priority", VarNumber::from(sh.priority));
    held
}

/// Every sign placed in `buffer`, in marktree order — `getbufinfo()`'s `signs`.
pub(crate) fn get_buffer_signs(buffer: Buf) -> ListRef {
    let signs = placed_signs(buffer, 0, ALL_GROUPS, |_| Keep::Yes);
    let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    for mark in signs {
        l.edit().push_dict(Some(sign_get_placed_info_dict(mark)));
    }
    l
}

/// Appends `buffer`'s `{ bufnr, signs }` entry to `retlist`, filtered by `lnum`,
/// `sign_id` and `group`.
///
/// A zero `lnum` or `sign_id` means "any"; the two combine, so naming both
/// asks for one specific sign on one specific line.
fn sign_get_placed_in_buf(
    buffer: Buf,
    lnum: LineNr,
    sign_id: c_int,
    group: Option<&[u8]>,
    retlist: &mut List,
) {
    let d = tv_dict_alloc();
    let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    retlist.push_dict(Some(d.clone()));
    put_nr(&mut d.edit(), "bufnr", VarNumber::from(buffer.handle));
    let _ = d.edit().add_list(b"signs", Some(l.clone()));

    let ns = group_get_ns(group);
    if !buf_has_signs(buffer) || ns < 0 {
        return;
    }

    let first_row = if lnum != LineNr::ZERO {
        lnum - 1
    } else {
        LineNr::ZERO
    };
    let mut signs = placed_signs(buffer, first_row.get(), ns, |mark| {
        if lnum != LineNr::ZERO && mark.pos.row >= lnum.get() {
            // The tree is in row order, so nothing past this row can match.
            return Keep::Stop;
        }
        // A zero `lnum` or `sign_id` means "any"; the two combine.
        let on_line = lnum == LineNr::ZERO || lnum == LineNr::new(mark.pos.row + 1);
        let is_id = sign_id == 0 || sign_id == mark.id.cast_signed();
        if on_line && is_id {
            Keep::Yes
        } else {
            Keep::No
        }
    });

    sort_signs(&mut signs);
    for mark in signs {
        l.edit().push_dict(Some(sign_get_placed_info_dict(mark)));
    }
}

/// Appends the placed-sign report for `buffer`, or for every buffer that has
/// signs when it is `None`.
fn sign_get_placed(
    buffer: Option<Buf>,
    lnum: LineNr,
    id: c_int,
    group: Option<&[u8]>,
    retlist: &mut List,
) {
    if let Some(buffer) = buffer {
        sign_get_placed_in_buf(buffer, lnum, id, group, retlist);
        return;
    }
    for cbuf in buffers() {
        if buf_has_signs(cbuf) {
            // `lnum` is deliberately dropped: an all-buffers query
            // reports every line whatever line was asked for.
            sign_get_placed_in_buf(cbuf, LineNr::ZERO, id, group, retlist);
        }
    }
}

/// The string under `key` in `dict`, copied out.
fn dict_bytes(dict: Option<&Dict>, key: &[u8]) -> Option<Vec<u8>> {
    let mut numbuf = NumBuf::new();
    numbuf.dict_string(dict, key).map(|s| s.to_bytes().to_vec())
}

/// Defines one sign from a dictionary; the builtins answer 0 for it.
///
/// `name` is `None` for the list form, where the name is the dictionary's
/// own `name` key. Every value is copied out of the dictionary, which is the
/// caller's: unescaping the text happens in a buffer of the definition's own.
fn sign_define_from_dict(name: Option<&[u8]>, dict: Option<&Dict>) -> Result<c_int, SignError> {
    let name = match name {
        Some(name) => name.to_vec(),
        None => match dict_bytes(dict, b"name") {
            Some(name) if !name.is_empty() => name,
            _ => return Err(SignError::Quiet),
        },
    };
    let mut attrs = SignAttrs {
        prio: -1,
        ..Default::default()
    };
    if dict.is_some() {
        attrs.icon = dict_bytes(dict, b"icon");
        attrs.linehl = dict_bytes(dict, b"linehl");
        attrs.text = dict_bytes(dict, b"text");
        attrs.texthl = dict_bytes(dict, b"texthl");
        attrs.culhl = dict_bytes(dict, b"culhl");
        attrs.numhl = dict_bytes(dict, b"numhl");
        attrs.prio = number_as_int(dict_get_number_def(dict, b"priority", -1));
    }
    sign_define_by_name(&name, attrs).map(|()| 0)
}

/// `sign_define()`.
pub(crate) fn f_sign_define(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if args[0].v_type() == VAR_LIST && args.len() <= 1 {
        let retlist = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
        each_dict(retlist, args[0].list_ref(), |d| {
            sign_define_from_dict(None, d)
        });
        return;
    }

    result.write_number(-1);
    let mut numbuf = NumBuf::new();
    let Some(name) = numbuf
        .string_chk(&args[0])
        .ok()
        .map(|name| name.to_bytes().to_vec())
    else {
        return;
    };
    if tv_check_for_opt_dict_arg(args, 1).is_err() {
        return;
    }
    let defined = sign_define_from_dict(Some(&name), dict_arg(args, 1));
    result.write_number(VarNumber::from(answer_or_report(defined)));
}

/// `sign_getdefined()`.
pub(crate) fn f_sign_getdefined(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let l = tv_list_alloc_ret(result, 0);
    let defs = match args.first() {
        Some(name) => sign_find(numbuf.string(name).to_bytes())
            .into_iter()
            .collect(),
        None => sign_defs(),
    };
    for sign in &defs {
        l.push_dict(Some(sign_get_info_dict(sign)));
    }
}

/// `sign_getplaced()`.
pub(crate) fn f_sign_getplaced(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf = None;
    let mut lnum: LineNr = LineNr::ZERO;
    let mut sign_id = 0;
    let mut group = None;

    let l = tv_list_alloc_ret(result, 0);

    if let Some(arg) = args.first() {
        buf = get_buf_arg(arg);
        if buf.is_none() {
            return;
        }
        if args.len() > 1 {
            if tv_check_for_nonnull_dict_arg(args, 1).is_err() {
                return;
            }
            let dict = args[1].dict_ref();

            if let Some(tv) = key(dict, "lnum") {
                lnum = tv_get_lnum(tv);
                if lnum <= LineNr::ZERO {
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
                let mut numbuf = NumBuf::new();
                let Ok(given) = numbuf.string_chk(tv) else {
                    return;
                };
                // The empty string means the global group.
                if !given.is_empty() {
                    group = Some(given.to_bytes().to_vec());
                }
            }
        }
    }

    sign_get_placed(buf, lnum, sign_id, group.as_deref(), l);
}

/// `sign_jump()`.
pub(crate) fn f_sign_jump(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);

    let jumped = sign_jump_from_args(args).map(LineNr::get);
    result.write_number(VarNumber::from(answer_or_report(jumped)));
}

/// `sign_jump()`'s arguments, and the jump.
fn sign_jump_from_args(args: &[TypVal]) -> Result<LineNr, SignError> {
    let id = number_as_int(tv_get_number_chk(&args[0])?);
    if id <= 0 {
        return Err(SignError::InvalidArgument);
    }
    // A copy: the jump runs autocommands.
    let group = group_arg(&args[1]).ok_or(Failed)?;
    let buf = get_buf_arg(&args[2]).ok_or(Failed)?;
    sign_jump(id, group.as_deref(), buf)
}

/// The named key's value, or the positional typval when there is one.
fn slot<'a>(tv: Option<&'a TypVal>, dict: Option<&'a Dict>, name: &str) -> Option<&'a TypVal> {
    tv.or_else(|| key(dict, name))
}

/// `sign_place()`'s positional arguments, all `None` for
/// `sign_placelist()`, where the dictionary carries them instead.
#[derive(Clone, Copy, Default)]
struct PlaceArgs<'a> {
    id: Option<&'a TypVal>,
    group: Option<&'a TypVal>,
    name: Option<&'a TypVal>,
    buffer: Option<&'a TypVal>,
}

/// Places one sign described by a dictionary; answers its id.
///
/// A key that does not read as its type has been reported by the reader
/// ([`SignError::Reported`]); a missing name or buffer is a quiet refusal.
fn sign_place_from_dict(given: PlaceArgs<'_>, dict: Option<&Dict>) -> Result<c_int, SignError> {
    let mut id = 0;
    if let Some(tv) = slot(given.id, dict, "id") {
        id = number_as_int(tv_get_number_chk(tv)?);
        if id < 0 {
            return Err(SignError::InvalidArgument);
        }
    }

    let mut group = None;
    if let Some(tv) = slot(given.group, dict, "group") {
        group = group_arg(tv).ok_or(Failed)?;
    }

    let name_tv = slot(given.name, dict, "name").ok_or(SignError::Quiet)?;
    let mut numbuf = NumBuf::new();
    let name = numbuf
        .string_chk(name_tv)
        .ok()
        .map(|name| name.to_bytes().to_vec())
        .ok_or(Failed)?;

    let buf_tv = slot(given.buffer, dict, "buffer").ok_or(SignError::Quiet)?;
    let buf = get_buf_arg(buf_tv).ok_or(Failed)?;

    let mut lnum: LineNr = LineNr::ZERO;
    if let Some(tv) = key(dict, "lnum") {
        lnum = tv_get_lnum(tv);
        if lnum <= LineNr::ZERO {
            return Err(SignError::InvalidArgument);
        }
    }

    let mut prio = -1;
    if let Some(tv) = key(dict, "priority") {
        prio = number_as_int(tv_get_number_chk(tv)?);
    }

    // `sign_place` writes the id back when it was zero (auto-allocate).
    let mut uid = id.cast_unsigned();
    sign_place(&mut uid, group.as_deref(), &name, buf, lnum, prio)?;
    Ok(uid.cast_signed())
}

/// `sign_place()`.
pub(crate) fn f_sign_place(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    let mut dict = None;
    if args.len() > 4 {
        if tv_check_for_nonnull_dict_arg(args, 4).is_err() {
            return;
        }
        dict = args[4].dict_ref();
    }
    let given = PlaceArgs {
        id: Some(&args[0]),
        group: Some(&args[1]),
        name: Some(&args[2]),
        buffer: Some(&args[3]),
    };
    let id = sign_place_from_dict(given, dict);
    result.write_number(VarNumber::from(answer_or_report(id)));
}

/// `sign_placelist()`.
pub(crate) fn f_sign_placelist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    each_dict_arg(args, result, |d| {
        sign_place_from_dict(PlaceArgs::default(), d)
    });
}

/// `sign_undefine()`.
pub(crate) fn f_sign_undefine(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) && args.len() <= 1 {
        let retlist = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
        for item in list_items(args[0].list_ref()) {
            let undefined = match numbuf.string_chk(&item.li_tv).ok() {
                Some(name) => sign_undefine_by_name(name.to_bytes()).map(|()| 0),
                None => Err(SignError::Reported(Failed)),
            };
            retlist.push_number(VarNumber::from(answer_or_report(undefined)));
        }
        return;
    }

    result.write_number(-1);
    if args.is_empty() {
        free_signs();
        result.write_number(0);
        return;
    }
    if let Some(name) = numbuf.string_chk(&args[0]).ok() {
        let undefined = sign_undefine_by_name(name.to_bytes()).map(|()| 0);
        result.write_number(VarNumber::from(answer_or_report(undefined)));
    }
}

/// Removes the signs a dictionary describes; the builtins answer 0 for it.
///
/// `group_tv` is `sign_unplace()`'s positional group and is `None` for
/// `sign_unplacelist()`, where the dictionary carries it.
fn sign_unplace_from_dict(
    group_tv: Option<&TypVal>,
    dict: Option<&Dict>,
) -> Result<c_int, SignError> {
    let mut numbuf = NumBuf::new();
    let group = match group_tv {
        Some(tv) => Some(numbuf.string(tv).to_bytes().to_vec()),
        None => dict_bytes(dict, b"group"),
    };
    // The empty string means the global group.
    let group = group.filter(|group| !group.is_empty());

    let mut id = 0;
    let mut buf = None;
    if dict.is_some() {
        if let Some(tv) = key(dict, "buffer") {
            buf = Some(get_buf_arg(tv).ok_or(Failed)?);
        }
        if key(dict, "id").is_some() {
            id = number_as_int(dict_get_number(dict, b"id"));
            if id <= 0 {
                return Err(SignError::InvalidArgument);
            }
        }
    }

    sign_unplace(buf, id, group.as_deref(), LineNr::ZERO).map(|()| 0)
}

/// `sign_unplace()`.
pub(crate) fn f_sign_unplace(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    if tv_check_for_string_arg(args, 0).is_err() || tv_check_for_opt_dict_arg(args, 1).is_err() {
        return;
    }
    let unplaced = sign_unplace_from_dict(Some(&args[0]), dict_arg(args, 1));
    result.write_number(VarNumber::from(answer_or_report(unplaced)));
}

/// `sign_unplacelist()`.
pub(crate) fn f_sign_unplacelist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    each_dict_arg(args, result, |d| sign_unplace_from_dict(None, d));
}
