//! Spell file reading and writing utilities.
//!
//! This module provides helper functions for reading and writing .spl spell files.
//! The main spell file loading/saving is coordinated from C, but these functions
//! handle the binary parsing and serialization of individual sections.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::range_plus_one)]
#![allow(clippy::option_if_let_else)]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

// =============================================================================
// File Format Constants
// =============================================================================

/// Magic string at start of Vim spell file
pub const VIMSPELLMAGIC: &[u8; 8] = b"VIMspell";

/// Current spell file version
pub const VIMSPELLVERSION: u8 = 50;

// Section IDs (matches spellfile.c enum)
/// Region section ID
pub const SN_REGION: u8 = 0;
/// Character flags section ID
pub const SN_CHARFLAGS: u8 = 1;
/// Midword characters section ID
pub const SN_MIDWORD: u8 = 2;
/// Prefix conditions section ID
pub const SN_PREFCOND: u8 = 3;
/// REP items section ID
pub const SN_REP: u8 = 4;
/// SAL (soundalike) items section ID
pub const SN_SAL: u8 = 5;
/// Soundfolding section ID
pub const SN_SOFO: u8 = 6;
/// MAP items section ID
pub const SN_MAP: u8 = 7;
/// Compound words section ID
pub const SN_COMPOUND: u8 = 8;
/// Syllable section ID
pub const SN_SYLLABLE: u8 = 9;
/// NOBREAK section ID
pub const SN_NOBREAK: u8 = 10;
/// Suggestion file timestamp section ID
pub const SN_SUGFILE: u8 = 11;
/// REPSAL items section ID
pub const SN_REPSAL: u8 = 12;
/// Common words section ID
pub const SN_WORDS: u8 = 13;
/// Don't split word for suggestions section ID
pub const SN_NOSPLITSUGS: u8 = 14;
/// Info section ID
pub const SN_INFO: u8 = 15;
/// Don't compound for suggestions section ID
pub const SN_NOCOMPOUNDSUGS: u8 = 16;
/// End of sections marker
pub const SN_END: u8 = 255;

/// Maximum length of a region name in bytes.
pub const REGION_NAME_LEN: usize = 2;

/// Maximum number of regions.
pub const MAX_REGIONS: usize = 8;

/// Maximum length of region string (8 regions * 2 chars each).
pub const MAX_REGION_STR_LEN: usize = MAX_REGIONS * REGION_NAME_LEN;

/// Section is required for correct spell checking.
pub const SNF_REQUIRED: u8 = 1;

/// Error: truncated file.
pub const SP_TRUNCERROR: c_int = -1;
/// Error: format error.
pub const SP_FORMERROR: c_int = -2;
/// Error: other error.
pub const SP_OTHERERROR: c_int = -3;

// =============================================================================
// File Header Parsing
// =============================================================================

/// Spell file header information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpellFileHeader {
    /// Magic bytes (should be VIMSPELLMAGIC)
    pub magic: [u8; 8],
    /// File version number
    pub version: u8,
}

impl Default for SpellFileHeader {
    fn default() -> Self {
        Self {
            magic: *VIMSPELLMAGIC,
            version: VIMSPELLVERSION,
        }
    }
}

/// Result of validating a spell file header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderValidation {
    /// Header is valid
    Valid = 0,
    /// Magic bytes don't match
    BadMagic = 1,
    /// Version is older than supported
    OldVersion = 2,
    /// Version is newer than supported
    NewVersion = 3,
    /// Buffer too short
    TooShort = 4,
}

/// Parse a spell file header from buffer.
///
/// Returns the header and the number of bytes consumed (9 bytes total).
/// Returns None if the buffer is too short.
#[must_use]
pub fn parse_spellfile_header(buf: &[u8]) -> Option<(SpellFileHeader, usize)> {
    if buf.len() < 9 {
        return None;
    }

    let mut magic = [0u8; 8];
    magic.copy_from_slice(&buf[0..8]);

    Some((
        SpellFileHeader {
            magic,
            version: buf[8],
        },
        9,
    ))
}

/// Validate a spell file header.
#[must_use]
pub fn validate_spellfile_header(header: &SpellFileHeader) -> HeaderValidation {
    if header.magic != *VIMSPELLMAGIC {
        return HeaderValidation::BadMagic;
    }

    if header.version < VIMSPELLVERSION {
        return HeaderValidation::OldVersion;
    }

    if header.version > VIMSPELLVERSION {
        return HeaderValidation::NewVersion;
    }

    HeaderValidation::Valid
}

/// FFI wrapper for parsing spell file header.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_spellfile_header(
    buf: *const u8,
    buf_len: usize,
    header_out: *mut SpellFileHeader,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || header_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_spellfile_header(slice) {
        Some((header, consumed)) => {
            *header_out = header;
            *consumed_out = consumed;
            0
        }
        None => SP_TRUNCERROR,
    }
}

/// FFI wrapper for validating spell file header.
///
/// # Safety
/// `header` must point to a valid SpellFileHeader.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_spellfile_header(header: *const SpellFileHeader) -> c_int {
    if header.is_null() {
        return HeaderValidation::TooShort as c_int;
    }

    validate_spellfile_header(&*header) as c_int
}

/// Write spell file header to buffer.
///
/// Returns the number of bytes written (9).
#[must_use]
pub fn write_spellfile_header(buf: &mut [u8], header: &SpellFileHeader) -> Option<usize> {
    if buf.len() < 9 {
        return None;
    }

    buf[0..8].copy_from_slice(&header.magic);
    buf[8] = header.version;
    Some(9)
}

/// FFI wrapper for writing spell file header.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_spellfile_header(
    buf: *mut u8,
    buf_len: usize,
    header: *const SpellFileHeader,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || header.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_spellfile_header(slice, &*header) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Section Header Parsing
// =============================================================================

/// Section header information.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SectionHeader {
    /// Section ID (0-254, 255 = end).
    pub section_id: u8,
    /// Section flags (SNF_REQUIRED).
    pub flags: u8,
    /// Section length in bytes.
    pub len: u32,
}

/// Parse a section header from buffer.
///
/// Returns the section header and the number of bytes consumed (6 bytes total).
/// Returns None if the buffer is too short or indicates end of sections.
///
/// Section format: <sectionID> <sectionflags> <sectionlen (4 bytes, BE)>
pub fn parse_section_header(buf: &[u8]) -> Option<(SectionHeader, usize)> {
    if buf.is_empty() {
        return None;
    }

    let section_id = buf[0];

    // Section end marker
    if section_id == 255 {
        return Some((
            SectionHeader {
                section_id: 255,
                flags: 0,
                len: 0,
            },
            1,
        ));
    }

    // Need at least 6 bytes: id (1) + flags (1) + len (4)
    if buf.len() < 6 {
        return None;
    }

    let flags = buf[1];
    let len = u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]);

    Some((
        SectionHeader {
            section_id,
            flags,
            len,
        },
        6,
    ))
}

/// FFI wrapper for parsing section header.
///
/// # Safety
/// `buf` must point to valid memory of at least `buf_len` bytes.
/// `header_out` must point to valid memory for a SectionHeader.
/// `consumed_out` must point to valid memory for a usize.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_section_header(
    buf: *const u8,
    buf_len: usize,
    header_out: *mut SectionHeader,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || header_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_section_header(slice) {
        Some((header, consumed)) => {
            *header_out = header;
            *consumed_out = consumed;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Region Section Parsing
// =============================================================================

/// Parse region section data.
///
/// Region format: <regionname (2 bytes)> ... (up to MAX_REGIONS)
/// Returns the number of regions parsed.
pub fn parse_region_section(buf: &[u8], output: &mut [u8]) -> Result<usize, c_int> {
    let len = buf.len();

    if len > MAX_REGION_STR_LEN {
        return Err(SP_FORMERROR);
    }

    if !len.is_multiple_of(REGION_NAME_LEN) {
        return Err(SP_FORMERROR);
    }

    // Check for NUL bytes in region data
    if buf.contains(&0) {
        return Err(SP_FORMERROR);
    }

    let out_len = output.len().min(len);
    output[..out_len].copy_from_slice(&buf[..out_len]);

    // NUL-terminate if there's room
    if out_len < output.len() {
        output[out_len] = 0;
    }

    Ok(len / REGION_NAME_LEN)
}

/// FFI wrapper for parsing region section.
///
/// # Safety
/// `buf` must point to valid memory of at least `buf_len` bytes.
/// `output` must point to valid memory of at least `output_len` bytes.
/// `region_count_out` must point to valid memory.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_region_section(
    buf: *const u8,
    buf_len: usize,
    output: *mut u8,
    output_len: usize,
    region_count_out: *mut usize,
) -> c_int {
    if buf.is_null() || output.is_null() || region_count_out.is_null() {
        return SP_OTHERERROR;
    }

    let in_slice = std::slice::from_raw_parts(buf, buf_len);
    let out_slice = std::slice::from_raw_parts_mut(output, output_len);

    match parse_region_section(in_slice, out_slice) {
        Ok(count) => {
            *region_count_out = count;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Character Flags Section Parsing
// =============================================================================

/// Character flags section data.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CharFlagsSection {
    /// Flags for characters 128-255 (CF_WORD, CF_UPPER).
    pub flags: [u8; 128],
    /// Number of valid flags.
    pub flags_len: usize,
    /// Folded characters buffer.
    pub folchars: [u8; 512],
    /// Length of folchars data.
    pub folchars_len: usize,
}

impl Default for CharFlagsSection {
    fn default() -> Self {
        Self {
            flags: [0; 128],
            flags_len: 0,
            folchars: [0; 512],
            folchars_len: 0,
        }
    }
}

/// Parse character flags section.
///
/// Format: <charflagslen (1 byte)> <charflags (N bytes)>
///         <folcharslen (2 bytes BE)> <folchars (N bytes)>
pub fn parse_charflags_section(buf: &[u8]) -> Result<(CharFlagsSection, usize), c_int> {
    let mut offset = 0;

    // Read charflagslen (1 byte)
    if buf.is_empty() {
        return Err(SP_TRUNCERROR);
    }
    let flags_len = buf[0] as usize;
    offset += 1;

    // Read charflags
    if offset + flags_len > buf.len() {
        return Err(SP_TRUNCERROR);
    }

    let mut section = CharFlagsSection::default();

    if flags_len > 0 {
        let copy_len = flags_len.min(128);
        section.flags[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        section.flags_len = copy_len;
    }
    offset += flags_len;

    // Read folcharslen (2 bytes BE)
    if offset + 2 > buf.len() {
        return Err(SP_TRUNCERROR);
    }
    let fol_len = u16::from_be_bytes([buf[offset], buf[offset + 1]]) as usize;
    offset += 2;

    // Read folchars
    if offset + fol_len > buf.len() {
        return Err(SP_TRUNCERROR);
    }

    if fol_len > 0 {
        let copy_len = fol_len.min(512);
        section.folchars[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        section.folchars_len = copy_len;
    }
    offset += fol_len;

    // Consistency check: if one is zero, both should be zero
    if (section.flags_len == 0) != (section.folchars_len == 0) {
        return Err(SP_FORMERROR);
    }

    Ok((section, offset))
}

/// FFI wrapper for parsing character flags section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_charflags_section(
    buf: *const u8,
    buf_len: usize,
    section_out: *mut CharFlagsSection,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || section_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_charflags_section(slice) {
        Ok((section, consumed)) => {
            *section_out = section;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Binary Reading Utilities
// =============================================================================

/// Read a big-endian u16 from buffer.
#[inline]
pub fn read_be_u16(buf: &[u8]) -> Option<u16> {
    if buf.len() < 2 {
        return None;
    }
    Some(u16::from_be_bytes([buf[0], buf[1]]))
}

/// Read a big-endian u24 from buffer.
#[inline]
pub fn read_be_u24(buf: &[u8]) -> Option<u32> {
    if buf.len() < 3 {
        return None;
    }
    Some(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32))
}

/// Read a big-endian u32 from buffer.
#[inline]
pub fn read_be_u32(buf: &[u8]) -> Option<u32> {
    if buf.len() < 4 {
        return None;
    }
    Some(u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]))
}

/// Read a big-endian u64 from buffer.
#[inline]
pub fn read_be_u64(buf: &[u8]) -> Option<u64> {
    if buf.len() < 8 {
        return None;
    }
    Some(u64::from_be_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    ]))
}

/// Read a length-prefixed string (1-byte length prefix).
///
/// Returns the string bytes and bytes consumed.
pub fn read_cnt_string_1(buf: &[u8]) -> Option<(&[u8], usize)> {
    if buf.is_empty() {
        return None;
    }
    let len = buf[0] as usize;
    if buf.len() < 1 + len {
        return None;
    }
    Some((&buf[1..1 + len], 1 + len))
}

/// Read a length-prefixed string (2-byte length prefix, BE).
///
/// Returns the string bytes and bytes consumed.
pub fn read_cnt_string_2(buf: &[u8]) -> Option<(&[u8], usize)> {
    let len = read_be_u16(buf)? as usize;
    if buf.len() < 2 + len {
        return None;
    }
    Some((&buf[2..2 + len], 2 + len))
}

// =============================================================================
// Replacement Section Parsing
// =============================================================================

/// A from-to replacement pair.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FromTo {
    /// "From" string (NUL-terminated).
    pub from: [u8; 256],
    /// Length of from string.
    pub from_len: u8,
    /// "To" string (NUL-terminated).
    pub to: [u8; 256],
    /// Length of to string.
    pub to_len: u8,
}

impl Default for FromTo {
    fn default() -> Self {
        Self {
            from: [0; 256],
            from_len: 0,
            to: [0; 256],
            to_len: 0,
        }
    }
}

/// Parse a single replacement item.
///
/// Format: <fromlen (1 byte)> <from (N bytes)> <tolen (1 byte)> <to (N bytes)>
pub fn parse_rep_item(buf: &[u8]) -> Result<(FromTo, usize), c_int> {
    let mut offset = 0;

    // Read from string
    if buf.is_empty() {
        return Err(SP_TRUNCERROR);
    }
    let from_len = buf[0] as usize;
    offset += 1;

    if offset + from_len >= buf.len() {
        return Err(SP_TRUNCERROR);
    }

    let mut item = FromTo::default();

    if from_len > 0 {
        let copy_len = from_len.min(255);
        item.from[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        item.from_len = copy_len as u8;
    }
    offset += from_len;

    // Read to string
    if offset >= buf.len() {
        return Err(SP_TRUNCERROR);
    }
    let to_len = buf[offset] as usize;
    offset += 1;

    if offset + to_len > buf.len() {
        return Err(SP_TRUNCERROR);
    }

    if to_len > 0 {
        let copy_len = to_len.min(255);
        item.to[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        item.to_len = copy_len as u8;
    }
    offset += to_len;

    Ok((item, offset))
}

/// FFI wrapper for parsing a replacement item.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_rep_item(
    buf: *const u8,
    buf_len: usize,
    item_out: *mut FromTo,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || item_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_rep_item(slice) {
        Ok((item, consumed)) => {
            *item_out = item;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Soundalike Section Parsing
// =============================================================================

/// Soundalike flags.
pub const SAL_F0LLOWUP: u8 = 1;
pub const SAL_COLLAPSE: u8 = 2;
pub const SAL_REM_ACCENTS: u8 = 4;

/// Soundalike section header.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SalHeader {
    /// Flags (SAL_F0LLOWUP, SAL_COLLAPSE, SAL_REM_ACCENTS).
    pub flags: u8,
    /// Number of SAL items.
    pub count: u16,
}

/// Parse soundalike section header.
///
/// Format: <salflags (1 byte)> <salcount (2 bytes BE)>
pub fn parse_sal_header(buf: &[u8]) -> Option<(SalHeader, usize)> {
    if buf.len() < 3 {
        return None;
    }

    Some((
        SalHeader {
            flags: buf[0],
            count: u16::from_be_bytes([buf[1], buf[2]]),
        },
        3,
    ))
}

/// FFI wrapper for parsing SAL header.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_sal_header(
    buf: *const u8,
    buf_len: usize,
    header_out: *mut SalHeader,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || header_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_sal_header(slice) {
        Some((header, consumed)) => {
            *header_out = header;
            *consumed_out = consumed;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Soundfold Section Parsing
// =============================================================================

/// Soundfold section data.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SofoSection {
    /// "From" characters.
    pub from: [u8; 512],
    /// Length of from string.
    pub from_len: u16,
    /// "To" characters.
    pub to: [u8; 512],
    /// Length of to string.
    pub to_len: u16,
}

impl Default for SofoSection {
    fn default() -> Self {
        Self {
            from: [0; 512],
            from_len: 0,
            to: [0; 512],
            to_len: 0,
        }
    }
}

/// Parse soundfold section.
///
/// Format: <sofofromlen (2 bytes BE)> <sofofrom (N bytes)>
///         <sofotolen (2 bytes BE)> <sofoto (N bytes)>
pub fn parse_sofo_section(buf: &[u8]) -> Result<(SofoSection, usize), c_int> {
    let mut offset = 0;

    // Read from string
    let from_len = read_be_u16(buf).ok_or(SP_TRUNCERROR)? as usize;
    offset += 2;

    if offset + from_len > buf.len() {
        return Err(SP_TRUNCERROR);
    }

    let mut section = SofoSection::default();

    if from_len > 0 {
        let copy_len = from_len.min(512);
        section.from[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        section.from_len = copy_len as u16;
    }
    offset += from_len;

    // Read to string
    if offset + 2 > buf.len() {
        return Err(SP_TRUNCERROR);
    }
    let to_len = read_be_u16(&buf[offset..]).ok_or(SP_TRUNCERROR)? as usize;
    offset += 2;

    if offset + to_len > buf.len() {
        return Err(SP_TRUNCERROR);
    }

    if to_len > 0 {
        let copy_len = to_len.min(512);
        section.to[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        section.to_len = copy_len as u16;
    }
    offset += to_len;

    Ok((section, offset))
}

/// FFI wrapper for parsing soundfold section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_sofo_section(
    buf: *const u8,
    buf_len: usize,
    section_out: *mut SofoSection,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || section_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_sofo_section(slice) {
        Ok((section, consumed)) => {
            *section_out = section;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Compound Section Parsing
// =============================================================================

/// Compound section header.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompoundHeader {
    /// Maximum number of words in compound.
    pub compmax: u8,
    /// Minimum word length for compounding.
    pub compminlen: u8,
    /// Maximum syllables in compound.
    pub compsylmax: u8,
    /// Compound options flags.
    pub compoptions: u16,
    /// Number of compound patterns.
    pub comppatcount: u16,
}

/// Parse compound section header.
///
/// Format: <compmax> <compminlen> <compsylmax> <compoptions (2 bytes BE)>
///         <comppatcount (2 bytes BE)>
pub fn parse_compound_header(buf: &[u8]) -> Option<(CompoundHeader, usize)> {
    if buf.len() < 7 {
        return None;
    }

    Some((
        CompoundHeader {
            compmax: buf[0],
            compminlen: buf[1],
            compsylmax: buf[2],
            compoptions: u16::from_be_bytes([buf[3], buf[4]]),
            comppatcount: u16::from_be_bytes([buf[5], buf[6]]),
        },
        7,
    ))
}

/// FFI wrapper for parsing compound header.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_compound_header(
    buf: *const u8,
    buf_len: usize,
    header_out: *mut CompoundHeader,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || header_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_compound_header(slice) {
        Some((header, consumed)) => {
            *header_out = header;
            *consumed_out = consumed;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Compound Flags Pattern Building
// =============================================================================

/// Maximum length for word in compound pattern buffer.
pub const MAXWLEN: usize = 254;

/// Result of parsing compound flags.
///
/// The C wrapper allocates `sl_compstartflags`, `sl_compallflags`, and
/// `sl_comprules` from these fields, then calls `vim_regcomp(pattern, ...)`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CompoundFlagsResult {
    /// Regex pattern string (null-terminated).
    /// Format: "^\(rule1\|rule2\)$"
    /// Max size: flags_len * 4 + 7 bytes (backslash doubling + utf8 + prefix/suffix)
    pub pattern: [u8; 4096],
    /// Length of the pattern (excluding null terminator).
    pub pattern_len: u32,
    /// Start flags (flags that can appear at start of compound word).
    pub startflags: [u8; MAXWLEN + 1],
    /// Length of startflags.
    pub startflags_len: u32,
    /// All flags (all non-special flags seen in compound rules).
    pub allflags: [u8; MAXWLEN + 1],
    /// Length of allflags.
    pub allflags_len: u32,
    /// Compound rules in original form (NULL if a wildcard was seen).
    pub comprules: [u8; MAXWLEN + 1],
    /// Length of comprules (0 if invalidated by wildcard).
    pub comprules_len: u32,
    /// Whether comprules is valid (false when wildcard was encountered).
    pub comprules_valid: bool,
}

impl Default for CompoundFlagsResult {
    fn default() -> Self {
        Self {
            pattern: [0; 4096],
            pattern_len: 0,
            startflags: [0; MAXWLEN + 1],
            startflags_len: 0,
            allflags: [0; MAXWLEN + 1],
            allflags_len: 0,
            comprules: [0; MAXWLEN + 1],
            comprules_len: 0,
            comprules_valid: true,
        }
    }
}

/// Encode a Unicode codepoint to UTF-8 bytes.
///
/// Returns the number of bytes written.
fn char2utf8(c: u32, buf: &mut [u8]) -> usize {
    if c < 0x80 {
        buf[0] = c as u8;
        1
    } else if c < 0x800 {
        buf[0] = (0xC0 | (c >> 6)) as u8;
        buf[1] = (0x80 | (c & 0x3F)) as u8;
        2
    } else if c < 0x1_0000 {
        buf[0] = (0xE0 | (c >> 12)) as u8;
        buf[1] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[2] = (0x80 | (c & 0x3F)) as u8;
        3
    } else {
        // Values > 0xFFFF shouldn't occur for byte-range compound flags,
        // but handle for completeness.
        buf[0] = (0xF0 | (c >> 18)) as u8;
        buf[1] = (0x80 | ((c >> 12) & 0x3F)) as u8;
        buf[2] = (0x80 | ((c >> 6) & 0x3F)) as u8;
        buf[3] = (0x80 | (c & 0x3F)) as u8;
        4
    }
}

/// Check if a byte value is already present in a NUL-terminated byte string.
fn byte_in_str(s: &[u8], len: usize, b: u8) -> bool {
    s[..len].contains(&b)
}

/// Parse compound flags and build the regex pattern and flag lists.
///
/// Takes the raw `<compflags>` bytes (after header and pattern strings have
/// been consumed). Builds:
/// - A Vim regex pattern: `"^\(rule1\|rule2\)$"` from `/`-separated rules
/// - `startflags`: flags that appear at the start of any compound rule item
/// - `allflags`: all non-special flags seen in the rules
/// - `comprules`: the original rules bytes (invalidated if wildcards are found)
///
/// Returns `Err(SP_FORMERROR)` if the pattern buffer overflows.
pub fn parse_compound_flags(flags_buf: &[u8]) -> Result<CompoundFlagsResult, c_int> {
    let mut result = CompoundFlagsResult::default();

    // Pattern starts with "^\("
    let prefix = b"^\\(";
    if result.pattern.len() < prefix.len() {
        return Err(SP_FORMERROR);
    }
    result.pattern[..prefix.len()].copy_from_slice(prefix);
    let mut pp = prefix.len();

    let mut atstart: i32 = 1; // 1 = at start of item, 2 = inside [...]
    let mut comprules_valid = true;
    let mut crp = 0usize; // index into comprules

    for &c in flags_buf {
        // Add non-special flags to allflags.
        if !matches!(c, b'?' | b'*' | b'+' | b'[' | b']' | b'/')
            && !byte_in_str(&result.allflags, result.allflags_len as usize, c)
        {
            let aidx = result.allflags_len as usize;
            if aidx < MAXWLEN {
                result.allflags[aidx] = c;
                result.allflags_len += 1;
            }
        }

        if atstart != 0 {
            // At start of item: copy flags to startflags.
            // For a [abc] item set atstart to 2 and copy up to the ']'.
            if c == b'[' {
                atstart = 2;
            } else if c == b']' {
                atstart = 0;
            } else {
                if !byte_in_str(&result.startflags, result.startflags_len as usize, c) {
                    let sidx = result.startflags_len as usize;
                    if sidx < MAXWLEN {
                        result.startflags[sidx] = c;
                        result.startflags_len += 1;
                    }
                }
                if atstart == 1 {
                    atstart = 0;
                }
            }
        }

        // Copy flag to comprules, unless we already hit a wildcard.
        if comprules_valid {
            if c == b'?' || c == b'+' || c == b'*' {
                comprules_valid = false;
                result.comprules_valid = false;
            } else if crp < MAXWLEN {
                result.comprules[crp] = c;
                crp += 1;
                result.comprules_len = crp as u32;
            }
        }

        // Append to pattern.
        if c == b'/' {
            // Slash separates two items: write "\|"
            if pp + 2 > result.pattern.len() - 4 {
                return Err(SP_FORMERROR);
            }
            result.pattern[pp] = b'\\';
            result.pattern[pp + 1] = b'|';
            pp += 2;
            atstart = 1;
        } else {
            // Normal char; "a?" becomes "a\?", "a+" becomes "a\+"
            let needs_backslash = matches!(c, b'?' | b'+' | b'~');
            let utf8_len = char2utf8(c as u32, &mut [0u8; 4]);
            let extra = usize::from(needs_backslash);
            if pp + extra + utf8_len > result.pattern.len() - 4 {
                return Err(SP_FORMERROR);
            }
            if needs_backslash {
                result.pattern[pp] = b'\\';
                pp += 1;
            }
            let written = char2utf8(c as u32, &mut result.pattern[pp..pp + utf8_len]);
            pp += written;
        }
    }

    // Append "\)$\0"
    if pp + 4 > result.pattern.len() {
        return Err(SP_FORMERROR);
    }
    result.pattern[pp] = b'\\';
    result.pattern[pp + 1] = b')';
    result.pattern[pp + 2] = b'$';
    result.pattern[pp + 3] = 0;
    result.pattern_len = (pp + 3) as u32;

    // Null-terminate flags arrays
    let sl = result.startflags_len as usize;
    result.startflags[sl] = 0;
    let al = result.allflags_len as usize;
    result.allflags[al] = 0;
    if comprules_valid {
        result.comprules[crp] = 0;
    }

    Ok(result)
}

/// FFI wrapper for parsing compound flags and building the regex pattern.
///
/// # Safety
/// All pointers must be valid. `buf` must point to `buf_len` bytes of compound
/// flags data (the raw `<compflags>` bytes only, NOT including the header).
#[no_mangle]
pub unsafe extern "C" fn rs_parse_compound_flags(
    buf: *const u8,
    buf_len: usize,
    result_out: *mut CompoundFlagsResult,
) -> c_int {
    if buf.is_null() || result_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_compound_flags(slice) {
        Ok(result) => {
            *result_out = result;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// SAL Item Parsing (buffer-based)
// =============================================================================

/// Maximum size of a SAL "from" string (lead + oneof + rules).
pub const SAL_FROM_MAX: usize = 512;
/// Maximum size of a SAL "to" string.
pub const SAL_TO_MAX: usize = 256;

/// A single parsed SAL (soundalike) item.
///
/// The C wrapper uses this to populate a `salitem_T` via xmalloc+copy.
/// `lead`, `oneof`, and `rules` are sub-slices of the `from` buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SalItem {
    /// The combined "from" buffer containing lead+NUL+oneof+NUL+rules+NUL.
    pub from: [u8; SAL_FROM_MAX],
    /// Total bytes used in `from` (including all NUL terminators).
    pub from_used: u16,
    /// Byte offset of lead within `from`.
    pub lead_offset: u16,
    /// Length of lead (bytes, not including NUL).
    pub lead_len: u16,
    /// Byte offset of oneof within `from` (0xFFFF if no oneof).
    pub oneof_offset: u16,
    /// Byte offset of rules within `from`.
    pub rules_offset: u16,
    /// The "to" string (null-terminated).
    pub to: [u8; SAL_TO_MAX],
    /// Length of to string (0 if none/empty).
    pub to_len: u16,
    /// Whether `to` is present (even if empty).
    pub has_to: bool,
}

impl Default for SalItem {
    fn default() -> Self {
        Self {
            from: [0; SAL_FROM_MAX],
            from_used: 0,
            lead_offset: 0,
            lead_len: 0,
            oneof_offset: 0xFFFF,
            rules_offset: 0,
            to: [0; SAL_TO_MAX],
            to_len: 0,
            has_to: false,
        }
    }
}

/// Parse one SAL item from a buffer.
///
/// The format is:
/// - 1 byte: from_len
/// - from_len bytes: from data (split by special chars into lead/oneof/rules)
/// - 1 byte: to_len
/// - to_len bytes: to data
///
/// Returns the parsed item and number of bytes consumed on success.
#[allow(clippy::too_many_lines)]
pub fn parse_sal_item(buf: &[u8]) -> Result<(SalItem, usize), c_int> {
    let mut offset = 0;
    let mut item = SalItem::default();

    if offset >= buf.len() {
        return Err(SP_TRUNCERROR);
    }
    let from_len = buf[offset] as usize;
    offset += 1;

    if offset + from_len > buf.len() {
        return Err(SP_TRUNCERROR);
    }

    let from_data = &buf[offset..offset + from_len];
    offset += from_len;

    // Parse from_data into lead / oneof / rules sections.
    // Special chars that mark end of lead: "0123456789(-<^$"
    let special_chars = b"0123456789(-<^$";
    let is_special = |c: u8| special_chars.contains(&c);

    let mut fp = 0usize; // index into item.from buffer

    // Lead: copy bytes until special char or end.
    let lead_start = fp;
    let mut i = 0;
    let mut trigger = 0u8; // the char that ended the lead
    while i < from_len {
        let c = from_data[i];
        if is_special(c) {
            trigger = c;
            break;
        }
        if fp >= SAL_FROM_MAX - 3 {
            break; // truncate gracefully
        }
        item.from[fp] = c;
        fp += 1;
        i += 1;
    }
    item.lead_offset = lead_start as u16;
    item.lead_len = (fp - lead_start) as u16;
    item.from[fp] = 0; // NUL terminate lead
    fp += 1;

    // Oneof: present if trigger == '('
    if trigger == b'(' {
        i += 1; // skip '('
        let oneof_start = fp;
        item.oneof_offset = oneof_start as u16;
        while i < from_len {
            let c = from_data[i];
            i += 1;
            if c == b')' {
                break;
            }
            if fp >= SAL_FROM_MAX - 2 {
                break;
            }
            item.from[fp] = c;
            fp += 1;
        }
        item.from[fp] = 0; // NUL terminate oneof
        fp += 1;
        // Next char after ')' is the first rules char (if any)
        if i < from_len {
            trigger = from_data[i];
            i += 1;
        } else {
            trigger = 0;
        }
    } else {
        item.oneof_offset = 0xFFFF; // no oneof
    }

    // Rules: remainder of from_data (starting from trigger char)
    let rules_start = fp;
    item.rules_offset = rules_start as u16;
    if trigger != 0 && i <= from_len {
        // Store the trigger char that ended the lead scan
        if fp < SAL_FROM_MAX - 1 {
            item.from[fp] = trigger;
            fp += 1;
        }
    }
    while i < from_len {
        let c = from_data[i];
        i += 1;
        if fp >= SAL_FROM_MAX - 1 {
            break;
        }
        item.from[fp] = c;
        fp += 1;
    }
    item.from[fp] = 0; // NUL terminate rules
    fp += 1;
    item.from_used = fp as u16;

    // Parse "to" string: 1 byte length + data
    if offset >= buf.len() {
        return Err(SP_TRUNCERROR);
    }
    let to_len = buf[offset] as usize;
    offset += 1;

    if to_len > 0 {
        if offset + to_len > buf.len() {
            return Err(SP_TRUNCERROR);
        }
        let copy_len = to_len.min(SAL_TO_MAX - 1);
        item.to[..copy_len].copy_from_slice(&buf[offset..offset + copy_len]);
        item.to[copy_len] = 0;
        item.to_len = copy_len as u16;
        item.has_to = true;
        offset += to_len;
    } else {
        item.has_to = false;
    }

    Ok((item, offset))
}

/// FFI wrapper for parsing a single SAL item from a buffer.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_sal_item(
    buf: *const u8,
    buf_len: usize,
    item_out: *mut SalItem,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || item_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_sal_item(slice) {
        Ok((item, consumed)) => {
            *item_out = item;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Suggestion File Section Parsing
// =============================================================================

/// Parse suggestion file timestamp.
///
/// Format: <timestamp (8 bytes BE)>
pub fn parse_sugfile_timestamp(buf: &[u8]) -> Option<(u64, usize)> {
    let ts = read_be_u64(buf)?;
    Some((ts, 8))
}

/// FFI wrapper for parsing sugfile timestamp.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_sugfile_timestamp(
    buf: *const u8,
    buf_len: usize,
    timestamp_out: *mut u64,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || timestamp_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_sugfile_timestamp(slice) {
        Some((ts, consumed)) => {
            *timestamp_out = ts;
            *consumed_out = consumed;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Word Tree Header Parsing
// =============================================================================

/// Parse word tree node count.
///
/// Format: <nodecount (4 bytes BE)>
pub fn parse_tree_nodecount(buf: &[u8]) -> Option<(u32, usize)> {
    let count = read_be_u32(buf)?;
    Some((count, 4))
}

/// FFI wrapper for parsing tree node count.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_tree_nodecount(
    buf: *const u8,
    buf_len: usize,
    count_out: *mut u32,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || count_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_tree_nodecount(slice) {
        Some((count, consumed)) => {
            *count_out = count;
            *consumed_out = consumed;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Binary Writing Utilities
// =============================================================================

/// Write a big-endian u16 to buffer.
///
/// Returns the number of bytes written (2).
#[inline]
pub fn write_be_u16(buf: &mut [u8], val: u16) -> Option<usize> {
    if buf.len() < 2 {
        return None;
    }
    let bytes = val.to_be_bytes();
    buf[0] = bytes[0];
    buf[1] = bytes[1];
    Some(2)
}

/// Write a big-endian u24 to buffer.
///
/// Returns the number of bytes written (3).
#[inline]
pub fn write_be_u24(buf: &mut [u8], val: u32) -> Option<usize> {
    if buf.len() < 3 {
        return None;
    }
    buf[0] = ((val >> 16) & 0xFF) as u8;
    buf[1] = ((val >> 8) & 0xFF) as u8;
    buf[2] = (val & 0xFF) as u8;
    Some(3)
}

/// Write a big-endian u32 to buffer.
///
/// Returns the number of bytes written (4).
#[inline]
pub fn write_be_u32(buf: &mut [u8], val: u32) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    let bytes = val.to_be_bytes();
    buf[..4].copy_from_slice(&bytes);
    Some(4)
}

/// Write a big-endian u64 to buffer.
///
/// Returns the number of bytes written (8).
#[inline]
pub fn write_be_u64(buf: &mut [u8], val: u64) -> Option<usize> {
    if buf.len() < 8 {
        return None;
    }
    let bytes = val.to_be_bytes();
    buf[..8].copy_from_slice(&bytes);
    Some(8)
}

/// Write a length-prefixed string (1-byte length prefix).
///
/// Returns the number of bytes written.
pub fn write_cnt_string_1(buf: &mut [u8], s: &[u8]) -> Option<usize> {
    if s.len() > 255 {
        return None;
    }
    let total = 1 + s.len();
    if buf.len() < total {
        return None;
    }
    buf[0] = s.len() as u8;
    buf[1..total].copy_from_slice(s);
    Some(total)
}

/// Write a length-prefixed string (2-byte length prefix, BE).
///
/// Returns the number of bytes written.
pub fn write_cnt_string_2(buf: &mut [u8], s: &[u8]) -> Option<usize> {
    if s.len() > 65535 {
        return None;
    }
    let total = 2 + s.len();
    if buf.len() < total {
        return None;
    }
    let len_bytes = (s.len() as u16).to_be_bytes();
    buf[0] = len_bytes[0];
    buf[1] = len_bytes[1];
    buf[2..total].copy_from_slice(s);
    Some(total)
}

// =============================================================================
// Section Header Writing
// =============================================================================

/// Write a section header to buffer.
///
/// Returns the number of bytes written (6 for normal section, 1 for end marker).
pub fn write_section_header(buf: &mut [u8], header: &SectionHeader) -> Option<usize> {
    if header.section_id == 255 {
        // Section end marker
        if buf.is_empty() {
            return None;
        }
        buf[0] = 255;
        return Some(1);
    }

    // Normal section: id (1) + flags (1) + len (4) = 6 bytes
    if buf.len() < 6 {
        return None;
    }

    buf[0] = header.section_id;
    buf[1] = header.flags;
    let len_bytes = header.len.to_be_bytes();
    buf[2..6].copy_from_slice(&len_bytes);
    Some(6)
}

/// FFI wrapper for writing section header.
///
/// # Safety
/// `buf` must point to valid writable memory of at least `buf_len` bytes.
/// `header` must point to a valid SectionHeader.
/// `written_out` must point to valid memory for a usize.
#[no_mangle]
pub unsafe extern "C" fn rs_write_section_header(
    buf: *mut u8,
    buf_len: usize,
    header: *const SectionHeader,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || header.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_section_header(slice, &*header) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Replacement Item Writing
// =============================================================================

/// Write a replacement item to buffer.
///
/// Format: <fromlen (1 byte)> <from (N bytes)> <tolen (1 byte)> <to (N bytes)>
pub fn write_rep_item(buf: &mut [u8], item: &FromTo) -> Option<usize> {
    let from_len = item.from_len as usize;
    let to_len = item.to_len as usize;
    let total = 1 + from_len + 1 + to_len;

    if buf.len() < total {
        return None;
    }

    let mut offset = 0;

    // Write from
    buf[offset] = item.from_len;
    offset += 1;
    buf[offset..offset + from_len].copy_from_slice(&item.from[..from_len]);
    offset += from_len;

    // Write to
    buf[offset] = item.to_len;
    offset += 1;
    buf[offset..offset + to_len].copy_from_slice(&item.to[..to_len]);
    offset += to_len;

    Some(offset)
}

/// FFI wrapper for writing a replacement item.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_rep_item(
    buf: *mut u8,
    buf_len: usize,
    item: *const FromTo,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || item.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_rep_item(slice, &*item) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Character Flags Section Writing
// =============================================================================

/// Write character flags section to buffer.
///
/// Format: <charflagslen (1 byte)> <charflags (N bytes)>
///         <folcharslen (2 bytes BE)> <folchars (N bytes)>
pub fn write_charflags_section(buf: &mut [u8], section: &CharFlagsSection) -> Option<usize> {
    let flags_len = section.flags_len;
    let fol_len = section.folchars_len;
    let total = 1 + flags_len + 2 + fol_len;

    if buf.len() < total {
        return None;
    }

    let mut offset = 0;

    // Write flags
    buf[offset] = flags_len as u8;
    offset += 1;
    buf[offset..offset + flags_len].copy_from_slice(&section.flags[..flags_len]);
    offset += flags_len;

    // Write folchars
    let fol_len_bytes = (fol_len as u16).to_be_bytes();
    buf[offset] = fol_len_bytes[0];
    buf[offset + 1] = fol_len_bytes[1];
    offset += 2;
    buf[offset..offset + fol_len].copy_from_slice(&section.folchars[..fol_len]);
    offset += fol_len;

    Some(offset)
}

/// FFI wrapper for writing character flags section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_charflags_section(
    buf: *mut u8,
    buf_len: usize,
    section: *const CharFlagsSection,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || section.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_charflags_section(slice, &*section) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Soundfold Section Writing
// =============================================================================

/// Write soundfold section to buffer.
///
/// Format: <sofofromlen (2 bytes BE)> <sofofrom (N bytes)>
///         <sofotolen (2 bytes BE)> <sofoto (N bytes)>
pub fn write_sofo_section(buf: &mut [u8], section: &SofoSection) -> Option<usize> {
    let from_len = section.from_len as usize;
    let to_len = section.to_len as usize;
    let total = 2 + from_len + 2 + to_len;

    if buf.len() < total {
        return None;
    }

    let mut offset = 0;

    // Write from
    let from_len_bytes = section.from_len.to_be_bytes();
    buf[offset] = from_len_bytes[0];
    buf[offset + 1] = from_len_bytes[1];
    offset += 2;
    buf[offset..offset + from_len].copy_from_slice(&section.from[..from_len]);
    offset += from_len;

    // Write to
    let to_len_bytes = section.to_len.to_be_bytes();
    buf[offset] = to_len_bytes[0];
    buf[offset + 1] = to_len_bytes[1];
    offset += 2;
    buf[offset..offset + to_len].copy_from_slice(&section.to[..to_len]);
    offset += to_len;

    Some(offset)
}

/// FFI wrapper for writing soundfold section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_sofo_section(
    buf: *mut u8,
    buf_len: usize,
    section: *const SofoSection,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || section.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_sofo_section(slice, &*section) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Soundalike Header Writing
// =============================================================================

/// Write soundalike header to buffer.
///
/// Format: <salflags (1 byte)> <salcount (2 bytes BE)>
pub fn write_sal_header(buf: &mut [u8], header: &SalHeader) -> Option<usize> {
    if buf.len() < 3 {
        return None;
    }

    buf[0] = header.flags;
    let count_bytes = header.count.to_be_bytes();
    buf[1] = count_bytes[0];
    buf[2] = count_bytes[1];
    Some(3)
}

/// FFI wrapper for writing SAL header.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_sal_header(
    buf: *mut u8,
    buf_len: usize,
    header: *const SalHeader,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || header.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_sal_header(slice, &*header) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Compound Header Writing
// =============================================================================

/// Write compound header to buffer.
///
/// Format: <compmax> <compminlen> <compsylmax> <compoptions (2 bytes BE)>
///         <comppatcount (2 bytes BE)>
pub fn write_compound_header(buf: &mut [u8], header: &CompoundHeader) -> Option<usize> {
    if buf.len() < 7 {
        return None;
    }

    buf[0] = header.compmax;
    buf[1] = header.compminlen;
    buf[2] = header.compsylmax;
    let opts_bytes = header.compoptions.to_be_bytes();
    buf[3] = opts_bytes[0];
    buf[4] = opts_bytes[1];
    let count_bytes = header.comppatcount.to_be_bytes();
    buf[5] = count_bytes[0];
    buf[6] = count_bytes[1];
    Some(7)
}

/// FFI wrapper for writing compound header.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_compound_header(
    buf: *mut u8,
    buf_len: usize,
    header: *const CompoundHeader,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || header.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_compound_header(slice, &*header) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Tree Node Count Writing
// =============================================================================

/// Write tree node count to buffer.
///
/// Format: <nodecount (4 bytes BE)>
pub fn write_tree_nodecount(buf: &mut [u8], count: u32) -> Option<usize> {
    write_be_u32(buf, count)
}

/// FFI wrapper for writing tree node count.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_tree_nodecount(
    buf: *mut u8,
    buf_len: usize,
    count: u32,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_tree_nodecount(slice, count) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Timestamp Writing
// =============================================================================

/// Write timestamp to buffer.
///
/// Format: <timestamp (8 bytes BE)>
pub fn write_timestamp(buf: &mut [u8], timestamp: u64) -> Option<usize> {
    write_be_u64(buf, timestamp)
}

/// FFI wrapper for writing timestamp.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_timestamp(
    buf: *mut u8,
    buf_len: usize,
    timestamp: u64,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_timestamp(slice, timestamp) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Tree Node Writing (Phase 6)
// =============================================================================

/// Byte values used in spell file tree.
///
/// These are special byte values that appear at the start of a node's sibling data.
/// Values > BY_SPECIAL are regular character bytes.
pub mod tree_bytes {
    /// End of word without flags or region; for postponed prefix: no <pflags>
    pub const BY_NOFLAGS: u8 = 0;
    /// Child is shared, index follows
    pub const BY_INDEX: u8 = 1;
    /// End of word, <flags> byte follows; for postponed prefix: <pflags> follows
    pub const BY_FLAGS: u8 = 2;
    /// End of word, <flags> and <flags2> bytes follow; never used in prefix tree
    pub const BY_FLAGS2: u8 = 3;
    /// Highest special byte value - values > BY_SPECIAL are regular characters
    pub const BY_SPECIAL: u8 = BY_FLAGS2;
}

/// Tree node flags for spell file writing.
///
/// These are the flag values used when writing word nodes to spell files.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TreeNodeFlags {
    /// Word flags (WF_* values)
    pub word_flags: u8,
    /// Region bitmask
    pub region: u8,
    /// Affix ID (0 if none)
    pub affixid: u8,
    /// Prefix ID (0 if none)
    pub prefixid: u8,
}

impl TreeNodeFlags {
    /// Create new tree node flags.
    pub const fn new() -> Self {
        Self {
            word_flags: 0,
            region: 0,
            affixid: 0,
            prefixid: 0,
        }
    }

    /// Check if any flags are set.
    pub const fn has_flags(&self) -> bool {
        self.word_flags != 0 || self.region != 0 || self.affixid != 0 || self.prefixid != 0
    }

    /// Count the number of bytes needed to encode these flags.
    pub const fn encoded_len(&self) -> usize {
        if !self.has_flags() {
            return 1; // Just BY_NOFLAGS
        }
        // BY_FLAGS + flags byte (+ optional region, affixid, prefixid)
        let mut len = 2;
        if self.region != 0 {
            len += 1;
        }
        if self.affixid != 0 {
            len += 1;
        }
        if self.prefixid != 0 {
            len += 1;
        }
        len
    }
}

/// Write tree node end-of-word marker and flags to buffer.
///
/// Returns the number of bytes written, or None if buffer is too small.
pub fn write_tree_node_flags(buf: &mut [u8], flags: &TreeNodeFlags) -> Option<usize> {
    if !flags.has_flags() {
        // No flags - just write BY_NOFLAGS
        if buf.is_empty() {
            return None;
        }
        buf[0] = tree_bytes::BY_NOFLAGS;
        return Some(1);
    }

    // Calculate required length
    let required = flags.encoded_len();
    if buf.len() < required {
        return None;
    }

    let mut offset = 0;

    // Write BY_FLAGS marker
    buf[offset] = tree_bytes::BY_FLAGS;
    offset += 1;

    // Build flags byte
    // Bits 0-2: basic flags (rare, region present, affix present)
    // Bit 3: prefix ID present
    // Bits 4-7: more flags
    let mut flags_byte = flags.word_flags;
    if flags.region != 0 {
        flags_byte |= 0x02; // WF_REGION
    }
    if flags.affixid != 0 {
        flags_byte |= 0x04; // WF_AFX
    }
    if flags.prefixid != 0 {
        flags_byte |= 0x08; // WF_PFX
    }
    buf[offset] = flags_byte;
    offset += 1;

    // Write optional bytes
    if flags.region != 0 {
        buf[offset] = flags.region;
        offset += 1;
    }
    if flags.affixid != 0 {
        buf[offset] = flags.affixid;
        offset += 1;
    }
    if flags.prefixid != 0 {
        buf[offset] = flags.prefixid;
        offset += 1;
    }

    Some(offset)
}

/// FFI wrapper for writing tree node flags.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_tree_node_flags(
    buf: *mut u8,
    buf_len: usize,
    flags: *const TreeNodeFlags,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || flags.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_tree_node_flags(slice, &*flags) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

/// Write a tree node sibling byte to buffer.
///
/// For regular characters, writes the byte directly.
/// Note: Special bytes (0-3) are only special when a node's wn_byte is 0 (end of word).
/// For non-end-of-word nodes, the byte is written directly regardless of value.
///
/// Returns bytes written or None if buffer too small.
pub fn write_tree_sibling_byte(buf: &mut [u8], byte: u8) -> Option<usize> {
    if buf.is_empty() {
        return None;
    }
    buf[0] = byte;
    Some(1)
}

/// FFI wrapper for writing tree sibling byte.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_tree_sibling_byte(
    buf: *mut u8,
    buf_len: usize,
    byte: u8,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_tree_sibling_byte(slice, byte) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

/// Write a tree node child index to buffer.
///
/// The index is written as a variable-length integer:
/// - 0x00-0x7F: 1 byte
/// - 0x80-0x7FFF: 2 bytes (first byte | 0x80)
/// - 0x8000-0x7FFFFF: 3 bytes (first byte | 0xC0)
/// - 0x800000-0x7FFFFFFF: 4 bytes (first byte | 0xE0)
///
/// Returns bytes written or None if buffer too small or index too large.
pub fn write_tree_child_index(buf: &mut [u8], index: u32) -> Option<usize> {
    if index <= 0x7F {
        // 1 byte
        if buf.is_empty() {
            return None;
        }
        buf[0] = index as u8;
        Some(1)
    } else if index <= 0x7FFF {
        // 2 bytes
        if buf.len() < 2 {
            return None;
        }
        buf[0] = ((index >> 8) as u8) | 0x80;
        buf[1] = index as u8;
        Some(2)
    } else if index <= 0x7F_FFFF {
        // 3 bytes
        if buf.len() < 3 {
            return None;
        }
        buf[0] = ((index >> 16) as u8) | 0xC0;
        buf[1] = (index >> 8) as u8;
        buf[2] = index as u8;
        Some(3)
    } else if index <= 0x7FFF_FFFF {
        // 4 bytes
        if buf.len() < 4 {
            return None;
        }
        buf[0] = ((index >> 24) as u8) | 0xE0;
        buf[1] = (index >> 16) as u8;
        buf[2] = (index >> 8) as u8;
        buf[3] = index as u8;
        Some(4)
    } else {
        // Index too large
        None
    }
}

/// FFI wrapper for writing tree child index.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_tree_child_index(
    buf: *mut u8,
    buf_len: usize,
    index: u32,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_tree_child_index(slice, index) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

/// Information about a spell file being written.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SpellFileWriteInfo {
    /// Number of regions in the spell file.
    pub region_count: u8,
    /// Whether this is an addition file.
    pub is_addition: bool,
    /// Region mask (which regions are present).
    pub region_mask: u8,
    /// Flags for the file.
    pub file_flags: u8,
}

impl SpellFileWriteInfo {
    /// Create new write info with default values.
    pub const fn new() -> Self {
        Self {
            region_count: 0,
            is_addition: false,
            region_mask: 0,
            file_flags: 0,
        }
    }

    /// Check if regions should be written.
    pub const fn has_regions(&self) -> bool {
        self.region_count > 0
    }
}

/// FFI function to create a new SpellFileWriteInfo.
#[no_mangle]
pub extern "C" fn rs_spell_file_write_info_new() -> SpellFileWriteInfo {
    SpellFileWriteInfo::new()
}

/// Write region section to buffer.
///
/// Format: <section_id><flags><length><regions...>
/// Each region is 2 bytes (e.g., "en", "us").
pub fn write_region_section(buf: &mut [u8], regions: &[u8]) -> Option<usize> {
    // Validate regions (must be pairs of ASCII chars)
    if !regions.len().is_multiple_of(REGION_NAME_LEN) || regions.len() > MAX_REGION_STR_LEN {
        return None;
    }

    let section_len = regions.len();
    let header = SectionHeader {
        section_id: SN_REGION,
        flags: 0,
        len: section_len as u32,
    };

    let header_len = write_section_header(buf, &header)?;
    if buf.len() < header_len + section_len {
        return None;
    }

    buf[header_len..header_len + section_len].copy_from_slice(regions);
    Some(header_len + section_len)
}

/// FFI wrapper for writing region section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_region_section(
    buf: *mut u8,
    buf_len: usize,
    regions: *const u8,
    regions_len: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }
    if regions.is_null() && regions_len > 0 {
        return SP_OTHERERROR;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf, buf_len);
    let regions_slice = if regions.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(regions, regions_len)
    };

    match write_region_section(buf_slice, regions_slice) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

/// Write end section marker to buffer.
///
/// Format: <SN_END (0xFF)>
pub fn write_end_section(buf: &mut [u8]) -> Option<usize> {
    if buf.is_empty() {
        return None;
    }
    buf[0] = SN_END;
    Some(1)
}

/// FFI wrapper for writing end section marker.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_write_end_section(
    buf: *mut u8,
    buf_len: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_end_section(slice) {
        Some(written) => {
            *written_out = written;
            0
        }
        None => SP_TRUNCERROR,
    }
}

// =============================================================================
// Prefix Condition Section Parsing
// =============================================================================

/// Maximum number of prefix conditions
pub const MAX_PREFCOND: usize = 65535;

/// A single prefix condition entry.
///
/// In C, prefix conditions are compiled into regex programs. Here we store
/// the raw condition string which can be passed to C for regex compilation.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PrefixCondition {
    /// The condition pattern (NUL-terminated)
    pub pattern: [u8; 256],
    /// Length of the pattern
    pub pattern_len: u8,
}

impl Default for PrefixCondition {
    fn default() -> Self {
        Self {
            pattern: [0; 256],
            pattern_len: 0,
        }
    }
}

/// Parse a single prefix condition from buffer.
///
/// Format: <condlen (1 byte)> <condstr (N bytes)>
///
/// Returns (condition, bytes_consumed) or error.
pub fn parse_prefcond(buf: &[u8]) -> Result<(PrefixCondition, usize), c_int> {
    if buf.is_empty() {
        return Err(SP_TRUNCERROR);
    }

    let condlen = buf[0] as usize;
    if condlen >= 254 {
        // Condition too long (MAXWLEN limit)
        return Err(SP_FORMERROR);
    }

    if buf.len() < 1 + condlen {
        return Err(SP_TRUNCERROR);
    }

    let mut cond = PrefixCondition::default();

    if condlen > 0 {
        // Check for NUL bytes in condition (invalid)
        let cond_bytes = &buf[1..1 + condlen];
        if cond_bytes.contains(&0) {
            return Err(SP_FORMERROR);
        }

        let copy_len = condlen.min(255);
        cond.pattern[..copy_len].copy_from_slice(&buf[1..1 + copy_len]);
        cond.pattern_len = copy_len as u8;
    }

    Ok((cond, 1 + condlen))
}

/// FFI wrapper for parsing a prefix condition.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_prefcond(
    buf: *const u8,
    buf_len: usize,
    cond_out: *mut PrefixCondition,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || cond_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_prefcond(slice) {
        Ok((cond, consumed)) => {
            *cond_out = cond;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

/// Parse prefix conditions section header.
///
/// Format: <prefcondcnt (2 bytes BE)>
///
/// Returns the count of prefix conditions.
pub fn parse_prefcond_count(buf: &[u8]) -> Result<(u16, usize), c_int> {
    let count = read_be_u16(buf).ok_or(SP_TRUNCERROR)?;
    if count == 0 {
        return Err(SP_FORMERROR);
    }
    Ok((count, 2))
}

/// FFI wrapper for parsing prefix condition count.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_prefcond_count(
    buf: *const u8,
    buf_len: usize,
    count_out: *mut u16,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || count_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_prefcond_count(slice) {
        Ok((count, consumed)) => {
            *count_out = count;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// REP/REPSAL Section Parsing
// =============================================================================

/// Parse REP/REPSAL section header.
///
/// Format: <repcount (2 bytes BE)>
///
/// Returns the count of replacement items.
pub fn parse_rep_count(buf: &[u8]) -> Result<(u16, usize), c_int> {
    let count = read_be_u16(buf).ok_or(SP_TRUNCERROR)?;
    Ok((count, 2))
}

/// FFI wrapper for parsing REP count.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_rep_count(
    buf: *const u8,
    buf_len: usize,
    count_out: *mut u16,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || count_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_rep_count(slice) {
        Ok((count, consumed)) => {
            *count_out = count;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

/// Build the first-index table for REP items.
///
/// For each byte value 0-255, stores the index of the first REP item
/// that starts with that byte, or -1 if none.
///
/// # Arguments
/// * `items` - Array of FromTo items
/// * `count` - Number of items
/// * `first` - Output array of 256 i16 values
///
/// # Safety
/// All pointers must be valid. `items` must have at least `count` elements.
/// `first` must have at least 256 elements.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_build_rep_first_table(
    items: *const FromTo,
    count: usize,
    first: *mut i16,
) {
    if items.is_null() || first.is_null() {
        return;
    }

    // Initialize all to -1
    for i in 0..256 {
        *first.add(i) = -1;
    }

    // Fill in first indexes
    for i in 0..count {
        let item = &*items.add(i);
        if item.from_len > 0 {
            let byte_val = item.from[0] as usize;
            if *first.add(byte_val) == -1 {
                *first.add(byte_val) = i as i16;
            }
        }
    }
}

// =============================================================================
// Midword Section Parsing
// =============================================================================

/// Parse midword section.
///
/// The midword section is just a string of characters that can appear
/// in the middle of a word but not at the start or end.
///
/// Format: <midword string (len bytes)>
pub fn parse_midword_section(buf: &[u8], output: &mut [u8]) -> Result<usize, c_int> {
    // Check for NUL bytes (invalid)
    if buf.contains(&0) {
        return Err(SP_FORMERROR);
    }

    let copy_len = buf.len().min(output.len().saturating_sub(1));
    output[..copy_len].copy_from_slice(&buf[..copy_len]);
    // NUL-terminate
    if copy_len < output.len() {
        output[copy_len] = 0;
    }

    Ok(copy_len)
}

/// FFI wrapper for parsing midword section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_midword_section(
    buf: *const u8,
    buf_len: usize,
    output: *mut u8,
    output_len: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || output.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let in_slice = std::slice::from_raw_parts(buf, buf_len);
    let out_slice = std::slice::from_raw_parts_mut(output, output_len);

    match parse_midword_section(in_slice, out_slice) {
        Ok(written) => {
            *written_out = written;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// MAP Section Parsing
// =============================================================================

/// Parse MAP section string.
///
/// The MAP section contains groups of similar characters separated by '/'.
/// Each group starts with a character count prefix.
///
/// Format: <mapstr (len bytes)>
pub fn parse_map_section(buf: &[u8], output: &mut [u8]) -> Result<usize, c_int> {
    // Check for NUL bytes (invalid)
    if buf.contains(&0) {
        return Err(SP_FORMERROR);
    }

    let copy_len = buf.len().min(output.len().saturating_sub(1));
    output[..copy_len].copy_from_slice(&buf[..copy_len]);
    // NUL-terminate
    if copy_len < output.len() {
        output[copy_len] = 0;
    }

    Ok(copy_len)
}

/// FFI wrapper for parsing MAP section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_map_section(
    buf: *const u8,
    buf_len: usize,
    output: *mut u8,
    output_len: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || output.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let in_slice = std::slice::from_raw_parts(buf, buf_len);
    let out_slice = std::slice::from_raw_parts_mut(output, output_len);

    match parse_map_section(in_slice, out_slice) {
        Ok(written) => {
            *written_out = written;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Syllable Section Parsing
// =============================================================================

/// Parse syllable section.
///
/// The syllable section contains syllable definitions for the language.
/// It's a simple string that is passed to `init_syl_tab()` in C.
///
/// Format: <syllable string (len bytes)>
pub fn parse_syllable_section(buf: &[u8], output: &mut [u8]) -> Result<usize, c_int> {
    // Check for NUL bytes (invalid)
    if buf.contains(&0) {
        return Err(SP_FORMERROR);
    }

    let copy_len = buf.len().min(output.len().saturating_sub(1));
    output[..copy_len].copy_from_slice(&buf[..copy_len]);
    // NUL-terminate
    if copy_len < output.len() {
        output[copy_len] = 0;
    }

    Ok(copy_len)
}

/// FFI wrapper for parsing syllable section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_syllable_section(
    buf: *const u8,
    buf_len: usize,
    output: *mut u8,
    output_len: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || output.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let in_slice = std::slice::from_raw_parts(buf, buf_len);
    let out_slice = std::slice::from_raw_parts_mut(output, output_len);

    match parse_syllable_section(in_slice, out_slice) {
        Ok(written) => {
            *written_out = written;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Info Section Parsing
// =============================================================================

/// Parse info section.
///
/// The info section contains information about the spell file (author, etc.).
/// It's a simple string.
///
/// Format: <info string (len bytes)>
pub fn parse_info_section(buf: &[u8], output: &mut [u8]) -> Result<usize, c_int> {
    // Info section can contain NUL bytes as separators between fields
    let copy_len = buf.len().min(output.len().saturating_sub(1));
    output[..copy_len].copy_from_slice(&buf[..copy_len]);
    // NUL-terminate
    if copy_len < output.len() {
        output[copy_len] = 0;
    }

    Ok(copy_len)
}

/// FFI wrapper for parsing info section.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_info_section(
    buf: *const u8,
    buf_len: usize,
    output: *mut u8,
    output_len: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || output.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }

    let in_slice = std::slice::from_raw_parts(buf, buf_len);
    let out_slice = std::slice::from_raw_parts_mut(output, output_len);

    match parse_info_section(in_slice, out_slice) {
        Ok(written) => {
            *written_out = written;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// WORDS Section Parsing
// =============================================================================

/// Parse a single word from the WORDS section.
///
/// Words are NUL-terminated in the section.
///
/// Returns (word_slice, bytes_consumed) or error.
pub fn parse_words_entry(buf: &[u8]) -> Result<(&[u8], usize), c_int> {
    // Find NUL terminator
    let nul_pos = buf.iter().position(|&b| b == 0).ok_or(SP_TRUNCERROR)?;

    Ok((&buf[..nul_pos], nul_pos + 1))
}

/// FFI wrapper for parsing a word entry.
///
/// Returns the length of the word (not including NUL), or a negative error code.
/// Copies the word to `output` (NUL-terminated).
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_parse_words_entry(
    buf: *const u8,
    buf_len: usize,
    output: *mut u8,
    output_len: usize,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || output.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match parse_words_entry(slice) {
        Ok((word, consumed)) => {
            let copy_len = word.len().min(output_len.saturating_sub(1));
            std::ptr::copy_nonoverlapping(word.as_ptr(), output, copy_len);
            *output.add(copy_len) = 0; // NUL-terminate
            *consumed_out = consumed;
            copy_len as c_int
        }
        Err(e) => e,
    }
}

// =============================================================================
// Tree Reading Orchestration
// =============================================================================

/// Result of reading a word tree from a buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TreeReadResult {
    /// Number of bytes consumed from input buffer
    pub bytes_consumed: usize,
    /// Total number of nodes in the tree
    pub node_count: u32,
    /// Error code (0 = success)
    pub error: c_int,
}

/// Read tree node count from buffer.
///
/// This reads the 4-byte node count that precedes the tree data.
///
/// Returns (node_count, bytes_consumed) or error.
pub fn read_tree_node_count(buf: &[u8]) -> Result<(u32, usize), c_int> {
    let count = read_be_u32(buf).ok_or(SP_TRUNCERROR)?;
    Ok((count, 4))
}

/// FFI wrapper for reading tree node count.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_read_tree_node_count(
    buf: *const u8,
    buf_len: usize,
    count_out: *mut u32,
    consumed_out: *mut usize,
) -> c_int {
    if buf.is_null() || count_out.is_null() || consumed_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match read_tree_node_count(slice) {
        Ok((count, consumed)) => {
            *count_out = count;
            *consumed_out = consumed;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Tree Node Reading (Phase 329-330)
// =============================================================================

/// Word flags used in tree nodes.
pub mod word_flags {
    /// Word has region specified.
    pub const WF_REGION: i32 = 0x02;
    /// Word has affix ID.
    pub const WF_AFX: i32 = 0x04;
    /// Word has prefix ID.
    pub const WF_PFX: i32 = 0x08;
}

/// Mask to mark a shared tree node index.
const SHARED_MASK: i32 = 0x0800_0000;

/// Read a 2-byte big-endian integer from buffer.
#[inline]
fn get2c(buf: &[u8], offset: usize) -> Option<i32> {
    if offset + 2 > buf.len() {
        return None;
    }
    Some(((buf[offset] as i32) << 8) | (buf[offset + 1] as i32))
}

/// Read a 3-byte big-endian integer from buffer.
#[inline]
fn get3c(buf: &[u8], offset: usize) -> Option<i32> {
    if offset + 3 > buf.len() {
        return None;
    }
    Some(((buf[offset] as i32) << 16) | ((buf[offset + 1] as i32) << 8) | (buf[offset + 2] as i32))
}

/// State tracker for reading tree nodes from a buffer.
///
/// This replaces the FILE* based reading in C with buffer-based reading.
#[derive(Debug)]
pub struct TreeReader<'a> {
    /// Input buffer
    buf: &'a [u8],
    /// Current read position
    pos: usize,
}

impl<'a> TreeReader<'a> {
    /// Create a new tree reader.
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Get the current position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Get a byte at the current position and advance.
    fn getc(&mut self) -> Option<i32> {
        if self.pos >= self.buf.len() {
            return None;
        }
        let c = self.buf[self.pos] as i32;
        self.pos += 1;
        Some(c)
    }

    /// Read a 2-byte big-endian value and advance.
    fn get2c(&mut self) -> Option<i32> {
        let val = get2c(self.buf, self.pos)?;
        self.pos += 2;
        Some(val)
    }

    /// Read a 3-byte big-endian value and advance.
    fn get3c(&mut self) -> Option<i32> {
        let val = get3c(self.buf, self.pos)?;
        self.pos += 3;
        Some(val)
    }
}

/// Read one row of siblings from the spell file and store it in the byte array
/// "byts" and index array "idxs". Recursively read the children.
///
/// This is a Rust port of `read_tree_node()` from spellfile.c.
///
/// # Arguments
/// * `reader` - Buffer reader
/// * `byts` - Output byte array (node bytes)
/// * `idxs` - Output index array (node indexes)
/// * `maxidx` - Size of arrays
/// * `startidx` - Current index in "byts" and "idxs"
/// * `prefixtree` - true for reading PREFIXTREE
/// * `maxprefcondnr` - Maximum for <prefcondnr>
///
/// # Returns
/// The index (>= 0) following the siblings, or SP_* error code.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
pub fn read_tree_node(
    reader: &mut TreeReader,
    byts: &mut [u8],
    idxs: &mut [i32],
    maxidx: i32,
    startidx: i32,
    prefixtree: bool,
    maxprefcondnr: i32,
) -> i32 {
    let mut idx = startidx;

    // Read <siblingcount>
    let len = match reader.getc() {
        Some(c) if c > 0 => c,
        _ => return SP_TRUNCERROR,
    };

    if startidx + len >= maxidx {
        return SP_FORMERROR;
    }

    byts[idx as usize] = len as u8;
    idx += 1;

    // Read the byte values, flag/region bytes and shared indexes.
    for _i in 1..=len {
        let Some(c) = reader.getc() else {
            return SP_TRUNCERROR;
        };

        let byte_val;
        if c <= tree_bytes::BY_SPECIAL as i32 {
            if c == tree_bytes::BY_NOFLAGS as i32 && !prefixtree {
                // No flags, all regions.
                idxs[idx as usize] = 0;
                byte_val = 0;
            } else if c != tree_bytes::BY_INDEX as i32 {
                // BY_FLAGS, BY_FLAGS2, or BY_NOFLAGS in prefix tree
                let idx_val = if prefixtree {
                    // Read the optional pflags byte, the prefix ID and the
                    // condition nr. In idxs[] store the prefix ID in the low
                    // byte, the condition index shifted up 8 bits, the flags
                    // shifted up 24 bits.
                    let mut val = if c == tree_bytes::BY_FLAGS as i32 {
                        match reader.getc() {
                            Some(pflags) => pflags << 24,
                            None => return SP_TRUNCERROR,
                        }
                    } else {
                        0
                    };

                    // Read <affixID>
                    match reader.getc() {
                        Some(affixid) => val |= affixid,
                        None => return SP_TRUNCERROR,
                    }

                    // Read <prefcondnr>
                    match reader.get2c() {
                        Some(n) if n < maxprefcondnr => val |= n << 8,
                        Some(_) => return SP_FORMERROR,
                        None => return SP_TRUNCERROR,
                    }

                    val
                } else {
                    // c must be BY_FLAGS or BY_FLAGS2
                    // Read flags and optional region and prefix ID. In
                    // idxs[] the flags go in the low two bytes, region above
                    // that and prefix ID above the region.
                    let c2 = c;
                    let Some(mut val) = reader.getc() else {
                        return SP_TRUNCERROR;
                    };

                    if c2 == tree_bytes::BY_FLAGS2 as i32 {
                        match reader.getc() {
                            Some(flags2) => val += flags2 << 8,
                            None => return SP_TRUNCERROR,
                        }
                    }

                    if val & word_flags::WF_REGION != 0 {
                        match reader.getc() {
                            Some(region) => val += region << 16,
                            None => return SP_TRUNCERROR,
                        }
                    }

                    if val & word_flags::WF_AFX != 0 {
                        match reader.getc() {
                            Some(affixid) => val += affixid << 24,
                            None => return SP_TRUNCERROR,
                        }
                    }

                    val
                };

                idxs[idx as usize] = idx_val;
                byte_val = 0;
            } else {
                // c == BY_INDEX
                // Read <nodeidx>
                let n = match reader.get3c() {
                    Some(n) if n >= 0 && n < maxidx => n,
                    Some(_) => return SP_FORMERROR,
                    None => return SP_TRUNCERROR,
                };

                idxs[idx as usize] = n + SHARED_MASK;

                // Read <xbyte>
                byte_val = match reader.getc() {
                    Some(xbyte) => xbyte,
                    None => return SP_TRUNCERROR,
                };
            }
        } else {
            // Regular character byte
            byte_val = c;
        }

        byts[idx as usize] = byte_val as u8;
        idx += 1;
    }

    // Recursively read the children for non-shared siblings.
    // Skip the end-of-word ones (zero byte value) and the shared ones
    // (and remove SHARED_MASK).
    for i in 1..=len {
        let sibling_idx = (startidx + i) as usize;
        if byts[sibling_idx] != 0 {
            if idxs[sibling_idx] & SHARED_MASK != 0 {
                idxs[sibling_idx] &= !SHARED_MASK;
            } else {
                idxs[sibling_idx] = idx;
                idx = read_tree_node(reader, byts, idxs, maxidx, idx, prefixtree, maxprefcondnr);
                if idx < 0 {
                    break;
                }
            }
        }
    }

    idx
}

/// Read a complete word tree from a buffer.
///
/// This is a Rust port of `spell_read_tree()` from spellfile.c.
///
/// # Arguments
/// * `buf` - Input buffer containing tree data
/// * `byts` - Output byte array (will be filled with node bytes)
/// * `idxs` - Output index array (will be filled with node indexes)
/// * `prefixtree` - true for reading PREFIXTREE
/// * `prefixcnt` - When prefixtree is true: prefix count (max prefcondnr)
///
/// # Returns
/// * `Ok((bytes_consumed, node_count))` on success
/// * `Err(SP_* error code)` on failure
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
pub fn read_tree(
    buf: &[u8],
    byts: &mut [u8],
    idxs: &mut [i32],
    prefixtree: bool,
    prefixcnt: i32,
) -> Result<(usize, i32), c_int> {
    // Read node count (4 bytes BE)
    let (nodecount, header_consumed) = read_tree_node_count(buf)?;

    if nodecount == 0 {
        return Ok((header_consumed, 0));
    }

    // Validate node count - check for overflow before casting
    if nodecount > i32::MAX as u32 {
        return Err(SP_FORMERROR);
    }
    let len = nodecount as i32;

    // Check array sizes
    if byts.len() < nodecount as usize || idxs.len() < nodecount as usize {
        return Err(SP_FORMERROR);
    }

    // Create reader starting after the node count
    let mut reader = TreeReader::new(&buf[header_consumed..]);

    // Read the tree recursively
    let result = read_tree_node(&mut reader, byts, idxs, len, 0, prefixtree, prefixcnt);

    if result < 0 {
        return Err(result);
    }

    Ok((header_consumed + reader.position(), result))
}

/// FFI wrapper for reading a word tree.
///
/// # Safety
/// All pointers must be valid. `byts` and `idxs` must have at least `array_len` elements.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_read_tree(
    buf: *const u8,
    buf_len: usize,
    byts: *mut u8,
    idxs: *mut i32,
    array_len: usize,
    prefixtree: bool,
    prefixcnt: c_int,
    bytes_consumed_out: *mut usize,
    node_count_out: *mut i32,
) -> c_int {
    if buf.is_null()
        || byts.is_null()
        || idxs.is_null()
        || bytes_consumed_out.is_null()
        || node_count_out.is_null()
    {
        return SP_OTHERERROR;
    }

    let buf_slice = std::slice::from_raw_parts(buf, buf_len);
    let byts_slice = std::slice::from_raw_parts_mut(byts, array_len);
    let idxs_slice = std::slice::from_raw_parts_mut(idxs, array_len);

    match read_tree(buf_slice, byts_slice, idxs_slice, prefixtree, prefixcnt) {
        Ok((consumed, count)) => {
            *bytes_consumed_out = consumed;
            *node_count_out = count;
            0
        }
        Err(e) => e,
    }
}

/// Read tree data, allocating output arrays.
///
/// This is the main entry point for tree reading that handles allocation.
/// Returns a result with allocated byte and index arrays.
///
/// # Safety
/// This function is safe but calls unsafe FFI functions internally.
#[allow(clippy::type_complexity)]
pub fn read_tree_alloc(
    buf: &[u8],
    prefixtree: bool,
    prefixcnt: i32,
) -> Result<(Vec<u8>, Vec<i32>, usize), c_int> {
    // First read the node count
    let (nodecount, _) = read_tree_node_count(buf)?;

    if nodecount == 0 {
        return Ok((Vec::new(), Vec::new(), 4));
    }

    // Allocate arrays
    let mut byts = vec![0u8; nodecount as usize];
    let mut idxs = vec![0i32; nodecount as usize];

    // Read the tree
    let (consumed, _count) = read_tree(buf, &mut byts, &mut idxs, prefixtree, prefixcnt)?;

    Ok((byts, idxs, consumed))
}

/// FFI function to get the node count from tree header without reading the tree.
///
/// # Safety
/// `buf` must be valid for `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_read_tree_peek_nodecount(
    buf: *const u8,
    buf_len: usize,
    nodecount_out: *mut u32,
) -> c_int {
    if buf.is_null() || nodecount_out.is_null() {
        return SP_OTHERERROR;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match read_tree_node_count(slice) {
        Ok((count, _)) => {
            *nodecount_out = count;
            0
        }
        Err(e) => e,
    }
}

// =============================================================================
// Spell File Loader State
// =============================================================================

/// State for spell file loading.
///
/// This struct tracks progress while loading a spell file, allowing
/// incremental parsing of sections.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct SpellLoadState {
    /// Current offset in the file buffer
    pub offset: usize,
    /// Total buffer length
    pub buf_len: usize,
    /// Number of regions found
    pub region_count: u8,
    /// Flags from sections
    pub flags: u32,
    /// Error code (0 = no error)
    pub error: c_int,
    /// Whether we've seen the end section marker
    pub sections_done: bool,
}

impl SpellLoadState {
    /// Create a new load state for a buffer.
    pub const fn new(buf_len: usize) -> Self {
        Self {
            offset: 0,
            buf_len,
            region_count: 0,
            flags: 0,
            error: 0,
            sections_done: false,
        }
    }

    /// Check if there are more bytes to read.
    pub const fn has_more(&self) -> bool {
        self.offset < self.buf_len && self.error == 0
    }
}

/// FFI function to create a new spell load state.
#[no_mangle]
pub extern "C" fn rs_spell_load_state_new(buf_len: usize) -> SpellLoadState {
    SpellLoadState::new(buf_len)
}

/// FFI function to check if a load state has an error.
///
/// # Safety
/// `state` must be a valid pointer to a `SpellLoadState`.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_load_state_has_error(state: *const SpellLoadState) -> bool {
    if state.is_null() {
        return true;
    }
    (*state).error != 0
}

/// FFI function to get the error code from a load state.
///
/// # Safety
/// `state` must be a valid pointer to a `SpellLoadState`.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_load_state_get_error(state: *const SpellLoadState) -> c_int {
    if state.is_null() {
        return SP_OTHERERROR;
    }
    (*state).error
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // File Header Tests
    // =========================================================================

    #[test]
    fn test_parse_spellfile_header() {
        // Valid header
        let mut buf = [0u8; 10];
        buf[0..8].copy_from_slice(VIMSPELLMAGIC);
        buf[8] = VIMSPELLVERSION;

        let (header, consumed) = parse_spellfile_header(&buf).unwrap();
        assert_eq!(header.magic, *VIMSPELLMAGIC);
        assert_eq!(header.version, VIMSPELLVERSION);
        assert_eq!(consumed, 9);

        // Too short
        assert!(parse_spellfile_header(&[0; 8]).is_none());
    }

    #[test]
    fn test_validate_spellfile_header() {
        // Valid header
        let header = SpellFileHeader::default();
        assert_eq!(validate_spellfile_header(&header), HeaderValidation::Valid);

        // Bad magic
        let header = SpellFileHeader {
            magic: *b"BADMAGIC",
            version: VIMSPELLVERSION,
        };
        assert_eq!(
            validate_spellfile_header(&header),
            HeaderValidation::BadMagic
        );

        // Old version
        let header = SpellFileHeader {
            magic: *VIMSPELLMAGIC,
            version: VIMSPELLVERSION - 1,
        };
        assert_eq!(
            validate_spellfile_header(&header),
            HeaderValidation::OldVersion
        );

        // New version
        let header = SpellFileHeader {
            magic: *VIMSPELLMAGIC,
            version: VIMSPELLVERSION + 1,
        };
        assert_eq!(
            validate_spellfile_header(&header),
            HeaderValidation::NewVersion
        );
    }

    #[test]
    fn test_write_spellfile_header() {
        let mut buf = [0u8; 20];
        let header = SpellFileHeader::default();

        let written = write_spellfile_header(&mut buf, &header).unwrap();
        assert_eq!(written, 9);
        assert_eq!(&buf[0..8], VIMSPELLMAGIC);
        assert_eq!(buf[8], VIMSPELLVERSION);
    }

    #[test]
    fn test_roundtrip_spellfile_header() {
        let mut buf = [0u8; 20];
        let original = SpellFileHeader::default();

        let written = write_spellfile_header(&mut buf, &original).unwrap();
        let (parsed, consumed) = parse_spellfile_header(&buf).unwrap();

        assert_eq!(written, consumed);
        assert_eq!(parsed.magic, original.magic);
        assert_eq!(parsed.version, original.version);
    }

    #[test]
    fn test_section_ids() {
        // Verify section IDs match expected values
        assert_eq!(SN_REGION, 0);
        assert_eq!(SN_CHARFLAGS, 1);
        assert_eq!(SN_MIDWORD, 2);
        assert_eq!(SN_PREFCOND, 3);
        assert_eq!(SN_REP, 4);
        assert_eq!(SN_SAL, 5);
        assert_eq!(SN_SOFO, 6);
        assert_eq!(SN_MAP, 7);
        assert_eq!(SN_COMPOUND, 8);
        assert_eq!(SN_SYLLABLE, 9);
        assert_eq!(SN_NOBREAK, 10);
        assert_eq!(SN_SUGFILE, 11);
        assert_eq!(SN_REPSAL, 12);
        assert_eq!(SN_WORDS, 13);
        assert_eq!(SN_NOSPLITSUGS, 14);
        assert_eq!(SN_INFO, 15);
        assert_eq!(SN_NOCOMPOUNDSUGS, 16);
        assert_eq!(SN_END, 255);
    }

    // =========================================================================
    // Section Header Tests
    // =========================================================================

    #[test]
    fn test_parse_section_header() {
        // Section with data
        let buf = [0x01, 0x00, 0x00, 0x00, 0x00, 0x10]; // ID=1, flags=0, len=16
        let (header, consumed) = parse_section_header(&buf).unwrap();
        assert_eq!(header.section_id, 1);
        assert_eq!(header.flags, 0);
        assert_eq!(header.len, 16);
        assert_eq!(consumed, 6);

        // Section end
        let buf = [255];
        let (header, consumed) = parse_section_header(&buf).unwrap();
        assert_eq!(header.section_id, 255);
        assert_eq!(consumed, 1);

        // Required section
        let buf = [0x05, SNF_REQUIRED, 0x00, 0x00, 0x01, 0x00]; // len=256
        let (header, _) = parse_section_header(&buf).unwrap();
        assert_eq!(header.flags, SNF_REQUIRED);
        assert_eq!(header.len, 256);

        // Too short
        assert!(parse_section_header(&[0x01, 0x00]).is_none());
    }

    #[test]
    fn test_parse_region_section() {
        // Two regions: "en" and "au"
        let buf = b"enau";
        let mut output = [0u8; 20];
        let count = parse_region_section(buf, &mut output).unwrap();
        assert_eq!(count, 2);
        assert_eq!(&output[..4], b"enau");
        assert_eq!(output[4], 0); // NUL terminated

        // Empty
        let buf = b"";
        let count = parse_region_section(buf, &mut output).unwrap();
        assert_eq!(count, 0);

        // Odd length (error)
        let buf = b"enauss"; // 6 bytes, ok
        assert!(parse_region_section(buf, &mut output).is_ok());

        let buf = b"ena"; // 3 bytes, error
        assert!(parse_region_section(buf, &mut output).is_err());
    }

    #[test]
    fn test_read_be() {
        assert_eq!(read_be_u16(&[0x01, 0x02]), Some(0x0102));
        assert_eq!(read_be_u24(&[0x01, 0x02, 0x03]), Some(0x0001_0203));
        assert_eq!(read_be_u32(&[0x01, 0x02, 0x03, 0x04]), Some(0x0102_0304));
        assert_eq!(
            read_be_u64(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            Some(0x0102_0304_0506_0708)
        );

        // Too short
        assert_eq!(read_be_u16(&[0x01]), None);
        assert_eq!(read_be_u32(&[0x01, 0x02]), None);
    }

    #[test]
    fn test_read_cnt_string() {
        // 1-byte prefix
        let buf = [3, b'a', b'b', b'c', b'x'];
        let (s, consumed) = read_cnt_string_1(&buf).unwrap();
        assert_eq!(s, b"abc");
        assert_eq!(consumed, 4);

        // 2-byte prefix
        let buf = [0, 3, b'a', b'b', b'c', b'x'];
        let (s, consumed) = read_cnt_string_2(&buf).unwrap();
        assert_eq!(s, b"abc");
        assert_eq!(consumed, 5);

        // Empty string
        let buf = [0];
        let (s, consumed) = read_cnt_string_1(&buf).unwrap();
        assert_eq!(s, b"");
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_parse_rep_item() {
        // "abc" -> "xyz"
        let buf = [3, b'a', b'b', b'c', 3, b'x', b'y', b'z'];
        let (item, consumed) = parse_rep_item(&buf).unwrap();
        assert_eq!(item.from_len, 3);
        assert_eq!(&item.from[..3], b"abc");
        assert_eq!(item.to_len, 3);
        assert_eq!(&item.to[..3], b"xyz");
        assert_eq!(consumed, 8);

        // Empty from
        let buf = [0, 2, b'a', b'b'];
        let (item, _) = parse_rep_item(&buf).unwrap();
        assert_eq!(item.from_len, 0);
        assert_eq!(item.to_len, 2);
    }

    #[test]
    fn test_parse_sal_header() {
        let buf = [SAL_F0LLOWUP | SAL_COLLAPSE, 0x00, 0x10]; // flags, count=16
        let (header, consumed) = parse_sal_header(&buf).unwrap();
        assert_eq!(header.flags, SAL_F0LLOWUP | SAL_COLLAPSE);
        assert_eq!(header.count, 16);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_parse_compound_header() {
        let buf = [
            3, // compmax
            2, // compminlen
            4, // compsylmax
            0x00, 0x05, // compoptions
            0x00, 0x03, // comppatcount
        ];
        let (header, consumed) = parse_compound_header(&buf).unwrap();
        assert_eq!(header.compmax, 3);
        assert_eq!(header.compminlen, 2);
        assert_eq!(header.compsylmax, 4);
        assert_eq!(header.compoptions, 5);
        assert_eq!(header.comppatcount, 3);
        assert_eq!(consumed, 7);
    }

    #[test]
    fn test_parse_tree_nodecount() {
        let buf = [0x00, 0x01, 0x00, 0x00]; // 65536
        let (count, consumed) = parse_tree_nodecount(&buf).unwrap();
        assert_eq!(count, 0x0001_0000);
        assert_eq!(consumed, 4);
    }

    // Writing tests

    #[test]
    fn test_write_be() {
        let mut buf = [0u8; 8];

        assert_eq!(write_be_u16(&mut buf, 0x0102), Some(2));
        assert_eq!(&buf[..2], &[0x01, 0x02]);

        assert_eq!(write_be_u24(&mut buf, 0x0001_0203), Some(3));
        assert_eq!(&buf[..3], &[0x01, 0x02, 0x03]);

        assert_eq!(write_be_u32(&mut buf, 0x0102_0304), Some(4));
        assert_eq!(&buf[..4], &[0x01, 0x02, 0x03, 0x04]);

        assert_eq!(write_be_u64(&mut buf, 0x0102_0304_0506_0708), Some(8));
        assert_eq!(&buf, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn test_write_cnt_string() {
        let mut buf = [0u8; 10];

        // 1-byte prefix
        let written = write_cnt_string_1(&mut buf, b"abc").unwrap();
        assert_eq!(written, 4);
        assert_eq!(&buf[..4], &[3, b'a', b'b', b'c']);

        // 2-byte prefix
        let written = write_cnt_string_2(&mut buf, b"abc").unwrap();
        assert_eq!(written, 5);
        assert_eq!(&buf[..5], &[0, 3, b'a', b'b', b'c']);

        // Empty
        let written = write_cnt_string_1(&mut buf, b"").unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0);
    }

    #[test]
    fn test_write_section_header() {
        let mut buf = [0u8; 10];

        // Normal section
        let header = SectionHeader {
            section_id: 1,
            flags: SNF_REQUIRED,
            len: 256,
        };
        let written = write_section_header(&mut buf, &header).unwrap();
        assert_eq!(written, 6);
        assert_eq!(&buf[..6], &[0x01, SNF_REQUIRED, 0x00, 0x00, 0x01, 0x00]);

        // End marker
        let header = SectionHeader {
            section_id: 255,
            flags: 0,
            len: 0,
        };
        let written = write_section_header(&mut buf, &header).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 255);
    }

    #[test]
    fn test_write_rep_item() {
        let mut buf = [0u8; 20];

        let mut item = FromTo::default();
        item.from[..3].copy_from_slice(b"abc");
        item.from_len = 3;
        item.to[..3].copy_from_slice(b"xyz");
        item.to_len = 3;

        let written = write_rep_item(&mut buf, &item).unwrap();
        assert_eq!(written, 8);
        assert_eq!(&buf[..8], &[3, b'a', b'b', b'c', 3, b'x', b'y', b'z']);
    }

    #[test]
    fn test_write_sal_header() {
        let mut buf = [0u8; 10];

        let header = SalHeader {
            flags: SAL_F0LLOWUP | SAL_COLLAPSE,
            count: 16,
        };
        let written = write_sal_header(&mut buf, &header).unwrap();
        assert_eq!(written, 3);
        assert_eq!(&buf[..3], &[SAL_F0LLOWUP | SAL_COLLAPSE, 0x00, 0x10]);
    }

    #[test]
    fn test_write_compound_header() {
        let mut buf = [0u8; 10];

        let header = CompoundHeader {
            compmax: 3,
            compminlen: 2,
            compsylmax: 4,
            compoptions: 5,
            comppatcount: 3,
        };
        let written = write_compound_header(&mut buf, &header).unwrap();
        assert_eq!(written, 7);
        assert_eq!(&buf[..7], &[3, 2, 4, 0x00, 0x05, 0x00, 0x03]);
    }

    #[test]
    fn test_write_tree_nodecount() {
        let mut buf = [0u8; 10];

        let written = write_tree_nodecount(&mut buf, 0x0001_0000).unwrap();
        assert_eq!(written, 4);
        assert_eq!(&buf[..4], &[0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_write_timestamp() {
        let mut buf = [0u8; 10];

        let written = write_timestamp(&mut buf, 0x0102_0304_0506_0708).unwrap();
        assert_eq!(written, 8);
        assert_eq!(&buf[..8], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn test_roundtrip_section_header() {
        let mut buf = [0u8; 10];

        let original = SectionHeader {
            section_id: 5,
            flags: SNF_REQUIRED,
            len: 1234,
        };
        let written = write_section_header(&mut buf, &original).unwrap();
        let (parsed, consumed) = parse_section_header(&buf).unwrap();

        assert_eq!(written, consumed);
        assert_eq!(parsed.section_id, original.section_id);
        assert_eq!(parsed.flags, original.flags);
        assert_eq!(parsed.len, original.len);
    }

    #[test]
    fn test_roundtrip_rep_item() {
        let mut buf = [0u8; 520];

        let mut original = FromTo::default();
        original.from[..5].copy_from_slice(b"hello");
        original.from_len = 5;
        original.to[..5].copy_from_slice(b"world");
        original.to_len = 5;

        let written = write_rep_item(&mut buf, &original).unwrap();
        let (parsed, consumed) = parse_rep_item(&buf).unwrap();

        assert_eq!(written, consumed);
        assert_eq!(parsed.from_len, original.from_len);
        assert_eq!(parsed.to_len, original.to_len);
        assert_eq!(&parsed.from[..5], b"hello");
        assert_eq!(&parsed.to[..5], b"world");
    }

    // =========================================================================
    // Phase 6: Tree Node Writing Tests
    // =========================================================================

    #[test]
    fn test_tree_node_flags_default() {
        let flags = TreeNodeFlags::new();
        assert!(!flags.has_flags());
        assert_eq!(flags.encoded_len(), 1);
    }

    #[test]
    fn test_tree_node_flags_with_region() {
        let mut flags = TreeNodeFlags::new();
        flags.region = 0x05;
        assert!(flags.has_flags());
        assert_eq!(flags.encoded_len(), 3); // BY_FLAGS + flags_byte + region
    }

    #[test]
    fn test_write_tree_node_flags_no_flags() {
        let mut buf = [0u8; 10];
        let flags = TreeNodeFlags::new();
        let written = write_tree_node_flags(&mut buf, &flags).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], tree_bytes::BY_NOFLAGS);
    }

    #[test]
    fn test_write_tree_node_flags_with_region() {
        let mut buf = [0u8; 10];
        let mut flags = TreeNodeFlags::new();
        flags.region = 0x05;
        let written = write_tree_node_flags(&mut buf, &flags).unwrap();
        assert_eq!(written, 3);
        assert_eq!(buf[0], tree_bytes::BY_FLAGS);
        assert_eq!(buf[1] & 0x02, 0x02); // WF_REGION set
        assert_eq!(buf[2], 0x05);
    }

    #[test]
    fn test_write_tree_sibling_byte_regular() {
        let mut buf = [0u8; 5];

        // Regular character byte
        let written = write_tree_sibling_byte(&mut buf, b'a').unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], b'a');
    }

    #[test]
    fn test_write_tree_sibling_byte_any_value() {
        let mut buf = [0u8; 5];

        // Any byte value is written directly (no escaping)
        let written = write_tree_sibling_byte(&mut buf, 0xFE).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0xFE);

        // Even "special" byte values are written directly for character nodes
        let written = write_tree_sibling_byte(&mut buf, tree_bytes::BY_FLAGS).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], tree_bytes::BY_FLAGS);
    }

    #[test]
    fn test_write_tree_child_index_one_byte() {
        let mut buf = [0u8; 10];

        let written = write_tree_child_index(&mut buf, 0x7F).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x7F);

        let written = write_tree_child_index(&mut buf, 0x00).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x00);
    }

    #[test]
    fn test_write_tree_child_index_two_bytes() {
        let mut buf = [0u8; 10];

        let written = write_tree_child_index(&mut buf, 0x80).unwrap();
        assert_eq!(written, 2);
        assert_eq!(buf[0] & 0x80, 0x80); // High bit set

        let written = write_tree_child_index(&mut buf, 0x7FFF).unwrap();
        assert_eq!(written, 2);
    }

    #[test]
    fn test_write_tree_child_index_three_bytes() {
        let mut buf = [0u8; 10];

        let written = write_tree_child_index(&mut buf, 0x8000).unwrap();
        assert_eq!(written, 3);
        assert_eq!(buf[0] & 0xC0, 0xC0); // 3-byte marker
    }

    #[test]
    fn test_write_tree_child_index_four_bytes() {
        let mut buf = [0u8; 10];

        let written = write_tree_child_index(&mut buf, 0x80_0000).unwrap();
        assert_eq!(written, 4);
        assert_eq!(buf[0] & 0xE0, 0xE0); // 4-byte marker
    }

    #[test]
    fn test_write_tree_child_index_too_large() {
        let mut buf = [0u8; 10];

        // Index > 0x7FFFFFFF should fail
        assert!(write_tree_child_index(&mut buf, 0x8000_0000).is_none());
    }

    #[test]
    fn test_write_region_section() {
        let mut buf = [0u8; 20];
        let regions = b"enus";

        let written = write_region_section(&mut buf, regions).unwrap();
        assert!(written > 4);
        // First byte should be section ID
        assert_eq!(buf[0], SN_REGION);
    }

    #[test]
    fn test_write_region_section_invalid() {
        let mut buf = [0u8; 20];

        // Odd length (not pairs)
        assert!(write_region_section(&mut buf, b"abc").is_none());

        // Too long
        let long_regions = b"enusgbdefritesfrptplnl"; // 22 bytes > 16
        assert!(write_region_section(&mut buf, long_regions).is_none());
    }

    #[test]
    fn test_write_end_section() {
        let mut buf = [0u8; 5];

        let written = write_end_section(&mut buf).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], SN_END);
    }

    #[test]
    fn test_spell_file_write_info() {
        let info = SpellFileWriteInfo::new();
        assert_eq!(info.region_count, 0);
        assert!(!info.is_addition);
        assert!(!info.has_regions());

        let mut info2 = info;
        info2.region_count = 3;
        assert!(info2.has_regions());
    }

    // =========================================================================
    // Prefix Condition Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_prefcond_empty() {
        // Empty condition
        let buf = [0u8; 1]; // condlen = 0
        let (cond, consumed) = parse_prefcond(&buf).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(cond.pattern_len, 0);
    }

    #[test]
    fn test_parse_prefcond_with_pattern() {
        // Condition with pattern "abc"
        let buf = [3, b'a', b'b', b'c'];
        let (cond, consumed) = parse_prefcond(&buf).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(cond.pattern_len, 3);
        assert_eq!(&cond.pattern[..3], b"abc");
    }

    #[test]
    fn test_parse_prefcond_truncated() {
        // Buffer too short
        let buf = [3, b'a']; // Says 3 bytes but only 1
        assert_eq!(parse_prefcond(&buf).unwrap_err(), SP_TRUNCERROR);
    }

    #[test]
    fn test_parse_prefcond_count() {
        let buf = [0x00, 0x05]; // 5 conditions
        let (count, consumed) = parse_prefcond_count(&buf).unwrap();
        assert_eq!(count, 5);
        assert_eq!(consumed, 2);
    }

    // =========================================================================
    // REP Section Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_rep_count() {
        let buf = [0x00, 0x0A]; // 10 items
        let (count, consumed) = parse_rep_count(&buf).unwrap();
        assert_eq!(count, 10);
        assert_eq!(consumed, 2);
    }

    // =========================================================================
    // SAL Item Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_sal_item_basic() {
        // Simple SAL item: "a" -> "b" (no special chars, so lead="a", no oneof, rules="")
        let buf = [1, b'a', 1, b'b'];
        let (item, consumed) = parse_sal_item(&buf).unwrap();
        assert_eq!(consumed, 4);
        // lead is "a\0" at offset 0
        assert_eq!(item.lead_len, 1);
        assert_eq!(item.from[0], b'a');
        assert_eq!(item.from[1], 0); // NUL terminator
        assert_eq!(item.oneof_offset, 0xFFFF); // no oneof
        assert_eq!(item.to_len, 1);
        assert_eq!(item.to[0], b'b');
        assert!(item.has_to);
    }

    #[test]
    fn test_parse_sal_item_empty_to() {
        // SAL item with empty "to": "abc" -> "" (no special chars, lead="abc")
        let buf = [3, b'a', b'b', b'c', 0];
        let (item, consumed) = parse_sal_item(&buf).unwrap();
        assert_eq!(consumed, 5);
        assert_eq!(item.lead_len, 3);
        assert_eq!(item.to_len, 0);
        assert!(!item.has_to);
    }

    #[test]
    fn test_parse_sal_item_with_special() {
        // SAL item: "ab1c" -> "x" -- '1' is a special char, so lead="ab", rules="1c"
        let buf = [4, b'a', b'b', b'1', b'c', 1, b'x'];
        let (item, consumed) = parse_sal_item(&buf).unwrap();
        assert_eq!(consumed, 7);
        assert_eq!(item.lead_len, 2); // "ab"
        assert_eq!(item.from[0], b'a');
        assert_eq!(item.from[1], b'b');
        assert_eq!(item.oneof_offset, 0xFFFF); // no oneof
        assert_eq!(item.to_len, 1);
        assert_eq!(item.to[0], b'x');
    }

    // =========================================================================
    // Midword Section Tests
    // =========================================================================

    #[test]
    fn test_parse_midword_section() {
        let input = b"'-";
        let mut output = [0u8; 10];
        let len = parse_midword_section(input, &mut output).unwrap();
        assert_eq!(len, 2);
        assert_eq!(&output[..2], b"'-");
        assert_eq!(output[2], 0); // NUL-terminated
    }

    // =========================================================================
    // MAP Section Tests
    // =========================================================================

    #[test]
    fn test_parse_map_section() {
        let input = b"aA/eE/iI";
        let mut output = [0u8; 20];
        let len = parse_map_section(input, &mut output).unwrap();
        assert_eq!(len, 8);
        assert_eq!(&output[..8], b"aA/eE/iI");
    }

    // =========================================================================
    // Words Section Tests
    // =========================================================================

    #[test]
    fn test_parse_words_entry() {
        let buf = b"hello\0world\0";
        let (word, consumed) = parse_words_entry(buf).unwrap();
        assert_eq!(word, b"hello");
        assert_eq!(consumed, 6); // "hello" + NUL
    }

    #[test]
    fn test_parse_words_entry_truncated() {
        let buf = b"hello"; // No NUL terminator
        assert_eq!(parse_words_entry(buf).unwrap_err(), SP_TRUNCERROR);
    }

    // =========================================================================
    // Tree Node Count Tests
    // =========================================================================

    #[test]
    fn test_read_tree_node_count() {
        let buf = [0x00, 0x01, 0x00, 0x00]; // 65536
        let (count, consumed) = read_tree_node_count(&buf).unwrap();
        assert_eq!(count, 0x0001_0000);
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_read_tree_node_count_truncated() {
        let buf = [0x00, 0x01]; // Only 2 bytes
        assert_eq!(read_tree_node_count(&buf).unwrap_err(), SP_TRUNCERROR);
    }

    // =========================================================================
    // Spell Load State Tests
    // =========================================================================

    #[test]
    fn test_spell_load_state_new() {
        let state = SpellLoadState::new(1024);
        assert_eq!(state.offset, 0);
        assert_eq!(state.buf_len, 1024);
        assert_eq!(state.error, 0);
        assert!(!state.sections_done);
        assert!(state.has_more());
    }

    #[test]
    fn test_spell_load_state_has_more() {
        let mut state = SpellLoadState::new(100);
        assert!(state.has_more());

        state.offset = 100;
        assert!(!state.has_more());

        state.offset = 50;
        state.error = SP_FORMERROR;
        assert!(!state.has_more()); // Error stops further reading
    }

    // =========================================================================
    // Tree Reading Tests (Phase 329-330)
    // =========================================================================

    #[test]
    fn test_tree_reader_basic() {
        let buf = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut reader = TreeReader::new(&buf);

        assert_eq!(reader.position(), 0);
        assert_eq!(reader.getc(), Some(0x01));
        assert_eq!(reader.position(), 1);
        assert_eq!(reader.getc(), Some(0x02));
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn test_tree_reader_get2c() {
        let buf = [0x01, 0x02, 0x03, 0x04];
        let mut reader = TreeReader::new(&buf);

        let val = reader.get2c().unwrap();
        assert_eq!(val, 0x0102);
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn test_tree_reader_get3c() {
        let buf = [0x01, 0x02, 0x03, 0x04];
        let mut reader = TreeReader::new(&buf);

        let val = reader.get3c().unwrap();
        assert_eq!(val, 0x0001_0203);
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn test_tree_reader_eof() {
        let buf = [0x01];
        let mut reader = TreeReader::new(&buf);

        assert_eq!(reader.getc(), Some(0x01));
        assert_eq!(reader.getc(), None); // EOF
    }

    #[test]
    fn test_read_tree_empty() {
        // Empty tree: node count = 0
        let buf = [0x00, 0x00, 0x00, 0x00];
        let mut byts = [0u8; 10];
        let mut idxs = [0i32; 10];

        let (consumed, count) = read_tree(&buf, &mut byts, &mut idxs, false, 0).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_read_tree_single_node_no_flags() {
        // Tree with 1 sibling, no flags, end-of-word
        // Format: <nodecount 4B> <siblingcount 1B> <BY_NOFLAGS 1B>
        #[rustfmt::skip]
        let buf = [
            0x00, 0x00, 0x00, 0x02, // nodecount = 2
            0x01,                   // siblingcount = 1
            tree_bytes::BY_NOFLAGS, // BY_NOFLAGS = end of word, no flags
        ];
        let mut byts = [0u8; 10];
        let mut idxs = [0i32; 10];

        let (consumed, count) = read_tree(&buf, &mut byts, &mut idxs, false, 0).unwrap();
        assert_eq!(consumed, 6);
        assert_eq!(count, 2);
        assert_eq!(byts[0], 1); // siblingcount stored in first position
        assert_eq!(byts[1], 0); // end-of-word marker
        assert_eq!(idxs[1], 0); // no flags
    }

    #[test]
    fn test_read_tree_simple_word() {
        // Tree representing a simple word like "a"
        // Structure: root has 1 sibling 'a', which has 1 child that is end-of-word
        //
        // Layout in buffer:
        // [0-3] nodecount = 4 (header, 4 bytes)
        // [4] siblingcount = 1 for root
        // [5] 'a' = 0x61 character byte
        // [6] siblingcount = 1 for child
        // [7] BY_NOFLAGS = 0 (end of word marker)
        //
        // Total: 4 + 1 + 1 + 1 + 1 = 8 bytes
        #[rustfmt::skip]
        let buf = [
            0x00, 0x00, 0x00, 0x04, // nodecount = 4
            0x01,                   // siblingcount = 1
            b'a',                   // character 'a' (> BY_SPECIAL, so regular char)
            // Child node for 'a':
            0x01,                   // siblingcount = 1
            tree_bytes::BY_NOFLAGS, // end of word (0)
        ];
        let mut byts = [0u8; 10];
        let mut idxs = [0i32; 10];

        let (consumed, count) = read_tree(&buf, &mut byts, &mut idxs, false, 0).unwrap();
        assert_eq!(consumed, 8);
        assert_eq!(count, 4);

        // Verify structure
        assert_eq!(byts[0], 1); // root siblingcount
        assert_eq!(byts[1], b'a'); // character
        assert_eq!(idxs[1], 2); // child index
        assert_eq!(byts[2], 1); // child siblingcount
        assert_eq!(byts[3], 0); // end-of-word marker
    }

    #[test]
    fn test_read_tree_with_flags() {
        // Tree with a word that has flags set (WF_REGION)
        #[rustfmt::skip]
        let buf = [
            0x00, 0x00, 0x00, 0x02,       // nodecount = 2
            0x01,                         // siblingcount = 1
            tree_bytes::BY_FLAGS,         // has flags
            word_flags::WF_REGION as u8,  // flags byte with region bit
            0x05,                         // region = 5
        ];
        let mut byts = [0u8; 10];
        let mut idxs = [0i32; 10];

        let (consumed, count) = read_tree(&buf, &mut byts, &mut idxs, false, 0).unwrap();
        assert_eq!(consumed, 8);
        assert_eq!(count, 2);

        // Check flags were stored correctly
        // idxs should have: flags in low bytes, region shifted up
        let stored = idxs[1];
        assert_eq!(stored & 0xFF, word_flags::WF_REGION); // flags
        assert_eq!((stored >> 16) & 0xFF, 0x05); // region
    }

    #[test]
    fn test_read_tree_truncated() {
        // Tree with incomplete data
        #[rustfmt::skip]
        let buf = [
            0x00, 0x00, 0x00, 0x10, // nodecount = 16
            0x05,                   // siblingcount = 5 (Missing sibling data!)
        ];
        let mut byts = [0u8; 20];
        let mut idxs = [0i32; 20];

        let result = read_tree(&buf, &mut byts, &mut idxs, false, 0);
        assert_eq!(result.unwrap_err(), SP_TRUNCERROR);
    }

    #[test]
    fn test_read_tree_alloc_empty() {
        let buf = [0x00, 0x00, 0x00, 0x00];

        let (byts, idxs, consumed) = read_tree_alloc(&buf, false, 0).unwrap();
        assert!(byts.is_empty());
        assert!(idxs.is_empty());
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_read_tree_alloc_simple() {
        // Simple tree with 2 nodes
        #[rustfmt::skip]
        let buf = [
            0x00, 0x00, 0x00, 0x02, // nodecount = 2
            0x01,                   // siblingcount = 1
            tree_bytes::BY_NOFLAGS, // end of word
        ];

        let (byts, idxs, consumed) = read_tree_alloc(&buf, false, 0).unwrap();
        assert_eq!(byts.len(), 2);
        assert_eq!(idxs.len(), 2);
        assert_eq!(consumed, 6);
        assert_eq!(byts[0], 1); // siblingcount
    }

    #[test]
    fn test_word_flags_constants() {
        // Verify flag constants match expected values
        assert_eq!(word_flags::WF_REGION, 0x02);
        assert_eq!(word_flags::WF_AFX, 0x04);
        assert_eq!(word_flags::WF_PFX, 0x08);
    }
}
