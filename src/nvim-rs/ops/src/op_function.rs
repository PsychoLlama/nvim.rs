//! op_function: handle the "g@" operator (call 'operatorfunc' callback).
//!
//! Implements `op_function`, which invokes the 'operatorfunc' Vim callback
//! with the motion type as a string argument.
//!
//! Migrated from ops.c `static void op_function()`.

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Motion type constants (must match normal_defs.h)
// =============================================================================

const K_MT_LINE_WISE: c_int = 1;
const K_MT_BLOCK_WISE: c_int = 2;

// =============================================================================
// typval_T layout constants
// =============================================================================

/// sizeof(typval_T) = 16 bytes (verified by C static assert in eval_struct_check.c)
const TYPVAL_SIZE: usize = 16;

/// VAR_STRING = 2
const VAR_STRING: c_int = 2;
/// VAR_UNKNOWN = 0
const VAR_UNKNOWN: c_int = 0;

// TriState values
/// kNone = -1
const K_NONE: c_int = -1;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // opfunc_cb accessor (opfunc_cb lives in option_shim.c after migration)
    fn nvim_get_opfunc_cb() -> *mut c_void; // returns *mut Callback
    fn nvim_get_p_opfunc_nonempty() -> bool;

    // virtual_op and finish_op globals
    fn nvim_get_virtual_op() -> c_int;
    fn nvim_set_virtual_op(val: c_int);
    fn nvim_dpo_get_finish_op() -> bool;
    fn nvim_set_finish_op(val: bool);

    // curbuf b_op_start/end management
    fn nvim_opfunc_set_op_marks(oap: *mut OpargT);

    // curbuf b_op_start/end save/restore (packed: upper32=lnum, lower32=col)
    fn nvim_excmds_curbuf_op_save(out_start: *mut u64, out_end: *mut u64);
    fn nvim_excmds_curbuf_op_restore(saved_start: u64, saved_end: u64);

    // Error message
    fn nvim_emsg_e774_operatorfunc();

    // CMOD_LOCKMARKS check
    fn nvim_cmdmod_has_lockmarks() -> c_int;

    // callback_call - the Callback type is treated as opaque *mut c_void
    fn callback_call(
        callback: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> bool;

    // tv_clear for the rettv
    fn tv_clear(tv: *mut c_void);
}

// =============================================================================
// op_function implementation
// =============================================================================

/// Handle the "g@" operator: call 'operatorfunc'.
///
/// Mirrors C `static void op_function(const oparg_T *oap)` exactly.
///
/// # Safety
/// `oap` must be a valid non-null pointer to an initialized `oparg_T`.
pub unsafe fn rs_op_function_impl(oap: *mut OpargT) {
    if !nvim_get_p_opfunc_nonempty() {
        nvim_emsg_e774_operatorfunc();
        return;
    }

    // Save b_op_start and b_op_end (for CMOD_LOCKMARKS restore)
    let mut orig_start: u64 = 0;
    let mut orig_end: u64 = 0;
    nvim_excmds_curbuf_op_save(&raw mut orig_start, &raw mut orig_end);

    // Set '[ and '] marks to text to be operated on.
    // If not line-wise and !inclusive, also calls decl(&curbuf->b_op_end).
    nvim_opfunc_set_op_marks(oap);

    // Build argv[2] on the stack (each typval_T is 16 bytes).
    // Layout: v_type (i32) | v_lock (i32) | vval (8 bytes)
    let mut argv = [0u8; TYPVAL_SIZE * 2];
    let argv0_ptr = argv.as_mut_ptr();
    let argv1_ptr = argv.as_mut_ptr().add(TYPVAL_SIZE);

    // argv[0]: VAR_STRING with motion type string
    let motion_str: *const c_char = match (*oap).motion_type {
        K_MT_BLOCK_WISE => c"block".as_ptr(),
        K_MT_LINE_WISE => c"line".as_ptr(),
        _ => c"char".as_ptr(), // K_MT_CHAR_WISE and any unknown
    };
    // v_type = VAR_STRING at offset 0 (little-endian i32)
    argv0_ptr.cast::<c_int>().write_unaligned(VAR_STRING);
    // v_lock at offset 4 stays 0
    // vval.v_string at offset 8: the motion string pointer
    argv0_ptr
        .add(8)
        .cast::<*const c_char>()
        .write_unaligned(motion_str);

    // argv[1]: VAR_UNKNOWN (v_type = 0, rest already zeroed)
    argv1_ptr.cast::<c_int>().write_unaligned(VAR_UNKNOWN);

    // Reset virtual_op so 'virtualedit' can be changed in the callback.
    let save_virtual_op = nvim_get_virtual_op();
    nvim_set_virtual_op(K_NONE);

    // Reset finish_op so mode() returns the right value.
    let save_finish_op = nvim_dpo_get_finish_op();
    nvim_set_finish_op(false);

    // Allocate rettv on the stack and call the callback.
    let mut rettv = [0u8; TYPVAL_SIZE];
    let rettv_ptr = rettv.as_mut_ptr().cast::<c_void>();

    let cb_ptr = nvim_get_opfunc_cb();
    if callback_call(cb_ptr, 1, argv0_ptr.cast::<c_void>(), rettv_ptr) {
        tv_clear(rettv_ptr);
    }

    nvim_set_virtual_op(save_virtual_op);
    nvim_set_finish_op(save_finish_op);

    if nvim_cmdmod_has_lockmarks() != 0 {
        nvim_excmds_curbuf_op_restore(orig_start, orig_end);
    }
}
