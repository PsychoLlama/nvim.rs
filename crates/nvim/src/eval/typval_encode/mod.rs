//! The container walk every typval encoder shares: upstream's
//! `eval/typval_encode.c.h`.
//!
//! Upstream spells one algorithm as a 132-line header that is `#include`d
//! **seven** times — msgpack and JSON in `eval/encode/`, `string()` and
//! `echo` beside them, Lua in `lua/converter/`, api `Object`s in
//! `api/private/converter.rs`, and the `nothing` sink `tv_clear` deep-frees
//! with in `eval/typval/`.  Each includer defines a set of
//! `TYPVAL_ENCODE_CONV_*` macros and the header emits the same walk around
//! them; transpiled, that came to some 12,700 lines of one algorithm.  Here
//! the walk is written once, generic over [`TypvalSink`], and each includer
//! becomes an `impl`.  Generic, not `dyn`: the walk is monomorphised per sink
//! so every hook stays the direct call the macro expansion was.
//!
//! The walk is deliberately **not recursive**.  Containers are pushed onto an
//! explicit stack ([`ConvStack`]), and a container already on it is a
//! self-reference, recognised instead of overflowing the machine stack.  That
//! is the whole reason upstream wrote it this way, and it is why a hook can
//! only ask the walk to stop ([`Flow`]) — it never gets to decline the descent.
//!
//! Upstream finds "already on the stack" by stamping each container with the
//! walk's `copyID` as it is pushed and restoring the old one as it is popped.
//! A frame here holds a *borrow* of its container, so the stack itself answers
//! the question: the stamp only ever meant "some frame below holds this", and
//! asking the frames is that, with nothing written to the values walked.  A
//! container is on the stack through a `List`/`Pairs` frame (a list) or a
//! `Dict` frame (a dictionary); a partial is never stamped, so never asked.
//!
//! The walk only reads.  The seventh sink upstream has — `nothing`, the deep
//! free `tv_clear` runs — writes to every slot it passes; it is a walk of its
//! own beside `tv_clear` now ([`super::typval::value`]), because a free can
//! take each value out of its slot and needs none of this.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::types::{Dict, Float, List, Partial, TypVal};

// The walk itself; this half is the contract it runs against.
mod walk;
pub(crate) use self::walk::encode_typval;

/// The encode was abandoned.
///
/// Only a sink refuses — with `Flow::Fail`, having reported which value it
/// could not represent — or the walk meets a `VAR_UNKNOWN`, which is an
/// internal error and reports itself. Nothing is left to hand back, which is
/// why this is a unit struct and not the `Result<(), ()>` it replaces.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Refused;

/// What a hook tells the walk to do next.
///
/// Upstream's hooks say this by falling through, by `goto`ing the
/// `typval_encode_stop_converting_one_item` label, or by `return FAIL` — the
/// label meaning two different things depending on which of the header's two
/// functions the macro was expanded into.  Here it is one verdict and the two
/// call sites read it their own way.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Flow {
    /// Carry on.  The macro fell through.
    Go,
    /// Stop converting this value and resume the stack walk.
    Stop,
    /// Abandon the encode.
    Fail,
}

/// The three container kinds a self-reference can be met as: upstream's
/// `MPConvStackValType` less the two partial stages, which are never
/// `copyID`-marked.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ConvType {
    Dict,
    List,
    /// A special dictionary's `_VAL`, a list of `[key, value]` pairs walked as
    /// though it were a dictionary.
    Pairs,
}

/// A container the walk has met: what a self-reference names.
#[derive(Copy, Clone)]
pub(crate) enum Container<'a> {
    List(&'a List),
    Dict(&'a Dict),
}

/// Which of a partial's three parts the walk is up to.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum PartialStage {
    Args,
    Self_,
    End,
}

/// One suspended container: upstream's `MPConvStackVal`, whose `type` tag and
/// `data` union become one enum, each arm holding a borrow of the container
/// and the position the walk has reached in it.
#[derive(Copy, Clone)]
pub(crate) enum Frame<'a> {
    Dict {
        dict: &'a Dict,
        /// The next hash-table slot to look at -- an index, so the frame
        /// is `Copy` and the key handed out last is the slot before it.
        slot: usize,
        /// How many items are still to come.
        todo: usize,
    },
    List {
        list: &'a List,
        /// The next item. `at == list.len()` is a drained frame.
        at: usize,
    },
    Pairs {
        list: &'a List,
        /// As [`Frame::List`].
        at: usize,
    },
    Partial {
        stage: PartialStage,
        /// `None` for `v:_null_partial`, which still prints as a funcref.
        partial: Option<&'a Partial>,
    },
    PartialArgs {
        argv: &'a [TypVal],
        /// The next argument.
        at: usize,
    },
}

impl Frame<'_> {
    /// Whether this frame walks `container` under the kind `conv_type`, for a
    /// sink resolving a self-reference back to a stack position.
    ///
    /// Note that a `Pairs` frame answers only to `Pairs`, never to `List`:
    /// upstream's backref searches compare the *tag* first, and a special
    /// map's `_VAL` list is therefore never found by a `kMPConvList` lookup.
    /// Keep that asymmetry — the index it produces is in the `@N` markers
    /// `string()` and `echo` print.
    pub(crate) fn walks(&self, conv_type: ConvType, container: Container<'_>) -> bool {
        match (*self, conv_type, container) {
            (Frame::Dict { dict, .. }, ConvType::Dict, Container::Dict(other)) => {
                ::core::ptr::eq(dict, other)
            }
            (Frame::List { list, .. }, ConvType::List, Container::List(other))
            | (Frame::Pairs { list, .. }, ConvType::Pairs, Container::List(other)) => {
                ::core::ptr::eq(list, other)
            }
            _ => false,
        }
    }

    /// Whether this frame holds `container` at all, whatever kind it walks
    /// it as: upstream's `copyID` stamp, which a list carries the same way
    /// whether a `List` or a `Pairs` frame put it there.
    fn holds(&self, container: Container<'_>) -> bool {
        match (*self, container) {
            (Frame::Dict { dict, .. }, Container::Dict(other)) => ::core::ptr::eq(dict, other),
            (Frame::List { list, .. } | Frame::Pairs { list, .. }, Container::List(other)) => {
                ::core::ptr::eq(list, other)
            }
            _ => false,
        }
    }
}

/// Frames held without allocating, as upstream's `MPConvStack` is a
/// `kvec_withinit_t` of the same size: most walks never go deeper.
const INLINE_FRAMES: usize = 8;

/// The walk's explicit stack of suspended containers.
pub(crate) type ConvStack<'a> = InlineStack<Frame<'a>, INLINE_FRAMES>;

impl ConvStack<'_> {
    /// Whether some frame already holds `container`: the walk has been here
    /// on the way down, so meeting it again is a self-reference.
    pub(crate) fn holds(&self, container: Container<'_>) -> bool {
        self.iter().any(|frame| frame.holds(container))
    }
}

/// A stack of `N` items held inline, spilling to the heap beyond that:
/// klib's `kvec_withinit_t`, which upstream uses for the walk's frames.
///
/// Indexable from the bottom, because that is the order the error path names
/// the frames in and the position a `@N` self-reference marker counts to.
pub(crate) struct InlineStack<T, const N: usize> {
    inline: [Option<T>; N],
    spilled: Vec<T>,
    len: usize,
}

impl<T, const N: usize> InlineStack<T, N> {
    pub(crate) fn new() -> Self {
        InlineStack {
            inline: [const { None }; N],
            spilled: Vec::new(),
            len: 0,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        match self.inline.get_mut(self.len) {
            Some(slot) => *slot = Some(item),
            None => self.spilled.push(item),
        }
        self.len += 1;
    }

    /// Take the top item off.
    pub(crate) fn pop(&mut self) -> Option<T> {
        self.len = self.len.checked_sub(1)?;
        match self.inline.get_mut(self.len) {
            Some(slot) => slot.take(),
            None => self.spilled.pop(),
        }
    }

    pub(crate) fn get_mut(&mut self, i: usize) -> &mut T {
        debug_assert!(i < self.len);
        match self.inline.get_mut(i) {
            Some(slot) => slot.as_mut().expect("an item below the top"),
            None => &mut self.spilled[i - N],
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The items from the bottom of the stack upwards.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.inline[..self.len.min(N)]
            .iter()
            .flatten()
            .chain(self.spilled.iter())
    }
}

/// What the two failing hooks need to name the value they failed on: the path
/// down to it and the name of the object being dumped.
pub(crate) struct ConvPath<'s, 'a> {
    pub stack: &'s ConvStack<'a>,
    pub objname: &'s CStr,
}

/// The `TYPVAL_ENCODE_CONV_*` macros one includer of `typval_encode.c.h`
/// defines, as one trait.
///
/// The ones with a `Flow` return are the ones some sink uses to stop; the
/// rest are `()` because no sink needs to, and the defaults are for the ones
/// most sinks leave empty.  Nothing here is handed the value the walk stands
/// on: upstream passes it for the `nothing` sink's sake, and no reading sink
/// looks at it.
pub(crate) trait TypvalSink {
    /// `TYPVAL_ENCODE_ALLOW_SPECIALS`: whether a two-key `{_TYPE, _VAL}`
    /// dictionary is read as the value it stands for rather than as a plain
    /// dictionary.
    const ALLOW_SPECIALS: bool;

    /// The name `internal_error` reports for a `VAR_UNKNOWN`, which upstream
    /// spells with the instantiation's own function name.
    const CONVERT_FN_NAME: &'static CStr;

    /// `TYPVAL_ENCODE_CHECK_BEFORE`, run before every value.
    fn check_before(&mut self) {}

    fn conv_nil(&mut self);
    fn conv_bool(&mut self, num: bool);
    fn conv_number(&mut self, num: i64);
    /// Only reachable through a special dictionary, so sinks that refuse
    /// those leave it empty.
    fn conv_unsigned_number(&mut self, num: u64) {
        let _ = num;
    }
    fn conv_float(&mut self, flt: Float) -> Flow;

    /// A `VAR_STRING`: its bytes, with a NULL string reading as none.
    fn conv_string(&mut self, bytes: &[u8]) -> Flow;
    /// A string that is known to be text rather than bytes: a dictionary key,
    /// or a special string's contents.  Only msgpack tells the two apart.
    fn conv_str_string(&mut self, bytes: &[u8]) -> Flow {
        self.conv_string(bytes)
    }
    /// A dictionary key, which is always a plain byte string.
    ///
    /// Split from [`Self::conv_str_string`] so that a sink whose keys are not
    /// values -- the API's, whose key is a field of the entry rather than an
    /// object on the stack -- can take the bytes without building one. The
    /// default is what every other sink wants: a key is a string like any
    /// other, and [`Self::conv_dict_after_key`] moves it into place.
    fn conv_dict_key(&mut self, key: &[u8]) -> Flow {
        self.conv_str_string(key)
    }
    /// A special `ext` value: its type code and payload.
    fn conv_ext_string(&mut self, bytes: &[u8], ext_type: i8) -> Flow;

    fn conv_blob(&mut self, bytes: &[u8]);

    /// A funcref or partial, before its arguments.  `fun` is `None` for a
    /// NULL name; `prefix` is `"g:"` where the name needs qualifying.
    fn conv_func_start(
        &mut self,
        fun: Option<&CStr>,
        prefix: &'static CStr,
        path: &ConvPath<'_, '_>,
    ) -> Flow;
    fn conv_func_before_args(&mut self, len: usize) {
        let _ = len;
    }
    /// `len` is `None` when the partial has no self dictionary.
    fn conv_func_before_self(&mut self, len: Option<usize>) {
        let _ = len;
    }
    fn conv_func_end(&mut self) {}

    fn conv_empty_list(&mut self);
    fn conv_empty_dict(&mut self);

    fn conv_list_start(&mut self, len: c_int) -> Flow;
    fn conv_list_between_items(&mut self) {}
    fn conv_list_end(&mut self) {}

    fn conv_dict_start(&mut self, len: usize) -> Flow;
    /// `TYPVAL_ENCODE_SPECIAL_DICT_KEY_CHECK`: veto a key a special map is
    /// about to emit.
    fn special_dict_key_check(&mut self, key: &TypVal) -> Flow {
        let _ = key;
        Flow::Go
    }
    fn conv_dict_after_key(&mut self) {}
    fn conv_dict_between_items(&mut self) {}
    fn conv_dict_end(&mut self) {}

    /// `container`, met as `conv_type`, is already on the stack.  Returning
    /// [`Flow::Go`] means "handled, stop converting this value" — a sink that
    /// writes a marker and one that says nothing both answer that; only the
    /// sinks that refuse self-reference outright answer [`Flow::Fail`].
    fn conv_recurse(
        &mut self,
        container: Container<'_>,
        conv_type: ConvType,
        path: &ConvPath<'_, '_>,
    ) -> Flow;
}
#[cfg(test)]
mod tests {
    use super::InlineStack;

    /// The spill boundary, asserted from both sides.
    ///
    /// How many items [`InlineStack`] holds inline is a *capacity*, and a
    /// capacity is not part of any answer: setting `INLINE_FRAMES` to 1 leaves
    /// every sweep byte-identical (measured, `1787432513-typvalmutate.py
    /// --blind stack-inline-frames`). What a differential *can* see is the
    /// boundary going wrong, because the walk indexes frames from the bottom
    /// and the corpus nests thirty deep — but only as a panic. This says it
    /// precisely.
    #[test]
    fn the_inline_stack_spills_and_comes_back_in_order() {
        let mut stack: InlineStack<usize, 4> = InlineStack::new();
        assert!(stack.is_empty());
        for i in 0..10 {
            stack.push(i);
        }
        assert_eq!(stack.len(), 10);
        // Past the budget the Vec takes over, and the two halves stay one
        // sequence indexed from the bottom.
        for i in 0..10 {
            assert_eq!(*stack.get_mut(i), i, "item {i}");
        }
        assert!(stack.iter().copied().eq(0..10));

        // A write lands on both sides of the boundary.
        *stack.get_mut(9) = 99;
        *stack.get_mut(3) = 98;
        assert_eq!((*stack.get_mut(9), *stack.get_mut(3)), (99, 98));

        // And popping walks back across it in the same order.
        assert_eq!(stack.pop(), Some(99));
        for i in (0..9).rev() {
            assert_eq!(stack.len(), i + 1);
            assert_eq!(stack.pop(), Some(if i == 3 { 98 } else { i }));
        }
        assert!(stack.is_empty());
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.iter().count(), 0);
    }

    /// A stack that never leaves the inline array, and one that never uses it.
    #[test]
    fn the_inline_stack_works_at_both_extremes() {
        let mut inline_only: InlineStack<String, 8> = InlineStack::new();
        inline_only.push("seven".to_owned());
        assert_eq!(inline_only.get_mut(0).as_str(), "seven");
        assert_eq!(inline_only.pop().as_deref(), Some("seven"));
        assert!(inline_only.is_empty());

        // `N == 0` is the degenerate arm the generic has to survive: every
        // push spills.
        let mut always_spills: InlineStack<u8, 0> = InlineStack::new();
        for i in 0..3 {
            always_spills.push(i);
        }
        assert_eq!(always_spills.len(), 3);
        for i in 0..3u8 {
            assert_eq!(*always_spills.get_mut(usize::from(i)), i);
        }
        assert_eq!(always_spills.pop(), Some(2));
        assert_eq!(*always_spills.get_mut(1), 1);
    }
}
