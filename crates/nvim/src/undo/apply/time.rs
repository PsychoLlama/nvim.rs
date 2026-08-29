//! Moving anywhere in the undo tree: `:earlier`, `:later`, `:undo N` and the
//! `g-`/`g+` shorthands.
//!
//! Where `u` and CTRL-R take one step along the branch the buffer is on, this
//! is a search: the state asked for can be anywhere in the tree, named by a
//! sequence number, by a timestamp or by a count of file writes, and it may
//! not exist at all — in which case the nearest state that does is taken
//! instead. The search walks the whole tree once, stamping the headers it may
//! still want with one mark and the dead ends with another, and then follows
//! that trail up and back down.

#![deny(unsafe_op_in_unsafe_fn)]

use super::super::store::{Header, Marks, header_chain};
use super::super::*;
use super::{u_undo_end, u_undoredo};
use crate::memline::MlFlags;
use crate::semsg_c;
use crate::winlayer::Buf;

// ---------------------------------------------------------------------------
// Anywhere in the tree: `:earlier`, `:later`, `:undo N`, `g-`, `g+`

/// What a move through the tree is aiming at, and in what unit.
///
/// The two flags are not one enum because either may be *dropped* during the
/// arithmetic: a target that falls off the end of the timestamp or
/// file-write scale is met by a plain sequence number instead.
#[derive(Clone, Copy)]
struct Aim {
    /// The value to reach, in the unit the flags select.
    target: c_int,
    /// Measure in seconds — `:earlier 10s`.
    sec: bool,
    /// Measure in file writes — `:earlier 1f`.
    file: bool,
}

impl Aim {
    /// The header's value in the unit this aim measures in.
    fn value_of(self, uh: &u_header_T) -> c_int {
        if self.sec {
            uh.uh_time as c_int
        } else if self.file {
            uh.uh_save_nr
        } else {
            uh.uh_seq
        }
    }
}

/// The nearest approach to the target the search has seen.
#[derive(Clone, Copy)]
struct Closest {
    /// Its value, in the aim's unit.
    val: c_int,
    /// The sequence number of the header that had it.
    seq: c_int,
    /// The out-of-reach value `val` started at. While `val` still holds it
    /// nothing has been seen at all, so any header is an improvement.
    start: c_int,
}

impl Closest {
    /// Remembers a header if it approaches the target more nearly than what
    /// we have.
    ///
    /// A header only counts when it lies in the direction of travel, which
    /// `seq_cur` measures; and when two carry the same timestamp, the one
    /// furthest along that direction wins.
    fn consider(&mut self, aim: Aim, val: c_int, seq: c_int, step: c_int, seq_cur: c_int) {
        // A header that was never written has no place on the file-write
        // scale.
        if aim.file && val == 0 {
            return;
        }
        let in_direction = if step < 0 {
            seq <= seq_cur
        } else {
            seq > seq_cur
        };
        if !in_direction {
            return;
        }
        let better = if aim.sec && val == self.val {
            if step < 0 {
                seq < self.seq
            } else {
                seq > self.seq
            }
        } else {
            self.val == self.start || approaches(val, aim.target, self.val)
        };
        if better {
            self.val = val;
            self.seq = seq;
        }
    }
}

/// Whether `val` is at least as near `target` as `closest` is —
/// `|val - target| <= |closest - target|`, spelled in differences that are
/// known non-negative rather than through `abs`, as the C spells it.
fn approaches(val: c_int, target: c_int, closest: c_int) -> bool {
    let from_val = if val > target {
        val - target
    } else {
        target - val
    };
    let from_closest = if closest > target {
        closest - target
    } else {
        target - closest
    };
    from_val <= from_closest
}

/// The buffer counters a move through the tree navigates by.
#[derive(Clone, Copy)]
struct UndoCounters {
    time_cur: time_t,
    seq_cur: c_int,
    seq_last: c_int,
    save_nr_cur: c_int,
    save_nr_last: c_int,
}

/// Where a move through the tree decided to go.
struct UndoDest {
    /// The sequence number to stop at. Zero is the state before any change.
    target: c_int,
    /// The stamps the search left along the way there.
    marks: Marks,
    /// Stop *above* the target header rather than on it. Set when a
    /// backwards move fell back to the nearest match.
    above: bool,
    /// The search reached a header carrying `target`.
    found: bool,
}

/// Moves through the undo tree by time, by file write, or by sequence
/// number: `:earlier`/`:later` and their `g-`/`g+` shorthands, and `:undo N`.
///
/// A negative `step` goes back in time. `sec` measures it in seconds, `file`
/// in file writes, and `absolute` makes it the sequence number to jump to —
/// `sec` is false then.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn undo_time(step: c_int, sec: bool, file: bool, absolute: bool) {
    // SAFETY: nothing here holds a borrow of editor state.
    if unsafe { text_locked() } {
        // SAFETY: as above.
        unsafe { text_locked_msg() };
        return;
    }
    // The change we are navigating past has to be synced first.
    // SAFETY: a live current buffer, by the contract above.
    let mut buf = unsafe { Buf::current() };
    if !buf.b_u_synced {
        // SAFETY: as above.
        u_sync(true);
        // SAFETY: as above; `u_sync` may have moved the tree under us.
        buf = unsafe { Buf::current() };
    }
    u_newcount.set(0);
    u_oldcount.set(if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
        -1
    } else {
        0
    });

    let (aim, closest_start) = undo_aim(buf, step, sec, file, absolute);
    let Some(dest) = undo_search(buf, step, absolute, aim, closest_start) else {
        return;
    };
    let mut did_undo = true;
    if dest.found || dest.target == 0 {
        // SAFETY: a live current buffer and window.
        unsafe { undo_up_to(&dest) };
        if dest.target > 0 {
            // SAFETY: as above.
            did_undo = unsafe { redo_down_to(&dest) };
        }
    }
    // SAFETY: as above.
    unsafe { u_undo_end(did_undo, absolute, false) };
}

/// Reads the counters off the buffer and turns `step` into an aim.
fn undo_aim(buf: Buf, step: c_int, sec: bool, file: bool, absolute: bool) -> (Aim, c_int) {
    let counters = UndoCounters {
        time_cur: buf.b_u_time_cur,
        seq_cur: buf.b_u_seq_cur,
        seq_last: buf.b_u_seq_last,
        save_nr_cur: buf.b_u_save_nr_cur,
        save_nr_last: buf.b_u_save_nr_last,
    };
    // ":earlier 1f" needs to know whether the change just above the current
    // one is itself a file write.
    let above = match buf.header(buf.b_u_curhead) {
        Some(curhead) => buf.header(curhead.uh_next),
        None => buf.header(buf.b_u_newhead),
    };
    aim_for(
        counters,
        step,
        sec,
        file,
        absolute,
        above.is_some_and(|uh| uh.uh_save_nr != 0),
    )
}

/// Turns a step into the value the tree search should aim at, and the
/// out-of-reach value the "nearest so far" starts from.
///
/// `saved_above` answers "is the change just above the current one a file
/// write". It only matters for `:earlier Nf`, where the changes made since
/// the last write count as one step of their own.
fn aim_for(
    counters: UndoCounters,
    step: c_int,
    sec: bool,
    file: bool,
    absolute: bool,
    saved_above: bool,
) -> (Aim, c_int) {
    let mut aim = Aim {
        target: 0,
        sec,
        file,
    };
    if absolute {
        // ":undo N" names the sequence number outright.
        aim.target = step;
        return (aim, -1);
    }
    aim.target = if sec {
        (counters.time_cur as c_int).wrapping_add(step)
    } else if file && step < 0 {
        // Back to a previous write. Changes made since the last one count as
        // a file write of their own, so that ":earlier 1f" undoes them.
        let target = counters
            .save_nr_cur
            .wrapping_add(step)
            .wrapping_add(c_int::from(!saved_above));
        if target <= 0 {
            // Before the first write is before the oldest change, and only a
            // sequence number can name that.
            aim.file = false;
        }
        target
    } else if file {
        // Forward to a newer write.
        let target = counters.save_nr_cur.wrapping_add(step);
        if target > counters.save_nr_last {
            // After the last write is after the newest change; likewise.
            aim.file = false;
            counters.seq_last + 1
        } else {
            target
        }
    } else {
        // `step` is a user count, so this is `int` arithmetic that wraps, as
        // it does in the C. `:later 2147483647` runs past `INT_MAX` and
        // lands below zero, where the clamp below has nothing to say; what
        // catches it is the search's own fallback to the nearest header.
        counters.seq_cur.wrapping_add(step)
    };
    if step < 0 {
        aim.target = aim.target.max(0);
        return (aim, -1);
    }
    // One past anything the tree can hold, so that the first header seen in
    // the right direction is an improvement on it.
    let closest = if aim.sec {
        os_time().wrapping_add(1) as c_int
    } else if aim.file {
        counters.save_nr_last + 2
    } else {
        counters.seq_last + 2
    };
    aim.target = aim.target.min(closest - 1);
    (aim, closest)
}

/// Searches the whole tree for the header the aim names, marking the path.
///
/// Up to two rounds: the first looks for the target itself and remembers the
/// header nearest it, the second — when the target is nowhere in the tree —
/// looks for that nearest header. The second round always goes by sequence
/// number, because several headers can share a timestamp.
///
/// Answers `None` once it has reported that there is nowhere to go.
fn undo_search(
    buf: Buf,
    step: c_int,
    absolute: bool,
    mut aim: Aim,
    closest_start: c_int,
) -> Option<UndoDest> {
    if aim.target == 0 {
        // Back to the origin. Nothing to search for: no header carries
        // sequence number zero, and the walk up stops when it runs out.
        return Some(UndoDest {
            target: 0,
            marks: Marks {
                mark: lastmark.get(),
                nomark: 0,
            },
            above: false,
            found: false,
        });
    }
    let mut closest = Closest {
        val: closest_start,
        seq: buf.b_u_seq_cur,
        start: closest_start,
    };
    let mut above = false;
    let mut marks = Marks { mark: 0, nomark: 0 };
    for round in 1..=2 {
        // The desired state can be anywhere in the tree, so the walk goes all
        // over it, stamping as it goes.
        marks = Marks::next();
        let start = if buf.b_u_curhead.is_some() {
            buf.b_u_curhead
        } else {
            buf.b_u_newhead // at a leaf of the tree
        };
        if walk_to_target(buf, start, step, round == 1, &mut aim, &mut closest, marks) {
            return Some(UndoDest {
                target: aim.target,
                marks,
                above,
                found: true,
            });
        }
        if absolute {
            // SAFETY: a NUL-terminated literal and an integer.
            unsafe {
                let fmt = gettext(c"E830: Undo number %ld not found");
                semsg_c!(fmt, step as int64_t);
            }
            return None;
        }
        if closest.val == closest.start {
            let text = if step < 0 {
                c"Already at oldest change"
            } else {
                c"Already at newest change"
            };
            msg(gettext(text), 0);
            return None;
        }
        // Aim at the nearest header instead, by sequence number; and when we
        // were going backwards, stop above it rather than on it.
        aim.target = closest.seq;
        aim.sec = false;
        aim.file = false;
        above = step < 0;
    }
    // Round two aims at a sequence number a header in the tree carried, so it
    // finds it. The C is defensive here all the same, and so is this.
    Some(UndoDest {
        target: aim.target,
        marks,
        above,
        found: false,
    })
}

/// One depth-first pass over the whole tree from `start`.
///
/// [`TreeWalk`](super::super::store::TreeWalk) stamps every header it
/// reaches with `marks.mark` and every one it has exhausted with
/// `marks.nomark`, which is what keeps a tree whose links run in four
/// directions from being walked twice. `stopping_above` un-stamps the header
/// the buffer already sits on, because that change is not one the move goes
/// through.
///
/// Answers whether the target was reached; when it was, `aim.target` has been
/// rewritten to the target's *sequence number*, which is a different number
/// whenever the aim was a timestamp or a file write.
fn walk_to_target(
    buf: Buf,
    start: UndoLink,
    step: c_int,
    scoring: bool,
    aim: &mut Aim,
    closest: &mut Closest,
    marks: Marks,
) -> bool {
    for visit in buf.tree_walk(start, marks).stopping_above(buf.b_u_curhead) {
        let uhp = visit.header;
        let val = aim.value_of(&uhp);
        if scoring {
            closest.consider(*aim, val, uhp.uh_seq, step, buf.b_u_seq_cur);
        }
        // Stop on a match — but a timestamp match keeps looking, because the
        // best sequence number carrying that timestamp may be further on.
        if aim.target == val && !aim.sec {
            aim.target = uhp.uh_seq;
            return true;
        }
    }
    false
}

/// Undoes up the tree until the destination is reached, or CTRL-C.
///
/// # Safety
///
/// A live current buffer and window.
unsafe fn undo_up_to(dest: &UndoDest) {
    while !got_int.get() {
        // The change warning first, for the reason `u_doit` gives.
        // SAFETY: a live current buffer, by the contract above.
        unsafe { change_warning(Buf::current(), 0) };
        // SAFETY: as above — the warning may have reloaded the buffer.
        let mut buf = unsafe { Buf::current() };
        let above = match buf.header(buf.b_u_curhead) {
            Some(curhead) => buf.header(curhead.uh_next),
            None => buf.header(buf.b_u_newhead),
        };
        let Some(mut uhp) = above else { break };
        if (dest.target > 0 && uhp.uh_walk != dest.marks.mark)
            || (uhp.uh_seq == dest.target && !dest.above)
        {
            break;
        }
        buf.b_u_curhead = uhp.link();
        // SAFETY: a live current buffer and window, and `b_u_curhead` names
        // the header we just resolved.
        unsafe { u_undoredo(true, true) };
        if dest.target > 0 {
            uhp.uh_walk = dest.marks.nomark; // don't come back down here
        }
    }
}

/// Redoes down the tree to the destination, taking the marked branch at every
/// fork. Answers whether nothing was redone, which is what the message at the
/// end reports as "before" rather than "after".
///
/// # Safety
///
/// A live current buffer and window.
unsafe fn redo_down_to(dest: &UndoDest) -> bool {
    let mut did_undo = true;
    while !got_int.get() {
        // SAFETY: a live current buffer, by the contract above.
        unsafe { change_warning(Buf::current(), 0) };
        // SAFETY: as above — the warning may have reloaded the buffer.
        let mut buf = unsafe { Buf::current() };
        let Some(fork) = buf.header(buf.b_u_curhead) else {
            break;
        };
        // SAFETY: a live buffer, and nothing here frees a header.
        let mut uhp = unsafe { take_marked_branch(buf, fork, dest.marks.mark) };

        buf.b_u_curhead = uhp.link();
        if uhp.uh_walk != dest.marks.mark {
            break; // must have reached the target
        }
        if uhp.uh_seq == dest.target && dest.above {
            // Going backwards in time, and this is not the exact header we
            // wanted: stop above it.
            buf.b_u_seq_cur = dest.target - 1;
            break;
        }
        // SAFETY: a live current buffer and window, and `b_u_curhead` names
        // the header we just resolved.
        unsafe { u_undoredo(false, true) };
        // Advance below the header just used; nothing below it means this
        // leaf is the new head.
        if uhp.uh_prev.is_none() {
            buf.b_u_newhead = uhp.link();
        }
        buf.b_u_curhead = uhp.uh_prev;
        did_undo = false;
        if uhp.uh_seq == dest.target {
            break; // found it
        }
        if !buf
            .header(uhp.uh_prev)
            .is_some_and(|prev| prev.uh_walk == dest.marks.mark)
        {
            // More to redo, but the marked path has come apart.
            // SAFETY: a NUL-terminated literal.
            unsafe { internal_error(c"undo_time()".as_ptr()) };
            break;
        }
    }
    did_undo
}

/// Of the alternate branches at `fork`, picks the one the search marked and
/// moves it to the front of the list, so that a later `u` and CTRL-R take it
/// too. Answers the branch to take, which is `fork` itself when the marked
/// run is only that.
///
/// # Safety
///
/// `buf` is the buffer `fork` belongs to, and nothing frees a header while
/// this runs.
unsafe fn take_marked_branch(mut buf: Buf, fork: Header, mark: c_int) -> Header {
    // The search marks a run of consecutive alternates; the far end of that
    // run along `uh_alt_next` is the branch it wants.
    // SAFETY: a live buffer that owns these headers, by the contract above.
    let head = unsafe { furthest_marked(buf, fork, mark, |uh| uh.uh_alt_prev) };
    // SAFETY: as above.
    let mut last = unsafe { furthest_marked(buf, head, mark, |uh| uh.uh_alt_next) };
    if last == head {
        return head;
    }
    // The whole list of alternates may start further back than the marked
    // run does, and that head is where the branch has to end up.
    // SAFETY: as above.
    let mut first = unsafe { header_chain(buf, head.link(), |uh| uh.uh_alt_prev) }
        .last()
        .unwrap_or(head);
    // Unlink it from where it sits... (`last != head` means it was reached
    // along `uh_alt_next`, so it has a predecessor; the C dereferences that
    // without asking.)
    if let Some(mut after) = buf.header(last.uh_alt_next) {
        after.uh_alt_prev = last.uh_alt_prev;
    }
    if let Some(mut before) = buf.header(last.uh_alt_prev) {
        before.uh_alt_next = last.uh_alt_next;
    }
    // ...and splice it in at the front.
    last.uh_alt_prev = UndoLink::NONE;
    last.uh_alt_next = first.link();
    first.uh_alt_prev = last.link();
    if buf.b_u_oldhead == first.link() {
        buf.b_u_oldhead = last.link();
    }
    if let Some(mut next) = buf.header(last.uh_next) {
        next.uh_prev = last.link();
    }
    last
}

/// Follows `step` from `uhp` for as long as the next header carries `mark`,
/// and answers the last header of that run.
///
/// # Safety
///
/// `buf` is the buffer `uhp` belongs to, and nothing frees a header while
/// this runs.
unsafe fn furthest_marked(
    buf: Buf,
    uhp: Header,
    mark: c_int,
    step: fn(&u_header_T) -> UndoLink,
) -> Header {
    // The chain always yields its start; every further hop has to be marked.
    // SAFETY: the buffer owns these headers, by the contract above.
    unsafe { header_chain(buf, uhp.link(), step) }
        .skip(1)
        .take_while(|uh| uh.uh_walk == mark)
        .last()
        .unwrap_or(uhp)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A buffer that has never been written, at sequence number 5 of 8.
    fn counters() -> UndoCounters {
        UndoCounters {
            time_cur: 1_000,
            seq_cur: 5,
            seq_last: 8,
            save_nr_cur: 0,
            save_nr_last: 0,
        }
    }

    #[test]
    fn a_plain_step_moves_by_sequence_number() {
        let (aim, closest) = aim_for(counters(), -2, false, false, false, false);
        assert_eq!(aim.target, 3);
        assert!(!aim.sec && !aim.file);
        assert_eq!(closest, -1);

        let (aim, closest) = aim_for(counters(), 2, false, false, false, false);
        assert_eq!(aim.target, 7);
        // One past the newest change, so the first header seen wins.
        assert_eq!(closest, 10);
    }

    #[test]
    fn a_forward_step_is_clamped_to_just_past_the_newest_change() {
        let (aim, closest) = aim_for(counters(), 1_000, false, false, false, false);
        assert_eq!(aim.target, closest - 1);
        assert_eq!(aim.target, 9);
    }

    #[test]
    fn a_forward_step_past_int_max_wraps_and_is_not_clamped() {
        // The C's `int` arithmetic, kept: the sum runs past `INT_MAX` and
        // lands below zero, where the clamp has nothing to say. Nothing in
        // the tree carries that number, so the search's second round falls
        // back to the header nearest it, which is the newest one.
        let (aim, _) = aim_for(counters(), c_int::MAX, false, false, false, false);
        assert_eq!(aim.target, counters().seq_cur.wrapping_add(c_int::MAX));
        assert!(aim.target < 0);
    }

    #[test]
    fn a_backward_step_is_clamped_to_the_origin() {
        let (aim, _) = aim_for(counters(), -99, false, false, false, false);
        assert_eq!(aim.target, 0);
    }

    #[test]
    fn an_absolute_step_is_the_sequence_number_itself() {
        let (aim, closest) = aim_for(counters(), 12345, false, false, true, false);
        assert_eq!(aim.target, 12345);
        assert_eq!(closest, -1);
    }

    #[test]
    fn unsaved_changes_count_as_one_file_write_back() {
        let counters = UndoCounters {
            save_nr_cur: 2,
            save_nr_last: 3,
            ..counters()
        };
        // The change above the current one is not a write, so ":earlier 1f"
        // undoes everything since the last one rather than a whole write.
        let (aim, _) = aim_for(counters, -1, false, true, false, false);
        assert!(aim.file);
        assert_eq!(aim.target, 2);
        // With the write immediately above, the same command goes one back.
        let (aim, _) = aim_for(counters, -1, false, true, false, true);
        assert_eq!(aim.target, 1);
    }

    #[test]
    fn stepping_off_either_end_of_the_write_scale_falls_back_to_sequences() {
        let counters = UndoCounters {
            save_nr_cur: 1,
            save_nr_last: 3,
            ..counters()
        };
        // Before the first write is before the oldest change.
        let (aim, _) = aim_for(counters, -2, false, true, false, true);
        assert!(!aim.file);
        assert_eq!(aim.target, 0);
        // And after the last write is after the newest change.
        let (aim, closest) = aim_for(counters, 9, false, true, false, false);
        assert!(!aim.file);
        assert_eq!(aim.target, counters.seq_last + 1);
        assert_eq!(closest, counters.seq_last + 2);
    }

    #[test]
    fn nearness_is_the_distance_to_the_target_either_side_of_it() {
        assert!(approaches(9, 10, 12));
        assert!(approaches(11, 10, 12));
        assert!(approaches(10, 10, 10));
        // A tie counts as an approach: "at least as near".
        assert!(approaches(11, 10, 9));
        // Further away on either side does not.
        assert!(!approaches(13, 10, 12));
        assert!(!approaches(7, 10, 12));
    }

    #[test]
    fn only_headers_in_the_direction_of_travel_are_considered() {
        let aim = Aim {
            target: 2,
            sec: false,
            file: false,
        };
        let mut closest = Closest {
            val: -1,
            seq: 5,
            start: -1,
        };
        // Going back from sequence 5: a later header is the wrong way.
        closest.consider(aim, 7, 7, -1, 5);
        assert_eq!(closest.val, -1);
        closest.consider(aim, 3, 3, -1, 5);
        assert_eq!((closest.val, closest.seq), (3, 3));
        // Nearer wins, further does not.
        closest.consider(aim, 1, 1, -1, 5);
        assert_eq!((closest.val, closest.seq), (1, 1));
        closest.consider(aim, 4, 4, -1, 5);
        assert_eq!((closest.val, closest.seq), (1, 1));
    }

    #[test]
    fn a_header_that_was_never_written_is_off_the_file_write_scale() {
        let aim = Aim {
            target: 2,
            sec: false,
            file: true,
        };
        let mut closest = Closest {
            val: -1,
            seq: 5,
            start: -1,
        };
        closest.consider(aim, 0, 3, -1, 5);
        assert_eq!(closest.val, -1);
    }

    #[test]
    fn headers_sharing_a_timestamp_are_settled_by_sequence_number() {
        let aim = Aim {
            target: 100,
            sec: true,
            file: false,
        };
        let mut closest = Closest {
            val: -1,
            seq: 5,
            start: -1,
        };
        closest.consider(aim, 100, 6, 1, 5);
        assert_eq!((closest.val, closest.seq), (100, 6));
        // Going forwards, the highest sequence number with that timestamp.
        closest.consider(aim, 100, 8, 1, 5);
        assert_eq!((closest.val, closest.seq), (100, 8));
        closest.consider(aim, 100, 7, 1, 5);
        assert_eq!(closest.seq, 8);
    }
}
