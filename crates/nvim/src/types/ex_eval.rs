#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// What [`enter_cleanup`](crate::ex_eval::enter_cleanup) parked for
/// [`leave_cleanup`](crate::ex_eval::leave_cleanup) to restore.
pub struct Cleanup {
    pub pending: ::core::ffi::c_int,
    pub(crate) exception: Option<ExcId>,
}

/// How deep `:if`/`:while`/`:for`/`:try` may nest.
pub(crate) const COND_STACK_DEPTH: usize = 50;

/// What a `:finally` postponed at one level of the condition stack: the
/// value a `:return` carries, which the level owns, or the exception, which
/// the exception table owns. Upstream's union of two pointer arrays, with
/// its tag made explicit.
#[derive(Default)]
pub(crate) enum Pend {
    #[default]
    None,
    /// A `:return`'s value; `None` for a bare `:return`.
    Return(Option<TypVal>),
    Exception(ExcId),
}

/// The `:if`/`:while`/`:for`/`:try` nesting of one running command line.
///
/// A level is pushed by the opening command and popped by its end command;
/// each array is indexed by level, and `idx` is the innermost, -1 when none
/// is open. A stack is reached by its [`CondId`] through the condition-stack
/// table, which keeps the ones not in use for the next command line.
pub(crate) struct CondStack {
    /// What each level is and how it stands.
    pub(crate) flags: [crate::ex_eval::CsFlags; COND_STACK_DEPTH],
    /// What a `:finally` at each level postponed: one of the `CSTP_*`
    /// values.
    pub(crate) pending: [::core::ffi::c_int; COND_STACK_DEPTH],
    /// The value or exception that goes with `pending`, and the exception an
    /// active catch clause caught. Reached through the accessors below.
    pend: [Pend; COND_STACK_DEPTH],
    /// The iteration of each open `:for`.
    pub(crate) for_info: [Option<Box<crate::eval::ForInfo>>; COND_STACK_DEPTH],
    /// Where in the stored lines each loop's body starts, -1 before it is
    /// known.
    pub(crate) line: [::core::ffi::c_int; COND_STACK_DEPTH],
    pub(crate) idx: ::core::ffi::c_int,
    pub(crate) loop_level: ::core::ffi::c_int,
    pub(crate) try_level: ::core::ffi::c_int,
    /// The `emsg_silent` each `:try` that reset it found, innermost last.
    pub(crate) saved_emsg_silent: Vec<::core::ffi::c_int>,
    pub(crate) loop_flags: crate::ex_eval::CsLoopFlags,
    /// How many levels have ever been open: the ones [`CondStack::clear`]
    /// has to look at.
    used: usize,
}

impl CondStack {
    /// An empty stack.
    pub(crate) fn new() -> CondStack {
        CondStack {
            flags: [crate::ex_eval::CsFlags::NONE; COND_STACK_DEPTH],
            pending: [0; COND_STACK_DEPTH],
            pend: ::core::array::from_fn(|_| Pend::None),
            for_info: ::core::array::from_fn(|_| None),
            line: [0; COND_STACK_DEPTH],
            idx: -1,
            loop_level: 0,
            try_level: 0,
            saved_emsg_silent: Vec::new(),
            loop_flags: crate::ex_eval::CsLoopFlags::NONE,
            used: 0,
        }
    }

    /// Whether every level is open.
    pub(crate) fn is_full(&self) -> bool {
        self.top() == Some(COND_STACK_DEPTH - 1)
    }

    /// Open a level and answer it. The caller has checked
    /// [`CondStack::is_full`].
    pub(crate) fn push(&mut self) -> usize {
        self.idx += 1;
        let at = self.top().expect("a level was just opened");
        self.used = self.used.max(at + 1);
        at
    }

    /// Empty the stack for its next command line, handing back what its
    /// levels still held for the caller to release.
    pub(crate) fn clear(&mut self) -> Vec<(Option<Box<crate::eval::ForInfo>>, Pend)> {
        let left = (0..self.used)
            .map(|at| (self.for_info[at].take(), self.take_pend(at)))
            .filter(|(info, pend)| info.is_some() || !matches!(pend, Pend::None))
            .collect();
        self.idx = -1;
        self.loop_level = 0;
        self.try_level = 0;
        self.saved_emsg_silent.clear();
        self.loop_flags = crate::ex_eval::CsLoopFlags::NONE;
        self.used = 0;
        left
    }

    /// The innermost level, if one is open.
    pub(crate) fn top(&self) -> Option<usize> {
        usize::try_from(self.idx).ok()
    }

    /// Whether the innermost level is open and not active.
    pub(crate) fn innermost_inactive(&self) -> bool {
        self.top()
            .is_some_and(|at| !self.flags[at].has(crate::ex_eval::CsFlags::ACTIVE))
    }

    /// What the command loop reads between commands.
    pub(crate) fn summary(&self) -> CondSummary {
        CondSummary {
            idx: self.idx,
            loop_level: self.loop_level,
            try_level: self.try_level,
            in_inactive: self.innermost_inactive(),
        }
    }

    /// The innermost level's flags; `NONE` when none is open.
    pub(crate) fn top_flags(&self) -> crate::ex_eval::CsFlags {
        self.top()
            .map_or(crate::ex_eval::CsFlags::NONE, |at| self.flags[at])
    }

    /// Take what level `at` has pending, leaving nothing.
    pub(crate) fn take_pend(&mut self, at: usize) -> Pend {
        ::core::mem::take(&mut self.pend[at])
    }

    /// Postpone a `:return`'s value at level `at`.
    pub(crate) fn set_pending_return(&mut self, at: usize, value: Option<TypVal>) {
        self.pend[at] = Pend::Return(value);
    }

    /// The exception postponed at level `at`, or the one its active catch
    /// clause caught. Meaningful when `pending[at]` carries `CSTP_THROW`, and
    /// when the level is in an active catch clause.
    pub(crate) fn pending_exception(&self, at: usize) -> Option<ExcId> {
        match self.pend[at] {
            Pend::Exception(id) => Some(id),
            _ => None,
        }
    }

    /// Postpone an exception at level `at`.
    pub(crate) fn set_pending_exception(&mut self, at: usize, exception: Option<ExcId>) {
        self.pend[at] = exception.map_or(Pend::None, Pend::Exception);
    }
}

/// The parts of a condition stack the command loop reads between commands:
/// all that changes them is a command, so the loop asks once after each.
pub(crate) struct CondSummary {
    pub(crate) idx: ::core::ffi::c_int,
    pub(crate) loop_level: ::core::ffi::c_int,
    pub(crate) try_level: ::core::ffi::c_int,
    /// The innermost conditional is not active: its commands are parsed,
    /// not run.
    pub(crate) in_inactive: bool,
}

impl CondSummary {
    /// An empty stack's.
    pub(crate) const EMPTY: CondSummary = CondSummary {
        idx: -1,
        loop_level: 0,
        try_level: 0,
        in_inactive: false,
    };
}

/// A condition stack's place in the condition-stack table.
pub(crate) type CondId = crate::id_table::TableId<CondStack>;

pub type ExceptType = ::core::ffi::c_uint;
pub struct ExceptionState {
    pub(crate) estate_current_exception: Option<ExcId>,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: ::core::ffi::c_int,
    pub estate_did_emsg: ::core::ffi::c_int,
}

/// One error message an error exception may be built from.
pub(crate) struct ErrorMsg {
    pub(crate) msg: crate::memory::XString,
    /// The script it came from, when there was one.
    pub(crate) sfile: Option<crate::memory::XString>,
    pub(crate) slnum: LineNr,
    pub(crate) multiline: bool,
}

/// The error messages one command gave on its way to becoming an error
/// exception: upstream's `msglist_T` chain, plus where in it the exception
/// value starts (the head's `throw_msg`) as an entry and a byte offset.
#[derive(Default)]
pub(crate) struct ErrorMsgs {
    pub(crate) entries: Vec<ErrorMsg>,
    pub(crate) throw_at: (usize, usize),
}

impl ErrorMsgs {
    /// The text the exception value is made from.
    pub(crate) fn throw_msg(&self) -> &[u8] {
        let (entry, offset) = self.throw_at;
        self.entries
            .get(entry)
            .map_or(&[], |m| &m.msg[offset.min(m.msg.len())..])
    }
}

pub struct Exception {
    pub type_0: ExceptType,
    /// What a `:catch` pattern is matched against: `v:exception`.
    pub(crate) value: crate::memory::XString,
    /// What an error exception was built from; empty for the other kinds.
    pub(crate) messages: ErrorMsgs,
    /// Where it was thrown: a script or function name, empty for a typed
    /// command. With `throw_lnum`, `v:throwpoint`.
    pub(crate) throw_name: crate::memory::XString,
    pub throw_lnum: LineNr,
    /// The stack trace the exception was thrown with, which it owns.
    pub stacktrace: Option<ListRef>,
}

/// An exception's place in the exception table.
pub(crate) type ExcId = crate::id_table::TableId<Exception>;
