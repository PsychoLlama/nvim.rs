//! `object.c`: the depth-first walk that turns a token stream into objects.
//!
//! The parser is an explicit stack of [`Node`]s, one per object being built,
//! so a document nests as deep as `capacity` without recursing. Every node is
//! visited twice — [`Decode::enter`] on the way down, [`Decode::exit`] on the
//! way back up — and the walker owns whatever the object actually becomes
//! (an `Object` in `msgpack_rpc/unpacker`, a Lua value in `lmpack`). A
//! node's `data` is the walker's to use: the RPC decoder keeps the object
//! being built there, `lmpack` its registry references.
//!
//! `exiting` is what makes that reentrant: a walk step either pushes a node
//! and returns, or pops as many as it can and returns, so the caller can
//! suspend between any two callbacks — and does, whenever the input runs
//! out mid-document.
//!
//! The stack is indexed, not pointed into: a walker is handed the node it is
//! visiting and the slice of its ancestors below it (the parent is the
//! last), which is all the C reached through `node - 1`.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::mpack_core::{MPACK_EOF, MPACK_ERROR, MPACK_OK, Next, TokBuf};
use super::token::{Kind, Tok};

/// The stack is full: the document nests deeper than `capacity`.
pub const MPACK_NOMEM: c_int = MPACK_ERROR.cast_signed() + 1;

pub const MPACK_MAX_OBJECT_DEPTH: c_int = 32;

const COMPLETE: c_int = MPACK_OK.cast_signed();
const PENDING: c_int = MPACK_EOF.cast_signed();

/// One object in the walk: its token, how many of its children have been
/// seen, and the walker's own state for it.
///
/// `pos` counts items for an array, pairs for a map, and bytes for a
/// `str`/`bin`/`ext` (whose children are chunks). A map's key/value phase
/// rides in `key_visited`, because a map may hold `2^32 - 1` pairs and its
/// children would not fit a 32-bit count.
#[derive(Clone, Debug, Default)]
pub struct Node<D> {
    pub tok: Tok,
    pub pos: usize,
    pub key_visited: bool,
    pub data: D,
}

/// A walk that builds objects out of tokens.
pub trait Decode<D> {
    /// A node is being opened for `tok`, with `parents` below it (the
    /// parent last): answer its state. For a chunk, `chunk` is its bytes;
    /// otherwise empty.
    fn enter(&mut self, tok: Tok, parents: &mut [Node<D>], chunk: &[u8]) -> D;
    /// `node` is complete and has been popped; its parent (the last of
    /// `parents`) has already counted it.
    fn exit(&mut self, node: Node<D>, parents: &mut [Node<D>]);
}

/// A walk that turns objects into tokens.
pub trait Encode<D> {
    /// Fill in `node.tok` for the next object, `parents` below it.
    fn enter(&mut self, node: &mut Node<D>, parents: &mut [Node<D>]);
    /// As [`Decode::exit`].
    fn exit(&mut self, node: Node<D>, parents: &mut [Node<D>]);
    /// The body a chunk node at the top of the stack copies out of — the
    /// whole body, asked for again each time a write of it resumes.
    fn chunk(&mut self, node: &Node<D>, parents: &[Node<D>]) -> &[u8];
}

/// A walk in progress, and the tokenizer feeding (or fed by) it.
#[derive(Clone, Debug)]
pub struct Parser<D> {
    pub tokbuf: TokBuf,
    nodes: Vec<Node<D>>,
    capacity: usize,
    exiting: bool,
    /// What the last encoding step answered, which a resumed write reports
    /// again (the C kept it in `status`).
    unparse_status: c_int,
}

impl<D: Default> Parser<D> {
    /// An empty walk `capacity` nodes deep, or 32 for 0.
    pub fn new(capacity: u32) -> Self {
        let capacity = if capacity != 0 {
            capacity as usize
        } else {
            MPACK_MAX_OBJECT_DEPTH.cast_unsigned() as usize
        };
        Parser {
            tokbuf: TokBuf::new(),
            nodes: Vec::with_capacity(capacity),
            capacity,
            exiting: false,
            unparse_status: 0,
        }
    }

    /// How deep the walk may nest.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Twice as deep, keeping the walk in progress (`lmpack_grow_parser`).
    pub fn grow(&mut self) {
        self.capacity *= 2;
        self.nodes.reserve(self.capacity - self.nodes.len());
    }

    /// Forget the walk in progress; the tokenizer too.
    pub fn reset(&mut self) {
        self.nodes.clear();
        self.tokbuf = TokBuf::new();
        self.exiting = false;
        self.unparse_status = 0;
    }

    /// One step of the walk, shared by both directions.
    ///
    /// Descending, a fresh node is opened and `enter` runs on it; ascending,
    /// every node that has consumed all its children is popped and handed to
    /// `exit`. Either way the answer is `MPACK_EOF` for "call me again" and
    /// `MPACK_OK` for "the root is complete".
    #[inline(always)]
    fn walk<W>(
        &mut self,
        walker: &mut W,
        exit: impl Fn(&mut W, Node<D>, &mut [Node<D>]),
        enter: impl FnOnce(&mut W, &mut [Node<D>]) -> Node<D>,
    ) -> c_int {
        if self.exiting {
            self.exiting = false;
            while let Some(node) = self.pop() {
                exit(walker, node, &mut self.nodes);
                if self.nodes.is_empty() {
                    return COMPLETE;
                }
            }
            return PENDING;
        }
        if self.nodes.len() == self.capacity {
            return MPACK_NOMEM;
        }
        let node = enter(walker, &mut self.nodes);
        self.nodes.push(node);
        self.exiting = true;
        PENDING
    }

    /// Close the top node if it is finished, and tell its parent so.
    ///
    /// Answers `None` while the top node still has children outstanding,
    /// which is what suspends the ascent and sends the walk back down.
    fn pop(&mut self) -> Option<Node<D>> {
        let (top, parents) = self.nodes.split_last_mut().expect("a node to pop");
        let tok = top.tok;
        // Containers and blobs count their children in `pos`.
        if tok.kind > Some(Kind::Chunk) && top.pos < tok.len as usize {
            return None;
        }
        // The root has no parent to report to.
        if let Some(parent) = parents.last_mut() {
            if tok.kind == Some(Kind::Chunk) {
                // A blob's children are bytes, not items.
                parent.pos += tok.len as usize;
            } else if parent.tok.kind == Some(Kind::Map) {
                if parent.key_visited {
                    parent.pos += 1;
                }
                parent.key_visited = !parent.key_visited;
            } else {
                parent.pos += 1;
            }
        }
        self.nodes.pop()
    }

    /// Feed one token (and, for a chunk, its bytes) to a decoding walk.
    #[inline]
    pub fn parse_tok<W: Decode<D>>(&mut self, tok: Tok, chunk: &[u8], walker: &mut W) -> c_int {
        self.walk(walker, W::exit, |walker, parents| Node {
            tok,
            pos: 0,
            key_visited: false,
            data: walker.enter(tok, parents, chunk),
        })
    }

    /// Decode as much of `input` as makes progress, advancing it.
    ///
    /// A token that would overflow the stack, or that is not msgpack, rolls
    /// the input back to where that token started, so the caller can grow
    /// the parser and retry from exactly the same place.
    #[inline]
    pub fn parse<W: Decode<D>>(&mut self, input: &mut &[u8], walker: &mut W) -> c_int {
        let mut status = PENDING;
        while !input.is_empty() && status != COMPLETE {
            let save = *input;
            let (tok, chunk) = match self.tokbuf.read(input) {
                Next::Eof => continue,
                Next::Error => {
                    status = MPACK_ERROR.cast_signed();
                    *input = save;
                    break;
                }
                Next::Token(tok) => (tok, &[][..]),
                Next::Chunk(bytes) => (chunk_token(bytes), bytes),
            };
            // Drive the walk until it stops wanting to pop.
            loop {
                status = self.parse_tok(tok, chunk, walker);
                if !self.exiting {
                    break;
                }
            }
            if status == MPACK_NOMEM {
                *input = save;
                break;
            }
        }
        status
    }

    /// Encode into `out` until it fills or the root object is finished.
    /// Answers the status and how many bytes were written.
    pub fn unparse<W: Encode<D>>(&mut self, out: &mut [u8], walker: &mut W) -> (c_int, usize) {
        let mut status = PENDING;
        let mut written = 0;
        while written < out.len() && status != COMPLETE {
            let mut tok = Tok::default();
            // A half-written token is resumed from the tokbuf, not re-asked
            // for: the walk has already moved past it.
            if !self.tokbuf.is_parked() {
                self.unparse_status = self.walk(walker, W::exit, |walker, parents| {
                    let mut node = Node::default();
                    walker.enter(&mut node, parents);
                    tok = node.tok;
                    node
                });
            }
            status = self.unparse_status;
            if status == MPACK_NOMEM {
                break;
            }
            if self.exiting {
                // What is being written is always the node on top: just
                // entered, or parked half-written.
                let (top, parents) = self.nodes.split_last().expect("an entered node");
                let chunk = if top.tok.kind == Some(Kind::Chunk) {
                    walker.chunk(top, parents)
                } else {
                    &[]
                };
                let (write_status, count) = self.tokbuf.write(&mut out[written..], &tok, chunk);
                written += count;
                if write_status != MPACK_OK {
                    status = c_int::try_from(write_status).expect("a status fits an int");
                }
            }
        }
        (status, written)
    }
}

/// The chunk token a body slice stands for.
fn chunk_token(bytes: &[u8]) -> Tok {
    Tok::new(
        Kind::Chunk,
        u32::try_from(bytes.len()).expect("a chunk is at most a token's length"),
        0,
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counts tokens into a flat list: `(depth, kind, len, chunk)`.
    #[derive(Default)]
    struct Trace(Vec<(usize, Option<Kind>, u32, Vec<u8>)>, Vec<usize>);

    impl Decode<()> for Trace {
        fn enter(&mut self, tok: Tok, parents: &mut [Node<()>], chunk: &[u8]) {
            self.0
                .push((parents.len(), tok.kind, tok.len, chunk.to_vec()));
        }
        fn exit(&mut self, _node: Node<()>, parents: &mut [Node<()>]) {
            self.1.push(parents.len());
        }
    }

    #[test]
    fn a_document_is_walked_down_and_back_up() {
        // [1, {"a": "bc"}]
        let doc = b"\x92\x01\x81\xa1a\xa2bc";
        let mut parser = Parser::<()>::new(0);
        let mut trace = Trace::default();
        let mut input = &doc[..];
        assert_eq!(parser.parse(&mut input, &mut trace), COMPLETE);
        assert!(input.is_empty());
        let kinds: Vec<_> = trace.0.iter().map(|e| (e.0, e.1)).collect();
        assert_eq!(
            kinds,
            [
                (0, Some(Kind::Array)),
                (1, Some(Kind::Uint)),
                (1, Some(Kind::Map)),
                (2, Some(Kind::Str)),
                (3, Some(Kind::Chunk)),
                (2, Some(Kind::Str)),
                (3, Some(Kind::Chunk)),
            ]
        );
        assert_eq!(trace.0[6].3, b"bc");
        // Exits: the uint, the key's chunk and key, the value's chunk and
        // value, the map, the array.
        assert_eq!(trace.1, [1, 3, 2, 3, 2, 1, 0]);
    }

    #[test]
    fn a_split_document_resumes_and_a_deep_one_rolls_back() {
        let doc = b"\x92\x01\x02";
        let mut parser = Parser::<()>::new(0);
        let mut trace = Trace::default();
        let mut first = &doc[..2];
        assert_eq!(parser.parse(&mut first, &mut trace), PENDING);
        let mut rest = &doc[2..];
        assert_eq!(parser.parse(&mut rest, &mut trace), COMPLETE);

        let deep = [0x91u8; 3];
        let mut parser = Parser::<()>::new(2);
        let mut input = &deep[..];
        assert_eq!(parser.parse(&mut input, &mut Trace::default()), MPACK_NOMEM);
        assert_eq!(input.len(), 1, "the token that did not fit is left unread");
        parser.grow();
        assert_eq!(parser.capacity(), 4);
        assert_eq!(parser.parse(&mut input, &mut Trace::default()), PENDING);
    }

    /// Encodes `[n, "xyz"]` for each `n` given.
    struct Emit(u32);

    impl Encode<u8> for Emit {
        fn enter(&mut self, node: &mut Node<u8>, parents: &mut [Node<u8>]) {
            node.tok = match parents.last() {
                None => Tok::new(Kind::Array, 2, 0, 0),
                Some(p) if p.tok.kind == Some(Kind::Str) => Tok::new(Kind::Chunk, 3, 0, 0),
                Some(p) if p.pos == 0 => Tok::new(Kind::Uint, 1, self.0, 0),
                Some(_) => Tok::new(Kind::Str, 3, 0, 0),
            };
        }
        fn exit(&mut self, _node: Node<u8>, _parents: &mut [Node<u8>]) {}
        fn chunk(&mut self, _node: &Node<u8>, _parents: &[Node<u8>]) -> &[u8] {
            b"xyz"
        }
    }

    #[test]
    fn an_encoding_walk_resumes_a_token_cut_by_a_full_buffer() {
        let mut parser = Parser::<u8>::new(0);
        let mut out = Vec::new();
        let mut walker = Emit(7);
        loop {
            let mut room = [0u8; 2];
            let (status, n) = parser.unparse(&mut room, &mut walker);
            out.extend_from_slice(&room[..n]);
            if status == COMPLETE {
                break;
            }
            assert_eq!(status, PENDING);
        }
        assert_eq!(out, b"\x92\x07\xa3xyz");
    }
}
