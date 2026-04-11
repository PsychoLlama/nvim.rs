//! Search path cache management
//!
//! Manages the runtime search path cache, which pre-computes and caches
//! expanded runtime and package paths for fast file lookup.
//!
//! The actual data structure operations (kvec, Map, Set) remain in C.
//! This module handles the cache lifecycle: validation, reference counting,
//! and invalidation.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque C Types
// =============================================================================

/// Opaque handle to optset_T
#[repr(C)]
pub struct OptsetT {
    _private: [u8; 0],
}

/// Opaque Arena type from C
#[repr(C)]
pub struct Arena {
    _private: [u8; 0],
}

/// NvimString matching C's String struct
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

/// Object union data (matches C union)
#[repr(C)]
#[derive(Clone, Copy)]
pub union ObjectData {
    pub boolean: bool,
    pub integer: i64,
    pub floating: f64,
    pub string: NvimString,
    pub array: Array,
}

/// Object struct matching C definition
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object {
    pub obj_type: c_int,
    pub data: ObjectData,
}

/// Array struct matching C definition (kvec)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Array {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut Object,
}

// =============================================================================
// Constants
// =============================================================================

const MAXPATHL: usize = 4096;

/// TriState: kNone=-1, kFalse=0, kTrue=1
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;
const K_TRUE: c_int = 1;

/// kObjectTypeString = 4
const K_OBJECT_TYPE_STRING: c_int = 4;
/// kObjectTypeBoolean = 1
const K_OBJECT_TYPE_BOOLEAN: c_int = 1;
/// kObjectTypeArray = 5
const K_OBJECT_TYPE_ARRAY: c_int = 5;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Global state accessors (in runtime_ffi.c)
    fn nvim_rt_sp_get_valid() -> bool;
    fn nvim_rt_sp_set_valid(valid: bool);
    fn nvim_rt_sp_get_ref() -> *mut c_int;
    fn nvim_rt_sp_set_ref(ref_ptr: *mut c_int);
    fn nvim_rt_sp_mutex_init();
    fn nvim_rt_sp_mutex_lock();
    fn nvim_rt_sp_mutex_unlock();

    // C functions that operate on the global RuntimeSearchPath directly
    fn nvim_rt_sp_build_and_set();
    fn nvim_rt_sp_free_path();
    fn nvim_rt_sp_copy_to_thread();

    // Deferred-safe check
    fn nvim_rt_nlua_is_deferred_safe() -> bool;

    // Global (non-cached) path accessors (Phase 1: for runtime_inspect)
    fn nvim_rt_rsp_get_path_size() -> usize;
    fn nvim_rt_rsp_get_path_item_path(idx: usize) -> *const c_char;
    fn nvim_rt_rsp_get_path_item_after(idx: usize) -> bool;
    fn nvim_rt_rsp_get_path_item_has_lua(idx: usize) -> c_int;

    // Cached path accessors (Phase 1: for runtime_get_named_common)
    fn nvim_rt_rsp_get_cached_size(ref_out: *mut c_int) -> usize;
    fn nvim_rt_rsp_get_item_path(idx: usize) -> *const c_char;
    fn nvim_rt_rsp_get_item_has_lua(idx: usize) -> c_int;
    fn nvim_rt_rsp_set_item_has_lua(idx: usize, val: c_int);
    fn nvim_rt_rsp_unref(ref_ptr: *const c_int);

    // Thread-safe path accessors (Phase 1: for runtime_get_named_thread)
    fn nvim_rt_rsp_get_thread_size() -> usize;
    fn nvim_rt_rsp_get_thread_item_path(idx: usize) -> *const c_char;
    fn nvim_rt_rsp_get_thread_item_has_lua(idx: usize) -> c_int;
    fn nvim_rt_rsp_set_thread_item_has_lua(idx: usize, val: c_int);

    // OS filesystem functions (implemented in nvim-os crate)
    fn os_isdir(path: *const c_char) -> bool;
    fn os_file_is_readable(path: *const c_char) -> bool;

    // Arena allocation
    fn arena_alloc(arena: *mut Arena, size: usize) -> *mut c_void;
    fn arena_memdupz(arena: *mut Arena, data: *const c_char, len: usize) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_void;

    // libc snprintf (variadic)
    fn snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
}

// =============================================================================
// Helpers
// =============================================================================

/// Allocate an Array with the given capacity in arena (or heap if arena is null).
#[allow(clippy::cast_ptr_alignment)]
unsafe fn arena_array_alloc(arena: *mut Arena, max_size: usize) -> Array {
    let item_size = std::mem::size_of::<Object>();
    let raw = if arena.is_null() {
        xmalloc(max_size * item_size)
    } else {
        arena_alloc(arena, max_size * item_size)
    };
    Array {
        size: 0,
        capacity: max_size,
        items: raw.cast::<Object>(),
    }
}

/// Push an Object into an Array.
/// # Safety
/// The array must have sufficient capacity.
unsafe fn array_push(arr: &mut Array, item: Object) {
    debug_assert!(arr.size < arr.capacity);
    arr.items.add(arr.size).write(item);
    arr.size += 1;
}

/// Create a Boolean Object.
fn boolean_obj(b: bool) -> Object {
    Object {
        obj_type: K_OBJECT_TYPE_BOOLEAN,
        data: ObjectData { boolean: b },
    }
}

/// Create an Array Object.
fn array_obj(a: Array) -> Object {
    Object {
        obj_type: K_OBJECT_TYPE_ARRAY,
        data: ObjectData { array: a },
    }
}

/// Create a String Object from a C string, allocated in arena.
/// Equivalent to C's CSTR_TO_ARENA_OBJ macro.
unsafe fn cstr_to_arena_obj(arena: *mut Arena, s: *const c_char) -> Object {
    let len = cstr_len(s);
    let data = arena_memdupz(arena, s, len);
    let str = NvimString { data, size: len };
    Object {
        obj_type: K_OBJECT_TYPE_STRING,
        data: ObjectData { string: str },
    }
}

/// Compute the length of a C string.
unsafe fn cstr_len(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// Public FFI Functions
// =============================================================================

/// Initialize runtime search path system.
///
/// Called once during startup to initialize the mutex.
#[export_name = "runtime_init"]
pub unsafe extern "C" fn rs_runtime_init() {
    nvim_rt_sp_mutex_init();
}

/// Callback for 'runtimepath' or 'packpath' option change.
///
/// Invalidates the cached search path so it gets rebuilt on next use.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_runtimepackpath(_args: *mut OptsetT) -> *const c_char {
    nvim_rt_sp_set_valid(false);
    std::ptr::null()
}

/// Validate and rebuild runtime search path if needed.
///
/// If the cache is invalid, rebuilds it from `p_rtp` and `p_pp`, then
/// copies the result to the thread-safe copy under mutex protection.
#[export_name = "runtime_search_path_validate"]
pub unsafe extern "C" fn rs_runtime_search_path_validate() {
    if !nvim_rt_nlua_is_deferred_safe() {
        // Cannot rebuild search path in an async context. Prefer stale cache
        // over erroring out, as the plugin will likely already have the
        // sought module in the cached path.
        return;
    }

    if !nvim_rt_sp_get_valid() {
        let ref_ptr = nvim_rt_sp_get_ref();
        if ref_ptr.is_null() {
            // No one holds a reference, safe to free
            nvim_rt_sp_free_path();
        }

        // Build new search path and store it in the global
        nvim_rt_sp_build_and_set();
        nvim_rt_sp_set_valid(true);
        nvim_rt_sp_set_ref(std::ptr::null_mut()); // initially unowned

        // Update thread-safe copy
        nvim_rt_sp_mutex_lock();
        nvim_rt_sp_copy_to_thread();
        nvim_rt_sp_mutex_unlock();
    }
}

/// Get cached search path with reference counting.
///
/// This is called from C code that works with RuntimeSearchPath values.
/// The function validates the cache and manages reference counting.
///
/// The `ref_out` parameter tracks whether the caller holds a reference.
/// If no one else held a reference, this caller becomes the ref holder.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_search_path_get_cached(ref_out: *mut c_int) {
    rs_runtime_search_path_validate();

    *ref_out = 0;
    if nvim_rt_sp_get_ref().is_null() {
        // Cached path was unreferenced. Keep a ref to prevent
        // runtime_search_path_validate() from freeing it too early.
        *ref_out = 1;
        nvim_rt_sp_set_ref(ref_out);
    }
}

/// Release reference to runtime search path.
///
/// Called from C code when done using a cached search path.
/// `ref_ptr` is the reference counter from `runtime_search_path_get_cached`.
/// If we're the current holder, just release. Otherwise, free the path copy.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_search_path_unref(ref_ptr: *const c_int) -> bool {
    if ref_ptr.is_null() || *ref_ptr == 0 {
        return false;
    }

    let current_ref = nvim_rt_sp_get_ref();
    if std::ptr::eq(current_ref, ref_ptr.cast_mut()) {
        nvim_rt_sp_set_ref(std::ptr::null_mut());
        false // caller should NOT free the path
    } else {
        true // caller should free the path
    }
}

// =============================================================================
// Phase 1: runtime_inspect, runtime_get_named, runtime_get_named_thread
// =============================================================================

/// Build Array of runtime search path entries for API inspection.
///
/// Equivalent to C's `runtime_inspect`.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_inspect(arena: *mut Arena) -> Array {
    let path_size = nvim_rt_rsp_get_path_size();
    let mut rv = arena_array_alloc(arena, path_size);

    for i in 0..path_size {
        let item_path = nvim_rt_rsp_get_path_item_path(i);
        let item_after = nvim_rt_rsp_get_path_item_after(i);
        let item_has_lua = nvim_rt_rsp_get_path_item_has_lua(i);

        let mut entry = arena_array_alloc(arena, 3);
        array_push(&mut entry, cstr_to_arena_obj(arena, item_path));
        array_push(&mut entry, boolean_obj(item_after));
        if item_has_lua != K_NONE {
            array_push(&mut entry, boolean_obj(item_has_lua == K_TRUE));
        }
        array_push(&mut rv, array_obj(entry));
    }
    rv
}

/// Find named files in the cached search path.
///
/// Equivalent to C's `runtime_get_named`.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_get_named(
    lua: bool,
    pat: Array,
    all: bool,
    arena: *mut Arena,
) -> Array {
    let mut ref_val: c_int = 0;
    let path_size = nvim_rt_rsp_get_cached_size(std::ptr::addr_of_mut!(ref_val));

    let mut buf = [0u8; MAXPATHL];
    let rv = runtime_get_named_common(
        lua,
        pat,
        all,
        path_size,
        buf.as_mut_ptr().cast::<c_char>(),
        MAXPATHL,
        arena,
        false, // use cached path
    );

    nvim_rt_rsp_unref(std::ptr::addr_of!(ref_val));
    rv
}

/// Thread-safe variant of runtime_get_named.
///
/// Equivalent to C's `runtime_get_named_thread`.
#[no_mangle]
pub unsafe extern "C" fn rs_runtime_get_named_thread(lua: bool, pat: Array, all: bool) -> Array {
    nvim_rt_sp_mutex_lock();
    let path_size = nvim_rt_rsp_get_thread_size();
    let mut buf = [0u8; MAXPATHL];
    let rv = runtime_get_named_common(
        lua,
        pat,
        all,
        path_size,
        buf.as_mut_ptr().cast::<c_char>(),
        MAXPATHL,
        std::ptr::null_mut(), // NULL arena: allocate on heap
        true,                 // use thread path
    );
    nvim_rt_sp_mutex_unlock();
    rv
}

/// Context for iterating a path in `runtime_get_named_common`.
struct PathCtx {
    use_thread: bool,
}

impl PathCtx {
    fn get_item_path(&self, i: usize) -> *const c_char {
        unsafe {
            if self.use_thread {
                nvim_rt_rsp_get_thread_item_path(i)
            } else {
                nvim_rt_rsp_get_item_path(i)
            }
        }
    }

    fn get_item_has_lua(&self, i: usize) -> c_int {
        unsafe {
            if self.use_thread {
                nvim_rt_rsp_get_thread_item_has_lua(i)
            } else {
                nvim_rt_rsp_get_item_has_lua(i)
            }
        }
    }

    fn set_item_has_lua(&self, i: usize, val: c_int) {
        unsafe {
            if self.use_thread {
                nvim_rt_rsp_set_thread_item_has_lua(i, val);
            } else {
                nvim_rt_rsp_set_item_has_lua(i, val);
            }
        }
    }
}

/// Shared implementation for runtime_get_named and runtime_get_named_thread.
///
/// `use_thread` selects which path to iterate: thread-safe copy or cached copy.
#[allow(clippy::too_many_arguments)]
unsafe fn runtime_get_named_common(
    lua: bool,
    pat: Array,
    all: bool,
    path_size: usize,
    buf: *mut c_char,
    buf_len: usize,
    arena: *mut Arena,
    use_thread: bool,
) -> Array {
    let ctx = PathCtx { use_thread };
    let mut rv = arena_array_alloc(arena, path_size * pat.size);
    'outer: for i in 0..path_size {
        let item_path = ctx.get_item_path(i);

        if lua {
            let has_lua = ctx.get_item_has_lua(i);
            let has_lua = if has_lua == K_NONE {
                // Check and cache whether this path has a lua/ subdirectory
                let size = snprintf(buf, buf_len, c"%s/lua/".as_ptr(), item_path) as usize;
                let result = if size < buf_len && os_isdir(buf) {
                    K_TRUE
                } else {
                    K_FALSE
                };
                ctx.set_item_has_lua(i, result);
                result
            } else {
                has_lua
            };

            if has_lua == K_FALSE {
                continue;
            }
        }

        for j in 0..pat.size {
            let pat_item = pat.items.add(j).read();
            if pat_item.obj_type == K_OBJECT_TYPE_STRING {
                let pat_str = pat_item.data.string;
                let size =
                    snprintf(buf, buf_len, c"%s/%s".as_ptr(), item_path, pat_str.data) as usize;
                if size < buf_len && os_file_is_readable(buf) {
                    array_push(&mut rv, cstr_to_arena_obj(arena, buf));
                    if !all {
                        break 'outer;
                    }
                }
            }
        }
    }
    rv
}
