//! `:function` itself -- defining, and the header a listing prints.
//!
//! `ex_function` decides which of the four things the command is (define,
//! list one, list a pattern, list everything), builds the `UserFunc` and
//! installs it in the table.  `list_func_head` prints the `function
//! Name(a, b = 1, ...) dict abort range` line, which is the same text in a
//! listing and in a `:verbose` report.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::snprintf;
use crate::strings::has_char;
use crate::strings::vim_strchr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::size_of_val;
use core::ptr;

use super::*;
use crate::eval::typval::value_check_lock_named;
use crate::eval::vars::with_var;
use crate::memory::XString;
use crate::types::{FAIL, Failed, NUL, Refcount};

/// Whether the function table changed under a listing, which means the
/// `UserFunc` the caller is holding may be gone.  Reports E454 when it did.
pub(crate) fn function_list_modified(prev_ht_changed: c_int) -> c_int {
    if prev_ht_changed != func_table().changed() {
        emsg(gettext(E_FUNCTION_LIST_WAS_MODIFIED));
        return 1;
    }
    0
}

/// The name to show a user: the unmangled `<SNR>123_name` when there is one.
///
/// # Safety
/// `func` is a live function.
pub(crate) unsafe fn printable_func_name(func: *mut UserFunc) -> *mut c_char {
    // SAFETY: the caller's promise -- `func` is a live function.
    let f = unsafe { Uf::new(func) };
    if !f.uf_name_exp.is_null() {
        f.uf_name_exp
    } else {
        uf_name_ptr(func)
    }
}

/// Print `function Name(a, b = 1, ...) range dict abort closure`, the head of
/// a listing and of a `:verbose` report.
///
/// # Safety
/// `func` is a live function.
pub(crate) unsafe fn list_func_head(
    func: *mut UserFunc,
    indent: bool,
    force: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `func` is a live function.
    let f = unsafe { Uf::new(func) };
    let prev_ht_changed = func_table().changed();

    msg_start();

    // Check no function was added or removed from a callback, as
    // `msg_start` may have invoked a redraw.
    if function_list_modified(prev_ht_changed) != 0 {
        return Err(Failed);
    }

    if indent {
        msg_str(c"   ");
    }
    let intro = if force {
        c"function! ".as_ptr()
    } else {
        c"function ".as_ptr()
    };
    msg_str(unsafe { cstr::at(intro) });
    msg_str(unsafe { cstr::at(printable_func_name(func)) });
    msg_putchar(b'(' as c_int);

    let args = ga_strings(&f.uf_args);
    let defaults = ga_strings(&f.uf_def_args);
    // The defaults are right-aligned with the arguments: the last
    // `defaults.len()` arguments are the ones that have one.
    // Upstream computes this in `int`, where a (impossible) surplus of
    // defaults would go negative and give every argument one.
    let first_default = args.len().saturating_sub(defaults.len());
    for (j, &arg) in args.iter().enumerate() {
        if j != 0 {
            msg_str(c", ");
        }
        msg_str(unsafe { cstr::at(arg) });
        if j >= first_default {
            msg_str(c" = ");
            msg_str(unsafe { cstr::at(defaults[j - first_default]) });
        }
    }
    if f.uf_varargs != 0 {
        if !args.is_empty() {
            msg_str(c", ");
        }
        msg_str(c"...");
    }
    msg_putchar(b')' as c_int);

    for (flag, text) in [
        (FuncFlags::ABORT, c" abort"),
        (FuncFlags::RANGE, c" range"),
        (FuncFlags::DICT, c" dict"),
        (FuncFlags::CLOSURE, c" closure"),
    ] {
        if f.uf_flags.has(flag) {
            msg_str(text);
        }
    }

    msg_clr_eos();
    if p_verbose() > 0 {
        last_set_msg(f.uf_script_ctx);
    }
    Ok(())
}

/// The definition `:function` is building, and the state its cleanup shares.
///
/// The transpiled body reached four labels (`erret`, `errret_2`,
/// `errret_keep`, `ret_free`), each undoing a little more than the one after
/// it, over a dozen locals that outlived the jumps. Those locals are these
/// fields, and each label is the tail of the method whose refusal names it.
///
/// The parse walks the command line by offset, `at`.
struct Definition<'a> {
    /// The command being run.
    excmd: &'a mut ExArg,
    /// Where in the command line the parse has got to.
    at: usize,
    /// The name being defined, owned until it is handed to the function.
    /// Null for a dictionary function, which is given a number instead.
    name: *mut c_char,
    /// The dictionary entry a `dict.func` name resolved to.
    fudi: Option<FuncDict>,
    /// The body's first line, when it is in the command itself
    /// (`exe "func T()\n…\nendfunc"`) rather than read from the source.
    line_arg: *mut c_char,
    /// The buffer `get_function_body` read the body into, which is this
    /// frame's to release however the definition ends.
    line_to_free: Option<XString>,
    /// The argument names, their defaults, and the body: filled in here and
    /// handed to the function, which is why every refusal below has to say
    /// whether they were.
    newargs: GArray,
    default_args: GArray,
    newlines: GArray,
    varargs: c_int,
    flags: FuncFlags,
    /// The function itself, once it is found or allocated.
    func: *mut UserFunc,
    /// Whether `func` is this frame's to free: it was allocated but never
    /// made it into the table.
    free_func: bool,
    /// Whether an existing function of this name is being replaced beside
    /// rather than in place, so the table takes the new name over the old.
    overwrite: bool,
    /// Whether a cmdline block was opened for the body being typed in.
    show_block: bool,
}

/// Which of the transpiled cleanup labels a refusal inside the definition
/// jumps to, as what each of them actually does.
enum Refusal {
    /// `erret`: unwind the part-built function -- the argument arrays it was
    /// handed go back to being empty, and its expanded name is released.
    Unwind,
    /// `errret_keep`: an existing function of this name stays exactly as it
    /// was, so nothing of it is touched on the way out.
    Keep,
}

/// The number the nameless (dictionary) functions are given.
static func_nr: GlobalCell<c_int> = GlobalCell::new(0);

impl Definition<'_> {
    /// `:function Name(...)`: everything between the name and the table.
    ///
    /// `paren` says the command defines rather than lists.
    fn define(&mut self, paren: bool) {
        if !paren {
            // ":function func": list that one function.
            // SAFETY: a null name is no name; otherwise it is this record's
            // own NUL-terminated string.
            let name = unsafe { cstr::at_opt(self.name) }
                .map(CStr::to_bytes)
                .unwrap_or_default();
            let _ = list_one_function(self.excmd, name, self.at);
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

        let slot = size_of::<*mut c_char>() as c_int;
        // SAFETY: both arrays are this record's own.
        unsafe { ga_init(&raw mut self.newargs, slot, 3) };
        // SAFETY: as above.
        unsafe { ga_init(&raw mut self.newlines, slot, 3) };

        if !self.excmd.skip && self.check_name().is_none() {
            return;
        }
        if self.install().is_none() {
            // errret_keep: the function did not take the three arrays over,
            // so they are still this frame's to release.
            // SAFETY: all three are this record's own.
            unsafe { ga_clear_strings(&raw mut self.newargs) };
            // SAFETY: as above.
            unsafe { ga_clear_strings(&raw mut self.default_args) };
            // SAFETY: as above.
            unsafe { ga_clear_strings(&raw mut self.newlines) };
        }
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
        // SAFETY: a null name is no name; otherwise it is this record's own
        // NUL-terminated string.
        let name = unsafe { cstr::at_opt(self.name) }.map(CStr::to_bytes);
        if let Some(name) = name
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
    ///
    /// `None` says the three argument arrays were not handed over and are
    /// still the caller's to release.
    fn install(&mut self) -> Option<()> {
        let names = &raw mut self.newargs;
        let (varp, defs) = (&raw mut self.varargs, &raw mut self.default_args);
        let skip = self.excmd.skip;
        let mut cursor = Cursor::new(self.excmd.line.rest_of(self.at));
        // SAFETY: the three out-parameters are this record's own.
        let parsed = unsafe { get_function_args(&mut cursor, b')', names, varp, defs, skip) };
        self.at += cursor.offset();
        if parsed.is_ok() {
            if KeyTyped.get() && ui_has(kUICmdline) {
                self.show_block = true;
                // SAFETY: the live command's own text.
                ui_ext_cmdline_block_append(0, self.excmd.line.cmd());
            }
            match self.build() {
                Ok(()) => return Some(()),
                Err(Refusal::Keep) => return None,
                Err(Refusal::Unwind) => {}
            }
            // erret: the arrays below were handed to the function, and are
            // cleared by the caller, so give it empty ones.
            if !self.func.is_null() {
                let slot = size_of::<*mut c_char>() as c_int;
                // SAFETY: `func` is the function just allocated.
                unsafe { ga_init(&raw mut (*self.func).uf_args, slot, 1) };
                // SAFETY: as above.
                unsafe { ga_init(&raw mut (*self.func).uf_def_args, slot, 1) };
            }
        }

        // errret_2:
        if !self.func.is_null() {
            // SAFETY: as above -- the expanded name is the function's own.
            unsafe { xfree((*self.func).uf_name_exp as *mut c_void) };
            // SAFETY: as above.
            unsafe { (*self.func).uf_name_exp = ptr::null_mut() };
        }
        if self.free_func {
            // SAFETY: allocated here and never installed.
            unsafe { xfree(self.func as *mut c_void) };
        }
        None
    }

    /// The definition proper, from the trailing attributes to the finished
    /// function in the table.
    fn build(&mut self) -> Result<(), Refusal> {
        self.parse_attributes()?;

        // Save the starting line number.
        let sourcing_lnum_top = sourcing_lnum();

        // Do not define the function when reading the body fails, and not
        // when skipping.
        let (lines, freep) = (&raw mut self.newlines, &mut self.line_to_free);
        let (line_arg, block) = (self.line_arg, self.show_block);
        // SAFETY: `lines` is this record's own array, and not the command.
        let read = unsafe { get_function_body(self.excmd, lines, line_arg, freep, block) };
        if read == FAIL || self.excmd.skip {
            return Err(Refusal::Unwind);
        }

        let namelen = self.resolve_target()?;
        if self.func.is_null() {
            self.create(namelen)?;
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
            if flag == FuncFlags::CLOSURE && current_fc().is_null() {
                // SAFETY: a null name is no name; otherwise it is this
                // record's own NUL-terminated string.
                let what = unsafe { cstr::at_opt(self.name) }
                    .map(CStr::to_bytes)
                    .unwrap_or_default();
                emsg_funcname(
                    c"E932: Closure function should not be at top level: %s",
                    what,
                );
                return Err(Refusal::Unwind);
            }
        }

        // A line break means the body follows in the same string, which is
        // what makes `exe "func T()\n...\nendfunc"` work.
        let next = self.excmd.line.byte_at(self.at);
        if next == b'\n' {
            self.line_arg = self.excmd.line.ptr_at(self.at + 1);
        } else if next != NUL as u8 && next != b'"' && !self.excmd.skip && did_emsg.get() == 0 {
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
                msg_putchar(b'\n' as c_int);
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
        // SAFETY: a null name is no name; otherwise it is this record's own
        // NUL-terminated string.
        let name = unsafe { cstr::at_opt(self.name) }.map(CStr::to_bytes);
        if self.fudi.as_ref().is_some_and(|fudi| !fudi.new_key) {
            emsg(gettext(E_FUNCDICT));
        } else if let Some(name) = name
            && !find_func(name).is_null()
        {
            emsg_funcname(E_FUNCEXTS, name);
        }
    }

    /// Find the function this definition replaces, or make room for a new
    /// one, answering the length of the name it is to carry.
    fn resolve_target(&mut self) -> Result<size_t, Refusal> {
        if self.fudi.is_some() {
            return self.number_dict_function();
        }
        // SAFETY: the NUL-terminated name being defined.
        let name = unsafe { cstr::bytes_at(self.name) };
        if with_var(name, false, |item| item.di_tv.v_type() == VAR_FUNC) == Some(true) {
            emsg_funcname(c"E707: Function name conflicts with variable: %s", name);
            return Err(Refusal::Unwind);
        }
        self.func = find_func(name);
        if !self.func.is_null() {
            self.replace_existing()?;
        }
        Ok(0)
    }

    /// An existing function of this name: replaced in place, replaced
    /// beside, or left alone with a report.
    fn replace_existing(&mut self) -> Result<(), Refusal> {
        let func = self.func;
        // A function can be replaced with "function!" and when sourcing the
        // same script again, but only once.
        // SAFETY, for every region below: `func` is the live function the
        // table answered, and `name` its NUL-terminated name.
        let (sid, seq) = unsafe { ((*func).uf_script_ctx.sc_sid, (*func).uf_script_ctx.sc_seq) };
        let sctx = current_sctx.get();
        let name = unsafe { cstr::bytes_at(self.name) };
        if !self.excmd.forceit && (sid != sctx.sc_sid || seq == sctx.sc_seq) {
            emsg_funcname(E_FUNCEXTS, name);
            return Err(Refusal::Keep);
        }
        if unsafe { (*func).uf_calls } > 0 {
            emsg_funcname(c"E127: Cannot redefine function %s: It is in use", name);
            return Err(Refusal::Keep);
        }
        if unsafe { (*func).uf_refcount }.is_shared() {
            // Referenced somewhere: don't redefine it, create a new one
            // beside it.
            unsafe { (*func).uf_refcount.release() };
            // SAFETY: as above.
            unsafe { (*func).uf_flags |= FuncFlags::REMOVED };
            self.func = ptr::null_mut();
            self.overwrite = true;
        } else {
            // Redefine the existing function, keeping its expanded name
            // across the clear.
            let exp_name = unsafe { (*func).uf_name_exp };
            // SAFETY: as above -- the old name is this record's own.
            unsafe { xfree(self.name as *mut c_void) };
            self.name = ptr::null_mut();
            // SAFETY: as above.
            unsafe { (*func).uf_name_exp = ptr::null_mut() };
            // SAFETY: as above.
            unsafe { func_clear_items(func) };
            // SAFETY: as above.
            unsafe { (*func).uf_name_exp = exp_name };
            // SAFETY: as above.
            unsafe { (*func).uf_profiling = 0 };
            // SAFETY: as above.
            unsafe { (*func).uf_prof_initialized = 0 };
        }
        Ok(())
    }

    /// A `dict.func` definition: the function is nameless and gets a
    /// sequential number, reachable only through a Funcref.
    fn number_dict_function(&mut self) -> Result<size_t, Refusal> {
        self.func = ptr::null_mut();
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

        let mut numbuf: [c_char; 65] = [0; 65];
        // SAFETY: the old name is this record's own.
        unsafe { xfree(self.name as *mut c_void) };
        func_nr.set(func_nr.get() + 1);
        let (into, cap) = (numbuf.as_mut_ptr(), size_of_val(&numbuf));
        // SAFETY: `numbuf` is this frame's own, and `cap` its size.
        let namelen = unsafe { snprintf!(into, cap, c"%d".as_ptr(), func_nr.get()) } as size_t;
        // SAFETY: the copy is of what was just rendered into `numbuf`.
        let owned = unsafe { xmemdupz(numbuf.as_ptr() as *const c_void, namelen) };
        self.name = owned as *mut c_char;
        Ok(namelen)
    }

    /// Allocate the function, add its dictionary entry when it has one, and
    /// put it in the table.
    fn create(&mut self, mut namelen: size_t) -> Result<(), Refusal> {
        // SAFETY: `name` is the NUL-terminated name being defined.
        if self.fudi.is_none()
            && has_char(unsafe { cstr::at(self.name) }, AUTOLOAD_CHAR)
            && !self.autoload_name_matches_script()
        {
            // SAFETY: as above.
            let shown = unsafe { c_str(self.name) };
            semsg!("E746: Function name does not match script file name: {shown}");
            return Err(Refusal::Unwind);
        }

        if namelen == 0 {
            // SAFETY: as above.
            namelen = unsafe { cstr::bytes_at(self.name) }.len();
        }
        // SAFETY: as above.
        self.func = unsafe { alloc_ufunc(self.name, namelen) };

        if self.fudi.is_some() {
            self.add_dict_entry(namelen)?;
            // Behave as though "dict" had been used.
            self.flags |= FuncFlags::DICT;
        }

        // Insert the new function in the function list.
        if self.overwrite {
            // SAFETY: `name` is the name the table is keyed on, and `func`
            // the function that now carries it.
            let hi = unsafe { func_table().find(self.name) };
            // SAFETY: as above.
            unsafe { func_table().set_key(hi, uf_name_ptr(self.func)) };
        // SAFETY: as above.
        } else if unsafe { func_table().add(uf_name_ptr(self.func)) }.is_err() {
            self.free_func = true;
            return Err(Refusal::Unwind);
        }
        // SAFETY: the function just installed.
        unsafe { (*self.func).uf_refcount = Refcount::ONE };
        Ok(())
    }

    /// Whether an `autoload#name` matches the script file it is being
    /// defined in, which is what makes it findable again.
    fn autoload_name_matches_script(&self) -> bool {
        let sourcing_name = sourcing_entry().es_name;
        if sourcing_name.is_null() {
            return false;
        }
        // SAFETY, for every region below: `name` is NUL-terminated, so is
        // the script name, `autoload_name` answers an owned string, and the
        // tail compared is inside the script name because it is shorter.
        let name_len = unsafe { cstr::bytes_at(self.name) }.len();
        // SAFETY: as above.
        let scriptname = unsafe { autoload_name(self.name, name_len) };
        // SAFETY: as above.
        let suffix = unsafe { vim_strchr(scriptname, b'/' as c_int) };
        // SAFETY: as above.
        let plen = unsafe { cstr::bytes_at(suffix) }.len() as isize;
        // SAFETY: as above.
        let slen = unsafe { cstr::bytes_at(sourcing_name) }.len() as isize;
        // SAFETY: as above.
        let tail = || unsafe { cstr::at(sourcing_name.offset(slen - plen)) };
        // SAFETY: as above.
        let matches = slen > plen && unsafe { path_fnamecmp(cstr::at(suffix), tail()) } == 0;
        // SAFETY: as above -- the name is this frame's own.
        unsafe { xfree(scriptname as *mut c_void) };
        matches
    }

    /// Put a Funcref to the new function in the dictionary entry the name
    /// resolved to, creating the entry when there was none.
    fn add_dict_entry(&mut self, namelen: size_t) -> Result<(), Refusal> {
        let Some(fudi) = &mut self.fudi else {
            return Ok(());
        };
        let (dict, key) = (&mut fudi.dict, &fudi.key[..]);
        // The entry an existing key named is found again, and one that has
        // gone since is added back.
        let existing = !fudi.new_key && dict.find(key).is_some();
        if !existing && dict.add_value(key, TV_INITIAL_VALUE).is_err() {
            // SAFETY: `func` was allocated just now and never installed.
            unsafe { xfree(self.func as *mut c_void) };
            self.func = ptr::null_mut();
            return Err(Refusal::Unwind);
        }
        let item = dict.find_mut(key).expect("the entry just found or added");
        // Overwrite the existing dict entry.
        tv_clear(&mut item.di_tv);
        // SAFETY: `name` is the NUL-terminated name being defined.
        let owned = unsafe { xmemdupz(self.name as *const c_void, namelen) } as *mut c_char;
        item.di_tv.write_func_name(owned);
        Ok(())
    }

    /// Hand the function everything the definition collected.
    fn fill_in(&mut self, sourcing_lnum_top: LineNr) {
        let func = self.func;
        // SAFETY, for every region below: `func` is the function just found
        // or allocated, and the three arrays are handed over to it.
        unsafe { (*func).uf_args = self.newargs };
        // SAFETY: as above.
        unsafe { (*func).uf_def_args = self.default_args };
        // SAFETY: as above.
        unsafe { (*func).uf_lines = self.newlines };
        if self.flags.has(FuncFlags::CLOSURE) {
            // SAFETY: as above.
            unsafe { register_closure(func) };
        } else {
            // SAFETY: as above.
            unsafe { (*func).uf_scoped = ptr::null_mut() };
        }

        // SAFETY: as above.
        if prof_def_func() {
            // SAFETY: as above.
            unsafe { func_do_profile(func) };
        }
        if sandbox.get() != 0 {
            self.flags |= FuncFlags::SANDBOX;
        }
        // SAFETY: as above.
        unsafe { (*func).uf_varargs = self.varargs };
        // SAFETY: as above.
        unsafe { (*func).uf_flags = self.flags };
        // SAFETY: as above.
        unsafe { (*func).uf_calls = 0 };
        // SAFETY: as above.
        unsafe { (*func).uf_script_ctx = current_sctx.get() };
        // SAFETY: as above.
        unsafe { (*func).uf_script_ctx.sc_lnum += sourcing_lnum_top };
        // SAFETY: as above.
        unsafe { nlua_set_sctx(&raw mut (*func).uf_script_ctx) };
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
        let c = c_int::from(b as c_char);
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
    // SAFETY: the caller's promise -- `excmd` is the Ex command being run.

    // ":function" without argument: list functions.
    if ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.arg))) != 0 {
        if !excmd.skip {
            // SAFETY: no pattern means every function.
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
        name: name.map_or(ptr::null_mut(), XString::into_raw),
        fudi: dict,
        line_arg: ptr::null_mut(),
        line_to_free: None,
        newargs: GArray::EMPTY,
        default_args: GArray::EMPTY,
        newlines: GArray::EMPTY,
        varargs: 0,
        flags: FuncFlags::NONE,
        func: ptr::null_mut(),
        free_func: false,
        overwrite: false,
        show_block: false,
    };
    definition.define(paren);

    // ret_free: what every path above leaves for this frame to release.
    // SAFETY: the definition's own name, and null is fine for `xfree`.
    unsafe { xfree(definition.name as *mut c_void) };
    did_emsg.set(did_emsg.get() | saved_did_emsg);
    if definition.show_block {
        ui_ext_cmdline_block_leave();
    }
}
