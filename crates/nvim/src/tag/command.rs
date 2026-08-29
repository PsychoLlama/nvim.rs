//! The `:tag` family of commands.
//!
//! [`do_tag`] is all of `:tag`, `:tselect`, `:tjump`, `:tnext`, `:tprev`,
//! `:tfirst`, `:tlast`, `:ltag`, `:pop` and `CTRL-T`. It decides which
//! match of which tag the user asked for, records the jump on the window's
//! tag stack, and hands the match to [`jumpto_tag`].
//!
//! The matches of the last tag looked up are remembered between calls, so
//! that `:tnext` does not have to read the tags files again.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::file_search::Name;
use crate::highlight_group::HLF_W;
use crate::message::msg_ptr;
use crate::message_fmt::c_str;
use crate::pos::MAXCOL;
use crate::semsg;
use crate::smsg_c;
use crate::types::{FAIL, IOSIZE, OK, Vv};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// The preview window's stand-in for a tag stack.
///
/// A window carries a whole `taggy_T` stack; `:ptag` has this one entry
/// instead, and the handle *names* it rather than borrowing it, so it
/// survives the `'tagfunc'` call and the autocommands a jump runs.
#[derive(Clone, Copy)]
pub(super) struct PtagEntry(*mut taggy_T);

/// The one place the preview entry's address is taken.
pub(super) fn ptag_entry_handle() -> PtagEntry {
    PtagEntry(ptag_entry.ptr())
}

impl PtagEntry {
    /// The match the preview window is showing, and the buffer it came from.
    pub(super) fn position(self) -> (c_int, c_int) {
        // SAFETY: the only constructor names a `static`.
        unsafe { ((*self.0).cur_match, (*self.0).cur_fnum) }
    }

    /// Remember where the preview window got to.
    fn set_position(self, cur_match: c_int, cur_fnum: c_int) {
        // SAFETY: as `position`.
        unsafe { (*self.0).cur_match = cur_match };
        // SAFETY: as `position`.
        unsafe { (*self.0).cur_fnum = cur_fnum };
    }

    /// The tag being previewed, or NULL when there is none.
    fn tagname(self) -> *mut c_char {
        // SAFETY: as `position`.
        unsafe { (*self.0).tagname }
    }

    /// Forget the previous preview tag and take a copy of `tag`.
    ///
    /// # Safety
    /// `tag` must be NUL-terminated.
    unsafe fn restart(self, tag: *const c_char) {
        // SAFETY: as `position`; the entry owns its name.
        unsafe { tagstack_clear_entry(&mut *self.0) };
        // SAFETY: the caller's NUL-terminated string.
        unsafe { (*self.0).tagname = xstrdup(tag) };
    }
}

/// The matches of the tag last looked up. Owned; `free_wild`d when replaced.
static matches: GlobalCell<*mut *mut c_char> = GlobalCell::new(ptr::null_mut());

/// How many of them there are.
static num_matches: GlobalCell<c_int> = GlobalCell::new(0);

/// The limit the last search used. `MAXCOL` means every match was found,
/// so there is no point looking again for a later one.
static max_num_matches: GlobalCell<c_int> = GlobalCell::new(0);

/// `:tlast`. The parent's `DT_*` family is the transpile's, which left
/// this one out; it is used here and nowhere else in the module.
const DT_LAST: c_uint = 6;

/// The view the tag stack starts a jump from: no remembered scroll
/// position.
const NO_VIEW: fmarkv_T = fmarkv_T {
    topline_offset: MAXLNUM as linenr_T,
    skipcol: 0,
};

/// Jump to a tag, or move about the tag stack.
///
/// `tag` is the tag or pattern, empty for the forms that take none.
/// `kind` is one of the `DT_*` constants, `count` the command's count,
/// `forceit` its `!`, and `verbose` asks for "tag not found" to be
/// reported.
///
/// # Safety
/// `tag` must be NUL-terminated, and the editor must be in a state where a
/// buffer may be entered.
pub unsafe fn do_tag(tag: *mut c_char, kind: c_int, count: c_int, forceit: c_int, verbose: bool) {
    // SAFETY: the caller's promise; the globals are live.
    if tfu_in_use.get() {
        // The stack is what `'tagfunc'` is being asked to describe.
        tag_emsg(c"E986: Cannot modify the tag stack within tagfunc");
        return;
    }
    if postponed_split.get() == 0 && !check_can_set_curbuf_forceit(forceit) {
        return;
    }

    let mut cmd = unsafe { DoTag::new(tag, kind, count, forceit, verbose) };
    if cmd.prepare() {
        cmd.set_priority_buffer();
        cmd.run();
    }
    cmd.finish();
}

/// One `:tag`-family command in progress.
struct DoTag {
    /// The tag or pattern given, empty for the argument-less forms.
    tag: *mut c_char,
    /// Which `DT_*` command this is. `DT_HELP` becomes `DT_TAG`, and a
    /// retry after a missing file becomes `DT_NEXT`.
    kind: c_int,
    count: c_int,
    forceit: c_int,
    verbose: bool,

    /// The window's tag stack as it was when the command started.
    ///
    /// Deliberately a pointer taken once: `'tagfunc'` may close the window
    /// out from under us, and comparing this against `curwin`'s stack
    /// afterwards is how that is noticed.
    tagstack: *mut taggy_T,
    /// Our own copy of the stack index and length, written back at the end.
    idx: c_int,
    len: c_int,
    /// Where the index was when the command started, to go back to when a
    /// `:pop` fails.
    old_idx: c_int,
    /// Where it was before a selection that the user may cancel.
    prev_idx: c_int,

    /// Which match to jump to, and the buffer the matches were ordered for.
    cur_match: c_int,
    cur_fnum: c_int,
    /// The match whose file turned out not to exist, so that the message
    /// is given once.
    error_cur_match: c_int,

    /// These matches are for a tag not looked up before.
    new_tag: bool,
    /// A `:help` tag is never read as a regexp.
    no_regexp: bool,
    /// `'tagfunc'` may answer for this command.
    use_tfu: bool,
    /// The jump is recorded on the window's tag stack.
    use_tagstack: bool,
    /// The cursor position is worth remembering on the stack entry.
    save_pos: bool,
    /// An error was already reported; don't add "tag N of M" to it.
    skip_msg: bool,

    /// Where the cursor was, put back when a `:tselect` is cancelled.
    saved_fmark: fmark_T,
    /// The buffer name matches are prioritised against.
    buf_ffname: *mut c_char,
    /// A copy of the stack entry's tag name: `'tagfunc'` may free the
    /// entry it came from.
    owned_name: Option<Name>,
    /// How many matches the previous command found.
    prev_num_matches: c_int,
}

// The methods below are safe because [`DoTag::new`] is where the promise is
// paid: it records the window's tag stack and the editor's globals as they
// were when the command started, and `run` re-checks the window before
// touching the stack again. What each of these used to ask of its caller —
// "the globals must be live" — is now this type's own invariant.
impl DoTag {
    /// # Safety
    /// `curwin` and `curbuf` must be live, and `tag` NUL-terminated.
    unsafe fn new(
        tag: *mut c_char,
        kind: c_int,
        count: c_int,
        forceit: c_int,
        verbose: bool,
    ) -> Self {
        // SAFETY: the caller's promise.
        let win = curwin.get();
        // `:help` tags are literal, and 'tagfunc' has no business
        // answering for them.
        let help = kind == DT_HELP as c_int;
        let idx = unsafe { (*win).w_tagstackidx };

        let cmd = DoTag {
            tag,
            kind: if help { DT_TAG as c_int } else { kind },
            count,
            forceit,
            verbose,
            tagstack: (unsafe { &raw mut (*win).w_tagstack }).cast(),
            idx,
            len: unsafe { (*win).w_tagstacklen },
            old_idx: idx,
            prev_idx: idx,
            cur_match: 0,
            cur_fnum: cur_buf().handle,
            error_cur_match: 0,
            new_tag: false,
            no_regexp: help,
            use_tfu: !help,
            use_tagstack: false,
            save_pos: false,
            skip_msg: false,
            saved_fmark: fmark_T {
                mark: pos_T::default(),
                fnum: 0,
                timestamp: 0,
                view: NO_VIEW,
                additional_data: ptr::null_mut(),
            },
            buf_ffname: cur_buf().b_ffname,
            owned_name: None,
            prev_num_matches: num_matches.get(),
        };
        unsafe { free_string_option(nofile_fname.get()) };
        nofile_fname.set(ptr::null_mut());
        cmd
    }

    /// One entry of the tag stack.
    ///
    /// # Safety
    /// `at` must be within the stack, and the window must not have been
    /// closed since the command started.
    unsafe fn entry(&self, at: c_int) -> *mut taggy_T {
        // SAFETY: the caller's promise.
        unsafe { self.tagstack.offset(at as isize) }
    }

    /// The stack entry the command stands on.
    ///
    /// A fresh [`Tagg`] per access rather than a borrow held across the
    /// body: `'tagfunc'` can run between two of these.
    fn current(&self) -> Tagg {
        // SAFETY: `idx` is inside the stack the command started on, and
        // `run` checks the window was not closed before getting here.
        unsafe { Tagg::new(self.entry(self.idx)) }
    }

    /// Work out which tag and which of its matches is wanted, and record
    /// the jump on the stack.
    ///
    /// Answers `false` when the command is already finished — an empty
    /// stack, a `:pop` that walked off the end, or a `CTRL-T` that has
    /// already jumped.
    fn prepare(&mut self) -> bool {
        // SAFETY: the caller's promise.
        if p_tgst.get() == 0 && unsafe { *self.tag } != 0 {
            // 'tagstack' is off: jump without recording anything.
            self.use_tagstack = false;
            self.new_tag = true;
            if g_do_tagpreview.get() != 0 {
                self.start_preview_tag();
            }
            return true;
        }

        // A preview tag has its own one-entry stack.
        self.use_tagstack = g_do_tagpreview.get() == 0;

        let named = matches!(self.kind as c_uint, DT_TAG | DT_SELECT | DT_JUMP | DT_LTAG);
        if unsafe { *self.tag } != 0 && named {
            self.push_new_tag();
            self.new_tag = true;
        } else if !self.walk_stack() {
            return false;
        }

        if g_do_tagpreview.get() != 0 {
            if !self.selecting() {
                ptag_entry_handle().set_position(self.cur_match, self.cur_fnum);
            }
        } else {
            self.record_position();
        }
        true
    }

    /// Whether the command shows a list and asks the user to pick.
    fn selecting(&self) -> bool {
        matches!(self.kind as c_uint, DT_SELECT | DT_JUMP)
    }

    /// Start a fresh preview tag, forgetting the previous one.
    fn start_preview_tag(&mut self) {
        // SAFETY: the caller's promise.
        unsafe { ptag_entry_handle().restart(self.tag) };
    }

    /// A new tag was named: put it on the stack (or on the preview entry).
    fn push_new_tag(&mut self) {
        // SAFETY: the caller's promise; every entry owns its own name.
        if g_do_tagpreview.get() != 0 {
            let previous = ptag_entry_handle().tagname();
            if !previous.is_null() && unsafe { strcmp(previous, self.tag) } == 0 {
                // Jumping to the same tag again: keep the current
                // match, so that the CursorHold example works.
                (self.cur_match, self.cur_fnum) = ptag_entry_handle().position();
            } else {
                self.start_preview_tag();
            }
            return;
        }

        // Anything above the entry last used is now unreachable.
        while self.idx < self.len {
            self.len -= 1;
            unsafe { tagstack_clear_entry(&mut *self.entry(self.len)) };
        }

        self.len += 1;
        if self.len > TAGSTACKSIZE {
            // Full: drop the oldest entry and shift the rest down.
            self.len = TAGSTACKSIZE;
            unsafe { tagstack_clear_entry(&mut *self.entry(0)) };
            for i in 1..self.len {
                unsafe { *self.entry(i - 1) = (*self.entry(i)).clone() };
            }
            self.idx -= 1;
            // The name moved down with the entry; the user data is
            // about to be replaced, and must not be freed twice.
            self.current().user_data = ptr::null_mut();
        }

        // SAFETY: `tag` is the NUL-terminated name the command was given.
        self.current().tagname = unsafe { xstrdup(self.tag) };
        cur_win().w_tagstacklen = self.len;
        // Worth remembering where the cursor was.
        self.save_pos = true;
    }

    /// No tag was named: move about the stack instead.
    ///
    /// Answers `false` when the command is finished — the stack was empty,
    /// or a `:pop` has already done the jump.
    fn walk_stack(&mut self) -> bool {
        // SAFETY: the caller's promise.
        let empty = if g_do_tagpreview.get() != 0 {
            ptag_entry_handle().tagname().is_null()
        } else {
            self.len == 0
        };
        if empty {
            tag_emsg(c"E73: Tag stack empty");
            return false;
        }

        match self.kind as c_uint {
            DT_POP => return self.pop_older(),
            DT_TAG | DT_LTAG => {
                if !self.go_newer() {
                    return false;
                }
                self.new_tag = true;
            }
            _ => self.go_other_match(),
        }
        true
    }

    /// `:pop` and `CTRL-T`: go back to where a jump started.
    ///
    /// This does the whole jump itself, so it always answers `false`.
    fn pop_older(&mut self) -> bool {
        // SAFETY: the caller's promise; the index is checked against the
        // stack before any entry is read.
        // Opening the file resets it.
        let old_key_typed = KeyTyped.get();

        self.idx -= self.count;
        if self.idx < 0 {
            tag_emsg(c"E555: At bottom of tag stack");
            // Off the bottom: go all the way there, unless we were
            // already at the bottom, in which case nothing happens.
            let was_at_bottom = self.idx + self.count == 0;
            self.idx = 0;
            if was_at_bottom {
                return false;
            }
        } else if self.idx >= self.len {
            // count == 0.
            tag_emsg(c"E556: At top of tag stack");
            return false;
        }

        // A copy: autocommands may invalidate the stack before it is
        // used.
        self.saved_fmark = self.current().fmark.clone();
        let mark = self.saved_fmark.clone();
        if mark.fnum != cur_buf().handle {
            // Another file. If it cannot be opened (it may have
            // changed) keep the original position on the stack.
            if unsafe {
                buflist_getfile(
                    mark.fnum,
                    mark.mark.lnum,
                    GETF_SETMARK as c_int,
                    self.forceit,
                )
            } == FAIL
            {
                self.idx = self.old_idx;
                return false;
            }
            // A BufReadPost autocommand may jump to the '" mark, which
            // is not wanted here.
            cur_win().w_cursor.lnum = mark.mark.lnum;
        } else {
            setpcmark();
            cur_win().w_cursor.lnum = mark.mark.lnum;
        }
        cur_win().w_cursor.col = mark.mark.col;
        cur_win().w_set_curswant = true;
        if jop_flags.get() & kOptJopFlagView as c_uint != 0 {
            unsafe { mark_view_restore(&raw mut self.saved_fmark) };
        }
        check_cursor(unsafe { Win::current() });
        if fdo_flags.get() & kOptFdoFlagTag as c_uint != 0 && old_key_typed {
            unsafe { fold_open_cursor() };
        }

        // The remembered matches are for a tag we have left.
        unsafe { forget_matches() };
        unsafe { tag_freematch() };
        false
    }

    /// `:tag` with no argument: go to the newer entry on the stack.
    ///
    /// Answers `false` when the command is finished.
    fn go_newer(&mut self) -> bool {
        // SAFETY: the caller's promise.
        if g_do_tagpreview.get() != 0 {
            (self.cur_match, self.cur_fnum) = ptag_entry_handle().position();
            return true;
        }

        self.save_pos = true;
        self.idx += self.count - 1;
        if self.idx >= self.len {
            // Beyond the last: say so, go to the last one, and don't
            // store the cursor position there.
            self.idx = self.len - 1;
            tag_emsg(c"E556: At top of tag stack");
            self.save_pos = false;
        } else if self.idx < 0 {
            // count == 0.
            tag_emsg(c"E555: At bottom of tag stack");
            self.idx = 0;
            return false;
        }
        self.cur_match = self.current().cur_match;
        self.cur_fnum = self.current().cur_fnum;
        true
    }

    /// `:tnext` and friends: another match of the tag already on the stack.
    fn go_other_match(&mut self) {
        // SAFETY: the caller's promise.
        // Where to go back to if the selection is cancelled.
        self.prev_idx = self.idx;

        if g_do_tagpreview.get() != 0 {
            (self.cur_match, self.cur_fnum) = ptag_entry_handle().position();
        } else {
            self.idx = (self.idx - 1).max(0);
            self.cur_match = self.current().cur_match;
            self.cur_fnum = self.current().cur_fnum;
        }

        match self.kind as c_uint {
            DT_FIRST => self.cur_match = self.count - 1,
            DT_SELECT | DT_JUMP | DT_LAST => self.cur_match = MAXCOL as c_int - 1,
            DT_NEXT => self.cur_match += self.count,
            DT_PREV => self.cur_match -= self.count,
            _ => {}
        }
        if self.cur_match >= MAXCOL as c_int {
            self.cur_match = MAXCOL as c_int - 1;
        } else if self.cur_match < 0 {
            tag_emsg(c"E425: Cannot go before first matching tag");
            self.skip_msg = true;
            self.cur_match = 0;
            self.cur_fnum = cur_buf().handle;
        }
    }

    /// Remember on the stack where the jump is starting from.
    fn record_position(&mut self) {
        // SAFETY: the caller's promise.
        self.saved_fmark = self.current().fmark.clone();
        if self.save_pos {
            let cursor = cur_win().w_cursor;
            self.current().fmark.mark = cursor;
            self.current().fmark.fnum = cur_buf().handle;
            // SAFETY: `curwin` is live and `cursor` is a position in it.
            self.current().fmark.view = unsafe { mark_view_make(curwin.get(), cursor) };
        }

        // `curwin` changes in `jumpto_tag` for `:stag`, or when an
        // autocommand jumps to another window, so store the index now.
        cur_win().w_tagstackidx = self.idx;
        if !self.selecting() {
            let entry = unsafe {
                (&raw mut (*curwin.get()).w_tagstack)
                    .cast::<taggy_T>()
                    .offset(self.idx as isize)
            };
            unsafe { (*entry).cur_match = self.cur_match };
            unsafe { (*entry).cur_fnum = self.cur_fnum };
        }
    }

    /// Prioritise matches against the buffer `cur_fnum` names.
    ///
    /// Using a remembered `cur_match` only makes sense if the order the
    /// matches came out in is the same as it was then.
    fn set_priority_buffer(&mut self) {
        // SAFETY: the caller's promise.
        if self.cur_fnum == cur_buf().handle {
            return;
        }
        if let Some(buf) = find_buf(self.cur_fnum) {
            self.buf_ffname = buf.b_ffname;
        }
    }

    /// Find the matches, choose one, and jump to it — retrying with the
    /// next match when the file a match names does not exist.
    fn run(&mut self) {
        // SAFETY: the caller's promise; `name` points either into a copy
        // this loop owns or into a global that outlives it.
        loop {
            let mut name = self.tag_name();
            // Whether the remembered matches are for a different tag.
            // Read before `search` replaces the remembered name, and
            // before a leading '/' is stepped over.
            let other = unsafe { self.other_name(name) };
            if (self.new_tag
                || (self.cur_match >= num_matches.get()
                    && max_num_matches.get() != MAXCOL as c_int)
                || other)
                && !unsafe { self.search(&mut name, other) }
            {
                return;
            }

            if num_matches.get() <= 0 {
                if self.verbose {
                    // SAFETY: the message macros expand to a `vim_snprintf` over // the format literal above and the editor's message buffers.
                    let name = unsafe { c_str(name) };
                    semsg!("E426: Tag not found: {name}");
                }
                g_do_tagpreview.set(0);
                return;
            }
            if !self.choose() || !unsafe { self.jump(name) } {
                return;
            }
        }
    }

    /// The name to look up: the stack entry's, the preview entry's, or the
    /// command's own argument.
    fn tag_name(&mut self) -> *mut c_char {
        // SAFETY: the caller's promise.
        if self.use_tagstack {
            // A copy: `'tagfunc'` may rewrite the stack under us.
            let owned = unsafe { Name::from_ptr(self.current().tagname) };
            self.owned_name = Some(owned);
            self.owned_name.as_mut().unwrap().as_mut_ptr()
        } else if g_do_tagpreview.get() != 0 {
            ptag_entry_handle().tagname()
        } else {
            self.tag
        }
    }

    /// Read the tags files (or ask `'tagfunc'`) for `name`.
    ///
    /// Answers `false` when the window went away while searching.
    ///
    /// # Safety
    /// `name` must be NUL-terminated; it is advanced past a leading `/`.
    unsafe fn search(&mut self, name: &mut *mut c_char, other: bool) -> bool {
        // SAFETY: the caller's promise; `find_tags` fills both locals and
        // the matches it answers become ours.
        if other {
            unsafe { xfree(tagmatchname.get().cast()) };
            tagmatchname.set(unsafe { xstrdup(*name) });
        }

        if matches!(self.kind as c_uint, DT_SELECT | DT_JUMP | DT_LTAG) {
            // Every match is wanted, to list them.
            self.cur_match = MAXCOL as c_int - 1;
        }
        max_num_matches.set(if self.kind == DT_TAG as c_int {
            MAXCOL as c_int
        } else {
            self.cur_match + 1
        });

        // An argument starting with '/' is a regexp; anything else is
        // a literal name, and case is then not folded.
        let mut flags = if !self.no_regexp && unsafe { **name } == b'/' as c_char {
            *name = unsafe { name.add(1) };
            TAG_REGEXP as c_int
        } else {
            TAG_NOIC as c_int
        };
        if self.verbose {
            flags |= TAG_VERBOSE as c_int;
        }
        if !self.use_tfu {
            flags |= TAG_NO_TAGFUNC as c_int;
        }

        let mut new_num_matches = 0;
        let mut new_matches = ptr::null_mut::<*mut c_char>();
        if unsafe {
            find_tags(
                *name,
                &raw mut new_num_matches,
                &raw mut new_matches,
                flags,
                max_num_matches.get(),
                self.buf_ffname,
            )
        } == OK
            && new_num_matches < max_num_matches.get()
        {
            // Fewer than the limit: that is all of them.
            max_num_matches.set(MAXCOL as c_int);
        }

        // A tag function may do anything, which may make all sorts of
        // things invalid. At least check that the tag stack is still
        // the one we started with.
        if self.tagstack != (unsafe { &raw mut (*curwin.get()).w_tagstack }).cast() {
            tag_emsg(c"E1299: Window unexpectedly closed while searching for tags");
            unsafe { free_wild(new_num_matches, new_matches) };
            return false;
        }

        if !self.new_tag && !other {
            unsafe { reorder_matches(new_matches, new_num_matches) };
        }
        unsafe { free_wild(num_matches.get(), matches.get()) };
        num_matches.set(new_num_matches);
        matches.set(new_matches);
        true
    }

    /// Whether `name` is not the tag the remembered matches are for.
    ///
    /// # Safety
    /// `name` must be NUL-terminated.
    unsafe fn other_name(&self, name: *const c_char) -> bool {
        // SAFETY: the caller's promise.
        unsafe { tagmatchname.get().is_null() || strcmp(tagmatchname.get(), name) != 0 }
    }

    /// Settle on which match to jump to, listing them and asking when the
    /// command is `:tselect` or an ambiguous `:tjump`.
    fn choose(&mut self) -> bool {
        // SAFETY: the caller's promise.
        let found = num_matches.get();
        let mut ask = false;
        if self.kind == DT_TAG as c_int && unsafe { *self.tag } != 0 {
            // A count on ":tag <name>" picks that match.
            self.cur_match = (self.count - 1).max(0);
        } else if self.kind == DT_SELECT as c_int || (self.kind == DT_JUMP as c_int && found > 1) {
            unsafe { print_tag_list(self.new_tag, self.use_tagstack, found, matches.get()) };
            ask = true;
        } else if self.kind == DT_LTAG as c_int {
            if unsafe { add_llist_tags(self.tag, found, matches.get()) } == FAIL {
                return false;
            }
            // Jump to the first tag.
            self.cur_match = 0;
        }

        if ask {
            let chosen = unsafe { prompt_for_input(ptr::null_mut(), 0, false, ptr::null_mut()) };
            if chosen <= 0 || chosen > found || got_int.get() {
                // No valid choice: change nothing.
                if self.use_tagstack {
                    self.current().fmark = self.saved_fmark.clone();
                    self.idx = self.prev_idx;
                }
                return false;
            }
            self.cur_match = chosen - 1;
        }

        if self.cur_match >= found {
            // Don't give this error when a file was not found and we
            // are looking for a match in another file that was not
            // found either: E429 says so below.
            if matches!(self.kind as c_uint, DT_NEXT | DT_FIRST) && nofile_fname.get().is_null() {
                tag_emsg(if found == 1 {
                    c"E427: There is only one matching tag"
                } else {
                    c"E428: Cannot go beyond last matching tag"
                });
                self.skip_msg = true;
            }
            self.cur_match = found - 1;
        }

        if self.use_tagstack {
            self.record_choice();
            self.idx += 1;
        } else if g_do_tagpreview.get() != 0 {
            ptag_entry_handle().set_position(self.cur_match, self.cur_fnum);
        }
        true
    }

    /// Record which match was taken on the stack entry, with whatever
    /// `'tagfunc'` attached to it.
    fn record_choice(&mut self) {
        // SAFETY: the caller's promise; the match outlives the copy taken
        // out of it.
        let entry = unsafe { self.entry(self.idx) };
        unsafe { (*entry).cur_match = self.cur_match };
        unsafe { (*entry).cur_fnum = self.cur_fnum };

        if !self.use_tfu {
            return;
        }
        let mut tp = TagParts::default();
        if !unsafe { parse_match(*matches.get().offset(self.cur_match as isize), &mut tp) }
            || tp.user_data.is_null()
        {
            return;
        }
        unsafe { xfree((*entry).user_data.cast()) };
        let len = unsafe { tp.user_data_end.offset_from(tp.user_data) } as size_t;
        unsafe { (*entry).user_data = xmemdupz(tp.user_data.cast(), len).cast() };
    }

    /// Jump to the chosen match.
    ///
    /// Answers `true` when the file it named did not exist and the next
    /// match is worth trying.
    ///
    /// # Safety
    /// `name` must be NUL-terminated and the globals live.
    unsafe fn jump(&mut self, name: *mut c_char) -> bool {
        // `v:swapcommand` is read by a SwapExists autocommand, which can
        // format anything it likes; the text is this frame's.
        let mut swapcmd = [0 as c_char; IOSIZE as usize];
        // SAFETY: the caller's promise.
        // Only when about to try the next match: otherwise E429 below
        // reports it.
        if !nofile_fname.get().is_null() && self.error_cur_match != self.cur_match {
            // SAFETY: the message macros expand to a `vim_snprintf` over
            // the format literal above and the editor's message buffers.
            unsafe {
                smsg_c!(
                    0,
                    gettext(c"File \"%s\" does not exist").as_ptr(),
                    nofile_fname.get(),
                )
            };
        }

        let entry = unsafe { *matches.get().offset(self.cur_match as isize) };
        let ignored_case = unsafe { *entry } as c_int & MT_IC_OFF as c_int != 0;
        self.report_count(ignored_case);

        // Let the SwapExists event know what tag is being jumped to.
        let str_m = IOSIZE as size_t;
        let fmt = c":ta %s\r".as_ptr();
        let len = unsafe { vim_snprintf_safelen(swapcmd.as_mut_ptr(), str_m, fmt, name) };
        unsafe { set_vim_var_string(Vv::Swapcommand, swapcmd.as_ptr(), len as ptrdiff_t) };
        let result = unsafe { jumpto_tag(entry, self.forceit, true) };
        unsafe { set_vim_var_string(Vv::Swapcommand, ptr::null(), -1) };

        if result != NOTAGFILE {
            // We may have jumped to another window; check the index is
            // still one this window has.
            if self.use_tagstack && self.idx > cur_win().w_tagstacklen {
                self.idx = cur_win().w_tagstackidx;
            }
            return false;
        }

        // The file does not exist: try the next matching tag, if there
        // can be one.
        let more = (self.kind == DT_PREV as c_int && self.cur_match > 0)
            || (matches!(self.kind as c_uint, DT_TAG | DT_NEXT | DT_FIRST)
                && (max_num_matches.get() != MAXCOL as c_int
                    || self.cur_match < num_matches.get() - 1));
        if !more {
            // SAFETY: the message macros expand to a `vim_snprintf` over // the format literal above and the editor's message buffers.
            let arg0 = unsafe { c_str(nofile_fname.get()) };
            semsg!("E429: File \"{arg0}\" does not exist");
            return false;
        }
        self.error_cur_match = self.cur_match;
        if self.use_tagstack {
            self.idx -= 1;
        }
        if self.kind == DT_PREV as c_int {
            self.cur_match -= 1;
        } else {
            self.kind = DT_NEXT as c_int;
            self.cur_match += 1;
        }
        true
    }

    /// Say which of how many matches this is, when that is worth saying.
    fn report_count(&self, ignored_case: bool) {
        let mut report = [0 as c_char; IOSIZE as usize];
        let found = num_matches.get();
        if self.selecting()
            || self.kind == DT_TAG as c_int
            || (found <= 1 && !ignored_case)
            || self.skip_msg
        {
            return;
        }
        // SAFETY: the caller's promise; `report` is `IOSIZE` bytes and
        // both writes are bounded by it.
        let buf = report.as_mut_ptr();
        let maxlen = IOSIZE as size_t;
        let format2 = gettext(c"tag %d of %d%s");
        let args = self.cur_match + 1;
        let arg6 = if max_num_matches.get() != MAXCOL as c_int {
            gettext(c" or more").as_ptr()
        } else {
            c"".as_ptr()
        };
        unsafe { snprintf(buf, maxlen, format2.as_ptr(), args, found, arg6) };
        if ignored_case {
            let src = gettext(c" Using tag with different case!");
            let dsize = IOSIZE as size_t;
            unsafe { xstrlcat(buf, src.as_ptr(), dsize) };
        }
        if (found > self.prev_num_matches || self.new_tag) && found > 1 {
            unsafe { msg_ptr(buf, if ignored_case { HLF_W } else { 0 }) };
            // Don't overwrite this message.
            msg_scroll.set(1);
        } else {
            unsafe { give_warning(buf, ignored_case, true) };
        }
        if ignored_case && msg_scrolled.get() == 0 && msg_silent.get() == 0 {
            unsafe { msg_delay(1007, true) };
        }
    }

    /// Write the stack index back, and clear the one-command flags.
    fn finish(&mut self) {
        // SAFETY: the caller's promise.
        // Only when using the tag stack, and only when the index is
        // one this window still has.
        if self.use_tagstack && self.idx <= cur_win().w_tagstacklen {
            cur_win().w_tagstackidx = self.idx;
        }
        // Don't split, or preview, next time.
        postponed_split.set(0);
        g_do_tagpreview.set(0);
    }
}

/// Move the matches we already had to the front of the new list.
///
/// Keeps the order from changing when `:tnext` jumps to another file,
/// which would otherwise re-prioritise everything.
///
/// # Safety
/// Both lists must hold their stated number of matches.
unsafe fn reorder_matches(new_matches: *mut *mut c_char, new_num_matches: c_int) {
    // SAFETY: the caller's promise; every match outlives the `TagParts`
    // taken from it.
    let mut at = 0;
    let mut old = TagParts::default();
    let mut new = TagParts::default();
    for j in 0..num_matches.get() {
        unsafe { parse_match(*matches.get().offset(j as isize), &mut old) };
        for i in at..new_num_matches {
            unsafe { parse_match(*new_matches.offset(i as isize), &mut new) };
            if unsafe { strcmp(old.tagname, new.tagname) } != 0 {
                continue;
            }
            let found = unsafe { *new_matches.offset(i as isize) };
            for k in (at + 1..=i).rev() {
                unsafe { *new_matches.offset(k as isize) = *new_matches.offset(k as isize - 1) };
            }
            unsafe { *new_matches.offset(at as isize) = found };
            at += 1;
            break;
        }
    }
}

/// Throw the remembered matches away.
///
/// # Safety
/// Must not be called while a match is still being read.
pub(crate) unsafe fn forget_matches() {
    // SAFETY: the caller's promise; the list is ours.
    unsafe { free_wild(num_matches.get(), matches.get()) };
    num_matches.set(0);
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
