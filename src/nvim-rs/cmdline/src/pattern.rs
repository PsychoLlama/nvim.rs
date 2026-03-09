//! Pattern processing for command-line expansion
//!
//! This module handles pattern transformation for wildcard matching,
//! including converting file wildcards to regex patterns and escaping
//! special characters.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::expand::ExpandContext;

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Memory allocation
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);

    // String functions
    fn path_tail(fname: *mut c_char) -> *mut c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn xmemcpyz(dst: *mut c_char, src: *const c_char, len: usize);
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
}

// =============================================================================
// Pure Rust Pattern Functions
// =============================================================================

/// Check if a context requires literal pattern (no wildcard transformation).
///
/// For these contexts, the pattern is used as-is without adding '^' prefix
/// or converting wildcards.
#[must_use]
pub const fn context_uses_literal_pattern(context: ExpandContext) -> bool {
    matches!(
        context,
        ExpandContext::Findfunc
            | ExpandContext::Help
            | ExpandContext::Colors
            | ExpandContext::Compiler
            | ExpandContext::Ownsyntax
            | ExpandContext::Filetype
            | ExpandContext::Keymap
            | ExpandContext::Packadd
            | ExpandContext::Runtime
            | ExpandContext::Checkhealth
            | ExpandContext::Lua
    )
}

/// Check if a context is for tag expansion that may need literal pattern.
///
/// Tag patterns starting with "/" use literal matching.
#[must_use]
pub const fn is_tag_literal_context(context: ExpandContext) -> bool {
    matches!(context, ExpandContext::Tags | ExpandContext::TagsListfiles)
}

/// Calculate the length needed for internal pattern transformation.
///
/// When converting wildcards for regex matching:
/// - `*` becomes `.*` (+1)
/// - `~` becomes `\~` (+1)
/// - `.` becomes `\.` for buffers (+1)
/// - `\` becomes `\\` for user-defined (+1)
/// - Base: +2 for `^` prefix and NUL terminator
#[must_use]
pub fn calculate_internal_pattern_len(fname: &[u8], context: ExpandContext) -> usize {
    let mut new_len = fname.len() + 2; // +2 for '^' at start, NUL at end

    for &c in fname {
        match c {
            b'*' | b'~' => new_len += 1,
            b'.' if context == ExpandContext::Buffers => new_len += 1,
            b'\\'
                if matches!(
                    context,
                    ExpandContext::UserDefined | ExpandContext::UserList
                ) =>
            {
                new_len += 1;
            }
            _ => {}
        }
    }

    new_len
}

/// Transform a pattern for internal matching (non-file contexts).
///
/// This converts file-matching wildcards to regex patterns:
/// - Prepends `^` for anchoring
/// - `*` -> `.*` (any characters)
/// - `?` -> `.` (single character)
/// - `~` -> `\~` (literal tilde)
/// - `.` -> `\.` for buffer context (literal dot)
/// - `\` -> `\\` for user-defined (literal backslash)
///
/// Returns the transformed pattern as a Vec<u8>.
#[must_use]
pub fn transform_to_internal_pattern(fname: &[u8], context: ExpandContext) -> Vec<u8> {
    let capacity = calculate_internal_pattern_len(fname, context);
    let mut result = Vec::with_capacity(capacity);
    result.push(b'^');

    let is_user_context = matches!(
        context,
        ExpandContext::UserDefined | ExpandContext::UserList
    );
    let is_buffer_context = context == ExpandContext::Buffers;

    let mut i = 0;
    while i < fname.len() {
        // Skip backslash for non-user contexts
        if !is_user_context && fname[i] == b'\\' {
            i += 1;
            if i == fname.len() {
                break;
            }
        }

        match fname[i] {
            b'*' => {
                result.push(b'.');
                result.push(b'*');
            }
            b'~' => {
                result.push(b'\\');
                result.push(b'~');
            }
            b'?' => {
                result.push(b'.');
            }
            b'.' if is_buffer_context => {
                result.push(b'\\');
                result.push(b'.');
            }
            b'\\' if is_user_context => {
                result.push(b'\\');
                result.push(b'\\');
            }
            c => {
                result.push(c);
            }
        }
        i += 1;
    }

    result
}

/// Check if a file pattern should have a star appended.
///
/// Stars are NOT added when:
/// - Pattern starts with `~` (home directory)
/// - Pattern already ends with `*` (possibly escaped)
/// - Pattern contains `$` in the tail (variable expansion)
/// - Pattern contains backtick (command substitution)
/// - Pattern ends with `$` (strip the `$` instead)
#[must_use]
pub fn should_append_star(fname: &[u8], tail_start: usize) -> bool {
    if fname.is_empty() {
        return false;
    }

    // Check if starts with ~ (and tail is at start)
    if fname[0] == b'~' && tail_start == 0 {
        return false;
    }

    // Check if already ends with * (considering backslash escaping)
    let mut ends_in_star = fname[fname.len() - 1] == b'*';
    #[cfg(not(windows))]
    {
        // Count trailing backslashes
        let mut k = fname.len().saturating_sub(2);
        while k > 0 && fname[k] == b'\\' {
            ends_in_star = !ends_in_star;
            k = k.saturating_sub(1);
        }
    }
    if ends_in_star {
        return false;
    }

    // Check for $ in tail
    if fname[tail_start..].contains(&b'$') {
        return false;
    }

    // Check for backtick anywhere
    if fname.contains(&b'`') {
        return false;
    }

    true
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Prepare a string for expansion.
///
/// When expanding file names: Copy the string and add '*' at the end.
/// When expanding other names: Convert wildcards to regex pattern.
///
/// # Safety
///
/// `fname` must be a valid pointer to a string of at least `len` bytes.
/// The returned pointer must be freed with `xfree()`.
#[must_use]
#[unsafe(export_name = "addstar")]
pub unsafe extern "C" fn rs_addstar(
    fname: *const c_char,
    len: usize,
    context: c_int,
) -> *mut c_char {
    if fname.is_null() {
        return ptr::null_mut();
    }

    let Some(ctx) = ExpandContext::from_raw(context) else {
        return ptr::null_mut();
    };

    // Get the input as a byte slice
    let input = std::slice::from_raw_parts(fname.cast::<u8>(), len);

    // Check if this is a file expansion context
    if !ctx.uses_internal_matching() {
        // File expansion - append star
        let retval = xmalloc(len + 4);
        if retval.is_null() {
            return ptr::null_mut();
        }

        xmemcpyz(retval, fname, len);

        // Find the tail of the path
        let tail = path_tail(retval);
        #[allow(clippy::cast_sign_loss)]
        let tail_start = if tail.is_null() {
            0
        } else {
            tail.offset_from(retval) as usize
        };

        let mut result_len = len;

        if should_append_star(input, tail_start) {
            #[allow(clippy::cast_possible_wrap)]
            {
                *retval.add(result_len) = b'*' as c_char;
            }
            result_len += 1;
        } else if result_len > 0 && input[result_len - 1] == b'$' {
            // Strip trailing $
            result_len -= 1;
        }

        *retval.add(result_len) = 0; // NUL terminator
        return retval;
    }

    // Internal matching contexts

    // Check for literal pattern contexts
    if context_uses_literal_pattern(ctx) {
        return xstrnsave(fname, len);
    }

    // Check for tag patterns starting with /
    if is_tag_literal_context(ctx) && !input.is_empty() && input[0] == b'/' {
        return xstrnsave(fname, len);
    }

    // Transform to internal pattern
    let pattern = transform_to_internal_pattern(input, ctx);

    let retval = xmalloc(pattern.len() + 1);
    if retval.is_null() {
        return ptr::null_mut();
    }

    ptr::copy_nonoverlapping(pattern.as_ptr().cast(), retval, pattern.len());
    *retval.add(pattern.len()) = 0; // NUL terminator

    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_uses_literal_pattern() {
        assert!(context_uses_literal_pattern(ExpandContext::Help));
        assert!(context_uses_literal_pattern(ExpandContext::Colors));
        assert!(context_uses_literal_pattern(ExpandContext::Lua));
        assert!(!context_uses_literal_pattern(ExpandContext::Commands));
        assert!(!context_uses_literal_pattern(ExpandContext::Files));
    }

    #[test]
    fn test_is_tag_literal_context() {
        assert!(is_tag_literal_context(ExpandContext::Tags));
        assert!(is_tag_literal_context(ExpandContext::TagsListfiles));
        assert!(!is_tag_literal_context(ExpandContext::Files));
        assert!(!is_tag_literal_context(ExpandContext::Commands));
    }

    #[test]
    fn test_calculate_internal_pattern_len() {
        // Base case: +2 for ^ and NUL
        assert_eq!(
            calculate_internal_pattern_len(b"abc", ExpandContext::Commands),
            5
        );

        // * needs +1
        assert_eq!(
            calculate_internal_pattern_len(b"a*c", ExpandContext::Commands),
            6
        );

        // ~ needs +1
        assert_eq!(
            calculate_internal_pattern_len(b"a~c", ExpandContext::Commands),
            6
        );

        // . needs +1 for buffers
        assert_eq!(
            calculate_internal_pattern_len(b"a.c", ExpandContext::Buffers),
            6
        );

        // . doesn't need +1 for commands
        assert_eq!(
            calculate_internal_pattern_len(b"a.c", ExpandContext::Commands),
            5
        );

        // \ needs +1 for user-defined
        assert_eq!(
            calculate_internal_pattern_len(b"a\\c", ExpandContext::UserDefined),
            6
        );
    }

    #[test]
    fn test_transform_to_internal_pattern_basic() {
        let result = transform_to_internal_pattern(b"abc", ExpandContext::Commands);
        assert_eq!(result, b"^abc");
    }

    #[test]
    fn test_transform_to_internal_pattern_star() {
        let result = transform_to_internal_pattern(b"a*c", ExpandContext::Commands);
        assert_eq!(result, b"^a.*c");
    }

    #[test]
    fn test_transform_to_internal_pattern_question() {
        let result = transform_to_internal_pattern(b"a?c", ExpandContext::Commands);
        assert_eq!(result, b"^a.c");
    }

    #[test]
    fn test_transform_to_internal_pattern_tilde() {
        let result = transform_to_internal_pattern(b"a~c", ExpandContext::Commands);
        assert_eq!(result, b"^a\\~c");
    }

    #[test]
    fn test_transform_to_internal_pattern_dot_buffers() {
        let result = transform_to_internal_pattern(b"a.c", ExpandContext::Buffers);
        assert_eq!(result, b"^a\\.c");
    }

    #[test]
    fn test_transform_to_internal_pattern_dot_commands() {
        let result = transform_to_internal_pattern(b"a.c", ExpandContext::Commands);
        assert_eq!(result, b"^a.c");
    }

    #[test]
    fn test_transform_to_internal_pattern_backslash_user() {
        let result = transform_to_internal_pattern(b"a\\c", ExpandContext::UserDefined);
        assert_eq!(result, b"^a\\\\c");
    }

    #[test]
    fn test_transform_to_internal_pattern_backslash_commands() {
        // Backslash is skipped for non-user contexts
        let result = transform_to_internal_pattern(b"a\\c", ExpandContext::Commands);
        assert_eq!(result, b"^ac");
    }

    #[test]
    fn test_should_append_star_basic() {
        assert!(should_append_star(b"abc", 0));
        assert!(should_append_star(b"path/file", 5));
    }

    #[test]
    fn test_should_append_star_tilde() {
        // Don't add star if starts with ~ and tail is at start
        assert!(!should_append_star(b"~user", 0));
        // OK if ~ is not at start
        assert!(should_append_star(b"path/~file", 5));
    }

    #[test]
    fn test_should_append_star_already_star() {
        assert!(!should_append_star(b"abc*", 0));
    }

    #[test]
    fn test_should_append_star_dollar_in_tail() {
        assert!(!should_append_star(b"path/$var", 5));
    }

    #[test]
    fn test_should_append_star_backtick() {
        assert!(!should_append_star(b"path/`cmd`", 5));
    }

    #[test]
    fn test_should_append_star_ends_dollar() {
        // Ends with $ should return false (we strip $ instead)
        assert!(!should_append_star(b"abc$", 0));
    }

    #[test]
    fn test_should_append_star_empty() {
        assert!(!should_append_star(b"", 0));
    }
}
