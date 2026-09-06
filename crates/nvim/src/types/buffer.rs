#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// c2rust's names for `Buffer`'s anonymous members.
#![allow(non_camel_case_types)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::buffer::BufFlags;
use crate::registry::{IdMap, IdSet};
use crate::syntax::{SynCluster, SynPat};

/// Namespace id to the highest extmark id handed out in it: `Buffer`'s
/// `b_extmark_ns`, which upstream declared `Map(uint32_t, uint32_t)[1]` so
/// that it decayed to a pointer.
pub(crate) type ExtmarkNs = IdMap<uint32_t, uint32_t>;
use crate::r#move::WinValid;
use crate::undo::store::UndoStore;
use crate::winlayer::{BufId, TabId, WinId};

pub type AlignTextPos = ::core::ffi::c_uint;
pub type BorderTextType = ::core::ffi::c_uint;
/// One `nvim_buf_attach` subscription's Lua callbacks.
///
/// Not `Copy`. Each `LuaRef` is a reference into the Lua registry that
/// `free_update_callbacks` releases; a second copy would be a second release.
#[derive(Clone)]
pub struct BufUpdateCallbacks {
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: bool,
    pub preview: bool,
}
pub type FloatAnchor = ::core::ffi::c_int;
/// `WinConfig::anchor` is a corner, spelled as two independent bits: the
/// north-west corner is neither of them.
pub const kFloatAnchorEast: FloatAnchor = 1;
pub const kFloatAnchorSouth: FloatAnchor = 2;
pub type FloatRelative = ::core::ffi::c_uint;
/// What `WinConfig::row`/`col` are measured from.
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
/// Not `Copy`: the two virtual-text chunk arrays are owned, and
/// [`crate::window::config::merge_win_config`] decides which of two configs
/// keeps them by comparing the `items` pointers. A `clone` is the shallow
/// copy that comparison expects; it duplicates no allocation.
#[derive(Clone)]
pub struct WinConfig {
    pub window: WindowHandle,
    pub bufpos: LPos,
    pub height: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub row: ::core::ffi::c_double,
    pub col: ::core::ffi::c_double,
    pub anchor: FloatAnchor,
    pub relative: FloatRelative,
    pub external: bool,
    pub focusable: bool,
    pub mouse: bool,
    pub split: WinSplit,
    pub zindex: ::core::ffi::c_int,
    pub style: WinStyle,
    pub border: bool,
    pub shadow: bool,
    pub border_chars: [[::core::ffi::c_char; 32]; 8],
    pub border_hl_ids: [::core::ffi::c_int; 8],
    pub border_attr: [::core::ffi::c_int; 8],
    pub title: bool,
    pub title_pos: AlignTextPos,
    pub title_chunks: VirtText,
    pub title_width: ::core::ffi::c_int,
    pub footer: bool,
    pub footer_pos: AlignTextPos,
    pub footer_chunks: VirtText,
    pub footer_width: ::core::ffi::c_int,
    pub noautocmd: bool,
    pub fixed: bool,
    pub hide: bool,
    pub _cmdline_offset: ::core::ffi::c_int,
}
pub type WinSplit = ::core::ffi::c_uint;
pub type WinStyle = ::core::ffi::c_uint;
pub type BfaFlags = ::core::ffi::c_uint;
pub type BlnFlags = ::core::ffi::c_uint;
/// `Copy`, and not an owner: the three fields together are a *weak* name for
/// a buffer -- the address it had, the number it had, and the free count that
/// says whether the address still means that buffer. Duplicating one
/// duplicates no claim on anything.
#[derive(Copy, Clone)]
pub struct BufferRef {
    pub br_buf: *mut Buffer,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
}
/// Not `Copy` and not `Clone`: a block is a node of the tab page's list and
/// `df_changes` is an array it allocates, so a by-value duplicate would name
/// a `ga_data` and a `df_next` it does not own. Code that wants a block's
/// ranges past the block's lifetime copies the two `LineNr` arrays.
pub struct DiffBlock {
    pub df_next: *mut DiffBlock,
    pub df_lnum: [LineNr; 8],
    pub df_count: [LineNr; 8],
    pub is_linematched: bool,
    pub has_changes: bool,
    /// The block's inline changes, cached by `diff_find_change_inline_diff`
    /// and windowed per line by `diff_find_change`.
    pub df_changes: Vec<DiffLineChange>,
}

impl DiffBlock {
    /// A block with no ranges and no cached changes -- `xcalloc`'s answer,
    /// which is what every caller wanted from it.
    pub fn new(df_next: *mut DiffBlock) -> Self {
        Self {
            df_next,
            df_lnum: [0; 8],
            df_count: [0; 8],
            is_linematched: false,
            has_changes: false,
            df_changes: Vec::new(),
        }
    }
}
#[derive(Default)]
pub struct DiffLine {
    pub changes: *mut DiffLineChange,
    pub num_changes: ::core::ffi::c_int,
    pub bufidx: ::core::ffi::c_int,
    pub lineoff: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
pub struct DiffLineChange {
    pub dc_start: [ColNr; 8],
    pub dc_end: [ColNr; 8],
    pub dc_start_lnum_off: [::core::ffi::c_int; 8],
    pub dc_end_lnum_off: [::core::ffi::c_int; 8],
}
pub type DispTick = uint64_t;
pub type DoBufAction = ::core::ffi::c_uint;
pub type DoBufStart = ::core::ffi::c_uint;
pub struct FcsChars {
    pub stl: ScreenChar,
    pub stlnc: ScreenChar,
    pub wbr: ScreenChar,
    pub horiz: ScreenChar,
    pub horizup: ScreenChar,
    pub horizdown: ScreenChar,
    pub vert: ScreenChar,
    pub vertleft: ScreenChar,
    pub vertright: ScreenChar,
    pub verthoriz: ScreenChar,
    pub fold: ScreenChar,
    pub foldopen: ScreenChar,
    pub foldclosed: ScreenChar,
    pub foldsep: ScreenChar,
    pub foldinner: ScreenChar,
    pub diff: ScreenChar,
    pub msgsep: ScreenChar,
    pub eob: ScreenChar,
    pub lastline: ScreenChar,
    pub trunc: ScreenChar,
    pub truncrl: ScreenChar,
}
/// One `:loadkeymap` entry: the two sides of a buffer-local language
/// mapping, each without its terminator. Owned by the buffer's `b_kmap_ga`.
pub(crate) struct KeymapEntry {
    pub(crate) from: Vec<u8>,
    pub(crate) to: Vec<u8>,
}

/// Neither `Copy` nor `Clone`, and now the *owner* of what hangs off it.
/// The registry holds a buffer as an `allocator::Owned<Buffer>`, so this
/// struct is dropped rather than `xfree`d and a field with a destructor
/// works: the buffer-local user commands are a `Vec`. Every one of the
/// seventy raw pointers below is either a borrowed edge into the graph or
/// an allocation this buffer releases in `free_buffer`, and duplicating one
/// would make a second owner of all of them.
pub struct Buffer {
    pub handle: Handle,
    pub b_ml: MemLine,
    /// The buffer list, `firstbuf`..`lastbuf`. A handle rather than an
    /// address: the registry resolves it, so a link can never outlive what
    /// it names, and the buffer stays movable. `winlayer::Buf::next`/`prev`
    /// and `winlayer::buffers`/`buffers_back` are how it is walked.
    pub(crate) b_next: Option<BufId>,
    pub(crate) b_prev: Option<BufId>,
    pub b_nwindows: ::core::ffi::c_int,
    pub b_flags: BufFlags,
    pub b_locked: ::core::ffi::c_int,
    pub b_locked_split: ::core::ffi::c_int,
    pub b_ro_locked: ::core::ffi::c_int,
    pub b_ffname: *mut ::core::ffi::c_char,
    pub b_sfname: *mut ::core::ffi::c_char,
    pub b_fname: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub b_changed: ::core::ffi::c_int,
    pub b_changed_invalid: bool,
    pub changedtick_di: ChangedtickDictItem,
    pub b_last_changedtick: VarNumber,
    pub b_last_changedtick_i: VarNumber,
    pub b_last_changedtick_pum: VarNumber,
    pub b_saving: bool,
    pub b_mod_set: bool,
    pub b_mod_top: LineNr,
    pub b_mod_bot: LineNr,
    pub b_mod_xlines: LineNr,
    pub b_wininfo: file_buffer_b_wininfo,
    pub b_mod_tick_syn: DispTick,
    pub b_mod_tick_decor: DispTick,
    pub b_mtime: int64_t,
    pub b_mtime_ns: int64_t,
    pub b_mtime_read: int64_t,
    pub b_mtime_read_ns: int64_t,
    pub b_orig_size: uint64_t,
    pub b_orig_mode: ::core::ffi::c_int,
    pub b_last_used: time_t,
    pub b_namedm: [FileMark; 26],
    pub b_visual: VisualInfo,
    pub b_visual_mode_eval: ::core::ffi::c_int,
    pub b_last_cursor: FileMark,
    pub b_last_insert: FileMark,
    pub b_last_change: FileMark,
    pub b_changelist: [FileMark; 100],
    pub b_changelistlen: ::core::ffi::c_int,
    pub b_new_change: bool,
    pub b_chartab: [uint64_t; 4],
    pub b_maphash: [*mut MapBlock; 256],
    pub b_first_abbr: *mut MapBlock,
    /// The buffer-local user commands, sorted by name. A `-buffer` command
    /// shadows a global one; `usercmd`'s `Table` is the walk over both.
    pub b_ucmds: Vec<UserCmd>,
    pub b_op_start: Pos,
    pub b_op_start_orig: Pos,
    pub b_op_end: Pos,
    pub b_marks_read: bool,
    pub b_modified_was_set: bool,
    pub b_did_filetype: bool,
    pub b_keep_filetype: bool,
    pub b_au_did_filetype: bool,
    /// Owns every header the three links below (and the tree they hang off)
    /// name; NULL until the buffer's first undoable change. See
    /// [`crate::undo::store`].
    pub b_u_store: *mut UndoStore,
    pub b_u_oldhead: UndoLink,
    pub b_u_newhead: UndoLink,
    pub b_u_curhead: UndoLink,
    pub b_u_numhead: ::core::ffi::c_int,
    pub b_u_synced: bool,
    pub b_u_seq_last: ::core::ffi::c_int,
    pub b_u_save_nr_last: ::core::ffi::c_int,
    pub b_u_seq_cur: ::core::ffi::c_int,
    pub b_u_time_cur: time_t,
    pub b_u_save_nr_cur: ::core::ffi::c_int,
    pub b_u_line_ptr: *mut ::core::ffi::c_char,
    pub b_u_line_lnum: LineNr,
    pub b_u_line_colnr: ColNr,
    pub b_scanned: bool,
    pub b_p_iminsert: OptInt,
    pub b_p_imsearch: OptInt,
    pub b_kmap_state: int16_t,
    pub(crate) b_kmap_ga: Vec<KeymapEntry>,
    pub b_p_initialized: bool,
    pub b_p_script_ctx: [ScriptCtx; 92],
    pub b_p_ac: ::core::ffi::c_int,
    pub b_p_ai: ::core::ffi::c_int,
    pub b_p_ai_nopaste: ::core::ffi::c_int,
    pub b_p_bkc: *mut ::core::ffi::c_char,
    pub b_bkc_flags: ::core::ffi::c_uint,
    pub b_p_ci: ::core::ffi::c_int,
    pub b_p_bin: ::core::ffi::c_int,
    pub b_p_bomb: ::core::ffi::c_int,
    pub b_p_bh: *mut ::core::ffi::c_char,
    pub b_p_bt: *mut ::core::ffi::c_char,
    pub b_p_busy: OptInt,
    pub b_has_qf_entry: ::core::ffi::c_int,
    pub b_p_bl: ::core::ffi::c_int,
    pub b_p_channel: OptInt,
    pub b_p_cin: ::core::ffi::c_int,
    pub b_p_cino: *mut ::core::ffi::c_char,
    pub b_p_cink: *mut ::core::ffi::c_char,
    pub b_p_cinw: *mut ::core::ffi::c_char,
    pub b_p_cinsd: *mut ::core::ffi::c_char,
    pub b_p_com: *mut ::core::ffi::c_char,
    pub b_p_cms: *mut ::core::ffi::c_char,
    pub b_p_cot: *mut ::core::ffi::c_char,
    pub b_cot_flags: ::core::ffi::c_uint,
    pub b_p_cpt: *mut ::core::ffi::c_char,
    pub b_p_cpt_cb: *mut Callback,
    pub b_p_cpt_count: ::core::ffi::c_int,
    pub b_p_cfu: *mut ::core::ffi::c_char,
    pub b_cfu_cb: Callback,
    pub b_p_ofu: *mut ::core::ffi::c_char,
    pub b_ofu_cb: Callback,
    pub b_p_tfu: *mut ::core::ffi::c_char,
    pub b_tfu_cb: Callback,
    pub b_p_ffu: *mut ::core::ffi::c_char,
    pub b_ffu_cb: Callback,
    pub b_p_eof: ::core::ffi::c_int,
    pub b_p_eol: ::core::ffi::c_int,
    pub b_p_fixeol: ::core::ffi::c_int,
    pub b_p_et: ::core::ffi::c_int,
    pub b_p_et_nobin: ::core::ffi::c_int,
    pub b_p_et_nopaste: ::core::ffi::c_int,
    pub b_p_fenc: *mut ::core::ffi::c_char,
    pub b_p_ff: *mut ::core::ffi::c_char,
    pub b_p_ft: *mut ::core::ffi::c_char,
    pub b_p_fo: *mut ::core::ffi::c_char,
    pub b_p_flp: *mut ::core::ffi::c_char,
    pub b_p_inf: ::core::ffi::c_int,
    pub b_p_isk: *mut ::core::ffi::c_char,
    pub b_p_def: *mut ::core::ffi::c_char,
    pub b_p_inc: *mut ::core::ffi::c_char,
    pub b_p_inex: *mut ::core::ffi::c_char,
    pub b_p_inex_flags: uint32_t,
    pub b_p_inde: *mut ::core::ffi::c_char,
    pub b_p_inde_flags: uint32_t,
    pub b_p_indk: *mut ::core::ffi::c_char,
    pub b_p_fp: *mut ::core::ffi::c_char,
    pub b_p_fex: *mut ::core::ffi::c_char,
    pub b_p_fex_flags: uint32_t,
    pub b_p_fs: ::core::ffi::c_int,
    pub b_p_kp: *mut ::core::ffi::c_char,
    pub b_p_lisp: ::core::ffi::c_int,
    pub b_p_lop: *mut ::core::ffi::c_char,
    pub b_p_menc: *mut ::core::ffi::c_char,
    pub b_p_mps: *mut ::core::ffi::c_char,
    pub b_p_ml: ::core::ffi::c_int,
    pub b_p_ml_nobin: ::core::ffi::c_int,
    pub b_p_ma: ::core::ffi::c_int,
    pub b_p_nf: *mut ::core::ffi::c_char,
    pub b_p_pi: ::core::ffi::c_int,
    pub b_p_qe: *mut ::core::ffi::c_char,
    pub b_p_ro: ::core::ffi::c_int,
    pub b_p_sw: OptInt,
    pub b_p_scbk: OptInt,
    pub b_p_si: ::core::ffi::c_int,
    pub b_p_sts: OptInt,
    pub b_p_sts_nopaste: OptInt,
    pub b_p_sua: *mut ::core::ffi::c_char,
    pub b_p_swf: ::core::ffi::c_int,
    pub b_p_smc: OptInt,
    pub b_p_syn: *mut ::core::ffi::c_char,
    pub b_p_ts: OptInt,
    pub b_p_tw: OptInt,
    pub b_p_tw_nobin: OptInt,
    pub b_p_tw_nopaste: OptInt,
    pub b_p_wm: OptInt,
    pub b_p_wm_nobin: OptInt,
    pub b_p_wm_nopaste: OptInt,
    pub b_p_vsts: *mut ::core::ffi::c_char,
    pub b_p_vsts_array: *mut ColNr,
    pub b_p_vsts_nopaste: *mut ::core::ffi::c_char,
    pub b_p_vts: *mut ::core::ffi::c_char,
    pub b_p_vts_array: *mut ColNr,
    pub b_p_keymap: *mut ::core::ffi::c_char,
    pub b_p_gefm: *mut ::core::ffi::c_char,
    pub b_p_gp: *mut ::core::ffi::c_char,
    pub b_p_mp: *mut ::core::ffi::c_char,
    pub b_p_efm: *mut ::core::ffi::c_char,
    pub b_p_ep: *mut ::core::ffi::c_char,
    pub b_p_path: *mut ::core::ffi::c_char,
    pub b_p_ar: ::core::ffi::c_int,
    pub b_p_tags: *mut ::core::ffi::c_char,
    pub b_p_tc: *mut ::core::ffi::c_char,
    pub b_tc_flags: ::core::ffi::c_uint,
    pub b_p_dict: *mut ::core::ffi::c_char,
    pub b_p_dia: *mut ::core::ffi::c_char,
    pub b_p_tsr: *mut ::core::ffi::c_char,
    pub b_p_tsrfu: *mut ::core::ffi::c_char,
    pub b_tsrfu_cb: Callback,
    pub b_p_ul: OptInt,
    pub b_p_udf: ::core::ffi::c_int,
    pub b_p_lw: *mut ::core::ffi::c_char,
    pub b_ind_level: ::core::ffi::c_int,
    pub b_ind_open_imag: ::core::ffi::c_int,
    pub b_ind_no_brace: ::core::ffi::c_int,
    pub b_ind_first_open: ::core::ffi::c_int,
    pub b_ind_open_extra: ::core::ffi::c_int,
    pub b_ind_close_extra: ::core::ffi::c_int,
    pub b_ind_open_left_imag: ::core::ffi::c_int,
    pub b_ind_jump_label: ::core::ffi::c_int,
    pub b_ind_case: ::core::ffi::c_int,
    pub b_ind_case_code: ::core::ffi::c_int,
    pub b_ind_case_break: ::core::ffi::c_int,
    pub b_ind_param: ::core::ffi::c_int,
    pub b_ind_func_type: ::core::ffi::c_int,
    pub b_ind_comment: ::core::ffi::c_int,
    pub b_ind_in_comment: ::core::ffi::c_int,
    pub b_ind_in_comment2: ::core::ffi::c_int,
    pub b_ind_cpp_baseclass: ::core::ffi::c_int,
    pub b_ind_continuation: ::core::ffi::c_int,
    pub b_ind_unclosed: ::core::ffi::c_int,
    pub b_ind_unclosed2: ::core::ffi::c_int,
    pub b_ind_unclosed_noignore: ::core::ffi::c_int,
    pub b_ind_unclosed_wrapped: ::core::ffi::c_int,
    pub b_ind_unclosed_whiteok: ::core::ffi::c_int,
    pub b_ind_matching_paren: ::core::ffi::c_int,
    pub b_ind_paren_prev: ::core::ffi::c_int,
    pub b_ind_maxparen: ::core::ffi::c_int,
    pub b_ind_maxcomment: ::core::ffi::c_int,
    pub b_ind_scopedecl: ::core::ffi::c_int,
    pub b_ind_scopedecl_code: ::core::ffi::c_int,
    pub b_ind_java: ::core::ffi::c_int,
    pub b_ind_js: ::core::ffi::c_int,
    pub b_ind_keep_case_label: ::core::ffi::c_int,
    pub b_ind_hash_comment: ::core::ffi::c_int,
    pub b_ind_cpp_namespace: ::core::ffi::c_int,
    pub b_ind_if_for_while: ::core::ffi::c_int,
    pub b_ind_cpp_extern_c: ::core::ffi::c_int,
    pub b_ind_pragma: ::core::ffi::c_int,
    pub b_no_eol_lnum: LineNr,
    pub b_start_eof: ::core::ffi::c_int,
    pub b_start_eol: ::core::ffi::c_int,
    pub b_start_ffc: ::core::ffi::c_int,
    pub b_start_fenc: *mut ::core::ffi::c_char,
    pub b_bad_char: ::core::ffi::c_int,
    pub b_start_bomb: ::core::ffi::c_int,
    pub b_bufvar: ScopeDictDictItem,
    pub b_vars: *mut Dict,
    pub b_may_swap: bool,
    pub b_did_warn: bool,
    pub b_help: bool,
    pub b_spell: bool,
    pub b_prompt_text: *mut ::core::ffi::c_char,
    pub b_prompt_callback: Callback,
    pub b_prompt_interrupt: Callback,
    pub b_prompt_append_new_line: bool,
    pub b_prompt_insert: ::core::ffi::c_int,
    pub b_prompt_start: FileMark,
    pub b_s: SynBlock,
    pub b_signcols: file_buffer_b_signcols,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    /// Where this buffer's extmarks live. Upstream declared it `MarkTree[1]`
    /// so that the name decayed to the pointer every `marktree.h` entry
    /// point wants; here it is the tree itself.
    pub b_marktree: MarkTree,
    /// Namespace id to the highest extmark id handed out in it. Asked and
    /// counted up; never walked.
    pub(crate) b_extmark_ns: ExtmarkNs,
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: file_buffer_update_channels,
    pub update_callbacks: file_buffer_update_callbacks,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
pub struct file_buffer_b_signcols {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
pub struct file_buffer_b_wininfo {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut WinInfo,
}
pub struct file_buffer_update_callbacks {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
pub struct file_buffer_update_channels {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
/// Neither `Copy` nor `Clone`. A frame is a *node* of the window layout
/// tree: its `fr_parent`/`fr_next`/`fr_prev`/`fr_child` links and its
/// `fr_win` back-edge are all owned by the tree's shape, and duplicating one
/// would put a second node into a structure the editor walks by pointer.
/// Frames are allocated one at a time and freed with the window; nothing in
/// the tree may copy one, and the absence of the derives is what says so.
pub struct Frame {
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    pub fr_parent: *mut Frame,
    pub fr_next: *mut Frame,
    pub fr_prev: *mut Frame,
    pub fr_child: *mut Frame,
    /// The window a leaf frame holds; null for a row or a column. Still an
    /// address, not a handle: `window::arith`'s unit tests build frame trees
    /// over `Window`s that were never registered. `winlayer::window_at` is
    /// how a saved tree's copy is compared safely.
    pub fr_win: *mut Window,
}
pub type GetFileRet = ::core::ffi::c_int;
pub type GetFileFlags = ::core::ffi::c_uint;
/// Not `Copy`: `multispace` and `leadmultispace` are owned runs, allocated
/// by 'listchars' and freed when the window's value is replaced.
#[derive(Clone)]
pub struct LcsChars {
    pub eol: ScreenChar,
    pub ext: ScreenChar,
    pub prec: ScreenChar,
    pub nbsp: ScreenChar,
    pub space: ScreenChar,
    pub tab1: ScreenChar,
    pub tab2: ScreenChar,
    pub tab3: ScreenChar,
    pub leadtab1: ScreenChar,
    pub leadtab2: ScreenChar,
    pub leadtab3: ScreenChar,
    pub lead: ScreenChar,
    pub trail: ScreenChar,
    pub multispace: *mut ScreenChar,
    pub leadmultispace: *mut ScreenChar,
    pub conceal: ScreenChar,
}
pub struct LLPos {
    pub lnum: LineNr,
    pub col: ColNr,
    pub len: ::core::ffi::c_int,
}
#[derive(Clone)]
pub struct MatchState {
    pub rm: RegMMatch,
    pub buf: *mut Buffer,
    pub lnum: LineNr,
    pub attr: ::core::ffi::c_int,
    pub attr_cur: ::core::ffi::c_int,
    pub first_lnum: LineNr,
    pub startcol: ColNr,
    pub endcol: ColNr,
    pub is_addpos: bool,
    pub has_cursor: bool,
    pub tm: ProfTime,
}
#[derive(Clone)]
pub struct MatchItem {
    pub mit_next: *mut MatchItem,
    pub mit_id: ::core::ffi::c_int,
    pub mit_priority: ::core::ffi::c_int,
    pub mit_pattern: *mut ::core::ffi::c_char,
    pub mit_match: RegMMatch,
    pub mit_pos_array: *mut LLPos,
    pub mit_pos_count: ::core::ffi::c_int,
    pub mit_pos_cur: ::core::ffi::c_int,
    pub mit_toplnum: LineNr,
    pub mit_botlnum: LineNr,
    pub mit_hl: MatchState,
    pub mit_hlg_id: ::core::ffi::c_int,
    pub mit_conceal_char: ::core::ffi::c_int,
}
pub struct PosSave {
    pub w_topline_save: ::core::ffi::c_int,
    pub w_topline_corr: ::core::ffi::c_int,
    pub w_cursor_save: Pos,
    pub w_cursor_corr: Pos,
}
#[derive(Copy, Clone)]
pub struct SynTime {
    pub total: ProfTime,
    pub slowest: ProfTime,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
}
pub struct SynBlock {
    pub b_keywtab: HashTab,
    pub b_keywtab_ic: HashTab,
    pub b_syn_error: bool,
    pub b_syn_slow: bool,
    pub b_syn_ic: ::core::ffi::c_int,
    pub b_syn_foldlevel: ::core::ffi::c_int,
    pub b_syn_spell: ::core::ffi::c_int,
    /// The block's `:syntax match`/`region` patterns, in definition order.
    /// A region is a run of consecutive entries: its START(s), an optional
    /// SKIP, then its END(s).
    pub(crate) b_syn_patterns: Vec<SynPat>,
    /// The block's `:syntax cluster`s. The index *is* the id, less
    /// `SYNID_CLUSTER`, so a cluster is emptied rather than removed.
    pub(crate) b_syn_clusters: Vec<SynCluster>,
    pub b_spell_cluster_id: ::core::ffi::c_int,
    pub b_nospell_cluster_id: ::core::ffi::c_int,
    pub b_syn_containedin: ::core::ffi::c_int,
    pub b_syn_sync_flags: ::core::ffi::c_int,
    pub b_syn_sync_id: int16_t,
    pub b_syn_sync_minlines: LineNr,
    pub b_syn_sync_maxlines: LineNr,
    pub b_syn_sync_linebreaks: LineNr,
    /// `:syntax sync linecont`'s pattern, owned; `b_syn_linecont_prog`
    /// is what it compiled to.
    pub(crate) b_syn_linecont_pat: Option<::std::ffi::CString>,
    /// OWNERSHIP -- **carve-out**, the same one as `SynPat::sp_prog`: a
    /// compiled program is a `regexp/` object with its own allocator
    /// discipline (`vim_regcomp` / `vim_regfree`), so it stays a raw pointer,
    /// released by `syntax_clear`.
    pub b_syn_linecont_prog: *mut RegProg,
    pub b_syn_linecont_time: SynTime,
    pub b_syn_linecont_ic: ::core::ffi::c_int,
    pub b_syn_topgrp: ::core::ffi::c_int,
    pub b_syn_conceal: ::core::ffi::c_int,
    pub b_syn_folditems: ::core::ffi::c_int,
    /// OWNERSHIP -- **carve-out**. The parser's state cache is one slab of
    /// `b_sst_len` `SynState`s, threaded into two intrusive singly-linked
    /// lists that point *into* it: the used entries (`b_sst_first`, sorted by
    /// line) and the recycled ones (`b_sst_firstfree`). A `Vec` cannot hold
    /// it -- growing one moves the entries, and every `sst_next` in both
    /// lists, plus whatever `*mut SynState` a caller is holding across a
    /// re-parse, would dangle. Resizing is a copy-and-rethread
    /// (`syn_stack_alloc`) and the slab is released by
    /// `syn_stack_free_block`, the only `xfree` of it. Retiring it means
    /// making the two lists indices into the slab -- a rewrite of the cache,
    /// not of its ownership.
    pub b_sst_array: *mut SynState,
    pub b_sst_len: ::core::ffi::c_int,
    /// The used entries, lowest line first. Points into [`Self::b_sst_array`].
    pub b_sst_first: *mut SynState,
    /// The recycled entries. Points into [`Self::b_sst_array`].
    pub b_sst_firstfree: *mut SynState,
    pub b_sst_freecount: ::core::ffi::c_int,
    pub b_sst_check_lnum: LineNr,
    pub b_sst_lasttick: DispTick,
    pub b_langp: GArray,
    pub b_spell_ismw: [bool; 256],
    pub b_spell_ismw_mb: *mut ::core::ffi::c_char,
    pub b_p_spc: *mut ::core::ffi::c_char,
    pub b_cap_prog: *mut RegProg,
    pub b_p_spf: *mut ::core::ffi::c_char,
    pub b_p_spl: *mut ::core::ffi::c_char,
    pub b_p_spo: *mut ::core::ffi::c_char,
    pub b_p_spo_flags: ::core::ffi::c_uint,
    pub b_cjk: ::core::ffi::c_int,
    pub b_syn_chartab: [uint8_t; 32],
    pub b_syn_isk: *mut ::core::ffi::c_char,
}
/// Deliberately neither `Copy` nor `Clone`: a tab page owns three
/// allocations (`tp_vars`, `tp_localdir`, `tp_diffbuf`'s memlines) and a
/// window list, so a bitwise copy would be a second owner of all of them.
/// Tab pages are reached through `winlayer::TabPage`, which is the `Copy`
/// handle naming this one.
pub struct Tabpage {
    pub handle: Handle,
    /// The tab page list off `first_tabpage`. A handle, as the buffer
    /// list's links are — `winlayer::TabPage::next` and `winlayer::tabs`
    /// are how it is walked.
    pub(crate) tp_next: Option<TabId>,
    pub tp_topframe: *mut Frame,
    /// The window this tab page was last working in, and the one before it.
    /// Handles, as its window list's ends are: both are read *after* a call
    /// that can close a window. **Stale while the tab page is current**,
    /// exactly as `tp_firstwin` is.
    pub(crate) tp_curwin: Option<WinId>,
    pub(crate) tp_prevwin: Option<WinId>,
    /// This tab page's window list, its two ends. Handles, as the links
    /// between them are. **Stale while the tab page is the current one** —
    /// the `firstwin`/`lastwin` globals are then the truth, which is what
    /// `winlayer::windows_in_tab` encodes.
    ///
    pub(crate) tp_firstwin: Option<WinId>,
    pub(crate) tp_lastwin: Option<WinId>,
    pub tp_old_rows_avail: int64_t,
    pub tp_old_columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut DiffBlock,
    pub tp_diffbuf: [*mut Buffer; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub tp_snapshot: [*mut Frame; 3],
    pub tp_winvar: ScopeDictDictItem,
    pub tp_vars: *mut Dict,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
/// Not `Copy`: `tagname` and `user_data` are owned strings.
#[derive(Clone)]
pub struct Taggy {
    pub tagname: *mut ::core::ffi::c_char,
    pub fmark: FileMark,
    pub cur_match: ::core::ffi::c_int,
    pub cur_fnum: ::core::ffi::c_int,
    pub user_data: *mut ::core::ffi::c_char,
}
/// Neither `Copy` nor `Clone`. A window owns its buffer list entry, its
/// grid, its option strings, its tag stack and its jump list; nothing in
/// the tree may duplicate one, and the absence of the derives is what says
/// so.
pub struct Window {
    pub handle: Handle,
    pub w_buffer: *mut Buffer,
    pub w_s: *mut SynBlock,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    /// The namespaces this window shows, when they are window-local.
    /// Membership only.
    pub(crate) w_ns_set: IdSet<uint32_t>,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    /// Whether the window's highlight namespace has to be re-resolved
    /// before it is next drawn.
    pub w_hl_needs_update: bool,
    /// This tab page's window list. Handles rather than addresses, as the
    /// buffer and tab page lists' links are: `winlayer::Win::next`/`prev`
    /// and `winlayer::windows`/`windows_back` are how they are walked.
    ///
    /// A window is registered from `win_alloc` to `win_free`, so a link can
    /// always be resolved -- with one hole the tree keeps: the autocommand
    /// window is *unregistered while idle*, and `aucmd_prepbuf` therefore
    /// puts it back in the registry before `win_append` files it here.
    pub(crate) w_prev: Option<WinId>,
    pub(crate) w_next: Option<WinId>,
    pub w_locked: bool,
    pub w_frame: *mut Frame,
    pub w_cursor: Pos,
    pub w_curswant: ColNr,
    /// Whether the next cursor move should recompute `w_curswant` — the
    /// column a vertical move aims for — rather than keep the one the last
    /// horizontal move set.
    pub w_set_curswant: bool,
    pub w_cursorline: LineNr,
    pub w_last_cursorline: LineNr,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: LineNr,
    pub w_old_cursor_fcol: ColNr,
    pub w_old_cursor_lcol: ColNr,
    pub w_old_visual_lnum: LineNr,
    pub w_old_visual_col: ColNr,
    pub w_old_curswant: ColNr,
    pub w_last_cursor_lnum_rnu: LineNr,
    pub w_p_lcs_chars: LcsChars,
    pub w_p_fcs_chars: FcsChars,
    pub w_topline: LineNr,
    /// Whether `w_topline` was set on purpose rather than left at its
    /// default, which decides whether entering the buffer may move it.
    pub w_topline_was_set: bool,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: ColNr,
    pub w_skipcol: ColNr,
    pub w_last_topline: LineNr,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: ColNr,
    pub w_last_skipcol: ColNr,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: PosSave,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: WinValid,
    pub w_valid_cursor: Pos,
    pub w_valid_leftcol: ColNr,
    pub w_valid_skipcol: ColNr,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: LineNr,
    pub w_viewport_last_botline: LineNr,
    pub w_viewport_last_topfill: LineNr,
    pub w_viewport_last_skipcol: LineNr,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: ColNr,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: LineNr,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut WLine,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: GArray,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: LineNr,
    pub w_redraw_bot: LineNr,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: DispTick,
    pub w_stl_cursor: Pos,
    pub w_stl_virtcol: ColNr,
    pub w_stl_topline: LineNr,
    pub w_stl_line_count: LineNr,
    pub w_stl_topfill: ::core::ffi::c_int,
    /// Whether the last line drawn in the window was empty, as the status
    /// line last saw it.
    pub w_stl_empty: bool,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: Pos,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut ArgList,
    pub w_arg_idx: ::core::ffi::c_int,
    /// Whether `w_arg_idx` no longer names the argument the window shows.
    pub w_arg_idx_invalid: bool,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: WinOpt,
    pub w_allbuf_opt: WinOpt,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictDictItem,
    pub w_vars: *mut Dict,
    pub w_pcmark: Pos,
    pub w_prev_pcmark: Pos,
    pub w_jumplist: [XFileMark; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut MatchItem,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [Taggy; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: LineNr,
    pub w_statuscol_line_count: LineNr,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut QfInfo,
    pub w_llist_ref: *mut QfInfo,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
#[derive(Clone)]
pub struct WinInfo {
    pub wi_win: *mut Window,
    pub wi_mark: FileMark,
    pub wi_optset: bool,
    pub wi_opt: WinOpt,
    pub wi_fold_manual: bool,
    pub wi_folds: GArray,
    pub wi_changelistidx: ::core::ffi::c_int,
}
/// Not `Copy`: the string options in here are owned, and `copy_options`
/// exists precisely to duplicate them. A shallow copy is a step in that,
/// never the whole of it.
#[derive(Clone)]
pub struct WinOpt {
    pub wo_arab: ::core::ffi::c_int,
    pub wo_bri: ::core::ffi::c_int,
    pub wo_briopt: *mut ::core::ffi::c_char,
    pub wo_diff: ::core::ffi::c_int,
    pub wo_fdc: *mut ::core::ffi::c_char,
    pub wo_eiw: *mut ::core::ffi::c_char,
    pub wo_fdc_save: *mut ::core::ffi::c_char,
    pub wo_fen: ::core::ffi::c_int,
    pub wo_fen_save: ::core::ffi::c_int,
    pub wo_fdi: *mut ::core::ffi::c_char,
    pub wo_fdl: OptInt,
    pub wo_fdl_save: OptInt,
    pub wo_fdm: *mut ::core::ffi::c_char,
    pub wo_fdm_save: *mut ::core::ffi::c_char,
    pub wo_fml: OptInt,
    pub wo_fdn: OptInt,
    pub wo_fde: *mut ::core::ffi::c_char,
    pub wo_fdt: *mut ::core::ffi::c_char,
    pub wo_fmr: *mut ::core::ffi::c_char,
    pub wo_lbr: ::core::ffi::c_int,
    pub wo_list: ::core::ffi::c_int,
    pub wo_nu: ::core::ffi::c_int,
    pub wo_rnu: ::core::ffi::c_int,
    pub wo_ve: *mut ::core::ffi::c_char,
    pub wo_ve_flags: ::core::ffi::c_uint,
    pub wo_nuw: OptInt,
    pub wo_wfb: ::core::ffi::c_int,
    pub wo_wfh: ::core::ffi::c_int,
    pub wo_wfw: ::core::ffi::c_int,
    pub wo_pvw: ::core::ffi::c_int,
    pub wo_lhi: OptInt,
    pub wo_rl: ::core::ffi::c_int,
    pub wo_rlc: *mut ::core::ffi::c_char,
    pub wo_scr: OptInt,
    pub wo_sms: ::core::ffi::c_int,
    pub wo_spell: ::core::ffi::c_int,
    pub wo_cuc: ::core::ffi::c_int,
    pub wo_cul: ::core::ffi::c_int,
    pub wo_culopt: *mut ::core::ffi::c_char,
    pub wo_cc: *mut ::core::ffi::c_char,
    pub wo_sbr: *mut ::core::ffi::c_char,
    pub wo_stc: *mut ::core::ffi::c_char,
    pub wo_stl: *mut ::core::ffi::c_char,
    pub wo_wbr: *mut ::core::ffi::c_char,
    pub wo_scb: ::core::ffi::c_int,
    pub wo_diff_saved: ::core::ffi::c_int,
    pub wo_scb_save: ::core::ffi::c_int,
    pub wo_wrap: ::core::ffi::c_int,
    pub wo_wrap_save: ::core::ffi::c_int,
    pub wo_cocu: *mut ::core::ffi::c_char,
    pub wo_cole: OptInt,
    pub wo_crb: ::core::ffi::c_int,
    pub wo_crb_save: ::core::ffi::c_int,
    pub wo_scl: *mut ::core::ffi::c_char,
    pub wo_siso: OptInt,
    pub wo_so: OptInt,
    pub wo_winhl: *mut ::core::ffi::c_char,
    pub wo_lcs: *mut ::core::ffi::c_char,
    pub wo_fcs: *mut ::core::ffi::c_char,
    pub wo_winbl: OptInt,
    pub wo_wrap_flags: uint32_t,
    pub wo_stl_flags: uint32_t,
    pub wo_wbr_flags: uint32_t,
    pub wo_fde_flags: uint32_t,
    pub wo_fdt_flags: uint32_t,
    pub wo_script_ctx: [ScriptCtx; 51],
}
#[derive(Copy, Clone)]
pub struct WLine {
    pub wl_lnum: LineNr,
    pub wl_size: uint16_t,
    pub wl_valid: bool,
    pub wl_folded: bool,
    pub wl_foldend: LineNr,
    pub wl_lastlnum: LineNr,
}

impl BufferRef {
    /// The "names nothing" state. `BufRef::NONE` is how the editor spells
    /// it; the two remaining raw holders (`main::au_new_curbuf` and
    /// `AcoSave::new_curbuf`) start from this.
    ///
    /// A `const fn` as well as a [`Default`] because two of them are
    /// statics.
    pub const fn new() -> Self {
        BufferRef {
            br_buf: ::core::ptr::null_mut(),
            br_fnum: 0,
            br_buf_free_count: 0,
        }
    }
}

impl Default for BufferRef {
    fn default() -> Self {
        Self::new()
    }
}
