//! The `.un~` file format. These bytes outlive the editor: a build that
//! writes a header another build cannot read loses every user's persistent
//! undo, so the constants and the encoder are asserted from outside the
//! crate as well as from inside it.

use c2rust_neovim::undo::format::*;

/// The nine bytes at offset 0 of every undo file since the format's
/// inception. `Vim`, 0x9f, `UnDo`, 0xe5.
#[test]
fn the_header_starts_with_the_documented_magic() {
    assert_eq!(&UF_START_MAGIC, b"Vim\x9fUnDo\xe5");
    assert_eq!(UF_START_MAGIC_LEN as usize, UF_START_MAGIC.len());
}

/// What a hex dump of the first eleven bytes shows.
#[test]
fn the_version_follows_the_magic_as_two_big_endian_bytes() {
    let mut head = Vec::from(UF_START_MAGIC);
    head.extend_from_slice(&encode_be(UF_VERSION as u64, 2)[..2]);
    assert_eq!(head, b"Vim\x9fUnDo\xe5\x00\x03");
}

#[test]
fn the_record_markers_are_unchanged() {
    assert_eq!(UF_HEADER_MAGIC, 0x5fd0);
    assert_eq!(UF_HEADER_END_MAGIC, 0xe7aa);
    assert_eq!(UF_ENTRY_MAGIC, 0xf518);
    assert_eq!(UF_ENTRY_END_MAGIC, 0x3581);
    assert_eq!(UF_LAST_SAVE_NR, 1);
    assert_eq!(UHP_SAVE_NR, 1);
}

/// Every field width the writer actually uses, both directions.
#[test]
fn every_field_width_round_trips() {
    let cases: &[(u64, usize)] = &[
        (0, 1),
        (1, 1),
        (0xff, 1),
        (UF_HEADER_MAGIC as u64, 2),
        (UF_ENTRY_END_MAGIC as u64, 2),
        (0xffff, 2),
        (0x7fff_ffff, 4),
        (0xffff_ffff, 4),
        // `uh_time` and `b_u_time_cur` are eight-byte fields.
        (0x0000_0000_6800_0000, 8),
        (u64::MAX, 8),
    ];
    for &(nr, len) in cases {
        let encoded = encode_be(nr, len);
        assert_eq!(decode_be(&encoded[..len]), nr, "{nr:#x} in {len} bytes");
        // Most significant byte first, and nothing written past the field.
        assert_eq!(encoded[0], (nr >> ((len - 1) * 8)) as u8);
        assert!(encoded[len..].iter().all(|&b| b == 0));
    }
}

/// A sequence number wider than its field keeps its low bytes rather than
/// saturating or erroring — the C shifted and truncated, and undo files in
/// the wild were written that way.
#[test]
fn an_oversized_value_truncates_from_the_top() {
    assert_eq!(&encode_be(0x1234, 1)[..1], &[0x34]);
    assert_eq!(&encode_be(0x1_0000_0001, 4)[..4], &[0, 0, 0, 1]);
}

// ---------------------------------------------------------------- u_write_undo
//
// Ported from `test/unit/undo_spec.lua`, which drove the same three entry
// points through LuaJIT's FFI. The Lua fixture built its buffer with
// `buflist_new()` and read `b_ffname` back off it; that is what forced
// `file_buffer`'s C layout to stay frozen, and it is the only thing the port
// does differently — a zeroed `buf_T` with the two fields the writer reads
// set by hand says the same thing about `u_write_undo` without pinning a
// layout. Everything else is the same call with the same assertion.
//
// Three of the spec's cases were empty `TODO`s upstream and carry nothing:
// "does not write an undofile when the buffer has no valid undofile name"
// (needs `u_get_undo_file_name()` to answer NULL), "does not overwrite an
// existing file that is not an undo file", and "does not overwrite an
// existing file that has the wrong permissions".

#[cfg(not(miri))]
mod write {
    use std::ffi::{CString, c_char};
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};

    use c2rust_neovim::main::{curbuf, p_udir};
    use c2rust_neovim::memory::xfree;
    use c2rust_neovim::types::buf_T;
    use c2rust_neovim::undo::format::UF_START_MAGIC;
    use c2rust_neovim::undo::{UNDO_HASH_SIZE, u_compute_hash, u_get_undo_file_name, u_write_undo};

    /// `p_udir` and `curbuf` are process-wide, so the cases run one at a
    /// time and put them back. Poisoning is ignored: a panicking case has
    /// already reported its own failure, and the next one restores the
    /// globals itself.
    static EDITOR: Mutex<()> = Mutex::new(());

    /// One case's editor state: an `'undodir'` of its own, a buffer with
    /// enough of `u_write_undo`'s inputs filled in to be worth writing, and
    /// the hash the writer stamps into the header.
    struct Fixture {
        _guard: MutexGuard<'static, ()>,
        dir: PathBuf,
        buf: Box<buf_T>,
        hash: [u8; UNDO_HASH_SIZE as usize],
        // Owns the bytes `p_udir`/`b_ffname` point at for the case's life.
        _udir: CString,
        ffname: Option<CString>,
        old_udir: *mut c_char,
        old_curbuf: *mut buf_T,
    }

    impl Fixture {
        fn new(case: &str) -> Fixture {
            let guard = EDITOR.lock().unwrap_or_else(|e| e.into_inner());
            let dir = std::env::temp_dir().join(format!("nvim-unit-undo-{case}"));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            let udir = CString::new(dir.to_str().unwrap()).unwrap();

            // `buflist_new()` leaves a fresh buffer synced with no undo
            // header; the spec then set `b_u_numhead` to pretend the buffer
            // had been changed, which is what makes the writer write.
            let mut buf: Box<buf_T> = Box::new(unsafe { std::mem::zeroed() });
            buf.b_u_synced = true;
            buf.b_u_numhead = 1;

            let old_udir = p_udir.get();
            let old_curbuf = curbuf.get();
            p_udir.set(udir.as_ptr().cast_mut());
            // `u_write_undo` syncs the current buffer before serialising;
            // ours is already synced, so this only has to be non-NULL.
            curbuf.set(&raw mut *buf);

            let mut fixture = Fixture {
                _guard: guard,
                dir,
                buf,
                hash: [0; UNDO_HASH_SIZE as usize],
                _udir: udir,
                ffname: None,
                old_udir,
                old_curbuf,
            };
            // SAFETY: the buffer is live and `hash` is `UNDO_HASH_SIZE` long.
            unsafe { u_compute_hash(&raw mut *fixture.buf, fixture.hash.as_mut_ptr()) };
            fixture
        }

        fn path(&self, name: &str) -> PathBuf {
            self.dir.join(name)
        }

        /// Point the buffer at `path` the way `buflist_new()` would: an
        /// absolute name, which is what `u_get_undo_file_name` mangles.
        fn set_ffname(&mut self, path: &std::path::Path) {
            let ffname = CString::new(path.to_str().unwrap()).unwrap();
            self.buf.b_ffname = ffname.as_ptr().cast_mut();
            self.ffname = Some(ffname);
        }

        /// Where `u_write_undo(NULL, ..)` puts this buffer's undo file.
        fn undo_file_name(&self) -> PathBuf {
            // SAFETY: `b_ffname` is this fixture's own NUL-terminated name.
            let name = unsafe { u_get_undo_file_name(self.buf.b_ffname, false) };
            assert!(!name.is_null(), "no undo file name for the buffer");
            // SAFETY: an `xmalloc`ed NUL-terminated string, ours to free.
            let owned = unsafe { std::ffi::CStr::from_ptr(name).to_bytes().to_vec() };
            // SAFETY: as above.
            unsafe { xfree(name.cast()) };
            PathBuf::from(String::from_utf8(owned).unwrap())
        }

        /// `u_write_undo(name, forceit, buf, hash)`.
        fn write(&mut self, name: Option<&std::path::Path>, forceit: bool) {
            let name = name.map(|p| CString::new(p.to_str().unwrap()).unwrap());
            let ptr = name.as_ref().map_or(std::ptr::null(), |n| n.as_ptr());
            // SAFETY: the buffer and hash are the fixture's, and `ptr` is
            // either NULL or a NUL-terminated name alive for the call.
            unsafe { u_write_undo(ptr, forceit, &raw mut *self.buf, self.hash.as_mut_ptr()) };
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            p_udir.set(self.old_udir);
            curbuf.set(self.old_curbuf);
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn is_undo_file(path: &std::path::Path) -> bool {
        fs::read(path).is_ok_and(|bytes| bytes.starts_with(&UF_START_MAGIC))
    }

    /// The hash the spec computed in its setup and then only ever passed
    /// through: an empty buffer hashes the empty input.
    #[test]
    fn the_buffer_hash_is_a_sha256_of_the_lines() {
        let fixture = Fixture::new("hash");
        assert_eq!(
            fixture.hash,
            [
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55,
            ]
        );
    }

    /// "writes an undo file to undodir given a buffer and hash".
    #[test]
    fn writes_an_undo_file_to_undodir() {
        let mut fixture = Fixture::new("undodir");
        let target = fixture.path("Xtest-unit-undo");
        fixture.set_ffname(&target);

        fixture.write(None, false);

        let written = fixture.undo_file_name();
        assert!(written.exists(), "{} was not written", written.display());
        assert!(is_undo_file(&written));
    }

    /// "writes a correctly-named undo file to undodir given a name, buffer,
    /// and hash" — an explicit name wins over the mangled one.
    #[test]
    fn writes_the_named_undo_file_when_given_a_name() {
        let mut fixture = Fixture::new("named");
        fixture.set_ffname(&fixture.path("Xtest-unit-undo"));
        let name = fixture.path("undofile.test");

        fixture.write(Some(&name), false);

        assert!(name.exists(), "{} was not written", name.display());
        assert!(is_undo_file(&name));
        // And nothing landed under the mangled name.
        assert!(!fixture.undo_file_name().exists());
    }

    /// "writes the undofile with the same permissions as the original file".
    #[test]
    fn the_undo_file_inherits_the_original_file_permissions() {
        let mut fixture = Fixture::new("perm");
        let original = fixture.path("test.file");
        fs::write(&original, b"testing permissions").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o640)).unwrap();
        let expected = fs::metadata(&original).unwrap().mode();
        fixture.set_ffname(&original);

        fixture.write(None, false);

        let written = fixture.undo_file_name();
        assert_eq!(fs::metadata(&written).unwrap().mode(), expected);
    }

    /// "writes an undofile only readable by the user if the buffer is
    /// unnamed" — 33152 is `S_IFREG | 0600`.
    #[test]
    fn an_unnamed_buffer_writes_an_owner_only_undo_file() {
        let mut fixture = Fixture::new("unnamed");
        // `b_ffname` stays NULL, so there is nothing to copy a mode from.
        let name = fixture.path("test.undo");

        fixture.write(Some(&name), false);

        assert_eq!(fs::metadata(&name).unwrap().mode(), 33152);
    }

    /// "forces writing undo file for :wundo! command" — `forceit` replaces
    /// a file that is not an undo file at all.
    #[test]
    fn forceit_overwrites_a_file_that_is_not_an_undo_file() {
        let mut fixture = Fixture::new("forceit");
        fixture.set_ffname(&fixture.path("Xtest-unit-undo"));
        let name = fixture.undo_file_name();
        let contents = b"testing permissions";
        fs::write(&name, contents).unwrap();

        fixture.write(Some(&name), true);

        assert_ne!(fs::read(&name).unwrap(), contents);
        assert!(is_undo_file(&name));
    }

    /// "overwrites an existing undo file". The spec slept a second and
    /// compared mtimes to a one-second resolution; the writer unlinks and
    /// re-creates, so the inode says the same thing without the sleep.
    #[test]
    fn a_second_write_replaces_the_existing_undo_file() {
        let mut fixture = Fixture::new("overwrite");
        fixture.set_ffname(&fixture.path("Xtest-unit-undo"));

        fixture.write(None, false);
        let written = fixture.undo_file_name();
        let first = fs::metadata(&written).unwrap();

        fixture.buf.b_u_numhead = 1; // Mark it as if there are changes again.
        fixture.write(None, false);

        let second = fs::metadata(&written).unwrap();
        assert_ne!(first.ino(), second.ino(), "the undo file was not replaced");
        assert!(second.modified().unwrap() >= first.modified().unwrap());
        assert!(is_undo_file(&written));
    }

    /// "does not write an undo file if there is no undo information for the
    /// buffer".
    #[test]
    fn nothing_is_written_when_the_buffer_has_no_undo_information() {
        let mut fixture = Fixture::new("nothing");
        fixture.set_ffname(&fixture.path("Xtest-unit-undo"));
        fixture.buf.b_u_numhead = 0;

        let written = fixture.undo_file_name();
        assert!(!written.exists());
        fixture.write(None, false);

        assert!(!written.exists(), "{} was written", written.display());
    }
}
