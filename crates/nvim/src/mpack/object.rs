//! `object.c`: the depth-first walk that turns a token stream into objects.
//!
//! The parser is an explicit stack of [`mpack_node_t`], one frame per
//! container being built, so a document nests as deep as `capacity` without
//! recursing and without ever unwinding through C. Every frame is visited
//! twice — `enter_cb` on the way down, `exit_cb` on the way back up — and
//! the callback owns whatever the object actually becomes (a `typval_T` in
//! `eval/decode/msgpack`, an `Object` in `msgpack_rpc/unpacker`, a Lua value
//! in `lmpack`).
//!
//! `exiting` is what makes that reentrant: a walk step either pushes a frame
//! and returns, or pops as many as it can and returns, so the caller can
//! suspend between any two callbacks. `status` carries an exception raised
//! *by* a callback (`lmpack` longjmps out of Lua errors), and every entry
//! point checks it first so a poisoned parser stays poisoned.
//!
//! The stack is a C flexible array member — `mpack_parser_t` declares 33
//! slots but `lmpack_grow_parser` reallocates for more — so the frames are
//! reached by offset from `items`, not by indexing the declared array. Every
//! function here keeps the parser as a raw pointer for that reason, and
//! because the callbacks are handed the same pointer and write through it.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::mem::offset_of;

use super::mpack_core::{MPACK_EOF, MPACK_ERROR, MPACK_OK, empty_token, mpack_read, mpack_write};
use crate::types::{
    mpack_node_t, mpack_parser_t, mpack_token_t, mpack_token_type_t, mpack_uint32_t, mpack_walk_cb,
    size_t,
};

/// A callback raised an error; the parser is unusable until it is reset.
pub const MPACK_EXCEPTION: c_int = -1;
/// The stack is full: the document nests deeper than `capacity`.
pub const MPACK_NOMEM: c_int = MPACK_ERROR as c_int + 1;

pub const MPACK_MAX_OBJECT_DEPTH: c_int = 32;

const MPACK_TOKEN_CHUNK: mpack_token_type_t = super::mpack_core::MPACK_TOKEN_CHUNK;
const MPACK_TOKEN_MAP: mpack_token_type_t = super::mpack_core::MPACK_TOKEN_MAP;

/// The sentinel `items[0].pos` carries so that frame 1 has no parent.
const NO_PARENT: size_t = size_t::MAX;

/// Frame `i` of a parser's stack, counting the unused sentinel at 0.
///
/// Address arithmetic only — the offset is computed rather than projected
/// through the pointer, so nothing is read until a caller dereferences the
/// answer. `i` is meaningful up to the parser's own `capacity`, which may
/// exceed the 33 slots the struct declares.
pub fn frame(parser: *mut mpack_parser_t, i: mpack_uint32_t) -> *mut mpack_node_t {
    parser
        .cast::<u8>()
        .wrapping_add(offset_of!(mpack_parser_t, items))
        .cast::<mpack_node_t>()
        .wrapping_add(i as usize)
}

/// The frame below `node`, or null when `node` is the root.
///
/// The walk callbacks are handed a bare node pointer with no index, so this
/// is how they find the container they are an element of.
///
/// # Safety
/// `node` must be a live frame of a parser stack.
pub unsafe fn parent_of(node: *mut mpack_node_t) -> *mut mpack_node_t {
    unsafe {
        let below = node.sub(1);
        if (*below).pos == NO_PARENT {
            core::ptr::null_mut()
        } else {
            below
        }
    }
}

/// Reset a parser to an empty stack `capacity` frames deep.
///
/// # Safety
/// `parser` must point at storage with room for `capacity + 1` frames —
/// `mpack_parser_t` for the default 32, or `lmpack_grow_parser`'s larger
/// allocation.
pub unsafe fn mpack_parser_init(parser: *mut mpack_parser_t, capacity: mpack_uint32_t) {
    let capacity = if capacity != 0 {
        capacity
    } else {
        MPACK_MAX_OBJECT_DEPTH as mpack_uint32_t
    };
    unsafe {
        super::mpack_core::mpack_tokbuf_init(&raw mut (*parser).tokbuf);
        (*parser).data.p = core::ptr::null_mut();
        (*parser).capacity = capacity;
        (*parser).size = 0;
        (*parser).exiting = 0;
        (*parser).status = 0;
        for i in 0..=capacity {
            frame(parser, i).write(core::mem::zeroed());
        }
        // Frame 0 is never pushed; its `pos` is the sentinel that stops
        // `parent_of` at the root.
        (*frame(parser, 0)).pos = NO_PARENT;
    }
}

/// One step of the walk, shared by both directions.
///
/// Descending, a fresh frame is opened and the caller's `enter` runs on it;
/// ascending, every frame that has consumed all its children is popped and
/// handed to `exit_cb`. Either way the answer is `MPACK_EOF` for "call me
/// again" and `MPACK_OK` for "the root is complete".
///
/// # Safety
/// `parser` must be initialised, and the callbacks must be the pair the
/// caller's node data was built for.
unsafe fn walk(
    parser: *mut mpack_parser_t,
    exit_cb: mpack_walk_cb,
    enter: impl FnOnce(*mut mpack_node_t),
) -> c_int {
    unsafe {
        if (*parser).status == MPACK_EXCEPTION {
            return MPACK_EXCEPTION;
        }
        if (*parser).exiting != 0 {
            (*parser).exiting = 0;
            while let Some(node) = pop(parser) {
                exit_cb.expect("non-null exit callback")(parser, node);
                if (*parser).status == MPACK_EXCEPTION {
                    return MPACK_EXCEPTION;
                }
                if (*parser).size == 0 {
                    return MPACK_OK as c_int;
                }
            }
            return MPACK_EOF as c_int;
        }
        if (*parser).size == (*parser).capacity {
            return MPACK_NOMEM;
        }

        // Open a frame for the new object. Its `data` is the callback's to
        // use; `pos` and `key_visited` count the children seen so far.
        (*parser).size += 1;
        let top = frame(parser, (*parser).size);
        (*top).data[0].p = core::ptr::null_mut();
        (*top).data[1].p = core::ptr::null_mut();
        (*top).pos = 0;
        (*top).key_visited = 0;

        enter(top);
        if (*parser).status == MPACK_EXCEPTION {
            return MPACK_EXCEPTION;
        }
        (*parser).exiting = 1;
        MPACK_EOF as c_int
    }
}

/// Feed one token to a decoding walk.
///
/// # Safety
/// See [`walk`].
pub unsafe fn mpack_parse_tok(
    parser: *mut mpack_parser_t,
    tok: mpack_token_t,
    enter_cb: mpack_walk_cb,
    exit_cb: mpack_walk_cb,
) -> c_int {
    unsafe {
        walk(parser, exit_cb, |node| {
            (*node).tok = tok;
            enter_cb.expect("non-null enter callback")(parser, node);
        })
    }
}

/// Take one token out of an encoding walk.
///
/// # Safety
/// See [`walk`]; `tok` must be writable.
pub unsafe fn mpack_unparse_tok(
    parser: *mut mpack_parser_t,
    tok: *mut mpack_token_t,
    enter_cb: mpack_walk_cb,
    exit_cb: mpack_walk_cb,
) -> c_int {
    unsafe {
        walk(parser, exit_cb, |node| {
            enter_cb.expect("non-null enter callback")(parser, node);
            *tok = (*node).tok;
        })
    }
}

/// Decode as much of `*buf` as makes progress, advancing it.
///
/// A token that would overflow the stack rolls the buffer back to where that
/// token started, so the caller can grow the parser and retry from exactly
/// the same place.
///
/// # Safety
/// `buf`/`buflen` must describe a readable slice; see [`walk`] for the rest.
pub unsafe fn mpack_parse(
    parser: *mut mpack_parser_t,
    buf: *mut *const c_char,
    buflen: *mut size_t,
    enter_cb: mpack_walk_cb,
    exit_cb: mpack_walk_cb,
) -> c_int {
    let mut status = MPACK_EOF as c_int;
    unsafe {
        if (*parser).status == MPACK_EXCEPTION {
            return MPACK_EXCEPTION;
        }
        while *buflen != 0 && status != MPACK_OK as c_int {
            let mut tok = empty_token();
            let (buf_save, buflen_save) = (*buf, *buflen);

            status = mpack_read(&raw mut (*parser).tokbuf, buf, buflen, &raw mut tok);
            if status == MPACK_EOF as c_int {
                continue;
            }
            if status != MPACK_ERROR as c_int {
                // Drive the walk until it stops wanting to pop.
                loop {
                    status = mpack_parse_tok(parser, tok, enter_cb, exit_cb);
                    if (*parser).status == MPACK_EXCEPTION {
                        return MPACK_EXCEPTION;
                    }
                    if (*parser).exiting == 0 {
                        break;
                    }
                }
                if status != MPACK_NOMEM {
                    continue;
                }
            }
            *buf = buf_save;
            *buflen = buflen_save;
            break;
        }
    }
    status
}

/// Encode into `*buf` until it fills or the root object is finished.
///
/// # Safety
/// `buf`/`buflen` must describe a writable slice; see [`walk`] for the rest.
pub unsafe fn mpack_unparse(
    parser: *mut mpack_parser_t,
    buf: *mut *mut c_char,
    buflen: *mut size_t,
    enter_cb: mpack_walk_cb,
    exit_cb: mpack_walk_cb,
) -> c_int {
    let mut status = MPACK_EOF as c_int;
    unsafe {
        if (*parser).status == MPACK_EXCEPTION {
            return MPACK_EXCEPTION;
        }
        while *buflen != 0 && status != MPACK_OK as c_int {
            let mut tok = empty_token();
            let tb = &raw mut (*parser).tokbuf;
            // A half-written token is resumed from the tokbuf, not re-asked
            // for: the walk has already moved past it.
            if (*tb).plen == 0 {
                (*parser).status = mpack_unparse_tok(parser, &raw mut tok, enter_cb, exit_cb);
            }
            if (*parser).status == MPACK_EXCEPTION {
                return MPACK_EXCEPTION;
            }
            status = (*parser).status;
            if status == MPACK_NOMEM {
                break;
            }
            if (*parser).exiting != 0 {
                let write_status = mpack_write(tb, buf, buflen, &raw mut tok);
                if write_status != MPACK_OK as c_int {
                    status = write_status;
                }
            }
        }
    }
    status
}

/// Move a walk in progress onto a bigger stack, keeping `d`'s capacity.
///
/// # Safety
/// Both parsers must be initialised and `d` must be at least as deep as `s`.
pub unsafe fn mpack_parser_copy(d: *mut mpack_parser_t, s: *mut mpack_parser_t) {
    unsafe {
        debug_assert!((*s).capacity <= (*d).capacity);
        // Every field but `capacity`, which describes the destination's own
        // allocation. The C memcpy's the struct header for this.
        (*d).data = (*s).data;
        (*d).size = (*s).size;
        (*d).status = (*s).status;
        (*d).exiting = (*s).exiting;
        (*d).tokbuf = (*s).tokbuf;
        for i in 0..=(*s).capacity {
            frame(d, i).write(frame(s, i).read());
        }
    }
}

/// Close the top frame if it is finished, and tell its parent so.
///
/// Answers `None` while the top frame still has children outstanding, which
/// is what suspends the ascent and sends the walk back down.
///
/// # Safety
/// The stack must not be empty.
unsafe fn pop(p: *mut mpack_parser_t) -> Option<*mut mpack_node_t> {
    unsafe {
        debug_assert!((*p).size != 0);
        let size = (*p).size;
        let top = frame(p, size);
        let tok = (*top).tok;
        // Containers and blobs count their children in `pos`.
        if tok.type_0 > MPACK_TOKEN_CHUNK && (*top).pos < tok.length as size_t {
            return None;
        }

        // Frame 1's neighbour below is the sentinel, so a root object has no
        // parent to report to.
        if size > 1 {
            let parent = frame(p, size - 1);
            if tok.type_0 == MPACK_TOKEN_CHUNK {
                // A blob's children are bytes, not items.
                (*parent).pos += tok.length as size_t;
            } else if (*parent).tok.type_0 == MPACK_TOKEN_MAP {
                // A map may hold up to 2^32 - 1 pairs, which will not fit a
                // 32-bit count of *visited children*; the key/value phase
                // rides alongside `pos` in its own flag instead.
                if (*parent).key_visited != 0 {
                    (*parent).pos += 1;
                }
                (*parent).key_visited = c_int::from((*parent).key_visited == 0);
            } else {
                (*parent).pos += 1;
            }
        }

        (*p).size -= 1;
        Some(top)
    }
}
