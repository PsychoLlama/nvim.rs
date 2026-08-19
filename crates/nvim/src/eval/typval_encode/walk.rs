//! The walk `typval_encode.c.h` emits around its hooks: the two functions
//! upstream calls `_typval_encode_<sink>_convert_one_value` and
//! `encode_vim_to_<sink>`, written once against [`TypvalSink`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{
    ConvFrame, ConvPath, ConvStack, ConvType, Flow, Frame, PartialStage, Refused, TypvalSink,
};
use crate::eval::encode::encode_vim_list_to_buf;
use crate::eval::typval::{
    tv_blob_len, tv_dict_find, tv_dict_hi2di, tv_dict_item_key, tv_list_copyid, tv_list_first,
    tv_list_last, tv_list_len, tv_list_set_copyid,
};
use crate::eval::vars::eval_msgpack_type_lists;
use crate::eval::{get_copyID, partial_name};
use crate::memory::xfree;
use crate::message::internal_error;
use crate::types::{
    VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, dictitem_T, int64_t, kBoolVarFalse, kBoolVarTrue,
    kSpecialVarNull, ptrdiff_t, size_t, typval_T, varnumber_T,
};
use ::libc::strlen;

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
pub(crate) unsafe fn tv_strlen(tv: *const typval_T) -> size_t {
    unsafe {
        debug_assert!((*tv).v_type == VAR_STRING);
        if (*tv).vval.v_string.is_null() {
            0
        } else {
            strlen((*tv).vval.v_string)
        }
    }
}

/// Mark `val` with `copyID`, or tell the sink it has been here before.
///
/// Answers [`Flow::Go`] for a container the walk has not seen (upstream's
/// `NOTDONE`), and otherwise whatever the sink makes of the self-reference.
unsafe fn check_self_reference<S: TypvalSink>(
    sink: &mut S,
    val: *mut c_void,
    val_copyid: *mut c_int,
    conv_type: ConvType,
    copyid: c_int,
    path: &ConvPath,
) -> Flow {
    unsafe {
        if *val_copyid == copyid {
            // The macro either bails or falls through to `return OK`, which
            // out here is "this value is done".
            return match sink.conv_recurse(val, conv_type, path) {
                Flow::Go => Flow::Stop,
                other => other,
            };
        }
        *val_copyid = copyid;
        Flow::Go
    }
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
unsafe fn convert_one_value<S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack,
    tv: *mut typval_T,
    copyid: c_int,
    objname: *const c_char,
) -> Result<(), Refused> {
    unsafe {
        sink.check_before();
        match (*tv).v_type {
            VAR_STRING => {
                item_hook!(sink.conv_string(tv, (*tv).vval.v_string, tv_strlen(tv)));
            }
            VAR_NUMBER => sink.conv_number(tv, (*tv).vval.v_number),
            VAR_FLOAT => item_hook!(sink.conv_float(tv, (*tv).vval.v_float)),
            VAR_BLOB => {
                let blob = (*tv).vval.v_blob;
                sink.conv_blob(tv, blob, tv_blob_len(blob));
            }
            VAR_FUNC => {
                let path = ConvPath { stack, objname };
                item_hook!(sink.conv_func_start(tv, (*tv).vval.v_string, c"", &path));
                sink.conv_func_before_args(tv, 0);
                sink.conv_func_before_self(tv, -1);
                sink.conv_func_end(tv, copyid);
            }
            VAR_PARTIAL => {
                let pt = (*tv).vval.v_partial;
                let fun = if pt.is_null() {
                    ptr::null_mut()
                } else {
                    partial_name(pt)
                };
                // When using uf_name prepend "g:" for a global function.
                let prefix = if !fun.is_null() && !pt.is_null() && (*pt).pt_name.is_null() && {
                    let c = *fun as u8;
                    c.is_ascii_uppercase()
                } {
                    c"g:"
                } else {
                    c""
                };
                {
                    let path = ConvPath { stack, objname };
                    item_hook!(sink.conv_func_start(tv, fun, prefix, &path));
                }
                stack.push(ConvFrame {
                    tv,
                    saved_copyid: copyid - 1,
                    frame: Frame::Partial {
                        stage: PartialStage::Args,
                        pt: (*tv).vval.v_partial,
                    },
                });
            }
            VAR_LIST => {
                let list = (*tv).vval.v_list;
                if list.is_null() || tv_list_len(list) == 0 {
                    sink.conv_empty_list(tv);
                } else {
                    let saved_copyid = tv_list_copyid(list);
                    {
                        let path = ConvPath { stack, objname };
                        item_hook!(check_self_reference(
                            sink,
                            list.cast(),
                            &raw mut (*list).lv_copyID,
                            ConvType::List,
                            copyid,
                            &path,
                        ));
                    }
                    item_hook!(sink.conv_list_start(tv, tv_list_len(list)));
                    debug_assert!(saved_copyid != copyid);
                    stack.push(ConvFrame {
                        tv,
                        saved_copyid,
                        frame: Frame::List {
                            list,
                            li: tv_list_first(list),
                        },
                    });
                    item_hook!(sink.conv_real_list_after_start(tv, stack.last_mut()));
                }
            }
            VAR_BOOL => {
                // Upstream switches over the two named values and ignores
                // anything else.
                let b = (*tv).vval.v_bool;
                if b == kBoolVarTrue || b == kBoolVarFalse {
                    sink.conv_bool(tv, b == kBoolVarTrue);
                }
            }
            VAR_SPECIAL => {
                if (*tv).vval.v_special == kSpecialVarNull {
                    sink.conv_nil(tv);
                }
            }
            VAR_DICT => {
                let dict = (*tv).vval.v_dict;
                if dict.is_null() || (*dict).dv_hashtab.ht_used == 0 {
                    sink.conv_empty_dict(tv, Some(&raw mut (*tv).vval.v_dict));
                } else {
                    if S::ALLOW_SPECIALS
                        && let Some(flow) = convert_special_dict(sink, stack, tv, copyid, objname)?
                    {
                        item_hook!(flow);
                        return Ok(());
                    }
                    let saved_copyid = (*dict).dv_copyID;
                    {
                        let path = ConvPath { stack, objname };
                        item_hook!(check_self_reference(
                            sink,
                            dict.cast(),
                            &raw mut (*dict).dv_copyID,
                            ConvType::Dict,
                            copyid,
                            &path,
                        ));
                    }
                    let dictp = &raw mut (*tv).vval.v_dict;
                    item_hook!(sink.conv_dict_start(tv, (*dict).dv_hashtab.ht_used));
                    debug_assert!(saved_copyid != copyid);
                    stack.push(ConvFrame {
                        tv,
                        saved_copyid,
                        frame: Frame::Dict {
                            dict,
                            dictp,
                            hi: (*dict).dv_hashtab.ht_array,
                            todo: (*dict).dv_hashtab.ht_used,
                        },
                    });
                    item_hook!(sink.conv_real_dict_after_start(tv, Some(dictp), stack.last_mut()));
                }
            }
            VAR_UNKNOWN => {
                internal_error(S::CONVERT_FN_NAME.as_ptr());
                return Err(Refused);
            }
            _ => {}
        }
        Ok(())
    }
}

/// The `{_TYPE: v:msgpack_types.x, _VAL: …}` form, for the sinks that read it.
///
/// `None` is upstream's `goto _convert_one_value_regular_dict`: the dictionary
/// looked special but is not, so the caller emits it as an ordinary one.
/// `Some(flow)` means it was handled — including the two arms that push a
/// container for the walk to drain.
unsafe fn convert_special_dict<S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack,
    tv: *mut typval_T,
    copyid: c_int,
    objname: *const c_char,
) -> Result<Option<Flow>, Refused> {
    unsafe {
        let dict = (*tv).vval.v_dict;
        if (*dict).dv_hashtab.ht_used != 2 {
            return Ok(None);
        }
        let type_di: *const dictitem_T = tv_dict_find(dict, c"_TYPE".as_ptr(), 5);
        if type_di.is_null() || (*type_di).di_tv.v_type != VAR_LIST {
            return Ok(None);
        }
        let val_di: *const dictitem_T = tv_dict_find(dict, c"_VAL".as_ptr(), 4);
        if val_di.is_null() {
            return Ok(None);
        }
        let type_list = (*type_di).di_tv.vval.v_list;
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
        let val_tv = &raw const (*val_di).di_tv;

        match SPECIAL_KINDS[found] {
            SpecialKind::Nil => sink.conv_nil(tv),
            SpecialKind::Bool => {
                if (*val_tv).v_type != VAR_NUMBER {
                    return Ok(None);
                }
                sink.conv_bool(tv, (*val_tv).vval.v_number != 0);
            }
            SpecialKind::Integer => {
                // A list of four integers: a sign (nominally ±1), then the
                // number in three unsigned pieces, most significant first.
                // How many bits each piece really carries is not checked.
                if (*val_tv).v_type != VAR_LIST {
                    return Ok(None);
                }
                let val_list = (*val_tv).vval.v_list;
                if tv_list_len(val_list) != 4 {
                    return Ok(None);
                }
                let sign_li = tv_list_first(val_list);
                let sign: varnumber_T = (*sign_li).li_tv.vval.v_number;
                if (*sign_li).li_tv.v_type != VAR_NUMBER || sign == 0 {
                    return Ok(None);
                }
                let highest_bits_li = (*sign_li).li_next;
                let highest_bits: varnumber_T = (*highest_bits_li).li_tv.vval.v_number;
                if (*highest_bits_li).li_tv.v_type != VAR_NUMBER || highest_bits < 0 {
                    return Ok(None);
                }
                let high_bits_li = (*highest_bits_li).li_next;
                let high_bits: varnumber_T = (*high_bits_li).li_tv.vval.v_number;
                if (*high_bits_li).li_tv.v_type != VAR_NUMBER || high_bits < 0 {
                    return Ok(None);
                }
                let low_bits_li = tv_list_last(val_list);
                let low_bits: varnumber_T = (*low_bits_li).li_tv.vval.v_number;
                if (*low_bits_li).li_tv.v_type != VAR_NUMBER || low_bits < 0 {
                    return Ok(None);
                }
                let number =
                    ((highest_bits as u64) << 62) | ((high_bits as u64) << 31) | (low_bits as u64);
                if sign > 0 {
                    sink.conv_unsigned_number(tv, number);
                } else {
                    sink.conv_number(tv, number.wrapping_neg() as int64_t);
                }
            }
            SpecialKind::Float => {
                if (*val_tv).v_type != VAR_FLOAT {
                    return Ok(None);
                }
                return Ok(Some(sink.conv_float(tv, (*val_tv).vval.v_float)));
            }
            SpecialKind::String => {
                if (*val_tv).v_type != VAR_LIST {
                    return Ok(None);
                }
                let mut len: size_t = 0;
                let mut buf: *mut c_char = ptr::null_mut();
                if !encode_vim_list_to_buf((*val_tv).vval.v_list, &raw mut len, &raw mut buf) {
                    return Ok(None);
                }
                let flow = sink.conv_str_string(tv, buf, len);
                if flow == Flow::Go {
                    xfree(buf.cast());
                }
                return Ok(Some(flow));
            }
            SpecialKind::Array => {
                if (*val_tv).v_type != VAR_LIST {
                    return Ok(None);
                }
                let val_list = (*val_tv).vval.v_list;
                let saved_copyid = tv_list_copyid(val_list);
                {
                    let path = ConvPath { stack, objname };
                    match check_self_reference(
                        sink,
                        val_list.cast(),
                        &raw mut (*val_list).lv_copyID,
                        ConvType::List,
                        copyid,
                        &path,
                    ) {
                        Flow::Go => {}
                        other => return Ok(Some(other)),
                    }
                }
                match sink.conv_list_start(tv, tv_list_len(val_list)) {
                    Flow::Go => {}
                    other => return Ok(Some(other)),
                }
                debug_assert!(saved_copyid != copyid && saved_copyid != copyid - 1);
                stack.push(ConvFrame {
                    tv,
                    saved_copyid,
                    frame: Frame::List {
                        list: val_list,
                        li: tv_list_first(val_list),
                    },
                });
            }
            SpecialKind::Map => {
                if (*val_tv).v_type != VAR_LIST {
                    return Ok(None);
                }
                let val_list = (*val_tv).vval.v_list;
                if val_list.is_null() || tv_list_len(val_list) == 0 {
                    sink.conv_empty_dict(tv, None);
                    return Ok(Some(Flow::Go));
                }
                // Every item has to be a two-element list, or this is not a
                // map after all.
                let mut li = tv_list_first(val_list);
                while !li.is_null() {
                    let item = &raw const (*li).li_tv;
                    if (*item).v_type != VAR_LIST || tv_list_len((*item).vval.v_list) != 2 {
                        return Ok(None);
                    }
                    li = (*li).li_next;
                }
                let saved_copyid = tv_list_copyid(val_list);
                {
                    let path = ConvPath { stack, objname };
                    match check_self_reference(
                        sink,
                        val_list.cast(),
                        &raw mut (*val_list).lv_copyID,
                        ConvType::Pairs,
                        copyid,
                        &path,
                    ) {
                        Flow::Go => {}
                        other => return Ok(Some(other)),
                    }
                }
                match sink.conv_dict_start(tv, tv_list_len(val_list) as size_t) {
                    Flow::Go => {}
                    other => return Ok(Some(other)),
                }
                debug_assert!(saved_copyid != copyid && saved_copyid != copyid - 1);
                stack.push(ConvFrame {
                    tv,
                    saved_copyid,
                    frame: Frame::Pairs {
                        list: val_list,
                        li: tv_list_first(val_list),
                    },
                });
            }
            SpecialKind::Ext => {
                if (*val_tv).v_type != VAR_LIST {
                    return Ok(None);
                }
                let val_list = (*val_tv).vval.v_list;
                if tv_list_len(val_list) != 2 {
                    return Ok(None);
                }
                let first = tv_list_first(val_list);
                let last = tv_list_last(val_list);
                let ext_type = (*first).li_tv.vval.v_number;
                if (*first).li_tv.v_type != VAR_NUMBER
                    || ext_type > i8::MAX as varnumber_T
                    || ext_type < i8::MIN as varnumber_T
                    || (*last).li_tv.v_type != VAR_LIST
                {
                    return Ok(None);
                }
                let mut len: size_t = 0;
                let mut buf: *mut c_char = ptr::null_mut();
                if !encode_vim_list_to_buf((*last).li_tv.vval.v_list, &raw mut len, &raw mut buf) {
                    return Ok(None);
                }
                let flow = sink.conv_ext_string(tv, buf, len, ext_type as i8);
                if flow == Flow::Go {
                    xfree(buf.cast());
                }
                return Ok(Some(flow));
            }
        }
        Ok(Some(Flow::Go))
    }
}

/// Walk `top_tv` and hand every value to `sink`.
///
/// Returns whether the encode ran to completion; a sink that refuses a value
/// has already reported why.
///
/// # Safety
/// `top_tv` must point at a live typval and `objname` at a NUL-terminated
/// name used only for error messages.
pub(crate) unsafe fn encode_typval<S: TypvalSink>(
    sink: &mut S,
    top_tv: *mut typval_T,
    objname: *const c_char,
) -> bool {
    unsafe { walk(sink, top_tv, objname).is_ok() }
}

unsafe fn walk<S: TypvalSink>(
    sink: &mut S,
    top_tv: *mut typval_T,
    objname: *const c_char,
) -> Result<(), Refused> {
    unsafe {
        let copyid = get_copyID();
        let mut stack = ConvStack::new();
        convert_one_value(sink, &mut stack, top_tv, copyid, objname)?;

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
            let tv: *mut typval_T;
            match stack.get_mut(idx).frame {
                Frame::Dict {
                    dict,
                    dictp,
                    mut hi,
                    mut todo,
                } => {
                    if todo == 0 {
                        let saved_copyid = stack.get_mut(idx).saved_copyid;
                        stack.pop();
                        (*dict).dv_copyID = saved_copyid;
                        sink.conv_dict_end(cur_tv, Some(dictp));
                        continue;
                    }
                    if todo != (*dict).dv_hashtab.ht_used {
                        sink.conv_dict_between_items(cur_tv, Some(dictp));
                    }
                    while !(*hi).is_kept() {
                        hi = hi.add(1);
                    }
                    let di = tv_dict_hi2di(hi);
                    todo -= 1;
                    hi = hi.add(1);
                    if let Frame::Dict {
                        hi: hi_slot,
                        todo: todo_slot,
                        ..
                    } = &mut stack.get_mut(idx).frame
                    {
                        *hi_slot = hi;
                        *todo_slot = todo;
                    }
                    let key = tv_dict_item_key(di);
                    walk_hook!(sink.conv_str_string(ptr::null_mut(), key, strlen(key)));
                    sink.conv_dict_after_key(cur_tv, Some(dictp));
                    tv = &raw mut (*di).di_tv;
                }
                Frame::List { list, li } => {
                    if li.is_null() {
                        let saved_copyid = stack.get_mut(idx).saved_copyid;
                        stack.pop();
                        tv_list_set_copyid(list, saved_copyid);
                        sink.conv_list_end(cur_tv);
                        continue;
                    }
                    if li != tv_list_first(list) {
                        sink.conv_list_between_items(cur_tv);
                    }
                    tv = &raw mut (*li).li_tv;
                    if let Frame::List { li: li_slot, .. } = &mut stack.get_mut(idx).frame {
                        *li_slot = (*li).li_next;
                    }
                }
                Frame::Pairs { list, li } => {
                    if li.is_null() {
                        let saved_copyid = stack.get_mut(idx).saved_copyid;
                        stack.pop();
                        tv_list_set_copyid(list, saved_copyid);
                        sink.conv_dict_end(cur_tv, None);
                        continue;
                    }
                    if li != tv_list_first(list) {
                        sink.conv_dict_between_items(cur_tv, None);
                    }
                    let kv_pair = (*li).li_tv.vval.v_list;
                    let key = &raw mut (*tv_list_first(kv_pair)).li_tv;
                    walk_hook!(sink.special_dict_key_check(key));
                    // The key goes through the whole walk, and may itself be a
                    // container: this frame is not necessarily the top one by
                    // the time it returns, which is why the advance below is
                    // by index.  It also stays *un*advanced across the key, so
                    // that an error raised there names this pair's index.
                    convert_one_value(sink, &mut stack, key, copyid, objname)?;
                    sink.conv_dict_after_key(cur_tv, None);
                    tv = &raw mut (*tv_list_last(kv_pair)).li_tv;
                    if let Frame::Pairs { li: li_slot, .. } = &mut stack.get_mut(idx).frame {
                        *li_slot = (*li).li_next;
                    }
                }
                Frame::Partial { stage, pt } => {
                    match stage {
                        PartialStage::Args => {
                            let argc = if pt.is_null() { 0 } else { (*pt).pt_argc };
                            sink.conv_func_before_args(cur_tv, argc as ptrdiff_t);
                            if let Frame::Partial { stage: slot, .. } =
                                &mut stack.get_mut(idx).frame
                            {
                                *slot = PartialStage::Self_;
                            }
                            if !pt.is_null() && (*pt).pt_argc > 0 {
                                walk_hook!(sink.conv_list_start(ptr::null_mut(), (*pt).pt_argc));
                                stack.push(ConvFrame {
                                    tv: ptr::null_mut(),
                                    saved_copyid: copyid - 1,
                                    frame: Frame::PartialArgs {
                                        arg: (*pt).pt_argv,
                                        argv: (*pt).pt_argv,
                                        todo: (*pt).pt_argc as size_t,
                                    },
                                });
                            }
                        }
                        PartialStage::Self_ => {
                            if let Frame::Partial { stage: slot, .. } =
                                &mut stack.get_mut(idx).frame
                            {
                                *slot = PartialStage::End;
                            }
                            let dict = if pt.is_null() {
                                ptr::null_mut()
                            } else {
                                (*pt).pt_dict
                            };
                            if dict.is_null() {
                                sink.conv_func_before_self(cur_tv, -1);
                            } else {
                                let used = (*dict).dv_hashtab.ht_used;
                                sink.conv_func_before_self(cur_tv, used as ptrdiff_t);
                                let dictp = &raw mut (*pt).pt_dict;
                                if used == 0 {
                                    sink.conv_empty_dict(ptr::null_mut(), Some(dictp));
                                    continue;
                                }
                                let saved_copyid = (*dict).dv_copyID;
                                {
                                    let path = ConvPath {
                                        stack: &stack,
                                        objname,
                                    };
                                    walk_hook!(check_self_reference(
                                        sink,
                                        dict.cast(),
                                        &raw mut (*dict).dv_copyID,
                                        ConvType::Dict,
                                        copyid,
                                        &path,
                                    ));
                                }
                                walk_hook!(sink.conv_dict_start(ptr::null_mut(), used));
                                debug_assert!(saved_copyid != copyid && saved_copyid != copyid - 1);
                                stack.push(ConvFrame {
                                    tv: ptr::null_mut(),
                                    saved_copyid,
                                    frame: Frame::Dict {
                                        dict,
                                        dictp,
                                        hi: (*dict).dv_hashtab.ht_array,
                                        todo: used,
                                    },
                                });
                                walk_hook!(sink.conv_real_dict_after_start(
                                    ptr::null_mut(),
                                    Some(dictp),
                                    stack.last_mut()
                                ));
                            }
                        }
                        PartialStage::End => {
                            sink.conv_func_end(cur_tv, copyid);
                            stack.pop();
                        }
                    }
                    continue;
                }
                Frame::PartialArgs { arg, argv, todo } => {
                    if todo == 0 {
                        stack.pop();
                        sink.conv_list_end(ptr::null_mut());
                        continue;
                    }
                    if argv != arg {
                        sink.conv_list_between_items(ptr::null_mut());
                    }
                    tv = arg;
                    if let Frame::PartialArgs {
                        arg: arg_slot,
                        todo: todo_slot,
                        ..
                    } = &mut stack.get_mut(idx).frame
                    {
                        *arg_slot = arg.add(1);
                        *todo_slot = todo - 1;
                    }
                }
            }
            convert_one_value(sink, &mut stack, tv, copyid, objname)?;
        }
        Ok(())
    }
}
