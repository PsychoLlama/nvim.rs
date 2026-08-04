//! The `ext_cmdline` UI: the command line as events rather than a grid.
//!
//! [`ui_ext_cmdline_show`] sends `cmdline_show` with the line, the cursor
//! position and the colour chunks; [`ui_ext_cmdline_block_append`] is the
//! multi-line block a `:if`/`:function` body builds up.  A UI that took over
//! the command line gets these instead of anything [`super::draw`] writes.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ui_ext_cmdline_show(mut line: *mut CmdlineInfo) {
    unsafe {
        let mut arena: Arena = ARENA_EMPTY;
        let mut content: Array = Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        };
        if cmdline_star.get() != 0 {
            content = arena_array(&raw mut arena, 1 as size_t);
            let mut len: size_t = 0 as size_t;
            let mut p: *mut ::core::ffi::c_char = (*ccline.ptr()).cmdbuff;
            while *p != 0 {
                len = len.wrapping_add(1);
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            let mut buf: *mut ::core::ffi::c_char =
                arena_alloc(&raw mut arena, len, false_0 != 0) as *mut ::core::ffi::c_char;
            memset(
                buf as *mut ::core::ffi::c_void,
                '*' as ::core::ffi::c_int,
                len,
            );
            let mut item: Array = arena_array(&raw mut arena, 3 as size_t);
            let c2rust_fresh17 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh17 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh18 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh18 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: String_0 {
                        data: buf,
                        size: len,
                    },
                },
            };
            let c2rust_fresh19 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh19 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh20 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.offset(c2rust_fresh20 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: item },
            };
        } else if (*line).last_colors.colors.size != 0 {
            content = arena_array(&raw mut arena, (*line).last_colors.colors.size);
            let mut i: size_t = 0 as size_t;
            while i < (*line).last_colors.colors.size {
                let mut chunk: CmdlineColorChunk =
                    *(*line).last_colors.colors.items.offset(i as isize);
                let mut item_0: Array = arena_array(&raw mut arena, 3 as size_t);
                let c2rust_fresh21 = item_0.size;
                item_0.size = item_0.size.wrapping_add(1);
                *item_0.items.offset(c2rust_fresh21 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (if chunk.hl_id == 0 as ::core::ffi::c_int {
                            0 as ::core::ffi::c_int
                        } else {
                            syn_id2attr(chunk.hl_id)
                        }) as Integer,
                    },
                };
                '_c2rust_label: {
                    if chunk.end >= chunk.start {
                    } else {
                        __assert_fail(
                            b"chunk.end >= chunk.start\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ex_getln.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3627 as ::core::ffi::c_uint,
                            b"void ui_ext_cmdline_show(CmdlineInfo *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let c2rust_fresh22 = item_0.size;
                item_0.size = item_0.size.wrapping_add(1);
                *item_0.items.offset(c2rust_fresh22 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: (*line).cmdbuff.offset(chunk.start as isize),
                            size: (chunk.end - chunk.start) as size_t,
                        },
                    },
                };
                let c2rust_fresh23 = item_0.size;
                item_0.size = item_0.size.wrapping_add(1);
                *item_0.items.offset(c2rust_fresh23 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: chunk.hl_id as Integer,
                    },
                };
                let c2rust_fresh24 = content.size;
                content.size = content.size.wrapping_add(1);
                *content.items.offset(c2rust_fresh24 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: item_0 },
                };
                i = i.wrapping_add(1);
            }
        } else {
            let mut item_1: Array = arena_array(&raw mut arena, 3 as size_t);
            let c2rust_fresh25 = item_1.size;
            item_1.size = item_1.size.wrapping_add(1);
            *item_1.items.offset(c2rust_fresh25 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh26 = item_1.size;
            item_1.size = item_1.size.wrapping_add(1);
            *item_1.items.offset(c2rust_fresh26 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*line).cmdbuff),
                },
            };
            let c2rust_fresh27 = item_1.size;
            item_1.size = item_1.size.wrapping_add(1);
            *item_1.items.offset(c2rust_fresh27 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 0 as Integer,
                },
            };
            content = arena_array(&raw mut arena, 1 as size_t);
            let c2rust_fresh28 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.offset(c2rust_fresh28 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: item_1 },
            };
        }
        let mut charbuf: [::core::ffi::c_char; 2] = [
            (*line).cmdfirstc as ::core::ffi::c_char,
            0 as ::core::ffi::c_char,
        ];
        ui_call_cmdline_show(
            content,
            (*line).cmdpos as Integer,
            cstr_as_string(&raw mut charbuf as *mut ::core::ffi::c_char),
            cstr_as_string((*line).cmdprompt),
            (*line).cmdindent as Integer,
            (*line).level as Integer,
            (*line).hl_id as Integer,
        );
        if (*line).special_char != 0 {
            charbuf[0 as ::core::ffi::c_int as usize] = (*line).special_char;
            ui_call_cmdline_special_char(
                cstr_as_string(&raw mut charbuf as *mut ::core::ffi::c_char),
                (*line).special_shift as Boolean,
                (*line).level as Integer,
            );
        }
        arena_mem_free(arena_finish(&raw mut arena));
    }
}

pub unsafe extern "C" fn ui_ext_cmdline_block_append(
    mut indent: size_t,
    mut line: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut buf: *mut ::core::ffi::c_char =
            xmallocz(indent.wrapping_add(strlen(line))) as *mut ::core::ffi::c_char;
        memset(
            buf as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            indent,
        );
        memcpy(
            buf.offset(indent as isize) as *mut ::core::ffi::c_void,
            line as *const ::core::ffi::c_void,
            strlen(line),
        );
        let mut item: Array = ARRAY_DICT_INIT;
        if item.size == item.capacity {
            item.capacity = if item.capacity != 0 {
                item.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            item.items = xrealloc(
                item.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(item.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh2 = item.size;
        item.size = item.size.wrapping_add(1);
        *item.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        if item.size == item.capacity {
            item.capacity = if item.capacity != 0 {
                item.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            item.items = xrealloc(
                item.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(item.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh3 = item.size;
        item.size = item.size.wrapping_add(1);
        *item.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(buf),
            },
        };
        if item.size == item.capacity {
            item.capacity = if item.capacity != 0 {
                item.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            item.items = xrealloc(
                item.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(item.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh4 = item.size;
        item.size = item.size.wrapping_add(1);
        *item.items.offset(c2rust_fresh4 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let mut content: Array = ARRAY_DICT_INIT;
        if content.size == content.capacity {
            content.capacity = if content.capacity != 0 {
                content.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            content.items = xrealloc(
                content.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(content.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh5 = content.size;
        content.size = content.size.wrapping_add(1);
        *content.items.offset(c2rust_fresh5 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: item },
        };
        if (*cmdline_block.ptr()).size == (*cmdline_block.ptr()).capacity {
            (*cmdline_block.ptr()).capacity = if (*cmdline_block.ptr()).capacity != 0 {
                (*cmdline_block.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*cmdline_block.ptr()).items = xrealloc(
                (*cmdline_block.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul((*cmdline_block.ptr()).capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh6 = (*cmdline_block.ptr()).size;
        (*cmdline_block.ptr()).size = (*cmdline_block.ptr()).size.wrapping_add(1);
        *(*cmdline_block.ptr()).items.offset(c2rust_fresh6 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: content },
        };
        if (*cmdline_block.ptr()).size > 1 as size_t {
            ui_call_cmdline_block_append(content);
        } else {
            ui_call_cmdline_block_show(cmdline_block.get());
        };
    }
}

pub unsafe extern "C" fn ui_ext_cmdline_block_leave() {
    unsafe {
        api_free_array(cmdline_block.get());
        cmdline_block.set(ARRAY_DICT_INIT);
        ui_call_cmdline_block_hide();
    }
}

pub unsafe extern "C" fn cmdline_screen_cleared() {
    unsafe {
        if !ui_has(kUICmdline) {
            return;
        }
        if (*cmdline_block.ptr()).size != 0 {
            ui_call_cmdline_block_show(cmdline_block.get());
        }
        let mut prev_level: ::core::ffi::c_int = (*ccline.ptr()).level - 1 as ::core::ffi::c_int;
        let mut line: *mut CmdlineInfo = (*ccline.ptr()).prev_ccline;
        while prev_level > 0 as ::core::ffi::c_int && !line.is_null() {
            if (*line).level == prev_level {
                if prev_level != cmdwin_level.get() {
                    (*line).redraw_state = kCmdRedrawAll;
                }
                prev_level -= 1;
            }
            line = (*line).prev_ccline;
        }
        redrawcmd();
    }
}

pub unsafe extern "C" fn cmdline_ui_flush() {
    unsafe {
        if !ui_has(kUICmdline) {
            return;
        }
        let mut level: ::core::ffi::c_int = (*ccline.ptr()).level;
        let mut line: *mut CmdlineInfo = ccline.ptr();
        while level > 0 as ::core::ffi::c_int && !line.is_null() {
            if (*line).level == level {
                let mut redraw_state: CmdRedraw = (*line).redraw_state;
                (*line).redraw_state = kCmdRedrawNone;
                if redraw_state as ::core::ffi::c_uint
                    == kCmdRedrawAll as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    cmdline_was_last_drawn.set(true_0 != 0);
                    ui_ext_cmdline_show(line);
                } else if redraw_state as ::core::ffi::c_uint
                    == kCmdRedrawPos as ::core::ffi::c_int as ::core::ffi::c_uint
                    && cmdline_was_last_drawn.get() as ::core::ffi::c_int != 0
                {
                    ui_call_cmdline_pos((*line).cmdpos as Integer, (*line).level as Integer);
                }
                level -= 1;
            }
            line = (*line).prev_ccline;
        }
    }
}
