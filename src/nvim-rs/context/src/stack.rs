use std::mem;

use crate::{ctx_stack, ffi, rs_ctx_free, Context};

/// Push a zero-initialized Context onto the stack (mirrors `kv_push` with `CONTEXT_INIT`).
///
/// Grows the backing array if needed using `xrealloc` (matches `kv_resize_full`).
unsafe fn ctx_stack_push_init() {
    if ctx_stack.size == ctx_stack.capacity {
        // kv_resize_full: double capacity (or start at 8)
        let new_cap = if ctx_stack.capacity == 0 {
            8
        } else {
            ctx_stack.capacity * 2
        };
        ctx_stack.items =
            ffi::xrealloc(ctx_stack.items.cast(), new_cap * mem::size_of::<Context>()).cast();
        ctx_stack.capacity = new_cap;
    }
    // Write CONTEXT_INIT (all-zero String/Array values) into items[size]
    ctx_stack.items.add(ctx_stack.size).write(mem::zeroed());
    ctx_stack.size += 1;
}

#[export_name = "ctx_free_all"]
pub unsafe extern "C" fn rs_ctx_free_all() {
    for i in 0..ctx_stack.size {
        let ctx = ctx_stack.items.add(i);
        if !ctx.is_null() {
            rs_ctx_free(ctx);
        }
    }
    // kv_destroy: xfree items and reset
    ffi::xfree(ctx_stack.items.cast());
    ctx_stack.size = 0;
    ctx_stack.capacity = 0;
    ctx_stack.items = std::ptr::null_mut();
}

/// Push a new (zero-init) context, optionally returning a pointer to the caller-supplied ctx.
/// Used internally by save/restore.
pub unsafe fn ctx_stack_push_or_ptr(ctx: *mut Context) -> *mut Context {
    if ctx.is_null() {
        ctx_stack_push_init();
        // last element: items[size - 1]
        ctx_stack.items.add(ctx_stack.size - 1)
    } else {
        ctx
    }
}

/// Pop the top context and return a pointer to the (still-allocated) slot.
pub unsafe fn ctx_stack_pop_slot() -> *mut Context {
    ctx_stack.size -= 1;
    ctx_stack.items.add(ctx_stack.size)
}
