//! Remote subcommand handling
//!
//! Implements `rs_remote_request` replacing the static C function in main.c.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// Values for window_layout (must match C enums in main.c)
const WIN_TABS: c_int = 3;

// TriState kTrue value
const K_TRUE: c_int = 1;

// =============================================================================
// C API types (mirrors of api/private/defs.h structures)
// =============================================================================

/// Matches C `String` struct
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

/// Matches C `ObjectData` union
#[repr(C)]
#[derive(Clone, Copy)]
union ObjectData {
    boolean: bool,
    integer: i64,
    floating: f64,
    string: NvimString,
    array: ApiArray,
    dict: ApiDict,
}

/// ObjectType discriminants matching C kObjectType* enum
const K_OBJ_NIL: c_int = 0;
const K_OBJ_BOOLEAN: c_int = 1;
const K_OBJ_INTEGER: c_int = 2;
#[allow(dead_code)]
const K_OBJ_FLOAT: c_int = 3;
const K_OBJ_STRING: c_int = 4;
const K_OBJ_ARRAY: c_int = 5;
const K_OBJ_DICT: c_int = 6;

/// Matches C `Object` struct
#[repr(C)]
#[derive(Clone, Copy)]
struct ApiObject {
    obj_type: c_int,
    data: ObjectData,
}

impl ApiObject {
    /// Construct an Integer object
    fn integer(v: i64) -> Self {
        Self {
            obj_type: K_OBJ_INTEGER,
            data: ObjectData { integer: v },
        }
    }

    /// Construct a String object from a raw C string pointer (borrows, no ownership)
    ///
    /// # Safety
    /// `s` must be a valid NUL-terminated C string for the duration of the call.
    unsafe fn cstr(s: *const c_char) -> Self {
        let len = if s.is_null() { 0 } else { libc_strlen(s) };
        Self {
            obj_type: K_OBJ_STRING,
            data: ObjectData {
                string: NvimString {
                    data: s.cast_mut(),
                    size: len,
                },
            },
        }
    }

    /// Construct an Array object from an ApiArray
    fn array(a: ApiArray) -> Self {
        Self {
            obj_type: K_OBJ_ARRAY,
            data: ObjectData { array: a },
        }
    }
}

/// Matches C `Array` kvec struct
#[repr(C)]
#[derive(Clone, Copy)]
struct ApiArray {
    size: usize,
    capacity: usize,
    items: *mut ApiObject,
}

/// Matches C `KeyValuePair` struct
#[repr(C)]
struct KeyValuePair {
    key: NvimString,
    value: ApiObject,
}

/// Matches C `Dict` kvec struct
#[repr(C)]
#[derive(Clone, Copy)]
struct ApiDict {
    size: usize,
    capacity: usize,
    items: *mut KeyValuePair,
}

/// Matches C `Error` struct
#[repr(C)]
struct ApiError {
    /// -1 = kErrorTypeNone
    error_type: c_int,
    msg: *mut c_char,
}

impl ApiError {
    fn init() -> Self {
        Self {
            error_type: -1,
            msg: std::ptr::null_mut(),
        }
    }

    fn is_set(&self) -> bool {
        self.error_type != -1
    }
}

// =============================================================================
// FFI declarations
// =============================================================================

unsafe extern "C" {
    fn rs_server_connect(server_addr: *mut c_char, errmsg: *mut *const c_char) -> u64;
    fn os_exit(r: c_int) -> !;
    fn os_getenv_noalloc(name: *const c_char) -> *const c_char;
    fn strequal(a: *const c_char, b: *const c_char) -> bool;
    fn fprintf(stream: *mut std::ffi::c_void, fmt: *const c_char, ...) -> c_int;
    fn printf(fmt: *const c_char, ...) -> c_int;

    /// libc strlen
    #[link_name = "strlen"]
    fn libc_strlen(s: *const c_char) -> usize;

    /// Allocate memory using Neovim's allocator
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;

    /// Free C API array (kv_destroy equivalent for Array)
    fn api_free_array(arr: ApiArray);

    /// Free a C API Object (handles nested allocs)
    fn api_free_object(obj: ApiObject);

    /// Execute Lua code. `chunkname` can be NULL. `arena` can be NULL.
    // The types here are layout-compatible with the declaration in config.rs.
    #[allow(clashing_extern_declarations)]
    fn nlua_exec(
        str: NvimString,
        chunkname: *const c_char,
        args: ApiArray,
        mode: c_int,
        arena: *mut std::ffi::c_void,
        err: *mut ApiError,
    ) -> ApiObject;

    static stderr: *mut std::ffi::c_void;
    static mut ui_client_channel_id: u64;
}

/// kRetObject = 0 (from lua/executor.h LuaRetMode enum)
const K_RET_OBJECT: c_int = 0;

// =============================================================================
// cs_remote_call implementation
// =============================================================================

/// Result of the cs_remote_call Lua invocation
struct CsRemoteResult {
    /// -1=kNone, 0=kFalse, 1=kTrue
    should_exit: c_int,
    /// -1=kNone, 0=kFalse, 1=kTrue
    tabbed: c_int,
}

/// Build API args, call `vim._cs_remote` via nlua_exec, parse result Dict.
///
/// This is a Rust port of the static C function `cs_remote_call` from main.c.
/// Calls `os_exit` on any error, matching the original C behaviour.
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
unsafe fn cs_remote_call(
    chan: u64,
    server_addr: *const c_char,
    connect_error: *const c_char,
    argc: c_int,
    argv: *mut *mut c_char,
    remote_args: c_int,
) -> CsRemoteResult {
    // Build the args array (heap-allocated, mirrors `kv_resize` + `ADD_C`)
    let n_args = (argc - remote_args) as usize;
    let args_items = xmalloc(n_args * std::mem::size_of::<ApiObject>()).cast::<ApiObject>();
    let mut args = ApiArray {
        size: 0,
        capacity: n_args,
        items: args_items,
    };
    for t_argc in remote_args..argc {
        let s = *argv.add(t_argc as usize);
        *args.items.add(args.size) = ApiObject::cstr(s);
        args.size += 1;
    }

    // Build the 4-element stack array (mirrors MAXSIZE_TEMP_ARRAY(a, 4))
    let mut a_items: [ApiObject; 4] = [ApiObject {
        obj_type: K_OBJ_NIL,
        data: ObjectData { integer: 0 },
    }; 4];
    let mut a = ApiArray {
        size: 0,
        capacity: 4,
        items: a_items.as_mut_ptr(),
    };
    // ADD_C(a, INTEGER_OBJ((int)chan))
    *a.items.add(a.size) = ApiObject::integer(chan as i64);
    a.size += 1;
    // ADD_C(a, CSTR_AS_OBJ(server_addr))
    *a.items.add(a.size) = ApiObject::cstr(server_addr);
    a.size += 1;
    // ADD_C(a, CSTR_AS_OBJ(connect_error))
    *a.items.add(a.size) = ApiObject::cstr(connect_error);
    a.size += 1;
    // ADD_C(a, ARRAY_OBJ(args))
    *a.items.add(a.size) = ApiObject::array(args);
    a.size += 1;

    // STATIC_CSTR_AS_STRING("return vim._cs_remote(...)")
    let code = c"return vim._cs_remote(...)";
    let s = NvimString {
        data: code.as_ptr().cast_mut(),
        size: code.to_bytes().len(),
    };

    let mut err = ApiError::init();
    let o = nlua_exec(
        s,
        std::ptr::null(),
        a,
        K_RET_OBJECT,
        std::ptr::null_mut(),
        &mut err,
    );
    // Free the heap-allocated args array
    api_free_array(args);

    if err.is_set() {
        fprintf(stderr, c"%s\n".as_ptr(), err.msg);
        os_exit(2);
    }

    if o.obj_type != K_OBJ_DICT {
        fprintf(
            stderr,
            c"vim._cs_remote returned unexpected value\n".as_ptr(),
        );
        os_exit(2);
    }

    let mut result = CsRemoteResult {
        should_exit: -1,
        tabbed: -1,
    };

    let rvdict = o.data.dict;
    for i in 0..rvdict.size {
        let pair = &*rvdict.items.add(i);
        let key = pair.key.data;
        if strequal(key, c"errmsg".as_ptr()) {
            if pair.value.obj_type != K_OBJ_STRING {
                fprintf(
                    stderr,
                    c"vim._cs_remote returned an unexpected type for 'errmsg'\n".as_ptr(),
                );
                os_exit(2);
            }
            fprintf(stderr, c"%s\n".as_ptr(), pair.value.data.string.data);
            os_exit(2);
        } else if strequal(key, c"result".as_ptr()) {
            if pair.value.obj_type != K_OBJ_STRING {
                fprintf(
                    stderr,
                    c"vim._cs_remote returned an unexpected type for 'result'\n".as_ptr(),
                );
                os_exit(2);
            }
            printf(c"%s".as_ptr(), pair.value.data.string.data);
        } else if strequal(key, c"tabbed".as_ptr()) {
            if pair.value.obj_type != K_OBJ_BOOLEAN {
                fprintf(
                    stderr,
                    c"vim._cs_remote returned an unexpected type for 'tabbed'\n".as_ptr(),
                );
                os_exit(2);
            }
            result.tabbed = c_int::from(pair.value.data.boolean);
        } else if strequal(key, c"should_exit".as_ptr()) {
            if pair.value.obj_type != K_OBJ_BOOLEAN {
                fprintf(
                    stderr,
                    c"vim._cs_remote returned an unexpected type for 'should_exit'\n".as_ptr(),
                );
                os_exit(2);
            }
            result.should_exit = c_int::from(pair.value.data.boolean);
        }
    }

    if result.should_exit == -1 || result.tabbed == -1 {
        fprintf(
            stderr,
            c"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n".as_ptr(),
        );
        os_exit(2);
    }

    api_free_object(o);
    result
}

// =============================================================================
// Public entry point
// =============================================================================

/// Handle remote subcommands.
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rs_remote_request(
    params: *mut MparmT,
    remote_args: c_int,
    server_addr: *mut c_char,
    argc: c_int,
    argv: *mut *mut c_char,
    ui_only: bool,
) {
    let is_ui = strequal(*argv.add(remote_args as usize), c"--remote-ui".as_ptr());

    if ui_only && !is_ui {
        // TODO(bfredl): this implies always starting the TUI.
        return;
    }

    let mut connect_error: *const c_char = std::ptr::null();
    let chan = rs_server_connect(server_addr, &mut connect_error);

    if is_ui {
        if chan == 0 {
            fprintf(
                stderr,
                c"Remote ui failed to start: %s\n".as_ptr(),
                connect_error,
            );
            os_exit(1);
        } else {
            let nvim_env = os_getenv_noalloc(c"NVIM".as_ptr());
            if strequal(server_addr, nvim_env) {
                fprintf(
                    stderr,
                    c"%s".as_ptr(),
                    c"Cannot attach UI of :terminal child to its parent. ".as_ptr(),
                );
                fprintf(
                    stderr,
                    c"%s\n".as_ptr(),
                    c"(Unset $NVIM to skip this check)".as_ptr(),
                );
                os_exit(1);
            }
        }
        ui_client_channel_id = chan;
        return;
    }

    // Call vim._cs_remote and parse the result
    let r = cs_remote_call(chan, server_addr, connect_error, argc, argv, remote_args);
    if r.should_exit == K_TRUE {
        os_exit(0);
    }
    if r.tabbed == K_TRUE {
        let p = &mut *params;
        p.window_count = argc - remote_args - 1;
        p.window_layout = WIN_TABS;
    }
}
