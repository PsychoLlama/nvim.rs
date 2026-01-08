//! Command table types and utilities for Ex commands.
//!
//! This module provides types and utilities for working with the command
//! table, including command flags and attribute checking.

use std::ffi::c_int;

// =============================================================================
// Command attribute flags (EX_*)
// =============================================================================

/// Allow a linespecs (range)
pub const EX_RANGE: u32 = 0x001;
/// Allow a ! after the command name
pub const EX_BANG: u32 = 0x002;
/// Allow extra args after command name
pub const EX_EXTRA: u32 = 0x004;
/// Expand wildcards in extra part
pub const EX_XFILE: u32 = 0x008;
/// No spaces allowed in the extra part
pub const EX_NOSPC: u32 = 0x010;
/// Default file range is 1,$
pub const EX_DFLALL: u32 = 0x020;
/// Extend range to include whole fold also
pub const EX_WHOLEFOLD: u32 = 0x040;
/// Argument required
pub const EX_NEEDARG: u32 = 0x080;
/// Check for trailing vertical bar
pub const EX_TRLBAR: u32 = 0x100;
/// Allow "x for register designation
pub const EX_REGSTR: u32 = 0x200;
/// Allow count in argument, after command
pub const EX_COUNT: u32 = 0x400;
/// No trailing comment allowed
pub const EX_NOTRLCOM: u32 = 0x800;
/// Zero line number allowed
pub const EX_ZEROR: u32 = 0x1000;
/// Do not remove CTRL-V from argument
pub const EX_CTRLV: u32 = 0x2000;
/// Allow "+command" argument
pub const EX_CMDARG: u32 = 0x4000;
/// Accepts buffer name
pub const EX_BUFNAME: u32 = 0x8000;
/// Accepts unlisted buffer too
pub const EX_BUFUNL: u32 = 0x10000;
/// Allow "++opt=val" argument
pub const EX_ARGOPT: u32 = 0x20000;
/// Allowed in the sandbox
pub const EX_SBOXOK: u32 = 0x40000;
/// Allowed in cmdline window
pub const EX_CMDWIN: u32 = 0x80000;
/// Forbidden in non-'modifiable' buffer
pub const EX_MODIFY: u32 = 0x100000;
/// Allow flags after count in argument
pub const EX_FLAGS: u32 = 0x200000;
/// Command can be executed when textlock is set
pub const EX_LOCK_OK: u32 = 0x1000000;
/// Keep sctx of where command was invoked
pub const EX_KEEPSCRIPT: u32 = 0x4000000;
/// Allow incremental command preview
pub const EX_PREVIEW: u32 = 0x8000000;

// Composite flags
/// Multiple extra files allowed
pub const EX_FILES: u32 = EX_XFILE | EX_EXTRA;
/// 1 file, defaults to current file
pub const EX_FILE1: u32 = EX_FILES | EX_NOSPC;
/// One extra word allowed
pub const EX_WORD1: u32 = EX_EXTRA | EX_NOSPC;

// =============================================================================
// Command flag checking utilities
// =============================================================================

/// Check if command allows a range.
#[inline]
pub const fn allows_range(argt: u32) -> bool {
    (argt & EX_RANGE) != 0
}

/// Check if command allows a bang (!).
#[inline]
pub const fn allows_bang(argt: u32) -> bool {
    (argt & EX_BANG) != 0
}

/// Check if command allows extra arguments.
#[inline]
pub const fn allows_extra(argt: u32) -> bool {
    (argt & EX_EXTRA) != 0
}

/// Check if command requires an argument.
#[inline]
pub const fn needs_arg(argt: u32) -> bool {
    (argt & EX_NEEDARG) != 0
}

/// Check if command allows a count.
#[inline]
pub const fn allows_count(argt: u32) -> bool {
    (argt & EX_COUNT) != 0
}

/// Check if command allows a register.
#[inline]
pub const fn allows_register(argt: u32) -> bool {
    (argt & EX_REGSTR) != 0
}

/// Check if command expands wildcards.
#[inline]
pub const fn expands_wildcards(argt: u32) -> bool {
    (argt & EX_XFILE) != 0
}

/// Check if command is allowed in sandbox.
#[inline]
pub const fn allowed_in_sandbox(argt: u32) -> bool {
    (argt & EX_SBOXOK) != 0
}

/// Check if command is allowed in cmdline window.
#[inline]
pub const fn allowed_in_cmdwin(argt: u32) -> bool {
    (argt & EX_CMDWIN) != 0
}

/// Check if command modifies the buffer.
#[inline]
pub const fn modifies_buffer(argt: u32) -> bool {
    (argt & EX_MODIFY) != 0
}

/// Check if command allows trailing bar.
#[inline]
pub const fn allows_trlbar(argt: u32) -> bool {
    (argt & EX_TRLBAR) != 0
}

/// Check if command allows ++opt arguments.
#[inline]
pub const fn allows_argopt(argt: u32) -> bool {
    (argt & EX_ARGOPT) != 0
}

/// Check if command allows +command argument.
#[inline]
pub const fn allows_cmdarg(argt: u32) -> bool {
    (argt & EX_CMDARG) != 0
}

/// Check if command accepts buffer name.
#[inline]
pub const fn accepts_bufname(argt: u32) -> bool {
    (argt & EX_BUFNAME) != 0
}

/// Check if command accepts unlisted buffer.
#[inline]
pub const fn accepts_bufunl(argt: u32) -> bool {
    (argt & EX_BUFUNL) != 0
}

/// Check if command allows flags after count.
#[inline]
pub const fn allows_flags(argt: u32) -> bool {
    (argt & EX_FLAGS) != 0
}

/// Check if command is allowed when text is locked.
#[inline]
pub const fn allowed_when_locked(argt: u32) -> bool {
    (argt & EX_LOCK_OK) != 0
}

/// Check if command supports preview.
#[inline]
pub const fn supports_preview(argt: u32) -> bool {
    (argt & EX_PREVIEW) != 0
}

// =============================================================================
// FFI wrappers
// =============================================================================

/// FFI wrapper for range check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_range(argt: u32) -> c_int {
    c_int::from(allows_range(argt))
}

/// FFI wrapper for bang check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_bang(argt: u32) -> c_int {
    c_int::from(allows_bang(argt))
}

/// FFI wrapper for extra args check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_extra(argt: u32) -> c_int {
    c_int::from(allows_extra(argt))
}

/// FFI wrapper for required argument check.
#[no_mangle]
pub extern "C" fn rs_cmd_needs_arg(argt: u32) -> c_int {
    c_int::from(needs_arg(argt))
}

/// FFI wrapper for count check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_count(argt: u32) -> c_int {
    c_int::from(allows_count(argt))
}

/// FFI wrapper for register check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_register(argt: u32) -> c_int {
    c_int::from(allows_register(argt))
}

/// FFI wrapper for sandbox check.
#[no_mangle]
pub extern "C" fn rs_cmd_allowed_in_sandbox(argt: u32) -> c_int {
    c_int::from(allowed_in_sandbox(argt))
}

/// FFI wrapper for cmdwin check.
#[no_mangle]
pub extern "C" fn rs_cmd_allowed_in_cmdwin(argt: u32) -> c_int {
    c_int::from(allowed_in_cmdwin(argt))
}

/// FFI wrapper for modify check.
#[no_mangle]
pub extern "C" fn rs_cmd_modifies_buffer(argt: u32) -> c_int {
    c_int::from(modifies_buffer(argt))
}

/// FFI wrapper for lock check.
#[no_mangle]
pub extern "C" fn rs_cmd_allowed_when_locked(argt: u32) -> c_int {
    c_int::from(allowed_when_locked(argt))
}

/// FFI wrapper for preview check.
#[no_mangle]
pub extern "C" fn rs_cmd_supports_preview(argt: u32) -> c_int {
    c_int::from(supports_preview(argt))
}

/// FFI wrapper for argopt check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_argopt(argt: u32) -> c_int {
    c_int::from(allows_argopt(argt))
}

/// FFI wrapper for cmdarg check.
#[no_mangle]
pub extern "C" fn rs_cmd_allows_cmdarg(argt: u32) -> c_int {
    c_int::from(allows_cmdarg(argt))
}

// =============================================================================
// Command completion types
// =============================================================================

/// Command completion types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionType {
    /// No completion
    None = 0,
    /// File completion
    File = 1,
    /// Directory completion
    Dir = 2,
    /// Buffer name completion
    Buffer = 3,
    /// Command name completion
    Command = 4,
    /// Tag completion
    Tag = 5,
    /// Option name completion
    Option = 6,
    /// Help tag completion
    Help = 7,
    /// Event name completion
    Event = 8,
    /// Syntax name completion
    Syntax = 9,
    /// Highlight group completion
    Highlight = 10,
    /// Colorscheme completion
    Colorscheme = 11,
    /// Environment variable completion
    Environment = 12,
    /// User defined completion
    User = 13,
    /// Mapping completion
    Mapping = 14,
    /// Menu completion
    Menu = 15,
    /// Expression completion
    Expression = 16,
    /// Shellcmd completion
    Shellcmd = 17,
    /// Sign completion
    Sign = 18,
    /// Filetype completion
    Filetype = 19,
    /// Locale completion
    Locale = 20,
}

impl CompletionType {
    /// Convert from C integer value.
    #[inline]
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::File),
            2 => Some(Self::Dir),
            3 => Some(Self::Buffer),
            4 => Some(Self::Command),
            5 => Some(Self::Tag),
            6 => Some(Self::Option),
            7 => Some(Self::Help),
            8 => Some(Self::Event),
            9 => Some(Self::Syntax),
            10 => Some(Self::Highlight),
            11 => Some(Self::Colorscheme),
            12 => Some(Self::Environment),
            13 => Some(Self::User),
            14 => Some(Self::Mapping),
            15 => Some(Self::Menu),
            16 => Some(Self::Expression),
            17 => Some(Self::Shellcmd),
            18 => Some(Self::Sign),
            19 => Some(Self::Filetype),
            20 => Some(Self::Locale),
            _ => Option::None,
        }
    }

    /// Convert to C integer value.
    #[inline]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// FFI wrapper for completion type conversion.
#[no_mangle]
pub extern "C" fn rs_completion_type_from_int(val: c_int) -> c_int {
    match CompletionType::from_c_int(val) {
        Some(t) => t.to_c_int(),
        Option::None => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_values() {
        assert_eq!(EX_RANGE, 0x001);
        assert_eq!(EX_BANG, 0x002);
        assert_eq!(EX_EXTRA, 0x004);
        assert_eq!(EX_XFILE, 0x008);
        assert_eq!(EX_NOSPC, 0x010);
        assert_eq!(EX_DFLALL, 0x020);
        assert_eq!(EX_WHOLEFOLD, 0x040);
        assert_eq!(EX_NEEDARG, 0x080);
        assert_eq!(EX_TRLBAR, 0x100);
        assert_eq!(EX_REGSTR, 0x200);
        assert_eq!(EX_COUNT, 0x400);
        assert_eq!(EX_SBOXOK, 0x40000);
        assert_eq!(EX_CMDWIN, 0x80000);
        assert_eq!(EX_MODIFY, 0x100000);
    }

    #[test]
    fn test_composite_flags() {
        assert_eq!(EX_FILES, EX_XFILE | EX_EXTRA);
        assert_eq!(EX_FILE1, EX_FILES | EX_NOSPC);
        assert_eq!(EX_WORD1, EX_EXTRA | EX_NOSPC);
    }

    #[test]
    fn test_flag_checks() {
        let argt = EX_RANGE | EX_BANG | EX_COUNT;

        assert!(allows_range(argt));
        assert!(allows_bang(argt));
        assert!(allows_count(argt));
        assert!(!allows_extra(argt));
        assert!(!needs_arg(argt));
        assert!(!allows_register(argt));
    }

    #[test]
    fn test_sandbox_check() {
        assert!(allowed_in_sandbox(EX_SBOXOK));
        assert!(!allowed_in_sandbox(EX_RANGE));
    }

    #[test]
    fn test_cmdwin_check() {
        assert!(allowed_in_cmdwin(EX_CMDWIN));
        assert!(!allowed_in_cmdwin(EX_RANGE));
    }

    #[test]
    fn test_modify_check() {
        assert!(modifies_buffer(EX_MODIFY));
        assert!(!modifies_buffer(EX_RANGE));
    }

    #[test]
    fn test_combined_flags() {
        // Typical command flags
        let write_argt = EX_RANGE | EX_BANG | EX_TRLBAR | EX_FILE1;
        assert!(allows_range(write_argt));
        assert!(allows_bang(write_argt));
        assert!(allows_trlbar(write_argt));
        assert!(allows_extra(write_argt));
        assert!(expands_wildcards(write_argt));
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_cmd_allows_range(EX_RANGE), 1);
        assert_eq!(rs_cmd_allows_range(0), 0);

        assert_eq!(rs_cmd_allows_bang(EX_BANG), 1);
        assert_eq!(rs_cmd_allows_bang(0), 0);

        assert_eq!(rs_cmd_allows_count(EX_COUNT), 1);
        assert_eq!(rs_cmd_allows_count(0), 0);

        assert_eq!(rs_cmd_allowed_in_sandbox(EX_SBOXOK), 1);
        assert_eq!(rs_cmd_allowed_in_sandbox(0), 0);
    }

    #[test]
    fn test_completion_type() {
        assert_eq!(CompletionType::from_c_int(0), Some(CompletionType::None));
        assert_eq!(CompletionType::from_c_int(1), Some(CompletionType::File));
        assert_eq!(CompletionType::from_c_int(3), Some(CompletionType::Buffer));
        assert_eq!(CompletionType::from_c_int(99), Option::None);

        assert_eq!(CompletionType::None.to_c_int(), 0);
        assert_eq!(CompletionType::File.to_c_int(), 1);
    }

    #[test]
    fn test_completion_type_ffi() {
        assert_eq!(rs_completion_type_from_int(0), 0);
        assert_eq!(rs_completion_type_from_int(1), 1);
        assert_eq!(rs_completion_type_from_int(99), -1);
    }
}
