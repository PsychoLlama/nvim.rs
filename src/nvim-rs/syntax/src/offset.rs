//! Offset adjustment for syntax pattern matches.
//!
//! Handles applying start/end offsets from synpat_T to regex match positions.
//! These are the ms=, me=, hs=, he=, rs=, re= offsets in :syn commands.

use std::ffi::c_int;

use crate::state::Position;
use crate::types::SPO_COUNT;

extern "C" {
    fn nvim_syn_get_pattern_offset(pat_idx: c_int, off_idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_off_flags(pat_idx: c_int) -> c_int;
    fn nvim_syn_get_buf_line_count() -> c_int;
    fn nvim_syn_mb_adjust_col(lnum: c_int, col: c_int, off: c_int) -> c_int;
    fn nvim_syn_mb_adjust_col_start(lnum: c_int, col: c_int, off: c_int) -> c_int;
    fn nvim_syn_get_line_len(lnum: c_int) -> c_int;
}

/// A regex match result with start and end positions.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegMatch {
    pub startpos: Position,
    pub endpos: Position,
}

/// Add offset to matched text for end of match or highlight.
///
/// This computes the adjusted end position based on the pattern's offset
/// configuration (e.g., me=e+1, he=s+2).
///
/// # Arguments
/// * `regmatch` - The regex match start/end positions
/// * `pat_idx` - Index of the pattern in the synblock
/// * `off_idx` - Which offset to apply (SPO_ME_OFF, SPO_HE_OFF, etc.)
/// * `extra` - Extra chars for offset to start
pub fn syn_add_end_off(regmatch: &RegMatch, pat_idx: i32, off_idx: i32, extra: i32) -> Position {
    let off_flags = unsafe { nvim_syn_get_pattern_off_flags(pat_idx) };
    let offset_val = unsafe { nvim_syn_get_pattern_offset(pat_idx, off_idx) };

    let (lnum, col, off) = if off_flags & (1 << off_idx) != 0 {
        // Offset from start of match
        (
            regmatch.startpos.lnum,
            regmatch.startpos.col,
            offset_val + extra,
        )
    } else {
        // Offset from end of match
        (regmatch.endpos.lnum, regmatch.endpos.col, offset_val)
    };

    let line_count = unsafe { nvim_syn_get_buf_line_count() };
    let adjusted_col = if lnum > line_count {
        0
    } else if off != 0 {
        unsafe { nvim_syn_mb_adjust_col(lnum, col, off) }
    } else {
        col
    };

    Position {
        lnum,
        col: adjusted_col,
    }
}

/// Add offset to matched text for start of match or highlight.
/// Avoids resulting column becoming negative.
///
/// # Arguments
/// * `regmatch` - The regex match start/end positions
/// * `pat_idx` - Index of the pattern in the synblock
/// * `off_idx` - Which offset to apply (SPO_MS_OFF, SPO_HS_OFF, etc.)
/// * `extra` - Extra chars for offset to end
pub fn syn_add_start_off(regmatch: &RegMatch, pat_idx: i32, off_idx: i32, extra: i32) -> Position {
    let off_flags = unsafe { nvim_syn_get_pattern_off_flags(pat_idx) };
    let offset_val = unsafe { nvim_syn_get_pattern_offset(pat_idx, off_idx) };

    let (mut lnum, col, off) = if off_flags & (1 << (off_idx + SPO_COUNT)) != 0 {
        // Offset from end of match
        (
            regmatch.endpos.lnum,
            regmatch.endpos.col,
            offset_val + extra,
        )
    } else {
        // Offset from start of match
        (regmatch.startpos.lnum, regmatch.startpos.col, offset_val)
    };

    let line_count = unsafe { nvim_syn_get_buf_line_count() };
    if lnum > line_count {
        lnum = line_count;
        let line_len = unsafe { nvim_syn_get_line_len(lnum) };
        return Position {
            lnum,
            col: line_len,
        };
    }

    let adjusted_col = if off != 0 {
        unsafe { nvim_syn_mb_adjust_col_start(lnum, col, off) }
    } else {
        col
    };

    Position {
        lnum,
        col: adjusted_col,
    }
}

/// Limit a position to not be after a given limit position.
pub fn limit_pos(pos: &mut Position, limit: &Position) {
    if pos.lnum > limit.lnum {
        *pos = *limit;
    } else if pos.lnum == limit.lnum && pos.col > limit.col {
        pos.col = limit.col;
    }
}

/// Like limit_pos but if pos.lnum is zero, set pos to limit entirely.
pub fn limit_pos_zero(pos: &mut Position, limit: &Position) {
    if pos.lnum == 0 {
        *pos = *limit;
    } else {
        limit_pos(pos, limit);
    }
}

// =============================================================================
// Exported FFI functions
// =============================================================================

/// Rust implementation of limit_pos.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_limit_pos(
    pos_lnum: *mut c_int,
    pos_col: *mut c_int,
    limit_lnum: c_int,
    limit_col: c_int,
) {
    let mut pos = Position {
        lnum: *pos_lnum,
        col: *pos_col,
    };
    let limit = Position {
        lnum: limit_lnum,
        col: limit_col,
    };
    limit_pos(&mut pos, &limit);
    *pos_lnum = pos.lnum;
    *pos_col = pos.col;
}

/// Rust implementation of limit_pos_zero.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_limit_pos_zero(
    pos_lnum: *mut c_int,
    pos_col: *mut c_int,
    limit_lnum: c_int,
    limit_col: c_int,
) {
    let mut pos = Position {
        lnum: *pos_lnum,
        col: *pos_col,
    };
    let limit = Position {
        lnum: limit_lnum,
        col: limit_col,
    };
    limit_pos_zero(&mut pos, &limit);
    *pos_lnum = pos.lnum;
    *pos_col = pos.col;
}

/// Rust implementation of syn_add_end_off.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_add_end_off(
    result_lnum: *mut c_int,
    result_col: *mut c_int,
    start_lnum: c_int,
    start_col: c_int,
    end_lnum: c_int,
    end_col: c_int,
    pat_idx: c_int,
    off_idx: c_int,
    extra: c_int,
) {
    let regmatch = RegMatch {
        startpos: Position {
            lnum: start_lnum,
            col: start_col,
        },
        endpos: Position {
            lnum: end_lnum,
            col: end_col,
        },
    };
    let result = syn_add_end_off(&regmatch, pat_idx, off_idx, extra);
    *result_lnum = result.lnum;
    *result_col = result.col;
}

/// Rust implementation of syn_add_start_off.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_add_start_off(
    result_lnum: *mut c_int,
    result_col: *mut c_int,
    start_lnum: c_int,
    start_col: c_int,
    end_lnum: c_int,
    end_col: c_int,
    pat_idx: c_int,
    off_idx: c_int,
    extra: c_int,
) {
    let regmatch = RegMatch {
        startpos: Position {
            lnum: start_lnum,
            col: start_col,
        },
        endpos: Position {
            lnum: end_lnum,
            col: end_col,
        },
    };
    let result = syn_add_start_off(&regmatch, pat_idx, off_idx, extra);
    *result_lnum = result.lnum;
    *result_col = result.col;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_pos_no_change() {
        let mut pos = Position { lnum: 5, col: 10 };
        let limit = Position { lnum: 10, col: 20 };
        limit_pos(&mut pos, &limit);
        assert_eq!(pos, Position { lnum: 5, col: 10 });
    }

    #[test]
    fn test_limit_pos_lnum_exceeds() {
        let mut pos = Position { lnum: 15, col: 10 };
        let limit = Position { lnum: 10, col: 20 };
        limit_pos(&mut pos, &limit);
        assert_eq!(pos, limit);
    }

    #[test]
    fn test_limit_pos_col_exceeds() {
        let mut pos = Position { lnum: 10, col: 30 };
        let limit = Position { lnum: 10, col: 20 };
        limit_pos(&mut pos, &limit);
        assert_eq!(pos, Position { lnum: 10, col: 20 });
    }

    #[test]
    fn test_limit_pos_zero_with_zero_lnum() {
        let mut pos = Position { lnum: 0, col: 0 };
        let limit = Position { lnum: 10, col: 20 };
        limit_pos_zero(&mut pos, &limit);
        assert_eq!(pos, limit);
    }

    #[test]
    fn test_limit_pos_zero_with_nonzero_lnum() {
        let mut pos = Position { lnum: 5, col: 10 };
        let limit = Position { lnum: 10, col: 20 };
        limit_pos_zero(&mut pos, &limit);
        assert_eq!(pos, Position { lnum: 5, col: 10 });
    }

    #[test]
    fn test_regmatch_default() {
        let m = RegMatch::default();
        assert_eq!(m.startpos, Position::default());
        assert_eq!(m.endpos, Position::default());
    }
}
