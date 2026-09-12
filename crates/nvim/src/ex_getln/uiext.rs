//! The `ext_cmdline` UI: the command line as events rather than a grid.
//!
//! [`ui_ext_cmdline_show`] sends `cmdline_show` with the line, the cursor
//! position and the colour chunks; [`ui_ext_cmdline_block_append`] is the
//! multi-line block a `:if`/`:function` body builds up.  A UI that took over
//! the command line gets these instead of anything [`super::draw`] writes.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;

/// Send `cmdline_show` for one command line: its content as
/// `[[attr, text, hl_id], …]`, the cursor position and the prompt.
pub(crate) fn ui_ext_cmdline_show(line: Cc) {
    let mut content: Array;
    if cmdline_star.get() != 0 {
        // Obscured (`inputsecret()`): one '*' per *character*.
        content = Array::with_capacity(1);
        let mut len: size_t = 0;
        let mut p = Cc::current().text();
        while unsafe { *p } != 0 {
            len += 1;
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
        }
        let stars = vec![b'*'; len];

        let mut item = Array::with_capacity(3);
        item.push(Object::integer(0));
        item.push(Object::string(String_0::from_bytes(&stars)));
        item.push(Object::integer(0));
        content.push(Object::array(item));
    } else if !line.last_colors.chunks().is_empty() {
        content = Array::with_capacity(line.last_colors.chunks().len());
        let mut i: size_t = 0;
        while i < line.last_colors.chunks().len() {
            let chunk: CmdlineColorChunk = line.last_colors.chunks()[i];
            let mut item = Array::with_capacity(3);
            item.push(Object::integer(if chunk.hl_id == 0 {
                0
            } else {
                syn_id2attr(chunk.hl_id) as Integer
            }));

            debug_assert!(chunk.end >= chunk.start);
            // SAFETY: the chunk names a run of the command line's own text.
            let text = unsafe {
                core::slice::from_raw_parts(
                    line.at(chunk.start).cast::<u8>(),
                    (chunk.end - chunk.start) as size_t,
                )
            };
            item.push(Object::string(String_0::from_bytes(text)));
            item.push(Object::integer(chunk.hl_id as Integer));
            content.push(Object::array(item));
            i += 1;
        }
    } else {
        let mut item = Array::with_capacity(3);
        item.push(Object::integer(0));
        item.push(Object::string(unsafe { cstr_to_string(line.text()) }));
        item.push(Object::integer(0));
        content = Array::with_capacity(1);
        content.push(Object::array(item));
    }

    let mut charbuf: [::core::ffi::c_char; 2] = [line.cmdfirstc as ::core::ffi::c_char, 0];
    ui_call_cmdline_show(
        content,
        line.cmdpos as Integer,
        unsafe { cstr_to_string(charbuf.as_mut_ptr()) },
        unsafe { cstr_to_string(line.cmdprompt) },
        line.cmdindent as Integer,
        line.level as Integer,
        line.hl_id as Integer,
    );
    if line.special_char != 0 {
        charbuf[0] = line.special_char;
        ui_call_cmdline_special_char(
            unsafe { cstr_to_string(charbuf.as_mut_ptr()) },
            line.special_shift as Boolean,
            line.level as Integer,
        );
    }
}

/// The `ext_cmdline` block: the lines a `:if` or `:function` body
/// accumulates while it is being typed.
///
/// Deliberately not `Copy`: `cmdline_block.get()` handed every caller a
/// second owner of the same `items` pointer, so the cell and the caller both
/// believed they had to free it. The array owns its lines, so the free is
/// the field's own, and [`ui_ext_cmdline_block_leave`]'s move a `take`.
pub(crate) struct CmdlineBlock(Array);

impl CmdlineBlock {
    pub(crate) const EMPTY: CmdlineBlock = CmdlineBlock(ARRAY_DICT_INIT);

    /// The lines, for the UI call that serialises them.
    ///
    /// A copy: the UI call takes over what it is handed, and the block goes
    /// on owning its own lines.
    fn lines(&self) -> Array {
        self.0.clone()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for CmdlineBlock {
    fn default() -> Self {
        CmdlineBlock::EMPTY
    }
}

/// Append one line to the `ext_cmdline` block — the body a `:if` or
/// `:function` accumulates while it is being typed.
///
/// # Safety
///
/// `line` must point at a NUL-terminated string.
pub unsafe fn ui_ext_cmdline_block_append(indent: size_t, line: *const ::core::ffi::c_char) {
    let line_len = unsafe { cstr::bytes_at(line) }.len();
    let buf = unsafe { xmallocz(indent + line_len) } as *mut ::core::ffi::c_char;
    unsafe { buf.cast::<u8>().write_bytes(b' ', indent) };
    let into = unsafe { buf.add(indent) }.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(line.cast(), line_len) };

    let mut item = Array::with_capacity(3);
    item.push(Object::integer(0));
    // SAFETY: `buf` is the NUL-terminated allocation just filled in, which
    // the string takes over.
    item.push(Object::string(unsafe {
        String_0::from_owned_parts(buf, indent + line_len)
    }));
    item.push(Object::integer(0));

    let mut content = Array::with_capacity(1);
    content.push(Object::array(item));

    // A leaf closure over the array itself: nothing it runs can re-enter
    // the block, so the exclusive borrow cannot overlap another.
    let first = cmdline_block.with_mut(|block| {
        block.0.push(Object::array(content.clone()));
        block.0.len() == 1
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
pub fn cmdline_screen_cleared() {
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
    redrawcmd();
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
                ui_ext_cmdline_show(line);
            } else if redraw_state == kCmdRedrawPos && cmdline_was_last_drawn.get() {
                ui_call_cmdline_pos(line.cmdpos as Integer, line.level as Integer);
            }
            level -= 1;
        }
        depth += 1;
    }
}
