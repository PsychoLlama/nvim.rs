//! `showmode()`: the mode message, as the editor builds it.
//!
//! The text is read where an external message UI reads it: a
//! `vim.ui_attach()` handler with `ext_messages` is sent every
//! `msg_showmode` event, so each case lists the messages a key sequence
//! produced, the empty one that takes the mode away included. Every
//! branch of the mode ladder is a row — Insert, Replace, Virtual Replace,
//! the three Visual and three Select shapes, 'paste', 'revins', the
//! `CTRL-O` "(insert)"/"(replace)" forms, the `CTRL-X` submode text, and
//! `recording @q` alone, after the mode, and with 'showmode' off — as the
//! oracle for splitting `showmode` into the text it chooses and the
//! drawing of it.
//!
//! A key sequence runs through `feedkeys(…, 'xt')`; Visual and Select mode
//! show their message from the redraw, which `<Cmd>redraw<CR>` asks for.

#![cfg(not(miri))]

use crate::support::{Probe, editor_lock};

/// Attach the handler that records the mode messages, as `_G.showmodes`.
fn attach(fx: &Probe) {
    fx.run(
        "lua _G.showmodes = {}
         _G.showmode_ns = vim.api.nvim_create_namespace('drawscreen_showmode')
         vim.ui_attach(_G.showmode_ns, { ext_messages = true }, function(event, content)
           if event == 'msg_showmode' then
             local text = {}
             for _, chunk in ipairs(content) do text[#text + 1] = chunk[2] end
             table.insert(_G.showmodes, table.concat(text))
           end
         end)",
    );
}

fn detach(fx: &Probe) {
    fx.run("lua vim.ui_detach(_G.showmode_ns); _G.showmodes = nil");
    fx.run("set showmode& paste& revins&");
    fx.run("let @q = ''");
    fx.home();
}

/// The mode messages `keys` (a double-quoted Vim string's body) produced.
fn showmodes(fx: &Probe, keys: &str) -> String {
    fx.run("lua _G.showmodes = {}");
    fx.run(&format!("call feedkeys(\"{keys}\", 'xt')"));
    fx.eval("luaeval('_G.showmodes')")
}

#[test]
fn each_mode_shows_its_own_message() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    fx.run("set showmode");
    attach(&fx);
    let rows: &[(&str, &str)] = &[
        ("i\\<Esc>", "['-- INSERT --', '']"),
        ("R\\<Esc>", "['-- REPLACE --', '']"),
        ("gR\\<Esc>", "['-- VREPLACE --', '']"),
        ("v\\<Cmd>redraw\\<CR>\\<Esc>", "['-- VISUAL --']"),
        ("V\\<Cmd>redraw\\<CR>\\<Esc>", "['-- VISUAL LINE --']"),
        (
            "\\<C-V>\\<Cmd>redraw\\<CR>\\<Esc>",
            "['-- VISUAL BLOCK --']",
        ),
        ("gh\\<Cmd>redraw\\<CR>\\<Esc>", "['-- SELECT --']"),
        ("gH\\<Cmd>redraw\\<CR>\\<Esc>", "['-- SELECT LINE --']"),
        (
            "g\\<C-H>\\<Cmd>redraw\\<CR>\\<Esc>",
            "['-- SELECT BLOCK --']",
        ),
    ];
    for (keys, want) in rows {
        assert_eq!(showmodes(&fx, keys), *want, "{keys}");
    }
    detach(&fx);
}

#[test]
fn options_and_pending_insert_change_the_wording() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    fx.run("set showmode");
    attach(&fx);
    fx.run("set paste");
    assert_eq!(showmodes(&fx, "i\\<Esc>"), "['-- INSERT (paste) --', '']");
    fx.run("set nopaste revins");
    assert_eq!(showmodes(&fx, "i\\<Esc>"), "['-- REVERSE INSERT --', '']");
    fx.run("set norevins");
    // CTRL-O leaves Insert mode for one command, and says it will return.
    assert_eq!(
        showmodes(&fx, "i\\<C-O>\\<Cmd>redraw\\<CR>\\<Esc>"),
        "['-- INSERT --', '-- (insert) --', '-- INSERT --', '']"
    );
    assert_eq!(
        showmodes(&fx, "R\\<C-O>\\<Cmd>redraw\\<CR>\\<Esc>"),
        "['-- REPLACE --', '-- (replace) --', '-- REPLACE --', '']"
    );
    // The CTRL-X submode replaces the mode name, and is budgeted on its own.
    assert_eq!(
        showmodes(&fx, "i\\<C-X>\\<Cmd>redraw\\<CR>\\<Esc>"),
        "['-- INSERT --', '-- ^X mode (^]^D^E^F^I^K^L^N^O^P^Rs^U^V^Y)', '']"
    );
    detach(&fx);
}

#[test]
fn recording_is_shown_after_the_mode_and_without_showmode() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    fx.run("set showmode");
    attach(&fx);
    assert_eq!(
        showmodes(&fx, "qq\\<Cmd>redraw\\<CR>q"),
        "['recording @q', '']"
    );
    assert_eq!(
        showmodes(&fx, "qqi\\<Esc>q"),
        "['recording @q', '-- INSERT --recording @q', 'recording @q', 'recording @q', '']"
    );
    fx.run("set noshowmode");
    assert_eq!(showmodes(&fx, "i\\<Esc>"), "[]", "no mode, no message");
    assert_eq!(
        showmodes(&fx, "qq\\<Cmd>redraw\\<CR>q"),
        "['recording @q', '']",
        "recording is shown whatever 'showmode' says"
    );
    detach(&fx);
}
