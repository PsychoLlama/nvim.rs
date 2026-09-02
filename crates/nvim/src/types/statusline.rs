#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// What a click on one statusline column does.
///
/// Not `Copy`: the `func` of a `kStlClickFuncRun` definition is an owned
/// string that `stl_clear_click_defs` frees.
#[derive(Clone)]
pub struct StlClickDefinition {
    pub type_0: StlClickDefinition_type_0,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type StlClickDefinition_type_0 = ::core::ffi::c_uint;
/// A click definition and the column it starts at.
///
/// Not `Copy`, following [`StlClickDefinition`].
#[derive(Clone)]
pub struct StlClickRecord {
    pub def: StlClickDefinition,
    pub start: *const ::core::ffi::c_char,
}
/// A letter of the statusline format language.
///
/// Upstream keeps the alphabet twice: forty-one `STL_*` constants whose
/// values *are* the letters, and `STL_ALL`, the string of every letter the
/// parser accepts. The enum is both -- the discriminant is the letter, so
/// [`StlOpt::from_byte`] is the acceptance test and the parser hands a
/// value the rest of the module can `match` by name.
///
/// The variants are ordered the way `STL_ALL` lists them.
#[derive(Copy, Clone, Eq, Debug)]
#[repr(u8)]
pub enum StlOpt {
    /// `%f`: the file name, as it was typed.
    FilePath = b'f',
    /// `%F`: the file name, in full.
    FullPath = b'F',
    /// `%t`: the file name's last component.
    FileName = b't',
    /// `%c`: the cursor's byte column.
    Column = b'c',
    /// `%v`: the cursor's screen column.
    VirtCol = b'v',
    /// `%V`: the screen column, shown only when it differs from the byte column.
    VirtColAlt = b'V',
    /// `%l`: the cursor's line number.
    Line = b'l',
    /// `%L`: how many lines the buffer has.
    NumLines = b'L',
    /// `%n`: the buffer number.
    BufNo = b'n',
    /// `%k`: the value of `'keymap'`.
    Keymap = b'k',
    /// `%o`: the cursor's byte offset in the buffer.
    Offset = b'o',
    /// `%O`: [`Offset`](Self::Offset) in hexadecimal.
    OffsetX = b'O',
    /// `%b`: the byte under the cursor.
    ByteVal = b'b',
    /// `%B`: [`ByteVal`](Self::ByteVal) in hexadecimal.
    ByteValX = b'B',
    /// `%r`: `[RO]` when the buffer is `'readonly'`.
    RoFlag = b'r',
    /// `%R`: [`RoFlag`](Self::RoFlag), abbreviated.
    RoFlagAlt = b'R',
    /// `%h`: `[Help]` in a help buffer.
    HelpFlag = b'h',
    /// `%H`: [`HelpFlag`](Self::HelpFlag), abbreviated.
    HelpFlagAlt = b'H',
    /// `%y`: `'filetype'`, in brackets.
    FileType = b'y',
    /// `%Y`: `'filetype'`, upper-cased and bare.
    FileTypeAlt = b'Y',
    /// `%w`: `[Preview]` in the preview window.
    PreviewFlag = b'w',
    /// `%W`: [`PreviewFlag`](Self::PreviewFlag), abbreviated.
    PreviewFlagAlt = b'W',
    /// `%m`: `[+]` when the buffer is changed.
    Modified = b'm',
    /// `%M`: [`Modified`](Self::Modified) without the brackets.
    ModifiedAlt = b'M',
    /// `%q`: `[Quickfix List]` or `[Location List]`.
    Quickfix = b'q',
    /// `%p`: how far down the buffer the cursor is.
    Percentage = b'p',
    /// `%P`: how much of the buffer the window shows.
    AltPercent = b'P',
    /// `%a`: `(file N of M)` for the argument list.
    ArgListStat = b'a',
    /// `%N`: the page number, for `'printheader'`.
    PageNum = b'N',
    /// `%S`: the pending command, as `'showcmd'` draws it.
    ShowCmd = b'S',
    /// `%C`: the fold column.
    FoldCol = b'C',
    /// `%s`: the sign column.
    SignCol = b's',
    /// `%{expr}`: an expression, evaluated and printed.
    VimExpr = b'{',
    /// `%=`: where the leftover width is spread.
    Separate = b'=',
    /// `%<`: where to start cutting when the line is too long.
    TruncMark = b'<',
    /// `%N*`: switch to user highlight group N.
    UserHl = b'*',
    /// `%#Group#`: switch to a named highlight group.
    Highlight = b'#',
    /// `%$Group$`: combine with a named highlight group.
    HighlightComb = b'$',
    /// `%NT`: open the region that switches to tab page N.
    TabPageNr = b'T',
    /// `%NX`: open the region that closes tab page N.
    TabCloseNr = b'X',
    /// `%@Func@`: open the region a click runs `Func` for.
    ClickFunc = b'@',
}

impl StlOpt {
    /// The letter, for the two places that still compare bytes: the format
    /// walk before it knows it has an item, and the `%#..#` scan for the
    /// letter that closes it.
    pub const fn letter(self) -> u8 {
        self as u8
    }

    /// The item `byte` names, or `None` when the alphabet has no letter for
    /// it -- upstream's `vim_strchr(STL_ALL, byte)`, and its NUL case:
    /// `STL_ALL` is a C string, so a NUL never matches.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            b'f' => StlOpt::FilePath,
            b'F' => StlOpt::FullPath,
            b't' => StlOpt::FileName,
            b'c' => StlOpt::Column,
            b'v' => StlOpt::VirtCol,
            b'V' => StlOpt::VirtColAlt,
            b'l' => StlOpt::Line,
            b'L' => StlOpt::NumLines,
            b'n' => StlOpt::BufNo,
            b'k' => StlOpt::Keymap,
            b'o' => StlOpt::Offset,
            b'O' => StlOpt::OffsetX,
            b'b' => StlOpt::ByteVal,
            b'B' => StlOpt::ByteValX,
            b'r' => StlOpt::RoFlag,
            b'R' => StlOpt::RoFlagAlt,
            b'h' => StlOpt::HelpFlag,
            b'H' => StlOpt::HelpFlagAlt,
            b'y' => StlOpt::FileType,
            b'Y' => StlOpt::FileTypeAlt,
            b'w' => StlOpt::PreviewFlag,
            b'W' => StlOpt::PreviewFlagAlt,
            b'm' => StlOpt::Modified,
            b'M' => StlOpt::ModifiedAlt,
            b'q' => StlOpt::Quickfix,
            b'p' => StlOpt::Percentage,
            b'P' => StlOpt::AltPercent,
            b'a' => StlOpt::ArgListStat,
            b'N' => StlOpt::PageNum,
            b'S' => StlOpt::ShowCmd,
            b'C' => StlOpt::FoldCol,
            b's' => StlOpt::SignCol,
            b'{' => StlOpt::VimExpr,
            b'=' => StlOpt::Separate,
            b'<' => StlOpt::TruncMark,
            b'*' => StlOpt::UserHl,
            b'#' => StlOpt::Highlight,
            b'$' => StlOpt::HighlightComb,
            b'T' => StlOpt::TabPageNr,
            b'X' => StlOpt::TabCloseNr,
            b'@' => StlOpt::ClickFunc,
            _ => return None,
        })
    }
}

/// Hand-written rather than derived: a derived `eq` is an ordinary call at
/// `-O0`, and the statusline is rebuilt on every redraw -- once per drawn
/// line under `'statuscolumn'`.
impl PartialEq for StlOpt {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}
#[derive(Copy, Clone, Default)]
pub struct statuscol_T {
    pub width: ::core::ffi::c_int,
    pub lnum: linenr_T,
    pub sign_cul_id: ::core::ffi::c_int,
    pub draw: bool,
    pub hlrec: *mut stl_hlrec_t,
    pub foldinfo: foldinfo_T,
    pub fold_vcol: [colnr_T; 9],
    pub sattrs: *mut SignTextAttrs,
}
/// One highlight run of a built statusline.
///
/// `Copy`: `start` points into the built string, which the caller owns.
#[derive(Copy, Clone)]
pub struct stl_hlrec {
    pub start: *mut ::core::ffi::c_char,
    pub userhl: ::core::ffi::c_int,
    /// The item the run came from, for the two drawers that tell the sign
    /// and fold columns' runs apart. `None` for a run that is not one of
    /// those -- upstream's zero, which is not a letter.
    pub item: Option<StlOpt>,
}
pub type stl_hlrec_t = stl_hlrec;
