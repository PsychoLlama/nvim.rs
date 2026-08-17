//! Calling functions in external native libraries — the machinery behind
//! the `libcall()` and `libcallnr()` builtins.
//!
//! # Boundary
//!
//! Everything here is FFI by definition: libuv's `dlopen` wrapper, and then
//! a call through a function pointer whose prototype is *guessed* from the
//! shape of the Vimscript call. Only four prototypes are supported, the
//! cross product of `{const char *, int}` argument and return; anything
//! else in the loaded library will be called with the wrong ABI. That is
//! upstream's contract, unchanged.
//!
//! Signals raised by the callee are not caught (upstream TODO, still open),
//! so a misbehaving library takes the editor down with it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::event::libuv::{uv_dlclose, uv_dlerror, uv_dlopen, uv_dlsym};
use crate::types::uv_lib_t;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

/// The single argument handed to the library function.
pub enum LibcallArg<'a> {
    Str(&'a CStr),
    Int(c_int),
}

/// Which return prototype to assume, and — in [`LibcallResult`] — what came
/// back.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LibcallReturn {
    Str,
    Int,
}

pub enum LibcallResult {
    /// `None` when the callee returned a pointer value of NULL, 1 or -1:
    /// upstream assumes those are never legal strings.
    Str(Option<CString>),
    Int(c_int),
}

type StrToStr = unsafe extern "C" fn(*const c_char) -> *const c_char;
type IntToStr = unsafe extern "C" fn(c_int) -> *const c_char;
type StrToInt = unsafe extern "C" fn(*const c_char) -> c_int;
type IntToInt = unsafe extern "C" fn(c_int) -> c_int;

/// A library held open by libuv, closed on drop.
struct DynLib(uv_lib_t);

impl DynLib {
    /// Load `name`, or report the loader's message.
    fn open(name: &CStr) -> Result<Self, CString> {
        let mut lib = DynLib(uv_lib_t {
            handle: ptr::null_mut(),
            errmsg: ptr::null_mut(),
        });
        // SAFETY: `name` is NUL-terminated and `lib.0` is a live, writable
        // handle. On failure libuv leaves a message behind (owned by the
        // handle, released by `uv_dlclose` when `lib` drops); `uv_dlerror`
        // never returns null.
        unsafe {
            if uv_dlopen(name.as_ptr(), &mut lib.0) != 0 {
                return Err(CStr::from_ptr(uv_dlerror(&lib.0)).to_owned());
            }
        }
        Ok(lib)
    }

    /// Address of `name` in the library, or the loader's message.
    fn symbol(&mut self, name: &CStr) -> Result<*mut c_void, CString> {
        let mut addr: *mut c_void = ptr::null_mut();
        // SAFETY: as in `open`; `addr` is a live, writable out-parameter.
        unsafe {
            if uv_dlsym(&mut self.0, name.as_ptr(), &mut addr) != 0 {
                return Err(CStr::from_ptr(uv_dlerror(&self.0)).to_owned());
            }
        }
        Ok(addr)
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        // SAFETY: the handle is either the one `uv_dlopen` filled in or the
        // zeroed one it failed on; libuv accepts both.
        unsafe { uv_dlclose(&mut self.0) };
    }
}

/// Copy a returned string, unless the callee returned one of the pointer
/// values upstream treats as illegal.
fn returned_string(res: *const c_char) -> Option<CString> {
    if res.is_null() || res.addr() == 1 || res.addr() == usize::MAX {
        return None;
    }
    // SAFETY: the callee promised a NUL-terminated string; nothing else can
    // be checked about it.
    Some(unsafe { CStr::from_ptr(res) }.to_owned())
}

/// Call `funcname` in `libname`, e.g. the Vimscript
/// `libcall("mylib.so", "somefn", "string-argument")`.
///
/// `None` means the library or the symbol could not be loaded; the reason
/// has already been reported to the user.
///
/// # Safety
///
/// Calls arbitrary foreign code under an assumed prototype. The caller has
/// no way to verify either; see the module docs.
pub unsafe fn os_libcall(
    libname: &CStr,
    funcname: &CStr,
    arg: LibcallArg<'_>,
    want: LibcallReturn,
) -> Option<LibcallResult> {
    let mut lib = match DynLib::open(libname) {
        Ok(lib) => lib,
        Err(err) => {
            crate::semsg!("dlerror = \"{}\"", err.to_string_lossy());
            return None;
        }
    };
    let addr = match lib.symbol(funcname) {
        Ok(addr) => addr,
        Err(err) => {
            crate::semsg!("dlerror = \"{}\"", err.to_string_lossy());
            return None;
        }
    };
    // SAFETY: `addr` is a non-null symbol address; the prototype is the
    // caller's assumption, per this function's contract. `lib` outlives the
    // call, so the code stays mapped.
    unsafe {
        Some(match (want, arg) {
            (LibcallReturn::Str, LibcallArg::Str(s)) => LibcallResult::Str(returned_string(
                core::mem::transmute::<*mut c_void, StrToStr>(addr)(s.as_ptr()),
            )),
            (LibcallReturn::Str, LibcallArg::Int(i)) => LibcallResult::Str(returned_string(
                core::mem::transmute::<*mut c_void, IntToStr>(addr)(i),
            )),
            (LibcallReturn::Int, LibcallArg::Str(s)) => {
                LibcallResult::Int(core::mem::transmute::<*mut c_void, StrToInt>(addr)(
                    s.as_ptr(),
                ))
            }
            (LibcallReturn::Int, LibcallArg::Int(i)) => {
                LibcallResult::Int(core::mem::transmute::<*mut c_void, IntToInt>(addr)(i))
            }
        })
    }
}
