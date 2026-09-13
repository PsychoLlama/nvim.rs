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
use crate::semsg;
use crate::strings::has_char;
use crate::strings::vim_strchr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::size_of_val;
use core::ptr;

use super::*;
use crate::eval::Cur;
use crate::eval::typval::{DictTab, tv_dict_item_free};
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
    if p_verbose.get() > 0 {
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
/// The same shape as [`Live<T>`](crate::winlayer::Live): whoever builds one
/// promises that `ea`/`args` name the live `:function` command being run and
/// that `cursor` walks its NUL-terminated argument, so every method below is
/// ordinary checked code resting on that.
struct Definition {
    /// The command being run.
    ea: Ea,
    /// The same, for the callees that still take the pointer.
    args: *mut ExArg,
    /// Where in the command's argument the parse has got to.
    cursor: Cur,
    /// The name being defined, owned until it is handed to the function.
    /// Null for a dictionary function, which is given a number instead.
    name: *mut c_char,
    /// The dictionary entry a `dict.func` name resolved to.
    fudi: FuncDict,
    /// The body's first line, when it is in the command itself
    /// (`exe "func T()\n…\nendfunc"`) rather than read from the source.
    line_arg: *mut c_char,
    /// The buffer `get_function_body` read the body into, which is this
    /// frame's to release however the definition ends.
    line_to_free: *mut c_char,
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

impl Definition {
    /// `:function Name(...)`: everything between the name and the table.
    ///
    /// `paren` says the command defines rather than lists.
    fn define(&mut self, paren: bool) {
        if !paren {
            // ":function func": list that one function.
            // SAFETY: the live command, and the name and cursor into it.
            let _ = unsafe { list_one_function(self.args, self.name, self.cursor.get()) };
            return;
        }

        self.cursor.skip(0);
        if self.cursor.byte() != b'(' {
            if self.ea.skip == 0 {
                // SAFETY: a message argument the caller holds as a
                // NUL-terminated string.
                let arg = unsafe { c_str(self.ea.arg) };
                semsg!("E124: Missing '(': {arg}");
                return;
            }
            // Attempt to carry on by skipping some text.
            // SAFETY: the cursor walks the command's NUL-terminated argument.
            if has_char(unsafe { cstr::at(self.cursor.get()) }, b'(' as c_int) {
                // SAFETY: as above.
                self.cursor
                    .set(unsafe { vim_strchr(self.cursor.get(), b'(' as c_int) });
            }
        }
        self.cursor.skip(1);

        let slot = size_of::<*mut c_char>() as c_int;
        // SAFETY: both arrays are this record's own.
        unsafe { ga_init(&raw mut self.newargs, slot, 3) };
        // SAFETY: as above.
        unsafe { ga_init(&raw mut self.newlines, slot, 3) };

        if self.ea.skip == 0 && self.check_name().is_none() {
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
        let arg = if self.name.is_null() {
            self.fudi.fd_newkey
        } else {
            self.name
        };
        // A dictionary function defined with bracket notation
        // (`obj['foo-bar']()`) is named by a *dictionary key*, which need
        // not follow the function naming rules, so the identifier check is
        // skipped for it.
        // SAFETY, for every region below: `fd_di` is a live item when it is
        // not null, and `arg` is a NUL-terminated name.
        let named = !arg.is_null()
            && (self.fudi.fd_di.is_null() || !unsafe { (*self.fudi.fd_di).di_tv.is_func() })
            && arg != self.fudi.fd_newkey;
        // SAFETY: as above -- `arg` is one of the two NUL-terminated names.
        if named && !reads_as_identifier(unsafe { cstr::at(arg) }) {
            // SAFETY: as above; the format takes one string.
            unsafe { emsg_funcname(e_invarg2.as_ptr(), arg) };
            return None;
        }
        // Disallow using the g: dict.
        if !self.fudi.fd_dict.is_null() && unsafe { (*self.fudi.fd_dict).dv_scope } == VAR_DEF_SCOPE
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
        let (argp, names) = (self.cursor.raw(), &raw mut self.newargs);
        let (varp, defs) = (&raw mut self.varargs, &raw mut self.default_args);
        let skip = self.ea.skip != 0;
        // SAFETY: the cursor walks the command's argument and the three
        // out-parameters are this record's own.
        let parsed = unsafe { get_function_args(argp, b')' as c_char, names, varp, defs, skip) };
        if parsed.is_ok() {
            if KeyTyped.get() && ui_has(kUICmdline) {
                self.show_block = true;
                // SAFETY: the live command's own text.
                unsafe { ui_ext_cmdline_block_append(0, self.ea.cmd) };
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
        let (lines, freep) = (&raw mut self.newlines, &raw mut self.line_to_free);
        // SAFETY: the live command, and both out-parameters are this
        // record's own.
        let (args, line_arg, block) = (self.args, self.line_arg, self.show_block);
        // SAFETY: as above.
        let read = unsafe { get_function_body(args, lines, line_arg, freep, block) };
        if read == FAIL || self.ea.skip != 0 {
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
            self.cursor.skip(0);
            // SAFETY, for every region in this loop: the cursor walks the
            // command's NUL-terminated argument, and `starts_with` stops at
            // the terminator.
            let attribute = [
                (&b"range"[..], FuncFlags::RANGE),
                (&b"dict"[..], FuncFlags::DICT),
                (&b"abort"[..], FuncFlags::ABORT),
                (&b"closure"[..], FuncFlags::CLOSURE),
            ]
            .into_iter()
            .find(|(word, _)| unsafe { cstr::starts_with(self.cursor.get(), word) });
            let Some((word, flag)) = attribute else {
                break;
            };
            self.flags |= flag;
            self.cursor.bump(word.len());
            if flag == FuncFlags::CLOSURE && current_funccal.get().is_null() {
                let what = if self.name.is_null() {
                    c"".as_ptr()
                } else {
                    self.name.cast_const()
                };
                let fmt = c"E932: Closure function should not be at top level: %s";
                // SAFETY: a format taking one NUL-terminated string, and a
                // name that is one.
                unsafe { emsg_funcname(fmt.as_ptr(), what) };
                return Err(Refusal::Unwind);
            }
        }

        // A line break means the body follows in the same string, which is
        // what makes `exe "func T()\n...\nendfunc"` work.
        if self.cursor.byte() == b'\n' {
            // SAFETY: the byte after a newline is inside the string.
            self.line_arg = unsafe { self.cursor.get().add(1) };
        } else if self.cursor.byte() != NUL as u8
            && self.cursor.byte() != b'"'
            && self.ea.skip == 0
            && did_emsg.get() == 0
        {
            // SAFETY: the cursor walks a NUL-terminated string.
            let p = unsafe { c_str(self.cursor.get()) };
            semsg!("E488: Trailing characters: {p}");
        }

        if KeyTyped.get() {
            self.report_existing();
            if self.ea.skip == 0 && did_emsg.get() != 0 {
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
        if self.ea.skip != 0 || self.ea.forceit != 0 {
            return;
        }
        if !self.fudi.fd_dict.is_null() && self.fudi.fd_newkey.is_null() {
            emsg(gettext(E_FUNCDICT));
        // SAFETY: a NUL-terminated function name.
        } else if !self.name.is_null() && !unsafe { find_func(self.name) }.is_null() {
            // SAFETY: as above -- the format takes one string.
            unsafe { emsg_funcname(E_FUNCEXTS.as_ptr(), self.name) };
        }
    }

    /// Find the function this definition replaces, or make room for a new
    /// one, answering the length of the name it is to carry.
    fn resolve_target(&mut self) -> Result<size_t, Refusal> {
        if !self.fudi.fd_dict.is_null() {
            return self.number_dict_function();
        }
        let mut ht: *mut DictTab = ptr::null_mut();
        // SAFETY, for every region below: `name` is the NUL-terminated name
        // being defined, and `ht` is this frame's own.
        let name_len = unsafe { cstr::bytes_at(self.name) }.len();
        let v = unsafe { find_var(self.name, name_len, &raw mut ht, false) };
        if !v.is_null() && unsafe { (*v).di_tv.v_type() } == VAR_FUNC {
            let clash = c"E707: Function name conflicts with variable: %s";
            unsafe { emsg_funcname(clash.as_ptr(), self.name) };
            return Err(Refusal::Unwind);
        }
        self.func = unsafe { find_func(self.name) };
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
        if self.ea.forceit == 0 && (sid != sctx.sc_sid || seq == sctx.sc_seq) {
            unsafe { emsg_funcname(E_FUNCEXTS.as_ptr(), self.name) };
            return Err(Refusal::Keep);
        }
        if unsafe { (*func).uf_calls } > 0 {
            let busy = c"E127: Cannot redefine function %s: It is in use";
            unsafe { emsg_funcname(busy.as_ptr(), self.name) };
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
        if self.fudi.fd_newkey.is_null() && self.ea.forceit == 0 {
            emsg(gettext(E_FUNCDICT));
            return Err(Refusal::Unwind);
        }
        // SAFETY: `fd_dict` is the live dictionary and `fd_di` its item when
        // it is not null; `ea.arg` names the command for the message.
        let locked = unsafe {
            if self.fudi.fd_di.is_null() {
                // Can't add a function to a locked dictionary.
                value_check_lock((*self.fudi.fd_dict).dv_lock, self.ea.arg, TV_CSTRING)
            } else {
                // Can't change an existing function if it is locked.
                value_check_lock((*self.fudi.fd_di).di_lock, self.ea.arg, TV_CSTRING)
            }
        };
        if locked {
            return Err(Refusal::Unwind);
        }

        let mut numbuf: [c_char; 65] = [0; 65];
        // SAFETY: the old name is this record's own.
        unsafe { xfree(self.name as *mut c_void) };
        func_nr.set(func_nr.get() + 1);
        let (into, cap) = (numbuf.as_mut_ptr(), size_of_val(&numbuf));
        // SAFETY: `numbuf` is this frame's own, and `cap` its size.
        let namelen = unsafe { snprintf(into, cap, c"%d".as_ptr(), func_nr.get()) } as size_t;
        // SAFETY: the copy is of what was just rendered into `numbuf`.
        let owned = unsafe { xmemdupz(numbuf.as_ptr() as *const c_void, namelen) };
        self.name = owned as *mut c_char;
        Ok(namelen)
    }

    /// Allocate the function, add its dictionary entry when it has one, and
    /// put it in the table.
    fn create(&mut self, mut namelen: size_t) -> Result<(), Refusal> {
        // SAFETY: `name` is the NUL-terminated name being defined.
        if self.fudi.fd_dict.is_null()
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

        if !self.fudi.fd_dict.is_null() {
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
        if self.fudi.fd_di.is_null() {
            // Add a new dict entry.
            // SAFETY: `fd_newkey` is the owned key text and `fd_dict` the
            // live dictionary.
            self.fudi.fd_di = unsafe { tv_dict_item_alloc(self.fudi.fd_newkey) };
            // SAFETY: as above -- the item is freed again if it is refused.
            if unsafe { (*self.fudi.fd_dict).add_item(self.fudi.fd_di) }.is_err() {
                // SAFETY: as above.
                unsafe { tv_dict_item_free(self.fudi.fd_di) };
                // SAFETY: `func` was allocated just now and never installed.
                unsafe { xfree(self.func as *mut c_void) };
                self.func = ptr::null_mut();
                return Err(Refusal::Unwind);
            }
        } else {
            // Overwrite the existing dict entry.
            // SAFETY: `fd_di` is the live item.
            unsafe { tv_clear(&mut (*self.fudi.fd_di).di_tv) };
        }
        // SAFETY: `name` is the NUL-terminated name being defined.
        let owned = unsafe { xmemdupz(self.name as *const c_void, namelen) } as *mut c_char;
        // SAFETY: the item is live and takes the copy over.
        unsafe { (*self.fudi.fd_di).di_tv.write_func_name(owned) };
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
fn reads_as_identifier(name: &CStr) -> bool {
    let mut bytes = name.to_bytes();
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
///
/// # Safety
/// `args` is a live `:function` command.
pub unsafe fn ex_function(args: *mut ExArg) {
    // SAFETY: the caller's promise -- `args` is the Ex command being run.
    let mut ea = unsafe { Ea::new(args) };

    // ":function" without argument: list functions.
    // SAFETY: `ea.arg` is the command's NUL-terminated argument.
    if ends_excmd(unsafe { *ea.arg } as c_int) != 0 {
        if ea.skip == 0 {
            // SAFETY: no pattern means every function.
            unsafe { list_functions(ptr::null_mut()) };
        }
        // SAFETY: as above.
        ea.nextcmd = unsafe { check_nextcmd(ea.arg) };
        return;
    }

    // ":function /pat": list functions matching the pattern.
    // SAFETY: as above.
    if unsafe { *ea.arg } == b'/' as c_char {
        // SAFETY: the live command.
        let p = unsafe { list_functions_matching_pat(args) };
        // SAFETY: `p` is the cursor that listing left.
        ea.nextcmd = unsafe { check_nextcmd(p) };
        return;
    }

    // Get the function name.  There are these situations:
    //   func       a normal function name: "name" == func, no dict
    //   dict.func  a new dictionary entry: "name" == NULL, fd_dict set,
    //              fd_di == NULL, fd_newkey == func
    //   dict.func  an existing entry holding a Funcref: "name" == func,
    //              fd_dict and fd_di set, fd_newkey == NULL
    //   dict.func  an existing entry that is not a Funcref:
    //              "name" == NULL, fd_dict and fd_di set
    //   s:func     a script-local name; g:func is the same as func
    let mut fudi = FUNCDICT_INIT;
    let mut p = ea.arg;
    // SAFETY: the command's argument, and both out-parameters are this
    // frame's own.
    let name =
        unsafe { save_function_name(&raw mut p, ea.skip != 0, TFN_NO_AUTOLOAD, &raw mut fudi) };
    // SAFETY: `p` is the cursor into the NUL-terminated argument.
    let paren = has_char(unsafe { cstr::at(p) }, b'(' as c_int);
    if name.is_null() && (fudi.fd_dict.is_null() || !paren) && ea.skip == 0 {
        // Return on an invalid expression in braces, unless the evaluation
        // was cancelled by an aborting error, an interrupt or an exception.
        if !aborting() {
            if !fudi.fd_newkey.is_null() {
                // SAFETY: a message argument the caller holds as a
                // NUL-terminated string.
                let fd_newkey = unsafe { c_str(fudi.fd_newkey) };
                semsg!("E716: Key not present in Dictionary: \"{fd_newkey}\"");
            }
            // SAFETY: the key is this command's own.
            unsafe { xfree(fudi.fd_newkey as *mut c_void) };
            return;
        }
        ea.skip = 1;
    }

    // An error in a function call while evaluating an expression in magic
    // braces should not stop the function being defined.
    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);

    let mut definition = Definition {
        ea,
        args,
        // SAFETY: `p` is this frame's own from here on, walking the
        // command's NUL-terminated argument.
        cursor: unsafe { Cur::new(&raw mut p) },
        name,
        fudi,
        line_arg: ptr::null_mut(),
        line_to_free: ptr::null_mut(),
        newargs: GARRAY_EMPTY,
        default_args: GARRAY_EMPTY,
        newlines: GARRAY_EMPTY,
        varargs: 0,
        flags: FuncFlags::NONE,
        func: ptr::null_mut(),
        free_func: false,
        overwrite: false,
        show_block: false,
    };
    definition.define(paren);

    // ret_free: what every path above leaves for this frame to release.
    // SAFETY: all three are the definition's own, and null is fine for
    // `xfree`.
    unsafe { xfree(definition.line_to_free as *mut c_void) };
    // SAFETY: as above.
    unsafe { xfree(definition.fudi.fd_newkey as *mut c_void) };
    // SAFETY: as above.
    unsafe { xfree(definition.name as *mut c_void) };
    did_emsg.set(did_emsg.get() | saved_did_emsg);
    if definition.show_block {
        ui_ext_cmdline_block_leave();
    }
}
