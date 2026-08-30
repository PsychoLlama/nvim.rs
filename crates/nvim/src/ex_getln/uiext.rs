//! The `ext_cmdline` UI: the command line as events rather than a grid.
//!
//! [`ui_ext_cmdline_show`] sends `cmdline_show` with the line, the cursor
//! position and the colour chunks; [`ui_ext_cmdline_block_append`] is the
//! multi-line block a `:if`/`:function` body builds up.  A UI that took over
//! the command line gets these instead of anything [`super::draw`] writes.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;

/// Send `cmdline_show` for one command line: its content as
/// `[[attr, text, hl_id], …]`, the cursor position and the prompt.
pub(crate) unsafe fn ui_ext_cmdline_show(line: Cc) {
    let mut arena: Arena = ARENA_EMPTY;

    // C's `ADD_C`: an arena array is allocated at its final size, so the
    // push is a store and a bump with no capacity test.
    let push = |arr: &mut Array, value: Object| {
        unsafe { *arr.items.add(arr.size) = value };
        arr.size += 1;
    };

    let mut content: Array;
    if cmdline_star.get() != 0 {
        // Obscured (`inputsecret()`): one '*' per *character*.
        content = arena_array(&raw mut arena, 1);
        let mut len: size_t = 0;
        let mut p = Cc::current().text();
        while unsafe { *p } != 0 {
            len += 1;
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
        }
        let buf = unsafe { arena_alloc(&raw mut arena, len, false) } as *mut ::core::ffi::c_char;
        unsafe { buf.cast::<u8>().write_bytes(b'*', len) };

        let mut item = arena_array(&raw mut arena, 3);
        push(&mut item, Object::integer(0));
        push(
            &mut item,
            Object::string(String_0::from_raw_parts(buf, len)),
        );
        push(&mut item, Object::integer(0));
        push(&mut content, Object::array(item));
    } else if !line.last_colors.chunks().is_empty() {
        content = arena_array(&raw mut arena, line.last_colors.chunks().len());
        let mut i: size_t = 0;
        while i < line.last_colors.chunks().len() {
            let chunk: CmdlineColorChunk = line.last_colors.chunks()[i];
            let mut item = arena_array(&raw mut arena, 3);
            push(
                &mut item,
                Object::integer(if chunk.hl_id == 0 {
                    0
                } else {
                    unsafe { syn_id2attr(chunk.hl_id) as Integer }
                }),
            );

            debug_assert!(chunk.end >= chunk.start);
            push(
                &mut item,
                Object::string(String_0::from_raw_parts(
                    line.at(chunk.start),
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
        push(
            &mut item,
            Object::string(unsafe { cstr_as_string(line.text()) }),
        );
        push(&mut item, Object::integer(0));
        content = arena_array(&raw mut arena, 1);
        push(&mut content, Object::array(item));
    }

    let mut charbuf: [::core::ffi::c_char; 2] = [line.cmdfirstc as ::core::ffi::c_char, 0];
    ui_call_cmdline_show(
        content,
        line.cmdpos as Integer,
        unsafe { cstr_as_string(charbuf.as_mut_ptr()) },
        unsafe { cstr_as_string(line.cmdprompt) },
        line.cmdindent as Integer,
        line.level as Integer,
        line.hl_id as Integer,
    );
    if line.special_char != 0 {
        charbuf[0] = line.special_char;
        ui_call_cmdline_special_char(
            unsafe { cstr_as_string(charbuf.as_mut_ptr()) },
            line.special_shift as Boolean,
            line.level as Integer,
        );
    }
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
}

/// The `ext_cmdline` block: the lines a `:if` or `:function` body
/// accumulates while it is being typed.
///
/// Deliberately not `Copy`: `cmdline_block.get()` handed every caller a
/// second owner of the same `items` pointer, so the cell and the caller both
/// believed they had to free it. Owning the array here makes the free this
/// type's `Drop`, and [`ui_ext_cmdline_block_leave`]'s move a `take`.
pub(crate) struct CmdlineBlock(Array);

impl CmdlineBlock {
    pub(crate) const EMPTY: CmdlineBlock = CmdlineBlock(ARRAY_DICT_INIT);

    /// The lines, for the UI call that serialises them.
    ///
    /// A shallow copy that the UI call reads and does not free; it must not
    /// outlive the next append, which is why every caller takes it inside
    /// the expression that hands it over.
    fn lines(&self) -> Array {
        self.0
    }

    fn is_empty(&self) -> bool {
        self.0.size == 0
    }
}

impl Default for CmdlineBlock {
    fn default() -> Self {
        CmdlineBlock::EMPTY
    }
}

impl Drop for CmdlineBlock {
    fn drop(&mut self) {
        // SAFETY: the array and its objects are this value's own.
        unsafe { api_free_array(self.0) };
    }
}

/// Append one line to the `ext_cmdline` block — the body a `:if` or
/// `:function` accumulates while it is being typed.
pub unsafe fn ui_ext_cmdline_block_append(indent: size_t, line: *const ::core::ffi::c_char) {
    let buf = unsafe { xmallocz(indent + cstr::bytes_at(line).len()) } as *mut ::core::ffi::c_char;
    unsafe { buf.cast::<u8>().write_bytes(b' ', indent) };
    let line_len = unsafe { cstr::bytes_at(line) }.len();
    let into = unsafe { buf.add(indent) }.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(line.cast(), line_len) };

    // C's `ADD`: `kv_push` onto a heap array, doubling from 8.
    let push = |arr: &mut Array, value: Object| {
        if arr.size == arr.capacity {
            arr.capacity = if arr.capacity != 0 {
                arr.capacity << 1
            } else {
                8
            };
            arr.items = unsafe {
                xrealloc(
                    arr.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>() * arr.capacity,
                )
            } as *mut Object;
        }
        unsafe { *arr.items.add(arr.size) = value };
        arr.size += 1;
    };

    let mut item: Array = ARRAY_DICT_INIT;
    push(&mut item, Object::integer(0));
    push(&mut item, Object::string(unsafe { cstr_as_string(buf) }));
    push(&mut item, Object::integer(0));

    let mut content: Array = ARRAY_DICT_INIT;
    push(&mut content, Object::array(item));

    // A leaf closure over the array itself: nothing it runs can re-enter
    // the block, so the exclusive borrow cannot overlap another.
    let first = cmdline_block.with_mut(|block| {
        push(&mut block.0, Object::array(content));
        block.0.size == 1
    });
    if first {
        ui_call_cmdline_block_show(cmdline_block.with(CmdlineBlock::lines));
    } else {
        ui_call_cmdline_block_append(content);
    }
}

/// Drop the `ext_cmdline` block and tell the UI to hide it.
pub fn ui_ext_cmdline_block_leave() {
    // The block moves out of the cell and is freed by its own `Drop`.
    drop(cmdline_block.take());
    ui_call_cmdline_block_hide();
}

/// Extra redrawing needed for `:redraw!` and on `ui_attach`.
pub unsafe fn cmdline_screen_cleared() {
    if !ui_has(kUICmdline) {
        return;
    }

    if !cmdline_block.with(CmdlineBlock::is_empty) {
        ui_call_cmdline_block_show(cmdline_block.with(CmdlineBlock::lines));
    }

    // Every command line suspended under this one wants redrawing too.
    let mut prev_level = Cc::current().level - 1;
    let mut depth = 1;
    while prev_level > 0 {
        let Some(mut line) = cmdline_at(depth) else {
            break;
        };
        if line.level == prev_level {
            // Don't redraw a command line already shown in the cmdline window.
            if prev_level != cmdwin_level.get() {
                line.redraw_state = kCmdRedrawAll;
            }
            prev_level -= 1;
        }
        depth += 1;
    }
    // SAFETY: redraws the command line the editor is on.
    unsafe { redrawcmd() };
}

/// Called by `ui_flush`: send whatever redraws keep the externalised command
/// line up to date.
pub fn cmdline_ui_flush() {
    if !ui_has(kUICmdline) {
        return;
    }
    let mut level = Cc::current().level;
    let mut depth = 0;
    while level > 0 {
        let Some(mut line) = cmdline_at(depth) else {
            break;
        };
        if line.level == level {
            let redraw_state = line.redraw_state;
            line.redraw_state = kCmdRedrawNone;
            if redraw_state == kCmdRedrawAll {
                cmdline_was_last_drawn.set(true);
                // SAFETY: a live command line, from `cmdline_at`.
                unsafe { ui_ext_cmdline_show(line) };
            } else if redraw_state == kCmdRedrawPos && cmdline_was_last_drawn.get() {
                ui_call_cmdline_pos(line.cmdpos as Integer, line.level as Integer);
            }
            level -= 1;
        }
        depth += 1;
    }
}
