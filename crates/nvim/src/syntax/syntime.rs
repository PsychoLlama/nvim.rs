//! `:syntime` — per-pattern timing.
//!
//! With timing on, every `syn_regexec` accumulates into the pattern's own
//! `syn_time_T`; [`syntime_report`] sorts the patterns by total time and prints
//! the table. Used to find the pattern that makes a syntax file slow.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::*;

/// `:syntime {on,off,clear,report}`.
pub(crate) unsafe fn ex_syntime(eap: *mut exarg_T) {
    let arg = unsafe { CStr::from_ptr((*eap).arg) };
    match arg.to_bytes() {
        b"on" => syn_time_on.set(true),
        b"off" => syn_time_on.set(false),
        b"clear" => unsafe { syntime_clear() },
        b"report" => unsafe { syntime_report() },
        _ => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str((*eap).arg) };
            semsg!("E475: Invalid argument: {arg}");
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
    if !unsafe { syntax_present(curwin.get()) } {
        msg(gettext(MSG_NO_ITEMS), 0);
        return;
    }
    for idx in 0..cur_pattern_count() {
        unsafe { syn_clear_time(&mut (*cur_pattern(idx).raw()).sp_time) };
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
    profile_cmp(unsafe { (*(v1 as *const TimeEntry)).total }, unsafe {
        (*(v2 as *const TimeEntry)).total
    })
}

/// `:syntime report` — the timing table, slowest pattern last.
unsafe fn syntime_report() {
    if !unsafe { syntax_present(curwin.get()) } {
        msg(gettext(MSG_NO_ITEMS), 0);
        return;
    }

    let mut entries: Vec<TimeEntry> = Vec::new();
    let mut total_total = profile_zero();
    let mut total_count: c_int = 0;
    for idx in 0..cur_pattern_count() {
        let spp = unsafe { cur_pattern(idx) };
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
            pattern: spp.sp_pattern,
        });
    }

    // Skip the sort when there is nothing to sort: `qsort` may not be
    // handed a NULL pointer, which an empty `Vec` would be.
    if entries.len() > 1 {
        unsafe {
            qsort(
                entries.as_mut_ptr() as *mut c_void,
                entries.len(),
                ::core::mem::size_of::<TimeEntry>(),
                Some(syn_compare_syntime),
            )
        };
    }

    unsafe {
        msg_puts_title(
            gettext(
                c"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN",
            )
            .as_ptr(),
        )
    };
    unsafe { msg_puts(c"\n".as_ptr()) };
    for entry in &entries {
        if got_int.get() {
            break;
        }
        unsafe { report_row(entry) };
    }
    if !got_int.get() {
        unsafe { msg_puts(c"\n".as_ptr()) };
        unsafe { msg_puts(profile_msg(total_total).as_ptr()) };
        unsafe { msg_advance(13) };
        unsafe { msg_outnum(total_count) };
        unsafe { msg_puts(c"\n".as_ptr()) };
    }
}

/// Print one row of the report, each field in its own fixed column.
///
/// `msg_advance` pads to a column, so a value wider than its field simply
/// pushes the rest of the row right; the trailing space after each value is
/// what keeps two of them from running together when that happens.
unsafe fn report_row(entry: &TimeEntry) {
    unsafe { msg_puts(profile_msg(entry.total).as_ptr()) };
    unsafe { msg_puts(c" ".as_ptr()) };
    unsafe { msg_advance(13) };
    unsafe { msg_outnum(entry.count) };
    unsafe { msg_puts(c" ".as_ptr()) };
    unsafe { msg_advance(20) };
    unsafe { msg_outnum(entry.matches) };
    unsafe { msg_puts(c" ".as_ptr()) };
    unsafe { msg_advance(26) };
    unsafe { msg_puts(profile_msg(entry.slowest).as_ptr()) };
    unsafe { msg_puts(c" ".as_ptr()) };
    unsafe { msg_advance(38) };
    unsafe { msg_puts(profile_msg(entry.average).as_ptr()) };
    unsafe { msg_puts(c" ".as_ptr()) };
    unsafe { msg_advance(50) };
    unsafe { msg_outtrans(highlight_group_name(entry.id - 1), 0, false) };
    unsafe { msg_puts(c" ".as_ptr()) };
    unsafe { msg_advance(69) };

    // The pattern gets whatever is left of the line; under 80 columns it
    // will wrap anyway, so a fixed 20 is as good as any.
    let room = if Columns.get() < 80 {
        20
    } else {
        Columns.get() - 70
    };
    let len = room.min(unsafe { cstr::bytes_at(entry.pattern) }.len() as c_int);
    unsafe { msg_outtrans_len(entry.pattern, len, 0, false) };
    unsafe { msg_puts(c"\n".as_ptr()) };
}
