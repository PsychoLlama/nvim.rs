//! `:function` itself -- defining, and the header a listing prints.
//!
//! `ex_function` decides which of the four things the command is (define,
//! list one, list a pattern, list everything), builds the `UserFunc` and
//! installs it in the table.  `list_func_head` prints the `function
//! Name(a, b = 1, ...) dict abort range` line, which is the same text in a
//! listing and in a `:verbose` report.

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

use crate::cstr;
use crate::lua::executor::set_sctx_from_lua;
use crate::memory::ThinCString;
use crate::message_fmt::msg_bytes;
use crate::runtime::{autoload_name_of, sourcing_name_bytes};
use crate::semsg;
use core::ffi::{c_char, c_int};
use std::rc::Rc;

use super::*;
use crate::eval::typval::value_check_lock_named;
use crate::eval::vars::with_var;
use crate::memory::XString;
use crate::types::{Failed, FuncBody, Refcount};

/// Whether the function table changed under a listing, which means the
/// function the caller is walking to may be gone.  Reports E454 when it did.
pub(crate) fn function_list_modified(prev_ht_changed: c_int) -> c_int {
    if prev_ht_changed != func_table_changed() {
        emsg(gettext(E_FUNCTION_LIST_WAS_MODIFIED));
        return 1;
    }
    0
}

/// Print `function Name(a, b = 1, ...) range dict abort closure`, the head of
/// a listing and of a `:verbose` report.
pub(crate) fn list_func_head(func: &UserFunc, indent: bool, force: bool) -> Result<(), Failed> {
    let prev_ht_changed = func_table_changed();

    msg_start();

    // Check no function was added or removed from a callback, as
    // `msg_start` may have invoked a redraw.
    if function_list_modified(prev_ht_changed) != 0 {
        return Err(Failed);
    }

    if indent {
        msg_str(c"   ");
    }
    msg_str(if force { c"function! " } else { c"function " });
    msg_str(func.printable_name().as_cstr());
    msg_putchar(c_int::from(b'('));

    let body = func.body();
    let (args, defaults) = (&body.args, &body.def_args);
    // The defaults are right-aligned with the arguments: the last
    // `defaults.len()` arguments are the ones that have one.
    // Upstream computes this in `int`, where a (impossible) surplus of
    // defaults would go negative and give every argument one.
    let first_default = args.len().saturating_sub(defaults.len());
    for (j, arg) in args.iter().enumerate() {
        if j != 0 {
            msg_str(c", ");
        }
        cstr::with_terminated(arg, msg_str);
        if j >= first_default {
            msg_str(c" = ");
            cstr::with_terminated(&defaults[j - first_default], msg_str);
        }
    }
    if body.varargs {
        if !args.is_empty() {
            msg_str(c", ");
        }
        msg_str(c"...");
    }
    msg_putchar(c_int::from(b')'));

    for (flag, text) in [
        (FuncFlags::ABORT, c" abort"),
        (FuncFlags::RANGE, c" range"),
        (FuncFlags::DICT, c" dict"),
        (FuncFlags::CLOSURE, c" closure"),
    ] {
        if func.has_flag(flag) {
            msg_str(text);
        }
    }

    msg_clr_eos();
    if p_verbose() > 0 {
        last_set_msg(func.script_ctx.get());
    }
    Ok(())
}

/// The definition `:function` is building, and the state its cleanup shares.
///
/// The transpiled body reached four labels (`erret`, `errret_2`,
/// `errret_keep`, `ret_free`), each undoing a little more than the one after
/// it. What they released is owned here, so a refusal only has to say which
/// of them it was; dropping the record does the rest.
///
/// The parse walks the command line by offset, `at`.
struct Definition<'a> {
    /// The command being run.
    excmd: &'a mut ExArg,
    /// Where in the command line the parse has got to.
    at: usize,
    /// The name being defined. `None` for a dictionary function until it is
    /// given a number, and once an existing function is redefined in place.
    name: Option<XString>,
    /// The dictionary entry a `dict.func` name resolved to.
    fudi: Option<FuncDict>,
    /// Where the body starts in the command line, when it is in the command
    /// itself (`exe "func T()\n…\nendfunc"`) rather than read from the
    /// source.
    line_arg: Option<usize>,
    /// The line `get_function_body` read last, which the command may take
    /// over when another command follows `:endfunction` on it.
    line_to_free: Option<XString>,
    /// The argument names, their defaults, and the body, handed to the
    /// function at the end.
    newargs: Vec<Box<[u8]>>,
    default_args: Vec<Box<[u8]>>,
    newlines: Vec<Option<Box<[u8]>>>,
    varargs: bool,
    flags: FuncFlags,
    /// The function itself, once it is found or allocated.
    func: Option<Rc<UserFunc>>,
    /// Whether an existing function of this name is being replaced beside
    /// rather than in place, so the table takes the new function over the
    /// old one's slot.
    overwrite: bool,
    /// Whether a cmdline block was opened for the body being typed in.
    show_block: bool,
}

/// Which of the transpiled cleanup labels a refusal inside the definition
/// jumps to. Both now only drop what the record owns; the distinction is
/// kept because it says whether an existing function was touched.
enum Refusal {
    /// `erret`: unwind the part-built function.
    Unwind,
    /// `errret_keep`: an existing function of this name stays exactly as it
    /// was.
    Keep,
}

/// The number the nameless (dictionary) functions are given.
static func_nr: GlobalCell<c_int> = GlobalCell::new(0);

impl Definition<'_> {
    /// The name being defined, or the empty name.
    fn name_bytes(&self) -> &[u8] {
        self.name.as_deref().unwrap_or_default()
    }

    /// `:function Name(...)`: everything between the name and the table.
    ///
    /// `paren` says the command defines rather than lists.
    fn define(&mut self, paren: bool) {
        if !paren {
            // ":function func": list that one function.
            let name = self.name.clone().unwrap_or_default();
            let _ = list_one_function(self.excmd, &name, self.at);
            return;
        }

        self.at = self.excmd.line.skip_white(self.at);
        if self.excmd.line.byte_at(self.at) != b'(' {
            if !self.excmd.skip {
                let arg = msg_bytes(self.excmd.line.arg());
                semsg!("E124: Missing '(': {arg}");
                return;
            }
            // Attempt to carry on by skipping some text.
            if let Some(paren) = self
                .excmd
                .line
                .rest_of(self.at)
                .iter()
                .position(|&b| b == b'(')
            {
                self.at += paren;
            }
        }
        self.at = self.excmd.line.skip_white(self.at + 1);

        if !self.excmd.skip && self.check_name().is_none() {
            return;
        }
        let _ = self.install();
    }

    /// Whether the name may be defined at all: it has to read as an
    /// identifier, and it may not go in the `g:` dictionary.
    ///
    /// `None` refuses, with the reason already reported.
    fn check_name(&mut self) -> Option<()> {
        // Check the name of the function, unless it is a dictionary
        // function that is being overwritten.
        // A dictionary function defined with bracket notation
        // (`obj['foo-bar']()`) is named by a *dictionary key*, which need
        // not follow the function naming rules, so the identifier check is
        // skipped for it -- and so is the name of an existing entry that
        // holds a Funcref.
        let holds_func = self.fudi.as_ref().is_some_and(|fudi| {
            !fudi.new_key
                && fudi
                    .dict
                    .find(&fudi.key)
                    .is_some_and(|item| item.di_tv.is_func())
        });
        if let Some(name) = self.name.as_deref()
            && !holds_func
            && !reads_as_identifier(name)
        {
            emsg_funcname(e_invarg2, name);
            return None;
        }
        // Disallow using the g: dict.
        if self
            .fudi
            .as_ref()
            .is_some_and(|fudi| fudi.dict.dv_scope == VAR_DEF_SCOPE)
        {
            emsg(gettext(c"E862: Cannot use g: here"));
            return None;
        }
        Some(())
    }

    /// Build the function and put it in the table.
    fn install(&mut self) -> Result<(), Refusal> {
        let skip = self.excmd.skip;
        let mut cursor = Cursor::new(self.excmd.line.rest_of(self.at));
        let parsed = get_function_args(
            &mut cursor,
            b')',
            Some(&mut self.newargs),
            Some(&mut self.varargs),
            Some(&mut self.default_args),
            skip,
        );
        self.at += cursor.offset();
        parsed.map_err(|_| Refusal::Unwind)?;
        if KeyTyped.get() && ui_has(kUICmdline) {
            self.show_block = true;
            ui_ext_cmdline_block_append(0, self.excmd.line.cmd());
        }
        self.build()
    }

    /// The definition proper, from the trailing attributes to the finished
    /// function in the table.
    fn build(&mut self) -> Result<(), Refusal> {
        self.parse_attributes()?;

        // Save the starting line number.
        let sourcing_lnum_top = sourcing_lnum();

        // Do not define the function when reading the body fails, and not
        // when skipping.
        let read = get_function_body(
            self.excmd,
            &mut self.newlines,
            self.line_arg,
            &mut self.line_to_free,
            self.show_block,
        );
        if read.is_err() || self.excmd.skip {
            return Err(Refusal::Unwind);
        }

        self.resolve_target()?;
        if self.func.is_none() {
            self.create()?;
        }
        self.fill_in(sourcing_lnum_top);
        Ok(())
    }

    /// The `range dict abort closure` attributes, and what may follow them:
    /// the body on the same line, a comment, or nothing.
    fn parse_attributes(&mut self) -> Result<(), Refusal> {
        loop {
            self.at = self.excmd.line.skip_white(self.at);
            let attribute = [
                (&b"range"[..], FuncFlags::RANGE),
                (&b"dict"[..], FuncFlags::DICT),
                (&b"abort"[..], FuncFlags::ABORT),
                (&b"closure"[..], FuncFlags::CLOSURE),
            ]
            .into_iter()
            .find(|(word, _)| self.excmd.line.starts_with(self.at, word));
            let Some((word, flag)) = attribute else {
                break;
            };
            self.flags |= flag;
            self.at += word.len();
            if flag == FuncFlags::CLOSURE && current_fc_id().is_none() {
                emsg_funcname(
                    c"E932: Closure function should not be at top level: %s",
                    self.name_bytes(),
                );
                return Err(Refusal::Unwind);
            }
        }

        // A line break means the body follows in the same string, which is
        // what makes `exe "func T()\n...\nendfunc"` work.
        let next = self.excmd.line.byte_at(self.at);
        if next == b'\n' {
            self.line_arg = Some(self.at + 1);
        } else if next != 0 && next != b'"' && !self.excmd.skip && did_emsg.get() == 0 {
            let rest = msg_bytes(self.excmd.line.rest_of(self.at));
            semsg!("E488: Trailing characters: {rest}");
        }

        if KeyTyped.get() {
            self.report_existing();
            if !self.excmd.skip && did_emsg.get() != 0 {
                return Err(Refusal::Unwind);
            }
            if !ui_has(kUICmdline) {
                // Don't overwrite the function name.
                msg_putchar(c_int::from(b'\n'));
            }
            cmdline_row.set(msg_row.get());
        }
        Ok(())
    }

    /// Report a function of this name that already exists, which for a body
    /// being typed in is worth saying before the whole of it is.
    fn report_existing(&self) {
        if self.excmd.skip || self.excmd.forceit {
            return;
        }
        if self.fudi.as_ref().is_some_and(|fudi| !fudi.new_key) {
            emsg(gettext(E_FUNCDICT));
        } else if let Some(name) = self.name.as_deref()
            && func_exists(name)
        {
            emsg_funcname(E_FUNCEXTS, name);
        }
    }

    /// Find the function this definition replaces, or make room for a new
    /// one.
    fn resolve_target(&mut self) -> Result<(), Refusal> {
        if self.fudi.is_some() {
            return self.number_dict_function();
        }
        let name = self.name_bytes();
        if with_var(name, false, |item| item.di_tv.v_type() == VAR_FUNC) == Some(true) {
            emsg_funcname(c"E707: Function name conflicts with variable: %s", name);
            return Err(Refusal::Unwind);
        }
        self.func = find_func(name);
        if let Some(func) = self.func.clone() {
            self.replace_existing(&func)?;
        }
        Ok(())
    }

    /// An existing function of this name: replaced in place, replaced
    /// beside, or left alone with a report.
    fn replace_existing(&mut self, func: &Rc<UserFunc>) -> Result<(), Refusal> {
        // A function can be replaced with "function!" and when sourcing the
        // same script again, but only once.
        let defined = func.script_ctx.get();
        let sctx = current_sctx.get();
        let name = self.name_bytes();
        if !self.excmd.forceit && (defined.sc_sid != sctx.sc_sid || defined.sc_seq == sctx.sc_seq) {
            emsg_funcname(E_FUNCEXTS, name);
            return Err(Refusal::Keep);
        }
        if func.calls.get() > 0 {
            emsg_funcname(c"E127: Cannot redefine function %s: It is in use", name);
            return Err(Refusal::Keep);
        }
        if func.refcount.get().is_shared() {
            // Referenced somewhere: don't redefine it, create a new one
            // beside it.
            func.release();
            func.flags.set(func.flags.get() | FuncFlags::REMOVED);
            self.func = None;
            self.overwrite = true;
        } else {
            // Redefine the existing function; its expanded name stays.
            self.name = None;
            func_clear_items(func);
            let mut prof = func.prof.borrow_mut();
            prof.profiling = false;
            prof.initialized = false;
        }
        Ok(())
    }

    /// A `dict.func` definition: the function is nameless and gets a
    /// sequential number, reachable only through a Funcref.
    fn number_dict_function(&mut self) -> Result<(), Refusal> {
        self.func = None;
        let Some(fudi) = &self.fudi else {
            return Err(Refusal::Unwind);
        };
        if !fudi.new_key && !self.excmd.forceit {
            emsg(gettext(E_FUNCDICT));
            return Err(Refusal::Unwind);
        }
        // Can't add a function to a locked dictionary, and can't change an
        // existing function if it is locked. The entry is found again: the
        // body that was just read may have run code that removed it.
        let lock = match fudi.dict.find(&fudi.key) {
            Some(item) if !fudi.new_key => item.di_lock,
            _ => fudi.dict.dv_lock,
        };
        if value_check_lock_named(lock, self.excmd.line.arg()) {
            return Err(Refusal::Unwind);
        }

        func_nr.set(func_nr.get() + 1);
        self.name = Some(XString::from_bytes(func_nr.get().to_string().as_bytes()));
        Ok(())
    }

    /// Allocate the function, add its dictionary entry when it has one, and
    /// put it in the table.
    fn create(&mut self) -> Result<(), Refusal> {
        let name = self.name.clone().unwrap_or_default();
        if self.fudi.is_none()
            && u8::try_from(AUTOLOAD_CHAR).is_ok_and(|c| name.contains(&c))
            && !self.autoload_name_matches_script(&name)
        {
            let shown = msg_bytes(&name);
            semsg!("E746: Function name does not match script file name: {shown}");
            return Err(Refusal::Unwind);
        }

        let func = Rc::new(alloc_ufunc(&name));

        if self.fudi.is_some() {
            self.add_dict_entry(&name)?;
            // Behave as though "dict" had been used.
            self.flags |= FuncFlags::DICT;
        }

        // Insert the new function in the function list.
        if self.overwrite {
            // The old function keeps living for whoever references it; the
            // table's slot is the new one's.
            replace_func(func.clone());
        } else if add_func(func.clone()).is_err() {
            return Err(Refusal::Unwind);
        }
        func.refcount.set(Refcount::ONE);
        self.func = Some(func);
        Ok(())
    }

    /// Whether an `autoload#name` matches the script file it is being
    /// defined in, which is what makes it findable again.
    fn autoload_name_matches_script(&self, name: &[u8]) -> bool {
        let Some(sourcing_name) = sourcing_name_bytes() else {
            return false;
        };
        let scriptname = autoload_name_of(name);
        // The part from the first separator on: "/name.vim".
        let at = scriptname.iter().position(|&b| b == b'/').unwrap_or(0);
        let suffix = scriptname.cstr_at(at);
        let plen = suffix.to_bytes().len();
        let slen = sourcing_name.len();
        slen > plen
            && cstr::with_terminated(&sourcing_name[slen - plen..], |tail| {
                path_fnamecmp(suffix, tail) == 0
            })
    }

    /// Put a Funcref to the new function in the dictionary entry the name
    /// resolved to, creating the entry when there was none.
    fn add_dict_entry(&mut self, name: &[u8]) -> Result<(), Refusal> {
        let Some(fudi) = &mut self.fudi else {
            return Ok(());
        };
        let (dict, key) = (&mut fudi.dict, &fudi.key[..]);
        // The entry an existing key named is found again, and one that has
        // gone since is added back.
        let existing = !fudi.new_key && dict.find(key).is_some();
        if !existing && dict.add_value(key, TV_INITIAL_VALUE).is_err() {
            return Err(Refusal::Unwind);
        }
        let item = dict.find_mut(key).expect("the entry just found or added");
        // Overwrite the existing dict entry.
        tv_clear(&mut item.di_tv);
        item.di_tv
            .write_func_name(Some(ThinCString::from_bytes(name)));
        Ok(())
    }

    /// Hand the function everything the definition collected.
    fn fill_in(&mut self, sourcing_lnum_top: LineNr) {
        let func = self.func.clone().expect("a function was found or made");
        *func.body.borrow_mut() = Rc::new(FuncBody {
            args: core::mem::take(&mut self.newargs),
            def_args: core::mem::take(&mut self.default_args),
            lines: core::mem::take(&mut self.newlines),
            varargs: self.varargs,
        });
        if self.flags.has(FuncFlags::CLOSURE) {
            register_closure(&func);
        } else {
            func.scoped.set(None);
        }

        if prof_def_func() {
            func_do_profile(&func);
        }
        if sandbox.get() != 0 {
            self.flags |= FuncFlags::SANDBOX;
        }
        func.flags.set(self.flags);
        func.calls.set(0);
        let mut sctx = current_sctx.get();
        sctx.sc_lnum += sourcing_lnum_top;
        set_sctx_from_lua(&mut sctx);
        func.script_ctx.set(sctx);
    }
}

/// Whether `name` is a run of name characters and nothing else, after the
/// `<SNR>123_` mangling a script-local name carries.
///
/// Each byte is read the way the C reads it, through a signed `c_char`, so
/// that a byte over 0x7f asks the character classes about a *negative*
/// number and is refused rather than wrapping into some other class.
fn reads_as_identifier(name: &[u8]) -> bool {
    let mut bytes = name;
    if bytes.first().map(|&b| c_int::from(b)) == Some(K_SPECIAL) {
        // Skip the mangling: `<SNR>`, a script number, and an underscore.
        // Upstream steps three bytes in when there is no underscore, which
        // is off the end of a name that short; an empty tail refuses.
        bytes = match bytes.iter().position(|&b| b == b'_') {
            Some(at) => &bytes[at + 1..],
            None => bytes.get(3..).unwrap_or_default(),
        };
    }
    bytes.iter().enumerate().all(|(i, &b)| {
        let c = c_int::from(c_char::from_ne_bytes([b]));
        if i == 0 {
            eval_isnamec1(c)
        } else {
            eval_isnamec(c)
        }
    })
}

/// `:function`.
///
/// Four commands in one: with no argument it lists everything, with a
/// `/pattern/` it lists the matches, with a bare name it lists that one, and
/// with a `(` it defines.
pub fn ex_function(excmd: &mut ExArg) {
    // ":function" without argument: list functions.
    if ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.arg))) != 0 {
        if !excmd.skip {
            list_functions(None);
        }
        excmd.line.next = excmd.line.check_next(excmd.line.arg);
        return;
    }

    // ":function /pat": list functions matching the pattern.
    if excmd.line.byte_at(excmd.line.arg) == b'/' {
        let at = list_functions_matching_pat(excmd);
        excmd.line.next = excmd.line.check_next(at);
        return;
    }

    // Get the function name.  There are these situations:
    //   func       a normal function name: "name" == func, no dict
    //   dict.func  a new dictionary entry: "name" == NULL, a dict entry
    //              with `new_key`
    //   dict.func  an existing entry holding a Funcref: "name" == func and
    //              a dict entry
    //   dict.func  an existing entry that is not a Funcref: "name" == NULL
    //              and a dict entry
    //   s:func     a script-local name; g:func is the same as func
    let arg = excmd.line.arg;
    let FunctionName {
        name, end, dict, ..
    } = save_function_name(excmd.line.rest_of(arg), excmd.skip, TFN_NO_AUTOLOAD, true);
    let at = arg + end;
    let paren = excmd.line.rest_of(at).contains(&b'(');
    if name.is_none() && (dict.is_none() || !paren) && !excmd.skip {
        // Return on an invalid expression in braces, unless the evaluation
        // was cancelled by an aborting error, an interrupt or an exception.
        if !aborting() {
            if let Some(FuncDict {
                key, new_key: true, ..
            }) = &dict
            {
                let key = msg_bytes(key);
                semsg!("E716: Key not present in Dictionary: \"{key}\"");
            }
            return;
        }
        excmd.skip = true;
    }

    // An error in a function call while evaluating an expression in magic
    // braces should not stop the function being defined.
    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);

    let mut definition = Definition {
        excmd,
        at,
        name,
        fudi: dict,
        line_arg: None,
        line_to_free: None,
        newargs: Vec::new(),
        default_args: Vec::new(),
        newlines: Vec::new(),
        varargs: false,
        flags: FuncFlags::NONE,
        func: None,
        overwrite: false,
        show_block: false,
    };
    definition.define(paren);

    // ret_free: what every path above leaves is the record's, and goes with
    // it.
    let show_block = definition.show_block;
    drop(definition);
    did_emsg.set(did_emsg.get() | saved_did_emsg);
    if show_block {
        ui_ext_cmdline_block_leave();
    }
}
