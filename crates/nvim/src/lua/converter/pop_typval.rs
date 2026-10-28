//! `pop_typval()`: a Lua value as a Vimscript one.
//!
//! The Lua->typval direction, and the mirror of [`super::push`].  One
//! explicit stack of [`OpenValue`]s rather than recursion, because a Lua
//! table may nest arbitrarily deep and the conversion has to be able to
//! refuse (`E5100`) rather than overflow.  Tables are classified by
//! [`classify_table`] first, so a table's *shape* -- list, dictionary,
//! empty-dict, or a `{_TYPE, _VAL}` special -- is decided once.
//!
//! A frame holds the **container it is filling**, as the owning handle:
//! nothing here names a typval slot by address, so appending to a list
//! cannot invalidate a frame and a refusal releases the half-built tree by
//! dropping the stack.  The Lua stack carries the rest of the position -- a
//! container's table sits at a fixed absolute index for as long as its frame
//! is open, and for a dictionary the key `lua_next` handed back waits one
//! slot above it while its value is converted.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::semsg;
use crate::types::DictItem;
use core::ffi::CStr;

use super::pop::TURN_SLOTS;
use super::{VARNUMBER_MAX, VARNUMBER_MIN, classify_table};
use crate::eval::decode::{decode_create_map_special_dict, decode_string};
use crate::eval::typval::{DictRef, ListRef, tv_dict_alloc, tv_list_alloc};
use crate::eval::userfunc::register_luafunc;
use crate::global_cell::GlobalCell;
use crate::lua::ffi::LUA_NOREF;
use crate::lua::stack::{At, Kind, Stack};
use crate::message::emsg;
use crate::narrow::{float_as_i64, len_as_int};
use crate::os::cshim::gettext;
use crate::types::{
    ListItem, LuaRef, TypVal, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeNil, kSpecialVarNull, lua_Number, size_t,
};

/// Refused for a table that is neither a list nor a dictionary.
const E5100_MIXED_KEYS: &CStr = c"E5100: Cannot convert given Lua table: table should contain \
                                 either only integer keys or only string keys";
/// Refused for a Lua value with no Vimscript image at all.
const E5101_BAD_TYPE: &CStr = c"E5101: Cannot convert given Lua type";

/// One container the walk has opened and is still filling.
///
/// Each holds its own reference to the container, so the half-built tree is
/// reachable for the self-reference check and is released by dropping the
/// stack.  `table` is the container's Lua table.
enum OpenValue {
    /// A Lua array table becoming a list, `wanted` items long.
    List {
        list: ListRef,
        wanted: size_t,
        table: At,
    },
    /// A Lua string-keyed table becoming a dictionary.  The key `lua_next`
    /// handed back sits at `table + 1` until its value has been converted.
    Dict { dict: DictRef, table: At },
    /// A table with a NUL byte in one of its keys, which has no dictionary
    /// image: the `{_TYPE = map, _VAL = [[key, value], …]}` special form,
    /// whose `_VAL` list is what the pairs go into.
    Pairs {
        /// The special dictionary, which is what this frame answers.
        special: TypVal,
        /// `_VAL`, a reference of this frame's own on a list the dictionary
        /// owns.
        pairs: ListRef,
        table: At,
    },
}

impl OpenValue {
    /// Where this container's Lua table sits.
    fn table(&self) -> At {
        match self {
            OpenValue::List { table, .. }
            | OpenValue::Dict { table, .. }
            | OpenValue::Pairs { table, .. } => *table,
        }
    }

    /// The value a *self-reference* to this container converts to: another
    /// reference to what is being built, rather than a descent that would
    /// never end.
    ///
    /// A map special answers its **`_VAL` list**, not the dictionary around
    /// it -- upstream's frame holds `data.l.list` for everything that is not
    /// a plain dictionary, and this is the member it copies.
    fn reentered(&self) -> TypVal {
        match self {
            OpenValue::List { list, .. } => TypVal::list(Some(list.clone())),
            OpenValue::Dict { dict, .. } => TypVal::dict(Some(dict.clone())),
            OpenValue::Pairs { pairs, .. } => TypVal::list(Some(pairs.clone())),
        }
    }

    /// Put this container's next element on the Lua stack, or answer `false`
    /// once there is none left -- having popped whatever it was holding
    /// above its table.
    fn advance(&mut self, stack: &mut Stack) -> bool {
        match self {
            OpenValue::List {
                list,
                wanted,
                table,
            } => {
                if list.lv_items.len() >= *wanted {
                    return false;
                }
                stack.raw_get_index(*table, len_as_int(list.lv_items.len() + 1));
                true
            }
            OpenValue::Dict { table, .. } | OpenValue::Pairs { table, .. } => {
                // Skip any non-string key: those are not part of the
                // dictionary being built.
                while stack.next(*table) {
                    if stack.kind(At::from_top(2)) == Kind::String {
                        return true;
                    }
                    stack.pop(1);
                }
                false
            }
        }
    }

    /// Store `value`, the conversion of the element
    /// [`advance`](Self::advance) last handed out.
    ///
    /// The converted Lua value is already off the stack, so a dictionary's
    /// pending key is once again the top of it.
    fn deliver(&mut self, stack: &mut Stack, value: TypVal) {
        match self {
            OpenValue::List { list, .. } => list.lv_items.push(ListItem::new(value)),
            OpenValue::Dict { dict, table } => {
                let key = stack.to_bytes(table.above(1)).unwrap_or_default();
                let mut item = DictItem::boxed(key);
                item.di_tv = value;
                if dict.add_item(item).is_err() {
                    // A Lua table cannot hand the same key back twice,
                    // so the only refusal left is the funcref-name
                    // check, which a key from a table never trips.
                    std::process::abort();
                }
            }
            OpenValue::Pairs { pairs, table, .. } => {
                let bytes = stack.to_bytes(table.above(1)).unwrap_or_default();
                // The `_VAL` of a map special is a list of two-element
                // `[key, value]` lists, and the key is forced to a blob so
                // that the NUL that put it here survives.
                let key = decode_string(bytes, true);
                let mut pair = tv_list_alloc(2);
                pair.lv_items.push(ListItem::new(key));
                pair.lv_items.push(ListItem::new(value));
                pairs.lv_items.push(ListItem::new(TypVal::list(Some(pair))));
            }
        }
    }

    /// The finished value, once the container has no elements left.
    fn finish(self) -> TypVal {
        match self {
            OpenValue::List { list, .. } => TypVal::list(Some(list)),
            OpenValue::Dict { dict, .. } => TypVal::dict(Some(dict)),
            OpenValue::Pairs { special, .. } => special,
        }
    }
}

/// What converting the Lua value on top of the stack produced.
enum Converted {
    /// A finished value.  The Lua value it came from is still on the stack.
    Value(TypVal),
    /// A container was opened and pushed onto the walk's stack; its table
    /// stays on the Lua stack until the frame closes.
    Opened,
}

/// Convert the Lua value on top of the stack, or open the container it is.
///
/// `None` refuses, with the reason already reported.
fn convert_top(stack: &mut Stack, open: &mut Vec<OpenValue>) -> Option<Converted> {
    match stack.kind(At::TOP) {
        Kind::Nil => Some(Converted::Value(TypVal::Special(kSpecialVarNull))),
        Kind::Boolean => Some(Converted::Value(TypVal::Bool(
            if stack.to_boolean(At::TOP) {
                kBoolVarTrue
            } else {
                kBoolVarFalse
            },
        ))),
        Kind::String => {
            let bytes = stack.to_bytes(At::TOP).unwrap_or_default();
            // `decode_string` copies the Lua string's bytes.
            Some(Converted::Value(decode_string(bytes, false)))
        }
        Kind::Number => {
            let n = stack.to_number(At::TOP);
            let no_integer = n > VARNUMBER_MAX as lua_Number
                || n < VARNUMBER_MIN as lua_Number
                || float_as_i64(n) as lua_Number != n;
            Some(Converted::Value(if no_integer {
                TypVal::Float(n)
            } else {
                TypVal::Number(float_as_i64(n))
            }))
        }
        Kind::Table => convert_table(stack, open),
        Kind::Function => {
            // The function, registered under a name the value takes.
            let name = register_luafunc(stack.reference(At::TOP));
            Some(Converted::Value(TypVal::func(Some(name))))
        }
        Kind::Userdata => {
            // TODO(bfredl): check mt.__call and convert to a function?
            stack.push_vim_nil();
            let is_nil = stack.raw_equal(At::TOP, At::from_top(2));
            stack.pop(1);
            if is_nil {
                Some(Converted::Value(TypVal::Special(kSpecialVarNull)))
            } else {
                emsg(gettext(E5101_BAD_TYPE));
                None
            }
        }
        _ => {
            emsg(gettext(E5101_BAD_TYPE));
            None
        }
    }
}

/// Convert the Lua *table* on top of the stack: a value outright when it is
/// empty or a self-reference, otherwise a container the walk descends into.
fn convert_table(stack: &mut Stack, open: &mut Vec<OpenValue>) -> Option<Converted> {
    // Only worth tracking a table reference when the table has a metatable
    // of its own.
    let mut table_ref: LuaRef = LUA_NOREF;
    if stack.get_metatable(At::TOP) {
        stack.pop(1);
        // The table itself, which the reference now names.
        table_ref = stack.reference(At::TOP);
    }

    // The traversal leaves the table where it found it.
    let table_props = classify_table(stack);

    // A container already open on this table is this same table: share it
    // rather than descend forever.
    for container in open.iter() {
        if stack.raw_equal(At::TOP, container.table()) {
            return Some(Converted::Value(container.reentered()));
        }
    }

    // A frame opened below names this slot for as long as it lives.
    let table = stack.top();
    match table_props.type_0 {
        kObjectTypeArray => {
            let mut list = tv_list_alloc(table_props.maxidx.cast_signed());
            list.lua_table_ref = table_ref;
            if table_props.maxidx == 0 {
                return Some(Converted::Value(TypVal::list(Some(list))));
            }
            open.push(OpenValue::List {
                list,
                wanted: table_props.maxidx,
                table,
            });
            Some(Converted::Opened)
        }
        kObjectTypeDict => {
            if table_props.string_keys_num == 0 {
                let mut dict = tv_dict_alloc();
                dict.lua_table_ref = table_ref;
                return Some(Converted::Value(TypVal::dict(Some(dict))));
            }
            if table_props.has_string_with_nul {
                // A key with a NUL in it has no Vimscript dictionary image,
                // so the whole table becomes the `{_TYPE = map, _VAL = [[k,
                // v], …]}` special form and the walk descends into `_VAL`.
                let (special, mut pairs) =
                    decode_create_map_special_dict(table_props.string_keys_num.cast_signed());
                pairs.lua_table_ref = table_ref;
                open.push(OpenValue::Pairs {
                    special,
                    pairs,
                    table,
                });
            } else {
                let mut dict = tv_dict_alloc();
                dict.lua_table_ref = table_ref;
                open.push(OpenValue::Dict { dict, table });
            }
            // Seed `lua_next`.
            stack.push_nil();
            Some(Converted::Opened)
        }
        kObjectTypeFloat => Some(Converted::Value(TypVal::Float(table_props.val))),
        kObjectTypeNil => {
            emsg(gettext(E5100_MIXED_KEYS));
            None
        }
        _ => std::process::abort(),
    }
}

/// What the walk does next.
enum Step {
    /// Convert the Lua value on top of the stack.
    Convert,
    /// Hand a finished value to the container below it, or answer it.
    Deliver(TypVal),
    /// Ask the open container for its next element.
    Advance,
}

/// Room for one turn of the walk -- the next element, and what converting
/// it pushes -- or `E1502`.
///
/// Asked before the first conversion and before each advance, which is
/// where upstream's loop asks: an element is converted in the same turn that
/// pushed it, so the depth at which a deep table is refused (and the refusal
/// it gets) is upstream's.
#[inline]
fn reserve_turn(stack: &mut Stack) -> Option<()> {
    let need = stack.top().index() + TURN_SLOTS;
    if !stack.reserve(need) {
        semsg!("E1502: Lua failed to grow stack to {need}");
        return None;
    }
    Some(())
}

/// Walk the Lua value on top of the stack, leaving it there.
///
/// `None` refuses, with the reason already reported; whatever the walk had
/// built is released with `stack`.
fn walk(stack: &mut Stack, open: &mut Vec<OpenValue>) -> Option<TypVal> {
    reserve_turn(stack)?;
    let mut step = Step::Convert;
    loop {
        step = match step {
            Step::Convert => match convert_top(stack, open)? {
                Converted::Value(value) => {
                    stack.pop(1);
                    Step::Deliver(value)
                }
                Converted::Opened => Step::Advance,
            },
            Step::Deliver(value) => match open.last_mut() {
                None => return Some(value),
                Some(container) => {
                    container.deliver(stack, value);
                    Step::Advance
                }
            },
            Step::Advance => {
                reserve_turn(stack)?;
                let container = open.last_mut().expect("a container to advance");
                if container.advance(stack) {
                    Step::Convert
                } else {
                    let done = open.pop().expect("the container just advanced");
                    // Its table has served its purpose.
                    stack.pop(1);
                    Step::Deliver(done.finish())
                }
            }
        };
    }
}

/// The walk's container stack, kept between conversions as `pop_object`'s
/// is, so that converting a table does not allocate one afresh.
static OPEN_VALUES: GlobalCell<Vec<OpenValue>> = GlobalCell::new(Vec::new());

/// The Lua value on top of the stack as a Vimscript one, popping exactly one
/// value.
///
/// `None` when it will not convert, with the reason already reported.
pub fn pop_typval(stack: &mut Stack) -> Option<TypVal> {
    let initial_size = stack.top();
    let mut open = OPEN_VALUES.take();
    let converted = walk(stack, &mut open);
    if converted.is_none() {
        // Every container the walk had open is released here.
        open.clear();
        stack.pop(stack.top().index() - initial_size.index() + 1);
    }
    debug_assert!(open.is_empty());
    OPEN_VALUES.set(open);
    debug_assert!(stack.top().index() == initial_size.index() - 1);
    converted
}
