//! `'cmdline highlighting'`: the callback that colours the line.
//!
//! [`color_cmdline`] calls whatever the `input()` caller or `:` gave it,
//! validates the returned list of `[start, end, group]` chunks and caches the
//! answer against the line it was computed for.  [`color_expr_cmdline`] is
//! the built-in colouring of `=` expressions.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::*;
use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{CallbackRefused, NumBuf, index_of, list_items, list_iter};
use crate::guard::Suppress;
use crate::message::msg_put_ascii;
use crate::message_fmt::msg_cstr;
use crate::narrow::{len_as_int, number_as_int};
use crate::tr_plural;
use crate::types::VAR_LIST;
use crate::viml::parser::expressions::EXPR_FLAGS_DISALLOW_EOC;
use core::ffi::CStr;

/// Colour a `=` expression command line with the Vimscript expression parser,
/// filling the gaps the parser leaves uncoloured with `hl_id` 0, into the
/// line's own chunk list.
pub(crate) fn color_expr_cmdline(mut colored_ccline: Cc) {
    let colors = {
        let text = cstr::until_nul(colored_ccline.bytes());
        let line = [text];
        let mut pstate = ParserState::new(&line, true);
        let _ = viml_pexpr_parse(&mut pstate, EXPR_FLAGS_DISALLOW_EOC.cast_signed());
        pstate.take_highlight()
    };

    // C's `kv_resize`: reserve exactly what the parser produced. The
    // gap-filling chunks below may still push past it.
    let chunks = &mut colored_ccline.last_colors;
    chunks.reserve_chunks(colors.len());

    let mut prev_end: size_t = 0;
    for chunk in &colors {
        if chunk.start.col != prev_end {
            chunks.push(CmdlineColorChunk {
                start: len_as_int(prev_end),
                end: len_as_int(chunk.start.col),
                hl_id: 0,
            });
        }
        chunks.push(CmdlineColorChunk {
            start: len_as_int(chunk.start.col),
            end: len_as_int(chunk.end_col),
            hl_id: syn_name2id(chunk.group),
        });
        prev_end = chunk.end_col;
    }
    let len = colored_ccline.len();
    if prev_end < usize::try_from(len).unwrap_or(0) {
        colored_ccline.last_colors.push(CmdlineColorChunk {
            start: len_as_int(prev_end),
            end: len,
            hl_id: 0,
        });
    }
}

/// Why the colouring stopped short of a full set of chunks.
///
/// Both of upstream's labels: `color_cmdline_end` when there was nothing
/// (more) to colour, which keeps whatever chunks were pushed, and
/// `color_cmdline_error` when the callback failed or answered something
/// unusable.
#[derive(Debug)]
enum Stop {
    /// Keep whatever chunks were pushed.
    Done,
    /// Every path that stops here has already printed why, or left it in
    /// [`Colouring::err`]. The chunks are thrown away and the line redrawn.
    Error,
}

// Upstream's function-level statics. `PREV_PROMPT_ID` starts at UINT_MAX so
// that the first prompt of a session, whatever its id, counts as a new one.
static PREV_PROMPT_ID: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(UINT_MAX);
static PREV_PROMPT_ERRORS: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

/// C's `PRINT_ERRMSG`: an error that scrolls the command line away, noted in
/// `$printed`. A macro rather than a method so the message stays a literal.
macro_rules! print_errmsg {
    ($printed:expr, $($arg:tt)*) => {{
        MSG_SCROLL.set(1);
        msg_put_ascii(b'\n');
        let _: bool = $crate::smsg!(HLF_E, $($arg)*);
        $printed = true;
    }};
}

/// One colouring of the command line: the callback, what it was given and
/// answered, and the error it raised.
struct Colouring {
    color_cb: Callback,
    /// Whether `color_cb` is this frame's own reference, to release.
    can_free_cb: bool,
    /// A copy of the command line, the callback's argument.
    arg: TypVal,
    /// C's record that the line was not terminated at its length, so its
    /// copy (rather than the line) is what gets cached.
    arg_allocated: bool,
    /// The callback's answer.
    tv: TypVal,
    /// What the callback or its lookup threw.
    err: Error,
    /// The message `err` is reported under.
    err_errmsg: &'static CStr,
    printed_errmsg: bool,
}

/// Colour the command line, through the user's callback where there is one.
///
/// `colored_ccline` is also the cache: when its `prompt_id` and `cmdbuff`
/// still match what `last_colors` was computed from, this does nothing.  The
/// whole line is always coloured.
///
/// Answers true if [`super::draw::draw_cmdline`] may proceed, false if there
/// is nothing for it to do.
pub(crate) fn color_cmdline(mut colored_ccline: Cc) -> bool {
    // Is the result of the previous call still valid?
    if colored_ccline
        .last_colors
        .is_current(colored_ccline.prompt_id, colored_ccline.cmdbuff.bytes())
    {
        return true;
    }

    colored_ccline.last_colors.clear_chunks();

    if !colored_ccline.in_use() || colored_ccline.byte_at(0) == 0 {
        // Nothing to do.
        colored_ccline.last_colors.forget();
        return true;
    }

    let mut run = Colouring {
        color_cb: Callback::None,
        can_free_cb: false,
        arg: TypVal::string(None),
        arg_allocated: false,
        tv: TV_INITIAL_VALUE,
        err: Error::none(),
        err_errmsg: E_INTERN2,
        printed_errmsg: false,
    };
    let stopped = run
        .take_callback(colored_ccline)
        .and_then(|()| run.call_callback(colored_ccline))
        .and_then(|()| run.take_chunks(colored_ccline));
    let ret = !matches!(stopped, Err(Stop::Error));
    if !ret {
        run.fail(colored_ccline);
    }
    run.finish(colored_ccline);
    ret
}

impl Colouring {
    /// Find the callback: the `input()` caller's, `g:Nvim_color_cmdline`
    /// for `:`, or none (a `=` line is coloured here and now). A prompt
    /// whose callback has failed too often gets none.
    fn take_callback(&mut self, colored_ccline: Cc) -> Result<(), Stop> {
        if colored_ccline.prompt_id != PREV_PROMPT_ID.get() {
            PREV_PROMPT_ERRORS.set(0);
            PREV_PROMPT_ID.set(colored_ccline.prompt_id);
        } else if PREV_PROMPT_ERRORS.get() >= MAX_CB_ERRORS {
            return Err(Stop::Done);
        }

        let mut dgc_ret = true;
        if colored_ccline.highlight_callback.is_set() {
            // Currently this should only happen while processing input()
            // prompts.
            debug_assert!(colored_ccline.input_fn);
            // A reference of this frame's own, rather than upstream's bit
            // copy: the callback is user code, and the command line it
            // came from is a global it can reach.
            self.color_cb = colored_ccline.highlight_callback.duplicate();
            self.can_free_cb = true;
        } else if colored_ccline.cmdfirstc == b':' {
            // C's TRY_WRAP.
            let mut tstate: TryState = TryState::INIT;
            try_enter(&mut tstate);
            self.err_errmsg = c"E5408: Unable to get g:Nvim_color_cmdline callback: %s";
            dgc_ret = dict_get_callback(
                Some(&globvar_dict()),
                b"Nvim_color_cmdline",
                &mut self.color_cb,
            )
            .map_err(CallbackRefused::report)
            .is_ok();
            self.err.absorb(try_leave(&tstate));
            self.can_free_cb = true;
        } else if colored_ccline.cmdfirstc == b'=' {
            color_expr_cmdline(colored_ccline);
        }
        if self.err.is_set() || !dgc_ret {
            return Err(Stop::Error);
        }
        if !self.color_cb.is_set() {
            return Err(Stop::Done);
        }
        Ok(())
    }

    /// Call the callback with a copy of the line, into [`Self::tv`].
    fn call_callback(&mut self, colored_ccline: Cc) -> Result<(), Stop> {
        if colored_ccline
            .cmdbuff
            .raw_byte_at(usize::try_from(colored_ccline.len()).unwrap_or(0))
            != 0
        {
            self.arg_allocated = true;
        }
        self.arg = TypVal::string_from(colored_ccline.text_bytes());
        // msg_start(), called by e.g. :echo, may shift the command line to
        // the first column even under msg_silent. Two ways round it
        // without altering message.c: use full_screen, or save and restore
        // msg_col. Saving full_screen does not work well with `:redraw!`;
        // msg_col is not ideal either, but it merely misses a leading `:`,
        // where full_screen leaves the line shifted one column right with
        // the cursor in the wrong place.
        //
        // TRY_WRAP too, because error messages would otherwise overwrite
        // the typed command line.
        GETLN_INTERRUPTED_HIGHLIGHT.set(false);
        let mut tstate: TryState = TryState::INIT;
        try_enter(&mut tstate);
        self.err_errmsg = c"E5407: Callback has thrown an exception: %s";
        let saved_msg_col = MSG_COL.get();
        let silenced = Suppress::messages();
        let called = callback_call(
            &self.color_cb,
            ::core::slice::from_ref(&self.arg),
            &mut self.tv,
        ) == Ok(true);
        drop(silenced);
        MSG_COL.set(saved_msg_col);
        if GOT_INT.get() {
            GETLN_INTERRUPTED_HIGHLIGHT.set(true);
        }
        self.err.absorb(try_leave(&tstate));

        if self.err.is_set() || !called {
            return Err(Stop::Error);
        }
        if self.tv.v_type() != VAR_LIST {
            print_errmsg!(self.printed_errmsg, "E5400: Callback should return list");
            return Err(Stop::Error);
        }
        Ok(())
    }

    /// Check the callback's `[start, end, group]` chunks and push them, with
    /// uncoloured chunks over the gaps between them and after the last.
    fn take_chunks(&mut self, mut colored_ccline: Cc) -> Result<(), Stop> {
        let Some(chunks) = self.tv.list_ref() else {
            return Err(Stop::Done);
        };
        let mut numbuf = NumBuf::new();
        let len = VarNumber::from(colored_ccline.len());
        let mut prev_end: VarNumber = 0;
        for (i, li) in list_iter(Some(chunks)).enumerate() {
            let i = index_of(i);
            if li.li_tv.v_type() != VAR_LIST {
                print_errmsg!(self.printed_errmsg, "E5401: List item {i} is not a List");
                return Err(Stop::Error);
            }
            if list_len(li.li_tv.list_ref()) != 3 {
                let len = list_len(li.li_tv.list_ref());
                print_errmsg!(
                    self.printed_errmsg,
                    "E5402: List item {i} has incorrect length: {len} /= 3"
                );
                return Err(Stop::Error);
            }

            // The item's own list, just checked to hold three items.
            let chunk = list_items(li.li_tv.list_ref());
            let start = tv_get_number_chk(&chunk[0].li_tv).map_err(|_| Stop::Error)?;
            if !(prev_end <= start && start < len) {
                print_errmsg!(
                    self.printed_errmsg,
                    "E5403: Chunk {i} start {start} not in range [{prev_end}, {len})"
                );
                return Err(Stop::Error);
            } else if splits_character(colored_ccline, start) {
                print_errmsg!(
                    self.printed_errmsg,
                    "E5405: Chunk {i} start {start} splits multibyte character"
                );
                return Err(Stop::Error);
            }

            if start != prev_end {
                colored_ccline.last_colors.push(CmdlineColorChunk {
                    start: number_as_int(prev_end),
                    end: number_as_int(start),
                    hl_id: 0,
                });
            }

            let end = tv_get_number_chk(&chunk[1].li_tv).map_err(|_| Stop::Error)?;
            if !(start < end && end <= len) {
                print_errmsg!(
                    self.printed_errmsg,
                    "E5404: Chunk {i} end {end} not in range ({start}, {len}]"
                );
                return Err(Stop::Error);
            } else if end < len && splits_character(colored_ccline, end) {
                print_errmsg!(
                    self.printed_errmsg,
                    "E5406: Chunk {i} end {end} splits multibyte character"
                );
                return Err(Stop::Error);
            }

            prev_end = end;
            let group = numbuf
                .string_chk(&chunk[2].li_tv)
                .map_err(|_| Stop::Error)?;
            colored_ccline.last_colors.push(CmdlineColorChunk {
                start: number_as_int(start),
                end: number_as_int(end),
                hl_id: syn_name2id(group),
            });
        }

        if prev_end < len {
            let tail = CmdlineColorChunk {
                start: number_as_int(prev_end),
                end: colored_ccline.len(),
                hl_id: 0,
            };
            colored_ccline.last_colors.push(tail);
        }
        PREV_PROMPT_ERRORS.set(0);
        Ok(())
    }

    /// `color_cmdline_error`: report what was thrown, count the failure
    /// against the prompt, drop the chunks and redraw.
    fn fail(&mut self, mut colored_ccline: Cc) {
        if self.err.is_set() {
            let why = msg_cstr(self.err.message_or_empty());
            let template = gettext(self.err_errmsg);
            print_errmsg!(self.printed_errmsg, "{}", tr_plural!(template, why));
            self.err.clear();
        }
        debug_assert!(self.printed_errmsg);
        PREV_PROMPT_ERRORS.set(PREV_PROMPT_ERRORS.get() + 1);
        colored_ccline.last_colors.clear_chunks();
        redrawcmdline();
    }

    /// `color_cmdline_end`: release the callback and cache the answer.
    fn finish(mut self, mut colored_ccline: Cc) {
        debug_assert!(!self.err.is_set());
        if self.can_free_cb {
            callback_free(&mut self.color_cb);
        }
        // Errors' "output" is cached just as well as regular results.
        //
        // Which text is cached is C's choice kept: the copy `arg` was given
        // when one was made -- the line as the callback saw it, which the
        // callback may since have edited -- and the line itself otherwise.
        // C took ownership of `arg`'s copy here; the cache owns its bytes
        // now, so the copy is released with `arg` instead.
        let id = colored_ccline.prompt_id;
        if self.arg_allocated {
            colored_ccline
                .last_colors
                .remember(id, self.arg.string_bytes());
        } else {
            // Two fields of the one line: the cache takes a copy of the text.
            let line = &mut *colored_ccline;
            line.last_colors.remember(id, line.cmdbuff.bytes());
        }
        tv_clear(&mut self.tv);
    }
}

/// Whether byte `at` of the line is inside a character rather than at its
/// start.
fn splits_character(colored_ccline: Cc, at: VarNumber) -> bool {
    UTF8LEN_TAB_ZERO[usize::from(colored_ccline.byte_at(number_as_int(at)))] == 0
}
