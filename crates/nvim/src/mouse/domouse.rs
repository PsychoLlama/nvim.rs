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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::*;
use crate::ascii::ascii_iswhite;
use crate::buffer::buf_is_quickfix;
use crate::charset::vim_iswordc;
use crate::cursor::with_cursor_rest;
use crate::drawscreen::state::{MODE_DISPLAYED, REDRAW_CMDLINE};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::eval::eval_has_provider;
use crate::ex_docmd::{do_cmdline_cmd, tabpage_new};
use crate::fold::{close_fold, open_fold};
use crate::getchar::state::{KEY_STUFFED, MOD_MASK};
use crate::getchar::{
    append_to_redobuff_char, safe_vgetc, stuff_readbuf, stuff_readbuf_char, stuff_readbuf_number,
    vpeekc, vungetc,
};
use crate::global_cell::GlobalCell;
use crate::keycodes::SpecialKey;
use crate::keycodes::{ModMask, mouse_button};
use crate::memline::{gchar_pos, inc};
use crate::message::state::MSG_SILENT;
use crate::mouse::state::{
    MOUSE_COL, MOUSE_DRAGGING, MOUSE_GRID, MOUSE_PAST_BOTTOM, MOUSE_PAST_EOL, MOUSE_ROW,
};
use crate::r#move::scroll_redraw;
use crate::normal::defs::MotionType;
use crate::normal::{
    VisualMode, clear_op, clear_op_beep, end_visual_mode, may_start_select, prep_redo,
    set_visual_active, set_visual_anchor, set_visual_mode, visual_active, visual_anchor,
    visual_mode, visual_select, with_visual_anchor,
};
use crate::ops::Op;
use crate::option::vars::p_smd;
use crate::pos::{equalpos, lt};
use crate::register::defs::{PUT_CURSEND, PUT_FIXINDENT};
use crate::register::{RegName, do_put, insert_reg, register_key, yank_register_mline};
use crate::search::{FORWARD, findmatch};
use crate::state::mode::{RESTART_EDIT, STATE, V_ISUAL_RESELECT, WHERE_PASTE_STARTED};
use crate::state::{MODE_INSERT, MODE_NORMAL, REPLACE_FLAG};
use crate::statusline::{
    STL_CLICK_DISABLED, STL_CLICK_FUNC_RUN, STL_CLICK_TAB_CLOSE, STL_CLICK_TAB_SWITCH,
};
use crate::types::{Direction, Key};
use crate::ui::state::COLUMNS;
use crate::ui::{MOUSE_VISUAL, ui_mouse_has};
use crate::window::{goto_tabpage, tabpage_move};
use crate::winlayer::graph::CMDWIN_TYPE;
use core::ops::ControlFlow;

/// Whether the last click was in the tab page line, so that its release is
/// ignored rather than treated as a click in a window.
static IN_TAB_LINE: GlobalCell<bool> = GlobalCell::new(false);
/// Where the multi-click selection started, which a drag extends away from.
static ORIG_CURSOR: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: LineNr::ZERO,
    col: ColNr::ZERO,
    coladd: 0,
});

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
/// @param op        operator argument, can be NULL
/// @param c          `K_LEFTMOUSE`, etc
/// @param dir        Direction to 'put' if necessary
/// @param fixindent  `PUT_FIXINDENT` if fixing indent necessary
///
/// @return           true if `start_arrow()` should be called for edit mode.
pub(crate) fn do_mouse(
    op: Option<Op>,
    c: Key,
    dir: Direction,
    count: c_int,
    fixindent: bool,
) -> bool {
    let (mut which_button, is_click, is_drag) = coalesce_drags(c);

    if c == SpecialKey::Mousemove {
        return false; // Mouse moved without a button pressed.
    }
    if let Some(answer) = ignore_stray_event(is_click, is_drag) {
        return answer;
    }
    if let Some(answer) = modifier_shortcuts(is_click, which_button, count) {
        return answer;
    }

    let regname = op.map_or(RegName::UNNAMED, |o| o.regname);
    if which_button == MOUSE_MIDDLE
        && let Some(answer) = middle_button_insert(op, regname, fixindent)
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
    let old_curwin = Win::current();

    // Only when initialized.
    if let Some(defs) = ClickDefs::tabline()
        && let Some(answer) = tab_line_click(defs, is_click, is_drag, which_button, old_curwin)
    {
        return answer;
    }

    let (m_pos_flag, m_pos) = match popup_translation(&mut which_button, is_click) {
        ControlFlow::Break(answer) => return answer,
        ControlFlow::Continue(at) => at,
    };

    let mouse_can_visual = ui_mouse_has(MOUSE_VISUAL);
    let visual_corner = visual_jump_flags(
        &mut jump_flags,
        is_click,
        which_button,
        mouse_can_visual,
        old_curwin,
    );

    // If an operator is pending, ignore all drags and releases until the next
    // mouse click.
    if let Some(mut op) = op.filter(|o| !is_drag && o.pending()) {
        GOT_CLICK.set(false);
        op.motion_type = MotionType::CharWise;
    }

    // When releasing the button let jump_to_mouse() know.
    if !is_click && !is_drag {
        jump_flags |= MOUSE_RELEASED;
    }

    // JUMP!
    let old_active = visual_active();
    let save_cursor = old_curwin.w_cursor;

    // Even though we gate *_VIS flags above, we want to make sure the cursor
    // doesn't move in visual mode unless it is set as a mouse option.
    if !visual_active() || mouse_can_visual {
        // Through a copy, put back after the jump: the jump can run
        // autocommands, which may read the pending operator.
        let mut inclusive = op.map(|op| op.inclusive);
        jump_flags = jump_to_mouse(jump_flags, inclusive.as_mut(), which_button);
        if let (Some(mut op), Some(inclusive)) = (op, inclusive) {
            op.inclusive = inclusive;
        }
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

    let win = Win::current();

    // When jumping to another window, clear a pending operator.  That's a bit
    // friendlier than beeping and not jumping to that window.
    if let Some(op) = op.filter(|o| win != old_curwin && o.pending()) {
        clear_op(op);
    }

    if MOD_MASK.get().is_empty()
        && !is_drag
        && jump_flags & (MOUSE_FOLD_CLOSE | MOUSE_FOLD_OPEN) != 0
        && which_button == MOUSE_LEFT
    {
        fold_click(win, jump_flags, old_curwin, save_cursor);
    }
    if is_drag {
        drag_scroll(win, landed);
    }

    let old_mode = visual_mode();
    if let Some((start_visual, end_visual)) = visual_corner {
        // Right click in Visual mode.
        extend_visual_block(win, start_visual, end_visual);
    } else if STATE.get() & MODE_INSERT != 0 && visual_active() {
        // If Visual mode started in insert mode, execute "CTRL-O".
        stuff_char(Key::CTRL_O);
    }

    dispatch_action(
        Action {
            op,
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

    show_visual_change(old_active, old_mode);
    moved
}

/// When 'mousemodel' is "popup" or "popup_setpos", translate mouse events:
/// right button up   -> pop-up menu
/// shift-left button -> right button
/// alt-left button   -> alt-right button
///
/// `Break` with [`do_mouse`]'s answer when the event was the menu's;
/// otherwise where the mouse is (`get_fpos_of_mouse`'s flag and position,
/// zero when 'mousemodel' is not a popup one).
fn popup_translation(which_button: &mut c_int, is_click: bool) -> ControlFlow<bool, (c_int, Pos)> {
    let mut m_pos = Pos {
        lnum: LineNr::ZERO,
        col: ColNr::ZERO,
        coladd: 0,
    };
    if !mouse_model_popup() {
        return ControlFlow::Continue((0, m_pos));
    }
    let m_pos_flag = get_fpos_of_mouse(&mut m_pos);
    let over_text = m_pos_flag & (IN_STATUS_LINE | MOUSE_WINBAR | MOUSE_STATUSCOL) == 0;
    if over_text
        && *which_button == MOUSE_RIGHT
        && !MOD_MASK.get().has(ModMask::SHIFT | ModMask::CTRL)
    {
        if !is_click {
            // Ignore right button release events, only shows the popup
            // menu on the button down event.
            return ControlFlow::Break(false);
        }
        return ControlFlow::Break(do_popup(*which_button, m_pos_flag, m_pos) & CURSOR_MOVED != 0);
    }
    // Only do this translation when mouse is over the buffer text.
    if over_text && *which_button == MOUSE_LEFT && MOD_MASK.get().has(ModMask::SHIFT | ModMask::ALT)
    {
        *which_button = MOUSE_RIGHT;
        MOD_MASK.set(MOD_MASK.get().without(ModMask::SHIFT));
    }
    ControlFlow::Continue((m_pos_flag, m_pos))
}

/// Open or close the fold the click landed on, keeping the cursor where it
/// was when the click stayed in the same window.
fn fold_click(win: Win, jump_flags: c_int, old_curwin: Win, save_cursor: Pos) {
    let mut win = win;
    let fold: fn(Pos, c_int) = if jump_flags & MOUSE_FOLD_OPEN != 0 {
        open_fold
    } else {
        close_fold
    };
    fold(win.w_cursor, 1);
    // Don't move the cursor if still in the same window.
    if win == old_curwin {
        win.w_cursor = save_cursor;
    }
}

/// A drag: minimize 'scrolloff' while extending Visual mode, and scroll down
/// when the mouse went above the window.
fn drag_scroll(win: Win, landed: Landed) {
    // Set global flag that we are extending the Visual area with mouse
    // dragging; temporarily minimize 'scrolloff'.
    if visual_active() && win.scrolloff() != 0 {
        // In the very first line, allow scrolling one line.
        MOUSE_DRAGGING.set(if MOUSE_ROW.get() == 0 { 2 } else { 1 });
    }

    // When dragging the mouse above the window, scroll down.
    if MOUSE_ROW.get() < 0 && !landed.status_line {
        scroll_redraw(0, 1);
        MOUSE_ROW.set(0);
    }
}

/// If Visual mode started, ended or changed kind, show it later.
fn show_visual_change(old_active: bool, old_mode: VisualMode) {
    if (!visual_active() && old_active && MODE_DISPLAYED.get())
        || (visual_active()
            && p_smd()
            && MSG_SILENT.get() == 0
            && (!old_active || visual_mode() != old_mode))
    {
        REDRAW_CMDLINE.set(true);
    }
}

// ---------------------------------------------------------------------------
// The stages

/// Read the button, click and drag flags out of the key code, swallowing a
/// run of identical drag events.
///
/// Speeds up dragging the status line.  Since characters added to the stuff
/// buffer further down need to come before the next character, this is not
/// done when the current character was stuffed.
fn coalesce_drags(c: Key) -> (c_int, bool, bool) {
    loop {
        let (which_button, is_click, is_drag) = mouse_button(c);
        if !is_drag || KEY_STUFFED.get() || vpeekc().is_nul() {
            return (which_button, is_click, is_drag);
        }

        let saved = MousePos::current();
        // Need to get the character, peeking doesn't get the actual one.
        let nc = safe_vgetc();
        if c == nc {
            // The next character is the same mouse event: use that one.
            continue;
        }
        vungetc(nc);
        MOUSE_GRID.set(saved.grid);
        MOUSE_ROW.set(saved.row);
        MOUSE_COL.set(saved.col);
        return (which_button, is_click, is_drag);
    }
}

/// Ignore drag and release events if we didn't get a click.
fn ignore_stray_event(is_click: bool, is_drag: bool) -> Option<bool> {
    if is_click {
        GOT_CLICK.set(true);
        return None;
    }
    if !GOT_CLICK.get() {
        return Some(false); // didn't get click, ignore
    }
    if !is_drag {
        GOT_CLICK.set(false); // release, reset got_click
        if IN_TAB_LINE.get() {
            IN_TAB_LINE.set(false);
            return Some(false);
        }
    }
    None
}

/// The events a modifier turns into something else entirely, or suppresses.
fn modifier_shortcuts(is_click: bool, which_button: c_int, count: c_int) -> Option<bool> {
    let mods = MOD_MASK.get();

    // CTRL right mouse button does CTRL-T.
    if is_click && mods.has(ModMask::CTRL) && which_button == MOUSE_RIGHT {
        if STATE.get() & MODE_INSERT != 0 {
            stuff_char(Key::CTRL_O);
        }
        if count > 1 {
            stuff_readbuf_number(count);
        }
        stuff_char(Key::CTRL_T);
        GOT_CLICK.set(false); // ignore drag&release now
        return Some(false);
    }

    // CTRL only works with left mouse button.
    if mods.has(ModMask::CTRL) && which_button != MOUSE_LEFT {
        return Some(false);
    }

    // When a modifier is down, ignore drag and release events, as well as
    // multiple clicks and the middle mouse button.
    // Accept shift-leftmouse drags when 'mousemodel' is "popup.*".
    if mods.has(ModMask::SHIFT | ModMask::CTRL | ModMask::ALT | ModMask::META)
        && (!is_click || mods.has(ModMask::MULTI_CLICK) || which_button == MOUSE_MIDDLE)
        && !(mods.has(ModMask::SHIFT | ModMask::ALT)
            && mouse_model_popup()
            && which_button == MOUSE_LEFT)
        && !(mods.has(ModMask::ALT) && !mouse_model_popup() && which_button == MOUSE_RIGHT)
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
fn middle_button_insert(op: Option<Op>, mut regname: RegName, fixindent: bool) -> Option<bool> {
    if STATE.get() == MODE_NORMAL {
        // If an operator was pending, we don't know what the user wanted to
        // do.  Go back to normal mode: Clear the operator and beep().
        if let Some(op) = op.filter(|o| o.pending()) {
            clear_op_beep(op);
            return Some(false);
        }
        // If visual was active, yank the highlighted text and put it before
        // the mouse pointer position.  In Select mode replace the highlighted
        // text with the clipboard.
        if visual_active() {
            if visual_select() {
                stuff_char(Key::CTRL_G);
                stuff_readbuf(b"\"+p");
            } else {
                stuff_char(Key::ascii(b'y'));
                stuff_char(SpecialKey::Middlemouse.into());
            }
            return Some(false);
        }
        // The rest is below jump_to_mouse().
    } else if STATE.get() & MODE_INSERT == 0 {
        return Some(false);
    }

    // Middle click in insert mode doesn't move the mouse, just insert the
    // contents of a register.  '.' register is special, can't insert that
    // with do_put().
    // Also paste at the cursor if the current mode isn't in 'mouse' (only
    // happens for the GUI).
    if STATE.get() & MODE_INSERT == 0 {
        return None;
    }

    if regname == b'.' {
        let _ = insert_reg(regname, None, true);
        return Some(false);
    }
    if regname.is_unnamed() && has_clipboard_provider() {
        regname = RegName::STAR;
    }

    // The register is looked up only in Replace mode, as the C's `&&` has it;
    // everywhere else `reg` stays null and `do_put` looks it up itself.
    let mut reg = None;
    let replacing = STATE.get() & REPLACE_FLAG != 0 && {
        let (mline, fetched) = yank_register_mline(regname);
        reg = fetched;
        !mline
    };
    if replacing {
        let _ = insert_reg(regname, reg.as_ref(), true);
        return Some(false);
    }

    let flags = if fixindent { PUT_FIXINDENT } else { 0 } | PUT_CURSEND;
    do_put(regname, reg.as_ref(), BACKWARD, 1, flags);
    // Repeat it with CTRL-R CTRL-O r or CTRL-R CTRL-P r
    append_to_redobuff_char(Key::ctrl(b'R'));
    append_to_redobuff_char(if fixindent {
        Key::ctrl(b'P')
    } else {
        Key::ctrl(b'O')
    });
    append_to_redobuff_char(register_key(if regname.is_unnamed() {
        RegName::QUOTE
    } else {
        regname
    }));
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
    if !(MOUSE_GRID.get() <= 1 && MOUSE_ROW.get() == 0 && first_window_row() > 0) {
        if is_drag && IN_TAB_LINE.get() {
            move_tab_to_mouse(defs);
            return Some(false);
        }
        return None;
    }

    if is_drag {
        if IN_TAB_LINE.get() {
            move_tab_to_mouse(defs);
        }
        return Some(false);
    }

    // A click in a tab selects that tab page.
    if is_click && CMDWIN_TYPE.get() == 0 && MOUSE_COL.get() < COLUMNS.get() {
        let def = defs.at(MOUSE_COL.get());
        IN_TAB_LINE.set(true);

        // The C's `STL_CLICK_TAB_SWITCH` arm falls through to
        // `STL_CLICK_TAB_CLOSE` for the middle button, which is what this flag
        // spells out.
        let mut close = def.type_0 == STL_CLICK_TAB_CLOSE;
        if def.type_0 == STL_CLICK_TAB_SWITCH {
            if which_button == MOUSE_MIDDLE {
                close = true;
            } else if MOD_MASK.get().masked(ModMask::MULTI_CLICK) == ModMask::TWO_CLICK {
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
        } else if def.type_0 == STL_CLICK_FUNC_RUN {
            call_click_def_func(defs, MOUSE_COL.get(), which_button);
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
    m_pos: Pos,
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
        let (defs, size) = Win::current().status_click_defs();
        (click_defs, limit) = (defs, size);
        click_col = MOUSE_COL.get();
    }

    if landed.statuscol && win.w_onebuf_opt.wo_rl != 0 {
        click_col = win.w_view_width - click_col - 1;
    }

    if (landed.statuscol || landed.status_line) && click_col >= limit {
        return Some(false);
    }

    if let Some(click_defs) = click_defs {
        match click_defs.at(click_col).type_0 {
            STL_CLICK_DISABLED => {
                // If there is no click definition, still open the popupmenu
                // for a statuscolumn click like a click in the sign/number
                // column does.
                if landed.statuscol
                    && mouse_model_popup()
                    && which_button == MOUSE_RIGHT
                    && !MOD_MASK.get().has(ModMask::SHIFT | ModMask::CTRL)
                {
                    do_popup(which_button, m_pos_flag, m_pos);
                }
            }
            STL_CLICK_FUNC_RUN => call_click_def_func(click_defs, click_col, which_button),
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
#[derive(Debug)]
struct Action {
    op: Option<Op>,
    which_button: c_int,
    is_click: bool,
    is_drag: bool,
    dir: Direction,
    count: c_int,
    fixindent: bool,
    regname: RegName,
    old_active: bool,
    mouse_can_visual: bool,
    landed: Landed,
}

fn dispatch_action(a: Action, win: Win) {
    let Action {
        op,
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
    let mods = MOD_MASK.get();

    if which_button == MOUSE_MIDDLE {
        // Middle mouse click: Put text before cursor.
        if regname.is_unnamed() && has_clipboard_provider() {
            regname = RegName::STAR;
        }
        let (mline, reg) = yank_register_mline(regname);
        if if mline {
            MOUSE_PAST_BOTTOM.get()
        } else {
            MOUSE_PAST_EOL.get()
        } {
            dir = FORWARD;
        }

        let (c1, c2) = if fixindent {
            let open = dir == BACKWARD;
            (
                if open {
                    Key::ascii(b'[')
                } else {
                    Key::ascii(b']')
                },
                Key::ascii(b'p'),
            )
        } else {
            let after = dir == FORWARD;
            (
                if after {
                    Key::ascii(b'p')
                } else {
                    Key::ascii(b'P')
                },
                Key::NUL,
            )
        };
        prep_redo(regname, count, Key::NUL, c1, Key::NUL, c2, Key::NUL);

        // Remember where the paste started, so in edit() Insstart can be set
        // to this position.
        if !RESTART_EDIT.get().is_nul() {
            WHERE_PASTE_STARTED.set(win.w_cursor);
        }
        let flags = if fixindent { PUT_FIXINDENT } else { 0 } | PUT_CURSEND;
        do_put(regname, reg.as_ref(), dir, count, flags);
        return;
    }

    let buf = Buf::current();
    let in_quickfix = buf_is_quickfix(Some(buf));
    let double_click = mods.masked(ModMask::MULTI_CLICK) == ModMask::TWO_CLICK;
    if (mods.has(ModMask::CTRL) || double_click) && in_quickfix {
        // Ctrl-Mouse click or double click in a quickfix window jumps to the
        // error under the mouse pointer.  A null location list means it is a
        // quickfix window rather than a location list one.
        let cmd = if win.w_llist_ref.is_none() {
            c".cc"
        } else {
            c".ll"
        };
        let _ = do_cmdline_cmd(cmd);
        GOT_CLICK.set(false); // ignore drag&release now
        return;
    }

    if mods.has(ModMask::CTRL) || (buf.b_help && double_click) {
        // Ctrl-Mouse click (or double click in a help window) jumps to the
        // tag under the mouse pointer.
        if STATE.get() & MODE_INSERT != 0 {
            stuff_char(Key::CTRL_O);
        }
        stuff_char(Key::CTRL_RSB);
        GOT_CLICK.set(false); // ignore drag&release now
        return;
    }

    if mods.has(ModMask::SHIFT) {
        // Shift-Mouse click searches for the next occurrence of the word
        // under the mouse pointer.
        if STATE.get() & MODE_INSERT != 0 || (visual_active() && visual_select()) {
            stuff_char(Key::CTRL_O);
        }
        stuff_char(Key::ascii(if which_button == MOUSE_LEFT {
            b'*'
        } else {
            b'#'
        }));
        return;
    }

    if landed.status_line || landed.sep_line {
        // Do nothing if on status line or vertical separator.
        // Handle double clicks otherwise.
        return;
    }

    if mods.has(ModMask::MULTI_CLICK)
        && STATE.get() & (MODE_NORMAL | MODE_INSERT) != 0
        && mouse_can_visual
    {
        win.multi_click(op, is_click, is_drag, mods);
        return;
    }

    if visual_active() && !old_active {
        set_visual_mode(if mods.has(ModMask::ALT) {
            VisualMode::BLOCK
        } else {
            VisualMode::CHAR
        });
    }
}

impl Win {
    /// A double, triple or quadruple click starts or widens a Visual selection: a
    /// word, a line, or -- for a double click on a bracket -- the block it opens.
    fn multi_click(mut self, op: Option<Op>, is_click: bool, is_drag: bool, mods: ModMask) {
        if is_click || !visual_active() {
            if visual_active() {
                ORIG_CURSOR.set(visual_anchor());
            } else {
                set_visual_anchor(self.w_cursor);
                ORIG_CURSOR.set(visual_anchor());
                set_visual_active(true);
                V_ISUAL_RESELECT.set(1);
                // Start Select mode if 'selectmode' contains "mouse".
                may_start_select(Key::ascii(b'o'));
                setmouse();
            }
            set_visual_mode(match mods.masked(ModMask::MULTI_CLICK) {
                // Double click with ALT pressed makes it blockwise.
                ModMask::TWO_CLICK if mods.has(ModMask::ALT) => VisualMode::BLOCK,
                ModMask::TWO_CLICK => VisualMode::CHAR,
                ModMask::THREE_CLICK => VisualMode::LINE,
                ModMask::FOUR_CLICK => VisualMode::BLOCK,
                _ => visual_mode(),
            });
        }

        // A double click selects a word or a block.
        if mods.masked(ModMask::MULTI_CLICK) == ModMask::TWO_CLICK {
            let matched = is_click && self.select_matching_block(op);
            if !matched && (is_click || is_drag) {
                // When not found a match or when dragging: extend to include a
                // word.
                if lt(self.w_cursor, ORIG_CURSOR.get()) {
                    self.with_cursor_copy(find_start_of_word);
                    with_visual(find_end_of_word);
                } else {
                    with_visual(find_start_of_word);
                    // Bytes of the character under the cursor, `None` at the end
                    // of the line.
                    let under = with_cursor_rest(|rest| {
                        (!rest.is_empty())
                            .then(|| c_int::try_from(cluster_len(rest)).unwrap_or(c_int::MAX))
                    });
                    if let Some(bytes) = under.filter(|_| sel_exclusive()) {
                        self.w_cursor.col += bytes;
                    }
                    self.with_cursor_copy(find_end_of_word);
                }
            }
            self.w_set_curswant = true;
        }

        if is_click {
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
        }
    }

    /// If the character under the cursor (skipping white space) is not a word
    /// character, try finding a match and select a (), {}, [], #if/#endif, etc.
    /// block.  Answers whether one was found.
    fn select_matching_block(mut self, op: Option<Op>) -> bool {
        let mut probe = self.w_cursor;
        while ascii_iswhite(gchar_pos(&probe)) {
            inc(&mut probe);
        }

        let Some(mut op) = op else {
            return false;
        };
        op.motion_type = MotionType::CharWise;

        if !visual_mode().is_char() {
            return false;
        }
        if vim_iswordc(gchar_pos(&probe)) {
            return false;
        }
        if !equalpos(self.w_cursor, visual_anchor()) {
            return false;
        }
        let Some(pos) = findmatch(Some(&mut op), Codepoint::NUL) else {
            return false;
        };

        self.w_cursor = pos;
        if op.motion_type == MotionType::LineWise {
            set_visual_mode(VisualMode::LINE);
        } else if sel_exclusive() {
            if lt(self.w_cursor, visual_anchor()) {
                with_visual_anchor(|visual| visual.col += 1);
            } else {
                self.w_cursor.col += 1;
            }
        }
        true
    }

    /// Run `f` over `self`'s cursor through a copy, put back afterwards.
    fn with_cursor_copy(mut self, f: fn(&mut Pos)) {
        let mut cursor = self.w_cursor;
        f(&mut cursor);
        self.w_cursor = cursor;
    }
}

// ---------------------------------------------------------------------------
// One wrapper per neighbouring entry point

/// Push `c` back onto the input, so it is read as the next key.
fn stuff_char(c: Key) {
    stuff_readbuf_char(c);
}

/// Whether a clipboard provider is loaded, so that an unnamed register should
/// become the `*` one.
fn has_clipboard_provider() -> bool {
    eval_has_provider(c"clipboard")
}

/// Run `f` over the Visual anchor through a copy: the word walk it is used for
/// reads the buffer line, which must not happen with the cell borrowed.
fn with_visual(f: fn(&mut Pos)) {
    let mut visual = visual_anchor();
    f(&mut visual);
    set_visual_anchor(visual);
}
