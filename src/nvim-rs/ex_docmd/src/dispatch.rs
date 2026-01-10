//! Ex command dispatch and execution infrastructure.
//!
//! This module provides utilities for the Ex command dispatch system:
//! - Command execution context management
//! - Modifier handling (silent, verbose, etc.)
//! - Command line continuation
//! - Error handling during command execution

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Command execution succeeded.
pub const EX_OK: c_int = 0;

/// Command execution failed.
pub const EX_FAIL: c_int = 1;

/// Command was interrupted.
pub const EX_INTERRUPT: c_int = 2;

/// Command needs more input (continuation).
pub const EX_CONTINUE: c_int = 3;

/// Command execution flags.
pub mod flags {
    use std::ffi::c_int;

    /// Execute silently (don't echo command).
    pub const DOCMD_VERBOSE: c_int = 0x01;

    /// Execute normally (main loop).
    pub const DOCMD_NOWAIT: c_int = 0x02;

    /// Repeat command until count exhausted.
    pub const DOCMD_REPEAT: c_int = 0x04;

    /// Keep state for line continuation.
    pub const DOCMD_KEEPLINE: c_int = 0x08;

    /// Expression evaluation mode.
    pub const DOCMD_EXMODE: c_int = 0x10;

    /// Don't display errors.
    pub const DOCMD_NOERROR: c_int = 0x20;

    /// Keytyped was already reset.
    pub const DOCMD_KEYTYPED: c_int = 0x40;

    /// Mark for redo after Ctrl-O.
    pub const DOCMD_RANGEOK: c_int = 0x80;
}

// =============================================================================
// Execution Context
// =============================================================================

/// Context for command execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecContext {
    /// Execution flags (DOCMD_*).
    pub flags: c_int,
    /// Recursion depth for nested commands.
    pub depth: c_int,
    /// Source command line (for error messages).
    pub source_line: c_int,
    /// Whether we're in verbose mode.
    pub verbose: bool,
    /// Whether we're in silent mode.
    pub silent: bool,
    /// Whether to wait for return after messages.
    pub wait_return: bool,
    /// Whether command interrupted the screen.
    pub did_modify_screen: bool,
}

impl ExecContext {
    /// Create a new execution context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            flags: 0,
            depth: 0,
            source_line: 0,
            verbose: false,
            silent: false,
            wait_return: true,
            did_modify_screen: false,
        }
    }

    /// Create a context for nested command execution.
    #[must_use]
    pub fn nested(&self) -> Self {
        Self {
            flags: self.flags,
            depth: self.depth + 1,
            source_line: 0,
            verbose: self.verbose,
            silent: self.silent,
            wait_return: self.wait_return,
            did_modify_screen: false,
        }
    }

    /// Check if verbose mode is active.
    #[must_use]
    pub const fn is_verbose(&self) -> bool {
        self.verbose || (self.flags & flags::DOCMD_VERBOSE != 0)
    }

    /// Check if silent mode is active.
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        self.silent
    }

    /// Check if we're at the top level.
    #[must_use]
    pub const fn is_toplevel(&self) -> bool {
        self.depth == 0
    }

    /// Set silent mode.
    pub fn set_silent(&mut self, silent: bool) {
        self.silent = silent;
    }

    /// Set verbose mode.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Mark that screen was modified.
    pub fn mark_screen_modified(&mut self) {
        self.did_modify_screen = true;
    }
}

// =============================================================================
// Command Modifiers
// =============================================================================

/// Ex command modifiers that affect execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdModifiers {
    /// :silent modifier was used.
    pub silent: bool,
    /// :silent! modifier was used (ignore errors).
    pub silent_error: bool,
    /// :verbose modifier was used (with count).
    pub verbose: c_int,
    /// :vertical modifier was used.
    pub vertical: bool,
    /// :horizontal modifier was used.
    pub horizontal: bool,
    /// :tab modifier was used.
    pub tab: c_int,
    /// :aboveleft or :leftabove modifier.
    pub aboveleft: bool,
    /// :belowright or :rightbelow modifier.
    pub belowright: bool,
    /// :topleft modifier.
    pub topleft: bool,
    /// :botright modifier.
    pub botright: bool,
    /// :browse modifier.
    pub browse: bool,
    /// :confirm modifier.
    pub confirm: bool,
    /// :hide modifier.
    pub hide: bool,
    /// :keepalt modifier.
    pub keepalt: bool,
    /// :keepjumps modifier.
    pub keepjumps: bool,
    /// :keepmarks modifier.
    pub keepmarks: bool,
    /// :keeppatterns modifier.
    pub keeppatterns: bool,
    /// :lockmarks modifier.
    pub lockmarks: bool,
    /// :noswapfile modifier.
    pub noswapfile: bool,
    /// :noautocmd modifier.
    pub noautocmd: bool,
    /// :sandbox modifier.
    pub sandbox: bool,
    /// :unsilent modifier.
    pub unsilent: bool,
}

impl CmdModifiers {
    /// Create new empty modifiers.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            silent: false,
            silent_error: false,
            verbose: 0,
            vertical: false,
            horizontal: false,
            tab: 0,
            aboveleft: false,
            belowright: false,
            topleft: false,
            botright: false,
            browse: false,
            confirm: false,
            hide: false,
            keepalt: false,
            keepjumps: false,
            keepmarks: false,
            keeppatterns: false,
            lockmarks: false,
            noswapfile: false,
            noautocmd: false,
            sandbox: false,
            unsilent: false,
        }
    }

    /// Check if any split direction modifier is set.
    #[must_use]
    pub const fn has_split_direction(&self) -> bool {
        self.aboveleft || self.belowright || self.topleft || self.botright
    }

    /// Check if any window modifier is set.
    #[must_use]
    pub const fn has_window_modifier(&self) -> bool {
        self.vertical || self.horizontal || self.tab > 0 || self.has_split_direction()
    }

    /// Clear all split direction modifiers.
    pub fn clear_split_direction(&mut self) {
        self.aboveleft = false;
        self.belowright = false;
        self.topleft = false;
        self.botright = false;
    }

    /// Clear all modifiers.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Check if we're in silent mode (either form).
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        self.silent || self.silent_error
    }
}

// =============================================================================
// Execution Result
// =============================================================================

/// Result of executing an Ex command.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExecResult {
    /// Status code (EX_OK, EX_FAIL, etc.).
    pub status: c_int,
    /// Whether to continue with next command.
    pub continue_exec: bool,
    /// Whether to break out of loop (for :break).
    pub do_break: bool,
    /// Whether to continue loop (for :continue).
    pub do_continue: bool,
    /// Whether to return from function (for :return).
    pub do_return: bool,
    /// Whether to throw exception (for :throw).
    pub do_throw: bool,
    /// Return value (for :return with value).
    pub return_value: c_int,
}

impl Default for ExecResult {
    fn default() -> Self {
        Self::ok()
    }
}

impl ExecResult {
    /// Create a successful result.
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            status: EX_OK,
            continue_exec: true,
            do_break: false,
            do_continue: false,
            do_return: false,
            do_throw: false,
            return_value: 0,
        }
    }

    /// Create a failure result.
    #[must_use]
    pub const fn fail() -> Self {
        Self {
            status: EX_FAIL,
            continue_exec: false,
            do_break: false,
            do_continue: false,
            do_return: false,
            do_throw: false,
            return_value: 0,
        }
    }

    /// Create an interrupted result.
    #[must_use]
    pub const fn interrupted() -> Self {
        Self {
            status: EX_INTERRUPT,
            continue_exec: false,
            do_break: false,
            do_continue: false,
            do_return: false,
            do_throw: false,
            return_value: 0,
        }
    }

    /// Create a break result (for :break).
    #[must_use]
    pub const fn do_break() -> Self {
        Self {
            status: EX_OK,
            continue_exec: false,
            do_break: true,
            do_continue: false,
            do_return: false,
            do_throw: false,
            return_value: 0,
        }
    }

    /// Create a continue result (for :continue).
    #[must_use]
    pub const fn do_continue() -> Self {
        Self {
            status: EX_OK,
            continue_exec: false,
            do_break: false,
            do_continue: true,
            do_return: false,
            do_throw: false,
            return_value: 0,
        }
    }

    /// Create a return result (for :return).
    #[must_use]
    pub const fn do_return(value: c_int) -> Self {
        Self {
            status: EX_OK,
            continue_exec: false,
            do_break: false,
            do_continue: false,
            do_return: true,
            do_throw: false,
            return_value: value,
        }
    }

    /// Check if execution succeeded.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.status == EX_OK
    }

    /// Check if we should stop execution.
    #[must_use]
    pub const fn should_stop(&self) -> bool {
        !self.continue_exec
    }

    /// Check if flow control statement was executed.
    #[must_use]
    pub const fn is_flow_control(&self) -> bool {
        self.do_break || self.do_continue || self.do_return || self.do_throw
    }
}

// =============================================================================
// Command Line State
// =============================================================================

/// State for line-continuation during command execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ContinuationState {
    /// Whether continuation is expected.
    pub expect_continuation: bool,
    /// Whether in here-document mode.
    pub in_heredoc: bool,
    /// Heredoc end marker.
    pub heredoc_trimmed: bool,
    /// Current nesting level (for control structures).
    pub nesting_level: c_int,
    /// Number of open parentheses.
    pub open_parens: c_int,
    /// Number of open brackets.
    pub open_brackets: c_int,
    /// Number of open braces.
    pub open_braces: c_int,
}

impl ContinuationState {
    /// Create a new continuation state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            expect_continuation: false,
            in_heredoc: false,
            heredoc_trimmed: false,
            nesting_level: 0,
            open_parens: 0,
            open_brackets: 0,
            open_braces: 0,
        }
    }

    /// Check if continuation is needed.
    #[must_use]
    pub const fn needs_continuation(&self) -> bool {
        self.expect_continuation
            || self.in_heredoc
            || self.open_parens > 0
            || self.open_brackets > 0
            || self.open_braces > 0
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Update based on a character.
    pub fn update_char(&mut self, c: u8) {
        match c {
            b'(' => self.open_parens += 1,
            b')' => {
                if self.open_parens > 0 {
                    self.open_parens -= 1;
                }
            }
            b'[' => self.open_brackets += 1,
            b']' => {
                if self.open_brackets > 0 {
                    self.open_brackets -= 1;
                }
            }
            b'{' => self.open_braces += 1,
            b'}' => {
                if self.open_braces > 0 {
                    self.open_braces -= 1;
                }
            }
            _ => {}
        }
    }

    /// Set continuation expected (backslash at end of line).
    pub fn set_continuation(&mut self, expected: bool) {
        self.expect_continuation = expected;
    }

    /// Enter heredoc mode.
    pub fn enter_heredoc(&mut self, trimmed: bool) {
        self.in_heredoc = true;
        self.heredoc_trimmed = trimmed;
    }

    /// Exit heredoc mode.
    pub fn exit_heredoc(&mut self) {
        self.in_heredoc = false;
        self.heredoc_trimmed = false;
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new execution context.
#[no_mangle]
pub extern "C" fn rs_exec_context_new() -> ExecContext {
    ExecContext::new()
}

/// Create a nested execution context.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_context_nested(ctx: *const ExecContext) -> ExecContext {
    if ctx.is_null() {
        ExecContext::new()
    } else {
        (*ctx).nested()
    }
}

/// Check if context is verbose.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_context_is_verbose(ctx: *const ExecContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).is_verbose())
}

/// Check if context is silent.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_context_is_silent(ctx: *const ExecContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).is_silent())
}

/// Check if context is at top level.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_context_is_toplevel(ctx: *const ExecContext) -> c_int {
    if ctx.is_null() {
        return 1;
    }
    c_int::from((*ctx).is_toplevel())
}

/// Create new command modifiers.
#[no_mangle]
pub extern "C" fn rs_cmd_modifiers_new() -> CmdModifiers {
    CmdModifiers::new()
}

/// Check if modifiers have window modifier.
///
/// # Safety
/// `mods` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_modifiers_has_window(mods: *const CmdModifiers) -> c_int {
    if mods.is_null() {
        return 0;
    }
    c_int::from((*mods).has_window_modifier())
}

/// Check if modifiers are silent.
///
/// # Safety
/// `mods` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_modifiers_is_silent(mods: *const CmdModifiers) -> c_int {
    if mods.is_null() {
        return 0;
    }
    c_int::from((*mods).is_silent())
}

/// Clear command modifiers.
///
/// # Safety
/// `mods` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_modifiers_clear(mods: *mut CmdModifiers) {
    if !mods.is_null() {
        (*mods).clear();
    }
}

/// Create successful execution result.
#[no_mangle]
pub extern "C" fn rs_exec_result_ok() -> ExecResult {
    ExecResult::ok()
}

/// Create failure execution result.
#[no_mangle]
pub extern "C" fn rs_exec_result_fail() -> ExecResult {
    ExecResult::fail()
}

/// Create interrupted execution result.
#[no_mangle]
pub extern "C" fn rs_exec_result_interrupted() -> ExecResult {
    ExecResult::interrupted()
}

/// Create break execution result.
#[no_mangle]
pub extern "C" fn rs_exec_result_break() -> ExecResult {
    ExecResult::do_break()
}

/// Create continue execution result.
#[no_mangle]
pub extern "C" fn rs_exec_result_continue() -> ExecResult {
    ExecResult::do_continue()
}

/// Create return execution result.
#[no_mangle]
pub extern "C" fn rs_exec_result_return(value: c_int) -> ExecResult {
    ExecResult::do_return(value)
}

/// Check if execution result is ok.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_result_is_ok(result: *const ExecResult) -> c_int {
    if result.is_null() {
        return 0;
    }
    c_int::from((*result).is_ok())
}

/// Check if execution should stop.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_result_should_stop(result: *const ExecResult) -> c_int {
    if result.is_null() {
        return 1;
    }
    c_int::from((*result).should_stop())
}

/// Create new continuation state.
#[no_mangle]
pub extern "C" fn rs_continuation_state_new() -> ContinuationState {
    ContinuationState::new()
}

/// Check if continuation is needed.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_continuation_state_needs(state: *const ContinuationState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).needs_continuation())
}

/// Reset continuation state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_continuation_state_reset(state: *mut ContinuationState) {
    if !state.is_null() {
        (*state).reset();
    }
}

/// Update continuation state with a character.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub unsafe extern "C" fn rs_continuation_state_update(state: *mut ContinuationState, c: c_int) {
    if !state.is_null() && c >= 0 && c <= 127 {
        (*state).update_char(c as u8);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_context() {
        let ctx = ExecContext::new();
        assert!(ctx.is_toplevel());
        assert!(!ctx.is_silent());
        assert!(!ctx.is_verbose());

        let nested = ctx.nested();
        assert!(!nested.is_toplevel());
        assert_eq!(nested.depth, 1);

        let mut ctx = ExecContext::new();
        ctx.set_silent(true);
        assert!(ctx.is_silent());

        ctx.set_verbose(true);
        assert!(ctx.is_verbose());
    }

    #[test]
    fn test_cmd_modifiers() {
        let mods = CmdModifiers::new();
        assert!(!mods.is_silent());
        assert!(!mods.has_window_modifier());
        assert!(!mods.has_split_direction());

        let mut mods = CmdModifiers::new();
        mods.silent = true;
        assert!(mods.is_silent());

        mods.silent = false;
        mods.silent_error = true;
        assert!(mods.is_silent());

        mods.vertical = true;
        assert!(mods.has_window_modifier());

        mods.vertical = false;
        mods.aboveleft = true;
        assert!(mods.has_split_direction());
        assert!(mods.has_window_modifier());

        mods.clear();
        assert!(!mods.has_window_modifier());
    }

    #[test]
    fn test_exec_result() {
        let result = ExecResult::ok();
        assert!(result.is_ok());
        assert!(!result.should_stop());
        assert!(!result.is_flow_control());

        let result = ExecResult::fail();
        assert!(!result.is_ok());
        assert!(result.should_stop());

        let result = ExecResult::do_break();
        assert!(result.is_ok());
        assert!(result.should_stop());
        assert!(result.is_flow_control());
        assert!(result.do_break);

        let result = ExecResult::do_return(42);
        assert!(result.is_ok());
        assert!(result.is_flow_control());
        assert!(result.do_return);
        assert_eq!(result.return_value, 42);
    }

    #[test]
    fn test_continuation_state() {
        let state = ContinuationState::new();
        assert!(!state.needs_continuation());

        let mut state = ContinuationState::new();
        state.update_char(b'(');
        assert!(state.needs_continuation());
        assert_eq!(state.open_parens, 1);

        state.update_char(b')');
        assert!(!state.needs_continuation());
        assert_eq!(state.open_parens, 0);

        state.set_continuation(true);
        assert!(state.needs_continuation());

        state.reset();
        assert!(!state.needs_continuation());

        state.enter_heredoc(true);
        assert!(state.needs_continuation());
        assert!(state.in_heredoc);
        assert!(state.heredoc_trimmed);

        state.exit_heredoc();
        assert!(!state.needs_continuation());
    }
}
