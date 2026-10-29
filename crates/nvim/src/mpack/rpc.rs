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
//! handle its request carried. The capacity is a power of nothing in
//! particular: the hash is `id % capacity`, and [`Session::grow`] doubles it
//! when the table fills.
//!
//! Nvim's own RPC does not come through here — `nvim/msgpack_rpc/` has its
//! own framer over [`super::mpack_core`]. This is the framer behind
//! `vim.mpack.Session`.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::mpack_core::{MPACK_EOF, MPACK_OK, Next, TokBuf};
use super::object::MPACK_NOMEM;
use super::token::{Kind, Tok};

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

const COMPLETE: c_int = MPACK_OK.cast_signed();
const PENDING: c_int = MPACK_EOF.cast_signed();

/// The envelope's `type` field. Adding [`MPACK_RPC_REQUEST`] to it gives the
/// kind this module reports.
const TYPE_REQUEST: u32 = 0;
const TYPE_RESPONSE: u32 = 1;
const TYPE_NOTIFICATION: u32 = 2;

/// A message's id, and the caller's handle for the request it answers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RpcMessage {
    pub id: u32,
    pub data: i64,
}

#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    used: bool,
    msg: RpcMessage,
}

/// The envelope tokens seen (or still to send), and how far along they are.
#[derive(Clone, Copy, Debug, Default)]
struct Header {
    toks: [Tok; 3],
    index: usize,
}

/// `mpack_rpc_session_t`: one connection's envelope state both ways, and
/// its outstanding requests.
#[derive(Clone, Debug)]
pub struct Session {
    reader: TokBuf,
    writer: TokBuf,
    receive: Header,
    send: Header,
    request_id: u32,
    slots: Vec<Slot>,
}

impl Session {
    /// No partial message either way, no request outstanding; `capacity`
    /// slots, or 32 for 0.
    pub fn new(capacity: u32) -> Self {
        let capacity = if capacity != 0 {
            capacity
        } else {
            MPACK_RPC_MAX_REQUESTS.cast_unsigned()
        };
        Session {
            reader: TokBuf::new(),
            writer: TokBuf::new(),
            receive: Header::default(),
            send: Header::default(),
            request_id: 0,
            slots: vec![Slot::default(); capacity as usize],
        }
    }

    /// Twice the slots, keeping every outstanding request.
    ///
    /// They are *reinserted* rather than copied: their hash is taken modulo
    /// the capacity, which this changes.
    pub fn grow(&mut self) {
        let old = core::mem::take(&mut self.slots);
        self.slots = vec![Slot::default(); old.len() * 2];
        for slot in old.iter().filter(|slot| slot.used) {
            put(&mut self.slots, slot.msg);
        }
    }

    /// Feed one token of an incoming envelope.
    ///
    /// Answers `MPACK_EOF` while the envelope is incomplete, one of the three
    /// message kinds when it closes (at which point `msg` is filled and the
    /// caller reads the body), or an `MPACK_RPC_E*` for a malformed one.
    pub fn receive_tok(&mut self, tok: Tok, msg: &mut RpcMessage) -> c_int {
        let hdr = &mut self.receive;
        let kind = match hdr.index {
            0 => return receive_array(hdr, tok),
            1 => match receive_type(hdr, tok) {
                Ok(Some(kind)) => kind,
                Ok(None) => return PENDING,
                Err(code) => return code,
            },
            _ => {
                debug_assert_eq!(hdr.index, 2);
                // The id: a request or a response, never a notification.
                if tok.kind != Some(Kind::Uint) || tok.len > 4 {
                    return MPACK_RPC_EMSGID;
                }
                msg.id = tok.lo;
                msg.data = 0;
                let kind = hdr.toks[1].lo.cast_signed() + MPACK_RPC_REQUEST;
                // A response has to name a request that is still open; its
                // slot is what carries the caller's own handle back.
                if kind == MPACK_RPC_RESPONSE && !pop(&mut self.slots, msg) {
                    return MPACK_RPC_ERESPID;
                }
                kind
            }
        };
        self.receive.index = 0;
        kind
    }

    /// Take the next token of an outgoing request envelope, allocating the
    /// message id on the first one.
    fn request_tok(&mut self, tok: &mut Tok, data: i64) -> c_int {
        if self.send.index != 0 {
            return send_rest(&mut self.send, tok);
        }
        // Try ids until one lands in a free slot: `put` answers `Kept` for an
        // id that is still outstanding, so the loop skips over it.
        loop {
            let msg = RpcMessage {
                id: self.request_id,
                data,
            };
            self.send = header(TYPE_REQUEST, 4, Some(msg.id));
            *tok = self.send.toks[0];
            let outcome = put(&mut self.slots, msg);
            if outcome == Put::Full {
                return MPACK_NOMEM;
            }
            // `% 0xffffffff` (not `+ 1`) is upstream's: id 0xffffffff never
            // appears, which costs one id out of four billion.
            self.request_id = self.request_id.wrapping_add(1) % 0xffff_ffff;
            if outcome == Put::Inserted {
                break;
            }
        }
        self.send.index += 1;
        PENDING
    }

    /// Take the next token of an outgoing response envelope.
    fn reply_tok(&mut self, tok: &mut Tok, id: u32) -> c_int {
        let send = &mut self.send;
        if send.index != 0 {
            return send_rest(send, tok);
        }
        *send = header(TYPE_RESPONSE, 4, Some(id));
        *tok = send.toks[0];
        send.index += 1;
        PENDING
    }

    /// Take the next token of an outgoing notification envelope, which is
    /// two tokens rather than three.
    fn notify_tok(&mut self, tok: &mut Tok) -> c_int {
        let send = &mut self.send;
        if send.index != 0 {
            debug_assert_eq!(send.index, 1);
            *tok = send.toks[1];
            send.index = 0;
            return COMPLETE;
        }
        *send = header(TYPE_NOTIFICATION, 3, None);
        *tok = send.toks[0];
        send.index += 1;
        PENDING
    }

    /// Read whole messages out of `input` until one closes or it empties.
    pub fn receive(&mut self, input: &mut &[u8], msg: &mut RpcMessage) -> c_int {
        loop {
            let tok = match self.reader.read(input) {
                Next::Token(tok) => tok,
                Next::Chunk(bytes) => Tok::new(
                    Kind::Chunk,
                    u32::try_from(bytes.len()).expect("a chunk is at most a token's length"),
                    0,
                    0,
                ),
                Next::Eof => return PENDING,
                Next::Error => return super::mpack_core::MPACK_ERROR.cast_signed(),
            };
            let status = self.receive_tok(tok, msg);
            if status >= MPACK_RPC_REQUEST || input.is_empty() {
                return status;
            }
        }
    }

    /// Write an envelope into `out`, resuming a half-written token from the
    /// writer tokbuf rather than asking `next` for it again. Answers the
    /// status and how many bytes went.
    fn send(
        &mut self,
        out: &mut [u8],
        mut next: impl FnMut(&mut Self, &mut Tok) -> c_int,
    ) -> (c_int, usize) {
        let mut status = PENDING;
        let mut written = 0;
        while status != COMPLETE && written < out.len() {
            let mut tok = Tok::default();
            if !self.writer.is_parked() {
                status = next(self, &mut tok);
            }
            if status == MPACK_NOMEM {
                break;
            }
            let (write_status, count) = self.writer.write(&mut out[written..], &tok, &[]);
            written += count;
            if write_status != MPACK_OK {
                status = write_status.cast_signed();
            }
        }
        (status, written)
    }

    /// A request's envelope, carrying `data` back with its response.
    pub fn request(&mut self, out: &mut [u8], data: i64) -> (c_int, usize) {
        self.send(out, |session, tok| session.request_tok(tok, data))
    }

    /// A response's envelope.
    pub fn reply(&mut self, out: &mut [u8], id: u32) -> (c_int, usize) {
        self.send(out, |session, tok| session.reply_tok(tok, id))
    }

    /// A notification's envelope.
    pub fn notify(&mut self, out: &mut [u8]) -> (c_int, usize) {
        self.send(out, Self::notify_tok)
    }
}

/// The envelope's opening array: three items or four, nothing else.
fn receive_array(hdr: &mut Header, tok: Tok) -> c_int {
    if tok.kind != Some(Kind::Array) {
        return MPACK_RPC_EARRAY;
    }
    if !(3..=4).contains(&tok.len) {
        return MPACK_RPC_EARRAYL;
    }
    hdr.toks[0] = tok;
    hdr.index += 1;
    PENDING
}

/// The envelope's message type, which also fixes the array's length.
///
/// Answers the message kind for a notification (which has no id and so ends
/// here), `None` when an id is still to come, or the error code.
fn receive_type(hdr: &mut Header, tok: Tok) -> Result<Option<c_int>, c_int> {
    let value = tok.lo;
    if tok.kind != Some(Kind::Uint) || tok.len > 1 || value > TYPE_NOTIFICATION {
        return Err(MPACK_RPC_ETYPE);
    }
    let expected = if value == TYPE_NOTIFICATION { 3 } else { 4 };
    if hdr.toks[0].len != expected {
        return Err(MPACK_RPC_EARRAYL);
    }
    hdr.toks[1] = tok;
    hdr.index += 1;
    Ok((value == TYPE_NOTIFICATION).then_some(MPACK_RPC_NOTIFICATION))
}

/// The second and third tokens of a request or response envelope. The third
/// closes it, which is what `MPACK_OK` tells the caller.
fn send_rest(send: &mut Header, tok: &mut Tok) -> c_int {
    *tok = send.toks[send.index];
    if send.index == 1 {
        send.index += 1;
        return PENDING;
    }
    debug_assert_eq!(send.index, 2);
    send.index = 0;
    COMPLETE
}

/// The tokens of an outgoing envelope: the array, the message type, and (for
/// everything but a notification) the id. The two integers carry no width,
/// as the C's did not: the encoder picks it from the value.
fn header(msg_type: u32, len: u32, id: Option<u32>) -> Header {
    let mut hdr = Header::default();
    hdr.toks[0] = Tok::new(Kind::Array, len, 0, 0);
    hdr.toks[1] = Tok::new(Kind::Uint, 0, msg_type, 0);
    if let Some(id) = id {
        hdr.toks[2] = Tok::new(Kind::Uint, 0, id, 0);
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
fn put(slots: &mut [Slot], msg: RpcMessage) -> Put {
    let Some(found) = probe(slots, msg.id, |s| !s.used || s.msg.id == msg.id) else {
        return Put::Full;
    };
    if slots[found].used && slots[found].msg.id == msg.id {
        return Put::Kept;
    }
    slots[found] = Slot { msg, used: true };
    Put::Inserted
}

/// Take an outstanding request back out, filling `msg` with what it carried.
fn pop(slots: &mut [Slot], msg: &mut RpcMessage) -> bool {
    let id = msg.id;
    let Some(found) = probe(slots, id, |s| s.used && s.msg.id == id) else {
        return false;
    };
    *msg = slots[found].msg;
    slots[found].used = false;
    true
}

/// Walk the table from `id`'s hash *downwards*, wrapping, until `wanted`
/// accepts a slot or every slot has been tried.
fn probe(slots: &[Slot], id: u32, wanted: impl Fn(&Slot) -> bool) -> Option<usize> {
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

    fn slots(n: usize) -> Vec<Slot> {
        vec![Slot::default(); n]
    }

    fn msg(id: u32) -> RpcMessage {
        RpcMessage {
            id,
            data: i64::from(id) * 10,
        }
    }

    fn uint(v: u32) -> Tok {
        Tok::new(Kind::Uint, 1, v, 0)
    }

    fn array(len: u32) -> Tok {
        Tok::new(Kind::Array, len, 0, 0)
    }

    #[test]
    fn a_request_is_matched_to_its_response_by_id() {
        let mut table = slots(4);
        assert_eq!(put(&mut table, msg(7)), Put::Inserted);
        assert_eq!(put(&mut table, msg(7)), Put::Kept, "already outstanding");
        let mut answer = RpcMessage { id: 7, data: 0 };
        assert!(pop(&mut table, &mut answer));
        assert_eq!(answer.data, 70);
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
            let mut answer = RpcMessage { id, data: 0 };
            assert!(pop(&mut table, &mut answer), "{id}");
            assert_eq!(answer.data, i64::from(id) * 10);
        }
    }

    #[test]
    fn a_full_table_refuses_the_next_request_until_it_grows() {
        let mut session = Session::new(2);
        let mut out = [0u8; 16];
        assert_eq!(session.request(&mut out, 1).0, COMPLETE);
        assert_eq!(session.request(&mut out, 2).0, COMPLETE);
        assert_eq!(session.request(&mut out, 3).0, MPACK_NOMEM);
        session.grow();
        assert_eq!(session.slots.len(), 4);
        let (status, n) = session.request(&mut out, 3);
        assert_eq!((status, &out[..n]), (COMPLETE, &b"\x94\x00\x02"[..]));
    }

    #[test]
    fn an_envelope_is_rejected_before_its_body_is_read() {
        let mut hdr = Header::default();
        assert_eq!(receive_array(&mut hdr, uint(3)), MPACK_RPC_EARRAY);
        assert_eq!(receive_array(&mut hdr, array(2)), MPACK_RPC_EARRAYL);
        assert_eq!(receive_array(&mut hdr, array(5)), MPACK_RPC_EARRAYL);

        // A four-item array is a request or a response, not a notification.
        assert_eq!(receive_array(&mut hdr, array(4)), PENDING);
        assert_eq!(receive_type(&mut hdr, uint(3)), Err(MPACK_RPC_ETYPE));
        assert_eq!(
            receive_type(&mut hdr, uint(TYPE_NOTIFICATION)),
            Err(MPACK_RPC_EARRAYL)
        );
        assert_eq!(receive_type(&mut hdr, uint(TYPE_REQUEST)), Ok(None));
    }

    #[test]
    fn a_notification_envelope_ends_at_the_type() {
        let mut hdr = Header::default();
        assert_eq!(receive_array(&mut hdr, array(3)), PENDING);
        assert_eq!(
            receive_type(&mut hdr, uint(TYPE_NOTIFICATION)),
            Ok(Some(MPACK_RPC_NOTIFICATION))
        );
    }

    #[test]
    fn a_response_comes_back_with_its_request_s_handle() {
        let mut session = Session::new(0);
        let mut out = [0u8; 16];
        let (_, n) = session.request(&mut out, 42);
        assert_eq!(&out[..n], b"\x94\x00\x00");
        let mut msg = RpcMessage::default();
        let mut input: &[u8] = b"\x94\x01\x00";
        assert_eq!(session.receive(&mut input, &mut msg), MPACK_RPC_RESPONSE);
        assert_eq!(msg, RpcMessage { id: 0, data: 42 });
        let mut input: &[u8] = b"\x94\x01\x00";
        assert_eq!(session.receive(&mut input, &mut msg), MPACK_RPC_ERESPID);
    }
}
