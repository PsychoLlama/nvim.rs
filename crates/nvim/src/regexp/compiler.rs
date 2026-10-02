//! [`RegCompiler`]: everything one pattern compile keeps while it runs.
//!
//! Upstream keeps this as some thirty file-scope statics — the pattern
//! cursor and its one-token lookbehind, what the pattern turned out to hold,
//! the backtracker's output cursor, the NFA's state counters — reset at the
//! start of every compile and read by every function of both compilers.
//! Here [`super::vim_regcomp`] makes one per engine attempt and passes it
//! down by `&mut`, so a compile that starts while another is in flight (a
//! pattern compiled from inside an error message, say, or from a `\=`
//! expression between two matches) gets fresh state by construction rather
//! than by everybody remembering to save and restore it.
//!
//! The fields fall in three groups: the reader (the cursor and what
//! [`super::parse`] keeps around it), the findings both engines record, and
//! each engine's own output — the backtracker's [`BtEmitter`] and the NFA's
//! postfix program and state counters.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_uint};

use super::bt::BtEmitter;
use super::nfa::Postfix;
use super::{MAGIC_OFF, MAGIC_ON, Magic, NSUBEXP, NfaState, RE_MAGIC, RE_STRICT, RE_STRING};
use crate::option::cpo_has;
use crate::types::{CpoFlag, uint8_t};
use crate::winlayer::Buf;

/// One compile's state. See the module docs.
pub(crate) struct RegCompiler {
    /// The pattern, NUL-terminated, and the flags the compile was asked for
    /// (`RE_MAGIC`, `RE_STRING`, `RE_STRICT`, `RE_AUTO`). Only ever read.
    pub(crate) pattern: *mut c_char,
    pub(crate) re_flags: c_int,

    // ---------------------------------------------------- the reader
    /// Where the parse has got to in the pattern.
    pub(crate) cursor: *mut c_char,
    /// The token at the cursor, once [`super::peekchr`] has read it; -1
    /// before. A metacharacter is its byte minus 256.
    pub(crate) token: c_int,
    /// The two tokens before it, and the one [`super::ungetchr`] pushed back
    /// (-1 for none).
    pub(crate) prev_token: c_int,
    pub(crate) prev2_token: c_int,
    pub(crate) next_token: c_int,
    /// How many bytes the previous token took, which is how far
    /// [`super::ungetchr`] steps back.
    pub(crate) prev_token_len: c_int,
    /// Is the cursor where a branch starts, so that `^` anchors and `*` is
    /// literal? And was the previous token?
    pub(crate) at_start: c_int,
    pub(crate) prev_at_start: c_int,
    /// How deep [`super::peekchr`]'s reparse of a `\x` escape is.
    pub(crate) escape_depth: c_int,
    /// The 'magic' level in force: the caller's, until a `\v`/`\m`/`\M`/`\V`
    /// changes it.
    pub(crate) magic: Magic,
    /// 'cpoptions' has `l`: `\t` and friends are literal inside `[]`.
    pub(crate) cpo_lit: bool,
    /// `RE_STRING`: `^` and `$` may match at a newline inside the text.
    pub(crate) string_match: c_int,
    /// `RE_STRICT`: a `[` without its `]` is an error, not a literal.
    pub(crate) strict: c_int,

    // ------------------------------------------- what the pattern holds
    /// The next `\(` group number, and the next `\z(` one.
    pub(crate) next_group: c_int,
    pub(crate) next_zgroup: c_int,
    /// `\z(` groups were defined (`REX_SET`) or `\z1` used (`REX_USE`).
    pub(crate) has_z: c_int,
    /// The `RF_*` findings: `\c`, `\C`, `\Z`, and what the program needs.
    pub(crate) flags: c_uint,
    /// The pattern ends in a `\n` a search should take as end-of-line.
    pub(crate) had_eol: c_int,
    /// Which `\(` groups have closed, so that `\N` may refer to them.
    pub(crate) closed_groups: [uint8_t; NSUBEXP as usize],
    /// A `\%[` member is being parsed: one atom, not a literal run.
    pub(crate) one_exactly: c_int,
    /// What `\k` and the other 'iskeyword' classes read.
    pub(crate) buf: Buf,

    // ------------------------------------------------ the backtracker
    pub(crate) code: BtEmitter,
    /// How many complex `\{n,m}` the program has; the matcher has ten
    /// counter slots.
    pub(crate) complex_braces: c_int,

    // ------------------------------------------------------- the NFA
    /// Something only this engine handles was seen, so a wide `\{n,m}`
    /// must not hand the pattern to the backtracker.
    pub(crate) wants_nfa: bool,
    /// The pattern uses `\ze`, or a back-reference.
    pub(crate) has_zend: c_int,
    pub(crate) has_backref: c_int,
    pub(crate) post: Postfix,
    pub(crate) nfa: NfaStates,
}

/// The NFA program's states while [`super::nfa`]'s builder lays them out:
/// counted on the first pass, written on the second.
pub(crate) struct NfaStates {
    /// How many the program needs.
    pub(crate) count: c_int,
    /// How many are written so far.
    pub(crate) built: c_int,
    /// The program's state array, `count` long, once it is allocated.
    pub(crate) base: *mut NfaState,
}

impl RegCompiler {
    /// A compiler over `pattern`, with `\k` reading `buffer`'s 'iskeyword'.
    ///
    /// The compiler keeps a pointer into `pattern` and must not outlive it:
    /// [`super::vim_regcomp`] makes one per engine attempt and drops it
    /// before returning.
    pub(crate) fn new(pattern: &CStr, re_flags: c_int, buffer: Buf) -> RegCompiler {
        let mut rc = RegCompiler {
            pattern: pattern.as_ptr().cast_mut(),
            re_flags,
            cursor: core::ptr::null_mut(),
            token: -1,
            prev_token: -1,
            prev2_token: -1,
            next_token: -1,
            prev_token_len: 0,
            at_start: 1,
            prev_at_start: 0,
            escape_depth: 0,
            magic: MAGIC_OFF,
            cpo_lit: false,
            string_match: 0,
            strict: 0,
            next_group: 1,
            next_zgroup: 1,
            has_z: 0,
            flags: 0,
            had_eol: 0,
            closed_groups: [0; NSUBEXP as usize],
            one_exactly: 0,
            buf: buffer,
            code: BtEmitter::new(),
            complex_braces: 0,
            wants_nfa: false,
            has_zend: 0,
            has_backref: 0,
            post: Postfix::new(),
            nfa: NfaStates {
                count: 0,
                built: 0,
                base: core::ptr::null_mut(),
            },
        };
        rc.restart();
        rc
    }

    /// Put the reader back at the start of the pattern and forget what it
    /// found: the backtracker parses the pattern twice, once to measure and
    /// once to write.
    pub(crate) fn restart(&mut self) {
        self.cursor = self.pattern;
        self.prev_token_len = 0;
        self.next_token = -1;
        self.prev_token = -1;
        self.prev2_token = -1;
        self.token = -1;
        self.at_start = 1;
        self.prev_at_start = 0;
        self.magic = if self.re_flags & RE_MAGIC != 0 {
            MAGIC_ON
        } else {
            MAGIC_OFF
        };
        self.string_match = self.re_flags & RE_STRING;
        self.strict = self.re_flags & RE_STRICT;
        self.cpo_lit = cpo_has(CpoFlag::LITERAL);
        self.complex_braces = 0;
        self.next_group = 1;
        self.closed_groups = [0; NSUBEXP as usize];
        self.next_zgroup = 1;
        self.has_z = 0;
        self.flags = 0;
        self.had_eol = 0;
    }
}
