//! The Lua runtime files compiled into the binary.
//!
//! Each `vim_dot_*_module` is one `runtime/lua/vim/*.lua` embedded with
//! `include_bytes!`, and [`builtin_modules`] is the table
//! `nlua_init_packages` walks to install them as preloaded packages.  Only
//! the table's *shape* matters here; the contents are the runtime's.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

const vim_dot__init_packages_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__init_packages_module.bin"
));

const vim_dot_inspect_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_inspect_module.bin"
));

const vim_dot_filetype_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_filetype_module.bin"
));

const vim_dot_fs_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_fs_module.bin"
));

const vim_dot_F_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_F_module.bin"
));

const vim_dot_keymap_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_keymap_module.bin"
));

const vim_dot_loader_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_loader_module.bin"
));

const vim_dot_text_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_text_module.bin"
));

const vim_dot__core_dot_defaults_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_defaults_module.bin"
));

const vim_dot__core_dot_editor_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_editor_module.bin"
));

const vim_dot__core_dot_ex_cmd_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_ex_cmd_module.bin"
));

const vim_dot__core_dot_exrc_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_exrc_module.bin"
));

const vim_dot__core_dot_help_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_help_module.bin"
));

const vim_dot__core_dot_log_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_log_module.bin"
));

const vim_dot__core_dot_options_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_options_module.bin"
));

const vim_dot__core_dot_server_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_server_module.bin"
));

const vim_dot__core_dot_shared_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_shared_module.bin"
));

const vim_dot__core_dot_stringbuffer_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_stringbuffer_module.bin"
));

const vim_dot__core_dot_system_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_system_module.bin"
));

const vim_dot__core_dot_ui2_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_ui2_module.bin"
));

const vim_dot__core_dot_util_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_util_module.bin"
));

pub(crate) static builtin_modules: SharedCell<[ModuleDef; 21]> = SharedCell::new([
    ModuleDef {
        name: b"vim._init_packages\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__init_packages_module.as_ptr(),
        size: vim_dot__init_packages_module.len(),
    },
    ModuleDef {
        name: b"vim.inspect\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_inspect_module.as_ptr(),
        size: vim_dot_inspect_module.len(),
    },
    ModuleDef {
        name: b"vim.filetype\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_filetype_module.as_ptr(),
        size: vim_dot_filetype_module.len(),
    },
    ModuleDef {
        name: b"vim.fs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_fs_module.as_ptr(),
        size: vim_dot_fs_module.len(),
    },
    ModuleDef {
        name: b"vim.F\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_F_module.as_ptr(),
        size: vim_dot_F_module.len(),
    },
    ModuleDef {
        name: b"vim.keymap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_keymap_module.as_ptr(),
        size: vim_dot_keymap_module.len(),
    },
    ModuleDef {
        name: b"vim.loader\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_loader_module.as_ptr(),
        size: vim_dot_loader_module.len(),
    },
    ModuleDef {
        name: b"vim.text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_text_module.as_ptr(),
        size: vim_dot_text_module.len(),
    },
    ModuleDef {
        name: b"vim._core.defaults\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_defaults_module.as_ptr(),
        size: vim_dot__core_dot_defaults_module.len(),
    },
    ModuleDef {
        name: b"vim._core.editor\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_editor_module.as_ptr(),
        size: vim_dot__core_dot_editor_module.len(),
    },
    ModuleDef {
        name: b"vim._core.ex_cmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_ex_cmd_module.as_ptr(),
        size: vim_dot__core_dot_ex_cmd_module.len(),
    },
    ModuleDef {
        name: b"vim._core.exrc\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_exrc_module.as_ptr(),
        size: vim_dot__core_dot_exrc_module.len(),
    },
    ModuleDef {
        name: b"vim._core.help\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_help_module.as_ptr(),
        size: vim_dot__core_dot_help_module.len(),
    },
    ModuleDef {
        name: b"vim._core.log\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_log_module.as_ptr(),
        size: vim_dot__core_dot_log_module.len(),
    },
    ModuleDef {
        name: b"vim._core.options\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_options_module.as_ptr(),
        size: vim_dot__core_dot_options_module.len(),
    },
    ModuleDef {
        name: b"vim._core.server\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_server_module.as_ptr(),
        size: vim_dot__core_dot_server_module.len(),
    },
    ModuleDef {
        name: b"vim._core.shared\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_shared_module.as_ptr(),
        size: vim_dot__core_dot_shared_module.len(),
    },
    ModuleDef {
        name: b"vim._core.stringbuffer\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_stringbuffer_module.as_ptr(),
        size: vim_dot__core_dot_stringbuffer_module.len(),
    },
    ModuleDef {
        name: b"vim._core.system\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_system_module.as_ptr(),
        size: vim_dot__core_dot_system_module.len(),
    },
    ModuleDef {
        name: b"vim._core.ui2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_ui2_module.as_ptr(),
        size: vim_dot__core_dot_ui2_module.len(),
    },
    ModuleDef {
        name: b"vim._core.util\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_util_module.as_ptr(),
        size: vim_dot__core_dot_util_module.len(),
    },
]);
