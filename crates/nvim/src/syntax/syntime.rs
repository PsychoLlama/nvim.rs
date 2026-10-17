//! `:syntime` — per-pattern timing.
//!
//! With timing on, every `syn_regexec` accumulates into the pattern's own
//! `SynTime`; [`syntime_report`] sorts the patterns by total time and prints
//! the table. Used to find the pattern that makes a syntax file slow.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::Candidate;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use super::*;

/// `:syntime {on,off,clear,report}`.
pub(crate) fn ex_syntime(excmd: &mut ExArg) {
    match excmd.line.arg() {
        b"on" => syn_time_on.set(true),
        b"off" => syn_time_on.set(false),
        b"clear" => syntime_clear(),
        b"report" => syntime_report(),
        _ => {
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E475: Invalid argument: {arg}");
        }
    }
}

/// Forget everything one pattern's timer accumulated.
pub(crate) fn syn_clear_time(st: &mut SynTime) {
    st.total = profile_zero();
    st.slowest = profile_zero();
    st.count = 0;
    st.match_0 = 0;
}

/// `:syntime clear` — forget the timings of every pattern in this window.
fn syntime_clear() {
    if !syntax_present(Win::current()) {
        msg(gettext(MSG_NO_ITEMS), 0);
        return;
    }
    for idx in 0..cur_pattern_count() {
        syn_clear_time(&mut cur_syn_block().pattern_mut(idx).sp_time);
    }
}

/// The arguments `:syntime` takes, for command-line completion.
pub(crate) fn get_syntime_arg(_expand: &Expand, idx: usize) -> Option<Candidate> {
    const ARGS: [&CStr; 4] = [c"on", c"off", c"clear", c"report"];
    ARGS.get(idx).map(|&arg| Candidate::Borrowed(arg))
}

/// One row of the `:syntime report` table: a pattern's accumulated timings,
/// copied out of its `SynTime` so the table can be sorted.
struct TimeEntry {
    total: ProfTime,
    count: c_int,
    matches: c_int,
    slowest: ProfTime,
    average: ProfTime,
    id: c_int,
    /// The pattern text, copied: printing the table can run anything a
    /// message can, so nothing is borrowed from the pattern array across it.
    pattern: Vec<u8>,
}

/// `:syntime report` — the timing table, slowest pattern last.
fn syntime_report() {
    if !syntax_present(Win::current()) {
        msg(gettext(MSG_NO_ITEMS), 0);
        return;
    }

    let mut entries: Vec<TimeEntry> = Vec::new();
    let mut total_total = profile_zero();
    let mut total_count: c_int = 0;
    for idx in 0..cur_pattern_count() {
        let block = cur_syn_block();
        let spp = block.pattern(idx);
        let time = spp.sp_time;
        if time.count <= 0 {
            continue;
        }
        total_total = profile_add(total_total, time.total);
        total_count += time.count;
        entries.push(TimeEntry {
            total: time.total,
            count: time.count,
            matches: time.match_0,
            slowest: time.slowest,
            average: profile_divide(time.total, time.count),
            id: spp.sp_syn.id as c_int,
            pattern: spp
                .sp_pattern
                .as_deref()
                .map_or_else(Vec::new, |p| p.to_bytes().to_vec()),
        });
    }

    // Ascending by `profile_cmp`, which is negative when its *first*
    // argument is the larger time. Stable, so patterns with exactly the same
    // total keep their definition order; upstream's `qsort` left that order
    // to the C library.
    entries.sort_by(|a, b| profile_cmp(a.total, b.total).cmp(&0));

    msg_title(gettext(
        c"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN",
    ));
    msg_str(c"\n");
    for entry in &entries {
        if got_int.get() {
            break;
        }
        report_row(entry);
    }
    if !got_int.get() {
        msg_str(c"\n");
        msg_str(cstr::in_chars(&profile_msg(total_total)));
        msg_advance(13);
        msg_outnum(total_count);
        msg_str(c"\n");
    }
}

/// Print one row of the report, each field in its own fixed column.
///
/// `msg_advance` pads to a column, so a value wider than its field simply
/// pushes the rest of the row right; the trailing space after each value is
/// what keeps two of them from running together when that happens.
fn report_row(entry: &TimeEntry) {
    msg_str(cstr::in_chars(&profile_msg(entry.total)));
    msg_str(c" ");
    msg_advance(13);
    msg_outnum(entry.count);
    msg_str(c" ");
    msg_advance(20);
    msg_outnum(entry.matches);
    msg_str(c" ");
    msg_advance(26);
    msg_str(cstr::in_chars(&profile_msg(entry.slowest)));
    msg_str(c" ");
    msg_advance(38);
    msg_str(cstr::in_chars(&profile_msg(entry.average)));
    msg_str(c" ");
    msg_advance(50);
    msg_display(syn_id2name(entry.id), 0, false);
    msg_str(c" ");
    msg_advance(69);

    // The pattern gets whatever is left of the line; under 80 columns it
    // will wrap anyway, so a fixed 20 is as good as any.
    let room = if Columns.get() < 80 {
        20
    } else {
        Columns.get() - 70
    };
    let pattern = &entry.pattern;
    let room = usize::try_from(room).unwrap_or(0);
    msg_display_bytes(&pattern[..room.min(pattern.len())], 0, false);
    msg_str(c"\n");
}
