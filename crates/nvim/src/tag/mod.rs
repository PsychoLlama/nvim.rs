#![forbid(unsafe_code)]

pub(crate) mod state;
use crate::ascii::{ascii_isspace, ascii_iswhite};
use crate::autocmd::has_autocmd;
use crate::buffer::{buf_is_help, buflist_getfile, find_buf};
use crate::charset::is_blank_line;
use crate::cmdexpand::expand_one;
use crate::cursor::check_cursor;
use crate::drawscreen::{UPD_VALID, redraw_later};
use crate::eval::typval::{
    callback_copy, callback_free, dict_find, dict_get_number, dict_get_string_alloc, tv_clear,
    tv_dict_alloc, tv_dict_alloc_lock, tv_get_number, tv_list_alloc,
};
use crate::eval::vars::set_vim_var_string;
use crate::eval::{callback_call, list2fpos, set_ref_in_callback};
use crate::ex_cmds::{getfile, prepare_tagpreview};
use crate::ex_docmd::{do_cmdline_cmd, set_no_hlsearch};
use crate::file_search::{FindContext, split_stop_directories};
use crate::fold::fold_open_cursor;
use crate::getchar::state::{KeyTyped, got_int};
use crate::global_cell::GlobalCell;
use crate::guard::secure;
use crate::help::help_heuristic;
use crate::input::prompt_for_input;
use crate::insexpand::{ins_compl_check_keys, ins_compl_interrupted};
use crate::mark::{fm_getname, mark_view_make, mark_view_restore, setpcmark};
use crate::memory::XString;
use crate::message::state::{msg_col, msg_didout, msg_scroll, msg_scrolled, msg_silent};
use crate::message::{e_invarg, e_listreq};
use crate::message::{
    emsg, give_warning, msg, msg_advance, msg_clr_eos, msg_delay, msg_display, msg_display_bytes,
    msg_display_char, msg_ext_set_kind, msg_putchar, msg_start, msg_str, msg_str_hl, msg_title,
    verbose_enter, verbose_leave, wait_return,
};
use crate::r#move::{set_topline, validate_cursor};
use crate::option::vars::{
    P_IC, P_SCS, P_WS, fdo_flags, jop_flags, p_enc, p_ic, p_scs, p_sft, p_tbs, p_tgst, p_tl, p_tr,
    p_verbose, p_ws, swb_flags, tc_flags,
};
use crate::option::{callback_from_option, install_callback, magic_isset};
use crate::options::{
    kOptFdoFlagTag, kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagUseopen, kOptSwbFlagUsetab,
    kOptSwbFlagVsplit,
};
use crate::os::cshim::gettext;
use crate::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::path::{full_name_save, has_wildcard, path_full_compare, vim_is_abs_name};
use crate::pos::MAXLNUM;
use crate::quickfix::set_errorlist;
use crate::regexp::vim_regexec;
use crate::search::state::{magic_overruled, no_hlsearch};
use crate::state::MODE_INSERT;
use crate::state::mode::State;
use crate::tag::state::{
    g_do_tagpreview, g_tag_at_cursor, keep_help_flag, postponed_split, postponed_split_flags,
};
use crate::types::AutoEvent;
use crate::types::TAB;
use crate::types::ui::kUIMessages;
use crate::types::{
    AdditionalData, Callback, ColNr, Dict, ExArg, Expand, FileMark, FileMarkView, FileOffset,
    GetFileFlags, GetFileRet, LineNr, List, OptMagic, OptSet, Pos, Taggy, Timestamp, VarNumber,
    VimConv,
};
use crate::ui::state::Columns;
use crate::ui::ui_has;
use crate::window::{
    check_can_set_curbuf_forceit, swbuf_goto_win_with_buf, win_close, win_enter, win_split,
    win_valid,
};

// The carve of the transpiled module; see each child's docs.
mod scan;
pub(crate) use self::scan::*;
mod parse;
pub(crate) use self::parse::*;
mod collect;
mod tagfile;
pub(crate) use self::tagfile::*;
mod jump;
pub(crate) use self::jump::*;
mod list;
pub(crate) use self::list::*;
mod query;
pub use self::query::*;

mod tagfunc;
pub use self::tagfunc::*;
mod stack;
pub use self::stack::*;
mod command;
pub use self::command::*;
pub const OPTION_MAGIC_OFF: OptMagic = 2;
pub const GETF_SETMARK: GetFileFlags = 1;
pub const GETFILE_OPEN_OTHER: GetFileRet = -1;
pub const GETFILE_SAME_FILE: GetFileRet = 0;
pub const FINDFILE_FILE: ::core::ffi::c_uint = 0;
pub const LSIZE: ::core::ffi::c_uint = 512;
pub const DT_LTAG: ::core::ffi::c_uint = 11;
pub const DT_JUMP: ::core::ffi::c_uint = 9;
pub const DT_HELP: ::core::ffi::c_uint = 8;
pub const DT_SELECT: ::core::ffi::c_uint = 7;
pub const DT_FIRST: ::core::ffi::c_uint = 5;
pub const DT_PREV: ::core::ffi::c_uint = 4;
pub const DT_NEXT: ::core::ffi::c_uint = 3;
pub const DT_POP: ::core::ffi::c_uint = 2;
pub const DT_TAG: ::core::ffi::c_uint = 1;
pub const TAG_MANY: ::core::ffi::c_uint = 300;
pub const TAG_NO_TAGFUNC: ::core::ffi::c_uint = 256;
pub const TAG_KEEP_LANG: ::core::ffi::c_uint = 128;
pub const TAG_INS_COMP: ::core::ffi::c_uint = 64;
pub const TAG_VERBOSE: ::core::ffi::c_uint = 32;
pub const TAG_NOIC: ::core::ffi::c_uint = 8;
pub const TAG_REGEXP: ::core::ffi::c_uint = 4;
pub const TAG_NAMES: ::core::ffi::c_uint = 2;
pub const TAG_HELP: ::core::ffi::c_uint = 1;
/// Where one field of a line starts and ends, as byte offsets into it.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    /// The field's bytes in `line`, the line it was taken from.
    pub(crate) fn of<'a>(&self, line: &'a [u8]) -> &'a [u8] {
        &line[self.start..self.end]
    }

    pub(crate) fn len(&self) -> usize {
        self.end - self.start
    }
}

/// The parts of one tags-file line, as offsets into the line.
///
/// The name and the file name are bracketed; the command runs from
/// `command` to the end of the line, or to `command_end` when extension
/// fields follow it. The parts a line need not have — the kind, the user
/// data — are `None` when absent.
#[derive(Clone, Default)]
pub(crate) struct TagParts {
    pub(crate) tagname: Span,
    pub(crate) fname: Span,
    pub(crate) command: usize,
    pub(crate) command_end: Option<usize>,
    pub(crate) tagkind: Option<Span>,
    pub(crate) user_data: Option<Span>,
    pub(crate) tagline: LineNr,
}
pub const MT_IC_OFF: ::core::ffi::c_uint = 4;
pub const MT_MASK: ::core::ffi::c_uint = 7;
pub const MT_COUNT: ::core::ffi::c_uint = 16;
pub const MT_GL_OTH: ::core::ffi::c_uint = 2;
pub const MT_GL_CUR: ::core::ffi::c_uint = 1;
pub const MT_ST_OTH: ::core::ffi::c_uint = 3;
pub const MT_ST_CUR: ::core::ffi::c_uint = 0;
pub const MT_RE_OFF: ::core::ffi::c_uint = 8;
pub const CMDBUFFSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TAGSTACKSIZE: ::core::ffi::c_int = 20 as ::core::ffi::c_int;

/// What [`jumpto_tag`](crate::tag::jumpto_tag) did.
///
/// `NOTAGFILE` as a value rather than a status: the file the match names
/// does not exist, which is not a failure -- the caller goes on to the next
/// match, and the error message is only shown once every match has been
/// tried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Jumped {
    /// The editor is at the tag.
    Done,
    /// The file the match names does not exist. The C's `NOTAGFILE`.
    NoSuchFile,
}
/// The name of a file a match pointed at that does not exist, kept so that
/// the E429 report can name it. `None` is "every match's file was there".
static MISSING_FILE: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The tag the remembered matches were found for, so that a `:tnext` can
/// tell whether it is still walking the same search.
static MATCHES_ARE_FOR: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The preview window's stand-in for a tag stack: `:ptag` keeps one entry.
static PREVIEW_ENTRY: GlobalCell<Taggy> = GlobalCell::new(Taggy {
    tagname: None,
    fmark: FileMark {
        mark: Pos {
            lnum: 0 as LineNr,
            col: 0 as ColNr,
            coladd: 0 as ColNr,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: FileMarkView {
            topline_offset: MAXLNUM as LineNr,
            skipcol: 0 as ColNr,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    cur_match: 0 as ::core::ffi::c_int,
    cur_fnum: 0 as ::core::ffi::c_int,
    user_data: None,
});
pub const TAG_SEP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
/// Forget which tag the remembered matches are for.
pub fn tag_freematch() {
    drop(MATCHES_ARE_FOR.take());
}
