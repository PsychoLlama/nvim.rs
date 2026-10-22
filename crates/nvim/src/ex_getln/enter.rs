//! Entering and leaving the command line: the outer loop.
//!
//! [`command_line_enter`] is C's `command_line_enter` — it builds the
//! `CommandLineState`, publishes a fresh `ccline`, runs the key loop through
//! `state_enter`, and unwinds all of it again on the way out.
//! [`getcmdline`] and [`getcmdline_prompt`] are the two entry points every
//! caller outside this module uses.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::cmdexpand::{Expanded, WildMode, WildOpts};
use crate::cstr;
use crate::eval::typval::DictRef;
use crate::getchar::typeahead;
use crate::guard::{Allow, Depth};
use crate::keycodes::Key;
use crate::memory::XString;
use crate::message::emsg;
use crate::option::vars::{P_STL, P_TAL, P_WBR};
use crate::optionstr::OptString;
use crate::types::{
    BackslashEscape, ExpandContext, NUL, OptionSetFlags, kBoolVarFalse, kBoolVarTrue,
};
use crate::winlayer::windows as windows_in_curtab;
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;

// ---------------------------------------------------------------------------
// The completion the key loop drives.

impl CommandLineState {
    /// C's `nextwild(&s->xpc, mode, options, s->firstc != '@')`, which every
    /// wildmenu key spells with the same two trailing arguments.
    pub(crate) fn next_wild(&self, mode: WildMode, options: WildOpts) -> ::core::ffi::c_int {
        let escape = self.firstc != '@' as ::core::ffi::c_int;
        Cc::current().with_completion(|xpc| nextwild(xpc, mode, options, escape))
    }
}

/// C's `showmatches(&s->xpc, …)`.
pub(crate) fn show_matches(display_wildmenu: bool, display_list: bool, noselect: bool) -> Expanded {
    Cc::current().with_completion(|xpc| showmatches(xpc, display_wildmenu, display_list, noselect))
}

/// C's `expand_one(&s->xpc, NULL, NULL, options, mode)`.
pub(crate) fn expand_completion(options: WildOpts, mode: WildMode) -> Option<XString> {
    Cc::current().with_completion(|xpc| expand_one(xpc, None, None, options, mode))
}

/// The command line's completion, for a read that runs no user code: C's
/// `s->xpc`.
pub(crate) fn completion<R>(f: impl FnOnce(&Expand) -> R) -> R {
    f(Cc::current().xpc())
}

/// [`completion`] for a write.
pub(crate) fn completion_mut<R>(f: impl FnOnce(&mut Expand) -> R) -> R {
    let mut cc = Cc::current();
    f(cc.xpc_mut())
}

/// How many matches the command line's completion holds: C's
/// `s->xpc.xp_numfiles`.
pub(crate) fn match_count() -> ::core::ffi::c_int {
    completion(Expand::match_count)
}

/// An all-zero [`CommandLineState`]: the fields C's designated initialiser
/// leaves out, which the C zeroes for it.
fn command_line_state_init() -> CommandLineState {
    CommandLineState {
        firstc: 0,
        count: 0,
        indent: 0,
        c: 0,
        gotesc: false,
        do_abbr: false,
        lookfor: None,
        hiscnt: 0,
        save_hiscnt: 0,
        histype: 0,
        is_state: INCSEARCH_STATE_INIT,
        did_wild_list: false,
        wim_index: 0,
        save_msg_scroll: 0,
        save_state: 0,
        prev_cmdpos: 0,
        prev_cmdbuff: None,
        save_p_icm: XString::new(),
        skip_pum_redraw: false,
        some_key_typed: false,
        ignore_drag_release: false,
        break_ctrl_c: false,
        b_im: None,
        cmdline_type: 0,
        event_cmdlineleavepre_triggered: false,
        did_hist_navigate: false,
    }
}

/// Initialize the current command-line info.
pub(crate) fn init_ccline(firstc: ::core::ffi::c_int, indent: ::core::ffi::c_int) {
    let mut cc = Cc::current();
    cc.overstrike = 0; // always start in insert mode
    debug_assert!(indent >= 0);

    // Set some variables for redrawcmd().
    cc.cmdfirstc = if firstc == '@' as ::core::ffi::c_int {
        0
    } else {
        firstc
    };
    cc.cmdindent = if firstc > 0 { indent } else { 0 };

    // Allocate the initial command-line buffer.
    cc.open(indent + 50);
    cc.cmdpos = 0;

    cc.last_colors = ColoredCmdline::NONE;
    sb_text_start_cmdline();

    // Autoindent for :insert and :append.
    if firstc <= 0 {
        cc.cmdbuff.fill(usize::try_from(indent).unwrap_or(0), b' ');
        cc.cmdpos = indent;
        cc.cmdspos = indent;
    }
}

pub(crate) fn ui_ext_cmdline_hide(abort: bool) {
    if ui_has(kUICmdline) {
        cmdline_was_last_drawn.set(false);
        let mut cc = Cc::current();
        cc.redraw_state = kCmdRedrawNone;
        ui_call_cmdline_hide(cc.level as Integer, abort as Boolean);
    }
}

/// Set `v:event` to a dictionary describing the command line, for the
/// `CmdlineEnter`/`CmdlineLeave` autocommands.  Answers the dictionary,
/// which the caller hands back to `restore_v_event`.
///
pub(crate) fn cmdline_event_dict(save_v_event: &mut SaveVEvent, cmdtype: &CStr) -> DictRef {
    let dict = get_v_event(save_v_event);
    let _ = dict.edit().add_str(b"cmdtype", Some(cmdtype));
    let _ = dict
        .edit()
        .add_number(b"cmdlevel", Cc::current().level as VarNumber);
    dict.edit().set_keys_readonly();
    dict
}

/// Internal entry point for cmdline mode.
///
/// `count` is only used for incremental search, `indent` is the indent for
/// inside conditionals, and `clear_ccline` asks for `ccline` to be cleared
/// first.
pub(crate) fn command_line_enter(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    indent: ::core::ffi::c_int,
    clear_ccline: bool,
) -> Option<XString> {
    // Can be invoked recursively; identify each level.
    static cmdline_level: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    let level = Depth::of(&cmdline_level);

    let save_cmdpreview = cmdpreview.get();
    cmdpreview.set(false);
    let mut state = CommandLineState {
        firstc,
        count,
        indent,
        save_msg_scroll: msg_scroll.get(),
        save_state: State.get(),
        prev_cmdpos: -1,
        ignore_drag_release: true,
        ..command_line_state_init()
    };
    let s = &mut state;
    s.save_p_icm = p_icm(XString::from_cstr);
    init_incsearch_state(&mut s.is_state);

    let mut cc = Cc::current();
    let mut did_save_ccline = false;
    if cc.in_use() {
        // Currently ccline can never be in use if clear_ccline is false;
        // some changes would be needed if that ever stops holding.
        debug_assert!(clear_ccline);
        // Being called recursively. Since ccline is global we have to
        // suspend the current line and resume it when returning.
        save_cmdline();
        did_save_ccline = true;
    } else if clear_ccline {
        ccline.set(CMDLINE_INFO_INIT);
    }

    if s.firstc == -1 {
        s.firstc = NUL;
        s.break_ctrl_c = true;
    }

    init_ccline(s.firstc, s.indent);
    debug_assert!(cc.in_use());
    let prompt_id = last_prompt_id.get();
    last_prompt_id.set(prompt_id.wrapping_add(1));
    cc.prompt_id = prompt_id;
    cc.level = cmdline_level.get();

    let mut err: Error = Error::none();
    let mut firstcbuf = [0u8; 2];

    if cmdline_level.get() == 50 {
        // Somehow got into a loop recursively calling getcmdline(), bail
        // out. (C's `goto theend`.)
        emsg(gettext(e_command_too_recursive));
    } else {
        cc.xpc = Some(Expand::new());
        clear_cmdline_orig();

        cmdmsg_rl.set(
            Win::current().w_onebuf_opt.wo_rl != 0
                && Win::current().w_onebuf_opt.wo_rlc.first_byte() as ::core::ffi::c_int
                    == 's' as ::core::ffi::c_int
                && (s.firstc == '/' as ::core::ffi::c_int || s.firstc == '?' as ::core::ffi::c_int),
        );

        msg_grid_validate();

        redir_off.set(true); // don't redirect the typed command
        if !cmd_silent.get() {
            gotocmdline(true);
            redrawcmdprompt(); // draw the prompt or the indent
            cc.cmdspos = cmd_startcol();
        }
        let (input_fn, xp_context, xp_arg) = (cc.input_fn, cc.xp_context, cc.xp_arg.clone());
        let xpc = cc.xpc_mut();
        xpc.context = ExpandContext::Nothing;
        xpc.backslash = BackslashEscape::NONE;
        xpc.shell = false;
        if input_fn != 0 {
            xpc.context = xp_context;
            xpc.pattern = 0;
            xpc.arg = xp_arg;
        }

        // Avoid scrolling when called by a recursive do_cmdline(), e.g.
        // when doing ":@0" and register 0 doesn't contain a CR.
        msg_scroll.set(0);

        State.set(MODE_CMDLINE);

        if s.firstc == '/' as ::core::ffi::c_int
            || s.firstc == '?' as ::core::ffi::c_int
            || s.firstc == '@' as ::core::ffi::c_int
        {
            // Use ":lmap" mappings for the search pattern and input().
            let which = if Buf::current().b_p_imsearch == B_IMODE_USE_INSERT as OptInt {
                ImOption::Insert
            } else {
                ImOption::Search
            };
            s.b_im = Some((which, Buf::current().id()));
            if im_value(Buf::current(), which) == B_IMODE_LMAP as OptInt {
                State.set(State.get() | MODE_LANGMAP);
            }
        }

        setmouse();

        s.cmdline_type = if firstc > 0 {
            firstc
        } else {
            '-' as ::core::ffi::c_int
        };
        firstcbuf[0] = s.cmdline_type as u8;

        if has_event(AutoEvent::CmdlineEnter) {
            let mut save_v_event: SaveVEvent = SAVE_V_EVENT_INIT;
            let dict = cmdline_event_dict(&mut save_v_event, cstr::in_bytes(&firstcbuf));

            // C's TRY_WRAP. restore_v_event() runs *inside* the try here
            // and outside it at CmdlineLeave below; that asymmetry is
            // upstream's.
            let mut tstate: TryState = TRY_STATE_INIT;
            try_enter(&mut tstate);
            cmdline_autocmd(AutoEvent::CmdlineEnter, cstr::in_bytes(&firstcbuf));
            restore_v_event(dict, &mut save_v_event);
            err.absorb(try_leave(&tstate));

            if err.is_set() {
                if !ui_has(kUIMessages) {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                msg_scroll.set(1);
                msg_str_hl(err.message_or_empty(), HLF_E, true);
                err.clear();
                redrawcmd();
            }
            err = Error::none();
        }
        may_trigger_modechanged();

        init_history();
        s.hiscnt = get_hislen(); // an impossible history value
        s.histype = hist_char2type(s.firstc) as ::core::ffi::c_int;
        do_digraph(-1); // init digraph typeahead

        // If something above caused an error, reset the flags: we do
        // want to type and execute commands. The display may be messed
        // up a bit.
        if did_emsg.get() != 0 {
            redrawcmd();
        }

        // Redraw the statusline, in case it uses the current mode through
        // the mode() function.
        if !cmd_silent.get() && !exmode_active.get() {
            let mut found_one = false;
            for mut wp in windows_in_curtab() {
                if P_STL.first_byte() != 0
                    || wp.w_onebuf_opt.wo_stl.first_byte() as ::core::ffi::c_int != NUL
                    || P_WBR.first_byte() != 0
                    || wp.w_onebuf_opt.wo_wbr.first_byte() as ::core::ffi::c_int != NUL
                {
                    wp.w_redr_status = true;
                    found_one = true;
                }
            }
            if P_TAL.first_byte() != 0 {
                redraw_tabline.set(true);
                found_one = true;
            }
            if redraw_custom_title_later() {
                found_one = true;
            }
            if found_one {
                redraw_statuslines();
            }
        }

        did_emsg.set(0);
        got_int.set(false);
        state_enter(ModeState::command_line(s));

        // Trigger CmdlineLeavePre autocommands if not already triggered.
        if !s.event_cmdlineleavepre_triggered {
            set_vim_var_char(s.c); // set v:char
            trigger_cmd_autocmd(s.cmdline_type, AutoEvent::CmdlineLeavePre);
        }

        if has_event(AutoEvent::CmdlineLeave) {
            let mut save_v_event: SaveVEvent = SAVE_V_EVENT_INIT;
            let dict = cmdline_event_dict(&mut save_v_event, cstr::in_bytes(&firstcbuf));
            // Not readonly, unlike the keys above:
            let _ = dict.edit().add_bool(
                b"abort",
                if s.gotesc {
                    kBoolVarTrue
                } else {
                    kBoolVarFalse
                },
            );
            set_vim_var_char(s.c); // set v:char

            // C's TRY_WRAP; the error is printed further below, to avoid
            // redraw issues.
            let mut tstate: TryState = TRY_STATE_INIT;
            try_enter(&mut tstate);
            cmdline_autocmd(AutoEvent::CmdlineLeave, cstr::in_bytes(&firstcbuf));
            err.absorb(try_leave(&tstate));

            if dict_get_number(Some(&dict), b"abort") != 0 {
                s.gotesc = true;
            }
            restore_v_event(dict, &mut save_v_event);
        }

        cmdmsg_rl.set(false);

        // We could have reached here without a chance to clean up the
        // wildmenu, if a special key like <Esc> or <C-\> was used as
        // 'wildchar'. Clean up anyway, to avoid memory corruption.
        if cmdline_pum_active() {
            cmdline_pum_remove(false);
        } else {
            // A previous cmdline_pum_remove() may have deferred redraw.
            pum_check_clear();
        }
        wildmenu_cleanup(cc);
        s.did_wild_list = false;
        s.wim_index = 0;

        if let Some(mut xpc) = cc.xpc.take() {
            expand_cleanup(&mut xpc);
        }
        clear_cmdline_orig();

        finish_incsearch_highlighting(s.gotesc, &mut s.is_state, false);

        if cc.in_use() {
            // Put the line in the history buffer (":" and "=" only when
            // it was typed).
            if s.histype != HIST_INVALID
                && !cc.is_empty()
                && s.firstc != NUL
                && (s.some_key_typed || s.histype == HIST_SEARCH)
            {
                add_to_history(
                    s.histype,
                    cc.bytes(),
                    true,
                    if s.histype == HIST_SEARCH {
                        s.firstc as u8
                    } else {
                        NUL as u8
                    },
                );
                if s.firstc == ':' as ::core::ffi::c_int {
                    new_last_cmdline.set(Some(XString::from_bytes(cstr::until_nul(cc.bytes()))));
                }
            }

            if s.gotesc {
                abandon_cmdline();
            }
        }

        // If the screen was shifted up, redraw the whole screen (later).
        // If the line is too long, clear it, so that the ruler and the
        // shown command do not get printed in the middle of it.
        msg_check();
        if p_ch() == 0 as OptInt && !ui_has(kUIMessages) {
            set_must_redraw(UPD_VALID);
        }
        msg_scroll.set(s.save_msg_scroll);
        redir_off.set(false);

        if err.is_set() {
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            emsg(err.message_or_empty());
            did_emsg.set(0);
            err.clear();
        }

        // When the command line was typed, no need for a wait-return
        // prompt.
        if s.some_key_typed && !err.is_set() {
            need_wait_return.set(false);
        }

        set_option_direct(
            kOptInccommand,
            OptVal::from_cstr(s.save_p_icm.as_cstr()),
            OptionSetFlags::NONE,
            SID_NONE,
        );
        State.set(s.save_state);
        if cmdpreview.get() != save_cmdpreview {
            cmdpreview.set(save_cmdpreview); // restore the preview state
            redraw_all_later(UPD_SOME_VALID);
        }
        may_trigger_modechanged();
        setmouse();
        sb_text_end_cmdline();
    }

    // C's `theend:`; the saved 'inccommand' goes with the state.
    // C's pair of `xfree`s and the three fields it nulled after them.
    cc.last_colors = ColoredCmdline::NONE;

    if ui_has(kUICmdline) {
        // Emit cmdline_block in Ex mode unless there is no command line,
        // which happens with <C-\><C-N> (upstream #39021).
        if exmode_active.get() && cc.in_use() {
            ui_ext_cmdline_block_append(0, cstr::until_nul(cc.bytes()));
        }
        ui_ext_cmdline_hide(s.gotesc);
    }
    if !cmd_silent.get() {
        redraw_custom_title_later();
        status_redraw_all(); // redraw to show the mode change
    }

    drop(level);

    // The text leaves the command line here, which is what C's
    // `ccline.cmdbuff = NULL` meant. It has to come before the resume:
    // `restore_cmdline` overwrites `ccline`, and what it overwrites is
    // freed now that the command line owns its buffer.
    let answer = cc.release();
    if did_save_ccline {
        restore_cmdline();
    }
    answer
}

/// The key loop's check, run before every key is fetched.
pub(crate) fn command_line_check(s: &mut CommandLineState) -> ::core::ffi::c_int {
    let cc = Cc::current();

    s.prev_cmdpos = cc.cmdpos;
    // The copy below reuses the room the last one had.
    let mut prev = s.prev_cmdbuff.take().unwrap_or_default();

    // Don't redirect the typed command. Repeated, because a ":redir"
    // inside completion may switch it on.
    redir_off.set(true);
    quit_more.set(false); // reset after CTRL-D, which had a more-prompt

    // There can't really be a reason why an error that occurs while
    // typing a command should cause the command not to be executed.
    did_emsg.set(0);

    if ex_normal_busy.get() == 0 && stuff_empty() && typeahead().is_empty() {
        // There is no pending input from sources other than user input,
        // so Vim is going to wait for the user to type a key. Consider
        // the command line typed even if the next key triggers a mapping.
        s.some_key_typed = true;
    }

    // Trigger SafeState if nothing is pending.
    may_trigger_safestate(match_count() <= 0);

    let cc = Cc::current();
    if cc.in_use() {
        // C's `xstrdup(ccline.cmdbuff)`: the line up to its first NUL.
        prev.clear();
        prev.extend_from_slice(cstr::until_nul(cc.bytes()));
        s.prev_cmdbuff = Some(prev);
    }

    // Defer the screen update to avoid pum flicker during wildtrigger().
    if s.c == Key::Wild.code() && s.firstc != '@' as ::core::ffi::c_int {
        s.skip_pum_redraw = true;
    }

    cursorcmd(); // set the cursor on the right spot
    ui_cursor_shape();
    1
}

pub(crate) fn abandon_cmdline() {
    dealloc_cmdbuff();
    if msg_scrolled.get() == 0 {
        compute_cmdrow();
    }
    // Avoid overwriting a key prompt.
    if !Cc::current().one_key {
        msg(c"", 0);
        redraw_cmdline.set(true);
    }
}

/// Accept a command line starting with `firstc`:
///
/// - `:` — an Ex command line
/// - `/` or `?` — a search pattern
/// - `=` — an expression
/// - `@` — text for the `input()` function
/// - `>` — text for debug mode
/// - NUL — text for the `:insert` command
/// - -1 — like NUL, and break on CTRL-C
///
/// The line is collected in the command line's own buffer, which grows to fit.
/// `count` is only used for incremental search and `indent` is the indent
/// for inside conditionals.  Answers an allocated string, or NULL if there
/// is no command line.
///
/// Careful: this can be called recursively.
pub fn getcmdline(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    indent: ::core::ffi::c_int,
    _do_concat: bool,
) -> Option<XString> {
    command_line_enter(firstc, count, indent, true)
}

/// [`getcmdline_prompt`] with no prompt text, completion, colouring or mouse
/// report: the line typed, owned, or `None` when it was abandoned.
pub(crate) fn getcmdline_bare(firstc: ::core::ffi::c_int) -> Option<XString> {
    getcmdline_prompt(
        firstc,
        None,
        0,
        ExpandContext::Nothing,
        None,
        Callback::None,
        false,
        None,
    )
}

/// Get a command line with a prompt.
///
/// Prepared to be called recursively from [`getcmdline`] — `f_input()` does
/// exactly that when evaluating an expression from `<C-r>=`.
///
/// `firstc` is the prompt type (`@` for `input()`, `>` for debug), `prompt`
/// what is displayed before the user text, `hl_id` the prompt highlight,
/// `xp_context`/`xp_arg` the completion, `highlight_callback` the colouring
/// of the user's input, `one_key` returns after a single key press (button
/// prompts) and `mouse_used`, when given, says whether the prompt was left
/// by a click rather than a key (a number prompt's answer). Answers the line
/// typed, or `None`.
///
/// The prompt and the completion argument are copied: the command line keeps
/// them for as long as it is up, and user code runs meanwhile.
/// `highlight_callback` is taken over: the prompt's command line holds it
/// for the call, and it is released when the prompt returns.
#[allow(clippy::too_many_arguments)]
pub fn getcmdline_prompt(
    firstc: ::core::ffi::c_int,
    prompt: Option<&CStr>,
    hl_id: ::core::ffi::c_int,
    xp_context: ExpandContext,
    xp_arg: Option<&CStr>,
    highlight_callback: Callback,
    one_key: bool,
    mouse_used: Option<&mut bool>,
) -> Option<XString> {
    let msg_col_save = msg_col.get();
    let mut cc = Cc::current();

    let mut did_save_ccline = false;
    if cc.in_use() {
        // Suspend the current cmdline and resume it below.
        save_cmdline();
        did_save_ccline = true;
    } else {
        ccline.set(CMDLINE_INFO_INIT);
    }
    let prompt_id = last_prompt_id.get();
    last_prompt_id.set(prompt_id.wrapping_add(1));
    cc.prompt_id = prompt_id;
    cc.cmdprompt = prompt.map(XString::from_cstr);
    cc.hl_id = hl_id;
    cc.xp_context = xp_context;
    cc.xp_arg = xp_arg.map(XString::from_cstr);
    cc.input_fn = (firstc == '@' as ::core::ffi::c_int) as ::core::ffi::c_int;
    cc.highlight_callback = highlight_callback;
    cc.one_key = one_key;
    cc.mouse_prompt = mouse_used.is_some();
    cc.mouse_used = false;

    let cmd_silent_saved = cmd_silent.get();
    let loud = Allow::messages();
    cmd_silent.set(false); // want to see the prompt

    let ret = command_line_enter(firstc, 1, 0, false);
    cc.redraw_state = kCmdRedrawNone;
    if let Some(clicked) = mouse_used {
        *clicked = cc.mouse_used;
    }
    // Out of the line before the line it suspended comes back.
    let mut highlight_callback = ::core::mem::replace(&mut cc.highlight_callback, Callback::None);
    if did_save_ccline {
        restore_cmdline();
    }
    drop(loud);
    cmd_silent.set(cmd_silent_saved);
    if cc.in_use() {
        msg_col.set(msg_col_save);
    }
    highlight_callback.clear();
    ret
}

/// [`getcmdline_prompt`] for a prompt the caller holds, with no completion
/// argument and no mouse: the line typed, or `None`.
pub(crate) fn getcmdline_prompt_text(
    firstc: ::core::ffi::c_int,
    prompt: &::core::ffi::CStr,
    hl_id: ::core::ffi::c_int,
    xp_context: ExpandContext,
    highlight_callback: Callback,
    one_key: bool,
) -> Option<XString> {
    getcmdline_prompt(
        firstc,
        Some(prompt),
        hl_id,
        xp_context,
        None,
        highlight_callback,
        one_key,
        None,
    )
}

/// A buffer's `'iminsert'` or `'imsearch'`: what C's `b_im_ptr` read.
pub(crate) fn im_value(buffer: Buf, which: ImOption) -> OptInt {
    match which {
        ImOption::Insert => buffer.b_p_iminsert,
        ImOption::Search => buffer.b_p_imsearch,
    }
}

/// Write a buffer's `'iminsert'` or `'imsearch'`: `*b_im_ptr = value`.
pub(crate) fn set_im_value(mut buffer: Buf, which: ImOption, value: OptInt) {
    match which {
        ImOption::Insert => buffer.b_p_iminsert = value,
        ImOption::Search => buffer.b_p_imsearch = value,
    }
}
