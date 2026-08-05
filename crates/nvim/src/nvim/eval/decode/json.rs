//! `json_decode()`: JSON text into a `typval_T`.
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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use super::{FAIL, OK, decode_create_map_special_dict};
use crate::src::nvim::eval::typval::{
    TV_INITIAL_VALUE, tv_clear, tv_dict_alloc, tv_list_alloc, tv_list_len, tv_list_ref,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::{
    VAR_BOOL, VAR_DICT, VAR_LIST, VAR_SPECIAL, VAR_UNKNOWN, VAR_UNLOCKED, kBoolVarFalse,
    kBoolVarTrue, kListLenMayKnow, kSpecialVarNull, list_T, ptrdiff_t, size_t, typval_T,
    typval_vval_union,
};

mod scan;
mod stack;

use self::scan::{parse_json_number, parse_json_string};
use self::stack::{Container, Decoder};

/// The ASCII bytes JSON's grammar names.  `BS`/`FF` are only reachable
/// through an escape; the other four are the whitespace between tokens.
pub(crate) const NUL: u8 = 0x00;
pub(crate) const BS: u8 = 0x08;
pub(crate) const TAB: u8 = 0x09;
pub(crate) const NL: u8 = 0x0a;
pub(crate) const FF: u8 = 0x0c;
pub(crate) const CAR: u8 = 0x0d;

const E474_BLANK_STRING: &CStr = c"E474: Attempt to decode a blank string";
const E474_NO_CONTAINER: &CStr = c"E474: No container to close: %.*s";
const E474_CLOSE_LIST_CURLY: &CStr = c"E474: Closing list with curly bracket: %.*s";
const E474_CLOSE_DICT_SQUARE: &CStr = c"E474: Closing dictionary with square bracket: %.*s";
const E474_TRAILING_COMMA: &CStr = c"E474: Trailing comma: %.*s";
const E474_VALUE_AFTER_COLON: &CStr = c"E474: Expected value after colon: %.*s";
const E474_EXPECTED_VALUE: &CStr = c"E474: Expected value: %.*s";
const E474_COMMA_OUTSIDE: &CStr = c"E474: Comma not inside container: %.*s";
const E474_DUPLICATE_COMMA: &CStr = c"E474: Duplicate comma: %.*s";
const E474_COMMA_AFTER_COLON: &CStr = c"E474: Comma after colon: %.*s";
const E474_COMMA_FOR_COLON: &CStr = c"E474: Using comma in place of colon: %.*s";
const E474_LEADING_COMMA: &CStr = c"E474: Leading comma: %.*s";
const E474_COLON_AFTER_COMMA: &CStr = c"E474: Colon after comma: %.*s";
const E474_COLON_OUTSIDE: &CStr = c"E474: Colon not inside container: %.*s";
const E474_COLON_NOT_IN_DICT: &CStr = c"E474: Using colon not in dictionary: %.*s";
const E474_UNEXPECTED_COLON: &CStr = c"E474: Unexpected colon: %.*s";
const E474_DUPLICATE_COLON: &CStr = c"E474: Duplicate colon: %.*s";
const E474_EXPECTED_NULL: &CStr = c"E474: Expected null: %.*s";
const E474_EXPECTED_TRUE: &CStr = c"E474: Expected true: %.*s";
const E474_EXPECTED_FALSE: &CStr = c"E474: Expected false: %.*s";
const E474_UNIDENTIFIED_BYTE: &CStr = c"E474: Unidentified byte: %.*s";
const E474_TRAILING_CHARACTERS: &CStr = c"E474: Trailing characters: %.*s";
const E474_UNEXPECTED_END: &CStr = c"E474: Unexpected end of input: %.*s";

const NULL_TV: typval_T = typval_T {
    v_type: VAR_SPECIAL,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union {
        v_special: kSpecialVarNull,
    },
};

const fn bool_tv(value: bool) -> typval_T {
    typval_T {
        v_type: VAR_BOOL,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union {
            v_bool: if value { kBoolVarTrue } else { kBoolVarFalse },
        },
    }
}

/// Decode `buf_len` bytes of JSON, assumed UTF-8, into `rettv`.
///
/// Answers `OK`, or `FAIL` with the error already reported.
///
/// # Safety
/// `buf` is a live, non-NULL buffer of `buf_len` bytes and `rettv` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn json_decode_string(
    buf: *const c_char,
    buf_len: size_t,
    rettv: *mut typval_T,
) -> c_int {
    // SAFETY: `buf`/`buf_len` are the caller's obligation, which upstream
    // spells FUNC_ATTR_NONNULL_ALL.  Every value on the decoder's stack is
    // owned by it until it is stored, and the failure path clears whatever is
    // left.
    unsafe {
        let bytes = ::core::slice::from_raw_parts(buf as *const u8, buf_len);

        let mut p = 0;
        while p < buf_len && matches!(bytes[p], b' ' | TAB | NL | CAR) {
            p += 1;
        }
        if p == buf_len {
            emsg(gettext(E474_BLANK_STRING.as_ptr()));
            return FAIL;
        }

        (*rettv).v_type = VAR_UNKNOWN;
        let mut dec = Decoder::new(bytes);
        let mut ret = OK;
        // Whether a container holds nothing yet, which is what makes a comma
        // a leading one.
        let is_empty = |c: &Container| {
            if !c.special_val.is_null() {
                tv_list_len(c.special_val) == 0
            } else if c.container.v_type == VAR_DICT {
                (*c.container.vval.v_dict).dv_hashtab.ht_used == 0
            } else {
                tv_list_len(c.container.vval.v_list) == 0
            }
        };
        'done: {
            'fail: {
                'scan: while p < buf_len {
                    // Only `{` may be reached with the flag still set: it is
                    // the byte the restart rewound to.
                    debug_assert!(bytes[p] == b'{' || !dec.next_map_special);
                    match bytes[p] {
                        b'}' | b']' => {
                            let Some(&last) = dec.containers.last() else {
                                dec.emsg_rest(E474_NO_CONTAINER, p);
                                break 'fail;
                            };
                            if bytes[p] == b'}' && last.container.v_type != VAR_DICT {
                                dec.emsg_rest(E474_CLOSE_LIST_CURLY, p);
                                break 'fail;
                            } else if bytes[p] == b']' && last.container.v_type != VAR_LIST {
                                dec.emsg_rest(E474_CLOSE_DICT_SQUARE, p);
                                break 'fail;
                            } else if dec.didcomma {
                                dec.emsg_rest(E474_TRAILING_COMMA, p);
                                break 'fail;
                            } else if dec.didcolon {
                                dec.emsg_rest(E474_VALUE_AFTER_COLON, p);
                                break 'fail;
                            } else if last.stack_index != dec.stack.len() - 1 {
                                debug_assert!(last.stack_index < dec.stack.len() - 1);
                                dec.emsg_rest(E474_EXPECTED_VALUE, p);
                                break 'fail;
                            }
                            if dec.stack.len() == 1 {
                                // The outermost container just closed: the
                                // document is done.
                                p += 1;
                                dec.containers.pop();
                                break 'scan;
                            }
                            let closed = dec.stack.pop().expect("the container itself");
                            if !dec.finish_value(closed, &mut p) {
                                break 'fail;
                            }
                            // A container is never a dictionary key, so it
                            // cannot have triggered the restart.
                            debug_assert!(!dec.next_map_special);
                        }
                        b',' => {
                            let Some(&last) = dec.containers.last() else {
                                dec.emsg_rest(E474_COMMA_OUTSIDE, p);
                                break 'fail;
                            };
                            if dec.didcomma {
                                dec.emsg_rest(E474_DUPLICATE_COMMA, p);
                                break 'fail;
                            } else if dec.didcolon {
                                dec.emsg_rest(E474_COMMA_AFTER_COLON, p);
                                break 'fail;
                            } else if last.container.v_type == VAR_DICT
                                && last.stack_index != dec.stack.len() - 1
                            {
                                dec.emsg_rest(E474_COMMA_FOR_COLON, p);
                                break 'fail;
                            } else if is_empty(&last) {
                                dec.emsg_rest(E474_LEADING_COMMA, p);
                                break 'fail;
                            }
                            dec.didcomma = true;
                            p += 1;
                            continue;
                        }
                        b':' => {
                            let Some(&last) = dec.containers.last() else {
                                dec.emsg_rest(E474_COLON_OUTSIDE, p);
                                break 'fail;
                            };
                            if last.container.v_type != VAR_DICT {
                                dec.emsg_rest(E474_COLON_NOT_IN_DICT, p);
                                break 'fail;
                            } else if last.stack_index != dec.stack.len().wrapping_sub(2) {
                                dec.emsg_rest(E474_UNEXPECTED_COLON, p);
                                break 'fail;
                            } else if dec.didcomma {
                                dec.emsg_rest(E474_COLON_AFTER_COMMA, p);
                                break 'fail;
                            } else if dec.didcolon {
                                dec.emsg_rest(E474_DUPLICATE_COLON, p);
                                break 'fail;
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
                                dec.emsg_rest(E474_EXPECTED_NULL, p);
                                break 'fail;
                            }
                            p += 3;
                            let value = dec.value(NULL_TV, false);
                            if !dec.finish_value(value, &mut p) {
                                break 'fail;
                            }
                            if dec.next_map_special {
                                continue;
                            }
                        }
                        b't' => {
                            if p + 3 >= buf_len || &bytes[p + 1..p + 4] != b"rue" {
                                dec.emsg_rest(E474_EXPECTED_TRUE, p);
                                break 'fail;
                            }
                            p += 3;
                            let value = dec.value(bool_tv(true), false);
                            if !dec.finish_value(value, &mut p) {
                                break 'fail;
                            }
                            if dec.next_map_special {
                                continue;
                            }
                        }
                        b'f' => {
                            if p + 4 >= buf_len || &bytes[p + 1..p + 5] != b"alse" {
                                dec.emsg_rest(E474_EXPECTED_FALSE, p);
                                break 'fail;
                            }
                            p += 4;
                            let value = dec.value(bool_tv(false), false);
                            if !dec.finish_value(value, &mut p) {
                                break 'fail;
                            }
                            if dec.next_map_special {
                                continue;
                            }
                        }
                        b'"' => {
                            // The error was reported by the scanner.
                            if !parse_json_string(&mut dec, &mut p) {
                                break 'fail;
                            }
                            if dec.next_map_special {
                                continue;
                            }
                        }
                        b'-' | b'0'..=b'9' => {
                            if !parse_json_number(&mut dec, &mut p) {
                                break 'fail;
                            }
                            if dec.next_map_special {
                                continue;
                            }
                        }
                        b'[' => {
                            let list = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
                            tv_list_ref(list);
                            let tv = typval_T {
                                v_type: VAR_LIST,
                                v_lock: VAR_UNLOCKED,
                                vval: typval_vval_union { v_list: list },
                            };
                            dec.open(tv, ::core::ptr::null_mut(), p);
                        }
                        b'{' => {
                            let mut tv = TV_INITIAL_VALUE;
                            let mut special_val: *mut list_T = ::core::ptr::null_mut();
                            if dec.next_map_special {
                                dec.next_map_special = false;
                                special_val = decode_create_map_special_dict(
                                    &raw mut tv,
                                    kListLenMayKnow as ptrdiff_t,
                                );
                            } else {
                                let dict = tv_dict_alloc();
                                (*dict).dv_refcount += 1;
                                tv = typval_T {
                                    v_type: VAR_DICT,
                                    v_lock: VAR_UNLOCKED,
                                    vval: typval_vval_union { v_dict: dict },
                                };
                            }
                            dec.open(tv, special_val, p);
                        }
                        _ => {
                            dec.emsg_rest(E474_UNIDENTIFIED_BYTE, p);
                            break 'fail;
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
                        dec.emsg_rest(E474_TRAILING_CHARACTERS, p);
                        break 'fail;
                    }
                    p += 1;
                }
                if dec.stack.len() == 1 && dec.containers.is_empty() {
                    *rettv = dec.stack.pop().expect("the decoded value").val;
                    break 'done;
                }
                dec.emsg_rest(E474_UNEXPECTED_END, 0);
            }
            ret = FAIL;
            while let Some(mut left) = dec.stack.pop() {
                tv_clear(&raw mut left.val);
            }
        }
        ret
    }
}

impl Decoder<'_> {
    /// Push a container that has just opened, both onto the container stack
    /// and — as a value in its own right — onto the value stack.
    fn open(&mut self, container: typval_T, special_val: *mut list_T, at: usize) {
        self.containers.push(Container {
            stack_index: self.stack.len(),
            special_val,
            at,
            container,
        });
        let value = self.value(container, false);
        self.stack.push(value);
    }
}
