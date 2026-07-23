//! Port of the former `test/unit/profile_spec.lua`: the pure `proftime_T`
//! arithmetic. The stateful pieces (wait-time accounting, the `:profile`
//! report, `--startuptime`) are covered by the functional and old suites.

use c2rust_neovim::src::nvim::profile::{
    profile_add, profile_cmp, profile_divide, profile_end, profile_msg_str, profile_passed_limit,
    profile_self, profile_setlimit, profile_signed, profile_start, profile_sub, profile_zero,
};

/// A time strictly later than `t` (the clock has nanosecond resolution, but
/// two adjacent readings may still tie).
fn later_than(t: u64) -> u64 {
    loop {
        let now = profile_start();
        if now != t {
            return now;
        }
    }
}

#[test]
fn cmp_orders_subsequent_starts() {
    let s1 = profile_start();
    let s2 = later_than(s1);
    assert!(profile_cmp(s1, s2) > 0);
    assert!(profile_cmp(s2, s1) < 0);
}

#[test]
fn cmp_zero_is_equal() {
    assert_eq!(profile_cmp(profile_zero(), profile_zero()), 0);
}

#[test]
fn cmp_orders_divisions() {
    let start = profile_start();
    assert!(profile_cmp(start, profile_divide(start, 10)) <= 0);
}

#[test]
fn divide_performs_division() {
    // The routine performs floating-point division for better rounding, so
    // check a range rather than an exact value.
    let divisor = 10;
    let start = profile_start();
    let divided = profile_divide(start, divisor);
    let mut res = divided;
    for _ in 1..divisor {
        res = profile_add(res, divided);
    }
    let (min, max) = (
        profile_sub(start, divisor as u64),
        profile_add(start, divisor as u64),
    );
    assert!(profile_cmp(min, res) >= 0);
    assert!(profile_cmp(max, res) <= 0);
}

#[test]
fn divide_by_nonpositive_count_is_zero() {
    assert_eq!(profile_divide(1_000_000, 0), 0);
    assert_eq!(profile_divide(1_000_000, -3), 0);
}

#[test]
fn zero_is_zero() {
    assert_eq!(profile_zero(), 0);
}

#[test]
fn start_increases() {
    let mut last = profile_start();
    for _ in 0..100 {
        let curr = profile_start();
        assert!(curr >= last);
        last = curr;
    }
}

#[test]
fn end_elapsed_is_nonzero() {
    let start = profile_start();
    later_than(start);
    assert_ne!(profile_end(start), profile_zero());
}

#[test]
fn end_outer_elapsed_at_least_inner() {
    for _ in 0..100 {
        let start_outer = profile_start();
        let start_inner = profile_start();
        let elapsed_inner = profile_end(start_inner);
        let elapsed_outer = profile_end(start_outer);
        assert!(elapsed_outer >= elapsed_inner);
    }
}

#[test]
fn setlimit_zero_means_no_limit() {
    assert_eq!(profile_setlimit(0), profile_zero());
    assert_eq!(profile_setlimit(-5), profile_zero());
}

#[test]
fn setlimit_is_in_the_future() {
    let future = profile_setlimit(1000);
    let now = profile_start();
    assert!(profile_cmp(future, now) < 0);
}

#[test]
fn passed_limit_start_is_in_the_past() {
    let start = profile_start();
    later_than(start);
    assert!(profile_passed_limit(start));
}

#[test]
fn passed_limit_double_start_is_in_the_future() {
    let start = profile_start();
    let future = profile_add(start, start);
    assert!(!profile_passed_limit(future));
}

#[test]
fn msg_prints_zero_time() {
    assert_eq!(profile_msg_str(profile_zero()), "  0.000000");
}

#[test]
fn msg_prints_seconds_dot_fraction() {
    let start = profile_start();
    let elapsed = profile_end(start);
    let s = profile_msg_str(elapsed);
    let trimmed = s.trim();
    let (secs, frac) = trimmed.split_once('.').expect("no dot in profile msg");
    // If whole seconds passed between two adjacent calls, the profiling
    // functions are too slow and need to be fixed.
    assert_eq!(secs, "0");
    assert!(frac.starts_with('0'));
    assert_eq!(frac.len(), 6);
}

#[test]
fn msg_prints_negative_durations() {
    // A wrapped difference formats as a (signed) negative time, #10452.
    assert_eq!(profile_msg_str(profile_sub(0, 500_000_000)), " -0.500000");
}

#[test]
fn add_adds() {
    let start = profile_start();
    assert_eq!(profile_add(profile_zero(), start), start);
}

#[test]
fn sub_subtracts() {
    let start = profile_start();
    assert_eq!(profile_sub(start, profile_zero()), start);

    let start1 = profile_start();
    let start2 = later_than(start1);
    let start3 = later_than(start2);
    assert!(profile_cmp(profile_sub(start2, start1), profile_sub(start3, start1)) > 0);
    assert!(profile_cmp(profile_sub(start3, start1), profile_sub(start2, start1)) < 0);
}

#[test]
fn signed_recovers_wrapped_differences() {
    assert_eq!(profile_signed(5), 5);
    // The Vim-compat formula `-(UINT64_MAX - tm)` (#10452) is off by one
    // from true two's complement: 2 - 5 reads back as -2, not -3.
    assert_eq!(profile_signed(profile_sub(2, 5)), -2);
}

#[test]
fn self_time_accounts_children() {
    // total > children: self accumulates total minus children.
    assert_eq!(profile_self(10, 100, 30), 80);
    // total <= children (recursive calls): self is unchanged.
    assert_eq!(profile_self(10, 30, 30), 10);
    assert_eq!(profile_self(10, 20, 30), 10);
}
