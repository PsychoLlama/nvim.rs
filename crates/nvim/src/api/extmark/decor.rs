//! Removing marks, and the decoration provider bridge.
//!
//! `nvim_buf_del_extmark` takes one mark and `nvim_buf_clear_namespace` a
//! range's worth.  `nvim_set_decoration_provider` is the other half of the
//! family: instead of marks stored in the buffer, a set of `LuaRef` callbacks
//! the redraw loop asks for decorations per window, line and buffer.
//! `parse_virt_text` is shared with it -- the `[[text, hl], ..]` chunk array
//! decoder every virtual-text entry point uses.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::kvec::Kvec;

pub unsafe fn nvim_buf_del_extmark(
    buf: Buffer,
    ns_id: Integer,
    id: Integer,
) -> Result<Boolean, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return false.reported(error);
    }
    if !ns_initialized(ns_id as uint32_t) {
        unsafe {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            )
        };
        return false.reported(error);
    }
    unsafe { extmark_del_id(b, ns_id as uint32_t, id as uint32_t) }.reported(error)
}

pub unsafe fn nvim_buf_clear_namespace(
    buf: Buffer,
    ns_id: Integer,
    line_start: Integer,
    mut line_end: Integer,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return ().reported(error);
    }
    if !(line_start >= 0 as Integer && line_start < MAXLNUM as ::core::ffi::c_int as Integer) {
        unsafe {
            api_err_invalid(
                err,
                c"line number".as_ptr(),
                c"out of range".as_ptr(),
                0 as int64_t,
                false,
            )
        };
        return ().reported(error);
    }
    if line_end < 0 as Integer || line_end > MAXLNUM as ::core::ffi::c_int as Integer {
        line_end = MAXLNUM as ::core::ffi::c_int as Integer;
    }
    unsafe {
        extmark_clear(
            b,
            if ns_id < 0 as Integer {
                0 as uint32_t
            } else {
                ns_id as uint32_t
            },
            line_start as ::core::ffi::c_int,
            0 as colnr_T,
            line_end as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
        )
    };
    ().reported(error)
}

pub unsafe fn nvim_set_decoration_provider(
    ns_id: Integer,
    opts: *mut KeyDict_set_decoration_provider,
) {
    let mut p: *mut DecorProvider = unsafe { get_decor_provider(ns_id as NS, true) };
    debug_assert!(!p.is_null(), "p != NULL");
    unsafe { decor_provider_clear(p) };
    unsafe { redraw_all_later(UPD_NOT_VALID) };
    let mut cbs: [DecorProviderCallback; 10] = [
        DecorProviderCallback {
            name: c"on_start".as_ptr(),
            source: unsafe { &raw mut (*opts).on_start },
            dest: unsafe { &raw mut (*p).redraw_start },
        },
        DecorProviderCallback {
            name: c"on_buf".as_ptr(),
            source: unsafe { &raw mut (*opts).on_buf },
            dest: unsafe { &raw mut (*p).redraw_buf },
        },
        DecorProviderCallback {
            name: c"on_win".as_ptr(),
            source: unsafe { &raw mut (*opts).on_win },
            dest: unsafe { &raw mut (*p).redraw_win },
        },
        DecorProviderCallback {
            name: c"on_line".as_ptr(),
            source: unsafe { &raw mut (*opts).on_line },
            dest: unsafe { &raw mut (*p).redraw_line },
        },
        DecorProviderCallback {
            name: c"on_range".as_ptr(),
            source: unsafe { &raw mut (*opts).on_range },
            dest: unsafe { &raw mut (*p).redraw_range },
        },
        DecorProviderCallback {
            name: c"on_end".as_ptr(),
            source: unsafe { &raw mut (*opts).on_end },
            dest: unsafe { &raw mut (*p).redraw_end },
        },
        DecorProviderCallback {
            name: c"_on_hl_def".as_ptr(),
            source: unsafe { &raw mut (*opts)._on_hl_def },
            dest: unsafe { &raw mut (*p).hl_def },
        },
        DecorProviderCallback {
            name: c"_on_spell_nav".as_ptr(),
            source: unsafe { &raw mut (*opts)._on_spell_nav },
            dest: unsafe { &raw mut (*p).spell_nav },
        },
        DecorProviderCallback {
            name: c"_on_conceal_line".as_ptr(),
            source: unsafe { &raw mut (*opts)._on_conceal_line },
            dest: unsafe { &raw mut (*p).conceal_line },
        },
        DecorProviderCallback {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            source: ::core::ptr::null_mut::<LuaRef>(),
            dest: ::core::ptr::null_mut::<LuaRef>(),
        },
    ];
    let mut i: size_t = 0 as size_t;
    while !cbs[i as usize].source.is_null()
        && !cbs[i as usize].dest.is_null()
        && !cbs[i as usize].name.is_null()
    {
        let mut v: *mut LuaRef = cbs[i as usize].source;
        if unsafe { *v } > 0 as ::core::ffi::c_int {
            unsafe { *cbs[i as usize].dest = unsafe { *v } };
            unsafe { *v = LUA_NOREF as LuaRef };
        }
        i = i.wrapping_add(1);
    }
    unsafe { (*p).state = kDecorProviderActive };
    unsafe { (*p).hl_valid += 1 };
    unsafe { (*p).hl_cached = false };
}

pub unsafe fn parse_virt_text(
    mut chunks: Array,
    mut err: *mut Error,
    mut width: *mut ::core::ffi::c_int,
) -> VirtText {
    let mut virt_text: VirtText = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
    let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    '_free_exit: {
        while i < chunks.size {
            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                != unsafe { (*chunks.items.add(i)).type_0 } as ::core::ffi::c_uint
            {
                unsafe {
                    api_err_exp(
                        err,
                        c"chunk".as_ptr(),
                        api_typename(kObjectTypeArray),
                        api_typename((*chunks.items.add(i)).type_0),
                    )
                };
                break '_free_exit;
            }
            let mut chunk: Array = unsafe { (*chunks.items.add(i)).data.array };
            if !(chunk.size > 0 as size_t
                && chunk.size <= 2 as size_t
                && unsafe { (*chunk.items).type_0 } as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                unsafe {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"%s".as_ptr(),
                        c"Invalid chunk: expected Array with 1 or 2 Strings".as_ptr(),
                    )
                };
                break '_free_exit;
            }
            let mut str: String_0 = unsafe { (*chunk.items).data.string };
            let mut hl_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            's_146: {
                if chunk.size == 2 as size_t {
                    let mut hl: Object =
                        unsafe { *chunk.items.offset(1 as ::core::ffi::c_int as isize) };
                    if hl.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut arr: Array = unsafe { hl.data.array };
                        let mut j: size_t = 0 as size_t;
                        loop {
                            if j >= arr.size {
                                break 's_146;
                            }
                            hl_id = unsafe {
                                object_to_hl_id(
                                    *arr.items.add(j),
                                    c"virt_text highlight".as_ptr(),
                                    err,
                                )
                            };
                            if unsafe { (*err).type_0 } as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_free_exit;
                            }
                            if j < arr.size.wrapping_sub(1 as size_t) {
                                // `kv_push`, whose growth step c2rust expanded inline.
                                unsafe {
                                    Kvec::new(
                                        &mut virt_text.size,
                                        &mut virt_text.capacity,
                                        &mut virt_text.items,
                                    )
                                    .push(VirtTextChunk {
                                        text: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        hl_id,
                                    })
                                };
                            }
                            j = j.wrapping_add(1);
                        }
                    } else {
                        hl_id =
                            unsafe { object_to_hl_id(hl, c"virt_text highlight".as_ptr(), err) };
                        if unsafe { (*err).type_0 } as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_free_exit;
                        }
                    }
                }
            }
            let mut text: *mut ::core::ffi::c_char = unsafe {
                transstr(
                    if str.len() > 0 as size_t {
                        str.data() as *const ::core::ffi::c_char
                    } else {
                        c"".as_ptr()
                    },
                    false,
                )
            };
            w += unsafe { mb_string2cells(text) } as ::core::ffi::c_int;
            // `kv_push`, whose growth step c2rust expanded inline.
            unsafe {
                Kvec::new(
                    &mut virt_text.size,
                    &mut virt_text.capacity,
                    &mut virt_text.items,
                )
                .push(VirtTextChunk { text, hl_id })
            };
            i = i.wrapping_add(1);
        }
        if !width.is_null() {
            unsafe { *width = w };
        }
        return virt_text;
    }
    unsafe { clear_virttext(&raw mut virt_text) };
    virt_text
}
