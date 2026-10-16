//! The `.un~` file format. These bytes outlive the editor: a build that
//! writes a header another build cannot read loses every user's persistent
//! undo, so the constants and the encoder are asserted from outside the
//! crate as well as from inside it.

use neovim::undo::format::*;

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

/// A header link is the same four-byte big-endian sequence number the file
/// carries, both ways round. `put_header_link` writes `link.seq()` and
/// `unserialize_uhp` reads it back through `UndoLink::to_seq`, so this is the
/// whole of the on-disk link format.
#[test]
fn a_link_round_trips_through_its_four_byte_field() {
    use neovim::types::UndoLink;

    for seq in [1, 2, 0x7f, 0x80, 0xffff, 0x0100_0000, i32::MAX] {
        let link = UndoLink::to_seq(seq);
        let field = encode_be(link.seq() as u64, 4);
        let read_back = i32::from_be_bytes([field[0], field[1], field[2], field[3]]);
        assert_eq!(UndoLink::to_seq(read_back), link, "seq {seq}");
    }
    // "No link" is the zero the writer put down for a NULL pointer, and a
    // negative field -- which no writer produces and only a corrupt file
    // has -- reads back as no link rather than as a lookup.
    assert_eq!(
        &encode_be(UndoLink::NONE.seq() as u64, 4)[..4],
        &[0, 0, 0, 0]
    );
    assert!(UndoLink::to_seq(-1).is_none());
}

// ---------------------------------------------------------------- u_write_undo
//
// Ported from `test/unit/undo_spec.lua`, which drove the same three entry
// points through LuaJIT's FFI. The Lua fixture built its buffer with
// `buflist_new()` and read `b_ffname` back off it; that is what forced
// `file_buffer`'s C layout to stay frozen, and it is the only thing the port
// does differently — a zeroed `Buffer` with the two fields the writer reads
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
    use std::ffi::CString;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::PathBuf;

    use neovim::memory::XString;
    use neovim::option::vars::P_UDIR;
    use neovim::optionstr::BUF_STRING_OPTIONS;
    use neovim::types::{BufName, Buffer};
    use neovim::undo::format::UF_START_MAGIC;
    use neovim::undo::{UNDO_HASH_SIZE, u_compute_hash, u_get_undo_file_name, u_write_undo};
    use neovim::winlayer::Buf;

    use crate::support::{Editor, editor_lock};

    /// One case's editor state: an `'undodir'` of its own, a buffer with
    /// enough of `u_write_undo`'s inputs filled in to be worth writing, and
    /// the hash the writer stamps into the header.
    struct Fixture {
        // `p_udir` and the current buffer's `b_u_synced` are process-wide,
        // so the cases run one at a time and put them back. This is the
        // *same* lock every other case in this binary takes: a second mutex
        // over the same globals would serialise these cases against each
        // other and against nothing else.
        _guard: Editor,
        dir: PathBuf,
        buf: Box<Buffer>,
        hash: [u8; UNDO_HASH_SIZE as usize],
        // Owns the bytes `p_udir` points at for the case's life; the
        // buffer's own name it owns itself.
        _udir: CString,
        old_udir: Option<XString>,
        old_synced: bool,
    }

    impl Fixture {
        fn new(case: &str) -> Fixture {
            let guard = editor_lock();
            let dir = std::env::temp_dir().join(format!("nvim-unit-undo-{case}"));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            let udir = CString::new(dir.to_str().unwrap()).unwrap();

            // `buflist_new()` leaves a fresh buffer synced with no undo
            // header; the spec then set `b_u_numhead` to pretend the buffer
            // had been changed, which is what makes the writer write.
            let mut buf: Box<Buffer> = {
                let mut storage = Box::<Buffer>::new_zeroed();
                // SAFETY: all-zero bytes are what upstream's `xcalloc` hands
                // a fresh buffer, except for `b_ucmds`, whose empty `Vec`
                // holds a non-null dangling pointer, and the string
                // options and the `U` line, whose `Option<XString>` reads as
                // a `Some` over a null pointer when zeroed. Both are written before
                // anything can read or drop them -- and this fixture's
                // buffer *is* dropped, so a garbage `Some` would be freed.
                unsafe {
                    (&raw mut (*storage.as_mut_ptr()).b_ucmds).write(Vec::new());
                    (&raw mut (*storage.as_mut_ptr()).name).write(BufName::default());
                    (&raw mut (*storage.as_mut_ptr()).b_u_line).write(None);
                    for field in BUF_STRING_OPTIONS {
                        let at = storage.as_mut_ptr().wrapping_byte_add(field.field_offset());
                        at.cast::<Option<XString>>().write(None);
                    }
                    storage.assume_init()
                }
            };
            buf.b_u_synced = true;
            buf.b_u_numhead = 1;

            let old_udir = P_UDIR.swap(Some(XString::from_cstr(&udir)));
            // `u_write_undo` syncs the *current* buffer before serialising
            // the one it was handed, and this fixture's buffer is not one
            // the editor can stand in: it is in no registry, and "current"
            // is the identity of a registered buffer. So the editor's own
            // buffer stays current and is only marked synced, which is all
            // that pointing `curbuf` at an already-synced fixture bought --
            // `u_sync` returns at its first test either way.
            let mut current = Buf::current();
            let old_synced = current.b_u_synced;
            current.b_u_synced = true;

            let mut fixture = Fixture {
                _guard: guard,
                dir,
                buf,
                hash: [0; UNDO_HASH_SIZE as usize],
                _udir: udir,
                old_udir,
                old_synced,
            };
            // SAFETY: the buffer is live.
            let buf = unsafe { Buf::new(&raw mut *fixture.buf) };
            u_compute_hash(buf, &mut fixture.hash);
            fixture
        }

        fn path(&self, name: &str) -> PathBuf {
            self.dir.join(name)
        }

        /// Point the buffer at `path` the way `buflist_new()` would: an
        /// absolute name, which is what `u_get_undo_file_name` mangles.
        fn set_ffname(&mut self, path: &std::path::Path) {
            let ffname = CString::new(path.to_str().unwrap()).unwrap();
            self.buf.name.set(XString::from_cstr(&ffname), None);
        }

        /// Where `u_write_undo(NULL, ..)` puts this buffer's undo file.
        fn undo_file_name(&self) -> PathBuf {
            let name = u_get_undo_file_name(self.buf.name.full(), false)
                .expect("no undo file name for the buffer");
            PathBuf::from(String::from_utf8(name.to_vec()).unwrap())
        }

        /// `u_write_undo(name, forceit, buf, hash)`.
        fn write(&mut self, name: Option<&std::path::Path>, forceit: bool) {
            let name = name.map(|p| CString::new(p.to_str().unwrap()).unwrap());
            // SAFETY: the buffer is the fixture's, and live.
            let buf = unsafe { Buf::new(&raw mut *self.buf) };
            u_write_undo(name.as_deref(), forceit, buf, &self.hash);
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            P_UDIR.restore(self.old_udir.take());
            Buf::current().b_u_synced = self.old_synced;
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
    /// re-creates instead, so an open handle on the first file answers the
    /// same question without the sleep. Its link count drops to zero exactly
    /// when the file it names has been unlinked. Comparing inode numbers
    /// would not do: the new file is free to land on the number the old one
    /// just released, and on ext4 it usually does.
    #[test]
    fn a_second_write_replaces_the_existing_undo_file() {
        let mut fixture = Fixture::new("overwrite");
        fixture.set_ffname(&fixture.path("Xtest-unit-undo"));

        fixture.write(None, false);
        let written = fixture.undo_file_name();
        let first = fs::File::open(&written).unwrap();
        let first_meta = first.metadata().unwrap();

        fixture.buf.b_u_numhead = 1; // Mark it as if there are changes again.
        fixture.write(None, false);

        let replaced = first.metadata().unwrap();
        assert_eq!(replaced.nlink(), 0, "the undo file was not replaced");
        let second = fs::metadata(&written).unwrap();
        assert!(second.modified().unwrap() >= first_meta.modified().unwrap());
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

// ---------------------------------------------------------------- the tree
//
// These drive the editor through commands and read the tree back only
// through what a user can see: the text, `undotree()`, and what `:rundo`
// says about a file. Every expectation was recorded on the build before the
// entry chain and the header store were retyped.

#[cfg(not(miri))]
mod tree {
    use crate::support::{Probe, Sandbox};

    /// A scratch buffer of the case's own, with no swap file, and every
    /// change its own undo step.
    fn scratch(fx: &Probe) {
        fx.run("new");
        fx.run("only!");
        fx.run("setlocal noswapfile undolevels=1000");
    }

    fn unscratch(fx: &Probe) {
        fx.run("silent! bwipeout!");
        fx.home();
    }

    /// Close the undo step: setting 'undolevels' syncs, whatever the value.
    fn sync(fx: &Probe) {
        fx.run("let &l:undolevels = &l:undolevels");
    }

    /// The offset of the first header's first entry's line count in an undo
    /// file, worked out from the layout `serialize_header`/`serialize_uhp`
    /// write: the file header (with its `U` line), the header magic, and the
    /// header's fixed fields up to its first entry.
    fn first_entry_size_at(file: &[u8]) -> usize {
        let u_line_len = u32::from_be_bytes(file[47..51].try_into().unwrap()) as usize;
        // lnum, col, three heads, numhead, seq_last, seq_cur (4 each), the
        // time (8) and the optional-field trailer (7).
        let header_magic = 51 + u_line_len + 8 * 4 + 8 + 7;
        assert_eq!(&file[header_magic..header_magic + 2], &[0x5f, 0xd0]);
        // Links (16), seq (4), cursor (12), vcol (4), flags (2), 26 marks
        // (312), Visual (32), time (8), trailer (7).
        let entry_magic = header_magic + 2 + 16 + 4 + 12 + 4 + 2 + 312 + 32 + 8 + 7;
        assert_eq!(&file[entry_magic..entry_magic + 2], &[0xf5, 0x18]);
        // top, bot, lcount, then the size.
        entry_magic + 2 + 12
    }

    /// `undotree()` without the clock: every `time` and `time_cur` masked.
    const TREE: &str =
        "substitute(string(undotree()), '\\(time\\(_cur\\)\\=\\)'': \\d\\+', '\\1'': T', 'g')";

    /// A branch built, written out, read back into a fresh load of the same
    /// text, and walked with `g-`/`g+`: the tree survives the round trip
    /// whole, alternates included.
    #[test]
    fn a_branch_survives_wundo_and_rundo() {
        let sandbox = Sandbox::dir("undo-branch-roundtrip");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, 'one')");
        sync(&fx);
        fx.run("call setline(1, 'two')");
        sync(&fx);
        fx.run("undo");
        fx.run("call setline(1, 'three')");
        sync(&fx);
        fx.run("call append(1, 'h\u{e9}llo w\u{f6}rld')");
        sync(&fx);
        fx.run("silent write Xbranch");
        let before = fx.eval(TREE);
        fx.run("wundo Xbranch.un");
        fx.run("silent edit! Xbranch");
        fx.run("silent rundo Xbranch.un");
        fx.check(TREE, &before);
        fx.check("getline(1, '$')", "['three', 'h\u{e9}llo w\u{f6}rld']");
        for (step, want) in [
            ("g-", "['g-', ['three'], 3]"),
            ("g-", "['g-', ['two'], 2]"),
            ("g-", "['g-', ['one'], 1]"),
            ("g-", "['g-', [''], 0]"),
            ("g+", "['g+', ['one'], 1]"),
            ("g+", "['g+', ['two'], 2]"),
            ("g+", "['g+', ['three'], 3]"),
            ("g+", "['g+', ['three', 'h\u{e9}llo w\u{f6}rld'], 4]"),
            ("g+", "['g+', ['three', 'h\u{e9}llo w\u{f6}rld'], 4]"),
        ] {
            fx.run(&format!("silent normal! {step}"));
            fx.check(&format!("[{step:?}, getline(1, '$'), changenr()]"), want);
        }
        unscratch(&fx);
    }

    /// `U` keeps its line in the undo file too: read back, it puts the line
    /// back as it was before the changes to it began.
    #[test]
    fn the_u_line_survives_wundo_and_rundo() {
        let sandbox = Sandbox::dir("undo-uline-roundtrip");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, ['first', 'second'])");
        sync(&fx);
        fx.run("silent write Xuline");
        fx.run("normal! 2Gx");
        fx.run("normal! x");
        fx.check("getline(1, '$')", "['first', 'cond']");
        fx.run("normal! U");
        fx.check("getline(1, '$')", "['first', 'second']");
        fx.run("normal! U");
        fx.check("getline(1, '$')", "['first', 'cond']");
        fx.run("silent write");
        fx.run("wundo Xuline.un");
        fx.run("silent edit! Xuline");
        fx.run("silent rundo Xuline.un");
        fx.run("normal! 2G");
        fx.run("normal! U");
        fx.check("getline(1, '$')", "['first', 'second']");
        unscratch(&fx);
    }

    /// 'undolevels' at -1 records nothing and drops the tree; set back, the
    /// next change starts a fresh one.
    #[test]
    fn undolevels_off_then_back_on() {
        let sandbox = Sandbox::dir("undo-levels-off");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, 'a')");
        sync(&fx);
        fx.run("call setline(1, 'b')");
        sync(&fx);
        fx.run("setlocal undolevels=-1");
        fx.run("call setline(1, 'c')");
        fx.check(
            TREE,
            "'{''seq_last'': 2, ''entries'': [], ''time_cur'': T, ''save_last'': 0, ''synced'': 0, \
             ''save_cur'': 0, ''seq_cur'': 2}'",
        );
        fx.run("setlocal undolevels=1000");
        fx.run("call setline(1, 'd')");
        sync(&fx);
        fx.check(
            TREE,
            "'{''seq_last'': 3, ''entries'': [{''seq'': 3, ''time'': T, ''newhead'': 1}], \
             ''time_cur'': T, ''save_last'': 0, ''synced'': 1, ''save_cur'': 0, ''seq_cur'': 3}'",
        );
        fx.run("silent undo");
        fx.check("getline(1)", "'c'");
        fx.run("silent undo");
        fx.check("getline(1)", "'c'");
        unscratch(&fx);
    }

    /// The undo file's own checks: text that no longer hashes the same, a
    /// file that stops in the middle of an entry, and a version from the
    /// future are each refused with their own message, and the tree the
    /// buffer had is left alone.
    #[test]
    fn a_bad_undo_file_is_refused() {
        let sandbox = Sandbox::dir("undo-bad-file");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, ['alpha', 'beta', 'gamma'])");
        sync(&fx);
        fx.run("call setline(2, 'BETA')");
        sync(&fx);
        fx.run("silent write Xbad");
        fx.run("wundo Xbad.un");
        let good = std::fs::read(sandbox.path("Xbad.un")).expect("the undo file");

        // Different text: the hash moved.
        fx.run("call setline(3, 'GAMMA')");
        // The warning goes to the screen, not to `execute()`.
        fx.check_output("rundo Xbad.un", "");
        fx.run("silent undo");

        // Cut in the middle of the last header's entries.
        sandbox.write("Xcut.un", &good[..good.len() - 40]);
        fx.check_error(
            "rundo Xcut.un",
            "E825: Corrupted undo file (extmark truncated): Xcut.un",
        );
        // Cut inside the file header.
        sandbox.write("Xshort.un", &good[..50]);
        // Silently: upstream rejects a shadow-line length it cannot read
        // without a word.
        fx.check_error("rundo Xshort.un", "none");
        // A version this build does not know.
        let mut future = good.clone();
        future[9..11].copy_from_slice(&[0, 4]);
        sandbox.write("Xfuture.un", &future);
        fx.check_error(
            "rundo Xfuture.un",
            "E824: Incompatible undo file: Xfuture.un",
        );
        // Not an undo file at all.
        sandbox.write("Xnot.un", b"not an undo file");
        fx.check_error("rundo Xnot.un", "E823: Not an undo file: Xnot.un");
        // A negative size on the first header's first entry.
        let mut negative = good.clone();
        let size_at = first_entry_size_at(&good);
        negative[size_at..size_at + 4].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        sandbox.write("Xneg.un", &negative);
        fx.check_error(
            "rundo Xneg.un",
            "E825: Corrupted undo file (entry size): Xneg.un",
        );
        // That entry holds no lines, so what follows its size is the next
        // record's marker: break it.
        let mut broken = good.clone();
        broken[size_at + 4..size_at + 8].copy_from_slice(&[0x7f, 0xff, 0xff, 0xff]);
        sandbox.write("Xbroken.un", &broken);
        fx.check_error(
            "rundo Xbroken.un",
            "E825: Corrupted undo file (entry end): Xbroken.un",
        );
        // An extmark record's payload edited past what a change could make
        // (the last record of the file is the last header's extmark).
        let mut wild = good.clone();
        let mark = good
            .windows(2)
            .rposition(|w| w == [0xf5, 0x18])
            .expect("an entry magic");
        wild[mark + 14..mark + 18].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        sandbox.write("Xwild.un", &wild);
        fx.check_error(
            "rundo Xwild.un",
            "E825: Corrupted undo file (extmark splice): Xwild.un",
        );
        // The good one still reads, after all of that.
        fx.check_error("rundo Xbad.un", "none");
        fx.check(
            TREE,
            "'{''seq_last'': 2, ''entries'': [{''seq'': 1, ''time'': T}, {''save'': 1, ''seq'': 2, \
             ''time'': T, ''newhead'': 1}], ''time_cur'': T, ''save_last'': 1, ''synced'': 1, \
             ''save_cur'': 1, ''seq_cur'': 2}'",
        );
        unscratch(&fx);
    }

    /// `:earlier 1f` and `:later 1f` move by file write.
    #[test]
    fn earlier_and_later_by_file_write() {
        let sandbox = Sandbox::dir("undo-earlier-file");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, 'zero')");
        sync(&fx);
        fx.run("silent write Xfw");
        fx.run("call setline(1, 'one')");
        sync(&fx);
        fx.run("call setline(1, 'one more')");
        sync(&fx);
        fx.run("silent write");
        fx.run("call setline(1, 'two')");
        sync(&fx);
        fx.run("silent write");
        fx.run("call setline(1, 'unsaved')");
        sync(&fx);
        for (cmd, want) in [
            ("earlier 1f", "['earlier 1f', 'two', 4]"),
            ("earlier 1f", "['earlier 1f', 'one more', 3]"),
            ("earlier 1f", "['earlier 1f', 'zero', 1]"),
            ("earlier 1f", "['earlier 1f', '', 0]"),
            ("later 1f", "['later 1f', 'zero', 1]"),
            ("later 1f", "['later 1f', 'one more', 3]"),
            ("later 1f", "['later 1f', 'two', 4]"),
            ("later 1f", "['later 1f', 'unsaved', 5]"),
        ] {
            fx.run(&format!("silent {cmd}"));
            fx.check(&format!("[{cmd:?}, getline(1), changenr()]"), want);
        }
        fx.check(
            TREE,
            "'{''seq_last'': 5, ''entries'': [{''save'': 1, ''seq'': 1, ''time'': T}, {''seq'': 2, \
             ''time'': T}, {''save'': 2, ''seq'': 3, ''time'': T}, {''save'': 3, ''seq'': 4, \
             ''time'': T}, {''seq'': 5, ''time'': T, ''newhead'': 1}], ''time_cur'': T, \
             ''save_last'': 3, ''synced'': 1, ''save_cur'': 3, ''seq_cur'': 5}'",
        );
        unscratch(&fx);
    }

    /// A change refused before anything was saved (the buffer is not
    /// modifiable) leaves the undo step open for `:undojoin` to fold the
    /// next change into, and one undo then takes both back.
    #[test]
    fn undojoin_after_a_refused_change() {
        let sandbox = Sandbox::dir("undo-join-refused");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, 'base')");
        sync(&fx);
        fx.run("call setline(1, 'first')");
        fx.run("setlocal nomodifiable");
        fx.check_error(
            "normal! x",
            "E21: Cannot make changes, ''modifiable'' is off",
        );
        fx.run("setlocal modifiable");
        fx.run("undojoin | call setline(1, 'joined')");
        sync(&fx);
        fx.run("silent undo");
        fx.check("getline(1)", "'base'");
        // And after an undo `:undojoin` is refused outright.
        fx.check_error("undojoin", "E790: undojoin is not allowed after undo");
        fx.check(
            TREE,
            "'{''seq_last'': 2, ''entries'': [{''seq'': 1, ''time'': T}, {''curhead'': 1, ''seq'': 2, \
             ''time'': T, ''newhead'': 1}], ''time_cur'': T, ''save_last'': 0, ''synced'': 1, \
             ''save_cur'': 0, ''seq_cur'': 1}'",
        );
        unscratch(&fx);
    }

    /// One-line changes to the same line fold into one entry, and a change
    /// to another line between them moves the first to the front: undo puts
    /// both lines back however they interleaved.
    #[test]
    fn interleaved_line_changes_undo_together() {
        let sandbox = Sandbox::dir("undo-interleaved");
        let fx = Probe::new(sandbox.editor());
        scratch(&fx);
        fx.run("call setline(1, ['l1', 'l2', 'l3', 'l\u{e9}4'])");
        sync(&fx);
        for (line, text) in [
            (1, "a"),
            (3, "b"),
            (1, "c"),
            (4, "d\u{fc}"),
            (3, "e"),
            (1, "f"),
        ] {
            fx.run(&format!("call setline({line}, '{text}')"));
        }
        fx.check("getline(1, '$')", "['f', 'l2', 'e', 'd\u{fc}']");
        sync(&fx);
        fx.run("silent undo");
        fx.check("getline(1, '$')", "['l1', 'l2', 'l3', 'l\u{e9}4']");
        fx.run("silent redo");
        fx.check("getline(1, '$')", "['f', 'l2', 'e', 'd\u{fc}']");
        fx.check("undotree().seq_last", "2");
        unscratch(&fx);
    }
}
