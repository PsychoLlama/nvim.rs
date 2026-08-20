//! `do_mouse()` -- the Normal/Visual-mode mouse command.
//!
//! One state machine over the button, the modifiers, the click count and
//! `'mousemodel'`, deciding between: starting or extending a Visual selection,
//! setting the cursor, opening the popup menu, pasting the selection, dragging
//! a status line or vertical separator, closing or moving a tab page, folding,
//! and the "which window does this belong to" question that has to be answered
//! before any of them.
//!
//! It is written here as the stages it runs in order, each answering `Some`
//! when the event is finished with: [`coalesce_drags`], [`ignore_stray_event`],
//! [`modifier_shortcuts`], [`middle_button_insert`], [`tab_line_click`], the
//! `'mousemodel'` translation, the jump itself, [`click_definition`], the fold
//! click, [`extend_visual_block`] and finally [`dispatch_action`], which is the
//! chain of `else if`s the C ends with.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ops::{Deref, DerefMut};

use super::*;
use crate::ascii::ascii_iswhite;
use crate::buffer::bt_quickfix;
use crate::charset::vim_iswordc;
use crate::cursor::get_cursor_pos_ptr;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::eval::eval_has_provider;
use crate::ex_docmd::{do_cmdline_cmd, tabpage_new};
use crate::fold::{closeFold, openFold};
use crate::getchar::{
    append_to_redobuff_char, safe_vgetc, stuff_readbuf, stuff_readbuf_char, stuff_readbuf_number,
    vpeekc, vungetc,
};
use crate::global_cell::GlobalCell;
use crate::keycodes::{
    Ctrl_G, Ctrl_O, Ctrl_P, Ctrl_R, Ctrl_RSB, Ctrl_T, Ctrl_V, K_MIDDLEMOUSE, K_MOUSEMOVE,
    get_mouse_button,
};
use crate::main::{
    Columns, KeyStuffed, State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect, VIsual_select,
    cmdwin_type, mod_mask, mode_displayed, mouse_col, mouse_dragging, mouse_grid,
    mouse_past_bottom, mouse_past_eol, mouse_row, msg_silent, p_smd, redraw_cmdline, restart_edit,
    where_paste_started,
};
use crate::memline::{gchar_pos, inc};
use crate::r#move::scroll_redraw;
use crate::normal::{clearop, clearopbeep, end_visual_mode, may_start_select, prep_redo};
use crate::pos::{equalpos, lt};
use crate::register::{do_put, insert_reg, yank_register_mline};
use crate::search::{FORWARD, findmatch};
use crate::state::{MODE_INSERT, MODE_NORMAL, REPLACE_FLAG};
use crate::statusline::{
    kStlClickDisabled, kStlClickFuncRun, kStlClickTabClose, kStlClickTabSwitch,
};
use crate::types::{NUL, OP_NOP, PUT_CURSEND, PUT_FIXINDENT, oparg_T, yankreg_T};
use crate::ui::ui_mouse_has;
use crate::window::{goto_tabpage, tabpage_move};

/// Whether the last click was in the tab page line, so that its release is
/// ignored rather than treated as a click in a window.
static in_tab_line: GlobalCell<bool> = GlobalCell::new(false);
/// Where the multi-click selection started, which a drag extends away from.
static orig_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});

/// The operator the command was given, when it was given one.
#[derive(Clone, Copy)]
struct Oap(*mut oparg_T);

impl Deref for Oap {
    type Target = oparg_T;

    fn deref(&self) -> &oparg_T {
        // SAFETY: the constructor's promise -- a live operator argument.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Oap {
    fn deref_mut(&mut self) -> &mut oparg_T {
        // SAFETY: the constructor's promise -- a live operator argument.
        unsafe { &mut *self.0 }
    }
}

impl Oap {
    /// Whether an operator is pending on it.
    fn pending(self) -> bool {
        self.op_type != OP_NOP
    }

    /// Clear the operator, and beep.
    fn clear_and_beep(self) {
        // SAFETY: the constructor's promise.
        unsafe { clearopbeep(self.0) };
    }

    /// Clear the operator without beeping.
    fn clear(self) {
        // SAFETY: the constructor's promise.
        unsafe { clearop(self.0) };
    }

    /// The match for the item under the cursor, as `%` would find it.
    fn findmatch(self) -> Option<Pos> {
        // SAFETY: the constructor's promise; the answer is a live static
        // position or null.
        let pos = unsafe { findmatch(self.0, NUL) };
        // SAFETY: as above.
        (!pos.is_null()).then(|| unsafe { Pos::new(pos) })
    }

    /// Where `jump_to_mouse` records whether the motion is inclusive.
    fn inclusive(self) -> *mut bool {
        // SAFETY: the constructor's promise.
        unsafe { &raw mut (*self.0).inclusive }
    }
}

/// Do the appropriate action for the current mouse click in the current mode.
/// Not used for Command-line mode.
///
/// Normal and Visual Mode:
/// ```text
/// event         modi-  position      visual       change   action
///               fier   cursor                     window
/// left press     -     yes         end             yes
/// left press     C     yes         end             yes     "^]" (2)
/// left press     S     yes     end (popup: extend) yes     "*" (2)
/// left drag      -     yes     start if moved      no
/// left relse     -     yes     start if moved      no
/// middle press   -     yes      if not active      no      put register
/// middle press   -     yes      if active          no      yank and put
/// right press    -     yes     start or extend     yes
/// right press    S     yes     no change           yes     "#" (2)
/// right drag     -     yes     extend              no
/// right relse    -     yes     extend              no
/// ```
///
/// Insert or Replace Mode:
/// ```text
/// event         modi-  position      visual       change   action
///               fier   cursor                     window
/// left press     -     yes     (cannot be active)  yes
/// left press     C     yes     (cannot be active)  yes     "CTRL-O^]" (2)
/// left press     S     yes     (cannot be active)  yes     "CTRL-O*" (2)
/// left drag      -     yes     start or extend (1) no      CTRL-O (1)
/// left relse     -     yes     start or extend (1) no      CTRL-O (1)
/// middle press   -     no      (cannot be active)  no      put register
/// right press    -     yes     start or extend     yes     CTRL-O
/// right press    S     yes     (cannot be active)  yes     "CTRL-O#" (2)
/// ```
///
/// (1) only if mouse pointer moved since press
/// (2) only if click is in same buffer
///
/// @param oap        operator argument, can be NULL
/// @param c          `K_LEFTMOUSE`, etc
/// @param dir        Direction to 'put' if necessary
/// @param fixindent  `PUT_FIXINDENT` if fixing indent necessary
///
/// @return           true if `start_arrow()` should be called for edit mode.
///
/// # Safety
/// `oap` must be a live operator argument or null.
pub unsafe fn do_mouse(
    oap: *mut oparg_T,
    c: c_int,
    dir: c_int,
    count: c_int,
    fixindent: bool,
) -> bool {
    // SAFETY: the caller's promise.
    // The caller's promise makes every deref below sound.
    let oap = (!oap.is_null()).then_some(Oap(oap));
    let (mut which_button, is_click, is_drag) = coalesce_drags(c);

    if c == K_MOUSEMOVE {
        return false; // Mouse moved without a button pressed.
    }
    if let Some(answer) = ignore_stray_event(is_click, is_drag) {
        return answer;
    }
    if let Some(answer) = modifier_shortcuts(is_click, which_button, count) {
        return answer;
    }

    let regname = oap.map_or(0, |o| o.regname);
    if which_button == MOUSE_MIDDLE
        && let Some(answer) = middle_button_insert(oap, regname, fixindent)
    {
        return answer;
    }

    // Flags for jump_to_mouse(); when dragging or button-up stay in the same
    // window.
    let mut jump_flags = if is_click {
        0
    } else {
        MOUSE_FOCUS | MOUSE_DID_MOVE
    };
    // SAFETY: `curwin` is live from startup to exit.
    let old_curwin = unsafe { Win::current() };

    // Only when initialized.
    // SAFETY: the tabline's definitions cover the screen's columns, which is
    // all the arms below index with.
    if let Some(defs) = unsafe { ClickDefs::tabline() }
        && let Some(answer) = tab_line_click(defs, is_click, is_drag, which_button, old_curwin)
    {
        return answer;
    }

    let mut m_pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut m_pos_flag = 0;
    // When 'mousemodel' is "popup" or "popup_setpos", translate mouse events:
    // right button up   -> pop-up menu
    // shift-left button -> right button
    // alt-left button   -> alt-right button
    if mouse_model_popup() {
        // SAFETY: a live local position.
        m_pos_flag = get_fpos_of_mouse(unsafe { Pos::new(&raw mut m_pos) });
        let over_text = m_pos_flag & (IN_STATUS_LINE | MOUSE_WINBAR | MOUSE_STATUSCOL) == 0;
        if over_text
            && which_button == MOUSE_RIGHT
            && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
        {
            if !is_click {
                // Ignore right button release events, only shows the popup
                // menu on the button down event.
                return false;
            }
            return do_popup(which_button, m_pos_flag, m_pos) & CURSOR_MOVED != 0;
        }
        // Only do this translation when mouse is over the buffer text.
        if over_text
            && which_button == MOUSE_LEFT
            && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0
        {
            which_button = MOUSE_RIGHT;
            mod_mask.set(mod_mask.get() & !MOD_MASK_SHIFT);
        }
    }

    // SAFETY: reads the `'mouse'` option against the current mode.
    let mouse_can_visual = unsafe { ui_mouse_has(MOUSE_VISUAL) };
    let visual_corner = visual_jump_flags(
        &mut jump_flags,
        is_click,
        which_button,
        mouse_can_visual,
        old_curwin,
    );

    // If an operator is pending, ignore all drags and releases until the next
    // mouse click.
    if let Some(mut oap) = oap.filter(|o| !is_drag && o.pending()) {
        got_click.set(false);
        oap.motion_type = kMTCharWise;
    }

    // When releasing the button let jump_to_mouse() know.
    if !is_click && !is_drag {
        jump_flags |= MOUSE_RELEASED;
    }

    // JUMP!
    let old_active = VIsual_active.get();
    let save_cursor = old_curwin.w_cursor;

    // Even though we gate *_VIS flags above, we want to make sure the cursor
    // doesn't move in visual mode unless it is set as a mouse option.
    if !VIsual_active.get() || mouse_can_visual {
        let inclusive = oap.map_or(ptr::null_mut(), Oap::inclusive);
        // SAFETY: `inclusive` is a field of the live operator, or null.
        jump_flags = unsafe { jump_to_mouse(jump_flags, inclusive, which_button) };
    }

    let moved = jump_flags & CURSOR_MOVED != 0;
    let landed = Landed::of(jump_flags);

    if (landed.winbar || landed.status_line || landed.statuscol) && is_click {
        if let Some(answer) = click_definition(landed, jump_flags, which_button, m_pos_flag, m_pos)
        {
            return answer;
        }
    } else if landed.winbar || landed.statuscol {
        // A drag or release event in the window bar and status column has no
        // side effects.
        return false;
    }

    // SAFETY: `curwin` is live; `jump_to_mouse` may have moved focus.
    let mut win = unsafe { Win::current() };

    // When jumping to another window, clear a pending operator.  That's a bit
    // friendlier than beeping and not jumping to that window.
    if let Some(oap) = oap.filter(|o| win != old_curwin && o.pending()) {
        oap.clear();
    }

    if mod_mask.get() == 0
        && !is_drag
        && jump_flags & (MOUSE_FOLD_CLOSE | MOUSE_FOLD_OPEN) != 0
        && which_button == MOUSE_LEFT
    {
        // Open or close a fold at this line.
        let fold: unsafe extern "C" fn(pos_T, c_int) = if jump_flags & MOUSE_FOLD_OPEN != 0 {
            openFold
        } else {
            closeFold
        };
        // SAFETY: a live position in the current buffer.
        unsafe { fold(win.w_cursor, 1) };
        // Don't move the cursor if still in the same window.
        if win == old_curwin {
            win.w_cursor = save_cursor;
        }
    }

    // Set global flag that we are extending the Visual area with mouse
    // dragging; temporarily minimize 'scrolloff'.
    if VIsual_active.get() && is_drag && win.scrolloff() != 0 {
        // In the very first line, allow scrolling one line.
        mouse_dragging.set(if mouse_row.get() == 0 { 2 } else { 1 });
    }

    // When dragging the mouse above the window, scroll down.
    if is_drag && mouse_row.get() < 0 && !landed.status_line {
        // SAFETY: scrolls the current window.
        unsafe { scroll_redraw(false as c_int, 1) };
        mouse_row.set(0);
    }

    let old_mode = VIsual_mode.get();
    if let Some((start_visual, end_visual)) = visual_corner {
        // Right click in Visual mode.
        extend_visual_block(win, start_visual, end_visual);
    } else if State.get() & MODE_INSERT != 0 && VIsual_active.get() {
        // If Visual mode started in insert mode, execute "CTRL-O".
        stuff_char(Ctrl_O);
    }

    dispatch_action(
        Action {
            oap,
            which_button,
            is_click,
            is_drag,
            dir,
            count,
            fixindent,
            regname,
            old_active,
            mouse_can_visual,
            landed,
        },
        win,
    );

    // If Visual mode changed show it later.
    if (!VIsual_active.get() && old_active && mode_displayed.get())
        || (VIsual_active.get()
            && p_smd.get() != 0
            && msg_silent.get() == 0
            && (!old_active || VIsual_mode.get() != old_mode))
    {
        redraw_cmdline.set(true);
    }

    moved
}

// ---------------------------------------------------------------------------
// The stages

/// Read the button, click and drag flags out of the key code, swallowing a
/// run of identical drag events.
///
/// Speeds up dragging the status line.  Since characters added to the stuff
/// buffer further down need to come before the next character, this is not
/// done when the current character was stuffed.
fn coalesce_drags(c: c_int) -> (c_int, bool, bool) {
    loop {
        let (mut is_click, mut is_drag) = (false, false);
        // SAFETY: `get_mouse_button` writes through the two locals.
        let which_button =
            unsafe { get_mouse_button(key_extra(c), &raw mut is_click, &raw mut is_drag) };
        // SAFETY: `vpeekc` only looks at the typeahead.
        if !is_drag || KeyStuffed.get() != 0 || unsafe { vpeekc() } == NUL {
            return (which_button, is_click, is_drag);
        }

        let saved = MousePos::current();
        // Need to get the character, peeking doesn't get the actual one.
        // SAFETY: takes one key from the typeahead.
        let nc = unsafe { safe_vgetc() };
        if c == nc {
            // The next character is the same mouse event: use that one.
            continue;
        }
        vungetc(nc);
        mouse_grid.set(saved.grid);
        mouse_row.set(saved.row);
        mouse_col.set(saved.col);
        return (which_button, is_click, is_drag);
    }
}

/// Ignore drag and release events if we didn't get a click.
fn ignore_stray_event(is_click: bool, is_drag: bool) -> Option<bool> {
    if is_click {
        got_click.set(true);
        return None;
    }
    if !got_click.get() {
        return Some(false); // didn't get click, ignore
    }
    if !is_drag {
        got_click.set(false); // release, reset got_click
        if in_tab_line.get() {
            in_tab_line.set(false);
            return Some(false);
        }
    }
    None
}

/// The events a modifier turns into something else entirely, or suppresses.
fn modifier_shortcuts(is_click: bool, which_button: c_int, count: c_int) -> Option<bool> {
    let mods = mod_mask.get();

    // CTRL right mouse button does CTRL-T.
    if is_click && mods & MOD_MASK_CTRL != 0 && which_button == MOUSE_RIGHT {
        if State.get() & MODE_INSERT != 0 {
            stuff_char(Ctrl_O);
        }
        if count > 1 {
            // SAFETY: appends to the stuff buffer.
            unsafe { stuff_readbuf_number(count) };
        }
        stuff_char(Ctrl_T);
        got_click.set(false); // ignore drag&release now
        return Some(false);
    }

    // CTRL only works with left mouse button.
    if mods & MOD_MASK_CTRL != 0 && which_button != MOUSE_LEFT {
        return Some(false);
    }

    // When a modifier is down, ignore drag and release events, as well as
    // multiple clicks and the middle mouse button.
    // Accept shift-leftmouse drags when 'mousemodel' is "popup.*".
    if mods & (MOD_MASK_SHIFT | MOD_MASK_CTRL | MOD_MASK_ALT | MOD_MASK_META) != 0
        && (!is_click || mods & MOD_MASK_MULTI_CLICK != 0 || which_button == MOUSE_MIDDLE)
        && !(mods & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0
            && mouse_model_popup()
            && which_button == MOUSE_LEFT)
        && !(mods & MOD_MASK_ALT != 0 && !mouse_model_popup() && which_button == MOUSE_RIGHT)
    {
        return Some(false);
    }

    // If the button press was used as the movement command for an operator
    // (eg "d<MOUSE>"), or it is the middle button that is held down, ignore
    // drag/release events.
    if !is_click && which_button == MOUSE_MIDDLE {
        return Some(false);
    }

    None
}

/// The middle button before the jump: a `put` of the selected text, which in
/// Insert mode does not move the cursor at all.
///
/// Answers `None` only in Normal mode with nothing selected and no operator
/// pending -- the rest is below `jump_to_mouse()`.
fn middle_button_insert(oap: Option<Oap>, mut regname: c_int, fixindent: bool) -> Option<bool> {
    if State.get() == MODE_NORMAL {
        // If an operator was pending, we don't know what the user wanted to
        // do.  Go back to normal mode: Clear the operator and beep().
        if let Some(oap) = oap.filter(|o| o.pending()) {
            oap.clear_and_beep();
            return Some(false);
        }
        // If visual was active, yank the highlighted text and put it before
        // the mouse pointer position.  In Select mode replace the highlighted
        // text with the clipboard.
        if VIsual_active.get() {
            if VIsual_select.get() {
                stuff_char(Ctrl_G);
                // SAFETY: a NUL-terminated literal.
                unsafe { stuff_readbuf(c"\"+p".as_ptr()) };
            } else {
                stuff_char('y' as c_int);
                stuff_char(K_MIDDLEMOUSE);
            }
            return Some(false);
        }
        // The rest is below jump_to_mouse().
    } else if State.get() & MODE_INSERT == 0 {
        return Some(false);
    }

    // Middle click in insert mode doesn't move the mouse, just insert the
    // contents of a register.  '.' register is special, can't insert that
    // with do_put().
    // Also paste at the cursor if the current mode isn't in 'mouse' (only
    // happens for the GUI).
    if State.get() & MODE_INSERT == 0 {
        return None;
    }

    if regname == '.' as c_int {
        // SAFETY: inserts the register's text at the cursor.
        unsafe { insert_reg(regname, ptr::null_mut(), true) };
        return Some(false);
    }
    if regname == 0 && has_clipboard_provider() {
        regname = '*' as c_int;
    }

    // The register is looked up only in Replace mode, as the C's `&&` has it;
    // everywhere else `reg` stays null and `do_put` looks it up itself.
    let mut reg: *mut yankreg_T = ptr::null_mut();
    // SAFETY: `reg` is a live local, and the answer is a register the editor
    // owns.
    let replacing =
        State.get() & REPLACE_FLAG != 0 && !unsafe { yank_register_mline(regname, &raw mut reg) };
    if replacing {
        // SAFETY: `reg` is a register the editor owns.
        unsafe { insert_reg(regname, reg, true) };
        return Some(false);
    }

    let flags = if fixindent { PUT_FIXINDENT } else { 0 } as c_int | PUT_CURSEND as c_int;
    // SAFETY: as above.
    unsafe { do_put(regname, reg, BACKWARD as c_int, 1, flags) };
    // Repeat it with CTRL-R CTRL-O r or CTRL-R CTRL-P r
    // SAFETY: appends to the redo buffer.
    unsafe {
        append_to_redobuff_char(Ctrl_R);
        append_to_redobuff_char(if fixindent { Ctrl_P } else { Ctrl_O });
        append_to_redobuff_char(if regname == 0 { '"' as c_int } else { regname });
    }
    Some(false)
}

/// A click, drag or release in the tab page line: select, close, create or
/// move a tab page, or run a `%@Func@` handler.
fn tab_line_click(
    defs: ClickDefs,
    is_click: bool,
    is_drag: bool,
    which_button: c_int,
    old_curwin: Win,
) -> Option<bool> {
    if !(mouse_grid.get() <= 1 && mouse_row.get() == 0 && first_window_row() > 0) {
        if is_drag && in_tab_line.get() {
            move_tab_to_mouse(defs);
            return Some(false);
        }
        return None;
    }

    if is_drag {
        if in_tab_line.get() {
            move_tab_to_mouse(defs);
        }
        return Some(false);
    }

    // A click in a tab selects that tab page.
    if is_click && cmdwin_type.get() == 0 && mouse_col.get() < Columns.get() {
        let def = defs.at(mouse_col.get());
        in_tab_line.set(true);

        // The C's `kStlClickTabSwitch` arm falls through to
        // `kStlClickTabClose` for the middle button, which is what this flag
        // spells out.
        let mut close = def.type_0 == kStlClickTabClose;
        if def.type_0 == kStlClickTabSwitch {
            if which_button == MOUSE_MIDDLE {
                close = true;
            } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                // Double click opens a new page.
                end_visual_mode();
                tabpage_new();
                tabpage_move(if def.tabnr == 0 { 9999 } else { def.tabnr - 1 });
            } else {
                // Go to specified tab page, or next one if not clicking on a
                // label.
                goto_tabpage(def.tabnr);
                // It's like clicking on the status line of a window.
                if !old_curwin.is_current() {
                    end_visual_mode();
                }
            }
        } else if def.type_0 == kStlClickFuncRun {
            call_click_def_func(defs, mouse_col.get(), which_button);
            // kStlClickDisabled does nothing, and an unknown kind is not
            // reachable: the statusline builder writes only these four.
        }

        if close {
            mouse_tab_close(def.tabnr);
        }
    }
    Some(true)
}

/// A click on a window bar, status line or status column: run the `%@Func@`
/// handler recorded for the column, or open the popup menu where a status
/// column has none.
///
/// Answers `Some` unless the click also toggled a fold, which the caller goes
/// on to handle.
fn click_definition(
    landed: Landed,
    jump_flags: c_int,
    which_button: c_int,
    m_pos_flag: c_int,
    m_pos: pos_T,
) -> Option<bool> {
    let mut pos = MousePos::current();
    let Some(win) = find_win_inner(&mut pos) else {
        return Some(false);
    };

    let (mut click_defs, mut limit) = if landed.status_line {
        win.status_click_defs()
    } else if landed.winbar {
        (win.winbar_click_defs(), c_int::MAX)
    } else {
        win.statuscol_click_defs()
    };
    let mut click_col = pos.col;

    if landed.global_status_line {
        // The global statusline is displayed for the current window, and
        // spans the whole screen.
        // SAFETY: `curwin` is live from startup to exit.
        let (defs, size) = unsafe { Win::current() }.status_click_defs();
        (click_defs, limit) = (defs, size);
        click_col = mouse_col.get();
    }

    if landed.statuscol && win.w_onebuf_opt.wo_rl != 0 {
        click_col = win.w_view_width - click_col - 1;
    }

    if (landed.statuscol || landed.status_line) && click_col >= limit {
        return Some(false);
    }

    if let Some(click_defs) = click_defs {
        match click_defs.at(click_col).type_0 {
            kStlClickDisabled => {
                // If there is no click definition, still open the popupmenu
                // for a statuscolumn click like a click in the sign/number
                // column does.
                if landed.statuscol
                    && mouse_model_popup()
                    && which_button == MOUSE_RIGHT
                    && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
                {
                    do_popup(which_button, m_pos_flag, m_pos);
                }
            }
            kStlClickFuncRun => call_click_def_func(click_defs, click_col, which_button),
            _ => debug_assert!(
                false,
                "winbar, statusline and statuscolumn only support %@ for clicks"
            ),
        }
    }

    if landed.statuscol && jump_flags & (MOUSE_FOLD_CLOSE | MOUSE_FOLD_OPEN) != 0 {
        return None;
    }
    Some(false)
}

/// Everything the click and the modifiers decide once the cursor is where it
/// belongs: the middle-button paste, the quickfix and tag jumps, the
/// Shift-click search and the multi-click word or block selection.
struct Action {
    oap: Option<Oap>,
    which_button: c_int,
    is_click: bool,
    is_drag: bool,
    dir: c_int,
    count: c_int,
    fixindent: bool,
    regname: c_int,
    old_active: bool,
    mouse_can_visual: bool,
    landed: Landed,
}

fn dispatch_action(a: Action, win: Win) {
    let Action {
        oap,
        which_button,
        is_click,
        is_drag,
        mut dir,
        count,
        fixindent,
        mut regname,
        old_active,
        mouse_can_visual,
        landed,
    } = a;
    let mods = mod_mask.get();

    if which_button == MOUSE_MIDDLE {
        // Middle mouse click: Put text before cursor.
        if regname == 0 && has_clipboard_provider() {
            regname = '*' as c_int;
        }
        let mut reg: *mut yankreg_T = ptr::null_mut();
        // SAFETY: `reg` is a live local, and the answer is a register the
        // editor owns.
        let mline = unsafe { yank_register_mline(regname, &raw mut reg) };
        if if mline {
            mouse_past_bottom.get()
        } else {
            mouse_past_eol.get()
        } {
            dir = FORWARD as c_int;
        }

        let (c1, c2) = if fixindent {
            let open = dir == BACKWARD as c_int;
            (if open { '[' as c_int } else { ']' as c_int }, 'p' as c_int)
        } else {
            let after = dir == FORWARD as c_int;
            (if after { 'p' as c_int } else { 'P' as c_int }, NUL)
        };
        prep_redo(regname, count, NUL, c1, NUL, c2, NUL);

        // Remember where the paste started, so in edit() Insstart can be set
        // to this position.
        if restart_edit.get() != 0 {
            where_paste_started.set(win.w_cursor);
        }
        let flags = if fixindent { PUT_FIXINDENT } else { 0 } as c_int | PUT_CURSEND as c_int;
        // SAFETY: `reg` is a register the editor owns.
        unsafe { do_put(regname, reg, dir, count, flags) };
        return;
    }

    // SAFETY: `curbuf` is live from startup to exit.
    let buf = unsafe { Buf::current() };
    // SAFETY: a live buffer.
    let in_quickfix = unsafe { bt_quickfix(buf.raw()) };
    let double_click = mods & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK;
    if (mods & MOD_MASK_CTRL != 0 || double_click) && in_quickfix {
        // Ctrl-Mouse click or double click in a quickfix window jumps to the
        // error under the mouse pointer.  A null location list means it is a
        // quickfix window rather than a location list one.
        let cmd = if win.w_llist_ref.is_null() {
            c".cc"
        } else {
            c".ll"
        };
        // SAFETY: a NUL-terminated literal.
        unsafe { do_cmdline_cmd(cmd.as_ptr()) };
        got_click.set(false); // ignore drag&release now
        return;
    }

    if mods & MOD_MASK_CTRL != 0 || (buf.b_help && double_click) {
        // Ctrl-Mouse click (or double click in a help window) jumps to the
        // tag under the mouse pointer.
        if State.get() & MODE_INSERT != 0 {
            stuff_char(Ctrl_O);
        }
        stuff_char(Ctrl_RSB);
        got_click.set(false); // ignore drag&release now
        return;
    }

    if mods & MOD_MASK_SHIFT != 0 {
        // Shift-Mouse click searches for the next occurrence of the word
        // under the mouse pointer.
        if State.get() & MODE_INSERT != 0 || (VIsual_active.get() && VIsual_select.get()) {
            stuff_char(Ctrl_O);
        }
        stuff_char(if which_button == MOUSE_LEFT {
            '*' as c_int
        } else {
            '#' as c_int
        });
        return;
    }

    if landed.status_line || landed.sep_line {
        // Do nothing if on status line or vertical separator.
        // Handle double clicks otherwise.
        return;
    }

    if mods & MOD_MASK_MULTI_CLICK != 0
        && State.get() & (MODE_NORMAL | MODE_INSERT) != 0
        && mouse_can_visual
    {
        multi_click(win, oap, is_click, is_drag, mods);
        return;
    }

    if VIsual_active.get() && !old_active {
        VIsual_mode.set(if mods & MOD_MASK_ALT != 0 {
            Ctrl_V
        } else {
            'v' as c_int
        });
    }
}

/// A double, triple or quadruple click starts or widens a Visual selection: a
/// word, a line, or -- for a double click on a bracket -- the block it opens.
fn multi_click(mut win: Win, oap: Option<Oap>, is_click: bool, is_drag: bool, mods: c_int) {
    if is_click || !VIsual_active.get() {
        if VIsual_active.get() {
            orig_cursor.set(VIsual.get());
        } else {
            VIsual.set(win.w_cursor);
            orig_cursor.set(VIsual.get());
            VIsual_active.set(true);
            VIsual_reselect.set(1);
            // Start Select mode if 'selectmode' contains "mouse".
            may_start_select('o' as c_int);
            setmouse();
        }
        VIsual_mode.set(match mods & MOD_MASK_MULTI_CLICK {
            // Double click with ALT pressed makes it blockwise.
            MOD_MASK_2CLICK if mods & MOD_MASK_ALT != 0 => Ctrl_V,
            MOD_MASK_2CLICK => 'v' as c_int,
            MOD_MASK_3CLICK => 'V' as c_int,
            MOD_MASK_4CLICK => Ctrl_V,
            _ => VIsual_mode.get(),
        });
    }

    // A double click selects a word or a block.
    if mods & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
        let matched = is_click && select_matching_block(win, oap);
        if !matched && (is_click || is_drag) {
            // When not found a match or when dragging: extend to include a
            // word.
            if lt(win.w_cursor, orig_cursor.get()) {
                // SAFETY: the cursor is a live position in the current buffer.
                unsafe { find_start_of_word(win.cursor()) };
                with_visual(find_end_of_word);
            } else {
                with_visual(find_start_of_word);
                // Bytes of the character under the cursor, `None` at the end
                // of the line.
                // SAFETY: the cursor is a live position in the current buffer.
                let under = unsafe {
                    let p = get_cursor_pos_ptr();
                    (*p != NUL as c_char).then(|| utfc_ptr2len(p))
                };
                if let Some(bytes) = under.filter(|_| selection_exclusive()) {
                    win.w_cursor.col += bytes;
                }
                // SAFETY: the cursor is a live position in the current buffer.
                unsafe { find_end_of_word(win.cursor()) };
            }
        }
        win.w_set_curswant = 1;
    }

    if is_click {
        // SAFETY: only schedules a redraw of the current buffer.
        unsafe { redraw_curbuf_later(UPD_INVERTED) }; // update the inversion
    }
}

/// If the character under the cursor (skipping white space) is not a word
/// character, try finding a match and select a (), {}, [], #if/#endif, etc.
/// block.  Answers whether one was found.
fn select_matching_block(mut win: Win, oap: Option<Oap>) -> bool {
    let mut end_visual = win.w_cursor;
    // SAFETY: a live local position in the current buffer.
    let probe = unsafe { Pos::new(&raw mut end_visual) };
    while ascii_iswhite(char_at(probe)) {
        advance(probe);
    }

    let Some(mut oap) = oap else {
        return false;
    };
    oap.motion_type = kMTCharWise;

    if VIsual_mode.get() != 'v' as c_int {
        return false;
    }
    // SAFETY: reads the character-class tables.
    if unsafe { vim_iswordc(char_at(probe)) } {
        return false;
    }
    if !equalpos(win.w_cursor, VIsual.get()) {
        return false;
    }
    let Some(pos) = oap.findmatch() else {
        return false;
    };

    win.w_cursor = *pos;
    if oap.motion_type == kMTLineWise {
        VIsual_mode.set('V' as c_int);
    } else if selection_exclusive() {
        if lt(win.w_cursor, VIsual.get()) {
            VIsual.with_mut(|visual| visual.col += 1);
        } else {
            win.w_cursor.col += 1;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// One wrapper per neighbouring entry point

/// Push `c` back onto the input, so it is read as the next key.
fn stuff_char(c: c_int) {
    // SAFETY: appends to the stuff buffer.
    unsafe { stuff_readbuf_char(c) };
}

/// The character at `pos`, or NUL past the end of the line.
fn char_at(pos: Pos) -> c_int {
    // SAFETY: a live position in the current buffer.
    unsafe { gchar_pos(pos.raw()) }
}

/// Step `pos` one character forward, over a line break if need be.
fn advance(pos: Pos) {
    // SAFETY: a live position in the current buffer.
    unsafe { inc(pos.raw()) };
}

/// Whether a clipboard provider is loaded, so that an unnamed register should
/// become the `*` one.
fn has_clipboard_provider() -> bool {
    // SAFETY: a NUL-terminated literal; `throw_if_fast` is false.
    unsafe { eval_has_provider(c"clipboard".as_ptr(), false) }
}

/// Run `f` over the Visual anchor through a copy: the word walk it is used for
/// reads the buffer line, which must not happen with the cell borrowed.
fn with_visual(f: unsafe fn(Pos)) {
    let mut visual = VIsual.get();
    // SAFETY: a live local position in the current buffer.
    unsafe { f(Pos::new(&raw mut visual)) };
    VIsual.set(visual);
}
