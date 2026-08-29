//! Entering and leaving the command line: the outer loop.
//!
//! [`command_line_enter`] is C's `command_line_enter` — it builds the
//! `CommandLineState`, publishes a fresh `ccline`, runs the key loop through
//! `state_enter`, and unwinds all of it again on the way out.
//! [`getcmdline`] and [`getcmdline_prompt`] are the two entry points every
//! caller outside this module uses.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{Expanded, WildMode, WildOpts};
use crate::drawscreen::windows_in_curtab;
use crate::getchar::typeahead;
use crate::guard::Allow;
use crate::types::{
    BackslashEscape, ExpandContext, NUL, OptionSetFlags, kBoolVarFalse, kBoolVarTrue,
    kErrorTypeNone,
};
use crate::winlayer::Live;
use crate::winlayer::{Buf, Win};

// ---------------------------------------------------------------------------
// The command-line state, as a live handle.

/// [`Live`]'s shape for the state the command-line key loop threads through
/// every one of its callees.
///
/// C passes `CommandLineState *s` to twenty-odd functions and reaches every
/// field through it; the structure lives in [`command_line_enter`]'s frame
/// for exactly as long as the loop runs, which is the promise
/// [`Live::new`] records. Field access through it is ordinary checked code.
pub(crate) type Cls = Live<CommandLineState>;

/// [`Live`]'s shape for the `'incsearch'` preview state, which lives inside
/// a [`Cls`] as its `is_state` field.
pub(crate) type Is = Live<incsearch_state_T>;

impl Cls {
    /// `&s->is_state`, computed without reading the state.
    pub(crate) fn is_state(self) -> Is {
        // SAFETY: a field of a live `CommandLineState` is live with it.
        unsafe { Is::new(self.field_ptr(core::mem::offset_of!(CommandLineState, is_state))) }
    }

    /// `&s->xpc`, for the completion machinery that still takes a pointer.
    pub(crate) fn xpc(self) -> *mut expand_T {
        self.field_ptr(core::mem::offset_of!(CommandLineState, xpc))
    }

    /// `&s->state`, the `VimState` header `state_enter` is handed.
    pub(crate) fn vim_state(self) -> *mut VimState {
        self.field_ptr(core::mem::offset_of!(CommandLineState, state))
    }

    /// C's `nextwild(&s->xpc, mode, options, s->firstc != '@')`, which every
    /// wildmenu key spells with the same two trailing arguments.
    pub(crate) fn next_wild(self, mode: WildMode, options: WildOpts) -> ::core::ffi::c_int {
        let escape = self.firstc != '@' as ::core::ffi::c_int;
        // SAFETY: `xpc()` addresses the completion state inside this live
        // `CommandLineState`, which is what `nextwild` asks for.
        unsafe { nextwild(self.xpc(), mode, options, escape) }
    }

    /// C's `showmatches(&s->xpc, …)`.
    pub(crate) fn show_matches(
        self,
        display_wildmenu: bool,
        display_list: bool,
        noselect: bool,
    ) -> Expanded {
        // SAFETY: as [`Cls::next_wild`].
        unsafe { showmatches(self.xpc(), display_wildmenu, display_list, noselect) }
    }

    /// C's `expand_one(&s->xpc, NULL, NULL, options, mode)`.
    pub(crate) fn expand(self, options: WildOpts, mode: WildMode) -> *mut ::core::ffi::c_char {
        let nothing = ::core::ptr::null_mut::<::core::ffi::c_char>();
        // SAFETY: as [`Cls::next_wild`]; a null `str`/`orig` is what C passes
        // for the modes that do not answer with the original text.
        unsafe { expand_one(self.xpc(), nothing, nothing, options, mode) }
    }
}

/// An all-zero [`CommandLineState`]: the fields C's designated initialiser
/// leaves out, which the C zeroes for it.
const COMMAND_LINE_STATE_INIT: CommandLineState = CommandLineState {
    state: VimState {
        check: None,
        execute: None,
    },
    firstc: 0,
    count: 0,
    indent: 0,
    c: 0,
    gotesc: false,
    do_abbr: false,
    lookfor: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    lookforlen: 0,
    hiscnt: 0,
    save_hiscnt: 0,
    histype: 0,
    is_state: INCSEARCH_STATE_INIT,
    did_wild_list: false,
    wim_index: 0,
    save_msg_scroll: 0,
    save_State: 0,
    prev_cmdpos: 0,
    prev_cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    save_p_icm: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    skip_pum_redraw: false,
    some_key_typed: false,
    ignore_drag_release: false,
    break_ctrl_c: false,
    xpc: EXPAND_T_INIT,
    b_im_ptr: ::core::ptr::null_mut::<OptInt>(),
    b_im_ptr_buf: ::core::ptr::null_mut::<buf_T>(),
    cmdline_type: 0,
    event_cmdlineleavepre_triggered: false,
    did_hist_navigate: false,
};

/// Initialize the current command-line info.
pub(crate) unsafe fn init_ccline(firstc: ::core::ffi::c_int, indent: ::core::ffi::c_int) {
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
    unsafe { sb_text_start_cmdline() };

    // Autoindent for :insert and :append.
    if firstc <= 0 {
        unsafe {
            memset(
                cc.text() as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                indent as size_t,
            )
        };
        unsafe { *cc.at(indent) = NUL as ::core::ffi::c_char };
        cc.cmdpos = indent;
        cc.cmdspos = indent;
        cc.set_len(indent);
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
pub(crate) unsafe fn cmdline_event_dict(
    save_v_event: *mut save_v_event_T,
    cmdtype: *const ::core::ffi::c_char,
) -> *mut dict_T {
    let dict = unsafe { get_v_event(save_v_event) };
    unsafe { tv_dict_add_str(dict, c"cmdtype".as_ptr(), c"cmdtype".count_bytes(), cmdtype) };
    unsafe {
        tv_dict_add_nr(
            dict,
            c"cmdlevel".as_ptr(),
            c"cmdlevel".count_bytes(),
            Cc::current().level as varnumber_T,
        )
    };
    unsafe { tv_dict_set_keys_readonly(dict) };
    dict
}

/// Internal entry point for cmdline mode.
///
/// `count` is only used for incremental search, `indent` is the indent for
/// inside conditionals, and `clear_ccline` asks for `ccline` to be cleared
/// first.
pub(crate) unsafe fn command_line_enter(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    indent: ::core::ffi::c_int,
    clear_ccline: bool,
) -> *mut uint8_t {
    // Can be invoked recursively; identify each level.
    static cmdline_level: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    cmdline_level.set(cmdline_level.get() + 1);

    let save_cmdpreview = cmdpreview.get();
    cmdpreview.set(false);
    let mut state = CommandLineState {
        firstc,
        count,
        indent,
        save_msg_scroll: msg_scroll.get(),
        save_State: State.get(),
        prev_cmdpos: -1,
        ignore_drag_release: true,
        ..COMMAND_LINE_STATE_INIT
    };
    // SAFETY: `state` lives in this frame for the whole of the key loop.
    let mut s = unsafe { Cls::new(&raw mut state) };
    s.save_p_icm = unsafe { xstrdup(p_icm.get()) };
    unsafe { init_incsearch_state(s.is_state()) };

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

    unsafe { init_ccline(s.firstc, s.indent) };
    debug_assert!(cc.in_use());
    let prompt_id = last_prompt_id.get();
    last_prompt_id.set(prompt_id.wrapping_add(1));
    cc.prompt_id = prompt_id;
    cc.level = cmdline_level.get();

    let mut err: Error = ERROR_INIT;
    let mut firstcbuf: [::core::ffi::c_char; 2] = [0; 2];

    if cmdline_level.get() == 50 {
        // Somehow got into a loop recursively calling getcmdline(), bail
        // out. (C's `goto theend`.)
        unsafe { emsg(gettext(e_command_too_recursive.as_ptr())) };
    } else {
        unsafe { expand_init(s.xpc()) };
        cc.xpc = s.xpc();
        unsafe { clear_cmdline_orig() };

        cmdmsg_rl.set(
            cur_win().w_onebuf_opt.wo_rl != 0
                && unsafe { *cur_win().w_onebuf_opt.wo_rlc } as ::core::ffi::c_int
                    == 's' as ::core::ffi::c_int
                && (s.firstc == '/' as ::core::ffi::c_int || s.firstc == '?' as ::core::ffi::c_int),
        );

        unsafe { msg_grid_validate() };

        redir_off.set(true); // don't redirect the typed command
        if !cmd_silent.get() {
            unsafe { gotocmdline(true) };
            unsafe { redrawcmdprompt() }; // draw the prompt or the indent
            cc.cmdspos = cmd_startcol();
        }
        s.xpc.xp_context = ExpandContext::Nothing;
        s.xpc.xp_backslash = BackslashEscape::NONE;
        s.xpc.xp_shell = false;

        if cc.input_fn != 0 {
            s.xpc.xp_context = cc.xp_context;
            s.xpc.xp_pattern = cc.text();
            s.xpc.xp_arg = cc.xp_arg;
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
            s.b_im_ptr = if cur_buf().b_p_imsearch == B_IMODE_USE_INSERT as OptInt {
                cur_buf_iminsert()
            } else {
                cur_buf_imsearch()
            };
            s.b_im_ptr_buf = curbuf.get();
            if unsafe { *s.b_im_ptr } == B_IMODE_LMAP as OptInt {
                State.set(State.get() | MODE_LANGMAP);
            }
        }

        setmouse();

        s.cmdline_type = if firstc > 0 {
            firstc
        } else {
            '-' as ::core::ffi::c_int
        };
        firstcbuf[0] = s.cmdline_type as ::core::ffi::c_char;

        if has_event(EVENT_CMDLINEENTER) {
            let mut save_v_event: save_v_event_T = SAVE_V_EVENT_INIT;
            let dict = unsafe { cmdline_event_dict(&raw mut save_v_event, firstcbuf.as_ptr()) };

            // C's TRY_WRAP. restore_v_event() runs *inside* the try here
            // and outside it at CmdlineLeave below; that asymmetry is
            // upstream's.
            let mut tstate: TryState = TRY_STATE_INIT;
            unsafe { try_enter(&raw mut tstate) };
            cmdline_autocmd(EVENT_CMDLINEENTER, firstcbuf.as_mut_ptr());
            unsafe { restore_v_event(dict, &raw mut save_v_event) };
            unsafe { try_leave(&raw mut tstate, &raw mut err) };

            if err.type_0 != kErrorTypeNone {
                if !ui_has(kUIMessages) {
                    unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
                }
                msg_scroll.set(1);
                unsafe { msg_puts_hl(err.msg, HLF_E, true) };
                unsafe { api_clear_error(&raw mut err) };
                unsafe { redrawcmd() };
            }
            err = ERROR_INIT;
        }
        unsafe { may_trigger_modechanged() };

        init_history();
        s.hiscnt = get_hislen(); // an impossible history value
        s.histype = hist_char2type(s.firstc) as ::core::ffi::c_int;
        do_digraph(-1); // init digraph typeahead

        // If something above caused an error, reset the flags: we do
        // want to type and execute commands. The display may be messed
        // up a bit.
        if did_emsg.get() != 0 {
            unsafe { redrawcmd() };
        }

        // Redraw the statusline, in case it uses the current mode through
        // the mode() function.
        if !cmd_silent.get() && !exmode_active.get() {
            let mut found_one = false;
            for wp in windows_in_curtab() {
                if unsafe { *p_stl.get() } as ::core::ffi::c_int != NUL
                    || unsafe { *(*wp).w_onebuf_opt.wo_stl } as ::core::ffi::c_int != NUL
                    || unsafe { *p_wbr.get() } as ::core::ffi::c_int != NUL
                    || unsafe { *(*wp).w_onebuf_opt.wo_wbr } as ::core::ffi::c_int != NUL
                {
                    unsafe { (*wp).w_redr_status = true };
                    found_one = true;
                }
            }
            if unsafe { *p_tal.get() } as ::core::ffi::c_int != NUL {
                redraw_tabline.set(true);
                found_one = true;
            }
            if unsafe { redraw_custom_title_later() } {
                found_one = true;
            }
            if found_one {
                unsafe { redraw_statuslines() };
            }
        }

        did_emsg.set(0);
        got_int.set(false);
        s.state.check = Some(command_line_check);
        s.state.execute = Some(command_line_execute);

        unsafe { state_enter(s.vim_state()) };

        // Trigger CmdlineLeavePre autocommands if not already triggered.
        if !s.event_cmdlineleavepre_triggered {
            unsafe { set_vim_var_char(s.c) }; // set v:char
            trigger_cmd_autocmd(s.cmdline_type, EVENT_CMDLINELEAVEPRE);
        }

        if has_event(EVENT_CMDLINELEAVE) {
            let mut save_v_event: save_v_event_T = SAVE_V_EVENT_INIT;
            let dict = unsafe { cmdline_event_dict(&raw mut save_v_event, firstcbuf.as_ptr()) };
            // Not readonly, unlike the keys above:
            unsafe {
                tv_dict_add_bool(
                    dict,
                    c"abort".as_ptr(),
                    c"abort".count_bytes(),
                    if s.gotesc {
                        kBoolVarTrue
                    } else {
                        kBoolVarFalse
                    },
                )
            };
            unsafe { set_vim_var_char(s.c) }; // set v:char

            // C's TRY_WRAP; the error is printed further below, to avoid
            // redraw issues.
            let mut tstate: TryState = TRY_STATE_INIT;
            unsafe { try_enter(&raw mut tstate) };
            cmdline_autocmd(EVENT_CMDLINELEAVE, firstcbuf.as_mut_ptr());
            unsafe { try_leave(&raw mut tstate, &raw mut err) };

            if unsafe { tv_dict_get_number(dict, c"abort".as_ptr()) } != 0 {
                s.gotesc = true;
            }
            unsafe { restore_v_event(dict, &raw mut save_v_event) };
        }

        cmdmsg_rl.set(false);

        // We could have reached here without a chance to clean up the
        // wildmenu, if a special key like <Esc> or <C-\> was used as
        // 'wildchar'. Clean up anyway, to avoid memory corruption.
        if cmdline_pum_active() {
            unsafe { cmdline_pum_remove(false) };
        } else {
            // A previous cmdline_pum_remove() may have deferred redraw.
            unsafe { pum_check_clear() };
        }
        unsafe { wildmenu_cleanup(cc) };
        s.did_wild_list = false;
        s.wim_index = 0;

        unsafe { expand_cleanup(s.xpc()) };
        cc.xpc = ::core::ptr::null_mut::<expand_T>();
        unsafe { clear_cmdline_orig() };

        unsafe { finish_incsearch_highlighting(s.gotesc, s.is_state(), false) };

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
                    unsafe {
                        ::core::slice::from_raw_parts(cc.text() as *const u8, cc.len() as usize)
                    },
                    true,
                    if s.histype == HIST_SEARCH {
                        s.firstc as u8
                    } else {
                        NUL as u8
                    },
                );
                if s.firstc == ':' as ::core::ffi::c_int {
                    unsafe { xfree(new_last_cmdline.get() as *mut ::core::ffi::c_void) };
                    new_last_cmdline.set(unsafe { xstrnsave(cc.text(), cc.len() as size_t) });
                }
            }

            if s.gotesc {
                unsafe { abandon_cmdline() };
            }
        }

        // If the screen was shifted up, redraw the whole screen (later).
        // If the line is too long, clear it, so that the ruler and the
        // shown command do not get printed in the middle of it.
        unsafe { msg_check() };
        if p_ch.get() == 0 as OptInt && !ui_has(kUIMessages) {
            set_must_redraw(UPD_VALID);
        }
        msg_scroll.set(s.save_msg_scroll);
        redir_off.set(false);

        if err.type_0 != kErrorTypeNone {
            if !ui_has(kUIMessages) {
                unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
            }
            unsafe { emsg(err.msg) };
            did_emsg.set(0);
            unsafe { api_clear_error(&raw mut err) };
        }

        // When the command line was typed, no need for a wait-return
        // prompt.
        if s.some_key_typed && err.type_0 == kErrorTypeNone {
            need_wait_return.set(false);
        }

        set_option_direct(
            kOptInccommand,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe { cstr_as_string(s.save_p_icm) },
                },
            },
            OptionSetFlags::NONE,
            SID_NONE,
        );
        State.set(s.save_State);
        if cmdpreview.get() != save_cmdpreview {
            cmdpreview.set(save_cmdpreview); // restore the preview state
            unsafe { redraw_all_later(UPD_SOME_VALID) };
        }
        unsafe { may_trigger_modechanged() };
        setmouse();
        unsafe { sb_text_end_cmdline() };
    }

    // C's `theend:`.
    unsafe { xfree(s.save_p_icm as *mut ::core::ffi::c_void) };
    // C's pair of `xfree`s and the three fields it nulled after them.
    cc.last_colors = ColoredCmdline::NONE;

    if ui_has(kUICmdline) {
        // Emit cmdline_block in Ex mode unless there is no command line,
        // which happens with <C-\><C-N> (upstream #39021).
        if exmode_active.get() && cc.in_use() {
            unsafe { ui_ext_cmdline_block_append(0, cc.text()) };
        }
        ui_ext_cmdline_hide(s.gotesc);
    }
    if !cmd_silent.get() {
        unsafe { redraw_custom_title_later() };
        unsafe { status_redraw_all() }; // redraw to show the mode change
    }

    cmdline_level.set(cmdline_level.get() - 1);

    // The text leaves the command line here, which is what C's
    // `ccline.cmdbuff = NULL` meant. It has to come before the resume:
    // `restore_cmdline` overwrites `ccline`, and what it overwrites is
    // freed now that the command line owns its buffer.
    let answer = cc.release();
    if did_save_ccline {
        restore_cmdline();
    }

    unsafe { xfree(s.prev_cmdbuff as *mut ::core::ffi::c_void) };
    answer as *mut uint8_t
}

/// The key loop's `state_check` callback, run before every key is fetched.
/// Installed in a `VimState`, so this one keeps its C ABI.
pub(crate) unsafe fn command_line_check(state: *mut VimState) -> ::core::ffi::c_int {
    // SAFETY: `state_enter` hands back the `VimState` header of the
    // `CommandLineState` it was given, live for the whole of the loop.
    let mut s = unsafe { Cls::new(state.cast::<CommandLineState>()) };
    let mut cc = Cc::current();

    s.prev_cmdpos = cc.cmdpos;
    unsafe { xfree(s.prev_cmdbuff as *mut ::core::ffi::c_void) };
    s.prev_cmdbuff = ::core::ptr::null_mut();

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
    unsafe { may_trigger_safestate(s.xpc.xp_numfiles <= 0) };

    if cc.in_use() {
        s.prev_cmdbuff = unsafe { xstrdup(cc.text()) };
    }

    // Defer the screen update to avoid pum flicker during wildtrigger().
    if s.c == K_WILD && s.firstc != '@' as ::core::ffi::c_int {
        s.skip_pum_redraw = true;
    }

    unsafe { cursorcmd() }; // set the cursor on the right spot
    unsafe { ui_cursor_shape() };
    1
}

pub(crate) unsafe fn abandon_cmdline() {
    dealloc_cmdbuff();
    if msg_scrolled.get() == 0 {
        unsafe { compute_cmdrow() };
    }
    // Avoid overwriting a key prompt.
    if !Cc::current().one_key {
        unsafe { msg(c"".as_ptr(), 0) };
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
pub unsafe fn getcmdline(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    indent: ::core::ffi::c_int,
    _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe { command_line_enter(firstc, count, indent, true) as *mut ::core::ffi::c_char }
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
/// prompts) and `mouse_used` is set when returning after a right click.
/// Answers an allocated command line, or NULL.
#[allow(clippy::too_many_arguments)]
pub unsafe fn getcmdline_prompt(
    firstc: ::core::ffi::c_int,
    prompt: *const ::core::ffi::c_char,
    hl_id: ::core::ffi::c_int,
    xp_context: ExpandContext,
    xp_arg: *const ::core::ffi::c_char,
    highlight_callback: Callback,
    one_key: bool,
    mouse_used: *mut bool,
) -> *mut ::core::ffi::c_char {
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
    cc.cmdprompt = prompt as *mut ::core::ffi::c_char;
    cc.hl_id = hl_id;
    cc.xp_context = xp_context;
    cc.xp_arg = xp_arg as *mut ::core::ffi::c_char;
    cc.input_fn = (firstc == '@' as ::core::ffi::c_int) as ::core::ffi::c_int;
    cc.highlight_callback = highlight_callback;
    cc.one_key = one_key;
    cc.mouse_used = mouse_used;

    let cmd_silent_saved = cmd_silent.get();
    let loud = Allow::messages();
    cmd_silent.set(false); // want to see the prompt

    let ret = unsafe { command_line_enter(firstc, 1, 0, false) } as *mut ::core::ffi::c_char;
    cc.redraw_state = kCmdRedrawNone;
    if did_save_ccline {
        restore_cmdline();
    }
    drop(loud);
    cmd_silent.set(cmd_silent_saved);
    if cc.in_use() {
        msg_col.set(msg_col_save);
    }
    ret
}

/// `&curbuf->b_p_iminsert`: a field's address, which reads nothing.
pub(crate) fn cur_buf_iminsert() -> *mut OptInt {
    // SAFETY: a field's address is the object's plus a constant.
    unsafe { &raw mut (*curbuf.get()).b_p_iminsert }
}

/// `&curbuf->b_p_imsearch`; see [`cur_buf_iminsert`].
pub(crate) fn cur_buf_imsearch() -> *mut OptInt {
    // SAFETY: a field's address is the object's plus a constant.
    unsafe { &raw mut (*curbuf.get()).b_p_imsearch }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
