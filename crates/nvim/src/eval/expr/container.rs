//! List and dict literals, including the `#{}` form.

#![forbid(unsafe_code)]

use crate::eval::Parsed;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::memory::ThinCString;
use crate::message_fmt::msg_bytes;
use crate::semsg;

use crate::eval::typval::{NumBuf, dict_find, tv_clear, tv_dict_alloc, tv_list_alloc};
use crate::eval::{Cursor, eval1};
use crate::types::{Failed, NUL, TypVal, kListLenShouldKnow, ptrdiff_t};

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// `[a, b, c]`, with the cursor on the `[`.
pub(crate) fn eval_list(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    // The one reference to the list being built. A path that gives up
    // drops it, which is what upstream's `list_free` on a list still at
    // refcount zero was.
    let mut held = evaluate.then(|| tv_list_alloc(kListLenShouldKnow as ptrdiff_t));
    cursor.bump(1);
    cursor.skip_white();

    let ok = 'items: {
        while cursor.byte() != b']' && cursor.byte() != NUL as u8 {
            let mut tv = UNSET_TV;
            if eval1(cursor, &mut tv, evaluate).is_err() {
                break 'items false;
            }
            if let Some(list) = held.as_mut() {
                list.push(tv);
            }
            let had_comma = cursor.byte() == b',';
            if had_comma {
                cursor.bump(1);
                cursor.skip_white();
            }
            if cursor.byte() == b']' {
                break;
            }
            // A trailing comma is allowed; a missing one is not.
            if had_comma {
                continue;
            }
            let at = msg_bytes(cursor.rest());
            semsg!("E696: Missing comma in List: {at}");
            break 'items false;
        }
        if cursor.byte() != b']' {
            let at = msg_bytes(cursor.rest());
            semsg!("E697: Missing end of List ']': {at}");
            break 'items false;
        }
        cursor.bump(1);
        cursor.skip_white();
        true
    };

    if ok {
        result.write_list(held);
        return Ok(());
    }
    // The half-built list goes with the handle, which is its only reference.
    drop(held);
    Err(Failed)
}

/// The bare word a `#{}` literal uses as a key: letters, digits, `_` and `-`.
pub(crate) fn get_literal_key(cursor: &mut Cursor<'_>, tv: &mut TypVal) -> Result<(), Failed> {
    let is_key_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    let rest = cursor.rest();
    let len = rest.iter().take_while(|&&b| is_key_char(b)).count();
    if len == 0 {
        return Err(Failed);
    }
    tv.write_string(Some(ThinCString::from_bytes(&rest[..len])));
    cursor.bump(len);
    cursor.skip_white();
    Ok(())
}

/// `{k: v}` and, with `literal` set, `#{k: v}`.
///
/// Answers [`Parsed::NotThis`] when the `{` opened a curly-braces name rather than a
/// dictionary, which the caller then re-reads as a name.
pub(crate) fn eval_dict(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    literal: bool,
) -> Result<Parsed, Failed> {
    let mut tv = UNSET_TV;
    let mut buf = NumBuf::new();

    // Is this `{expr}` rather than a Dict? It has to be decided without
    // evaluating, or a function in it would be called twice. `{}` is an
    // empty Dict and `#{abc}` is never a curly-braces name.
    let mut curly = Cursor::new(cursor.text());
    curly.bump(cursor.offset() + 1);
    curly.skip_white();
    if curly.byte() != b'}' && !literal && eval1(&mut curly, &mut tv, false).is_ok() && {
        curly.skip_white();
        curly.byte() == b'}'
    } {
        return Ok(Parsed::NotThis);
    }

    // The one reference to the dictionary being built. A path that gives
    // up drops it, which is what upstream's `tv_dict_free` on a dictionary
    // still at refcount zero was.
    let mut held = evaluate.then(tv_dict_alloc);
    let mut tvkey = UNSET_TV;
    tv = UNSET_TV;
    cursor.bump(1);
    cursor.skip_white();

    let ok = 'items: {
        while cursor.byte() != b'}' && cursor.byte() != NUL as u8 {
            let read_key = if literal {
                get_literal_key(cursor, &mut tvkey)
            } else {
                eval1(cursor, &mut tvkey, evaluate)
            };
            if read_key.is_err() {
                break 'items false;
            }
            if cursor.byte() != b':' {
                let at = msg_bytes(cursor.rest());
                semsg!("E720: Missing colon in Dictionary: {at}");
                tv_clear(&mut tvkey);
                break 'items false;
            }

            // The key borrows `buf`, so it must not outlive this pass.
            let mut key: &[u8] = b"";
            if evaluate {
                let Some(text) = buf.string_chk(&tvkey) else {
                    tv_clear(&mut tvkey);
                    break 'items false;
                };
                key = text.to_bytes();
            }
            cursor.bump(1);
            cursor.skip_white();
            if eval1(cursor, &mut tv, evaluate).is_err() {
                tv_clear(&mut tvkey);
                break 'items false;
            }
            if let Some(dict) = held.as_mut() {
                if dict_find(Some(dict), key).is_some() {
                    let key = msg_bytes(key);
                    semsg!("E721: Duplicate key in Dictionary: \"{key}\"");
                    tv_clear(&mut tvkey);
                    tv_clear(&mut tv);
                    break 'items false;
                }
                let _ = dict.add_value(key, tv.take());
            }
            tv_clear(&mut tvkey);

            let had_comma = cursor.byte() == b',';
            if had_comma {
                cursor.bump(1);
                cursor.skip_white();
            }
            if cursor.byte() == b'}' {
                break;
            }
            if had_comma {
                continue;
            }
            let at = msg_bytes(cursor.rest());
            semsg!("E722: Missing comma in Dictionary: {at}");
            break 'items false;
        }
        if cursor.byte() != b'}' {
            let at = msg_bytes(cursor.rest());
            semsg!("E723: Missing end of Dictionary '}}': {at}");
            break 'items false;
        }
        cursor.bump(1);
        cursor.skip_white();
        true
    };

    if ok {
        result.write_dict(held);
        return Ok(Parsed::Done);
    }
    // The half-built dictionary goes with the handle, which is its only
    // reference.
    drop(held);
    Err(Failed)
}

/// `#{...}`, with the cursor on the `#`.
pub(crate) fn eval_lit_dict(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<Parsed, Failed> {
    if cursor.at(1) != b'{' {
        return Ok(Parsed::NotThis);
    }
    cursor.bump(1);
    eval_dict(cursor, result, evaluate, true)
}
