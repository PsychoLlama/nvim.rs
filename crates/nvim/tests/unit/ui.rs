//! The UI layer's bookkeeping, as a `vim.ui_attach()` handler sees it.
//!
//! A handler registers the external widgets it draws; `ui_refresh` ORs
//! them into the attached UIs' intersection, and the editor then behaves
//! as though a UI had asked for them. The cases pin that bookkeeping:
//! `ext_messages` takes the message area out of the grid ('cmdheight' 0)
//! and brings `ext_cmdline` with it; the widgets are the union over every
//! handler and are recomputed when one goes; re-registering a namespace
//! replaces its handler; a handler that keeps raising is dropped after its
//! fourth error, with the error and the "Excessive errors" message
//! scheduled. And the wire arguments of the generic `redraw` events a
//! handler is offered: `msg_show`'s kind, content, `replace_last`,
//! `history` and `append`, one event per message, and the history flag a
//! message kept before any external UI attached leaves behind (upstream's,
//! pinned as it behaves). The oracle for moving
//! the `ext_*` state and the sinks' table.

#![cfg(not(miri))]

use crate::support::{Probe, editor_lock};

/// Register `ns` (a namespace name) for `widgets` (a Lua table body), with
/// a handler that appends `<ns>:<event>[:<details>]` to `_G.ui_events`
/// for every event but the window and grid ones.
fn attach(fx: &Probe, ns: &str, widgets: &str) {
    fx.run(&format!(
        "lua _G.ui_events = _G.ui_events or {{}}
         vim.ui_attach(vim.api.nvim_create_namespace('{ns}'), {{ {widgets} }}, function(event, ...)
           -- Window and grid events are the shared editor's, left over from
           -- whatever an earlier case closed; not what these cases are about.
           if event:match('^grid_') or event:match('^win_') then return end
           local args, text = {{ ... }}, event
           if event == 'msg_show' then
             local chunks = vim.tbl_map(function(c) return c[2] end, args[2])
             text = table.concat({{ event, args[1], table.concat(chunks, '|'),
               tostring(args[3]), tostring(args[4]), tostring(args[5]) }}, ':')
           end
           table.insert(_G.ui_events, '{ns}:' .. text)
         end)"
    ));
}

fn detach(fx: &Probe, ns: &str) {
    fx.run(&format!(
        "lua vim.ui_detach(vim.api.nvim_create_namespace('{ns}'))"
    ));
}

/// The events `cmd` offered the handlers.
fn events(fx: &Probe, cmd: &str) -> String {
    fx.run("lua _G.ui_events = {}");
    fx.run(cmd);
    fx.eval("luaeval('_G.ui_events')")
}

#[test]
fn ext_messages_takes_the_message_area_out_of_the_grid_and_brings_the_cmdline() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    assert_eq!(fx.eval("&cmdheight"), "1");
    attach(&fx, "ui_a", "ext_messages = true");
    assert_eq!(fx.eval("&cmdheight"), "0");
    // Only a UI drawing its own command line is told it went away.
    assert_eq!(
        events(&fx, "call feedkeys(':ab\\<Esc>', 'xt')"),
        "['ui_a:msg_show:empty::false:false:false', 'ui_a:cmdline_hide']"
    );
    detach(&fx, "ui_a");
    assert_eq!(fx.eval("&cmdheight"), "1");
    fx.run("lua _G.ui_events = nil");
}

#[test]
fn the_widgets_are_the_union_over_handlers_and_follow_a_detach() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    attach(&fx, "ui_a", "ext_messages = true");
    attach(&fx, "ui_b", "ext_popupmenu = true");
    assert_eq!(fx.eval("&cmdheight"), "0");
    // Every handler is offered every event, in registration order.
    assert_eq!(
        events(&fx, "echo 'q'"),
        "['ui_a:msg_show:echo:q:false:false:false', 'ui_b:msg_show:echo:q:false:false:false']"
    );
    detach(&fx, "ui_a");
    assert_eq!(fx.eval("&cmdheight"), "1", "nobody draws messages now");
    assert_eq!(events(&fx, "echo 'q'"), "[]", "so none are sent");
    // Re-registering a namespace replaces its handler and its widgets.
    attach(&fx, "ui_b", "ext_messages = true");
    assert_eq!(fx.eval("&cmdheight"), "0");
    assert_eq!(
        events(&fx, "echo 'r'"),
        "['ui_b:msg_show:echo:r:false:false:false']",
        "one handler, not two"
    );
    detach(&fx, "ui_b");
    assert_eq!(fx.eval("&cmdheight"), "1");
    fx.run("lua _G.ui_events = nil");
}

#[test]
fn msg_show_carries_the_kind_and_history_of_each_message() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    attach(&fx, "ui_a", "ext_messages = true");
    // A message kept while nothing drew messages leaves the history flag
    // up for the next one (see the case below); spend it first.
    events(&fx, "echo ''");
    let rows: &[(&str, &str)] = &[
        (
            "echo 'a' | echo 'b'",
            "['ui_a:msg_show:echo:a:false:false:false', 'ui_a:msg_show:echo:b:false:false:false']",
        ),
        (
            "echomsg 'x' 'y'",
            "['ui_a:msg_show:echomsg:x y:false:true:false']",
        ),
        (
            "lua print('p')",
            "['ui_a:msg_show:lua_print:p:false:true:false']",
        ),
        ("redraw", "[]"),
    ];
    for (cmd, want) in rows {
        assert_eq!(events(&fx, cmd), *want, "{cmd}");
    }
    detach(&fx, "ui_a");
    fx.run("messages clear");
    fx.run("lua _G.ui_events = nil");
}

#[test]
fn a_handler_that_keeps_raising_is_dropped_after_its_fourth_error() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    fx.run(
        "lua _G.ui_calls = 0
         vim.ui_attach(vim.api.nvim_create_namespace('ui_a'), { ext_messages = true },
           function() _G.ui_calls = _G.ui_calls + 1; error('boom') end)",
    );
    assert_eq!(fx.eval("&cmdheight"), "0");
    for _ in 0..4 {
        fx.run("echo 'x'");
    }
    assert_eq!(fx.eval("luaeval('_G.ui_calls')"), "4");
    assert_eq!(fx.eval("&cmdheight"), "1", "dropped, widgets and all");
    fx.run("echo 'x'");
    assert_eq!(fx.eval("luaeval('_G.ui_calls')"), "4", "never called again");
    // The errors are scheduled, not raised in the handler's caller: the
    // loop has to run for them to arrive.
    fx.run("let v:errmsg = ''");
    fx.run("messages clear");
    fx.run("sleep 1m");
    assert_eq!(
        fx.eval("v:errmsg"),
        "'Excessive errors in vim.ui_attach() callback (ns=ui_a)'"
    );
    let history = fx.output("messages");
    assert_eq!(
        history
            .matches("Error in \"msg_show\" UI event handler (ns=ui_a):")
            .count(),
        4,
        "history: {history:?}"
    );
    fx.run("messages clear");
    fx.run("lua _G.ui_calls = nil");
    fx.settle();
}

/// Upstream's `msg_ext_history` is cleared only by a flush that has an
/// external message UI to send to, so a message kept in the history while
/// none was attached leaves it set, and the first message after one
/// attaches is reported as kept too. Pinned as upstream behaves.
#[test]
fn a_message_kept_before_the_ui_attached_marks_the_next_as_history() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    fx.run("echomsg 'kept with no external UI'");
    attach(&fx, "ui_a", "ext_messages = true");
    assert_eq!(
        events(&fx, "echo 'a'"),
        "['ui_a:msg_show:echo:a:false:true:false']"
    );
    assert_eq!(
        events(&fx, "echo 'b'"),
        "['ui_a:msg_show:echo:b:false:false:false']"
    );
    detach(&fx, "ui_a");
    fx.run("messages clear");
    fx.run("lua _G.ui_events = nil");
}
