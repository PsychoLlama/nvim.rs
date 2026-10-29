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
//! The channel owns the frames: `rpc_send_call` pushes one, waits until the
//! decoder has filled it in (or the channel has closed), and pops it again.
//! The whole matching policy is decided here, by request id.

use crate::types::{ChannelCallFrame, uint32_t};

/// Outstanding requests, oldest first.
///
/// Nested calls unwind in order — the reply that resumes a `rpc_send_call`
/// pops the frame that call pushed — so this is a stack, not a map, even
/// though a well-behaved msgpack-rpc peer is allowed to answer out of order.
#[derive(Default)]
pub struct CallStack {
    frames: Vec<ChannelCallFrame>,
}

impl CallStack {
    pub const fn new() -> Self {
        CallStack { frames: Vec::new() }
    }

    /// Pushes a frame for a request just sent; answers its depth, which is
    /// how the waiting call finds it again.
    pub fn push(&mut self, frame: ChannelCallFrame) -> usize {
        self.frames.push(frame);
        self.frames.len() - 1
    }

    /// Takes the most recent frame back.
    ///
    /// Unconditional, like the `kv_size(..)--` it replaces: `rpc_send_call`
    /// pops after its wait whatever else happened in the meantime, and every
    /// call nested inside it has popped its own by then.
    pub fn pop(&mut self) -> Option<ChannelCallFrame> {
        self.frames.pop()
    }

    /// The frame `depth` down from the bottom.
    pub fn frame(&self, depth: usize) -> &ChannelCallFrame {
        &self.frames[depth]
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
    pub fn top_matching(&mut self, request_id: uint32_t) -> Option<&mut ChannelCallFrame> {
        self.frames
            .last_mut()
            .filter(|frame| frame.request_id == request_id)
    }

    /// The newest frame waiting on `request_id`.
    ///
    /// Only `msgpack-rpc` clients get this: they are permitted to interleave,
    /// so a reply may belong to a call further down the stack. Searching from
    /// the top means the most recent of two frames sharing an id wins, which
    /// is what a wrapped `next_request_id` produces.
    pub fn find(&mut self, request_id: uint32_t) -> Option<&mut ChannelCallFrame> {
        self.frames
            .iter_mut()
            .rev()
            .find(|frame| frame.request_id == request_id)
    }

    /// Every outstanding frame, oldest first.
    ///
    /// The order matters to `chan_close_on_err`, which fills each unanswered
    /// frame with the same error message.
    pub fn frames_mut(&mut self) -> impl Iterator<Item = &mut ChannelCallFrame> + '_ {
        self.frames.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Object;

    fn frame(request_id: uint32_t, tag: i64) -> ChannelCallFrame {
        ChannelCallFrame {
            request_id,
            returned: false,
            errored: false,
            result: Object::Integer(tag),
        }
    }

    fn tag(frame: Option<&mut ChannelCallFrame>) -> Option<i64> {
        frame.and_then(|f| f.result.as_integer())
    }

    #[test]
    fn an_empty_stack_matches_nothing() {
        let mut stack = CallStack::new();
        assert!(stack.top_matching(0).is_none());
        assert!(stack.top_matching(7).is_none());
        assert!(stack.find(7).is_none());
        assert_eq!(stack.frames_mut().count(), 0);
    }

    #[test]
    fn the_top_only_answers_to_its_own_id() {
        let mut stack = CallStack::new();
        stack.push(frame(3, 0));
        stack.push(frame(4, 1));
        assert_eq!(tag(stack.top_matching(4)), Some(1));
        assert!(stack.top_matching(3).is_none());
    }

    #[test]
    fn find_reaches_below_the_top() {
        let mut stack = CallStack::new();
        stack.push(frame(3, 0));
        stack.push(frame(4, 1));
        assert_eq!(tag(stack.find(3)), Some(0));
        assert_eq!(tag(stack.find(4)), Some(1));
        assert!(stack.find(5).is_none());
    }

    #[test]
    fn find_prefers_the_newer_of_two_frames_sharing_an_id() {
        let mut stack = CallStack::new();
        stack.push(frame(9, 0));
        stack.push(frame(9, 1));
        assert_eq!(tag(stack.find(9)), Some(1));
    }

    #[test]
    fn pop_restores_the_previous_top_and_depth_finds_a_frame() {
        let mut stack = CallStack::new();
        assert_eq!(stack.push(frame(1, 0)), 0);
        assert_eq!(stack.push(frame(2, 1)), 1);
        assert_eq!(stack.frame(0).request_id, 1);
        assert_eq!(stack.pop().map(|f| f.request_id), Some(2));
        assert_eq!(tag(stack.top_matching(1)), Some(0));
        assert!(stack.pop().is_some());
        assert!(stack.top_matching(1).is_none());
        assert!(stack.pop().is_none());
    }

    #[test]
    fn frames_are_visited_oldest_first() {
        let mut stack = CallStack::new();
        stack.push(frame(1, 0));
        stack.push(frame(2, 1));
        stack.push(frame(3, 2));
        let seen: Vec<_> = stack.frames_mut().map(|f| f.request_id).collect();
        assert_eq!(seen, vec![1, 2, 3]);
    }
}
