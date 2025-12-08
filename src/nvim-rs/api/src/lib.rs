//! API utilities for Neovim
//!
//! This crate provides C-compatible implementations of API utility functions.

use std::ffi::c_int;

/// Mask for all internal calls
const INTERNAL_CALL_MASK: u64 = 1u64 << 63;

/// Check whether a channel_id refers to an internal call.
///
/// Internal calls include Vimscript code and Lua code, identified by
/// having the high bit set in the channel_id.
///
/// # Arguments
/// * `channel_id` - The channel ID to check
///
/// # Returns
/// 1 if the channel_id refers to an internal channel, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_is_internal_call(channel_id: u64) -> c_int {
    c_int::from((channel_id & INTERNAL_CALL_MASK) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_internal_call() {
        // External calls (bit 63 not set)
        assert_eq!(rs_is_internal_call(0), 0);
        assert_eq!(rs_is_internal_call(1), 0);
        assert_eq!(rs_is_internal_call(12345), 0);
        assert_eq!(rs_is_internal_call((1u64 << 62) - 1), 0);

        // Internal calls (bit 63 set)
        // VIML_INTERNAL_CALL = INTERNAL_CALL_MASK
        assert_ne!(rs_is_internal_call(INTERNAL_CALL_MASK), 0);
        // LUA_INTERNAL_CALL = VIML_INTERNAL_CALL + 1
        assert_ne!(rs_is_internal_call(INTERNAL_CALL_MASK + 1), 0);
        // Any value with bit 63 set
        assert_ne!(rs_is_internal_call(u64::MAX), 0);
        assert_ne!(rs_is_internal_call(INTERNAL_CALL_MASK | 0x12345), 0);
    }
}
