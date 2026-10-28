//! `pop_object()`: a Lua value as an API [`Object`].
//!
//! The same explicit-stack walk as [`super::pop_typval`], over the same
//! [`Stack`], producing api types instead of `TypVal`s.  It is a separate
//! walk because the two type systems disagree at the leaves: an `Object` has
//! no `VAR_SPECIAL` and carries `LuaRef`s for functions.
//!
//! A frame holds the **container it is filling**, as the owning value:
//! nothing here names an object slot by address, so nothing an append does
//! can invalidate a frame, and a refusal releases the half-built tree by
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

use core::ffi::CStr;

use super::pop::TURN_SLOTS;
use super::{API_INTEGER_MAX, API_INTEGER_MIN, classify_table};
use crate::global_cell::GlobalCell;
use crate::lua::stack::{At, Kind, Stack};
use crate::narrow::{float_as_i64, len_as_int};
use crate::types::{
    ApiDict, Array, DictKey, Error, Object, String_0, kObjectTypeArray, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeNil, lua_Number, size_t,
};

/// Refused for a Lua value with no api image.
const CANNOT_CONVERT: &CStr = c"Cannot convert given Lua type";

/// One container the walk has opened and is still filling.
///
/// `wanted` is how many elements the container takes, decided from the Lua
/// table's shape before the descent: a table with more than that in it has
/// the rest ignored, which is what the capacity test upstream writes says.
enum OpenValue {
    /// A Lua array table becoming an [`Array`].
    Array {
        array: Array,
        wanted: size_t,
        table: At,
    },
    /// A Lua string-keyed table becoming an [`ApiDict`].  The key `lua_next`
    /// handed back sits at `table + 1` until its value has been converted.
    Dict {
        dict: ApiDict,
        wanted: size_t,
        table: At,
    },
}

impl OpenValue {
    /// Put this container's next element on the Lua stack, or answer `false`
    /// once there is none left -- having popped whatever it was holding
    /// above its table.
    fn advance(&mut self, stack: &mut Stack) -> bool {
        match self {
            OpenValue::Array {
                array,
                wanted,
                table,
            } => {
                if array.len() >= *wanted {
                    return false;
                }
                stack.raw_get_index(*table, len_as_int(array.len()) + 1);
                true
            }
            OpenValue::Dict {
                dict,
                wanted,
                table,
            } => {
                if dict.len() >= *wanted {
                    // The key the last turn left above the table goes too.
                    stack.pop(1);
                    return false;
                }
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
    fn deliver(&mut self, stack: &mut Stack, value: Object) {
        match self {
            OpenValue::Array { array, .. } => array.push(value),
            OpenValue::Dict { dict, table, .. } => {
                // The key copies the Lua string's bytes -- into the entry
                // itself when they are short, which an api key nearly
                // always is.
                let key = DictKey::new(stack.to_bytes(table.above(1)).unwrap_or_default());
                dict.insert(key, value);
            }
        }
    }

    /// The finished value, once the container has no elements left.
    fn finish(self) -> Object {
        match self {
            OpenValue::Array { array, .. } => Object::array(array),
            OpenValue::Dict { dict, .. } => Object::dict(dict),
        }
    }
}

/// What converting the Lua value on top of the stack produced.
enum Converted {
    /// A finished value.  The Lua value it came from is still on the stack.
    Value(Object),
    /// A container was opened and pushed onto the walk's stack; its table
    /// stays on the Lua stack until the frame closes.
    Opened,
}

/// Convert the Lua value on top of the stack, or open the container it is.
///
/// With `as_ref`, a Lua function becomes a `LuaRef` rather than a refusal.
fn convert_top(
    stack: &mut Stack,
    as_ref: bool,
    open: &mut Vec<OpenValue>,
) -> Result<Converted, Error> {
    match stack.kind(At::TOP) {
        Kind::Nil => Ok(Converted::Value(Object::Nil)),
        Kind::Boolean => Ok(Converted::Value(Object::boolean(stack.to_boolean(At::TOP)))),
        Kind::String => {
            let bytes = stack.to_bytes(At::TOP).unwrap_or_default();
            Ok(Converted::Value(Object::string(String_0::from_bytes(
                bytes,
            ))))
        }
        Kind::Number => {
            let n = stack.to_number(At::TOP);
            let no_integer = n > API_INTEGER_MAX as lua_Number
                || n < API_INTEGER_MIN as lua_Number
                || float_as_i64(n) as lua_Number != n;
            Ok(Converted::Value(if no_integer {
                Object::float(n)
            } else {
                Object::integer(float_as_i64(n))
            }))
        }
        Kind::Table => convert_table(stack, open),
        Kind::Function if as_ref => {
            let func = stack.reference(At::TOP);
            Ok(Converted::Value(Object::luaref(func)))
        }
        Kind::Userdata => {
            // `vim.NIL` is the one userdata with an api image.
            stack.push_vim_nil();
            let is_nil = stack.raw_equal(At::TOP, At::from_top(2));
            stack.pop(1);
            if is_nil {
                Ok(Converted::Value(Object::Nil))
            } else {
                Err(Error::validation(c"Cannot convert userdata"))
            }
        }
        _ => Err(Error::validation(CANNOT_CONVERT)),
    }
}

/// Convert the Lua *table* on top of the stack: a value outright when it is
/// empty or carries a number, otherwise a container the walk descends into.
fn convert_table(stack: &mut Stack, open: &mut Vec<OpenValue>) -> Result<Converted, Error> {
    // The traversal leaves the table where it found it.
    let table_props = classify_table(stack);
    // A frame opened below names this slot for as long as it lives.
    let table = stack.top();
    match table_props.type_0 {
        kObjectTypeArray => {
            let wanted = table_props.maxidx;
            if wanted == 0 {
                return Ok(Converted::Value(Object::array(Array::EMPTY)));
            }
            open.push(OpenValue::Array {
                array: Array::with_capacity(wanted),
                wanted,
                table,
            });
            Ok(Converted::Opened)
        }
        kObjectTypeDict => {
            let wanted = table_props.string_keys_num;
            if wanted == 0 {
                return Ok(Converted::Value(Object::dict(ApiDict::EMPTY)));
            }
            open.push(OpenValue::Dict {
                dict: ApiDict::with_capacity(wanted),
                wanted,
                table,
            });
            // Seed `lua_next`.
            stack.push_nil();
            Ok(Converted::Opened)
        }
        kObjectTypeFloat => Ok(Converted::Value(Object::float(table_props.val))),
        kObjectTypeNil => Err(Error::validation(c"Cannot convert given Lua table")),
        _ => std::process::abort(),
    }
}

/// What the walk does next.
enum Step {
    /// Convert the Lua value on top of the stack.
    Convert,
    /// Hand a finished value to the container below it, or answer it.
    Deliver(Object),
    /// Ask the open container for its next element.
    Advance,
}

/// Room for the next key and value, and for the `vim.NIL` a userdata leaf
/// compares itself with. Asked once per element, which is where the pushes
/// are; the value the walk starts on is already on the stack.
#[inline]
fn reserve_turn(stack: &mut Stack) -> Result<(), Error> {
    if !stack.reserve(stack.top().index() + TURN_SLOTS) {
        return Err(Error::exception(c"Lua failed to grow stack"));
    }
    Ok(())
}

/// Walk the Lua value on top of the stack, leaving it there.
///
/// Whatever the walk had built when it refused is released with `stack`.
fn walk(stack: &mut Stack, as_ref: bool, open: &mut Vec<OpenValue>) -> Result<Object, Error> {
    let mut step = Step::Convert;
    loop {
        step = match step {
            Step::Convert => match convert_top(stack, as_ref, open)? {
                Converted::Value(value) => {
                    stack.pop(1);
                    Step::Deliver(value)
                }
                Converted::Opened => Step::Advance,
            },
            Step::Deliver(value) => match open.last_mut() {
                None => return Ok(value),
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

/// The walk's container stack, kept between conversions so that converting
/// a table does not allocate one afresh. Taken for the length of a walk and
/// put back empty; a conversion that started inside another (none does
/// today) would find it taken and start an empty one of its own.
static OPEN_VALUES: GlobalCell<Vec<OpenValue>> = GlobalCell::new(Vec::new());

/// Convert the Lua value on top of the stack, popping exactly one value.
///
/// With `as_ref`, a Lua function becomes a `LuaRef` rather than a refusal.
pub fn pop_object(stack: &mut Stack, as_ref: bool) -> Result<Object, Error> {
    let initial_size = stack.top();
    let mut open = OPEN_VALUES.take();
    let converted = walk(stack, as_ref, &mut open);
    if converted.is_err() {
        // Every container the walk had open is released here.
        open.clear();
        stack.pop(stack.top().index() - initial_size.index() + 1);
    }
    debug_assert!(open.is_empty());
    OPEN_VALUES.set(open);
    debug_assert!(stack.top().index() == initial_size.index() - 1);
    converted
}
