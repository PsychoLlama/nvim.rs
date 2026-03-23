//! Line manipulation command implementations.
//!
//! This module provides implementations for Ex commands that manipulate lines:
//! - `:copy` (`:t`) - Copy lines to another location
//! - `:move` (`:m`) - Move lines to another location
//! - `:delete` (`:d`) - Delete lines
//! - `:yank` (`:y`) - Yank lines to a register
//! - `:put` (`:pu`) - Put register contents
//! - `:join` (`:j`) - Join lines together
//!
//! ## Implementation Notes
//!
//! These commands work with line ranges and optionally registers.
//! The actual buffer modifications are performed by Neovim's core functions.

use std::ffi::{c_char, c_int};

use crate::range::{LineNr, LineRange};

/// Type of line manipulation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineOp {
    /// Copy lines (`:t`, `:copy`)
    Copy,
    /// Move lines (`:m`, `:move`)
    Move,
    /// Delete lines (`:d`, `:delete`)
    Delete,
    /// Yank lines to register (`:y`, `:yank`)
    Yank,
    /// Put from register (`:pu`, `:put`)
    Put,
    /// Join lines (`:j`, `:join`)
    Join,
}

impl LineOp {
    /// Check if this operation modifies the buffer.
    #[inline]
    #[must_use]
    pub const fn modifies_buffer(&self) -> bool {
        matches!(
            self,
            LineOp::Copy | LineOp::Move | LineOp::Delete | LineOp::Put | LineOp::Join
        )
    }

    /// Check if this operation uses a register.
    #[inline]
    #[must_use]
    pub const fn uses_register(&self) -> bool {
        matches!(self, LineOp::Yank | LineOp::Put | LineOp::Delete)
    }

    /// Check if this operation requires a destination.
    #[inline]
    #[must_use]
    pub const fn requires_destination(&self) -> bool {
        matches!(self, LineOp::Copy | LineOp::Move)
    }
}

/// Options for copy and move operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CopyMoveOptions {
    /// Source range of lines.
    pub range: LineRange,
    /// Destination line (insert after this line).
    pub dest: LineNr,
}

impl CopyMoveOptions {
    /// Create options for copying/moving a range to a destination.
    #[must_use]
    pub const fn new(range: LineRange, dest: LineNr) -> Self {
        Self { range, dest }
    }

    /// Check if the destination is valid for copy/move.
    ///
    /// For copy: destination can be anywhere.
    /// For move: destination must not be within the source range.
    #[must_use]
    pub fn is_valid_for_move(&self) -> bool {
        // Destination cannot be within the source range
        self.dest < self.range.start || self.dest > self.range.end
    }

    /// Get the number of lines in the source range.
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> LineNr {
        self.range.len()
    }
}

/// Options for delete operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeleteOptions {
    /// Range of lines to delete.
    pub range: LineRange,
    /// Register to store deleted text (0 for default register).
    pub register: u8,
    /// Whether to save to register.
    pub use_register: bool,
}

impl DeleteOptions {
    /// Create options for deleting a range.
    #[must_use]
    pub const fn new(range: LineRange) -> Self {
        Self {
            range,
            register: 0,
            use_register: true,
        }
    }

    /// Create options for deleting without saving to register.
    #[must_use]
    pub const fn without_register(range: LineRange) -> Self {
        Self {
            range,
            register: 0,
            use_register: false,
        }
    }

    /// Set the register to use.
    #[must_use]
    pub const fn with_register(mut self, register: u8) -> Self {
        self.register = register;
        self.use_register = true;
        self
    }
}

/// Options for yank operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct YankOptions {
    /// Range of lines to yank.
    pub range: LineRange,
    /// Register to yank to (0 for default register).
    pub register: u8,
}

impl YankOptions {
    /// Create options for yanking a range to the default register.
    #[must_use]
    pub const fn new(range: LineRange) -> Self {
        Self { range, register: 0 }
    }

    /// Create options for yanking to a specific register.
    #[must_use]
    pub const fn to_register(range: LineRange, register: u8) -> Self {
        Self { range, register }
    }
}

/// Options for put operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PutOptions {
    /// Line to put after (0 = before first line).
    pub line: LineNr,
    /// Register to put from (0 for default register).
    pub register: u8,
    /// Put before the line instead of after.
    pub before: bool,
}

impl PutOptions {
    /// Create options for putting after a line.
    #[must_use]
    pub const fn after(line: LineNr) -> Self {
        Self {
            line,
            register: 0,
            before: false,
        }
    }

    /// Create options for putting before a line.
    #[must_use]
    pub const fn before(line: LineNr) -> Self {
        Self {
            line,
            register: 0,
            before: true,
        }
    }

    /// Set the register to put from.
    #[must_use]
    pub const fn from_register(mut self, register: u8) -> Self {
        self.register = register;
        self
    }
}

/// Options for join operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JoinOptions {
    /// Range of lines to join.
    pub range: LineRange,
    /// Don't insert spaces when joining.
    pub no_space: bool,
}

impl JoinOptions {
    /// Create options for joining a range with spaces.
    #[must_use]
    pub const fn new(range: LineRange) -> Self {
        Self {
            range,
            no_space: false,
        }
    }

    /// Create options for joining without spaces (gJ style).
    #[must_use]
    pub const fn without_space(range: LineRange) -> Self {
        Self {
            range,
            no_space: true,
        }
    }
}

/// Result of a line manipulation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineOpResult {
    /// Number of lines affected.
    pub lines: i32,
    /// Whether the buffer was changed.
    pub changed: bool,
}

impl LineOpResult {
    /// Create a new result.
    #[must_use]
    pub const fn new(lines: i32, changed: bool) -> Self {
        Self { lines, changed }
    }

    /// Create a result indicating no change.
    #[must_use]
    pub const fn no_change() -> Self {
        Self {
            lines: 0,
            changed: false,
        }
    }
}

/// Error type for line manipulation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineError {
    /// Invalid range.
    InvalidRange,
    /// Invalid destination.
    InvalidDestination(LineNr),
    /// Cannot move into own range.
    MoveIntoSelf,
    /// Invalid register.
    InvalidRegister(u8),
    /// Empty register.
    EmptyRegister(u8),
    /// Buffer is readonly.
    Readonly,
}

impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineError::InvalidRange => write!(f, "invalid range"),
            LineError::InvalidDestination(dest) => write!(f, "invalid destination: {dest}"),
            LineError::MoveIntoSelf => write!(f, "cannot move to same range"),
            LineError::InvalidRegister(r) => write!(f, "invalid register: {}", *r as char),
            LineError::EmptyRegister(r) => write!(f, "register is empty: {}", *r as char),
            LineError::Readonly => write!(f, "buffer is readonly"),
        }
    }
}

impl std::error::Error for LineError {}

/// Validate a copy/move operation.
///
/// # Arguments
/// * `range` - Source range
/// * `dest` - Destination line
/// * `line_count` - Total lines in buffer
/// * `is_move` - True if this is a move operation
///
/// # Returns
/// `Ok(())` if valid, `Err` with the error otherwise.
pub fn validate_copy_move(
    range: LineRange,
    dest: LineNr,
    line_count: LineNr,
    is_move: bool,
) -> Result<(), LineError> {
    // Validate range
    if range.is_empty() {
        return Err(LineError::InvalidRange);
    }
    if range.start < 1 || range.end > line_count {
        return Err(LineError::InvalidRange);
    }

    // Validate destination
    if dest < 0 || dest > line_count {
        return Err(LineError::InvalidDestination(dest));
    }

    // For move, destination can't be within the source range
    if is_move && dest >= range.start && dest < range.end {
        return Err(LineError::MoveIntoSelf);
    }

    Ok(())
}

/// Calculate the new line number after a move operation.
///
/// When lines are moved, line numbers can shift. This function calculates
/// where a given line number ends up after moving a range.
///
/// # Arguments
/// * `lnum` - Line number to track
/// * `range` - Range being moved
/// * `dest` - Destination line
///
/// # Returns
/// The new line number after the move.
#[must_use]
pub fn adjust_line_after_move(lnum: LineNr, range: LineRange, dest: LineNr) -> LineNr {
    let count = range.len();

    if dest < range.start {
        // Moving up
        if lnum >= range.start && lnum <= range.end {
            // Line is in the moved range
            lnum - range.start + dest + 1
        } else if lnum > dest && lnum < range.start {
            // Line is between destination and source - shifts down
            lnum + count
        } else {
            // Line is outside affected area
            lnum
        }
    } else {
        // Moving down (dest > range.end)
        if lnum >= range.start && lnum <= range.end {
            // Line is in the moved range
            lnum + (dest - range.end)
        } else if lnum > range.end && lnum <= dest {
            // Line is between source and destination - shifts up
            lnum - count
        } else {
            // Line is outside affected area
            lnum
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FAIL constant from vim_defs.h
const FAIL: c_int = 0;

/// ML_EMPTY flag from memline_defs.h
const ML_EMPTY: c_int = 0x01;

/// `:change` command implementation.
///
/// Deletes the specified range of lines and then calls `:append` to
/// interactively insert replacement text.
///
/// # Safety
/// `eap` must be a valid pointer to an exarg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_change(eap: *mut crate::ExArgHandle) {
    use crate::{
        deleted_lines_mark, get_indent_lnum, ml_delete, nvim_check_cursor_lnum_call,
        nvim_curbuf_get_b_p_ai, nvim_curbuf_get_ml_flags, nvim_exarg_get_forceit,
        nvim_exarg_get_line1, nvim_exarg_get_line2, nvim_exarg_set_line2, u_save,
    };

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);

    if line2 >= line1 && u_save(line1 - 1, line2 + 1) == FAIL {
        return;
    }

    // The ! flag toggles autoindent
    let forceit = nvim_exarg_get_forceit(eap) != 0;
    let b_p_ai = nvim_curbuf_get_b_p_ai() != 0;
    if if forceit { !b_p_ai } else { b_p_ai } {
        crate::append_indent = get_indent_lnum(line1);
    }

    let mut lnum = line2;
    while lnum >= line1 {
        if (nvim_curbuf_get_ml_flags() & ML_EMPTY) != 0 {
            // nothing to delete
            break;
        }
        ml_delete(line1);
        lnum -= 1;
    }

    // Make sure the cursor is not beyond the end of the file now
    nvim_check_cursor_lnum_call();
    deleted_lines_mark(line1, line2 - lnum);

    // ":append" on the line above the deleted lines
    nvim_exarg_set_line2(eap, line1);
    rs_ex_append(eap);
}

/// Constants for State global.
const MODE_INSERT: c_int = 0x10;
const MODE_LANGMAP: c_int = 0x20;
const MODE_CMDLINE: c_int = 0x08;
const MODE_NORMAL: c_int = 0x01;

/// B_IMODE_LMAP constant from buffer_defs.h
const B_IMODE_LMAP: c_int = 1;

/// NUL character (C NUL = '\0').
const NUL: c_char = 0;

/// NL character (newline = '\n').
const NL: c_char = 0x0A;

/// TAB character.
const TAB: c_char = 0x09;

/// BL_SOL constant for beginline.
const BL_SOL: c_int = 2;
/// BL_FIX constant for beginline.
const BL_FIX: c_int = 4;

extern "C" {
    static mut State: c_int;
}

/// `:insert` and `:append` command implementation, also used by `:change`.
///
/// # Safety
/// `eap` must be a valid pointer to an exarg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_append(eap: *mut crate::ExArgHandle) {
    use crate::{
        appended_lines, appended_lines_mark, beginline, get_indent_lnum, ml_append, ml_delete,
        nvim_check_cursor_lnum_call, nvim_cmdmod_has_lockmarks, nvim_curbuf_get_b_ml_ml_line_count,
        nvim_curbuf_get_b_p_ai, nvim_curbuf_get_ml_flags, nvim_curbuf_set_op_end,
        nvim_curbuf_set_op_start, nvim_curwin_set_cursor_lnum, nvim_exarg_get_cmdidx,
        nvim_exarg_get_forceit, nvim_exarg_get_line2, nvim_excmds_call_getline,
        nvim_excmds_ea_getline_is_null, nvim_excmds_get_arg_mut, nvim_excmds_get_b_p_iminsert,
        nvim_excmds_get_cstack_looplevel, nvim_excmds_get_nextcmd, nvim_excmds_set_nextcmd_direct,
        nvim_excmds_toggle_b_p_ai, nvim_get_Rows, nvim_set_ex_no_reprint, nvim_set_lines_left,
        nvim_set_msg_scroll, nvim_set_need_wait_return, nvim_ui_cursor_shape_wrapper, u_save,
        vim_strchr, xfree, xmemdupz, xstrdup,
    };

    let mut did_undo = false;
    let mut lnum = nvim_exarg_get_line2(eap);
    let mut indent: c_int = 0;
    let empty = (nvim_curbuf_get_ml_flags() & ML_EMPTY) != 0;
    let cmdidx = nvim_exarg_get_cmdidx(eap);
    let forceit = nvim_exarg_get_forceit(eap) != 0;
    // the ! flag toggles autoindent
    if forceit {
        nvim_excmds_toggle_b_p_ai();
    }

    // First autoindent comes from the line we start on
    if cmdidx != crate::CMD_CHANGE && nvim_curbuf_get_b_p_ai() != 0 && lnum > 0 {
        crate::append_indent = get_indent_lnum(lnum);
    }

    if cmdidx != crate::CMD_APPEND {
        lnum -= 1;
    }

    // when the buffer is empty need to delete the dummy line
    let mut empty_flag = empty;
    if empty_flag && lnum == 1 {
        lnum = 0;
    }

    // behave like in Insert mode
    State = MODE_INSERT;
    if nvim_excmds_get_b_p_iminsert() == B_IMODE_LMAP {
        State |= MODE_LANGMAP;
    }

    loop {
        nvim_set_msg_scroll(1);
        nvim_set_need_wait_return(0);
        if nvim_curbuf_get_b_p_ai() != 0 {
            let append_indent_val = crate::append_indent;
            if append_indent_val >= 0 {
                indent = append_indent_val;
                crate::append_indent = -1;
            } else if lnum > 0 {
                indent = get_indent_lnum(lnum);
            }
        }

        let arg = nvim_excmds_get_arg_mut(eap);
        let theline: *mut c_char;

        if *arg == b'|' as c_char {
            // Get the text after the trailing bar.
            theline = xstrdup(arg.add(1));
            *arg = NUL;
        } else if nvim_excmds_ea_getline_is_null(eap) != 0 {
            // No getline() function, use the lines that follow.
            let nextcmd = nvim_excmds_get_nextcmd(eap);
            if nextcmd.is_null() {
                break;
            }
            let mut p = vim_strchr(nextcmd, c_int::from(NL));
            if p.is_null() {
                p = nextcmd.add(libc::strlen(nextcmd.cast()));
            }
            theline = xmemdupz(nextcmd, p.offset_from(nextcmd) as usize);
            if *p != NUL {
                // advance past the NL
                nvim_excmds_set_nextcmd_direct(eap, (p as *mut c_char).add(1));
            } else {
                nvim_excmds_set_nextcmd_direct(eap, std::ptr::null_mut());
            }
        } else {
            let save_state = State;
            // Set State to avoid cursor shape being set to MODE_INSERT when getline() returns.
            State = MODE_CMDLINE;
            let c = if nvim_excmds_get_cstack_looplevel(eap) > 0 {
                -1
            } else {
                c_int::from(NUL)
            };
            theline = nvim_excmds_call_getline(eap, c, indent);
            State = save_state;
        }
        nvim_set_lines_left(nvim_get_Rows() - 1);
        if theline.is_null() {
            break;
        }

        // Look for the "." after automatic indent.
        let mut vcol: c_int = 0;
        let mut p = theline;
        while indent > vcol {
            if *p == b' ' as c_char {
                vcol += 1;
            } else if *p == TAB {
                vcol += 8 - vcol % 8;
            } else {
                break;
            }
            p = p.add(1);
        }
        if (*p == b'.' as c_char && *p.add(1) == NUL)
            || (!did_undo && u_save(lnum, lnum + 1 + if empty_flag { 1 } else { 0 }) == FAIL)
        {
            xfree(theline.cast());
            break;
        }

        // don't use autoindent if nothing was typed.
        if *p == NUL {
            *theline = NUL;
        }

        did_undo = true;
        ml_append(lnum, theline, 0, 0);
        if empty_flag {
            // there are no marks below the inserted lines
            appended_lines(lnum, 1);
        } else {
            appended_lines_mark(lnum, 1);
        }

        xfree(theline.cast());
        lnum += 1;

        if empty_flag {
            ml_delete(2);
            empty_flag = false;
        }
    }
    State = MODE_NORMAL;
    nvim_ui_cursor_shape_wrapper();

    if forceit {
        nvim_excmds_toggle_b_p_ai();
    }

    // "start" is set to eap->line2+1 unless that position is invalid
    // "end" is set to lnum when something has been appended, otherwise same as "start"
    if nvim_cmdmod_has_lockmarks() == 0 {
        let line2 = nvim_exarg_get_line2(eap);
        let ml_line_count = nvim_curbuf_get_b_ml_ml_line_count();
        let mut start_lnum = if line2 < ml_line_count {
            line2 + 1
        } else {
            ml_line_count
        };
        if cmdidx != crate::CMD_APPEND {
            start_lnum -= 1;
        }
        let end_lnum = if line2 < lnum { lnum } else { start_lnum };
        nvim_curbuf_set_op_start(start_lnum, 0);
        nvim_curbuf_set_op_end(end_lnum, 0);
    }
    nvim_curwin_set_cursor_lnum(lnum);
    nvim_check_cursor_lnum_call();
    beginline(BL_SOL | BL_FIX);

    nvim_set_need_wait_return(0); // don't use wait_return() now
    nvim_set_ex_no_reprint(1);
}

/// `:copy`/`:t` command implementation.
///
/// Copies lines from `line1..=line2` to after line `n`.
///
/// # Safety
/// Must be called from Neovim's main thread with valid buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_copy(line1: c_int, line2: c_int, n: c_int) {
    use crate::{
        appended_lines_mark, ml_append, ml_get, ml_get_len, msgmore, nvim_check_pos_visual,
        nvim_cmdmod_has_lockmarks, nvim_curbuf_set_op_end, nvim_curbuf_set_op_start,
        nvim_curwin_get_cursor_lnum, nvim_curwin_set_cursor_lnum, nvim_get_visual_active, u_save,
        xfree, xstrnsave,
    };

    let count = line2 - line1 + 1;

    if nvim_cmdmod_has_lockmarks() == 0 {
        nvim_curbuf_set_op_start(n + 1, 0);
        nvim_curbuf_set_op_end(n + count, 0);
    }

    // There are three situations:
    // 1. destination is above line1
    // 2. destination is between line1 and line2
    // 3. destination is below line2
    if u_save(n, n + 1) == FAIL {
        return;
    }

    nvim_curwin_set_cursor_lnum(n);
    let mut l1 = line1;
    let mut l2 = line2;
    while l1 <= l2 {
        // Need to make a copy because the line will be unlocked within ml_append()
        let src = ml_get(l1);
        let src_len = ml_get_len(l1);
        let p = xstrnsave(src, src_len as usize);
        let cursor_lnum = nvim_curwin_get_cursor_lnum();
        ml_append(cursor_lnum, p, 0, 0);
        xfree(p.cast());

        // situation 2: skip already copied lines
        if l1 == n {
            l1 = nvim_curwin_get_cursor_lnum();
        }
        l1 += 1;
        let cursor_lnum = nvim_curwin_get_cursor_lnum();
        if cursor_lnum < l1 {
            l1 += 1;
        }
        if cursor_lnum < l2 {
            l2 += 1;
        }
        nvim_curwin_set_cursor_lnum(cursor_lnum + 1);
    }

    appended_lines_mark(n, count);
    if nvim_get_visual_active() != 0 {
        nvim_check_pos_visual();
    }

    msgmore(count);
}

/// OK constant from vim_defs.h
const OK: c_int = 1;

/// ML_DEL_MESSAGE flag for ml_delete_flags.
const ML_DEL_MESSAGE: c_int = 1;

/// kExtmarkNOOP: extmarks shouldn't be moved.
const KEXTMARK_NOOP_MOVE: c_int = 0;
/// kExtmarkUndo: operation should be reversible.
const KEXTMARK_UNDO_MOVE: c_int = 1;

/// `:move` command implementation.
///
/// Moves lines line1-line2 to after line dest.
/// Returns FAIL (0) for failure, OK (1) otherwise.
///
/// # Safety
/// Must be called from Neovim's main thread with valid buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_do_move(line1: c_int, line2: c_int, mut dest: c_int) -> c_int {
    use crate::{
        changed_lines, ml_append, ml_get, ml_get_len, nvim_cmdmod_has_lockmarks,
        nvim_curbuf_get_b_ml_ml_line_count, nvim_curbuf_set_op_end, nvim_curbuf_set_op_start,
        nvim_curwin_set_cursor_lnum, nvim_excmds_buf_updates_send_changes, nvim_excmds_emsg_e134,
        nvim_excmds_extmark_move_region, nvim_excmds_fold_move_range_all_wins,
        nvim_excmds_mark_adjust_nofold, nvim_excmds_ml_delete_flags,
        nvim_excmds_ml_find_line_or_offset, nvim_excmds_smsg_lines_moved, nvim_get_curbuf, u_save,
        xfree, xstrnsave,
    };

    if dest >= line1 && dest < line2 {
        nvim_excmds_emsg_e134();
        return FAIL;
    }

    // Do nothing if we are not actually moving any lines.
    if dest == line1 - 1 || dest == line2 {
        let cursor_lnum = if dest >= line1 {
            dest
        } else {
            dest + (line2 - line1) + 1
        };
        nvim_curwin_set_cursor_lnum(cursor_lnum);
        return OK;
    }

    let start_byte = nvim_excmds_ml_find_line_or_offset(line1);
    let end_byte = nvim_excmds_ml_find_line_or_offset(line2 + 1);
    let extent_byte = end_byte - start_byte;
    let dest_byte = nvim_excmds_ml_find_line_or_offset(dest + 1);

    let num_lines = line2 - line1 + 1;

    // First we copy the old text to its new location.
    if u_save(dest, dest + 1) == FAIL {
        return FAIL;
    }

    let mut extra: c_int = 0;
    for l in line1..=line2 {
        let str = xstrnsave(ml_get(l + extra), ml_get_len(l + extra) as usize);
        ml_append(dest + l - line1, str, 0, 0);
        xfree(str.cast());
        if dest < line1 {
            extra += 1;
        }
    }

    // Adjust marks: first move marks in old text to end of file (temporarily).
    let mut last_line = nvim_curbuf_get_b_ml_ml_line_count();
    nvim_excmds_mark_adjust_nofold(line1, line2, last_line - line2, 0, KEXTMARK_NOOP_MOVE);

    crate::disable_fold_update += 1;
    changed_lines(
        nvim_get_curbuf(),
        last_line - num_lines + 1,
        0,
        last_line + 1,
        num_lines,
        0,
    );
    crate::disable_fold_update -= 1;

    let mut line_off: c_int = 0;
    let mut byte_off: i64 = 0;
    if dest >= line2 {
        nvim_excmds_mark_adjust_nofold(line2 + 1, dest, -num_lines, 0, KEXTMARK_NOOP_MOVE);
        nvim_excmds_fold_move_range_all_wins(line1, line2, dest);
        if nvim_cmdmod_has_lockmarks() == 0 {
            nvim_curbuf_set_op_start(dest - num_lines + 1, 0);
            nvim_curbuf_set_op_end(dest, 0);
        }
        line_off = -num_lines;
        byte_off = -extent_byte;
    } else {
        nvim_excmds_mark_adjust_nofold(dest + 1, line1 - 1, num_lines, 0, KEXTMARK_NOOP_MOVE);
        nvim_excmds_fold_move_range_all_wins(dest + 1, line1 - 1, line2);
        if nvim_cmdmod_has_lockmarks() == 0 {
            nvim_curbuf_set_op_start(dest + 1, 0);
            nvim_curbuf_set_op_end(dest + num_lines, 0);
        }
    }
    nvim_excmds_mark_adjust_nofold(
        last_line - num_lines + 1,
        last_line,
        -(last_line - dest - extra),
        0,
        KEXTMARK_NOOP_MOVE,
    );

    crate::disable_fold_update += 1;
    changed_lines(
        nvim_get_curbuf(),
        last_line - num_lines + 1,
        0,
        last_line + 1,
        -extra,
        0,
    );
    crate::disable_fold_update -= 1;

    // Send update regarding the new lines that were added.
    nvim_excmds_buf_updates_send_changes(dest + 1, i64::from(num_lines), 0);

    // Now delete the original text.
    if u_save(line1 + extra - 1, line2 + extra + 1) == FAIL {
        return FAIL;
    }

    for _ in line1..=line2 {
        nvim_excmds_ml_delete_flags(line1 + extra, ML_DEL_MESSAGE);
    }
    if crate::global_busy == 0 && i64::from(num_lines) > crate::p_report {
        nvim_excmds_smsg_lines_moved(i64::from(num_lines));
    }

    nvim_excmds_extmark_move_region(
        line1 - 1,
        0,
        start_byte,
        line2 - line1 + 1,
        0,
        extent_byte,
        dest + line_off,
        0,
        dest_byte + byte_off,
        KEXTMARK_UNDO_MOVE,
    );

    // Leave the cursor on the last of the moved lines.
    if dest >= line1 {
        nvim_curwin_set_cursor_lnum(dest);
    } else {
        nvim_curwin_set_cursor_lnum(dest + (line2 - line1) + 1);
    }

    if line1 < dest {
        dest += num_lines + 1;
        last_line = nvim_curbuf_get_b_ml_ml_line_count();
        if dest > last_line + 1 {
            dest = last_line + 1;
        }
        changed_lines(nvim_get_curbuf(), line1, 0, dest, 0, 0);
    } else {
        changed_lines(nvim_get_curbuf(), dest + 1, 0, line1 + num_lines, 0, 0);
    }

    // Send nvim_buf_lines_event regarding lines that were deleted.
    nvim_excmds_buf_updates_send_changes(line1 + extra, 0, i64::from(num_lines));

    OK
}

/// Validate a copy operation.
///
/// Returns 1 if valid, 0 if invalid.
pub extern "C" fn rs_validate_copy(
    start: c_int,
    end: c_int,
    dest: c_int,
    line_count: c_int,
) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_copy_move(range, dest, line_count, false).is_ok())
}

/// Validate a move operation.
///
/// Returns 1 if valid, 0 if invalid.
pub extern "C" fn rs_validate_move(
    start: c_int,
    end: c_int,
    dest: c_int,
    line_count: c_int,
) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_copy_move(range, dest, line_count, true).is_ok())
}

/// Adjust a line number after a move operation.
pub extern "C" fn rs_adjust_line_after_move(
    lnum: c_int,
    start: c_int,
    end: c_int,
    dest: c_int,
) -> c_int {
    let range = LineRange::new(start, end);
    adjust_line_after_move(lnum, range, dest)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_op() {
        assert!(LineOp::Copy.modifies_buffer());
        assert!(LineOp::Move.modifies_buffer());
        assert!(LineOp::Delete.modifies_buffer());
        assert!(!LineOp::Yank.modifies_buffer());
        assert!(LineOp::Put.modifies_buffer());
        assert!(LineOp::Join.modifies_buffer());

        assert!(LineOp::Yank.uses_register());
        assert!(LineOp::Put.uses_register());
        assert!(LineOp::Delete.uses_register());
        assert!(!LineOp::Copy.uses_register());

        assert!(LineOp::Copy.requires_destination());
        assert!(LineOp::Move.requires_destination());
        assert!(!LineOp::Delete.requires_destination());
    }

    #[test]
    fn test_copy_move_options() {
        let range = LineRange::new(5, 10);
        let opts = CopyMoveOptions::new(range, 20);
        assert_eq!(opts.range, range);
        assert_eq!(opts.dest, 20);
        assert_eq!(opts.line_count(), 6);

        // Valid for move (dest outside range)
        assert!(opts.is_valid_for_move());

        // Invalid for move (dest inside range)
        let opts = CopyMoveOptions::new(range, 7);
        assert!(!opts.is_valid_for_move());
    }

    #[test]
    fn test_delete_options() {
        let range = LineRange::new(5, 10);
        let opts = DeleteOptions::new(range);
        assert!(opts.use_register);
        assert_eq!(opts.register, 0);

        let opts = DeleteOptions::without_register(range);
        assert!(!opts.use_register);

        let opts = DeleteOptions::new(range).with_register(b'a');
        assert!(opts.use_register);
        assert_eq!(opts.register, b'a');
    }

    #[test]
    fn test_yank_options() {
        let range = LineRange::new(5, 10);
        let opts = YankOptions::new(range);
        assert_eq!(opts.register, 0);

        let opts = YankOptions::to_register(range, b'a');
        assert_eq!(opts.register, b'a');
    }

    #[test]
    fn test_put_options() {
        let opts = PutOptions::after(10);
        assert_eq!(opts.line, 10);
        assert!(!opts.before);

        let opts = PutOptions::before(10);
        assert!(opts.before);

        let opts = PutOptions::after(10).from_register(b'a');
        assert_eq!(opts.register, b'a');
    }

    #[test]
    fn test_join_options() {
        let range = LineRange::new(5, 10);
        let opts = JoinOptions::new(range);
        assert!(!opts.no_space);

        let opts = JoinOptions::without_space(range);
        assert!(opts.no_space);
    }

    #[test]
    fn test_line_op_result() {
        let result = LineOpResult::new(5, true);
        assert_eq!(result.lines, 5);
        assert!(result.changed);

        let result = LineOpResult::no_change();
        assert_eq!(result.lines, 0);
        assert!(!result.changed);
    }

    #[test]
    fn test_line_error_display() {
        let err = LineError::InvalidRange;
        assert_eq!(format!("{err}"), "invalid range");

        let err = LineError::InvalidDestination(150);
        assert_eq!(format!("{err}"), "invalid destination: 150");

        let err = LineError::MoveIntoSelf;
        assert!(format!("{err}").contains("same range"));

        let err = LineError::InvalidRegister(b'@');
        assert!(format!("{err}").contains("@"));
    }

    #[test]
    fn test_validate_copy_move() {
        // Valid copy
        let result = validate_copy_move(LineRange::new(5, 10), 20, 100, false);
        assert!(result.is_ok());

        // Valid copy to position 0 (before first line)
        let result = validate_copy_move(LineRange::new(5, 10), 0, 100, false);
        assert!(result.is_ok());

        // Invalid range
        let result = validate_copy_move(LineRange::empty(), 20, 100, false);
        assert!(matches!(result, Err(LineError::InvalidRange)));

        // Invalid destination
        let result = validate_copy_move(LineRange::new(5, 10), 150, 100, false);
        assert!(matches!(result, Err(LineError::InvalidDestination(150))));

        // Move into self
        let result = validate_copy_move(LineRange::new(5, 10), 7, 100, true);
        assert!(matches!(result, Err(LineError::MoveIntoSelf)));

        // Copy to same position is allowed
        let result = validate_copy_move(LineRange::new(5, 10), 7, 100, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_adjust_line_after_move() {
        // Move lines 5-10 to after line 20
        let range = LineRange::new(5, 10);
        let dest = 20;

        // Line within range - moves to new position
        assert_eq!(adjust_line_after_move(5, range, dest), 15);
        assert_eq!(adjust_line_after_move(10, range, dest), 20);

        // Line between source and dest - shifts up
        assert_eq!(adjust_line_after_move(15, range, dest), 9);

        // Line after dest - unchanged
        assert_eq!(adjust_line_after_move(25, range, dest), 25);

        // Line before source - unchanged
        assert_eq!(adjust_line_after_move(3, range, dest), 3);
    }

    #[test]
    fn test_adjust_line_after_move_up() {
        // Move lines 15-20 to after line 5
        let range = LineRange::new(15, 20);
        let dest = 5;

        // Line within range - moves to new position
        assert_eq!(adjust_line_after_move(15, range, dest), 6);
        assert_eq!(adjust_line_after_move(20, range, dest), 11);

        // Line between dest and source - shifts down
        assert_eq!(adjust_line_after_move(10, range, dest), 16);

        // Line before dest - unchanged
        assert_eq!(adjust_line_after_move(3, range, dest), 3);

        // Line after source - unchanged
        assert_eq!(adjust_line_after_move(25, range, dest), 25);
    }

    #[test]
    fn test_rs_validate_copy() {
        assert_eq!(rs_validate_copy(5, 10, 20, 100), 1);
        assert_eq!(rs_validate_copy(5, 10, 150, 100), 0);
    }

    #[test]
    fn test_rs_validate_move() {
        assert_eq!(rs_validate_move(5, 10, 20, 100), 1);
        assert_eq!(rs_validate_move(5, 10, 7, 100), 0); // Into self
    }

    #[test]
    fn test_rs_adjust_line_after_move() {
        // Move 5-10 to after 20
        assert_eq!(rs_adjust_line_after_move(5, 5, 10, 20), 15);
        assert_eq!(rs_adjust_line_after_move(15, 5, 10, 20), 9);
    }
}
