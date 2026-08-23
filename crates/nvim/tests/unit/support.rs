//! Shared helpers for calling the C-ABI surface from Rust tests.

#[cfg(not(miri))]
use std::ffi::OsString;
use std::ffi::{CStr, CString, c_char};
#[cfg(not(miri))]
use std::path::{Path, PathBuf};
#[cfg(not(miri))]
use std::sync::{Mutex, MutexGuard};

use c2rust_neovim::memory::xfree;

#[cfg(not(miri))]
pub mod tv;

/// A NUL-terminated copy of `s`, kept alive by the caller's binding.
pub fn cstr(s: impl Into<Vec<u8>>) -> CString {
    CString::new(s).unwrap()
}

/// Copy an allocated C string's bytes, then `xfree` the original — the same
/// "prove it was allocated" pattern the Lua specs used via `internalize`.
///
/// # Safety
/// `ptr` must be a valid NUL-terminated string from the `xmalloc` family.
pub unsafe fn take_bytes(ptr: *mut c_char) -> Vec<u8> {
    let owned = CStr::from_ptr(ptr).to_bytes().to_vec();
    xfree(ptr.cast());
    owned
}

/// [`take_bytes`], decoded as UTF-8.
///
/// # Safety
/// `ptr` must be a valid NUL-terminated string from the `xmalloc` family.
pub unsafe fn internalize(ptr: *mut c_char) -> String {
    String::from_utf8(take_bytes(ptr)).unwrap()
}

/// Editor globals are process-wide and `cargo test` runs cases in parallel,
/// so anything that reads or writes one takes this first.
///
/// The LuaJIT harness got this for free: `itp` forks a child per case. Here
/// there is one process, so the serialisation is explicit. Poisoning is
/// ignored — a panicking case has already reported its own failure, and the
/// next one restores what it touched itself.
///
/// Miri cannot run [`init_editor`] (`early_init` reaches `clock_gettime`),
/// so everything that needs a live editor is compiled out there — as the
/// undofile cases already are.
#[cfg(not(miri))]
static EDITOR: Mutex<()> = Mutex::new(());

/// Proof that the holder has exclusive use of the editor's globals, and
/// that [`init_editor`] has already run.
///
/// It exists so the "caller holds the editor lock" precondition is a type
/// rather than a comment: every helper here that reads editor state takes
/// one, so a case cannot reach that state without having gone through
/// [`editor_lock`] — which is also the only thing that brings the editor
/// up. Without the token, a case that runs before the first lock and one
/// that runs after would see different globals.
#[cfg(not(miri))]
pub struct Editor {
    _guard: MutexGuard<'static, ()>,
}

/// Exclusive use of the editor's globals for the caller's scope.
#[cfg(not(miri))]
pub fn editor_lock() -> Editor {
    let guard = EDITOR.lock().unwrap_or_else(|e| e.into_inner());
    init_editor();
    Editor { _guard: guard }
}

/// Bring up as much of the editor as the code a case reaches will
/// dereference, once per process.
///
/// The LuaJIT harness ran `event_init()` + `early_init(NULL)` in a fresh
/// forked child for every case. Here there is one process shared with every
/// other case, so this runs lazily — the first time anything takes the
/// editor lock — and never again.
///
/// `early_init` alone is not enough for a case that provokes an error
/// message, and the failure is a long way from the cause: `emsg` resolves
/// the message's highlight group, formats it, and clears to the end of the
/// screen, and that last step walks `msg_grid_adj`, whose target is NULL
/// until the compositor sets it. So the grid is allocated and pointed at
/// `default_grid` here, which is what `msg_grid_validate` does for an editor
/// that is not using a separate message grid.
///
/// Messages still reach stdout (nothing is attached to consume them), which
/// is exactly what the Lua lane did too.
#[cfg(not(miri))]
fn init_editor() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    // SAFETY: the caller holds the editor lock, and `Once` makes this the
    // only initialisation.
    ONCE.call_once(|| unsafe {
        c2rust_neovim::main::event_init();
        c2rust_neovim::main::early_init(std::ptr::null_mut());
        c2rust_neovim::drawscreen::default_grid_alloc();
        c2rust_neovim::main::msg_grid_adj.with_mut(|view| {
            view.target = c2rust_neovim::main::default_grid.ptr();
        });
    });
}

/// Everything a case has to claim before it can touch process-wide state,
/// and everything it has to put back afterwards.
///
/// The LuaJIT harness got all of this by forking a child per case. Here
/// there is one process shared with every other case, so each claim is
/// explicit and each restoration is this type's [`Drop`]:
///
/// - the **editor globals**, through [`editor_lock`];
/// - the **working directory**, saved on the way in and restored on the way
///   out, whoever changed it and however many times;
/// - a **private directory** to stand in, named after the case so two cases
///   cannot delete each other's fixtures, and *canonicalised* because the
///   temp directory is often reached through a link while the entry points
///   under test answer resolved paths;
/// - the **environment block**, one variable at a time: [`remember_env`] (or
///   any of the writers here) records the old value once, and the drop puts
///   every recorded variable back.
///
/// [`remember_env`]: Sandbox::remember_env
///
/// Take [`Sandbox::globals`] when only the editor's own state is in play and
/// [`Sandbox::dir`] when the case needs somewhere to put files.
#[cfg(not(miri))]
pub struct Sandbox {
    /// The private directory, when one was asked for.
    dir: Option<PathBuf>,
    saved_cwd: PathBuf,
    /// Variables written through this sandbox, with the value they had
    /// before the first write. One entry per name, in first-write order.
    saved_env: Vec<(String, Option<OsString>)>,
    editor: Editor,
}

#[cfg(not(miri))]
impl Sandbox {
    /// The editor lock, plus the promise that the working directory and
    /// every variable written through this sandbox are restored on drop.
    pub fn globals() -> Sandbox {
        let editor = editor_lock();
        Sandbox {
            dir: None,
            saved_cwd: std::env::current_dir().expect("a working directory"),
            saved_env: Vec::new(),
            editor,
        }
    }

    /// [`Sandbox::globals`] plus an empty private directory, entered.
    ///
    /// `name` distinguishes this case's directory from every other case's;
    /// pass the test function's own name.
    pub fn dir(name: &str) -> Sandbox {
        let mut sandbox = Sandbox::globals();
        let dir = std::env::temp_dir().join(format!("nvim-unit-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a private sandbox");
        let dir = dir.canonicalize().expect("the sandbox resolves");
        std::env::set_current_dir(&dir).expect("standing in the sandbox");
        sandbox.dir = Some(dir);
        sandbox
    }

    /// The editor lock this sandbox holds, to hand to a helper whose
    /// precondition is "the caller holds it" — the same token
    /// [`editor_lock`] answers.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// The private directory's absolute, resolved path.
    pub fn root(&self) -> &Path {
        self.dir.as_deref().expect("this sandbox has no directory")
    }

    /// The private directory as the string an absolute expectation is built
    /// from.
    pub fn as_str(&self) -> &str {
        self.root().to_str().expect("a temp path is text")
    }

    /// A name inside the private directory. Nothing is created.
    pub fn path(&self, name: &str) -> PathBuf {
        self.root().join(name)
    }

    /// A directory inside the private directory, and its parents.
    pub fn mkdir(&self, name: &str) -> PathBuf {
        let at = self.path(name);
        std::fs::create_dir_all(&at).expect("a fixture directory");
        at
    }

    /// An empty file inside the private directory.
    pub fn touch(&self, name: &str) -> PathBuf {
        self.write(name, b"")
    }

    /// A file inside the private directory holding `contents`.
    pub fn write(&self, name: &str, contents: &[u8]) -> PathBuf {
        let at = self.path(name);
        std::fs::write(&at, contents).expect("a fixture file");
        at
    }

    /// Record `name`'s current value, once, so the drop can put it back.
    ///
    /// Call this before writing the variable by any route the sandbox does
    /// not own — a direct `os_setenv`, or an entry point under test that
    /// rewrites `$PATH` of its own accord.
    pub fn remember_env(&mut self, name: &str) {
        if !self.saved_env.iter().any(|(seen, _)| seen == name) {
            self.saved_env
                .push((name.to_string(), std::env::var_os(name)));
        }
    }

    /// Set a variable for the rest of the case, through the crate's own
    /// `os_setenv` — the block the editor reads is the process's, and the
    /// spec set it this way because "Lua doesn't have setenv".
    pub fn set_env(&mut self, name: &str, value: &str) -> std::ffi::c_int {
        self.remember_env(name);
        // SAFETY: both strings are this frame's and NUL-terminated.
        unsafe { c2rust_neovim::os::env::os_setenv(cstr(name).as_ptr(), cstr(value).as_ptr(), 1) }
    }

    /// `os_setenv` with `overwrite` off: an existing value wins.
    pub fn set_env_if_unset(&mut self, name: &str, value: &str) -> std::ffi::c_int {
        self.remember_env(name);
        // SAFETY: as above.
        unsafe { c2rust_neovim::os::env::os_setenv(cstr(name).as_ptr(), cstr(value).as_ptr(), 0) }
    }

    /// Remove a variable for the rest of the case.
    pub fn unset_env(&mut self, name: &str) -> std::ffi::c_int {
        self.remember_env(name);
        // SAFETY: the name is this frame's and NUL-terminated.
        unsafe { c2rust_neovim::os::env::os_unsetenv(cstr(name).as_ptr()) }
    }
}

#[cfg(not(miri))]
impl Drop for Sandbox {
    fn drop(&mut self) {
        // The directory has to be left before it can be removed, and the
        // caller's directory has to come back either way.
        let _ = std::env::set_current_dir(&self.saved_cwd);
        if let Some(dir) = &self.dir {
            // A case may have taken the read or execute bit off something,
            // and `remove_dir_all` cannot enter a directory it cannot read.
            for entry in walk(dir) {
                let _ = std::fs::set_permissions(
                    &entry,
                    std::os::unix::fs::PermissionsExt::from_mode(0o700),
                );
            }
            let _ = std::fs::remove_dir_all(dir);
        }
        for (name, value) in &self.saved_env {
            let name = cstr(name.as_str());
            // SAFETY: both strings are this frame's and NUL-terminated.
            unsafe {
                match value {
                    Some(value) => {
                        let value =
                            CString::new(value.as_encoded_bytes()).expect("it came from the block");
                        c2rust_neovim::os::env::os_setenv(name.as_ptr(), value.as_ptr(), 1);
                    }
                    None => {
                        c2rust_neovim::os::env::os_unsetenv(name.as_ptr());
                    }
                }
            }
        }
    }
}

/// Every path under `at`, shallowest first, for the permission reset above.
/// A symbolic link is listed but never descended.
#[cfg(not(miri))]
fn walk(at: &Path) -> Vec<PathBuf> {
    let mut out = vec![at.to_path_buf()];
    if let Ok(entries) = std::fs::read_dir(at) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                out.extend(walk(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

/// The Rust twin of `test/unit/testutil.lua`'s `alloc_log` and
/// `test/unit/eval/testutil.lua`'s `alloc_logging_t`.
///
/// A spec case that reads
///
/// ```lua
/// alloc_log:clear()
/// lib.tv_list_append_string(l, 'test', 3)
/// alloc_log:check({
///   a.str(l.lv_last.li_tv.vval.v_string, 'tes'),
///   a.li(l.lv_last),
/// })
/// ```
///
/// ports to
///
/// ```ignore
/// log.clear();
/// unsafe { tv_list_append_string(l, cstr("test").as_ptr(), 3) };
/// log.check(&[
///     alloc::string(unsafe { (*(*l).lv_last).li_tv.vval.v_string }, 3),
///     alloc::li(unsafe { (*l).lv_last }),
/// ]);
/// ```
///
/// The one property to preserve when porting a case: **every size is
/// derived from the layout**, never written as a literal. The Lua
/// expectations spell them `ffi.sizeof('list_T')` and
/// `ffi.offsetof('dictitem_T', 'di_key') + n + 1`; here they are
/// `size_of::<list_T>()` and `offset_of!(dictitem_T, di_key) + n + 1`. That
/// is what makes an expectation a statement about the allocation rather than
/// about this machine, and it is why the cases port at all.
#[cfg(not(miri))]
pub mod alloc {
    use std::ffi::{c_char, c_void};
    use std::mem::{offset_of, size_of};

    use c2rust_neovim::memory::alloc_log::{AllocEvent, Recorder, clear_tmp_allocs};
    use c2rust_neovim::types::{
        DictWatcher, dict_T, dictitem_T, list_T, listitem_T, partial_T, typval_T,
    };

    /// A recording of this thread's editor allocations, plus the editor lock
    /// — recording only means anything with one case running at a time.
    ///
    /// Dropping it stops the recording and releases the lock.
    pub struct AllocLog {
        recorder: Recorder,
        editor: super::Editor,
    }

    impl AllocLog {
        /// Take the editor and start recording.
        pub fn start() -> AllocLog {
            let editor = super::editor_lock();
            AllocLog {
                recorder: Recorder::start(),
                editor,
            }
        }

        /// The editor lock this recording holds, to hand to a helper that
        /// needs one — [`check_emsg`](super::check_emsg), say.
        pub fn editor(&self) -> &super::Editor {
            &self.editor
        }

        /// Assert the events recorded since the last check, and start over —
        /// `alloc_log:check(exp)`, which also clears.
        #[track_caller]
        pub fn check(&self, expected: &[AllocEvent]) {
            let actual = self.recorder.take();
            assert_eq!(actual, expected, "allocation sequence");
        }

        /// [`check`](Self::check), with the temporary allocations dropped
        /// first — `alloc_log:clear_tmp_allocs(..)` followed by a check.
        #[track_caller]
        pub fn check_net(&self, clear_null_frees: bool, expected: &[AllocEvent]) {
            let mut actual = self.recorder.take();
            clear_tmp_allocs(&mut actual, clear_null_frees);
            assert_eq!(actual, expected, "net allocation sequence");
        }

        /// Everything recorded since the last check, oldest first, leaving
        /// the log empty. For a case whose assertion is not a fixed
        /// sequence — "did this allocate at all".
        pub fn take(&self) -> Vec<AllocEvent> {
            self.recorder.take()
        }

        /// Forget everything recorded so far — `alloc_log:clear()`.
        pub fn clear(&self) {
            self.recorder.clear();
        }
    }

    /// `tv_list_alloc`'s allocation: `a.list(l)`.
    pub fn list(l: *const list_T) -> AllocEvent {
        AllocEvent::Calloc {
            count: 1,
            size: size_of::<list_T>(),
            ret: l as *mut c_void,
        }
    }

    /// `tv_list_item_alloc`'s allocation: `a.li(li)`.
    pub fn li(li: *const listitem_T) -> AllocEvent {
        AllocEvent::Malloc {
            size: size_of::<listitem_T>(),
            ret: li as *mut c_void,
        }
    }

    /// `tv_dict_alloc`'s allocation: `a.dict(d)`.
    pub fn dict(d: *const dict_T) -> AllocEvent {
        AllocEvent::Calloc {
            count: 1,
            size: size_of::<dict_T>(),
            ret: d as *mut c_void,
        }
    }

    /// `tv_dict_item_alloc_len`'s allocation: `a.di(di, key_len)`.
    ///
    /// The size is the whole point of the case — a `dictitem_T` is
    /// over-allocated so the NUL-terminated key fits in its flexible `di_key`
    /// member, but never below the struct's own size.
    pub fn di(di: *const dictitem_T, key_len: usize) -> AllocEvent {
        AllocEvent::Malloc {
            size: size_of::<dictitem_T>().max(offset_of!(dictitem_T, di_key) + key_len + 1),
            ret: di as *mut c_void,
        }
    }

    /// A NUL-terminated copy of `len` bytes: `a.str(s, len)`.
    pub fn string(s: *const c_char, len: usize) -> AllocEvent {
        AllocEvent::Malloc {
            size: len + 1,
            ret: s as *mut c_void,
        }
    }

    /// `tv_dict_watcher_add`'s allocation: `a.dwatcher(w)`.
    pub fn dwatcher(w: *const DictWatcher) -> AllocEvent {
        AllocEvent::Malloc {
            size: size_of::<DictWatcher>(),
            ret: w as *mut c_void,
        }
    }

    /// A `partial_T` built by the harness rather than by the code under
    /// test: the spec's `a.lua_pt(pt)`.
    pub fn partial(pt: *const partial_T) -> AllocEvent {
        AllocEvent::Calloc {
            count: 1,
            size: size_of::<partial_T>(),
            ret: pt as *mut c_void,
        }
    }

    /// A partial's argument vector, likewise the harness's: the spec's
    /// `a.lua_tvs(argv, argc)`.
    pub fn argv(argv: *const typval_T, argc: usize) -> AllocEvent {
        AllocEvent::Malloc {
            size: size_of::<typval_T>() * argc,
            ret: argv as *mut c_void,
        }
    }

    /// A release: `a.freed(p)`.
    pub fn freed<T>(p: *const T) -> AllocEvent {
        AllocEvent::Free {
            ptr: p as *mut c_void,
        }
    }
}

/// The Rust twin of `typval_spec.lua`'s `check_emsg`: run `f` and assert
/// whether it pushed `msg` onto the message history.
///
/// `None` asserts the history did not move, which is how the spec says "this
/// call reported nothing".
///
/// The [`Editor`] token is the lock this reads the message history under;
/// see [`editor_lock`].
#[cfg(not(miri))]
#[track_caller]
pub fn check_emsg<R>(editor: &Editor, f: impl FnOnce() -> R, msg: Option<&str>) -> R {
    check_emsg_bytes(editor, f, msg.map(str::as_bytes))
}

/// [`check_emsg`] over the message's raw bytes.
///
/// A decoder error quotes the input it choked on, which is not always
/// valid UTF-8 — and a lossy decode would turn every invalid byte into the
/// same replacement character, so the assertion has to be over bytes.
#[cfg(not(miri))]
#[track_caller]
pub fn check_emsg_bytes<R>(_editor: &Editor, f: impl FnOnce() -> R, msg: Option<&[u8]>) -> R {
    use c2rust_neovim::message::msg_hist_last;

    let before = msg_hist_last.get();
    let ret = f();
    let after = msg_hist_last.get();
    match msg {
        Some(expected) => {
            assert!(
                !after.is_null(),
                "expected the message {:?}, got none",
                String::from_utf8_lossy(expected)
            );
            assert_ne!(
                before,
                after,
                "expected a new message: {:?}",
                String::from_utf8_lossy(expected)
            );
            let chunk = unsafe { (*(*after).msg.items).clone() };
            let text = unsafe { CStr::from_ptr(chunk.text.data()) };
            assert_eq!(
                String::from_utf8_lossy(text.to_bytes()),
                String::from_utf8_lossy(expected)
            );
            assert_eq!(text.to_bytes(), expected);
        }
        None => assert_eq!(before, after, "unexpected message"),
    }
    ret
}
