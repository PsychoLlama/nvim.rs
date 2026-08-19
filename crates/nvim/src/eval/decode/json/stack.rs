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

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int};

use crate::eval::typval::{
    tv_clear, tv_dict_add, tv_dict_find, tv_dict_item_alloc, tv_list_alloc, tv_list_append_list,
    tv_list_append_owned_tv, tv_list_len,
};
use crate::os::cshim::gettext;
use crate::types::{FAIL, VAR_LIST, VAR_STRING, list_T, typval_T};
use ::libc::abort;

const E474_COMMA_BEFORE_LIST_ITEM: &CStr = c"E474: Expected comma before list item: %s";
const E474_COLON_BEFORE_DICT_VALUE: &CStr = c"E474: Expected colon before dictionary value: %s";
const E474_STRING_KEY: &CStr = c"E474: Expected string key: %s";
const E474_COMMA_BEFORE_DICT_KEY: &CStr = c"E474: Expected comma before dictionary key: %s";

/// One container the decoder is currently inside.
#[derive(Copy, Clone)]
pub(crate) struct Container {
    /// Where the container's own value sits in [`Decoder::stack`].
    pub(crate) stack_index: usize,
    /// The `_VAL` list of a special map, or NULL when the container is an
    /// ordinary list or dictionary.
    pub(crate) special_val: *mut list_T,
    /// Offset of the byte that opened it: what the restart rewinds to, and
    /// what an error inside it is reported against.
    pub(crate) at: usize,
    /// The container's own value: `VAR_LIST` for `[`, `VAR_DICT` for `{`
    /// — a special map's is the special dictionary, with the `_VAL` list in
    /// [`Self::special_val`].
    pub(crate) container: typval_T,
}

/// One decoded value not yet stored in any container.
#[derive(Copy, Clone)]
pub(crate) struct Value {
    /// The value is a special dictionary wrapping a string, so it can be a
    /// dictionary *value* but never a key.
    pub(crate) is_special_string: bool,
    /// Whether a comma, or a colon, was the token before this value.  Each is
    /// recorded per value because the restart has to put them back.
    pub(crate) didcomma: bool,
    pub(crate) didcolon: bool,
    pub(crate) val: typval_T,
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
    pub(crate) fn emsg_rest(&self, fmt: &CStr, at: usize) {
        let rest = &self.buf[at..];
        // SAFETY: `rest` outlives the call and `semsg` copies what it keeps;
        // `%.*s` reads at most the length given.
        unsafe {
            semsg_c!(
                gettext(fmt.as_ptr()),
                rest.len() as c_int,
                rest.as_ptr() as *const c_char,
            )
        };
    }

    /// Upstream's `OBJ()`: a scanned value, tagged with the punctuation that
    /// preceded it.
    pub(crate) fn value(&self, val: typval_T, is_special_string: bool) -> Value {
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
        unsafe {
            if self.containers.is_empty() {
                self.stack.push(obj);
                return true;
            }

            let mut last = self.innermost();
            let mut val_location = *at;
            // The value being stored *is* the container on top: it has just
            // closed, so it belongs to the one below, and the error position
            // to report against is where it opened.
            if obj.val.v_type == last.container.v_type
                // vval.v_list and vval.v_dict have the same size and offset.
                && obj.val.vval.v_list == last.container.vval.v_list
            {
                self.containers.pop();
                val_location = last.at;
                last = self.innermost();
            }

            if last.container.v_type == VAR_LIST {
                if tv_list_len(last.container.vval.v_list) != 0 && !obj.didcomma {
                    semsg_c!(
                        gettext(E474_COMMA_BEFORE_LIST_ITEM.as_ptr()),
                        self.buf[val_location..].as_ptr() as *const c_char,
                    );
                    tv_clear(&raw mut obj.val);
                    return false;
                }
                debug_assert!(last.special_val.is_null());
                tv_list_append_owned_tv(last.container.vval.v_list, obj.val);
                return true;
            }

            // A dictionary, with its key already on the stack: this is the
            // value that goes with it.
            if last.stack_index == self.stack.len().wrapping_sub(2) {
                if !obj.didcolon {
                    semsg_c!(
                        gettext(E474_COLON_BEFORE_DICT_VALUE.as_ptr()),
                        self.buf[val_location..].as_ptr() as *const c_char,
                    );
                    tv_clear(&raw mut obj.val);
                    return false;
                }
                let mut key = self.stack.pop().expect("a dictionary key below the value");
                if last.special_val.is_null() {
                    // A key that could not be a `dict_T` key has already sent
                    // this container down the special-map path below.
                    debug_assert!(!(key.is_special_string || key.val.vval.v_string.is_null()));
                    let obj_di = tv_dict_item_alloc(key.val.vval.v_string);
                    tv_clear(&raw mut key.val);
                    if tv_dict_add(last.container.vval.v_dict, obj_di) == FAIL {
                        abort();
                    }
                    (*obj_di).di_tv = obj.val;
                } else {
                    let kv_pair = tv_list_alloc(2);
                    tv_list_append_list(last.special_val, kv_pair);
                    tv_list_append_owned_tv(kv_pair, key.val);
                    tv_list_append_owned_tv(kv_pair, obj.val);
                }
                return true;
            }

            // A dictionary with nothing pending: this value is a key.
            if !obj.is_special_string && obj.val.v_type != VAR_STRING {
                semsg_c!(
                    gettext(E474_STRING_KEY.as_ptr()),
                    self.buf[*at..].as_ptr() as *const c_char,
                );
                tv_clear(&raw mut obj.val);
                return false;
            }
            if !obj.didcomma
                && last.special_val.is_null()
                && (*last.container.vval.v_dict).dv_hashtab.ht_used != 0
            {
                semsg_c!(
                    gettext(E474_COMMA_BEFORE_DICT_KEY.as_ptr()),
                    self.buf[val_location..].as_ptr() as *const c_char,
                );
                tv_clear(&raw mut obj.val);
                return false;
            }

            // Three kinds of key a `dict_T` cannot hold: one that is itself a
            // special dictionary, one carrying an embedded NUL (decoded as a
            // blob, so `v_string` is NULL), and a duplicate.  Any of them
            // sends the whole container back to be re-parsed as a special
            // map, which can hold every one of them.
            if last.special_val.is_null()
                && (obj.is_special_string
                    || obj.val.vval.v_string.is_null()
                    || !tv_dict_find(last.container.vval.v_dict, obj.val.vval.v_string, -1)
                        .is_null())
            {
                tv_clear(&raw mut obj.val);
                // Rewind to the `{` and reopen it as a special map.
                // Everything decoded inside it is dropped — the container's
                // own value included, which frees the half-filled dictionary.
                self.containers.pop();
                let reopened = self.stack[last.stack_index];
                while self.stack.len() > last.stack_index {
                    let mut dropped = self.stack.pop().expect("the loop bound is the depth");
                    tv_clear(&raw mut dropped.val);
                }
                *at = last.at;
                self.didcomma = reopened.didcomma;
                self.didcolon = reopened.didcolon;
                self.next_map_special = true;
                return true;
            }

            self.stack.push(obj);
            true
        }
    }
}
