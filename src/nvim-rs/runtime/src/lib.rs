//! Runtime file management for Neovim
//!
//! This crate provides Rust implementations of runtime file management functions
//! from `src/nvim/runtime.c`. It handles:
//!
//! - Script sourcing and execution stack management
//! - Runtime path searching
//! - Package management (plugin loading)
//! - Default file loading
//!
//! # Architecture
//!
//! The runtime system manages script execution through:
//! - An execution stack tracking the source of current code
//! - Script item tracking for sourced files
//! - Path searching through 'runtimepath' and 'packpath'
//!
//! # Modules
//!
//! - `stack` - Execution stack management
//! - `search` - Runtime path searching
//! - `path` - Path manipulation and expansion
//! - `source` - Script sourcing operations
//! - `package` - Package/plugin management
//! - `commands` - Ex command handlers
//! - `script` - Script item management
//! - `defaults` - Default file loading

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_sign_loss)] // C FFI requires these casts
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_safety_doc)] // Many FFI functions are unsafe by nature
#![allow(clippy::option_if_let_else)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Submodules
// =============================================================================

pub mod commands;
pub mod defaults;
pub mod package;
pub mod path;
pub mod pathsearch;
pub mod script;
pub mod search;
pub mod searchpath;
pub mod source;
pub mod stack;

// =============================================================================
// Constants
// =============================================================================

/// do_source() flags
pub mod doso {
    use std::ffi::c_int;

    /// No special flags
    pub const NONE: c_int = 0;
    /// Loading vimrc file
    pub const VIMRC: c_int = 1;
}

/// do_in_path() flags
pub mod dip {
    use std::ffi::c_int;

    /// All matches, not just the first one
    pub const ALL: c_int = 0x01;
    /// Find directories instead of files
    pub const DIR: c_int = 0x02;
    /// Give an error message when none found
    pub const ERR: c_int = 0x04;
    /// Also use "start" directory in 'packpath'
    pub const START: c_int = 0x08;
    /// Also use "opt" directory in 'packpath'
    pub const OPT: c_int = 0x10;
    /// Do not use 'runtimepath'
    pub const NORTP: c_int = 0x20;
    /// Skip "after" directories
    pub const NOAFTER: c_int = 0x40;
    /// Only use "after" directories
    pub const AFTER: c_int = 0x80;
    /// Find both files and directories
    pub const DIRFILE: c_int = 0x200;
}

// =============================================================================
// Execution Stack Entry Types
// =============================================================================

/// Execution stack entry type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtypeT {
    /// Toplevel
    Top = 0,
    /// Sourcing script
    Script = 1,
    /// User function
    Ufunc = 2,
    /// Autocommand
    Aucmd = 3,
    /// Modeline
    Modeline = 4,
    /// Exception
    Except = 5,
    /// Command line argument
    Args = 6,
    /// Environment variable
    Env = 7,
    /// Internal operation
    Internal = 8,
    /// Loading spell file
    Spell = 9,
}

impl EtypeT {
    /// Convert from integer, returning None if invalid
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Top),
            1 => Some(Self::Script),
            2 => Some(Self::Ufunc),
            3 => Some(Self::Aucmd),
            4 => Some(Self::Modeline),
            5 => Some(Self::Except),
            6 => Some(Self::Args),
            7 => Some(Self::Env),
            8 => Some(Self::Internal),
            9 => Some(Self::Spell),
            _ => None,
        }
    }

    /// Check if this entry type has a name that should be shown
    pub const fn has_name(&self) -> bool {
        matches!(
            self,
            Self::Script | Self::Ufunc | Self::Aucmd | Self::Modeline | Self::Args | Self::Env
        )
    }
}

/// Argument for estack_sfile()
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstackArgT {
    /// No argument
    None = 0,
    /// For <sfile>
    Sfile = 1,
    /// For <stack>
    Stack = 2,
    /// For <script>
    Script = 3,
}

impl EstackArgT {
    /// Convert from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::Sfile),
            2 => Some(Self::Stack),
            3 => Some(Self::Script),
            _ => None,
        }
    }
}

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to execution stack entry (estack_T)
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct EstackHandle(*mut c_void);

impl EstackHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to script item (scriptitem_T)
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ScriptItemHandle(*mut c_void);

impl ScriptItemHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to source cookie (source_cookie_T)
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SourceCookieHandle(*mut c_void);

impl SourceCookieHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Line number type (matches linenr_T in Neovim)
pub type LinenrT = i32;

/// Script ID type
pub type ScidT = c_int;

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Execution stack accessors
    fn nvim_get_exestack_len() -> c_int;
    fn nvim_exestack_has_data() -> bool;

    // Script items accessors
    fn nvim_script_items_get_len() -> c_int;
    fn nvim_script_item_get(id: ScidT) -> ScriptItemHandle;
    fn nvim_scriptitem_get_name(si: ScriptItemHandle) -> *const c_char;
    fn nvim_scriptitem_is_lua(si: ScriptItemHandle) -> bool;

    // Global state
    fn nvim_get_current_sctx_sid() -> ScidT;
    fn nvim_get_sourcing_name() -> *const c_char;
    fn nvim_get_sourcing_lnum() -> LinenrT;
}

// =============================================================================
// Execution Stack Functions
// =============================================================================

// NOTE: rs_have_sourcing_info, rs_get_sourcing_name, rs_get_sourcing_lnum,
// rs_exestack_len, rs_exestack_empty, rs_script_id_valid, rs_get_current_script_id
// are defined in other crates (ex_docmd, etc.) and exported from there.
// They are NOT re-exported here to avoid duplicate symbol errors.

// =============================================================================
// Path Flag Helpers
// =============================================================================

/// Check if DIP_ALL flag is set
pub fn rs_dip_has_all(flags: c_int) -> bool {
    (flags & dip::ALL) != 0
}

/// Check if DIP_DIR flag is set
pub fn rs_dip_has_dir(flags: c_int) -> bool {
    (flags & dip::DIR) != 0
}

/// Check if DIP_ERR flag is set
pub fn rs_dip_has_err(flags: c_int) -> bool {
    (flags & dip::ERR) != 0
}

/// Check if DIP_START flag is set
pub fn rs_dip_has_start(flags: c_int) -> bool {
    (flags & dip::START) != 0
}

/// Check if DIP_OPT flag is set
pub fn rs_dip_has_opt(flags: c_int) -> bool {
    (flags & dip::OPT) != 0
}

/// Check if DIP_NORTP flag is set
pub fn rs_dip_has_nortp(flags: c_int) -> bool {
    (flags & dip::NORTP) != 0
}

/// Check if DIP_NOAFTER flag is set
pub fn rs_dip_has_noafter(flags: c_int) -> bool {
    (flags & dip::NOAFTER) != 0
}

/// Check if DIP_AFTER flag is set
pub fn rs_dip_has_after(flags: c_int) -> bool {
    (flags & dip::AFTER) != 0
}

/// Check if DIP_DIRFILE flag is set
pub fn rs_dip_has_dirfile(flags: c_int) -> bool {
    (flags & dip::DIRFILE) != 0
}

/// Check if searching for packages (START or OPT set)
pub fn rs_dip_is_package_search(flags: c_int) -> bool {
    (flags & (dip::START | dip::OPT)) != 0
}

// =============================================================================
// Re-exports from submodules
// =============================================================================

pub use commands::{rs_ex_runtime, rs_get_runtime_cmd_flags, rs_set_context_in_runtime_cmd};
pub use package::{
    rs_add_pack_start_dirs, rs_ex_packadd, rs_ex_packloadall, rs_load_pack_plugin, rs_load_plugins,
    rs_load_start_packages, rs_pack_has_entries,
};
pub use pathsearch::{
    rs_do_in_path_and_pp, rs_do_in_runtimepath, rs_gen_expand_wildcards_and_cb, rs_source_callback,
    rs_source_callback_vim_lua, rs_source_in_path_vim_lua, rs_source_runtime,
    rs_source_runtime_vim_lua,
};

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etype_from_int() {
        assert_eq!(EtypeT::from_int(0), Some(EtypeT::Top));
        assert_eq!(EtypeT::from_int(1), Some(EtypeT::Script));
        assert_eq!(EtypeT::from_int(9), Some(EtypeT::Spell));
        assert_eq!(EtypeT::from_int(10), None);
        assert_eq!(EtypeT::from_int(-1), None);
    }

    #[test]
    fn test_etype_has_name() {
        assert!(!EtypeT::Top.has_name());
        assert!(EtypeT::Script.has_name());
        assert!(EtypeT::Ufunc.has_name());
        assert!(EtypeT::Aucmd.has_name());
        assert!(EtypeT::Modeline.has_name());
        assert!(!EtypeT::Except.has_name());
        assert!(EtypeT::Args.has_name());
        assert!(EtypeT::Env.has_name());
        assert!(!EtypeT::Internal.has_name());
        assert!(!EtypeT::Spell.has_name());
    }

    #[test]
    fn test_estack_arg_from_int() {
        assert_eq!(EstackArgT::from_int(0), Some(EstackArgT::None));
        assert_eq!(EstackArgT::from_int(1), Some(EstackArgT::Sfile));
        assert_eq!(EstackArgT::from_int(2), Some(EstackArgT::Stack));
        assert_eq!(EstackArgT::from_int(3), Some(EstackArgT::Script));
        assert_eq!(EstackArgT::from_int(4), None);
    }

    #[test]
    fn test_dip_flags() {
        assert!(rs_dip_has_all(dip::ALL));
        assert!(!rs_dip_has_all(dip::DIR));

        assert!(rs_dip_has_dir(dip::DIR));
        assert!(!rs_dip_has_dir(dip::ALL));

        let combined = dip::ALL | dip::DIR | dip::ERR;
        assert!(rs_dip_has_all(combined));
        assert!(rs_dip_has_dir(combined));
        assert!(rs_dip_has_err(combined));
        assert!(!rs_dip_has_start(combined));
    }

    #[test]
    fn test_dip_package_search() {
        assert!(!rs_dip_is_package_search(0));
        assert!(rs_dip_is_package_search(dip::START));
        assert!(rs_dip_is_package_search(dip::OPT));
        assert!(rs_dip_is_package_search(dip::START | dip::OPT));
        assert!(!rs_dip_is_package_search(dip::ALL | dip::DIR));
    }

    #[test]
    fn test_handle_null() {
        assert!(EstackHandle::null().is_null());
        assert!(ScriptItemHandle::null().is_null());
        assert!(SourceCookieHandle::null().is_null());
    }

    #[test]
    fn test_doso_constants() {
        assert_eq!(doso::NONE, 0);
        assert_eq!(doso::VIMRC, 1);
    }
}
