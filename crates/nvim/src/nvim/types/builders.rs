//! Constructing API values.
//!
//! [`Object`], [`Array`] and [`Dict`] are C layouts: a tag plus a union, and
//! a pointer/length/capacity triple. Building one literally takes a dozen
//! lines of struct syntax per element, which is why the transpiled call
//! sites run to hundreds of lines for a single `nvim_echo`.
//!
//! Two pieces here. [`Object`]'s constructors tag the union correctly by
//! construction. [`ArrayBuf`] and [`DictBuf`] own a fixed-size element
//! buffer and hand out an [`Array`]/[`Dict`] borrowing it — the safe
//! spelling of C's `MAXSIZE_TEMP_ARRAY`, for callees that read the value and
//! return (`rpc_send_event`, the API dispatchers) rather than take
//! ownership. Nothing here allocates or frees; strings keep whatever
//! ownership their creator gave them.

#![forbid(unsafe_code)]

use super::{Array, Dict, Float, Integer, KeyValuePair, Object, ObjectType, String_0, object_data};
use core::ffi::c_char;

// Private copies of the ObjectType discriminants: this module tags unions
// and every other module carries its own transpiled copy of these, so
// exporting them here would collide on glob import.
const TYPE_NIL: ObjectType = 0;
const TYPE_BOOLEAN: ObjectType = 1;
const TYPE_INTEGER: ObjectType = 2;
const TYPE_FLOAT: ObjectType = 3;
const TYPE_STRING: ObjectType = 4;
const TYPE_ARRAY: ObjectType = 5;
const TYPE_DICT: ObjectType = 6;

/// `text`'s bytes viewed as an API string. Borrowed, never freed — the
/// bytes are in the binary's read-only data.
pub const fn static_string(text: &'static str) -> String_0 {
    String_0 {
        data: text.as_ptr().cast::<c_char>().cast_mut(),
        size: text.len(),
    }
}

impl Object {
    pub const NIL: Self = Self {
        type_0: TYPE_NIL,
        data: object_data { boolean: false },
    };

    pub const fn boolean(value: bool) -> Self {
        Self {
            type_0: TYPE_BOOLEAN,
            data: object_data { boolean: value },
        }
    }

    pub const fn integer(value: Integer) -> Self {
        Self {
            type_0: TYPE_INTEGER,
            data: object_data { integer: value },
        }
    }

    pub const fn float(value: Float) -> Self {
        Self {
            type_0: TYPE_FLOAT,
            data: object_data { floating: value },
        }
    }

    /// An API string, keeping whatever ownership `value` already had.
    pub const fn string(value: String_0) -> Self {
        Self {
            type_0: TYPE_STRING,
            data: object_data { string: value },
        }
    }

    /// [`Object::string`] for a string literal. See [`static_string`].
    pub const fn literal(text: &'static str) -> Self {
        Self::string(static_string(text))
    }

    pub const fn array(value: Array) -> Self {
        Self {
            type_0: TYPE_ARRAY,
            data: object_data { array: value },
        }
    }

    pub const fn dict(value: Dict) -> Self {
        Self {
            type_0: TYPE_DICT,
            data: object_data { dict: value },
        }
    }
}

/// Storage for an [`Array`] of at most `N` elements, on the stack of
/// whoever declares it.
///
/// [`Self::array`] borrows the buffer, so the [`Array`] it returns is valid
/// only while the `ArrayBuf` lives and is not pushed to again — the borrow
/// checker enforces the second half, the first is why this is a local
/// variable at every call site.
pub struct ArrayBuf<const N: usize> {
    items: [Object; N],
    size: usize,
}

impl<const N: usize> ArrayBuf<N> {
    pub const fn new() -> Self {
        Self {
            items: [Object::NIL; N],
            size: 0,
        }
    }

    /// Appends `value`. Panics past `N` elements — the capacity is a
    /// property of the call site, not of anything a user can influence.
    pub fn push(&mut self, value: Object) -> &mut Self {
        assert!(self.size < N, "ArrayBuf overflow");
        self.items[self.size] = value;
        self.size += 1;
        self
    }

    /// The elements pushed so far, as an [`Array`] borrowing this buffer.
    pub fn array(&mut self) -> Array {
        Array {
            size: self.size,
            capacity: N,
            items: self.items.as_mut_ptr(),
        }
    }

    /// [`Self::array`], wrapped for nesting inside another builder.
    pub fn object(&mut self) -> Object {
        Object::array(self.array())
    }
}

impl<const N: usize> Default for ArrayBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage for a [`Dict`] of at most `N` entries. [`ArrayBuf`]'s rules
/// apply unchanged.
pub struct DictBuf<const N: usize> {
    items: [KeyValuePair; N],
    size: usize,
}

impl<const N: usize> DictBuf<N> {
    pub const fn new() -> Self {
        Self {
            items: [KeyValuePair {
                key: static_string(""),
                value: Object::NIL,
            }; N],
            size: 0,
        }
    }

    /// Appends `key: value`, with `key` a literal. Dict keys in generated
    /// calls always are; a computed key wants [`Self::insert_string`].
    pub fn insert(&mut self, key: &'static str, value: Object) -> &mut Self {
        self.insert_string(static_string(key), value)
    }

    /// [`Self::insert`], keeping whatever ownership `key` already had.
    pub fn insert_string(&mut self, key: String_0, value: Object) -> &mut Self {
        assert!(self.size < N, "DictBuf overflow");
        self.items[self.size] = KeyValuePair { key, value };
        self.size += 1;
        self
    }

    /// The entries inserted so far, as a [`Dict`] borrowing this buffer.
    pub fn dict(&mut self) -> Dict {
        Dict {
            size: self.size,
            capacity: N,
            items: self.items.as_mut_ptr(),
        }
    }

    /// [`Self::dict`], wrapped for nesting inside another builder.
    pub fn object(&mut self) -> Object {
        Object::dict(self.dict())
    }
}

impl<const N: usize> Default for DictBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The union payloads can only be read back through `unsafe`, which this
    // module forbids; what is checkable here is the bookkeeping the C side
    // reads first — tags, sizes, and that the value points at the buffer.

    #[test]
    fn array_borrows_the_buffer_and_reports_what_was_pushed() {
        let mut buf = ArrayBuf::<4>::new();
        buf.push(Object::integer(7));
        buf.push(Object::boolean(true));
        assert_eq!(buf.items[0].type_0, TYPE_INTEGER);
        assert_eq!(buf.items[1].type_0, TYPE_BOOLEAN);
        assert_eq!(buf.items[2].type_0, TYPE_NIL);
        let expected = buf.items.as_mut_ptr();
        let array = buf.array();
        assert_eq!((array.size, array.capacity), (2, 4));
        assert_eq!(array.items, expected);
    }

    #[test]
    fn dict_nests_in_an_array() {
        let mut opts = DictBuf::<1>::new();
        opts.insert("verbose", Object::boolean(true));
        assert_eq!(opts.items[0].key.size, "verbose".len());
        assert_eq!(opts.items[0].value.type_0, TYPE_BOOLEAN);

        let entry = opts.object();
        let mut args = ArrayBuf::<2>::new();
        args.push(Object::literal("hello"));
        args.push(entry);
        assert_eq!(args.items[0].type_0, TYPE_STRING);
        assert_eq!(args.items[1].type_0, TYPE_DICT);
        assert_eq!(args.array().size, 2);
    }

    #[test]
    #[should_panic(expected = "ArrayBuf overflow")]
    fn pushing_past_capacity_panics() {
        ArrayBuf::<1>::new()
            .push(Object::NIL)
            .push(Object::literal("one too many"));
    }
}
