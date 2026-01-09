//! Buffer name completion support.
//!
//! This module provides helper functions for buffer name completion.
//! The core buffer operations remain in C.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
}

/// Check if completion was interrupted during buffer name search.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for buffer name search.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_get_direction() -> c_int {
    nvim_get_compl_direction()
}

#[cfg(test)]
mod tests {
    // Buffer name completion doesn't have specific mode constants
    // as it's triggered via user-defined functions
}
