//! `json_decode()`: JSON text into a `TypVal`.
//!
//! One pass over the document, dispatching on the byte under the cursor.  The
//! two values with a syntax of their own go to [`scan`]; everything else is
//! punctuation, a keyword, or the open bracket of a container.  Values are
//! never returned upward — each one is handed straight to
//! [`stack::Decoder::finish_value`], which knows where it belongs.
//!
//! The one thing that is not a single pass is a **dictionary that turns out
//! to need a special map**: a key that is not a plain non-empty string, or a
//! duplicate one, sends the scanner back to the `{` with
//! `Decoder::next_map_special` set, and everything decoded inside it so far is
//! thrown away.  That is why every arm that stores a value has to check the
//! flag and resume *without* advancing.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::decode_create_map_special_dict;
use crate::eval::typval::{ListRef, tv_clear, tv_dict_alloc, tv_list_alloc};
use crate::message_fmt::emsg_text;
use crate::os::libc::ptrdiff_t;
use crate::types::{Failed, TypVal, kBoolVarFalse, kBoolVarTrue, kListLenMayKnow, kSpecialVarNull};

mod error;
mod scan;
mod stack;

use self::error::DecodeError;

use self::scan::{parse_json_number, parse_json_string};
use self::stack::{Container, Decoder, OpenContainer};

/// The ASCII bytes JSON's grammar names.  `BS`/`FF` are only reachable
/// through an escape; the other three are the whitespace between tokens.
pub(crate) const BS: u8 = 0x08;
pub(crate) const TAB: u8 = 0x09;
pub(crate) const NL: u8 = 0x0a;
pub(crate) const FF: u8 = 0x0c;
pub(crate) const CAR: u8 = 0x0d;

const NULL_TV: TypVal = TypVal::Special(kSpecialVarNull);

const fn bool_tv(value: bool) -> TypVal {
    TypVal::Bool(if value { kBoolVarTrue } else { kBoolVarFalse })
}

/// Decode `buf_len` bytes of JSON, assumed UTF-8, into `result`.
///
/// Answers `Err` once the refusal has been said: the scanner, the string
/// and number scanners and the value stack answer a `DecodeError`, and
/// this is where it is reported. Every value on the decoder's stack is owned
/// by it until it is stored, and the failure path clears whatever is left.
pub fn json_decode_string(bytes: &[u8], result: &mut TypVal) -> Result<(), Failed> {
    let buf_len = bytes.len();

    let mut p = 0;
    while p < buf_len && matches!(bytes[p], b' ' | TAB | NL | CAR) {
        p += 1;
    }
    if p == buf_len {
        emsg_text(DecodeError::BlankString.to_string());
        return Err(Failed);
    }

    result.overwrite(TypVal::Unknown);
    let mut dec = Decoder::new(bytes);
    let mut ret = Ok(());
    // Whether a container holds nothing yet, which is what makes a comma
    // a leading one.
    let is_empty = |c: &Container| match (&c.special_val, &c.container) {
        (Some(pairs), _) => pairs.items().is_empty(),
        (None, OpenContainer::Dict(d)) => d.dv_hashtab.ht_used == 0,
        (None, OpenContainer::List(l)) => l.items().is_empty(),
    };
    'done: {
        let refusal = 'fail: {
            'scan: while p < buf_len {
                // Only `{` may be reached with the flag still set: it is
                // the byte the restart rewound to.
                debug_assert!(bytes[p] == b'{' || !dec.next_map_special);
                match bytes[p] {
                    b'}' | b']' => {
                        let Some(last) = dec.containers.last() else {
                            break 'fail DecodeError::NoContainer(dec.rest(p));
                        };
                        if bytes[p] == b'}' && !last.container.is_dict() {
                            break 'fail DecodeError::CloseListCurly(dec.rest(p));
                        } else if bytes[p] == b']' && last.container.is_dict() {
                            break 'fail DecodeError::CloseDictSquare(dec.rest(p));
                        } else if dec.didcomma {
                            break 'fail DecodeError::TrailingComma(dec.rest(p));
                        } else if dec.didcolon {
                            break 'fail DecodeError::ValueAfterColon(dec.rest(p));
                        } else if last.stack_index != dec.stack.len() - 1 {
                            debug_assert!(last.stack_index < dec.stack.len() - 1);
                            break 'fail DecodeError::ExpectedValue(dec.rest(p));
                        }
                        if dec.stack.len() == 1 {
                            // The outermost container just closed: the
                            // document is done.
                            p += 1;
                            dec.containers.pop();
                            break 'scan;
                        }
                        let closed = dec.stack.pop().expect("the container itself");
                        if let Err(refusal) = dec.finish_value(closed, &mut p) {
                            break 'fail refusal;
                        }
                        // A container is never a dictionary key, so it
                        // cannot have triggered the restart.
                        debug_assert!(!dec.next_map_special);
                    }
                    b',' => {
                        let Some(last) = dec.containers.last() else {
                            break 'fail DecodeError::CommaOutside(dec.rest(p));
                        };
                        if dec.didcomma {
                            break 'fail DecodeError::DuplicateComma(dec.rest(p));
                        } else if dec.didcolon {
                            break 'fail DecodeError::CommaAfterColon(dec.rest(p));
                        } else if last.container.is_dict()
                            && last.stack_index != dec.stack.len() - 1
                        {
                            break 'fail DecodeError::CommaForColon(dec.rest(p));
                        } else if is_empty(last) {
                            break 'fail DecodeError::LeadingComma(dec.rest(p));
                        }
                        dec.didcomma = true;
                        p += 1;
                        continue;
                    }
                    b':' => {
                        let Some(last) = dec.containers.last() else {
                            break 'fail DecodeError::ColonOutside(dec.rest(p));
                        };
                        if !last.container.is_dict() {
                            break 'fail DecodeError::ColonNotInDict(dec.rest(p));
                        } else if last.stack_index != dec.stack.len().wrapping_sub(2) {
                            break 'fail DecodeError::UnexpectedColon(dec.rest(p));
                        } else if dec.didcomma {
                            break 'fail DecodeError::ColonAfterComma(dec.rest(p));
                        } else if dec.didcolon {
                            break 'fail DecodeError::DuplicateColon(dec.rest(p));
                        }
                        dec.didcolon = true;
                        p += 1;
                        continue;
                    }
                    b' ' | TAB | NL | CAR => {
                        p += 1;
                        continue;
                    }
                    b'n' => {
                        if p + 3 >= buf_len || &bytes[p + 1..p + 4] != b"ull" {
                            break 'fail DecodeError::ExpectedNull(dec.rest(p));
                        }
                        p += 3;
                        let value = dec.value(NULL_TV, false);
                        if let Err(refusal) = dec.finish_value(value, &mut p) {
                            break 'fail refusal;
                        }
                        if dec.next_map_special {
                            continue;
                        }
                    }
                    b't' => {
                        if p + 3 >= buf_len || &bytes[p + 1..p + 4] != b"rue" {
                            break 'fail DecodeError::ExpectedTrue(dec.rest(p));
                        }
                        p += 3;
                        let value = dec.value(bool_tv(true), false);
                        if let Err(refusal) = dec.finish_value(value, &mut p) {
                            break 'fail refusal;
                        }
                        if dec.next_map_special {
                            continue;
                        }
                    }
                    b'f' => {
                        if p + 4 >= buf_len || &bytes[p + 1..p + 5] != b"alse" {
                            break 'fail DecodeError::ExpectedFalse(dec.rest(p));
                        }
                        p += 4;
                        let value = dec.value(bool_tv(false), false);
                        if let Err(refusal) = dec.finish_value(value, &mut p) {
                            break 'fail refusal;
                        }
                        if dec.next_map_special {
                            continue;
                        }
                    }
                    b'"' => {
                        if let Err(refusal) = parse_json_string(&mut dec, &mut p) {
                            break 'fail refusal;
                        }
                        if dec.next_map_special {
                            continue;
                        }
                    }
                    b'-' | b'0'..=b'9' => {
                        if let Err(refusal) = parse_json_number(&mut dec, &mut p) {
                            break 'fail refusal;
                        }
                        if dec.next_map_special {
                            continue;
                        }
                    }
                    b'[' => {
                        let list = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
                        dec.open(OpenContainer::List(list), None, p);
                    }
                    b'{' => {
                        if dec.next_map_special {
                            dec.next_map_special = false;
                            let len = kListLenMayKnow as ptrdiff_t;
                            let (special, pairs) = decode_create_map_special_dict(len);
                            let dict = special.dict_shared().expect("a special map").clone();
                            dec.open_value(OpenContainer::Dict(dict), Some(pairs), special, p);
                        } else {
                            dec.open(OpenContainer::Dict(tv_dict_alloc()), None, p);
                        }
                    }
                    _ => {
                        break 'fail DecodeError::UnidentifiedByte(dec.rest(p));
                    }
                }
                dec.didcomma = false;
                dec.didcolon = false;
                p += 1;
                if dec.containers.is_empty() {
                    break 'scan;
                }
            }

            // Past the value: only whitespace may follow.
            while p < buf_len {
                if !matches!(bytes[p], NL | b' ' | TAB | CAR) {
                    break 'fail DecodeError::TrailingCharacters(dec.rest(p));
                }
                p += 1;
            }
            if dec.stack.len() == 1 && dec.containers.is_empty() {
                let decoded = dec.stack.pop().expect("the decoded value").val;
                result.overwrite(decoded);
                break 'done;
            }
            DecodeError::UnexpectedEnd(dec.rest(0))
        };
        emsg_text(refusal.to_string());
        ret = Err(Failed);
        while let Some(mut left) = dec.stack.pop() {
            tv_clear(&mut left.val);
        }
    }
    ret
}

impl Decoder<'_> {
    /// Push a container that has just opened, both onto the container stack
    /// and — as a value in its own right — onto the value stack.
    fn open(&mut self, container: OpenContainer, special_val: Option<ListRef>, at: usize) {
        let value = match &container {
            OpenContainer::List(list) => TypVal::list(Some(list.clone())),
            OpenContainer::Dict(dict) => TypVal::dict(Some(dict.clone())),
        };
        self.open_value(container, special_val, value, at);
    }

    /// [`Self::open`] for a container whose value is already built: the
    /// container stack keeps a handle, and the value stack keeps the value.
    fn open_value(
        &mut self,
        container: OpenContainer,
        special_val: Option<ListRef>,
        value: TypVal,
        at: usize,
    ) {
        self.containers.push(Container {
            stack_index: self.stack.len(),
            special_val,
            at,
            container,
        });
        let value = self.value(value, false);
        self.stack.push(value);
    }
}
