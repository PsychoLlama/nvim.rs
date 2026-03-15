//! Error message constants for option validation
//!
//! This module provides Rust copies of error message strings used by
//! optionstr.c validation code. These are kept in sync with the C definitions.

// =============================================================================
// Error Message Constants
// =============================================================================

/// E535: Illegal character after <%c>
pub static E_ILLEGAL_CHARACTER_AFTER_CHR: &[u8] = b"E535: Illegal character after <%c>\0";

/// E536: Comma required
pub static E_COMMA_REQUIRED: &[u8] = b"E536: Comma required\0";

/// E595: 'showbreak' contains unprintable or wide character
pub static E_SHOWBREAK_CONTAINS_UNPRINTABLE_OR_WIDE_CHARACTER: &[u8] =
    b"E595: 'showbreak' contains unprintable or wide character\0";

/// E1511: Wrong number of characters for field "%s"
pub static E_WRONG_NUMBER_OF_CHARACTERS_FOR_FIELD_STR: &[u8] =
    b"E1511: Wrong number of characters for field \"%s\"\0";

/// E1512: Wrong character width for field "%s"
pub static E_WRONG_CHARACTER_WIDTH_FOR_FIELD_STR: &[u8] =
    b"E1512: Wrong character width for field \"%s\"\0";

/// E834: Conflicts with value of 'listchars'
pub static E_CONFLICTS_WITH_VALUE_OF_LISTCHARS: &[u8] =
    b"E834: Conflicts with value of 'listchars'\0";

/// E835: Conflicts with value of 'fillchars'
pub static E_CONFLICTS_WITH_VALUE_OF_FILLCHARS: &[u8] =
    b"E835: Conflicts with value of 'fillchars'\0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constants_are_null_terminated() {
        assert_eq!(*E_ILLEGAL_CHARACTER_AFTER_CHR.last().unwrap(), 0);
        assert_eq!(*E_COMMA_REQUIRED.last().unwrap(), 0);
        assert_eq!(
            *E_SHOWBREAK_CONTAINS_UNPRINTABLE_OR_WIDE_CHARACTER
                .last()
                .unwrap(),
            0
        );
        assert_eq!(
            *E_WRONG_NUMBER_OF_CHARACTERS_FOR_FIELD_STR.last().unwrap(),
            0
        );
        assert_eq!(*E_WRONG_CHARACTER_WIDTH_FOR_FIELD_STR.last().unwrap(), 0);
        assert_eq!(*E_CONFLICTS_WITH_VALUE_OF_LISTCHARS.last().unwrap(), 0);
        assert_eq!(*E_CONFLICTS_WITH_VALUE_OF_FILLCHARS.last().unwrap(), 0);
    }
}
