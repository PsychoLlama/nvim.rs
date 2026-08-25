//! `:syntime` — per-pattern timing.
//!
//! With timing on, every `syn_regexec` accumulates into the pattern's own
//! `syn_time_T`; [`syntime_report`] sorts the patterns by total time and prints
//! the table. Used to find the pattern that makes a syntax file slow.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::*;

/// `:syntime {on,off,clear,report}`.
pub(crate) unsafe fn ex_syntime(eap: *mut exarg_T) {
    unsafe {
        let arg = CStr::from_ptr((*eap).arg);
        match arg.to_bytes() {
            b"on" => syn_time_on.set(true),
            b"off" => syn_time_on.set(false),
            b"clear" => syntime_clear(),
            b"report" => syntime_report(),
            _ => {
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
            }
        }
    }
}

/// Forget everything one pattern's timer accumulated.
pub(crate) unsafe fn syn_clear_time(st: &mut syn_time_T) {
    st.total = profile_zero();
    st.slowest = profile_zero();
    st.count = 0;
    st.match_0 = 0;
}

/// `:syntime clear` — forget the timings of every pattern in this window.
unsafe fn syntime_clear() {
    unsafe {
        if !syntax_present(curwin.get()) {
            msg(gettext(MSG_NO_ITEMS.as_ptr()), 0);
            return;
        }
        for idx in 0..cur_pattern_count() {
            syn_clear_time(&mut (*cur_pattern(idx)).sp_time);
        }
    }
}

/// The arguments `:syntime` takes, for command-line completion.
pub(crate) fn get_syntime_arg(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    const ARGS: [&CStr; 4] = [c"on", c"off", c"clear", c"report"];
    match ARGS.get(idx as usize) {
        Some(s) => s.as_ptr().cast_mut(),
        None => ::core::ptr::null_mut(),
    }
}

/// One row of the `:syntime report` table: a pattern's accumulated timings,
/// copied out of its `syn_time_T` so the table can be sorted.
#[derive(Copy, Clone)]
struct TimeEntry {
    total: proftime_T,
    count: c_int,
    matches: c_int,
    slowest: proftime_T,
    average: proftime_T,
    id: c_int,
    pattern: *mut c_char,
}

/// Order two rows by total time, for [`qsort`].
///
/// Still `qsort` and not `sort_by`: two patterns can accumulate exactly the
/// same total, and which of them the sort leaves first is then unprovable for
/// any other algorithm.
unsafe extern "C" fn syn_compare_syntime(v1: *const c_void, v2: *const c_void) -> c_int {
    unsafe {
        profile_cmp(
            (*(v1 as *const TimeEntry)).total,
            (*(v2 as *const TimeEntry)).total,
        )
    }
}

/// `:syntime report` — the timing table, slowest pattern last.
unsafe fn syntime_report() {
    unsafe {
        if !syntax_present(curwin.get()) {
            msg(gettext(MSG_NO_ITEMS.as_ptr()), 0);
            return;
        }

        let mut entries: Vec<TimeEntry> = Vec::new();
        let mut total_total = profile_zero();
        let mut total_count: c_int = 0;
        for idx in 0..cur_pattern_count() {
            let spp = cur_pattern(idx);
            let time = (*spp).sp_time;
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
                id: (*spp).sp_syn.id as c_int,
                pattern: (*spp).sp_pattern,
            });
        }

        // Skip the sort when there is nothing to sort: `qsort` may not be
        // handed a NULL pointer, which an empty `Vec` would be.
        if entries.len() > 1 {
            qsort(
                entries.as_mut_ptr() as *mut c_void,
                entries.len(),
                ::core::mem::size_of::<TimeEntry>(),
                Some(syn_compare_syntime),
            );
        }

        msg_puts_title(gettext(
            c"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN"
                .as_ptr(),
        ));
        msg_puts(c"\n".as_ptr());
        for entry in &entries {
            if got_int.get() {
                break;
            }
            report_row(entry);
        }
        if !got_int.get() {
            msg_puts(c"\n".as_ptr());
            msg_puts(profile_msg(total_total).as_ptr());
            msg_advance(13);
            msg_outnum(total_count);
            msg_puts(c"\n".as_ptr());
        }
    }
}

/// Print one row of the report, each field in its own fixed column.
///
/// `msg_advance` pads to a column, so a value wider than its field simply
/// pushes the rest of the row right; the trailing space after each value is
/// what keeps two of them from running together when that happens.
unsafe fn report_row(entry: &TimeEntry) {
    unsafe {
        msg_puts(profile_msg(entry.total).as_ptr());
        msg_puts(c" ".as_ptr());
        msg_advance(13);
        msg_outnum(entry.count);
        msg_puts(c" ".as_ptr());
        msg_advance(20);
        msg_outnum(entry.matches);
        msg_puts(c" ".as_ptr());
        msg_advance(26);
        msg_puts(profile_msg(entry.slowest).as_ptr());
        msg_puts(c" ".as_ptr());
        msg_advance(38);
        msg_puts(profile_msg(entry.average).as_ptr());
        msg_puts(c" ".as_ptr());
        msg_advance(50);
        msg_outtrans(highlight_group_name(entry.id - 1), 0, false);
        msg_puts(c" ".as_ptr());
        msg_advance(69);

        // The pattern gets whatever is left of the line; under 80 columns it
        // will wrap anyway, so a fixed 20 is as good as any.
        let room = if Columns.get() < 80 {
            20
        } else {
            Columns.get() - 70
        };
        let len = room.min(strlen(entry.pattern) as c_int);
        msg_outtrans_len(entry.pattern, len, 0, false);
        msg_puts(c"\n".as_ptr());
    }
}
