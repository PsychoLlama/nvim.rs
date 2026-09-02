#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The requests this editor has sent over a channel and is still waiting on.
//!
//! A frame lives on the stack of the `rpc_send_call` that pushed it, so the
//! stack stores raw pointers. The request id is copied in alongside, which is
//! what lets the whole matching policy be decided here without reading through
//! any of them.

use crate::types::{ChannelCallFrame, uint32_t};

/// One outstanding request.
struct Entry {
    /// The id sent on the wire. Fixed for the frame's life, so it can be
    /// compared without touching the frame.
    request_id: uint32_t,
    frame: *mut ChannelCallFrame,
}

/// Outstanding requests, oldest first.
///
/// Nested calls unwind in order — the reply that resumes a `rpc_send_call`
/// pops the frame that call pushed — so this is a stack, not a map, even
/// though a well-behaved msgpack-rpc peer is allowed to answer out of order.
#[derive(Default)]
pub struct CallStack {
    entries: Vec<Entry>,
}

impl CallStack {
    pub const fn new() -> Self {
        CallStack {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, request_id: uint32_t, frame: *mut ChannelCallFrame) {
        self.entries.push(Entry { request_id, frame });
    }

    /// Drops the most recent frame.
    ///
    /// Unconditional, like the `kv_size(..)--` it replaces: `rpc_send_call`
    /// pops after its wait whatever else happened in the meantime.
    pub fn pop(&mut self) {
        self.entries.pop();
    }

    /// The most recent frame, if the id matches.
    ///
    /// This is the policy for every client that is not `msgpack-rpc`: such a
    /// peer is expected to answer the request it is being asked about right
    /// now, and anything else is a synchronisation error.
    ///
    /// Upstream spelled this `kv_last(call_stack)` and then checked the result
    /// for NULL, which an empty kvec never produces — it indexes one element
    /// *before* the (possibly null) item array. Answering `None` for an empty
    /// stack is what that null check was written to catch, and it is reachable
    /// from any peer that sends an unsolicited response.
    pub fn top_matching(&self, request_id: uint32_t) -> Option<*mut ChannelCallFrame> {
        self.entries
            .last()
            .filter(|e| e.request_id == request_id)
            .map(|e| e.frame)
    }

    /// The newest frame waiting on `request_id`.
    ///
    /// Only `msgpack-rpc` clients get this: they are permitted to interleave,
    /// so a reply may belong to a call further down the stack. Searching from
    /// the top means the most recent of two frames sharing an id wins, which
    /// is what a wrapped `next_request_id` produces.
    pub fn find(&self, request_id: uint32_t) -> Option<*mut ChannelCallFrame> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.request_id == request_id)
            .map(|e| e.frame)
    }

    /// Every outstanding frame, oldest first.
    ///
    /// The order matters to `chan_close_on_err`, which fills each unanswered
    /// frame with the same error message.
    pub fn frames(&self) -> impl Iterator<Item = *mut ChannelCallFrame> + '_ {
        self.entries.iter().map(|e| e.frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr;

    fn frame(n: usize) -> *mut ChannelCallFrame {
        ptr::without_provenance_mut(n * 16 + 16)
    }

    #[test]
    fn an_empty_stack_matches_nothing() {
        let stack = CallStack::new();
        assert!(stack.top_matching(0).is_none());
        assert!(stack.top_matching(7).is_none());
        assert!(stack.find(7).is_none());
        assert_eq!(stack.frames().count(), 0);
    }

    #[test]
    fn the_top_only_answers_to_its_own_id() {
        let mut stack = CallStack::new();
        stack.push(3, frame(0));
        stack.push(4, frame(1));
        assert_eq!(stack.top_matching(4), Some(frame(1)));
        assert!(stack.top_matching(3).is_none());
    }

    #[test]
    fn find_reaches_below_the_top() {
        let mut stack = CallStack::new();
        stack.push(3, frame(0));
        stack.push(4, frame(1));
        assert_eq!(stack.find(3), Some(frame(0)));
        assert_eq!(stack.find(4), Some(frame(1)));
        assert!(stack.find(5).is_none());
    }

    #[test]
    fn find_prefers_the_newer_of_two_frames_sharing_an_id() {
        let mut stack = CallStack::new();
        stack.push(9, frame(0));
        stack.push(9, frame(1));
        assert_eq!(stack.find(9), Some(frame(1)));
    }

    #[test]
    fn pop_restores_the_previous_top() {
        let mut stack = CallStack::new();
        stack.push(1, frame(0));
        stack.push(2, frame(1));
        stack.pop();
        assert_eq!(stack.top_matching(1), Some(frame(0)));
        stack.pop();
        assert!(stack.top_matching(1).is_none());
        stack.pop();
    }

    #[test]
    fn frames_are_visited_oldest_first() {
        let mut stack = CallStack::new();
        stack.push(1, frame(0));
        stack.push(2, frame(1));
        stack.push(3, frame(2));
        let seen: Vec<_> = stack.frames().collect();
        assert_eq!(seen, vec![frame(0), frame(1), frame(2)]);
    }
}
