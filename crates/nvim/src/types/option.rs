#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::types::Failed;

use crate::global_cell::GlobalCell;
use crate::option::OptSlot;
use crate::options::vars::{BoolOpt, NumOpt, StrOpt};
use crate::winlayer::{Buf, Win};

crate::flag_set! {
    /// `stl_syntax`: which of `'title'` and `'icon'` hold a *statusline
    /// format* rather than plain text, and so have to be re-evaluated
    /// whenever the window contents change.
    pub struct StlSyntax;

    const ICON = 1;
    const TITLE = 2;
}

crate::flag_set! {
    /// `option.h`'s `OptionSetFlags`: which scope an option-setting call
    /// means, plus the handful of behaviour switches that ride along with it.
    ///
    /// [`NONE`](Self::NONE) — neither scope named — is not "no scope": it is
    /// upstream's "both", the `:set` (as opposed to `:setlocal`/`:setglobal`)
    /// spelling, which sets the local value and the global one together.
    pub struct OptionSetFlags;

    /// Use the global value.
    const GLOBAL = 0x01;
    /// Use the local value.
    const LOCAL = 0x02;
    /// The option came from a modeline.
    const MODELINE = 0x04;
    /// Only set window-local options.
    const WINONLY = 0x08;
    /// Do not set window-local options.
    const NOWIN = 0x10;
    /// List options one per line.
    const ONECOLUMN = 0x20;
    /// `"skiprtp"` in `'sessionoptions'`.
    const SKIPRTP = 0x80;
}

pub type OptScope = ::core::ffi::c_uint;
pub type OptScopeFlags = uint8_t;
/// An option's value, and which of the three kinds of option it belongs to.
///
/// Distinct from [`OptValType`], which is what the option *table* declares a
/// row to be: [`OptVal::kind`] is how the two are compared. A value and its
/// row agree everywhere except [`OptVal::Nil`], which no row declares.
#[derive(Copy, Clone)]
pub enum OptVal {
    /// A global-local option with no value in this scope, and what the API
    /// reports as nil. `kOptValTypeNil`.
    Nil,
    /// A boolean option's value: on, off, or **absent** — a global-local
    /// option with no value in this scope, which upstream spells as the
    /// third state of the option variable's `int` (-1) and reports to the
    /// API as nil. The variable keeps that alphabet; the value does not.
    /// `kOptValTypeBoolean`.
    Boolean(Option<bool>),
    /// `kOptValTypeNumber`.
    Number(OptInt),
    /// `kOptValTypeString`.
    String(OptStr),
}

/// The bytes of a string option's value: a pointer and a length, with
/// ownership carried by the option protocol rather than by the type.
///
/// **Deliberately not [`String_0`].** An API string owns its bytes; a string
/// option's value may be the generated table's `.rodata` default, the shared
/// empty string every unset string option points at, or an allocation the
/// option variable is about to take over -- which is why
/// [`optval_free`](crate::option::optval_free) exists and why this pair is
/// `Copy`. The option table is a `const` in `.rodata` and names its string
/// defaults there, so a value that allocated could not be one of its rows.
#[derive(Copy, Clone)]
pub struct OptStr {
    data: *mut ::core::ffi::c_char,
    size: size_t,
}

impl OptStr {
    /// A view of a `.rodata` string: the generated option table's defaults,
    /// which `alloc_options_default` replaces with owned copies at startup.
    /// Nothing may free one of these.
    pub const fn borrowed(text: &'static ::core::ffi::CStr) -> Self {
        Self {
            data: text.as_ptr().cast_mut(),
            size: text.count_bytes(),
        }
    }

    /// The pair, for the option variables and the C callees that hold a
    /// `char *` and a length separately.
    ///
    /// Building one is safe -- an `OptStr` releases nothing by itself. The
    /// obligation is on whoever passes the value it ends up in to
    /// [`optval_free`](crate::option::optval_free): `data` must by then be
    /// the shared empty string, a `.rodata` default, or one allocation with
    /// one owner.
    pub const fn from_raw_parts(data: *mut ::core::ffi::c_char, size: size_t) -> Self {
        Self { data, size }
    }

    /// The pointer. Never null: an unset string option holds the shared
    /// empty string rather than nothing.
    pub const fn data(&self) -> *mut ::core::ffi::c_char {
        self.data
    }

    /// The byte count, not counting the terminator every option value has.
    pub const fn len(&self) -> size_t {
        self.size
    }

    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl OptVal {
    /// A string value over a `.rodata` literal. See [`OptStr::borrowed`].
    pub const fn static_string(text: &'static ::core::ffi::CStr) -> Self {
        OptVal::String(OptStr::borrowed(text))
    }

    /// The type this value is, as the option table spells one.
    pub const fn kind(&self) -> OptValType {
        match self {
            OptVal::Nil => -1,
            OptVal::Boolean(_) => 0,
            OptVal::Number(_) => 1,
            OptVal::String(_) => 2,
        }
    }

    pub const fn is_nil(&self) -> bool {
        matches!(self, OptVal::Nil)
    }

    /// A boolean option's value, `None` for the unset global-local marker
    /// and for every other kind of option.
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            OptVal::Boolean(boolean) => *boolean,
            _ => None,
        }
    }

    /// The tri-state word upstream's union held: 0 false, 1 true, -1 not
    /// set in this scope. Vimscript still reads a boolean option's value as
    /// a number, which is what this is for.
    pub const fn tristate(&self) -> Option<::core::ffi::c_int> {
        match self {
            OptVal::Boolean(None) => Some(-1),
            OptVal::Boolean(Some(false)) => Some(0),
            OptVal::Boolean(Some(true)) => Some(1),
            _ => None,
        }
    }

    pub const fn as_number(&self) -> Option<OptInt> {
        match self {
            OptVal::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// The string pair, for a string value.
    pub const fn as_string(&self) -> Option<OptStr> {
        match self {
            OptVal::String(s) => Some(*s),
            _ => None,
        }
    }
}

pub type OptValType = ::core::ffi::c_int;

/// Why a `did_set_*` callback refused a value.
///
/// Upstream's callbacks answered a `char *` that was either one of the
/// message statics or a pointer into a buffer the *caller* had to supply
/// and keep alive -- `set_option` carried an `errbuf`/`errbuflen` pair down
/// four levels for that, and the frame held a third copy of it. A message
/// that names what it disliked owns its bytes here, so the buffer and both
/// parameters are gone and a callback's answer outlives its frame.
pub enum OptError {
    /// One of the message statics, already translated.
    Static(&'static ::core::ffi::CStr),
    /// A message the callback formatted, naming the value it disliked.
    Owned(crate::memory::XString),
}

impl OptError {
    /// The room a formatted rejection is given, which is the size upstream
    /// told `vim_snprintf` its `errbuf` was. Stated once, because the
    /// message owns its bytes and is measured at the terminator the
    /// formatter left rather than carried beside a length.
    pub const ROOM: usize = IOSIZE as usize;

    /// The message, for a caller that reports it.
    pub fn as_cstr(&self) -> &::core::ffi::CStr {
        match self {
            OptError::Static(message) => message,
            OptError::Owned(message) => message.as_cstr(),
        }
    }
}

impl From<&'static ::core::ffi::CStr> for OptError {
    fn from(message: &'static ::core::ffi::CStr) -> Self {
        OptError::Static(message)
    }
}

impl From<crate::memory::XString> for OptError {
    fn from(message: crate::memory::XString) -> Self {
        OptError::Owned(message)
    }
}

/// What the option table names for an option that has just been set: a
/// callback that vets the new value and reports why it does not like it.
pub type OptDidSetCb = Option<unsafe fn(&mut OptSet) -> Result<(), OptError>>;
pub type OptExpandCb = Option<
    unsafe fn(
        *mut OptExpand,
        *mut ::core::ffi::c_int,
        *mut *mut *mut ::core::ffi::c_char,
    ) -> Result<(), Failed>,
>;
pub struct OptExpand {
    // Upstream carries an `oe_varp` here, and its two readers -- the
    // 'listchars'/'fillchars' and 'eventignore'/'eventignorewin' expansions
    // -- used it to tell one of a callback's two options from the other by
    // comparing addresses. Both ask `oe_idx` instead, which is what the
    // option *is* rather than where it happens to live, so nothing reads a
    // variable here at all.
    pub oe_idx: OptIndex,
    pub oe_opt_value: *mut ::core::ffi::c_char,
    pub oe_append: bool,
    pub oe_include_orig_val: bool,
    pub oe_regmatch: *mut RegMatch,
    pub oe_xp: *mut Expand,
    pub oe_set_arg: *mut ::core::ffi::c_char,
}
pub struct OptSet {
    /// The option's storage in the scope being set, with the type its row
    /// declares. `pub(crate)` because [`OptSlot`] is: every `did_set_*`
    /// callback that reads one lives in this crate, and the frame is opaque
    /// to everything outside it.
    pub(crate) os_varp: OptSlot,
    pub os_idx: OptIndex,
    pub os_flags: OptionSetFlags,
    /// The values the set is moving between. The C carried the bare union
    /// and left every `did_set_*` callback to know which arm was live from
    /// the row the table put it in; these carry their own kind.
    pub os_oldval: OptVal,
    pub os_newval: OptVal,
    pub os_value_checked: bool,
    pub os_value_changed: bool,
    pub os_restore_chartab: bool,
    /// The window and buffer the set is happening in, as handles taken
    /// where `set_option` built the frame and both were provably live.
    /// Upstream carried two `void *` and left every `did_set_*` callback to
    /// rebuild a window or buffer from an address the frame had been holding
    /// across whatever the set already ran.
    pub os_win: Win,
    pub os_buf: Buf,
}
/// Where an option keeps its global value.
///
/// An option's variable is a tagged pointer — the same bytes are an `int`,
/// an `OptInt` or a `char *` depending on the row's `type_0` — and the
/// table used to state the tag once and the address once, with nothing
/// tying them together: `var` was a `*mut c_void` filled in from whichever
/// global the metadata named. Here the arm carries a *selector* naming one
/// field of [`crate::options::vars::Options`], so a row cannot point a
/// string option at a number and still compile, and the table holds no
/// address at all.
#[derive(Copy, Clone)]
pub enum OptVar {
    /// The option has no global variable: its value lives only in a window
    /// or a buffer.
    NoGlobal,
    /// A boolean option's field.
    Boolean(BoolOpt),
    /// A number option's field.
    Number(NumOpt),
    /// A string option's field.
    String(StrOpt),
    /// An immutable option has nowhere to keep a value, so it reads its own
    /// default in place — the `def_val.data` of its own row, whose active
    /// member is this option's type. Nothing writes through it: `set_option`
    /// refuses the option long before it gets that far.
    OwnDefault,
}

impl OptVar {
    /// Whether the option has a global variable at all — the question the
    /// null `var` used to answer. An immutable option counts: its own
    /// default stands in for one.
    pub fn has_global(self) -> bool {
        !matches!(self, OptVar::NoGlobal)
    }

    /// Whether `type_0` describes the bytes this points at. The table
    /// asserts it for every row at compile time, which is what lets the
    /// `varp` plumbing read a variable as its option's type without
    /// checking first.
    pub const fn agrees_with(self, type_0: OptValType) -> bool {
        match self {
            // Neither carries a variable of its own to disagree with.
            OptVar::NoGlobal | OptVar::OwnDefault => true,
            OptVar::Boolean(_) => type_0 == kOptValTypeBoolean,
            OptVar::Number(_) => type_0 == kOptValTypeNumber,
            OptVar::String(_) => type_0 == kOptValTypeString,
        }
    }
}

#[derive(Copy, Clone)]
pub struct VimOption {
    pub fullname: *mut ::core::ffi::c_char,
    pub shortname: *mut ::core::ffi::c_char,
    pub flags: uint32_t,
    pub type_0: OptValType,
    pub scope_flags: OptScopeFlags,
    pub var: OptVar,
    pub flags_var: Option<&'static GlobalCell<::core::ffi::c_uint>>,
    pub scope_idx: [ssize_t; 3],
    pub immutable: bool,
    /// The words a string option accepts, empty for one that accepts
    /// anything. Upstream kept the array and its length in two fields and
    /// terminated the array with a null pointer besides — three spellings of
    /// one fact, and the walk had to agree with all of them.
    pub values: &'static [&'static ::core::ffi::CStr],
    pub opt_did_set_cb: OptDidSetCb,
    pub opt_expand_cb: OptExpandCb,
    /// The default the table declares — the seed for
    /// `crate::option::state`'s copy, which is the one `:set opt&` reads
    /// and the one startup rewrites once it can expand the environment.
    pub def_val: OptVal,
}

crate::char_flags! {
    /// A letter of `'cpoptions'` — which Vi compatibilities are switched on.
    /// Ask with [`cpo_has`](crate::option::cpo_has).
    pub struct CpoFlag;

    /// `a`: `:read` sets the alternate file name.
    const ALTREAD = b'a';
    /// `A`: `:write` sets the alternate file name.
    const ALTWRITE = b'A';
    /// `b`: `\|` ends a mapping.
    const BAR = b'b';
    /// `B`: a backslash in a mapping is not special.
    const BSLASH = b'B';
    /// `c`: searching continues at the end of the match.
    const SEARCH = b'c';
    /// `C`: do not concatenate sourced lines.
    const CONCAT = b'C';
    /// `d`: `./tags` in `'tags'` means the current directory.
    const DOTTAG = b'd';
    /// `D`: no digraph after `r`, `f`, etc.
    const DIGRAPH = b'D';
    /// `e`: an executed register ending in a newline runs its last line.
    const EXECBUF = b'e';
    /// `E`: operating on an empty region is an error.
    const EMPTYREGION = b'E';
    /// `f`: `:read file` sets the file name when there is none.
    const FNAMER = b'f';
    /// `F`: `:write file` sets the file name when there is none.
    const FNAMEW = b'F';
    /// `i`: interrupting a read leaves the buffer modified.
    const INTMOD = b'i';
    /// `I`: remove auto-indent more often.
    const INDENT = b'I';
    /// `J`: two spaces are needed to detect the end of a sentence.
    const ENDOFSENT = b'J';
    /// `K`: do not wait for a key code in mappings.
    const KOFFSET = b'K';
    /// `l`: the character after a backslash in a collection is literal.
    const LITERAL = b'l';
    /// `L`: `'list'` changes the effective `'wrapmargin'`.
    const LISTWM = b'L';
    /// `m`: `'showmatch'` waits half a second even on more input.
    const SHOWMATCH = b'm';
    /// `M`: `%` ignores the use of backslashes.
    const MATCHBSL = b'M';
    /// `n`: the `'number'` column is also used for wrapped text.
    const NUMCOL = b'n';
    /// `o`: a search offset is not kept for the next search.
    const LINEOFF = b'o';
    /// `O`: silently overwrite a file that appeared since the buffer opened.
    const OVERNEW = b'O';
    /// `P`: `:write >>file` sets the file name when there is none.
    const FNAMEAPP = b'P';
    /// `q`: `3J` leaves the cursor after the first join.
    const JOINCOL = b'q';
    /// `r`: `:s` with no pattern redoes the last `:s`, not the last search.
    const REDO = b'r';
    /// `R`: filtering lines removes their marks.
    const REMMARK = b'R';
    /// `s`: buffer-local options are copied on first entry to the buffer.
    const BUFOPT = b's';
    /// `S`: buffer-local options are copied on every entry to the buffer.
    const BUFOPTGLOB = b'S';
    /// `t`: the tag pattern is remembered for `n`.
    const TAGPAT = b't';
    /// `u`: `u` undoes itself.
    const UNDO = b'u';
    /// `v`: backspacing in Replace keeps the deleted text on screen.
    const BACKSPACE = b'v';
    /// `W`: `:w!` does not overwrite a read-only file.
    const FWRITE = b'W';
    /// `x`: `<Esc>` on the command line executes it.
    const ESC = b'x';
    /// `X`: `R` with a count deletes the characters only once.
    const REPLCNT = b'X';
    /// `y`: a yank can be redone with `.`.
    const YANK = b'y';
    /// `Z`: `:w!` does not reset `'readonly'`.
    const KEEPRO = b'Z';
    /// `$`: a one-line change draws a `$` instead of redrawing.
    const DOLLAR = b'$';
    /// `!`: a repeated filter command does not reuse the last external one.
    const FILTER = b'!';
    /// `%`: `%` does not match inside unmatched preprocessor directives.
    const MATCH = b'%';
    /// `+`: `:write file` resets `'modified'`.
    const PLUS = b'+';
    /// `>`: appending to a register inserts a newline first.
    const REGAPPEND = b'>';
    /// `;`: `,` and `;` skip over the character they are already on.
    const SCOLON = b';';
    /// `~`: do not resolve symlinks when changing directory.
    const NOSYMLINKS = b'~';
    /// `_`: `cw` on a blank changes only that blank.
    const CHANGEW = b'_';
}

crate::char_flags! {
    /// A letter of `'shortmess'` — which messages are shortened or dropped.
    /// Ask with [`shortmess`](crate::option::shortmess), which also honours
    /// [`ABBREVIATIONS`](Self::ABBREVIATIONS).
    pub struct ShmFlag;

    /// `r`: "readonly".
    const RO = b'r';
    /// `m`: "modified".
    const MOD = b'm';
    /// `l`: "L" instead of "lines".
    const LINES = b'l';
    /// `w`: `[w]` instead of "written".
    const WRI = b'w';
    /// `a`: shorten all of [`RO`](Self::RO), [`MOD`](Self::MOD),
    /// [`LINES`](Self::LINES) and [`WRI`](Self::WRI), and nothing else.
    const ABBREVIATIONS = b'a';
    /// `W`: do not say "written" at all.
    const WRITE = b'W';
    /// `t`: truncate file messages.
    const TRUNC = b't';
    /// `T`: truncate all messages.
    const TRUNCALL = b'T';
    /// `o`: overwrite file messages.
    const OVER = b'o';
    /// `O`: overwrite more messages.
    const OVERALL = b'O';
    /// `s`: no "search hit BOTTOM" messages.
    const SEARCH = b's';
    /// `A`: no ATTENTION messages.
    const ATTENTION = b'A';
    /// `I`: no intro message.
    const INTRO = b'I';
    /// `c`: no completion menu messages.
    const COMPLETIONMENU = b'c';
    /// `C`: no completion scanning messages.
    const COMPLETIONSCAN = b'C';
    /// `q`: no "recording" message.
    const RECORDING = b'q';
    /// `F`: no file info messages.
    const FILEINFO = b'F';
    /// `S`: no search count, the `[1/10]` indicator.
    const SEARCHCOUNT = b'S';
}

crate::char_flags! {
    /// A letter of `'formatoptions'` — how automatic formatting behaves.
    /// Ask with [`has_format_option`](crate::textformat::has_format_option),
    /// which reads the *buffer's* value and answers no under `'paste'`.
    pub struct FoFlag;

    /// `t`: wrap text at `'textwidth'`.
    const WRAP = b't';
    /// `c`: wrap comments at `'textwidth'`, inserting the leader.
    const WRAP_COMS = b'c';
    /// `r`: insert the comment leader after hitting `<CR>`.
    const RET_COMS = b'r';
    /// `o`: insert the comment leader after `o` or `O`.
    const OPEN_COMS = b'o';
    /// `/`: with `o`, do not insert the leader for a trailing `//` comment.
    const NO_OPEN_COMS = b'/';
    /// `q`: `gq` formats comments too.
    const Q_COMS = b'q';
    /// `n`: recognise numbered lists when formatting.
    const Q_NUMBER = b'n';
    /// `2`: the second line of a paragraph gives the indent.
    const Q_SECOND = b'2';
    /// `v`: Vi-compatible wrapping — only on blanks typed this insert.
    const INS_VI = b'v';
    /// `l`: a line already longer than `'textwidth'` is not wrapped.
    const INS_LONG = b'l';
    /// `b`: wrap only on a blank at or before `'textwidth'`.
    const INS_BLANK = b'b';
    /// `m`: break before and after a multi-byte character.
    const MBYTE_BREAK = b'm';
    /// `M`: no space before or after a multi-byte character when joining.
    const MBYTE_JOIN = b'M';
    /// `B`: no space between two multi-byte characters when joining.
    const MBYTE_JOIN2 = b'B';
    /// `1`: do not break a line after a one-letter word.
    const ONE_LETTER = b'1';
    /// `w`: trailing white space continues the paragraph.
    const WHITE_PAR = b'w';
    /// `a`: reformat the paragraph on every change.
    const AUTO = b'a';
    /// `]`: respect `'textwidth'` rigorously.
    const RIGOROUS_TW = b']';
    /// `j`: remove comment leaders when joining lines.
    const REMOVE_COMS = b'j';
    /// `p`: do not break a single space after a period.
    const PERIOD_ABBR = b'p';
}

crate::char_flags! {
    /// A letter of `'backspace'` — what `<BS>` may erase in Insert mode.
    /// Ask with [`can_bs`](crate::option::can_bs).
    ///
    /// [`NOSTOP`](Self::NOSTOP) behaves exactly like [`START`](Self::START)
    /// except that it does not stop at the start of the insert point, so
    /// `can_bs(START)` deliberately answers yes for either letter.
    pub struct BsFlag;

    /// `i`: erase autoindent.
    const INDENT = b'i';
    /// `l`: erase past the start of the line.
    const EOL = b'l';
    /// `s`: erase past the start of the insert point.
    const START = b's';
    /// `p`: [`START`](Self::START), without stopping at the insert point.
    const NOSTOP = b'p';
}

/// The fixed value of `'maxcombine'`: the most composing characters that can
/// follow a base character.
pub const MAX_MCO: ::core::ffi::c_int = 6;

/// No value at all -- the option is not set here.
pub const kOptValTypeNil: OptValType = -1;

/// A `'boolean'` option's value.
pub const kOptValTypeBoolean: OptValType = 0;

/// A `'number'` option's value.
pub const kOptValTypeNumber: OptValType = 1;

/// A `'string'` option's value.
pub const kOptValTypeString: OptValType = 2;
