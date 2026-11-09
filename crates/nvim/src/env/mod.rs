//! What the editor makes of the environment: `$VAR` and `~` inside a path
//! ([`expand`]), the `~` shorthand going back out ([`home`]), where its own
//! runtime lives (`$VIM`/`$VIMRUNTIME`, [`runtime`]), and the XDG base
//! directories with the per-application paths under them ([`stdpaths`]).
//!
//! Reading and writing the process environment itself is the operating
//! system's, in `crate::os::env`; everything here is path logic over the
//! answers.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod expand;
mod home;
mod runtime;
mod stdpaths;

pub(crate) use expand::expand_env_within;
pub(crate) use expand::{expand_env, expand_env_escaped};
pub(crate) use home::home_to_tilde;
pub(crate) use runtime::{DEFAULT_VIM_DIR, DEFAULT_VIMRUNTIME_DIR};
pub(crate) use runtime::{getenv_resolved, install_prefix};
pub(crate) use stdpaths::{appname_is_valid, get_appname, xdg_home};
pub(crate) use stdpaths::{state_subpath, user_config_path, xdg_var};

/// One of the XDG base-directory variables nvim reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XdgVar {
    ConfigHome,
    DataHome,
    CacheHome,
    StateHome,
    RuntimeDir,
    ConfigDirs,
    DataDirs,
}
