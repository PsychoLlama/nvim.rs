//! Reading a function body, one line at a time.
//!
//! `get_function_body` is `:function`'s line loop: it tracks `:if`/`:while`/
//! `:for`/`:try` nesting so that the matching `:endfunction` is the right
//! one, keeps continuation lines and comments verbatim, honours a
//! here-document inside the body, and refuses to nest more than
//! MAX_FUNC_NESTING definitions deep.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;
use crate::ex_docmd::check_for_word_in;
use crate::memory::XString;
use crate::runtime::sourced_lnum_of;
use crate::semsg;
use crate::swmsg;
use core::ffi::c_int;

use super::*;
use crate::types::Failed;

/// How many `:function` definitions may nest inside one another.
pub const MAX_FUNC_NESTING: c_int = 50;

/// Whether the command `cmd` starts with is one of the interpreter
/// commands that takes a `<<` heredoc, matched by its shortest
/// abbreviation plus whatever may follow: `py`/`py3`/`pyx`/`pyt`hon,
/// `pe`rl, `tc`l, `lua`, `rub`y, `mz`scheme.
fn is_heredoc_command(cmd: &[u8]) -> bool {
    let b = |i: usize| cmd.get(i).copied().unwrap_or(0);
    (b(0) == b'p'
        && b(1) == b'y'
        && (!b(2).is_ascii_alphanumeric()
            || b(2) == b't'
            || ((b(2) == b'3' || b(2) == b'x') && !b(3).is_ascii_alphabetic())))
        || (b(0) == b'p' && b(1) == b'e' && (!b(2).is_ascii_alphabetic() || b(2) == b'r'))
        || (b(0) == b't' && b(1) == b'c' && (!b(2).is_ascii_alphabetic() || b(2) == b'l'))
        || (b(0) == b'l' && b(1) == b'u' && b(2) == b'a' && !b(3).is_ascii_alphabetic())
        || (b(0) == b'r'
            && b(1) == b'u'
            && b(2) == b'b'
            && (!b(3).is_ascii_alphabetic() || b(3) == b'y'))
        || (b(0) == b'm' && b(1) == b'z' && (!b(2).is_ascii_alphabetic() || b(2) == b's'))
}

/// Whether `text` starts with the modifier `word` (`trim`, `eval`) and
/// nothing but a blank or the end follows it.
fn is_word(text: &[u8], word: &[u8]) -> bool {
    text.starts_with(word) && matches!(text.get(word.len()), None | Some(b' ' | b'\t'))
}

/// What the body reader knows between lines: the block nesting it indents
/// by, how many definitions are open inside the body, and the end marker
/// of an open heredoc or `:append`.
#[derive(Debug)]
struct BodyScan {
    indent: c_int,
    /// `:function`s opened inside the body and not yet ended.
    nesting: c_int,
    /// The line that ends a heredoc or an `:append`, while one is open.
    skip_until: Option<Vec<u8>>,
    is_heredoc: bool,
    /// The indent a `trim` heredoc's lines lose.
    heredoc_trimmed: Vec<u8>,
    /// Join continuation lines (off inside a heredoc).
    do_concat: bool,
}

impl BodyScan {
    fn new() -> Self {
        Self {
            indent: 2,
            nesting: 0,
            skip_until: None,
            is_heredoc: false,
            heredoc_trimmed: Vec::new(),
            do_concat: true,
        }
    }

    /// The reader ran out of lines: say what was left open.
    fn report_missing_end(&self) {
        if let Some(marker) = &self.skip_until {
            let marker = msg_bytes(marker);
            semsg!("E1145: Missing heredoc end marker: {marker}");
        } else {
            emsg(gettext(c"E126: Missing :endfunction"));
        }
    }

    /// Inside a heredoc or an `:append`, `line` is body text; close the
    /// block when it is the end marker. Don't check for ":endfunc" between
    /// * ":append" and "."
    /// * ":python <<EOF" and "EOF"
    /// * ":let {var-name} =<< [trim] {marker}" and "{marker}"
    fn match_end_marker(&mut self, line: &[u8], indented: usize) {
        let Some(marker) = &self.skip_until else {
            return;
        };
        let unindented = self.heredoc_trimmed.is_empty() || (self.is_heredoc && indented == 0);
        if unindented || line.starts_with(&self.heredoc_trimmed) {
            let at = if unindented {
                0
            } else {
                self.heredoc_trimmed.len()
            };
            if line[at..] == marker[..] {
                self.skip_until = None;
                self.heredoc_trimmed.clear();
                self.do_concat = true;
                self.is_heredoc = false;
            }
        }
    }

    /// An `:endfunction` was seen: whether it is the outermost one, which
    /// ends the body. The count drops on every one seen.
    fn leave_function(&mut self) -> bool {
        let outermost = self.nesting == 0;
        self.nesting -= 1;
        outermost
    }

    /// Read the command at `p` in `line` for what changes how the lines
    /// after it are read: block indent, a nested definition, an `:append`
    /// or a heredoc.
    fn scan_command(&mut self, line: &[u8], mut p: usize, indented: usize) {
        // Increase the indent inside "if", "while", "for" and
        // "try", decrease it at "end".
        let word = &line[p..];
        if self.indent > 2 && word.starts_with(b"end") {
            self.indent -= 2;
        } else if word.starts_with(b"if")
            || word.starts_with(b"wh")
            || word.starts_with(b"for")
            || word.starts_with(b"try")
        {
            self.indent += 2;
        }

        // Check for defining a function inside this function.
        if let Some(after) = check_for_word_in(line, p, b"function", 2) {
            p = after;
            self.enter_nested_function(line, p);
        }

        // Check for ":append", ":change", ":insert", which run
        // until a line holding only a dot.
        // A null context is "not completing".
        p += skip_range(&line[p..], None);
        let ranged = check_for_word_in(line, p, b"append", 1)
            .or_else(|| check_for_word_in(line, p, b"change", 1))
            .or_else(|| check_for_word_in(line, p, b"insert", 1));
        if let Some(after) = ranged
            && (byte_at(line, after) == b'!'
                || byte_at(line, after) == b'|'
                || ascii_iswhite_nl_or_nul(c_int::from(byte_at(line, after))))
        {
            self.skip_until = Some(b".".to_vec());
            p = after;
        }

        p = self.start_command_heredoc(line, p, indented);
        if !self.is_heredoc {
            self.start_let_heredoc(line, p, indented);
        }
    }

    /// `:function` at `p`: a definition (a name and its `(`) nests one
    /// deeper, up to [`MAX_FUNC_NESTING`].
    fn enter_nested_function(&mut self, line: &[u8], mut p: usize) {
        if byte_at(line, p) == b'!' {
            p += 1 + skip::white(&line[p + 1..]);
        }
        p += fname_script_len(&line[p..]);
        p += trans_function_name(&line[p..], true, 0, false).end;
        if byte_at(line, p + skip::white(&line[p..])) == b'(' {
            if self.nesting == MAX_FUNC_NESTING - 1 {
                emsg(gettext(E_FUNCTION_NESTING_TOO_DEEP));
            } else {
                self.nesting += 1;
                self.indent += 2;
            }
        }
    }

    /// Heredoc: check for ":python <<EOF", ":lua <<EOF", etc. Answers where
    /// the command's reading continues.
    fn start_command_heredoc(&mut self, line: &[u8], mut p: usize, indented: usize) -> usize {
        let mut arg = p + skip::to_white(&line[p..]);
        arg += skip::white(&line[arg..]);
        if line[arg..].starts_with(b"<<") && is_heredoc_command(&line[p..]) {
            // ":python <<" continues until a dot, like ":append".
            p = arg + 2 + skip::white(&line[arg + 2..]);
            if is_word(&line[p..], b"trim") {
                // Ignore leading white space.
                p += 4 + skip::white(&line[p + 4..]);
                self.heredoc_trimmed = line[..indented].to_vec();
            }
            self.skip_until = Some(if p == line.len() {
                b".".to_vec()
            } else {
                line[p..p + skip::to_white(&line[p..])].to_vec()
            });
            self.do_concat = false;
            self.is_heredoc = true;
        }
        p
    }

    /// Check for ":cmd v =<< [trim] EOF" and ":cmd [a, b] =<< [trim] EOF",
    /// where "cmd" is "let" or "const".
    ///
    /// Upstream steps past "let" with the copy it parses the targets from,
    /// but past "const" with the line's own cursor, so for "const" the
    /// targets are read from the word "const" itself and never reach the
    /// "=<<".
    fn start_let_heredoc(&mut self, line: &[u8], mut p: usize, indented: usize) {
        let targets = match check_for_word_in(line, p, b"let", 2) {
            Some(after) => Some(after),
            None => check_for_word_in(line, p, b"const", 5).map(|after| {
                let at = p;
                p = after;
                at
            }),
        };
        let operator = targets
            .and_then(|at| skip_var_list(&line[at..], true).map(|list| at + list.end))
            .map(|at| at + skip::white(&line[at..]));
        let Some(operator) = operator.filter(|&at| line[at..].starts_with(b"=<<")) else {
            return;
        };
        p = operator + 3 + skip::white(&line[operator + 3..]);
        let mut has_trim = false;
        loop {
            // Both modifiers may appear, in either order
            // and more than once.
            if is_word(&line[p..], b"trim") {
                p += 4 + skip::white(&line[p + 4..]);
                has_trim = true;
            } else if is_word(&line[p..], b"eval") {
                p += 4 + skip::white(&line[p + 4..]);
            } else {
                break;
            }
        }
        if has_trim {
            self.heredoc_trimmed = line[..indented].to_vec();
        }
        self.skip_until = Some(line[p..p + skip::to_white(&line[p..])].to_vec());
        self.do_concat = false;
        self.is_heredoc = true;
    }
}

/// Where the command after `:endfunction` starts: in the command's own
/// line, or in the line the last read handed back.
#[derive(Debug)]
enum NextCommand {
    InCommand(usize),
    InRead(usize),
}

impl ExArg<'_> {
    /// Read the body of a `:function`, up to its `:endfunction`.
    ///
    /// Lines come either from the command's own line, from `line_arg` on (an
    /// `:execute`d definition, split on newlines), or from the command line /
    /// the script being sourced.  Each one is appended to `newlines`, with a
    /// `None` per continuation line so that the index stays the line number.
    /// `line_to_free` keeps whatever the last read handed back.
    pub(crate) fn read_function_body(
        &mut self,
        newlines: &mut Vec<Option<Box<[u8]>>>,
        mut line_arg: Option<usize>,
        line_to_free: &mut Option<XString>,
        show_block: bool,
    ) -> Result<(), Failed> {
        let mut saved_wait_return = NEED_WAIT_RETURN.get();
        let mut scan = BodyScan::new();

        let ended = loop {
            if KEY_TYPED.get() {
                MSG_SCROLL.set(1);
                saved_wait_return = false;
            }
            NEED_WAIT_RETURN.set(false);

            let Some((line, line_at)) = self.next_function_line(&mut line_arg, line_to_free, &scan)
            else {
                scan.report_missing_end();
                break false;
            };
            let line = &line[..];
            if show_block {
                ui_ext_cmdline_block_append(usize::try_from(scan.indent).unwrap_or(0), line);
            }

            // Detect line continuation: SOURCING_LNUM increased by more
            // than one.
            let mut sourcing_lnum_off = sourced_lnum_of(self);
            if sourcing_lnum() < sourcing_lnum_off {
                sourcing_lnum_off -= sourcing_lnum().get();
            } else {
                sourcing_lnum_off = LineNr::ZERO;
            }

            let indented = skip::white(line);
            if scan.skip_until.is_some() {
                scan.match_end_marker(line, indented);
            } else {
                // Skip ':' and blanks.
                let mut p = line
                    .iter()
                    .take_while(|&&b| ascii_iswhite(c_int::from(b)) || b == b':')
                    .count();

                // Check for "endfunction": only the outermost ends the body. A
                // nested one is stepped over like any other matched word, so
                // the indent test reads what follows it.
                if let Some(after) = check_for_word_in(line, p, b"endfunction", 4) {
                    p = after;
                    if scan.leave_function() {
                        self.continue_after_endfunction(line, p, line_at, line_arg, line_to_free);
                        break true;
                    }
                }
                scan.scan_command(line, p, indented);
            }

            // Add the line to the function, and a `None` per continuation
            // line, so that the line count equals the index.
            newlines.reserve(1 + usize::try_from(sourcing_lnum_off).unwrap_or(0));
            newlines.push(Some(line.into()));
            for _ in 0..sourcing_lnum_off.get() {
                newlines.push(None);
            }

            // Check for the end of eap->arg.
            if line_arg.is_some_and(|at| self.line.byte_at(at) == 0) {
                line_arg = None;
            }
        };

        NEED_WAIT_RETURN.set(NEED_WAIT_RETURN.get() || saved_wait_return);
        // OK when the body ended and no error was detected.
        if ended && DID_EMSG.get() == 0 {
            Ok(())
        } else {
            Err(Failed)
        }
    }

    /// The body's next line, and where it starts in the command's own line
    /// when it is a part of that; `None` when the reader has run out.
    fn next_function_line(
        &mut self,
        line_arg: &mut Option<usize>,
        line_to_free: &mut Option<XString>,
        scan: &BodyScan,
    ) -> Option<(Vec<u8>, Option<usize>)> {
        if let Some(at) = *line_arg {
            // Use eap->arg, split up in parts by line breaks.
            let rest = self.line.rest_of(at);
            let line = match rest.iter().position(|&b| b == b'\n') {
                None => {
                    *line_arg = Some(at + rest.len());
                    rest.to_vec()
                }
                Some(len) => {
                    let line = rest[..len].to_vec();
                    // The command's line is split in place, as upstream
                    // does.
                    self.line.terminate_at(at + len);
                    *line_arg = Some(at + len + 1);
                    line
                }
            };
            if KEY_TYPED.get() {
                LINES_LEFT.set(ROWS.get() - 1);
            }
            Some((line, Some(at)))
        } else {
            *line_to_free = None;
            *line_to_free = self.next_body_line(Some(b':'), scan.indent, scan.do_concat);
            if KEY_TYPED.get() {
                LINES_LEFT.set(ROWS.get() - 1);
            }
            let read = line_to_free.as_ref()?;
            Some((read.as_cstr().to_bytes().to_vec(), None))
        }
    }

    /// The body's `:endfunction` ends at `after` in `line`: point the
    /// command at what follows it (`|` or the rest of an `:execute`d
    /// definition), or warn about trailing text.
    fn continue_after_endfunction(
        &mut self,
        line: &[u8],
        mut after: usize,
        line_at: Option<usize>,
        line_arg: Option<usize>,
        line_to_free: &mut Option<XString>,
    ) {
        if byte_at(line, after) == b'!' {
            after += 1;
        }
        let mut nextcmd = None;
        let more_args = line_arg.is_some_and(|at| {
            let rest = self.line.rest_of(at);
            skip::white(rest) < rest.len()
        });
        if byte_at(line, after) == b'|' {
            nextcmd = Some(match line_at {
                Some(at) => NextCommand::InCommand(at + after + 1),
                None => NextCommand::InRead(after + 1),
            });
        } else if more_args {
            nextcmd = line_arg.map(NextCommand::InCommand);
        } else if !matches!(byte_at(line, after), 0 | b'"') && p_verbose() > 0 {
            let rest = msg_bytes(&line[after..]);
            swmsg!(true, "W22: Text found after :endfunction: {rest}");
        }
        match nextcmd {
            // Another command follows in the command's own line:
            // the offset is all that is needed.
            Some(NextCommand::InCommand(at)) => self.line.next = Some(at),
            // It is in the line the last read handed back, which
            // the command takes over.
            Some(NextCommand::InRead(at)) => {
                let taken = line_to_free.take().expect("the line just read");
                self.line.take_over(taken.into_vec());
                self.line.next = Some(at);
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_heredoc_command, is_word};

    #[test]
    fn heredoc_commands_match_by_their_shortest_abbreviation() {
        for cmd in [
            &b"py <<"[..],
            b"python3 <<",
            b"pyx",
            b"pythonx",
            b"perl",
            b"pe",
            b"tcl",
            b"lua <<",
            b"ruby",
            b"rub",
            b"mzscheme",
        ] {
            assert!(is_heredoc_command(cmd), "{}", String::from_utf8_lossy(cmd));
        }
        for cmd in [&b"p"[..], b"pyfile", b"luafile", b"let", b""] {
            assert!(!is_heredoc_command(cmd), "{}", String::from_utf8_lossy(cmd));
        }
    }

    #[test]
    fn a_modifier_word_ends_at_a_blank_or_the_line() {
        assert!(is_word(b"trim", b"trim"));
        assert!(is_word(b"trim EOF", b"trim"));
        assert!(is_word(b"eval\tEOF", b"eval"));
        assert!(!is_word(b"trimmed", b"trim"));
        assert!(!is_word(b"tri", b"trim"));
    }
}
