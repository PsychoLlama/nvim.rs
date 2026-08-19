//! The `ext_cmdline` UI: the command line as events rather than a grid.
//!
//! [`ui_ext_cmdline_show`] sends `cmdline_show` with the line, the cursor
//! position and the colour chunks; [`ui_ext_cmdline_block_append`] is the
//! multi-line block a `:if`/`:function` body builds up.  A UI that took over
//! the command line gets these instead of anything [`super::draw`] writes.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

/// Send `cmdline_show` for one command line: its content as
/// `[[attr, text, hl_id], …]`, the cursor position and the prompt.
pub(crate) unsafe fn ui_ext_cmdline_show(line: *mut CmdlineInfo) {
    unsafe {
        let mut arena: Arena = ARENA_EMPTY;

        // C's `ADD_C`: an arena array is allocated at its final size, so the
        // push is a store and a bump with no capacity test.
        let push = |arr: &mut Array, value: Object| {
            *arr.items.add(arr.size) = value;
            arr.size += 1;
        };

        let mut content: Array;
        if cmdline_star.get() != 0 {
            // Obscured (`inputsecret()`): one '*' per *character*.
            content = arena_array(&raw mut arena, 1);
            let mut len: size_t = 0;
            let mut p = (*ccline.ptr()).cmdbuff;
            while *p != 0 {
                len += 1;
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            let buf = arena_alloc(&raw mut arena, len, false) as *mut ::core::ffi::c_char;
            memset(
                buf as *mut ::core::ffi::c_void,
                '*' as ::core::ffi::c_int,
                len,
            );

            let mut item = arena_array(&raw mut arena, 3);
            push(&mut item, Object::integer(0));
            push(
                &mut item,
                Object::string(String_0::from_raw_parts(buf, len)),
            );
            push(&mut item, Object::integer(0));
            push(&mut content, Object::array(item));
        } else if (*line).last_colors.colors.size != 0 {
            content = arena_array(&raw mut arena, (*line).last_colors.colors.size);
            let mut i: size_t = 0;
            while i < (*line).last_colors.colors.size {
                let chunk: CmdlineColorChunk = *(*line).last_colors.colors.items.add(i);
                let mut item = arena_array(&raw mut arena, 3);
                push(
                    &mut item,
                    Object::integer(if chunk.hl_id == 0 {
                        0
                    } else {
                        syn_id2attr(chunk.hl_id) as Integer
                    }),
                );

                debug_assert!(chunk.end >= chunk.start);
                push(
                    &mut item,
                    Object::string(String_0::from_raw_parts(
                        (*line).cmdbuff.offset(chunk.start as isize),
                        (chunk.end - chunk.start) as size_t,
                    )),
                );
                push(&mut item, Object::integer(chunk.hl_id as Integer));
                push(&mut content, Object::array(item));
                i += 1;
            }
        } else {
            let mut item = arena_array(&raw mut arena, 3);
            push(&mut item, Object::integer(0));
            push(&mut item, Object::string(cstr_as_string((*line).cmdbuff)));
            push(&mut item, Object::integer(0));
            content = arena_array(&raw mut arena, 1);
            push(&mut content, Object::array(item));
        }

        let mut charbuf: [::core::ffi::c_char; 2] = [(*line).cmdfirstc as ::core::ffi::c_char, 0];
        ui_call_cmdline_show(
            content,
            (*line).cmdpos as Integer,
            cstr_as_string(charbuf.as_mut_ptr()),
            cstr_as_string((*line).cmdprompt),
            (*line).cmdindent as Integer,
            (*line).level as Integer,
            (*line).hl_id as Integer,
        );
        if (*line).special_char != 0 {
            charbuf[0] = (*line).special_char;
            ui_call_cmdline_special_char(
                cstr_as_string(charbuf.as_mut_ptr()),
                (*line).special_shift as Boolean,
                (*line).level as Integer,
            );
        }
        arena_mem_free(arena_finish(&raw mut arena));
    }
}

/// Append one line to the `ext_cmdline` block — the body a `:if` or
/// `:function` accumulates while it is being typed.
pub unsafe fn ui_ext_cmdline_block_append(indent: size_t, line: *const ::core::ffi::c_char) {
    unsafe {
        let buf = xmallocz(indent + strlen(line)) as *mut ::core::ffi::c_char;
        memset(
            buf as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            indent,
        );
        memcpy(
            buf.add(indent) as *mut ::core::ffi::c_void,
            line as *const ::core::ffi::c_void,
            strlen(line),
        );

        // C's `ADD`: `kv_push` onto a heap array, doubling from 8.
        let push = |arr: &mut Array, value: Object| {
            if arr.size == arr.capacity {
                arr.capacity = if arr.capacity != 0 {
                    arr.capacity << 1
                } else {
                    8
                };
                arr.items = xrealloc(
                    arr.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>() * arr.capacity,
                ) as *mut Object;
            }
            *arr.items.add(arr.size) = value;
            arr.size += 1;
        };

        let mut item: Array = ARRAY_DICT_INIT;
        push(&mut item, Object::integer(0));
        push(&mut item, Object::string(cstr_as_string(buf)));
        push(&mut item, Object::integer(0));

        let mut content: Array = ARRAY_DICT_INIT;
        push(&mut content, Object::array(item));

        push(&mut *cmdline_block.ptr(), Object::array(content));
        if (*cmdline_block.ptr()).size > 1 {
            ui_call_cmdline_block_append(content);
        } else {
            ui_call_cmdline_block_show(cmdline_block.get());
        }
    }
}

/// Drop the `ext_cmdline` block and tell the UI to hide it.
pub unsafe fn ui_ext_cmdline_block_leave() {
    unsafe {
        api_free_array(cmdline_block.get());
        cmdline_block.set(ARRAY_DICT_INIT);
        ui_call_cmdline_block_hide();
    }
}

/// Extra redrawing needed for `:redraw!` and on `ui_attach`.
pub unsafe fn cmdline_screen_cleared() {
    unsafe {
        if !ui_has(kUICmdline) {
            return;
        }

        if (*cmdline_block.ptr()).size != 0 {
            ui_call_cmdline_block_show(cmdline_block.get());
        }

        let mut prev_level = (*ccline.ptr()).level - 1;
        let mut line = (*ccline.ptr()).prev_ccline;
        while prev_level > 0 && !line.is_null() {
            if (*line).level == prev_level {
                // Don't redraw a command line already shown in the cmdline
                // window.
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

/// Called by `ui_flush`: send whatever redraws keep the externalised command
/// line up to date.
pub unsafe fn cmdline_ui_flush() {
    unsafe {
        if !ui_has(kUICmdline) {
            return;
        }
        let mut level = (*ccline.ptr()).level;
        let mut line = ccline.ptr();
        while level > 0 && !line.is_null() {
            if (*line).level == level {
                let redraw_state = (*line).redraw_state;
                (*line).redraw_state = kCmdRedrawNone;
                if redraw_state == kCmdRedrawAll {
                    cmdline_was_last_drawn.set(true);
                    ui_ext_cmdline_show(line);
                } else if redraw_state == kCmdRedrawPos && cmdline_was_last_drawn.get() {
                    ui_call_cmdline_pos((*line).cmdpos as Integer, (*line).level as Integer);
                }
                level -= 1;
            }
            line = (*line).prev_ccline;
        }
    }
}
