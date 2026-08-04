//! The `ext_messages` emitters.
//!
//! With `ext_messages` set the message text is not drawn at all: it is
//! accumulated into highlight-coloured chunks ([`msg_ext_emit_chunk`]) and
//! handed to the UI as a `msg_show` event ([`msg_ext_ui_flush`]), which then
//! decides where to put it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn msg_ext_set_kind(mut msg_kind: *const ::core::ffi::c_char) {
    unsafe {
        msg_ext_ui_flush();
        msg_ext_kind.set(msg_kind);
        redir_col.set(if msg_ext_append.get() as ::core::ffi::c_int != 0 {
            redir_col.get()
        } else {
            0 as ::core::ffi::c_int
        });
    }
}

pub unsafe extern "C" fn msg_ext_set_append(mut append: bool) {
    unsafe {
        msg_ext_ui_flush();
        msg_ext_append.set(append);
    }
}

pub unsafe extern "C" fn msg_ext_set_trigger(mut trigger: *const ::core::ffi::c_char) {
    unsafe {
        msg_ext_ui_flush();
        msg_ext_trigger.set(trigger);
    }
}

pub(crate) unsafe extern "C" fn msg_ext_emit_chunk() {
    unsafe {
        if (*msg_ext_chunks.ptr()).is_null() {
            msg_ext_init_chunks();
        }
        if msg_ext_last_attr.get() == -1 as sattr_T {
            return;
        }
        let mut chunk: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        if chunk.size == chunk.capacity {
            chunk.capacity = if chunk.capacity != 0 {
                chunk.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            chunk.items = xrealloc(
                chunk.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh1 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: msg_ext_last_attr.get() as Integer,
            },
        };
        msg_ext_last_attr.set(-1 as ::core::ffi::c_int as sattr_T);
        let mut text: String_0 = ga_take_string(msg_ext_last_chunk.ptr());
        if chunk.size == chunk.capacity {
            chunk.capacity = if chunk.capacity != 0 {
                chunk.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            chunk.items = xrealloc(
                chunk.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh2 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 { string: text },
        };
        if chunk.size == chunk.capacity {
            chunk.capacity = if chunk.capacity != 0 {
                chunk.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            chunk.items = xrealloc(
                chunk.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh3 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: msg_ext_last_hl_id.get() as Integer,
            },
        };
        if (*msg_ext_chunks.get()).size == (*msg_ext_chunks.get()).capacity {
            (*msg_ext_chunks.get()).capacity = if (*msg_ext_chunks.get()).capacity != 0 {
                (*msg_ext_chunks.get()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*msg_ext_chunks.get()).items = xrealloc(
                (*msg_ext_chunks.get()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul((*msg_ext_chunks.get()).capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh4 = (*msg_ext_chunks.get()).size;
        (*msg_ext_chunks.get()).size = (*msg_ext_chunks.get()).size.wrapping_add(1);
        *(*msg_ext_chunks.get()).items.offset(c2rust_fresh4 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_11 { array: chunk },
        };
    }
}

pub(crate) unsafe extern "C" fn msg_ext_init_chunks() -> *mut Array {
    unsafe {
        let mut tofree: *mut Array = msg_ext_chunks.get();
        msg_ext_chunks.set(xcalloc(1 as size_t, ::core::mem::size_of::<Array>()) as *mut Array);
        msg_col.set(0 as ::core::ffi::c_int);
        return tofree;
    }
}

pub unsafe extern "C" fn msg_ext_ui_flush() {
    unsafe {
        if !ui_has(kUIMessages) {
            msg_ext_kind.set(::core::ptr::null::<::core::ffi::c_char>());
            return;
        } else if msg_ext_skip_flush.get() {
            return;
        }
        msg_ext_emit_chunk();
        if (*msg_ext_chunks.get()).size > 0 as size_t {
            let mut tofree: *mut Array = msg_ext_init_chunks();
            ui_call_msg_show(
                cstr_as_string(msg_ext_kind.get()),
                *tofree,
                msg_ext_overwrite.get() as Boolean,
                msg_ext_history.get() as Boolean,
                msg_ext_append.get() as Boolean,
                msg_ext_id.get(),
                cstr_as_string(msg_ext_trigger.get()),
            );
            if msg_ext_history.get() {
                api_free_array(*tofree);
            } else {
                let mut msg_0: HlMessage = HlMessage {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<HlMessageChunk>(),
                };
                let mut i: size_t = 0 as size_t;
                while i < (*tofree).size {
                    let mut chunk: *mut Object =
                        (*(*tofree).items.offset(i as isize)).data.array.items;
                    if msg_0.size == msg_0.capacity {
                        msg_0.capacity = if msg_0.capacity != 0 {
                            msg_0.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        msg_0.items = xrealloc(
                            msg_0.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg_0.capacity),
                        ) as *mut HlMessageChunk;
                    } else {
                    };
                    let c2rust_fresh0 = msg_0.size;
                    msg_0.size = msg_0.size.wrapping_add(1);
                    *msg_0.items.offset(c2rust_fresh0 as isize) = HlMessageChunk {
                        text: (*chunk.offset(1 as ::core::ffi::c_int as isize))
                            .data
                            .string,
                        hl_id: (*chunk.offset(2 as ::core::ffi::c_int as isize))
                            .data
                            .integer as ::core::ffi::c_int,
                    };
                    xfree(chunk as *mut ::core::ffi::c_void);
                    i = i.wrapping_add(1);
                }
                xfree((*tofree).items as *mut ::core::ffi::c_void);
                msg_hist_add_multihl(msg_0, true_0 != 0, ::core::ptr::null_mut::<MessageData>());
            }
            xfree(tofree as *mut ::core::ffi::c_void);
            msg_ext_overwrite.set(false_0 != 0);
            msg_ext_history.set(false_0 != 0);
            msg_ext_append.set(false_0 != 0);
            msg_ext_kind.set(::core::ptr::null::<::core::ffi::c_char>());
            (*msg_id_next.ptr()) += ((*msg_ext_id.ptr()).data.integer == msg_id_next.get())
                as ::core::ffi::c_int as int64_t;
            msg_ext_id.set(object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_11 {
                    integer: msg_id_next.get(),
                },
            });
        }
    }
}

pub unsafe extern "C" fn msg_ext_flush_showmode() {
    unsafe {
        static clear: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if ui_has(kUIMessages) as ::core::ffi::c_int != 0
            && (msg_ext_last_attr.get() != -1 as sattr_T || clear.get() as ::core::ffi::c_int != 0)
        {
            clear.set(msg_ext_last_attr.get() != -1 as sattr_T);
            msg_ext_emit_chunk();
            let mut tofree: *mut Array = msg_ext_init_chunks();
            ui_call_msg_showmode(*tofree);
            api_free_array(*tofree);
            xfree(tofree as *mut ::core::ffi::c_void);
        }
    }
}
