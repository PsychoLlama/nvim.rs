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
//! `sorti_T`/`SubResult`/`LineData` layouts, and `check_secure`,
//! `prepare_tagpreview`, `skip_vimgrep_pat` and `ex_oldfiles`, four helpers
//! that belong to no one command and that other modules import by name.
//!
//! The family's process-wide state lives with the code that drives it:
//! `prevcmd` and `global_need_msg_kind` in [`filter`], the seven `sort_*`
//! flags in [`sort`], `append_indent` in [`append`], and `old_sub` plus
//! `global_need_beginline` in [`subst`] -- the last two are the only ones
//! read from outside their own child.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::autocmd::apply_autocmds;
use crate::charset::{skip, vim_is_ident_char};
use crate::cstr;
use crate::eval::typval::{NumBuf, tv_list_find_str, tv_list_len};
use crate::eval::vars::get_vim_var_list;
use crate::ex_docmd::{cmdmod_has, do_exedit};
use crate::input::prompt_for_input;
use crate::main::{
    cmdmod, e_curdir, e_interr, e_invarg, e_noprevre, e_sandbox, g_do_tagpreview, got_int,
    msg_scroll, quit_more, sandbox, secure,
};
use crate::memory::xfree;
use crate::message::{
    emsg, message_filtered, msg, msg_clr_eos, msg_end, msg_outnum, msg_outtrans, msg_putchar,
    msg_puts, msg_start, msg_starthere, msgmore,
};
use crate::option::set_option_direct;
use crate::options::kOptFoldcolumn;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save;
use crate::os::input::os_breakcheck;
use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, skip_regexp};
use crate::types::AutoEvent;
use crate::types::CAR;
use crate::types::CmdIdx;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::{
    CmdModFlags, ExtmarkOp, NUL, OptVal, OptionSetFlags, String_0, UndoObjectType, Vv, bcount_t,
    bfa_values, bln_values, dobuf_action_values, exarg_T, getf_retvalues, linenr_T, list_T, lpos_T,
    size_t, uint8_t, win_T,
};
use crate::window::{win_enter, win_split};
use crate::winlayer::{Buf, Win, windows};
use core::ptr;

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

use crate::regexp::re_multiline;
pub const _ISalpha: ::core::ffi::c_uint = 1024;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const REGSUB_BACKSLASH: ::core::ffi::c_uint = 4;
pub const REGSUB_MAGIC: ::core::ffi::c_uint = 2;
pub const REGSUB_COPY: ::core::ffi::c_uint = 1;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const GETFILE_OPEN_OTHER: getf_retvalues = -1;
pub const GETFILE_SAME_FILE: getf_retvalues = 0;
pub const GETFILE_NOT_WRITTEN: getf_retvalues = 2;
pub const GETFILE_ERROR: getf_retvalues = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const BFA_KEEP_UNDO: bfa_values = 4;
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
    pub start: lpos_T,
    pub end: lpos_T,
    pub pre_match: linenr_T,
}
/// The matches an `'inccommand'` preview has to show, and how many lines they
/// need on screen -- which is what caps the scan when the preview window is
/// smaller than the range.
#[derive(Default)]
pub struct PreviewLines {
    pub subresults: Vec<SubResult>,
    pub lines_needed: linenr_T,
}
#[derive(Copy, Clone)]
pub struct subflags_T {
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
    pub start: lpos_T,
    pub end: lpos_T,
    pub matchcols: ::core::ffi::c_int,
    pub matchbytes: bcount_t,
    pub subcols: ::core::ffi::c_int,
    pub subbytes: bcount_t,
    pub lnum_before: linenr_T,
    pub lnum_after: linenr_T,
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
/// Fire `event` for `buf`: no file name, no pattern, no forcing -- the shape
/// every buffer-lifecycle autocommand in this family uses.
///
/// Safe: [`Buf`] is the live buffer `apply_autocmds` asks for, and the two
/// file names it also wants are null here.
pub(super) fn buf_autocmd(event: AutoEvent, buf: Buf) -> bool {
    // SAFETY: a live buffer and no file names.
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, buf.raw()) }
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
pub unsafe fn prepare_tagpreview(mut undo_sync: bool) -> bool {
    // SAFETY: every region below reads the live window list and the live
    // current window, or calls a window-layout function that does; both are
    // the editor's own and live from startup to exit.
    if cur_win().w_onebuf_opt.wo_pvw != 0 {
        return false;
    }
    for wp in windows() {
        if wp.w_onebuf_opt.wo_pvw != 0 {
            // SAFETY: a window of the editor's own list.
            unsafe { win_enter(wp.raw(), undo_sync) };
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
    cur_win().w_onebuf_opt.wo_pvw = 1;
    cur_win().w_onebuf_opt.wo_wfh = 1;
    cur_win().w_onebuf_opt.wo_scb = 0;
    cur_win().w_onebuf_opt.wo_crb = 0;
    cur_win().w_onebuf_opt.wo_diff = 0;
    set_option_direct(
        kOptFoldcolumn,
        OptVal::String(String_0::from_raw_parts(
            c"0".as_ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
        )),
        OptionSetFlags::NONE,
        SID_NONE,
    );
    true
}
/// Step over `:vimgrep`'s pattern argument: a bare word, or a pattern between
/// two delimiters followed by any of the `g`/`j`/`f` flags.
///
/// Answers where the argument ends, or NULL when the closing delimiter is
/// missing.  When `s` is given it receives the pattern's first byte and the
/// pattern is NUL-terminated in place; when `flags` is given the trailing
/// letters are OR-ed into it.
///
/// # Safety
/// `p` must be a NUL-terminated pattern, writable when `s` is given, and `s`
/// and `flags` must be live or NULL.
pub unsafe fn skip_vimgrep_pat(
    p: *mut ::core::ffi::c_char,
    s: *mut *mut ::core::ffi::c_char,
    flags: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    // SAFETY: caller's contract.
    let bytes = unsafe { cstr::bytes_at(p) };
    let first = bytes.first().copied().unwrap_or(NUL as uint8_t);
    // SAFETY: an ASCII byte widened, which is what the ctype table indexes.
    if unsafe { vim_is_ident_char(first as ::core::ffi::c_int) } {
        // A bare word, ending at the first white space.
        let mut at = skip::to_white(bytes);
        if !s.is_null() {
            // SAFETY: caller's contract.
            unsafe { *s = p };
            if at < bytes.len() {
                // Terminate the word in place and resume past the space.
                unsafe { *p.add(at) = NUL as ::core::ffi::c_char };
                at += 1;
            }
        }
        return p.wrapping_add(at);
    }

    // A delimited pattern.  The delimiter is whatever byte opened it.
    let delim = first as ::core::ffi::c_int;
    if !s.is_null() {
        // SAFETY: caller's contract; the pattern starts past the delimiter.
        unsafe { *s = p.add(1) };
    }
    // SAFETY: as above.
    let mut end = unsafe { skip_regexp(p.add(1), delim, 1) };
    // SAFETY: the skip stopped inside the pattern or at its NUL.
    if unsafe { *end } as ::core::ffi::c_int != delim {
        return ::core::ptr::null_mut();
    }
    if !s.is_null() {
        // SAFETY: caller's contract -- the closing delimiter is writable.
        unsafe { *end = NUL as ::core::ffi::c_char };
    }
    end = end.wrapping_add(1);

    // SAFETY: one past the closing delimiter is still inside the string.
    let tail = unsafe { cstr::bytes_at(end) };
    let mut n = 0;
    for &byte in tail {
        let flag = match byte {
            b'g' => VGR_GLOBAL,
            b'j' => VGR_NOJUMP,
            b'f' => VGR_FUZZY,
            _ => break,
        };
        if !flags.is_null() {
            // SAFETY: caller's contract.
            unsafe { *flags |= flag as ::core::ffi::c_int };
        }
        n += 1;
    }
    end.wrapping_add(n)
}
/// `:oldfiles` -- list `v:oldfiles`, numbered; under `:browse`, then ask for
/// a number and edit that file.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_oldfiles(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let eap = unsafe { &mut *eap };
    // SAFETY: `v:oldfiles` is the editor's own list, live or NULL.
    let list = unsafe { get_vim_var_list(Vv::Oldfiles) };
    if list.is_null() {
        msg(gettext(c"No old files"), 0);
        return;
    }

    say::start();
    msg_scroll.set(1);
    // SAFETY: a live list, whose items are its own.
    unsafe { list_oldfiles(list) };
    got_int.set(false);
    if !cmdmod_has(CmdModFlags::BROWSE) {
        return;
    }

    quit_more.set(false);
    // SAFETY: main thread; no prompt text and no "did the user cancel" flag.
    let nr = unsafe { prompt_for_input(ptr::null_mut(), 0, false, ptr::null_mut::<bool>()) };
    say::starthere();
    // SAFETY: `list` is still the editor's list.
    if nr <= 0 || nr > unsafe { tv_list_len(list) } {
        return;
    }
    let mut numbuf = NumBuf::new();
    // SAFETY: as above; `nr` is inside the list.
    let picked = unsafe { tv_list_find_str(list, nr - 1, &mut numbuf) };
    if picked.is_null() {
        return;
    }
    // SAFETY: `picked` is a live string, and the expansion is ours to free.
    let expanded = Owned(unsafe { expand_env_save(picked.cast_mut()) });
    eap.arg = expanded.0;
    eap.cmdidx = CmdIdx::edit;
    cmdmod.with_mut(|m| m.cmod_flags.clear(CmdModFlags::BROWSE));
    // SAFETY: the command block is the one borrowed here; the argument it
    // points at outlives the call.
    unsafe { do_exedit(&raw mut *eap, ptr::null_mut::<win_T>()) };
}

/// Number and print every entry of `list`, stopping on an interrupt.
///
/// # Safety
/// `list` must be a live list of strings.
unsafe fn list_oldfiles(list: *mut list_T) {
    let mut number = NumBuf::new();
    let mut text = NumBuf::new();
    // SAFETY: caller's contract.
    let mut item = unsafe { (*list).lv_first };
    let mut nr = 0;
    while !item.is_null() && !got_int.get() {
        nr += 1;
        // SAFETY: a live item of the list.
        let value = &raw mut unsafe { &mut *item }.li_tv;
        // SAFETY: `value` is that item's own.
        if !unsafe { message_filtered(number.string(value)) } {
            // SAFETY: main thread, message state; the text is the item's.
            unsafe { msg_outnum(nr) };
            say::puts(c": ");
            unsafe { msg_outtrans(text.string(value), 0, false) };
            say::clear_eos();
            say::putchar('\n' as ::core::ffi::c_int);
            os_breakcheck();
        }
        // SAFETY: as above.
        item = unsafe { (*item).li_next };
    }
}

/// An `xmalloc`ed C string that frees itself.
///
/// The family's one documented allocator seam: a C callee (`fix_fname`,
/// `fname_expand`, `makeswapname`, `make_filter_cmd`, `xstrdup` of an option
/// or of the previous replacement) answers an allocation that ends its life
/// with `xfree`, and upstream releases it at every `goto` out. A NULL is
/// allowed and frees nothing, which stands in for the `char *x = NULL; ...
/// xfree(x)` shape around a conditional allocation.
pub(crate) struct Owned(pub(crate) *mut ::core::ffi::c_char);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: the callee's allocation, handed over, or NULL.
        unsafe { xfree(self.0.cast()) };
    }
}

impl Owned {
    /// Hand the pointer on to a caller who owns it from here.
    pub(crate) fn release(self) -> *mut ::core::ffi::c_char {
        ::core::mem::ManuallyDrop::new(self).0
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
    ///
    /// # Safety
    /// `lnum` must be a line of the current buffer, and nothing may read
    /// another line of it while this runs.
    pub(crate) unsafe fn fill_line(&mut self, lnum: linenr_T) -> usize {
        // SAFETY: caller's contract.
        let mut lines = unsafe { crate::memline::Lines::current() };
        let text = lines.line(lnum);
        let len = text.len();
        self.0.clear();
        self.0.extend_from_slice(text);
        self.0.push(NUL as u8);
        len
    }

    /// The copy as the NUL-terminated string every C neighbour wants.
    pub(crate) fn as_ptr(&self) -> *mut ::core::ffi::c_char {
        self.0.as_ptr().cast::<::core::ffi::c_char>().cast_mut()
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
    use super::{msg_clr_eos, msg_end, msg_putchar, msg_puts, msg_start, msg_starthere, msgmore};
    use ::core::ffi::{CStr, c_int};

    /// [`msg_start`]: begin a message.
    pub(crate) fn start() {
        // SAFETY: the module's promise.
        unsafe { msg_start() }
    }

    /// [`msg_starthere`]: put the next message where the cursor is.
    pub(crate) fn starthere() {
        // SAFETY: as above.
        unsafe { msg_starthere() }
    }

    /// [`msg_end`]: finish a message, prompting if it did not fit.  False
    /// when `wait_return` was called.
    pub(crate) fn end() -> bool {
        // SAFETY: as above.
        unsafe { msg_end() }
    }

    /// [`msg_putchar`]: show one character.
    pub(crate) fn putchar(c: c_int) {
        // SAFETY: as above.
        unsafe { msg_putchar(c) }
    }

    /// [`msg_puts`]: show a string that carries its own NUL.
    pub(crate) fn puts(s: &CStr) {
        // SAFETY: as above, and a `CStr` is what the pointer form wants.
        unsafe { msg_puts(s.as_ptr()) }
    }

    /// [`msg_clr_eos`]: clear from the message position to the end.
    pub(crate) fn clear_eos() {
        // SAFETY: as above.
        unsafe { msg_clr_eos() }
    }

    /// [`msgmore`]: "N more lines" / "N fewer lines".
    pub(crate) fn more(n: c_int) {
        // SAFETY: as above.
        unsafe { msgmore(n) }
    }
}
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const DBL_MAX: ::core::ffi::c_double = __DBL_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const __DBL_MAX__: ::core::ffi::c_double = 1.7976931348623157e+308f64;

/// The window the editor is working in.
///
/// One copy for the whole family: every file here had its own, which is
/// seventeen `unsafe` lines saying the same thing.
pub(super) fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The buffer the editor is working in.
pub(super) fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
