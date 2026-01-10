//! File and path functions for VimL.
//!
//! This module implements file/path-related VimL functions from `src/nvim/eval/funcs.c`:
//! - `fnamemodify()` - Modify file name according to modifiers
//! - `glob()` - Expand wildcards
//! - `resolve()` - Resolve symlinks
//! - `simplify()` - Simplify path
//! - `pathshorten()` - Shorten path components
//!
//! These are mostly helpers; actual file operations use the nvim-path crate.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

// =============================================================================
// Filename Modifiers
// =============================================================================

/// Filename modifier flags (matching C's path_modifiers).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PathModifiers {
    /// :p - full path
    pub full_path: bool,
    /// :~ - relative to home
    pub relative_home: bool,
    /// :. - relative to current directory
    pub relative_cwd: bool,
    /// :h - head (directory part)
    pub head: bool,
    /// :t - tail (file name only)
    pub tail: bool,
    /// :r - remove extension
    pub remove_ext: bool,
    /// :e - extension only
    pub ext_only: bool,
    /// :s - substitute
    pub substitute: bool,
    /// :S - shell escape
    pub shell_escape: bool,
    /// :8 - 8.3 filename (Windows)
    pub short_name: bool,
    /// Number of :h modifiers
    pub head_count: c_int,
    /// Number of :t modifiers
    pub tail_count: c_int,
    /// Number of :r modifiers
    pub remove_ext_count: c_int,
    /// Number of :e modifiers
    pub ext_only_count: c_int,
}

impl PathModifiers {
    /// Create new empty modifiers.
    pub const fn new() -> Self {
        Self {
            full_path: false,
            relative_home: false,
            relative_cwd: false,
            head: false,
            tail: false,
            remove_ext: false,
            ext_only: false,
            substitute: false,
            shell_escape: false,
            short_name: false,
            head_count: 0,
            tail_count: 0,
            remove_ext_count: 0,
            ext_only_count: 0,
        }
    }

    /// Check if any modifiers are set.
    pub const fn is_empty(&self) -> bool {
        !self.full_path
            && !self.relative_home
            && !self.relative_cwd
            && !self.head
            && !self.tail
            && !self.remove_ext
            && !self.ext_only
            && !self.substitute
            && !self.shell_escape
            && !self.short_name
    }

    /// Check if path expansion is needed.
    pub const fn needs_expansion(&self) -> bool {
        self.full_path || self.relative_home || self.relative_cwd
    }

    /// Check if path component extraction is needed.
    pub const fn needs_component(&self) -> bool {
        self.head || self.tail || self.remove_ext || self.ext_only
    }
}

/// Parse a single modifier character.
///
/// Returns true if the character was a valid modifier.
pub fn parse_modifier_char(mods: &mut PathModifiers, c: u8) -> bool {
    match c {
        b'p' => {
            mods.full_path = true;
            true
        }
        b'~' => {
            mods.relative_home = true;
            true
        }
        b'.' => {
            mods.relative_cwd = true;
            true
        }
        b'h' => {
            mods.head = true;
            mods.head_count += 1;
            true
        }
        b't' => {
            mods.tail = true;
            mods.tail_count += 1;
            true
        }
        b'r' => {
            mods.remove_ext = true;
            mods.remove_ext_count += 1;
            true
        }
        b'e' => {
            mods.ext_only = true;
            mods.ext_only_count += 1;
            true
        }
        b's' => {
            mods.substitute = true;
            true
        }
        b'S' => {
            mods.shell_escape = true;
            true
        }
        b'8' => {
            mods.short_name = true;
            true
        }
        _ => false,
    }
}

/// Parse modifier string (e.g., ":p:h:r").
///
/// Returns the modifiers and the number of bytes consumed.
pub fn parse_modifiers(s: &[u8]) -> (PathModifiers, usize) {
    let mut mods = PathModifiers::new();
    let mut i = 0;

    while i < s.len() {
        if s[i] != b':' {
            break;
        }
        i += 1;
        if i >= s.len() {
            break;
        }
        if !parse_modifier_char(&mut mods, s[i]) {
            // Unknown modifier, back up
            i -= 1;
            break;
        }
        i += 1;
    }

    (mods, i)
}

/// FFI export: parse path modifiers.
///
/// # Safety
/// - `s` must be a valid pointer to at least `len` bytes.
/// - `mods` must be a valid pointer to PathModifiers.
/// - `consumed` must be a valid pointer to c_int or null.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_path_modifiers(
    s: *const u8,
    len: c_int,
    mods: *mut PathModifiers,
    consumed: *mut c_int,
) -> bool {
    if s.is_null() || mods.is_null() || len < 0 {
        if !consumed.is_null() {
            *consumed = 0;
        }
        return false;
    }

    let slice = std::slice::from_raw_parts(s, len as usize);
    let (parsed, count) = parse_modifiers(slice);
    *mods = parsed;
    if !consumed.is_null() {
        *consumed = count as c_int;
    }
    true
}

/// FFI export: check if modifiers are empty.
#[no_mangle]
pub extern "C" fn rs_path_modifiers_is_empty(mods: *const PathModifiers) -> bool {
    if mods.is_null() {
        return true;
    }
    unsafe { (*mods).is_empty() }
}

/// FFI export: check if modifiers need expansion.
#[no_mangle]
pub extern "C" fn rs_path_modifiers_needs_expansion(mods: *const PathModifiers) -> bool {
    if mods.is_null() {
        return false;
    }
    unsafe { (*mods).needs_expansion() }
}

// =============================================================================
// Path Component Helpers
// =============================================================================

/// Find the last path separator in a path.
///
/// Returns the index after the separator, or 0 if none found.
pub fn find_path_tail(path: &[u8]) -> usize {
    // Search backwards for path separator
    for (i, &c) in path.iter().enumerate().rev() {
        if c == b'/' || c == b'\\' {
            return i + 1;
        }
    }
    0
}

/// Find the extension start position.
///
/// Returns the index of the dot, or the length if no extension.
pub fn find_extension(path: &[u8]) -> usize {
    let tail_start = find_path_tail(path);
    let tail = &path[tail_start..];

    // Search backwards from end for '.'
    // Skip leading dots (hidden files like .bashrc)
    let skip_leading = tail.iter().take_while(|&&c| c == b'.').count();

    for (i, &c) in tail[skip_leading..].iter().enumerate().rev() {
        if c == b'.' {
            return tail_start + skip_leading + i;
        }
    }
    path.len()
}

/// Get the directory part of a path (head).
///
/// Returns the byte slice up to and including the last separator,
/// or an empty slice if no separator found.
pub fn path_head(path: &[u8]) -> &[u8] {
    let tail_pos = find_path_tail(path);
    if tail_pos == 0 {
        &[]
    } else {
        &path[..tail_pos]
    }
}

/// Get the filename part of a path (tail).
pub fn path_tail(path: &[u8]) -> &[u8] {
    let tail_pos = find_path_tail(path);
    &path[tail_pos..]
}

/// Get the path without extension (root).
pub fn path_root(path: &[u8]) -> &[u8] {
    let ext_pos = find_extension(path);
    &path[..ext_pos]
}

/// Get only the extension (without the dot).
pub fn path_ext(path: &[u8]) -> &[u8] {
    let ext_pos = find_extension(path);
    if ext_pos < path.len() {
        &path[ext_pos + 1..]
    } else {
        &[]
    }
}

/// FFI export: find path tail position.
#[no_mangle]
pub unsafe extern "C" fn rs_find_path_tail(path: *const u8, len: c_int) -> c_int {
    if path.is_null() || len < 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(path, len as usize);
    find_path_tail(slice) as c_int
}

/// FFI export: find extension position.
#[no_mangle]
pub unsafe extern "C" fn rs_find_extension(path: *const u8, len: c_int) -> c_int {
    if path.is_null() || len < 0 {
        return len;
    }
    let slice = std::slice::from_raw_parts(path, len as usize);
    find_extension(slice) as c_int
}

// =============================================================================
// Glob Pattern Helpers
// =============================================================================

/// Glob pattern flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobFlags {
    /// Return files only (no directories)
    pub files_only: bool,
    /// Return directories only
    pub dirs_only: bool,
    /// Include hidden files (starting with .)
    pub include_hidden: bool,
    /// Follow symlinks
    pub follow_symlinks: bool,
    /// Return absolute paths
    pub absolute_paths: bool,
    /// Case-insensitive matching
    pub ignore_case: bool,
    /// List files instead of expanding wildcards
    pub list_mode: bool,
    /// Don't sort results
    pub no_sort: bool,
}

impl GlobFlags {
    /// Create default flags.
    pub const fn new() -> Self {
        Self {
            files_only: false,
            dirs_only: false,
            include_hidden: false,
            follow_symlinks: true,
            absolute_paths: false,
            ignore_case: false,
            list_mode: false,
            no_sort: false,
        }
    }
}

/// Check if a character is a glob metacharacter.
pub fn is_glob_meta(c: u8) -> bool {
    matches!(c, b'*' | b'?' | b'[' | b']' | b'{' | b'}')
}

/// Check if a path contains glob metacharacters.
pub fn has_wildcards(path: &[u8]) -> bool {
    // Skip backslash-escaped characters
    let mut i = 0;
    while i < path.len() {
        if path[i] == b'\\' && i + 1 < path.len() {
            i += 2;
            continue;
        }
        if is_glob_meta(path[i]) {
            return true;
        }
        i += 1;
    }
    false
}

/// FFI export: check if path has wildcards.
#[no_mangle]
pub unsafe extern "C" fn rs_path_has_wildcards(path: *const u8, len: c_int) -> bool {
    if path.is_null() || len < 0 {
        return false;
    }
    let slice = std::slice::from_raw_parts(path, len as usize);
    has_wildcards(slice)
}

/// FFI export: check if character is glob metacharacter.
#[no_mangle]
pub extern "C" fn rs_is_glob_meta(c: c_int) -> bool {
    if c < 0 || c > 255 {
        return false;
    }
    is_glob_meta(c as u8)
}

// =============================================================================
// Path Simplification
// =============================================================================

/// Count the number of path components.
pub fn count_path_components(path: &[u8]) -> usize {
    if path.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut in_component = false;

    for &c in path {
        if c == b'/' || c == b'\\' {
            if in_component {
                count += 1;
                in_component = false;
            }
        } else {
            in_component = true;
        }
    }

    // Count last component if not ending in separator
    if in_component {
        count += 1;
    }

    count
}

/// Check if path component is "." (current directory).
pub fn is_current_dir(component: &[u8]) -> bool {
    component == b"."
}

/// Check if path component is ".." (parent directory).
pub fn is_parent_dir(component: &[u8]) -> bool {
    component == b".."
}

/// FFI export: count path components.
#[no_mangle]
pub unsafe extern "C" fn rs_count_path_components(path: *const u8, len: c_int) -> c_int {
    if path.is_null() || len < 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts(path, len as usize);
    count_path_components(slice) as c_int
}

// =============================================================================
// Path Shortening (pathshorten)
// =============================================================================

/// Shorten each path component to a single character.
///
/// For "foo/bar/baz.txt" returns "f/b/baz.txt".
/// The last component is not shortened.
///
/// Returns the number of bytes written to `out`.
pub fn shorten_path(path: &[u8], out: &mut [u8]) -> usize {
    if path.is_empty() || out.is_empty() {
        return 0;
    }

    let mut out_pos = 0;

    // Find the last separator to identify the last component
    let last_sep = path
        .iter()
        .enumerate()
        .rev()
        .find(|(_, &c)| c == b'/' || c == b'\\')
        .map(|(idx, _)| idx);

    let mut component_start = 0;

    for (i, &c) in path.iter().enumerate() {
        if c == b'/' || c == b'\\' {
            // Shorten this component (all components before the last)
            if component_start < i && out_pos < out.len() {
                // Skip leading dots for hidden files
                let mut comp_i = component_start;
                while comp_i < i && path[comp_i] == b'.' && out_pos < out.len() {
                    out[out_pos] = b'.';
                    out_pos += 1;
                    comp_i += 1;
                }
                if comp_i < i && out_pos < out.len() {
                    out[out_pos] = path[comp_i];
                    out_pos += 1;
                }
            }
            // Copy separator
            if out_pos < out.len() {
                out[out_pos] = c;
                out_pos += 1;
            }
            component_start = i + 1;
        }
    }

    // Copy last component entirely (after last separator, or entire path if no separator)
    let last_start = last_sep.map_or(0, |s| s + 1);
    for &b in &path[last_start..] {
        if out_pos >= out.len() {
            break;
        }
        out[out_pos] = b;
        out_pos += 1;
    }

    out_pos
}

/// FFI export: shorten path.
///
/// # Safety
/// - `path` must be valid for `path_len` bytes.
/// - `out` must be valid for `out_len` bytes.
/// - `written` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_shorten_path(
    path: *const u8,
    path_len: c_int,
    out: *mut u8,
    out_len: c_int,
    written: *mut c_int,
) -> bool {
    if path.is_null() || out.is_null() || path_len < 0 || out_len < 0 {
        if !written.is_null() {
            *written = 0;
        }
        return false;
    }

    let path_slice = std::slice::from_raw_parts(path, path_len as usize);
    let out_slice = std::slice::from_raw_parts_mut(out, out_len as usize);

    let count = shorten_path(path_slice, out_slice);
    if !written.is_null() {
        *written = count as c_int;
    }
    true
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifiers() {
        let (mods, consumed) = parse_modifiers(b":p:h:r");
        assert_eq!(consumed, 6);
        assert!(mods.full_path);
        assert!(mods.head);
        assert!(mods.remove_ext);
        assert_eq!(mods.head_count, 1);
        assert_eq!(mods.remove_ext_count, 1);

        let (mods, consumed) = parse_modifiers(b":t");
        assert_eq!(consumed, 2);
        assert!(mods.tail);

        let (mods, consumed) = parse_modifiers(b":h:h:h");
        assert_eq!(consumed, 6);
        assert!(mods.head);
        assert_eq!(mods.head_count, 3);

        let (mods, consumed) = parse_modifiers(b"no_colon");
        assert_eq!(consumed, 0);
        assert!(mods.is_empty());
    }

    #[test]
    fn test_find_path_tail() {
        assert_eq!(find_path_tail(b"/home/user/file.txt"), 11);
        assert_eq!(find_path_tail(b"file.txt"), 0);
        assert_eq!(find_path_tail(b"/"), 1);
        assert_eq!(find_path_tail(b""), 0);
        assert_eq!(find_path_tail(b"C:\\Users\\file.txt"), 9);
    }

    #[test]
    fn test_find_extension() {
        assert_eq!(find_extension(b"file.txt"), 4);
        assert_eq!(find_extension(b"file.tar.gz"), 8);
        assert_eq!(find_extension(b"file"), 4);
        assert_eq!(find_extension(b".bashrc"), 7); // Hidden file, no extension
        assert_eq!(find_extension(b".hidden.txt"), 7);
        assert_eq!(find_extension(b"/path/to/file.txt"), 13);
    }

    #[test]
    fn test_path_components() {
        assert_eq!(path_head(b"/home/user/file.txt"), b"/home/user/");
        assert_eq!(path_tail(b"/home/user/file.txt"), b"file.txt");
        assert_eq!(path_root(b"/home/user/file.txt"), b"/home/user/file");
        assert_eq!(path_ext(b"/home/user/file.txt"), b"txt");
    }

    #[test]
    fn test_has_wildcards() {
        assert!(has_wildcards(b"*.txt"));
        assert!(has_wildcards(b"file?.txt"));
        assert!(has_wildcards(b"file[0-9].txt"));
        assert!(!has_wildcards(b"file.txt"));
        assert!(!has_wildcards(b"\\*.txt")); // Escaped
    }

    #[test]
    fn test_count_path_components() {
        assert_eq!(count_path_components(b"/home/user/file.txt"), 3);
        assert_eq!(count_path_components(b"file.txt"), 1);
        assert_eq!(count_path_components(b"/"), 0);
        assert_eq!(count_path_components(b""), 0);
        assert_eq!(count_path_components(b"a/b/c/"), 3);
    }

    #[test]
    fn test_shorten_path() {
        let mut out = [0u8; 64];

        let len = shorten_path(b"foo/bar/baz.txt", &mut out);
        assert_eq!(&out[..len], b"f/b/baz.txt");

        let len = shorten_path(b"/home/user/documents/file.txt", &mut out);
        assert_eq!(&out[..len], b"/h/u/d/file.txt");

        let len = shorten_path(b"file.txt", &mut out);
        assert_eq!(&out[..len], b"file.txt");

        let len = shorten_path(b".hidden/file.txt", &mut out);
        assert_eq!(&out[..len], b".h/file.txt");
    }

    #[test]
    fn test_is_special_dirs() {
        assert!(is_current_dir(b"."));
        assert!(!is_current_dir(b".."));
        assert!(is_parent_dir(b".."));
        assert!(!is_parent_dir(b"."));
    }
}
