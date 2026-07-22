//! Reads a terminal's compiled terminfo description from the system database.
//!
//! This is an in-tree port of the subset of unibilium 2.1.2 that nvim used:
//! `unibi_from_term` (locate and parse the description for a terminal name)
//! plus the read-only capability getters. Copyright 2008-2013 Lukas Mai;
//! unibilium is LGPL-3.0-or-later, and this port stays under that license.
//!
//! Behavior matches the C byte for byte, quirks included, because which
//! description gets loaded (and whether loading fails, falling back to the
//! builtin entries) is user-visible:
//!
//! - At most 4096 bytes of a description file are read; larger entries fail
//!   to parse and nvim falls back to its builtin terminfo.
//! - The search order is `$TERMINFO`, `$HOME/.terminfo`, then `$TERMINFO_DIRS`
//!   *or* (only if unset) the compiled-in directory list. Each directory is
//!   tried as `dir/<first char>/<name>`, then `dir/<first byte in hex>/<name>`
//!   (the macOS layout).
//! - A corrupt description or unexpected I/O error aborts the whole search
//!   rather than moving on to the next directory.
//! - Capabilities are addressed by unibilium's chained enum values (booleans
//!   `1..=44`, numerics `46..=84`, strings `86..=499`), which terminfo.rs
//!   still uses; absent numerics read as -1 and absent strings as `None`.

use std::ffi::{CStr, CString, OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

/// `unibi_boolean_end_` .. `unibi_string_end_`: the cap-index ranges of
/// unibilium's chained enums. Booleans occupy `(BOOL_BEGIN, BOOL_END)`
/// exclusive, and so on; the offsets below convert a cap to its array index.
const BOOL_BEGIN: u32 = 0;
const BOOL_END: u32 = 45;
const NUM_BEGIN: u32 = 45;
const NUM_END: u32 = 85;
const STR_BEGIN: u32 = 85;
const STR_END: u32 = 500;

const NBOOL: usize = (BOOL_END - BOOL_BEGIN - 1) as usize; // 44
const NNUM: usize = (NUM_END - NUM_BEGIN - 1) as usize; // 39
const NSTR: usize = (STR_END - STR_BEGIN - 1) as usize; // 414

/// The compiled-in search path, consulted only when `$TERMINFO_DIRS` is
/// unset. cmake.deps configured unibilium's copy by probing `ncurses*-config
/// --terminfo-dirs` and falling back to this list; the Nix sandbox has no
/// ncurses, so every shipped binary got exactly this fallback.
const TERMINFO_DIRS: &[u8] = b"/etc/terminfo:/lib/terminfo:/usr/share/terminfo:/usr/lib/terminfo:\
      /usr/local/share/terminfo:/usr/local/lib/terminfo";

/// `unibi_from_fp`'s read cap: descriptions are read into a fixed 4096-byte
/// buffer, so anything larger is truncated and fails to parse.
const MAX_BUF: usize = 4096;

const MAGIC_16BIT: u16 = 0o432;
const MAGIC_32BIT: u16 = 0o1036;

/// A parsed terminfo description. The C `unibi_term` also kept the terminal
/// name/aliases and extended numeric values; nvim never reads those, so they
/// are validated during parsing but not stored.
pub struct Term {
    bools: [bool; NBOOL],
    nums: [i32; NNUM],
    strs: Vec<Option<CString>>,
    ext_bools: Vec<bool>,
    ext_bool_names: Vec<CString>,
    ext_strs: Vec<Option<CString>>,
    ext_str_names: Vec<CString>,
}

impl Term {
    /// `unibi_get_bool`: false when absent (or out of range).
    pub fn get_bool(&self, cap: u32) -> bool {
        debug_assert!(cap > BOOL_BEGIN && cap < BOOL_END);
        let i = cap.wrapping_sub(BOOL_BEGIN + 1) as usize;
        self.bools.get(i).copied().unwrap_or(false)
    }

    /// `unibi_get_num`: -1 when absent.
    pub fn get_num(&self, cap: u32) -> i32 {
        debug_assert!(cap > NUM_BEGIN && cap < NUM_END);
        let i = cap.wrapping_sub(NUM_BEGIN + 1) as usize;
        self.nums.get(i).copied().unwrap_or(-2)
    }

    /// `unibi_get_str`: `None` when absent.
    pub fn get_str(&self, cap: u32) -> Option<&CStr> {
        debug_assert!(cap > STR_BEGIN && cap < STR_END);
        let i = cap.wrapping_sub(STR_BEGIN + 1) as usize;
        self.strs.get(i)?.as_deref()
    }

    /// The names of the extended boolean capabilities that are set.
    pub fn ext_bool_names(&self) -> impl Iterator<Item = &CStr> {
        self.ext_bools
            .iter()
            .zip(&self.ext_bool_names)
            .filter(|(&set, _)| set)
            .map(|(_, name)| name.as_c_str())
    }

    /// The extended string capabilities, as (name, value) pairs. A `None`
    /// value is an entry whose offset was absent or out of range, which the
    /// C API also reported as a present name with a NULL value.
    pub fn ext_strs(&self) -> impl Iterator<Item = (&CStr, Option<&CStr>)> {
        self.ext_str_names
            .iter()
            .zip(&self.ext_strs)
            .map(|(name, val)| (name.as_c_str(), val.as_deref()))
    }
}

fn ru16(p: &[u8], off: usize) -> usize {
    u16::from_le_bytes([p[off], p[off + 1]]) as usize
}

/// `get_short16`: absent (0xffff) and cancelled (0xfffe) both land above
/// 0x7fff and read as -1.
fn short16(p: &[u8], off: usize) -> i32 {
    let n = ru16(p, off);
    if n <= 0x7fff {
        n as i32
    } else {
        -1
    }
}

/// `get_int32`: same top-bit rule for the 32-bit number format.
fn int32(p: &[u8], off: usize) -> i32 {
    let n = u32::from_le_bytes([p[off], p[off + 1], p[off + 2], p[off + 3]]);
    if n <= 0x7fff_ffff {
        n as i32
    } else {
        -1
    }
}

/// A NUL-terminated string starting at `off` in `table`, whose final byte
/// the parser has already forced to NUL (as the C did in its copy).
fn cstr_at(table: &[u8], off: usize) -> CString {
    let end = table[off..].iter().position(|&b| b == 0).unwrap() + off;
    CString::new(&table[off..end]).unwrap()
}

/// `unibi_from_mem`: parse a compiled terminfo description. `None` is the
/// C's NULL-with-errno for every malformed-input path.
pub fn from_mem(data: &[u8]) -> Option<Term> {
    let mut p = data;
    if p.len() < 12 {
        return None;
    }
    let magic = ru16(p, 0) as u16;
    let numsize = match magic {
        MAGIC_16BIT => 2,
        MAGIC_32BIT => 4,
        _ => return None,
    };
    let namlen = ru16(p, 2);
    let boollen = ru16(p, 4);
    let numlen = ru16(p, 6);
    let strslen = ru16(p, 8);
    let tablsz = ru16(p, 10);
    p = &p[12..];

    // Terminal name and aliases: validated, not stored.
    if p.len() < namlen {
        return None;
    }
    p = &p[namlen..];

    if p.len() < boollen {
        return None;
    }
    let mut bools = [false; NBOOL];
    for i in 0..boollen.min(NBOOL) {
        bools[i] = p[i] != 0;
    }
    p = &p[boollen..];
    if (namlen + boollen) % 2 == 1 && !p.is_empty() {
        p = &p[1..];
    }

    if p.len() < numlen * numsize {
        return None;
    }
    let mut nums = [-1i32; NNUM];
    for (i, num) in nums.iter_mut().enumerate().take(numlen) {
        *num = if numsize == 2 {
            short16(p, i * 2)
        } else {
            int32(p, i * 4)
        };
    }
    p = &p[numlen * numsize..];

    if p.len() < strslen * 2 {
        return None;
    }
    let str_offs: Vec<i32> = (0..strslen.min(NSTR)).map(|i| short16(p, i * 2)).collect();
    p = &p[strslen * 2..];

    if p.len() < tablsz {
        return None;
    }
    let mut table = p[..tablsz].to_vec();
    if tablsz > 0 {
        table[tablsz - 1] = 0;
    }
    let mut strs: Vec<Option<CString>> = Vec::with_capacity(NSTR);
    for i in 0..NSTR {
        strs.push(match str_offs.get(i) {
            Some(&off) if off >= 0 && (off as usize) < tablsz => {
                Some(cstr_at(&table, off as usize))
            }
            _ => None,
        });
    }
    p = &p[tablsz..];
    if tablsz % 2 == 1 && !p.is_empty() {
        p = &p[1..];
    }

    let mut term = Term {
        bools,
        nums,
        strs,
        ext_bools: Vec::new(),
        ext_bool_names: Vec::new(),
        ext_strs: Vec::new(),
        ext_str_names: Vec::new(),
    };

    // The extended-capability section. Present iff at least a header remains
    // and every header field fits in 15 bits — otherwise it is silently
    // ignored, not an error. Errors *inside* the section reject the whole
    // description.
    if p.len() < 10 {
        return Some(term);
    }
    let extbool = ru16(p, 0);
    let extnum = ru16(p, 2);
    let extstr = ru16(p, 4);
    let extoff = ru16(p, 6);
    let exttab = ru16(p, 8);
    if [extbool, extnum, extstr, extoff, exttab]
        .iter()
        .any(|&n| n > 0x7fff)
    {
        return Some(term);
    }
    p = &p[10..];

    let extall = extbool + extnum + extstr;
    if p.len() < extbool + extbool % 2 + extnum * numsize + extstr * 2 + extall * 2 + exttab {
        return None;
    }

    term.ext_bools = p[..extbool].iter().map(|&b| b != 0).collect();
    p = &p[extbool + extbool % 2..];

    // Extended numerics: nvim never reads them, so only their space matters.
    p = &p[extnum * numsize..];

    // The string table holds the value strings first, then the names of all
    // extended capabilities. Value offsets are relative to the table start;
    // out-of-range ones are NULL values. The C computed each value's length
    // in the *raw* table (last byte not yet forced to NUL) and rejected any
    // layout where the values don't tile the table's front exactly.
    let raw_tbl = &p[extstr * 2 + extall * 2..][..exttab];
    let mut s_sum = 0usize;
    let mut s_max = 0usize;
    let mut val_offs: Vec<Option<usize>> = Vec::with_capacity(extstr);
    for i in 0..extstr {
        let v = short16(p, i * 2);
        if v < 0 || v as usize >= exttab {
            val_offs.push(None);
            continue;
        }
        let v = v as usize;
        let end = match raw_tbl[v..].iter().position(|&b| b == 0) {
            Some(k) => v + k + 1,
            None => exttab,
        };
        s_sum += end - v;
        s_max = s_max.max(end);
        val_offs.push(Some(v));
    }
    p = &p[extstr * 2..];
    if s_max != s_sum {
        return None;
    }

    // Name offsets are relative to the tail of the table, after the values.
    let names_sz = exttab - s_sum;
    let mut name_offs: Vec<usize> = Vec::with_capacity(extall);
    for i in 0..extall {
        let v = short16(p, i * 2);
        if v < 0 || v as usize >= names_sz {
            return None;
        }
        name_offs.push(v as usize);
    }
    p = &p[extall * 2..];

    let mut ext_table = p[..exttab].to_vec();
    if exttab > 0 {
        ext_table[exttab - 1] = 0;
    }
    term.ext_strs = val_offs
        .iter()
        .map(|off| off.map(|v| cstr_at(&ext_table, v)))
        .collect();
    let names: Vec<CString> = name_offs
        .iter()
        .map(|&v| cstr_at(&ext_table, s_sum + v))
        .collect();
    term.ext_bool_names = names[..extbool].to_vec();
    term.ext_str_names = names[extbool + extnum..].to_vec();

    Some(term)
}

/// How one lookup attempt ended, standing in for the C's errno protocol:
/// `NotFound`/`Denied` (ENOENT/EPERM/EACCES) move the search along, while
/// `Abort` (any other error, including a file that exists but fails to
/// parse) ends the whole search empty-handed.
enum Lookup {
    Found(Box<Term>),
    NotFound,
    Denied,
    Abort,
}

/// `unibi_from_file` + `unibi_from_fd`: read at most `MAX_BUF` bytes, then
/// parse whatever was read.
fn try_file(path: &Path) -> Lookup {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return match e.kind() {
                std::io::ErrorKind::NotFound => Lookup::NotFound,
                std::io::ErrorKind::PermissionDenied => Lookup::Denied,
                _ => Lookup::Abort,
            }
        }
    };
    let mut buf = [0u8; MAX_BUF];
    let mut n = 0;
    while n < MAX_BUF {
        match file.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(r) => n += r,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Lookup::Abort,
        }
    }
    match from_mem(&buf[..n]) {
        Some(t) => Lookup::Found(Box::new(t)),
        None => Lookup::Abort,
    }
}

/// `from_dir`: try `dir[/mid]/<first char>/<term>`, then the macOS-style
/// `dir[/mid]/<first byte in hex>/<term>` if the first attempt's file did
/// not exist. Paths are assembled byte-wise like the C's sprintf, so an
/// empty `dir` yields an absolute path.
fn from_dir(dir: &OsStr, mid: Option<&str>, term: &CStr) -> Lookup {
    let term = term.to_bytes();
    let mut base = dir.as_bytes().to_vec();
    base.push(b'/');
    if let Some(mid) = mid {
        base.extend_from_slice(mid.as_bytes());
        base.push(b'/');
    }

    let mut letter = base.clone();
    letter.push(term[0]);
    letter.push(b'/');
    letter.extend_from_slice(term);
    match try_file(Path::new(&OsString::from_vec(letter))) {
        Lookup::NotFound => {}
        hit => return hit,
    }

    let mut hex = base;
    hex.extend_from_slice(format!("{:02x}", term[0]).as_bytes());
    hex.push(b'/');
    hex.extend_from_slice(term);
    try_file(Path::new(&OsString::from_vec(hex)))
}

/// `from_dirs`: a colon-separated directory list; empty entries are skipped.
fn from_dirs(list: &[u8], term: &CStr) -> Option<Term> {
    for dir in list.split(|&b| b == b':') {
        if dir.is_empty() {
            continue;
        }
        match from_dir(OsStr::from_bytes(dir), None, term) {
            Lookup::Found(t) => return Some(*t),
            Lookup::NotFound | Lookup::Denied => {}
            Lookup::Abort => return None,
        }
    }
    None
}

/// `unibi_from_term`: locate and parse the description for a terminal name.
pub fn from_term(term: &CStr) -> Option<Term> {
    let bytes = term.to_bytes();
    if bytes.is_empty() || bytes[0] == b'.' || bytes.contains(&b'/') {
        return None;
    }

    // $TERMINFO is a single directory. Uniquely, any failure — even one that
    // would abort the search below — falls through to the next stage.
    if let Some(dir) = std::env::var_os("TERMINFO") {
        if let Lookup::Found(t) = from_dir(&dir, None, term) {
            return Some(*t);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        match from_dir(&home, Some(".terminfo"), term) {
            Lookup::Found(t) => return Some(*t),
            Lookup::NotFound | Lookup::Denied => {}
            Lookup::Abort => return None,
        }
    }

    // $TERMINFO_DIRS *replaces* the compiled-in list; when it is set (even
    // to an empty string) the fallback directories are never consulted.
    if let Some(dirs) = std::env::var_os("TERMINFO_DIRS") {
        return from_dirs(dirs.as_bytes(), term);
    }
    from_dirs(TERMINFO_DIRS, term)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a compiled terminfo description from its sections.
    struct Entry {
        magic: u16,
        names: &'static [u8],
        bools: Vec<u8>,
        nums: Vec<u32>,
        str_offs: Vec<u16>,
        table: Vec<u8>,
        ext: Option<Ext>,
    }

    #[derive(Default)]
    struct Ext {
        bools: Vec<u8>,
        nums: Vec<u32>,
        val_offs: Vec<u16>,
        name_offs: Vec<u16>,
        table: Vec<u8>,
    }

    impl Entry {
        fn new() -> Self {
            Entry {
                magic: MAGIC_16BIT,
                names: b"test|Test terminal",
                bools: Vec::new(),
                nums: Vec::new(),
                str_offs: Vec::new(),
                table: Vec::new(),
                ext: None,
            }
        }

        fn build(&self) -> Vec<u8> {
            let numsize = if self.magic == MAGIC_16BIT { 2 } else { 4 };
            let mut out = Vec::new();
            for v in [
                self.magic as usize,
                self.names.len() + 1,
                self.bools.len(),
                self.nums.len(),
                self.str_offs.len(),
                self.table.len(),
            ] {
                out.extend_from_slice(&(v as u16).to_le_bytes());
            }
            out.extend_from_slice(self.names);
            out.push(0);
            out.extend_from_slice(&self.bools);
            if out.len() % 2 == 1 {
                out.push(0);
            }
            for &n in &self.nums {
                out.extend_from_slice(&n.to_le_bytes()[..numsize]);
            }
            for &o in &self.str_offs {
                out.extend_from_slice(&o.to_le_bytes());
            }
            out.extend_from_slice(&self.table);
            if let Some(ext) = &self.ext {
                if out.len() % 2 == 1 {
                    out.push(0);
                }
                for v in [
                    ext.bools.len(),
                    ext.nums.len(),
                    ext.val_offs.len(),
                    ext.val_offs.len() + ext.bools.len() + ext.nums.len(),
                    ext.table.len(),
                ] {
                    out.extend_from_slice(&(v as u16).to_le_bytes());
                }
                out.extend_from_slice(&ext.bools);
                if ext.bools.len() % 2 == 1 {
                    out.push(0);
                }
                for &n in &ext.nums {
                    out.extend_from_slice(&n.to_le_bytes()[..numsize]);
                }
                for &o in &ext.val_offs {
                    out.extend_from_slice(&o.to_le_bytes());
                }
                for &o in &ext.name_offs {
                    out.extend_from_slice(&o.to_le_bytes());
                }
                out.extend_from_slice(&ext.table);
            }
            out
        }
    }

    // Caps used below, by their unibilium enum values.
    const BACK_COLOR_ERASE: u32 = 29; // boolean index 28
    const COLUMNS: u32 = 46; // numeric index 0
    const LINES: u32 = 48; // numeric index 2
    const CARRIAGE_RETURN: u32 = 88; // string index 2

    #[test]
    fn parses_the_standard_sections() {
        let mut e = Entry::new();
        e.bools = vec![0; 29];
        e.bools[28] = 1;
        e.nums = vec![80, 0xffff, 24];
        e.str_offs = vec![0xffff, 0xfffe, 0];
        e.table = b"\r\0".to_vec();

        let t = from_mem(&e.build()).unwrap();
        assert!(t.get_bool(BACK_COLOR_ERASE));
        assert!(!t.get_bool(1));
        assert_eq!(t.get_num(COLUMNS), 80);
        assert_eq!(t.get_num(LINES), 24);
        // Absent (0xffff in the file, or past the stored count) reads as -1.
        assert_eq!(t.get_num(47), -1);
        assert_eq!(t.get_num(84), -1);
        assert_eq!(t.get_str(CARRIAGE_RETURN).unwrap().to_bytes(), b"\r");
        assert_eq!(t.get_str(86), None); // 0xffff: absent
        assert_eq!(t.get_str(87), None); // 0xfffe: cancelled
        assert_eq!(t.get_str(499), None);
    }

    #[test]
    fn parses_the_32bit_number_format() {
        let mut e = Entry::new();
        e.magic = MAGIC_32BIT;
        e.nums = vec![80, 0x12345, 0xffff_ffff];
        let t = from_mem(&e.build()).unwrap();
        assert_eq!(t.get_num(COLUMNS), 80);
        assert_eq!(t.get_num(47), 0x12345);
        assert_eq!(t.get_num(LINES), -1);
    }

    #[test]
    fn parses_the_extended_section() {
        let mut e = Entry::new();
        // Values first ("\x1b[s"), then a name per capability, booleans
        // before strings (Tc, Ns, RGB, Se).
        let mut ext = Ext::default();
        ext.bools = vec![1, 0];
        ext.val_offs = vec![0, 0xffff];
        ext.name_offs = vec![0, 3, 6, 10];
        ext.table = b"\x1b[s\0Tc\0Ns\0RGB\0Se\0".to_vec();
        e.ext = Some(ext);

        let t = from_mem(&e.build()).unwrap();
        let set: Vec<_> = t.ext_bool_names().map(|n| n.to_bytes().to_vec()).collect();
        assert_eq!(set, vec![b"Tc".to_vec()]);
        let strs: Vec<_> = t
            .ext_strs()
            .map(|(n, v)| (n.to_bytes().to_vec(), v.map(|v| v.to_bytes().to_vec())))
            .collect();
        assert_eq!(
            strs,
            vec![
                (b"RGB".to_vec(), Some(b"\x1b[s".to_vec())),
                (b"Se".to_vec(), None),
            ]
        );
    }

    #[test]
    fn a_malformed_extended_section_rejects_the_description() {
        // Two value offsets pointing at the same string overlap, which the
        // tiling check rejects.
        let mut e = Entry::new();
        let mut ext = Ext::default();
        ext.val_offs = vec![0, 0];
        ext.name_offs = vec![4, 6];
        ext.table = b"\x1bX\0\0a\0b\0".to_vec();
        e.ext = Some(ext);
        assert!(from_mem(&e.build()).is_none());

        // A name offset out of range is likewise fatal.
        let mut e = Entry::new();
        let mut ext = Ext::default();
        ext.name_offs = vec![9];
        ext.table = b"a\0".to_vec();
        ext.bools = vec![1];
        e.ext = Some(ext);
        assert!(from_mem(&e.build()).is_none());
    }

    #[test]
    fn an_oversized_extended_header_is_ignored_not_fatal() {
        let e = Entry::new();
        let mut data = e.build();
        if data.len() % 2 == 1 {
            data.push(0);
        }
        // 5-field ext header with a field above 0x7fff: section ignored.
        data.extend_from_slice(&[0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0]);
        let t = from_mem(&data).unwrap();
        assert_eq!(t.ext_bool_names().count(), 0);
    }

    #[test]
    fn truncated_input_is_rejected() {
        let mut e = Entry::new();
        e.bools = vec![1; 10];
        e.nums = vec![80];
        e.str_offs = vec![0];
        e.table = b"x\0".to_vec();
        let data = e.build();
        assert!(from_mem(&data).is_some());
        for len in 0..data.len() - 1 {
            // Every prefix must fail: the trailing table byte is load-bearing.
            assert!(from_mem(&data[..len]).is_none(), "prefix of {len} bytes");
        }
        assert!(from_mem(b"").is_none());
        assert!(from_mem(&[0u8; 12]).is_none()); // bad magic
    }

    #[test]
    fn lookup_walks_the_letter_then_hex_directories() {
        let tmp = std::env::temp_dir().join(format!("unibi-test-{}", std::process::id()));
        let mut e = Entry::new();
        e.nums = vec![80];
        let data = e.build();

        std::fs::create_dir_all(tmp.join("db/f")).unwrap();
        std::fs::write(tmp.join("db/f/foo"), &data).unwrap();
        std::fs::create_dir_all(tmp.join("db/62")).unwrap();
        std::fs::write(tmp.join("db/62/bar"), &data).unwrap();

        let list = tmp.join("db");
        let list = list.as_os_str().as_bytes();
        let foo = CStr::from_bytes_with_nul(b"foo\0").unwrap();
        let bar = CStr::from_bytes_with_nul(b"bar\0").unwrap();
        let baz = CStr::from_bytes_with_nul(b"baz\0").unwrap();
        assert!(from_dirs(list, foo).is_some());
        assert!(from_dirs(list, bar).is_some(), "hex fallback directory");
        assert!(from_dirs(list, baz).is_none());

        // A corrupt entry aborts the search: later directories that do have
        // the terminal are never consulted.
        std::fs::create_dir_all(tmp.join("bad/f")).unwrap();
        std::fs::write(tmp.join("bad/f/foo"), b"garbage").unwrap();
        let mut list2 = tmp.join("bad").as_os_str().as_bytes().to_vec();
        list2.push(b':');
        list2.extend_from_slice(list);
        assert!(from_dirs(&list2, foo).is_none());

        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn from_term_rejects_unusable_names() {
        for name in [&b""[..], b".hidden", b"a/b"] {
            let name = CString::new(name).unwrap();
            assert!(from_term(&name).is_none());
        }
    }
}
