#![forbid(unsafe_code)]

//! Parsing of control sequences (CSI) and their parameters.
//!
//! Everything here works on a byte slice of the input buffer and yields byte
//! ranges, so the driver keeps the raw-pointer bookkeeping and this stays pure.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

use crate::tui::termkey::termkey::{
    TERMKEY_EVENT_PRESS, TERMKEY_EVENT_RELEASE, TERMKEY_EVENT_REPEAT, TERMKEY_EVENT_UNKNOWN,
};
use crate::types::TermKeyEvent;
use core::ffi::c_int;

/// How many parameters the parser records. Upstream hardcoded this and ignored
/// the caller's array size, so a caller must supply at least this many slots.
pub const CSI_MAX_PARAMS: usize = 16;

/// A parsed control sequence.
pub struct CsiSeq {
    /// One entry per parameter: its byte range within the parsed slice, or
    /// `None` for an omitted parameter (`CSI 1;;3 m`).
    pub params: [Option<(usize, usize)>; CSI_MAX_PARAMS],
    pub nparams: usize,
    /// The final byte, plus the private-use introducer in bits 8-15 and the
    /// intermediate byte in bits 16-23 when present — the packing the key
    /// handlers and nvim's unknown-CSI reporting both match on.
    pub command: u32,
    /// Bytes consumed, counting the introducer and the final byte.
    pub len: usize,
}

/// Parse the control sequence starting at `intro_len` bytes into `bytes`.
///
/// Returns `None` when the sequence is unterminated, i.e. more input could
/// still complete it.
pub fn parse(bytes: &[u8], intro_len: usize) -> Option<CsiSeq> {
    // The sequence ends at the first byte in the final-byte range.
    let end = (intro_len..bytes.len()).find(|&i| (0x40..0x80).contains(&bytes[i]))?;

    let mut command = bytes[end] as u32;
    let mut params: [Option<(usize, usize)>; CSI_MAX_PARAMS] = [None; CSI_MAX_PARAMS];
    let mut nparams = 0;
    let mut start: Option<usize> = None;
    let mut pos = intro_len;

    // A private-use introducer ('<' to '?') goes in the second byte.
    if (b'<'..=b'?').contains(&bytes[pos]) {
        command |= (bytes[pos] as u32) << 8;
        pos += 1;
    }
    while pos < end {
        let byte = bytes[pos];
        if (b'0'..b';').contains(&byte) {
            // Digits and the sub-parameter separator ':' both belong to the
            // parameter; `param_groups` splits them apart later.
            start.get_or_insert(pos);
        } else if byte == b';' {
            params[nparams] = start.map(|s| (s, pos));
            start = None;
            nparams += 1;
            if nparams >= CSI_MAX_PARAMS {
                break;
            }
        } else if (0x20..=0x2f).contains(&byte) {
            // An intermediate byte ends the parameters and joins the command.
            command |= (byte as u32) << 16;
            break;
        }
        pos += 1;
    }
    if let Some(s) = start {
        params[nparams] = Some((s, pos));
        nparams += 1;
    }
    Some(CsiSeq {
        params,
        nparams,
        command,
        len: end + 1,
    })
}

/// Split one parameter into its value and first sub-parameter.
///
/// `CSI 27;5u` has parameters "27" and "5"; `CSI 27;5:3u` has "27" and "5:3",
/// whose sub-parameter 3 is the key event. An omitted parameter reads as -1.
///
/// Upstream wrote the sub-parameters into a caller-supplied array bounded by an
/// in/out count, but its post-loop flush ran one slot past that bound: with the
/// single-slot array every caller passed, a parameter with two colons (`CSI
/// 27;5:3:1u`, which any program under `:terminal` can emit) wrote past the end
/// of a stack variable. Returning the one sub-parameter the callers actually
/// read has no such edge.
pub fn param_groups(param: Option<&[u8]>) -> (c_int, Option<c_int>) {
    let Some(bytes) = param else {
        return (-1, None);
    };
    let mut groups = bytes.split(|&b| b == b':').map(|group| {
        let mut value: c_int = 0;
        for &byte in group {
            // `parse` only ever admits digits and ':' into a parameter.
            debug_assert!(byte.is_ascii_digit());
            // A terminal can send arbitrarily many digits; upstream wrapped
            // rather than trapping, and a debug build of this tree would panic.
            value = value.wrapping_mul(10).wrapping_add((byte - b'0') as c_int);
        }
        value
    });
    (groups.next().unwrap_or(0), groups.next())
}

/// The key event a `CSI ... : N u` sub-parameter names.
pub fn parse_key_event(n: c_int) -> TermKeyEvent {
    match n {
        1 => TERMKEY_EVENT_PRESS,
        2 => TERMKEY_EVENT_REPEAT,
        3 => TERMKEY_EVENT_RELEASE,
        _ => TERMKEY_EVENT_UNKNOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(seq: &CsiSeq, bytes: &[u8]) -> Vec<Option<c_int>> {
        seq.params[..seq.nparams]
            .iter()
            .map(|p| p.map(|(s, e)| param_groups(Some(&bytes[s..e])).0))
            .collect()
    }

    #[test]
    fn parses_parameters_and_the_final_byte() {
        let bytes = b"\x1b[5;25v";
        let seq = parse(bytes, 2).unwrap();
        assert_eq!(seq.command, b'v' as u32);
        assert_eq!(seq.len, bytes.len());
        assert_eq!(args(&seq, bytes), [Some(5), Some(25)]);
    }

    #[test]
    fn packs_the_private_introducer_and_intermediate_into_the_command() {
        let seq = parse(b"\x1b[?w", 2).unwrap();
        assert_eq!(seq.command, (b'?' as u32) << 8 | b'w' as u32);
        assert_eq!(seq.nparams, 0);

        let seq = parse(b"\x1b[?$x", 2).unwrap();
        assert_eq!(
            seq.command,
            (b'$' as u32) << 16 | (b'?' as u32) << 8 | b'x' as u32
        );
    }

    #[test]
    fn an_omitted_parameter_reads_as_minus_one() {
        let bytes = b"\x1b[1;;3m";
        let seq = parse(bytes, 2).unwrap();
        assert_eq!(args(&seq, bytes), [Some(1), None, Some(3)]);
        assert_eq!(param_groups(None), (-1, None));
    }

    #[test]
    fn an_unterminated_sequence_needs_more_input() {
        assert!(parse(b"\x1b[", 2).is_none());
        assert!(parse(b"\x1b[12;", 2).is_none());
        assert!(parse(b"\x9b1", 1).is_none());
    }

    #[test]
    fn stops_recording_after_the_parameter_limit() {
        let mut bytes = b"\x1b[".to_vec();
        bytes.extend(std::iter::repeat_n(b';', 40));
        bytes.push(b'm');
        let seq = parse(&bytes, 2).unwrap();
        assert_eq!(seq.nparams, CSI_MAX_PARAMS);
        assert_eq!(seq.len, bytes.len());
    }

    #[test]
    fn sub_parameters_split_on_colons() {
        assert_eq!(param_groups(Some(b"5")), (5, None));
        assert_eq!(param_groups(Some(b"5:3")), (5, Some(3)));
        // The third group and beyond are ignored, not written past the end.
        assert_eq!(param_groups(Some(b"5:3:1")), (5, Some(3)));
        assert_eq!(param_groups(Some(b":3")), (0, Some(3)));
        assert_eq!(param_groups(Some(b"")), (0, None));
    }

    #[test]
    fn a_long_run_of_digits_wraps_instead_of_trapping() {
        assert_eq!(param_groups(Some(b"99999999999999999999")).0, 1661992959);
    }

    #[test]
    fn key_events_outside_the_known_set_are_unknown() {
        assert_eq!(parse_key_event(1), TERMKEY_EVENT_PRESS);
        assert_eq!(parse_key_event(3), TERMKEY_EVENT_RELEASE);
        assert_eq!(parse_key_event(0), TERMKEY_EVENT_UNKNOWN);
        assert_eq!(parse_key_event(9), TERMKEY_EVENT_UNKNOWN);
    }
}
