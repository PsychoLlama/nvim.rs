//! Default file loading
//!
//! This module handles loading default runtime files (ftplugin, indent, syntax, etc).

use std::ffi::{c_char, c_int};

// =============================================================================
// Default File Types
// =============================================================================

/// Default file type categories
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultType {
    /// filetype.vim/lua - filetype detection
    Filetype = 0,
    /// ftplugin/*.vim/lua - filetype plugins
    Ftplugin = 1,
    /// indent/*.vim/lua - indentation
    Indent = 2,
    /// syntax/*.vim/lua - syntax highlighting
    Syntax = 3,
    /// colors/*.vim/lua - color schemes
    Colors = 4,
    /// compiler/*.vim/lua - compiler settings
    Compiler = 5,
}

impl DefaultType {
    /// Get the directory name for this type
    pub const fn dir_name(&self) -> &'static [u8] {
        match self {
            Self::Filetype => b"ftdetect\0",
            Self::Ftplugin => b"ftplugin\0",
            Self::Indent => b"indent\0",
            Self::Syntax => b"syntax\0",
            Self::Colors => b"colors\0",
            Self::Compiler => b"compiler\0",
        }
    }

    /// Convert from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Filetype),
            1 => Some(Self::Ftplugin),
            2 => Some(Self::Indent),
            3 => Some(Self::Syntax),
            4 => Some(Self::Colors),
            5 => Some(Self::Compiler),
            _ => None,
        }
    }
}

/// Get directory name for a default type.
pub fn rs_default_type_dir(dtype: c_int) -> *const c_char {
    match DefaultType::from_int(dtype) {
        Some(t) => t.dir_name().as_ptr().cast(),
        None => std::ptr::null(),
    }
}

// =============================================================================
// Standard Filenames
// =============================================================================

/// Standard default file names
pub const FILETYPE_VIM: &[u8] = b"filetype.vim\0";
pub const FILETYPE_LUA: &[u8] = b"filetype.lua\0";
pub const SCRIPTS_VIM: &[u8] = b"scripts.vim\0";
pub const MENU_VIM: &[u8] = b"menu.vim\0";
pub const DEFAULTS_VIM: &[u8] = b"defaults.vim\0";

/// Get filetype.vim filename.
pub fn rs_filetype_vim() -> *const c_char {
    FILETYPE_VIM.as_ptr().cast()
}

/// Get filetype.lua filename.
pub fn rs_filetype_lua() -> *const c_char {
    FILETYPE_LUA.as_ptr().cast()
}

/// Get scripts.vim filename.
pub fn rs_scripts_vim() -> *const c_char {
    SCRIPTS_VIM.as_ptr().cast()
}

/// Get menu.vim filename.
pub fn rs_menu_vim() -> *const c_char {
    MENU_VIM.as_ptr().cast()
}

/// Get defaults.vim filename.
pub fn rs_defaults_vim() -> *const c_char {
    DEFAULTS_VIM.as_ptr().cast()
}

// =============================================================================
// Default Loading State
// =============================================================================

/// Flags for what defaults have been loaded
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultsLoaded {
    /// filetype.vim has been loaded
    pub filetype: bool,
    /// filetype.lua has been loaded
    pub filetype_lua: bool,
    /// scripts.vim has been loaded
    pub scripts: bool,
    /// menu.vim has been loaded
    pub menu: bool,
    /// syntax on/enable has been run
    pub syntax: bool,
    /// :filetype on has been run
    pub filetype_on: bool,
    /// :filetype plugin on has been run
    pub filetype_plugin: bool,
    /// :filetype indent on has been run
    pub filetype_indent: bool,
}

/// Create default loaded state (nothing loaded).
pub fn rs_defaults_loaded_new() -> DefaultsLoaded {
    DefaultsLoaded::default()
}

/// Check if basic filetype detection is loaded.
pub fn rs_defaults_has_filetype(loaded: &DefaultsLoaded) -> bool {
    loaded.filetype || loaded.filetype_lua
}

/// Check if full filetype support is enabled.
pub fn rs_defaults_filetype_enabled(loaded: &DefaultsLoaded) -> bool {
    loaded.filetype_on && rs_defaults_has_filetype(loaded)
}

/// Check if syntax highlighting is enabled.
pub fn rs_defaults_syntax_enabled(loaded: &DefaultsLoaded) -> bool {
    loaded.syntax
}

// =============================================================================
// init.vim/init.lua Locations
// =============================================================================

/// Standard init file names
pub const INIT_VIM: &[u8] = b"init.vim\0";
pub const INIT_LUA: &[u8] = b"init.lua\0";
pub const VIMRC: &[u8] = b".vimrc\0";
pub const EXRC: &[u8] = b".exrc\0";
pub const GVIMRC: &[u8] = b".gvimrc\0";

/// Get init.vim filename.
pub fn rs_init_vim() -> *const c_char {
    INIT_VIM.as_ptr().cast()
}

/// Get init.lua filename.
pub fn rs_init_lua() -> *const c_char {
    INIT_LUA.as_ptr().cast()
}

/// Get .vimrc filename.
pub fn rs_vimrc() -> *const c_char {
    VIMRC.as_ptr().cast()
}

/// Get .exrc filename.
pub fn rs_exrc() -> *const c_char {
    EXRC.as_ptr().cast()
}

/// Get .gvimrc filename.
pub fn rs_gvimrc() -> *const c_char {
    GVIMRC.as_ptr().cast()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_type() {
        assert_eq!(DefaultType::from_int(0), Some(DefaultType::Filetype));
        assert_eq!(DefaultType::from_int(5), Some(DefaultType::Compiler));
        assert_eq!(DefaultType::from_int(6), None);
    }

    #[test]
    fn test_default_type_dir() {
        let dir = rs_default_type_dir(DefaultType::Ftplugin as c_int);
        assert!(!dir.is_null());

        let invalid = rs_default_type_dir(100);
        assert!(invalid.is_null());
    }

    #[test]
    fn test_defaults_loaded() {
        let loaded = rs_defaults_loaded_new();
        assert!(!rs_defaults_has_filetype(&loaded));
        assert!(!rs_defaults_filetype_enabled(&loaded));
        assert!(!rs_defaults_syntax_enabled(&loaded));

        let mut loaded = loaded;
        loaded.filetype = true;
        assert!(rs_defaults_has_filetype(&loaded));
        assert!(!rs_defaults_filetype_enabled(&loaded)); // filetype_on not set

        loaded.filetype_on = true;
        assert!(rs_defaults_filetype_enabled(&loaded));
    }

    #[test]
    fn test_filenames() {
        assert!(!rs_filetype_vim().is_null());
        assert!(!rs_filetype_lua().is_null());
        assert!(!rs_scripts_vim().is_null());
        assert!(!rs_menu_vim().is_null());
        assert!(!rs_defaults_vim().is_null());
        assert!(!rs_init_vim().is_null());
        assert!(!rs_init_lua().is_null());
        assert!(!rs_vimrc().is_null());
        assert!(!rs_exrc().is_null());
        assert!(!rs_gvimrc().is_null());
    }
}
