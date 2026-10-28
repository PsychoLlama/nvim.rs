//! The runtime half of the `vim.api` bindings: what every binding does
//! around its own conversions.
//!
//! `tools/apigen` emits one safe `convert` per API function into
//! `lua/api_wrappers/` and the table [`BINDINGS`] that names them. This
//! module builds the `vim.api` table from that table, one Lua function per
//! entry through the [`Stack`]'s [`Dispatch`] trampoline, and is what each
//! of those functions runs: check that Lua handed over the arguments the API
//! function declares, refuse a call that may not run here, hand over to
//! `convert`, and turn the [`Error`] it answers into the [`Raise`] the
//! trampoline raises once this frame — and with it the `Error` and every
//! argument `convert` converted — is gone.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::api_error;
use crate::ex_getln::get_text_locked_msg;
use crate::lua::api_wrappers::BINDINGS;
use crate::lua::executor::nlua_is_deferred_safe;
use crate::lua::stack::{At, CFunction, Dispatch, Raise, Stack, dispatch_entry};
use crate::message::{e_fast_api_disabled, e_textlock};
use crate::types::{Error, kErrorTypeException, kErrorTypeValidation};

/// How a result goes back to Lua: `PUSH_SPECIAL` converts `nil` and the
/// other special values the pre-0.11 way, which is what clients of methods
/// older than API level 11 expect. Either way the push borrows the result,
/// and the binding releases the Lua references it holds by dropping it.
pub(crate) const PUSH: bool = false;
pub(crate) const PUSH_SPECIAL: bool = true;

/// The parameter a failed conversion blamed, named in the message: `None`
/// until a conversion fails.
pub(crate) type Blame = Option<&'static CStr>;

/// A binding's own half: pop each argument off the Lua stack, call the API
/// function, hand the result back. Returning — off the end or out of a failed
/// conversion — releases exactly the arguments that were converted, in
/// declaration order, because each one's release is a guard dropped on the
/// way out.
pub(crate) type Convert = fn(&mut Stack, &mut Blame) -> Result<(), Error>;

/// One entry of `vim.api`.
pub(crate) struct Binding {
    /// The name Lua calls it by, which is the API function's.
    name: &'static CStr,
    /// How many arguments the API function declares.
    arity: c_int,
    /// How many results `convert` leaves: 0 or 1.
    results: c_int,
    /// Whether the API function is `fast`, and so may run from a context
    /// where deferring is unsafe.
    fast: bool,
    convert: Convert,
    /// Where this entry sits in [`BINDINGS`], and the C function Lua calls
    /// for it: the trampoline instantiated for that index.
    index: usize,
    entry: CFunction,
}

impl Binding {
    /// An API function that may not run in a fast event, as entry `N` of
    /// [`BINDINGS`].
    pub(crate) const fn deferred<const N: usize>(
        name: &'static CStr,
        arity: c_int,
        results: c_int,
        convert: Convert,
    ) -> Binding {
        Binding {
            name,
            arity,
            results,
            fast: false,
            convert,
            index: N,
            entry: dispatch_entry::<Api, N>(),
        }
    }

    /// A `fast` API function, which has no context to refuse, as entry `N`.
    pub(crate) const fn fast<const N: usize>(
        name: &'static CStr,
        arity: c_int,
        results: c_int,
        convert: Convert,
    ) -> Binding {
        Binding {
            name,
            arity,
            results,
            fast: true,
            convert,
            index: N,
            entry: dispatch_entry::<Api, N>(),
        }
    }
}

/// [`BINDINGS`] as the trampoline's table.
struct Api;

impl Dispatch for Api {
    fn run(stack: &mut Stack, index: usize) -> Result<c_int, Raise> {
        match BINDINGS.get(index) {
            Some(binding) => call(stack, binding),
            // Only `Binding`'s constructors name an index, and
            // `add_api_functions` checks each against its slot.
            None => Err(Raise::message("vim.api: no such function")),
        }
    }
}

/// Build the `vim.api` table and set it on the table on top of the stack.
pub(crate) fn add_api_functions(stack: &mut Stack) {
    stack.new_table(0, BINDINGS.len());
    for (index, binding) in BINDINGS.iter().enumerate() {
        assert_eq!(
            binding.index, index,
            "{:?} is not where its entry says",
            binding.name
        );
        stack.push_function(binding.entry);
        stack.set_field(At::from_top(2), binding.name);
    }
    stack.set_field(At::from_top(2), c"api");
}

/// Run `binding` over the frame Lua called it with: everything a binding does
/// around its own conversions. Answers the number of results `convert` left
/// on the stack.
///
/// The arity check comes first and the fast-event refusal second, before any
/// argument is converted, as upstream's generated bindings have them; the
/// `active_lstate` switch is `convert`'s, around the API call alone.
fn call(stack: &mut Stack, binding: &Binding) -> Result<c_int, Raise> {
    if stack.top() != At::arg(binding.arity) {
        return Err(refusal(&wrong_arity(binding.arity), None));
    }
    if !binding.fast && !nlua_is_deferred_safe() {
        return Err(Raise::message(fast_api_refusal(binding.name)));
    }
    let mut blame = None;
    match (binding.convert)(stack, &mut blame) {
        Ok(()) => Ok(binding.results),
        Err(err) => Err(refusal(&err, blame)),
    }
}

/// The failed call's message as one string: the parameter a failed
/// conversion blamed, if one did, and the message itself. ([`Raise::message`]
/// puts the source position in front.)
fn refusal(err: &Error, blame: Blame) -> Raise {
    let message = err.message_or_empty().to_bytes();
    match blame {
        Some(param) => Raise::message([b"Invalid '", param.to_bytes(), b"': ", message].concat()),
        None => Raise::message(message),
    }
}

/// `E5560: <name> must not be called in a fast event context`.
fn fast_api_refusal(name: &CStr) -> Vec<u8> {
    let template = e_fast_api_disabled.to_bytes();
    let at = template
        .windows(2)
        .position(|w| w == b"%s")
        .expect("the message has a slot for the name");
    [&template[..at], name.to_bytes(), &template[at + 2..]].concat()
}

/// Refuses a call that arrived with a different number of arguments than the
/// API function declares.
fn wrong_arity(argc: c_int) -> Error {
    if argc == 1 {
        api_error!(kErrorTypeValidation, "Expected {argc} argument")
    } else {
        api_error!(kErrorTypeValidation, "Expected {argc} arguments")
    }
}

/// Refuses a call made while the text is locked.
pub(crate) fn text_locked_error() -> Error {
    Error::from_message(kErrorTypeException, get_text_locked_msg())
}

/// Refuses a call made from an expression mapping, which the cmdline window
/// alone would have allowed.
pub(crate) fn expr_map_locked_error() -> Error {
    Error::from_message(kErrorTypeException, e_textlock)
}
