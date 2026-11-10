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
use crate::ascii::{BS, CAR, FF, NL, TAB};
use crate::eval::typval::{ListRef, tv_clear, tv_dict_alloc, tv_list_alloc};
use crate::memory::ThinCString;
use crate::message_fmt::emsg_text;
use crate::os::libc::ptrdiff_t;
use crate::types::{
    BOOL_VAR_FALSE, BOOL_VAR_TRUE, Failed, LIST_LEN_MAY_KNOW, SPECIAL_VAR_NULL, TypVal,
};

mod error;
mod scan;
mod stack;

use self::error::DecodeError;

use self::scan::{parse_json_number, parse_json_string};
use self::stack::{Container, Decoder, OpenContainer};

const NULL_TV: TypVal = TypVal::Special(SPECIAL_VAR_NULL);

const fn bool_tv(value: bool) -> TypVal {
    TypVal::Bool(if value { BOOL_VAR_TRUE } else { BOOL_VAR_FALSE })
}

/// Decode `bytes` of JSON, assumed UTF-8, into `result`.
///
/// Answers `Err` once the refusal has been said: the scanner, the string
/// and number scanners and the value stack answer a `DecodeError`, and
/// this is where it is reported. Every value on the decoder's stack is owned
/// by it until it is stored, and the failure path clears whatever is left.
pub fn json_decode_string(bytes: &[u8], result: &mut TypVal) -> Result<(), Failed> {
    let start = bytes
        .iter()
        .position(|&b| !matches!(b, b' ' | TAB | NL | CAR));
    let Some(start) = start else {
        emsg_text(DecodeError::BlankString.to_string());
        return Err(Failed);
    };

    result.overwrite(TypVal::Unknown);
    let mut dec = Decoder::new(bytes);
    match dec.document(start) {
        Ok(decoded) => {
            result.overwrite(decoded);
            Ok(())
        }
        Err(refusal) => {
            emsg_text(refusal.to_string());
            while let Some(mut left) = dec.stack.pop() {
                tv_clear(&mut left.val);
            }
            Err(Failed)
        }
    }
}

/// What the scanner does after the byte under the cursor was handled.
#[derive(Debug)]
enum Step {
    /// A token was taken: clear the comma and colon flags, step past the
    /// byte, and stop if that closed the outermost container.
    Took,
    /// Look at the cursor again as it is: punctuation that set its own flag
    /// and stepped past itself, whitespace, or a restart rewound to a `{`.
    Rescan,
    /// The outermost container closed and the cursor is past it.
    Closed,
}

impl Container {
    /// Whether the container holds nothing yet, which is what makes a comma
    /// a leading one.
    fn is_empty(&self) -> bool {
        match (&self.special_val, &self.container) {
            (Some(pairs), _) => pairs.items().is_empty(),
            (None, OpenContainer::Dict(d)) => d.dv_hashtab.ht_used == 0,
            (None, OpenContainer::List(l)) => l.items().is_empty(),
        }
    }
}

impl Decoder<'_> {
    /// Scan one value from `p` and check that only whitespace follows it:
    /// answers the decoded value.
    fn document(&mut self, mut p: usize) -> Result<TypVal, DecodeError> {
        self.scan(&mut p)?;
        // Past the value: only whitespace may follow.
        let buf_len = self.buf.len();
        while p < buf_len {
            if !matches!(self.buf[p], NL | b' ' | TAB | CAR) {
                return Err(DecodeError::TrailingCharacters(self.rest(p)));
            }
            p += 1;
        }
        if self.stack.len() == 1 && self.containers.is_empty() {
            return Ok(self.stack.pop().expect("the decoded value").val);
        }
        Err(DecodeError::UnexpectedEnd(self.rest(0)))
    }

    /// The scan proper: one pass from `p` over a value, dispatching on the
    /// byte under the cursor until the outermost value is complete or the
    /// input ends.
    fn scan(&mut self, p: &mut usize) -> Result<(), DecodeError> {
        let buf_len = self.buf.len();
        while *p < buf_len {
            // Only `{` may be reached with the flag still set: it is the
            // byte the restart rewound to.
            debug_assert!(self.buf[*p] == b'{' || !self.next_map_special);
            let step = match self.buf[*p] {
                b'}' | b']' => self.close(p)?,
                b',' => self.comma(p)?,
                b':' => self.colon(p)?,
                b' ' | TAB | NL | CAR => {
                    *p += 1;
                    Step::Rescan
                }
                b'n' => self.keyword(p, b"null", NULL_TV, DecodeError::ExpectedNull)?,
                b't' => self.keyword(p, b"true", bool_tv(true), DecodeError::ExpectedTrue)?,
                b'f' => self.keyword(p, b"false", bool_tv(false), DecodeError::ExpectedFalse)?,
                b'"' => {
                    parse_json_string(self, p)?;
                    self.took_value()
                }
                b'-' | b'0'..=b'9' => {
                    parse_json_number(self, p)?;
                    self.took_value()
                }
                b'[' => {
                    let list = tv_list_alloc(LIST_LEN_MAY_KNOW as ptrdiff_t);
                    self.open(OpenContainer::List(list), None, *p);
                    Step::Took
                }
                b'{' => {
                    self.open_dict(*p);
                    Step::Took
                }
                _ => return Err(DecodeError::UnidentifiedByte(self.rest(*p))),
            };
            match step {
                Step::Rescan => continue,
                Step::Closed => return Ok(()),
                Step::Took => {}
            }
            self.didcomma = false;
            self.didcolon = false;
            *p += 1;
            if self.containers.is_empty() {
                return Ok(());
            }
        }
        Ok(())
    }

    /// A value was stored: go on past it, or look again at the `{` a
    /// restart rewound to.
    fn took_value(&self) -> Step {
        if self.next_map_special {
            Step::Rescan
        } else {
            Step::Took
        }
    }

    /// `}` or `]`: close the innermost container and store it as a value in
    /// the one around it.
    fn close(&mut self, p: &mut usize) -> Result<Step, DecodeError> {
        let close = self.buf[*p];
        let Some(last) = self.containers.last() else {
            return Err(DecodeError::NoContainer(self.rest(*p)));
        };
        if close == b'}' && !last.container.is_dict() {
            return Err(DecodeError::CloseListCurly(self.rest(*p)));
        } else if close == b']' && last.container.is_dict() {
            return Err(DecodeError::CloseDictSquare(self.rest(*p)));
        } else if self.didcomma {
            return Err(DecodeError::TrailingComma(self.rest(*p)));
        } else if self.didcolon {
            return Err(DecodeError::ValueAfterColon(self.rest(*p)));
        } else if last.stack_index != self.stack.len() - 1 {
            debug_assert!(last.stack_index < self.stack.len() - 1);
            return Err(DecodeError::ExpectedValue(self.rest(*p)));
        }
        if self.stack.len() == 1 {
            // The outermost container just closed: the document is done.
            *p += 1;
            self.containers.pop();
            return Ok(Step::Closed);
        }
        let closed = self.stack.pop().expect("the container itself");
        self.finish_value(closed, p)?;
        // A container is never a dictionary key, so it cannot have
        // triggered the restart.
        debug_assert!(!self.next_map_special);
        Ok(Step::Took)
    }

    /// `,`: legal only after a value inside a container.
    fn comma(&mut self, p: &mut usize) -> Result<Step, DecodeError> {
        let Some(last) = self.containers.last() else {
            return Err(DecodeError::CommaOutside(self.rest(*p)));
        };
        if self.didcomma {
            return Err(DecodeError::DuplicateComma(self.rest(*p)));
        } else if self.didcolon {
            return Err(DecodeError::CommaAfterColon(self.rest(*p)));
        } else if last.container.is_dict() && last.stack_index != self.stack.len() - 1 {
            return Err(DecodeError::CommaForColon(self.rest(*p)));
        } else if last.is_empty() {
            return Err(DecodeError::LeadingComma(self.rest(*p)));
        }
        self.didcomma = true;
        *p += 1;
        Ok(Step::Rescan)
    }

    /// `:`: legal only after a key inside a dictionary.
    fn colon(&mut self, p: &mut usize) -> Result<Step, DecodeError> {
        let Some(last) = self.containers.last() else {
            return Err(DecodeError::ColonOutside(self.rest(*p)));
        };
        if !last.container.is_dict() {
            return Err(DecodeError::ColonNotInDict(self.rest(*p)));
        } else if last.stack_index != self.stack.len().wrapping_sub(2) {
            return Err(DecodeError::UnexpectedColon(self.rest(*p)));
        } else if self.didcomma {
            return Err(DecodeError::ColonAfterComma(self.rest(*p)));
        } else if self.didcolon {
            return Err(DecodeError::DuplicateColon(self.rest(*p)));
        }
        self.didcolon = true;
        *p += 1;
        Ok(Step::Rescan)
    }

    /// `null`, `true` or `false`, spelled out in full: stored as `value`,
    /// refused as `refuse` when the spelling stops short.
    fn keyword(
        &mut self,
        p: &mut usize,
        word: &[u8],
        value: TypVal,
        refuse: fn(ThinCString) -> DecodeError,
    ) -> Result<Step, DecodeError> {
        let last = word.len() - 1;
        if *p + last >= self.buf.len() || self.buf[*p + 1..=*p + last] != word[1..] {
            return Err(refuse(self.rest(*p)));
        }
        *p += last;
        let value = self.value(value, false);
        self.finish_value(value, p)?;
        Ok(self.took_value())
    }

    /// `{`: a dictionary, or the special map a restart rewound to.
    fn open_dict(&mut self, at: usize) {
        if self.next_map_special {
            self.next_map_special = false;
            let len = LIST_LEN_MAY_KNOW as ptrdiff_t;
            let (special, pairs) = decode_create_map_special_dict(len);
            let dict = special.dict_shared().expect("a special map").clone();
            self.open_value(OpenContainer::Dict(dict), Some(pairs), special, at);
        } else {
            self.open(OpenContainer::Dict(tv_dict_alloc()), None, at);
        }
    }

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
