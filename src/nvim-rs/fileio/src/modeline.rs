//! Modeline parsing for Neovim.
//!
//! This module handles parsing of modelines - special comments in files that
//! contain Vim/Neovim settings. Modelines are typically found in the first
//! or last few lines of a file.
//!
//! Supported modeline formats:
//! - `ex: option=value option=value`
//! - `vi: option=value option=value`
//! - `vim: option=value option=value`
//! - `vim: set option=value option=value:`
//! - `Vim: set option=value option=value:`
//!
//! With version constraints:
//! - `vim<700: ...` (versions less than 700)
//! - `vim=700: ...` (version equals 700)
//! - `vim>700: ...` (versions greater than 700)

use std::ffi::c_int;

// =============================================================================
// Modeline Detection
// =============================================================================

/// Type of modeline prefix found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelinePrefix {
    /// No modeline prefix found
    None,
    /// "ex:" prefix
    Ex,
    /// "vi:" prefix
    Vi,
    /// "vim:" prefix (lowercase)
    Vim,
    /// "Vim:" prefix (uppercase V, requires "set")
    VimUpper,
}

/// Result of modeline detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelineMatch {
    /// The type of modeline prefix found
    pub prefix: ModelinePrefix,
    /// Byte offset where the modeline content starts (after the prefix and colon)
    pub content_start: usize,
    /// Version constraint if present (for vim<N, vim=N, vim>N)
    pub version_constraint: Option<VersionConstraint>,
}

/// Version constraint from a modeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionConstraint {
    /// The operator (<, =, >)
    pub operator: VersionOp,
    /// The version number
    pub version: i32,
}

/// Version comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionOp {
    /// Less than
    Lt,
    /// Equal to
    Eq,
    /// Greater than
    Gt,
}

impl VersionConstraint {
    /// Check if the given Vim version satisfies this constraint.
    pub fn is_satisfied(&self, vim_version: i32) -> bool {
        match self.operator {
            VersionOp::Lt => vim_version < self.version,
            VersionOp::Eq => vim_version == self.version,
            VersionOp::Gt => vim_version > self.version,
        }
    }
}

/// Detect a modeline prefix in a line.
///
/// Searches for "ex:", "vi:", "vim:", or "Vim:" at the start of the line
/// or after whitespace.
///
/// # Arguments
/// * `line` - The line to check
/// * `vim_version` - Current Vim version for version constraints
///
/// # Returns
/// `Some(ModelineMatch)` if a modeline is found, `None` otherwise
pub fn detect_modeline(line: &[u8], vim_version: i32) -> Option<ModelineMatch> {
    if line.is_empty() {
        return None;
    }

    let mut prev_is_space = true; // Start of line counts as "after space"
    let mut i = 0;

    while i < line.len() {
        // Only check for modeline prefixes at start of line or after whitespace
        if !prev_is_space {
            prev_is_space = line[i].is_ascii_whitespace();
            i += 1;
            continue;
        }

        // Check for "ex:" (only valid if not at start of line, i.e., prev != -1 in C)
        if i > 0 && line[i..].starts_with(b"ex:") {
            return Some(ModelineMatch {
                prefix: ModelinePrefix::Ex,
                content_start: i + 3,
                version_constraint: None,
            });
        }

        // Check for "vi:"
        if line[i..].starts_with(b"vi:") {
            return Some(ModelineMatch {
                prefix: ModelinePrefix::Vi,
                content_start: i + 3,
                version_constraint: None,
            });
        }

        // Check for "vim" or "Vim"
        if i + 3 <= line.len()
            && (line[i] == b'v' || line[i] == b'V')
            && line[i + 1] == b'i'
            && line[i + 2] == b'm'
        {
            let is_upper = line[i] == b'V';
            let after_vim = &line[i + 3..];

            if let Some(result) = parse_vim_modeline(after_vim, i + 3, is_upper, vim_version) {
                return Some(result);
            }
        }

        prev_is_space = line[i].is_ascii_whitespace();
        i += 1;
    }

    None
}

/// Parse a vim-style modeline after the "vim" or "Vim" prefix.
fn parse_vim_modeline(
    after_vim: &[u8],
    base_offset: usize,
    is_upper: bool,
    vim_version: i32,
) -> Option<ModelineMatch> {
    if after_vim.is_empty() {
        return None;
    }

    // Check for version constraint: <, =, or > followed by digits
    let (version_constraint, rest_start) = if !after_vim.is_empty()
        && (after_vim[0] == b'<' || after_vim[0] == b'=' || after_vim[0] == b'>')
    {
        let op = match after_vim[0] {
            b'<' => VersionOp::Lt,
            b'=' => VersionOp::Eq,
            b'>' => VersionOp::Gt,
            _ => unreachable!(),
        };

        // Parse version number
        let mut end = 1;
        while end < after_vim.len() && after_vim[end].is_ascii_digit() {
            end += 1;
        }

        if end > 1 {
            let version_str = std::str::from_utf8(&after_vim[1..end]).ok()?;
            let version: i32 = version_str.parse().ok()?;
            (
                Some(VersionConstraint {
                    operator: op,
                    version,
                }),
                end,
            )
        } else {
            (None, 0)
        }
    } else if !after_vim.is_empty() && after_vim[0].is_ascii_digit() {
        // Just digits after "vim" (implicit version check)
        let mut end = 0;
        while end < after_vim.len() && after_vim[end].is_ascii_digit() {
            end += 1;
        }
        let version_str = std::str::from_utf8(&after_vim[..end]).ok()?;
        let version: i32 = version_str.parse().ok()?;
        // Implicit: version must be <= vim_version
        if vim_version < version {
            return None;
        }
        (None, end)
    } else {
        (None, 0)
    };

    let rest = &after_vim[rest_start..];

    // Must have ":" after the version (or immediately after "vim")
    if rest.is_empty() || rest[0] != b':' {
        return None;
    }

    // Check version constraint if present
    if let Some(ref constraint) = version_constraint {
        if !constraint.is_satisfied(vim_version) {
            return None;
        }
    }

    // For "Vim:" (uppercase), require "set" after the colon
    if is_upper {
        let after_colon = &rest[1..];
        let trimmed = skip_whitespace(after_colon);
        if !trimmed.starts_with(b"set") {
            return None;
        }
    }

    Some(ModelineMatch {
        prefix: if is_upper {
            ModelinePrefix::VimUpper
        } else {
            ModelinePrefix::Vim
        },
        content_start: base_offset + rest_start + 1, // +1 for the colon
        version_constraint,
    })
}

/// Skip leading ASCII whitespace.
fn skip_whitespace(data: &[u8]) -> &[u8] {
    let start = data
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(data.len());
    &data[start..]
}

// =============================================================================
// Modeline Content Parsing
// =============================================================================

/// Result of parsing modeline options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelineOption<'a> {
    /// Option name
    pub name: &'a [u8],
    /// Option value (if present)
    pub value: Option<&'a [u8]>,
    /// Whether option was negated (no prefix)
    pub negated: bool,
    /// Whether option was inverted (inv prefix)
    pub inverted: bool,
}

/// Extract the content portion of a modeline (after the prefix).
///
/// Handles:
/// - "set " or "se " prefix stripping
/// - Escaped colons (\:)
/// - Terminating colon for "set" command
///
/// # Arguments
/// * `content` - The modeline content after the prefix
///
/// # Returns
/// The options string and whether a "set" command was found
pub fn extract_modeline_content(content: &[u8]) -> (&[u8], bool) {
    let trimmed = skip_whitespace(content);

    // Check for "set " or "se " prefix
    let (options_start, has_set) = if trimmed.starts_with(b"set ") {
        (4, true)
    } else if trimmed.starts_with(b"se ") {
        (3, true)
    } else {
        (0, false)
    };

    let options = &trimmed[options_start..];

    // Find the end of the options
    // For "set" command, look for terminating ':'
    // Otherwise, options go to end of line
    if has_set {
        // Find terminating colon (not escaped)
        let end = find_unescaped_colon(options).unwrap_or(options.len());
        (&options[..end], true)
    } else {
        // For non-set modelines, content continues to next ':'
        let end = find_unescaped_colon(options).unwrap_or(options.len());
        (&options[..end], false)
    }
}

/// Find the position of an unescaped colon.
fn find_unescaped_colon(data: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\\' && i + 1 < data.len() && data[i + 1] == b':' {
            // Skip escaped colon
            i += 2;
        } else if data[i] == b':' {
            return Some(i);
        } else {
            i += 1;
        }
    }
    None
}

// =============================================================================
// Security Checks
// =============================================================================

/// Options that are not allowed in modelines for security reasons.
pub const MODELINE_FORBIDDEN_OPTIONS: &[&str] = &[
    "exrc",
    "secure",
    "shell",
    "shellcmdflag",
    "shellpipe",
    "shellquote",
    "shellredir",
    "shelltemp",
    "shellxescape",
    "shellxquote",
    "modeline",
    "modelineexpr",
];

/// Check if an option name is forbidden in modelines.
pub fn is_modeline_forbidden(option: &[u8]) -> bool {
    let opt_str = match std::str::from_utf8(option) {
        Ok(s) => s.to_ascii_lowercase(),
        Err(_) => return false,
    };

    MODELINE_FORBIDDEN_OPTIONS
        .iter()
        .any(|&forbidden| opt_str == forbidden)
}

/// Options that are only allowed in modelines when 'modelineexpr' is on.
pub const MODELINE_EXPR_OPTIONS: &[&str] = &[
    "foldexpr",
    "foldtext",
    "includeexpr",
    "indentexpr",
    "statusline",
    "tabline",
    "foldmarker",
    "foldignore",
];

/// Check if an option requires 'modelineexpr' to be set.
pub fn requires_modelineexpr(option: &[u8]) -> bool {
    let opt_str = match std::str::from_utf8(option) {
        Ok(s) => s.to_ascii_lowercase(),
        Err(_) => return false,
    };

    MODELINE_EXPR_OPTIONS
        .iter()
        .any(|&expr_opt| opt_str == expr_opt)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI wrapper for detect_modeline.
///
/// # Safety
/// - `line` must point to a valid buffer of at least `len` bytes
/// - `prefix_out` must be a valid pointer for writing
/// - `content_start_out` must be a valid pointer for writing
#[no_mangle]
pub unsafe extern "C" fn rs_detect_modeline(
    line: *const u8,
    len: usize,
    vim_version: c_int,
    prefix_out: *mut c_int,
    content_start_out: *mut usize,
) -> c_int {
    if line.is_null() || prefix_out.is_null() || content_start_out.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(line, len);

    match detect_modeline(slice, vim_version) {
        Some(result) => {
            *prefix_out = match result.prefix {
                ModelinePrefix::None => 0,
                ModelinePrefix::Ex => 1,
                ModelinePrefix::Vi => 2,
                ModelinePrefix::Vim => 3,
                ModelinePrefix::VimUpper => 4,
            };
            *content_start_out = result.content_start;
            1
        }
        None => {
            *prefix_out = 0;
            *content_start_out = 0;
            0
        }
    }
}

/// FFI wrapper for is_modeline_forbidden.
///
/// # Safety
/// `option` must point to a valid buffer of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_is_modeline_forbidden(option: *const u8, len: usize) -> c_int {
    if option.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(option, len);
    c_int::from(is_modeline_forbidden(slice))
}

/// FFI wrapper for requires_modelineexpr.
///
/// # Safety
/// `option` must point to a valid buffer of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_requires_modelineexpr(option: *const u8, len: usize) -> c_int {
    if option.is_null() {
        return 0;
    }

    let slice = std::slice::from_raw_parts(option, len);
    c_int::from(requires_modelineexpr(slice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_modeline_vi() {
        let line = b" vi: set ts=4 sw=4:";
        let result = detect_modeline(line, 800).unwrap();
        assert_eq!(result.prefix, ModelinePrefix::Vi);
        // content_start is right after "vi:" at position 4
        assert_eq!(result.content_start, 4);
    }

    #[test]
    fn test_detect_modeline_vim() {
        let line = b"/* vim: set ts=4: */";
        let result = detect_modeline(line, 800).unwrap();
        assert_eq!(result.prefix, ModelinePrefix::Vim);
    }

    #[test]
    fn test_detect_modeline_ex() {
        // "ex:" requires something before it
        let line = b"# ex: set ts=4:";
        let result = detect_modeline(line, 800).unwrap();
        assert_eq!(result.prefix, ModelinePrefix::Ex);
    }

    #[test]
    fn test_detect_modeline_vim_version() {
        // vim700: - requires version 700 or higher
        let line = b"vim700: set ts=4:";
        let result = detect_modeline(line, 800);
        assert!(result.is_some());

        // Should fail with older version
        let result = detect_modeline(line, 600);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_modeline_vim_version_constraint() {
        // vim<800: - versions less than 800
        let line = b"vim<800: set ts=4:";
        let result = detect_modeline(line, 700).unwrap();
        assert_eq!(result.prefix, ModelinePrefix::Vim);
        assert!(result.version_constraint.is_some());

        // Should fail with version 800 or higher
        let result = detect_modeline(line, 800);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_modeline_vim_upper() {
        // "Vim:" requires "set" after it
        let line = b"Vim: set ts=4:";
        let result = detect_modeline(line, 800).unwrap();
        assert_eq!(result.prefix, ModelinePrefix::VimUpper);

        // Without "set" should fail
        let line = b"Vim: ts=4";
        let result = detect_modeline(line, 800);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_modeline_no_match() {
        let line = b"This is not a modeline";
        assert!(detect_modeline(line, 800).is_none());

        // "ex:" at start of line doesn't count
        let line = b"ex: set ts=4:";
        assert!(detect_modeline(line, 800).is_none());
    }

    #[test]
    fn test_extract_modeline_content() {
        let content = b"set ts=4 sw=4:";
        let (opts, has_set) = extract_modeline_content(content);
        assert!(has_set);
        assert_eq!(opts, b"ts=4 sw=4");

        let content = b"ts=4 sw=4";
        let (opts, has_set) = extract_modeline_content(content);
        assert!(!has_set);
        assert_eq!(opts, b"ts=4 sw=4");
    }

    #[test]
    fn test_extract_modeline_content_escaped_colon() {
        let content = b"set path=./\\:../:";
        let (opts, has_set) = extract_modeline_content(content);
        assert!(has_set);
        // The escaped colon should be included
        assert!(opts.starts_with(b"path="));
    }

    #[test]
    fn test_is_modeline_forbidden() {
        assert!(is_modeline_forbidden(b"shell"));
        assert!(is_modeline_forbidden(b"SHELL")); // Case insensitive
        assert!(is_modeline_forbidden(b"exrc"));
        assert!(is_modeline_forbidden(b"modeline"));

        assert!(!is_modeline_forbidden(b"tabstop"));
        assert!(!is_modeline_forbidden(b"shiftwidth"));
    }

    #[test]
    fn test_requires_modelineexpr() {
        assert!(requires_modelineexpr(b"foldexpr"));
        assert!(requires_modelineexpr(b"statusline"));
        assert!(requires_modelineexpr(b"FOLDTEXT")); // Case insensitive

        assert!(!requires_modelineexpr(b"tabstop"));
        assert!(!requires_modelineexpr(b"expandtab"));
    }

    #[test]
    fn test_version_constraint() {
        let lt = VersionConstraint {
            operator: VersionOp::Lt,
            version: 800,
        };
        assert!(lt.is_satisfied(700));
        assert!(!lt.is_satisfied(800));
        assert!(!lt.is_satisfied(900));

        let eq = VersionConstraint {
            operator: VersionOp::Eq,
            version: 800,
        };
        assert!(!eq.is_satisfied(700));
        assert!(eq.is_satisfied(800));
        assert!(!eq.is_satisfied(900));

        let gt = VersionConstraint {
            operator: VersionOp::Gt,
            version: 800,
        };
        assert!(!gt.is_satisfied(700));
        assert!(!gt.is_satisfied(800));
        assert!(gt.is_satisfied(900));
    }
}
