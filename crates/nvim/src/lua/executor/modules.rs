//! The Lua runtime files compiled into the binary.
//!
//! Each row of [`BUILTIN_MODULES`] is one `runtime/lua/vim/*.lua` compiled to
//! LuaJIT bytecode by `build.rs` and embedded with `include_bytes!`; the
//! table is what `nlua_init_packages` walks to install them as preloaded
//! packages, in this order.  `build.rs`'s `EMBEDDED_LUA_MODULES` is the same
//! list and the two must agree.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;

/// One embedded module: the name `require` knows it by, and its bytecode.
pub(crate) struct ModuleDef {
    pub name: &'static CStr,
    /// The compiled chunk *plus* the NUL `build.rs` appends, which is why
    /// [`Self::chunk`] exists.
    data: &'static [u8],
}

impl ModuleDef {
    /// The bytecode alone, without the terminator.
    pub(crate) fn chunk(&self) -> &'static [u8] {
        &self.data[..self.data.len() - 1]
    }
}

/// Declare the table. Each row names the module and the `build.rs` output it
/// comes from — the file name is the module name with `.` spelled `_dot_`.
macro_rules! builtin_modules {
    ($($name:literal => $file:literal,)*) => {
        /// Every `vim.*` module compiled into the binary, in preload order.
        pub(crate) static BUILTIN_MODULES: &[ModuleDef] = &[$(
            ModuleDef {
                name: $name,
                data: include_bytes!(concat!(
                    env!("OUT_DIR"), "/lua_modules/", $file, "_module.bin"
                )),
            },
        )*];
    };
}

builtin_modules! {
    c"vim._init_packages" => "vim_dot__init_packages",
    c"vim.inspect" => "vim_dot_inspect",
    c"vim.filetype" => "vim_dot_filetype",
    c"vim.fs" => "vim_dot_fs",
    c"vim.F" => "vim_dot_F",
    c"vim.keymap" => "vim_dot_keymap",
    c"vim.loader" => "vim_dot_loader",
    c"vim.text" => "vim_dot_text",
    c"vim._core.defaults" => "vim_dot__core_dot_defaults",
    c"vim._core.editor" => "vim_dot__core_dot_editor",
    c"vim._core.ex_cmd" => "vim_dot__core_dot_ex_cmd",
    c"vim._core.exrc" => "vim_dot__core_dot_exrc",
    c"vim._core.help" => "vim_dot__core_dot_help",
    c"vim._core.log" => "vim_dot__core_dot_log",
    c"vim._core.options" => "vim_dot__core_dot_options",
    c"vim._core.server" => "vim_dot__core_dot_server",
    c"vim._core.shared" => "vim_dot__core_dot_shared",
    c"vim._core.stringbuffer" => "vim_dot__core_dot_stringbuffer",
    c"vim._core.system" => "vim_dot__core_dot_system",
    c"vim._core.ui2" => "vim_dot__core_dot_ui2",
    c"vim._core.util" => "vim_dot__core_dot_util",
}
