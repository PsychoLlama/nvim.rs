//! Initialization sequence
//!
//! This module provides Rust implementations for Neovim's
//! initialization sequence (event_init, event_teardown, early_init, etc.).

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// FFI declarations for event_init / event_teardown
// =============================================================================

unsafe extern "C" {
    // Loop operations (take opaque *mut c_void = *mut Loop)
    fn loop_init(loop_: *mut c_void, data: *mut c_void);
    fn loop_close(loop_: *mut c_void, wait: bool) -> bool;
    fn loop_poll_events(loop_: *mut c_void, ms: i64) -> bool;

    // MultiQueue operations
    fn multiqueue_new_child(parent: *mut c_void) -> *mut c_void;
    fn multiqueue_process_events(mq: *mut c_void);

    // rs_loop_get_events is the Rust function exposed as loop_get_events
    fn rs_loop_get_events(loop_: *mut c_void) -> *mut c_void;

    // Subsystem init/teardown
    fn signal_init();
    fn channel_init();
    fn terminal_init();
    fn ui_init();
    fn input_stop();
    fn channel_teardown();
    fn proc_teardown(loop_: *mut c_void);
    fn timer_teardown();
    fn server_teardown();
    fn signal_teardown();
    fn terminal_teardown();

    // C accessors for main_loop and resize_events globals
    fn nvim_get_main_loop() -> *mut c_void;
    fn nvim_set_resize_events(mq: *mut c_void);
    fn nvim_time_msg(msg: *const c_char);
}

// =============================================================================
// Phase 1: event_init and event_teardown
// =============================================================================

/// Initialize the event loop and subsystems.
///
/// # Safety
/// Must be called exactly once at startup from the main thread.
#[unsafe(export_name = "event_init")]
pub unsafe extern "C" fn rs_event_init() {
    let main_loop = nvim_get_main_loop();
    loop_init(main_loop, std::ptr::null_mut());

    let events = rs_loop_get_events(main_loop);
    let resize_mq = multiqueue_new_child(events);
    nvim_set_resize_events(resize_mq);

    signal_init();
    // msgpack-rpc initialization
    channel_init();
    terminal_init();
    ui_init();
    nvim_time_msg(c"event init".as_ptr());
}

/// Drain pending events and tear down subsystems.
///
/// Returns false if main_loop could not be closed gracefully.
///
/// # Safety
/// Must be called from the main thread during shutdown.
#[unsafe(export_name = "event_teardown")]
pub unsafe extern "C" fn rs_event_teardown() -> bool {
    let main_loop = nvim_get_main_loop();
    let events = rs_loop_get_events(main_loop);

    if events.is_null() {
        input_stop();
        return true;
    }

    multiqueue_process_events(events);
    loop_poll_events(main_loop, 0); // Drain thread_events, fast_events.
    input_stop();
    channel_teardown();
    proc_teardown(main_loop);
    timer_teardown();
    server_teardown();
    signal_teardown();
    terminal_teardown();

    loop_close(main_loop, true)
}

// =============================================================================
// Phase 2: early_init
// =============================================================================

unsafe extern "C" {
    // early_init subsystem initializers
    fn os_hint_priority();
    fn estack_init();
    fn cmdline_init();
    fn eval_init();
    fn rs_init_path(exename: *const c_char);
    fn init_normal_cmds();
    fn runtime_init();
    fn highlight_init();
    fn init_locale();
    fn set_init_tablocal();
    fn win_alloc_first();
    fn alist_init(al: *mut c_void);
    fn init_homedir();
    fn set_init_1(clean: bool);
    fn log_init();
    fn set_lang_var();
    fn qf_init_stack();

    // C accessors for global_alist and mparm_T
    fn nvim_get_global_alist_ptr() -> *mut c_void;
    fn nvim_set_global_alist_id(id: c_int);
    fn nvim_paramp_get_clean(paramp: *mut c_void) -> bool;

    // argv0 global (the program name, from main.c)
    static nvim_argv0: *mut c_char;
}

/// Perform early initialization.
///
/// Sets up the eval system, option defaults, locale, and first window/tab.
/// Needed for unit tests.
///
/// # Safety
/// Must be called from main thread during startup. Initializers must run in order.
#[unsafe(export_name = "early_init")]
pub unsafe extern "C" fn rs_early_init(paramp: *mut c_void) {
    os_hint_priority();
    estack_init();
    cmdline_init();
    eval_init(); // init global variables

    let exename: *const c_char = if nvim_argv0.is_null() {
        c"nvim".as_ptr()
    } else {
        nvim_argv0
    };
    rs_init_path(exename);

    init_normal_cmds(); // Init the table of Normal mode commands.
    runtime_init();
    highlight_init();

    // MSWIN: GetVersionEx / windowsVersion block omitted on non-Windows.
    // On Windows, this would set the `windowsVersion` global.
    // That block is left as a C-only concern if MSWIN is ever targeted.

    nvim_time_msg(c"early init".as_ptr());

    // Setup to use the current locale (for ctype() and many other things).
    // NOTE: Translated messages with encodings other than latin1 will not
    // work until set_init_1() has been called!
    init_locale();

    // tabpage local options (p_ch) must be set before allocating first tabpage.
    set_init_tablocal();

    // Allocate the first tabpage, window and buffer.
    win_alloc_first();
    nvim_time_msg(c"init first window".as_ptr());

    let global_alist = nvim_get_global_alist_ptr();
    alist_init(global_alist); // Init the argument list to empty.
    nvim_set_global_alist_id(0);

    // Set the default values for the options.
    // First find out the home directory, needed to expand "~" in options.
    init_homedir(); // find real value of $HOME

    let clean = if paramp.is_null() {
        false
    } else {
        nvim_paramp_get_clean(paramp)
    };
    set_init_1(clean);
    log_init();
    nvim_time_msg(c"inits 1".as_ptr());

    set_lang_var(); // set v:lang and v:ctype

    // initialize quickfix list
    qf_init_stack();
}

// =============================================================================
// Init Flags (kept for API compatibility)
// =============================================================================

/// Initialization flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InitFlags {
    /// No swap file
    pub no_swap: bool,
    /// No plugins
    pub no_plugins: bool,
    /// No user config
    pub no_config: bool,
    /// No site config
    pub no_site: bool,
    /// Clean mode (implies no_plugins, no_config)
    pub clean: bool,
    /// Headless mode
    pub headless: bool,
    /// Embed mode
    pub embed: bool,
    /// Listen mode
    pub listen: bool,
}

impl InitFlags {
    /// Create new init flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            no_swap: false,
            no_plugins: false,
            no_config: false,
            no_site: false,
            clean: false,
            headless: false,
            embed: false,
            listen: false,
        }
    }

    /// Check if any config is disabled.
    #[must_use]
    pub const fn is_config_disabled(&self) -> bool {
        self.clean || self.no_config
    }

    /// Check if plugins are disabled.
    #[must_use]
    pub const fn is_plugins_disabled(&self) -> bool {
        self.clean || self.no_plugins
    }

    /// Check if running in non-interactive mode.
    #[must_use]
    pub const fn is_non_interactive(&self) -> bool {
        self.headless || self.embed
    }
}

// =============================================================================
// Init Step Result
// =============================================================================

/// Result of an initialization step.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitResult {
    /// Step succeeded
    Ok = 0,
    /// Step failed (non-fatal)
    Warning = 1,
    /// Step failed (fatal)
    Error = 2,
    /// Step skipped
    Skipped = 3,
}

impl InitResult {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Warning,
            2 => Self::Error,
            3 => Self::Skipped,
            _ => Self::Ok,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result is successful.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok | Self::Warning | Self::Skipped)
    }

    /// Check if result is fatal.
    #[must_use]
    pub const fn is_fatal(self) -> bool {
        matches!(self, Self::Error)
    }
}

// =============================================================================
// Init Order
// =============================================================================

/// Initialization order/priority.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitOrder {
    /// Core runtime (memory, signals)
    #[default]
    Core = 0,
    /// Event loop
    Events = 1,
    /// Message handling
    Messages = 2,
    /// Options
    Options = 3,
    /// Mappings
    Mappings = 4,
    /// Autocmds
    Autocmds = 5,
    /// Plugins
    Plugins = 6,
    /// User scripts
    UserScripts = 7,
}

impl InitOrder {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Events,
            2 => Self::Messages,
            3 => Self::Options,
            4 => Self::Mappings,
            5 => Self::Autocmds,
            6 => Self::Plugins,
            7 => Self::UserScripts,
            _ => Self::Core,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_flags() {
        let flags = InitFlags::new();
        assert!(!flags.is_config_disabled());
        assert!(!flags.is_plugins_disabled());

        let mut flags = flags;
        flags.clean = true;
        assert!(flags.is_config_disabled());
        assert!(flags.is_plugins_disabled());
    }

    #[test]
    fn test_init_result() {
        assert!(InitResult::Ok.is_ok());
        assert!(InitResult::Warning.is_ok());
        assert!(InitResult::Skipped.is_ok());
        assert!(!InitResult::Error.is_ok());
        assert!(InitResult::Error.is_fatal());
    }

    #[test]
    fn test_init_order() {
        assert_eq!(InitOrder::from_c_int(0), InitOrder::Core);
        assert_eq!(InitOrder::from_c_int(6), InitOrder::Plugins);
    }
}
