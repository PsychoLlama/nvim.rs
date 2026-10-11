//! The Ex commands that act on the buffer's *text*.
//!
//! Carved by what the command does to it:
//!
//! | child | what |
//! | --- | --- |
//! | [`text`] | `:left`/`:right`/`:center` and `:ascii` |
//! | [`sort`] | `:sort` and `:uniq` |
//! | [`lines`] | `:move` and `:copy` |
//! | [`filter`] | `:!`, `:range!`, `:shell` and `:print` |
//! | [`write`] | `:write`/`:update`/`:wall`/`:wq` and their guards |
//! | [`ecmd`] | `do_ecmd`: every command that changes which file a window shows |
//! | [`append`] | `:append`/`:insert`/`:change`/`:z` |
//! | [`global`] | `:global`/`:vglobal` |
//! | [`subst`] | `:substitute`, split again around its 1,220-line engine |
//!
//! What stays here is what the children share -- the flag constants, the
//! `SortLine`/`SubResult`/`LineData` layouts, and `check_secure`,
//! `prepare_tagpreview`, `skip_vimgrep_pat` and `ex_oldfiles`, four helpers
//! that belong to no one command and that other modules import by name.
//!
//! The family's process-wide state lives with the code that drives it:
//! `PREVCMD` and `GLOBAL_NEED_MSG_KIND` in [`filter`], the seven `sort_*`
//! flags in [`sort`], `APPEND_INDENT` in [`append`], and `OLD_SUB` plus
//! `GLOBAL_NEED_BEGINLINE` in [`subst`] -- the last two are the only ones
//! read from outside their own child.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::autocmd::fire_autocmds;
use crate::charset::{skip, vim_is_ident_char};
use crate::cstr;
use crate::eval::typval::{NumBuf, list_find_str, list_iter, list_len};
use crate::eval::vars::get_vim_var_list_handle;
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::{cmdmod_has, do_exedit};
use crate::getchar::state::got_int;
use crate::guard::{sandbox, secure};
use crate::input::prompt_for_input;
use crate::message::state::{msg_scroll, quit_more};
use crate::message::{e_curdir, e_interr, e_invarg, e_noprevre, e_sandbox};
use crate::message::{
    emsg, message_filtered, msg, msg_clr_eos, msg_display, msg_end, msg_outnum, msg_putchar,
    msg_start, msg_starthere, msg_str, msgmore,
};
use crate::option::set_option_direct;
use crate::options::kOptFoldcolumn;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save_opt_of;
use crate::os::input::os_breakcheck;
use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, skip_regexp_at};
use crate::tag::state::g_do_tagpreview;
use crate::types::AutoEvent;
use crate::types::CAR;
use crate::types::CmdIdx;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::{
    BCount, BfaFlags, BlnFlags, CmdModFlags, DoBufAction, ExArg, ExtmarkOp, GetFileRet, LPos,
    LineNr, List, NUL, OptVal, OptionSetFlags, UndoObjectType, Vv,
};
use crate::window::{win_enter, win_split};
use crate::winlayer::{Buf, Win, windows};
use core::ffi::CStr;

// The carve of the transpiled module; see each child's docs.
mod append;
mod ecmd;
mod filter;
mod global;
mod lines;
mod sort;
mod subst;
mod text;
mod write;

pub use self::append::*;
pub use self::ecmd::*;
pub use self::filter::*;
pub use self::global::*;
pub use self::lines::*;
pub use self::sort::*;
pub use self::subst::*;
pub use self::text::*;
pub use self::write::*;

use crate::global_cell::GlobalCell;
use core::ffi::c_int;

pub(crate) static SUB_NSUBS: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static SUB_NLINES: GlobalCell<LineNr> = GlobalCell::new(0);

/// `:vimgrep`'s pattern argument at the start of a command line's string.
struct VimgrepPat<'a> {
    /// The pattern: the bare word, or what is between the delimiters.
    pattern: &'a [u8],
    /// The `g`/`j`/`f` flags after a delimited one.
    flags: ::core::ffi::c_int,
    /// Where the argument ends: at the white space after a bare word, past
    /// the flags after a delimited one.
    end: usize,
    /// Where what follows starts: past that white space, which upstream
    /// writes its NUL over when it is asked for the pattern.
    rest: usize,
}

/// Split `:vimgrep`'s pattern argument off the start of `text`, which stops
/// at its first NUL. `None` when a delimited pattern is not closed -- or
/// when there is no argument at all, where upstream's walk starts past the
/// terminator.
fn vimgrep_pat(text: &[u8]) -> Option<VimgrepPat<'_>> {
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    let &first = text.first()?;
    if vim_is_ident_char(::core::ffi::c_int::from(first)) {
        // A bare word, ending at the first white space.
        let at = skip::to_white(text);
        return Some(VimgrepPat {
            pattern: &text[..at],
            flags: 0,
            end: at,
            rest: (at + 1).min(text.len()),
        });
    }

    // A delimited pattern.  The delimiter is whatever byte opened it.
    let delim = ::core::ffi::c_int::from(first);
    let mut terminated = text.to_vec();
    terminated.push(0);
    let close = 1 + skip_regexp_at(&terminated[1..], delim, 1);
    if ::core::ffi::c_int::from(cstr::byte_at(text, close)) != delim {
        return None;
    }
    let mut flags = 0;
    let mut n = 0;
    for &byte in &text[close + 1..] {
        flags |= match byte {
            b'g' => VGR_GLOBAL,
            b'j' => VGR_NOJUMP,
            b'f' => VGR_FUZZY,
            _ => break,
        } as ::core::ffi::c_int;
        n += 1;
    }
    let end = close + 1 + n;
    Some(VimgrepPat {
        pattern: &text[1..close],
        flags,
        end,
        rest: end,
    })
}

/// `:vimgrep`'s pattern argument at the start of `arg`: the pattern, the
/// flags after it, and where in `arg` what follows starts. `None` when the
/// closing delimiter is missing.
pub(crate) fn split_vimgrep_pat(arg: &[u8]) -> Option<(Vec<u8>, ::core::ffi::c_int, usize)> {
    let pat = vimgrep_pat(arg)?;
    Some((pat.pattern.to_vec(), pat.flags, pat.rest))
}

pub const _ISalpha: ::core::ffi::c_uint = 1024;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const REGSUB_BACKSLASH: ::core::ffi::c_uint = 4;
pub const REGSUB_MAGIC: ::core::ffi::c_uint = 2;
pub const REGSUB_COPY: ::core::ffi::c_uint = 1;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const GETFILE_OPEN_OTHER: GetFileRet = -1;
pub const GETFILE_SAME_FILE: GetFileRet = 0;
pub const GETFILE_NOT_WRITTEN: GetFileRet = 2;
pub const GETFILE_ERROR: GetFileRet = 1;
pub const BLN_NOCURWIN: BlnFlags = 128;
pub const BLN_LISTED: BlnFlags = 2;
pub const BLN_CURBUF: BlnFlags = 1;
pub const DOBUF_WIPE: DoBufAction = 4;
pub const DOBUF_DEL: DoBufAction = 3;
pub const DOBUF_UNLOAD: DoBufAction = 2;
pub const BFA_KEEP_UNDO: BfaFlags = 4;
pub const HIST_SEARCH: ::core::ffi::c_int = 1;
pub const VIM_QUESTION: ::core::ffi::c_uint = 4;
pub const VIM_YES: ::core::ffi::c_uint = 2;
pub const ML_DEL_MESSAGE: ::core::ffi::c_uint = 1;
pub const READ_FILTER: ::core::ffi::c_uint = 2;
pub const READ_NOWINENTER: ::core::ffi::c_uint = 128;
pub const READ_KEEP_UNDO: ::core::ffi::c_uint = 32;
pub const BCO_ENTER: ::core::ffi::c_uint = 1;
pub const CCGD_EXCMD: ::core::ffi::c_uint = 16;
pub const CCGD_FORCEIT: ::core::ffi::c_uint = 4;
pub const CCGD_MULTWIN: ::core::ffi::c_uint = 2;
pub const CCGD_AW: ::core::ffi::c_uint = 1;
#[derive(Copy, Clone)]
pub struct SubResult {
    pub start: LPos,
    pub end: LPos,
    pub pre_match: LineNr,
}
/// The matches an `'inccommand'` preview has to show, and how many lines they
/// need on screen -- which is what caps the scan when the preview window is
/// smaller than the range.
#[derive(Default)]
pub struct PreviewLines {
    pub subresults: Vec<SubResult>,
    pub lines_needed: LineNr,
}
#[derive(Copy, Clone)]
pub struct SubFlags {
    pub do_all: bool,
    pub do_ask: bool,
    pub do_count: bool,
    pub do_error: bool,
    pub do_print: bool,
    pub do_list: bool,
    pub do_number: bool,
    pub do_ic: SubIgnoreType,
}
pub type SubIgnoreType = ::core::ffi::c_uint;
pub const kSubMatchCase: SubIgnoreType = 2;
pub const kSubIgnoreCase: SubIgnoreType = 1;
pub const kSubHonorOptions: SubIgnoreType = 0;
pub struct LineData {
    pub start_col: ::core::ffi::c_int,
    pub start: LPos,
    pub end: LPos,
    pub matchcols: ::core::ffi::c_int,
    pub matchbytes: BCount,
    pub subcols: ::core::ffi::c_int,
    pub subbytes: BCount,
    pub lnum_before: LineNr,
    pub lnum_after: LineNr,
}
pub const VGR_FUZZY: ::core::ffi::c_uint = 4;
pub const VGR_NOJUMP: ::core::ffi::c_uint = 2;
pub const VGR_GLOBAL: ::core::ffi::c_uint = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_OTHER: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EXFLAG_LIST: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EXFLAG_NR: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EXFLAG_PRINT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
/// Fire `event` for `buffer`: no file name, no pattern, no forcing -- the shape
/// every buffer-lifecycle autocommand in this family uses.
///
/// Safe: [`Buf`] is the live buffer `apply_autocmds` asks for, and the two
/// file names it also wants are null here.
pub(super) fn buf_autocmd(event: AutoEvent, buffer: Buf) -> bool {
    fire_autocmds(event, false, Some(buffer))
}
/// Refuse anything that reaches outside the editor while 'secure' is on or a
/// sandbox is open -- shell commands, `:write`, `:cd` and friends.
///
/// Returns true and gives a message when the command must not run.
///
/// Safe: the only promise is that the editor exists; the two messages are
/// static strings.
pub fn check_secure() -> bool {
    if secure.get() != 0 {
        secure.set(2);
        emsg(gettext(e_curdir));
        return true;
    }
    if sandbox.get() != 0 {
        emsg(gettext(e_sandbox));
        return true;
    }
    false
}
pub fn prepare_tagpreview(undo_sync: bool) -> bool {
    if Win::current().w_onebuf_opt.wo_pvw != 0 {
        return false;
    }
    for wp in windows() {
        if wp.w_onebuf_opt.wo_pvw != 0 {
            win_enter(wp, undo_sync);
            return false;
        }
    }
    if win_split(
        if g_do_tagpreview.get() > 0 as ::core::ffi::c_int {
            g_do_tagpreview.get()
        } else {
            0 as ::core::ffi::c_int
        },
        0 as ::core::ffi::c_int,
    )
    .is_err()
    {
        return false;
    }
    Win::current().w_onebuf_opt.wo_pvw = 1;
    Win::current().w_onebuf_opt.wo_wfh = 1;
    Win::current().w_onebuf_opt.wo_scb = 0;
    Win::current().w_onebuf_opt.wo_crb = 0;
    Win::current().w_onebuf_opt.wo_diff = 0;
    set_option_direct(
        kOptFoldcolumn,
        OptVal::static_string(c"0"),
        OptionSetFlags::NONE,
        SID_NONE,
    );
    true
}
/// How many bytes `:vimgrep`'s pattern argument and its flags take at the
/// start of `text`, or `None` when a delimited pattern is not closed.
pub(crate) fn skip_vimgrep_pat_at(text: &CStr) -> Option<usize> {
    vimgrep_pat(text.to_bytes()).map(|pat| pat.end)
}
/// `:oldfiles` -- list `v:oldfiles`, numbered; under `:browse`, then ask for
/// a number and edit that file.
pub fn ex_oldfiles(excmd: &mut ExArg) {
    // A reference of this frame's own: the prompt below runs user code,
    // which may assign `v:oldfiles`.
    let Some(list) = get_vim_var_list_handle(Vv::Oldfiles) else {
        msg(gettext(c"No old files"), 0);
        return;
    };

    say::start();
    msg_scroll.set(1);
    list_oldfiles(&list);
    got_int.set(false);
    if !cmdmod_has(CmdModFlags::BROWSE) {
        return;
    }

    quit_more.set(false);
    let nr = prompt_for_input(None, 0, false, None);
    say::starthere();
    if nr <= 0 || nr > list_len(Some(&list)) {
        return;
    }
    let mut numbuf = NumBuf::new();
    // `nr` is inside the list.
    let Some(picked) = list_find_str(Some(&list), nr - 1, &mut numbuf) else {
        return;
    };
    let expanded = expand_env_save_opt_of(picked, false);
    excmd.set_arg_text(&expanded);
    excmd.cmdidx = CmdIdx::edit;
    cmdmod.with_mut(|m| m.cmod_flags.clear(CmdModFlags::BROWSE));
    do_exedit(excmd, None);
}

/// Number and print every entry of `list`, stopping on an interrupt.
fn list_oldfiles(list: &List) {
    let mut number = NumBuf::new();
    let mut text = NumBuf::new();
    let mut nr = 0;
    for item in list_iter(Some(list)) {
        if got_int.get() {
            break;
        }
        nr += 1;
        let value = &item.li_tv;
        if !message_filtered(number.string(value)) {
            msg_outnum(nr);
            say::puts(c": ");
            msg_display(text.string(value), 0, false);
            say::clear_eos();
            say::putchar('\n' as ::core::ffi::c_int);
            os_breakcheck();
        }
    }
}

/// A NUL-terminated copy of one line, refilled in place.
///
/// `ml_append` unlocks the memline block the line it is handed lives in, so
/// every "append a copy of this line" in the family has to take the bytes out
/// first; `:uniq` and `:sort u` want the same buffer to compare one line with
/// the one before. Upstream does both with `xmalloc`ed scratch and a `strcpy`
/// or an `xstrnsave`/`xfree` pair per line. One `Vec` that keeps its capacity
/// says it once, and starts out holding the empty string so a comparison
/// against a copy that has not been filled in yet still reads a string.
pub(crate) struct LineCopy(Vec<u8>);

impl LineCopy {
    pub(crate) fn new() -> LineCopy {
        LineCopy(vec![NUL as u8])
    }

    /// Replace the contents with `bytes` and a NUL.
    pub(crate) fn fill(&mut self, bytes: &[u8]) {
        self.0.clear();
        self.0.extend_from_slice(bytes);
        self.0.push(NUL as u8);
    }

    /// Replace the contents with line `lnum` of the current buffer, and
    /// answer how many bytes that line held.
    pub(crate) fn fill_line(&mut self, lnum: LineNr) -> usize {
        let mut lines = crate::memline::Lines::current();
        let text = lines.line(lnum);
        let len = text.len();
        self.0.clear();
        self.0.extend_from_slice(text);
        self.0.push(NUL as u8);
        len
    }

    /// The copy's bytes and its one NUL.
    pub(crate) fn with_nul(&self) -> &[u8] {
        &self.0
    }

    /// The copy as a C string.
    pub(crate) fn as_cstr(&self) -> &::core::ffi::CStr {
        ::core::ffi::CStr::from_bytes_until_nul(&self.0).expect("the copy ends in a NUL")
    }
}

/// The message calls this family makes, as checked code.
///
/// Every one is an `unsafe fn` whose whole promise is "the message layer's
/// own statics are consistent" -- a whole-program invariant no call site can
/// do anything about, and which none of them was discharging with more than a
/// copy of the same sentence. Discharged once here instead of at forty call
/// sites. The real fix is the message layer's own signatures; this is the
/// family's share of it.
pub(crate) mod say {
    use super::{msg_clr_eos, msg_end, msg_putchar, msg_start, msg_starthere, msg_str, msgmore};
    use ::core::ffi::{CStr, c_int};

    /// [`msg_start`]: begin a message.
    pub(crate) fn start() {
        msg_start()
    }

    /// [`msg_starthere`]: put the next message where the cursor is.
    pub(crate) fn starthere() {
        msg_starthere()
    }

    /// [`msg_end`]: finish a message, prompting if it did not fit.  False
    /// when `wait_return` was called.
    pub(crate) fn end() -> bool {
        msg_end()
    }

    /// [`msg_putchar`]: show one character.
    pub(crate) fn putchar(c: c_int) {
        msg_putchar(c)
    }

    /// [`msg_str`]: show a string that carries its own NUL.
    pub(crate) fn puts(s: &CStr) {
        msg_str(s)
    }

    /// [`msg_clr_eos`]: clear from the message position to the end.
    pub(crate) fn clear_eos() {
        msg_clr_eos()
    }

    /// [`msgmore`]: "N more lines" / "N fewer lines".
    pub(crate) fn more(n: c_int) {
        msgmore(n)
    }
}
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const DBL_MAX: ::core::ffi::c_double = __DBL_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const __DBL_MAX__: ::core::ffi::c_double = 1.7976931348623157e+308f64;
