//! The 'vartabstop' / 'varsofttabstop' arithmetic.
//!
//! A tabstop list is stored as a `colnr_T` array whose element 0 is the
//! number of stops that follow. A null pointer, or a list whose count is
//! zero, means "no list": the uniform 'tabstop' width applies instead. The
//! last stop repeats forever, which is what every `t > tabcount` branch here
//! is about.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::types::{OptInt, colnr_T};

// Nested so that ffigen, which flattens a file's top-level constants into one
// C namespace, does not publish a name this generic.
mod limit {
    /// The largest width a single stop may name.
    pub const TABSTOP_MAX: ::core::ffi::c_int = 9999;
}
pub use limit::TABSTOP_MAX;

/// A list of tabstops: `stops[0]` is the count, `stops[1..=count]` the
/// widths.
#[derive(Clone, Copy)]
pub struct TabStops<'a> {
    stops: &'a [colnr_T],
}

impl<'a> TabStops<'a> {
    /// Wrap a list, or answer `None` when it is absent or empty — the two
    /// cases every caller treats as "use the uniform width".
    ///
    /// `stops` must be at least `stops[0] + 1` long.
    pub fn new(stops: &'a [colnr_T]) -> Option<Self> {
        (!stops.is_empty() && stops[0] != 0).then_some(TabStops { stops })
    }

    /// How many stops the list names.
    pub fn count(self) -> c_int {
        self.stops[0]
    }

    /// The width of stop `t`, counting from one.
    fn width(self, t: c_int) -> colnr_T {
        self.stops[t as usize]
    }

    /// The first stop's width, which is what 'shiftwidth' falls back to.
    pub fn first(self) -> c_int {
        self.width(1)
    }

    /// The stop that `col` falls inside: its one-based index and the column
    /// it starts at. Answers `None` when `col` is past the last stop.
    fn containing(self, col: colnr_T) -> Option<(c_int, colnr_T)> {
        let mut tabcol: colnr_T = 0;
        for t in 1..=self.count() {
            tabcol += self.width(t);
            if tabcol > col {
                return Some((t, tabcol));
            }
        }
        None
    }

    /// The column where the repeating tail begins, and the width it repeats
    /// with — used once `col` is past the last named stop.
    fn tail(self) -> (colnr_T, colnr_T) {
        let total: colnr_T = self.stops[1..=self.count() as usize].iter().sum();
        (total, self.width(self.count()))
    }

    /// How many columns from `col` to the next tabstop.
    pub fn padding(self, col: colnr_T) -> c_int {
        match self.containing(col) {
            Some((_, tabcol)) => tabcol - col,
            None => {
                let (total, last) = self.tail();
                last - (col - total) % last
            }
        }
    }

    /// The width of the tabstop at `col`. With `left`, the width of the stop
    /// the cursor would move *back* over, which for the first stop is `col`
    /// itself.
    pub fn at(self, col: colnr_T, left: bool) -> c_int {
        match self.containing(col) {
            Some((1, _)) if left => col,
            Some((t, _)) => self.width(t - c_int::from(left)),
            None => self.width(self.count()),
        }
    }

    /// The column the tabstop containing `col` starts at.
    pub fn start(self, col: colnr_T) -> colnr_T {
        match self.containing(col) {
            Some((t, tabcol)) => tabcol - self.width(t),
            None => {
                let (total, last) = self.tail();
                // The repeating tail is offset by however far the named stops
                // overshoot a multiple of the last width.
                col - (col - total % last) % last
            }
        }
    }

    /// The tabs and trailing spaces that fill the columns `start_col` to
    /// `end_col`.
    pub fn from_to(self, start_col: colnr_T, end_col: colnr_T) -> (c_int, c_int) {
        let mut spaces = end_col - start_col;
        let padding = self.padding(start_col);
        if spaces < padding {
            return (0, spaces);
        }
        let mut tabs = 1;
        spaces -= padding;
        // Continue through the named stops the run covers, then repeat the
        // last one.
        let mut t = self.containing(start_col).map_or(self.count(), |(t, _)| t);
        while spaces != 0 && {
            t += 1;
            t <= self.count()
        } {
            let width = self.width(t);
            if spaces < width {
                return (tabs, spaces);
            }
            tabs += 1;
            spaces -= width;
        }
        let last = self.width(self.count());
        (tabs + spaces / last, spaces % last)
    }
}

/// Whether two lists name the same stops. A null list only equals another
/// null one.
pub fn eq(a: Option<&[colnr_T]>, b: Option<&[colnr_T]>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a[0] == b[0] && a[1..=a[0] as usize] == b[1..=b[0] as usize],
        _ => false,
    }
}

/// [`TabStops::padding`] for a uniform 'tabstop' of `ts`. A zero means the
/// default of eight.
pub fn uniform_padding(col: colnr_T, ts: OptInt) -> c_int {
    let ts = if ts == 0 { 8 } else { ts };
    (ts - col as OptInt % ts) as c_int
}

/// [`TabStops::from_to`] for a uniform 'tabstop' of `ts`.
pub fn uniform_from_to(start_col: colnr_T, end_col: colnr_T, ts: c_int) -> (c_int, c_int) {
    let mut spaces = end_col - start_col;
    let mut tabs = 0;
    let initspc = ts - start_col % ts;
    if spaces >= initspc {
        spaces -= initspc;
        tabs += 1;
    }
    tabs += spaces / ts;
    (tabs, spaces % ts)
}

/// Why a 'vartabstop' value was rejected. The caller reports it, because the
/// two messages differ in whether they name the offending part.
#[derive(Debug)]
pub enum ParseError {
    /// A stop is not a positive number; the offset names where.
    NotPositive(usize),
    /// The value is malformed; the offset names where, or 0 for the whole.
    Malformed(usize),
    /// A stop is out of range; the offset names where.
    OutOfRange(usize),
}

/// Parse a comma-separated 'vartabstop' value into the count-prefixed array
/// the option holds. An empty value, or a bare `0`, means "no list".
pub fn parse(var: &[u8]) -> Result<Option<Vec<colnr_T>>, ParseError> {
    if var.is_empty() || var == b"0" {
        return Ok(None);
    }

    // First pass: validate, and count the values.
    let mut valcount = 1;
    for (i, &byte) in var.iter().enumerate() {
        if i == 0 || var[i - 1] == b',' {
            // `strtol` accepts leading white space and a sign, so a part that
            // parses to a non-positive number is told apart from one that is
            // not a number at all.
            match leading_number(&var[i..]) {
                Some(n) if n > 0 => {}
                Some(_) => return Err(ParseError::NotPositive(i)),
                None => return Err(ParseError::Malformed(i)),
            }
        }
        if !byte.is_ascii_digit() {
            // A comma is only a separator between two values.
            if byte == b',' && i > 0 && var[i - 1] != b',' && i + 1 < var.len() {
                valcount += 1;
            } else {
                return Err(ParseError::Malformed(0));
            }
        }
    }

    let mut array = Vec::with_capacity(valcount as usize + 1);
    array.push(valcount);
    let mut at = 0;
    for part in var.split(|&b| b == b',') {
        // `atoi` here, so anything after the digits is simply ignored.
        let n = leading_number(part).unwrap_or(0);
        if n <= 0 || n > TABSTOP_MAX as i64 {
            return Err(ParseError::OutOfRange(at));
        }
        array.push(n as colnr_T);
        at += part.len() + 1;
    }
    Ok(Some(array))
}

/// The number `strtol`/`atoi` would read at the start of `s`, or `None` when
/// there are no digits at all.
fn leading_number(s: &[u8]) -> Option<i64> {
    let mut i = 0;
    while i < s.len() && (s[i] == b' ' || s[i] == b'\t') {
        i += 1;
    }
    let negative = i < s.len() && (s[i] == b'-' || s[i] == b'+') && {
        let minus = s[i] == b'-';
        i += 1;
        minus
    };
    let digits = i;
    let mut value: i64 = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        value = value
            .saturating_mul(10)
            .saturating_add((s[i] - b'0') as i64);
        i += 1;
    }
    (i > digits).then_some(if negative { -value } else { value })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `:set vartabstop=4,8,2` — the list nvim would build.
    fn stops(widths: &[colnr_T]) -> Vec<colnr_T> {
        let mut v = vec![widths.len() as colnr_T];
        v.extend_from_slice(widths);
        v
    }

    #[test]
    fn an_empty_or_zero_list_is_no_list() {
        assert!(TabStops::new(&[0]).is_none());
        assert!(TabStops::new(&[]).is_none());
        assert!(TabStops::new(&stops(&[4])).is_some());
    }

    #[test]
    fn padding_walks_the_named_stops_then_repeats_the_last() {
        let v = stops(&[4, 8, 2]);
        let ts = TabStops::new(&v).unwrap();
        // Stops end at columns 4, 12 and 14.
        assert_eq!(ts.padding(0), 4);
        assert_eq!(ts.padding(3), 1);
        assert_eq!(ts.padding(4), 8);
        assert_eq!(ts.padding(11), 1);
        assert_eq!(ts.padding(12), 2);
        // Past the list the final width of 2 repeats.
        assert_eq!(ts.padding(14), 2);
        assert_eq!(ts.padding(15), 1);
        assert_eq!(ts.padding(100), 2);
    }

    #[test]
    fn uniform_padding_defaults_to_eight() {
        assert_eq!(uniform_padding(0, 0), 8);
        assert_eq!(uniform_padding(3, 4), 1);
        assert_eq!(uniform_padding(4, 4), 4);
    }

    #[test]
    fn at_reports_the_width_of_the_stop_the_column_is_in() {
        let v = stops(&[4, 8, 2]);
        let ts = TabStops::new(&v).unwrap();
        assert_eq!(ts.at(0, false), 4);
        assert_eq!(ts.at(5, false), 8);
        assert_eq!(ts.at(13, false), 2);
        assert_eq!(ts.at(99, false), 2);
        // Moving left out of the first stop can only go back to column zero.
        assert_eq!(ts.at(3, true), 3);
        assert_eq!(ts.at(5, true), 4);
        assert_eq!(ts.at(13, true), 8);
    }

    #[test]
    fn start_reports_where_the_current_stop_begins() {
        let v = stops(&[4, 8, 2]);
        let ts = TabStops::new(&v).unwrap();
        assert_eq!(ts.start(0), 0);
        assert_eq!(ts.start(3), 0);
        assert_eq!(ts.start(4), 4);
        assert_eq!(ts.start(11), 4);
        assert_eq!(ts.start(12), 12);
        assert_eq!(ts.start(14), 14);
        assert_eq!(ts.start(15), 14);
        assert_eq!(ts.start(17), 16);
    }

    #[test]
    fn from_to_fills_a_run_with_tabs_and_a_remainder() {
        let v = stops(&[4, 8, 2]);
        let ts = TabStops::new(&v).unwrap();
        // Not even one stop away: spaces only.
        assert_eq!(ts.from_to(0, 3), (0, 3));
        assert_eq!(ts.from_to(0, 4), (1, 0));
        assert_eq!(ts.from_to(0, 12), (2, 0));
        assert_eq!(ts.from_to(0, 13), (2, 1));
        assert_eq!(ts.from_to(0, 14), (3, 0));
        // Into the repeating tail.
        assert_eq!(ts.from_to(0, 18), (5, 0));
        assert_eq!(ts.from_to(0, 19), (5, 1));
        assert_eq!(ts.from_to(5, 12), (1, 0));
    }

    #[test]
    fn uniform_from_to_matches_a_single_repeated_stop() {
        assert_eq!(uniform_from_to(0, 3, 4), (0, 3));
        assert_eq!(uniform_from_to(0, 4, 4), (1, 0));
        assert_eq!(uniform_from_to(0, 9, 4), (2, 1));
        assert_eq!(uniform_from_to(2, 8, 4), (2, 0));
    }

    #[test]
    fn lists_compare_by_content() {
        let a = stops(&[4, 8]);
        let b = stops(&[4, 8]);
        let c = stops(&[4, 9]);
        let d = stops(&[4]);
        assert!(eq(Some(&a), Some(&b)));
        assert!(!eq(Some(&a), Some(&c)));
        assert!(!eq(Some(&a), Some(&d)));
        assert!(eq(None, None));
        assert!(!eq(Some(&a), None));
        assert!(!eq(None, Some(&a)));
    }

    #[test]
    fn parsing_builds_a_count_prefixed_array() {
        assert_eq!(parse(b"4").unwrap(), Some(vec![1, 4]));
        assert_eq!(parse(b"4,8,2").unwrap(), Some(vec![3, 4, 8, 2]));
        assert_eq!(parse(b"").unwrap(), None);
        assert_eq!(parse(b"0").unwrap(), None);
    }

    #[test]
    fn parsing_rejects_what_the_option_code_rejects() {
        assert!(matches!(parse(b"-1"), Err(ParseError::NotPositive(0))));
        assert!(matches!(parse(b"4,-1"), Err(ParseError::NotPositive(2))));
        assert!(matches!(parse(b"x"), Err(ParseError::Malformed(0))));
        assert!(matches!(parse(b"4,,8"), Err(ParseError::Malformed(_))));
        assert!(matches!(parse(b"4,"), Err(ParseError::Malformed(_))));
        assert!(matches!(parse(b",4"), Err(ParseError::Malformed(_))));
        assert!(matches!(parse(b"10000"), Err(ParseError::OutOfRange(0))));
        assert!(matches!(parse(b"4,10000"), Err(ParseError::OutOfRange(2))));
    }

    #[test]
    fn leading_numbers_match_strtol() {
        assert_eq!(leading_number(b"12x"), Some(12));
        assert_eq!(leading_number(b"  -3"), Some(-3));
        assert_eq!(leading_number(b"+7"), Some(7));
        assert_eq!(leading_number(b"x"), None);
        assert_eq!(leading_number(b""), None);
        assert_eq!(leading_number(b"-"), None);
    }
}
