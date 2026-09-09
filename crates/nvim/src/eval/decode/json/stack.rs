//! The decoder's state: two stacks, and the step that joins a value to the
//! container above it.
//!
//! [`Decoder::stack`] holds values not yet stored anywhere — including the
//! containers themselves, which sit there until they close — and
//! [`Decoder::containers`] says which of those are open and what each one is.
//! [`Decoder::finish_value`] is upstream's `json_decoder_pop`: every scanned
//! value goes through it, and it is where a plain dictionary discovers it has
//! to be re-parsed as a special map.
//!
//! Upstream spells the stacks as two `kvec_t`s and passes them, the parse
//! position and the three flag bytes as seven separate arguments to every
//! scanning function.  They are one struct here.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::message_fmt::{c_str, emsg_text, msg_bytes};
use crate::semsg;
use crate::tr_c;
use core::ffi::{CStr, c_char, c_int};

use crate::eval::typval::{
    tv_clear, tv_dict_add, tv_dict_find, tv_dict_item_alloc, tv_list_alloc, tv_list_append_list,
    tv_list_append_owned_tv, tv_list_len,
};
use crate::types::{Dict, List, TypVal, VAR_STRING};
use ::libc::abort;

/// Which kind of container is open, and the container itself.
///
/// A **borrowed** handle, not a value: the container's own typval sits on
/// [`Decoder::stack`] at [`Container::stack_index`] and is what holds the
/// reference.  Nothing reached through here releases anything.
#[derive(Copy, Clone)]
pub(crate) enum OpenContainer {
    /// A `[`, and the list it allocated.
    List(*mut List),
    /// A `{`, and the dictionary it allocated -- for a special map, the
    /// special dictionary, whose `_VAL` list is [`Container::special_val`].
    Dict(*mut Dict),
}

impl OpenContainer {
    /// Whether the open container is a dictionary.
    pub(crate) fn is_dict(self) -> bool {
        matches!(self, OpenContainer::Dict(_))
    }

    /// The dictionary, on a path that has already established it is one.
    pub(crate) fn dict(self) -> *mut Dict {
        match self {
            OpenContainer::Dict(d) => d,
            OpenContainer::List(_) => unreachable!("the open container is a dictionary here"),
        }
    }
}

/// One container the decoder is currently inside.
#[derive(Copy, Clone)]
pub(crate) struct Container {
    /// Where the container's own value sits in [`Decoder::stack`].
    pub(crate) stack_index: usize,
    /// The `_VAL` list of a special map, or NULL when the container is an
    /// ordinary list or dictionary.
    pub(crate) special_val: *mut List,
    /// Offset of the byte that opened it: what the restart rewinds to, and
    /// what an error inside it is reported against.
    pub(crate) at: usize,
    /// A handle on the container itself; see [`OpenContainer`].
    pub(crate) container: OpenContainer,
}

/// One decoded value not yet stored in any container.
///
/// The stack **owns** what it holds: a value leaves it by being stored in
/// its container, by being cleared, or -- for the document's one value --
/// by being written to the decoder's result.
pub(crate) struct Value {
    /// The value is a special dictionary wrapping a string, so it can be a
    /// dictionary *value* but never a key.
    pub(crate) is_special_string: bool,
    /// Whether a comma, or a colon, was the token before this value.  Each is
    /// recorded per value because the restart has to put them back.
    pub(crate) didcomma: bool,
    pub(crate) didcolon: bool,
    pub(crate) val: TypVal,
}

/// Everything the JSON scanner carries from byte to byte.
pub(crate) struct Decoder<'a> {
    /// The whole document.  Every position in the decoder is an offset into
    /// it, so an error can quote the rest of the input.
    pub(crate) buf: &'a [u8],
    pub(crate) stack: Vec<Value>,
    pub(crate) containers: Vec<Container>,
    pub(crate) didcomma: bool,
    pub(crate) didcolon: bool,
    /// Set when the dictionary being parsed turned out to need a special map.
    /// The scanner then resumes at the rewound position *without* advancing,
    /// so that the `{` is read a second time.
    pub(crate) next_map_special: bool,
}

impl<'a> Decoder<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Decoder {
            buf,
            stack: Vec::new(),
            containers: Vec::new(),
            didcomma: false,
            didcolon: false,
            next_map_special: false,
        }
    }

    /// `semsg(_(fmt), LENP(p, e))`: report `fmt` with the document from `at`
    /// onwards as its `%.*s` argument.
    ///
    /// The bytes go out as they came in — an invalid UTF-8 sequence is quoted
    /// verbatim, which is what several of these messages are about.
    pub(crate) fn emsg_rest(&self, fmt: &'static CStr, at: usize) {
        let rest = &self.buf[at..];
        // The bytes go out as they came in; `%.*s` reads at most the length
        // given, which is `rest`'s own.
        let (len, at) = (rest.len() as c_int, msg_bytes(rest));
        emsg_text(tr_c!(fmt, len, at));
    }

    /// Upstream's `OBJ()`: a scanned value, tagged with the punctuation that
    /// preceded it.
    pub(crate) fn value(&self, val: TypVal, is_special_string: bool) -> Value {
        Value {
            is_special_string,
            didcomma: self.didcomma,
            didcolon: self.didcolon,
            val,
        }
    }

    /// The innermost open container.
    ///
    /// Every caller has just checked that there is one, or has just closed a
    /// container the grammar guarantees is nested inside another — a
    /// top-level container never reaches [`Self::finish_value`], because the
    /// scanner ends the document instead.  Upstream reads `kv_last` of an
    /// empty vector here rather than saying so.
    fn innermost(&self) -> Container {
        *self
            .containers
            .last()
            .expect("finish_value is only reached inside a container")
    }

    /// Store a finished value: upstream's `json_decoder_pop`.
    ///
    /// `at` is the parse position, used for error text and rewound when the
    /// container has to be restarted as a special map — in which case
    /// [`Self::next_map_special`] is set and the caller must resume the scan
    /// without advancing.  Answers `false` after reporting an error, having
    /// cleared `obj`.
    ///
    /// # Safety
    /// `obj` owns its value and `at` indexes [`Self::buf`].
    pub(crate) unsafe fn finish_value(&mut self, mut obj: Value, at: &mut usize) -> bool {
        if self.containers.is_empty() {
            self.stack.push(obj);
            return true;
        }

        let mut last = self.innermost();
        let mut val_location = *at;
        // The value being stored *is* the container on top: it has just
        // closed, so it belongs to the one below, and the error position
        // to report against is where it opened.
        // Upstream reads `vval.v_list` for both cases, the two members
        // having the same size and offset; the handle's kind picks the
        // reader here, so a Dict container cannot compare two NULLs and
        // match.
        let is_the_container = match last.container {
            OpenContainer::List(l) => obj.val.as_list() == Some(l),
            OpenContainer::Dict(d) => obj.val.as_dict() == Some(d),
        };
        if is_the_container {
            self.containers.pop();
            val_location = last.at;
            last = self.innermost();
        }

        if let OpenContainer::List(list) = last.container {
            if unsafe { tv_list_len(list) } != 0 && !obj.didcomma {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(self.buf[val_location..].as_ptr() as *const c_char) };
                semsg!("E474: Expected comma before list item: {arg0}");
                unsafe { tv_clear(&raw mut obj.val) };
                return false;
            }
            debug_assert!(last.special_val.is_null());
            unsafe { tv_list_append_owned_tv(list, obj.val) };
            return true;
        }

        // A dictionary, with its key already on the stack: this is the
        // value that goes with it.
        if last.stack_index == self.stack.len().wrapping_sub(2) {
            if !obj.didcolon {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(self.buf[val_location..].as_ptr() as *const c_char) };
                semsg!("E474: Expected colon before dictionary value: {arg0}");
                unsafe { tv_clear(&raw mut obj.val) };
                return false;
            }
            let mut key = self.stack.pop().expect("a dictionary key below the value");
            if last.special_val.is_null() {
                // A key that could not be a `Dict` key has already sent
                // this container down the special-map path below.
                debug_assert!(!(key.is_special_string || key.val.string_or_null().is_null()));
                let obj_di = unsafe { tv_dict_item_alloc(key.val.string_or_null()) };
                unsafe { tv_clear(&raw mut key.val) };
                if unsafe { tv_dict_add(last.container.dict(), obj_di) }.is_err() {
                    unsafe { abort() };
                }
                unsafe { (*obj_di).di_tv = obj.val };
            } else {
                let kv_pair = tv_list_alloc(2);
                unsafe { tv_list_append_list(last.special_val, kv_pair) };
                unsafe { tv_list_append_owned_tv(kv_pair, key.val) };
                unsafe { tv_list_append_owned_tv(kv_pair, obj.val) };
            }
            return true;
        }

        // A dictionary with nothing pending: this value is a key.
        if !obj.is_special_string && obj.val.v_type() != VAR_STRING {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(self.buf[*at..].as_ptr() as *const c_char) };
            semsg!("E474: Expected string key: {arg0}");
            unsafe { tv_clear(&raw mut obj.val) };
            return false;
        }
        if !obj.didcomma
            && last.special_val.is_null()
            && unsafe { (*last.container.dict()).dv_hashtab.ht_used } != 0
        {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(self.buf[val_location..].as_ptr() as *const c_char) };
            semsg!("E474: Expected comma before dictionary key: {arg0}");
            unsafe { tv_clear(&raw mut obj.val) };
            return false;
        }

        // Three kinds of key a `Dict` cannot hold: one that is itself a
        // special dictionary, one carrying an embedded NUL (decoded as a
        // blob, so `v_string` is NULL), and a duplicate.  Any of them
        // sends the whole container back to be re-parsed as a special
        // map, which can hold every one of them.
        if last.special_val.is_null()
            && (obj.is_special_string
                || obj.val.string_or_null().is_null()
                || !unsafe { tv_dict_find(last.container.dict(), obj.val.string_or_null(), -1) }
                    .is_null())
        {
            unsafe { tv_clear(&raw mut obj.val) };
            // Rewind to the `{` and reopen it as a special map.
            // Everything decoded inside it is dropped — the container's
            // own value included, which frees the half-filled dictionary.
            self.containers.pop();
            let reopened = &self.stack[last.stack_index];
            (self.didcomma, self.didcolon) = (reopened.didcomma, reopened.didcolon);
            while self.stack.len() > last.stack_index {
                let mut dropped = self.stack.pop().expect("the loop bound is the depth");
                unsafe { tv_clear(&raw mut dropped.val) };
            }
            *at = last.at;
            self.next_map_special = true;
            return true;
        }

        self.stack.push(obj);
        true
    }
}
