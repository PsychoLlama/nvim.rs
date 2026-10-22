//! The `:tag` family of commands.
//!
//! [`do_tag`] is all of `:tag`, `:tselect`, `:tjump`, `:tnext`, `:tprev`,
//! `:tfirst`, `:tlast`, `:ltag`, `:pop` and `CTRL-T`. It decides which
//! match of which tag the user asked for, records the jump on the window's
//! tag stack, and hands the match to [`jumpto_tag`].
//!
//! The matches of the last tag looked up are remembered between calls, so
//! that `:tnext` does not have to read the tags files again.

#![forbid(unsafe_code)]

use super::Jumped;
use super::*;
use crate::formatted;
use crate::highlight_group::HLF_W;
use crate::memory::{ThinCString, XString};
use crate::message::{MSG_IOBUFF_LEN, msg};
use crate::message_fmt::{msg_cstr, to_message};
use crate::pos::MAXCOL;
use crate::semsg;
use crate::types::{IOSIZE, Vv};
use crate::winlayer::{Buf, Win, WinId};
use crate::{smsg, tr};
use core::ffi::{CStr, c_int, c_uint};

/// The matches of the tag last looked up, in the order they are numbered.
static TAG_MATCHES: GlobalCell<Vec<TagMatch>> = GlobalCell::new(Vec::new());

/// How many matches there are.
fn match_count() -> c_int {
    TAG_MATCHES.with(|found| c_int::try_from(found.len()).unwrap_or(c_int::MAX))
}

/// A copy of match `i`, which must be one of them.
fn match_at(i: c_int) -> TagMatch {
    let i = usize::try_from(i).expect("a match index");
    TAG_MATCHES.with(|found| found[i].clone())
}

/// The limit the last search used. `MAXCOL` means every match was found,
/// so there is no point looking again for a later one.
static MATCH_LIMIT: GlobalCell<c_int> = GlobalCell::new(0);

/// `:tlast`. The parent's `DT_*` family is the transpile's, which left
/// this one out; it is used here and nowhere else in the module.
const DT_LAST: c_uint = 6;

/// The view the tag stack starts a jump from: no remembered scroll
/// position.
const NO_VIEW: FileMarkView = FileMarkView {
    topline_offset: MAXLNUM,
    skipcol: 0,
};

/// The preview window's entry: the match it shows, and the buffer it came
/// from.
fn ptag_position() -> (c_int, c_int) {
    PREVIEW_ENTRY.with(|entry| (entry.cur_match, entry.cur_fnum))
}

/// Remember where the preview window got to.
fn set_ptag_position(cur_match: c_int, cur_fnum: c_int) {
    PREVIEW_ENTRY.with_mut(|entry| {
        entry.cur_match = cur_match;
        entry.cur_fnum = cur_fnum;
    });
}

/// Forget the previous preview tag and remember `tag` instead.
fn restart_ptag(tag: &XString) {
    PREVIEW_ENTRY.with_mut(|entry| {
        tagstack_clear_entry(entry);
        entry.tagname = Some(ThinCString::from_bytes(tag));
    });
}

/// Jump to a tag, or move about the tag stack.
///
/// `tag` is the tag or pattern, empty for the forms that take none.
/// `kind` is one of the `DT_*` constants, `count` the command's count,
/// `forceit` its `!`, and `verbose` asks for "tag not found" to be
/// reported.
pub fn do_tag(tag: &CStr, kind: c_int, count: c_int, forceit: c_int, verbose: bool) {
    if TAGFUNC_RUNNING.get() {
        // The stack is what `'tagfunc'` is being asked to describe.
        tag_emsg(c"E986: Cannot modify the tag stack within tagfunc");
        return;
    }
    if postponed_split.get() == 0 && !check_can_set_curbuf_forceit(forceit) {
        return;
    }

    let mut cmd = DoTag::new(tag, kind, count, forceit, verbose);
    if cmd.prepare() {
        cmd.set_priority_buffer();
        cmd.run();
    }
    cmd.finish();
}

/// One `:tag`-family command in progress.
struct DoTag {
    /// The tag or pattern given, empty for the argument-less forms.
    tag: XString,
    /// Which `DT_*` command this is. `DT_HELP` becomes `DT_TAG`, and a
    /// retry after a missing file becomes `DT_NEXT`.
    kind: c_int,
    count: c_int,
    forceit: c_int,
    verbose: bool,

    /// The window whose tag stack the command works on, by identity:
    /// `'tagfunc'` may close it, and comparing it against the current
    /// window afterwards is how that is noticed.
    win: WinId,
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
    saved_fmark: FileMark,
    /// The name of the buffer matches are prioritised against, a copy:
    /// `'tagfunc'` may rename the buffer.
    buf_ffname: Option<XString>,
    /// How many matches the previous command found.
    prev_num_matches: c_int,
}

impl DoTag {
    fn new(tag: &CStr, kind: c_int, count: c_int, forceit: c_int, verbose: bool) -> Self {
        let win = Win::current();
        // `:help` tags are literal, and 'tagfunc' has no business
        // answering for them.
        let help = kind == DT_HELP as c_int;
        let idx = win.w_tagstackidx;

        let cmd = DoTag {
            tag: XString::from_cstr(tag),
            kind: if help { DT_TAG as c_int } else { kind },
            count,
            forceit,
            verbose,
            win: win.id(),
            idx,
            len: win.w_tagstacklen,
            old_idx: idx,
            prev_idx: idx,
            cur_match: 0,
            cur_fnum: Buf::current().handle,
            error_cur_match: 0,
            new_tag: false,
            no_regexp: help,
            use_tfu: !help,
            use_tagstack: false,
            save_pos: false,
            skip_msg: false,
            saved_fmark: FileMark {
                mark: Pos::default(),
                fnum: 0,
                timestamp: 0,
                view: NO_VIEW,
                extra: None,
            },
            buf_ffname: Buf::current().name.full().map(XString::from_cstr),
            prev_num_matches: match_count(),
        };
        drop(MISSING_FILE.take());
        cmd
    }

    /// Work on stack entry `at` of the window the command started in.
    ///
    /// Answers `None` when that window has gone, or the entry is not one
    /// the stack has.
    fn with_entry<R>(&self, at: c_int, f: impl FnOnce(&mut Taggy) -> R) -> Option<R> {
        let mut win = self.win.get()?;
        let at = usize::try_from(at).ok()?;
        win.w_tagstack.get_mut(at).map(f)
    }

    /// [`with_entry`](Self::with_entry) for the entry the command stands on.
    fn with_current<R>(&self, f: impl FnOnce(&mut Taggy) -> R) -> Option<R> {
        self.with_entry(self.idx, f)
    }

    /// Work out which tag and which of its matches is wanted, and record
    /// the jump on the stack.
    ///
    /// Answers `false` when the command is already finished — an empty
    /// stack, a `:pop` that walked off the end, or a `CTRL-T` that has
    /// already jumped.
    fn prepare(&mut self) -> bool {
        if !p_tgst() && !self.tag.is_empty() {
            // 'tagstack' is off: jump without recording anything.
            self.use_tagstack = false;
            self.new_tag = true;
            if g_do_tagpreview.get() != 0 {
                restart_ptag(&self.tag);
            }
            return true;
        }

        // A preview tag has its own one-entry stack.
        self.use_tagstack = g_do_tagpreview.get() == 0;

        let named = matches!(self.kind as c_uint, DT_TAG | DT_SELECT | DT_JUMP | DT_LTAG);
        if !self.tag.is_empty() && named {
            self.push_new_tag();
            self.new_tag = true;
        } else if !self.walk_stack() {
            return false;
        }

        if g_do_tagpreview.get() != 0 {
            if !self.selecting() {
                set_ptag_position(self.cur_match, self.cur_fnum);
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

    /// A new tag was named: put it on the stack (or on the preview entry).
    fn push_new_tag(&mut self) {
        if g_do_tagpreview.get() != 0 {
            let same = PREVIEW_ENTRY.with(|entry| {
                entry
                    .tagname
                    .as_ref()
                    .is_some_and(|previous| previous.as_bytes() == &self.tag[..])
            });
            if same {
                // Jumping to the same tag again: keep the current
                // match, so that the CursorHold example works.
                (self.cur_match, self.cur_fnum) = ptag_position();
            } else {
                restart_ptag(&self.tag);
            }
            return;
        }

        let Some(mut win) = self.win.get() else {
            return;
        };
        let stack = &mut win.w_tagstack;
        // Anything above the entry last used is now unreachable.
        while self.idx < self.len {
            self.len -= 1;
            tagstack_clear_entry(&mut stack[stack_index(self.len)]);
        }

        self.len += 1;
        if self.len > TAGSTACKSIZE {
            // Full: drop the oldest entry and shift the rest down.
            self.len = TAGSTACKSIZE;
            drop_oldest(&mut stack[..stack_index(self.len)]);
            self.idx -= 1;
        }

        stack[stack_index(self.idx)].tagname = Some(ThinCString::from_bytes(&self.tag));
        win.w_tagstacklen = self.len;
        // Worth remembering where the cursor was.
        self.save_pos = true;
    }

    /// No tag was named: move about the stack instead.
    ///
    /// Answers `false` when the command is finished — the stack was empty,
    /// or a `:pop` has already done the jump.
    fn walk_stack(&mut self) -> bool {
        let empty = if g_do_tagpreview.get() != 0 {
            PREVIEW_ENTRY.with(|entry| entry.tagname.is_none())
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
        let Some(mark) = self.with_current(|entry| entry.fmark.clone()) else {
            return false;
        };
        self.saved_fmark = mark.clone();
        if mark.fnum != Buf::current().handle {
            // Another file. If it cannot be opened (it may have
            // changed) keep the original position on the stack.
            if buflist_getfile(
                mark.fnum,
                mark.mark.lnum,
                GETF_SETMARK as c_int,
                self.forceit,
            )
            .is_err()
            {
                self.idx = self.old_idx;
                return false;
            }
            // A BufReadPost autocommand may jump to the '" mark, which
            // is not wanted here.
            Win::current().w_cursor.lnum = mark.mark.lnum;
        } else {
            setpcmark();
            Win::current().w_cursor.lnum = mark.mark.lnum;
        }
        Win::current().w_cursor.col = mark.mark.col;
        Win::current().w_set_curswant = true;
        if jop_flags.get() & kOptJopFlagView as c_uint != 0 {
            mark_view_restore(Some(&self.saved_fmark));
        }
        check_cursor(Win::current());
        if fdo_flags.get() & kOptFdoFlagTag as c_uint != 0 && old_key_typed {
            fold_open_cursor();
        }

        // The remembered matches are for a tag we have left.
        forget_matches();
        tag_freematch();
        false
    }

    /// `:tag` with no argument: go to the newer entry on the stack.
    ///
    /// Answers `false` when the command is finished.
    fn go_newer(&mut self) -> bool {
        if g_do_tagpreview.get() != 0 {
            (self.cur_match, self.cur_fnum) = ptag_position();
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
        if let Some(position) = self.with_current(|entry| (entry.cur_match, entry.cur_fnum)) {
            (self.cur_match, self.cur_fnum) = position;
        }
        true
    }

    /// `:tnext` and friends: another match of the tag already on the stack.
    fn go_other_match(&mut self) {
        // Where to go back to if the selection is cancelled.
        self.prev_idx = self.idx;

        if g_do_tagpreview.get() != 0 {
            (self.cur_match, self.cur_fnum) = ptag_position();
        } else {
            self.idx = (self.idx - 1).max(0);
            if let Some(position) = self.with_current(|entry| (entry.cur_match, entry.cur_fnum)) {
                (self.cur_match, self.cur_fnum) = position;
            }
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
            self.cur_fnum = Buf::current().handle;
        }
    }

    /// Remember on the stack where the jump is starting from.
    fn record_position(&mut self) {
        let save_pos = self.save_pos;
        let selecting = self.selecting();
        let (cur_match, cur_fnum) = (self.cur_match, self.cur_fnum);
        let cursor = Win::current().w_cursor;
        let view = mark_view_make(Win::current(), cursor);
        let handle = Buf::current().handle;
        let saved = self.with_current(|entry| {
            let saved = entry.fmark.clone();
            if save_pos {
                entry.fmark.mark = cursor;
                entry.fmark.fnum = handle;
                entry.fmark.view = view;
            }
            if !selecting {
                entry.cur_match = cur_match;
                entry.cur_fnum = cur_fnum;
            }
            saved
        });
        if let Some(saved) = saved {
            self.saved_fmark = saved;
        }

        // `curwin` changes in `jumpto_tag` for `:stag`, or when an
        // autocommand jumps to another window, so store the index now.
        Win::current().w_tagstackidx = self.idx;
    }

    /// Prioritise matches against the buffer `cur_fnum` names.
    ///
    /// Using a remembered `cur_match` only makes sense if the order the
    /// matches came out in is the same as it was then.
    fn set_priority_buffer(&mut self) {
        if self.cur_fnum == Buf::current().handle {
            return;
        }
        if let Some(buf) = find_buf(self.cur_fnum) {
            self.buf_ffname = buf.name.full().map(XString::from_cstr);
        }
    }

    /// Find the matches, choose one, and jump to it — retrying with the
    /// next match when the file a match names does not exist.
    fn run(&mut self) {
        loop {
            let mut name = self.tag_name();
            // Whether the remembered matches are for a different tag.
            // Read before `search` replaces the remembered name.
            let other = MATCHES_ARE_FOR.with(|remembered| {
                remembered
                    .as_ref()
                    .is_none_or(|remembered| remembered != &name)
            });
            if (self.new_tag
                || (self.cur_match >= match_count() && MATCH_LIMIT.get() != MAXCOL as c_int)
                || other)
                && !self.search(&mut name, other)
            {
                return;
            }

            if match_count() <= 0 {
                if self.verbose {
                    let name = msg_cstr(name.as_cstr());
                    semsg!("E426: Tag not found: {name}");
                }
                g_do_tagpreview.set(0);
                return;
            }
            if !self.choose() || !self.jump(&name) {
                return;
            }
        }
    }

    /// The name to look up — the stack entry's, the preview entry's, or the
    /// command's own argument — as a copy: `'tagfunc'` may rewrite the
    /// stack under us.
    fn tag_name(&self) -> XString {
        let from_entry = |entry: &Taggy| {
            entry
                .tagname
                .as_ref()
                .map_or_else(XString::new, |name| XString::from_bytes(name.as_bytes()))
        };
        if self.use_tagstack {
            self.with_current(|entry| from_entry(entry))
                .unwrap_or_default()
        } else if g_do_tagpreview.get() != 0 {
            PREVIEW_ENTRY.with(from_entry)
        } else {
            self.tag.clone()
        }
    }

    /// Read the tags files (or ask `'tagfunc'`) for `name`, stepping it
    /// past the `/` that makes it a regexp: what the messages after this
    /// name is the pattern.
    ///
    /// Answers `false` when the window went away while searching.
    fn search(&mut self, name: &mut XString, other: bool) -> bool {
        if other {
            MATCHES_ARE_FOR.set(Some(name.clone()));
        }

        if matches!(self.kind as c_uint, DT_SELECT | DT_JUMP | DT_LTAG) {
            // Every match is wanted, to list them.
            self.cur_match = MAXCOL as c_int - 1;
        }
        MATCH_LIMIT.set(if self.kind == DT_TAG as c_int {
            MAXCOL as c_int
        } else {
            self.cur_match + 1
        });

        // An argument starting with '/' is a regexp; anything else is
        // a literal name, and case is then not folded.
        let mut flags = match &name[..] {
            [b'/', rest @ ..] if !self.no_regexp => {
                *name = XString::from_bytes(rest);
                TAG_REGEXP as c_int
            }
            _ => TAG_NOIC as c_int,
        };
        if self.verbose {
            flags |= TAG_VERBOSE as c_int;
        }
        if !self.use_tfu {
            flags |= TAG_NO_TAGFUNC as c_int;
        }

        let buf_ffname = self.buf_ffname.as_ref().map(XString::as_cstr);
        let found = find_tags(name, flags, MATCH_LIMIT.get(), buf_ffname);
        let mut new_matches = match found {
            Ok(found) => {
                if c_int::try_from(found.len()).is_ok_and(|n| n < MATCH_LIMIT.get()) {
                    // Fewer than the limit: that is all of them.
                    MATCH_LIMIT.set(MAXCOL as c_int);
                }
                found
            }
            Err(_) => Vec::new(),
        };

        // A tag function may do anything, which may make all sorts of
        // things invalid. At least check that the window whose stack we
        // are working on is still the current one.
        if Win::current().id() != self.win {
            tag_emsg(c"E1299: Window unexpectedly closed while searching for tags");
            return false;
        }

        if !self.new_tag && !other {
            TAG_MATCHES.with(|old| reorder_matches(old, &mut new_matches));
        }
        TAG_MATCHES.set(new_matches);
        true
    }

    /// Settle on which match to jump to, listing them and asking when the
    /// command is `:tselect` or an ambiguous `:tjump`.
    fn choose(&mut self) -> bool {
        let found = match_count();
        let mut ask = false;
        if self.kind == DT_TAG as c_int && !self.tag.is_empty() {
            // A count on ":tag <name>" picks that match.
            self.cur_match = (self.count - 1).max(0);
        } else if self.kind == DT_SELECT as c_int || (self.kind == DT_JUMP as c_int && found > 1) {
            // A copy: the listing may be interrupted by anything.
            let matches = TAG_MATCHES.with(Clone::clone);
            print_tag_list(self.new_tag, self.use_tagstack, &matches);
            ask = true;
        } else if self.kind == DT_LTAG as c_int {
            let matches = TAG_MATCHES.with(Clone::clone);
            add_llist_tags(&self.tag, &matches);
            // Jump to the first tag.
            self.cur_match = 0;
        }

        if ask {
            let chosen = prompt_for_input(None, 0, false, None);
            if chosen <= 0 || chosen > found || got_int.get() {
                // No valid choice: change nothing.
                if self.use_tagstack {
                    let saved = self.saved_fmark.clone();
                    self.with_current(|entry| entry.fmark = saved);
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
            if matches!(self.kind as c_uint, DT_NEXT | DT_FIRST)
                && MISSING_FILE.with(Option::is_none)
            {
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
            set_ptag_position(self.cur_match, self.cur_fnum);
        }
        true
    }

    /// Record which match was taken on the stack entry, with whatever
    /// `'tagfunc'` attached to it.
    fn record_choice(&mut self) {
        let (cur_match, cur_fnum) = (self.cur_match, self.cur_fnum);
        let user_data = if self.use_tfu {
            let found = match_at(self.cur_match);
            match found.parse() {
                (tp, true) => tp
                    .user_data
                    .map(|span| ThinCString::from_bytes(span.of(found.line()))),
                (_, false) => None,
            }
        } else {
            None
        };
        self.with_current(|entry| {
            entry.cur_match = cur_match;
            entry.cur_fnum = cur_fnum;
            if user_data.is_some() {
                entry.user_data = user_data;
            }
        });
    }

    /// Jump to the chosen match.
    ///
    /// Answers `true` when the file it named did not exist and the next
    /// match is worth trying.
    fn jump(&mut self, name: &XString) -> bool {
        // Only when about to try the next match: otherwise E429 below
        // reports it.
        if self.error_cur_match != self.cur_match
            && let Some(missing) = MISSING_FILE.with(Clone::clone)
        {
            let arg0 = msg_cstr(missing.as_cstr());
            smsg!(0, "File \"{arg0}\" does not exist");
        }

        let found = match_at(self.cur_match);
        let ignored_case = usize::from(found.priority) & MT_IC_OFF as usize != 0;
        self.report_count(ignored_case);

        // Let the SwapExists event know what tag is being jumped to, cut
        // where upstream's `IOSIZE` scratch cut it.
        let swapcmd = formatted!(IOSIZE as usize, b":ta %s\r", &name[..]);
        set_vim_var_string(Vv::Swapcommand, Some(&swapcmd));
        let result = jumpto_tag(found, self.forceit, true);
        set_vim_var_string(Vv::Swapcommand, None);

        if result != Ok(Jumped::NoSuchFile) {
            // We may have jumped to another window; check the index is
            // still one this window has.
            if self.use_tagstack && self.idx > Win::current().w_tagstacklen {
                self.idx = Win::current().w_tagstackidx;
            }
            return false;
        }

        // The file does not exist: try the next matching tag, if there
        // can be one.
        let more = (self.kind == DT_PREV as c_int && self.cur_match > 0)
            || (matches!(self.kind as c_uint, DT_TAG | DT_NEXT | DT_FIRST)
                && (MATCH_LIMIT.get() != MAXCOL as c_int || self.cur_match < match_count() - 1));
        if !more {
            let missing = MISSING_FILE.with(Clone::clone).unwrap_or_default();
            let arg0 = msg_cstr(missing.as_cstr());
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
        let found = match_count();
        if self.selecting()
            || self.kind == DT_TAG as c_int
            || (found <= 1 && !ignored_case)
            || self.skip_msg
        {
            return;
        }
        let more = if MATCH_LIMIT.get() != MAXCOL as c_int {
            tr!(" or more")
        } else {
            String::new()
        };
        let mut text = tr!("tag {} of {}{}", self.cur_match + 1, found, more);
        if ignored_case {
            text.push_str(&tr!(" Using tag with different case!"));
        }
        // `IObuff` is where upstream assembled this, and it truncated there.
        let report = to_message(text, MSG_IOBUFF_LEN);
        if (found > self.prev_num_matches || self.new_tag) && found > 1 {
            msg(&report, if ignored_case { HLF_W } else { 0 });
            // Don't overwrite this message.
            msg_scroll.set(1);
        } else {
            give_warning(&report, ignored_case, true);
        }
        if ignored_case && msg_scrolled.get() == 0 && msg_silent.get() == 0 {
            msg_delay(1007, true);
        }
    }

    /// Write the stack index back, and clear the one-command flags.
    fn finish(&mut self) {
        // Only when using the tag stack, and only when the index is
        // one this window still has.
        if self.use_tagstack && self.idx <= Win::current().w_tagstacklen {
            Win::current().w_tagstackidx = self.idx;
        }
        // Don't split, or preview, next time.
        postponed_split.set(0);
        g_do_tagpreview.set(0);
    }
}

/// A stack position as an index; the command keeps them in range.
fn stack_index(at: c_int) -> usize {
    usize::try_from(at).expect("a tag stack index")
}

/// Move the matches we already had to the front of the new list.
///
/// Keeps the order from changing when `:tnext` jumps to another file,
/// which would otherwise re-prioritise everything. Two matches are the
/// same when their lines are.
fn reorder_matches(old: &[TagMatch], new: &mut [TagMatch]) {
    let mut at = 0;
    for previous in old {
        if let Some(i) = (at..new.len()).find(|&i| new[i].line() == previous.line()) {
            new[at..=i].rotate_right(1);
            at += 1;
        }
    }
}

/// Throw the remembered matches away.
pub(crate) fn forget_matches() {
    TAG_MATCHES.set(Vec::new());
}
