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
use crate::charset::{skiptowhite, vim_is_ident_char};
use crate::eval::typval::{NumBuf, tv_list_find_str, tv_list_len};
use crate::eval::vars::get_vim_var_list;
use crate::ex_docmd::{cmdmod_has, do_exedit};
use crate::input::prompt_for_input;
use crate::main::{
    cmdmod, curtab, e_curdir, e_interr, e_invarg, e_invarg2, e_noprevre, e_sandbox, firstwin,
    g_do_tagpreview, got_int, msg_scroll, quit_more, sandbox, secure,
};
use crate::memory::xfree;
use crate::message::{
    emsg, message_filtered, msg, msg_clr_eos, msg_outnum, msg_outtrans, msg_putchar, msg_puts,
    msg_start, msg_starthere,
};
use crate::option::set_option_direct;
use crate::options::kOptFoldcolumn;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save;
use crate::os::input::os_breakcheck;
use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, skip_regexp};
use crate::types::{
    CMD_append, CMD_center, CMD_change, CMD_edit, CMD_left, CMD_right, CmdModFlags, ExtmarkOp,
    FAIL, NUL, OptVal, OptValData, OptValType, OptionSetFlags, String_0, UndoObjectType, Vv,
    bcount_t, bfa_values, bln_values, buf_T, dobuf_action_values, event_T, exarg_T, getf_retvalues,
    linenr_T, list_T, listitem_T, lpos_T, size_t, uint8_t, win_T,
};
use crate::window::{win_enter, win_split};
use crate::winlayer::Win;
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
pub const kOptValTypeString: OptValType = 2;
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
pub const STR2NR_FORCE: ::core::ffi::c_uint = 128;
pub const STR2NR_HEX: ::core::ffi::c_uint = 4;
pub const STR2NR_OCT: ::core::ffi::c_uint = 2;
pub const STR2NR_BIN: ::core::ffi::c_uint = 1;
pub const HIST_SEARCH: ::core::ffi::c_int = 1;
pub const VIM_QUESTION: ::core::ffi::c_uint = 4;
pub const VIM_YES: ::core::ffi::c_uint = 2;
pub const ECMD_NOWINENTER: ::core::ffi::c_uint = 64;
pub const ECMD_ALTBUF: ::core::ffi::c_uint = 32;
pub const ECMD_ADDBUF: ::core::ffi::c_uint = 16;
pub const ECMD_FORCEIT: ::core::ffi::c_uint = 8;
pub const ECMD_OLDBUF: ::core::ffi::c_uint = 4;
pub const ECMD_SET_HELP: ::core::ffi::c_uint = 2;
pub const ECMD_HIDE: ::core::ffi::c_uint = 1;
pub const ECMD_ONE: ::core::ffi::c_int = 1;
pub const ECMD_LAST: ::core::ffi::c_int = -1;
pub const ECMD_LASTL: ::core::ffi::c_int = 0;
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
#[derive(Copy, Clone)]
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
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
/// Fire `event` for `buf`: no file name, no pattern, no forcing -- the shape
/// every buffer-lifecycle autocommand in this family uses.
///
/// # Safety
/// `buf` must be a live buffer.
pub(super) unsafe fn buf_autocmd(event: event_T, buf: *mut buf_T) -> bool {
    // SAFETY: caller's contract.
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, buf) }
}
/// Refuse anything that reaches outside the editor while 'secure' is on or a
/// sandbox is open -- shell commands, `:write`, `:cd` and friends.
///
/// Returns true and gives a message when the command must not run.
///
/// # Safety
/// Main thread, message state.
pub unsafe fn check_secure() -> bool {
    if secure.get() != 0 {
        secure.set(2);
        // SAFETY: a live message string.
        unsafe { emsg(gettext(&raw const e_curdir as *const ::core::ffi::c_char)) };
        return true;
    }
    if sandbox.get() != 0 {
        // SAFETY: as above.
        unsafe { emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char)) };
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
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        unsafe { (*curtab.get()).tp_firstwin }
    };
    while !wp.is_null() {
        if unsafe { (*wp).w_onebuf_opt.wo_pvw } != 0 {
            unsafe { win_enter(wp, undo_sync) };
            return false;
        }
        wp = unsafe { (*wp).w_next };
    }
    if win_split(
        if g_do_tagpreview.get() > 0 as ::core::ffi::c_int {
            g_do_tagpreview.get()
        } else {
            0 as ::core::ffi::c_int
        },
        0 as ::core::ffi::c_int,
    ) == FAIL
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
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0::from_raw_parts(
                    c"0".as_ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
                ),
            },
        },
        OptionSetFlags::NONE,
        SID_NONE,
    );
    true
}
pub unsafe fn skip_vimgrep_pat(
    mut p: *mut ::core::ffi::c_char,
    mut s: *mut *mut ::core::ffi::c_char,
    mut flags: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    // SAFETY: caller's contract -- `p` walks a NUL-terminated pattern and
    // every step below follows a byte just read and found non-NUL; `s` and
    // `flags` are written only after a null check.
    if unsafe { vim_is_ident_char(*p as uint8_t as ::core::ffi::c_int) } {
        if !s.is_null() {
            unsafe { *s = p };
        }
        p = unsafe { skiptowhite(p) };
        if !s.is_null() && unsafe { *p } as ::core::ffi::c_int != NUL {
            unsafe { *p = NUL as ::core::ffi::c_char };
            p = unsafe { p.offset(1) };
        }
    } else {
        if !s.is_null() {
            unsafe { *s = p.offset(1 as ::core::ffi::c_int as isize) };
        }
        let mut c: ::core::ffi::c_int = unsafe { *p } as uint8_t as ::core::ffi::c_int;
        p = unsafe { skip_regexp(p.offset(1 as ::core::ffi::c_int as isize), c, 1) };
        if unsafe { *p } as ::core::ffi::c_int != c {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !s.is_null() {
            unsafe { *p = NUL as ::core::ffi::c_char };
        }
        p = unsafe { p.offset(1) };
        while unsafe { *p } as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
            || unsafe { *p } as ::core::ffi::c_int == 'j' as ::core::ffi::c_int
            || unsafe { *p } as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
        {
            if !flags.is_null() {
                if unsafe { *p } as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
                    unsafe { *flags |= VGR_GLOBAL as ::core::ffi::c_int };
                } else if unsafe { *p } as ::core::ffi::c_int == 'j' as ::core::ffi::c_int {
                    unsafe { *flags |= VGR_NOJUMP as ::core::ffi::c_int };
                } else {
                    unsafe { *flags |= VGR_FUZZY as ::core::ffi::c_int };
                }
            }
            p = unsafe { p.offset(1) };
        }
    }
    p
}
pub unsafe fn ex_oldfiles(mut eap: *mut exarg_T) {
    // SAFETY: caller's contract -- `eap` is the live Ex-command argument;
    // the rest is the message layer and `v:oldfiles`, which the editor owns.
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut l: *mut list_T = unsafe { get_vim_var_list(Vv::Oldfiles) };
    let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if l.is_null() {
        unsafe { msg(gettext(c"No old files".as_ptr()), 0 as ::core::ffi::c_int) };
        return;
    }
    unsafe { msg_start() };
    msg_scroll.set(1);
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = unsafe { (*l_).lv_first };
        while !li.is_null() {
            if got_int.get() {
                break;
            }
            nr += 1;
            let mut fname: *const ::core::ffi::c_char =
                unsafe { numbuf.string(&raw mut (*li).li_tv) };
            if !unsafe { message_filtered(fname) } {
                unsafe { msg_outnum(nr) };
                unsafe { msg_puts(c": ".as_ptr()) };
                unsafe {
                    msg_outtrans(
                        numbuf2.string(&raw mut (*li).li_tv),
                        0 as ::core::ffi::c_int,
                        false,
                    )
                };
                unsafe { msg_clr_eos() };
                unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
                os_breakcheck();
            }
            li = unsafe { (*li).li_next };
        }
    }
    got_int.set(false);
    if cmdmod_has(CmdModFlags::BROWSE) {
        quit_more.set(false);
        nr = unsafe {
            prompt_for_input(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                false,
                ::core::ptr::null_mut::<bool>(),
            )
        };
        unsafe { msg_starthere() };
        if nr > 0 as ::core::ffi::c_int && nr <= unsafe { tv_list_len(l) } {
            let p: *const ::core::ffi::c_char =
                unsafe { tv_list_find_str(l, nr - 1 as ::core::ffi::c_int, &mut numbuf2) };
            if p.is_null() {
                return;
            }
            let s: *mut ::core::ffi::c_char =
                unsafe { expand_env_save(p as *mut ::core::ffi::c_char) };
            unsafe { (*eap).arg = s };
            unsafe { (*eap).cmdidx = CMD_edit };
            cmdmod.with_mut(|m| m.cmod_flags.clear(CmdModFlags::BROWSE));
            unsafe { do_exedit(eap, ::core::ptr::null_mut::<win_T>()) };
            unsafe { xfree(s as *mut ::core::ffi::c_void) };
        }
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
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
