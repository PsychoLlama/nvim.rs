//! The walk `typval_encode.c.h` emits around its hooks: the two functions
//! upstream calls `_typval_encode_<sink>_convert_one_value` and
//! `encode_vim_to_<sink>`, written once against [`TypvalSink`].
//!
//! Every frame holds a borrow of the container it is suspended in, all of
//! them reached from the one value the walk was handed, so the walk is a
//! shared borrow of that value from start to finish: nothing a sink does can
//! reach the value through the walk, and the walk writes nothing to it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use super::{
    Container, ConvPath, ConvStack, ConvType, Flow, Frame, PartialStage, Refused, TypvalSink,
};
use crate::eval::encode::encode_vim_list_to_buf;
use crate::eval::partial_name;
use crate::eval::typval::{blob_bytes, dict_find, list_len};
use crate::eval::vars::eval_msgpack_type_lists;
use crate::message::internal_error;
use crate::types::{
    Dict, List, TypVal, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, kBoolVarFalse, kBoolVarTrue,
    kSpecialVarNull,
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

/// Tell the sink it has met `container` before, if some frame already holds
/// it: upstream's `_TYPVAL_ENCODE_CHECK_SELF_REFERENCE`.
///
/// Answers [`Flow::Go`] for a container the walk is not inside (upstream's
/// `NOTDONE`), and otherwise whatever the sink makes of the self-reference —
/// with its "handled" read as "this value is done", which is what the macro's
/// fall-through to `return OK` meant.
fn check_self_reference<S: TypvalSink>(
    sink: &mut S,
    stack: &ConvStack<'_>,
    container: Container<'_>,
    conv_type: ConvType,
    objname: &CStr,
) -> Flow {
    if !stack.holds(container) {
        return Flow::Go;
    }
    let path = ConvPath { stack, objname };
    match sink.conv_recurse(container, conv_type, &path) {
        Flow::Go => Flow::Stop,
        other => other,
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

/// A list's length as the `int` [`TypvalSink::conv_list_start`] is told.
fn list_count(list: &List) -> c_int {
    list_len(Some(list))
}

/// Convert one value, pushing any container it opens onto `stack`.
///
/// Only scalars are finished here; a list or dictionary is announced to the
/// sink and left for the walk to feed back one item at a time.
fn convert_one_value<'a, S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack<'a>,
    tv: &'a TypVal,
    objname: &CStr,
) -> Result<(), Refused> {
    sink.check_before();
    match tv.v_type() {
        VAR_STRING => {
            let bytes = tv.string_ref().map_or(&[][..], |s| s.as_bytes());
            item_hook!(sink.conv_string(bytes));
        }
        VAR_NUMBER => sink.conv_number(tv.number_or_zero()),
        VAR_FLOAT => item_hook!(sink.conv_float(tv.float_or_zero())),
        VAR_BLOB => sink.conv_blob(blob_bytes(tv.blob_ref())),
        VAR_FUNC => {
            let name = tv.func_name().map(|name| name.as_cstr());
            let path = ConvPath { stack, objname };
            item_hook!(sink.conv_func_start(name, c"", &path));
            sink.conv_func_before_args(0);
            sink.conv_func_before_self(None);
            sink.conv_func_end();
        }
        VAR_PARTIAL => {
            let partial = tv.partial_ref();
            let fun = partial.map(partial_name);
            // When using uf_name prepend "g:" for a global function.
            let prefix = if partial.is_some_and(|pt| {
                pt.pt_name.is_none()
                    && partial_name(pt)
                        .to_bytes()
                        .first()
                        .is_some_and(u8::is_ascii_uppercase)
            }) {
                c"g:"
            } else {
                c""
            };
            {
                let path = ConvPath { stack, objname };
                item_hook!(sink.conv_func_start(fun, prefix, &path));
            }
            stack.push(Frame::Partial {
                stage: PartialStage::Args,
                partial,
            });
        }
        VAR_LIST => match tv.list_ref().filter(|list| !list.items().is_empty()) {
            None => sink.conv_empty_list(),
            Some(list) => {
                let seen = Container::List(list);
                item_hook!(check_self_reference(
                    sink,
                    stack,
                    seen,
                    ConvType::List,
                    objname
                ));
                item_hook!(sink.conv_list_start(list_count(list)));
                stack.push(Frame::List { list, at: 0 });
            }
        },
        VAR_BOOL => {
            // Upstream switches over the two named values and ignores
            // anything else.
            let b = tv.as_bool().unwrap_or(kBoolVarFalse);
            if b == kBoolVarTrue || b == kBoolVarFalse {
                sink.conv_bool(b == kBoolVarTrue);
            }
        }
        VAR_SPECIAL => {
            if tv.as_special() == Some(kSpecialVarNull) {
                sink.conv_nil();
            }
        }
        VAR_DICT => match tv.dict_ref().filter(|dict| dict.dv_hashtab.ht_used != 0) {
            None => sink.conv_empty_dict(),
            Some(dict) => {
                if S::ALLOW_SPECIALS
                    && let Some(flow) = convert_special_dict(sink, stack, dict, objname)?
                {
                    item_hook!(flow);
                    return Ok(());
                }
                let seen = Container::Dict(dict);
                item_hook!(check_self_reference(
                    sink,
                    stack,
                    seen,
                    ConvType::Dict,
                    objname
                ));
                let used = dict.dv_hashtab.ht_used;
                item_hook!(sink.conv_dict_start(used));
                stack.push(Frame::Dict {
                    dict,
                    slot: 0,
                    todo: used,
                });
            }
        },
        VAR_UNKNOWN => {
            internal_error(S::CONVERT_FN_NAME);
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
fn convert_special_dict<'a, S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack<'a>,
    dict: &'a Dict,
    objname: &CStr,
) -> Result<Option<Flow>, Refused> {
    if dict.len() != 2 {
        return Ok(None);
    }
    let Some(type_di) = dict_find(Some(dict), b"_TYPE") else {
        return Ok(None);
    };
    if type_di.di_tv.v_type() != VAR_LIST {
        return Ok(None);
    }
    let Some(val_di) = dict_find(Some(dict), b"_VAL") else {
        return Ok(None);
    };
    let type_list = type_di
        .di_tv
        .list_ref()
        .map_or(::core::ptr::null(), ::core::ptr::from_ref);
    let found = eval_msgpack_type_lists
        .get()
        .iter()
        .position(|&l| ::core::ptr::eq(l, type_list));
    // Upstream runs the check a second time here, before it knows whether
    // this is a special dictionary at all.
    sink.check_before();
    let Some(found) = found else {
        return Ok(None);
    };
    let val = &val_di.di_tv;

    match SPECIAL_KINDS[found] {
        SpecialKind::Nil => sink.conv_nil(),
        SpecialKind::Bool => {
            if val.v_type() != VAR_NUMBER {
                return Ok(None);
            }
            sink.conv_bool(val.number_or_zero() != 0);
        }
        SpecialKind::Integer => {
            // A list of four integers: a sign (nominally ±1), then the
            // number in three unsigned pieces, most significant first.
            // How many bits each piece really carries is not checked.
            let Some(parts) = val
                .list_ref()
                .filter(|_| val.v_type() == VAR_LIST)
                .map(List::items)
                .filter(|parts| parts.len() == 4)
            else {
                return Ok(None);
            };
            if parts.iter().any(|li| li.li_tv.v_type() != VAR_NUMBER) {
                return Ok(None);
            }
            let [sign, highest_bits, high_bits, low_bits] =
                [0, 1, 2, 3].map(|i| parts[i].li_tv.number_or_zero());
            let (Ok(highest), Ok(high), Ok(low)) = (
                u64::try_from(highest_bits),
                u64::try_from(high_bits),
                u64::try_from(low_bits),
            ) else {
                return Ok(None);
            };
            if sign == 0 {
                return Ok(None);
            }
            // The pieces are not masked, so a wide one spills into its
            // neighbour's bits, as upstream's do.
            let number = (highest << 62) | (high << 31) | low;
            if sign > 0 {
                sink.conv_unsigned_number(number);
            } else {
                sink.conv_number(number.wrapping_neg().cast_signed());
            }
        }
        SpecialKind::Float => {
            if val.v_type() != VAR_FLOAT {
                return Ok(None);
            }
            return Ok(Some(sink.conv_float(val.float_or_zero())));
        }
        SpecialKind::String => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let Some(bytes) = encode_vim_list_to_buf(val.list_ref()) else {
                return Ok(None);
            };
            return Ok(Some(sink.conv_str_string(&bytes)));
        }
        SpecialKind::Array => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let Some(val_list) = val.list_ref() else {
                // `v:_null_list`: an empty array. Upstream reads the copyID
                // of the NULL list here and crashes; an empty list is what
                // every other reader makes of it.
                return Ok(Some(match sink.conv_list_start(0) {
                    Flow::Go => {
                        sink.conv_list_end();
                        Flow::Go
                    }
                    other => other,
                }));
            };
            let seen = Container::List(val_list);
            match check_self_reference(sink, stack, seen, ConvType::List, objname) {
                Flow::Go => {}
                other => return Ok(Some(other)),
            }
            match sink.conv_list_start(list_count(val_list)) {
                Flow::Go => {}
                other => return Ok(Some(other)),
            }
            stack.push(Frame::List {
                list: val_list,
                at: 0,
            });
        }
        SpecialKind::Map => {
            if val.v_type() != VAR_LIST {
                return Ok(None);
            }
            let Some(val_list) = val.list_ref().filter(|l| !l.items().is_empty()) else {
                sink.conv_empty_dict();
                return Ok(Some(Flow::Go));
            };
            // Every item has to be a two-element list, or this is not a
            // map after all.
            for li in val_list.items() {
                if li.li_tv.v_type() != VAR_LIST || list_len(li.li_tv.list_ref()) != 2 {
                    return Ok(None);
                }
            }
            let seen = Container::List(val_list);
            match check_self_reference(sink, stack, seen, ConvType::Pairs, objname) {
                Flow::Go => {}
                other => return Ok(Some(other)),
            }
            match sink.conv_dict_start(val_list.items().len()) {
                Flow::Go => {}
                other => return Ok(Some(other)),
            }
            stack.push(Frame::Pairs {
                list: val_list,
                at: 0,
            });
        }
        SpecialKind::Ext => {
            let Some([first, last]) = val
                .list_ref()
                .filter(|_| val.v_type() == VAR_LIST)
                .and_then(|l| <&[_; 2]>::try_from(l.items()).ok())
            else {
                return Ok(None);
            };
            let ext_type = first.li_tv.number_or_zero();
            let Ok(ext_type) = i8::try_from(ext_type) else {
                return Ok(None);
            };
            if first.li_tv.v_type() != VAR_NUMBER || last.li_tv.v_type() != VAR_LIST {
                return Ok(None);
            }
            let Some(bytes) = encode_vim_list_to_buf(last.li_tv.list_ref()) else {
                return Ok(None);
            };
            return Ok(Some(sink.conv_ext_string(&bytes, ext_type)));
        }
    }
    Ok(Some(Flow::Go))
}

/// Walk `top` and hand every value to `sink`.
///
/// Returns whether the encode ran to completion; a sink that refuses a value
/// has already reported why.
pub(crate) fn encode_typval<S: TypvalSink>(sink: &mut S, top: &TypVal, objname: &CStr) -> bool {
    let mut stack = ConvStack::new();
    walk(sink, &mut stack, top, objname).is_ok()
}

fn walk<'a, S: TypvalSink>(
    sink: &mut S,
    stack: &mut ConvStack<'a>,
    top: &'a TypVal,
    objname: &CStr,
) -> Result<(), Refused> {
    convert_one_value(sink, stack, top, objname)?;

    while let Some(idx) = stack.len().checked_sub(1) {
        // Upstream keeps a `MPConvStackVal *` into the stack across the
        // hooks and the nested key conversion below, which a `kvi_push`
        // may have reallocated out from under it (O-B14-5).  Here every
        // read and every advance goes through `stack` by index instead,
        // so the borrow checker is what guarantees no reference outlives
        // a push.
        let frame = *stack.get_mut(idx);
        // The value this pass hands to `convert_one_value`.
        let tv: &'a TypVal = match frame {
            Frame::Dict { dict, slot, todo } => {
                if todo == 0 {
                    stack.pop();
                    sink.conv_dict_end();
                    continue;
                }
                if todo != dict.dv_hashtab.ht_used {
                    sink.conv_dict_between_items();
                }
                let mut slot = slot;
                let item = loop {
                    if let Some(item) = dict.item_at(slot) {
                        break item;
                    }
                    slot += 1;
                };
                *stack.get_mut(idx) = Frame::Dict {
                    dict,
                    slot: slot + 1,
                    todo: todo - 1,
                };
                // The key is read as bytes the item already knows the length
                // of: every entry of every encode passes here.
                walk_hook!(sink.conv_dict_key(item.key()));
                sink.conv_dict_after_key();
                &item.di_tv
            }
            Frame::List { list, at } => {
                let Some(item) = list.items().get(at) else {
                    stack.pop();
                    sink.conv_list_end();
                    continue;
                };
                if at > 0 {
                    sink.conv_list_between_items();
                }
                *stack.get_mut(idx) = Frame::List { list, at: at + 1 };
                &item.li_tv
            }
            Frame::Pairs { list, at } => {
                let Some(item) = list.items().get(at) else {
                    stack.pop();
                    sink.conv_dict_end();
                    continue;
                };
                if at > 0 {
                    sink.conv_dict_between_items();
                }
                // A `[key, value]` pair, checked when the frame was pushed.
                let pair = item.li_tv.list_ref().map_or(&[][..], List::items);
                let (Some(key), Some(value)) = (pair.first(), pair.get(1)) else {
                    unreachable!("a special map's items are checked to be pairs");
                };
                walk_hook!(sink.special_dict_key_check(&key.li_tv));
                // The key goes through the whole walk, and may itself be a
                // container: this frame is not necessarily the top one by
                // the time it returns, which is why the advance below is
                // by index.  It also stays *un*advanced across the key, so
                // that an error raised there names this pair's index.
                convert_one_value(sink, stack, &key.li_tv, objname)?;
                sink.conv_dict_after_key();
                *stack.get_mut(idx) = Frame::Pairs { list, at: at + 1 };
                &value.li_tv
            }
            Frame::Partial { stage, partial } => {
                match stage {
                    PartialStage::Args => {
                        let argv = partial.map_or(&[][..], |pt| &pt.pt_argv[..]);
                        sink.conv_func_before_args(argv.len());
                        *stack.get_mut(idx) = Frame::Partial {
                            stage: PartialStage::Self_,
                            partial,
                        };
                        if !argv.is_empty() {
                            let argc = c_int::try_from(argv.len()).unwrap_or(c_int::MAX);
                            walk_hook!(sink.conv_list_start(argc));
                            stack.push(Frame::PartialArgs { argv, at: 0 });
                        }
                    }
                    PartialStage::Self_ => {
                        *stack.get_mut(idx) = Frame::Partial {
                            stage: PartialStage::End,
                            partial,
                        };
                        let Some(dict) = partial.and_then(|pt| pt.pt_dict.as_deref()) else {
                            sink.conv_func_before_self(None);
                            continue;
                        };
                        let used = dict.dv_hashtab.ht_used;
                        sink.conv_func_before_self(Some(used));
                        if used == 0 {
                            sink.conv_empty_dict();
                            continue;
                        }
                        let seen = Container::Dict(dict);
                        walk_hook!(check_self_reference(
                            sink,
                            stack,
                            seen,
                            ConvType::Dict,
                            objname
                        ));
                        walk_hook!(sink.conv_dict_start(used));
                        stack.push(Frame::Dict {
                            dict,
                            slot: 0,
                            todo: used,
                        });
                    }
                    PartialStage::End => {
                        sink.conv_func_end();
                        stack.pop();
                    }
                }
                continue;
            }
            Frame::PartialArgs { argv, at } => {
                let Some(arg) = argv.get(at) else {
                    stack.pop();
                    sink.conv_list_end();
                    continue;
                };
                if at > 0 {
                    sink.conv_list_between_items();
                }
                *stack.get_mut(idx) = Frame::PartialArgs { argv, at: at + 1 };
                arg
            }
        };
        convert_one_value(sink, stack, tv, objname)?;
    }
    Ok(())
}
