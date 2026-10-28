//! `vim.diff()`: the Lua binding over the bundled xdiff.
//!
//! The three shapes the binding can answer in are [`Mode`]: a unified-diff
//! *string* (the default), *nothing* with an `on_hunk` callback run per hunk,
//! or a *list* of `{start_a, count_a, start_b, count_b}` tuples
//! (`result_type = 'indices'`). The engine runs over the two strings where
//! they sit on the stack, through [`diff_with`]; a hunk callback is called
//! once the walk is done, over the hunks it collected, which is the order
//! and the stopping rule the engine's own callback had.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::api_error;
use crate::linematch::{block_from_lnum, linematch_nbuffers};
use crate::lua::converter::pop_keydict;
use crate::lua::executor::{release_luaref, until_nul};
use crate::lua::stack::{At, CFunction, Kind, Raise, Stack};
use crate::lua_functions;
use crate::message_fmt::msg_bytes_opt;
use crate::types::{Error, KeyDict_xdl_diff, LineNr, Object, kErrorTypeException, lua_Integer};
use crate::xdiff::ffi::{Emit, diff_with};
use crate::xdiff::xtypes::{
    XDF_HISTOGRAM_DIFF, XDF_IGNORE_BLANK_LINES, XDF_IGNORE_CR_AT_EOL, XDF_IGNORE_WHITESPACE,
    XDF_IGNORE_WHITESPACE_AT_EOL, XDF_IGNORE_WHITESPACE_CHANGE, XDF_INDENT_HEURISTIC,
    XDF_NEED_MINIMAL, XDF_PATIENCE_DIFF,
};

lua_functions! {
    /// `vim.diff`.
    enum Diff in DIFF_FUNCTION {
        0 => c"diff": xdl_diff,
    }
}

/// `vim.diff`, for the state that registers it.
pub(crate) fn diff_function() -> CFunction {
    DIFF_FUNCTION[0].entry()
}

/// What `vim.diff` was asked to produce.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// A unified-diff string.
    Unified,
    /// Nothing: `on_hunk` is called once per hunk instead.
    OnHunk,
    /// A list of `{start_a, count_a, start_b, count_b}` tuples.
    Locations,
}

/// A hunk's four coordinates: where the change starts in each document and
/// how many lines it covers there. Zero-based, as `xdl_diff` reports them;
/// the `+ 1` that makes a non-empty side one-based happens on the way out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hunk {
    start_a: c_int,
    count_a: c_int,
    start_b: c_int,
    count_b: c_int,
}

impl Hunk {
    fn from_cb(start_a: c_int, count_a: c_int, start_b: c_int, count_b: c_int) -> Self {
        Hunk {
            start_a,
            count_a,
            start_b,
            count_b,
        }
    }

    /// The four numbers as Lua sees them: a non-empty side is one-based,
    /// an empty one keeps its start where it is.
    fn one_based(self) -> [c_int; 4] {
        [
            if self.count_a > 0 {
                self.start_a + 1
            } else {
                self.start_a
            },
            self.count_a,
            if self.count_b > 0 {
                self.start_b + 1
            } else {
                self.start_b
            },
            self.count_b,
        ]
    }
}

/// The bits `linematch_nbuffers` reports a decision with: which of the two
/// buffers the step consumed a line from.
const COMPARED_BUFFER0: c_int = 1 << 0;
const COMPARED_BUFFER1: c_int = 1 << 1;

/// What the options table asked of the engine, beyond the [`Mode`].
#[derive(Default)]
struct Config {
    /// The `XDF_*` bits.
    flags: u64,
    ctxlen: i64,
    interhunkctxlen: i64,
    linematch: i64,
}

/// A hunk extent as the engine reports it, narrowed to the `int`s upstream's
/// callback took.
fn as_hunk(start_a: i64, count_a: i64, start_b: i64, count_b: i64) -> Hunk {
    #[allow(clippy::cast_possible_truncation)]
    let narrow = |n: i64| n as c_int;
    Hunk::from_cb(
        narrow(start_a),
        narrow(count_a),
        narrow(start_b),
        narrow(count_b),
    )
}

/// The finer hunks `linematch` finds inside `hunk`, reported to `emit`. Runs
/// only when `linematch` is on and the hunk is small enough.
fn linematch_hunks(text_a: &[u8], text_b: &[u8], hunk: Hunk, iwhite: bool, emit: impl FnMut(Hunk)) {
    let block_a = block_from_lnum(text_a, LineNr::from(hunk.start_a) + 1).unwrap_or_default();
    let block_b = block_from_lnum(text_b, LineNr::from(hunk.start_b) + 1).unwrap_or_default();
    let decisions = linematch_nbuffers(&[block_a, block_b], &[hunk.count_a, hunk.count_b], iwhite);
    walk_decisions(hunk, &decisions, emit);
}

/// The finer hunks a run of `linematch` decisions carves `hunk` into.
///
/// Each decision says which of the two buffers its step consumed a line
/// from, so runs of the same decision are one hunk: the walk closes a hunk
/// when the decision changes and once more at the end. It therefore always
/// emits at least one hunk, even for no decisions at all.
fn walk_decisions(hunk: Hunk, decisions: &[c_int], mut emit: impl FnMut(Hunk)) {
    let mut lnuma = hunk.start_a;
    let mut lnumb = hunk.start_b;
    let mut current = Hunk::from_cb(lnuma, 0, lnumb, 0);
    for (i, &decision) in decisions.iter().enumerate() {
        if i != 0 && decisions[i - 1] != decision {
            emit(current);
            current = Hunk::from_cb(lnuma, 0, lnumb, 0);
        }
        if decision & COMPARED_BUFFER0 != 0 {
            lnuma += 1;
            current.count_a += 1;
        }
        if decision & COMPARED_BUFFER1 != 0 {
            lnumb += 1;
            current.count_b += 1;
        }
    }
    emit(current);
}

/// Read the options table at slot 3 into `config`, and answer which shape
/// the caller asked for, plus why a bad option stopped the read where it
/// was. An `on_hunk` callback is left on top of the stack.
fn process_xdl_diff_opts(stack: &mut Stack, config: &mut Config) -> (Mode, Option<Error>) {
    let mut opts = KeyDict_xdl_diff::default();
    let mut blame = None;
    let popped = pop_keydict(stack, &mut opts, &mut blame);

    // A bad option outranks a failed pop, which is the order the two had
    // when both wrote through one slot.
    let (mode, why) = apply_opts(stack, &opts, config);

    if let Some(on_hunk) = opts.on_hunk {
        release_luaref(on_hunk);
    }
    (mode, why.or(popped.err()))
}

/// The body of [`process_xdl_diff_opts`], split out so the keydict is freed
/// on every path — upstream's `goto exit`.
fn apply_opts(
    stack: &mut Stack,
    opts: &KeyDict_xdl_diff,
    config: &mut Config,
) -> (Mode, Option<Error>) {
    let mut had_result_type_indices = false;
    if let Some(result_type) = opts.result_type.as_ref()
        && result_type.as_cstr() != c"unified"
    {
        if result_type.as_cstr() == c"indices" {
            had_result_type_indices = true;
        } else {
            let why = Error::validation(c"not a valid result_type");
            return (Mode::Unified, Some(why));
        }
    }

    if let Some(named) = opts.algorithm.as_ref()
        && named.as_cstr() != c"myers"
    {
        let algorithm = [
            (c"minimal", XDF_NEED_MINIMAL),
            (c"patience", XDF_PATIENCE_DIFF),
            (c"histogram", XDF_HISTOGRAM_DIFF),
        ]
        .into_iter()
        .find(|(name, _)| *name == named.as_cstr());
        match algorithm {
            Some((_, flag)) => config.flags |= flag,
            None => {
                let why = Error::validation(c"not a valid algorithm");
                return (Mode::Unified, Some(why));
            }
        }
    }

    if let Some(ctxlen) = opts.ctxlen {
        config.ctxlen = ctxlen;
    }
    if let Some(interhunkctxlen) = opts.interhunkctxlen {
        config.interhunkctxlen = interhunkctxlen;
    }
    if let Some(given) = opts.linematch.as_ref() {
        match given {
            Object::Boolean(on) => config.linematch = if *on { i64::MAX } else { 0 },
            Object::Integer(n) => config.linematch = *n,
            _ => {
                let why = Error::validation(c"linematch must be a boolean or integer");
                return (Mode::Unified, Some(why));
            }
        }
    }

    for (given, flag) in [
        (opts.ignore_whitespace, XDF_IGNORE_WHITESPACE),
        (opts.ignore_whitespace_change, XDF_IGNORE_WHITESPACE_CHANGE),
        (
            opts.ignore_whitespace_change_at_eol,
            XDF_IGNORE_WHITESPACE_AT_EOL,
        ),
        (opts.ignore_cr_at_eol, XDF_IGNORE_CR_AT_EOL),
        (opts.ignore_blank_lines, XDF_IGNORE_BLANK_LINES),
        (opts.indent_heuristic, XDF_INDENT_HEURISTIC),
    ] {
        if given.unwrap_or(false) {
            config.flags |= flag;
        }
    }

    if let Some(on_hunk) = opts.on_hunk {
        // The callback is left on the stack for the hunk walk to copy.
        stack.push_reference(on_hunk);
        let is_function = stack.kind(At::TOP) == Kind::Function;
        let why = (!is_function).then(|| Error::validation(c"on_hunk is not a function"));
        return (Mode::OnHunk, why);
    }
    if had_result_type_indices {
        return (Mode::Locations, None);
    }
    (Mode::Unified, None)
}

/// One of the two document arguments, checked: a string of at most
/// `INT_MAX` bytes.
fn check_document(stack: &Stack, at: c_int) -> Result<(), Raise> {
    let Some(text) = stack.string(At::arg(at)) else {
        return Err(Raise::argument(at, "expected string"));
    };
    if c_int::try_from(text.len()).is_err() {
        return Err(Raise::argument(at, "string too long"));
    }
    Ok(())
}

/// Call the `on_hunk` callback (on top of the stack) for each of `hunks`, in
/// order. A number it answers below zero stops the walk, and so does a
/// failing callback -- whose error is then the answer.
fn call_on_hunk(stack: &mut Stack, hunks: &[Hunk]) -> Result<(), Option<Error>> {
    let callback = stack.top();
    for hunk in hunks {
        stack.push_value(callback);
        for value in hunk.one_based() {
            stack.push_integer(value as lua_Integer);
        }
        if stack.pcall(4, 1, None).is_err() {
            let why = stack.to_bytes(At::TOP).map(until_nul);
            return Err(Some(api_error!(
                kErrorTypeException,
                "on_hunk: {}",
                msg_bytes_opt(why)
            )));
        }
        // `lua_tonumber(..)` as the C `int` it was cast to.
        #[allow(clippy::cast_possible_truncation)]
        let answer = if stack.is_number(At::TOP) {
            stack.to_number(At::TOP) as c_int
        } else {
            0
        };
        stack.set_top(callback);
        if answer < 0 {
            return Err(None);
        }
    }
    Ok(())
}

/// `vim.diff(a, b[, opts])`.
fn xdl_diff(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() < 2 {
        return Err(Raise::message("Expected at least 2 arguments"));
    }
    // Both stay on the stack for the whole call.
    check_document(stack, 1)?;
    check_document(stack, 2)?;

    let mut err = Error::none();
    let mut config = Config::default();
    let mut mode = Mode::Unified;

    if stack.top().index() == 3 {
        if stack.kind(At::arg(3)) != Kind::Table {
            return Err(Raise::argument(3, "expected table"));
        }
        let why;
        (mode, why) = process_xdl_diff_opts(stack, &mut config);
        if let Some(why) = why {
            err = why;
        }
    }

    // What the walk produced, pushed or called back once it is done.
    let mut unified = Vec::new();
    let mut hunks = Vec::new();
    if !err.is_set() {
        let text_a = stack.string(At::arg(1)).unwrap_or_default();
        let text_b = stack.string(At::arg(2)).unwrap_or_default();
        let iwhite = config.flags & XDF_IGNORE_WHITESPACE > 0;
        let linematch = config.linematch;
        let mut on_line = |parts: &[&[u8]]| {
            for part in parts {
                unified.extend_from_slice(part);
            }
            Ok(())
        };
        let mut on_hunk = |start_a, count_a, start_b, count_b| {
            let hunk = as_hunk(start_a, count_a, start_b, count_b);
            // A hunk wider than `linematch` is not worth refining.
            if mode == Mode::Locations
                && linematch > 0
                && i64::from(hunk.count_a + hunk.count_b) <= linematch
            {
                linematch_hunks(text_a, text_b, hunk, iwhite, |finer| hunks.push(finer));
            } else {
                hunks.push(hunk);
            }
            Ok(())
        };
        let mut emit = match mode {
            Mode::Unified => Emit::Lines(&mut on_line),
            Mode::OnHunk | Mode::Locations => Emit::Hunks(&mut on_hunk),
        };
        let walked = diff_with(
            text_a,
            text_b,
            config.flags,
            config.ctxlen,
            config.interhunkctxlen,
            &mut emit,
        );
        if walked.is_err() {
            err = Error::exception(c"diff operation failed");
        }
    }
    if !err.is_set()
        && mode == Mode::OnHunk
        && let Err(why) = call_on_hunk(stack, &hunks)
    {
        err = why.unwrap_or_else(|| Error::exception(c"diff operation failed"));
    }

    if err.is_set() {
        return Err(Raise::message(err.message_or_empty().to_bytes()));
    }
    match mode {
        Mode::Unified => {
            stack.push_bytes(&unified);
            Ok(1)
        }
        Mode::Locations => {
            stack.new_table(0, 0);
            for (n, hunk) in (1..).zip(hunks) {
                stack.new_table(0, 0);
                for (slot, value) in (1..).zip(hunk.one_based()) {
                    stack.push_integer(value as lua_Integer);
                    stack.raw_set_index(At::from_top(2), slot);
                }
                stack.raw_set_index(At::from_top(2), n);
            }
            Ok(1)
        }
        Mode::OnHunk => Ok(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both buffers consumed a line: the step is an unchanged line.
    const BOTH: c_int = COMPARED_BUFFER0 | COMPARED_BUFFER1;

    /// The hunks `walk_decisions` carves out of `hunk`, collected.
    fn walk(hunk: Hunk, decisions: &[c_int]) -> Vec<Hunk> {
        let mut hunks = Vec::new();
        walk_decisions(hunk, decisions, |finer| hunks.push(finer));
        hunks
    }

    /// A non-empty side is reported one-based; an empty one keeps its start,
    /// because a zero count means "before this line" rather than "at it".
    #[test]
    fn only_a_non_empty_side_shifts_to_one_based() {
        assert_eq!(Hunk::from_cb(3, 2, 7, 4).one_based(), [4, 2, 8, 4]);
        // A pure deletion: nothing in the second document.
        assert_eq!(Hunk::from_cb(3, 2, 7, 0).one_based(), [4, 2, 7, 0]);
        // A pure addition: nothing in the first.
        assert_eq!(Hunk::from_cb(3, 0, 7, 4).one_based(), [3, 0, 8, 4]);
        assert_eq!(Hunk::from_cb(0, 0, 0, 0).one_based(), [0, 0, 0, 0]);
    }

    /// A negative count cannot come out of `xdl_diff`, but the rule is
    /// `> 0` rather than `!= 0` and this is what says so.
    #[test]
    fn a_zero_count_is_the_only_thing_that_holds_the_start() {
        assert_eq!(Hunk::from_cb(5, 1, 5, 1).one_based(), [6, 1, 6, 1]);
        assert_eq!(Hunk::from_cb(5, 0, 5, 0).one_based(), [5, 0, 5, 0]);
    }

    /// No decisions still closes a hunk: `linematch` answering nothing
    /// leaves the empty hunk the walk started with, at the original start.
    #[test]
    fn an_empty_decision_run_still_emits_one_hunk() {
        assert_eq!(
            walk(Hunk::from_cb(4, 0, 9, 0), &[]),
            [Hunk::from_cb(4, 0, 9, 0)]
        );
    }

    /// One run of one decision is one hunk, and the counts are the run's
    /// length in whichever documents it consumed from.
    #[test]
    fn one_run_is_one_hunk() {
        assert_eq!(
            walk(Hunk::from_cb(0, 3, 0, 3), &[BOTH; 3]),
            [Hunk::from_cb(0, 3, 0, 3)]
        );
        assert_eq!(
            walk(Hunk::from_cb(0, 2, 0, 0), &[COMPARED_BUFFER0; 2]),
            [Hunk::from_cb(0, 2, 0, 0)]
        );
        assert_eq!(
            walk(Hunk::from_cb(0, 0, 0, 2), &[COMPARED_BUFFER1; 2]),
            [Hunk::from_cb(0, 0, 0, 2)]
        );
    }

    /// A change of decision closes a hunk and opens the next one where the
    /// closed one left the two line numbers -- which advance only in the
    /// document the decision consumed from.
    #[test]
    fn a_changed_decision_closes_the_hunk_at_the_lines_consumed() {
        let hunk = Hunk::from_cb(10, 2, 20, 2);
        let deletion_then_addition = [COMPARED_BUFFER0, COMPARED_BUFFER0, 2, 2];
        assert_eq!(
            walk(hunk, &deletion_then_addition),
            [Hunk::from_cb(10, 2, 20, 0), Hunk::from_cb(12, 0, 20, 2),]
        );
    }

    /// Three runs, each one step long: every hunk carries exactly the one
    /// line its decision consumed, and the starts chain through.
    #[test]
    fn every_alternation_is_its_own_hunk() {
        let alternating = [COMPARED_BUFFER0, COMPARED_BUFFER1, COMPARED_BUFFER0];
        assert_eq!(
            walk(Hunk::from_cb(0, 2, 0, 1), &alternating),
            [
                Hunk::from_cb(0, 1, 0, 0),
                Hunk::from_cb(1, 0, 0, 1),
                Hunk::from_cb(1, 1, 1, 0),
            ]
        );
    }

    /// The finer hunks cover the outer one exactly: the counts add up to
    /// what `xdl_diff` reported, and nothing overlaps.
    #[test]
    fn the_finer_hunks_tile_the_original() {
        let hunk = Hunk::from_cb(7, 3, 11, 3);
        let decisions = [BOTH, COMPARED_BUFFER0, COMPARED_BUFFER1, BOTH];
        let hunks = walk(hunk, &decisions);
        assert_eq!(hunks.iter().map(|h| h.count_a).sum::<c_int>(), hunk.count_a);
        assert_eq!(hunks.iter().map(|h| h.count_b).sum::<c_int>(), hunk.count_b);
        let (mut lnuma, mut lnumb) = (hunk.start_a, hunk.start_b);
        for finer in hunks {
            assert_eq!((finer.start_a, finer.start_b), (lnuma, lnumb));
            lnuma += finer.count_a;
            lnumb += finer.count_b;
        }
    }
}
