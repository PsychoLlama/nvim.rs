//! The editor's Lua states: creating them, and lending the main one out.
//!
//! The main state is built once by [`nlua_init`] (or [`nlua_run_script`] for
//! `nvim -l`) and owned here for the life of the process. Editor code reaches
//! it as a [`Stack`] through [`main_stack`]; the state an API call came in on
//! (a coroutine's, say) is [`active_stack`], set around each call by
//! [`ActiveState`]. A luv thread gets a state of its own from
//! [`nlua_init_state`], which luv closes when the thread is done.
//!
//! What stays raw here is what only C can do: `luaL_newstate` and
//! `luaL_openlibs`, `lua_close`, and opening the vendored C libraries (luv,
//! lpeg, libmpack, cjson, tree-sitter) onto a state.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md (`lua/executor/state.rs`).
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use core::ptr::NonNull;
use std::ffi::CString;
use std::io::Write;

use super::{
    VIM_FUNCTIONS, add_common, in_script, init_packages, install_luv, nlua_exec_file,
    nlua_thread_acquire_vm, register_runtime, require_ref, thread_api,
};
use crate::event::libuv::{uv_thread_equal, uv_thread_self};
use crate::global_cell::{GlobalCell, SharedCell};
use crate::guard::Restore;
use crate::lua::converter::binding::add_api_functions;
use crate::lua::converter::init_types;
use crate::lua::ffi::{lua_close, luaL_newstate, luaL_openlibs};
use crate::lua::stack::{At, Stack};
use crate::lua::stdlib::{add_editor_stdlib, share_module};
use crate::lua::treesitter::nlua_treesitter_init;
use crate::lua::xdiff::diff_function;
use crate::mpack::lmpack::luaopen_mpack;
use crate::os::cshim::gettext;
use crate::profile::startup_timing;
use crate::runtime::runtime_search_path_validate;
use crate::startup::os_exit;
use crate::types::{lua_State, uv_thread_t};

unsafe extern "C-unwind" {
    /// lpeg's own `luaopen_*`, linked in from the vendored library.
    fn luaopen_lpeg(lstate: *mut lua_State) -> c_int;
    /// luv's hooks for a thread to get and give back a state of its own.
    fn luv_set_thread_cb(
        acquire: Option<unsafe extern "C-unwind" fn() -> *mut lua_State>,
        release: Option<unsafe extern "C-unwind" fn(*mut lua_State)>,
    );
}

/// The editor's main state, from [`nlua_init`] (or [`nlua_run_script`]) on.
static MAIN_STATE: GlobalCell<Option<NonNull<lua_State>>> = GlobalCell::new(None);

/// The state the running API call came in on: the main state's, or one of
/// its coroutines'. What a callback's `:verbose` source walks.
static ACTIVE_STATE: GlobalCell<Option<NonNull<lua_State>>> = GlobalCell::new(None);

/// The thread `nlua_init` ran on: the only one allowed to rebuild the
/// runtime search path.
static main_thread: SharedCell<uv_thread_t> = SharedCell::new(0);

/// The main state, as a [`Stack`] over its running frame.
///
/// The frame is whatever runs on the main state right now: the editor's own
/// level, or the C function a Lua caller is in -- whose own `Stack`, further
/// up the Rust stack, is then this same frame. Both stay sound because every
/// user of one leaves the slots below its first push alone (it pops what it
/// pushed, and a call consumes only the function and arguments it pushed),
/// so a string either one lends stays anchored.
///
/// # Panics
/// Before [`nlua_init`] has made the state.
pub fn main_stack() -> Stack {
    let state = MAIN_STATE.get().expect("the Lua runtime is up");
    // SAFETY: the main state lives from `nlua_init` to the process's end,
    // and is used on the main thread only; the frame sharing is as above.
    unsafe { Stack::from_raw(state.as_ptr()) }
}

/// The state the running API call came in on, as a [`Stack`]; `None` when no
/// state has been made yet.
pub(crate) fn active_stack() -> Option<Stack> {
    let state = ACTIVE_STATE.get()?;
    // SAFETY: the main state or one of its coroutines, which the API call
    // that set it is running on (so it is live), on the main thread; frame
    // sharing as for `main_stack`.
    Some(unsafe { Stack::from_raw(state.as_ptr()) })
}

/// Marks `stack`'s state as the one the running API call came in on, until
/// dropped.
#[must_use = "the state is only active while the guard lives"]
pub struct ActiveState {
    _restore: Restore<Option<NonNull<lua_State>>>,
}

impl ActiveState {
    /// Make `stack`'s state the active one.
    pub fn enter(stack: &Stack) -> ActiveState {
        ActiveState {
            _restore: Restore::of(&ACTIVE_STATE, NonNull::new(stack.lua_state())),
        }
    }
}

/// Publish `lstate` as the main state.
fn adopt_main(lstate: NonNull<lua_State>) {
    MAIN_STATE.set(Some(lstate));
    ACTIVE_STATE.set(Some(lstate));
}

/// A fresh state with the standard libraries open, or `None` when LuaJIT
/// cannot make one.
fn new_state() -> Option<NonNull<lua_State>> {
    // SAFETY: `luaL_newstate` makes a state nothing else has seen, and
    // `luaL_openlibs` opens the libraries on it.
    unsafe {
        let lstate = NonNull::new(luaL_newstate())?;
        luaL_openlibs(lstate.as_ptr());
        Some(lstate)
    }
}

/// Write `message` to stderr, as upstream's `fprintf(stderr, ..)` does.
fn to_stderr(message: &[u8]) {
    let _ = std::io::stderr().write_all(message);
}

/// The vendored C libraries every state gets: libmpack, lpeg, `vim.diff` and
/// cjson, on the `vim` table on top of `stack`'s state.
fn add_vendored(stack: &mut Stack) {
    let lstate = stack.lua_state();
    // vim.mpack -- shared, or luv is reinitialised by require'mpack'.
    // SAFETY: the state `stack` is over, with room for the module's table.
    unsafe { luaopen_mpack(lstate) };
    share_module(stack, c"mpack", 2);

    // vim.lpeg
    // SAFETY: as above.
    unsafe { luaopen_lpeg(lstate) };
    share_module(stack, c"lpeg", 3);

    // vim.text.diff
    // TODO(justinmk): set vim.text.diff here, or rename this to "_diff".
    stack.push_function(diff_function());
    stack.set_field(At::from_top(2), c"diff");

    // vim.json
    // SAFETY: as above.
    unsafe { crate::cjson::lua_cjson::lua_cjson_new(lstate) };
    stack.set_field(At::from_top(2), c"json");
}

/// Everything the *main* state gets: the replaced `print` and `debug.debug`,
/// the api, the `vim.*` C functions, treesitter, the standard library, and
/// the embedded runtime modules.
fn main_state_init(stack: &mut Stack) -> bool {
    register_runtime(stack);

    // The `vim` table, built on the stack and stored last.
    stack.new_table(0, 0);
    add_api_functions(stack);
    init_types(stack);
    stack.register_functions(VIM_FUNCTIONS);

    add_common(stack, false);
    // SAFETY: the main state, with the `vim` table on top.
    unsafe { install_luv(stack.lua_state(), false, false) };

    // Only `--startuptime` needs `require` wrapped, and the wrapper needs
    // the original to delegate to.
    if startup_timing() {
        stack.get_global(c"require");
        require_ref.set(stack.reference(At::TOP));
        stack.pop(1);
        stack.push_function(super::require_function());
        stack.set_global(c"require");
    }

    // SAFETY: the main state, with the `vim` table on top.
    unsafe { nlua_treesitter_init(stack.lua_state()) };
    add_editor_stdlib(stack);
    add_vendored(stack);
    stack.set_global(c"vim");

    init_packages(stack, false)
}

/// Populate the global `arg` table from the command line, with `arg[0]` the
/// script's own name: `argv[lua_arg0 - 1]`, then everything after it. A
/// `lua_arg0` of -1 (no script) leaves the table empty.
fn init_argv(stack: &mut Stack, argv: &[CString], lua_arg0: c_int) {
    stack.new_table(0, 0);
    if let Some(script) = usize::try_from(lua_arg0 - 1)
        .ok()
        .filter(|&at| at < argv.len())
    {
        stack.push_bytes(argv[script].to_bytes());
        stack.raw_set_index(At::from_top(2), 0);
        for (n, arg) in (1..).zip(&argv[script + 1..]) {
            stack.push_bytes(arg.to_bytes());
            stack.raw_set_index(At::from_top(2), n);
        }
    }
    stack.set_global(c"arg");
}

/// Open the editor's Lua state, with `arg` from `argv` (see [`init_argv`]).
/// Fatal if it cannot be built.
///
/// Once, from `main()`; `--version` calls it with no arguments.
pub fn nlua_init(argv: &[CString], lua_arg0: c_int) {
    let Some(lstate) = new_state() else {
        to_stderr(gettext(c"E970: Failed to initialize Lua interpreter\n").to_bytes());
        os_exit(1);
    };
    adopt_main(lstate);
    let mut stack = main_stack();
    if !main_state_init(&mut stack) {
        to_stderr(gettext(c"E970: Failed to initialize builtin Lua modules\n").to_bytes());
        os_exit(1);
    }
    // SAFETY: two functions of the right shape, for luv to call on a
    // thread's behalf.
    unsafe { luv_set_thread_cb(Some(nlua_thread_acquire_vm), Some(nlua_common_free_all_mem)) };

    // SAFETY: libuv's thread id of the caller, which needs nothing.
    main_thread.set(unsafe { uv_thread_self() });
    init_argv(&mut stack, argv, lua_arg0);
}

/// `nvim -ll script.lua`: run the script and leave, with no editor at all.
/// `argv[lua_arg0 - 1]` is the script.
///
/// Once, from `main()`, instead of [`nlua_init`].
pub fn nlua_run_script(argv: &[CString], lua_arg0: c_int) -> ! {
    in_script.set(true);
    let lstate = nlua_init_state(false);
    let lstate = NonNull::new(lstate).expect("a script's state");
    adopt_main(lstate);
    // SAFETY: as in `nlua_init`.
    unsafe { luv_set_thread_cb(Some(nlua_thread_acquire_vm), Some(nlua_common_free_all_mem)) };
    init_argv(&mut main_stack(), argv, lua_arg0);
    let script = usize::try_from(lua_arg0 - 1).expect("the script follows `-ll`");
    let lua_ok = nlua_exec_file(&argv[script]);
    std::process::exit(if lua_ok { 0 } else { 1 });
}

/// A state for a luv thread (`thread`), or for a `-l` script.
///
/// It gets the shared half of the `vim` table and the thread-safe half of the
/// standard library; a thread also gets the one api function it is allowed.
/// Called on the thread the state will belong to.
pub(crate) fn nlua_init_state(thread: bool) -> *mut lua_State {
    // The runtime path is shared, so only the main thread may rebuild it.
    // SAFETY: libuv's id of the calling thread, and two ids compared.
    let on_main_thread = unsafe {
        let this_thread: uv_thread_t = uv_thread_self();
        uv_thread_equal(main_thread.ptr(), &raw const this_thread) != 0
    };
    if !in_script.get() && on_main_thread {
        runtime_search_path_validate();
    }

    let Some(lstate) = new_state() else {
        // luv's `acquire_vm` answers null for a state it could not get.
        return core::ptr::null_mut();
    };
    // SAFETY: the state just made, on the thread it belongs to, used by
    // nothing else until this returns it.
    let mut stack = unsafe { Stack::from_raw(lstate.as_ptr()) };
    if !in_script.get() {
        stack.push_function(super::print_function());
        stack.set_global(c"print");
    }
    stack.push_integer(0);
    stack.set_field(At::REGISTRY, c"nlua.refcount");

    stack.new_table(0, 0);
    add_common(&mut stack, thread);
    // SAFETY: as above, with the `vim` table on top.
    unsafe { install_luv(lstate.as_ptr(), thread, in_script.get()) };
    add_vendored(&mut stack);
    if !in_script.get() {
        stack.new_table(0, 0);
        stack.register_functions(thread_api());
        stack.set_field(At::from_top(2), c"api");
    }
    stack.set_global(c"vim");

    init_packages(&mut stack, in_script.get());

    // package.loaded.vim = vim
    stack.get_global(c"package");
    stack.get_field(At::TOP, c"loaded");
    stack.get_global(c"vim");
    stack.set_field(At::from_top(2), c"vim");
    stack.pop(2);

    lstate.as_ptr()
}

/// luv's `release_vm`: close a thread's state.
///
/// # Safety
/// `lstate` must be a state [`nlua_init_state`] built, and unused afterwards.
unsafe extern "C-unwind" fn nlua_common_free_all_mem(lstate: *mut lua_State) {
    // SAFETY: the caller's promise.
    unsafe { lua_close(lstate) };
}

/// `require('vim._core.defaults')`, run once the editor is far enough along
/// to have options and mappings.
pub fn nlua_init_defaults() {
    let mut stack = main_stack();
    stack.get_global(c"require");
    stack.push_bytes(b"vim._core.defaults");
    if super::nlua_pcall(&mut stack, 1, 0).is_err() {
        let error = stack.to_bytes(At::TOP).map(<[u8]>::to_vec);
        let error = error.unwrap_or_else(|| b"(null)".to_vec());
        to_stderr(&[&error[..until_nul(&error)], b"\n"].concat());
    }
}

/// Where a C string's copy of `bytes` would end.
fn until_nul(bytes: &[u8]) -> usize {
    bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())
}
