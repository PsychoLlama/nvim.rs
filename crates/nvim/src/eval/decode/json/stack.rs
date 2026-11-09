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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::error::DecodeError;
use crate::eval::typval::DictItem;
use crate::memory::ThinCString;

use crate::eval::typval::{DictRef, ListRef, dict_find, tv_clear, tv_list_alloc};
use crate::types::{TypVal, VAR_STRING};

/// A container that is open: a handle on the value [`Decoder::stack`] holds.
#[derive(Clone, Debug)]
pub(crate) enum OpenContainer {
    List(ListRef),
    Dict(DictRef),
}

impl OpenContainer {
    pub(crate) fn is_dict(&self) -> bool {
        matches!(self, OpenContainer::Dict(_))
    }

    /// Whether `value` is this container.
    fn is(&self, value: &TypVal) -> bool {
        match self {
            OpenContainer::List(l) => value.list_shared().is_some_and(|v| v.ptr_eq(l)),
            OpenContainer::Dict(d) => value.dict_shared().is_some_and(|v| v.ptr_eq(d)),
        }
    }
}

/// What [`Decoder::containers`] records about one open container.
#[derive(Clone, Debug)]
pub(crate) struct Container {
    /// Where the container itself sits in [`Decoder::stack`].
    pub(crate) stack_index: usize,
    /// For a special map, the `_VAL` list its pairs go into.
    pub(crate) special_val: Option<ListRef>,
    /// The offset of the opening bracket, where a restart rewinds to.
    pub(crate) at: usize,
    pub(crate) container: OpenContainer,
}

#[derive(Debug)]
pub(crate) struct Value {
    pub(crate) is_special_string: bool,
    pub(crate) didcomma: bool,
    pub(crate) didcolon: bool,
    pub(crate) val: TypVal,
}

#[derive(Debug)]
pub(crate) struct Decoder<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) stack: Vec<Value>,
    pub(crate) containers: Vec<Container>,
    pub(crate) didcomma: bool,
    pub(crate) didcolon: bool,
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

    /// The document from `at` on, as a refusal quotes it: up to the first
    /// NUL, where upstream's `%s`/`%.*s` stopped reading.
    ///
    /// A document joined from a List of lines is not NUL-terminated, and
    /// upstream reads past its end here; the end of the document stops
    /// this.
    pub(crate) fn rest(&self, at: usize) -> ThinCString {
        let rest = &self.buf[at..];
        let shown = rest
            .iter()
            .position(|&b| b == 0)
            .map_or(rest, |nul| &rest[..nul]);
        ThinCString::from_bytes(shown)
    }

    pub(crate) fn value(&self, val: TypVal, is_special_string: bool) -> Value {
        Value {
            is_special_string,
            didcomma: self.didcomma,
            didcolon: self.didcolon,
            val,
        }
    }

    fn innermost(&self) -> Container {
        self.containers
            .last()
            .expect("finish_value is only reached inside a container")
            .clone()
    }

    pub(crate) fn finish_value(
        &mut self,
        mut obj: Value,
        at: &mut usize,
    ) -> Result<(), DecodeError> {
        if self.containers.is_empty() {
            self.stack.push(obj);
            return Ok(());
        }

        let mut last = self.innermost();
        let mut val_location = *at;
        if last.container.is(&obj.val) {
            self.containers.pop();
            val_location = last.at;
            last = self.innermost();
        }

        if let OpenContainer::List(list) = &last.container {
            if !list.items().is_empty() && !obj.didcomma {
                tv_clear(&mut obj.val);
                return Err(DecodeError::CommaBeforeItem(self.rest(val_location)));
            }
            debug_assert!(last.special_val.is_none());
            list.edit().push(obj.val);
            return Ok(());
        }

        if last.stack_index == self.stack.len().wrapping_sub(2) {
            if !obj.didcolon {
                tv_clear(&mut obj.val);
                return Err(DecodeError::ColonBeforeValue(self.rest(val_location)));
            }
            let key = self.stack.pop().expect("a dictionary key below the value");
            match (&last.special_val, &last.container) {
                (None, OpenContainer::Dict(dict)) => {
                    debug_assert!(!key.is_special_string);
                    let key_text = key
                        .val
                        .string_ref()
                        .expect("a plain key is a non-null String");
                    let mut obj_di = DictItem::boxed(key_text.as_bytes());
                    drop(key);
                    obj_di.di_tv = obj.val;
                    // A key already there sent the map down the special path
                    // before its value was scanned.
                    dict.edit()
                        .add_item(obj_di)
                        .unwrap_or_else(|_| unreachable!("a fresh key"));
                }
                (Some(special_val), _) => {
                    let mut kv_pair = tv_list_alloc(2);
                    kv_pair.push(key.val);
                    kv_pair.push(obj.val);
                    special_val.edit().push_list(Some(kv_pair));
                }
                (None, OpenContainer::List(_)) => unreachable!("a list was handled above"),
            }
            return Ok(());
        }

        if !obj.is_special_string && obj.val.v_type() != VAR_STRING {
            tv_clear(&mut obj.val);
            return Err(DecodeError::StringKey(self.rest(*at)));
        }
        let plain = match (&last.special_val, &last.container) {
            (None, OpenContainer::Dict(dict)) => Some(dict),
            _ => None,
        };
        if !obj.didcomma && plain.is_some_and(|dict| dict.dv_hashtab.ht_used != 0) {
            tv_clear(&mut obj.val);
            return Err(DecodeError::CommaBeforeKey(self.rest(val_location)));
        }

        if let Some(dict) = plain
            && (obj.is_special_string
                || obj
                    .val
                    .string_ref()
                    .is_none_or(|key| dict_find(Some(dict), key.as_bytes()).is_some()))
        {
            tv_clear(&mut obj.val);
            self.containers.pop();
            let reopened = &self.stack[last.stack_index];
            (self.didcomma, self.didcolon) = (reopened.didcomma, reopened.didcolon);
            while self.stack.len() > last.stack_index {
                let mut dropped = self.stack.pop().expect("the loop bound is the depth");
                tv_clear(&mut dropped.val);
            }
            *at = last.at;
            self.next_map_special = true;
            return Ok(());
        }

        self.stack.push(obj);
        Ok(())
    }
}
