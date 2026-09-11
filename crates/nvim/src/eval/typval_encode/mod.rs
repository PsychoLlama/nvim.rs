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
//! explicit stack ([`ConvStack`]) and marked with the current `copy_id` while
//! they are on it, so a container that references itself is recognised instead
//! of overflowing the machine stack.  That is the whole reason upstream wrote
//! it this way, and it is why a hook can only ask the walk to stop ([`Flow`])
//! — it never gets to decline the descent.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::MaybeUninit;

use crate::eval::typval::DictSlot;

use crate::types::{Blob, Dict, Float, List, Partial, TypVal, int64_t, ptrdiff_t, size_t};

// The walk itself; this half is the contract it runs against.
mod walk;
pub(crate) use self::walk::{encode_typval, encode_typval_read};

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

/// The three container kinds `check_self_reference` can be asked about:
/// upstream's `MPConvStackValType` less the two partial stages, which are
/// never `copy_id`-marked.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ConvType {
    Dict,
    List,
    /// A special dictionary's `_VAL`, a list of `[key, value]` pairs walked as
    /// though it were a dictionary.
    Pairs,
}

/// Which of a partial's three parts the walk is up to.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum PartialStage {
    Args,
    Self_,
    End,
}

/// One suspended container: upstream's `MPConvStackVal`, whose `type` tag and
/// `data` union become one enum.
#[derive(Copy, Clone)]
pub(crate) enum Frame {
    Dict {
        dict: *mut Dict,
        /// Where the dictionary pointer *lives*, so a sink can clear it:
        /// the typval that holds it, or a partial's `pt_dict` field.
        dictp: DictSlot,
        /// The slot the walk stands on -- an *index*, because the small run
        /// lives inside the `HashTab` and a body may take `&mut` to it.
        idx: usize,
        todo: size_t,
    },
    List {
        list: *mut List,
        /// The item the walk stands on -- an *index*, because the list owns
        /// its items and a body may edit it.  `at == tv_list_len(list)` is
        /// a drained frame.
        at: usize,
    },
    Pairs {
        list: *mut List,
        /// As [`Frame::List`].
        at: usize,
    },
    Partial {
        stage: PartialStage,
        pt: *mut Partial,
    },
    PartialArgs {
        arg: *mut TypVal,
        argv: *mut TypVal,
        todo: size_t,
    },
}

/// A stack entry: the value being walked plus the `copy_id` to restore when it
/// is popped.
#[derive(Copy, Clone)]
pub(crate) struct ConvFrame {
    /// The `TypVal` this container came out of.  NULL for the two frames a
    /// partial pushes, which stand for its argument list and self dictionary.
    pub tv: *mut TypVal,
    pub saved_copyid: c_int,
    pub frame: Frame,
}

impl ConvFrame {
    /// The container this frame walks and which kind it is, for a sink
    /// resolving a self-reference back to a stack position.
    ///
    /// Note that a `Pairs` frame answers `Pairs`, never `List`: upstream's
    /// backref searches compare the *tag* first, and a special map's `_VAL`
    /// list is therefore never found by a `kMPConvList` lookup.  Keep that
    /// asymmetry — the index it produces is in the `@N` markers `string()`
    /// and `echo` print.
    pub(crate) fn container(&self) -> Option<(ConvType, *const c_void)> {
        match self.frame {
            Frame::Dict { dict, .. } => Some((ConvType::Dict, dict.cast())),
            Frame::List { list, .. } => Some((ConvType::List, list.cast())),
            Frame::Pairs { list, .. } => Some((ConvType::Pairs, list.cast())),
            Frame::Partial { .. } | Frame::PartialArgs { .. } => None,
        }
    }
}

/// Frames held without allocating.  Upstream's `MPConvStack` is a
/// `kvec_withinit_t` of the same size, and the reason for it is `tv_clear`:
/// the `nothing` sink runs this walk on *every* container the interpreter
/// frees, so a malloc per walk would be a malloc per free.
const INLINE_FRAMES: usize = 8;

/// The walk's explicit stack of suspended containers.
pub(crate) type ConvStack = InlineStack<ConvFrame, INLINE_FRAMES>;

/// A stack of `N` items held inline, spilling to the heap beyond that:
/// klib's `kvec_withinit_t`, which upstream uses both for the walk's own
/// frames and for the half-built values two of the sinks assemble.
///
/// Indexable from the bottom, because that is the order the error path names
/// the frames in and the position a `@N` self-reference marker counts to.
pub(crate) struct InlineStack<T: Copy, const N: usize> {
    /// **Deliberately uninitialised**, exactly as upstream's `kvi_init` leaves
    /// `init_array`.  `tv_clear` runs this walk on every value the interpreter
    /// drops -- scalars included, which never push a frame at all -- so
    /// zeroing eight 56-byte frames on entry is ~5% of the interpreter and 26%
    /// of `evalbench`'s `tvclear` phase.  Measured, not guessed.
    ///
    /// `len` is the invariant: slots below it are initialised, slots at and
    /// above it are not.
    inline: [MaybeUninit<T>; N],
    spilled: Vec<T>,
    len: usize,
}

impl<T: Copy, const N: usize> InlineStack<T, N> {
    pub(crate) fn new() -> Self {
        InlineStack {
            inline: [MaybeUninit::uninit(); N],
            spilled: Vec::new(),
            len: 0,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        if self.len < N {
            self.inline[self.len].write(item);
        } else {
            self.spilled.push(item);
        }
        self.len += 1;
    }

    /// Drop the top item.  The caller has already read whatever it needs out
    /// of it -- upstream's `kv_pop` only decrements the count and its callers
    /// keep reading the popped slot.
    pub(crate) fn pop(&mut self) {
        self.len -= 1;
        if self.len >= N {
            self.spilled.pop();
        }
    }

    pub(crate) fn get_mut(&mut self, i: usize) -> &mut T {
        debug_assert!(i < self.len);
        if i < N {
            // SAFETY: `i < len`, so this slot has been written.
            unsafe { self.inline[i].assume_init_mut() }
        } else {
            &mut self.spilled[i - N]
        }
    }

    pub(crate) fn last_mut(&mut self) -> &mut T {
        let last = self.len - 1;
        self.get_mut(last)
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
            // SAFETY: every slot below `len` has been written.
            .map(|slot| unsafe { slot.assume_init_ref() })
            .chain(self.spilled.iter())
    }
}

/// What the two failing hooks need to name the value they failed on: the path
/// down to it and the name of the object being dumped.
pub(crate) struct ConvPath<'a> {
    pub stack: &'a ConvStack,
    pub objname: &'a CStr,
}

/// The `TYPVAL_ENCODE_CONV_*` macros one includer of `typval_encode.c.h`
/// defines, as one trait.
///
/// Most methods are `unsafe` because they are handed a raw pointer the walk
/// borrowed from its caller.  The ones with a `Flow` return are the ones some
/// sink uses to stop; the rest are `()` because no sink needs to, and the
/// defaults are for the ones most sinks leave empty.
///
/// `tv` is the value the walk is standing on, and `None` where there is
/// none: the two frames a partial pushes stand for its argument list and its
/// self dictionary, neither of which is a typval.
pub(crate) trait TypvalSink {
    /// `TYPVAL_ENCODE_ALLOW_SPECIALS`: whether a two-key `{_TYPE, _VAL}`
    /// dictionary is read as the value it stands for rather than as a plain
    /// dictionary.
    const ALLOW_SPECIALS: bool;

    /// The name `internal_error` reports for a `VAR_UNKNOWN`, which upstream
    /// spells with the instantiation's own function name.
    const CONVERT_FN_NAME: &'static CStr;

    /// Whether the sink *writes* to the values it is walking.
    ///
    /// Only the `nothing` sink does -- it is `tv_clear`'s deep free, and
    /// empties every slot it passes. Every other sink reads, which is what
    /// lets [`encode_typval_read`](crate::eval::typval_encode::encode_typval_read)
    /// take a shared borrow; that entry point refuses a sink that says
    /// `true` here, at compile time.
    const WRITES_BACK: bool = false;

    /// `TYPVAL_ENCODE_CHECK_BEFORE`, run before every value.
    fn check_before(&mut self) {}

    fn conv_nil(&mut self, tv: Option<&mut TypVal>);
    fn conv_bool(&mut self, tv: Option<&mut TypVal>, num: bool);
    fn conv_number(&mut self, tv: Option<&mut TypVal>, num: int64_t);
    /// Only reachable through a special dictionary, so sinks that refuse
    /// those leave it empty.
    ///
    fn conv_unsigned_number(&mut self, tv: Option<&mut TypVal>, num: u64) {
        let _ = (tv, num);
    }
    fn conv_float(&mut self, tv: Option<&mut TypVal>, flt: Float) -> Flow;

    /// A `VAR_STRING`, or the `_VAL` of a special string.  `buf` may be NULL.
    ///
    /// # Safety
    /// The walk's contract on the value it is standing on, above. `buf` is null, or readable for `len` bytes and
    /// owned by the value — it must not be freed or kept past the call.
    unsafe fn conv_string(
        &mut self,
        tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
    ) -> Flow;
    /// A string that is known to be text rather than bytes: a dictionary key,
    /// or a special string's contents.  Only msgpack and `nothing` tell the
    /// two apart.
    ///
    /// For a dictionary key the buffer is the dictionary's.  For a special
    /// string it is the walk's, freed the way [`Self::conv_ext_string`]
    /// describes — so a sink that fails here leaks it, exactly as upstream's
    /// JSON encoder does.
    ///
    /// # Safety
    /// The walk's contract on the value it is standing on, above. `buf` is null, or readable for `len` bytes. For
    /// a dictionary key it belongs to the dictionary; for a special string it
    /// belongs to the walk and is freed on [`Flow::Go`], so an implementation
    /// must not keep it either way.
    unsafe fn conv_str_string(
        &mut self,
        tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
    ) -> Flow {
        unsafe { self.conv_string(tv, buf, len) }
    }
    /// A dictionary key, which is always a plain byte string.
    ///
    /// Split from [`Self::conv_str_string`] so that a sink whose keys are not
    /// values -- the API's, whose key is a field of the entry rather than an
    /// object on the stack -- can take the bytes without building one. The
    /// default is what every other sink wants: a key is a string like any
    /// other, and [`Self::conv_dict_after_key`] moves it into place.
    ///
    /// The bytes belong to the dictionary and are borrowed for the call.
    fn conv_dict_key(&mut self, key: &[u8]) -> Flow {
        // SAFETY: the slice is live for the call, which is all
        // `conv_str_string` asks of the buffer.
        unsafe { self.conv_str_string(None, key.as_ptr().cast::<c_char>().cast_mut(), key.len()) }
    }

    /// A special `ext` value.
    ///
    /// `buf` is the walk's, and the walk frees it — *unless* the hook returns
    /// something other than [`Flow::Go`], in which case it never gets there
    /// and the hook owns it.  Upstream has exactly this split (the `xfree`
    /// sits after the macro, which the bailing sinks `return` past), and one
    /// of the two paths it produces is a leak: see [`Self::conv_str_string`].
    ///
    /// # Safety
    /// The walk's contract on the value it is standing on, above. `buf` is readable for `len` bytes and belongs to
    /// the walk **only while [`Flow::Go`] is returned** — on any other answer
    /// the implementation has taken it over and owes it an `xfree`.
    unsafe fn conv_ext_string(
        &mut self,
        tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
        ext_type: i8,
    ) -> Flow;

    /// # Safety
    /// The walk's contract on the value it is standing on, above. `blob` points at a live blob holding `len` bytes,
    /// borrowed for the call.
    unsafe fn conv_blob(&mut self, tv: Option<&mut TypVal>, blob: *const Blob, len: c_int);

    /// A funcref or partial, before its arguments.  `fun` may be NULL;
    /// `prefix` is `"g:"` where the name needs qualifying.
    ///
    /// # Safety
    /// The walk's contract on the value it is standing on, above. `fun` is null or a NUL-terminated name borrowed
    /// for the call, and `path` borrows the walk's own stack — reading it after
    /// the call would read a stack that has moved on.
    unsafe fn conv_func_start(
        &mut self,
        tv: Option<&mut TypVal>,
        fun: *mut c_char,
        prefix: &'static CStr,
        path: &ConvPath,
    ) -> Flow;
    fn conv_func_before_args(&mut self, tv: Option<&mut TypVal>, len: ptrdiff_t) {
        let _ = (tv, len);
    }
    /// `len` is −1 when the partial has no self dictionary.
    ///
    fn conv_func_before_self(&mut self, tv: Option<&mut TypVal>, len: ptrdiff_t) {
        let _ = (tv, len);
    }
    fn conv_func_end(&mut self, tv: Option<&mut TypVal>, copyid: c_int) {
        let _ = (tv, copyid);
    }

    fn conv_empty_list(&mut self, tv: Option<&mut TypVal>);
    /// `dictp` is where the dictionary pointer lives, so a sink can clear it;
    /// `None` is upstream's `TYPVAL_ENCODE_NODICT_VAR`, meaning the map being
    /// emitted has no `Dict` behind it.
    ///
    /// The dictionary hooks take *no* `tv`: where the walk is standing on a
    /// value at all, that value is the slot, and it arrives as
    /// `DictSlot::Value`.  Handing a hook both would be handing it two
    /// writable paths to one typval.
    ///
    /// # Safety
    /// `dictp`, when given, points at the slot the dictionary pointer lives
    /// in, which an implementation may overwrite but must not free out from
    /// under the walk.
    unsafe fn conv_empty_dict(&mut self, dictp: Option<DictSlot>);

    fn conv_list_start(&mut self, tv: Option<&mut TypVal>, len: c_int) -> Flow;
    /// Called with the frame just pushed for this list, which a sink may edit
    /// to make the walk skip its items.
    ///
    /// # Safety
    /// The walk's contract on the value it is standing on, above. `frame` is the walk's own stack frame
    /// for this list; editing it changes which items the walk visits, and
    /// leaving it inconsistent with the list is what would desynchronise the
    /// walk.
    unsafe fn conv_real_list_after_start(
        &mut self,
        tv: Option<&mut TypVal>,
        frame: &mut ConvFrame,
    ) -> Flow {
        let _ = (tv, frame);
        Flow::Go
    }
    fn conv_list_between_items(&mut self, tv: Option<&mut TypVal>) {
        let _ = tv;
    }
    fn conv_list_end(&mut self, tv: Option<&mut TypVal>) {
        let _ = tv;
    }

    fn conv_dict_start(&mut self, tv: Option<&mut TypVal>, len: size_t) -> Flow;
    /// The dictionary counterpart of [`Self::conv_real_list_after_start`].
    ///
    /// # Safety
    /// As [`Self::conv_real_list_after_start`] for `frame`, and as
    /// [`Self::conv_empty_dict`] for `dictp`.
    unsafe fn conv_real_dict_after_start(
        &mut self,
        dictp: Option<DictSlot>,
        frame: &mut ConvFrame,
    ) -> Flow {
        let _ = (dictp, frame);
        Flow::Go
    }
    /// `TYPVAL_ENCODE_SPECIAL_DICT_KEY_CHECK`: veto a key a special map is
    /// about to emit.
    ///
    fn special_dict_key_check(&mut self, key: &TypVal) -> Flow {
        let _ = key;
        Flow::Go
    }
    /// # Safety
    /// As [`Self::conv_empty_dict`] for `dictp`.
    unsafe fn conv_dict_after_key(&mut self, dictp: Option<DictSlot>) {
        let _ = dictp;
    }
    /// # Safety
    /// As [`Self::conv_empty_dict`] for `dictp`.
    unsafe fn conv_dict_between_items(&mut self, dictp: Option<DictSlot>) {
        let _ = dictp;
    }
    /// # Safety
    /// As [`Self::conv_empty_dict`] for `dictp`.
    unsafe fn conv_dict_end(&mut self, dictp: Option<DictSlot>) {
        let _ = dictp;
    }

    /// The container `val` is already on the stack.  Returning [`Flow::Go`]
    /// means "handled, stop converting this value" — a sink that writes a
    /// marker and one that says nothing both answer that; only the sinks that
    /// refuse self-reference outright answer [`Flow::Fail`].
    ///
    /// # Safety
    /// `val` is the container the walk found itself back at — a `*mut List`,
    /// `*mut Dict` or `*mut Partial` according to `conv_type` — live and
    /// already on the walk's stack. `path` borrows that stack and must not
    /// outlive the call.
    unsafe fn conv_recurse(
        &mut self,
        val: *mut c_void,
        conv_type: ConvType,
        path: &ConvPath,
    ) -> Flow;
}
#[cfg(test)]
mod tests {
    use super::InlineStack;

    /// The spill boundary, asserted from both sides.
    ///
    /// How many frames [`InlineStack`] holds inline is a *capacity*, and a
    /// capacity is not part of any answer: setting `INLINE_FRAMES` to 1 leaves
    /// every sweep byte-identical (measured, `1787432513-typvalmutate.py
    /// --blind stack-inline-frames`), the same shape p20-22 measured for
    /// `garray.rs`. What a differential *can* see is the boundary going wrong,
    /// because `eval/typval_encode`'s walk indexes frames from the bottom and
    /// the corpus nests thirty deep — but only as a panic. This says it
    /// precisely, and being pure it also runs under Miri, which is the only
    /// thing that checks the `MaybeUninit` discipline `inline` is built on.
    #[test]
    fn the_inline_stack_spills_and_comes_back_in_order() {
        let mut stack: InlineStack<usize, 4> = InlineStack::new();
        assert!(stack.is_empty());

        for i in 0..4 {
            stack.push(i);
        }
        assert_eq!(stack.len(), 4);
        assert_eq!(*stack.last_mut(), 3);

        // Past the budget the Vec takes over, and the two halves stay one
        // sequence indexed from the bottom.
        for i in 4..10 {
            stack.push(i);
        }
        assert_eq!(stack.len(), 10);
        for i in 0..10 {
            assert_eq!(*stack.get_mut(i), i, "frame {i}");
        }
        assert_eq!(*stack.last_mut(), 9);

        // A write through `last_mut` lands on both sides of the boundary.
        *stack.last_mut() = 99;
        assert_eq!(*stack.last_mut(), 99);
        *stack.get_mut(3) = 98;
        assert_eq!(*stack.get_mut(3), 98);

        // And popping walks back across it in the same order.
        for i in (0..10).rev() {
            assert_eq!(stack.len(), i + 1);
            stack.pop();
        }
        assert!(stack.is_empty());
    }

    /// A stack that never leaves the inline array, and one that never uses it.
    #[test]
    fn the_inline_stack_works_at_both_extremes() {
        let mut inline_only: InlineStack<u8, 8> = InlineStack::new();
        inline_only.push(7);
        assert_eq!(*inline_only.last_mut(), 7);
        inline_only.pop();
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
        always_spills.pop();
        assert_eq!(*always_spills.last_mut(), 1);
    }
}
