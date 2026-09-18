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
use crate::types::CmdIdx;

#[derive(Clone)]
pub struct CmdParseInfo {
    pub cmdmod: CmdMod,
    pub magic: CmdParseMagic,
}
#[derive(Copy, Clone)]
pub struct CmdParseMagic {
    pub file: bool,
    pub bar: bool,
}
/// A reader `do_cmdline` pulls its lines from, given a `(c, cookie, indent,
/// do_concat)` and returning allocated memory or null at the end of input.
///
/// Spelled separately from [`LineGetter`] so a caller can name the bare
/// function type -- comparing two readers means `ptr::fn_addr_eq` on it.
pub type LineGetterFn = unsafe fn(
    ::core::ffi::c_int,
    *mut ::core::ffi::c_void,
    ::core::ffi::c_int,
    bool,
) -> *mut ::core::ffi::c_char;
pub type LineGetter = Option<LineGetterFn>;
#[derive(Copy, Clone)]
pub struct SubReplacementString {
    pub sub: *mut ::core::ffi::c_char,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
/// What an Ex command's range counts -- upstream's `ADDR_*`, the value
/// `ExArg::addr_type`, `CommandDefinition::cmd_addr_type` and
/// `UserCmd::uc_addr_type` carry.
///
/// A range is `1,5` whatever it addresses; this is what those numbers *are*.
/// `:1,5delete` is line numbers, `:1,5bdelete` buffer numbers and `:1,5close`
/// window numbers, and the address parser reads `.`, `$` and `'m` differently
/// for each. c2rust gave the family a bare `c_uint`, so every one of the ~70
/// `match` sites over it needed a catch-all arm for values that cannot exist.
///
/// `#[repr(u32)]` with the upstream discriminants: `ExArg` and `UserCmd` are
/// `repr(C)`, and the discriminants are what `ex_cmds.lua` and the
/// `nvim_parse_cmd` API answer with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum CmdAddr {
    /// Buffer lines -- the default, and what `:1,5d` means.
    Lines = 0,
    /// Window numbers, as `:1,5close` counts them.
    Windows = 1,
    /// Argument-list indices.
    Arguments = 2,
    /// Buffer numbers, loaded buffers only.
    LoadedBuffers = 3,
    /// Buffer numbers, listed or not.
    Buffers = 4,
    /// Tab-page numbers.
    Tabs = 5,
    /// Tab pages counted from the current one (`:tabmove`).
    TabsRelative = 6,
    /// Quickfix entries, valid ones only.
    QuickfixValid = 7,
    /// Quickfix entry numbers.
    Quickfix = 8,
    /// A plain non-negative number the command reads itself.
    Unsigned = 9,
    /// Something only the command knows how to count.
    Other = 10,
    /// The command takes no range at all.
    NoRange = 11,
}

crate::flag_set! {
    /// What syntax an Ex command accepts -- upstream's `EX_*`, the bits
    /// `ExArg::argt` and the command table's `cmd_argt` carry. The whole
    /// of `:` is described by this one word: which of a range, a `!`, an
    /// argument, a register and a count the command takes, and where it is
    /// allowed to run.
    pub struct ExArgt;

    /// The command takes a range.
    const RANGE = 0x001;
    /// The command takes a `!` after its name.
    const BANG = 0x002;
    /// The command takes an argument.
    const EXTRA = 0x004;
    /// Expand `%`, `#` and the other wildcards in the argument.
    const XFILE = 0x008;
    /// The argument is one word: no spaces allowed.
    const NOSPC = 0x010;
    /// A missing range means the whole file, not the current line.
    const DFLALL = 0x020;
    /// Extend the range to whole closed folds.
    const WHOLEFOLD = 0x040;
    /// An argument is required.
    const NEEDARG = 0x080;
    /// A `|` ends the command, and so does a `"` comment.
    const TRLBAR = 0x100;
    /// A register name may follow the command.
    const REGSTR = 0x200;
    /// A count may follow the command.
    const COUNT = 0x400;
    /// A `"` does *not* start a comment: the command's own argument may
    /// contain one.
    const NOTRLCOM = 0x800;
    /// Line number zero is allowed in the range.
    const ZEROR = 0x1000;
    /// `CTRL-V` quotes the next character in the argument.
    const CTRLV = 0x2000;
    /// `++opt=arg` file options are copied into `eap->cmd`.
    const CMDARG = 0x4000;
    /// The argument names a buffer, for `:buffer`-style completion.
    const BUFNAME = 0x8000;
    /// The named buffer may be an unlisted one.
    const BUFUNL = 0x10000;
    /// `++opt=arg` file options are allowed.
    const ARGOPT = 0x20000;
    /// The command is allowed inside the `'sandbox'`.
    const SBOXOK = 0x40000;
    /// The command is allowed in the command-line window.
    const CMDWIN = 0x80000;
    /// The command changes the buffer, so `'modifiable'` and the text lock
    /// are checked first.
    const MODIFY = 0x100000;
    /// Trailing `l`, `#` or `p` flags are allowed.
    const FLAGS = 0x200000;
    /// The command is allowed when the buffer is locked against changes.
    const LOCK_OK = 0x1000000;
    /// A user command that keeps the script context of its definition.
    const KEEPSCRIPT = 0x4000000;
    /// The command has an `'inccommand'` preview implementation.
    const PREVIEW = 0x8000000;
}
crate::flag_set! {
    /// The `:silent`, `:noautocmd`, `:keepmarks` … command modifiers, as the
    /// bits [`CmdMod::cmod_flags`] carries.
    pub struct CmdModFlags;

    /// `:sandbox` -- the command runs with `sandbox` raised.
    const SANDBOX = 1;
    /// `:silent` -- do not echo what the command says.
    const SILENT = 2;
    /// `:silent!` -- do not show its errors either.
    const ERRSILENT = 4;
    /// `:unsilent` -- say it even inside a `:silent`.
    const UNSILENT = 8;
    /// `:noautocmd` -- fire no autocommands for the duration.
    const NOAUTOCMD = 16;
    /// `:hide` -- a buffer left behind becomes hidden rather than unloaded.
    const HIDE = 32;
    /// `:browse` -- ask for a file name. There is no file dialog, so this
    /// only ever reaches the "not supported" arm.
    const BROWSE = 64;
    /// `:confirm` -- prompt before a destructive step.
    const CONFIRM = 128;
    /// `:keepalt` -- leave the alternate file alone.
    const KEEPALT = 256;
    /// `:keepmarks` -- leave the marks alone.
    const KEEPMARKS = 512;
    /// `:keepjumps` -- leave the jump list and the `\'` mark alone.
    const KEEPJUMPS = 1024;
    /// `:lockmarks` -- do not move the marks for lines the command adds or
    /// removes.
    const LOCKMARKS = 2048;
    /// `:keeppatterns` -- leave the search history alone.
    const KEEPPATTERNS = 4096;
    /// `:noswapfile` -- a buffer the command opens gets no swap file.
    const NOSWAPFILE = 8192;
}
/// The `:silent`/`:noautocmd`/`:tab`/… run in front of one Ex command.
///
/// Not `Copy`: `cmod_filter_pat` and `cmod_filter_regmatch.regprog` are
/// allocations the modifier set owns, and the trailing `cmod_*_save`
/// fields are what `apply_cmdmod` put aside so `undo_cmdmod` can put it
/// back — a duplicate of those would undo the same suppression twice.
#[derive(Clone)]
pub struct CmdMod {
    pub cmod_flags: CmdModFlags,
    pub cmod_split: ::core::ffi::c_int,
    pub cmod_tab: ::core::ffi::c_int,
    pub cmod_filter_pat: *mut ::core::ffi::c_char,
    pub cmod_filter_regmatch: RegMatch,
    pub cmod_filter_force: bool,
    pub cmod_verbose: ::core::ffi::c_int,
    pub cmod_save_ei: *mut ::core::ffi::c_char,
    pub cmod_did_sandbox: ::core::ffi::c_int,
    pub cmod_verbose_save: OptInt,
    pub cmod_save_msg_silent: ::core::ffi::c_int,
    pub cmod_save_msg_scroll: ::core::ffi::c_int,
    pub cmod_did_esilent: ::core::ffi::c_int,
}

impl CmdMod {
    /// No modifiers at all: the all-zero set a command starts from, and
    /// what the C reaches with `CLEAR_FIELD(cmdmod)`. A `const` because
    /// two other all-zero initialisers embed it.
    pub const NONE: CmdMod = CmdMod {
        cmod_flags: CmdModFlags::NONE,
        cmod_split: 0,
        cmod_tab: 0,
        cmod_filter_pat: ::core::ptr::null_mut(),
        cmod_filter_regmatch: RegMatch {
            regprog: ::core::ptr::null_mut(),
            startp: [::core::ptr::null_mut(); 10],
            endp: [::core::ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        },
        cmod_filter_force: false,
        cmod_verbose: 0,
        cmod_save_ei: ::core::ptr::null_mut(),
        cmod_did_sandbox: 0,
        cmod_verbose_save: 0,
        cmod_save_msg_silent: 0,
        cmod_save_msg_scroll: 0,
        cmod_did_esilent: 0,
    };
}

impl Default for CmdMod {
    fn default() -> Self {
        Self::NONE
    }
}
/// One Ex command's line, and the cursors into it.
///
/// Upstream threads four raw pointers for this. `eap->cmdlinep` names the
/// line and is a `char **` rather than a `char *` because expanding `%` and
/// `` `cmd` `` *reallocates* the line mid-command; `eap->cmd`, `eap->arg`
/// and `eap->nextcmd` point into whatever it currently is, and every one of
/// them is silently stale the moment it does not. An offset into an owner
/// survives that reallocation, so the line is owned here and the cursors
/// are offsets.
///
/// The buffer is not one string. [`separate_nextcmd`] writes a NUL over the
/// `|` that ended the command, so `next` addresses a *second*
/// NUL-terminated string inside the same allocation, and the argument
/// expansion appends a third. [`rest_of`](CmdLine::rest_of) is what reads
/// one of them: from an offset to the next NUL, which is exactly what a C
/// consumer handed `eap->arg` sees.
///
/// [`separate_nextcmd`]: crate::ex_docmd::separate_nextcmd
/// The terminator a line with no buffer hands out, so that a cursor into it
/// reads as the empty string. See [`CmdLine::ptr_at`].
static EMPTY_LINE: [u8; 1] = [0];

#[derive(Clone, Default)]
pub struct CmdLine {
    /// The line's bytes, empty or ending in a NUL.
    text: Vec<u8>,
    /// The buffer a handler swapped out from under the command
    /// (`eap->cmdline_tofree` upstream): `:let x = 1` continued over more
    /// lines is evaluated out of a joined copy, which then *becomes* the
    /// line so that `nextcmd` can point into it. The old bytes stay alive
    /// until the command ends, because the cursors of the command still
    /// running address them.
    retired: Option<Vec<u8>>,
    /// Where the command word starts, after the range and the modifiers.
    pub cmd: usize,
    /// Where the command's argument starts.
    pub arg: usize,
    /// Where the command after a `|` starts, if there is one. A `None` is
    /// upstream's null `nextcmd`: not "offset zero", but "no next command".
    pub next: Option<usize>,
    /// `nvim_cmd`'s pre-split argument vector, as `(offset, length)` pairs
    /// into the line it rendered. Empty for a command that was typed.
    pub args: Vec<(usize, usize)>,
    /// This line is the `+` Ex mode substitutes for an empty one, not a `+`
    /// the user typed. Upstream tells the two apart by comparing `eap->cmd`
    /// against the *address* of the static it substituted, which an owned
    /// line cannot do; `ex_range_without_command` is the one reader.
    pub substituted: bool,
}

impl CmdLine {
    /// No line at all: what an `ExArg` that has not been handed one holds.
    pub const EMPTY: CmdLine = CmdLine {
        text: Vec::new(),
        retired: None,
        cmd: 0,
        arg: 0,
        next: None,
        args: Vec::new(),
        substituted: false,
    };

    /// The line `text` holds, which must be empty or end in a NUL.
    pub fn from_vec(text: Vec<u8>) -> CmdLine {
        debug_assert!(
            text.last().is_none_or(|byte| *byte == 0),
            "a command line is a NUL-terminated buffer"
        );
        CmdLine {
            text,
            ..CmdLine::EMPTY
        }
    }

    /// The line `bytes` spells, which is copied and terminated.
    pub fn from_bytes(bytes: &[u8]) -> CmdLine {
        let mut text = Vec::with_capacity(bytes.len() + 1);
        text.extend_from_slice(bytes);
        text.push(0);
        CmdLine::from_vec(text)
    }

    /// The buffer back, terminator included, leaving the cursors behind.
    pub fn into_vec(self) -> Vec<u8> {
        self.text
    }

    /// How many bytes the buffer holds, terminator included.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Is there no line at all?
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The whole buffer, every string in it and every terminator.
    pub fn buffer(&self) -> &[u8] {
        &self.text
    }

    /// The whole buffer, writable.
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.text
    }

    /// The string that starts at `at`: up to, and not including, the next
    /// NUL. What a C consumer handed `line + at` reads.
    pub fn rest_of(&self, at: usize) -> &[u8] {
        let tail = &self.text[at.min(self.text.len())..];
        let end = tail
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(tail.len());
        &tail[..end]
    }

    /// [`rest_of`](CmdLine::rest_of) with its terminator, as a `&CStr`.
    ///
    /// # Panics
    /// If the buffer holds no NUL at or after `at`.
    pub fn cstr_from(&self, at: usize) -> &::core::ffi::CStr {
        ::core::ffi::CStr::from_bytes_until_nul(&self.text[at.min(self.text.len())..])
            .expect("a command line is a NUL-terminated buffer")
    }

    /// The byte at `at`, or a NUL past the end -- which is how the C's `*p`
    /// reads at the terminator, and what every "is this the end?" test here
    /// is comparing against.
    pub fn byte_at(&self, at: usize) -> u8 {
        self.text.get(at).copied().unwrap_or(0)
    }

    /// The whole line: the first string in the buffer, which is what
    /// upstream's `*eap->cmdlinep` names.
    pub fn line(&self) -> &[u8] {
        self.rest_of(0)
    }

    /// The command word and everything after it.
    pub fn cmd(&self) -> &[u8] {
        self.rest_of(self.cmd)
    }

    /// The command's argument.
    pub fn arg(&self) -> &[u8] {
        self.rest_of(self.arg)
    }

    /// The command after the `|`, if there is one.
    pub fn next_cmd(&self) -> Option<&[u8]> {
        self.next.map(|at| self.rest_of(at))
    }

    /// Everything from `at` on, terminators and all.
    ///
    /// The *cheap* tail: [`rest_of`](CmdLine::rest_of) has to find the NUL
    /// first, which is a scan of the whole argument, so a walk that stops at
    /// a NUL of its own accord reads this instead and pays nothing. That is
    /// the difference between a `skipwhite` and a `strlen` at every step of
    /// the parse, and a bench said so: `evalbench` +2.5 % when these three
    /// and `check_for_word` went through `rest_of`.
    fn tail(&self, at: usize) -> &[u8] {
        &self.text[at.min(self.text.len())..]
    }

    /// Past the white space at `at`. Stops at the NUL, which is not white.
    pub fn skip_white(&self, at: usize) -> usize {
        at + crate::charset::skip::white(self.tail(at))
    }

    /// Past the non-white bytes at `at`. Bounded by the string, because a
    /// NUL *is* a non-white byte.
    pub fn skip_to_white(&self, at: usize) -> usize {
        at + crate::charset::skip::to_white(self.rest_of(at))
    }

    /// Past the decimal digits at `at`. Stops at the NUL.
    pub fn skip_digits(&self, at: usize) -> usize {
        at + crate::charset::skip::digits(self.tail(at))
    }

    /// Does the string at `at` start with `prefix`? Stops at the NUL, which
    /// matches nothing a caller passes.
    pub fn starts_with(&self, at: usize, prefix: &[u8]) -> bool {
        debug_assert!(!prefix.contains(&0), "a NUL would match past the end");
        self.tail(at).starts_with(prefix)
    }

    /// How many bytes of `name` the string at `at` matches, stopping at the
    /// first difference — and so at the NUL. [`starts_with`](CmdLine::starts_with)'s partial form.
    pub fn shared_prefix(&self, at: usize, name: &[u8]) -> usize {
        self.tail(at)
            .iter()
            .zip(name)
            .take_while(|(got, want)| got == want)
            .count()
    }

    /// Where the string that starts at `at` ends.
    pub fn end_of(&self, at: usize) -> usize {
        at + self.rest_of(at).len()
    }

    /// The command after `at`, if `at` is at the separator that
    /// introduces one. `check_nextcmd` upstream.
    pub fn check_next(&self, at: usize) -> Option<usize> {
        let at = self.skip_white(at);
        matches!(self.byte_at(at), b'|' | b'\n').then_some(at + 1)
    }

    /// Write `byte` at `at`.
    pub fn set_byte(&mut self, at: usize, byte: u8) {
        self.text[at] = byte;
    }

    /// End the string that starts before `at` here, which is how a `|` is
    /// turned into two commands.
    pub fn terminate_at(&mut self, at: usize) {
        self.set_byte(at, 0);
    }

    /// Delete the byte at `at` by pulling the rest of *its own string*, the
    /// terminator included, over it.
    ///
    /// Only that string moves: anything the buffer holds past its NUL --
    /// the next command, an expanded argument -- stays where it is, so
    /// every offset except the ones inside this string keeps its meaning.
    /// That is exactly the `memmove(p, p + 1, strlen(p + 1) + 1)` upstream
    /// writes, and the reason this is not `Vec::remove`.
    pub fn drop_byte(&mut self, at: usize) {
        let end = self.end_of(at);
        self.text.copy_within(at + 1..=end, at);
    }

    /// Replace the `srclen` bytes at `at` with `repl`, and answer how much
    /// longer the buffer got.
    ///
    /// Every offset past the replacement moves by that amount; the caller
    /// owns the ones this type does not know about.
    pub fn splice(&mut self, at: usize, srclen: usize, repl: &[u8]) -> isize {
        let end = (at + srclen).min(self.text.len());
        self.text.splice(at..end, repl.iter().copied());
        let delta = repl.len().cast_signed() - (end - at).cast_signed();
        Self::shift(&mut self.cmd, at, delta);
        Self::shift(&mut self.arg, at, delta);
        if let Some(next) = self.next.as_mut() {
            Self::shift(next, at, delta);
        }
        for (offset, _) in &mut self.args {
            Self::shift(offset, at, delta);
        }
        delta
    }

    /// Move one offset by `delta`, if it is past `at`.
    fn shift(offset: &mut usize, at: usize, delta: isize) {
        if *offset > at {
            *offset = offset.wrapping_add_signed(delta);
        }
    }

    /// Throw away everything before `at`, so that what started there is now
    /// the whole line, and forget every cursor.
    ///
    /// What `do_cmdline` does between two `|`-separated commands: the tail
    /// moves to the front of the same allocation rather than becoming a
    /// second one.
    pub fn restart_at(&mut self, at: usize) {
        let end = self.end_of(at);
        self.text.copy_within(at..=end, 0);
        self.text.truncate(end - at + 1);
        self.cmd = 0;
        self.arg = 0;
        self.next = None;
        self.args.clear();
        self.retired = None;
        self.substituted = false;
    }

    /// Take `new` over as the line, retiring what was there.
    ///
    /// The cursors are *not* rebased: they still address the retired
    /// buffer, which is why it stays alive. Upstream's `eap->cmdline_tofree`
    /// dance, with the lifetime written down.
    pub fn take_over(&mut self, new: Vec<u8>) {
        self.retired = Some(::core::mem::replace(&mut self.text, new));
    }

    /// Does `at` address the retired buffer rather than the live one?
    pub fn is_retired(&self, at: usize) -> bool {
        self.retired.is_some() && at >= self.text.len()
    }

    /// A writable pointer at `at`, for a callee that still takes one.
    ///
    /// A line with no buffer at all answers the shared terminator, because
    /// a cursor into the empty line must still *read* as the empty string;
    /// a caller that writes through one is wrong either way.
    pub fn ptr_at(&mut self, at: usize) -> *mut ::core::ffi::c_char {
        if self.text.is_empty() {
            return EMPTY_LINE.as_ptr().cast::<::core::ffi::c_char>().cast_mut();
        }
        let at = at.min(self.text.len());
        self.text[at..].as_mut_ptr().cast()
    }

    /// A readable pointer at `at`, for a callee that still takes one.
    pub fn ptr_from(&self, at: usize) -> *const ::core::ffi::c_char {
        if self.text.is_empty() {
            return EMPTY_LINE.as_ptr().cast();
        }
        self.text[at.min(self.text.len())..].as_ptr().cast()
    }

    /// Does `p` point into the live buffer?
    pub fn contains(&self, p: *const ::core::ffi::c_char) -> bool {
        let base = self.text.as_ptr().addr();
        (base..=base + self.text.len()).contains(&p.addr())
    }

    /// Where `p`, which must point into the live buffer, is.
    ///
    /// # Panics
    /// In a debug build, if `p` is not inside the line. A cursor taken from
    /// somewhere else is the mistake this type exists to make impossible,
    /// and a wrapped offset would only hide it.
    pub fn offset_of(&self, p: *const ::core::ffi::c_char) -> usize {
        debug_assert!(self.contains(p), "a cursor from outside the line");
        p.addr() - self.text.as_ptr().addr()
    }
}

/// One parsed Ex command line.
///
/// Not `Copy`: `line` is the buffer the command is parsed out of, owned for
/// as long as it runs.
#[derive(Clone)]
pub struct ExArg {
    /// The command line and the three cursors into it.
    pub line: CmdLine,
    pub cmdidx: CmdIdx,
    pub argt: ExArgt,
    pub skip: bool,
    pub forceit: bool,
    pub addr_count: ::core::ffi::c_int,
    pub line1: LineNr,
    pub line2: LineNr,
    pub addr_type: CmdAddr,
    pub flags: ::core::ffi::c_int,
    pub do_ecmd_cmd: *mut ::core::ffi::c_char,
    pub do_ecmd_lnum: LineNr,
    pub append: bool,
    pub usefilter: bool,
    pub amount: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub force_bin: ::core::ffi::c_int,
    pub read_edit: bool,
    pub mkdir_p: bool,
    pub force_ff: ::core::ffi::c_int,
    pub force_enc: ::core::ffi::c_int,
    pub bad_char: ::core::ffi::c_int,
    pub useridx: ::core::ffi::c_int,
    /// Why the command failed, owned by whoever raised it.
    ///
    /// Upstream threads a `char *` here that may point at a static string
    /// or at one of two shared static buffers, so a second error assembled
    /// before the first was reported overwrote it. Owning the text removes
    /// the sharing.
    pub errmsg: Option<::std::ffi::CString>,
    pub ea_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub cstack: *mut CondStack,
}

impl ExArg {
    /// The argument as a writable pointer.
    ///
    /// Transitional: every one of these is a handler that has not been
    /// moved onto [`CmdLine::arg`] yet, and the accessor goes when the last
    /// of them has.
    pub fn arg_ptr(&mut self) -> *mut ::core::ffi::c_char {
        self.line.ptr_at(self.line.arg)
    }

    /// Put the argument cursor where `p`, which must point into the line,
    /// is. Transitional, as [`arg_ptr`](ExArg::arg_ptr).
    pub fn set_arg_ptr(&mut self, p: *const ::core::ffi::c_char) {
        self.line.arg = self.line.offset_of(p);
    }

    /// The command word as a writable pointer. Transitional.
    pub fn cmd_ptr(&mut self) -> *mut ::core::ffi::c_char {
        self.line.ptr_at(self.line.cmd)
    }

    /// Put the command cursor where `p` is. Transitional.
    pub fn set_cmd_ptr(&mut self, p: *const ::core::ffi::c_char) {
        self.line.cmd = self.line.offset_of(p);
    }

    /// The next command as a writable pointer, null when there is none.
    /// Transitional.
    pub fn nextcmd_ptr(&mut self) -> *mut ::core::ffi::c_char {
        match self.line.next {
            Some(at) => self.line.ptr_at(at),
            None => ::core::ptr::null_mut(),
        }
    }

    /// Say where the next command is, or that there is none. Transitional.
    pub fn set_nextcmd_ptr(&mut self, p: *const ::core::ffi::c_char) {
        self.line.next = (!p.is_null()).then(|| self.line.offset_of(p));
    }

    /// Lend the command cursor to a callee that advances a
    /// `*mut *mut c_char`, and take back where it left off. Transitional,
    /// as [`cmd_ptr`](ExArg::cmd_ptr).
    pub fn with_cmd_cursor<T>(
        &mut self,
        walk: impl FnOnce(*mut *mut ::core::ffi::c_char) -> T,
    ) -> T {
        let mut cursor = self.cmd_ptr();
        let answer = walk(&raw mut cursor);
        self.set_cmd_ptr(cursor);
        answer
    }

    /// Lend the argument cursor, as [`with_cmd_cursor`](ExArg::with_cmd_cursor).
    pub fn with_arg_cursor<T>(
        &mut self,
        walk: impl FnOnce(*mut *mut ::core::ffi::c_char) -> T,
    ) -> T {
        let mut cursor = self.arg_ptr();
        let answer = walk(&raw mut cursor);
        self.set_arg_ptr(cursor);
        answer
    }

    /// Make `text` the command's whole line, with `arg` at its start.
    ///
    /// What a handler does when it rewrites its own argument into a buffer
    /// of its own: `:diffget 3` builds the count, `:oldfiles` expands the
    /// name it picked, `:find` answers a path off `'path'`. Upstream points
    /// `eap->arg` at that other buffer and leaves the command line behind
    /// it, so the argument outlives the buffer or the buffer outlives the
    /// command; owning the text is the same rewrite with the lifetime
    /// written down.
    pub fn set_arg_text(&mut self, text: &[u8]) {
        self.line = CmdLine::from_bytes(text);
    }

    /// [`set_arg_text`](ExArg::set_arg_text), for a *synthetic command*:
    /// the whole line, command word included, is written here rather than
    /// typed. `prep_exarg`'s `:e ++enc=…` is the one caller.
    pub fn set_cmd_text(&mut self, text: &[u8]) {
        self.line = CmdLine::from_bytes(text);
    }

    /// The whole line as a writable pointer -- upstream's
    /// `*eap->cmdlinep`. Transitional.
    pub fn line_ptr(&mut self) -> *mut ::core::ffi::c_char {
        self.line.ptr_at(0)
    }
}

impl Default for ExArg {
    /// The all-zero `ExArg` that `CLEAR_FIELD(ea)` produces upstream.
    fn default() -> Self {
        ExArg {
            line: CmdLine::EMPTY,
            cmdidx: CmdIdx::append,
            argt: ExArgt::NONE,
            skip: false,
            forceit: false,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: CmdAddr::Lines,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut(),
            do_ecmd_lnum: 0,
            append: false,
            usefilter: false,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: false,
            mkdir_p: false,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: None,
            ea_getline: None,
            cookie: ::core::ptr::null_mut(),
            cstack: ::core::ptr::null_mut(),
        }
    }
}
