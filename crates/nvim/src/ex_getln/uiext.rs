//! The `ext_cmdline` UI: the command line as events rather than a grid.
//!
//! [`ui_ext_cmdline_show`] sends `cmdline_show` with the line, the cursor
//! position and the colour chunks; [`ui_ext_cmdline_block_append`] is the
//! multi-line block a `:if`/`:function` body builds up.  A UI that took over
//! the command line gets these instead of anything [`super::draw`] writes.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;

/// Send `cmdline_show` for one command line: its content as
/// `[[attr, text, hl_id], …]`, the cursor position and the prompt.
pub(crate) fn ui_ext_cmdline_show(line: Cc) {
    let mut content: Array;
    if cmdline_star.get() != 0 {
        // Obscured (`inputsecret()`): one '*' per *character*.
        content = Array::with_capacity(1);
        let stars = {
            let cc = Cc::current();
            let line = cstr::until_nul(cc.bytes());
            let mut len: size_t = 0;
            let mut p = 0;
            while p < line.len() {
                len += 1;
                p += c_cluster_len(&line[p..]).max(1);
            }
            vec![b'*'; len]
        };

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
            let text = line.bytes();
            let from = usize::try_from(chunk.start).unwrap_or(0).min(text.len());
            let to = usize::try_from(chunk.end)
                .unwrap_or(0)
                .clamp(from, text.len());
            item.push(Object::string(String_0::from_bytes(&text[from..to])));
            item.push(Object::integer(chunk.hl_id as Integer));
            content.push(Object::array(item));
            i += 1;
        }
    } else {
        let mut item = Array::with_capacity(3);
        item.push(Object::integer(0));
        item.push(Object::string(String_0::from_bytes(cstr::until_nul(
            line.bytes(),
        ))));
        item.push(Object::integer(0));
        content = Array::with_capacity(1);
        content.push(Object::array(item));
    }

    let firstc = [line.cmdfirstc as u8];
    // C's `cstr_to_string`: the null string when there is no prompt.
    let prompt = line.cmdprompt.as_ref().map_or(String_0::NULL, |prompt| {
        String_0::from_bytes(prompt.as_cstr().to_bytes())
    });
    ui_call_cmdline_show(
        content,
        line.cmdpos as Integer,
        String_0::from_bytes(cstr::until_nul(&firstc)),
        prompt,
        line.cmdindent as Integer,
        line.level as Integer,
        line.hl_id as Integer,
    );
    if line.special_char != 0 {
        ui_call_cmdline_special_char(
            String_0::from_bytes(&[line.special_char as u8]),
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
pub fn ui_ext_cmdline_block_append(indent: size_t, line: &[u8]) {
    let mut text = Vec::with_capacity(indent + line.len());
    text.resize(indent, b' ');
    text.extend_from_slice(line);

    let mut item = Array::with_capacity(3);
    item.push(Object::integer(0));
    item.push(Object::string(String_0::from_bytes(&text)));
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
