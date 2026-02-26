//! Callback dispatch -- callback_call migration.
//!
//! Migrated from `eval_shim.c` Phase 4. Updated Phase 12 to use CallbackT directly.
//!
//! Handles funcref, partial, lua, and none callback types.

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, Ordering};

use crate::eval::TypevalHandle;
use crate::funcexe::FuncExeT;

// =============================================================================
// CallbackT: #[repr(C)] mirror of C's Callback struct
//
// C definition (eval/typval_defs.h):
//   typedef struct {
//     union {
//       char *funcref;           // offset 0, size 8
//       partial_T *partial;      // offset 0, size 8
//       LuaRef luaref;           // offset 0, size 4 (but union is 8)
//     } data;
//     CallbackType type;         // offset 8, size 4
//   } Callback;                  // total: 16 bytes (4 bytes trailing pad)
//
// LuaRef = int (4 bytes), so union size = max(8,8,4) = 8 bytes.
// CallbackType is a C enum (int = 4 bytes), padded to align at 4.
// sizeof(Callback) = 8 + 4 + 4 (padding) = 16.
// =============================================================================

/// Union data field of Callback struct (8 bytes).
///
/// # Safety
/// Access to any variant is unsafe; caller must match the `cb_type` field.
#[repr(C)]
pub union CallbackData {
    /// kCallbackFuncref: heap-allocated function name string.
    pub funcref: *mut c_char,
    /// kCallbackPartial: pointer to partial_T.
    pub partial: *mut c_void,
    /// kCallbackLua: Lua registry reference (LuaRef = int, stored in 8-byte slot).
    pub luaref: c_int,
}

/// Rust mirror of C `Callback` struct.
///
/// Must have exactly the same layout as C's `Callback` (verified by
/// `_Static_assert` in `eval_shim.c`).
#[repr(C)]
pub struct CallbackT {
    /// Union data (funcref / partial / luaref).
    pub data: CallbackData,
    /// Which variant is active (kCallbackNone/Funcref/Partial/Lua).
    pub cb_type: c_int,
    // 4 bytes trailing padding (implicit, added by repr(C) to reach sizeof = 16)
}

// =============================================================================
// Constants (matching C kCallbackXxx enum)
// =============================================================================

pub const K_CALLBACK_NONE: c_int = 0;
pub const K_CALLBACK_FUNCREF: c_int = 1;
pub const K_CALLBACK_PARTIAL: c_int = 2;
pub const K_CALLBACK_LUA: c_int = 3;

// =============================================================================
// callback_depth: Rust-owned static (was C static in eval_shim.c)
// =============================================================================

/// Tracks recursive callback call depth.
/// Was `static int callback_depth` in eval_shim.c.
static CALLBACK_DEPTH: AtomicI32 = AtomicI32::new(0);

/// Return current callback_depth.
///
/// # Safety
/// No special requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_get_callback_depth() -> c_int {
    CALLBACK_DEPTH.load(Ordering::Relaxed)
}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Error message
    fn emsg(s: *const c_char) -> c_int;

    // p_mfd option (max function depth)
    fn nvim_p_mfd_get() -> c_int;

    // VV_LUA partial getter (canonical name)
    fn nvim_get_vlua_partial() -> *mut c_void;

    // rs_check_luafunc_name: already Rust
    fn rs_check_luafunc_name(str_: *const c_char, paren: bool) -> c_int;

    // nvim_callback_call_lua: retained in C (uses LUARET_TRUTHY macro)
    fn nvim_callback_call_lua(luaref: c_int) -> bool;

    // Direct call_func (used with FuncExeT)
    fn call_func(
        funcname: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        funcexe: *mut FuncExeT,
    ) -> c_int;

    // Cursor position accessor
    fn nvim_curwin_get_cursor_lnum() -> i32;

    // rs_partial_name (already Rust, declared in eval.rs as const)
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;

    // callback_free and callback_put (C functions, called directly from Rust)
    fn callback_free(cb: *mut c_void);
    fn callback_put(cb: *mut c_void, tv: *mut c_void);
}

// Error message
static E_CMD_TOO_RECURSIVE: &[u8] = b"E169: Command too recursive\0";

// =============================================================================
// Depth management
// =============================================================================

/// Check if callback_depth > p_mfd.
///
/// Inlines C `nvim_callback_depth_exceeded`.
#[inline]
unsafe fn callback_depth_exceeded() -> bool {
    CALLBACK_DEPTH.load(Ordering::Relaxed) > nvim_p_mfd_get()
}

/// Increment callback_depth by 1.
#[inline]
fn callback_depth_inc() {
    CALLBACK_DEPTH.fetch_add(1, Ordering::Relaxed);
}

/// Decrement callback_depth by 1.
#[inline]
fn callback_depth_dec() {
    CALLBACK_DEPTH.fetch_sub(1, Ordering::Relaxed);
}

// =============================================================================
// v:lua funcref check (inlined from nvim_cb_check_vlua_funcref)
// =============================================================================

/// If `name` starts with "v:lua." and the remainder is a valid Lua func name,
/// return a pointer to the Lua function name portion (name + 6).
/// Otherwise return null.
///
/// Inlines C `nvim_cb_check_vlua_funcref`.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[inline]
unsafe fn check_vlua_funcref(name: *const c_char) -> *const c_char {
    let prefix = b"v:lua.\0";
    let mut p = name;
    let mut q = prefix.as_ptr() as *const c_char;
    // compare first 6 bytes "v:lua."
    for _ in 0..6 {
        if *p != *q {
            return std::ptr::null();
        }
        p = p.add(1);
        q = q.add(1);
    }
    // p now points to the Lua function name portion
    let lualen = rs_check_luafunc_name(p, false);
    if lualen == 0 {
        return std::ptr::null();
    }
    p
}

// =============================================================================
// Lua callback call (inlined from nvim_callback_call_lua)
// =============================================================================

/// Handle kCallbackLua: call the Lua ref with no arguments.
///
/// Delegates to C `nvim_callback_call_lua` which uses `LUARET_TRUTHY` macro.
///
/// # Safety
/// `luaref` must be a valid Lua registry reference.
#[inline]
unsafe fn callback_call_lua(luaref: c_int) -> bool {
    nvim_callback_call_lua(luaref)
}

// =============================================================================
// nvim_callback_free / nvim_callback_put wrappers (Rust-side)
// =============================================================================

/// Wrap callback_free(cb) for use from Rust callers.
/// Accepts *mut c_void to be compatible with all callers (timer.rs uses opaque handle).
///
/// # Safety
/// `cb` must be a valid Callback pointer.
#[no_mangle]
pub unsafe extern "C" fn nvim_callback_free(cb: *mut c_void) {
    callback_free(cb);
}

/// Wrap callback_put(cb, tv) for use from Rust callers.
///
/// # Safety
/// `cb` and `tv` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn nvim_callback_put(cb: *mut c_void, tv: *mut c_void) {
    callback_put(cb, tv);
}

// =============================================================================
// callback_call
// =============================================================================

/// Core callback dispatch: handles funcref, partial, lua, and none callback types.
///
/// Migrated from C `callback_call` in eval_shim.c. Updated to use CallbackT directly.
///
/// # Safety
/// - `callback` must be a valid `CallbackT *`
/// - `argvars` must be a valid `typval_T[argcount]` or null if argcount == 0
/// - `rettv` must be a valid typval handle
pub unsafe fn callback_call_impl(
    callback: *const CallbackT,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: TypevalHandle,
) -> bool {
    if callback_depth_exceeded() {
        emsg(E_CMD_TOO_RECURSIVE.as_ptr() as *const c_char);
        return false;
    }

    let cb_type = (*callback).cb_type;

    match cb_type {
        K_CALLBACK_FUNCREF => {
            let name = (*callback).data.funcref;
            // Check if this is a v:lua.* funcref
            let lua_name = check_vlua_funcref(name);
            let (call_name, partial) = if !lua_name.is_null() {
                let vv_lua_partial = nvim_get_vlua_partial();
                (lua_name, vv_lua_partial)
            } else {
                (name as *const c_char, std::ptr::null_mut())
            };
            callback_depth_inc();
            let lnum = nvim_curwin_get_cursor_lnum();
            let mut funcexe = FuncExeT::new();
            funcexe.fe_firstline = lnum;
            funcexe.fe_lastline = lnum;
            funcexe.fe_evaluate = true;
            funcexe.fe_partial = partial;
            let ret = call_func(
                call_name,
                -1,
                rettv.as_ptr(),
                argcount,
                argvars,
                &mut funcexe,
            ) != 0;
            callback_depth_dec();
            ret
        }
        K_CALLBACK_PARTIAL => {
            let partial = (*callback).data.partial;
            let name = rs_partial_name(partial as *const c_void);
            callback_depth_inc();
            let lnum = nvim_curwin_get_cursor_lnum();
            let mut funcexe = FuncExeT::new();
            funcexe.fe_firstline = lnum;
            funcexe.fe_lastline = lnum;
            funcexe.fe_evaluate = true;
            funcexe.fe_partial = partial;
            let ret = call_func(name, -1, rettv.as_ptr(), argcount, argvars, &mut funcexe) != 0;
            callback_depth_dec();
            ret
        }
        K_CALLBACK_LUA => {
            let luaref = (*callback).data.luaref;
            callback_call_lua(luaref)
        }
        K_CALLBACK_NONE => false,
        _ => false,
    }
}

/// FFI export for callback_call.
///
/// # Safety
/// See `callback_call_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn callback_call(
    callback: *const CallbackT,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: TypevalHandle,
) -> bool {
    callback_call_impl(callback, argcount, argvars, rettv)
}
