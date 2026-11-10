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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::mpack_core::{Next, Stop, TokBuf};
use super::token::{Kind, Tok};

/// What a whole incoming envelope is (libmpack's `MPACK_RPC_REQUEST` …).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Request,
    Response,
    Notification,
}

/// Why an incoming envelope is not (yet) a message: libmpack carried these
/// in the same `int` as the kinds and the codec's statuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unreceived {
    /// The input ran out first: hand over more (`MPACK_EOF`).
    Pending,
    /// The bytes are not msgpack (`MPACK_ERROR`).
    NotMsgpack,
    /// The envelope is not an array (`MPACK_RPC_EARRAY`).
    NotArray,
    /// The array's length does not fit its type (`MPACK_RPC_EARRAYL`).
    ArrayLength,
    /// The type is not 0, 1 or 2 (`MPACK_RPC_ETYPE`).
    Type,
    /// The id is not a 32-bit unsigned integer (`MPACK_RPC_EMSGID`).
    MessageId,
    /// A response names no outstanding request (`MPACK_RPC_ERESPID`).
    ResponseId,
}

pub const MPACK_RPC_MAX_REQUESTS: c_int = 32;

/// The envelope's `type` field.
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
    /// Answers [`Unreceived::Pending`] while the envelope is incomplete, the
    /// message kind when it closes (at which point `msg` is filled and the
    /// caller reads the body), or why a malformed one is refused.
    pub fn receive_tok(
        &mut self,
        tok: Tok,
        msg: &mut RpcMessage,
    ) -> Result<MessageKind, Unreceived> {
        let hdr = &mut self.receive;
        let kind = match hdr.index {
            0 => return receive_array(hdr, tok).and(Err(Unreceived::Pending)),
            1 => match receive_type(hdr, tok)? {
                Some(kind) => kind,
                None => return Err(Unreceived::Pending),
            },
            _ => {
                debug_assert_eq!(hdr.index, 2);
                // The id: a request or a response, never a notification.
                if tok.kind != Some(Kind::Uint) || tok.len > 4 {
                    return Err(Unreceived::MessageId);
                }
                msg.id = tok.lo;
                msg.data = 0;
                if hdr.toks[1].lo == TYPE_REQUEST {
                    MessageKind::Request
                } else {
                    // A response has to name a request that is still open;
                    // its slot is what carries the caller's own handle back.
                    if !pop(&mut self.slots, msg) {
                        return Err(Unreceived::ResponseId);
                    }
                    MessageKind::Response
                }
            }
        };
        self.receive.index = 0;
        Ok(kind)
    }

    /// Take the next token of an outgoing request envelope, allocating the
    /// message id on the first one.
    fn request_tok(&mut self, tok: &mut Tok, data: i64) -> Result<(), Stop> {
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
                return Err(Stop::Full);
            }
            // `% 0xffffffff` (not `+ 1`) is upstream's: id 0xffffffff never
            // appears, which costs one id out of four billion.
            self.request_id = self.request_id.wrapping_add(1) % 0xffff_ffff;
            if outcome == Put::Inserted {
                break;
            }
        }
        self.send.index += 1;
        Err(Stop::Pending)
    }

    /// Take the next token of an outgoing response envelope.
    fn reply_tok(&mut self, tok: &mut Tok, id: u32) -> Result<(), Stop> {
        let send = &mut self.send;
        if send.index != 0 {
            return send_rest(send, tok);
        }
        *send = header(TYPE_RESPONSE, 4, Some(id));
        *tok = send.toks[0];
        send.index += 1;
        Err(Stop::Pending)
    }

    /// Take the next token of an outgoing notification envelope, which is
    /// two tokens rather than three.
    fn notify_tok(&mut self, tok: &mut Tok) -> Result<(), Stop> {
        let send = &mut self.send;
        if send.index != 0 {
            debug_assert_eq!(send.index, 1);
            *tok = send.toks[1];
            send.index = 0;
            return Ok(());
        }
        *send = header(TYPE_NOTIFICATION, 3, None);
        *tok = send.toks[0];
        send.index += 1;
        Err(Stop::Pending)
    }

    /// Read whole messages out of `input` until one closes or it empties.
    pub fn receive(
        &mut self,
        input: &mut &[u8],
        msg: &mut RpcMessage,
    ) -> Result<MessageKind, Unreceived> {
        loop {
            let tok = match self.reader.read(input) {
                Next::Token(tok) => tok,
                Next::Chunk(bytes) => Tok::new(
                    Kind::Chunk,
                    u32::try_from(bytes.len()).expect("a chunk is at most a token's length"),
                    0,
                    0,
                ),
                Next::Eof => return Err(Unreceived::Pending),
                Next::Error => return Err(Unreceived::NotMsgpack),
            };
            let status = self.receive_tok(tok, msg);
            if status != Err(Unreceived::Pending) || input.is_empty() {
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
        mut next: impl FnMut(&mut Self, &mut Tok) -> Result<(), Stop>,
    ) -> (Result<(), Stop>, usize) {
        let mut status = Err(Stop::Pending);
        let mut written = 0;
        while status.is_err() && written < out.len() {
            let mut tok = Tok::default();
            if !self.writer.is_parked() {
                status = next(self, &mut tok);
            }
            if status == Err(Stop::Full) {
                break;
            }
            let (write_status, count) = self.writer.write(&mut out[written..], &tok, &[]);
            written += count;
            if write_status.is_err() {
                status = write_status;
            }
        }
        (status, written)
    }

    /// A request's envelope, carrying `data` back with its response.
    pub fn request(&mut self, out: &mut [u8], data: i64) -> (Result<(), Stop>, usize) {
        self.send(out, |session, tok| session.request_tok(tok, data))
    }

    /// A response's envelope.
    pub fn reply(&mut self, out: &mut [u8], id: u32) -> (Result<(), Stop>, usize) {
        self.send(out, |session, tok| session.reply_tok(tok, id))
    }

    /// A notification's envelope.
    pub fn notify(&mut self, out: &mut [u8]) -> (Result<(), Stop>, usize) {
        self.send(out, Self::notify_tok)
    }
}

/// The envelope's opening array: three items or four, nothing else.
fn receive_array(hdr: &mut Header, tok: Tok) -> Result<(), Unreceived> {
    if tok.kind != Some(Kind::Array) {
        return Err(Unreceived::NotArray);
    }
    if !(3..=4).contains(&tok.len) {
        return Err(Unreceived::ArrayLength);
    }
    hdr.toks[0] = tok;
    hdr.index += 1;
    Ok(())
}

/// The envelope's message type, which also fixes the array's length.
///
/// Answers the message kind for a notification (which has no id and so ends
/// here), `None` when an id is still to come, or why it is refused.
fn receive_type(hdr: &mut Header, tok: Tok) -> Result<Option<MessageKind>, Unreceived> {
    let value = tok.lo;
    if tok.kind != Some(Kind::Uint) || tok.len > 1 || value > TYPE_NOTIFICATION {
        return Err(Unreceived::Type);
    }
    let expected = if value == TYPE_NOTIFICATION { 3 } else { 4 };
    if hdr.toks[0].len != expected {
        return Err(Unreceived::ArrayLength);
    }
    hdr.toks[1] = tok;
    hdr.index += 1;
    Ok((value == TYPE_NOTIFICATION).then_some(MessageKind::Notification))
}

/// The second and third tokens of a request or response envelope. The third
/// closes it, which is what `Ok` tells the caller.
fn send_rest(send: &mut Header, tok: &mut Tok) -> Result<(), Stop> {
    *tok = send.toks[send.index];
    if send.index == 1 {
        send.index += 1;
        return Err(Stop::Pending);
    }
    debug_assert_eq!(send.index, 2);
    send.index = 0;
    Ok(())
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
        assert_eq!(session.request(&mut out, 1).0, Ok(()));
        assert_eq!(session.request(&mut out, 2).0, Ok(()));
        assert_eq!(session.request(&mut out, 3).0, Err(Stop::Full));
        session.grow();
        assert_eq!(session.slots.len(), 4);
        let (status, n) = session.request(&mut out, 3);
        assert_eq!((status, &out[..n]), (Ok(()), &b"\x94\x00\x02"[..]));
    }

    #[test]
    fn an_envelope_is_rejected_before_its_body_is_read() {
        let mut hdr = Header::default();
        assert_eq!(receive_array(&mut hdr, uint(3)), Err(Unreceived::NotArray));
        assert_eq!(
            receive_array(&mut hdr, array(2)),
            Err(Unreceived::ArrayLength)
        );
        assert_eq!(
            receive_array(&mut hdr, array(5)),
            Err(Unreceived::ArrayLength)
        );

        // A four-item array is a request or a response, not a notification.
        assert_eq!(receive_array(&mut hdr, array(4)), Ok(()));
        assert_eq!(receive_type(&mut hdr, uint(3)), Err(Unreceived::Type));
        assert_eq!(
            receive_type(&mut hdr, uint(TYPE_NOTIFICATION)),
            Err(Unreceived::ArrayLength)
        );
        assert_eq!(receive_type(&mut hdr, uint(TYPE_REQUEST)), Ok(None));
    }

    #[test]
    fn a_notification_envelope_ends_at_the_type() {
        let mut hdr = Header::default();
        assert_eq!(receive_array(&mut hdr, array(3)), Ok(()));
        assert_eq!(
            receive_type(&mut hdr, uint(TYPE_NOTIFICATION)),
            Ok(Some(MessageKind::Notification))
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
        assert_eq!(
            session.receive(&mut input, &mut msg),
            Ok(MessageKind::Response)
        );
        assert_eq!(msg, RpcMessage { id: 0, data: 42 });
        let mut input: &[u8] = b"\x94\x01\x00";
        assert_eq!(
            session.receive(&mut input, &mut msg),
            Err(Unreceived::ResponseId)
        );
    }
}
