//! Helper functions for startup
//!
//! Implements `rs_get_number_arg` and `rs_check_swap_exists_action`
//! replacing the static C functions in main.c.

use std::ffi::c_int;

// Extern C declarations
unsafe extern "C" {
    fn ui_call_error_exit(status: c_int);
    fn getout(exitval: c_int) -> !;
    fn handle_swap_exists(old_curbuf: *mut std::ffi::c_void);
    static mut swap_exists_action: c_int;
}

// swap_exists_action values (from memline.h / globals.h)
const SEA_QUIT: c_int = 1;

/// Gets the integer value of a numeric command line argument if given,
/// such as '-o10'.
///
/// # Safety
/// `p` must be a valid C string. `idx` must be a valid pointer to an int.
///
/// Returns `def` unmodified if argument isn't given or is non-numeric.
/// Returns argument's numeric value otherwise.
#[export_name = "get_number_arg"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_get_number_arg(
    p: *const std::ffi::c_char,
    idx: *mut c_int,
    def: c_int,
) -> c_int {
    // Check if p[*idx] is an ASCII digit
    let i = *idx as usize;
    let byte = *(p.add(i)) as u8;
    if byte.is_ascii_digit() {
        // atoi from the current position
        let ptr = p.add(i);
        let val = libc_atoi(ptr);
        // Advance idx past all digits
        let mut j = i;
        loop {
            let b = *(p.add(j)) as u8;
            if !b.is_ascii_digit() {
                break;
            }
            j += 1;
        }
        *idx = j as c_int;
        val
    } else {
        def
    }
}

/// Parse an integer at the given C string pointer using simple ASCII parsing.
///
/// # Safety
/// `ptr` must point to a nul-terminated sequence starting with digits.
unsafe fn libc_atoi(ptr: *const std::ffi::c_char) -> c_int {
    let mut result: c_int = 0;
    let mut i = 0usize;
    loop {
        let b = *(ptr.add(i)) as u8;
        if !b.is_ascii_digit() {
            break;
        }
        result = result
            .saturating_mul(10)
            .saturating_add((b - b'0') as c_int);
        i += 1;
    }
    result
}

/// Check the result of the ATTENTION dialog:
/// When "Quit" selected, exit Nvim.
/// When "Recover" selected, recover the file.
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_check_swap_exists_action() {
    if swap_exists_action == SEA_QUIT {
        ui_call_error_exit(1);
        getout(1);
    }
    handle_swap_exists(std::ptr::null_mut());
}
