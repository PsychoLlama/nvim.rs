//! The walk `typval_encode.c.h` emits around its hooks: the two functions
//! upstream calls `_typval_encode_<sink>_convert_one_value` and
//! `encode_vim_to_<sink>`, written once against [`TypvalSink`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{
    ConvFrame, ConvPath, ConvStack, ConvType, Flow, Frame, PartialStage, Refused, TypvalSink,
};
use crate::eval::encode::encode_vim_list_to_buf;
use crate::eval::typval::{
    DictSlot, Dt, Li, Pt, Tv, di_tv, dv_copyid, li_tv, lv_copyid, tv_blob_len, tv_dict_find,
    tv_dict_hi2di, tv_dict_item_key, tv_list_copyid, tv_list_first, tv_list_last, tv_list_len,
    tv_list_set_copyid,
};
use crate::eval::vars::eval_msgpack_type_lists;
use crate::eval::{get_copy_id, partial_name};
use crate::memory::xfree;
use crate::message::internal_error;
use crate::types::{
    Dict, DictItem, List, Partial, TypVal, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC,
    VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarNumber, int64_t,
    kBoolVarFalse, kBoolVarTrue, kSpecialVarNull, ptrdiff_t, size_t,
};

/// Apply a hook's verdict inside `convert_one_value`, where "stop" is
/// upstream's `goto typval_encode_stop_converting_one_item` into that
/// function's own tail.
macro_rules! item_hook {
    ($e:expr) => {
        match $e {
            Flow::Go => {}
            Flow::Stop => return Ok(()),
            Flow::Fail => return Err(Refused),
        }
    };
}

/// Apply a hook's verdict inside the stack walk, where the same label is the
/// top of the loop.
macro_rules! walk_hook {
    ($e:expr) => {
        match $e {
            Flow::Go => {}
            Flow::Stop => continue,
            Flow::Fail => return Err(Refused),
        }
    };
}

/// Length of a `VAR_STRING`'s string, NULL reading as empty.
///
/// # Safety
/// `tv` must point at a live `VAR_STRING` typval.
pub(crate) unsafe fn tv_strlen(tv: &TypVal) -> size_t {
    // SAFETY: the caller's promise: a live VAR_STRING typval.
    let val = tv;
    debug_assert!(val.v_type() == VAR_STRING);
    if val.string_or_null().is_null() {
        0
    } else {
        unsafe { cstr::bytes_at((*tv).string_or_null()) }.len()
    }
}

/// Mark `val` with `copy_id`, or tell the sink it has been here before.
///
/// Answers [`Flow::Go`] for a container the walk has not seen (upstream's
/// `NOTDONE`), and otherwise whatever the sink makes of the self-reference.
///
/// # Safety
///
/// `val` must be the container the walk is standing on and `val_copyid` the
/// address of that container's `copyID` field, both live and unaliased for
/// the call.
unsafe fn check_self_reference<S: TypvalSink>(
    sink: &mut S,
    val: *mut c_void,
    val_copyid: *mut c_int,
    conv_type: ConvType,
    copyid: c_int,
    path: &ConvPath,
) -> Flow {
    if unsafe { *val_copyid } == copyid {
        // The macro either bails or falls through to `return OK`, which
        // out here is "this value is done".
        return match unsafe { sink.conv_recurse(val, conv_type, path) } {
            Flow::Go => Flow::Stop,
            other => other,
        };
    }
    unsafe { *val_copyid = copyid };
    Flow::Go
}

/// [`check_self_reference`] for a list, whose copyID is a field of its own.
///
/// The pair `(val, val_copyid)` is always an object and *that object's* copyID
/// slot, so spelling the projection once here keeps every call site to one
/// line instead of the eight rustfmt gives a six-argument call.
///
/// # Safety
/// `list` must point at a live list and `copyid` be one the caller reserved
/// from `get_copyID`; the sink's own obligations pass straight through.
#[inline]
unsafe fn check_list_seen<S: TypvalSink>(
    sink: &mut S,
    list: *mut List,
    conv_type: ConvType,
    copyid: c_int,
    path: &ConvPath,
) -> Flow {
    // SAFETY: the caller's live list; `lv_copy_id` is a field of it.
    let seen = lv_copyid(list);
    // SAFETY: as above.
    unsafe { check_self_reference(sink, list.cast(), seen, conv_type, copyid, path) }
}

/// [`check_list_seen`] for a dictionary.
///
/// # Safety
/// As [`check_list_seen`], with `dict` a live dictionary.
#[inline]
unsafe fn check_dict_seen<S: TypvalSink>(
    sink: &mut S,
    dict: *mut Dict,
    copyid: c_int,
    path: &ConvPath,
) -> Flow {
    // SAFETY: the caller's live dictionary; `dv_copy_id` is a field of it.
    let seen = dv_copyid(dict);
    let ty = ConvType::Dict;
    // SAFETY: as above.
    unsafe { check_self_reference(sink, dict.cast(), seen, ty, copyid, path) }
}

/// The eight `_TYPE` markers a special dictionary can carry, in the order
/// `eval_msgpack_type_lists` holds them (upstream's `MessagePackType`).
#[derive(Copy, Clone, PartialEq, Eq)]
enum SpecialKind {
    Nil,
    Bool,
    Integer,
    Float,
    String,
    Array,
    Map,
    Ext,
}

const SPECIAL_KINDS: [SpecialKind; 8] = [
    SpecialKind::Nil,
    SpecialKind::Bool,
    SpecialKind::Integer,
    SpecialKind::Float,
    SpecialKind::String,
    SpecialKind::Array,
    SpecialKind::Map,
    SpecialKind::Ext,
];

/// Convert one value, pushing any container it opens onto `stack`.
///
/// Only scalars are finished here; a list or dictionary is announced to the
/// sink and left for the walk to feed back one item at a time.
///
/// # Safety
///
/// `tv` must point at the value the walk is standing on, live and unaliased
/// for the call, and `objname` at the NUL-terminated name the walk's error
/// messages use.  A raw pointer, not a borrow: the value comes back out of
/// [`ConvStack`], whose frames hold the containers they are suspended in by
/// address, and each hook takes its own `&mut` of the slot in turn.
unsafe fn convert_one_value<S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack,
    tv: *mut TypVal,
    copyid: c_int,
    objname: &CStr,
) -> Result<(), Refused> {
    sink.check_before();
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv) };
    // The hook's borrow of the slot the walk is standing on.  Taken *after*
    // whatever else the arm reads out of it, so that no read goes through the
    // raw pointer while the borrow is live.
    macro_rules! slot {
        () => {
            tv.as_mut()
        };
    }
    match val.v_type() {
        VAR_STRING => {
            let (buf, len) = (val.string_or_null(), unsafe { tv_strlen(&*tv) });
            item_hook!(unsafe { sink.conv_string(slot!(), buf, len) });
        }
        VAR_NUMBER => {
            let n = val.number_or_zero();
            unsafe { sink.conv_number(slot!(), n) };
        }
        VAR_FLOAT => {
            let f = val.float_or_zero();
            item_hook!(unsafe { sink.conv_float(slot!(), f) });
        }
        VAR_BLOB => {
            let blob = val.blob_or_null();
            let len = unsafe { tv_blob_len(blob) };
            unsafe { sink.conv_blob(slot!(), blob, len) };
        }
        VAR_FUNC => {
            let name = val.func_name_or_null();
            let path = ConvPath { stack, objname };
            item_hook!(unsafe { sink.conv_func_start(slot!(), name, c"", &path) });
            unsafe { sink.conv_func_before_args(slot!(), 0) };
            unsafe { sink.conv_func_before_self(slot!(), -1) };
            unsafe { sink.conv_func_end(slot!(), copyid) };
        }
        VAR_PARTIAL => {
            let pt = val.partial_or_null();
            let fun = if pt.is_null() {
                ptr::null_mut()
            } else {
                unsafe { partial_name(pt) }
            };
            // When using uf_name prepend "g:" for a global function.
            let prefix =
                if !fun.is_null() && !pt.is_null() && unsafe { (*pt).pt_name }.is_null() && {
                    let c = unsafe { *fun } as u8;
                    c.is_ascii_uppercase()
                } {
                    c"g:"
                } else {
                    c""
                };
            {
                let path = ConvPath { stack, objname };
                item_hook!(unsafe { sink.conv_func_start(slot!(), fun, prefix, &path) });
            }
            stack.push(ConvFrame {
                tv,
                saved_copyid: copyid - 1,
                frame: Frame::Partial {
                    stage: PartialStage::Args,
                    pt: val.partial_or_null(),
                },
            });
        }
        VAR_LIST => {
            let list = val.list_or_null();
            if list.is_null() || unsafe { tv_list_len(list) } == 0 {
                unsafe { sink.conv_empty_list(slot!()) };
            } else {
                let saved_copyid = unsafe { tv_list_copyid(list) };
                {
                    let path = ConvPath { stack, objname };
                    let ty = ConvType::List;
                    item_hook!(unsafe { check_list_seen(sink, list, ty, copyid, &path) });
                }
                let len = unsafe { tv_list_len(list) };
                item_hook!(unsafe { sink.conv_list_start(slot!(), len) });
                debug_assert!(saved_copyid != copyid);
                stack.push(ConvFrame {
                    tv,
                    saved_copyid,
                    frame: Frame::List {
                        list,
                        li: unsafe { tv_list_first(list) },
                    },
                });
                item_hook!(unsafe { sink.conv_real_list_after_start(slot!(), stack.last_mut()) });
            }
        }
        VAR_BOOL => {
            // Upstream switches over the two named values and ignores
            // anything else.
            let b = val.as_bool().unwrap_or(kBoolVarFalse);
            if b == kBoolVarTrue || b == kBoolVarFalse {
                unsafe { sink.conv_bool(slot!(), b == kBoolVarTrue) };
            }
        }
        VAR_SPECIAL => {
            if val.as_special() == Some(kSpecialVarNull) {
                unsafe { sink.conv_nil(slot!()) };
            }
        }
        VAR_DICT => {
            let dict = val.dict_or_null();
            // SAFETY: the typval's own dictionary, live while the typval is.
            let d = unsafe { Dt::new(dict) };
            if dict.is_null() || d.dv_hashtab.ht_used == 0 {
                unsafe { sink.conv_empty_dict(Some(DictSlot::Value(tv))) };
            } else {
                if S::ALLOW_SPECIALS
                    && let Some(flow) =
                        unsafe { convert_special_dict(sink, stack, tv, copyid, objname) }?
                {
                    item_hook!(flow);
                    return Ok(());
                }
                let saved_copyid = d.dv_copy_id;
                {
                    let path = ConvPath { stack, objname };
                    item_hook!(unsafe { check_dict_seen(sink, dict, copyid, &path) });
                }
                let dictp = DictSlot::Value(tv);
                let used = d.dv_hashtab.ht_used;
                item_hook!(unsafe { sink.conv_dict_start(slot!(), used) });
                debug_assert!(saved_copyid != copyid);
                stack.push(ConvFrame {
                    tv,
                    saved_copyid,
                    frame: Frame::Dict {
                        dict,
                        dictp,
                        idx: 0,
                        todo: d.dv_hashtab.ht_used,
                    },
                });
                let dp = Some(dictp);
                item_hook!(unsafe { sink.conv_real_dict_after_start(dp, stack.last_mut()) });
            }
        }
        VAR_UNKNOWN => {
            unsafe { internal_error(S::CONVERT_FN_NAME.as_ptr()) };
            return Err(Refused);
        }
        _ => {}
    }
    Ok(())
}

/// The `{_TYPE: v:msgpack_types.x, _VAL: …}` form, for the sinks that read it.
///
/// `None` is upstream's `goto _convert_one_value_regular_dict`: the dictionary
/// looked special but is not, so the caller emits it as an ordinary one.
/// `Some(flow)` means it was handled — including the two arms that push a
/// container for the walk to drain.
///
/// # Safety
///
/// `tv` must point at the special dictionary the walk is standing on, live
/// and unaliased for the call, and `objname` at the NUL-terminated name the
/// walk's error messages use.
unsafe fn convert_special_dict<S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack,
    tv: *mut TypVal,
    copyid: c_int,
    objname: &CStr,
) -> Result<Option<Flow>, Refused> {
    let dict = unsafe { (*tv).dict_or_null() };
    if unsafe { (*dict).dv_hashtab.ht_used } != 2 {
        return Ok(None);
    }
    let type_di: *const DictItem = unsafe { tv_dict_find(dict, c"_TYPE".as_ptr(), 5) };
    if type_di.is_null() || unsafe { (*type_di).di_tv.v_type() } != VAR_LIST {
        return Ok(None);
    }
    let val_di: *const DictItem = unsafe { tv_dict_find(dict, c"_VAL".as_ptr(), 4) };
    if val_di.is_null() {
        return Ok(None);
    }
    let type_list = unsafe { (*type_di).di_tv.list_or_null() };
    let found = eval_msgpack_type_lists
        .get()
        .iter()
        .position(|&l| l == type_list.cast_const());
    // Upstream runs the check a second time here, before it knows whether
    // this is a special dictionary at all.
    sink.check_before();
    let Some(found) = found else {
        return Ok(None);
    };
    let val_tv = di_tv(val_di.cast_mut());
    // SAFETY: the `_VAL` item's value, live while the dictionary is.
    let val = unsafe { Tv::new(val_tv) };
    // As `convert_one_value`: the borrow is taken last.
    macro_rules! slot {
        () => {
            tv.as_mut()
        };
    }

    match SPECIAL_KINDS[found] {
        SpecialKind::Nil => unsafe { sink.conv_nil(slot!()) },
        SpecialKind::Bool => {
            if val.v_type() != VAR_NUMBER {
                return Ok(None);
            }
            let b = val.number_or_zero() != 0;
            unsafe { sink.conv_bool(slot!(), b) };
        }
        SpecialKind::Integer => {
            // A list of four integers: a sign (nominally ±1), then the
            // number in three unsigned pieces, most significant first.
            // How many bits each piece really carries is not checked.
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let val_list = val.list_or_null();
            if unsafe { tv_list_len(val_list) } != 4 {
                return Ok(None);
            }
            // SAFETY: the four items of a list this long, walked forwards
            // from the head as upstream does.
            let sign_li = unsafe { Li::new(tv_list_first(val_list)) };
            let sign: VarNumber = sign_li.number();
            if sign_li.v_type() != VAR_NUMBER || sign == 0 {
                return Ok(None);
            }
            let highest_bits_li = unsafe { Li::new(sign_li.li_next) };
            let highest_bits: VarNumber = highest_bits_li.number();
            if highest_bits_li.v_type() != VAR_NUMBER || highest_bits < 0 {
                return Ok(None);
            }
            let high_bits_li = unsafe { Li::new(highest_bits_li.li_next) };
            let high_bits: VarNumber = high_bits_li.number();
            if high_bits_li.v_type() != VAR_NUMBER || high_bits < 0 {
                return Ok(None);
            }
            let low_bits_li = unsafe { Li::new(tv_list_last(val_list)) };
            let low_bits: VarNumber = low_bits_li.number();
            if low_bits_li.v_type() != VAR_NUMBER || low_bits < 0 {
                return Ok(None);
            }
            let number =
                ((highest_bits as u64) << 62) | ((high_bits as u64) << 31) | (low_bits as u64);
            if sign > 0 {
                unsafe { sink.conv_unsigned_number(slot!(), number) };
            } else {
                let n = number.wrapping_neg() as int64_t;
                unsafe { sink.conv_number(slot!(), n) };
            }
        }
        SpecialKind::Float => {
            if val.v_type() != VAR_FLOAT {
                return Ok(None);
            }
            let f = val.float_or_zero();
            return Ok(Some(unsafe { sink.conv_float(slot!(), f) }));
        }
        SpecialKind::String => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let mut len: size_t = 0;
            let mut buf: *mut c_char = ptr::null_mut();
            let val_list = val.list_or_null();
            if !unsafe { encode_vim_list_to_buf(val_list, &raw mut len, &raw mut buf) } {
                return Ok(None);
            }
            let flow = unsafe { sink.conv_str_string(slot!(), buf, len) };
            if flow == Flow::Go {
                unsafe { xfree(buf.cast()) };
            }
            return Ok(Some(flow));
        }
        SpecialKind::Array => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let val_list = val.list_or_null();
            let saved_copyid = unsafe { tv_list_copyid(val_list) };
            {
                let path = ConvPath { stack, objname };
                let ty = ConvType::List;
                match unsafe { check_list_seen(sink, val_list, ty, copyid, &path) } {
                    Flow::Go => {}
                    other => return Ok(Some(other)),
                }
            }
            let len = unsafe { tv_list_len(val_list) };
            match unsafe { sink.conv_list_start(slot!(), len) } {
                Flow::Go => {}
                other => return Ok(Some(other)),
            }
            debug_assert!(saved_copyid != copyid && saved_copyid != copyid - 1);
            stack.push(ConvFrame {
                tv,
                saved_copyid,
                frame: Frame::List {
                    list: val_list,
                    li: unsafe { tv_list_first(val_list) },
                },
            });
        }
        SpecialKind::Map => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let val_list = val.list_or_null();
            if val_list.is_null() || unsafe { tv_list_len(val_list) } == 0 {
                unsafe { sink.conv_empty_dict(None) };
                return Ok(Some(Flow::Go));
            }
            // Every item has to be a two-element list, or this is not a
            // map after all.
            let mut li = unsafe { tv_list_first(val_list) };
            while !li.is_null() {
                let item = li_tv(li);
                if unsafe { (*item).v_type() } != VAR_LIST
                    || unsafe { tv_list_len((*item).list_or_null()) } != 2
                {
                    return Ok(None);
                }
                li = unsafe { (*li).li_next };
            }
            let saved_copyid = unsafe { tv_list_copyid(val_list) };
            {
                let path = ConvPath { stack, objname };
                let ty = ConvType::Pairs;
                match unsafe { check_list_seen(sink, val_list, ty, copyid, &path) } {
                    Flow::Go => {}
                    other => return Ok(Some(other)),
                }
            }
            let len = unsafe { tv_list_len(val_list) } as size_t;
            match unsafe { sink.conv_dict_start(slot!(), len) } {
                Flow::Go => {}
                other => return Ok(Some(other)),
            }
            debug_assert!(saved_copyid != copyid && saved_copyid != copyid - 1);
            stack.push(ConvFrame {
                tv,
                saved_copyid,
                frame: Frame::Pairs {
                    list: val_list,
                    li: unsafe { tv_list_first(val_list) },
                },
            });
        }
        SpecialKind::Ext => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let val_list = val.list_or_null();
            if unsafe { tv_list_len(val_list) } != 2 {
                return Ok(None);
            }
            // SAFETY: the two items of a two-item list.
            let first = unsafe { Li::new(tv_list_first(val_list)) };
            // SAFETY: as above.
            let last = unsafe { Li::new(tv_list_last(val_list)) };
            let ext_type = first.number();
            if first.v_type() != VAR_NUMBER
                || ext_type > i8::MAX as VarNumber
                || ext_type < i8::MIN as VarNumber
                || last.v_type() != VAR_LIST
            {
                return Ok(None);
            }
            let mut len: size_t = 0;
            let mut buf: *mut c_char = ptr::null_mut();
            let bytes = last.list();
            if !unsafe { encode_vim_list_to_buf(bytes, &raw mut len, &raw mut buf) } {
                return Ok(None);
            }
            let flow = unsafe { sink.conv_ext_string(slot!(), buf, len, ext_type as i8) };
            if flow == Flow::Go {
                unsafe { xfree(buf.cast()) };
            }
            return Ok(Some(flow));
        }
    }
    Ok(Some(Flow::Go))
}

/// Walk `top_tv` and hand every value to `sink`.
///
/// Returns whether the encode ran to completion; a sink that refuses a value
/// has already reported why.
///
/// # Safety
/// `objname` must point at a NUL-terminated name used only for error
/// messages.
pub(crate) unsafe fn encode_typval<S: TypvalSink>(
    sink: &mut S,
    top_tv: &mut TypVal,
    objname: &CStr,
) -> bool {
    unsafe { walk(sink, ptr::from_mut(top_tv), objname).is_ok() }
}

/// [`encode_typval`] for a sink that only reads.
///
/// The walk hands each hook a `&mut` of the slot it is standing on, so a
/// sink that wrote through one would be writing through a shared borrow --
/// which is why `S::WRITES_BACK` is asserted here rather than trusted.
///
/// # Safety
/// As [`encode_typval`].
pub(crate) unsafe fn encode_typval_read<S: TypvalSink>(
    sink: &mut S,
    top_tv: &TypVal,
    objname: &CStr,
) -> bool {
    const { assert!(!S::WRITES_BACK, "this sink writes to what it walks") };
    let top = ptr::from_ref(top_tv).cast_mut();
    unsafe { walk(sink, top, objname).is_ok() }
}

/// # Safety
///
/// `objname` must point at the NUL-terminated name the error messages use.
unsafe fn walk<S: TypvalSink>(
    sink: &mut S,
    top_tv: *mut TypVal,
    objname: &CStr,
) -> Result<(), Refused> {
    let copyid = get_copy_id();
    let mut stack = ConvStack::new();
    unsafe { convert_one_value(sink, &mut stack, top_tv, copyid, objname) }?;

    while !stack.is_empty() {
        // Upstream keeps a `MPConvStackVal *` into the stack across the
        // hooks and the nested key conversion below, which a `kvi_push`
        // may have reallocated out from under it (O-B14-5).  Here every
        // read and every advance goes through `stack` by index instead,
        // so the borrow checker is what guarantees no reference outlives
        // a push -- and each one touches only the fields it needs,
        // because a whole `ConvFrame` is 56 bytes and this loop runs once
        // per item of every container the interpreter builds, walks or
        // frees.  Holding one raw pointer for the pass instead was
        // measured and dropped: it saves two predicted branches an item
        // and nothing a CGU-1 A/B can see.
        let idx = stack.len() - 1;
        let cur_tv = stack.get_mut(idx).tv;
        // The value this pass hands to `convert_one_value`.
        let tv: *mut TypVal;
        match stack.get_mut(idx).frame {
            Frame::Dict {
                dict,
                dictp,
                idx: mut slot,
                mut todo,
            } => {
                // SAFETY: the dictionary this frame was pushed for, which the
                // frame holds a reference to for as long as it is on the
                // stack.
                let mut d = unsafe { Dt::new(dict) };
                if todo == 0 {
                    let saved_copyid = stack.get_mut(idx).saved_copyid;
                    stack.pop();
                    d.dv_copy_id = saved_copyid;
                    unsafe { sink.conv_dict_end(Some(dictp)) };
                    continue;
                }
                if todo != d.dv_hashtab.ht_used {
                    unsafe { sink.conv_dict_between_items(Some(dictp)) };
                }
                while !d.dv_hashtab.slot(slot).is_kept() {
                    slot += 1;
                }
                let di = unsafe { tv_dict_hi2di(d.dv_hashtab.slot(slot)) };
                todo -= 1;
                slot += 1;
                if let Frame::Dict {
                    idx: slot_field,
                    todo: todo_slot,
                    ..
                } = &mut stack.get_mut(idx).frame
                {
                    *slot_field = slot;
                    *todo_slot = todo;
                }
                let key = tv_dict_item_key(di);
                walk_hook!(unsafe { sink.conv_str_string(None, key, cstr::bytes_at(key).len()) });
                unsafe { sink.conv_dict_after_key(Some(dictp)) };
                tv = di_tv(di);
            }
            Frame::List { list, li } => {
                if li.is_null() {
                    let saved_copyid = stack.get_mut(idx).saved_copyid;
                    stack.pop();
                    unsafe { tv_list_set_copyid(list, saved_copyid) };
                    unsafe { sink.conv_list_end(cur_tv.as_mut()) };
                    continue;
                }
                if li != unsafe { tv_list_first(list) } {
                    unsafe { sink.conv_list_between_items(cur_tv.as_mut()) };
                }
                tv = li_tv(li);
                if let Frame::List { li: li_slot, .. } = &mut stack.get_mut(idx).frame {
                    // SAFETY: an item of the frame's list, checked non-null.
                    *li_slot = unsafe { Li::new(li) }.li_next;
                }
            }
            Frame::Pairs { list, li } => {
                if li.is_null() {
                    let saved_copyid = stack.get_mut(idx).saved_copyid;
                    stack.pop();
                    unsafe { tv_list_set_copyid(list, saved_copyid) };
                    unsafe { sink.conv_dict_end(None) };
                    continue;
                }
                if li != unsafe { tv_list_first(list) } {
                    unsafe { sink.conv_dict_between_items(None) };
                }
                // SAFETY: an item of the frame's list, checked non-null above.
                let item = unsafe { Li::new(li) };
                let kv_pair = item.list();
                let key = li_tv(unsafe { tv_list_first(kv_pair) });
                // SAFETY: an item of a `[key, value]` pair, live while the
                // list is.
                walk_hook!(unsafe { sink.special_dict_key_check(&*key) });
                // The key goes through the whole walk, and may itself be a
                // container: this frame is not necessarily the top one by
                // the time it returns, which is why the advance below is
                // by index.  It also stays *un*advanced across the key, so
                // that an error raised there names this pair's index.
                unsafe { convert_one_value(sink, &mut stack, key, copyid, objname) }?;
                unsafe { sink.conv_dict_after_key(None) };
                tv = li_tv(unsafe { tv_list_last(kv_pair) });
                if let Frame::Pairs { li: li_slot, .. } = &mut stack.get_mut(idx).frame {
                    *li_slot = item.li_next;
                }
            }
            Frame::Partial { stage, pt } => {
                // SAFETY: the partial the frame was pushed for; only read
                // once `pt` has been checked non-null, as upstream does.
                let part = unsafe { Pt::new(pt) };
                match stage {
                    PartialStage::Args => {
                        let argc = if pt.is_null() { 0 } else { part.pt_argc };
                        let argc = argc as ptrdiff_t;
                        unsafe { sink.conv_func_before_args(cur_tv.as_mut(), argc) };
                        if let Frame::Partial { stage: slot, .. } = &mut stack.get_mut(idx).frame {
                            *slot = PartialStage::Self_;
                        }
                        if !pt.is_null() && part.pt_argc > 0 {
                            let pt_argc = part.pt_argc;
                            let pt_argv = part.pt_argv;
                            walk_hook!(sink.conv_list_start(None, pt_argc));
                            stack.push(ConvFrame {
                                tv: ptr::null_mut(),
                                saved_copyid: copyid - 1,
                                frame: Frame::PartialArgs {
                                    arg: pt_argv,
                                    argv: pt_argv,
                                    todo: pt_argc as size_t,
                                },
                            });
                        }
                    }
                    PartialStage::Self_ => {
                        if let Frame::Partial { stage: slot, .. } = &mut stack.get_mut(idx).frame {
                            *slot = PartialStage::End;
                        }
                        let dict = if pt.is_null() {
                            ptr::null_mut()
                        } else {
                            part.pt_dict
                        };
                        if dict.is_null() {
                            unsafe { sink.conv_func_before_self(cur_tv.as_mut(), -1) };
                        } else {
                            // SAFETY: the dictionary the frame was pushed for.
                            let frame_dict = unsafe { Dt::new(dict) };
                            let used = frame_dict.dv_hashtab.ht_used;
                            let count = used as ptrdiff_t;
                            unsafe { sink.conv_func_before_self(cur_tv.as_mut(), count) };
                            let dictp = DictSlot::Field(
                                part.field_ptr(::core::mem::offset_of!(Partial, pt_dict)),
                            );
                            if used == 0 {
                                unsafe { sink.conv_empty_dict(Some(dictp)) };
                                continue;
                            }
                            let saved_copyid = frame_dict.dv_copy_id;
                            {
                                let path = ConvPath {
                                    stack: &stack,
                                    objname,
                                };
                                walk_hook!(unsafe { check_dict_seen(sink, dict, copyid, &path) });
                            }
                            walk_hook!(sink.conv_dict_start(None, used));
                            debug_assert!(saved_copyid != copyid && saved_copyid != copyid - 1);
                            stack.push(ConvFrame {
                                tv: ptr::null_mut(),
                                saved_copyid,
                                frame: Frame::Dict {
                                    dict,
                                    dictp,
                                    idx: 0,
                                    todo: used,
                                },
                            });
                            let dp = Some(dictp);
                            walk_hook!(unsafe {
                                sink.conv_real_dict_after_start(dp, stack.last_mut())
                            });
                        }
                    }
                    PartialStage::End => {
                        unsafe { sink.conv_func_end(cur_tv.as_mut(), copyid) };
                        stack.pop();
                    }
                }
                continue;
            }
            Frame::PartialArgs { arg, argv, todo } => {
                if todo == 0 {
                    stack.pop();
                    sink.conv_list_end(None);
                    continue;
                }
                if argv != arg {
                    sink.conv_list_between_items(None);
                }
                tv = arg;
                if let Frame::PartialArgs {
                    arg: arg_slot,
                    todo: todo_slot,
                    ..
                } = &mut stack.get_mut(idx).frame
                {
                    *arg_slot = unsafe { arg.add(1) };
                    *todo_slot = todo - 1;
                }
            }
        }
        unsafe { convert_one_value(sink, &mut stack, tv, copyid, objname) }?;
    }
    Ok(())
}
