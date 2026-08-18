//! `rpc.c`: the msgpack-RPC framing layer.
//!
//! A msgpack-RPC message is an array of three or four items whose first two
//! are always `[type, id]` (`[type]` for a notification). This module reads
//! and writes exactly that envelope; the method name, the arguments and the
//! result are the *caller's* to parse out of the same token stream, which is
//! why every entry point hands the buffer back mid-message.
//!
//! Outstanding requests live in `slots`, an open-addressed table keyed on the
//! message id and probed *downwards*, so a response can be matched to the
//! `mpack_data_t` its request carried. `capacity` is a power of nothing in
//! particular: the hash is `id % capacity`, and `lmpack_grow_session` doubles
//! it when the table fills.
//!
//! Nvim's own RPC does not come through here — `nvim/msgpack_rpc/` has its
//! own framer over [`super::mpack_core`]. This is the framer behind
//! `vim.mpack.Session`.
//!
//! The session is a C flexible array member, so the table is reached by
//! offset from `slots`; every entry point turns it into a slice once and the
//! insert/lookup half below is ordinary safe code over that slice.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::mem::offset_of;

use super::mpack_core::{
    MPACK_EOF, MPACK_OK, MPACK_TOKEN_ARRAY, MPACK_TOKEN_UINT, empty_token, from_tok, mpack_read,
    mpack_tokbuf_init, mpack_write, to_tok, value_data,
};
use super::object::MPACK_NOMEM;
use super::token::{Kind, Tok};
use crate::types::{
    mpack_data_t, mpack_rpc_header_t, mpack_rpc_message_t, mpack_rpc_session_t, mpack_rpc_slot_s,
    mpack_token_t, mpack_uint32_t, size_t,
};

/// Message kinds, continuing the `MPACK_*` status numbering so that one
/// `int` can carry either.
pub const MPACK_RPC_REQUEST: c_int = 4;
pub const MPACK_RPC_RESPONSE: c_int = 5;
pub const MPACK_RPC_NOTIFICATION: c_int = 6;

/// Envelope errors, likewise.
pub const MPACK_RPC_EARRAY: c_int = 7;
pub const MPACK_RPC_EARRAYL: c_int = 8;
pub const MPACK_RPC_ETYPE: c_int = 9;
pub const MPACK_RPC_EMSGID: c_int = 10;
pub const MPACK_RPC_ERESPID: c_int = 11;

pub const MPACK_RPC_MAX_REQUESTS: c_int = 32;

/// The envelope's `type` field. Adding [`MPACK_RPC_REQUEST`] to it gives the
/// kind this module reports.
const TYPE_REQUEST: mpack_uint32_t = 0;
const TYPE_RESPONSE: mpack_uint32_t = 1;
const TYPE_NOTIFICATION: mpack_uint32_t = 2;

/// The slot table as a slice. Address arithmetic only, so nothing is read
/// until the caller uses the answer.
///
/// # Safety
/// `session` must be initialised and its allocation must hold `capacity`
/// slots; the slice must not outlive a reallocation of the session.
unsafe fn table<'a>(session: *mut mpack_rpc_session_t) -> &'a mut [mpack_rpc_slot_s] {
    unsafe {
        let base = session
            .cast::<u8>()
            .wrapping_add(offset_of!(mpack_rpc_session_t, slots))
            .cast::<mpack_rpc_slot_s>();
        core::slice::from_raw_parts_mut(base, (*session).capacity as usize)
    }
}

/// Reset a session: no partial message either way, no request outstanding.
///
/// # Safety
/// `session` must point at storage with room for `capacity` slots.
pub unsafe extern "C-unwind" fn mpack_rpc_session_init(
    session: *mut mpack_rpc_session_t,
    capacity: mpack_uint32_t,
) {
    let capacity = if capacity != 0 {
        capacity
    } else {
        MPACK_RPC_MAX_REQUESTS as mpack_uint32_t
    };
    unsafe {
        (*session).capacity = capacity;
        (*session).request_id = 0;
        mpack_tokbuf_init(&raw mut (*session).reader);
        mpack_tokbuf_init(&raw mut (*session).writer);
        // The C only resets `index` and leaves the headers' tokens whatever
        // the allocation held; writing them makes the whole session a value
        // `mpack_rpc_session_copy` can copy without reading uninitialised
        // bytes.
        (*session).receive = empty_header();
        (*session).send = empty_header();
        table(session).fill(empty_slot());
    }
}

/// Feed one token of an incoming envelope.
///
/// Answers `MPACK_EOF` while the envelope is incomplete, one of the three
/// message kinds when it closes (at which point `msg` is filled and the
/// caller reads the body), or an `MPACK_RPC_E*` for a malformed one.
///
/// # Safety
/// `session` must be initialised and `msg` writable.
pub unsafe extern "C-unwind" fn mpack_rpc_receive_tok(
    session: *mut mpack_rpc_session_t,
    tok: mpack_token_t,
    msg: *mut mpack_rpc_message_t,
) -> c_int {
    unsafe {
        let hdr = &mut (*session).receive;
        let kind = match hdr.index {
            0 => return receive_array(hdr, tok),
            1 => match receive_type(hdr, tok) {
                Ok(Some(kind)) => kind,
                Ok(None) => return MPACK_EOF as c_int,
                Err(code) => return code,
            },
            _ => {
                debug_assert_eq!(hdr.index, 2);
                // The id: a request or a response, never a notification.
                if tok.type_0 != MPACK_TOKEN_UINT || tok.length > 4 {
                    return MPACK_RPC_EMSGID;
                }
                (*msg).id = to_tok(&tok).lo;
                (*msg).data.p = core::ptr::null_mut();
                let kind = to_tok(&hdr.toks[1]).lo as c_int + MPACK_RPC_REQUEST;
                // A response has to name a request that is still open; its
                // slot is what carries the caller's own handle back.
                if kind == MPACK_RPC_RESPONSE && !pop(table(session), &mut *msg) {
                    return MPACK_RPC_ERESPID;
                }
                kind
            }
        };
        (*session).receive.index = 0;
        kind
    }
}

/// The envelope's opening array: three items or four, nothing else.
fn receive_array(hdr: &mut mpack_rpc_header_t, tok: mpack_token_t) -> c_int {
    if tok.type_0 != MPACK_TOKEN_ARRAY {
        return MPACK_RPC_EARRAY;
    }
    if !(3..=4).contains(&tok.length) {
        return MPACK_RPC_EARRAYL;
    }
    hdr.toks[0] = tok;
    hdr.index += 1;
    MPACK_EOF as c_int
}

/// The envelope's message type, which also fixes the array's length.
///
/// Answers the message kind for a notification (which has no id and so ends
/// here), `None` when an id is still to come, or the error code.
fn receive_type(hdr: &mut mpack_rpc_header_t, tok: mpack_token_t) -> Result<Option<c_int>, c_int> {
    let value = to_tok(&tok).lo;
    if tok.type_0 != MPACK_TOKEN_UINT || tok.length > 1 || value > TYPE_NOTIFICATION {
        return Err(MPACK_RPC_ETYPE);
    }
    let expected = if value == TYPE_NOTIFICATION { 3 } else { 4 };
    if hdr.toks[0].length != expected {
        return Err(MPACK_RPC_EARRAYL);
    }
    hdr.toks[1] = tok;
    hdr.index += 1;
    Ok((value == TYPE_NOTIFICATION).then_some(MPACK_RPC_NOTIFICATION))
}

/// Take the next token of an outgoing request envelope, allocating the
/// message id on the first one.
///
/// # Safety
/// `session` must be initialised and `tok` writable.
pub unsafe extern "C-unwind" fn mpack_rpc_request_tok(
    session: *mut mpack_rpc_session_t,
    tok: *mut mpack_token_t,
    data: mpack_data_t,
) -> c_int {
    unsafe {
        if (*session).send.index != 0 {
            return send_rest(&mut (*session).send, &mut *tok);
        }
        // Try ids until one lands in a free slot: `put` answers `Kept` for an
        // id that is still outstanding, so the loop skips over it.
        loop {
            let msg = mpack_rpc_message_t {
                id: (*session).request_id,
                data,
            };
            (*session).send = header(TYPE_REQUEST, 4, Some(msg.id));
            *tok = (*session).send.toks[0];
            let outcome = put(table(session), msg);
            if outcome == Put::Full {
                return MPACK_NOMEM;
            }
            // `% 0xffffffff` (not `+ 1`) is upstream's: id 0xffffffff never
            // appears, which costs one id out of four billion.
            (*session).request_id = ((*session).request_id + 1) % 0xffff_ffff;
            if outcome == Put::Inserted {
                break;
            }
        }
        (*session).send.index += 1;
        MPACK_EOF as c_int
    }
}

/// Take the next token of an outgoing response envelope.
///
/// # Safety
/// See [`mpack_rpc_request_tok`].
pub unsafe extern "C-unwind" fn mpack_rpc_reply_tok(
    session: *mut mpack_rpc_session_t,
    tok: *mut mpack_token_t,
    id: mpack_uint32_t,
) -> c_int {
    unsafe {
        let send = &mut (*session).send;
        if send.index != 0 {
            return send_rest(send, &mut *tok);
        }
        *send = header(TYPE_RESPONSE, 4, Some(id));
        *tok = send.toks[0];
        send.index += 1;
        MPACK_EOF as c_int
    }
}

/// Take the next token of an outgoing notification envelope, which is two
/// tokens rather than three.
///
/// # Safety
/// See [`mpack_rpc_request_tok`].
pub unsafe extern "C-unwind" fn mpack_rpc_notify_tok(
    session: *mut mpack_rpc_session_t,
    tok: *mut mpack_token_t,
) -> c_int {
    unsafe {
        let send = &mut (*session).send;
        if send.index != 0 {
            debug_assert_eq!(send.index, 1);
            *tok = send.toks[1];
            send.index = 0;
            return MPACK_OK as c_int;
        }
        *send = header(TYPE_NOTIFICATION, 3, None);
        *tok = send.toks[0];
        send.index += 1;
        MPACK_EOF as c_int
    }
}

/// The second and third tokens of a request or response envelope. The third
/// closes it, which is what `MPACK_OK` tells the caller.
fn send_rest(send: &mut mpack_rpc_header_t, tok: &mut mpack_token_t) -> c_int {
    *tok = send.toks[send.index as usize];
    if send.index == 1 {
        send.index += 1;
        return MPACK_EOF as c_int;
    }
    debug_assert_eq!(send.index, 2);
    send.index = 0;
    MPACK_OK as c_int
}

/// Read whole messages out of `*buf` until one closes or the buffer empties.
///
/// # Safety
/// `buf`/`buflen` must describe a readable slice; `session` and `msg` must be
/// writable.
pub unsafe extern "C-unwind" fn mpack_rpc_receive(
    session: *mut mpack_rpc_session_t,
    buf: *mut *const c_char,
    buflen: *mut size_t,
    msg: *mut mpack_rpc_message_t,
) -> c_int {
    let mut status;
    unsafe {
        loop {
            let mut tok = empty_token();
            status = mpack_read(&raw mut (*session).reader, buf, buflen, &raw mut tok);
            if status != MPACK_OK as c_int {
                break;
            }
            status = mpack_rpc_receive_tok(session, tok, msg);
            if status >= MPACK_RPC_REQUEST || *buflen == 0 {
                break;
            }
        }
    }
    status
}

/// Write an envelope into `*buf`, resuming a half-written token from the
/// session's writer tokbuf rather than asking `next` for it again.
///
/// # Safety
/// `buf`/`buflen` must describe a writable slice; `next` must be one of the
/// `*_tok` functions above, bound to `session`.
unsafe fn send(
    session: *mut mpack_rpc_session_t,
    buf: *mut *mut c_char,
    buflen: *mut size_t,
    mut next: impl FnMut(*mut mpack_token_t) -> c_int,
) -> c_int {
    let mut status = MPACK_EOF as c_int;
    unsafe {
        while status != MPACK_OK as c_int && *buflen != 0 {
            let mut tok = empty_token();
            if (*session).writer.plen == 0 {
                status = next(&raw mut tok);
            }
            if status == MPACK_NOMEM {
                break;
            }
            let write_status = mpack_write(&raw mut (*session).writer, buf, buflen, &tok);
            if write_status != MPACK_OK as c_int {
                status = write_status;
            }
        }
    }
    status
}

/// # Safety
/// See [`send`].
pub unsafe extern "C-unwind" fn mpack_rpc_request(
    session: *mut mpack_rpc_session_t,
    buf: *mut *mut c_char,
    buflen: *mut size_t,
    data: mpack_data_t,
) -> c_int {
    unsafe {
        send(session, buf, buflen, |tok| {
            mpack_rpc_request_tok(session, tok, data)
        })
    }
}

/// # Safety
/// See [`send`].
pub unsafe extern "C-unwind" fn mpack_rpc_reply(
    session: *mut mpack_rpc_session_t,
    buf: *mut *mut c_char,
    buflen: *mut size_t,
    id: mpack_uint32_t,
) -> c_int {
    unsafe {
        send(session, buf, buflen, |tok| {
            mpack_rpc_reply_tok(session, tok, id)
        })
    }
}

/// # Safety
/// See [`send`].
pub unsafe extern "C-unwind" fn mpack_rpc_notify(
    session: *mut mpack_rpc_session_t,
    buf: *mut *mut c_char,
    buflen: *mut size_t,
) -> c_int {
    unsafe {
        send(session, buf, buflen, |tok| {
            mpack_rpc_notify_tok(session, tok)
        })
    }
}

/// Move a session onto a bigger slot table, keeping `dst`'s capacity.
///
/// The outstanding requests are *reinserted* rather than copied: their hash
/// is taken modulo the capacity, which the move changes.
///
/// # Safety
/// Both sessions must be initialised and `dst` at least as large as `src`.
pub unsafe extern "C-unwind" fn mpack_rpc_session_copy(
    dst: *mut mpack_rpc_session_t,
    src: *mut mpack_rpc_session_t,
) {
    unsafe {
        debug_assert!((*src).capacity <= (*dst).capacity);
        (*dst).reader = (*src).reader;
        (*dst).writer = (*src).writer;
        (*dst).receive = (*src).receive;
        (*dst).send = (*src).send;
        (*dst).request_id = (*src).request_id;
        let (to, from) = (table(dst), table(src));
        to.fill(empty_slot());
        for slot in from.iter().filter(|slot| slot.used != 0) {
            put(to, slot.msg);
        }
    }
}

/// An unused slot.
fn empty_slot() -> mpack_rpc_slot_s {
    mpack_rpc_slot_s {
        msg: mpack_rpc_message_t {
            id: 0,
            data: mpack_data_t {
                p: core::ptr::null_mut(),
            },
        },
        used: 0,
    }
}

/// A zeroed envelope header.
fn empty_header() -> mpack_rpc_header_t {
    mpack_rpc_header_t {
        toks: [empty_token(); 3],
        index: 0,
    }
}

/// The tokens of an outgoing envelope: the array, the message type, and (for
/// everything but a notification) the id.
fn header(
    msg_type: mpack_uint32_t,
    len: mpack_uint32_t,
    id: Option<mpack_uint32_t>,
) -> mpack_rpc_header_t {
    let mut hdr = empty_header();
    hdr.toks[0] = from_tok(&Tok::new(Kind::Array, len, 0, 0));
    hdr.toks[1].type_0 = MPACK_TOKEN_UINT;
    hdr.toks[1].data = value_data(msg_type, 0);
    if let Some(id) = id {
        hdr.toks[2].type_0 = MPACK_TOKEN_UINT;
        hdr.toks[2].data = value_data(id, 0);
    }
    hdr
}

/// What [`put`] did with a message.
#[derive(PartialEq, Eq, Debug)]
enum Put {
    Inserted,
    /// The id is already outstanding; the existing entry was left alone.
    Kept,
    /// Every slot is taken.
    Full,
}

/// Record an outstanding request.
fn put(slots: &mut [mpack_rpc_slot_s], msg: mpack_rpc_message_t) -> Put {
    let Some(found) = probe(slots, msg.id, |s| s.used == 0 || s.msg.id == msg.id) else {
        return Put::Full;
    };
    if slots[found].used != 0 && slots[found].msg.id == msg.id {
        return Put::Kept;
    }
    slots[found] = mpack_rpc_slot_s { msg, used: 1 };
    Put::Inserted
}

/// Take an outstanding request back out, filling `msg` with what it carried.
fn pop(slots: &mut [mpack_rpc_slot_s], msg: &mut mpack_rpc_message_t) -> bool {
    let id = msg.id;
    let Some(found) = probe(slots, id, |s| s.used != 0 && s.msg.id == id) else {
        return false;
    };
    *msg = slots[found].msg;
    slots[found].used = 0;
    true
}

/// Walk the table from `id`'s hash *downwards*, wrapping, until `wanted`
/// accepts a slot or every slot has been tried.
fn probe(
    slots: &[mpack_rpc_slot_s],
    id: mpack_uint32_t,
    wanted: impl Fn(&mpack_rpc_slot_s) -> bool,
) -> Option<usize> {
    let capacity = slots.len();
    let mut hash = id as usize % capacity;
    for _ in 0..capacity {
        if wanted(&slots[hash]) {
            return Some(hash);
        }
        hash = if hash > 0 { hash - 1 } else { capacity - 1 };
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(n: usize) -> Vec<mpack_rpc_slot_s> {
        vec![empty_slot(); n]
    }

    fn msg(id: mpack_uint32_t) -> mpack_rpc_message_t {
        mpack_rpc_message_t {
            id,
            data: mpack_data_t { u: id as u64 * 10 },
        }
    }

    fn uint(v: mpack_uint32_t) -> mpack_token_t {
        from_tok(&Tok::new(Kind::Uint, 1, v, 0))
    }

    fn array(len: mpack_uint32_t) -> mpack_token_t {
        from_tok(&Tok::new(Kind::Array, len, 0, 0))
    }

    #[test]
    fn a_request_is_matched_to_its_response_by_id() {
        let mut table = slots(4);
        assert_eq!(put(&mut table, msg(7)), Put::Inserted);
        assert_eq!(put(&mut table, msg(7)), Put::Kept, "already outstanding");
        let mut answer = mpack_rpc_message_t {
            id: 7,
            data: mpack_data_t { u: 0 },
        };
        assert!(pop(&mut table, &mut answer));
        assert_eq!(unsafe { answer.data.u }, 70);
        assert!(
            !pop(&mut table, &mut answer),
            "a slot is freed by popping it"
        );
    }

    #[test]
    fn colliding_ids_probe_downwards_and_wrap() {
        let mut table = slots(4);
        // 3, 7 and 11 all hash to 3; the probe walks 3, 2, 1.
        for id in [3, 7, 11] {
            assert_eq!(put(&mut table, msg(id)), Put::Inserted, "{id}");
        }
        assert_eq!(table[3].msg.id, 3);
        assert_eq!(table[2].msg.id, 7);
        assert_eq!(table[1].msg.id, 11);
        for id in [3, 7, 11] {
            let mut answer = mpack_rpc_message_t {
                id,
                data: mpack_data_t { u: 0 },
            };
            assert!(pop(&mut table, &mut answer), "{id}");
            assert_eq!(unsafe { answer.data.u }, id as u64 * 10);
        }
    }

    #[test]
    fn a_full_table_refuses_the_next_request() {
        let mut table = slots(2);
        assert_eq!(put(&mut table, msg(0)), Put::Inserted);
        assert_eq!(put(&mut table, msg(1)), Put::Inserted);
        assert_eq!(put(&mut table, msg(2)), Put::Full);
    }

    #[test]
    fn an_envelope_is_rejected_before_its_body_is_read() {
        let mut hdr = empty_header();
        assert_eq!(receive_array(&mut hdr, uint(3)), MPACK_RPC_EARRAY);
        assert_eq!(receive_array(&mut hdr, array(2)), MPACK_RPC_EARRAYL);
        assert_eq!(receive_array(&mut hdr, array(5)), MPACK_RPC_EARRAYL);

        // A four-item array is a request or a response, not a notification.
        assert_eq!(receive_array(&mut hdr, array(4)), MPACK_EOF as c_int);
        assert_eq!(receive_type(&mut hdr, uint(3)), Err(MPACK_RPC_ETYPE));
        assert_eq!(
            receive_type(&mut hdr, uint(TYPE_NOTIFICATION)),
            Err(MPACK_RPC_EARRAYL)
        );
        assert_eq!(receive_type(&mut hdr, uint(TYPE_REQUEST)), Ok(None));
    }

    #[test]
    fn a_notification_envelope_ends_at_the_type() {
        let mut hdr = empty_header();
        assert_eq!(receive_array(&mut hdr, array(3)), MPACK_EOF as c_int);
        assert_eq!(
            receive_type(&mut hdr, uint(TYPE_NOTIFICATION)),
            Ok(Some(MPACK_RPC_NOTIFICATION))
        );
    }
}
